table! {
    appstream_packages (id) {
        id -> Nullable<Integer>,

        app_id -> Text,
        branch -> Text,
        remote -> Text,

        name -> Text,
        version -> Text,
        summary -> Text,
        categories -> Text,
        developer_name -> Text,
        project_group -> Text,
        release_date -> Nullable<Date>,

        component -> Text,
    }
}

table! {
    info (id){
        id -> Nullable<Integer>,
        db_version -> Text,
        db_timestamp -> Text,
    }
}
