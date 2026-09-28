#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 模型对拍那几格要 cargo：缺了就在格里那一步 cargo test 判红，不交给 gate.sh 预判（用户 2026-09-26 定：项目更严）；此外只要跑门禁本身就要的 bash、git、python3
# gate-stage: harness 类：harness 档一条用例一个场景；模型对拍（随机历史快档与五个取样点，每一步拿实现的结局与只住内存的理想模型比，每段一格、各点名一条用例）
# gate-category: harness 类
# gate-covers: 模型对拍
# gate-similar: code-source-discipline.sh 它的 test-file-names 格也调 crash-case-check.py，但调的是 file-names 那一样、判测试文件名，归代码类；这里判用例里面的循环与模型对拍
# gate-similar: 54-layer0-replay.sh 开跑前也调 crash-case-check.py，但调的是 placement 与 modules 两样（崩溃枚举用例住哪、登记没有，checker 档测试文件声明模块），它跑的是 checker 档的崩溃点重放，归 checker-tier 类；这里只判 harness 档
# gate-similar: code-tooling.sh 它的 research-script-selftests 格只跑 crash-case-check.py 与 stage-run-or-skip.sh 的 --selftest（判别力），不在真仓上判；这里在真仓上判
# gate-similar: checker-tier-crates-mutation-replay.sh 它按 crates/mutations.tsv 把点名 random_histories 的变异逐条施加再跑，证明模型对拍自己会红；这里在不改代码的仓上判模型对拍跑了、每段都判过
# gate-similar: checker-tier-research-build-and-replay.sh 也经内存包装跑 cargo test --release，但编的是 research 工作区的实验，不跑 crates/singlefs-harness 的测试二进制
# gate-similar: checker-independence-and-sync.sh 它的 checker-implementation-disjoint 格判三个包之间的依赖方向与共享代码，对象是 Cargo.toml 与 use 语句；这里判的是 harness 档用例本身的粒度与对拍
# gate-similar: harness-test-environment.sh 同属 harness 类，判的是跑完测试之后机器干不干净，读宿主状态，不读测试源码也不跑 cargo
#
# 七格：一条用例一个场景，与模型对拍六段（每段一格）。模型对拍原先整道跑一个 cargo test、不带 --include-ignored，
# 而那六段的用例都标了 #[ignore]（harness 耗时用例），一段都没跑到（判决 research/prompts/gate-shrink-r1-main-verification.md
# 「逐问判决」3-B4）；现在每格带 --include-ignored --exact 点名跑它那一条用例。
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就七格都跑，格名写错退 2）：
# gate-cell: one-scenario harness 档一条用例一个场景
# gate-cell: model-fast-tier 模型对拍：随机历史快档
# gate-cell: model-reuse-sampling 模型对拍：偏向抬 F 之后复用的取样点
# gate-cell: model-rollback-sampling 模型对拍：偏向抬 F 之后回退的取样点
# gate-cell: model-allocation-record-wall 模型对拍：越过原分配记录墙的取样点
# gate-cell: model-unit-area-wall 模型对拍：小盘上逼近单元区墙的取样点
# gate-cell: model-unit-area-wall-admission-judged 模型对拍：小盘上逼近单元区墙的取样点（空间准入判着）
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头），
# 这里只登记格、解析项目根。
#
# ── one-scenario：harness 档一条用例一个场景 ──
# 判据：research/scripts/crash-case-check.py --only one-scenario <项目根>：crates/singlefs-harness/tests/*.rs 里带 #[test] 的函数，
# 某个 for 循环体每一轮新建一个池（build_pool、build_through_*、format_pool、MemoryPool::with_devices），判红；
# 用例上面写了「// harness-test-granularity:one-scenario <理由>」的不判。判法只在那一份脚本里。harness 档 tests/ 下一条用例都没有：这一格退 77。
# 管不到的：一条用例里不建池的多场景（改的是同一个池）；harness 耗时用例标得对不对（轻重看平常跑的时候量到的单条用时，靠人）；
# 认不出的写法照 crash-case-check.py 文件头。
# 弄坏开关：CRASH_CASE_CHECK_BREAK=fresh-pool-loop-unseen 让循环里新建池认不出，one-scenario-red 退 0、样本判错。
#
# ── model-*：模型对拍六段 ──
# 被测的是 `crates/singlefs-harness/tests/random_histories.rs` 里调 `run_history_campaign`、种子区间与步数写死、把报告经 `print_uncaptured`
# 写进标准输出的那六条用例：快档与五个偏向某种历史的取样点（落到墙边的那几段每一步之后也跑池级 checker，已知红第 0 条那一形只记不停）。
# 每段的报告里有一行「模型对拍 N 步：…」（`crates/singlefs-harness/src/history.rs` 的报告渲染），
# 模型模块 `crates/singlefs-harness/src/model.rs` 只用 `singlefs_format` 的常量（D13（验证路线） 已定项 5）。
# 点的是这六条、不是别的：同一个测试二进制里另外两处也打报告，都不是门禁该跑的——大档
# `random_histories_large_tier_seeds_and_length_from_the_environment` 的种子数、步数、比重全从环境变量取（release 下后台跑），
# `shrink_one_failing_seed_from_the_environment` 要 SINGLEFS_RANDOM_HISTORY_SHRINK_SEED 必给；其余用例不经 run_history_campaign、不打「模型对拍」那一行。
# 格、段标题与用例名的对照（段标题是用例里 print_uncaptured 打的「── … ──」那一行；改名时两边一起改）：
#   model-fast-tier                       随机历史快档                                       random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
#   model-reuse-sampling                  随机历史：偏向抬 F 之后复用的取样点                 reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms
#   model-rollback-sampling               随机历史：偏向抬 F 之后回退的取样点                 rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms
#   model-allocation-record-wall          随机历史：越过原分配记录墙的取样点                   allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall
#   model-unit-area-wall                  随机历史：小盘上逼近单元区墙的取样点                 unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device
#   model-unit-area-wall-admission-judged 随机历史：小盘上逼近单元区墙的取样点（空间准入判着） unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval
# 每格在 release 下跑 `cargo test --release -p singlefs-harness --test random_histories -- --include-ignored --exact <用例名> --nocapture`
# （第一格编译，后面几格用同一份编译产物），要求它那一段打出「模型对拍」那一行、步数大于 0——测试绿而模型一步没判，等于没对拍。
# 点的名字在源码里找不到时 cargo 一条都不跑、照样退 0：那一段的「模型对拍」行找不到、判红，输出里另说明「一条用例都没跑」。
# 找段标题认两种排法：标题独占一行（libtest 多线程时的样子），或跟在「test <名字> ... 」后面（RUST_TEST_THREADS=1 时 libtest 先打用例名再跑）。
# 判别力：模型对拍自己会不会红，由 `crates/mutations.tsv` 里点名这个测试二进制的那几条变异（crates 变异复跑那一道门禁）证明；这几格只判「跑了、判过、没报对不上」。
# 要不要跑：每格先问能不能复用上一次整轮全绿的判定（这一道读的那几条路径登记在 `.claude/gate.d/stage-inputs.tsv`，键是这份文件名），
# 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变；再问这次改动碰没碰 crates/。可跳过的格退 77（本次未跑），不退 0——
# `exit 0` 的跳过在汇总里与「判过了」一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。one-scenario 那一格每次都判，不问复用。
# 为什么用树、为什么只一条 ref、为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
# cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
# 谁跑这一道都一样，外面不再包一层。上限取 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。包装自己的结局（退出码 250–254）不是测试的判定，单独判红、单独给出路。
# 样本：被判目录里没有 Cargo.toml 而有 `model-differential-cargo-output.log` 时，不跑 cargo，只判那份录好的输出里这一格那一段，
# 并打出本该跑的那条 cargo 命令（样本的 want 核它带 --include-ignored --exact 与用例名）。两样都没有：没有 crates/singlefs-harness，这几格退 77。
# 弄坏开关 GATE_MODEL_DIFFERENTIAL_BREAK=<项>，每一项都让某份样本判错：
#   ignored-left-out   cargo 命令去掉 --include-ignored（改之前的写法）：model-differential-green 里核命令的那几条 want 找不到
#   zero-steps-passed  步数为 0 的段按判过记：model-reuse-sampling-red、model-allocation-record-wall-red 退 0
#
# 样本（fixtures/harness-model-differential-and-scenarios.sh/，每份的 .gate-cells 点名它判的格）：
#   one-scenario-red                        一条每轮新建池的 harness 用例
#   one-scenario-green                      一条写了 one-scenario 豁免的循环用例与两条普通用例
#   model-differential-green                六段都打了「模型对拍 N 步」、步数都大于 0（六格一起点名，六格都绿；另核打出的 cargo 命令带 --include-ignored --exact 与用例名）
#   model-fast-tier-red                     快档那一段没有「模型对拍」那一行（录的是多条用例一起跑的输出，另有两段也缺，只点名这一格）
#   model-fast-tier-renamed-red             点的用例名在源码里找不到，cargo 一条都没跑（running 0 tests）
#   model-reuse-sampling-red                复用那一段模型对拍 0 步
#   model-rollback-sampling-red             回退那一段没有「模型对拍」那一行，紧跟着的快档那一段有：别段的行不许算进这一段
#   model-allocation-record-wall-red        分配记录墙那一段模型对拍 0 步，段标题跟在「test <名字> ... 」后面（RUST_TEST_THREADS=1 的排法）
#   model-unit-area-wall-red                只有「空间准入判着」那一段：标题多出一截的那一段不许被认成这一格那一段
#   model-unit-area-wall-admission-judged-red  只有空间准入关着的那一段，没有「空间准入判着」那一段
#   model-unit-area-wall-sequential-green   段标题跟在「test <名字> ... 」后面，模型对拍 4749 步，判绿
# 汇总与 --check 的弄坏开关归共用库（STAGE_CELLS_BREAK=red-swallowed 让各 red 样本退 0）。
#
#   bash .claude/gate.d/harness-model-differential-and-scenarios.sh [项目根]                          七格都跑
#   bash .claude/gate.d/harness-model-differential-and-scenarios.sh --list                            逐行打格名与判什么，不跑格
#   bash .claude/gate.d/harness-model-differential-and-scenarios.sh --check <格名>[,<格名>…] [项目根]   只跑点名的格（例：--check model-fast-tier 只跑快档那一条用例）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_NAME="$(basename "$0")"
source "$STAGE_DIR/lib/stage-cells.sh"

