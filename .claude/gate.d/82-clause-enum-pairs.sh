#!/usr/bin/env bash
# gate-stage: 条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上
# gate-similar: 67-milestone-closeout-owed.sh 它判里程碑收口表收全开着的欠账号、行号只许顺序号，只读 kb；这一道拿登记表把决策分项正文里的名字与 crates 源码里 Rust 枚举的变体逐个配对，任一边多一个或少一个都红
# gate-similar: 81-audit-contradictions.sh 它判总审核记录第五节每行有去向，只读 records/ 与 kb；这一道要读 crates 源码里的枚举，判条文与代码的集合对不对得上
# gate-similar: 79-tree-table-reserve.sh 它对决策正文里的认购表求和、比预留；这一道不做算术，判登记表里每一对（分项里的名字、枚举变体）两头都在，枚举里没登记的变体也红
# gate-similar: 53-format-const-placeholders.sh 它判格式常量文件里的 placeholder 注释指得到分项或欠账，方向是代码到 kb 的单向；这一道对封闭集合双向逐个成员比，枚举多一个成员与表多一行都红
# gate-similar: 93-feature-bits.sh 它判 feature bit 位号在记账表与 D15 登记表之间逐行一致、代码常量的位在记账表里，对象是位号；这一道对象是条文列举的封闭集合与 Rust 枚举变体，按 82-clause-enum-pairs.tsv 配对
# gate-similar: 27-format-constants.sh 它判 kb 里 format-const 标记的数值与源码常量相等、旧值不再出现，对象是数值；这一道对象是枚举变体的名字集合，不看数值
#
# 2026-09-23 一天里查出四例「条文与实现说反话」，其中三例形状相同：条文逐个列了一个集合，
# 而代码里那个枚举比它多一个或少一个成员，没有任何东西会红。门禁 27 号只管登记过的格式常量、
# 92 号只判「格式变了 checker 没跟」，两道都够不着这一类。
#
# 这道阶段拿 .claude/gate.d/82-clause-enum-pairs.tsv 逐对判三样：
#   ① 枚举里每个变体都在表里有一行（代码加了成员而没登记 ⇒ 红）；
#   ② 表里每一行的变体在枚举里真的存在（代码删了成员而表没跟 ⇒ 红）；
#   ③ 表里每一行的「条文里对应的名字」在那条分项的正文里找得到（条文没跟上 ⇒ 红）。
#
# ⚠️ 它判的是**名字对得上**，不判「这个成员的语义是不是条文说的那个」。后者是语义判断，靠 review。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

TABLE=".claude/gate.d/82-clause-enum-pairs.tsv"
if [[ ! -f "$TABLE" ]]; then
  echo "  ⊘ 本次未跑：$TABLE 不在，今天无对象可判"
  exit 77
fi

if ! report="$(TABLE="$TABLE" python3 - 2>&1 <<'PY'
import os, re, glob

rows = []
for number, line in enumerate(open(os.environ["TABLE"], encoding="utf-8"), 1):
    stripped = line.rstrip("\n")
    if not stripped.strip() or stripped.lstrip().startswith("#"):
        continue
    parts = stripped.split("\t")
    if len(parts) != 5 or not all(part.strip() for part in parts):
        print("BAD", f"{os.environ['TABLE']}:{number}", "这一行不是五段非空（决策分项、枚举落点、变体名、条文里的名字、为什么）", sep="\t")
        continue
    rows.append((number, *[part.strip() for part in parts]))

def clause_text(item):
    match = re.match(r"^(D\d+)\s+已定项\s+(\d+)$", item)
    if not match:
        return None, f"决策分项写成「{item}」，认不出「D<n> 已定项 <k>」这个形态"
    decision, number = match.group(1), match.group(2)
    files = glob.glob(f".claude/kb/decisions/{int(decision[1:]):02d}-*.md")
    if not files:
        return None, f"找不到 {decision} 的正文文件"
    text = open(files[0], encoding="utf-8").read()
    section = re.search(rf"^#### 已定项 {number}[:：].*?(?=^#### |\Z)", text, re.S | re.M)
    if not section:
        return None, f"{files[0]} 里找不到「#### 已定项 {number}」这一节"
    return section.group(0), None

