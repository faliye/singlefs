# 遗留代码轮的改动（2026-10-03）

冻结副本 `/tmp/claude-1000/m3-leftover-code-r1/tree/`（被判 32 份文件的 sha256 在 `research/prompts/m3-leftover-code-r1-snapshot/crates-sha256.txt`）。三块改动来源不同，各自给法不同。

## 一、五条定案的实现（M1）

2026-09-28 那一版按「文件::项名」整段抽的代码在 `research/prompts/_m2-five-decisions-code-r1-diff.md`（3688 行，随提交 dbcc0d2f 进了 HEAD）。之后这些文件被别的批改过，判今天冻结副本里的那几项：`crates/singlefs-core/src/mount.rs` 第 440 行 `SomeSystemConfigurationSlotUnreadOrUnverified` 一带与第 1312 行 `isolate_slots_referenced_only_by_abandoned_roots`；`transaction.rs` 第 681 行 `self_verified_system_configurations_of_every_device`；`recovery.rs` 第 1832 行 `effective_rollback_floor_of_the_roots_read`；`crates/singlefs-checker/src/walk.rs` 第 4616 行 `deferred_slots_referenced_only_below_the_floor_per_device`。

## 二、写零折叠块内乱序（M2）：`device_log.rs` 对 HEAD 的 diff

```diff
diff --git a/crates/singlefs-checker-tier/src/device_log.rs b/crates/singlefs-checker-tier/src/device_log.rs
index 6253a9fa..abe99a23 100644
--- a/crates/singlefs-checker-tier/src/device_log.rs
+++ b/crates/singlefs-checker-tier/src/device_log.rs
@@ -6,7 +6,10 @@
 //! 扇区 0 是系统配置（magic 8、version 8、nr_entries 8、sectorsize 4）；条目从扇区 1 起，每条一个扇区的头
 //! （sector 8、nr_sectors 8、flags 8、data_len 8，补齐到一个扇区），写条目的数据紧跟其后、占 nr_sectors 个扇区。
 
+use std::ops::Range;
+
 use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::block_device::ZERO_FILL_CHUNK_BYTES;
 
 use singlefs_harness::{fnv1a_64, fnv1a_64_of_zeros, RecordedOperationKind, RetainedOperation};
 
@@ -210,14 +213,23 @@ pub fn declared_zero_fills(
 }
 
 /// 把盘上那几段清零折回一件事：程序声明清了 `[offset, offset + length)` 的地方，
-/// 日志里连着的若干个写只要**恰好铺满这一段**（按序、首尾相接、每一条都是全 0），就换成一个整段的写事件。
+/// 日志里连着的若干个写只要**恰好铺满这一段**（每一条都是全 0、互不重叠、不越过段尾；块与块之间按序，
+/// 同一块之内不论次序），就换成一个整段的写事件。
+/// 块的切法与后端写零时相同：从这一段的起点起每 [`ZERO_FILL_CHUNK_BYTES`] 一块，最后一块到段尾为止、可能短
+/// （`singlefs_core::block_device` 的 `write_zeroes_in_chunks`）。
+/// 逐条看时：一条写要么正好接在已铺满的那一截后面（可以跨块，照旧），要么整条落在已铺满那一截的末端所在的那一块里、
+/// 末端之后（同一块里后面的部分先到），等前面接上再并进来。
 ///
 /// 为什么要折：一次 `write_zeroes_at` 在程序那一侧是一个动作，落到盘上却是好几条写——
-/// 后端按 [`singlefs_core::block_device::ZERO_FILL_CHUNK_BYTES`] 拆过一次，来宾内核还会按
+/// 后端按 [`ZERO_FILL_CHUNK_BYTES`] 拆过一次，来宾内核还会按
 /// `max_sectors_kb` 再拆一次，拆成几条是块层的事，程序管不着、也不该由它决定比对过不过。
 ///
-/// 为什么折了还判得动：折的条件是**铺满**——少一块、多一块、顺序反了、中间夹一条不是全 0 的写，
-/// 都折不起来，那一段就原样留着、逐项比对照样判红（「漏写环的最后 1 MiB」正是少一块）。
+/// 为什么块内不论次序、块间仍按序：后端每一块是一次同步写，来宾内核把它拆成几个同时在途的请求，
+/// blklogwrites 按数据盘写完的先后记日志，同一块的几部分谁先谁后由完成次序定，程序管不着；
+/// 下一块要等这一块的同步写返回才发，块与块之间的次序仍是程序定的，倒过来就不折。
+///
+/// 为什么折了还判得动：折的条件是**铺满**——少一块、多一块、块与块之间顺序反了、块内留缝或重叠、
+/// 中间夹一条不是全 0 的写、越过段尾，都折不起来，那一段就原样留着、逐项比对照样判红（「漏写环的最后 1 MiB」正是少一块）。
 /// 折只在程序声明过的那几段里做，别处一个字节都不碰：页缓存那一档的回写合并落在普通写上，判别力不受影响。
 ///
 /// 「这一条是不是全 0」按内容哈希判：`fnv1a_64` 对 `length` 个 0 有唯一取值（[`crate::fnv1a_64_of_zeros`]），
@@ -248,22 +260,50 @@ pub fn fold_declared_zero_fills(
     folded
 }
 
-/// 从 `index` 起的连续几条写，是不是恰好铺满 `declared` 里的某一段；是就交回那一段与用掉的条数。
+/// 从 `index` 起的连续几条写，是不是按 [`fold_declared_zero_fills`] 的规则恰好铺满 `declared` 里的某一段；
+/// 是就交回那一段与用掉的条数。
 fn tile_of_zeros_starting_at(
     observed: &[DeviceEvent],
     index: usize,
     declared: &[(DeviceOffsetInBytes, u64)],
 ) -> Option<(DeviceOffsetInBytes, u64, usize)> {
-    let DeviceEvent::Write { offset: start, .. } = &observed[index] else {
-        return None;
-    };
-    let (declared_offset, declared_length) = declared
+    declared
         .iter()
-        .find(|(declared_offset, _)| declared_offset == start)?;
-    let end = declared_offset.0 + declared_length;
-    let mut cursor = declared_offset.0;
+        .find_map(|(declared_offset, declared_length)| {
+            entries_tiling_one_zero_fill(observed, index, *declared_offset, *declared_length)
+                .map(|consumed| (*declared_offset, *declared_length, consumed))
+        })
+}
+
+/// `position` 落在这一段清零的哪一块里：从段的起点 `fill_start` 起每 [`ZERO_FILL_CHUNK_BYTES`] 一块，
+/// 最后一块到段尾 `fill_end` 为止。与后端写零的拆块循环（`singlefs_core::block_device` 的 `write_zeroes_in_chunks`）同一种切法。
+fn zero_fill_chunk_containing(fill_start: u64, fill_end: u64, position: u64) -> Range<u64> {
+    let chunk_index = (position - fill_start) / ZERO_FILL_CHUNK_BYTES;
+    let chunk_start = fill_start + chunk_index * ZERO_FILL_CHUNK_BYTES;
+    chunk_start..(chunk_start + ZERO_FILL_CHUNK_BYTES).min(fill_end)
+}
+
+/// 从 `index` 起数日志里连着的几条写，看它们是不是恰好铺满 `[declared_offset, declared_offset + declared_length)`；
+/// 是就交回用掉的条数。规则见 [`fold_declared_zero_fills`]。
+fn entries_tiling_one_zero_fill(
+    observed: &[DeviceEvent],
+    index: usize,
+    declared_offset: DeviceOffsetInBytes,
+    declared_length: u64,
+) -> Option<usize> {
+    // 长度 0 的清零在盘上没有条目可折；照下面的循环会交回「用掉 0 条」，`fold_declared_zero_fills` 就原地不动了。
+    if declared_length == 0 {
+        return None;
+    }
+    let fill_start = declared_offset.0;
+    let fill_end = fill_start + declared_length;
+    // `covered_up_to` 之前已经铺满；`arrived_ahead_of_cursor` 是同一块里后面的部分先到的那几条，
+    // 两两不重叠、都在 `covered_up_to` 之后，等前面接上再并进来。
+    let mut covered_up_to = fill_start;
+    let mut arrived_ahead_of_cursor: Vec<Range<u64>> = Vec::new();
     let mut consumed = 0usize;
-    while cursor < end {
+    // 每一轮吃掉日志里的一条，轮数不超过 `observed.len() - index`；`covered_up_to` 只增不减、不越过 `fill_end`。
+    while covered_up_to < fill_end {
         let Some(DeviceEvent::Write {
             offset,
             length,
@@ -272,13 +312,44 @@ fn tile_of_zeros_starting_at(
         else {
             return None;
         };
-        if offset.0 != cursor || *content_hash != fnv1a_64_of_zeros(*length) {
+        if *content_hash != fnv1a_64_of_zeros(*length) {
             return None;
         }
-        cursor += *length;
+        let piece = offset.0..offset.0 + length;
+        let chunk_of_cursor = zero_fill_chunk_containing(fill_start, fill_end, covered_up_to);
+        let continues_at_cursor = piece.start == covered_up_to && piece.end <= fill_end;
+        let lands_ahead_inside_the_chunk_of_cursor =
+            piece.start > covered_up_to && piece.end <= chunk_of_cursor.end;
+        if !continues_at_cursor && !lands_ahead_inside_the_chunk_of_cursor {
+            return None;
+        }
+        let overlaps_an_earlier_arrival = arrived_ahead_of_cursor
+            .iter()
+            .any(|earlier| earlier.start < piece.end && piece.start < earlier.end);
+        if overlaps_an_earlier_arrival {
+            return None;
+        }
+        arrived_ahead_of_cursor.push(piece);
+        while let Some(position) = arrived_ahead_of_cursor
+            .iter()
+            .position(|arrived| arrived.start <= covered_up_to)
+        {
+            let joined = arrived_ahead_of_cursor.swap_remove(position);
+            covered_up_to = covered_up_to.max(joined.end);
+        }
+        assert!(
+            arrived_ahead_of_cursor
+                .iter()
+                .all(|arrived| arrived.start > covered_up_to),
+            "接得上的都并进来了，还没并的只会在已铺满那一截之后"
+        );
         consumed += 1;
     }
-    (cursor == end).then_some((*declared_offset, *declared_length, consumed))
+    assert_eq!(
+        covered_up_to, fill_end,
+        "每一条都不越过段尾（接在末端的查了段尾、先到的查了所在那一块的块尾），已铺满的末端停在段尾"
+    );
+    Some(consumed)
 }
 
 /// 逐项比，报第一处不一致（下标、程序以为的、盘上收到的）。
@@ -592,4 +663,179 @@ mod tests {
         }];
         assert_eq!(fold_declared_zero_fills(&plain, &declared), plain);
     }
+
+    /// 日志里的一条全 0 写。
+    fn zero_write(offset: u64, length: u64) -> DeviceEvent {
+        DeviceEvent::Write {
+            offset: DeviceOffsetInBytes(offset),
+            length,
+            content_hash: fnv1a_64_of_zeros(length),
+        }
+    }
+
+    /// 来宾内核把后端的一块写零拆成两个同时在途的请求时前一个的长度：2026-10-01 虚机档实测拆成 2032 + 6160 扇区。
+    const FRONT_PART_OF_A_GUEST_SPLIT_CHUNK_BYTES: u64 = 2032 * 512;
+
+    /// 一块拆成前后两部分，交回 (前, 后)。
+    fn chunk_split_in_two(chunk_start: u64, chunk_length: u64) -> (DeviceEvent, DeviceEvent) {
+        (
+            zero_write(chunk_start, FRONT_PART_OF_A_GUEST_SPLIT_CHUNK_BYTES),
+            zero_write(
+                chunk_start + FRONT_PART_OF_A_GUEST_SPLIT_CHUNK_BYTES,
+                chunk_length - FRONT_PART_OF_A_GUEST_SPLIT_CHUNK_BYTES,
+            ),
+        )
+    }
+
+    /// blklogwrites 按数据盘写完的先后记日志，同一块的后一部分可能先记：块内不论次序，整段照样折成一件。
+    #[test]
+    fn zero_fill_whose_two_parts_of_one_chunk_arrive_reversed_folds_into_one_event() {
+        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
+        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
+        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
+        let (first_chunk_front, first_chunk_back) =
+            chunk_split_in_two(fill_start, ZERO_FILL_CHUNK_BYTES);
+        let (second_chunk_front, second_chunk_back) =
+            chunk_split_in_two(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES);
+        let observed = vec![
+            first_chunk_back,
+            first_chunk_front,
+            second_chunk_front,
+            second_chunk_back,
+        ];
+        assert_eq!(
+            fold_declared_zero_fills(&observed, &declared),
+            vec![zero_write(fill_start, fill_length)],
+            "第一块两部分倒序、第二块按序：整段折成一件"
+        );
+    }
+
+    /// 第二块整块排在第一块前面：下一块要等这一块的同步写返回才发，块与块之间倒过来不是块层的事，一条都不折。
+    #[test]
+    fn zero_fill_whose_second_chunk_arrives_before_the_first_does_not_fold() {
+        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
+        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
+        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
+        let (first_chunk_front, first_chunk_back) =
+            chunk_split_in_two(fill_start, ZERO_FILL_CHUNK_BYTES);
+        let (second_chunk_front, second_chunk_back) =
+            chunk_split_in_two(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES);
+        let observed = vec![
+            second_chunk_front,
+            second_chunk_back,
+            first_chunk_front,
+            first_chunk_back,
+        ];
+        assert_eq!(
+            fold_declared_zero_fills(&observed, &declared),
+            observed,
+            "第二块排在第一块前面：一条都不折"
+        );
+    }
+
+    /// 第一块还没铺满时第二块的一部分先到（两块的条目交错）：块与块之间仍按序，一条都不折。
+    #[test]
+    fn zero_fill_with_a_piece_of_the_next_chunk_arriving_before_the_current_chunk_is_full_does_not_fold(
+    ) {
+        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
+        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
+        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
+        let (first_chunk_front, first_chunk_back) =
+            chunk_split_in_two(fill_start, ZERO_FILL_CHUNK_BYTES);
+        let (second_chunk_front, second_chunk_back) =
+            chunk_split_in_two(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES);
+        let observed = vec![
+            first_chunk_back,
+            second_chunk_front,
+            first_chunk_front,
+            second_chunk_back,
+        ];
+        assert_eq!(
+            fold_declared_zero_fills(&observed, &declared),
+            observed,
+            "第二块的前一部分插在第一块两部分之间：一条都不折"
+        );
+    }
+
+    /// 第一块里留了一道缝（[1/4, 1/2) 块没写），后一部分先到、第二块照样铺满：铺不满，一条都不折。
+    #[test]
+    fn zero_fill_chunk_with_a_gap_does_not_fold() {
+        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
+        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
+        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
+        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
+        let observed = vec![
+            zero_write(fill_start, quarter_chunk),
+            zero_write(fill_start + 2 * quarter_chunk, 2 * quarter_chunk),
+            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
+        ];
+        assert_eq!(
+            fold_declared_zero_fills(&observed, &declared),
+            observed,
+            "第一块中间缺一截：一条都不折"
+        );
+    }
+
+    /// 第一块里两条重叠（后面的 [1/4, 1) 块先到，[0, 1/2) 块后到），字节加起来比一块多：不是恰好铺满，一条都不折。
+    #[test]
+    fn zero_fill_chunk_whose_pieces_overlap_does_not_fold() {
+        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
+        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
+        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
+        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
+        let observed = vec![
+            zero_write(fill_start + quarter_chunk, 3 * quarter_chunk),
+            zero_write(fill_start, 2 * quarter_chunk),
+            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
+        ];
+        assert_eq!(
+            fold_declared_zero_fills(&observed, &declared),
+            observed,
+            "第一块里两条重叠：一条都不折"
+        );
+    }
+
+    /// 段长 2.5 块、起点不在块大小的整数倍上（mkfs 那几段 32 KiB 的清零起点就不在 4 MiB 边界上）：
+    /// 最后一块短（半块），它的两部分倒序照样折成一件。最后一块是 [6.75, 7.25) 个块大小，跨过设备上第 7 个块大小那道边界：
+    /// 块要是从设备偏移 0 起算而不是从段的起点起算，这两部分就分在两块里、折不起来。
+    #[test]
+    fn zero_fill_whose_short_last_chunk_arrives_reversed_folds_into_one_event() {
+        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
+        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES + 3 * quarter_chunk;
+        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES + 2 * quarter_chunk;
+        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
+        let last_chunk_start = fill_start + 2 * ZERO_FILL_CHUNK_BYTES;
+        let observed = vec![
+            zero_write(fill_start, ZERO_FILL_CHUNK_BYTES),
+            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
+            zero_write(last_chunk_start + quarter_chunk, quarter_chunk),
+            zero_write(last_chunk_start, quarter_chunk),
+        ];
+        assert_eq!(
+            fold_declared_zero_fills(&observed, &declared),
+            vec![zero_write(fill_start, fill_length)],
+            "最后半块的两部分倒序：整段折成一件"
+        );
+    }
+
+    /// 段长 2.5 块：最后一块只到段尾。先到的那一条落在最后一块里、却伸出段尾 1/4 块：越过段尾，一条都不折。
+    #[test]
+    fn zero_fill_whose_short_last_chunk_gets_a_piece_running_past_the_end_does_not_fold() {
+        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
+        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
+        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES + 2 * quarter_chunk;
+        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
+        let last_chunk_start = fill_start + 2 * ZERO_FILL_CHUNK_BYTES;
+        let observed = vec![
+            zero_write(fill_start, ZERO_FILL_CHUNK_BYTES),
+            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
+            zero_write(last_chunk_start + quarter_chunk, 2 * quarter_chunk),
+            zero_write(last_chunk_start, quarter_chunk),
+        ];
+        assert_eq!(
+            fold_declared_zero_fills(&observed, &declared),
+            observed,
+            "最后一块里先到的那一条伸出段尾：一条都不折"
+        );
+    }
 }
```

