# m2-s1-r1 正推腿报告（Sonnet）

范围：H1、H2、H5；逐条核正文第二、三、四节的前提、代码事实与数；每条臂列「定下来之后要改哪些东西」清单。不判 H3、H4、H6、H7，不碰别的腿的格。

工作区状态：`git hash-object` 核过背景材料第三节表里的四个文件，与背景材料记的哈希逐字相同（`transaction.rs`=`d7de039c...`、`journal.rs`=`2467e85e...`、`recovery.rs`=`7e89052c...`、`write_accounting.rs`=`2659af01...`），本报告引用的行号都是这一版。

---

## 一、H1：E16「发根恒为 1 块/fsync」今天还成不成立

**判定：字面成立，作为「所以发根代价可忽略」的支撑今天不成立——主 agent 第一节的推论站得住。**

**正推**：

1. 原句现查：`research/prompts/_e16-background.md:79`「轴一『每次 fsync 发不发根』：写放大上恒为 1 块/fsync（可忽略）」。这份材料同一份文件 `:35` 给出的 E16 模型只有「完全 128 叉树、内部 4 层、块 4 KiB、根环槽 1 块」一种结构，没有记账、分配记录、映射、树表四样固定点。
2. E16 自己的实验页已经预告了这个缺口，原句现查 `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:220`：「**记账块自身的 COW 代价完全没建模**（D22（单元原子性怎么合成） 已定项 3 已定记账结构走 COW，而这笔代价没进模型）。若它按 checkpoint 次数缩放，checkpoint 次数最多的 `intent` 会被进一步拉开劣势」。
3. E155 按 `crates/` 今天的写法把四样固定点补进模型后，甲与 wal_full 每次 fsync 的字节差从「1 块」变成 P=1 时 139 776 字节（占甲 40.6%）、P=10⁸ 时 1 462 708 字节（占甲 79.9%）——数已在 H5 逐格核对，与产物 `research/results/e155-fsync-write-volume-2026-09-17-stage4.out` 完全一致。
4. 代码核实「根槽写」这一步本身确实只是一次写：`crates/singlefs-core/src/transaction.rs:76-80`，`WriteRootRecordForceUnitAccess` 只带 `checkpoint_txg` 与 `root_slot` 两个字段，一次调用写一个槽。但甲今天没有「只发根、不重写四样固定点」这条路径：`grep -n "checkpoint" crates/singlefs-core/src/transaction.rs` 47 处命中全是 `checkpoint_txg` 这个字段名（用 `Bash` 现跑，命中数与背景材料第三节所记一致），没有独立的「只写根」分支；D16 已定项 1「攒够一批脏节点再发布一次根」与「fsync 的效果是『提前触发一次发布』」（原句见下一段）把发根与四样固定点的重写绑成同一步。

**反推（什么观测会推翻这条判定）**：若能在 `crates/` 里找到一条把「发根」与「重写四样固定点」拆开、单独只发根的合法路径，「1 块/fsync 可忽略」这句话作为推论就仍然成立，说明是 D16/D23 后来才把两者捆死，不是 E16 的模型本身有缺口。已跑 `grep -n "checkpoint" crates/singlefs-core/src/transaction.rs` 与通读 `CommitStep` 五种步骤的 `match`（`transaction.rs:121-186`，`PoolWriter::perform`），没有找到这样的分支——五种步骤里固定点结构与叶/祖先走同一种 `WriteUnitToEveryDevice`，没有条件跳过。这条判定没被推翻。

**校验（独立路子）**：不依赖 E155 的模型，直接读 D16 已定项 1 原文（`.claude/kb/decisions/16-发布语义.md:876`）「攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次」，以及连带定死的三条第 1 条（`:894-896`）「没有任何操作可以绕过 journal 直接发根……fsync 的效果是『提前触发一次发布』，不是另开一条日志格式、另一套恢复逻辑」——这两句独立确认：本工程今天的语义模型里，「发根」这个动作本来就不是一个可以单独拆出来计价的原子步骤，它与「一次发布」等价，而一次发布按 D23 已定项 1 定义为「脏叶 + 全部祖先 + 根槽 + 一条记录」外加（按已定项 3/D5/D19/D3）四样固定点。这条独立读法与 E155 的数值证据方向一致：今天没有条款支持「发根」单独计价为 1 块。

## 二、H2：轴一四条依据按 `crates/` 今天的样子逐条核

**先报一个必须先讲清楚的事实缺口**：这四条依据（环截断不了 / 多一条挂载期重放路径 / journal 进验证链 / 屏障一个没省）在全仓只有一处文字——`.claude/kb/decisions/23-journal的角色与格式.md:990` 的一个斜杠分隔括注。已跑：

