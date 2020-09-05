use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::FlatpakBackend;
use crate::backend::Package;

pub struct PackageActionButton {
    pub widget: gtk::Box,
    flatpak_backend: Rc<FlatpakBackend>,
    package: Rc<RefCell<Option<Package>>>,

    builder: gtk::Builder,
}

impl PackageActionButton {
    pub fn new(flatpak_backend: Rc<FlatpakBackend>) -> Self {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/package_action_button.ui");
        get_widget!(builder, gtk::Box, package_action_button);

        let package = Rc::new(RefCell::new(None));

        let package_action_button = Self {
            widget: package_action_button,
            flatpak_backend,
            package,
            builder,
        };

        package_action_button.setup_signals();
        package_action_button
    }

    fn setup_signals(&self) {
        // install
        get_widget!(self.builder, gtk::Button, install_button);
        install_button.connect_clicked(clone!(@strong self.flatpak_backend as flatpak_backend, @strong self.package as package => move |_|{
            flatpak_backend.clone().install_package(package.borrow().as_ref().unwrap().clone());
        }));

        // uninstall
        get_widget!(self.builder, gtk::Button, uninstall_button);
        uninstall_button.connect_clicked(clone!(@strong self.flatpak_backend as flatpak_backend, @strong self.package as package => move |_|{
            flatpak_backend.clone().uninstall_package(package.borrow().as_ref().unwrap().clone());
        }));
    }

    pub fn set_package(&mut self, package: Package) {
        get_widget!(self.builder, gtk::Stack, button_stack);

        match self.flatpak_backend.clone().is_package_installed(&package) {
            true => {
                button_stack.set_visible_child_name("installed");
            }
            false => {
                button_stack.set_visible_child_name("install");
            }
        };

        *self.package.borrow_mut() = Some(package);
    }
}
