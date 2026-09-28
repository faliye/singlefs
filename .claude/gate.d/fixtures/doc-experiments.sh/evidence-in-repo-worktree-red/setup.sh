#!/usr/bin/env bash
# evidence-in-repo 格在链接 worktree 里的红样本（research/scripts/gate-staged.sh 与 --staged 的临时树就是链接 worktree）：
# 样本根里建一个主仓 .origin-repository/，给它加一个链接 worktree，再把那个 worktree 整个搬到样本根，
# 于是样本根的 git rev-parse --git-dir 落在 .origin-repository/.git/worktrees/ 下。
# 搬过来之后这一轮改了：kb 里把还清依据写成 /tmp 下的报告、新写的提示里把实现员报告引成 /tmp 下的文件（判据二，不看时间戳，必须判红）；
# 装置 E920 改了而产物比它旧（判据一里只能比修改时刻的那一种：链接 worktree 里不判，列成本次未跑）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=worktree-skips-whole-cell 让这一格整格退 77，这份样本判错。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
here="$PWD"
git init -q -b master .origin-repository
mkdir -p .origin-repository/research/e7-index-bench/src/bin .origin-repository/research/results .origin-repository/.claude/kb .origin-repository/research/prompts
printf 'fn main() { println!("e920"); }\n' > .origin-repository/research/e7-index-bench/src/bin/e920_worktree_stale.rs
printf 'E7RESULT name=e920 rows=1\n' > .origin-repository/research/results/e920-worktree-stale-2026-09-01.out
printf '# 欠账\n\n| C920 | 样本欠账 | 还没还 |\n\n## 历史版本\n' > .origin-repository/.claude/kb/worktree-owed.md
git -C .origin-repository add -A
git -C .origin-repository commit -qm base
git -C .origin-repository worktree add -q "$here/linked-checkout" -b linked
cp -a linked-checkout/. .
rm -rf linked-checkout
administrative_directory="$(sed -n 's/^gitdir: //p' .git)"
printf '%s\n' "$here/.git" > "$administrative_directory/gitdir"
mkdir -p .origin-repository/.git/info
printf '.origin-repository/\n' >> .origin-repository/.git/info/exclude
printf 'fn main() { println!("e920 改过了"); }\n' > research/e7-index-bench/src/bin/e920_worktree_stale.rs
cat > .claude/kb/worktree-owed.md <<'EOF'
# 欠账

| C920 | 样本欠账 | 2026-09-18 还清，依据：/tmp/claude-1000/worktree-round/report.md |

## 历史版本
EOF
cat > research/prompts/worktree-r1-opus.md <<'EOF'
# worktree-r1 云端攻方

1. 背景材料 `research/prompts/_worktree-r1-background.md`。
2. 实现员报告 `/tmp/claude-1000/worktree-impl/report.md`（它自己列的设计判断）。
EOF
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e920-worktree-stale-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 research/e7-index-bench/src/bin/e920_worktree_stale.rs
