#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: checker 与实现只共享常量模块，记账语义不许同源；harness 档不依赖 checker 档
#
# 判据：D13（验证路线） 已定项 5 逐字「checker 与实现之间只共享一样东西：一份由 kb 的字段表
# 生成的常量模块，生成器从 kb 读、两边都只消费，任何人不许手改」，以及 D13（验证路线） 已定项 15
# 的依赖方向（checker 档包依赖 harness 与 checker，反过来不许）。四条，任一条不成立判红：
#   ① 按本仓 `crates/*/Cargo.toml` 建内部依赖图，`singlefs-checker` 与 `singlefs-core` 各取
#      传递闭包再求交集，减去 `singlefs-format` 之后必须为空——这是 C12（增量语义共用） 要的
#      「两侧调用集合交集为空（格式解析、常量除外）」在 crate 粒度上的形态。取闭包而不是只看直接依赖：
#      checker 依赖 X、X 依赖实现时，两侧照样共用代码，而直接依赖表上一处都看不出来；外部 crate
#      在图上是叶子，仍然进闭包、仍然参与求交。**罩不到的**：外部 crate 之间的传递依赖（本仓没有
#      它们的 `Cargo.toml`，要靠真跑 `cargo tree` 才看得到）；
#   ② checker 的源码里零处引 `singlefs_core`（不改 `Cargo.toml` 也引不进来，扫一遍便宜，
#      而漏了它这一道就只剩一份声明在守）；
#   ③ 共享的那个常量模块 `crates/singlefs-format/src/**/*.rs` 里，除去 `#[cfg(test)]` 标着的那一项
#      （它后面紧跟的那一个 item：花括号配平到收尾，或一行以分号收尾），正文不许有分支或循环
#      （`if` / `match` / `for` / `while` / `loop`）——它只许发射标量与纯算术。`#[cfg(test)]` 之后的别的 item 照扫。
#   ④ harness 档包 `singlefs-harness` 的传递闭包（三种依赖表都算，dev-dependencies 也算）里没有 checker 档包
#      `singlefs-checker-tier`，它的源码里也零处引 `singlefs_checker_tier`：harness 档随时跑，一旦依赖 checker 档，
#      改一行代码跑 harness 就把 checker 档一起编进来，两档又耦合回去（.claude/rules/verification.md「定义与名字」）。
#
# 依赖表怎么读（①②两条都靠它）：`[dependencies]`、`[dev-dependencies]`、`[build-dependencies]` 与
# `[target.'…'.dependencies]` 这几种节头都认；`[dependencies.X]` 这种按 crate 开的表认成 X；
# `X.workspace = true` 的键认成 X；`别名 = { package = "真名", … }` 认成真名——闭包按真名求交，
# 而 checker 源码里用那个别名（`别名::…`）引实现，与引 `singlefs_core` 同样判红。
# 根目录 `Cargo.toml` 的 `[workspace.dependencies]` 里给某个键写了 `package =` 改名的，`X.workspace = true` 按那里的真名认。
#      少了这一条，把记账语义搬进共享模块就能绕开 ①②，而那正是两侧同源最隐蔽的一条路。
#
# 为什么：checker 重建记账、运行时增量维护记账，I-3.1（已分配统计对得上） 拿两者对账。两侧只要共用同一段
# 语义代码，「加减语义本身写错」就会在两边同样地错，而 I-3.1（已分配统计对得上） 全绿——已核实 bcachefs
# 在这一格上被自己绊倒（`disk_accounting.h:208` 是运行时与 GC 重建共用的同一行，只差 `gc` 下标）。
# 今天这三条在仓里都成立，而**没有任何东西盯着它们**：给 checker 的 `Cargo.toml` 加一行依赖，
# 门禁一个字都不会说。
#
# ⚠️ 射程：判的是「两侧有没有共享代码」，判不了「两边各自手写的那份语义对不对」——
# 后者归模型对拍与变异表。C12（增量语义共用） 另一半（把运行时某条记账分支的符号取反，
# I-3.1（已分配统计对得上） 必须变红）不在这一道里，它住在 `crates/mutations.tsv`，由门禁 59 号复跑。
# ⚠️ 五个 crate 的名字写死在这里：改名或搬家时这一道找不到文件会判红，不会静默跳过。
#
# 判别力：fixtures/94-checker-implementation-disjoint.sh/red 一次造出三条的形状（依赖表另用别名、`[dependencies.X]`、
# `[target.'…'.dependencies]` 与 `X.workspace = true` 几种写法，常量模块在前部一个 item 上贴了 `#[cfg(test)]`），
# green 是干净的一份（测试模块里的分支不算）；red 里的 harness 在 dev-dependencies 里经一个中间包依赖 checker 档包、
# 源码里引一处 `singlefs_checker_tier`，green 里的 checker 档包依赖 harness（方向对的那一边不判红）。
#
#   bash .claude/gate.d/94-checker-implementation-disjoint.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
python3 - <<'PY'
import glob, os, re, sys

