#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 模型对拍那几格要 cargo：缺了就在格里那一步 cargo test 判红，不交给 gate.sh 预判；此外只要跑门禁本身就要的 bash、git、python3
# gate-stage: harness 类：harness 档一条用例一个场景；模型对拍（随机历史快档与五个取样点，每一步拿实现的结局与只住内存的理想模型比，每段一格、各点名一条用例）
# gate-fixture: yes
# gate-category: harness 类
# gate-covers: 模型对拍
# gate-similar: code-source-discipline.sh 它的 test-file-names 格也调 crash-case-check.py，但调的是 file-names 那一样、判测试文件名，归代码类；这里判用例里面的循环与模型对拍
# gate-similar: checker-tier-crash-point-replay.sh 开跑前也调 crash-case-check.py，但调的是 placement 与 modules 两样（崩溃枚举用例住哪、登记没有，checker 档测试文件声明模块），它跑的是 checker 档的崩溃点重放，归 checker-tier 类；这里只判 harness 档
# gate-similar: code-tooling.sh 它的 research-script-selftests 格只跑 crash-case-check.py 的 --selftest（判别力），不在真仓上判；这里在真仓上判
# gate-similar: checker-tier-crates-mutation-replay.sh 它按 crates/mutations.tsv 把点名 random_histories 的变异逐条施加再跑，证明模型对拍自己会红；这里在不改代码的仓上判模型对拍跑了、每段都判过
# gate-similar: checker-tier-research-build-and-replay.sh 也经内存包装跑 cargo test --release，但编的是 research 工作区的实验，不跑 crates/singlefs-harness 的测试二进制
# gate-similar: checker-independence-and-sync.sh 它的 checker-implementation-disjoint 格判三个包之间的依赖方向与共享代码，对象是 Cargo.toml 与 use 语句；这里判的是 harness 档用例本身的粒度与对拍
# gate-similar: harness-test-environment.sh 同属 harness 类，判的是跑完测试之后机器干不干净，读宿主状态，不读测试源码也不跑 cargo
#
# gate-cell: one-scenario harness 档一条用例一个场景
# gate-cell: model-fast-tier 模型对拍：随机历史快档
# gate-cell: model-reuse-sampling 模型对拍：偏向抬 F 之后复用的取样点
# gate-cell: model-rollback-sampling 模型对拍：偏向抬 F 之后回退的取样点
# gate-cell: model-allocation-record-wall 模型对拍：越过原分配记录墙的取样点
# gate-cell: model-unit-area-wall 模型对拍：小盘上逼近单元区墙的取样点
# gate-cell: model-unit-area-wall-admission-judged 模型对拍：小盘上逼近单元区墙的取样点（空间准入判着）
# 格结构经 lib/stage-cells.sh。
#
# ── one-scenario ──
# 判法只在 scripts/crash/crash-case-check.py --only one-scenario：crates/singlefs-harness/tests/*.rs 里带 #[test] 的函数，
# 某个 for 循环体每一轮新建一个池，判红；用例上面写了「// harness-test-granularity:one-scenario <理由>」的不判。一条用例都没有：退 77。
# 管不到的：一条用例里不建池的多场景（改的是同一个池）；慢档用例标得对不对。
# 弄坏开关：CRASH_CASE_CHECK_BREAK=fresh-pool-loop-unseen 让循环里新建池认不出，one-scenario-red 退 0。
#
# ── model-*：模型对拍六段 ──
# 被测的是 crates/singlefs-harness/tests/random_histories.rs 里调 run_history_campaign、经 print_uncaptured 把报告写进标准输出的六条用例，
# 每段报告里有一行「模型对拍 N 步：…」（crates/singlefs-harness/src/history.rs 的报告渲染）。
# 同一个二进制里的大档与 shrink 用例参数从环境变量取，门禁不跑。
# 格、段标题与用例名（段标题是用例里 print_uncaptured 打的「── … ──」那一行；改名时两边一起改）：
#   model-fast-tier                       随机历史快档                                       random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
#   model-reuse-sampling                  随机历史：偏向抬 F 之后复用的取样点                 reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms
#   model-rollback-sampling               随机历史：偏向抬 F 之后回退的取样点                 rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms
#   model-allocation-record-wall          随机历史：越过原分配记录墙的取样点                   allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall
#   model-unit-area-wall                  随机历史：小盘上逼近单元区墙的取样点                 unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall
#   model-unit-area-wall-admission-judged 随机历史：小盘上逼近单元区墙的取样点（空间准入判着） unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval
# 每格跑 `cargo test --release -p singlefs-harness --test random_histories -- --include-ignored --exact <用例名> --nocapture`
# （六条都标了 #[ignore]，不带 --include-ignored 一段都跑不到），要那一段打出「模型对拍」那一行、步数大于 0。
# 点的名字在源码里找不到时 cargo 一条都不跑、照样退 0：那一段的行找不到，判红。
# 找段标题认两种排法：标题独占一行，或跟在「test <名字> ... 」后面（RUST_TEST_THREADS=1 的排法）。
# 模型对拍自己会不会红由 crates/mutations.tsv 里点名这个测试二进制的变异证明；这几格只判跑了、判过、没报对不上。
# 复用：六格各是一条判定，键是那条用例的测试函数的单元键（单元闭包、release 档、工具链、解析出的依赖版本；判法是这份文件里的 model_section_verdict，
#   在这份门禁所在的仓里算闭包；cargo 命令的样子不带路径进一行 line:；算法在 scripts/gate/admission.py 文件头「单元键」）。键相同就不起 cargo、报「复用」；
#   判绿的照键记进判定库（<git common-dir>/singlefs-unit-verdicts/<这份文件名>:<格名>/），改一格用例自己闭包里的函数只作废那一格；拒算的照跑、不记库。
#   每格末行报「单元键：现判 N、复用 M、拒算 R、本次未跑 K」。
#   按条复用失灵（verdict-plan 退非 0、退 0 而没打 PLAN、它或 verdict-record 的复用哨兵报「复用失灵」「判定不稳」「强制现判没起作用」）：判法只在
#   scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」）；它判失灵，那一格照旧现判跑完、结论判红。不比树、不比 HEAD，不问 stage-must-run.sh 与 change-touches-crates.sh；
#   stage-inputs.tsv 本阶段那一行只登记入口，不进键。one-scenario 每次都判（只读用例源码，不起 cargo）。
# 读的环境变量：GATE_MODEL_DIFFERENTIAL_MEMORY_MAX，cargo test 经 scripts/hosts/run-with-memory-cap.sh 跑的内存上限，没设取 8G；
# 谁跑这一道外面都不再包一层。包装退 245 或 250–254 不是测试的判定，单独判红、单独给出路。
# 被判目录里没有 Cargo.toml 而有 model-differential-cargo-output.log 时不跑 cargo、不算键，只判录好的输出里这一段，并打出本该跑的 cargo 命令；
# 两样都没有：这几格退 77。被判目录里有 .model-differential-sample-tools/ 时是样本档：把它放 PATH 最前面（里面的假 cargo 照 cases/<用例名>.log 打输出，
# 假 rustc 打版本），不经内存包装与 capped.sh；另有 .unit-verdict-sample/prior/（上一版的树）时，每格先拿上一版算键、照判绿记进这一版样本的判定库，再照常查库。
# 弄坏开关 GATE_MODEL_DIFFERENTIAL_BREAK=<项>：
#   ignored-left-out   cargo 命令去掉 --include-ignored：model-differential-green 里核命令的那几条 want 找不到
#   zero-steps-passed  步数为 0 的段按判过记：model-reuse-sampling-red、model-allocation-record-wall-red 退 0
#   threads-not-defaulted  cargo 命令不经 scripts/hosts/capped.sh default（改前的样子，吃 cargo 自己的缺省）：model-differential-green 里核「经 capped.sh default 起」的那条 want 找不到
#   one-key-for-every-cell  六格共用一份输入（每格的键另加整份 random_histories.rs）：model-unit-key-one-cell 里「复用」那五句找不到（只改了一格的函数，六格一起作废）
# 共用库的 STAGE_CELLS_BREAK=red-swallowed 让各 red 样本退 0。
# admission.py 的 ADMISSION_BREAK=plan-failure-not-red（按条复用失灵也不判红）让 model-fast-tier-plan-exit-red、model-fast-tier-plan-sentinel-red 退 0。
#
# 样本（fixtures/harness-model-differential-and-scenarios.sh/，每份的 .gate-cells 点名它判的格）：
#   one-scenario-red                        一条每轮新建池的 harness 用例
#   one-scenario-green                      一条写了 one-scenario 豁免的循环用例与两条普通用例
#   model-differential-green                六段都打了「模型对拍 N 步」、步数都大于 0，六格都绿；另核打出的 cargo 命令带 --include-ignored --exact 与用例名
#   model-fast-tier-red                     快档那一段没有「模型对拍」那一行（另有两段也缺，只点名这一格）
#   model-fast-tier-renamed-red             点的用例名在源码里找不到，cargo 一条都没跑（running 0 tests）
#   model-reuse-sampling-red                复用那一段模型对拍 0 步
#   model-rollback-sampling-red             回退那一段没有「模型对拍」那一行，紧跟着的快档那一段有：别段的行不许算进这一段
#   model-allocation-record-wall-red        分配记录墙那一段模型对拍 0 步，段标题跟在「test <名字> ... 」后面
#   model-unit-area-wall-red                只有「空间准入判着」那一段：标题多出一截的那一段不许被认成这一格那一段
#   model-unit-area-wall-admission-judged-red  只有空间准入关着的那一段，没有「空间准入判着」那一段
#   model-unit-area-wall-sequential-green   段标题跟在「test <名字> ... 」后面，模型对拍 4749 步，判绿
#   model-unit-key-one-cell                 按格复用：一份小工作区，六条用例各调自己的函数；setup.sh 照这一版拷一份上一版进 .unit-verdict-sample/prior/、只改回
#                                           复用取样点那一格调的函数：只有 model-reuse-sampling 现判（假 cargo 打那一段），另五格复用、不起 cargo，六格都绿
#   model-fast-tier-plan-exit-red           按条复用失灵：.unit-verdict-sample/plan-failure 写 exit-1（verdict-plan 退 1），快档那一格照旧现判、那一段判过，结论判红
#   model-fast-tier-plan-sentinel-red       同形，写 sentinel-reuse-failed（verdict-plan 的复用哨兵报复用失灵），判红
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

