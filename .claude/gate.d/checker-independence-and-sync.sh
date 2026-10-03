#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: checker 类：checker 与实现只共享常量模块、记账语义不许同源，harness 档不依赖 checker 档；布局的格式常量变了，它的 checker 判定路径要在同一次改动里跟
# gate-category: checker 类
# gate-similar: code-source-discipline.sh 它的 format-constants 格也按 lib-format-const.py 读格式常量，但判的是 kb 字段表与常量模块两边的值逐个对得上；这里判格式常量变了 checker 跟没跟，不比两边的值；它的 mutation-tables 格按 lib-owed.py 读欠账表，判实验源码与变异表的纪律
# gate-similar: doc-registries.sh 它也读格式冻结政策的登记表与 kb 字段表，判的是 kb 各表之间、kb 与产物之间的账；这里判布局清单与 checker 源码；也按 lib-owed.py 读 checks-owed.md，判的是欠账表自己的形状与收口；这里只核滞后登记点名的欠账还开着
# gate-similar: checker-tier-crates-mutation-replay.sh 它复跑 crates/mutations.tsv 的变异，「运行时记账分支取反，I-3.1 必须红」那一半归它；这里只判两侧有没有共享代码
# gate-similar: checker-tier-layer0-replay.sh 开跑前调 crash-case-check.py 判 checker 档测试文件声明模块，对象是测试文件；这里判的是包之间的依赖方向
# gate-similar: harness-model-differential-and-scenarios.sh 同样盯 harness 档，但判的是用例粒度与模型对拍，不读 Cargo.toml
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就两格都跑，格名写错退 2）：
# gate-cell: checker-implementation-disjoint checker 与实现只共享常量模块，harness 档不依赖 checker 档
# gate-cell: layout-checker-sync 布局的格式常量变了，它的 checker 判定路径要在同一次改动里跟
# 参数解析、样本根的 .gate-cells、每格一个子 shell、汇总与退出码在共用库 lib/stage-cells.sh。
#
# ── checker-implementation-disjoint ──
# 依据：.claude/kb/decisions/13-验证路线.md 已定项 5（两侧只共享 kb 字段表生成的常量模块）与已定项 15（依赖方向）；⑤⑥⑦ 另依已定项 3（RefFS）
# 与用户 2026-10-02 定 F4「RefFS 单独一个包，按依赖图判」（research/prompts/m3-formal-r4-main-verification.md「用户定案」，
# 判决 research/prompts/m3-formal-r5-main-verification.md 第二节 U4 一行）。七条，任一条不成立判红：
#   ① 按 crates/*/Cargo.toml 建内部依赖图，singlefs-checker 与 singlefs-core 各取传递闭包求交，减去 singlefs-format 必须为空。
#      取闭包：checker 经中间包依赖实现时，直接依赖表上看不出。外部 crate 是叶子、照样参与求交；
#      罩不到外部 crate 之间的传递依赖（要真跑 cargo tree）。
#   ② checker 源码零处引 singlefs_core，经 `package =` 改名引进来的别名（`别名::`）同样算。
#   ③ crates/singlefs-format/src/**/*.rs 正文不许有 if / match / for / while / loop；`#[cfg(test)]` 只豁免紧跟它的那一个 item。
#   ④ singlefs-harness 的传递闭包（dev-dependencies 也算）里没有 singlefs-checker-tier，源码零处引 singlefs_checker_tier
#      （.claude/rules/verification.md「定义与名字」）。
#   ⑤ singlefs-reference-filesystem（RefFS）的传递闭包（dev-dependencies 也算）⊆ {它自己, singlefs-format}：外部 crate 也不许有。
#   ⑥ RefFS 的源码（包目录下全部 .rs，target/ 除外）零处引 singlefs_core、singlefs_checker、singlefs_harness、singlefs_checker_tier
#      （真名按标识符认，`singlefs_checker_tier` 不算一处 `singlefs_checker`）；经 `package =` 改名引进来的别名照 ② 的认法（`别名::`）同样算。
#   ⑦ singlefs-core 与 singlefs-checker 的传递闭包（dev-dependencies 也算）里没有 RefFS；包还不存在时照样按包名判。
#   RefFS 包目录 crates/singlefs-reference-filesystem 不存在时：crates/*/Cargo.toml 里另有一个包叫这个名字、或有包依赖这个名字，判红（搬家或改名了，
#      照下一段改写死的路径）；都没有，⑤ ⑥ 两条报本次未跑：输出里打一行 `!`，设了 $GATE_NOT_RUN_FILE 时往里追加一行，这一格照 ①–④ ⑦ 判绿或判红
#      （.claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」：只跑了一部分，跳过的逐项报）。包目录在而没有 Cargo.toml，
#      或一个 .rs 都没有，判红。
# 依赖表认 [dependencies] / [dev-dependencies] / [build-dependencies] / [target.'…'.dependencies]、[dependencies.X]、
# `X.workspace = true`（根 Cargo.toml [workspace.dependencies] 里的 package = 改名也认）、`别名 = { package = "真名", … }`；闭包按真名求交。
# 为什么：checker 重建记账、运行时增量维护记账，I-3.1（已分配统计对得上） 拿两者对账；两侧共用语义代码时写错会两边同样错，对账照样全绿。
# 射程：判不了两边各写的语义对不对（归模型对拍与变异表）；「运行时记账分支取反，I-3.1 必须红」在 crates/mutations.tsv，
# 归门禁 checker-tier-crates-mutation-replay；也判不了 RefFS 的语义对不对、它与实现是不是落在同一个崩溃语义的误解上（归 D13（验证路线） 已定项 6 的合谋测试）。
# 六个 crate 路径写死，改名或搬家时判红、不跳过（RefFS 那一个在包还不存在时照上一段）。
#
# ── layout-checker-sync ──
# 读 .claude/gate.d/layouts.tsv：一套布局一行，制表符分四列（布局名 / incompat 位号 / 格式定义路径 / checker 判定路径，后两列逗号分隔）。
# 四条，任一条不成立判红：
#   ① 四列齐全，位号是十进制数字，后两列至少各一条路径；
#   ② 表里每条路径都存在；
#   ③ 位号在 .claude/kb/decisions/15-格式冻结政策.md 已定项 4 登记表里有行，布局名与那一行的含义列逐字相同；
#   ④ 这次改动让某套布局格式定义里的「常量名 → 值」增、删或改值，而它的 checker 判定路径一条都没碰、常量名也不在
#      .claude/gate.d/layouts-checker-lag.tsv 里。豁免首次挂标记：名字新出现在一条格式定义路径，而同一套布局另一条格式定义路径
#      这次没变、值相同（去掉 `_` 分隔后按整数比）。
# 不在 git 仓里退 77。滞后表、标记与第 ④ 条都判完再退出，一次报全。
# 常量经 lib-format-const.py 从 .rs 顶格的 `pub const` 与 .md 的 `<!-- format-const: 名字 = 值 … -->` 标记抽；标记读不出或同一份里重复登记判红。
# .rs 的值按空白归一后的原文比、不要求整数字面量：常量模块里有 `DATA_UNIT_HEADER_BYTES + …` 这类算出来的常量。
# 变更探测器暂用「常量名 → 值」集合，不是 D15（格式冻结政策） 已定项 3 的 spec_hash：format-spec/<组件>.toml 还没有
# （C119（冻结组件没有 spec 文件）），有了就把抽取源换过去。
# 改动范围取 research/scripts/changed-paths.sh 的 gate_diff_base gate 与 gate_changed_paths（带未跟踪文件）；
# git 调用带 `-c core.quotepath=false`，否则中文路径被打成八进制引号串，与布局清单比不上。
# 滞后表指的欠账号按 lib-owed.py 读 checks-owed.md：「### 已还清」整行标题之前算开着，认不出那个标题判红。
#
# 样本（fixtures/checker-independence-and-sync.sh/，每份的 .gate-cells 点名它判的那一格）：
#   checker-implementation-disjoint-red    四条各犯一次（别名、[dependencies.X]、target 表、workspace 键；前部 item 贴 #[cfg(test)]；
#                                          harness 经 dev-dependencies 里的中间包依赖 checker 档包、源码引一处 singlefs_checker_tier）：红
#   checker-implementation-disjoint-green  干净（测试模块里的分支不算；checker 档包依赖 harness 方向对）：绿
#   layout-checker-sync-red                改了格式常量没碰 checker，另带多写键又重复登记的格式定义、挂在认不出的欠账表上的滞后表：红
#   layout-checker-sync-green              checker 跟着改（中文文件名）；滞后表的欠账号排在正文提到「### 已还清」的开着的账后面：绿
#   layout-checker-sync-first-registration-same-value          首次挂标记、值没变、checker 没碰：绿
#   layout-checker-sync-first-registration-underscored-literal .rs 字面量带 `_`、标记写不带的同一个数：绿
#   layout-checker-sync-first-registration-changed-value       源值这次也变了：红
#   checker-implementation-disjoint-red 里没有 RefFS 包：⑤ ⑥ 报本次未跑；-green 带 RefFS 包（dev-dependencies 里用别名引 singlefs-format，
#                                          测试引 singlefs_reference_filesystem 自己；harness 与 checker 档依赖它，方向对）：⑤⑥⑦ 绿
#   checker-implementation-disjoint-reference-closure-red          RefFS 依赖一个外部 crate，dev-dependencies 经中间包够到 singlefs-core：⑤ 红
#   checker-implementation-disjoint-reference-sources-red          RefFS 源码引 singlefs_checker 与 singlefs_checker_tier，测试用 dev-dependencies
#                                                                  里的别名 `oracle`（package = "singlefs-core"）：⑥ 红（⑤ 跟着红）
#   checker-implementation-disjoint-reference-dependents-red       singlefs-core 的 dev-dependencies 用别名依赖 RefFS，singlefs-checker 经中间包依赖它：
#                                                                  ⑦ 红（① 跟着红：两边闭包都有 RefFS）
#   checker-implementation-disjoint-reference-moved-red            RefFS 包搬到 crates/reference-filesystem、harness 照名字依赖它：判红
#   checker-implementation-disjoint-reference-without-sources-red  RefFS 包只有 Cargo.toml、一个 .rs 都没有：判红
# 弄坏开关：
#   STAGE_CELLS_BREAK=red-swallowed       共用库汇总吞红：两份 -red 与 changed-value 退 0
#   CHECKER_INDEPENDENCE_BREAK=<项>       每一项拆掉一条新判红分支，点名的样本判错：
#     reference-closure-unchecked          ⑤ 不判：reference-closure-red 退 0
#     reference-direct-names-unchecked     ⑥ 的真名不判：reference-sources-red 的「源码里有 2 处引」出不来
#     reference-aliases-unchecked          ⑥ 的别名不判：reference-sources-red 的「1 处用别名 `oracle::`」出不来
#     reference-dependents-unchecked       ⑦ 不判：reference-dependents-red 的「依赖闭包（dev-dependencies 也算）里有 RefFS 包」出不来
#     reference-moved-unchecked            包目录不在而包名还在时按本次未跑报：reference-moved-red 退 0
#     reference-sources-empty-accepted     一个 .rs 都没有也照判：reference-without-sources-red 退 0
#     reference-absent-unreported          包不在时不报本次未跑：checker-implementation-disjoint-red 的「⑤ ⑥ 两条本次未跑」出不来
#   REJECTION_BLOCKS_BREAK=summaries-only lib/rejection-blocks.py 只打摘要：layout-checker-sync-red 的「format/布局.md:5  标记按文法读不出来」明细出不来
#
#   bash .claude/gate.d/checker-independence-and-sync.sh [项目根]                          两格都跑
#   bash .claude/gate.d/checker-independence-and-sync.sh --list                            逐行打格名与判什么，不跑格
#   bash .claude/gate.d/checker-independence-and-sync.sh --check <格名>[,<格名>…] [项目根]   只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_NAME="$(basename "$0")"
source "$STAGE_DIR/lib/stage-cells.sh"

