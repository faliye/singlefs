# 实审 B3a-2 报告（implementation-writer，2026-09-26 写）

## 一、结论

| 条 | 结局 | 一句话 |
|---|---|---|
| 1 录制器按设备记屏障 | 已改 + 新用例 | `SharedStream::push`（`crates/singlefs-harness/src/lib.rs` 第 183 行起）只并掉「同一块盘在一串连着的屏障里已经记过」的那一道，不同设备各记一步 |
| 1、3 切段照 A、FUA 只放行自己那块盘 | 已改 + 新用例 | 新 `SegmentClosingRule`（`segments.rs` 第 107 行起）：当前段里每块有写的盘都被自己的屏障或 FUA 放行才关段；`split_into_segments` 与 `crash::writes_and_segments_with_stream_indexes_and_entries` 都经它判 |
| 判别力（只吞盘 1 一处屏障） | 新用例，原代码上红 | 第一条流吞盘 1「记录 → 根槽」那一道（录制器外面）：段尾 `26,2,1,2` → `26,5`，层 0 小流里记录核对器报「根在案而记录一份都不在」 |
| 2 跟着改的 5 份 | 改 3 份，2 份不用改 | step_five / step_one 的操作数与种类串、`device_log.rs` 的 FLUSH 期望；E142 两个 bin 不改代码（每道池屏障多打一行 `window_barrier`） |
| 4 原地覆写第三态 | 已改 + 新用例 | `TearableInPlaceOverwrites`（B3a 第四节三条）、`torn_image_of_in_place_overwrite`、`WritesWithTornImages`（撕裂镜像接在枚举用写表末尾）、计划按混合进制拆；全量与甲二都枚举，镜像喂恢复 / checker / 记录核对器 |
| 层 0 钉值 | 改了，**没跑** | 第一条流全量 67108885 → 150994980、甲二 29 → 54；第二条流（64 段）全量 6649413719 → 14960689284、甲二 205 → 390（闭式由探针现算、不枚举）；恢复计数按「撕裂 ≡ 同一次写没持久」推 |
| 主 agent 中途消息（门禁 33 点名 5 行） | 5 行改完；证红 4 行 | 116/153/784/785/859 改到今天源码恰好命中一次；859 点名的用例被我的第三态带红（钉 29），基线红、证不了 |

推翻条件：主树上 `cargo test -p singlefs-harness --test crash_segments_per_device_and_torn_in_place_overwrites` / `--lib` / `--test first_transaction_step_five_publish` / `--test first_transaction_step_one_mkfs` 任一红；`mutations-append.tsv` 进表后 59 号有一行不红；两条层 0 流跑出的状态数、按发布分的串、恢复计数与第七节不同。

## 二、写过的文件

- `crates/singlefs-harness/src/lib.rs`：屏障合并条件、文档。
- `crates/singlefs-harness/src/segments.rs`：`SegmentAfterOperation`、`SegmentClosingRule`，`split_into_segments` 改走它。
- `crates/singlefs-harness/src/crash.rs`：切段走 `SegmentClosingRule`；新 `TearableInPlaceOverwrites`（第 1811 行）、`is_tearable_in_place_overwrite`（第 1885 行）、`torn_image_of_in_place_overwrite`（第 1930 行）、`WritesWithTornImages`（第 1954 行）、`WriteLandingInCrashState`、`WriteLandingChoices`、`layer0_state_count_with_torn_in_place_overwrites`（第 2090 行）；`Layer0SegmentExpansion::state_count` 多一个参数；`Layer0StatePlan::persisted_writes_of_state`（第 2236 行）按混合进制出持久集合；枚举本体（第 2793 行起）、观察者、计划哈希用枚举写表；`mod tests` 旧 2 条跟着改签名、新 2 条。`layer0_state_count`、`closed_form_state_count` 签名与语义不动（两态口径）。
- `crates/singlefs-harness/src/device_log.rs`：屏障只投给 `operation.device`；它的用例按设备摆屏障、加「盘 1 漏发」。
- `tests/first_transaction_step_five_publish.rs`：取号后 +3 → +4；21/14/31/47 → 23/18/33/53；种类串屏障按设备数。
- `tests/first_transaction_step_one_mkfs.rs`：21 → 23、种类串 `barrier×2`。
- `tests/first_transaction_step_seven_layer0.rs`、`tests/second_transaction_step_zero_layer0.rs`：第七节。
- 新文件 `tests/crash_segments_per_device_and_torn_in_place_overwrites.rs`（9 条用例，两条带 `// crash-case-check:not-a-crash-case`）。
- `crates/mutations.tsv`：只改主 agent 点名的 5 行（第五节），没追加；要追加的 18 行在 `/tmp/claude-1000/impl-rev-b3a2/mutations-append.tsv`。
- E142 两个 bin：没改。

