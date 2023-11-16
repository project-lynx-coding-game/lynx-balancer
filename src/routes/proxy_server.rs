use crate::{auth_manager, AppState};

use actix_proxy::IntoHttpResponse;
use actix_session::Session;
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
    session: Session,
) -> HttpResponse {
    let mut data = data.lock().await;

    if let Err(e) = auth_manager::authorize_from_session(&session, &mut data.auth_manager).await {
        return HttpResponse::BadRequest().body(e.to_string());
    };

    let username = session.get::<String>("session_username").unwrap().unwrap();

    let url;
    if data.use_cache_query {
        url = data.url_cache.get_or_query(username).await;
    } else {
        url = data.url_cache.get(username).await;
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
    session: Session,
) -> HttpResponse {
    let mut data = data.lock().await;

    if let Err(e) = auth_manager::authorize_from_session(&session, &mut data.auth_manager).await {
        return HttpResponse::BadRequest().body(e.to_string());
    };

    let username = session.get::<String>("session_username").unwrap().unwrap();

    let url;
    if data.use_cache_query {
        url = data.url_cache.get_or_query(username).await;
    } else {
        url = data.url_cache.get(username).await;
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
