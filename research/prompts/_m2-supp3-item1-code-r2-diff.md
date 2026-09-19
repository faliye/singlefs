# 附录二：增补 3 第 1 件第二轮与层 0 并行化第一轮的代码改动（基准 `00c9d4f`；生成于 2026-09-18 20:34 UTC）

## 一、diff（相对 `00c9d4f`，`git diff 00c9d4f -- crates/singlefs-harness/src/crash.rs crates/singlefs-harness/src/lib.rs crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs crates/mutations.tsv .claude/gate.d/54-layer0-replay.sh` 原样）

```diff
diff --git a/.claude/gate.d/54-layer0-replay.sh b/.claude/gate.d/54-layer0-replay.sh
index da113ce..0f435ab 100755
--- a/.claude/gate.d/54-layer0-replay.sh
+++ b/.claude/gate.d/54-layer0-replay.sh
@@ -5,16 +5,63 @@
 # 里程碑「第一个事务」步 7：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑三件事：步 6 的恢复与 oracle、
 # 池级 checker（23 条不变量）、记录核对器（根在案而记录缺席、恢复自称新态而单元缺席）；三者的计数都由用例钉死。
 # 全量 262165 个状态在 debug 下要几分钟，所以平时 `cargo test` 里那条用例标 ignored；这里在 release 下跑它，
-# 把用例打印的 `LAYER0 …` 计数行报出来。exhaustive=true 才算全量，不是全量判红——层 0 全量是里程碑出口。
+# 把用例打印的 `LAYER0 …` 计数行与逐条不变量的 `CHECKER …` 行报出来。exhaustive=true 才算全量，不是全量判红——层 0 全量是里程碑出口。
 # 判别力：用例自己对着产物的十个计数断言（states / violations / root_persisted … 逐字），oracle 的判别力由同文件的靶向阳性对照证明
 # （根槽已持久而某个单元两份都没持久 ⇒ 8 个单元逐个都判红）。本阶段没有 fixtures 样本：判红要 cargo 真跑，装不进 fixtures 目录。
+#
+# 多线程（增补 2 收口表第 41 行；2026-09-18 用户定：测试与崩溃检测优先多线程）：crash.rs 按状态序号区间切片、多线程跑，
+# 线程数由 SINGLEFS_LAYER0_THREADS 传进去——没设就取本机核数（nproc）。每跑完一片，用例打一行 `LAYER0_PROGRESS`（片号、序号区间、段号、
+# 已跑完的状态数），这里边跑边转到本阶段的输出里，不删；成功行里报实际起了几个工作线程。
+# 没显式把 SINGLEFS_LAYER0_THREADS 设成 1、而本机多于 1 核却只起了 1 个工作线程：判红（多半是线程数没传进去）。
 set -uo pipefail
 ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
 cd "$ROOT" 2>/dev/null || exit 2
 [[ -f Cargo.toml && -d crates/singlefs-harness ]] || { echo "  ! 没有 crates/singlefs-harness，本阶段跳过（步 0 之前没有装置）"; exit 77; }
 
+machine_cores="$(nproc)"
+if [[ -n "${SINGLEFS_LAYER0_THREADS:-}" ]]; then
+  threads_origin="显式设的"
+else
+  SINGLEFS_LAYER0_THREADS="$machine_cores"
+  threads_origin="没设，取本机核数"
+fi
+export SINGLEFS_LAYER0_THREADS
+
+# run_layer0_test_binary <测试二进制> <日志>：cargo 的整段输出进日志；`LAYER0_PROGRESS` 行边跑边转到本阶段的输出（跑全量时这里一直在涨）。
+run_layer0_test_binary() {
+  cargo test --release -p singlefs-harness --test "$1" -- --include-ignored --nocapture 2>&1 \
+    | tee "$2" \
+    | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
+    | sed -u 's/^/    /'
+  return "${PIPESTATUS[0]}"
+}
+
+# worker_threads_of_full_run <日志> <计数行>：计数行里的状态数对上的那一行 `LAYER0_PARALLEL_FINISHED`，取实际起的工作线程数。
+worker_threads_of_full_run() {
+  local states
+  states="$(sed -n 's/^[A-Z0-9]* states=\([0-9]*\) .*/\1/p' <<<"$2")"
+  grep "^LAYER0_PARALLEL_FINISHED states=$states " "$1" | head -1 | sed -n 's/.* worker_threads=\([0-9]*\) .*/\1/p'
+}
+
+# 没显式设成 1、本机多于 1 核却只起了 1 个工作线程（或根本没打收尾行）：判红。返回 0 表示线程数没问题。
+worker_threads_are_acceptable() { # <流的名字> <实际起的工作线程数>
+  if [[ -z "$2" ]]; then
+    echo "  ✗ $1：全量用例跑过了，却没打印状态数对得上的 LAYER0_PARALLEL_FINISHED 行，判不出起了几个工作线程"
+    echo "     → 怎么办：全量那条用例要经 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0_in_state_slices 跑（它开跑与跑完各打一行 LAYER0_PARALLEL_*）；"
+    echo "                绕开它自己逐个跑状态的写法退回了单线程，改回去。"
+    return 1
+  fi
+  if [[ "$2" == 1 && "$machine_cores" -gt 1 && ! ( "$threads_origin" == "显式设的" && "$SINGLEFS_LAYER0_THREADS" == 1 ) ]]; then
+    echo "  ✗ $1：本机 $machine_cores 核、SINGLEFS_LAYER0_THREADS=$SINGLEFS_LAYER0_THREADS（$threads_origin），全量枚举却只起了 1 个工作线程"
+    echo "     → 怎么办：看 crash.rs 的 Layer0Parallelism::from_environment 读没读到 SINGLEFS_LAYER0_THREADS、state_slices 切出来的片数够不够分给每个线程；"
+    echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_LAYER0_THREADS=1 bash .claude/gate.d/54-layer0-replay.sh。"
+    return 1
+  fi
+  return 0
+}
+
 log="$(mktemp)"
-if ! cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture >"$log" 2>&1; then
+if ! run_layer0_test_binary first_transaction_step_seven_layer0 "$log"; then
   tail -40 "$log"
   echo "  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）"
   echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture"
@@ -23,6 +70,8 @@ if ! cargo test --release -p singlefs-harness --test first_transaction_step_seve
   exit 1
 fi
 line="$(grep '^LAYER0 ' "$log" | head -1)"
+checker_line="$(grep -A1 '^LAYER0 ' "$log" | grep '^CHECKER ' | head -1)"
+worker_threads="$(worker_threads_of_full_run "$log" "$line")"
 rm -f "$log"
 if [[ -z "$line" ]]; then
   echo "  ✗ 用例跑过了，却没打印 LAYER0 计数行"
@@ -34,10 +83,12 @@ if [[ "$line" != *"exhaustive=true"* ]]; then
   echo "     → 怎么办：枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)；少了说明某一段没展开子集，去 crash.rs 的 enumerate_layer0 看。"
   exit 1
 fi
-echo "  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器）：${line#LAYER0 }"
+worker_threads_are_acceptable "第一个事务那条流" "$worker_threads" || exit 1
+echo "  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；${worker_threads} 个工作线程，SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核）：${line#LAYER0 }"
+echo "  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：${checker_line#CHECKER }"
 # 第二个事务（发布 B）：取号 → 暖机 → A → B 整条流，多版本 oracle（里程碑「第二个事务」步 0 / 步 6 在 B 上的那一半）。
 log_b="$(mktemp)"
-if ! cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --include-ignored --nocapture >"$log_b" 2>&1; then
+if ! run_layer0_test_binary second_transaction_step_zero_layer0 "$log_b"; then
   tail -40 "$log_b"
   echo "  ✗ 两次发布那条流的层 0 用例判红（上面是 cargo test 的尾部）"
   echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --include-ignored --nocapture"
@@ -46,6 +97,7 @@ if ! cargo test --release -p singlefs-harness --test second_transaction_step_zer
   exit 1
 fi
 line_b="$(grep '^LAYER0B ' "$log_b" | head -1)"
+worker_threads_b="$(worker_threads_of_full_run "$log_b" "$line_b")"
 rm -f "$log_b"
 if [[ -z "$line_b" ]]; then
   echo "  ✗ 两次发布那条流的用例跑过了，却没打印 LAYER0B 计数行"
@@ -57,4 +109,5 @@ if [[ "$line_b" != *"exhaustive=true"* ]]; then
   echo "     → 怎么办：枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)；少了说明某一段没展开子集，去 crash.rs 的 enumerate_layer0_selecting_versions 看。"
   exit 1
 fi
-echo "  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle）：${line_b#LAYER0B }"
+worker_threads_are_acceptable "两次发布那条流" "$worker_threads_b" || exit 1
+echo "  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；${worker_threads_b} 个工作线程，SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核）：${line_b#LAYER0B }"
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index f563881..8b4820a 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -126,3 +126,16 @@ Z1-a：取号之前不算实例表	crates/singlefs-core/src/mount.rs
 Z1-a：实例表准入漏算链指针	crates/singlefs-core/src/mount.rs	            if rows_in_version + rows_to_write + 1 > records_per_page {	            if rows_in_version + rows_to_write > records_per_page {	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- writable_mounts_fill	writable_mounts_fill_the_instance_table_page_and_the_next_one_is_refused_before_acquisition
 Z1-a：实例表准入把正好写满一片也拒掉	crates/singlefs-core/src/mount.rs	            if rows_in_version + rows_to_write + 1 > records_per_page {	            if rows_in_version + rows_to_write + 1 >= records_per_page {	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- mount_after_crashes	mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it
 Z1-a：实例表准入按每次挂载只写一行算	crates/singlefs-core/src/mount.rs	        rows_written.len(),\n        warm_up_publishes_planned.len(),	        1,\n        warm_up_publishes_planned.len(),	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- mount_after_crashes	mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it
+增补 3 第 1 件：随机历史快档判出「复用时追加记录而不改写」（第 41 行同一处）	crates/singlefs-core/src/allocator.rs	            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");\n            existing.generation = generation;\n            existing.is_released = false;	            records.push(AllocationRecord {\n                device,\n                slot: placement.slot,\n                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),\n                generation,\n                is_released: false,\n            });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
+增补 3 第 1 件：随机历史偏向抬 F 之后复用的取样点判出「复用时新记录罩住的已回收记录不删」（第 121 行同一处）	crates/singlefs-core/src/allocator.rs	            records.retain(|record| !(record.device == device && record.slot == record_slot));	            // 变异：罩住的已回收记录不删	-p singlefs-harness --test second_transaction_supplement_three_random_history -- reuse_heavy_random_histories	reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms
+增补 2 第 41 行 层 0 并行：丢掉第一片（状态数等于闭式的断言要红）	crates/singlefs-harness/src/crash.rs	    (0..state_count.div_ceil(states_per_slice))	    (1..state_count.div_ceil(states_per_slice))	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+增补 2 第 41 行 层 0 并行：相邻两片重叠（状态数等于闭式的断言要红）	crates/singlefs-harness/src/crash.rs	                .saturating_add(states_per_slice)\n                .min(state_count);	                .saturating_add(states_per_slice)\n                .saturating_add(1)\n                .min(state_count);	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+增补 2 第 41 行 层 0 并行：相邻两片重叠（切片单测）	crates/singlefs-harness/src/crash.rs	                .saturating_add(states_per_slice)\n                .min(state_count);	                .saturating_add(states_per_slice)\n                .saturating_add(1)\n                .min(state_count);	-p singlefs-harness --lib -- state_slices_cover_every_state	state_slices_cover_every_state_exactly_once_in_ordinal_order
+增补 2 第 41 行 层 0 并行：按序号取状态时段内子集掩码取反	crates/singlefs-harness/src/crash.rs	            let subset_mask = ordinal - self.state_ranges_by_segment[segment_of_state].start;	            let subset_mask = !(ordinal - self.state_ranges_by_segment[segment_of_state].start);	-p singlefs-harness --lib -- the_state_plan_hands_out	the_state_plan_hands_out_the_same_persisted_sets_in_the_same_order_as_walking_segment_by_segment
+增补 2 第 41 行 层 0 并行：并片时第一处违例取后面那一片的	crates/singlefs-harness/src/crash.rs	        if self.first_violation.is_none() {\n            self.first_violation = first_violation;\n        }	        if first_violation.is_some() {\n            self.first_violation = first_violation;\n        }	-p singlefs-harness --test second_transaction_step_zero_layer0 -- one_state_slices_on_eight_threads	one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread
+增补 2 第 41 行 层 0 并行：并片时 checker 每条不变量的第一处违例取后面那一片的	crates/singlefs-harness/src/crash.rs	            self.checker_first_violation\n                .entry(invariant)\n                .or_insert(detail);	            self.checker_first_violation.insert(invariant, detail);	-p singlefs-harness --lib -- absorbing_a_following_slice	absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation
+增补 2 第 41 行 层 0 并行：不读 SINGLEFS_LAYER0_THREADS	crates/singlefs-harness/src/crash.rs	        let (worker_threads, worker_threads_source) = match environment_value {	        let (worker_threads, worker_threads_source) = match environment_value.and(Err::<String, _>(std::env::VarError::NotPresent)) {	-p singlefs-harness --lib -- worker_threads_come_from_the_environment_variable	worker_threads_come_from_the_environment_variable_before_available_parallelism
+增补 2 第 41 行 层 0 并行：SINGLEFS_LAYER0_THREADS=0 悄悄退回 1 个线程	crates/singlefs-harness/src/crash.rs	                text.parse::<NonZeroUsize>().unwrap_or_else(|error| {	                text.parse::<NonZeroUsize>().or(Ok::<NonZeroUsize, std::num::ParseIntError>(NonZeroUsize::MIN)).unwrap_or_else(|error| {	-p singlefs-harness --lib -- zero_worker_threads_in_the_environment_variable	zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back
+增补 3 第 1 件（代码三方第一轮第 1 条）：已知红第 1 条不看 F 是否落在回退留下的空档里	crates/singlefs-harness/src/history.rs	        && observation.raised_floor_lands_only_on_abandoned_roots == Some(true)\n        && !observation.root_ring_has_turned()	        && !observation.root_ring_has_turned()	-p singlefs-harness --test second_transaction_supplement_three_random_history -- an_allocated_statistic_over_count	an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding
+增补 3 第 1 件（代码三方第一轮第 3 条）：随机历史快档判出「复用改写已回收记录时不改分配代」（N2）	crates/singlefs-core/src/allocator.rs	            existing.generation = generation;\n            existing.is_released = false;	            existing.is_released = false;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
+增补 3 第 1 件（代码三方第一轮第 4 条）：随机历史快档判出「checker 判绿的镜像上冷启动读回报错」（B6：补齐字节从最后一个载荷字节算起）	crates/singlefs-core/src/unit.rs	    if bytes[payload_end..].iter().any(|byte| *byte != 0) {	    if bytes[payload_end - 1..].iter().any(|byte| *byte != 0) {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
diff --git a/crates/singlefs-harness/src/crash.rs b/crates/singlefs-harness/src/crash.rs
index c412988..a5c9e94 100644
--- a/crates/singlefs-harness/src/crash.rs
+++ b/crates/singlefs-harness/src/crash.rs
@@ -5,6 +5,11 @@
 use std::collections::BTreeMap;
 
 use std::collections::BTreeSet;
+use std::num::{NonZeroU64, NonZeroUsize};
+use std::ops::Range;
+use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
+use std::sync::mpsc;
+use std::time::Instant;
 
 use singlefs_checker::image::{ImageReader, InvariantVerdict};
 use singlefs_checker::walk::check_pool_image;
@@ -34,22 +39,29 @@ pub struct SparseDevice {
 impl SparseDevice {
     #[must_use]
     pub fn read(&self, offset: DeviceOffsetInBytes, length: usize) -> Vec<u8> {
+        let mut out = vec![0u8; length];
+        self.read_into(offset, &mut out);
+        out
+    }
+    /// 读进调用方的缓冲：先整段写 0，再只拷区间里写过的扇区（按区间查一次，不逐扇区查）——可写挂载与冷启动要把 768 MiB 的
+    /// journal 环逐条读一遍，几乎全是没写过的扇区（随机历史，增补 3 第 1 件）。
+    pub fn read_into(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) {
         assert!(offset.0.is_multiple_of(SECTOR_BYTES), "读要按扇区对齐");
-        let length_in_bytes = u64::try_from(length).expect("长度");
+        let length_in_bytes = u64::try_from(buffer.len()).expect("长度");
         assert!(
             length_in_bytes.is_multiple_of(SECTOR_BYTES),
             "读长度要是整扇区"
         );
         let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
-        let mut out = vec![0u8; length];
+        buffer.fill(0);
         let first_sector = offset.0 / SECTOR_BYTES;
-        for sector_index in 0..length_in_bytes / SECTOR_BYTES {
-            if let Some(sector) = self.sectors.get(&(first_sector + sector_index)) {
-                let start = usize::try_from(sector_index).expect("扇区下标") * sector_bytes;
-                out[start..start + sector_bytes].copy_from_slice(sector);
-            }
+        for (sector, bytes) in self
+            .sectors
+            .range(first_sector..first_sector + length_in_bytes / SECTOR_BYTES)
+        {
+            let start = usize::try_from(sector - first_sector).expect("扇区下标") * sector_bytes;
+            buffer[start..start + sector_bytes].copy_from_slice(bytes);
         }
-        out
     }
     pub fn write(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
         assert!(offset.0.is_multiple_of(SECTOR_BYTES), "写要按扇区对齐");
@@ -102,7 +114,7 @@ impl singlefs_core::block_device::BlockDevice for SparseBlockDevice {
         offset: DeviceOffsetInBytes,
         buffer: &mut [u8],
     ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
-        buffer.copy_from_slice(&self.image.read(offset, buffer.len()));
+        self.image.read_into(offset, buffer);
         Ok(())
     }
     fn write_at(
@@ -550,6 +562,66 @@ impl Layer0Tally {
             .collect::<Vec<String>>()
             .join(" ")
     }
+
+    /// 把紧跟在后面的那一片的计数并进来：计数逐项相加；「第一处」只在前面各片都没有时取这一片的。各片按状态序号从小到大并，
+    /// 「第一处」就是序号最小的那一处，与单线程逐个跑逐项相同。按字段拆开写全：新加一个字段而这里没并，编译不过。
+    fn absorb_following_slice(&mut self, following_slice: Layer0Tally) {
+        let Layer0Tally {
+            states,
+            violations,
+            root_persisted_states,
+            no_file_states,
+            file_read_states,
+            failed_states,
+            journal_differing_states,
+            verification_ran_states,
+            verification_failed_states,
+            first_violation,
+            ignored_violations,
+            first_ignored_violation,
+            record_root_without_record,
+            record_claimed_state_missing_unit,
+            checker_evaluated_states,
+            checker_violated_states,
+            checker_first_violation,
+            checker_not_applicable_states,
+        } = following_slice;
+        self.states += states;
+        self.violations += violations;
+        self.root_persisted_states += root_persisted_states;
+        self.no_file_states += no_file_states;
+        self.file_read_states += file_read_states;
+        self.failed_states += failed_states;
+        self.journal_differing_states += journal_differing_states;
+        self.verification_ran_states += verification_ran_states;
+        self.verification_failed_states += verification_failed_states;
+        if self.first_violation.is_none() {
+            self.first_violation = first_violation;
+        }
+        self.ignored_violations += ignored_violations;
+        if self.first_ignored_violation.is_none() {
+            self.first_ignored_violation = first_ignored_violation;
+        }
+        self.record_root_without_record += record_root_without_record;
+        self.record_claimed_state_missing_unit += record_claimed_state_missing_unit;
+        for (invariant, evaluated_states) in checker_evaluated_states {
+            *self.checker_evaluated_states.entry(invariant).or_insert(0) += evaluated_states;
+        }
+        for (invariant, violated_states) in checker_violated_states {
+            *self.checker_violated_states.entry(invariant).or_insert(0) += violated_states;
+        }
+        for (invariant, detail) in checker_first_violation {
+            self.checker_first_violation
+                .entry(invariant)
+                .or_insert(detail);
+        }
+        for (invariant, not_applicable_states) in checker_not_applicable_states {
+            *self
+                .checker_not_applicable_states
+                .entry(invariant)
+                .or_insert(0) += not_applicable_states;
+        }
+    }
 }
 
 /// oracle（E77（发布的持久顺序） 判据 1）：读回的内容要对；根槽已持久就不许恢复到旧态；走读不许失败。
@@ -811,8 +883,285 @@ pub fn enumerate_layer0_selecting(
     enumerate_layer0_selecting_versions(base, writes, segments, root_index, &versions, expand)
 }
 
+/// 层 0 枚举的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。门禁 54 号显式传进来。
+pub const LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_LAYER0_THREADS";
+
+/// 默认切法的片数下限：线程数为 1 时全量用例也按片报进度。
+const LAYER0_MINIMUM_SLICE_COUNT: u64 = 64;
+/// 默认切法里每个工作线程摊到的片数：越往后的状态历史越长、越贵，片切得比线程多，先跑完的线程接着领下一片。
+const LAYER0_SLICES_PER_WORKER_THREAD: u64 = 16;
+/// 默认切法里每片的状态数下限：平时 `cargo test` 里几十个状态的枚举只切成几片，进度行不刷屏。
+const LAYER0_MINIMUM_STATES_PER_SLICE: u64 = 16;
+
+/// 工作线程数是从哪来的：与实际起的线程数一起打进 `LAYER0_PARALLEL_START` / `LAYER0_PARALLEL_FINISHED` 两行（门禁 54 号拿实际起的线程数判「没显式设成 1、机器多于 1 核却只用了 1 个线程」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum Layer0WorkerThreadsSource {
+    /// 环境变量 [`LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
+    EnvironmentVariable,
+    /// 没设环境变量，取 `available_parallelism`。
+    AvailableParallelism,
+    /// 没设环境变量，`available_parallelism` 也报不出来：只用 1 个线程，照实报出来。
+    AvailableParallelismUnknown,
+    /// 调用方在代码里直接给的（用例拿不同切法对拍）。
+    GivenByCaller,
+}
+
+impl Layer0WorkerThreadsSource {
+    fn name(self) -> &'static str {
+        match self {
+            Self::EnvironmentVariable => "environment_variable",
+            Self::AvailableParallelism => "available_parallelism",
+            Self::AvailableParallelismUnknown => "available_parallelism_unknown",
+            Self::GivenByCaller => "given_by_caller",
+        }
+    }
+}
+
+/// 每片几个状态。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum Layer0SliceLength {
+    /// 片数取 max(64, 16 × 工作线程数)，每片至少 16 个状态。
+    ScaledToWorkerThreads,
+    /// 每片固定这么多个状态（用例把切法推到两头：每片 1 个，或整条流 1 片）。
+    StatesPerSlice(NonZeroU64),
+}
+
+/// 层 0 按状态序号区间切片、多线程跑：几个工作线程、每片几个状态。切法与线程数只影响跑得多快，不影响计数与「第一处违例」。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct Layer0Parallelism {
+    pub worker_threads: NonZeroUsize,
+    pub worker_threads_source: Layer0WorkerThreadsSource,
+    pub slice_length: Layer0SliceLength,
+}
+
+impl Layer0Parallelism {
+    /// 线程数取环境变量 [`LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`]，没设就取 `available_parallelism`；片长按线程数定。
+    ///
+    /// # Panics
+    /// 环境变量设了却不是正整数：配错了就停，不悄悄退回单线程。
+    #[must_use]
+    pub fn from_environment() -> Self {
+        Self::from_environment_value(
+            std::env::var(LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE),
+            std::thread::available_parallelism,
+        )
+    }
+
+    /// [`Self::from_environment`] 的判定本身：环境变量读到什么、`available_parallelism` 报什么都由调用方给（用例不改进程的环境变量）。
+    ///
+    /// # Panics
+    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）。
+    #[must_use]
+    fn from_environment_value(
+        environment_value: Result<String, std::env::VarError>,
+        available_parallelism: impl FnOnce() -> std::io::Result<NonZeroUsize>,
+    ) -> Self {
+        let (worker_threads, worker_threads_source) = match environment_value {
+            Ok(text) => (
+                text.parse::<NonZeroUsize>().unwrap_or_else(|error| {
+                    panic!(
+                        "{LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}：{error}"
+                    )
+                }),
+                Layer0WorkerThreadsSource::EnvironmentVariable,
+            ),
+            Err(std::env::VarError::NotPresent) => match available_parallelism() {
+                Ok(available) => (available, Layer0WorkerThreadsSource::AvailableParallelism),
+                Err(_unavailable) => (
+                    NonZeroUsize::MIN,
+                    Layer0WorkerThreadsSource::AvailableParallelismUnknown,
+                ),
+            },
+            Err(std::env::VarError::NotUnicode(raw)) => panic!(
+                "{LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到的不是 UTF-8：{raw:?}"
+            ),
+        };
+        Self {
+            worker_threads,
+            worker_threads_source,
+            slice_length: Layer0SliceLength::ScaledToWorkerThreads,
+        }
+    }
+}
+
+/// 枚举次序里每个状态的序号怎么落到「第几段、段内哪个子集」：次序与单线程逐段逐个子集走相同——
+/// 前面的段全持久 + 当前段任意真子集（子集掩码从 0 数到 2^|段| − 2），最后再加全部持久那一个状态。
+struct Layer0StatePlan<'segments> {
+    segments: &'segments [Vec<usize>],
+    /// 每一段展开出来的状态的序号区间，首尾相接、随段号递增；不展开的段是空区间。
+    state_ranges_by_segment: Vec<Range<u64>>,
+    /// 状态总数：各段展开出来的，再加最后全部持久那一个。
+    state_count: u64,
+}
+
+impl<'segments> Layer0StatePlan<'segments> {
+    /// `expand` 只在调用线程上逐段问一次，所以它不必能跨线程。
+    fn new(segments: &'segments [Vec<usize>], expand: &dyn Fn(usize, &[usize]) -> bool) -> Self {
+        let mut next_ordinal = 0u64;
+        let state_ranges_by_segment = segments
+            .iter()
+            .enumerate()
+            .map(|(segment_index, segment)| {
+                let first_ordinal = next_ordinal;
+                if expand(segment_index, segment) {
+                    next_ordinal += (1u64 << segment.len()) - 1;
+                }
+                first_ordinal..next_ordinal
+            })
+            .collect();
+        Self {
+            segments,
+            state_ranges_by_segment,
+            state_count: next_ordinal + 1,
+        }
+    }
+
+    /// 序号落在哪一段；等于段数说明是最后全部持久那一个状态。
+    fn segment_of_state(&self, ordinal: u64) -> usize {
+        self.state_ranges_by_segment
+            .partition_point(|state_range| state_range.end <= ordinal)
+    }
+
+    /// 这个状态里持久了的写：所在的段之前每一段整段持久（展不展开都一样），所在的段按段内子集掩码（第 k 位对应段里第 k 个写）。
+    fn persisted_writes_of_state(&self, ordinal: u64, write_count: usize) -> Vec<bool> {
+        let segment_of_state = self.segment_of_state(ordinal);
+        let mut persisted = vec![false; write_count];
+        for segment in &self.segments[..segment_of_state] {
+            for write_index in segment {
+                persisted[*write_index] = true;
+            }
+        }
+        if let Some(segment) = self.segments.get(segment_of_state) {
+            let subset_mask = ordinal - self.state_ranges_by_segment[segment_of_state].start;
+            for (bit, write_index) in segment.iter().enumerate() {
+                if subset_mask & (1 << bit) != 0 {
+                    persisted[*write_index] = true;
+                }
+            }
+        }
+        persisted
+    }
+
+    /// 进度行里的段号：最后全部持久那一个状态不在任何一段里，报成 `all_persisted`。
+    fn segment_label(&self, segment_index: usize) -> String {
+        if segment_index < self.segments.len() {
+            segment_index.to_string()
+        } else {
+            "all_persisted".to_string()
+        }
+    }
+}
+
+/// 把 [0, `state_count`) 按序号切成首尾相接的区间。
+fn state_slices(state_count: u64, parallelism: &Layer0Parallelism) -> Vec<Range<u64>> {
+    let states_per_slice = match parallelism.slice_length {
+        Layer0SliceLength::ScaledToWorkerThreads => {
+            let worker_threads =
+                u64::try_from(parallelism.worker_threads.get()).expect("线程数装得进 u64");
+            let slice_count = LAYER0_MINIMUM_SLICE_COUNT
+                .max(worker_threads.saturating_mul(LAYER0_SLICES_PER_WORKER_THREAD));
+            state_count
+                .div_ceil(slice_count)
+                .max(LAYER0_MINIMUM_STATES_PER_SLICE)
+        }
+        Layer0SliceLength::StatesPerSlice(states_per_slice) => states_per_slice.get(),
+    };
+    (0..state_count.div_ceil(states_per_slice))
+        .map(|slice_index| {
+            let first_ordinal = slice_index * states_per_slice;
+            let end_ordinal = first_ordinal
+                .saturating_add(states_per_slice)
+                .min(state_count);
+            first_ordinal..end_ordinal
+        })
+        .collect()
+}
+
+/// 逐状态的观察者：拿到崩溃镜像与看 journal 那一遍恢复的报告。
+pub type Layer0StateObserver<'observer> =
+    &'observer mut dyn FnMut(&CrashImage<'_>, &RecoveryReport);
+
+/// 工作线程评状态时 panic，就把旗子立起来：别的工作线程看到旗子不再领新片，调用线程不必等整条流跑完才知道红了。
+struct RaiseFlagWhenPanicking<'flag>(&'flag AtomicBool);
+
+impl Drop for RaiseFlagWhenPanicking<'_> {
+    fn drop(&mut self) {
+        if std::thread::panicking() {
+            self.0.store(true, Ordering::Relaxed);
+        }
+    }
+}
+
+/// 工作线程要不要把每个状态的持久集合与看 journal 那一遍恢复的报告带回调用线程。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum StateReportRetention {
+    /// 有观察者：带回去，由调用线程按序号交给它。
+    HandEachStateToObserver,
+    /// 没有观察者：只带计数回去（全量流上两百多万个报告不必留）。
+    CountOnly,
+}
+
+/// 一片跑完交回调用线程的东西。
+struct FinishedSlice {
+    slice_index: usize,
+    tally: Layer0Tally,
+    /// 按序号排好的（持久集合，看 journal 那一遍恢复的报告）；`CountOnly` 时是空的。
+    observed_states: Vec<(Vec<bool>, RecoveryReport)>,
+}
+
+/// 在工作线程上跑一片：每个状态自己建崩溃镜像、自己记进这一片的计数；基线与写表只读、各线程共用。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "每个参数各是一样东西：基线、写表、状态计划、这一片的区间、被判的根、版本表、要不要带回报告"
+)]
+fn evaluate_state_slice(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+    plan: &Layer0StatePlan<'_>,
+    slice_index: usize,
+    slice: Range<u64>,
+    judged_root_index: usize,
+    versions: &[PublishedVersion],
+    retention: StateReportRetention,
+) -> FinishedSlice {
+    let mut tally = Layer0Tally::default();
+    let mut observed_states = Vec::new();
+    for ordinal in slice {
+        let persisted = plan.persisted_writes_of_state(ordinal, writes.len());
+        match retention {
+            StateReportRetention::HandEachStateToObserver => {
+                let consulted_report = evaluate_state_for_versions(
+                    base,
+                    writes,
+                    persisted.clone(),
+                    judged_root_index,
+                    versions,
+                    &mut tally,
+                );
+                observed_states.push((persisted, consulted_report));
+            }
+            StateReportRetention::CountOnly => {
+                evaluate_state_for_versions(
+                    base,
+                    writes,
+                    persisted,
+                    judged_root_index,
+                    versions,
+                    &mut tally,
+                );
+            }
+        }
+    }
+    FinishedSlice {
+        slice_index,
+        tally,
+        observed_states,
+    }
+}
+
 /// 枚举：前面的段全持久 + 当前段任意真子集，最后再加全部持久那一个状态；`expand` 决定哪一段展开子集
 /// （不展开的段只以整段持久进入后面的状态，平时 `cargo test` 里跳过 18 个写那一段就靠它）。`judged_root_index` 是被判的那次根槽 FUA 写。
