mod releases_box;
pub use releases_box::ReleasesBox;

mod project_urls_box;
pub use project_urls_box::ProjectUrlsBox;

mod screenshots_box;
pub use screenshots_box::ScreenshotsBox;

use crate::backend::Package;

pub trait PackageWidget {
    fn new() -> Self
    where
        Self: Sized;
    fn set_package(&self, package: Package);
    fn reset(&self);
}
