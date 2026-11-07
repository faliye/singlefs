# m3-close-b19-switch-sweep-code-r1 攻方腿报告（2026-11-06）

立场：哪一格会错。攻击面：正文第四节 Q2 四问与 Q3。读的是此刻主工作区的 `crates/`（`mount.rs`、`instance_table_sweep.rs` 的 sha256 与开工快照 `crates-sha256.txt` 那两行相同，现查）。

这条腿没写模型目录：要试的都写成副本上的测试，副本与补丁在草稿目录，命令在「要主 agent 跑的命令」一节。下面凡是标「推的，待第 N 条」的数，是我读码与手算出来的，主 agent 跑出那一条之前都只算推的。

**副本与补丁（草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-opus/`）**

| 文件 | sha256 | 是什么 |
|---|---|---|
| `copy/crates/singlefs-core/src/mount.rs` | `bb85bd464cea4048c3429868900b23ef75381e620b9d6b5ff6e272f3a76b5b71` | 主工作区那份末尾接上 `patches/mount-window-tests.rs`（cargo fmt 过） |
| `copy/crates/singlefs-core/src/instance_table_sweep.rs` | `276c7ec4d7f979d13cb4af642555daacf4ba7295f8082c4004de5650dedf64ee` | `misplaced_ring_reads_tests` 模块末尾接上 `patches/sweep-genesis-tests.rs` |
| `copy/crates/singlefs-verification/tests/writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero.rs` | `43ff013dfd2d4d7dd559714e26430f6024f939da7b704702328e3e7675f12684` | 末尾接上 `patches/switch-e2e-tests.rs` |
| `copy-mutant-without-todays-value/crates/singlefs-core/src/mount.rs` | `bd2cfed923222049c5cbd909c2d00398cc49424e3aa7668b187dc9e789984424` | 上面那份再把第 1652 行的 `first_txg_today.max(…)` 去掉（变异体，别的两份文件与 `copy` 逐字节相同） |
| `patches/mount-window-tests.rs` | `1473420904b2d819071798a930fcdb48cb3c68b537830e3780f8116c057dcca3` | 甲窗口对拍与边角用例 |
| `patches/sweep-genesis-tests.rs` | `d03d939de3d9f50d7aca97a60a7c25daf3c316b85363b28e7fc0358b90a0f0a6` | 清扫错位读回第 0 代根的盘面与两条用例 |
| `patches/switch-e2e-tests.rs` | `6e2abb1ab8f917c3665bd60b5bacae5992981b404860ba3efdcbfecbf7d06952` | 经真可写挂载的五条边角用例 |
| `copy-attack-tests.diff` | `c0f205f553b02164bc0066d9a6c1d68c313a95f0ecaa77e7ee3b446662a329bf` | `copy/crates` 相对主工作区 `crates/` 的整份差（只三份文件） |

## 各格判定一览

| 格 | 判定 | 依据 |
|---|---|---|
| Q2.1 S 不是 2 的幂 | 没打中：代码与登记定义相等 | 读码证明（落点公式在任意 3S 个连续 txg 上是双射）；S = 5、6、7 的值推的，待第 1、3 条 |
| Q2.1 N = 0 | 没打中：相等 | 同上；待第 1 条 |
| Q2.1 N 贴着 u64 顶端 | 没打中：登记没写这一边，代码「装不下的 txg 不越过、max H 是 `u64::MAX` 才报到顶」与「装不下的 txg 上住不了根」相容 | 读码；待第 1 条 |
| Q2.1 读得出的根只有第 0 代根 | 代码与登记相等；**登记定义自己有一道缝**：第 0 代根三份同字节，「N 自己那一槽」恒取区域 0 槽 0，读出 N 的若是区域 1、2 的槽 0，区域 0 槽 0 读坏时不越过 | 读码；后果只在多处读坏下出现（推的） |
| Q2.1 全部槽都读坏 | 走不到：读阶段择不到根先拒，开关那一步不执行 | `crates/singlefs-core/src/mount.rs:9214`；待第 3 条 |
| Q2.1 另一格：N 取自哪一遍读 | **打中（字面）**：登记写 N 与 `highest_root_txg` 是同一个量，代码取甲自己那一遍；挂载期间读故障变了时两者不同，代码给的数可以比登记读法小（S = 4 的算例 22 对 11）；不在 E205 的故障模型里 | 读码；算例推的，待第 1 条 |
| Q2.2 开关漏出写行或中途换档 | 没打中：调用链逐层给在第二节 | 读码 |
| Q2.3 无故障的合法历史 | 没打中：改后清扫与改前逐槽读数相同，不会少删或多拒 | 读码论证，第三节 |
| Q2.3 错位读回第 0 代根 | **打中**：C973 改法之后，读实例 1 唯一那条根所在的区域 1 槽 0 两次都错位读回区域 0 槽 0 的第 0 代根，落点核照收，实例 1 那一行仍判成要删——正是 C973 验收要拦的那一形 | 推的，待第 2 条 |
| Q2.3 错位读而实例集合不变 | 改后这一轮不删、改前照删：多拒一轮，但这是条款要的口径（这一槽这一次读坏），不算缺陷 | 读码 |
| Q2.4 去掉「max(今天的值, …)」 | **打中**：没有一行变异罩它，现有 12 + 3 条用例里没有一格让这一项起作用 | 推的，待第 4、5、6 条 |
| Q2.4 后台计算产品入口递今天那一档 | **打中**：`crates/singlefs-core/src/mount.rs:9556` 那一处没有变异行 | `grep` 可核，第四节 |
| Q2.4 N 含不含被抛弃的根 | 没有用例罩（盘面上没有被抛弃的根）；不是一行文本变异能表达的 | 读码，推的 |
| Q2.4 端到端用例的 S | 只取 4、8、16；我加了 5、6、7 | 待第 3 条 |
| Q2.4 清扫两条用例 | 都只错位读回非第 0 代根，罩不住上面那一形 | 读码 |
| Q3 | `crates/singlefs-core/src/` 里读根环解析根的只有四处，三处核落点，`recovery.rs:5054` 那一处不核，判它不用核 | 第五节 grep 与逐处判 |
| 背景材料对不上的一句 | 正文第二节写甲「取 N = 读得出的根里最小的 txg」，代码与登记都是最大 | 第一节末 |

**背景材料对不上的一句（我核过）**：`research/prompts/_m3-close-b19-switch-sweep-code-r1-body.md:27`：「取 N = 读得出的根里最小的 txg。」而代码 `crates/singlefs-core/src/mount.rs:1576` 是「.max()」，登记 `research/prompts/e205-preregistration.md:43`：「N = 这次挂载读根环读得出、自证过（含落点公式核）的根的最大 txg」。代码与登记一致，正文那一句写反了；变异 `crates/mutations.tsv:6874` 正是「甲的 N 取读得出的根里最小的 txg」那一格。判决引正文这一句时按代码改。

## 一、Q2.1 甲的窗口在四种边角上

**代码怎么算**（`crates/singlefs-core/src/mount.rs` 现查）：

- N 取甲自己那一遍读数：`crates/singlefs-core/src/mount.rs:1569`：「let ring_slot_readings = every_root_ring_slot_read_once(devices, system_configuration);」，自证过的取最大，一条都没有取 0（`crates/singlefs-core/src/mount.rs:1577`：「.unwrap_or(CheckpointTxg(0));」）。
- 偏移从 1 走到 3S − 1：`crates/singlefs-core/src/mount.rs:1638`：「for offset_past_the_highest_root in 1..ring_length_in_txgs {」；N 加偏移装不下 u64 就停（`crates/singlefs-core/src/mount.rs:1643`：「break;」）；记最后一次碰到的越过槽的 txg。
- 没有越过的槽给今天的值（`crates/singlefs-core/src/mount.rs:1650`：「None => Ok(first_txg_today),」），有的给 `crates/singlefs-core/src/mount.rs:1652`：「Ok(first_txg_today.max(checkpoint_txg_after_the_version(highest_txg)?))」。

**为什么在任意 S 上等于登记定义**：落点公式是 `crates/singlefs-core/src/root_ring.rs:172`：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」与 `crates/singlefs-core/src/root_ring.rs:173`：「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」。t 落到（区域 r，槽 j）当且仅当 t ≡ r + 3j（模 3S）：只用到整除与取模，S 取什么整数都成立，不要求 2 的幂。所以任意 3S 个连续 txg 每槽恰好一个，[N + 1, N + 3S − 1] 恰好缺 N 那一槽，循环「从小到大记最后一个」就是登记的 max H。登记写的是整数上的定义；u64 里装不下的 txg 代码当不存在。

副本上的对拍用例 `window_value_equals_the_registration_definition_at_every_slots_per_region_and_both_ends_of_u64`（第 1 条命令）：登记一侧按上面那个同余式直接求，不走偏移循环；S 取 4..=16 全部，N 取 0..=200 与 `u64::MAX − 100..=u64::MAX`，越过的槽取每个单槽、空集、整环、除三个槽 0 之外的全部，今天的值取三档，逐格比。推的：全部相等，待第 1 条打出 `WINDOW_AGAINST_REGISTRATION cases=… mismatches=0`。

**各边角的写行 txg**（三档依次是今天的实现 / 甲只并读坏 / 甲并全零；全是推的。纯算式的那几行待第 1 条 `WINDOW_EDGE` 那几行，经真挂载的待第 3 条 `NOT_POWER_OF_TWO`、`GENESIS_ONLY`、`ALL_SLOTS_UNREADABLE` 那几行）：

| 边角 | 盘面 | 代码给的 | 登记定义给的 |
|---|---|---|---|
| S = 5 / 6 / 7，mkfs 之后 | N = 0，三个槽 0 是第 0 代根，别的 3S − 3 槽读回全零 | 1 / 1 / 15、18、21 | 同（max H = 3S − 1） |
| S = 5 / 6 / 7，起点镜像不注故障 | N = 6，txg 7..3S − 1 那几槽读回全零 | 7 / 7 / 15、18、21 | 同 |
| S = 5 / 6 / 7，起点镜像只坏 txg 5 那一槽 | N = 6，txg 5 那一槽在窗口里落 5 + 3S | 7 / 21、24、27 / 21、24、27 | 同 |
| N = 0，只有第 0 代根，区域 0 槽 0 读坏（S = 4） | 那一槽就是 N 那一槽 | 1 / 1 / 12 | 同（不越过） |
| N = 0，只有第 0 代根，区域 2 槽 0 读坏（S = 4） | 那一槽在窗口里落 txg 2 | 1 / 3 / 12 | 同 |
| N = `u64::MAX − 1`，S = 5，整环越过（纯算式） | 窗口里只有 `u64::MAX` 装得下 | 报序号到顶 | 同 |
| 同上，`u64::MAX` 那一槽不越过 | 越过的槽在窗口里都装不下 | 今天的值 | 登记没写；代码与「装不下的 txg 上住不了根」相容 |
| N = `u64::MAX` | 窗口里一个 txg 都装不下 | 今天的值 | 同上 |
| 全部槽都读坏（mkfs 之后 S = 4，12 槽设备报读错） | — | 三档都在读阶段拒，走不到开关 | 登记没写 |

全部槽都读坏走不到开关：读阶段先择根，择不到就拒（`crates/singlefs-core/src/mount.rs:9214`：「choose_root(cache, system_configuration).ok_or(RecoveryFailure::NoValidRoot)?;」），开关在它之后的第 9800 行。所以「N 一条都没有取 0」（第 1577 行）在读故障不变时走不到；只有挂载期间读故障变了、读阶段读到根而甲那一遍一条都没读到才走到，那时结果被 max 拉回今天的值。

**登记定义自己的一道缝（读得出的根只有第 0 代根时）**：第 0 代根三份同字节，`crates/singlefs-core/src/root_ring.rs:190`：「.any(|genesis_slot| genesis_slot == ring_slot);」三个槽 0 都认。N = 0 时「N 自己那一槽」恒是区域 0 槽 0（txg 0 的落点），不管第 0 代根是从哪个槽读出来的。读出 N 的是区域 1 或 2 的槽 0、而区域 0 槽 0 读坏时，区域 0 槽 0 不被越过；它可能住着 txg 3S·k 的根，前提是区域 1 槽 0 那次 txg 1 的根槽写丢了、而且 txg ≥ 1 的根全部读不出。这是多处读坏，在承诺外。代码与登记在这里相等（副本里已有用例 `crates/singlefs-core/src/mount.rs:14156`：「fn the_ring_slot_of_the_highest_readable_root_itself_is_not_passed() {」把它钉成「不越过」，变异 `crates/mutations.tsv:6863` 罩着）。所以这不算代码错，是交给 E205 判读时要知道的射程。

**另一格（打中，字面）：N 取自哪一遍读**。登记写 N「与 `first_txg_of_new_instance` 里 `highest_root_txg` 是同一个量」（第 43 行）；代码取甲自己那一遍，注释里承认两遍可以不同（`crates/singlefs-core/src/mount.rs:1538`：「今天的值读的是另一遍：挂载期间读故障不变时两遍读数相同；」）。算例（S = 4，副本用例 `n_taken_from_the_switch_pass_is_not_highest_root_txg_when_a_read_error_is_transient`）：环里 txg 1..10，txg 9 那一槽两遍都读坏，txg 10 那一槽只在甲那一遍瞬时读坏，今天的值 11。登记读法 N = 10、越过 {txg 9 那一槽}，得 22；代码 N = 8、越过 {txg 9、txg 10 两槽}，得 11。推的，待第 1 条 `TRANSIENT_PASS` 那一行。四句：
1. 分不分辨臂：两条甲臂一起受影响，不分辨甲只并读坏与甲并全零；分辨的是「甲」与「甲的字面定义」。
2. 系统当时看不看得到：看得到，两遍读数都在挂载里。
3. 满足的是哪一句：登记第 43 行「与 … `highest_root_txg` 是同一个量」那一分句。
4. 跑前条款的改法在这一格上中不中：登记第一节「六格」写的故障「只在这次挂载里有，被这次挂载写过的落点当场撤掉」，挂载里写行之前读故障不变，E205 的每一格两遍读数都相同，这一格不进 E205 的读数。这一格只在有人拿开关去量瞬时读错时才要紧。

## 二、Q2.2 开关有没有中途换档、有没有漏到写行之外

没打中。调用链逐层（行号现查）：

**产品入口两条，各自写死今天那一档**
- `mount_writable` → `crates/singlefs-core/src/mount.rs:8928`：「mount_writable_with_space_admission(parameters, devices, SpaceAdmission::JudgedByTheFormula)」→ `crates/singlefs-core/src/mount.rs:8941`：「mount_writable_with_test_only_switches(parameters, devices, space_admission, ShadowLedger::On)」→ `crates/singlefs-core/src/mount.rs:8956`：「mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(」→ `crates/singlefs-core/src/mount.rs:9398`：「FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing,」。
- `mount_writable_with_background_compute_settings`（带 GPU 设置的产品入口，文档说它与 `mount_writable` 等价）→ `crates/singlefs-core/src/mount.rs:9515`：「mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold(」→ `crates/singlefs-core/src/mount.rs:9556`：「FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing,」。
- 只供测试的入口只有一处把开关递下去：`crates/singlefs-core/src/mount.rs:8988`：「first_txg_of_the_new_instance,」。别的只供测试的入口（第 9425、9450、9478、9597 行）都写死今天那一档。

**挂载里只取一次，下游只读这一个值**
- 开关进 `mount_writable_with_every_test_only_switch` 之后只在 `crates/singlefs-core/src/mount.rs:9800`：「let first_txg = first_txg_of_new_instance_under_the_test_only_switch(」用一次。读阶段、判退、建上一版都在它之前，之后不再重算。
- 取到的 txg 交给重建分配器（`crates/singlefs-core/src/mount.rs:9816`：「first_txg,」）与 `InstanceStart`（`crates/singlefs-core/src/mount.rs:9854`：「first_txg,」）。`InstanceStart` 全文只在第 9848 行构造一次，`establish_instance` 只在第 9844 行被调一次。
- 档名只进 `MountOutput`（`crates/singlefs-core/src/mount.rs:7926`：「first_txg_of_the_new_instance_branch: start.first_txg_of_the_new_instance.branch_name(),」）。枚举是 `Copy`，按值传，全文没有给它重新赋值的地方（`grep -n 'first_txg_of_the_new_instance' crates/singlefs-core/src/mount.rs` 去掉注释后 11 处，逐处是声明、传参或这一处读档名）。
- `start.first_txg` 的读者有三处：准入预演（第 5997、6022、6039 行，在 `dry_run_of_the_publishes_after_acquisition` 里）、写行（第 8562、8580、8606 行，在 `acquire_then_publish_until_the_continuation` 里）、暖机计划（`crates/singlefs-core/src/mount.rs:7739`：「warm_up_publish_txgs(parameters, &all_devices, start.first_txg)?;」）。三处取的是同一个值，第 8625 行还断言写行那次发布的 txg 就是它。
- 别处另算「新实例第一个 txg」的：`grep -n 'first_txg_of_new_instance(\|highest_root_txg(' crates/singlefs-core/src/*.rs` 去掉注释后只有第 1512、1557 两行，都在开关这条链里。

**下游用了换过的值的地方（登记许的，不算漏）**：重建分配器拿 `first_txg` 算读坏、读回全零的槽「能住的最大 txg」的区间上沿（第 3295、3607、3613 行），以及根环表的「这个进程已到的 txg」（第 3362 行 `first_txg − 1`）。登记第一节写「下游照用换过的值，其余一步与今天那一档相同」，这几处照用。后果是甲那两档挂载之后抬 F 的上限、可回收的槽会跟着变；E205 若只报写行 txg，这些不进读数，若报「池级 checker 红几条」就会进。这是登记许的，不算漏。

## 三、Q2.3 清扫改调落点核之后，少删、多拒与改法罩不住的一形

**无故障的合法历史：没有少删、没有多拒（读码论证）**。改动只在 `crates/singlefs-core/src/instance_table_sweep.rs:491`：「root_read = RootRecord::parse_slot(&bytes, filesystem_identifier).filter(|root| {」后接落点核。要改前改后结局不同，得有一槽读出自证过、落点不合的根。合法写根环槽的只有两处：`grep -n 'slot_offset(' crates/singlefs-core/src/*.rs`，去掉读、对齐判与测试模块，剩发布路径（`crates/singlefs-core/src/transaction.rs:582`：「device.write_at(」，落点是 `target_for_publish(checkpoint_txg, …)`）与 mkfs 种第 0 代根（`crates/singlefs-core/src/make_filesystem.rs:775`：「let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);」，三个槽 0）。前一种按公式落，后一种落点核另认（`crates/singlefs-core/src/root_ring.rs:188`：「if checkpoint_txg == crate::make_filesystem::GENESIS_CHECKPOINT_TXG {」）。S 是系统不可变配置，两边取同一份。撕裂写过不了整槽校验和，改前改后都算自证不过。所以没有故障时，落点核对每一条读得出的根都放行，候选行逐行相同。你问的「真根恰在别槽的合法形」，只有第 0 代根那三份同字节的根，落点核本来就认。

**有错位读而实例集合不变：多拒一轮（按条款，不算缺陷）**。一槽两读都错位读回别槽的根，而被换掉的实例在环里另有根：改前候选行不变、照删；改后这一轮不删（`ARootRingSlotIsNotSelfVerifiedAfterOneReread`）。这与读者把错位读记成「这一槽这一次读坏」同一口径（`crates/singlefs-core/src/recovery.rs:2010`：「if !root_sits_in_the_ring_slot_the_formula_gives(」），清扫本来就对自证不过的槽这样做。

**打中：错位读回的是第 0 代根，改法罩不住，清扫照删**（推的，待第 2 条）。

盘面（S = 4，区域到盘 [0, 1, 0]，两块 128 MiB 盘；副本用例 `a_misplaced_read_returning_the_genesis_root_over_the_only_ring_root_of_an_instance_still_judges_its_row_to_delete`，造盘面的函数 `pool_with_the_only_root_of_an_instance_in_a_ring_slot_seeded_by_make_filesystem`）。每一步与许可它的那一句：
1. mkfs 把第 0 代根种进三个区域的槽 0（`crates/singlefs-core/src/make_filesystem.rs:775`）。
2. 实例 1 可写挂载，写行 txg 1 落到区域 1 槽 0。发布的写序是单元 → 记录 → 根槽（`crates/singlefs-core/src/transaction.rs:554`：「device.write_at(slot.to_device_offset(), unit, WriteDurability::Plain)?;」、`crates/singlefs-core/src/transaction.rs:562`：「device.write_at(offset, record, WriteDurability::Plain)?;」、第 582 行根槽）。根槽写落盘之后断电，系统配置轮换与暖机 txg 2 都丢。这与 E205 起点镜像「最新那条根的根槽 FUA 持久、两块盘的系统配置轮换没持久」同形，是一次断电，在承诺里。
3. 实例 2 可写挂载：今天的值 = max(根 1, 记录 1) + 1 = 2，写行 txg 2 落区域 2 槽 0；暖机 txg 3（区域 0 槽 1，盘 0）、txg 4（区域 1 槽 1，盘 1）；实例 2 写下实例 1 那一行。之后新建文件 txg 5、覆盖写 txg 6..11，环满（区域 0 槽 0 要到 txg 12 才轮到，仍是第 0 代根）。这几个 txg 是推的，用例只断言「环满、区域 0 槽 0 是第 0 代根、实例 1 只剩区域 1 槽 0 那一条、实例表里有实例 1 那一行」，不写死 txg。
4. 清扫判那一遍：读区域 1 槽 0（盘 1）两次都错位读回区域 0 槽 0（盘 0）的字节。读者照 C973 自己那两条用例的 `PoolReadingOneRingSlotMisplaced`，它同样许错位读回别的盘上那一槽。
5. 读回的是第 0 代根，txg 0：落点核走第 0 代那一支，`crates/singlefs-core/src/root_ring.rs:190`：「.any(|genesis_slot| genesis_slot == ring_slot);」区域 1 槽 0 也是第 0 代根的槽，放行。实例 1 在 `roots_of_the_ring` 里一条根都没有，`crates/singlefs-core/src/instance_table_sweep.rs:363`：「.filter(|row| !instances_with_a_root_in_the_ring.contains(&row.instance))」把实例 1 那一行判成要删。

对照两条（同一份副本）：不错位时 `NoCandidateRow`；把错位读回的换成区域 0 槽 1 那条实例 2 的根（非第 0 代根），C973 改法照拦，停在区域 1 槽 0（`a_misplaced_read_returning_a_root_other_than_the_genesis_root_on_the_same_pool_deletes_no_row`）。

这正是 C973 验收那一句要的盘面：`.claude/kb/checks-owed.md:613`：「造一份满足补的条件的盘面（某槽的真根被换成别槽自证过的根，被换掉的是该实例在环里唯一的根），清扫那一遍必须不删那一行」。第 2 条命令跑出这一格的话，C973 不能按现在的改法销账。

四句：
1. 分不分辨臂：不分辨「改前 / 改后」，两边都删，所以这一格不在改法的射程里。它打中的是两边共用的前提：第 0 代根在三个槽 0 都认（`.claude/kb/decisions/16-发布语义.md:195`：「第 0 代根照 mkfs 种根的槽另认」）。
2. 系统当时看不看得到判别它的东西：单看读回的字节看不到。区域 1 槽 0 合法地住着第 0 代根也是可能的（实例 1 写行那次根槽写丢了），那时实例 1 环里确实没根、删行是对的，两种情形读回的字节逐字相同。看得到的是上下文：环里已有 txg ≥ 1 的根，而区域 1 槽 0 读回第 0 代根，说明「写行那次根槽写丢了」与「错位读」二者必居其一。
3. 满足的是哪一句：C973 验收「某槽的真根被换成别槽自证过的根，被换掉的是该实例在环里唯一的根」两分句都满足，「清扫那一遍必须不删那一行」不满足。
4. 跑前条款的改法在这一格上中不中：C973 去向写的改法（清扫改调读者同一个落点核）在这一格上不起作用。

**射程**：产品阈值是 365 行（`InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE`）。要越过它，得先有 366 次可写挂载、至少 366 个 txg，而这期间一个槽 0 的每一次被轮到的根槽写都得恰好断电丢掉，第 0 代根才留得下。推的：产品阈值下够不着。够得着的是直接调 `judge_the_instance_table_sweep`（C973 两条用例就是这个射程）、只供测试的阈值 `ANY_ROW`，以及没有阈值的后台入口 `sweep_the_instance_table_while_mounted`（今天没有调度器调它）。错位读本身算一次读故障，与 C973 同一类。

**改法（都是我提的，没在任何装置上量过，被攻过零轮）**：
- 改法甲：清扫读到某个区域 r 槽 0 住着第 0 代根，而环里已有落点公式会写进这一槽的、更大的 txg 的根时（r = 1、2 时是任何 txg ≥ r 的根；r = 0 时是 txg ≥ 3S 的根），这一轮不删。代价：写行那次根槽写真丢了的合法形，这一行要等那一槽被下一圈写过才删（推迟，不是删错）。
- 改法乙：mkfs 种的三份第 0 代根带上区域号，进校验和，落点核按区域认。这是格式改动，碰 D22 已定项 8「第 0 代根种进全部根环区域」与 D16 已定项 6「平局只剩第 0 代那几份同字节的根」两句，要另走一轮。

## 四、Q2.4 用例与变异测漏了哪几处

**4.1 去掉「max(今天的值, …)」：没有变异罩着，现有用例全绿**（推的，待第 4、5、6 条）。
- 那一行 `crates/singlefs-core/src/mount.rs:1652`：「Ok(first_txg_today.max(checkpoint_txg_after_the_version(highest_txg)?))」。改它的两行变异 `crates/mutations.tsv:6857`（取 max H）与 `crates/mutations.tsv:6865`（回卷），替换文里都还留着 `first_txg_today.max(`，没有一行把它去掉。
- 现有用例里没有一格让这一项起作用：12 条端到端用例走 Some 那一支的每一格，甲的值都大于今天的值（mkfs 之后 12/24/48 > 1；起点镜像 12/24/48、18/30/54 > 7；六格 7、12/24/48 > 5）。模块里 3 条单元用例走的是 None、到顶与装不下三支。
- 这一项要紧的盘面：起点镜像 S = 4，txg 5 那一槽读坏，txg 6 那一槽读回全零（与「txg 6 的根槽写崩在记录之后、根槽之前」同一张读数），记录读得出。今天的值 7；甲只并读坏 N = 4，只越过 txg 5 那一槽，H = 5。去掉这一项给 6，与实例 1 txg 6 那条记录同 txg，形成同 txg 不同实例的倒挂（登记的「形甲」）。副本里加了两条罩它的：单元用例 `todays_value_wins_when_the_records_run_past_every_ring_slot_passed` 与端到端 `writable_mount_of_the_starting_image_with_records_past_every_ring_slot_read_bad_keeps_todays_value_past_read_bad`，预期 (7, 7, 12)。
- 变异体副本 `copy-mutant-without-todays-value` 预期：现有 3 条单元用例与 12 条端到端全绿（变异活下来）；我加的那两条与对拍那一条判红。

**4.2 后台计算那条产品入口递今天那一档：没有变异行**。
- `grep -c 'FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing' crates/mutations.tsv` 是 5。逐行看，只有 `crates/mutations.tsv:6873`（「产品入口走甲并全零」）的原文锚在 `mount_writable` 那一串，第 9398 行带 `before_the_one_reread,` 的那一处；另四行是别的变异的替换文或原文。
- 第 9556 行（`mount_writable_with_background_compute_settings` 那一串）改成甲并全零，没有变异行判它。调这条入口的用例里，`grep -c 'checkpoint_txg' crates/singlefs-verification/tests/background_compute_settings_and_switch.rs` 是 0，写行 txg 没有用例比。推的：这一处改坏仍绿。
- 补法：在 mkfs 之后 S = 8 那条用例里，加一次经 `mount_writable_with_background_compute_settings`（`cpu` 设置）挂、比写行 txg = 1，再加一行变异。

**4.3 N 含不含被抛弃的根：用例里没有被抛弃的根**。登记要 N 与 `highest_root_txg` 同一个量，`highest_root_txg` 经 `visit_valid_roots`，不按实例表滤（`crates/singlefs-core/src/recovery.rs:2080`：「visit_valid_roots(」）。代码第 1570–1577 行同样不滤，两者对得上。但 12 条端到端用例的盘面只有实例 1 一个实例，环里没有被抛弃的根。有人把 N 换成同文件里滤掉被抛弃根的 `newest_valid_root_txg_of_the_readings`，没有用例会红。这不是一行文本变异能表达的，推的。

**4.4 端到端用例的 S 只取 4、8、16**。3S 永远不是 2 的幂，我想不出只在 S 是 2 的幂时才对的写法，所以这只是取样上的缺口，不是一处漏罩的改法。副本里加了 S = 5、6、7 两条端到端用例（mkfs 之后；起点镜像不注故障与只坏 txg 5），预期见第一节表。

**4.5 清扫两条用例**（`crates/singlefs-core/src/instance_table_sweep.rs` 测试模块）：两条都只错位读回实例 2 的非第 0 代根，罩不住第三节那一形。两行变异 `crates/mutations.tsv:6908`、`crates/mutations.tsv:6909` 各有一条用例判红，读码对得上。我另想的两个改坏读码都会被抓住：把落点核的 S 换成编译期 8，阳性对照那条会停在别的槽；把重读改成读三次，第一条用例第三读读对、不再停。等价改法：窗口循环里的 `break` 换成 `continue`，之后的偏移都装不下，结果相同。

## 五、Q3 `crates/singlefs-core/src/` 里还有没有读根环而不调落点核的地方

有一处，`recovery.rs:5054`，判它不用核。查法（此刻主工作区）：

```
grep -n 'RootRecord::parse_slot(' crates/singlefs-core/src/*.rs | grep -v 'root_record.rs'
grep -n 'slot_offset(\|read_root_ring_slot(\|read_root_ring_slot_telling_never_written_apart(' crates/singlefs-core/src/*.rs
```

第一条的原样输出（`root_record.rs` 里的是定义与它自己的测试，去掉了）：

```
crates/singlefs-core/src/instance_table_sweep.rs:491:            root_read = RootRecord::parse_slot(&bytes, filesystem_identifier).filter(|root| {
crates/singlefs-core/src/mount.rs:3026:    match RootRecord::parse_slot(&bytes, filesystem_identifier) {
crates/singlefs-core/src/recovery.rs:2008:    match RootRecord::parse_slot(&bytes, filesystem_identifier) {
crates/singlefs-core/src/recovery.rs:5054:            (RootRecord::parse_slot(&bytes, &root.filesystem_identifier).as_ref() == Some(root))
```

逐处判：

| 处 | 核不核 | 判定 |
|---|---|---|
| `crates/singlefs-core/src/recovery.rs:2008` `read_root_ring_slot` | 核（`crates/singlefs-core/src/recovery.rs:2010`：「if !root_sits_in_the_ring_slot_the_formula_gives(」） | 恢复、择根、取号、`visit_valid_roots` 一族、`system_configuration_repair.rs:246`、`mount.rs` 第 4017、4029、5210、6957、7215 行都经它 |
| `crates/singlefs-core/src/mount.rs:3026` `read_root_ring_slot_telling_never_written_apart` | 核（`crates/singlefs-core/src/mount.rs:3031`：「if !root_sits_in_the_ring_slot_the_formula_gives(」） | 重建分配器、E2 重读、甲那一遍（第 1691、3426 行）经它 |
| `instance_table_sweep.rs:491` | 核（这一轮的改法） | — |
| `recovery.rs:5054` `root_record_bytes_of_the_version` | **不核** | 不用核：它在环里找一槽、解出来与传进来的那条根逐字段相同，交那一槽的原字节；找不到就按字段编。错位读读回的若是同一条根，字节相同（同一条根只有第 0 代那三份同字节的副本）；若是别的根，比不上、接着找。它不认「哪一槽住着什么」，只取字节，落点错不改交回的值。它在本仓里没有调用者：验证包的挂载透明判官调的是验证包自己那一份（`crates/singlefs-verification/src/mount_transparency.rs:24`：「readable_roots, root_record_bytes_of_the_version, scan_journal, tree_table_entry_bytes_of_root,」从 `crate::verification_read` 引入，定义在 `crates/singlefs-verification/src/verification_read.rs:681`：「pub(crate) fn root_record_bytes_of_the_version(」）；`grep -rn root_record_bytes_of_the_version crates --include=*.rs` 只命中这两份定义与判官那两行 |

第二条命令余下的命中：`make_filesystem.rs:439` 是 mkfs 判对齐的偏移，不读；`make_filesystem.rs:775`、`transaction.rs:583` 是写；`instance_table_sweep.rs` 第 1116、1286 行，与 `mount.rs` 第 11802、11850、13580–13600、13758 行，都在 `#[cfg(test)]` 模块里（`mount.rs` 第 10900 行之后每个 `mod` 的上一行都是 `#[cfg(test)]`，这一条用 awk 核过；`instance_table_sweep.rs` 的测试模块起于第 867、954 行）。这一轮射程只到 `crates/singlefs-core/src/`，`singlefs-checker` 没查。

## 六、我自己提的改法各修哪一格

| 改法 | Q2.3 第 0 代根错位读 | Q2.4 去掉今天的值 | Q2.4 后台计算入口 | Q2.1 N 取自哪一遍 |
|---|---|---|---|---|
| 清扫：槽 0 读到第 0 代根而环里已有更大的、该写进这一槽的 txg，这一轮不删（第三节改法甲） | 推的：修（`a_misplaced_read_returning_the_genesis_root…` 会改判成不删，没实现、没跑） | 不碰 | 不碰 | 不碰 |
| 第 0 代根带区域号（第三节改法乙） | 推的：修，格式改动 | 不碰 | 不碰 | 不碰 |
| 加单元与端到端用例罩「max(今天的值)」+ 一行变异 | 不碰 | 推的，待主 agent 跑第 4、5、6 条（用例已写在副本里） | 不碰 | 不碰 |
| mkfs 之后 S = 8 那条用例加后台计算入口那一挂 + 一行变异 | 不碰 | 不碰 | 推的：修（没写） | 不碰 |
| 登记改成「N 取甲那一遍读数」，或代码改成 N 用 `highest_root_txg` 而读坏的槽另算 | 不碰 | 不碰 | 不碰 | 推的：修字面；E205 的读数不变 |

全部只在我的读码上推过，没在任何装置上量过，被攻过零轮。

## 要主 agent 跑的命令

六条都是指定范围的测试目标。前三条在副本 `copy` 里跑，后三条在变异体副本 `copy-mutant-without-todays-value` 里跑；两份副本都在草稿目录 `.deliverables` 里写成「交接」，用完由主 agent 删。副本上的数不算入库装置上的数。

1. 在 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-opus/copy` 起：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 --lib，筛 first_txg_window_against_the_registration_definition_tests; 范围=1 个测试目标; 为什么=攻方腿副本上加的甲窗口对拍、边角值、今天的值那一项与两遍读数四条用例，核 Q2.1 与 Q2.4 的推的数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-opus/copy -- cargo test -p singlefs-core --lib first_txg_window_against_the_registration_definition_tests -- --nocapture`
   看：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里「test result: ok. 4 passed; 0 failed」，`WINDOW_AGAINST_REGISTRATION cases=… mismatches=0` 一行，`WINDOW_EDGE` 11 行（第一节表的纯算式那几行），`TRANSIENT_PASS registration_reading=Ok(22) code_reading=Ok(11)` 一行。
2. 同一目录起：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 --lib，筛 instance_table_sweep::misplaced_ring_reads_tests; 范围=1 个测试目标; 为什么=攻方腿副本上加的错位读回第 0 代根那一形与阳性对照，核 Q2.3 的打中' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-opus/copy -- cargo test -p singlefs-core --lib instance_table_sweep::misplaced_ring_reads_tests -- --nocapture`
   看：`exit=0`，「4 passed; 0 failed」（原有两条加我两条）；`GENESIS_MISPLACED_READ rows_to_delete=[…]` 一行里有实例 1 那一行。打中成立看这一行；若 `a_misplaced_read_returning_the_genesis_root…` 那一条以「停在了 …」判红，是被落点核或别的拒因挡住，打中不成立，照那一行的拒因判。若以「前提：…」判红，是盘面没造成，打中没核成。
3. 同一目录起：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero; 范围=1 个测试目标; 为什么=攻方腿副本上加的 S 不是 2 的幂、今天的值压过甲、只有第 0 代根、整环读坏五条端到端用例，连同原有 12 条' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-opus/copy -- cargo test -p singlefs-verification --test writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero -- --nocapture`
   看：`exit=0`，「17 passed; 0 failed」；`NOT_POWER_OF_TWO` 6 行、`TODAY_WINS` 1 行、`GENESIS_ONLY` 1 行、`ALL_SLOTS_UNREADABLE` 3 行（三档的拒因原样）。
4. 在 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-opus/copy-mutant-without-todays-value` 起：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 --lib，筛 first_txg_past_the_ring_slots_read_bad_at_mount_tests; 范围=1 个测试目标; 为什么=变异体去掉 max(今天的值) 之后，原有三条单元用例是不是仍绿（Q2.4 漏罩）' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-opus/copy-mutant-without-todays-value -- cargo test -p singlefs-core --lib first_txg_past_the_ring_slots_read_bad_at_mount_tests`
   看：`exit=0`、「3 passed; 0 failed」时，变异在原有单元用例上活下来。
5. 同一目录起：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero; 范围=1 个测试目标; 为什么=变异体去掉 max(今天的值) 之后，原有 12 条是不是仍绿、攻方加的那一条是不是判红' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-opus/copy-mutant-without-todays-value -- cargo test -p singlefs-verification --test writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero`
   看：`exit` 非 0，「16 passed; 1 failed」，失败的只有 `writable_mount_of_the_starting_image_with_records_past_every_ring_slot_read_bad_keeps_todays_value_past_read_bad`（甲只并读坏给 6）。原有 12 条全在 passed 里，变异就在原有端到端用例上活下来。
6. 同一目录起：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 --lib，筛 first_txg_window_against_the_registration_definition_tests; 范围=1 个测试目标; 为什么=变异体上攻方加的对拍与今天的值那一条要判红（证新用例会红）' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-opus/copy-mutant-without-todays-value -- cargo test -p singlefs-core --lib first_txg_window_against_the_registration_definition_tests`
   看：`exit` 非 0，「2 passed; 2 failed」，红的是 `window_value_equals_the_registration_definition_at_every_slots_per_region_and_both_ends_of_u64` 与 `todays_value_wins_when_the_records_run_past_every_ring_slot_passed`。

跑完之后要接着读输出的，新派同立场的腿（带「承接：」这份报告与六个任务号）。编译目录由服务给，主 agent 用完 `task-server/task clean <号>`。

## 没打中的形状（试过哪些、范围多大）

- Q2.1：四种边角全按读码与对拍式推了，范围是 S 4..=16 全部、N 在两端各约 200 个值、越过的槽取单槽、空、整环、除槽 0 外四类、今天的值三档（第 1 条），经真挂载的取 S 5、6、7 与第 0 代根两槽、整环读坏（第 3 条）。代码与登记在这些格上推的都相等，只有「N 取自哪一遍」一格字面不合。
- Q2.2：产品入口两条、只供测试的入口六条逐条看了递的档；挂载里取第一个 txg 的地方全文 grep 只有第 9800 行一处；`InstanceStart` 只构造一次。没有中途换档、没有漏出写行之外（登记许的下游照用除外）。
- Q2.3：无故障的合法形，按「合法写根环槽的只有发布路径与 mkfs 两处」逐类排除；错位读回非第 0 代根，改法照拦（副本里有阳性对照）。打中的只有第 0 代根那一支。
- Q3：四处 `parse_slot` 逐处判，三处核、一处不用核。

## 这条腿自己的限度

- 打中的三格（Q2.3 第 0 代根、Q2.4 两处漏罩）与 Q2.1 的字面一格，数全是推的：用例写在副本里，一条都没跑，编不编得过也没核（只过了 `cargo fmt`，它解析得了语法）。编不过或前提断言红，是我的用例写错，不是打中被推翻；照第 2 条「看」那几句分。
- 第 0 代根那一形的盘面靠一个「某次写落盘之后断电」的包装盘造，挂载在断电之后照跑完（之后的写都丢、读回旧字节）。实例 1 那次挂载交回什么不看，只用断电那一刻的盘面。挂载要是在断电之后 panic，第 2 条会红在 `pool_with_the_only_root…` 里，那时要换断电点。
- 错位读跨盘（盘 1 的槽读回盘 0 的字节），沿用 C973 自己那两条用例的读者。同盘错位在区域到盘 [0, 1, 0] 下造不出这一形（第 0 代根在盘 0 只有区域 0、2 两槽，区域 2 槽 0 要等实例的第二次发布才写；推的）。
- 没跑崩溃点枚举，没在 `memory_pool.rs` 的稀疏盘上建池（用的是两份用例文件各自已有的内存盘），没写模型目录。
- `singlefs-checker` 不在 Q3 射程里，没查。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳。副本上量出的数不算入库装置上的数，判决要引的数由主 agent 复跑。
- 没往统一任务服务交任务，六条命令在上一节。
- 副本 `copy`、`copy-mutant-without-todays-value`（各 51M，`du -sh` 量的）没删：写成「交接」，留给主 agent 跑第 1–6 条，用完由主 agent 删。草稿目录里另有 `patches/`、`copy-attack-tests.diff`（补丁）与几份读码笔记（`mutations-e205-c973.txt`、`sweep-tests.txt`、`minimal-copy.txt`、`fmt*.txt`）。
