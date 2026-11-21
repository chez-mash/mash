//! Mash's own user guides, built into the binary so the assistant can answer
//! questions about Mash from them. Nothing is read from disk or the network
//! when a question is asked.

use std::fmt::Write as _;

/// The guides, each with the page it is published at on the documentation
/// site.
const GUIDES: [(&str, &str, &str); 21] = [
    (
        "quick-start",
        "Quick start",
        include_str!("../../../../README.md"),
    ),
    (
        "installation",
        "Installation",
        include_str!("../../../../docs/installation.md"),
    ),
    (
        "search",
        "Search",
        include_str!("../../../../docs/search.md"),
    ),
    (
        "find-missing-chat",
        "Find a missing chat",
        include_str!("../../../../docs/find-missing-chat.md"),
    ),
    (
        "indexing",
        "Indexing",
        include_str!("../../../../docs/indexing.md"),
    ),
    (
        "agents",
        "Agents",
        include_str!("../../../../docs/agents.md"),
    ),
    (
        "plugin",
        "Plugin",
        include_str!("../../../../docs/plugin.md"),
    ),
    (
        "records",
        "Memory records",
        include_str!("../../../../docs/records.md"),
    ),
    ("mcp", "MCP server", include_str!("../../../../docs/mcp.md")),
    (
        "dashboard",
        "Dashboard",
        include_str!("../../../../docs/dashboard.md"),
    ),
    (
        "mashy",
        "Ask Mashy",
        include_str!("../../../../docs/mashy.md"),
    ),
    (
        "dashboard-access",
        "Open the dashboard on another device",
        include_str!("../../../../docs/dashboard-access.md"),
    ),
    (
        "services",
        "Services",
        include_str!("../../../../docs/services.md"),
    ),
    (
        "shared-index",
        "Set up a shared index",
        include_str!("../../../../docs/shared-index.md"),
    ),
    (
        "search-across-machines",
        "Search across your machines",
        include_str!("../../../../docs/search-across-machines.md"),
    ),
    (
        "synchronization",
        "Synchronization",
        include_str!("../../../../docs/synchronization.md"),
    ),
    (
        "configuration",
        "Configuration",
        include_str!("../../../../docs/configuration.md"),
    ),
    (
        "upgrades",
        "Upgrades and backups",
        include_str!("../../../../docs/upgrades.md"),
    ),
    (
        "privacy",
        "Privacy",
        include_str!("../../../../docs/privacy.md"),
    ),
    (
        "troubleshooting",
        "Troubleshooting",
        include_str!("../../../../docs/troubleshooting.md"),
    ),
    (
        "cli",
        "Command line",
        include_str!("../../../../docs/cli.md"),
    ),
];

/// Sections one lookup returns, and how much of each.
const SHOWN: usize = 4;
/// How much of one guide is shown when it is asked for by name.
const WHOLE: usize = 6_500;
const SECTION: usize = 1_600;

struct Section {
    page: &'static str,
    title: &'static str,
    heading: &'static str,
    body: &'static str,
}

// Words every question about Mash shares say nothing about which guide.
const COMMON: [&str; 22] = [
    "mash", "the", "and", "how", "what", "does", "for", "with", "from", "that", "this", "command",
    "commands", "guide", "guides", "can", "are", "not", "use", "using", "when", "which",
];

/// Everyday words for what the guides call something else.
const SAID_AS: [(&str, &str); 14] = [
    ("password", "credential"),
    ("secret", "credential"),
    ("token", "credential"),
    ("key", "credential"),
    ("delete", "forget"),
    ("remove", "forget"),
    ("erase", "forget"),
    ("laptop", "machine"),
    ("computer", "machine"),
    ("device", "machine"),
    ("backup", "export"),
    ("restore", "import"),
    ("uninstall", "uninstall"),
    ("memory", "record"),
];

