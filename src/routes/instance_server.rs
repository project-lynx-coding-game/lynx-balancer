use crate::AppState;

use actix_web::{web, HttpResponse};
use futures::lock::Mutex;

pub async fn start_instance(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    // TODO: add username and token to request body
    // TODO: validate token
    let mut data = data.lock().await;
    let new_instance = data
        .instance_host
        .start_instance("test-user".to_string())
        .await;
    match new_instance {
        Ok(instance) => {
            data.url_cache
                .set("test-user".to_string(), instance.url.clone())
                .await;
            HttpResponse::Ok().body(instance.url)
        }
        Err(e) => {
            eprintln!("Error: {e}");
            HttpResponse::InternalServerError().body("Oh no error baby")
        }
    }
}

pub async fn stop_instance(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    // TODO: add username and token to request body
    // TODO: validate token
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
