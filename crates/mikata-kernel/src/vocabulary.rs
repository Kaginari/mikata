//! The hook vocabulary: where a hook runs, what it is shown, what it may answer and what the
//! human grants. It is plain data, the same for native units and for plugins, so the plugin host
//! forwards it as JSON unchanged.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The longest note a unit may add, in characters; a longer one is cut to this.
pub const NOTE_CAP: usize = 2_000;

/// Where in the loop a hook runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Point {
    /// An ask opened, before its first turn.
    AskBegin,
    /// The ask's uncached preface is being written.
    Preface,
    /// Before each model call: the meters.
    TurnMeter,
    /// Before one tool call runs.
    ToolBefore,
    /// After one tool call ran.
    ToolAfter,
    /// The body answered; before the gate's word is final.
    AnswerLand,
    /// The verdict is in.
    AnswerLanded,
    /// The run is ending.
    SessionEnd,
}

impl Point {
    /// Every point, in the order the loop meets them in one ask.
    pub const ALL: [Point; 8] = [
        Point::AskBegin,
        Point::Preface,
        Point::TurnMeter,
        Point::ToolBefore,
        Point::ToolAfter,
        Point::AnswerLand,
        Point::AnswerLanded,
        Point::SessionEnd,
    ];

    /// The point's name as it is written in JSON, `tool_before` for [`Point::ToolBefore`].
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Point::AskBegin => "ask_begin",
            Point::Preface => "preface",
            Point::TurnMeter => "turn_meter",
            Point::ToolBefore => "tool_before",
            Point::ToolAfter => "tool_after",
            Point::AnswerLand => "answer_land",
            Point::AnswerLanded => "answer_landed",
            Point::SessionEnd => "session_end",
        }
    }
}

/// The gate's word on an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// The checks passed.
    Pass,
    /// The checks failed.
    Fail,
    /// Nothing checked the answer.
    Unverified,
}

/// What a hook is shown. Each event belongs to one [`Point`], named by its `point` tag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "point", rename_all = "snake_case")]
pub enum Event {
    /// An ask opened, before its first turn.
    AskBegin {
        /// The human's ask.
        ask: String,
    },
    /// The ask's uncached preface is being written.
    Preface {
        /// The human's ask.
        ask: String,
    },
    /// Before each model call: the meters.
    TurnMeter {
        /// Model calls made in this ask.
        steps: u32,
        /// The most model calls the ask may make.
        step_limit: u32,
        /// Weighted tokens spent so far.
        spent_wt: u64,
        /// Tokens the context holds now.
        context_used: u64,
        /// Tokens the context can hold.
        context_window: u64,
    },
    /// Before one tool call runs.
    ToolBefore {
        /// The call's id.
        id: String,
        /// The tool's name.
        name: String,
        /// The call's input, as the body wrote it.
        input: Value,
    },
    /// After one tool call ran.
    ToolAfter {
        /// The call's id.
        id: String,
        /// The tool's name.
        name: String,
        /// The call's input, as the body wrote it.
        input: Value,
        /// What the tool answered.
        content: String,
        /// Whether the call failed.
        is_error: bool,
        /// The paths the call wrote.
        wrote: Vec<String>,
    },
    /// The body answered; before the gate's word is final.
    AnswerLand {
        /// The body's answer.
        answer: String,
    },
    /// The verdict is in.
    AnswerLanded {
        /// The body's answer.
        answer: String,
        /// The gate's word.
        verdict: Verdict,
        /// The heading the answer was shown under, when it has one.
        heading: Option<String>,
    },
    /// The run is ending.
    SessionEnd {
        /// Why it ends.
        ending: String,
    },
}

impl Event {
    /// The point this event is shown at.
    #[must_use]
    pub fn point(&self) -> Point {
        match self {
            Event::AskBegin { .. } => Point::AskBegin,
            Event::Preface { .. } => Point::Preface,
            Event::TurnMeter { .. } => Point::TurnMeter,
            Event::ToolBefore { .. } => Point::ToolBefore,
            Event::ToolAfter { .. } => Point::ToolAfter,
            Event::AnswerLand { .. } => Point::AnswerLand,
            Event::AnswerLanded { .. } => Point::AnswerLanded,
            Event::SessionEnd { .. } => Point::SessionEnd,
        }
    }
}

