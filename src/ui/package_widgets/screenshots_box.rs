use appstream::enums::ImageKind;
use futures_util::future::FutureExt;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use isahc::config::RedirectPolicy;
use isahc::prelude::*;

use crate::backend::Package;
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
        let builder = gtk::Builder::from_resource("/org/gnome/Store/gtk/screenshots_box.ui");
        get_widget!(builder, gtk::Box, screenshots_box);

        Self {
            widget: screenshots_box,
            builder,
        }
    }

    fn set_package(&self, package: Package) {
        get_widget!(self.builder, gtk::Box, screenshots_box);
        get_widget!(self.builder, libhandy::Carousel, carousel);
        utils::remove_all_items(&carousel);
        screenshots_box.set_visible(false);

        for screenshot in &package.component.screenshots {
            for image in &screenshot.images {
                if image.kind == ImageKind::Thumbnail {
                    continue;
                }

                let c = carousel.clone();
                let ssb = screenshots_box.clone();
                let fut =
                    Self::download_image(image.url.clone(), 350).map(move |result| match result {
                        Ok(pixbuf) => {
                            let image = gtk::Image::from_pixbuf(Some(&pixbuf));
                            image.set_visible(true);
                            ssb.set_visible(true);
                            c.add(&image);
                        }
                        Err(err) => warn!("Unable to download thumbnail: {}", err),
                    });
                spawn!(fut);
            }
        }
    }

    fn reset(&self) {
        get_widget!(self.builder, gtk::Box, screenshots_box);
        get_widget!(self.builder, libhandy::Carousel, carousel);

        screenshots_box.set_visible(false);
        utils::remove_all_items(&carousel);
    }
}