CRASH_CASE_CHECK="$REPO_ROOT/scripts/crash/crash-case-check.py"
ADMISSION="$REPO_ROOT/scripts/gate/admission.py"
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
  [model-unit-area-wall]="unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall"
  [model-unit-area-wall-admission-judged]="unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval"
)
MODEL_HOWTO_TAIL="模型对拍的判据与格表写在 .claude/gate.d/$STAGE_NAME 文件头；签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样，先看模型那一格引的条款，再判改实现还是改模型；用例改了名就把文件头格表与 MODEL_TEST 里那个名字一起改"

stage_cell one-scenario cell_one_scenario "harness 档一条用例一个场景" \
  "照上面 crash-case-check 那一句出路改：循环里每一轮新建一个池的，拆成带参数的函数加每个取值一条 #[test]；判据写在 scripts/crash/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「harness 档里再分快档用例与慢档用例」"
stage_cell model-fast-tier cell_model_fast_tier "模型对拍：随机历史快档" \
  "单跑这一条看细节：bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-fast-tier]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-reuse-sampling cell_model_reuse_sampling "模型对拍：偏向抬 F 之后复用的取样点" \
  "单跑这一条看细节：bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-reuse-sampling]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-rollback-sampling cell_model_rollback_sampling "模型对拍：偏向抬 F 之后回退的取样点" \
  "单跑这一条看细节：bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-rollback-sampling]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-allocation-record-wall cell_model_allocation_record_wall "模型对拍：越过原分配记录墙的取样点" \
  "单跑这一条看细节：bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-allocation-record-wall]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-unit-area-wall cell_model_unit_area_wall "模型对拍：小盘上逼近单元区墙的取样点" \
  "单跑这一条看细节：bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-unit-area-wall]} --nocapture；$MODEL_HOWTO_TAIL"
