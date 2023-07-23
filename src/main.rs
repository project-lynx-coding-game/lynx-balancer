mod instance_host;

use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use clap::Parser;

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
    
    HttpServer::new(|| App::new().service(echo))
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
