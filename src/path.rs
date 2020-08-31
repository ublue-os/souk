use crate::config;

use std::fs;
use std::path::PathBuf;

lazy_static! {
    pub static ref DATA: PathBuf = {
        let mut path = glib::get_user_data_dir().unwrap();
        path.push(config::NAME);
        path
    };
    pub static ref CONFIG: PathBuf = {
        let mut path = glib::get_user_config_dir().unwrap();
        path.push(config::NAME);
        path
    };
    pub static ref CACHE: PathBuf = {
        let mut path = glib::get_user_cache_dir().unwrap();
        path.push(config::NAME);
        path
    };
}

pub fn init() -> std::io::Result<()> {
    fs::create_dir_all(DATA.to_owned())?;
    fs::create_dir_all(CONFIG.to_owned())?;
    fs::create_dir_all(CACHE.to_owned())?;
    Ok(())
}
