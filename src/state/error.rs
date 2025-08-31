use snafu::prelude::*;

use super::free_space::FreeSpaceError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum AppStateError {
    #[snafu(display("Failed to initialize the torrent API"))]
    InitAPI { source: hightorrent_api::ApiError },
    #[snafu(display("Failed to communicate with the torrent client"))]
    API { source: hightorrent_api::ApiError },
    #[snafu(display("Failed to get free space information"))]
    FreeSpace { source: FreeSpaceError },
}