我的改动行数（对本轮开工时的快照 `originals/`，`git diff --no-index --numstat`）：lib.rs +11/−5、segments.rs +80/−24、crash.rs +726/−93、device_log.rs +29/−7、step_five +12/−10、step_one +4/−4、step_seven_layer0 +81/−34、step_zero_layer0 +127/−53、新用例 816 行。

## 三、实现要点与设计判断

1. 录制器：一串连着的屏障（中间没有任何写）里，同一块盘已经记过的那一道不再记；不同盘各记一步。每盘都发屏障时段序列与闭式与今天逐字相同（用例 `a_barrier_on_every_device_cuts_the_same_segments_as_one_pool_barrier`、真流上 `the_first_transaction_stream_keeps_its_segments_…` 钉 `2,2,1,2,2,1,26,2,1,2` 与 67108885）。
2. 切段（A 合并）：写入把它那块盘记成「没放行」；屏障移出它那块盘；FUA 写算一个写、并移出它那块盘；段里有写且没放行的盘为空才关段。多余的屏障（这块盘自上一道以来没有写）永远不触发关段，所以录制器并不并同盘屏障都不影响段。
3. 第三态：写表下标不动，撕裂镜像（罩那次写整个范围，字节 = 新旧不同那一截前一半新、后一半旧，旧字节取「这次写之前的写全落了」）与「同段之后、同盘、重叠的写」的重放接在枚举用写表末尾；撕裂态 = 原写不持久 + 撕裂镜像持久，重放跟被重放那次写落不落。三态数字：0 没持久、1 撕裂、2 持久，只取两态的写上就是今天的子集掩码，次序逐个相同（旧的两条计划用例原样绿）。
4. E142 两个 bin：代码不用改——每条屏障步照打一行 `name=window_barrier step= device=`，按设备记之后每道池屏障打两行、`step=` 编号后移；`research/e7-index-bench` 的 `window_segment_sizes_from_dump`（第 5466 行起）对连着的第二道屏障不关段，段大小不变。入库产物（`research/results/e142-r16-crates-dump-2026-09-25.out` 等）与新 bin 的输出会不同，重出归主 agent 另派执行员。

**要主 agent 定的（我先按下面的写法做了，改法都只落在一两个函数里）**：
- A. 撕裂镜像的粒度：我按字节取「新旧不同那一截前一半新、后一半旧」。理由：两条流里取三态的只有系统配置槽写，字段表 489 字节全在槽的第一个扇区（`crates/singlefs-format/src/lib.rs:224`），按扇区撕两槽只会是整新或整旧，「新旧都读不出」按扇区造不出来。这与 B3a 认法第 2 条「一个扇区的写撕不开」（按扇区粒度）口径不一致；要全按扇区，第三态在这两条流上就退化成与另外两态逐字节相同的重复状态。
- B. 旧字节取「这次写之前的写全落了」时的：同段更早、与它重叠的写没落的状态里，旧那一半多带了那次写的字节（两条流上这种组合一次都没有）。
- C. 种类串里屏障按设备数（`barrier×2`），没折成一道池屏障一个；登记表与 step_one_overwrite 的钉值因此跟着变（第八节）。
- D. 根槽写长于一个扇区又罩住旧内容时 `TearableInPlaceOverwrites::of` 断言（撕裂镜像接在写表末尾会被择根与记录核对器当成又一条根）。走不到的理由：根槽写宽 = 物理块（`crates/singlefs-core/src/make_filesystem.rs:327`），层 0 的池都按 512 字节物理块建（`crates/singlefs-harness/tests/common/mod.rs` 第 113、134、163、260 行），根槽写一个扇区，第 2 条先把它挡掉。

