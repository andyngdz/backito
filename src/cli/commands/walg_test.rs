use super::super::SourceChoice;
use super::run;
use crate::cli::args::WalgCommand;
use std::path::PathBuf;

fn missing_config() -> SourceChoice {
    SourceChoice::File(PathBuf::from("/tmp/backito-no-such-config-anywhere.toml"))
}

#[tokio::test]
async fn a_walg_command_without_a_config_reports_the_missing_file() {
    let failure = run(WalgCommand::Base, &missing_config())
        .await
        .expect_err("a config that is not there cannot be read");

    // Every walg subcommand loads configuration first, so a missing file has to
    // read as a missing file rather than as a wal-g failure.
    assert!(
        failure.to_string().contains("read config"),
        "got: {failure}"
    );
}

#[tokio::test]
async fn archive_also_needs_a_config_before_it_can_decide_to_skip() {
    // `archive` tolerates a missing [walg] table, but not a missing config: the
    // difference between "archiving is off" and "nothing was configured at all"
    // is one Postgres should hear about.
    let failure = run(
        WalgCommand::Archive {
            segment: "000000010000000000000001".to_owned(),
        },
        &missing_config(),
    )
    .await
    .expect_err("a config that is not there cannot be read");

    assert!(
        failure.to_string().contains("read config"),
        "got: {failure}"
    );
}

#[test]
fn the_source_choice_travels_into_the_archive_command_as_a_flag() {
    // Postgres re-invokes backito from a minimal environment, so whichever
    // source this run used has to be named on that command line.
    assert_eq!(SourceChoice::Environment.cli_flags(), "--env");
    assert_eq!(
        SourceChoice::File(PathBuf::from("/etc/backito/backito.toml")).cli_flags(),
        "--config /etc/backito/backito.toml"
    );
}
