//! The human's word on an answer: that it was wrong, and taking that back. One act, whether it is
//! said at the core's door (`mikatad memory wrong`) or typed in a session (`/wrong`): what is
//! kept, where, and what the harness will do with it are decided here and nowhere else.
//!
//! Before anything is kept, [`preview`] says what it would be: the ask it is kept under, and the
//! line the harness would hold when that ask comes again, or that the words are advice. A person
//! does not always say what must come out, and only words that do are held.

use std::path::Path;

use mikata_world::log::{ASKED, Entry, append_entry, last_asked};
use mikata_world::memory;

/// What saying an answer was wrong would keep, before it is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// The words, trimmed.
    pub words: String,
    /// The ask they are kept under, by its first line.
    pub asked: String,
    /// The line the harness holds whenever that ask comes again; `None` when the words state no
    /// output and stay advice.
    pub held: Option<String>,
}

/// A correction, kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    /// Its id: what takes it back.
    pub id: String,
    /// The ask it is kept under.
    pub asked: String,
    /// The notes learned under that ask that it corrects.
    pub linked: Vec<String>,
    /// The line held, read from the words as memory keeps them; `None` for advice.
    pub held: Option<String>,
}

/// What the human would keep by saying `words` of the answer to `ask`, or to the journal's last
/// ask when none is named. Nothing is written.
///
/// # Errors
/// No words, or no ask to keep them under.
pub fn preview(world: &Path, words: &str, ask: Option<&str>) -> Result<Preview, String> {
    let words = words.trim();
    if words.is_empty() {
        return Err("say what is wrong, in your words: what still fails".to_owned());
    }
    let asked = ask
        .map(|a| a.trim().to_owned())
        .filter(|a| !a.is_empty())
        .or_else(|| last_asked(world))
        .ok_or_else(|| {
            "no entry of the journal says what it was asked, so there is no ask to keep this under"
                .to_owned()
        })?;
    Ok(Preview {
        held: crate::done::held_line(words),
        words: words.to_owned(),
        asked,
    })
}

/// Keep what `preview` showed: the words go to the journal, then to memory as a correction,
/// shared and grounded by the human's word, under the ask. Never a body's act: the caller says
/// who is speaking.
///
/// # Errors
/// The world keeps no journal, or the journal or memory cannot be written.
pub fn keep(world: &Path, said: &Preview) -> Result<Kept, String> {
    let entry = Entry {
        body: "the human".to_owned(),
        title: format!("wrong: {}", said.asked),
        lines: vec![
            (ASKED.to_owned(), said.asked.clone()),
            ("Said".to_owned(), said.words.clone()),
        ],
    };
    let heading = match append_entry(world, &entry) {
        Ok(Some(h)) => h,
        Ok(None) => {
            return Err("this world keeps no journal (.mikata/log.md) to write it in".to_owned());
        }
        Err(e) => return Err(format!("the journal cannot be written: {e}")),
    };
    let (id, linked) = memory::corrected(
        world,
        &said.words,
        &said.asked,
        &format!("log.md {heading}"),
    )
    .map_err(|e| format!("the correction could not be kept: {e}"))?;
    // Read from the words as memory keeps them: that is what a later session holds.
    let kept = memory::correction(world, &id)
        .ok()
        .flatten()
        .unwrap_or_else(|| said.words.clone());
    Ok(Kept {
        held: crate::done::held_line(&kept),
        id,
        asked: said.asked.clone(),
        linked,
    })
}

/// Take a correction back: written to the journal, and the correction sleeps, neither told nor
/// held when its ask comes again. Nothing is deleted. Returns the words it said.
///
/// # Errors
/// No such correction awake, or the journal or memory cannot be read or written.
pub fn take_back(world: &Path, id: &str) -> Result<String, String> {
    let said = match memory::correction(world, id) {
        Ok(Some(said)) => said,
        Ok(None) => return Err(format!("{id} is no correction of yours that is awake")),
        Err(e) => return Err(format!("memory cannot be read: {e}")),
    };
    let entry = Entry {
        body: "the human".to_owned(),
        title: format!("withdrawn: {id}"),
        lines: vec![("Said".to_owned(), said.clone())],
    };
    let heading = match append_entry(world, &entry) {
        Ok(Some(h)) => h,
        Ok(None) => {
            return Err("this world keeps no journal (.mikata/log.md) to write it in".to_owned());
        }
        Err(e) => return Err(format!("the journal cannot be written: {e}")),
    };
    memory::withdraw(world, id, &format!("log.md {heading}"))
        .map_err(|e| format!("the correction could not be withdrawn: {e}"))?;
    Ok(said)
}

