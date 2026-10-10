#!/usr/bin/env bash
# 复跑第 37 批代码轮第一轮攻方腿（Opus）的模型：纯 Python，不编 Rust、不起 cargo、不跑任何 checker 档测试。
# 用法：bash tools/rerun.sh [冻结副本根]   （默认 ~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree）
# 输出与报告「复跑」一节贴的原样输出逐字比；源码里模型依赖的那几行对不上时脚本退 3（模型作废）。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
root="${1:-$HOME/.local/share/singlefs/work/m3-formal-code-b37-r1/tree}"
nice -n 19 python3 -I "$here/b1_profile_union.py" "$root" "$root/crates/mutations.tsv"
