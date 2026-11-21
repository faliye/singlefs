#!/usr/bin/env bash
# 注释历史判法「」里的三方腿角色名那一条，在一个样本根上照两臂各列一次命中：
#   current：不设开关，判法此刻的样子（收尾了的「」里的角色名换成占位符再判）；
#   quoted-role-names-judged：判法自己的弄坏开关，「」里的角色名照历史词判（样本根带 .gate-break-sample，判法只在这样的根上认开关）。
# 样本文件六行注释，各行是什么写在 SAMPLE_LINES 旁边；输出 name= 行，逐行写两臂各列没列。
# 用法：quoted-role-name-probe.sh <仓根> <输出目录>；退 0 跑完，退 2 用法错，别的是判法自己的退出码（输出照打）。
set -uo pipefail
if (($# != 2)); then
  echo "用法：quoted-role-name-probe.sh <仓根> <输出目录>" >&2
  exit 2
fi
repo="$1"
out="$2"
checker="${repo:?}/scripts/gate/comment-history-check.py"
[[ -f "$checker" ]] || { echo "  ✗ 没有 $checker" >&2; exit 2; }
root="${out:?}/comment-history-sample-root"
mkdir -p "$root/scripts/gate"
: > "$root/.gate-break-sample"
sample="$root/scripts/gate/quoted-role-name-sample.sh"
# 第 2 到 4 行：来历写在「」外面，「」里只有角色名；第 5 行：「」里另有轮次；第 6 行：照原文引的格名；第 7 行：不带「」的角色名
cat > "$sample" <<'SAMPLE'
#!/usr/bin/env bash
# 这一支补的是「攻方腿」打中的那一格
# 「辩方腿」指出这里漏判，补了这一判
# 「正推腿」报的那一条照改
# 「第二轮攻方腿」打中的
# 「Bash写范围:三方腿:正推腿写别的轮的报告，拒」
# 正推腿报的那一条
echo ok
SAMPLE
status=0
for arm in current quoted-role-names-judged; do
  listing="$out/comment-history-$arm.txt"
  if [[ "$arm" == current ]]; then
    python3 "$checker" --list "$root" > "$listing" 2>&1
  else
    COMMENT_HISTORY_CHECK_BREAK=quoted-role-names-judged python3 "$checker" --list "$root" > "$listing" 2>&1
  fi
  rc=$?
  ((rc == 0)) || status=$rc
  listed=""
  for line in 2 3 4 5 6 7; do
    if grep -q "quoted-role-name-sample.sh:$line:" "$listing"; then listed+="$line,"; fi
  done
  echo "name=quoted_role_names arm=$arm checker_exit=$rc listed_lines=${listed%,}"
done
exit "$status"