```
grep -rn "环截断不了\|挂载期重放路径\|journal 进验证链\|屏障一个没省" .claude/kb/
```

命中只有 `23-journal的角色与格式.md` 这一处（`:990`）。全仓（含 `decisions-history/`、E16/E22/E44 实验页、milestone 文档）没有任何地方对这四条各自展开过论证；唯一相关的展开句是同文件 `:996`「轴一依据 2 说的『发根时重放不施加任何东西』只对 journal redo 成立」，但这句话本身没有点名它对应四个标签里的哪一个。**这意味着下面对①②③的判断，除了能对着别处的已定条款与代码事实做交叉核实之外，标签本身的原始论证已经不可考——这是本轮判 H2 的一个结构性限制，不是我漏读**。

### ①「环截断不了」

**读法（按 `research/prompts/_e16-background.md:78-81` 与 E155 5.4 重建，标「推论性转述」，非引文**：若不在每次 fsync 发根（即 wal_full / 乙），journal 环里的在飞记录数在两次 checkpoint 之间只涨不消，要等 checkpoint 的根落盘才能截断；甲每次 fsync 立即发根，环恒定很小。

**今天按 crates 核**：E155 5.4 原句现查 `research/prompts/e155-preregistration.md:820`「做完会怎样：……在飞记录在间隔里单调涨到 `N × r_f + r_c`……checkpoint 的根槽落盘之后归 0」——产物字段印证：`write_ahead_log_full_in_flight_peak=17`（P=1,F1,seq,n=16,Lbalanced 一格，产物同一行）。这与①依据的方向完全一致：**wal_full 今天确实是「环截断不了，要等 checkpoint」的那一类**，依据没有被推翻。

**但这条依据答的是「journal 环占用/多深」，不是「每次 fsync 写多少字节」**——它与 H1 打中的那部分（四样固定点的写放大）是两个正交维度：一个钉的是 journal 环大小要开多大，一个钉的是发布路径本身的字节。E155 没有削弱这条依据，它答的不是同一个问题。

### ②「多一条挂载期重放路径」

**今天按 crates 核**：甲下 redo 为空，原句现查 `.claude/kb/decisions/23-journal的角色与格式.md:636-637`「正确说法是『甲下 redo 为空』，不是『甲下恢复无动作』」，代码印证 `recovery.rs:821-917` 的 `replay_journal`，施加一条记录只换根的四个字段（`recovery.rs:905-914`：`tree_table`、`tree_identifier_watermark`、`rollback_floor`、`mapping_root`），不推导任何树/分配/记账。wal_full 若要实现，恢复期必须新增「从记录里的子树根现推四样固定点」这条路径（这正是 H3 的范围），今天零代码：`grep -rn "推导\|reconstruct" crates/singlefs-core/src/recovery.rs` 命中 0 次（已跑）。

⇒ **这条依据按 crates 今天的样子成立，没有被 E155 削弱**：甲确实没有这条额外路径，wal_full 确实需要新增一条且目前零实现。

### ③「journal 进验证链」

**今天按 crates 核**：D13 已定项 5 定「记录核对器」是第二实现的成本承担者（`.claude/kb/decisions/13-验证路线.md` 已定项 5，见附录）；代码已有实现 `crates/singlefs-harness/src/crash.rs:478` 起，其两条判据在甲的持久顺序下恒 0，原句现查 `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs:89`「记录核对器两条判据在已定的持久顺序下恒 0」。这套判据是按甲「施加=换四个字段」这套简单语义写的；换成 wal_full，记录核对器要理解「记录点名子树根、固定点延到 checkpoint」这套新语义，今天没有对应实现。

⇒ **这条依据同样成立、没被削弱**：对甲成立（验证链简单，已钉死恒 0），对 wal_full 今天不成立（要多实现、验证链变复杂，且值多少一次都没量过）。D23 已定项 1 自己留的反向判据（G23.3 那段，见附录「重开轴二的可判据」）——若 O2 抓到而其余三者都没抓到的事件数持续为 0，则这条依据的增量价值实测为零——**这个反向判据不是 E155 触发的**，E155 是写路径的计数模型，不是崩溃测试，所以它没有让这条依据的价值归零，H2 不因 E155 而动这一条。

### ④「屏障一个没省」

