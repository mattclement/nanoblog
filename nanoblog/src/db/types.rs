use serde::{Serialize, Deserialize};
use r2d2_redis::redis::{
    Value,
    ToRedisArgs,
    FromRedisValue,
    RedisResult,
    RedisError,
    ErrorKind,
};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub slug: String,
    pub title: String,
    pub body: String,
    pub date_created: String,
    pub date_updated: Option<String>,
}

impl Default for Post {
    fn default() -> Self {
        Self {
            slug: "".into(),
            title: "".into(),
            body: "".into(),
            date_created: "".into(),
            date_updated: None
        }
    }
}

/// Listing for the index page
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostMetadata {
    pub slug: String,
    pub title: String,
    pub date_created: String,
}

/// Convert a `Post` into its `PostMetadata`
impl From<Post> for PostMetadata {
    fn from(post: Post) -> Self {
        Self {
            slug: post.slug,
            title: post.title,
            date_created: post.date_created,
        }
    }
}

impl FromRedisValue for PostMetadata {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        if let Value::Data(ref bytes) = *v {
            let val = serde_json::from_slice::<Self>(bytes)
                .map_err(|e|
                    RedisError::from((ErrorKind::TypeError, "PostMetadata", e.to_string()))
                )?;
            return Ok(val);
        }
        Err(RedisError::from((ErrorKind::TypeError, "wasnt passed bytes")))
    }
}

impl FromRedisValue for Post {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        if let Value::Data(ref bytes) = *v {
            let val = serde_json::from_slice::<Self>(bytes)
                .map_err(|e|
                    RedisError::from((ErrorKind::TypeError, "Post", e.to_string()))
                )?;
            return Ok(val);
        }
        Err(RedisError::from((ErrorKind::TypeError, "wasnt passed bytes")))
    }
}

impl ToRedisArgs for &PostMetadata {
    fn write_redis_args(&self, out: &mut Vec<Vec<u8>>) {
        out.push(
            serde_json::to_vec(self).expect("Couldn't serialize PostMetadata")
        );
    }
}
