use crate::AppState;

use actix_session::Session;
use actix_web::{web, HttpResponse};
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginPostRequest {
    pub username: String,
    pub password: String, // hashed
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RegisterPostRequest {
    pub username: String,
    pub password: String, // hashed
}

pub async fn register(
    data: web::Data<Mutex<AppState>>,
    info: web::Json<RegisterPostRequest>,
    session: Session,
) -> HttpResponse {
    if let Ok(Some(_)) = session.get::<String>("session_token") {
        return HttpResponse::BadRequest().body("Already logged in");
    }

    let mut data = data.lock().await;
    let ret = data
        .auth_manager
        .register(info.username.clone(), info.password.clone())
        .await;
    match ret {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(token) => {
            session
                .insert("session_token", &token)
                .expect("Cannot set session cookie");
            session
                .insert("session_username", &info.username)
                .expect("Cannot set session username");
            HttpResponse::Ok().body(token)
        }
    }
}

pub async fn login(
    data: web::Data<Mutex<AppState>>,
    info: web::Json<LoginPostRequest>,
    session: Session,
) -> HttpResponse {
    if let Ok(Some(_)) = session.get::<String>("session_token") {
        return HttpResponse::BadRequest().body("Already logged in");
    }

    let mut data = data.lock().await;
    let ret = data
        .auth_manager
        .login(info.username.clone(), info.password.clone())
        .await;
    match ret {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(token) => {
            session
                .insert("session_token", &token)
                .expect("Cannot set session cookie");
            session
                .insert("session_username", &info.username)
                .expect("Cannot set session username");
            HttpResponse::Ok().body(token)
        }
    }
}

pub async fn logout(_data: web::Data<Mutex<AppState>>, session: Session) -> HttpResponse {
    if let Err(_) = session.get::<String>("session_token") {
        return HttpResponse::BadRequest().body("Not logged in");
    }

    match session.remove("session_token") {
        Some(_) => {
            session.remove("session_username");
            HttpResponse::Ok().body(())
        }
        None => HttpResponse::BadRequest().body("Not logged in"),
    }
}
