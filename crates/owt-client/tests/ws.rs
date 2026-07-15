//! WebSocket contract tests (testing-strategy.md § API contract / WS conformance).
//!
//! `owt-api` is not implemented yet, so these drive the client against a minimal
//! in-test `tokio-tungstenite` server: a `connect → subscribe → update` round trip over
//! [`WsConnection`], and a drop-and-recover check that [`WsSession`] reconnects and
//! resubscribes the desired topic set.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use owt_api_types::ws::{ClientFrame, ServerFrame};
use owt_client::config::ServerConfig;
use owt_client::{WsClient, WsSession};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

/// Config pointing the WS client at `127.0.0.1:{port}` (→ `ws://…/v1/ws`).
fn cfg(port: u16) -> ServerConfig {
    ServerConfig {
        url: format!("http://127.0.0.1:{port}"),
        bearer_token: None,
    }
}

/// Encode a server `Update` frame as a WS text message.
fn update_msg(topic: &str) -> Message {
    let frame = ServerFrame::Update {
        topic: topic.to_owned(),
        data: serde_json::json!({ "mid": 0.42 }),
    };
    Message::Text(serde_json::to_string(&frame).unwrap().into())
}

/// Read one text frame and assert it is a `ClientFrame` (a subscribe/unsubscribe).
async fn expect_client_frame<S>(ws: &mut tokio_tungstenite::WebSocketStream<S>) -> ClientFrame
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let msg = ws.next().await.expect("a frame").expect("no ws error");
    let text = msg.to_text().expect("text frame");
    serde_json::from_str(text).expect("valid client frame")
}

#[tokio::test]
async fn connect_subscribe_and_receive_update() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let frame = expect_client_frame(&mut ws).await;
        assert!(matches!(frame, ClientFrame::Subscribe { topic } if topic == "market:x:book"));
        ws.send(update_msg("market:x:book")).await.unwrap();
        ws.close(None).await.ok();
    });

    let client = WsClient::new(&cfg(port)).unwrap();
    let mut conn = client.connect().await.expect("connects");
    conn.subscribe("market:x:book")
        .await
        .expect("subscribe sent");

    match conn.next_frame().await.expect("a server frame") {
        ServerFrame::Update { topic, .. } => assert_eq!(topic, "market:x:book"),
        other => panic!("expected update, got {other:?}"),
    }

    server.await.unwrap();
}

#[tokio::test]
async fn session_reconnects_and_resubscribes() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let subs = Arc::new(AtomicUsize::new(0));

    let server_subs = subs.clone();
    let server = tokio::spawn(async move {
        // Connection 1: read the subscribe, then hard-drop to force a reconnect.
        let (s1, _) = listener.accept().await.unwrap();
        let mut ws1 = accept_async(s1).await.unwrap();
        expect_client_frame(&mut ws1).await;
        server_subs.fetch_add(1, Ordering::SeqCst);
        drop(ws1);

        // Connection 2: the driver must resubscribe the desired topic, then we push an
        // update to prove the reconnected socket is live.
        let (s2, _) = listener.accept().await.unwrap();
        let mut ws2 = accept_async(s2).await.unwrap();
        expect_client_frame(&mut ws2).await;
        server_subs.fetch_add(1, Ordering::SeqCst);
        ws2.send(update_msg("market:x:book")).await.unwrap();
        ws2.close(None).await.ok();
    });

    let client = WsClient::new(&cfg(port)).unwrap();
    let (session, mut frames, _conn) = WsSession::spawn(client);
    session.subscribe("market:x:book");

    // The update only arrives if the session reconnected and resubscribed.
    let frame = tokio::time::timeout(Duration::from_secs(5), frames.recv())
        .await
        .expect("update within timeout")
        .expect("frame channel stays open");
    match frame {
        ServerFrame::Update { topic, .. } => assert_eq!(topic, "market:x:book"),
        other => panic!("expected update, got {other:?}"),
    }

    server.await.unwrap();
    assert_eq!(
        subs.load(Ordering::SeqCst),
        2,
        "subscribed on the first connection and resubscribed on the second"
    );
}
