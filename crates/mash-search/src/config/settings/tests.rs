use super::*;

/// A history location that is absolute on the host running the test, as
/// configured roots must be.
fn home(name: &str) -> PathBuf {
    PathBuf::from(if cfg!(windows) { r"C:\h" } else { "/h" }).join(name)
}

fn file() -> String {
    format!(
        "version = 1\n\n[providers.codex]\nroots = ['{}']\n\n[index]\nall_history = true\n\n[views.query]\nlimit = 7\n",
        home("codex").display()
    )
}

fn detected() -> ProviderRoots {
    ProviderRoots::from([(Provider::ClaudeCode, vec![home("claude")])])
}

fn change(text: &str) -> Change {
    Change {
        revision: revision(text),
        agents: BTreeMap::new(),
        since_days: None,
        destination: IndexDestination::Local,
        memory_capture: true,
        hide_credentials: crate::redaction::Settings::default(),
        unshared: None,
        assistant: None,
    }
}

fn setup(text: &str, base_url: &str, key: &str) -> AssistantSetup {
    AssistantSetup {
        revision: revision(text),
        base_url: Some(base_url.into()),
        model: Some("anthropic/claude-haiku-5.5".into()),
        key: key.into(),
    }
}

#[test]
fn the_assistant_can_be_set_up_with_a_key_saved_under_its_variable() {
    let unset = |_: &str| false;
    let text = file();
    let (updated, name) = assistant_revision(
        &text,
        &setup(&text, "https://openrouter.ai/api/v1", " sk-1 "),
        unset,
    )
    .unwrap();
    assert_eq!(name, "MASH_ASSISTANT_KEY");
    let view = describe(Path::new("/c.toml"), &updated, &detected()).unwrap();
    assert_eq!(
        view.assistant,
        Some(Assistant {
            enabled: true,
            model: "anthropic/claude-haiku-5.5".into()
        })
    );
    // The key itself never goes into the file.
    assert!(!updated.contains("sk-1"));
    // A file that already names the key's variable keeps that name.
    let named = format!(
        "{}\n[assistant]\nbase_url = 'https://x.test/v1'\ntoken_env = 'KEY'\nmodel = 'a/b'\nenabled = false\n",
        file()
    );
    let (_, name) = assistant_revision(
        &named,
        &setup(&named, "https://openrouter.ai/api/v1", "k"),
        unset,
    )
    .unwrap();
    assert_eq!(name, "KEY");
    // A new key alone keeps the configured service and model, and turns Mashy on.
    let key_only = AssistantSetup {
        revision: revision(&named),
        base_url: None,
        model: None,
        key: "k2".into(),
    };
    let (updated, _) = assistant_revision(&named, &key_only, unset).unwrap();
    assert!(updated.contains("https://x.test/v1") && updated.contains("a/b"));
    let view = describe(Path::new("/c.toml"), &updated, &detected()).unwrap();
    assert_eq!(
        view.assistant.map(|assistant| assistant.enabled),
        Some(true)
    );
    // Without a configured service, a key alone is not enough.
    let bare = AssistantSetup {
        revision: revision(&text),
        base_url: None,
        model: None,
        key: "k".into(),
    };
    assert!(assistant_revision(&text, &bare, unset).is_err());
}

#[test]
fn setting_the_assistant_up_refuses_what_would_not_work() {
    let unset = |_: &str| false;
    let text = file();
    let good = "https://openrouter.ai/api/v1";
    for key in ["", "  ", "two\nlines"] {
        assert!(assistant_revision(&text, &setup(&text, good, key), unset).is_err());
    }
    // A plain-HTTP service away from this machine would send the key in the clear.
    assert!(assistant_revision(&text, &setup(&text, "http://example.com/v1", "k"), unset).is_err());
    // A variable set for the dashboard would be used instead of the saved key.
    assert!(assistant_revision(&text, &setup(&text, good, "k"), |_: &str| true).is_err());
    // An edit made elsewhere in between is not overwritten.
    let mut stale = setup(&text, good, "k");
    stale.revision = "old".into();
    assert!(assistant_revision(&text, &stale, unset).is_err());
}

#[test]
fn the_assistant_is_switched_and_given_a_model_only_where_one_is_configured() {
    let with = format!(
        "{}\n[assistant]\nbase_url = 'https://openrouter.ai/api/v1'\ntoken_env = 'KEY'\nmodel = 'a/b'\n",
        file()
    );
    let view = describe(Path::new("/c.toml"), &with, &detected()).unwrap();
    assert_eq!(
        view.assistant,
        Some(Assistant {
            enabled: true,
            model: "a/b".into()
        })
    );
    let mut request = change(&with);
    request.assistant = Some(Assistant {
        enabled: false,
        model: "c/d".into(),
    });
    let updated = revise(&with, &request, &detected()).unwrap();
    let view = describe(Path::new("/c.toml"), &updated, &detected()).unwrap();
    assert_eq!(view.assistant, request.assistant);
    // The service and the key's name are kept as written.
    assert!(updated.contains("openrouter.ai") && updated.contains("KEY"));
    // A change that leaves the assistant out leaves it as it is.
    let kept = revise(&updated, &change(&updated), &detected()).unwrap();
    let view = describe(Path::new("/c.toml"), &kept, &detected()).unwrap();
    assert_eq!(view.assistant, request.assistant);
    // An empty model name is refused, and so is an assistant the file lacks.
    let mut unnamed = change(&with);
    unnamed.assistant = Some(Assistant {
        enabled: true,
        model: " ".into(),
    });
    assert!(revise(&with, &unnamed, &detected()).is_err());
    let file = &file();
    assert!(
        describe(Path::new("/c.toml"), file, &detected())
            .unwrap()
            .assistant
            .is_none()
    );
    let mut absent = change(file);
    absent.assistant = request.assistant.clone();
    assert!(revise(file, &absent, &detected()).is_err());
}

