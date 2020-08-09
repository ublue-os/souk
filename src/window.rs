use gtk::prelude::*;
use flatpak::Installation;
use flatpak::InstallationExt;
use flatpak::prelude::*;

pub struct Window {
    pub widget: gtk::ApplicationWindow,
}

impl Window {

    pub fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/window.ui");
        let widget: gtk::ApplicationWindow = builder.get_object("window").expect("Failed to find the window object");
        let installed_box: gtk::Box = builder.get_object("installed_box").expect("Failed to find the installed_box object");

        let installation = flatpak::Installation::new_system(Some(&gio::Cancellable::new())).unwrap();
        let result = installation.list_installed_refs(Some(&gio::Cancellable::new())).unwrap();

        for package in result {
            package.get_appdata_name().map(|name|{
                let mut v = "".to_string();
                let version = package.get_appdata_version();
                match version{
                    Some(version) => v = version.to_string(),
                    None => (),
                }

                let text = format!("{} - Version: {}", name.to_string(), v);

                let label = gtk::Label::new(Some(&text));
                installed_box.add(&label);
            });
        }

        widget.show_all();

        Self {
            widget,
        }
    }

}

