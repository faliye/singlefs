# m3-close-b19-e165-r3 攻方腿（Opus）：攻 C650 的新判据（2026-11-05）

立场：假设第二轮判决给的新判据是错的，造盘面、历史与改坏去打穿它。攻击面只有正文第三节 Q1 三问（等价那一句的两个方向、只罩读者侧的留档用例判别力、第二轮读数能不能核）；不重复前两轮攻过的「形甲造不造得出」「多处读坏能不能造出形乙」。没读禁读清单里的辩方这一轮产出。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-b19-e165-r3-opus-model/`，两份文件（sha256 本腿写完时现算）：

| 文件 | 行数 | sha256 |
|---|---|---|
| `e165_r3_choice_equivalence.rs`（harness 档测试目标源码，复跑时拷进副本的 `crates/singlefs-harness/tests/`） | 862 | `f5ffe840afc1c6eacd8e35f052b5e014324c848cdca1d3bc53251b27e4f0c22e` |
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps` 那一段，另加两个只被它调的函数 `apply_the_change`、`run_the_targets`） | 247 | `7beef123336c1be1095191c71087cc52f90f9c409b73d24415f0ac394330335a` |

复跑命令、在哪起、看哪几行写在文末「要主 agent 跑的命令」。模型一条都没编、没跑（子 agent 不交任务）：下文模型读数一律是「推的，待主 agent 跑」；标「量的」的只有两类：主 agent 第二轮复跑 7660 已经打出的行（主 agent 存的 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r2-main/model-7660-opusmodel-lines.txt`，23 行，sha256 `4be8e381…`），与本腿用 `grep`、`awk`、`sha256sum`、`diff` 现数的文本事实。

## 各格判定一览

| 格 | 判定 | 依据的种类 |
|---|---|---|
| Q1.1 v1823 | 等价句「后一句 ⇒ 造不出让它红的盘面」**不成立**：差 txg 0 那一截（同一 txg 0 上有两种实例代号的盘面满足后一句、v1823 照样择得不同）；补上「txg 0 上只有一种实例代号」后成立。承诺内合法写者造不出那一形，挡它的是写者侧「txg 0 只有 mkfs 写、实例 0」，不是读者侧 | 量的（7660 任意编码产品臂 `v1823_differs_from_today` 1441、1400、1441 而 `form_a_at_a_nonzero_txg=0`）；读码 |
| Q1.1 v1825 | 同 v1823：不成立，差 txg 0 那一截；补上之后成立；承诺内造不出，挡在写者侧 | 量的（同上三行 `v1825_differs_from_today` 2703、2691、2686）；读码 |
| Q1.1 v1826 | **打中**：后一句与 v1826 无关。满足「同一非 0 txg 至多一条」而 v1826 与基线择得不同的盘面成片存在，仓里择根胶水那条用例摆的就是一个；7660 在写者写得出的编码、产品读者上量到 6497、7990、8401 环（每档 15000 环）。承诺内合法写者造不出它（落在 C971 那一形，第二轮判在承诺外），但挡它的是「进择根的根不倒挂」，新判据没写这一句、留档用例也看不见它 | 量的（7660 三行，第一节第 2 小节整行抄）；读码 |
| Q1.1 反方向「造不出 ⇒ 后一句」 | 盘面级不成立：形甲落在不是最大的那个 txg 上时三条变体都不红（7660 对照臂 `form_a_at_a_nonzero_txg` 6897 而 `v1825_differs_from_today` 1577，恰等于 `form_a_at_the_newest_nonzero_txg`）。当成「合法域上的两句话」，两句都真，等价平凡成立，但不承重 | 量的（7660 对照臂三行） |
| Q1.1 新判据改成什么 | 三条变体分开写各自的盘面级充要条件与挡法：v1823、v1825 由「读者侧落点核」加「写者侧 txg 0 只有 mkfs 写」两样罩；v1826 由「不倒挂」罩（写者与挂载门一侧），读者侧那一句对它不起作用。全文在第一节第 5 小节，只在我的模型上量过、被攻过零轮 | 推的（模型逐盘核待跑） |
| Q1.2 留档用例的判别力 | **不够**，三处：一，判据字面「至多一条」比原型自己打出的「恰好在落点槽」弱；二，原型的 S 只取 4、8、16 三个 2 的幂，`% S` 改成 `& (S − 1)` 这一类改坏在这三档上逐字节同；三，原型对产品臂一条 assert 都没有（只 assert 对照臂），读数靠人读 | 读码（原型第 49、398 行）；改坏副本上的读数待跑 |
| Q1.2 同时躲过 Verus 必红世界与留档用例的改坏 | **有，四种**（都在 `singlefs-core`，必红世界 296 行一行都不改这个 crate）：读者 txg 0 分支放宽、读者落点式改成按位与、择根胶水把实例代号摆到 txg 前面、挂载一侧下一个 txg 加 0。前两种只在本腿逐档穷举那条用例上红；胶水那一种由仓里胶水用例杀；挂载那一种在承诺内（无故障、无崩溃，mkfs 后可写挂载一次）造出 v1825 红的盘面 | 量的（必红世界表逐行数文件列）；各改坏的红绿推的，待主 agent 跑 |
| Q1.2 写者侧「txg 0 只有 mkfs 写」要不要另一条用例 | **要**：读者侧用例按构造看不见写者，Verus 不验 `singlefs-core`；本腿给了一条（`e165_r3_writable_mounts_on_a_fresh_pool`），v1826 那一半（不倒挂）另要一条，本腿没写 | 读码；用例待跑 |
| Q1.3 第二轮读数能不能核 | **核得动**：7660 打出的 23 行 OPUSMODEL 与主 agent 存的那份逐行同集；7660 日志 `name=frozen_source` 九行的 sha256 与今天主工作区逐个相同、摆法按 (S, 族, 环号) 定种子，今天原样复跑应逐字段同（`seconds=` 除外）。判决第一节第 15 行用「…」截掉了同一行里的 `v1826_differs_from_today` 那一列，那一列正是 v1826 那一格的反例 | 量的（`diff`、`sha256sum`）；复跑命令在文末 |

## 第一节 Q1.1：等价那一句的两个方向

被判的句子（第二轮判决「去向」表第一行）：`research/prompts/m3-close-b19-e165-r2-main-verification.md:43`：「等价于『同一非 0 txg 至多一条根进择根』」，前半句是「承诺内的合法写者造不出让它们红的盘面」。下面把「后一句」记作 RHS：进 `choose_root` 的根里，每个非 0 txg 至多一条。

### 1. 三条变体各自「与基线择得不同」的盘面级充要条件（读码推的，模型逐盘核待跑）

基线是择根核 `crates/singlefs-replay-prefix/src/lib.rs:4014`：「Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0」 那一段（txg 为主、平局实例代号高者赢、键相同不换）。三条变体照必红世界表的替换文：`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:155`：「择根·改判定·比较不看实例代号（只比 checkpoint_txg；原型 m2_ignore_instance）」、`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:157`：「C650·改判定·择根 txg 平局时实例代号低者赢（改坏择根序：平局取低）」、`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:158`：「C650·改判定·择根以实例代号为主、txg 破平局（改坏择根序：实例代号为主）」。对进择根的任意一组根（读根环次序）：

| 变体 | 与基线择得不同 ⟺ | 与 RHS 的关系 |
|---|---|---|
| v1823 | 最大的那个 txg 上，读根环次序最前的那条不是那个 txg 上实例代号最高的 | 要「最大 txg 上有两种实例代号」：RHS 只排掉最大 txg 非 0 的情形，最大 txg 是 0 时照样会 |
| v1825 | 最大的那个 txg 上有两种实例代号 | 同上 |
| v1826 | 有一条根的实例代号高于基线所选那条（那条根的 txg 必定更低） | 无关：两条根 txg 不同，RHS 本来就满足 |

推法：v1823 取最大 txg 上读根环次序最前那条，基线取那个 txg 上实例最高、并列时最前那条，两者不同当且仅当最前那条实例不是最高。v1825 取最大 txg 上实例最低的，与基线不同当且仅当最低不等于最高。v1826 先取实例最高、再取其中 txg 最大的；若没有根的实例高于基线所选那条，最高实例就是基线所选那条的实例，而基线所选那条 txg 全局最大，v1826 落到同一个键上；反之 v1826 取到的实例更高，必不同。

所以：

- **「RHS ⇒ 不红」这个方向，三条都不成立。** v1823、v1825 缺「txg 0 上只有一种实例代号」那一截（RHS 字面只管非 0 txg）；v1826 整个与 RHS 无关，缺的是「不倒挂」：进择根的任两条根，txg 小的那条实例代号不高于 txg 大的那条（够用；恰好的条件是「没有根的实例高于基线所选那条」）。
- **「不红 ⇒ RHS」这个方向，盘面级也不成立：** 两条同 txg、不同实例的根只要不在最大的 txg 上，三条变体都与基线同。
- 只当成「承诺内合法域上的两句话」比：两句在第二轮的证据下都真，等价平凡成立，但两句之间没有推出关系，留档 RHS 罩不住前半句。

### 2. 盘面：量过的与仓里现成的

7660（主 agent 第二轮复跑）已经量到上面三条，全部来自产品读者（核落点公式那一臂），每档 15000 环。整行抄自 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r2-main/model-7660-opusmodel-lines.txt`：

