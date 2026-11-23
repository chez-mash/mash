//! Copies of Claude Code sessions that ran over SSH on another machine.
//!
//! The Claude desktop app keeps a local copy of every session it runs over
//! SSH under `projects/ssh-<sessionId>/`: the session's transcript
//! `<sessionId>.jsonl` and its subagents' `agent-<agentId>.jsonl` side by side.
//! The original lives on the remote machine, which may publish it to a shared
//! index under the same chat identity. Only that exact folder name, matching
//! the transcript's own session ID, marks a copy; a working folder that merely
//! looks remote never does.
use std::path::Path;

/// Whether a Claude Code transcript of `session` was read from the desktop
/// app's copy of a session that ran over SSH.
pub fn is_ssh_copy(transcript: &Path, session: &str) -> bool {
    transcript
        .parent()
        .and_then(Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .and_then(|folder| folder.strip_prefix("ssh-"))
        .is_some_and(|folder| !session.is_empty() && folder == session)
}

#[cfg(test)]
mod tests {
    use super::is_ssh_copy;
    use std::path::Path;

    #[test]
    fn only_the_folder_named_for_the_session_marks_a_copy() {
        let copy = Path::new("/home/me/.claude/projects/ssh-abc/abc.jsonl");
        assert!(is_ssh_copy(copy, "abc"));
        // Its subagents sit beside it.
        assert!(is_ssh_copy(Path::new("/p/ssh-abc/agent-a1.jsonl"), "abc"));
        // Another session's folder, an ordinary project, and look-alikes.
        assert!(!is_ssh_copy(copy, "other"));
        assert!(!is_ssh_copy(
            Path::new("/p/-home-me-ssh-abc/abc.jsonl"),
            "abc"
        ));
        assert!(!is_ssh_copy(Path::new("/p/-ssh-abc/abc.jsonl"), "abc"));
        assert!(!is_ssh_copy(Path::new("/p/ssh-abc/sub/abc.jsonl"), "abc"));
        assert!(!is_ssh_copy(Path::new("/p/ssh-/x.jsonl"), ""));
    }
}
