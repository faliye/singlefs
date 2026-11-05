#!/usr/bin/env bash
# 红样本（围栏外续行拆 --bin）：基准提交里有 验证包的清单与一个走判段的装置 bin（e990_judge_sample 调判段入口 crash_amplification::run_from_environment）；
# 提交之后新建 e990 的跑前登记：列表项下面缩进的一行是命令（不在围栏里），以反斜杠续行，--bin 拆到下一行、没带 gpu。两行并成一条认出 --bin、判红，行号记头一行（第 6 行）。
# 出处：d137-ledger-defs-r2 辩方报告的 C3（改前只并围栏里的续行：头一行没有 --bin、下一行没有 cargo，只报「没点名 --bin 的 1 条」、退 77）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p crates/singlefs-verification/src/bin research/prompts
printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > crates/singlefs-verification/Cargo.toml
printf 'use singlefs_verification::crash_amplification::run_from_environment;\n\nfn main() {\n    run_from_environment();\n}\n' > crates/singlefs-verification/src/bin/e990_judge_sample.rs
printf '# 样本说明\n' > research/prompts/README.md
git add -A
git commit -qm '基准：验证包的清单与一个走判段的装置 bin'
cat > research/prompts/e990-preregistration.md <<'EOF'
# E990 样本登记（围栏外续行拆 --bin）

## 五、起法

- 可行性档：
  task-server/task submit -- cargo run --release -p singlefs-verification \
    --bin e990_judge_sample -- feasibility
EOF
