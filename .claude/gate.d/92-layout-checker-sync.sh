#!/usr/bin/env bash
# gate-stage: 布局的格式常量变了，它的 checker 判定路径要在同一次改动里跟
#
# 判据：`.claude/gate.d/layouts.tsv` 一套布局一行，四列用制表符分隔（布局名 / incompat 位号 /
# 格式定义路径 / checker 判定路径，后两列逗号分隔）。四条，任一条不成立判红：
#   ① 每一行四列齐全，位号是十进制数字，后两列至少各一条路径；
#   ② 表里每条路径都存在——指向不存在的路径，等于那一格没人管，而表看着还是满的；
#   ③ 每个 incompat 位号在 `.claude/kb/decisions/15-格式冻结政策.md` 已定项 4 那张登记表里有行，
#      且布局名与那一行的含义列逐字相同（位号的唯一登记位在那张表，这里只引用，不另立第二处）；
#   ④ 这次改动让某套布局的格式定义里「常量名 → 值」的集合发生增、删或改值，
#      而它的 checker 判定路径一个都没被这次改动碰过 ⇒ 判红。
#
# 为什么：一套布局的格式改了而它的 checker 判定路径没跟，门禁照样全绿——checker 会拿旧口径
# 去判新字节，而「checker 没报错」被读成「镜像是好的」。C13（checker 判定失效） 拦的是判定逻辑失效，
# 这一道拦的是它最常见的成因：格式先走了一步。
#
# ⚠️ 射程：④ 判的是「格式变了没人跟」，判不了 checker 跟得对不对——后者要人看。
# ⚠️ 这里的变更探测器**不是** D15（格式冻结政策） 已定项 3 要的那个 `spec_hash`：那一个恒对
# `format-spec/<组件>.toml` 求，而那份 toml 今天一个都不存在（C119（冻结组件没有 spec 文件））。
# 在它出现之前，这一道拿「常量名 → 值的集合」当变更探测器——哈希本来就只是变更探测器，
# 而一次 diff 也是。spec toml 有了就把抽取源换过去，四条判据不变。
# 常量从两处抽：`.rs` 里顶格的 `pub const 名字: 类型 = 值;`，`.md` 里的 `<!-- format-const: 名字 = 值 … -->` 标记。
# 两种都按 `lib-format-const.py` 读（27、39 号用的是同一份）：标记按文法读不出来的、同一份格式定义里
# 同一个名字登记了不止一次的，这一道判红——前者在变更探测里看不见，后者两个值里改了哪个说不清。
# `.rs` 的值按空白归一后的原文比（value_reading="normalized_text"），不像 27 号那样要求整数字面量：
# 格式常量模块里有 `DATA_UNIT_HEADER_BYTES + …` 这类算出来的常量，这一道只问它变没变。
#
# 改动范围（基准与路径集合）都取共用脚本 research/scripts/changed-paths.sh：gate_diff_base gate 与
# gate_changed_paths 带未跟踪文件，不在这里另算一份（门禁 64 号判）。git 调用一律带 `-c core.quotepath=false`：
# 默认的 quoting 把中文路径打成八进制引号串，与布局清单里的路径逐字比对不上，checker 明明跟了也判「没碰」。
#
# 滞后登记表指的欠账号开没开着，按 `lib-owed.py` 读 checks-owed.md（67、96 号用的是同一份）：
# 「### 已还清」整行标题之前的是开着的；认不出那个标题就判红，不对着一张认不出的表判。
#
# 滞后表、标记与第 ④ 条三样都判完再退出，一次把问题说全。
#
# 判别力：fixtures/92-layout-checker-sync.sh/red 是一个改了格式常量、没碰 checker 的小仓，
# 另带一份多写了键又重复登记的格式定义、一张挂在认不出的欠账表上的滞后表，必须判红；
# green 是同一处改动加上 checker 跟着改（checker 路径是中文文件名），另带一张滞后表，挂的欠账号
# 排在一行正文提到「### 已还清」的开着的账后面，必须判绿。
#
#   bash .claude/gate.d/92-layout-checker-sync.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
LIBRARY_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT" 2>/dev/null || exit 2
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
# 基准取法与 56、68、69、75、97 号同一份：research/scripts/changed-paths.sh 的 gate 取法
LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
# shellcheck source=../../research/scripts/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
base="$(gate_diff_base gate)"
CHANGED_PATHS="$(mktemp)"
trap 'rm -f "$CHANGED_PATHS"' EXIT
# 路径集合先落到文件、判过退出码再交给 python：git 失败时集合静默为空，会被读成「这次什么都没碰」
gate_changed_paths "$base" untracked > "$CHANGED_PATHS" || {
  echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base，gate_changed_paths 退出码 $?）"
  echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到改动范围时第 ④ 条什么都没比，不是通过。"
  exit 1
}
python3 - "$base" "$LIBRARY_DIRECTORY" "$CHANGED_PATHS" <<'PY'
import importlib.util, os, re, subprocess, sys

