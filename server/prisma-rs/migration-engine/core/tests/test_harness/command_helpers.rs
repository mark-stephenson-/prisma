use super::introspect_database;
use migration_connector::*;
use migration_core::{commands::*, MigrationEngine};
use sql_migration_connector::database_inspector::*;

pub fn infer_and_apply<C, D>(engine: &MigrationEngine<C, D>, datamodel: &str) -> DatabaseSchema
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
{
    infer_and_apply_with_migration_id(&engine, &datamodel, "the-migration-id")
}

pub fn infer_and_apply_with_migration_id<C, D>(
    engine: &MigrationEngine<C, D>,
    datamodel: &str,
    migration_id: &str,
) -> DatabaseSchema
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
{
    let input = InferMigrationStepsInput {
        migration_id: migration_id.to_string(),
        datamodel: datamodel.to_string(),
        assume_to_be_applied: Vec::new(),
    };

    let steps = run_infer_command(&engine, input);

    apply_migration(&engine, steps, migration_id)
}

pub fn run_infer_command<C, D>(engine: &MigrationEngine<C, D>, input: InferMigrationStepsInput) -> Vec<MigrationStep>
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
{
    let cmd = InferMigrationStepsCommand::new(input);
    let output = cmd.execute(&engine).expect("InferMigration failed");

    assert!(
        output.general_errors.is_empty(),
        format!("InferMigration returned unexpected errors: {:?}", output.general_errors)
    );

    output.datamodel_steps
}

pub fn apply_migration<C, D>(
    engine: &MigrationEngine<C, D>,
    steps: Vec<MigrationStep>,
    migration_id: &str,
) -> DatabaseSchema
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
{
    let input = ApplyMigrationInput {
        migration_id: migration_id.to_string(),
        steps: steps,
        force: None,
    };

    let cmd = ApplyMigrationCommand::new(input);
    let output = cmd.execute(&engine).expect("ApplyMigration failed");

    assert!(
        output.general_errors.is_empty(),
        format!("ApplyMigration returned unexpected errors: {:?}", output.general_errors)
    );

    introspect_database(&engine)
}

pub fn unapply_migration<C, D>(engine: &MigrationEngine<C, D>) -> DatabaseSchema
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
{
    let input = UnapplyMigrationInput {};
    let cmd = UnapplyMigrationCommand::new(input);
    let _ = cmd.execute(&engine);

    introspect_database(&engine)
}
