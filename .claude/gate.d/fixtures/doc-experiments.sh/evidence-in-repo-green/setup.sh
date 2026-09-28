#!/usr/bin/env bash
# 本该判绿：装置改了而产物比它新（E903）；另一个装置这一轮没碰，产物再旧也不判（E904）；
# kb 里三种说做法的 /tmp 写法（命令、镜像路径、整行抄的产物行）不算引依据；
# 这一轮新写的提示里「草稿放 /tmp/…」也不算；一份改过但不是新写的提示里写着「依据：/tmp/…」，
# 它登记在原样保存的证据里，本阶段不判——扫到的 /tmp 处数与没判的份数把这一条钉住；
# E905 只改了 // 注释里的路径（path-moves.md 要求的改写），产物再旧也不判，成功行列出它。
# E909 登记了准入，产物比源码旧而产物头的输入指纹与今天的输入相同，按指纹判绿，成功行列出它。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo；原先同一道里别的九格也跑这份样本，为它们补的东西（留着不碍事）在下面各处的注释里写明（一条样本决策、每个实验号一页正文、产物不进版本库）；
# 补的实验页与决策是 .md，这一格扫的 .md 因此从 3 份变 9 份。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p research/e7-index-bench/src/bin research/mutations research/results .claude/kb research/prompts
printf 'fn main() { println!("e903"); }\n' > research/e7-index-bench/src/bin/e903_fresh_product.rs
printf 'fn main() { println!("e904"); }\n' > research/e7-index-bench/src/bin/e904_untouched.rs
printf '// 字段表见 first-layout.md\nfn main() { println!("e905"); }\n' > research/e7-index-bench/src/bin/e905_comment_only.rs
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
mkdir -p .claude/kb/experiments .claude/kb/decisions
printf 'E906\t样本：按要求才跑\n' > research/on-request-experiments.tsv
printf 'fn main() { println!("e906"); }\n' > research/e7-index-bench/src/bin/e906_on_request.rs
# 给 decision-links 格：一条没有分项的样本决策，各实验页影响的决策表里那一行「备料」指它
printf '## D1 样本决策 —— 已定\n\n样本决策，只给实验页的影响的决策表指。\n\n## 历史版本\n' > .claude/kb/decisions/01-样本决策.md
# E906 的实验页：标题写成「## E906 」、点名产物与复跑入口、带一张影响的决策表，是给 decision-links、results-cited、repro-command 三格的；
# 第一个参数是表里那一行的回看格，第二个参数是这一轮新写进页里的那一句（基准那一版没有）
experiment_page_906() {
  printf '## E906 按要求才跑的样本 —— 样本\n\n按要求才跑。\n\n产物 `research/results/e906-on-request-2026-09-01.out`，复跑经 `research/scripts/replay.sh`。\n\n%s### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n| D1（样本决策） | 备料 | %s |\n\n## 历史版本\n' \
    "$2" "$1" > .claude/kb/experiments/906-on-request.md
}
experiment_page_906 '2026-09-01 不受影响：样本页只作对照' ''
# E909：登记了准入，产物比源码旧、产物头的输入指纹与今天的输入相同 ⇒ 按指纹判绿（按修改时刻会判红）
mkdir -p .claude/gate.d
e909_source=research/e7-index-bench/src/bin/e909_fingerprint_matches.rs
printf 'E909\t%s\t# 样本：登记了准入\n' "$e909_source" > .claude/gate.d/stage-inputs.tsv
printf 'fn main() { println!("e909"); }\n' > "$e909_source"
git add -A && git commit -qm base
# 产物在基准提交之后才落盘、不进版本库（这一格只看产物的修改日子与指纹，不看跟没跟踪）：
# 同一道里的 archive-past-rounds 格把跟踪着又没改过的产物当上一轮的，这样它在这份样本里判绿
printf 'E7RESULT name=e903 rows=1\n' > research/results/e903-fresh-product-2026-09-06.out
printf 'E7RESULT name=e905 rows=1\n' > research/results/e905-comment-only-2026-09-01.out
printf 'E7RESULT name=e904 rows=1\n' > research/results/e904-untouched-2026-09-01.out
printf 'E7RESULT name=e906 rows=1\n' > research/results/e906-on-request-2026-09-01.out
# research 下每个实验号一页正文（未跟踪，整页算这一轮新写），点名它的产物与复跑入口、带一张影响的决策表：
# 给 experiment-orphans、results-cited、repro-command、decision-links 四格（E7 是装置目录 research/e7-index-bench 的号）
experiment_page() {   # 号、简称、点名产物的那一句
  printf '## E%s %s —— 样本\n\n%s\n\n### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n| D1（样本决策） | 备料 | 2026-09-06 不受影响：样本页只作对照 |\n\n## 历史版本\n' \
    "$1" "$2" "$3" > ".claude/kb/experiments/$1-样本.md"
}
experiment_page 7 装置目录的号 '装置目录 research/e7-index-bench 用的是这个号。'
experiment_page 903 产物比装置新 '产物 `research/results/e903-fresh-product-2026-09-06.out`，复跑经 `research/scripts/replay.sh`。'
experiment_page 904 没碰的旧装置 '产物 `research/results/e904-untouched-2026-09-01.out`，复跑经 `research/scripts/replay.sh`。'
experiment_page 905 只改了注释的装置 '产物 `research/results/e905-comment-only-2026-09-01.out`，复跑经 `research/scripts/replay.sh`。'
experiment_page 909 按指纹判的装置 '产物 `research/results/e909-fingerprint-matches-2026-09-01.out`，复跑经 `research/scripts/replay.sh`。'
printf 'fn main() { println!("e903 改过了"); }\n' > research/e7-index-bench/src/bin/e903_fresh_product.rs
printf '// 字段表见 layout/01-first.md\nfn main() { println!("e905"); }\n' > research/e7-index-bench/src/bin/e905_comment_only.rs
printf '# oldround-r1 云端攻方\n\n报告原件的依据：/tmp/claude-1000/oldround-r1-opus/report.md\n\n（这一行是这一轮加的注，冻结材料的路径不改）\n' > research/prompts/oldround-r1-opus.md
cat > research/prompts/newround-r2-sonnet.md <<'EOF'
# newround-r2 云端辩方

