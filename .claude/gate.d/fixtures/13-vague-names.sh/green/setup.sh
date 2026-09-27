#!/usr/bin/env bash
# 绿样本现场：只有带领域词的名字，和一个 trait 规定的方法 get（在 impl <trait> for <类型> 块里，不判）。
set -e
mkdir -p .claude crates/demo/src
printf 'run  # 没说跑的是什么\ncell  # 没说是哪一格\nget  # 没说取的是什么\n' > .claude/naming-vague-words
printf '# 样本：没有要排除的\n' > .claude/naming-vague-exclude
printf 'trait Store { fn get(&self) -> u64; }\nstruct SlotStore;\nimpl Store for SlotStore {\n    fn get(&self) -> u64 { 0 }\n}\nfn evaluate_stream_domain() {}\nfn main() { evaluate_stream_domain(); }\n' > crates/demo/src/main.rs
git init -q . && git add -A