#[test]
fn a_change_edits_only_what_it_names_and_keeps_the_rest_of_the_file() {
    let file = &file();
    let mut request = change(file);
    request.agents = BTreeMap::from([(Provider::ClaudeCode, true)]);
    request.since_days = Some(30);
    request.memory_capture = false;
    request.hide_credentials.publication = false;
    let updated = revise(file, &request, &detected()).unwrap();
    let view = describe(Path::new("/c.toml"), &updated, &detected()).unwrap();
    assert!(view.agents[&Provider::Codex].indexed);
    assert!(view.agents[&Provider::ClaudeCode].indexed);
    assert_eq!(view.agents[&Provider::ClaudeCode].roots, [home("claude")]);
    assert_eq!(view.since_days, Some(30));
    assert!(!view.memory_capture);
    assert!(view.hide_credentials.output && !view.hide_credentials.publication);
    // A setting the page does not offer survives the rewrite.
    assert!(updated.contains("limit = 7"));
}

#[test]
fn an_agent_with_no_history_here_is_offered_nothing_and_cannot_be_turned_on() {
    let file = &file();
    let view = describe(Path::new("/c.toml"), file, &detected()).unwrap();
    assert!(!view.agents.contains_key(&Provider::Cursor));
    assert!(!view.agents[&Provider::ClaudeCode].indexed);
    let mut request = change(file);
    request.agents = BTreeMap::from([(Provider::Cursor, true)]);
    let error = revise(file, &request, &detected()).unwrap_err();
    assert!(error.error.message.contains("no cursor history"));
}

#[test]
fn stale_or_unusable_changes_leave_the_file_alone() {
    let file = &file();
    let mut stale = change(file);
    stale.revision = revision("something else");
    assert!(
        revise(file, &stale, &detected())
            .unwrap_err()
            .error
            .message
            .contains("changed since")
    );
    let mut none = change(file);
    none.agents = BTreeMap::from([(Provider::Codex, false)]);
    assert!(
        revise(file, &none, &detected())
            .unwrap_err()
            .error
            .message
            .contains("at least one")
    );
    // History cannot be sent to a shared server that is not configured.
    let mut both = change(file);
    both.destination = IndexDestination::Both;
    assert!(revise(file, &both, &detected()).is_err());
    let mut days = change(file);
    days.since_days = Some(0);
    assert!(revise(file, &days, &detected()).is_err());
}

#[test]
fn applying_keeps_the_previous_file_beside_the_new_one() {
    let file = &file();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config.toml");
    std::fs::write(&path, file).unwrap();
    let mut request = change(file);
    request.since_days = Some(7);
    // Detection reads this machine; the change names no agent, so it is unused.
    let view = apply(&path, &request).unwrap();
    assert_eq!(view.since_days, Some(7));
    assert_ne!(view.revision, revision(file));
    assert_eq!(
        std::fs::read_to_string(root.path().join("config.toml.previous")).unwrap(),
        *file
    );
}

#[test]
fn unshared_agents_are_saved_in_the_shared_table_and_kept_when_a_change_leaves_them_out() {
    let joined = format!(
        "{}\n[shared]\ngroup_id = 'g'\nnode_id = 'n'\napi_url = 'https://example.com'\ntoken_env = 'TOKEN'\n",
        file()
    );
    let mut request = change(&joined);
    request.destination = IndexDestination::Both;
    request.unshared = Some(vec![Provider::Cursor, Provider::Copilot, Provider::Cursor]);
    let updated = revise(&joined, &request, &detected()).unwrap();
    let shared = parse_document(updated.as_bytes()).unwrap().shared.unwrap();
    assert_eq!(shared.unshared, vec![Provider::Copilot, Provider::Cursor]);
    // A change that does not mention the list keeps it.
    let mut later = change(&updated);
    later.destination = IndexDestination::Both;
    later.since_days = Some(30);
    let kept = revise(&updated, &later, &detected()).unwrap();
    assert_eq!(
        parse_document(kept.as_bytes())
            .unwrap()
            .shared
            .unwrap()
            .unshared,
        vec![Provider::Copilot, Provider::Cursor]
    );
    // Without a shared server there is nothing to keep agents off.
    let mut unjoined = change(&file());
    unjoined.unshared = Some(vec![Provider::Cursor]);
    assert!(
        revise(&file(), &unjoined, &detected())
            .unwrap_err()
            .error
            .message
            .contains("join a shared index")
    );
}