/// What a hook may answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Reply {
    /// Nothing to add.
    Pass,
    /// Advice, said to the body as a context update.
    Note {
        /// What to say.
        text: String,
    },
    /// At [`Point::Preface`] only: a block in the preface's `plugins` slot.
    Text {
        /// The block.
        text: String,
    },
    /// At [`Point::ToolBefore`] the call is not run and is answered `why`; at
    /// [`Point::AnswerLand`] the answer is handed back with `why`.
    Veto {
        /// Why.
        why: String,
    },
    /// At [`Point::TurnMeter`] only: the ask ends with `why`.
    Stop {
        /// Why.
        why: String,
    },
}

impl Reply {
    /// The grant this reply needs, or `None` for [`Reply::Pass`], which needs none.
    #[must_use]
    pub fn needs(&self) -> Option<Grant> {
        match self {
            Reply::Pass => None,
            Reply::Note { .. } | Reply::Text { .. } => Some(Grant::Advise),
            Reply::Veto { .. } => Some(Grant::Veto),
            Reply::Stop { .. } => Some(Grant::Stop),
        }
    }

    /// Whether this reply counts at `point`: a note everywhere but at the session's end, text in
    /// the preface, a veto before a tool call and when an answer lands, a stop at the meters.
    #[must_use]
    pub fn counts_at(&self, point: Point) -> bool {
        match self {
            Reply::Pass => true,
            Reply::Note { .. } => point != Point::SessionEnd,
            Reply::Text { .. } => point == Point::Preface,
            Reply::Veto { .. } => matches!(point, Point::ToolBefore | Point::AnswerLand),
            Reply::Stop { .. } => point == Point::TurnMeter,
        }
    }

    /// Whether this reply ends the point: a veto or a stop.
    #[must_use]
    pub fn ends(&self) -> bool {
        matches!(self, Reply::Veto { .. } | Reply::Stop { .. })
    }

    /// The reply's kind as it is written in JSON, `veto` for [`Reply::Veto`].
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Reply::Pass => "pass",
            Reply::Note { .. } => "note",
            Reply::Text { .. } => "text",
            Reply::Veto { .. } => "veto",
            Reply::Stop { .. } => "stop",
        }
    }
}

/// What the human grants a unit. A reply above its grant is dropped and journaled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grant {
    /// Sees events; every reply is read as [`Reply::Pass`].
    Observe,
    /// May answer [`Reply::Note`] and [`Reply::Text`].
    Advise,
    /// May answer [`Reply::Veto`].
    Veto,
    /// May answer [`Reply::Stop`].
    Stop,
    /// A plugin's MCP tools pass through the four MCP doors; the frame does not change.
    Tools,
}

impl Grant {
    /// The grant's name as it is written in JSON and in the human's file, `veto` for
    /// [`Grant::Veto`].
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Grant::Observe => "observe",
            Grant::Advise => "advise",
            Grant::Veto => "veto",
            Grant::Stop => "stop",
            Grant::Tools => "tools",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn events_are_tagged_by_their_point_in_snake_case() {
        let event = Event::ToolBefore {
            id: "t1".into(),
            name: "bash".into(),
            input: json!({ "command": "ls" }),
        };
        let value = serde_json::to_value(&event).unwrap();
        assert_eq!(
            value,
            json!({ "point": "tool_before", "id": "t1", "name": "bash", "input": { "command": "ls" } })
        );
        assert_eq!(serde_json::from_value::<Event>(value).unwrap(), event);

        let landed = Event::AnswerLanded {
            answer: "done".into(),
            verdict: Verdict::Unverified,
            heading: None,
        };
        assert_eq!(
            serde_json::to_value(&landed).unwrap(),
            json!({ "point": "answer_landed", "answer": "done", "verdict": "unverified", "heading": null })
        );
    }

