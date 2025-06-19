//use rocket::{request::FromParam, State};
// use rocket::State;
use rocket_dyn_templates::Template;
//use rocket_dyn_templates::tera::to_value;
//use qbt::torrent::{ApiTorrent as Torrent, TorrentList};
//
//use crate::{AppState, Context};
//use crate::templating::menu::MenuEntry;
use crate::AppState;
use hightorrent_api::Api;

// uub struct TorrentID(String);

//impl<'r> FromParam<'r> for TorrentID<'r> {
//    type Error = String;
//
//    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
//        // We can convert `param` into a `str` since we'll check every
//        // character for safety later.
//        let (key, val_str) = match param.find(':') {
//            Some(i) if i > 0 => (&param[..i], &param[(i + 1)..]),
//            _ => return Err(param)
//        };
//
//        if !key.chars().all(|c| c.is_ascii_alphabetic()) {
//            return Err(param);
//        }
//
//        val_str.parse()
//            .map(|value| MyParam { key, value })
//            .map_err(|_| param)
//    }
//}
//
#[get("/<_id>")]
pub async fn get(state: AppState, _id: &str) -> Template {
    //let context = progress_context(&state, |_t| true).await;
    let context = state.context();
    Template::render("progress", &context)
}

#[get("/")]
pub async fn get_chores(state: AppState) -> Template {
    let mut context = state.context();
    let _torrents = match state.api.list().await {
        Ok(l) => l,
        Err(e) => {
            context.error(e);
            return Template::render("index", &context);
        }
    };

    let context = state.context();
    Template::render("progress", &context)
}
