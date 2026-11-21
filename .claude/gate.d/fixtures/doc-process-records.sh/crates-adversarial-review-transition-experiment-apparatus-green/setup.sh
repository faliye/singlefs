#!/usr/bin/env bash
# 角色表把 crates/singlefs-implementation-experiments 登成实验装置；过渡挂账表在基准里登了它 src/bin 下的一份装置，工作区改了这一份、没有判决
# ⇒ 实验装置包的 src 是非产品代码、挂账表认这一行，这一格不要求点名，判绿报 1 个挂账与登记的理由
# （弄坏开关 NON_PRODUCT_CRATES_BREAK=role-table-ignored 下这一行的路径不是挂账表认的那几类，判红）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-implementation-experiments/src/bin .claude/gate.d
printf 'fn main() { assert!(true); }\n' > crates/singlefs-implementation-experiments/src/bin/e1_demo.rs
printf '# 样本里的角色表\nsinglefs-implementation-experiments\t实验装置\tcrates/singlefs-implementation-experiments\t样本里的实验包，只放装置二进制\n' > .claude/gate.d/crate-roles.tsv
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf '# 样本里的过渡挂账表\ncrates/singlefs-implementation-experiments/src/bin/e1_demo.rs\t存量实验装置还没补三方\n' > .claude/gate.d/review-test-code-pending.tsv
git add -A && git commit -qm base
printf 'fn main() { assert!(1 == 1); }\n' > crates/singlefs-implementation-experiments/src/bin/e1_demo.rs
