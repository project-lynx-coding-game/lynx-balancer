mod instance_host;

use crate::instance_host::{InstanceHost, Instance};
use crate::instance_host::kubernetes_host::KubernetesHost;

use actix_web::{web, post, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use serde::{Deserialize, Serialize};

struct AppState {
    instance_host: Box<dyn InstanceHost>,
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
   port: u16
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    
    HttpServer::new(||
        App::new()
            .app_data(web::Data::new(AppState {
                instance_host: Box::new(KubernetesHost::new()),
            }))
            .service(
                web::scope("/instance_host")
                .service(web::resource("/start").route(web::post().to(start_instance)))
                .service(web::resource("/stop").route(web::post().to(stop_instance)))
            )
            .service(echo)
    )
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
