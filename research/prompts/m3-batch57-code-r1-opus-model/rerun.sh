#!/usr/bin/env bash
# 攻方腿（m3-batch57-code-r1 opus）的复跑：rerun.sh <工作目录> <阶段>…（或 all；阶段表见 all 那一行，列表也可以 `rerun.sh <工作目录> list`）。
# 环境：SINGLEFS_REPO（被判的冻结副本，缺省 ~/.local/share/singlefs/work/m3-batch57-code-r1/repo）、SINGLEFS_SCRIPTS（仓的 scripts，
# 缺省按模型目录在仓里的位置推）、OPUS_THREADS（线程上限，缺省 10）、OPUS_MEMORY_CAP（内存包装上限，缺省 4G）。
# 副本只拷 Cargo.toml、Cargo.lock、crates/，原型 crate 拷成副本工作区成员 crates/opus-probe-b57（测试目标名不带 layer0），
# K3 原型片段接到副本里 corrupt_on_disk_content_is_refused_instead_of_panicking.rs 的末尾。base、mutw（改 walk.rs 与变异表那一行写侧的变异）、
# mutr（改 recovery.rs 的变异）三个副本各用自己的 CARGO_TARGET_DIR；同一时刻只起一条 cargo。产物写 <工作目录>/out/<阶段>.txt。
set -euo pipefail
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${SINGLEFS_REPO:-$HOME/.local/share/singlefs/work/m3-batch57-code-r1/repo}"
SCRIPTS="${SINGLEFS_SCRIPTS:-$MODEL/../../scripts}"
THREADS="${OPUS_THREADS:-10}"
CAP="${OPUS_MEMORY_CAP:-4G}"
WORK="$(mkdir -p "$1" && cd "$1" && pwd)"
shift
mkdir -p "$WORK/out" "$WORK/tmp"
export TMPDIR="$WORK/tmp"
CORRUPT=crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs

copy_of() { # copy_of <名> [变异名]：建副本、装原型 crate 与 K3 片段、打变异；变异副本把可能被改过的三份 touch，逼 cargo 重编
  local name="$1" mutant="${2:-}"
  local dir="$WORK/$name"
  rm -rf "$dir"; mkdir -p "$dir"
  cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$dir/"
  rsync -a --exclude target "$REPO/crates/" "$dir/crates/"
  cp -r "$MODEL/probe-crate" "$dir/crates/opus-probe-b57"
  cat "$MODEL/probes/corrupt_on_disk_k3_append.rs" >> "$dir/$CORRUPT"
  python3 - "$dir/Cargo.toml" <<'PY'
import sys
from pathlib import Path
p = Path(sys.argv[1]); s = p.read_text()
old = '    "crates/singlefs-reference-filesystem",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, old + '    "crates/opus-probe-b57",\n'))
PY
  if [[ "$name" == mut* ]]; then
    (cd "$dir" && touch crates/singlefs-checker/src/walk.rs crates/singlefs-core/src/recovery.rs crates/singlefs-core/src/container_tree.rs)
  fi
  if [[ -n "$mutant" ]]; then python3 "$MODEL/mutants.py" "$dir" "$mutant"; fi
}

cargo_in() { # cargo_in <副本名> <cargo 参数…>：经线程上限与内存包装跑
  local name="$1"; shift
  (cd "$WORK/$name" && CARGO_TARGET_DIR="$WORK/target-$name" nice -n 19 bash "$SCRIPTS/capped.sh" "$THREADS" \
    bash "$SCRIPTS/run-with-memory-cap.sh" "$CAP" cargo "$@")
}

normalize() { sed -E 's/elapsed=[0-9.]+s//g; s/finished in [0-9.]+s//g'; }

run_tests() { # run_tests <副本名> <输出> <cargo test 参数…>（参数里可带 -- 之后的过滤）：只留结果行、失败名与原型的 OPUS 行
  local name="$1" out="$2"; shift 2
  local args=() after=() seen=0
  for a in "$@"; do
    if [[ $seen == 1 ]]; then after+=("$a"); elif [[ "$a" == -- ]]; then seen=1; elif [[ "$a" != --release ]]; then args+=("$a"); fi
  done
  set +e
  cargo_in "$name" test --offline --release "${args[@]}" -- --nocapture "${after[@]}" > "$WORK/tmp/last.log" 2>&1
  local rc=$?
  set -e
  { echo "command args=$* rc=$rc"; grep -E '^OPUS|^test result|^failures:$|^    [a-z_0-9:]+$|^thread .* panicked' "$WORK/tmp/last.log" \
      | normalize | grep -v '^test result: ok\. 0 passed' | python3 "$MODEL/sort_lines.py"; } >> "$out"
}

