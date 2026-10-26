#!/usr/bin/env bash
# 派发闸 ④（写范围，认「报告路径 / 报告写进 / 报告写到」）与 ⑰（报告文件名，只认「报告：」）认的标签互不相交：三条提示各喂一次 --check。
#   甲：实现员骨架，「报告：」填 research/prompts/ 下的文件（实现员写范围之外）——④ 不认「报告：」，看放不放；
#   乙：同一份，标签改成「报告写进 」——对照，④ 要拒；
#   丙：clerk 骨架，「报告：」改成「报告路径：…/report.md」——⑰ 不认「报告路径：」，看放不放。
# 用法：report_label_probe.sh <仓根> <草稿目录> <模型目录>
set -uo pipefail
repo="$1"; work="$2"; model="$3"
cd "$repo"
batch='records/2026-10-22-子agent结果落地审计.md#4'
python3 scripts/agents/dispatch-prompt.py implementation-writer --set 草稿目录="$work/probe-writer" --set 认领用途=探针 --batch "$batch" \
  --set 报告=research/prompts/probe-impl-result.md --out "$work/probe-writer-raw.md" > /dev/null
python3 "$model/tools/fill_skeleton.py" "$work/probe-writer-raw.md" "$work/probe-writer.md"
sed 's#^报告：research/prompts/probe-impl-result.md#报告写进 research/prompts/probe-impl-result.md#' "$work/probe-writer.md" > "$work/probe-writer-control.md"
python3 scripts/agents/dispatch-prompt.py clerk --set 草稿目录="$work/probe-clerk" --set 认领用途=探针 --batch "$batch" --out "$work/probe-clerk-raw.md" > /dev/null
python3 "$model/tools/fill_skeleton.py" "$work/probe-clerk-raw.md" "$work/probe-clerk-filled.md"
sed 's#^报告：\(.*\)/result.md#报告路径：\1/report.md#' "$work/probe-clerk-filled.md" > "$work/probe-clerk.md"
for probe in probe-writer probe-writer-control probe-clerk; do
  echo "== $probe：$(grep -m1 '^报告' "$work/$probe.md")"
  python3 scripts/agents/dispatch-prompt.py --check "$work/$probe.md" | head -2 | cut -c1-160
  echo "check-rc=${PIPESTATUS[0]}"
done