**今天按 crates 核**：D16 已定项 7 原句现查 `.claude/kb/decisions/16-发布语义.md:287-288`「一次发布的持久顺序恒为：COW 单元/节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）」，`:300-301`「屏障口径：两道 FLUSH + 根槽 FUA = 每次发布三个序点」——即两道屏障 + 一次 FUA。E155 5.4 wal_full 的 fsync 次序原句 `research/prompts/e155-preregistration.md:817`「次序：单元 → 屏障 → 记录 → 屏障。**不写根槽、不写超级块槽**」——wal_full 的 fsync 仍然是两道屏障，与甲相同；省掉的是根槽那次 **FUA**，FUA 挪到 checkpoint 才发生一次。产物字段印证：`jia_barriers=4`（两道×两盘）`jia_fua=1`（同一份 P=1,F1,seq 行）。

⇒ **这条依据字面上今天仍然成立**（wal_full 每次 fsync 确实一个屏障都没省），但它把「屏障」与「FUA」放在同一句话里当成同一类东西的省不省，而 crates 的持久顺序把两者分开记（屏障是 FLUSH、FUA 是根槽专属的写语义）——wal_full 省掉的其实是 FUA 这一步，不是屏障。**这是依据表述本身的一处精度问题，我把它标出来，不据此判依据被推翻**：字面判据（屏障数）没错，但如果这条依据原本想论证「wal_full 不省任何同步开销」，那就漏算了 FUA 这一项的节省，读者据此可能高估甲相对 wal_full 的同步开销优势。

**H2 小结**：四条依据按今天的 `crates/` 样子，**没有一条被 E155 或今天的代码事实推翻**；①②③方向不变、④字面成立但表述有一处「屏障≠FUA」的精度问题需要写清楚。四条依据全部只答「wal_full 相对甲多花什么代价」，一条都不答「甲相对 wal_full 每次 fsync 多写多少字节」——这正是 D23 已定项 1 当年立论时没有覆盖、E155 现在补上的那个维度，H1 与 H2 因此互不冲突：H1 打中的是「1 块可忽略」这句已经过时的量化，H2 核的四条依据本身仍然站得住，只是它们从来没有回答过 H1 问的那个问题。

## 三、H5：第四节那张表对不对得上产物、wal_full 与乙的代价有没有漏项、漏项会不会翻次序

### 3.1 表格逐格核对——全部对得上

用脚本从产物里现取同一批格，与背景材料第四节的表逐格比对：

```
grep "name=row1_grid" research/results/e155-fsync-write-volume-2026-09-17-stage4.out | grep "policy=Lbalanced" | grep -E "..." | grep " n=16 "
```

七行全部核对（家族/落点/P、甲 fsync、wal_full fsync、wal_full 摊销、乙 fsync、乙 摊销，逐字节相同；百分比现算逐格相同）：

| 族 | 落点 | P | 甲 | wal_full fsync | wal_full 摊销 | 乙 fsync | 乙 摊销 | 甲−wal_full/甲（现算） | 材料写的 |
|---|---|---|---|---|---|---|---|---|---|
| F1 | seq | 1 | 344576 | 204800 | 214048 | 172032 | 183328 | 40.57% | 40.6% ✓ |
| F1 | seq | 10⁴ | 740712 | 237568 | 269738.25 | 172032 | 208298.25 | 67.93% | 67.9% ✓ |
| F1 | seq | 10⁸ | 1831348 | 368640 | 460786.875 | 172032 | 276469.375 | 79.88% | 79.9% ✓ |
| F1 | rand | 10⁸ | 1553416 | 368640 | 464703.875 | 172032 | 388379.125 | 76.28% | 76.3% ✓ |
| F8A | seq | 1 | 803328 | 663552 | 672800 | 630784 | 642080 | 17.40% | 17.4% ✓ |
| F8A | seq | 10⁴ | 1400224 | 730692 | 774739.125 | 632376 | 682579.375 | 47.81% | 47.8% ✓ |
| F8A | seq | 10⁸ | 2326460 | 861764 | 955533.5 | 632376 | 740494.875 | 62.97% | 63.0% ✓ |

`default_ring_ok=true` 这一句也逐格核过（`write_ahead_log_full_default_ring_ok=true`、`write_ahead_log_leaf_default_ring_ok=true` 七行全部为真）。**判定：第四节那张表与产物 `e155-fsync-write-volume-2026-09-17-stage4.out` 逐格、逐百分比对得上，没有一处偏差。**

### 3.2 漏项：K9′、checkpoint 自己的屏障与 FUA、施加记录的挂载期代价、环要开多大

逐项核实是不是漏了、漏了会不会翻次序：

