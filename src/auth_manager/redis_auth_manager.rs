use crate::auth_manager::AuthManager;

use async_trait::async_trait;
use redis::{aio::Connection, RedisError, RedisResult};
use redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use actix_web::{web, HttpResponse};

use jwt_simple::prelude::*;

pub fn create_jwt(username: &str) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: uid.to_owned(),
        role: role.to_string(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|_| Error::JWTTokenCreationError)
}

pub struct RedisAuthManager {
    con: Connection,
}

impl RedisAuthManager {
    pub async fn new(url: String) -> RedisAuthManager {
        let client = redis::Client::open(url).unwrap();
        let con = client.get_async_connection().await.unwrap();

        RedisAuthManager { con }
    }
}

#[async_trait]
impl AuthManager for RedisAuthManager {
    async fn register(
        &self,
        username: String,
        password: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: check for already existing user
        // TODO: add user to database
        // TODO: generate tokens and return AuthResponse
        let _: String = self.con.get(&username).await.expect("User already exists");

        let ret: Result<(), RedisError> = self.con.set(username, password).await;
        match ret {
            Ok(_) => (),
            Err(e) => return Err("Error creating user".into()),
        }


        Ok(())
    }
    async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: check if user exists, if not error
        // TODO: check hashed password, if bad return error
        // TODO: generate tokens and return AuthResponse
        Ok(())
    }
}