## 四、新测试在原代码上红（先写用例、看红、再改）

原代码快照副本上只加新用例文件的 part A（第 1、3 条那 6 条），`cargo test -p singlefs-harness --test crash_segments_per_device_and_torn_in_place_overwrites`（经 `run-with-memory-cap.sh 8G`、`capped.sh 4`），日志 `/tmp/claude-1000/impl-rev-b3a2/red-demo-before.log`：
```text
test a_barrier_on_every_device_cuts_the_same_segments_as_one_pool_barrier ... ok
test a_force_unit_access_write_releases_only_its_own_device ... FAILED
test a_barrier_closes_the_segment_only_once_every_device_with_a_write_in_it_is_released ... FAILED
test a_device_that_misses_its_barrier_records_a_different_stream_than_every_device_barriering ... FAILED
test back_to_back_barriers_on_one_device_are_recorded_once_per_device ... FAILED
test one_device_missing_its_barrier_before_the_root_slot_is_reported_by_the_record_checker ... FAILED
test result: FAILED. 1 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
```
绿的那条钉今天的形状（每盘都发屏障时与今天相同）。那一版里这条还拿「只有盘 0 屏障」的流对比，改后不成立，已改成直接钉段。第三态那 3 条 + lib 2 条用的是新 API，在原代码上编不过；它们的红由第五节变异证。改后：新目标 `9 passed`、`--lib` `89 passed`、step_five `8 passed`、step_one `9 passed`（第六节）。

## 五、变异证红

副本 `/tmp/claude-1000/impl-rev-b3a2/copy-work`（开工快照 + 我的文件，自己的 target）；表 = 主表（已改好 5 行）+ 用 `insert-row.py` 追加的 18 行。命令：`PROVE_RED_LOG_DIRECTORY=…/prove-red-logs bash research/scripts/capped.sh 4 bash research/scripts/prove-red.sh --copy …/copy-work --memory 8G singlefs-harness <22 个名字>`，基线（`prove-red-logs/baseline.log`）没红；末行原样：`✓ 点名 22 条：跑了 20 条，跳过 2 条，跑的都抓到了`。跑完我的 4 个源文件与主树逐字节相同（`cmp`）。

| 变异（名字简写） | 改坏哪一行 | 红的断言（原样摘） |
|---|---|---|
| 录制器不分设备并屏障 ×3 行 | lib.rs `.any(|previous| previous.device == operation.device)` → 比 kind | 新用例 `每块盘的屏障各记一步 left: [DeviceIdentity(0)]`；step_five `left: 24 right: 25`；step_one `left: "14+1+4" right: "12+1+1+1+4"` |
| 同盘连着的屏障各记一步 | lib.rs 合并条件后加 `&& false` | `两轮逐盘屏障之间没有写：每块盘一道`（多出两步） |
| FUA 放行每块盘 | segments.rs FUA 臂 `.remove` → `.clear()` | `left: [[0, 1], [2]] right: [[0, 1, 2]]` |
| 屏障放行每块盘 ×2 行 | segments.rs 屏障臂 `.remove` → `.clear()` | `left: [[0, 1], [2], [3]]`；真流 `left: [2,2,1,2,2,1,26,2,1,2] right: […,26, 5]` |
| 屏障把自己记成没放行 | 屏障臂 `.remove` → `.insert` | `left: [[0, 1, 2, 3, 4, 5, 6]]` |
| 设备日志把别盘屏障投成 FLUSH | device_log.rs 屏障臂删掉设备判定 | `写、它自己那道屏障的 FLUSH… left: 5 right: 4` |
| 原地覆写只取两态 ×4 行 | crash.rs `NotPersistedTornOrPersisted => 3` → `2` | `left: 67108885 right: 150994980`；手摆写表 `left: 18 right: 27`；另两行点层 0 目标，跳过、留 59 号 |
| 认原地覆写不看基镜像 | `base_holds… \|\| an_earlier…` → 只留后者 | `left: [5] right: [0, 5]` |
| 撕裂态不叠撕裂镜像 | `persisted[torn_image_index] = true` → `false` | `(states_with_a_torn_slot, torn_slots) left: (0, 0) right: (5, 6)` |
| 撕裂镜像取整次新写 | `+ differing_span_in_bytes / 2` → `+ differing_span_in_bytes` | `left: [1,2,2,2,2,2,2,1] right: [1,2,2,2,1,1,1,1]` |
| 撕裂 / 持久数字对调 | `1 => Torn, 2 => Persisted` 对调 | `第 k 个状态就是参照走法的第 k 个` |
| 不重放同段更晚重叠写 | `*later_index > write_index` 加 `&& false` | `left: 8 right: 9`（写表少一条重放） |
| 116（改锚点） | 段内序号取反 | `段内序号 18446744073709551615 超出…` |
| 153（改锚点，改盯 segments.rs 屏障臂） | 屏障臂 `close_…` → `StaysOpen` | 崩溃注入 `写死的这段历史上崩溃状态判红` |
| 784（改锚点：`/ 2`） | `ordinal_within_the_segment / 2` → 不除 | `段内序号 4 超出这几次写（[0, 1]）的组合数` |
| 785（改锚点：单元写两种选法） | `else { 2 }` → `1` | `left: 9 right: 16` |

