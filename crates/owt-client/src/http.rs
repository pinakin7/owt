//! The typed REST client over the `/v1` endpoints (`docs/design/query-api.md`).
//!
//! Every method is typed against the `owt-api-types` contract. Until the server ships
//! they build the request they *would* send and return [`ClientError::Unimplemented`];
//! the request-building seam (`req`) is exactly where the live calls land in the
//! follow-up (ADR-0002).

use owt_api_types::API_VERSION;
use owt_api_types::dto::{
    BookDto, EntityView, EventDetail, MarketDetail, NewsDto, SearchHit, TradeDto, WatchlistView,
};
use owt_api_types::pagination::Page;
use reqwest::{Client, RequestBuilder};

use crate::config::ServerConfig;
use crate::error::ClientError;

/// A REST client bound to one server.
#[derive(Debug, Clone)]
pub struct RestClient {
    client: Client,
    /// Base URL with any trailing slash trimmed.
    base: String,
    bearer: Option<String>,
}

impl RestClient {
    /// Build a client from the effective server config.
    pub fn new(cfg: &ServerConfig) -> Result<Self, ClientError> {
        let client = Client::builder()
            .build()
            .map_err(|e| ClientError::Http(e.to_string()))?;
        Ok(Self {
            client,
            base: cfg.url.trim_end_matches('/').to_owned(),
            bearer: cfg.bearer_token.clone(),
        })
    }

    /// Build a `GET /v1/{path}` request, attaching the bearer token if configured.
    /// The single seam every endpoint routes through.
    fn req(&self, path: &str) -> RequestBuilder {
        let url = format!("{}/{}/{}", self.base, API_VERSION, path);
        let rb = self.client.get(url);
        match &self.bearer {
            Some(token) => rb.bearer_auth(token),
            None => rb,
        }
    }

    /// `GET /v1/search?q=…`
    pub async fn search(&self, query: &str) -> Result<Page<SearchHit>, ClientError> {
        let _req = self.req("search").query(&[("q", query)]);
        Err(ClientError::Unimplemented("RestClient::search"))
    }

    /// `GET /v1/markets/{slug}`
    pub async fn market(&self, slug: &str) -> Result<MarketDetail, ClientError> {
        let _req = self.req(&format!("markets/{slug}"));
        Err(ClientError::Unimplemented("RestClient::market"))
    }

    /// `GET /v1/markets/{slug}/book`
    pub async fn book(&self, slug: &str) -> Result<BookDto, ClientError> {
        let _req = self.req(&format!("markets/{slug}/book"));
        Err(ClientError::Unimplemented("RestClient::book"))
    }

    /// `GET /v1/markets/{slug}/trades`
    pub async fn trades(&self, slug: &str) -> Result<Vec<TradeDto>, ClientError> {
        let _req = self.req(&format!("markets/{slug}/trades"));
        Err(ClientError::Unimplemented("RestClient::trades"))
    }

    /// `GET /v1/markets/{slug}/news`
    pub async fn market_news(&self, slug: &str) -> Result<Vec<NewsDto>, ClientError> {
        let _req = self.req(&format!("markets/{slug}/news"));
        Err(ClientError::Unimplemented("RestClient::market_news"))
    }

    /// `GET /v1/events/{slug}`
    pub async fn event(&self, slug: &str) -> Result<EventDetail, ClientError> {
        let _req = self.req(&format!("events/{slug}"));
        Err(ClientError::Unimplemented("RestClient::event"))
    }

    /// `GET /v1/entities/{id}?include=odds,news,related`
    pub async fn entity(&self, id: &str) -> Result<EntityView, ClientError> {
        let _req = self
            .req(&format!("entities/{id}"))
            .query(&[("include", "odds,news,related")]);
        Err(ClientError::Unimplemented("RestClient::entity"))
    }

    /// `GET /v1/watchlists/{id}`
    pub async fn watchlist(&self, id: &str) -> Result<WatchlistView, ClientError> {
        let _req = self.req(&format!("watchlists/{id}"));
        Err(ClientError::Unimplemented("RestClient::watchlist"))
    }
}
