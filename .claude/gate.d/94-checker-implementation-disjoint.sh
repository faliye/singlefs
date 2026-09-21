#!/usr/bin/env bash
# gate-stage: checker 与实现只共享常量模块，记账语义不许同源
#
# 判据：D13（验证路线） 已定项 5 逐字「checker 与实现之间只共享一样东西：一份由 kb 的字段表
# 生成的常量模块，生成器从 kb 读、两边都只消费，任何人不许手改」。三条，任一条不成立判红：
#   ① 按本仓 `crates/*/Cargo.toml` 建内部依赖图，`singlefs-checker` 与 `singlefs-core` 各取
#      传递闭包再求交集，减去 `singlefs-format` 之后必须为空——这是 C12（增量语义共用） 要的
#      「两侧调用集合交集为空（格式解析、常量除外）」在 crate 粒度上的形态。取闭包而不是只看直接依赖：
#      checker 依赖 X、X 依赖实现时，两侧照样共用代码，而直接依赖表上一处都看不出来；外部 crate
#      在图上是叶子，仍然进闭包、仍然参与求交。**罩不到的**：外部 crate 之间的传递依赖（本仓没有
#      它们的 `Cargo.toml`，要靠真跑 `cargo tree` 才看得到）；
#   ② checker 的源码里零处引 `singlefs_core`（不改 `Cargo.toml` 也引不进来，扫一遍便宜，
#      而漏了它这一道就只剩一份声明在守）；
#   ③ 共享的那个常量模块 `crates/singlefs-format/src/**/*.rs` 里，`#[cfg(test)]` 之前的正文
#      不许有分支或循环（`if` / `match` / `for` / `while` / `loop`）——它只许发射标量与纯算术。
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
# ⚠️ 三个 crate 的名字写死在这里：改名或搬家时这一道找不到文件会判红，不会静默跳过。
#
# 判别力：fixtures/94-checker-implementation-disjoint.sh/red 一次造出三条的形状，green 是干净的一份。
#
#   bash .claude/gate.d/94-checker-implementation-disjoint.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
python3 - <<'PY'
import glob, os, re, sys

CHECKER = "crates/singlefs-checker"
CORE = "crates/singlefs-core"
SHARED = "crates/singlefs-format"
SHARED_CRATE_NAME = "singlefs-format"

def fail(message, steps):
    print(f"  ✗ {message}")
    for step in steps:
        print(f"     → {step}")
    sys.exit(1)

def manifest_of(crate_dir):
    path = os.path.join(crate_dir, "Cargo.toml")
    if not os.path.isfile(path):
        fail(f"找不到 {path}", [
            f"怎么办：crate 改名或搬家了，同步改这个阶段里写死的三个路径（{CHECKER} / {CORE} / {SHARED}）。",
            "        这一道判的是 checker 与实现之间共享了什么，路径对不上就没有对象可判，所以它判红而不是跳过。",
        ])
    package_name, names, in_dependencies, in_package = None, set(), False, False
    for line in open(path, encoding="utf-8"):
        stripped = line.strip()
        if stripped.startswith("["):
            in_dependencies = stripped in ("[dependencies]", "[dev-dependencies]")
            in_package = stripped == "[package]"
            continue
        if stripped.startswith("#") or "=" not in stripped:
            continue
        key, value = (part.strip() for part in stripped.split("=", 1))
        if in_package and key == "name":
            package_name = value.strip('"')
        elif in_dependencies:
            names.add(key)
    return package_name or os.path.basename(crate_dir), names

# 只看直接依赖不够：checker 依赖 X、X 依赖实现时两侧照样共用代码，而直接依赖表上一处都看不出来。
# 按本仓 crates/ 下的 Cargo.toml 建内部依赖图，两边各取传递闭包再求交集；
# 外部 crate 在图上是叶子（本仓没有它的 Cargo.toml），仍然进闭包、仍然参与求交。
internal = {}
for manifest_path in sorted(glob.glob("crates/*/Cargo.toml")):
    name, dependencies = manifest_of(os.path.dirname(manifest_path))
    internal[name] = dependencies

def closure_of(crate_dir):
    name, _ = manifest_of(crate_dir)
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
core_name, _ = manifest_of(CORE)
core_dependencies.add(core_name)
problems = []
shared_beyond_constants = sorted((checker_dependencies & core_dependencies) - {SHARED_CRATE_NAME})
if shared_beyond_constants:
    problems.append((
        f"checker 与实现除常量模块之外还共享 {len(shared_beyond_constants)} 个依赖（按本仓内部依赖图算的传递闭包，不只直接依赖）：",
        list(shared_beyond_constants),
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

BRANCH = re.compile(r"(?<![\w!])(if|match|for|while|loop)(?![\w])")
shared_sources = sorted(glob.glob(f"{SHARED}/src/**/*.rs", recursive=True))
if not shared_sources:
    fail(f"{SHARED}/src 下一个 .rs 都没有", [
        "怎么办：常量模块搬家了就同步改这个阶段里的路径；扫到 0 个文件而报绿，与判过了一模一样。",
    ])
branches, scanned_lines = [], 0
for path in shared_sources:
    for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
        if line.strip().startswith("#[cfg(test)]"):
            break   # 测试模块之后的分支不判：它不是被两边消费的那一部分
        scanned_lines += 1
        code = line.split("//", 1)[0]
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

if problems:
    for title, entries, steps in problems:
        print(f"  ✗ {title}")  # gate-lint:summary
        for entry in entries:
            print(f"      {entry}")  # gate-lint:detail
        for step in steps:
            print(f"     → {step}")
    sys.exit(1)

print(f"  ✓ checker 与实现只共享常量模块 `{SHARED_CRATE_NAME}`（传递闭包的交集减去它为空：checker 闭包 {len(checker_dependencies)} 个、实现闭包 {len(core_dependencies)} 个，内部依赖图 {len(internal)} 个 crate）；"
      f"checker 的 {len(checker_sources)} 份源码零处引 `singlefs_core`；共享模块 {len(shared_sources)} 份源码的正文 {scanned_lines} 行里没有分支与循环")
print(f"    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半"
      f"「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑")
PY
