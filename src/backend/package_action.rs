use gio::prelude::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy, GEnum)]
#[repr(u32)]
#[genum(type_name = "SoukPackageAction")]
pub enum SoukPackageAction {
    None = 0,
    Install = 1,
    Uninstall = 2,
    Update = 3,
}

impl Default for SoukPackageAction {
    fn default() -> Self {
        SoukPackageAction::None
    }
}
