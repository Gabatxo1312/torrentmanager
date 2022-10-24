use std::path::PathBuf;

use crate::Context;
use crate::utils::{magnet_hash, torrent_hash};
use super::UploadForm;

pub enum Content {
    Magnet(MagnetContent),
    Torrent(TorrentContent),
}

impl Content {
    pub async fn from_form(form: &mut UploadForm<'_>, context: &mut Context) -> Option<Content> {
        // TODO: Why does rocket return Some(form.torrent) and Some(magnet) even in strict mode
        // when the submission was empty?!
        let temp_file = form.torrent.as_mut().unwrap();
        let magnet = form.magnet.unwrap();

        if temp_file.len() != 0 {
            // TODO: write errors and create correct path
            temp_file.persist_to("/tmp/lol.torrent").await.unwrap();
            match torrent_hash("/tmp/lol.torrent") {
                Ok(hash) => {
                    Some(Content::Torrent(TorrentContent {
                        hash,
                        path: PathBuf::from("/tmp/lol.torrent"),
                    }))
                }, Err(e) => {
                    context.error_owned(format!("{}", e));
                    None
                }
            }
        } else if magnet == "" {
            context.error("Please provide a magnet URI or a torrent file");
            None
        } else {
            match magnet_hash(magnet) {
                Ok(hash) => {
                    Some(Content::Magnet(MagnetContent {
                        hash,
                        uri: magnet.to_string(),
                    }))
                }, Err(e) => {
                    context.error_owned(format!("{}", e));
                    None
                }
            }
        }
    }
}

pub struct MagnetContent {
    hash: String,
    uri: String,
}

pub struct TorrentContent {
    hash: String,
    path: PathBuf,
}
