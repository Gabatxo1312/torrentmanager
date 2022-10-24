use rocket::fs::TempFile;
use rocket::form::FromForm;

use crate::Context;
use content::Content;

// TODO: move content
pub mod content;
pub mod index;
pub mod step1;

pub struct ValidUploadForm {
    content: Content,
    collection: String,
}

#[derive(Debug, FromForm)]
pub struct UploadForm<'r> {
    collection: Option<&'r str>,
    magnet: Option<&'r str>,
    torrent: Option<TempFile<'r>>,
}

impl UploadForm<'_> {
    pub async fn validate(&mut self, context: &mut Context) -> Option<ValidUploadForm> {

        if self.collection.is_none() {
            context.error("Please give a collection tag to place the content");
        } else {
            // TODO: check if collection actually exists
            
        }

        let content = Content::from_form(self, context).await;

        match (self.collection, content) {
            (Some(collection), Some(content)) => {
                Some(ValidUploadForm {
                    collection: collection.to_string(),
                    content,
                })
            }, _ => {
                // collection or content is missing
                None
            }
        }
    }
}
