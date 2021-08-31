extern crate sqlite;

use std::env;

pub fn get_dbconnection() -> sqlite::Connection {
    if let Ok(db_path) = env::var("DB_PATH") {
        sqlite::open(db_path).unwrap()
    } else {
        sqlite::open(":memory:").unwrap()
    }
}

pub fn setup_database() -> () {
    //https://docs.rs/sqlite/0.26.0/sqlite/
    let conn = get_dbconnection();
    conn
        .execute(
            "
            --drop table pipeline;
	        CREATE TABLE IF NOT EXISTS pipeline(
                -- without 'LOCALTIME' it will be in UTC however it would make it more complex when querying with golang time
                ts DATETIME DEFAULT(STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
                id integer primary key autoincrement,
                pid INT GENERATED ALWAYS AS (json_extract(data, '$.object_attributes.id')) STORED,
                project_id INT GENERATED ALWAYS AS (json_extract(data, '$.project.id')) VIRTUAL,
                created_at GENERATED ALWAYS AS (json_extract(data, '$.object_attributes.created_at')) VIRTUAL,
                data TEXT);
            --drop table job;
            CREATE TABLE IF NOT EXISTS job(
                ts DATETIME DEFAULT(STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
                id integer primary key autoincrement,
                jid INT GENERATED ALWAYS AS (json_extract(data, '$.build_id')) STORED,
                project_id GENERATED ALWAYS AS (json_extract(data, '$.project_id')) VIRTUAL,
                pid INT GENERATED ALWAYS AS (json_extract(data, '$.pipeline_id')) STORED,
                created_at GENERATED ALWAYS AS (json_extract(data, '$.build_created_at')) VIRTUAL,
                data TEXT
                );
            --drop table deployment;
            CREATE TABLE IF NOT EXISTS deployment(
                ts DATETIME DEFAULT(STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
                id integer primary key autoincrement,
                did INT GENERATED ALWAYS AS (json_extract(data, '$.deployable_id')) VIRTUAL,
                status_changed_at GENERATED ALWAYS AS (json_extract(data, '$.status_changed_at')) VIRTUAL,
                data TEXT
                );
            --drop table log;
            CREATE TABLE IF NOT EXISTS log(
            ts DATETIME DEFAULT(STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            host TEXT NOT NULL,
            application TEXT,
            message TEXT NOT NULL,
            logfile TEXT);

            create index IF NOT EXISTS dep_ts on deployment(ts);
            create index IF NOT EXISTS job_ts on job(ts);
            create index IF NOT EXISTS job_pid on job(pid);
            create index IF NOT EXISTS job_jid on job(jid);
            create index IF NOT EXISTS pipe_ts on pipeline(ts);
            create index IF NOT EXISTS pipe_pid on pipeline(pid);
            create index IF NOT EXISTS log_ts on log(ts);

            PRAGMA main.page_size = 4096;
            PRAGMA main.cache_size=10000;
            PRAGMA main.locking_mode=EXCLUSIVE;
            PRAGMA main.synchronous=NORMAL;
            PRAGMA main.journal_mode=WAL;
            PRAGMA main.cache_size=5000;
            ",
        )
        .unwrap()
}
