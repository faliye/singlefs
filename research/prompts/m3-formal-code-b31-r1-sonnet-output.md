# m3-formal-code-b31-r1 云端正推腿报告（Sonnet，三方正推，N1–N5）

立场（正文第四节「云端正推」）：从用户 2026-10-05 定案、kb 条款与 E171 跑前登记修订 6 正推「每处改动应该是什么」，与冻结副本 `/tmp/claude-1000/m3-formal-code-b31-r1/tree/` 里今天的 `crates/` 逐格比；一样写一样，不一样把两边整行并排抄出来。没看别的腿的报告，没读禁读清单里的文件。

## 0. 实现今天的样子（我的观测，不是背景材料转述）

- 读过的实现：`crates/singlefs-core/src/{allocator.rs,make_filesystem.rs,mount.rs,transaction.rs,root_ring.rs}`、`crates/singlefs-harness/tests/` 里被改的 11 份与 `common/mod.rs`、`crates/singlefs-checker-tier/src/on_device_modes.rs`、E170 / 旧 E171 / E156 / 新 E171 装置，行号都是冻结副本里的。
- 能核的事实我自己复跑了：harness 档与 core 单测在草稿副本 `/tmp/claude-1000/m3-formal-code-b31-r1-sonnet/tree/` 里编跑（冻结副本 rsync，排除 target）；checker 档（装置内联用例、变异整表、QEMU、实验复跑）按派发一条没跑，涉及它们的结论写「推的」。
- 复跑入口 `research/prompts/m3-formal-code-b31-r1-sonnet-model/tools/rerun.sh`，原样输出在同目录 `out/`（README.md 有文件表）。
- 草稿副本里加过的东西（探针、变异、`scenario.rs` 一个私有函数改 pub）都只在草稿副本，仓里与冻结副本一个字没动。

## N1　core 断言

**判定：一致。** 断言条件恰好是「没装根环表而记了一条根」；我没有找到一条产品路径在装之前记根；`expect` 消息与 `# Panics` 写清了依赖。两处观察不改判定。