CRASH_CASE_CHECK="$REPO_ROOT/research/scripts/crash-case-check.py"
TEST_BINARY="random_histories"
declare -A MODEL_SECTION=(
  [model-fast-tier]="随机历史快档"
  [model-reuse-sampling]="随机历史：偏向抬 F 之后复用的取样点"
  [model-rollback-sampling]="随机历史：偏向抬 F 之后回退的取样点"
  [model-allocation-record-wall]="随机历史：越过原分配记录墙的取样点"
  [model-unit-area-wall]="随机历史：小盘上逼近单元区墙的取样点"
  [model-unit-area-wall-admission-judged]="随机历史：小盘上逼近单元区墙的取样点（空间准入判着）"
)
declare -A MODEL_TEST=(
  [model-fast-tier]="random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation"
  [model-reuse-sampling]="reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms"
  [model-rollback-sampling]="rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms"
  [model-allocation-record-wall]="allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall"
  [model-unit-area-wall]="unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device"
  [model-unit-area-wall-admission-judged]="unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval"
)
MODEL_HOWTO_TAIL="模型对拍的判据与格表写在 .claude/gate.d/$STAGE_NAME 文件头；签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样，先看模型那一格引的条款，再判改实现还是改模型；用例改了名就把文件头格表与 MODEL_TEST 里那个名字一起改"