第 23 行（S = 4）、第 13 行（S = 8）、第 17 行（S = 16），写者写得出的编码（txg 0 只配实例 0、txg ≥ 1 配实例 1..=3）、产品臂：

```
test e165_form_a_whole_ring_choices ... OPUSMODEL name=ring_choices slots_per_region=4 family=writer_reachable arm=product rings=15000 threads=31 form_a_at_a_nonzero_txg=0 form_a_at_the_newest_nonzero_txg=0 form_a_at_the_newest_txg_zero=0 v1823_differs_from_today=0 v1825_differs_from_today=0 v1826_differs_from_today=6497 product_choose_root_differs_from_today=0 smallest_ring_with_form_a_at_a_nonzero_txg=none smallest_ring_where_v1823_or_v1825_differs=none seconds=0.01
OPUSMODEL name=ring_choices slots_per_region=8 family=writer_reachable arm=product rings=15000 threads=31 form_a_at_a_nonzero_txg=0 form_a_at_the_newest_nonzero_txg=0 form_a_at_the_newest_txg_zero=0 v1823_differs_from_today=0 v1825_differs_from_today=0 v1826_differs_from_today=7990 product_choose_root_differs_from_today=0 smallest_ring_with_form_a_at_a_nonzero_txg=none smallest_ring_where_v1823_or_v1825_differs=none seconds=0.02
OPUSMODEL name=ring_choices slots_per_region=16 family=writer_reachable arm=product rings=15000 threads=31 form_a_at_a_nonzero_txg=0 form_a_at_the_newest_nonzero_txg=0 form_a_at_the_newest_txg_zero=0 v1823_differs_from_today=0 v1825_differs_from_today=0 v1826_differs_from_today=8401 product_choose_root_differs_from_today=0 smallest_ring_with_form_a_at_a_nonzero_txg=none smallest_ring_where_v1823_or_v1825_differs=none seconds=0.04
```

读法：`form_a_at_a_nonzero_txg=0` 就是 RHS 在这 15000 环上全成立；同一行 `v1826_differs_from_today` 是 6497、7990、8401——RHS 成立的盘面上 v1826 有四到五成择得不同。这一族的实例代号随机配、不看 txg，倒挂成片，所以这三行只说「RHS 罩不住 v1826」，不说「合法历史上会」。

第 11、15、19 行（任意编码一族，另收 txg 0、实例非 0 的根）、产品臂：`form_a_at_a_nonzero_txg=0`，`v1823_differs_from_today` 1441、1400、1441，`v1825_differs_from_today` 2703、2691、2686，等于各行的 `form_a_at_the_newest_txg_zero`（2703、2691、2686）——v1825 与 v1823 只在 txg 0 那一截红，与第 1 小节的充要条件同形。

