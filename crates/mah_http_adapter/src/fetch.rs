use std::fmt::Debug;

use async_trait::async_trait;
use reqwest::{Request, Response};

#[async_trait]
pub trait Fetch: Clone + Debug + Send + Sync {
    async fn fetch(&self, request: Request) -> Result<Response, reqwest::Error>;
}

#[derive(Clone, Debug)]
pub struct DefaultFetch {
    client: reqwest::Client,
}

impl DefaultFetch {
    pub fn new() -> Self {
        Self::with_client(Default::default())
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Default for DefaultFetch {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fetch for DefaultFetch {
    async fn fetch(&self, request: Request) -> Result<Response, reqwest::Error> {
        self.client.execute(request).await
    }
}
