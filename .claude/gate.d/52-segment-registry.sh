#!/usr/bin/env bash
# gate-stage: 段序列登记表与 E142 产物逐字比对
#
# 判据（C316（提交步骤的登记位有四处且互不相同） 欠账的第①半）：
#   .claude/kb/layout/01-first-txn.md 「八、根槽写路径的段序列登记表」里每一行的段序列数字串
#   （`4+1+1+1+2` 这种）都是人从 E142（第一个事务的干跑） 产物里抄过来的——抄错一位，
#   或者产物重跑之后表没跟着改，此前没有任何东西会报警。
#
# 做法：转发给 research/scripts/check-segment-registry.py，逻辑不写第二份
#   （表怎么解析、path 怎么从「出处」栏原样抠出来、第四条产物路径怎么按排除法认领，
#   都写在那个脚本自己的文档字符串与注释里）。
#
# 该脚本自己的 --selftest（改坏拷贝里的一个段序列数字，确认判红；未改动的拷贝确认判绿）
# 由 47 号阶段（三方论证 research 脚本的自证）复跑，这里不重复跑一遍。
#
# ⚠️ 脚本按**本阶段自己的位置**取仓里的那一份，不按 cwd 取；`--root` 指被判的仓。判别力样本会把 cwd 换成样本目录，
#   按 cwd 取就成了样本里那份拷贝：仓里的脚本退化了（例如 main() 不再传退出码），样本照样判对，这一道与 47 号的
#   --selftest 一起放过（三方判决 research/prompts/gate-fix-forks-r1-main-verification.md 的 T6）。同一个坑见 31 号头部。
# 样本：fixtures/52-segment-registry.sh 只放脚本读的三样合成输入——layout 表的「八、」一节（一行表、一句 path 声明、
#   一句整条流）、replay.sh 里 E142 那一行、产物里几行 name=segments。red 十行各埋一种错（段序列、种类、操作数、状态数、
#   产物里没有这条 path、没写段序列、产物这一行没有 kinds、钉住的用例文件不在、用例里没有那个函数、出处既无 path 也无用例），
#   另把整条流的状态数写错、第二条流的写数与数组与用例里的状态数各错一处，脚本每条逐行比对的分支都有一处会红，逐条点名；
#   green 一致，判绿（三方判决 gate-fix-forks-r3 的 T6）。「声明的 path 集合剩不下恰好一条」走退 2 那一支，放不进这份红样本，没有样本格。
#   钉活代码的那几句与真产物的解析由脚本自己的 --selftest（47 号跑）拿真文件测，样本不再测一遍。
#
#   bash .claude/gate.d/52-segment-registry.sh [仓根]
set -uo pipefail
# 阶段自己的位置在 cd 之前取：用相对路径调本阶段、又另给项目根时，cd 之后 $(dirname "$0") 就解析不到了
REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
ROOT="${1:-$REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2
SCRIPT="$REPOSITORY/research/scripts/check-segment-registry.py"
# 脚本随仓走，不在就是被删了或挪了——退 1 不退 77，与 57、70、73、77 号同一条（三方判决 gate-fix-forks-r1 的 T7）
[[ -f "$SCRIPT" ]] || {
  echo "  ✗ 找不到 $SCRIPT"
  echo "     → 怎么办：它随仓走（research/scripts/ 下），不在就是被删了或挪了：从 git 找回来，挪了就改这一行的路径。"
  exit 1
}

python3 "$SCRIPT" --root "$ROOT" || exit 1