stage_cell checker-implementation-disjoint cell_checker_implementation_disjoint "checker 与实现只共享常量模块，harness 档不依赖 checker 档" \
  "照上面各条的出路改：checker 要用的逻辑在 checker 里自己写一份，只有格式常量从 singlefs_format 取；共享模块只发射标量；harness 档不依赖 checker 档（D13（验证路线） 已定项 5、15）；RefFS 只依赖 singlefs-format、不引那四个包，实现与池级 checker 不依赖它（用户 2026-10-02 定 F4）"
stage_cell layout-checker-sync cell_layout_checker_sync "布局的格式常量变了，它的 checker 判定路径要在同一次改动里跟" \
  "在同一次改动里改这套布局的 checker 判定路径；checker 今天确实判不了的，把常量名登记进 .claude/gate.d/layouts-checker-lag.tsv 并挂一条开着的欠账；布局清单与位号登记照上面那一条的出路改"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 bash .claude/gate.d/$STAGE_NAME --list 看。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$REPO_ROOT}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }

# ── checker-implementation-disjoint ──
cell_checker_implementation_disjoint() {
python3 - <<'PY'
import glob, os, re, sys

CHECKER = "crates/singlefs-checker"
CORE = "crates/singlefs-core"
SHARED = "crates/singlefs-format"
SHARED_CRATE_NAME = "singlefs-format"
HARNESS = "crates/singlefs-harness"
CHECKER_TIER = "crates/singlefs-checker-tier"
REFERENCE = "crates/singlefs-reference-filesystem"
REFERENCE_CRATE_NAME = "singlefs-reference-filesystem"
BREAK = os.environ.get("CHECKER_INDEPENDENCE_BREAK", "")

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
            f"怎么办：crate 改名或搬家了，同步改这个阶段里写死的六个路径（{CHECKER} / {CORE} / {SHARED} / {HARNESS} / {CHECKER_TIER} / {REFERENCE}）。",
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

internal, internal_aliases, crate_directories = {}, {}, {}
for manifest_path in sorted(glob.glob("crates/*/Cargo.toml")):
    name, dependencies, _aliases = manifest_of(os.path.dirname(manifest_path))
    internal[name] = dependencies
    internal_aliases[name] = _aliases
    crate_directories.setdefault(name, []).append(os.path.dirname(manifest_path))

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
    # `#[cfg(test)]` 只豁免紧跟的那一个 item（花括号配平到收尾，或一行以分号收尾）：豁免到文件尾，前部贴一个就藏住整份正文的分支。
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

# ④ harness 档不依赖 checker 档
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

# ⑤⑥⑦ RefFS 单独一个包：它只依赖 singlefs-format，不引实现、池级 checker、harness 档、checker 档，实现与池级 checker 不依赖它
REFERENCE_DECISION = "用户 2026-10-02 定 F4：RefFS 单独一个包，依赖只有 std 与 `singlefs-format`（research/prompts/m3-formal-r5-main-verification.md 第二节 U4 一行）"

def dependency_chain(start, target):
    """start 到 target 的一条依赖路径（按本仓内部依赖图走，最短的那条），每一跳写成别名的另注一句。"""
    parents, queue = {start: None}, [start]
    while queue and target not in parents:
        current = queue.pop(0)
        for dependency in sorted(internal.get(current, ())):
            if dependency not in parents:
                parents[dependency] = current
                queue.append(dependency)
    if target not in parents:
        return f"{start} → … → {target}"
    chain = [target]
    while parents[chain[-1]] is not None:
        chain.append(parents[chain[-1]])
    chain.reverse()
    notes = []
    for upper, lower in zip(chain, chain[1:]):
        via = [alias for alias, real in sorted(internal_aliases.get(upper, {}).items()) if real == lower]
        if via:
            notes.append(f"{upper} 的 Cargo.toml 里写成别名 `{'`、`'.join(via)}`")
    return " → ".join(chain) + (f"（{'；'.join(notes)}）" if notes else "")

checker_name, _, _ = manifest_of(CHECKER)
harness_name, _, _ = manifest_of(HARNESS)
reference_present = os.path.isdir(REFERENCE)
reference_name = manifest_of(REFERENCE)[0] if reference_present else REFERENCE_CRATE_NAME
reference_closure, reference_sources, reference_forbidden_aliases = set(), [], []
if reference_present:
    # ⑤ 闭包 ⊆ {自己, singlefs-format}
    reference_closure = closure_of(REFERENCE)
    outside = sorted(reference_closure - {reference_name, SHARED_CRATE_NAME})
    if outside and BREAK != "reference-closure-unchecked":
        problems.append((
            f"RefFS 包 `{reference_name}` 的依赖闭包（dev-dependencies 也算）里有 {len(outside)} 个不在 {{它自己, `{SHARED_CRATE_NAME}`}} 里：",
            [dependency_chain(reference_name, name) for name in outside],
            [f"怎么办：{REFERENCE_DECISION}。外部 crate 也不许有。",
             "        RefFS 要用的东西在它自己的包里写一份；它与实现共用一段代码，就会与实现一起错（D13（验证路线） 已定项 3）。",
             "        要拿实现或池级 checker 与 RefFS 对拍的用例写进 harness 档或 checker 档，不写进 RefFS 包的 dev-dependencies。"],
        ))
    # ⑥ 源码零处引那四个包，真名与别名都算
    reference_sources = sorted(path for path in glob.glob(f"{REFERENCE}/**/*.rs", recursive=True)
                               if "/target/" not in path[len(REFERENCE):])
    if not reference_sources and BREAK != "reference-sources-empty-accepted":
        problems.append((
            f"{REFERENCE} 下一个 .rs 都没有：",
            [f"{REFERENCE}/Cargo.toml 在，源码一份都扫不到"],
            ["怎么办：RefFS 搬了源码目录就同步改这个阶段里的路径；扫到 0 个文件而报绿，与判过了一模一样。"],
        ))
    forbidden_names = {core_name, checker_name, harness_name, checker_tier_name}
    forbidden_identifiers = sorted(name.replace("-", "_") for name in forbidden_names)
    direct_pattern = re.compile(r"(?<![\w])(" + "|".join(map(re.escape, forbidden_identifiers)) + r")(?![\w])")
    _, _, reference_aliases = manifest_of(REFERENCE)
    reference_forbidden_aliases = sorted(alias.replace("-", "_") for alias, real in reference_aliases.items()
                                         if real in forbidden_names and alias.replace("-", "_") not in forbidden_identifiers)
    alias_pattern = re.compile(r"(?<![\w])(" + "|".join(map(re.escape, reference_forbidden_aliases)) + r")\s*::") if reference_forbidden_aliases else None
    direct_references, alias_references_in_reference = [], []
    for path in reference_sources:
        for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
            code = line.split("//", 1)[0]
            if direct_pattern.search(code):
                direct_references.append(f"{path}:{line_number}：{line.strip()}")
            elif alias_pattern and alias_pattern.search(code):
                alias_references_in_reference.append(f"{path}:{line_number}：{line.strip()}")
    if direct_references and BREAK != "reference-direct-names-unchecked":
        problems.append((
            f"RefFS 包的源码里有 {len(direct_references)} 处引 `{'` / `'.join(forbidden_identifiers)}`：",
            direct_references,
            [f"怎么办：{REFERENCE_DECISION}。RefFS 是独立的参照实现，不与实现、池级 checker、harness 档、checker 档共用代码",
             "        （D13（验证路线） 已定项 3：RefFS 与实现不许共用崩溃点枚举的那段代码）。要用的逻辑在 RefFS 里自己写一份；格式常量从 `singlefs_format` 取。"],
        ))
    if alias_references_in_reference and BREAK != "reference-aliases-unchecked":
        renamed_to = sorted({reference_aliases[alias] for alias in reference_aliases if alias.replace("-", "_") in reference_forbidden_aliases})
        problems.append((
            f"RefFS 包的源码里有 {len(alias_references_in_reference)} 处用别名 `{'::`、`'.join(reference_forbidden_aliases)}::` 引那四个包"
            f"（RefFS 的 Cargo.toml 里 package = 指到 {'、'.join(renamed_to)}）：",
            alias_references_in_reference,
            ["怎么办：改名引进来照样是引，与直接写真名同一件事（照 ② 的认法）。",
             "        把这条依赖从 RefFS 的 Cargo.toml 里删掉，要用的逻辑在 RefFS 里自己写一份。"],
        ))
else:
    # 包目录不在：包名还在别处、或还有包依赖它，是搬家或改名了；都没有，⑤ ⑥ 本次未跑
    elsewhere = sorted(directory for directory in crate_directories.get(REFERENCE_CRATE_NAME, []))
    depended_on_by = sorted(name for name, dependencies in internal.items() if REFERENCE_CRATE_NAME in dependencies)
    if (elsewhere or depended_on_by) and BREAK != "reference-moved-unchecked":
        problems.append((
            f"{REFERENCE} 不在，而包 `{REFERENCE_CRATE_NAME}` 还在别处、或还有包依赖它：",
            [f"包放在 {directory}" for directory in elsewhere] + [f"{name} 依赖它" for name in depended_on_by],
            [f"怎么办：RefFS 搬家或改名了，同步改这个阶段里写死的路径（{REFERENCE}）与包名；",
             "        路径对不上 ⑤ ⑥ 就没有对象可判，所以判红而不是报本次未跑。"],
        ))
    elif BREAK != "reference-absent-unreported":
        print(f"  ! RefFS 包 {REFERENCE} 不在，crates/*/Cargo.toml 里也没有包叫 `{REFERENCE_CRATE_NAME}` 或依赖它：⑤ ⑥ 两条本次未跑（⑦ 照判）")
        not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
        if not_run_file:
            with open(not_run_file, "a", encoding="utf-8") as not_run_handle:
                not_run_handle.write(f"checker-independence-and-sync.sh 的 checker-implementation-disjoint 格 ⑤ ⑥ 两条本次未跑：RefFS 包 {REFERENCE} 还不存在\n")
# ⑦ 实现与池级 checker 的闭包里没有 RefFS（包不在时按包名判）
reference_dependents = [dependency_chain(name, reference_name)
                        for name, closure in ((core_name, core_dependencies - {core_name}), (checker_name, checker_dependencies))
                        if reference_name in closure]
if reference_dependents and BREAK != "reference-dependents-unchecked":
    problems.append((
        f"实现或池级 checker 的依赖闭包（dev-dependencies 也算）里有 RefFS 包 `{reference_name}`：",
        reference_dependents,
        [f"怎么办：{REFERENCE_DECISION}；实现与池级 checker 都不依赖它，否则三个 oracle 不再互相独立（D13（验证路线） 已定项 3、6）。",
         "        把这条依赖删掉；要拿 RefFS 对拍的用例写进 harness 档或 checker 档。"],
    ))
if reference_present:
    reference_summary = (f"RefFS 包 `{reference_name}` 的依赖闭包 {len(reference_closure)} 个都在 {{它自己, `{SHARED_CRATE_NAME}`}} 里，"
                         f"它的 {len(reference_sources)} 份源码零处引实现、池级 checker、harness 档、checker 档（别名引进来的 {len(reference_forbidden_aliases)} 个）")
else:
    reference_summary = f"RefFS 包 {REFERENCE} 不在，⑤ ⑥ 本次未跑"

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
      f"harness 档的依赖闭包 {len(harness_dependencies)} 个里没有 checker 档包，它的 {len(harness_sources)} 份源码零处引它；"
      f"{reference_summary}；实现闭包 {len(core_dependencies)} 个、池级 checker 闭包 {len(checker_dependencies)} 个里都没有 RefFS 包")
