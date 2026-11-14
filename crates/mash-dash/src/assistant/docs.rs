//! Mash's own user guides, built into the binary so the assistant can answer
//! questions about Mash from them. Nothing is read from disk or the network
//! when a question is asked.

use std::fmt::Write as _;

/// The guides, each with the page it is published at on the documentation
/// site.
const GUIDES: [(&str, &str, &str); 17] = [
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
        "services",
        "Services",
        include_str!("../../../../docs/services.md"),
    ),
    (
        "shared-index",
        "Sharing across machines",
        include_str!("../../../../docs/shared-index.md"),
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
const SHOWN: usize = 3;
const SECTION: usize = 1_800;

struct Section {
    page: &'static str,
    title: &'static str,
    heading: &'static str,
    body: &'static str,
}

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

/// The guide sections that best match `about`, or the list of pages and
/// their sections when nothing is asked for.
pub(super) fn look_up(about: &str) -> String {
    let all = sections();
    let words: Vec<String> = about
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric() && character != '-')
        .filter(|word| word.len() > 2)
        .map(str::to_owned)
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
    let mut scored: Vec<(usize, &Section)> = all
        .iter()
        .filter_map(|section| {
            let heading = format!("{} {}", section.title, section.heading).to_lowercase();
            let body = section.body.to_lowercase();
            let score: usize = words
                .iter()
                .map(|word| {
                    8 * heading.matches(word.as_str()).count()
                        + body.matches(word.as_str()).count().min(6)
                })
                .sum();
            // Every word must appear somewhere when there are only a few.
            let all_present = words.len() > 3
                || words
                    .iter()
                    .all(|word| heading.contains(word.as_str()) || body.contains(word.as_str()));
            (score > 0 && all_present).then_some((score, section))
        })
        .collect();
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    if scored.is_empty() {
        return format!(
            "Mash's guides say nothing that matches {about:?}. Try other words, or call docs with no words to see what the guides cover. Do not guess."
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
        let found = look_up("install qdrant");
        assert!(found.contains("mash qdrant install"), "{found}");
        // What matches best comes first, and its page is named for a link.
        assert!(look_up("export a transcript").contains("(page dashboard)"));
        assert!(look_up("zzqqxx").contains("say nothing that matches"));
        assert!(look_up("").contains("Troubleshooting (page troubleshooting)"));
        assert!(page("search") && !page("made-up"));
    }

    #[test]
    fn one_lookup_stays_bounded() {
        assert!(look_up("mash index search dashboard chats").chars().count() < 7_500);
    }
}
