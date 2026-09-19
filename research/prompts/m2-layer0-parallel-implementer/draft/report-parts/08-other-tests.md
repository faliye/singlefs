## 八、其余跑重放的用例：每一条用了几个线程、改没改（验收第 7 条）

线程数一律按 `SINGLEFS_LAYER0_THREADS` 没设、本机 `available_parallelism` = 32 取；实际起的工作线程数 = min(32, 片数)，片数 = ⌈状态数 / 每片状态数⌉，默认每片至少 16 个状态。下表「实际」一栏取自 `--test-threads=1 --nocapture` 下每条用例打的 `LAYER0_PARALLEL_START` 行（`logs/per-test-threads.log`）；全量两条取自计时日志。「改没改」指用例源码：走枚举族的用例一行没改，靠 `crash.rs` 自动变成多线程。

| 文件 | 用例 | 状态数 | 实际工作线程（改之前都是 1） | 用例源码改没改 |
|---|---|---|---|---|
| first_transaction_step_seven_layer0.rs | `layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations`（ignored，54 号跑） | 262165 | 32（512 片） | 没改 |
| 同上 | `layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape` | 22 | 2 | 没改 |
| 同上 | `removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker` | 25 | 2 | 没改 |
| 同上 | `positive_control_root_persisted_without_each_unit_is_caught_by_the_oracle` | 10（直接调 `evaluate_state`） | 1 | 没改：状态是手摆的，不走枚举 |
| 同上 | `applying_a_record_whose_units_are_missing_is_caught_by_the_record_checker` | 1（直接调 `recover`） | 1 | 没改 |
| second_transaction_step_zero_layer0.rs | `full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean`（ignored，54 号跑） | 2104413 | 32（512 片） | 没改 |
| 同上 | `every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims` | 108 | 7 | 没改 |
| 同上 | `one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread`（新） | 108 × 2 | 1 与 8（显式） | 新加 |
| 同上 | `residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it`（带观察者） | 31 | 2；观察者在调用线程上按序号调 | 没改 |
| 同上 | `stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state`（带观察者） | 8 | 1（只有 1 片）；观察者里每个状态再跑一遍恢复，也在调用线程上 | 没改 |
| 同上 | `targeted_controls_on_the_second_publish_go_red_where_they_should` | 几个手摆的状态（直接调 `evaluate_state_for_versions`） | 1 | 没改 |
| second_transaction_step_three_formatted_pool_layer0.rs | `every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims` | 22 | 2 | 没改 |
| 同上 | 另两条（段序列、与第一个事务那条流逐项相同） | 不枚举 | — | 没改 |
| second_transaction_step_three_formatted_pool.rs | `formatted_pool_mount_starting_after_a_leftover_record_is_refused_before_acquiring_an_instance` | 1 个手摆的崩溃状态 | 1 | 没改 |
| second_transaction_supplement_three_random_history.rs（别的会话在改，没碰） | 随机历史快档、大档、收缩 | — | 自己的 `SINGLEFS_RANDOM_HISTORY_THREADS`，快档默认 min(核数, 16) | 没碰 |

`crates/` 里 `enumerate_layer0*` 的调用点只有上面这三个测试文件（`grep -rn 'enumerate_layer0' crates --include=*.rs`）；`src/bin/` 三个二进制不跑崩溃状态枚举（`first_transaction_device_log_check` 只对两份盘镜像跑一次 `recover`）。平时 `cargo test` 里的快用例只有几十到一百来个状态，默认切法下起 1 到 7 个线程；这些用例本来就快，没另外调切法。
