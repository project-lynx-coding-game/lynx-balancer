mod auth_manager;
mod cache_provider;
mod instance_host;
mod routes;

use crate::auth_manager::AuthManager;
use crate::instance_host::kubernetes_host::KubernetesHost;
use crate::instance_host::InstanceHost;
use crate::routes::{auth, cache_server, instance_server, proxy_server};

use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use cache_provider::local_cache::LocalCache;
use cache_provider::redis_cache::RedisCache;
use cache_provider::CacheProvider;
use clap::{Parser, ValueEnum};
use futures::lock::Mutex;
use tracing::info;

pub struct AppState {
    // It's quite complex but Sync and Send traits mean
    // that the impl can be moved across threads
    // https://doc.rust-lang.org/nomicon/send-and-sync.html
    instance_host: Box<dyn InstanceHost + Sync + Send>,
    auth_manager: Box<dyn AuthManager + Sync + Send>,
    url_cache: Box<dyn CacheProvider<String, String> + Sync + Send>,
    use_cache_query: bool,
}

/// Lynx balancer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port number
    #[arg(long, default_value_t = 8080)]
    port: u16,
    /// Port for cache server
    #[arg(long, default_value_t = 8081)]
    cache_port: u16,
    /// Port for proxy server
    #[arg(long, default_value_t = 8082)]
    proxy_port: u16,
    /// Not functional!!!
    #[arg(
        long,
        default_value = "redis://my-redis-master.lynx-balancer.svc.cluster.local:6379"
    )]
    redis_url: String,
    #[arg(long)]
    cache_query_url: Option<String>,

    #[arg(
        long,
        require_equals = true,
        num_args = 0..=1,
        default_value_t = Cache::LocalCache,
        value_enum
    )]
    cache: Cache,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Cache {
    RedisCache,
    LocalCache,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => (),
        Err(_) => println!("ERROR tracing could not be enabled!"),
    }

    info!("Preparing `instance_host` and `url_cache`");
    let data = Data::new(Mutex::new(AppState {
        instance_host: Box::new(KubernetesHost::new()),
        use_cache_query: args.cache_query_url.is_some(),
        //TODO: investigate Handle::block_on because
        //I dont like having asyncronous new method
        url_cache: match args.cache {
            Cache::LocalCache => Box::new(LocalCache::new(args.cache_query_url)),
            Cache::RedisCache => Box::new(RedisCache::new(args.redis_url).await),
        },
    }));

    let cache_server_data = data.clone();
    let proxy_data = data.clone();

    let balancer = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(
                web::scope("/instance")
                    .service(
                        web::resource("/start")
                            .route(web::post().to(instance_server::start_instance)),
                    )
                    .service(
                        web::resource("/stop")
                            .route(web::post().to(instance_server::stop_instance)),
                    ),
            )
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login)),
            )
    })
    .bind(("0.0.0.0", args.port))?
    .run();

    let cache_server = HttpServer::new(move || {
        App::new().app_data(cache_server_data.clone()).service(
            web::scope("/cache")
                .service(web::resource("/get").route(web::get().to(cache_server::cache_get)))
                .service(web::resource("/set").route(web::post().to(cache_server::cache_set))),
        )
    })
    .bind(("0.0.0.0", args.cache_port))?
    .run();

    let proxy = HttpServer::new(move || {
        App::new()
            .app_data(proxy_data.clone())
            .service(proxy_server::get_proxy)
            .service(proxy_server::post_proxy)
    })
    .bind(("0.0.0.0", args.proxy_port))?
    .run();

    futures::try_join!(balancer, cache_server, proxy)?;

    Ok(())
}