- **checkpoint 自己的屏障与 FUA：没有漏。** wal_full/乙的 checkpoint 成本已经算进「摊销」那一列（`write_ahead_log_full_checkpoint_bytes`/`_calls` 字段与 `jia_barriers=4`/`jia_fua=1` 同款字段并存于产物同一行），第四节表用的正是摊销列，checkpoint 自己的屏障与 FUA 已经在这个数里。**这一项不是漏项。**
- **环要开多大：没有漏，写在材料第四节正文里。** 「这些格上 WAL 两臂的在飞记录峰值都在默认环的 65 536 条之内（`default_ring_ok=true`）」——已核对逐格为真（见 3.1）；产物同时给了 `write_ahead_log_full_ring_bytes_needed` 字段。**这一项不是漏项。**
- **K9′（间隔内被换掉的中间版也写一条已释放的分配记录）：确认是真漏项，E155 自己也承认未建模。** 原句现查 `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:35`：「K9′ 未建模，产物里那一行明写『未建』」；产物印证 `research/results/e155-fsync-write-volume-2026-09-17-stage4.out` 里 `name=geometry_sensitivity_sample point=k9_prime_not_modeled` 那一行只给出当前 K9 口径下的 checkpoint 字节数、没有 K9′ 的对照值。**方向判断**：K9′ 只会给 wal_full/乙的 checkpoint 成本加钱（多写「间隔内被换掉的中间版」的已释放分配记录），不会动甲（甲每次 fsync 都立即持久，没有「间隔内被换掉」这个概念，`e7_155` 源码注释「K9′：只对 WAL 两臂 checkpoint 有意义（甲不受影响）」，`research/e7-index-bench/src/bin/e155_fsync_write_volume.rs:2120` 附近）。**会不会翻次序**：K9′ 加的钱只会让 wal_full/乙相对甲的优势变小，不会让它们变得比甲更贵到翻过临界——因为它加的是 O(间隔内重复写次数 × 一条分配记录的字节数) 这个量级（一条记录约十几字节到几十字节，`D3（空间分配） 已定项 7` 的条目宽 20 字节），而甲相对 wal_full 的差距在大 P 时是四样固定点树本身随规模长高带来的（P=10⁸ 时 1 462 708 字节），两者不是同一数量级。**判定：K9′ 是真漏项，方向已知（不利于 wal_full/乙），量级不足以翻转 3.1 那张表的次序，但没有实测数字钉死这句话，属于推论**。
- **施加记录的挂载期代价（恢复重放时现推四样固定点的开销）：确认是漏项，而且性质与前三项不同。** E155 是纯写路径的计数模型（跑前登记 5.1「不与 crates 共用代码，不量时间」），从未打算量恢复/挂载期的代价，第四节的「每次持久化写多少字节」这张表问的是运行时写路径，不是崩溃恢复路径。**这一项漏是真的，但它答的不是第四节这张表要答的问题**：把它补进来不会改变「每次 fsync 写多少字节」这个量，但会改变 wal_full 相不相容于已定条款（这正是 H3/H4 的范围，不是 H5 要判的次序）。**这里只报告漏了什么，不判它翻不翻转本表的次序——它本来就不影响本表**。

### 3.3 一处发现：E155 附录自己的「反向几何敏感性」叙述与自己的产物对不上

背景材料第四节末尾引了 E155 的限度自述，附录里逐字抄的这句（`.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:35`）：

> 「8.2 反向几何敏感性只在代表格 P=10⁵、F1、seq、Lbalanced、N=16 上跑（不是全量 1800 格扫描）：φ=0.5、g=0、K3′ 三个反向点在这一格上甲 / wal_full 的字节比值都没变（`_ratio_changes=false`），pbs=4096 让甲的比值变了（`jia_ratio_changes=true`……）」

现查产物同一行：

```
grep "geometry_sensitivity_sample" research/results/e155-fsync-write-volume-2026-09-17-stage4.out
```

```
E7RESULT name=geometry_sensitivity_sample point=phi_0.5 p=100000 base_jia=1037532 reverse_jia=1277958 base_write_ahead_log_full=303104 reverse_write_ahead_log_full=303104 base_write_ahead_log_leaf=172032 reverse_write_ahead_log_leaf=172032 jia_ratio_changes=true write_ahead_log_full_ratio_changes=false
```

**`phi_0.5` 那一行 `jia_ratio_changes=true`，不是 `false`。** 核实源码算式（`research/e7-index-bench/src/bin/e155_fsync_write_volume.rs:2115-2116`）：`jia_ratio_changes` = `(base_jia/base_wal_full − reverse_jia/reverse_wal_full).abs() > 1e-9`；代入 1037532/303104=3.4230 与 1277958/303104=4.2164，差值远超阈值，代码算出 `true` 是对的，**是 kb 的叙述句把 `phi_0.5` 错分进了「都没变」那一组，它应该和 `pbs_4096` 分在同一组（都是 `jia_ratio_changes=true`）**。`g_0`、`k3_prime` 两行现查确为 `jia_ratio_changes=false`，这两个没错。

