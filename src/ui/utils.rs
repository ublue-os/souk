use appstream::{MarkupTranslatableString, TranslatableString};
use chrono::{DateTime, Utc};
use gio::prelude::*;
use gtk::prelude::*;
use html2pango::block::markup_html;
use html2pango::block::HtmlBlock;
use libhandy::prelude::*;

use std::path::PathBuf;

use crate::backend::SoukPackage;

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

pub fn set_date_label(label: &gtk::Label, date: Option<DateTime<Utc>>) {
    match date {
        Some(date) => label.set_text(&date.format("%Y-%m-%d").to_string()),
        None => label.set_text("–"),
    };
}

pub fn set_icon(package: &SoukPackage, image: &gtk::Image, size: i32) {
    let remote: String = package.get_remote();
    let name: String = package.get_name();

    let mut path = PathBuf::new();
    path.push(format!(
        "/var/lib/flatpak/appstream/{}/{}/active/icons/{}x{}/{}.png",
        remote,
        std::env::consts::ARCH,
        size,
        size,
        name
    ));

    image.set_from_file(&path);
}

pub fn show_error_dialog(message: &str) {
    let dialog = gtk::MessageDialog::new::<gtk::MessageDialog>(
        None,
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Close,
        &format!("<span font_family=\"monospace\">{}</span>", message),
    );

    dialog.set_title("Something went wrong");
    dialog.set_property_use_markup(true);

    dialog.show();
    dialog.connect_response(|d, _| d.hide());
}

pub fn clear_flowbox(flowbox: &gtk::FlowBox) {
    let listmodel = flowbox.observe_children().unwrap();
    while let Some(o) = listmodel.get_object(0) {
        let widget = o.clone().downcast::<gtk::Widget>().unwrap();
        flowbox.remove(&widget);
    }
}

pub fn clear_carousel(carousel: &libhandy::Carousel) {
    for _ in 0..carousel.get_n_pages() {
        let page = carousel.get_nth_page(0).unwrap();
        carousel.remove(&page);
    }
}
