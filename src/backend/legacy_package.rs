use appstream::Collection;
use appstream::Component;
use dyn_clone::DynClone;
use flatpak::prelude::*;
use flatpak::{InstalledRef, RemoteRef};
use glib::KeyFile;

use std::path::PathBuf;

use crate::database::DbPackage;

#[derive(Debug, Clone, PartialEq)]
pub enum PackageKind {
    App,
    Runtime,
    Extension,
}

impl PackageKind {
    pub fn from_keyfile(keyfile: KeyFile) -> Self {
        if keyfile.has_group("ExtensionOf") {
            return PackageKind::Extension;
        }
        if keyfile.has_group("Runtime") {
            return PackageKind::Runtime;
        }
        PackageKind::App
    }
}

impl std::string::ToString for PackageKind {
    fn to_string(&self) -> String {
        match self {
            PackageKind::App => "app".to_string(),
            PackageKind::Runtime => "runtime".to_string(),
            PackageKind::Extension => "extension".to_string(),
        }
    }
}

pub trait Package: DynClone {
    fn base_package(&self) -> &BasePackage;

    fn kind(&self) -> PackageKind {
        self.base_package().kind.clone()
    }

    fn name(&self) -> String {
        self.base_package().name.clone()
    }

    fn arch(&self) -> String {
        self.base_package().arch.clone()
    }

    fn branch(&self) -> String {
        self.base_package().branch.clone()
    }

    fn commit(&self) -> String {
        self.base_package().commit.clone()
    }

    fn remote(&self) -> String {
        self.base_package().remote.clone()
    }

    fn appdata(&self) -> Option<Component> {
        self.base_package().appdata.clone()
    }

    fn ref_name(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            &self.kind().to_string(),
            &self.name(),
            &self.arch(),
            &self.branch()
        )
    }
}

impl std::fmt::Debug for Box<dyn Package> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Package")
            .field("ref", &self.ref_name())
            .field("remote", &self.remote())
            .finish()
    }
}

dyn_clone::clone_trait_object!(Package);

//
// BasePackage
//
#[derive(Clone, PartialEq)]
pub struct BasePackage {
    kind: PackageKind,
    name: String,
    arch: String,
    branch: String,
    commit: String,
    remote: String,
    appdata: Option<Component>,
}

impl Package for BasePackage {
    fn base_package(&self) -> &BasePackage {
        &self
    }
}

impl From<DbPackage> for BasePackage {
    fn from(db_package: DbPackage) -> Self {
        let kind = match db_package.kind.as_ref() {
            "app" => PackageKind::App,
            "runtime" => PackageKind::Runtime,
            _ => PackageKind::Extension,
        };

        let appdata = serde_json::from_str(&db_package.appdata).ok();

        Self {
            kind,
            name: db_package.name,
            arch: db_package.arch,
            branch: db_package.branch,
            commit: db_package.commit,
            remote: db_package.remote,
            appdata,
        }
    }
}

impl std::fmt::Debug for BasePackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasePackage")
            .field("ref", &self.ref_name())
            .field("remote", &self.remote())
            .finish()
    }
}

//
// InstalledPackage
//
#[derive(Debug, Clone, PartialEq)]
pub struct InstalledPackage {
    base_package: BasePackage,

    commit: String,
    is_current: bool,
    installed_size: i64,
}

impl InstalledPackage {
    pub fn commit(&self) -> String {
        self.commit.clone()
    }

    pub fn is_current(&self) -> bool {
        self.is_current
    }

    pub fn installed_size(&self) -> i64 {
        self.installed_size
    }
}

impl Package for InstalledPackage {
    fn base_package(&self) -> &BasePackage {
        &self.base_package
    }
}

impl From<InstalledRef> for InstalledPackage {
    fn from(installed_ref: InstalledRef) -> Self {
        let keyfile_bytes = installed_ref
            .load_metadata(Some(&gio::Cancellable::new()))
            .unwrap();
        let keyfile = glib::KeyFile::new();
        keyfile
            .load_from_bytes(&keyfile_bytes, glib::KeyFileFlags::NONE)
            .unwrap();

        // Load appdata
        let mut path = PathBuf::new();
        let appstream_dir = installed_ref.get_deploy_dir().unwrap().to_string();
        path.push(appstream_dir);
        path.push(&format!(
            "files/share/app-info/xmls/{}.xml.gz",
            installed_ref.get_name().unwrap().to_string()
        ));

        // Parse appstream data
        let appdata = Collection::from_gzipped(path.clone())
            .map(|appdata| appdata.components[0].clone())
            .ok();

        let base_package = BasePackage {
            kind: PackageKind::from_keyfile(keyfile),
            name: installed_ref.get_name().unwrap().to_string(),
            arch: installed_ref.get_arch().unwrap().to_string(),
            branch: installed_ref.get_branch().unwrap().to_string(),
            commit: installed_ref.get_commit().unwrap().to_string(),
            remote: installed_ref.get_origin().unwrap().to_string(),
            appdata,
        };

        Self {
            base_package,
            commit: installed_ref.get_commit().unwrap().to_string(),
            is_current: installed_ref.get_is_current(),
            installed_size: installed_ref.get_installed_size() as i64,
        }
    }
}

//
// RemotePackage
//
#[derive(Debug, Clone, PartialEq)]
pub struct RemotePackage {
    base_package: BasePackage,

    download_size: i64,
    installed_size: i64,
}

impl RemotePackage {
    pub fn download_size(&self) -> i64 {
        self.download_size
    }

    pub fn installed_size(&self) -> i64 {
        self.installed_size
    }
}

impl Package for RemotePackage {
    fn base_package(&self) -> &BasePackage {
        &self.base_package
    }
}

impl From<(RemoteRef, Option<Component>)> for RemotePackage {
    fn from(remote_ref: (RemoteRef, Option<Component>)) -> Self {
        let keyfile_bytes = remote_ref.0.get_metadata().unwrap();
        let keyfile = glib::KeyFile::new();
        keyfile
            .load_from_bytes(&keyfile_bytes, glib::KeyFileFlags::NONE)
            .unwrap();

        let base_package = BasePackage {
            kind: PackageKind::from_keyfile(keyfile),
            name: remote_ref.0.get_name().unwrap().to_string(),
            arch: remote_ref.0.get_arch().unwrap().to_string(),
            branch: remote_ref.0.get_branch().unwrap().to_string(),
            commit: remote_ref.0.get_commit().unwrap().to_string(),
            remote: remote_ref.0.get_remote_name().unwrap().to_string(),
            appdata: remote_ref.1,
        };

        Self {
            base_package,
            download_size: remote_ref.0.get_download_size() as i64,
            installed_size: remote_ref.0.get_installed_size() as i64,
        }
    }
}

impl From<DbPackage> for RemotePackage {
    fn from(db_package: DbPackage) -> Self {
        let kind = match db_package.kind.as_ref() {
            "app" => PackageKind::App,
            "runtime" => PackageKind::Runtime,
            _ => PackageKind::Extension,
        };

        let appdata = serde_json::from_str(&db_package.appdata).ok();

        let base_package = BasePackage {
            kind,
            name: db_package.name,
            arch: db_package.arch,
            branch: db_package.branch,
            commit: db_package.commit,
            remote: db_package.remote,
            appdata,
        };

        Self {
            base_package,
            download_size: db_package.download_size,
            installed_size: db_package.installed_size,
        }
    }
}