**方向判断（不动次序）**：φ=0.5（填充率减半）下甲的绝对字节从 1037532 涨到 1277958（+23.2%），而 wal_full 的字节没变（两种落点结构在填充率减半时都还没长高一层，wal_full 的 checkpoint 净改动模型对填充率不敏感）；也就是说**这个错误的方向是让甲相对 wal_full 更贵，不是更便宜**——它不会给「重开轴一」的倾向添麻烦，反而是材料本该更强调、却写反了的一条证据。

**判定**：这不影响第四节主表（3.1，全部主几何 φ=1 的格，不受这条反向点影响），但**这是背景材料附录整段抄录的 kb 原文里一个真实的、可现查证伪的叙述错误**，应当计入「数读得对不对」的判定：第四节表本身干净，但材料引的 E155 附录里有一句自我总结与自己的产物不一致，判决交用户时应当把这一句改成「φ=0.5、pbs=4096 两个反向点甲/wal_full 比值都变了，g=0、K3′ 两个没变」。

**判定：第四节主表本身对得上产物；wal_full/乙的代价里 K9′ 与挂载期施加代价确认是漏项，方向都不利于 wal_full/乙、量级不足以翻转 3.1 那张表的次序（K9′ 部分是推论，未实测钉死）；另外发现 E155 附录自己的一句叙述（`8.2` 反向几何敏感性）与自己的产物矛盾，已用命令坐实，方向同样不利于甲（不构成对倾向的反证），需要在写回时改正措辞。**

## 四、逐条核正文第二、三、四节

### 4.1 第三节「实现今天的样子」——逐条核实

| 行 | 材料怎么说 | 现查结果 |
|---|---|---|
| 四文件哈希 | `transaction.rs`/`journal.rs`/`recovery.rs`/`write_accounting.rs` 四个哈希 | `git hash-object` 现跑，与材料记的四个哈希逐字相同（本报告开头已列） |
| `CommitStep` 五种步骤，行 66–87 | 五种步骤枚举，无 checkpoint/fsync 分开入口 | 枚举实际起于 64 行（`#[derive]`）、`pub enum` 在 66 行、闭合 87 行——**材料的「第 66–87 行」精确**；五种步骤确认（`WriteUnitToEveryDevice`/`WriteJournalRecordToEveryDevice`/`WriteRootRecordForceUnitAccess`/`RotateSuperblockSlots`/`Barrier`），无通配臂 |
| `grep -n "checkpoint"` 47 处全是字段名 | 无 T_dirty、无攒批 | 现跑 `grep -c "checkpoint" crates/singlefs-core/src/transaction.rs` = 47，逐行看命中全是 `checkpoint_txg` 字段名/注释；`grep -n "T_dirty\|Tdirty" crates/singlefs-core/src/*.rs` 0 命中——**成立** |
| `JournalRecord` 第 81–94 行 | 十二个字段 | 结构体实际 `#[derive]` 在 80 行、`pub struct` 在 81 行、闭合 94 行——**材料「第 81–94 行」精确**；字段清单逐个核对与材料相符 |
| `replay_journal` 第 821–917 行，施加=换四字段 | 前缀判定 + 换四个字段，不改树/不分配 | `fn replay_journal` 确认在 821 行；施加动作在 905–914 行，四个字段 `tree_table`/`tree_identifier_watermark`/`rollback_floor`/`mapping_root`——**材料精确** |
| 发布 B 两盘合计 21 次写调用、344 576 字节 | 与 E155 甲 P=1、F1、seq 一格逐项相同 | `crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs:364,369` 现查 `calls_and_bytes(21, 344_576)`；产物 `stage4.out:4`（阳性对照行）`jia_fsync_bytes=344576`——**逐字节相同，成立** |
| 根环 24 个槽，g=24 | K6 取 g=24，「lib.rs 185–186」 | 现查：`ROOT_RING_REGIONS=3`、`ROOT_RING_SLOTS_PER_REGION=8`（3×8=24，与「3 区×8 槽」原句相符）确实在 `crates/singlefs-format/src/lib.rs`，但精确行号是 **184–185**，不是「185–186」；且原文省了 crate 路径（K1 那一行写的是 `crates/singlefs-format/src/lib.rs 50–54, 92`，K6 只写「`lib.rs` 185–186」）。**数值与机制成立，行号有一处轻微偏差（差 1 行），已用 `grep -n` 现查坐实，不影响任何结论**（这是 `research/prompts/e155-preregistration.md` 自己的引用问题，不是背景材料转述出的新错误） |

