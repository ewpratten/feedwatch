use rss::Channel;
use worker::{Fetch, Url};

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
    pub tags: Vec<String>
}

impl Subscription {
    // Gets the RSS channel for this subscription
    pub async fn get_channel(&self) -> Result<Channel, SubscriptionFetchError> {
        // Fetch the URL
        let mut response = Fetch::Url(Url::parse(&self.url)?).send().await?;

        // Parse into a Channel
        Ok(Channel::read_from(response.text().await?.as_bytes())?)
    }
}

pub fn get_subscriptions() -> Vec<Subscription> {
    serde_json::from_str(include_str!("../../static/subscriptions.json")).unwrap()
}
