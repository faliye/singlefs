# m4-close-fixes-code-r2 攻方腿（Opus，第二轮：第一轮的修补本身、产生结论的方法、从未被看过的地方）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

日期：2026-11-09。被判树：主工作区 `crates/singlefs-core/src/` 七份，与 `research/prompts/m4-close-fixes-code-r2-snapshot/crates-sha256.txt` 比 7 个 OK（量的，`sha256sum -c`）。代码行号都是这棵树的。辩方腿 `research/prompts/m4-close-fixes-code-r2-sonnet-output.md` 已落盘（本轮不禁读），它报过的同形兄弟读与查无此名的变异行，我独立核了一遍，下面只写我核到的。

模型目录 `research/prompts/m4-close-fixes-code-r2-opus-model/`。**编不编得过、绿不绿都没跑**（子 agent 不交任务，只过了 `rustfmt`）。凡标「模型」的都是「推的，待主 agent 跑」；代码行是我现查的。

| 文件 | sha256 |
|---|---|
| `tests/attack_fixes_r2_one_copy.rs` | `8d0a69243c44ed0677eb1070ac88c9e8b68a488554d681713069e8f4a81f4780` |
| `tools/rerun.sh`（照 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps`；模板外部分 `diff` 0 行，量的） | `3f384154e29977980ed141719ea9dacd6daa49e0549c959209b9fc80c37b4799` |

复跑命令在「要主 agent 跑的命令」。

## 各格判定一览

| 格 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| 条款格：D4 已定项 4 与用户定案 | **不成立** | 用户选「按字面全接上」，这批接了判决点名的那几处；同一条推抬 F 路上的另一读、这批自己新加的一读都不计；豁免三类那半句 kb 还是旧话 | 代码、模型 |
| 攻击面 1 映射 key 被换 | **打中** | 条款有 I-1.11「不等判损坏」，产品读者不判；提示过期时第一次发布把映射盖回隔离的旧落点，是 D21 判据说的错答案；只读路径走不到错读 | 代码、模型 |
| 攻击面 2 ① 没读到的坏副本 | 没打中（代码）；kb 过期 | 硬规则射程没扩是对的，D4 第 73 行那半句与实现反着 | 现查 |
| 攻击面 2 ② 攒批读不到待发 | 没打中，条款空白 | 我查的 D16 里没有读自己待发写的条款 | 现查 |
| 攻击面 2 ③ 账的口径 | **打中（轻）** | 同是挂载读到一次，数据单元与映射树叶之后的计数不一样 | 模型 |
| 攻击面 2 ④ 读盘核之后才被拒 | 没打中（推的），没有用例 | 读码走得通，下一次改动搬 | 代码 |
| 攻击面 2 ⑤ 提前返回丢账 | **打中（轻）** | 与同一批修补自己的口径相反；只有造坏的盘走得到（推的） | 代码 |
| 攻击面 2 ⑥ `Relocated.mapping_keys` | 没打中 | 文档写明了，没有消费者据它做事（查到的调用点只有测试） | 现查 |
| 攻击面 2 ⑦ 实例表读不出整批拒 | **打中（轻）** | 一组与实例表无关的过期落点就把好 key 连带丢掉 | 模型 |
| 攻击面 3 账与只试一次 | **打中（轻）**；搬两次没打中 | 再读一次就再试一次，「只试一次」只管没有再读的历史；崩溃之后可写挂载没有用例 | 模型、代码 |
| 攻击面 4 计数 | **打中** | 2 次在被判那段历史上对；放开「F 抬不抬得动」它就不是 2；另有一读这批自己加的不计 | 模型 |
| 攻击面 5 判法 | **打中** | 树表那条断言没有判别力；放回之后 checker 还抓不抓得住产品写错，没有一行证过 | 现查 |
| 攻击面 6 T3 与 newtype | T3 打中（名字），newtype 没打中 | 几处名字与一处死链；`TreeTable` 那一支走不到 | 现查 |
| 检查单：走不到的理由 | 不成立一处 | 报的那条成立；另一支走不到的没报 | 代码 |
| 检查单：类型与名字 | 不成立 | 见攻击面 6 | 现查 |
| 检查单：不兼容旧代码 | 成立 | `grep` 0 条 | 量的 |
| 检查单：不新建判定库 | 成立（只核 diff 与两个新目标） | | 现查 |
| 检查单：多机可用 | 成立；正文一句要改 | `crash_points_…` 用临时镜像文件，不是「盘都在进程里造」 | 现查 |
| 检查单：测试优先多线程 | 成立 | | 现查 |
| 检查单：带 --item | 推不出（算打中） | 表上 28 行对得上；两份测试文档点名的行不在表里；GPU 那一行点全量域，基线在共用卡上红过 | 量的 |
| 检查单：新崩溃流 | 推不出（算打中） | 两批与下界判法确实是新的；「坏盘 × 部分落盘 × 可写挂载」没有一处判 | 现查 |

## 攻击面 4：读实例表、读盘核、搬留存读源接上计数之后

### 同一条推抬 F 路上还有一读，不计（用户定案那一格）

判决第三节用户选的是字面：`research/prompts/m4-close-fixes-code-r1-main-verification.md:36`：「用户 2026-11-09 弹窗：」，「别的读」那一格选的是按字面全接上。条款：`.claude/kb/decisions/04-校验和位置.md:73`：「可写挂载里的读都计数并搬，包括部分写与截断的读回、可写挂载时建上一版那一读、可写挂载期间别的读」。

推抬 F 的入口 `push_one_floor_raise_within_the_admission_budget` 先算上限，上限那一步沿链读实例表，读者是盘表：

- `crates/singlefs-core/src/mount.rs:4994`：「let ceilings = admission_ceilings_on_caller_inputs_agreeing_with_the_disk(」
- `crates/singlefs-core/src/mount.rs:4801`：「let current_instance_table = instance_table_chain_of_root(devices, &current.root)」

接上计数的是后面 `raise_the_floor_through` 那一读。抬得动时两读都发、后一读记账，数上看不出差。**抬不动时只有不计的这一读**：`FloorAlreadyAtTheCeiling` 在 `raise_the_floor_through` 之前交回。

可达历史（模型 `the_chain_read_for_the_admission_ceiling_counts_nothing_when_the_floor_cannot_be_raised`，每步指到许可它的代码）：

1. 三个单元的池、可写挂载；弄坏数据单元 1 的一份，`read_the_file` 读到（别的读 = 1，`crates/singlefs-core/src/mounted_session.rs:2694`：「可写挂载期间的用户读文件：读现行那一版」）。
2. 弄坏实例表第 0 片一份（对照：套一层数只坏一份的读者沿链读，数到 1）。
3. 建一个空 inode；另发那一批被空间拒一次（只供测试的开关；真拒走同一支，`crates/singlefs-core/src/mounted_session.rs:2304`：「if !compaction_publish_refused_short_of_space(&refusal_of_the_relocation) {」）。
4. 非空根不足 4 条，F 抬不动（实现员量到的 `FloorAlreadyAtTheCeiling { floor: 0, ceiling: 0 }` 就是这个形状，`~/.local/share/singlefs/work/m4-close-fixes-writer-3/result.md` 第 16–18 行）。

期望（推的，待主 agent 跑）：`NoSpaceAfterRaisingTheFloor`；别的读仍是 1；下一次改动 `NotNeeded`；实例表不换。

用户动作放开扫：弄坏之前多发 0、1、2、3 次改动（决定 F 抬不抬得动），只固定前缀与故障。只有结局是 `NoSpaceAfterRaisingTheFloor` 的那几格钉断言，别的格打观测行。预期 1（正好 4 条非空根，边界，没把握）、2、3 这几格抬得动，别的读 2、实例表下一次换（推的）——所以这一格是「用户动作定的」，不是装置写死的。

被判用例那条 2 次：`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:636`：「"推抬 F 沿链读实例表一次、搬留存单元读源一次"」。**在它那段历史上对**：弄坏之前有建快照与部分写，非空根够，F 抬得动。我逐步数了：部分写读回进另一个数；开关先拒，推抬 F 读到实例表这一组（1）；再发那一批读留存源，那一组（2）；释放之前读盘核同一组不再加。它罩不到抬不动那一格。

### 这批自己新加的一读，不计，也不修

另发那一批认位置时，账上还有认不出的组就沿链读实例表：

- `crates/singlefs-core/src/mounted_session.rs:2163`：「let repair = match repair_of_the_exempt_units_at_these_locations(」，读者是下一行的 `&*devices`。
- `crates/singlefs-core/src/transaction.rs:11966`：「let chain = instance_table_chain_of_root(reader, &version.root)?;」
- 紧接着只比账上那几组：`crates/singlefs-core/src/transaction.rs:11967`：「let any_page_read_with_one_copy_failing = chain」

这一读读到的坏副本不进账、不换：它不经数只坏一份的那一层，判「要不要换」只看账上已有的组。

可达历史（模型 `the_chain_read_of_the_repair_itself_counts_nothing_and_leaves_the_damaged_page_in_place`）：弄坏数据单元 1 一份与实例表第 0 片一份；整份顺序写。释放之前读盘核把那个数据单元记上（别的读 = 1，被判用例 `the_release_check_reading_one_damaged_copy_…` 同形）；这次发布已换下它，认不出 key，进 `locations_without_a_mapping_key`；另发那一批为这一组沿链读实例表，读到坏的那一份，`NotNeeded`；账清空。对照数到 1。下一次改动账是空的，连这一读都不发，实例表一直带着坏的那一份。期望（推的，待主 agent 跑）：别的读始终是 1，实例表不换。

这一读是这批修补写进来的，算「第一轮的修补本身」。

### 别的同形读

| 读 | 代码 | 进不进账 | 说明 |
|---|---|---|---|
| 整理门读根环与树表 | `crates/singlefs-core/src/mount.rs:4256`：「let table = instance_table_chain_of_root(reader, current_root)?.records;」，会话调用点 `crates/singlefs-core/src/mounted_session.rs:3218`：「let publishes_read_from_the_ring = rollback_states_of_the_ring(」、`crates/singlefs-core/src/mounted_session.rs:4098`：「match rollback_states_of_the_ring(」 | 不进 | 现行那一版的实例表，修得了 |
| 回退读现行实例表 | `crates/singlefs-core/src/mount.rs:10728`：「let current_table = instance_table_chain_of_root(&*devices, &current.root)」 | 不进 | 会话内回退走这里（`crates/singlefs-core/src/mount.rs:10980`：「会话内回滚：挂着的会话上做一次」） |
| 回退的释放之前读盘核 | `crates/singlefs-core/src/mount.rs:10847`：「unit, &locations, &*devices,」 | 不进 | 与接上计数的发布路径同一段读法，这一处没套 |
| 清扫 | `crates/singlefs-core/src/instance_table_sweep.rs:356`：「let rows = instance_table_chain_of_root(reader, current_root)」、`crates/singlefs-core/src/instance_table_sweep.rs:741`：「let chain = match instance_table_chain_of_root(&*devices, &current_file_version.root) {」、`crates/singlefs-core/src/instance_table_sweep.rs:915`：「let rows = match instance_table_chain_of_root(&*devices, current.root()) {」 | 不进 | 挂载末尾与挂着时后台 |

我没有把全部读穷举完（`region_scan`、整理读没逐个核），这张表只说我核到的。

四句：① 分辨臂：分辨「按字面全接上」与「只接判决点名的四处」；用户选了前者。② 系统看得到：这几处手里都有分配器或能拿到（`admission_ceilings_…` 手里有 `allocator`），改读者即可，不是判别子观测不到；回退读的是旧版本的那几读除外（读到了也没有东西可搬）。③ 字面分句：D4 第 73 行「可写挂载期间别的读」。④ 改法见改法表第 1、2 行。

### 同一份坏副本重复计入、口径不一致（也是设计问题第 3 条）

账的口径：`crates/singlefs-core/src/allocator.rs:1870`：「读到只坏一份的次数：一组位置条目等着下一次用户改动的这一程里只算一次。」

挂载交回之前按 key 分两种处置（`crates/singlefs-core/src/mount.rs:7876`：「allocator.forget_the_locations_read_with_one_copy_failing(|locations| {」）：认出 key 的那几组**从账上去掉**，认不出的**留在账上**。于是同样是「挂载读到一次」（模型 `the_other_reads_count_differs_between_a_data_unit_and_a_mapping_tree_leaf_read_while_mounting`，期望推的）：

| 之后发生的 | 数据单元（认出 key） | 映射树叶（认不出） |
|---|---|---|
| 再经会话读一次 | 别的读 +1 | 别的读 +0 |
| 没人再读，头一次改动那一批换下 | 别的读 +1：那一批**自己的**释放之前读盘核读到源落点上那一份，账上已经没有这一组 | 别的读 +0 |

数据单元那一列第二行的 +1 不是任何人多读了一次：是修复动作自己的读盘核把同一份坏副本再记一笔，紧接着又被 `crates/singlefs-core/src/mounted_session.rs:2207`：「relocation.source.slot == location.slot」那一判从账上去掉。一个池上挂载计 1、别的读计 1，读者把两个数相加得 2。

四句：① 不分辨臂（条款没定口径）。② 系统看得到。③ D4 第 73 行只说「计数」，没说次数怎么数，所以**不算违条款**，算口径自相矛盾：同一个函数文档说「一组……只算一次」。④ 实现员第 3 条问的「要不要统一」：改法表第 3 行是最小的一种，只修这个不对称；要不要四种口径统一仍交用户。

读失败时账与数：`mount.rs` 与 `discard` 那两处都是读完先记账再判读成没读成（diff 里 `instance_table_chain_read`），没打中。`relocations_checked_against_the_version` 同。提前返回那一处见攻击面 2 ⑤。

## 攻击面 1：映射 key 被换、封得对的盘

**条款有没有写：有。** `crates/singlefs-checker/invariants/01-块自描述.md:17`：「不等判损坏。」（I-1.11（映射 key 与单元头相符），`walk_central_mapping_entries` 已判）。这是 checker 的条款，「产品读者要不要同判」我没查到明写的句子；实现员报告说「条款没写」只对这后半句成立。

**映射树是派生态还是权威态，分两段答**：key 各段照 I-1.11 可从单元头重算；位置那一段，单元被搬之后只有映射知道。D21 的判据：`.claude/kb/decisions/21-权威态与派生态的分界.md:219`：「问的是语义，不是字节。」，同一行接着按保守答案还是错答案分。这一形的答案取决于提示过没过期：

| 形 | 重建之后这把 key 的落点 | 判据 |
|---|---|---|
| 提示没过期（被判用例 `a_disk_with_a_key_replaced_…`） | 提示 = 映射原来的落点，盖回去是对的 | 保守答案 |
| 提示过期（**这批修补让它走得到**：另发那一批只改映射，父节点里的提示还指旧落点） | 查不到就留提示：`crates/singlefs-core/src/recovery.rs:3718`：「mapping_entries.mapping_position_of_key.get(key).copied()」；第一次发布按指针重算，把这把 key 盖回**隔离着的旧落点**，搬到的新槽没人引用 | **错答案** |

可达历史（模型 `a_key_replaced_after_a_relocation_points_the_mapping_back_at_the_quarantined_copies`，期望推的）：弄坏 `Data(2)` 一份 → 经会话读 → 建 inode，那一批搬到新槽 → 盘上那一条 key 换成事务号加一（造法照被判用例，封得对、仍升序）→ 可写挂载。期望 `mapped_locations(挂载之后, key)` 是旧落点。接着的改动与 checker 结果打观测行，不钉（我猜那一批在 `placement_to_release_after_checking_every_device` 被拒、checker 报 I-3.15，推的）。

被判用例 `crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:1265`：「"这把 key 回到映射里"」把「盖回」钉成了对的行为，只在提示没过期那一形上真。

重建只数条数：`crates/singlefs-core/src/recovery.rs:3607`：「mapping_entry_count_is_one_per_mapped_unit(」。key 集合就在内存里（`mapped_units` 与叶条目），核相等不多读一次盘。

**只读路径走不走得到错读：没打中。** `read_at` 与 `read_mapped_tree_node_via_hint_then_central_mapping` 都先按提示读、按提示里的整单元校验和判；提示对不上转查映射，查不到报错（`MappingMiss`，`RecoveryFailure` 那一支）。换 key 不改位置条目与单元字节，读不出别的单元的字节（读码，推的）。

四句：① 分辨臂：分辨「拒」与「收下盖回」，只在提示过期那一形上两臂结局不同。② 系统看得到：key 集合都在内存。③ 满足的是 I-1.11「不等判损坏」对盘的判定，产品一侧是 D21 已定项 10「错答案 ⇒ 必须逐出该结构，或者进权威态」的判据（这句是判据，不是对这一形的定案，所以**要不要拒要交用户**）。④ 改法表第 4 行；它把「查不到 key 照计」在建上一版那条路上变成走不到，修补第 3 条的用例要跟着改。

**这一形的分量要说清**：封得对而 key 错，不是设备故障造得出的，要有写者缺陷或内存翻转（推的）。不伤正常盘。

什么现象推翻：模型那一条挂载被拒或落点是新槽。

## 攻击面 2：实现员七条

**① 没被读到的坏副本被换下时不核**：代码没打中。`copies_of_the_exempt_units_read_with_one_copy_failing` 只跟着账上的组读，D19 已定项 5 硬规则 1 的射程没扩。但 kb 那半句与实现反着：`.claude/kb/decisions/04-校验和位置.md:73`：「它们在挂载这一串的发布里本来就换落点，旧落点走释放之前读盘核那一路（推的，多片实例表链里不被改写的那几片没核）」。今天「本来就换」对映射树叶是假的（已改成另发那一批），「走读盘核」只对记着位置的成立，多片链不存在不被改写的片。r1 判决第五节说随这批收尾写回，写回时这三处都要改。

**② 攒批会话读不到待发写**：没打中，条款空白。`crates/singlefs-core/src/mounted_session.rs:6121`：「pub fn session(&self) -> &MountedSession {」交出来的 `read_the_file` 读 `current`。`grep -n '可见\|读得到\|读到自己' .claude/kb/decisions/16-发布语义.md` 里没有讲读自己待发写的句子（量的，我只查了 D16）。这一条交用户前要先查 POSIX 语义相关的决策，我没查全。

**③ 账的口径**：见攻击面 4 末节，打中（轻）。

**④ 发布在读盘核之后才被拒**：没打中（读码，推的）。读盘核记账在 `publish_admitted` 之前（`crates/singlefs-core/src/transaction.rs:13467`：「allocator.count_the_other_unit_reads_with_one_copy_failing(」），分配器换回不退账；改动没发成时 `relocate_…` 不跑，账留到下一次发成的改动，单元还在原落点就认得出。没有用例是真的，模型也没写。

**⑤ 提前返回丢账**：打中（轻）。`prepare_the_version_publish_relocating_units` 里 deadlist 那一圈：`crates/singlefs-core/src/transaction.rs:13403`：「let role = role_standing_for_the_unit_of_a_deadlist_entry(entry)?;」、`crates/singlefs-core/src/transaction.rs:13413`：「.map_err(not_releasable)?;」，这两个 `?` 在已经读过的那几条之后返回，第 13467 行的记账走不到。搬源那一圈的 `refuse_locations_that_do_not_name_two_pool_devices(…)?` 同形。同一批修补另一处写的是相反的口径：`crates/singlefs-core/src/transaction.rs:10882`：「有一项核不过时前面几项读到的也进」。走到要盘上条目的落点指池外（只有造坏的盘，推的）。四句：不分辨臂；看得到；D4 第 73 行「都计数」；改法表第 6 行。

**⑥ `Relocated.mapping_keys` 含本来就重写的节点**：没打中。文档写了；`grep -rl` 全部 `crates/` 里点这个类型或字段的，产品代码只有定义它的 `mounted_session.rs` 与注释里提到它的 `compaction_plan.rs`，别的是三个测试目标（量的），没有据它做事的产品代码。

**⑦ 实例表读不出整批拒**：打中（轻）。`crates/singlefs-core/src/mounted_session.rs:2174`：「RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(」。账上那一组可以与实例表毫无关系（这次改动已换下的数据单元）：`locations_without_a_mapping_key` 非空就要读链，链读不出就连带把这一批要搬的好 key 丢掉，只试这一次。模型 `a_stale_location_and_an_unreadable_instance_table_drop_the_relocation_of_an_unrelated_key`：弄坏 `Data(0)` 与 `Data(2)` 各一份、读到；两份实例表都弄坏；部分写重写单元 1、2。期望（推的）：`RefusedNotForSpace(InstanceTableOfTheVersionUnreadable)`，`Data(0)` 留在坏落点。实例表既然读不出，这一批本来也写不了 `Rewrite`；跳过实例表照搬同样在任何写之前。分量：两份实例表都坏的池下一次可写挂载建上一版就拒（`crates/singlefs-core/src/mount.rs:1407`：「let table = instance_table_chain_of_root(reader, root)」），所以这一形丢的是**可写挂载里最后一次把数据单元搬好的机会**。四句：不分辨臂；看得到（读不出是当场知道的）；条款没写，实现员已说明；改法表第 5 行。

## 攻击面 3：只试一次、被空间拒推抬 F 再试

**账什么时候清**（`crates/singlefs-core/src/mounted_session.rs:2117`：「这几组与这一批要搬的源落点上记下的，发完（发成或被拒）都从账上去掉」）：逐个出口看了，实例表读不出、两样都空、发完三个出口都清；改动没发成不清（`published?` 在前面）；挂载结束账随分配器丢（只在内存）。没打中。

**退回与拷贝**：`allocator.rs` 的账是 `Arc`，发布被拒换回分配器、预演走拷贝都共用；分配器只有挂载时 `rebuild_from_records` 新建，会话期间我查到的 `*allocator =` 都是拷贝换回（`grep` 量的，`mount.rs` 五处、`transaction.rs` 四处）。没打中。

**重试时计划与 `repair` 不重算**：`repair` 在推抬 F 之前算一次，再发时接在推过的那一版后面。我怕 `Rewrite.replaced_chain` 过期撞断言（「计划带着的旧链是上一版根记录指着的那一条」）：抬 F 的空发布照抄实例表（`crates/singlefs-core/src/mount.rs:5702`：「instance_table: InstanceTablePlan::Carry(current.root.instance_table),」），不过期。没打中。一个不伤事的尾巴：抬 F 的空发布可能已经换下记着的那片叶（叶里有分配记录树节点的 key 时，推的），结局照样把它报进 `units_exempt_from_the_mapping_replaced`，文档写「同一次发布换下」对不上；路径重写多写一遍好叶。

**同一组搬两次**：没打中。key 只进一批，`relocations_of_the_units_read_with_one_copy_failing_while_mounting` 按 key 去重（挂载那几把与别的读那几把拼起来也走它）；搬完映射换新槽，旧组认不出。崩溃之后：批落盘了，下一次挂载读新槽；批没落盘，账本来只在内存，挂载读到再记一次、再试一次（读码，推的）。

**「只试一次」的射程：打中（轻）。** 条款 `.claude/kb/decisions/04-校验和位置.md:73`：「那一批搬没搬成都只试这一次，结局随那次改动报出，之后的用户改动不再接着搬」（这句说的是挂载读到的）。类型文档把它推到全部三处：`crates/singlefs-core/src/mounted_session.rs:225`：「那一批搬没搬成都只试这一次，结局报在这里，之后的用户改动不再接着搬。」。可是账清掉之后同一组再读到就再记，下一次改动再试（`crates/singlefs-core/src/mounted_session.rs:2686`：「一组位置条目等着下一次用户改动的这一程里只算一次，处置过之后再读到再算」）。模型 `a_batch_refused_not_for_space_is_tried_again_once_the_unit_is_read_again`：读—改动（被拒）循环三遍，期望（推的）三次都是 `RefusedNotForSpace`、别的读 3。被判用例 `crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:984`：「fn a_mapping_tree_leaf_rewrite_refused_not_for_space_is_not_tried_again_by_the_next_change() {」两次改动之间没有读，罩不到。

不用用户动作也会再记：实例表片被拒之后，同一次 `publish_user_change` 里紧接着的 `crates/singlefs-core/src/mounted_session.rs:2886`：「published.discards_after_the_change = discard_the_segments_reusable_as_a_whole(」只要有段要 discard 就再读一遍链、再记一笔；空间紧时每次改动都可能为这一批推一串抬 F，掉回退候选（推的，没造）。

四句：① 分辨「每个坏副本只试一次」与「每读到一次试一次」。② 系统看得到（再记一笔只要留一张试过的组）。③ D4 那句的字面只管挂载读到的，别的读「同形」是 r1 判决第四节第 2 条的话；类型文档比条款说得宽，是文档与行为不一致，**条款上不算违**。④ 改法表第 7 行：要么文档收窄（我推荐），要么记试过的组（代价：账只增不减）。

**崩溃点上**：账不落盘，下一次挂载靠重读。被判流只对放回字节的基线判状态、只对没放回的盘跑 `recover`：`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:453`：「let report = recover(&image, JournalPolicy::Consult);」。**崩溃之后可写挂载（真坏盘、部分落盘）一处都没判**，这条读码没打中，但没有观测；见攻击面 5 与「这条腿自己的限度」。

## 攻击面 5：放回字节再判，够不够

被判用例自己写明了这一判看不见什么：`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:377`：「放回之后 checker 看不见现行版本还引用不引用坏的那一份」。逐样看：

**树表那条断言：没有判别力（打中，实现员自己写了偏弱，其实是零）。** `crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:1049`：「"现行版本不再引用弄坏的那个树表落点"」。树表每次发布都 COW 换新槽，这条对任何一次发布都真，主动修复整个删掉也真；紧接着「换到新槽」「新槽上那一份新写」两条同样。这条用例的牙全在 `crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:1062`：「"旧落点两份都隔离"」。同样的话适用于放回之后 checker 那一判：隔离没做，那一槽也只是进 defer、这么短的历史里不复用，`put_the_damaged_byte_back` 放得回去，checker 绿。

**放回之后 checker 还抓不抓得住产品写错：没有一行证过（打中）。** 实现员把「实例表那一批丢掉最新一行」写成这一判的唯一证明，证红时没红，分类成取样点不敏感（`records/2026-10-13-里程碑四预开发.md:1570`：「属 `.claude/rules/mutation-sampling.md` 第三类取样点不敏感」）。记录写了换成「不释放换下的旧链」重证（同一行），最后换成的却是 `主动修复崩溃流：判之前不把弄坏的字节放回基线`（`records/2026-10-13-里程碑四预开发.md:1571`：「判之前不把弄坏的字节放回基线」）。这一行证的是反方向：不放回就红。于是 182 个状态绿这句话的判别力只有用例里那几条断言撑着，放回之后 checker 有没有在判这一批写出来的东西，表上没有证据。这条是**检查判据自身的证红缺口**，照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」。实例表丢行 checker 不判这件事本身也要记账（`walk.rs` 只判被抛弃）。

**`version_references_the_location` 的范围**：覆盖树表、映射树根与节点、实例表链、进映射单元、留存条目（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:226`：「locations.extend(version.root.tree_table.locations);」起）。它不覆盖提示：搬迁之后父节点里的提示还指旧落点，读者照样先读坏的那一份再读旧落点另一份。所以「现行版本不再引用」对提示那一层不是字面真话，读也没有变快；这是混合位置权威的已知形态，不算打中，文档用词要知道。

