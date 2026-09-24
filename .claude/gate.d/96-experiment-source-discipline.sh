#!/usr/bin/env bash
# gate-stage: 实验源码的两条读数纪律——种子不许折叠，读数不许恒为字面量 0
#
# 判据：扫 `research/e7-index-bench/src/bin/*.rs`，两条各自成段、两条都判完再退出（一次把问题说全）。
#   ① C59（种子折叠成同一个状态）：名字里带 `seed` 的标识符后面直接跟 `| 1` 或 `& !1`，判红。
#      `seed | 1` 把 2 与 3、4 与 5 折成同一个状态：命令行给五个种子，实际只有三个访问模式，
#      轮间变异系统性偏小——而那个变异正是「这个数稳不稳」的唯一依据。
#      改法是先过一次乘法混淆再置位：`seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1`，
#      奇数乘子是双射，不同种子给不同状态。
#   ② C60（恒定读数没有故障注入自证）：本文件里声明的整型结构体字段，只要满足两条就判红——
#      它在整个文件里从来没出现在写入位置（`= 非零表达式`、`+=` 这类复合赋值、`&mut`），
#      而它每一处结构体字面量初始化都是字面量 0。这样的字段读出来恒为 0，
#      与「被测对象真的是 0」在产物里长得一模一样。
#
# 为什么：这两条都属于「装置说谎而门禁全绿」那一类。①让「N 轮」这个证据强度虚报，
# ②让一个恒定读数冒充实测读数（实测两次同型，E69（反向索引取权威态的增量维护代价）
# 的 `units_read_on_commit` 零处自增，而对照臂那一侧有实打实的自增）。
# 两条的原文与实测在 `.claude/kb/checks-owed.md` 欠着那张表的 C59、C60 两行。
#
# 存量违规怎么办：改一处就会改掉那个实验的全部产物，跟着要重跑、要逐个回对正文引的数
# （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「重跑之后要回对正文」）。
# 所以存量登记进两张豁免表，各挂在一条开着的欠账上：
#   `.claude/gate.d/experiment-seed-fold-lag.tsv`（挂 C59）
#   `.claude/gate.d/experiment-constant-reading-lag.tsv`（挂 C60）
# 两张表都是四列用制表符分隔：文件路径、定位（①写行号，②写 `结构体.字段`）、欠账编号、为什么还没改。
# 表对它们的三条闸：编号要在欠账表欠着那一张里找得到（按 `lib-owed.py` 读，67、92 号用的是同一份：
# 「### 已还清」整行标题之前的是开着的；认不出那个标题就判红，不对着一张认不出的表判）；每一行都要对得上这一轮真扫出来的一处违规
# （对不上就是已经改好了或者行号挪了，删掉或改掉这一行）；**只缩不涨**——
# 与基准提交比，同一个文件的登记行数不许变多，基准里没有的文件不许出现。
# 少了最后这一条，「新写的实验不在豁免表里判红」一行 tsv 就能绕过去。
#
# ⚠️ 射程，以及罩不到的是什么：
#   ① 只认名字里带 `seed` 的标识符这一种写法。先把种子搬进别的名字再折叠
#      （`let base = seed; let mut state = base | 1;`）它一个字都不说；
#      折叠之后再混淆（`(seed | 1) ^ K`、`(seed | 1).wrapping_mul(K)`）它认得——折叠发生在混淆之前。
#      它判的是表达式形态，不追这个值是不是真被当成 PRNG 状态用了：`seed | 1` 折叠种子，
#      拿去做什么都一样折。
#   ② 只认本文件里 `struct X { 字段: 整型 }` 声明的字段。局部变量的恒定读数、非整型字段、
#      别的 crate 里声明的结构体、宏展开出来的字段，一个都罩不到。
#      字段初始化用非字面量表达式（`field: total`）算写过——那是一次真读数；`field: 0` 不算。
#      它也判不了 C60 的另一半（每个进结论的读数都要有一次让它变号的故障注入自证），那一半要人做。
#   两条都只看注释与字符串之外的源码。
#   两张豁免表的「只缩不涨」按**每个文件的登记行数**比，不按逐行比：行号会随无关改动漂。
#   代价是同一个文件里删一行再加一行它看不出来。
#
# 判别力：fixtures/96-experiment-source-discipline.sh/red 是一个同时犯两条的小仓，另带一张挂在认不出的欠账表上的豁免表，必须判红；
# green 是同一份源码加上两张对得上的豁免表，必须判绿。
#
#   bash .claude/gate.d/96-experiment-source-discipline.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
OWED_LIBRARY="$(cd "$(dirname "$0")" && pwd)/lib-owed.py"
cd "$ROOT" 2>/dev/null || exit 2
base=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  # 基准取法与 56、68、69、75、97 号同一份：research/scripts/changed-paths.sh 的 gate 取法
  LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
  # shellcheck source=../../research/scripts/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  base="$(gate_diff_base gate)"
