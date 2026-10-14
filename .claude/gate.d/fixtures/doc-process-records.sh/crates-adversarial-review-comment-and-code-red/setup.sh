#!/usr/bin/env bash
# 一份只改注释（摘出去）、一份改了代码、一份改了文档注释里的 doctest（照代码算），都没有判决 ⇒ 红，只列后两份。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src .claude/gate.d
printf '// 旧说明\npub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn code() -> &'"'"'static str { "a//b" }\n' > crates/demo/src/code.rs
printf '/// ```\n/// assert_eq!(demo::doc(), 1);\n/// ```\npub fn doc() -> u32 { 1 }\n' > crates/demo/src/doctest.rs
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '// 新说明\npub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn code() -> &'"'"'static str { "a//c" }\n' > crates/demo/src/code.rs
printf '/// ```\n/// assert_eq!(demo::doc(), 2);\n/// ```\npub fn doc() -> u32 { 1 }\n' > crates/demo/src/doctest.rs