prove-red 的参数带 `-- <过滤>`（与表里已有各行同形），同一次运行只跑过滤到的那几条，「同时红」只看得到它们：每次都是点名的那 1 条红、其余 0 条。

5 行腐化锚点的处置（主 agent 中途消息，在「层 0 两份钉值改到一半」那一步收到）：116、784、785 行为没变，只换原文到今天的写法；153 原文盯的是 crash.rs 切段里的屏障臂，切段挪进 `SegmentClosingRule` 之后改盯 segments.rs 屏障臂（同一个意思：屏障不关段），还是那条崩溃注入用例红；859 同 785 的改法（锚点改好、命中 1 次），它点名的 `crash_enumeration_sharded_across_processes` 钉第一条流甲二 29，补第三态之后是 54，基线就红（第八节），证不了，等那份文件改钉值之后复证。改完 `mutations/check_anchors.py` 核我这几个文件的全部 61 行：`checked 61`、0 行不中。

`mutations-append.tsv`（18 行，六段，名字表里都没有）：证过 16 行；留给门禁 59 号 2 行（「…只取两态（第一条流甲二快档退回 29）」「…（第二条流甲二快档退回 205）」，点的是层 0 目标）。追加请用 `insert-row.py`，每行 `--absent` 同名。

## 六、交回前的验证（末尾原样）

都在副本 copy-work（开工快照 + 我的文件）上跑：主树的 checker 这一刻有别的会话改到一半、编不过（`walk.rs:1241 this method takes 2 arguments but 3 arguments were supplied`），在主树上跑判不出我的改动。
- 新目标：`test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.95s`
- `--lib`：`test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 99.60s`
- step_five：`test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.83s`；step_one：`test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.51s`
- 两个 layer0 目标、E142 两个 bin 的用例：没跑（layer0 按派发不跑；bin 没改）。
- `cargo build --offline --all-targets`：无 error / warning 输出。
- `cargo fmt --check -p singlefs-harness`：只剩别人的 3 个文件（`a_floor_raise_refused_for_space_counts_as_short_of_space.rs`、`entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write.rs`、`writable_mount_refuses_a_device_table_disagreeing_with_the_system_configuration.rs`）；我的文件按 rustfmt 的输出逐字相同（`cmp`）。
- clippy（check.sh 那套 `-D warnings` + 7 条）`--keep-going --all-targets`：`could not compile` 只在 `e156_allocation_basis_counts`、`e158_root_choice_repair` 两个 bin（别人的，B3a 报告已记）；我的文件 0 条。
- 登记给我的门禁（主树）：33 号 `exit=1`，点名 12 行全在 `crash_injection.rs` 与 checker `walk.rs`（别人在改），我的 5 行已不在其中；53 号 `✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位…）`；92、94、93 号 exit=0（94：`✓ checker 与实现只共享常量模块 singlefs-format…`；93：`✓ feature bit 位号…一致（…扫了 56 个 .rs…）`）；89 号 exit=77（本次未跑）；74 号 `exit=1`：主树上 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`、`random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation` 两条红。对照（release，同一个测试二进制）：开工快照原件与加上我的改动各跑一次，都是 `23 passed; 1 failed`、红的都是 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`（快照里就红），74 号那两条在我的副本上绿 ⇒ 74 号那两条红来自主树上别人没提交的改动，不是这一轮的。

