use gio::prelude::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy, GEnum)]
#[repr(u32)]
#[genum(type_name = "SoukPackageKind")]
pub enum SoukPackageKind {
    App = 0,
    Runtime = 1,
    Extension = 2,
}

impl Default for SoukPackageKind {
    fn default() -> Self {
        SoukPackageKind::App
    }
}
