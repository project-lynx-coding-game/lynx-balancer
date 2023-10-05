mod cache_provider;
mod instance_host;

use crate::instance_host::kubernetes_host::KubernetesHost;
use crate::instance_host::InstanceHost;

use actix_web::http::header::ContentType;
use actix_web::web::{Data, Bytes};
use actix_web::{web, App, HttpResponse, HttpServer, get, post};
use actix_proxy::{IntoHttpResponse};
use awc;
use cache_provider::local_cache::LocalCache;
use cache_provider::redis_cache::RedisCache;
use cache_provider::{CacheGetRequest, CacheProvider, CacheSetRequest};
use clap::{Parser, ValueEnum};
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use tracing::info;

struct AppState {
    // It's quite complex but Sync and Send traits mean
    // that the impl can be moved across threads
    // https://doc.rust-lang.org/nomicon/send-and-sync.html
    instance_host: Box<dyn InstanceHost + Sync + Send>,
    url_cache: Box<dyn CacheProvider<String, String> + Sync + Send>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserGetRequest {
    username: String,
}

async fn start_instance(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    let mut data = data.lock().await;
    let new_instance = data
        .instance_host
        .start_instance("test-user".to_string())
        .await;
    match new_instance {
        Ok(instance) => {
            data.url_cache
                .set("test-user".to_string(), instance.url.clone()).await;
            HttpResponse::Ok().body(instance.url)
        }
        Err(e) => {
            eprintln!("Error: {e}");
            HttpResponse::InternalServerError().body("Oh no error baby")
        }
    }
}

async fn stop_instance(
    data: web::Data<Mutex<AppState>>,
) -> HttpResponse {
    let data = data.lock().await;
    match data
        .instance_host
        .stop_instance("test-user".to_string())
        .await
    {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().body("Instance could not be stopped"),
    }
    HttpResponse::Ok().body("done")
}

async fn cache_get(
    data: web::Data<Mutex<AppState>>,
    info: web::Query<CacheGetRequest<String>>,
) -> HttpResponse {
    let mut data = data.lock().await;
    let result = data.url_cache.get(info.key.clone()).await;
    match result {
        Some(url) => HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(url.clone()),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn cache_set(
    data: web::Data<Mutex<AppState>>,
    info: web::Query<CacheSetRequest<String, String>>,
) -> HttpResponse {
    let mut data = data.lock().await;
    data.url_cache.set(info.key.clone(), info.value.clone()).await;
    HttpResponse::Ok().finish()
}

#[get("/{tail:.*}")]
async fn get_proxy(data: web::Data<Mutex<AppState>>, path: web::Path<String>, bytes: Bytes) -> HttpResponse {
    // TODO: unpacking username from http request will be different, it has to be planned out
    let mut data = data.lock().await;
    let url = data.url_cache.get("test-user".to_string()).await;
    if let Some(url) = url {
        let client = awc::Client::default();

        let final_url = "http://".to_owned() + &url + "/" + &path.into_inner();
        let res = client.get(final_url).send_body(bytes).await.unwrap();
        res.into_http_response()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("/{tail:.*}")]
async fn post_proxy(data: web::Data<Mutex<AppState>>, path: web::Path<String>, bytes: Bytes) -> HttpResponse {
    // TODO: unpacking username from http request will be different, it has to be planned out
    let mut data = data.lock().await;
    let url = data.url_cache.get("test-user".to_string()).await;
    if let Some(url) = url {
        let client = awc::Client::default();

        let final_url = "http://".to_owned() + &url + "/" + &path.into_inner();
        let res = client.post(final_url).send_body(bytes).await.unwrap();
        res.into_http_response()
    } else {
        HttpResponse::NotFound().finish()
    }
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
    #[arg(long, default_value = "redis://my-redis-master.lynx-balancer.svc.cluster.local:6379")]
    redis_url: String,

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
    LocalCache
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
        //url_cache: Box::new(LocalCache::new()),
        //TODO: investigate Handle::block_on because
        //I dont like having asyncronous new method
        url_cache: match args.cache {
            Cache::LocalCache => Box::new(LocalCache::new()),
            Cache::RedisCache => Box::new(RedisCache::new(args.redis_url).await)
        },
    }));

    let cache_server_data = data.clone();
    let proxy_data = data.clone();

    let balancer = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(
                web::scope("/instance")
                    .service(web::resource("/start").route(web::post().to(start_instance)))
                    .service(web::resource("/stop").route(web::post().to(stop_instance))),
            )
    })
    .bind(("0.0.0.0", args.port))?
    .run();

    let cache_server = HttpServer::new(move || {
        App::new().app_data(cache_server_data.clone()).service(
            web::scope("/cache")
                .service(web::resource("/get").route(web::get().to(cache_get)))
                .service(web::resource("/set").route(web::post().to(cache_set))),
        )
    })
    .bind(("0.0.0.0", args.cache_port))?
    .run();

    let proxy = HttpServer::new(move || {
        App::new()
            .app_data(proxy_data.clone())
            .service(get_proxy)
            .service(post_proxy)
    })
    .bind(("0.0.0.0", args.proxy_port))?
    .run();

    futures::try_join!(balancer, cache_server, proxy)?;

    Ok(())
}
