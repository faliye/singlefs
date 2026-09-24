#!/usr/bin/env bash
# gate-stage: feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里
#
# 还 C11（feature bit 跳号）。D15（格式冻结政策） 已定项 10 逐字写着记账落在 `.claude/kb/feature-bits.md`，
# 位号的唯一登记位是 D15（格式冻结政策） 已定项 4 那张表；两处都是人手写的，没有任何东西盯着它们与代码同步。
# 回收一个位的后果不是「文档不准」：旧镜像那一位会被新代码读成「新特性已启用」，文件系统会真的做错事。
#
# 四条判据，任一条不成立判红：
#   ① 记账表的（类别，位号）集合与 D15（格式冻结政策） 已定项 4 登记表逐行对得上，
#      且记账表的「语义一句话」逐字包含登记表同一位的「含义」——这一条挡住记账表与登记位分叉；
#   ② 每张位图内位号严格递增、不跳号（已分配的位必须是 0..n-1）；
#   ③ `crates/*/src/**/*.rs` 里名字带 INCOMPAT / COMPAT_RO / COMPAT 段的常量，
#      值解得出位号的，那个位号必须在记账表里；
#   ④ 同一个（类别，位号）出现两行判红——退役位被赋予新语义就是这个形状。
#
# 射程：③ 只认**写在一行里、值是位掩码字面量**的常量声明。值由别的常量合成的（`SUPPORTED_INCOMPAT_BITS`）、
# 声明跨行的，解不出位号，逐个列进成功那句的「没判位号的」名单——那份名单与被扫集合出自同一次扫描，现算。
# 不带 feature bit 常量名的裸字面量（`crates/singlefs-checker/src/lib.rs` 判 incompat 时的 `0x01`）不在射程里，靠 review。
#
# 判别力：fixtures/93-feature-bits.sh/red 的记账表跳号、同一位登记两行不同语义、多出一位登记表里没有，
# 代码样本又引用了表里没有的一位，②③④ 与 ① 都必须报出来；green 只有位 0 一行、语义与登记表对得上，必须判绿。
#
#   bash .claude/gate.d/93-feature-bits.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -f .claude/kb/feature-bits.md ]] || { echo "  ! 没有 .claude/kb/feature-bits.md，本阶段无对象可判"; exit 77; }

python3 - <<'PY'
import glob, re, sys

FEATURE_BITS_PATH = ".claude/kb/feature-bits.md"
BITMAPS = ("incompat", "compat_ro", "compat")
BITS_PER_BITMAP = 256
EXPECTED_HEADER = ["位号", "类别", "名称", "引入版本", "引入 commit", "状态", "语义一句话"]

problems = []


def reject(summary, details, howto):
    problems.append((summary, details, howto))


def normalize(text):
    return re.sub(r"\s+", "", text.replace("*", "").replace("`", ""))


def is_separator(cells):
    return all(re.fullmatch(r":?-{2,}:?", cell) for cell in cells)


def read_table(lines, start_index, column_count):
    """从 lines[start_index] 起读一张 markdown 表，返回去掉表头与分隔行之后的每行单元格。"""
    rows = []
    for line in lines[start_index:]:
        stripped = line.strip()
        if not stripped.startswith("|"):
            break
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if len(cells) != column_count:
            break
        if is_separator(cells):
            continue
        rows.append(cells)
    return rows


# ── D15 已定项 4 登记表：位号的唯一登记位 ────────────────────────
decision_files = sorted(glob.glob(".claude/kb/decisions/15-*.md"))
d15_allocated = {}      # (类别, 位号) -> 含义
d15_unallocated = {}    # 类别 -> [(起, 止)]
d15_path = ""
if not decision_files:
    reject(
        "找不到 D15（格式冻结政策） 的决策文件（.claude/kb/decisions/15-*.md）——位号的唯一登记位不在，没法比对",
        [],
        "确认决策正文还在 .claude/kb/decisions/ 下；改了文件名就把 15- 这个编号前缀留着，本阶段按它找。",
    )
