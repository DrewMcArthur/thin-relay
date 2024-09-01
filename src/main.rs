use dashmap::DashMap;
use futures::stream::StreamExt;
use futures::{FutureExt, SinkExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use url::Url;
use warp::ws::{Message, WebSocket};
use warp::Filter;

type Clients = Arc<DashMap<String, broadcast::Sender<String>>>;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let clients: Clients = Arc::new(DashMap::new());
    // todo: where do we get a list of all the PDS urls?
    let pds_urls = vec!["lionsmane.us-east.host.bsky.network"]
        .into_iter()
        .map(|url| url.to_string())
        .collect();

    start_subscriptions(pds_urls, clients.clone()).await;

    let ws_route = warp::path!("xrpc" / "com.atproto.sync.subscribeRepos")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| client_connected(socket, clients))
        });

    let routes = ws_route.with(warp::cors().allow_any_origin());
    let addr = ([127, 0, 0, 1], 8080);

    logging::info("Server listening on ws://127.0.0.1:8080/xrpc/com.atproto.sync.subscribeRepos");
    warp::serve(routes).run(addr).await;
}

fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn client_connected(ws: WebSocket, clients: Clients) {
    let (mut user_ws_tx, user_ws_rx) = ws.split();

    let (tx, mut rx) = broadcast::channel::<String>(100);
    let id = uuid::Uuid::new_v4().to_string();

    clients.insert(id.clone(), tx);

    let mut client_to_server = Box::pin(
        user_ws_rx
            .for_each(|message| async {
                if let Ok(msg) = message {
                    if msg.is_text() {
                        // Handle client messages if needed
                    }
                }
            })
            .fuse(),
    );

    let mut server_to_client = Box::pin(
        async move {
            while let Ok(message) = rx.recv().await {
                if user_ws_tx.send(Message::text(message)).await.is_err() {
                    break;
                }
            }
        }
        .fuse(),
    );

    futures::select! {
        _ = client_to_server => (),
        _ = server_to_client => (),
    }

    clients.remove(&id);
}

async fn subscribe_to_pds(url: &str, clients: Clients) {
    let url = format!("wss://{}/xrpc/com.atproto.sync.subscribeRepos", url);
    let url = Url::parse(&url).expect("Invalid URL");
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to PDS");
    let (mut pds_ws_tx, mut pds_ws_rx) = ws_stream.split();

    logging::info("Connected to PDS: {pds_url}");

    while let Some(Ok(WsMessage::Text(message))) = pds_ws_rx.next().await {
        let message = Arc::new(message);
        let clients = clients.iter();

        for entry in clients {
            let _ = entry.value().send((*message).clone());
        }
    }

    let _ = pds_ws_tx.close().await;
}

pub async fn start_subscriptions(pds_urls: Vec<String>, clients: Clients) {
    for url in pds_urls {
        let clients = clients.clone();
        task::spawn(async move {
            subscribe_to_pds(&url, clients).await;
        });
    }
}
