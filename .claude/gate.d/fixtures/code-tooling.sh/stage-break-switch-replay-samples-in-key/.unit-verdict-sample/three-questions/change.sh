#!/usr/bin/env bash
# 复用三问的「喂的样本」改动：alpha-stage.sh 的 alpha-red 那份样本的 expect 多加一句照样对得上的 want（阶段代码一个字不动）；样本内容进键，alpha 的两条现判
set -euo pipefail
printf 'want=样本小阶段判\n' >> .claude/gate.d/fixtures/alpha-stage.sh/alpha-red/expect
