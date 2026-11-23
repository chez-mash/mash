use super::*;

const NOW: u64 = 1_800_000_000;

fn token_hash(token: &str) -> String {
    blake3::hash(token.as_bytes()).to_hex().to_string()
}

fn added_with(node: &str) -> (Added, String) {
    let mut added = Added::default();
    let secret = add(&mut added, &[], "harbor", node, NOW).unwrap();
    (added, secret)
}

#[test]
fn a_join_secret_is_kept_as_a_hash_and_works_once() {
    let (mut added, secret) = added_with("studio");
    let text = toml::to_string(&added).unwrap();
    assert!(!text.contains(&secret), "the file keeps no secret");
    assert_eq!(check(&added, &secret, NOW).0, JoinStatus::Ready);
    let first = token_hash("first-token-0123456789abcdef0123456789");
    assert_eq!(
        redeem(&mut added, &secret, &first, NOW + 10),
        Ok(("harbor".to_owned(), "studio".to_owned()))
    );
    assert_eq!(added.nodes[0].state(NOW + 10), State::Joined);
    // A replayed string is refused for any other machine's token...
    let second = token_hash("second-token-0123456789abcdef012345678");
    assert_eq!(
        redeem(&mut added, &secret, &second, NOW + 20),
        Err(JoinStatus::Used)
    );
    assert_eq!(check(&added, &secret, NOW + 20).0, JoinStatus::Used);
    // ...and answers the same machine again, whose first answer was lost.
    assert!(redeem(&mut added, &secret, &first, NOW + 30).is_ok());
    assert_eq!(added.nodes[0].token_hash.as_deref(), Some(first.as_str()));
}

#[test]
fn a_join_secret_expires_a_day_after_the_server_made_it() {
    let (mut added, secret) = added_with("studio");
    let token = token_hash("token-0123456789abcdef0123456789abcdef");
    // The server's own clock decides, against the time it recorded.
    assert_eq!(
        check(&added, &secret, NOW + JOIN_LIFETIME - 1).0,
        JoinStatus::Ready
    );
    assert_eq!(
        check(&added, &secret, NOW + JOIN_LIFETIME).0,
        JoinStatus::Expired
    );
    assert_eq!(
        redeem(&mut added, &secret, &token, NOW + JOIN_LIFETIME),
        Err(JoinStatus::Expired)
    );
    assert_eq!(added.nodes[0].state(NOW + JOIN_LIFETIME), State::Expired);
    // A clock that ran backwards past creation still has a bounded window.
    assert_eq!(check(&added, &secret, NOW - 3600).0, JoinStatus::Ready);
}

#[test]
fn a_name_is_taken_by_any_added_or_configured_machine() {
    let (mut added, _) = added_with("studio");
    let error = add(&mut added, &[], "harbor", "studio", NOW).unwrap_err();
    assert_eq!(error.error.kind, crate::ErrorKind::Conflict);
    assert!(
        error
            .error
            .message
            .contains("already has a machine called studio")
    );
    let configured = [("harbor".to_owned(), "laptop".to_owned())];
    assert!(add(&mut added, &configured, "harbor", "laptop", NOW).is_err());
    // Another group may use the name, and a removed machine still holds it.
    assert!(add(&mut added, &configured, "other", "laptop", NOW).is_ok());
    remove(&mut added, "harbor", "studio", NOW).unwrap();
    assert!(add(&mut added, &[], "harbor", "studio", NOW).is_err());
    assert!(add(&mut added, &[], "harbor", "has space", NOW).is_err());
}

