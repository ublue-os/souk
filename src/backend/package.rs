use appstream::Component;
use appstream::Release;

use std::fmt;

use crate::backend::utils;

#[derive(Clone, PartialEq)]
pub struct Package {
    pub is_app: bool,

    pub app_id: String,
    pub arch: String,
    pub branch: String,

    pub remote: String,
    pub component: Component,
}

impl Package {
    pub fn new(component: Component, remote: String) -> Self {
        let flatpak_ref = utils::get_flatpak_ref(&component);
        let values: Vec<&str> = flatpak_ref.split("/").collect();

        let is_app = match values[0] {
            "app" => true,
            "runtime" => false,
            _ => panic!("Invalid flatpak package type"),
        };
        let app_id = values[1].to_string();
        let arch = values[2].to_string();
        let branch = values[3].to_string();

        Self {
            is_app,
            app_id,
            arch,
            branch,
            remote,
            component,
        }
    }

    pub fn get_ref_name(&self) -> String {
        let flatpak_type = if self.is_app {
            "app".to_string()
        } else {
            "runtime".to_string()
        };

        format!(
            "{}/{}/{}/{}",
            &flatpak_type, &self.app_id, &self.arch, &self.branch
        )
    }

    pub fn get_newest_release(&self) -> Option<Release> {
        let mut newest = None;
        let mut newest_date = None;

        for release in &self.component.releases {
            if newest.is_none() {
                newest = Some(release.clone());
                newest_date = Some(release.date.clone());
            } else {
                if release.date > *newest_date.as_ref().unwrap() {
                    newest = Some(release.clone());
                    newest_date = Some(release.date.clone());
                }
            }
        }

        newest
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