print(f"    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半"
      f"「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 checker-tier-crates-mutation-replay 复跑")
PY
}

# ── layout-checker-sync ──
cell_layout_checker_sync() {
  local LIBRARY_DIRECTORY LIB_CHANGED_PATHS base
  LIBRARY_DIRECTORY="$STAGE_DIR"
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，这一格本次无对象可判"; exit 77; }
  LIB_CHANGED_PATHS="$REPO_ROOT/research/scripts/changed-paths.sh"
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
rejection_blocks = load_library("rejection_blocks", "lib/rejection-blocks.py")
# code-source-discipline 读 .rs 的值要整数字面量（integer_literal）；两道统一与否还没定，统一时改这一个名字。
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

# ③ 位号与布局名对上 D15（格式冻结政策） 已定项 4 的登记表
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

NUMERIC_LITERAL_AFTER_UNDERSCORE_STRIP = re.compile(r"-?[0-9]+")

def numeric_value_or_none(value_text):
    """去掉数字分隔符 `_` 后当整数读；读不出整数就交回 None——这时只能按原文本比。
    .rs 里的整数字面量允许 `805_306_368` 这种写法，format-const 标记的文法不认下划线，
    只写得出 `805306368`；两边字面不同、数值相同，不按原文本比就会把「同一个值」误判成「变了」。"""
    stripped = value_text.strip().replace("_", "")
    return int(stripped) if NUMERIC_LITERAL_AFTER_UNDERSCORE_STRIP.fullmatch(stripped) else None

def same_underlying_value(value_text_a, value_text_b):
    numeric_a, numeric_b = numeric_value_or_none(value_text_a), numeric_value_or_none(value_text_b)
    if numeric_a is not None and numeric_b is not None:
        return numeric_a == numeric_b
    return value_text_a == value_text_b

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

# 键是（格式定义路径, 常量名）：同一个常量在 kb 字段表与常量模块里各登记一次，只按名字合并会让后读到的盖掉前一份的改值。
drifted, empty_sources, checked_constants, excused, followed, registered_without_change = [], [], 0, [], [], []
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

    def first_registration_of_unchanged_value(new_path, name, value):
        """一个 (格式定义路径, 常量名) 键这次才出现，检查同一套布局里别的格式定义路径这次
        是不是恰好没变、且原值的底层数值就等于这个新出现的值——是就说明这只是给已经存在的值
        第一次挂标记，不是这次改动把常量变成了这个值。「没变」按同一个格式定义路径自己的原文
        逐字比（baseline == current，不许用数值等价放宽——那样会把这个路径自己的改值放过）；
        跨路径比对新出现的值时用 same_underlying_value：.rs 字面量带下划线分隔、标记的文法不许带，
        字面不同但数值相同不能算「变了」。"""
        for other_path in row["format"]:
            if other_path == new_path:
                continue
            other_key = (other_path, name)
            if (other_key in baseline and other_key in current
                    and baseline[other_key] == current[other_key]
                    and same_underlying_value(baseline[other_key], value)):
                return True
        return False

    changes = []
    for key in sorted(set(current) | set(baseline)):
        path, name = key
        if key not in baseline:
            if first_registration_of_unchanged_value(path, name, current[key]):
                registered_without_change.append(
                    f'{name} = {current[key]}（{path}，同一套布局里别的格式定义路径这次没变、原值就是这个）')
                continue
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
    rejection_blocks.print_blocks(failures, continuation_indent="     ")
    sys.exit(1)

paths_total = sum(len(row["format"]) + len(row["checker"]) for row in rows)
drifted_total = len(followed) + len(excused)
print(f"  ✓ 布局清单 {len(rows)} 套布局、{paths_total} 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；"
      f"这次改动比 {base}，{checked_constants} 个格式常量里变了 {drifted_total} 个"
      f"（checker 在同一次改动里跟了 {len(followed)} 个，按滞后表放行 {len(excused)} 个），都不欠 checker 跟进"
      + (f"；另有 {len(registered_without_change)} 个是第一次登记标记，源里的值这次没变，不算这次的变动"
         if registered_without_change else ""))
for entry in followed:
    print(f"      跟上了：{entry}")
if excused:
    print(f"    这 {len(excused)} 个变动是按 {lag_path} 放行的（checker 今天判不了它们，各挂着一条开着的欠账）：")
    for entry in excused:
        print(f"      {entry}")
if registered_without_change:
    print(f"    这 {len(registered_without_change)} 个是首次登记，不是这次改动引入的变化：")
    for entry in registered_without_change:
        print(f"      首次登记：{entry}")
if empty_sources:
    print(f"    没抽到常量的格式定义路径 {len(empty_sources)} 条（第 ④ 条对它们没有对象可判，只受第 ②③ 条管）：")
    for entry in empty_sources:
        print(f"      {entry}")
PY
}

stage_cells_run "$ROOT"
