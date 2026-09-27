# 实审 B3a 报告（implementation-writer，2026-09-26 UTC 写）

## 一、结论

| 条 | 结局 | 一句话 |
|---|---|---|
| 1 录制器合并屏障、切段不分设备 | **停下交主 agent**，没改代码 | 审阅判得对（草稿演示用例红，见第三节）；按规格实现有一处设计岔路没定、且会波及派发清单外的 5 份文件 |
| 3 FUA 按池关段 | **停下交主 agent**，没改代码 | 审阅判得对（演示用例红）；与第 1 条同一套按设备切段，卡在同一个岔路 |
| 10 稀疏盘不判越界 | 已改 + 新用例 | `SparseBlockDevice` 读 / 写 / 清零按文件后端同一条判据报 `OutOfRange` / `Unaligned`；`MemoryPool`、`CrashImage` 两个读口子越界、没对齐、没这块盘都答 None |
| 14 前一半 并片单测 | 已加用例 | 两片的「走读失败」「验证失败」都非 0，钉住相加 |
| 14 后一半 journal 提示 | 已改 + 新用例 | 崩溃镜像的提示改成与基镜像同一套「写过的扇区 → 对齐的记录槽」，与全环扫描在同一批镜像上逐项比 |
| 4 原地覆写第三态 | 只量，没实现 | 第一条流全量 67108885 → 150994980、甲二 29 → 54；第二条流全量 6649413719 → 14960689284、甲二 205 → 390（第四节） |

推翻条件：主树上 `cargo test -p singlefs-harness --test <三个新目标>` 或 `--lib` 任一条红；变异表新增行在门禁 59 号上有一条不红；第 1、3 条的岔路主 agent 另有定法（那样第三节的波及清单要按定法重列）。

## 二、这一轮写过的文件

- `crates/singlefs-harness/src/crash.rs`：新增 `check_request_against_the_device_contract`、`sector_as_the_physical_block_size`；`SparseBlockDevice` 加 `check_request`，`read_at` / `write_at` / `write_zeroes_at` 先核请求；`impl PoolReader for MemoryPool` 的 `read` 核请求；`impl ImageReader for MemoryPool` 的 `device_bytes` 改成池里没有的盘答 None；`impl PoolReader for CrashImage` 的 `journal_record_offsets_hint` 改算法；`mod tests` 新增 `absorbing_a_following_slice_adds_the_failed_and_the_verification_failed_states_of_both_slices`；多一行 `use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};`。枚举、分片、并片的行为没动。
- 新文件 `crates/singlefs-harness/tests/sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs`（第 10 条，2 条用例）
- 新文件 `crates/singlefs-harness/tests/crash_image_journal_hint_matches_the_full_ring_scan.rs`（第 14 条后一半，3 条用例）
- 新文件 `crates/singlefs-harness/tests/in_place_overwrite_torn_state_count.rs`（第 4 条量数，3 条用例）
- `crates/mutations.tsv`：末尾追加 15 行（名字见第五节）
- `crates/singlefs-harness/src/lib.rs`、`segments.rs`：**没动**（第 1、3 条停下）

## 三、第 1、3 条：停下交主 agent

### 3.1 审阅判得对（草稿副本里的演示用例在今天的代码上红）

演示用例 `/tmp/claude-1000/impl-rev-b3a/demo/red_demo_items_one_and_three.rs`（只在副本 copy-base 里跑，不进仓）。命令：
`bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 cargo test --offline --manifest-path /tmp/claude-1000/impl-rev-b3a/copy-base/Cargo.toml -p singlefs-harness --test red_demo_items_one_and_three`，原样输出摘自 `demo/red-demo-items-1-3.log`：

