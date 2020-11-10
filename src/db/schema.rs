table! {
    appstream_packages (id) {
        id -> Nullable<Integer>,

        kind -> Text,
        name -> Text,
        arch -> Text,
        branch -> Text,
        commit -> Text,
        remote -> Text,

        download_size -> BigInt,
        installed_size -> BigInt,

        display_name -> Text,
        version -> Text,
        summary -> Text,
        categories -> Text,
        developer_name -> Text,
        project_group -> Text,
        release_date -> Nullable<Date>,

        appdata -> Text,
    }
}

table! {
    info (id){
        id -> Nullable<Integer>,
        db_version -> Text,
        db_timestamp -> Text,
    }
}
