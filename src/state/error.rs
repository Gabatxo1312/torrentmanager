use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub), context(suffix(Error)))]
pub enum AppError {
    #[snafu(display("Failed to read configuration file {}:\n{}", path.display(), source))]
    // TODO: many config file paths
    NoConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Broken configuration file {}:\n{}", path.canonicalize().unwrap().display(), source))]
    BrokenConfig {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[snafu(display("Failed to connect to qBittorrent:\n{source}"))]
    FailedApi { source: hightorrent_api::ApiError },
    #[snafu(display("QBittorrent client instance {} is not configured", name))]
    NotConfiguredClient { name: String },
}

impl AppError {
    pub fn to_html(&self) -> String {
        self.to_string().replace('\n', "<br>")
    }
}
