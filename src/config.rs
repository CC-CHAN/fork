use dotenv::dotenv;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
    //pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionConfig {
    pub url: String,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    // TODO: to slice
    pub salt: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub log: LogConfig,
    pub session: SessionConfig,
    pub auth: AuthConfig,
}

pub fn init_config() -> Config {
    dotenv().ok();

    Config {
        database: DatabaseConfig {
            url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            pool_size: env::var("DATEBASE_POOL_SIZE")
                .expect("DATEBASE_POOL_SIZE must be set")
                .parse()
                .unwrap(),
        },
        log: LogConfig {
            level: env::var("LOG_LEVEL").expect("LOG_LEVEL must be set"),
        },
        session: SessionConfig {
            url: env::var("SESSION_REDIS_URL").expect("SESSION_REDIS_URL must be set"),
            key: env::var("SESSION_KEY").expect("SESSION_KEY must be set"),
        },
        auth: AuthConfig {
            salt: env::var("AUTH_SALT").expect("AUTH_SALT must be set"),
        },
    }
}

lazy_static! {
    pub static ref CONFIG: Config = init_config();
}
