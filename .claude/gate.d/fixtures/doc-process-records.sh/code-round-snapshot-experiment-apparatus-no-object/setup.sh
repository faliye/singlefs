#!/usr/bin/env bash
# 角色表把 crates/singlefs-implementation-experiments 登成实验装置；crates/demo 没登记（照产品算，这份样本不改它）。
# 新写的判决只点名实验装置包 src/bin 下的一份装置 ⇒ 不是产品代码，没有对象可判，退 77、报只点名了角色不是产品的包的代码
# （弄坏开关 NON_PRODUCT_CRATES_BREAK=role-table-ignored 下它照产品判，判决没写开工快照，判红）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-implementation-experiments/src/bin crates/demo/src research/prompts .claude/gate.d
printf 'fn main() { assert!(true); }\n' > crates/singlefs-implementation-experiments/src/bin/e1_demo.rs
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf '# 样本里的角色表\nsinglefs-implementation-experiments\t实验装置\tcrates/singlefs-implementation-experiments\t样本里的实验包，只放装置二进制\n' > .claude/gate.d/crate-roles.tsv
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'fn main() { assert!(1 == 1); }\n' > crates/singlefs-implementation-experiments/src/bin/e1_demo.rs
cat > research/prompts/apparatus-r1-main-verification.md <<'MD'
# apparatus 第一轮：主 agent 核实

改动：crates/singlefs-implementation-experiments/src/bin/e1_demo.rs 的断言（实验装置包）。三条腿都没打中。
MD
git add research/prompts/apparatus-r1-main-verification.md
