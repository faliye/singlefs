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
# 常量从两处抽：`.rs` 里的 `pub const 名字: 类型 = 值;`，`.md` 里的 `<!-- format-const: 名字 = 值 … -->` 标记。
#
# 改动范围与 56 号同一条：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，
# 都没有就与 HEAD 比；工作区、暂存区与未跟踪文件都算。
#
# 判别力：fixtures/92-layout-checker-sync.sh/red 是一个改了格式常量、没碰 checker 的小仓，必须判红；
# green 是同一处改动加上 checker 跟着改，必须判绿。
#
#   bash .claude/gate.d/92-layout-checker-sync.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
base="HEAD"
if [[ -n "${GATE_BASE:-}" ]] && git rev-parse --verify -q "${GATE_BASE}^{commit}" >/dev/null 2>&1; then
  base="$GATE_BASE"
elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1; then
  base="$(git merge-base HEAD '@{upstream}')"
fi
python3 - "$base" <<'PY'
import os, re, subprocess, sys

base = sys.argv[1]
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
changed = set()
for args in (["diff", "--name-only", base, "--"],
             ["diff", "--name-only", "--cached", "--"],
             ["ls-files", "--others", "--exclude-standard", "--"]):
    result = subprocess.run(["git", *args], capture_output=True, text=True)
    changed.update(name for name in result.stdout.split("\n") if name.strip())

RUST_CONST = re.compile(r"^pub const\s+(\w+)\s*:[^=]+=\s*(.+?);", re.M | re.S)
MARK_CONST = re.compile(r"<!--\s*format-const:\s*(\w+)\s*=\s*(-?\d+)")

def constants_in(text, path):
    found = {}
    if path.endswith(".rs"):
        for name, value in RUST_CONST.findall(text):
            found[name] = re.sub(r"\s+", " ", value).strip()
    else:
        for name, value in MARK_CONST.findall(text):
            found[name] = value
    return found

def baseline_text(path):
    result = subprocess.run(["git", "show", f"{base}:{path}"], capture_output=True, text=True)
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

if lag:
    if not os.path.isfile(owed_path):
        fail(f"滞后登记表有 {len(lag)} 行，而找不到欠账表 {owed_path}", [
            "怎么办：欠账表挪了位置就同步改这个阶段里的路径；没有欠账表就核不了滞后登记指的账开没开着，",
            "          而一条指向空处的滞后登记，与 checker 真的跟上了在这一道的输出里一模一样。",
        ])
    owed_text = open(owed_path, encoding="utf-8").read()
    head = owed_text.split("### 已还清")[0]
    owed_open = set(re.findall(r"^\|\s*(C\d+)\s*\|", head, re.M))
    dangling = [f"{name}：{number}（第 {line_number} 行）"
                for name, (number, line_number) in sorted(lag.items())
                if number not in owed_open]
    if dangling:
        print(f"  ✗ 滞后登记表里 {len(dangling)} 行指的欠账编号不在 checks-owed.md 欠着那张表里：")  # gate-lint:summary
        for entry in dangling:
            print(f"      {entry}")  # gate-lint:detail
        print(f"     → 怎么办：登记一条滞后，等于承认这个格式常量今天 checker 判不了，那笔账要有人排期。")
        print(f"               去 {owed_path} 立一条欠账（写清拦什么、怎么拦会红、缺什么前置），把它的编号写回这一行；")
        print("               那笔账已经还清了就把这一行删掉——checker 跟上了就不该再登记滞后。")
        sys.exit(1)

# 键是（格式定义路径, 常量名），不是光一个常量名：同一个常量在 kb 字段表与常量模块里各登记一次
# （门禁 27 号绑住这两处），按名字合并时后读到的那一份会把前一份盖掉，那一侧的改值就此看不见。
drifted, empty_sources, checked_constants, excused, followed = [], [], 0, [], []
for row in rows:
    current, baseline = {}, {}
    for path in row["format"]:
        found = constants_in(open(path, encoding="utf-8").read(), path)
        if not found:
            empty_sources.append(f'{row["name"]}：{path}')
        for name, value in found.items():
            current[(path, name)] = value
        for name, value in constants_in(baseline_text(path), path).items():
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

if drifted:
    total = sum(len(entries) for _, entries in drifted)
    print(f"  ✗ {total} 个格式常量在这次改动里变了，而它所属布局的 checker 判定路径一个都没被碰（基准 {base}）：")  # gate-lint:summary
    for row, entries in drifted:
        print(f'      {row["name"]}（checker 判定路径：{", ".join(row["checker"])}）')  # gate-lint:detail
        for entry in entries:
            print(f"        {entry}")  # gate-lint:detail
    print("     → 怎么办：两条出路。① 在同一次改动里改这套布局的 checker 判定路径，让它按新格式判——")
    print("               格式走了一步而 checker 停在旧口径时，它会拿旧宽度去解新字节，而且全绿；")
    print(f"               ② checker 今天确实判不了它，就把常量名登记进 {lag_path}：")
    print("               三列写常量名、一条开着的欠账编号、为什么今天不判。账在册才有人排期。")
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
