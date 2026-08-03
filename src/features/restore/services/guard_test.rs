use super::{RestoreAuthorisation, ensure_writable};
use crate::infra::docker::PostgresTarget;

fn target() -> PostgresTarget {
    PostgresTarget {
        container: "backito-target-that-does-not-exist".to_owned(),
        database: "postgres".to_owned(),
        user: "postgres".to_owned(),
    }
}

#[tokio::test]
async fn force_skips_the_emptiness_check_entirely() {
    // The target does not exist, so reaching the check at all would error.
    // Passing means --force short-circuited before any database call.
    let authorised = ensure_writable(&target(), &RestoreAuthorisation::Forced).await;

    assert!(authorised.is_ok());
}

#[tokio::test]
async fn without_force_a_missing_target_cannot_be_declared_empty() {
    // Refusing here is the point: "I could not look" must never read as
    // "there was nothing there".
    let refused = ensure_writable(&target(), &RestoreAuthorisation::RequireEmpty)
        .await
        .expect_err("an unreachable target must not pass as empty");

    assert!(!refused.to_string().is_empty());
}
