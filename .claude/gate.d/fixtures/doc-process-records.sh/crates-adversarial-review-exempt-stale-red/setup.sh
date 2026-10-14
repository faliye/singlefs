#!/usr/bin/env bash
# 改了 crates/demo/src/comment.rs，没有判决点名；豁免表登记的是它改前那一版的哈希 ⇒ 对不上不算豁免，判红列出它。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src .claude/gate.d
printf '// 依据：已定项 3\npub fn comment() -> u32 { 1 }\n' > crates/demo/src/comment.rs
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
old="$(sha256sum crates/demo/src/comment.rs | cut -d' ' -f1)"
printf '// 依据：D3（样本） 已定项 3\npub fn comment() -> u32 { 2 }\n' > crates/demo/src/comment.rs
printf 'crates/demo/src/comment.rs\t%s\t用户豁免的是改前那一版\n' "$old" > .claude/crates-review-exempt
