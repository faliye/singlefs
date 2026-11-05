#!/usr/bin/env bash
# 红样本（别名导入）：基准提交里有 验证包的清单与两个装置 bin：
#   e990_judge_sample 写 use singlefs_verification::crash_amplification as amplification 之后调 amplification::run_from_environment（源文件里没有「crash_amplification::」这串字）；
#   e990_counts_only 只从 crash_state_enumeration:: 取崩溃点（crash_points_of_the_steps），不调判段入口。
# 提交之后新建 e990 的跑前登记：围栏里两条 cargo run 都没带 gpu。别名导入那一条按判段入口认出走判段、判红（第 6 行）；只取崩溃点那一条不走判段、不报（第 7 行）。
# 出处：攻方模型 research/prompts/d137-ledger-defs-r1-opus-model/ 的 alias-import-judge 与 marker-without-judge 两个世界（改前按「crash_amplification::」记号认：前一条漏判、后一条误红）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p crates/singlefs-verification/src/bin research/prompts
printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > crates/singlefs-verification/Cargo.toml
printf 'use singlefs_verification::crash_amplification as amplification;\n\nfn main() {\n    amplification::run_from_environment();\n}\n' > crates/singlefs-verification/src/bin/e990_judge_sample.rs
printf 'use singlefs_verification::crash_state_enumeration::crash_points_of_the_steps;\n\nfn main() {\n    let steps = crash_points_of_the_steps;\n    let _ = steps;\n}\n' > crates/singlefs-verification/src/bin/e990_counts_only.rs
printf '# 样本说明\n' > research/prompts/README.md
git add -A
git commit -qm '基准：验证包的清单与两个装置 bin'
cat > research/prompts/e990-preregistration.md <<'EOF'
# E990 样本登记（别名导入）

## 五、起法

```bash
task-server/task submit -- cargo run --release -p singlefs-verification --bin e990_judge_sample -- feasibility
task-server/task submit -- cargo run --release -p singlefs-verification --bin e990_counts_only -- counts
```
EOF