base, library_directory, changed_paths_file = sys.argv[1], sys.argv[2], sys.argv[3]

def load_library(module_name, file_name):
    library_spec = importlib.util.spec_from_file_location(module_name, os.path.join(library_directory, file_name))
    library = importlib.util.module_from_spec(library_spec)
    library_spec.loader.exec_module(library)
    return library

format_const = load_library("format_const", "lib-format-const.py")
owed_library = load_library("owed", "lib-owed.py")
# 27 号读 .rs 的值要整数字面量（integer_literal）；这一道沿用只做空白归一的旧口径，
# 两道该不该统一成一种还没定，统一时改这一个名字。
RUST_VALUE_READING = "normalized_text"
GIT = ["git", "-c", "core.quotepath=false"]

manifest_path = ".claude/gate.d/layouts.tsv"
lag_path = ".claude/gate.d/layouts-checker-lag.tsv"
registry_path = ".claude/kb/decisions/15-格式冻结政策.md"
owed_path = ".claude/kb/checks-owed.md"

def fail(message, steps):
    print(f"  ✗ {message}")
    for step in steps:
        print(f"     → {step}")
    sys.exit(1)

if not os.path.isfile(manifest_path):
    fail(f"没有 {manifest_path}", [
        "怎么办：建这张表，四列用制表符分隔：布局名、incompat 位号、格式定义路径、checker 判定路径（后两列逗号分隔）；# 开头的行是注释。",
    ])

rows, malformed = [], []
for line_number, line in enumerate(open(manifest_path, encoding="utf-8"), 1):
    line = line.rstrip("\n")
    if not line.strip() or line.startswith("#"):
        continue
    fields = line.split("\t")
    if len(fields) != 4 or not all(field.strip() for field in fields) or not fields[1].strip().isdigit():
        malformed.append(f"第 {line_number} 行：{line}")
        continue
    name, bit, format_paths, checker_paths = (field.strip() for field in fields)
    rows.append({
        "line": line_number,
        "name": name,
        "bit": int(bit),
        "format": [p.strip() for p in format_paths.split(",") if p.strip()],
        "checker": [p.strip() for p in checker_paths.split(",") if p.strip()],
    })

if malformed:
    print("  ✗ 布局清单里这些行不是四列、有空格子、或位号不是十进制数字：")  # gate-lint:summary
    for entry in malformed:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：四列用制表符分隔：布局名、incompat 位号、格式定义路径、checker 判定路径；后两列多条用逗号分隔，一条都不许空。")
    sys.exit(1)

if not rows:
    fail("布局清单一行内容都没有（只有注释）", [
        "怎么办：至少登记今天这一套布局——不登记，这一道扫到 0 套布局，与判过了在门禁输出里一模一样。",
    ])

# ② 路径存在
missing_paths = []
for row in rows:
    for path in row["format"] + row["checker"]:
        if not os.path.exists(path):
            missing_paths.append(f'{row["name"]}（第 {row["line"]} 行）：{path}')
if missing_paths:
    print(f"  ✗ 布局清单里 {len(missing_paths)} 条路径不存在：")  # gate-lint:summary
    for entry in missing_paths:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：路径搬了家就同步改这张表（.claude/rules/path-moves.md）；那一格真的没有对象了就把它删掉，")
    print("               别留着——指向不存在的路径会让人以为那一格有人管着。")
    sys.exit(1)

# ③ 位号与布局名要与 D15 已定项 4 的登记表对得上
if not os.path.isfile(registry_path):
    fail(f"找不到 {registry_path}", ["怎么办：决策文件改名或搬家了，同步改这个阶段里的路径。"])
registry_text = open(registry_path, encoding="utf-8").read()
registry = {}
for match in re.finditer(r"^\|\s*incompat\s*\|\s*(\d+)\s*\|\s*([^|]+?)\s*\|", registry_text, re.M):
    registry[int(match.group(1))] = match.group(2).strip()
