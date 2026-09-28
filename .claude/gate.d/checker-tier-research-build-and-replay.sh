#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要 cargo：缺了就在单测那一格判红、复跑那一格由它调的 research/scripts/replay.sh 编实验二进制时判红，不交给 gate.sh 预判（用户 2026-09-26 定：项目更严）；此外只要跑门禁本身就要的 bash、git、python3
# gate-stage: research 工作区的构建、单测与研究脚本的自证；入库的实验数今天还复现得出来吗（原是 research 构建与单测、实验复跑两道：同一个 research 工作区、同一种 release 编译，合成一道两格，只编一次）
# gate-category: checker-tier 类
# gate-similar: harness-model-differential-and-scenarios.sh 它也经内存包装跑 release 的 cargo test，但编的是仓根的 crates 工作区、判的是模型对拍那几段；这里编的是 research 工作区，两个工作区互相看不到对方，编译产物不共用，并进来反而要多编一份
# gate-similar: code-source-discipline.sh 它的实验源码纪律那一格静态判 research 的实验二进制与变异表（锚点、每个实验有变异表），不编译、不跑；这里真编、真跑单测与复跑，判的是编得过、跑得出同样的数
# gate-similar: doc-experiments.sh 它判实验页与产物登记（索引行、产物写回、产物落在仓里），读的是 kb 与 research/results/ 的文字；这里把产物重跑一遍逐字节比，读的是实验二进制的输出
#
# 两格（--list 打格名表，--check <格名>[,<格名>…] 只跑点名的格，不给就两格都跑，格名写错退 2）：
# gate-cell: research-unit-tests research 工作区编得过、单测全绿、研究脚本的自证通过
# gate-cell: experiment-replay 入库的纯模型实验重跑一遍，与 research/results/ 里的产物逐字节相同
#   research-unit-tests  每次都跑：先 cargo build --release 编整个 research 工作区（复跑那一格的 replay.sh 编的就是这一份），再 cargo test --release，
#                        再跑几份研究脚本的 --selftest
#   experiment-replay    按输入复用：这一道读的路径（.claude/gate.d/stage-inputs.tsv 里本阶段那一行）相对上次整轮全绿那棵暂存树没变就退 77；
#                        派给 research/scripts/replay.sh 的是表里的 exact 行减掉慢的那几个
# 只编一次：两格编的都是 research/target 下的 release 产物。单测那一格先 cargo build --release 编好 replay.sh 要的那一份，
# 复跑那一格里 replay.sh 的 cargo build --release 就不再编（cargo 按指纹判新鲜）；只跑复跑那一格时由 replay.sh 自己编。
# replay.sh 里带 feature 的几个驱动（e162、e163）另编它们那一份，与单测那一格编的不是同一组特性，照旧各编一次。
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头）。
#
# 复跑那一格逐项可点名（.claude/rules/verification.md「门禁的结构」：重型的按实验点名）：
#   --list-items             逐行打 replay.sh 表里的实验号与它是 exact 还是 timing、在不在慢的那几个里，不编、不跑
#   --item <实验号>           复跑那一格只跑点名的实验（给几次取并集；timing 行与慢的也照跑）；不问整道的复用判定（那一问是整张表的），
#                            replay.sh 按实验自己的准入照旧判「输入没变」；成功行写明只跑了点名的几个，不是整格结论。
#                            实验号不在表里退 2、列出可点的号。只跑点名的实验：--check experiment-replay --item E142
#
# ── research-unit-tests：research 构建与单测、研究脚本的自证（原 research 构建与单测那一道）──
# **为什么要单开一条**：共享门禁的「构建与单测」阶段只看 `crates/`，
# 而本工程还没有 `crates/`，于是那一阶段恒报「项目尚无 Rust 代码，本阶段不适用」。
# 与此同时，**kb 里几乎每一条实测结论都由 `research/` 下那些实验二进制背书**
# （单测有多少条，看这一格成功行现数的那个数），而它们**从来没有被门禁碰过**。
#
# ⚠️ **实测踩过（2026-08-29）**：`Cargo.toml` 里留了一个指向已删源码的 `[[bin]]`，
# `cargo test` 直接报 `can't find bin`——**而门禁全绿**。
# 一个连编都编不过的证据仓库，比没有证据更糟：它看起来还在。
#
# 这里的 cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
# 谁跑这一道都一样，外面不再包一层。上限取 GATE_RESEARCH_BUILD_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。峰值表的键写死：光看「cargo test --release」分不出是哪个工作区。
# 包装自己的结局（退出码 250–254）不是构建与单测的判定，单独判红、单独给出路。cargo build 不跑编出来的代码，不经包装。
#
# ── experiment-replay：入库的实验数今天还复现得出来吗（原实验复跑那一道）──
# 判据：把纯模型实验重跑一遍，和 `research/results/` 里那份原始产物**逐字节**比对。
# 对不上不等于代码坏了，也可能是产物该更新了——两种都要人来判，所以判红。
#
# ⚠️ **默认跑表里所有 exact 行，减掉慢的那几个。** 慢的那几个（下面的 SLOW 清单里判定为 exact 的行）
# 与**表里判定为 timing 的每一行**都压在 `GATE_REPLAY_FULL=1` 后面，
# 且**这一格会把它们逐个列成「本次没跑」**——
# `.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」。
#
# ⚠️ **没跑的清单要从表里现算，不许只列 SLOW。** 2026-09-16 现查：默认档跑 119 个 exact 行，
# 而表里 9 个 timing 行一个都不跑，报告却只列了 E9 / E17 / E20 / E21——
# **E44 / E58 / E107 / E128 / E140 / E144 六个既没跑，也没出现在任何一行里**。
# 当轮把 `GATE_REPLAY_FULL=1` 补跑了一次才发现：E58 的产物早已对不上（源码的指针载荷
# 108 → 53 改过而产物没重跑），E17 在 release 下直接 panic 跑不起来——而默认档一直是绿的。
#
# 这次改动没碰这一格判的东西就退 77（本次未跑），不退 0（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。
# 它判的是入库实验数今天还复现不复现得出来，输入是实验二进制、replay.sh 的表与留存产物；
# research/prompts/ 是冻结证据，改它不可能改变任何实验的输出，所以不在前缀里。C8（范围判定）的粗粒度前身。
# 先问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv` 里本阶段那一行）
# 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
# 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。复用判定按本阶段的文件名查，
# 只问复跑那一格：单测那一格每次都跑，不因为输入没变而跳过。这一格从来不写全绿标记（原实验复跑那一道就不写），
# 按合并之前实验复跑那一道的文件名判过的复用不再作数，第一次按新名字判要跑满一遍。
#
# 样本：fixtures/checker-tier-research-build-and-replay.sh/ 下四份，各放 .gate-cells 只点名它判的那一格：
#   research-unit-tests-red    research 工作区里有一条单测红着的小仓（原 research 构建与单测那一道的 red），必须判红
#   research-unit-tests-green  research 工作区里一条单测、全绿；research/scripts/ 下放四份只认 --selftest 的替身（不是真脚本的拷贝，
#                              拷来的副本会与真脚本分叉），判绿，成功行报一个测试批次、4 份研究脚本的自证
#   green                      带假 replay.sh 的小仓（原实验复跑那一道的 green），表里有一行 exact 的快实验、一行 SLOW 里的 exact、一行 SLOW 里只有 timing 行的，
#                              快的那行要派给 replay.sh，另外两行都要逐个列进「没派」
#   red                        表一行都没有（原实验复跑那一道的 red），必须判红
# 汇总与 --check 的弄坏开关归共用库（STAGE_CELLS_BREAK=red-swallowed 让两份红样本整道退 0；marker-ignored 让每份样本两格都跑，
# research-unit-tests-green 里复跑那一格对着没有表的 replay.sh 判红）。
#
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh [项目根]                                两格都跑
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests [项目根]     只跑单测那一格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --list-items [项目根]                    逐行打可点名的实验号
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check experiment-replay --item E142 [项目根]   只复跑点名的实验
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
STAGE_REPOSITORY="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_NAME="$(basename "$0")"
source "$STAGE_DIR/lib/stage-cells.sh"

