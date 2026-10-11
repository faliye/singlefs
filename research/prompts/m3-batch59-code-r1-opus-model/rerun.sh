#!/usr/bin/env bash
# 攻方腿（m3-batch59-code-r1 opus）的复跑：rerun.sh <工作目录> <阶段>…（或 all；阶段表 `rerun.sh <工作目录> list`）。
# 环境：SINGLEFS_REPO（被判的冻结副本，缺省 ~/.local/share/singlefs/work/m3-batch59-code-r1/repo）、SINGLEFS_SCRIPTS（仓的 scripts，
# 缺省按模型目录在仓里的位置推）、OPUS_THREADS（线程上限，缺省 10）、OPUS_MEMORY_CAP（内存包装上限，缺省 6G）、
# OPUS_SEED_TARGET（可选：拿来当 release 编译种子的旧编译目录；拷之前先 touch 副本的全部源码，cargo 只复用三方依赖）。
# 副本只拷 Cargo.toml、Cargo.lock、crates/；原型 crate 拷成副本工作区成员 crates/opus-probe-b59（测试目标名不带 layer0，不开 gpu 特性）；
# probes/core_review_l4_append.rs 接到副本 core_review_tree_table_duplicates_and_slot_one_search.rs 的末尾。
# base 与 mut（打变异的那一份）两个副本各用自己的 CARGO_TARGET_DIR，debug 那一段另用 target-debug；同一时刻只起一条 cargo，
# 每条先只编不跑（经 capped.sh），再经 capped.sh 与 run-with-memory-cap.sh 跑。产物写 <工作目录>/out/<阶段>.txt，
# 挂钟另写 <工作目录>/out/<阶段>.wall（不进 compare）。
set -euo pipefail
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${SINGLEFS_REPO:-$HOME/.local/share/singlefs/work/m3-batch59-code-r1/repo}"
SCRIPTS="${SINGLEFS_SCRIPTS:-$MODEL/../../scripts}"
THREADS="${OPUS_THREADS:-10}"
CAP="${OPUS_MEMORY_CAP:-6G}"
WORK="$(mkdir -p "$1" && cd "$1" && pwd)"
shift
# 测试镜像（harness 的 build_pool 照 std::env::temp_dir() 落文件）放 OPUS_TEST_TMPDIR，缺省 <工作目录>/tmp；
# 盘的 IO 压力高时设成 tmpfs 下的目录（本腿交回前那一遍用 /dev/shm/m3-batch59-code-r1-opus，跑完删）
TEST_TMPDIR="${OPUS_TEST_TMPDIR:-$WORK/tmp}"
mkdir -p "$WORK/out" "$WORK/tmp" "$TEST_TMPDIR"
export TMPDIR="$TEST_TMPDIR"

copy_of() { # copy_of <名> [变异名]：建副本、装原型 crate、打变异
  local name="$1" mutant="${2:-}"
  local dir="$WORK/$name"
  rm -rf "${dir:?}"; mkdir -p "$dir"
  cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$dir/"
  rsync -a --exclude target "$REPO/crates/" "$dir/crates/"
  cp -r "$MODEL/probe-crate" "$dir/crates/opus-probe-b59"
  cat "$MODEL/probes/core_review_l4_append.rs" >> "$dir/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs"
  python3 - "$dir/Cargo.toml" <<'PY'
import sys
from pathlib import Path
p = Path(sys.argv[1]); s = p.read_text()
old = '    "crates/singlefs-reference-filesystem",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, old + '    "crates/opus-probe-b59",\n'))
PY
  if [[ "$name" == mut* ]]; then # 变异会改的两份 touch，逼 cargo 重编（rsync -a 带回冻结副本的旧 mtime，不 touch 会判 Fresh）
    (cd "$dir" && touch crates/singlefs-checker/src/walk.rs crates/singlefs-core/src/recovery.rs)
  else # base 与 debug 副本可能拿旧编译目录当种子：全部源码 touch
    (cd "$dir" && find crates -type f -exec touch {} +)
  fi
  if [[ -n "$mutant" ]]; then python3 "$MODEL/mutants.py" "$dir" "$mutant"; fi
}

seed_target() { # seed_target <编译目录名> <种子目录>：编译目录还不在、种子在时拷种子（三方依赖复用；工作区成员换了路径照样重编）
  local target="$WORK/$1" seed="${2:-}"
  if [[ ! -d "$target" && -n "$seed" && -d "$seed" ]]; then cp -a "$seed" "$target"; fi
}

cargo_in() { # cargo_in <副本名> <编译目录名> <cargo 参数…>：经线程上限跑；不带 --no-run 的再经内存包装
  local name="$1" target="$2"; shift 2
  if [[ " $* " == *" --no-run "* ]]; then
    (cd "$WORK/$name" && CARGO_TARGET_DIR="$WORK/$target" nice -n 19 bash "$SCRIPTS/capped.sh" "$THREADS" cargo "$@")
  else
    (cd "$WORK/$name" && CARGO_TARGET_DIR="$WORK/$target" OPUS_THREADS="$THREADS" nice -n 19 bash "$SCRIPTS/capped.sh" "$THREADS" \
      bash "$SCRIPTS/run-with-memory-cap.sh" "$CAP" cargo "$@")
  fi
}

