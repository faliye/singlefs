#!/usr/bin/env bash
# unreadable-at-mount-r2 云端攻方腿复跑：从冻结副本建五份副本（今天 base、甲 jia、甲-读错 jiaeio、(a) a、(b)-从树表现算 b），
# 另有我提的两份收窄 / 放宽：甲-读错-每盘 jiaeiodev、(b)-从映射现算 bmap（被攻过零轮）。
# 打第一轮的 common-mod.rs.diff 与本目录的 common-mod-memory.rs.diff（镜像改在内存稀疏盘上），再打各自候选的 diff；
# 拷进两张网格用例与轨迹原型，编译、跑、汇总。用法：
#   bash research/scripts/run-with-memory-cap.sh 16G bash rerun.sh <冻结副本根（含 crates/）> <工作目录（新建）>
set -euo pipefail
FROZEN=${1:?冻结副本根}; WORK=${2:?工作目录}
MODEL=$(cd "$(dirname "$0")" && pwd)
REPO=/home/fy5090/code/singlefs
R1=$REPO/research/prompts/unreadable-at-mount-r1-opus-model
CAP=$REPO/research/scripts/capped.sh
mkdir -p "$WORK/runs" "$WORK/tmp"
for copy in base jia jiaeio jiaeiodev a b bmap; do
  mkdir -p "$WORK/$copy/.cargo"
  cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$WORK/$copy/"; cp "$REPO/.cargo/config.toml" "$WORK/$copy/.cargo/"
  rsync -a --exclude target "$FROZEN/crates" "$WORK/$copy/"
  (cd "$WORK/$copy" && patch -p1 --quiet < "$R1/common-mod.rs.diff" && patch -p1 --quiet < "$MODEL/common-mod-memory.rs.diff")
  case $copy in
    jia) (cd "$WORK/$copy" && patch -p1 --quiet < "$R1/candidate-jia.diff") ;;
    jiaeio) (cd "$WORK/$copy" && patch -p1 --quiet < "$R1/candidate-jia.diff" && patch -p1 --quiet < "$MODEL/candidate-jiaeio-over-jia.diff") ;;
    jiaeiodev) (cd "$WORK/$copy" && patch -p1 --quiet < "$R1/candidate-jia.diff" && patch -p1 --quiet < "$MODEL/candidate-jiaeio-over-jia.diff" && patch -p1 --quiet < "$MODEL/candidate-jiaeiodev-over-jiaeio.diff") ;;
    a) (cd "$WORK/$copy" && patch -p1 --quiet < "$R1/candidate-a.diff") ;;
    bmap) (cd "$WORK/$copy" && patch -p1 --quiet < "$R1/candidate-a.diff" && patch -p1 --quiet < "$MODEL/candidate-b-over-a.diff" && patch -p1 --quiet < "$MODEL/candidate-bmap-over-b.diff") ;;
    b) (cd "$WORK/$copy" && patch -p1 --quiet < "$R1/candidate-a.diff" && patch -p1 --quiet < "$MODEL/candidate-b-over-a.diff") ;;
    base) ;;
  esac
  cp "$R1/c331_candidates_grid.rs" "$R1/c393_candidates_grid.rs" "$WORK/$copy/crates/singlefs-harness/tests/"
  mkdir -p "$WORK/$copy/crates/r2-opus-trajectory-proto/src"
  cp "$MODEL/r2-opus-trajectory-proto/Cargo.toml" "$WORK/$copy/crates/r2-opus-trajectory-proto/"
  cp "$MODEL/r2-opus-trajectory-proto/src/main.rs" "$WORK/$copy/crates/r2-opus-trajectory-proto/src/"
  sed -i 's#    "crates/singlefs-checker-tier",#    "crates/singlefs-checker-tier",\n    "crates/r2-opus-trajectory-proto",#' "$WORK/$copy/Cargo.toml"
  (cd "$WORK/$copy" && CARGO_TARGET_DIR="$WORK/target-$copy" nice -n 19 bash "$CAP" 16 cargo build --offline --release -p r2-opus-trajectory-proto \
     && CARGO_TARGET_DIR="$WORK/target-$copy" nice -n 19 bash "$CAP" 16 cargo test --offline --release -p singlefs-harness --test c331_candidates_grid --test c393_candidates_grid --no-run)
done
grid() {
  (cd "$WORK/$1" && TMPDIR="$WORK/tmp" CARGO_TARGET_DIR="$WORK/target-$1" SINGLEFS_R2_B_COMPARE=1 nice -n 19 bash "$CAP" 16 \
     cargo test --offline --release -p singlefs-harness --test "$2" -- --nocapture > "$WORK/runs/$1-$2.log" 2>&1) || echo "grid $1 $2 exit=$?"
}
for copy in base a b bmap; do grid $copy c393_candidates_grid; done
for copy in base jia jiaeio jiaeiodev; do grid $copy c331_candidates_grid; done
for copy in base jia jiaeio jiaeiodev a b bmap; do
  for part in torn single_slot c393_single c393_double; do
    TMPDIR="$WORK/tmp" nice -n 19 "$WORK/target-$copy/release/r2-opus-trajectory-proto" "$part" > "$WORK/runs/$copy-traj-$part.log" 2>&1 || echo "traj $copy $part exit=$?"
  done
done
python3 "$R1/summarize_c331.py" "$WORK/runs" base jia jiaeio jiaeiodev > "$WORK/c331-summary.txt"
python3 "$R1/summarize_c393.py" "$WORK/runs" "" base a b bmap > "$WORK/c393-summary.txt"
python3 "$MODEL/summarize_trajectories.py" "$WORK/runs" base jia jiaeio jiaeiodev a b bmap > "$WORK/trajectory-summary.txt"
echo "done: $WORK/c331-summary.txt $WORK/c393-summary.txt $WORK/trajectory-summary.txt"
