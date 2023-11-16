use crate::auth_manager::AuthManager;

use async_trait::async_trait;
use redis::{aio::Connection, RedisError, RedisResult};
use redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use actix_web::{web, HttpResponse};
use actix_session::Session;

use jwt_simple::prelude::*;

pub struct RedisAuthManager {
    con: Connection,
}

impl RedisAuthManager {
    pub async fn new(url: String) -> RedisAuthManager {
        println!("{}", url);
        let client = redis::Client::open(url).unwrap();
        let con = client.get_async_connection().await.unwrap();

        RedisAuthManager { con }
    }
}

// Two entries are created:
// <username> - <password>
// <username>_key - <key>
// requires separate redis instance from cache

#[async_trait]
impl AuthManager for RedisAuthManager {
    async fn register(
        &mut self,
        username: String,
        password: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let ret: RedisResult<String> = self.con.get(&username).await;
        if let Ok(_) = ret {
            return Err("User already exists".into());
        }

        let ret: Result<(), RedisError> = self.con.set(&username, password).await;
        if let Err(e) = ret {
            return Err(("Error creating user: ".to_owned() + &e.to_string()).into());
        }

        let key = HS256Key::generate();
        let ret: Result<(), RedisError> = self.con.set(username + "_key", key.to_bytes()).await;
        let claims = Claims::create(Duration::from_hours(2));
        let token = key.authenticate(claims)?;
        Ok(token)
    }
    async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let pass: String = self.con.get(&username).await.expect("User does not exist.");
        if pass != password {
            return Err("Wrong password".into());
        }
        let ret: Vec<u8> = self.con.get(username + "_key").await.unwrap();
        let key = HS256Key::from_bytes(&ret);
        let claims = Claims::create(Duration::from_hours(2));
        let token = key.authenticate(claims)?;
        Ok(token)
    }

    async fn validate_token(
        &mut self,
        username: String,
        token: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ret: Vec<u8> = self.con.get(username + "_key").await.unwrap();
        let key = HS256Key::from_bytes(&ret);
        match key.verify_token::<NoCustomClaims>(&token, None) {
            Ok(_) => Ok(()),
            Err(_) => Err("invalid token".into())
        }
    }
}
