#!/usr/bin/env bash
# 本该判红，一份样本同时放四种坏法：装置改了而产物比它旧；变异表改了而这个实验号一份产物都没有；
# kb 里把还清依据写成 /tmp 下的报告；这一轮新写的提示里把实现员报告引成 /tmp 下的文件；
# 另有一个装置注释与代码一起改（E907），不许按只改注释放过。
# 时间戳一律用 Z 写死，免得跟着跑门禁那台机器的时区漂。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p research/e7-index-bench/src/bin research/mutations research/results .claude/kb research/prompts
printf 'fn main() { println!("e901"); }\n' > research/e7-index-bench/src/bin/e901_stale_product.rs
printf '条目\t原文\t改成\n' > research/mutations/e902_no_product.tsv
printf 'E7RESULT name=e901 rows=1\n' > research/results/e901-stale-product-2026-09-01.out
printf '// 字段表见 first-layout.md\nfn main() { println!("e907"); }\n' > research/e7-index-bench/src/bin/e907_comment_and_code.rs
printf 'E7RESULT name=e907 rows=1\n' > research/results/e907-comment-and-code-2026-09-01.out
printf '# 欠账\n\n| C901 | 样本欠账 | 还没还 |\n\n## 历史版本\n' > .claude/kb/sample-owed.md
mkdir -p .claude/kb/experiments
printf 'E908\t样本：按要求才跑\n' > research/on-request-experiments.tsv
printf 'fn main() { println!("e908"); }\n' > research/e7-index-bench/src/bin/e908_on_request_no_note.rs
printf 'E7RESULT name=e908 rows=1\n' > research/results/e908-on-request-no-note-2026-09-01.out
printf '# E908 样本\n\n装置改过。\n\n## 历史版本\n' > .claude/kb/experiments/908-on-request-no-note.md
git add -A && git commit -qm base
printf 'fn main() { println!("e901 改过了"); }\n' > research/e7-index-bench/src/bin/e901_stale_product.rs
printf '// 字段表见 layout/01-first.md\nfn main() { println!("e907 改过了"); }\n' > research/e7-index-bench/src/bin/e907_comment_and_code.rs
printf '条目\t原文\t改成\n判定恒真\ta == b\ttrue\n' > research/mutations/e902_no_product.tsv
cat > .claude/kb/sample-owed.md <<'EOF'
# 欠账

| C901 | 样本欠账 | 2026-09-18 还清，依据：/tmp/claude-1000/sample-round/report.md |

## 历史版本
EOF
cat > research/prompts/newround-r1-opus.md <<'EOF'
# newround-r1 云端攻方

1. 背景材料 `research/prompts/_newround-r1-background.md`。
2. 实现员报告 `/tmp/claude-1000/sample-impl/report.md`（它自己列的设计判断）。
EOF
touch -d '2026-09-01T00:00:00Z' research/results/e901-stale-product-2026-09-01.out research/results/e907-comment-and-code-2026-09-01.out
printf 'fn main() { println!("e908 改过了"); }\n' > research/e7-index-bench/src/bin/e908_on_request_no_note.rs
touch -d '2026-09-01T00:00:00Z' research/results/e908-on-request-no-note-2026-09-01.out
touch -d '2026-09-05T00:00:00Z' research/e7-index-bench/src/bin/e908_on_request_no_note.rs
touch -d '2026-09-05T00:00:00Z' research/e7-index-bench/src/bin/e901_stale_product.rs research/mutations/e902_no_product.tsv research/e7-index-bench/src/bin/e907_comment_and_code.rs