extract() { # 只留结果行、失败名、原型的 OPUS 行，与失败用例里的断言消息（去掉线程号与源码位置）
  python3 "$MODEL/extract.py"
}

run_tests() { # run_tests <副本名> <编译目录名> <输出> <cargo test 参数…> [-- 测试参数…]（--release 由调用方给）
  local name="$1" target="$2" out="$3"; shift 3
  local args=() after=() seen=0
  for a in "$@"; do
    if [[ $seen == 1 ]]; then after+=("$a"); elif [[ "$a" == -- ]]; then seen=1; else args+=("$a"); fi
  done
  set +e
  cargo_in "$name" "$target" test --offline "${args[@]}" --no-run > "$WORK/tmp/last-build.log" 2>&1
  local build_rc=$?
  local started; started=$(date +%s)
  # --no-fail-fast：一个测试二进制红了，同一条命令里排在后面的二进制照跑
  cargo_in "$name" "$target" test --offline --no-fail-fast "${args[@]}" -- "${after[@]}" > "$WORK/tmp/last.log" 2>&1
  local rc=$?
  set -e
  echo "args=$* build_rc=$build_rc wall_s=$(( $(date +%s) - started ))" >> "${out%.txt}.wall"
  { echo "command args=$* rc=$rc"; extract < "$WORK/tmp/last.log"; } >> "$out"
}

CHECKER_LIB=(-p singlefs-checker --lib)
KNOB_HARNESS=(-p singlefs-harness --test checker_known_bad_images --test container_trees_grow_a_level_when_the_root_is_full)
M3_TARGETS=(-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking --test core_review_tree_table_duplicates_and_slot_one_search)
# 合法历史的几份（读者逐条判 I-3.13 那一形下看它们照不照绿）；root_ring_turn_in_one_mount 本机 release 下 199 秒，跑不进前台时限，不放
# 分三段，每段在本机负载下跑得进前台时限
LEGIT_a=(-p singlefs-harness --test accounting_and_mapping_tree_split --test many_inodes_write_and_read
  --test multi_record_publishes_of_transaction_zero --test random_histories)
LEGIT_b=(-p singlefs-harness --test rollback_lineage_record_and_its_checkpoint_cost_term --test writable_mount_takes_a_second_instance
  --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed)
LEGIT_unlink=(-p singlefs-harness --test unlink_single_file_publishes)
PROBE=(-p opus-probe-b59 --test m1_gpu_flow_proxy)

M1_GROUP_j3=tombstone-2-1-1,lineage-2-1-1,retirement-2-1-1
M1_GROUP_l1=tombstone-2-1-2-1-ring,lineage-2-1-2-1-ring,retirement-2-1-2-1-ring
M1_GROUP_l2=tombstone-2-1-2-2-1-ring,lineage-2-1-2-2-1-ring,retirement-2-1-2-2-1-ring

m1_stage() { # m1_stage <副本名> <编译目录名> <输出> <组>：GPU 旋钮流的 CPU 替身，一组三条（单线程跑测试、替身自己按 OPUS_THREADS 开线程）
  local copy="$1" target="$2" out="$3" group="M1_GROUP_$4"
  OPUS_M1_FLOWS="${!group}" run_tests "$copy" "$target" "$out" --release "${PROBE[@]}" -- --nocapture --test-threads 1
}

knob_harness_stage() { # knob_harness_stage <副本名> <编译目录名> <输出> <a|b>：旋钮坏镜像（release）只跑调到带旋钮入口
  # （check_pool_image_given_the_container_capacity_knobs）的那 36 条——名字带 knob 的 34 条与按产品镜像判的两条，旋钮那一支只在带旋钮的入口上走到；
  # a 段：checker 单测、多片换旋钮那 12 条（第 58 批 J3 九条、第 59 批 L2 三条）、容器树长层；b 段：其余 24 条。整份 265 条在本机负载下一次跑不进前台时限
  local copy="$1" target="$2" out="$3" part="$4"
  if [[ "$part" == a ]]; then
    run_tests "$copy" "$target" "$out" --release "${CHECKER_LIB[@]}"
    run_tests "$copy" "$target" "$out" --release -p singlefs-harness --test checker_known_bad_images -- records_under_changing_knobs
    run_tests "$copy" "$target" "$out" --release -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full
  else
    run_tests "$copy" "$target" "$out" --release -p singlefs-harness --test checker_known_bad_images -- \
      knob judged_as_a_product_image_reddens_only_the_full_container_invariant --skip records_under_changing_knobs
  fi
}

