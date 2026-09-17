#!/usr/bin/env bash
# gate-stage: 项目 subagent 的写范围闸注册着、会拒绝，写范围表与定义一致
#
# 判据，任一条不成立判红：
#   ① `.claude/settings.json` 的 PreToolUse 里有一条 matcher 同时覆盖 Write 与 Edit、命令指向 `agent-write-scope.sh` 的 hook；
#   ② `.claude/hooks/agent-write-scope.sh --selftest` 通过（自证里有走真实 stdin 入口的情形）；
#   ③ `.claude/agents/` 里 tools 含 Write 或 Edit 的每个定义，在 `.claude/hooks/agent-write-scope.tsv` 里至少有一条路径模式；
#   ④ 表里每个 agent 名都有定义，每行至少两列。
#
# 为什么：执行类 agent 越界写，靠定义里一句「只写写范围」拦不住；hook 被删、自证坏了、新加一个能写文件的定义忘了登记，
# 这道闸都会静默消失或静默放行——只有门禁会在它消失时说话（与 46 号同一个道理）。
# 2026-09-17 写 hook 时实测撞过一次静默放行：程序从标准输入读，hook 的 JSON 读不到，判定一律放行，而直接调判定函数的自证全绿。
#
#   bash .claude/gate.d/63-agent-write-scope.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
HOOK="$(cd "$(dirname "$0")/../hooks" && pwd)/agent-write-scope.sh"
cd "$ROOT" 2>/dev/null || exit 2
table_output="$(python3 - <<'PY'
import glob, json, os, re, sys
failed = False
settings_path = ".claude/settings.json"
def matcher_covers(matcher, tool):
    if matcher in ("*", ""):
        return True
    if re.fullmatch(r"[A-Za-z0-9_|]+", matcher):
        return tool in matcher.split("|")
    try:
        return re.search(matcher, tool) is not None
    except re.error:
        return False
try:
    entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("PreToolUse") or []
except Exception as error:
    entries = None
    failed = True
    print(f"  ✗ 读不了 {settings_path}：{error}")
    print("     → 怎么办：修好这份 JSON，再在 hooks.PreToolUse 里注册写范围闸。")
if entries is not None:
    registered = [entry for entry in entries
                  if matcher_covers(entry.get("matcher", ""), "Write") and matcher_covers(entry.get("matcher", ""), "Edit")
                  and any("agent-write-scope.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册写范围闸：PreToolUse 里没有 matcher 同时覆盖 Write 与 Edit、命令指向 agent-write-scope.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Write|Edit"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/agent-write-scope.sh"。')
table_path = ".claude/hooks/agent-write-scope.tsv"
patterns_by_agent, malformed = {}, []
if os.path.isfile(table_path):
    for line_number, line in enumerate(open(table_path, encoding="utf-8"), 1):
        line = line.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) < 2 or not fields[0].strip() or not fields[1].strip():
            malformed.append(f"第 {line_number} 行：{line}")
            continue
        patterns_by_agent.setdefault(fields[0].strip(), []).append(fields[1].strip())
else:
    failed = True
    print(f"  ✗ 没有 {table_path}")
    print("     → 怎么办：建这张表：agent 名、路径模式、出处三列，用制表符分隔，按每个能写文件的定义的「写范围」一节转写。")
writers = []
for path in sorted(glob.glob(".claude/agents/*.md")):
    head = open(path, encoding="utf-8").read().split("\n---", 1)[0]
    tools_line = next((line for line in head.split("\n") if line.startswith("tools:")), "")
    tools = {tool.strip() for tool in tools_line[len("tools:"):].split(",")}
    if tools & {"Write", "Edit"}:
        writers.append(os.path.basename(path)[:-3])
if malformed:
    failed = True
    print("  ✗ 写范围表里这些行不到两列、或 agent 名与路径模式有空的：")  # gate-lint:summary
    for entry in malformed:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行至少写 agent 名与路径模式两列，用制表符分隔。")
unscoped = [name for name in writers if name not in patterns_by_agent]
if unscoped:
    failed = True
    print("  ✗ 这些定义的 tools 里有 Write 或 Edit，写范围表里却没有登记——hook 会把它们的每次写都拒掉：")  # gate-lint:summary
    for name in unscoped:
        print(f"     {name}")  # gate-lint:detail
    print(f"     → 怎么办：照它定义里「写范围」一节，在 {table_path} 给它加路径模式。")
ghosts = [name for name in patterns_by_agent if not os.path.isfile(f".claude/agents/{name}.md")]
if ghosts:
    failed = True
    print("  ✗ 写范围表里登记的这些 agent 在 .claude/agents/ 里没有定义：")  # gate-lint:summary
    for name in ghosts:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：改成现在的定义名，或删掉这几行。")
if failed:
    sys.exit(1)
pattern_count = sum(len(patterns) for patterns in patterns_by_agent.values())
print(f"TABLE_OK {len(writers)} {pattern_count}")
PY
)"; table_rc=$?
printf '%s\n' "$table_output" | grep -v '^TABLE_OK '
if [[ $table_rc -ne 0 ]]; then exit 1; fi
read -r _ writer_count pattern_count <<<"$(printf '%s\n' "$table_output" | grep '^TABLE_OK ')"
selftest_output="$(bash "$HOOK" --selftest 2>&1)"; selftest_rc=$?
if [[ $selftest_rc -ne 0 ]]; then
  echo "  ✗ agent-write-scope.sh 的自证没过："
  printf '%s\n' "$selftest_output" | sed 's/^/    /'   # gate-lint:detail
  echo "     → 怎么办：修 .claude/hooks/agent-write-scope.sh 的 decide() 或入口，再跑 --selftest 看它转绿。"
  exit 1
fi
echo "  ✓ 写范围闸注册着、自证通过，表与定义一致（${writer_count} 个有 Write 或 Edit 的定义、${pattern_count} 条路径模式）"
