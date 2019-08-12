use datamodel;
use migration_connector::*;
use migration_core::{parse_datamodel, MigrationEngine};
use sql_migration_connector::database_inspector::*;
use sql_migration_connector::migration_database::{MigrationDatabase, Sqlite};
use sql_migration_connector::SqlFamily;
use sql_migration_connector::SqlMigrationConnector;
use std::sync::Arc;

pub const SCHEMA_NAME: &str = "migration_engine";

pub fn parse(datamodel_string: &str) -> datamodel::Datamodel {
    parse_datamodel(datamodel_string).unwrap()
}

pub fn test_each_connector<F, C, D>(test_fn: F)
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
    F: Fn(SqlFamily, &MigrationEngine<C, D>) -> () + std::panic::RefUnwindSafe,
{
    test_each_connector_with_ignores(Vec::new(), test_fn);
}

pub fn test_only_connector<F, C, D>(sql_family: SqlFamily, test_fn: F)
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
    F: Fn(SqlFamily, &MigrationEngine<C, D>) -> () + std::panic::RefUnwindSafe,
{
    let all = vec![SqlFamily::Postgres, SqlFamily::Mysql, SqlFamily::Sqlite];
    let ignores = all.into_iter().filter(|f| f != &sql_family).collect();
    test_each_connector_with_ignores(ignores, test_fn);
}

pub fn test_each_connector_with_ignores<F, C, D>(ignores: Vec<SqlFamily>, test_fn: F)
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
    F: Fn(SqlFamily, &MigrationEngine<C, D>) -> () + std::panic::RefUnwindSafe,
{
    // SQLite
    if !ignores.contains(&SqlFamily::Sqlite) {
        println!("Testing with SQLite now");

        let config = sqlite_test_config();
        let engine = test_engine(&config.url());

        test_fn(SqlFamily::Sqlite, &engine);
    } else {
        println!("Ignoring SQLite")
    }

    // POSTGRES
    if !ignores.contains(&SqlFamily::Postgres) {
        println!("Testing with Postgres now");

        let config = postgres_test_config();
        let engine = test_engine(&config.url());

        test_fn(SqlFamily::Postgres, &engine);
    } else {
        println!("Ignoring Postgres")
    }

    // MYSQL
    if !ignores.contains(&SqlFamily::Mysql) {
        println!("Testing with MySQL now");

        let engine = test_engine(&mysql_test_config());
        println!("ENGINE DONE");

        test_fn(SqlFamily::Mysql, &engine);
    } else {
        println!("Ignoring MySQL")
    }
}

pub fn test_engine<C, D>(connector: C) -> MigrationEngine<C, D>
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
{
    let engine = MigrationEngine::new(connector).unwrap();

    engine.reset().expect("Engine reset failed.");
    engine.init().expect("Engine init failed");

    engine
}

pub fn introspect_database<C, D>(engine: &MigrationEngine<C, D>) -> DatabaseSchema
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
{
    let inspector: Box<DatabaseInspector> = match engine.connector().connector_type() {
        "postgresql" => Box::new(DatabaseInspector::postgres(postgres_url())),
        "sqlite" => Box::new(DatabaseInspector::sqlite(sqlite_test_file())),
        "mysql" => Box::new(DatabaseInspector::mysql(mysql_url())),
        _ => unimplemented!(),
    };
    let mut result = inspector.introspect(&SCHEMA_NAME.to_string());
    // the presence of the _Migration table makes assertions harder. Therefore remove it from the result.
    result.tables = result.tables.into_iter().filter(|t| t.name != "_Migration").collect();
    result
}

pub fn database(sql_family: SqlFamily) -> Arc<MigrationDatabase> {
    match sql_family {
        SqlFamily::Postgres => postgres_database(),
        SqlFamily::Sqlite => sqlite_database(),
        SqlFamily::Mysql => mysql_database(),
    }
}

fn postgres_database() -> Arc<MigrationDatabase> {
    let postgres = SqlMigrationConnector::postgres_helper(&postgres_url());
    postgres.db_connection
}

fn sqlite_database() -> Arc<MigrationDatabase> {
    Arc::new(Sqlite::new(&sqlite_test_file()).expect("Loading SQLite failed"))
}

fn mysql_database() -> Arc<MigrationDatabase> {
    let helper = SqlMigrationConnector::mysql_helper(&mysql_url());
    helper.db_connection
}

pub fn sqlite_test_config() -> String {
    format!(
        r#"
        datasource my_db {{
            provider = "sqlite"
            url = "file:{}"
            default = true
        }}
    "#,
        sqlite_test_file()
    )
}

pub fn sqlite_test_file() -> String {
    let server_root = std::env::var("SERVER_ROOT").expect("Env var SERVER_ROOT required but not found.");
    let database_folder_path = format!("{}/db", server_root);
    let file_path = format!("{}/{}.db", database_folder_path, SCHEMA_NAME);
    file_path
}

pub fn postgres_test_config() -> String {
    format!(
        r#"
        datasource my_db {{
            provider = "postgresql"
            url = "{}"
            default = true
        }}
    "#,
        postgres_url()
    )
}

pub fn mysql_test_config() -> String {
    format!(
        r#"
        datasource my_db {{
            provider = "mysql"
            url = "{}"
            default = true
        }}
    "#,
        mysql_url()
    )
}

pub fn postgres_url() -> String {
    dbg!(format!(
        "postgresql://postgres:prisma@{}:5432/db?schema={}",
        db_host_postgres(),
        SCHEMA_NAME
    ))
}

pub fn mysql_url() -> String {
    dbg!(format!("mysql://root:prisma@{}:3306/{}", db_host_mysql(), SCHEMA_NAME))
}

fn db_host_postgres() -> String {
    match std::env::var("IS_BUILDKITE") {
        Ok(_) => "test-db-postgres".to_string(),
        Err(_) => "127.0.0.1".to_string(),
    }
}

fn db_host_mysql() -> String {
    match std::env::var("IS_BUILDKITE") {
        Ok(_) => "test-db-mysql".to_string(),
        Err(_) => "127.0.0.1".to_string(),
    }
}
