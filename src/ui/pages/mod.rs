use glib::StaticType;

mod explore_page;
pub use explore_page::SoukExplorePage;

mod installed_page;
pub use installed_page::SoukInstalledPage;

mod loading_page;
pub use loading_page::LoadingPage;

mod package_details_page;
pub use package_details_page::PackageDetailsPage;

mod search_page;
pub use search_page::SoukSearchPage;

//TODO: Wouldn't it make sense to add a trait for pages?

pub fn init() {
    SoukExplorePage::static_type();
    SoukInstalledPage::static_type();
    SoukSearchPage::static_type();
}