CHECKER = "crates/singlefs-checker"
CORE = "crates/singlefs-core"
SHARED = "crates/singlefs-format"
SHARED_CRATE_NAME = "singlefs-format"
HARNESS = "crates/singlefs-harness"
CHECKER_TIER = "crates/singlefs-checker-tier"

def fail(message, steps):
    print(f"  ✗ {message}")
    for step in steps:
        print(f"     → {step}")
    sys.exit(1)

DEPENDENCY_TABLE = re.compile(r"^\[(?:target\.(?:'[^']*'|\"[^\"]*\"|[^\]]+)\.)?(?:dev-|build-)?dependencies\]$")
DEPENDENCY_ITEM_TABLE = re.compile(r"^\[(?:target\.(?:'[^']*'|\"[^\"]*\"|[^\]]+)\.)?(?:dev-|build-)?dependencies\.([A-Za-z0-9_\-]+)\]$")
PACKAGE_FIELD = re.compile(r"\bpackage\s*=\s*\"([^\"]+)\"")

def workspace_renames():
    """根目录 Cargo.toml 的 [workspace.dependencies] 里写了 package = 的键 → 真名。"""
    renames, in_table = {}, False
    if not os.path.isfile("Cargo.toml"):
        return renames
    for line in open("Cargo.toml", encoding="utf-8"):
        stripped = line.split("#", 1)[0].strip()
        if stripped.startswith("["):
            in_table = stripped == "[workspace.dependencies]"
            continue
        if in_table and "=" in stripped:
            key, value = (part.strip() for part in stripped.split("=", 1))
            renamed = PACKAGE_FIELD.search(value)
            if renamed:
                renames[key.strip('"')] = renamed.group(1)
    return renames

WORKSPACE_RENAMES = workspace_renames()

def manifest_of(crate_dir):
    """(包名, 依赖的真名集合, {别名: 真名})。依赖表的几种写法见文件头。"""
    path = os.path.join(crate_dir, "Cargo.toml")
    if not os.path.isfile(path):
        fail(f"找不到 {path}", [
            f"怎么办：crate 改名或搬家了，同步改这个阶段里写死的五个路径（{CHECKER} / {CORE} / {SHARED} / {HARNESS} / {CHECKER_TIER}）。",
            "        这一道判的是 checker 与实现之间共享了什么，路径对不上就没有对象可判，所以它判红而不是跳过。",
        ])
    package_name, names, aliases = None, set(), {}
    in_dependencies, in_package, item_table = False, False, None
    def add(key, value):
        key = key.strip('"')
        real = None
        workspace_key = re.match(r"^([A-Za-z0-9_\-]+)\.workspace$", key)
        if workspace_key:
            key = workspace_key.group(1)
            real = WORKSPACE_RENAMES.get(key)
        elif "." in key:
            return   # `X.version = …` 之类的点号键：crate 名是点号前那一段
        renamed = PACKAGE_FIELD.search(value or "")
        if renamed:
            real = renamed.group(1)
        elif re.search(r"\bworkspace\s*=\s*true\b", value or ""):
            real = real or WORKSPACE_RENAMES.get(key)
        real = real or key
        names.add(real)
        if real != key:
            aliases[key] = real
    for line in open(path, encoding="utf-8"):
        stripped = line.strip()
        if stripped.startswith("["):
            in_package = stripped == "[package]"
            in_dependencies = bool(DEPENDENCY_TABLE.match(stripped))
            item = DEPENDENCY_ITEM_TABLE.match(stripped)
            item_table = item.group(1) if item else None
            if item_table:
                add(item_table, "")
            continue
        if stripped.startswith("#") or "=" not in stripped:
            continue
        key, value = (part.strip() for part in stripped.split("=", 1))
        if in_package and key == "name":
            package_name = value.strip('"')
        elif item_table and key == "package":
            real = value.strip('"')
            names.discard(item_table)
            names.add(real)
            if real != item_table:
                aliases[item_table] = real
        elif in_dependencies:
            dotted = re.match(r"^([A-Za-z0-9_\-]+)\.(?!workspace$)", key)
            add(dotted.group(1) if dotted else key, value)
    return package_name or os.path.basename(crate_dir), names, aliases

# 只看直接依赖不够：checker 依赖 X、X 依赖实现时两侧照样共用代码，而直接依赖表上一处都看不出来。
# 按本仓 crates/ 下的 Cargo.toml 建内部依赖图，两边各取传递闭包再求交集；
# 外部 crate 在图上是叶子（本仓没有它的 Cargo.toml），仍然进闭包、仍然参与求交。
internal = {}
for manifest_path in sorted(glob.glob("crates/*/Cargo.toml")):
    name, dependencies, _aliases = manifest_of(os.path.dirname(manifest_path))
    internal[name] = dependencies

