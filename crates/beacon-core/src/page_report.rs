//! The Page report: what the engine could not use from the page a tab shows.
//!
//! The engine sends one [`PageDiagnostics`] per navigation, with its first frame: the HTML
//! parser's errors and the CSS the page's own stylesheets said that was dropped. This keeps the
//! latest one per tab and turns it into rows any frontend can draw, so every shell gets the same
//! wording.

use crate::tab::TabId;
use gosub_engine::diagnostics::{DroppedCssKind, PageDiagnostics};
use std::collections::HashMap;
use std::sync::Mutex;

/// A tab's report, or the fact that its page has not reported yet.
#[derive(Debug, Clone, PartialEq)]
pub enum Report {
    /// A navigation is under way and has not drawn its first frame.
    Waiting,
    Ready(PageDiagnostics),
}

static REPORTS: Mutex<Option<HashMap<TabId, Report>>> = Mutex::new(None);

fn with_reports<T>(f: impl FnOnce(&mut HashMap<TabId, Report>) -> T) -> Option<T> {
    let mut reports = REPORTS.lock().ok()?;
    Some(f(reports.get_or_insert_with(HashMap::new)))
}

/// A navigation started in `tab`: the report it had was for the page it is leaving.
pub fn navigation_started(tab: TabId) {
    with_reports(|reports| reports.insert(tab, Report::Waiting));
}

/// Keep `diagnostics` as `tab`'s report.
pub fn record(tab: TabId, diagnostics: PageDiagnostics) {
    with_reports(|reports| reports.insert(tab, Report::Ready(diagnostics)));
}

/// Forget a closed tab's report.
pub fn forget(tab: TabId) {
    with_reports(|reports| reports.remove(&tab));
}

/// `tab`'s report; `None` before its first navigation.
pub fn report(tab: TabId) -> Option<Report> {
    with_reports(|reports| reports.get(&tab).cloned()).flatten()
}

/// Which part of the report a row belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Html,
    Css,
}

impl Section {
    pub fn label(self) -> &'static str {
        match self {
            Section::Html => "HTML",
            Section::Css => "CSS",
        }
    }
}

/// One line of the report as a table shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub section: Section,
    /// What kind of problem, in a few words.
    pub kind: &'static str,
    /// What it was: a property, an at-rule, the parser's error name.
    pub subject: String,
    /// Where: `line:column` in the document, or `stylesheet:line` for CSS (the first place,
    /// when it happened more than once).
    pub location: String,
    /// How many times. HTML errors are listed one by one; CSS is grouped by what was dropped.
    pub count: usize,
    /// A sentence on what the row means and what to do about it.
    pub hint: &'static str,
}

/// The short name of a stylesheet URL: its last path segment without the query, or the whole
/// thing when it has none. A `<style>` element's sheet is the document's URL with `#inline`.
pub fn sheet_name(url: &str) -> String {
    if url.ends_with("#inline") {
        return "<style>".to_string();
    }
    let path = url.split(['?', '#']).next().unwrap_or(url);
    match path.trim_end_matches('/').rsplit('/').next() {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => url.to_string(),
    }
}

fn css_kind(kind: DroppedCssKind) -> (&'static str, &'static str) {
    match kind {
        DroppedCssKind::UnknownProperty => (
            "Unknown property",
            "The engine has no such property: a misspelling, a vendor prefix, or a property it does not support yet. The declaration is ignored.",
        ),
        DroppedCssKind::InvalidValue => (
            "Rejected value",
            "The property is known but this value is not valid for it, or the engine cannot read it yet. The declaration is ignored, so an earlier one for the same property applies.",
        ),
        DroppedCssKind::InvalidDeclaration => (
            "Unreadable declaration",
            "The parser could not read this declaration at all and skipped to the next one.",
        ),
        DroppedCssKind::InvalidSelector => (
            "Unsupported selector",
            "The rule's selector is invalid or uses a selector the engine does not support, so the whole rule is ignored.",
        ),
        DroppedCssKind::UnsupportedAtRule => (
            "Unsupported at-rule",
            "The engine does not implement this at-rule, or it is somewhere it is not allowed. Everything inside it is ignored.",
        ),
        _ => ("Dropped", "The engine did not apply this."),
    }
}

const HTML_HINT: &str = "An error the HTML parser reported. Browsers recover from these the way the spec says, but the page may not be built the way its author meant.";

