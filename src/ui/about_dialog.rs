use crate::config;
use gtk::prelude::*;

pub fn show_about_dialog(window: gtk::ApplicationWindow) {
    let vcs_tag = config::VCS_TAG;
    let version: String = match config::PROFILE {
        "development" => format!("{} \n(Development Commit {})", config::VERSION, vcs_tag),
        "beta" => format!("Beta {}", config::VERSION.split_at(4).1),
        _ => format!("{}-stable", config::VERSION),
    };

    gtk::AboutDialogBuilder::new()
        .program_name(config::NAME)
        .logo_icon_name(config::APP_ID)
        .license_type(gtk::License::Gpl30)
        .version(&version)
        .transient_for(&window)
        .modal(true)
        .authors(vec![
            "Felix Häcker".to_string(),
            "Christopher Davis".to_string(),
        ])
        .artists(vec![
            "Felix Häcker".to_string(),
            "Tobias Bernard".to_string(),
        ])
        .build()
        .show();
}
