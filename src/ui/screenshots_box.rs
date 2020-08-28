use appstream_rs::{Image, Screenshot};
use futures_util::future::FutureExt;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use isahc::config::RedirectPolicy;
use isahc::prelude::*;

use crate::error::Error;
use crate::ui::utils;

pub struct ScreenshotsBox {
    pub widget: gtk::Box,
    screenshots: Vec<Screenshot>,
    builder: gtk::Builder,
}

impl ScreenshotsBox {
    pub fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/screenshots_box.ui");
        get_widget!(builder, gtk::Box, screenshots_box);

        let screenshots = Vec::new();

        let screenshots_box = Self {
            widget: screenshots_box,
            screenshots,
            builder,
        };

        screenshots_box.setup_signals();
        screenshots_box
    }

    fn setup_signals(&self) {}

    pub fn set_screenshots(&mut self, screenshots: Vec<Screenshot>) {
        if self.screenshots == screenshots {
            return;
        }
        self.screenshots = screenshots.clone();

        get_widget!(self.builder, gtk::Box, screenshots_box);
        get_widget!(self.builder, libhandy::Carousel, carousel);
        utils::remove_all_items(&carousel);
        screenshots_box.set_visible(false);

        for screenshot in screenshots {
            for image in screenshot.images {
                match image {
                    Image::Source { url, width: _, height: _ } => {
                        let c = carousel.clone();
                        let ssb = screenshots_box.clone();
                        let fut = Self::download_image(url, 350).map(move |result| match result {
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
                    _ => (),
                }
            }
        }
    }

    async fn download_image(url: url::Url, size: i32) -> Result<Pixbuf, Error> {
        let mut response = Request::get(url.to_string()).redirect_policy(RedirectPolicy::Follow).body(()).unwrap().send_async().await?;
        let mut body = response.body_mut();
        let mut bytes = vec![];
        async_std::io::copy(&mut body, &mut bytes).await.unwrap();

        let input_stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&bytes));
        Ok(Pixbuf::from_stream_at_scale_async_future(&input_stream, 1000, size, true).await?)
    }
}
