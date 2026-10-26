#!/usr/bin/env bash
# differential.sh <仓根> <草稿目录>：改后（after）、改前（before）两份副本各编 tests/attack_followup_differential.rs，
# 按同一串种子跑随机合法操作，每一步一行；两份逐行比，打出段数、步数、逐行相同与否、走到的操作与留存条目搬迁的覆盖。
# 种子与步数：ATTACK_SEED_FIRST（默认 1）、ATTACK_SEEDS（默认 512）、ATTACK_STEPS（默认 64）；release 档编，种子分到 SINGLEFS_THREAD_CAP 条线程（capped.sh 设）。
set -u
repo=$1; work=$2
first=${ATTACK_SEED_FIRST:-1}; seeds=${ATTACK_SEEDS:-512}; steps=${ATTACK_STEPS:-64}
for copy in after before; do
  ( cd "$work/$copy" && CARGO_TARGET_DIR="$work/target-$copy" nice -n 19 bash "$repo/scripts/hosts/two-logs.sh" --quiet "$work/differential-build-$copy" -- \
      bash "$repo/scripts/hosts/capped.sh" 31 bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G \
      cargo test --offline --release -p singlefs-harness --test attack_followup_differential --no-run ) || { echo "name=differential build_failed=$copy"; exit 1; }
  binary=$(grep -o "$work/target-$copy/release/deps/attack_followup_differential-[0-9a-f]*" "$work/differential-build-$copy.log" | tail -1)
  ( cd "$work/$copy/crates/singlefs-harness" && ATTACK_SEED_FIRST=$first ATTACK_SEEDS=$seeds ATTACK_STEPS=$steps nice -n 19 \
      bash "$repo/scripts/hosts/capped.sh" 31 bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G \
      "$binary" --exact attack_followup_differential_history --nocapture ) > "$work/differential-$copy.txt" 2>&1
  echo "name=differential_run copy=$copy rc=$? $(grep -h "^test result" "$work/differential-$copy.txt" | tail -1 | sed "s/; finished in [0-9.]*s//")"
done
grep '^seed=' "$work/differential-after.txt" > "$work/differential-after.lines"
grep '^seed=' "$work/differential-before.txt" > "$work/differential-before.lines"
if cmp -s "$work/differential-after.lines" "$work/differential-before.lines"; then verdict=identical; else verdict=different; fi
lines=$(wc -l < "$work/differential-after.lines")
echo "name=differential seeds=$seeds steps=$steps lines=$lines before_vs_after=$verdict"
diff "$work/differential-before.lines" "$work/differential-after.lines" | head -5 | sed 's/^/name=differential_first_difference /'
for op in new_inodes write overwrite create_snapshot destroy_snapshot unlink compaction relocate raise_floor; do
  echo "name=differential_coverage op=$op steps=$(grep -c " $op" "$work/differential-after.lines") ok=$(grep -E " $op[^ ]* ok" "$work/differential-after.lines" | wc -l) err=$(grep -E " $op[^ ]* err:" "$work/differential-after.lines" | wc -l)"
done
echo "name=differential_coverage panics=$(grep -c ' panic:' "$work/differential-after.lines") kept_relocated_total=$(grep -o 'kept_relocated_total=[0-9]*' "$work/differential-after.lines" | cut -d= -f2 | paste -sd+ | bc) histories_with_kept_relocation=$(grep -o 'kept_relocated_total=[1-9][0-9]*' "$work/differential-after.lines" | wc -l) max_dupkept=$(grep -o 'dupkept=[0-9]*' "$work/differential-after.lines" | cut -d= -f2 | sort -n | tail -1) max_dupfiles=$(grep -o 'dupfiles=[0-9]*' "$work/differential-after.lines" | cut -d= -f2 | sort -n | tail -1)"