/// The report as rows: the HTML errors in order, then the dropped CSS grouped by kind, property
/// and stylesheet, most frequent first.
pub fn rows(diagnostics: &PageDiagnostics) -> Vec<Row> {
    let mut out: Vec<Row> = diagnostics
        .html_errors
        .iter()
        .map(|error| Row {
            section: Section::Html,
            kind: "Parse error",
            subject: error.message.clone(),
            location: format!("line {}:{}", error.line, error.column),
            count: 1,
            hint: HTML_HINT,
        })
        .collect();

    let mut css: Vec<(Row, String)> = Vec::new();
    for dropped in &diagnostics.css_dropped {
        let (kind, hint) = css_kind(dropped.kind);
        match css
            .iter_mut()
            .find(|(row, sheet)| row.kind == kind && row.subject == dropped.subject && *sheet == dropped.stylesheet)
        {
            Some((row, _)) => row.count += 1,
            None => css.push((
                Row {
                    section: Section::Css,
                    kind,
                    subject: dropped.subject.clone(),
                    location: format!("{}:{}", sheet_name(&dropped.stylesheet), dropped.line),
                    count: 1,
                    hint,
                },
                dropped.stylesheet.clone(),
            )),
        }
    }
    // Stable, so rows that happened equally often keep the order the page met them in.
    css.sort_by_key(|(row, _)| std::cmp::Reverse(row.count));
    out.extend(css.into_iter().map(|(row, _)| row));
    out
}

/// One line saying what the report holds, for a summary bar.
pub fn summary(diagnostics: &PageDiagnostics) -> String {
    let plural = |n: usize, one: &str, many: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {many}") };
    let mut text = format!(
        "{} · {}",
        plural(diagnostics.html_error_count, "HTML parse error", "HTML parse errors"),
        plural(diagnostics.css_dropped_count, "dropped CSS item", "dropped CSS items"),
    );
    let shown = diagnostics.html_errors.len() + diagnostics.css_dropped.len();
    let total = diagnostics.html_error_count + diagnostics.css_dropped_count;
    if shown < total {
        text.push_str(&format!(" (first {shown} listed)"));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use gosub_engine::diagnostics::{DroppedCss, HtmlParseError};

    fn dropped(kind: DroppedCssKind, subject: &str, sheet: &str, line: u32) -> DroppedCss {
        DroppedCss {
            stylesheet: sheet.to_string(),
            kind,
            subject: subject.to_string(),
            line,
        }
    }

    fn sample() -> PageDiagnostics {
        PageDiagnostics {
            html_errors: vec![HtmlParseError {
                message: "expected-doctype-but-got-start-tag".into(),
                line: 1,
                column: 1,
            }],
            html_error_count: 1,
            css_dropped: vec![
                dropped(DroppedCssKind::UnsupportedAtRule, "@keyframes", "https://a.test/site.css?v=3", 40),
                dropped(
                    DroppedCssKind::UnknownProperty,
                    "-webkit-transform",
                    "https://a.test/site.css?v=3",
                    7,
                ),
                dropped(
                    DroppedCssKind::UnknownProperty,
                    "-webkit-transform",
                    "https://a.test/site.css?v=3",
                    19,
                ),
                dropped(
                    DroppedCssKind::UnknownProperty,
                    "-webkit-transform",
                    "https://a.test/page#inline",
                    2,
                ),
            ],
            css_dropped_count: 4,
        }
    }

    /// HTML errors one by one, then CSS grouped per stylesheet, the most frequent first, each
    /// group at the first place it happened.
    #[test]
    fn rows_list_html_then_grouped_css() {
        let rows = rows(&sample());
        let shown: Vec<(&str, &str, &str, usize)> = rows
            .iter()
            .map(|row| (row.kind, row.subject.as_str(), row.location.as_str(), row.count))
            .collect();
        assert_eq!(
            shown,
            vec![
                ("Parse error", "expected-doctype-but-got-start-tag", "line 1:1", 1),
                ("Unknown property", "-webkit-transform", "site.css:7", 2),
                ("Unsupported at-rule", "@keyframes", "site.css:40", 1),
                ("Unknown property", "-webkit-transform", "<style>:2", 1),
            ]
        );
    }

    #[test]
    fn the_summary_says_when_the_list_is_cut_short() {
        let mut report = sample();
        assert_eq!(summary(&report), "1 HTML parse error · 4 dropped CSS items");
        report.html_error_count = 900;
        assert_eq!(summary(&report), "900 HTML parse errors · 4 dropped CSS items (first 5 listed)");
    }

    #[test]
    fn a_new_navigation_waits_for_its_own_report() {
        let tab = TabId::default();
        assert_eq!(report(tab), None);
        record(tab, sample());
        assert!(matches!(report(tab), Some(Report::Ready(_))));
        navigation_started(tab);
        assert_eq!(report(tab), Some(Report::Waiting));
        forget(tab);
        assert_eq!(report(tab), None);
    }
}
