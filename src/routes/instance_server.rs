use crate::AppState;

use actix_web::{web, HttpResponse};
use futures::lock::Mutex;

pub async fn start_instance(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    let mut data = data.lock().await;
    // TODO: if existing user, first stop previous instance

    let new_instance = data
        .instance_host
        .start_instance("test-user".to_string())
        .await;
    match new_instance {
        Ok(instance) => {
            data.url_cache
                .set("test-user".to_string(), instance.get_url_with_port())
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
    let mut data = data.lock().await;
    // TODO: save state of scene host?
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