    #[test]
    fn every_event_names_the_point_its_tag_names() {
        let events = [
            Event::AskBegin { ask: String::new() },
            Event::Preface { ask: String::new() },
            Event::TurnMeter {
                steps: 1,
                step_limit: 2,
                spent_wt: 3,
                context_used: 4,
                context_window: 5,
            },
            Event::ToolBefore {
                id: String::new(),
                name: String::new(),
                input: Value::Null,
            },
            Event::ToolAfter {
                id: String::new(),
                name: String::new(),
                input: Value::Null,
                content: String::new(),
                is_error: false,
                wrote: vec!["a.rs".into()],
            },
            Event::AnswerLand {
                answer: String::new(),
            },
            Event::AnswerLanded {
                answer: String::new(),
                verdict: Verdict::Pass,
                heading: Some("h".into()),
            },
            Event::SessionEnd {
                ending: String::new(),
            },
        ];
        for (event, point) in events.iter().zip(Point::ALL) {
            assert_eq!(event.point(), point);
            let value = serde_json::to_value(event).unwrap();
            assert_eq!(value["point"], point.name());
            assert_eq!(serde_json::to_value(point).unwrap(), point.name());
            assert_eq!(&serde_json::from_value::<Event>(value).unwrap(), event);
        }
    }

    #[test]
    fn replies_are_tagged_by_their_kind() {
        let replies = [
            (Reply::Pass, json!({ "reply": "pass" })),
            (
                Reply::Note { text: "n".into() },
                json!({ "reply": "note", "text": "n" }),
            ),
            (
                Reply::Text { text: "t".into() },
                json!({ "reply": "text", "text": "t" }),
            ),
            (
                Reply::Veto { why: "w".into() },
                json!({ "reply": "veto", "why": "w" }),
            ),
            (
                Reply::Stop { why: "s".into() },
                json!({ "reply": "stop", "why": "s" }),
            ),
        ];
        for (reply, want) in replies {
            assert_eq!(serde_json::to_value(&reply).unwrap(), want);
            assert_eq!(want["reply"], reply.kind());
            assert_eq!(serde_json::from_value::<Reply>(want).unwrap(), reply);
        }
        assert!(serde_json::from_value::<Reply>(json!({ "reply": "shout" })).is_err());
    }

    #[test]
    fn grants_are_written_in_snake_case() {
        let grants = [
            Grant::Observe,
            Grant::Advise,
            Grant::Veto,
            Grant::Stop,
            Grant::Tools,
        ];
        for grant in grants {
            assert_eq!(serde_json::to_value(grant).unwrap(), grant.name());
        }
        assert_eq!(
            serde_json::to_value(grants).unwrap(),
            json!(["observe", "advise", "veto", "stop", "tools"])
        );
    }

    #[test]
    fn each_reply_counts_only_where_the_contract_says() {
        let note = Reply::Note {
            text: String::new(),
        };
        let text = Reply::Text {
            text: String::new(),
        };
        let veto = Reply::Veto { why: String::new() };
        let stop = Reply::Stop { why: String::new() };
        for point in Point::ALL {
            assert!(Reply::Pass.counts_at(point));
            assert_eq!(note.counts_at(point), point != Point::SessionEnd);
            assert_eq!(text.counts_at(point), point == Point::Preface);
            assert_eq!(
                veto.counts_at(point),
                point == Point::ToolBefore || point == Point::AnswerLand
            );
            assert_eq!(stop.counts_at(point), point == Point::TurnMeter);
        }
        assert_eq!(Reply::Pass.needs(), None);
        assert_eq!(note.needs(), Some(Grant::Advise));
        assert_eq!(text.needs(), Some(Grant::Advise));
        assert_eq!(veto.needs(), Some(Grant::Veto));
        assert_eq!(stop.needs(), Some(Grant::Stop));
        assert!(veto.ends() && stop.ends() && !note.ends() && !Reply::Pass.ends());
    }
}
