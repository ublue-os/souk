mod utils;
pub mod page;

mod app_buttons_box;
pub use app_buttons_box::AppButtonsBox;

mod app_tile;
pub use app_tile::AppTile;

mod project_urls_box;
pub use project_urls_box::ProjectUrlsBox;

mod releases_box;
pub use releases_box::ReleasesBox;

mod screenshots_box;
pub use screenshots_box::ScreenshotsBox;

mod window;
pub use window::FfApplicationWindow;
pub use window::View;
