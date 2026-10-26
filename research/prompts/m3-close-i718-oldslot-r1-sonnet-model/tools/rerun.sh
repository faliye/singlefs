#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条经内存包装跑，工作目录随意）：
#   bash scripts/hosts/run-with-memory-cap.sh <上限> bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 scripts/hosts/run-with-memory-cap.sh（判是不是本仓）
#     --frozen  冻结副本（腿开工时拷出的那棵树）：要是一个目录
#     --work    新的草稿目录：不在、或是空目录；跑出来的全写在这里
#     --model   模型目录（原型、变异表、out/）：默认是本脚本的上一层——模型目录整份拷走时它跟着走；只拷这一份脚本时用它指过去
# 退出码：0 跑完；2 用法错（缺参数、仓根不像本仓、冻结副本不在、草稿目录已存在且非空、模型目录不在），stderr 列用法；
#         别的是 rerun_steps 里出错那一步的退出码。--selftest：0 全过，1 有格红。
#
# 准入与运行条件不写、不调 preflight：这份模板拷进模型目录、草稿目录跑，不从自身路径就找不到 .claude/scripts/preflight.sh；
# 环境要求（仓根、冻结副本、草稿目录）由 parse_arguments 自己判，不满足退 2。
#
# 弄坏开关（只给 --selftest 证红用）：
#   RERUN_TEMPLATE_BREAK=root-from-self-path  仓根改回从本脚本路径往上推四层（旧 rerun.sh 的写法），--repo 不认
#       「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」那一格红
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条经 scripts/hosts/run-with-memory-cap.sh 跑" >&2
  cat >&2 <<'USAGE'
用法：rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
      rerun.sh --selftest
USAGE
  exit 2
}

parse_arguments() {
  REPO_ROOT="" FROZEN="" WORK="" MODEL=""
  while (($#)); do
    case "$1" in
      --repo|--frozen|--work|--model)
        (($# >= 2)) || refuse "$1 后面要跟一个路径"
        case "$1" in
          --repo) REPO_ROOT="$2" ;;
          --frozen) FROZEN="$2" ;;
          --work) WORK="$2" ;;
          --model) MODEL="$2" ;;
        esac
        shift 2 ;;
      *) refuse "认不出参数：$1" ;;
    esac
  done
  [[ -n "$REPO_ROOT" ]] || refuse "缺 --repo <仓根>"
  [[ -n "$FROZEN" ]] || refuse "缺 --frozen <冻结副本>"
  [[ -n "$WORK" ]] || refuse "缺 --work <新的草稿目录>"
  if [[ "${RERUN_TEMPLATE_BREAK:-}" == root-from-self-path ]]; then
    REPO_ROOT="$(dirname "${BASH_SOURCE[0]}")/../../../.."
  fi
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 scripts/hosts/run-with-memory-cap.sh）"
  REPO_ROOT="$(cd "$REPO_ROOT" && pwd)"
  [[ -d "$FROZEN" ]] || refuse "--frozen $FROZEN 不是目录"
  FROZEN="$(cd "$FROZEN" && pwd)"
  MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"
  [[ -d "$MODEL" ]] || refuse "模型目录 $MODEL 不在（只拷了这一份脚本的，用 --model 指过去）"
  MODEL="$(cd "$MODEL" && pwd)"
  if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
    refuse "--work $WORK 已存在且非空：给一个新的草稿目录，不在旧输出上接着写"
  fi
  mkdir -p "$WORK"
  WORK="$(cd "$WORK" && pwd)"
}

# ── 这一腿（I-7.18 旧槽第一轮正推腿）要跑的步骤 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起。线程上限照派发提示的 31。
# 本脚本没有 --selftest（模板的自证盯着模板自带的 rerun_steps，换了步骤就不适用）。
THREADS=31
VARIANTS="baseline a a_seed b b_strict"
GROUPS_OF_THE_PROBE="group_next_slot_states group_every_prefix_of_the_publish_on_a_full_ring group_wrong_t_and_p_mirrors group_young_ring_oldest_root_lost_afterwards group_wrong_t_mirrors_on_a_history_with_empty_publishes group_a_device_missing group_second_revolution_wrong_t_and_p_mirrors"

prepare_tree() {   # prepare_tree <目录> [<补丁>…]：冻结副本拷过来、打补丁、放量具
  local directory="$1"; shift
  mkdir -p "$directory"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$directory/"
  local patch_file
  for patch_file in "$@"; do
    (cd "$directory" && patch -p1 < "$MODEL/patches/$patch_file" > /dev/null)
  done
  cp "$MODEL/probe/"*.rs "$directory/crates/singlefs-harness/tests/"
}

