# m4-gc-r1 云端正推（Sonnet）报告：自建 GC 的设计，G1–G9

日期 2026-10-13。立场：云端正推——从已定条款原文与 `crates/` 今天的代码推出「应该是什么」，再逐格摆出候选、代价和要交用户的岔路。读的是工作树 `/home/fy5090/.local/share/singlefs/work/m4-predev`，没读主仓；禁读清单里的文件一份没读。本报告没有改动工作树里别的文件，草稿与小脚本在 `/home/fy5090/.local/share/singlefs/work/m4-gc-r1-forward/`。

标记约定：**观测** = 我在工作树里读到、grep 到的；**推的** = 我从观测推出、没有命令能直接坐实的；**复核不了** = 要虚机、要实测或要主 agent 做的。引文一律写「原文」（`路径:行号`），行号是我在被引文件里现查的，不是背景材料里的行号。

## 〇、九格一句话

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 字段表 | 可定 | logged_ops 一条意图 = key（种类 2 + 签发 txg 8）+ 定宽 value 34，条目 44 字节，一个 16 KiB 节点装 369 条；整理与销毁快照共用这个宽度；新池字节不变。岔路：续做点存不存（顶着 D26 已定项 6 的一行）。 |
| G2 选段与计划 | 可定，背景材料漏三处 | 计划函数要多吃「钉住的豁免槽」「本次挂载开过的段」两个输入；它的前半（区域扫描）要读单元头、不是纯函数，后半才是；目的地要由分配器给候选。 |
| G3 一批的事务形态 | 可定，最大的一块在 checker | 搬一批 = 第三个发布入口；今天的代码表示不了「映射位置 ≠ 指针提示」，checker 与 GPU 判器的走读只认提示、不查映射；「目的地记录写哪个代」与 I-3.10 当面冲突。C142 在核心读路径上关得了，在 checker 上关不了。 |
| G4 停机、N₀、水位 | 三处条款自相缠住 | 按今天的口径（占用）产出谓词在最后一批那一刻恒假；N₀ 的合取 7 若连整理一起拦会死锁；盘上水位三个字段的名字与 D26 的三个量对不上。 |
| G5 discard | 乙照字面不安全 | 发 discard 必须在「回收了、没被扣住、没被隔离」之后，且在触发它的那次发布的根 FUA 之后；丙与出口 2 冲突。 |
| G6 崩溃流 | 可定 | 要先有一次重挂才有可选的段；一批的单元段写数 = 2 ×（搬的单元数 + 路径节点数），搬 4 个就碰到展开深度 30。 |
| G7 恒等变换 | 可定 | Verus 能证纯计划函数的性质与一条抽象状态引理，证不了执行层；执行层靠模型对拍与必出反例的世界。 |
| G8 调度 | 甲只给机制 | 只有会话显式触发满足不了 D3 已定项 14 的「自动」；要加「用户改动后附带一批」才算。 |
| G9 区域扫描 | 可定义 | 对一个 R 逐槽读分配记录、读单元头反查映射 key、再查中央映射，把槽分成活的可搬、钉住、已死、不一致四类。 |

## 一、方法与复核口径

- 读了：`.claude/kb/milestone/04-fourth-txn.md` 二、`.claude/kb/decisions/26-后台整理与放置回收.md` 全篇、D3 / D5 / D8 / D13 / D16 / D19 / D21 / D24 点名的分项、`.claude/kb/checks-owed.md` 里点名的欠账行、`.claude/kb/layout/03-third-txn.md` 的形态表、`research/prompts/m3-intent-r1-main-verification.md`；m3-intent-r1 两条腿的报告已归档，F1–F6 从版本库历史里取回（提交 31128273 的上一版里的 `m3-intent-r1-opus-output.md`，存在草稿目录 `m3-intent-opus-old.md`，只当线索）。
- 代码：`crates/singlefs-core/src/` 的 `transaction.rs`、`recovery.rs`、`allocator.rs`、`mount.rs`、`mounted_session.rs`、`mounted_read.rs`、`unlink.rs`、`journal.rs`、`instance_table_sweep.rs`、`block_device.rs`、`system_configuration.rs`；`crates/singlefs-checker/src/{image,walk}.rs` 与 `invariants/`；`crates/singlefs-checker-tier/` 的崩溃枚举与 WGSL；`crates/singlefs-harness/src/{lib,history,segments}.rs`；`crates/singlefs-replay-prefix/`。
- 算术（G1 的字节数、G6 的写数）用 `python3` 在草稿目录跑，输出贴在对应格里。
- 没有跑 cargo、没有起虚机；凡涉及「跑起来会怎样」的都标「推的」。
- 背景材料第二节里三处崩溃放量上限的行号和工作树对不上，已按工作树现查改正：`MAXIMUM_WRITES_PER_LOCATION` 在 `crash_judge_tables.rs:54`（材料写 :51），`MAXIMUM_ENUMERATED_WRITES` 在 `crash_facts.rs:369`（材料写 :287），`FULL_EXPAND_UP_TO=30` 在 `crash-amplification.sh:319`（材料写 :222）；材料写的「一段的状态数装进 u64（`crash.rs:1548`）」我没找到对应那句话的行，**复核不了**。其余材料里的行号（`records.rs:430`、`transaction.rs:8694` / `:4178` / `:6217` / `:6272` / `:6337`、`mount.rs:6447`、`mounted_session.rs:980`、`unlink.rs:28`、`allocator.rs` 各处、`system_configuration.rs:303` 等）我逐个对过，都对得上。

## 二、我另外现查到、背景材料第二节没写的实现事实

下面编号 O1–O11 在各格里直接引用。

### O1 映射条目的位置今天只由指针里的位置现算，「映射位置 ≠ 指针提示」这个状态表示不了

