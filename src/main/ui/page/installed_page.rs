// Souk - installed_page.rs
// Copyright (C) 2023-2024  Felix Häcker <haeckerfelix@gnome.org>
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

use std::cell::Cell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass, ParamSpec, Properties};
use gtk::{glib, CompositeTemplate};

use crate::main::flatpak::package::SkPackage;
use crate::main::ui::installation::SkInstallationListBox;
use crate::main::SkApplication;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::SkInstalledPage)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/page/installed_page.ui")]
    pub struct SkInstalledPage {
        // TODO: Implement, or remove
        #[property(get, set)]
        show_without_appstream: Cell<bool>,

        #[template_child]
        installation_listbox: TemplateChild<SkInstallationListBox>,
        #[template_child]
        listbox: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstalledPage {
        const NAME: &'static str = "SkInstalledPage";
        type ParentType = adw::Bin;
        type Type = super::SkInstalledPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkInstalledPage {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            let worker = SkApplication::default().worker();

            // Different installation got selected
            self.installation_listbox.connect_local(
                "notify::selected-installation",
                false,
                clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    None,
                    move |_| {
                        this.update_selected_installation();
                        None
                    }
                ),
            );

            // Preselect preferred installation
            let inst = worker.installations().preferred();
            self.installation_listbox.set_selected_installation(&inst);
        }
    }

    impl WidgetImpl for SkInstalledPage {}

    impl BinImpl for SkInstalledPage {}

    impl SkInstalledPage {
        fn update_selected_installation(&self) {
            self.listbox.unbind_model();

            let inst = self.installation_listbox.selected_installation().unwrap();
            self.listbox.bind_model(Some(&inst.packages()), |package| {
                let package: &SkPackage = package.downcast_ref().unwrap();

                let uninstall_button = gtk::Button::from_icon_name("user-trash-symbolic");
                uninstall_button.connect_clicked(clone!(
                    #[weak]
                    package,
                    move |btn| {
                        btn.set_sensitive(false);

                        let worker = SkApplication::default().worker();
                        let fut = async move {
                            let _ = worker.uninstall_flatpak(&package, false).await;
                        };
                        crate::main::spawn_future_local(fut);
                    }
                ));

                let row = adw::ActionRow::builder()
                    .title(package.name())
                    .subtitle(package.remote().name())
                    .build();

                row.add_suffix(&uninstall_button);
                row.into()
            });
        }
    }
}

glib::wrapper! {
    pub struct SkInstalledPage(
        ObjectSubclass<imp::SkInstalledPage>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for SkInstalledPage {
    fn default() -> Self {
        glib::Object::new()
    }
}