else:
    d15_path = decision_files[0]
    d15_lines = open(d15_path, encoding="utf-8").read().split("\n")
    section_start = section_end = -1
    for index, line in enumerate(d15_lines):
        if line.startswith("#### 已定项 4"):
            section_start = index
        elif section_start >= 0 and line.startswith("#### "):
            section_end = index
            break
    if section_end < 0:
        section_end = len(d15_lines)
    header_index = -1
    for index in range(section_start, section_end):
        if section_start < 0:
            break
        cells = [cell.strip() for cell in d15_lines[index].strip().strip("|").split("|")]
        if d15_lines[index].strip().startswith("|") and len(cells) == 4 and "bitmap" in cells[0] and "含义" in cells[2]:
            header_index = index
            break
    if header_index < 0:
        reject(
            f"{d15_path} 的「已定项 4」里找不到位分配登记表（表头要是 bitmap / 位 / 含义 / 出处 四列）",
            [f"已定项 4 这一节：第 {section_start + 1} 行起" if section_start >= 0 else "连「#### 已定项 4」这个标题都没找到"],
            "位号的唯一登记位就是那张表，表没了这一道就没有比对的对象。把表写回已定项 4，四列照旧：bitmap、位、含义、出处。",
        )
    else:
        malformed = []
        for cells in read_table(d15_lines, header_index + 1, 4):
            bitmap, bit_field, meaning, _source = cells
            if bitmap == "bitmap":
                continue
            if bitmap not in BITMAPS:
                malformed.append(f"类别「{bitmap}」不是 incompat / compat_ro / compat 之一：{' | '.join(cells)}")
                continue
            if re.fullmatch(r"\d+", bit_field):
                d15_allocated[(bitmap, int(bit_field))] = meaning
            elif re.fullmatch(r"\d+\.\.\d+", bit_field):
                if "未分配" not in meaning:
                    malformed.append(f"{bitmap} {bit_field} 是一段区间，含义里却没写「未分配」：{meaning}")
                    continue
                low, high = bit_field.split("..")
                d15_unallocated.setdefault(bitmap, []).append((int(low), int(high)))
            else:
                malformed.append(f"「位」这一格既不是位号也不是 a..b 区间：{' | '.join(cells)}")
        if malformed:
            reject(
                f"{d15_path} 的位分配登记表有行读不出来，位号集合拼不完整",
                malformed,
                "每行的「位」写成一个位号（已分配）或 a..b 区间（未分配，含义里要写「未分配」），类别只许 incompat / compat_ro / compat。",
            )
        elif not d15_allocated and not d15_unallocated:
            reject(
                f"{d15_path} 的位分配登记表一行都没读出来",
                [],
                "扫到 0 行不是通过。检查表是不是紧挨着表头、有没有被围栏包住；表的四列是 bitmap、位、含义、出处。",
            )
        else:
            holes = []
            for bitmap in BITMAPS:
                covered = sorted(bit for (bitmap_name, bit) in d15_allocated if bitmap_name == bitmap)
                for low, high in d15_unallocated.get(bitmap, []):
                    covered.extend(range(low, high + 1))
                covered_sorted = sorted(covered)
                if covered_sorted != list(range(BITS_PER_BITMAP)):
                    missing = sorted(set(range(BITS_PER_BITMAP)) - set(covered_sorted))
                    repeated = sorted({bit for bit in covered_sorted if covered_sorted.count(bit) > 1})
                    holes.append(f"{bitmap}：漏了 {len(missing)} 位（最小 {missing[0] if missing else '—'}）、重了 {len(repeated)} 位")
            if holes:
                reject(
                    f"{d15_path} 的位分配登记表自己没罩满 0..{BITS_PER_BITMAP - 1}，有洞或有重叠",
                    holes,
                    "每张位图的已分配位加上未分配区间要正好覆盖 0..255 一次。罩不满时「这一位归谁」就没有答案，比对也无从谈起。",
                )

# ── 记账表 ──────────────────────────────────────────────
feature_lines = open(FEATURE_BITS_PATH, encoding="utf-8").read().split("\n")
marker_index = -1
for index, line in enumerate(feature_lines):
    if line.strip() == "<!-- feature-bits:table -->":
        marker_index = index
        break
header_index = -1
if marker_index >= 0:
    for index in range(marker_index + 1, len(feature_lines)):
        stripped = feature_lines[index].strip()
        if stripped.startswith("|"):
            header_index = index
            break
        if stripped.startswith("#"):
            break

rows = []
if marker_index < 0 or header_index < 0:
    reject(
        f"{FEATURE_BITS_PATH} 里找不到记账表（要有一行 <!-- feature-bits:table -->，它后面第一张 markdown 表就是记账表）",
        [],
        "在记账表上面单独写一行 <!-- feature-bits:table -->，表头七列照 D15（格式冻结政策） 已定项 10：" + " | ".join(EXPECTED_HEADER),
    )