stage_cell one-scenario cell_one_scenario "harness 档一条用例一个场景" \
  "照上面 crash-case-check 那一句出路改：循环里每一轮新建一个池的，拆成带参数的函数加每个取值一条 #[test]；判据写在 research/scripts/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「harness 档里再分轻用例与耗时用例」"
stage_cell model-fast-tier cell_model_fast_tier "模型对拍：随机历史快档" \
  "单跑这一条看细节：bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-fast-tier]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-reuse-sampling cell_model_reuse_sampling "模型对拍：偏向抬 F 之后复用的取样点" \
  "单跑这一条看细节：bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-reuse-sampling]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-rollback-sampling cell_model_rollback_sampling "模型对拍：偏向抬 F 之后回退的取样点" \
  "单跑这一条看细节：bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-rollback-sampling]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-allocation-record-wall cell_model_allocation_record_wall "模型对拍：越过原分配记录墙的取样点" \
  "单跑这一条看细节：bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-allocation-record-wall]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-unit-area-wall cell_model_unit_area_wall "模型对拍：小盘上逼近单元区墙的取样点" \
  "单跑这一条看细节：bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-unit-area-wall]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-unit-area-wall-admission-judged cell_model_unit_area_wall_admission_judged "模型对拍：小盘上逼近单元区墙的取样点（空间准入判着）" \
  "单跑这一条看细节：bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-unit-area-wall-admission-judged]} --nocapture；$MODEL_HOWTO_TAIL"
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

