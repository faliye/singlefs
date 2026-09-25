#!/usr/bin/env bash
# gate-stage: 最近一次总审核登记的文档级矛盾，每一行都要有去向
# gate-similar: 79-tree-table-reserve.sh 它对决策正文里的认购表求和、比预留，读 decisions/；这一道读 records/ 最近一次总审核第五节，判每行处置列有去向，没有算术
# gate-similar: 67-milestone-closeout-owed.sh 它判里程碑文件点名的开着欠账号都进了收口表或豁免，对象是里程碑文件；这一道的对象是总审核记录第五节的矛盾行，要求处置列写「已改」并带能在点名的 kb 文件里找到的引文，或点名一个开着的欠账号
# gate-similar: 82-clause-enum-pairs.sh 它判条文列的集合与 Rust 枚举逐个成员对上，输入是登记表与 crates 源码；这一道输入是总审核记录与 kb 正文，判的是矛盾行有没有去向
# gate-similar: 99-multipath-registry.sh 它判实验页「路径与结论登记」表的形状、源码落点与共用项，对象是实验页；这一道对象是总审核第五节，红在处置列既不是核得动的「已改」、也不指开着的欠账
# gate-similar: 88-quoted-result-lines.sh 它判 kb 正文里整行抄的 E7RESULT 产物行在 research/results/ 里逐字找得到，方向是 kb 到产物；这一道判总审核处置列里「」引的原文在点名的 kb 文件今天的正文里找得到，方向是记录到 kb，另外要判欠账号还开着
#
# `records/` 里的总审核记录第五节登记文档级矛盾。这些矛盾**在 kb 里一个落点都没有**，
# 只活在那份记录里：没有任何检查会在它们腐化时变红，而其中几条按记录自陈会误导实现者
# （C358（总审核第五节的文档级矛盾没有回扫闸））。
#
# 这道阶段判：第五节表里每一行的处置列，要么以「已改」开头，要么点名一个 checks-owed.md
# **开着那张表**里的欠账号。只点名已还清的号不算去向——那笔账还了，这一行的矛盾却还在。
#
# 「已改」还要指得到今天的正文（C358（总审核第五节的文档级矛盾没有回扫闸） 的原话）：处置列要用「」引至少一句原文，每一句归一之后
# （去掉 ** 与反引号、空白压成一个）都要在这一行点名的 kb 文件的今天正文里找得到——点名指这一行四列里
# 出现的 D / E 编号（→ decisions/、experiments/ 下同号的那份）、C 号（→ checks-owed.md）、I- 号
# （→ invariants.md）与直接写出的 .md 文件名；四列里一份 kb 文件都没点名的，才在 kb 全部正文里找。
# 正文 = 「## 历史版本」之前，不含 decisions-history/ 与 *-history.md。
# 引号后面紧跟「0 命中」「不在」「没有了」的是在说那句话已经删掉，不算指向正文的引文；
# 这类「已经不在」的说法本阶段**不核**（「0 命中」在哪个范围里 0 命中，处置列写不清，按整份文件判会误红）。
# 只判引文找不找得到，不判引文说的是不是那一行的矛盾——后者要人看。
#
# 挂账表 .claude/audit-rows-pending 2026-09-23 已清空并关闭：立闸那天挂着的 26 行逐行现查过，
# 全部给了去向。表里再出现任何一行非注释内容就判红——新的矛盾行一律当场给去向，不许挂账。
#
# 判别力：fixtures/81-audit-contradictions.sh/red 放一行「未改」且不给欠账号、一行只点名已还清的号、
# 一行挂进已关闭的挂账表、一行「已改」却不引原文、两行「已改」引的原文在点名的文件里找不到而只在别的文件里有
# （一行按 D 编号点名、一行按文件名点名）、一行引的原文哪里都没有，必须判红；green 放按 D 编号与按文件名点名、
# 原文都在点名的文件里各一行，一行没点名文件而原文在 kb 别处，一行带「「旧说法」0 命中」而 kb 里没有「旧说法」，必须判绿。
#
# 内嵌 python 崩了不许走绿：它的退出码要取，而且必须报出 COUNT 那一行，缺一样判红
# （2026-09-23 门禁审计那一轮查出：不取退出码时，python 崩了成功句照印、只是数变空白）。
#
#   bash .claude/gate.d/81-audit-contradictions.sh [仓根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

PENDING=".claude/audit-rows-pending"

