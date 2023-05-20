mod config;
mod sr;
use clap::Parser;
use config::PepeArgs;
use ezsockets::Server;
use sr::{
    poller::{SrPollOptions, SrPoller},
    websocket::SrTrafficMessageServer,
};
use tracing::{event, Level};

#[tokio::main]
async fn main() {
    let args = PepeArgs::parse();
    let (tx, rx) = tokio::sync::broadcast::channel(args.channel_capacity);
    let options = SrPollOptions::new(args.polling_interval);
    let mut poller = SrPoller::new(tx, options);
    let polling_enabled = true;

    if polling_enabled {
        tokio::spawn(async move {
            poller.poll().await;
        });
    }

    tracing_subscriber::fmt::init();

    let (server, _) = Server::create(|_server| SrTrafficMessageServer { rx });

    args.log_configuration();

    match ezsockets::tungstenite::run(server, args.bind_address(), |_socket| async move { Ok(()) })
        .await
    {
        Ok(_) => (),
        Err(err) => event!(Level::ERROR, "{}", err),
    }
}
