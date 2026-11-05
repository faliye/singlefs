#!/usr/bin/env bash
# 红样本（只点名 --test 的 cargo test）：基准提交里有 验证包的清单与一个集成测试目标 tests/crash_judge_sample.rs（调判段入口 crash_amplification::run_from_environment）；
# 提交之后新建 e990 的跑前登记：围栏里 cargo test 只点名 --test crash_judge_sample、没带 gpu。测试目标按 tests/<名>.rs 认出源文件，走判段，第 6 行判红。
# 出处：d137-ledger-defs-r2 辩方报告的 C4（改前只认 --bin，这一条只报「没点名 --bin 的 1 条」、退 77）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p crates/singlefs-verification/tests research/prompts
printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > crates/singlefs-verification/Cargo.toml
printf 'use singlefs_verification::crash_amplification::run_from_environment;\n\n#[test]\nfn judging_sample_runs_the_judge() {\n    run_from_environment();\n}\n' > crates/singlefs-verification/tests/crash_judge_sample.rs
printf '# 样本说明\n' > research/prompts/README.md
git add -A
git commit -qm '基准：验证包的清单与一个走判段的测试目标'
cat > research/prompts/e990-preregistration.md <<'EOF'
# E990 样本登记（只点名 --test）

## 五、起法

```bash
task-server/task submit -- cargo test --release -p singlefs-verification --test crash_judge_sample
```
EOF
