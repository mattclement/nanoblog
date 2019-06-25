#![allow(clippy::needless_lifetimes, dead_code)]
use std::collections::HashMap;
use std::ops::Deref;

use r2d2_redis::redis;
use r2d2_redis::redis::Commands;

pub mod types;
pub mod conn_pool;

pub use conn_pool::Database;
pub use types::*;


impl Database {
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
        let post = self.run(move |conn| conn.get(title))
            .await
            .map_err(|e| e.to_string())?;
        Ok(post)
    }

    /// Retrieve the hash mapping post titles to publish dates.
    pub async fn list_posts(&self) -> HashMap<String, PostMetadata> {
        self.run(move |conn| {
            conn.hgetall("posts")
        })
        .await
        .unwrap_or_default()
    }

    /// Add a post to the index listing by adding it to the hash map storing active posts
    pub async fn activate_post(&self, post: PostMetadata) -> Result<(), String> {
        self.run(move |conn| conn.hset("posts", &post.slug, &post))
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Save a json serialized version of a `Post` keyed by the post title
    pub async fn save_post(&self, post: Post) -> Result<(), String> {
        self.run(move |conn| {
            conn.set(
                &post.slug,
                serde_json::to_string(&post).unwrap_or_default()
            )
        })
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
