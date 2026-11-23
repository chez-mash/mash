//! Whether a Windows app opens links of a scheme, read from the registry the
//! way Windows itself decides: the scheme's class key carries a `URL Protocol`
//! value, and its `shell\open\command` key says what runs. An installer writes
//! a command line there; a packaged (MSIX) app writes a `DelegateExecute`
//! value instead, so any non-empty string value counts.
//!
//! The registry is read with `reg query`, so the answer is plain text that the
//! tests can supply without a registry.

/// The keys a scheme may be registered under: the user's own classes, then the
/// merged view that also holds machine-wide ones.
pub(super) fn scheme_keys(scheme: &str) -> [String; 2] {
    [
        format!(r"HKEY_CURRENT_USER\Software\Classes\{scheme}"),
        format!(r"HKEY_CLASSES_ROOT\{scheme}"),
    ]
}

/// Whether `reg query <key> /s` printed `output` for a registered protocol.
pub(super) fn registers_protocol(key: &str, output: &str) -> bool {
    let command = format!(r"{key}\shell\open\command");
    let mut protocol = false;
    let mut opens = false;
    let mut section = "";
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            section = line.trim_end();
            continue;
        }
        let Some((name, data)) = string_value(line) else {
            continue;
        };
        if section.eq_ignore_ascii_case(key) && name.eq_ignore_ascii_case("URL Protocol") {
            protocol = true;
        }
        if section.eq_ignore_ascii_case(&command) && !data.is_empty() {
            opens = true;
        }
    }
    protocol && opens
}

/// One string value as `reg query` prints it: its name, type and data,
/// separated by four spaces. Other types are not commands.
fn string_value(line: &str) -> Option<(&str, &str)> {
    let mut fields = line.trim_start().splitn(3, "    ");
    let name = fields.next()?;
    let kind = fields.next()?;
    let data = fields.next().unwrap_or("").trim();
    matches!(kind, "REG_SZ" | "REG_EXPAND_SZ").then_some((name, data))
}

/// Whether an app on this machine opens links of `scheme`.
#[cfg(windows)]
pub(super) fn registered(scheme: &str) -> bool {
    use std::os::windows::process::CommandExt;
    // No console window flashes up for the dashboard's own lookups.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    scheme_keys(scheme).iter().any(|key| {
        std::process::Command::new("reg")
            .args(["query", key, "/s"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .is_some_and(|output| registers_protocol(key, &String::from_utf8_lossy(&output.stdout)))
    })
}