recovery_stages() { # recovery.rs 的改坏与基线：J1 与水位那两份用例文件（含接进去的两条原型），再单跑打印 L4 几种造法结局的那一条
  run_tests "$1" "$2" "$3" --release "${M3_TARGETS[@]}"
  run_tests "$1" "$2" "$3" --release -p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search -- \
    --nocapture --test-threads 1 opus_probe_b59_l4_constructions
}

WALK_MUTANTS=(w-second-to-last w-first-only w-last-two w-last-three w-last-four)
RECOVERY_MUTANTS=(row3868 row3861 row3668 r-tag-judged r-count-skips-unregistered-tag r-each-entry-judges-i313
  r-each-entry-judges-i313+r-tag-judged+r-count-skips-unregistered-tag)
STAGES=(base-build base-m1-j3 base-m1-l1 base-m1-l2 base-knob-harness-a base-knob-harness-b base-m3 debug-fast-tier)
for mutant in "${WALK_MUTANTS[@]}"; do STAGES+=("mut-$mutant-m1-j3" "mut-$mutant-m1-l1" "mut-$mutant-m1-l2" "mut-$mutant-harness-a" "mut-$mutant-harness-b"); done
for mutant in "${RECOVERY_MUTANTS[@]}"; do STAGES+=("mut-$mutant"); done
for part in a b unlink; do STAGES+=("base-legit-$part" "mut-r-each-entry-judges-i313-legit-$part"); done

mutant_copy() { # 同一个变异的阶段被打断后重起时不重拷，接着上次的编译
  local mutant="$1"
  if [[ "$(cat "$WORK/mut.applied" 2>/dev/null)" != "$mutant" || ! -d "$WORK/mut" ]]; then
    copy_of mut "$mutant" > /dev/null
    echo "$mutant" > "$WORK/mut.applied"
  fi
  seed_target target-mut "${OPUS_SEED_TARGET:-$WORK/target-base}" # 种子给了就拿种子（小），没给拿 base 的编译目录
}

stage() {
  local out="$WORK/out/$1.txt"
  rm -f "$out" "${out%.txt}.wall"
  case "$1" in
    base-build)
      [[ -d "$WORK/base" ]] || copy_of base # 被前台时限打断后重起不重拷（重拷会再 touch 全部源码、整份重编）
      seed_target target-base "${OPUS_SEED_TARGET:-}"
      local group rc=0
      for group in PROBE CHECKER_LIB KNOB_HARNESS M3_TARGETS LEGIT_a LEGIT_b LEGIT_unlink; do
        local -n targets="$group"
        cargo_in base target-base test --offline --release "${targets[@]}" --no-run >> "$WORK/tmp/base-build.log" 2>&1 || rc=$?
        echo "base-build $group rc=$rc" >> "$out"
        unset -n targets
      done ;;
    base-m1-*) m1_stage base target-base "$out" "${1#base-m1-}" ;;
    base-knob-harness-*) knob_harness_stage base target-base "$out" "${1##*-}" ;;
    base-m3) recovery_stages base target-base "$out" ;;
    base-legit-*) local -n legit="LEGIT_${1#base-legit-}"; run_tests base target-base "$out" --release "${legit[@]}" ;;
    mut-*-legit-*) local rest="${1#mut-}"; local -n legit="LEGIT_${rest##*-legit-}"; mutant_copy "${rest%-legit-*}"
      run_tests mut target-mut "$out" --release "${legit[@]}" ;;
    debug-fast-tier) # 快档（debug、不带 --ignored）整份跑旋钮坏镜像那一个目标：去掉 #[ignore] 的三条与新三条同别的用例一起跑；再只跑旋钮多片那 12 条
      [[ -d "$WORK/debug" ]] || copy_of debug
      run_tests debug target-debug "$out" -p singlefs-harness --test checker_known_bad_images
      run_tests debug target-debug "$out" -p singlefs-harness --test checker_known_bad_images -- records_under_changing_knobs ;;
    mut-w-*-m1-*) local rest="${1#mut-}"; mutant_copy "${rest%-m1-*}"; m1_stage mut target-mut "$out" "${rest##*-m1-}" ;;
    mut-w-*-harness-*) local rest="${1#mut-}"; mutant_copy "${rest%-harness-*}"; knob_harness_stage mut target-mut "$out" "${1##*-}" ;;
    mut-row*|mut-r-*) mutant_copy "${1#mut-}"; recovery_stages mut target-mut "$out" ;;
    *) echo "不认识的阶段 $1" >&2; return 2 ;;
  esac
}

if [[ "${1:-}" == list ]]; then printf '%s\n' "${STAGES[@]}"; exit 0; fi
if [[ "${1:-}" == all ]]; then set -- "${STAGES[@]}" compare; fi
for name in "$@"; do
  if [[ "$name" == compare ]]; then python3 "$MODEL/compare.py" "$MODEL/expected" "$WORK/out"; else stage "$name"; fi
done