## 七、层 0 钉值（两份 layer0 文件，没跑，集成时要跑）

闭式与按发布分由探针现算（副本里把 step_zero_layer0 整份拷成非 layer0 名字的目标、摘掉原有 `#[test]`，只 `prepare` 再调 `layer0_state_count_with_torn_in_place_overwrites`，不枚举；源码 `draft/probe_second_stream_counts.rs`，日志 `probe-second-stream.log`）：
- 第二条流（到 E 再卸载，64 段、477 写、取三态的 46 次全是系统配置槽写）：两态 6649413719 / 甲二 205 → 三态 14960689284 / 390；按发布分（全量）`instance1_txg1=12 instance1_txg2=12 instance1_txg3=150994947 instance1_txg4=150994947 instance2_txg5=262163 instance2_txg6=589827 instance2_txg7=589827 instance2_txg8=150994947 instance2_txg9=9437187 instance2_txg10..14=2415919107 instance2_txg15=65555 instance2_txg16=589827 instance2_txg17=2415919107 instance2_txg18=65555 instance2_txg19=589827 after_the_last_root=8 every_write_persisted=1`，甲二每次发布 21、txg1/2 各 12、末段 8。到 C：454426691 / 159；到 D：463863878 / 180。残留记录流 22 → 37（能走到残留记录的 10 → 20）；陈旧 tail 流 8 → 13。
- 第一条流：150994980 / 54；按发布分全量 `12 12 150994947 8 1`、甲二 `12 12 21 8 1`；与新用例里拿同一函数算的数相同（那条是跑了的）。
- **推的、没跑的**（step_seven 的恢复计数）：根已持久 4 → 9、读出文件 7 → 12、环里没记录 4 → 9、journal 承重 3 与验证跑过 6 不变。依据：撕裂的只有系统配置槽轮换写，撕裂的那一槽解不开、择同盘另一槽，与那次写没持久时择的是同一槽（没持久时那一槽里是更旧的一代）；这些状态里没有待施加的记录。新用例 `the_torn_state_of_a_system_configuration_rotation_…` 在最后一段 9 个状态上跑过：`file_read=9`、违例 0、checker 与记录核对器 0、每条不变量「评估 + 不适用 = 9」。
- 受影响的层 0 流与崩溃枚举用例（集成时要跑）：`crash-case:layer0-first-stream`、`crash-case:layer0-second-stream` 两条全量；两份 layer0 文件的快档；另见第八节会跟着变的层 0 文件。

## 八、清单外会跟着红的文件（我不能改，交主 agent）

