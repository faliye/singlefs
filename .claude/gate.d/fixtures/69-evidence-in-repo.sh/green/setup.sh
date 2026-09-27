#!/usr/bin/env bash
# 本该判绿：装置改了而产物比它新（E903）；另一个装置这一轮没碰，产物再旧也不判（E904）；
# kb 里三种说做法的 /tmp 写法（命令、镜像路径、整行抄的产物行）不算引依据；
# 这一轮新写的提示里「草稿放 /tmp/…」也不算；一份改过但不是新写的提示里写着「依据：/tmp/…」，
# 它登记在原样保存的证据里，本阶段不判——扫到的 /tmp 处数与没判的份数把这一条钉住；
# E905 只改了 // 注释里的路径（path-moves.md 要求的改写），产物再旧也不判，成功行列出它。
# E909 登记了准入，产物比源码旧而产物头的输入指纹与今天的输入相同，按指纹判绿，成功行列出它。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p research/e7-index-bench/src/bin research/mutations research/results .claude/kb research/prompts
printf 'fn main() { println!("e903"); }\n' > research/e7-index-bench/src/bin/e903_fresh_product.rs
printf 'fn main() { println!("e904"); }\n' > research/e7-index-bench/src/bin/e904_untouched.rs
printf 'E7RESULT name=e903 rows=1\n' > research/results/e903-fresh-product-2026-09-06.out
printf '// 字段表见 first-layout.md\nfn main() { println!("e905"); }\n' > research/e7-index-bench/src/bin/e905_comment_only.rs
printf 'E7RESULT name=e905 rows=1\n' > research/results/e905-comment-only-2026-09-01.out
printf 'E7RESULT name=e904 rows=1\n' > research/results/e904-untouched-2026-09-01.out
cat > .claude/kb/sample-harness.md <<'EOF'
# 样本装置

ls -d /tmp/singlefs-vmbench.* 2>/dev/null

- 装置：本机 ext4 上 8 GiB O_DIRECT 文件（`/tmp/e900.img`），单元 32768、页 4096。

E7RESULT name=config dev=/tmp/e900.img size=8589934592 unit=32768

## 历史版本
EOF
cat > research/prompts/oldround-r1-opus.md <<'EOF'
# oldround-r1 云端攻方

报告原件的依据：/tmp/claude-1000/oldround-r1-opus/report.md
EOF
mkdir -p .claude/kb/experiments
printf 'E906\t样本：按要求才跑\n' > research/on-request-experiments.tsv
printf 'fn main() { println!("e906"); }\n' > research/e7-index-bench/src/bin/e906_on_request.rs
printf 'E7RESULT name=e906 rows=1\n' > research/results/e906-on-request-2026-09-01.out
printf '# E906 样本\n\n按要求才跑。\n\n## 历史版本\n' > .claude/kb/experiments/906-on-request.md
# E909：登记了准入，产物比源码旧、产物头的输入指纹与今天的输入相同 ⇒ 按指纹判绿（按修改时刻会判红）
mkdir -p .claude/gate.d
e909_source=research/e7-index-bench/src/bin/e909_fingerprint_matches.rs
printf 'E909\t%s\t# 样本：登记了准入\n' "$e909_source" > .claude/gate.d/stage-inputs.tsv
printf 'fn main() { println!("e909"); }\n' > "$e909_source"
git add -A && git commit -qm base
printf 'fn main() { println!("e903 改过了"); }\n' > research/e7-index-bench/src/bin/e903_fresh_product.rs
printf '// 字段表见 layout/01-first.md\nfn main() { println!("e905"); }\n' > research/e7-index-bench/src/bin/e905_comment_only.rs
printf '# oldround-r1 云端攻方\n\n报告原件的依据：/tmp/claude-1000/oldround-r1-opus/report.md\n\n（这一行是这一轮加的注，冻结材料的路径不改）\n' > research/prompts/oldround-r1-opus.md
cat > research/prompts/newround-r2-sonnet.md <<'EOF'
# newround-r2 云端辩方

4. 草稿放你自己的目录 /tmp/claude-1000/newround-r2-sonnet/；报告写进 research/prompts/newround-r2-sonnet-output.md。
EOF
touch -d '2026-09-01T00:00:00Z' research/results/e904-untouched-2026-09-01.out research/e7-index-bench/src/bin/e904_untouched.rs research/results/e905-comment-only-2026-09-01.out
touch -d '2026-09-05T00:00:00Z' research/e7-index-bench/src/bin/e903_fresh_product.rs
printf 'fn main() { println!("e906 改过了"); }\n' > research/e7-index-bench/src/bin/e906_on_request.rs
# 「没有正式跑」那句是这一轮改装置时新写进实验页的（69 号只认这一轮新加的行）
printf '# E906 样本\n\n按要求才跑。\n\n装置 2026-09-05 改过，之后没有正式跑。\n\n## 历史版本\n' > .claude/kb/experiments/906-on-request.md
touch -d '2026-09-01T00:00:00Z' research/results/e906-on-request-2026-09-01.out
touch -d '2026-09-05T00:00:00Z' research/e7-index-bench/src/bin/e906_on_request.rs
touch -d '2026-09-06T00:00:00Z' research/results/e903-fresh-product-2026-09-06.out
# E909 的指纹不经准入模块，照它文件头写的算法用 sha256sum 另算一遍：逐文件「sha256  路径」、登记行、工具链，整张再算一次
printf 'fn main() { println!("e909 改过了"); }\n' > "$e909_source"
registration_hash="$(printf 'E909\t%s\n' "$e909_source" | sha256sum | cut -d' ' -f1)"
toolchain_versions="$(cargo -V && rustc -V)"
toolchain_hash="$(printf '%s\n' "$toolchain_versions" | sha256sum | cut -d' ' -f1)"
e909_fingerprint="$( { sha256sum -- "$e909_source"; printf '%s  %s\n' "$registration_hash" "<登记行：E909>" "$toolchain_hash" "<工具链：${toolchain_versions//$'\n'/；}>"; } | sha256sum | cut -d' ' -f1)"
printf 'E7INPUT name=input_fingerprint key=E909 sha256=%s files=1\nE7RESULT name=e909 rows=1\n' "$e909_fingerprint" > research/results/e909-fingerprint-matches-2026-09-01.out
touch -d '2026-09-01T00:00:00Z' research/results/e909-fingerprint-matches-2026-09-01.out
touch -d '2026-09-05T00:00:00Z' "$e909_source"