unregistered = []
for row in rows:
    registered_name = registry.get(row["bit"])
    if registered_name is None:
        unregistered.append(f'{row["name"]}：incompat 位 {row["bit"]} 在已定项 4 登记表里没有行')
    elif registered_name != row["name"]:
        unregistered.append(f'incompat 位 {row["bit"]}：清单写「{row["name"]}」，登记表写「{registered_name}」')
if unregistered:
    print(f"  ✗ {len(unregistered)} 套布局的位号与 D15（格式冻结政策） 已定项 4 的登记表对不上：")  # gate-lint:summary
    for entry in unregistered:
        print(f"      {entry}")  # gate-lint:detail
    print(f"     → 怎么办：位号的唯一登记位是 {registry_path} 已定项 4 那张表（kb-discipline 第 4 条：同一个事实只许有一处权威记录）。")
    print("               开一条新布局线要先在那张表追加一位，再照它的含义列逐字写进布局清单；这张清单不许自己发明位号。")
    sys.exit(1)

# ④ 格式常量集合变了，checker 判定路径要跟
with open(changed_paths_file, encoding="utf-8") as changed_paths_handle:
    changed = {name for name in changed_paths_handle.read().split("\n") if name.strip()}

marker_problems = []

def constants_in(text, path, record_problems):
    """常量名 → 值的原文；record_problems 为真时把读不出来的标记与重复登记记进 marker_problems。"""
    found = {}
    if path.endswith(".rs"):
        for declaration in format_const.read_rust_consts(text, value_reading=RUST_VALUE_READING,
                                                         only_top_level_public=True):
            found[declaration.name] = declaration.value
        return found
    parsed = format_const.parse_marks(text)
    for mark in parsed.marks:
        found[mark.name] = mark.value_text
    if record_problems:
        for unparsable in parsed.unparsable:
            marker_problems.append(f"{path}:{unparsable.line_number}  标记按文法读不出来：「{unparsable.excerpt}」")
        for duplicate in parsed.duplicates:
            marker_problems.append(f"{path}  {duplicate.name} 在这一份里登记了 {len(duplicate.line_numbers)} 次"
                                   f"（第 {'、'.join(str(line_number) for line_number in duplicate.line_numbers)} 行）")
    return found

def baseline_text(path):
    result = subprocess.run([*GIT, "show", f"{base}:{path}"], capture_output=True, text=True)
    return result.stdout if result.returncode == 0 else ""

lag = {}
if os.path.isfile(lag_path):
    for line_number, line in enumerate(open(lag_path, encoding="utf-8"), 1):
        line = line.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 3 or not all(field.strip() for field in fields):
            fail(f"{lag_path} 第 {line_number} 行不是三列：{line}", [
                "怎么办：三列用制表符分隔：常量名、一条开着的欠账编号、为什么 checker 今天不判它。理由不许省。",
            ])
        lag[fields[0].strip()] = (fields[1].strip(), line_number)

failures = []   # 每项是一段要打印的拒绝：(摘要, 明细, 出路)

if lag:
    owed = owed_library.read_owed_table(owed_path)
    if not os.path.isfile(owed_path):
        failures.append((f"滞后登记表有 {len(lag)} 行，而找不到欠账表 {owed_path}", [], [
            "怎么办：欠账表挪了位置就同步改这个阶段里的路径；没有欠账表就核不了滞后登记指的账开没开着，",
            "          而一条指向空处的滞后登记，与 checker 真的跟上了在这一道的输出里一模一样。",
        ]))
    elif not owed.paid_heading_found:
        failures.append((f"滞后登记表有 {len(lag)} 行，而 {owed_path} 里认不出「### 已还清」那一行标题，分不出哪些账还开着", [], [
            "怎么办：欠账表按「### 已还清」整行标题切成开着与还清两段（.claude/gate.d/lib-owed.py）；标题改了名或丢了就改回来，",
            "          别让这一道对着一张认不出的表判——认不出时连历史版本节里的表格行都会被算成开着的账。",
        ]))
    else:
        dangling = [f"{name}：{number}（第 {line_number} 行）"
                    for name, (number, line_number) in sorted(lag.items())
                    if number not in owed.open_names]
        if dangling:
            failures.append((f"滞后登记表里 {len(dangling)} 行指的欠账编号不在 checks-owed.md 欠着那张表里：", dangling, [
                "怎么办：登记一条滞后，等于承认这个格式常量今天 checker 判不了，那笔账要有人排期。",
                f"          去 {owed_path} 立一条欠账（写清拦什么、怎么拦会红、缺什么前置），把它的编号写回这一行；",
                "          那笔账已经还清了就把这一行删掉——checker 跟上了就不该再登记滞后。",
            ]))