stage_cell research-unit-tests cell_research_unit_tests "research 工作区编得过、单测全绿、研究脚本的自证通过" \
  "先修好编不过或红着的单测（kb 里的实测结论全靠它们背书），研究脚本自证红了照它自己那句出路修；改完只重跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests"
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
  local research_selftests_passed=0 no_test_batches=0 out rc b n
  local MEMORY_CAP_RUNNER RESEARCH_BUILD_MEMORY_MAX OOV STAGE_MINE RELABEL CLAIM
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
  if (( rc >= 250 && rc <= 254 )); then
    tail -8 <<<"$out" | sed 's/^/     /'
    echo "  ✗ research 的构建与单测没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $rc（上限 $RESEARCH_BUILD_MEMORY_MAX），这一次的输出不算判定"
    echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一格要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；"
    echo "               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑。"
    echo "               253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑。"
    exit 1
  fi
  if (( rc != 0 )); then
    echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
    grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
    echo "     → 怎么办：先修好——kb 里的实测结论全靠它们背书，编不过就等于那些数字今天没有来源。"
    exit 1
  fi
  # 条数用一个 awk 数（「test result: ok. N passed; …」的第 4 段），不再串 sed、paste、bc：串里哪一环失败，条数就静默变成「?」
  b="$(grep -c '^test result: ok' <<<"$out")"
  n="$(awk '/^test result: ok\./ { passed += $4 } END { print passed + 0 }' <<<"$out")"
  [[ "$n" =~ ^[0-9]+$ ]] || n="没数出来（awk 输出「$n」）"
  # cargo test 退 0 而一个测试批次都没有：单测这一半没有对象可判，不记通过；
  # 下面几个脚本的自检照跑（它们红了照判红），跑完退 77
  if (( b == 0 )); then
    no_test_batches=1
    echo "  ! 本次无对象可判：cargo test 退 0，但输出里一个「test result: ok」批次都没有，research 的单测这一半没判"
  else
    echo "  ✓ research 构建通过，$b 个测试批次、共 $n 个单测全绿"
  fi

  # 下面几个脚本的自检：脚本找不到就判红——它搬了家或改了名，自检就静默不跑了，而成功行一个字都不提
  missing_selftest_target() {
    echo "  ✗ 找不到 $1：它的自检这一次没跑"
    echo "     → 怎么办：它搬了家或改了名，就把这一格里的路径一起改（.claude/rules/path-moves.md）；"
    echo "               它被删了，就把这一格里它那一段自检一起删。别让自检静默跳过。"
    exit 1
  }

  # 本地腿的字词损坏闸，它自己会不会红：三方论证的本地腿靠 `ask-local.sh` 里那道闸挡损坏输出，而那道闸此前没人验过。
  # 实测（2026-09-03）：一份含 `inaccessibleisabled` 的输出被判绿，差点当成证据用掉。
  OOV="$R/scripts/oov-check.py"
  if [[ -f "$OOV" ]]; then
    if ! out="$(python3 "$OOV" --selftest 2>&1)"; then
      echo "$out" | sed 's/^/  /'
      echo "     → 怎么办：本地腿的损坏闸判错了样本，修 splice_of 的规则再跑。"
      echo "                闸不准 ⇒ 三方论证里那一腿的输出可信度归零。"
      exit 1
    fi
    echo "$out" | sed 's/^  /  /'
    research_selftests_passed=$((research_selftests_passed + 1))
  else
    missing_selftest_target "$OOV"
  fi

  # 并发会话的暂存工具，它自己会不会红：stage-mine.py 挑错块，别的会话的半成品就被卷进这一次提交。
  STAGE_MINE="$R/scripts/stage-mine.py"
  if [[ -f "$STAGE_MINE" ]]; then
    if ! out="$(python3 "$STAGE_MINE" --selftest 2>&1)"; then
      echo "$out" | sed 's/^/  /'
      echo "     → 怎么办：stage-mine.py 挑块挑错了样本，修切块或下滑的规则再跑。"
      exit 1
    fi
    echo "  ✓ stage-mine.py $(tail -1 <<<"$out")"
    research_selftests_passed=$((research_selftests_passed + 1))
  else
    missing_selftest_target "$STAGE_MINE"
  fi
  RELABEL="$R/scripts/relabel-item.py"
  if [[ -f "$RELABEL" ]]; then
    if ! out="$(python3 "$RELABEL" --selftest 2>&1)"; then
      echo "$out" | sed 's/^/  /'
      echo "     → 怎么办：relabel-item.py 改写之后 doc-decisions 格「分项引用状态」的库复判不过，修它的归属或改写再跑。"
      exit 1
    fi
    echo "  ✓ relabel-item.py $(tail -1 <<<"$out")"
    research_selftests_passed=$((research_selftests_passed + 1))
  else
    missing_selftest_target "$RELABEL"
  fi
  CLAIM="$R/scripts/claim-experiment.sh"
  if [[ -f "$CLAIM" ]]; then
    if ! out="$(bash "$CLAIM" --selftest 2>&1)"; then
      echo "$out" | sed 's/^/  /'
      echo "     → 怎么办：claim-experiment.sh 放过了已用的号、或同一个号占了两次，修 used_numbers / claim 再跑。"
      exit 1
    fi
    echo "  ✓ claim-experiment.sh $(tail -1 <<<"$out")"
    research_selftests_passed=$((research_selftests_passed + 1))
  else
    missing_selftest_target "$CLAIM"
  fi
  # 单测那一半没有对象（见上面「本次无对象可判」那一行）：自检都绿也不记通过
  if (( no_test_batches )); then exit 77; fi
  echo "  ✓ research 构建与单测全绿，研究脚本的自证过了 $research_selftests_passed 份"
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
  # ⚠️ **不许写死清单。** 第一版把 FAST 手抄成 22 个编号，此后新增的实验
  # （E40 起共 14 个）虽然进了 replay.sh 的表，却**一个都没被门禁跑过**，
  # 而本阶段仍旧打印「本次没跑：E9 E17 E20 E21」——**列出来的那句话本身是假的**。
  # 这正是 `.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」要拦的形态：
  # 漏跑不可怕，漏跑而且报告说跑了才可怕。
  # ⇒ 改成从 replay.sh 的表里现算：取判定为 exact 的行，减掉慢的那几个。
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
  # replay.sh 汇总行报「输入没变没跑 N」并逐个列名；这里不再说成「跑了」。
  echo "  ! 本次派给 replay.sh ${#FAST[@]} 个（表里的 exact 行减掉慢的；其中准入判「输入没变」而没跑的，见上面 replay.sh 汇总里那一行）；没派 ${#SKIPPED[@]} 个：${SKIPPED[*]}"
  echo "    标（慢）的是 SLOW 清单（${SLOW[*]}）里判定为 exact 的行；标（计时）的是表里判定为 timing 的行，默认档一行都不跑。"
  echo "    跑： GATE_REPLAY_FULL=1 bash .claude/scripts/gate.sh，或逐个点名：bash .claude/gate.d/${STAGE_NAME} --check experiment-replay --item <实验号>"
  echo "    ⚠️ 它们的结论靠 replay.sh 里的区间断言钉住，不跑就等于那几条断言本轮没验。"
}

stage_cells_run "$ROOT"
