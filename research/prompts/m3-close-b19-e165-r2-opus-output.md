# m3-close-b19-e165-r2 攻方腿（Opus）：Q1 形甲第二份样本（2026-11-05）

立场：假设「形甲造不出」是错的，造合法历史、读故障与崩溃去打穿它。攻击面只有正文第三节 Q1。没读禁读清单里的两份（辩方这一轮的产出、第一轮攻方报告「形甲」那一段）。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-b19-e165-r2-opus-model/`，两份文件（sha256 是本腿写完时现算的）：

| 文件 | sha256 |
|---|---|
| `e165_attack_form_a_reader.rs`（714 行，harness 档测试目标源码，复跑时拷进副本的 `crates/singlefs-harness/tests/`） | `e07dbb207a4814bf3387433bdb9ddd9cdbe27e92f10315e092358d1766e069b0` |
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷来，只改了 `rerun_steps` 那一段） | `65e548a7f13ebffb12c877c04fae62544c4a9d6c8b5494dc0ad6b391ae5e608e` |

复跑命令与看哪几行，写在文末「要主 agent 跑的命令」一节。模型一条都没编、没跑过（子 agent 不交任务），下文模型的读数一律是「推的，待主 agent 跑」。

## 各格判定一览

| 格 | 判定 | 依据的种类 |
|---|---|---|
| Q1 主问：两条 txg 相同（非 0）、实例代号不同的根同时自证过、同时进 `choose_root` | **没打中**。不只是「找不到合法历史」：对 `choose_root` 读到的**任意**盘面内容（不论历史、崩溃点、读故障，也不论哪个写者写了什么），入参里每个非 0 txg 至多一条 Some | 读码推的（第一节第 1 小节五步链），模型待主 agent 跑 |
| Q1 逐个写根的地方：grep 列全、每一处有结论 | 列了三条 grep、命中数与逐处结论（第一节第 2 小节）；结论只有两类：非 0 txg 由读者一侧挡，与写者写什么无关；txg 0 的根编码只有 mkfs 写、实例 0 | 读码推的 |
| Q1 附问：txg 0 那三份第 0 代根之外，有没有别的路径让根撞同一 txg | 没有。非 0 txg 在入参里只能出自它落点公式那一槽；txg 0 的编码只出自 mkfs（实例 0），发布路径的 txg 全是「某一版 + 1」 | 读码推的（第一节第 3 小节） |
| 基线、v1823、v1825 各择哪条 | 在入参最大 txg 不是 0 的每一份入参上，三者择同一条（那个 txg 只有一条根）；最大 txg 是 0 时，合法写者只写得出实例 0 的第 0 代根、三份同键，三者也择同一条 | 读码推的（第一节第 4 小节） |
| D22（单元原子性怎么合成） 已定项 7 那一行与代码的一处出入（线索，不在 Q1 判） | 实例表清扫读根环那一遍（`instance_table_sweep.rs`）不核落点公式；D22 写的是「公式核在读根环的两个读者里」 | 读码现查，后果推的（第三节） |

## 第一节 Q1：形甲造不造得出

### 1. 结论与五步链

结论：今天的 `choose_root` 交给择根核的入参里，任意非 0 的 txg T 至多有一条 Some。这一句对 `choose_root` 读到的任意字节都成立，所以任何合法历史、任何崩溃点、D16（发布语义）与 D23（journal 的角色与格式）口径内外的任何读故障都造不出形甲；要造，只能改读者那一侧的代码。

五步链（每步指到代码那一行，行号都是这一次 `grep -n` / `awk 'NR==…'` 现取的；`recovery.rs`、`root_ring.rs`、`make_filesystem.rs`、`transaction.rs` 的 sha256 与开工快照 `research/prompts/m3-close-b19-e165-r2-snapshot/crates-sha256.txt` 里那一行相同，本腿现比过；`mount.rs`、`instance_table_sweep.rs` 不在那份清单里，引它们的行是此刻主工作区的）：

1. 入参的每一格对应一个不同的 (区域, 槽)：入参按 `every_root_ring_slot` 逐槽建，`crates/singlefs-core/src/recovery.rs:1894`：「every_root_ring_slot(&immutable.sizes)」；它交回区域 0..3 × 槽 0..S，每对一次，`crates/singlefs-core/src/recovery.rs:1820`：「(0..immutable_sizes.root_ring_slots_per_region.count())」、`crates/singlefs-core/src/recovery.rs:1821`：「.map(move |slot| RootRingSlot { region, slot })」。
2. 一格是 Some，当且仅当那一格的 `read_root_ring_slot` 交回 SelfVerified：`crates/singlefs-core/src/recovery.rs:1904`：「RootRingSlotReading::SelfVerified(root) => Some(root),」，读不出与自证不过都是 None（同文件 1905–1908 行）。
3. `read_root_ring_slot` 交回 SelfVerified 之前核落点公式，不合的当自证不过：`crates/singlefs-core/src/recovery.rs:1867`：「if !root_sits_in_the_ring_slot_the_formula_gives(」，那一支交回 `crates/singlefs-core/src/recovery.rs:1873`：「RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)」。核用的 S 与第 1 步枚举槽用的 S 是同一个字段（`immutable.sizes.root_ring_slots_per_region`，同一份系统配置）。
4. txg 不是 0 的根，落点公式只认一个槽：`crates/singlefs-core/src/root_ring.rs:192`：「target_for_publish(checkpoint_txg, slots_per_region) == ring_slot」；txg 是 0 的另认 mkfs 种根的三槽，`crates/singlefs-core/src/root_ring.rs:188`：「if checkpoint_txg == crate::make_filesystem::GENESIS_CHECKPOINT_TXG {」。`target_for_publish` 是 txg 的函数，`crates/singlefs-core/src/root_ring.rs:172`：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」、`crates/singlefs-core/src/root_ring.rs:173`：「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」。
5. 于是 txg 为 T（T ≠ 0）的 Some 只能出在 (区域, 槽) 等于 `target_for_publish(T, S)` 的那一格；由第 1 步，这样的格恰好一个。两条 txg 同为 T 的 Some 要占两格，不存在。

这条链里没有一步看「槽里的字节是谁写的、什么时候写的、读的时候出没出错」：读故障能做的（读不出、读回乱码、读回别处的字节、读回旧内容）都只改「某一格读到什么字节」，第 3、4 步对每一格读到的字节逐格判。两个物理位置撞在一起（区域重叠、错位读读到另一块盘或另一槽）也不改结论：判的是「这一格要的 (区域, 槽)」与「读到的根的 txg」，不看字节从哪里来。

推翻条件（什么现象说明这句错了）：模型产品臂在任一几何、任一编码族上报 `form_a_at_a_nonzero_txg` 大于 0；或者 `choose_root` 之外另有一处把根摊成择根核入参、而不经 `read_root_ring_slot`（本腿 grep 了 `choose_root_slot` 的调用方：`crates/singlefs-core/src` 里只有 `recovery.rs` 第 1921 行那一处，命令与命中在第 2 小节）。

### 2. 写根槽的地方：grep 列全与逐处结论

因为挡在读者一侧，逐个写者要回答的只剩一件事：它写不写得出「txg 0、实例代号非 0、本池 fsid、自证得过」的根编码（这是第 4 步里 txg 0 另认三槽留下的唯一口子）。根编码只有一个编码器：`crates/singlefs-core/src` 里写 `ROOT_MAGIC` 的只有 `crates/singlefs-core/src/root_record.rs:368`：「writer.put(&ROOT_MAGIC);」，它在 `RootRecord::to_slot` 里（同文件第 356 行）。

用到的三条 grep（在仓根跑，这一次的命中数）：

| 命令 | 命中 | 读法 |
|---|---|---|
| `grep -rn 'write_at(' crates/*/src --include=*.rs \| grep -v 'fn write_at' \| wc -l` | 101 | 按「在不在所在文件第一个 `#[cfg(test)]` 之后」分：产品代码 29 行（`singlefs-core` 10、`singlefs-harness` 5、checker 档装置二进制 14），测试模块里 72 行 |
| `grep -rn '\.to_slot(' crates --include=*.rs \| wc -l` | 96 | 根编码与系统配置编码合计；`crates/singlefs-core/src` 里去掉 `system_configuration` 的 15 行，产品代码是 `make_filesystem.rs:767` 与 `transaction.rs` 1784、2368、16756 三行，`recovery.rs:4912` 只拿来比字节不写；其余在 `#[cfg(test)]` 之后 |
| `grep -rnw 'slot_offset' crates --include=*.rs \| wc -l` | 289 | 算根环槽偏移的每一处（读与写都在里面），逐文件计数存在草稿目录 `slot_offset_all.txt` |
| `grep -rn 'choose_root_slot\|choose_root(' crates/ --include=*.rs` | 82 行、45 个文件（加 `-l` 数） | 产品代码里摊择根核入参的只有 `recovery.rs:1921` 一处（`grep -rn 'choose_root_slot(' crates/singlefs-core/src` 去掉注释行只命中它）；其余是测试、装置与 harness 的调用方，全经 `choose_root` |

逐处结论（产品代码那 29 行 `write_at` 与 4 行根编码；「挡法」一列：「读者挡」指第 1 小节五步链，与这一处写什么无关）：

| 位置（此刻主工作区） | 写什么 | txg 0、实例非 0 的根写得出吗 | 非 0 txg 形甲的挡法 |
|---|---|---|---|
| `crates/singlefs-core/src/transaction.rs:581`：「device.write_at(」 | 发布的根槽 FUA：落点由同一个 `CommitStep` 里的 txg 算（`crates/singlefs-core/src/transaction.rs:570`：「let target = target_for_publish(」）。带单元、零单元、只写实例表三条发布路径与冻结重发都经 `persist_the_root_then_rotate_the_system_configuration` 到这里（`crates/singlefs-core/src/transaction.rs:1659`：「writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {」）；回退、清扫、暖机、抬 F、取号那一次发布都是这几条路径之一 | 写不出：根编码的 txg 都是某一版加一，见第 3 小节 | 读者挡；另外写者自己也只写落点槽，同 txg 的后一次写盖在前一次上 |
| `crates/singlefs-core/src/make_filesystem.rs:776`：「device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;」 | mkfs 的第 0 代根，三个区域各自槽 0 各一份、同一份字节（`crates/singlefs-core/src/make_filesystem.rs:767`：「let root_slot = root.to_slot(root_slot_bytes);」） | 写的就是 txg 0，实例代号是 `crates/singlefs-core/src/make_filesystem.rs:52`：「pub const MKFS_INSTANCE_GENERATION: InstanceGeneration = InstanceGeneration(0);」 | 三份同键同字节，不是形甲 |
| `crates/singlefs-core/src/make_filesystem.rs:705`：「device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;」 | mkfs 在每块盘上把三个区域清零（`write_zeroes_at`，不是 `write_at`，另列） | 全零过不了自证 | 不产生根 |
| `transaction.rs` 553、561、700 | 单元（`slot.to_device_offset()`）、journal 记录（`record_offset`）、系统配置槽 | 不是根编码（不经 `to_slot` 的根那一支；journal 记录与系统配置各有自己的编码） | 落进根槽也只是一格读到的字节，读者挡 |
| `make_filesystem.rs` 710、715、804 | 实例表、树表第 0 代单元、系统配置槽 | 不是根编码 | 同上 |
| `crates/singlefs-core/src/instance_table_sweep.rs:604`：「if let Err(error) = device.write_at(」 | 清扫抹单元头，写全零 | 全零过不了自证 | 同上 |
| `crates/singlefs-core/src/mounted_session.rs:1355`：「match self.disk.write_at(offset, bytes, durability) {」与 harness 四份包装（`crates/singlefs-harness/src/fault_injection.rs:1182`、`crates/singlefs-harness/src/lib.rs:490`、`crates/singlefs-harness/src/read_tally.rs:257`、`crates/singlefs-harness/src/history.rs` 7550 与 7779，都是 `self.inner.write_at(offset, bytes, durability)` 一行转发） | 转发调用方给的字节 | 写的就是调用方的字节，调用方是上面几行 | 同上 |
| checker 档 `src/bin/` 里 14 行（`e158_root_choice_repair.rs` 4、`e181_…` 4、`e185_…` 4、`e182_…` 1、`e183_…` 1） | 实验装置往手摆的镜像里写 | 装置想写什么都行，不是合法历史 | 读者挡（形甲在这些镜像上同样进不了 `choose_root` 的非 0 txg） |
| 测试模块里 72 行、`crates/*/tests/` 里的手摆镜像（如 `root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs` 把实例 1、2 的 txg 0 根写进区域 1、2 的槽 0） | 夹具 | 测试能写出 txg 0、实例非 0 的根——这正是读者另认三槽留下的口子，合法写者写不出 | 非 0 txg 照样读者挡 |
| 根编码 `transaction.rs` 1784、2368、16756 三行（`prepare_the_publish_without_units`、`prepare_the_row_publish_on_a_version_without_file`、`publish_admitted` 里装 `PublishWrites.root_slot`） | 发布要写的根字节，交第一行落盘 | 见第 3 小节：txg 来自计划或「上一版 + 1」 | 读者挡 |

### 3. txg 0 以外有没有路径让根撞同一 txg；txg 0 那一格

非 0 txg：第 1 小节第 5 步已经把「同一 txg 的两条根同时进入参」排除，与哪条路径写无关。写者一侧另有一层（推的，不是挡法本身）：同一 txg 只落一个槽，后写的盖掉先写的，盘上同一时刻只留一份。新实例复用一个旧 txg 要求取号那一刻旧那条根与它那次发布的记录全读不出（第一轮挂起的形乙的边界情形：新实例第一次发布的 txg 恰好等于读不出的那条），那一次写也落在旧根那一槽上，崩在根槽持久之前留下旧的，持久之后留下新的，择根入参里都只有一条。

txg 0：读者让 txg 0 住三槽，所以形甲只可能出在 txg 0 上，前提是盘上有一条 txg 0、实例非 0、自证得过的根。产品代码里写根编码的四处（上一小节）的 txg 来源，本腿逐处现查：

- mkfs：txg 0、实例 0（`crates/singlefs-core/src/make_filesystem.rs:52` 那一行）。
- 发布的 txg 一律是某一版加一：`crates/singlefs-core/src/transaction.rs:5844`：「.checked_add(1)」（`checkpoint_txg_of_the_publish_after`，`transaction.rs` 里有 18 处调它）；挂载一侧 `crates/singlefs-core/src/mount.rs:1427`：「version.0.checked_add(1).map(CheckpointTxg).ok_or(」；新实例第一次发布取 `crates/singlefs-core/src/mount.rs:1419`：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（环里一条根、一条记录都读不出时两项都是 `unwrap_or(CheckpointTxg(0))`，加一得 1）；清扫发布 `instance_table_sweep.rs` 第 738–744 行取上一版加一；暖机那条老入口 `crates/singlefs-core/src/transaction.rs:1385`：「for txg_number in 1..=WARM_UP_EMPTY_PUBLISHES {」写常数 1、2。加一越过 `u64::MAX` 都报错，不绕回 0。
- 所以写者写出的根编码里 txg 0 的只有 mkfs 那一份、实例 0；txg 0 那三槽读到的若都是第 0 代根，键全是 (0, 0)。这与 D16（发布语义） 已定项 6 第一条骑手条款里那句一致：`.claude/kb/decisions/16-发布语义.md:195`：「一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根、取读根环次序最前的」。

剩下的一种（推的，在模型的「任意编码」族里量）：一条不是写者写的、自证得过的 txg 0 根出现在盘上，只有伪造（CRC32C 与 fsid 都碰对）或测试夹具才做得到，在 D16、D23 的读故障口径之外。

### 4. 基线、v1823、v1825 各择哪条

三条择根序只在「入参最大的那个 txg 上有两条实例代号不同的根」时分开（v1823 取那个 txg 在读根环次序里最前的一条，v1825 取实例最低的，基线取实例最高的；本地写法见模型 `ChoiceOrder::replaces`，与 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 第 14840 行起 `FaultLocalRule::replaces` 的比较式同形，本腿读过、没拷）。由第 1 小节，最大 txg 不是 0 时那个 txg 只有一条，三者择它；最大 txg 是 0 时入参里只剩 txg 0 那三槽，合法写者写出的都是同字节的第 0 代根，三者都择读根环次序最前的那一份（键相同时三者都不换）。所以在今天的读者上，v1823、v1825 与基线在每一份合法写者写得出的盘面上择同一条，不只是在合法历史上：这比第一轮判决的「合法历史无对象」宽，宽出来的那一截是「再加任意读故障与崩溃」。v1826 不在这一题里（它分开的是形乙）。

打中之后要答的四句，这一格没打中，不答。

## 第二节 模型：做什么、跑前写死的读法

不建写者历史，直接摆盘面：这是第一轮攻方模型（真写者历史 + 崩溃枚举）之外另一条路，两者不共用装置与取样（本腿没读、没拷第一轮的模型源码，只读了它 `tools/rerun.sh` 的跑法）。

三条用例（`e165_attack_form_a_reader.rs`）：

| 用例 | 判什么 | 态数（摆了几份盘面） | 线程 |
|---|---|---|---|
| `e165_form_a_admission_per_slot` | S ∈ {4, 8, 16}，两块内存盘照第一版区域归属 mkfs 一次；每一槽 × txg 0..6S+3 × 实例 0..4 的每条根编码写进那一槽、调产品 `read_root_ring_slot` 读一次，数每个 txg 有几槽读得出。对照臂同一份字节改用只验自证与 fsid、不核落点公式的读法（`RootRecord::parse_slot`）。另数：根写在落点槽的偏移、却写在另一块盘上时产品读者读不读得到 | 每个 S 每臂 3S × (6S+3) × 4 份：S=4 是 1296、S=8 是 4896、S=16 是 19008，两臂合计 50400；另一块盘那一格每个 S 是 6S+2 份 | 1 |
| `e165_form_a_whole_ring_choices` | 每个 S × 两族编码（写者写得出的：txg 0 只配实例 0、txg ≥ 1 配实例 1..3；任意编码：另收 txg 0、实例非 0）各摆 15000 环；每环每槽全零、乱码或一条根，根的 txg 一半照那一槽的落点公式取、一半取自这一环撞在一起的 {0, 低, 低 + 3S}（摆在哪槽不看公式），四分之一的环只在第 0 代根那三槽摆根。两臂各建一份入参，数形甲（非 0 txg 上的、最大 txg 上的）、本地 today / v1823 / v1825 / v1826 各择哪条，产品臂另比产品 `choose_root` 与本地 today | 3 × 2 × 15000 = 90000 环，两臂读同一份 | 登记的每台默认重活线程数（`default_heavy_threads`，此刻登记 31），`std::thread::scope` 按环号区间切片 |
| `e165_form_a_sliced_matches_single_thread` | S = 8、任意编码，3000 环单线程与切片多线程各判一遍，两臂的计数逐项相同 | 6000 环 | 1 与登记值 |

全部用例合计摆 146574 份盘面（50400 + 174 + 90000 + 6000，压在 20 万以内；推的，待主 agent 跑出的 `OPUSMODEL` 行逐格对）。单项挂钟推的在一分钟以内，过 300 秒每片打一行 `name=progress`。没有写者历史，就没有崩溃状态要枚举，`crash_state_count_with_torn_in_place_overwrites` 这一步不适用；摆法的取样域是第四节「没打中的形状」那张表。

跑前写死的读法（这几行写在跑之前，结果反过来照改结论）：

| 读数 | 预期 | 反过来怎么改 |
|---|---|---|
| 产品臂 `name=admission` 的 `nonzero_txgs_admitted_in_two_or_more_slots`、`nonzero_txgs_not_exactly_at_their_formula_slot` | 都是 0；`slots_admitting_txg_zero` 等于 `genesis_slots` | 任一不是 0 ⇒ 第 1 小节第 4 步读错了，Q1 改判「打中」，按那一格的 txg 与槽造最小盘面 |
| 产品臂 `name=ring_choices` 两族的 `form_a_at_a_nonzero_txg`、`form_a_at_the_newest_nonzero_txg` | 都是 0 | 同上，`smallest_ring_with_form_a_at_a_nonzero_txg` 给环号，凭 (S, 族, 环号) 复现那一环 |
| 产品臂写者族的 `v1823_differs_from_today`、`v1825_differs_from_today`、`form_a_at_the_newest_txg_zero` | 都是 0 | 不是 0 ⇒ 第 3 小节「txg 0 只有 mkfs 写」的取样有漏，看 `smallest_ring_where_v1823_or_v1825_differs` |
| 对照臂两条用例的非 0 txg 形甲计数（assert 大于 0） | 大于 0 | 报 0 ⇒ 计数器没判别力，模型作废（用例当场红） |
| 产品臂任意编码族的 `form_a_at_the_newest_txg_zero`（assert 大于 0）与 v1823、v1825 不同于 today 的环数 | 大于 0 | 报 0 ⇒ 产品臂上的计数器看不见形甲，模型作废（阳性对照，在产品读者上证会红） |
| 产品臂 `product_choose_root_differs_from_today`（assert 等于 0） | 0 | 不是 0 ⇒ 本地 today 抄错，v1823 至 v1826 的比较作废 |
| `name=other_device` 的 `admitted_by_the_product_reader` | 0 | 不是 0 ⇒ 读者没按区域归属挑盘，第 1 小节要补一步 |
| `name=cross_check` 的 `single_thread_equals_sliced` | 两臂都 true | false ⇒ 切片合并有错，多线程读数不用 |

## 第三节 线索（不在 Q1 判，交主 agent）

读根环的读者里有一处不核落点公式：实例表清扫判删哪几行时读根环那一遍，`crates/singlefs-core/src/instance_table_sweep.rs:461`：「fn every_root_of_the_ring_self_verified(」，逐槽读、自证不过重读一次，收根只看 `crates/singlefs-core/src/instance_table_sweep.rs:485`：「root_read = RootRecord::parse_slot(&bytes, filesystem_identifier);」，没有 `root_sits_in_the_ring_slot_the_formula_gives`（本腿 `grep -rn 'root_sits_in_the_ring_slot_the_formula_gives' crates/*/src` 去掉注释行、`use` 行与 `root_ring.rs` 自己的定义和单测，只剩 `recovery.rs` 第 1867 行与 `mount.rs` 第 2808 行两处调用）。它接着按读到的根的实例代号定删哪几行：`crates/singlefs-core/src/instance_table_sweep.rs:361`：「.filter(|row| !instances_with_a_root_in_the_ring.contains(&row.instance))」。

D22（单元原子性怎么合成） 已定项 7 字段表「实例代号」那一行写的是 `.claude/kb/decisions/22-单元原子性怎么合成.md:141`：「公式核在读根环的两个读者里」，点名的两个是择根读槽与重建分配器那一遍；清扫这一遍不在里面。

推的后果（没造、没量）：一次错位读把槽 X 读成槽 Y 里那条自证得过的根时，清扫看不见 X 里那条根的实例代号；那条根若是它的实例在环里的最后一条，这一行会被当成「环里没有它的根」删掉，而那条根还在 X 里、之后可能被择到。错位读算不算 D16、D23 口径内的读故障，以及这一形是不是已有欠账，本腿没查，交主 agent 判要不要另立一题。它与形甲无关：清扫不择根。

## 第四节 没打中的形状

| 形状 | 怎么试的 | 取样范围 | 结局 |
|---|---|---|---|
| 两条非 0 同 txg、不同实例的根摆在两个不同槽（错位写、错位读、物理重叠） | 读码五步链；模型逐槽 × 编码、整环随机摆 | 读码对任意内容；模型 S ∈ {4, 8, 16}，txg 0..6S+3，实例 0..4，整环 90000 环 | 读者第 3 步挡（推的；模型待跑） |
| 同一槽先后两条同 txg、不同实例（新实例复用一个读不出的旧 txg） | 读写者：根写在 `target_for_publish(txg)` 那一槽 | 读码 | 后写盖前写，入参里一条（推的） |
| 两块盘各存一份同一槽 | 读 `read_root_ring_slot` 选盘：只读区域归属那块盘；mkfs 只认第一版归属 `crates/singlefs-core/src/make_filesystem.rs:563`：「if devices.len() == 2 && parameters.region_devices != FIRST_VERSION_REGION_DEVICES {」 | 读码；模型 `name=other_device` 每个 S 6S+2 份 | 另一块盘上那份读不到（推的；模型待跑） |
| txg 0 上实例不同（三槽另认） | 查全部根编码器与发布 txg 的来源 | 产品代码里写根编码的四处、全部发布计划的 txg 字段（25 处构造，去掉测试模块逐处看过） | 合法写者写不出；只有测试夹具与「任意编码」族写得出 |
| 两份不同的系统配置让 S 或区域归属不同 | 读 `choose_root`：枚举与核公式用同一份 `immutable` | 读码 | 一次择根只用一份，无从并列（推的） |
| `choose_root` 之外摊择根核入参的路径 | grep `choose_root_slot` | 全 `crates/` | 产品代码只有 `recovery.rs:1921` |
| 挂载两遍逐槽并、重建分配器读根环 | 读 `mount.rs` 那一遍：同样核公式（第 2808 行） | 读码 | 每槽仍只一条；它也不进 `choose_root` |

## 第五节 这条腿自己的限度

- 结论全部是读码推的；模型一行没编、没跑。模型是我写的、被攻过零轮，改法这一腿没提。
- 模型与五步链共用同一份被测代码（产品 `read_root_ring_slot`、`choose_root`），只是一条是读、一条是跑；它能抓到「我读错了代码」，抓不到「代码与条款都错在同一处」。
- 模型的根编码只改了 txg 与实例代号两项（别的字段照第 0 代根）。第 3、4 步只看 txg 与槽，推的不影响；`instance_dependent_admissions` 那一列在跑出来时可以核这一点。
- 根槽宽只取了 512 字节、槽距 4096；4096 字节根槽没取。
- 「txg 0 只有 mkfs 写」是逐个查产品代码里的写根编码处与发布计划得来的，grep 罩不住用别的写法拼出根编码的代码（本腿只查了 `ROOT_MAGIC` 的写入处只有一处）。
- 没读第一轮攻方模型与报告的形甲论证，与它们重合多少不知道。

## 第六节 要主 agent 跑的命令

一条一项。判决要引的数照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」由主 agent 把模型目录拷进自己的草稿目录再交；下面写的是在本腿草稿目录里跑的形态，`--work` 换成主 agent 自己的新目录即可。

1. 模型整份复跑（编一次、三条用例一趟）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r2-opus/rerun-model`
   - 范围声明：脚本里那条 `cargo test --release --offline -p singlefs-harness --test e165_attack_form_a_reader -- --nocapture --test-threads 1` 自己带 `SINGLEFS_RUN_SCOPE='种类=target; …; 范围=1 个测试目标; …'`（`tools/rerun.sh` 的 `rerun_steps`）；submit 前面要不要再写一条照 `.claude/hooks/run-scope-guard.sh` 判，被拒照它的出路改。
   - `--frozen` 给仓根：本轮没有冻结副本目录，开工快照只有 `research/prompts/m3-close-b19-e165-r2-snapshot/` 里的 sha256 清单；脚本打的 `name=frozen_source` 行可与 `crates-sha256.txt` 逐行比。
   - 在哪起：`--cwd` 那个目录（本腿草稿目录）；跑完 `$WORK/tree` 是一份仓副本，引完数照共用约束删掉。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志里 `name=rerun_roots … model_exit=0`、`test result: ok. 3 passed`；再照第二节「跑前写死的读法」那张表逐行读 `OPUSMODEL name=admission`（每个 S 两臂各一行，共 6 行）、`name=other_device`（3 行）、`name=ring_choices`（3 个 S × 2 族 × 2 臂 = 12 行）、`name=cross_check`（2 行）。`model_exit` 非 0 时日志末尾打 `error` 与 `panicked` 行：编不过是本腿的源码错（本腿没编过），交回新派同立场的腿修；assert 红在对照或阳性对照上是模型作废，不是打中。
2. `rerun.sh` 的自证（假驱动，只拷副本、不编不跑）：
   - 命令：`task-server/task submit -- bash research/prompts/m3-close-b19-e165-r2-opus-model/tools/rerun.sh --selftest`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`✓ 自检通过（查了 6 格）` 那一行，`exit=0`。

交回前本腿跑了 `python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-e165-r2-opus-output.md`，末行贴在第七节。

## 第七节 没做什么

- 没判正推、辩方那几格（Q2 一格没碰），没替主 agent 采纳。
- 没往统一任务服务交任何任务；模型没编、没跑，判决要引的数都没复跑（命令在第六节）。文中模型的读数与时长都是「推的，待主 agent 跑」。
- 没建写者历史、没枚举崩溃状态：Q1 的结论落在读者一侧、对任意内容成立，本腿判断写者历史上的崩溃枚举不增加判别力；这是推的，主 agent 不认可的话，第一轮攻方模型那条真写者历史的路子是现成的另一条。
- 第三节那条线索没造盘面、没量、没查欠账表。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r2-opus` 里只有本腿 grep 的输出（`slot_offset_all.txt`、`to_slot_all.txt`、`write_at_src.txt`、`write_at_classified.txt`、`plans.txt`、`recovery_fns.txt`）与改 `rerun.sh` 用的两份片段（`old_steps.txt`、`new_steps.txt`）；没建仓副本、没建编译目录，没有要删的。
- `python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-e165-r2-opus-output.md` 的末行（写这一行之前那一遍，退 0）：「✓ 核了 30 处引文、8 处裸行号，对上 30 处，没判 8 处（其中裸行号没有引文可比 8 处；逐处列在上面）」。
