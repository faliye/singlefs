#!/usr/bin/env bash
# 红样本（表格整格是命令，格里行尾注释写着 --features gpu）：d137-ledger-defs-r3 攻方模型 research/prompts/d137-ledger-defs-r3-opus-model/ 的
# prereg-table-cell-trailing-comment 世界原样做成。基准提交里有 验证包的清单与一个走判段的装置 bin（e990_judge_sample 调判段入口 crash_amplification::run_from_environment）；
# 提交之后新建 e990 的跑前登记：表格一格整格是反引号里的命令，没带 gpu，反引号里行尾 shell 注释写着「以后加 --features gpu」。
# bash 不把注释交给 cargo：截掉注释之后没带 gpu，第 7 行判红（改前只截围栏里的注释，注释里的 --features gpu 当成带了，判绿）。
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
# E990 样本登记（表格整格是命令，格里行尾注释写着特性）

## 五、起法

| 档 | 起法 |
|---|---|
| 可行性档 | `task-server/task submit -- cargo run --release -p singlefs-verification --bin e990_judge_sample  # 以后加 --features gpu` |
EOF
