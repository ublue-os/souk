use crate::config;
use gtk::prelude::*;

pub fn show_about_dialog(window: gtk::ApplicationWindow) {
    let vcs_tag = config::VCS_TAG;
    let version: String = match config::PROFILE {
        "development" => format!("{} \n(Development Commit {})", config::VERSION, vcs_tag),
        "beta" => format!("Beta {}", config::VERSION.split_at(4).1),
        _ => format!("{}-stable", config::VERSION),
    };

    let dialog = gtk::AboutDialog::new();
    dialog.set_program_name(config::NAME);
    dialog.set_logo_icon_name(Some(config::APP_ID));
    dialog.set_license_type(gtk::License::Gpl30);
    dialog.set_version(Some(version.as_str()));
    dialog.set_transient_for(Some(&window));
    dialog.set_modal(true);

    dialog.set_authors(&["Felix Häcker", "Christopher Davis"]);
    dialog.set_artists(&["Felix Häcker"]);

    dialog.connect_response(|dialog, _| dialog.close());
    dialog.show();
}
