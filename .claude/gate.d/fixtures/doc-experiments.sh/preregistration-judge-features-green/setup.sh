#!/usr/bin/env bash
# 绿样本：基准提交里有 验证包的清单与两个装置 bin（e992_judge_sample 调判段入口 crash_state_enumeration::enumerate_crash_states_with_expansion，e992_counts_only 不调），
# 与装置包 scripts/experiment-apparatus/Cargo.toml（[[bin]] e992-apparatus-judge 指到调 crash_amplification::GivenStatesEngine 的源文件）；
# 提交之后新建 e992 的跑前登记：三条走判段的命令各用 --features lookup-work-counters,gpu、--features=gpu、-F gpu（头一条 -- 之后给二进制的「--bin e992_counts_only」不当 cargo 的参数），
# 不走判段的 e992_counts_only 不带特性、不判；e992_gone 在清单里找不到，报本次未判；cargo test --lib 没点名 --bin，只报数。判绿报「走判段的 3 条」。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p crates/singlefs-verification/src/bin scripts/experiment-apparatus research/prompts
printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > crates/singlefs-verification/Cargo.toml
printf 'use singlefs_verification::crash_state_enumeration::enumerate_crash_states_with_expansion;\n\nfn main() {\n    enumerate_crash_states_with_expansion();\n}\n' > crates/singlefs-verification/src/bin/e992_judge_sample.rs
printf 'fn main() {\n    println!("只计数，不进判段");\n}\n' > crates/singlefs-verification/src/bin/e992_counts_only.rs
cat > scripts/experiment-apparatus/Cargo.toml <<'EOF'
[package]
name = "e7-index-bench"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "e992-apparatus-judge"
path = "e992_apparatus_judge.rs"
EOF
printf 'use singlefs_verification::crash_amplification::GivenStatesEngine;\n\nfn main() {\n    GivenStatesEngine::start_from_environment();\n}\n' > scripts/experiment-apparatus/e992_apparatus_judge.rs
printf '# 样本说明\n' > research/prompts/README.md
git add -A
git commit -qm '基准：验证包与装置包的清单与三个装置 bin'
cat > research/prompts/e992-preregistration.md <<'EOF'
# E992 样本登记

## 五、起法

- `cargo run -q --release -p singlefs-verification --features lookup-work-counters,gpu --bin e992_judge_sample -- feasibility --bin e992_counts_only`（可行性档）
- `cargo test --release -p singlefs-verification --features=gpu --bin e992_judge_sample`（内联测试）
- `cargo run --release -p e7-index-bench -F gpu --bin e992-apparatus-judge`（装置包里的判段）
- `cargo run --release -p singlefs-verification --bin e992_counts_only`（计数档）
- `cargo run --release -p singlefs-verification --bin e992_gone`（旧装置）
- `cargo test --release -p singlefs-verification --lib`（整包单测）
EOF
