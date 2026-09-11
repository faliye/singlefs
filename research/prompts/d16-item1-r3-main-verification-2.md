# D16（发布语义） 未定项 1 第三轮——主 agent 对反推腿 R1、R2 的独立复现

**口径**：2026-09-12 凌晨（JST）主 agent 做。反推腿（`d16-item1-r3-opus-output.md`）报 R1「推空期间一次根槽写失败 ⇒ 臂 A 出假性 ENOSPC」、R2「一个根槽从此写不进 ⇒ 臂 A 在占用 70% 时报 ENOSPC 并卡死」。
主 agent 没用它的驱动，在自己的仓外副本（`d16-item1-r3-main-verification.md` 那一份副本上续加）里另写两个探针：
- `probe_drain_single_failure`：填满、再做 5 个删建窗口之后，下一次准入处置里第 k 次根槽写失败（第 0 次是发布正在攒的窗口；臂 A 取 k = 1..N − 1，臂 G 取 1..6），再做 20 个删建窗口；开销 5 块，8 个种子。
- `probe_broken_slot`：70% 预热 2N + 10 个窗口之后，槽 1（区域 0、盘 0）从此每次写都失败（按写失败处理：槽里留旧根、txg 推进一格），再做 1000 个删建窗口；8 个种子。
R2 依据的条款现查：D23（journal 的角色与格式） 已定项 14 索引行逐字「⚠️ 由此『同一个根槽连续失败』不再是一条判据」，判别瞬时 / 持续失败靠「对目标设备的固定落点做一次探针写」。
**这是复现，不是跑前登记过的实验**；复现的是计数模型里的行为，不证明真设备上的发生率。

## 副本相对仓里源码改了什么（`diff -u`，含上一份记录里的两个开关与探针）

