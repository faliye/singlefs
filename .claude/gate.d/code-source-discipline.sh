#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的 Rust 源码、测试文件名、kb 文本、登记表与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 代码类的源码纪律：名字不只由空泛词拼成、测试文件不按里程碑起名；kb 登记的格式常量、条文集合与 feature bit 位号与代码逐个对得上；实验二进制有成形的变异表且锚点唯一、有钉绝对值的断言、种子不折叠与读数不恒为字面量 0
# gate-category: 代码类
# gate-similar: doc-text.sh 也是全仓扫写法，但判的是登记过的禁用写法与改过名的术语的旧名，对象是全部文本文件；这里的 vague-names 与 test-file-names 两格判 .rs 里声明的名字由哪些词拼成、测试文件叫什么
# gate-similar: harness-model-differential-and-scenarios.sh 也调 research/scripts/crash-case-check.py，但调的是 one-scenario 那一样、判用例里面的循环，归 harness 类；这里的 test-file-names 格只调 file-names 那一样，判的是文件名
# gate-similar: .claude/singlefs-ai-sop/scripts/naming-lint.sh 是上游的命名纪律，判单字母与常见缩写、词表写死在上游；空泛词表是本项目定的，上游这一批不发版，先放项目本地
# gate-similar: doc-registries.sh 它判欠账表的形状、里程碑收口表收全开着的欠账号、总审核记录每行有去向，读 kb 与 records/；这里的 kb 对账三格拿登记表把决策分项正文里的名字与 Rust 枚举变体配对、把 format-const 标记的值与源码 const 比、把 feature bit 位号与代码常量比，reading-discipline 格按同一份 lib-owed.py 只判两张豁免表挂的欠账号还开着，对象是实验二进制源码；它判 kb 里字段表、字节布局表、段序列登记表与里程碑各步之间的对账，field-table-sums 那一格也经 lib-format-const.py 读标记，判的是字段表加和与结构总宽，不读代码；这里的 format-constants 格拿标记的值与 research/、crates/ 源码里的 const 比、旧值字面串不许留，不做算术；它的 declared-counts 那一格也按 lib-owed.py 读欠账表，只数得出欠着多少条；这里只拿开着的编号核豁免表
# gate-similar: checker-independence-and-sync.sh 它也经 lib-format-const.py 读标记、按 lib-owed.py 判布局滞后登记表点名的欠账号还开着，看的是这次改动里布局常量变了 checker 跟没跟；这里看此刻两边的值相等，对象是实验二进制源码与变异表
# gate-similar: checker-tier-crates-mutation-replay.sh 它按 crates/mutations.tsv 逐条改坏源码再跑点名的测试，要编译、跑几个钟头；这里的 mutation-tables 格经同一个模块 research/scripts/crates-mutation-rows.py 的 static-check 只做静态判，不跑变异
# gate-similar: doc-experiments.sh 它的 evidence-in-repo 格看这一轮改过的装置与变异表在 research/results/ 里有没有一份不比它旧的产物；这里判变异表本身成不成形、锚点对不对，与实验源码里的断言和读数，不看产物
#
# 八格原是三道阶段，2026-09-28 按用户定案（门禁按五类组织、名字按类起、去掉编号；可以合成的合成一份，不互相引用）合成这一道，
# 格的判法一个字没动，原来各写一份的 --check 解析与逐格汇总换成共用库：
#   原「空泛名与测试文件名」那一道        vague-names、test-file-names（更早是空泛名那一道与测试粒度那一道的「测试文件名不按里程碑起」那一样）
#   原「格式常量、条文枚举与 feature bit」那一道  format-constants、clause-enums、feature-bits（更早是格式常量、条文与枚举对子、feature bit 三道；clause-enums 的登记表随之改名为 .claude/gate.d/clause-enum-pairs.tsv）
#   原「实验源码与变异表纪律」那一道      mutation-tables、absolute-assertions、reading-discipline（更早是变异表、绝对值断言、实验源码纪律三道）
# 规则：.claude/rules/verification.md「门禁的结构」「门禁管哪一半」。
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就八格按下面的次序全跑，格名写错退 2）：
# gate-cell: vague-names 函数名与类型名不只由空泛词拼成
# gate-cell: test-file-names 测试文件名不按里程碑、步号起
# gate-cell: format-constants 格式常量在 kb 与实验源码之间同步
# gate-cell: clause-enums 条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上
# gate-cell: feature-bits feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里
# gate-cell: mutation-tables 每个实验二进制都有成形的变异表、锚点唯一，crates/mutations.tsv 的静态几样
# gate-cell: absolute-assertions 每个实验二进制都有钉绝对值的断言
# gate-cell: reading-discipline 实验源码的两条读数纪律：种子不许折叠，读数不许恒为字面量 0
#   vague-names          词表 .claude/naming-vague-words，还没改完的文件 .claude/naming-vague-exclude
#   test-file-names      跑 research/scripts/crash-case-check.py --only file-names
#   format-constants     调 lib-format-const.py
#   clause-enums         按 .claude/gate.d/clause-enum-pairs.tsv 配对
#   feature-bits         .claude/kb/feature-bits.md 与 D15（格式冻结政策） 已定项 4
#   mutation-tables      research/mutations/ 与 crates/mutations.tsv（后者经 research/scripts/crates-mutation-rows.py static-check；「变异行点名的测试跑得到」那几样在这一格里）
#   absolute-assertions  research/e7-index-bench/src/bin 下全部，别处 src/bin 下以 e<数字>_ 开头的
#   reading-discipline   C59 种子折叠、C60 恒为字面量 0 的读数，两张豁免表挂开着的欠账
# 每格的判据、射程、管不到的写在它自己那一段注释里。
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头），
# 这里只登记格、解析项目根。
#
# 样本：fixtures/code-source-discipline.sh/ 下每格一对 <格名>-red / <格名>-green，每份样本根放 .gate-cells 只点名那一格。
# 各格注释里说的 red / green 就是那一格自己的这一对；一对样本的现场是原来那一道的整份红 / 绿现场原样搬来的（格不读的文件留着，
# 样本判的输出与合并前一字不差），expect 只留那一格的 want，加上共用库报的那一格的判定与汇总。
# 汇总与 --check 的弄坏开关归共用库：STAGE_CELLS_BREAK=red-swallowed 让每一份 -red 样本整道退 0；
# STAGE_CELLS_BREAK=marker-ignored 让八格都跑、「跑了 1 格（样本根的 .gate-cells 点名）」那条 want 找不到。
# 格的判法各有弄坏开关，每一项都让某一格的样本判错：
#   GATE_VAGUE_NAMES_BREAK=trait-methods-judged           vague-names 把 trait 块里的方法也判：vague-names-green 判红
#   GATE_VAGUE_NAMES_BREAK=exclusions-ignored             vague-names 不认排除表：vague-names-red 里「已经一个空泛名都没有了」那一句报不出来
#   CRASH_CASE_CHECK_BREAK=milestone-names-unseen         test-file-names 认不出按里程碑起的名：test-file-names-red 里点名那份测试文件的一句报不出来
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants  不比源码 const 的值与 kb 的现行值：format-constants-red 里「const SAMPLE_HDR = 84」那几行出不来
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=clause-enums      表里的名字按子串认作正文列举的一项：clause-enums-red 里「表里的「乙」不是……」那一行出不来
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=feature-bits      不判同一个（类别，位号）登记两行：feature-bits-red 里「同一个位号登记了不止一行」出不来
#   REJECTION_BLOCKS_BREAK=summaries-only                  reading-discipline 逐块打印拒绝只打摘要（共用 lib/rejection-blocks.py）：reading-discipline-red 里「e2_sample.rs:5    seed | 1」那几行明细出不来
# mutation-tables、absolute-assertions、reading-discipline 三格合并之前就没有格内的弄坏开关，判别力只靠各自的样本；
# 原来各道外壳里的 aggregate、check-ignored、combine、check-option 与「一格不跑按判过记」几支随外壳一起换成了共用库的开关。
#
#   bash .claude/gate.d/code-source-discipline.sh [项目根]                                八格都跑
#   bash .claude/gate.d/code-source-discipline.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/code-source-discipline.sh --check <格名>[,<格名>…] [项目根]         只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_NAME="$(basename "$0")"
STAGE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
# 共用模块在这份阶段所在的仓里（样本目录里没有 research/scripts/）：cd 进被判的仓之前取
STAGE_REPOSITORY="$(cd "$STAGE_DIRECTORY/../.." && pwd)"
CRASH_CASE_CHECK="$STAGE_REPOSITORY/research/scripts/crash-case-check.py"
FORMAT_CONST_LIBRARY="$STAGE_DIRECTORY/lib-format-const.py"
OWED_LIBRARY="$STAGE_DIRECTORY/lib-owed.py"
source "$STAGE_DIRECTORY/lib/stage-cells.sh"
bad() { printf '  ✗ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

stage_cell vague-names cell_vague_names "函数名与类型名不只由空泛词拼成" \
  "按 .claude/singlefs-ai-sop/rules/code-discipline.md「名字：光看名字，就说得出它是什么、做什么」改名；一时改不完的文件登记进 .claude/naming-vague-exclude（一行 <路径>  # 理由），排除表里指向不存在的与已经改完的行删掉，排除只缩不涨"
stage_cell test-file-names cell_test_file_names "测试文件名不按里程碑、步号起" \
  "照 crash-case-check 那一句出路按内容改名（领域在前、场景在后）；判法写在 research/scripts/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「定义与名字」"
stage_cell format-constants cell_format_constants "格式常量在 kb 与实验源码之间同步" \
  "权威是 kb 里的 format-const 标记：改常量三处一起动（kb 标记与正文、实验源码的 const 与钉死它的单测、重跑实验并更新 research/results/ 的产物），旧值只许留在「## 历史版本」之后与 *-history.md 里"
stage_cell clause-enums cell_clause_enums "条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上" \
  "先判哪一边错了：代码加了成员就写回那条分项正文、再往 .claude/gate.d/clause-enum-pairs.tsv 补一行；条文才是对的就改代码；别只改表让两边看着一致，改分项正文照 .claude/rules/format-evolution.md 记进变更史"
stage_cell feature-bits cell_feature_bits "feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里" \
  "位号的唯一登记位是 D15（格式冻结政策） 已定项 4 那张表：两处不一致改 .claude/kb/feature-bits.md 的记账表；每张位图从 0 严格递增不跳号；代码新引用的位先在两处登记"
stage_cell mutation-tables cell_mutation_tables "每个实验二进制都有成形的变异表、锚点唯一，crates/mutations.tsv 的静态几样" \
  "缺表的写 research/mutations/<bin名>.tsv；锚点改到今天源码里逐字存在、只出现一次的那一段；crates/mutations.tsv 照上面各条的出路改（六段、只许 \\n 转义、点名的测试跑得到、改的文件在门禁 checker-tier-crates-mutation-replay 的拷贝范围里），改完单跑那几行证明点名的测试红"
stage_cell absolute-assertions cell_absolute_assertions "每个实验二进制都有钉绝对值的断言" \
  "给它加一条把被量的那个数钉死的断言，绝对值由独立算术给出、不许从代码里读回来；加完用 research/scripts/mutate.sh 证明它会红"
stage_cell reading-discipline cell_reading_discipline "实验源码的两条读数纪律：种子不许折叠，读数不许恒为字面量 0" \
  "种子先过一次乘法混淆再置位（seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1），恒为 0 的字段接上真读数；动不了的存量登记进 .claude/gate.d/experiment-seed-fold-lag.tsv / experiment-constant-reading-lag.tsv，挂 .claude/kb/checks-owed.md 里开着的 C59 / C60，只缩不涨"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      bad "认不出的选项 $1"
      howto "只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 bash .claude/gate.d/code-source-discipline.sh --list 看。"
      exit 2 ;;
    *)
      if [[ -n "$root_argument" ]]; then
        bad "给了两个项目根：$root_argument 与 $1"
        howto "参数只认一个项目根；不给项目根就取这个脚本往上两级。"
        exit 2
      fi
      root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || { bad "进不去项目根 $ROOT"; howto "第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
ROOT="$(pwd)"

# ── vague-names ──
# 判据：`git ls-files` 列出的 crates/**/*.rs 与 research/e7-index-bench/src/**/*.rs 里，每一处 `fn <名>` 与 `struct|enum|type|trait <名>`：
# 函数名按 `_` 切词、类型名按大写字母切词，切出来的每一个词（小写）都在 .claude/naming-vague-words 里，判违规，逐处列「文件:行 名字」。
# 不判的：`impl <trait> for <类型>` 块里的方法（名字是 trait 定的）、`main`、宏展开出来的名字（认不出）。
# 还没改完名的文件逐个登记在 .claude/naming-vague-exclude（一行 <路径>  # 理由），那份文件里的违规不判；排除项指向不存在的文件、
# 或那份文件里已经一处违规都没有了，判红（排除只缩不涨）。一份 .rs 都没扫到：这一格本次无对象可判。
# 管不到的：名字里夹了一个非空泛词、而那个词本身也什么都没说（run_the_cell_again）——词表只认整名全空泛，这一类靠 review。
cell_vague_names() {
python3 - <<'PY'
import os, re, subprocess, sys

WORDS_FILE = ".claude/naming-vague-words"
EXCLUDE_FILE = ".claude/naming-vague-exclude"
BROKEN = os.environ.get("GATE_VAGUE_NAMES_BREAK", "")

def registry(path):
    entries = []
    if not os.path.isfile(path):
        return None
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        body = line.split("#", 1)[0].strip()
        if body:
            entries.append((number, body, line.split("#", 1)[1].strip() if "#" in line else ""))
    return entries

words = registry(WORDS_FILE)
if words is None:
    print(f"  ✗ 读不到空泛词表 {WORDS_FILE}")
    print("     → 怎么办：从 git 恢复它；这一格没有词表就什么都判不了，不是通过。")
    sys.exit(1)
vague = {word.lower() for _number, word, _reason in words}
excluded = registry(EXCLUDE_FILE) or []

listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "crates", "research/e7-index-bench/src"],
                        capture_output=True)
