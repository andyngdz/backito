use super::{CONFIG_FILENAME, Overwrite, template, write_config};
use crate::features::init::InitError;
use crate::infra::config::{ConfigSource, ContainerSource, TomlSource};
use tempfile::TempDir;

#[test]
fn the_config_lands_under_its_expected_name() {
    let directory = TempDir::new().expect("temp dir");

    let path = write_config(directory.path(), Overwrite::Refuse).expect("write");

    assert_eq!(
        path.file_name().and_then(|name| name.to_str()),
        Some(CONFIG_FILENAME)
    );
    assert!(path.exists());
}

#[test]
fn the_written_template_parses_as_a_config() {
    // A template that does not load is worse than none: the first thing a user
    // does after `init` is run a command against it.
    let directory = TempDir::new().expect("temp dir");
    let path = write_config(directory.path(), Overwrite::Refuse).expect("write");

    // Only the config half is checked here: the template carries no credentials
    // by design, so reading it needs no secret source.
    let core = TomlSource::new(Some(&path))
        .load()
        .expect("the shipped template must parse");

    assert!(matches!(
        core.database.container,
        ContainerSource::Named(ref name) if !name.is_empty()
    ));
    assert!(!core.storage.bucket.is_empty());
}

#[test]
fn the_template_leaves_the_endpoint_as_a_placeholder() {
    // Shipping a real endpoint would be someone else's account id.
    assert!(template().contains("<account-id>"));
}

#[test]
fn an_existing_config_is_not_overwritten_by_default() {
    let directory = TempDir::new().expect("temp dir");
    let path = directory.path().join(CONFIG_FILENAME);
    std::fs::write(&path, "# hand-written\n").expect("seed");

    let failure = write_config(directory.path(), Overwrite::Refuse).expect_err("must refuse");

    assert!(matches!(failure, InitError::ConfigExists { .. }));
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        "# hand-written\n",
        "the user's file must be left exactly as it was"
    );
}

#[test]
fn force_replaces_an_existing_config() {
    let directory = TempDir::new().expect("temp dir");
    let path = directory.path().join(CONFIG_FILENAME);
    std::fs::write(&path, "# hand-written\n").expect("seed");

    write_config(directory.path(), Overwrite::Allow).expect("write");

    assert_eq!(std::fs::read_to_string(&path).expect("read"), template());
}
