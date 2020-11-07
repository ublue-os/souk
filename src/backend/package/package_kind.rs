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

impl std::string::ToString for SoukPackageKind {
    fn to_string(&self) -> String {
        match self {
            SoukPackageKind::App => "app".to_string(),
            SoukPackageKind::Runtime => "runtime".to_string(),
            SoukPackageKind::Extension => "extension".to_string(),
        }
    }
}
