pub struct Client {
    host: String,
    token: String,
    client: reqwest::Client,
}

impl Client {
    pub fn new(host: String, token: String) -> Result<Self, String> {
        use std::time::Duration;

        let client = reqwest::Client::builder()
            .gzip(true)
            .timeout(Duration::from_secs(3))
            .build().unwrap();

        let c = Self { host, token, client };
        c.check()?;
        Ok(c)
    }

    fn check(&self) -> Result<(), String> {
        let url = format!("https://{}/api/ping", &self.host);

        let res = self.client.get(&url)
            .bearer_auth(&self.token)
            .send()
            .map_err(|e| e.to_string())?;

        match res.status() {
            reqwest::StatusCode::OK => Ok(()),
            _ => Err("Invalid token".into()),
        }
    }

    pub fn list(self, verbose: bool) -> Result<Vec<String>, std::io::Error> {
        Ok(vec!())
    }

    pub fn publish(self, dry_run: bool, diff: bool, post: &str) -> Result<String, std::io::Error> {
        Ok(String::new())
    }

    pub fn revoke(self, dry_run: bool, post: &str) -> Result<(), std::io::Error> {
        Ok(())
    }
}

