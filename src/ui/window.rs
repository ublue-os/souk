// Souk - window.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
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
use glib::{clone, subclass};
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate, FileChooserAction, FileChooserDialog, ResponseType};
use libflatpak::prelude::*;
use libflatpak::{BundleRef, Ref};

use crate::app::SkApplication;
use crate::config;
use crate::flatpak::SkTransaction;
use crate::i18n::i18n;
use crate::ui::SkTransactionRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/window.ui")]
    pub struct SkApplicationWindow {
        #[template_child]
        pub flatpak_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub remote_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub transactions_listbox: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkApplicationWindow {
        const NAME: &'static str = "SkApplicationWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::SkApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkApplicationWindow {}

    impl WidgetImpl for SkApplicationWindow {}

    impl WindowImpl for SkApplicationWindow {}

    impl ApplicationWindowImpl for SkApplicationWindow {}

    impl AdwApplicationWindowImpl for SkApplicationWindow {}
}

glib::wrapper! {
    pub struct SkApplicationWindow(
        ObjectSubclass<imp::SkApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

#[gtk::template_callbacks]
impl SkApplicationWindow {
    pub fn new() -> Self {
        let window = glib::Object::new::<Self>(&[]).unwrap();

        window.setup_widgets();
        window.setup_signals();
        window.setup_gactions();

        window
    }

    pub fn setup_widgets(&self) {
        let imp = self.imp();
        let app = SkApplication::default();

        // Add devel style class for development or beta builds
        if config::PROFILE == "development" || config::PROFILE == "beta" {
            self.add_css_class("devel");
        }

        let model = app.worker().transactions();
        imp.transactions_listbox
            .bind_model(Some(&model), |transaction| {
                let transaction: SkTransaction = transaction.clone().downcast().unwrap();
                SkTransactionRow::new(&transaction).upcast()
            });
    }

    fn setup_signals(&self) {}

    fn setup_gactions(&self) {}

    #[template_callback]
    fn install_flatpak(&self) {
        let flatpak = self.imp().flatpak_entry.text();
        let ref_ = Ref::parse(&flatpak).unwrap();
        let remote = self.imp().remote_entry.text();

        let install_fut = async move {
            SkApplication::default()
                .worker()
                .install_flatpak(&ref_, &remote, "default")
                .await
                .expect("Unable to install flatpak");
        };
        spawn!(install_fut);
    }

    #[template_callback]
    fn select_flatpak_bundle(&self) {
        let dialog = FileChooserDialog::new(
            Some(&i18n("Install Flatpak Bundle")),
            Some(self),
            FileChooserAction::Open,
            &[(&i18n("_Open"), gtk::ResponseType::Accept)],
        );

        dialog.set_modal(true);
        dialog.connect_response(
            clone!(@strong dialog, @weak self as this => move |dialog, resp| {
                if resp == ResponseType::Accept {
                    let file = dialog.file().unwrap();
                    let bundle_ref = BundleRef::new(&file).unwrap();

                    dbg!(bundle_ref.origin());

                    let install_fut = async move {
                        SkApplication::default().worker().install_flatpak_bundle(&bundle_ref, "default").await.expect("Unable to install bundle");
                    };
                    spawn!(install_fut);
                }
                dialog.hide();
                dialog.close();
            }),
        );

        dialog.show();
    }
}

impl Default for SkApplicationWindow {
    fn default() -> Self {
        SkApplication::default()
            .active_window()
            .unwrap()
            .downcast()
            .unwrap()
    }
}
