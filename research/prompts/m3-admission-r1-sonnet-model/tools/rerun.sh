#!/usr/bin/env bash
# m3-admission-r1 云端正推腿的复跑：把冻结副本拷进草稿目录，装上腿的探针，逐条起 cargo test，输出写 $WORK/out/，最后与 $MODEL/out/ 逐行比。
# 用法（整条经内存包装跑）：
#   bash scripts/hosts/run-with-memory-cap.sh 16G bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
# 退出码：0 跑完；2 用法错；别的是 rerun_steps 里出错那一步的退出码。
set -euo pipefail

refuse() {
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条经 scripts/hosts/run-with-memory-cap.sh 跑" >&2
  cat >&2 <<'USAGE'
用法：rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
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

# ── 这一腿要跑的步骤 ──
# 每条 cargo 经 capped.sh 起（线程上限 15），同一时刻一条；输出写 $WORK/out/<名>.log，只留 name= 行与 test result 行进 $WORK/out/<名>.txt。
rerun_steps() {
  local tree="$WORK/tree" tests target="${RERUN_TARGET_DIR:-$WORK/target}" groups="${RERUN_GROUPS:-probes,mutrows,demand}" cargo_cap="bash $REPO_ROOT/scripts/hosts/capped.sh 15"
  mkdir -p "$tree" "$WORK/out"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$tree/"
  cp -r "$REPO_ROOT/.cargo" "$tree/.cargo"
  find "$tree/crates" -type f -exec touch {} +
  tests="$tree/crates/singlefs-harness/tests"
  cp "$MODEL/probes/a3_probe.rs" "$MODEL/probes/c582_probe.rs" "$tests/"
  cat "$MODEL/probes/append-to-overwrite_in_one_instance.rs" >> "$tests/overwrite_in_one_instance.rs"
  cat "$MODEL/probes/append-to-commit_generated_units_fall_back_when_the_segment_runs_out.rs" >> "$tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs"
  export CARGO_TARGET_DIR="$target" MODEL_TREE="$tree" MODEL_TARGET="$target"
  local step
  run_cargo() {   # run_cargo <名> <cargo test 的参数…>
    local name="$1"; shift
    $cargo_cap nice -n 19 cargo test --offline --manifest-path "$tree/Cargo.toml" "$@" > "$WORK/out/$name.log" 2>&1 || echo "  (步 $name 退出码 $?，日志在 out/$name.log)"
    grep -E 'name=|reclaimed|test result|^test ' "$WORK/out/$name.log" | cut -c1-600 > "$WORK/out/$name.txt" || true
  }
  if [[ ",$groups," == *,probes,* ]]; then
  run_cargo a3-a2-a4-a5-probe -p singlefs-harness --test a3_probe -- --nocapture
  run_cargo c582-probe -p singlefs-harness --test c582_probe -- --nocapture
  run_cargo c546-probe -p singlefs-harness --test commit_generated_units_fall_back_when_the_segment_runs_out -- --nocapture probe_c546
  run_cargo c576-switch-off-probe -p singlefs-harness --test overwrite_in_one_instance -- --nocapture probe_c576
  run_cargo c568-existing-test -p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode -- a_system_configuration_slot_one root_ring_regions_longer
  run_cargo c545-existing-5-tests -p singlefs-harness --test admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments
  run_cargo a4c-ckpt-cost-existing -p singlefs-harness --test admission_checkpoint_cost_per_device_paths -- --nocapture
  python3 -I "$MODEL/probes/geom.py" > "$WORK/out/geom.txt"
  fi
  # 变异行：基线（--baseline）与变异各一次，行 12、76、77、78、80、145；MUTROW_ARGS 是点名的测试参数
  mutation_row() {   # mutation_row <行号> <MUTROW_ARGS>
    local row="$1" args="$2"
    MUTROW_ARGS="$args" python3 -I "$MODEL/probes/mutrow.py" "$row" --baseline > "$WORK/out/mutrow-$row-baseline.txt" 2>&1 || true
    MUTROW_ARGS="$args" python3 -I "$MODEL/probes/mutrow.py" "$row" > "$WORK/out/mutrow-$row-mutated.txt" 2>&1 || true
  }
  if [[ ",$groups," == *,mutrows,* ]]; then
  mutation_row 12 "-p singlefs-harness --test overwrite_in_one_instance -- publish_running_out_of_space_midway"
  mutation_row 76 "-p singlefs-harness --test unequal_devices -- devices_answering_different_places"
  mutation_row 77 "-p singlefs-harness --test unequal_devices -- user_data_slots_that_differ"
  mutation_row 78 "-p singlefs-harness --test unequal_devices -- devices_answering_different_places"
  mutation_row 80 "-p singlefs-harness --test commit_generated_units_fall_back_when_the_segment_runs_out -- raising_the_floor_when_the_only"
  mutation_row 145 "-p singlefs-harness --test admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments -- with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal"
  fi
  # 需求乘盘数的临时改动：admission 单测里应有一条红
  if [[ ",$groups," == *,demand,* ]]; then
  python3 -I "$MODEL/probes/mutgen.py" crates/singlefs-core/src/admission.rs "$MODEL/probes/demand-times-devices.old.txt" "$MODEL/probes/demand-times-devices.new.txt" -p singlefs-core --lib admission > "$WORK/out/demand-times-devices.txt" 2>&1 || true
  fi
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  for step in "$WORK"/out/*.txt; do
    local base; base="$(basename "$step")"
    if [[ -f "$MODEL/out/$base" ]]; then
      # 并行跑的测试线程打印次序不定：两边都按行排序再比，去掉耗时行
      if diff <(grep -v -E 'finished in|real|user|sys' "$step" | sed -E 's/\(([0-9]+)\)//g' | sort) <(grep -v -E 'finished in|real|user|sys' "$MODEL/out/$base" | sed -E 's/\(([0-9]+)\)//g' | sort) > /dev/null; then
        echo "name=rerun_compare file=$base same=yes"
      else
        echo "name=rerun_compare file=$base same=no"
      fi
    else
      echo "name=rerun_compare file=$base same=no_reference_in_model_out"
    fi
  done
}

parse_arguments "$@"
rerun_steps
