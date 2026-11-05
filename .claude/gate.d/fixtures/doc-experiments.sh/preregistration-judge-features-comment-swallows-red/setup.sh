#!/usr/bin/env bash
# 红样本（以反斜杠收尾的注释行吞掉下一条命令）：基准提交里有 验证包的清单与一个走判段的装置 bin（e990_judge_sample 调判段入口 crash_amplification::run_from_environment）；
# 提交之后新建 e990 的跑前登记：围栏里头一行是以反斜杠收尾的 shell 注释，下一行起 e990_judge_sample、没带 gpu。bash 里注释不续行，下一行照样单独跑：
# 门禁同样不并、单独认第 7 行，判红。
# 出处：攻方模型 research/prompts/d137-ledger-defs-r2-opus-model/ 的 comment-backslash-swallows-command 世界（改前把两行并成以 # 起头的一条、不在命令位置，退 77）。
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
# E990 样本登记（comment-backslash-swallows-command）

## 五、起法

```bash
# 可行性档，下一行起，没带 gpu 特性 \
task-server/task submit -- cargo run --release -p singlefs-verification --bin e990_judge_sample -- feasibility
```
EOF