stage_cell model-unit-area-wall-admission-judged cell_model_unit_area_wall_admission_judged "模型对拍：小盘上逼近单元区墙的取样点（空间准入判着）" \
  "单跑这一条看细节：bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --include-ignored --exact ${MODEL_TEST[model-unit-area-wall-admission-judged]} --nocapture；$MODEL_HOWTO_TAIL"
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
      echo "     → 怎么办：照上面那一句出路改；判据写在 scripts/crash/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「harness 档里再分快档用例与慢档用例」。"
      exit 1 ;;
  esac
}

# model_section_verdict <输出> <段标题> <用例名>：这一格的判法（单元键里的 judge，文件头「复用」）：一段的报告从它的标题行起、到下一个「── … ──」标题行为止
# （不按固定行数取，报告长了照样找得到），要有「模型对拍 N 步」那一行、N 大于 0。判绿打一行「判过」与那一行原样、返回 0；判红打红句与出路、返回 1
model_section_verdict() {
  local log="$1" section="$2" test_name="$3" line steps
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
    return 1
  fi
  steps="$(sed -n 's/.*模型对拍 \([0-9]*\) 步.*/\1/p' <<<"$line")"
  if [[ -z "$steps" || "$steps" == 0 ]] && [[ "${GATE_MODEL_DIFFERENTIAL_BREAK:-}" != zero-steps-passed ]]; then
    echo "  ✗ 这一段模型一步都没判：「$section」（用例 $test_name）"
    echo "     → 怎么办：测试绿而模型没判过任何一步，等于没对拍（show-me-test.md「扫到 0 项也不是通过」）。"
    echo "                看 history.rs 的执行器每一步调入口之前有没有先问模型（judge_by_model），前提不满足的步不算。"
    return 1
  fi
  local judged_sections=0
  judged_sections=$((judged_sections + 1))
  echo "  ✓ 模型对拍这一段判过、实现与模型没有对不上的（查了 $judged_sections 段，用例 $test_name）："
  echo "      ${section}：$(sed 's/^[[:space:]]*//' <<<"$line")"
  return 0
}