## 三、对接上游第三轮判决之后的改动（M3）

### 3a 第三轮判决小项（实现员报告 `/tmp/claude-1000/impl-post-r3/report.md`）

```diff
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -1738,9 +1738,9 @@
 崩溃放量 GPU 判器等卡时写库那一头不打按挂钟的行	crates/singlefs-checker-tier/src/crash_amplification.rs	                write_progress_line(gpu_judge_wall_clock_line(	                drop(gpu_judge_wall_clock_line(	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_gpu_judge_reports_a_wall_clock_line_while_it_waits_for_the_cards	the_gpu_judge_reports_a_wall_clock_line_while_it_waits_for_the_cards
 崩溃放量 环境守卫设的值不进环境（库内开关用例走守卫）	crates/singlefs-checker-tier/src/crash_amplification.rs	            "{name} 不在记下原值的那一批里，丢掉时还原不了它"\n        );\n        std::env::set_var(name, value);	            "{name} 不在记下原值的那一批里，丢掉时还原不了它"\n        );\n        let _ = (name, value);	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::environment_switches_are_read_only_when_the_configuration_file_is_absent	environment_switches_are_read_only_when_the_configuration_file_is_absent
 崩溃放量 环境守卫设的值不进环境（库内库目录用例走守卫）	crates/singlefs-checker-tier/src/crash_amplification.rs	            "{name} 不在记下原值的那一批里，丢掉时还原不了它"\n        );\n        std::env::set_var(name, value);	            "{name} 不在记下原值的那一批里，丢掉时还原不了它"\n        );\n        let _ = (name, value);	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::a_library_under_a_directory_a_reboot_erases_is_refused_unless_declared_temporary	a_library_under_a_directory_a_reboot_erases_is_refused_unless_declared_temporary
-崩溃放量 展开上限变量改经 history 读（随机历史脚手架经 crash_amplification 带进登记的崩溃枚举用例的判法代码）	crates/singlefs-checker-tier/src/crash_amplification.rs	            process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),	            singlefs_harness::history::process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),	-p singlefs-checker-tier --lib -- crash_identity::tests::every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries	every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries
+崩溃放量 展开上限变量改经 history 读（随机历史脚手架经 crash_amplification 带进登记的崩溃枚举用例的判法代码）	crates/singlefs-checker-tier/src/crash_amplification.rs	            process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),	            { let _ = singlefs_harness::history::RANDOM_HISTORY_VARIABLE_PREFIX; process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE) }.as_deref(),	-p singlefs-checker-tier --lib -- crash_identity::tests::every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries	every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries
 数值环境变量 故障注入线程数写错悄悄退回本机并行度（不经共用解析）	crates/singlefs-harness/src/fault_injection.rs	        match number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(\n            FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,\n            value.as_deref(),\n        ) {	        match value\n            .as_deref()\n            .and_then(|text| text.trim().parse::<std::num::NonZeroUsize>().ok())\n        {	-p singlefs-harness --lib -- fault_injection::tests::an_unreadable_fault_injection_worker_thread_count_panics_naming_the_variable_and_the_value	an_unreadable_fault_injection_worker_thread_count_panics_naming_the_variable_and_the_value
-数值环境变量 随机历史大档种子数写错悄悄退回默认（不经共用解析）	crates/singlefs-harness/tests/random_histories.rs	    number_from_the_variable_value_panicking_when_unreadable::<u64>(name, value).unwrap_or(default)	    value.and_then(|text| text.trim().parse::<u64>().ok()).unwrap_or(default)	-p singlefs-harness --test random_histories -- _panics_naming_the_variable_and_the_value	an_unreadable_random_history_seed_count_panics_naming_the_variable_and_the_value
+数值环境变量 随机历史大档种子数写错悄悄退回默认（不经共用解析）	crates/singlefs-harness/tests/random_histories.rs	    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(name, value)\n        .map_or(default, std::num::NonZeroU64::get)	    value\n        .and_then(|text| text.trim().parse::<u64>().ok())\n        .unwrap_or(default)	-p singlefs-harness --test random_histories -- _panics_naming_the_variable_and_the_value	an_unreadable_random_history_seed_count_panics_naming_the_variable_and_the_value
 数值环境变量 随机历史线程数写 0 悄悄退回本机并行度（不照共用解析的 NonZeroUsize）	crates/singlefs-harness/tests/random_histories.rs	    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(\n        WORKER_THREADS_ENVIRONMENT_VARIABLE,\n        value,\n    )\n    .unwrap_or_else(	    number_from_the_variable_value_panicking_when_unreadable::<usize>(\n        WORKER_THREADS_ENVIRONMENT_VARIABLE,\n        value,\n    )\n    .and_then(std::num::NonZeroUsize::new)\n    .unwrap_or_else(	-p singlefs-harness --test random_histories -- _panics_naming_the_variable_and_the_value	a_random_history_worker_thread_count_of_zero_panics_naming_the_variable_and_the_value
 崩溃注入 重放行崩溃点数写错悄悄取 0（不经共用解析）	crates/singlefs-checker-tier/src/crash_injection.rs	    number_from_the_variable_value_panicking_when_unreadable::<usize>(variable, Some(text)).expect(	    Some(text.trim().parse::<usize>().unwrap_or(0)).expect(	-p singlefs-checker-tier --lib -- crash_injection::tests::an_unreadable_crash_injection_points_value_panics_naming_the_variable_and_the_value	an_unreadable_crash_injection_points_value_panics_naming_the_variable_and_the_value
 崩溃放量 环境守卫清变量时反倒留一个写坏的值（库内显卡优先级用例走守卫）	crates/singlefs-checker-tier/src/crash_amplification.rs	        for name in names {\n            std::env::remove_var(name);\n        }	        for name in names {\n            std::env::set_var(name, "planted-by-the-mutation");\n        }	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::the_card_priority_is_read_from_the_configuration_file_with_one_layer_of_quotes_removed	the_card_priority_is_read_from_the_configuration_file_with_one_layer_of_quotes_removed
@@ -1751,3 +1751,19 @@
 崩溃注入 线程数改回自己解析、写 0 照收（不照共用解析的 NonZeroUsize）	crates/singlefs-checker-tier/src/crash_injection.rs	        match number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(\n            variable_name,\n            value,\n        ) {\n            Some(count) => Self::FromTheEnvironmentVariable(count.get()),	        match value.map(|text| {\n            text.trim().parse::<usize>().unwrap_or_else(|error| {\n                panic!("{variable_name}={text} 不是十进制正整数：{error}")\n            })\n        }) {\n            Some(count) => Self::FromTheEnvironmentVariable(count),	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_injection_worker_thread_count_of_zero_panics_naming_the_variable_and_the_value	a_crash_injection_worker_thread_count_of_zero_panics_naming_the_variable_and_the_value
 层 0 线程数改回自己解析、另写一句报错（不经共用解析：写错的消息不带「变量=原值 认不出」）	crates/singlefs-checker-tier/src/crash.rs	            match number_from_the_variable_value_panicking_when_unreadable::<NonZeroUsize>(\n                LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE,\n                environment_value,\n            ) {	            match environment_value.map(|text| {\n                text.parse::<NonZeroUsize>().unwrap_or_else(|error| {\n                    panic!("{LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}：{error}")\n                })\n            }) {	-p singlefs-checker-tier --lib -- zero_worker_threads_in_the_environment_variable	zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back
 进程环境变量 共用读法：值不是 UTF-8 悄悄当成没设（调用方取没设时的那一档）	crates/singlefs-harness/src/numeric_environment_variable.rs	        Err(std::env::VarError::NotUnicode(raw)) => {\n            panic!("{variable} 的值不是 UTF-8：{raw:?}；不当成没设")\n        }	        Err(std::env::VarError::NotUnicode(_raw)) => None,	-p singlefs-harness --lib -- numeric_environment_variable::tests::a_variable_whose_value_is_not_unicode_panics_naming_the_variable	a_variable_whose_value_is_not_unicode_panics_naming_the_variable
+数值环境变量 复现入口的 Result 版解析不去前后空白（收缩入口里种子与每段步数写成「 11」「 24」停下，共用解析照收）	crates/singlefs-harness/src/history.rs	    match text.trim().parse::<Number>() {\n        Ok(number) => Ok(Some(number)),	    match text.parse::<Number>() {\n        Ok(number) => Ok(Some(number)),	-p singlefs-harness --lib -- history::tests::blanks_around_the_seed_and_the_step_count_of_one_reproduction_entry_are_dropped_alike	blanks_around_the_seed_and_the_step_count_of_one_reproduction_entry_are_dropped_alike
+数值环境变量 复现入口的 Result 版解析不去前后空白（崩溃注入重放入口里种子起点照收、每段步数停下）	crates/singlefs-harness/src/history.rs	    match text.trim().parse::<Number>() {\n        Ok(number) => Ok(Some(number)),	    match text.parse::<Number>() {\n        Ok(number) => Ok(Some(number)),	-p singlefs-checker-tier --lib -- crash_injection::tests::blanks_around_the_first_seed_and_the_step_count_of_the_replay_entry_are_dropped_alike	blanks_around_the_first_seed_and_the_step_count_of_the_replay_entry_are_dropped_alike
+计数变量 复现入口每段步数改回非负整数读法（写 0 照收，每段 0 步的一批只跑起点）	crates/singlefs-harness/src/history.rs	        let operations_per_history_given = count_from_reproduction_environment(	        let operations_per_history_given = number_from_reproduction_environment(	-p singlefs-harness --lib -- history::tests::a_reproduction_step_count_of_zero_is_refused_with_the_recognized_values	a_reproduction_step_count_of_zero_is_refused_with_the_recognized_values
+计数变量 崩溃注入重放入口每段步数写 0 照收（经 history 的复现入口解析）	crates/singlefs-harness/src/history.rs	        let operations_per_history_given = count_from_reproduction_environment(	        let operations_per_history_given = number_from_reproduction_environment(	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_injection_step_count_of_zero_panics_naming_the_variable_and_the_value	a_crash_injection_step_count_of_zero_panics_naming_the_variable_and_the_value
+计数变量 复现入口的计数读法照收 0（故障注入大档种子数写 0 照收）	crates/singlefs-harness/src/history.rs	        number_given_in_reproduction_environment::<std::num::NonZeroU64>(\n            lookup,\n            variable,\n            COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT,\n        )?\n        .map_or(default, std::num::NonZeroU64::get),	        number_given_in_reproduction_environment::<u64>(\n            lookup,\n            variable,\n            COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT,\n        )?\n        .unwrap_or(default),	-p singlefs-harness --test fault_injection_fast_tier -- a_fault_injection_seed_count_of_zero	a_fault_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value
+计数变量 复现入口的计数读法照收 0（坏盘输入大档种子数写 0 照收）	crates/singlefs-harness/src/history.rs	        number_given_in_reproduction_environment::<std::num::NonZeroU64>(\n            lookup,\n            variable,\n            COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT,\n        )?\n        .map_or(default, std::num::NonZeroU64::get),	        number_given_in_reproduction_environment::<u64>(\n            lookup,\n            variable,\n            COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT,\n        )?\n        .unwrap_or(default),	-p singlefs-checker-tier --test bad_disk_input_campaign -- a_bad_disk_seed_count_of_zero	a_bad_disk_seed_count_of_zero_panics_naming_the_variable_and_the_value
+计数变量 故障注入大档种子数改回非负整数读法（写 0 照收）	crates/singlefs-harness/tests/fault_injection_fast_tier.rs	    let seed_count = count_from_reproduction_environment(	    let seed_count = number_from_reproduction_environment(	-p singlefs-harness --test fault_injection_fast_tier -- a_fault_injection_seed_count_of_zero	a_fault_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value
+计数变量 故障注入大档每段注入次数改回非负整数读法（写 0 照收，一次都不注入）	crates/singlefs-harness/tests/fault_injection_fast_tier.rs	    let faults_given = count_from_reproduction_environment(	    let faults_given = number_from_reproduction_environment(	-p singlefs-harness --test fault_injection_fast_tier -- a_fault_injection_fault_count_of_zero	a_fault_injection_fault_count_of_zero_panics_naming_the_variable_and_the_value
+计数变量 随机历史大档种子数写 0 照收（不照共用解析的 NonZeroU64）	crates/singlefs-harness/tests/random_histories.rs	    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(name, value)\n        .map_or(default, std::num::NonZeroU64::get)	    number_from_the_variable_value_panicking_when_unreadable::<u64>(name, value)\n        .unwrap_or(default)	-p singlefs-harness --test random_histories -- _panics_naming_the_variable_and_the_value	a_random_history_seed_count_of_zero_panics_naming_the_variable_and_the_value
+数值环境变量 随机历史大档种子起点写错悄悄退回默认（不经共用解析）	crates/singlefs-harness/tests/random_histories.rs	    number_from_the_variable_value_panicking_when_unreadable::<u64>(name, value).unwrap_or(default)	    value.and_then(|text| text.trim().parse::<u64>().ok()).unwrap_or(default)	-p singlefs-harness --test random_histories -- _panics_naming_the_variable_and_the_value	an_unreadable_random_history_first_seed_panics_naming_the_variable_and_the_value
+计数变量 崩溃注入重放入口种子数写 0 照收（不照共用解析的 NonZeroU64）	crates/singlefs-checker-tier/src/crash_injection.rs	    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(\n        variable,\n        lookup(variable).as_deref(),\n    )\n    .map(std::num::NonZeroU64::get)	    number_from_the_variable_value_panicking_when_unreadable::<u64>(\n        variable,\n        lookup(variable).as_deref(),\n    )	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value	a_crash_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value
+计数变量 崩溃注入重放入口只抽样时崩溃点数写 0 悄悄当成 1	crates/singlefs-checker-tier/src/crash_injection.rs	            crash_points_per_history: std::num::NonZeroUsize::new(points)	            crash_points_per_history: std::num::NonZeroUsize::new(points.max(1))	-p singlefs-checker-tier --lib -- crash_injection::tests::zero_crash_points_with_sampling_only_panics_naming_the_variable_and_the_value	zero_crash_points_with_sampling_only_panics_naming_the_variable_and_the_value
+计数变量 崩溃注入重放入口短段整段枚举时长段另抽 0 个也改成至少 1（只枚举短段那一种摆法读不回）	crates/singlefs-checker-tier/src/crash_injection.rs	            sampled_in_longer_segments: points,	            sampled_in_longer_segments: points.max(1),	-p singlefs-checker-tier --lib -- crash_injection::tests::zero_crash_points_beside_whole_short_segments_reads_back_as_short_segments_only	zero_crash_points_beside_whole_short_segments_reads_back_as_short_segments_only
+计数变量 崩溃注入重放入口短段写数上限写 0 照收（与 none 同义的第二种写法）	crates/singlefs-checker-tier/src/crash_injection.rs	            segment_writes: positive_count_from_the_variable_text(	            segment_writes: usize_from_the_variable_text(	-p singlefs-checker-tier --lib -- crash_injection::tests::a_short_segment_write_bound_of_zero_panics_naming_the_variable_and_the_value	a_short_segment_write_bound_of_zero_panics_naming_the_variable_and_the_value
+扫到 0 项 崩溃注入只判清单外失败那一组不核判过几个崩溃状态（0 段的一批判成绿）	crates/singlefs-checker-tier/tests/crash_injection_campaign.rs	        CrashInjectionAssertions::NewFindingsOnly => {\n            assert_at_least_one_crash_state_was_judged(report);\n        }	        CrashInjectionAssertions::NewFindingsOnly => {}	-p singlefs-checker-tier --test crash_injection_campaign -- a_batch_that_judged_no_crash_state	a_batch_that_judged_no_crash_state_is_red_under_the_new_findings_only_assertions
+判法闭包 记录核对器 σ 那条流的入口文件经 history 引进程环境变量的读法（随机历史脚手架进 c561 的判法摘要）	crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs	        singlefs_harness::numeric_environment_variable::process_environment_variable(\n            CPU_THREADS_ENVIRONMENT_VARIABLE,\n        )\n        .as_deref(),	        { let _ = singlefs_harness::history::RANDOM_HISTORY_VARIABLE_PREFIX; singlefs_harness::numeric_environment_variable::process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE) }.as_deref(),	-p singlefs-checker-tier --test record_checker_judges_absence_by_the_persisted_set -- this_flow_does_not_reference	this_flow_does_not_reference_the_random_history_scaffolding
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -2459,7 +2459,8 @@
 /// （随机历史、坏盘输入、故障注入的入口也读这几个），认不得时照它报的那一句 panic。消息都带变量名与原值。
 ///
 /// # Panics
-/// 某个变量的取值认不得：数值变量不是装得进的非负整数（短段写数也不是 `none`），或取值是一组名字的变量不在认得的那几个里。
+/// 某个变量的取值认不得：数值变量不是装得进的非负整数（短段写数也不是 `none`），或取值是一组名字的变量不在认得的那几个里；
+/// 「跑几个」的计数写 0（种子数、每段步数、只抽样时每段几个崩溃状态、短段写数上限）也算认不得。
 #[must_use]
 pub fn crash_injection_campaign_from_environment(
     lookup: &dyn Fn(&str) -> Option<String>,
@@ -2470,22 +2471,34 @@
         &CrashInjectionVariable::FirstSeed.variable_name(),
     )
     .unwrap_or(defaults.first_seed);
-    let seed_count =
-        u64_from_reproduction_environment(lookup, &CrashInjectionVariable::Seeds.variable_name())
-            .unwrap_or(defaults.seed_count);
+    let seed_count = count_from_reproduction_environment_panicking_when_unreadable(
+        lookup,
+        &CrashInjectionVariable::Seeds.variable_name(),
+    )
+    .unwrap_or(defaults.seed_count);
     let (default_points, default_short_segment_writes) = draw_environment_values(defaults.draw);
     let points_variable = CrashInjectionVariable::Points.variable_name();
-    let points = usize_from_reproduction_environment(lookup, &points_variable, &default_points);
+    let points_text = lookup(&points_variable).unwrap_or(default_points);
+    let points = usize_from_the_variable_text(&points_variable, &points_text);
     let short_segment_writes_variable = CrashInjectionVariable::ShortSegmentWrites.variable_name();
     let short_segment_writes =
         lookup(&short_segment_writes_variable).unwrap_or(default_short_segment_writes);
+    // 只抽样时崩溃点数是每段摆几个，写 0 就一个崩溃状态都不摆；短段整段枚举时它是长段上另抽几个，0 是「只枚举短段」。
+    // 短段写数是写数上限，写 0 没有一段够得上，与 none 同义却让重放行多一种写法，按认不出停下。
     let draw = if short_segment_writes == SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE {
         CrashPointDraw::Sampled {
-            crash_points_per_history: points,
+            crash_points_per_history: std::num::NonZeroUsize::new(points)
+                .unwrap_or_else(|| {
+                    panic!(
+                        "{points_variable}={points_text:?} 认不出：只抽样（短段写数是 {SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE}）时每段摆几个崩溃状态要写正整数，\
+                         0 个就是这一批一个崩溃状态都不摆"
+                    )
+                })
+                .get(),
         }
     } else {
         CrashPointDraw::EveryProperSubsetOfShortSegments {
-            segment_writes: usize_from_the_variable_text(
+            segment_writes: positive_count_from_the_variable_text(
                 &short_segment_writes_variable,
                 &short_segment_writes,
             ),
@@ -2523,7 +2536,7 @@
     }
 }
 
-/// 读一个 u64 的非负整数变量（种子起点、种子数）：没设是 `None`，由调用方取默认。解析经数值环境变量共用的那一份。
+/// 读一个 u64 的非负整数变量（种子起点）：没设是 `None`，由调用方取默认。解析经数值环境变量共用的那一份。
 ///
 /// # Panics
 /// 设了而不是装得进 u64 的非负整数：消息带变量名与原值。
@@ -2537,17 +2550,20 @@
     )
 }
 
-/// 读一个装得进 usize 的非负整数变量；没设时取 `default_text`（照同一个解析读，默认值写错一样报出来）。
+/// 读一个「跑几个」的计数变量（种子数）：没设是 `None`，由调用方取默认。解析经数值环境变量共用的那一份，取 `NonZeroU64`：
+/// 写 0 就是这一批一段都不跑，照样跑完、按只判清单外失败的那一组判成绿（上游 `show-me-test.md`「扫到 0 项也不是通过」），与写错一样停下。
 ///
 /// # Panics
-/// 变量的值（没设时是 `default_text`）不是装得进 usize 的非负整数。
-fn usize_from_reproduction_environment(
+/// 设了而不是装得进 u64 的正整数：消息带变量名与原值。
+fn count_from_reproduction_environment_panicking_when_unreadable(
     lookup: &dyn Fn(&str) -> Option<String>,
     variable: &str,
-    default_text: &str,
-) -> usize {
-    let text = lookup(variable).unwrap_or_else(|| default_text.to_string());
-    usize_from_the_variable_text(variable, &text)
+) -> Option<u64> {
+    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(
+        variable,
+        lookup(variable).as_deref(),
+    )
+    .map(std::num::NonZeroU64::get)
 }
 
 /// 一个装得进 usize 的非负整数变量的原文换成数：解析经数值环境变量共用的那一份，写错就 panic、消息带变量名与原值。
@@ -2560,6 +2576,19 @@
     )
 }
 
+/// 一个计数变量（短段的写数上限）的原文换成数：同 [`usize_from_the_variable_text`]，取 `NonZeroUsize`，写 0 同样停下。
+///
+/// # Panics
+/// `text` 不是装得进 usize 的正整数。
+fn positive_count_from_the_variable_text(variable: &str, text: &str) -> usize {
+    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
+        variable,
+        Some(text),
+    )
+    .expect("交进去的值是 Some（没设时调用方已取了默认原文），共用解析只在值是 None 时交回 None")
+    .get()
+}
+
 /// 一次崩溃注入跑什么：种子区间、每段几步、怎么摆崩溃状态、比重、历史怎么跑、线程数、判红时镜像留不留，
 /// 以及跑完照哪一组断言判（跑的时候不读它，只交给报告的重放行）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -2969,8 +2998,21 @@
         variable: &'static str,
         value: &'static str,
     ) -> CrashInjectionCampaign {
+        campaign_read_back_with_these_variables_set(&[(variable, value)])
+    }
+
+    /// 重放入口只设了 `assignments` 里这几个变量（变量名、取值）时读回的那一批；其余取一组写死的默认
+    /// （种子起点 1、2 段、每段 3 步、只抽样每段 4 个）。
+    fn campaign_read_back_with_these_variables_set(
+        assignments: &[(&str, &str)],
+    ) -> CrashInjectionCampaign {
         crash_injection_campaign_from_environment(
-            &|asked: &str| (asked == variable).then(|| value.to_string()),
+            &|asked: &str| {
+                assignments
+                    .iter()
+                    .find(|(variable, _value)| *variable == asked)
+                    .map(|(_variable, value)| (*value).to_string())
+            },
             &CrashInjectionCampaign {
                 first_seed: 1,
                 seed_count: 2,
@@ -3052,6 +3094,74 @@
         );
     }
 
+    /// 同一个重放入口里种子起点（经数值环境变量共用的那一份解析）与每段步数（经 `history` 的复现入口解析）前后都带空白：
+    /// 两个结局相同，都去掉空白读回（原先一个照收、一个停下）。
+    #[test]
+    fn blanks_around_the_first_seed_and_the_step_count_of_the_replay_entry_are_dropped_alike() {
+        let read_back = campaign_read_back_with_these_variables_set(&[
+            ("SINGLEFS_CRASH_INJECTION_FIRST_SEED", " 11"),
+            ("SINGLEFS_CRASH_INJECTION_OPERATIONS", " 24"),
+        ]);
+        assert_eq!(
+            (read_back.first_seed, read_back.operations_per_history),
+            (11, 24),
+            "种子起点与每段步数前后的空白都去掉读回"
+        );
+    }
+
+    /// 种子数写 0：重放入口停下，消息带变量名与原值，不照收（一段都不跑的一批照只判清单外失败的那一组判成绿）。
+    #[test]
+    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SEEDS=\"0\" 认不出")]
+    fn a_crash_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
+        let _campaign =
+            campaign_read_back_with_only_this_variable_set("SINGLEFS_CRASH_INJECTION_SEEDS", "0");
+    }
+
+    /// 每段步数写 0：重放入口照 `history` 的复现入口报的那一句停下，消息带变量名与原值。
+    #[test]
+    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_OPERATIONS=0：只认 正整数")]
+    fn a_crash_injection_step_count_of_zero_panics_naming_the_variable_and_the_value() {
+        let _campaign = campaign_read_back_with_only_this_variable_set(
+            "SINGLEFS_CRASH_INJECTION_OPERATIONS",
+            "0",
+        );
+    }
+
+    /// 只抽样时每段崩溃点数写 0：重放入口停下，消息带变量名与原值（每段一个崩溃状态都不摆）。
+    #[test]
+    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_POINTS=\"0\" 认不出")]
+    fn zero_crash_points_with_sampling_only_panics_naming_the_variable_and_the_value() {
+        let _campaign =
+            campaign_read_back_with_only_this_variable_set("SINGLEFS_CRASH_INJECTION_POINTS", "0");
+    }
+
+    /// 短段整段枚举时长段上另抽 0 个：照收，是「只枚举短段」那一种摆法（0 只在只抽样时停下）。
+    #[test]
+    fn zero_crash_points_beside_whole_short_segments_reads_back_as_short_segments_only() {
+        assert_eq!(
+            campaign_read_back_with_these_variables_set(&[
+                ("SINGLEFS_CRASH_INJECTION_POINTS", "0"),
+                ("SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES", "3"),
+            ])
+            .draw,
+            CrashPointDraw::EveryProperSubsetOfShortSegments {
+                segment_writes: 3,
+                sampled_in_longer_segments: 0,
+            },
+            "长段上另抽 0 个、写数不超过 3 的段整段枚举"
+        );
+    }
+
+    /// 短段写数上限写 0：重放入口停下，消息带变量名与原值（没有一段够得上，与 `none` 同义却多一种写法）。
+    #[test]
+    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES=\"0\" 认不出")]
+    fn a_short_segment_write_bound_of_zero_panics_naming_the_variable_and_the_value() {
+        let _campaign = campaign_read_back_with_only_this_variable_set(
+            "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES",
+            "0",
+        );
+    }
+
     /// 线程数从环境变量取，没设取 `available_parallelism`；设了不是正整数就停，不悄悄退回单线程。
     #[test]
     fn worker_thread_count_comes_from_the_environment_variable_or_available_parallelism() {
--- a/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs
+++ b/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs
@@ -23,11 +23,11 @@
     known_panic_site_among, known_panic_site_of,
     newest_root_has_the_trees_the_targeted_damages_need,
     release_check_of_the_instance_table_placement_released_on_the_second_device_only,
-    run_bad_disk_campaign, BadDiskCampaign, BadDiskFinding, BadDiskInputWorkerThreads,
-    BadDiskObservation, BadDiskReport, BadDiskTally, DamageKind, DamageRefusedAtTheHeaderJudgement,
-    DamagedImage, KnownPanicSite, ReadBackVerdict, ReaderOutcome,
-    BAD_DISK_INPUT_WORKER_THREADS_ENVIRONMENT_VARIABLE, EVERY_BASE_IMAGE_TIER, EVERY_DAMAGE_KIND,
-    KNOWN_PANIC_SITES,
+    run_bad_disk_campaign, BadDiskCampaign, BadDiskFinding, BadDiskInputVariable,
+    BadDiskInputWorkerThreads, BadDiskObservation, BadDiskReport, BadDiskTally, DamageKind,
+    DamageRefusedAtTheHeaderJudgement, DamagedImage, KnownPanicSite, ReadBackVerdict,
+    ReaderOutcome, BAD_DISK_INPUT_WORKER_THREADS_ENVIRONMENT_VARIABLE, EVERY_BASE_IMAGE_TIER,
+    EVERY_DAMAGE_KIND, KNOWN_PANIC_SITES,
 };
 use singlefs_core::address::DeviceIdentity;
 use singlefs_core::admission::SpaceAdmission;
@@ -36,12 +36,13 @@
 use singlefs_core::transaction::TransactionUnit;
 use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
 use singlefs_harness::history::{
-    environment_assignments_in_line, execute_history_with, generate_history_with_weights,
-    process_environment_variable, CapturedPanic, GeneratedHistory, GenerationWeights,
+    count_from_reproduction_environment, environment_assignments_in_line, execute_history_with,
+    generate_history_with_weights, CapturedPanic, GeneratedHistory, GenerationWeights,
     HistoryDeviceWidth, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
     PerStepChecker, SeededRandomSource,
 };
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::numeric_environment_variable::process_environment_variable;
 use singlefs_harness::SharedStream;
 
 /// 快档的规模：段数与每段步数。规模与种子基都写死（种子基是这个测试周期的常量）。
@@ -1027,7 +1028,18 @@
 /// 大档那条入口读环境：变量与判红时打的重放行同一张表（`bad_disk_campaign_from_environment`），没设的取大档的默认
 /// （种子基起 500 段、每段 16 步、各类操作都抽、不跑每一步的 checker、两块 4 GiB 的盘、判空间准入），
 /// 线程数从 `SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS` 取。入口与「重放行喂回入口」那两条自证都经这一个函数。
+/// 种子数是计数：先照 `history` 的计数读法核一遍（写 0 与写错一样停下，0 段的一批照「零 panic、不读回没提交过的版本」判成绿），
+/// 读回的种子数仍由 `bad_disk_campaign_from_environment` 给。
+///
+/// # Panics
+/// 某个变量的取值认不得（种子数写 0 也算）：消息带变量名与原值。
 fn large_tier_campaign_from(lookup: &dyn Fn(&str) -> Option<String>) -> BadDiskCampaign {
+    let _seed_count_checked_to_be_positive = count_from_reproduction_environment(
+        lookup,
+        &BadDiskInputVariable::Seeds.variable_name(),
+        1,
+    )
+    .unwrap_or_else(|error| panic!("{}", error.render()));
     bad_disk_campaign_from_environment(
         lookup,
         &BadDiskCampaign {
@@ -1044,6 +1056,15 @@
     .unwrap_or_else(|error| panic!("{}", error.render()))
 }
 
+/// 大档入口里种子数写 0：停下，消息带变量名与原值，不照收（一段都不跑的一批照「零 panic、不读回没提交过的版本」判成绿）。
+#[test]
+#[should_panic(expected = "SINGLEFS_BAD_DISK_SEEDS=0：只认 正整数")]
+fn a_bad_disk_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
+    let _campaign = large_tier_campaign_from(&|variable: &str| {
+        (variable == "SINGLEFS_BAD_DISK_SEEDS").then(|| "0".to_string())
+    });
+}
+
 /// 大档：规模与取样参数照判红时打的重放行从环境变量取，后台跑。
 #[test]
 #[ignore = "大档：照判红时打的重放行设环境变量；SINGLEFS_BAD_DISK_FIRST_SEED 起 SINGLEFS_BAD_DISK_SEEDS 段、每段 SINGLEFS_BAD_DISK_OPERATIONS 步，比重 SINGLEFS_BAD_DISK_WEIGHTS，checker SINGLEFS_BAD_DISK_CHECKER，盘宽 SINGLEFS_BAD_DISK_DEVICES，空间准入 SINGLEFS_BAD_DISK_SPACE_ADMISSION，线程数 SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS；没设的取大档默认（种子基、500 段、16 步、broad、skip、4gib、judged）；不进每次的 cargo test"]
--- a/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs
+++ b/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs
@@ -1,5 +1,5 @@
 //! 崩溃放量共用的流程步骤。几个测试文件调这里的同一个函数，就得到同一个身份、同一串路径段：前缀节点在库里只核一次，
-//! 别的流直接复用（用户 2026-09-28 定：前缀共享是剪枝的基础，没有它每次放量都从头核）。
+//! 别的流直接复用（前缀共享是剪枝的基础，没有它每次放量都从头核）。
 //! 崩溃点的文件是这一份（`file!()`），不是调它的测试文件：改调用方的测试文件不动这里的节点。判法摘要与设点的文件无关，
 //! 只随两个判器的闭包变（`crash_identity::JUDGE_ENTRY_FILES`）。
 #![allow(
--- a/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
@@ -184,7 +184,9 @@
         report.render()
     );
     match report.assertions {
-        CrashInjectionAssertions::NewFindingsOnly => {}
+        CrashInjectionAssertions::NewFindingsOnly => {
+            assert_at_least_one_crash_state_was_judged(report);
+        }
         CrashInjectionAssertions::SmallFastTier => {
             assert_the_small_fast_tier_ran_each_stage_on_every_crash_state(report);
         }
@@ -197,6 +199,36 @@
     }
 }
 
+/// 只判清单外失败的那一组也要判过至少一个崩溃状态：一个都没判过的一批没有清单外失败，只是因为什么都没判
+/// （上游 `show-me-test.md`「扫到 0 项也不是通过」）。别的三组各自要的计数都不少于段数，已经含着这一条。
+fn assert_at_least_one_crash_state_was_judged(report: &CrashInjectionReport) {
+    assert!(
+        report.tally.crash_points > 0,
+        "这一批一个崩溃状态都没判过，扫到 0 项不算通过；{}",
+        how_to_replay(report)
+    );
+}
+
+/// 一段都不跑的一批照只判清单外失败的那一组判：判红，不因为没有清单外失败就判成绿。
+#[test]
+#[should_panic(expected = "这一批一个崩溃状态都没判过")]
+fn a_batch_that_judged_no_crash_state_is_red_under_the_new_findings_only_assertions() {
+    let report = run_crash_injection_campaign(&CrashInjectionCampaign {
+        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
+        seed_count: 0,
+        operations_per_history: 8,
+        draw: CrashPointDraw::Sampled {
+            crash_points_per_history: 2,
+        },
+        weights: GenerationWeights::BROAD,
+        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
+        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
+        assertions: CrashInjectionAssertions::NewFindingsOnly,
+    });
+    judge_crash_injection_report(&report);
+}
+
 /// 小快档那一组的结构计数：三截每一截在每个崩溃状态上都真的跑了，二次崩溃摆出来过、每个都判过，恢复没失败过。
 fn assert_the_small_fast_tier_ran_each_stage_on_every_crash_state(report: &CrashInjectionReport) {
     let replay = how_to_replay(report);
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -45,12 +45,13 @@
 use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::mount::RollbackTarget;
-use singlefs_harness::history::process_environment_variable;
 use singlefs_harness::memory_pool::{
     writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
     WrittenContents,
 };
-use singlefs_harness::numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable;
+use singlefs_harness::numeric_environment_variable::{
+    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
+};
 use singlefs_harness::segments::StepKind;
 
 const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO";
--- a/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs
+++ b/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs
@@ -63,6 +63,25 @@
     .unwrap_or_else(|| std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::MIN))
 }
 
+/// 这条流的入口文件不引随机历史脚手架（`singlefs_harness` 的 `history` 模块）：登记的崩溃枚举用例按模块引用收判法代码，
+/// 入口文件一引它，`history.rs` 连同它带进来的模型、对拍、场景、故障注入就都进了这一格的判法摘要，改随机历史就让这一格整格重判。
+/// 进程环境变量的读法经 `numeric_environment_variable` 直接引（`history` 不转出它）。
+/// 读源文件判、不调 `crash_identity` 的闭包函数：引 `crash_identity` 本身就会把它收进这一格的判法代码。
+#[test]
+fn this_flow_does_not_reference_the_random_history_scaffolding() {
+    let source_path = concat!(
+        env!("CARGO_MANIFEST_DIR"),
+        "/tests/record_checker_judges_absence_by_the_persisted_set.rs"
+    );
+    let source = std::fs::read_to_string(source_path)
+        .unwrap_or_else(|error| panic!("读不了这条流自己的源文件 {source_path}：{error}"));
+    let random_history_module = ["singlefs_harness", "history"].join("::");
+    assert!(
+        !source.contains(&random_history_module),
+        "这条流的入口文件引了 {random_history_module}：随机历史脚手架会进这一格的判法摘要"
+    );
+}
+
 #[cfg(feature = "gpu")]
 #[test]
 #[should_panic(expected = "SINGLEFS_CRASH_JUDGE_CPU_THREADS=\"sixteen\" 认不出")]
@@ -1114,8 +1133,10 @@
     // CPU 判器按区间切片并行判（每个状态十几到几十毫秒，单线程判 6.5 万态要一个多小时）：线程数从
     // SINGLEFS_CRASH_JUDGE_CPU_THREADS 取，没设取本机可用并行度；每线程 8 片，按片序拼回，与单线程逐状态相同
     let threads = cpu_threads_from_the_value_of_the_variable(
-        singlefs_harness::history::process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE)
-            .as_deref(),
+        singlefs_harness::numeric_environment_variable::process_environment_variable(
+            CPU_THREADS_ENVIRONMENT_VARIABLE,
+        )
+        .as_deref(),
     )
     .get();
     let slice_count = u64::try_from(threads * 8).expect("装得进 u64");
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -4080,7 +4080,9 @@
     }
 }
 
-/// 读一个非负整数变量：没设取 `default`。
+/// 读一个非负整数变量：没设取 `default`。前后空白先去掉，与数值环境变量共用的那一份解析
+/// （`numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable`）同一口径：
+/// 同一个复现入口里有的数经它、有的数经共用解析，`" 11"` 不能这边照收、那边停下。
 ///
 /// # Errors
 /// 设了但不是非负整数。
@@ -4089,16 +4091,53 @@
     variable: &str,
     default: u64,
 ) -> Result<u64, ReproductionEnvironmentError> {
-    match lookup(variable) {
-        None => Ok(default),
-        Some(text) => {
-            text.parse()
-                .map_err(|_not_a_number| ReproductionEnvironmentError::Unrecognized {
-                    variable: variable.to_string(),
-                    value: text.clone(),
-                    recognized: vec!["非负整数".to_string()],
-                })
-        }
+    Ok(
+        number_given_in_reproduction_environment::<u64>(lookup, variable, "非负整数")?
+            .unwrap_or(default),
+    )
+}
+
+/// 读一个「跑几个」的计数变量（种子数、每段步数、每段注入次数这一类）：没设取 `default`；写 0 与写错一样认不得。
+/// 0 个就是这一批一项都没跑，照样跑完、照样退 0，按只判清单外失败的那一组判成绿（上游 `show-me-test.md`「扫到 0 项也不是通过」）。
+/// 前后空白照 [`number_from_reproduction_environment`] 先去掉。
+///
+/// # Errors
+/// 设了但不是正整数。
+pub fn count_from_reproduction_environment(
+    lookup: &dyn Fn(&str) -> Option<String>,
+    variable: &str,
+    default: u64,
+) -> Result<u64, ReproductionEnvironmentError> {
+    Ok(
+        number_given_in_reproduction_environment::<std::num::NonZeroU64>(
+            lookup,
+            variable,
+            COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT,
+        )?
+        .map_or(default, std::num::NonZeroU64::get),
+    )
+}
+
+/// [`count_from_reproduction_environment`] 认不得时报的「认得的取值」。
+pub const COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT: &str = "正整数（0 个就是一项都没跑）";
+
+/// 复现入口里一个数值变量的取值换成 `Number`：没设是 `None`；前后空白先去掉；认不出交回带原值的
+/// [`ReproductionEnvironmentError::Unrecognized`]，认得的取值写成 `recognized`。
+fn number_given_in_reproduction_environment<Number: std::str::FromStr>(
+    lookup: &dyn Fn(&str) -> Option<String>,
+    variable: &str,
+    recognized: &str,
+) -> Result<Option<Number>, ReproductionEnvironmentError> {
+    let Some(text) = lookup(variable) else {
+        return Ok(None);
+    };
+    match text.trim().parse::<Number>() {
+        Ok(number) => Ok(Some(number)),
+        Err(_not_a_number) => Err(ReproductionEnvironmentError::Unrecognized {
+            variable: variable.to_string(),
+            value: text,
+            recognized: vec![recognized.to_string()],
+        }),
     }
 }
 
@@ -4195,9 +4234,10 @@
     }
 
     /// 从环境读回（`lookup` 交回变量的取值，没设是 None）：没设的那一项取 `defaults` 的。
+    /// 每段步数是计数（[`count_from_reproduction_environment`]）：每段 0 步的一批只跑了起点，写 0 与写错一样认不得。
     ///
     /// # Errors
-    /// 某个变量的取值认不得。
+    /// 某个变量的取值认不得（每段步数写 0 也算）。
     pub fn from_environment(
         prefix: ReproductionVariablePrefix,
         lookup: &dyn Fn(&str) -> Option<String>,
@@ -4205,7 +4245,7 @@
     ) -> Result<HistorySampling, ReproductionEnvironmentError> {
         let operations_variable =
             prefix.variable_named(HistorySamplingVariable::Operations.suffix());
-        let operations_per_history_given = number_from_reproduction_environment(
+        let operations_per_history_given = count_from_reproduction_environment(
             lookup,
             &operations_variable,
             u64::try_from(defaults.operations_per_history).expect("步数装得进 u64"),
@@ -4285,10 +4325,6 @@
         .collect()
 }
 
-/// 复现入口读进程环境经共用的那一份（定义在 `numeric_environment_variable`）；这里转出去，经 `history` 引它的测试文件不用改。
-/// checker 档的库代码直接引 `numeric_environment_variable` 的，不经这里（`crash_identity` 那条依赖判法：随机历史脚手架只许由测试文件自己直接引进来）。
-pub use crate::numeric_environment_variable::process_environment_variable;
-
 /// 随机历史判红时打的那一行复现命令：种子与这一段的全部取样参数，交「收缩一个种子」那条入口
 /// （`random_histories` 的 `shrink_one_failing_seed_from_the_environment`）照抄就是原来那一段历史、原来那种跑法。
 #[must_use]
@@ -5462,6 +5498,66 @@
             })
         );
     }
+
+    /// 「收缩一个种子」那条入口里种子与每段步数前后都带空白：两个都去掉空白读回，与数值环境变量共用的那一份解析同一口径
+    /// （同一个入口里一个数照收、另一个数停下，复现行手抄时多一个空格就分不清是哪个变量写错了）。
+    #[test]
+    fn blanks_around_the_seed_and_the_step_count_of_one_reproduction_entry_are_dropped_alike() {
+        let defaults = HistorySampling {
+            operations_per_history: 30,
+            weights: GenerationWeights::BROAD,
+            execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        };
+        let assignments = BTreeMap::from([
+            (
+                "SINGLEFS_RANDOM_HISTORY_SHRINK_SEED".to_string(),
+                " 11".to_string(),
+            ),
+            (
+                "SINGLEFS_RANDOM_HISTORY_OPERATIONS".to_string(),
+                " 24".to_string(),
+            ),
+        ]);
+        assert_eq!(
+            random_history_to_reproduce_from_environment(
+                &|variable: &str| assignments.get(variable).cloned(),
+                &defaults,
+            ),
+            Ok((
+                HistorySeed(11),
+                HistorySampling {
+                    operations_per_history: 24,
+                    weights: GenerationWeights::BROAD,
+                    execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+                }
+            )),
+            "种子与每段步数前后的空白都去掉读回"
+        );
+    }
+
+    /// 每段步数写 0：复现入口报出变量名、给的值与认得的取值，不照收（每段 0 步的一批只跑了起点，照样判成绿）。
+    #[test]
+    fn a_reproduction_step_count_of_zero_is_refused_with_the_recognized_values() {
+        let defaults = HistorySampling {
+            operations_per_history: 30,
+            weights: GenerationWeights::BROAD,
+            execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        };
+        assert_eq!(
+            HistorySampling::from_environment(
+                RANDOM_HISTORY_VARIABLE_PREFIX,
+                &|variable: &str| {
+                    (variable == "SINGLEFS_RANDOM_HISTORY_OPERATIONS").then(|| "0".to_string())
+                },
+                &defaults,
+            ),
+            Err(ReproductionEnvironmentError::Unrecognized {
+                variable: "SINGLEFS_RANDOM_HISTORY_OPERATIONS".to_string(),
+                value: "0".to_string(),
+                recognized: vec![COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT.to_string()],
+            })
+        );
+    }
 
     /// 同一个种子、同一个步数生成的历史逐项相同；换一个种子就不同。
     #[test]
--- a/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs
+++ b/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs
@@ -72,9 +72,11 @@
     FaultSchedule, FlippedBit, InjectedFault, NamedRootRingSlots, RootRingSlotTarget,
     SharedFaultPlan,
 };
-use singlefs_harness::history::{process_environment_variable, HistoryDeviceWidth};
+use singlefs_harness::history::HistoryDeviceWidth;
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
-use singlefs_harness::numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable;
+use singlefs_harness::numeric_environment_variable::{
+    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
+};
 use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};
 
 const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;
--- a/crates/singlefs-harness/tests/fault_injection_fast_tier.rs
+++ b/crates/singlefs-harness/tests/fault_injection_fast_tier.rs
@@ -31,16 +31,18 @@
 };
 use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
 use singlefs_harness::history::{
-    environment_assignments_in_line, execute_history_with, execute_history_with_faults,
-    generate_history_with_weights, number_from_reproduction_environment,
-    process_environment_variable, render_environment_assignments, ContentChoice, ContentLength,
-    FailureSignature, FloorTargetChoice, GeneratedHistory, GenerationWeights, HarnessJudgement,
-    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
-    HistoryRun, HistorySampling, HistorySeed, HistoryStartingPoint, PerStepChecker,
-    ReproductionEnvironmentError, ReproductionVariablePrefix, StartingPointStep, StepPosition,
+    count_from_reproduction_environment, environment_assignments_in_line, execute_history_with,
+    execute_history_with_faults, generate_history_with_weights,
+    number_from_reproduction_environment, render_environment_assignments, ContentChoice,
+    ContentLength, FailureSignature, FloorTargetChoice, GeneratedHistory, GenerationWeights,
+    HarnessJudgement, HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation,
+    HistoryOperationKind, HistoryRun, HistorySampling, HistorySeed, HistoryStartingPoint,
+    PerStepChecker, ReproductionEnvironmentError, ReproductionVariablePrefix, StartingPointStep,
+    StepPosition,
 };
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
 use singlefs_harness::model::ModelDisagreementAspect;
+use singlefs_harness::numeric_environment_variable::process_environment_variable;
 use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
 use singlefs_harness::segments::{FixedGeometry, StepKind};
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
@@ -126,6 +128,7 @@
 }
 
 /// 大档那条入口读环境：读的变量与 [`fault_injection_replay_line`] 写出的逐个相同，没设的取 `defaults` 的；线程数不在重放行里，照 `defaults` 的。
+/// 种子数与每段注入次数是计数：写 0 与写错一样认不得（0 段、或每段一次都不注入的一批照样判成绿）。
 fn fault_injection_campaign_from_environment(
     lookup: &dyn Fn(&str) -> Option<String>,
     defaults: &FaultInjectionCampaign,
@@ -135,13 +138,13 @@
         &FaultInjectionVariable::FirstSeed.variable_name(),
         defaults.first_seed,
     )?;
-    let seed_count = number_from_reproduction_environment(
+    let seed_count = count_from_reproduction_environment(
         lookup,
         &FaultInjectionVariable::Seeds.variable_name(),
         defaults.seed_count,
     )?;
     let faults_variable = FaultInjectionVariable::Faults.variable_name();
-    let faults_given = number_from_reproduction_environment(
+    let faults_given = count_from_reproduction_environment(
         lookup,
         &faults_variable,
         u64::try_from(defaults.faults_per_history).expect("注入次数装得进 u64"),
@@ -337,6 +340,24 @@
     .unwrap_or_else(|error| panic!("{}", error.render()))
 }
 
+/// 大档入口里种子数写 0：停下，消息带变量名与原值，不照收（一段都不跑的一批照「全部返回错误、不 panic」判成绿）。
+#[test]
+#[should_panic(expected = "SINGLEFS_FAULT_INJECTION_SEEDS=0：只认 正整数")]
+fn a_fault_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
+    let _campaign = large_tier_campaign_from(&|variable: &str| {
+        (variable == "SINGLEFS_FAULT_INJECTION_SEEDS").then(|| "0".to_string())
+    });
+}
+
+/// 大档入口里每段注入次数写 0：停下，消息带变量名与原值，不照收（一次都不注入的一批照样判成绿）。
+#[test]
+#[should_panic(expected = "SINGLEFS_FAULT_INJECTION_FAULTS=0：只认 正整数")]
+fn a_fault_injection_fault_count_of_zero_panics_naming_the_variable_and_the_value() {
+    let _campaign = large_tier_campaign_from(&|variable: &str| {
+        (variable == "SINGLEFS_FAULT_INJECTION_FAULTS").then(|| "0".to_string())
+    });
+}
+
 /// 大档：规模与取样参数照判红时打的重放行从环境变量取，后台跑（`cargo test --release -p singlefs-harness --test
 /// fault_injection_fast_tier -- --ignored --nocapture`）。
 #[test]