fi
python3 - "$base" "$OWED_LIBRARY" <<'PY'
import glob, importlib.util, os, re, subprocess, sys

base = sys.argv[1]
owed_library_spec = importlib.util.spec_from_file_location("owed", sys.argv[2])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)
BIN_GLOB = "research/e7-index-bench/src/bin/*.rs"
OWED_PATH = ".claude/kb/checks-owed.md"
SEED_LAG = ".claude/gate.d/experiment-seed-fold-lag.tsv"
READING_LAG = ".claude/gate.d/experiment-constant-reading-lag.tsv"

INTEGER = r"(?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize)"
# 名字带 seed 的标识符紧跟 `| 1` 或 `& !1`。`1` 后面允许一个整型后缀，
# 不允许再跟数字或下划线——`seed | 16` 与 `seed | 1_000` 不是这个形态。
SEED_FOLD = re.compile(r"(?<![\w.])(\w*[sS]eed\w*)\s*(?:\|\s*1|&\s*!\s*1)(?:" + INTEGER + r")?(?![\w.])")
SEED_IDENT = re.compile(r"(?<![\w.])\w*[sS]eed\w*(?![\w])")
STRUCT_HEAD = re.compile(r"\bstruct\s+(\w+)\s*(?:<[^{};]*>)?\s*\{")
STRUCT_FIELD = re.compile(r"(?:^|,)\s*(?:pub\s+)?(\w+)\s*:\s*" + INTEGER + r"\s*(?=,|$)", re.M)
LITERAL_ZERO = re.compile(r"^0(?:" + INTEGER + r")?$")

failures = []          # 每项是一块要打印的拒绝：(摘要, 明细列表, 出路列表)
def reject(summary, details, steps):
    failures.append((summary, details, steps))

def blank_out(text):
    """把注释与字符串字面量换成同样长度的空白，行号与列号都不变。"""
    out, index, size = [], 0, len(text)
    while index < size:
        char = text[index]
        if char == "/" and index + 1 < size and text[index + 1] == "/":
            end = text.find("\n", index)
            end = size if end < 0 else end
            out.append(" " * (end - index)); index = end
        elif char == "/" and index + 1 < size and text[index + 1] == "*":
            depth, end = 1, index + 2
            while end < size and depth:
                if text[end] == "/" and end + 1 < size and text[end + 1] == "*": depth += 1; end += 2
                elif text[end] == "*" and end + 1 < size and text[end + 1] == "/": depth -= 1; end += 2
                else: end += 1
            out.append("".join(c if c == "\n" else " " for c in text[index:end])); index = end
        elif char == '"':
            end = index + 1
            while end < size:
                if text[end] == "\\": end += 2; continue
                if text[end] == '"': end += 1; break
                end += 1
            out.append("".join(c if c == "\n" else " " for c in text[index:end])); index = end
        else:
            out.append(char); index += 1
    return "".join(out)

