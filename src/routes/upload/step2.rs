use hightorrent_api::Api;
use rocket::form::{Form, FromForm};
use rocket::State;
use rocket_dyn_templates::Template;
use snafu::{OptionExt, ResultExt};

use super::error::*;
use crate::database::UploadPath;
use crate::utils::unwrap_or_err;
use crate::{AppState, Database, UploadID};

#[post("/step2", data = "<form>")]
/// Second form, when a magnet link was sent
pub async fn post(mut form: Form<ConfirmForm<'_>>, state: &State<AppState>) -> Template {
    let mut context = state.context();

    match form.validate(&state).await {
        Ok(valid_form) => {
            // Not duplicate, we can add...
            if let Err(e) = valid_form
                .content
                .add_to(&state, valid_form.path, &valid_form.collection)
                .await
            {
                context.error(e);
            } else {
                context.insert_string(
                    "truncated_hash",
                    valid_form.content.to_single_target().truncated(),
                );
            }
            Template::render("upload/step2", &context)
        }
        Err(errors) => {
            for error in errors {
                context.error(error);
            }
            Template::render("upload/step1", &context)
        }
    }
}

pub struct ValidConfirmForm {
    content: UploadID,
    collection: String,
    path: UploadPath,
}

#[derive(Debug, FromForm)]
pub struct ConfirmForm<'r> {
    collection: Option<&'r str>,
    content: Option<&'r str>,
    basedir: Option<&'r str>,
    targetdir: Option<&'r str>,
}

impl ConfirmForm<'_> {
    pub async fn validate(
        &mut self,
        state: &AppState,
    ) -> Result<ValidConfirmForm, Vec<UploadError>> {
        let mut errors: Vec<UploadError> = Vec::new();

        let verified_content = unwrap_or_err(self.verify_content(&state).await, &mut errors);
        let _verified_collection =
            unwrap_or_err(self.verify_collection(&state.database), &mut errors);
        let verified_path = unwrap_or_err(self.verify_path(), &mut errors);

        if errors.is_empty() {
            let verified_content = verified_content.unwrap();
            let verified_path = verified_path.unwrap();
            let verified_collection = self.collection.unwrap().to_string();

            Ok(ValidConfirmForm {
                content: verified_content,
                path: verified_path,
                collection: verified_collection,
            })
        } else {
            Err(errors)
        }
    }

    pub fn verify_path(&self) -> Result<UploadPath, UploadError> {
        let basedir = self.basedir.unwrap();
        let targetdir = self.targetdir.unwrap();

        UploadPath::from_str(basedir, targetdir).context(WrongUploadPathError)
    }

    pub fn verify_collection(&self, database: &Database) -> Result<(), UploadError> {
        let collection = self.collection.context(MissingCollectionError)?;
        let _db_collection =
            database
                .collections()
                .get(&collection)
                .context(WrongCollectionError {
                    collection: collection.to_string(),
                })?;
        Ok(())
    }

    pub async fn verify_content(&mut self, state: &AppState) -> Result<UploadID, UploadError> {
        let content = self.content.unwrap();
        let upload_db = state.database.uploads();

        let id = UploadID::from_str(&content).context(FailedDatabaseError)?;
        if upload_db.has_upload(&id).context(FailedDatabaseError)? {
            let target = id.single_target();
            // Prevent duplicates
            if let Some(_) = state.api.get(&target).await.context(UploadBackendError)? {
                Err(UploadError::DuplicateBackendTorrent { id: id.to_string() })
            } else {
                Ok(id)
            }
        } else {
            Err(UploadError::WrongUploadID { id: id.to_string() })
        }
    }
}
