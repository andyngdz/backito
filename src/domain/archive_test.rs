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

#[test]
fn the_stamp_is_read_back_out_of_the_key() {
    let archive = ArchiveName::new("app", "20260803-0942");

    assert_eq!(archive.stamp(), Some("20260803-0942"));
}

#[test]
fn a_label_containing_hyphens_still_yields_its_stamp() {
    let archive = ArchiveName::new("crit-prod", "20260808-1830");

    assert_eq!(archive.as_str(), "crit-prod-backup-20260808-1830.dump");
    assert_eq!(archive.stamp(), Some("20260808-1830"));
}

#[test]
fn a_key_this_tool_did_not_write_yields_no_stamp() {
    for foreign in [
        "someone-elses-object.dump",
        "nostamp",
        "app-backup-20260803-0942.sha256",
    ] {
        let adopted = ArchiveName::from_key(foreign);

        assert_eq!(adopted.stamp(), None, "reading {foreign}");
    }
}

#[test]
fn a_label_ending_in_the_stem_takes_the_rightmost_marker() {
    let archive = ArchiveName::new("nightly-backup", "20260803-0942");

    assert_eq!(archive.as_str(), "nightly-backup-backup-20260803-0942.dump");
    assert_eq!(archive.stamp(), Some("20260803-0942"));
}

#[test]
fn a_typed_key_for_this_label_is_accepted() {
    let parsed = ArchiveName::parse_for("app-backup-20260803-0942.dump", "app");

    assert_eq!(
        parsed.map(|name| name.as_str().to_owned()),
        Some("app-backup-20260803-0942.dump".to_owned())
    );
}

#[test]
fn a_typed_key_that_escapes_the_workspace_is_refused() {
    // The key becomes a local filename when the archive downloads, so a path
    // segment inside it writes outside the scratch directory. Matching the label
    // is not enough on its own: the first two here clear that test.
    for escaping in [
        "app-backup-../../etc/passwd.dump",
        "app-backup-../20260803-0942.dump",
        "../app-backup-20260803-0942.dump",
        "/etc/app-backup-20260803-0942.dump",
    ] {
        assert!(
            ArchiveName::parse_for(escaping, "app").is_none(),
            "{escaping} should be refused"
        );
    }
}

#[test]
fn a_typed_key_from_another_label_is_refused() {
    assert!(ArchiveName::parse_for("other-backup-20260803-0942.dump", "app").is_none());
}

#[test]
fn the_sidecar_is_refused_where_the_dump_is_wanted() {
    // A common slip: copy the checksum key out of a bucket listing and pass it
    // to --archive. Downloading it would restore nothing.
    assert!(ArchiveName::parse_for("app-backup-20260803-0942.dump.sha256", "app").is_none());
}

#[test]
fn a_key_whose_stamp_is_not_a_stamp_is_refused() {
    for malformed in [
        "app-backup-.dump",
        "app-backup-2026080-0942.dump",
        "app-backup-20260803-094.dump",
        "app-backup-20260803x0942.dump",
        "app-backup-abcdefgh-0942.dump",
    ] {
        assert!(
            ArchiveName::parse_for(malformed, "app").is_none(),
            "{malformed} should be refused"
        );
    }
}
