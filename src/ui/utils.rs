use appstream::enums::Icon;
use appstream::{License, MarkupTranslatableString, TranslatableString};
use chrono::{DateTime, Utc};
use gio::prelude::*;
use gtk::prelude::*;
use html2pango::*;

use std::path::PathBuf;

use crate::backend::Package;

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

pub fn set_label_markup_translatable_string(
    label: &gtk::Label,
    text: Option<MarkupTranslatableString>,
) {
    match text {
        Some(t) => {
            let text = &t.get_default().unwrap_or(&"???".to_string()).to_string();
            let markup = markup(&text);
            label.set_use_markup(true);
            label.set_markup(&markup);
        }
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

pub fn set_icon(package: &Package, image: &gtk::Image, size: i32) {
    // TODO: Don't hardcode system installation
    let mut path = PathBuf::new();
    path.push(format!(
        "/var/lib/flatpak/appstream/{}/x86_64/active/icons/{}x{}/",
        package.remote, size, size
    ));

    let icon = match package.clone().component.icons.pop() {
        Some(icon) => icon,
        None => {
            debug!("Unable to find icon for package {}", package.app_id);
            return;
        }
    };

    match icon {
        Icon::Cached {
            path: name,
            width: _,
            height: _,
        } => {
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
