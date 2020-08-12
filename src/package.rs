use appstream_rs::{Bundle, Component};
use flatpak::{Remote, RemoteExt};

use std::rc::Rc;
use std::fmt;

#[derive(Clone, PartialEq)]
pub struct Package {
    pub component: Component,
    remote: Remote,
}

impl Package{
    pub fn new(component: Component, remote: Remote) -> Self {
        Self {
            component,
            remote,
        }
    }

    pub fn get_app_id(&self) -> String {
        self.component.id.0.clone()
    }

    pub fn get_full_ref_name(&self) -> String {
        match &self.component.bundle[0]{
            Bundle::Flatpak{id, runtime, sdk} => {
                return id.clone();
            },
            _ => return "".to_string(),
        };
    }

    pub fn get_origin(&self) -> String {
        self.remote.get_name().unwrap().to_string()
    }
}

impl fmt::Debug for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Package")
         .field("id", &self.get_app_id())
         .field("origin", &self.get_origin())
         .finish()
    }
}

