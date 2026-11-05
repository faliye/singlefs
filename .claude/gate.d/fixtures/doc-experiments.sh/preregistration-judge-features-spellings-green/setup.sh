#!/usr/bin/env bash
# 绿样本（特性的另两种写法）：基准提交里有 验证包的清单与一个走判段的装置 bin（e990_judge_sample 调判段入口 crash_amplification::run_from_environment）；
# 提交之后新建 e990 的跑前登记：围栏里两条走判段的命令，一条写 -Fgpu（-F 紧贴值），一条写 --features "lookup-work-counters gpu"（引号里两个特性用空格隔开），
# 都算带了 gpu，判了 2 条、判绿。出处：d137-ledger-defs-r2 辩方报告的 C1、C2（改前按空白切词：-Fgpu 不认、引号里的 gpu" 切丢，两条都误红）。
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
# E990 样本登记（-Fgpu 与带引号的多特性）

## 五、起法

```bash
task-server/task submit -- cargo run --release -p singlefs-verification -Fgpu --bin e990_judge_sample -- feasibility
task-server/task submit -- cargo run --release -p singlefs-verification --features "lookup-work-counters gpu" --bin e990_judge_sample -- control
```
EOF
