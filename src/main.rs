mod cache_provider;
mod instance_host;

use crate::instance_host::kubernetes_host::KubernetesHost;
use crate::instance_host::{Instance, InstanceHost};

use actix_web::http::header::ContentType;
use actix_web::web::Data;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use cache_provider::local_cache::LocalCache;
use cache_provider::{CacheGetRequest, CacheProvider, CacheSetRequest};
use clap::Parser;
use futures::lock::Mutex;

struct AppState {
    // It's quite complex but Sync and Send traits mean
    // that the impl can be moved across threads
    // https://doc.rust-lang.org/nomicon/send-and-sync.html
    instance_host: Box<dyn InstanceHost + Sync + Send>,
    url_cache: Box<dyn CacheProvider<String, String> + Sync + Send>,
}

async fn start_instance(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    let mut data = data.lock().await;
    data.instance_host.start_instance();
    HttpResponse::Ok().body("done")
}

async fn stop_instance(
    data: web::Data<Mutex<AppState>>,
    request: web::Json<Instance>,
) -> HttpResponse {
    let mut data = data.lock().await;
    data.instance_host.stop_instance(request.into_inner());
    HttpResponse::Ok().body("done")
}

async fn cache_get(
    data: web::Data<Mutex<AppState>>,
    info: web::Query<CacheGetRequest<String>>,
) -> HttpResponse {
    let data = data.lock().await;
    let result = data.url_cache.get(&info.key);
    match result {
        Some(url) => HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(url.clone()),
        None => HttpResponse::Ok().body("zamn"),
    }
}

async fn cache_set(
    data: web::Data<Mutex<AppState>>,
    info: web::Query<CacheSetRequest<String, String>>,
) -> HttpResponse {
    let mut data = data.lock().await;
    data.url_cache.set(info.key.clone(), info.value.clone());
    HttpResponse::Ok().finish()
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

/// Lynx balancer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port number
    #[arg(default_value_t = 8080)]
    port: u16,
    /// Port for cache server
    #[arg(default_value_t = 8081)]
    cache_port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let data = Data::new(Mutex::new(AppState {
        instance_host: Box::new(KubernetesHost::new()),
        url_cache: Box::new(LocalCache::new()),
    }));

    let cache_server_data = data.clone();

    let balancer = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(
                web::scope("/instance_host")
                    .service(web::resource("/start").route(web::post().to(start_instance)))
                    .service(web::resource("/stop").route(web::post().to(stop_instance))),
            )
            .service(echo)
    })
    .bind(("127.0.0.1", args.port))?
    .run();

    let cache_server = HttpServer::new(move || {
        App::new().app_data(cache_server_data.clone()).service(
            web::scope("/cache")
                .service(web::resource("/get").route(web::get().to(cache_get)))
                .service(web::resource("/set").route(web::post().to(cache_set))),
        )
    })
    .bind(("127.0.0.1", args.cache_port))?
    .run();

    futures::try_join!(balancer, cache_server)?;

    Ok(())
}
