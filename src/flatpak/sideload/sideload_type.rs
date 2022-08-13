use gio::prelude::*;
use gio::File;
use glib::Enum;
use gtk::{gio, glib};

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkSideloadType")]
pub enum SkSideloadType {
    REF,
    REPO,
    BUNDLE,
    NONE,
}

impl SkSideloadType {
    pub fn determine_type(file: &File) -> SkSideloadType {
        let file = file.path().unwrap();

        match file.extension().unwrap_or_default().to_str().unwrap() {
            "flatpakref" => SkSideloadType::REF,
            "flatpakrepo" => SkSideloadType::REPO,
            "flatpak" => SkSideloadType::BUNDLE,
            _ => SkSideloadType::NONE,
        }
    }
}
