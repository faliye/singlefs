#!/usr/bin/env bash
# 模型：把 hook 的 JSON 直接喂给仓里的写范围闸 .claude/hooks/agent-write-scope.sh（只读调用，不改 hook、不改表、不写仓），
# 看几类执行 agent 的写目标闸判什么。闸只判不写：退出码 0 = 放行，2 = 拒绝，其它 = Claude Code 当非阻塞错误（工具照常执行）。
#   SINGLEFS_ROOT=/home/fy5090/code/singlefs HOOK_PROBE_PARENT=/tmp/claude-1000/agent-defs-r1-opus bash hook-probes.sh
# 临时目录必须在 /tmp/claude-1000/ 下（H10 要落在执行 agent 写范围表的 /tmp/claude-1000/** 那一条里）；跑完删掉。
# 输出每行：name=probe id=… agent=… tool=… got=… note=…；末行 name=done。
set -uo pipefail
ROOT="${SINGLEFS_ROOT:-/home/fy5090/code/singlefs}"
HOOK="$ROOT/.claude/hooks/agent-write-scope.sh"
SCRATCH="$(mktemp -d "${HOOK_PROBE_PARENT:-/tmp/claude-1000/agent-defs-r1-opus}/hook-probe.XXXXXX")"
trap 'rm -rf "${SCRATCH:?}"' EXIT

probe() { # id agent tool path note
  local json got
  json="$(python3 -c 'import json,sys; print(json.dumps({"tool_name": sys.argv[2], "agent_type": sys.argv[1], "tool_input": {"file_path": sys.argv[3], "content": "x"}}))' "$2" "$3" "$4")"
  printf '%s' "$json" | CLAUDE_PROJECT_DIR="$ROOT" bash "$HOOK" >/dev/null 2>&1; got=$?
  printf 'name=probe id=%s agent=%s tool=%s got=%s path=%s note=%s\n' "$1" "$2" "$3" "$got" "$4" "$5"
}

probe H1 kb-scribe Edit "$ROOT/CLAUDE.md" "Edit 被拒；同一个目标写进 replace-batch.py 规格走 Bash 不经过闸"
probe H2 kb-scribe Edit "$ROOT/.claude/kb/decisions/22-单元原子性怎么合成.md" "范围内"
probe H3 kb-scribe Edit "$ROOT/records/2026-09-13-总审核.md" "Edit 被拒；relabel-item.py 走 Bash 照样改它"
probe H4 kb-scribe Edit "$ROOT/.claude/rules/three-way-inference.md" "Edit 被拒；relabel-item.py 走 Bash 照样改它"
probe H5 implementation-writer Write "/tmp/claude-1000/agent-defs-r1-sonnet/draft.md" "别的腿这一轮的草稿目录"
probe H6 implementation-writer Write "/tmp/claude-1000/-home-fy5090-code-singlefs/some-session/scratchpad/notes.md" "主 agent 会话的 scratchpad"
probe H7 experiment-runner Write "$ROOT/research/results/e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out" "别的实验已被 replay.sh 点名的留存产物"
probe H8 experiment-runner Edit "$ROOT/research/prompts/e152-preregistration.md" "别的实验的跑前登记（整份可改，不止修订一段）"
probe H9 implementation-writer Edit "$ROOT/crates/singlefs-core/src/mount.rs" "输入里标了别的会话在改的文件，闸不知道"

probe H13 experiment-runner Edit "$ROOT/research/prompts/e151-r6-prereg.md" "重跑的登记（仓里 6 份 -rN-prereg.md 形态）写修订"
probe H14 experiment-runner Edit "$ROOT/.claude/kb/experiments-history.md" "实验页的文末历史（新建与每次重跑都记一条）"

probe H15 kb-writer Edit "$ROOT/crates/singlefs-core/src/allocator.rs" "agent_type 在 .claude/agents/ 里没有文件（定义改名之后续做的旧实例同形）：按内置 agent 放行"

mkdir -p "$SCRATCH/copy"
ln -s "$ROOT/.claude/rules" "$SCRATCH/copy/rules-link"
probe H10 implementation-writer Write "$SCRATCH/copy/rules-link/new-rule.md" "经符号链接落进 $(readlink -f "$SCRATCH/copy/rules-link")（闸只做 normpath，不解链接）"

mkdir -p "$SCRATCH/hooks-without-table"
cp "$HOOK" "$SCRATCH/hooks-without-table/agent-write-scope.sh"
json='{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"'"$ROOT"'/crates/a.rs"}}'
printf '%s' "$json" | CLAUDE_PROJECT_DIR="$ROOT" bash "$SCRATCH/hooks-without-table/agent-write-scope.sh" >/dev/null 2>&1; got=$?
printf 'name=probe id=H11 agent=kb-scribe tool=Edit got=%s path=%s note=%s\n' "$got" "$ROOT/crates/a.rs" "同一份 hook 旁边没有表（读表抛异常）：退出码不是 2"
printf '{not json' | CLAUDE_PROJECT_DIR="$ROOT" bash "$HOOK" >/dev/null 2>&1; got=$?
printf 'name=probe id=H12 agent=- tool=- got=%s path=- note=%s\n' "$got" "输入不是 JSON：按放行处理"
echo "name=done probes=15"