logged() {   # logged <日志名> -- <命令…>：全量与只含错误警告的两份日志写进 $WORK/logs
  local name="$1"; shift 2
  bash "$REPO_ROOT/scripts/hosts/two-logs.sh" --quiet "$WORK/logs/$name" -- "$@" > /dev/null 2>&1 || true
}

run_probe_groups() {   # run_probe_groups <树> <日志名前缀> <变体开关值> [<编译目录>]：量具的六个分组
  local tree="$1" prefix="$2" variant="$3" target_directory="${4:-$WORK/target}" group
  for group in $GROUPS_OF_THE_PROBE; do
    (cd "$tree" && I718_VARIANT="$variant" SINGLEFS_RUN_SCOPE="种类=target; 跑什么=probe_i718_oldslot 的 $group 一个测试; 范围=1 个测试; 为什么=量 $prefix 下 I-7.18 对各镜像的判定" CARGO_TARGET_DIR="$target_directory" \
      nice -n 19 bash "$REPO_ROOT/scripts/hosts/two-logs.sh" --quiet "$WORK/logs/$group-$prefix" -- bash "$REPO_ROOT/scripts/hosts/capped.sh" "$THREADS" \
      cargo test --offline --release -p singlefs-harness --test probe_i718_oldslot -- --nocapture --test-threads 1 "$group" > /dev/null 2>&1) || true
  done
}

run_scenarios() {   # run_scenarios <树> <变体开关值>：investigator 的三个场景探针
  local tree="$1" variant="$2" scenario
  for scenario in probe_plain probe_torn probe_zeroed; do
    (cd "$tree" && I718_VARIANT="$variant" SINGLEFS_RUN_SCOPE="种类=target; 跑什么=probe_i718_baseline 的 $scenario 一个测试; 范围=1 个测试; 为什么=量 $variant 下三个场景交出的镜像上 I-3.1、I-7.18、I-7.19 的判定" CARGO_TARGET_DIR="$WORK/target" \
      nice -n 19 bash "$REPO_ROOT/scripts/hosts/two-logs.sh" --quiet "$WORK/logs/scen-$scenario-$variant" -- bash "$REPO_ROOT/scripts/hosts/capped.sh" "$THREADS" \
      cargo test --offline --release -p singlefs-harness --test probe_i718_baseline -- --nocapture --test-threads 1 "$scenario" > /dev/null 2>&1) || true
  done
}