+/// 按状态序号区间切片、多线程跑，线程数见 [`Layer0Parallelism::from_environment`]。
 #[must_use]
 pub fn enumerate_layer0_selecting_versions(
     base: &MemoryPool,
@@ -822,19 +1171,21 @@ pub fn enumerate_layer0_selecting_versions(
     versions: &[PublishedVersion],
     expand: &dyn Fn(usize, &[usize]) -> bool,
 ) -> Layer0Tally {
-    enumerate_layer0_selecting_versions_observing_each_state(
+    enumerate_layer0_in_state_slices(
         base,
         writes,
         segments,
         judged_root_index,
         versions,
         expand,
-        &mut |_crash_image, _consulted_report| {},
+        Layer0Parallelism::from_environment(),
+        None,
     )
 }
 
 /// 同上，每个状态评完之后把这个崩溃镜像与看 journal 那一遍恢复的报告交给 `observe_state`：
 /// 用例按自己独立算的谓词逐状态核恢复（预置的残留记录该不该施加、改坏 tail 之后终态与没改坏的是否逐项相等）。
+/// 观察者在调用线程上、按状态序号从小到大调用，次序与单线程逐个跑相同，所以它不必能跨线程。
 #[must_use]
 pub fn enumerate_layer0_selecting_versions_observing_each_state(
     base: &MemoryPool,
@@ -845,55 +1196,148 @@ pub fn enumerate_layer0_selecting_versions_observing_each_state(
     expand: &dyn Fn(usize, &[usize]) -> bool,
     observe_state: &mut dyn FnMut(&CrashImage<'_>, &RecoveryReport),
 ) -> Layer0Tally {
-    let mut tally = Layer0Tally::default();
-    let mut persisted_before = vec![false; writes.len()];
-    for (segment_index, segment) in segments.iter().enumerate() {
-        if expand(segment_index, segment) {
-            let full_mask = (1u64 << segment.len()) - 1;
-            for mask in 0..full_mask {
-                let mut persisted = persisted_before.clone();
-                for (bit, write_index) in segment.iter().enumerate() {
-                    if mask & (1 << bit) != 0 {
-                        persisted[*write_index] = true;
-                    }
+    enumerate_layer0_in_state_slices(
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+        expand,
+        Layer0Parallelism::from_environment(),
+        Some(observe_state),
+    )
+}
+
+/// 层 0 枚举的本体：把 [0, 状态数) 按 `parallelism` 切成首尾相接的序号区间，工作线程按片号从小到大领片、各自跑完交回；
+/// 调用线程收到一片就打一行 `LAYER0_PROGRESS`（片号、序号区间、段号、已跑完的片数与状态数），再按片号从小到大并计数、调观察者。
+/// 计数按片的次序相加，「第一处违例」取序号最小的那一处（`Layer0Tally` 的并片），结果与线程数、切法无关。
+/// 开跑与跑完各打一行 `LAYER0_PARALLEL_START` / `LAYER0_PARALLEL_FINISHED`（状态数、片数、实际起的工作线程数、线程数从哪来、耗时）。
+///
+/// # Panics
+/// 某个工作线程在评状态时 panic（恢复或 checker 里的断言：别的线程不再领新片，手上那一片跑完就退）；观察者 panic；
+/// 有一片领了却没交回（不变量被破坏）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "前六个与单线程时的枚举相同，多出来的是切法与观察者"
+)]
+#[must_use]
+pub fn enumerate_layer0_in_state_slices(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+    segments: &[Vec<usize>],
+    judged_root_index: usize,
+    versions: &[PublishedVersion],
+    expand: &dyn Fn(usize, &[usize]) -> bool,
+    parallelism: Layer0Parallelism,
+    mut observe_state: Option<Layer0StateObserver<'_>>,
+) -> Layer0Tally {
+    let plan = Layer0StatePlan::new(segments, expand);
+    let slices = state_slices(plan.state_count, &parallelism);
+    let spawned_worker_threads = parallelism.worker_threads.get().min(slices.len());
+    let retention = match observe_state {
+        Some(_) => StateReportRetention::HandEachStateToObserver,
+        None => StateReportRetention::CountOnly,
+    };
+    let started = Instant::now();
+    println!(
+        "LAYER0_PARALLEL_START states={} slices={} states_per_slice={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={}",
+        plan.state_count,
+        slices.len(),
+        slices.first().map_or(0, |slice| slice.end - slice.start),
+        parallelism.worker_threads,
+        parallelism.worker_threads_source.name()
+    );
+    let next_slice_index = AtomicUsize::new(0);
+    let some_worker_thread_panicked = AtomicBool::new(false);
+    let (tally, merged_slice_count) = std::thread::scope(|scope| {
+        // 收发两端都建在这个闭包里：观察者 panic 时接收端随闭包一起丢掉，工作线程下一次交片就发不出去、随即退出。
+        let (finished_slice_sender, finished_slice_receiver) = mpsc::channel::<FinishedSlice>();
+        for _ in 0..spawned_worker_threads {
+            let finished_slice_sender = finished_slice_sender.clone();
+            let plan = &plan;
+            let slices = &slices;
+            let next_slice_index = &next_slice_index;
+            let some_worker_thread_panicked = &some_worker_thread_panicked;
+            scope.spawn(move || loop {
+                let _raise_the_flag_if_this_thread_panics =
+                    RaiseFlagWhenPanicking(some_worker_thread_panicked);
+                if some_worker_thread_panicked.load(Ordering::Relaxed) {
+                    break;
                 }
-                let consulted_report = evaluate_state_for_versions(
+                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
+                let Some(slice) = slices.get(slice_index) else {
+                    break;
+                };
+                let finished = evaluate_state_slice(
                     base,
                     writes,
-                    persisted.clone(),
+                    plan,
+                    slice_index,
+                    slice.clone(),
                     judged_root_index,
                     versions,
-                    &mut tally,
-                );
-                observe_state(
-                    &CrashImage {
-                        base,
-                        writes,
-                        persisted,
-                    },
-                    &consulted_report,
+                    retention,
                 );
-            }
+                // 发不出去说明调用线程已经不收了（观察者 panic）：不再领新的片。
+                if finished_slice_sender.send(finished).is_err() {
+                    break;
+                }
+            });
         }
-        for write_index in segment {
-            persisted_before[*write_index] = true;
+        // 只留工作线程手里的发送端：它们都退出之后，下面的接收循环才结束。
+        drop(finished_slice_sender);
+        let mut merged = Layer0Tally::default();
+        let mut waiting_for_earlier_slices: BTreeMap<usize, FinishedSlice> = BTreeMap::new();
+        let mut next_slice_to_merge = 0usize;
+        let mut finished_states = 0u64;
+        for (finished_slice_count, finished) in finished_slice_receiver.iter().enumerate() {
+            let slice = &slices[finished.slice_index];
+            finished_states += slice.end - slice.start;
+            println!(
+                "LAYER0_PROGRESS slice={}/{} states=[{},{}) segments={}..={} finished_slices={}/{} finished_states={finished_states}/{} elapsed_seconds={:.1}",
+                finished.slice_index + 1,
+                slices.len(),
+                slice.start,
+                slice.end,
+                plan.segment_label(plan.segment_of_state(slice.start)),
+                plan.segment_label(plan.segment_of_state(slice.end - 1)),
+                finished_slice_count + 1,
+                slices.len(),
+                plan.state_count,
+                started.elapsed().as_secs_f64()
+            );
+            waiting_for_earlier_slices.insert(finished.slice_index, finished);
+            while let Some(in_order) = waiting_for_earlier_slices.remove(&next_slice_to_merge) {
+                if let Some(observe) = observe_state.as_mut() {
+                    for (persisted, consulted_report) in in_order.observed_states {
+                        observe(
+                            &CrashImage {
+                                base,
+                                writes,
+                                persisted,
+                            },
+                            &consulted_report,
+                        );
+                    }
+                }
+                merged.absorb_following_slice(in_order.tally);
+                next_slice_to_merge += 1;
+            }
         }
-    }
-    let consulted_report = evaluate_state_for_versions(
-        base,
-        writes,
-        persisted_before.clone(),
-        judged_root_index,
-        versions,
-        &mut tally,
+        (merged, next_slice_to_merge)
+    });
+    assert_eq!(
+        merged_slice_count,
+        slices.len(),
+        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
     );
-    observe_state(
-        &CrashImage {
-            base,
-            writes,
-            persisted: persisted_before,
-        },
-        &consulted_report,
+    println!(
+        "LAYER0_PARALLEL_FINISHED states={} slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} elapsed_seconds={:.1}",
+        plan.state_count,
+        slices.len(),
+        parallelism.worker_threads,
+        parallelism.worker_threads_source.name(),
+        started.elapsed().as_secs_f64()
     );
     tally
 }
@@ -965,6 +1409,280 @@ mod tests {
             .all(|(index, byte)| index == 600 || *byte == 1));
         assert_eq!(closed_form_state_count(&[vec![0, 1], vec![2]]), 1 + 3 + 1);
     }
+
+    /// 单线程逐段逐个子集走（并行之前 `enumerate_layer0_selecting_versions_observing_each_state` 的次序）给出的持久集合，按次序排好。
+    fn persisted_sets_walking_segment_by_segment(
+        segments: &[Vec<usize>],
+        write_count: usize,
+        expand: &dyn Fn(usize, &[usize]) -> bool,
+    ) -> Vec<Vec<bool>> {
+        let mut persisted_sets = Vec::new();
+        let mut persisted_before = vec![false; write_count];
+        for (segment_index, segment) in segments.iter().enumerate() {
+            if expand(segment_index, segment) {
+                for subset_mask in 0..(1u64 << segment.len()) - 1 {
+                    let mut persisted = persisted_before.clone();
+                    for (bit, write_index) in segment.iter().enumerate() {
+                        if subset_mask & (1 << bit) != 0 {
+                            persisted[*write_index] = true;
+                        }
+                    }
+                    persisted_sets.push(persisted);
+                }
+            }
+            for write_index in segment {
+                persisted_before[*write_index] = true;
+            }
+        }
+        persisted_sets.push(persisted_before);
+        persisted_sets
+    }
+
+    /// 按序号取状态（并行切片靠它）与逐段逐个子集走，给出同一串持久集合：展开的段夹着不展开的段、不展开的段在头上和尾上都算。
+    #[test]
+    fn the_state_plan_hands_out_the_same_persisted_sets_in_the_same_order_as_walking_segment_by_segment(
+    ) {
+        let segments = vec![
+            vec![0, 1],
+            vec![2],
+            vec![3, 4, 5],
+            vec![6, 7],
+            vec![8, 9, 10, 11],
+            vec![12],
+        ];
+        let write_count = 13;
+        let every_segment = |_segment_index: usize, _segment: &[usize]| true;
+        let skip_three_write_segment_and_the_ends = |segment_index: usize, segment: &[usize]| {
+            segment.len() != 3 && segment_index != 0 && segment_index != 5
+        };
+        let only_the_four_write_segment =
+            |_segment_index: usize, segment: &[usize]| segment.len() == 4;
+        let assert_plan_matches_the_walk = |expand: &dyn Fn(usize, &[usize]) -> bool| {
+            let walked = persisted_sets_walking_segment_by_segment(&segments, write_count, expand);
+            let plan = Layer0StatePlan::new(&segments, expand);
+            assert_eq!(
+                plan.state_count,
+                u64::try_from(walked.len()).expect("状态数"),
+                "状态数与逐段走的相同"
+            );
+            let by_ordinal: Vec<Vec<bool>> = (0..plan.state_count)
+                .map(|ordinal| plan.persisted_writes_of_state(ordinal, write_count))
+                .collect();
+            assert_eq!(by_ordinal, walked, "第 k 个状态就是逐段走到的第 k 个");
+        };
+        assert_plan_matches_the_walk(&every_segment);
+        assert_plan_matches_the_walk(&skip_three_write_segment_and_the_ends);
+        assert_plan_matches_the_walk(&only_the_four_write_segment);
+    }
+
+    /// 切片首尾相接、从 0 起、到状态数止、一片都不空：丢一片或两片重叠，这里与用例里「状态数等于闭式」的断言都红。
+    /// 默认切法下全量两条流切出来的片数不少于线程数（每个线程都领得到片）。
+    #[test]
+    fn state_slices_cover_every_state_exactly_once_in_ordinal_order() {
+        let thread_counts = [1usize, 3, 32, 200];
+        let fixed_lengths = [1u64, 7, 16, u64::MAX];
+        for state_count in [1u64, 2, 15, 16, 17, 22, 108, 1000, 262_165, 2_104_413] {
+            let mut parallelisms: Vec<Layer0Parallelism> = thread_counts
+                .iter()
+                .map(|worker_threads| Layer0Parallelism {
+                    worker_threads: NonZeroUsize::new(*worker_threads).expect("非 0"),
+                    worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
+                    slice_length: Layer0SliceLength::ScaledToWorkerThreads,
+                })
+                .collect();
+            parallelisms.extend(
+                fixed_lengths
+                    .iter()
+                    .map(|states_per_slice| Layer0Parallelism {
+                        worker_threads: NonZeroUsize::MIN,
+                        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
+                        slice_length: Layer0SliceLength::StatesPerSlice(
+                            NonZeroU64::new(*states_per_slice).expect("非 0"),
+                        ),
+                    }),
+            );
+            for parallelism in parallelisms {
+                let slices = state_slices(state_count, &parallelism);
+                let mut next_ordinal = 0u64;
+                for slice in &slices {
+                    assert_eq!(
+                        slice.start, next_ordinal,
+                        "{state_count} 个状态、{parallelism:?}：片首接上一片的尾"
+                    );
+                    assert!(slice.end > slice.start, "{parallelism:?}：没有空片");
+                    next_ordinal = slice.end;
+                }
+                assert_eq!(
+                    next_ordinal, state_count,
+                    "{parallelism:?}：最后一片止于状态数"
+                );
+                if state_count >= 262_165 {
+                    assert!(
+                        slices.len() >= parallelism.worker_threads.get()
+                            || matches!(
+                                parallelism.slice_length,
+                                Layer0SliceLength::StatesPerSlice(_)
+                            ),
+                        "{state_count} 个状态、{parallelism:?}：片数 {} 不少于线程数",
+                        slices.len()
+                    );
+                }
+            }
+        }
+    }
+
+    /// 线程数：环境变量设了就用它（不再问 `available_parallelism`）；没设取 `available_parallelism`；那个也报不出来只用 1 个线程、照实报来源。
+    #[test]
+    fn worker_threads_come_from_the_environment_variable_before_available_parallelism() {
+        let thirty_two = || Ok(NonZeroUsize::new(32).expect("非 0"));
+        let from_variable = Layer0Parallelism::from_environment_value(Ok("4".to_string()), || {
+            panic!("环境变量设了就不该再问 available_parallelism")
+        });
+        assert_eq!(
+            (
+                from_variable.worker_threads.get(),
+                from_variable.worker_threads_source
+            ),
+            (4, Layer0WorkerThreadsSource::EnvironmentVariable)
+        );
+        let explicit_single =
+            Layer0Parallelism::from_environment_value(Ok("1".to_string()), thirty_two);
+        assert_eq!(
+            (
+                explicit_single.worker_threads.get(),
+                explicit_single.worker_threads_source
+            ),
+            (1, Layer0WorkerThreadsSource::EnvironmentVariable),
+            "显式设成 1 就是 1"
+        );
+        let unset = Layer0Parallelism::from_environment_value(
+            Err(std::env::VarError::NotPresent),
+            thirty_two,
+        );
+        assert_eq!(
+            (unset.worker_threads.get(), unset.worker_threads_source),
+            (32, Layer0WorkerThreadsSource::AvailableParallelism)
+        );
+        let unknown =
+            Layer0Parallelism::from_environment_value(Err(std::env::VarError::NotPresent), || {
+                Err(std::io::Error::other("平台报不出核数"))
+            });
+        assert_eq!(
+            (unknown.worker_threads.get(), unknown.worker_threads_source),
+            (1, Layer0WorkerThreadsSource::AvailableParallelismUnknown)
+        );
+        assert_eq!(unset.slice_length, Layer0SliceLength::ScaledToWorkerThreads);
+    }
+
+    /// 环境变量设成 0：停下，不悄悄退回单线程。不写成 `#[should_panic]`：那样 libtest 的输出行带「- should panic」，门禁 59 号认不出这条测试红没红。
+    #[test]
+    fn zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back() {
+        let outcome = std::panic::catch_unwind(|| {
+            Layer0Parallelism::from_environment_value(Ok("0".to_string()), || {
+                Ok(NonZeroUsize::new(32).expect("非 0"))
+            })
+        });
+        let panic_payload = outcome.expect_err("设成 0 要停下，不许退回 1 个线程接着跑");
+        let panic_message = panic_payload
+            .downcast_ref::<String>()
+            .cloned()
+            .unwrap_or_default();
+        assert!(
+            panic_message.contains("SINGLEFS_LAYER0_THREADS 要是正整数"),
+            "停下时说清是哪个环境变量配错了：{panic_message}"
+        );
+    }
+
+    /// 并片：计数相加，「第一处」取前面那一片的；前面那一片没有才取后面的。
+    #[test]
+    fn absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation() {
+        let mut earlier = Layer0Tally {
+            states: 3,
+            violations: 1,
+            root_persisted_states: 0,
+            no_file_states: 3,
+            file_read_states: 0,
+            failed_states: 0,
+            journal_differing_states: 0,
+            verification_ran_states: 0,
+            verification_failed_states: 0,
+            first_violation: Some("前面那一片的".to_string()),
+            ignored_violations: 0,
+            first_ignored_violation: None,
+            record_root_without_record: 0,
+            record_claimed_state_missing_unit: 0,
+            checker_evaluated_states: BTreeMap::from([("I-1.1", 3)]),
+            checker_violated_states: BTreeMap::from([("I-1.1", 1)]),
+            checker_first_violation: BTreeMap::from([("I-1.1", "前面那一片的 I-1.1".to_string())]),
+            checker_not_applicable_states: BTreeMap::from([("I-3.1", 3)]),
+        };
+        let following = Layer0Tally {
+            states: 5,
+            violations: 2,
+            root_persisted_states: 5,
+            no_file_states: 0,
+            file_read_states: 5,
+            failed_states: 0,
+            journal_differing_states: 1,
+            verification_ran_states: 1,
+            verification_failed_states: 0,
+            first_violation: Some("后面那一片的".to_string()),
+            ignored_violations: 1,
+            first_ignored_violation: Some("后面那一片的 Ignore".to_string()),
+            record_root_without_record: 1,
+            record_claimed_state_missing_unit: 1,
+            checker_evaluated_states: BTreeMap::from([("I-1.1", 5), ("I-3.1", 5)]),
+            checker_violated_states: BTreeMap::from([("I-1.1", 2), ("I-3.1", 1)]),
+            checker_first_violation: BTreeMap::from([
+                ("I-1.1", "后面那一片的 I-1.1".to_string()),
+                ("I-3.1", "后面那一片的 I-3.1".to_string()),
+            ]),
+            checker_not_applicable_states: BTreeMap::new(),
+        };
+        earlier.absorb_following_slice(following);
+        assert_eq!(
+            (
+                earlier.states,
+                earlier.violations,
+                earlier.ignored_violations,
+                earlier.no_file_states,
+                earlier.file_read_states,
+                earlier.root_persisted_states,
+                earlier.journal_differing_states,
+                earlier.verification_ran_states,
+                earlier.record_root_without_record,
+                earlier.record_claimed_state_missing_unit,
+            ),
+            (8, 3, 1, 3, 5, 5, 1, 1, 1, 1),
+            "计数逐项相加"
+        );
+        assert_eq!(earlier.first_violation.as_deref(), Some("前面那一片的"));
+        assert_eq!(
+            earlier.first_ignored_violation.as_deref(),
+            Some("后面那一片的 Ignore"),
+            "前面那一片没有，取后面的"
+        );
+        assert_eq!(
+            earlier.checker_evaluated_states,
+            BTreeMap::from([("I-1.1", 8), ("I-3.1", 5)])
+        );
+        assert_eq!(
+            earlier.checker_violated_states,
+            BTreeMap::from([("I-1.1", 3), ("I-3.1", 1)])
+        );
+        assert_eq!(
+            earlier.checker_not_applicable_states,
+            BTreeMap::from([("I-3.1", 3)])
+        );
+        assert_eq!(
+            earlier.checker_first_violation,
+            BTreeMap::from([
+                ("I-1.1", "前面那一片的 I-1.1".to_string()),
+                ("I-3.1", "后面那一片的 I-3.1".to_string()),
+            ]),
+            "每条不变量的第一处各取最早有的那一片"
+        );
+    }
 }
 
 #[cfg(test)]
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
index f85018d..46268c4 100644
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -17,6 +17,7 @@ pub mod crash;
 pub mod device_log;
 pub mod first_transaction_regions;
 pub mod hexadecimal;
+pub mod history;
 pub mod scenario;
 pub mod segments;
 pub mod sha256;
@@ -122,6 +123,11 @@ impl SharedStream {
     pub fn operations(&self) -> Vec<RecordedOperation> {
         self.0.borrow().operations.clone()
     }
+    /// 流里已有几步（不拷整条流）：随机历史拿它判「这一步发没发写」。
+    #[must_use]
+    pub fn operation_count(&self) -> usize {
+        self.0.borrow().operations.len()
+    }
     /// 每一步连同它的内容；没开内容保留的流里 `contents` 全是 None。
     #[must_use]
     pub fn retained_operations(&self) -> Vec<RetainedOperation> {
diff --git a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs b/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
index 03232b9..dca2536 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
@@ -6,6 +6,8 @@
 
 mod common;
 
+use std::num::{NonZeroU64, NonZeroUsize};
+
 use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration, SlotNumber};
 use singlefs_core::block_device::{BlockDevice, WriteDurability};
@@ -15,16 +17,18 @@ use singlefs_core::mount::{
 };
 use singlefs_core::recovery::{
     choose_superblock, recover, scan_journal, JournalPolicy, PoolReader, RecoveryOutcome,
+    RecoveryReport,
 };
 use singlefs_core::superblock::Superblock;
 use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
 use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
 use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};
 use singlefs_harness::crash::{
-    closed_form_state_count, enumerate_layer0_selecting_versions,
+    closed_form_state_count, enumerate_layer0_in_state_slices, enumerate_layer0_selecting_versions,
     enumerate_layer0_selecting_versions_observing_each_state, enumerate_layer0_versions,
-    evaluate_state_for_versions, writes_and_segments, CrashImage, Layer0Tally, MemoryPool,
-    PublishedVersion, RetainedWrite,
+    evaluate_state_for_versions, writes_and_segments, CrashImage, Layer0Parallelism,
+    Layer0SliceLength, Layer0Tally, Layer0WorkerThreadsSource, MemoryPool, PublishedVersion,
+    RetainedWrite,
 };
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::RetainedOperation;
@@ -468,6 +472,67 @@ fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_r
     assert_checker_and_record_checker_clean(&tally);
 }
 
+/// 层 0 按状态序号区间切片、多线程跑（2026-09-18 用户定：测试与崩溃检测优先多线程）：到 E 的固定脚本、平时跑的那 108 个状态，
+/// 切成每片 1 个状态、8 个线程抢着跑，与整条流 1 片、1 个线程逐个跑相比，计数逐项相同、「第一处违例」是序号最小的那一处、
+/// 观察者按同一个次序看到同一串持久集合。版本表故意把 B（实例 1 第 4 代）的内容换成 C 的，让违例散在很多个状态上，「第一处」才有得选。
+#[test]
+fn one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread(
+) {
+    let prepared = prepare("layer0-slices-merge", Script::ReuseAfterRaisingFloor);
+    let mut versions_with_the_wrong_second_content = prepared.versions.clone();
+    versions_with_the_wrong_second_content
+        .iter_mut()
+        .find(|version| {
+            version.instance == InstanceGeneration(1) && version.checkpoint_txg == CheckpointTxg(4)
+        })
+        .expect("版本表里有 B 那一版")
+        .content = third_content();
+    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
+    let run = |parallelism: Layer0Parallelism| {
+        let mut observed_persisted_sets: Vec<Vec<bool>> = Vec::new();
+        let tally = enumerate_layer0_in_state_slices(
+            &prepared.base,
+            &prepared.writes,
+            &prepared.segments,
+            prepared.judged_root_index,
+            &versions_with_the_wrong_second_content,
+            &expand,
+            parallelism,
+            Some(
+                &mut |crash_image: &CrashImage<'_>, _consulted_report: &RecoveryReport| {
+                    observed_persisted_sets.push(crash_image.persisted.clone());
+                },
+            ),
+        );
+        (tally, observed_persisted_sets)
+    };
+    let (one_slice_tally, one_slice_order) = run(Layer0Parallelism {
+        worker_threads: NonZeroUsize::MIN,
+        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
+        slice_length: Layer0SliceLength::StatesPerSlice(NonZeroU64::MAX),
+    });
+    let (one_state_slices_tally, one_state_slices_order) = run(Layer0Parallelism {
+        worker_threads: NonZeroUsize::new(8).expect("8 不是 0"),
+        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
+        slice_length: Layer0SliceLength::StatesPerSlice(NonZeroU64::MIN),
+    });
+    assert_eq!(one_slice_tally.states, 108, "平时跑的那 108 个状态");
+    assert!(
+        one_slice_tally.violations >= 2 && one_slice_tally.ignored_violations >= 2,
+        "B 的内容换掉之后违例散在多个状态上：{} 个、不看 journal 那一遍 {} 个",
+        one_slice_tally.violations,
+        one_slice_tally.ignored_violations
+    );
+    assert_eq!(
+        one_state_slices_order, one_slice_order,
+        "观察者按序号次序看到同一串持久集合"
+    );
+    assert_eq!(
+        one_state_slices_tally, one_slice_tally,
+        "计数与每一处「第一处」都逐项相同"
+    );
+}
+
 /// 全量：固定脚本到 E 为止 54 段、闭式 2 104 413 个状态（八个 18 写段各 262143、七个 10 写段各 1023、两个 4 写段各 15，其余 2 写段各 3、1 写段各 1）。
 /// 54 号门禁在 release 下跑它，认下面打印的 `LAYER0B` 行里 `exhaustive=true`。
 #[test]
```

## 二、新文件 `crates/singlefs-harness/src/history.rs`（未进 git）与 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（未进 git）：按项抽取（`quote-rust-items.py`，20/20 项；回读逐字节比对，退出码 0）

按主 agent 给的「文件::项名」清单逐项抽取，命令与命中如下（合计 20 个 spec，一次调用）：

```
python3 research/scripts/quote-rust-items.py \
  'crates/singlefs-harness/src/history.rs::HarnessJudgement' \
  'crates/singlefs-harness/src/history.rs::impl HarnessJudgement' \
  'crates/singlefs-harness/src/history.rs::harness_judgement_of_outcome' \
  'crates/singlefs-harness/src/history.rs::ColdStartReadBack' \
  'crates/singlefs-harness/src/history.rs::AppliedEffect' \
  'crates/singlefs-harness/src/history.rs::FailureObservation' \
  'crates/singlefs-harness/src/history.rs::impl FailureObservation' \
  'crates/singlefs-harness/src/history.rs::only_allocated_statistic_above_walked' \
  'crates/singlefs-harness/src/history.rs::ring_turn_leaves_allocated_statistic_above_walked' \
  'crates/singlefs-harness/src/history.rs::raise_after_rollback_leaves_allocated_statistic_above_walked' \
  'crates/singlefs-harness/src/history.rs::KNOWN_RED_FORMS' \
  'crates/singlefs-harness/src/history.rs::allocation_generation_judgement' \
  'crates/singlefs-harness/src/history.rs::apply_cold_start_recover' \
  'crates/singlefs-harness/src/history.rs::raised_floor_lands_only_on_abandoned_roots' \
  'crates/singlefs-harness/src/history.rs::GenerationWeights' \
  'crates/singlefs-harness/src/history.rs::impl GenerationWeights' \
  'crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs::assert_every_path_was_exercised' \
  'crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs::random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation' \
  'crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs::reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms' \
  'crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs::an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding'
```

`struct GenerationWeights` 与它的 `impl` 块（含 `REUSE_AFTER_RAISING_THE_FLOOR`，落在 285-360 行区间内）分两条 spec 各自取全；`raised_floor_lands_only_on_abandoned_roots` 抽出的区间是 2024-2055，函数签名本身在第 2030 行，与主 agent 给的行号一致。以下逐项贴出：

### crates/singlefs-harness/src/history.rs:703-714（HarnessJudgement）

```rust
/// 执行器自己判的失败：不是 checker 的不变量、不是 panic，是入口交回的东西或结局自己就说明错了。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HarnessJudgement {
    /// 这一步发布改写或新增的分配记录，代不是这次发布的 txg（`records_changed_with_a_generation_outside_the_publish_txgs`）。
    AllocationGenerationIsNotThePublishTxg {
        records: Vec<AllocationRecord>,
        first_publish_txg: CheckpointTxg,
        last_publish_txg: CheckpointTxg,
    },
    /// 冷启动恢复报错，而这一步之前的镜像 checker 判过绿（判红的话历史已经停了；冷启动不写盘，镜像没变）。
    ColdStartRecoveryFailedOnCheckerGreenImage { outcome: String },
}
```

### crates/singlefs-harness/src/history.rs:716-729（impl HarnessJudgement）

```rust
impl HarnessJudgement {
    /// 签名与报告里的名字。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            HarnessJudgement::AllocationGenerationIsNotThePublishTxg { .. } => {
                "改写或新增的分配记录的代不是这次发布的 txg"
            }
            HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage { .. } => {
                "checker 判绿的镜像上冷启动恢复报错"
            }
        }
    }
}
```

### crates/singlefs-harness/src/history.rs:731-751（harness_judgement_of_outcome）

```rust
/// 一步操作的结局本身判不判失败：冷启动读回报错就算（不看读回的内容对不对，那是第 2 件模型的事）。
#[must_use]
pub fn harness_judgement_of_outcome(outcome: &StepOutcome) -> Option<HarnessJudgement> {
    match outcome {
        StepOutcome::Applied(AppliedEffect::Recovered { outcome, read_back }) => match read_back {
            ColdStartReadBack::Failed => Some(
                HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage {
                    outcome: outcome.clone(),
                },
            ),
            ColdStartReadBack::NoFile | ColdStartReadBack::FileRead => None,
        },
        StepOutcome::Applied(
            AppliedEffect::Published { .. }
            | AppliedEffect::Mounted { .. }
            | AppliedEffect::RaisedFloor { .. },
        )
        | StepOutcome::Refused { .. }
        | StepOutcome::NotApplicable(_) => None,
    }
}
```

### crates/singlefs-harness/src/history.rs:753-759（ColdStartReadBack）

```rust
/// 冷启动读回的三种结局（`recovery::RecoveryOutcome` 的成员，不带内容）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColdStartReadBack {
    NoFile,
    FileRead,
    Failed,
}
```

### crates/singlefs-harness/src/history.rs:761-781（AppliedEffect）

```rust
/// 入口返回 Ok 之后这一步做成了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppliedEffect {
    Published {
        reuse: RecordReuse,
    },
    Mounted {
        instance: InstanceGeneration,
        publishes: usize,
    },
    RaisedFloor {
        new_floor: CheckpointTxg,
        publishes: usize,
        reclaimed_placements: usize,
        reuse: RecordReuse,
    },
    Recovered {
        outcome: String,
        read_back: ColdStartReadBack,
    },
}
```

### crates/singlefs-harness/src/history.rs:864-881（FailureObservation）

```rust
/// 一次失败（违例或 panic）连同它发生时的盘面事实，拿去对「已知红」清单。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureObservation {
    pub position: StepPosition,
    pub operation_kind: Option<HistoryOperationKind>,
    /// checker 判红的不变量与它报的第一处。
    pub violations: Vec<(&'static str, String)>,
    pub panic: Option<CapturedPanic>,
    /// 按 checker 的读法从盘上读：根环里最新那条根的 txg；读不到时 None。
    pub newest_ring_root_txg: Option<u64>,
    /// 根环一圈的槽数 R × S（按 checker 从超级块读出的几何）；读不到时 None。
    pub root_ring_slot_count: Option<u64>,
    /// 执行器自己判出的失败（分配代、冷启动读回）。
    pub harness_judgement: Option<HarnessJudgement>,
    /// 抬 F 那一步（入口返回 Ok）之后判红时：抬之前的镜像上，新 F 那个 txg 上的根是不是全属于被抛弃的实例
    /// （`raised_floor_lands_only_on_abandoned_roots`）；别的步、读不出、那个 txg 上没有根，都是 None。
    pub raised_floor_lands_only_on_abandoned_roots: Option<bool>,
}
```

### crates/singlefs-harness/src/history.rs:883-889（impl FailureObservation）

```rust
impl FailureObservation {
    /// 根环转过一圈：盘上最新根的 txg ≥ R × S，第 0 代根的槽已被盖过。
    #[must_use]
    pub fn root_ring_has_turned(&self) -> bool {
        root_ring_has_turned(self.newest_ring_root_txg, self.root_ring_slot_count)
    }
}
```

### crates/singlefs-harness/src/history.rs:912-922（only_allocated_statistic_above_walked）

```rust
/// 没有 panic、执行器没判出失败、判红的只有 I-3.1、而且是记账的已分配大于遍历全部有效根得到的（记账多算，不是少算）。
fn only_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
    observation.panic.is_none()
        && observation.harness_judgement.is_none()
        && !observation.violations.is_empty()
        && observation.violations.iter().all(|(invariant, detail)| {
            *invariant == "I-3.1"
                && allocated_and_walked_bytes(detail)
                    .is_some_and(|(allocated, walked)| allocated > walked)
        })
}
```

### crates/singlefs-harness/src/history.rs:924-926（ring_turn_leaves_allocated_statistic_above_walked）

```rust
fn ring_turn_leaves_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
    observation.root_ring_has_turned() && only_allocated_statistic_above_walked(observation)
}
```

### crates/singlefs-harness/src/history.rs:928-937（raise_after_rollback_leaves_allocated_statistic_above_walked）

```rust
/// 抬 F 那一步之后、抬之前的镜像上新 F 那个 txg 上的根全属于被抛弃的实例（F 落在回退留下的空档里）、根环没转圈、只有 I-3.1 红且
/// 记账多于遍历（代码三方第一轮判决第二节第 1 条收窄：此前不看空档，不经回退的抬 F 之后记账多算也被接走）。
fn raise_after_rollback_leaves_allocated_statistic_above_walked(
    observation: &FailureObservation,
) -> bool {
    observation.operation_kind == Some(HistoryOperationKind::RaiseRollbackFloor)
        && observation.raised_floor_lands_only_on_abandoned_roots == Some(true)
        && !observation.root_ring_has_turned()
        && only_allocated_statistic_above_walked(observation)
}
```

### crates/singlefs-harness/src/history.rs:939-941（KNOWN_RED_FORMS）

```rust
/// 「已知红」清单。修好一条就删一条，删掉之后那条的复现（`tests/second_transaction_supplement_three_random_history.rs` 里钉着）要转绿。
/// 第 0 条的宽度（转圈跨几次挂载也算）2026-09-18 主 agent 定案保留，记在收口表第 ② 行。
pub const KNOWN_RED_FORMS: [KnownRedForm; 2] = [
```

### crates/singlefs-harness/src/history.rs:1435-1453（allocation_generation_judgement）

```rust
/// 一次发布前后的分配记录：改写或新增的记录的代要在这次（几次）发布的 txg 里。
fn allocation_generation_judgement(
    before: &[AllocationRecord],
    after: &[AllocationRecord],
    first_publish_txg: CheckpointTxg,
    last_publish_txg: CheckpointTxg,
) -> Option<HarnessJudgement> {
    let records = records_changed_with_a_generation_outside_the_publish_txgs(
        before,
        after,
        first_publish_txg,
        last_publish_txg,
    );
    (!records.is_empty()).then_some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
        records,
        first_publish_txg,
        last_publish_txg,
    })
}
```

### crates/singlefs-harness/src/history.rs:1752-1764（apply_cold_start_recover）

```rust
fn apply_cold_start_recover(pool: &mut HistoryPool) -> StepOutcome {
    pool.session = None;
    let report = recover(&pool.devices, JournalPolicy::Consult);
    let read_back = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => ColdStartReadBack::NoFile,
        RecoveryOutcome::FileRead { .. } => ColdStartReadBack::FileRead,
        RecoveryOutcome::Failed { .. } => ColdStartReadBack::Failed,
    };
    StepOutcome::Applied(AppliedEffect::Recovered {
        outcome: recovery_outcome_member(&report.outcome),
        read_back,
    })
}
```

### crates/singlefs-harness/src/history.rs:2024-2055（raised_floor_lands_only_on_abandoned_roots）

```rust
/// 抬 F 之前的镜像上，txg = `new_floor` 的根是不是全属于被抛弃的实例（F 落在回退留下的空档里）：按最新根指着的实例表，
/// 有行 (i, T) 且根的 txg > T 的 i 就是被抛弃的（与 `mount::mount_rollback` 判候选集同一句）。那个 txg 上一条根都没有、
/// 超级块或最新根或它的实例表读不出，都是 None（不算落在空档里）。
/// 读法用的是实现的 `recovery::readable_roots` / `choose_root` / `instance_table_of_root`（代码三方第一轮攻方的 P1 就这么读）：
/// checker 判候选集时解实例表的那一段不对外（`singlefs-checker` 的 `walk.rs` 里 `Walk::instance_table_rows`），这里没另写一份解析。
#[must_use]
pub fn raised_floor_lands_only_on_abandoned_roots(
    image_before_raising: &MemoryPool,
    new_floor: CheckpointTxg,
) -> Option<bool> {
    let superblock = choose_superblock(image_before_raising).ok()?;
    let roots_at_floor: Vec<RootRecord> = readable_roots(
        image_before_raising,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| root.checkpoint_txg == new_floor)
    .collect();
    if roots_at_floor.is_empty() {
        return None;
    }
    let newest_root = choose_root(image_before_raising, &superblock)?;
    let newest_table = instance_table_of_root(image_before_raising, &newest_root)?;
    Some(roots_at_floor.iter().all(|root| {
        newest_table
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    }))
}
```

### crates/singlefs-harness/src/history.rs:272-283（GenerationWeights）

```rust
/// 生成一段历史时各样东西的比重（每张表写份数；操作那三张按百分比写，和为 100）：起点两种各占几份，三种会话估计下各类操作各占几份。
/// 抽法是 [0, 份数之和) 里取一个数、按表的次序落到哪一格：起点那张写成 1 : 1 时就是第一版的「种子的第一个数对 2 取余」，
/// 快档与大档的每个种子生成的历史与第一版逐项相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenerationWeights {
    /// 报告里的名字。
    pub name: &'static str,
    pub starting_points: &'static [(HistoryStartingPoint, u64)],
    pub with_session_closed: &'static [(HistoryOperationKind, u64)],
    pub with_session_open_without_file: &'static [(HistoryOperationKind, u64)],
    pub with_session_open_with_file: &'static [(HistoryOperationKind, u64)],
}
```

### crates/singlefs-harness/src/history.rs:285-360（impl GenerationWeights）

```rust
impl GenerationWeights {
    /// 快档与大档用的那一组。关着的会话只抽挂载与冷启动；树表 0 条的版本上多抽第一个文件，不然从 mkfs 起的历史多半先推零单元发布、
    /// 再也接不上第一个文件；带文件的版本上零单元发布只会记「前提不满足」，少抽。起点两种各半。
    pub const BROAD: GenerationWeights = GenerationWeights {
        name: "各类操作都抽（快档、大档）",
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 1),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 70),
            (HistoryOperationKind::CloseAndMountRollback, 22),
            (HistoryOperationKind::ColdStartRecover, 8),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 60),
            (HistoryOperationKind::PublishWithoutUnits, 20),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::CloseAndMountRollback, 4),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishOverwrite, 3),
            (HistoryOperationKind::RaiseRollbackFloor, 3),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 52),
            (HistoryOperationKind::RaiseRollbackFloor, 16),
            (HistoryOperationKind::CloseAndMountWritable, 12),
            (HistoryOperationKind::CloseAndMountRollback, 10),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishFirstFile, 4),
            (HistoryOperationKind::PublishWithoutUnits, 2),
        ],
    };

    /// 第 121 行那一类（复用时新记录罩住别的已回收记录）的专门取样点：多从第一个文件起、多覆盖写、多抬 F、多可写挂载，
    /// 少回退与冷启动。罩住别的已回收记录要：抬 F 回收一批 1 槽的固定点单元，重开之后上一次挂载的聚簇段不再挡用户数据，
    /// 下一个 2 槽的数据单元落在两条相邻的已回收记录上。
    pub const REUSE_AFTER_RAISING_THE_FLOOR: GenerationWeights = GenerationWeights {
        name: "偏向抬 F 之后的复用（第 121 行那一类的取样点）",
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 90),
            (HistoryOperationKind::CloseAndMountRollback, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 85),
            (HistoryOperationKind::PublishWithoutUnits, 5),
            (HistoryOperationKind::CloseAndMountWritable, 4),
            (HistoryOperationKind::CloseAndMountRollback, 2),
            (HistoryOperationKind::ColdStartRecover, 2),
            (HistoryOperationKind::PublishOverwrite, 1),
            (HistoryOperationKind::RaiseRollbackFloor, 1),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 55),
            (HistoryOperationKind::RaiseRollbackFloor, 22),
            (HistoryOperationKind::CloseAndMountWritable, 18),
            (HistoryOperationKind::CloseAndMountRollback, 2),
            (HistoryOperationKind::ColdStartRecover, 1),
            (HistoryOperationKind::PublishFirstFile, 1),
            (HistoryOperationKind::PublishWithoutUnits, 1),
        ],
    };

    fn for_session(&self, expected: ExpectedSession) -> &'static [(HistoryOperationKind, u64)] {
        match expected {
            ExpectedSession::Closed => self.with_session_closed,
            ExpectedSession::OpenWithoutFile => self.with_session_open_without_file,
            ExpectedSession::OpenWithFile => self.with_session_open_with_file,
        }
    }
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:47-137（assert_every_path_was_exercised）