# model_verdict_request <请求文件> <用例名> <cargo 命令的样子>：这一格那一条判定的请求（admission.py 文件头「判定请求」）：入口是那条用例的测试函数，
# 判法 model_section_verdict（在这份门禁所在的仓里算闭包），cargo 命令的样子（不带路径与内存上限）进一行 line:。
# 弄坏开关 one-key-for-every-cell 下另加整份测试文件（file:）：六格共用一份输入，改一格的函数六格一起作废
model_verdict_request() {
  local entry="crates/singlefs-harness/tests/$TEST_BINARY.rs::$2"
  local -a tokens=(profile:release "judge:.claude/gate.d/$STAGE_NAME::model_section_verdict" "judge-root:$REPO_ROOT" "line:cargo=$3")
  if [[ "${GATE_MODEL_DIFFERENTIAL_BREAK:-}" == one-key-for-every-cell ]]; then tokens+=("file:crates/singlefs-harness/tests/$TEST_BINARY.rs"); fi
  { printf '%s\t%s' "$entry" "$entry"; printf '\t%s' "${tokens[@]}"; printf '\n'; } > "$1"
}

# model_seed_prior_verdict <种类> <请求文件> <草稿目录>：样本档（.model-differential-sample-tools/ 在、另放 .unit-verdict-sample/prior/ 上一版的树）：
# 拿上一版算这一格的键、照判绿记进这一版样本的判定库（文件头「样本」），这一版再照常查库
model_seed_prior_verdict() {
  local kind="$1" request_file="$2" scratch="$3" prior="$ROOT/.unit-verdict-sample/prior"
  if ! python3 "$ADMISSION" verdict-plan "$prior" "$kind" "$request_file" "$scratch/seed.plan" --library-root "$ROOT" >"$scratch/seed.out" 2>&1; then
    echo "  ✗ 样本档：上一版的树上算不出这一格的键（$(tail -n 2 "$scratch/seed.out" | tr '\n' ' ')）"
    echo "     → 怎么办：样本坏了，看 setup.sh 造的 .unit-verdict-sample/prior/ 全不全"
    return 1
  fi
  cut -f1 "$request_file" > "$scratch/seed.names"
  python3 "$ADMISSION" verdict-record "$prior" "$kind" "$scratch/seed.plan" "$scratch/seed.names" --how "样本档：上一版" --library-root "$ROOT" >"$scratch/seed.record" 2>&1 || {
    echo "  ✗ 样本档：上一版那一格的判定没记进库（$(tail -n 2 "$scratch/seed.record" | tr '\n' ' ')）"
    echo "     → 怎么办：看 $scratch/seed.record 里 admission.py verdict-record 报的错，修好样本或判定库路径再跑"; return 1; }
  echo "  · 样本档：上一版（.unit-verdict-sample/prior）这一格照判绿记进这一版的判定库"
}

