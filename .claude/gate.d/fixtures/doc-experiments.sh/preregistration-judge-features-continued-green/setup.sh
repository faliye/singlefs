#!/usr/bin/env bash
# 绿样本（续行拆 --features gpu）：基准提交里有 验证包的清单与一个走判段的装置 bin（e990_judge_sample 调判段入口 crash_amplification::run_from_environment）；
# 提交之后新建 e990 的跑前登记：围栏里经 task submit 交的 cargo run，--bin 在头一行、--features gpu 用反斜杠续行拆到下一行。两行并成一条认出 gpu、判绿，行号记头一行（第 6 行）。
# 出处：攻方模型 research/prompts/d137-ledger-defs-r1-opus-model/ 的 continued-features-on-next-line 世界（改前逐行认：头一行认出走判段的 bin、特性是空的，误红）。
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
# E990 样本登记（续行拆 --features gpu）

## 五、起法

```bash
task-server/task submit -- cargo run --release -p singlefs-verification --bin e990_judge_sample \
  --features gpu -- feasibility
```
EOF