```text
test item_one_a_device_that_misses_its_barrier_records_a_different_stream ... FAILED
test item_three_a_fua_on_device_zero_does_not_order_the_plain_write_on_device_one_before_later_writes ... FAILED
assertion `left != right` failed: 盘 1 漏发屏障的录制流要与每盘都发屏障的不同
要枚举得出「盘 0 后一次写已落、盘 1 那次写没落」：盘 1 那次写不能在更早的段里（段：[[0, 1], [2]]）
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

第 1 条：两块盘各写一次 → 盘 0、盘 1 各一道屏障 → 盘 0 根槽 FUA；把盘 1 那道吞在录制器外面，录下来的 4 步逐项相同（`left` 与 `right` 两串一字不差，都只有一条 `device: DeviceIdentity(0), kind: Barrier`）。第 3 条：`[盘 1 普通写, 盘 0 FUA, 盘 0 普通写]` 切成 `[[0, 1], [2]]`，盘 1 那次写被当成在盘 0 后一次写之前已持久。

### 3.2 卡在哪：一处设计岔路没定

规格定的次序是「一次写只被它自己那块盘上之后的屏障排在后面的写之前」（FUA 同理，只放行它那块盘）。这个次序一般不是「段的链」：盘 0 发了屏障、盘 1 没发时，盘 0 屏障前的写 < 屏障后的写，而盘 1 屏障前的写与两者都不比。今天的枚举器（`Layer0StatePlan`，前若干段全持久 + 当前段子集）只吃段的链，而实分一刚改过、规格说别动，所以只能把这个次序近似成一条链，两种近似各丢一边：

| 读法 | 怎么切 | 多出 / 漏掉的状态 | 与规格两句的关系 |
|---|---|---|---|
| A 合并 | 当前段里每块有写的盘都被它自己的屏障或 FUA 放行了才关段；有一块没放行，这道屏障不关段，前后并成一段 | **不漏**任何可达态；多出不可达态（盘 0 屏障后的写落了、屏障前的没落） | 「那块盘上屏障前的写与之后的写同段」成立；但丢了「盘 0 的写被盘 0 屏障排在前」这一半 |
| B 顺延 | 屏障只关已放行那几块盘的写，没放行那块盘的写挪进下一段 | 不多出；**漏**可达态（盘 1 屏障前的写已落、盘 0 屏障前的写没落全） | 字面更贴「那块盘上屏障前的写」；但凭空加了「盘 0 屏障前的写 < 盘 1 屏障前的写」 |

每盘都发屏障时 A、B 与今天切出的段、状态数逐字相同。我倾向 A（层 0 的用处是不漏可达态），但这是枚举域的定义，按定义不自己定。第 3 条同一个岔路：`[盘 1 写, 盘 0 FUA, 盘 0 写]` 按 A 是一段 3 写，按 B 是 `[盘 0 FUA] | [盘 1 写, 盘 0 写]`。
判别力用例按 A 要挑小：只吞盘 1 在一处的屏障（整条都吞时 A 把第一条流 mkfs 之后并成一段 36 写，甲二 32767 个状态、全量 2^36）；按 B 整条吞盘 1 屏障时最后一段 18 写，甲二 127 个状态。以上段长是按两种切法手推的，没跑。

### 3.3 按设备录屏障（不合并不同设备的屏障）会波及派发清单外的文件

录制流里一道池屏障从 1 步变成盘数步，下面这些钉的是今天「池屏障记一步」的形状，都不在「要动的 crates 文件」里（行号 `grep -n` 现取）：

| 文件:行 | 今天钉的 | 按设备录之后（手推，没跑） |
|---|---|---|
| `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:184` | `mkfs_operation_count + 3`（取号两写 + 一道屏障） | + 4 |
| 同文件 `:327`、`:351`、`:363`、`:375` | mkfs 21 步、暖机 14 步、第一个事务 31 步、mkfs 之后整条 47 步 | 23、18、33、53（各多出池屏障组数：2、4、2、6） |
| `crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs:287`、`:292` | mkfs 21 次操作、`StreamIntegrity::Consistent { operations: 21 }` | 23 |
| `crates/singlefs-harness/src/device_log.rs:148`、`:158` | 「每道池屏障在每块盘上各是一个 FLUSH（…录制器把连续几道并成一道）」：每条屏障步给每块盘都推一个 FLUSH | 每块盘多推一个 FLUSH，门禁 55 号拿 QEMU 设备日志对它比，要改成只推给 `operation.device` 那块盘 |
| `crates/singlefs-harness/src/bin/e142_first_transaction_write_dump.rs:202`、`e142_first_transaction_write_dump_one_device.rs:239` | 每条屏障步打一行 `name=window_barrier` | 行数变多；`research/scripts/replay.sh:443` 复跑 E142 的实现快照会与入库产物对不上 |
| `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs:680`、`:696`（层 0，A1 在改） | 摘掉根槽前最后一步屏障，段 `[2, 2, 1, 2, 2, 1, 26, 3, 2]` | 只摘掉盘 1 那一道：按 A 末两段并成 `26, 5`，按 B 成 `26, 1, 1, 3`，计数跟着变 |

另有一处要一起定：`segments.rs` 的种类串按步数数屏障（`barrier×N`），按设备录之后每段的 `barrier` 个数翻倍，登记表（`.claude/kb/layout/01-first-txn.md` 八）、门禁 52 号与上面两份用例钉的种类串全变；要保持不变，就得在种类串里把连续几步屏障记成一个（一道池屏障一个 `barrier`），这也是一处写法选择。

### 3.4 不波及那几份文件的另一种录法（供主 agent 比）

录制器照旧把一道池屏障记一步，但一组连续屏障没罩住「当前有写、还没被放行的每块盘」时整组不记（盘 1 漏发时，录制流在那一处就少一道屏障）；切段不变。等价于 A 读法，池屏障的步数、种类串、上表每一处都不动。代价：不是「按设备记屏障」，盘 0 那一道也从流里消失；与规格字面不符，按定义不自己选。FUA 那一半（第 3 条）两种录法都要改切段：FUA 只在当前段别的盘都已放行时关段。

### 3.5 D13（验证路线） 已定项 4 要跟着改的句子（`.claude/kb/decisions/13-验证路线.md` 第 71 行）

原句（第 71 行整行）：

> **定案**：崩溃点重放要枚举的崩溃状态集合定义为：屏障把录下来的写请求流切成段，一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写也是段边界：它关掉它所在的那一段，与前面没被屏障隔开的写同段，它之后的写不与它同段。撕裂子集**不枚举**，一次写的任何撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像要真的生成出来喂给 checker，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。

新句（只换第一、二句；按 A 读法写，定成 B 时换括号里那一句）：

> **定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流按设备切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段关上，有一块盘还没放行，这道屏障不关段、前后的写同段（B 读法：只关已放行那几块盘的写，没放行那块盘的写并进下一段）。一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。

## 四、第 4 条：补第三态的状态数（只量，没实现）

原地覆写的写怎么认（`tests/in_place_overwrite_torn_state_count.rs` 的 `is_tearable_in_place_overwrite`，三条都成立才取 3 态）：
1. 不是单元写（单元写 COW，落在分配器给的槽上，旧内容没人要，照旧 2 态，与用户定案「COW 写照旧」一致）；
2. 长于一个扇区（层 0 的撕裂粒度是 512 字节扇区，一扇区的写撕不开：根槽写就是一扇区）；
3. 罩住的范围里原来有东西：基镜像那一段有非零字节，或写表里更早有一次同一块盘上、带字节（不是整段清零）的写与它重叠。

每段 n 写、其中 m 个取 3 态：全量 3^m·2^(n−m) − 1；甲二（原地写 k 个其中 m 个取 3 态、单元写 c 个）3^m·2^(k−m)·(c>0 ? 2 : 1) − 1；整条流再加 1。m 全为 0 时两式等于今天的闭式（用例 `without_any_tearable_overwrite_the_counts_are_todays_closed_forms` 拿 `crash::closed_form_state_count`、`crash::layer0_state_count` 对拍）。

| 流 | 写 / 段 | 取 3 态的写（按种类：总数 / 取 3 态） | 全量 今天 → 补第三态 | 甲二 今天 → 补第三态 |
|---|---|---|---|---|
| 第一条流（第一个事务，mkfs 之后） | 41 / 10 | 系统配置槽 8 / 8；journal 6 / 0；根槽 3 / 0；单元 24 / 0 | 67108885 → 150994980 | 29 → 54 |
| 第二条流（到 E 再正常卸载） | 477 / 64 | 系统配置槽 46 / 46；journal 38 / 0；根槽 19 / 0；单元 374 / 0；清零 0 / 0 | 6649413719 → 14960689284 | 205 → 390 |

原样输出：第一条流是主树上 `cargo test -p singlefs-harness --test in_place_overwrite_torn_state_count -- --nocapture` 打的（`run-main-in_place_overwrite_torn_state_count.log`）：
```text
MEASURE stream=first_transaction writes=41 segments=10 tearable_by_kind=[("system_configuration_slot", 8, 8), ("journal_record", 6, 0), ("root_record_fua", 3, 0), ("unit_write", 24, 0)] today_full=67108885 torn_full=150994980 today_quick_tier=29 torn_quick_tier=54
```
第二条流是草稿副本 copy-base 上的探针打的（`demo/probe_second_stream_torn_state_count.rs`：把 `tests/second_transaction_step_zero_layer0.rs` 整份拷成一个名字不带 layer0 的目标、摘掉原有的全部 `#[test]`、只调 `prepare(ReuseAfterRaisingFloorThenNormalUnmount)` 再算数，不枚举一个状态；日志 `demo/probe-second-stream.log`）：
```text
MEASURE stream=second script=through_e_then_unmount writes=477 segments=64 sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 2, 2, 18, 2, 1, 18, 2, 1, 18, 2, 1, 26, 2, 1, 22, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 2, 2, 16, 2, 1, 18, 2, 1, 30, 2, 1, 2, 2, 16, 2, 1, 18, 2, 1, 2] tearable_per_segment=[2, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 2, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 2] tearable_by_kind=[("system_configuration_slot", 46, 46), ("journal_record", 38, 0), ("root_record_fua", 19, 0), ("unit_write", 374, 0), ("zero_fill", 0, 0)] tearable_journal_offsets=[] today_full=6649413719 torn_full=14960689284 today_quick_tier=205 torn_quick_tier=390
```
口径：第二条流是这一轮工作区（含 A1 未提交的第 19 条改动，64 段）的样子，不是 kb 里登记的 61 段、6649413746；A1 改完之后要按这个探针重量。两条流里取 3 态的都只有系统配置槽写：journal 记录全写在 mkfs 清过的环里、没有回绕，「覆盖旧记录的 journal 写」这两条流一次都没有（`tearable_journal_offsets=[]`），第三态在 journal 上的那一半层 0 这两条流走不到。全量两条都约 ×2.25（来自单元写大段里那 2 个系统配置槽写：9/4）。

