#!/usr/bin/env bash
# Q4（第一轮 Z4-丙，第二轮辩方判「修到」）：叫停登记的出处那一行，日期写在全角括号里「（YYYY-MM-DD）」，与文件名日期相同。
# 历史：用户当天叫停一轮，主 agent 在当天的调度记录里记一行（日期写进括号），照规则在 research/prompts/abandoned-rounds.tsv 登记一行、〔〕逐字抄那一段；
#   派实现员改那一轮快照里的文件，⑥ 放；提交前门禁 doc-text 的 date-once 格红，照它的出路 doc-consolidate.py migrate 删正文里重复文件名的日期，照着改回记录；
#   之后叫停表的判法（门禁 abandoned-rounds 格与派发闸 ⑥ 共用 lib_abandoned_rounds.py）与 ⑥ 各判一次。对照：日期不在括号里的同一句。
# 树：拿给的那一版副本 cp -al 一份（副本本身是腿草稿目录里从冻结副本 rsync 出来的，不含 multi-host.env），改到的三份文件先断开硬链接再写。
# 用法：q4_date_once.sh <名>=<副本根> <工作目录>
set -euo pipefail
trim() { python3 -c 'import sys; n = int(sys.argv[1]); sys.stdout.write("".join(line.rstrip("\n")[:n] + "\n" for line in sys.stdin))' "$1"; }
label="${1%%=*}" source_root="${1#*=}" work="$2"
record="records/2026-10-14-样本调度.md"
run_case() {   # 名 轮名 记录那一行 〔〕原话
  local name="$1" round="$2" line="$3" excerpt="$4" tree="$work/$label-$1" prompt
  rm -rf "$tree"; mkdir -p "$work"; cp -al "$source_root" "$tree"
  cp --remove-destination "$source_root/research/prompts/abandoned-rounds.tsv" "$tree/research/prompts/abandoned-rounds.tsv"
  printf '%s\t2026-10-14\t用户叫停（出处 %s 第 6 行〔%s〕），两条腿当场停掉\t不写判决\n' "$round" "$record" "$excerpt" >> "$tree/research/prompts/abandoned-rounds.tsv"
  printf '%s\n' '# 样本调度' '' '## 课题' '' "- 开轮 $round：冻结副本在工作根下，快照 research/prompts/$round-snapshot/。" "$line" '- 别的活照常。' > "$tree/$record"
  : > "$tree/research/prompts/_$round-body.md"
  mkdir -p "$tree/research/prompts/$round-snapshot"
  printf '%064d  %s\n' 0 crates/singlefs-core/src/lib.rs > "$tree/research/prompts/$round-snapshot/frozen-copy-sha256.txt"
  prompt="$work/$label-$name-writer-prompt.txt"
  printf '%s\n' '开工先读 `.claude/agent-common.md`。' '重型测试：不跑' "草稿目录：$work/$label-$name-scratch/" "报告：$work/$label-$name-scratch/report.md" \
    '条款：照判决改法给一处断言补一行注释。' '要动的 crates 文件：crates/singlefs-core/src/lib.rs' '预计挂钟：2 小时内' '线程上限：31' > "$prompt"
  state() {
    local lib dispatch code
    lib="$(cd "$tree" && PYTHONDONTWRITEBYTECODE=1 python3 -B .claude/hooks/lib_abandoned_rounds.py . 2>&1 | grep -F "$round" | trim 400)" || true
    dispatch="$(cd "$tree" && CLAUDE_PROJECT_DIR="$tree" PYTHONDONTWRITEBYTECODE=1 bash .claude/hooks/runner-dispatch-guard.sh --check-prompt "$prompt" --type implementation-writer --root "$tree" 2>&1)" && code=0 || code=$?
    printf '%s\t%s\t%s\t第 6 行：%s\t叫停表：%s\t⑥ 派实现员退出=%s\t%s\n' "$label" "$name" "$1" "$(sed -n 6p "$tree/$record")" "$lib" "$code" \
      "$(grep -E "三方轮 $round" <<<"$dispatch" | head -1 | trim 160)"
  }
  state 登记之后
  ( cd "$tree" && PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/kb/doc-consolidate.py migrate "$record" --out "$tree/migrated.md" --force > "$work/$label-$name-migrate.txt" 2>&1 )
  mv "$tree/migrated.md" "$tree/$record"
  state date-once出路之后
  rm -rf "$tree"
}
run_case paren hx-paren-r1 '- hx-paren-r1 用户叫停（2026-10-14）：「这一轮先停掉，不写判决」，两条腿当场停掉。' 'hx-paren-r1 用户叫停（2026-10-14）'
run_case plain hx-plain-r1 '- hx-plain-r1 用户 2026-10-14 叫停：「这一轮先停掉，不写判决」，两条腿当场停掉。' 'hx-plain-r1 用户 2026-10-14 叫停'
run_case paren-comma hx-comma-r1 '- hx-comma-r1 用户叫停（2026-10-14，当场）：「这一轮先停掉，不写判决」，两条腿当场停掉。' 'hx-comma-r1 用户叫停（2026-10-14，当场）'
