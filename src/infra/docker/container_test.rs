use super::{DockerSubcommand, is_running};

#[test]
fn subcommands_render_their_docker_argument() {
    assert_eq!(DockerSubcommand::Run.as_arg(), "run");
    assert_eq!(DockerSubcommand::Rm.as_arg(), "rm");
    assert_eq!(DockerSubcommand::Exec.as_arg(), "exec");
    assert_eq!(DockerSubcommand::Ps.as_arg(), "ps");
}

#[tokio::test]
async fn an_unknown_container_reads_as_not_running() {
    // An empty `docker ps` listing is an answer, not an error. Only docker
    // itself being unusable is a failure, which is the distinction this call
    // exists to keep.
    let running = is_running("backito-container-that-does-not-exist")
        .await
        .expect("unknown container must not fail the call");

    assert!(!running);
}
