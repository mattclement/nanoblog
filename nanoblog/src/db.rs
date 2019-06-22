#![allow(clippy::needless_lifetimes)]
extern crate r2d2_redis;
extern crate tokio;

use std::collections::HashMap;
use std::ops::Deref;

use r2d2_redis::{r2d2, redis, RedisConnectionManager};
use crate::db::r2d2_redis::redis::Commands;

use futures01::future::poll_fn;
use r2d2::{Pool, PooledConnection};

use tokio_threadpool::blocking;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub title: String,
    pub body: String,
    pub date_created: String,
    pub date_updated: Option<String>,
}

impl Default for Post {
    fn default() -> Self {
        Self {title: "".into(), body: "".into(), date_created: "".into(), date_updated: None}
    }
}

pub type ConnectionPool = Pool<RedisConnectionManager>;
pub type Connection = PooledConnection<RedisConnectionManager>;


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

    /// Verify whether the bearer token exists.
    pub async fn validate_token(self, token: String) -> bool {
        let exists = self.run(move |conn| {
            redis::cmd("SISMEMBER")
                .arg("bearer_tokens")
                .arg(token)
                .query::<bool>(conn.deref())
        })
        .await;

        match exists {
            Err(_) => false,
            Ok(x) => x
        }
    }

    /// Get a `Post`, by its `title`  property.
    pub async fn get_post(&self, title: String) -> Result<Post, String> {
        self.run(move |conn| {
            let x: r2d2_redis::redis::RedisResult<String> = conn.get(title);
            x
        })
        .await
        .map_err(|e| e.to_string())
        .map(|contents| serde_json::from_str::<Post>(&contents).unwrap_or_default())
    }

    /// Retrieve the hash mapping post titles to publish dates.
    pub async fn list_posts(&self) -> HashMap<String, String> {
        self.run(move |conn| {
            conn.hgetall("posts")
        })
        .await
        .unwrap_or_default()
    }

    /// Add a post to the index listing by adding it to the hash map storing active posts
    pub async fn activate_post(&self, post: Post) -> Result<(), String> {
        self.run(move |conn| conn.hset("posts", post.title, post.date_created))
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Save a json serialized version of a `Post` keyed by the post title
    pub async fn save_post(&self, post: Post) -> Result<(), String> {
        self.run(move |conn| {
            conn.set(
                post.title.clone(),
                serde_json::to_string(&post).unwrap_or_default()
            )
        })
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
