use reqwest::Client;
use anyhow::{Ok, Result};

pub struct HttpClient {
    client : Client
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new()
        }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(
        &self , 
        url : &str 
    ) -> Result<T> {

        let res = self.client.get(url).send().await?.error_for_status()?.json::<T>().await?;

        Ok(res)
    }
}