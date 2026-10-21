#!/usr/bin/env bash
# V2 的 cargo 一段：在 <工作目录>/repo（rerun.sh 从仓根拷出来的 crates 副本）里，对四个变体各编、各跑两个测试目标，
# 把 W2PROBE 行归并成落点表写进 <工作目录>/v2-out/。整条经 scripts/hosts/run-with-memory-cap.sh 跑（rerun.sh 外面已包），这里不再套。
# 用法：run_v2.sh <模型目录> <工作目录> <仓根>
set -u
MODEL="$1"; WORK="$2"; REPO_ROOT="$3"
REPO="$WORK/repo"; OUT="$WORK/v2-out"
mkdir -p "$OUT"
cp "$MODEL/v2/v2_extra_entries.rs" "$MODEL/v2/tree_table_entry_with_an_unknown_flags_bit_in_the_newest_version.rs" "$REPO/crates/singlefs-harness/tests/"
for variant in none blacklist whitelist objectlevel; do
  python3 -I "$MODEL/v2/apply_variant.py" --repo "$REPO" --variant "$variant" > "$OUT/apply-$variant.txt" || { echo "name=v2_apply_failed variant=$variant"; exit 2; }
  for target in v2_extra_entries tree_table_entry_with_an_unknown_flags_bit_in_the_newest_version; do
    (cd "$REPO" && SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 $target; 范围=1 个测试目标; 为什么=C693 第二轮 V2 草稿副本（变体 $variant）上量三种盘面三处的落点" \
      nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test "$target" -- --test-threads 1 --nocapture) > "$OUT/$variant-$target.log" 2>&1
    echo "name=v2_cargo variant=$variant target=$target exit=$? test_result=$(grep -h 'test result' "$OUT/$variant-$target.log" | cut -c1-60)"
  done
  python3 -I "$MODEL/v2/summarize.py" "$OUT/$variant-v2_extra_entries.log" "$OUT/$variant-tree_table_entry_with_an_unknown_flags_bit_in_the_newest_version.log" | sed -E "s@^## .*(v2_extra_entries|tree_table_entry_with_an_unknown_flags_bit_in_the_newest_version)\\.log@## \\1@" > "$OUT/summary-$variant.txt"
done
# 白名单变体上，已有的五个 F2 相关测试目标必须全绿（30 条）
python3 -I "$MODEL/v2/apply_variant.py" --repo "$REPO" --variant whitelist > /dev/null
(cd "$REPO" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的五个 F2 相关测试目标; 范围=5 个测试目标; 为什么=C693 第二轮 V2 白名单变体上核已有 F2 用例没被连带改红' \
  nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness \
  --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed \
  --test read_recovery_fields_follow_the_fallback_to_the_witnessed_version \
  --test mount_transparency_judges_the_system_configuration_witness_by_r2 \
  --test writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version \
  --test system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both) > "$OUT/f2-regress-whitelist.log" 2>&1
echo "name=v2_f2_regress exit=$? passed=$(grep -h 'test result' "$OUT/f2-regress-whitelist.log" | sed -E 's/.*ok\. ([0-9]+) passed.*/\1/' | paste -sd+ | bc)"
python3 -I "$MODEL/v2/apply_variant.py" --repo "$REPO" --variant none > /dev/null