**只弄坏头一份、只翻一个字节**：`damage_the_copy_at` 每条都打 `locations[0]`。第二份坏时读路径根本读不到它（头一份对上就停），只有释放之前读盘核看得到：这一支记账与隔离的组合没有用例。读回 `None`（设备读错）也没有用例，账走同一支（读码，推的）。

**多片实例表链**：实现员报告没有用例。模型 `an_instance_table_chain_of_two_pages_with_the_second_page_damaged_is_rewritten_whole` 补这一形（挂载 375 次，不扫，第二片一份坏）。我读码没看出错，等跑。

**替代方案（只在模型上提，被攻过零轮）**：修好之后卸载（F 抬到现行那一版，旧根离开候选集），**不放回**，判真盘。I-2.1、I-4.8、I-7.4 的条款射程都是候选集（`crates/singlefs-checker/invariants/06-事务与崩溃一致性.md:11`：「从回退候选集」），所以真盘理应绿。绿就不用放回，判的是用户手里那张盘；红就是放回之前看不到的东西。模型 `the_pool_checker_on_the_disk_with_the_damage_still_on_it_after_the_repair_and_the_unmount` 五个臂只打观测行、不钉断言（跑之前写下：绿我就撤回「放回遮东西」这半句，只留「要有证红」；红就逐条判是不是产品缺陷），头一个臂是没修的对照，该报 I-2.1 或 I-4.8，不报就说明这一判没牙。

