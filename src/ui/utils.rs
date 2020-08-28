use appstream::types::{License, TranslatableString};
use appstream::enums::Icon;
use chrono::{DateTime, Utc};
use flatpak::prelude::*;
use gio::prelude::*;
use gtk::prelude::*;

use std::path::PathBuf;

pub fn set_label(label: &gtk::Label, text: Option<String>) {
    match text {
        Some(text) => label.set_text(&text),
        None => label.set_text("–"),
    };
}

pub fn set_label_translatable_string(label: &gtk::Label, text: Option<TranslatableString>) {
    match text {
        Some(text) => label.set_text(&text.get_default().unwrap_or(&"???".to_string())),
        None => label.set_text("–"),
    };
}

pub fn set_license_label(label: &gtk::Label, license: Option<License>) {
    match license {
        Some(license) => label.set_text(&license.0),
        None => label.set_text("–"),
    };
}

pub fn set_date_label(label: &gtk::Label, date: Option<DateTime<Utc>>) {
    match date {
        Some(date) => label.set_text(&date.format("%Y-%m-%d").to_string()),
        None => label.set_text("–"),
    };
}

pub fn set_icon(remote: &flatpak::Remote, image: &gtk::Image, icon: &Icon, size: i32) {
    let appstream_dir = remote.get_appstream_dir(Some("x86_64")).unwrap();
    let mut path = PathBuf::new();
    path.push(appstream_dir.get_path().unwrap().to_str().unwrap());
    path.push(format!("icons/{}x{}/", size, size));

    match icon {
        Icon::Cached {path: name, width: _, height: _} => {
            path.push(name);
            image.set_from_file(&path);
        }
        _ => (),
    };
}

// Removes all child items
pub fn remove_all_items<T>(container: &T)
where
    T: IsA<gtk::Container> + gtk::ContainerExt,
{
    let children = container.get_children();
    for widget in children {
        container.remove(&widget);
    }
}
