use appstream::Component;
use dyn_clone::DynClone;
use flatpak::prelude::*;
use flatpak::RemoteRef;
use glib::KeyFile;

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
