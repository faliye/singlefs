#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: checker 类：checker 与实现只共享常量模块、记账语义不许同源，harness 档不依赖 checker 档；布局的格式常量变了，它的 checker 判定路径要在同一次改动里跟
# gate-category: checker 类
# gate-similar: code-source-discipline.sh 它的 format-constants 格也按 lib-format-const.py 读格式常量，但判的是 kb 字段表与常量模块两边的值逐个对得上；这里判格式常量变了 checker 跟没跟，不比两边的值；它的 mutation-tables 格按 lib-owed.py 读欠账表，判实验源码与变异表的纪律
# gate-similar: doc-registries.sh 它也读格式冻结政策的登记表与 kb 字段表，判的是 kb 各表之间、kb 与产物之间的账；这里判布局清单与 checker 源码；也按 lib-owed.py 读 checks-owed.md，判的是欠账表自己的形状与收口；这里只核滞后登记点名的欠账还开着
# gate-similar: checker-tier-crates-mutation-replay.sh 它复跑 crates/mutations.tsv 的变异，C12「运行时记账分支取反 ⇒ I-3.1 必须红」那一半归它；这里只判两侧有没有共享代码
# gate-similar: 54-layer0-replay.sh 开跑前调 crash-case-check.py 判 checker 档测试文件声明模块，对象是测试文件；这里判的是包之间的依赖方向
# gate-similar: harness-model-differential-and-scenarios.sh 同样盯 harness 档，但判的是用例粒度与模型对拍，不读 Cargo.toml
#
# 两格，判 crates 的依赖与 checker 跟不跟格式（原是两道：checker 与实现不相交、布局常量与 checker 同步，合成这一道）。
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就两格都跑，格名写错退 2）：
# gate-cell: checker-implementation-disjoint checker 与实现只共享常量模块，harness 档不依赖 checker 档
# gate-cell: layout-checker-sync 布局的格式常量变了，它的 checker 判定路径要在同一次改动里跟
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头），
# 这里只登记格、解析项目根。
#
# ── checker-implementation-disjoint：checker 与实现只共享常量模块，harness 档不依赖 checker 档 ──
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
#      而漏了它这一格就只剩一份声明在守）；
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
# I-3.1（已分配统计对得上） 必须变红）不在这一格里，它住在 `crates/mutations.tsv`，由 crates 变异复跑那一道判。
# ⚠️ 五个 crate 的名字写死在这里：改名或搬家时这一格找不到文件会判红，不会静默跳过。
#
# ── layout-checker-sync：布局的格式常量变了，它的 checker 判定路径要在同一次改动里跟 ──
# 判据：`.claude/gate.d/layouts.tsv` 一套布局一行，四列用制表符分隔（布局名 / incompat 位号 /
# 格式定义路径 / checker 判定路径，后两列逗号分隔）。四条，任一条不成立判红：
#   ① 每一行四列齐全，位号是十进制数字，后两列至少各一条路径；
#   ② 表里每条路径都存在——指向不存在的路径，等于那一格没人管，而表看着还是满的；
#   ③ 每个 incompat 位号在 `.claude/kb/decisions/15-格式冻结政策.md` 已定项 4 那张登记表里有行，
#      且布局名与那一行的含义列逐字相同（位号的唯一登记位在那张表，这里只引用，不另立第二处）；
#   ④ 这次改动让某套布局的格式定义里「常量名 → 值」的集合发生增、删或改值，
#      而它的 checker 判定路径一个都没被这次改动碰过 ⇒ 判红；但一个名字新出现在某条格式定义
#      路径时，若同一套布局的另一条格式定义路径里这次没变、值恰好等于新出现的这个值，只算头一次
#      给已经存在的值挂标记，不算这次改动引入的变化——常量没变，checker 不欠它，不判红。
# 不在 git 仓里：这一格退 77（没有「这次改动」可比）。
#
# 为什么：一套布局的格式改了而它的 checker 判定路径没跟，门禁照样全绿——checker 会拿旧口径
# 去判新字节，而「checker 没报错」被读成「镜像是好的」。C13（checker 判定失效） 拦的是判定逻辑失效，
# 这一格拦的是它最常见的成因：格式先走了一步。
# 首次登记不算改动：给一个值一直没变的常量第一次挂上标记，是把没写下来的事实补写下来，
# 不是格式走了一步；判红只会让人去登记一笔不存在的欠账（layouts-checker-lag.tsv）
# 或者去改一个本来就不用跟的 checker——两条出路都不对症。
#
# ⚠️ 射程：④ 判的是「格式变了没人跟」，判不了 checker 跟得对不对——后者要人看。
# ⚠️ 这里的变更探测器**不是** D15（格式冻结政策） 已定项 3 要的那个 `spec_hash`：那一个恒对
# `format-spec/<组件>.toml` 求，而那份 toml 今天一个都不存在（C119（冻结组件没有 spec 文件））。
# 在它出现之前，这一格拿「常量名 → 值的集合」当变更探测器——哈希本来就只是变更探测器，
# 而一次 diff 也是。spec toml 有了就把抽取源换过去，四条判据不变。
# 常量从两处抽：`.rs` 里顶格的 `pub const 名字: 类型 = 值;`，`.md` 里的 `<!-- format-const: 名字 = 值 … -->` 标记。
# 两种都按 `lib-format-const.py` 读（code-source-discipline.sh 的 format-constants 格与 doc-registries.sh 的 field-table-sums 那一格用的是同一份）：
# 标记按文法读不出来的、同一份格式定义里同一个名字登记了不止一次的，这一格判红——前者在变更探测里看不见，后者两个值里改了哪个说不清。
# `.rs` 的值按空白归一后的原文比（value_reading="normalized_text"），不像 code-source-discipline.sh 的 format-constants 格那样要求整数字面量：
# 格式常量模块里有 `DATA_UNIT_HEADER_BYTES + …` 这类算出来的常量，这一格只问它变没变。
#
# 改动范围（基准与路径集合）都取共用脚本 research/scripts/changed-paths.sh：gate_diff_base gate 与
# gate_changed_paths 带未跟踪文件，不在这里另算一份（工具层自检那一道的 change-range-single-source 格判）。git 调用一律带 `-c core.quotepath=false`：
# 默认的 quoting 把中文路径打成八进制引号串，与布局清单里的路径逐字比对不上，checker 明明跟了也判「没碰」。
#
# 滞后登记表指的欠账号开没开着，按 `lib-owed.py` 读 checks-owed.md（doc-registries.sh 与 code-source-discipline.sh 的 mutation-tables 格用的是同一份）：
# 「### 已还清」整行标题之前的是开着的；认不出那个标题就判红，不对着一张认不出的表判。
#
# 滞后表、标记与第 ④ 条三样都判完再退出，一次把问题说全。
#
# 样本（fixtures/checker-independence-and-sync.sh/，每份的 .gate-cells 点名它判的那一格）：
#   checker-implementation-disjoint-red    一次造出四条的形状（依赖表另用别名、`[dependencies.X]`、`[target.'…'.dependencies]` 与
#                                          `X.workspace = true` 几种写法，常量模块在前部一个 item 上贴了 `#[cfg(test)]`；harness 在
#                                          dev-dependencies 里经一个中间包依赖 checker 档包、源码里引一处 `singlefs_checker_tier`），必须判红
#   checker-implementation-disjoint-green  干净的一份（测试模块里的分支不算；checker 档包依赖 harness，方向对的那一边不判红）
#   layout-checker-sync-red                改了格式常量、没碰 checker 的小仓，另带一份多写了键又重复登记的格式定义、
#                                          一张挂在认不出的欠账表上的滞后表，必须判红
#   layout-checker-sync-green              同一处改动加上 checker 跟着改（checker 路径是中文文件名），另带一张滞后表，挂的欠账号
#                                          排在一行正文提到「### 已还清」的开着的账后面，必须判绿
#   layout-checker-sync-first-registration-same-value         给一个值一直没变的常量第一次挂标记、checker 没碰，必须判绿
#   layout-checker-sync-first-registration-underscored-literal 同上，.rs 字面量带下划线分隔、标记里写不带下划线的同一个数，必须判绿
#   layout-checker-sync-first-registration-changed-value      同一次改动里源值也变了、新标记记的是新值，必须判红——
#                                          首次登记那条豁免只认「另一处这次没变」，源值真变了不豁免
# 汇总与 --check 的弄坏开关归共用库（STAGE_CELLS_BREAK=red-swallowed 让两份 -red 样本与 changed-value 退 0）。
# layout-checker-sync 逐块打印拒绝经 lib/rejection-blocks.py（与 code-source-discipline 的 reading-discipline 格共用）：
# REJECTION_BLOCKS_BREAK=summaries-only 只打摘要，layout-checker-sync-red 里「format/布局.md:5  标记按文法读不出来」那几行明细出不来。
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
  "照上面各条的出路改：checker 要用的逻辑在 checker 里自己写一份，只有格式常量从 singlefs_format 取；共享模块只发射标量；harness 档不依赖 checker 档（D13（验证路线） 已定项 5、15）"
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
      f"「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 checker-tier-crates-mutation-replay 复跑")
PY
}

# ── layout-checker-sync ──
cell_layout_checker_sync() {
  local LIBRARY_DIRECTORY LIB_CHANGED_PATHS base
  LIBRARY_DIRECTORY="$STAGE_DIR"
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，这一格本次无对象可判"; exit 77; }
  # 基准取法与别的门禁用改动范围的几格同一份：research/scripts/changed-paths.sh 的 gate 取法
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
# 逐块打印拒绝与 code-source-discipline 的 reading-discipline 格共用这一份（lib/rejection-blocks.py）
rejection_blocks = load_library("rejection_blocks", "lib/rejection-blocks.py")
# code-source-discipline 读 .rs 的值要整数字面量（integer_literal）；这一道沿用只做空白归一的旧口径，
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

# 键是（格式定义路径, 常量名），不是光一个常量名：同一个常量在 kb 字段表与常量模块里各登记一次
# （门禁 code-source-discipline 绑住这两处），按名字合并时后读到的那一份会把前一份盖掉，那一侧的改值就此看不见。
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
