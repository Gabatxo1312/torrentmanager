use camino::Utf8PathBuf;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Debug, Parser)]
/// Demo application for tokio-listener
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    /// Where to load the config from
    /// (default: ~/.local/share/torrentmanager/config.toml)
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<Utf8PathBuf>,

    pub listen: Option<tokio_listener::ListenerAddress>,
}
