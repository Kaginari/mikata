"""A Mikata plugin that only watches: every event of every point, one JSON line each, in
``observed.jsonl`` in its working folder (Mikata gives each plugin a scratch folder of its own) or
in the file its first argument names. It answers every hook with a pass, so it changes nothing a
body does: the plugin the bench loads to measure what hosting a plugin costs.

    [plugins.observe]
    command = "python3"
    args = ["/absolute/path/to/examples/plugins/observe.py"]
    grants = ["observe"]
    races = ["mikata"]
"""

import json
import os
import sys
import time

# Mikata runs plugins with PYTHONSAFEPATH=1: this folder is put on the path by hand.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from mikata_plugin import POINTS, Plugin  # noqa: E402

plugin = Plugin("observe", version="0.1.0", grants=["observe"])
LOG = sys.argv[1] if len(sys.argv) > 1 else "observed.jsonl"


def watch(event):
    """Writes the event down; answers nothing."""
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(json.dumps({"t": time.time(), **event}) + "\n")


for point in POINTS:
    plugin.on(point)(watch)

if __name__ == "__main__":
    plugin.run()
