#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate glib;
#[macro_use]
extern crate gtk_macros;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use gettextrs::*;

mod app;
mod backend;
mod config;
mod db;
mod error;
mod path;
mod ui;

use crate::app::SoukApplication;

fn main() {
    // Initialize logger
    pretty_env_logger::init();

    // Initialize GTK
    gtk::init().expect("Failed to initialize GTK.");

    // Initialize libhandy
    libhandy::init();

    // Initialize custom widgets
    crate::ui::pages::init();
    crate::ui::package_widgets::init();

    // Initialize paths
    path::init().expect("Unable to create paths.");

    // Setup language / translations
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::PKGNAME, config::LOCALEDIR);
    textdomain(config::PKGNAME);

    // Load gresources
    let res = gio::Resource::load(
        config::PKGDATADIR.to_owned() + &format!("/{}.gresource", config::APP_ID),
    )
    .expect("Could not load resources");
    gio::resources_register(&res);

    // Start application itself
    // Run app itself
    SoukApplication::run();
}
