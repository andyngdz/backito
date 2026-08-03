//! Driving Postgres through the `docker` CLI.

mod container;
mod count_query;
mod errors;
mod inspect_archive;
mod postgres_cli;

pub use container::{DOCKER_BIN, is_running, remove, require_running, start_throwaway, wait_ready};
pub use errors::DockerError;
pub use inspect_archive::count_table_data_entries;
pub use postgres_cli::{
    PostgresTarget, copy_into, dump_to_file, restore_in_container, table_counts,
};