/// Take back every note a unit wrote (`unit@version`): written to the journal, and each note
/// sleeps in every tier it reached, recalled no more. Nothing is deleted. Returns what was taken
/// back.
///
/// # Errors
/// The stamp does not read, no note the unit wrote is awake, or the journal or memory cannot be
/// read or written.
pub fn take_back_by(world: &Path, unit: &str) -> Result<Vec<memory::Written>, String> {
    let stamp: memory::UnitStamp = unit.parse()?;
    let awake =
        memory::written_by(world, &stamp).map_err(|e| format!("memory cannot be read: {e}"))?;
    if awake.is_empty() {
        return Err(format!("no note {stamp} wrote is awake"));
    }
    let mut ids: Vec<&str> = awake.iter().map(|w| w.note.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    let entry = Entry {
        body: "the human".to_owned(),
        title: format!("withdrawn: the notes of {stamp}"),
        lines: vec![("Notes".to_owned(), ids.join(", "))],
    };
    let heading = match append_entry(world, &entry) {
        Ok(Some(h)) => h,
        Ok(None) => {
            return Err("this world keeps no journal (.mikata/log.md) to write it in".to_owned());
        }
        Err(e) => return Err(format!("the journal cannot be written: {e}")),
    };
    memory::withdraw_by(world, &stamp, &format!("log.md {heading}"))
        .map_err(|e| format!("the notes could not be withdrawn: {e}"))
}

/// What a preview says to the human, in one line or two: held or advice, and under which ask.
#[must_use]
pub fn shown(p: &Preview) -> String {
    match &p.held {
        Some(line) => format!(
            "held: when \"{}\" is asked again, this is a line of its Done when list, and an \
             answer that leaves it wrong does not land: {line}",
            p.asked
        ),
        None => format!(
            "advice, for \"{}\": your words state no output, so they will be told and not held. \
             To have it held, say what must come out: `<input>` must give `<output>`",
            p.asked
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{keep, preview, shown, take_back};

    fn answered() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".mikata")).unwrap();
        std::fs::write(
            dir.path().join(".mikata/log.md"),
            "\n---\n\n### [2026-11-22T10:00:00Z] summoner — gate: PASS, unchecked\n\
             - **Asked:** Make total skip blanks\n- **Changed:** `total.py`\n",
        )
        .unwrap();
        dir
    }

    /// A preview writes nothing and says what would be held, or that the words are advice, and
    /// under which ask: the one named, else the journal's last.
    #[test]
    fn a_preview_says_held_or_advice_and_writes_nothing() {
        let dir = answered();
        let before = std::fs::read_to_string(dir.path().join(".mikata/log.md")).unwrap();
        let p = preview(
            dir.path(),
            "  `total 1 2` must give `6`; it gives `3` ",
            None,
        )
        .unwrap();
        assert_eq!(p.asked, "Make total skip blanks");
        assert_eq!(p.held.as_deref(), Some("`total 1 2` must give `6`"));
        assert!(shown(&p).starts_with("held: when \"Make total skip blanks\" is asked again"));
        let advice = preview(dir.path(), "it still adds the blanks", Some("Other ask")).unwrap();
        assert_eq!(
            (advice.asked.as_str(), advice.held.as_deref()),
            ("Other ask", None)
        );
        assert!(shown(&advice).contains("your words state no output"));
        assert!(preview(dir.path(), "   ", None).is_err());
        let empty = tempfile::tempdir().unwrap();
        assert!(preview(empty.path(), "x must give `1`", None).is_err());
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".mikata/log.md")).unwrap(),
            before,
            "nothing written"
        );
    }

    /// What is kept is what was shown, and taking it back names the words it said.
    #[test]
    fn what_is_kept_is_what_was_shown_and_can_be_taken_back() {
        let dir = answered();
        let p = preview(dir.path(), "`total 1 2` must give `6`; it gives `3`", None).unwrap();
        let kept = keep(dir.path(), &p).unwrap();
        assert_eq!(kept.asked, p.asked);
        assert_eq!(kept.held, p.held);
        let journal = std::fs::read_to_string(dir.path().join(".mikata/log.md")).unwrap();
        assert!(
            journal.contains("the human — wrong: Make total skip blanks"),
            "{journal}"
        );
        let said = take_back(dir.path(), &kept.id).unwrap();
        assert!(said.contains("must give `6`"), "{said}");
        assert!(take_back(dir.path(), &kept.id).is_err(), "taken back once");
    }

    /// Every note a unit wrote is taken back by its stamp, in the journal first; a stamp that does
    /// not read, or one whose notes are all asleep, is refused and writes nothing.
    #[test]
    fn a_unit_s_notes_are_taken_back_by_its_stamp() {
        use mikata_world::memory::{Status, Tier, UnitStamp, as_unit, fold, propose, read};
        let dir = answered();
        let stamp: UnitStamp = "plugin:noisy@0.2.0".parse().unwrap();
        let note = as_unit(&stamp, || {
            propose(dir.path(), "summoner", "skip the slow tests", vec![], None).unwrap()
        })
        .note;
        let other = propose(dir.path(), "summoner", "the lexer is in src/", vec![], None)
            .unwrap()
            .note;
        assert!(super::take_back_by(dir.path(), "plugin:noisy").is_err());
        let gone = super::take_back_by(dir.path(), "plugin:noisy@0.2.0").unwrap();
        assert_eq!(gone.len(), 1);
        assert_eq!(gone[0].note, note);
        let journal = std::fs::read_to_string(dir.path().join(".mikata/log.md")).unwrap();
        assert!(
            journal.contains("the human — withdrawn: the notes of plugin:noisy@0.2.0"),
            "{journal}"
        );
        let notes = fold(&read(dir.path(), &Tier::Personal("summoner".into())).unwrap().rows);
        assert_eq!(notes[&note].status, Status::Dormant);
        assert_eq!(notes[&other].status, Status::Proposed);
        let e = super::take_back_by(dir.path(), "plugin:noisy@0.2.0").unwrap_err();
        assert!(e.contains("no note plugin:noisy@0.2.0 wrote is awake"), "{e}");
        let after = std::fs::read_to_string(dir.path().join(".mikata/log.md")).unwrap();
        assert_eq!(after, journal, "a refusal writes nothing");
    }
}