else:
    header_cells = [cell.strip() for cell in feature_lines[header_index].strip().strip("|").split("|")]
    if header_cells != EXPECTED_HEADER:
        reject(
            f"{FEATURE_BITS_PATH} 记账表的表头与 D15（格式冻结政策） 已定项 10 定的列序对不上",
            [f"读到：{' | '.join(header_cells)}", f"要的是：{' | '.join(EXPECTED_HEADER)}"],
            "七列照 D15（格式冻结政策） 已定项 10 写，顺序不许换——别处按列位取值，换了顺序取出来的就是别的东西。",
        )
    else:
        malformed = []
        for cells in read_table(feature_lines, header_index + 1, len(EXPECTED_HEADER)):
            bit_field, bitmap = cells[0], cells[1]
            if not re.fullmatch(r"\d+", bit_field):
                malformed.append(f"「位号」不是一个整数：{' | '.join(cells)}")
                continue
            if bitmap not in BITMAPS:
                malformed.append(f"「类别」不是 incompat / compat_ro / compat 之一：{' | '.join(cells)}")
                continue
            empty = [EXPECTED_HEADER[i] for i, cell in enumerate(cells) if not cell]
            if empty:
                malformed.append(f"{bitmap} 位 {bit_field}：这几列是空的：{'、'.join(empty)}")
                continue
            rows.append((int(bit_field), bitmap, cells[2], cells[3], cells[4], cells[5], cells[6]))
        if malformed:
            reject(
                f"{FEATURE_BITS_PATH} 记账表有行读不出来",
                malformed,
                "每行七格都要填：位号写整数，类别写 incompat / compat_ro / compat，代码侧没有常量的「名称」写破折号。",
            )

# ── ④ 同一个（类别，位号）出现两行 ───────────────────────────
seen = {}
duplicates = []
for bit, bitmap, name, version, commit, status, semantics in rows:
    key = (bitmap, bit)
    if key in seen:
        previous = seen[key]
        shape = "两行语义不同——退役位被赋予新语义就是这个形状" if normalize(previous) != normalize(semantics) else "两行语义相同，是重复登记"
        duplicates.append(f"{bitmap} 位 {bit}：{shape}；先写的是「{previous}」，后写的是「{semantics}」")
    else:
        seen[key] = semantics
if duplicates:
    reject(
        f"{FEATURE_BITS_PATH} 记账表里同一个位号登记了不止一行",
        duplicates,
        "一位只许一行。位一旦用过不许回收（D15（格式冻结政策） 已定项 10）：退役的位把状态改成「退役」、语义永久锁定为退役前最后一次使用的那一句，"
        "新特性另取一个没用过的位，不许改写旧行的语义。",
    )

# ── ② 每张位图内严格递增、不跳号 ─────────────────────────────
order_problems = []
for bitmap in BITMAPS:
    in_file_order = [bit for bit, row_bitmap, *_rest in rows if row_bitmap == bitmap]
    if not in_file_order:
        continue
    if any(later <= earlier for earlier, later in zip(in_file_order, in_file_order[1:])):
        order_problems.append(f"{bitmap}：表里的位号不是严格递增的，读到的次序是 {in_file_order}")
    distinct = sorted(set(in_file_order))
    if distinct != list(range(len(distinct))):
        gaps = sorted(set(range(distinct[-1] + 1)) - set(distinct))
        order_problems.append(f"{bitmap}：已分配的位跳号了，缺 {gaps}（已分配的位必须是 0..{len(distinct) - 1}）")
if order_problems:
    reject(
        f"{FEATURE_BITS_PATH} 记账表的位号次序不对",
        order_problems,
        "每张位图的位号从 0 开始严格递增、一位不跳（D15（格式冻结政策） 已定项 10「位号严格递增、不许跳号占位」）。"
        "跳号占位等于替一个还没人写的特性把位占住，而占位的那一位在盘上与「已启用」长得一模一样。",
    )

# ── ① 与 D15 登记表逐行对得上 ───────────────────────────────
if rows and (d15_allocated or d15_unallocated):
    registry_keys = set(d15_allocated)
    ledger_keys = {(bitmap, bit) for bit, bitmap, *_rest in rows}
    mismatches = []
    for bitmap, bit in sorted(ledger_keys - registry_keys):
        where = ""
        for low, high in d15_unallocated.get(bitmap, []):
            if low <= bit <= high:
                where = f"，登记表把它列在未分配区间 {low}..{high} 里"
        mismatches.append(f"记账表登记了 {bitmap} 位 {bit}，{d15_path} 已定项 4 的登记表里没有这一位{where}")
    for bitmap, bit in sorted(registry_keys - ledger_keys):
        mismatches.append(f"{d15_path} 已定项 4 把 {bitmap} 位 {bit} 分出去了（{d15_allocated[(bitmap, bit)]}），记账表里没有它")
    for bit, bitmap, name, version, commit, status, semantics in rows:
        meaning = d15_allocated.get((bitmap, bit))
        if meaning is None:
            continue
        if normalize(meaning) not in normalize(semantics):
            mismatches.append(
                f"{bitmap} 位 {bit} 的语义与登记表对不上：登记表的「含义」是「{meaning}」，记账表的「语义一句话」是「{semantics}」"
            )
    if mismatches:
        reject(
            f"记账表 {FEATURE_BITS_PATH} 与位号的唯一登记位（{d15_path} 已定项 4）对不上",
            mismatches,
            "位号的唯一登记位是 D15（格式冻结政策） 已定项 4 那张表，记账表只记账：两处不一致时改记账表，不改登记表。"
            "「语义一句话」要逐字包含登记表同一位的「含义」——整行抄过来再往后接细节，别做一个更短的版本。",
        )

