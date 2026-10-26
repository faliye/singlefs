#!/usr/bin/env bash
# K3 ⑳ 与 K2-5 的探针：造几份派发提示，喂 <草稿>/tree 里拷的派发闸 --check-prompt，看「草稿目录」与三方腿报告形态怎么认。
# 用法：bash scratch_prompts.sh <草稿目录（里面有 tree/）> <仓根（只读：--root 与骨架）>
set -uo pipefail
work="$1"; repo="$2"; prompts="$work/scratch-prompts"; mkdir -p "$prompts"
root=/home/fy5090/.local/share/singlefs/work
guard="$work/tree/.claude/hooks/runner-dispatch-guard.sh"
head_lines() {
  printf '%s\n' "# 派发：clerk" "" "重型测试：不跑" "批次：records/x.md#1" "线程上限：31" "草稿目录：$root/k3-probe-mine" \
    "认领：bash scripts/hosts/work-directory-ownership.sh claim $root/k3-probe-mine --purpose '探针'" "报告：$root/k3-probe-mine/result.md" \
    "预计挂钟：2 小时内" "" "## 关的是哪几条" "" "把前一段的结论抄进表。"
}
tail_lines() { printf '%s\n' "" "## 出口" "" "表写完。"; }
{ head_lines; echo "前一段交回在先，草稿目录 $root/k3-probe-previous/ 里有它的探针，报告 $root/k3-probe-previous/report.md 是那一段写的，只读不改。"; tail_lines; } > "$prompts/relay-comma.md"
{ head_lines; echo "前一段交回在先，那一段的草稿在 $root/k3-probe-previous/ 里，报告 $root/k3-probe-previous/report.md 是那一段写的，只读不改。"; tail_lines; } > "$prompts/relay-control.md"
{ head_lines; echo "要读的："; echo "- 草稿目录 $root/k3-probe-previous/ 下的 logs/"; echo "- 报告 $root/k3-probe-previous/report.md（前一段的）"; tail_lines; } > "$prompts/relay-list.md"
sed -e "s#^草稿目录：$root/k3-probe-mine#草稿目录在 $root/k3-probe-mine（自己建）#" -e "s#^报告：$root/k3-probe-mine/result.md#报告：$root/k3-probe-mine/report.md#" -e "/^认领：/d" "$prompts/relay-control.md" > "$prompts/own-zai.md"
sed -e "s#^报告：$root/k3-probe-mine/result.md#报告：$root/k3-probe-mine/report.md#" "$prompts/relay-control.md" > "$prompts/own-control.md"
(cd "$repo" && python3 scripts/agents/dispatch-prompt.py three-way-attack --set "草稿目录=$root/k3-probe-leg" --batch 'records/x.md#1' 2>/dev/null) \
  | sed -e 's/<填 禁读清单>/无/' -e "s/--purpose '<填[^']*'/--purpose '探针'/" -e 's#<填 背景材料>#research/prompts/_x-r1-background.md#' -e 's/<填 攻击面>/K1/' \
        -e 's/^预计挂钟：.*/预计挂钟：2 小时内/' -e '/^分片：/d' -e '/^续跑：/d' -e '/^多机：/d' -e 's/<填 逐条列[^>]*>/K1。/' -e 's/<填 做到什么[^>]*>/写完。/' > "$prompts/leg-base.md"
sed 's#^报告：.*#报告：research/prompts/x-r1-opus-output.md#' "$prompts/leg-base.md" > "$prompts/leg-filled.md"
sed 's#^报告：.*#报告：research/prompts/<轮>-opus-output.md#' "$prompts/leg-base.md" > "$prompts/leg-form-placeholder.md"
sed 's#^报告：.*#报告：research/prompts/x-r1/x-r1-opus-output.md#' "$prompts/leg-base.md" > "$prompts/leg-subdir.md"
for pair in relay-comma:clerk relay-control:clerk relay-list:clerk own-zai:clerk own-control:clerk leg-filled:three-way-attack leg-form-placeholder:three-way-attack leg-subdir:three-way-attack; do
  name="${pair%%:*}" type="${pair#*:}"
  (cd "$repo" && bash "$guard" --check-prompt "$prompts/$name.md" --type "$type" --root "$repo") > "$prompts/$name.out" 2>&1
  rc=$?
  echo "$name（$type）：退 $rc；$(grep -m1 -e '^✗' -e '✓ 派发闸' "$prompts/$name.out" | cut -c1-200)"
done
python3 - <<'PY'
import re
# 交回闸 ② 认报告路径的那一句，原样取自 .claude/hooks/handback-guard.sh 的 LEG_REPORT_MENTION
LEG_REPORT_MENTION = re.compile(r"research/prompts/[^\s`'\"，。；：、（）()「」]+-output\.md")
for message in ("报告 research/prompts/x-r1/x-r1-opus-output.md sha256 …", "报告 research/prompts/<轮>-opus-output.md"):
    print("交回闸 ② 的 LEG_REPORT_MENTION 认出：", LEG_REPORT_MENTION.findall(message))
PY