# ── 被扫集合：一次扫描，两条判据与「没查的是哪些」都从它现算 ───────────────
paths = sorted(glob.glob(BIN_GLOB))
if not paths:
    print(f"  ✗ {BIN_GLOB} 一个文件都没扫到")
    print("     → 怎么办：实验 bin 目录搬了家就同步改这个阶段里的 BIN_GLOB（.claude/rules/path-moves.md）；")
    print("               扫到 0 个对象而报绿，与判过了在门禁输出里一模一样。")
    sys.exit(1)

sources, unreadable = {}, []
for path in paths:
    try:
        sources[path] = blank_out(open(path, encoding="utf-8").read())
    except OSError as error:
        unreadable.append(f"{path}：{error}")

# ① 折叠写法：文件 → [(行号, 原样片段)]
seed_folds, seed_identifier_count, files_without_seed = {}, 0, []
for path, text in sources.items():
    seed_identifier_count += len(SEED_IDENT.findall(text))
    if not SEED_IDENT.search(text):
        files_without_seed.append(os.path.basename(path))
    hits = []
    for line_number, line in enumerate(text.split("\n"), 1):
        hits.extend((line_number, match.group(0).strip()) for match in SEED_FOLD.finditer(line))
    if hits:
        seed_folds[path] = hits

# ② 恒为字面量 0 的整型字段：文件 → [(结构体.字段, 初始化处数)]
constant_readings, field_count, files_without_field = {}, 0, []
for path, text in sources.items():
    declared = {}
    for head in STRUCT_HEAD.finditer(text):
        # 花括号要数着配对，不能拿「下一个 }」凑合：一行写完的结构体会一路吃到别的结构体的收尾，
        # 把别人的字段记到自己名下（实测本仓有一行写完的结构体声明）。
        depth, cursor = 1, head.end()
        while cursor < len(text) and depth:
            if text[cursor] == "{": depth += 1
            elif text[cursor] == "}": depth -= 1
            cursor += 1
        if depth:
            continue
        for field in STRUCT_FIELD.finditer(text[head.end():cursor - 1]):
            declared.setdefault(field.group(1), head.group(1))
    field_count += len(declared)
    if not declared:
        files_without_field.append(os.path.basename(path))
    frozen = []
    for field, owner in sorted(declared.items()):
        name = re.escape(field)
        prefix = r"(?<![\w.])(?:\w+\s*\.\s*)*" + name
        compound = re.search(prefix + r"\s*(?:[-+*/|&^%]=|<<=|>>=)", text)
        assigned = [match.group(1).strip() for match in re.finditer(prefix + r"\s*=(?!=)\s*([^;,}\n]+)", text)]
        borrowed = re.search(r"&\s*mut\s+(?:\w+\s*\.\s*)*" + name + r"\b", text)
        if compound or borrowed or any(not LITERAL_ZERO.match(value) for value in assigned):
            continue
        initialised = [match.group(1).strip() for match in re.finditer(r"(?<![\w.])" + name + r"\s*:\s*([^,}\n]+)", text)]
        initialised = [value for value in initialised if not re.fullmatch(INTEGER, value)]
        if initialised and all(LITERAL_ZERO.match(value) for value in initialised):
            frozen.append((f"{owner}.{field}", len(initialised)))
    if frozen:
        constant_readings[path] = frozen

if unreadable:
    reject(f"{len(unreadable)} 个实验源码读不动，这一轮对它们两条判据都没跑：",
           unreadable,
           ["怎么办：把读不动的原因修掉（权限、编码）再跑；读不动的文件在这一道里既不算判过也不算判红，",
            "          而一个整批读不动的目录会让两条判据都扫到 0 项，末尾照样报绿。"])

# ── 豁免表 ─────────────────────────────────────────────────────────────
owed_table = owed_library.read_owed_table(OWED_PATH)
owed_open = set(owed_table.open_names)
owed_unrecognised = owed_table.file_found and not owed_table.paid_heading_found

