use super::run;
use crate::cli::args::WalgCommand;
use std::path::Path;

#[tokio::test]
async fn a_walg_command_without_a_config_reports_the_missing_file() {
    let missing = Path::new("/tmp/backito-no-such-config-anywhere.toml");

    let failure = run(WalgCommand::Base, Some(missing))
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
    let missing = Path::new("/tmp/backito-no-such-config-anywhere.toml");

    let failure = run(
        WalgCommand::Archive {
            segment: "000000010000000000000001".to_owned(),
        },
        Some(missing),
    )
    .await
    .expect_err("a config that is not there cannot be read");

    assert!(
        failure.to_string().contains("read config"),
        "got: {failure}"
    );
}
