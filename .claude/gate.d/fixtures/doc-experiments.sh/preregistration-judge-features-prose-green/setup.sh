#!/usr/bin/env bash
# 绿样本（行文内嵌那一形）：基准提交里有 验证包的清单与一个走判段的装置 bin（e993_judge_sample 调判段入口 crash_amplification::run_from_environment）；
# 提交之后新建 e993 的跑前登记：围栏里带范围声明经 task submit 交的那一条带了 gpu，判了 1 条；起同一个 bin 而没带 gpu 的四处都不在命令位置：
# 围栏里 ✓ 起头的输出行、列表里标签之后的、表格里夹着字的格、内存一节行文里内嵌的（真仓 e173-r1-prereg.md 第 697 行那一形），四处都不判。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p crates/singlefs-verification/src/bin research/prompts
printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > crates/singlefs-verification/Cargo.toml
printf 'use singlefs_verification::crash_amplification::run_from_environment;\n\nfn main() {\n    run_from_environment();\n}\n' > crates/singlefs-verification/src/bin/e993_judge_sample.rs
printf '# 样本说明\n' > research/prompts/README.md
git add -A
git commit -qm '基准：验证包的清单与一个走判段的装置 bin'
cat > research/prompts/e993-preregistration.md <<'EOF'
# E993 样本登记

## 五、起法

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=e993 的可行性档' task-server/task submit -- cargo run -q --release -p singlefs-verification --features gpu --bin e993_judge_sample -- feasibility
✓ 编过（cargo test --no-run -p singlefs-verification --bin e993_judge_sample）
```

- 可行性档的旧写法：`cargo run --release -p singlefs-verification --bin e993_judge_sample`

| 跑法 | 说明 |
|---|---|
| 内联测试 | 照 `cargo test --release -p singlefs-verification --bin e993_judge_sample` 起，带上 gpu |

## 十二、内存

上限 40G 的推法（推的）：36G 下 `cargo run --release -p singlefs-verification --bin e993_judge_sample` 的 352 秒没撞上限。
EOF
