use super::{PostgresTarget, PostgresTool};

fn target() -> PostgresTarget {
    PostgresTarget {
        container: "app-db".to_owned(),
        database: "postgres".to_owned(),
        user: "postgres".to_owned(),
    }
}

#[test]
fn each_tool_names_its_binary() {
    assert_eq!(PostgresTool::Dump.as_arg(), "pg_dump");
    assert_eq!(PostgresTool::Restore.as_arg(), "pg_restore");
    assert_eq!(PostgresTool::Psql.as_arg(), "psql");
}

#[test]
fn the_exec_prefix_targets_the_configured_container_and_database() {
    let prefix = target().exec_prefix(PostgresTool::Dump);

    assert_eq!(
        prefix,
        vec![
            "exec".to_owned(),
            "app-db".to_owned(),
            "pg_dump".to_owned(),
            "-U".to_owned(),
            "postgres".to_owned(),
            "-d".to_owned(),
            "postgres".to_owned(),
        ]
    );
}

#[test]
fn every_argument_stays_separate_so_a_name_cannot_become_shell_syntax() {
    let hostile = PostgresTarget {
        container: "db; rm -rf /".to_owned(),
        database: "postgres".to_owned(),
        user: "postgres".to_owned(),
    };

    let prefix = hostile.exec_prefix(PostgresTool::Psql);

    // The whole hostile string is one argument; docker receives it as a
    // container name, never as something a shell could split.
    assert!(prefix.contains(&"db; rm -rf /".to_owned()));
}
