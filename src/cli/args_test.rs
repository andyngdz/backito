use super::{Cli, Command, WalgCommand};
use clap::{CommandFactory, Parser};

#[test]
fn the_grammar_itself_is_well_formed() {
    // clap's own audit of the derived grammar: duplicate argument ids, a
    // `conflicts_with` naming an argument that does not exist, and a misplaced
    // `trailing_var_arg` all panic here rather than at a user's first run.
    Cli::command().debug_assert();
}

#[test]
fn config_and_env_cannot_both_be_given() {
    // They are two whole config sources. Letting both through would mean one
    // silently filling the other's gaps, which is exactly what the two-source
    // split exists to prevent.
    let refused = Cli::try_parse_from(["backito", "--env", "--config", "prod.toml", "backup"]);

    assert!(refused.is_err());
}

#[test]
fn the_entrypoint_keeps_every_argument_meant_for_the_program_it_hands_over_to() {
    // The whole point of `walg entrypoint` is to exec the image's own command,
    // so a flag meant for that command must not be parsed as one of ours.
    let cli = Cli::try_parse_from([
        "backito",
        "walg",
        "entrypoint",
        "--fragment",
        "/etc/wal-g.conf",
        "docker-entrypoint.sh",
        "postgres",
        "-c",
        "shared_buffers=1GB",
    ])
    .expect("the trailing program and its arguments must parse");

    let Command::Walg(WalgCommand::Entrypoint { program, args, .. }) = cli.command else {
        panic!("expected the entrypoint command");
    };
    assert_eq!(program, "docker-entrypoint.sh");
    assert_eq!(args, ["postgres", "-c", "shared_buffers=1GB"]);
}

#[test]
fn a_bare_verify_takes_the_newest_archive() {
    let cli = Cli::try_parse_from(["backito", "verify"]).expect("verify needs no arguments");

    assert!(matches!(cli.command, Command::Verify { archive: None }));
}

#[test]
fn list_defaults_to_the_readable_shape() {
    let cli = Cli::try_parse_from(["backito", "list"]).expect("list needs no arguments");

    assert!(matches!(cli.command, Command::List { keys_only: false }));
}
