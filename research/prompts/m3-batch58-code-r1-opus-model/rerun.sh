#!/usr/bin/env bash
# 攻方腿（m3-batch58-code-r1 opus）的复跑：rerun.sh <工作目录> <阶段>…（或 all；阶段表 `rerun.sh <工作目录> list`）。
# 环境：SINGLEFS_REPO（被判的冻结副本，缺省 ~/.local/share/singlefs/work/m3-batch58-code-r1/repo）、SINGLEFS_SCRIPTS（仓的 scripts，
# 缺省按模型目录在仓里的位置推）、OPUS_THREADS（线程上限，缺省 10）、OPUS_MEMORY_CAP（内存包装上限，缺省 3G；K2 那一段 OPUS_K2_MEMORY_CAP，缺省 10G：4G 时撞上限）。
# 副本只拷 Cargo.toml、Cargo.lock、crates/；原型 crate 拷成副本工作区成员 crates/opus-probe-b58（测试目标名不带 layer0，不开 gpu 特性）；
# K3 原型片段接到副本里 corrupt_on_disk_content_is_refused_instead_of_panicking.rs 的末尾。base 与 mut（打变异的那一份）两个副本各用自己的
# CARGO_TARGET_DIR；同一时刻只起一条 cargo，每条经 capped.sh 与 run-with-memory-cap.sh。产物写 <工作目录>/out/<阶段>.txt。
set -euo pipefail
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${SINGLEFS_REPO:-$HOME/.local/share/singlefs/work/m3-batch58-code-r1/repo}"
SCRIPTS="${SINGLEFS_SCRIPTS:-$MODEL/../../scripts}"
THREADS="${OPUS_THREADS:-10}"
CAP="${OPUS_MEMORY_CAP:-3G}"
K2_CAP="${OPUS_K2_MEMORY_CAP:-10G}"
WORK="$(mkdir -p "$1" && cd "$1" && pwd)"
shift
mkdir -p "$WORK/out" "$WORK/tmp"
export TMPDIR="$WORK/tmp"
CORRUPT=crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs

copy_of() { # copy_of <名> [变异名]：建副本、装原型 crate 与 K3 片段、打变异；变异副本把可能被改过的几份 touch，逼 cargo 重编
  local name="$1" mutant="${2:-}"
  local dir="$WORK/$name"
  rm -rf "$dir"; mkdir -p "$dir"
  cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$dir/"
  rsync -a --exclude target "$REPO/crates/" "$dir/crates/"
  cp -r "$MODEL/probe-crate" "$dir/crates/opus-probe-b58"
  cat "$MODEL/probes/corrupt_on_disk_k3_append.rs" >> "$dir/$CORRUPT"
  python3 - "$dir/Cargo.toml" <<'PY'
import sys
from pathlib import Path
p = Path(sys.argv[1]); s = p.read_text()
old = '    "crates/singlefs-reference-filesystem",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, old + '    "crates/opus-probe-b58",\n'))
PY
  if [[ "$name" == mut* ]]; then
    (cd "$dir" && touch crates/singlefs-checker/src/walk.rs crates/singlefs-core/src/recovery.rs)
  fi
  if [[ -n "$mutant" ]]; then python3 "$MODEL/mutants.py" "$dir" "$mutant"; fi
}

cargo_in() { # cargo_in <副本名> <cargo 参数…>：经线程上限跑；跑编出来的代码的（不带 --no-run）再经内存包装，上限 $CAP（K2 那一段另设）
  local name="$1"; shift
  if [[ " $* " == *" --no-run "* ]]; then
    (cd "$WORK/$name" && CARGO_TARGET_DIR="$WORK/target-$name" nice -n 19 bash "$SCRIPTS/capped.sh" "$THREADS" cargo "$@")
  else
    (cd "$WORK/$name" && CARGO_TARGET_DIR="$WORK/target-$name" nice -n 19 bash "$SCRIPTS/capped.sh" "$THREADS" \
      bash "$SCRIPTS/run-with-memory-cap.sh" "$CAP" cargo "$@")
  fi
}

normalize() { sed -E 's/finished in [0-9.]+s//g; s/built_in_s=[0-9]+//g'; }

