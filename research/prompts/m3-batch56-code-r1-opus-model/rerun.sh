#!/usr/bin/env bash
# 攻方腿（m3-batch56-code-r1 opus）的复跑：rerun.sh <工作目录> <阶段>…（或 all）。
# 环境：SINGLEFS_REPO（被判的冻结副本，缺省 ~/.local/share/singlefs/work/m3-batch56-code-r1/repo）、SINGLEFS_SCRIPTS（仓的 research/scripts，
# 缺省按模型目录在仓里的位置推）、OPUS_THREADS（线程上限，缺省 10）、OPUS_MEMORY_CAP（内存包装上限，缺省 10G）。
# 每个副本只拷 Cargo.toml、Cargo.lock、crates/，各用自己的 CARGO_TARGET_DIR；同一时刻只起一条 cargo。产物写 <工作目录>/out/<阶段>.txt。
set -euo pipefail
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${SINGLEFS_REPO:-$HOME/.local/share/singlefs/work/m3-batch56-code-r1/repo}"
SCRIPTS="${SINGLEFS_SCRIPTS:-$MODEL/../../scripts}"
THREADS="${OPUS_THREADS:-10}"
CAP="${OPUS_MEMORY_CAP:-10G}"
WORK="$(mkdir -p "$1" && cd "$1" && pwd)"
shift
mkdir -p "$WORK/out" "$WORK/tmp"
export TMPDIR="$WORK/tmp"

copy_of() { # copy_of <名> [变异名…]：建副本、装原型 crate、打变异
  local name="$1"; shift
  local dir="$WORK/$name"
  rm -rf "$dir"; mkdir -p "$dir"
  cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$dir/"
  rsync -a --exclude target "$REPO/crates/" "$dir/crates/"
  cp -r "$MODEL/probe-crate" "$dir/crates/opus-probe-b56"
  python3 - "$dir/Cargo.toml" <<'PY'
import sys
from pathlib import Path
p = Path(sys.argv[1]); s = p.read_text()
old = '    "crates/singlefs-reference-filesystem",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, old + '    "crates/opus-probe-b56",\n'))
PY
  # 变异副本共用一个编译目录：上一份变异改过的文件拷回原样之后 mtime 比上次编译旧，cargo 认不出要重编，所以变异会碰的几份一律 touch。
  if [[ "$name" == mut ]]; then
    (cd "$dir" && touch crates/singlefs-core/src/container_tree.rs crates/singlefs-core/src/inode_tree.rs \
      crates/singlefs-core/src/recovery.rs crates/singlefs-core/src/mount.rs crates/singlefs-checker/src/walk.rs)
  fi
  for mutant in "$@"; do python3 "$MODEL/mutants.py" "$dir" "$mutant"; done
}

cargo_in() { # cargo_in <副本名> <cargo 参数…>：经线程上限与内存包装跑
  local name="$1"; shift
  (cd "$WORK/$name" && CARGO_TARGET_DIR="$WORK/target-$name" nice -n 19 bash "$SCRIPTS/capped.sh" "$THREADS" \
    bash "$SCRIPTS/run-with-memory-cap.sh" "$CAP" cargo "$@")
}

normalize() { sed -E 's/elapsed=[0-9.]+s//g; s/finished in [0-9.]+s//g'; }

run_tests() { # run_tests <副本名> <输出> <cargo test 参数…>：只留结果行与原型的 OPUS 行
  local name="$1" out="$2"; shift 2
  set +e
  cargo_in "$name" test --offline --release "$@" -- --nocapture > "$WORK/tmp/last.log" 2>&1
  local rc=$?
  set -e
  { echo "command args=$* rc=$rc"; grep -E '^OPUS|^test result|^failures:$|^    [a-z_0-9:]+$' "$WORK/tmp/last.log" | normalize | grep -v '^test result: ok\. 0 passed' | python3 "$MODEL/sort_k2r.py" ; } >> "$out"
}

