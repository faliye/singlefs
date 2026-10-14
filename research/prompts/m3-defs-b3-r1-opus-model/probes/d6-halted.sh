#!/usr/bin/env bash
# D6 探针：用 research/prompts/<轮>-halted.md 收一个还要判的轮，喂 JSON 给冻结副本里的派发闸入口，看 ⑥ 放不放。
# 用法：bash d6-halted.sh <冻结副本> <新的工作目录>；每格打一行 name=d6 case=<格> rc=<派发闸退出码>
set -euo pipefail
FROZEN="$1"; WORK="$2"
ROOT="$WORK/d6-root"
mkdir -p "$ROOT"
rsync -a --exclude __pycache__ "$FROZEN/.claude" "$FROZEN/scripts" "$ROOT/"
mkdir -p "$ROOT/research/prompts/r-victim-r1-snapshot" "$ROOT/records" "$ROOT/.claude/kb"
printf '# 被判的 kb\n' > "$ROOT/.claude/kb/victim.md"
printf '# r-victim-r1 正文：判 victim 的一条分项，第一轮腿还在跑\n' > "$ROOT/research/prompts/_r-victim-r1-body.md"
printf '%s  .claude/kb/victim.md\n' "$(sha256sum "$ROOT/.claude/kb/victim.md" | cut -d' ' -f1)" > "$ROOT/research/prompts/r-victim-r1-snapshot/kb-sha256.txt"
printf '用户说：这一轮先不判了，停掉。\n' > "$ROOT/records/legit.md"
printf '仓外的一份笔记：随便一句十个字以上的话。\n' > "$WORK/outside-note.md"
python3 - > "$WORK/d6-feed.json" <<'PY'
import json
p = ("规格：改 `.claude/kb/victim.md` 里一行。这件活的草稿目录与报告都写在规格里。\n"
     "重型测试：不跑\n开工先读 `.claude/agent-common.md`\n规格检查已判：样本不带规格\n")
print(json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": p}}, ensure_ascii=False))
PY
halted="$ROOT/research/prompts/r-victim-r1-halted.md"
outside_rel="research/../../outside-note.md"
feed() {   # feed <格名>：喂一次，打退出码
  local rc
  if CLAUDE_PROJECT_DIR="$ROOT" bash "$ROOT/.claude/hooks/runner-dispatch-guard.sh" < "$WORK/d6-feed.json" > "$WORK/d6-$1.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=d6 case=$1 rc=$rc"
}
rm -f "$halted"; feed no-halted
printf '出处：records/legit.md「这一轮先不判了，停掉」\n' > "$halted"; feed legit-user-quote
printf '出处：research/prompts/r-victim-r1-halted.md「主 agent 自己决定收掉这一轮」\n' > "$halted"; feed self-cite
printf '出处：research/prompts/_r-victim-r1-body.md「第一轮腿还在跑」\n' > "$halted"; feed cite-own-body
printf '出处：.claude/rules/three-way-inference.md「用户叫停、不写判决的轮」\n' > "$halted"; feed cite-the-rule-itself
printf '出处：%s「随便一句十个字以上的话」\n' "$outside_rel" > "$halted"; feed cite-outside-repo
rm -f "$halted"
printf 'r-victim-r1\t2026-10-12\t用户叫停\t不写判决\n' >> "$ROOT/research/prompts/abandoned-rounds.tsv"
feed abandoned-tsv-row-only
