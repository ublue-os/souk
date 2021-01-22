use appstream::enums::ImageKind;
use futures_util::future::FutureExt;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use isahc::config::RedirectPolicy;
use isahc::prelude::*;

use crate::backend::SoukPackage;
use crate::error::Error;
use crate::ui::package_widgets::PackageWidget;
use crate::ui::utils;

pub struct ScreenshotsBox {
    pub widget: gtk::Box,
    builder: gtk::Builder,
}

impl ScreenshotsBox {
    async fn download_image(url: url::Url, size: i32) -> Result<Pixbuf, Error> {
        let mut response = Request::get(url.to_string())
            .redirect_policy(RedirectPolicy::Follow)
            .body(())
            .unwrap()
            .send_async()
            .await?;
        let mut body = response.body_mut();
        let mut bytes = vec![];
        async_std::io::copy(&mut body, &mut bytes).await.unwrap();

        let input_stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&bytes));
        Ok(Pixbuf::from_stream_at_scale_async_future(&input_stream, 1000, size, true).await?)
    }
}

impl PackageWidget for ScreenshotsBox {
    fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/screenshots_box.ui");
        get_widget!(builder, gtk::Box, screenshots_box);

        Self {
            widget: screenshots_box,
            builder,
        }
    }

    fn set_package(&self, package: &SoukPackage) {
        let screenshots = package
            .get_appdata()
            .expect("No appdata available")
            .screenshots;

        get_widget!(self.builder, gtk::Box, screenshots_box);
        get_widget!(self.builder, libadwaita::Carousel, carousel);
        utils::clear_carousel(&carousel);
        screenshots_box.set_visible(false);

        for screenshot in &screenshots {
            for image in &screenshot.images {
                if image.kind == ImageKind::Thumbnail {
                    continue;
                }

                let c = carousel.clone();
                let ssb = screenshots_box.clone();
                let fut =
                    Self::download_image(image.url.clone(), 350).map(move |result| match result {
                        Ok(pixbuf) => {
                            let picture = gtk::Picture::new_for_pixbuf(Some(&pixbuf));
                            picture.set_can_shrink(true);
                            c.append(&picture);
                            ssb.show();
                        }
                        Err(err) => warn!("Unable to download thumbnail: {}", err),
                    });
                spawn!(fut);
            }
        }
    }

    fn reset(&self) {
        get_widget!(self.builder, gtk::Box, screenshots_box);
        get_widget!(self.builder, libadwaita::Carousel, carousel);

        screenshots_box.set_visible(false);
        utils::clear_carousel(&carousel);
    }
}
