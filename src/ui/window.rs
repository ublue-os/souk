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
use gtk::{gio, glib, CompositeTemplate};

use crate::app::SkApplication;
use crate::config;
use crate::flatpak::transaction::SkTransaction;
use crate::ui::transaction::SkTransactionRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/window.ui")]
    pub struct SkApplicationWindow {
        #[template_child]
        pub status_page: TemplateChild<adw::StatusPage>,
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

    impl ObjectImpl for SkApplicationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_widgets();
            obj.setup_signals();
            obj.setup_gactions();
        }
    }

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
        glib::Object::new::<Self>(&[]).unwrap()
    }

    fn setup_widgets(&self) {
        let imp = self.imp();
        let app = SkApplication::default();

        // Add devel style class for development or beta builds
        if config::PROFILE == "development" || config::PROFILE == "beta" {
            self.add_css_class("devel");
        }

        // Active transactions listbox
        let model = app.worker().transactions();
        imp.transactions_listbox
            .bind_model(Some(&model), |transaction| {
                let transaction: SkTransaction = transaction.clone().downcast().unwrap();
                SkTransactionRow::new(&transaction).upcast()
            });
        model.connect_items_changed(clone!(@weak self as this => move |model, _, _, _|{
            let imp = this.imp();
            imp.transactions_listbox.set_visible(model.n_items() != 0);
        }));

        // DND support for sideloading
        let drop_target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
        drop_target.connect_drop(move |_, data, _, _| {
            if let Ok(filelist) = data.get::<gdk::FileList>() {
                let app = SkApplication::default();
                app.open(&filelist.files(), "");
                return true;
            }
            false
        });
        imp.status_page.add_controller(&drop_target);
    }

    fn setup_signals(&self) {}

    fn setup_gactions(&self) {}
}

impl Default for SkApplicationWindow {
    fn default() -> Self {
        Self::new()
    }
}
