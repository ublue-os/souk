use crate::config;

use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;

pub static DATA: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = glib::get_user_data_dir();
    path.push(config::NAME);
    path
});

pub static CONFIG: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = glib::get_user_config_dir();
    path.push(config::NAME);
    path
});

pub static CACHE: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = glib::get_user_cache_dir();
    path.push(config::NAME);
    path
});

pub fn init() -> std::io::Result<()> {
    fs::create_dir_all(DATA.to_owned())?;
    fs::create_dir_all(CONFIG.to_owned())?;
    fs::create_dir_all(CACHE.to_owned())?;
    Ok(())
}
