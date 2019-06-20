use std::collections::HashMap;

pub struct Client {
    host: String,
    token: String,
    client: reqwest::Client,
}

type ListResponse = HashMap<String, String>;


impl Client {
    pub fn new(host: String, token: String) -> Result<Self, String> {
        use std::time::Duration;

        let client = reqwest::Client::builder()
            .gzip(true)
            .timeout(Duration::from_secs(3))
            .danger_accept_invalid_certs(true)
            .build().unwrap();

        let c = Self { host, token, client };
        c.check()?;
        Ok(c)
    }

    fn url_for_path(&self, path: &str) -> String {
        format!("https://{}/api/{}", &self.host, path)
    }

    fn get(&self, path: &str) -> reqwest::Result<reqwest::Response> {
        let url = self.url_for_path(path);

        self.client.get(&url)
            .bearer_auth(&self.token)
            .send()
    }

    fn check(&self) -> Result<(), String> {
        let res = self.get("ping")
            .map_err(|e| e.to_string())?;

        match res.status() {
            reqwest::StatusCode::OK => Ok(()),
            _ => Err("Invalid token".into()),
        }
    }

    pub fn list_posts(&self, verbose: bool) -> Result<Vec<String>, String> {
        self.get("posts")
            .map_err(|e| e.to_string())
            .map(|mut b| {
                let kv: ListResponse = b.json().expect("Bad JSON returned");
                kv.into_iter()
                    .map(|(title, date)| {
                        if verbose {
                            format!("{} [{}]", title, date)
                        } else {
                            title.to_string()
                        }
                    })
                    .collect()
            })
    }

    pub fn publish(&self, post: &str, dry_run: bool, diff: bool) -> Result<String, std::io::Error> {
        println!("{}", post);
        Ok(String::new())
    }

    pub fn unpublish(&self, post: &str, dry_run: bool) -> Result<(), std::io::Error> {
        Ok(())
    }
}

