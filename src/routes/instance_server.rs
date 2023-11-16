use crate::{AppState, auth_manager};

use actix_web::{web, HttpResponse};
use actix_session::Session;
use futures::lock::Mutex;

pub async fn start_instance(data: web::Data<Mutex<AppState>>, session: Session) -> HttpResponse {
    let mut data = data.lock().await;
    if let Err(e) = auth_manager::authorize_from_session(session, &mut data.auth_manager).await {
        return HttpResponse::BadRequest().body(e.to_string())
    };

    // TODO: check if already in cache
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

pub async fn stop_instance(data: web::Data<Mutex<AppState>>, session: Session) -> HttpResponse {
    let mut data = data.lock().await;
    if let Err(e) = auth_manager::authorize_from_session(session, &mut data.auth_manager).await {
        return HttpResponse::BadRequest().body(e.to_string())
    };
    // TODO: remove from cache
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
