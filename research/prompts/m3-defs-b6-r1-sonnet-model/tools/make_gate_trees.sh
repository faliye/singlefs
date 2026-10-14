#!/usr/bin/env bash
# 造几棵小树给门禁格 abandoned-rounds 与 long-run-header-exempt 用（格要主 agent 跑，子 agent 起不了 --check）；同一批树也喂派发闸 ⑥ 与判法库，两边并排看。
# 用法：make_gate_trees.sh <树的根目录（要不存在或空）>
set -euo pipefail
root="${1:?用法：make_gate_trees.sh <树的根目录>}"
[[ ! -e "$root" || -z "$(ls -A "$root")" ]] || { echo "  ✗ $root 已存在且非空" >&2; exit 2; }
mkdir -p "$root"
quote='「先放一放这一轮吧」'
# 树 A：2026-10-11 登记、没出处——门禁格对 2026-10-12 之前的行不要求出处，⑥ 要
a="$root/legacy-sourceless"; mkdir -p "$a/records" "$a/research/prompts/lg-r1-snapshot"
printf 'lg-r1 %s\n' "$quote" > "$a/records/log.md"
printf '# body\n' > "$a/research/prompts/_lg-r1-body.md"; printf '# out\n' > "$a/research/prompts/lg-r1-opus-output.md"
printf '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  crates/foo/src/lib.rs\n' > "$a/research/prompts/lg-r1-snapshot/frozen-sha256.txt"
printf 'lg-r1\t2026-10-11\t用户叫停，没写出处\t并进别处\n' > "$a/research/prompts/abandoned-rounds.tsv"
# 树 B：2026-10-13 登记、出处那一行另有一个不以 -r<数字> 结尾、登记表里也没出现的轮名——判法库收，文字（「除这一轮的轮名外不许有别的轮名」）说不许
b="$root/other-name-unrecognised"; mkdir -p "$b/records" "$b/research/prompts/ot-r1-snapshot"
printf 'ot-r1 与 m3-close-b1 %s\n' "$quote" > "$b/records/log.md"
printf '# body\n' > "$b/research/prompts/_ot-r1-body.md"; printf '# out\n' > "$b/research/prompts/ot-r1-opus-output.md"
printf '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  crates/foo/src/lib.rs\n' > "$b/research/prompts/ot-r1-snapshot/frozen-sha256.txt"
printf 'ot-r1\t2026-10-13\t用户叫停；出处 records/log.md 第 1 行\t并进别处\n' > "$b/research/prompts/abandoned-rounds.tsv"
# 树 C：豁免表三行——指向不存在的路径、脚本头已按登记值写齐、E171 式的现算头（这一行该放）
c="$root/exempt-table"; mkdir -p "$c/.claude" "$c/scripts"
printf '#!/usr/bin/env bash\n# long-run-shards: 按用例切\n# long-run-resume: 同一条命令重起只补没做完的\n# long-run-hosts: 台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量\n' > "$c/scripts/registered.sh"
printf '#!/usr/bin/env bash\n# long-run-shards: 按用例切\n# long-run-resume: 同一条命令重起只补没做完的\n# long-run-hosts: 台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台额度从 host-resources.py --json 现算\n' > "$c/scripts/live.sh"
printf 'scripts/missing.sh  # 路径不存在\nscripts/registered.sh  # 头已按登记值写齐\nscripts/live.sh  # 头按现算写、不能改\n' > "$c/.claude/long-run-live-hosts-exempt"
# 树 D：出处是符号链接（records/link.md 指到 records 外）、出处在 records 子目录（records/sub/deep.md）
d="$root/symlink-source"; mkdir -p "$d/records/sub" "$d/outside" "$d/research/prompts"
printf 'sy-r1 %s\n' "$quote" > "$d/outside/log.md"; ln -s ../outside/log.md "$d/records/link.md"
printf 'sub-r1 %s\n' "$quote" > "$d/records/sub/deep.md"
printf 'sy-r1\t2026-10-13\t出处 records/link.md 第 1 行\t并进别处\nsub-r1\t2026-10-13\t出处 records/sub/deep.md 第 1 行\t并进别处\n' > "$d/research/prompts/abandoned-rounds.tsv"
echo "  ✓ 造了 4 棵树：$a $b $c $d"