def read_lag(path, label):
    """读一张豁免表，返回 {(文件, 定位): (欠账编号, 行号)}；形状坏了的行当场登记成拒绝。"""
    rows, malformed = {}, []
    if not os.path.isfile(path):
        return rows
    for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
        line = line.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 4 or not all(field.strip() for field in fields):
            malformed.append(f"第 {line_number} 行：{line}")
            continue
        source, locator, number, _reason = (field.strip() for field in fields)
        rows[(source, locator)] = (number, line_number)
    if malformed:
        reject(f"{label} 里 {len(malformed)} 行不是四列、或有空格子：", malformed,
               [f"怎么办：{path} 每行四列用制表符分隔：文件路径、定位、欠账编号、为什么还没改；四列都要有内容，理由不许省。"])
    return rows

def baseline_counts(path):
    """基准提交里这张表每个文件登记了几行；表在基准里不存在时返回 None（这次是初次登记）。"""
    if not base:
        return None
    result = subprocess.run(["git", "show", f"{base}:{path}"], capture_output=True, text=True)
    if result.returncode != 0:
        return None
    counts = {}
    for line in result.stdout.split("\n"):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 4:
            continue
        counts[fields[0].strip()] = counts.get(fields[0].strip(), 0) + 1
    return counts

def judge(label, owed_number, lag_path, violations, locator_of, what_to_fix, how_to_fix):
    """一条判据一段：没豁免的违规、指向空处的编号、对不上的登记行、涨了的登记行，四样都判完。"""
    lag = read_lag(lag_path, label)
    excused, unexcused = [], []
    for path, hits in sorted(violations.items()):
        for hit in hits:
            locator = locator_of(hit)
            if (path, locator) in lag:
                excused.append((path, locator))
            else:
                unexcused.append(f"{path}:{locator}    {hit[1] if isinstance(hit[1], str) else ''}".rstrip())
    if unexcused:
        reject(f"{label}：{len(unexcused)} 处违规没有登记在豁免表里：", unexcused,
               [f"怎么办：两条出路，别两边都不做。① 就地改掉——{how_to_fix}；",
                "               改了就要重跑这个实验并按新产物逐个回对正文引的数（门禁 87 号会逐字节判红直到重跑）。",
                f"               ② 今天改不动的，登记进 {lag_path}：四列写文件路径、定位、{owed_number}、为什么还没改。",
                f"               ⚠️ 这张表只缩不涨——新写的实验源码一律走出路 ①，{what_to_fix}。"])
    if lag and owed_unrecognised:
        reject(f"{label}：豁免表有 {len(lag)} 行要核欠账编号，而 {OWED_PATH} 里认不出「### 已还清」那一行标题，分不出哪些账还开着", [],
               ["怎么办：欠账表按「### 已还清」整行标题切成开着与还清两段（.claude/gate.d/lib-owed.py）；标题改了名或丢了就改回来，",
                "               别让这一道对着一张认不出的表判——认不出时连历史版本节里的表格行都会被算成开着的账。"])
    dangling = [f"{source}:{locator} 挂的是 {number}（第 {line_number} 行）"
                for (source, locator), (number, line_number) in sorted(lag.items())
                if number not in owed_open and not owed_unrecognised]
    if dangling:
        reject(f"{label}：豁免表里 {len(dangling)} 行挂的欠账编号不在 {OWED_PATH} 欠着那张表里：", dangling,
               ["怎么办：登记一条豁免，等于承认这一处今天还没改，那笔账要有人排期。",
                f"               编号写成欠着那张表里开着的一行（这一条是 {owed_number}）；",
                "               那笔账已经还清了，就把这些豁免行一起删掉——账还清了就不该再有豁免。"])
    live = {(path, locator_of(hit)) for path, hits in violations.items() for hit in hits}
    stale = [f"{source}:{locator}（第 {line_number} 行）"
             for (source, locator), (_number, line_number) in sorted(lag.items())
             if (source, locator) not in live]
    if stale:
        reject(f"{label}：豁免表里 {len(stale)} 行对不上这一轮扫出来的任何一处违规：", stale,
               ["怎么办：这一处已经改好了就删掉这一行——留着的豁免行会让人以为那个文件还欠着，",
                "               而它其实已经干净了（这张表只缩不涨，改好一处就删一行）；",
                "               只是定位挪了（行号随无关改动漂、字段改了名），就把定位那一列改成现在的值。"])
    baseline = baseline_counts(lag_path)
    grown = []
    if baseline is not None:
        current = {}
        for source, _locator in lag:
            current[source] = current.get(source, 0) + 1
        for source, count in sorted(current.items()):
            was = baseline.get(source, 0)
            if count > was:
                grown.append(f"{source}：基准 {base} 里 {was} 行，现在 {count} 行")
    if grown:
        reject(f"{label}：豁免表涨了——{len(grown)} 个文件的登记行数比基准多：", grown,
               ["怎么办：这张表只缩不涨。新冒出来的违规一律就地改，不许加一行豁免把它按下去——",
                "               加得了一行，这一道对新写的实验源码就什么都拦不住了。",
                f"               真有非加不可的理由，那是一次规则变更：先改 {lag_path} 的表头注释与这个阶段的判据，说清为什么。"])
    return len(excused), len(lag), baseline is not None

