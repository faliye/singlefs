#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 判决里「## 这一轮被判过的文件」一节按路径列出了三份改过的产品代码 ⇒ 本该判绿。节里的列法各不相同：
#   lib.rs 写在三级小标题底下的列表项里（节到下一个一、二级标题为止，三级小标题留在节里），标题行尾带空白也认；
#   table.rs 写在表格行里；fenced.rs 写在节里的代码围栏里（围栏里的路径算在节里）。
# 判决开头另有一段代码围栏，围栏里也有一行同样的标题：围栏里的标题不开节、不结束节，围栏外正文再提 lib.rs 也不影响。
mkdir -p crates/demo/src research/prompts .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn table() -> u32 { 1 }\n' > crates/demo/src/table.rs
printf 'pub fn fenced() -> u32 { 1 }\n' > crates/demo/src/fenced.rs
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf 'pub fn table() -> u32 { 2 }\n' > crates/demo/src/table.rs
printf 'pub fn fenced() -> u32 { 2 }\n' > crates/demo/src/fenced.rs
{
  printf '# demo 第一轮：主 agent 核实\n\n写法示例（围栏里的标题行不算标题）：\n\n'
  printf '```\n## 这一轮被判过的文件\n```\n\n'
  printf '## 一、证据核对\n\n三条腿都没打中。\n\n'
  printf '## 这一轮被判过的文件  \n\n### 产品代码\n\n- crates/demo/src/lib.rs：publish 返回 2\n\n'
  printf '### 其余\n\n| 文件 | 判了什么 |\n|---|---|\n| crates/demo/src/table.rs | table 返回 2 |\n\n'
  printf '```text\ncrates/demo/src/fenced.rs\n```\n\n'
  printf '## 回看\n\n正文别处再提 crates/demo/src/lib.rs 不影响上面那一节。\n'
} > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md
