#!/usr/bin/env bash
# gate-stage: 收口表第 27 行那几笔欠账的前置有没有进来（进来了就得补会红的用例）
# gate-similar: 67-milestone-closeout-owed.sh 它读收口表本身，判开着的欠账号收全了、行号只许顺序号；这一道不读收口表，读 crates 源码，按写死的探针判第 27 行那几笔的前置动没动，全对上退 77 而不是报绿
# gate-similar: 79-tree-table-reserve.sh 它对决策正文里的认购表求和、比预留；这一道对 crates 源码做逐字探针计数，红的条件是命中次数与登记值不等
# gate-similar: 37-decision-summary-width.sh 它判决策索引结论列的字数上限，只读 decisions.md；这一道读 crates 源码里的几段代码，判那几笔欠账今天可不可达
# gate-similar: 82-clause-enum-pairs.sh 它判条文列的集合与 Rust 枚举逐个成员对上，读 crates 源码是为了取枚举变体；这一道读源码是为了数几段写死的原文命中几次，没有条文那一侧
# gate-similar: 33-mutation-tables.sh 它数变异表每条原文在源码里恰好命中一次，锚点腐化就红，服务于变异复跑；这一道数的是登记的今天命中次数（可以是 0 或 2），全对上退 77，对不上说明那笔欠账的前置进来了、要回去重核
#
# 里程碑「第二个事务」增补 2 收口表第 27 行点名了一批「今天不可达的欠账」：走不到那一格，
# 就写不出会红的用例。这道阶段不判那几笔本身，判的是它们各自的**前置**动没动：
# 每一笔登记一条逐字探针（文件、原文、今天的命中次数）。
#   全部对上 ⇒ 前置一个都没进来，今天无对象可判 ⇒ 退出码 77（gate.sh 记「本次未跑」，不记通过、不算覆盖）。
#   任一条对不上 ⇒ 那一笔的前置进来了，或者它压着的那段代码改了形态 ⇒ 红，回去重核那一笔。
# 探针盯的是「这一笔的描述今天还成不成立」，所以重构那几处也会红：那时候要做的同样是回去重核，
# 不是把探针的期望值改成新数（改成新数等于把这一笔的描述悄悄换掉）。
#
# 成功那句报出查了几条探针、覆盖几笔，以及**没做成探针的是哪几笔**（逐个列名，现算不手抄；
# rules/show-me-test.md：扫到 0 项不是通过，报了「查了多少」还要说「没查的是哪些」）。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

# 四段：笔名、文件、原文（逐字）、今天的命中次数。同一笔可以有几条探针。
PROBES="$(cat <<'TSV'
豁免集与隔离集按记录起点做 key	crates/singlefs-core/src/mount.rs	let key = (device.0, slot.0);	1
扣住位在空发布循环之后无条件放开	crates/singlefs-core/src/mount.rs	    allocator.release_reclaim_holds();	1
alloc-basis 第三轮 ①：挂载内回收	crates/singlefs-core/src/mount.rs	reclaim_released_up_to(	2
C333（删行那次发布被重放）：行回收	crates/singlefs-core/src/instance_table.rs	rows.retain(	0
C333（删行那次发布被重放）：行回收	crates/singlefs-core/src/instance_table.rs	rows.remove(	0
TSV
)"

# 做不成逐字探针的那几笔，逐个列名——它们同样开着，只是这道阶段够不着。
UNPROBED="$(cat <<'TSV'
alloc-basis 第三轮 ②：根槽写失败推进一格重发	这条路径今天不存在，没有一处逐字文本代表「它进来了」
alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像	扣住位只住内存、盘上没有落点，没有可扫的对象
TSV
)"

if ! report="$(PROBES="$PROBES" python3 - 2>&1 <<'PY'
import os

probes = []
for line in os.environ["PROBES"].split("\n"):
    if not line.strip():
        continue
    name, path, original, expected = line.split("\t")
    probes.append((name, path, original, int(expected)))

for name, path, original, expected in probes:
    try:
        source = open(path, encoding="utf-8").read()
    except FileNotFoundError:
        print("BAD", name, f"{path} 不在了（探针指着的文件没了）", sep="\t")
        continue
    hits = source.count(original)
    if hits != expected:
        print("BAD", name, f"{path} 里「{original.strip()}」命中 {hits} 次，登记的今天的值是 {expected} 次", sep="\t")
print("CHECKED", len(probes), len({name for name, _, _, _ in probes}), sep="\t")
PY
)"; then
  echo "  ✗ 探针没跑完：内嵌 python 自己出错了，一条都没核"
  printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
  echo "    → 怎么办：按上面的报错修 .claude/gate.d/89-closeout-row27-preconditions.sh 里那段 python；"
  echo "      没跑完不是「本次未跑」，是这一道自己坏了。"
  exit 1
fi

if ! grep -q '^CHECKED' <<<"$report"; then
  echo "  ✗ 探针没跑完：内嵌 python 没报出 CHECKED 那一行（多半是它自己崩了）"
  echo "    → 怎么办：单独跑一遍这道阶段看 python 的报错；没有 CHECKED 就是一条都没核，"
  echo "      而「本次未跑」那句看着与核过了一模一样。"
  exit 1
fi

mapfile -t bad < <(grep '^BAD' <<<"$report")
read -r _ probe_count item_count < <(grep '^CHECKED' <<<"$report")

if ((${#bad[@]})); then
  echo "  ✗ 收口表第 27 行那几笔欠账的前置动了（或者它压着的那段代码改了形态）："
  while IFS=$'\t' read -r _ name why; do
    printf '      %s：%s\n' "$name" "$why"   # gate-lint:detail
  done < <(printf '%s\n' "${bad[@]}")
  echo "    → 怎么办：先回 .claude/kb/milestone/02-second-txn.md 收口表第 27 行重核这一笔今天可不可达。"
  echo "      可达了就当场做成一条会红的用例（改坏一行、看着它红），把这一笔从第 27 行挪出去，探针那一行一起删；"
  echo "      只是重构、这一笔还不可达，就把探针的原文改成今天逐字存在的那一段，命中次数照实写，并在报告里写明重核过。"
  echo "      ⚠️ 别只把期望值改成新数：那等于把这一笔的描述换掉，而外面看不出换过。"
  exit 1
fi

unprobed_count="$(grep -c . <<<"$UNPROBED")"
echo "  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（${probe_count} 条逐字探针、覆盖 ${item_count} 笔，逐条对上今天的值）"
echo "    没做成探针的 ${unprobed_count} 笔："
while IFS=$'\t' read -r name why; do
  [[ -z "$name" ]] && continue
  printf '      %s：%s\n' "$name" "$why"
done <<<"$UNPROBED"
exit 77
