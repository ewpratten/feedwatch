use rss::Channel;
use worker::{console_log, Cache, Fetch, Url, Headers};

#[derive(Debug, thiserror::Error)]
pub enum SubscriptionFetchError {
    #[error(transparent)]
    WorkerError(#[from] worker::Error),
    #[error(transparent)]
    UrlError(#[from] url::ParseError),
    #[error(transparent)]
    RssError(#[from] rss::Error),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Subscription {
    pub name: String,
    pub url: String,
    pub tags: Vec<String>,
}

impl Subscription {
    // Gets the RSS channel for this subscription
    pub async fn get_channel(&self) -> Result<Channel, SubscriptionFetchError> {
        // Open the channel cache
        let cache = Cache::open("feedwatch_channels".to_string()).await;

        // If we have a cached response, use it
        let mut response = match cache.get(self.url.clone(), false).await? {
            Some(response) => response,
            None => {
                // Send a request to the remote server
                console_log!("Cache miss. Fetching: {}", self.url);
                let mut response = Fetch::Url(Url::parse(&self.url)?).send().await?.with_headers({
                    let mut headers = Headers::new();
                    headers.set("Cache-Control", "max-age=600")?;
                    headers
                });

                // Store the response in the cache
                cache.put(self.url.clone(), response.cloned()?).await?;

                response
            }
        };

        // Parse into a Channel
        Ok(Channel::read_from(response.text().await?.as_bytes())?)
    }
}

pub fn get_subscriptions() -> Vec<Subscription> {
    serde_json::from_str(include_str!("../../static/subscriptions.json")).unwrap()
}