在副本 copy-radius（开工快照 + 我的改动）上逐个跑过的（日志 `radius-logs/`）：
- `tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`：`8 passed; 3 failed`——三条钉录制流里抬 F / 卸载那一串的「一道屏障」，按设备记之后是两步，例：`left: [(Barrier, DeviceIdentity(0)), (Barrier, DeviceIdentity(1)), (Write, DeviceIdentity(0))] right: [(Barrier, DeviceIdentity(0)), (Write, DeviceIdentity(0))]`（第 767 行附近），另两条 `failing_to_write_the_new_floor_…`、`normal_unmount_raises_the_floor_…`、`raising_the_floor_writes_the_new_floor_…`。
- `tests/second_transaction_step_one_overwrite.rs:97` 种类串：新值 `[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]`。
- `tests/crash_enumeration_sharded_across_processes.rs`：`5 passed; 5 failed`——钉第一条流甲二 29（今天 54）、对拍的金样输出与账本文件头里的计划哈希（撕裂镜像进了哈希）。这份文件也是 859 行点名的。
- 同一批跑绿的：`crash_enumeration_resumes_from_its_progress_file`、`record_checker_judges_absence_by_the_persisted_set`、`in_place_overwrite_torn_state_count`、`crash_image_journal_hint_matches_the_full_ring_scan`、`second_transaction_step_three_formatted_pool`、`…c533…`、`…presumed_clause_checks`、`…one_write_accounting`、`…record_checker_reuse_legality`、bin `first_transaction_on_device`、`first_transaction_device_log_check`。
- `tests/second_transaction_supplement_three_crash_injection.rs`：快档 `7 passed; 1 failed`，新发现 7 个；开工快照原件上跑同一条，同样 7 个、同种子同段号 ⇒ 不是这一轮带出来的。
- 没跑、推会红的层 0 文件（都拿 `closed_form_state_count(&expanded)` 或写死的数比 `tally.states`，展开的段里有系统配置槽写就红）：`second_transaction_step_three_formatted_pool_layer0.rs`（钉 22）、`…step_three_acquisition_barrier_layer0.rs`、`…position_addressed_trees_layer0.rs`、`…parallel_line_one_layer0.rs`、`…parallel_line_three_spill_over_layer0.rs`、`…supplement_two_tree_split_layer0.rs`；崩溃枚举用例 `second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`（ignore，`crash-case:floor-raise-pushed-by-the-session`：状态数比两态闭式，观察者把持久集合与录制写表 zip，撕裂镜像被截掉）。改法：比 `layer0_state_count_with_torn_in_place_overwrites(base, writes, segments, 那条用例的展开)`。

## 九、kb 要跟着改的句子（我写不了 kb）

D13 已定项 4（`.claude/kb/decisions/13-验证路线.md` 第 71 行）新定案句：
> **定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流按设备切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段（A 合并：不漏可达态，多出不可达态）。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西）多取第三态「新旧都读不出」，全量与快档都枚举；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式（每段 3^m · 2^(n−m) − 1，m 是段里原地覆写的写数）。

同一处要跟着看的：第 12 行索引表那一格（「屏障与 FUA 写切段、段内任意整写子集；撕裂态并进「没持久」…」）；第 73 行射程「撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率」对原地覆写不再成立（旧内容也没了）；第三节 A（撕裂镜像的粒度）定了之后第三态的镜像写法也进这一句。

`.claude/kb/layout/01-first-txn.md` 第八节（门禁 52 号核）：
- mkfs 那一行：`21 次操作` → `23 次操作`，「两道池屏障」后加「（每道两块盘各记一步）」，种类串 → `[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]`（52 号拿它比 step_one 用例里的串）。
- 暖机、普通发布两行与「整条流」那句的种类串与操作数（新值：暖机 `[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]`；普通发布 33 次操作、`[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]`；整条流 53 次操作、每段 `barrier` → `barrier×2`）要等 E142 重出产物再改（52 号拿这几行比产物；产物重出之前改了就红）。状态数这几句照旧写两态闭式（52 号按 1 + Σ(2^|段| − 1) 核），各加一句「层 0 按原地覆写三态枚举：整条流 150994980、快档 54」。
- 第二条流那句：今天的用例是 64 段、6649413719（两态）、快档 205，kb 还写 61 段、6649413746、232（A1 那一轮的改动，不是这一轮的）；新句按 64 段那组数写两态闭式，另加「层 0 三态枚举 14960689284、快档 390；到 C 27 段 202113066（三态 454426691）、到 D 30 段 206307373（三态 463863878）」。

## 十、`git diff --stat -- crates litmus`（原样；工作区里同时有别的会话的改动，分不出谁的，我写过的以第二节为准；新用例文件未跟踪、不在表里）