latest="$(ls -1 records/*-总审核.md 2>/dev/null | sort | tail -1)"
if [[ -z "$latest" ]]; then
  echo "  ⊘ 本次未跑：records/ 下没有总审核记录，今天无对象可判"
  exit 77
fi

if ! report="$(LATEST="$latest" PENDING="$PENDING" python3 - 2>&1 <<'PY'
import glob, os, re

latest = os.environ["LATEST"]
pending_path = os.environ["PENDING"]

# 挂账表已关闭：任何一行非注释内容都是新挂的账。
if os.path.exists(pending_path):
    for number, line in enumerate(open(pending_path, encoding="utf-8"), 1):
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            print("BAD", f"{pending_path}:{number}",
                  "挂账表 2026-09-23 已清空并关闭，不许再加行；这一行的矛盾要当场给去向", sep="\t")

# 只认开着那张表：「### 已还清」之后的号是还清了的，点名它们不算去向。
owed_text = open(".claude/kb/checks-owed.md", encoding="utf-8").read()
open_owed = set(re.findall(r"^\|\s*(C\d+)\s*\|", owed_text.split("### 已还清")[0], re.M))
repaid_owed = set(re.findall(r"^\|\s*(C\d+)\s*\|", owed_text.split("### 已还清", 1)[1].split("## 历史版本", 1)[0], re.M)) \
    if "### 已还清" in owed_text else set()   # 只到「## 历史版本」为止：历史节里的表行不是已还清

lines = open(latest, encoding="utf-8").read().split("\n")
in_section = False
rows = []
for number, line in enumerate(lines, 1):
    if re.match(r"^##\s*五[、.]", line):
        in_section = True
        continue
    if in_section and re.match(r"^##\s", line):
        break
    if in_section and line.startswith("| "):
        rows.append((number, line))

# 表头与分隔行不是矛盾行。
rows = [(number, line) for number, line in rows
        if not line.startswith("|---") and not re.match(r"^\|\s*#\s*\|", line)]

# 「已改」要指得到今天的正文：处置列用「」引的原文，按空白与 ** ` 归一之后，要在今天的 kb 正文
# （「## 历史版本」之前，不含变更史）里找得到，而且要在这一行点名的文件里找得到（D / E 编号、C 号 → checks-owed.md、
# I- 号 → invariants.md、直接写出的 .md 文件名）；一个 kb 文件都没点名的行才在 kb 全部正文里找。
# 不退到全部正文里找：旧条款的原句常被实验页原样引着，改掉之后照样搜得到，会把已经不在的那句当成还在。
# 引号后面紧跟「0 命中」「不在」「没有了」的，说的是那句话已经删掉，不当成指向正文的引文。
QUOTE = re.compile(r"「([^」]+)」")
NEGATED_AFTER = re.compile(r"^\s*(?:字样|都|也|已经)?\s*(?:0\s*命中|不在|没有了)")

def normalized(text):
    return re.sub(r"\s+", " ", re.sub(r"[*`]", "", text)).strip()

kb_body_by_path = {}
for path in sorted(glob.glob(".claude/kb/**/*.md", recursive=True)):
    if path.endswith("-history.md") or "/decisions-history/" in path:
        continue
    kb_body_by_path[path] = normalized(open(path, encoding="utf-8").read().split("\n## 历史版本")[0])

def kb_paths_named_by(text):
    named = set()
    for number in re.findall(r"(?<![A-Za-z0-9_])D(\d+)(?!\d)", text):
        named |= set(glob.glob(f".claude/kb/decisions/{int(number):02d}-*.md"))
    for number in re.findall(r"(?<![A-Za-z0-9_])E(\d+)(?!\d)", text):
        named |= set(glob.glob(f".claude/kb/experiments/{int(number):02d}-*.md"))
    if re.search(r"(?<![A-Za-z0-9_])C\d+(?!\d)", text):
        named.add(".claude/kb/checks-owed.md")
    if re.search(r"(?<![A-Za-z0-9_])I-\d", text):
        named.add(".claude/kb/invariants.md")
    for file_name in re.findall(r"([\w\-]+\.md)\b", text):
        named |= {path for path in kb_body_by_path if os.path.basename(path) == file_name}
    return {path for path in named if path in kb_body_by_path}

