//! A lesson line as a body writes it when it answers the human.
//!
//! The wire reads a tag only where it opens a line, and between bodies that is right: a report is
//! written for a parser. An answer to the human is markdown, and a body sets its lessons down
//! there the way it sets down any list: `- @U law …`, `` `@U colony …` ``, `**@U law path** …`.
//! Read as the wire reads, each of those is prose, and what the session learned is lost with no
//! word said. Counted over 284 kept runs of two bodies: one lost 80 of its 440 lessons this way,
//! the other 97 of 108.

use mikata_wire::UnsaidKind;

/// What markdown puts before a line's first word: a list mark, a heading, a quote.
const MARKS: [char; 6] = ['-', '*', '+', '•', '#', '>'];
/// What it wraps a word or a whole line in.
const WRAPS: [&str; 2] = ["**", "`"];
/// The tag of a lesson line.
const TAG: &str = "@U";

/// `text` with each lesson line bare, as the wire reads one: the marks before its `@U` dropped,
/// the emphasis or code span that opened on it closed, and the colon a body sets after the kind
/// as after a label (`@U colony: …`) gone. Only a line whose first word behind those marks is
/// `@U` is touched, and behind a mark only one that names its kind: a heading that reads
/// `**@U lines**` is a heading. Every other line stays as written, so no other tag is read from
/// inside a list, and a status can never come from one.
pub(crate) fn bare(text: &str) -> String {
    text.lines()
        .map(|line| lesson(line).unwrap_or_else(|| line.to_owned()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The line as a bare lesson, when it is one that the wire would not read as written; `None`
/// for any other line, a lesson the wire reads already among them.
fn lesson(line: &str) -> Option<String> {
    let mut rest = unmarked(line);
    let mut open = Vec::new();
    while let Some(wrap) = WRAPS.iter().find(|w| rest.starts_with(**w)) {
        rest = &rest[wrap.len()..];
        open.push(*wrap);
    }
    let marked = rest.len() < line.trim_start().len();
    let said = rest
        .strip_prefix(TAG)
        .filter(|after| after.starts_with(char::is_whitespace))?;
    let mut said = said.trim().to_owned();
    // Each wrap closes where it is next seen, the innermost first: around the tag and its path
    // (`**@U law a.py** the fact`), around the whole line, or never. A colon set after it, as
    // after a label, goes with it.
    for wrap in open.iter().rev() {
        if let Some(at) = said.find(wrap) {
            let after = &said[at + wrap.len()..];
            let after = after.strip_prefix(':').unwrap_or(after);
            said = format!("{}{after}", &said[..at]);
        }
    }
    let (word, fact) = said
        .split_once(char::is_whitespace)
        .unwrap_or((said.as_str(), ""));
    let kind = word.strip_suffix(':').unwrap_or(word);
    let labelled = kind.len() < word.len();
    // Behind a mark, a lesson names its kind. Bare, it is the wire's to read unless its kind
    // carries a colon, which the wire would take for part of the fact.
    if UnsaidKind::from_word(kind).is_none() || !(marked || labelled) {
        return None;
    }
    Some(
        format!("{TAG} {kind} {}", fact.trim_start())
            .trim_end()
            .to_owned(),
    )
}

/// The line behind its list marks, numbers, heading marks and quote marks, however they nest.
fn unmarked(line: &str) -> &str {
    let mut rest = line.trim_start();
    loop {
        let digits = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        let after = if digits.len() < rest.len() {
            digits.strip_prefix(['.', ')'])
        } else {
            rest.strip_prefix(MARKS).map(|a| a.trim_start_matches('#'))
        };
        match after.filter(|a| a.starts_with(char::is_whitespace)) {
            Some(a) => rest = a.trim_start(),
            None => return rest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::bare;
    use mikata_wire::{Report, UnsaidKind};

    /// What the wire keeps of an answer: each lesson's kind and text.
    fn kept(answer: &str) -> Vec<(Option<UnsaidKind>, String)> {
        Report::parse(&format!("@S DONE\n{}", bare(answer)))
            .unwrap()
            .unsaid
            .into_iter()
            .map(|u| (u.kind, u.text))
            .collect()
    }

    /// Every form below is a line a body wrote in a kept run, cut short.
    #[test]
    fn a_lesson_behind_a_list_mark_a_code_span_or_emphasis_is_kept() {
        let law = Some(UnsaidKind::Law);
        let colony = Some(UnsaidKind::Colony);
        for (written, kind, text) in [
            (
                "- @U law sqlglot/generators/hive.py datatype_sql must copy `collate`",
                law,
                "sqlglot/generators/hive.py datatype_sql must copy `collate`",
            ),
            (
                "- @U colony `pytz` and `duckdb` are not installed here",
                colony,
                "`pytz` and `duckdb` are not installed here",
            ),
            (
                "`@U law sqlglot/parser.py ALTER_DROP_COLUMN_CSV check=1912`",
                law,
                "sqlglot/parser.py ALTER_DROP_COLUMN_CSV check=1912",
            ),
            (
                "- `@U law sqlglot/parser.py` - `_parse_partition_by` checks the text",
                law,
                "sqlglot/parser.py - `_parse_partition_by` checks the text",
            ),
            (
                "**@U law sqlglot/generators/presto.py** Check that `_array_slice_to_slice` converts",
                law,
                "sqlglot/generators/presto.py Check that `_array_slice_to_slice` converts",
            ),
            (
                "**`@U colony`**: The repository is worked by running `python3 -m unittest`",
                colony,
                "The repository is worked by running `python3 -m unittest`",
            ),
            (
                "## @U colony dialects/snowflake.py ALTER_TABLE_ADD_REQUIRED_FOR_EACH_COLUMN = False",
                colony,
                "dialects/snowflake.py ALTER_TABLE_ADD_REQUIRED_FOR_EACH_COLUMN = False",
            ),
            (
                "  1. @U colony run the tests with `npm run test:src`",
                colony,
                "run the tests with `npm run test:src`",
            ),
            (
                "> * @U law a.py pop on an empty stack raises",
                law,
                "a.py pop on an empty stack raises",
            ),
            (
                "- @U colony: in this environment `unittest discover` has 4 known errors",
                colony,
                "in this environment `unittest discover` has 4 known errors",
            ),
            (
                "@U colony: a bare lesson whose kind carries a colon",
                colony,
                "a bare lesson whose kind carries a colon",
            ),
        ] {
            assert_eq!(kept(written), [(kind, text.to_owned())], "{written}");
        }
    }

    #[test]
    fn a_whole_answer_keeps_each_lesson_once_and_its_prose_as_written() {
        let answer = "Done: the collation survives.\n\n**Lessons**\n\n\
                      - @U law a.py the first fact\n- @U colony the second fact\n\
                      @U colony a bare one, as it always read\n\n- a plain item\n";
        let bare = bare(answer);
        assert_eq!(kept(answer).len(), 3, "{bare}");
        for prose in [
            "Done: the collation survives.",
            "**Lessons**",
            "- a plain item",
        ] {
            assert!(bare.lines().any(|l| l == prose), "{prose}: {bare}");
        }
    }

    /// The reader is for lessons alone: a list never yields a status, a verdict or an order, and
    /// a sentence that speaks of the tag is a sentence.
    #[test]
    fn nothing_but_a_lesson_is_read_from_behind_a_mark() {
        for written in [
            "- @S FAIL the build is red",
            "- @V accept n1",
            "`@$ slime-a: do it`",
            "- the `@U law` lines above are what I learned",
            "- @Ulaw a.py no space, no tag",
            "**@U lines**",
            "- @U lines I wrote above",
            "## @U",
            "@U a bare lesson of no kind is the wire's to read",
            "-@U law a.py a dash is not a list mark without its space",
            "@U law a.py a bare lesson is left alone",
            "plain prose",
            "",
        ] {
            assert_eq!(bare(written), written);
        }
        let report = Report::parse(&format!("@S DONE\n{}", bare("- @S FAIL\n- @V accept n1")));
        let report = report.unwrap();
        assert_eq!(report.status, "DONE");
        assert!(report.verdicts.is_empty(), "{report:?}");
    }
}
