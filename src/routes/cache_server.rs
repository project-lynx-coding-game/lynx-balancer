use crate::cache_provider;
use crate::AppState;

use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse};
use cache_provider::{CacheGetRequest, CacheSetRequest};
use futures::lock::Mutex;

pub async fn cache_get(
    data: web::Data<Mutex<AppState>>,
    info: web::Query<CacheGetRequest<String>>,
) -> HttpResponse {
    let mut data = data.lock().await;
    let result = data.url_cache.get(info.key.clone()).await;
    match result {
        Some(url) => HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(url.clone()),
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn cache_set(
    data: web::Data<Mutex<AppState>>,
    info: web::Query<CacheSetRequest<String, String>>,
) -> HttpResponse {
    let mut data = data.lock().await;
    data.url_cache
        .set(info.key.clone(), info.value.clone())
        .await;
    HttpResponse::Ok().finish()
}
