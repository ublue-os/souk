use glib::StaticType;

mod action_button;
pub use action_button::SoukActionButton;

mod versions_box;
pub use versions_box::VersionsBox;

mod project_urls_box;
pub use project_urls_box::ProjectUrlsBox;

mod screenshots_box;
pub use screenshots_box::ScreenshotsBox;

mod url_row;
pub use url_row::SoukUrlRow;

use crate::backend::SoukPackage;

pub trait PackageWidget {
    fn new() -> Self
    where
        Self: Sized;

    fn set_package(&self, package: &SoukPackage);

    fn reset(&self);
}

pub fn init() {
    SoukUrlRow::static_type();
}
