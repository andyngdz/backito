use super::target_for;
use crate::infra::config::ContainerSource;
use crate::infra::config::ScheduleSettings;
use crate::infra::config::{DatabaseSettings, Settings, StorageCredentials, StorageSettings};

fn settings() -> Settings {
    Settings {
        database: DatabaseSettings {
            label: "app".to_owned(),
            container: ContainerSource::Named("app-db".to_owned()),
            name: "postgres".to_owned(),
            user: "readonly".to_owned(),
            image: "postgres:17".to_owned(),
            restore_jobs: 4,
        },
        storage: StorageSettings {
            endpoint: "http://127.0.0.1:1".to_owned(),
            bucket: "backito-test".to_owned(),
            region: "auto".to_owned(),
        },
        credentials: StorageCredentials {
            access_key_id: "test-access-key".to_owned(),
            secret_access_key: "test-secret-key".to_owned(),
        },
        schedule: ScheduleSettings::default(),
    }
}

#[test]
fn the_target_comes_from_configuration_not_defaults() {
    let target = target_for(&settings().database, "app-db".to_owned());

    assert_eq!(target.container, "app-db");
    assert_eq!(target.database, "postgres");
    // A configured non-default role must survive: dumping as the wrong role is
    // how a backup silently loses tables it cannot read.
    assert_eq!(target.user, "readonly");
}