run_regressions() {   # run_regressions <树> <变体开关值>：既有回归（C773 的 9 条、坏镜像 278 条、池级 checker 库测试）
  local tree="$1" variant="$2"
  (cd "$tree" && I718_VARIANT="$variant" SINGLEFS_RUN_SCOPE="种类=target; 跑什么=root_record_carries 目标的全部测试; 范围=1 个测试目标; 为什么=量 $variant 下既有回归测试的通过数" CARGO_TARGET_DIR="$WORK/target" \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/two-logs.sh" --quiet "$WORK/logs/regress-rootrecord-$variant" -- bash "$REPO_ROOT/scripts/hosts/capped.sh" "$THREADS" \
    cargo test --offline --release -p singlefs-harness --test root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot -- --include-ignored > /dev/null 2>&1) || true
  (cd "$tree" && I718_VARIANT="$variant" SINGLEFS_RUN_SCOPE="种类=target; 跑什么=checker_known_bad_images 目标的全部测试; 范围=1 个测试目标; 为什么=量 $variant 下既有回归测试的通过数" CARGO_TARGET_DIR="$WORK/target" \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/two-logs.sh" --quiet "$WORK/logs/regress-knownbad-$variant" -- bash "$REPO_ROOT/scripts/hosts/capped.sh" "$THREADS" \
    cargo test --offline --release -p singlefs-harness --test checker_known_bad_images > /dev/null 2>&1) || true
  (cd "$tree" && I718_VARIANT="$variant" SINGLEFS_RUN_SCOPE="种类=target; 跑什么=cargo test -p singlefs-checker 的测试目标 --lib; 范围=1 个测试目标; 为什么=量 $variant 下既有回归测试的通过数" CARGO_TARGET_DIR="$WORK/target" \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/two-logs.sh" --quiet "$WORK/logs/regress-checkerlib-$variant" -- bash "$REPO_ROOT/scripts/hosts/capped.sh" "$THREADS" \
    cargo test --offline --release -p singlefs-checker --lib > /dev/null 2>&1) || true
}

compare_with_the_switch_build() {   # compare_with_the_switch_build <候选名> <开关值>：候选自己的补丁编一份，量具输出与开关版同名变体逐行比
  local name="$1" variant="$2" patch_file="$3" group
  prepare_tree "$WORK/tree-cross-$name" "$patch_file"
  (cd "$WORK/tree-cross-$name" && SINGLEFS_RUN_SCOPE="种类=target; 跑什么=只编 probe_i718_oldslot 一个测试目标; 范围=1 个测试目标，只编不跑; 为什么=交叉核候选 $name 自己的补丁与开关版同名变体的输出一致" CARGO_TARGET_DIR="$WORK/target-cross-$name" \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/two-logs.sh" --quiet "$WORK/logs/cross-build-$name" -- bash "$REPO_ROOT/scripts/hosts/capped.sh" "$THREADS" \
    cargo test --offline --release -p singlefs-harness --test probe_i718_oldslot --no-run > /dev/null 2>&1) || true
  run_probe_groups "$WORK/tree-cross-$name" "cross-$name" "" "$WORK/target-cross-$name"
  for group in $GROUPS_OF_THE_PROBE; do
    if diff <(grep -o 'ROW group=.*\|ROWDEGRADED.*' "$WORK/logs/$group-$variant.log") <(grep -o 'ROW group=.*\|ROWDEGRADED.*' "$WORK/logs/$group-cross-$name.log") > /dev/null; then
      echo "cross-check $name $group: 与开关版 $variant 逐行相同 ($(grep -c 'ROW group=\|ROWDEGRADED' "$WORK/logs/$group-cross-$name.log") 行)" >> "$WORK/out/cross-check.txt"
    else
      echo "cross-check $name $group: 与开关版 $variant 不同" >> "$WORK/out/cross-check.txt"
    fi
  done
  rm -rf "$WORK/tree-cross-$name" "$WORK/target-cross-$name"
}

validate_the_kernel_patches() {   # 走读内核的三份补丁各自解析并校验（前言常量是假值，只校验判一不变量那个函数的闭包）
  local naga_target="$WORK/naga-target" kernel_source="crates/singlefs-checker-tier/src/crash_judge_checker.wgsl" name patch_file
  local common_directory="$FROZEN/crates/singlefs-checker-tier/src"
  mkdir -p "$WORK/wgsl"
  (cd "$MODEL/tools/naga-check" && SINGLEFS_RUN_SCOPE="种类=target; 跑什么=只编 naga-check 小程序; 范围=1 个测试目标，只编不跑; 为什么=校验候选在走读内核里的 WGSL 写法" CARGO_TARGET_DIR="$naga_target" \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/two-logs.sh" --quiet "$WORK/logs/naga-build" -- bash "$REPO_ROOT/scripts/hosts/capped.sh" "$THREADS" \
    cargo build --offline --release > /dev/null 2>&1) || true
  for name in baseline:- a:candidate-a-gpu.patch a_seed:candidate-a-seed-edge-gpu.patch b:candidate-b-gpu.patch; do
    patch_file="${name#*:}"; name="${name%%:*}"
    mkdir -p "$WORK/wgsl/$name/crates/singlefs-checker-tier/src"
    cp "$FROZEN/$kernel_source" "$WORK/wgsl/$name/$kernel_source"
    if [[ "$patch_file" != "-" ]]; then (cd "$WORK/wgsl/$name" && patch -p1 < "$MODEL/patches/$patch_file" > /dev/null); fi
    cat "$common_directory/crash_judge_common.wgsl" "$common_directory/crash_judge_checker_common.wgsl" "$WORK/wgsl/$name/$kernel_source" > "$WORK/wgsl/$name/body.wgsl"
    python3 "$MODEL/tools/closure.py" "$WORK/wgsl/$name/body.wgsl" judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root "$WORK/wgsl/$name/closure.wgsl" > /dev/null
    echo "naga $name: $(bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 2G "$naga_target/release/naga-check" "$WORK/wgsl/$name/closure.wgsl" "$MODEL/tools/naga-check/prelude-extra.wgsl" 2>&1 | grep -E '^(PARSE|VALID)' | tr '\n' ' ')" >> "$WORK/out/naga.txt"
  done
  # 阴性对照：把甲含种子边那份里的 `eq64(…)` 换成一个 vec2 直接参与 `||`，校验必须红
  sed 's/accepted = accepted || eq64(reclaim_threshold, recomputed_with_the_old_content);/accepted = accepted || reclaim_threshold;/' "$WORK/wgsl/a_seed/body.wgsl" > "$WORK/wgsl/a_seed/body-broken.wgsl"
  python3 "$MODEL/tools/closure.py" "$WORK/wgsl/a_seed/body-broken.wgsl" judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root "$WORK/wgsl/a_seed/closure-broken.wgsl" > /dev/null
  echo "naga 阴性对照（故意写错类型，必须红）: $(bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 2G "$naga_target/release/naga-check" "$WORK/wgsl/a_seed/closure-broken.wgsl" "$MODEL/tools/naga-check/prelude-extra.wgsl" 2>&1 | grep -E '^(PARSE|VALID)' | tr '\n' ' ')" >> "$WORK/out/naga.txt"
}

# 分段跑：环境变量 RERUN_ONLY 逗号分隔选段（build、run-<变体>、cross-A、cross-Aseed、cross-B、naga、summary），不设就全跑。全跑约十几分钟，前台有超时的会话分段跑要自己
# 处理 --work 非空的拒绝（本脚本不改模板那一段）。
wanted() { [[ -z "${RERUN_ONLY:-}" || ",${RERUN_ONLY}," == *",$1,"* ]]; }

rerun_steps() {
  mkdir -p "$WORK/logs" "$WORK/out"
  if wanted build; then : > "$WORK/out/cross-check.txt"; fi
  # 一、开关版：基线与四种判法在同一份编译里按环境变量 I718_VARIANT 切
  if wanted build; then
    prepare_tree "$WORK/tree-switch" measurement-switch.patch
    (cd "$WORK/tree-switch" && SINGLEFS_RUN_SCOPE="种类=target; 跑什么=只编量具的两个测试目标、既有回归的两个测试目标和池级 checker 库测试; 范围=5 个测试目标，只编不跑; 为什么=I-7.18 旧槽第一轮正推腿量三个候选的判别力" CARGO_TARGET_DIR="$WORK/target" \
      nice -n 19 bash "$REPO_ROOT/scripts/hosts/two-logs.sh" --quiet "$WORK/logs/build-switch" -- bash "$REPO_ROOT/scripts/hosts/capped.sh" "$THREADS" \
      cargo test --offline --release -p singlefs-harness --test probe_i718_oldslot --test probe_i718_baseline --test root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot --test checker_known_bad_images -p singlefs-checker --lib --no-run > /dev/null 2>&1) || true
  fi
  local variant
  for variant in $VARIANTS; do
    if wanted "run-$variant"; then
      run_probe_groups "$WORK/tree-switch" "$variant" "$variant"
      run_scenarios "$WORK/tree-switch" "$variant"
      run_regressions "$WORK/tree-switch" "$variant"
    fi
  done
  # 二、候选自己的补丁各编一份，与开关版同名变体逐行比
  if wanted cross-A; then compare_with_the_switch_build A a candidate-a-cpu.patch; fi
  if wanted cross-Aseed; then compare_with_the_switch_build Aseed a_seed candidate-a-seed-edge-cpu.patch; fi
  if wanted cross-B; then compare_with_the_switch_build B b candidate-b-cpu.patch; fi
  # 三、走读内核
  if wanted naga; then
    : > "$WORK/out/naga.txt"
    validate_the_kernel_patches
  fi
  # 四、汇总
  if wanted summary; then
    python3 "$MODEL/tools/summarize.py" "$WORK/logs" $VARIANTS > "$WORK/out/summary.txt"
    (cd "$MODEL/patches" && sha256sum *.patch) > "$WORK/out/patch-sha256.txt"
    rm -rf "$WORK/tree-switch"
    echo "name=oldslot_r1 repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
    cat "$WORK/out/summary.txt" "$WORK/out/cross-check.txt" "$WORK/out/naga.txt"
    if [[ -f "$MODEL/out/summary.txt" ]]; then
      if diff -u "$MODEL/out/summary.txt" "$WORK/out/summary.txt" > "$WORK/out/summary.diff"; then
        echo "与 $MODEL/out/summary.txt 逐行相同"
      else
        echo "与 $MODEL/out/summary.txt 有差异，见 $WORK/out/summary.diff"
      fi
    fi
  fi
}

parse_arguments "$@"
rerun_steps
