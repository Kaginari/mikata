"""A Mikata plugin that is a memory: what to put in place of the native one, as an example of a
part of the loop swapped for a third party's.

It keeps the lessons a body writes in its answer — each line holding ``@U`` — and shows the ones
that share the most words with the next ask, in the ask's preface, quoted as data. Its notes live
in ``notes.jsonl`` in its working folder (Mikata gives each plugin a scratch folder of its own,
kept per world), so they outlive the session. Switch the native memory off and this one on:

    [units]
    memory = false

    [plugins.notes]
    command = "python3"
    args = ["/absolute/path/to/examples/plugins/notes_memory.py"]
    grants = ["advise"]
    races = ["mikata"]
"""

import json
import os
import re
import sys

# Mikata runs plugins with PYTHONSAFEPATH=1: this folder is put on the path by hand.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from mikata_plugin import Plugin, text  # noqa: E402

plugin = Plugin("notes", version="0.1.0", grants=["advise"])
NOTES = "notes.jsonl"
SHOWN = 5
WORD = re.compile(r"[a-z0-9_]{3,}")


def words(s):
    return set(WORD.findall(s.lower()))


def notes():
    if not os.path.exists(NOTES):
        return []
    with open(NOTES, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


@plugin.preface
def recall(event):
    """The notes that share the most words with the ask, best first."""
    asked = words(event.get("ask", ""))
    scored = sorted(((len(asked & words(n["text"])), n["text"]) for n in notes()), reverse=True)
    best = [t for s, t in scored if s > 0][:SHOWN]
    if best:
        return text("What earlier sessions learned (the notes plugin):\n" + "\n".join("- " + t for t in best))
    return None


@plugin.answer_landed
def keep(event):
    """Each `@U` line of the answer, kept once."""
    have = {n["text"] for n in notes()}
    with open(NOTES, "a", encoding="utf-8") as f:
        for line in event.get("answer", "").splitlines():
            if "@U" not in line:
                continue
            lesson = line.split("@U", 1)[1].strip(" `*-")
            lesson = re.sub(r"^(colony|law)\s+", "", lesson)
            if lesson and lesson not in have:
                f.write(json.dumps({"text": lesson}) + "\n")
                have.add(lesson)
    return None


if __name__ == "__main__":
    plugin.run()