## 五、每条新测试的证红

草稿副本 `/tmp/claude-1000/impl-rev-b3a/copy-mutate`（主树改完之后 `rsync -a --exclude target --exclude .git` 拷的，自己的 target）。先跑不改动的基线四个二进制：`--lib` 86 passed、三个新目标 2 / 3 / 3 passed，基线红集为空（`proofs/baseline-1..4.log`）。再逐条施加一处变异、跑那条测试所在的整个测试二进制、拷回原件并 `touch`（跑完两份被改过的文件 sha256 前 16 位与原件相同：crash.rs `c9f9603478feb8c0`、量数用例 `2ee35ffd04b09589`）。驱动脚本 `run_proofs.sh`，表 `mutation-proofs.tsv`，每条的日志 `proofs/proof-N.log`。

| 证 | 改坏哪一行 | 哪条断言红（原样） | 同时红的 |
|---|---|---|---|
| 1 | `crash.rs` `SparseBlockDevice::write_at` 的 `self.check_request(offset, u64::try_from(bytes.len())…)?;` 改成 `let _ = …;` | `a_sparse_block_device_accepts_and_refuses_every_request_exactly_like_the_file_backend`：`assertion left == right failed: 写：偏移 1048064 长度 1024 / left: Accepted / right: OutOfRange { offset: DeviceOffsetInBytes(1048064), length: 1024, device_size: 1048576 }` | 无（同二进制另一条绿） |
| 2 | `crash.rs` `impl PoolReader for MemoryPool` 的 `read` 里 `.ok()?;` 改成 `.ok().or(Some(()))?;` | `memory_pool_and_crash_image_readers_answer_none_outside_a_device_like_a_pool_of_real_devices`：`MemoryPool 的 PoolReader：盘 0 偏移 1048064 长度 1024 / left: Some([0, 0, …]) / right: None` | 无 |
| 3 | `crash.rs` `absorb_following_slice` 的 `self.failed_states += failed_states;` 改成 `=` | `absorbing_a_following_slice_adds_the_failed_and_the_verification_failed_states_of_both_slices`：`走读失败 2 + 5、验证跑过 4 + 8、验证失败 3 + 7：两片逐项相加 / left: (15, 5, 12, 10) / right: (15, 7, 12, 10)` | 无（--lib 85 passed、1 failed） |
| 4 | `crash.rs` `CrashImage::journal_record_offsets_hint` 的 `offsets.extend(record_slot_offsets(…));` 改回 `offsets.push(write.offset);` | `a_record_written_off_the_record_slot_grid_is_read_by_neither_scan`：`一条记录错开一个扇区写：按提示扫与全环扫描读出的记录要逐条相同 / left: {(InstanceGeneration(1), 1): JournalRecord {…}} / right: {}` | `one_write_covering_two_record_slots_hands_both_slots_to_the_scan`（按提示少读一条）；段边界那条绿 |
| 5 | 同函数循环头 `if !is_persisted \|\| write.device != device {` 改成 `if true \|\| …`（提示只剩基镜像那一份） | `on_the_first_transaction_stream_the_hint_finds_what_the_full_ring_scan_finds_in_every_compared_state` 红（3 条全红，日志里先打出来的是另两条的 `left: {} / right: {(InstanceGeneration(1), 1): …}`） | 同二进制另两条 |
| 6 | 量数用例 `segment_state_count_with_the_torn_third_state` 里 `copy_on_write_choices` 没有单元写时的 `1` 改成 `2` | `without_any_tearable_overwrite_the_counts_are_todays_closed_forms`：`left: 18 / right: 12` | 另两条 |
| 7 | 量数用例里取 3 态那一臂 `3u128` 改成 `2u128` | `a_tearable_in_place_overwrite_takes_a_third_state_and_every_other_write_two`：`left: 18 / right: 27`；`the_first_transaction_stream_counts_with_the_torn_third_state_as_measured`：`left: (67108885, 29, 29) / right: (150994980, 29, 54)` | 无（第一条对拍用例绿） |

