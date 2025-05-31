use crate::auth_manager::AuthManager;



use async_trait::async_trait;
use redis::{aio::Connection, RedisError, RedisResult};
use redis::{AsyncCommands};

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
// <username>_pass - <password>
// <username>_key - <key>

#[async_trait]
impl AuthManager for RedisAuthManager {
    async fn register(
        &mut self,
        username: String,
        password: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if username.contains("_") {
            return Err("Username contains illegal character: _".into());
        }

        let ret: RedisResult<String> = self.con.get(username.clone() + "_pass").await;
        if let Ok(_) = ret {
            return Err("User already exists".into());
        }

        let ret: Result<(), RedisError> = self.con.set(username.clone() + "_pass", password).await;
        if let Err(e) = ret {
            return Err(("Error creating user: ".to_owned() + &e.to_string()).into());
        }

        let key = HS256Key::generate();
        let _ret: Result<(), RedisError> = self
            .con
            .set(username.clone() + "_key", key.to_bytes())
            .await;
        let claims = Claims::create(Duration::from_hours(2));
        let token = key.authenticate(claims)?;
        Ok(token)
    }
    async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let ret: Result<String, RedisError> = self.con.get(username.clone() + "_pass").await;
        if let Err(_e) = ret {
            return Err("User does not exist".into());
        }
        let pass = ret.unwrap();
        if pass != password {
            return Err("Wrong password".into());
        }
        let ret: Vec<u8> = self.con.get(username.clone() + "_key").await.unwrap();
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
            Err(_) => Err("invalid token".into()),
        }
    }
}