--- a/crates/singlefs-harness/tests/random_histories.rs
+++ b/crates/singlefs-harness/tests/random_histories.rs
@@ -32,15 +32,14 @@
 use singlefs_harness::history::{
     allocation_records_on_the_image_under, classify_failure, environment_assignments_in_line,
     execute_history, execute_history_observing, execute_history_with, execute_history_with_faults,
-    generate_history, generate_history_with_weights, process_environment_variable,
-    raised_floor_lands_only_on_abandoned_roots, random_history_to_reproduce_from_environment,
-    run_history_campaign, shrink_to_reproduction, AppliedEffect, ColdStartReadBack, ContentChoice,
-    ContentLength, FailureObservation, FailureSignature, FindingShrinking, FloorTargetChoice,
-    GeneratedHistory, GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
-    HistoryOperation, HistoryOperationKind, HistoryRun, HistorySampling, HistorySeed,
-    HistoryStartingPoint, HistoryTally, MountAllocationComparison, MountSpaceAdmissionOutcome,
-    NewFindingReport, PerStepChecker, RollbackTargetChoice, StepOutcome, StepPosition,
-    RANDOM_HISTORY_VARIABLE_PREFIX,
+    generate_history, generate_history_with_weights, raised_floor_lands_only_on_abandoned_roots,
+    random_history_to_reproduce_from_environment, run_history_campaign, shrink_to_reproduction,
+    AppliedEffect, ColdStartReadBack, ContentChoice, ContentLength, FailureObservation,
+    FailureSignature, FindingShrinking, FloorTargetChoice, GeneratedHistory, GenerationWeights,
+    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
+    HistoryRun, HistorySampling, HistorySeed, HistoryStartingPoint, HistoryTally,
+    MountAllocationComparison, MountSpaceAdmissionOutcome, NewFindingReport, PerStepChecker,
+    RollbackTargetChoice, StepOutcome, StepPosition, RANDOM_HISTORY_VARIABLE_PREFIX,
 };
 use singlefs_harness::memory_pool::{MemoryPool, RecordCheck, SparseBlockDevice};
 use singlefs_harness::model::{
@@ -48,7 +47,9 @@
     ObservedRefusalReason,
 };
 use singlefs_harness::model_comparison::refusal_reason_of_rollback_error;