mutant_copy() { # mutant_copy <副本名> <变异名>：同一个变异的阶段被打断后重起时不重拷，接着上次的编译
  local name="$1" mutant="$2"
  if [[ "$(cat "$WORK/$name.applied" 2>/dev/null)" != "$mutant" || ! -d "$WORK/$name" ]]; then
    copy_of "$name" "$mutant" > /dev/null
    echo "$mutant" > "$WORK/$name.applied"
  fi
}

stage() {
  local out="$WORK/out/$1.txt"
  rm -f "$out"
  case "$1" in
    base-build) copy_of base; cargo_in base test --offline --release -p opus-probe-b57 -p singlefs-checker --lib -p singlefs-harness \
        --test checker_known_bad_images --test container_trees_grow_a_level_when_the_root_is_full \
        --test corrupt_on_disk_content_is_refused_instead_of_panicking --no-run > /dev/null 2>&1; echo built > "$out" ;;
    base-fast) # 被判的几份用例（含第 57 批新用例）在副本上照绿，慢档另跑
      run_tests base "$out" -p singlefs-checker --lib
      run_tests base "$out" -p singlefs-harness --test checker_known_bad_images
      run_tests base "$out" -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full
      run_tests base "$out" -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking ;;
    base-slow) # 慢档：两条随机历史（旋钮 1、旋钮 2）、产品容量退役 960 次与回退 1360 次
      run_tests base "$out" -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full -- --ignored
      run_tests base "$out" -p singlefs-harness --test checker_known_bad_images -- --ignored ;;
    k1-random-b1|k1-random-b2|k1-random-b3|k1-random-b4)
      local first last ops mode
      case "$1" in
        k1-random-b1) first=1; last=10; ops=120; mode=all ;;
        k1-random-b2) first=11; last=20; ops=150; mode=all ;;
        k1-random-b3) first=21; last=30; ops=150; mode=one ;;
        k1-random-b4) first=31; last=40; ops=150; mode=one ;;
      esac
      OPUS_SEED_FIRST=$first OPUS_SEED_LAST=$last OPUS_OPS=$ops OPUS_THREADS="$THREADS" OPUS_MODE=$mode \
        run_tests base "$out" -p opus-probe-b57 --test k1_random_knob_histories ;;
    k4-middle) run_tests base "$out" -p opus-probe-b57 --test k4_middle_short_container ;;
    k3-probe) run_tests base "$out" -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- opus_k3 ;;
    mut-*)
      local mutant="${1#mut-}" copy=mutw
      if [[ "$mutant" == k3-* || "$mutant" == row385[01]* ]]; then copy=mutr; fi
      mutant_copy "$copy" "$mutant"
      case "$mutant" in
        k3-*|row385[01]*) run_tests "$copy" "$out" -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking ;;
        h1-first-only)
          run_tests "$copy" "$out" -p singlefs-checker --lib
          run_tests "$copy" "$out" -p singlefs-harness --test checker_known_bad_images
          run_tests "$copy" "$out" -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full -- --include-ignored
          run_tests "$copy" "$out" -p opus-probe-b57 --test k4_middle_short_container ;;
        k2-image-path-skips-product)
          run_tests "$copy" "$out" -p singlefs-checker --lib
          run_tests "$copy" "$out" -p singlefs-harness --test checker_known_bad_images
          run_tests "$copy" "$out" -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full
          run_tests "$copy" "$out" -p singlefs-harness --test checker_known_bad_images -- --ignored ;;
      esac ;;
    row-*) # 第 57 批变异表里不碰 .wgsl 的那一行：照它自己的 cargo 参数跑，看点名的测试红没红
      local number="${1#row-}" args expected
      mutant_copy mutw "row$number"
      IFS=$'\t' read -r _ args expected < <(python3 "$MODEL/mutants.py" --rows "$WORK/mutw" | grep "^$number"$'\t')
      # shellcheck disable=SC2086
      run_tests mutw "$out" $args
      if grep -qE "^    ([a-z_0-9]+::)*$expected\$" "$out"; then echo "row $number named-test-red=yes" >> "$out"; else echo "row $number named-test-red=NO" >> "$out"; fi ;;
    compare) python3 "$MODEL/compare.py" "$MODEL/expected" "$WORK/out" | tee "$out" ;;
    *) echo "不认识的阶段 $1" >&2; exit 2 ;;
  esac
}

all_stages() {
  echo base-build base-fast base-slow k1-random-b1 k1-random-b2 k1-random-b3 k1-random-b4 k4-middle k3-probe
  python3 "$MODEL/mutants.py" --list | sed 's/^/mut-/'
  echo mut-row3850 mut-row3851
  python3 "$MODEL/mutants.py" --rows "$REPO" | cut -f1 | grep -v -e '^3850$' -e '^3851$' | sed 's/^/row-/'
}

if [[ "${1:-}" == list ]]; then all_stages | tr ' ' '\n'; exit 0; fi
if [[ "${1:-}" == all ]]; then set -- $(all_stages) compare; fi
for name in "$@"; do stage "$name"; done
