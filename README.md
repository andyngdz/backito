# backito

Back up a containerised Postgres database to S3-compatible storage, and prove
the archive restores.

A backup nobody has restored is a guess. `backito verify` turns that guess into
evidence: it pulls the newest archive, loads it into a throwaway container, and
compares row counts table by table against the live database.

## Install

```bash
cargo install --path .
```

Needs `docker` on PATH. Postgres client tools are not needed on the host —
`pg_dump` and `pg_restore` run inside the container, so their version always
matches the server's.

## Configure

Run this once per project:

```bash
cd your-project
backito init
```

It writes `backito.toml` and adds it to `.gitignore`, then tells you what is
left to fill in. Re-running refuses to clobber your file unless you pass
`--force`.

The config is gitignored on purpose: an R2 endpoint carries your account id, so
it is per-machine setup rather than shared source. Open it and fill in
`endpoint` and `bucket`:

```toml
[database]
label     = "app"          # archive keys start with this
container = "app-db"       # docker container running Postgres
name      = "postgres"     # database to dump
user      = "postgres"     # role, defaults to postgres
image     = "postgres:17"  # image `verify` restores into
restore_jobs = 4           # pg_restore parallelism, drop to 1 for a tight target

[storage]
endpoint = "https://<account-id>.r2.cloudflarestorage.com"
bucket   = "app-database-backups"
region   = "auto"

[schedule]                     # only `daemon` and `health` read this
backup_interval = "24h"
verify_interval = "7d"         # "0s" disables verification
retain          = 7            # archives kept per label
```

Intervals are written as a number and a unit: `30s`, `15m`, `24h`, `7d`. Every
`[schedule]` value has a default, so a project that only runs one-shot commands
can leave the table out.

### Naming the container, or naming the service

`container` pins one container by name. That is exact, and it stays exact only
while something guarantees the name. Under an orchestrator nothing does: compose
derives `<project>-<service>-<n>`, and uncloud appends a fresh random suffix on
every redeploy, so a name written into the config goes stale the next time you
deploy.

Name the service instead, and backito asks Docker which container is running it:

```toml
[database]
service = "db"                              # instead of container
container_label = "uncloud.service.name"    # optional, see below
```

Set exactly one of `container` and `service`; backito refuses a config with both
or neither. `container_label` defaults to `com.docker.compose.service`, which is
what plain compose writes, so a compose project needs nothing but the `service`
line. Other orchestrators label differently: uncloud writes
`uncloud.service.name`.

`daemon` resolves the container on every pass rather than once at startup, so a
redeploy partway through a schedule does not leave it talking about a container
that no longer exists.

Credentials come from the environment:

```bash
export BACKITO_ACCESS_KEY_ID='...'
export BACKITO_SECRET_ACCESS_KEY='...'
```

Give the bucket its own credential. A token scoped to one bucket cannot damage
anything else if it leaks — and `backito` never probes or creates a bucket, so a
scoped token is enough.

## Use

```bash
backito init              # write backito.toml and gitignore it
backito backup            # dump, check, hash, upload
backito verify            # prove the newest archive restores
backito restore --force   # load an archive into a real database
backito daemon            # back up on a schedule until stopped
backito health            # is there a recent enough backup? exit code says
```

`backup` prints the stored object key on stdout and nothing else, so it composes:

```bash
KEY=$(backito backup)
```

Progress goes to stderr:

```
✓ Checking database connection — app-db
✓ Checking storage connection — app-database-backups (7 objects)
✓ Backing up database — 882.34 MiB
✓ Inspecting archive — 45 tables with data
✓ Computing checksum — 4511776496382…
⠹ Uploading archive [========>               ] 331 MiB/882 MiB (18 MiB/s)
```

## What `verify` actually checks

```
PASS  app-backup-20260803-0942.dump restored into a scratch database and matched the source
      44 tables compared
      591 rows behind the source, which kept writing after the dump
      78 pg_restore errors, which do not decide the result
      checksum: matches the digest stored with the archive
```

Three things that trip people up, encoded here so they do not have to be
rediscovered:

**`pg_restore` errors are not failures.** Restoring into a managed Postgres
image reports dozens of errors for system objects the image already owns —
event triggers, the `auth` schema, extensions. The count is shown for
transparency and never decides the result.

**A restored copy behind the source is drift, not loss.** The source keeps
taking writes after the dump is taken. `backito` names that as drift. A restored
copy *ahead* of the source, or a table missing from either side, fails —
neither can be explained by drift.

**A missing checksum is a failure.** An archive whose bytes cannot be checked
has been restored, not verified.

Exit codes: `0` pass, `1` the command could not run, `2` verification ran and
found a mismatch. A scheduled check can tell those apart.

## Safety

`verify` only ever writes to a container named `backito-scratch-<label>`, has no
volume and no published port, and removes it when done. It cannot touch a real
database because it never addresses one.

`restore` is the only command that writes into a database you already run. It
refuses a target holding data unless `--force` is passed. There is no
interactive prompt: a prompt that can hang a cron job is worse than a flag.

## Scheduling

Two ways, depending on whether something else already supervises processes.

### In a container: `backito daemon`

```bash
backito daemon
```

It backs up, prunes to `retain`, verifies when `verify_interval` comes round,
and sleeps. A failed pass is reported and retried rather than ending the loop:
a scheduler that exits on its first failure stops backing up at the moment
something is wrong, which is the moment backups matter. A bucket it cannot list
at startup does end it, because that is a configuration that will never work
rather than an outage that will pass.

On start it asks the bucket when the last backup landed and waits out whatever
is left of the interval. Restarting the container does not take another backup,
so redeploying five times in an afternoon still leaves one archive for the day.

### As a healthcheck: `backito health`

```yaml
healthcheck:
  test: ["CMD", "backito", "health"]
  interval: 5m
```

Exit 0 while the newest archive is younger than two `backup_interval` periods,
exit 1 once it is older, or when there is no readable archive at all. One missed
backup is a retry; two is a pattern.

It reads the bucket rather than a local marker file, which is what makes it
survive the container: a restarted or rebuilt process reports the same answer as
the one it replaced, instead of looking healthy because it has forgotten. Note
that this is not a liveness probe, and that is the point. A backup loop that is
running fine but has stopped being able to upload looks exactly like one that
works, and that is the failure worth catching.

### On a host: cron

```bash
PATH=/home/you/.cargo/bin:/usr/local/bin:/usr/bin:/bin

0 3 * * *  cd /path/to/project && backito backup
0 4 * * 0  cd /path/to/project && flock -n /tmp/backito.lock backito verify
```

No quiet flag is needed: the progress display renders nothing when stderr is not
a terminal, so a scheduled run is silent unless something went wrong — and then
cron mails you the warning or error. A silent run means a clean one.

Three things cron gets wrong by default:

- **PATH is minimal.** `backito` calls `docker`, and is itself usually in
  `~/.cargo/bin`. Set `PATH` at the top of the crontab, as above.
- **The working directory is `$HOME`.** `backito` reads `backito.toml` from the
  current directory, so `cd` first or pass `--config /absolute/path`.
- **Overlapping runs collide.** The scratch and inspect containers have fixed
  names, and starting one removes a container of the same name. `flock` keeps a
  long verify from being cut short by the next backup.

## License

MIT