新测试 9 条，每条至少被上表一处变异证红。`crates/mutations.tsv` 末尾用 `research/scripts/insert-row.py` 追加 15 行（逐行 `--after` 上一行、`--absent` 同名），变异名：
- 证过（上表 1–7 与同一处变异点名另一条测试、已在同一次运行里看到红的两行）：「代码审阅第 10 条（实审 B3a）：稀疏盘写不核请求，越界与没对齐的写照收」「代码审阅第 10 条（实审 B3a）：内存镜像读不核请求，越界读给 Some(全 0)」「代码审阅第 14 条（实审 B3a）：并片时走读失败的状态数不相加（取后面那一片的）」「代码审阅第 14 条（实审 B3a）：崩溃镜像的 journal 提示给写的起点、不按记录槽对齐过滤（错开一个扇区的记录按提示读得到）」「代码审阅第 14 条（实审 B3a）：崩溃镜像的 journal 提示给写的起点、一次写罩两个记录槽只给第一个」「代码审阅第 14 条（实审 B3a）：崩溃镜像的 journal 提示只给基镜像那一份（这一状态落了的记录按提示读不到）」「代码审阅第 4 条量数（实审 B3a）：甲二快档没有单元写的段也乘 2（与 harness 自己的闭式对不上）」「代码审阅第 4 条量数（实审 B3a）：原地覆写的写也只取 2 态」「代码审阅第 4 条量数（实审 B3a）：原地覆写的写也只取 2 态（第一个事务那条流的量数）」
- 留给门禁 59 号：「代码审阅第 10 条（实审 B3a）：稀疏盘读不核请求，越界读给全 0」「代码审阅第 10 条（实审 B3a）：稀疏盘清零不核请求，越界清零照收」「代码审阅第 10 条（实审 B3a）：请求判据里偏移加长度按回绕算，溢出 u64 的请求当成在盘内」「代码审阅第 10 条（实审 B3a）：内存镜像的 ImageReader 对池里没有的盘也报字节数」「代码审阅第 14 条（实审 B3a）：并片时 journal 验证失败的状态数不相加（取后面那一片的）」「代码审阅第 4 条量数（实审 B3a）：一个扇区的写也算撕得开（根槽写成了原地覆写）」