stage() {
  local out="$WORK/out/$1.txt"
  rm -f "$out"
  case "$1" in
    base-build) copy_of base; cargo_in base test --offline --release -p opus-probe-b56 --no-run > /dev/null 2>&1; echo built > "$out" ;;
    k2-random-b1|k2-random-b2|k2-random-b3)
      local first last ops
      case "$1" in
        k2-random-b1) first=1; last=10; ops=100 ;;
        k2-random-b2) first=11; last=20; ops=150 ;;
        k2-random-b3) first=21; last=30; ops=150 ;;
      esac
      OPUS_SEED_FIRST=$first OPUS_SEED_LAST=$last OPUS_OPS=$ops OPUS_THREADS="$THREADS" run_tests base "$out.raw" -p opus-probe-b56 --test k2_random_histories
      grep '^OPUS_K2R ' "$out.raw" | sort -t= -k2 -n > "$out"; grep -v '^OPUS_K2R ' "$out.raw" >> "$out"; rm -f "$out.raw" ;;
    k2-knob) run_tests base "$out" -p opus-probe-b56 --test k2_knob_bad_images ;;
    k2-crash-counts)
      for window in rollback create unlink-largest; do
        OPUS_COUNT_ONLY=1 OPUS_WINDOW=$window run_tests base "$out" -p opus-probe-b56 --test k2_crash_abandoned
      done ;;
    k2-crash-timing) OPUS_TIMING_EXPAND_UP_TO=2 OPUS_WINDOW=rollback OPUS_THREADS="$THREADS" run_tests base "$out" -p opus-probe-b56 --test k2_crash_abandoned ;;
    k1-base) run_tests base "$out" -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking ;;
    k4-shape) OPUS_SEED_FIRST=1 OPUS_SEED_LAST=8 OPUS_UNLINKS=1200 run_tests base "$out" -p opus-probe-b56 --test k4_leaf_shape_stress ;;
    k4-family)
      copy_of k4family
      python3 "$MODEL/inject.py" "$WORK/k4family/crates/singlefs-core/src/inode_tree.rs" "$MODEL/probes/inode_tree_k4_family.rs.snippet" > /dev/null
      run_tests k4family "$out" -p singlefs-core --lib inode_tree::tests ;;
    mut-*)
      local mutant="${1#mut-}"
      # 变异副本挨个用同一个目录与同一个编译目录（同一时刻只有一份），只重编被改的那几个包；
      # 同一个变异的阶段被打断后重起时不重拷（重拷要 touch，编译又从头来），接着上次的编译。
      if [[ "$(cat "$WORK/mut.applied" 2>/dev/null)" != "$mutant" || ! -d "$WORK/mut" ]]; then
        copy_of mut "$mutant"
        echo "$mutant" > "$WORK/mut.applied"
      fi
      case "$mutant" in
        w-close-one-early)
          run_tests mut "$out" -p singlefs-core --lib _tree::
          run_tests mut "$out" -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full
          run_tests mut "$out" -p singlefs-harness --test checker_known_bad_images
          OPUS_SEED_FIRST=1 OPUS_SEED_LAST=10 OPUS_OPS=60 OPUS_THREADS="$THREADS" run_tests mut "$out" -p opus-probe-b56 --test k2_random_histories ;;
        c-*)
          run_tests mut "$out" -p singlefs-harness --test checker_known_bad_images
          run_tests mut "$out" -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full ;;
        fix-*)
          run_tests mut "$out" -p singlefs-harness --test checker_known_bad_images
          run_tests mut "$out" -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full
          run_tests mut "$out" -p opus-probe-b56 --test k2_knob_bad_images
          OPUS_SEED_FIRST=1 OPUS_SEED_LAST=10 OPUS_OPS=100 OPUS_THREADS="$THREADS" run_tests mut "$out" -p opus-probe-b56 --test k2_random_histories ;;
        g1-*)
          run_tests mut "$out" -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking ;;
        k4-*)
          run_tests mut "$out" -p singlefs-core --lib inode_tree::
          run_tests mut "$out" -p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference ;;
        k5-*)
          run_tests mut "$out" -p singlefs-core --lib tree_identifier_watermark_of_the_rollback_tests ;;
      esac ;;
    compare) python3 "$MODEL/compare.py" "$MODEL/expected" "$WORK/out" | tee "$out" ;;
    *) echo "不认识的阶段 $1" >&2; exit 2 ;;
  esac
}

if [[ "${1:-}" == all ]]; then
  set -- base-build k2-knob k2-crash-counts k2-crash-timing k1-base k2-random-b1 k2-random-b2 k2-random-b3 k4-shape k4-family \
    $(python3 "$MODEL/mutants.py" --list | sed 's/^/mut-/') compare
fi
for name in "$@"; do stage "$name"; done
