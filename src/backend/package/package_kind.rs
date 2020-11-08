use glib::KeyFile;

#[derive(Debug, Eq, PartialEq, Clone, Copy, GEnum)]
#[repr(u32)]
#[genum(type_name = "SoukPackageKind")]
pub enum SoukPackageKind {
    App = 0,
    Runtime = 1,
    Extension = 2,
}

impl SoukPackageKind {
    pub fn from_keyfile(keyfile: KeyFile) -> Self {
        if keyfile.has_group("ExtensionOf") {
            return SoukPackageKind::Extension;
        }
        if keyfile.has_group("Runtime") {
            return SoukPackageKind::Runtime;
        }
        SoukPackageKind::App
    }
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

impl From<SoukPackageKind> for flatpak::RefKind {
    fn from(kind: SoukPackageKind) -> Self {
        match kind {
            SoukPackageKind::App => flatpak::RefKind::App,
            SoukPackageKind::Runtime => flatpak::RefKind::Runtime,
            SoukPackageKind::Extension => flatpak::RefKind::Runtime,
        }
    }
}
