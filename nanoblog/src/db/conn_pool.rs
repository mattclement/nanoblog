#![allow(clippy::needless_lifetimes)]
use r2d2_redis::{r2d2, redis, RedisConnectionManager};

use futures01::future::poll_fn;
use r2d2::{Pool, PooledConnection};

use tokio_threadpool::blocking;

pub type ConnectionPool = Pool<RedisConnectionManager>;
pub type Connection = PooledConnection<RedisConnectionManager>;

/// This was modified from
/// https://github.com/colinbankier/realworld-tide/blob/master/src/db.rs.
/// A database "repository", for running database workloads.
/// Manages a connection pool and running blocking tasks in a
/// way that does not block the tokio event loop.
#[derive(Clone)]
pub struct Database {
    connection_pool: ConnectionPool,
}

impl Database {
    pub fn new() -> Self {
        let host = std::env::var("REDIS_HOST");
        let pw = std::env::var("REDIS_PASSWORD");

        let conn_string = match (host, pw) {
            (Ok(host), Err(_)) => redis::parse_redis_url(&format!("redis://{}", host)),
            (Ok(host), Ok(pw)) => redis::parse_redis_url(&format!("redis://user:{}@{}", pw, host)),
            (Err(_), Ok(pw)) => redis::parse_redis_url(&format!("redis://user:{}@localhost", pw)),
            (_, _) => redis::parse_redis_url("redis://localhost"),
        };

        let manager = RedisConnectionManager::new(conn_string.unwrap()).unwrap();
        let pool = r2d2::Pool::builder().build(manager).unwrap();

        Database {
            connection_pool: pool,
        }
    }

    /// Runs the given closure in a way that is safe for blocking IO to the database.
    /// The closure will be passed a `Connection` from the pool to use.
    pub async fn run<F, T>(&self, f: F) -> T
    where
        F: FnOnce(Connection) -> T + Send + std::marker::Unpin + 'static,
        T: Send + 'static,
    {
        use futures::compat::Future01CompatExt;
        let pool = self.connection_pool.clone();
        // `tokio_threadpool::blocking` returns a `Poll` compatible with "old style" futures.
        // `poll_fn` converts this into a future, then
        // `f.take()` allows the borrow checker to be sure `f` is not moved into the inner closure
        // multiple times if `poll_fn` is called multple times.
        let mut f = Some(f);
        poll_fn(|| {
            blocking(|| (f.take().unwrap())(pool.get().unwrap()))
                .map_err(|_| panic!("the threadpool shut down"))
        })
        .compat()
        .await
        .expect("Error running async database task.")
    }
}
