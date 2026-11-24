"""A Mikata plugin in one file, standard library only.

A plugin is an MCP server over stdio that also speaks ``mikata/*``: Mikata starts it from the
human's global file, shakes hands with MCP ``initialize``, reads what it hooks from
``capabilities.experimental.mikata``, then sends ``mikata/hook`` requests (the params are the
event, the result is the reply) and, at the end, the ``mikata/unload`` notification.

    from mikata_plugin import Plugin, veto

    plugin = Plugin("no-force-push", version="0.1.0", grants=["veto"])

    @plugin.tool_before
    def guard(event):
        if "--force" in event["input"].get("command", ""):
            return veto("no force pushes here")

    plugin.run()

A hook gets the event as a dict (its ``point`` key names the point) and returns ``None`` for a
pass, or one of ``note``, ``text``, ``veto`` and ``stop``. A hook must answer within the
human's deadline (two seconds unless they say otherwise); a late, failed or malformed answer, or
one the human did not grant, is read as a pass and counted, and three in a session end the
plugin. Tools are listed only when declared with ``@plugin.tool``, and reach a body only when
the human grants ``tools``.

Mikata runs a plugin with ``PYTHONSAFEPATH=1``, so a plugin beside this file adds its own folder
to ``sys.path`` before importing it (see ``no_force_push.py``).
"""

import json
import sys
import traceback

#: The version of the ``mikata`` capability this file speaks.
PROTOCOL = 1

#: Every point a plugin may hook, in the order the loop meets them in one ask.
POINTS = (
    "ask_begin",
    "preface",
    "turn_meter",
    "tool_before",
    "tool_after",
    "answer_land",
    "answer_landed",
    "session_end",
)

#: What a plugin may ask to be granted. The human's grant cuts the ask down.
GRANTS = ("observe", "advise", "veto", "stop", "tools")


def note(text):
    """Advice, said to the body as a context update (needs ``advise``)."""
    return {"reply": "note", "text": str(text)}


def text(block):
    """At ``preface`` only: a block in the preface's plugins slot (needs ``advise``)."""
    return {"reply": "text", "text": str(block)}


def veto(why):
    """At ``tool_before`` the call is not run and is answered ``why``; at ``answer_land`` the
    answer is handed back (needs ``veto``)."""
    return {"reply": "veto", "why": str(why)}


def stop(why):
    """At ``turn_meter`` only: the ask ends with ``why`` (needs ``stop``)."""
    return {"reply": "stop", "why": str(why)}


PASS = {"reply": "pass"}


class Plugin:
    """One plugin: its hooks by point, its tools, and the loop that serves them."""

    def __init__(self, name, version="0.1.0", grants=("observe",)):
        unknown = [g for g in grants if g not in GRANTS]
        if unknown:
            raise ValueError("unknown grants: %s; the grants are %s" % (unknown, GRANTS))
        self.name = name
        self.version = version
        self.grants = list(grants)
        self.hooks = {}
        self.tools = {}
        self.on_unload = None
        for point in POINTS:
            setattr(self, point, self._hook(point))

    def _hook(self, point):
        def register(fn):
            self.hooks[point] = fn
            return fn

        register.__doc__ = "Hook ``%s``: the decorated function gets the event." % point
        return register

    def on(self, point):
        """Hook ``point`` by its name: ``@plugin.on("tool_before")``."""
        if point not in POINTS:
            raise ValueError("unknown point %r; the points are %s" % (point, POINTS))
        return self._hook(point)

    def tool(self, name, description="", schema=None):
        """Declare an MCP tool: the decorated function gets the call's arguments as a dict and
        returns its text. Offered to a body only when the human grants ``tools``."""

        def register(fn):
            self.tools[name] = {
                "fn": fn,
                "spec": {
                    "name": name,
                    "description": description,
                    "inputSchema": schema or {"type": "object", "properties": {}},
                },
            }
            return fn

        return register

    def unload(self, fn):
        """Run ``fn`` when Mikata unloads the plugin, before it ends (within a second)."""
        self.on_unload = fn
        return fn

    def capability(self):
        """What the handshake says: the points hooked and the grants asked for."""
        return {
            "version": PROTOCOL,
            "id": self.name,
            "plugin_version": self.version,
            "points": [p for p in POINTS if p in self.hooks],
            "grants": self.grants,
        }

    def handle(self, msg):
        """The answer to one message, or ``None`` when it wants none; ``False`` to end."""
        method = msg.get("method")
        params = msg.get("params") or {}
        if method == "mikata/unload":
            if self.on_unload:
                self.on_unload()
            return False
        if "id" not in msg or method is None:
            return None
        if method == "initialize":
            capabilities = {"experimental": {"mikata": self.capability()}}
            if self.tools:
                capabilities["tools"] = {}
            return {
                "protocolVersion": params.get("protocolVersion", "2025-06-18"),
                "capabilities": capabilities,
                "serverInfo": {"name": self.name, "version": self.version},
            }
        if method == "ping":
            return {}
        if method == "tools/list":
            return {"tools": [t["spec"] for t in self.tools.values()]}
        if method == "tools/call":
            tool = self.tools.get(params.get("name"))
            if tool is None:
                raise LookupError("no tool %r" % params.get("name"))
            try:
                out = tool["fn"](params.get("arguments") or {})
                return {"content": [{"type": "text", "text": str(out)}], "isError": False}
            except Exception as e:  # noqa: BLE001 - a tool's failure is its answer
                return {"content": [{"type": "text", "text": str(e)}], "isError": True}
        if method == "mikata/hook":
            fn = self.hooks.get(params.get("point"))
            reply = fn(params) if fn else None
            return PASS if reply is None else reply
        raise NotImplementedError("no method %r" % method)

    def run(self, stdin=None, stdout=None):
        """Serve requests on stdin, one JSON message per line, until unloaded or stdin closes."""
        stdin = stdin or sys.stdin
        stdout = stdout or sys.stdout

        def say(obj):
            stdout.write(json.dumps(obj) + "\n")
            stdout.flush()

        for line in stdin:
            line = line.strip()
            if not line:
                continue
            try:
                msg = json.loads(line)
            except ValueError:
                say({"jsonrpc": "2.0", "id": None, "error": {"code": -32700, "message": "parse error"}})
                continue
            try:
                result = self.handle(msg)
            except NotImplementedError as e:
                say({"jsonrpc": "2.0", "id": msg.get("id"), "error": {"code": -32601, "message": str(e)}})
                continue
            except Exception as e:  # noqa: BLE001 - the host counts a refused hook as a strike
                traceback.print_exc(file=sys.stderr)
                say({"jsonrpc": "2.0", "id": msg.get("id"), "error": {"code": -32603, "message": str(e)}})
                continue
            if result is False:
                return
            if result is not None:
                say({"jsonrpc": "2.0", "id": msg["id"], "result": result})