model_differential_cell() {
  local cell_name="$1"
  local section="${MODEL_SECTION[$cell_name]}" test_name="${MODEL_TEST[$cell_name]}"
  local memory_cap_runner="$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh"
  local memory_max="${GATE_MODEL_DIFFERENTIAL_MEMORY_MAX:-8G}"
  local cargo_exit sample_tools=0 kind="$STAGE_NAME:$cell_name" decision="judge" reason="" plan_output="" record_output plan_exit record_exit plan_red=0
  local cargo_command=() cargo_shape=()

  if [[ ! "$memory_max" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    echo "  ✗ GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 写成了「$memory_max」，内存包装不认（或认成字节数）：这一格没跑"
    echo "     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G"
    exit 1
  fi
  cargo_command=(cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --include-ignored --exact "$test_name" --nocapture)
  if [[ "${GATE_MODEL_DIFFERENTIAL_BREAK:-}" == ignored-left-out ]]; then
    cargo_command=(cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --exact "$test_name" --nocapture)
  fi
  cargo_shape=("${cargo_command[@]}")
  # 编译与测试线程不吃 cargo 自己的缺省（核数）：经 capped.sh default 起，取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）；
  # 弄坏开关 GATE_MODEL_DIFFERENTIAL_BREAK=threads-not-defaulted 下不加，是改前的样子
  if [[ "${GATE_MODEL_DIFFERENTIAL_BREAK:-}" != threads-not-defaulted ]]; then
    cargo_command=(bash "$REPO_ROOT/scripts/hosts/capped.sh" default "${cargo_command[@]}")
  fi

  scratch="$(mktemp -d)"
  trap 'rm -rf -- "${scratch:?}"' EXIT
  log="$scratch/cargo.log"
  if [[ -f Cargo.toml && -d crates/singlefs-harness ]]; then
    # 样本档（文件头「样本」）：假 cargo、假 rustc 放 PATH 最前面，不经内存包装与 capped.sh
    if [[ -d "$ROOT/.model-differential-sample-tools" ]]; then
      sample_tools=1
      PATH="$ROOT/.model-differential-sample-tools:$PATH"
      export PATH
      cargo_command=("${cargo_shape[@]}")
      echo "  ! 样本档：$ROOT/.model-differential-sample-tools 在，起 cargo 的是那里面的假 cargo（不经内存包装）；这一趟不算真跑过模型对拍"
    fi
    # 按格复用（文件头「复用」）：这一格一条判定，键相同就不起 cargo
    model_verdict_request "$scratch/request" "$test_name" "${cargo_shape[*]}"
    if ((sample_tools)) && [[ -d "$ROOT/.unit-verdict-sample/prior" ]]; then model_seed_prior_verdict "$kind" "$scratch/request" "$scratch" || exit 1; fi
    if python3 "$ADMISSION" verdict-plan "$ROOT" "$kind" "$scratch/request" "$scratch/plan" >"$scratch/plan.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
    if ((plan_exit == 0)) && grep -q '^PLAN ' "$scratch/plan.out"; then
      plan_output="$(grep -v '^PLAN ' "$scratch/plan.out" | head -n 1)"
      decision="$(cut -f2 <<<"$plan_output")"
      reason="$(cut -f4- <<<"$plan_output")"
    else
      decision="judge"
      reason="算不出（$(tail -n 2 "$scratch/plan.out" | tr '\n' ' ' | cut -c1-300)），这一趟现判、不记库"
      plan_output=""
      # 按条复用失灵（文件头「复用」）：判法只在 admission.py 的 plan-failure；它判失灵，这一格照旧现判跑完、结论判红
      python3 "$ADMISSION" plan-failure "$STAGE_NAME 的 $cell_name 格" verdict-plan "$plan_exit" "$scratch/plan.out" || plan_red=1
    fi
    case "$decision" in
      reuse)
        echo "  · 「$section」复用：$reason（用例 $test_name，这一趟没起 cargo）"
        echo "  · 单元键：现判 0、复用 1、拒算 0、本次未跑 0"
        return 0 ;;
      refused) echo "  · 单元键拒算（$reason）：这一格照跑、不复用、不记库" ;;
      *) echo "  · 单元键：要现判（$reason）" ;;
    esac
    echo "  · 跑：${cargo_command[*]}$( ((sample_tools)) || printf '（经内存包装，上限 %s）' "$memory_max")"
    if ((sample_tools)); then
      if "${cargo_command[@]}" >"$log" 2>&1; then cargo_exit=0; else cargo_exit=$?; fi
    elif bash "$memory_cap_runner" "$memory_max" "${cargo_command[@]}" >"$log" 2>&1; then
      cargo_exit=0
    else
      cargo_exit=$?
    fi
    grep '^MODEL_DIFFERENTIAL_SAMPLE ' "$log" | sed 's/^/    /'
    if ((cargo_exit != 0)); then
      tail -40 "$log"
      case "$cargo_exit" in
        245|250|251|252|253|254)
          echo "  ✗ 「$section」那条用例没判出结果：内存包装 scripts/hosts/run-with-memory-cap.sh 退 $cargo_exit（上限 $memory_max），这一次的输出不算判定"
          echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一格要的比上限多，把 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 设大再跑；"
          echo "                251、252、254 先跑 bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑。"
          echo "                253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑。245 是同一段时间内核报了显存不够：nvidia-smi 看谁在用卡，等它跑完再重跑这一格。"
          ;;
        *)
          echo "  ✗ 「$section」那条用例判红（上面是 cargo test 的尾部；这一趟为什么现判：$reason）"
          echo "     → 怎么办：单跑看细节（经内存包装，上限同这一格）："
          echo "                bash scripts/hosts/run-with-memory-cap.sh $memory_max ${cargo_shape[*]}"
          echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
          ;;
      esac
      exit 1
    fi
  elif [[ -f model-differential-cargo-output.log ]]; then
    echo "  · 样本：录好的输出 model-differential-cargo-output.log 代替 ${cargo_command[*]}（不算键、不查库）"
    cp model-differential-cargo-output.log "$log"
  else
    echo "  ! 没有 crates/singlefs-harness，这一格本次无对象可判（增补 3 第 2 件之前没有模型）"
    exit 77
  fi

  model_section_verdict "$log" "$section" "$test_name" || exit 1
  if [[ -n "$plan_output" && "$decision" == judge ]]; then
    cut -f1 "$scratch/request" > "$scratch/green.names"
    record_output="$(grep -m1 '模型对拍 [0-9]* 步' "$log" | sed 's/^[[:space:]]*//' | cut -c1-200)"
    if python3 "$ADMISSION" verdict-record "$ROOT" "$kind" "$scratch/plan" "$scratch/green.names" --how "${cargo_shape[*]}" --summary "$record_output" \
         >"$scratch/record.out" 2>&1; then record_exit=0; else record_exit=$?; fi
    if ((record_exit == 0)); then
      echo "  · 判绿记库：$(tail -n 1 "$scratch/record.out")"
    else
      echo "  ! 判绿记库：admission.py verdict-record 没成（$(tail -n 2 "$scratch/record.out" | tr '\n' ' ')），下一趟这一格照跑"
      python3 "$ADMISSION" plan-failure "$STAGE_NAME 的 $cell_name 格" verdict-record "$record_exit" "$scratch/record.out" || plan_red=1
    fi
    echo "  · 单元键：现判 1、复用 0、拒算 0、本次未跑 0"
  elif [[ "$decision" == refused ]]; then
    echo "  · 单元键：现判 0、复用 0、拒算 1、本次未跑 0（拒算的照跑了、判绿，不记库）"
  fi
  if ((plan_red)); then
    echo "  ✗ 「$section」这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；这一趟照旧现判跑完、那一段判过了"
    echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/$STAGE_NAME --check $cell_name；不能复用就判红，不退回现判当绿"
    exit 1
  fi
}

cell_model_fast_tier() { model_differential_cell model-fast-tier; }
cell_model_reuse_sampling() { model_differential_cell model-reuse-sampling; }
cell_model_rollback_sampling() { model_differential_cell model-rollback-sampling; }
cell_model_allocation_record_wall() { model_differential_cell model-allocation-record-wall; }
cell_model_unit_area_wall() { model_differential_cell model-unit-area-wall; }
cell_model_unit_area_wall_admission_judged() { model_differential_cell model-unit-area-wall-admission-judged; }

stage_cells_run "$ROOT"
