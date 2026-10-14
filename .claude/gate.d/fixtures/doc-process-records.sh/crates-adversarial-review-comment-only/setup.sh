#!/usr/bin/env bash
# 两份产品代码都只改了注释（一份改行注释与行尾注释，一份改文档注释的文字），没有判决 ⇒ 都不走三方，这一格无对象可判、逐个报出来。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src .claude/gate.d
printf '// 旧说明\npub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf '/// 旧文档\npub fn named() -> u32 { 1 }\n' > crates/demo/src/named.rs
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '// 新说明，写清为什么\npub fn publish() -> u32 { 1 } // 行尾补一句\n' > crates/demo/src/lib.rs
printf '/// 新文档：返回什么、什么时候 panic\npub fn named() -> u32 { 1 }\n' > crates/demo/src/named.rs