```diff
--- research/e7-index-bench/src/bin/e138_per_disk_floor.rs	2026-09-11 15:55:27.015560082 +0000
+++ /tmp/claude-1000/-home-fy5090-code-singlefs/86b2ac0e-7e65-418c-97a7-f3d755730fe6/scratchpad/verify-copy/research/e7-index-bench/src/bin/e138_per_disk_floor.rs	2026-09-11 16:46:37.395871497 +0000
@@ -300,6 +300,9 @@
     nonempty_txgs: HashSet<u64>,
     admin_rollback_happened: bool,
     halted: bool,
+    force_full_disposal: bool,
+    charge_worst_residue: bool,
+    broken_slot: Option<usize>,
     metrics: Metrics,
     /// 管理员回退之前盘上那一份分配状态，被抛弃时间线的根引用的是它（E135 的 abandoned_view 原样）。
     abandoned_view: Option<(Rc<PoolState>, u64)>,
@@ -340,6 +343,9 @@
             nonempty_txgs: HashSet::new(),
             admin_rollback_happened: false,
             halted: false,
+            force_full_disposal: false,
+            charge_worst_residue: false,
+            broken_slot: None,
             metrics: Metrics::fresh(),
             abandoned_view: None,
         }
@@ -492,7 +498,8 @@
 
     /// 普通分配看得见的：扣掉保留池。
     fn user_allocatable(&self) -> u64 {
-        self.reusable_count().saturating_sub(self.reserve_blocks)
+        let plain = self.reusable_count().saturating_sub(self.reserve_blocks);
+        if self.charge_worst_residue { plain.min(self.df_guaranteed()) } else { plain }
     }
 
     /// 取 count 块：先取可再分配的已释放块，再取没用过的。复用一块就把它上一段区间记成损坏。
@@ -647,7 +654,7 @@
         let attempt = self.root_write_attempts;
         self.root_write_attempts += 1;
         let is_planned_failure = self.planned_root_write_failures.remove(&attempt);
-        if self.fail_next_slot_write || is_planned_failure {
+        if self.fail_next_slot_write || is_planned_failure || self.broken_slot == Some(slot) {
             self.fail_next_slot_write = false;
             self.relabel_generation(txg, txg + 1);
             self.next_txg = txg + 1;
@@ -824,7 +831,7 @@
                 return WindowOutcome::Halted;
             }
             let committed_txg = self.newest_durable_txg;
-            let publications = 1 + self.push_after_commit(committed_txg, true);
+            let publications = 1 + self.push_after_commit(committed_txg, !self.force_full_disposal);
             self.metrics.maximum_publications_per_admission = self.metrics.maximum_publications_per_admission.max(publications);
             if self.halted {
                 return WindowOutcome::Halted;
@@ -1611,6 +1618,150 @@
         assert!(simulation.pending_floor.is_some(), "新 F 还要照生效条件继续等");
     }
 
+
+    fn ledger(simulation: &Simulation) -> String {
+        let signed = simulation.df_raw() as i64 - (simulation.reserve_blocks + simulation.push_residue_blocks) as i64;
+        format!("reusable={} pinned={} reserve={} residue={} df_raw={} df_g_signed={signed} user_alloc={} floor_c={} floor_a={} txg={}",
+            simulation.reusable_count(), simulation.pinned_count(), simulation.reserve_blocks, simulation.push_residue_blocks,
+            simulation.df_raw(), simulation.user_allocatable(), simulation.floor_carried, simulation.floor_active, simulation.newest_durable_txg)
+    }
+
+
+
+
+    #[test]
+    fn probe_drain_single_failure() {
+        for (label, arm) in [("A", Arm::RingPersisted), ("G", Arm::FloorPerDisk)] {
+            for slots_per_region in [4u64, 16] {
+                let cost = 5u64;
+                let ring_depth = REGION_COUNT * slots_per_region;
+                let maximum_offset = if arm == Arm::RingPersisted { ring_depth - 1 } else { FLOOR_PER_DISK_PUBLICATION_BOUND - 1 };
+                let mut total = Metrics::fresh();
+                let mut runs = 0u64; let mut runs_with_false = 0u64; let mut runs_halted = 0u64;
+                for seed in 0..8 {
+                    for offset in 1..=maximum_offset {
+                        let mut simulation = Simulation::new(arm, slots_per_region, cost, seed, false, None);
+                        simulation.prefill(STEADY_FILL);
+                        for _ in 0..(2 * simulation.ring_depth + 10) { simulation.run_user_window(true, false); }
+                        simulation.metrics = Metrics::fresh();
+                        for _ in 0..CAPACITY_BLOCKS { if simulation.run_user_window(false, false) != WindowOutcome::Created { break; } }
+                        for _ in 0..5 { simulation.run_user_window(true, false); }
+                        let before = simulation.metrics;
+                        // 下一次准入处置里第 offset 次根槽写失败（第 0 次是发布正在攒的窗口）
+                        let base = simulation.root_write_attempts;
+                        simulation.planned_root_write_failures.insert(base + offset);
+                        for _ in 0..20 { if simulation.halted { break; } simulation.run_user_window(true, false); }
+                        let after = simulation.metrics;
+                        runs += 1;
+                        runs_with_false += u64::from(after.false_enospc_guaranteed > before.false_enospc_guaranteed);
+                        runs_halted += u64::from(simulation.halted);
+                        total.add(&simulation.metrics);
+                    }
+                }
+                println!("DRAINFAIL {label} s={slots_per_region} c={cost} runs={runs} runs_with_false_guaranteed={runs_with_false} halted={runs_halted} max_publications={} bound={} false_guaranteed_total={} fakes={}",
+                    total.maximum_publications_per_admission, arm.publication_bound(ring_depth), total.false_enospc_guaranteed, total.fake_candidates);
+            }
+        }
+    }
+
+    #[test]
+    fn probe_broken_slot() {
+        for (label, arm) in [("A", Arm::RingPersisted), ("G", Arm::FloorPerDisk)] {
+            for slots_per_region in [4u64, 16] {
+                for cost in [1u64, 5] {
+                    let mut halted_seeds = 0; let mut no_space_seeds = 0; let mut first_report = String::new();
+                    for seed in 0..8 {
+                        let mut simulation = Simulation::new(arm, slots_per_region, cost, seed, false, None);
+                        simulation.prefill(STEADY_FILL);
+                        for _ in 0..(2 * simulation.ring_depth + 10) { simulation.run_user_window(true, false); }
+                        simulation.broken_slot = Some(1);
+                        let mut windows = 0u64;
+                        let mut saw_no_space = false;
+                        for _ in 0..1000 {
+                            if simulation.halted { break; }
+                            windows += 1;
+                            let df_before = simulation.df_guaranteed();
+                            if simulation.run_user_window(true, false) == WindowOutcome::NoSpace {
+                                if !saw_no_space && first_report.is_empty() {
+                                    first_report = format!("first NoSpace at window {windows}: df_guaranteed_before={df_before} df_guaranteed_after={} live={} oldest_valid={} newest={}",
+                                        simulation.df_guaranteed(), CAPACITY_BLOCKS as u64 - simulation.df_raw(), simulation.oldest_persisted_valid_txg(), simulation.newest_durable_txg);
+                                }
+                                saw_no_space = true;
+                            }
+                        }
+                        halted_seeds += u64::from(simulation.halted);
+                        no_space_seeds += u64::from(saw_no_space);
+                    }
+                    println!("BROKEN {label} s={slots_per_region} c={cost} seeds=8 halted={halted_seeds} no_space={no_space_seeds} {first_report}");
+                }
+            }
+        }
+    }
+
+    #[test]
+    fn probe_variant_summary() {
+        for (label, arm, is_full, is_charged) in [("G_original", Arm::FloorPerDisk, false, false), ("G_prime", Arm::FloorPerDisk, true, false),
+                                                  ("G_charged", Arm::FloorPerDisk, false, true), ("A", Arm::RingPersisted, false, false)] {
+            for slots_per_region in [4u64, 16] {
+                for cost in [1u64, 5] {
+                    let mut total = Metrics::fresh();
+                    for seed in 0..SEED_COUNT {
+                        let mut simulation = Simulation::new(arm, slots_per_region, cost, seed, false, None);
+                        simulation.force_full_disposal = is_full;
+                        simulation.charge_worst_residue = is_charged;
+                        simulation.prefill(STEADY_FILL);
+                        for _ in 0..(2 * simulation.ring_depth + 10) { simulation.run_user_window(true, false); }
+                        let stalls = simulation.metrics.stalls;
+                        simulation.metrics = Metrics::fresh();
+                        simulation.metrics.stalls = stalls;
+                        for _ in 0..CAPACITY_BLOCKS { if simulation.run_user_window(false, false) != WindowOutcome::Created { break; } }
+                        for _ in 0..WINDOWS_PER_WORLD {
+                            if simulation.halted { break; }
+                            if simulation.run_user_window(true, false) == WindowOutcome::NoSpace { simulation.metrics.write_after_delete_failed += 1; }
+                        }
+                        total.add(&simulation.metrics);
+                    }
+                    println!("VARIANT {label} s={slots_per_region} c={cost} write_after_delete_failed={} false_guaranteed={} false_raw={} true_enospc={} stalls={} max_publications={} fakes={}",
+                        total.write_after_delete_failed, total.false_enospc_guaranteed, total.false_enospc_raw, total.true_enospc, total.stalls,
+                        total.maximum_publications_per_admission, total.fake_candidates);
+                }
+            }
+        }
+    }
+
+    #[test]
+    fn probe_cold_start() {
+        for (label, arm, is_full) in [("G_original", Arm::FloorPerDisk, false), ("G_prime", Arm::FloorPerDisk, true), ("A", Arm::RingPersisted, false)] {
+            for slots_per_region in [4u64, 16] {
+                let mut failure_windows: Vec<(u64, u64)> = Vec::new();
+                for seed in 0..SEED_COUNT {
+                    let mut simulation = Simulation::new(arm, slots_per_region, 5, seed, false, None);
+                    simulation.force_full_disposal = is_full;
+                    simulation.prefill(STEADY_FILL);
+                    for _ in 0..(2 * simulation.ring_depth + 10) { simulation.run_user_window(true, false); }
+                    let mut fill_windows = 0;
+                    loop {
+                        fill_windows += 1;
+                        let before = ledger(&simulation);
+                        match simulation.run_user_window(false, false) {
+                            WindowOutcome::Created => {}
+                            other => { if seed < 2 { println!("{label} s={slots_per_region} seed={seed} fill_end window={fill_windows} {other:?}\n  before: {before}\n  after:  {}", ledger(&simulation)); } break; }
+                        }
+                    }
+                    for window in 1..=WINDOWS_PER_WORLD {
+                        let before = ledger(&simulation);
+                        let outcome = simulation.run_user_window(true, false);
+                        if seed < 2 && window <= 3 { println!("{label} s={slots_per_region} seed={seed} cycle window={window} {outcome:?}\n  before: {before}\n  after:  {}", ledger(&simulation)); }
+                        if outcome == WindowOutcome::NoSpace { failure_windows.push((seed, window)); }
+                    }
+                }
+                let seeds: BTreeSet<u64> = failure_windows.iter().map(|(seed, _)| *seed).collect();
+                let windows: BTreeSet<u64> = failure_windows.iter().map(|(_, window)| *window).collect();
+                println!("SUMMARY {label} s={slots_per_region} c=5 failures={} seeds_with_failure={} failure_windows={:?}", failure_windows.len(), seeds.len(), windows);
+            }
+        }
+    }
+
     #[test]
     fn raw_df_detector_fires_for_ring_arm_near_full() {
         let total = run_near_full(Arm::RingPersisted, 4, 5, 0, None);
```

