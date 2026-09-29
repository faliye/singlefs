#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 实验页与产物（十格：索引行与正文标题、产物写回、产物与依据落在仓里、决策与实验双向登记、判决行的 false 被点名、复跑命令、实验号有正文、整行抄的产物行、上一轮记录已归档、多路径互证的登记）
# gate-category: 文档类
# 不声明 gate-covers：.claude/gate-not-implemented.tsv 里的两项（崩溃点重放、模型对拍）判的是被测代码的行为，这一道只判实验页、实验索引与产物的文本，一项都不覆盖。
# gate-similar: checker-tier-research-build-and-replay.sh 它的 experiment-replay 格按 research/scripts/replay.sh 登记表逐条复跑装置、逐字节比产物，要编译、归重型；这一道只读登记表、实验页与产物的文本，判产物有没有被点名、判决行的 false 有没有被解释、整行抄的产物行找不找得到，不跑装置
# gate-similar: doc-registries.sh 它的 experiment-refs 格判全 kb 里的实验号引用有定义，对象是引用；audit-contradictions 格核欠账表与里程碑收口表的引文在 kb 里找得到，方向是记录到 kb；segment-registry 格拿段序列登记表与 E142 的产物逐字比，对象是一份登记表。这一道判实验页、实验索引与产物本身，实验与决策之间的双向登记归这里的 decision-links 格，quoted-result-lines 格方向是 kb 与 research 正文到产物，multipath-registry 格还着 C48 可机检的那一半而不判欠账表本身
# gate-similar: doc-process-records.sh 也用 gate 取法的改动范围、也读 research/prompts/，但它判三方与同步留下的记录写没写、点没点名；这一道判产物与依据落没落进仓、新判决回看没回看决策、上一轮记录归没归档
# gate-similar: code-source-discipline.sh 它判每个实验二进制都有变异表、变异条目锚得住、钉绝对值的断言与读数纪律，对象是实验源码与变异表的内容；这一道的 evidence-in-repo 格只看这一轮改过的装置与变异表有没有一份不比它旧的产物
# gate-similar: code-tooling.sh 它的 change-range-single-source 格判各阶段取改动范围只经 changed-paths.sh 一份取法；这里是那份取法的使用者之一，改动范围在这一道里只取一次、六格共用
#
# 十格原是十道阶段（实验索引同步、产物写回、产物落在仓里、决策与实验双向登记、
# 判决行 false 字段被点名、复跑命令、孤儿实验号、整行抄的产物行、归档上一轮、
# 多路径登记），2026-09-28 按 .claude/singlefs-ai-sop/rules/sop-first.md「加门禁或钩子之前，先找已有的」第 2 条并成这一道：
# 十道都判 .claude/kb/experiments/ 的实验页、实验索引 .claude/kb/experiments.md、research/results/ 的产物与 research/scripts/replay.sh 的登记表。
# 每一格的判据、出路、成功行里报的数、射程与「管不到的」照原样留着；原文里互相点名的号改成了格名。
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就十格按下面的次序全跑，格名写错退 2）：
# gate-cell: index-sync 实验索引行与正文标题说的是不是同一件事
# gate-cell: results-cited 实验产物有没有写回
# gate-cell: evidence-in-repo 跑出来的产物与引来的依据都要落在仓里，不许只留在 /tmp
# gate-cell: decision-links 决策与实验双向登记，实验或结论变了之后回看过决策
# gate-cell: verdict-false-named 实验产物判决行里的 false 字段有没有被实验页点名
# gate-cell: repro-command 跑过的实验有没有留下复跑命令
# gate-cell: experiment-orphans research 里的实验号在 kb 里有没有正文
# gate-cell: quoted-result-lines kb 与 research 正文里整行抄的产物行，产物里逐字找得到
# gate-cell: archive-past-rounds 上一轮及更早的实验记录已经归档进版本库，工作区里不留
# gate-cell: multipath-registry 多条路径互证的实验，路径与共用项要登记且指得到
# （quoted-result-lines 只认去掉首尾空白后以 `E7RESULT ` 开头的行。）
#
# 改动范围：results-cited、evidence-in-repo、decision-links、quoted-result-lines、archive-past-rounds、multipath-registry 六格要「这次改动」，
# 只经共用库 research/scripts/changed-paths.sh 取一次、各格共用（选中的格里有这六格之一才取；不在 git 仓里不取）：
#   基准 gate_diff_base gate；改动的路径 gate_changed_paths <基准> untracked（evidence-in-repo、decision-links 用）；
#   新增的路径 gate_changed_paths <基准> untracked A（evidence-in-repo 用）；
#   新增的行 gate_added_lines <基准> .claude/kb ':(glob)research/**/*.md' 一次取全：results-cited 只看 .claude/kb/experiments.md 与
#   .claude/kb/experiments/ 下的、evidence-in-repo 只看 .claude/kb/experiments/ 下的、quoted-result-lines 全看；
#   archive-past-rounds 把同一个基准经环境变量 GATE_BASE 交给 research/scripts/archive-past-rounds.py（它取基准也是 gate_diff_base gate）。
#   取新增的行失败时三格一起按取不到办：results-cited 与 quoted-result-lines 退回全量判，evidence-in-repo 判红。
#   各格在自己的子 shell 里跑，所以改动范围在父进程里取、落进文件，各格只读；按 --check 选中的格取（样本根的 .gate-cells 在这之后才生效，那时按全部格取）。
#
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头），
# 这里只登记格、解析项目根、在父进程里取改动范围。每一格照原样打出自己的判定行（成功行报它查了多少项，红行带出路）。
#
# 判别力：fixtures/doc-experiments.sh/ 下每一格一对样本 <格名>-red 与 <格名>-green，verdict-false-named 另有一份 verdict-false-named-zero（登记表一行都读不到），
# evidence-in-repo 另有一对链接 worktree 里的 evidence-in-repo-worktree-red 与 evidence-in-repo-worktree-green。每一格的样本放了什么，写在那一格的注释里。
# 每份样本根放 .gate-cells 只点名它判的那一格，别的格不跑；setup.sh 里为别的格补的东西（空的 decisions/ 目录、待回填清单、点名产物与复跑命令的一句、
# 「原始输出未留存」的一句）是十格一起跑时留下的，留着不碍事。
# 汇总与 --check 的弄坏开关归共用库：STAGE_CELLS_BREAK=red-swallowed 让红样本整道退 0；STAGE_CELLS_BREAK=marker-ignored 让每份样本十格全跑，
# 汇总那条「跑了 1 格」的 want 找不到。
# 格的判法的弄坏开关：verdict-false-named 格的 GATE_VERDICT_FALSE_SKIP_NAMED_CHECK=1（把每个 false 字段都当成已点名）；
# evidence-in-repo 格的 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=worktree-skips-whole-cell（链接 worktree 里整格退 77，判据二也不判：evidence-in-repo-worktree-red 退 77、样本判错）
# 与 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=worktree-trusts-timestamps（链接 worktree 里照信修改时刻：evidence-in-repo-worktree-green 判红、样本判错）。
#
#   bash .claude/gate.d/doc-experiments.sh [项目根]                              十格都跑
#   bash .claude/gate.d/doc-experiments.sh --list                                逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-experiments.sh --check <格名>[,<格名>…] [项目根]     只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

STAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT_REPOSITORY_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
LIB_CHANGED_PATHS="$SCRIPT_REPOSITORY_ROOT/research/scripts/changed-paths.sh"
ARCHIVE_SCRIPT="$SCRIPT_REPOSITORY_ROOT/research/scripts/archive-past-rounds.py"
source "$STAGE_DIR/lib/stage-cells.sh"
# 要「这次改动」的六格：选中的格里有它们之一，才去取改动范围
CHANGE_RANGE_CELLS=" results-cited evidence-in-repo decision-links quoted-result-lines archive-past-rounds multipath-registry "

stage_cell index-sync cell_index_sync "实验索引行与正文标题说的是不是同一件事" \
  "以正文标题为准改 .claude/kb/experiments.md 的索引行（作废、状态词、结论列的数跟着正文），两边一一对应"
stage_cell results-cited cell_results_cited "实验产物有没有写回" \
  "在 experiments.md 对应实验的口径段点名 research/results/ 下的产物；确实留不下的写明「原始输出未留存」与原因；点名了而树里与历史里都没有的改成现存的名字"
stage_cell evidence-in-repo cell_evidence_in_repo "跑出来的产物与引来的依据都要落在仓里，不许只留在 /tmp" \
  "跑一次把产物存进 research/results/ 并在 research/scripts/replay.sh 指到它；把 /tmp 下当依据的东西拷进仓里（报告进 research/prompts/，数进 research/results/），引用改指仓内路径"
stage_cell decision-links cell_decision_links "决策与实验双向登记，实验或结论变了之后回看过决策" \
  "实验页补「### 影响的决策」表、决策分项的「**依据**」段引回实验，实验或结论变了就回看并在表里写日期与「改了 / 不受影响：理由」：.claude/rules/format-evolution.md「决策与实验双向登记」"
stage_cell verdict-false-named cell_verdict_false_named "实验产物判决行里的 false 字段有没有被实验页点名" \
  "在实验页那一次跑的历史条目里点名这个 false 字段并写明为什么是 false；replay.sh 登记表读不到行就先修表的形状"
stage_cell repro-command cell_repro_command "跑过的实验有没有留下复跑命令" \
  "在实验页正文的口径一节补一句复跑命令（格式与别处一致）；原始输出确实没留的写明「原始输出未留存」"
stage_cell experiment-orphans cell_experiment_orphans "research 里的实验号在 kb 里有没有正文" \
  "给这个实验号建正文 .claude/kb/experiments/<号>-*.md（测什么 / 判据 / 失败条款 / 口径 / 复跑命令），并登记进 experiments.md 的索引表"
stage_cell quoted-result-lines cell_quoted_result_lines "kb 与 research 正文里整行抄的产物行，产物里逐字找得到" \
  "打开这个实验复跑命令登记的那份产物，把对应的 E7RESULT 行原样抄回来，别手改数；产物换了就连同引它的正文一起换"
stage_cell archive-past-rounds cell_archive_past_rounds "上一轮及更早的实验记录已经归档进版本库，工作区里不留" \
  "照这一格打出来的基准跑 python3 research/scripts/archive-past-rounds.py --apply 删掉上一轮的记录（不带基准会把这一次提交要带的也删掉）"
stage_cell multipath-registry cell_multipath_registry "多条路径互证的实验，路径与共用项要登记且指得到" \
  "在实验页写「### 路径与结论登记」一节（形态见 .claude/rules/format-evolution.md），源码落点写现存路径、共用项写实；multipath-registry-lag.tsv 只缩不涨，补齐的删行"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 --list 看。"
      exit 2
      ;;
    *)
      if [[ -n "$root_argument" ]]; then
        echo "  ✗ 给了两个项目根：$root_argument 与 $1"
        echo "     → 怎么办：参数只认一个项目根与 --check <格名>[,<格名>…]；不给项目根就取这个脚本往上两级。"
        exit 2
      fi
      root_argument="$1"
      shift
      ;;
  esac
done
ROOT="${root_argument:-$SCRIPT_REPOSITORY_ROOT}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
ROOT="$(pwd)"

# ── 改动范围：只经共用库取一次，六格共用（在父进程里取、落进文件，各格的子 shell 只读）
changed_paths_library_loaded=0
in_git_work_tree=0
head_born=0
shared_base=""
changed_paths_ok=0
added_paths_ok=0
added_lines_ok=0
CHANGE_RANGE_DIRECTORY="$(mktemp -d)"
trap 'rm -rf "${CHANGE_RANGE_DIRECTORY:?}"' EXIT
SHARED_CHANGED_PATHS="$CHANGE_RANGE_DIRECTORY/changed-paths"
SHARED_ADDED_PATHS="$CHANGE_RANGE_DIRECTORY/added-paths"
SHARED_ADDED_LINES="$CHANGE_RANGE_DIRECTORY/added-lines"
: > "$SHARED_CHANGED_PATHS"; : > "$SHARED_ADDED_PATHS"; : > "$SHARED_ADDED_LINES"
change_range_needed=0
for cell in "${STAGE_CELLS_SELECTED[@]}"; do
  [[ "$CHANGE_RANGE_CELLS" == *" $cell "* ]] && change_range_needed=1
done
if ((change_range_needed)); then
  # shellcheck source=../../research/scripts/changed-paths.sh
  if source "$LIB_CHANGED_PATHS"; then changed_paths_library_loaded=1; fi
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    in_git_work_tree=1
    if git rev-parse --verify -q HEAD >/dev/null 2>&1; then head_born=1; fi
    if ((changed_paths_library_loaded)); then
      shared_base="$(gate_diff_base gate)"
      if gate_changed_paths "$shared_base" untracked > "$SHARED_CHANGED_PATHS"; then changed_paths_ok=1; fi
      if gate_changed_paths "$shared_base" untracked A > "$SHARED_ADDED_PATHS"; then added_paths_ok=1; fi
      # research/ 只取 .md：未跟踪的产物可能很大，整份读一遍白费
      if gate_added_lines "$shared_base" .claude/kb ':(glob)research/**/*.md' > "$SHARED_ADDED_LINES"; then added_lines_ok=1; fi
    fi
  fi
fi

# ── 格 index-sync：实验索引行与正文标题说的是不是同一件事
#
# `experiments.md` 的索引表是检索这批实验的入口，而编号的登记位在各正文的
# `## E<n> 简称 —— 状态` 那一行。**两处各写各的，就会一处改对、一处停在旧值**——
# 单独检索到索引那一行的人拿到的是作废结论，而它看起来完全正常。
#
# ⚠️ **这条是实测出来的**（2026-09-06 一轮实验↔决策一致性核查）：
# E69（反向索引取权威态的增量维护代价）的正文标题从 2026-08-31 起就是「结论整体作废」，
# 而索引行状态列写着「已跑（08-31，8 单测）」、结论列**原样登记着**被判作废的那两句
# （「无共享时恰好 0」是计数器从没自增过的仪表伪影，「按盘容量计费」被判为那个实现的性质）。
# E45（事务跨多条记录的环占用）与 E61（反向链 hash 算法的均匀性）的索引行都带了作废标记，
# 只有那一行没带 —— 一处漏改，五天没人看得出来。
#
# 查四样：
#   1. 正文标题带「结论作废 / 结论整体作废」的，索引行里必须也出现「作废」
#   2. 正文标题的状态词是「未跑」「部分已跑」或「已跑（够判）」的，索引状态列必须跟着说
#   3. 索引结论列里的数，正文里要找得到（千位分隔的空格先归一；按整个数找，「10」不许靠「100」里的子串算找到，
#      带小数的许正文多几位小数）
#   4. 两边一一对应：正文有而索引无 = 入了正文没进索引，索引有而正文无 = 指到空处
#
# ⚠️ **它抓不到的那一半要说清楚**：一个旧值只要还以历史叙述的形态留在正文里
# （E17（write buffer 合并上限）的「从 31.6M 抬到 31.8M」就是这样），
# 第 3 项照样判它「找得到」。**同一轮核查里 E17 与 E79（根记录的容量）两行的旧值都是人比出来的，
# 这一格拦不住它们** —— 拦得住的是第 1 项那一类：正文已经宣告作废而索引只字未提。
#
# 样本：index-sync-red 里一行正文宣告作废而索引没提、一行索引结论列的数正文里找不到、一行正文「已跑（够判）」而索引没跟、
# 一行按子串会被「100」冒充找到的「10」；index-sync-green 里补零链接、结论整体作废两边都写了、够判两边都写了。
cell_index_sync() {
  local IDX=.claude/kb/experiments.md
  local EXP=.claude/kb/experiments
  [[ -f "$IDX" && -d "$EXP" ]] || { echo "  ! 找不到 $IDX 或 $EXP，这一格跳过"; exit 77; }

  python3 - "$IDX" "$EXP" <<'PY'
import re, sys, glob, os

idx_path, exp_dir = sys.argv[1], sys.argv[2]

def unspace(s):
    for ch in (" ", " ", " "):   # 窄不换行空格、窄空格、不换行空格
        s = s.replace(ch, " ")
    return re.sub(r"(?<=\d)[ 　](?=\d\d\d)", "", s)

bodies = {}
for p in glob.glob(os.path.join(exp_dir, "*.md")):
    n = int(os.path.basename(p).split("-")[0])
    text = open(p, encoding="utf-8").read()
    bodies[n] = (os.path.basename(p), text.splitlines()[0], unspace(text))

rows = []
for line in open(idx_path, encoding="utf-8"):
    if not line.startswith("| E"):
        continue
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    m = re.match(r"E(\d+)", cells[0])
    if m and len(cells) >= 3:
        rows.append((int(m.group(1)), cells[1], cells[2], line))

def status_seg(title):
    seg = title.split("——", 1)[1] if "——" in title else title
    return seg.split("：", 1)[0]

bad = []
for n in sorted(set(bodies) - {r[0] for r in rows}):
    bad.append(f"E{n} 有正文（{bodies[n][0]}）却不在 experiments.md 的索引表里")
for n, st, concl, raw in rows:
    if n not in bodies:
        bad.append(f"E{n} 索引有行，{exp_dir}/ 下没有正文文件")
        continue
    fname, title, text = bodies[n]
    # 一：正文宣告结论作废，索引一个字不提
    # 只看状态列与结论列：简称里碰巧带「作废」二字（例如一个叫「…作废…」的实验）
    # 不能算索引说了这件事。
    if re.search(r"结论(整体)?作废|结论已作废", title) and "作废" not in (st + concl):
        bad.append(f"E{n} 正文标题写着结论作废，索引行里没有「作废」二字（{fname}）")
    # 二：状态词
    seg = status_seg(title)
    if seg.strip().startswith("未跑") and "未跑" not in st:
        bad.append(f"E{n} 正文状态是「未跑」，索引状态列写「{st}」")
    if "部分已跑" in seg and "部分" not in st:
        bad.append(f"E{n} 正文状态是「部分已跑」，索引状态列写「{st}」")
    if "够判" in seg and "够判" not in st:
        bad.append(f"E{n} 正文状态是「已跑（够判）」，索引状态列写「{st}」")
    # 三：索引结论列里的数，正文里找得到吗
    c = unspace(concl)
    for mm in re.finditer(r"\d[\d.]*", c):
        v = mm.group(0).rstrip(".")
        prev = c[max(0, mm.start() - 3):mm.start()]
        if re.search(r"[EDCIK\-]$", prev) or re.match(r"^20\d\d$", v) or len(v) <= 1:
            continue
        # 左边不许紧挨数字或小数点，整数右边不许紧挨数字：按子串比时「10 ms」会在「100 ms」里算找到。
        # 带小数的许正文多几位小数（索引写「10.6」、正文写「10.63」是索引按位截短，不算找不到）
        right_boundary = "" if "." in v else r"(?!\d)"
        if not re.search(r"(?<![\d.])" + re.escape(v) + right_boundary, text):
            bad.append(f"E{n} 索引结论列的「{v}」在 {fname} 正文里找不到")

if bad:
    print(f"  ✗ 实验索引与正文对不上 {len(bad)} 处：")   # gate-lint:summary
    for b in bad:
        print("      " + b)                                # gate-lint:detail
    print("  → 怎么办：以**正文标题**为准改索引行（编号的登记位是正文那行 `## E<n> 简称 —— 状态`，")
    print("    索引表只是导航）；若该改的是正文，就先改正文再回来改索引，并在")
    print("    .claude/kb/experiments-history.md 里按「改前 / 改后 / 依据」记一条。")
    print("    正文有而索引无的那一类：把它登记进 experiments.md 的索引表（编号、状态、一句话结论、正文链接）。")
    sys.exit(1)

if not rows and not bodies:
    print(f"  ! 本次无对象可判：{idx_path} 里一行 `| E<n>` 索引都没有，{exp_dir}/ 下也没有正文")
    sys.exit(77)
print(f"  ✓ 实验索引行与正文标题一致（索引 {len(rows)} 行、正文 {len(bodies)} 份）")
PY
}

