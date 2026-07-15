//! REST contract tests for `RestClient` (testing-strategy.md § API contract).
//!
//! `owt-api` is not implemented yet, so these drive the client against a `wiremock`
//! mock: they assert the outgoing request shape (method, `/v1/...` path, query, bearer
//! header) and that canned responses decode into the `owt-api-types` DTOs — plus the
//! two error-mapping paths (`ProblemDetails` → `Api`, unparseable body → `Status`).

use owt_api_types::dto::{HitKind, Side};
use owt_api_types::error::ErrorCode;
use owt_client::config::ServerConfig;
use owt_client::{ClientError, RestClient};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A client pointed at `url` with no bearer token.
fn client(url: &str) -> RestClient {
    RestClient::new(&ServerConfig {
        url: url.to_owned(),
        bearer_token: None,
    })
    .expect("client builds")
}

#[tokio::test]
async fn search_sends_query_and_decodes_a_page() {
    let server = MockServer::start().await;
    let body = json!({
        "items": [
            { "kind": "market", "slug": "will-fed-cut", "title": "Fed cut?", "snippet": "YES 42¢" },
            { "kind": "entity", "slug": "ent-powell", "title": "Jerome Powell", "snippet": null }
        ],
        "next": "cursor-token"
    });
    Mock::given(method("GET"))
        .and(path("/v1/search"))
        .and(query_param("q", "fed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let page = client(&server.uri()).search("fed").await.expect("ok");
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].kind, HitKind::Market);
    assert_eq!(page.items[0].slug, "will-fed-cut");
    assert_eq!(
        page.next.as_ref().map(|c| c.0.as_str()),
        Some("cursor-token")
    );
}