# 键是（格式定义路径, 常量名），不是光一个常量名：同一个常量在 kb 字段表与常量模块里各登记一次
# （门禁 27 号绑住这两处），按名字合并时后读到的那一份会把前一份盖掉，那一侧的改值就此看不见。
drifted, empty_sources, checked_constants, excused, followed = [], [], 0, [], []
for row in rows:
    current, baseline = {}, {}
    for path in row["format"]:
        found = constants_in(open(path, encoding="utf-8").read(), path, record_problems=True)
        if not found:
            empty_sources.append(f'{row["name"]}：{path}')
        for name, value in found.items():
            current[(path, name)] = value
        for name, value in constants_in(baseline_text(path), path, record_problems=False).items():
            baseline[(path, name)] = value
    checked_constants += len(current)
    changes = []
    for key in sorted(set(current) | set(baseline)):
        path, name = key
        if key not in baseline:
            changes.append((name, f'新增 {name} = {current[key]}（{path}）'))
        elif key not in current:
            changes.append((name, f'删掉 {name}（原值 {baseline[key]}，{path}）'))
        elif current[key] != baseline[key]:
            changes.append((name, f'改值 {name}：{baseline[key]} → {current[key]}（{path}）'))
    if not changes:
        continue
    if any(path in changed for path in row["checker"]):
        followed.extend(entry for _name, entry in changes)
        continue
    unexcused = [entry for name, entry in changes if name not in lag]
    excused.extend(entry for name, entry in changes if name in lag)
    if unexcused:
        drifted.append((row, unexcused))

if marker_problems:
    failures.append((f"格式定义里 {len(marker_problems)} 处 format-const 标记读不出来或重复登记，变更探测对它们不作数：", marker_problems, [
        "怎么办：读不出来的照 <!-- format-const: 名字 = 整数 stale=旧串|旧串 --> 改写，stale= 之外不许有别的键；",
        "          它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」；",
        "          重复的只留定这个值的那一处，别处要提它写成不带 <!-- 的文字（例如「`format-const: 名字`」）。",
    ]))

if drifted:
    total = sum(len(entries) for _, entries in drifted)
    details = []
    for row, entries in drifted:
        details.append(f'{row["name"]}（checker 判定路径：{", ".join(row["checker"])}）')
        details.extend(f"  {entry}" for entry in entries)
    failures.append((f"{total} 个格式常量在这次改动里变了，而它所属布局的 checker 判定路径一个都没被碰（基准 {base}）：", details, [
        "怎么办：两条出路。① 在同一次改动里改这套布局的 checker 判定路径，让它按新格式判——",
        "          格式走了一步而 checker 停在旧口径时，它会拿旧宽度去解新字节，而且全绿；",
        f"          ② checker 今天确实判不了它，就把常量名登记进 {lag_path}：",
        "          三列写常量名、一条开着的欠账编号、为什么今天不判。账在册才有人排期。",
    ]))

if failures:
    for summary, details, steps in failures:
        print(f"  ✗ {summary}")  # gate-lint:summary
        for detail in details:
            print(f"      {detail}")  # gate-lint:detail
        for step in steps:
            print(f"     → {step}" if step.startswith("怎么办") else f"     {step}")
    sys.exit(1)

paths_total = sum(len(row["format"]) + len(row["checker"]) for row in rows)
drifted_total = len(followed) + len(excused)
print(f"  ✓ 布局清单 {len(rows)} 套布局、{paths_total} 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；"
      f"这次改动比 {base}，{checked_constants} 个格式常量里变了 {drifted_total} 个"
      f"（checker 在同一次改动里跟了 {len(followed)} 个，按滞后表放行 {len(excused)} 个），都不欠 checker 跟进")
for entry in followed:
    print(f"      跟上了：{entry}")
if excused:
    print(f"    这 {len(excused)} 个变动是按 {lag_path} 放行的（checker 今天判不了它们，各挂着一条开着的欠账）：")
    for entry in excused:
        print(f"      {entry}")
if empty_sources:
    print(f"    没抽到常量的格式定义路径 {len(empty_sources)} 条（第 ④ 条对它们没有对象可判，只受第 ②③ 条管）：")
    for entry in empty_sources:
        print(f"      {entry}")
PY
