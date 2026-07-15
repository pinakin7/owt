//! The multiplexed WebSocket client (`docs/design/tui-client.md` § Connection lifecycle).
//!
//! One WS carries many logical subscriptions keyed by topic. Frames are the
//! `owt-api-types` [`ClientFrame`]/[`ServerFrame`] protocol. [`WsClient`] is the
//! config-bound dialer; [`WsClient::connect`] opens a live socket and hands back a
//! [`WsConnection`] that owns it. The reconnect/resubscribe loop over these primitives
//! lives in [`crate::session`] (ADR-0002).

use futures::{SinkExt, StreamExt};
use owt_api_types::ws::{ClientFrame, ServerFrame};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::config::ServerConfig;
use crate::error::ClientError;

/// The concrete client-side WebSocket stream type produced by [`connect_async`].
type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Encode a client frame to a WebSocket text message.
pub fn encode(frame: &ClientFrame) -> Result<Message, ClientError> {
    let text = serde_json::to_string(frame).map_err(|e| ClientError::Ws(e.to_string()))?;
    Ok(Message::Text(text.into()))
}

/// Decode a WebSocket text message into a server frame.
pub fn decode(msg: &Message) -> Result<ServerFrame, ClientError> {
    match msg {
        Message::Text(text) => {
            serde_json::from_str(text.as_str()).map_err(|e| ClientError::Ws(e.to_string()))
        }
        other => Err(ClientError::Ws(format!(
            "expected text frame, got {other:?}"
        ))),
    }
}

/// A multiplexed WebSocket client bound to one server.
#[derive(Debug, Clone)]
pub struct WsClient {
    /// WebSocket URL derived from the server base (`ws(s)://…/v1/ws`).
    url: String,
    bearer: Option<String>,
}

impl WsClient {
    /// Build a client from the effective server config, deriving the `ws`/`wss` URL
    /// from the REST base.
    pub fn new(cfg: &ServerConfig) -> Result<Self, ClientError> {
        let base = cfg.url.trim_end_matches('/');
        let url = if let Some(rest) = base.strip_prefix("https://") {
            format!("wss://{rest}/v1/ws")
        } else if let Some(rest) = base.strip_prefix("http://") {
            format!("ws://{rest}/v1/ws")
        } else {
            return Err(ClientError::InvalidUrl(cfg.url.clone()));
        };
        Ok(Self {
            url,
            bearer: cfg.bearer_token.clone(),
        })
    }

    /// The resolved WebSocket URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Open a live connection, attaching the bearer token as an `Authorization` header
    /// on the handshake when configured.
    pub async fn connect(&self) -> Result<WsConnection, ClientError> {
        let mut request = self
            .url
            .as_str()
            .into_client_request()
            .map_err(|e| ClientError::Ws(e.to_string()))?;
        if let Some(token) = &self.bearer {
            let value = HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|e| ClientError::Ws(e.to_string()))?;
            request.headers_mut().insert("Authorization", value);
        }
        let (stream, _resp) = connect_async(request)
            .await
            .map_err(|e| ClientError::Ws(e.to_string()))?;
        Ok(WsConnection { stream })
    }
}

/// A live multiplexed WebSocket connection: the socket the [`WsClient`] opened.
#[derive(Debug)]
pub struct WsConnection {
    stream: Socket,
}

impl WsConnection {
    /// Subscribe to a topic.
    pub async fn subscribe(&mut self, topic: &str) -> Result<(), ClientError> {
        self.send_frame(&ClientFrame::Subscribe {
            topic: topic.to_owned(),
        })
        .await
    }

    /// Unsubscribe from a topic.
    pub async fn unsubscribe(&mut self, topic: &str) -> Result<(), ClientError> {
        self.send_frame(&ClientFrame::Unsubscribe {
            topic: topic.to_owned(),
        })
        .await
    }

    /// Send an encoded client frame over the socket.
    async fn send_frame(&mut self, frame: &ClientFrame) -> Result<(), ClientError> {
        let msg = encode(frame)?;
        self.stream
            .send(msg)
            .await
            .map_err(|e| ClientError::Ws(e.to_string()))
    }

    /// Await the next server frame, decoding it from the wire. Returns an error if the
    /// socket closed or yielded a non-text/undecodable frame.
    pub async fn next_frame(&mut self) -> Result<ServerFrame, ClientError> {
        match self.stream.next().await {
            Some(Ok(msg)) => decode(&msg),
            Some(Err(e)) => Err(ClientError::Ws(e.to_string())),
            None => Err(ClientError::Ws("connection closed".to_owned())),
        }
    }
}