def closure_of(crate_dir):
    name, _, _ = manifest_of(crate_dir)
    seen, queue = set(), [name]
    while queue:
        current = queue.pop()
        for dependency in internal.get(current, ()):
            if dependency not in seen:
                seen.add(dependency)
                queue.append(dependency)
    return seen

checker_dependencies = closure_of(CHECKER)
core_dependencies = closure_of(CORE)
core_name, _, _ = manifest_of(CORE)
_, _, checker_aliases = manifest_of(CHECKER)
core_dependencies.add(core_name)
problems = []
shared_beyond_constants = sorted((checker_dependencies & core_dependencies) - {SHARED_CRATE_NAME})
def shown_dependency(name):
    via = [alias for alias, real in sorted(checker_aliases.items()) if real == name]
    return name + (f"（checker 的 Cargo.toml 里写成别名 `{'`、`'.join(via)}`）" if via else "")
if shared_beyond_constants:
    problems.append((
        f"checker 与实现除常量模块之外还共享 {len(shared_beyond_constants)} 个依赖（按本仓内部依赖图算的传递闭包，不只直接依赖）：",
        [shown_dependency(name) for name in shared_beyond_constants],
        [f"怎么办：D13（验证路线） 已定项 5 只许共享 `{SHARED_CRATE_NAME}` 这一个常量模块。两侧共用同一段代码时，",
         "        「加减语义本身写错」会在两边同样地错，而 I-3.1（已分配统计对得上） 拿两者对账照样全绿。",
         "        checker 要用的东西自己写一份，或者把它做成只发射标量的常量模块。"],
    ))

checker_sources = sorted(glob.glob(f"{CHECKER}/src/**/*.rs", recursive=True))
if not checker_sources:
    fail(f"{CHECKER}/src 下一个 .rs 都没有", [
        "怎么办：checker 搬家了就同步改这个阶段里的路径；扫到 0 个文件而报绿，与判过了一模一样。",
    ])
core_references = []
for path in checker_sources:
    for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
        code = line.split("//", 1)[0]
        if "singlefs_core" in code:
            core_references.append(f"{path}:{line_number}：{line.strip()}")
if core_references:
    problems.append((
        f"checker 的源码里有 {len(core_references)} 处引实现侧的 `singlefs_core`：",
        core_references,
        ["怎么办：checker 是独立解析器，解析、校验、遍历各写一份（D13（验证路线） 已定项 5）。",
         "        要用的逻辑在 checker 里自己写一份；只有格式常量可以从 `singlefs_format` 取。"],
    ))
# 别名：Cargo.toml 里把实现侧的 crate 改名引进来（`implementation = { package = "singlefs-core", … }`），
# 源码写 `implementation::…`，上面按 `singlefs_core` 找的那一条一处都看不见。
core_aliases = sorted(alias.replace("-", "_") for alias, real in checker_aliases.items() if real == core_name)
alias_references = []
for alias in core_aliases:
    alias_pattern = re.compile(r"(?<![\w])" + re.escape(alias) + r"\s*::")
    for path in checker_sources:
        for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
            if alias_pattern.search(line.split("//", 1)[0]):
                alias_references.append(f"{path}:{line_number}：{line.strip()}")
if alias_references:
    problems.append((
        f"checker 的源码里有 {len(alias_references)} 处用别名 `{'`、`'.join(core_aliases)}::` 引实现侧（Cargo.toml 里 package = \"{core_name}\"）：",
        alias_references,
        ["怎么办：改名引进来照样是引实现，与直接写 `singlefs_core` 同一件事（D13（验证路线） 已定项 5）。",
         "        把这条依赖从 checker 的 Cargo.toml 里删掉，要用的逻辑在 checker 里自己写一份。"],
    ))

BRANCH = re.compile(r"(?<![\w!])(if|match|for|while|loop)(?![\w])")
shared_sources = sorted(glob.glob(f"{SHARED}/src/**/*.rs", recursive=True))
if not shared_sources:
    fail(f"{SHARED}/src 下一个 .rs 都没有", [
        "怎么办：常量模块搬家了就同步改这个阶段里的路径；扫到 0 个文件而报绿，与判过了一模一样。",
    ])
