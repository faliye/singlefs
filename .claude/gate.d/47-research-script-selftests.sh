#!/usr/bin/env bash
# gate-stage: 三方论证那几个 research 脚本的自证还会红
#
# 判据：`research/scripts/ask-local-selftest.sh`、`checklist-specs.py --selftest`、`quote-kb.py --selftest`、
# `kb-sections.py --selftest`、`check-segment-registry.py --selftest`、`replace-once.py --selftest`、`replace-batch.py --selftest`、`e152-tables.py --selftest`、`agent-watch.py --selftest`、`rewrite-moved-paths.py --selftest`、`quote-rust-items.py --selftest`、`relay-timing-lint.py --selftest`、`cache-keepalive.sh --selftest`、`stale-candidates.py --selftest` 与 `--benchmark`、`test-environment-check.py --selftest`、`change-touches-crates.sh --selftest` 十七份都通过。
# 为什么：这几份自证此前都写着，却没有任何门禁阶段在跑（2026-09-12 现查 gate.d 与 .claude/scripts 零处调用）——自证只在写它的那天被跑过一次，
# 之后脚本改坏了也没人知道。2026-09-12 实测的两个坑都住在这里：
# ask-local.sh 判红时正文照样打到 stdout（一份作废输出顶着 -output-s1.md 落盘），
# 以及取法用不加引号的 $SPECS 传过 shell 被拆词（checklist-specs.py 就是为它写的）。
#
# check-segment-registry.py 不是三方论证脚本，但同一个道理成立：它调用外部真实的
# .claude/kb/layout/01-first-txn.md 与 research/results/ 产物，52 号阶段（段序列登记表
# 与 E142 产物逐字比对）没法像别的阶段那样用 .claude/gate.d/fixtures/ 隔离沙箱验证判别力，
# 只能靠它自己的 --selftest；那份 --selftest 同样要有人在门禁里替它复跑，不然只在写它的那天跑过一次。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" || exit 1

failed=0
checked=0
for runner in "bash research/scripts/ask-local-selftest.sh" "python3 research/scripts/checklist-specs.py --selftest" \
              "python3 research/scripts/quote-kb.py --selftest" "python3 research/scripts/kb-sections.py --selftest" \
              "python3 research/scripts/check-segment-registry.py --selftest" "python3 research/scripts/replace-once.py --selftest" \
              "python3 research/scripts/replace-batch.py --selftest" "python3 research/scripts/e152-tables.py --selftest" \
              "python3 research/scripts/agent-watch.py --selftest" "python3 research/scripts/rewrite-moved-paths.py --selftest" "python3 research/scripts/quote-rust-items.py --selftest" \
              "python3 research/scripts/relay-timing-lint.py --selftest" "bash research/scripts/cache-keepalive.sh --selftest" \
              "python3 research/scripts/stale-candidates.py --selftest" "python3 research/scripts/stale-candidates.py --benchmark" \
              "python3 research/scripts/test-environment-check.py --selftest" \
              "bash research/scripts/change-touches-crates.sh --selftest"; do
  checked=$((checked + 1))
  output="$($runner 2>&1)"; rc=$?
  if [[ $rc -ne 0 ]]; then
    echo "  ✗ $runner 没过（退出码 $rc）："
    printf '%s\n' "$output" | sed 's/^/    /'   # gate-lint:detail
    echo "  → 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。"
    failed=1
  fi
done
((failed)) && exit 1
echo "  ✓ research 脚本的自证都通过（查了 $checked 份：ask-local 判红分支、清单生成取法、机械整抄、小节清单、段序列登记表比对、定点替换、批量定点替换、E152 出表、子 agent 监控、搬迁路径改写、按项名抽 Rust 代码、读子进程输出的计时写法、子 agent 的缓存计时器、阶段同步的候选表与它的阳性对照、测试环境残留与宿主盘检查）"