paths = sorted(path for path in listed.stdout.decode("utf-8", "replace").split("\0") if path.endswith(".rs") and os.path.isfile(path))
if not paths:
    print("  ! 本次无对象可判：crates/ 与 research/e7-index-bench/src/ 下一份 .rs 都没列出来")
    sys.exit(77)

DECLARATION = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+|async\s+|unsafe\s+|extern\s+\"[^\"]*\"\s+)*(fn|struct|enum|type|trait)\s+([A-Za-z_][A-Za-z0-9_]*)")
TRAIT_IMPL = re.compile(r"^\s*impl\b.*\bfor\b")

def words_of(kind, name):
    if kind == "fn":
        return [part.lower() for part in name.split("_") if part]
    return [part.lower() for part in re.findall(r"[A-Z][a-z0-9]*|[a-z0-9]+", name)]

def violations_in(path):
    found = []
    depth = 0
    trait_impl_depths = []
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
        code = line.split("//", 1)[0]
        if TRAIT_IMPL.match(code) and BROKEN != "trait-methods-judged":
            trait_impl_depths.append(depth)
        match = DECLARATION.match(code)
        inside_trait_impl = bool(trait_impl_depths) and depth > trait_impl_depths[-1]
        if match and not (match.group(1) == "fn" and inside_trait_impl) and match.group(2) != "main":
            parts = words_of(match.group(1), match.group(2))
            if parts and all(part in vague for part in parts):
                found.append(f"{path}:{number} {match.group(1)} {match.group(2)}")
        depth += code.count("{") - code.count("}")
        while trait_impl_depths and depth <= trait_impl_depths[-1] and "}" in code:
            trait_impl_depths.pop()
    return found

excluded_paths = {entry for _number, entry, _reason in excluded}
problems, exclusion_problems = [], []
scanned = 0
for path in paths:
    scanned += 1
    found = violations_in(path)
    if path in excluded_paths and BROKEN != "exclusions-ignored":
        if not found:
            exclusion_problems.append(f"{EXCLUDE_FILE}：{path} 里已经一个空泛名都没有了，这一行删掉")
        continue
    problems += found
for number, entry, reason in excluded:
    if not os.path.isfile(entry):
        exclusion_problems.append(f"{EXCLUDE_FILE}:{number} 指向不存在的文件 {entry}")
    elif len(reason) < 4:
        exclusion_problems.append(f"{EXCLUDE_FILE}:{number} {entry} 没写为什么先不改（# 后面至少 4 个字）")

for problem in problems:
    print(f"  ✗ {problem}")  # gate-lint:detail
if problems:
    print(f"  ✗ {len(problems)} 个函数名或类型名只由空泛词拼成（每个词都在 {WORDS_FILE} 里）")  # gate-lint:summary
    print("     → 怎么办：按 .claude/singlefs-ai-sop/rules/code-discipline.md「名字：光看名字，就说得出它是什么、做什么」改名：说出跑的是什么、判的是哪条、取的是哪个量；")
    print(f"                一时改不完的文件登记进 {EXCLUDE_FILE}（一行 <路径>  # 理由），改完删那一行。")
for problem in exclusion_problems:
    print(f"  ✗ {problem}")  # gate-lint:detail
if exclusion_problems:
    print(f"  ✗ {EXCLUDE_FILE} 有 {len(exclusion_problems)} 行不起作用或没写理由")  # gate-lint:summary
    print("     → 怎么办：指向不存在的与已经改完的那几行删掉；留着的每一行写明为什么先不改。排除只缩不涨。")
if problems or exclusion_problems:
    sys.exit(1)
print(f"  ✓ 没有只由空泛词拼成的函数名与类型名（扫了 {scanned} 份 .rs，词表 {len(vague)} 个词，排除 {len(excluded)} 份还没改完的文件）")
PY
}

# ── test-file-names ──
# 判据：research/scripts/crash-case-check.py --only file-names <项目根>：crates/singlefs-harness/tests/*.rs 与 crates/singlefs-checker-tier/tests/*.rs
# 的文件名不带里程碑序数、步号、增补号、并行线号、欠账号（那份脚本的 MILESTONE_FILE_NAME），判法只在那一份里。
# 两个 tests/ 下一份 .rs 都没有：这一格本次无对象可判。
# 管不到的：按里程碑起、却不是那几种写法的名字（`milestone_three_*` 这类），靠 review；别的包的测试文件不扫。
cell_test_file_names() {
  local file_names_exit
  if python3 "$CRASH_CASE_CHECK" --only file-names "$ROOT"; then file_names_exit=0; else file_names_exit=$?; fi
  case "$file_names_exit" in
    0|77) return "$file_names_exit" ;;
  esac
  echo "  ✗ 测试文件名：crash-case-check --only file-names 退 $file_names_exit（逐处列在上面）"
  echo "     → 怎么办：照上面那一句出路改名；判法写在 research/scripts/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「定义与名字」。"
  return 1
}

