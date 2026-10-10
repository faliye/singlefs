#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要 cargo：缺了就在单测那一格判红、复跑那一格由它调的 research/scripts/replay.sh 编实验二进制时判红，不交给 gate.sh 预判；此外只要跑门禁本身就要的 bash、git、python3
# gate-stage: research 工作区的构建与单测；入库的实验数今天还复现得出来吗
# gate-category: checker-tier 类
# gate-similar: harness-model-differential-and-scenarios.sh 它也经内存包装跑 release 的 cargo test，但编的是仓根的 crates 工作区、判的是模型对拍那几段；这里编的是 research 工作区，两个工作区互相看不到对方，编译产物不共用，并进来反而要多编一份
# gate-similar: code-source-discipline.sh 它的实验源码纪律那一格静态判 research 的实验二进制与变异表（锚点、每个实验有变异表），不编译、不跑；这里真编、真跑单测与复跑，判的是编得过、跑得出同样的数
# gate-similar: doc-experiments.sh 它判实验页与产物登记（索引行、产物写回、产物落在仓里），读的是 kb 与 research/results/ 的文字；这里把产物重跑一遍逐字节比，读的是实验二进制的输出
# gate-similar: code-tooling.sh 研究脚本的 --selftest 都在它的 research-script-selftests 格跑（不要 research 工作区先编好、经 light-selftest-guard.py 盯着）；这里只编 research 工作区、跑它的单测与复跑，一份研究脚本的自证都不跑
#
# gate-cell: research-unit-tests research 工作区编得过、单测全绿
# gate-cell: experiment-replay 入库的纯模型实验重跑一遍，与 research/results/ 里的产物逐字节相同
#   research-unit-tests  每次都跑：cargo build --release 编整个 research 工作区，再 cargo test --release。
#                        上游「构建与单测」只编 crates/，kb 里实测结论背后的实验二进制只有这一格编。
#   experiment-replay    .claude/gate.d/stage-inputs.tsv 里本阶段那一行的路径相对上次整轮全绿那棵暂存树没变就退 77
#                        （为什么按树、按一条 ref、不看工作区，写在 research/scripts/stage-must-run.sh 文件头）；
#                        再问 change-touches-crates.sh，前缀不含 research/prompts/（冻结证据改不了实验输出）。
#                        派给 research/scripts/replay.sh 的是表里的 exact 行减掉 SLOW；SLOW 里的与表里每一行 timing 都逐个列成「没派」。
#                        对不上不一定是代码坏了，也可能是产物该更新，两种都要人判，所以判红。
#                        C8（门禁范围判不出来）还欠按布局算受影响集合，这里按前缀判只是它的粗粒度前身。
# 格结构经 lib/stage-cells.sh。
#
# 读的环境变量：
#   GATE_RESEARCH_BUILD_MEMORY_MAX  cargo test 经 research/scripts/run-with-memory-cap.sh 跑的内存上限，没设取 8G；谁跑这一道外面都不再包一层。
#                                   包装退 245 或 250–254 不是单测的判定，单独判红、单独给出路。cargo build 不跑编出来的代码，不经包装。
#   GATE_REPLAY_FULL=1              复跑那一格连 SLOW 与 timing 行一起跑
#
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh [项目根]                                两格都跑
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests [项目根]     只跑单测那一格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --list-items [项目根]                    逐行打 replay.sh 表里的实验号、exact 还是 timing、在不在 SLOW 里，不编、不跑
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check experiment-replay --item E142 [项目根]   只复跑点名的实验
#     --item 给几次取并集，timing 与 SLOW 也照跑；成功行写明只跑了点名的几个，不是整格结论；实验号不在表里退 2
#
# 样本：fixtures/checker-tier-research-build-and-replay.sh/ 下四份，各放 .gate-cells 只点名它判的那一格：
#   research-unit-tests-red    research 工作区里有一条单测红着的小仓，判红
#   research-unit-tests-green  一条单测全绿、research/scripts/ 下一份脚本都没有，判绿，成功行报一个测试批次
#   green                      带假 replay.sh 的小仓，表里一行快的 exact、一行 SLOW 里的 exact、一行 SLOW 里只有 timing 的；
#                              快的那行派给 replay.sh，另两行逐个列进「没派」
#   red                        表一行都没有，判红
# 弄坏开关归共用库：STAGE_CELLS_BREAK=red-swallowed 让两份红样本整道退 0；=marker-ignored 让每份样本两格都跑，
#   research-unit-tests-green 里复跑那一格对着没有表的 replay.sh 判红。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
STAGE_REPOSITORY="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_NAME="$(basename "$0")"
source "$STAGE_DIR/lib/stage-cells.sh"