# ── 格 results-cited：实验产物有没有写回
#
# 判据：`research/results/` 里的实验产物，必须在 `kb/experiments.md` 里被点名。
# 点不到名 = 跑过但结论没写回，或者写回了却无法复核（读的人拿不到那份原始数据）。
# 产物 = `research/results/` 下（含子目录）的每一个普通文件，不只 `*.out`（`.log`、`.txt` 一样是原始输出）；
# 顶层的按文件名、子目录里的按相对 `research/results/` 的路径点名。不算产物的几类照名字跳过，成功行逐个列名。
# 三道各报各的对象数；三道都一个对象都没有时这一格退 77（不记通过）。
#
# ⚠️ **这条是实测出来的，不是想出来的**：2026-08-29 有一次把 E20 从 2 档扩到 6 档、
# 跑了三轮、原始输出 22 KB 落了盘，而 experiments.md 里那一节还是两点对比，
# 决策侧一次都没引——报告只存在于对话里。
#
# 样本：results-cited-red 里一份没被点名的产物、一份名字像逐轮中间件而登记了却没点名的产物、一个标着已跑却没点产物的实验、
# 一页未跟踪的新实验页点名了树里与历史里都没有的产物；results-cited-green 里点名齐、一份没登记的逐轮中间件列进没判的清单。
cell_results_cited() {
  # ⚠️ 2026-08-29 起实验正文拆到 `kb/experiments/` 下，索引只剩导航表。
  # 两侧都要扫：产物可能被任一实验正文点名。
  EXP_DIR=.claude/kb/experiments
  EXP_ALL="$(mktemp)"; UNCITED="$(mktemp)"; NAMED="$(mktemp)"
  # 中间文件全用 mktemp 并交给 trap：固定路径加 $$ 不会撞，但判红那一支 exit 之前只删了一份，另一份每红一次就在 /tmp 留一份
  trap 'rm -f "$EXP_ALL" "$UNCITED" "$NAMED"' EXIT
  cat .claude/kb/experiments.md "$EXP_DIR"/*.md > "$EXP_ALL" 2>/dev/null
  EXP="$EXP_ALL"
  RES=research/results
  [[ -s "$EXP" ]] || { echo "  ! 找不到实验正文，这一格跳过"; exit 77; }
  [[ -d "$RES" ]] || { echo "  ! 没有 $RES 目录，这一格无对象可判"; exit 77; }

  # 三道都判完再退出，一次把问题说全；判的工具自己没跑成（awk 出错）才当场退出。
  failed=0
  missing=()
  # research/scripts/replay.sh 登记的入库产物（取行的形状与 verdict-false-named 格、门禁 checker-tier-research-build-and-replay.sh 的 experiment-replay 格同一份：编号|二进制|参数|产物|exact 或 timing，产物取文件名）：
  # 名字像逐轮中间件（*.r<一位数>.out）的，登记了照样要被点名。
  # replay.sh 不在或读不了：登记的逐轮产物认不出来，全被当成中间件跳过——有这种名字的产物时判红，不许静默放过。
  REPLAY_TABLE=research/scripts/replay.sh
  replay_table_readable=1
  REGISTERED=""
  if [[ -r "$REPLAY_TABLE" && -f "$REPLAY_TABLE" ]]; then
    REGISTERED="$(sed -nE 's/^E[0-9]+\|[^|]*\|[^|]*\|([^|]*\/)?([^|/]*)\|(exact|timing)$/\2/p' "$REPLAY_TABLE")" || replay_table_readable=0
  else
    replay_table_readable=0
  fi
  judged_products=0
  not_products=()
  round_named_skipped=()
  while IFS= read -r f; do
    relative="${f#"$RES"/}"
    b="$(basename "$f")"
    # 三类不算产物：本地腿的问答、逐轮复跑的中间件（replay.sh 登记了的除外）、确认目录
    case "$relative" in
      *local*|*.round*|confirm*) not_products+=("$relative"); continue ;;
      *.r[0-9].out)
        case "$(printf "\n%s\n" "$REGISTERED")" in
          *"$(printf "\n%s\n" "$b")"*) : ;;
          *) not_products+=("$relative"); round_named_skipped+=("$relative"); continue ;;
        esac ;;
    esac
    judged_products=$((judged_products + 1))
    grep -qF "$relative" "$EXP" || missing+=("$relative")
  done < <(find "$RES" -type f | sort)

  if ((${#missing[@]})); then
    echo "  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核："
    for m in "${missing[@]}"; do echo "     $RES/$m"; done
    echo "     → 怎么办：把结论写进 experiments.md 对应实验的正文，"
    echo "               并在口径段点名这份原始输出；确实是废弃产物就删掉它。"
    failed=1
  elif ((judged_products == 0)); then
    echo "  ! 第一道本次无对象可判：$RES 下没有要点名的产物（不算产物的 ${#not_products[@]} 份除外）"
  else
    echo "  ✓ $RES 下的实验产物全部被 experiments.md 点名（查了 ${judged_products} 份）"
  fi
  if ((replay_table_readable == 0 && ${#round_named_skipped[@]})); then
    echo "  ✗ 读不到 $REPLAY_TABLE：它登记的入库产物认不出来，${#round_named_skipped[@]} 份名字像逐轮中间件的产物全被当成中间件跳过了"
    echo "     → 怎么办：replay.sh 搬了家或改了名，就把这里的 REPLAY_TABLE 一起改（.claude/rules/path-moves.md）；读权限丢了就补上。"
    failed=1
  fi
  if ((${#not_products[@]})); then
    echo "     第一道没判的 ${#not_products[@]} 份（本地腿的问答、没登记的逐轮中间件、确认目录）："
    printf '       %s\n' "${not_products[@]}"
  fi

  # ── 反方向：已跑的实验必须要么点名产物，要么显式说明产物没留 ──
  # 只查一个方向会漏掉「实验写了结论、但产物从没存在过」——那种情况下
  # 读的人既翻不到档案也不知道翻不到，比明说「没留」更糟。
  awk '
    /^## E[0-9]+ /{
      if (cur != "" && done && !cited && !excused) print cur
      cur=$2; done=($0 ~ /已跑|已测/); cited=0; excused=0
      if (done) ran_count++
      next
    }
    # 目录名后面要跟着一个文件名才算点名：只写「产物放在 research/results/ 下」的那一节什么都没点
    /research\/results\/[A-Za-z0-9_.-]/{ cited=1 }
    # 归档之后引用只剩文件名（「每一次提交删上一次的实验记录」，产物去版本库历史里查），
    # 所以裸文件名也算点了名——但它点的那份下面还要逐个去树里与 git 历史里找得到，
    # 不然「点名」就退化成「写个像文件名的串」。
    # 前面要有词边界：少了它，`-stage1.out` 里的 "stag|e1.out" 会被咬成一个产物名
    match($0, /(^|[^a-zA-Z0-9_-])e[0-9]+[a-zA-Z0-9._-]*\.out/) {
      tok=substr($0, RSTART, RLENGTH); sub(/^[^a-zA-Z0-9_-]/, "", tok)
      named[tok]=cur; cited=1
    }
    /原始输出未留存|输出未留存/{ excused=1 }
    END{ if (cur != "" && done && !cited && !excused) print cur
         for (n in named) print "NAMED" "\t" n "\t" named[n] > "/dev/stderr"
         print "RAN_COUNT" "\t" ran_count + 0 > "/dev/stderr" }
  ' "$EXP" > "$UNCITED" 2>"$NAMED" || {
    echo "  ✗ 判「已跑实验有没有点名产物」的 awk 没跑成（退出码 $?），这一格没判："
    sed 's/^/     /' "$NAMED"   # gate-lint:detail
    echo "     → 怎么办：看上面 awk 的报错修这一段；它没跑完时未点名的实验一个都不会被列出来，不许当成通过。"
    exit 1
  }
  if [[ -s "$UNCITED" ]]; then
    echo "  ✗ 这些实验标着已跑，却既没点名原始输出、也没说明产物为什么没留："
    sed 's/^/     /' "$UNCITED"
    echo "     → 怎么办：存一份产物并在口径段点名；确实留不下（要虚机/真设备）"
    echo "               就写明「原始输出未留存」以及为什么，别让读的人以为能翻到。"
    failed=1
  fi
  rm -f "$UNCITED"
  ran_count="$(sed -n 's/^RAN_COUNT\t//p' "$NAMED")"

  # ── 第三道：点名的产物，树里没有就必须在版本库历史里找得到 ──
  # 射程与 quoted-result-lines 格同一条（用户 2026-09-21 定）：只判**这次改动新增或改写的**点名行。
  # 早先写下的点名是历史，仅作参考；而别的会话正在跑、产物还没提交的实验，它的页也不该由这一次提交来判。
  # 改动范围取这一道开头只经共用脚本 research/scripts/changed-paths.sh 取一次的那一份（基准 gate_diff_base gate，新增行 gate_added_lines，
  # 这一格只看 .claude/kb/experiments.md 与 .claude/kb/experiments/ 下的：未跟踪的实验页整份算新增——新写的页在 git add 之前，里面点名的产物也要判）。
  # 不在 git 仓里、或取不到改动范围，就判全部（保守）。
  # ⚠️ 没有上游、也没设 GATE_BASE 时，共用脚本的基准是 HEAD，窗口只含工作区与暂存区（与 quoted-result-lines 格同一格，交用户定：gate-fix-forks-r1-forks.md 的 T11）。
  # 少了这一道，上一道就退化成「正文里写个像文件名的串」：归档之后引用只剩文件名，
  # 而一个从没存在过的文件名与一份归档进历史的产物，在正文里长得一模一样。
  if ((! changed_paths_library_loaded)); then
    echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
    echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"
    exit 1
  fi
  BASE="" touched="" scope_note="全部的行（不在 git 仓里，判全部）"
  if ((in_git_work_tree)); then
    BASE="$shared_base"
    if ((added_lines_ok)); then
      touched="$(awk -F'\t' '$1 == ".claude/kb/experiments.md" || $1 == ".claude/kb/experiments" || index($1, ".claude/kb/experiments/") == 1' "$SHARED_ADDED_LINES" \
                 | cut -f2- | grep -oE "e[0-9]+[a-zA-Z0-9._-]*\.out" | sort -u)"
      scope_note="这次改动（基准 $BASE）新增或改写的行"
    else
      BASE="" scope_note="全部的行（取不到基准 $shared_base 起的新增行，判全部）"
    fi
  fi
  missing=0; checked=0; skipped=0
  declare -A skipped_names_by_owner=()
  while read -r _ name owner; do   # 字段是 NAMED、文件名、实验号，都不含空格，默认分隔够用
    [[ -n "$name" ]] || continue
    # 不用管道接 grep -q：pipefail 下管道的退出码会被后一段盖掉（command-safety.md）。
    # 这里拿 case 在 shell 里逐字比：$touched 是换行分隔的一串名字，两端各补一个换行再比整段。
    if [[ -n "$BASE" ]]; then
      case "$(printf "\n%s\n" "$touched")" in
        *"$(printf "\n%s\n" "$name")"*) : ;;
        *) skipped=$((skipped+1))
           skipped_names_by_owner["${owner:-（不在任何实验节里）}"]+=" $name"
           continue ;;
      esac
    fi
    checked=$((checked+1))
    [[ -f "$RES/$name" ]] && continue
    # 正在归档的：HEAD 里还在、工作区里删了（这一批照归档规则删上一轮的产物，删除还没提交，下一行的 git log 还找不到它）
    git -c core.quotepath=false cat-file -e "HEAD:$RES/$name" 2>/dev/null && continue
    if [[ -n "$(git -c core.quotepath=false log --all --diff-filter=D --format=%h --name-only -- "*$name" 2>/dev/null | head -1)" ]]; then
      continue
    fi
    echo "     $owner 点名 $name —— 树里没有，git 历史里也没有"   # gate-lint:detail
    missing=$((missing+1))
  done < <(grep '^NAMED' "$NAMED" | sort -u)
  rm -f "$NAMED"
  if [[ $missing -gt 0 ]]; then
    echo "  ✗ $missing 份被点名的产物既不在 $RES 下、也不在版本库历史里"   # gate-lint:summary
    echo "     → 怎么办：产物归档了就该能从历史取回（git log --all --diff-filter=D --name-only 找删它的提交，"
    echo "               再 git show <提交>^:<路径> 读回来）；取不回来说明它从没存在过，"
    echo "               把那一节改成「原始输出未留存」并写明为什么，别让读的人以为翻得到。"
    failed=1
  elif ((checked == 0)); then
    echo "  ! 第三道本次无对象可判：$scope_note里没有点名产物的"
  else
    echo "  ✓ $scope_note点名的 $checked 份产物在树里或版本库历史里都找得到"
  fi
  if ((skipped)); then
    # 没判的现算：不在这次改动新增或改写的行里的点名（早先写下的，历史参考；射程与 quoted-result-lines 格同一条），按实验逐个列
    echo "     第三道没判的 $skipped 份：不在这次改动新增或改写的行里的点名，按实验列："
    while IFS= read -r skipped_owner; do
      echo "       $skipped_owner：${skipped_names_by_owner[$skipped_owner]# }"
    done < <(printf '%s\n' "${!skipped_names_by_owner[@]}" | sort -V)
  fi
  ((failed)) && exit 1
  if ((judged_products == 0 && ${ran_count:-0} == 0 && checked == 0)); then
    echo "  ! 本次无对象可判：没有要点名的产物、没有标着已跑的实验、这次改动里也没有点名产物的行"
    exit 77
  fi
  if ((${ran_count:-0} == 0)); then
    echo "  ! 第二道本次无对象可判：没有标着已跑的实验"
  else
    echo "  ✓ 已跑的 $((ran_count)) 个实验都点了名或写明了产物未留存"
  fi
}

# ── 格 evidence-in-repo：跑出来的产物与引来的依据都要落在仓里，不许只留在 /tmp
#
# 实测（2026-09-18 核出）：E142（新池新建文件的干跑） 第十一次跑改了装置与变异表、跑了一个半小时，
# 产物只在 /tmp/claude-1000/e142-r11-runner/e142-r11-draft.out，research/results/ 里最新的还是 09-17 那份，
# research/scripts/replay.sh 的登记行也还指着它；另一处是一份还清依据写成 /tmp/claude-1000/<轮>/report.md。
# /tmp 下的草稿目录会话一重启就没了，那一轮的活等于白干，而此前没有任何东西报警。用户 2026-09-18 定：这类事拿门禁约束。
#
# 改动范围：这一道开头只经共用库 research/scripts/changed-paths.sh 取一次的那一份（GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，
# 都没有就与 HEAD 比；工作区、暂存区与未跟踪文件都算；与 doc-process-records.sh 用改动范围的几格、doc-registries.sh 的 invariant-anchors 那一格同一份代码；
# 它的 git 调用带 core.quotepath=false，中文路径不转义）。
#
# 判据一「实验跑了没留存」：改动范围里每个还在盘上的
#   research/e7-index-bench/src/bin/e<号>_*.rs 或 research/mutations/e<号>_*.tsv，
#   research/results/ 下（含子目录）要有一份这个实验号的产物——文件名形如 e<号> 后面跟非数字或到头（e142-…、e142_…、e12.1.out）——
#   而且它的 mtime 不早于那份源码。一个这样的产物都没有、或最新的那份比源码旧 ⇒ 红。
#   登记了准入的实验（.claude/gate.d/stage-inputs.tsv 有 E<号> 那一行）先按输入指纹判（research/scripts/admission.py 的 stored_product_status，
#   与装置入口、replay.sh 同一份算法）：research/results/ 里有一份产物头上记着 E<号> 的指纹、与今天的输入算出来的相同 ⇒ 绿，mtime 再旧也算；
#   带指纹的产物都对不上 ⇒ 红；一份带指纹的都没有 ⇒ 照上面按 mtime 判。mtime 会被 git checkout / stash pop 刷新，指纹不会。
#   红行里的改动日子按东京的日期报，不报钟点。
#   只改了 // 注释的装置源码（与基准比，增删的每一行都是 // 注释或空行）不判、在成功行里逐个列名：
#   path-moves.md 要求全仓改路径，注释里的路径改了碰不到产物（2026-09-18 实测六份装置因此误判）。
#   按要求才跑的实验（research/on-request-experiments.tsv 登记的实验号，例：E152 按里程碑、用户说跑才跑）不要求新产物：
#   装置改了、产物旧时改查它的实验页（.claude/kb/experiments/<号>-*.md）这一轮新加的行（与基准比，按 gate_added_lines 取；未跟踪的整份算新加）
#   里写没写「没有正式跑」或「未正式跑」，写了就在成功行列名，没写 ⇒ 红。页里早先那句说的是上一次改装置，不给这一次作证。
#   用户 2026-09-19 定：E152 本质上是用户要跑才跑的里程碑性能对比，每次改装置都跑没有意义。
# 判据二「证据指向仓外」：下面这些文件里，把 /tmp 路径当依据引用 ⇒ 红。认的形态是
#   「依据 / 证据 / 出处 / 产物 / 报告 / 原件 / 结论 / 存档 / 留档 / 留存 / 记录 / 落点 / 见 / 详见 / 参见」
#   之后隔至多两个空白、至多一个连接记号（是 / 在 / 为 / ： / : / = / →）、至多两个空白，再（可带一个 ` 或 「）接上 /tmp/ 路径。
#   判的文件是：.claude/kb/ 下全部 .md；research/prompts/ 下这一轮**新写**的 .md（新增或未跟踪）。
#
# 为什么两半的射程不一样：kb 正文只写现状（kb-discipline.md 第 8 条），旧文件照样该改；
# 而 research/prompts/ 登记在 .claude/doc-lint-exclude 里，是「原样保存的证据不许事后改」
# （evidence-discipline.md），一道要求回去改旧提示的检查只会逼出证据链断掉或整个被绕过（sop-first.md）。
# 新写的那一批还没冻，此刻正是能改的时候，所以只判它们。
#
# 不判的，成功行现算着列：这次改动没碰实验装置与变异表时判据一没有对象；
# research/prompts/ 下这一轮没新写的 .md（数目现算）；判据二只扫 .md，模型目录里的 .py 不扫。
# 在链接 worktree 里（git rev-parse --git-dir 落在 .git/worktrees/ 下；research/scripts/gate-staged.sh 与 --staged 的临时 worktree 就是这种）
# 全部文件是检出那一刻写的，mtime 分不出装置与产物谁新：判据一里只能比修改时刻的那几份（产物在、没按指纹判出对不上）不判，
# 在这一格的输出里逐份列成「本次未跑」，并往 $GATE_NOT_RUN_FILE 报一行（这个变量不在时不写）；
# 判据一里不看时间戳的（按指纹判的、一份产物都没有的、只改了注释的）与判据二照判——提交路径上的整轮门禁就跑在链接 worktree 里，
# 判据二因此在提交时判得到。判据一全是要比修改时刻的、判据二又一份 .md 都没有时，这一格无对象可判（77）。
# 比修改时刻的那几份，不带 --staged 在工作区里跑这一格才判得了。
#
# ⚠️ 管不到的：产物对不对得上源码（那是 checker-tier-research-build-and-replay.sh 的逐字节复跑，这一格只看有没有、新不新，
# 而变异表的重跑日志 checker-tier-research-build-and-replay.sh 一行都不看）；replay.sh 的登记行指没指到最新那份（只写在出路里，不判）；
# 句子是说做法还是引依据（这一格按上面那个形状认，说「草稿放 /tmp/…」不算引依据；反过来，
# 一句「报告原件在 /tmp/…，这里已原样转存」照样判红，改法是把 /tmp 那一截删掉）；
# 已经提交的 prompts 文件后来加进去的 /tmp 依据（只判整份新写的文件，不判改动的行）；
# git checkout / stash pop 会把源码的 mtime 刷新，那时判据一会要求重跑一次。
# 判别力：fixtures/doc-experiments.sh/evidence-in-repo-red 放一个产物比源码旧的装置、一个一份产物都没有的变异表、
# 一份 kb 里写「依据：/tmp/…」、一份这一轮新写的 prompts 里写「报告 `/tmp/…`」，必须判红；
# evidence-in-repo-green 放一个产物比源码新的装置、一个不在改动范围里的旧装置、kb 里三种说做法的 /tmp 写法、
# 一份改过但不是新写的 prompts 里的「依据：/tmp/…」，必须判绿并报对数。
# 指纹那一半：red 放一个登记了准入、产物比源码新而产物头的指纹对不上的装置（E910，按 mtime 会判绿）；
# green 放一个登记了准入、产物比源码旧而产物头的指纹与今天的输入相同的装置（E909，按 mtime 会判红），指纹由 setup.sh 用 sha256sum 另算。
# 链接 worktree 那一半（setup.sh 在样本根里建主仓 .origin-repository/，把它的一个链接 worktree 搬到样本根）：
# evidence-in-repo-worktree-red 放一份 kb 里写「依据：/tmp/…」、一份这一轮新写的 prompts 里写「报告 `/tmp/…`」、一个产物比源码旧的装置（E920），
# 必须判红、点名两处 /tmp 依据、把 E920 列成本次未跑；evidence-in-repo-worktree-green 放一个产物比源码旧的装置（E921）与只说做法的 /tmp 写法，
# 必须判绿、把 E921 列成本次未跑。弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=worktree-skips-whole-cell（整格退 77）让 -red 退 77，
# =worktree-trusts-timestamps（照信检出时写的修改时刻）让 -green 判红。
cell_evidence_in_repo() {
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，这一格跳过"; exit 77; }
  # 链接 worktree 里时间戳是检出那一刻写的：只有判据一里要比修改时刻的那几份不判，别的照判（见这一格的注释）
  local timestamps_from_checkout=0
  if [[ "$(git rev-parse --git-dir 2>/dev/null)" == */worktrees/* ]]; then
    timestamps_from_checkout=1
    if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == worktree-skips-whole-cell ]]; then
      echo "  ! 弄坏开关 worktree-skips-whole-cell：$ROOT 是链接 worktree，整格退 77，判据二也不判"
      exit 77
    fi
    if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == worktree-trusts-timestamps ]]; then
      echo "  ! 弄坏开关 worktree-trusts-timestamps：$ROOT 是链接 worktree，照信检出时写的修改时刻"
      timestamps_from_checkout=0
    fi
  fi
  if ((! changed_paths_library_loaded)); then
    echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"
    echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"
    exit 1
  fi
  base="$shared_base"
  if ((! changed_paths_ok || ! added_paths_ok)); then
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    exit 1
  fi
  # 实验页这一轮新加的行：按要求才跑的实验只认这一轮写进实验页的「没有正式跑」，页里早先那句说的是上一次改装置
  if ((! added_lines_ok)); then
    echo "  ✗ 取不到实验页这一轮新加了哪些行（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到时按要求才跑的实验判不了，不是通过。"
    exit 1
  fi
  # 几份清单经文件与进程替换当文件传：当成命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - "$base" "$SHARED_CHANGED_PATHS" "$SHARED_ADDED_PATHS" "$(dirname "$LIB_CHANGED_PATHS")" \
    <(awk -F'\t' '$1 == ".claude/kb/experiments" || index($1, ".claude/kb/experiments/") == 1' "$SHARED_ADDED_LINES") "$timestamps_from_checkout" <<'PY'
import os, re, subprocess, sys
from datetime import datetime
from zoneinfo import ZoneInfo

base = sys.argv[1]
# 链接 worktree 里（1）修改时刻是检出时写的：判据一里要比修改时刻的那几份不判，列成本次未跑
timestamps_from_checkout = sys.argv[6] == "1"
# 准入模块（门禁与实验共用，research/scripts/admission.py）：登记了准入的实验按产物头的输入指纹判，不按修改时刻判。
sys.path.insert(0, sys.argv[4])
try:
    import admission
except Exception as error:
    print(f"  ✗ 读不到准入模块 {sys.argv[4]}/admission.py：{error}")
    print("     → 怎么办：恢复 research/scripts/admission.py（判「产物跟不跟得上输入」的指纹只有那一份算法），再跑这一格。")
    sys.exit(1)

def read_list(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return sorted({line for line in handle.read().split("\n") if line})

changed_files = read_list(sys.argv[2])
added_files = set(read_list(sys.argv[3]))

results_dir = "research/results"
kb_dir = ".claude/kb"
prompts_dir = "research/prompts"

source_forms = (
    (re.compile(r"^research/e7-index-bench/src/bin/e([0-9]+)_[^/]*\.rs$"), "装置源码"),
    (re.compile(r"^research/mutations/e([0-9]+)_[^/]*\.tsv$"), "变异表"),
)
product_form = re.compile(r"^e([0-9]+)(?![0-9])")
on_request_list = "research/on-request-experiments.tsv"
on_request_numbers = set()
if os.path.isfile(on_request_list):
    with open(on_request_list, encoding="utf-8") as handle:
        for line in handle:
            cells = line.split("#", 1)[0].split("\t")
            if cells[0].strip():
                on_request_numbers.add(cells[0].strip().lstrip("Ee"))
not_run_note = re.compile(r"没有正式跑|未正式跑")
# 实验页这一轮新加的行（gate_added_lines 的「路径<TAB>行文」）
added_experiment_page_lines = []
with open(sys.argv[5], encoding="utf-8", errors="replace") as handle:
    for added in handle.read().split("\n"):
        if "\t" in added:
            added_experiment_page_lines.append(tuple(added.split("\t", 1)))

def experiment_page_says_not_run(number):
    """按要求才跑的实验，这一轮有没有在实验页新写一句「装置改过、之后没有正式跑」。
    只认这一轮新加的行：页里早先那句说的是上一次改装置（门禁说明里提到这个词的那种句子也在其中），不给这一次作证。"""
    page_prefix = os.path.join(kb_dir, "experiments", f"{number}-")
    return any(page_path.startswith(page_prefix) and not_run_note.search(page_line)
               for page_path, page_line in added_experiment_page_lines)

# ── 判据一：改动范围里的实验装置与变异表，要有一份不比它旧的产物 ───────────
changed_sources = []
for path in changed_files:
    for form, kind in source_forms:
        match = form.match(path)
        if match and os.path.isfile(path):
            changed_sources.append((path, match.group(1), kind))
            break

newest_product_by_number = {}
if os.path.isdir(results_dir):
    for directory, _subdirectories, file_names in os.walk(results_dir):
        for file_name in file_names:
            match = product_form.match(file_name)
            if not match:
                continue
            product_path = os.path.join(directory, file_name)
            try:
                modified_at = os.path.getmtime(product_path)
            except OSError:
                continue
            number = match.group(1)
            previous = newest_product_by_number.get(number)
            if previous is None or modified_at > previous[1]:
                newest_product_by_number[number] = (product_path, modified_at)

TOKYO = ZoneInfo("Asia/Tokyo")

def moment(seconds):
    """修改时刻按东京的日期报（只报日子，不报钟点）：比新旧用的是秒，这里只是给人看的那一截。"""
    return datetime.fromtimestamp(seconds, TOKYO).strftime("%Y-%m-%d")

def changes_only_comments(path):
    """装置源码与基准比，增删的每一行都是 // 注释或空行：这种改动（多半是按 path-moves.md 改注释里的路径）碰不到产物。"""
    if path in added_files:
        return False
    diff = subprocess.run(["git", "-c", "core.quotepath=false", "diff", "-U0", base, "--", path],
                          capture_output=True, text=True, errors="replace")
    if diff.returncode != 0:
        return False
    changed_lines = [line[1:] for line in diff.stdout.split("\n")
                     if line[:1] in "+-" and not line.startswith(("+++", "---"))]
    return bool(changed_lines) and all(line.strip() == "" or line.lstrip().startswith("//") for line in changed_lines)

unstored_runs = []
comment_only_sources = []
on_request_not_run = []
on_request_missing_note = []
fingerprint_matched = []
fingerprint_absent = []
fingerprint_errors = []
timestamp_unjudged = []
for path, number, kind in changed_sources:
    if kind == "装置源码" and changes_only_comments(path):
        comment_only_sources.append(path)
        continue
    source_modified_at = os.path.getmtime(path)
    newest = newest_product_by_number.get(number)
    # 先按指纹判：登记了准入（.claude/gate.d/stage-inputs.tsv 有 E<号> 那一行）、research/results/ 里有带这个键指纹的产物时，
    # 指纹相同就是产物按今天的输入跑的（修改时刻再旧也算），不同就是跟不上；没登记或没有一份带指纹的，照旧按修改时刻判。
    try:
        fingerprint_status, fingerprint_detail = admission.stored_product_status(".", f"E{number}")
    except (admission.InputManifestError, admission.RegistrationError) as error:
        fingerprint_errors.append(f"{path}（{kind}）：E{number} 登记了准入，却算不出输入指纹：{error}")
        continue
    if fingerprint_status == "matched":
        fingerprint_matched.append(f"E{number}（{path}；{fingerprint_detail}）")
        continue
    if fingerprint_status == "unfingerprinted":
        fingerprint_absent.append(f"E{number}")
    fingerprint_stale = fingerprint_status == "stale"
    if timestamps_from_checkout and not fingerprint_stale and newest is not None:
        # 产物在、指纹没判出对不上，剩下只能比修改时刻，而修改时刻是检出时写的：这一份本次未跑
        timestamp_unjudged.append(f"{path}（{kind}，E{number}）")
        continue
    if number in on_request_numbers and (fingerprint_stale or newest is None or newest[1] < source_modified_at):
        if experiment_page_says_not_run(number):
            on_request_not_run.append(f"E{number}（{path}）")
        else:
            on_request_missing_note.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：E{number} 按要求才跑，实验页没写明装置改过之后没有正式跑"
                                           "（只认这一轮新加进实验页的那一句）")
        continue
    if fingerprint_stale:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：E{number} 的输入与产物头的指纹对不上，{fingerprint_detail}")
        continue
    if newest is None:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：{results_dir} 下一份 E{number} 的产物都没有")
    elif newest[1] < source_modified_at:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：最新的 E{number} 产物 {newest[0]} 还是 {moment(newest[1])} 那份")

# ── 判据二：kb 与这一轮新写的提示里，不许把 /tmp 路径当依据引用 ────────────
citation_form = re.compile(
    r"(依据|证据|出处|产物|报告|原件|结论|存档|留档|留存|记录|落点|详见|参见|见)"
    r"[ \t]{0,2}(?:是|在|为|：|:|=|→)?[ \t]{0,2}[`「]?"
    r"(/tmp/[^\s`\"'()（）「」，。；：、|]*)"
)
tmp_form = re.compile(r"/tmp/[^\s`\"'()（）「」，。；：、|]*")

scanned_files = []
skipped_prompt_files = 0
if os.path.isdir(kb_dir):
    for directory, _subdirectories, file_names in os.walk(kb_dir):
        for file_name in sorted(file_names):
            if file_name.endswith(".md"):
                scanned_files.append(os.path.join(directory, file_name))
if os.path.isdir(prompts_dir):
    for directory, _subdirectories, file_names in os.walk(prompts_dir):
        for file_name in sorted(file_names):
            if not file_name.endswith(".md"):
                continue
            path = os.path.join(directory, file_name)
            if path in added_files:
                scanned_files.append(path)
            else:
                skipped_prompt_files += 1

outside_citations = []
tmp_mentions = 0
for path in sorted(set(scanned_files)):
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError:
        continue
    for line_number, line in enumerate(lines, 1):
        tmp_mentions += len(tmp_form.findall(line))
        for match in citation_form.finditer(line):
            excerpt = line[max(0, match.start() - 24):match.end()].strip()
            outside_citations.append(f"{path}:{line_number}：…{excerpt}")

if not changed_sources and not scanned_files:
    print(f"  ! 这次改动没碰实验装置与变异表（基准 {base}），{kb_dir} 与 {prompts_dir} 下也没有要扫的 .md，这一格无对象可判")
    sys.exit(77)
if changed_sources and len(timestamp_unjudged) == len(changed_sources) and not scanned_files:
    print(f"  ! 链接 worktree 里这一轮改过的 {len(changed_sources)} 份实验装置与变异表都要比修改时刻（检出时写的，判不了），"
          f"{kb_dir} 与 {prompts_dir} 下也没有要扫的 .md，这一格无对象可判：{'、'.join(timestamp_unjudged)}")
    sys.exit(77)

failed = False
if unstored_runs:
    failed = True
    print("  ✗ 这些实验装置与变异表这一轮改了，research/results/ 里却没有一份不比它旧的产物——这一跑多半只留在 /tmp 的草稿里：")  # gate-lint:summary
    for entry in unstored_runs:
        print(f"     {entry}")  # gate-lint:detail
    print(f"     → 怎么办：跑一次把产物存进 {results_dir}/，并在 research/scripts/replay.sh 把这个实验的登记行指到它。")
    print("               这一跑已经跑过、产物还在 /tmp 的草稿目录里，就现在拷进来（会话一重启那个目录就没了）；")
    print("               只改了 // 注释的装置这一格不判；改了字符串里的路径，照 .claude/rules/path-moves.md 第 4 条连同留存产物一起改，复跑仍逐字相同。")
    print("               登记了准入的实验（.claude/gate.d/stage-inputs.tsv 有 E<号> 那一行）按产物头的输入指纹判：指纹对不上就是产物跟不上今天的输入，")
    print("               重跑一次（装置入口放行并把新指纹打在产物头），修改时刻新旧不作数。")
if fingerprint_errors:
    failed = True
    print("  ✗ 这些实验登记了准入，却算不出输入指纹，判不了产物跟不跟得上：")  # gate-lint:summary
    for entry in fingerprint_errors:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：单跑 python3 research/scripts/admission.py experiment . E<号> 看它报什么；多半是 .claude/gate.d/stage-inputs.tsv 那一行的路径写错，")
    print("               或 cargo -V / rustc -V 跑不出来（bash .claude/scripts/env.sh 报缺什么）。")
if on_request_missing_note:
    failed = True
    print("  ✗ 这些按要求才跑的实验装置改了、没有新产物，实验页也没写明之后没有正式跑：")  # gate-lint:summary
    for entry in on_request_missing_note:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：不用跑（research/on-request-experiments.tsv 里的实验只在用户说跑时才跑）；")
    print("               这一轮在实验页新写一句「装置某日改过（改了什么），之后没有正式跑」，要数的时候等用户说跑；页里早先那句说的是上一次改装置，不算。")
if outside_citations:
    failed = True
    print("  ✗ 这些地方把 /tmp 下的东西当依据引用了——那是会话自己的草稿目录，一重启就没了，三个月后没人核得动：")  # gate-lint:summary
    for entry in outside_citations:
        print(f"     {entry}")  # gate-lint:detail
    print(f"     → 怎么办：把 /tmp 下那份东西拷进仓里该在的位置（报告、提示与判决进 {prompts_dir}/，跑出来的数进 {results_dir}/），")
    print("               引用改成指那一份：在引用它的文件里把 /tmp 路径逐处换成仓内路径。")
    print("               原件已经没了，就写明它没了、把还核得动的那部分落进仓里；只是在说做法（草稿放在哪），把句子里的依据词去掉。")
if timestamp_unjudged:
    print(f"    本次未跑 {len(timestamp_unjudged)} 份要比修改时刻的（链接 worktree 里全部文件的时间戳都是检出那一刻写的，分不出装置与产物谁新；"
          f"判据二与判据一里不看时间戳的照判了）：{'、'.join(timestamp_unjudged)}")
    print("    不带 --staged 在工作区里跑 bash .claude/gate.d/doc-experiments.sh --check evidence-in-repo 才判得了这几份。")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    if not_run_file:
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"doc-experiments.sh 的 evidence-in-repo 格有 {len(timestamp_unjudged)} 份装置与变异表本次未跑"
                         f"（链接 worktree 里修改时刻是检出时写的）：{'、'.join(timestamp_unjudged)}\n")
if failed:
    sys.exit(1)

print(f"  ✓ 产物与依据都落在仓里（判了 {len(changed_sources) - len(timestamp_unjudged)} 份这一轮改过的实验装置与变异表、{len(set(scanned_files))} 份 .md，扫到 {tmp_mentions} 处 /tmp 路径、没有一处是引依据；基准 {base}）")
if not changed_sources:
    print("    没判：这次改动没碰 research/e7-index-bench/src/bin/ 与 research/mutations/，判据一这一半没有对象")
print(f"    没判 {skipped_prompt_files} 份 {prompts_dir} 下这一轮没新写的 .md（登记在 .claude/doc-lint-exclude 的原样保存证据，改不得）")
if comment_only_sources:
    print(f"    没判 {len(comment_only_sources)} 份只改了 // 注释的装置源码（碰不到产物，不要求重跑）：{'、'.join(comment_only_sources)}")
if on_request_not_run:
    print(f"    没要求重跑 {len(on_request_not_run)} 份按要求才跑的实验（装置比产物新，实验页写明了没有正式跑）：{'、'.join(on_request_not_run)}")
if fingerprint_matched:
    print(f"    按指纹判 {len(fingerprint_matched)} 份（登记了准入，产物头的输入指纹与今天的输入相同，修改时刻不作数）：{'、'.join(fingerprint_matched)}")
if fingerprint_absent:
    print(f"    按修改时刻判 {len(fingerprint_absent)} 份登记了准入、research/results/ 里却还没有一份带指纹产物的：{'、'.join(fingerprint_absent)}")
PY
}

# ── 格 decision-links：决策与实验双向登记，实验或结论变了之后回看过决策
#
# 规则在 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。判据：
#   ① 实验页（待回填清单之外的）有「### 影响的决策」一节，表头 | 决策分项 | 关系 | 回看 |；
#      实验正文（历史版本与这一节之外）提到的每条决策（`D<号>（`）都要有一行。
#   ② 表里每一行：决策真实存在，写了分项的那条分项也存在；关系是 支撑 / 推翻 / 备料 / 不影响 之一；
#      回看格以日期开头，后跟「改了」或「不受影响：理由」。
#   ③ 双向：关系是支撑或推翻的那条分项，它的「**依据**」段要引回这个实验；反过来，
#      分项「**依据**」段引的每个实验，那个实验页里要有这一行、关系是支撑或推翻。待回填清单里的决策不判这一条。
#   ④ 时效（全量）：实验最近一次变动的日期——页内历史节带日期的 ### 标题、experiments-history.md 里标题
#      （标题没点名实验时看条目正文）点名它的条目——晚于表里某一行的回看日期，判红。
#   ⑤ 按这次改动（这一道开头取的那一份：GATE_BASE / @{upstream} 的 merge-base / HEAD 起，工作区、暂存区、未跟踪文件都算）：
#      实验页正文（历史版本与影响的决策两节之外）改了、或 research/results/e<号>… 的产物变了，
#      这次改动要在那张表里改过至少一行；新加的三方判决 research/prompts/*-main-verification.md
#      要有「## 回看决策」一节（同样的表，或一行「不涉及决策：理由」）；这次改动里新写成「改了」的行，
#      它指的决策文件要在这次改动里。这次改动把实验的标题状态改成「已跑」（状态段恰好是这两个字）时，
#      决策正文（历史版本之前）引了它的每条决策（`E<号>（`，粗体、反引号、空格、半角括号的写法也认）都要回看：
#      这次改动在那份决策里新增或改写的行点到了它，或者表里那条决策的一行改过。搬了家的页按实验号找基准那一版的标题。
#   ⑥ 待回填清单 .claude/decision-links-pending：一行一个 E<号> 或 D<号>，# 后写理由；指到的要存在；
#      只减不增——比基准那一版多出来的行判红；基准里还没有这张清单时整张都算新加，git 取不到基准那一版判红。
#   认不出标题的实验页（首个二级标题不是「## E<号> 」）与决策文件（不是「## D<号> 」）判红，不静默跳过；
#   ⑤ 取某一页的新增行时 git diff 失败，判红，不当成那一页没改。
#   ⑦ 瘦身形态（待回填清单之外的决策）：首行 `## D<号> 简称 —— 状态` 后面不带括注；分项标题不带日期；
#      每个已定项有「**定案**：」「**射程**：」「**依据**：」「**欠**：」四块；索引表的定案格不带日期、不超过 100 字。
#   ⑧ 有实验才有决策（用户 2026-09-19）：已定项的依据段至少引一个实验，或写「无实验：理由」（理由至少八个字）；
#      成功行报出写「无实验」的已定项有几个。
#   ⑨ 实验必须对应决策（用户 2026-09-19）：待回填清单之外的实验页，表里至少一行关系是支撑 / 推翻 / 备料；
#      标题里写着「结论……作废」或「结论……退役」的实验不判（它们留着是为了记下那条路不通）。
#
# 为什么：2026-09-19 用户指出决策知识腐烂、分项膨胀、决策与实验关联不够，要求「做实验或者有结论后要回去看决策
# 是不是受到了影响」。当天现量：154 个实验页里 296 对「实验引了决策、决策没引回来」，决策里没有一处统一的依据栏；
# 一个实验重跑、结论变了之后，没有任何东西逼人回去看它撑着的那几条决策。
#
# 管不到的：回看写的理由对不对、关系判得对不对（支撑写成不影响也过得了③以外的判据），靠人与抽查；
# 实验正文只用编号不用「D<号>（」形态提到的决策，①看不见。
#
# 只删与所属标题相同的行内日期、只把同一天的第二个小节并进第一个的改动，不算「正文改了」（判法借 research/scripts/doc-consolidate.py，
# 两版整理后的正文按行排序逐字相同就是），样本 decision-links-formatting-green 判绿、成功行报「不算正文改了 1 份」。
# 样本：decision-links-red 在一个临时仓里把上面每一条各犯一次（want 逐条点名）；decision-links-green 放瘦身的与没瘦身的决策、
# 待回填清单、结论作废的实验、这次改动改了正文与产物又回看了表、新判决带「## 回看决策」、搬了家的实验页，必须判绿并报对数。
cell_decision_links() {
  if [[ ! -d .claude/kb/experiments || ! -d .claude/kb/decisions ]]; then
    echo "  ✗ 找不到 .claude/kb/experiments 或 .claude/kb/decisions"
    echo "  → 怎么办：kb 挪了位置就同步改这一格里的路径。"
    exit 1
  fi
  base="HEAD"
  in_git=0
  changed_list=/dev/null
  if ((in_git_work_tree && head_born)); then
    in_git=1
    # 改动范围与 doc-process-records.sh 用改动范围的几格、evidence-in-repo 格、doc-registries.sh 的 invariant-anchors 那一格同一份取法：research/scripts/changed-paths.sh 的 gate 取法，未跟踪也算
    if ((! changed_paths_library_loaded)); then
      echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
      echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"
      exit 1
    fi
    base="$shared_base"
    if ((! changed_paths_ok)); then
      echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
      echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
      exit 1
    fi
    changed_list="$SHARED_CHANGED_PATHS"
  fi
  # 只删同日日期、只并同日小节的改动不算「正文改了」：判法借 research/scripts/doc-consolidate.py（门禁 doc-text 的 date-once 格用同一份）
  DOC_CONSOLIDATE="$(dirname "$LIB_CHANGED_PATHS")/doc-consolidate.py"
  export DOC_CONSOLIDATE
  python3 - "$ROOT" "$base" "$in_git" "$changed_list" <<'PY'
import glob, importlib.util, os, re, subprocess, sys

root, base, in_git = sys.argv[1], sys.argv[2], sys.argv[3] == '1'
changed = {line for line in open(sys.argv[4], encoding='utf-8').read().split('\n') if line}
kb = os.path.join(root, '.claude', 'kb')
RELATIONS = ('支撑', '推翻', '备料', '不影响')
BASIS_RELATIONS = ('支撑', '推翻')
E_REF = re.compile(r'(?<![A-Za-z0-9])E(\d+)（')
D_REF = re.compile(r'(?<![A-Za-z0-9])D(\d+)（')
# 「决策正文引了它」按 doc-lint 认引用的写法认：粗体、反引号、编号与括号之间有空格、半角括号都算（第三轮三方 T1-b）
MENTION_REF = re.compile(r'(?<![A-Za-z0-9._-])[*`]*E(\d+)[*`]*\s*[（(]')
TABLE_HEAD = re.compile(r'^\|\s*决策分项\s*\|\s*关系\s*\|\s*回看\s*\|\s*$')
REVIEW = re.compile(r'^(20\d\d-\d\d-\d\d)\s*(改了|不受影响：\s*\S.{3,})')
DATE_HEAD = re.compile(r'^### (20\d\d-\d\d-\d\d)')


def read(path):
    return open(path, encoding='utf-8').read()


def history_start(lines):
    for index, line in enumerate(lines):
        if re.match(r'^## 历史版本\s*$', line):
            return index
    return len(lines)


def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], cwd=root, capture_output=True, text=True)


def split_cells(line):
    return [cell.strip() for cell in re.sub(r'\\\|', '\x00', line.strip()).strip('|').split('|')]


def parse_decision_ref(cell):
    """「D28（挂载期承诺量） 已定项 2」→ (28, ('已定项', 2))；只写决策 → (28, None)；认不出 → None。"""
    head = re.match(r'^\**D(\d+)（', cell)
    if not head:
        return None
    item = re.search(r'(已定项|未定项)\s*(\d+)\s*\**$', cell)
    return int(head.group(1)), ((item.group(1), int(item.group(2))) if item else None)


def section_range(lines, heading_pattern, stop_level):
    """标题行号与它那一节的结束行号（下一个级别 ≤ stop_level 的标题之前），找不到返回 None。"""
    for index, line in enumerate(lines):
        if re.match(heading_pattern, line):
            for end in range(index + 1, len(lines)):
                level = re.match(r'^(#{1,6}) ', lines[end])
                if level and len(level.group(1)) <= stop_level:
                    return index, end
            return index, len(lines)
    return None


def table_rows(lines, start, end):
    """一节里那张影响的决策表的行：[(行号, 单元格)]；没有表头返回 None。"""
    rows, seen_head = [], False
    for index in range(start + 1, end):
        line = lines[index]
        if TABLE_HEAD.match(line.strip()):
            seen_head = True
            continue
        if seen_head and re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', line.strip()):
            continue
        if seen_head and line.strip().startswith('|'):
            rows.append((index, split_cells(line)))
        elif seen_head:
            break
    return rows if seen_head else None


problems = []   # (类别, 说明)


def bad(kind, text):
    if (kind, text) not in problems:   # 同一份文件被几处调到时（git 失败那一类）只报一次
        problems.append((kind, text))


# ── 决策：分项与各分项的依据段 ──
decisions = {}
for path in sorted(glob.glob(os.path.join(kb, 'decisions', '*.md'))):
    lines = read(path).split('\n')
    body = lines[:history_start(lines)]
    head = next((re.match(r'^## D(\d+) ', line) for line in body if re.match(r'^## D(\d+) ', line)), None)
    if not head:
        # 认不出标题的决策文件不许静默跳过：跳过就不计数、不列名，这份决策的分项与依据一条都没判
        bad('认不出标题', f'{os.path.relpath(path, root)}：「## 历史版本」之前没有一行写成「## D<号> 简称 —— 状态」，这份决策一条都没判')
        continue
    number = int(head.group(1))
    items, basis, shapes, index_cells = set(), {}, [], []
    for index, line in enumerate(body):
        item = re.match(r'^(#{3,4}) (已定项|未定项) (\d+)', line)
        if not item:
            continue
        key = (item.group(2), int(item.group(3)))
        items.add(key)
        cited, in_basis, labels, basis_text = set(), False, set(), ''
        for inner in body[index + 1:]:
            level = re.match(r'^(#{1,6}) ', inner)
            if level and len(level.group(1)) <= len(item.group(1)):
                break
            label = re.match(r'^\*\*(定案|射程|依据|欠)\*\*：', inner)
            if label:
                labels.add(label.group(1))
            if inner.startswith('**依据**'):
                in_basis = True
            elif re.match(r'^\*\*[^*]{1,12}\*\*：', inner) or level:
                in_basis = False
            if in_basis:
                cited |= {int(e) for e in E_REF.findall(inner)}
                basis_text += inner + '\n'
        basis[key] = cited
        shapes.append((key, line, labels, cited, basis_text))
    for section in ('已定项', '未定项'):
        found = section_range(body, r'^### %s\s*$' % section, 3)
        if found:
            for line in body[found[0] + 1:found[1]]:
                if line.startswith('#### '):
                    break
                row = re.match(r'^\|\s*(\d+)\s*\|', line)
                if row:
                    items.add((section, int(row.group(1))))
                    index_cells.append((section, int(row.group(1)), split_cells(line)[-1]))
                # 没瘦身的决策有的用编号列表列分项（`9. **名字**…`），doc-decisions.sh 数分项时也认这种写法
                listed = re.match(r'^(\d+)\.\s', line)
                if listed:
                    items.add((section, int(listed.group(1))))
    title = next(line for line in body if re.match(r'^## D(\d+) ', line))
    decisions[number] = {'path': os.path.relpath(path, root), 'items': items, 'basis': basis,
                         'shapes': shapes, 'index_cells': index_cells, 'title': title,
                         'mentions': {int(e) for e in MENTION_REF.findall('\n'.join(body))}}

# ── 待回填清单 ──
pending_path = os.path.join(root, '.claude', 'decision-links-pending')
pending_rel = '.claude/decision-links-pending'
pending = set()
pending_lines = read(pending_path).split('\n') if os.path.exists(pending_path) else []
for number, line in enumerate(pending_lines, 1):
    if not line.strip() or line.lstrip().startswith('#'):
        continue
    entry = re.match(r'^(E|D)(\d+)\s+#\s*(\S.{3,})$', line.strip())
    if not entry:
        bad('待回填', f'{pending_rel}:{number}：写成「E<号>  # 理由」或「D<号>  # 理由」，理由至少四个字——「{line.strip()[:30]}」')
        continue
    pending.add((entry.group(1), int(entry.group(2))))
if in_git and os.path.exists(pending_path):
    # 「基准里还没有这张清单」与「git 取不到基准」要分开：前者整张都算新加，后者判不了只减不增，两种都不许静默放过
    listed = git('ls-tree', '--name-only', base, '--', pending_rel)
    before, base_note = None, ''
    if listed.returncode != 0:
        bad('待回填', f'取不到基准 {base} 里的 {pending_rel}（git ls-tree 退 {listed.returncode}：{listed.stderr.strip()[:80]}），「只减不增」这一条没判')
    elif not listed.stdout.strip():
        before, base_note = set(), f'（基准 {base} 里还没有这张清单，这次改动新建了它，整张都算新加）'
    else:
        previous = git('show', f'{base}:{pending_rel}')
        if previous.returncode != 0:
            bad('待回填', f'读不出基准 {base} 里的 {pending_rel}（git show 退 {previous.returncode}：{previous.stderr.strip()[:80]}），「只减不增」这一条没判')
        else:
            before = set()
            for line in previous.stdout.split('\n'):
                entry = re.match(r'^(E|D)(\d+)\s', line.strip())
                if entry:
                    before.add((entry.group(1), int(entry.group(2))))
    if before is not None:
        for kind, number in sorted(pending - before):
            bad('待回填', f'{pending_rel} 新加了 {kind}{number}：清单只减不增，新写的实验页与决策当场登记，不进清单{base_note}')

# ── 瘦身形态与「有实验才有决策」（待回填清单之外的决策） ──
DATE = re.compile(r'20\d\d-\d\d-\d\d')
slim_items = no_experiment_items = 0
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    where = info['path']
    # 半定 / 待定的决策，doc-decisions.sh 要标题里写明未定几项（`—— 半定（一项未定）`），只许这一种括注
    if not re.match(r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二两三四五六七八九十]+\s*[项条]未定）)?\s*$', info['title']):
        bad('瘦身形态', f'{where}：首行写成「## D{decision} 简称 —— 状态」，状态后面除了半定 / 待定要写的「（N 项未定）」不带别的括注——「{info["title"][:40]}」')
    for (kind, item_number), heading, labels, cited, basis_text in info['shapes']:
        if DATE.search(heading):
            bad('瘦身形态', f'{where}：D{decision} {kind} {item_number} 的标题带日期，日期与来历进变更史')
        if kind != '已定项':
            continue
        slim_items += 1
        for label in ('定案', '射程', '依据', '欠'):
            if label not in labels:
                bad('瘦身形态', f'{where}：D{decision} 已定项 {item_number} 缺「**{label}**：」一块')
        if '依据' in labels and not cited:
            if re.search(r'无实验：\s*\S.{7,}', basis_text):
                no_experiment_items += 1
            else:
                bad('没有实验', f'{where}：D{decision} 已定项 {item_number} 的依据一个实验都没引——有实验才有决策；实在没有可量的，写「无实验：理由」')
    for kind, item_number, cell in info['index_cells']:
        # 「**状态：已定。**」是 doc-decisions.sh 要的规范标记（检索端出来的是这一行），不算在一句话的字数里；
        # doc-decisions.sh 的 blocking-verdict 格要写在未定项登记行里的两句规范标记（「改新池新建文件的字节：…」与「动不动格式：…」）同理不算——
        # blocking-verdict 格明写它要在登记行里（写在表下那一段里它定位不到），而它必须带日期，与这一条的「不带日期、
        # 不超过 100 字」直接相撞：2026-09-20 给 D2 未定项 21 与 D26 未定项 8 / 9 / 10 补上字节判决之后四行一起红。
        # 两句都剥：blocking-verdict 格是两把尺，一句量「这一版写不写出不同字节」、一句量「将来动不动格式」，
        # 缺哪一句它都判红，而两句都必须带日期 ⇒ 两句都不算进这一条的「不带日期、不超过 100 字」。
        without_marks = cell
        for mark in ('改新池新建文件的字节', '动不动格式'):
            without_marks = re.sub(mark + r'：[^（(]*[（(][^）)]*[）)]', '', without_marks)
        plain = re.sub(r'\*\*|`', '', re.sub(r'\*\*状态：[^*]*\*\*', '', without_marks)).strip()
        if DATE.search(plain) or len(plain) > 100:
            bad('瘦身形态', f'{where}：D{decision} {kind}索引表第 {item_number} 行的定案格要一句话、不带日期、不超过 100 字（现在 {len(plain)} 字）')

# ── 实验最近一次变动的日期：页内历史节 + experiments-history.md ──
central_dates = {}
central_path = os.path.join(kb, 'experiments-history.md')
if os.path.exists(central_path):
    lines = read(central_path).split('\n')
    start = history_start(lines)
    current_date, heading_refs, body_refs = None, set(), set()

    def flush():
        refs = heading_refs or body_refs
        for experiment in refs:
            if current_date and current_date > central_dates.get(experiment, ''):
                central_dates[experiment] = current_date

    for line in lines[start:]:
        dated = DATE_HEAD.match(line)
        if dated or line.startswith('### ') or line.startswith('## '):
            flush()
            current_date = dated.group(1) if dated else None
            heading_refs = {int(e) for e in E_REF.findall(line)} if dated else set()
            body_refs = set()
            continue
        body_refs |= {int(e) for e in E_REF.findall(line)}
    flush()

# ── 实验页 ──
experiments, rows_total = {}, 0
for path in sorted(glob.glob(os.path.join(kb, 'experiments', '*.md'))):
    rel = os.path.relpath(path, root)
    lines = read(path).split('\n')
    head_line = next((line for line in lines if re.match(r'^## E(\d+) ', line)), None)
    if not head_line:
        # 认不出标题的实验页不许静默跳过：跳过就不计数、不列名，它的影响的决策表一行都没判
        bad('认不出标题', f'{rel}：没有一行写成「## E<号> 简称 —— 状态」（编号后面要跟空格），这一页一条都没判')
        continue
    number = int(re.match(r'^## E(\d+) ', head_line).group(1))
    # 只认「结论……作废 / 退役」：标题里别处出现这两个词（「上界 A 作废」「容器退役记录」）的实验结论照样成立，照判（第三轮三方 T1-f）
    voided = re.search(r'结论[^，。：]{0,3}(作废|退役)', head_line) is not None
    hist = history_start(lines)
    section = section_range(lines[:hist], r'^### 影响的决策\s*$', 3)
    rows = table_rows(lines, section[0], section[1]) if section else None
    outside = [line for index, line in enumerate(lines[:hist]) if not (section and section[0] <= index < section[1])]
    mentioned = {int(d) for d in D_REF.findall('\n'.join(outside))}
    local_dates = [m.group(1) for m in (DATE_HEAD.match(line) for line in lines[hist:]) if m]
    latest = max(local_dates + ([central_dates[number]] if number in central_dates else []), default='')
    is_pending = ('E', number) in pending
    experiments[number] = {'rel': rel, 'lines': lines, 'hist': hist, 'section': section, 'rows': rows or []}
    if rows is None and not is_pending:
        bad('没有表', f'{rel}：E{number} 没有「### 影响的决策」表（表头 | 决策分项 | 关系 | 回看 |）')
        continue
    covered = set()
    for index, cells in rows or []:
        rows_total += 1
        where = f'{rel}:{index + 1}'
        if len(cells) != 3:
            bad('行形状', f'{where}：影响的决策表一行要三格，这一行 {len(cells)} 格')
            continue
        ref = parse_decision_ref(cells[0])
        if not ref:
            bad('行形状', f'{where}：第一格要写「D<号>（简称）」或「D<号>（简称） 已定项 k」——「{cells[0][:30]}」')
            continue
        decision, item = ref
        covered.add(decision)
        if decision not in decisions:
            bad('指不到', f'{where}：D{decision} 在 decisions/ 下没有正文')
            continue
        if item and item not in decisions[decision]['items']:
            bad('指不到', f'{where}：D{decision} 没有{item[0]} {item[1]}')
        relation = cells[1].strip('* ')
        if relation not in RELATIONS:
            bad('行形状', f'{where}：关系写「{relation[:12]}」，只认 支撑 / 推翻 / 备料 / 不影响')
            continue
        if relation in BASIS_RELATIONS and item is None and decisions[decision]['items']:
            bad('行形状', f'{where}：关系是{relation}就要写到分项（D{decision} 有分项）')
        review = REVIEW.match(cells[2].strip('* '))
        if not review:
            bad('行形状', f'{where}：回看格写「YYYY-MM-DD 改了」或「YYYY-MM-DD 不受影响：理由」——「{cells[2][:30]}」')
            continue
        if latest and review.group(1) < latest:
            bad('回看过期', f'{where}：E{number} 最近一次变动在 {latest}，对 D{decision} 的回看停在 {review.group(1)}')
        if relation in BASIS_RELATIONS and item and ('D', decision) not in pending \
                and number not in decisions[decision]['basis'].get(item, set()):
            bad('不对称', f'{where}：E{number} 说它{relation} D{decision} {item[0]} {item[1]}，而那条分项的「**依据**」段没引 E{number}')
    if not is_pending and not voided and rows is not None \
            and not any(len(cells) == 3 and cells[1].strip('* ') in ('支撑', '推翻', '备料') for _, cells in rows):
        bad('没对应决策', f'{rel}：E{number} 的影响的决策表里没有一行是支撑 / 推翻 / 备料——实验必须对应决策（结论作废或退役的实验在标题里写明，就不判这一条）')
    if not is_pending:
        for decision in sorted(mentioned - covered):
            bad('没登记', f'{rel}：E{number} 正文提到 D{decision}，影响的决策表里没有它那一行')

# ── 决策这一侧：依据引的实验，实验页里要有支撑或推翻那一行 ──
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    for item, cited in sorted(info['basis'].items()):
        for experiment in sorted(cited):
            page = experiments.get(experiment)
            if not page:
                bad('指不到', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，kb 里没有这个实验页")
                continue
            ok = any(parse_decision_ref(cells[0]) == (decision, item) and cells[1].strip('* ') in BASIS_RELATIONS
                     for _, cells in page['rows'] if len(cells) == 3)
            if not ok:
                bad('不对称', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，{page['rel']} 的影响的决策表里没有「支撑 / 推翻」这一行")
for kind, number in sorted(pending):
    exists = number in (experiments if kind == 'E' else decisions)
    if not exists:
        bad('待回填', f'{pending_rel} 里的 {kind}{number} 在 kb 里没有正文')

# ── 按这次改动 ──
triggered = 0
formatting_only = 0
became_ran_checked = 0


def added_lines(rel):
    """这次改动里 rel 新加的行：[(新文件里的行号, 内容)]；未跟踪的文件整份都算。"""
    if git('ls-files', '--error-unmatch', rel).returncode != 0:
        return [(index, line) for index, line in enumerate(read(os.path.join(root, rel)).split('\n'))]
    shown = git('diff', '--no-renames', '--no-color', '--no-ext-diff', '-U0', base, '--', rel)
    if shown.returncode != 0:
        # git 失败时取不到新增行：当成「一行没改」就会把 ⑤ 整条放过去
        bad('取不到改动', f'{rel}：git diff 相对基准 {base} 失败（退 {shown.returncode}：{shown.stderr.strip()[:80]}），这一页新增了哪些行判不了，⑤ 对它没判')
        return []
    diff = shown.stdout
    result, new_line, in_header = [], 0, True
    for line in diff.split('\n'):
        hunk = re.match(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@', line)
        if hunk:
            in_header = False
            new_line = int(hunk.group(1)) - 1
            count = int(hunk.group(2)) if hunk.group(2) is not None else 1
            if count == 0:
                result.append((new_line, None))   # 纯删除：记它在新文件里的位置
            continue
        # 「+++ 」只在第一个 @@ 之前是文件头：正文里以「++ 」开头的一行新增在 diff 里也长成「+++ 」
        if line.startswith('+') and not in_header:
            result.append((new_line, line[1:]))
            new_line += 1
    return result


def ran_status(title_line):
    """标题的状态段（「——」之后、括注之前）恰好是「已跑」：部分已跑、已跑但作废都不算。"""
    status = re.match(r'^## E\d+ .*?——\s*(.*)$', title_line or '')
    return bool(status) and re.sub(r'（.*$', '', status.group(1)).strip() == '已跑'


def base_title(rel, number=None):
    """基准那一版的标题行；基准里没有这一页返回 None。"""
    shown = git('show', f'{base}:{rel}')
    # 基准里这个路径没有页（这一批搬了家）：按实验号在基准那棵树里找它原来那页的标题，搬家不算「改成已跑」
    if shown.returncode != 0 and number is not None:
        found = git('grep', '-h', '-m1', '-E', f'^## E{number} ', base, '--', '.claude/kb/experiments/')
        hit = next((l[l.find('## E'):] for l in found.stdout.split('\n') if '## E' in l), None)
        return hit if found.returncode == 0 else None
    if shown.returncode != 0:
        return None
    return next((line for line in shown.stdout.split('\n') if re.match(r'^## E\d+ ', line)), None)


_consolidate = None


def consolidate_module():
    """research/scripts/doc-consolidate.py：读不到就当没有这一层判法（改动照样算正文改了）。"""
    global _consolidate
    if _consolidate is None:
        path = os.environ.get('DOC_CONSOLIDATE', '')
        if not os.path.isfile(path):
            _consolidate = False
        else:
            spec = importlib.util.spec_from_file_location('doc_consolidate', path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            _consolidate = module
    return _consolidate


def normalized_body_lines(text):
    """整理之后的正文行（去空白与列表、标题、表格的符号）按行排序：两版一样就是只删了同日日期、只并了同日小节。"""
    module = consolidate_module()
    body, _hist = module.split_history(text.split('\n'))
    migrated, _report = module.migrate_document('\n'.join(body), '', drop_history=False)
    return sorted(stripped for stripped in (re.sub(r'[\s#*|>\-]', '', line) for line in migrated.split('\n')) if stripped)


def formatting_only_change(rel):
    """这次对实验页的改动是不是只删了与所属标题相同的日期、只把同一天的第二个小节并进第一个。"""
    if not consolidate_module():
        return False
    shown = git('show', f'{base}:{rel}')
    if shown.returncode != 0:
        return False
    try:
        return normalized_body_lines(shown.stdout) == normalized_body_lines(read(os.path.join(root, rel)))
    except Exception:  # noqa: BLE001 —— 判法自己出错时按「正文改了」判，不放过
        return False


def check_changed_rows(rel, row_lines):
    for text in row_lines:
        cells = split_cells(text)
        if len(cells) != 3:
            continue
        ref = parse_decision_ref(cells[0])
        review = REVIEW.match(cells[2].strip('* '))
        if ref and review and review.group(2) == '改了' and ref[0] in decisions \
                and decisions[ref[0]]['path'] not in changed:
            bad('改了没改', f"{rel}：这次写了「改了」D{ref[0]}，而 {decisions[ref[0]]['path']} 不在这次改动里")


if in_git:
    result_hits = {}
    for rel in changed:
        match = re.match(r'^research/results/e(\d+)[^0-9]', rel)
        if match:
            result_hits.setdefault(int(match.group(1)), rel)
    for number, page in sorted(experiments.items()):
        rel = page['rel']
        body_changed = False
        added = added_lines(rel) if rel in changed and os.path.exists(os.path.join(root, rel)) else []
        section = page['section']
        for line_number, _ in added:
            inside_table = section and section[0] <= line_number < section[1]
            if line_number < page['hist'] and not inside_table:
                body_changed = True
        if body_changed and formatting_only_change(rel):
            body_changed = False
            formatting_only += 1
        table_touched = [text for line_number, text in added
                         if text is not None and section and section[0] <= line_number < section[1]
                         and text.strip().startswith('|') and not TABLE_HEAD.match(text.strip())
                         and not re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', text.strip())]
        check_changed_rows(rel, table_touched)
        reasons = []
        if body_changed:
            reasons.append('正文改了')
        if number in result_hits:
            reasons.append(f'产物 {result_hits[number]} 变了')
        if reasons:
            triggered += 1
            if not table_touched:
                bad('没回看', f"{rel}：这次改动里 E{number} {'、'.join(reasons)}，影响的决策表一行都没回看")
        # ⑤ 的第二半：这一批把实验改成已跑，正文里引了它的每条决策都要回看——决策文件在这一批里，
        # 或者影响表里那条决策的一行在这一批里改过。只看「表里任一行被改过」放不过「决策在欠账块里等它、表里却没它」那一种。
        title_line = next((line for line in page['lines'] if re.match(r'^## E\d+ ', line)), None)
        if body_changed and ran_status(title_line) and not ran_status(base_title(rel, number)):
            reviewed = {ref[0] for ref in (parse_decision_ref(split_cells(text)[0]) for text in table_touched) if ref}
            for decision, info in sorted(decisions.items()):
                if number not in info['mentions'] or ('D', decision) in pending:
                    continue
                became_ran_checked += 1
                # 决策文件在这一批里还不够：要这一批在那份决策里新增或改写的行点到了这个实验号（改一处别的不算回看）
                touched_mention = info['path'] in changed and any(
                    text and re.search(r'(?<![A-Za-z0-9])E%d(?![0-9])' % number, text) for _, text in added_lines(info['path']))
                if not touched_mention and decision not in reviewed:
                    bad('没回看引用它的决策', f"{rel}：这次改动把 E{number} 改成已跑，{info['path']} 的正文引了它，"
                        f"而这次改动在那份决策里没有一行点到它、影响的决策表里 D{decision} 那一行也没回看")
    for rel in sorted(changed):
        if not re.match(r'^research/prompts/[^/]+-main-verification\.md$', rel) or not os.path.exists(os.path.join(root, rel)):
            continue
        if git('cat-file', '-e', f'{base}:{rel}').returncode == 0:
            continue
        triggered += 1
        lines = read(os.path.join(root, rel)).split('\n')
        section = section_range(lines, r'^## 回看决策\s*$', 2)
        if not section:
            bad('没回看', f'{rel}：新写的三方判决没有「## 回看决策」一节')
            continue
        rows = table_rows(lines, section[0], section[1])
        waived = any(re.match(r'^不涉及决策：\s*\S.{3,}', line.strip()) for line in lines[section[0] + 1:section[1]])
        if not rows and not waived:
            bad('没回看', f'{rel}：「## 回看决策」里既没有表行，也没有一行「不涉及决策：理由」')
            continue
        for index, cells in rows or []:
            ref = parse_decision_ref(cells[0]) if len(cells) == 3 else None
            if not ref or not REVIEW.match(cells[2].strip('* ')):
                bad('行形状', f'{rel}:{index + 1}：回看决策表一行写成 | D<号>（简称） [已定项 k] | 关系 | YYYY-MM-DD 改了 / 不受影响：理由 |')
        check_changed_rows(rel, [lines[index] for index, _ in rows or []])

pending_experiments = sum(1 for kind, _ in pending if kind == 'E')
pending_decisions = sum(1 for kind, _ in pending if kind == 'D')
summary = (f'实验页 {len(experiments)} 个（待回填 {pending_experiments}）、决策 {len(decisions)} 条（待回填 {pending_decisions}）、'
           f'已瘦身决策的已定项 {slim_items} 个（其中写「无实验」的 {no_experiment_items} 个）、'
           f'表行 {rows_total} 行、这次改动触发回看 {triggered} 处、改成已跑而引了它的决策 {became_ran_checked} 条；'
           f'只删同日日期、只并同日小节的改动 {formatting_only} 份不算正文改了')
if problems:
    kinds = {}
    for kind, _ in problems:
        kinds[kind] = kinds.get(kind, 0) + 1
    print(f"  ✗ 决策与实验的登记有 {len(problems)} 处问题（{'、'.join(f'{k} {v}' for k, v in kinds.items())}）；{summary}：")  # gate-lint:summary
    for kind, text in problems:
        print(f'      [{kind}] {text}')  # gate-lint:detail
    print('  → 怎么办：格式与判据见 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。')
    print('            实验出了新结论、换了产物，就逐行回看它影响的决策，回看格写当天日期与「改了」或「不受影响：理由」；')
    print('            支撑 / 推翻的那条分项在「**依据**」段引回这个实验；待回填清单只能删行，新实验页当场写全这张表。')
    sys.exit(1)
print(f'  ✓ 决策与实验双向登记对得上、回看不过期（查了实验页 {len(experiments)} 个、决策 {len(decisions)} 条；{summary}）')
PY
}

# ── 格 verdict-false-named：实验产物判决行里的 false 字段有没有被实验页点名
#
# 判据：`research/scripts/replay.sh` 登记表里每一行登记的入库产物（路径在 research/results/ 下），
# 逐行找 `E7RESULT name=verdict ` 开头的行（现查过全仓产物：14 份产物里出现的 `name=verdict` 一律
# 是这个字面形式，没有 `name=verdict_candidate` / `_base` / `_unit` 这几种源码里存在、但从未落进已入库
# 产物的变体——这几种不在这一格射程里，出现了这一格也看不见，算已知空白）。
# 判决行里任何 `字段=false`，都要在它对应实验号的实验页里被点名，否则判红。
#
# 实验号 → 实验页：`.claude/kb/experiments/<去掉前导 E 的号>-*.md`（登记表第 1 列去掉前导 `E`，
# 与 kb-discipline.md「编号只能做索引」的登记位是同一份文件名约定）。
#
# 「点名」怎么认（现查过 kb/experiments/*.md 里已有的写法定的）：同一行里字段名**紧跟着**它的值 `false`
# （`字段=false`，字段名与等号、等号与 false 之间可以夹反引号与空白；同一行里别的字段的 false 不算），
# （kb 里一行常常是一整段散文，含标点分隔的若干分句，这里按物理行界定，不切分句），
# 且这一行落在下面两处之一：
#   ① 实验页「## 历史版本」下**最新那一条**历史条目（即 `## 历史版本` 之后第一个 `### ` 到下一个
#      `### `/`## ` 之间的文本）；
#   ② 正文（`## 历史版本` 之前的全部文本）。
# 只认字段名出现在正文的某处不够——正文可能是旧一轮留下的、没跟这一次的 false 一起刷新
# （已知会红的一格 E142 就是这样：正文「判决」一节原样抄着 `control_violations_ok=true` 的旧行，
# 字段名确实在正文里，但那是旧值的旧记录，不是这一次 false 的说明；只有字段名与 `false` 同一行
# 出现才算真的把这次的 false 讲清楚了）。
# 正文里那一行自己标着「第 N 次跑」、而标的次数里没有最新历史条目标题里的那一次，它是上一次跑留下的，不算点名。
# 实验页文件名补零（08-、09-）与不补零两种都认。登记的产物一份都不在 research/results/ 下、或一个 false 字段都没有，
# 本次无对象可判，这一格退 77。
#
# 出路：在实验页那一次跑的历史条目里写明这个字段为什么是 false——是判据如实判出的结论，
# 还是装置或期望值的问题；后者照问题单重跑。
#
# 背景：`records/2026-09-16-subagent拆分提案.md` 第四十节第 33 行。
# 不开豁免：登记产物里的 false 一个都不豁免；今天有几个、点了名的几个，由成功行与汇总行现报，这里不写死。
# 与别的格、别的阶段的分工：results-cited 格判产物文件本身有没有被点名，不看产物内部判决字段的值；
# quoted-result-lines 格判 kb 正文整行抄的 E7RESULT 行是不是逐字见于产物（反方向：产物是不是配得上 kb 已经抄的话），不看某个字段是不是 false；
# checker-tier-research-build-and-replay.sh 的 experiment-replay 格判产物今天复不复现得出来（逐字节比对），不读产物内容里的判决字段。
#
# 弄坏开关：GATE_VERDICT_FALSE_SKIP_NAMED_CHECK=1 时跳过点名检查（把每个 false 字段都当成已点名），
# 只用来证明这道检查真的在起作用——不是提交时能用的旁路。
# 样本：verdict-false-named-red 里三个没点名的 false（一个在正文里只写字段名、一个点名的那一行标着上一次跑、一个根本没提）；
# verdict-false-named-green 里一个在最新历史条目点名、一个实验页文件名补了零；verdict-false-named-zero 的登记表一行都读不到。
cell_verdict_false_named() {
  REPLAY=research/scripts/replay.sh
  [[ -f "$REPLAY" ]] || { echo "  ! 找不到 $REPLAY，这一格无对象可判"; exit 77; }

  python3 - "$REPLAY" <<'PY'
import glob
import os
import re
import sys

replay_path = sys.argv[1]

ROW_RE = re.compile(r'^(E[0-9]+)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$')
rows = []
with open(replay_path, encoding='utf-8') as handle:
    for line in handle:
        line = line.rstrip('\n')
        m = ROW_RE.match(line)
        if m:
            rows.append((m.group(1), m.group(4)))

# 表格格式变了、一行都读不到，判红，不许当成「0 项，通过」——与门禁 checker-tier-research-build-and-replay.sh 的 experiment-replay 格同一条判据
# （show-me-test.md「扫到 0 项也不是通过」）。
if not rows:
    print(f"  ✗ 从 {replay_path} 一行都没读到——登记表的格式变了？")
    print("     → 怎么办：这一格靠 '编号|二进制|参数|产物|exact/timing' 这个形状取行（与门禁 checker-tier-research-build-and-replay.sh 的 experiment-replay 格")
    print("               同一份判据），改表格式要同时改这两处。")
    sys.exit(1)

RESULTS_DIR = 'research/results'
EXP_DIR = '.claude/kb/experiments'
VERDICT_PREFIX = 'E7RESULT name=verdict '
skip_named_check = os.environ.get('GATE_VERDICT_FALSE_SKIP_NAMED_CHECK') == '1'

missing_files = []      # 登记了却在 research/results/ 下找不到文件（多半是已归档，见 checker-tier-research-build-and-replay.sh 复跑里「产物已归档」那一档）
checked_files = 0
verdict_lines = 0
false_fields = []       # (exp, product, lineno, field)

for exp, product in rows:
    path = os.path.join(RESULTS_DIR, product)
    if not os.path.isfile(path):
        missing_files.append((exp, product))
        continue
    checked_files += 1
    with open(path, encoding='utf-8', errors='replace') as handle:
        for lineno, raw in enumerate(handle, 1):
            stripped = raw.strip()
            if not stripped.startswith(VERDICT_PREFIX):
                continue
            verdict_lines += 1
            rest = stripped[len(VERDICT_PREFIX):]
            for token in rest.split():
                if '=' not in token:
                    continue
                field, value = token.split('=', 1)
                if value == 'false':
                    false_fields.append((exp, product, lineno, field))


def experiment_page(exp):
    number = int(exp[1:])
    # 实验页文件名有的补零（08-、09-），有的不补：两种都认
    matches = sorted(set(glob.glob(os.path.join(EXP_DIR, f'{number}-*.md'))
                         + glob.glob(os.path.join(EXP_DIR, f'{number:02d}-*.md'))))
    return matches[0] if matches else None


def split_page(text):
    marker = '\n## 历史版本'
    idx = text.find(marker)
    body = text if idx < 0 else text[:idx]
    if idx < 0:
        return body, ''
    tail = text[idx + len(marker):]
    head_match = re.search(r'^### .*$', tail, re.M)
    if not head_match:
        return body, '', ''
    start = head_match.end()
    next_match = re.search(r'^#{2,3} ', tail[start:], re.M)
    end = start + next_match.start() if next_match else len(tail)
    return body, tail[start:end], head_match.group(0)


RUN_TAG = re.compile(r'第([一二三四五六七八九十百零〇两0-9]+)次跑')


def is_named(field, body, latest, latest_heading):
    """返回 (点没点名, 正文里那一行若是上一次跑留下的就交出它)。字段名要紧跟着它自己的 =false，同一行别的字段的 false 不算。"""
    if skip_named_check:
        return True, None
    pattern = re.compile(r'(?<![A-Za-z0-9_])' + re.escape(field) + r'`?\s*[=:：]\s*`?false(?![A-Za-z0-9_])')
    if any(pattern.search(line) for line in latest.split('\n')):
        return True, None
    latest_tags = set(RUN_TAG.findall(latest_heading))
    stale = None
    for line in body.split('\n'):
        if not pattern.search(line):
            continue
        tags = set(RUN_TAG.findall(line))
        if tags and latest_tags and not (tags & latest_tags):
            stale = stale or line.strip()
            continue
        return True, None
    return False, stale


unnamed = []   # (exp, product, lineno, field, reason)
named_count = 0
page_cache = {}
for exp, product, lineno, field in false_fields:
    if exp not in page_cache:
        page_cache[exp] = experiment_page(exp)
    page = page_cache[exp]
    if page is None:
        unnamed.append((exp, product, lineno, field,
                         f'找不到实验页 {EXP_DIR}/{exp[1:]}-*.md'))
        continue
    with open(page, encoding='utf-8') as handle:
        text = handle.read()
    body, latest, latest_heading = split_page(text)
    named, stale = is_named(field, body, latest, latest_heading)
    if named:
        named_count += 1
    elif stale:
        unnamed.append((exp, product, lineno, field,
                         f'{page} 正文里点到它的那一行标的是别的某次跑（「{stale[:40]}」），'
                         f'最新历史条目是「{latest_heading.lstrip("# ").strip()[:30]}」：那是上一次跑留下的，不算点名这一次的 false'))
    else:
        unnamed.append((exp, product, lineno, field,
                         f'{page} 里既不在最新历史条目、也不在正文里被点名'))

print(f'  · 登记了 {len(rows)} 行产物；{checked_files} 份文件在 {RESULTS_DIR} 下找到、'
      f'{len(missing_files)} 份登记了却找不到（多半已归档）')
if missing_files:
    print(f'     没查到文件的登记行（{len(missing_files)} 个）：')
    for exp, product in missing_files:
        print(f'       {exp}  {RESULTS_DIR}/{product}')  # gate-lint:detail
print(f'  · 查到的文件里共 {verdict_lines} 行判决（name=verdict）；{len(false_fields)} 个字段是 false，'
      f'其中 {named_count} 个已点名、{len(unnamed)} 个没点名')

if unnamed:
    print('  ✗ 这些判决字段是 false，对应实验页没有点名：')
    for exp, product, lineno, field, reason in unnamed:
        print(f'     {exp}  {RESULTS_DIR}/{product}:{lineno}  {field}=false  ({reason})')   # gate-lint:detail
    print('     → 怎么办：在实验页那一次跑的历史条目里写明这个字段为什么是 false——是判据如实判出')
    print('               的结论，还是装置或期望值的问题；后者照问题单重跑，把结论一起写回历史条目。')
    sys.exit(1)

if checked_files == 0:
    print(f'  ⊘ 本次无对象可判：登记的 {len(rows)} 行产物一份都不在 {RESULTS_DIR} 下（上面逐个列了）')
    sys.exit(77)
if not false_fields:
    print(f'  ⊘ 本次无对象可判：查到的 {checked_files} 份产物里 {verdict_lines} 行判决，没有一个字段是 false')
    sys.exit(77)
print(f'  ✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 {named_count} 个）')
PY
}

# ── 格 repro-command：跑过的实验有没有留下复跑命令
#
# 判据：一个实验正文若点名了 `research/results/` 里的产物，就必须同时写出**怎么把它跑出来**——
# 一条 `cargo run --release --bin X`，或者点名复跑入口 `research/scripts/replay.sh` / `research/scripts/vm-bench.sh`。
# 提到别的 `research/scripts/*.sh`（`mutate.sh` 这类）不算写了复跑命令：它们跑不出那份产物。
#
# ⚠️ **这条是实测出来的**：2026-08-29 的复跑轮里，E9（key 编码对遍历局部性的影响）的入库产物是
# **25 次运行拼起来的**（5 种子 × 5 改名档），而那个循环一个字都没写进 kb。
# 复跑的人只能从产物的 config 行反推参数——反推对了才发现它本来就复跑得出来。
# 「产物在」和「产物跑得出来」是两件事，results-cited 格只查了前一件。
#
# 例外：正文写明「原始输出未留存」的（要真设备 / 虚机的那类），这一格不管，成功行逐个列名。
# 读不了的页判红（grep 出错与「没点产物」不是一回事）；一页都没点产物，本次无对象可判，这一格退 77。
# 实验页目录里一页 .md 都没有也是无对象可判、退 77（原来那一道把没展开的通配 `*.md` 当成一页读不了的页判红，2026-09-28 并进来时改掉）。
# 样本：repro-command-red 里一页点了产物却只提到变异脚本；repro-command-green 里一页写了 cargo run、一页写明原始输出未留存。
cell_repro_command() {
  EXP_DIR=.claude/kb/experiments
  [[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，这一格跳过"; exit 77; }
  PRODUCT_RE='research/results/[A-Za-z0-9._-]+\.out'
  REPRO_RE='cargo run --release --bin |research/scripts/(replay|vm-bench)\.sh'

  bad=(); unreadable=(); exempt=(); pointing=0
  for f in "$EXP_DIR"/*.md; do
    # 一页 .md 都没有时通配原样留下，grep 找不到它会退 2、被当成「读不了的页」：那是无对象可判，不是读不了
    [[ -e "$f" || -L "$f" ]] || continue
    grep -qE "$PRODUCT_RE" "$f"; rc=$?
    if ((rc == 2)); then unreadable+=("$(basename "$f")"); continue; fi
    ((rc == 0)) || continue
    pointing=$((pointing + 1))
    if grep -qF '原始输出未留存' "$f"; then exempt+=("$(basename "$f")"); continue; fi
    grep -qE "$REPRO_RE" "$f" && continue
    bad+=("$(basename "$f")")
  done

  if ((${#unreadable[@]})); then
    echo "  ✗ 这些实验页读不了，点没点产物、写没写复跑命令都没判："
    printf '      %s\n' "${unreadable[@]}"
    echo "  → 怎么办：按 grep 的报错修好（权限、编码），再跑；读不了的页不能当成没点产物放过去。"
    exit 1
  fi

  if ((${#bad[@]})); then
    echo "  ✗ 这些实验点了产物却没写复跑命令，读的人只能靠反推："
    printf '      %s\n' "${bad[@]}"
    echo "  → 怎么办：在正文的口径一节补一句，格式与别处一致——"
    echo "    代码 \`research/e7-index-bench/src/bin/eNN_xxx.rs\`（\`cargo run --release --bin eNN-xxx\`），"
    echo "    原始输出 \`research/results/eNN-xxx-YYYY-MM-DD.out\`。"
    echo "    多次运行拼起来的产物，要把那个循环也写出来（种子、参数各扫了哪些值）。"
    echo "    要真设备 / 虚机因而没留产物的，正文写明「原始输出未留存」，这一格就不管它。"
    echo "    复跑入口只认 cargo run --release --bin 与 research/scripts/replay.sh、research/scripts/vm-bench.sh；提到 mutate.sh 这类别的脚本不算。"
    exit 1
  fi
  if ((pointing == 0)); then
    echo "  ⊘ 本次无对象可判：$EXP_DIR 下没有一页点到 research/results/ 里的产物"
    exit 77
  fi
  echo "  ✓ 点了产物的实验都写了复跑命令（点了产物的 ${pointing} 页里判了 $((pointing - ${#exempt[@]})) 页）"
  echo "    写明「原始输出未留存」不判的 ${#exempt[@]} 页：${exempt[*]:-（没有）}"
}

# ── 格 experiment-orphans：research 里的实验号在 kb 里有没有正文
#
# 判据：`research/` 下（逐层往下扫，`target/` 编译目录不扫）凡是以 `eNN` 命名的东西（提示、产物、源码、变异表、数据），
# 以及住在 `crates/singlefs-checker-tier/src/bin/` 下的实验装置（`eNN_*.rs`），
# `kb/experiments/` 里就必须有对应编号的正文文件。没有 = 干了活但没入库，
# 而**跑过的东西没入库比没跑更危险**——它会以「我们量过」的形式活在对话里，谁也复核不了。
#
# ⚠️ **这条是实测出来的**：2026-08-29 复跑轮现查，`research/prompts/e34-rootring-geometry-local.md`
# 与 `research/results/e34-rootring-local.out` 都在，而 kb 里没有 E34。
# results-cited 格抓不到它，因为那一格把文件名含 `local` 的一律当本地腿问答排除掉了
# ——**排除规则正好盖住了这一个**。
# 样本：experiment-orphans-red 里 research/data/ 与 crates/singlefs-checker-tier/src/bin/ 下各一个 kb 里没有正文的实验号；
# experiment-orphans-green 里的实验号都有正文。
cell_experiment_orphans() {
  EXP_DIR=.claude/kb/experiments
  [[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，这一格跳过"; exit 77; }

  have=$(ls "$EXP_DIR" | grep -oE '^[0-9]+' | sed 's/^0*//' | sort -un)
  # 射程逐个目录报：不在的列进成功行，读不了的（find 出错）判红——射程静默缩小与判过了在输出里长得一样
  SCOPE_DIRS=(research crates/singlefs-checker-tier/src/bin)
  absent=(); names_file="$(mktemp)"; errors_file="$(mktemp)"
  trap 'rm -f "$names_file" "$errors_file"' EXIT
  for d in "${SCOPE_DIRS[@]}"; do
    if [[ ! -d "$d" ]]; then absent+=("$d"); continue; fi
    if ! find "$d" -name target -prune -o -regextype posix-extended -regex '.*/e[0-9]+[^/]*' -printf '%f\n' >>"$names_file" 2>>"$errors_file"; then
      echo "  ✗ 扫 $d 时 find 出错，这个目录里的实验号没扫全："
      sed 's/^/      /' "$errors_file"
      echo "  → 怎么办：按上面的报错修好（读不了的子目录、权限），再跑；扫不全的目录会让孤儿实验号静默漏掉。"
      exit 1
    fi
  done
  want=$(grep -ohE '^e[0-9]+' "$names_file" | sed 's/^e//' | sort -un)

  orphan=()
  for n in $want; do grep -qx "$n" <<<"$have" || orphan+=("E$n"); done

  if ((${#orphan[@]})); then
    echo "  ✗ research 里有这些实验号的东西，kb/experiments/ 里却没有正文："
    for e in "${orphan[@]}"; do
      printf '      %-6s' "$e"
      find "${SCOPE_DIRS[@]}" -name target -prune -o \( -name "${e,,}[-_]*" -o -name "${e,,}.*" \) -print 2>/dev/null | head -5 | tr '\n' ' '
      echo
    done
    echo "  → 怎么办：给它建正文（测什么 / 判据 / 失败条款 / 口径 / 复跑命令），并登记进 experiments.md 的索引表；"
    echo "    若那轮的结论不打算入库，就把 research 下那些文件删掉——留着等于留一份没人能复核的证据。"
    exit 1
  fi
  if [[ -z "$want" ]]; then
    echo "  ⊘ 本次无对象可判：${SCOPE_DIRS[*]} 下没有一个以 eNN 命名的东西（不在的目录：${absent[*]:-（没有）}）"
    exit 77
  fi
  echo "  ✓ research 里的实验号在 kb 里都有正文（$(wc -w <<<"$want") 个）"
  echo "    扫了 ${SCOPE_DIRS[*]}（target/ 不扫）；不在的目录 ${#absent[@]} 个：${absent[*]:-（没有）}"
}

# ── 格 quoted-result-lines：kb 与 research 正文里整行抄的产物行（只认去掉首尾空白后以 `E7RESULT ` 开头的行），产物里逐字找得到
#
# 判据：kb 正文（「## 历史版本」之前；*-history.md 与 decisions-history/ 不算）里去掉首尾空白后
# 以 `E7RESULT ` 开头的行，必须在 research/results/*.out 的某一行里逐字出现。
# 找不到 = 抄的时候改了数，或者产物重跑之后正文没跟。
#
# ⚠️ **这条是实测出来的，不是想出来的**（2026-09-16）：规则早就写着「引产物就整行抄」「重跑之后要回对正文」
# （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`），检查一直没有——checker-tier-research-build-and-replay.sh 的 experiment-replay 格只比产物与二进制、不看正文，
# results-cited 格只看产物有没有被点名。树表条目 148 → 200 那一轮撞见 E146 的判决块混着两版产物（config 行 145、
# 下面几行 148），E142 的正文有三行停在旧产物上；这道检查接上的当天又扫出 E139 两行、E144 一行，
# 正文里的数在它自己点名的留存产物里找不到。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   1. 只认整行。句中反引号里的片段、按字段转述成一句话的数，一概不判。
#   2. 不按实验号配对：正文允许引别的实验的产物（E152 引 E142 的 write_list）。代价是两个实验恰好吐出同一行时，
#      引错了实验认不出来。**这次改动新增或改写的行只对照 `research/scripts/replay.sh` 登记着的产物**
#      （树里还留着的旧一轮产物不算：登记的已经是新一轮，正文却新抄了旧一轮的行，就是「重跑之后正文没跟」）；
#      登记的产物已归档出了树的，去版本库历史里取它最后那一版。退回全量判时照旧对照全部产物加全部归档。
#   3. 历史节里的旧行不判：旧产物被删掉之后，历史节里留着当时抄的那一行是对的。
#   4. **只判这次改动新增或改写的行**（用户 2026-09-21 定）：正文里早先抄下的行是历史，仅作参考——
#      那一轮跑过、当时对得上，之后产物按「每次提交删上一次的实验记录」删掉了，再判就是判一个不存在的对照。
#      要推翻早先的结论，按 `.claude/rules/three-way-inference.md` 重新跑三腿，那时产物是新的、这一格照样判得到。
#      不在 git 仓里、或取不到新增行（git 失败）时**退回全量判**（保守：宁可多判，不可漏判）。
#      ⚠️ 没有上游、也没设 GATE_BASE 时，共用脚本的基准是 HEAD，窗口只含工作区与暂存区：已提交没推的那几次不在里面。
#      这一格与「共用脚本认不认门禁导出的 GATE_DIFF_BASE」同根，交用户定（research/prompts/gate-fix-forks-r1-forks.md 的 T11）。
# 样本：quoted-result-lines-red 里一页未跟踪的新实验页整行抄了两行在登记着的产物里找不到的产物行（一行只在没登记的旧一轮产物里有）；
# quoted-result-lines-green 里正文抄的那一行在产物里逐字找得到、历史节里的旧行不判。
cell_quoted_result_lines() {
  [[ -d .claude/kb ]] || { echo "  ! 找不到 .claude/kb，这一格无对象可判"; exit 77; }

  # 改动范围取这一道开头只经共用脚本 research/scripts/changed-paths.sh 取一次的那一份（门禁 code-tooling.sh 的 change-range-single-source 格判阶段里不另算一份）：基准 gate_diff_base gate，
  # 新增行 gate_added_lines——未跟踪的文件整份算新增，新写的 kb 页在 git add 之前抄的产物行也要判。
  if ((! changed_paths_library_loaded)); then
    echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
    echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"
    exit 1
  fi
  SCOPE_BASE=""
  if ((in_git_work_tree && added_lines_ok)); then
    SCOPE_BASE="$shared_base"
  fi
  ADDED_LINES="$SHARED_ADDED_LINES"
  export SCOPE_BASE ADDED_LINES

  python3 - <<'PY'
import glob, os, subprocess, sys

# 射程不止 kb：research/ 下也有正文整行抄产物（research/perf-by-milestone.md 就是），
# 而它原先一份门禁都罩不到——2026-09-21 术语改名把那里一行引文改成了假话，没有任何检查会说。
# 冻结证据目录（research/prompts/ 的提示与腿的产出、research/results/ 的产物本身）不判：
# 前者按 evidence-discipline 不许事后改，后者就是被比对的那一方。
import re
kb = sorted(f for f in glob.glob('.claude/kb/**/*.md', recursive=True)
                 + glob.glob('research/**/*.md', recursive=True)
            if not f.endswith('-history.md') and '/decisions-history/' not in f
            and not f.startswith('research/prompts/') and not f.startswith('research/results/'))
quoted = []
for path in kb:
    text = open(path, encoding='utf-8').read()
    cut = text.find('\n## 历史版本')
    body = text if cut < 0 else text[:cut]
    for number, line in enumerate(body.split('\n'), 1):
        stripped = line.strip()
        if stripped.startswith('E7RESULT '):
            quoted.append((path, number, stripped))
# 只判这次改动新增或改写的行：共用脚本交来的新增行（路径<TAB>行文）里挑 E7RESULT 行，
# 不在 git 仓里或取不到新增行（SCOPE_BASE 为空）就退回全量判（保守，宁可多判不可漏判）。
scope = '全量'
all_quoted = list(quoted)
if os.environ.get('SCOPE_BASE', '').strip():
    added = set()
    with open(os.environ['ADDED_LINES'], encoding='utf-8', errors='replace') as added_handle:
        for record in added_handle.read().split('\n'):
            added_path, separator, added_text = record.partition('\t')
            if separator and added_text.strip().startswith('E7RESULT '):
                added.add((added_path, added_text.strip()))
    quoted = [(path, number, stripped) for path, number, stripped in quoted
              if (path, stripped) in added]
    scope = '这次改动新增或改写的'

if not quoted:
    print('  ! %s kb 正文里没有整行抄的 E7RESULT 行，这一格无对象可判' % scope)
    sys.exit(77)

# 登记的产物：research/scripts/replay.sh 登记表里每一行的第 4 列（与 verdict-false-named 格、门禁 checker-tier-research-build-and-replay.sh 的 experiment-replay 格同一个形状）
REPLAY = 'research/scripts/replay.sh'
ROW_RE = re.compile(r'^(E[0-9]+)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$')
registered = None
if os.path.isfile(REPLAY):
    registered = set()
    with open(REPLAY, encoding='utf-8') as handle:
        for line in handle:
            row = ROW_RE.match(line.rstrip('\n'))
            if row:
                registered.add(row.group(4))
if scope != '全量' and not registered:
    print(f'  ✗ 读不到 {REPLAY} 的登记表（文件不在，或一行 编号|二进制|参数|产物|exact/timing 都没读到），分不出哪些产物是这一轮登记的')
    print('     → 怎么办：登记表是新抄的产物行唯一的对照；它挪了地方或改了格式，就同步改这一格（与 verdict-false-named 格、门禁 checker-tier-research-build-and-replay.sh 的 experiment-replay 格同一份形状）。')
    sys.exit(1)

products = sorted(glob.glob('research/results/*.out'))
product_lines = set()
registered_lines = set()
line_sources = {}
for product in products:
    is_registered = registered is not None and os.path.basename(product) in registered
    with open(product, encoding='utf-8', errors='ignore') as handle:
        for line in handle:
            text = line.strip().replace('\r', '')
            product_lines.add(text)
            line_sources.setdefault(text, set()).add(os.path.basename(product))
            if is_registered:
                registered_lines.add(text)

# 树里找不到就去版本库历史里找：产物按「每次提交删上一次的实验记录」归档进 git，
# 树里没有不等于对照物没有。少了这一段，归档之后每一行历史引文都会被报成「找不到」，
# 而它们当时逐字是对的——那样这道检查会由「抓抄错的数」退化成「抓归档过的产物」。
# 反过来它也更硬：引文与**归档那一刻的产物字节**不符，照样红（2026-09-21 术语改名
# 把 research/perf-by-milestone.md 里一行引文改成了假话，就是这一格抓出来的）。
archived_count = 0
def archived_product_lines(only_names=None):
    """归档进版本库的产物的行；only_names 给了就只取这些文件名的（登记着、却已出了树的那几份）。"""
    global archived_count
    lines = set()
    log = subprocess.run(['git', '-c', 'core.quotepath=false', 'log', '--all', '--diff-filter=D', '--format=%H', '--name-only',
                          '--', 'research/results'], capture_output=True, text=True).stdout
    commit = None
    for entry in log.split('\n'):
        entry = entry.strip()
        if not entry:
            continue
        if len(entry) == 40 and all(character in '0123456789abcdef' for character in entry):
            commit = entry
            continue
        if commit and entry.endswith('.out') and (only_names is None or os.path.basename(entry) in only_names):
            blob = subprocess.run(['git', 'show', '%s^:%s' % (commit, entry)],
                                  capture_output=True, text=True)
            if blob.returncode == 0:
                archived_count += 1
                for one in blob.stdout.split('\n'):
                    lines.add(one.strip().replace('\r', ''))
    # 正在归档的：HEAD 里还在、工作区里已经删了的产物（这一批照归档规则删上一轮的产物，删除还没提交，
    # git log --diff-filter=D 还找不到它）。它们的字节就在 HEAD 里，与已提交的归档是同一件事。
    in_head = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-tree', '-r', '--name-only', 'HEAD', '--', 'research/results'],
                             capture_output=True, text=True)
    for entry in in_head.stdout.split('\n') if in_head.returncode == 0 else []:
        entry = entry.strip()
        if entry.endswith('.out') and not os.path.exists(entry) and (only_names is None or os.path.basename(entry) in only_names):
            blob = subprocess.run(['git', 'show', 'HEAD:%s' % entry], capture_output=True, text=True)
            if blob.returncode == 0:
                archived_count += 1
                for one in blob.stdout.split('\n'):
                    lines.add(one.strip().replace('\r', ''))
    return lines

if scope == '全量':
    compared_lines = product_lines
else:
    compared_lines = registered_lines
missing = [(path, number, stripped) for path, number, stripped in quoted if stripped not in compared_lines]
if missing:
    if scope == '全量':
        compared_lines |= archived_product_lines()
    else:
        on_disk = {os.path.basename(product) for product in products}
        compared_lines |= archived_product_lines(only_names=registered - on_disk)
    missing = [item for item in missing if item[2] not in compared_lines]
if missing:
    compared_what = (f'research/results/ 的 {len(products)} 份产物' if scope == '全量'
                     else f'{REPLAY} 登记着的 {len(registered)} 份产物')
    print(f'  ✗ {scope} kb 正文里有 {len(missing)} 行整行抄的产物行，在 {compared_what}里一份都找不到：')
    for path, number, stripped in missing:
        print(f'     {path}:{number}  {stripped[:160]}')  # gate-lint:detail
        unregistered = sorted(line_sources.get(stripped, ()))
        if unregistered:
            print(f'       只在没在 replay.sh 登记的产物里有（{"、".join(unregistered[:3])}）：那是旧一轮的，登记的已经换了')  # gate-lint:detail
    print('     → 怎么办：打开这个实验复跑命令登记的那份产物，把对应的行原样抄回来，别手改数；')
    print('               正文里照这几行写的结论句一起回对。产物确实没留存，就删掉这几行并写明「原始输出未留存」。')
    sys.exit(1)
compared_count = len(products) if scope == '全量' else len(registered)
print(f'  ✓ {scope} kb 正文里整行抄的产物行都在产物里逐字找得到（判了 {len(quoted)} 行，对照 {compared_count} 份'
      + ('产物）' if scope == '全量' else '登记着的产物）'))
judged = {(path, number) for path, number, _ in quoted}
skipped = {}
for path, number, _ in all_quoted:
    if (path, number) not in judged:
        skipped.setdefault(path, []).append(number)
skipped_count = sum(len(numbers) for numbers in skipped.values())
print(f'     没判的 {skipped_count} 行（早先抄下的，历史参考，产物按「每次提交删上一次的实验记录」删掉了；要推翻它们按三方重跑）'
      + (''.join(f'\n       {path}:{",".join(str(n) for n in numbers)}' for path, numbers in sorted(skipped.items())) if skipped else '：（没有）'))
PY
}

# ── 格 archive-past-rounds：上一轮及更早的实验记录已经归档进版本库，工作区里不留
#
# 规则见 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」之下那三段：
# 冻结只在本轮内有效；这一轮提交之后，下一次提交把上一次留下的那批删掉；本次的照旧留着、本次的验证照旧做得了；
# 要查删掉的那些去版本库历史里找，对早先的结论有疑问就重新验证，不翻旧证据。
#
# ⚠️ **这条是实测出来的**（2026-09-21）：留着的旧记录不只占地方，它让当前工程的命名、路径、口径出现二义性——
# 那天一次全仓术语改名量出 22741 处旧名，其中 19639 处（86%）躺在上一轮及更早的提示与产物里，
# 改它们对今天的工程没有任何用处，不改又让「全仓一致」这件事永远做不完。
#
# 保留三类：三方判决 `*-main-verification.md`（kb 的依据指着它）、`abandoned-rounds.tsv`（门禁 doc-process-records.sh 的 abandoned-rounds 格的输入）、
# 还被代码当输入的产物（门禁阶段、脚本或装置源码的非注释行点名它，见 archive-past-rounds.py 的 still_an_input）。
# 「这一轮」的起点与门禁 doc-process-records.sh 的 knowledge-sync 格同一个基准：research/scripts/changed-paths.sh 的 `gate_diff_base gate`
# （GATE_BASE，否则 @{upstream} 的 merge-base，都没有就 HEAD）；就是这一道开头取的那一个，经 GATE_BASE 交给脚本，
# 脚本判红时出路里打出来的基准因此是这一个具体的提交，不再是占位句。
# 不在 git 仓里这一格退 77（分不出哪些是上一轮提交进来的；原来那一道在那里让脚本带着调用栈退出、判红，2026-09-28 并进来时改掉）。
# 判别力：fixtures/doc-experiments.sh/archive-past-rounds-red 是一个留着上一轮产物的小仓，必须判红；
# archive-past-rounds-green 只有本轮的，必须判绿。
cell_archive_past_rounds() {
  [[ -d "$ROOT/research/prompts" || -d "$ROOT/research/results" ]] || { echo "  ! 没有 research/prompts 与 research/results，这一格无对象可判"; exit 77; }
  # 不在 git 仓里分不出哪些是上一轮提交进来的（脚本的 git ls-files 会带着调用栈退出）：这一格无对象可判，不记通过
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，分不出哪些记录是上一轮提交进来的，这一格无对象可判"; exit 77; }
  if ((changed_paths_library_loaded)); then
    if GATE_BASE="$shared_base" python3 "$ARCHIVE_SCRIPT" --check "$ROOT"; then
      exit 0
    fi
  elif python3 "$ARCHIVE_SCRIPT" --check "$ROOT"; then
    exit 0
  fi
  echo "  ✗ 工作区里还留着上一轮及更早的实验记录（上面按目录列出份数）"
  echo "     → 怎么办：照上面那一行打出来的基准跑 --apply 删掉（不带基准它会按 HEAD 算、把这一次提交要带的记录也删掉），"
  echo "               它同时把别处指向这些文件的引用改成只留文件名、不留路径；删完跑共享门禁的「链接指向」阶段确认链接都还到得了。"
  exit 1
}

# ── 格 multipath-registry：多条路径互证的实验，路径与共用项要登记且指得到
#
# 还 checks-owed.md C48（多条校验路径共用同一个前提）可机检的那一半。
#
# 判据与形态在 `.claude/rules/format-evolution.md`「实验页的「路径与结论登记」」，这一格判三条：
#   ① 登记表的形状：`### 路径与结论登记` 之下恰好一张表，表头逐字
#      `| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |`，每行五格；
#   ② 源码落点指得到：写成 `路径:行号` 的，那个文件要在仓里现存；只活在正文里的写
#      「无实现：理由」，理由不许空；
#   ③ 共用项那一格不许空，也不许只写「无」（去掉括号、标点与「共用项」这类字眼之后只剩「无」「没有」「N/A」「none」
#      这一类的都算只写了「无」）——多条路径逐格相等不构成证据，当它们共用
#      同一个错误前提时，而那一列正是唯一能让人看出「共用了什么」的地方；
#   表头下面要有分隔行；数据行按「不是分隔行」认，不按位置跳过前两行（少了分隔行时第一条数据行照判）。
#   没有一页写了登记节、滞后表也一行都没有，本次无对象可判，这一格退 77。基准里还没有滞后表（只缩不涨没法比）
#   与 git 取不到基准那一版分开：后者判红。
#   ④ 还没写登记节的实验页登记在 `.claude/gate.d/multipath-registry-lag.tsv`，这张表每一行：
#      两列（页面路径、普查判定的依据）用制表符分隔、都不许空；路径形如
#      `.claude/kb/experiments/<编号>-….md` 且文件在仓里现存；那一页确实还没有登记节——补上了就判过期，删行；
#      **只缩不涨**：与基准提交比，基准里这张表没有的实验编号不许出现。按编号比、不按整条路径比：
#      页面改名、登记行抄错了题目改正，都不算新加了一份。
#
# ⚠️ **射程只到已经写了这一节的实验页**。建表时 10 份实验页在做多条路径互证（普查
# `research/prompts/c48-c49-multipath-survey.md`，已归档，去 git 历史里找：
# `git show 13bccea:research/prompts/c48-c49-multipath-survey.md`），只有 1 份写了登记节；
# 其余登记在滞后表里，那张表**只缩不涨**：一份补齐了就删一行，新写的实验页不许往里加。
# 要判「该写而没写」那一半得先认出「这个实验在做多条路径互证」，
# 而按关键词扫正文认不出来——普查里 15 份命中有 5 份，grep 命中的那一行说的是别的意思，
# 真结构在页面别处。那一半的欠账在 C48（多条校验路径共用同一个前提）。
#
# 判别力：fixtures/doc-experiments.sh/{multipath-registry-red,multipath-registry-green}/ 各一份实验页，
# red 那份四行里一行共用项写「无」、一行写「N/A」、一行源码落点指不到文件，另一份页的表缺分隔行、第一条数据行共用项写「无」，
# 另带一张滞后表：一行指向不存在的页、
# 一行指向已经写了登记节的页、一行少了依据那一列、一行的实验编号基准里没有，必须判红；
# green 的滞后表只有一行、指向一份还没写登记节的页、与基准相同，必须判绿。
cell_multipath_registry() {
  EXPERIMENTS=.claude/kb/experiments
  LAG=.claude/gate.d/multipath-registry-lag.tsv
  [[ -d "$EXPERIMENTS" ]] || { echo "  ! 没有 $EXPERIMENTS，这一格无对象可判"; exit 77; }
  base=""
  if ((in_git_work_tree)); then
    # 基准取法与 doc-process-records.sh 用改动范围的几格、evidence-in-repo 格、decision-links 格、doc-registries.sh 的 invariant-anchors 那一格同一份：research/scripts/changed-paths.sh 的 gate 取法
    if ((! changed_paths_library_loaded)); then
      echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
      echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"
      exit 1
    fi
    base="$shared_base"
  fi

  python3 - "$EXPERIMENTS" "$LAG" "$base" <<'PY'
import os, re, subprocess, sys, glob

experiments, lag_path, base = sys.argv[1], sys.argv[2], sys.argv[3]
HEADING = '### 路径与结论登记'
WANTED = '| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |'
LAG_PAGE = re.compile(r'\.claude/kb/experiments/(?P<number>[0-9]+)-[^/]+\.md')

def has_registration(text):
    return HEADING in text

SAYS_NOTHING = ('', '无', '没有', '没', 'na', 'none', 'nil', '不适用')

def says_nothing(shared):
    """共用项那一格去掉括号、标点、空白与「共用项 / 共用 / 共享」这类字眼之后，只剩「无」这一类：等于没写。"""
    core = re.sub(r'[\s（）()\[\]【】「」`*。．.，,、;；:：/\\\-—–_]+', '', shared).lower()
    core = re.sub(r'共用项|共用|共享', '', core)
    return core in SAYS_NOTHING

def lag_rows(text):
    """滞后表的内容行：[(行号, 各列)]，注释与空行不算。"""
    rows = []
    for line_number, line in enumerate(text.split('\n'), 1):
        if line.strip() and not line.startswith('#'):
            rows.append((line_number, line.rstrip('\r').split('\t')))
    return rows

problems, checked_pages, checked_rows = [], 0, 0
for path in sorted(glob.glob(os.path.join(experiments, '*.md'))):
    text = open(path, encoding='utf-8').read()
    if not has_registration(text):
        continue
    checked_pages += 1
    section = text.split(HEADING, 1)[1]
    cut = re.search(r'^#{2,4}\s', section, re.M)
    if cut:
        section = section[:cut.start()]
    rows = [line for line in section.split('\n') if line.startswith('|')]
    if not rows:
        problems.append(f"{path}：「{HEADING}」之下一张表都没有")
        continue
    if rows[0].strip() != WANTED:
        problems.append(f"{path}：表头不是 {WANTED}，实际是 {rows[0].strip()[:60]}")
        continue
    if len(rows) < 2 or not re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', rows[1].strip()):
        problems.append(f"{path}：表头下面没有分隔行（|---|---|…|），表格渲染不出来；下面的数据行照判")
    for row in rows[1:]:
        if re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', row.strip()):
            continue
        cells = [cell.strip() for cell in row.strip().strip('|').split('|')]
        if len(cells) != 5:
            problems.append(f"{path}：这一行不是五格：{row.strip()[:60]}")
            continue
        checked_rows += 1
        number, name, _how, where, shared = cells
        for hit in re.findall(r'`([^`]+?):(\d+)`', where):
            if not os.path.exists(hit[0]):
                problems.append(f"{path}：第 {number} 行「{name}」的源码落点指不到文件：{hit[0]}")
        if not re.search(r'`[^`]+?:\d+`', where) and not where.startswith('无实现：'):
            problems.append(f"{path}：第 {number} 行「{name}」的源码落点既不是 路径:行号，也不是「无实现：理由」：{where[:40]}")
        if where.startswith('无实现：') and not where[len('无实现：'):].strip():
            problems.append(f"{path}：第 {number} 行「{name}」写了「无实现：」而理由是空的")
        if says_nothing(shared):
            problems.append(f"{path}：第 {number} 行「{name}」的共用项那一格是空的或只写了「{shared}」")

# ④ 滞后表：形状、路径、过期、只缩不涨
lag_problems, lagging, lag_pages_by_number = [], [], {}
if os.path.isfile(lag_path):
    for line_number, fields in lag_rows(open(lag_path, encoding='utf-8').read()):
        where = f"{os.path.basename(lag_path)} 第 {line_number} 行"
        if len(fields) != 2 or not all(field.strip() for field in fields):
            lag_problems.append(f"{where}：不是两列（页面路径、普查判定的依据）或有一列是空的：{chr(9).join(fields)[:80]}")
            continue
        page = fields[0].strip()
        page_shape = LAG_PAGE.fullmatch(page)
        if not page_shape:
            lag_problems.append(f"{where}：路径不是 .claude/kb/experiments/<编号>-….md 的形状：{page}")
            continue
        lag_pages_by_number[int(page_shape.group('number'))] = page
        if not os.path.isfile(page):
            lag_problems.append(f"{where}：指向的实验页不存在：{page}")
            continue
        if has_registration(open(page, encoding='utf-8').read()):
            lag_problems.append(f"{where}：{page} 已经写了「{HEADING}」，这一行过期了")
            continue
        lagging.append(page)

baseline_numbers = None
if base:
    # 「基准里还没有这张表」与「git 取不到基准那一版」分开：后者当成没有基线，只缩不涨就静默不比了
    listed = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-tree', '--name-only', base, '--', lag_path],
                            capture_output=True, text=True)
    shown = None
    if listed.returncode != 0:
        lag_problems.append(f"取不到基准 {base} 里的 {lag_path}（git ls-tree 退 {listed.returncode}：{listed.stderr.strip()[:80]}），只缩不涨没比")
    elif listed.stdout.strip():
        shown = subprocess.run(['git', '-c', 'core.quotepath=false', 'show', f'{base}:{lag_path}'],
                               capture_output=True, text=True)
        if shown.returncode != 0:
            lag_problems.append(f"读不出基准 {base} 里的 {lag_path}（git show 退 {shown.returncode}：{shown.stderr.strip()[:80]}），只缩不涨没比")
            shown = None
    if shown is not None:
        baseline_numbers = set()
        for _line_number, fields in lag_rows(shown.stdout):
            page_shape = LAG_PAGE.fullmatch(fields[0].strip())
            if page_shape:
                baseline_numbers.add(int(page_shape.group('number')))
grown = sorted(set(lag_pages_by_number) - baseline_numbers) if baseline_numbers is not None else []

if problems:
    print(f"  ✗ {len(problems)} 处登记不合形态：")                       # gate-lint:summary
    for entry in problems:
        print(f"     {entry}")                                          # gate-lint:detail
    print("     → 怎么办：形态见 .claude/rules/format-evolution.md「实验页的「路径与结论登记」」。")
    print("               表头逐字 | # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |，每行五格；")
    print("               源码落点写 `路径:行号 函数名`，只活在正文里的写「无实现：理由」；")
    print("               共用项写这条路径读了哪些常量、哪些规则、哪份规范，一个都不许省——")
    print("               多条路径逐格相等不构成证据，当它们共用同一个错误前提时。")
if lag_problems:
    print(f"  ✗ 滞后表 {lag_path} 里 {len(lag_problems)} 行对不上仓里的实验页：")  # gate-lint:summary
    for entry in lag_problems:
        print(f"     {entry}")                                          # gate-lint:detail
    print("     → 怎么办：路径抄错了就照 .claude/kb/experiments/ 里现存的文件名改；那一页已经补了登记节，就删掉这一行——")
    print("               留着的滞后行会让人以为那一页还欠着，而指向不存在的页等于那一格没人管。")
if grown:
    print(f"  ✗ 滞后表涨了：这些实验页的编号，基准 {base} 的滞后表里没有：")  # gate-lint:summary
    for number in grown:
        print(f"     {lag_pages_by_number[number]}")                     # gate-lint:detail
    print("     → 怎么办：这张表只缩不涨。新写的实验页要做多条路径互证，就当场写「### 路径与结论登记」那一节，")
    print("               不许加一行滞后把它按下去——加得了一行，这一格对新写的实验页就什么都拦不住了。")
if problems or lag_problems or grown:
    sys.exit(1)

if checked_pages == 0 and not lagging and not lag_pages_by_number:
    print(f"  ⊘ 本次无对象可判：没有一页实验写了「{HEADING}」，{os.path.basename(lag_path)} 里也一行都没有")
    sys.exit(77)
compared = f"与基准 {base} 比没涨" if baseline_numbers is not None else "基准里还没有这张表（或不是 git 仓），只缩不涨这一条没比"
print(f"  ✓ 登记了路径与共用项的实验页判过了（{checked_pages} 份、{checked_rows} 条路径）；"
      f"还没写登记节的 {len(lagging)} 份在 {os.path.basename(lag_path)} 里、每一份都现存且确实还没写，{compared}；逐份补齐后删行")
PY
}

stage_cells_run "$ROOT"
