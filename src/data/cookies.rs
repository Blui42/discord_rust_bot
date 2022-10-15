use std::collections::HashMap;
use std::fs;

pub struct Cookies<'a> {
    data: HashMap<String, u64>,
    path: &'a str,
}

impl<'a> Cookies<'a> {
    // Might get relevant later in development
    #[allow(dead_code)]
    pub fn get(&self, user: u64) -> Option<u64> {
        self.data.get(&user.to_string()).copied()
    }
    #[allow(dead_code)]
    pub fn set(&mut self, user: u64, cookies: u64) {
        self.data.insert(user.to_string(), cookies);
    }
    pub fn give(&mut self, user: u64, cookies: u64) {
        self.data.entry(user.to_string()).and_modify(|x| *x += cookies).or_default();
    }

    pub fn new(path: &'a str) -> Self {
        let file = fs::read_to_string(path);
        let file_contents: &str = file.as_ref().map_or("{}", |x| x);
        let data: HashMap<_, _> = serde_json::from_str(file_contents).unwrap_or_default();
        Self { data, path }
    }

    pub async fn save(&self) -> Result<(), std::io::Error> {
        if let Ok(file_info) = serde_json::to_string_pretty(&self.data) {
            tokio::fs::write(&self.path, file_info).await?;
        }
        Ok(())
    }
}
impl<'a> Drop for Cookies<'a> {
    fn drop(&mut self) {
        if let Ok(file_content) = serde_json::to_string_pretty(&self.data) {
            if let Err(why) = fs::write(&self.path, file_content) {
                eprintln!("Error writing to file: {why}");
            }
        }
    }
}