# ── 格 format-constants：格式常量在 kb 与实验源码之间同步
#
# **实测出来的，不是想出来的**（2026-08-31）：D23（journal 的角色与格式）已定项 9 在
# 2026-08-30 把记录头从 84 抬到 86，kb 的两个下游数跟着改成 426 / 4010 并写明「产物不改」。
# 而 `research/e7-index-bench/src/bin/e43_extension_point_budget.rs` 里
# `const JOURNAL_HDR: u64 = 84`、单测 `assert_eq!(..., 428)`、产物 `hdr=84 room=428`
# **三处都停在旧值**，`grep -rn JOURNAL_HDR .claude/gate.d/ .claude/scripts/` 零命中
# ⇒ 没有任何东西把实验源码里的格式常量绑到 kb 的现行值上。
#
# 为什么这条静默：`checker-tier-research-build-and-replay.sh` 是**逐字节比对产物**，源码与产物一起停在旧值时它永远绿；
# `doc-registries.sh` 只看 kb 内部。两条都在跑，而这一类失同步从两条中间漏过去。
#
# 权威在 kb，形态是一行机器可读标记，紧挨着定这个值的那句话：
#
#     <!-- format-const: JOURNAL_HEADER_BYTES = 86 stale=hdr=84|room=428 -->
#
# `stale=` 列的是**旧值的字面串**（`|` 分隔，可省）。它们不许再出现在 kb 正文与
# 实验源码里（`research/results/` 的产物不扫，理由见第 3 段）——但**允许出现在「## 历史版本」之后与 *-history.md 里**，
# 那正是 `.claude/singlefs-ai-sop/rules/writing-discipline.md`「正文只写现状，历史进文末」
# 给旧值留的位置。
#
# ⚠️ **它管得了哪一半**：只检查**已经登记了标记**的常量。一个新加的、没登记标记的
# 格式常量仍然可以静默漂移——那一半靠人，落点见 kb/checks-owed.md。
#
# 标记与 Rust const 的解析在 `lib-format-const.py`，doc-registries、checker-independence-and-sync 用的是同一份，不另抄。
# 以 `<!-- format-const` 开头而按文法读不出来的（多写了一个键、值不是整数）判红，不跳过；
# 同一个名字登记两次判红，同一份文件里写两次也算——第二个值不许被静默丢掉。
# 源码一侧只认类型是单个标识符的 const（`u64`、`usize`），值要是整数字面量：
# 实验里同名的扫描表（`const NODE_BYTES: [u64; 2] = …`）不是那个格式常量。
#
# 样本：red 放一处源码落后于 kb 的常量、一处正文残留的旧值、一条多写了键的标记、一个同一份文件里登记两次的名字，
# 以及值跨行写、带 pub(crate) 的两处落后声明，必须判红；green 另放一张与格式常量同名的扫描表（数组类型），必须判绿。
cell_format_constants() {
  [[ -d .claude/kb ]] || { echo "  ! 找不到 kb，这一格跳过"; exit 77; }

  python3 - "$FORMAT_CONST_LIBRARY" <<'PY'
import glob, importlib.util, os, sys

library_spec = importlib.util.spec_from_file_location("format_const", sys.argv[1])
format_const = importlib.util.module_from_spec(library_spec)
library_spec.loader.exec_module(format_const)
break_value_comparison = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "format-constants"

def body_of(text):
    """正文 = 「## 历史版本」之前那一段。历史里留旧值是文档纪律要求的，不判红。"""
    i = text.find('\n## 历史版本')
    return text if i < 0 else text[:i]

# ⚠️ `*-history.md` **不是登记位**：变更史里会原样引用标记（记「本轮加了哪个标记」），
# 那是历史陈述，不是第二处权威记录。与旧值字面串的豁免同一条理由
# （`writing-discipline.md`「正文只写现状，历史进文末」）。
kb_all = sorted(glob.glob('.claude/kb/**/*.md', recursive=True))
kb_files = [f for f in kb_all if not f.endswith('-history.md') and '/decisions-history/' not in f]

# ---- 1. 收标记：读不出来的判红；同一个常量只许登记一处，同一份文件里写两次也算（kb-discipline 第 4 条）----
marks, registrations, bad = {}, {}, []
unparsable_count = 0
for f in kb_files:
    parsed = format_const.parse_marks(open(f, encoding='utf-8').read())
    for unparsable in parsed.unparsable:
        unparsable_count += 1
        bad.append(f'{f}:{unparsable.line_number}  format-const 标记按文法读不出来：「{unparsable.excerpt}」')
    for mark in parsed.marks:
        registrations.setdefault(mark.name, []).append(f'{f}:{mark.line_number}')
        marks.setdefault(mark.name, (f, mark.value, list(mark.stale_literals)))

duplicate_count = 0
for name, places in registrations.items():
    if len(places) > 1:
        duplicate_count += 1
        bad.append(f'{name}：登记了 {len(places)} 处（{" 与 ".join(places)}）—— 同一个事实只许一处权威记录')

if not marks and not bad:
    print('  ! kb 里一个 format-const 标记都没有，这一格**什么也没验**')
    print('     → 在定住格式常量的那句话旁边加 <!-- format-const: 名字 = 值 stale=旧字面串 -->')
    sys.exit(1)

# ---- 2. 源码里的 const 定义必须等于 kb 的现行值 ----
srcs = sorted(glob.glob('research/**/*.rs', recursive=True) + glob.glob('crates/**/*.rs', recursive=True))  # 2026-09-14 起格式常量模块住 crates/singlefs-format，同一套标记绑住它
seen_in_src = set()
for f in srcs:
    source_text = open(f, encoding='utf-8', errors='ignore').read()
    for declaration in format_const.read_rust_consts(source_text, only_scalar_types=True):
        if declaration.name not in marks:
            continue
        name = declaration.name
        seen_in_src.add(name)
        # 值要读到分号为止：只取开头那段数字时，`16384 * 2` 读成 16384 判通过（静默放行），
        # `16 * 1024` 与 `16_384` 读成 16 判红（2026-09-11 改名回扫时实测）。
        if declaration.value is None:
            shown = format_const.normalized_value_text(declaration.value_text)
            bad.append(f'{f}:{declaration.line_number}  const {name} 的值写成了「{shown}」，门禁读不出它等于几 → 写成整数字面量（可带 _ 分隔）')
            continue
        kbf, want, _ = marks[name]
        if break_value_comparison:
            continue   # 弄坏开关：不比源码 const 的值与 kb 的现行值
        if declaration.value != want:
            bad.append(f'{f}:{declaration.line_number}  const {name} = {declaration.value}，而 {kbf} 定的现行值是 {want}')

# ---- 3. 旧值的字面串不许留在正文与源码里 ----
# ⚠️ `research/results/` **不在扫描范围**，理由与 doc-lint 排除 `research/prompts/` 同一条
# （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」，登记在 `.claude/doc-lint-exclude`）：
# 产物是**那一轮的原始输出**，改它等于产物不再对应它的输入，证据链当场断掉。
# 「源码改了而产物没重跑」由 `checker-tier-research-build-and-replay.sh` 逐字节比对抓——改了源码它就会红，直到重跑。
scan = [(f, format_const.strip_marks(body_of(open(f, encoding='utf-8').read()))) for f in kb_files]
scan += [(f, open(f, encoding='utf-8', errors='ignore').read()) for f in srcs]

for name, (kbf, want, stale) in sorted(marks.items()):
    for s in stale:
        for f, text in scan:
            if s in text:
                ln = text[:text.index(s)].count('\n') + 1
                bad.append(f'{f}:{ln}  还留着 {name} 的旧值字面串「{s}」（现行值 {want}）')

if bad:
    print('  ✗ 格式常量在 kb 与实验源码之间对不上：')
    for b in bad[:20]:
        print(f'     {b}')
    if len(bad) > 20:
        print(f'     …… 另有 {len(bad)-20} 处')
    print('     → 权威是 kb 里的 format-const 标记。改常量要三处一起动：')
    print('       ① kb 标记与正文 ② 实验源码的 const 与钉死它的单测 ③ 重跑实验并更新 research/results/ 的产物')
    print('     → 旧值只许留在「## 历史版本」之后与 *-history.md 里。')
    if unparsable_count:
        print('     → 读不出来的标记照这一条文法改写：<!-- format-const: 名字 = 整数 stale=旧串|旧串 -->，')
        print('       stale= 之外不许有别的键；它读不出来时，这个常量在这一格眼里就没登记过。')
        print('       它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」。')
    if duplicate_count:
        print('     → 一个名字只留一处登记（定这个值的那一处）；别处要提它，写成不带 <!-- 的文字，例如「`format-const: 名字`」。')
    sys.exit(1)

only_kb = sorted(set(marks) - seen_in_src)
print(f'  ✓ 格式常量同步（{len(marks)} 个已登记，{len(seen_in_src)} 个在源码里被钉住）')
if only_kb:
    print(f'     ! 这些还没有任何实验源码用到，这一格对它们只验了唯一性：{", ".join(only_kb)}')
PY
}

