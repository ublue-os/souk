mod action_button;
pub use action_button::SoukActionButton;

mod releases_box;
pub use releases_box::ReleasesBox;

mod project_urls_box;
pub use project_urls_box::ProjectUrlsBox;

mod screenshots_box;
pub use screenshots_box::ScreenshotsBox;

use crate::backend::SoukPackage;

pub trait PackageWidget {
    fn new() -> Self
    where
        Self: Sized;

    fn set_package(&self, package: &SoukPackage);

    fn reset(&self);
}