**「坏盘 × 部分落盘」没判**：基线放回之后枚举状态：`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:266`：「put_the_damaged_bytes_back_on_the_baseline(&mut base, &writes, damaged);」。不放回的盘只走录制流前缀：`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:447`：「image.apply(&operations[..operations_before]);」，读者是冷恢复。所以「这一批写了一半、坏副本还在、接着可写挂载」这一形三件事都没碰。

四句：① 不分辨产品臂，分辨的是判法。② 不涉及。③ 打中的是正文第五条问的「够不够替代」；字面答：结构类不变量够，现行版本引用够，checker 对这批写的判别力**推不出**。④ 改法表第 8、9 行。

## 攻击面 6：T3 改名之后、newtype

**newtype 没打中**：产品调用点两处都换了（`recovery.rs` 第 7021 行、`mounted_read.rs` 第 574 行，`grep` 量的），`InodeNumber`、两个新 newtype 只在这一判里用，排不错。

**T3 成员语义没打中**：`units_exempt_from_the_mapping_replaced` 只在结局里报，别的调用点只有测试（上面 ⑥）。那个重试尾巴见攻击面 3。

**名字打中（检查单：类型与名字）**：

| 名字 | 为什么说不对 | 出处 |
|---|---|---|
| `relocations_of_the_units_read_with_one_copy_failing_while_mounting` | 现在也收会话读文件、推抬 F 等记下的 key | `crates/singlefs-core/src/transaction.rs:12123`：「pub fn relocations_of_the_units_read_with_one_copy_failing_while_mounting(」 |
| 文档死链 `mapping_keys_of_the_units_at_these_locations` | 函数改名成 `units_at_these_locations`，这一处没跟 | `crates/singlefs-core/src/transaction.rs:12116`：「[`mapping_keys_of_the_units_at_these_locations`] 认出来的」 |
| `RelocationKeptForTheSnapshotsRefusedForSpace` | 只供测试的开关，强制的是整批，没有快照也进 | `crates/singlefs-core/src/compaction_plan.rs:527`：「pub enum RelocationKeptForTheSnapshotsRefusedForSpace {」 |
| `RelocationKeptForTheSnapshotsRefusedNotForSpace` | 同上 | `crates/singlefs-core/src/compaction_plan.rs:543`：「pub enum RelocationKeptForTheSnapshotsRefusedNotForSpace {」 |
| `FloorRaiseForTheRelocationKeptForTheSnapshotsFails` | 同上 | `crates/singlefs-core/src/compaction_plan.rs:554`：「pub enum FloorRaiseForTheRelocationKeptForTheSnapshotsFails {」 |

实现员说开关名没改是因为牵 20 多行变异，这是理由不是结论，交主 agent 定。

## 第四节八格逐格

**检查单：走不到的理由——不成立一处。** 实现员报的那一条成立：`crates/singlefs-core/src/transaction.rs:10937`：「"另发的那一批要么搬单元、要么换豁免映射的单元：调用方两样都空就不发"」（报告写第 10935 行，今天 `assert!` 在第 10936 行，差在这批之后的改动，不是报错）。调用点只有 `publish_the_relocation_batch_once`，上游两样都空已交回 `NotNeeded`，推抬 F 重试不改 `planned`、`repair`。

没报的一支：`crates/singlefs-core/src/transaction.rs:11953`：「} else if version.root.tree_table.locations == *locations {」。要走到它，账上得有**这次改动发成之后现行那一版**的树表落点；账上的组都是这次发布之前读到的，树表每次发布都换新槽，而从改动最后一次发布到认位置之间没有读（读码，推的）。映射树根经 `zip(&mapping_tree.pointers)` 匹配那一支同理。`grep` 这一句在 `crates/mutations.tsv` 与 `crates/singlefs-verification/tests/` 里 0 命中（量的）。`RepairOfTheExemptUnits` 文档还把树表写进「换下的角色」次序里。这不是 `unreachable!`，但属于这一格问的东西，也对不上 `.claude/rules/fs-design.md` 硬要求第 2 条（每条分支能被测试强制进入；公开函数可以直接调，今天没有用例调）。

`expect` 依赖的不变量逐处看：账的锁、片序号没事；`a_mapping_key_below_the_node` 三个 `expect`，空节点读树时判过（`crates/singlefs-core/src/code_two_tree.rs:1017`：「if header.entries.is_empty() {」），只有节点在 `mapping_tree` 形状里才进来，成立。一处小矛盾：`.expect("账的锁没有中毒：持锁的几处只做加法与表操作")` 与持锁时 `checked_add(1).expect(…)` 同在，越界不可达（推的），不伤。

**检查单：类型与名字——不成立。** 见攻击面 6 的表。newtype 成立。

**检查单：不兼容旧代码——成立。** `grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-fixes-code-r2-diff.md` 0 条（量的）。

**检查单：不新建判定库——成立。** diff 只在 `crates/singlefs-core/src/`；两个新测试目标判定走 `GivenStatesEngine`，没有另开存放处（只核这两份）。

**检查单：多机可用——成立，正文一句要改。** `unit_read_…` 盘在进程里；`crash_points_…` 不是：`crates/singlefs-verification/tests/common/mod.rs` 的 `build_pool` 是文件镜像（实现员第一份报告「拒算的测试」一节说的是对的，正文「报告称盘都在进程里造」只对前一个）。两者都不读本机独有状态（读码）。

**检查单：测试优先多线程——成立。** `crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs` 单池单场景，判定处带 `one_test_changes_the_environment_at_a_time`；状态表交引擎判。规则那一句我只读到 `.claude/rules/verification.md` 第 57 行的单卡带 `--test-threads 1`，没读全份。

**检查单：带 --item——推不出，算打中。**

- 28 行对得上：表里前缀计数 9 + 6 + 2 + 2 + 1 + 1 = 21 行，加 7 行整行替换（量的，`grep -c`；替换的 7 行我按实现员报告算，没逐行对）。
- 两份用例的判别力文档点到不在表里的行：`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs` 第 584 行点「现行版本不再引用弄坏的落点：搬留存单元之后留存条目还记旧落点」，`crates/mutations.tsv` 里 0 命中（量的）。
- GPU 那一行第五、六列点的是全量域那条测试名，不是缩域。同形兄弟流的注释写了为什么不这么点：`crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:21787`：「这条流全量域约两千万态，变异行不点名这一条」。这条流的基线在共用卡上红过：`records/2026-10-13-里程碑四预开发.md:1569`：「GPU 新流基线红（证红 10661）是」，同一行写了全量域跑 1895 秒、派活超时 300 秒。整表归提交时的话，这一行在提交时大概率再撞一次（推的）。「挂钟长不是收窄理由」说的是不收窄崩溃点，不等于这一行跑得成。
- 同形第二处 `crates/singlefs-verification/src/crash_judge_recovery.wgsl:663`：「if (lt64(inode, separator) || lt64(inode, identity_container)) {」（建上一版）没有锚这一句的变异行（量的，`grep` 只命中一行、锚点是第 286 行那一处）。r1 判决那一格点的是打开那一处，这一处算同形漏。
- 带 `--item` 门禁任务取消、余下证红没出结局（正文第二节），这些不算判过。

**检查单：新崩溃流多罩了什么——推不出，算打中。** 分三层答：

- 确实新的：实例表行不变的整链重写、映射条目不变的路径重写，两者旧落点带隔离记号落进分配记录（我查到别的流上 `InstanceTablePlan::Rewrite` 只来自写行、回退、清扫）。写的形状本身与搬迁批（映射叶到根）相近，真正新的是隔离记号落在这几个角色上。
- GPU 新流：分隔 key 改高确实只有它，判的是第 286 行那一判。缩域到 17，全量没跑。
- 状态口径窄：`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:230`：「if segment.len() < WRITES_OF_A_SEGMENT_EXPANDED_BY_SUBSET_BELOW {」，十写以上的段只有 `crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:239`：「for prefix in 0..segment.len() {」。标记 `crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:314`：「crash-case-check:not-a-crash-case」让它不进崩溃点重放。实现员交了问题（`~/.local/share/singlefs/work/m4-close-fixes-writer/result.md` 第 99 行「不是全量枚举，长段只按前缀」，交主 agent 定），我没在正文与记录里看到定了什么。共用约束：`.claude/agent-common.md:73`：「**崩溃点测试准确率和正确率优先，其次再衡量时间成本**」。这批是新的发布形态，按这一句该走 `enumerate_crash_states` 一族、登记进重放；「一个合成流、单跑几秒」不是收窄的理由。我没量全量枚举的状态数，交主 agent。

## 条款格：D4 已定项 4 与用户定案

不成立，三处：① 「按字面全接上」：攻击面 4 那两读与那张表。② 第 73 行「豁免三类……旧落点走释放之前读盘核」的字面与实现不一致，写回未做（正文已说随这批写回）。③ 「只试一次」的类型文档比条款宽（攻击面 3）。

成立的部分我逐个对过：点名的四处都接了；三类的主动修复在被判历史上走得通（代码；两轮用例主 agent 量过 11 条过）；查不到 key 不改记没坏三处都改了；T3 主名字改了。

## 改法表（都只在我的模型上提过、被攻过零轮；今天连模型都没跑）

| 行 | 改法 | 修哪一格 | 还剩什么 | 量过 / 推的 |
|---|---|---|---|---|
| 1 | `admission_ceilings_on_caller_inputs_agreeing_with_the_disk` 与另发那一批认位置的那一读都经数只坏一份的读者并记账（认位置那一读记到的片，同一批接着换） | 攻击面 4 头两节 | 回退、清扫、整理门那几读 | 推的，待主 agent 跑 |
| 2 | 那几读要么照接，要么把 D4「别的读」收窄成「读现行那一版、有东西可换的读」，交用户 | 同形读那张表 | 用户上次没选收窄 | 推的 |
| 3 | 挂载交回之前认出 key 的组也留在账上（不加次数），到另发那一批再去掉 | 攻击面 4 末节 | 四种口径是否统一 | 推的，待主 agent 跑 |
| 4 | 建上一版核「映射 key 集合 = 进映射单元的 key + 留存条目的 key」，不等拒（内存里比，不多读盘） | 攻击面 1 | 交用户：判别的是写者缺陷，不是设备故障；`a_disk_with_…`、`a_read_back_…` 要改；挂载以外的读者要不要同判 | 推的，待主 agent 跑 |
| 5 | 实例表读不出时不拒整批：豁免那半跳过、`planned` 照发，结局报明实例表没换 | ⑦ | 结局成员要多一样 | 推的，待主 agent 跑 |
| 6 | `prepare_the_version_publish_relocating_units` 的提前返回也记账（读盘核那几圈放进一个闭包或内层函数，出来再记） | ⑤ | | 推的 |
| 7 | `mounted_session.rs` 第 225 行文档收窄成「这一组记下的这一次只试一次，再读到再记」，并在 D4 写回时写明；或记试过的组 | 攻击面 3 | 选后者账只增不减 | 推的 |
| 8 | 树表那条断言删掉或换成「旧落点带隔离记号落进盘上分配记录」；加一行证「放回之后 checker 抓得到这一批写错」（记录里提过的「不释放换下的旧链」） | 攻击面 5 | 这一行还可能照样不敏感，那就要写明 checker 对这一批没牙 | 推的，待主 agent 跑 |
| 9 | 卸载之后判真盘（不放回）作为另一条判法，崩溃流加「不放回的状态 × `mount_writable`」 | 攻击面 5、3 | 模型那一条跑出来才知道绿不绿 | 推的，待主 agent 跑 |
| 10 | 删 `TreeTable` 与根节点那两支走不到的匹配，文档跟着改；`transaction.rs` 第 12116 行死链；开关与函数改名 | 检查单两格 | 开关改名牵变异行 | 推的 |
| 11 | 这批写流走 `enumerate_crash_states`、登记重放；GPU 第 663 行那一判加变异行；GPU 行点缩域那一条 | 检查单两格 | 挂钟没量 | 推的 |

各行互不替代：第 1 行不碰第 2 行那几读，第 4 行不碰别的格。判决第四节没有给跑前的改法选项，四句里「跑前条款给的改法」都指这张表。

## 没打中的形状

读码逐步走过、没走出问题的（只抽这一次，按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不支撑结论）：

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 推抬 F 之后 `Rewrite.replaced_chain` 过期撞断言 | 抬 F 那一串的计划 | 照抄实例表，不过期 |
| 同一把 key 进计划两次（挂载记下与别的读记下拼起来） | `relocations_of_the_units_read_with_one_copy_failing_while_mounting` | 按 key 去重 |
| `plan_rewriting_the_paths_to_the_moved_entries` 的断言被路由 key 打中 | 路由 key 这次被删（叶里头一把是这次重写的节点） | 有 `.filter(\|key\| mapping_keys.contains(key))`；那片叶因为删 key 本来就重写 |
| 账经分配器换回丢失 | 全部 `*allocator =` 与 `rebuild_from_records` | `Arc` 共用；只有挂载时新建 |
| 持锁时闭包再取锁 | 三处 `forget` 的闭包 | 都只读局部变量 |
| 数只坏一份的读者漏转方法，读盘核的重读绕不开记下的那一份 | `PoolReader` 全部方法 | 逐个转发 |
| 攒批入口绕过 `relocate_…` | `publish_the_pending_changes`、屏障、`publish_user_change` | 都经 `MountedSession::publish_user_change` |
| 提示过期被算成映射落点上的只坏一份（搬迁之后反复读） | 读文件、读树节点两处 | 两处都套 `PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition` |
| 挂载预演与真发落点分叉撞断言（豁免单元现在真发时可能隔离） | `raise_the_floor_through` 末尾 | 有隔离就不比（`establish_instance` 同形，我只读了抬 F 那一处，挂载那一处只看了 `grep`） |
| 只读路径错读 | 换 key、不换字节 | 校验和挡住 |

## 这条腿自己的限度

- **模型一条没跑**，连编译都没过；断言都是读码推的预期，猜错了就是报告错。
- 崩溃点没造模型：`enumerate_crash_states` 一族我没调，「坏盘 × 部分落盘 × 可写挂载」只读码，没有观测。
- 同形读没穷举：`region_scan`、整理读没逐个核。
- 没读全部实现：`walk.rs` 只读了不变量登记行与记录里的那一句；GPU 内核只看了第 286、663 行。
- D16 只 `grep`，POSIX 相关决策没查全；D21 只读到已定项 9、10 的相关段。
- 一条腿只抽一次样，上表与各处「没打中」都不支撑结论。
- 我取样的历史都是 3 个单元（映射树一层）或 400 个单元（两层），实例表一片或两片，弄坏的都是头一份、翻一个字节，没有扫盘表与设备读错。

## 要主 agent 跑的命令

1. 复跑模型（整份）
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-fixes-code-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r2-opus/copy --work /home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r2-opus/rerun-1`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。`--frozen` 那一份是 `minimal-copy.py --with-test-inputs` 建的（51M，量的），`.deliverables` 写了交接。要拷到别处复跑，模型目录整份拷、`--frozen` 仍指这份。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志 `name=attack_fixes_r2_one_copy_target rc=0 expected=0`、`name=attack_fixes_r2_summary target_as_expected=yes`；`attack-fixes-r2:` 观测行逐条对上面的期望（`rerun-1/observations.txt`）；`rerun-1/one-copy-test.log` 看 `error`、`panicked at`。
   - 期望观测（推的）：`ceiling changes_before_the_damage=0` 结局 `NoSpaceAfterRaisingTheFloor`、`other_after_the_change=1`、`table_moved=false`；`=1`（边界，没把握）、`=2`、`=3` 结局 `Relocated`、别的读 2、`table_moved=true`。`repair_chain_read` 两次 `NotNeeded`、`counted_by_a_counting_reader=1`、`table_moved=false`。`other_reads_after_the_mount` 六个数 `1 1 1 1 0 0`。`key_replaced_after_relocation mapped_is_old=true mapped_is_new=false`。`unreadable_instance_table … unit_kept_still_at_the_damaged_location=true`。`tried_again other=3`。`unrestored` 五行：对照报 I-2.1 或 I-4.8 这类，其余四行我**没有预期**（跑之前写下：绿撤回「放回遮东西」这半句，红逐条判）。`two_pages` 打耗时。
   - 挂钟（推的）：编验证包一次加一个目标，几分钟到十几分钟；不枚举崩溃状态。`two_pages` 那一条 375 次挂载，按单次不到 1 秒算不到 6 分钟，那一行的 `elapsed_seconds` 是实量。超过 40 分钟的量不到这里（推的）；真超了先看这一条，把 `MOUNTS_MAKING_THE_INSTANCE_TABLE_TWO_PAGES` 所在的用例摘掉再跑。
   - 编译错、断言红都交回：新派同立场的腿，「承接：research/prompts/m4-close-fixes-code-r2-opus-output.md」带任务号。

等这条结果才能往下做的：把各处「推的，待主 agent 跑」换成量的；`unrestored` 红的逐条判四句；改法表第 1、3、4、5、8、9 行标量过与否。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数都没复跑（命令在上一节）。
- 没改 `crates/`；模型只在 `--frozen` 那份副本的拷贝里放一个测试目标。
- 没写 kb；D4 第 73 行、C946 的写回不归这一轮。
- 第四节「GPU 全量域」「带 `--item` 门禁」没跑，归主 agent。

没删 /home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r2-opus/copy：交主 agent 作 `--frozen` 跑上面那一条，跑完由主 agent 删（51M，量的）。这个草稿目录里另有 `copy.txt`（建副本的输出）、`.deliverables`。

## 结论什么会推翻

- 上限那一读：模型 `changes_before_the_damage=0` 那一格结局不是 `NoSpaceAfterRaisingTheFloor`，或别的读是 2（说明那条路上别处计了，比如 discard 那一读在这段历史上也发了；那样这一格换历史，不算消失）。
- 认位置那一读：`repair_chain_read` 的 `table_moved=true`，或别的读是 2。
- 映射 key 那一形：挂载被拒，或 `mapped_is_new=true`（提示在搬迁时其实被写新了）。
- 判法那一格：有人拿出一行变异，放回字节之后只有 checker 红；或 `unrestored` 四行全绿，那样「放回遮东西」撤回、只留证红缺口。
- 用户「按字面全接上」的原话说的就是判决点名的那四处：同形读那张表整张作废（我按判决第三节字面读的，没读变更史原话）。
- 走不到的那一支：造出一条产品路径，让账上有这次改动之后现行那一版的树表落点。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m4-close-fixes-code-r2-opus-output.md` 这一次的原样末行：

```
  ✓ 核了 67 处引文、0 处裸行号，对上 67 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```