/// Each guide cut at its second-level headings; what precedes the first is
/// the page's introduction.
fn sections() -> Vec<Section> {
    let mut all = Vec::new();
    for (page, title, text) in GUIDES {
        let mut heading = "Introduction";
        let mut start = 0;
        let mut offset = 0;
        let mut fenced = false;
        for line in text.split_inclusive('\n') {
            if line.starts_with("```") {
                fenced = !fenced;
            }
            if !fenced && line.starts_with("## ") {
                all.push(Section {
                    page,
                    title,
                    heading,
                    body: &text[start..offset],
                });
                heading = line[3..].trim();
                start = offset + line.len();
            }
            offset += line.len();
        }
        all.push(Section {
            page,
            title,
            heading,
            body: &text[start..],
        });
    }
    all.retain(|section| !section.body.trim().is_empty());
    all
}

fn clip(text: &str, limit: usize) -> &str {
    match text.char_indices().nth(limit) {
        Some((end, _)) => &text[..end],
        None => text,
    }
}

/// The sections that match, best first.
fn ranked<'a>(about: &str, words: &[String], all: &'a [Section]) -> Vec<(usize, &'a Section)> {
    let asked = about.to_lowercase();
    let spoken: Vec<&str> = asked.split_whitespace().collect();
    let commands: Vec<String> = spoken
        .windows(2)
        .filter(|pair| pair[0].trim_matches('`') == "mash")
        .map(|pair| {
            format!(
                "mash {}",
                pair[1].trim_matches(|character: char| !character.is_alphanumeric())
            )
        })
        .filter(|command| command.len() > 6)
        .collect();
    // A section matching more of the words outranks one that repeats a
    // single word; a word in the heading counts for more than one in the text.
    let mut scored: Vec<(usize, &Section)> = all
        .iter()
        .filter_map(|section| {
            let heading = format!("{} {}", section.title, section.heading).to_lowercase();
            let body = section.body.to_lowercase();
            let mut matched = 0;
            let mut score = 0;
            for word in words {
                // A plural or a verb ending still finds its word.
                let stem = word.trim_end_matches("ing").trim_end_matches('s');
                let stem = if stem.len() >= 4 { stem } else { word.as_str() };
                let in_heading = heading.matches(stem).count();
                let in_body = body.matches(stem).count();
                if in_heading + in_body > 0 {
                    matched += 1;
                    score += 8 * in_heading + in_body.min(6);
                }
            }
            // A command named in the question, such as `mash export`, points
            // straight at the sections that show it.
            let named = commands
                .iter()
                .filter(|command| body.contains(command.as_str()))
                .count();
            (matched > 0 || named > 0).then_some((score + 20 * matched + 60 * named, section))
        })
        .collect();
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    scored
}

