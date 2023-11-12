use crate::AppState;

use actix_proxy::IntoHttpResponse;
use actix_web::web::Bytes;
use actix_web::{get, post, web, HttpResponse};
use awc;
use futures::lock::Mutex;

#[get("/{tail:.*}")]
pub async fn get_proxy(
    request: actix_web::HttpRequest,
    data: web::Data<Mutex<AppState>>,
    path: web::Path<String>,
    bytes: Bytes,
) -> HttpResponse {
    // TODO: unpacking username from http request will be different, it has to be planned out
    // TODO: add username and token to request
    // TODO: validate token
    let mut data = data.lock().await;
    let url;
    if data.use_cache_query {
        url = data.url_cache.get_or_query("test-user".to_string()).await;
    } else {
        url = data.url_cache.get("test-user".to_string()).await;
    }

    if let Some(url) = url {
        let client = awc::Client::default();

        let mut final_url = "http://".to_owned() + &url + "/" + &path.into_inner();
        if request.query_string() != "" {
            final_url += "?";
            final_url += request.query_string();
        }

        let res = client.get(final_url).send_body(bytes).await.unwrap();
        res.into_http_response()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("/{tail:.*}")]
pub async fn post_proxy(
    request: actix_web::HttpRequest,
    data: web::Data<Mutex<AppState>>,
    path: web::Path<String>,
    bytes: Bytes,
) -> HttpResponse {
    // TODO: unpacking username from http request will be different, it has to be planned out
    // TODO: add username and token to request
    // TODO: validate token
    let mut data = data.lock().await;
    let url;
    if data.use_cache_query {
        url = data.url_cache.get_or_query("test-user".to_string()).await;
    } else {
        url = data.url_cache.get("test-user".to_string()).await;
    }

    if let Some(url) = url {
        let client = awc::Client::default();

        let mut final_url = "http://".to_owned() + &url + "/" + &path.into_inner();
        if request.query_string() != "" {
            final_url += "?";
            final_url += request.query_string();
        }

        let res = client.post(final_url).send_body(bytes).await.unwrap();
        res.into_http_response()
    } else {
        HttpResponse::NotFound().finish()
    }
}
