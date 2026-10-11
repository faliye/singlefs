#!/usr/bin/env bash
# m3-w3-r1 攻方腿（Opus）的复跑。用法：SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录> <段>
# 段：build（工作目录要是空的：拷冻结树、核清单、生成原型装置、编 P 与 V1 两个测试二进制、编抽象模型）、quick（两臂快档）、
#     device-k1a（G-long V1 第 24 段逐步打印）、device-k1b（F2 V1 四段 Gone、定制族 V1 两段逐步打印）、model-1 / model-2 / model-3（抽象模型）、
#     compare（bash compare.sh 期望目录 <工作目录>/out），
#     可选的长段 glong-v1-a、glong-v1-b、f2-v1、glong-p-1..4、f2-p-1..4、crafted-v1-1..3（副本装置全量，合计约 35 分钟）。
# 每段一条前台命令跑得完（4 分钟以内）；编译与跑都经 scripts/hosts/run-with-memory-cap.sh（编译 6G、跑 4G），线程 6。
# 冻结树取 FROZEN_TREE（默认 ~/.local/share/singlefs/work/m3-w3-r1/tree），对 $SINGLEFS_REPO/research/prompts/m3-w3-r1-snapshot/tree-sha256.txt 核。
set -euo pipefail
W=${1:?工作目录}; PART=${2:?段}
R=${SINGLEFS_REPO:?设 SINGLEFS_REPO 为仓根}
M=$(cd "$(dirname "$0")" && pwd)
FROZEN=${FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-w3-r1/tree}
E=$R/research/results/e175-product-path-inode-watermark-regression-2026-10-08
CAP="bash $R/scripts/hosts/run-with-memory-cap.sh"
THREADS="bash $R/scripts/hosts/capped.sh 6"
mkdir -p "$W/out" "$W/bin"
device() { # $1 臂 $2.. 环境变量赋值；跑一族的一组序号进结果文件
  local arm=$1; shift
  (cd "$W" && env E175_ARM=$arm E175_WORKER_THREADS=6 E175_TIME_BUDGET_SECONDS=100000 "$@" \
    nice -n 19 $THREADS $CAP 4G "$W/bin/w3-$arm" --ignored --exact run_one_family_on_one_arm_into_the_result_file >/dev/null)
}
crafted() { # $1 臂 $2 序号 $3 结果文件 $4 逐步打印的序号
  (cd "$W" && env E175_ARM=$1 W3_RESULT_FILE=$3 E175_WORKER_THREADS=6 W3_INDICES=$2 W3_TRACE_INDICES=${4:-} \
    nice -n 19 $THREADS $CAP 4G "$W/bin/w3-$1" --ignored --exact w3_run_crafted_indices_into_the_result_file >/dev/null)
}
model() { nice -n 19 $CAP 4G "$W/target-model/release/w3-timeline-model" "$@"; }
case "$PART" in
  build)
    [ -z "$(ls -A "$W" | grep -v '^out$\|^bin$')" ] || { echo "工作目录不空：$W" >&2; exit 2; }
    rsync -a --exclude target --exclude .git "$FROZEN/" "$W/repo/"
    (cd "$W/repo" && sha256sum -c --quiet "$R/research/prompts/m3-w3-r1-snapshot/tree-sha256.txt")
    python3 "$M/make_device.py" "$E/device-source.rs" "$W/repo/crates/singlefs-harness/tests/w3_attack_candidates.rs"
    build() {
      (cd "$W/repo" && CARGO_TARGET_DIR=$W/target nice -n 19 $THREADS $CAP 6G cargo test --release --offline -p singlefs-harness \
        --test w3_attack_candidates --no-run --message-format=json 2>>"$W/build.log") | python3 -c 'import sys,json
for l in sys.stdin:
    try: m=json.loads(l)
    except Exception: continue
    if m.get("reason")=="compiler-artifact" and m.get("target",{}).get("name")=="w3_attack_candidates" and m.get("executable"): print(m["executable"])'
    }
    exe=$(build); cp "$exe" "$W/bin/w3-P"
    cp "$W/repo/crates/singlefs-core/src/mount.rs" "$W/mount.rs.pristine"
    patch -s "$W/repo/crates/singlefs-core/src/mount.rs" < "$E/patch-V1.diff"
    exe=$(build); cp "$exe" "$W/bin/w3-V1"
    cp "$W/mount.rs.pristine" "$W/repo/crates/singlefs-core/src/mount.rs"
    (cd "$W/repo" && sha256sum -c --quiet "$R/research/prompts/m3-w3-r1-snapshot/tree-sha256.txt" && echo "冻结树副本已还原")
    rsync -a --exclude target "$M/timeline-model/" "$W/timeline-model/"
    (cd "$W/timeline-model" && CARGO_TARGET_DIR=$W/target-model nice -n 19 $THREADS $CAP 6G cargo build --release --offline 2>>"$W/build.log")
    sha256sum "$W/bin/w3-P" "$W/bin/w3-V1" "$W/target-model/release/w3-timeline-model" ;;
  quick)
    for arm in P V1; do (cd "$W" && E175_ARM=$arm nice -n 19 $THREADS $CAP 4G "$W/bin/w3-$arm" --test-threads 6 2>&1 | grep '^test result') ; done ;;
  device-k1a)
    rm -f "$W/out/device-g-long-V1-index24.out"
    device V1 E175_FAMILY=G-long E175_RESULT_FILE=$W/out/device-g-long-V1-index24.out E175_INDICES=24 W3_TRACE_INDICES=24 ;;
  device-k1b)
    rm -f "$W/out/device-f2-V1-gone.out" "$W/out/device-crafted-V1.out"
    device V1 E175_FAMILY=F2 E175_RESULT_FILE=$W/out/device-f2-V1-gone.out E175_INDICES=10,43,49,60
    crafted V1 28,692 "$W/out/device-crafted-V1.out" 28,692 ;;
  model-1) for i in 0 1 2; do model main $i > "$W/out/model-main-$i.out"; done ;;
  model-2) for i in 3 4 5; do model main $i > "$W/out/model-main-$i.out"; done ;;
  model-3) model mutation > "$W/out/model-mutation.out"; model ring6 > "$W/out/model-ring6.out" ;;
  glong-v1-a) device V1 E175_FAMILY=G-long E175_RESULT_FILE=$W/out/full-g-long-V1.out E175_INDICES=$(seq -s, 0 15) ;;
  glong-v1-b) device V1 E175_FAMILY=G-long E175_RESULT_FILE=$W/out/full-g-long-V1.out E175_INDICES=$(seq -s, 16 31) ;;
  f2-v1) device V1 E175_FAMILY=F2 E175_RESULT_FILE=$W/out/full-f2-V1.out E175_INDICES=$(seq -s, 0 63) ;;
  glong-p-[1-4]) k=${PART#glong-p-}; device P E175_FAMILY=G-long E175_RESULT_FILE=$W/out/full-g-long-P.out E175_INDICES=$(seq -s, $(( (k-1)*16 )) $(( k*16-1 ))) ;;
  f2-p-[1-4]) k=${PART#f2-p-}; device P E175_FAMILY=F2 E175_RESULT_FILE=$W/out/full-f2-P.out E175_INDICES=$(seq -s, $(( (k-1)*64 )) $(( k*64-1 ))) ;;
  crafted-v1-1) crafted V1 "$(seq -s, 0 1499)" "$W/out/full-crafted-V1.out" ;;
  crafted-v1-2) crafted V1 "$(seq -s, 1500 3299)" "$W/out/full-crafted-V1.out" ;;
  crafted-v1-3) crafted V1 "$(seq -s, 3300 5219)" "$W/out/full-crafted-V1.out" ;;
  compare) bash "$M/compare.sh" "$M/expected" "$W/out" ;;
  *) echo "不认的段：$PART" >&2; exit 2 ;;
esac
echo "段 $PART 跑完"