seed_excused, seed_rows, seed_compared = judge(
    "C59（种子折叠成同一个状态）", "C59", SEED_LAG, seed_folds,
    lambda hit: str(hit[0]),
    "它们一处都不许出现在这张表里",
    "把 `seed | 1` 改成先乘法混淆再置位，例 `seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1`")
reading_excused, reading_rows, reading_compared = judge(
    "C60（恒定读数没有故障注入自证）", "C60", READING_LAG, constant_readings,
    lambda hit: hit[0],
    "它们一处都不许出现在这张表里",
    "要么让这个字段真的被写（把它算出来），要么删掉它——一个恒为 0 的读数不该进产物")

if failures:
    for summary, details, steps in failures:
        print(f"  ✗ {summary}")  # gate-lint:summary
        for detail in details:
            print(f"      {detail}")  # gate-lint:detail
        for step in steps:
            print(f"     → {step}" if step.startswith("怎么办") else f"       {step}")
    sys.exit(1)

def listing(names):
    """一行塞几个名字，按显示宽度折行——名字本身不许被折断，折断的文件名 grep 不出来。"""
    lines, current = [], ""
    for index, name in enumerate(names):
        piece = name + ("、" if index + 1 < len(names) else "")
        if current and len(current) + len(piece) > 100:
            lines.append("        " + current); current = ""
        current += piece
    if current:
        lines.append("        " + current)
    return "\n".join(lines)

seed_hits = sum(len(hits) for hits in seed_folds.values())
reading_hits = sum(len(hits) for hits in constant_readings.values())
compared = "、".join(name for name, done in (("C59", seed_compared), ("C60", reading_compared)) if done)
compared = compared or "两张豁免表在基准里都还不存在，这一次是初次登记，没有基线可比"
print(f"  ✓ 实验源码纪律：扫了 {len(paths)} 个文件（读不动的 0 个）；"
      f"C59 查了 {seed_identifier_count} 处名字带 seed 的标识符，折叠写法 {seed_hits} 处（豁免表 {seed_rows} 行，放行 {seed_excused} 处）；"
      f"C60 查了 {field_count} 个整型结构体字段，恒为字面量 0 的 {reading_hits} 个（豁免表 {reading_rows} 行，放行 {reading_excused} 处）；"
      f"只缩不涨（基准 {base or '不是 git 仓，这一条没跑'}）：{compared}")
print(f"    C59 对这 {len(files_without_seed)} 个文件没有对象可判（一处名字带 seed 的标识符都没有）：")
print(listing(files_without_seed) if files_without_seed else "        （没有）")
print(f"    C60 对这 {len(files_without_field)} 个文件没有对象可判（一个整型结构体字段都没有）：")
print(listing(files_without_field) if files_without_field else "        （没有）")
PY
