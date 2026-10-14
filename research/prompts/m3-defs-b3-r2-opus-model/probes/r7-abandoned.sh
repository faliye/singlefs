#!/usr/bin/env bash
# R7 探针：派发闸 ⑥ 读 research/prompts/abandoned-rounds.tsv 收「用户叫停」的轮。records/ 下一份记录里放几种行（真叫停、别的轮的叫停原话与这一轮同一行、
# 开轮的调度行、引规则原文的行、用户说先放一放），登记表每格只留一行，喂 kb-scribe 的派发（要改的文件在还开着的 r-victim-r1 快照里）。
# 用法：bash r7-abandoned.sh <冻结副本> <新的工作目录>；每格一行 name=r7 case=<格> rc=<派发闸退出码>
# 同一份脚本也喂第一轮冻结副本：那一版 ⑥ 不读登记表、读 <轮>-halted.md，halted 那几格只在那一版有意义。
set -euo pipefail
FROZEN="$1"; WORK="$2"
ROOT="$WORK/r7-root"
mkdir -p "$ROOT"
rsync -a --exclude __pycache__ "$FROZEN/.claude" "$FROZEN/scripts" "$ROOT/"
mkdir -p "$ROOT/research/prompts/r-victim-r1-snapshot" "$ROOT/records" "$ROOT/.claude/kb"
printf '# 被判的 kb\n' > "$ROOT/.claude/kb/victim.md"
printf '# r-victim-r1 正文：判 victim 的一条分项，第一轮腿还在跑\n' > "$ROOT/research/prompts/_r-victim-r1-body.md"
printf '%s  .claude/kb/victim.md\n' "$(sha256sum "$ROOT/.claude/kb/victim.md" | cut -d' ' -f1)" > "$ROOT/research/prompts/r-victim-r1-snapshot/kb-sha256.txt"
cat > "$ROOT/records/r7.md" <<'REC'
# 调度记录（样本）
- 用户：「这一轮先不判了，停掉吧」。去向：r-victim-r1 两条腿停掉。
- 用户叫停 r-other-r1：「这一轮先停掉不做了」；r-victim-r1 照常跑，等腿交回。
- 开 r-victim-r1，攻方腿攻击面「R1、R2、R5、R7、R9 这几格」。
- r-victim-r1 正文引了规则原文「每一次工具调用写进文件的内容不超过 150 行」。
- 用户说「r-victim-r1 先放一放，等我看完判决再接着跑」。
REC
python3 - > "$WORK/r7-feed.json" <<'PY'
import json
p = ("规格：改 `.claude/kb/victim.md` 里一行。这件活的草稿目录与报告都写在规格里。\n"
     "重型测试：不跑\n开工先读 `.claude/agent-common.md`\n规格检查已判：样本不带规格\n")
print(json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": p}}, ensure_ascii=False))
PY
TABLE="$ROOT/research/prompts/abandoned-rounds.tsv"
HALTED="$ROOT/research/prompts/r-victim-r1-halted.md"
feed() {   # feed <格名>：喂一次，打退出码
  local rc
  if CLAUDE_PROJECT_DIR="$ROOT" bash "$ROOT/.claude/hooks/runner-dispatch-guard.sh" < "$WORK/r7-feed.json" > "$WORK/r7-$1.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=r7 case=$1 rc=$rc"
}
row() {   # row <第三列>：登记表只留这一行
  printf 'r-victim-r1\t2026-10-13\t%s\t不写判决\n' "$1" > "$TABLE"
}
rm -f "$TABLE" "$HALTED"; feed no-registration
row '用户叫停（records/r7.md 第 2 行）'; feed legit-user-quote
row '用户叫停（records/r7.md 第 3 行）'; feed other-rounds-halt-quote-same-line
row '用户叫停（records/r7.md 第 4 行）'; feed dispatch-log-line
row '用户叫停（records/r7.md 第 5 行）'; feed rule-quote-line
row '用户叫停（records/r7.md 第 6 行）'; feed user-said-pause-only
# 与门禁 doc-process-records 的 abandoned-rounds 格对不上的几种行（那一格的判法见 gate-samples/r7-*，要主 agent 跑）
printf 'r-victim-r1\t10月13日\t用户叫停（records/r7.md 第 2 行）\t不写判决\n' > "$TABLE"; feed date-not-iso
printf 'r-victim-r1\t2026-10-13\t用户叫停（records/r7.md 第 9 行）\t不写判决\nr-victim-r1\t2026-10-13\t用户叫停（records/r7.md 第 2 行）\t不写判决\n' > "$TABLE"; feed duplicated-rows-second-good
printf 'r-victim-r1\t2026-10-13\t用户叫停，原话见调度记录\t不写判决\n' > "$TABLE"; feed no-source
# 第一轮那一版的写法：halted 文件引同一份记录里别的轮的叫停原话
rm -f "$TABLE"
printf '出处：records/r7.md「这一轮先停掉不做了」\n' > "$HALTED"; feed halted-other-rounds-quote
rm -f "$HALTED"
