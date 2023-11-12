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

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn register(
    data: web::Data<Mutex<AppState>>,
    info: web::Json<RegisterPostRequest>,
) -> HttpResponse {
    let mut data = data.lock().await;
    // TODO: check for already existing user
    // TODO: add user to database
    // TODO: generate tokens and return AuthResponse
    HttpResponse::Ok().body("registration complete")
}

pub async fn login(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    let mut data = data.lock().await;
    // TODO: check if user exists, if not error
    // TODO: check hashed password, if bad return error
    // TODO: generate tokens and return AuthResponse
    HttpResponse::Ok().body("login complete")
}
