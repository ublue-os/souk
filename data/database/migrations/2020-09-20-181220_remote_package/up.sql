DROP TABLE IF EXISTS info;
DROP TABLE IF EXISTS appstream_packages;

CREATE TABLE info (
	id 		integer NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
	db_version	text NOT NULL,
	db_timestamp	text NOT NULL
);

CREATE TABLE appstream_packages (
	id 		integer NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,

	kind		text NOT NULL,
	name		text NOT NULL,
	arch		text NOT NULL,
	branch		text NOT NULL,
	'commit'	text NOT NULL,
	remote		text NOT NULL,

	download_size	bigint NOT NULL,
	installed_size	bigint NOT NULL,

	display_name	text NOT NULL,
	version		text NOT NULL,
	summary		text NOT NULL,
	categories	text NOT NULL,
	developer_name	text NOT NULL,
	project_group	text NOT NULL,
	release_date	date,

	appdata		text NOT NULL,

	unique (name, branch, remote, `commit`)
);