def enum_variants(where):
    path, _, name = where.rpartition(":")
    if not path or not name:
        return None, f"枚举落点写成「{where}」，认不出「<文件>:<枚举名>」这个形态"
    if not os.path.exists(path):
        return None, f"{path} 不在了"
    text = open(path, encoding="utf-8").read()
    block = re.search(rf"enum\s+{re.escape(name)}\s*\{{(.*?)^\}}", text, re.S | re.M)
    if not block:
        return None, f"{path} 里找不到 `enum {name}`"
    body = re.sub(r"//.*", "", block.group(1))
    body = re.sub(r"#\[[^\]]*\]", "", body)
    return re.findall(r"^\s*([A-Z][A-Za-z0-9]*)\s*(?:\{|\(|,|$)", body, re.M), None

clauses, enums = {}, {}
checked = 0
for number, item, where, variant, chinese, _why in rows:
    if item not in clauses:
        clauses[item] = clause_text(item)
    section, why = clauses[item]
    if why:
        print("BAD", f"{os.environ['TABLE']}:{number}", why, sep="\t")
        continue
    if where not in enums:
        enums[where] = enum_variants(where)
    variants, why = enums[where]
    if why:
        print("BAD", f"{os.environ['TABLE']}:{number}", why, sep="\t")
        continue
    checked += 1
    if variant not in variants:
        print("BAD", f"{os.environ['TABLE']}:{number}",
              f"表里写着变体 `{variant}`，而 {where} 的枚举里没有它（代码删了成员而表没跟）", sep="\t")
    if chinese not in section:
        print("BAD", f"{os.environ['TABLE']}:{number}",
              f"{item} 的正文里找不到「{chinese}」（条文没跟上代码，或者名字改了而表没跟）", sep="\t")

# 反过来：枚举里有、表里没有的变体。
for where, (variants, why) in enums.items():
    if why:
        continue
    registered = {variant for _n, _i, w, variant, _c, _y in rows if w == where}
    for variant in variants:
        if variant not in registered:
            print("BAD", where,
                  f"枚举里有变体 `{variant}`，而 {os.environ['TABLE']} 里一行都没有（代码加了成员而没登记）", sep="\t")

print("COUNT", checked, len(enums), len(clauses), sep="\t")
PY
)"; then
  echo "  ✗ 扫描没跑完：内嵌 python 自己出错了，一对都没比"
  printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
  echo "    → 怎么办：按上面的报错修 .claude/gate.d/82-clause-enum-pairs.sh 里那段 python；"
  echo "      这一步没跑完就是什么都没查，不是通过。"
  exit 1
fi

if ! grep -q '^COUNT' <<<"$report"; then
  echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（多半是它自己崩了）"
  echo "    → 怎么办：单独跑一遍这道阶段看 python 的报错；没有 COUNT 就是一对都没比，"
  echo "      而汇总行看着与比过了一模一样。"
  exit 1
fi

mapfile -t bad < <(grep '^BAD' <<<"$report")

if ((${#bad[@]})); then
  echo "  ✗ 条文列的集合与代码里的枚举对不上："
  while IFS=$'\t' read -r _ where why; do
    printf '      %s：%s\n' "$where" "$why"   # gate-lint:detail
  done < <(printf '%s\n' "${bad[@]}")
  echo "    → 怎么办：先判是哪一边错了——代码加了成员就把它写回那条分项的正文、再往 $TABLE 补一行；"
  echo "      条文才是对的就改代码。⚠️ 别只改表让两边看着一致：表是登记位，不是第三种说法。"
  echo "      改分项正文是一次决策变更，照 .claude/rules/format-evolution.md 记进当月变更史。"
  exit 1
fi

read -r _ checked enum_count clause_count < <(grep '^COUNT' <<<"$report")
echo "  ✓ 条文列的集合与代码里的枚举逐个对得上（${checked} 对成员、${enum_count} 个枚举、${clause_count} 条分项）"
exit 0
