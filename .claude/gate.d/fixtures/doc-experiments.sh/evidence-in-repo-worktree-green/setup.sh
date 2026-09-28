#!/usr/bin/env bash
# evidence-in-repo 格在链接 worktree 里的绿样本：现场搭法与 evidence-in-repo-worktree-red 相同
# （样本根里建主仓 .origin-repository/，把它的一个链接 worktree 整个搬到样本根）。
# 搬过来之后这一轮改了：装置 E921 改了而产物比它旧（链接 worktree 里只能比修改时刻，不判，列成本次未跑）；
# kb 里一句只说做法的 /tmp 写法（草稿放在哪）、一份新写的提示里没有 /tmp 依据（判据二照判，判绿）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=worktree-trusts-timestamps 让这一格照信修改时刻，E921 判红，这份样本判错。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
here="$PWD"
git init -q -b master .origin-repository
mkdir -p .origin-repository/research/e7-index-bench/src/bin .origin-repository/research/results .origin-repository/.claude/kb .origin-repository/research/prompts
printf 'fn main() { println!("e921"); }\n' > .origin-repository/research/e7-index-bench/src/bin/e921_worktree_stale.rs
printf 'E7RESULT name=e921 rows=1\n' > .origin-repository/research/results/e921-worktree-stale-2026-09-01.out
printf '# 笔记\n\n还没有要记的。\n\n## 历史版本\n' > .origin-repository/.claude/kb/worktree-notes.md
git -C .origin-repository add -A
git -C .origin-repository commit -qm base
git -C .origin-repository worktree add -q "$here/linked-checkout" -b linked
cp -a linked-checkout/. .
rm -rf linked-checkout
administrative_directory="$(sed -n 's/^gitdir: //p' .git)"
printf '%s\n' "$here/.git" > "$administrative_directory/gitdir"
mkdir -p .origin-repository/.git/info
printf '.origin-repository/\n' >> .origin-repository/.git/info/exclude
printf 'fn main() { println!("e921 改过了"); }\n' > research/e7-index-bench/src/bin/e921_worktree_stale.rs
printf '# 笔记\n\n草稿放在 /tmp/claude-1000/worktree-round/ 下，跑完拷进 research/results/。\n\n## 历史版本\n' > .claude/kb/worktree-notes.md
cat > research/prompts/worktree-r1-opus.md <<'EOF'
# worktree-r1 云端攻方

1. 背景材料 `research/prompts/_worktree-r1-background.md`。
EOF
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e921-worktree-stale-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 research/e7-index-bench/src/bin/e921_worktree_stale.rs
