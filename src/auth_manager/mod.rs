pub mod redis_auth_manager;

use actix_session::Session;
use async_trait::async_trait;


#[async_trait]
pub trait AuthManager {
    async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<String, Box<dyn std::error::Error>>;
    async fn register(
        &mut self,
        username: String,
        password: String,
    ) -> Result<String, Box<dyn std::error::Error>>;
    async fn validate_token(
        &mut self,
        username: String,
        token: String,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub async fn authorize_from_session(
    session: &Session,
    auth_manager: &mut Box<dyn AuthManager + Sync + Send>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ret = session.get::<String>("session_token");
    if let Err(_) = ret {
        return Err("Cannot get session token".into());
    }
    let ret = ret.unwrap();
    if let None = ret {
        return Err("Cannot get session token".into());
    }
    let token = ret.unwrap();
    let ret = session.get::<String>("session_username");
    if let Err(_) = ret {
        return Err("Cannot get session username".into());
    }
    let ret = ret.unwrap();
    if let None = ret {
        return Err("Cannot get session username".into());
    }
    let username = ret.unwrap();

    auth_manager.validate_token(username, token).await
}
