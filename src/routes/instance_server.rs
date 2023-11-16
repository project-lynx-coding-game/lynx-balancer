use crate::{auth_manager, AppState};

use actix_session::Session;
use actix_web::{web, HttpResponse};
use futures::lock::Mutex;

pub async fn start_instance(data: web::Data<Mutex<AppState>>, session: Session) -> HttpResponse {
    let mut data = data.lock().await;
    if let Err(e) = auth_manager::authorize_from_session(&session, &mut data.auth_manager).await {
        return HttpResponse::BadRequest().body(e.to_string());
    };

    let username = session.get::<String>("session_username").unwrap().unwrap();

    // TODO: check if already in cache
    // TODO: if existing user, first stop previous instance
    let new_instance = data.instance_host.start_instance(username.clone()).await;
    match new_instance {
        Ok(instance) => {
            data.url_cache
                .set(username, instance.get_url_with_port())
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
    if let Err(e) = auth_manager::authorize_from_session(&session, &mut data.auth_manager).await {
        return HttpResponse::BadRequest().body(e.to_string());
    };

    let username = session.get::<String>("session_username").unwrap().unwrap();

    // TODO: remove from cache
    // TODO: save state of scene host?
    match data.instance_host.stop_instance(username).await {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().body("Instance could not be stopped"),
    }
    HttpResponse::Ok().body("done")
}
