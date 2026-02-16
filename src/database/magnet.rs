use chrono::Utc;
use hightorrent_api::hightorrent::{MagnetLink, MagnetLinkError, TorrentID};
use sea_orm::entity::prelude::*;
use sea_orm::*;
use snafu::prelude::*;

use crate::database::operation::*;
use crate::extractors::user::User;
use crate::routes::magnet::MagnetForm;
use crate::state::AppState;
use crate::state::logger::LoggerError;

/// A category to store associated files.
///
/// Each category has a name and an associated path on disk, where
/// symlinks to the content will be created.
// TODO: typed model fields
// see https://github.com/SeaQL/sea-orm/issues/2811
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "magnet")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub torrent_id: TorrentID,
    pub link: MagnetLink,
    pub name: String,
    pub resolved: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum MagnetError {
    #[snafu(display("The magnet is invalid"))]
    InvalidMagnet { source: MagnetLinkError },
    // TODO: this is not an error
    // we should redirect to the magnet page (eg. progress)
    #[snafu(display("There is already a magnet with the TorrentID `{torrent_id}`"))]
    TorrentIDTaken { torrent_id: String },
    #[snafu(display("Database error"))]
    DB { source: sea_orm::DbErr },
    #[snafu(display("The magnet (ID: {id}) does not exist"))]
    NotFound { id: i32 },
    #[snafu(display("Failed to save the operation log"))]
    Logger { source: LoggerError },
}

#[derive(Clone, Debug)]
pub struct MagnetOperator {
    pub state: AppState,
    pub user: Option<User>,
}

impl MagnetOperator {
    pub fn new(state: AppState, user: Option<User>) -> Self {
        Self { state, user }
    }

    /// List magnets
    ///
    /// Should not fail, unless SQLite was corrupted for some reason.
    pub async fn list(&self) -> Result<Vec<Model>, MagnetError> {
        Entity::find()
            .all(&self.state.database)
            .await
            .context(DBSnafu)
    }

    /// List unresolved magnet
    ///
    /// Should not fail, unless SQLite was corrupted for some reason.
    pub async fn resolved_list_unimported(&self) -> Result<Vec<Model>, MagnetError> {
        Entity::find()
            .filter(Column::Resolved.eq(true))
            // .filter(Column::TorrentId.is_null())
            .all(&self.state.database)
            .await
            .context(DBSnafu)
    }

    pub async fn get(&self, id: i32) -> Result<Model, MagnetError> {
        let db = &self.state.database;

        Entity::find_by_id(id)
            .one(db)
            .await
            .context(DBSnafu)?
            .ok_or(MagnetError::NotFound { id })
    }

    /// Delete an uploaded magnet
    pub async fn delete(&self, id: i32) -> Result<String, MagnetError> {
        let db = &self.state.database;

        let uploaded_magnet = Entity::find_by_id(id)
            .one(db)
            .await
            .context(DBSnafu)?
            .ok_or(MagnetError::NotFound { id })?;

        let clone: Model = uploaded_magnet.clone();
        uploaded_magnet.delete(db).await.context(DBSnafu)?;

        let operation_log = OperationLog {
            user: self.user.clone(),
            date: Utc::now(),
            table: Table::Magnet,
            operation: OperationType::Delete,
            operation_id: OperationId {
                object_id: clone.id,
                name: clone.name.to_owned(),
            },
            operation_form: None,
        };

        self.state
            .logger
            .write(operation_log)
            .await
            .context(LoggerSnafu)?;

        Ok(clone.name)
    }

    /// Create a new uploaded magnet
    ///
    /// Fails if:
    ///
    /// - the magnet is invalid
    pub async fn create(&self, f: &MagnetForm) -> Result<Model, MagnetError> {
        let magnet = MagnetLink::new(&f.magnet).context(InvalidMagnetSnafu)?;

        // Check duplicates
        let list = self.list().await?;

        if list.iter().any(|x| x.torrent_id == magnet.id()) {
            return Err(MagnetError::TorrentIDTaken {
                torrent_id: magnet.id().to_string(),
            });
        }

        let model = ActiveModel {
            torrent_id: Set(magnet.id()),
            link: Set(magnet.clone()),
            name: Set(magnet.name().to_string()),
            // TODO: check if we already have the torrent in which case it's already resolved!
            resolved: Set(false),
            ..Default::default()
        }
        .save(&self.state.database)
        .await
        .context(DBSnafu)?;

        // Should not fail
        let model = model.try_into_model().unwrap();

        let operation_log = OperationLog {
            user: self.user.clone(),
            date: Utc::now(),
            table: Table::Magnet,
            operation: OperationType::Create,
            operation_id: OperationId {
                object_id: model.id.to_owned(),
                name: model.name.to_string(),
            },
            operation_form: Some(Operation::Magnet(f.clone())),
        };

        self.state
            .logger
            .write(operation_log)
            .await
            .context(LoggerSnafu)?;

        Ok(model)
    }
}
