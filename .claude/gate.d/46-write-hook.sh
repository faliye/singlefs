#!/usr/bin/env bash
# gate-stage: Write 覆盖未跟踪文件的 hook 注册着、而且会拒绝
#
# 判据：`.claude/settings.json` 的 PreToolUse 里有一条 matcher 为 Write、命令指向
# `.claude/hooks/refuse-overwrite-untracked.sh` 的 hook，并且那个脚本的 --selftest 通过。
# 为什么：hook 被人删掉或改坏时，工具层那道闸静默消失，而同一个坑（Write 盖掉别的会话未提交的文件，
# 2026-09-11 实测）会再来一次；只有门禁会在它消失时说话。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
SETTINGS="$ROOT/.claude/settings.json"
HOOK="$ROOT/.claude/hooks/refuse-overwrite-untracked.sh"

registered="$(python3 - "$SETTINGS" <<'PY' 2>&1
import json, sys
try:
    data = json.load(open(sys.argv[1], encoding='utf-8'))
except Exception as error:
    print(f'读不了：{error}'); sys.exit(1)
entries = (data.get('hooks') or {}).get('PreToolUse') or []
matched = sum(1 for entry in entries if entry.get('matcher') == 'Write'
              and any('refuse-overwrite-untracked.sh' in (hook.get('command') or '') for hook in entry.get('hooks') or []))
if matched:
    print(matched); sys.exit(0)
print('PreToolUse 里没有 matcher=Write 指向 refuse-overwrite-untracked.sh 的一条'); sys.exit(1)
PY
)"; rc=$?
if [[ $rc -ne 0 ]]; then
  echo "  ✗ .claude/settings.json 没注册那道 hook：$registered"
  echo "  → 怎么办：在 .claude/settings.json 的 hooks.PreToolUse 里加回 matcher \"Write\"、"
  echo "            command \"bash \\\"\$CLAUDE_PROJECT_DIR\\\"/.claude/hooks/refuse-overwrite-untracked.sh\"。"
  exit 1
fi
selftest_output="$(bash "$HOOK" --selftest 2>&1)"; rc=$?
if [[ $rc -ne 0 ]]; then
  echo "  ✗ refuse-overwrite-untracked.sh 的自检没过："
  printf '%s\n' "$selftest_output" | sed 's/^/    /'   # gate-lint:detail
  echo "  → 怎么办：修 .claude/hooks/refuse-overwrite-untracked.sh 的 decide()，再跑 --selftest 看它转绿。"
  exit 1
fi
cases="$(grep -oE '查了 [0-9]+ 种情形' <<<"$selftest_output" | grep -oE '[0-9]+')"
echo "  ✓ Write 覆盖未跟踪文件的 hook 注册着，自检通过（查了 $registered 条注册、${cases:-0} 种情形）"
