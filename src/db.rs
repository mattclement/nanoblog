extern crate tokio;
extern crate r2d2_redis;

use std::thread;
use std::ops::Deref;

use r2d2_redis::{redis, r2d2, RedisConnectionManager};
use r2d2_redis::redis::Commands;


// use diesel::pg::PgConnection;
// use diesel::r2d2::ConnectionManager;

use futures01::future::poll_fn;
use r2d2::{Pool, PooledConnection};

use tokio_threadpool::blocking;

pub type ConnectionPool = Pool<RedisConnectionManager>;
pub type Connection = PooledConnection<RedisConnectionManager>;

/// A database "repository", for running database workloads.
/// Manages a connection pool and running blocking tasks in a
/// way that does not block the tokio event loop.
#[derive(Clone)]
pub struct Database {
    connection_pool: ConnectionPool
}

impl Database {
    pub fn new() -> Self {
        let manager = RedisConnectionManager::new("redis://localhost").unwrap();
        let pool = r2d2::Pool::builder()
            .build(manager)
            .unwrap();

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
        // `tokio::await` is used to convert the old style future to a `std::futures::Future`.
        // `f.take()` allows the borrow checker to be sure `f` is not moved into the inner closure
        // multiple times if `poll_fn` is called multple times.
        let mut f = Some(f);
        poll_fn(|| blocking(|| (f.take().unwrap())(
            pool.get().unwrap()
        ))
        .map_err(|_| panic!("the threadpool shut down"))).compat().await
        .expect("Error running async database task.")
    }

    pub async fn get_post(self, title: String) -> String {
        self.run(move |conn| {
            redis::cmd("GET").arg(title).query::<String>(conn.deref())
        }).await.unwrap_or("".into())
    }
}
