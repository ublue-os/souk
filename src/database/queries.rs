use crate::database;
use crate::database::*;
use crate::diesel::prelude::*;

macro_rules! connect_db {
    () => {
        database::connection::get_connection().get().unwrap();
    };
}

pub fn insert_db_package(db_package: DbPackage) -> Result<(), diesel::result::Error> {
    use crate::database::schema::appstream_packages::dsl::*;
    let con = connect_db!();

    diesel::insert_into(appstream_packages).values(db_package).execute(&*con).map_err(From::from).map(|_| ())
}

pub fn insert_db_info(db_info: DbInfo) -> Result<(), diesel::result::Error> {
    use crate::database::schema::info::dsl::*;
    let con = connect_db!();

    diesel::insert_into(info).values(db_info).execute(&*con).map_err(From::from).map(|_| ())
}

pub fn get_db_info() -> Result<Vec<DbInfo>, diesel::result::Error> {
    use crate::database::schema::info::dsl::*;
    let con = connect_db!();

    info.load::<DbInfo>(&con).map_err(From::from)
}

pub fn reset() -> Result<(), diesel::result::Error> {
    use crate::database::schema::appstream_packages::dsl::*;
    use crate::database::schema::info::dsl::*;
    let con = connect_db!();

    diesel::delete(info).execute(&*con)?;
    diesel::delete(appstream_packages).execute(&*con)?;
    Ok(())
}