checked = 0
fixed_rows, quote_count, unnamed_rows = 0, 0, 0
for number, line in rows:
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    if len(cells) < 4:
        print("BAD", f"{latest}:{number}", "这一行不足四列，读不出处置列", sep="\t")
        continue
    checked += 1
    disposition = cells[3]
    if disposition.startswith("已改"):
        fixed_rows += 1
        affirmed_quotes = [quote.group(1) for quote in QUOTE.finditer(disposition)
                           if not NEGATED_AFTER.match(disposition[quote.end():])]
        if not affirmed_quotes:
            print("BAD", f"{latest}:{number}",
                  f"第 {cells[0]} 行处置列写「已改」，却没有用「」引一句今天正文里的原文（只写「0 命中」「不在了」的那种不算）",
                  sep="\t")
            continue
        named_paths = kb_paths_named_by(" ".join(cells[1:4]))
        if not named_paths:
            unnamed_rows += 1
        searched_paths = named_paths or set(kb_body_by_path)
        for quote in affirmed_quotes:
            quote_count += 1
            wanted = normalized(quote)
            if any(wanted in kb_body_by_path[path] for path in searched_paths):
                continue
            elsewhere = sorted(os.path.relpath(path, ".claude/kb") for path, text in kb_body_by_path.items()
                               if path not in searched_paths and wanted in text)
            where_else = f"；只在 {'、'.join(elsewhere[:3])} 里有，那一行没点名它" if elsewhere else ""
            named_shown = "、".join(os.path.relpath(path, ".claude/kb") for path in sorted(named_paths)) or "kb 全部正文"
            print("BAD", f"{latest}:{number}",
                  f"第 {cells[0]} 行处置列写「已改」，引的「{quote[:40]}」在 {named_shown} 的今天正文里找不到{where_else}",
                  sep="\t")
        continue
    cited = set(re.findall(r"\bC\d+\b", disposition))
    if cited & open_owed:
        continue
    if cited & repaid_owed:
        print("BAD", f"{latest}:{number}",
              f"第 {cells[0]} 行处置列只点名了已还清的 {'、'.join(sorted(cited & repaid_owed))}——账还了，这一行的矛盾还在",
              sep="\t")
        continue
    if cited:
        print("BAD", f"{latest}:{number}",
              f"第 {cells[0]} 行处置列点名 {'、'.join(sorted(cited))}，而 checks-owed.md 里没有这个号",
              sep="\t")
        continue
    print("BAD", f"{latest}:{number}",
          f"第 {cells[0]} 行处置列写「{disposition[:40]}」，既不是「已改」、也没点名一个还开着的欠账号",
          sep="\t")

print("COUNT", latest, checked, len(rows), fixed_rows, quote_count, unnamed_rows, sep="\t")
PY
)"; then
  echo "  ✗ 扫描没跑完：内嵌 python 自己出错了，一行都没判"
  printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
  echo "    → 怎么办：按上面的报错修 .claude/gate.d/81-audit-contradictions.sh 里那段 python；"
  echo "      这一步没跑完就是什么都没查，不是通过。"
  exit 1
fi

if ! grep -q '^COUNT' <<<"$report"; then
  echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（多半是它自己崩了）"
  echo "    → 怎么办：单独跑一遍这道阶段看 python 的报错；没有 COUNT 就是一行都没判，"
  echo "      而汇总行看着与判过了一模一样。"
  exit 1
fi

mapfile -t bad < <(grep -E '^BAD' <<<"$report")

if ((${#bad[@]})); then
  echo "  ✗ 最近一次总审核第五节有行没有去向："
  while IFS=$'\t' read -r _ where why; do
    printf '      %s：%s\n' "$where" "$why"   # gate-lint:detail
  done < <(printf '%s\n' "${bad[@]}")
  echo "    → 怎么办：给这一行一个去向——改了就把处置列写成「已改…」，并用「」引一句今天正文里的原文、"
  echo "      点名它住在哪份文件（D / E 编号、C 号、I- 号或文件名）；引的那句今天找不到，就是那处又被改过，"
  echo "      回去现查这一行的矛盾还在不在：不在了就换成今天的原文，又回来了就当它没改。"
  echo "      还没改就在 .claude/kb/checks-owed.md 开着的那张表里立一笔账、把那个编号写进处置列。"
  echo "      只点名已还清的号不算；挂账表 .claude/audit-rows-pending 已关闭，不许往里加行。"
  exit 1
fi

read -r _ where checked total fixed quotes unnamed < <(grep '^COUNT' <<<"$report")
echo "  ✓ 最近一次总审核第五节每一行都有去向（$where：第五节 ${total} 行，判了 ${checked} 行；写「已改」的 ${fixed} 行引了 ${quotes} 句原文，都在那一行点名的文件的今天正文里找得到；其中 ${unnamed} 行没点名 kb 文件，按 kb 全部正文找）"
exit 0