发布路径装映射条目时，数据单元的位置取自「pointer.locations,」（`crates/singlefs-core/src/transaction.rs:8476`），条目由「build_mapping_entry(key, *locations),」（`crates/singlefs-core/src/transaction.rs:8619`）写出，key 来自「mapping_key_for_data(pointer.head, pointer.write_order),」（`crates/singlefs-core/src/transaction.rs:8475`）。从盘上重建时 `mapped_units` 只存 key：「let mut mapped_units: Vec<(TransactionUnit, Vec<u8>)> = data_pointers」（`crates/singlefs-core/src/recovery.rs:2982`），对映射条目只核条数：「fn mapping_entry_count_is_one_per_mapped_unit(」（`crates/singlefs-core/src/recovery.rs:4825`）。⇒ 搬迁之后若只改映射、不改内存里的指针，下一次发布会把映射条目写回旧位置；重挂之后重建出来的指针是盘上父节点里的旧提示，同样把映射写回旧位置。搬迁要让内存指针与重建都以映射位置为准。推翻它的现象：找到一处发布路径从映射条目、而不是指针取位置。

### O2 读路径是「先读提示，读不出才查映射」

树节点：「if let Ok(bytes) = read_unit_via_locations(reader, &pointer.locations, unit_bytes) {」（`crates/singlefs-core/src/recovery.rs:456`）；数据单元：「if let Ok(bytes) = read_unit_via_locations(reader, &data_pointer.locations, data_unit_bytes) {」（`crates/singlefs-core/src/recovery.rs:508`）。D19 已定项 8 的条款是「位置提示读不出时一律经映射回退」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:183`），豁免三类（映射树节点、树表、实例表）只按提示读：映射树按父指针读，「read_unit_via_locations(reader, &pointer.locations, node_bytes)」（`crates/singlefs-core/src/mounted_read.rs:401`）。挂载态查映射是对整片读进来的条目做线性查找：「.find(|entry| entry.key == mapping_key)」（`crates/singlefs-core/src/mounted_read.rs:453`）。⇒ 搬一个被映射管着的单元，旧位置的副本在被回收复用之前仍能读对（字节一样），复用之后提示读出的是别人的字节、整单元校验和对不上，转去查映射。

### O3 checker 与 GPU 判器的走读只认提示，不查映射回退

「/// 按位置条目读一个被引用的单元：每条位置条目都读、都判 I-2.1（校验和与内容匹配），返回第一份对得上的。」（`crates/singlefs-checker/src/image.rs:1556`）；走读跟指针往下时「中央映射条目不走这里」（`crates/singlefs-checker/src/walk.rs:1404`），映射条目另走 `walk_central_mapping_entries`，只核头相符：「映射是解引用与释放判定的唯一入口」（`crates/singlefs-checker/src/walk.rs:1893`）。GPU 判器有逐句对应的一份：「walk_central_mapping_entries：I-2.5、I-1.11、I-2.1」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2632`）。我在 `crates/singlefs-checker/src/walk.rs` 里 grep「提示读不出」「经映射回退」「stale」「映射回退」四个词，零命中。⇒ 搬迁之后只要父节点没重写、旧槽被复用，合法镜像在今天的 checker 上判 I-2.1 红。

### O4 发布管线是「角色」制，没有「原样搬一个单元」的角色

`TransactionUnit` 是封闭枚举（「pub enum TransactionUnit {」`crates/singlefs-core/src/transaction.rs:2270`），一次发布的重写集由 `resolve` 现算；文件内容只有 `plan.file` 一条路，且「文件内容重写恒从第 0 个单元写起」（`crates/singlefs-core/src/transaction.rs:3371`）。「重写」的语义是新内容、新出生序号、新映射 key、父节点跟着重写；搬迁要的是同内容、同 key、新位置、父节点不动。照 `DeletedInodesRoot` 的足迹估新加一个角色的工作量：`grep -rc DeletedInodesRoot --include=*.rs crates` 的非零行共 12 个文件（`transaction.rs` 37 处，`model.rs` 7 处，`walk.rs` 4 处，其余 9 个文件各 1–3 处）。

### O5 空段计数只在回收那一步才涨

`used_per_segment` 只在 `mark_reclaimed` 里减：「self.used_per_segment[segment] -= 1;」（`crates/singlefs-core/src/allocator.rs:724`），`empty_segments` 数的是它为 0 的段：「pub fn empty_segments(&self) -> u64 {」（`crates/singlefs-core/src/allocator.rs:736`）。回收的门槛是「.filter(|record| record.is_released && record.generation <= floor)」（`crates/singlefs-core/src/allocator.rs:2171`），即 D16 的「可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」（`.claude/kb/decisions/16-发布语义.md:33`）。搬走的源槽进 defer、不算空。一条根算不算「非空」看 inode 树与 extent 树的根指针变没变（「一条有效根算非空」`.claude/kb/decisions/16-发布语义.md:46`）；纯映射搬迁不动这两棵树的根指针（推的，依据是 O1：这样的发布只改映射、分配记录、记账、树表），所以 GC 发布不推进「最近 4 个不同状态」的窗口，源槽要等根环转一圈（默认每区 8 槽 × 3 区 = 24 条根：「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」`crates/singlefs-format/src/lib.rs:291`，「pub const ROOT_RING_REGIONS: u64 = 3;」`crates/singlefs-format/src/lib.rs:273`）或用户改动把 F 抬过去才回收。

### O6 落点政策里没有「目的地排除区」

「不在 `R` 保护期内（第一版 R 为空）」（`crates/singlefs-core/src/allocator.rs:4`）只是文件头注释；用户数据落点函数只排开放 / 本次挂载开过的段：「pub fn lowest_user_data_slot(」（`crates/singlefs-core/src/allocator.rs:785`）。D3 已定项 7 登记表第 5 条要的「整理意图期内那批槽不许当分配目的地」（`.claude/kb/decisions/03-空间分配.md:144`）没有代码。

### O7 discard 的底座有三处现成的先例

块设备 trait 的文档是「六个动作，只有这六个。」（`crates/singlefs-core/src/block_device.rs:114`），`grep -rn 'impl.*BlockDevice for' --include=*.rs crates | wc -l` 输出 35。录制流上一次加新步骤（整段清零）的足迹：`grep -rln WriteZeroes --include=*.rs crates | wc -l` 输出 20，入口「WriteZeroes,」在 `crates/singlefs-harness/src/lib.rs:40`。绕开提交步骤直接写盘的先例是实例表清扫：「if let Err(error) = device.write_at(」（`crates/singlefs-core/src/instance_table_sweep.rs:603`）、「if let Err(error) = device.barrier() {」（`crates/singlefs-core/src/instance_table_sweep.rs:617`）。提交步骤是封闭枚举：「提交步骤的封闭枚举：五种步骤种类」（`crates/singlefs-core/src/transaction.rs:151`）。设备日志能解出 Discard，构造点只有「events.push(DeviceEvent::Discard { offset, length });」（`crates/singlefs-checker-tier/src/device_log.rs:121`），消费它的地方把它和 Flush、Mark 一起丢：「DeviceEvent::Flush | DeviceEvent::Discard { .. } | DeviceEvent::Mark => None,」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs:550`，同文件 :676、:840 各一处）。

### O8 logged_ops 今天是空壳

种类码在「pub const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;」（`crates/singlefs-core/src/records.rs:430`）；写树表时硬写空根：「TREE_KIND_LOGGED_OPERATIONS,」（`crates/singlefs-core/src/transaction.rs:8727`）后面接 `NodePointer::empty_root()`；读侧 key 宽返回 None：「| TREE_KIND_LOGGED_OPERATIONS => None,」（`crates/singlefs-core/src/recovery.rs:3848`）；checker 对 I-4.4 恒报不适用：「pub const INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION: &str」（`crates/singlefs-checker/src/image.rs:279`）。

### O9 现有最长的崩溃枚举段已经压在展开深度上

「今天是 30 写：多记录流与删除流」（`scripts/crash/crash-amplification.sh:50`）；展开深度「FULL_EXPAND_UP_TO=30」（`scripts/crash/crash-amplification.sh:319`）；删除流的枚举域「崩溃点重放的枚举域 1463812278 个状态」（`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:22`），同一段文档注释写着意图那次发布 13 个单元、两批 12 与 11 个、最后一批 12 个。

### O10 没有快照读路径

`grep -rln snapshot crates/singlefs-core/src` 只命中 `records.rs`（树表条目的 `previous_snapshot_txg` 字段与注释）；`grep -rn 'fn .*snapshot' crates/singlefs-core/src` 零命中。⇒ G3 里「快照读路径是否都经映射」今天无对象，只能对恢复、挂载态、checker 三套读者回答。

### O11 GC 的每次发布都占根环的一个槽，会把用户状态挤出回退候选集

根槽是按 txg 轮着写的：「根槽 FUA 写：落区域 `txg mod 3` 的槽 `(txg div 3) mod 8`」（`crates/singlefs-core/src/transaction.rs:164`，8 是 mkfs 默认每区槽数），所以每次发布盖掉 3S 次发布之前那条根（默认 24）。D16 已定项 1 自己承认这一点：「根环容量是这句的边界」，「任何机制都给不出环里没有的状态」（`.claude/kb/decisions/16-发布语义.md:29`）。GC 发布不改变用户可见状态（O5 的推论），连发 3S 次之后，环里全是同一个用户状态，「最近 4 个可退到的不同状态」塌成 1 个。⇒ 一次整理运行的发布数要有上限，上限 ≤ 3S 减去要保住的不同状态各自最后一条根之后的发布数；这是推的，没跑。它同时是源槽回收的节奏（O5）：靠环转一圈回收，等于用掉了用户的回退深度。

## 三、G1 logged_ops 意图记录的字段表（C364）

### 判定

可定。字段表写得出，也不动新池新建文件的字节（树表仍 12 条，logged_ops 仍零根，见 O8）。三处要主 agent 或用户知情：续做点存不存、活意图的上界、整理与销毁快照共不共用一棵树。

### 条款给的硬边界（我读到的）

- 意图是树里一条普通 key 的 value：「零格式位——不新增任何盘上字段，`c0` 住意图记录」（`.claude/kb/decisions/26-后台整理与放置回收.md:50`）。
- 码 2 树的头和条目区：「含 29 字节预留位后头 = `115 + 2 × key 宽`」（`.claude/kb/decisions/08-核心索引结构.md:343`）；条目宽是节点头里一个 u16，不能一棵树两种宽：「条目宽字段只有一个 u16，按类两宽时这两句顶不住」（`.claude/kb/decisions/08-核心索引结构.md:358`）。
- 读者遇到非零的未用位一律拒：树表条目 flags 的政策是「未知位一律拒收——该条目 EIO」（`.claude/kb/decisions/08-核心索引结构.md:260`），本表的补零照同一政策办（类推，条款没点名 logged_ops）。
- 活意图数有界：「活意图数有界（整理一次推进一个批次）」（`.claude/kb/decisions/26-后台整理与放置回收.md:27`），上界是几条条款没写（m3-intent-r1 攻方 F5）。
- 进度住意图记录：「整理进度 | 意图记录 | 它就是 D8」（`.claude/kb/decisions/26-后台整理与放置回收.md:127`）。
- 销毁快照要装的是 D5 已定项 12 写的「写意图 (头, lo = S.txg, hi, P = prev(S).txg)」（`.claude/kb/decisions/05-快照-空间记账机制.md:277`），那一行没有续做点：条目被放掉就是进度，施加前核条目还在（D8 已定项 5 第 4 条）。

### 候选

候选 A（单树、定宽联合条目）：
- key 10 字节 = 种类 2 + 签发 txg 8，按字段逐段无符号比较（D8 已定项 11），于是先按种类、再按签发先后排，续做时最老的先走。
- value 34 字节，种类相关，余下补零：整理（种类 1）= R 的设备 4 + 起始槽 6 + 槽数 2、`c0` 8、`segs0` 8、续做点 6；销毁快照（种类 2）= 头 ID 8 + lo 8 + hi 8 + P 8，后补零 2。
- 定义句（C364 第一轮指出缺的）：`c0` = 签发那次发布的 checkpoint_txg；`segs0` = 签发那次发布写下的记账第 11 项里 R 所在盘的那一行的值（以值写，不去查别的代）。
- 条目 44 字节，头 135，一个节点装 369 条。命令与原样输出：`python3 -I arith.py`（草稿目录，输出存 `arith.out`）：「logged_ops key 10 value 34 entry 44 header/entries_per_node (135, 369)」。
- 一次发布至多签发一条意图（否则 key 里要加序号 2 字节，条目变 46）。

候选 B（两棵树）：整理一棵、销毁快照一棵。新增种类码 14（D8 已定项 9 只追加不重用码）、树表 12 → 13 条，新池字节变（树表声明长度、水位、树 ID 全动），换来每棵树的条目宽各自合身。没有任何一格需要它：A 浪费的只是销毁快照条目里的 2 字节补零。

候选 C（变宽条目）：与上面第二条硬边界直接冲突，出局。

### 五个小分叉（各自独立，对 A 的宽度都不起作用，除标明的）

| 分叉 | 候选 | 我的读法 |
|---|---|---|
| 续做点 | a1 存（6 字节，当提示用，删意图前对整个 R 全量核目标态、不过就回到 R 起点——C364 第二轮的写法，转述自 `.claude/kb/checks-owed.md:277`）；a2 不存（每批从 R 起点重扫，64 槽的记录查找在内存里，目标态本身就是进度） | a1 照字面兑现 D26 已定项 6 的那一行；a2 每批少改一次 logged_ops 根（见下面「新写出的字节」），而且销毁快照那一种本来就没有续做点。推的：a2 在 R = 一个聚簇段时没有损失。 |
| 暂停标志 | b1 不存（暂停 = 意图留着不推进；停机 = 删条目）；b2 存 1 字节 | m3-intent-r1 攻方 F1 的反例成立于 b2：要写进去得发一次布，而暂停的常见起因恰是分配给不出落点。b1 下 F1 不成立。 |
| R 带不带设备 | c1 带（设备 4）；c2 不带（全池同号） | 第一版两盘同槽：「`Placement` 两盘同槽，各盘不同槽第一版不支持」（`crates/singlefs-core/src/allocator.rs:9`），c1 与 c2 逐字节等价；C364 第二轮推荐按设备（转述自 `.claude/kb/checks-owed.md:277`），ZNS 的口子也要它。 |
| `segs0` 口径 | d1 占用口径（记账第 11 项，今天 `empty_segments()`，defer 里的槽算占用）；d2 记录口径（D3 已定项 10 ① 的字面） | 定义句写成「记账第 11 项的值」时 d1 是唯一有现成维护量的；d2 要新加每段的 defer 计数，见 G4。 |
| 活意图上界 | e1 整理全池至多 1 条；e2 每盘 1 条 | 第一版串行提交、一次推进一个批次（D13 已定项 2「先串行提交」），e1 够用，R 排他检查是 O(1)。销毁快照的条数由快照数定，不受这个界管。 |

### m3-intent-r1 攻方 F1–F6 在候选 A 上逐条重核

| 编号 | 原反例 | 在 A 上 | 理由 |
|---|---|---|---|
| F1 | 暂停标志写不进去 | 不成立（取 b1） | 没有这个字段 |
| F2 | `segs0` 同名的量有三种口径 | 对字段表不成立，对条款仍成立 | 定义句把 `segs0` 钉在记账第 11 项的值上；但 D3 已定项 10 ① 与 D26 已定项 1 ① 的字面冲突还在 kb 里（见 G4 的并排抄句） |
| F3 | R 写成（设备，段号），段长不在盘上 | 不成立 | R = 起始槽 + 槽数，不依赖 `CLUSTER_SEGMENT_SLOTS` |
| F4 | 续做点只越过已核实做完的 | a1 下靠「全量核、不过回到起点」挡住；a2 下不适用 | 回退把已搬的槽复活成分配代 ≤ `c0` 时，全量核会发现目标态没到 |
| F5 | 活意图数不写上界 | 取 e1 后成立的是「1」 | 销毁快照另算 |
| F6 | 续做点、`c0` 按增量写 | 不成立 | 三个字段都是值或位置，不是「再搬 N 个」；D8 已定项 5 的「意图声明目标状态，绝不声明增量。」（`.claude/kb/decisions/08-核心索引结构.md:122`）可按字段白名单判 |

### 代价：要动的 crates 文件、新写出的字节

| 项 | A（推荐形态） | B（两棵树） |
|---|---|---|
| 字段表常量与读写 | `crates/singlefs-format/src/lib.rs` 加 key 宽 10、条目宽 44 的常量（format-const 登记）；`crates/singlefs-core/src/` 新模块（形态照 `lineage_tree.rs` 216 行、`tombstone_tree.rs` 316 行：只算、不发写） | 同左，再加一份 |
| 树的角色与发布 | `crates/singlefs-core/src/transaction.rs` 新角色 `LoggedOperationsRoot`，树表条目的根指针从 `NodePointer::empty_root()`（:8727）改成取这一版的根；足迹照 `DeletedInodesRoot`（O4：12 个文件） | 两个角色 |
| 读侧 | `crates/singlefs-core/src/recovery.rs:3848` 的 `key_width_for_kind` 返回 `Some(10)`，重建时读这棵树；同文件 :6303 附近那条断言 logged_ops 为 None 的单测要改 | 两处 |
| 挂载 | `crates/singlefs-core/src/mount.rs` 挂载时读出活意图，交分配器建 R 排他表（G2 / G3 用） | 同 |
| checker | `crates/singlefs-checker/src/walk.rs` 实现 I-4.4（今天恒不适用，`image.rs:279` 的理由串要删）：字段白名单 + 补零 + R 落在单元区内 + `c0` 不晚于该版 txg；GPU 判器下标 73 的内核要补（`invariants/README.md` 写「内核不置位」） | 两份 |
| 文档 | `.claude/kb/layout/04-` 开头那份加字节表；`.claude/kb/decisions/08-核心索引结构.md` 已定项 5 / 8 / 9 / 11 改 C364 与 logged_ops 的字；`invariants/06-事务与崩溃一致性.md` I-4.4 一行 | 再加种类码 14 |
| 新池字节 | 否 | 是（树表 13 条、水位 65550、声明长度 2600） |
| 运行时新写出的字节 | 签发一条意图：logged_ops 根兼叶 1 个节点 16384 字节 × 2 盘，条目 44 字节在里面。a1 下每批再改一次条目，再写一份根节点（16384 × 2）；a2 下每批 0。收掉（最后一批同发布）再写一份。映射条目 55 字节、分配记录 20 字节 × 2 盘随节点一起变。 | 同，种类各自一份 |

### 推荐与依据

推荐 A + b1 + c1 + d1 + e1，续做点 a1 / a2 交用户（见岔路）。依据：A 的全部字节都落在 D8 已定项 11 的现成形态里、不动新池字节；F1–F6 在它上面除 F2 的条款冲突外都不成立。

### 会推翻它的现象

- 发现一次发布必须同时签发两条意图（key 冲突）⇒ key 加序号。
- D8 已定项 11 的头公式在 key 宽 10 上算错（我用 `115 + 2 × 10 = 135` 算的，没有读 `code_two_tree.rs` 核）⇒ 条数变，其余不变。
- 用户把续做点读成「必须存」且批大小要大到单元段写数越过 30（G6）⇒ a1 的每批 +2 写要进估算。

### 要交用户的岔路（不带倾向）

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| a1 续做点存 / a2 不存 | 一批多写的单元数与单元段写数（a1 每批 +1 个节点 × 2 盘 = +2 写）；a2 下 R 超过一个聚簇段时重扫的记录查找次数 | G6 的流上批大小已定，量出 a1 的最长单元段写数是否越过 30；用户对 D26 已定项 6 那一行「整理进度 \| 意图记录」读成字面还是读成可派生 |
| 联合条目 A / 两棵树 B | B 多出的新池字节与 checker 项数 | A 的 2 字节补零被证明装不下第三种意图（出现第三种使用者）时才有理由换 |
| 活意图上界 e1 / e2 | 是否出现两块盘各需一条意图的历史（异构盘，D2 已定项 2 的那一半） | 第一版固定两盘同槽，e1 成立；放开各盘不同槽时重判 |

## 四、G2 选段与搬迁计划：纯函数、输入、输出

### 判定

可定。要改正背景材料三处：

1. 计划函数不能只吃「分配记录、中央映射、记账、水位」。有三类槽搬不得或选不得：映射树的节点、树表单元、实例表各片这三类豁免映射，它们的权威位置在根记录和父指针里（「映射树自己的节点、树表单元、实例表单元」`.claude/kb/decisions/19-块指针的结构与宽度预算.md:183`；代码里是「TransactionUnit::MappingTree => previous.root.mapping_root.locations,」`crates/singlefs-core/src/transaction.rs:3315` 与「TransactionUnit::TreeTable => previous.root.tree_table.locations,」`crates/singlefs-core/src/transaction.rs:3316` 这种直接取根记录里的位置）。纯映射搬迁碰不了它们，所以要给计划函数一个「钉住的槽」输入，段里有一个钉住的槽就不能当 R。
2. 「本次挂载开过的聚簇段」不能当 R：开放段还在 bump，R 排他与提交内生块的 bump 会当面冲突；D3 已定项 8 第 2 条把这些段对用户数据关着，重挂之后才开放（「混放允许」`.claude/kb/decisions/03-空间分配.md:165`）。分配器里这个集合就是 `cluster_segments`（`crates/singlefs-core/src/allocator.rs:1576`）。后果在 G6：崩溃流里要先重挂一次，才有段可选。
3. 「输出里带目的落点」只有在目的地候选由分配器给出时才是纯的：哪些槽此刻能发（没被分配、没被影子账隔离、没被抬 F 扣住）是分配器内存里的三张位图，不是已发布状态推得出的（`isolated: Vec<bool>,` `crates/singlefs-core/src/allocator.rs:270`，`held_until_floor_takes_effect: Vec<bool>,` `crates/singlefs-core/src/allocator.rs:278`）。

### 分层（推荐）

三个函数，只有第一个碰设备：

- `scan_region`（G9 定义）：读 R 里每个已分配槽的单元头，反查映射 key，分类。有 I/O，所以不叫「纯函数」，但输出只依赖盘上字节。
- `choose_region`：纯函数。输入 = 全部分配记录（按设备、槽升序，`TransactionOutput.allocation_records`，`crates/singlefs-core/src/transaction.rs:2854`）、单元区起点与槽数、`cluster_segments`、活整理意图的 R、钉住的槽、判据与 `c0`；输出 = `Option<Region>`，Region = 设备 + 起始槽 + 槽数（64 的整数倍对齐是选段政策、不是格式）。
- `plan_batch`：纯函数。输入 = `scan_region` 的结果、分配器给的目的地候选（已滤掉 R、隔离、扣住，按槽升序，数据单元要偶数起点成对空槽）、批上限 K；输出 = 搬迁清单（映射 key、源落点、目的落点，目的落点从候选里按次序取、互不重叠、跨度与源相同）。

模块：`crates/singlefs-core/src/compaction_plan.rs`（形态照 `unlink.rs`：「一样只算、不发一个写、不动分配器、不碰落点」`crates/singlefs-core/src/unlink.rs:3`）与 `region_scan.rs`。GPU 的口子 = 这两个函数不碰分配器和设备写（D24 已定项 2 第 3 条：「GPU 路径与 CPU 路径必须是两份实现」`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:34`，现在只有 CPU 这一份，不写占位代码）。

### 候选

选段判据（`choose_region` 里的打分，资格相同：不在 `cluster_segments`、不在活 R 里、没有钉住的槽、至少一个活槽）：
- S-a greedy：活槽（未释放记录罩住的槽数）最少者先。只用记录。
- S-b cost-benefit（F2FS 的做法，线索，未在本项目验证）：`(64 − 活槽) × 年龄 ÷ (64 + 活槽)`，年龄 = 当前 txg − 段内活记录分配代的最大值。年龄只用已发布记录里的分配代，取得到；这要求搬迁后目的地记录的分配代保持单元的诞生代（见 G3 的 I-3.10 一格），否则搬过的冷数据会显得年轻。
- S-c 轮转：按地址从低到高巡回，每次取下一个有资格的段（D27 已定项 6 的「连续巡回」是打包用的，这里只当对照）。

目的地政策（`plan_batch` 的候选怎么给）：
- D-a 码 1 数据单元走用户数据落点函数（最低偶数槽成对空槽、不在 `cluster_segments`，「pub fn try_allocate_user_data(」`crates/singlefs-core/src/allocator.rs:1950`），码 2 / 码 3 节点单元走开放段 bump（「pub fn try_allocate_commit_generated(」`crates/singlefs-core/src/allocator.rs:2004`），都加「不在活 R 里」。
- D-b 两类都走用户数据那条（节点 1 槽、容器 2 槽），不碰开放段。理由：开放段里装的是寿命一个 checkpoint 的提交内生块（D3 已定项 8 依据里 E151 的 0 是下界），冷节点混进去，段永远凑不齐全空。
- D-c 另开一个只给 GC 用的冷目的段（第二个开放段），冷数据放一起。代价：分配器多一个段状态、「全空聚簇段数」与「可开聚簇段」两个量要重新对账（`lowest_empty_segment` 按隔离位与扣住位找段起点，`crates/singlefs-core/src/allocator.rs:809`）。

### 与 D3 已定项 8 的对得上

- 「R 内不许分配」：`plan_batch` 的候选滤掉 R；分配器侧 `lowest_user_data_slot` 与开放段 bump 都加一个「排除区间」参数，由挂载时读出的活意图建（O6 说这一条今天没有代码）。
- 「开放段不许当目的地」只对码 1 成立（D3 已定项 8 第 1 条：「第一版用户数据块不进聚簇段」`.claude/kb/decisions/03-空间分配.md:164`）；对节点单元，D3 已定项 5 要的是提交自身产生的块从聚簇段 bump，搬来的冷节点不是「自身产生」的块，条款没写它去哪，就是上面 D-a / D-b / D-c 这一问。

### 单测怎么钉绝对值（形态照 E93：手算锚点 + 守恒 + 阳性对照）

- 锚点 1：三个段活槽数 [60, 10, 0]，S-a 选第二个；全空的第三个不选（没有活槽）。
- 锚点 2：第二个段里放一个钉住的槽 ⇒ 改选第一个。
- 锚点 3：第二个段在 `cluster_segments` 里 ⇒ 改选第一个；全部有资格的段都被排除 ⇒ `None`。
- 锚点 4：K = 3、源是三个 2 槽的数据单元 ⇒ 目的地三对偶数槽、互不重叠、不在 R 内；候选不够 ⇒ 少搬、不报错。
- 守恒：各批搬迁清单的并集 = R 内活映射 key 集合，批间不交。
- 变异（每条必红）：去掉钉住检查；去掉 `cluster_segments` 排除；目的地允许落在 R 内；打分比较方向反过来；K 差一；数据单元目的地不对齐偶数槽。

### 代价

| 项 | 要动的文件 | 新写出的字节 |
|---|---|---|
| 两个新模块与单测 | `crates/singlefs-core/src/compaction_plan.rs`、`region_scan.rs`、`lib.rs` 登记 | 0 |
| 分配器排除区间与目的地候选 | `crates/singlefs-core/src/allocator.rs`：给 `lowest_user_data_slot`（:785）与开放段 bump（:2068）加参数，加 `free_destination_candidates`；记录表是 `Vec` 线性查找（「&& record.slot == placement.slot)」`crates/singlefs-core/src/allocator.rs:1871`），一批 K 次释放就是 K 次线性扫，记录数随盘容量走，这是今天就有的上限，GC 把它放大 K 倍（推的，没量） | 0 |
| 钉住的槽的来源 | 现行版本 `TransactionOutput` 的根记录指针（树表、映射根、实例表链）与映射树节点指针，读现成字段 | 0 |

### 推荐与依据

推荐 S-a + D-a。依据：S-a 只用记录，没有年龄就没有依赖 G3 的分配代取法；D-a 沿用两条现成的落点函数、只加排除区间，改动最小。S-b 与 D-b / D-c 留给 E10 的臂（见岔路）。

### 会推翻它的现象

- 实现里存在不经映射的位置来源，使「钉住」的集合比三类大（我只核了发布、重建、挂载读、checker 四条路径）。
- 一个段里的活单元几乎全是豁免类（映射树节点、实例表）⇒ 几乎没有段有资格，GC 起不来（推的：实例表每次写行重写，映射树的节点随发布 COW，长寿的豁免单元不多）。

### 要交用户的岔路（不带倾向）

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 选段 S-a / S-b / S-c | 同一写预算下 runs 轨迹、全空聚簇段数轨迹（E93 的 bump_compact 臂在真实现上） | E10 长跑里三臂 × 5 种子的轨迹；老化负载过 E93 判据 4（关整理时 runs 终值 ≥ L/4） |
| 节点单元目的地 D-a / D-b / D-c | 目的段里「提交内生块」与「搬来的冷节点」的混放比例；全空聚簇段数是否仍涨 | 同上，且分配器能区分两类落点 |

## 五、G3 一批搬迁的事务形态

### 判定

形态可定，但这一格比背景材料估的重，重在三处：（1）发布管线今天表示不了「同内容、同 key、新位置、父节点不动」（O1、O4）；（2）checker 与 GPU 判器的走读不会读过期提示（O3）；（3）有三处条款要先定：目的地记录写哪个代、意图怎么收（G4）、GC 发布算不算改变用户可见状态（O5）。

### 一批的结构（推荐形态 P-1）

一次发布 P_b 写下面这些，别的照抄：

1. 搬迁副本：K 个单元原样字节写到目的地，每盘同槽（沿用 `CommitStep::WriteUnitToEveryDevice`，「Placement 两盘同槽」`crates/singlefs-core/src/allocator.rs:9` 的注释说第一版各盘不同槽不支持）。单元头不重新盖戳，写序、出生代、出生序号都是原来的。
2. 中央映射：这 K 个 key 的条目换位置，key 不变；映射树 COW 含这些 key 的叶和到根的路径。
3. 分配记录：每个源落点每盘改写成已释放，释放代 = P_b 的 txg（「pub fn release_leaving_the_record_allocated_on(」`crates/singlefs-core/src/allocator.rs:1858`），槽进 defer；每个目的地落点每盘记一条已分配。
4. 记账树整棵本代重写（D5 已定项 2：每条根的记账树只装这条根那一代的行）；树表单元重写（每次发布都写）。
5. logged_ops：签发批加条目；最后一批或提前收时删条目；a1 下每批改续做点。
6. journal：⌈(K + 新写节点数) ÷ 64⌉ 条记录（每条 64 个点名项，超出走「末条再跨记录」`crates/singlefs-core/src/transaction.rs:178`），点名项 = 搬迁副本（新位置 + 原 key）与新写的节点；一个事务号就够，因为搬迁副本的 key 不含这次的事务号（数据单元「一事务一单元」那条切分纪律管的是新写的数据单元）。恢复验点名单元只是逐份读、比整单元 CRC：「pub(crate) fn every_named_unit_of_the_record_verifies<Reader: PoolReader + ?Sized>(」`crates/singlefs-core/src/recovery.rs:3606`，搬迁副本过得了。
7. 根记录 FUA 写、系统配置轮换：与别的发布相同。

不动：被搬单元的内容、`birth`（D5 已定项 15：「不改。后台整理把一个单元从一个落点搬到另一个落点」`.claude/kb/decisions/05-快照-空间记账机制.md:344`）、deadlist（「deadlist 条目按版本身份记」`.claude/kb/decisions/26-后台整理与放置回收.md:104`）、父节点、稀疏旁表。

### 候选：发布形态

- P-1 第三个发布入口 `publish_relocation_batch`，与 `publish_unlink_step` 并列；额外参数照删除那一步的办法单独交给 `prepare_the_version_publish`，不并进 `PublishPlan`：「不并进 [`PublishPlan`]：checker 档几份装置按字段逐个构造 `PublishPlan`，加字段就要动它们」（`crates/singlefs-core/src/transaction.rs:4175`）。`PublishPlan {` 在 crates 里构造 49 处（`grep -rn 'PublishPlan {' --include=*.rs crates | wc -l`），`prepare_the_version_publish(` 6 处。
- P-2 当文件版本的一种（`plan.file`）：出局。文件路径恒从第 0 个单元写起、换出生序号和映射 key（O4）。
- P-3 强制重写（搬 = 同内容重写 + 全祖先链 COW）：对独占的节点单元能走，对被快照共享的块要换 birth，违 D5 已定项 15；E94 量过这条路的代价形态：指针权威臂「根引用重写恒 = K + 1」（`.claude/kb/experiments/94-搬共享落点的触碰面.md:56`）。只当豁免类（映射树节点）的后备。

### 候选：内存里谁是位置的真相（O1 的出路）

- L-1 搬迁后同时改 `TransactionOutput` 里该单元指针的位置，重建时用映射条目的位置覆盖盘上父节点里的提示。好处：父节点下次被重写时写的就是新位置（C417 的「随重写刷新」自然成立）。代价：内存里的指针与盘上父节点字节不再逐字相同，凡是拿重建结果和发布结果逐字段比的用例（harness 里很多）要放宽到「位置字段以映射为准」。
- L-2 `TransactionOutput` 另存「key → 映射位置」，指针保持盘上提示；装映射条目时它优先；重建时由映射条目填。内存与盘上父节点一致，对拍不用改；代价：父节点重写时要明确选写提示还是映射位置，选写提示则提示永远旧（C417 的另一种政策），选写映射位置则父节点内容随之变（它本来在被重写，不额外）。
- 两者都要动：`crates/singlefs-core/src/transaction.rs` 装映射条目那一段（:8466 起到 :8622）、`crates/singlefs-core/src/recovery.rs` 重建 `mapped_units` 那一段（:2982 起）。

### 冲突：目的地那条分配记录写哪个代（并排抄句）

- I-3.10：「记录与单元是同一次发布写出来的，两个数因此同源」（`crates/singlefs-checker/invariants/05-空间记账.md:17`）。同一行的判法：未释放的记录，分配代等于它罩住的单元头里的诞生代号。
- D5 已定项 15：「不改。后台整理把一个单元从一个落点搬到另一个落点」（`.claude/kb/decisions/05-快照-空间记账机制.md:344`）；D26 已定项 5：「搬迁不改 `birth`」（`.claude/kb/decisions/26-后台整理与放置回收.md:102`）。搬迁副本是 P_b 写的，记录也是 P_b 写的，但单元头里的诞生代号是原来的。

两句不能同时按字面成立：目的地记录的分配代若写 P_b 的 txg，I-3.10 在每个搬过的单元上判红。候选：

- G-a 目的地记录的分配代写单元头的诞生代号，I-3.10 一个字不改。分配代的意思由「分配出去那次发布的 txg」变成「单元的诞生代」。对 D26 已定项 2 的目标态没有影响（目的地不在 R 内，R 排他），对 S-b 的年龄反而对（冷数据搬过之后仍然老）。要改的条款：D3 已定项 7 里「分配代」的口径一句、D26 已定项 5 的「目的地侧一个式子」。
- G-b 目的地记录写 P_b 的 txg，I-3.10 改成「分配代 ≥ 诞生代号」。I-3.10 那一行自己写着它拦的是「复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着」，改成 ≥ 之后这个变异判不出。

### C142 与「读路径是否只有一个位置来源」

- 恢复、挂载态读、从盘上重建上一版：都是先提示、读不出才查映射（O2）。搬迁后源槽未复用时，旧提示读出的是同一份字节，且位置条目里的 CRC 对得上；复用之后读出别人的字节，CRC 对不上，转映射，读对。所以对这三套读者，C142 的七步里「第二个引用者的指针没改到」这一步不再造成错读，只造成多一跳（挂载态一次解引用 3 次设备读，「提示过期的一次解引用 = 两条提示各试一次 + 映射落点一次 = 3」`crates/singlefs-core/src/mounted_read.rs:392`）。残留风险是 32 位 CRC 在复用槽上碰撞，与 D13 已定项 4 的撕裂粒度同一条知情接受。
- 例外一：豁免三类不搬，不在此列（它们的父指针是权威，O2）。
- 例外二：checker 与 GPU 判器只认提示（O3），C142 在这一侧关不了，要先加规则 N1（下面）。
- 例外三：今天没有快照读路径（O10），快照落地后要再核一遍它走的是不是这同一条。
- 结论：C142 的核心读路径部分可关，但关它的是 D19 已定项 5 与 8 取混合位置权威那一步，不是 GC；GC 要做的是把「提示可能过期」从假设变成会被测试强制进入的路径（G6 的开关）。

### 每个中间态为什么成立

| 崩在哪 | 恢复落到 | I-7.4 / I-3.1 / I-4.6 |
|---|---|---|
| 搬迁副本与新节点只持久一部分，记录没持久 | 上一个根 | 目的地槽在上一版账里是空闲的（分配器只发没分配、没隔离、没扣住的槽），没有候选根引用它们，I-7.4 成立。副本是孤儿，但头里的写序是已发布的实例，实例表清扫只抹按实例表判未发布的头（`crates/singlefs-core/src/instance_table_sweep.rs:1` 的模块文档），不会碰它。 |
| 记录持久、根没持久 | 按 D23 已定项 14 施加该记录的新根段 | 点名单元逐份验过才施加，副本过 CRC；老根仍引用源槽，源在 defer，I-7.4 成立；I-3.1 取根环里全部有效根的引用并集，新根引用目的地、老根引用源，并集 = 账。 |
| 根持久 | 新版本 | 同上；源要等回收门槛才可复用。 |
| 任何一处 | 文件结构没动 | I-4.6 只判 nlink = 0 的记录的单元数对 size（「中间态特有的一格」`crates/singlefs-checker/invariants/06-事务与崩溃一致性.md:12` 同行），搬迁不改这两样，成立。I-4.4 见 G1。 |

两处是推的、没跑：副本孤儿存在时，扫描重建（里程碑四四·1）会看到同 key 的两份相同字节，需要一条「同 key 多份时取分配记录未释放的那份」的规则（零轮）；GC 发布不推进「最近 4 个不同状态」（O5），源槽回收的节奏由根环决定。

### checker 多判哪几条

| 编号 | 内容 | 抓什么 | 判别力自证 |
|---|---|---|---|
| N1（改读法） | 走读读一个被引用的单元时，两份提示都对不上，再按映射位置读；引用集合记映射位置，不记过期的提示 | 搬迁之后旧槽被复用的合法镜像不被判红（O3） | 去掉这条回退，搬过且旧槽已复用的镜像必须由绿转红 |
| N2（新） | 反查：每条未释放的分配记录罩住的槽，读单元头得 key，映射里该 key 的位置含这个槽；豁免类则要被根记录或父指针直接指着 | 搬了没改映射、改了映射没搬、泄漏（有记录没人引用） | 只改映射不写副本 / 只写副本不改映射 |
| N3（新） | 活整理意图的 R 内没有分配代 > `c0` 的记录 | R 排他被破（D26 已定项 1 第 3 条；C243 的实现侧自证） | 关掉 R 保护，净增量塌向 0 |
| N4（实现） | I-4.4：logged_ops 字段白名单、补零、R 在单元区内、`c0` 不晚于该版 | 意图里混进增量字段 | 往条目里塞一个「再搬 N 个」 |
| N5（新） | C413：增量维护的碎片度与全空段统计 = 全扫重算（审计路径不与增量共享代码） | 触发量漂移（C91） | 增量少加一次 |
| N6（新） | 同一映射 key 在任意两个候选版本里的单元字节相同 | GC 改了内容（G7 的必出反例） | 搬迁时改一个字节 |
| N7（视 G-a / G-b） | G-a 下 I-3.10 不改；G-b 下改成 ≥ | 见上 | 见上 |

违例位用了 92 / 128（`crates/singlefs-checker/invariants/README.md`「违例位是 128 位…用了 92 位」那一段），N2、N3、N5、N6 四条新位装得下；每条在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`（4864 行）与 `crash_judge_checker_scan.wgsl`（4382 行）里各有一份镜像，N1 还要动 `crash_judge_recovery.wgsl`（656 行）。这是 G3 里最大的一块人工量，推的：不小于 O4 估的发布路径那一块。

### 代价总表

| 项 | 要动的文件 | 新写出的字节 |
|---|---|---|
| 发布入口与角色 | `crates/singlefs-core/src/transaction.rs`（新入口、新角色、:8466–:8622 位置来源、:8141–:8164 释放与取落点、点名项分配、`prepare_the_version_publish` 加参数 6 处调用） | 一批的字节见下 |
| 位置真相 | `crates/singlefs-core/src/recovery.rs`（:2982 起重建，:4825 的条数核加位置核） | 0 |
| 分配器 | `crates/singlefs-core/src/allocator.rs`（搬迁版的释放与取落点、目的地记录的代、排除区间） | 0 |
| 会话与挂载 | `crates/singlefs-core/src/mounted_session.rs`、`mount.rs`（G8） | 0 |
| journal | `crates/singlefs-core/src/journal.rs` 不动：`NamedUnit` 已是「位置条目 × 2 + 类标签 + 出生树 + 出生 txg + key 尾段」（:94） | 0 |
| checker | `crates/singlefs-checker/src/walk.rs`、`image.rs`、`invariants/05-空间记账.md`、`06-…`、`02-根环.md`；三份 WGSL | 0 |
| 文档 | D26 已定项 5、D3 已定项 7、I-3.10、`.claude/kb/layout/04-` 字节表 | 0 |

一批新写出的字节（两盘各一份）≈ 2 ×（K × 单元字节 + P × 16384）+ 记录 ⌈(K + P) ÷ 64⌉ × 4096 × 2，P = 这次新写的提交内生节点数（映射叶与路径、分配记录叶与路径、记账、树表、logged_ops 根）。P 我取 6 / 8 / 12 三档（推的，没跑）；P 随 K 涨：E94 的映射臂在 N = 64 的散布批次上 COW 了 62 个节点（「| 16 | 1024 | 60 | **244** | **17** | 36 | **62** | **3.9×** |」`.claude/kb/experiments/94-搬共享落点的触碰面.md:53`），所以下表 K = 64 一行是下界。命令与输出：`python3 -I bytes.py`（草稿目录，输出存 `bytes.out`），摘：

| K、单元 | P = 6 | P = 8 | P = 12 |
|---|---|---|---|
| K = 1 数据单元 | 270336 | 335872 | 466944 |
| K = 4 数据单元 | 466944 | 532480 | 663552 |
| K = 16 数据单元 | 1253376 | 1318912 | 1449984 |
| K = 64 数据单元 | 4407296 | 4472832 | 4603904 |

### 推荐与依据

推荐 P-1 + L-2 + G-a。依据：P-1 是 `publish_unlink_step` 已经走通的模式（独立入口、额外参数不并进 `PublishPlan`）；L-2 不破坏「内存版本与盘上父节点逐字对拍」这个现成的验证手段；G-a 不弱化 I-3.10。L-1 更小但要放宽很多对拍；G-b 是弱化一条已实现的不变量，要用户点头。

### 会推翻它的现象

- 读出一条发布路径或对拍用例，要求映射条目位置必须等于父指针位置（L-2 要动它；我只读了 `transaction.rs` 装映射的一段和 `recovery.rs` 重建的一段，没有逐个读 harness 用例）。
- 实测 P 远大于 12（映射叶散布、分配记录叶散布），单元段写数一批就越过 30（G6），则 K 只能取 1–2，G3 的批大小结论要让位给 G6。
- 发现 GC 发布其实被算成「非空」（O5 的推论不成立），则源槽回收的节奏与 G4 的 N₀ 估算要重来。

### 要交用户的岔路（不带倾向）

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 目的地记录的代：G-a 写诞生代 / G-b 写 P_b 的 txg 并把 I-3.10 改成 ≥ | 搬过的单元上 I-3.10 的判红数；「复用时留着上一次的代」这个变异是否还判得出 | 把两种写法各跑一遍 `checker_known_bad_images` 里 I-3.10 的坏镜像，G-b 下该变异不红即翻面 |
| 位置真相：L-1 改内存指针 / L-2 另存映射位置 | 需要放宽的对拍用例个数（`grep` 数 `TransactionOutput` 与重建结果逐字段比较的用例） | 数出两种下要改的用例数，再看是否有用例拿「指针位置 = 映射位置」当断言 |
| 父节点重写时写提示还是映射位置（C417） | 提示过期率（多跳读占比）；父节点重写是否额外多写 | E96 补「随重写刷新」之外的第二种政策的臂（C417 自己写的前置） |
| GC 发布算不算「非空」（O5 推论） | 源槽回收等了多少次发布 | 在真实现上跑一串纯 GC 发布，数 `deferred_slots` 随发布数的轨迹 |
| 豁免类（映射树节点、实例表）怎么办：不搬只排除 / 强制重写（P-3） | 有资格的段占比 | 长跑里统计「段里钉住的槽」占的段数 |

## 六、G4 停机谓词、N₀ 与三条水位的第一版取值（含 C373、C140）

### 判定

形式可定，数值不能凭空定。条款在这一格自相缠住三处，先摆出来：

**缠一：「全空聚簇段数」两份定义相反，代码取其一（C372）。** 并排抄句：
- D3 已定项 10 ①：「段内 64 槽都没有未释放分配记录的段数」（`.claude/kb/decisions/03-空间分配.md:208`），按这句，只剩 defer 槽的段算全空；同一行括号里却说「记账行记的就是它，`empty_segments`」。
- D26 已定项 1 ①：「defer 段不算空」（`.claude/kb/decisions/26-后台整理与放置回收.md:25`）。
- 代码：`used_per_segment` 只在回收时减（O5，「self.used_per_segment[segment] -= 1;」`crates/singlefs-core/src/allocator.rs:724`），`empty_segments` 取占用为 0 的段，即后一种。D3 自己那一行括号与前半句相反。

**缠二：停机谓词在最后一批那一刻按占用口径恒假。** 谓词是 D26 已定项 1 的产出谓词：「全空聚簇段数（现在）− `segs0` ≥ 1」（`.claude/kb/decisions/26-后台整理与放置回收.md:11`，`segs0` 必须以值写进意图见同文件 :26）。最后一批把 R 的活槽全释放，但它们进 defer，占用不减（O5；释放只加 defer 计数：「pub fn mark_released(&mut self, slot: SlotNumber, span: u64) {」`crates/singlefs-core/src/allocator.rs:555`，函数体里是「self.deferred_slots += span;」`crates/singlefs-core/src/allocator.rs:562`）；回收要等根环转一圈或 F 抬过（默认 24 条根，且 GC 发布不推进 F 的上限，O5）。所以最后一批发布完的那一刻，全空聚簇段数没涨，谓词为假。C372 第二轮攻方腿那段 64 槽扫描的结论（转述自 `.claude/kb/checks-owed.md:281`）与此同向。能同时成立的配对只有两个：「按记录口径 + 删意图那一刻判」，或「按占用口径 + defer 排空之后判」（同行）。

**缠三：合取 7 若连整理一起拦会死锁（推的）。** D3 登记表：「共七条，全部为真才许分配」（`.claude/kb/decisions/03-空间分配.md:136`），第 7 条「全空聚簇段数 ≥ N₀」（同文件 :146）；D26 已定项 3：「整理不是释放路径，它走常规准入」（`.claude/kb/decisions/26-后台整理与放置回收.md:67`）。按字面，全空聚簇段数掉到 N₀ 以下时，每一次分配都被拒，包括整理批次自己要的目的地与提交内生块，于是全空聚簇段数永远涨不回来——而 N₀ 立项是为了让棘轮不发散（「`N₀` 取能让棘轮不发散的最小值」`.claude/kb/decisions/26-后台整理与放置回收.md:68`）。底额要起作用，合取 7 必须只拦前台分配。条款没写这一句（C621 点名了「合取 7 归式子还是落点」，`.claude/kb/checks-owed.md:443` 同行）。

### 候选一：意图的收法（回答 C373、C140）

- W1 目标态到达与最后一批同发布删意图（D8 已定项 5 原文：「删除意图必须与最后一批工作在同一个事务里提交」`.claude/kb/decisions/08-核心索引结构.md:120`）。产出谓词不参与收；防连发要另算一个「待回收抵扣」。
- W2 目标态到达后意图不删，作「待回收令牌」活着，R 继续排他；收的条件是 R 自己在占用口径下全空了（`used_per_segment` 为 0，产出落地），或超过签发时写死的常量 W 个 txg（W 不应超过根环长度 3S，见 O11 与 G8 的发布预算），或谓词（占用口径）为真，或提前收。整理意图全池至多一条（G1 的 e1），所以有令牌在，不会再连发。D8 已定项 5 自己说意图分两类、整理落地时「按类分开写」（`.claude/kb/decisions/08-核心索引结构.md:108` 同句：「提示类（整理）可暂停、可在停机谓词下提前收」），W2 就落在提示类这一边。
- W3 谓词为真或目标态到达，任一即收（C373 的第三种读法）。
- 对 C373：谓词判真而目标态没到时删不删。W1 / W3 删，W2 也允许提前收（提示类可提前收，R 的排他随之解除；被搬了一半的状态每个都合法）。「停机不删意图则 R 一直锁着、续做再判仍真、意图永不完成」（转述自 `.claude/kb/checks-owed.md:282`）不会发生，因为提前收是允许的、W 是常量上限。
- 对 C140：产出用 R 自己的占用口径全空来判（逐意图），不用全局差值——全局差值会被前台吃掉一个空段抵消（E151 报过「被前台 bump 分配器当场吃掉」，转述自 `.claude/kb/checks-owed.md:194`）。全局口径留作 C140 的检查：每 N 条意图完成之后全空聚簇段数严格增加，在没有前台写的世界里断言。
- W2 要多一处：收这一步是 logged_ops 的一次修改，要搭在某次发布上。形态：每次发布前检查「有没有可收的意图」，有就把删条目并进这次发布（O(1)，至多一条）。这是 W2 的主要代价，也是它是否值得的分歧点。

### 候选二：N₀ 与合取 7 管谁

合取 7 管谁：
- X-1 只拦前台分配（用户数据与前台发布的提交内生块），整理批次豁免。
- X-2 拦所有分配，另让「全空聚簇段数」读数加上待回收的段（把缠二里的差额算进去）。
- X-3 拦所有分配，整理批次也受约束，N₀ 取小到整理自己的需求之下（实际上等于不留底额）。

N₀ 的值（草算，推的，命令：`python3 -I n0.py`，输出存草稿目录 `n0.out`）：一次 GC 发布新写 c 个提交内生块、换下 c 个；换下的要等根环转一圈才回收（回收门槛见 O5），稳态里 defer 滞留 (3S − 1) × c 个块；加一段给下一次开段。S 是每区槽数（格式常量下限 4、mkfs 默认 8、上限 16），c 取 5 / 10 / 13：

| S | c = 5 | c = 10 | c = 13 |
|---|---|---|---|
| 4 | 2 | 3 | 4 |
| 8（默认） | 3 | 5 | 6 |
| 16 | 5 | 9 | 11 |

- n0-a 取 1；n0-b 取 2；n0-c 取 ⌈(3S − 1) × c ÷ 64⌉ + 1，S 读系统配置里的每区槽数，c 按 D28 已定项 4 的现算 ckpt_cost（挂载时算一次，满足 `.claude/rules/fs-design.md` 硬要求 5「分支变量不许在操作中途改变」）。
- 这个下界假设稳态里每次发布换下的块都得等整个环转完；普通写也一样滞留，它们靠准入式子里的保留池兜（D3 已定项 4 的结论句：「可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数 − 环窗口(d)」`.claude/kb/decisions/03-空间分配.md:87`），N₀ 补的是「整段」这个形状，而不是容量。公式没有模型验过，**这一格要先有计数模型（E109 / E110 那种纯算术、钉绝对值、带变异表）**，再交用户。

### 候选三：三条水位

盘上字段与条款的名字对不上，先并排：
- 盘上：「整理低水位 / 高水位 / 停止线」（`.claude/kb/layout/01-first-txn.md:151`），三个 u64，0 = 内置默认。
- D26 已定项 1：「三个水位，任一越线即起跑」（`.claude/kb/decisions/26-后台整理与放置回收.md:25`），三个是三个量（全空聚簇段数、碎片度、不可回收字节）。
- 代码：「三条水位第一版恒 0，0 的含义是「用内置默认」」`crates/singlefs-core/src/system_configuration.rs:90`，内置默认本身也是 0：「const COMPACTION_WATERMARK_BUILT_IN_DEFAULT: u64 = 0;」（`crates/singlefs-core/src/system_configuration.rs:91`）；harness 抄了一份：「const BUILT_IN_COMPACTION_WATERMARKS_OF_THE_CLAUSES: [u64; 3] = [0, 0, 0];」（`crates/singlefs-harness/src/mount_transparency.rs:1398`）。
- 「内置默认 = 0」等于不起跑。GC 落地就要把内置默认换成非零，这两处和 kb 要一起改。

字段含义候选：
- V-a 三个字段 = 三个量各一个起跑阈值：[0] 全空聚簇段数（≤ 越线）、[1] 碎片度（F = runs ÷ L 的定点数，要先写缩放，≥ 越线）、[2] 不可回收字节（≥ 越线）；三个量都带设备维，任一块盘越线即起跑。盘上字节不变，只是 layout 的名字改。
- V-b 三个字段 = 同一个量（全空聚簇段数）的低 / 高 / 停止线，其余两个量没有阈值。与 D26 已定项 1「三个量任一越线」矛盾，要改条款。

第一版取值的形式（数不是我能定的）：
- 起跑只开「全空聚簇段数」一条，阈值 = N₀ + 2（让整理在底额被碰到之前起跑，且给在飞的那一条意图留一段）；碎片度与不可回收字节两个字段第一版取 0 = 关（E10 还没有碎片度对遍历带宽的曲线，D26 已定项 1 要的是「在那之前取保守默认、做成挂载可调」，`.claude/kb/decisions/26-后台整理与放置回收.md:29`）。
- 做成挂载可调：水位在 `SystemRuntimeConfiguration`（无字段的 unit struct，文档写着「改它无代价、每次挂载可重设并回写」`crates/singlefs-core/src/system_configuration.rs:275`），同一段文档又说「这三行今天既没有可设的入口」（`crates/singlefs-core/src/system_configuration.rs:278`），原因是挂载选项通道不存在（C422，`.claude/kb/checks-owed.md:316`）。所以「挂载可调」要么先补 C422 的通道，要么第一版用只供测试的覆盖值代替（`.claude/rules/fs-design.md` 硬要求 2：每条分支能被测试强制进入），再加「强制起跑」「强制停」两个开关。推荐后者，C422 另算。
- 起跑条件在一次整理运行开始时读一次；运行中途不重读水位（硬要求 5）。停的条件只有 R 级的（上面 W2 的收法）与会话的预算。

### 代价：要动的文件、新写出的字节

| 项 | 要动的文件 | 新写出的字节 |
|---|---|---|
| 水位含义与内置默认 | `crates/singlefs-core/src/system_configuration.rs`（:88–:91、:303）；`crates/singlefs-harness/src/mount_transparency.rs:1398` 与它的用例；`.claude/kb/layout/01-first-txn.md:151` 改名 | 新池字节不变（字段仍是 24 字节，值写 0） |
| 合取 7 | `crates/singlefs-core/src/admission.rs`（今天只实现了合取表第 1 条，文件头第 2 行）与 `allocator.rs` 的落点函数；N₀ 的读数要有在飞 overlay（D26 已定项 3 射程欠的三样：在飞 overlay、一条 I-3.1 形态的不变量、设备维） | 0 |
| W2 的收 | `crates/singlefs-core/src/transaction.rs` 的发布入口前加一个检查；`mounted_session.rs` | 收一次 = 一次 logged_ops 根重写 16384 × 2 盘 |
| 待回收抵扣（若走 W1 而不走 W2） | `allocator.rs` 每段加 defer 计数；若落盘则记账多一统计量（D5 已定项 4 清单 + 1）、新池每盘多一行 | 落盘：新池记账树多 1 行 × 盘数；不落盘：0 |
| 检查 | N3、N5（G3）；必红用例「关掉 R 保护，净增量塌向 0」（C243 的实现侧自证）；「填满 → 删除 → 填满」循环里删除事务的提交内生块数不随轮数发散（C139 的计数模型） | 0 |

### 推荐与依据

推荐 W2 + X-1 + V-a（只开全空聚簇段数一条）+ n0-c（算式，值由计数模型定）。依据：W2 靠「至多一条活整理意图」自己挡住连发，不用新统计量；X-1 是缠三唯一不死锁的读法；V-a 与 D26 已定项 1 的「三个量」同形、盘上字节不动。W2 与 D8 已定项 5 的「删除意图必须与最后一批工作在同一个事务里提交」按类分开写（C621 的第 6 处）是用户级改动，所以 W2 以下全是零轮形态，交用户时要这样标。

### 会推翻它的现象

- 计数模型里 W2 的令牌在空闲系统上挂得比 W 久、且挂着的 R 让「可开聚簇段」少于 N₀ ⇒ W 取小或改 W1 + 抵扣。
- 实际 c（一次 GC 发布新写的提交内生块数）在真实现上远大于 13 ⇒ n0-c 的值上移。
- 发现提示类意图的「提前收」在崩溃枚举里造出新的合法性问题（半搬状态里某条不变量红）⇒ 回头改成只在目标态到达时收。

### E10 的岔路单草稿（G4 交付，不带倾向）

草稿另存一份 `/home/fy5090/.local/share/singlefs/work/m4-gc-r1-forward/e10-forks-draft.md`（与下表逐行相同），可原样发给实验设计员。

| # | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 1 | 起跑量：只用全空聚簇段数 / 三个量任一越线 / 低高停止线滞回 | 老化轨迹上三个量先后越线的次序；误触发（起跑后产出为 0）的次数 | 含整理运行的老化轨迹，三个量的时间序列各一条，5 个种子以上 |
| 2 | 全空聚簇段数起跑阈值取 N₀ + 1 / N₀ + 2 / N₀ + ⌈单批最坏占用 ÷ 64⌉ | 前台首次回落（分配走不进聚簇段）出现的轮次与整理起跑的轮次谁先 | 量出「整理在前台首次回落之前起跑」的最小阈值 |
| 3 | 碎片度起跑阈值：关 / 取 E93 判据 4 的 L/4 的某个比例 / 取遍历带宽的拐点 | 遍历带宽（QEMU 真设备）随碎片度的曲线有没有拐点 | 曲线上读出拐点，且拐点在不同填充率下不漂（至少 3 个填充率） |
| 4 | 不可回收字节阈值：关 / 占容量的百分比 | 到 ENOSPC 之前 `df` 与实际可用的滞后量 | 填满—删除—填满循环里量出滞后量的上界 |
| 5 | N₀：1 / 2 / ⌈(3S − 1) × c ÷ 64⌉ + 1 | 「填满—删除—填满」循环里删除事务的提交内生块数是否随轮数发散（C139 的第二半）；整理完成率 | 计数模型在 S ∈ {4, 8, 16}、c ∈ {5, 10, 13} 九格上的绝对值，再加实现臂至少三格 |
| 6 | 合取 7 管谁：只拦前台 / 拦所有加待回收读数 / 拦所有不留底额 | 全空聚簇段数 = N₀ 时整理能不能起跑 | 实现单测（钉绝对值）+ 计数模型 |
| 7 | 意图收法：目标态即收 / 令牌式（W2）/ 谓词或目标态任一 | 空闲系统上令牌挂的 txg 数；连发次数；同一个段被反复搬的次数 | 「删除后空闲」与「混合负载」两种轨迹各一条 |
| 8 | 批大小 K：1 / 4 / 一批罩整个 R / 受记录上限 | 每腾出一个段所需的发布数；每段释放对应的提交内生块总量 | 单元段写数先在 G6 的流上量出来，再在老化长跑里量 |
| 9 | 选段：greedy / cost-benefit / 轮转 | 同写预算下 runs 终值与全空聚簇段数轨迹 | 三臂 × 5 种子，过 E93 判据 4 的作废闸 |
| 10 | 降级判据「③ 达到 ② 的水平」里的「水平」：runs 终值比 / 遍历带宽比 / 两者 | ③（关整理 + 结构抗老化）与 ②（开整理）差距的数 | 登记里写死比值门槛；E10 页的判据只有「③ 若能达到 ② 的水平」这一句（`.claude/kb/experiments/10-老化结构抗老化vs事后整理.md:13`），没有数值 |
| 11 | 长跑多久算够 | 轨迹是否到平台（峰值、为正轮数、期末值，`test-discipline.md`「端点不是轨迹」） | 平台后再跑一倍长度，三个数不变 |
| 12 | 整理的挂钟代价（C412）：一批的挂钟 / 一段的挂钟 | 批大小翻倍，挂钟是否跟着涨 | 口径写明设备、负载、批大小，钉绝对值 |

## 七、G5 discard 什么时候发

### 判定

乙按字面不安全，甲要修正后才安全，丙与出口 2 冲突。三句话各有出处：

- 提交顺序是 单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → 系统配置轮换 → 屏障（我读 `crates/singlefs-core/src/transaction.rs:1248` 的 `persist_publish_writes` 与 :1402 的 `persist_the_root_then_rotate_the_system_configuration` 得出）。回收的决定做在这个顺序之前：「这次的根要盖掉根环里的一个槽：落点都取完了、这次不再分配，在装记账行之前记下它」（`crates/singlefs-core/src/transaction.rs:8165`），也就是回收算的是「这条根写进去之后」的环，但这条根此刻还没持久。回收出来的槽，若在这次发布的根 FUA 之前就 discard，崩在 FUA 之前，恢复落到的老根还引用着它——I-7.4 要求候选根引用的块「未被重新分配给其他对象、也未被清扫抹头」（`crates/singlefs-checker/invariants/02-根环.md:11`），discard 就是一种抹。所以发 discard 必须在整个发布返回之后。
- 乙「GC 产出全空聚簇段那次发布之后整段发」：最后一批发布完，R 的槽全在 defer（O5），老根还引用它们，不能发。乙只有在「全空」按占用口径（即回收之后）判时才成立，那就是下面的乙'。
- 出口 2 的字面是「自建 GC 落地：意图、分批搬迁、产出全空聚簇段、discard」（`.claude/kb/milestone/04-fourth-txn.md:16` 同行开头），优化项表又把 discard 写成交给盘那条线的底座（`.claude/kb/milestone/04-fourth-txn.md:163`）。丙（第一版不发、只留口子）与出口 2 的字面不符；而 `.claude/singlefs-ai-sop/rules/code-discipline.md`「用不到的代码删掉」（`04-fourth-txn.md` 优化项一节引用的那条）又不许留没有调用者的方法。丙要用户先改出口 2 的字面。

### 候选

- 甲 逐回收发：槽被回收、且分配器此刻认它为「可发」（没分配、没被影子账隔离、没被抬 F 扣住，即 `held_until_floor_takes_effect` 与 `isolated` 都没置，`crates/singlefs-core/src/allocator.rs:270` 与 :278 的两张位图；扣住的槽要等「pub fn release_reclaim_holds(&mut self) {」`crates/singlefs-core/src/allocator.rs:2198` 放开）之后，把相邻的可发槽并成区间，发布返回之后发。
- 乙' 整段发：同甲，但只发 64 槽对齐的整段（占用口径下全空聚簇段数涨的那一刻）。粒度 1 MiB，少于 SSD 擦除块通常的大小的碎 discard 不发（SSD 内部块大小在本机测不到，**复核不了**）。
- 乙 按字面：出局（上面）。
- 丙 第一版不发：见上。

还有一个时机分叉，与甲 / 乙' 正交：T1 每次回收后立刻发；T2 攒到至少 N 段或卸载时发；T3 只在卸载时发。

### 块设备 trait 加什么（O7）

- 新增一个有默认实现的方法，例如 `discard_at(offset, length)`，默认返回「不支持」这个结果值（不是错误），调用方计数。默认实现使 35 个实现者（`grep -rn 'impl.*BlockDevice for' --include=*.rs crates | wc -l` 输出 35）不必全改；但录制设备、故障注入设备、内存盘必须覆盖，否则静默吞掉（违反 `.claude/rules/fs-design.md` 硬要求 4「分支必须可观测」）。
- 文件后端：复用打洞，`fallocate` 的 PUNCH_HOLE 且不改文件长度（`punch_hole_keeping_the_file_length`，`crates/singlefs-core/src/block_device.rs:391`），读回全 0，比真盘的 discard 语义强，对 checker 无害（读回零或旧字节，两者都不是任何候选根引用的有效单元）。
- 真块设备（BLKDISCARD ioctl）：要第二处 unsafe。现状是「std 没有 fallocate 的安全封装；全 crate 只在这一处放开 unsafe」（`crates/singlefs-core/src/block_device.rs:389`），第一版 QEMU 与镜像文件够用，真盘的 ioctl 留到真盘线（里程碑四五）再开。
- 内存盘与录制设备：记 Discard 事件；录制流加一种步骤，足迹照整段清零那一次（O7：20 个文件，入口 `crates/singlefs-harness/src/lib.rs:40`、`segments.rs` 的分类、四个手抄名字表的 bin、`fault_injection.rs`、`memory_pool.rs`）。
- 设备日志判器：今天解出 Discard 然后和 Flush、Mark 一起丢（O7）。要改成：GC 流里每个 Discard 都必须落在一个「可发」区间内，且在对应发布的最后一道屏障之后。

### 崩溃点重放里 discard 算不算写（D13 已定项 4）

- 要算。D13 已定项 4 的枚举域是「一次写取两态（持久 / 没持久）」（`.claude/kb/decisions/13-验证路线.md:134`）；discard 当成「范围写零」：持久后该范围读 0，没持久读旧字节，与整段清零同一个模型。不算的话，「discard 发早了」这个错在崩溃枚举里不会出现。
- 它发在发布返回之后，单独成一段，每个整段一个范围；一段里只有 discard，状态数 2^(范围数)，可忽略。
- 必红世界：把 discard 提前到根 FUA 之前（或提前到抬 F 生效之前）⇒ 某个崩溃状态里老候选根引用的单元被零化，I-2.1 / I-7.4 红。登记进 `crates/mutations.tsv`。

### 发了 discard 的落点在 I-7.4 下怎么判

不加新不变量。I-7.4 与 I-2.1 在「应用了 discard 的崩溃镜像」上照判：候选根引用的单元被零化就红。要的是崩溃镜像生成器认 Discard（上面）。

### 代价：要动的文件、新写出的字节

| 项 | 要动的文件 | 新写出的字节 |
|---|---|---|
| trait 与后端 | `crates/singlefs-core/src/block_device.rs`（trait 加一个有默认的方法，文件后端与 O_DIRECT 后端实现；文档「六个动作」改） | 0 |
| 分配器 | `crates/singlefs-core/src/allocator.rs`：待发集合、可发谓词、取区间的函数；三个回收点（`mount.rs:3020`、发布内的 `record_root_written…`、`mount.rs:4764` 抬 F）都往待发集合里放 | 0 |
| 发出 | `transaction.rs` 或 `mounted_session.rs` 的发布返回之后；形态同实例表清扫（绕开 `CommitStep`，O7），不新增提交步骤，于是不触发 D17 连锁表写的那条门禁（改共享的 `CommitStep` 枚举要重开一轮层 2 重放，`.claude/kb/decisions/17-实现分层与第三方管道.md:154`） | 0 |
| 录制与崩溃镜像 | `crates/singlefs-harness/src/{lib,segments,fault_injection,memory_pool}.rs`、`crates/singlefs-checker-tier/src/{device_log,crash}.rs` 与四个手抄名字表的 bin | 0 |
| 变异与文档 | `crates/mutations.tsv`；D26 未定项 9「还缺一个前提」一段、D12 已定项 4 | 0 |

### 推荐与依据

推荐乙'（整段发）+ T1，发布返回之后、可发谓词为真时。依据：只在一个会改变全空聚簇段数的时刻动作，粒度 1 MiB 对 SSD 友好；放在发布返回之后避开根 FUA 的窗口；可发谓词复用分配器已有的三张位图，不另立判据（满足硬要求 1：判据显式、可计算）。

### 会推翻它的现象

- 查出回收发生的某个点不在 `mount.rs` 的三处与发布内那一处之内（我只数了这四处）⇒ 待发集合漏放。
- 发布返回之后、discard 之前崩溃，重挂后待发集合丢失 ⇒ 这些槽永远不发（只是少发，不错）；要补则在挂载末尾重算一次待发，代价 O(段数)。
- 真 SSD 上 discard 后读回旧字节（不保证零）使某个未被引用的槽仍可读——对 checker 无害，对 scrub 的「可读垃圾」口径（I-1.9）有影响，**复核不了**（没有真盘）。

### 要交用户的岔路（不带倾向）

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 甲 逐回收 / 乙' 整段 / 丙 不发 | 盘内写放大（FTL 模拟器的输入，D26 未定项 8 / 9 的输入）；discard 命令数 | 乙' 与甲的区别要真盘或 FTL 模拟器才量得出，本机量不到；丙要用户先改出口 2 的字面 |
| T1 立刻 / T2 攒批 / T3 只在卸载 | 一次回收到下一次分配复用的间隔里，discard 是否赶在复用之前 | QEMU 上能量「复用前没发」的次数；真盘效果量不到 |
| 文件后端的 discard 是否打洞 | 镜像文件的实际占用字节随老化的轨迹 | 在镜像文件上量 `du` 随轮数的变化（本机可量） |

## 八、G6 崩溃流与必红世界

### 判定

可定。二·3 要的两条流里，GC 搬迁那一条现在就能设计；搬被快照共享的块那一条要等一·2 的快照形态，这里只写骨架。

### 接在哪条流后面

- 前缀要有一次重挂。选段的资格要求 R 不在本次挂载开过的段里（G2），而崩溃流里的池从 mkfs 起只有一个实例，所有用过的段都是「本次挂载开过的」，不重挂就没有段可选。HistoryOperation 里已有重挂这一步（`CloseAndMountWritable`，`crates/singlefs-harness/src/history.rs:475` 起的枚举）。
- 碎片要造得出来：多记录流「取号 → 暖机 × 2 → A → B → C」（`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs`）里每次发布整份重写第一个文件，A、B 的单元释放进 defer，C 的是活的。重挂后 A、B 的槽按环里最旧有效根的门槛多半还没回收（O5），所以老段里是「活槽少、占用高」，正好是 G4 缠二那个形态。需要回收的话前缀里再接一次抬 F（`crash_enumeration_floor_raise_pushed_by_admission.rs`）。前缀的精确步骤我没试，**推的**，要实现员在副本上试。
- 候选前缀：（a）多记录流 + 重挂 + 意图 + 批 + 最后一批；（b）接在 unlink 流之后（文件删光后没有活数据可搬，不选）。取 a。

### 每批写几个单元、对着三个上限估态数

单元段写数 = 盘数 ×（搬的单元数 K + 这次新写的提交内生单元数 P），每次写取两态，状态数上界 2^写数。P 取现有删除流各次发布的 11–13 个（O9）。命令与原样输出：`python3 -I states.py`（草稿目录，存 `states.out`）：

| P | K | 单元段写数 | 2^写数 | 展开 |
|---|---|---|---|---|
| 11 | 1 | 24 | 16777216 | 全量 |
| 11 | 2 | 26 | 67108864 | 全量 |
| 11 | 4 | 30 | 1073741824 | 全量（正好压线） |
| 11 | 5 | 32 | 4294967296 | 越过 30，exhaustive=false |
| 12 | 3 | 30 | 1073741824 | 全量（正好压线） |
| 12 | 4 | 32 | 4294967296 | 越过 30，exhaustive=false |

- 展开深度 30（「FULL_EXPAND_UP_TO=30」`scripts/crash/crash-amplification.sh:319`）已经被现有两条流的第 16 段用满：「今天是 30 写：多记录流与删除流」（`scripts/crash/crash-amplification.sh:50`）。所以 GC 流的批大小只能取 1–3（P = 12 时）或 1–4（P = 11 时），再大就是抽样、不是穷举，出口 2 的「全域 0 红」要有对应的措辞。
- 一个落点至多 64 次写（「pub const MAXIMUM_WRITES_PER_LOCATION: usize = 64;」`crates/singlefs-checker-tier/src/crash_judge_tables.rs:54`）：搬迁流里一个槽至多被写进一份副本、被回收复用一次，离 64 很远（推的）。
- 一条流至多 2048 次写（「pub const MAXIMUM_ENUMERATED_WRITES: usize = 2048;」`crates/singlefs-checker-tier/src/crash_facts.rs:369`）：新增四次发布、每次约 30 写，离上限远；前缀（多记录流 + 重挂）本身的总写数我没数，**复核不了**。
- 一段的状态数装进 u64：2^30 远小于 2^64；背景材料引的 `crash.rs:1548` 我没找到，没复核。
- 状态总数（推的，仅数量级）：意图那次发布 K = 0（约 2^16）、两批各 2^26、最后一批 2^26，加前缀，与删除流的 1463812278 同数量级（O9）。

### 必红世界（每条先证会红）

| 编号 | 改坏哪一处 | 哪条判器红 |
|---|---|---|
| M1 | 源落点在映射发布之前就被释放（把释放放进前一次发布，或本发布里不改映射条目） | G3 的 N2、I-3.11（已分配减 defer 等于最新根走读）；这就是 `04-fourth-txn.md` 二·3 写的例子 |
| M2 | 只改映射、不写副本 | N2（反查读不出）、I-2.1 |
| M3 | 写了副本、不记目的地的分配记录 | I-3.1、I-5.4、N2 |
| M4 | 搬迁时改一个字节 | N6（同 key 字节相同） |
| M5 | 关掉 R 排他（目的地落回 R） | N3；C243 的实现侧自证「净增量塌向 0」 |
| M6 | 目标态写成「清空 R」（不单调）：前台填回 R | 每条意图在有界批数内被收的检查（C373） |
| M7 | 续做点不回起点（若取 a1） | F4 的历史：回退复活已越过的槽，意图删了而目标态没到 |
| M8 | 释放不经映射、经提示 | D19 已定项 5 硬规则 1；释放到别的单元 |
| M9 | discard 发在根 FUA 之前（G5） | I-7.4 / I-2.1 |

### 只供测试的开关（`.claude/rules/fs-design.md` 硬要求 2：每条分支能被测试强制进入）

照 `DeletionSwitches`（「pub struct DeletionSwitches {」`crates/singlefs-core/src/unlink.rs:28`）装在分配器上的做法，加 `CompactionSwitches`：
- 批大小上限（照 `UnlinkBatchDataUnitsCappedForTests`，`crates/singlefs-core/src/unlink.rs:60`），崩溃流里取 1–3。
- 强制选哪一段：绕开资格（`cluster_segments`、阈值），直接指定起始槽。
- 强制起跑 / 强制停。
- 关闭 R 排他（阳性对照用，M5）。
- 强制「提示过期」：复用窗口置 0（`ReuseWindow::ForcedToZero`，`crates/singlefs-core/src/allocator.rs:1890`）已经存在：搬完源槽立刻回收复用，旧提示读出别人的字节，转映射（N1 的路径）；不用新开关，用这个。

### 要动的文件与登记处

- 新测试 `crates/singlefs-checker-tier/tests/crash_enumeration_compaction_after_remount_stream.rs`，形态照 `crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`；公共步骤在 `tests/common_crash_points/mod.rs` 加 `publish_the_compaction_intent`、`publish_the_next_compaction_batch`。
- 登记三处，照 unlink 流的现有行：`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行（「crash-case:layer0-unlink-after-the-multi-record-publish-stream」`.claude/gate.d/stage-inputs.tsv:57`）；`.claude/gate.d/crash-point-replay-items.tsv:18`；`scripts/crash/crash-amplification.sh` 的分项表（unlink 那一行在 :331）。
- 必红用例表：`.claude/kb/verification-build.md` 的「harness 自己要证明会红的用例」表；变异：`crates/mutations.tsv`。
- 第二条流（搬被快照共享的块）：骨架 = 一·4 的「建快照 → 快照下覆盖写」流之后接 GC 搬迁被快照引用的块；必红世界多一条：搬迁后只更新活头一侧的位置、快照一侧仍读旧提示 ⇒ N1 缺失时红。等一·1 定快照形态后再细化。

新写出的字节：测试装置里的字节，不是产品字节。

### 推荐与依据

推荐前缀 a、批大小 1–3、开关用 `CompactionSwitches` + 现成的复用窗口置 0。依据：O9 的数；复用窗口置 0 是已有的、有用例的强制路径。

### 会推翻它的现象

- 实现出来的 P 比 12 大（映射叶散布、分配记录叶散布）⇒ 批大小要降到 1–2。
- 多记录流 + 重挂的前缀上，选段资格一个段都过不了（O5：回收门槛太低，活槽少的段不存在）⇒ 前缀要接抬 F。

### 要交用户的岔路（不带倾向）

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 批大小 1 / 2 / 3 / 4 全量穷举，还是 5 以上抽样 | 实现里一批的单元段实际写数（P 的真值） | 在副本上跑一批，数 `PublishWrites.units` 的个数；超过 30 就只能抽样 |
| 出口 2 的「全域 0 红」是否接受 `exhaustive=false` 的抽样 | 抽样与穷举对同一组变异的杀伤率 | 对 M1–M9 各跑抽样一遍，数杀伤；E165 有现成的界 |

## 九、G7 「GC 是恒等变换」怎么证

### 判定

可定。Verus 能证的是两样：纯计划函数的性质，和一条抽象状态引理（把「照计划搬完、读到的内容不变」写成模型层的引理）。证不了执行层：Verus 只编 `singlefs-replay-prefix` 这一个 crate（「只依赖 vstd，让 Verus 只编这一个文件」`crates/singlefs-replay-prefix/Cargo.toml:6`），不编 `singlefs-core`，所以发布路径是不是把字节原样写到目的地、设备写序、崩溃，都不在它的射程里。执行层靠 N6（同 key 单元字节相同）、模型对拍和必出反例的世界（出口 5 的字面也是这三样，其中「必出反例的世界」见 `.claude/kb/milestone/04-fourth-txn.md:65`）。

### 现状（观测）

- 已有的 Verus 对象：`replay_core`（「pub fn replay_core<」`crates/singlefs-replay-prefix/src/lib.rs:3543`），crate 文档写「Verus 验的就是编进产品、被恢复路径调用的这一份」（`crates/singlefs-replay-prefix/src/lib.rs:2`）；`singlefs-core` 在 `crates/singlefs-core/Cargo.toml` 里按路径依赖它；必红世界表 168 行（`wc -l crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`）。
- 门禁把这个 crate 的目录写死：「Verus 四格的输入只有 crates/singlefs-replay-prefix/」（`.claude/gate.d/checker-tier-formal-verification.sh:37`）。`grep -rc 'singlefs-replay-prefix'` 在几个文件里的计数：`.claude/gate.d/checker-tier-formal-verification.sh` 4、`.claude/gate.d/verus-pins.tsv` 4、`scripts/gate/verus-proof-check.py` 1、`scripts/gate/admission.py` 20、工作区 `Cargo.toml` 1。

### 候选：证明放哪

- V-1 放进现有的 `singlefs-replay-prefix` crate（新模块），沿用同一套门禁、pins 与必红世界表。代价：crate 的描述（「取 journal 前缀并施加的纯函数核」）要改；`cargo verus` 对 crate 内多模块的支持我没试，推的能行（lib.rs 一个文件已经 4284 行，拆模块不是必须）。
- V-2 新 crate `singlefs-gc-plan`（只依赖 vstd）。代价：上面那五个文件里写死的 crate 名要泛化成列表（门禁脚本、pins、证明脚本、admission、工作区），再加一份自己的必红世界表；换来的是边界清楚。

### 候选：证什么

- P1 选段健全：选出的 R 属于有资格的段（不在 `cluster_segments`、不在活 R 里、没有钉住的槽、至少一个活槽）；最优性：对每个有资格的段 s'，`score(R) ≥ score(s')`（S-a 时是活槽数不多于）。最优性要循环不变量，难度比前一条高，可以先只证健全。
- P2 计划健全：key 互异；目的地都取自候选；目的地都不在 R 内；目的地两两不重叠；目的地跨度等于源跨度，数据单元起点为偶数；条数不超过 K；完整性：把全部批的计划并起来恰好是 R 内活映射 key 集合。
- P3 进展：在模型层应用一批（源标已释放、目的地标已分配），R 内「未释放且分配代 ≤ `c0`」的记录数严格减少，非 R 内这类记录不增。有限批后到目标态。这是 D26 已定项 2 的「单调」的形式化。
- P4 恒等引理（抽象）：抽象状态 = key 到内容；具体状态 = key 到位置，加上位置到字节。读 key = 取它位置上的字节。引理：计划满足 P2、执行是「把源字节原样写到目的地、把该 key 的位置改成目的地、别的不变」，则每个 key 读到的字节不变，计划内外的 key 都不变（计划外的 key 因目的地与它们的位置不重叠）。
- 证不了、交别处：执行器真的原样写了；设备写序；崩溃。

### 模型对拍

- `HistoryOperation`（「pub enum HistoryOperation {」`crates/singlefs-harness/src/history.rs:475`）加一种 `CompactWhileMounted(选段选择)`，`HistoryOperationKind::ALL` 从 12 项变 13 项（`crates/singlefs-harness/src/history.rs:540`）。
- 模型（`crates/singlefs-harness/src/model.rs`、`model_comparison.rs`）与 RefFS（`crates/singlefs-reference-filesystem`，翻译层 `crates/singlefs-harness/src/reference_filesystem_translation.rs`）里这一步是恒等：状态不变。比较的是每步之后读出的内容。
- 「随机历史穿插 GC，结果与不跑 GC 逐项相同」（出口 5）：同一个种子的历史跑两遍，一遍不插 GC，一遍在若干点插；比较终态的逻辑内容，逐项含 inode 每条、extent 单元序号集与内容校验和（D8 已定项 16 第 7 条给过这个口径：「含 inode 每条（号、对象出生代、size、nlink）」`.claude/kb/decisions/08-核心索引结构.md:483`，不含落点）。
- 对拍有种子命中：断言在整批种子上至少一次 Compact 真的搬了单元，否则对拍空转；这是必红世界之外的第二道自证。
- `KNOWN_RED_FORMS` 不新增，是里程碑四三·2 开工前要先定的那一条（转述自 `.claude/kb/milestone/04-fourth-txn.md` 的「三 生命周期闭环与长期老化」一节）。

### 必出反例的世界

- Verus 侧：必红世界表加 8–12 行：去掉「不在 R 内」、去掉「两两不重叠」、把比较方向反过来、去掉完整性的一个子句、P4 里把「原样写」换成「写后异或 1」。
- 对拍侧：执行器搬迁时改一个字节 ⇒ 对拍红且 N6 红。

### 代价

| 项 | 要动的文件 | 新写出的字节 |
|---|---|---|
| Verus 核与证明 | V-1：`crates/singlefs-replay-prefix/src/` 加模块，`verus-must-red-worlds.tsv` 加行；V-2：新 crate 加门禁五处。规模参照 lib.rs 4284 行（推的：几百到两千多行） | 0 |
| 胶水 | `crates/singlefs-core/src/compaction_plan.rs` 调核（形态同 `recovery::replay_journal` 调 `replay_core`），两边类型手抄映射（lib.rs 文档里承认的做法） | 0 |
| 对拍 | `history.rs`、`model.rs`、`model_comparison.rs`、`reference_filesystem_translation.rs`、新用例 | 0 |

### 推荐与依据

推荐 V-1 + P1（先只证健全）+ P2 + P3 + P4，对拍与必出反例如上。依据：V-1 不碰门禁脚本；P2、P3、P4 是出口 5 要的「恒等」的全部可证部分。

### 会推翻它的现象

- `cargo verus` 在 crate 内加第二个模块报错，或 P2 的完整性在登记的 rlimit 下超限 ⇒ 改 V-2 或把完整性降为运行时断言。
- 计划函数需要 `BTreeMap` 之外的容器，Verus 的 vstd 不支持 ⇒ 改用有序 `Vec`。

### 要交用户的岔路（不带倾向）

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| V-1 放现有 crate / V-2 新 crate | `cargo verus` 在多模块上能否通过；门禁泛化的改动行数 | 在副本上试加一个空模块跑 `cargo verus verify` |
| P1 只证健全 / 连最优性 | 证明在登记 rlimit 下的耗时 | 先证健全，量出耗时，再决定加最优性 |
| 对拍插 GC 的位置：每步后 / 随机点 / 只在批间 | 命中种子数；各插法对 M1–M9 的杀伤 | 同一批种子各跑一遍，数杀伤 |

## 十、G8 调度：第一版谁触发 GC

### 判定

甲只给机制，满足不了 D3 已定项 14 的承诺。并排抄句：

- D3 已定项 14：「就等于承诺后台整理是常驻的、自动的、用户无需知道的」（`.claude/kb/decisions/03-空间分配.md:277`）。
- 甲的定义（背景材料 G8）：只由会话显式触发。显式触发是有人调，不是自动。

事实：挂着时没有调度器，「入口在，调度器没有」（`crates/singlefs-core/src/instance_table_sweep.rs:14`）；会话是单线程所有权，发布入口是 `&mut self`（「pub fn publish_user_change<Device: BlockDevice>(」`crates/singlefs-core/src/mounted_session.rs:1049`）；事务层先承诺串行提交（「事务层先承诺串行提交」`.claude/kb/decisions/13-验证路线.md:87`）；GC 批次在并行提交下的取号归里程碑六（「GC 批次在并行提交下怎么取号、堵不堵前台 fsync 不归里程碑四」`.claude/kb/milestone/04-fourth-txn.md:62`）。

### 候选

- 甲 会话显式触发：`MountedSession` 加一个入口，一次调用跑到停机、预算用完或暂停。不自动。
- 乙 挂载末尾跑一批：在 `continue_the_deletion_queue_until_it_is_empty`（`crates/singlefs-core/src/mount.rs:6447`）之后、实例表清扫之前或之后。挂载时间随起跑与否变；m3-intent-r1 I3 定的是「挂载里只续 `deleted_inodes`」（转述自 `research/prompts/m3-intent-r1-main-verification.md` 的 I3 行，条款侧是「续做入口按树分、共用发布路径」`.claude/kb/decisions/08-核心索引结构.md:108`），提示类「不挡挂载」同行。乙把一批整理放进挂载，读起来贴着字面但违精神，我读它为不推荐。
- 丙 后台线程：会话是 `&mut self` 单线程所有权，加线程要共享可变会话；并行提交归里程碑六。本里程碑不做。
- 丁 用户改动后附带一批：`publish_user_change` 成功之后，若水位越线，再跑至多一批（预算内）。与删除续做同一个风格：会话每次用户改动先把删除队列走空（「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空」`.claude/kb/decisions/08-核心索引结构.md:482`），再做这次改动，丁再在后面附带一批整理。这是不加线程就做到「自动」的最小形态。

### 次序与空间被拒时怎么办

- 次序：删除续做（承诺类）→ 为空间推的抬 F（只为用户请求）→ GC 批。删除先走空，GC 才不会搬正在死的单元。
- GC 被空间拒时：g1 停下并报告「暂停（空间）」；g2 像删除类发布那样推抬 F 再试。推荐 g1，理由：抬 F 会把回退候选集缩到最近 4 个状态（「回退候选集保留最近 4 个可退到的不同状态」`.claude/kb/decisions/16-发布语义.md:29`，推空发布的预算见同文件 :42 的「一次准入最多 B = 4 + 2 k_tol 次发布」），为后台整理缩用户的回退深度不合算；用户请求被拒才推 F（`push_one_floor_raise_within_the_admission_budget`，`crates/singlefs-core/src/mount.rs:4140`）。
- 观测（硬要求 4）：起跑次数、批数、停机原因（谓词 / 预算 / 空间拒 / 暂停）各一个计数，放在 `UserChangePublished` 一类的返回值里。

### 一次整理运行的发布预算（O11）

- 每次运行（甲的一次调用，或丁的一次附带）最多发 B_gc 次发布。B_gc 的上界：根环 3S 条减去用户要保住的不同状态各自最后一条根之后已经过去的发布数，再留余量；默认 S = 8 时环是 24 条。用户改动的发布也占槽，所以两次整理运行之间必须隔着用户改动，不能连着跑到环转一圈。
- 取值候选：B_gc = 1（丁的附带批）/ 一个小常数（例如 4，等于 D16 已定项 1 里「4 个不同状态」的 4）/ 3S 减 4 减已经过去的发布数（现算）。现算是显式可计算的判据（硬要求 1），常数要在 S = 4 时仍成立（环 12 条）。
- 观测：每次运行报「用掉几个根槽」「之后回退候选集里不同状态数」。
- 这条预算与 G4 的 W2 令牌的上限 W（单位 txg）是同一个量纲：W 不应超过根环长度。

### 代价

| 项 | 要动的文件 | 新写出的字节 |
|---|---|---|
| 会话入口与返回值 | `crates/singlefs-core/src/mounted_session.rs`（`compact` 入口，丁再在 `publish_user_change` 尾部加附带一批，约 :1049 后） | 0（批的字节见 G3） |
| 挂载读活意图 | `crates/singlefs-core/src/mount.rs`（乙另加一段，丁与甲不需要） | 0 |
| 只供测试的开关 | `CompactionSwitches`（G6）；加触发方式的强制：显式、用户改动后、强制 | 0 |

### 推荐与依据

推荐甲 + 丁，不做乙和丙。依据：甲是丁的手动版，也是测试入口；丁补上 D3 已定项 14 的「自动」，不加线程。

### 会推翻它的现象

- 丁让一次用户改动的延迟变长到用户可见（里程碑六才答 fsync 尾延迟）⇒ 附带批只在会话空闲时跑；会话没有空闲信号，**复核不了**。
- 用户读 D3 已定项 14 的「自动」为「线程」⇒ 丁不算数，要提前做丙。

### 要交用户的岔路（不带倾向）

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 甲 / 甲 + 丁 / 乙 / 丙 | 用户改动的延迟分布；挂载耗时；用户可见 ENOSPC 次数 | QEMU 上 E10 长跑量三项；丙要里程碑六的并行形态先定 |
| GC 被空间拒：g1 停 / g2 推抬 F | 回退深度（可退到的不同状态数）与 GC 完成率的交换 | 同一条老化轨迹两种各跑一遍，数这两个量 |

## 十一、G9 「区域扫描」是什么

### 判定

可定义。里程碑五要接手的两句话是：「走里程碑四 GC 落地的那套意图机制与区域扫描，不新增 `CommitStep` 成员」（`.claude/kb/milestone/05-fifth-txn.md:7`）和「全死的容器由区域扫描回收」（`.claude/kb/milestone/05-fifth-txn.md:19`）；D27 的依据是「全死的容器由后台整理的区域扫描发现」（`.claude/kb/decisions/27-小数据打包容器.md:12`），同行的判活办法是「释放判定走中央映射逐槽点查」。里程碑四二·1 开工前要先定的一条写着「二·1 要定它指什么、交不交」（`.claude/kb/milestone/04-fourth-txn.md:67`）。

### 一句话定义

区域扫描 = 对一个区域 R（设备 + 起始槽 + 槽数）按槽升序逐个看：读这个槽的分配记录；记录未释放的，读单元头得到映射 key；再拿这个 key 查中央映射、核映射里的位置是不是这个槽；据此把每个已分配的槽分成四类。它是 G2 `choose_region` 之后、`plan_batch` 之前的那一步，G2 把它单列为 `scan_region`（有 I/O，不是纯函数）。

### 输入与输出

输入：
- R；该设备的分配记录（按槽升序，现行版本的 `TransactionOutput.allocation_records`，`crates/singlefs-core/src/transaction.rs:2854`）；
- 中央映射条目（挂载态整片读进来的那份，查找是线性的，O2；扫描时先建一次「位置到 key」的反向索引，O(条目数)）；
- 钉住的槽集合：现行根记录里的树表、映射根、实例表链的指针，加映射树各节点的指针（G2）；
- 读单元头的读者：`crate::unit` 里已有 `parse_data_unit`、`parse_index_node`、`parse_packed_unit`（`crates/singlefs-core/src/unit.rs:529`、`:390`、`:463`）。

输出 `RegionScan`，每个已分配槽一项，四类之一：
- 可搬：记录未释放，单元头给出的 key 在映射里，映射里的位置含这个槽，且不是钉住的。带 key、源落点、类标签、诞生代。
- 钉住：被根记录或父指针直接指着的豁免类单元。不搬，只记。
- 已死：记录未释放，但映射里没有任何条目指向它。M4 只报告、不动（一个已分配又没人引用的槽在 M4 里是泄漏，N2 会红）；M5 里一个全死的容器就是这一类，释放它的发布形态归 M5。
- 不一致：单元头读不出、类标签与 key 不符、或头里的 key 与映射里的位置对不上（两个独立来源不一致）。报告并计数，不搬。

已释放或无记录的槽跳过。

### 两个独立来源互核

D21 已定项 1 的定案是单元自带「我属于谁」，所以这个物理位置上是哪个对象，只读那一个单元就答得出（`.claude/kb/decisions/21-权威态与派生态的分界.md:33`）；映射的反向索引是另一个来源。两者必须一致，这一条就是 N2 的运行时版，必红自证：改一个映射条目的位置，或改一个单元头里的 key，扫描必须报「不一致」。

### 对 D27 容器的口子

M4 现有的码 3 容器（inode 叶容器、墓碑、谱系、退役）一个容器一个映射 key，扫描按「一容器一 key」处理。D27 的打包容器是 M5 的新单元类，一个容器装 N 个槽 key（槽里带 key）；那时在 `region_scan.rs` 里加一个分支：容器的 N 个 key 逐个查映射，全不指向它才算已死。M4 不写这个分支（`.claude/singlefs-ai-sop/rules/code-discipline.md`「用不到的代码删掉」），只保证 `RegionScan` 的「已死」类和释放接口在那里等着。

### 代价

| 项 | 要动的文件 | 新写出的字节 |
|---|---|---|
| 扫描函数与分类 | `crates/singlefs-core/src/region_scan.rs`（新）、`lib.rs` 登记 | 0 |
| 单元头读法 | 复用 `unit.rs` 的三个 parse，必要时加一个只返回 key 的小函数 | 0 |
| 单测 | 绝对值：一个 R 里放可搬 / 钉住 / 已死 / 不一致各一个，分类逐项钉；变异：改映射位置、改头 key | 0 |
| 与 M5 的接口 | `RegionScan` 的已死类 | 0 |

### 推荐与依据

推荐如上定义，并把 `scan_region` 单列。依据：D27 已定项 4 与 5 靠它才不用开新结构，M4 把接口交出来就满足里程碑五段 0 的「核里程碑四交出的：整理意图机制、区域扫描」（`.claude/kb/milestone/05-fifth-txn.md:26`）。

### 会推翻它的现象

- 实现发现读单元头的开销太大（R 是 64 槽，每槽读 16 KiB 或 32 KiB，最多 1 MiB 的读）⇒ 改成先只用映射反向索引，抽样核头。
- M5 的 D27 容器要求区域扫描一次覆盖多个段 ⇒ R 的粒度（G2 取一个聚簇段）要放宽。

### 要交用户的岔路（不带倾向）

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 区域扫描读单元头 / 只用映射反向索引 | 一个段扫描的读字节数与耗时；两个来源不一致的发现率 | 在真实现上跑一遍老化后的池，数不一致的段数与扫描耗时 |
| 已死类 M4 只报告 / M4 就释放 | 老化后「已分配而无人引用」的槽数 | 数为 0 则只报告足够；非 0 则那是泄漏要先查 |

## 十二、与背景材料第三节格题面逐格比

| 格 | 材料的说法 | 我推出的 | 判 |
|---|---|---|---|
| G1 | 一条意图是树里一条普通 key 的 value；要装种类、R、`c0`、`segs0`、续做点、暂停标志、销毁快照的四元组；key 怎么排、宽度、定宽还是变宽未定 | 单树定宽联合条目（key 10 + value 34）；变宽与 D8 已定项 11 的头格式冲突；暂停标志不存 | 规则没说（候选已摆），与 D8 已定项 11 一致 |
| G2 | 「输入只从已发布状态读」；R 的粒度是聚簇段 | 目的地候选来自分配器内存里的三张位图，不是已发布状态；输入要加钉住的槽、本次挂载开过的段；前半（区域扫描）读设备 | 与材料不一致，已在 G2 改 |
| G3 | 一批 = 读源 → 原内容写新落点 → 映射条目改 → 分配记录源侧释放代、目的侧分配代 → 源进 defer；问「快照与活头读路径是否都经映射」「`recovery.rs:448` 是不是唯一的位置来源」 | `recovery.rs:448` 不是唯一，还有 :501 的数据单元那一条与挂载态读；三套读者（恢复、挂载态、重建）先提示后映射，豁免三类只按提示；checker 与 GPU 判器不查映射；目的侧「分配代」与 I-3.10 冲突 | 冲突（见 G3 并排抄句） |
| G4 | 停机 `全空聚簇段数(现在) − segs0 ≥ 1`；三水位任一越线；三个量；`N₀` 取最小值 | 占用口径下最后一批那一刻恒假；盘上三个字段叫低 / 高 / 停止线、条款说三个量；合取 7 连整理一起拦会死锁 | 冲突（见 G4 缠一、二、三与名字对照） |
| G5 | 候选乙：GC 产出全空聚簇段那次发布之后整段发 | 那一刻源槽在 defer、老根还引用，discard 抹了老候选根的单元 | 乙冲突（见 G5） |
| G6 | 两条流；每落点 64 次写、每流 2048 次写、展开深度 30；必红世界 | 数对得上；行号三处和工作树不符（已改正）；要先重挂才有可选的段；批大小只能取 1–4 | 一致（行号改正） |
| G7 | Verus 能证计划函数的纯函数性质；模型对拍加一种 `HistoryOperation` | 同；另加抽象状态引理 P4、进展 P3；执行层靠 N6 与对拍 | 规则没说（证什么条款没写），方向一致 |
| G8 | 候选甲：只由会话显式触发 | 与 D3 已定项 14 的「自动」冲突；加丁（用户改动后附带一批） | 冲突（见 G8） |
| G9 | 区域扫描 = 选段函数对一个 R 逐落点读分配记录与映射、认出可搬与已死？ | 是；但它读单元头、有 I/O，要和纯的选段函数、计划函数分成三个 | 规则没说（D26 正文没有这个词），方向一致 |

## 十三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 | 规则没说 | 候选 A：单树定宽联合条目，key 10 + value 34，条目 44、一节点 369 条，新池字节不变；续做点存不存交用户 |
| G2 | 一致（补三处） | 三层：区域扫描（读设备）→ 选段（纯）→ 计划（纯，目的地候选由分配器给）；输入补钉住的槽与本次挂载开过的段 |
| G3 | 冲突 | 第三个发布入口；位置真相要改成以映射为准；目的侧分配代与 I-3.10 当面冲突；checker 与 GPU 判器要学会读过期提示 |
| G4 | 冲突 | 占用口径下产出谓词在最后一批那一刻恒假；合取 7 要只拦前台；盘上水位名字与条款对不上；N₀ 与数值先要计数模型 |
| G5 | 冲突（乙）/ 规则没说 | 乙按字面会抹老候选根的单元；推荐整段发、在发布返回之后、可发谓词为真时；丙与出口 2 冲突 |
| G6 | 一致 | 前缀要重挂；批大小 1–4；九个必红世界；开关用 `CompactionSwitches` 加现成的复用窗口置 0 |
| G7 | 规则没说 | Verus 证 P1–P4 的纯部分；执行层靠 N6、模型对拍、必出反例的世界；证明放现有 crate 或新 crate 交用户 |
| G8 | 冲突 | 甲只给机制，满足不了「自动」；甲加丁；GC 被空间拒时停下、不推抬 F；一次运行的发布数要有上限，否则把用户的回退深度挤出根环（O11） |
| G9 | 规则没说 | 对一个 R 逐槽读记录、读单元头反查 key、查映射，分四类：可搬、钉住、已死、不一致 |

## 十四、复跑与校验

- 四个算术脚本和输出在模型目录 `research/prompts/m4-gc-r1-sonnet-model/`（`arith.py`、`bytes.py`、`n0.py`、`states.py`，`out/` 下四份输出，`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来、只改了 `rerun_steps`）。它们是纯算术，不读仓。
- 我跑的复跑命令与结果：`bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m4-gc-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <空目录> --work <新目录>`，输出四行 `same_as_model_output=true`。
- 模型目录 sha256（`sha256sum` 原样）：`out/arith.out` 01761145cc9a16826bd7b07bdaa2b044296e01ef98b6c8b6eaf61cedcf51e642；`out/bytes.out` b4d695e4a3a53834f5dfbcb6f3b970c1c5ef5322f837a555b0300396d0cf7601；`out/n0.out` d577c160c4f736a715c4756b8da5b9a212009a1fde21eaa033c532ab69a32d29；`out/states.out` 3041215cce75af06e3d11fdb6b7a56e16a0203a7a8e6559a4526f8bb760a4931。
- 引文核对：`PYTHONPATH=/home/fy5090/code/singlefs/.claude/singlefs-ai-sop/scripts python3 scripts/agents/cite-check.py research/prompts/m4-gc-r1-sonnet-output.md --background research/prompts/_m4-gc-r1-background.md`（工作树里没有 `.claude/singlefs-ai-sop/`，脚本顶部要 import 它的 `preflight`，我用主仓里的那份目录补了 `PYTHONPATH`，只读）；末行见交回时的消息。

## 十五、没做什么

- 没判别：不替主 agent 采纳、不出判决；每格的「推荐」是我的正推结论，要交用户的岔路只写了候选、翻面观测、够判条件，不带倾向。
- 没跑 cargo、没起 QEMU、没跑任何实验；凡标「推的」的都是读代码和条款推出来的；G3 的 P、G4 的 c、G6 的 P 都是推的取值，不是量的。
- 没建计数模型：N₀ 的草算公式没有模型验，已在 G4 写明要先建（E109 / E110 形态）再交用户。
- 没读 harness 里全部用例，所以 L-2 对「内存指针与盘上父节点逐字对拍」的影响面只读了发布与重建两段，没有逐个数用例。
- 没读 `.claude/kb/prior-art.md`「十、快照下的死块记账、搬被共享的块与 discard 时机」：它只是线索，不进正推。
- 没核 GPU 判器 WGSL 的实现细节，只数了行数、核了它有镜像（`crash_judge_checker.wgsl:2632`）；N1–N6 在 WGSL 里怎么写没有设计。
- 没替攻方腿找反例；m3-intent-r1 的 F1–F6 只做了逐条重核。
- 没写 `research/prompts/m4-gc-r1-forks.md`；岔路表都在各格里、E10 的岔路单草稿另存在草稿目录。
