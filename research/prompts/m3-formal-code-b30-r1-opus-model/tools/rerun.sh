#!/usr/bin/env bash
# b30 攻方腿（m3-formal-code-b30-r1-opus）复跑入口。
# 用法：bash tools/rerun.sh <工作目录> <步>…；步名见下面 case，`all` 依次跑全部（合计约 30 分钟，单步都在 4 分钟以内）。
# 环境：FROZEN（冻结副本，默认 /tmp/claude-1000/m3-formal-code-b30-r1/tree）、REPO（主仓，取内存包装与线程脚本，默认 /home/fy5090/code/singlefs）、
#       CAP（线程上限，默认 7）。cargo 一律经 tools/cargo_capped.sh：run-with-memory-cap.sh 8G + capped.sh。只编、跑 singlefs-harness 的测试目标。
# 产物写 <工作目录>/out/，与模型目录 out/ 里同名文件的 B30*、test result、SWEEP 行比。
set -u
here=$(cd "$(dirname "$0")" && pwd)
model=$(cd "$here/.." && pwd)
work=${1:?工作目录}; shift
export FROZEN=${FROZEN:-/tmp/claude-1000/m3-formal-code-b30-r1/tree}
tree="$work/tree"; out="$work/out"
mkdir -p "$out"
tests="$tree/crates/singlefs-harness/tests"
cargo_t() { "$here/cargo_capped.sh" "$tree" test --offline -p singlefs-harness "$@"; }
variant() { "$here/run_variant.sh" "$tree" "$FROZEN" "$out" "$@"; }

