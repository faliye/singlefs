#!/usr/bin/env bash
# 攻方腿 m3-crash-enumeration-merge-r1 的模型复跑。
# 第一件（谁都能跑）：K5 计数模型，纯算术，末行 MODEL_SELFCHECK compared=4 mismatched=0、退 0。
# 第二件（交主 agent 定跑不跑）：closure_probe.rs 是一个链 singlefs-checker-tier 库的小程序，只算判法闭包与几条用例的闭包、不判一个状态；
#   攻方腿在 merge-probe 包里跑过一次，之后被自动模式分类器判成绕开「子 agent 不跑 checker 档」而拒了读输出与再编译，
#   起法写在报告「要主 agent 跑的命令」一节，这里不起。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
nice -n 19 python3 "$here/count_model.py"