branches, scanned_lines, skipped_test_lines = [], 0, 0
for path in shared_sources:
    # `#[cfg(test)]` 只豁免它标着的那一个 item（花括号配平到收尾，或一行以分号收尾），不豁免整份文件剩下的部分：
    # 把它贴在文件前部任一个 item 上，后面整份正文的分支就都看不见了。
    skipping, depth, opened = False, 0, False
    for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
        code = line.split("//", 1)[0]
        if not skipping and code.strip().startswith("#[cfg(test)]"):
            skipping, depth, opened = True, 0, False
            code = code.strip()[len("#[cfg(test)]"):]
            if not code.strip():
                skipped_test_lines += 1
                continue
        if skipping:
            skipped_test_lines += 1
            depth += code.count("{") - code.count("}")
            opened = opened or "{" in code
            if (opened and depth <= 0) or (not opened and code.strip().endswith(";")):
                skipping = False
            continue
        scanned_lines += 1
        found = BRANCH.search(code)
        if found:
            branches.append(f"{path}:{line_number}：{line.strip()}")
if branches:
    problems.append((
        f"共享的常量模块正文里有 {len(branches)} 处分支或循环：",
        branches,
        [f"怎么办：`{SHARED_CRATE_NAME}` 是 checker 与实现唯一共享的东西，只许发射标量与纯算术",
         "        （D13（验证路线） 已定项 5「生成器只发射标量值」）。有了分支，两侧就在共用一段语义，",
         "        而前两条一个字都不会说——把那段逻辑搬回各自的 crate，两边各写一份。"],
    ))

# ④ 依赖方向：harness 档不依赖 checker 档（D13（验证路线） 已定项 15）。
harness_dependencies = closure_of(HARNESS)
checker_tier_name, _, _ = manifest_of(CHECKER_TIER)
_, _, harness_aliases = manifest_of(HARNESS)
if checker_tier_name in harness_dependencies:
    via = [alias for alias, real in sorted(harness_aliases.items()) if real == checker_tier_name]
    problems.append((
        f"harness 档包的依赖闭包里有 checker 档包 `{checker_tier_name}`（按本仓内部依赖图算的传递闭包，dev-dependencies 也算）：",
        [checker_tier_name + (f"（harness 的 Cargo.toml 里写成别名 `{'`、`'.join(via)}`）" if via else "")],
        ["怎么办：依赖方向只许 checker 档 → harness 与 checker（D13（验证路线） 已定项 15，.claude/rules/verification.md「定义与名字」）。",
         "        harness 要用的脚手架留在 harness 自己这边或拷一份进来（宁可多份，不要耦合）；",
         "        用得到 checker 档东西的用例本身就属于 checker 档，把它搬进 crates/singlefs-checker-tier/tests/。"],
    ))
harness_sources = sorted(glob.glob(f"{HARNESS}/src/**/*.rs", recursive=True) + glob.glob(f"{HARNESS}/tests/**/*.rs", recursive=True))
if not harness_sources:
    fail(f"{HARNESS} 的 src/ 与 tests/ 下一个 .rs 都没有", [
        "怎么办：harness 搬家了就同步改这个阶段里的路径；扫到 0 个文件而报绿，与判过了一模一样。",
    ])
tier_crate_identifiers = {"singlefs_checker_tier"} | {alias.replace("-", "_") for alias, real in harness_aliases.items() if real == checker_tier_name}
tier_pattern = re.compile(r"(?<![\w])(" + "|".join(sorted(map(re.escape, tier_crate_identifiers))) + r")(?![\w])")
tier_references = []
for path in harness_sources:
    for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
        if tier_pattern.search(line.split("//", 1)[0]):
            tier_references.append(f"{path}:{line_number}：{line.strip()}")
if tier_references:
    problems.append((
        f"harness 档的源码里有 {len(tier_references)} 处引 checker 档包：",
        tier_references,
        ["怎么办：harness 档不依赖 checker 档（D13（验证路线） 已定项 15）。要用的东西在 harness 里自己写一份，",
         "        或者把这条用例搬进 crates/singlefs-checker-tier/tests/（它要 checker 档的东西，它就是 checker 档的用例）。"],
    ))

if problems:
    for title, entries, steps in problems:
        print(f"  ✗ {title}")  # gate-lint:summary
        for entry in entries:
            print(f"      {entry}")  # gate-lint:detail
        for step in steps:
            print(f"     → {step}")
    sys.exit(1)

print(f"  ✓ checker 与实现只共享常量模块 `{SHARED_CRATE_NAME}`（传递闭包的交集减去它为空：checker 闭包 {len(checker_dependencies)} 个、实现闭包 {len(core_dependencies)} 个，内部依赖图 {len(internal)} 个 crate）；"
      f"checker 的 {len(checker_sources)} 份源码零处引 `singlefs_core`（别名引进来的 {len(core_aliases)} 个）；"
      f"共享模块 {len(shared_sources)} 份源码的正文 {scanned_lines} 行里没有分支与循环（`#[cfg(test)]` 标着的项 {skipped_test_lines} 行不扫）；"
      f"harness 档的依赖闭包 {len(harness_dependencies)} 个里没有 checker 档包，它的 {len(harness_sources)} 份源码零处引它")
print(f"    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半"
      f"「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑")
PY