# ── one-scenario ──
cell_one_scenario() {
  local scenario_exit harness_test_file_count
  if python3 "$CRASH_CASE_CHECK" --only one-scenario "$ROOT"; then scenario_exit=0; else scenario_exit=$?; fi
  case "$scenario_exit" in
    0)
      harness_test_file_count="$(find crates/singlefs-harness/tests -maxdepth 1 -name '*.rs' 2>/dev/null | wc -l)"
      echo "  ✓ harness 用例粒度这一格通过（one-scenario，扫了 $harness_test_file_count 份 harness 档测试文件，用例条数在上面那一行）" ;;
    77)
      echo "  ! crates/singlefs-harness/tests/ 下一条用例都没有：这一格本次无对象可判（不记通过）"
      exit 77 ;;
    *)
      echo "  ✗ harness 用例粒度：crash-case-check --only one-scenario 退 $scenario_exit（逐处列在上面）"
      echo "     → 怎么办：照上面那一句出路改；判据写在 research/scripts/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「harness 档里再分轻用例与耗时用例」。"
      exit 1 ;;
  esac
}

# ── model-*：跑点名的那一条用例（或读录好的输出），判它那一段打出「模型对拍 N 步」、N 大于 0 ──
model_differential_cell() {
  local cell_name="$1"
  local section="${MODEL_SECTION[$cell_name]}" test_name="${MODEL_TEST[$cell_name]}"
  local memory_cap_runner="$REPO_ROOT/research/scripts/run-with-memory-cap.sh"
  local memory_max="${GATE_MODEL_DIFFERENTIAL_MEMORY_MAX:-8G}"
  local cargo_exit line steps sections_checked=0
  local cargo_command=()

  source "$REPO_ROOT/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一格要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一格按红记。"; exit 1; }
  stage_run_or_skip "$REPO_ROOT/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$STAGE_NAME"
  stage_run_or_skip "$REPO_ROOT/research/scripts/change-touches-crates.sh" "这次改动没碰它判的东西" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；判据与前缀见 research/scripts/change-touches-crates.sh。" -- "$ROOT" crates/

  if [[ ! "$memory_max" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    echo "  ✗ GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 写成了「$memory_max」，内存包装不认（或认成字节数）：这一格没跑"
    echo "     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G"
    exit 1
  fi
  cargo_command=(cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --include-ignored --exact "$test_name" --nocapture)
  if [[ "${GATE_MODEL_DIFFERENTIAL_BREAK:-}" == ignored-left-out ]]; then
    cargo_command=(cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --exact "$test_name" --nocapture)
  fi

  log="$(mktemp)"
  trap 'rm -f "$log"' EXIT
  if [[ -f Cargo.toml && -d crates/singlefs-harness ]]; then
    echo "  · 跑：${cargo_command[*]}（经内存包装，上限 $memory_max）"
    if bash "$memory_cap_runner" "$memory_max" "${cargo_command[@]}" >"$log" 2>&1; then
      cargo_exit=0
    else
      cargo_exit=$?
    fi
    if ((cargo_exit != 0)); then
      tail -40 "$log"
      case "$cargo_exit" in
        250|251|252|253|254)
          echo "  ✗ 「$section」那条用例没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $cargo_exit（上限 $memory_max），这一次的输出不算判定"
          echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一格要的比上限多，把 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 设大再跑；"
          echo "                251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑。"
          echo "                253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑。"
          ;;
        *)
          echo "  ✗ 「$section」那条用例判红（上面是 cargo test 的尾部）"
          echo "     → 怎么办：单跑看细节（经内存包装，上限同这一格）："
          echo "                bash research/scripts/run-with-memory-cap.sh $memory_max ${cargo_command[*]}"
          echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
          ;;
      esac
      exit 1
    fi
  elif [[ -f model-differential-cargo-output.log ]]; then
    echo "  · 样本：录好的输出 model-differential-cargo-output.log 代替 ${cargo_command[*]}"
    cp model-differential-cargo-output.log "$log"
  else
    echo "  ! 没有 crates/singlefs-harness，这一格本次无对象可判（增补 3 第 2 件之前没有模型）"
    exit 77
  fi

  # 一段的报告从它的标题行起、到下一个「── … ──」标题行为止；不按固定行数取，报告长了照样找得到。
  # 标题行认独占一行与跟在「test <名字> ... 」后面两种（文件头「找段标题认两种排法」）。
  line="$(awk -v header="── ${section} ──" '
    !inside && ($0 == header || (length($0) > length(header) && substr($0, length($0) - length(header) + 1) == header && index($0, "test ") == 1)) { inside = 1; next }
    inside && /^── .* ──$/ { exit }
    inside' "$log" | grep -m1 '模型对拍 [0-9]* 步' || true)"
  if [[ -z "$line" ]]; then
    echo "  ✗ 测试跑过了，这一段却没有「模型对拍 N 步」那一行：「$section」（用例 $test_name）"
    if grep -qE '^running 0 tests|^test result: ok\. 0 passed' "$log"; then
      echo "     cargo 这一次一条用例都没跑：点的名字 $test_name 在 tests/$TEST_BINARY.rs 里找不到（--exact 要整名相同）"
    fi
    echo "     → 怎么办：用例改了名就把这份文件头的格表与 MODEL_TEST 里那个名字一起改；名字对得上的，history.rs 的报告渲染要在每段报告里打「模型对拍 N 步：…」，"
    echo "                测试文件里每一段都要经 print_uncaptured 把报告写进标准输出；行没了多半是执行器绕开了 judge_by_model，那一段等于没对拍。"
    exit 1
  fi
  steps="$(sed -n 's/.*模型对拍 \([0-9]*\) 步.*/\1/p' <<<"$line")"
  if [[ -z "$steps" || "$steps" == 0 ]] && [[ "${GATE_MODEL_DIFFERENTIAL_BREAK:-}" != zero-steps-passed ]]; then
    echo "  ✗ 这一段模型一步都没判：「$section」（用例 $test_name）"
    echo "     → 怎么办：测试绿而模型没判过任何一步，等于没对拍（show-me-test.md「扫到 0 项也不是通过」）。"
    echo "                看 history.rs 的执行器每一步调入口之前有没有先问模型（judge_by_model），前提不满足的步不算。"
    exit 1
  fi
  sections_checked=$((sections_checked + 1))
  echo "  ✓ 模型对拍这一段判过、实现与模型没有对不上的（查了 $sections_checked 段，用例 $test_name）："
  echo "      ${section}：$(sed 's/^[[:space:]]*//' <<<"$line")"
}

cell_model_fast_tier() { model_differential_cell model-fast-tier; }
cell_model_reuse_sampling() { model_differential_cell model-reuse-sampling; }
cell_model_rollback_sampling() { model_differential_cell model-rollback-sampling; }
cell_model_allocation_record_wall() { model_differential_cell model-allocation-record-wall; }
cell_model_unit_area_wall() { model_differential_cell model-unit-area-wall; }
cell_model_unit_area_wall_admission_judged() { model_differential_cell model-unit-area-wall-admission-judged; }

stage_cells_run "$ROOT"
