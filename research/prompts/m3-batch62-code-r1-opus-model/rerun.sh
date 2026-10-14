#!/usr/bin/env bash
# 攻方腿（m3-batch62-code-r1 opus）的复跑：rerun.sh <工作目录> <阶段>…（或 all；阶段表 `rerun.sh <工作目录> list`）。
# 环境：SINGLEFS_REPO（被判的冻结副本，缺省 ~/.local/share/singlefs/work/m3-batch62-code-r1/repo）、SINGLEFS_SCRIPTS（仓的 scripts/hosts，
# 缺省按模型目录在仓里的位置推）、OPUS_THREADS（线程上限，缺省 15）、OPUS_MEMORY_CAP（内存包装上限，缺省 16G）。
# 副本只拷 Cargo.toml、Cargo.lock、crates/；expose.py 在副本的 verdict_store.rs 里加三个公开包装与一行设了 OPUS_TRACE_BATCHES 才打的批账；
# 原型 crate 拷成副本工作区成员 crates/opus-probe-b62（只依赖判定库，测试目标 batch62_store_probe，名字不带 layer0，不开 gpu 特性）。
# 不跑 checker 档包的任何测试：被判的三条单测在原型里逐条照抄（mirror_*），跑的是原型 crate 的测试二进制。
# base 与变异副本 mut 共用一个编译目录 target（release）；同一时刻只起一条 cargo，先只编不跑，再经内存包装跑。产物写 <工作目录>/out/<阶段>.txt，
# 只留 OPUS_ 读数行、OPUS_BATCH 批账行、每条用例的结局行与断言消息（extract 那一段）。compare 把 out/ 与模型目录 expected/ 逐份比。
# 一次起完：rerun.sh <工作目录> all；前台时限内编不完时分段：prep base → build base（重起到 rc=0）→ probe base base，
# 变异照 prep mut <变异> → build mut → probe mut mut-<变异>。
set -euo pipefail
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${SINGLEFS_REPO:-$HOME/.local/share/singlefs/work/m3-batch62-code-r1/repo}"
HOSTS="${SINGLEFS_SCRIPTS:-$MODEL/../../../scripts/hosts}"
THREADS="${OPUS_THREADS:-15}"
CAP="${OPUS_MEMORY_CAP:-16G}"
WORK="$(mkdir -p "$1" && cd "$1" && pwd)"
shift
mkdir -p "$WORK/out" "$WORK/tmp"
export TMPDIR="$WORK/tmp"
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"
TARGET=(-p opus-probe-b62 --test batch62_store_probe)
MUTANTS=(flip take0 facts-after take1 rev)

copy_of() { # copy_of <名> [变异名]：建副本、加包装、装原型 crate、打变异
  local name="$1" mutant="${2:-}" dir="$WORK/$1"
  rm -rf "${dir:?}"; mkdir -p "$dir"
  cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$dir/"
  rsync -a --exclude target "$REPO/crates/" "$dir/crates/"
  cp -r "$MODEL/probe-crate" "$dir/crates/opus-probe-b62"
  python3 - "$dir/Cargo.toml" <<'PY'
import sys
from pathlib import Path
manifest = Path(sys.argv[1])
text = manifest.read_text()
old = '    "crates/singlefs-reference-filesystem",\n'
assert text.count(old) == 1, "根 Cargo.toml 的成员表认不出"
manifest.write_text(text.replace(old, old + '    "crates/opus-probe-b62",\n'))
PY
  python3 "$MODEL/expose.py" "$dir"
  if [[ -n "$mutant" ]]; then python3 "$MODEL/mutants.py" "$dir" "$mutant"; fi
  # 第一次拿种子编译目录编之前全部 touch（种子里的产物按别的副本的时刻记）；同一个工作目录里重拷副本时可设 OPUS_NO_TOUCH=1：
  # 拷来的文件带冻结副本的原时刻（比上一次编好的产物旧），只有 expose.py、mutants.py 改过的 verdict_store.rs 与原型 crate 是新的，只重编这两个包
  if [[ "${OPUS_NO_TOUCH:-}" != 1 ]]; then (cd "$dir" && find crates -type f -exec touch {} +); fi
}