### 4.2 第二节「前提」表——抽查代表性几行

18 行前提逐条对照出处文件，抽查其中承重最大的几条（其余各条在第一、二节判定里已逐个引用过原文，不重复列）：

| 行 | 现查结果 |
|---|---|
| 一（D23 已定项 1 全文） | `.claude/kb/decisions/23-journal的角色与格式.md:972-1115` 现查，材料附录整段抄录与源文件逐字相同（用 `grep -nF` 抽了「取甲：每次 fsync 写脏叶」「三条被降级的依据」两句，命中且行号在区间内） |
| 二（D16「攒够一批脏节点」等） | `.claude/kb/decisions/16-发布语义.md:78-104` 现查，「攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次」原句在文件 76 行，附录抄的区间 78-104 覆盖到它的后续论证，句子本身现查命中 |
| 七（重放下界·前缀判定五条 vs 六条） | ⚠️ 材料写「前缀判定五条」，现查 `.claude/kb/decisions/23-journal的角色与格式.md:793` 原句是「**前缀判定的完整口径是六条，缺一不可**」——**这是材料第二节前提表自己的一处不准确**：正文（H4/H1 都会用到这条前提）应写「六条」，附录里引的原文本身是对的（背景材料 776-819 行整段抄了六条），只是第二节索引表这一行的简称写成了「五条」。核实命令：`grep -n "前缀判定的完整口径是" .claude/kb/decisions/23-journal的角色与格式.md` → 793 行，逐字「六条，缺一不可」 |
| 十四（O2 是一元谓词；记录核对器） | `.claude/kb/decisions/13-验证路线.md:367-401` 现查，「O2（独立解析器 + checker） 的定义域是单个镜像」「任何需要第二个输入……都不属于它」两句命中、行号在区间内 |
| 十六（E16 表与多流负载） | 已在 H1、H2 全文核实 |
| 十七（E155 三条臂定义与 K1–K9） | 已在 H1、H5、五节全文核实 |
| 十八（prior-art，三家都不走甲这条路） | `research/prompts/m2-s1-prior-art-report.md` 第五节现查，「这三家没有一家在 fsync 路径上重写主结构的脏叶、全部祖先、和根」原句命中；报告自己写明「读源码，没实测，未在本项目验证」——材料转述准确，没有把「反证」升级成「正证」 |

**判定：第二节前提表整体准确，只有一处需要改正——「行七」的简称「五条」应改成「六条」，与 D23 已定项 14 原文和材料自己附录里整段抄的六条口径保持一致。**

### 4.3 第四节表——见上文 H5（3.1），逐格核对全部通过。

## 五、每条臂「定下来之后要改哪些东西」

范围按派发要求给六类：决策正文、`crates/` 文件、记录格式与 log-incompat 位、层 0 的流与状态数、checker 与记录核对器、字节表。乙不判 H6（是否还是候选），下面只列它与甲/wal_full 的差异要点，不展开完整清单。

### 甲（维持不改）

| 类 | 要改什么 |
|---|---|
| 决策正文 | 不改 D23 已定项 1/D16 已定项 1/7/8。若 H5 的「E155 附录叙述与产物矛盾」（3.3）需要写回，改 `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:35` 那一句的分组措辞；若第二节前提「五条→六条」需要写回，改 `research/prompts/_m2-s1-r1-background.md` 或它下一轮的重写版本，不改 kb 本身（kb 那句原文本来就是六条，是背景材料转述错了） |
| `crates/` 文件 | 不改 |
| 记录格式与 log-incompat 位 | 不改；D23 已定项 3 的「新记录类型 + log-incompat 位」演进通道原地待命，`grep -rn "log.incompat\|LOG_INCOMPAT" crates/` 现查 0 命中——这个位今天还没有格式实现，维持甲不需要它 |
| 层 0 流与状态数 | 不改；`crates/singlefs-harness/tests/*_layer0.rs` 现有全部按甲的语义写 |
| checker 与记录核对器 | 不改；`crates/singlefs-harness/src/crash.rs:478` 起的记录核对器两条判据在甲的持久顺序下继续恒 0（`first_transaction_step_seven_layer0.rs:89`、`second_transaction_step_zero_layer0.rs:357`、`second_transaction_step_three_formatted_pool_layer0.rs:93` 三处现查同一句断言消息） |
| 字节表 | 不改；`.claude/kb/layout/01-first-txn.md`、`.claude/kb/layout/02-second-txn.md` 与发布 B 的 344 576 字节维持现状 |