```rust
/// 各条路径都真的跑到了：每类操作至少一次 Ok，回退的目标候选集里外都试过，抬 F 撞过上限也回收过落点，复用改写过已释放的记录
/// （含跨度变了的与被删的），六种内容长度都进过入口，冷启动读回过文件，checker 真的判过 I-3.1 与 I-5.4，至少一段历史转过根环。
fn assert_every_path_was_exercised(tally: &HistoryTally) {
    for kind in HistoryOperationKind::ALL {
        let applied = tally
            .operations_by_kind
            .get(&kind)
            .map_or(0, |operation| operation.applied);
        assert!(applied >= 1, "{kind:?} 一次 Ok 都没有");
    }
    let refusals = &tally.refusals_by_member;
    for member in [
        "MountError::RollbackFloorAboveCeiling",
        "MountError::RollbackTargetNotInRing",
        "MountError::RollbackToVersionWithoutFileUnsupported",
        "PublishError::ContentExceedsDataUnit",
        "PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp",
    ] {
        assert!(count_of(refusals, member) >= 1, "没见过 {member}");
    }
    assert!(
        refusals
            .keys()
            .any(|member| member.starts_with("MountError::RollbackTargetNotACandidate")),
        "回退的目标没落到过被抛弃的根或 F 之下的根"
    );
    assert!(tally.raises_that_reclaimed >= 1, "抬 F 一次都没回收到落点");
    assert!(
        tally.records_rewritten_from_released >= 1,
        "一条已释放的记录都没被复用"
    );
    assert!(
        tally.records_rewritten_with_changed_span >= 1,
        "复用时跨度一次都没变过（Z1-d 那条路没跑到）"
    );
    assert!(
        tally.released_records_removed >= 1,
        "复用时一条被罩住的已释放记录都没删过"
    );
    for length in [
        ContentLength::Empty,
        ContentLength::InsideOneDataUnit { selector: 0 },
        ContentLength::OneByteBelowDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitPayloadCapacity,
        ContentLength::OneByteAboveDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitBytes,
    ] {
        assert!(
            tally
                .content_lengths_attempted
                .get(length.name())
                .copied()
                .unwrap_or(0)
                >= 1,
            "内容长度「{}」没进过入口",
            length.name()
        );
    }
    // 装得下的四种长度各至少发成一次：只看「进过入口」的话，全被拒也算进过（代码三方第一轮攻方的 B1 / P3）。
    for length in [
        ContentLength::Empty,
        ContentLength::InsideOneDataUnit { selector: 0 },
        ContentLength::OneByteBelowDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitPayloadCapacity,
    ] {
        assert!(
            tally
                .content_lengths_published
                .get(length.name())
                .copied()
                .unwrap_or(0)
                >= 1,
            "装得下的内容长度「{}」一次都没发成",
            length.name()
        );
    }
    assert!(
        count_of(&tally.recovery_outcomes, "FileRead") >= 1,
        "冷启动一次都没读回文件"
    );
    for invariant in ["I-3.1", "I-5.4"] {
        assert!(
            tally.invariant_holds.get(invariant).copied().unwrap_or(0) >= 1,
            "checker 一次都没判过 {invariant}（全是不适用）"
        );
    }
    assert!(
        tally.histories_that_turned_the_root_ring >= 1,
        "没有一段历史转过根环"
    );
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:139-163（random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation）

```rust
/// 快档：种子 [0, 96)、每段 30 步。每一步之后跑池级 checker；入口的 `Err` 算合法结局，panic 与违例算失败，撞到「已知红」清单里的形态
/// 照记、那段到此为止，清单外的一条都不许有（有就按签名归类、报出种子与失败在哪一步；收缩交给「收缩一个种子」那条 `#[ignore]` 用例，
/// debug 下收缩一类要两分多钟）。计数照打，并核各条路径真的跑到了。
#[test]
fn random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        FAST_TIER_FIRST_SEED,
        FAST_TIER_SEEDS,
        FAST_TIER_OPERATIONS_PER_HISTORY,
        &GenerationWeights::BROAD,
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    assert_every_path_was_exercised(&report.tally);
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:165-200（reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms）

