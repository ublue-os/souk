use appstream::{MarkupTranslatableString, TranslatableString};
use chrono::{DateTime, Utc};
use gio::prelude::*;
use gtk4::prelude::*;
use html2pango::block::markup_html;
use html2pango::block::HtmlBlock;

use std::path::PathBuf;

use crate::backend::{Package, PackageKind};

pub fn set_label_translatable_string(label: &gtk4::Label, text: Option<TranslatableString>) {
    match text {
        Some(text) => label.set_text(&text.get_default().unwrap_or(&"???".to_string())),
        None => label.set_text("–"),
    };
}

pub fn set_label_markup_translatable_string(
    label: &gtk4::Label,
    text: Option<MarkupTranslatableString>,
) {
    match text {
        Some(t) => {
            let text = &t.get_default().unwrap_or(&"???".to_string()).to_string();
            let markup = render_markup(text).unwrap_or("???".to_string());
            label.set_use_markup(true);
            label.set_markup(&markup);
        }
        None => label.set_text("–"),
    };
}

pub fn render_markup(text: &str) -> Option<String> {
    let mut markup: Vec<String> = vec![];
    if let Ok(blocks) = markup_html(text) {
        for block in blocks {
            let text = match block {
                HtmlBlock::UList(elements) => elements
                    .iter()
                    .map(|li| format!("  • {}", li))
                    .collect::<Vec<String>>()
                    .join("\n"),
                HtmlBlock::OList(elements) => elements
                    .iter()
                    .enumerate()
                    .map(|(i, li)| format!("  {}. {}", i + 1, li))
                    .collect::<Vec<String>>()
                    .join("\n"),
                HtmlBlock::Text(t) => t,
                _ => String::new(),
            };
            markup.push(text + "\n");
        }
        Some(
            markup
                .into_iter()
                .filter(|x| !x.is_empty())
                .collect::<Vec<String>>()
                .join("\n")
                .trim_end()
                .to_string(),
        )
    } else {
        debug!("Could not parse: {}", text);
        None
    }
}

pub fn set_date_label(label: &gtk4::Label, date: Option<DateTime<Utc>>) {
    match date {
        Some(date) => label.set_text(&date.format("%Y-%m-%d").to_string()),
        None => label.set_text("–"),
    };
}

pub fn set_icon(package: &dyn Package, image: &gtk4::Image, size: i32) {
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
            PackageKind::App => image.set_from_icon_name(Some("dialog-question-symbolic")),
            PackageKind::Runtime | PackageKind::Extension => {
                image.set_from_icon_name(Some("system-run-symbolic"))
            }
        };
    }
}

pub fn show_error_dialog(builder: gtk4::Builder, message: &str) {
    let dialog = gtk4::MessageDialog::new::<gtk4::MessageDialog>(
        None,
        gtk4::DialogFlags::MODAL,
        gtk4::MessageType::Error,
        gtk4::ButtonsType::Close,
        &format!("<span font_family=\"monospace\">{}</span>", message),
    );

    dialog.set_title("Something went wrong");
    dialog.set_property_use_markup(true);

    dialog.show();
    dialog.connect_response(|d, _| d.hide());
}

// Removes all child items
pub fn remove_all_items<T>(container: &T)
where
    T: IsA<gtk4::Widget> + gtk4::WidgetExt,
{
    // TODO: Port this too!
    /*
    let children = container.get_children();
    for widget in children {
        container.remove(&widget);
    }*/
}
