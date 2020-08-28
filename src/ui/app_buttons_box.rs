use gtk::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;

use crate::backend::FlatpakBackend;
use crate::backend::Package;

pub struct AppButtonsBox {
    pub widget: gtk::Box,
    flatpak_backend: Rc<FlatpakBackend>,
    package: Rc<RefCell<Option<Package>>>,

    builder: gtk::Builder,
}

impl AppButtonsBox {
    pub fn new(flatpak_backend: Rc<FlatpakBackend>) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/app_buttons_box.ui");
        get_widget!(builder, gtk::Box, app_buttons_box);

        let package = Rc::new(RefCell::new(None));

        let app_buttons_box = Self {
            widget: app_buttons_box,
            flatpak_backend,
            package,
            builder,
        };

        app_buttons_box.setup_signals();
        app_buttons_box
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Button, install_button);
        install_button.connect_clicked(clone!(@strong self.flatpak_backend as flatpak_backend, @strong self.package as package => move |_|{
            flatpak_backend.clone().install_package(package.borrow().as_ref().unwrap().clone());
        }));
    }

    pub fn set_package(&mut self, package: Package) {
        get_widget!(self.builder, gtk::Stack, button_stack);

        match self.flatpak_backend.clone().is_package_installed(&package){
            true => {
                button_stack.set_visible_child_name("installed");
            },
            false => {
                button_stack.set_visible_child_name("install");
            }
        };

        *self.package.borrow_mut() = Some(package);
    }
}
