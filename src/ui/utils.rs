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

pub fn render_markup_widget(text: Option<MarkupTranslatableString>) -> Option<gtk::Box> {
    let details_box = gtk::Box::new(gtk::Orientation::Vertical, 8);

    match text {
        Some(t) => {
            let text = &t.get_default().unwrap_or(&"???".to_string()).to_string();

            if let Ok(blocks) = markup_html(text) {
                for block in blocks {
                    match block {
                        HtmlBlock::UList(elements) => {
                            let bx = gtk::Box::new(gtk::Orientation::Vertical, 3);
                            for li in elements.iter() {
                                let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                                let bullet = gtk::Label::new(Some("•"));
                                bullet.set_valign(gtk::Align::Start);
                                let w = gtk::Label::new(None);
                                set_label_styles(&w);
                                h_box.append(&bullet);
                                h_box.append(&w);
                                w.set_markup(&li);
                                bx.append(&h_box);
                            }
                            details_box.append(&bx);
                        }
                        HtmlBlock::OList(elements) => {
                            let bx = gtk::Box::new(gtk::Orientation::Vertical, 3);
                            for (i, ol) in elements.iter().enumerate() {
                                let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                                let bullet = gtk::Label::new(Some(&format!("{}.", i + 1)));
                                bullet.set_valign(gtk::Align::Start);
                                let w = gtk::Label::new(None);
                                h_box.append(&bullet);
                                h_box.append(&w);
                                w.set_markup(&ol);
                                set_label_styles(&w);
                                bx.append(&h_box);
                            }
                            details_box.append(&bx);
                        }
                        HtmlBlock::Text(t) => {
                            // TODO html2pango inserts a single newline
                            // after each paragraph. This could change in a future
                            // version.
                            for text in t.split('\n') {
                                let label = gtk::Label::new(Some(&text));
                                set_label_styles(&label);
                                details_box.append(&label);
                            }
                        }
                        _ => (),
                    };
                }
                Some(details_box)
            } else {
                debug!("Could not parse: {}", text);
                Some(gtk::Box::new(gtk::Orientation::Horizontal, 6))
            }
        }
        None => Some(gtk::Box::new(gtk::Orientation::Horizontal, 6)),
    }
}

fn set_label_styles(w: &gtk::Label) {
    w.set_wrap(true);
    w.set_wrap_mode(pango::WrapMode::WordChar);
    w.set_justify(gtk::Justification::Left);
    w.set_xalign(0.0);
    w.set_valign(gtk::Align::Start);
    w.set_halign(gtk::Align::Fill);
}

pub fn render_markup(text: &str) -> Option<String> {
    let mut markup: Vec<String> = vec![];
    if let Ok(blocks) = markup_html(text) {
        for block in blocks {
            if let HtmlBlock::Text(t) = block {
                markup.push(t);
            }
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
