use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Debug, Parser)]
/// Demo application for tokio-listener
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity,

    #[clap(flatten)]
    pub listener: tokio_listener::ListenerAddressPositional,
}
