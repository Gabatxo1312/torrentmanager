use chrono::{Duration, TimeZone, Utc};
use rocket_dyn_templates::Template;
use hightorrent_api::hightorrent::{SingleTarget, Torrent, TorrentList};
use hightorrent_api::Api;

use crate::templating::menu::MenuEntry;
use crate::{AppState, Context};

fn is_ongoing(t: &Torrent) -> bool {
    t.progress < 100
        && !more_than_x_hours(t.date_start, 24)
        && t.state != "pausedUP"
        && t.state != "pausedDL"
}

fn is_stuck(t: &Torrent) -> bool {
    t.progress < 100
        && more_than_x_hours(t.date_start, 24)
        && &t.state != "pausedUP"
        && &t.state != "pausedDL"
}

fn is_unmanaged(_t: &Torrent) -> bool {
    //! t.tags.contains(&String::from("TorrentManager"))
    // TODO: enable when we have migrated everything
    false
}

pub struct ProgressCounter {
    pub everything: u32,
    pub ongoing: u32,
    pub stuck: u32,
    pub unmanaged: u32,
}

impl ProgressCounter {
    fn from_list(list: &TorrentList) -> ProgressCounter {
        let mut everything: u32 = 0;
        let mut ongoing: u32 = 0;
        let mut stuck: u32 = 0;
        let mut unmanaged: u32 = 0;

        for t in list.clone() {
            everything += 1;
            if is_unmanaged(&t) {
                unmanaged += 1;
            } else {
                if is_ongoing(&t) {
                    ongoing += 1;
                }
                if is_stuck(&t) {
                    stuck += 1;
                }
            }
        }
        ProgressCounter {
            everything,
            ongoing,
            stuck,
            unmanaged,
        }
    }
}

pub fn progress_menu(counter: ProgressCounter) -> Vec<MenuEntry> {
    let all = MenuEntry {
        link: String::from("/progress"),
        icon: String::from("fa-globe"),
        name: format!("Everything ({})", counter.everything),
    };

    let ongoing = MenuEntry {
        link: String::from("/progress/ongoing"),
        icon: String::from("fa-spinner"),
        name: format!("Ongoing ({})", counter.ongoing),
    };
    let stuck = MenuEntry {
        link: String::from("/progress/stuck"),
        icon: String::from("fa-ban"),
        name: format!("Stuck ({})", counter.stuck),
    };
    let unmanaged = MenuEntry {
        link: String::from("/progress/unmanaged"),
        icon: String::from("fa-info-circle"),
        name: format!("Unmanaged ({})", counter.unmanaged),
    };
    vec![all, ongoing, stuck, unmanaged]
}

async fn progress_context<P>(state: &AppState, filter: P) -> Context
where
    P: Fn(&Torrent) -> bool,
{
    let mut context = state.context();
    match state.api.list().await {
        Ok(l) => {
            // Filter
            let counter = ProgressCounter::from_list(&l);
            context.insert_vec("progress_menu", progress_menu(counter));
            context.insert_vec(
                "torrents",
                l.into_iter().filter(filter).collect::<Vec<Torrent>>(),
            );
        }
        Err(e) => {
            context.insert_vec("torrents", Vec::<String>::new());
            context.error(e);
        }
    }
    context
}

fn more_than_x_hours(ref_date: i64, hours: u64) -> bool {
    let old_date = Utc.timestamp_opt(ref_date, 0).unwrap();
    let duration = Duration::hours(hours as i64);
    let now = Utc::now();
    old_date + duration < now
}

#[get("/")]
pub async fn index(state: AppState) -> Template {
    debug!("Displaying progress");
    let context = progress_context(&state, |_t| true).await;
    Template::render("progress", &context)
}

#[get("/ongoing")]
pub async fn ongoing(state: AppState) -> Template {
    let context = progress_context(&state, is_ongoing).await;
    Template::render("progress", &context)
}

#[get("/stuck")]
pub async fn stuck(state: AppState) -> Template {
    let context = progress_context(&state, is_stuck).await;
    Template::render("progress", &context)
}

#[get("/unmanaged")]
pub async fn unmanaged(state: AppState) -> Template {
    let context = progress_context(&state, is_unmanaged).await;
    Template::render("progress", &context)
}

#[get("/hash/<hash>")]
pub async fn hash(hash: String, state: AppState) -> Template {
    let mut context = state.context();

    match state.api.list().await {
        Ok(list) => match SingleTarget::new(&hash) {
            Ok(target) => {
                let counter = ProgressCounter::from_list(&list);
                context.insert_vec("progress_menu", progress_menu(counter));

                if let Some(found_torrent) = list.get(&target) {
                    match state.api.get_files(&target).await {
                        Ok(files) => {
                            context.insert_vec("torrents", vec![found_torrent]);
                            context.insert_vec("files", files);
                        }
                        Err(e) => {
                            context.insert_vec("torrents", Vec::<String>::new());
                            context.error(e);
                        }
                    }
                } else {
                    context.insert_vec("torrents", Vec::<String>::new());
                    context.error_owned(format!("No such torrent: {}", hash));
                }
            }
            Err(e) => {
                context.insert_vec("torrents", Vec::<String>::new());
                context.error(e);
            }
        },
        Err(e) => {
            context.insert_vec("torrents", Vec::<String>::new());
            context.error(e);
        }
    }

    Template::render("progress", &context)
}