```text
 crates/mutations.tsv                               |  620 +-
 crates/singlefs-checker/src/image.rs               |  216 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1911 +++--
 crates/singlefs-core/src/admission.rs              |  272 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  132 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   24 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |  179 +-
 crates/singlefs-core/src/mount.rs                  | 2557 +++++--
 crates/singlefs-core/src/mounted_read.rs           |   22 +-
 crates/singlefs-core/src/recovery.rs               |  493 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  433 +-
 crates/singlefs-core/src/write_request_split.rs    |   51 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 6818 +++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 2498 ++++++-
 crates/singlefs-harness/src/crash_injection.rs     |  821 ++-
 crates/singlefs-harness/src/device_log.rs          |   38 +-
 crates/singlefs-harness/src/fault_injection.rs     |  749 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             | 1122 ++-
 crates/singlefs-harness/src/lib.rs                 |   60 +-
 crates/singlefs-harness/src/model.rs               |  964 ++-
 crates/singlefs-harness/src/model_comparison.rs    |  338 +-
 crates/singlefs-harness/src/on_device_modes.rs     |    4 +-
 crates/singlefs-harness/src/read_tally.rs          |    2 +-
 crates/singlefs-harness/src/segments.rs            |  106 +-
 .../tests/checker_known_bad_images.rs              |  986 ++-
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/common_tree_split/mod.rs                 |  138 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 .../tests/first_transaction_step_five_publish.rs   |   22 +-
 .../tests/first_transaction_step_one_mkfs.rs       |    8 +-
 .../tests/first_transaction_step_seven_layer0.rs   |  233 +-
 .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 +-
 .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
 ...action_step_three_acquisition_barrier_layer0.rs |   29 +-
 ...second_transaction_step_three_formatted_pool.rs |  567 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  663 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  658 +-
 ...transaction_supplement_three_fault_injection.rs |  412 +-
 ..._transaction_supplement_three_random_history.rs |  434 +-
 ...transaction_supplement_two_admission_formula.rs |  342 +-
 ...ent_two_c519_whole_device_loss_after_warm_up.rs |   29 +-
 ...two_c533_row_publish_record_without_its_root.rs |    7 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   26 +-
 ...rop_and_devices_without_the_selected_version.rs |  258 +-
 ...nsaction_supplement_two_instance_table_chain.rs |  100 +-
 ...tion_supplement_two_instance_table_page_full.rs |   77 +-
 ...plement_two_instance_table_second_page_write.rs |    4 +-
 ..._two_multi_record_transaction_zero_publishes.rs |    5 +-
 ...action_supplement_two_presumed_clause_checks.rs |  105 +-
 ...plement_two_publish_failure_resent_unchanged.rs |    2 +-
 ...n_supplement_two_release_checksum_quarantine.rs |   88 +-
 ..._transaction_supplement_two_rollback_witness.rs | 1079 ---
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 --
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    2 +-
 ...nt_two_row_publish_checks_before_acquisition.rs |    2 +-
 ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |   10 +-
 ...second_transaction_supplement_two_tree_split.rs |  147 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 87 files changed, 28976 insertions(+), 11040 deletions(-)
```

## 十一、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 crash-verifier；没提交。
- 名字带 layer0 的目标一个没跑：两份 layer0 文件的钉值只编过、没跑；第七节里恢复计数那几项是推的。
- 没改 kb（第九节给句子）、没改清单外的文件（第八节列会红的）、E142 两个 bin 没改、E142 产物没重出。
- `crates/mutations.tsv` 没追加（18 行在 `mutations-append.tsv`，交主 agent 追加）；859 行没证红（点名的用例被我的改动带红，要等那份文件改钉值）。
- 第八节那批清单外测试二进制、74 号的对照是为报告查波及面跑的，超出「只跑动到的测试二进制」；全量 `cargo test` 与 `check.sh` 整轮没跑。
- 草稿副本已删：`/tmp/claude-1000/impl-rev-b3a2/copy-work`（15G，含它自己的 target）、`/tmp/claude-1000/impl-rev-b3a2/copy-radius`（4.5G）。留在草稿目录（2.8M，都是草稿、没入库，这一轮没有实验产物）：`originals/`（开工时我那几份文件的快照）、`draft/`（新用例各段、探针源码）、`mutations/`（改行与追加的脚本）、`mutations-append.tsv`、`prove-red-logs/`、`radius-logs/`、各次日志与 `progress.md`。
