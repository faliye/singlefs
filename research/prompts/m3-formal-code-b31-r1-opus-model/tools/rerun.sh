#!/bin/bash
# 第 31 批代码轮第一轮攻方腿（m3-formal-code-b31-r1 opus）的复跑入口。主 agent 在自己的草稿目录里照它跑，不在腿的目录里跑。
# 用法：bash tools/rerun.sh <草稿目录> <步骤>
#   copy        冻结副本 rsync 进 <草稿目录>/copy（排除 target），拷 .cargo/config.toml，把原型用例放进 harness 的 tests/
#   build       编 harness 档全部测试二进制（release；第一次从零编约 3 分钟以上，超过前台 240 秒的话再起一次接着编）
#   ignored     harness 档 65 条 #[ignore] 慢档用例逐条跑（两条读环境变量的大档除外），日志在 <草稿目录>/ign/
#   mutants     baseline 与 R1 R2 R3 R4 B3 row:2736 各一趟（tools/mutant-run.sh，每趟约 80 秒），日志在 <草稿目录>/mutants/
#   sweep       场景乙：种子 0–3 × 60 步、100–139 × 80 步、200–219 × 150 步（tools/sweep.sh，每批约 2–3 分钟）
#   on-device   on_device_modes 根环表原型（on-device-ring-table/），编约 30 秒、跑约 6 秒
# 每条命令经 run-with-memory-cap.sh（上限 8G；原型二进制 auto）与 capped.sh 5；同会话有别的子 agent 在跑时设好下面两个理由。
set -euo pipefail
MODEL="$(cd "$(dirname "$0")/.." && pwd)"
REPO="$(cd "$MODEL/../../.." && pwd)"
FROZEN=/tmp/claude-1000/m3-formal-code-b31-r1/tree
D="$1"; step="$2"
CAP="$REPO/scripts/hosts/run-with-memory-cap.sh"
CAPPED="$REPO/scripts/hosts/capped.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-第31批攻方腿复跑：主agent照tools/rerun.sh复核}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示要求cargo一律显式给8G上限}"
export CARGO_TARGET_DIR="$D/target"
mkdir -p "$D"
case "$step" in
  copy)
    mkdir -p "$D/copy"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$D/copy/"
    mkdir -p "$D/copy/.cargo" && cp "$REPO/.cargo/config.toml" "$D/copy/.cargo/"
    cp "$MODEL/harness-tests/b31_opus_n3_reclaim_predicate_and_ring_table.rs" "$D/copy/crates/singlefs-harness/tests/"
    diff -rq "$FROZEN/crates" "$D/copy/crates" || true ;;
  build)
    cd "$D/copy" && nice -n 19 bash "$CAP" 8G bash "$CAPPED" 5 cargo build --release --offline -p singlefs-harness --tests ;;
  ignored)
    mkdir -p "$D/ign"
    cd "$D/copy" && nice -n 19 bash "$CAP" 8G bash "$CAPPED" 5 cargo test --release --offline -p singlefs-harness --no-run --message-format=json 2>/dev/null \
      | python3 -c 'import json,sys
for l in sys.stdin:
    try: m=json.loads(l)
    except Exception: continue
    if m.get("reason")=="compiler-artifact" and m.get("executable") and m["profile"]["test"]: print(m["target"]["name"], m["executable"])' > "$D/bins.txt"
    cd "$D/copy/crates/singlefs-harness"
    while read -r name binary; do
      for test in $(nice -n 19 bash "$CAP" 2G "$binary" --list --ignored 2>/dev/null | sed -n 's/: test$//p'); do
        case "$test" in *large_tier*|*from_the_environment*) continue ;; esac
        nice -n 19 bash "$CAP" 4G "$binary" --ignored --exact "$test" > "$D/ign/$name--$test.log" 2>&1 && rc=0 || rc=$?
        echo "$name::$test rc=$rc" | tee -a "$D/ign/summary.txt"
      done
    done < "$D/bins.txt" ;;
  mutants)
    for name in baseline R1-floor-plus-one R2-floor-minus-one R3-release-generation-minus-one R4-record-the-root-before-taking-placements B3-target-for-publish-one-slot-short row:2736 baseline; do
      nice -n 19 bash "$CAP" 8G bash "$CAPPED" 5 bash "$MODEL/tools/mutant-run.sh" "$D" "$name" || true
    done ;;
  sweep)
    nice -n 19 bash "$CAP" 8G bash "$MODEL/tools/sweep.sh" "$D" s1 0 4 1 60
    nice -n 19 bash "$CAP" 8G bash "$MODEL/tools/sweep.sh" "$D" s2 100 10 4 80
    nice -n 19 bash "$CAP" 8G bash "$MODEL/tools/sweep.sh" "$D" s3 200 5 4 150 ;;
  on-device)
    mkdir -p "$D/proto/src" "$D/proto/.cargo"
    sed "s#@COPY@#$D/copy#g" "$MODEL/on-device-ring-table/Cargo.toml.template" > "$D/proto/Cargo.toml"
    cp "$MODEL/on-device-ring-table/src/main.rs" "$D/proto/src/" && cp "$D/copy/Cargo.lock" "$D/proto/" && cp "$REPO/.cargo/config.toml" "$D/proto/.cargo/"
    cd "$D/proto" && CARGO_TARGET_DIR="$D/target-proto" nice -n 19 bash "$CAP" 8G bash "$CAPPED" 5 cargo build --release --offline
    nice -n 19 bash "$CAP" auto "$D/target-proto/release/b31-opus-on-device-ring-table" ;;
  *) echo "认不得的步骤：$step"; exit 2 ;;
esac
