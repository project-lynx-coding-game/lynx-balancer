mod cache_provider;
mod instance_host;

use crate::instance_host::kubernetes_host::KubernetesHost;
use crate::instance_host::{Instance, InstanceHost};

use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use cache_provider::local_cache::LocalCache;
use cache_provider::CacheProvider;
use clap::Parser;
use serde::{Deserialize, Serialize};

struct AppState {
    instance_host: Box<dyn InstanceHost>,
    url_cache: Box<dyn CacheProvider<String, String>>,
}

async fn start_instance(data: web::Data<AppState>) -> HttpResponse {
    data.instance_host.start_instance();
    HttpResponse::Ok().body("done")
}

async fn stop_instance(data: web::Data<AppState>, request: web::Json<Instance>) -> HttpResponse {
    data.instance_host.stop_instance(request.into_inner());
    HttpResponse::Ok().body("done")
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
    port: u16,
    /// Port for cache server
    cache_port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let balancer = HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                instance_host: Box::new(KubernetesHost::new()),
                url_cache: Box::new(LocalCache::new()),
            }))
            .service(
                web::scope("/instance_host")
                    .service(web::resource("/start").route(web::post().to(start_instance)))
                    .service(web::resource("/stop").route(web::post().to(stop_instance))),
            )
            .service(echo)
    })
    .bind(("127.0.0.1", args.port))?
    .run();

    let cache_server = HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                instance_host: Box::new(KubernetesHost::new()),
                url_cache: Box::new(LocalCache::new()),
            }))
            .service(web::resource("/start").route(web::post().to(start_instance)))
            .service(web::resource("/stop").route(web::post().to(stop_instance)))
    })
    .bind(("127.0.0.1", args.cache_port))?
    .run();

    futures::try_join!(balancer, cache_server)?;

    Ok(())
}
