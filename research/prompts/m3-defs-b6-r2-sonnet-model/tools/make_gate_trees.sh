#!/usr/bin/env bash
# 造几棵小树给门禁格 abandoned-rounds、long-run-header-exempt、heavy-thread-defaults 用（格要主 agent 跑，子 agent 起不了 --check）；
# 同一批树也喂派发闸 ⑥ 与判法库，两边并排看。第二轮在第一轮的四棵之上改了豁免表的写法（带 sha256）并加了几棵。
# 用法：make_gate_trees.sh <树的根目录（要不存在或空）>
set -euo pipefail
root="${1:?用法：make_gate_trees.sh <树的根目录>}"
[[ ! -e "$root" || -z "$(ls -A "$root")" ]] || { echo "  ✗ $root 已存在且非空" >&2; exit 2; }
mkdir -p "$root"
quote='「先放一放这一轮吧」'
sha() { sha256sum "$1" | awk '{print $1}'; }
round_tree() {   # round_tree <目录> <轮名> <日期> <理由> <records 第 1 行内容> [附加材料文件…]
  local dir=$1 name=$2 date=$3 reason=$4 record=$5; shift 5
  mkdir -p "$dir/records" "$dir/research/prompts/$name-snapshot"
  printf '%s\n' "$record" > "$dir/records/log.md"
  printf '# body\n' > "$dir/research/prompts/_$name-body.md"; printf '# out\n' > "$dir/research/prompts/$name-opus-output.md"
  printf '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  crates/foo/src/lib.rs\n' > "$dir/research/prompts/$name-snapshot/frozen-sha256.txt"
  printf '%s\t%s\t%s\t并进别处\n' "$name" "$date" "$reason" > "$dir/research/prompts/abandoned-rounds.tsv"
  for extra in "$@"; do printf '# x\n' > "$dir/research/prompts/$extra"; done
}
# 树 A：2026-10-11 登记、没出处。第一轮：门禁格对 2026-10-12 之前的行免出处（绿）而 ⑥ 要（不收）；第二轮：两边同一张名单，名单外的行都要出处（门禁格应红，⑥ 不收）
round_tree "$root/legacy-sourceless" lg-r1 2026-10-11 "用户叫停，没写出处" "lg-r1 $quote"
# 树 A2：名单内的旧行 c196-r1（日期早、没出处）：第二轮两边都免
round_tree "$root/legacy-listed" c196-r1 2026-09-12 "旧行，没出处可补" "c196-r1 $quote"
# 树 B：出处那一行另有 m3-close-b1，仓里没有它的任何材料——第一轮、第二轮的库都收
round_tree "$root/other-name-unrecognised" ot-r1 2026-10-13 "用户叫停；出处 records/log.md 第 1 行" "ot-r1 与 m3-close-b1 $quote"
# 树 B2：同上，但仓里有 m3-close-b1 的开工材料——第二轮的库不收（红），第一轮收
round_tree "$root/other-name-recognised" ot-r1 2026-10-13 "用户叫停；出处 records/log.md 第 1 行" "ot-r1 与 m3-close-b1 $quote" _m3-close-b1-body.md
# 树 B3：出处那一行另有 other-r2（-r<数字> 形），仓里没有它的材料——第一轮不收，第二轮收（改过头候选）
round_tree "$root/other-form-nomaterial" ot-r1 2026-10-13 "用户叫停；出处 records/log.md 第 1 行" "ot-r1 与 other-r2 $quote"
# 树 C：豁免表（第二轮写法：路径  sha256  # 理由）五行——缺路径、头已按登记值写齐、E171 式现算头（sha256 对得上，该放）、现算头但 sha256 登记错了、写法不对（没有 sha256）
c="$root/exempt-table"; mkdir -p "$c/.claude" "$c/scripts"
canon="台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量"
printf '#!/usr/bin/env bash\n# long-run-shards: 按用例切\n# long-run-resume: 同一条命令重起只补没做完的\n# long-run-hosts: %s\n' "$canon" > "$c/scripts/registered.sh"
printf '#!/usr/bin/env bash\n# long-run-shards: 按用例切\n# long-run-resume: 同一条命令重起只补没做完的\n# long-run-hosts: 台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台额度从 host-resources.py --json 现算\n' > "$c/scripts/live.sh"
cp "$c/scripts/live.sh" "$c/scripts/stale.sh"
{ printf 'scripts/missing.sh  %s  # 路径不存在\n' "$(printf '0%.0s' $(seq 64))"
  printf 'scripts/registered.sh  %s  # 头已按登记值写齐\n' "$(sha "$c/scripts/registered.sh")"
  printf 'scripts/live.sh  %s  # 头按现算写、不能改\n' "$(sha "$c/scripts/live.sh")"
  printf 'scripts/stale.sh  %s  # 登记之后改过一行\n' "$(sha "$c/scripts/live.sh")"
  printf 'scripts/old-format.sh  # 没有 sha256 的第一轮写法\n'; } > "$c/.claude/long-run-live-hosts-exempt"
printf '# 登记之后改过一行\n' >> "$c/scripts/stale.sh"
printf '#!/usr/bin/env bash\n# long-run-shards: x\n' > "$c/scripts/old-format.sh"
# 树 D：出处是符号链接（records/link.md 指到 records 外）、出处在 records 子目录（records/sub/deep.md）
d="$root/symlink-source"; mkdir -p "$d/records/sub" "$d/outside" "$d/research/prompts"
printf 'sy-r1 %s\n' "$quote" > "$d/outside/log.md"; ln -s ../outside/log.md "$d/records/link.md"
printf 'sub-r1 %s\n' "$quote" > "$d/records/sub/deep.md"
printf 'sy-r1\t2026-10-13\t出处 records/link.md 第 1 行\t并进别处\nsub-r1\t2026-10-13\t出处 records/sub/deep.md 第 1 行\t并进别处\n' > "$d/research/prompts/abandoned-rounds.tsv"
echo "  ✓ 造了 7 棵树：legacy-sourceless legacy-listed other-name-unrecognised other-name-recognised other-form-nomaterial exempt-table symlink-source"