### wal_full（若定案改用它）

| 类 | 要改什么 |
|---|---|
| 决策正文 | D23 已定项 1 改写「取甲」为「取 wal_full」，第一依据（「延后祖先 ⇒ 新根不可能自洽」）**不适用于 wal_full**——wal_full 不延后祖先，只延后固定点与根，需要重新论证「固定点/根延后是否也会破坏自洽」（这与轴一/轴二的原有二分不同，是一个新的第三个维度，H2 已指出四条依据全部没有回答这个维度）；D16 连带定死的三条第 1 条要重新解释「fsync 的效果」（不再是「一次全量发布」，而是「一次不含固定点与根的部分发布」）；`.claude/rules/fs-design.md`「一个事务层，所有结构共用」要求的是「一个封闭的提交步骤枚举」，wal_full 需要在这个枚举里新增至少一种步骤（fsync 时只写单元+祖先+记录、不含固定点/根/超级块），并且 checkpoint 时的「补写固定点+根+超级块」也要在同一个枚举里表达，不能另开状态机（这条是 fs-design.md 的硬约束，任何 wal_full 的实现方案都要先满足它） |
| `crates/` 文件 | `transaction.rs`：`CommitStep` 枚举要拆细（今天的 `WriteUnitToEveryDevice`/`WriteJournalRecordToEveryDevice`/`WriteRootRecordForceUnitAccess` 三种今天在一次发布里被 `PoolWriter::perform`（121-186 行）统一驱动，要拆成「fsync 提交」与「checkpoint 提交」两条驱动路径，且要共用同一个封闭枚举）；`journal.rs`：`JournalRecord` 结构体今天只有「新根段」四个字段（`new_tree_table`/`new_mapping_root`/`new_tree_identifier_watermark`/`new_rollback_floor`），wal_full 的 fsync 记录要改成点名子树根（不是新根段），需要新的记录变体或新字段，且要有类型字段区分「fsync 记录」与「checkpoint 记录」（D23 已定项 1 的 A 条已经要求这个类型字段，今天代码里没有实现——`grep -n "record_type\|RecordType" crates/singlefs-core/src/journal.rs` 现查 0 命中）；`recovery.rs`：`replay_journal`（821-917 行）今天施加一条记录只换四个字段，wal_full 下要新增「从记录里的子树根信息现推四样固定点」这条逻辑（H2 依据②指出的挂载期新路径），这是全新代码，不是改现有函数；`write_accounting.rs`：要新增按「fsync 记录」/「checkpoint 记录」分类计写字节的口径，今天只有「一次发布」一种口径 |
| 记录格式与 log-incompat 位 | 需要新增：D23 已定项 3 已经把这条路铺好（「新记录类型 + 超级块 log-incompat 位」），但今天两者都是零实现（`grep -rn "log.incompat\|LOG_INCOMPAT" crates/` 0 命中；`grep -n "record_type" crates/singlefs-core/src/journal.rs` 0 命中）。好消息是 incompat 位图的机制性基础设施已经存在（`crates/singlefs-core/src/system_configuration.rs:18,207-213` 的 `incompat_bits_are_mountable`、`SUPPORTED_INCOMPAT_BITS`），wal_full 只需要新占一位，不需要重新发明位图读写逻辑 |
| 层 0 流与状态数 | milestone 文档自己写明「一次发布写几个单元变了，段内状态数按 2 的写数次方变」（`.claude/kb/milestone/02-second-txn.md:2010` 附近「改设计会碰到已经落地的东西：段序列登记表、层 0 两条流的状态数」）——wal_full 下 fsync 那条流的写数变少（不写固定点+根+超级块），状态数会按 2^Δ 缩小；但新增了 checkpoint 那条流（今天层 0 测试没有这个概念），需要新增一整条崩溃点枚举流，工作量不是「缩小」而是「换一种流」 |
| checker 与记录核对器 | 记录核对器（`crash.rs:478`）今天两条判据是按「甲=换四字段」写的（H2 依据③已指出），wal_full 下要新增判据处理「fsync 记录点名子树根、checkpoint 记录才点名根/固定点」这套两级语义，且要能判「checkpoint 之前崩溃、fsync 记录已持久但对应子树没有被上层任何已发布根引用」这类新状态；`crates/singlefs-checker/src/lib.rs:533` 的 `check_journal_record`（一元谓词）要新增对记录类型字段的解析与按类型分派的校验规则 |
| 字节表 | `.claude/kb/layout/01-first-txn.md`、`.claude/kb/layout/02-second-txn.md` 两份字节表要重写：今天两份表按「一次发布=一次 fsync」写死，wal_full 下要分「fsync 写的字节」与「checkpoint 写的字节」两张表；E142（第一个事务的干跑）的干跑重跑 |

