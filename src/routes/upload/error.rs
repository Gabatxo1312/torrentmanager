use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)), visibility(pub))]
pub enum UploadError {
    #[snafu(display("Please give a collection tag to place the content"))]
    MissingCollection,
    #[snafu(display("Wrong collection: {collection}"))]
    WrongCollection { collection: String },
    #[snafu(display("Failed to read torrent file {}:\n{}", path.display(), source))]
    FailedReadTorrent {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Wrong uploaded torrent:\n{source}"))]
    WrongTorrent {
        source: hightorrent_api::hightorrent::TorrentFileError,
    },
    #[snafu(display("Wrong uploaded magnet:\n{source}"))]
    WrongMagnet {
        source: hightorrent_api::hightorrent::MagnetLinkError,
    },
    #[snafu(display("Please give a torrent file or magnet URI"))]
    MissingTorrentOrMagnet,
    #[snafu(display("You gave a torrent AND a magnet URI. Please choose wisely"))]
    BothTorrentAndMagnet,
    #[snafu(display("Failed to write torrent/magnet to disk:\n{source}"))]
    FailedDatabase {
        source: crate::database::DatabaseError,
    },
    #[snafu(display("Failed to write torrent/magnet to disk:\n{source}"))]
    FailedWrite { source: std::io::Error },
    //#[snafu(display("Duplicate torrent: {}", name))]
    //DuplicateTorrent { name: String },
    #[snafu(display("The proposed uploaded is invalid!"))]
    WrongUploadPath,
    #[snafu(display("The submitted UploadID does not exist: {id}"))]
    WrongUploadID { id: String },
    #[snafu(display("The submitted torrent/magnet is already in qbittorrent backend: {id}"))]
    DuplicateBackendTorrent { id: String },
    #[snafu(display("Could not add or check torrent/magnet because of backend error:\n{source}"))]
    UploadBackend { source: hightorrent_api::ApiError },
}
