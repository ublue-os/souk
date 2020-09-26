use appstream::{MarkupTranslatableString, TranslatableString};
use chrono::{DateTime, Utc};
use gio::prelude::*;
use gtk::prelude::*;
use html2pango::*;

use std::path::PathBuf;

use crate::backend::{Package, PackageKind};

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

pub fn set_date_label(label: &gtk::Label, date: Option<DateTime<Utc>>) {
    match date {
        Some(date) => label.set_text(&date.format("%Y-%m-%d").to_string()),
        None => label.set_text("–"),
    };
}

pub fn set_icon(package: &dyn Package, image: &gtk::Image, size: i32) {
    let mut path = PathBuf::new();
    path.push(format!(
        "/var/lib/flatpak/appstream/{}/{}/active/icons/{}x{}/{}.png",
        package.remote(),
        std::env::consts::ARCH,
        size,
        size,
        package.name()
    ));

    if path.exists() {
        image.set_from_file(&path);
    } else {
        match package.kind() {
            PackageKind::App => image.set_from_icon_name(
                Some("dialog-question-symbolic"),
                gtk::IconSize::__Unknown(size),
            ),
            PackageKind::Runtime | PackageKind::Extension => image
                .set_from_icon_name(Some("system-run-symbolic"), gtk::IconSize::__Unknown(size)),
        };
    }
}

pub fn show_error_dialog(builder: gtk::Builder, message: &str) {
    let app = builder.get_application().unwrap();
    let window = app.get_active_window().unwrap();

    let dialog = gtk::MessageDialog::new(
        Some(&window),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Close,
        &format!("<span font_family=\"monospace\">{}</span>", message),
    );

    dialog.set_title("Something went wrong");
    dialog.set_property_use_markup(true);

    glib::idle_add_local(move || {
        dialog.run();
        dialog.hide();
        glib::Continue(false)
    });
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