cargo_in() { # cargo_in <副本名> <cargo 参数…>：经内存包装与线程上限
  local name="$1"; shift
  (cd "$WORK/$name" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$HOSTS/run-with-memory-cap.sh" "$CAP" \
    bash "$HOSTS/capped.sh" "$THREADS" cargo "$@")
}

extract() { # 留读数行、批账行、用例结局行与断言消息；线程号与耗时每次不同，抹掉，好与 expected/ 逐字比
  { grep -E '^(OPUS_|test |test result:|thread .* panicked|  left:|  right:|.*(两份|照产品|一字节|多批|编号|F[0-9]|核对|前提|事实|路径|按计划|写下|导入之后).*)' || true; } \
    | sed -E 's/ \([0-9]+\) panicked/ panicked/; s/finished in [0-9.]+s/finished in Xs/'
}

build_copy() { # build_copy <副本名>：只编不跑（编到一半被前台时限打断的，再起一次接着编；副本不重拷，不重编已编好的）
  local name="$1" build_rc=0
  cargo_in "$name" test --offline --release "${TARGET[@]}" --no-run >> "$WORK/tmp/$name-build.log" 2>&1 || build_rc=$?
  echo "build $name rc=$build_rc"
  return "$build_rc"
}

run_probe() { # run_probe <副本名> <输出>：先只编（已编好的不重编）、再跑原型：扫批长那一条之外的五条（打批账），扫批长那一条另起（不打批账）
  local name="$1" out="$2" build_rc=0 rc=0
  build_copy "$name" > /dev/null || build_rc=$?
  echo "build_rc=$build_rc" >> "$out"
  if [[ $build_rc != 0 ]]; then tail -30 "$WORK/tmp/$name-build.log" >> "$out"; return 0; fi
  # 库建在 tmpfs 下（OPUS_RUN_TMP，缺省 /dev/shm/opus-b62-run）：同步写在满载的盘上排队，扫批长那一条在盘上前台跑不完；杀的不是机器，不影响判的东西
  local run_tmp="${OPUS_RUN_TMP:-/dev/shm/opus-b62-run}"
  mkdir -p "$run_tmp"
  OPUS_TRACE_BATCHES=1 TMPDIR="$run_tmp" cargo_in "$name" test --offline --release --no-fail-fast "${TARGET[@]}" -- --nocapture --test-threads 1 \
    --skip sweep_batch_lengths_against_one_batch > "$WORK/tmp/$name-run.log" 2>&1 || rc=$?
  echo "run_rc=$rc" >> "$out"
  extract < "$WORK/tmp/$name-run.log" >> "$out"
  rc=0
  TMPDIR="$run_tmp" cargo_in "$name" test --offline --release --no-fail-fast "${TARGET[@]}" -- --nocapture --test-threads 1 \
    --exact sweep_batch_lengths_against_one_batch > "$WORK/tmp/$name-sweep.log" 2>&1 || rc=$?
  echo "sweep_rc=$rc" >> "$out"
  extract < "$WORK/tmp/$name-sweep.log" >> "$out"
}

stage() {
  local out="$WORK/out/$1.txt"
  rm -f "$out"
  case "$1" in
    base) copy_of base; run_probe base "$out" ;;
    mut-*) local mutant="${1#mut-}"
      [[ " ${MUTANTS[*]} " == *" $mutant "* ]] || { echo "不认识的变异 $mutant" >&2; return 2; }
      copy_of mut "$mutant"; run_probe mut "$out" ;;
    *) echo "不认识的阶段 $1" >&2; return 2 ;;
  esac
}

# 分段起（前台时限内编不完时）：prep <副本名> [变异]、build <副本名>（可重起）、probe <副本名> <阶段名>（写 out/<阶段名>.txt）
case "${1:-}" in
  prep) copy_of "$2" "${3:-}"; [[ -z "${3:-}" ]] || echo "$3" > "$WORK/$2.mutant"; exit 0 ;;
  build) build_copy "$2"; exit $? ;;
  probe) rm -f "$WORK/out/$3.txt"; run_probe "$2" "$WORK/out/$3.txt"; exit 0 ;;
esac
if [[ "${1:-}" == list ]]; then printf '%s\n' base "${MUTANTS[@]/#/mut-}"; exit 0; fi
if [[ "${1:-}" == all ]]; then set -- base "${MUTANTS[@]/#/mut-}" compare; fi
for name in "$@"; do
  if [[ "$name" == compare ]]; then
    for expected in "$MODEL"/expected/*.txt; do
      if cmp -s "$expected" "$WORK/out/$(basename "$expected")"; then echo "一致 $(basename "$expected")"; else echo "不一致 $(basename "$expected")"; fi
    done
  else stage "$name"; fi
done
