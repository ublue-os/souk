use appstream::TranslatableString;
use chrono::NaiveDate;

use super::schema::*;
use crate::backend::{Package, RemotePackage};

#[derive(Queryable, Insertable, Debug, Clone)]
#[table_name = "appstream_packages"]
pub struct DbPackage {
    pub id: Option<i32>,

    pub kind: String,
    pub name: String,
    pub arch: String,
    pub branch: String,
    pub commit: String,
    pub remote: String,

    pub download_size: i64,
    pub installed_size: i64,

    pub display_name: String,
    pub version: String,
    pub summary: String,
    pub categories: String,
    pub developer_name: String,
    pub project_group: String,
    pub release_date: Option<NaiveDate>,

    pub appdata: String,
}

impl DbPackage {
    fn get_string(string: &Option<TranslatableString>) -> String {
        match string {
            Some(value) => value.get_default().unwrap_or(&"".to_string()).to_string(),
            None => "".to_string(),
        }
    }
}

impl From<RemotePackage> for DbPackage {
    fn from(package: RemotePackage) -> Self {
        let mut display_name = "".to_string();
        let mut version = "".to_string();
        let mut summary = "".to_string();
        let mut categories = "".to_string();
        let mut developer_name = "".to_string();
        let mut project_group = "".to_string();
        let mut release_date = None;
        let mut appdata = "".to_string();

        if let Some(ad) = package.appdata() {
            display_name = Self::get_string(&Some(ad.name.clone()));
            summary = Self::get_string(&ad.summary);
            developer_name = Self::get_string(&ad.developer_name);
            project_group = ad.project_group.clone().unwrap_or("".to_string());

            if let Some(release) = ad.releases.clone().pop() {
                version = release.version;
            }

            if let Some(release) = ad.releases.clone().pop() {
                if let Some(date) = release.date {
                    release_date = Some(date.naive_utc().date());
                }
            }

            for category in &ad.categories {
                categories = categories + &format!("{};", category.to_string());
            }

            appdata = serde_json::to_string(&ad).unwrap().to_string();
        }

        DbPackage {
            id: None,

            kind: package.kind().clone().to_string(),
            name: package.name().clone(),
            arch: package.arch().clone(),
            branch: package.branch().clone(),
            commit: package.commit().clone(),
            remote: package.remote().clone(),

            download_size: package.download_size().clone(),
            installed_size: package.installed_size().clone(),

            display_name,
            version,
            summary,
            categories,
            developer_name,
            project_group,
            release_date,

            appdata,
        }
    }
}

impl PartialEq for DbPackage {
    fn eq(&self, other: &Self) -> bool {
        self.appdata == other.appdata && self.commit == other.commit
    }
}

#[derive(Queryable, Insertable, Default, Debug, Clone)]
#[table_name = "info"]
pub struct DbInfo {
    pub id: Option<i32>,
    pub db_version: String,
    pub db_timestamp: String,
}
