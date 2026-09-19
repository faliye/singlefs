## 六、判别力：每条新测试怎么证明会红

做法：`mutant/` 是 `copy/` 的另一份副本（`rsync -a --exclude target --exclude .git`），自己的 target；`draft/run-mutations.sh` 逐条改坏 `copy/` 里的 `crash.rs` 一处，跑四个测试二进制（`--lib`、`first_transaction_step_seven_layer0`、`second_transaction_step_zero_layer0`、`second_transaction_step_three_formatted_pool_layer0`，debug，`--no-fail-fast`），记红了哪些；改完从 `copy/` 拷回原件再 `touch`，`cmp` 过。基线（不改动）四个二进制全绿，基线红集为空（`logs/mutation-baseline.log`：23 / 5 / 3 / 7 passed）。被测代码里没有 `debug_assert`。日志：`logs/mutation-<名>.log`，摘要用 `draft/summarize-mutation.py` 取。

| 改坏哪一行（副本 crash.rs） | 红了哪些测试 → 哪条断言 |
|---|---|
| M1 第 1069 行 `(0..state_count.div_ceil(states_per_slice))` → `(1..…)`：丢掉第一片 | 7 条：`state_slices_cover_every_state_exactly_once_in_ordinal_order`（「最后一片止于状态数」）；`layer0_partial_enumeration_…`（first_transaction_step_seven_layer0.rs:219 的 22 个状态逐项计数）；`every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_…`（second_transaction_step_three_formatted_pool_layer0.rs:148「展开的段按闭式数」）；`every_crash_state_outside_the_two_unit_segments_…`（second_transaction_step_zero_layer0.rs:447「展开的段按闭式数」）；`stale_tail_…`（:1141 状态数）；`residual_record_…`（:970 状态数）；`one_state_slices_on_eight_threads_…`（:519「平时跑的那 108 个状态」） |
| M2 第 1073 行 `.saturating_add(states_per_slice)` 之后再 `.saturating_add(1)`：相邻两片重叠 1 个状态 | 6 条：`state_slices_cover_…`（「片首接上一片的尾」）；`layer0_partial_enumeration_…`（:219）；formatted pool 快用例（:148「展开的段按闭式数」）；two_unit_segments 快用例（:447「展开的段按闭式数」）；`residual_record_…`（:970）；`one_state_slices_…`（:526 观察者次序） |
| M3 第 598 行 `if self.first_violation.is_none()` → `if first_violation.is_some()`：并片时第一处违例取后面那一片的 | 2 条：`absorbing_a_following_slice_…`（crash.rs 单测，第一处违例那条 assert_eq）；`one_state_slices_on_eight_threads_…`（:530「计数与每一处「第一处」都逐项相同」） |
| M4 第 616 行 `checker_first_violation.entry(invariant).or_insert(detail)` → `insert(invariant, detail)`：checker 第一处取后面那一片的 | 1 条：`absorbing_a_following_slice_…`（「每条不变量的第一处各取最早有的那一片」）。集成用例没红：到 E 的快档 108 个状态上 checker 零违例，这一格只有单测盯着 |
| M5 第 1310 行 `waiting_for_earlier_slices.remove(&next_slice_to_merge)` → `pop_first()`：谁先到先并、先调观察者 | 1 条：`one_state_slices_on_eight_threads_…`（:526「观察者按序号次序看到同一串持久集合」）。靠 8 个线程抢 108 片时到达次序乱，理论上可能碰巧有序而不红，所以没进变异表 |
| M6 第 1035 行 `let subset_mask = ordinal - …start` → 取反：段内子集算错 | 3 条：`the_state_plan_hands_out_…`（「第 k 个状态就是逐段走到的第 k 个」）；`layer0_partial_enumeration_…`（:219）；`residual_record_…`（:971 走到残留记录的状态数） |
| M7 第 959 行 `match environment_value {` → `match environment_value.and(Err::<String, _>(NotPresent)) {`：不读环境变量 | 2 条：`worker_threads_come_from_the_environment_variable_…`（「环境变量设了就不该再问 available_parallelism」那个 panic）；`zero_worker_threads_…` |
| M8 第 961 行 `text.parse::<NonZeroUsize>()` 之后 `.or(Ok(NonZeroUsize::MIN))`：设成 0 悄悄退回 1 个线程 | 1 条：`zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back` |

M8 跑的时候那条测试还写成 `#[should_panic]`；门禁 59 号的判红正则是 `^test … \.\.\. FAILED$`，`should_panic` 的测试输出行带「- should panic」认不出（本地复跑 59 号那 8 行时它报「没跑到」，`logs/gate59-new-rows.log`）。已改成 `catch_unwind` 加断言的普通测试；在这一版上重跑：基线四个二进制照旧全绿（`logs/mutation-baseline-final-code.log`），M8 红在 `zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back`（crash.rs 第 1585 行的 `expect_err`「设成 0 要停下，不许退回 1 个线程接着跑」，`logs/mutation-M8_zero_threads_fall_back_to_one.log`）；59 号判法复跑 8 行全红在点名的测试上（第九节）。

### `crates/mutations.tsv` 末尾追加的 8 行（补丁里，主工作区的表没动）

变异名（第一段）：
1. `增补 2 第 41 行 层 0 并行：丢掉第一片（状态数等于闭式的断言要红）` → 点名 `every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims`
2. `增补 2 第 41 行 层 0 并行：相邻两片重叠（状态数等于闭式的断言要红）` → 同上
3. `增补 2 第 41 行 层 0 并行：相邻两片重叠（切片单测）` → `state_slices_cover_every_state_exactly_once_in_ordinal_order`
4. `增补 2 第 41 行 层 0 并行：按序号取状态时段内子集掩码取反` → `the_state_plan_hands_out_the_same_persisted_sets_in_the_same_order_as_walking_segment_by_segment`
5. `增补 2 第 41 行 层 0 并行：并片时第一处违例取后面那一片的` → `one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread`
6. `增补 2 第 41 行 层 0 并行：并片时 checker 每条不变量的第一处违例取后面那一片的` → `absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation`
7. `增补 2 第 41 行 层 0 并行：不读 SINGLEFS_LAYER0_THREADS` → `worker_threads_come_from_the_environment_variable_before_available_parallelism`
8. `增补 2 第 41 行 层 0 并行：SINGLEFS_LAYER0_THREADS=0 悄悄退回 1 个线程` → `zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back`

派发只要一行；验收第 4 条的「丢一片、两片重叠」各给了一行（第 1、2 行，点名的是已有的「展开的段按闭式数」那条断言所在的快用例），其余 6 行钉的是这一轮新写的合并、次序与线程数判定。嫌多可以只留第 1、2 行。