# ── 格 clause-enums：条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上
#
# 2026-09-23 一天里查出四例「条文与实现说反话」，其中三例形状相同：条文逐个列了一个集合，
# 而代码里那个枚举比它多一个或少一个成员，没有任何东西会红。format-constants 那一格只管登记过的格式常量、
# checker-independence-and-sync 只判「格式变了 checker 没跟」，两处都够不着这一类。
#
# 这一格拿 .claude/gate.d/clause-enum-pairs.tsv 逐对判三样：
#   ① 枚举里每个变体都在表里有一行（代码加了成员而没登记 ⇒ 红）；
#   ② 表里每一行的变体在枚举里真的存在（代码删了成员而表没跟 ⇒ 红）；
#   ③ 表里每一行的「条文里对应的名字」是那条分项正文里用「 / 」列举的一项，逐字、不按子串认（条文没跟上 ⇒ 红）；
#   ④ 反过来，那条分项正文里那串「 / 」列举的每一项都在表里有一行（条文加了成员、枚举与表都没动 ⇒ 红）。
#      那串列举取正文里与表中名字重合最多的一串「甲 / 乙 / 丙」（项里不含括号、逗号、句号、分号与 `*`），
#      末项后面紧跟的「六种」这类计数词不算进项名。
#   表里一行登记都没有（只剩注释），本次无对象可判，退 77。
#
# ⚠️ 它判的是**名字对得上**，不判「这个成员的语义是不是条文说的那个」。后者是语义判断，靠 review。
#
# 样本：red 的枚举多一个没登记的变体 `Bing`、表里一行只写了列举项「乙种」的一截「乙」、条文多列了一项「丙种」，三处都要点名；
# green 的枚举、表与条文列举逐个对上。
cell_clause_enums() {
  local TABLE=".claude/gate.d/clause-enum-pairs.tsv"
  if [[ ! -f "$TABLE" ]]; then
    echo "  ⊘ 本次未跑：$TABLE 不在，今天无对象可判"
    exit 77
  fi

  local report
  if ! report="$(TABLE="$TABLE" python3 - 2>&1 <<'PY'
import os, re, glob

break_substring_match = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "clause-enums"

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

LIST_RUN = re.compile(r"[^/（）()，,。；;\n*]+(?:\s*/\s*[^/（）()，,。；;\n*]+)+")
COUNT_WORD = re.compile(r"[一二三四五六七八九十两0-9]+\s*种$")

def enumerated_items(section, names):
    """分项正文里与 names 重合最多的那一串「甲 / 乙 / 丙」列举：返回项的列表；一串都没有返回 None。"""
    best, best_overlap = None, 0
    for run in LIST_RUN.finditer(section):
        items = [COUNT_WORD.sub("", item.strip()).strip() for item in run.group(0).split("/")]
        overlap = len(set(items) & set(names))
        if overlap > best_overlap:
            best, best_overlap = items, overlap
    return best

def is_enumerated_item(chinese, items):
    if break_substring_match:
        return any(chinese in item for item in items)   # 弄坏开关：按子串认
    return chinese in items

if not rows:
    print("EMPTY", sep="\t")

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

# ③ 的逐字那一半与 ④：按（分项，枚举）成组，取正文里那串列举，两边集合逐项比。
groups = {}
for number, item, where, variant, chinese, _why in rows:
    if clauses.get(item, (None, "x"))[1] or enums.get(where, (None, "x"))[1]:
        continue
    groups.setdefault((item, where), []).append((number, chinese))
for (item, where), members in sorted(groups.items()):
    names = [chinese for _number, chinese in members]
    items = enumerated_items(clauses[item][0], names)
    if items is None:
        print("BAD", f"{item}", f"正文里找不到一串用「 / 」分隔、含表中名字的列举，{where} 那张表逐项对不了", sep="\t")
        continue
    for number, chinese in members:
        if chinese in clauses[item][0] and not is_enumerated_item(chinese, items):
            print("BAD", f"{os.environ['TABLE']}:{number}",
                  f"表里的「{chinese}」不是 {item} 正文那串列举里的一项（列举是：{' / '.join(items)}；按子串对上的不算）", sep="\t")
    for extra in items:
        if extra not in names:
            print("BAD", f"{item}",
                  f"条文列了「{extra}」，而 {os.environ['TABLE']} 里 {where} 没有它那一行（条文加了成员而枚举与表都没跟）", sep="\t")

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
    echo "    → 怎么办：按上面的报错修 .claude/gate.d/${STAGE_NAME} 里 clause-enums 那一格的 python；"
    echo "      这一步没跑完就是什么都没查，不是通过。"
    exit 1
  fi

  if ! grep -q '^COUNT' <<<"$report"; then
    echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（多半是它自己崩了）"
    echo "    → 怎么办：单独跑一遍这一格（--check clause-enums）看 python 的报错；没有 COUNT 就是一对都没比，"
    echo "      而汇总行看着与比过了一模一样。"
    exit 1
  fi

  local -a bad
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

  if grep -q '^EMPTY' <<<"$report"; then
    echo "  ⊘ 本次无对象可判：$TABLE 里一行登记都没有（只剩注释与空行）"
    exit 77
  fi
  local checked enum_count clause_count
  read -r _ checked enum_count clause_count < <(grep '^COUNT' <<<"$report")
  echo "  ✓ 条文列的集合与代码里的枚举逐个对得上（${checked} 对成员、${enum_count} 个枚举、${clause_count} 条分项；两个方向都逐项比过：表里的名字都是正文列举的一项，正文列举的每一项都在表里）"
  exit 0
}

# ── 格 feature-bits：feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里
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
# 不带 feature bit 常量名的裸字面量（判 incompat 时直接写 `0x01` 这一类，不经具名常量）不在射程里，靠 review。
#
# 样本：red 的记账表跳号、同一位登记两行不同语义、多出一位登记表里没有，
# 代码样本又引用了表里没有的一位，②③④ 与 ① 都必须报出来；green 只有位 0 一行、语义与登记表对得上，必须判绿。
cell_feature_bits() {
  [[ -f .claude/kb/feature-bits.md ]] || { echo "  ! 没有 .claude/kb/feature-bits.md，这一格无对象可判"; exit 77; }

  python3 - <<'PY'
import glob, os, re, sys

break_duplicate_check = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "feature-bits"
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
        "确认决策正文还在 .claude/kb/decisions/ 下；改了文件名就把 15- 这个编号前缀留着，这一格按它找。",
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
if duplicates and not break_duplicate_check:   # 弄坏开关：不判同一个（类别，位号）登记两行
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
}