```rust
/// 第 121 行那一类（复用时新记录罩住别的已回收记录）的专门取样点：比重取 `GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR`，
/// 种子 [0, 48)、每段 30 步（代码三方第一轮判决第二节第 7 条）。2026-09-18 在 release 下量：第 121 行的变异施加之后，这组比重在种子
/// [0, 6000) × 30 步上 799 个种子判出（13.3%），按 48 个种子一窗切 125 窗、每窗至少 2 个；门禁这一窗 [0, 48) 判出 7 个。
/// 先判清单外的失败（变异下红在这一条），再核这一路真的跑到了：罩住别的已释放记录的起点、复用时跨度变了、抬 F 回收到落点——
/// 这几个数不看罩住的那条删没删，变异下照样成立，门禁 59 号看到的红只会是分类判出来的。
#[test]
fn reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        REUSE_SAMPLING_FIRST_SEED,
        REUSE_SAMPLING_SEEDS,
        REUSE_SAMPLING_OPERATIONS_PER_HISTORY,
        &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：偏向抬 F 之后复用的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    let tally = &report.tally;
    assert!(
        tally.released_records_covered >= 1,
        "新分配的记录一次都没罩住别的已释放记录的起点（第 121 行那条路没跑到）"
    );
    assert!(
        tally.records_rewritten_with_changed_span >= 1,
        "复用时跨度一次都没变过"
    );
    assert!(tally.raises_that_reclaimed >= 1, "抬 F 一次都没回收到落点");
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:311-381（an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding）

```rust
/// 不经回退的抬 F 之后 I-3.1 记账多算是新发现，不是清单第 1 条（代码三方第一轮判决第二节第 1 条）。历史照攻方的 `opus_z2_history.rs`：
/// 第一个文件之后可写挂载（实例 2，txg 4、5）、覆盖写三次（6、7、8）、抬 F（选择子 3 ⇒ F = 3），一次回退都没有。今天的代码上这段跑完，
/// 抬 F 之前的镜像上 txg 3 那条根是实例 1 的有效根 ⇒ 「F 那个 txg 上的根全属于被抛弃的实例」为假；拿它配上攻方在回收门槛差一
/// （A1）下量到的那一步违例（`盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040`），分类是新发现。同一个判定在第 1 条的复现
/// （种子 80 那一段，F = 8 落在被抛弃的实例 3 上）为真，那一条由上面的用例钉着。
#[test]
fn an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding() {
    let three_overwrites = std::iter::repeat_n(
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed: 7,
        }),
        3,
    );
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(three_overwrites)
            .chain(std::iter::once(HistoryOperation::RaiseRollbackFloor(
                FloorTargetChoice {
                    steps_above_current_floor: 3,
                },
            )))
            .collect(),
    };
    let mut image_before_raising: Option<MemoryPool> = None;
    let run = execute_history_observing(&history, &SharedStream::new(), &mut |observation| {
        if observation.position == StepPosition::Operation(3) {
            image_before_raising = Some(observation.image.clone());
        }
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "今天的代码上这段跑完");
    let StepOutcome::Applied(AppliedEffect::RaisedFloor { new_floor, .. }) = run.outcomes[4] else {
        panic!("第 4 步抬 F 要成：{:?}", run.outcomes[4]);
    };
    assert_eq!(new_floor, CheckpointTxg(3));
    let lands_only_on_abandoned = raised_floor_lands_only_on_abandoned_roots(
        &image_before_raising.expect("观察者见过第 3 步之后的镜像"),
        new_floor,
    );
    assert_eq!(
        lands_only_on_abandoned,
        Some(false),
        "txg 3 那条根是实例 1 的有效根，不在任何回退留下的空档里"
    );
    let observation = FailureObservation {
        position: StepPosition::Operation(4),
        operation_kind: Some(HistoryOperationKind::RaiseRollbackFloor),
        violations: vec![(
            "I-3.1",
            "盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040".to_string(),
        )],
        panic: None,
        newest_ring_root_txg: Some(10),
        root_ring_slot_count: Some(24),
        harness_judgement: None,
        raised_floor_lands_only_on_abandoned_roots: lands_only_on_abandoned,
    };
    let ending = classify_failure(observation);
    assert!(
        matches!(
            &ending,
            HistoryEnding::NewFinding {
                signature: FailureSignature::CheckerViolations { invariants },
                ..
            } if invariants == &vec!["I-3.1"]
        ),
        "不经回退的抬 F 之后 I-3.1 多算要是新发现：{ending:?}"
    );
}
```
