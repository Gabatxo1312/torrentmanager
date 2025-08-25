use clap::Parser;

use torrentmanager::serve;

mod cli;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize logging with selected log-level
    // Defaults to errors only (-q to suppress).
    // -v for warning, -vv for info, -vvv for debug, -vvvv for trace
    let cli_args = cli::Args::parse();
        env_logger::Builder::new()
        .filter_level(cli_args.verbosity.log_level_filter())
        .init();

    // Always remove previous socket before binding
    // TODO: check file lock to see if socket still in use by other process
    let mut listener = cli_args.listener;
    listener.listener_options.unix_listen_unlink = true;
    let listener = listener.bind().await.unwrap();

    serve(listener).await;
}