stage_cell research-unit-tests cell_research_unit_tests "research 工作区编得过、单测全绿" \
  "先修好编不过或红着的单测（kb 里的实测结论全靠它们背书）；改完只重跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests"
stage_cell experiment-replay cell_experiment_replay "入库的纯模型实验重跑一遍，与 research/results/ 里的产物逐字节相同" \
  "对不上先判是代码坏了还是产物该更新（两种都要人判）；只重跑红的那几个实验：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check experiment-replay --item <实验号>"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

list_items=0; root_argument=""; named_experiments=()
while (($#)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then
        echo "  ✗ --item 后面缺实验号"
        echo "     → 怎么办：写成 --item <实验号>（例 --item E142），几个实验就给几次；可点的号用 --list-items 看。"
        exit 2
      fi
      named_experiments+=("$2"); shift 2 ;;
    --item=*) named_experiments+=("${1#--item=}"); shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--list-items、--item <实验号> 与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" || exit 2
R=research
REPLAY_SCRIPT="$R/scripts/replay.sh"

# replay.sh 表里的行：编号|二进制|参数|产物|exact 或 timing。门禁靠这个形状取行，改表格式要同时改这里。
replay_rows_of_kind() { sed -n "s/^\(E[0-9]*\)|[^|]*|[^|]*|[^|]*|$1\$/\1/p" "$REPLAY_SCRIPT" 2>/dev/null; }
SLOW=(E9 E17 E20 E21)

if ((list_items)); then
  if [[ ! -f "$REPLAY_SCRIPT" ]]; then
    echo "  ✗ 没有 $REPLAY_SCRIPT：列不出可点名的实验"
    echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"
    exit 1
  fi
  listed=0
  while IFS= read -r experiment; do
    [[ -n "$experiment" ]] || continue
    kinds="$(sed -n "s/^${experiment}|[^|]*|[^|]*|[^|]*|\(exact\|timing\)\$/\1/p" "$REPLAY_SCRIPT" | sort -u | paste -sd, -)"
    slow_note=""
    for s in "${SLOW[@]}"; do [[ "$s" == "$experiment" ]] && slow_note="（慢，默认不跑）"; done
    [[ "$kinds" == timing ]] && slow_note="（只有 timing 行，默认不跑）"
    printf '%s\t%s%s\n' "$experiment" "$kinds" "$slow_note"
    listed=$((listed + 1))
  done < <({ replay_rows_of_kind exact; replay_rows_of_kind timing; } | awk '!seen[$0]++')
  ((listed)) || { echo "  ✗ 从 $REPLAY_SCRIPT 里一行都没读到：表的格式变了？"; echo "     → 怎么办：这里靠 '编号|二进制|参数|产物|exact 或 timing' 这个形状取行，改表格式要同时改这里。"; exit 1; }
  exit 0
fi

if ((${#named_experiments[@]})); then
  mapfile -t known_experiments < <({ replay_rows_of_kind exact; replay_rows_of_kind timing; } | awk '!seen[$0]++')
  unknown_experiments=()
  for experiment in "${named_experiments[@]}"; do
    found=0
    for known in ${known_experiments[@]+"${known_experiments[@]}"}; do [[ "$known" == "$experiment" ]] && found=1; done
    ((found)) || unknown_experiments+=("$experiment")
  done
  if ((${#unknown_experiments[@]})); then
    echo "  ✗ --item 点名的实验不在 $REPLAY_SCRIPT 的表里：${unknown_experiments[*]}"
    echo "     → 怎么办：实验号照 bash .claude/gate.d/${STAGE_NAME} --list-items 列的写（表里有 ${#known_experiments[@]} 个）。"
    exit 2
  fi
fi

# ── research-unit-tests ──
cell_research_unit_tests() {
  local out rc b n
  local MEMORY_CAP_RUNNER RESEARCH_BUILD_MEMORY_MAX
  [[ -f "$R/Cargo.toml" ]] || { echo "  ! 没有 $R/Cargo.toml，这一格无对象可判"; exit 77; }
  command -v cargo >/dev/null || {
    echo "  ✗ 没有 cargo，装不了就没法验 research 的证据"
    echo "     → 怎么办：装 Rust 工具链（curl https://sh.rustup.rs -sSf | sh），"
    echo "               或把已装的 cargo 放进 PATH。跳过这一步等于 research/ 的数字没人验过。"
    exit 1; }
  MEMORY_CAP_RUNNER="$STAGE_REPOSITORY/research/scripts/run-with-memory-cap.sh"
  RESEARCH_BUILD_MEMORY_MAX="${GATE_RESEARCH_BUILD_MEMORY_MAX:-8G}"
  if [[ ! "$RESEARCH_BUILD_MEMORY_MAX" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    echo "  ✗ GATE_RESEARCH_BUILD_MEMORY_MAX 写成了「$RESEARCH_BUILD_MEMORY_MAX」，内存包装不认（或认成字节数）：这一格没跑"
    echo "     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G"
    exit 1
  fi
  # 先编整个 research 工作区的 release 产物：复跑那一格的 replay.sh 编的就是这一份，跑到那一格时 cargo 判它新鲜、不再编
  out="$(cd "$R" && cargo build --release 2>&1)"
  rc=$?
  if (( rc != 0 )); then
    grep -E '^error' <<<"$out" | head -8 | sed 's/^/     /'
    echo "  ✗ research 编不过（cargo build --release 退出码 $rc）"
    echo "     → 怎么办：先修好——kb 里的实测结论全靠这些实验二进制背书，编不过就等于那些数字今天没有来源。"
    exit 1
  fi
  out="$(cd "$R" && RUN_WITH_MEMORY_CAP_KEY="gate checker-tier-research-build-and-replay: cargo test --release (research)" \
    bash "$MEMORY_CAP_RUNNER" "$RESEARCH_BUILD_MEMORY_MAX" cargo test --release 2>&1)"
  rc=$?
  if (( rc == 245 || (rc >= 250 && rc <= 254) )); then
    tail -8 <<<"$out" | sed 's/^/     /'
    echo "  ✗ research 的构建与单测没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $rc（上限 $RESEARCH_BUILD_MEMORY_MAX），这一次的输出不算判定"
    echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一格要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；"
    echo "               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑。"
    echo "               253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑。245 是同一段时间内核报了显存不够：nvidia-smi 看谁在用卡，等它跑完再重跑这一格。"
    exit 1
  fi
  if (( rc != 0 )); then
    echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
    grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
    echo "     → 怎么办：先修好——kb 里的实测结论全靠它们背书，编不过就等于那些数字今天没有来源。"
    exit 1
  fi
  # 条数只用一个 awk 数（「test result: ok. N passed; …」的第 4 段）：串几道工具的话哪一环失败，条数就静默变成「?」
  b="$(grep -c '^test result: ok' <<<"$out")"
  n="$(awk '/^test result: ok\./ { passed += $4 } END { print passed + 0 }' <<<"$out")"
  [[ "$n" =~ ^[0-9]+$ ]] || n="没数出来（awk 输出「$n」）"
  # 一个测试批次都没有：不记通过，退 77
  if (( b == 0 )); then
    echo "  ! 本次无对象可判：cargo test 退 0，但输出里一个「test result: ok」批次都没有，research 的单测没判"
    exit 77
  fi
  echo "  ✓ research 构建与单测全绿：$b 个测试批次、共 $n 个单测"
}

# ── experiment-replay ──
cell_experiment_replay() {
  local e s skip dup ALL_EXACT=() ALL_TIMING=() FAST=() SKIPPED=()
  if ((${#named_experiments[@]})); then
    # 点名的实验：不问整道的复用判定（那一问是整张表的），replay.sh 按实验自己的准入照旧判「输入没变」
    bash "$REPLAY_SCRIPT" "${named_experiments[@]}" || exit 1
    echo "  ✓ 只复跑了点名的 ${#named_experiments[@]} 个实验：${named_experiments[*]}（其中准入判「输入没变」而没跑的，见上面 replay.sh 汇总里那一行）；这不是整格结论"
    exit 0
  fi
  source "$STAGE_REPOSITORY/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一格要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一格按红记。"; exit 1; }
  stage_run_or_skip "$STAGE_REPOSITORY/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一格读哪几条路径见 .claude/gate.d/stage-inputs.tsv 里 ${STAGE_NAME} 那一行。" -- "$ROOT" "$STAGE_NAME"
  stage_run_or_skip "$STAGE_REPOSITORY/research/scripts/change-touches-crates.sh" "这次改动没碰它判的东西" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；判据与前缀见 research/scripts/change-touches-crates.sh。" \
    -- "$ROOT" crates/ research/e7-index-bench/ research/scripts/ research/results/
  # 跑哪些、没跑哪些都从 replay.sh 的表里现算，不写死清单：写死的清单漏掉新加的实验，「没跑」那句就成了假话
  # （.claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」）。
  mapfile -t ALL_EXACT < <(replay_rows_of_kind exact)
  if ((${#ALL_EXACT[@]} == 0)); then
    echo "  ✗ 从 research/scripts/replay.sh 里一行都没读到——表的格式变了？"
    echo "    → 本阶段靠 '编号|二进制|参数|产物|exact' 这个形状取行，改表格式要同时改这里。"
    exit 1
  fi
  mapfile -t ALL_TIMING < <(replay_rows_of_kind timing)
  for e in "${ALL_EXACT[@]}"; do
    skip=0
    for s in "${SLOW[@]}"; do [[ "$e" == "$s" ]] && skip=1; done
    if ((skip)); then SKIPPED+=("$e（慢）"); else FAST+=("$e"); fi
  done
  # timing 行默认一行都不跑：逐个进「没跑」清单。去重只比已经进了清单的那几行（同一个编号既有 exact 行又有 timing 行时），
  # 不比 SLOW：SLOW 里的编号若只有 timing 行，拿 SLOW 去重会让它既不跑、也不进清单。
  for e in ${ALL_TIMING[@]+"${ALL_TIMING[@]}"}; do
    dup=0
    for s in ${SKIPPED[@]+"${SKIPPED[@]}"}; do [[ "$s" == "$e（慢）" || "$s" == "$e（计时）" ]] && dup=1; done
    ((dup)) || SKIPPED+=("$e（计时）")
  done

  if [[ "${GATE_REPLAY_FULL:-0}" == 1 ]]; then
    bash "$REPLAY_SCRIPT" || exit 1
    exit 0
  fi

  bash "$REPLAY_SCRIPT" "${FAST[@]}" || exit 1
  # 派给 replay.sh 的行里，登记了准入（.claude/gate.d/stage-inputs.tsv）而输入自留存产物以来没变的不跑，
  # replay.sh 汇总行报「输入没变没跑 N」并逐个列名；这里不说成「跑了」。
  echo "  ! 本次派给 replay.sh ${#FAST[@]} 个（表里的 exact 行减掉慢的；其中准入判「输入没变」而没跑的，见上面 replay.sh 汇总里那一行）；没派 ${#SKIPPED[@]} 个：${SKIPPED[*]}"
  echo "    标（慢）的是 SLOW 清单（${SLOW[*]}）里判定为 exact 的行；标（计时）的是表里判定为 timing 的行，默认档一行都不跑。"
  echo "    跑： GATE_REPLAY_FULL=1 bash .claude/scripts/gate.sh，或逐个点名：bash .claude/gate.d/${STAGE_NAME} --check experiment-replay --item <实验号>"
  echo "    ⚠️ 它们的结论靠 replay.sh 里的区间断言钉住，不跑就等于那几条断言本轮没验。"
}

stage_cells_run "$ROOT"
