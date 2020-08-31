CREATE TABLE info (
	id 		integer NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
	db_version	text NOT NULL,
	db_timestamp	text NOT NULL
);

CREATE TABLE appstream_packages (
	id 		integer NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,

	app_id		text NOT NULL,
	branch		text NOT NULL,
	remote		text NOT NULL,

	name		text NOT NULL,
	version		text NOT NULL,
	summary		text NOT NULL,
	categories	text NOT NULL,
	developer_name	text NOT NULL,
	project_group	text NOT NULL,
	release_date	date,

	component	text NOT NULL,

	unique (app_id, branch, remote)
);
