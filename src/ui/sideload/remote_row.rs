// Souk - remote_row.rs
// Copyright (C) 2022  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::gio::File;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use once_cell::sync::Lazy;

use crate::flatpak::installation::SkRemote;
use crate::i18n::i18n;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/remote_row.ui")]
    pub struct SkRemoteRow {
        #[template_child]
        pub icon_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub external_link_image: TemplateChild<gtk::Image>,

        pub remote: RefCell<Option<SkRemote>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkRemoteRow {
        const NAME: &'static str = "SkRemoteRow";
        type ParentType = adw::ActionRow;
        type Type = super::SkRemoteRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkRemoteRow {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "remote",
                    "",
                    "",
                    SkRemote::static_type(),
                    ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "remote" => obj.remote().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "remote" => obj.set_remote(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SkRemoteRow {}

    impl ListBoxRowImpl for SkRemoteRow {}

    impl PreferencesRowImpl for SkRemoteRow {}

    impl ActionRowImpl for SkRemoteRow {}
}

glib::wrapper! {
    pub struct SkRemoteRow(
        ObjectSubclass<imp::SkRemoteRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;

}

impl SkRemoteRow {
    pub fn remote(&self) -> Option<SkRemote> {
        self.imp().remote.borrow().clone()
    }

    pub fn set_remote(&self, remote: &SkRemote) {
        *self.imp().remote.borrow_mut() = Some(remote.clone());

        self.update_widgets();
        self.notify("remote");
    }

    fn update_widgets(&self) {
        let imp = self.imp();
        let remote = self.remote().unwrap();

        // Icon
        if let Some(value) = remote.icon() {
            let file = File::for_uri(&value);
            if let Ok(texture) = gtk::gdk::Texture::from_file(&file) {
                imp.icon_image.set_paintable(Some(&texture));
            }
        } else {
            imp.icon_image
                .set_icon_name(Some("package-x-generic-symbolic"));
        }

        // Title
        if let Some(value) = remote.title() {
            self.set_title(&value);
        } else {
            self.set_title(&i18n("Unknown SkRemote Name"));
        }

        // Subtitle
        let mut subtitle = if let Some(value) = remote.description() {
            value.to_string()
        } else {
            "".to_string()
        };
        if let Some(value) = remote.comment() {
            if subtitle != value && !value.is_empty() {
                subtitle = format!("{} {}", subtitle, value);
            }
        }
        if subtitle.is_empty() {
            subtitle = format!("<small>{}</small>", remote.repository_url());
        } else {
            subtitle = format!("{}\n\n<small>{}</small>", subtitle, remote.repository_url());
        }
        self.set_subtitle(&subtitle);

        // Homepage
        imp.external_link_image
            .set_visible(remote.homepage().is_some());

        self.connect_activated(clone!(@weak remote => move|s|{
            if let Some(homepage) = remote.homepage(){
                let window: gtk::Window = s.root().unwrap().downcast().unwrap();
                gtk::show_uri(Some(&window), &homepage, gtk::gdk::CURRENT_TIME);
            }
        }));
    }
}