step() {
  case $1 in
    prep)
      rm -rf "${tree:?}"; mkdir -p "$tree"
      rsync -a --exclude target "$FROZEN"/ "$tree"/
      cp "$model"/tests/b30_opus_leaf_container_sweep.rs "$model"/tests/b30_opus_glue_leaf_role_is_injective.rs \
         "$model"/tests/b30_opus_e1_i54_legal_histories.rs "$model"/tests/b30_opus_time_threshold_slot_digests.rs "$tests"/
      cp "$tests"/checker_known_bad_images.rs "$tests"/b30_opus_i54_probe.rs
      cat "$model"/tests/b30_opus_i54_append.rs >> "$tests"/b30_opus_i54_probe.rs
      cp "$tests"/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs "$tests"/b30_opus_image_crc_probe.rs
      cat "$model"/tests/b30_opus_image_crc_append.rs >> "$tests"/b30_opus_image_crc_probe.rs
      cp "$tests"/pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs "$tests"/b30_opus_alg_probe.rs
      cat "$model"/tests/b30_opus_alg_append.rs >> "$tests"/b30_opus_alg_probe.rs
      echo "prep done: $tree" ;;
    # M1：入库四个变体与往后挪一片的四个，各跑探针扫描（20 段历史）与 30-1 入库用例
    m1-baseline|m1-kx1_cap_at_third|m1-kx2_cap_at_third|m1-retain_only_second|m1-drop_only_third|m1-glue_cap_at_fourth|m1-model_cap_at_fourth|m1-retain_up_to_third|m1-drop_up_to_fourth)
      variant "${1#m1-}" ;;
    m1-broad-glue)   SKIP_SWEEP=1 TAG=broad variant glue_cap_at_fourth lib model_compares_inode_creations_into_the_third_inode_leaf_container model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds unlink_in_histories_judged_by_the_model ;;
    m1-random-glue)  SKIP_SWEEP=1 TAG=random variant glue_cap_at_fourth random_histories ;;
    m1-lib-shifted)  for v in model_cap_at_fourth retain_up_to_third drop_up_to_fourth; do SKIP_SWEEP=1 TAG=lib variant "$v" lib; done ;;
    m1-broad-retain) SKIP_SWEEP=1 TAG=broad variant retain_up_to_third model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers model_compares_inode_creations_into_the_third_inode_leaf_container unlink_in_histories_judged_by_the_model random_histories ;;
    m1-broad-drop)   SKIP_SWEEP=1 TAG=broad variant drop_up_to_fourth model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers model_compares_inode_creations_into_the_third_inode_leaf_container unlink_in_histories_judged_by_the_model random_histories ;;
    m1-b3)           for v in baseline glue_cap_at_fourth kx1_cap_at_third; do SKIP_SWEEP=1 TAG=b3 variant "$v" b30_opus_glue_leaf_role_is_injective; done ;;
    # M2：合法历史上的 E1 与按版本 I-5.4（两批）、E1 探针臂（数走到几次容器子指针）、逐单元清零、F1、加密压缩几段、F2
    m2-legal-1) B30_SEEDS=3 B30_SEED_BASE=1000 cargo_t --release --test b30_opus_e1_i54_legal_histories -- --nocapture > "$out/e1-i54-legal-batch1.log" 2>&1; python3 "$here/sum_legal.py" "$out/e1-i54-legal-batch1.log" ;;
    m2-legal-2) B30_SEEDS=3 B30_OPS=60 B30_SEED_BASE=5000 cargo_t --release --test b30_opus_e1_i54_legal_histories -- --nocapture > "$out/e1-i54-legal-batch2.log" 2>&1; python3 "$here/sum_legal.py" "$out/e1-i54-legal-batch2.log" ;;
    m2-arm)
      python3 "$here/e1_probe_variant.py" "$tree" apply || return 2
      B30_SEEDS=3 B30_SEED_BASE=1000 cargo_t --release --test b30_opus_e1_i54_legal_histories -- --nocapture > "$out/e1-probe-arm-batch1.log" 2>&1
      python3 "$here/e1_probe_variant.py" "$tree" restore
      python3 "$here/sum_legal.py" "$out/e1-probe-arm-batch1.log"; grep B30E1 "$out/e1-probe-arm-batch1.log" | grep -o 'B30PROBE [^ ]*' | sort | uniq -c ;;
    m2-units) cargo_t --test b30_opus_i54_probe -- --nocapture b30_opus > "$out/i54-units-and-time.log" 2>&1; grep -E 'B30UNIT|B30TIME|test result' "$out/i54-units-and-time.log" ;;
    m2-f1)
      python3 "$here/i54_tree_table_fix.py" "$tree" || return 2
      cargo_t --test b30_opus_i54_probe -- --nocapture b30_opus_i54_after > "$out/i54-units-with-f1.log" 2>&1
      cargo_t --test checker_known_bad_images > "$out/known-bad-with-f1.log" 2>&1
      B30_SEEDS=3 B30_SEED_BASE=1000 cargo_t --release --test b30_opus_e1_i54_legal_histories -- --nocapture > "$out/e1-i54-legal-batch1-with-f1.log" 2>&1
      cp "$FROZEN/crates/singlefs-checker/src/walk.rs" "$tree/crates/singlefs-checker/src/walk.rs"
      grep -E 'slot=50178|slot=50252|slot=50264|test result' "$out/i54-units-with-f1.log"; grep 'test result' "$out/known-bad-with-f1.log"; python3 "$here/sum_legal.py" "$out/e1-i54-legal-batch1-with-f1.log" | tail -1 ;;
    m2-alg) cargo_t --test b30_opus_alg_probe -- --nocapture b30_opus > "$out/alg-probe.log" 2>&1; grep -E 'B30ALG|test result' "$out/alg-probe.log" ;;
    m2-f2)
      python3 "$here/pointer_head_fix.py" "$tree" || return 2
      cargo_t --test b30_opus_alg_probe --test checker_known_bad_images --test checker_judges_reserved_bytes_and_widths_it_used_to_skip -- --nocapture > "$out/alg-probe-with-f2.log" 2>&1
      B30_SEEDS=3 B30_SEED_BASE=1000 cargo_t --release --test b30_opus_e1_i54_legal_histories -- --nocapture > "$out/e1-i54-legal-batch1-with-f2.log" 2>&1
      cp "$FROZEN/crates/singlefs-checker/src/image.rs" "$tree/crates/singlefs-checker/src/image.rs"
      grep -E 'B30ALG|test result' "$out/alg-probe-with-f2.log" | cut -c1-160; python3 "$here/sum_legal.py" "$out/e1-i54-legal-batch1-with-f2.log" | tail -1 ;;
    # M3：两个槽摘要与整盘 CRC 换回秒等于旧钉值；T_time 四字节不动池级 checker 与恢复（在 m2-units 里）
    m3-pins)
      cargo_t --test b30_opus_time_threshold_slot_digests -- --nocapture > "$out/slot-digests.log" 2>&1
      cargo_t --test b30_opus_image_crc_probe -- --nocapture b30_opus > "$out/image-crc.log" 2>&1
      grep -E 'B30PIN|B30CRC|test result' "$out/slot-digests.log" "$out/image-crc.log" ;;
    all)
      for s in prep m1-baseline m1-kx1_cap_at_third m1-kx2_cap_at_third m1-retain_only_second m1-drop_only_third m1-glue_cap_at_fourth \
               m1-model_cap_at_fourth m1-retain_up_to_third m1-drop_up_to_fourth m1-broad-glue m1-random-glue m1-lib-shifted m1-broad-retain \
               m1-broad-drop m1-b3 m2-legal-1 m2-legal-2 m2-arm m2-units m2-f1 m2-alg m2-f2 m3-pins; do
        echo "== $s"; step "$s" || return $?
      done ;;
    *) echo "不认识的步：$1" >&2; return 2 ;;
  esac
}
for s in "$@"; do echo "=== step $s"; step "$s" || exit $?; done
