use super::resolve;
use crate::infra::config::ContainerSource;

#[tokio::test]
async fn a_pinned_name_is_returned_without_asking_docker() {
    let source = ContainerSource::Named("app-db".to_owned());

    let resolved = resolve(&source)
        .await
        .expect("a pinned name always resolves");

    assert_eq!(resolved, "app-db");
}

#[tokio::test]
async fn an_unmatched_service_names_the_label_it_looked_for() {
    let source = ContainerSource::Service {
        label: "com.docker.compose.service".to_owned(),
        service: "no-such-service-anywhere".to_owned(),
    };

    let failure = resolve(&source)
        .await
        .expect_err("no container carries this label");

    let message = failure.to_string();
    assert!(
        message.contains("com.docker.compose.service")
            && message.contains("no-such-service-anywhere"),
        "the failure should name both halves of the filter, got: {message}"
    );
}
