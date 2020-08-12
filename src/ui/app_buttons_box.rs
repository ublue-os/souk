use appstream_rs::Component;
use flatpak::{InstallationExt, Remote, RemoteExt, Transaction, TransactionExt};
use gtk::prelude::*;

use std::rc::Rc;

use crate::flatpak_backend::FlatpakBackend;

pub struct AppButtonsBox {
    pub widget: gtk::Box,
    flatpak_backend: Rc<FlatpakBackend>,

    remote: Option<Remote>,
    component: Option<Component>,
    transaction: Option<Transaction>,

    builder: gtk::Builder,
}

impl AppButtonsBox {
    pub fn new(flatpak_backend: Rc<FlatpakBackend>) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/app_buttons_box.ui");
        get_widget!(builder, gtk::Box, app_buttons_box);

        let remote = None;
        let component = None;
        let transaction = None;

        let app_buttons_box = Self {
            widget: app_buttons_box,
            flatpak_backend,
            remote,
            component,
            transaction,
            builder,
        };

        app_buttons_box.setup_signals();
        app_buttons_box
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Button, install_button);
        install_button.connect_clicked(clone!(@strong self.remote as remote, @strong self.transaction as transaction => move |_|{


        }));
    }

    pub fn set_app(&mut self, component: Component, remote: Remote) {
        get_widget!(self.builder, gtk::Stack, button_stack);
        /*

        //match self.appstream_cache.is_installed(component.clone(), Some(&remote)){
        //    true => {
        //        button_stack.set_visible_child_name("installed");
        //    },
        //    false => {
        button_stack.set_visible_child_name("install");
        //    }
        //};

        self.remote = Some(remote);
        //self.component = Some(component);
        self.transaction = Some(self.appstream_cache.create_transaction());

        let transaction = self.transaction.as_ref().unwrap();

        match &component.bundle[0] {
            Bundle::Flatpak { runtime, sdk, name } => {
                //self.appstream_cache.system_install.install(&self.remote.as_ref().unwrap().get_name().unwrap(), flatpak::RefKind::App, "de.haeckerfelix.Shortwave",
                //Some("x86_64"), Some("stable"), None, Some(&gio::Cancellable::new())).unwrap();

                //warn!("Name {}", name);
                //transaction.add_install(&self.remote.as_ref().unwrap().get_name().unwrap(), &name, &[]).unwrap();
                //transaction.run(Some(&gio::Cancellable::new())).unwrap();
            }
            _ => (),
        };*/
    }
}
