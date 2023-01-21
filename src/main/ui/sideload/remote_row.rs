// Souk - remote_row.rs
// Copyright (C) 2022-2023  Felix Häcker <haeckerfelix@gnome.org>
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::gio::File;
use gtk::{glib, CompositeTemplate};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::flatpak::installation::SkRemote;
use crate::main::i18n::i18n;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/remote_row.ui")]
    pub struct SkRemoteRow {
        #[template_child]
        pub icon_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub external_link_image: TemplateChild<gtk::Image>,

        pub remote: OnceCell<SkRemote>,
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
                    ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "remote" => self.obj().remote().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "remote" => self.remote.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            let remote = self.obj().remote();

            // Icon
            if !remote.icon().is_empty() {
                let file = File::for_uri(&remote.icon());
                if let Ok(texture) = gtk::gdk::Texture::from_file(&file) {
                    self.icon_image.set_paintable(Some(&texture));
                }
            } else {
                let icon_name = "repo-symbolic";
                self.icon_image.set_icon_name(Some(icon_name));
            }

            // Title
            if !remote.title().is_empty() {
                self.obj().set_title(&remote.title());
            } else {
                self.obj().set_title(&i18n("Unknown Repository"));
            }

            // Subtitle
            let mut subtitle = remote.description();
            if subtitle != remote.comment() && !remote.comment().is_empty() {
                subtitle = format!("{} {}", subtitle, remote.comment());
            }
            if subtitle.is_empty() {
                subtitle = format!("<small>{}</small>", remote.repository_url());
            } else {
                subtitle = format!(
                    "{}\n<small><span baseline_shift=\"-16pt\">{}</span></small>",
                    subtitle,
                    remote.repository_url()
                );
            }
            self.obj().set_subtitle(&subtitle);

            // Homepage
            let has_homepage = !remote.homepage().is_empty();
            self.external_link_image.set_visible(has_homepage);
            self.obj().set_activatable(has_homepage);

            self.obj().connect_activated(clone!(@weak remote => move|s|{
                let window: gtk::Window = s.root().unwrap().downcast().unwrap();
                gtk::show_uri(Some(&window), &remote.homepage(), gtk::gdk::CURRENT_TIME);
            }));
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
    pub fn new(remote: &SkRemote) -> Self {
        glib::Object::new(&[("remote", &remote)])
    }

    pub fn remote(&self) -> SkRemote {
        self.imp().remote.get().unwrap().clone()
    }
}