-use singlefs_harness::numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable;
+use singlefs_harness::numeric_environment_variable::{
+    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
+};
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
 
 // 下面五段的种子基都是同一个：永久固定、不重抽的那个数
@@ -1654,7 +1655,7 @@
         "SINGLEFS_RANDOM_HISTORY_FIRST_SEED",
         SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
     );
-    let seed_count = number_from_environment("SINGLEFS_RANDOM_HISTORY_SEEDS", 1000);
+    let seed_count = count_from_environment("SINGLEFS_RANDOM_HISTORY_SEEDS", 1000);
     let worker_threads = worker_threads_from_environment();
     let sampling = HistorySampling::from_environment(
         RANDOM_HISTORY_VARIABLE_PREFIX,
@@ -1880,6 +1881,24 @@
     number_from_the_value_or_default(name, process_environment_variable(name).as_deref(), default)
 }
 
+/// 「跑几个」的计数变量的值（`None` 是没设，种子数这一类）：没设取 `default`；设了而不是正整数（0 也算）就 panic、消息带变量名与原值。
+/// 0 段的一批没有「已知红」清单外的失败，只是因为一段都没跑，照样判成绿（上游 `show-me-test.md`「扫到 0 项也不是通过」）。
+///
+/// # Panics
+/// 设了而不是正整数。
+fn count_from_the_value_or_default(name: &str, value: Option<&str>, default: u64) -> u64 {
+    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(name, value)
+        .map_or(default, std::num::NonZeroU64::get)
+}
+
+/// 进程环境里的计数变量，判法同 [`count_from_the_value_or_default`]。
+///
+/// # Panics
+/// 设了而不是正整数，或值不是 UTF-8。
+fn count_from_environment(name: &str, default: u64) -> u64 {
+    count_from_the_value_or_default(name, process_environment_variable(name).as_deref(), default)
+}
+
 const WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_RANDOM_HISTORY_THREADS";
 
 /// 线程数变量的值（`None` 是没设）：没设取本机可用并行度（取不到是 1）；设了而不是正整数（0 也算）就 panic、消息带变量名与原值，
