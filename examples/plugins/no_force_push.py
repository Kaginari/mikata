"""A Mikata plugin that vetoes force pushes.

Every ``bash`` call is read before it runs; one whose command runs ``git push`` with
``--force``, ``--force-with-lease``, ``-f`` (alone or among other short flags) or a ``+refspec``
is not run, and the body is told why. Add it to your global file:

    [plugins.no-force-push]
    command = "python3"
    args = ["/absolute/path/to/examples/plugins/no_force_push.py"]
    grants = ["veto"]
    races = ["mikata"]
"""

import os
import shlex
import sys

# Mikata runs plugins with PYTHONSAFEPATH=1: this folder is put on the path by hand.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from mikata_plugin import Plugin, veto  # noqa: E402

plugin = Plugin("no-force-push", version="0.1.0", grants=["veto"])

#: The words that separate one command from the next.
SEPARATORS = {";", "&&", "||", "|", "&", "(", ")", "{", "}"}

#: git's global options that take a value as the next word.
GIT_VALUED = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--exec-path"}


def commands(line):
    """The simple commands of a shell line, each a list of words (best effort)."""
    try:
        lexer = shlex.shlex(line, posix=True, punctuation_chars=";&|(){}")
        lexer.whitespace_split = True
        words = list(lexer)
    except ValueError:
        words = line.split()
    out, cur = [], []
    for w in words:
        if w in SEPARATORS:
            if cur:
                out.append(cur)
            cur = []
        else:
            cur.append(w)
    if cur:
        out.append(cur)
    return out


def push_args(words):
    """The arguments of ``git push`` when ``words`` run it, else ``None``."""
    i = 0
    # Variable assignments and a `command` or `env` prefix before the program.
    while i < len(words) and ("=" in words[i] and not words[i].startswith("-") or words[i] in ("command", "env")):
        i += 1
    if i >= len(words) or os.path.basename(words[i]) != "git":
        return None
    i += 1
    while i < len(words) and words[i].startswith("-"):
        i += 2 if words[i] in GIT_VALUED else 1
    if i >= len(words) or words[i] != "push":
        return None
    return words[i + 1:]


def forces(args):
    """Whether these ``git push`` arguments force."""
    for a in args:
        if a == "--force" or a.startswith("--force-with-lease") or a == "--force-if-includes":
            return True
        if a.startswith("-") and not a.startswith("--") and "f" in a[1:]:
            return True
        if a.startswith("+") and len(a) > 1:
            return True
    return False


@plugin.tool_before
def guard(event):
    if event.get("name") != "bash":
        return None
    line = (event.get("input") or {}).get("command") or ""
    for words in commands(line):
        args = push_args(words)
        if args is not None and forces(args):
            return veto("force pushes are not allowed here: `%s`; push without force, or ask the human" % " ".join(words))
    return None


if __name__ == "__main__":
    plugin.run()
