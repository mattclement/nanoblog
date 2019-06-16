pub struct Client {
    host: String,
    token: String,
}

impl Client {
    pub fn new(host: String, token: String) -> Result<Self, std::io::Error> {
        Ok(Self { host, token })
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