#[tokio::test]
async fn market_detail_decodes() {
    let server = MockServer::start().await;
    let body = json!({
        "market_id": "will-fed-cut",
        "event_id": "fed-sept",
        "title": "Will the Fed cut in September?",
        "outcomes": [{ "name": "Yes", "price": 0.42 }, { "name": "No", "price": 0.58 }],
        "last": 0.42, "bid": 0.41, "ask": 0.43, "spread": 0.02,
        "vol_24h": 3200000.0, "liquidity": "high", "tags": ["fed", "macro"]
    });
    Mock::given(method("GET"))
        .and(path("/v1/markets/will-fed-cut"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let detail = client(&server.uri())
        .market("will-fed-cut")
        .await
        .expect("ok");
    assert_eq!(detail.title, "Will the Fed cut in September?");
    assert_eq!(detail.outcomes.len(), 2);
    assert_eq!(detail.market_id.to_string(), "will-fed-cut");
}

#[tokio::test]
async fn book_decodes() {
    let server = MockServer::start().await;
    let body = json!({
        "bids": [{ "price": 0.41, "size": 1050.0 }],
        "asks": [{ "price": 0.43, "size": 1200.0 }],
        "mid": 0.42
    });
    Mock::given(method("GET"))
        .and(path("/v1/markets/will-fed-cut/book"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let book = client(&server.uri())
        .book("will-fed-cut")
        .await
        .expect("ok");
    assert_eq!(book.mid, 0.42);
    assert_eq!(book.bids.len(), 1);
}

#[tokio::test]
async fn trades_decode() {
    let server = MockServer::start().await;
    let body = json!([
        { "ts": "2026-07-10T10:14:02Z", "side": "buy", "price": 0.42, "size": 90.0 },
        { "ts": "2026-07-10T10:13:58Z", "side": "sell", "price": 0.41, "size": 120.0 }
    ]);
    Mock::given(method("GET"))
        .and(path("/v1/markets/will-fed-cut/trades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let trades = client(&server.uri())
        .trades("will-fed-cut")
        .await
        .expect("ok");
    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].side, Side::Buy);
}

#[tokio::test]
async fn market_news_decodes() {
    let server = MockServer::start().await;
    let body = json!([
        { "news_id": "n1", "publisher": "Reuters", "headline": "Fed signals cut",
          "url": "https://example.com/r", "published_at": "2026-07-10T09:58:00Z" }
    ]);
    Mock::given(method("GET"))
        .and(path("/v1/markets/will-fed-cut/news"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let news = client(&server.uri())
        .market_news("will-fed-cut")
        .await
        .expect("ok");
    assert_eq!(news.len(), 1);
    assert_eq!(news[0].publisher, "Reuters");
}

#[tokio::test]
async fn event_decodes() {
    let server = MockServer::start().await;
    let body = json!({
        "event_id": "fed-sept",
        "title": "Fed September Meeting",
        "markets": [{ "market_id": "will-fed-cut", "title": "Rate cut?" }],
        "timeline": [
            { "ts": "2026-07-10T10:02:00Z", "kind": "news", "title": "Powell speech",
              "numeric_delta": null, "confidence": 0.9 }
        ]
    });
    Mock::given(method("GET"))
        .and(path("/v1/events/fed-sept"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let event = client(&server.uri()).event("fed-sept").await.expect("ok");
    assert_eq!(event.markets.len(), 1);
    assert_eq!(event.timeline.len(), 1);
}

#[tokio::test]
async fn entity_sends_include_and_decodes() {
    let server = MockServer::start().await;
    let body = json!({
        "entity_id": "ent-powell",
        "kind": "person",
        "name": "Jerome Powell",
        "description": "Fed Chair",
        "odds": [{ "label": "Cut", "probability": 0.42, "market_id": "will-fed-cut" }],
        "news": [],
        "related": [{ "entity_id": "ent-fed", "name": "Federal Reserve" }]
    });
    Mock::given(method("GET"))
        .and(path("/v1/entities/ent-powell"))
        .and(query_param("include", "odds,news,related"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let entity = client(&server.uri())
        .entity("ent-powell")
        .await
        .expect("ok");
    assert_eq!(entity.name, "Jerome Powell");
    assert_eq!(entity.related.len(), 1);
}

#[tokio::test]
async fn watchlist_decodes() {
    let server = MockServer::start().await;
    let body = json!({
        "name": "macro",
        "rows": [{ "market_id": "will-fed-cut", "title": "Fed cut?", "price": 0.42,
                   "chg_24h": 0.03, "spread": 0.02, "volume": 3200000.0 }]
    });
    Mock::given(method("GET"))
        .and(path("/v1/watchlists/default"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let view = client(&server.uri())
        .watchlist("default")
        .await
        .expect("ok");
    assert_eq!(view.name, "macro");
    assert_eq!(view.rows.len(), 1);
}

#[tokio::test]
async fn bearer_token_is_attached() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/markets/will-fed-cut/book"))
        .and(header("authorization", "Bearer secret-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bids": [], "asks": [], "mid": 0.5
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = RestClient::new(&ServerConfig {
        url: server.uri(),
        bearer_token: Some("secret-token".to_owned()),
    })
    .expect("client builds");
    client
        .book("will-fed-cut")
        .await
        .expect("authorized request succeeds");
}

#[tokio::test]
async fn problem_json_maps_to_api_error() {
    let server = MockServer::start().await;
    let problem = json!({
        "code": "not_found",
        "title": "market not found",
        "status": 404,
        "detail": "no market with that slug"
    });
    Mock::given(method("GET"))
        .and(path("/v1/markets/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(problem))
        .mount(&server)
        .await;

    let err = client(&server.uri())
        .market("missing")
        .await
        .expect_err("should fail");
    match err {
        ClientError::Api(p) => {
            assert_eq!(p.code, ErrorCode::NotFound);
            assert_eq!(p.status, 404);
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn unparseable_error_body_maps_to_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/markets/boom"))
        .respond_with(ResponseTemplate::new(500).set_body_string("kaboom"))
        .mount(&server)
        .await;

    let err = client(&server.uri())
        .market("boom")
        .await
        .expect_err("should fail");
    match err {
        ClientError::Status { status, body } => {
            assert_eq!(status, 500);
            assert_eq!(body, "kaboom");
        }
        other => panic!("expected Status error, got {other:?}"),
    }
}
