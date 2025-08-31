use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Debug, Parser)]
/// Demo application for tokio-listener
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[clap(flatten)]
    pub listener: tokio_listener::ListenerAddressPositional,
}
