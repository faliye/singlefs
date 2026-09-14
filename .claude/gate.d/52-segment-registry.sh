#!/usr/bin/env bash
# gate-stage: 段序列登记表与 E142 产物逐字比对
#
# 判据（C316（提交步骤的登记位有四处且互不相同） 欠账的第①半）：
#   .claude/kb/first-txn-layout.md 「八、根槽写路径的段序列登记表」里每一行的段序列数字串
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
# ⚠️ 本阶段没有 .claude/gate.d/fixtures/ 判别力样本：它转发的检查依赖
#   .claude/kb/first-txn-layout.md 与 research/results/ 下的真实产物，这两样在 89 号阶段
#   （项目本地阶段自检）的隔离沙箱里不存在——与 87-replay.sh、47-research-script-selftests.sh
#   同理，这两个阶段同样转发外部 research/scripts/ 下的真实脚本，也同样没有样本。
#   89 号阶段会把本阶段显式列成「未自检」，这是如实反映、不是漏做
#   （rules/show-me-test.md「门禁不许假装通过」）。
#
#   bash .claude/gate.d/52-segment-registry.sh [仓根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -f research/scripts/check-segment-registry.py ]] || { echo "  ! 找不到 research/scripts/check-segment-registry.py，本阶段跳过"; exit 77; }

python3 research/scripts/check-segment-registry.py --root "$ROOT" || exit 1
