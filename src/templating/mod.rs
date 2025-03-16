pub mod context;
pub mod menu;
pub use menu::EntryFn;
pub use menu::MenuEntry;
pub use menu::MenuFn;

// zola macro
#[macro_export]
macro_rules! required_arg {
    ($ty: ty, $e: expr, $err: expr) => {
        match $e {
            Some(v) => match from_value::<$ty>(v.clone()) {
                Ok(u) => u,
                Err(_) => return Err($err.into()),
            },
            None => return Err($err.into()),
        }
    };
}

// zola macro
#[macro_export]
macro_rules! optional_arg {
    ($ty: ty, $e: expr, $err: expr) => {
        match $e {
            Some(v) => match from_value::<$ty>(v.clone()) {
                Ok(u) => Some(u),
                Err(_) => return Err($err.into()),
            },
            None => None,
        }
    };
}

/// Customize tera instance to be used by the app
pub fn tera() -> impl rocket::fairing::Fairing {
    rocket_dyn_templates::Template::custom(|engines| {
        engines.tera.register_function("menu_entry", EntryFn);
        engines.tera.register_function("menu", MenuFn);
    })
}