## 六、交回前的验证（第 4 步，末尾原样）

负载：开跑前 `ps` 看到别的会话的 `cargo test`（random_history、checker_cross_links、writable_mount…）与 e158 的 release 构建，没有 qemu / vm-bench / e152 / fio；全程 `nice -n 19` + `capped.sh 4`，没记到等锁。

- `cargo fmt -p singlefs-harness -- --check`：
```text
exit=0
```
- `cargo build --offline --all-targets -p singlefs-harness`：
```text
   Compiling singlefs-harness v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.79s
exit=0
```
- `cargo clippy --offline --keep-going -p singlefs-harness --all-targets -- -D warnings`＋check.sh 那 7 条 lint：红，全在别的会话的两个 bin（`src/bin/e156_allocation_basis_counts.rs` 6 处、`src/bin/e158_root_choice_repair.rs` 8 处，都是 `shadow_unrelated`），我动的文件零条：
```text
error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts") due to 3 previous errors
error: could not compile `singlefs-harness` (bin "e158_root_choice_repair" test) due to 4 previous errors
error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts" test) due to 3 previous errors
error: could not compile `singlefs-harness` (bin "e158_root_choice_repair") due to 2 previous errors
exit=101
```
- 动到的测试二进制（主树，经 `run-with-memory-cap.sh 8G` + `capped.sh 4`）：
```text
test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 176.51s
exit=0
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.32s
exit=0
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.93s
exit=0
```
- 登记给我的门禁阶段（`nice -n 19 bash .claude/gate.d/<阶段>`，末行与退出码）：
```text
[33-mutation-tables.sh]
  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 898 条的原文各命中源码一次；没有同名实验二进制、锚点不在本阶段射程的表 5 张：research/mutations/e125_zoned_wp.tsv research/mutations/e129_tear_injector.tsv research/mutations/e129_thin_neighbour.tsv research/mutations/e129_thin_rmw.tsv resear
exit=0
[53-format-const-placeholders.sh]
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
exit=0
[74-model-differential.sh]
                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。
exit=1
[92-layout-checker-sync.sh]
      第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）：.claude/kb/layout/02-second-txn.md
exit=0
[94-checker-implementation-disjoint.sh]
    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑
exit=0
[93-feature-bits.sh]
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
exit=0
[89-closeout-row27-preconditions.sh]
    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐
exit=77
```

