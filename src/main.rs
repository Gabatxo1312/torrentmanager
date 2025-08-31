use clap::Parser;
use snafu::ErrorCompat;
use tokio_listener::{Listener, SystemOptions, UserOptions};

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

    let config = if let Some(config_path) = &cli_args.config {
        // Config file supplied from CLI
        AppConfig::load(config_path).await?
    } else {
        // Default config
        AppConfig::load_from_xdg().await?
    };

    // CLI listen option has precedence over config file
    let listener = cli_args.listen.as_ref().unwrap_or(&config.listen);

    let sys_opts = SystemOptions::default();
    let mut usr_opts = UserOptions::default();
    // Always remove previous socket before binding
    // TODO: check file lock to see if socket still in use by other process
    usr_opts.unix_listen_unlink = true;

    // Binds to the requested listener to start the server
    let listener = Listener::bind(listener, &sys_opts, &usr_opts)
        .await
        .unwrap();

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
