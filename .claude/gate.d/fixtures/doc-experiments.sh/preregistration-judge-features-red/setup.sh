#!/usr/bin/env bash
# 红样本：基准提交里有 验证包的清单与两个装置 bin（e990_judge_sample 调判段入口 crash_amplification::run_from_environment，e990_counts_only 不调）；
# 提交之后新建两份跑前登记：e990 列表行首的 cargo run 起 e990_judge_sample 没带特性，e991 第 1 次重跑登记的表格里整格的 cargo test 带了 lookup-work-counters 没带 gpu，两处判红；
# 同一份里起 e990_counts_only（不走判段）不带特性的一条不报。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p crates/singlefs-verification/src/bin research/prompts
printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > crates/singlefs-verification/Cargo.toml
printf 'use singlefs_verification::crash_amplification::run_from_environment;\n\nfn main() {\n    run_from_environment();\n}\n' > crates/singlefs-verification/src/bin/e990_judge_sample.rs
printf 'fn main() {\n    println!("只计数，不进判段");\n}\n' > crates/singlefs-verification/src/bin/e990_counts_only.rs
printf '# 样本说明\n' > research/prompts/README.md
git add -A
git commit -qm '基准：验证包的清单与两个装置 bin'
cat > research/prompts/e990-preregistration.md <<'EOF'
# E990 样本登记

## 五、起法

- `cargo run -q --release -p singlefs-verification --bin e990_judge_sample -- feasibility`（可行性档）
- `cargo run --release -p singlefs-verification --bin e990_counts_only`（计数档）
EOF
cat > research/prompts/e991-r1-prereg.md <<'EOF'
| 跑法 | 命令 | 备注 |
|---|---|---|
| 内联测试 | `cargo test --release -p singlefs-verification --features lookup-work-counters --bin e990_judge_sample` | 带了计数特性没带 gpu |
EOF