第 10、14、18 行（写者写得出的编码、对照臂：不核落点公式）：`form_a_at_a_nonzero_txg` 6897、10329、11237，`form_a_at_the_newest_nonzero_txg` 1577、1485、1023，`v1825_differs_from_today` 1577、1485、1023——形甲多于 v1825 红的环，多出的那些就是「不红而 RHS 不成立」的反方向反例；v1825 红的环恰是最大 txg 上有形甲的环。

仓里现成的两张盘（读码，两条用例都是手摆镜像，不是合法历史）：

- v1826：`crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs:120`：「fn a_later_checkpoint_txg_in_an_earlier_slot_beats_a_higher_instance_through_choose_root() {」 摆的是 mkfs 池上 (txg 3, 实例 1) 与 (txg 2, 实例 2) 各在自己的落点槽（S = 8）：每个非 0 txg 一条，RHS 成立；v1826 择 (2, 2)，基线择 (3, 1)。
- v1823、v1825：`crates/singlefs-harness/tests/root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs:60`：「fn of_two_self_verified_roots_with_the_same_checkpoint_txg_choose_root_takes_the_higher_instance_although_it_sits_in_a_later_slot(」 摆的是三区域槽 0 上 txg 0 的实例 0、1、2：没有非 0 txg，RHS 成立；v1825 择 (0, 0)，v1823 择读根环次序最前的 (0, 0)，基线择 (0, 2)。

本腿模型的 `e165_r3_window_disks_split_the_equivalence` 把这件事在窗口里穷举：S ∈ {4, 16}，txg 0..=5 落的六槽每槽 4 到 10 种摆法（空、本槽 txg、本槽 txg + 3S、种根的两槽另摆 txg 0；实例 0..=2），每档 44800 个盘面全过产品读槽与产品 `choose_root`；按 RHS、「txg 0 只有一种实例代号」（下称 T0）、「不倒挂」（下称 NI）分组数各变体择得不同的盘面，并逐盘核第 1 小节三条充要条件（对不上的盘数 assert 为 0；三条错的条件对不上的盘数 assert 大于 0，作阳性对照）。跑前写死的预期见第四节。

### 3. 承诺内的合法写者造不造得出（每条变体一个结论）

| 变体 | 结论 | 挡法在哪一侧、指到哪一句 |
|---|---|---|
| v1823 | 造不出（读码推的） | 要「最大 txg 上两种实例代号」。非 0 txg：读者侧落点核让一个非 0 txg 只住一槽（`crates/singlefs-core/src/recovery.rs:1867`：「if !root_sits_in_the_ring_slot_the_formula_gives(」、`crates/singlefs-core/src/root_ring.rs:192`：「target_for_publish(checkpoint_txg, slots_per_region) == ring_slot」；第二轮量过）。txg 0：读者另认种根三槽（`crates/singlefs-core/src/root_ring.rs:188`：「if checkpoint_txg == crate::make_filesystem::GENESIS_CHECKPOINT_TXG {」），挡在写者侧——mkfs 写实例 0（`crates/singlefs-core/src/make_filesystem.rs:52`：「pub const MKFS_INSTANCE_GENERATION: InstanceGeneration = InstanceGeneration(0);」），挂载一侧的 txg 一律某一版加一（`crates/singlefs-core/src/mount.rs:1427`：「version.0.checked_add(1).map(CheckpointTxg).ok_or(」），新实例首版取 `crates/singlefs-core/src/mount.rs:1419`：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」，环里最小也是 txg 0，加一得 1 |
| v1825 | 造不出（读码推的） | 同 v1823 |
| v1826 | 造不出（承诺内；前两轮的结论，本腿没重攻） | 要倒挂：一条实例更高、txg 更低的根与一条实例更低、txg 更高的根同时进择根。新实例首版 txg 高于挂载时读得出的每一条根与记录（同上 `crates/singlefs-core/src/mount.rs:1419` 那一行），倒挂要「挂载时更高的那条根与它的记录都读不出、之后又读得出」，第一轮量到单故障之内由 N-配置与 R2 挡，第二轮判它落在 C971 那一形、在承诺外（`.claude/kb/decisions/22-单元原子性怎么合成.md:185`：「双故障形不在承诺里（用户 2026-09-28 定不罩）」）。这一挡在写者与挂载门一侧，与 RHS 无关 |

本腿这一轮另找了前两轮没走的几条路，看承诺内能不能不经读故障造出倒挂或 txg 0 上的第二种实例，都没造出（读码，没建模型）：管理员回退（D23 已定项 14 写的是「挂着时的一次向前发布」，txg 照取 max + 1）、实例表清扫的发布（取上一版加一）、抬 F 与卸载（`checkpoint_txg_after_the_version`，加一）、暖机那条老入口（`warm_up` 写常数 txg 1、2；产品可写挂载 `mount_writable` 不调它，调它的是 harness 的 `scenario.rs`、`history.rs` 两处驱动、checker 档与 GPU 装置、`mount.rs` 第 10660 行 `#[cfg(test)]` 之后的测试模块；harness 那两处调它时环是空的，交回的 txg 1、2 不与已有根撞，本腿只读了 `transaction.rs` 那一段的注释，没逐个核调用方）。由用户定的动作（挂载之后写不写文件、挂几次、卸不卸载）不改这一格：每一次可写挂载都重新取 max + 1。

最小历史：三条在承诺内都造不出，不给。承诺外的那一形（v1826）第一轮已给最小历史（第一轮判决第一节）。

### 4. 打中之后的四句（v1826 那一格：RHS 罩不住它）

1. **分不分辨臂**：分辨。它只打中 v1826，v1823、v1825 在补上 T0 之后照旧被 RHS 罩着；被判的对象是「判据里那一句等价」，不是 v1826 本身（v1826 照旧由 Verus 必红世界杀）。
2. **被判的系统当时看不看得到判别它的东西**：这一格判的是留档用例这一判法看不看得到。看不到：读者侧逐槽穷举只调 `read_root_ring_slot`，不择根、不看两条根之间 txg 与实例的序；v1826 红的盘面在读者侧与不红的盘面逐槽相同（每槽读得出与否一样）。这正是第二轮「判别子观测不到」那一形在新判据上又出现一次：新判据留档的东西观测不到 v1826 要的判别子。
3. **满足的是判据字面的哪一个分句**：「承诺内的合法写者造不出让它们红的盘面，等价于『同一非 0 txg 至多一条根进择根』」里「等价于」那个分句的 ⇐ 方向，主语「它们」包括 v1826。前半句（造不出）本身没被打穿。
4. **跑前条款给的每个改法在打中的那几格上还中不中**：第二轮判决没给改法，只给了「做成读者侧逐槽穷举的入库用例」这一个动作；它在这一格上照样中（用例绿着，v1826 那一半没人看）。

### 5. 新判据该改成什么（只在我的模型上量过、被攻过零轮）

> 三条择根变体由 Verus 必红世界杀（只罩择根核里那一份写法，见第二节）；v1826 另由 KL 杀。三条在盘面上与基线择得不同的充要条件各不相同（v1823：进择根的最大 txg 上读根环次序最前那条不是实例最高的；v1825：那个 txg 上有两种实例代号；v1826：有一条根的实例代号高于基线所选那条），承诺内合法写者造不出这三种盘面，靠三样，各留一条会红的入库用例：
> （甲）读者侧落点核：非 0 txg 恰好只在它的落点槽、txg 0 恰好只在种根的三槽、读得出与否不随实例代号变，S 取格式区间每一档（4..=16）逐槽穷举；
> （乙）写者侧：txg 0 的根只有 mkfs 写、实例代号 0，挂载一侧每一次发布的 txg 都大于它接着的那一版；
> （丙）写者与挂载门一侧：进择根的根不倒挂——新实例首版 txg 高于挂载时读得出的每一条根与记录，更新的状态读不出时拒可写；倒挂只在承诺外的双故障形（C971）上出现。
> （甲）（乙）合起来罩 v1823、v1825；（丙）单独罩 v1826，（甲）对 v1826 不起作用。

比第二轮那一句多要求的：把「等价」拆成三条各自的充要条件；（甲）从「至多一条」收严成「恰好在落点槽」并把 S 从三档扩到全区间；新加（乙）（丙）两条写者侧的留档。比原句只收严、不放宽：原句要的读者侧用例照留，新要求都是加的。

（甲）是本腿 `e165_r3_admission_over_every_slots_per_region`，（乙）是本腿 `e165_r3_writable_mounts_on_a_fresh_pool`（第二节量它杀不杀挂载那一种改坏，待跑）；（丙）本腿没写用例，推的候选是「挂载时最新那条根与它的记录读不出、之后读得出，可写挂载要拒」的 harness 档用例，现有哪条已罩着本腿没查全（`writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs`、`root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs` 这一类，没读）。

## 第二节 Q1.2：只罩读者侧的留档用例，判别力够不够

### 1. 两样守卫各自看得见什么

- **Verus 必红世界只改 `crates/singlefs-replay-prefix/src/` 下的文件**（量的）：`awk -F'\t' '!/^#/ && NF==6 {print $2}' crates/singlefs-replay-prefix/verus-must-red-worlds.tsv | sort | uniq -c` 数出 296 行，文件列全在 `crates/singlefs-replay-prefix/src/`（`lib.rs` 158 行，其余六份证明文件 138 行），`crates/singlefs-core` 一行都没有。所以「三条择根变体由 Verus 必红世界杀」只对择根核里那一份写法成立；同一个语义写在 `singlefs-core` 里（择根胶水 `crates/singlefs-core/src/recovery.rs:1916`：「checkpoint_txg: PrefixKernelCheckpointTxg(root.checkpoint_txg.0),」 那一段把根摊成择新键，读者 `root_ring.rs` 的落点核，写者 `mount.rs` 的取 txg），必红世界一行都不碰。
- **留档用例原型**（第二轮判决点名「攻方第二份模型那一条」，即 `e165_attack_form_a_reader.rs` 的 `e165_form_a_admission_per_slot`）读码三处弱点：
  1. 判据字面只要「至多一条」；原型打出的 `nonzero_txgs_not_exactly_at_their_formula_slot`（恰好在落点槽）与 `slots_admitting_txg_zero`（txg 0 恰好在种根三槽）两列更严，但判据没收它们。
  2. S 只取三档：`research/prompts/m3-close-b19-e165-r2-opus-model/e165_attack_form_a_reader.rs:49`：「const SLOTS_PER_REGION_SAMPLED: [u64; 3] = [4, 8, 16];」，三个都是 2 的幂；而 mkfs 默认 S 落在 5..=8（`crates/singlefs-core/src/root_ring.rs:82`：「第一版 mkfs 的默认 S：min(8, ⌊P × chunk ÷ 固定结构槽距⌋)，小于 5 拒」），5、6、7 是会被用户拿到的默认值。
  3. 产品臂一条 assert 都没有：这条用例里唯一的 assert 在 `research/prompts/m3-close-b19-e165-r2-opus-model/e165_attack_form_a_reader.rs:398`：「if arm == SlotReaderArm::ControlWithoutTheFormulaCheck {」 那一支（对照臂要报出形甲）。照原样入库，它对产品读者改成什么样都绿，红绿要靠人读 OPUSMODEL 行。

### 2. 同时躲过两样守卫的改坏（每种只改 `singlefs-core` 一行；改法原文与替换文写死在 `tools/rerun.sh` 的 `apply_the_change`）

| 改法（`rerun.sh` 里的名字） | 改哪一行 | 必红世界 | 原型逐槽用例 | 后果（推的） |
|---|---|---|---|---|
| 读者 txg 0 分支放宽 `reader_admits_txg_zero_in_every_slot` | `root_ring.rs` 种根槽那一判改成恒真：txg 0 的根在任何槽都收 | 不碰（文件不在表里） | 绿；`slots_admitting_txg_zero` 变成全部 3S 槽，没 assert | 合法写者只在种根三槽写 txg 0（mkfs 先清根环区），错位读读回的也是 (0, 0) 那一份，择根不变；但读者不再照 D22 已定项 7 那一行核 txg 0 的落点，挂载记读坏槽那一账在错位读下会变（没分析） |
| 读者落点式改成按位与 `reader_formula_masks_with_slots_per_region_minus_one` | `root_ring.rs:192` 那一判换成 `(T / 3) & (S − 1)`（写者仍按 `% S` 写） | 不碰 | 绿，且三档输出与没改过逐字节同（4、8、16 上 `& (S − 1)` 等于 `% S`） | S ∈ {5, 6, 7, 9..=15} 的池上，许多 txg 的根写在 `% S` 那一槽、读者只认 `& (S − 1)` 那一槽，自己写的根读成读坏，择根退到更旧的根或第 0 代根，已确认的发布丢；「至多一条」在这些 S 上照样成立（每个 txg 仍只认一槽），只有「恰好在落点槽」那一列分得出 |
| 择根胶水实例代号在前 `glue_puts_the_instance_before_the_txg` | `recovery.rs:1916` 择新键的 txg 一项改成 `(实例 << 32) \| (txg 的低 32 位)`：语义就是 v1826，只是写在胶水里 | 不碰（择根核没动，Verus 照绿） | 绿（只调读槽，不择根） | 与 v1826 同；仓里胶水用例（`root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs`）专为这一形写的，该红；原型的整环用例 `product_choose_root_differs_from_today` 那一条 assert 也该红 |
| 挂载一侧下一个 txg 加 0 `mount_next_txg_adds_zero` | `crates/singlefs-core/src/mount.rs:1427` 的 `checked_add(1)` 改成 `checked_add(0)`（新实例首版与暖机、抬 F、卸载那几次都接在「同一版」上） | 不碰 | 绿（不调写者） | **承诺内、不带读故障、不带崩溃**：mkfs 之后用户只做一次可写挂载，写行那次发布写 (txg 0, 实例 1) 进区域 0 槽 0（`target_for_publish(0)` 就是种根槽），暖机三次都还是 txg 0（`crates/singlefs-core/src/mount.rs:6233`：「let next_txg = checkpoint_txg_after_the_version(latest_txg)?;」）；盘上区域 0 槽 0 是 (0, 1)、区域 1、2 槽 0 仍是 mkfs 的 (0, 0)，最大 txg 是 0、上面两种实例代号，v1825 择 (0, 0)、基线择 (0, 1)。再挂一次，实例 2 又写 txg 0 |

挂载那一种是本节要的「改坏写者、让承诺内合法历史造出变体红的盘面、两样守卫都看不见」的那一形；它不是「改坏择根序或读者」，是正文 Q1.2 问的写者侧那一半。前三种是「改坏读者或择根」：读者那两种躲过两样守卫、只被本腿逐档用例抓；胶水那一种躲过两样守卫、被仓里胶水用例抓。

为什么挂载那一种躲得过读者侧用例：读者侧逐槽穷举自己摆字节、只调 `read_root_ring_slot`，被测代码里没有一行写者（按构造，读码）。为什么躲得过必红世界：`mount.rs` 不在表里（上一小节量的）。仓里别的用例杀不杀它没查全：本腿在 `rerun.sh` 里只带了 `first_file_version_checks_the_previous_record` 一个现成目标去看，结局待跑。

### 3. 写者侧「txg 0 的根只有 mkfs 写」要不要另一条用例

要。理由就是上一小节最后一行：读者侧用例按构造看不见写者，必红世界不验 `singlefs-core`，而 v1823、v1825 在承诺内造不出，靠的正是写者这一半（第一节第 3 小节）。本腿给的那一条是 `e165_r3_writable_mounts_on_a_fresh_pool`：mkfs 之后可写挂载、再挂一次，每次之后读根环，assert txg 0 上只有实例 0、三条变体与 today 择同一条、两次挂载都做成；用户动作只放开了「挂几次」这一维（一次、两次），「挂载之后写不写文件」没放开——写一个文件之后最大 txg 就离开 0，挂载那一种改坏在那条历史上不让变体红，这一格只判「最大 txg 停在 0 的那条历史上中不中」。v1826 那一半（不倒挂）另要一条，本腿没写（第一节第 5 小节）。

## 第三节 Q1.3：第二轮那几个读数能不能核

### 1. 本腿现核的（量的，只读文本）

- 7660 的日志：`task-server/task log 7660 --all`（号取自第二轮判决第一节）存进草稿目录 `task-7660-log.txt`（35 行）。其中 OPUSMODEL 行的集合与主 agent 存的 `model-7660-opusmodel-lines.txt` 逐行相同：`diff <(grep -o 'OPUSMODEL.*' <日志> | sort) <(grep -o 'OPUSMODEL.*' <主 agent 那份> | sort)` 无输出；主 agent 那份 23 行、`grep -c OPUSMODEL` 23。
- 7660 日志里九行 `name=frozen_source` 的 sha256 与今天主工作区的同名文件逐个相同（`sha256sum` 现算：`recovery.rs` 3d453f96…、`root_ring.rs` 92402c9c…、`make_filesystem.rs` b45515cb…、`root_record.rs` 853b1205…、`transaction.rs` 1782390f…、`singlefs-replay-prefix/src/lib.rs` 66f08858…、`memory_pool.rs` 3a3fe304…、`.claude/heavy-thread-default` 66aeed9f…），`name=model_source` 的 e07dbb20… 与模型文件今天的 sha256 相同；日志末两行是「test result: ok. 3 passed」与 `name=rerun_roots … model_exit=0`。
- 7661（自证）日志第 9 行：「✓ 自检通过（查了 6 格）」，与判决第一节写的一致。
- 判决第一节逐条对 7660 的行：
  - 「S = 4、8、16 三档：nonzero_txgs_admitted_in_two_or_more_slots=0；对照臂同三档是 26、50、98」：对上（主 agent 那份第 21、3、6 行产品臂是 0；第 1、4、7 行对照臂 26、50、98）。对照臂的数是 6S + 2（txg 1..6S + 3 的每一个在全部 3S 槽都收），按模型第 351 行 `txg_bound = 2 * ring_length + 3` 推得出，读数不随摆法抽样变。
  - 写者族产品臂那一句：三档 `form_a_at_a_nonzero_txg=0`、`v1823_differs_from_today=0`、`v1825_differs_from_today=0` 对上；但判决第 15 行写成 `research/prompts/m3-close-b19-e165-r2-main-verification.md:15`：「form_a_at_a_nonzero_txg=0 … v1823_differs_from_today=0 v1825_differs_from_today=0」，用「…」截掉中间两列、并在 v1825 之后截断，同一行紧跟着的 `v1826_differs_from_today=6497`（S = 4；S = 8、16 是 7990、8401）没抄。那一列正是第一节打中的那一格的反例。照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@引产物就整行抄` 的「引产物整行抄，带上文件名」，这一处该整行抄。
  - 任意编码一族 `form_a_at_the_newest_txg_zero` 2703、2691、2686：对上（第 11、15、19 行）。
  - 「单线程与切片多线程对拍 3000 环 single_thread_equals_sliced=true」：对上，两臂各一行（第 9、22 行）。
- 能不能逐字段复现：摆法的种子只由 (S, 族, 环号) 定（模型第 460 行 `ring_seed`），切片合并按片的次序，`threads=` 打的是登记值 31；被测源文件与 7660 那一次相同（上一条）。推的：今天原样复跑，23 行除 `seconds=` 之外逐字段同；`threads=` 跟着 `.claude/heavy-thread-default` 走，那份文件改了就不同。

### 2. 复跑的整条命令与该打出的行

命令在文末「要主 agent 跑的命令」第 2 项。跑完该打出的（日志里 `OPUSMODEL` 起头与 `test … OPUSMODEL` 起头的行，共 23 行，与 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r2-main/model-7660-opusmodel-lines.txt` 去掉 `seconds=` 一列之后逐行同）；判决第一节引的那几格逐个看：

| 判决引的读数 | 在哪一行 | 该打出的字段 |
|---|---|---|
| 产品读者逐槽，三档 | `name=admission arm=product slots_per_region=4`、`=8`、`=16` 三行 | `nonzero_txgs_admitted_in_two_or_more_slots=0`、`nonzero_txgs_not_exactly_at_their_formula_slot=0`、`slots_admitting_txg_zero` 等于 `genesis_slots` |
| 对照臂逐槽，三档 | `name=admission arm=control_without_the_formula_check` 三行 | `nonzero_txgs_admitted_in_two_or_more_slots=26`、`=50`、`=98` |
| 写者族整环、产品臂 | `name=ring_choices … family=writer_reachable arm=product` 三行 | `form_a_at_a_nonzero_txg=0`、`v1823_differs_from_today=0`、`v1825_differs_from_today=0`，**以及** `v1826_differs_from_today=6497`、`=7990`、`=8401` |
| 任意编码族、产品臂 | `family=any_encoding arm=product` 三行 | `form_a_at_a_nonzero_txg=0`、`form_a_at_the_newest_txg_zero=2703`、`=2691`、`=2686` |
| 对拍 | `name=cross_check` 两行 | `single_thread_equals_sliced=true` |
| 自证 | 第 3 项命令的日志 | 「✓ 自检通过（查了 6 格）」 |

推翻条件：今天复跑有任一行除 `seconds=` 外与 7660 不同（被测源文件 sha256 又相同），说明这份模型的读数不由种子与源文件定，第二轮判决第一节那几个数不能当「量的」引。

## 第四节 模型：用例、数量、线程与跑前写死的读法

### 1. 用例与数量（数量按源码常量算的，推的，待主 agent 跑出的 OPUSMODEL 行核）

| 用例 | 判什么 | 盘面或态数 | 线程 |
|---|---|---|---|
| `e165_r3_window_disks_split_the_equivalence` | 第一节第 1 小节三条充要条件逐盘核；按 RHS、T0、NI 分组数各变体择得不同的盘面 | 每档 7 × 10 × 10 × 4 × 4 × 4 = 44800，S ∈ {4, 16} 两档 89600 | 登记的每台默认重活线程数（`default_heavy_threads`，此刻登记 31），`std::thread::scope` 按盘面号区间切片 |
| `e165_r3_window_disks_sliced_matches_single_thread` | S = 4 前 22400 个盘面单线程与切片多线程逐项相同 | 44800（两遍） | 1 与登记值 |
| `e165_r3_admission_over_every_slots_per_region` | S = 4..=16 每档逐槽 × txg 0..6S+3 × 实例 0..=1 | Σ 3S × (6S + 3) × 2 = 55692 次读槽 | 每档一线程（13 档，线程数取 min(登记值, 13)） |
| `e165_r3_writable_mounts_on_a_fresh_pool` | mkfs 后可写挂载两次，每次之后读根环 | 2 张盘 | 1 |

合计 189492 份盘面或读槽，压在 20 万以内。单项挂钟推的都在一分钟以内（第二轮同形的逐槽与整环用例在 7660 上整个目标 0.24 秒）；过 300 秒每片打一行 `name=progress`。`rerun.sh` 在同一份副本上编五棵树（没改过的与第二节四种改坏），每棵树重编 `singlefs-core` 与它下游；挂钟主要是编译，推的二十到三十五分钟，没量。没有崩溃状态（不调崩溃枚举函数）：可写挂载那条用例只读「挂载做完或报错、panic 之后」盘上的根环，不枚举崩溃点；第二节挂载那一种改坏的打中也不靠崩溃。

### 2. 跑前写死的读法（结果反过来照改结论）

没改过的树（`tree=unmutated`）：

| 读数 | 预期 | 反过来怎么改 |
|---|---|---|
| `name=window_disks` 两档：`placed_but_not_admitted=0`、`admitted_outside_the_window=0`、`product_choose_root_differs_from_today=0`、`nonzero_txg_at_most_one` 等于 `disks` | 都如此（assert 前三项） | 前三项不对：模型摆法或本地 today 错，作废；最后一项不等：产品读者在落点槽上漏收，第一节第 2 小节的读法要重看 |
| `characterization_mismatches=[0, 0, 0]`；`wrong_characterization_mismatches` 三项都大于 0 | 如此（assert） | 前者不是 0：第一节第 1 小节的充要条件推错了，那一节撤回；后者有 0：阳性对照没判别力，模型作废 |
| `differs[v1823,v1825,v1826]` 三项都大于 0；`differs_given_nonzero_txg_at_most_one` 等于 `differs` | 如此 | v1826 那一项是 0：「RHS 罩不住 v1826」在这个窗口里没量到，第一节第 4 小节退回只靠 7660 那三行 |
| `differs_given_txg_zero_one_instance` | v1823、v1825 两项是 0，v1826 大于 0 | v1823、v1825 不是 0：RHS 加 T0 不够，新判据（甲）（乙）要再补 |
| `differs_given_txg_zero_one_instance_and_no_inversion` | 三项都是 0 | 不是 0：（甲）（乙）（丙）三样合起来也罩不住，新判据撤回 |
| `name=window_example … why=smallest_disk_where_v1826_differs_with_one_txg_zero_instance` | 打出一张两条非 0 txg 根倒挂的盘 | 打不出：同上一行 v1826 那一格 |
| `name=admission_every_geometry` 13 行 `holds=true`，`geometries_that_fail=[]` | 如此（assert） | 不是：今天的读者在某一档 S 上就不合落点公式，这是新发现，先查那一档 |
| `name=writable_mount_ring` 两行：`outcome=ok`、`txg_zero_instances={0}`、`variants_differing=[]` | 如此（assert） | 不是：今天的写者就写出了 txg 0 上的第二种实例，第一节第 3 小节 v1823、v1825 那两格翻成「造得出」 |
| 六个测试目标 `test result: ok` | 全绿，`cargo_exit=0` | 有红：先看红的是哪条，本腿模型红就是模型错 |

四种改坏的树（`tree=<改法>`；每格写「该红 / 该绿」）：

| 用例或目标 | 读者 txg 0 放宽 | 读者按位与 | 胶水实例在前 | 挂载加 0 |
|---|---|---|---|---|
| 原型 `e165_form_a_admission_per_slot` | 绿（`slots_admitting_txg_zero` 变成全部槽） | 绿，三档行与没改过逐字节同（`seconds=` 除外） | 绿 | 绿 |
| 原型 `e165_form_a_whole_ring_choices` | 绿 | 绿 | 红（`product_choose_root_differs_from_today` 那条 assert） | 绿 |
| `e165_r3_admission_over_every_slots_per_region` | 红，13 档全落 `geometries_that_fail` | 红，`geometries_that_fail=[5, 6, 7, 9, 10, 11, 12, 13, 14, 15]`，这些档 `nonzero_txgs_admitted_in_two_or_more_slots=0` 而 `nonzero_txgs_not_exactly_at_their_formula_slot` 大于 0 | 绿 | 绿 |
| `e165_r3_window_disks_split_the_equivalence` | 绿 | 绿（4、16 都是 2 的幂） | 红（`product_choose_root_differs_from_today`） | 绿 |
| `e165_r3_writable_mounts_on_a_fresh_pool` | 绿 | 绿（S = 8） | 绿 | 红：`mounts_done=1` 那一行 `txg_zero_instances={0, 1}`、`variants_differing=["v1825"]`（挂载报错或 panic 也照打那一行） |
| `root_ring_readers_take_a_root_outside_its_formula_slot_for_a_bad_read` | 绿（只摆非 0 txg） | 绿（S = 8） | 绿 | 不知道（它的池走过新池新建文件） |
| 胶水用例 `root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance` | 绿 | 绿 | 红（`a_later_checkpoint_txg_in_an_earlier_slot_beats_a_higher_instance_through_choose_root`，`recover_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root` 也该红） | 不知道（有一条可写挂载） |
| 平局用例 `root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot` | 绿 | 绿 | 绿（txg 0 平局，实例在前也择实例 2） | 绿 |
| `first_file_version_checks_the_previous_record` | 绿 | 绿 | 绿 | 不知道 |

反过来怎么改：挂载加 0 那一列 `e165_r3_writable_mounts_on_a_fresh_pool` 绿（txg 0 上仍只有实例 0），第二节挂载那一行的打中撤回；读者按位与那一列原型行与没改过不同，「S 只取 2 的幂」那一条撤回；胶水那一列胶水用例绿，「胶水由仓里用例杀」那一句撤回；任一棵改坏的树 `name=tree … cargo_exit` 之前有 `could not compile`，那一种改坏作废（`rerun.sh` 退 3），本腿改法写错了。

## 第五节 没打中的形状

| 形状 | 怎么试的 | 取样范围 | 结局 |
|---|---|---|---|
| 承诺内不经读故障造出倒挂（v1826 要的） | 读码：管理员回退、实例表清扫发布、抬 F、卸载、暖机老入口、新实例首版的 txg 来源 | 产品代码里取 txg 的五处（`crates/singlefs-core/src/mount.rs:1419`、`crates/singlefs-core/src/mount.rs:1427`、`mount.rs` 第 5492 行起 `checkpoint_txg_of_the_planned_publish_after`、`instance_table_sweep.rs` 清扫计划的 `checked_add(1)`、`transaction.rs` 的 `warm_up_after_journal_counter`） | 都是「接着的那一版 + 1」或「环里最大 + 1」，造不出（推的） |
| 承诺内造出 txg 0 上第二种实例（v1823、v1825 要的） | 同上，加 mkfs 写的实例代号 | 同上 | 没改过的代码造不出（推的）；改坏挂载那一行就造得出（第二节，待跑） |
| 读者改坏只在大 txg 上露出（例如 txg 截成 32 位） | 没摆 | 本腿逐档用例的 txg 只到 6S + 2 | 没试；txg 到 2³² 要四十亿次发布，读码看不出这样写的必要，记为取样限度 |
| 读者改坏随实例代号变 | 逐档用例实例 0..=1、`instance_dependent_admissions` 一列 | 两种实例 | 没造这样的改坏（没有读到会写出它的理由） |
| `target_for_publish` 本身改成按位与（写者读者一起改） | 读码 | — | 写者读者同改、自洽，读者侧用例照绿，择根不受影响；它改的是盘上布局，归池级 checker 的独立落点公式判，本腿没跑 |
| 挂载加 0 那一种改坏在「挂载之后写一个文件」的历史上 | 读码 | — | 写文件的发布走 `transaction.rs` 自己的加一，最大 txg 离开 0，三条变体不红；这条历史上不中（推的，没跑） |

## 第六节 这条腿自己的限度

- 模型与 `rerun.sh` 一行都没编、没跑；Rust 源码是对着第二轮能编过的模型与今天的源码写的，编不过就是本腿的源码错，交回新派同立场的腿修。上文模型读数与时长一律推的。
- 模型与读码共用同一份被测代码（产品读槽、产品 `choose_root`、产品挂载）；能抓到「我读错了代码」，抓不到「代码与条款错在同一处」。
- 逐槽穷举的窗口只罩 txg 0..=5 落的六槽、两档 S、实例 0..=2；充要条件的推法对任意组根成立，窗口只是核它。
- 四种改坏是本腿挑的，挑法是「两样守卫各自看不见的那一侧各挑一个看得见后果的」，不是全量变异；别的改坏躲不躲得过没量。
- 可写挂载那条用例只放开了「挂几次」，没放开「挂载之后写不写文件、写几个」；只读挂载做完或报错之后的盘，不枚举崩溃状态。
- 新判据（丙）那一条用例本腿没写，现成用例里哪条已罩着没查全。
- 7660、7661 的日志是本腿读的（号取自第二轮判决），读数照「量的」写，只因为主 agent 当时跑过、本腿只做了逐行比对。

## 要主 agent 跑的命令

一条一项。判决要引的数照 `.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做` 由主 agent 把模型目录拷进自己的草稿目录再交；下面写的是在本腿草稿目录里跑的形态，`--cwd` 与 `--work` 换成主 agent 自己的新目录即可。本轮没有冻结副本目录，`--frozen` 照第二轮给仓根，脚本打的 `name=frozen_source` 行可与 `research/prompts/m3-close-b19-e165-r3-snapshot/crates-sha256.txt` 逐行比。

1. 本腿模型整份复跑（同一份副本上五棵树，每棵编一次、六个测试目标一趟）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r3-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r3-opus/rerun-model`
   - 范围声明：脚本里那条 `cargo test` 自己带 `SINGLEFS_RUN_SCOPE='种类=target; …; 范围=6 个测试目标; …'`（`run_the_targets`），submit 前面不再写；被 `.claude/hooks/run-scope-guard.sh` 拒就照它的出路改。
   - 在哪起：`--cwd` 那个目录。跑完 `$WORK/tree` 是一份仓副本加它的 `target`，读完数照共用约束删掉；五份 `$WORK/tree-<改法>.out` 是各棵树的整份输出。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`（等于没改过那棵树 cargo 的退出码；3 是有一棵树编不过，4 是改坏的树上 `singlefs-core` 没重编，这两种整趟作废）；`name=rerun_roots … unmutated_exit=0 compile_failures=0 stale_builds=0`；五行 `name=tree label=<改法> cargo_exit=`（没改过的是 0，四种改坏照第四节第 2 小节那张表，该红的那几棵非 0）；再照第四节两张表逐行读 `tree=<改法>` 起头的 `OPUSMODEL`、`test … ok/FAILED`、`panicked at` 行。
2. 第二轮攻方模型原样复跑（Q1.3）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r3-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r3-opus/rerun-r2-model`
   - 在哪起：同上；跑完 `$WORK/tree` 照删。
   - 看哪几行：`exit=0`；「test result: ok. 3 passed」；`name=rerun_roots … model_exit=0`；九行 `name=frozen_source` 与 7660 那九行 sha256 相同；然后（直接跑的文本比对，不经服务）`diff <(task-server/task log <号> --all | grep -o 'OPUSMODEL.*' | sed 's/ seconds=[0-9.]*//' | sort) <(grep -o 'OPUSMODEL.*' /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r2-main/model-7660-opusmodel-lines.txt | sed 's/ seconds=[0-9.]*//' | sort)` 无输出算核上；再照第三节第 2 小节那张表逐格看。
3. 本腿 `rerun.sh` 的自证（假驱动，只拷副本、不编不跑）：
   - 命令：`task-server/task submit -- bash research/prompts/m3-close-b19-e165-r3-opus-model/tools/rerun.sh --selftest`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：「✓ 自检通过（查了 6 格）」那一行，`exit=0`。

## 没做什么

- 没判正推、辩方那几格（Q2 一格没碰），没替主 agent 采纳。
- 没往统一任务服务交任何任务；模型没编、没跑，判决要引的数都没复跑（命令在上一节）。副本上量出的数（第 1 项的四棵改坏的树）不算入库装置上的数。
- 新判据（丙）那一条的用例没写；第二节挂载那一种改坏被仓里哪些现成用例杀没查全。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r3-opus` 里只有本腿读的两份任务日志（`task-7660-log.txt`、`task-7661-log.txt`）；没建仓副本、没建编译目录，没有要删的。
- `python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-e165-r3-opus-output.md` 的末行（写这一行之前那一遍，退 0）：「✓ 核了 20 处引文、6 处裸行号，对上 20 处，没判 6 处（其中裸行号没有引文可比 6 处；逐处列在上面）」。
