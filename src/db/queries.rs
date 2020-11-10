use crate::db;
use crate::db::*;
use crate::diesel::prelude::*;
use diesel::dsl::sql;

use crate::backend::SoukPackage;

#[allow(dead_code)]
pub enum DisplayLevel {
    Apps,       /* Apps (de.haeckerfelix.Souk) */
    Runtimes,   /* Apps + Runtimes (org.gnome.Sdk) */
    Extensions, /* Apps + Runtimes + Extensions (*.Debug, *.Sources, *.Docs, ...) */
}

impl DisplayLevel {
    pub fn to_sql_literal(&self) -> String {
        match &self {
            DisplayLevel::Apps => "kind = 'app'".to_string(),
            DisplayLevel::Runtimes => "kind = 'app' OR kind = 'runtime'".to_string(),
            DisplayLevel::Extensions => {
                "kind = 'app' OR kind = 'runtime' OR kind = 'extension'".to_string()
            }
        }
    }
}

macro_rules! connect_db {
    () => {
        db::connection::get_connection().get().unwrap();
    };
}

pub fn get_package(
    pkg_app_id: String,
    pkg_branch: String,
    pkg_remote: String,
) -> Result<Option<SoukPackage>, diesel::result::Error> {
    use crate::db::schema::appstream_packages::dsl::*;
    let con = connect_db!();

    let mut packages = appstream_packages
        .filter(name.eq(pkg_app_id))
        .filter(branch.eq(pkg_branch))
        .filter(remote.eq(pkg_remote))
        .load::<DbPackage>(&con)?;

    Ok(packages.pop().map(|p| p.into()))
}

pub fn get_packages_by_name(
    pkg_name: String,
    limit: i64,
    level: DisplayLevel,
) -> Result<Vec<SoukPackage>, diesel::result::Error> {
    use crate::db::schema::appstream_packages::dsl::*;
    let con = connect_db!();

    let db_packages = appstream_packages
        .filter(name.like(format!("%{}%", &pkg_name)))
        .filter(sql(&level.to_sql_literal()))
        .limit(limit)
        .load::<DbPackage>(&con)?;

    let mut packages = Vec::new();
    for db_package in db_packages {
        let package = db_package.into();
        packages.push(package);
    }

    Ok(packages)
}

pub fn get_recently_updated_packages(
    limit: i64,
    level: DisplayLevel,
) -> Result<Vec<SoukPackage>, diesel::result::Error> {
    use crate::db::schema::appstream_packages::dsl::*;
    let con = connect_db!();

    let db_packages = appstream_packages
        .order(release_date.desc())
        .filter(sql(&level.to_sql_literal()))
        .limit(limit)
        .load::<DbPackage>(&con)?;

    let mut packages = Vec::new();
    for db_package in db_packages {
        let package = db_package.into();
        packages.push(package);
    }

    Ok(packages)
}

pub fn get_packages_by_developer_name(
    pkg_developer_name: &str,
    limit: i64,
    level: DisplayLevel,
) -> Result<Vec<SoukPackage>, diesel::result::Error> {
    use crate::db::schema::appstream_packages::dsl::*;
    let con = connect_db!();

    let db_packages = appstream_packages
        .filter(developer_name.eq(pkg_developer_name))
        .filter(sql(&level.to_sql_literal()))
        .limit(limit)
        .load::<DbPackage>(&con)?;

    let mut packages = Vec::new();
    for db_package in db_packages {
        let package = db_package.into();
        packages.push(package);
    }

    Ok(packages)
}

pub fn get_db_info() -> Result<Vec<DbInfo>, diesel::result::Error> {
    use crate::db::schema::info::dsl::*;
    let con = connect_db!();

    info.load::<DbInfo>(&con).map_err(From::from)
}

pub fn insert_db_packages(db_packages: Vec<DbPackage>) -> Result<(), diesel::result::Error> {
    use crate::db::schema::appstream_packages::dsl::*;
    let con = connect_db!();

    diesel::insert_into(appstream_packages)
        .values(db_packages)
        .execute(&*con)
        .map_err(From::from)
        .map(|_| ())
}

pub fn insert_db_info(db_info: DbInfo) -> Result<(), diesel::result::Error> {
    use crate::db::schema::info::dsl::*;
    let con = connect_db!();

    diesel::insert_into(info)
        .values(db_info)
        .execute(&*con)
        .map_err(From::from)
        .map(|_| ())
}

pub fn reset() -> Result<(), diesel::result::Error> {
    debug!("Reset database...");

    use crate::db::schema::appstream_packages::dsl::*;
    use crate::db::schema::info::dsl::*;
    let con = connect_db!();

    diesel::delete(info).execute(&*con)?;
    diesel::delete(appstream_packages).execute(&*con)?;
    Ok(())
}