#[test]
fn replacing_a_credential_stops_the_old_one_at_once() {
    let (mut added, secret) = added_with("studio");
    let token = token_hash("token-0123456789abcdef0123456789abcdef");
    redeem(&mut added, &secret, &token, NOW).unwrap();
    let fresh = replace(&mut added, "harbor", "studio", NOW + 100).unwrap();
    assert_ne!(fresh, secret);
    let entry = &added.nodes[0];
    assert_eq!(entry.token_hash, None, "the old token no longer authorizes");
    assert_eq!(entry.created_at, Some(NOW + 100));
    assert!(revoked(&added, &blake3::Hash::from_hex(&token).unwrap()));
    assert_eq!(check(&added, &secret, NOW + 100).0, JoinStatus::Revoked);
    assert_eq!(check(&added, &fresh, NOW + 100).0, JoinStatus::Ready);
    // The old token cannot be registered again as the new one.
    assert_eq!(
        redeem(&mut added, &fresh, &token, NOW + 101),
        Err(JoinStatus::Unknown)
    );
    let other = token_hash("other-0123456789abcdef0123456789abcdef");
    assert!(redeem(&mut added, &fresh, &other, NOW + 101).is_ok());
}

#[test]
fn removing_keeps_the_machine_until_its_history_is_deleted() {
    let (mut added, secret) = added_with("studio");
    let token = token_hash("token-0123456789abcdef0123456789abcdef");
    redeem(&mut added, &secret, &token, NOW).unwrap();
    assert!(request_delete(&mut added, "harbor", "studio", NOW).is_err());
    remove(&mut added, "harbor", "studio", NOW + 5).unwrap();
    assert_eq!(
        added.nodes[0].state(NOW + 5),
        State::Removed {
            delete_requested: false
        }
    );
    assert!(revoked(&added, &blake3::Hash::from_hex(&token).unwrap()));
    request_delete(&mut added, "harbor", "studio", NOW + 6).unwrap();
    assert!(replace(&mut added, "harbor", "studio", NOW + 7).is_err());
    forget(&mut added, "harbor", "studio");
    assert_eq!(added.nodes.len(), 0);
    assert!(revoked(&added, &blake3::Hash::from_hex(&token).unwrap()));
}

#[test]
fn strangers_and_malformed_hashes_are_unknown() {
    let (mut added, _) = added_with("studio");
    assert_eq!(check(&added, "not-a-secret", NOW).0, JoinStatus::Unknown);
    let (_, secret) = added_with("studio");
    assert_eq!(
        redeem(&mut added, &secret, "not-hex", NOW),
        Err(JoinStatus::Unknown)
    );
}

#[test]
fn the_file_is_private_strict_and_survives_concurrent_writers() {
    let directory = tempfile::tempdir().unwrap();
    let configuration = directory.path().join("server.toml");
    let threads = (0..8)
        .map(|index| {
            let configuration = configuration.clone();
            std::thread::spawn(move || {
                update(&configuration, |added| {
                    add(added, &[], "harbor", &format!("machine-{index}"), NOW)
                })
                .unwrap()
            })
        })
        .collect::<Vec<_>>();
    for thread in threads {
        thread.join().unwrap();
    }
    let added = read(&configuration).unwrap();
    assert_eq!(added.nodes.len(), 8, "no writer lost another's machine");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(nodes_path(&configuration))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    for text in [
        "[[nodes]]\ngroup_id='g'\nnode_id='n'\ntoken_env='MASH_X'\n",
        "[[nodes]]\ngroup_id='g'\nnode_id='n'\n",
        "[[nodes]]\ngroup_id='g'\nnode_id='n'\ntoken_hash='00'\n",
        "[[nodes]]\ngroup_id='g'\nnode_id='n'\ntoken_hash='00'\nsurprise=1\n",
    ] {
        assert!(parse(text).is_err(), "{text}");
    }
    // A file written before join strings expired still reads.
    let old = parse(&format!(
        "\n[[nodes]]\ngroup_id = \"team\"\nnode_id = \"laptop\"\ntoken_hash = \"{}\"\n",
        token_hash("x")
    ))
    .unwrap();
    assert_eq!(old.nodes[0].state(NOW), State::Joined);
}

#[test]
fn the_fingerprint_is_the_end_of_the_kept_hash() {
    let (added, _) = added_with("studio");
    let fingerprint = added.nodes[0].fingerprint().unwrap();
    assert_eq!(fingerprint.len(), 4);
    assert!(
        added.nodes[0]
            .join_hash
            .as_ref()
            .unwrap()
            .ends_with(&fingerprint)
    );
}