/// The guide sections that best match `about`, or the list of pages and
/// their sections when nothing is asked for.
pub(super) fn look_up(about: &str, wanted: &str) -> String {
    let all = sections();
    // A guide asked for by name is read from its start.
    let wanted = wanted.trim().to_lowercase();
    if let Some((name, title, text)) = GUIDES.iter().find(|(name, _, _)| *name == wanted) {
        return format!(
            "From Mash's own guides: {title} (page {name}), from its start.\n\n{}{}",
            clip(text, WHOLE),
            if text.chars().count() > WHOLE {
                "\n(The guide goes on; ask with a few words for a later part.)"
            } else {
                ""
            }
        );
    }
    let words: Vec<String> = about
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric() && character != '-')
        .filter(|word| word.len() > 2 && !COMMON.contains(word))
        .map(|word| {
            // People ask in their own words; the guides have one for each thing.
            let plain = word.trim_end_matches('s');
            SAID_AS
                .iter()
                .find(|(said, _)| *said == plain)
                .map_or(word, |(_, written)| written)
                .to_owned()
        })
        .collect();
    let mut shown = String::new();
    if words.is_empty() {
        shown.push_str(
            "Mash's guides and their sections. Ask again with a few words to read one.\n",
        );
        for (page, title, _) in GUIDES {
            let headings: Vec<&str> = all
                .iter()
                .filter(|section| section.page == page && section.heading != "Introduction")
                .map(|section| section.heading)
                .collect();
            let _ = writeln!(shown, "{title} (page {page}): {}", headings.join("; "));
        }
        return shown;
    }
    let scored = ranked(about, &words, &all);
    if scored.is_empty() {
        return format!(
            "Mash's guides say nothing that matches {about:?}. Try other words, or read a whole guide with page set to one of: {}. Do not guess.",
            pages()
        );
    }
    shown.push_str(
        "From Mash's own guides. Answer from these; they describe this version of Mash.\n",
    );
    for (_, section) in scored.iter().take(SHOWN) {
        let body = section.body.trim();
        let _ = writeln!(
            shown,
            "\n--- {} (page {}), section \"{}\" ---\n{}{}",
            section.title,
            section.page,
            section.heading,
            clip(body, SECTION),
            if body.chars().count() > SECTION {
                "\n(The section goes on.)"
            } else {
                ""
            }
        );
    }
    let _ = writeln!(
        shown,
        "\nIf this is not it, read a whole guide with page set to one of: {}.",
        pages()
    );
    if scored.len() > SHOWN {
        let rest: Vec<String> = scored
            .iter()
            .skip(SHOWN)
            .take(6)
            .map(|(_, section)| format!("{}: {}", section.title, section.heading))
            .collect();
        let _ = writeln!(shown, "\nAlso about this: {}.", rest.join("; "));
    }
    shown
}

fn pages() -> String {
    GUIDES.map(|(name, _, _)| name).join(", ")
}

/// Whether a page name is one of the guides, so a link to it is real.
pub(super) fn page(name: &str) -> bool {
    GUIDES.iter().any(|(page, _, _)| *page == name)
}

#[cfg(test)]
mod tests {
    use super::{look_up, page, sections};

    #[test]
    fn the_guides_are_found_by_what_they_are_about() {
        assert!(sections().len() > 40);
        let found = look_up("install qdrant", "");
        assert!(found.contains("mash qdrant install"), "{found}");
        // What matches best comes first, and its page is named for a link.
        assert!(look_up("export a transcript", "").contains("(page dashboard)"));
        assert!(look_up("zzqqxx", "").contains("say nothing that matches"));
        // Found by testers: questions whose words are spread over a section.
        let export = look_up("mash export command save include", "");
        assert!(
            export.contains("mash export \"/absolute/path/to/new-backup\""),
            "{export}"
        );
        // A guide can be read from its start when the words of a question miss it.
        assert!(look_up("", "privacy").contains("## What stays on your machine"));
        assert!(look_up("text enclosed in private", "").contains("<private>"));
        assert!(look_up("zzqqxx", "").contains("shared-index"));
        assert!(look_up("", "").contains("Troubleshooting (page troubleshooting)"));
        assert!(page("search") && !page("made-up"));
    }

    #[test]
    fn one_lookup_stays_bounded() {
        assert!(
            look_up("mash index search dashboard chats", "")
                .chars()
                .count()
                < 8_500
        );
    }

    #[test]
    fn dashboard_task_guides_are_available() {
        assert!(page("mashy") && page("dashboard-access"));
        assert!(look_up("", "mashy").contains("API key"));
        assert!(look_up("", "dashboard-access").contains("mash dash remote"));
        assert!(look_up("open the dashboard on my phone", "").contains("(page dashboard-access)"));
        assert!(page("find-missing-chat"));
        assert!(look_up("find a missing chat", "").contains("(page find-missing-chat)"));
    }

    #[test]
    fn shared_use_and_setup_guides_are_available() {
        assert!(page("search-across-machines") && page("shared-index"));
        assert!(look_up("", "search-across-machines").contains("--shared"));
        assert!(look_up("", "search-across-machines").contains("--local"));
        assert!(look_up("", "shared-index").contains("mash-server --setup --add-node"));
    }
}