# ── ③ 代码里引用的位必须在记账表里 ────────────────────────────
CONST_NAME = re.compile(r"\bconst\s+([A-Z][A-Z0-9_]*)")
CONST_FULL = re.compile(r"\bconst\s+([A-Z][A-Z0-9_]*)\s*:\s*[^=;]+=\s*([^;]+);")
NUMERIC = re.compile(r"^(?:0x([0-9a-fA-F_]+)|([0-9_]+))(?:[ui](?:8|16|32|64|128|size))?$")
SHIFT = re.compile(r"^1(?:[ui](?:8|16|32|64|128|size))?\s*<<\s*([0-9_]+)(?:[ui](?:8|16|32|64|128|size))?$")


def bitmap_of(constant_name):
    segments = constant_name.split("_")
    if "INCOMPAT" in segments:
        return "incompat"
    for index in range(len(segments) - 1):
        if segments[index] == "COMPAT" and segments[index + 1] == "RO":
            return "compat_ro"
    if "COMPAT" in segments:
        return "compat"
    return None


def decode_bit(expression):
    stripped = re.sub(r"//.*$", "", expression).strip()
    match = NUMERIC.fullmatch(stripped)
    if match:
        value = int(match.group(1).replace("_", ""), 16) if match.group(1) else int(match.group(2).replace("_", ""))
        if value != 0 and value & (value - 1) == 0:
            return value.bit_length() - 1, ""
        return None, f"值 `{stripped}` 不是单个位掩码"
    match = SHIFT.fullmatch(stripped)
    if match:
        return int(match.group(1).replace("_", "")), ""
    return None, f"值 `{stripped}` 不是位掩码字面量"


rust_files = sorted(glob.glob("crates/*/src/**/*.rs", recursive=True))
decoded = []
undecided = []
for path in rust_files:
    for line_number, line in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        expressions = {name: expression for name, expression in CONST_FULL.findall(line)}
        for name in CONST_NAME.findall(line):
            bitmap = bitmap_of(name)
            if bitmap is None:
                continue
            if name not in expressions:
                undecided.append((name, path, line_number, "常量声明没写在一行里，解不出值"))
                continue
            bit, reason = decode_bit(expressions[name])
            if bit is None:
                undecided.append((name, path, line_number, reason))
            else:
                decoded.append((bitmap, bit, name, path, line_number))

ledger_keys = {(bitmap, bit) for bit, bitmap, *_rest in rows}
unregistered = [
    f"{path}:{line_number} 的 {name} 指 {bitmap} 位 {bit}，{FEATURE_BITS_PATH} 的记账表里没有这一位"
    for bitmap, bit, name, path, line_number in decoded
    if (bitmap, bit) not in ledger_keys
]
if unregistered:
    reject(
        f"代码引用了记账表里不存在的 feature bit 位",
        unregistered,
        f"先在 {FEATURE_BITS_PATH} 的记账表里给这一位写一行（七列齐全，引入 commit 用 git log 查，别填猜的），"
        "并在 D15（格式冻结政策） 已定项 4 的登记表里把它从未分配区间里切出来——两处都到位这一道才绿。",
    )

for summary, details, howto in problems:
    print(f"  ✗ {summary}")                      # gate-lint:summary
    for detail in details:
        print(f"     {detail}")                  # gate-lint:detail
    print(f"     → 怎么办：{howto}")
if problems:
    sys.exit(1)

skipped_text = "；".join(
    f"{name}（{path}:{line_number}，{reason}）" for name, path, line_number, reason in undecided
) or "无"
print(
    f"  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 {len(rows)} 位、"
    f"登记表分出去 {len(d15_allocated)} 位；"
    f"扫了 {len(rust_files)} 个 .rs，认出 {len(decoded) + len(undecided)} 处 feature bit 常量、解出位号 {len(decoded)} 处；"
    f"没判位号的 {len(undecided)} 处：{skipped_text}）"
)
PY