# ── 格 mutation-tables：每个实验二进制都要有变异表
#
# 判据：`research/e7-index-bench/src/bin/` 下每个 `*.rs` 在 `research/mutations/`
# 下都要有**同名** `.tsv` 变异表，且表里至少一条成形的变异（三段制表符分隔）。
#
# 为什么：kb 里每一句「N 条变异全部被抓」都靠这些表复现（C40（变异表没存）——
# 结论留下了而产生结论的装置没留下，正是 2026-08-29 审计实测过的失败形态；
# 2026-09-03 把最后 19 张欠表补齐后，用这一格拦住它再欠回去）。
#
# `crates/mutations.tsv`（门禁 checker-tier-crates-mutation-replay 的表，六段）在这里静态判 checker-tier-crates-mutation-replay 派活之前判的每一样，
# 而 checker-tier-crates-mutation-replay 整道要跑几个钟头，改 `crates/` 的实现员不跑它。2026-09-18 第二波修复改掉 4 行锚点、另有 2 行更早就坏了，
# 实现员只核了自己追加的 8 行，到崩溃验证员跑 checker-tier-crates-mutation-replay 才整表退出（`records/2026-09-16-subagent拆分提案.md` 第三十二节）。
# 成形、锚点、测试名、拷贝范围、转义这几样的判法只有一份：research/scripts/crates-mutation-rows.py 的 row_problems，
# checker-tier-crates-mutation-replay 派活之前调它，这里经 static-check 子命令调它，不另抄（2026-09-28 之前两边各判一份，checker-tier-crates-mutation-replay 要六段一段都不空、这里放过空段，
# 替换文空着的删行变异在这里判绿、到 checker-tier-crates-mutation-replay 派活之前整道判红）：
#   成形：六段；替换文可以空（删掉原文，改坏时原文换成空串），其余五段一段都不许空；
#   锚点：原文里的 \n 还原成换行，在「文件」那一段指的源码里恰好命中一次，文件不在、读不了也算；
#   测试名：必须红的测试名是一个用例名（不带正则符号与空白）；
#   转义：原文与替换文里只许 \n 这一种反斜杠转义（别的按字面写进源码，那条变异编不过、等于没跑，C427）；
# 另判两样会让一行变异「跑了却没跑到点名的测试」的写法（checker-tier-crates-mutation-replay 不判，只在这里）：点名的测试标了 #[ignore] 而参数里没有 --include-ignored；
# `--` 之后的筛选词一个都筛不到点名的测试（libtest 比的是完整测试名：模块路径由点名的函数在哪个文件、套在哪几层内联 mod 里算出）；两行重复也只在这里判。
# 另判门禁 checker-tier-crates-mutation-replay 的拷贝范围两样（checker-tier-crates-mutation-replay 只把 stage-inputs.tsv 里它那一行登记的路径拷进每片的源码副本）：
#   ① crates/mutations.tsv 某一行改的文件落在拷贝范围之外：checker-tier-crates-mutation-replay 派活之前整道判红，一条都不跑（row_problems 那一份）；
#   ② crates/ 下 .rs 里以 "../ 起头、按所在 crate 的根解析之后逃出 crates/、又不在拷贝范围里的字符串字面量（#[path = …] 不算，注释里的不算）：
#      运行时读它的测试在 checker-tier-crates-mutation-replay 的副本里读不到，没改坏就红，checker-tier-crates-mutation-replay 记「基线不绿」（2026-09-28：litmus/ 没登记时，
#      crates/singlefs-harness/tests/publish_order_matches_litmus.rs 与另一份读 CARGO_MANIFEST_DIR/../../litmus 的测试就是这样）。
#   管不到的：路径拼在变量里、环境变量给的、format! 拼出来的、不以 "../ 起头而运行期再 join("..") 的，这几种要等 checker-tier-crates-mutation-replay 的基线在副本里现跑才红；
#   指到仓根本身的（"../../"）按不在范围里判（之后拼上的是哪一份判不出）。
#
# ⚠️ **这一格不跑变异**（跑一遍全部表要逐条重编译，量级是小时）。
# 「表今天还会不会红」由每轮改动实验代码时手跑 `research/scripts/mutate.sh` 证明，
# 复跑记录见各实验正文的「口径与复跑」——这一格只保证装置在、形状对，不冒充跑过。
#
# 样本：red 里 research 那侧有一份没有同名表的实验二进制（e2_naked）、一条锚点命中 0 次、一条替换文带 \& 的、一张不是 UTF-8 的表（setup.sh 现造）；
# crates/mutations.tsv 那侧有锚点腐化、转义写进源码、文件列是目录、点名耗时用例没带开关、两种筛选词对不上、文件在拷贝范围外、
# 测试名带符号、测试名空着（setup.sh 现写）各一行，另有一份逃出 crates/ 的相对路径字面量；
# green 是同形的一套都对得上的（含替换文空着的删行变异、读 litmus/ 的测试），每个实验二进制各有一张锚点唯一的表。
cell_mutation_tables() {
BIN_DIR=research/e7-index-bench/src/bin
MUT_DIR=research/mutations
[[ -d "$BIN_DIR" && -d "$MUT_DIR" ]] || { echo "  ! 找不到 $BIN_DIR 或 $MUT_DIR，这一格跳过"; exit 77; }

missing=(); malformed=()
for src in "$BIN_DIR"/*.rs; do
  stem="$(basename "$src" .rs)"
  tsv="$MUT_DIR/$stem.tsv"
  if [[ ! -f "$tsv" ]]; then missing+=("$tsv 不存在（被测的是 $src）"); continue; fi
  # 至少一条成形的变异行：非注释、非空、恰好三段
  ok_rows=$(awk -F'\t' '!/^#/ && NF==3 && $1!="" && $2!="" {n++} END{print n+0}' "$tsv")
  bad_rows=$(awk -F'\t' '!/^#/ && NF!=3 && $0!="" {n++} END{print n+0}' "$tsv")
  if [[ "$ok_rows" -eq 0 || "$bad_rows" -gt 0 ]]; then malformed+=("$tsv：成形 $ok_rows 条、坏 $bad_rows 行"); fi
done

# ── 锚点还对得上吗（C327（变异表的锚点腐化没有会红的检查））──
# 成形不等于替换得上：`mutate.sh` 要求每条「原文」在对应源码里**恰好命中一次**，
# 命中 0 次或多次就退出码 3、**后面的条目一条都不跑**，而那张表对上面那几项检查是绿的。
# 2026-09-14 现查有 4 条这样的锚点（e143 两条、e67 一条、e79 一条），而它们所在的实验页
# 都写着「N 条变异全抓」——那句话当时已经复跑不出来了。
# ⚠️ 这里仍然**不跑变异**，只做子串计数，代价是毫秒级。
anchor_report="$(BIN_DIR="$BIN_DIR" MUT_DIR="$MUT_DIR" python3 - <<'PY'
import os, glob, re
bin_dir = os.environ["BIN_DIR"]; mut_dir = os.environ["MUT_DIR"]
bad = []; stray = []; unreadable = []; checked = 0
def stray_escapes(segment_name, segment):
    """mutate.sh 与 checker-tier-crates-mutation-replay 只把 \\n 还原成换行，别的反斜杠按字面写进源码（C427）。"""
    return [(segment_name, match.group(0)) for match in re.finditer(r"\\(.)", segment) if match.group(1) != "n"]
for tsv in sorted(glob.glob(os.path.join(mut_dir, "*.tsv"))):
    stem = os.path.basename(tsv)[:-4]
    src_path = os.path.join(bin_dir, stem + ".rs")
    # 没有同名二进制的表（shell 探针的变异表）不在这一格的射程：它们的被测对象不是 .rs；成功行逐个列名
    if not os.path.exists(src_path):
        print("NO_BINARY", tsv, sep="\t")
        continue
    # 读不了（不是 UTF-8、是个目录、没权限）的一份判红、接着判下一份：崩在这里会让整段报告一行不剩
    try:
        src = open(src_path, encoding="utf-8").read()
        table_lines = open(tsv, encoding="utf-8").read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        unreadable.append((stem, f"{type(error).__name__}: {error}"))
        continue
    for lineno, line in enumerate(table_lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.rstrip("\n").split("\t")
        if len(parts) != 3:
            continue
        name, frm, to = parts
        checked += 1
        # 口径与 mutate.sh 一致：表里的 \n 先还原成换行，再数子串
        hits = src.count(frm.replace("\\n", "\n"))
        if hits != 1:
            bad.append((stem, name, lineno, hits))
        for segment_name, segment in (("原文", frm), ("替换文", to)):
            for found in stray_escapes(segment_name, segment):
                stray.append((stem, name, lineno) + found)
print("CHECKED", checked)
for b in bad:
    print("BAD", *b, sep="\t")
for s in stray:
    print("STRAY", *s, sep="\t")
for u in unreadable:
    print("UNREADABLE", *u, sep="\t")
PY
)"
anchor_exit_code=$?
# ── crates/mutations.tsv（六段：变异名、文件、原文、替换文、cargo test 参数、必须红的测试名）──
# checker-tier-crates-mutation-replay 派活之前判的那几样（成形、锚点、测试名、拷贝范围、转义）不在这里判，在下面 static-check 那一段调共用模块；
# 这一段只判 checker-tier-crates-mutation-replay 不判的三样（两行重复、#[ignore]、筛选词），行照共用模块的 read_table 读（成形的口径只有它一份）。
MUTATION_ROWS_MODULE="$STAGE_REPOSITORY/research/scripts/crates-mutation-rows.py"
crates_report=""
if [[ -f crates/mutations.tsv ]]; then
  crates_report="$(MUTATION_ROWS_MODULE="$MUTATION_ROWS_MODULE" python3 - <<'PY'
import importlib.util
import os
import re
import sys
sys.dont_write_bytecode = True
specification = importlib.util.spec_from_file_location("crates_mutation_rows", os.environ["MUTATION_ROWS_MODULE"])
mutation_rows = importlib.util.module_from_spec(specification)
specification.loader.exec_module(mutation_rows)
first_line_of_change = {}
first_line_of_name = {}
attributes_by_package = {}
def module_path_of_file(package, path):
    """crates/<包>/src 下一个 .rs 在 libtest 测试名里的模块前缀：src/lib.rs 与 src/bin/<名>.rs 是空，src/a.rs 与 src/a/mod.rs 是 a，src/a/b.rs 是 a::b；tests/ 下的是空。"""
    relative = os.path.relpath(path, os.path.join("crates", package))
    parts = relative.split(os.sep)
    if parts[0] != "src" or parts[1:2] == ["bin"] or parts[1:] == ["lib.rs"]:
        return []
    parts = parts[1:]
    parts[-1] = parts[-1][:-len(".rs")]
    if parts[-1] == "mod":
        parts = parts[:-1]
    return parts
def enclosing_inline_modules(text):
    """(偏移, 那一刻套着的内联 mod 名单) 的有序表：按 `mod 名 {` 与花括号配平；字符串与注释里的花括号不另判。"""
    events, stack, depth = [], [], 0
    for token in re.finditer(r"\bmod\s+(\w+)\s*\{|[{}]", text):
        if token.group(1):
            depth += 1
            stack.append((token.group(1), depth))
        elif token.group(0) == "{":
            depth += 1
        else:
            if stack and stack[-1][1] == depth:
                stack.pop()
            depth -= 1
        events.append((token.end(), [name for name, _ in stack]))
    return events
def test_attributes_of_package(package):
    """包 crates/<包> 的 tests/ 与 src/ 下每个 fn 名 → [(它上面那串属性与注释行, libtest 里的完整测试名)]；每个包只扫一次。"""
    if package not in attributes_by_package:
        table = {}
        for directory_name in ("tests", "src"):
            for root, _directories, files in os.walk(os.path.join("crates", package, directory_name)):
                for file_name in sorted(files):
                    if not file_name.endswith(".rs"):
                        continue
                    path = os.path.join(root, file_name)
                    try:
                        text = open(path, encoding="utf-8").read()
                    except (OSError, UnicodeDecodeError):
                        continue
                    prefix = module_path_of_file(package, path)
                    events = enclosing_inline_modules(text)
                    for found in re.finditer(r"((?:[ \t]*#\[[^\n]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(", text):
                        modules = []
                        for offset, stack in events:
                            if offset > found.start(2):
                                break
                            modules = stack
                        table.setdefault(found.group(2), []).append((found.group(1), "::".join(prefix + modules + [found.group(2)])))
        attributes_by_package[package] = table
    return attributes_by_package[package]
rows, _malformed, _memory, _timeout = mutation_rows.read_table("crates/mutations.tsv")
for row in rows:
    line_number, fields = row.line_number, row.fields
    name, path = row.name, row.path
    # 两行的（文件, 原文, 替换文, 点名的测试）逐字相同 ⇒ 表里少了一条真改法，而条数看着没少（2026-09-22 实测：
    # 第 24 行那批八条里第 253 与第 257 行逐字相同，② 格因此没有自己的守卫）。
    # 只按改法认会误拒：同一个改法让两条不同的测试变红是合法的（第 221 与 227 行：一条单测、一条 checker 的 I-8.7 判别力）。
    change = (path, fields[2], fields[3], fields[5])
    if change in first_line_of_change:
        print("CRATES_DUP", line_number, name, f"（文件, 原文, 替换文, 点名的测试）与第 {first_line_of_change[change]} 行逐字相同", sep="\t")
    else:
        first_line_of_change[change] = line_number
    # 变异名相同 ⇒ 按名字分格的报告把两条并成一格，其中一条的结论看不见。
    if name in first_line_of_name:
        print("CRATES_DUP", line_number, name, f"变异名与第 {first_line_of_name[name]} 行相同", sep="\t")
    else:
        first_line_of_name[name] = line_number
    # 点名的测试标了 #[ignore]（harness 耗时用例、checker 档的全量），cargo test 参数里却没有 --include-ignored / --ignored：
    # checker-tier-crates-mutation-replay 与 prove-red.sh 照这一行跑，点名的测试被 libtest 跳过，这条变异永远报「没红」，而它守的那一格其实没人看。
    arguments = fields[4].split()
    package = arguments[arguments.index("-p") + 1] if "-p" in arguments[:-1] else None
    test_name = fields[5].split("::")[-1].strip()
    # cargo test 参数里 `--` 之后的位置参数是 libtest 的筛选词：点名的测试一个都筛不到，那条测试根本不跑，变异永远报没红。
    libtest_arguments = arguments[arguments.index("--") + 1:] if "--" in arguments else []
    filters = [word for word in libtest_arguments if not word.startswith("-")]
    exact = "--exact" in libtest_arguments
    # libtest 拿筛选词去比完整测试名（模块路径 + 函数名）：完整名从点名的函数在哪个文件、套在哪几层内联 mod 里算出来；找不到那个函数的不判。
    candidates = [full_name for _attributes, full_name in test_attributes_of_package(package).get(test_name, [])] if package and test_name else []
    if candidates and filters and not any((full_name == word) if exact else (word in full_name) for word in filters for full_name in candidates):
        print("CRATES_FILTER", line_number, name, f"筛选词 {' '.join(filters)}{'（--exact）' if exact else ''} 筛不到点名的 {' 或 '.join(candidates)}", sep="\t")
    if package and test_name and not ({"--include-ignored", "--ignored"} & set(arguments)):
        found_tests = test_attributes_of_package(package).get(test_name, [])
        if found_tests and all("#[ignore" in attributes for attributes, _full_name in found_tests):
            print("CRATES_IGNORED", line_number, name, f"点名的 {test_name} 标了 #[ignore]，cargo test 参数「{fields[4]}」里没有 --include-ignored", sep="\t")
print("CRATES_CHECKED", len(rows))
PY
)"
  crates_exit_code=$?
fi
# ── checker-tier-crates-mutation-replay 派活之前判的那几样与拷贝范围 ②（这一格注释里「成形、锚点……」那一段）：共用模块的 static-check 逐处打
# MALFORMED / ANCHOR / NAME / ROW / STRAY / LITERAL，末尾 COPIED 与 CHECKED；退 0 一处都没有、退 1 有、别的退出码是它自己没判完（ERROR 行是原因）
scope_report=""; scope_exit_code=0
if [[ -f "$MUTATION_ROWS_MODULE" ]]; then
  scope_report="$(python3 "$MUTATION_ROWS_MODULE" static-check "$ROOT")" || scope_exit_code=$?
else
  scope_exit_code=127; scope_report=$'ERROR\t'"找不到 $MUTATION_ROWS_MODULE"
fi
scope_copied="$(sed -n 's/^COPIED\t//p' <<<"$scope_report")"
scope_checked="$(sed -n 's/^CHECKED\t//p' <<<"$scope_report")"
mapfile -t crates_malformed < <(grep $'^MALFORMED\t' <<<"$scope_report")
mapfile -t crates_bad < <(grep $'^ANCHOR\t' <<<"$scope_report")
mapfile -t crates_names < <(grep $'^NAME\t' <<<"$scope_report")
mapfile -t scope_rows < <(grep $'^ROW\t' <<<"$scope_report")
mapfile -t crates_stray < <(grep $'^STRAY\t' <<<"$scope_report")
mapfile -t scope_literals < <(grep $'^LITERAL\t' <<<"$scope_report")
mapfile -t scope_errors < <(sed -n 's/^ERROR\t//p' <<<"$scope_report")
IFS=$'\t' read -r scope_row_count scope_source_count scope_literal_count <<<"$scope_checked"
crates_checked="$(sed -n 's/^CRATES_CHECKED //p' <<<"$crates_report")"
mapfile -t crates_dup < <(grep '^CRATES_DUP' <<<"$crates_report")
mapfile -t crates_ignored < <(grep '^CRATES_IGNORED' <<<"$crates_report")
mapfile -t crates_filter < <(grep '^CRATES_FILTER' <<<"$crates_report")
anchor_checked="$(sed -n 's/^CHECKED //p' <<<"$anchor_report")"
mapfile -t anchor_bad < <(grep '^BAD' <<<"$anchor_report")
mapfile -t stray_rows < <(grep '^STRAY' <<<"$anchor_report")
mapfile -t anchor_unreadable < <(grep '^UNREADABLE' <<<"$anchor_report")
mapfile -t tables_without_binary < <(sed -n 's/^NO_BINARY\t//p' <<<"$anchor_report")
# 两段 python 自己崩了（退出码非 0、或没报出 CHECKED 那一行）：它们没判完，不许读成「没有 BAD 行」
script_failures=()
if [[ "${anchor_exit_code:-1}" -ne 0 || -z "$anchor_checked" ]]; then
  script_failures+=("research 变异表的锚点检查没跑完（python 退出码 ${anchor_exit_code:-?}，CHECKED 行「${anchor_checked}」）")
fi
if [[ -f crates/mutations.tsv ]] && [[ "${crates_exit_code:-1}" -ne 0 || -z "$crates_checked" ]]; then
  script_failures+=("crates/mutations.tsv 的重复、#[ignore]、筛选词检查没跑完（python 退出码 ${crates_exit_code:-?}，CRATES_CHECKED 行「${crates_checked}」）")
fi
if [[ "$scope_exit_code" -ne 0 && "$scope_exit_code" -ne 1 ]] || [[ -z "$scope_checked" ]]; then
  script_failures+=("门禁 checker-tier-crates-mutation-replay 派活之前那几样与拷贝范围的检查没跑完（research/scripts/crates-mutation-rows.py static-check 退出码 ${scope_exit_code}，CHECKED 行「${scope_checked}」；${scope_errors[*]:-没报原因}）")
fi

if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]} + ${#crates_bad[@]} + ${#crates_dup[@]} + ${#crates_ignored[@]} + ${#crates_filter[@]} + ${#stray_rows[@]} + ${#crates_stray[@]} + ${#crates_malformed[@]} + ${#crates_names[@]} + ${#anchor_unreadable[@]} + ${#script_failures[@]} + ${#scope_rows[@]} + ${#scope_literals[@]})); then
  if ((${#script_failures[@]})); then
    echo "  ✗ 锚点或拷贝范围检查的 python 没跑完，这一段等于没判："
    printf '      %s\n' "${script_failures[@]}"
    echo "    → 怎么办：看上面 python 打出的报错（多半是某份文件读不了），修好再跑；没跑完不许当成锚点都对得上。"
  fi
  if ((${#anchor_unreadable[@]})); then
    echo "  ✗ 这些变异表或它的被测源码读不了（不是 UTF-8、是目录、没权限），表里的条目一条都没判："
    while IFS=$'\t' read -r _ stem why; do
      printf '      %s 或 %s：%s\n' "$MUT_DIR/$stem.tsv" "$BIN_DIR/$stem.rs" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${anchor_unreadable[@]}")
    echo "    → 怎么办：转成 UTF-8 文本（或把放错的目录挪走），再跑；mutate.sh 读它时同样会失败。"
  fi
  if ((${#missing[@]})); then
    echo "  ✗ 这些实验二进制没有同名变异表："   # gate-lint:detail
    printf '      %s\n' "${missing[@]}"
  fi
  if ((${#malformed[@]})); then
    echo "  ✗ 这些变异表不成形（要求每行三段制表符分隔，至少一条）："
    printf '      %s\n' "${malformed[@]}"
    echo "    → 每行写成「变异名<TAB>原文<TAB>替换文」三段，坏行补齐或删掉，至少留一条成形的。"
  fi
  if ((${#anchor_bad[@]})); then
    echo "  ✗ 这些变异条目的「原文」在源码里不是恰好命中一次（mutate.sh 会退出码 3，后面的条目一条都不跑）："
    while IFS=$'\t' read -r _ stem name lineno hits; do
      printf '      %s:%s %s：原文在 %s 里命中 %s 次\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$BIN_DIR/$stem.rs" "$hits"   # gate-lint:detail
    done < <(printf '%s\n' "${anchor_bad[@]}")
    echo "    命中 0 次：源码改过而表没跟；命中多次：原文要多带一行上下文才唯一。"
  fi
  if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]})); then
    echo "  → 怎么办：缺表的写 research/mutations/<bin名>.tsv（每行：变异名<TAB>原文<TAB>替换文）；"
    echo "    锚点对不上的把「原文」改成今天源码里逐字存在、且只出现一次的那一段，"
    echo "    改完跑 bash research/scripts/mutate.sh <bin> <源文件> <表> 证明每条都被抓，再来。"
  fi
  if ((${#stray_rows[@]})); then
    echo "  ✗ 这些变异条目的原文或替换文里有 \\n 以外的反斜杠转义（mutate.sh 只还原 \\n，别的按字面写进源码）："
    while IFS=$'\t' read -r _ stem name lineno segment escape; do
      printf '      %s:%s %s：%s里的 %s\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$segment" "$escape"   # gate-lint:detail
    done < <(printf '%s\n' "${stray_rows[@]}")
    echo "    → 怎么办：去掉那个反斜杠。替换文按字面写进源码，\\& \\\" 这类会原样落进去、那份源码编不过，"
    echo "      变异既不算被抓也不算没红（.claude/rules/mutation-sampling.md 第八类：无效变异），而跑变异的脚本报的是「点名的测试没跑到」。"
  fi
  if ((${#crates_stray[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的原文或替换文里有 \\n 以外的反斜杠转义（门禁 checker-tier-crates-mutation-replay 只还原 \\n）："
    while IFS=$'\t' read -r _ lineno name segment escape; do
      printf '      crates/mutations.tsv:%s %s：%s里的 %s\n' "$lineno" "$name" "$segment" "$escape"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_stray[@]}")
    echo "    → 怎么办：去掉那个反斜杠，改完单跑那一行证明点名的测试红（2026-09-20 第 180 行踩过：替换文写 \\&report.outcome，"
    echo "      写进源码是字面反斜杠、编不过，checker-tier-crates-mutation-replay 把它报成「点名的测试没跑到」，出路指向「先造一条会红的用例」，方向是反的）。"
  fi
  if ((${#crates_dup[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行与前面某一行重复（表里的条数没少，真改法少了一条）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_dup[@]}")
    echo "    → 怎么办：这一行本来要守的那一格今天没有守卫。把它换成真正打在那一格上的改法（改完单跑它、证明点名的测试红），"
    echo "      或者确认它多余就整行删掉。⚠️ 同一个改法让**两条不同的测试**变红是合法的，这道检查按「改法 + 点名的测试」四项认，不会拦那一种。"
  fi
  if ((${#crates_filter[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行 cargo test 参数里的筛选词筛不到点名的测试（checker-tier-crates-mutation-replay 跑它时那条测试不跑，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_filter[@]}")
    echo "    → 怎么办：把 \`--\` 后面的筛选词改成点名测试名的一段（测试改过名、拆过的多半是这个），或者删掉筛选词跑整个测试目标。"
  fi
  if ((${#crates_ignored[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行点名的测试标了 #[ignore]，cargo test 参数里却没有 --include-ignored（checker-tier-crates-mutation-replay 跑它时那条测试被跳过，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_ignored[@]}")
    echo "    → 怎么办：在那一行 cargo test 参数的 \`--\` 后面加 --include-ignored（没有 \`--\` 就补 \`-- --include-ignored\`），改完单跑那一行证明点名的测试红。"
  fi
  if ((${#crates_malformed[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行不成形（六段制表符分隔：变异名、文件、原文、替换文、cargo test 参数、必须红的测试名；替换文可以空，那是删掉原文，其余五段不许空；门禁 checker-tier-crates-mutation-replay 派活之前同样判红，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_malformed[@]}")
    echo "    → 怎么办：补上空着的那一段，制表符删补到恰好六段；原文 / 替换文里的换行写成 \\n。要删掉原文就让替换文空着（两个制表符挨着）。"
  fi
  if ((${#crates_names[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的「必须红的测试名」不是一个用例名（带了正则符号或空白，门禁 checker-tier-crates-mutation-replay 按字面在测试输出里永远找不到，派活之前同样判红）："
    while IFS=$'\t' read -r _ lineno name expected; do
      printf '      crates/mutations.tsv:%s %s：「%s」\n' "$lineno" "$name" "$expected"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_names[@]}")
    echo "    → 怎么办：那一列写成用例名本身（或带模块路径 tests::<名字>），去掉 \$、^ 这类符号；checker-tier-crates-mutation-replay 已经按「在输出里只许对上一个用例」判，不用再靠 \$ 锚定。"
  fi
  if ((${#crates_bad[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 checker-tier-crates-mutation-replay 预扫会整张表退出，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_bad[@]}")
    echo "    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \\n），改完单跑那几行证明点名的测试红。"
  fi
  if ((${#scope_rows[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行改的文件不在门禁 checker-tier-crates-mutation-replay 每片拷的范围里（checker-tier-crates-mutation-replay 派活之前就整道判红，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name path; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$path"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_rows[@]}")
    echo "    → 怎么办：把那个路径（或它所在的目录）登记进 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay.sh 那一行（拷贝范围就是那一行登记的路径，现在是：${scope_copied}），"
    echo "      或者把这条变异改到拷贝范围之内的文件上。"
  fi
  if ((${#scope_literals[@]})); then
    echo "  ✗ crates/ 下这些 .rs 里有以 \"../ 起头、按所在 crate 的根解析之后逃出 crates/、又不在门禁 checker-tier-crates-mutation-replay 拷贝范围里的字符串字面量（checker-tier-crates-mutation-replay 的副本里没有那个路径，运行时读它的测试没改坏就红，checker-tier-crates-mutation-replay 记「基线不绿」）："
    while IFS=$'\t' read -r _ file lineno literal resolved; do
      printf '      %s:%s %s → %s\n' "$file" "$lineno" "$literal" "$resolved"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_literals[@]}")
    echo "    → 怎么办：把那个路径所在的目录登记进 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay 那一行（它同时是 checker-tier-crates-mutation-replay 的复用判定输入，登记之后 checker-tier-crates-mutation-replay 整张重跑一次），"
    echo "      或者让测试读 crates/ 之内的文件；#[path = …] 指的源文件不算，不用管。"
  fi
  exit 1
fi
n=$(ls "$BIN_DIR"/*.rs | wc -l)
crates_summary="；没有 crates/mutations.tsv"
[[ -f crates/mutations.tsv ]] && crates_summary="；crates/mutations.tsv ${scope_row_count} 条的原文各命中源码一次（成形：六段，替换文可以空、其余五段不空；点名的测试名都是用例名）"
crates_summary+="；门禁 checker-tier-crates-mutation-replay 的拷贝范围（${scope_copied}）：变异表 ${scope_row_count} 行改的文件都在里面，crates/ 下 ${scope_source_count} 份 .rs 里逃出 crates/ 的相对路径字面量 ${scope_literal_count} 处都在里面"
if ((${#tables_without_binary[@]})); then
  crates_summary+="；没有同名实验二进制、锚点不在这一格射程的表 ${#tables_without_binary[@]} 张：$(printf '%s ' "${tables_without_binary[@]}")"
fi
echo "  ✓ $n 个实验二进制都有成形的变异表，${anchor_checked} 条变异的原文各命中源码一次${crates_summary}（这一格不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复、点名标了 #[ignore] 的测试都带 --include-ignored、筛选词筛得到点名的测试——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）"
}

# ── 格 absolute-assertions：每个实验二进制都要有钉绝对值的断言
#
# 还 test-discipline.md 的这一条：
# 「只让多条臂互相比，测不出『所有臂一起错』……每一条互比断言旁边，
#   必须有一条把绝对值钉死的断言。」
#
# **判据只做机器判得了的那一半**：这个实验有没有**任何一条**把某个量与数字字面量比死的断言。
# 判不了的那一半（钉的是不是对的那个量、绝对值是不是独立算出来的）留给人。
#
# ⚠️ **零绝对值断言是一个可判定的、且实测出过问题的形态**（2026-08-29 对抗验证）：
# `e9_keylayout` 与 `e21_cpu` 当时各是 0 条，七个 / 两个单测钉的全是结构性质。
# e9 支撑 D8（核心索引结构） 的 key 布局（1.46×），e21 支撑 D24（后台重活能不能卸给 GPU）
# 的跨语言比值——两个都是承重结论，而它们量出来的那个数没有任何东西钉。
#
# 判别力已证（2026-08-29）：把 e9 的四条绝对值断言注释掉 ⇒ 这一格判红。
#
# 射程：research/e7-index-bench/src/bin 下的每一份，加上别处 `crates/*/src/bin`、`research/*/src/bin` 下文件名以 `e<数字>_` 开头的
# （实验编号的写法，`.claude/abbreviations` 登记的 `e<数字>`）——按「是不是实验」认，不按住在哪个目录认：
# 住在 crates/ 下的实验与 research/ 下的同规矩。别处不以 `e<数字>_` 开头的是装置工具（例：拿设备日志逐项比 ground truth 的），
# 不判，成功行逐个列名，清单现算；research/prompts/ 下腿的模型是冻结证据，不算实验二进制，不列。
# 三方判决：research/prompts/gate-fix-forks-r1-main-verification.md 的 T4。
#
# ⚠️ **断言要按语句认，不按行认**（2026-09-25 修）：rustfmt 会把长断言拆成多行——
# `assert!(` / `assert_eq!(` 单独一行起，中间的比较式、消息各占一行，`);` 单独一行收尾。
# 逐行 grep 只在断言仍是单行时管用，拆成多行之后同一条断言在任何一行上都凑不齐
# 「宏名 + 数字字面量 + 收尾括号」，会被误判成零绝对值断言（实测：
# `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` 被 rustfmt 拆行后，
# 这一格判它一条都没有；那份文件里的三条 `assert!(X == 字面量, "…")` 其实都在）。
# 判据本身不改，只改「怎么认出一条断言」：从 `assert(_eq)?!(` 起用圆括号配平找到语句收尾的那个 `)`，
# 拼成一条逻辑行再套判据；拼之前先挖掉字符串字面量的内容与行注释，
# 免得消息文本里的括号、或分号后的注释，把配平或判据算乱。
# 样本：red 放一份只比相对值的（e7 目录，单行，e1_relative）、一份住在 crates/ 下、
# 以 e<数字>_ 开头、只比相对值的（单行，e2_crates_relative），再加一份 e7 目录下拆成多行、但比的是字符串的（e2_multiline_relative，证明多行不会被误判成钉了绝对值），
# 与一份注释掉的、只判非零的、消息文本里带「, 5)」的（e6_commented_and_structural）；四份都要点名判红。
# green 里 e7 目录下每一份都钉了绝对值（e1_pinned 单行、e5_multiline_pinned 拆成多行用 `assert_eq!`，
# 另两份是别的格的样本各加了一条），crates/ 下一份单行（e3_crates_pinned）、一份拆成多行用 `assert!` 带消息（e4_crates_multiline_pinned，照 E142 那三条的样子），判绿；
# 另放一份不以 e<数字>_ 开头的 crates/demo/src/bin/probe.rs，成功行要把它列成没判的。

# 绝对值断言：与数字字面量比死。两种形态都认，判据不变（见这一格注释里的多行修法）。
# assert! 那一支只认 ==、<=、>=、<、>，而且运算符前一个字不能是 !、=、<、>：`!= 0` 是结构性质（非空、非零），不是把量钉死在一个数上。
ABSOLUTE_ASSERTION_PATTERN='assert_eq!\([^;]*, *-?[0-9][0-9_]*(\.[0-9]+)?\)|assert!\([^;]*[^!<>=](==|<=|>=|<|>) *-?[0-9][0-9_]*(\.[0-9]+)?[,)]'

# 把一个文件里被拆成多行的 assert!()/assert_eq!() 拼回一条条完整语句，逐条打到 stdout。
# 用圆括号配平判定语句收尾：从匹配到的 `assert(_eq)?!(` 起累计括号深度，深度回到 0 就是这一条的收尾。
# 拼之前用 strip() 把字符串字面量整体换成 `""`、行注释砍掉，配平只数真代码里的括号。
# 每行先掐掉两端空白再拼接、不额外插分隔符：判据要求「数字紧跟收尾括号」，
# rustfmt 常把收尾的 `);` 单独放一行、前面缩进一截——按原样拼会在数字和 `)` 之间垫出空白，
# 反而把这一条断言拼成判据认不出的样子；逐行掐两端空白再首尾相接，才是单行写法本来的样子。
join_assertion_statements() {
  awk '
    function trim(s) {
      gsub(/^[ \t]+|[ \t]+$/, "", s)
      return s
    }
    function strip(line,    s) {
      s = line
      gsub(/"([^"\\]|\\.)*"/, "\"\"", s)
      sub(/\/\/.*/, "", s)
      return s
    }
    function paren_delta(s,    i, c, d) {
      d = 0
      for (i = 1; i <= length(s); i++) {
        c = substr(s, i, 1)
        if (c == "(") d++
        else if (c == ")") d--
      }
      return d
    }
    BEGIN { in_statement = 0; depth = 0; buffer = "" }
    {
      raw_line = $0
      if (!in_statement) {
        # 起点在挖掉字符串与行注释之后的那一行上找：注释掉的 `// assert_eq!(x, 5);`、字符串里的「assert!(」都不是一条断言
        code_line = strip(raw_line)
        if (!match(code_line, /assert(_eq)?!\(/)) next
        piece = trim(substr(code_line, RSTART))
        buffer = piece
        depth = paren_delta(piece)
        in_statement = 1
      } else {
        # 判据套在挖掉字符串与行注释之后的语句上：消息文本里的「, 5)」不算把量钉死
        piece = trim(strip(raw_line))
        buffer = buffer piece
        depth += paren_delta(piece)
      }
      if (depth <= 0) {
        print buffer
        in_statement = 0
        buffer = ""
        depth = 0
      }
    }
  ' "$1"
}