run_tests() { # run_tests <副本名> <输出> <cargo test 参数…>（参数里可带 -- 之后的过滤）：同一组参数先只编不跑（不排内存的队），再经内存包装跑；只留结果行、失败名与原型的 OPUS 行
  local name="$1" out="$2"; shift 2
  local args=() after=() seen=0
  for a in "$@"; do
    if [[ $seen == 1 ]]; then after+=("$a"); elif [[ "$a" == -- ]]; then seen=1; elif [[ "$a" != --release ]]; then args+=("$a"); fi
  done
  set +e
  cargo_in "$name" test --offline --release "${args[@]}" --no-run > "$WORK/tmp/last-build.log" 2>&1
  OPUS_THREADS="$THREADS" cargo_in "$name" test --offline --release "${args[@]}" -- --nocapture "${after[@]}" > "$WORK/tmp/last.log" 2>&1
  local rc=$?
  set -e
  { echo "command args=$* rc=$rc"; grep -E '^OPUS|^test result|^failures:$|^    [a-z_0-9:]+$|^thread .* panicked' "$WORK/tmp/last.log" \
      | normalize | grep -v '^test result: ok\. 0 passed' | python3 "$MODEL/sort_lines.py"; } >> "$out"
}

mutant_copy() { # mutant_copy <变异名>：同一个变异的阶段被打断后重起时不重拷，接着上次的编译
  local mutant="$1"
  if [[ "$(cat "$WORK/mut.applied" 2>/dev/null)" != "$mutant" || ! -d "$WORK/mut" ]]; then
    copy_of mut "$mutant" > /dev/null
    echo "$mutant" > "$WORK/mut.applied"
  fi
}

K1_SLOW_FILTER=records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two

k1_mutant_stages() { # walk.rs 的改坏：GPU 三条流的 CPU 替身、被判的坏镜像（默认档与三条慢档）、checker 单测、容器树长层
  local copy="$1" out="$2"
  run_tests "$copy" "$out" -p opus-probe-b58 --test k1_gpu_flow_proxy
  run_tests "$copy" "$out" -p singlefs-checker --lib
  run_tests "$copy" "$out" -p singlefs-harness --test checker_known_bad_images
  run_tests "$copy" "$out" -p singlefs-harness --test checker_known_bad_images -- --ignored "$K1_SLOW_FILTER"
  run_tests "$copy" "$out" -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full
}

build_k1() { # 先只编不跑（不经内存包装，不排内存的队）
  cargo_in "$1" test --offline --release -p opus-probe-b58 -p singlefs-checker --lib -p singlefs-harness \
    --test checker_known_bad_images --test container_trees_grow_a_level_when_the_root_is_full --no-run > /dev/null 2>&1
}

stage() {
  local out="$WORK/out/$1.txt"
  rm -f "$out"
  case "$1" in
    base-build) copy_of base; cargo_in base test --offline --release -p opus-probe-b58 -p singlefs-checker --lib -p singlefs-harness \
        --test checker_known_bad_images --test container_trees_grow_a_level_when_the_root_is_full \
        --test corrupt_on_disk_content_is_refused_instead_of_panicking --no-run > /dev/null 2>&1; echo built > "$out" ;;
    base-k1) k1_mutant_stages base "$out" ;;
    base-k3)
      run_tests base "$out" -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking ;;
    k2-j4) CAP="$K2_CAP" run_tests base "$out" -p opus-probe-b58 --test k2_j4_state_sets ;;
    mut-w-*) mutant_copy "${1#mut-}"; build_k1 mut; k1_mutant_stages mut "$out" ;;
    mut-r-*|mut-row*)
      mutant_copy "${1#mut-}"
      cargo_in mut test --offline --release -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking --no-run > /dev/null 2>&1
      run_tests mut "$out" -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking ;;
    compare) python3 "$MODEL/compare.py" "$MODEL/expected" "$WORK/out" | tee "$out" ;;
    *) echo "不认识的阶段 $1" >&2; exit 2 ;;
  esac
}

all_stages() {
  echo base-build base-k1 base-k3 k2-j4
  python3 "$MODEL/mutants.py" --list | sed 's/^/mut-/'
  python3 "$MODEL/mutants.py" --rows "$REPO" | cut -f1 | awk '$1 >= 3859' | sed 's/^/mut-row/'
}

if [[ "${1:-}" == list ]]; then all_stages | tr ' ' '\n'; exit 0; fi
if [[ "${1:-}" == all ]]; then set -- $(all_stages) compare; fi
for name in "$@"; do stage "$name"; done
