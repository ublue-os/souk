use appstream::Component;
use appstream::TranslatableString;
use chrono::NaiveDate;

use super::schema::*;
use crate::backend::Package;

#[derive(Queryable, Insertable, Debug, Clone)]
#[table_name = "appstream_packages"]
pub struct DbPackage {
    pub id: Option<i32>,

    pub app_id: String,
    pub branch: String,
    pub remote: String,

    pub name: String,
    pub version: String,
    pub summary: String,
    pub categories: String,
    pub developer_name: String,
    pub project_group: String,
    pub release_date: Option<NaiveDate>,

    pub component: String,
}

impl DbPackage {
    pub fn from_package(package: &Package) -> Self {
        let version = match package.get_newest_release() {
            Some(release) => release.version,
            None => "".to_string(),
        };

        let mut categories = "".to_string();
        for category in &package.component.categories {
            categories = categories + &format!("{};", category.to_string());
        }

        let release_date: Option<NaiveDate> = match package.get_newest_release() {
            Some(release) => match release.date {
                Some(date) => Some(date.naive_utc().date()),
                None => None,
            },
            None => None,
        };

        DbPackage {
            id: None,
            app_id: package.app_id.clone(),
            branch: package.branch.clone(),
            remote: package.remote.clone(),

            name: Self::get_string(&Some(package.component.name.clone())),
            version,
            summary: Self::get_string(&package.component.summary),
            categories,
            developer_name: Self::get_string(&package.component.developer_name),
            project_group: package
                .component
                .project_group
                .clone()
                .unwrap_or("".to_string()),
            release_date,

            component: serde_json::to_string(&package.component).unwrap(),
        }
    }

    fn get_string(string: &Option<TranslatableString>) -> String {
        match string {
            Some(value) => value.get_default().unwrap_or(&"".to_string()).to_string(),
            None => "".to_string(),
        }
    }

    pub fn to_package(&self) -> Package {
        let component: Component =
            serde_json::from_str(&self.component).expect("Unable to parse component JSON.");
        Package::new(component, self.remote.clone())
    }
}

impl PartialEq for DbPackage {
    fn eq(&self, other: &Self) -> bool {
        self.component == other.component
    }
}

#[derive(Queryable, Insertable, Default, Debug, Clone)]
#[table_name = "info"]
pub struct DbInfo {
    pub id: Option<i32>,
    pub db_version: String,
    pub db_timestamp: String,
}