4. 草稿放你自己的目录 /tmp/claude-1000/newround-r2-sonnet/；报告写进 research/prompts/newround-r2-sonnet-output.md。
EOF
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e904-untouched-2026-09-01.out research/e7-index-bench/src/bin/e904_untouched.rs research/results/e905-comment-only-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 research/e7-index-bench/src/bin/e903_fresh_product.rs
printf 'fn main() { println!("e906 改过了"); }\n' > research/e7-index-bench/src/bin/e906_on_request.rs
# 「没有正式跑」那句是这一轮改装置时新写进实验页的（evidence-in-repo 格只认这一轮新加的行）；影响的决策表那一行同时回看（decision-links 格要）
experiment_page_906 '2026-09-05 不受影响：装置改过、结论不变' $'装置 2026-09-05 改过，之后没有正式跑。\n\n'
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e906-on-request-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 research/e7-index-bench/src/bin/e906_on_request.rs
TZ=Asia/Tokyo touch -d 2026-09-06 research/results/e903-fresh-product-2026-09-06.out
# E909 的指纹不经准入模块，照它文件头写的算法用 sha256sum 另算一遍：逐文件「sha256  路径」、登记行、工具链，整张再算一次
printf 'fn main() { println!("e909 改过了"); }\n' > "$e909_source"
registration_hash="$(printf 'E909\t%s\n' "$e909_source" | sha256sum | cut -d' ' -f1)"
toolchain_versions="$(cargo -V && rustc -V)"
toolchain_hash="$(printf '%s\n' "$toolchain_versions" | sha256sum | cut -d' ' -f1)"
e909_fingerprint="$( { sha256sum -- "$e909_source"; printf '%s  %s\n' "$registration_hash" "<登记行：E909>" "$toolchain_hash" "<工具链：${toolchain_versions//$'\n'/；}>"; } | sha256sum | cut -d' ' -f1)"
printf 'E7INPUT name=input_fingerprint key=E909 sha256=%s files=1\nE7RESULT name=e909 rows=1\n' "$e909_fingerprint" > research/results/e909-fingerprint-matches-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e909-fingerprint-matches-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 "$e909_source"