@@ -1910,7 +1929,21 @@
 #[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_SEEDS=\"ten\" 认不出")]
 fn an_unreadable_random_history_seed_count_panics_naming_the_variable_and_the_value() {
     let _seed_count =
-        number_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_SEEDS", Some("ten"), 1000);
+        count_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_SEEDS", Some("ten"), 1000);
+}
+
+#[test]
+#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_FIRST_SEED=\"ten\" 认不出")]
+fn an_unreadable_random_history_first_seed_panics_naming_the_variable_and_the_value() {
+    let _first_seed =
+        number_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_FIRST_SEED", Some("ten"), 7);
+}
+
+#[test]
+#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_SEEDS=\"0\" 认不出")]
+fn a_random_history_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
+    let _seed_count =
+        count_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_SEEDS", Some("0"), 1000);
 }
 
 #[test]
```

### 3b 坏盘输入种子数写 0 挪回入口（`/tmp/claude-1000/impl-b0-guard/report.md`）

```diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index bcf2bcbb..92b956ed 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -1767,3 +1767,4 @@ test-script-align-code-r2 C604 源码扫描：src/ 测试模块里的计时用
 计数变量 崩溃注入重放入口短段写数上限写 0 照收（与 none 同义的第二种写法）	crates/singlefs-checker-tier/src/crash_injection.rs	            segment_writes: positive_count_from_the_variable_text(	            segment_writes: usize_from_the_variable_text(	-p singlefs-checker-tier --lib -- crash_injection::tests::a_short_segment_write_bound_of_zero_panics_naming_the_variable_and_the_value	a_short_segment_write_bound_of_zero_panics_naming_the_variable_and_the_value
 扫到 0 项 崩溃注入只判清单外失败那一组不核判过几个崩溃状态（0 段的一批判成绿）	crates/singlefs-checker-tier/tests/crash_injection_campaign.rs	        CrashInjectionAssertions::NewFindingsOnly => {\n            assert_at_least_one_crash_state_was_judged(report);\n        }	        CrashInjectionAssertions::NewFindingsOnly => {}	-p singlefs-checker-tier --test crash_injection_campaign -- a_batch_that_judged_no_crash_state	a_batch_that_judged_no_crash_state_is_red_under_the_new_findings_only_assertions
 判法闭包 记录核对器 σ 那条流的入口文件经 history 引进程环境变量的读法（随机历史脚手架进 c561 的判法摘要）	crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs	        singlefs_harness::numeric_environment_variable::process_environment_variable(\n            CPU_THREADS_ENVIRONMENT_VARIABLE,\n        )\n        .as_deref(),	        { let _ = singlefs_harness::history::RANDOM_HISTORY_VARIABLE_PREFIX; singlefs_harness::numeric_environment_variable::process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE) }.as_deref(),	-p singlefs-checker-tier --test record_checker_judges_absence_by_the_persisted_set -- this_flow_does_not_reference	this_flow_does_not_reference_the_random_history_scaffolding
+计数变量 坏盘输入入口种子数改回非负整数读法（写 0 照收；测试侧那一遍已删，判法只在入口）	crates/singlefs-checker-tier/src/bad_disk_input.rs	    let seed_count = count_from_reproduction_environment(	    let seed_count = number_from_reproduction_environment(	-p singlefs-checker-tier --test bad_disk_input_campaign -- a_bad_disk_seed_count_of_zero	a_bad_disk_seed_count_of_zero_panics_naming_the_variable_and_the_value
diff --git a/crates/singlefs-checker-tier/src/bad_disk_input.rs b/crates/singlefs-checker-tier/src/bad_disk_input.rs
index dfe7cf49..2feb4c0d 100644
--- a/crates/singlefs-checker-tier/src/bad_disk_input.rs
+++ b/crates/singlefs-checker-tier/src/bad_disk_input.rs
@@ -49,10 +49,10 @@ use singlefs_format::{
 
 use crate::crash_injection::{seed_slices, CrashInjectionWorkerThreads};
 use singlefs_harness::history::{
-    execute_history_with, generate_history_with_weights, number_from_reproduction_environment,
-    render_environment_assignments, with_panic_capture, CapturedPanic, GenerationWeights,
-    HistoryDeviceWidth, HistoryExecution, HistorySampling, HistorySeed,
-    ReproductionEnvironmentError, ReproductionVariablePrefix, SeededRandomSource,
+    count_from_reproduction_environment, execute_history_with, generate_history_with_weights,
+    number_from_reproduction_environment, render_environment_assignments, with_panic_capture,
+    CapturedPanic, GenerationWeights, HistoryDeviceWidth, HistoryExecution, HistorySampling,
+    HistorySeed, ReproductionEnvironmentError, ReproductionVariablePrefix, SeededRandomSource,
 };
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice, SECTOR_BYTES};
 use singlefs_harness::model::ObservedReadBack;
@@ -894,10 +894,11 @@ pub fn bad_disk_input_replay_line(
 }
 
 /// 大档那条入口读环境：读的变量与 [`bad_disk_input_replay_line`] 写出的逐个相同，没设的取 `defaults` 的；
-/// 线程数不在重放行里，照 `defaults` 的。
+/// 线程数不在重放行里，照 `defaults` 的。种子数是「跑几段」的计数，经 `history` 的计数读法读（写 0 与写错一样认不得：
+/// 0 段的一批照「零 panic、不读回没提交过的版本」判成绿，上游 `show-me-test.md`「扫到 0 项也不是通过」）。
 ///
 /// # Errors
-/// 某个变量的取值认不得。
+/// 某个变量的取值认不得（种子数写 0 也算）。
 pub fn bad_disk_campaign_from_environment(
     lookup: &dyn Fn(&str) -> Option<String>,
     defaults: &BadDiskCampaign,
@@ -907,7 +908,7 @@ pub fn bad_disk_campaign_from_environment(
         &BadDiskInputVariable::FirstSeed.variable_name(),
         defaults.first_seed,
     )?;
-    let seed_count = number_from_reproduction_environment(
+    let seed_count = count_from_reproduction_environment(
         lookup,
         &BadDiskInputVariable::Seeds.variable_name(),
         defaults.seed_count,
diff --git a/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs b/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs
index e05691a2..dde05090 100644
--- a/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs
+++ b/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs
@@ -23,11 +23,11 @@ use singlefs_checker_tier::bad_disk_input::{
     known_panic_site_among, known_panic_site_of,
     newest_root_has_the_trees_the_targeted_damages_need,
     release_check_of_the_instance_table_placement_released_on_the_second_device_only,
-    run_bad_disk_campaign, BadDiskCampaign, BadDiskFinding, BadDiskInputVariable,
-    BadDiskInputWorkerThreads, BadDiskObservation, BadDiskReport, BadDiskTally, DamageKind,
-    DamageRefusedAtTheHeaderJudgement, DamagedImage, KnownPanicSite, ReadBackVerdict,
-    ReaderOutcome, BAD_DISK_INPUT_WORKER_THREADS_ENVIRONMENT_VARIABLE, EVERY_BASE_IMAGE_TIER,
-    EVERY_DAMAGE_KIND, KNOWN_PANIC_SITES,
+    run_bad_disk_campaign, BadDiskCampaign, BadDiskFinding, BadDiskInputWorkerThreads,
+    BadDiskObservation, BadDiskReport, BadDiskTally, DamageKind, DamageRefusedAtTheHeaderJudgement,
+    DamagedImage, KnownPanicSite, ReadBackVerdict, ReaderOutcome,
+    BAD_DISK_INPUT_WORKER_THREADS_ENVIRONMENT_VARIABLE, EVERY_BASE_IMAGE_TIER, EVERY_DAMAGE_KIND,
+    KNOWN_PANIC_SITES,
 };
 use singlefs_core::address::DeviceIdentity;
 use singlefs_core::admission::SpaceAdmission;
@@ -36,10 +36,9 @@ use singlefs_core::mount::{mount_writable, MountError};
 use singlefs_core::transaction::TransactionUnit;
 use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
 use singlefs_harness::history::{
-    count_from_reproduction_environment, environment_assignments_in_line, execute_history_with,
-    generate_history_with_weights, CapturedPanic, GeneratedHistory, GenerationWeights,
-    HistoryDeviceWidth, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
-    PerStepChecker, SeededRandomSource,
+    environment_assignments_in_line, execute_history_with, generate_history_with_weights,
+    CapturedPanic, GeneratedHistory, GenerationWeights, HistoryDeviceWidth, HistoryExecution,
+    HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, SeededRandomSource,
 };
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
 use singlefs_harness::numeric_environment_variable::process_environment_variable;
@@ -1028,18 +1027,11 @@ const SITES_DELETED_BY_THE_THIRD_BATCH: [KnownPanicSite; 3] = [
 /// 大档那条入口读环境：变量与判红时打的重放行同一张表（`bad_disk_campaign_from_environment`），没设的取大档的默认
 /// （种子基起 500 段、每段 16 步、各类操作都抽、不跑每一步的 checker、两块 4 GiB 的盘、判空间准入），
 /// 线程数从 `SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS` 取。入口与「重放行喂回入口」那两条自证都经这一个函数。
-/// 种子数是计数：先照 `history` 的计数读法核一遍（写 0 与写错一样停下，0 段的一批照「零 panic、不读回没提交过的版本」判成绿），
-/// 读回的种子数仍由 `bad_disk_campaign_from_environment` 给。
+/// 种子数写 0 由入口自己拒（`bad_disk_campaign_from_environment` 按计数读它），这里不另读一遍。
 ///
 /// # Panics
 /// 某个变量的取值认不得（种子数写 0 也算）：消息带变量名与原值。
 fn large_tier_campaign_from(lookup: &dyn Fn(&str) -> Option<String>) -> BadDiskCampaign {
-    let _seed_count_checked_to_be_positive = count_from_reproduction_environment(
-        lookup,
-        &BadDiskInputVariable::Seeds.variable_name(),
-        1,
-    )
-    .unwrap_or_else(|error| panic!("{}", error.render()));
     bad_disk_campaign_from_environment(
         lookup,
         &BadDiskCampaign {
```

### 3c 没留 diff 的两件，判冻结副本里的现状

- 窄一断言与白名单（`/tmp/claude-1000/impl-narrow-guard/report.md`）：`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 447–470 行一带（`recover(…, JournalPolicy::Consult)` 落到的根与基镜像最新的根不等就拒）；`crates/singlefs-checker-tier/src/crash_identity.rs` 第 890 行 `RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED` 与第 1584–1650 行引入者逐个判。
- 抬 F 搭建拆分（`/tmp/claude-1000/impl-floor-raise-setup/report.md`）：新文件 `crates/singlefs-harness/tests/common_admission_pool/mod.rs`，`crates/singlefs-harness/tests/common_admission/mod.rs` 经 `#[path]` 共用并转出。
