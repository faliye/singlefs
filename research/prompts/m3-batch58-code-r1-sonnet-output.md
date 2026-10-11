# m3-batch58-code-r1 正推腿（Sonnet）报告

轮：m3-batch58-code-r1；格：K1–K3；立场：正推（核测试测的是不是条款与前一轮判决点名的那件事）。
代码行号一律按冻结副本 `~/.local/share/singlefs/work/m3-batch58-code-r1/repo/`（下文路径省去前缀，`--root` 取那一份）；kb 行号按仓根 kb 现查。
自己量的东西在 `~/.local/share/singlefs/work/m3-batch58-code-r1-sonnet/`，日志与探针放 `research/prompts/m3-batch58-code-r1-sonnet-model/`。

## 〇、条款与判决点名的东西（逐条 grep -nF 现查）

| 谁 | 位置 | 内容 |
|---|---|---|
| D18 已定项 24 第 3 条 | `.claude/kb/decisions/18-块里携带什么信息.md:578` | 「开放容器：有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。」 |
| I-9.21（产品支） | `.claude/kb/invariants.md:309` | 「除最右那一个之外的每个打包记录类型 1 容器记录数 == 815」 |
| I-9.22（旋钮支） | `.claude/kb/invariants.md:310` | 「每个打包记录类型 1 容器记录数 ≤ 旋钮值」 |
| I-1.10 | `.claude/kb/invariants.md:35` | 「判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：先于按字段表解条目。」 |
| D5 已定项 4 | `.claude/kb/decisions/05-快照-空间记账机制.md:130` | 「13 | 清扫水位（截至哪个 txg 的释放已由整轮清扫确认不可读）」 |
| D18 已定项 24 第 4 条 | `.claude/kb/decisions/18-块里携带什么信息.md:579` | 「回收：第一版不回收墓碑，统计量 13、14（D5（快照 / 空间记账机制） 已定项 4）第一版不登记」 |

- 条款与实现的一处差：kb 的 I-9.22 行（`.claude/kb/invariants.md:310`）今天只写「≤ 旋钮值」，没有「最右那片之外每片 == 旋钮值」。walk.rs 的旋钮支判了后一句（`crates/singlefs-checker/src/walk.rs:12058` 起的 `let non_rightmost_container_holds_the_knob_value =`，墓碑树同形在 `walk.rs:12154` 一带），依据是 D18 已定项 24 第 3 条的「满了才关」（上表第 1 行）。第 57 批 kb 写回还没落（kb 的 sha256 与开工快照一致，已核），所以甲这批全部「只红 I-9.22 / I-9.37」的断言，条款原文只管到「≤ 旋钮值」和「满了才关」两处，I-9.37 在 kb 里根本没有行（定义在 `~/.local/share/singlefs/work/m3-batch57/container-judges/report.md`，不在仓里）。判：**规则没说**（kb 行落后于实现与用例）；这是随 kb 写回一批的事，不是用例的错。推翻它：kb 的 I-9.22 行在写回时加上「最右那片之外每片 == 旋钮值」。
- 次序的依据不在 kb：冷走读与重建里「分配记录逐条判代与跨度 → 记账条目宽 → 逐条判代与 seq → 数记账条目 → 数映射条目」这条链，出处是判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G1、`m3-batch56-code-r1-main-verification.md` 第三节 H4 和 `m3-batch57-code-r1-main-verification.md` 第三节 J1。I-1.10 只写了「条目宽先于按字段表解条目」，没写分配记录与数条目的先后。判：次序这一格 kb **规则没说**，条款是判决，用例钉的是判决。

## K2　J4 改造法（GPU 产品墓碑支对拍）

判：**一致**（两个造法在被枚举的东西上逐项相同）；GPU 与 CPU 判得一样与不撞别的读宽上限这两半**复核不了**（GPU 对拍归 checker 档，本轮不跑；下面给读码依据与能在 harness 档量的替身）。

### K2-1 起点镜像下状态集合与原造法一样吗

