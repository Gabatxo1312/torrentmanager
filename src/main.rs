use clap::Parser;
use snafu::ErrorCompat;

use torrentmanager::config::{AppConfig, ConfigError};
use torrentmanager::serve;

mod cli;

async fn main_inner() -> Result<(), ConfigError> {
    // Initialize logging with selected log-level
    // Defaults to errors only (-q to suppress).
    // -v for warning, -vv for info, -vvv for debug, -vvvv for trace
    let cli_args = cli::Args::parse();
    env_logger::Builder::new()
        .filter_level(cli_args.verbosity.log_level_filter())
        .init();

    // Load default config
    // TODO: load with CLI argument
    let config = AppConfig::load_from_xdg().await?;

    // Always remove previous socket before binding
    // TODO: check file lock to see if socket still in use by other process
    let mut listener = cli_args.listener;
    listener.listener_options.unix_listen_unlink = true;
    let listener = listener.bind().await.unwrap();

    serve(listener, config).await;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(errors) = main_inner().await {
        for error in errors.iter_chain() {
            log::error!("{error}");
        }

        std::process::exit(1);
    }
}