74 号红不是这一轮带进来的：它红在 `tests/second_transaction_supplement_three_random_history.rs` 的 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`（第 418 行那条断言「覆盖写、第一个文件一次都没被「每块盘上都没有」拒过」）。同一条用例在我动手之前拷的副本 copy-base 上照样红（`wall-base.log`：`test result: FAILED. 1 passed; 1 failed; …`，同一句断言）。看报出的拒绝成员，会话推抬 F 时的拒绝现在包成 `UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(…)`（主树这一次 97 + 69 + 159 + 132 次），用例只数 `NoSpaceAfterRaisingTheFloor(…)` 那一包——像是 A1 第 23 条改了成员名、这条用例没跟上（推的，没核 A1 的改动）。89 号退 77（本次未跑）：它自己报的是「alloc-basis 第三轮」清单条数对不上，与这一轮无关。

## 七、`git diff --stat -- crates litmus`（原样；工作区里同时有 A1、B1、B3b、E156 / E158 的未提交改动，分不出是谁的，我写过的文件以第二节为准；三个新文件未跟踪，不在这张表里）

```text
 crates/mutations.tsv                               |  566 +-
 crates/singlefs-checker/src/image.rs               |   77 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                |  847 ++-
 crates/singlefs-core/src/admission.rs              |  272 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  132 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   10 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |   11 +-
 crates/singlefs-core/src/mount.rs                  | 2182 ++++--
 crates/singlefs-core/src/mounted_read.rs           |    7 +-
 crates/singlefs-core/src/recovery.rs               |  413 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  433 +-
 crates/singlefs-core/src/write_request_split.rs    |    2 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 3339 ++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 1633 ++++-
 crates/singlefs-harness/src/crash_injection.rs     |  820 ++-
 crates/singlefs-harness/src/device_log.rs          |    2 +-
 crates/singlefs-harness/src/fault_injection.rs     |  749 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             | 1083 ++-
 crates/singlefs-harness/src/lib.rs                 |   44 +-
 crates/singlefs-harness/src/model.rs               |  964 ++-
 crates/singlefs-harness/src/model_comparison.rs    |  338 +-
 crates/singlefs-harness/src/on_device_modes.rs     |    4 +-
 crates/singlefs-harness/src/read_tally.rs          |    2 +-
 crates/singlefs-harness/src/segments.rs            |    2 +-
 .../tests/checker_known_bad_images.rs              |  805 ++-
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/common_tree_split/mod.rs                 |  138 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 .../tests/first_transaction_step_seven_layer0.rs   |  156 +-
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
 ...second_transaction_step_three_formatted_pool.rs |  566 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  563 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  196 +-
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
 85 files changed, 22084 insertions(+), 10583 deletions(-)