cell_absolute_assertions() {
BINS=research/e7-index-bench/src/bin
# $BINS 不在不等于无对象：住在 crates/ 下、以 e<数字>_ 开头的实验照判；两处都一份没有，才在末尾退 77
judged=() uncovered=()
for f in "$BINS"/*.rs; do
  [[ -f "$f" ]] && judged+=("$f")   # 目录是空的时候 glob 原样留着，不是一份实验
done
in_bins=${#judged[@]}
for directory in crates/*/src/bin research/*/src/bin; do
  [[ -d "$directory" && "$directory" != "$BINS" ]] || continue
  for f in "$directory"/*.rs; do
    [[ -f "$f" ]] || continue
    if [[ "$(basename "$f")" =~ ^e[0-9]+_ ]]; then judged+=("$f"); else uncovered+=("$f"); fi
  done
done
bad=0; n=0
for f in "${judged[@]}"; do
  n=$((n+1))
  c=$(join_assertion_statements "$f" | grep -cE "$ABSOLUTE_ASSERTION_PATTERN")
  if (( c == 0 )); then
    echo "  ✗ $f 一条绝对值断言都没有"
    bad=$((bad+1))
  fi
done

if ((bad)); then
  echo "     → 怎么办：给它加一条把**被量的那个数**钉死的断言，绝对值要由**独立算术**给出，"
  echo "               不许从代码里读回来。加完用 research/scripts/mutate.sh 证明它会红——"
  echo "               实测教训：先加的断言可能一条变异都拦不住（E9 踩过），只有变异测试分得开。"
  exit 1
fi
((n)) || { echo "  ! $BINS 下一个 .rs 都没有、别处也没有 e<数字>_ 开头的，这一格无对象可判"; exit 77; }
echo "  ✓ $n 个实验二进制各自至少有一条绝对值断言（$BINS 下 $in_bins 份，别处 src/bin 下以 e<数字>_ 开头的 $(( n - in_bins )) 份）"
echo "    没判的 ${#uncovered[@]} 份（别处 src/bin 下不以 e<数字>_ 开头，按装置工具算）："
for f in ${uncovered[@]+"${uncovered[@]}"}; do echo "      $f"; done
}

# ── 格 reading-discipline：实验源码的两条读数纪律——种子不许折叠，读数不许恒为字面量 0
#
# 判据：扫 `research/e7-index-bench/src/bin/*.rs` 与住在 crates 下的实验装置 `crates/singlefs-checker-tier/src/bin/e<号>*.rs`
# （与 absolute-assertions 那一格同规矩：住在 crates 下的实验照样是实验），两条各自成段、两条都判完再退出（一次把问题说全）。
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
# 表对它们的三条闸：编号要在欠账表欠着那一张里找得到（按 `lib-owed.py` 读，doc-registries、checker-independence-and-sync 用的是同一份：
# 「### 已还清」整行标题之前的是开着的；认不出那个标题就判红，不对着一张认不出的表判）；每一行都要对得上这一轮真扫出来的一处违规
# （对不上就是已经改好了或者行号挪了，删掉或改掉这一行）；**只缩不涨**——
# 与基准提交比，同一个文件的登记行数不许变多，基准里没有的文件不许出现。基准里还没有这张表（初次登记）与
# git 取不到基准那一版是两回事：前者没有基线可比、成功行照实报，后者判红，不许当成初次登记放过去。
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
# 样本：red 里 e8_seed_and_readings.rs 同时犯两条，另带一张挂在认不出的欠账表上的豁免表、
# 一份住在 crates/singlefs-checker-tier/src/bin/ 下同样折叠了种子的装置，豁免表基准那一版的 blob 被 setup.sh 从对象库里删掉，必须判红；
# green 是同一份源码加上两张对得上的豁免表，必须判绿（别的格的实验二进制一处都不犯，扫到的文件数随之是 4）。
cell_reading_discipline() {
base=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  # 基准取法与 doc-process-records 用改动范围的几格、doc-experiments、doc-registries 的 invariant-anchors 那一格同一份：research/scripts/changed-paths.sh 的 gate 取法
  LIB_CHANGED_PATHS="$STAGE_REPOSITORY/research/scripts/changed-paths.sh"
  # shellcheck source=../../research/scripts/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  base="$(gate_diff_base gate)"
fi
python3 - "$base" "$OWED_LIBRARY" "$STAGE_DIRECTORY/lib/rejection-blocks.py" <<'PY'
import glob, importlib.util, os, re, subprocess, sys

base = sys.argv[1]
owed_library_spec = importlib.util.spec_from_file_location("owed", sys.argv[2])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)
# 逐块打印拒绝与 checker-independence-and-sync 的 layout-checker-sync 格共用这一份（lib/rejection-blocks.py）
rejection_blocks_spec = importlib.util.spec_from_file_location("rejection_blocks", sys.argv[3])
rejection_blocks = importlib.util.module_from_spec(rejection_blocks_spec)
rejection_blocks_spec.loader.exec_module(rejection_blocks)
BIN_GLOBS = ("research/e7-index-bench/src/bin/*.rs", "crates/singlefs-checker-tier/src/bin/e[0-9]*.rs")
BIN_GLOB = " 与 ".join(BIN_GLOBS)
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
paths = sorted(path for pattern in BIN_GLOBS for path in glob.glob(pattern))
if not paths:
    print(f"  ✗ {BIN_GLOB} 一个文件都没扫到")
    print("     → 怎么办：实验 bin 目录搬了家就同步改这一格里的 BIN_GLOBS（.claude/rules/path-moves.md）；")
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
           ["怎么办：把读不动的原因修掉（权限、编码）再跑；读不动的文件在这一格里既不算判过也不算判红，",
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
    # 「基准里没有这张表」与「git 取不到基准那一版」分开判：后者当成初次登记，只缩不涨就静默不比了
    listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-tree", "--name-only", base, "--", path],
                            capture_output=True, text=True)
    if listed.returncode != 0:
        reject(f"取不到基准 {base} 里的 {path}（git ls-tree 退 {listed.returncode}），「只缩不涨」这一条没比：",
               [listed.stderr.strip()[:200]],
               ["怎么办：按上面 git 的报错修好仓库状态（基准要存在、对象库没坏）再跑；取不到基线不是「初次登记」。"])
        return None
    if not listed.stdout.strip():
        return None
    result = subprocess.run(["git", "show", f"{base}:{path}"], capture_output=True, text=True)
    if result.returncode != 0:
        reject(f"读不出基准 {base} 里的 {path}（git show 退 {result.returncode}），「只缩不涨」这一条没比：",
               [result.stderr.strip()[:200]],
               ["怎么办：按上面 git 的报错修好仓库状态（多半是对象库缺了那一份）再跑；读不出基线不是「初次登记」。"])
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
                "               改了就要重跑这个实验并按新产物逐个回对正文引的数（门禁 checker-tier-research-build-and-replay 会逐字节判红直到重跑）。",
                f"               ② 今天改不动的，登记进 {lag_path}：四列写文件路径、定位、{owed_number}、为什么还没改。",
                f"               ⚠️ 这张表只缩不涨——新写的实验源码一律走出路 ①，{what_to_fix}。"])
    if lag and owed_unrecognised:
        reject(f"{label}：豁免表有 {len(lag)} 行要核欠账编号，而 {OWED_PATH} 里认不出「### 已还清」那一行标题，分不出哪些账还开着", [],
               ["怎么办：欠账表按「### 已还清」整行标题切成开着与还清两段（.claude/gate.d/lib-owed.py）；标题改了名或丢了就改回来，",
                "               别让这一格对着一张认不出的表判——认不出时连历史版本节里的表格行都会被算成开着的账。"])
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
                "               加得了一行，这一格对新写的实验源码就什么都拦不住了。",
                f"               真有非加不可的理由，那是一次规则变更：先改 {lag_path} 的表头注释与这一格的判据，说清为什么。"])
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
    rejection_blocks.print_blocks(failures, continuation_indent="       ")
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
}

stage_cells_run "$ROOT"
