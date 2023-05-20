pub mod srargs {
    use clap::Parser;
    use tracing::{event, Level};

    #[derive(Parser, Debug)]
    #[command(version, long_about = None)]
    #[command(author = "Daniel Heldt")]
    #[command(about = "Swedish traffic messages websocket service")]
    pub struct PepeArgs {
        #[arg(short, long, default_value_t = 8080)]
        pub port: usize,

        #[arg(short, long, default_value_t = String::from("0.0.0.0"))]
        pub address: String,

        #[arg(
            long,
            default_value_t = 10,
            help = "Interval for polling Swedish Radio API in seconds"
        )]
        pub polling_interval: usize,

        #[arg(
            short,
            long,
            default_value_t = 32,
            help = "Capacity of potentional receivers. Good rule of thumb is expected amount times two plus one."
        )]
        pub channel_capacity: usize,
    }

    impl PepeArgs {
        pub fn bind_address(&self) -> String {
            format!("{}:{}", self.address, self.port)
        }

        pub fn log_configuration(&self) {
            event!(
                Level::INFO,
                "Starting server on port {}",
                self.bind_address()
            );

            event!(
                Level::INFO,
                "Channel capacity set to {}",
                self.channel_capacity
            );

            event!(
                Level::INFO,
                "Swedish Radio API polling interval {}",
                self.polling_interval
            );
        }
    }
}