## 探针输出（整行抄，`cargo test --release --bin e138_per_disk_floor -- probe_drain_single_failure probe_broken_slot --nocapture --test-threads=1`）

```text
BROKEN A s=4 c=1 seeds=8 halted=8 no_space=8 first NoSpace at window 84: df_guaranteed_before=1159 df_guaranteed_after=1171 live=2793 oldest_valid=27 newest=138
BROKEN A s=4 c=5 seeds=8 halted=8 no_space=8 first NoSpace at window 81: df_guaranteed_before=1075 df_guaranteed_after=1083 live=2797 oldest_valid=27 newest=134
BROKEN A s=16 c=1 seeds=8 halted=8 no_space=8 first NoSpace at window 81: df_guaranteed_before=1087 df_guaranteed_after=1099 live=2793 oldest_valid=99 newest=236
BROKEN A s=16 c=5 seeds=8 halted=8 no_space=8 first NoSpace at window 67: df_guaranteed_before=715 df_guaranteed_after=723 live=2797 oldest_valid=99 newest=222
BROKEN G s=4 c=1 seeds=8 halted=0 no_space=0 
BROKEN G s=4 c=5 seeds=8 halted=0 no_space=0 
BROKEN G s=16 c=1 seeds=8 halted=0 no_space=0 
BROKEN G s=16 c=5 seeds=8 halted=0 no_space=0 
DRAINFAIL A s=4 c=5 runs=88 runs_with_false_guaranteed=88 halted=0 max_publications=12 bound=12 false_guaranteed_total=88 fakes=0
DRAINFAIL A s=16 c=5 runs=376 runs_with_false_guaranteed=376 halted=0 max_publications=48 bound=48 false_guaranteed_total=376 fakes=0
DRAINFAIL G s=4 c=5 runs=48 runs_with_false_guaranteed=8 halted=0 max_publications=7 bound=7 false_guaranteed_total=8 fakes=0
DRAINFAIL G s=16 c=5 runs=48 runs_with_false_guaranteed=8 halted=0 max_publications=7 bound=7 false_guaranteed_total=8 fakes=0
```
