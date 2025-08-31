use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Debug, Parser)]
/// Demo application for tokio-listener
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    pub listen: Option<tokio_listener::ListenerAddress>,
}
