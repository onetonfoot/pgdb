use anyhow::{anyhow, Result};
use ctrlc;
use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_fetch::{PgFetchSettings, PostgresVersion};
use pg_embed::postgres::{PgEmbed, PgSettings};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pgdb", about = "Postgres CLI")]
struct Opt {
    /// If persistent is false clean up files and directories on drop, otherwise keep them
    #[structopt(long)]
    persistent: bool,

    /// Wether to print the access url to the stdout
    #[structopt(long)]
    print_url: bool,

    /// User
    #[structopt(long, default_value = "postgres")]
    user: String,

    /// Password
    #[structopt(long, default_value = "password")]
    password: String,

    /// Database name
    #[structopt(long, default_value = "postgres")]
    database: String,

    /// Optional a directory with migration scripts to apply
    #[structopt(long)]
    migration_dir: Option<PathBuf>,

    /// Port
    #[structopt(long, default_value = "5432")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::from_args();
    let should_migration = opt.migration_dir.is_some();

    // Postgresql settings
    let pg_settings = PgSettings {
        database_dir: PathBuf::from("data/db"),
        port: opt.port as i16,
        user: opt.user,
        password: opt.password,
        auth_method: PgAuthMethod::Plain,
        persistent: opt.persistent,
        timeout: Some(Duration::from_secs(15)),
        migration_dir: opt.migration_dir,
    };

    let version = PostgresVersion("14.1.0");
    let fetch_settings = PgFetchSettings {
        version,
        ..Default::default()
    };

    let maybe_pg = PgEmbed::new(pg_settings, fetch_settings).await;
    let mut pg = maybe_pg.map_err(|e| anyhow!("Invalid db settings\n {e:?}"))?;

    // Download, unpack, create password file and database cluster
    pg.setup()
        .await
        .map_err(|e| anyhow!("Failed to setup database\n {e:?}"))?;

    // start postgresql database
    pg.start_db()
        .await
        .map_err(|e| anyhow!("Failed to start database\n {e:?}"))?;

    if !pg.database_exists(&opt.database).await.unwrap() {
        pg.create_database(&opt.database)
            .await
            .map_err(|e| anyhow!("Failed to create database\n {e:?}"))?;
    };

    if should_migration {
        pg.migrate(&opt.database)
            .await
            .map_err(|e| anyhow!("Failed to apply migrations\n {e:?}"))?;
    }

    if opt.print_url {
        let pg_uri: &str = &pg.db_uri;
        println!("{pg_uri}");
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    println!("To exit press Ctrl-C...");
    while running.load(Ordering::SeqCst) {}
    println!("Got it! Stopping DB...");
    pg.stop_db()
        .await
        .map_err(|_e| anyhow!("Failed to stop DB"))?;

    return Ok(());
}
