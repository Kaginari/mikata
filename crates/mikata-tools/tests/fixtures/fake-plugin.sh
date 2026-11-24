#!/usr/bin/env bash
# A fake plugin for the tests: `fake-plugin.sh <log> [plain|deaf]`.
# It shakes hands with the `mikata` capability (hooking tool_before and answer_land, asking for
# advise, veto and stop), logs its pid, each hook and an unload to <log>, and answers a
# tool_before by the tool's name: `veto` vetoes, `note` notes, `stop` answers a stop (which does
# not count there), `garbage` answers no reply, `slow` answers after a second, `crash` ends the
# plugin; any other name passes. With `plain` it is a plain MCP server; with `deaf` it does not
# end when it is unloaded.
log=$1
mode=${2:-}
echo "pid $$" >>"$log"
cap='"experimental":{"mikata":{"version":1,"id":"fake","plugin_version":"0.1.0","points":["tool_before","answer_land"],"grants":["advise","veto","stop"]}}'
[ "$mode" = plain ] && cap='"tools":{}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  method=$(printf '%s' "$line" | sed -n 's/.*"method":"\([^"]*\)".*/\1/p')
  case "$method" in
    initialize)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"2025-06-18","capabilities":{%s},"serverInfo":{"name":"fake","version":"1"}}}\n' "$id" "$cap"
      ;;
    tools/list)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[]}}\n' "$id"
      ;;
    mikata/unload)
      echo "unload" >>"$log"
      [ "$mode" = deaf ] && continue
      exit 0
      ;;
    mikata/hook)
      point=$(printf '%s' "$line" | sed -n 's/.*"point":"\([^"]*\)".*/\1/p')
      name=$(printf '%s' "$line" | sed -n 's/.*"name":"\([^"]*\)".*/\1/p')
      echo "hook $point $name" >>"$log"
      reply='{"reply":"pass"}'
      case "$name" in
        veto) reply='{"reply":"veto","why":"not this one"}' ;;
        note) reply='{"reply":"note","text":"mind the tests \u001b[31m"}' ;;
        stop) reply='{"reply":"stop","why":"enough"}' ;;
        garbage) reply='{"reply":"shout"}' ;;
        slow) sleep 1 ;;
        crash) exit 3 ;;
      esac
      printf '{"jsonrpc":"2.0","id":%s,"result":%s}\n' "$id" "$reply"
      ;;
  esac
done
