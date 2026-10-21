#!/usr/bin/env bash
# seed_runs.sh <随机历史扫描二进制>：按固定的四个种子各跑一段，打 DEFENSE_SEED 行（截到 330 字）与零搬迁批的跟踪行；四段并行。
set -u
BIN="$1"; REPO_ROOT="${REPO_ROOT:?}"; WORK="${WORK:?}"
mkdir -p "$WORK/logs"
run_one() {  # run_one <标签> <比重> <盘宽> <种子偏移>
  DEFENSE_TRACE=1 DEFENSE_WEIGHTS="$2" DEFENSE_WIDTH="$3" DEFENSE_SEED_START="$4" DEFENSE_SEED_COUNT=1 DEFENSE_OPS=40 DEFENSE_CHECK=none \
    nice -n 19 timeout 215 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 4G "$BIN" --ignored --nocapture --exact defense_random_sweep \
    > "$WORK/logs/seed-$1.log" 2>&1
  echo "$?" > "$WORK/logs/seed-$1.exit"
}
rm -f "$WORK"/logs/seed-*.log "$WORK"/logs/seed-*.exit
run_one rollback-940015 compaction_rollback 4g 940015 &
run_one rollback-940105 compaction_rollback 4g 940105 &
run_one rollback-950093 compaction_rollback 4g 950093 &
run_one plain-256-960004 compaction 256 960004 &
wait
for tag in rollback-940015 rollback-940105 rollback-950093 plain-256-960004; do
  echo "SEEDRUN tag=$tag exit=$(cat "$WORK/logs/seed-$tag.exit")"
  grep -h '^DEFENSE_SEED' "$WORK/logs/seed-$tag.log" | cut -c1-330
  grep -h '^DEFENSE_TRACE kind=' "$WORK/logs/seed-$tag.log" | grep 'relocated=0'
done