```

## 八、停下交主 agent 的问题

1. 第 1、3 条的切段读法：A 合并（不漏可达态、多出不可达态）还是 B 顺延（不多出、漏可达态），第三节 3.2。
2. 第 1 条的录法：按设备逐步记（要连带改第三节 3.3 那 5 份文件、种类串里屏障怎么数也要定），还是 3.4 那种「没罩全就整组不记」（不波及，但与规格字面不符）。定了之后派发清单要把 5.3 里要动的文件加进去；`first_transaction_step_seven_layer0.rs` 是层 0 文件、A1 在改。
3. 顺带看到、没改（不在这一轮条目里）：`crash.rs` 里 `impl ImageReader for CrashImage` 的 `candidate_journal_slots`（第 690 行起）与第 14 条后一半同形——给 checker 的候选记录槽用 `(write.offset.0 - ring_start) / JOURNAL_RECORD_BYTES` 向下取整（第 700 行），一次写罩两槽只给第一个；环长写死 `JOURNAL_RING_DEFAULT_BYTES`（第 656、698 行），不读池的实际环长。
4. 第 10 条的判据在 harness 里照 core `block_device.rs` 私有的 `check_aligned_and_in_range` 写了一份（两边对拍用例钉住）；把 core 那一份改成 `pub` 就能删掉这一份，core 不在这一轮的文件里。

## 九、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交。
- 第 1、3 条没改代码（停下，第三节）；`lib.rs`、`segments.rs` 一字没动；kb 句子只给了原句与新句（3.5），没写 kb。
- 名字带 layer0 的测试目标一个没跑。改了 `SparseBlockDevice`、`MemoryPool`、`CrashImage` 的行为（越界与没对齐从照收 / 给全 0 / panic 变成报错 / None），用到它们的别的测试二进制（随机历史、坏盘输入、各层 0 用例等）我没跑，留给提交时的整轮验证；它们只在设备之外的请求上行为不同，在盘内的请求逐字不变。
- 第二条流的量数是这一刻工作区（A1 未提交）的流；A1 改完要重量（探针源码在 `demo/probe_second_stream_torn_state_count.rs`）。
- 74 号那条红与 clippy 在 e156 / e158 上的红没修（不是这一轮的文件）。
- 草稿副本已删：`/tmp/claude-1000/impl-rev-b3a/copy-base`（1.9G，含它自己的 target）、`/tmp/claude-1000/impl-rev-b3a/copy-mutate`（2.1G，含它自己的 target）。演示用例、探针源码与各次日志留在 `/tmp/claude-1000/impl-rev-b3a/demo/`、`proofs/` 与草稿目录根下，供主 agent 核；它们是草稿、没进 `research/results/`（这一轮没有实验产物要入库）。
