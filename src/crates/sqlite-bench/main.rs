//！ benchmarks

// SELECT * FROM sqlite_master WHERE type = 'index';

fn bench_spawn_blocking_vs_mono() {}

fn bench_create_index_vs_unique() {
    fn inner(is_unique: bool) {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        if is_unique {
            db.execute(
                "CREATE TABLE IF NOT EXISTS dav_users (uid BLOB PRIMARY KEY, auth BLOB UNIQUE)",
                (),
            )
            .unwrap();
        } else {
            db.execute(
                "CREATE TABLE IF NOT EXISTS dav_users (uid BLOB PRIMARY KEY, auth BLOB)",
                (),
            )
            .unwrap();
            db.execute(
                "CREATE INDEX IF NOT EXISTS dav_users_index on dav_users (auth)",
                (),
            )
            .unwrap();
        }
        for i in 0..400000 {
            let uid = i.to_string();
            let auth = uid.to_string() + "_auth";

            let mut stmd = db
                .prepare_cached("REPLACE INTO dav_users VALUES (?, ?)")
                .unwrap();
            stmd.execute((&uid, &auth)).unwrap();

            let mut stmd = db
                .prepare_cached("SELECT uid FROM dav_users WHERE auth = ?")
                .unwrap();
            let mut rows = stmd.query((&auth,)).unwrap();
            let row = rows.next().unwrap().unwrap();
            let uid_queried: String = row.get(0).unwrap();
            assert_eq!(uid, uid_queried);
        }
    }

    let begin = std::time::Instant::now();
    inner(true);
    println!(
        "> bench_create_index_vs_unique:unique = {}",
        begin.elapsed().as_millis()
    );

    let begin = std::time::Instant::now();
    inner(false);
    println!(
        "> bench_create_index_vs_unique:create_index = {}",
        begin.elapsed().as_millis()
    );
}

fn main() {
    bench_create_index_vs_unique();
    bench_create_index_vs_unique();
    bench_create_index_vs_unique();
}

// cargo run --release
