use super::{ArchiveDigest, ArchiveName};

#[test]
fn archive_key_carries_label_stamp_and_extension() {
    let archive = ArchiveName::new("app", "20260803-0942");

    assert_eq!(archive.as_str(), "app-backup-20260803-0942.dump");
    assert_eq!(
        archive.checksum_key(),
        "app-backup-20260803-0942.dump.sha256"
    );
}

#[test]
fn keys_sort_chronologically_as_plain_strings() {
    let older = ArchiveName::new("app", "20260803-0942");
    let newer = ArchiveName::new("app", "20260804-0101");

    // `verify` picks the newest archive by string order, so this ordering is a
    // contract, not an incidental property of the format.
    assert!(newer > older);
}

#[test]
fn only_this_label_s_archives_are_adopted() {
    assert!(ArchiveName::belongs_to(
        "app-backup-20260803-0942.dump",
        "app"
    ));
    assert!(!ArchiveName::belongs_to("app-data.tar.gz", "app"));
    assert!(!ArchiveName::belongs_to(
        "app-backup-20260803-0942.dump.sha256",
        "app"
    ));
    // Another project sharing the bucket must never be picked up.
    assert!(!ArchiveName::belongs_to(
        "other-backup-20260803-0942.dump",
        "app"
    ));
}

#[test]
fn a_legacy_naming_scheme_is_not_mistaken_for_this_one() {
    // These two coexisted in a real bucket. Sorting whole keys puts the OLDER
    // `legacy-` one last, because "app-b" < "legacy-b" -- so a listing that
    // accepted both would verify a stale archive and call it the newest.
    let ours = "app-backup-20260803-1047.dump";
    let legacy = "legacy-backup-20260803-0942.dump";

    assert!(ArchiveName::belongs_to(ours, "app"));
    assert!(!ArchiveName::belongs_to(legacy, "app"));
    assert!(ours < legacy, "the trap this filter exists to avoid");
}

#[test]
fn sidecar_round_trips_through_sha256sum_layout() {
    let archive = ArchiveName::new("app", "20260803-0942");
    let digest = ArchiveDigest::from_hex("451177649638251ebbec5ab96579b6d7");

    let body = digest.to_sidecar(&archive);

    assert_eq!(
        body,
        "451177649638251ebbec5ab96579b6d7  app-backup-20260803-0942.dump\n"
    );
    assert_eq!(ArchiveDigest::from_sidecar(&body), Some(digest));
}

#[test]
fn sidecar_without_a_digest_parses_to_none() {
    assert_eq!(ArchiveDigest::from_sidecar("   \n"), None);
}
