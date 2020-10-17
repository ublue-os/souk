use appstream::enums::ImageKind;
use futures_util::future::FutureExt;
use gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use isahc::config::RedirectPolicy;
use isahc::prelude::*;
use libhandy4::CarouselExt;
use gio::prelude::*;

use crate::backend::Package;
use crate::error::Error;
use crate::ui::package_widgets::PackageWidget;
use crate::ui::utils;

pub struct ScreenshotsBox {
    pub widget: gtk4::Box,
    builder: gtk4::Builder,
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
        let builder = gtk4::Builder::from_resource("/org/gnome/Store/gtk/screenshots_box.ui");
        get_widget!(builder, gtk4::Box, screenshots_box);

        Self {
            widget: screenshots_box,
            builder,
        }
    }

    fn set_package(&self, package: &dyn Package) {
        let screenshots = package.appdata().expect("No appdata available").screenshots;

        get_widget!(self.builder, gtk4::Box, screenshots_box);
        get_widget!(self.builder, libhandy4::Carousel, carousel);
        utils::remove_all_items(&carousel, |widget|{
            carousel.remove(&widget);
        });
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
                            let image = gtk4::Image::from_pixbuf(Some(&pixbuf));
                            image.set_visible(true);
                            ssb.set_visible(true);
                            c.append(&image);
                        }
                        Err(err) => warn!("Unable to download thumbnail: {}", err),
                    });
                spawn!(fut);
            }
        }
    }

    fn reset(&self) {
        get_widget!(self.builder, gtk4::Box, screenshots_box);
        get_widget!(self.builder, libhandy4::Carousel, carousel);

        screenshots_box.set_visible(false);
        utils::remove_all_items(&carousel, |widget|{
            carousel.remove(&widget);
        });
    }
}
