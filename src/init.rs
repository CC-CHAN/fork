use crate::config::CONFIG;
use async_redis_session::RedisSessionStore;
use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use r2d2::Pool;
use std::str::FromStr;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

fn init_session_store() -> RedisSessionStore {
    debug!("init session store connection");
    RedisSessionStore::new(CONFIG.session.url.as_str()).unwrap()
}

type DbConnectionPool = Pool<ConnectionManager<PgConnection>>;

fn init_db_connectin_pool() -> DbConnectionPool {
    let url = CONFIG.database.url.as_str();
    let pool_size = CONFIG.database.pool_size;
    debug!(
        "init databse connection: [{}], with pool size: {}",
        url, pool_size
    );
    let manager = ConnectionManager::<PgConnection>::new(url);
    r2d2::Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .unwrap()
}

#[derive(Clone)]
pub struct AppConnections {
    pub db_connections: DbConnectionPool,
    pub session_store: RedisSessionStore,
}

pub fn init_appliations() -> AppConnections {
    // init application width logging
    let sub = FmtSubscriber::builder()
        .with_max_level(Level::from_str(CONFIG.log.level.as_str()).unwrap())
        .finish();

    tracing::subscriber::set_global_default(sub).unwrap();
    info!("logger is initiated, init_appliations");
    AppConnections {
        db_connections: init_db_connectin_pool(),
        session_store: init_session_store(),
    }
}
