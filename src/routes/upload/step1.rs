use hightorrent_api::hightorrent::{MagnetLink, TorrentFile};
use hightorrent_api::Api;
use rocket::{
    form::{Form, FromForm},
    fs::TempFile,
};
use rocket_dyn_templates::Template;
use snafu::{OptionExt, ResultExt};

use super::error::*;
use crate::{AppState, Database, UploadID};

use std::path::Path;

/// Extracts the infohash of a magnet link
pub fn magnet_hash<T: AsRef<str>>(magnet: T) -> Result<String, UploadError> {
    let magnet = MagnetLink::new(magnet.as_ref()).context(WrongMagnetError)?;
    Ok(magnet.hash().to_string())
}

/// Extracts the infohash of a torrent file
pub async fn torrent_hash<T: AsRef<Path>>(torrent: T) -> Result<String, UploadError> {
    let torrent_slice = std::fs::read(torrent.as_ref()).context(FailedReadTorrentError {
        path: torrent.as_ref().to_path_buf(),
    })?;
    let torrent = TorrentFile::from_slice(&torrent_slice).context(WrongTorrentError)?;
    Ok(torrent.hash().to_string())
}

#[post("/", data = "<form>")]
/// Second form, when a magnet link was sent
pub async fn post(mut form: Form<UploadForm<'_>>, state: AppState) -> Template {
    let mut context = state.context();

    match form.validate(&state).await {
        Ok(mut valid_form) => {
            // Sort folder entries alphabetically
            valid_form.collection_folders.sort_unstable();
            context.insert_string("collection", &valid_form.collection);
            context.insert_string("content", &valid_form.content);
            context.insert_vec("folders", valid_form.collection_folders);
            Template::render("upload/step1", &context)
        }
        Err(errors) => {
            for error in errors {
                context.error(error);
            }
            Template::render("upload/index", &context)
        }
    }
}

pub struct ValidUploadForm {
    content: UploadID,
    collection: String,
    collection_folders: Vec<String>,
}

#[derive(Debug, FromForm)]
pub struct UploadForm<'r> {
    collection: Option<&'r str>,
    magnet: Option<&'r str>,
    torrent: Option<TempFile<'r>>,
}

impl UploadForm<'_> {
    pub async fn validate(
        &mut self,
        state: &AppState,
    ) -> Result<ValidUploadForm, Vec<UploadError>> {
        match (
            self.verify_content(state).await,
            self.verify_collection(&state.database),
        ) {
            (Ok(content), Ok(collection_folders)) => Ok(ValidUploadForm {
                collection: self.collection.unwrap().to_string(),
                collection_folders,
                content,
            }),
            (Err(e), Ok(_collection)) => Err(vec![e]),
            (Ok(_content), Err(e)) => Err(vec![e]),
            (Err(e1), Err(e2)) => Err(vec![e1, e2]),
        }
    }

    pub fn verify_collection(&self, database: &Database) -> Result<Vec<String>, UploadError> {
        let collection = self.collection.context(MissingCollectionError)?;
        let collections = database.collections();
        let db_collection = collections.get(collection).context(WrongCollectionError {
            collection: collection.to_string(),
        })?;
        let collection_entries = db_collection.folders().context(FailedDatabaseError)?;
        Ok(collection_entries)
    }

    pub async fn verify_content(&mut self, state: &AppState) -> Result<UploadID, UploadError> {
        info!("Verifying uploaded torrent...");
        let magnet = self.magnet.unwrap();
        let torrent = self.torrent.as_mut().unwrap();
        let upload_db = state.database.uploads();

        match (!magnet.is_empty(), torrent.len() != 0) {
            (true, true) => Err(UploadError::BothTorrentAndMagnet),
            (true, false) => {
                let hash = magnet_hash(magnet)?;
                // TODO: check duplicate torrent, requires accessing State.api which is not passed
                // currently... maybe storing api inside database?!
                let id = UploadID::magnet(&hash).context(FailedDatabaseError)?;
                upload_db
                    .persist_upload(&id, magnet)
                    .context(FailedDatabaseError)?;
                // Prevent duplicates
                let target = id.single_target();
                if state
                    .api
                    .get(&target)
                    .await
                    .context(UploadBackendError)?
                    .is_some()
                {
                    Err(UploadError::DuplicateBackendTorrent { id: id.to_string() })
                } else {
                    Ok(id)
                }
            }
            (false, true) => {
                let tmp_dest = temp_file::TempFileBuilder::new()
                    .build()
                    .context(FailedWriteError)?;
                torrent
                    .persist_to(tmp_dest.path())
                    .await
                    .context(FailedWriteError)?;
                let hash = torrent_hash(tmp_dest.path()).await?;
                let id = UploadID::torrent(&hash).context(FailedDatabaseError)?;
                upload_db
                    .persist_upload_copy(&id, tmp_dest.path())
                    .context(FailedDatabaseError)?;
                // Prevent duplicates
                let target = id.single_target();
                if state
                    .api
                    .get(&target)
                    .await
                    .context(UploadBackendError)?
                    .is_some()
                {
                    Err(UploadError::DuplicateBackendTorrent { id: id.to_string() })
                } else {
                    Ok(id)
                }
            }
            (false, false) => Err(UploadError::MissingTorrentOrMagnet),
        }
    }
}
