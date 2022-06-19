use gio::prelude::*;
use gio::File;
use glib::Enum;
use gtk::{gio, glib};

#[derive(Copy, Debug, Clone, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkSideloadType")]
pub enum SkSideloadType {
    Ref,
    Repo,
    Bundle,
    None,
}

impl SkSideloadType {
    pub fn determine_type(file: &File) -> SkSideloadType {
        let file = file.path().unwrap();

        match file.extension().unwrap_or_default().to_str().unwrap() {
            "flatpakref" => SkSideloadType::Ref,
            "flatpakrepo" => SkSideloadType::Repo,
            "flatpak" => SkSideloadType::Bundle,
            _ => SkSideloadType::None,
        }
    }
}