### 乙（wal_leaf，仅供对照，不判 H6）

除 wal_full 上面全部要改的之外，乙额外要处理：extent/inode 两棵树的祖先也延后到 checkpoint（wal_full 只延后固定点+根，乙连祖先都延后）——这会让 `journal.rs` 的记录变体从「点名子树根」进一步收窄成「点名叶」，`recovery.rs` 的挂载期重建逻辑要多推导两棵用户树从叶到根的整条路径（不只是四样固定点），这正是 D23 已定项 1「乙重放要分配器」判决今天是否仍成立的核心（H4 的范围，这里只列出乙比 wal_full 多出的改动面，不判它站不站得住）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| H1 轴一的前提 | **不成立（打中主 agent 的推论）** | 「1 块/fsync 可忽略」字面上仍是对根槽写这一步的正确描述，但 E16 自己的模型从没算过四样固定点，E155 补上之后甲比 wal_full 每次 fsync 多付 40.6%–79.9%，这不是「可忽略」 |
| H2 四条依据 | **四条都没被推翻**，但全仓只有 `decisions/23:990` 一处斜杠括注，三条（①②③）没有独立展开的原文，只能按上下文推论转述 | ①②③按 crates 今天的样子对甲成立、对 wal_full 目前不成立（wal_full 需要的新逻辑零实现）；④字面成立但把「屏障」与「FUA」混为一谈，wal_full 省的其实是 FUA，不是屏障 |
| H5 数读得对不对 | **第四节主表逐格对得上产物；漏项方向已知、不足以翻转次序；但发现材料附录自己转述的 E155「反向几何敏感性」一句与产物矛盾** | 七行数值、七个百分点全部现算相符；K9′、挂载期施加代价确认漏，方向不利于 wal_full/乙、量级不够翻转；`phi_0.5` 那一格 `jia_ratio_changes` 实际是 `true`，材料抄的原句写成「都没变」，方向同样不利于甲 |
| 第二节前提表 | 基本准确，一处需改 | 「行七」简称写「五条」，D23 已定项 14 原文与背景材料自己附录抄的都是「六条」 |
| 第三节代码事实 | 全部核实为准确 | 四文件哈希、五种 `CommitStep`、`JournalRecord` 字段、`replay_journal` 施加逻辑、发布 B 21次/344576字节、g=24 的机制全部现查通过；仅 `e155-preregistration.md` 自己的 K6 引用行号差 1 行（184-185 而非 185-186），影响可忽略 |
| 第四节数据表 | 准确 | 见 H5 |

## 没做什么

- 不判 H3（wal_full 施加记录要不要分配器）、H4（wal_full 与已定条款接不接得上的逐条穷举）、H6（乙的地位）、H7（甲的最强形态）——按分工不归我，交给 Opus 攻方腿与本地辩方腿。
- 没有验证 K9′ 加进模型之后的具体数字，只给出方向判断（不利于 wal_full/乙，量级不够翻转次序）——这需要有人真的把 K9′ 建进 E155 的模型代码里重跑，我这一轮没有写模型、没有跑代码，按定义不越权改 `research/e7-index-bench/`。
- 没有核对第二节前提表里没在 H1/H2/H5 直接用到的其余十几行逐字与源文件比对（六、八、九、十、十一、十二、十三、十五）——它们不在我的判据范围内，抽查覆盖了承重最大的几行（前缀条数「五条/六条」那处差错就是抽查中发现的）。
- 没有验证 `research/prompts/m2-s1-forks.md`、`m2-s1-prior-art-report.md` 之外，禁读清单里 `e152-*` 与 `research/perf-by-milestone.md` 工作区改动——按禁读要求不读。
- 没有跑门禁（`.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-forward`），只跑了只读的 `grep`/`git hash-object`/`sed`/`wc -l` 类命令，没有编译、没有跑 `cargo test`。
- 「E155 附录叙述与产物矛盾」（3.3）这一条发现，只坐实了 `phi_0.5` 这一行，`g_0`/`k3_prime`/`pbs_4096` 三行已现查确认与叙述一致，不再逐一列出核对过程。

## 交付物

- 报告：`research/prompts/m2-s1-r1-sonnet-output.md`（本文件）
- 未使用模型目录（这一轮没有写模型，`research/prompts/m2-s1-r1-sonnet-model/` 未建）
- 草稿目录 `/tmp/claude-1000/m2-s1-r1-sonnet/` 本轮未用（未落草稿）
