#!/usr/bin/env bash
# gate-stage: Write 覆盖未跟踪文件的 hook 注册着、而且会拒绝
#
# 判据：`.claude/settings.json` 的 PreToolUse 里有一条 matcher 覆盖 Write、命令指向
# `.claude/hooks/write-guard.sh` 的 hook，并且那个脚本的 --selftest 通过、自检里有「整份覆盖」那几种情形。
# 2026-09-17 起这一道与写范围闸合在 write-guard.sh 里（原来是单独的 refuse-overwrite-untracked.sh）。
# 为什么：hook 被人删掉或改坏时，工具层那道闸静默消失，而同一个坑（Write 盖掉别的会话未提交的文件，
# 2026-09-11 实测）会再来一次；只有门禁会在它消失时说话。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
SETTINGS="$ROOT/.claude/settings.json"
HOOK="$ROOT/.claude/hooks/write-guard.sh"

registered="$(python3 - "$SETTINGS" <<'PY' 2>&1
import json, sys
try:
    data = json.load(open(sys.argv[1], encoding='utf-8'))
except Exception as error:
    print(f'读不了：{error}'); sys.exit(1)
entries = (data.get('hooks') or {}).get('PreToolUse') or []
matched = sum(1 for entry in entries if 'Write' in (entry.get('matcher') or '').split('|')
              and any('write-guard.sh' in (hook.get('command') or '') for hook in entry.get('hooks') or []))
if matched:
    print(matched); sys.exit(0)
print('PreToolUse 里没有 matcher 覆盖 Write、指向 write-guard.sh 的一条'); sys.exit(1)
PY
)"; rc=$?
if [[ $rc -ne 0 ]]; then
  echo "  ✗ .claude/settings.json 没注册那道 hook：$registered"
  echo "  → 怎么办：在 .claude/settings.json 的 hooks.PreToolUse 里加回 matcher \"Write|Edit\"、"
  echo "            command \"bash \\\"\$CLAUDE_PROJECT_DIR\\\"/.claude/hooks/write-guard.sh\"。"
  exit 1
fi
selftest_output="$(bash "$HOOK" --selftest 2>&1)"; rc=$?
if [[ $rc -ne 0 ]]; then
  echo "  ✗ write-guard.sh 的自检没过："
  printf '%s\n' "$selftest_output" | sed 's/^/    /'   # gate-lint:detail
  echo "  → 怎么办：修 .claude/hooks/write-guard.sh 的 decide_overwrite()，再跑 --selftest 看它转绿。"
  exit 1
fi
if ! grep -q '未跟踪的已有文件整份覆盖拒绝' <<<"$selftest_output"; then
  echo "  ✗ write-guard.sh 的自检通过了，但没报「未跟踪的已有文件整份覆盖拒绝」：整份覆盖那一道可能从自检里漏掉了"
  echo "  → 怎么办：在 .claude/hooks/write-guard.sh 的 selftest() 里补回「覆盖:」那几种情形，成功行照原样报。"
  exit 1
fi
cases="$(grep -oE '查了 [0-9]+ 种情形' <<<"$selftest_output" | grep -oE '[0-9]+')"
echo "  ✓ 写 hook（含 Write 覆盖未跟踪文件那一道）注册着，自检通过（查了 $registered 条注册、${cases:-0} 种情形）"
