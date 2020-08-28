use appstream_rs::{Bundle, Component};
use flatpak::{Remote, RemoteExt, Ref};

use std::rc::Rc;
use std::fmt;

use crate::utils;

#[derive(Clone, PartialEq)]
pub struct Package {
    pub is_app: bool,

    pub app_id: String,
    pub arch: String,
    pub branch: String,

    pub remote: String,
    pub component: Component,
}

impl Package{
    pub fn new(component: Component, flatpak_remote: Remote) -> Self {
        let flatpak_ref = utils::get_flatpak_ref(&component);
        let values: Vec<&str> = flatpak_ref.split("/").collect();

        let is_app = match values[0]{
            "app" => true,
            "runtime" => false,
            _ => panic!("Invalid flatpak package type"),
        };
        let app_id = values[1].to_string();
        let arch = values[2].to_string();
        let branch = values[3].to_string();
        let remote = flatpak_remote.get_name().unwrap().to_string();

        Self {
            is_app,
            app_id,
            arch,
            branch,
            remote,
            component,
        }
    }

    pub fn get_ref_name (&self) -> String {
        let flatpak_type = if self.is_app {
            "app".to_string()
        }else{
            "runtime".to_string()
        };

        format!("{}/{}/{}/{}", &flatpak_type, &self.app_id, &self.arch, &self.branch)
    }
}

impl fmt::Debug for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Package")
         .field("ref", &self.get_ref_name())
         .field("remote", &self.remote)
         .finish()
    }
}