**1.1 条件。** core 的断言就是对 `root_ring` 取 `expect`：「let occupancy = self.root_ring.as_mut().expect(」（`crates/singlefs-core/src/allocator.rs:1919`）。紧接着的跳号断言没变：「let expected_txg = CheckpointTxg(occupancy.latest_recorded_txg.0 + 1);」（`crates/singlefs-core/src/allocator.rs:1924`）。

**1.2 `expect` 消息与 `# Panics`。** 消息点名出路与后果：「分配器没装根环表却记了一条根：写根的分配器都由建它的一方装上根环表」（`crates/singlefs-core/src/allocator.rs:1920`）；`# Panics` 一节第一条：「分配器没装根环表（`install_root_ring_occupancy`）：不装就一次不回收」（`crates/singlefs-core/src/allocator.rs:1913`）。消息里带 C551 与「转过根环之后 I-3.1 红」，依赖的是 I-3.1（已分配统计对得上）与可再分配谓词，没有另写 I-5.4 / I-7.4（它们是回收错了才红的那几条，不是漏装的直接后果）。够用。

**1.3 合法路径逐类过一遍（读码 + grep）。** 命令（冻结副本 `crates/` 下，原样输出，已去掉 `allocator.rs` 自己的行）：
```
$ grep -rn 'record_root_written_by_this_process\|record_zero_unit_roots_leading_to' crates --include=*.rs | grep -v ':\s*///' | grep -v allocator.rs
crates/singlefs-checker-tier/src/on_device_modes.rs:232:        allocator.record_root_written_by_this_process(root.checkpoint_txg);
crates/singlefs-core/src/mount.rs:1368:pub fn record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount<
crates/singlefs-core/src/mount.rs:1380:    allocator.record_root_written_by_this_process_rereading_slots_bad_at_mount(
crates/singlefs-core/src/mount.rs:3688:            copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
crates/singlefs-core/src/mount.rs:3744:                copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
crates/singlefs-core/src/transaction.rs:5713:    allocator.record_zero_unit_roots_leading_to(version_to_build_on.checkpoint_txg);
（另 mount.rs:3595、4826、transaction.rs:1846、7847 是 1368 那个函数的调用点，注释行省略）
```
| 路径 | 记根之前装了吗 | 依据 |
|---|---|---|
| 挂载重建 | 装了 | 产品里除 mkfs 外建分配器只有 `rebuilt_allocator` 一族：「let mut allocator = PoolAllocator::rebuild_from_records(device_maps, tree.records.clone());」（`crates/singlefs-core/src/mount.rs:2168`）与「let mut allocator = PoolAllocator::new(device_maps);」（`crates/singlefs-core/src/mount.rs:2250`，`format_time_allocator`，经 `allocator_of_version_without_file` 回到 `rebuilt_allocator`），都在 `rebuilt_allocator` 里被「allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(」（`crates/singlefs-core/src/mount.rs:2105`）装上；装之前这个函数里没有记根调用 |
| 抬 F 预演（整串在拷贝上） | 装了 | 记根发在 `copy` 上：「copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);」（`crates/singlefs-core/src/mount.rs:3688`、`crates/singlefs-core/src/mount.rs:3744`）；`copy` 是已装表的分配器的克隆 |
| 回退、卸载、多实例 | 装了 | 都用 `mount_writable` 交回的分配器（装过表）；发布失败整个换回发布之前那一份，表随之退回（文档：「这次发布失败时调用方把分配器整个换回发布之前」`crates/singlefs-core/src/allocator.rs:1908`；换回动作本身是「std::mem::replace(allocator, allocator_before_this_publish)」`crates/singlefs-core/src/transaction.rs:1319`） |
| 取号（实例号）| 不碰分配器 | 函数只收 `pool`、不收分配器：「pub fn acquire_instance<Device: BlockDevice>(」`crates/singlefs-core/src/transaction.rs:900`，函数体两行 `crates/singlefs-core/src/transaction.rs:903`、`crates/singlefs-core/src/transaction.rs:904` |
| mkfs 同进程会话 | 装了 | core 自己的建法「allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(」（`crates/singlefs-core/src/make_filesystem.rs:731`） |
| 测试 / 装置里手建 | 冻结副本里 `grep -rn 'PoolAllocator::new(' crates --include=*.rs` 出 31 处（31-1 报告数的是 30 处，多的一处是 31-2 新加的 U12 帮手 `allocator_without_the_root_ring_as_before_revision_6`，它故意不装表）；31-1 判会撞的 15 处里：这批装了表的 11 处（见 N2），31-3 装的（E170、旧 E171、E156 内联单测、`on_device_modes`），31-2 装的（新 E171）；**剩 E156 的 PC-分配器一臂**（「request.allocator_source = HhAllocatorSource::WithoutRootRingPositiveControl;」`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:4736`）不装，今天会在第一次记根时 panic；31-1 / 31-3 实现员都已报，归 E156 第 5 次重跑，不在这批补丁里 |

**1.4 装的表与盘上的环不符而不被拦。** core 只拦跳号（上面 1924 行）；表里 `occupants`、`effective_floor` 对不对不判。这批里有两处手拼的表（N2 讲），读数都没变；这是「不被拦」的实例，不是缺陷。另：`record_zero_unit_roots_leading_to` 对没装表的分配器仍然什么都不做：「pub fn record_zero_unit_roots_leading_to(&mut self, version_to_build_on: CheckpointTxg) {」（`crates/singlefs-core/src/allocator.rs:2037`），所以一个只发零单元发布的没装表分配器不会 panic；接着发带单元的发布才会。无害。

**1.5 复跑。** 新单测（`# Panics` 那条）：
```
test allocator::tests::recording_a_root_on_an_allocator_without_a_root_ring_occupancy_panics_instead_of_never_reclaiming - should panic ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 190 filtered out; finished in 0.00s
```
变异表第 2734 行把断言换回静默返回，我逐行复跑：`ROW 2734 抓到(红) rc=101 compile_error=False`（`out/rows.txt`）。

**推翻 N1 的现象：** harness 档任一二进制或 checker 档任一装置在「挂载建 / mkfs 建」的分配器上看到该断言的消息；或 grep 出一处记根调用在 `install_root_ring_occupancy` 之前、且不是手建分配器。

## N2　`build_pool` 与 11 处装根环表

**判定：装的表与参数一致（逐处对上）；两处手拼读数不变、但「照可写挂载读到的样子」那句不精确；读数「该变而没变」的我量到两类；C551 新用例只罩得住一部分回收错法（4 类里抓 3 类、第 4 类被已有用例抓）。**

**2.1 装的是不是 mkfs 刚写下的那一张。** 表的建法在 core：「pub fn root_ring_occupancy_after_make_filesystem(」（`crates/singlefs-core/src/make_filesystem.rs:681`），三个区域的槽 0 各一条第 0 代根，F 取第 0 代根带的：「genesis.root.rollback_floor,」（`crates/singlefs-core/src/make_filesystem.rs:697`）；写盘用的是同一个迭代器：「for ring_slot in genesis_root_ring_slots() {」（`crates/singlefs-core/src/make_filesystem.rs:619`）。`build_pool` 装法：「allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(」（`crates/singlefs-harness/tests/common/mod.rs:700`）。参数配对，我逐处对了「mkfs 用的变量」与「装表用的变量」（读码 + grep 行号，冻结副本）：

| 文件 | mkfs 那行 | 装表实参 |
|---|---|---|
| `common/mod.rs` `build_pool` | `formatted_recorded_images` 里「let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");」（`crates/singlefs-harness/tests/common/mod.rs:189`） | `&parameters, &genesis`，「let parameters = parameters();」（`crates/singlefs-harness/tests/common/mod.rs:682`） |
| `fsync_drop_…rs` | 「let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");」（`crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:178`） | `&rig.parameters`（`crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:211`），`Rig` 存的就是 `parameters` |
| `multi_record_…`、`common_tree_split`、`crash_points_…`（checker 档） | `make_filesystem(&publish_parameters, …)` | `&publish_parameters` |
| `unequal_devices`、`new_pool_file_creation_publish`、`write_accounting_per_structure`、`crash_segments_…`（checker 档） | `make_filesystem(&parameters(), …)` 或 `(&parameters, …)` | 同一个变量 |
没有一处混用。上面 7 份我没有逐份重读 mkfs 前后几十行，只对了变量名；配对错的后果是 `slots_per_region` 不同，`overwrite` 用例里会立刻跳号断言红，harness 档全绿说明没错。

**2.2 mkfs 那张表与挂载会读到的是不是同一个。** 挂载读环时对「还没轮到的槽」不记：「一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回」（`crates/singlefs-core/src/mount.rs:1978`，注释原文），所以刚 mkfs 完挂载得到的表也是三个槽 0 上各一条第 0 代根，与 mkfs 同进程那张逐项相同。我另外用一次真实的新池新建文件跑（`run_new_pool_file_creation_keeping_the_allocator` 交回 core 自己的分配器）核了「core 持有的表」与 31-3 拼的表（见 N3 3.1），`{:?}` 文本逐字相同。

**2.3 两处手拼。**
- `accounting_tree_node_full`：「CheckpointTxg(NEW_POOL_FILE_CREATION_TXG - 1),」（`crates/singlefs-harness/tests/accounting_tree_node_full.rs:62`）。这个用例不经 mkfs、盘是全新的，根环上一条根都没有，空 `occupants`、F 0、记到 txg 2，与盘一致。第一次记根（txg 3）后最旧有效根成了 3，回收门槛 3；这次发布是第一个文件版本、不释放任何落点，没有可回收的，读数不变（两条用例跑绿）。
- `checker_known_bad_images`：「CheckpointTxg(row_publish_txg.0 - 1),」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:5644`），`occupants` 只放实例 1 最新那一条根。注释写「照可写挂载在实例 1 那一版上读到的样子」，**与可写挂载读到的不同**：挂载会读到环里全部有效根（至少还有第 0 代根与暖机那两条），最旧有效根是 0 而不是这一条的 txg。对这条用例没有影响：重建的分配器里只有 mkfs 的两个未释放单元、没有已释放记录，写行那次释放的实例表链释放代（写行 txg）高于任何门槛。结论：读数不变（89 条用例跑绿），注释的说法不精确，建议改成「只记最新那一条（够这次发布用）」。零轮。

**2.4 装了之后哪些用例的读数会变而没变。** 我在草稿副本 core 里加了回收事件打印（`tools/instrument_core.py`），逐个二进制直接跑（`--nocapture`；不带它 libtest 会吞掉通过用例的 stderr，我第一次就这样量了 0），另用环境变量还原第 31 批之前的行为（测试 / 装置不装表、没装表时记根静默返回）。13 个二进制（`out/census-reclaim-events.txt`）：
```
post checker_known_bad_images events=9   exit=0 ok. 89 passed      pre events=9
post row_publish_admission    events=65  exit=0 ok. 2 passed       pre events=10
post random_histories         events=6   exit=0 ok. 23 passed;19 ignored   pre events=6
post overwrite_in_one_instance events=16 exit=0 ok. 12 passed      pre events=0 exit=101（FAILED. 11 passed; 1 failed）
post root_ring_turn_in_the_make_filesystem_session events=5 ok     post root_ring_turn_in_one_mount events=60 ok
其余 7 个（release_checksum_quarantine、commit_generated_units_…、publish_failure_resent_unchanged、sequential_write_one_record_per_data_unit、an_abandoned_roots_rollback_floor_…、test_only_switches_force_every_branch、unequal_devices）：events=0，pre 没跑
```
- 第一次回收都在 txg 26：新用例的事件 `txg=26 n=2`、`txg=27 n=24`（`out/census-reclaim-events.txt` 同次日志，草稿目录 `runs3/post/12-…`）。原因：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:114`）与「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:115`）让 txg 24 才盖掉 (区域 0, 槽 0) 上的第 0 代根，最旧有效根 txg 24 起才离开 0；最小的释放代是 3（第一个文件换下 mkfs 树表），要到最旧有效根 = 3（txg 26）才第一次回收。这一条与 kb 一致：「| **槽序** | **跨区轮转**（区域 = `txg mod R`）；区域内槽位 = `(txg div R) mod S`（已定项 16） |」（`.claude/kb/decisions/22-单元原子性怎么合成.md:57`）。
- 结论 A（取样点只到 txg 23 以内的罩不罩得住回收）：**罩不住，也不需要**：这些用例（上面 7 个 events=0 的）在 txg ≤ 25 里一次回收都不会发生，所以第 31 批装表对它们的读数本来就没有影响；它们也不覆盖回收，覆盖回收的只有新用例与 `overwrite_in_one_instance` 里那条改了判法的用例（外加挂载那一族已有用例）。
- 结论 B（读数变了而用例没发现）：`row_publish_admission`（2 条，用 `build_pool` 再覆盖写 49 / 47 次、可写挂载）回收事件 10 → 65，用例仍绿，因为它们钉的是下界：「pool.allocator.records().len() > ONE_LEAF_OF_ALLOCATION_RECORDS,」（`crates/singlefs-harness/tests/row_publish_admission.rs:126`），记录条数下界 812 在回收之后仍成立（回收不删记录，D3 已定项 7：「条目留到该落点被重新分配时覆盖」，`.claude/kb/decisions/03-空间分配.md:131`）。它们不是 31-1 报的「会红的一条」，没人报过「读数变了」。零轮：不用改，只是别让以后有人拿这两条当「回收不发生」的证据。
- 我没量的：另 146 个 harness 档二进制（慢，机器负载 30 左右，前台 240 秒只跑得完约 10 个）。主 agent 合入验证里 165 个二进制全绿，只说明没红，不说明读数没变。

**2.5 C551 新用例罩不罩得住「记根时不回收」以外的回收错法。** 新用例做的是：覆盖写到 txg 30，末尾逐盘比 defer 账：「device_map.deferred_slots(),」（`crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs:71`），再在末尾一次跑池级 checker：「let verdicts = check_pool_image(&pool.memory_pool());」（`crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs:77`）。常数写死环 3 × 8：「const OLDEST_VALID_ROOT_AFTER_THE_LAST_OVERWRITE: CheckpointTxg = CheckpointTxg(7);」（`crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs:18`）。我改 core 记根回收门槛 5 种错法，每种在草稿副本上跑两条用例（`tools/mutate_core_reclaim.py`，`out/mutants.txt`；原样结果）：
| 变异 | 改法 | C551 新用例 | 改判法的 812 用例 |
|---|---|---|---|
| a | 门槛多一代（+1） | 红（FAILED） | 红：第 22 次覆盖写 `left: 5799936 right: 5816320` |
| b | 门槛少一代（−1） | 红 | 红：第 23 次 `left: 6078464 right: 6062080` |
| c | 环转之前拿 txg−2 当门槛（提前回收） | 红 | 红：第 2 次覆盖写 `left: 720896 right: 737280` |
| d | 环转之前（门槛 0）提前拿 3 当门槛（只多回收 mkfs 树表那一个落点） | **绿**（ok） | **绿**（ok） |
| e | 门槛不看 F | 绿 | 绿（F 恒 0 的会话里等价，不算打中） |
- d 的机理（`out/probe-first-file.txt`、`out/probe-first-overwrite.txt`，原样）：基线第一个文件之后每盘 `allocated=17 deferred=1`，变异 d 下 `allocated=16 deferred=0`（txg 3 就回收了释放代 3 那一槽）；再覆盖写一次（txg 4），变异 d 下槽 50178 的记录变成「span_slots: 2, generation: CheckpointTxg(4), is_released: false」——被新分配**复用**了，而它在基线里是「generation: CheckpointTxg(3), is_released: true」。复用之后「已分配」统计与记录自洽，两条新用例（都按记录与统计比）看不出；但 txg 0–2 那几条根还引用着这一槽（回退到它们会读到被改写的单元）。
- d 被什么抓：已有用例 `turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it` 红（`tools/mutant_d_existing_suite.sh`：`test result: FAILED. 2 passed; 1 failed`）；**我另加的每个 txg 后跑池级 checker 的探针**（`tools/per_txg_checker_probe.rs`，同一段 txg 4–30 历史）：基线 `PERTXG first_violation=None`，变异 d 下 `PERTXG first_violation=Some((4, ["I-2.1", "I-3.1", "I-4.8", "I-5.1", "I-7.4"]))`。所以：①新用例只在末尾跑一次 checker，抓不到「提前回收、复用、之后被盖出环」这一类；②在每个 txg 后都跑 checker（零轮的改法）就抓得到，且基线 txg 4–30 每一版都全绿，这同时是 C551 的推翻条件的一个更强版本：「`build_pool` 装上根环表之后，同一条用例判全部不变量。」（`.claude/kb/checks-owed.md:425`，那是 812 用例；它今天仍只判改过判法的那一条）。我不替主 agent采纳，只报：改法零轮。

**推翻 N2 的现象：** `build_pool` 之后覆盖写到 txg 30 的池里 core 持有的表与 mkfs 刚写的不同（我在 N3 探针里是相同的）；或哪一份装表的参数与 mkfs 的参数不同（会导致 `slots_per_region` 不同，用例已绿，不会）；或 `row_publish_admission` 的 `> 812` 在合入后红（它今天绿）。

## N3　31-3 的装置与用例

**判定：**3.1 `on_device_modes` 拼的表与 core 给的**逐项相同（站住，复跑核过）**；3.2 E170 / 旧 E171 / E156 内联单测「装了之后输出不变」的推论**站得住（推的，读码 + 事件数据，checker 档没跑）**；3.3 `overwrite_in_one_instance` 新判法与 D3 已定项 7、D16 已定项 1 是**同一个谓词，模拟没写错**，但这条用例与第一条新用例一样**看不见「提前回收、随即被复用」那一类错**（N2 的变异 d）。

**3.1 拼法逐项比。** 31-3 的函数：「pub fn allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process<」（`crates/singlefs-checker-tier/src/on_device_modes.rs:187`）。
| 项 | on_device_modes 拼的 | core `root_ring_occupancy_after_make_filesystem` 给的 | 相同吗 |
|---|---|---|---|
| 区域与槽 | 「RootRingSlot { region, slot: 0 },」（`crates/singlefs-checker-tier/src/on_device_modes.rs:216`），`region` 取遍 `ROOT_RING_REGIONS` | 三个区域各取槽 0（见本表下方的引文行，因为引文里有竖线，不放进表格） | 相同 |
| F | 「ROLLBACK_FLOOR_AT_MKFS,」（`crates/singlefs-checker-tier/src/on_device_modes.rs:223`） | 「genesis.root.rollback_floor,」（`crates/singlefs-core/src/make_filesystem.rs:697`）；mkfs 写根时用的就是「rollback_floor: ROLLBACK_FLOOR_AT_MKFS,」（`crates/singlefs-core/src/make_filesystem.rs:598`） | 值相同（都是常量 0） |
| 第 0 代根 txg、`latest` | 暖机第一条根的 txg 减 1：「.checked_sub(1)」（`crates/singlefs-checker-tier/src/on_device_modes.rs:208`） | `genesis.root.checkpoint_txg` | 今天相同（都是 0）；写法耦合：mkfs 若改第 0 代 txg、暖机不再紧接第 0 代，两边会分叉（31-3 报告第 9 节第 1 条已写） |
| 占用 | 补记 `run.warm_up.roots`（txg 1、2）与 `run.output.root`（txg 3）：「allocator.record_root_written_by_this_process(root.checkpoint_txg);」（`crates/singlefs-checker-tier/src/on_device_modes.rs:232`） | `publish_first_file` 里 `record_zero_unit_roots_leading_to` 补记 1、2，发布自己记 3 | 同一个次序 |
core 一侧的原文：「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」（`crates/singlefs-core/src/make_filesystem.rs:672`）。

**复跑（把 31-3 那个函数的函数体逐行抄进 harness 档用例，对 core 自己持有的分配器比）：** `run_new_pool_file_creation_keeping_the_allocator`（`crates/singlefs-harness/src/scenario.rs:158`，草稿副本里把它改 pub）同一次跑交回 core 自己的分配器，两边 `root_ring_occupancy()` 的 `{:?}` 文本比（`tools/on_device_modes_recipe_vs_core.rs`，`out/tests.txt`）。原样：
```
genesis_txg=0 roots=[1, 2]
test the_ring_the_on_device_helper_assembles_equals_the_ring_core_holds_after_the_new_pool_file_creation ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```
（拼出来的文本是 `occupants: {(0,0): ValidRoot 0, (0,1): ValidRoot 3, (1,0): ValidRoot 1, (2,0): ValidRoot 2}, effective_floor 0, latest_recorded_txg 3`，与 core 的逐字相同。）

**3.2 E170 / 旧 E171 / E156 内联单测输出不变的推论。** 推论的两块前提我分别核了：
- 「这几个分配器记到的 txg 不到 23」：E170 流二在 txg 4 之后换成挂载交回的分配器：「pool.allocator = mounted.allocator;」（`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:430`）；旧 E171 同形：「self.allocator = mounted.allocator;」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:851`），旧 E171 只有两条流（`enum StreamName` 两个成员，`grep -n 'inodes_across' …` 零命中）；E156 内联单测最长的一条覆盖写 12 次：「for step in 1..=12u64 {」（`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:5391`），txg 3 + 12 = 15。
- 「回收一次都不会发生」：事件数据——`build_pool` 一族里第一次回收出现在 txg 26（`txg=26 n=2`，N2 2.4），txg ≤ 25 里最旧有效根还没越过最小的释放代 3。所以 E170 流一流二、旧 E171 两条流、E156 内联单测里装不装表对写流与落点没有影响。
- 另一个会受装表影响的东西——分配器的 `{:?}` 文本——旧 E171 与 E170 都不记：`grep -n 'debug_text(&.*allocator\|memory_object\|allocator_digest' e171_three_oracle_collusion.rs e170_o3_spec_executor_export.rs` 零命中（新 E171 才记内存对象摘要，那是登记修订 6 的预言，不是这一条）。
- 没核的：这三个装置与 `replay.sh` 的实际复跑输出（checker 档 / 实验复跑，派发不跑）。31-3 报告第 4 节说旧 E171 的 s4 坏实现表第 95 行结局会变（「不补记零单元根」那一行，以前没装表是空操作、现在在第一个文件那次发布撞跳号断言）——读码上成立（`record_zero_unit_roots_leading_to` 对没装表的什么都不做、装了表的才补记，`crates/singlefs-core/src/allocator.rs:2037`），我没复跑。
**推翻 3.2 的现象：** 复跑 E170 / 旧 E171 基线导出，写流、段表或判定行与入库产物不同（排除读 `crates/mutations.tsv` 行数的几份）。

**3.3 `overwrite_in_one_instance` 新判法。**
- 谓词。kb 的可再分配谓词：「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |」（`.claude/kb/decisions/16-发布语义.md:33`）；环里最旧有效根：「盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg」（`.claude/kb/decisions/16-发布语义.md:34`）。用例里的判据：「record.is_released && record.generation <= reclaim_floor;」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:728`），门槛：「let reclaim_floor = output.root.rollback_floor.max(」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:721`），`max` 的另一半是用例自己模拟的环：「.map(|region| (RootRingSlot { region, slot: 0 }, genesis_txg))」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:681`）。「≤」「max(F, 最旧有效根)」与 kb 一致；一个实例、没有被抛弃的根，环里每条根都有效，所以「最旧有效根」就是模拟出的最小 txg。D3 已定项 7 允许记录留着：「条目留到该落点被重新分配时覆盖」（`.claude/kb/decisions/03-空间分配.md:131`），所以「已分配 = 不是已回收的记录的跨度之和」成立；记录在回收之后不删（`reclaim_released_records_up_to` 只记进 `reclaimed`：「if !self.reclaimed.insert(key) {」`crates/singlefs-core/src/allocator.rs:1838`）。
- 模拟写错会不会让用例与实现一起错。用例的模拟与 core 的表**共用**的只有 `target_for_publish`（用例直接调它）；它与 kb 独立一致：「| **槽序** | **跨区轮转**（区域 = `txg mod R`）；区域内槽位 = `(txg div R) mod S`（已定项 16） |」（`.claude/kb/decisions/22-单元原子性怎么合成.md:57`）。「第 0 代根住三个区域的槽 0」两边各自写了一遍（用例写字面 `slot: 0`，core 走 `genesis_root_ring_slots`），我对了 mkfs 源码：「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」（`crates/singlefs-core/src/make_filesystem.rs:672`）与它被 mkfs 写盘使用（`make_filesystem.rs:619`）。复跑逐 txg 比（`tools/sim_vs_core_root_ring.rs`：`build_pool` 之后连续覆盖写到 txg 90，每次发布后把用例的模拟与 core 分配器持有的表的最旧有效根（读 `{:?}` 文本里的 ValidRoot）比）：
```
compared_txgs=87
test the_overwrite_test_simulation_of_the_root_ring_equals_the_ring_the_allocator_holds_at_every_txg ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.86s
```
- 模拟或谓词错一格用例会不会红：N2 的 a（+1）、b（−1）、c（提前）三种，用例都红（第 22、23、2 次覆盖写的 `left/right` 见 N2 表）；也就是门槛两侧都钉死、边界「≤」被钉住。
- 看不见的一类：N2 的变异 d（只提前回收释放代 3 那一槽）下这条用例仍绿，机理见 N2（复用后记录与统计自洽，用例的判据由记录与统计构成）。这不是新用例的错：用例要判的是统计与记录的关系，不是回收的时机；时机由已有挂载用例与每 txg 的 checker 判。
- 上限 900 → 812：「while pool.output.allocation_records.len() <= 812 && overwrites < 200 {」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:716`）。复跑整份：`test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.85s`（`out/tests.txt`）；31-3 报「200 次覆盖写只到 882 条」我没单独复跑，但 812 条越过去了（用例末尾的断言过了）。

**推翻 3.3 的现象：** 有一个 core 提交使门槛偏一格而这条用例仍绿（我试过 ±1、提前，都红）；或模拟在某个 txg 与 core 的表分歧（87 个 txg 上没有）。

## N4　31-2 的 E171 新装置

**判定：**6.2 第 1 条（装根环表与 S1 启动核对）、第 3 条（A1 / A2 注入入口）、6.5 的 U10–U16 / V32–V36 **与登记一致**；注入入口在跑法里**没有一条路让真跑时注入非空**（grep 核过）；U13、U14 期望数的前提**站得住（推的：checker 档用例没跑）**；`p6-on-dbef78ed.patch` 与仓里那份之差**恰好第 29 批那 4 行（复跑核过）**。一处写法偏离登记（S1 比摘要不比全文）和一处措辞（「末行判决」其实是倒数第二行）不改判定。

**4.1 6.2 第 1 条与启动核对。** 登记：「`LiveStream::start`（第 1087 行起）在 `allocator.mark_format_time_units(…)`（第 1093–1102 行）之后加一句」（`research/prompts/e171-r1-prereg.md:1165`）。装置里：「allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:1132`），空闲图没换（仍是 `DeviceFreeMap::new`，登记「不整个换成」那一句）。核对函数与存流：「fn allocator_carries_the_root_ring_of_make_filesystem_check(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:1092`）、「checks: vec![root_ring_check],」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:1156`）。偏离：登记写「`{:?}` 文本相同」（`research/prompts/e171-r1-prereg.md:1168`），装置比的是两边 `{:?}` 文本的 sha256（31-2 报告第 9 节第 1 条已写）；文本相同 ⇔ 摘要相同（不计碰撞），而 `StopCheck` 比字符串（`holds()` 是 `local == crates`），没装时 local 是 `"None"`，不会假相等。U10 里整段文本也直接比了一遍（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:12161` 起）。

**4.2 A1 / A2 与「真跑一律交空」。** 登记：「四个调用点（第 8080、8113、8187、8248 行）一个字不改。」（`research/prompts/e171-r1-prereg.md:1178`）；「`run_confirmed_loss_form`（第 7938 行）一律交三个空；命令行不开放」（`research/prompts/e171-r1-prereg.md:1176`）。装置：补丁 20 个 hunk 里没有一个碰那四个 `if let Some(exit_code) = stop_…` 块（我读了 diff 全文，hunk 头在 `_m3-formal-code-b31-r1-diff.md` 里；四处调用点在仓里文件里是第 8150 / 8187 / 8262 / 8324 行：「if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8150`），与 31-2 报告第 4 节「改后各处行号」相同）。三个注入点读的都是同一个结构：「shapes.piece_panic_injections.pairing.as_ref(),」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8110`）、「shapes.piece_panic_injections.piece_panic_control.as_ref(),」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8167`）、「shapes.piece_panic_injections.sampling_point_export.as_ref(),」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8256`、`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8318`）；阳性对照里「替换」只在传了非空时生效：「let Some(injection) = replacing_the_built_in_rules.or(built_in_injection.as_ref()) else {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9691`），传空时就是原先的内置规则。
- 「有没有一条路让真跑时非空」我 grep 了这个文件里全部构造点（命令与结果）：
```
$ grep -nE 'ConfirmedLossShapes \{|export_prepared_stream\(|piece_panic_control_of\(|equivalence_pairing_with_injected_piece_panics\(|env::var|std::env::args' e171_three_oracle_collusion_after_the_oracle_fixes.rs | cut -c1-140
5045:    export_prepared_stream(
5064:fn export_prepared_stream(
8031:        ConfirmedLossShapes {
8050:struct ConfirmedLossShapes {
8108:        let pairing = equivalence_pairing_with_injected_piece_panics(
8164:        let control = piece_panic_control_of(
8248:        let mut export = export_prepared_stream(
8310:        let mut export = export_prepared_stream(
9438:    equivalence_pairing_with_injected_piece_panics(stream, None)
9442:fn equivalence_pairing_with_injected_piece_panics(
9676:fn piece_panic_control_of(
9702:    let injected = equivalence_pairing_with_injected_piece_panics(stream, Some(injection));
9782:    let arguments: Vec<String> = std::env::args().skip(1).collect();
11062:        let export = export_prepared_stream(
11827:        let injected = equivalence_pairing_with_injected_piece_panics(&stream, Some(&injection));
11868:    fn confirmed_loss_shapes_for_the_test() -> ConfirmedLossShapes {
11873:        ConfirmedLossShapes {
12367:    ) -> ConfirmedLossShapes {
12372:        ConfirmedLossShapes {
```
  读法：`mod tests` 从第 9812 行起（「mod tests {」`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9812`），第 11062 行起的都是用例。非测试代码里：`ConfirmedLossShapes` 只有 8031 一处构造（`run_confirmed_loss_form`，三格都是 `None`，第 8041 行「sampling_point_export: None,」`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8041`）；基线与逐行经 `export_stream` 调第 5045 行的 `export_prepared_stream`，交 `None`；8108 / 8164 / 8248 / 8310 读的是 `shapes` 里那三格。
  命令行与环境：全文件只有 `std::env::args()`（`parse_command_line` 解析，选项里没有注入）；`env::var` 零命中。`equivalence_pairing_of` 标了 `#[cfg(test)]`（「fn equivalence_pairing_of(stream: &PreparedStream) -> EquivalencePairing {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9437`）），所以非测试代码里已经没有不经注入入口的对拍调用点；没有任何一处能把非空注入带进真跑。唯一的「非空注入」仍是阳性对照的内置那三条（`piece_panic_control_injection`，修订 5 就有，没改）。

**4.3 U13、U14 的期望数（「c1 落了」的态数）前提。** 用例的数（帮手 `first_in_flight_write_and_the_states_where_it_landed`，`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:12336`）：「.filter(|ordinal| digits.digit_of(*ordinal, position) != 0)」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:12356`），区间是上一行的 `(digits.first_ordinal..tables.state_count)`。前提三条：
- ① 对拍流（k = 2）判的态集合就是末段全部态：「(digits.first_ordinal..observation_tables.tables.state_count).collect();」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9460`）；与用例数的区间相同。
- ② 「落」= 落地串那一位不是 `0`，注入条件 `Landed(c1)` 用的是同一个口径（`PiecePanicInjection::rule_of`）；数字 0 = 没落（`digit_of != 0` 含撕裂）。
- ③ k = 2 流上总态数与类数是已有用例钉死的：「assert_eq!(pairing.states, 288, "72 × 2² 态全枚举");」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:11195`）、「assert_eq!(pairing.classes, 144, "72 种取法 × C 落没落");」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:11200`），c1 是整写（288 = 72 × 2²，没有撕裂的第三取法）⇒ c1 落了的态 = 144。对拍不注入时不停在 S13：已有用例「assert!(!pairing.stops_at_s13());」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:11218`）。U13 里 S1「实际截的 k 与分法里的 C 写数相等」那一条两边出自同一条 k = 2 的流，成立；生产的 k = 6（「const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 6;」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8569`））与用例的 k = 2（「const IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS: usize = 2;」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:11170`））不一样是有意的（登记 U13 写明用 `equivalence_pairing_stream()`）。注入之后对拍流上每个「c1 落」的态正常那一路 panic、逐件重做一件都不 panic，计成 unattributed，这个停点在 S13 停点之前：判决行 `q5_not_run=unattributed_panic` 的 `stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(` 调用（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8150`）排在「if pairing.stops_at_s13() {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8157`）之前。
- 登记 U13 写「末行判决」，实际 `lines.finish()` 追加 `name=done emitted=N`：「let body = format!("name=done emitted={}", self.emitted);」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:5335`），所以判决行是倒数第二行；用例写的是后者（`assert_the_q5_stopped_as_unattributed` 的注释与断言）。措辞偏差，用例对。
- U15 的态数 = 末段各写取法数之积加全持久态：因为 `first_ordinal + product == state_count`，全持久态就是区间最后一个，集合大小就是积（用例用 `BTreeSet` 去了重）；U16 的 `classification.judged_ordinals(&digits).len()` 与装置调 `export_prepared_stream(…ListedStatesKeepingTheConsultedReports(judged)…)` 用的是同一个表达式。
- **复跑不了**：U10–U16 与 V32–V36 都是 checker 档装置内联用例，派发不跑；上面都是读码 + 已有用例钉的数，写「推的」。**推翻 U13 / U14 的现象：** 提交时这两条用例的 `unattributed_panic_states=` 不是 144（例如对拍流上有别的真 panic，31-2 报告第 6 节末也写了）。

**4.4 `p6-on-dbef78ed.patch` 打出来的那份与仓里那份之差。** 登记要求之差恰好第 29 批那 4 行：「即两份装置恰好只差第 29 批那 4 行」（`research/prompts/e171-r1-prereg.md:1204`）。我没用 31-2 的草稿件，独立取料复核（`out/n4-p6.txt`，`rerun.sh <草稿> static`）：Q5 树那份取自 `scripts/experiments/target/e171-r1-frozen-tree-revision-5-q5/…`，旧基底取自 31-2 的 `device-945e41c8.rs`（`git hash-object` = `6bfee63…`，与 `crates.patch` 的 `index 6bfee63..3dd188c` 的左端相同，冻结副本里那份 = `3dd188c…`）。原样：
```
945e41c8…  device-945e41c8.rs        dbef78ed…  scripts/experiments/target/e171-r1-frozen-tree-revision-5-q5/…        cc736772…  冻结副本里的那份
patching file …/q5-plus-p6.rs (read from …/q5.rs)           （patch --fuzz=0，无偏移、无模糊）
2c933e21…  q5-plus-p6.rs
--- diff q5+p6 对仓里那份
3475a3476,3479
>         RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {
>             "system_configuration_unreadable_after_the_read_stage_criterion_true_without_read_faults"
>                 .to_string()
>         }
--- diff q5 对 945e41c8     3437a3438,3441（同样那 4 行）
  832 a.txt   832 b.txt   两份补丁去掉行号头之后逐字节相同；grep -c '^@@'：crates.patch 20、p6-on-dbef78ed.patch 20
```
即 P6 在 Q5 树上无模糊打上、与仓里那份只差那 4 行；两份补丁的内容相同、只是行号头不同；hunk 数 20 与 31-2 报告第 4 节那张表的行数相同。

**4.5 旁注（不属于本格）：** 磁盘上已有 `scripts/experiments/target/e171-r1-frozen-tree-revision-6/`，它的装置与 P6 结果相比只有 14 行 `diff`，全在流三本地常量与注释（`LOCAL_THIRD_STREAM_WRITES` 6000 → 5980、`…FULL_STATES` 441 453 147 → 362 809 947、`…TWO_STATE_CLOSED_FORM` 441 451 942 → 362 808 742，及段长表几个 24 → 20）：这正是登记 6.2 第 2 条「流三本地常量重取」的形状；我没核它与修订 6 的关系，也没核那几个数。

## N5　变异行

**判定：一致（没发现基线即红或互相掩护的行）；一条旧行红的机理变了（E156 M35）。**
- **锚点与点名测试（静态，`tools/check_rows.py`，`out/n5-static.txt`）。** 新加与替换的 16 行（C551 8 行：2734–2741；E171R1 装置 V32–V36 与 U12 共 7 行：2742–2748；替换了点名测试的 m4b 那一行：570），原文各自恰好命中 1 次；点名的测试都是恰好一处定义的 `#[test]` fn、没有 `#[ignore]`、`--` 后的筛选词是测试名的子串；无 `\n` 以外的反斜杠转义。末行原样：`CHECKED 19 rows, problems=0`（我的前缀过滤把表里已有的 V3、V30、V31 三行也带进来了，所以是 19；16 行加那 3 行都 OK）。
- **harness 档与 core 的 5 行逐行改坏复跑**（`tools/run_mutation_rows.py`，`out/rows.txt`，原样）：
```
ROW 2734 抓到(红) rc=101 compile_error=False :: C551：分配器没装根环表记根时静默返回（不断言）
ROW 2735 抓到(红) rc=101 compile_error=False :: C551：记根时环转了也不回收（…）
ROW 2736 抓到(红) rc=101 compile_error=False :: C551：build_pool 不装根环表
ROW 2737 抓到(红) rc=101 compile_error=False :: C551 后：回收时「已分配」不减（…）
ROW 570 抓到(红) rc=101 compile_error=False :: 增补 3 第 2 件（…攻方变异 m4b 同一处）：…直接钉已分配统计的那条用例判出（树分裂换锚点）
```
  基线（不改源码）这 5 行点名的测试都绿：新单测 ok、C551 新用例 ok、`overwrite_in_one_instance` 12 passed（N1 1.5、N2、N3）。
- **与已有行原文相同而互相掩护？** 三组「同改法、不同点名测试」：2734 与 2748（core 断言删掉，点名 core 单测 / U12）、2742 与 2743（V32，点名 U10 / U11）、569 与 570（m4b，点名 random_histories 那条 / `overwrite_in_one_instance` 那条）。门禁的重复判据是四项逐字相同：「change = (path, fields[2], fields[3], fields[5])」（`.claude/gate.d/code-source-discipline.sh:1127`），旁边的注释：「只按改法认会误拒：同一个改法让两条不同的测试变红是合法的。」（`.claude/gate.d/code-source-discipline.sh:1126`），所以不拒；每行单独在干净副本上改、单独跑，互不掩护。
- **基线即红？** harness 档 5 行不是（上面）；checker 档 11 行（2738–2748）我没跑（派发不跑），读码：点名的测试都调到被改的那一处——E170 那条 `prepare_stream(StreamName::NewPoolFileCreation)`（`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:3905`）、E156 内联单测 `new_pool_file_creation_state()`（`…e156_allocation_basis_counts.rs:5105`）、`second_instance_mode_…` 调 `allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process`（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1826`）。基线是否绿取决于装置内联用例，推的。
- **E156 M35（第 803 行，不在这批补丁里）红的机理变了。** 它把产品路径换成不装表的分配器：「E156 M35（第 4 次重跑登记第九节）：岔路 1 的族的分配器退回不装根环表（挂载内不回收）」（`crates/mutations.tsv:803`），原文替换：`allocator_after_make_filesystem(parameters, devices, genesis)` → `allocator_without_the_root_ring(devices, genesis)`（锚点在 `e156_allocation_basis_counts.rs` 里恰好 1 处，第 1461 行）。合入后这个变异让 core 在第一次记根时 panic，点名的 `hh_without_holes_…` 照样红，但不再由「不回收」那一格自己的断言判出；这一行今天要在 E156 第 5 次重跑里重新定位（31-1 报告第 9 节已提）。零轮。

**推翻 N5 的现象：** 提交时门禁 checker-tier-crates-mutation-replay 在 11 行 checker 档里任一行报「基线不绿」或「没红」；或 5 行 harness / core 行里任一行我的复跑与门禁结果不同。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| N1 core 断言 | 一致 | 条件恰为 `root_ring == None` 时记根；产品路径里没有在装表之前记根的；消息与 `# Panics` 写清；仅 E156 PC 臂今天会撞（已知，归 E156 第 5 次重跑） |
| N2 装表参数与 build_pool | 一致 | 11 处装表实参与 mkfs 同变量；mkfs 那张表与 core 持有的相同（见 N3 3.1） |
| N2 两处手拼 | 一致（读数不变）；说法不精确 | `checker_known_bad_images` 注释「照可写挂载读到的样子」与挂载实际读到的不同（最旧有效根 0 对 1 条根），对这条用例无影响 |
| N2 读数该变而没变 | 发现（零轮） | 13 个二进制里只有 `row_publish_admission` 的回收事件 10 → 65 而用例仍绿（钉的是 `> 812` 下界）；其余用例 txg ≤ 25 一次回收都不发生；另 146 个二进制没量 |
| N2 新用例罩回收错法 | 部分罩住 | 门槛 ±1、提前（a b c）红；只提前回收一槽再被复用（d）两条新用例都绿，被已有 `root_ring_turn_in_one_mount` 红、被每 txg 跑 checker 的探针红（txg 4 起 5 条不变量） |
| N3 on_device_modes 的表 | 一致（复跑） | `{:?}` 文本与 core 同一次新池新建文件后持有的表逐字相同；写法耦合第 0 代 txg = 暖机首根 − 1 |
| N3 E170 / 旧 E171 / E156 不变 | 一致（推的） | 记到的 txg ≤ 15 / ≤ 4，回收最早 txg 26；旧 E171 与 E170 不记分配器摘要；checker 档没跑 |
| N3 812 用例新判法 | 一致；看不见 d | 与 D16 谓词、D3 已定项 7 同一个；模拟与 core 的表在 87 个 txg 上相同；共用的只有 `target_for_publish`，它与 D22 一致 |
| N4 E171 新装置 | 一致（推的 + p6 复跑） | 6.2 第 1、3 条与 U10–U16、V32–V36 对上登记；真跑注入恒空（grep）；U13 / U14 期望数 144 的前提站得住；p6 之差恰 4 行 |
| N5 变异行 | 一致；一条旧行机理变 | 16 行锚点各 1 次、测试是 `#[test]`、无 ignore；harness / core 5 行逐行复跑全红；三组同改法不同测试不掩护；E156 M35 现在由 core panic 抓、不再由它自己的回收断言抓 |

## 没做什么

- 派发不跑的一律没跑：checker 档的内联用例（U10–U16、E170 / E156 / on_device 的变异点名用例）、`crates/mutations.tsv` 的 11 行 checker 档（2738–2748）、QEMU、实验复跑、`replay.sh`；涉及它们的结论都写了「推的」。
- 没判别、没采纳：每 txg 跑 checker、改 `checker_known_bad_images` 的注释、E156 M35 行的重新定位、`NewPoolFileCreationRun` 带 genesis，都是零轮线索，交主 agent。
- harness 档只跑了 13 个二进制 + 草稿副本上的 5 种变异 + 4 个探针；其余 146 个二进制没量（负载高、前台 240 秒），主 agent 合入验证里的 165 个全绿只说明没红。
- D3 已定项 7 我只引了第 131 行那一条，没把整节从头读完；D16 已定项 1 整节读过；背景材料里的 kb 附录没整份读，kb 条款都去 kb 文件里现查行号。
- 31-3 报告第 4 节「旧 E171 s4 表第 95 行结局会变」我只核了读码上的机理（`allocator.rs:2037` 的早退），没复跑。
- 磁盘上的 `scripts/experiments/target/e171-r1-frozen-tree-revision-6/` 装置与 P6 结果的 14 行差异（N4 4.5）只看了差异，没核它与登记第 6.2 节第 2 条的关系。
- 草稿目录处置：交回前删掉 `/tmp/claude-1000/m3-formal-code-b31-r1-sonnet/tree/`（含它的 `target/`）与 `/tmp/claude-1000/m3-formal-code-b31-r1-sonnet/rerun-draft/`（含 `tree/`）；留着只有文本的日志：`/tmp/claude-1000/m3-formal-code-b31-r1-sonnet/runs3/` 与 `build*.log`，原样输出已进 `research/prompts/m3-formal-code-b31-r1-sonnet-model/out/`。

## cite-check 末行

命令：`python3 scripts/agents/cite-check.py research/prompts/m3-formal-code-b31-r1-sonnet-output.md --root /tmp/claude-1000/m3-formal-code-b31-r1/tree --root . --background research/prompts/_m3-formal-code-b31-r1-background.md`

```
  ✓ 核了 74 处引文，对上 74 处，没判 0 处（逐处列在上面）
```
