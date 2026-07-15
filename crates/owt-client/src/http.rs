//! The typed REST client over the `/v1` endpoints (`docs/design/query-api.md`).
//!
//! Every method is typed against the `owt-api-types` contract and routed through the
//! request-building seam (`req`) and the response seam (`send`).

use owt_api_types::API_VERSION;
use owt_api_types::dto::{
    BookDto, EntityView, EventDetail, MarketDetail, NewsDto, SearchHit, TradeDto, WatchlistView,
};
use owt_api_types::error::ProblemDetails;
use owt_api_types::pagination::Page;
use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;

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

    /// Send a built request and decode the response into `T`. The single response seam:
    /// transport/decode failures become [`ClientError::Http`]; a non-2xx response with an
    /// RFC 7807 body becomes [`ClientError::Api`], otherwise [`ClientError::Status`].
    async fn send<T: DeserializeOwned>(&self, rb: RequestBuilder) -> Result<T, ClientError> {
        let resp = rb
            .send()
            .await
            .map_err(|e| ClientError::Http(e.to_string()))?;
        let status = resp.status();
        if status.is_success() {
            resp.json::<T>()
                .await
                .map_err(|e| ClientError::Http(e.to_string()))
        } else {
            let body = resp.text().await.unwrap_or_default();
            match serde_json::from_str::<ProblemDetails>(&body) {
                Ok(problem) => Err(ClientError::Api(problem)),
                Err(_) => Err(ClientError::Status {
                    status: status.as_u16(),
                    body,
                }),
            }
        }
    }

    /// `GET /v1/search?q=…`
    pub async fn search(&self, query: &str) -> Result<Page<SearchHit>, ClientError> {
        self.send(self.req("search").query(&[("q", query)])).await
    }

    /// `GET /v1/markets/{slug}`
    pub async fn market(&self, slug: &str) -> Result<MarketDetail, ClientError> {
        self.send(self.req(&format!("markets/{slug}"))).await
    }

    /// `GET /v1/markets/{slug}/book`
    pub async fn book(&self, slug: &str) -> Result<BookDto, ClientError> {
        self.send(self.req(&format!("markets/{slug}/book"))).await
    }

    /// `GET /v1/markets/{slug}/trades`
    pub async fn trades(&self, slug: &str) -> Result<Vec<TradeDto>, ClientError> {
        self.send(self.req(&format!("markets/{slug}/trades"))).await
    }

    /// `GET /v1/markets/{slug}/news`
    pub async fn market_news(&self, slug: &str) -> Result<Vec<NewsDto>, ClientError> {
        self.send(self.req(&format!("markets/{slug}/news"))).await
    }

    /// `GET /v1/events/{slug}`
    pub async fn event(&self, slug: &str) -> Result<EventDetail, ClientError> {
        self.send(self.req(&format!("events/{slug}"))).await
    }

    /// `GET /v1/entities/{id}?include=odds,news,related`
    pub async fn entity(&self, id: &str) -> Result<EntityView, ClientError> {
        self.send(
            self.req(&format!("entities/{id}"))
                .query(&[("include", "odds,news,related")]),
        )
        .await
    }

    /// `GET /v1/watchlists/{id}`
    pub async fn watchlist(&self, id: &str) -> Result<WatchlistView, ClientError> {
        self.send(self.req(&format!("watchlists/{id}"))).await
    }
}
