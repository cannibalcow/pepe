pub mod srclient;
pub mod srpoll;
pub mod srws;

use ezsockets::Server;
use srpoll::srpoll::{SrPollOptions, SrPoller};
use srws::srws::SrTrafficMessageServer;
use tokio;
use tracing::{event, Level};

#[tokio::main()]
async fn main() {
    let (tx, rx) = tokio::sync::broadcast::channel(16);
    let options = SrPollOptions::new(10);
    let mut poller = SrPoller::new(tx, options);
    let polling_enabled = true;

    if polling_enabled {
        tokio::spawn(async move {
            poller.poll().await;
        });
    }

    tracing_subscriber::fmt::init();

    let (server, _) = Server::create(|_server| SrTrafficMessageServer { rx });

    event!(Level::INFO, "Starting server on port 8080");
    ezsockets::tungstenite::run(server, "127.0.0.1:8080", |_socket| async move { Ok(()) })
        .await
        .unwrap();
}
