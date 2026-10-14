#!/usr/bin/env bash
# 四份新判决都点名了产品代码文件 crates/demo/src/lib.rs，各缺一样 ⇒ 本该判红，逐份点名缺的：
#   nomention-r1   快照目录在树里、清单也齐，但判决正文一个字没提快照目录（判据 ①）
#   nodir-r1       正文写了 research/prompts/nodir-r1-snapshot/，那个目录不在树里（判据 ②）
#   nomanifest-r1  目录在、正文也写了，里面只有 kb-sha256.txt，没有 crates-sha256.txt（判据 ②）
#   badhash-r1     清单里点名了 lib.rs，但哈希只有 6 位十六进制（判据 ③）
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src research/prompts
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
for round in nomention-r1 nodir-r1 nomanifest-r1 badhash-r1; do
  printf '# %s：主 agent 核实\n\n' "$round" > "research/prompts/$round-main-verification.md"
done
mkdir research/prompts/nomention-r1-snapshot
sha256sum crates/demo/src/lib.rs > research/prompts/nomention-r1-snapshot/crates-sha256.txt
printf '改动：crates/demo/src/lib.rs 的 publish 返回 2。三条腿都没打中。\n' >> research/prompts/nomention-r1-main-verification.md
printf '开工快照：research/prompts/nodir-r1-snapshot/。\n改动：crates/demo/src/lib.rs 的 publish 返回 2。\n' >> research/prompts/nodir-r1-main-verification.md
mkdir research/prompts/nomanifest-r1-snapshot
: > research/prompts/nomanifest-r1-snapshot/kb-sha256.txt
printf '开工快照：research/prompts/nomanifest-r1-snapshot/。\n改动：crates/demo/src/lib.rs 的 publish 返回 2。\n' >> research/prompts/nomanifest-r1-main-verification.md
mkdir research/prompts/badhash-r1-snapshot
printf 'abc123  crates/demo/src/lib.rs\n' > research/prompts/badhash-r1-snapshot/crates-sha256.txt
printf '开工快照：research/prompts/badhash-r1-snapshot/。\n改动：crates/demo/src/lib.rs 的 publish 返回 2。\n' >> research/prompts/badhash-r1-main-verification.md
