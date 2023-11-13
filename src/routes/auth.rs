use crate::AppState;

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
) -> HttpResponse {
    let mut data = data.lock().await;
    let ret = data.auth_manager.register(info.username.clone(), info.password.clone()).await;
    match ret {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(token) => HttpResponse::Ok().body(token),
    }
}

pub async fn login(data: web::Data<Mutex<AppState>>, info: web::Json<LoginPostRequest>) -> HttpResponse {
    let mut data = data.lock().await;
    let ret = data.auth_manager.login(info.username.clone(), info.password.clone()).await;
    match ret {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(token) => HttpResponse::Ok().body(token),
    }
}