新造法在 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13780-13783` 把前 815 次 unlink 施加进起点镜像、写表只录第 816 个 unlink：

- `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13781`：「base.apply(&operations[pool.mkfs_operation_count..operations_before_the_last_unlink]);」
- `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13783`：「&operations[operations_before_the_last_unlink..],」

我没有跑那条 GPU 用例，但这两个造法产出什么（起点镜像、写表、段、哪几次写可撕）全是 harness 档函数，我拷了冻结副本，把两种造法并排造一遍逐项比（探针在拷贝里，源文本与原样输出在 `research/prompts/m3-batch58-code-r1-sonnet-model/probe-k2-appended-to-checker_known_bad_images.rs.txt` 与 `probe-k2.log`）。原样输出：

```
PROBE_K2 operations_total=55717 mkfs=25 before_last_unlink=55649
PROBE_K2 old_writes=45878 old_segments=6543 first_write_of_last_unlink=45822
PROBE_K2 new_writes=56 new_segments=8 new_segment_sizes=[26, 2, 1, 2, 20, 2, 1, 2]
PROBE_K2 writes_identical=56
PROBE_K2 straddling_segments=0 old_tail_segments=8 tail_equals_new=true
PROBE_K2 earlier_segments_folded_into_base=6535 earlier_writes=45822
PROBE_K2 new_slots_written=58 max_writes_over_one_slot=2
PROBE_K2 base_touched_16KiB_slots=2230
PROBE_K2 old_flow_touched_16KiB_slots=2230
PROBE_K2 tearable_old_equals_new=true tearable_count_old=4 tearable_count_new=4
PROBE_K2 new_write_kinds={"JournalRecord/4096B": 4, "RootRecordFua/512B": 2, "SystemConfigurationSlot/4096B": 4, "UnitWrite/16384B": 40, "UnitWrite/32768B": 6}
PROBE_K2 new_flow_touched_16KiB_slots=2230 subset_of_old=true
```

读法：
- 旧造法写表 45878 条、6543 段；第 816 个 unlink 的第一条写是第 45822 条。新造法写表 56 条、8 段。新写表与旧写表的尾部 56 条逐条相同（设备、种类、FUA 位、偏移、内容），旧的尾部 8 段（首写 ≥ 45822）减去 45822 之后与新的 8 段逐段相同，没有一段跨界（`straddling_segments=0`，所以起点镜像不会把一段劈成两半）。
- 旧造法里前 6535 段不展开：`crates/singlefs-checker-tier/src/crash.rs:1869` 「Self::NotExpanded => 0,」，不展开的段自己一个状态都不出，只作为「整段持久」进入后面的状态。所以旧造法的状态集合本来就只由第 816 个 unlink 那 8 段生出，新造法的起点镜像就是「前面各段全持久」那一份——集合相同，不是「少了一些」。这一点同时说明：用例注释「崩溃状态与这里相同」成立（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13748` 一带的文档）。
- 原地覆写（撕裂第三态）的判据 `crates/singlefs-checker-tier/src/crash.rs:2034` 「let base_holds_nonzero_bytes_there = base.devices.get(&write.device).is_some_and(|device| {」读基镜像与更早的写：旧造法里「更早的写」是前 45822 条，新造法里它们已在起点镜像里。我把这一条判据照抄进探针，旧造法（基 = mkfs 之后、整条写表）与新造法（基 = 起点镜像、尾部写表）逐条比：`tearable_old_equals_new=true`，各 4 条（journal 记录与系统配置槽的原地写），状态数不变。
- 展开规则同一条：`compare_on_every_state` 与旧闭包都是「段长 ≤ 展开上限（默认 8）才展开」，新写表的 8 段里 26 与 20 写的两段不展开，其余六段（2、1、2、2、1、2 写）展开，与旧造法对第 816 个 unlink 那几段的做法相同。

### K2-2 第二片改号那一形，GPU 与 CPU

`renumber_the_last_container_of_the_flow`（同文件第 13204 行起）只动 `flow.writes` 里命中的单元写与指着它们的位置条目（`corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root` 在 `flow.writes` 里找 14 字节的位置模式）。第二片容器、根的条目、journal 点名项与根记录都在第 816 个 unlink 的写里，这 56 条写与旧造法逐条相同（上表），所以改号后的写表在尾部 56 条上与旧造法逐字节相同；`last_unit_copies` 取「最后两条命中」，命中的两条在尾部。CPU 侧 `cpu_checker_red_invariants.contains("I-9.21")` 的来源、GPU 与 CPU 的逐位比对都没变。**GPU 与 CPU 实际判得是否逐位相同：复核不了**（要卡，归 checker 档，主 agent 跑过的结果以判决为准）。推翻它：主 agent 的 GPU 那一趟这条红，或「建得出表」之后别的拒绝。

### K2-3 会不会撞内核别的读宽上限

建表的几道上限与新流的量（`crates/singlefs-checker-tier/src/crash_facts.rs:287` 「pub const MAXIMUM_ENUMERATED_WRITES: usize = 2048;」，`crates/singlefs-checker-tier/src/crash_judge_tables.rs:51` 「pub const MAXIMUM_WRITES_PER_LOCATION: usize = 64;」）：

| 上限 | 值 | 新流的量 | 出处 |
|---|---|---|---|
| 枚举写表的写数 | 2048 | 56 | 探针 `new_writes` |
| 一个落点被写盖几次 | 64 | 2 | 探针 `max_writes_over_one_slot`（按 16 KiB 槽数） |
| 落点（碰过的 16 KiB 槽）数 | 无上限常量；每落点存字节，量 ≈ 槽数 × 16 KiB | 2230 槽，≈ 35 MiB | 探针 `new_flow_touched_16KiB_slots`，与旧造法同一个数、是它的子集 |
| 打包后表偏移 | 2³² 字 | 远小于 | `crates/singlefs-checker-tier/src/crash_judge_tables.rs` 的 `check_the_packed_directory` |

旧造法在 `crates/singlefs-checker-tier/src/crash_judge_tables.rs:331` 「if enumerated.len() > MAXIMUM_ENUMERATED_WRITES {」就停了，没走到后面的落点表、期望版本表，所以「后面那些表在这条流上建不建得出」旧造法没验过；新造法的落点集合是旧造法落点集合的子集（`subset_of_old=true`），落点那一段不会比旧造法更大。期望版本表有 1633 个版本（`versions_with_the_first_file_through_the_last_publish` 取 txg 3 到最后一版），每版 6 个字加 3000 字节文件的分块校验，内核里 `expected_version_index` 是对它的线性扫（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3153` 一带），读码没找到表项数上限。表字节数对卡的绑定上限：`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:539` 一带的 `assert!(table_bytes <= limit, …)`，表 ≈ 35 MiB 加期望版本表，我没有卡上的 `max_storage_buffer_binding_size`，**复核不了**；以量级论，远低于常见的 128 MiB 下限。推翻它：主 agent 那一趟 GPU 报「建得出表」之外的拒绝，或 `table_bytes` 的断言。

## K1　旋钮多片坏镜像（甲 J3）

判：**一致**（测的是 D18 已定项 24 第 3 条「满了才关」在旋钮下的那一半，每条用例只因它点名的那一种写错而红）；三处「规则没说 / 量不稳」单列在最后。

### K1-1 复跑（冻结副本的拷贝，经内存包装，上限 10G，线程 10，同一时刻一条 cargo）

命令（日志与脚本在 `research/prompts/m3-batch58-code-r1-sonnet-model/`）：

```
CARGO_TARGET_DIR=$W/target nice -n 19 bash scripts/run-with-memory-cap.sh 10G bash scripts/capped.sh 10 cargo test --offline --no-run -p singlefs-harness --test checker_known_bad_images --manifest-path $W/repo/Cargo.toml
$W/target/debug/deps/checker_known_bad_images-fa5092303c98133d under_changing_knobs_redden --test-threads 10
```

原样末行（`run-new-k1.log`）：「test result: ok. 6 passed; 0 failed; 3 ignored; 0 measured; 256 filtered out; finished in 2.41s」。三条慢档带 `--include-ignored`（`run-new-k1-slow.log`）：「test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 262 filtered out; finished in 16.85s」。九条在正确判法下全绿。

### K1-2 每条是不是只因要钉的那一种写错而红

两道锁，都是硬断言：

- `crates/singlefs-harness/tests/checker_known_bad_images.rs:16759` 「[tree.knob_capacity_invariant()],」：违例集合必须恰好等于这一棵的旋钮条（I-9.22 / I-9.37），别的不变量红一条就败。
- `crates/singlefs-harness/tests/checker_known_bad_images.rs:16767` 「if detail.contains(tree.name_in_the_violation()) && detail.contains(detail_fragment)」：违例说明里要有「不是最右的容器 N 装了 1 条，不是旋钮值 2」。说明只留第一处违例（`crates/singlefs-checker/src/image.rs:364` 「self.first_violation.entry(invariant).or_insert_with(detail);」），所以「容器号逐片加一」「每片至少一条」「多于旋钮值」任何一句先红，片段对不上，用例就败。三种排布各自的片段指向不同的容器（容器 2、容器 3、容器 2），不会互相冒充。

排布与形状：`crates/singlefs-harness/tests/checker_known_bad_images.rs:17061` 「records_per_container_of_each_record: &[2, 2, 1, 1],」→ [2, 1, 1]；`…:17070` 「records_per_container_of_each_record: &[2, 2, 2, 2, 1, 1],」→ [2, 2, 1, 1]；`…:17081` 「records_per_container_of_each_record: &[2, 2, 1, 1, 2, 1],」→ [2, 1, 2, 1]。判之前先断言真造出了这个形状（`…:17106` 的 `assert_eq!(tree.record_counts_of_each_container(&pool.output), layout.record_counts_in_leaf_order`），写侧「满了才关」按当时的旋钮：旋钮换小之后下一条开新片（D18 已定项 24 第 3 条，上表第 1 行）。判的旋钮 2 对每一版都是合法的上限（每片 ≤ 2），所以「多于旋钮值」一句不会先红。

条款侧：`.claude/kb/invariants.md:310` 的 I-9.22 行只有「每个打包记录类型 1 容器记录数 ≤ 旋钮值」，「最右那片之外每片 == 旋钮值」这半句 kb 里没有（见〇）。用例钉的是后半句，依据是 D18 已定项 24 第 3 条的「满了才关」与 I-9.21（`.claude/kb/invariants.md:309`）同形的写法；判**规则没说**（kb 行落后于实现与用例）。

### K1-3 转一圈根环之后候选集里还剩哪几版，[2,1,2,1] 那一版是不是真被判到

- 根环：三个区域、每区 8 槽，共 24 槽（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:112` 「/// 根环一圈的根槽数（三个区域、每区 `RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 8 槽，`common::parameters`）再多几次：」；`…:115` 「pub const PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER: u64 = 30;」）。[2,1,2,1] 那一版写完之后再发 30 次发布，24 个槽全被这 30 次里最新的 24 次盖掉，候选集里剩的 24 版都发生在写完之后，容器树都是 [2,1,2,1]（建 inode 不动三棵容器树）。写的中途那几版（[2,1,1]、[2,1,2]）不在候选集里。
- 判到：这 24 版里容器 2 都是叶序第 2 片、共 4 片，不是最右，装 1 条 ≠ 旋钮值 2，红在容器 2；容器 3（第 3 片，2 条）满，不红。只判倒数第二片的判法在这 24 版上判的是容器 3，不红，用例败。
- 我拷了这条用例，把 `TurnedOverByCreatingInodes` 改成 `KeepsTheVersionsThatWroteTheContainerTree`（不转根环），再放进「只判倒数第二片」变异（表第 3854 行）跑：`row-3854-variant-no-ring-turn.log` 「test tombstone_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two ... ok」，变异照绿。所以「所以要转出去」这句注释（`…:17077`）成立：不转，写的中途那版 [2,1,1] 留在候选集里，容器 2 在那版是倒数第二片，变异也红在它上面，用例分不开。
- 另两个排布不转根环：[2,1,1] 与 [2,2,1,1] 的候选集里有写的中途各版，这些版（[1]、[2]、[2,1]、[2,2]、[2,2,1]）在旋钮 2 下都合法，只有最后一版红，所以不转也是干净的；用例断言通过（K1-1）即证。

### K1-4 变异复跑（我自己在拷贝上跑，变异表行施加、跑、还原，脚本 `run_mut.sh` / `mutate_row.py`；每次还原后 sha256 与冻结副本相同）

| 表行 | 变异 | 红的用例 | 与表里写的相符 |
|---|---|---|---|
| 3852 | 墓碑旋钮支只判叶序第一片 | 墓碑 [2,1,1]、墓碑 [2,2,1,1]（两条） | 是 |
| 3853 | 墓碑旋钮支只判最左片 | 墓碑 [2,1,1]、墓碑 [2,2,1,1] | 是 |
| 3854 | 墓碑旋钮支只判倒数第二片 | 墓碑 [2,1,2,1]（慢档，debug + `--include-ignored`；表里是 release） | 是 |
| 3855 | 谱系 / 退役旋钮支（共用一个判法）只判叶序第一片 | 谱系与退役各两条（四条），墓碑四条全绿 | 是 |
| 3856 | 同判法只判最左片 | 谱系与退役各两条 | 是 |
| 3857 | 同判法只判倒数第二片 | 谱系 [2,1,2,1]、退役 [2,1,2,1]（慢档），墓碑 [2,1,2,1] 绿 | 是 |

三棵树的会红用例各自只在它那棵树的判法被改坏时才红（3855–3857 改的是谱系 / 退役共用的 `judge_lineage_or_retirement_container_capacity_of_a_version`，墓碑那六条全绿；3852–3854 改墓碑的判法，谱系 / 退役全绿）。推翻 K1-2：任何一条用例在正确判法下红，或在不是它点名的写错的变异下红。

### K1-5 GPU 那三条是不是真在比旋钮支（读码；GPU 没跑，复核不了红绿）

- 旋钮值是编进扫描内核的：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2352` 「let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(knobs);」；CPU 一侧每个状态用同一组旋钮重判（`…:2458` 一带的 `check_pool_image_given_the_container_capacity_knobs(`），比的是违例位与判到位（`…:2581` 「|| gpu_checker_bits != *cpu_checker_bits」）。三条都走 `assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening`，要求 CPU 红对应的旋钮条、GPU 与 CPU 无一处不一致。
- 内核里三棵树的旋钮支是同一个函数：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3357` 「if (d_container_record_count(container_index) != knob_records) {」，墓碑在 `…:3367`，谱系在 `…:3213`、退役在 `…:4127` 带各自的旋钮常量调进来。表里 GPU 变异（第 3858 行）改的就是这一行，三条里只有墓碑那条被点名，谱系与退役两条测的是接线（各自的旋钮常量、读容器的路径）。
- 三条流的规模（我把同样的池造法在 harness 档量了，`probe-k1.log`）：墓碑 298 写 / 47 段、谱系 166 写 / 31 段、退役 387 写 / 59 段，一个落点最多被写 8 次，状态数约 60–109；远低于 2048 条写、每落点 64 次写的建表上限（K2-3），谱系记录表上限 256（已知欠账）也碰不到（谱系最多 4 条记录）。
- 三条流的旋钮序列都是 `[2, 2, 1, 1]`（`…:13838` 「const KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER: [u64; 4] = [2, 2, 1, 1];」），每条流是 [2,1,1]，没有候选集的问题（GPU 判的是崩溃状态里的各根槽，流不转根环；[2,1,1] 之前的各版 [1]、[2]、[2,1] 在旋钮 2 下合法）。

推翻 K1-5：主 agent 那一趟 GPU 里这三条之一绿而 CPU 不红，或第 3858 行的变异没抓到。

### K1-6 单列的三处

1. 重构不改旧用例：`pool_with_records_in_one_container_tree_under_its_knob` 改成交 `vec![k; n]` 给新函数，谱系支把「装旋钮」从循环外挪进循环里（每回退一次装一次，值不变）；旧用例照绿由主 agent 的整轮 194 个二进制 2008 条保证，我没重跑整轮。
2. 三条 `#[ignore = "harness 慢档用例：…debug 下单条跑过 60 秒"]`：我量到的单条用时很不稳。同一个二进制、nice 19、本机忙时单条跑：墓碑 114.50 秒、谱系 51.71 秒、退役 170.43 秒（`bihsc2e0i` 的原样输出，三行「finished in …」，没存日志）；负载降下来之后三条一起跑 `unmutated-slow-three.log` 「test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 262 filtered out; finished in 2.49s」。`.claude/rules/verification.md` 写的是「单条用时看平常跑的时候量到的，不另外单独量」，所以用时与 `#[ignore]` 是否该标，规则没说怎么在负载不同时取数；要不要把这三条放回快档（同一文件里别的转根环 30 次的用例都在快档），请主 agent 在空闲时量一次。它的后果：快档跑不到 [2,1,2,1] 那三条，只有慢档 / 变异表那一遍才抓「只判倒数第二片」。
3. 同类写法留在别处、不在这批 diff 里：`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:584` 还用「STATISTIC_INODE_WATERMARK + 1」造未登记的标签 13（第 57 批 J2 说的那一种），见 K3-3。

## K3　冷走读次序补钉与换造法（乙 J1、J2）

判：**一致**——三张新盘各只由它点名的那一个次序分得出；J2 换造法后用例不再依赖任何统计量标签。可写挂载那一条钉的是重建里「分配记录逐条判」排在「数映射条目」之前这一个次序。

### K3-1 复跑与变异（拷贝上；同 K1 的做法，日志 `run-k3.log`、`row-38xx.log`）

五条（含批 56 的两条老 H4）在正确判法下全绿，`run-k3.log` 原样末行：「test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out; finished in 3.23s」。

变异行（`crates/mutations.tsv` 的行号，施加、跑 `allocation_records_whose_generation_is_after_the_root`、还原）：

| 表行 | 改成 | 红的用例（只有这些） |
|---|---|---|
| 3859 | 冷走读：数映射条目挪到分配记录判之前 | `…_before_the_mapping_entry_count_by_the_recovery` |
| 3860 | 冷走读：分配记录判挪到条目宽与逐条判之后、紧贴数记账条目 | `…_before_each_accounting_entry_is_judged_by_the_recovery` |
| 3861 | 重建：分配记录判挪到数映射条目之后 | `…_before_the_mapping_entry_count_by_the_writable_mount` 与老的 `…_before_the_accounting_entry_count_by_the_writable_mount` |
| 3850 | （批 57 H4，J2 要求照红）冷走读：分配记录判挪回数记账条目、数映射条目之后 | 三条只读恢复的（含老的 `…_accounting_entry_count_by_the_recovery`） |
| 3851 | （批 57 H4，J2 要求照红）重建：分配记录判挪到数记账条目之后 | 老的 `…_accounting_entry_count_by_the_writable_mount` |

每一行恰好红它该红的那一两条，另外几条照绿，说明三张新盘互不冒充，也不被老 H4 的盘冒充。

### K3-2 三张新盘是不是只由点名的那一次序分得出（有没有别的判先拒了）

- 每张盘造出来的当场就断言「只坏其中一处时的报法」：映射盘 `crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1996` 「前提：只坏映射那一处时，只读恢复在 txg 4 那条根上报映射条目数」；水位行盘 `…:2033` 「前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目的代晚于根」；记账数盘 `…:1972` 「前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目数」。这三句说明各自那一处单独坏时，没有更靠前的判先拒；两处一起坏时报分配记录，就只能是分配记录判在它们前面。
- 分配记录那一处（`move_the_allocation_records_of_the_tree_table_slot_after_the_root`，`…:1941` 「assert_eq!(changed_records, 2, "罩树表单元那个槽的分配记录两盘各一条");」）三张盘共用，与批 56 H4 同一处，两盘一起改，「每个落点每盘一条」「分配记录装得进池几何」两判在它前面，都照过。
- 次序依据（kb 没有，是判决）：冷走读 `crates/singlefs-core/src/recovery.rs:5395` 「allocation_record_generations_and_spans_are_judged(allocation_records, root)?;」→ `…:5396` 「every_accounting_entry_is_as_wide_as_its_field_table(」→ 逐条判 → `…:5402` 「accounting_entry_count_is_three_plus_six_per_device(」→ `…:5407` 「mapping_entry_count_is_one_per_mapped_unit(」；重建 `…:2894` 「allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;」→ `…:2918` → `…:2928` 「accounting_entry_count_is_three_plus_six_per_device(」→ `…:2944` 「mapping_entry_count_is_one_per_mapped_unit(」。冷走读那一段的注释（`…:5393`）说两边次序相同。
- 水位行盘（J1 ②）钉的是「分配记录判在逐条判之前」；它的注释写成也排在「记账条目宽」之前（`…:2114`），盘上每条记账条目宽都是 34，条目宽那一判没有坏，所以「在条目宽之前」这半句这张盘钉不到——这是正文第二节已记的欠账（「分配记录判在条目宽之前」还没钉），不算打中，只记注释比用例多说了半句。

### K3-3 删记账根 key 最小一行，会不会碰到 inode 号水位行或别的判的前提

- 记账条目的 key 以统计量标签打头（`crates/singlefs-core/src/records.rs:317` 一带的 `sort_key` 是 `(statistic, tree, device, generation)`），inode 号水位的标签是 12（`records.rs:29`，登记表里最大的一个），key 最小的那一行标签必定小于 12，碰不到水位行；用例自己也断言：`…:1961` 「dropped_row.statistic, STATISTIC_INODE_WATERMARK,」（`assert_ne!` 的第一个参数行，断言删的不是水位行）。
- 台阶行仍在（16 条里去掉一条剩 15，台阶行一条，数记账条目 14 ≠ 3 + 6 × 2 = 15），数条目判不到「台阶行认不出」；前提断言（`…:1972`）证明它在只读恢复里确实报记账条目数。
- J2 的目的（用例不依赖统计量标签 13）我直接量了：在拷贝的 `recovery.rs` 里同时加上两种读者改法（逐条判里拒标签 > 12；数条目时不算标签 > 12 的行；补丁 `patch_readers.py`），跑这五条：`readers-AB.log` 原样末行「test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out; finished in 2.70s」，五条全绿。对照：同一个补丁下，仓里还在用标签 13 的另一条老用例（`core_review_tree_table_duplicates_and_slot_one_search.rs:584` 「own_row.statistic = STATISTIC_INODE_WATERMARK + 1;」所在的那条）红：`readers-AB-core-review.log` 「test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.03s」。说明补丁是咬得动的，新造法确实不再靠 13。这条老用例不在本批改动里，记作线索：同一个依赖留在那儿（D18 已定项 24 第 4 条「统计量 13、14（D5（快照 / 空间记账机制） 已定项 4）第一版不登记」，而 D5 的表里 13 是清扫水位），将来登记 13 时它先红。
- 删一行会不会让别的判的前提落空：删行之后，只读恢复与可写挂载在分配记录判（排在所有这些之前）就返回了；水位行存在的判（`recovery.rs` 重建里 `InodeNumberWatermarkRowMissingFromTheAccountingTree` 那一句）在数条目之后，删的又不是水位行。读码没找到会在错的理由上绿的路径。

### K3-4 可写挂载那一条钉的是重建里的哪一个次序

`rebuild_version_in_the_unit_area_starting_at`（`crates/singlefs-core/src/recovery.rs:2519` 起）里 `…:2894` 的分配记录逐条判，要排在 `…:2944` 的数映射条目之前；它读完记账树（`…:2900` 起的 `read_code_two_tree`）、逐条判、数记账条目、查水位行在中间。变异 3861 把前者挪到数映射条目之后，这一条红；因为中间还夹着数记账条目，老的「数记账条目」那条可写挂载用例同时红（上表），所以映射盘的可写挂载这一条本身只证「在数映射条目之前」，与老用例合起来证「在数记账条目之前」。分配记录判挪到数记账条目与数映射条目之间的变异（读码推，我没跑）：老的数记账条目那条红，这条映射盘的可写挂载用例照绿——这是设计，每条各钉自己的那一个先后，不是漏。

推翻 K3：某张盘在正确判法下报的不是点名的成员，或上表某一行变异不红点名的那条，或在不改标签的读者改法下用例变红。

## 前一轮判决第三节 J1–J4 的验收列，逐条对

| 编号 | 验收（`research/prompts/m3-batch57-code-r1-main-verification.md` 第三节） | 本腿核到的 |
|---|---|---|
| J1 | 「数映射条目挪到分配记录判之前」「分配记录判挪到逐条判之后」两种变异各红一条；各一行变异证红 | 表第 3859、3860、3861 行各红点名的那一两条（K3-1）；两盘各有的只读恢复 / 可写挂载都有（映射盘两条、水位行盘只读恢复一条，与判决写的一致） |
| J2 | 攻方两种读者改法下用例照绿；H4 两行变异照红 | 两种读者改法同时加上，五条全绿（K3-3）；表第 3850、3851 行照红（K3-1） |
| J3 | 三棵树各一张 [K, K−1, …] 与 [K, K, K−1, …]；攻方「只判叶序第一片」、正推「只判最左片」「只判倒数第二片」变异各红；CPU、GPU 各一行变异；GPU 对拍每棵树一条 | 三棵树 × 三种排布九条 CPU 用例（K1-1），六行 CPU 变异各红点名的用例（K1-4）；GPU 三条与一行变异在案（第 3858 行），GPU 红绿复核不了 |
| J4 | 缩短被枚举的写，表建得出、GPU 与 CPU 逐位相同、坏镜像红；然后证第 57 批那 4 行 GPU 变异 | 起点镜像 + 尾部写表，写、段、可撕判据与旧造法逐项相同，建表几道上限远未触及（K2）；GPU 逐位相同与那 4 行变异复核不了 |

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 每条是否只因要钉的写错而红 | 一致 | 违例集合必须等于该树的旋钮条且首处违例说明含「不是最右的容器 N 装了 1 条，不是旋钮值 2」；九条正确判法下全绿，六行 CPU 变异各红点名的用例，别的不红 |
| K1 转根环后候选集 / [2,1,2,1] 是否判到 | 一致 | 24 槽全被写完之后的 30 次发布盖掉，候选 24 版都是 [2,1,2,1]；不转根环的变体下「只判倒数第二片」照绿（row-3854-variant-no-ring-turn.log） |
| K1 GPU 三条是否比旋钮支 | 一致（读码）；红绿复核不了 | 旋钮常量编进内核，三棵树共用 `d_judge_containers_under_the_capacity_knob`，CPU 一侧用同一组旋钮重判；流规模在建表上限之内 |
| K1 条款侧：I-9.22 / I-9.37 | 规则没说 | kb 的 I-9.22 行只写「≤ 旋钮值」，I-9.37 在 kb 里没有行；「最右之外 == 旋钮值」靠 D18 已定项 24 第 3 条；随 kb 写回一批补 |
| K1 慢档标签 | 规则没说 | 三条 `#[ignore]` 的用时在负载不同时 2.5 秒到 170 秒不等，规则只说「看平常跑的时候量到的」；请主 agent 在空闲时量 |
| K2 新起点镜像下状态集合 | 一致 | 写 56 条逐条相同、8 段相同、没有跨界段、可撕判据相同；旧造法前 6535 段本来不出状态（`NotExpanded => 0`），不是「少了一些状态」 |
| K2 改号那一形 GPU 与 CPU | 复核不了 | 改号只动尾部 56 条写，与旧造法逐字节相同；GPU 逐位结果要卡 |
| K2 别的读宽上限 | 一致（读码与量）；卡上 `table_bytes` 复核不了 | 枚举写 56 ≤ 2048、落点写次数 2 ≤ 64、落点集合 2230 槽 = 旧造法落点集合的子集 |
| K3 三张新盘是否只由点名次序分得出 | 一致 | 三行新变异每行只红点名的用例，老 H4 两行照红，且每张盘造的时候先断言单坏一处时的报法 |
| K3 删记账根最小一行 | 一致 | 标签号必小于水位 12，用例断言删的不是水位行；两种拒标签 > 12 的读者改法下五条全绿，对照的老用例红 |
| K3 可写挂载那一条钉哪个次序 | 一致 | 重建里分配记录逐条判（`recovery.rs:2894`）在数映射条目（`recovery.rs:2944`）之前 |
| K3 注释比用例多说半句 | 规则没说 | 水位行盘注释写「在记账条目宽之前」，盘上宽度没坏，钉不到；属已记欠账 |

## 没做什么

- 没跑任何 GPU 用例、checker 档测试、门禁阶段（`.claude/gate.d/` 下一个都没起）、整轮 `cargo test`；GPU 那一格的红绿与表里第 3858 行变异、第 57 批 4 行 GPU 变异我复核不了，要卡。
- 没重跑 `merge-and-verify.sh` 的整轮 194 个二进制；旧用例是否因重构而红，以主 agent 整轮为准。
- 没造「分配记录判挪到条目宽与逐条判之间」的变异去证欠账，因为它在正文第二节已记。
- 没评攻方腿的东西，没替主 agent 采纳任何一条。
- 自己的拷贝（`~/.local/share/singlefs/work/m3-batch58-code-r1-sonnet/repo`）与编译目录（`…/target`，2 GB）已删；探针源文本、各次日志与变异脚本留在 `research/prompts/m3-batch58-code-r1-sonnet-model/`，要复跑就重新拷冻结副本、把探针追加到 `checker_known_bad_images.rs` 末尾。草稿目录里只剩日志与脚本。
