mod cache_provider;
mod instance_host;

use crate::instance_host::kubernetes_host::KubernetesHost;
use crate::instance_host::InstanceHost;

use actix_web::{web, App, HttpResponse, HttpServer};
use clap::Parser;
use std::cell::RefCell;

struct AppState {
    instance_host: RefCell<Box<dyn InstanceHost>>,
}

async fn start_instance(data: web::Data<AppState>) -> HttpResponse {
    let new_instance = data
        .instance_host
        .borrow_mut()
        .start_instance("test-user".to_string())
        .await;
    match new_instance {
        Ok(instance) => HttpResponse::Ok().body(instance.url),
        Err(e) => {
            eprintln!("Error: {e}");
            HttpResponse::InternalServerError().body("Oh no error baby")
        }
    }
}

async fn stop_instance(data: web::Data<AppState>) -> HttpResponse {
    match data
        .instance_host
        .borrow()
        .stop_instance("test-user".to_string())
        .await
    {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().body("Instance could not be stopped"),
    }
    HttpResponse::Ok().body("done")
}

/// Lynx balancer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port number
    #[arg(default_value_t = 8080)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => (),
        Err(_) => println!("ERROR tracing could not be enabled!"),
    }

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                instance_host: RefCell::new(Box::new(KubernetesHost::new())),
            }))
            .service(
                web::scope("/instance")
                    .service(web::resource("/start").route(web::post().to(start_instance)))
                    .service(web::resource("/stop").route(web::post().to(stop_instance))),
            )
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
