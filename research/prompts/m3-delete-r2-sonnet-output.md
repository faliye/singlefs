# m3-delete-r2 辩方腿报告（Sonnet，2026-10-01）

批次：records/2026-10-01-删除与并发事务.md#2。这一条腿只复核第一轮判决 `research/prompts/m3-delete-r1-main-verification.md`，不替主 agent 撤销或维持。
读的是冻结副本 `/tmp/claude-1000/m3-delete-r2/tree/crates/`，引代码的行号都对它（`sha256sum -c` 对清单全 OK）；kb 行号对主仓 `.claude/kb/` 此刻的样子。
我提的改法与收严都只在我的副本模型上量过或根本没实现，算被攻过零轮。

我的模型与产物在 `research/prompts/m3-delete-r2-sonnet-model/`：`m3_delete_r2_defense_cells.rs`（4 个用例）、`core-intent-carry.diff`（在第一轮攻方原型之上多一个「意图那一次」）、
`rerun.sh`、`run-final-d-lines.txt`（17 行 `D name=`）、`cells.py` 与 `cells.out`（逐格值的算式与输出）。下文说「行 N」指 `run-final-d-lines.txt` 的第 N 行。
原型没有意图记录、`deleted_inodes`、墓碑：意图那一次用「extent 与数据单元照抄、inode 1 记录写成 nlink = 0」的一次发布代替；S1 多出来的墓碑单元（码 3，2 槽）与 `deleted_inodes` 根（码 2，1 槽）不在盘上，按槽数另加，下文逐处标「S1 另加」。

## 0　一览

| # | 判决条目 | 辩方判定 |
|---|---|---|
| 1 | 第二节第 2 行：把跑前判据「续做同一批两次盘上字节」判成写错 | **够得着一半**：跨崩溃那一读法打得中、不分辨臂；另一半够不着——同一份镜像各自重挂载续做，盘上逐字节相同（量过），这个读法下判据没写错，换成三条时被一并丢掉 |
| 2 | 第二节第 2 行换上的三条 | **不够**：待删占用减两次三条都过（标签 4、6 今天没有任何读者）；同一落点放两次三条拦得住（代码在施加时自己核） |
| 3 | 第二节第 1 行：S-b「size 不跟着截」出局 | **站得住**：出局理由在 S1 上照样成立；S1 上省得更多（每批 −3 槽、状态 ÷16）但理由没变 |
| 4 | 第二节第 4 行：待删占用口径 A / A2 出局 | **站得住**；S1 第 6 条（R3）记的不是 A / A2，但同形的一击落在 R3 上，除非把「用已扣下的那一份」写成式子 |
| 5 | 惰性建树出局 | **站得住**：回退取根按种类 `expect` 没变 |
| 6 | 判决第五节 K-U4 行 R3 的代价数「约 4.5%」 | **够不着**：13 槽是 K = 2 一个点；K = 144 连续落点 18 槽、散落 264 槽；3 单元文件的预付是它自己数据的 8.3 倍 |
| 7 | 第三节 U5 extent 删空形态（挂起） | `deleted_inodes` 上 E-b 更好；extent 根的 key 区间由位置规定、不用另立约定；inode 树照 kb 的零条目根 |
| 8 | 第三节 U6 流（F1 的状态数） | 第一轮的「约 2 936 万」没有墓碑单元，已过时：S1 是 41 943 084（E-c）或 38 797 356（E-b） |

## 1　第二节逐行复核

### 1.1　第 1 行：S-b 出局

理由有两个：中间态被今天的恢复拒；池级 checker 不红。
- 拒点：`crates/singlefs-core/src/recovery.rs:3672`「fn data_unit_count_matches_the_inode_size(」；攻方的量：`research/prompts/m3-delete-r1-opus-model/run-final-2-e-lines.txt:2`「E name=size_kept step=batch_keep_4 size=179487 units=4 recover=Failed」，同一行末尾 `checker_red=[]`。
- S1 没动这个核对。S1 第 4 条只加读者的答案与写入口的拒，第 5 条改的恢复点是空树分支（`crates/singlefs-core/src/recovery.rs:2453`「what: "inode 树根没有条目",」、`crates/singlefs-core/src/recovery.rs:2506`「what: "inode 树里没有第一个文件那条记录",」、`crates/singlefs-core/src/recovery.rs:3298`「what: "extent 树里第一个文件一个数据单元都没有",」）。S-a 的中间态（size 跟着截）过 3672 那一核不用改；S-b 要放行就得在它的调用处给 nlink = 0 开免核例外，而 checker 没有「单元数对得上 size」，免核的例外没有东西盯。
- S1 上 S-b 省的（量过）：每批不重写 inode 叶（1 单元、2 槽）与根（1 单元、1 槽）。S-a 批 1 的角色表在 `research/prompts/m3-delete-r2-sonnet-model/run-final-d-lines.txt:15`「slots_per_device=13 demand_slots=13 alloc_tree_nodes_rewritten=5」，其中 `InodeLeafContainer:1,InodeRoot:1`；去掉它们是 10 槽、10 单元，写数 24 → 20，两态闭式 2^24 + 11 → 2^20 + 11（÷16，算的）。
- 判：**站得住**。费用不是理由：`.claude/agent-common.md:59`「崩溃点测试准确率和正确率优先，其次再衡量时间成本」。
- 什么现象会推翻：在 3672 的调用处加 nlink = 0 免核例外，同时 checker 加「删除中 inode 的单元数 = ⌈size ÷ 净荷⌉」一条——两个出局理由都失效，S-b 才重新可选。

### 1.2　第 2 行：「续做两次字节相同」判成写错——够得着一半

**判决怎么写的**：跑前判据是「续做同一批两次盘上字节不同 ⇒ 那个形态出局」（`research/prompts/_m3-delete-r1-body.md:60`「续做同一批两次盘上字节不同 ⇒ 那个形态出局」），里程碑三·2 验收是「同一批续做两次盘上字节相同（幂等）」（`.claude/kb/milestone/03-third-txn.md:84`「同一批续做两次盘上字节相同（幂等）」）。判决依攻方的量判「打中不分辨臂」，换成计划相等、再续做不写、终态逻辑相同三条。

**攻方量的是什么**（`research/prompts/m3-delete-r1-opus-model/run-final-2-e-lines.txt:9`「E name=redo_after_crash instance_first=1 instance_redo=2 txg_first=6 txg_redo=8」）：一边是原进程里（实例 1、txg 6）做完的一批，另一边是重挂载之后（实例 2、txg 8）做的同一批；12 个重写单元 `rewritten_units_byte_equal=0`。差别来自实例代号与 txg，写进每个单元头。

**够得着的一半**：这一读法下任何「一批 = 一次发布」的形态都一样，崩溃之后换实例代号是 D18 已定的行为（`.claude/kb/decisions/18-块里携带什么信息.md:312`「作废一个 checkpoint 必须换实例代号」），不分辨臂；判据在「原进程里的一次 vs 重挂载之后的一次」上不可能满足。判决对这一半的处置（不拿它判臂、判据另改）对。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 的第一行「打中不分辨臂」办。

**够不着的一半**：判据字面是「续做同一批两次」，攻方比的两边里第一边不是续做（是原来那次发布）。我量了字面读法：同一份「意图已提交」的镜像，各自重新挂载、各自续做同一批（留 2），两份盘比较：
`research/prompts/m3-delete-r2-sonnet-model/run-final-d-lines.txt:5`「D name=redo_from_the_same_image images_byte_equal=true outputs_equal=true instance_first=2 instance_second=2 txg_first=8 txg_second=8」。
同一份镜像进去，取到同一个实例号、同一串 txg，盘上逐字节相同。攻方自己量的 `redo_same_instance images_byte_equal=true`（`research/prompts/m3-delete-r1-opus-model/run-final-2-e-lines.txt:8`）是同实例同分配器副本，我这条走的是挂载路径，所以更完整。
所以这个读法下判据**没写错**：能满足、能测、比「计划相等」强（落点与单元头字节也比）。判决把它连同跨崩溃那一半一起换成了三条，同一镜像的确定性没人守。

**为什么值得留**：今天 core 的计划路径里没有无序集合（`grep -n "HashMap\|HashSet"` 对 `transaction.rs allocator.rs extent_tree.rs inode_tree.rs mount.rs recovery.rs` 输出为空）。将来有人在续做计划里引入无序迭代：① 计划相等（角色集合、落点集合）仍等，③ 终态逻辑内容仍同，只有同镜像逐字节比会红。
**收严（我提的，零轮）**：三条之外加 ①b「同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同」；三·2 验收那一格改写成 ①b + 三条，不是只剩三条。

**同类先例**：把判据从「字节」换成「逻辑」的已落地案例我没找到；能贴的只有上面 evidence-discipline 那张表（「打中不分辨臂」「打中归错了判据」两行）。
**什么现象会推翻我这半句**：`mount_writable` 在同一镜像上取的实例号或 txg 不确定（引入时钟或随机数），①b 就不可满足，字节判据才真写错。我只量了一个样本（3 单元文件、留 2 一批），一次不算数。

### 1.3　第 3 行：按记录条数分批不收

判决的理由：删除的一批不重写数据单元，记录条数不随被删单元数长。我量了一个 K = 144 的批：`research/prompts/m3-delete-r2-sonnet-model/run-final-d-lines.txt:9`「step=batch_1_keep_156」的角色表合计 17 个单元，对 `crates/singlefs-format/src/lib.rs:206` 一条记录点名 67 项，一条记录；K = 1 的批 12 个单元（行 15）。单元数几乎不随 K 变。判：**站得住**。
什么现象会推翻：某一批的重写角色多到超过 67 项（分配记录树节点散落到几十片叶，行 4 的一批是 255 个分配记录树节点）——那一批要跨记录，记录条数就随散落程度变，但仍不随被删单元数线性长。

### 1.4　第 4 行：待删占用口径 A / A2 出局

- 出局理由是「已分配」不减、释放只进 defer（`crates/singlefs-core/src/allocator.rs:556`「self.deferred_slots += span;」），A 在意图那一步每盘多扣 196 608 字节（`research/prompts/m3-delete-r1-opus-model/run-final-2-e-lines.txt:56`「E name=double_count step=intent pending_kind=A_unreleased_extents demand_equals_free_slots=true admitted=false」）。S1 第 6 条记的是「将来各批要新写的最坏字节」，不是尚未释放的 extent 字节，所以 A、A2 的出局理由在 S1 上没被用到，也没被推翻。
- **同形的一击落在 R3 上**：S1 第 6 条写「删除各批不再按普通准入判，用已扣下的那一份」，没有式子。最朴素的实现是：准入读数里标签 4、6 已扣，需求照旧是这一发布新写的角色（`crates/singlefs-core/src/transaction.rs:5125`「let demand = demand_of_the_roles_on_each_device(」，读数在 `crates/singlefs-core/src/transaction.rs:5137`「&admission_reading_before_a_publish(allocator, previous),」）。在「写入之后可用恰为 0」的池上这个实现每一次删除发布都被拒（第 3.4 节表）——量出的现象与 A 的 `admitted=false` 是同一个形状，跑前判据「某次删除发布在已经预付的池上被准入拒 ⇒ 那个式子不收」字面满足。
- 同样打中别的臂？R2（意图那次一次扣住）同；R0 与 E 没有预付，不受这一击，但满池删不动（`research/prompts/m3-delete-r1-opus-model/run-final-2-e-lines.txt:4`「E name=full_pool_delete_batch result=Err(SpaceAdmissionRefused」）。
- 判：A、A2 的出局**站得住**；R3 要先写「抵扣」式子（第 3.3 节）才不被同一击打中。
- 什么现象会推翻：`mark_released` 改成释放时就减「已分配」（A 不再扣两次，攻方自己写的推翻条件）；或入库装置上朴素实现的 R3 在写后可用为 0 的池上放行了删除（说明读数里的标签已被别处抵过）。

### 1.5　第 5 行：删掉 inode 1 之后恢复拒

量的行在（`research/prompts/m3-delete-r1-opus-model/run-final-2-e-lines.txt:52`「E name=recover step=batch_keep_0 outcome=Failed」、`:54`「E name=writable_mount step=batch_keep_0 result=Err(Recovery(UnitMalformed」、`:53`「E name=read_only_mount step=batch_keep_0 result=Ok」）。判：**站得住**。
提醒一处行号漂移：判决与两份腿报告引的 recovery.rs 行号是第一轮副本的，冻结副本里「extent 树里第一个文件一个数据单元都没有」在 3298（第一轮写 3365）、「inode 树根没有条目」在 2453（第一轮 2452）、「inode 树里没有第一个文件那条记录」在 2506（第一轮 2505）。第二轮要改的位置按冻结副本。
什么现象会推翻：入库装置上删到 0 个单元之后 `recover` 不报 `UnitMalformed`（`extents_of_the_first_file` 不再拒空的单元表）。

### 1.6　惰性建树（第三节 U1 建树时机）出局

理由：回退按种类在目标那一版里找条目、找不到就 `expect`（`crates/singlefs-core/src/mount.rs:4963`「.expect("重建带文件的一版时判过树表里 extent 树与 inode 树各有一条」」），回退只换 extent、inode 两种树的根（`crates/singlefs-core/src/mount.rs:4972`「root: if entry.kind == TREE_KIND_EXTENT || entry.kind == TREE_KIND_INODE {」），发号次序名单按固定 7 种核（`crates/singlefs-core/src/recovery.rs:2188`「const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 7] = [」）。S1 没动这三处，理由原样成立：惰性建下，R_old 若在第一次 unlink 之前，目标那一版没有这条，新根的树表要「删一条」。判：**站得住**。
零根注册的成本也要写进来（判决没写）：树表 7 → 8、水位 19 → 20，所有「树表第 1 版 7 条」的断言与字节表跟着改（事实表 `research/prompts/m3-delete-r1-facts.md` (a) 节 a3 列了）。这笔成本每个池都付，哪怕从不删。
什么现象会推翻：回退改成对新树的种类也按种类取目标那一版的根、找不到时写零根——那时惰性建与零根注册的差别只剩树表字节。
与第 5 节 E-b 的关系：空队列取「根指针置零」时，惰性建树的第一次 unlink 与零根注册的第一次 unlink 在盘上是同一件事（树表条目的根从零变非零），两者的差别只剩「条目哪个版本起有」。

## 2　第三节「采纳」各行：有没有同样打中所有替代方案

我没有逐行重做的不写判定，标「没复核」。

| 行 | 辩方判定 | 证据与推翻条件 |
|---|---|---|
| U1 key：8 字节、无出生代、无 value | 站得住（没找到反例） | 号不复用靠水位只加不减（`crates/singlefs-core/src/transaction.rs:6266`「let inode_number_watermark = match previous {」）；前提「随所选根」归 V4 |
| U1 建树时机 | 站得住 | 第 1.6 节 |
| U1 回退取 R_old | 不归我（攻方 V4） | 第 3.6 节说明标签 4、6 做成版本的函数时 R_old 重算免费 |
| U1 冻结层与态别（派生态、第 3 / 4 层） | 站得住，带一个条件 | 第 6.1 节 |
| U2 墓碑 | 不归我（m3-tomb-r1） | 接口：意图那次多一个码 3 墓碑单元（2 槽），算进同一份预付（第 3.5 节） |
| U3 方向与 size（S-a） | 站得住 | 第 1.1 节 |
| U3 粒度：个数分批、只供测试的旋钮 | 站得住 | 第 1.3 节；旋钮对预付的影响第 3.4 节 |
| U3 粒度：产品默认 144 | 自由选取，不是推出来的 | `crates/singlefs-format/src/lib.rs:156`「pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;」；W 随 K 变（第 3.1 节），没有「与 K 无关」的批成本 |
| U3 只有一条路 | 站得住 | 代价：1 个单元的文件也是意图 + 1 批两次发布，预付约 25 槽对数据 2 槽（第 3.5 节最后一条，推的） |
| U3 nlink | 站得住 | 偏移 36；今天写死 1：`crates/singlefs-core/src/records.rs:56`「writer.put_u32(1); // nlink」 |
| U3 续做挂点 | 不归我（V3） | — |
| U4 口径 A / A2 | 站得住 | 第 1.4 节 |
| U4 删除自己要空间 | 站得住（攻方量过） | 我量的各批需求：意图 14（含 S1 另加 3）、批 1 13、批 2 12、批 3 11 槽每盘（第 3.1 节） |
| K-U4 R3 的代价数 | **够不着** | 第 3.1、3.2 节：「13 × 16 KiB ≈ 4.5%」是 K = 2 一个点 |
| U5 恢复 | 站得住；行号漂移 | 第 1.5 节 |
| U5 extent 删空形态 | 见 V5 | 第 5 节 |
| U5 再建带数据的文件（X1） | 没复核 | — |
| U5 模型第三态 | 站得住，改动面比判决写的大 | 第 5.4 节 |
| U5 checker 缺口 | 站得住 | 第 1.1 节（`checker_red=[]`） |
| U6 命名 | 站得住 | `crates/singlefs-checker-tier/src/crash_amplification.rs:2389`「name: format!("{entry_name}.{period}.{step_in_period}"),」 |
| U6 流（F1 的数） | 数已过时 | 第 6.4 节 |
| U6 截断成 0 另立流 | 没复核 | — |
| U7 | 没复核 | — |

表里没写推翻条件的行：推翻 = 对应小节写的现象；指向「没复核」的行不下结论。

## 3　V2：R3 预付的式子

### 3.1　量到的（4 GiB × 2 盘、2 副本、每槽 16 384 字节；需求每盘同数）

需求就是这次重写的全部角色的槽数和（`crates/singlefs-core/src/admission.rs:962`「let slots_of_the_demand: u64 = rewritten_roles.iter().map(|role| role.span_slots()).sum();」），数据单元与码 3 容器 2 槽、码 2 节点 1 槽。下表是每盘新写的槽，S1 另加的两样在最后一列。

| 几何 | 发布 | 量到（行） | S1 另加 | 每盘合计 |
|---|---|---|---|---|
| 3 单元、K = 1 | 顺序写 | 19（行 13） | — | 19 |
| | 意图（extent 照抄、inode 记录 nlink = 0） | 11（行 14） | `deleted_inodes` 根 1 + 墓碑 2 | 14 |
| | 批 1（留 2） | 13（行 15） | — | 13 |
| | 批 2（留 1） | 12（行 16） | — | 12 |
| | 批 3（留 0） | 12（行 17，记录留着） | 删 inode 叶 −2、`deleted_inodes` 根 +1 | 11 |
| 300 单元、K = 144、落点连续 | 意图 / 批 1 / 批 2 / 批 3 | 12 / 18 / 15 / 14（行 8–11） | +3 / 0 / 0 / −1 | 15 / 18 / 15 / 13 |
| 144 单元、K = 144、一单元一片叶 | 意图 / 批 1（留 0） | 15 / 264（行 3、4） | +3 / −1 | 18 / 263 |

散落那一格的造法：把每块盘的空闲槽标成已分配，只留相隔 1 624 槽（两片叶）的两槽空洞与尾部一大片全空段，数据单元按最低空槽取落点，于是一个单元一片分配记录树的叶；分配器拷贝上标、盘上不写（同第一轮攻方的满池造法）。
行 4 `alloc_tree_nodes_rewritten=255`：一批重写了 255 个分配记录树节点。连续那一格同一个量是 5–7（行 9：`AllocationTreeNodeBelowTheRoot:6`）。

**W 随什么变（推的，从上表与格式常量算）**：
- 固定角色：树表 1、记账 1、映射根 1、inode 叶 2、inode 根 1，意图再加 3；extent 路径 2–3；映射树根之下 0–2。合计 K = 1 约 11、K = 144 约 15。
- 分配记录树：每盘被改的记录数 ≈ K（被释放的数据单元）+ 约 26（被换下的节点约 13、这批新取的落点约 13），每条记录最坏各在一片叶里。叶罩 812 槽（`crates/singlefs-format/src/lib.rs:140`「pub const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;」），内部扇出 169（`crates/singlefs-format/src/lib.rs:146`「pub const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;」），这个池每盘 262 片叶、2 个第 1 层节点。两盘的叶各一份，所以上界 = 2 × min(K + 26, 262) + 5：K = 1 是 59，K = 144 是 345（`research/prompts/m3-delete-r2-sonnet-model/cells.out:25`「字面最坏上界（算的）：分配记录树 2*min(K+26,262)+5；K=1: 59 K=144: 345」）。
- 所以字面最坏 W：K = 1 约 70（意图 73），K = 144 约 360。**任何 K 下字面最坏都不小于被删数据本身**：一个被释放的单元在两盘各有一条记录，落在两片叶，每片叶重写要每盘 1 槽，共 2 槽每盘，等于这个数据单元自己每盘的 2 槽。

### 3.2　判决 K-U4 行的代价数够不着

判决写「每批最坏需求在副本上量到 13 槽每盘 … 按批大小 144 推每 144 个数据单元常驻约 13 × 16 KiB = 208 KiB 每盘、约为数据的 4.5%（推的，只一个几何点）」。13 是攻方一批 2 个单元、6 单元文件那一点（`research/prompts/m3-delete-r1-opus-model/run-final-2-e-lines.txt:3`「E name=full_pool demand_slots_of_the_batch=[13, 13]」）。每盘槽数、对数据的比（算式在 `cells.out:21-24`）：

| 文件 | 预付（槽每盘，意图 + 全部批） | 数据（槽每盘） | 比 |
|---|---|---|---|
| 3 单元、K = 1 | 14 + 13 + 12 + 11 = 50 | 6 | 8.3 倍（`cells.out:21`） |
| 300 单元、K = 144、连续 | 15 + 18 + 15 + 13 = 61 | 600 | 10.2%（`cells.out:22`） |
| 144 单元、K = 144、散落 | 18 + 263 = 281 | 288 | 97.6%（`cells.out:23`） |
| 判决的点 | 13 | 288 | 4.5%（`cells.out:24`） |

读法：判决的 4.5% 是「一批的需求 ÷ 一批的数据」，漏了意图那一次、漏了小文件时固定开销压倒数据、也漏了落点散落。D3 要求「预留量动态贴着实际需求，不白占；量要小」（`.claude/kb/decisions/03-空间分配.md:277`「预留量动态贴着实际需求，不白占；量要小」）：字面最坏取法（3.1 的上界）与它冲突，3 单元文件预付 4 × 73 = 292 槽每盘对数据 6 槽，48.7 倍（算的）。
**同样打中别的臂？** 打中所有「写入时预付」的臂（R2、R3）；不打 R0、E，它们的毛病是满池删不动。这是交用户的岔路单 K-U4 一行里该改的数，不改判 R3。
什么现象会推翻：入库装置上同一段历史重量，各批需求与行 13–17 不同；或散落造法造不出「一单元一片叶」（行 4 的 255 个节点是它的证据）。

### 3.3　式子（我提的，零轮）

记号：每盘槽数；记账行按全部副本之和记（`.claude/kb/decisions/28-挂载期承诺量.md:24`「按全部副本之和记字节」），每盘扣 和除以副本数（向上取整，`crates/singlefs-core/src/admission.rs:99`「BytesOnOneDevice(self.0.div_ceil(replicas.replicas()))」）。K 取写入口的有效 K，N = ⌈size ÷ 净荷⌉，批数 B(N) = max(1, ⌈N ÷ K⌉)。
1. **标签是版本的函数，不是计数器**：标签 6 = Σ（nlink ≥ 1 的 inode）P(N_i)，P(N) = W_意图 + Σ_{j=1..B(N)} W_j；标签 4 = Σ（队列里的 inode）Σ_{剩下的批} W_j，剩下的单元数取 inode 记录的 size（size 本来就跟着截）。每次发布从这一版重算，不存、不增减。
2. **W 取法二选一**：(a) 构造式上界（只依赖 K、盘数、副本数、容量，checker 不走映射就能算，代价是量到值的 5 到 20 倍：K = 1 是 73 对 14，K = 144 是 360 对 18，算的）；(b) 按这个文件当前的落点现算（释放的每个落点落在哪片叶，加新取落点的上界；checker 走映射树与分配记录树本来就走，但每次发布重算整个文件）。(b) 才满足「量要小」，(a) 才让 I-3.4 只靠 inode 记录重算。我没有量 (b) 的重算代价。
3. **抵扣（不许重扣）**：删除类发布 j 的准入读数用「本次发布之后」的标签：放行 ⟺ 需求_j ≤ 可用(发布前) + [(T4 + T6)_前 − (T4 + T6)_后] 每盘。W_j 取各发布的最坏，需求_j ≤ 份额_j 恒成立，所以只要写入时判过「可用 ≥ 0」，之后每次删除发布放行，可用不降（3.4 的表）。写入类发布同理：放行 ⟺ 需求_写 ≤ 可用(发布前) − (T6_后 − T6_前)。
4. **接线处**：需求一侧 `crates/singlefs-core/src/transaction.rs:5125`，读数一侧 `crates/singlefs-core/src/admission.rs:1021`「PoolWideCommitments::of_the_first_version(checkpoint_reserve_pool(」（今天写死两个 0，`admission_reading_before_a_publish` 已经收 `previous`，所以标签从 `previous` 算，不用给 `PoolAllocator` 加字段）；记账行取值 `crates/singlefs-core/src/transaction.rs:6283`「PoolWideAccountingRow::PendingDeleteBytes => {」起两处读同一个函数（`transaction.rs:6282` 的注释就是为防两处各抄一个 0 分叉）。可写挂载取号前那一判走同一个读数函数，自动带上标签。先例：每次发布现算、只住内存的量已经有一个（`.claude/kb/decisions/28-挂载期承诺量.md:26`「每次发布重算、不设常数上限、只住内存」，checkpoint 保留池）；标签 4、6 比它多一步——要落进记账行，且从版本重算。

### 3.4　3 单元文件、K = 1 的逐格值

单位：标签是全部副本之和的字节（= 每盘槽 × 16 384 × 2）；可用是每盘字节，取真读数（带实例切换预留与 checkpoint 保留池，`crates/singlefs-core/src/admission.rs:1031`「pub fn admission_reading_before_a_publish(」）再减 (T4 + T6) ÷ 2。每盘无预付的可用取量到的行 13：`research/prompts/m3-delete-r2-sonnet-model/run-final-d-lines.txt:13`「baseline_available_per_device=[3357786112, 3357786112]」，往后每次减这一发布的需求（意图 14、批 13 / 12 / 11 槽，含 S1 另加，算的）。
算式与输出在 `research/prompts/m3-delete-r2-sonnet-model/cells.out:1-12`。

**策略 X：W_j 取各发布自己的需求（14、13、12、11），P = 50 槽每盘**

| 时刻 | 标签 6 | 标签 4 | 每盘可用 |
|---|---|---|---|
| 顺序写 3 单元之后 | 1 638 400 | 0 | 3 356 966 912 |
| 意图之后 | 0 | 1 179 648（剩 13 + 12 + 11 = 36 槽） | 3 356 966 912 |
| 批 1（留 2）之后 | 0 | 753 664（23 槽） | 3 356 966 912 |
| 批 2（留 1）之后 | 0 | 360 448（11 槽） | 3 356 966 912 |
| 批 3（留 0，删记录）之后 | 0 | 0 | 3 356 966 912 |

每一格的可用相同：预付的字节每次发布原样从标签转进「已分配」，没有一格扣两次、没有一格被拒。X 只在「落点连续、这个几何」下是最坏，是 3.1 第一个点的实测值。

**策略 Y：常数 W = 14（取各发布最大），P = 56 槽每盘**

| 时刻 | 标签 6 | 标签 4 | 每盘可用 |
|---|---|---|---|
| 顺序写 3 单元之后 | 1 835 008 | 0 | 3 356 868 608 |
| 意图之后 | 0 | 1 376 256（42 槽） | 3 356 868 608 |
| 批 1 之后 | 0 | 917 504（28 槽） | 3 356 884 992 |
| 批 2 之后 | 0 | 458 752（14 槽） | 3 356 917 760 |
| 批 3 之后 | 0 | 0 | 3 356 966 912 |

多预付的 6 槽（98 304 字节每盘）到末批才退回。常数 W 是 checker 能不碰落点重算的前提（3.3 第 2 条）。

**紧池上朴素实现与抵扣的对比**（写入之后可用恰为 0；算的，`research/prompts/m3-delete-r2-sonnet-model/cells.out:13-17`）：

| 发布 | 需求（槽每盘） | 发布前可用 | 朴素（按发布前读数判） | 抵扣（按发布之后的标签判） |
|---|---|---|---|---|
| 意图 | 14 | 0 | 拒 | 放行 |
| 批 1 | 13 | 0 | 拒 | 放行 |
| 批 2 | 12 | 0 | 拒 | 放行 |
| 批 3 | 11 | 0 | 拒 | 放行 |

朴素实现四次都拒：跑前判据「某次删除发布在已经预付的池上被准入拒 ⇒ 那个式子不收」字面满足，所以 R3 只收带抵扣的式子。我没有在入库装置上实现准入，这张表是算的。

**K 旋钮**：第一轮正推腿 4.4 节第 2 条建议预付按产品 K（`research/prompts/m3-delete-r1-sonnet-output.md` 里写「K 不进待删占用的式子」那一条）。按产品 K = 144 预付 3 单元文件：B = 1，P = 14 + 13 = 27 槽；用旋钮 K = 1 删，同一个紧池上意图与批 1 放行（各抵 14、13），批 2 需求 12、份额 0、可用 0 ⇒ 拒。所以**预付按实际生效的 K**（写入口属性、挂载期间常量，`.claude/rules/fs-design.md` 的硬要求 5 同一条），产品 K = 144。代价：标签值依赖一个不在盘上的 K，checker 重算要收 K 作参数（产品镜像默认 144，崩溃放量用旋钮时同传）。入库镜像里 K 不会变；换一个 K 挂载同一个池，标签与重算值对不上，只在测试里可能发生。

### 3.5　其余问句逐条

- **单位**（C375）：按全部副本之和记，每盘 ⌈和 ÷ 副本数⌉；第一版每个单元落每块盘，所以和 = 每盘需求 × 副本数（`crates/singlefs-core/src/admission.rs:949`「/// 一次发布的需求：这次新写的全部角色」那一段）。
- **哪几个入口、哪一步加**：`publish_first_file`（`crates/singlefs-core/src/transaction.rs:4624`「pub fn publish_first_file<Device: BlockDevice>(」，N = 1：标签 6 从 0 到 P(1)）、`publish_sequential_write`（`crates/singlefs-core/src/transaction.rs:4772`「pub fn publish_sequential_write<Device: BlockDevice>(」，标签 6 := P(N_新)）、`publish_overwrite`（`crates/singlefs-core/src/transaction.rs:4719`「pub fn publish_overwrite<Device: BlockDevice>(」，单元内内容，:= P(1)）。三者都收进同一条路 `prepare_the_version_publish`，在需求算定、读数取定之后、动分配器之前加（接线处见 3.3 第 4 条），不在入口各加一遍。
- **文件变短退多少**：不单独记退。标签 6 每次从这一版重算，变短那一发布里 P(N_旧) − P(N_新) 自然退掉。
- **意图那次怎么转**：同一发布里标签 6 退整份 P、标签 4 记剩下各批（X 策略 P − W_意图），净减 W_意图 = 这次发布自己新写的量，与「已分配」同一发布里增加的量相等（上表意图一行可用不变）。
- **各批的需求与已扣的那一份怎么抵、不许重扣**：3.3 第 3 条；恒等式是每发布「已分配」增 D_j、标签 4 + 6 减 W_j，D_j ≤ W_j，三项之和单调不增。
- **I-3.4 与已承诺预留的值 checker 能不能只从镜像重算**：能算记账行，不能照字面判 I-3.4。(i) checker 今天不读标签 4、6：`crates/singlefs-checker/src/walk.rs:116`「const STATISTIC_INODE_WATERMARK: u16 = 12;」是它认的常量里与池级三行相关的唯一一个，取值点只有标签 12、1、2、5；恢复只判标签 12 那一行在（`crates/singlefs-core/src/recovery.rs:2597`「InodeNumberWatermarkRowMissingFromTheAccountingTree」）。(ii) 已承诺预留没有任何不变量（`grep -n "已承诺预留" .claude/kb/invariants.md` 输出为空）。(iii) I-3.4 的字面「可用空间统计」盘上没有这个字段：可用只在内存里按式子算（`crates/singlefs-core/src/admission.rs:357`「可用(d)，逐盘，按读数里盘的次序：」），所以 I-3.4 只能改写成「标签 4 == 重算值」「标签 6 == 重算值」两条。(iv) 重算要的输入：队列里的号、每个 inode 记录的 size 与 nlink（都在盘上）、盘数与副本数与容量（系统配置）、K（不在盘上，参数）；W 取 3.3 第 2 条的 (a) 才不碰落点。还要什么输入 = K 与 W 的取法写成条款。
- **墓碑那个单元算不算进同一份预付**：算。它在意图那次写，是「解开自己」的一部分（`.claude/kb/decisions/03-空间分配.md:52`「删除路径仍然不申请空间」），3.1 的意图一行已含 2 槽；`deleted_inodes` 根另加 1 槽。形态归 m3-tomb-r1，这里只定接口：墓碑每多一个单元，W_意图加它的槽数。
- **一格 S1 没写**：`publish_new_inodes` 建的 size 0 inode 能不能 unlink。能，则建的时候就要预付 P(0) = W_意图 + 一批 ≈ 25 槽每盘（14 + 11，推的）：建 1 000 个空 inode 常驻约 410 MB 每盘（25 × 16 384 × 1 000，算的）；不能，则 unlink 要拒 size 0 的号。

### 3.6　与 V4 联动

标签 4、6 做成版本的函数之后，回退「按 R_old 的状态重算」不用另写重算逻辑：回退那次发布装记账行时标签 4、6 从 R_old 那一版算（`crates/singlefs-core/src/mount.rs:4989` 一带今天只换水位那一行、别的原样抄）。反过来，标签做成计数器就得在回退处为它另写一条取 R_old 的路。

## 4　V1：三条判据够不够判 D8 已定项 5 的「幂等」与 I-4.4

D8 已定项 5 的硬要求写「意图必须幂等」（`.claude/kb/decisions/08-核心索引结构.md:109`「意图必须幂等」），第 4 条把幂等落成「施加时先核它要删的东西还在」，并点名后果：「两个区间重叠的级联会把同一块放两次，统计量减两次」（`.claude/kb/decisions/08-核心索引结构.md:124`「两个区间重叠的级联会把同一块放两次，统计量减两次」）。I-4.4 只管意图 key 的字段不含增量（`.claude/kb/invariants.md:168`「| I-4.4 | 意图只描述目标态 |」）。

### 4.1　每条能抓什么、抓不到什么

| 判据 | 抓得到 | 抓不到 |
|---|---|---|
| ① 计划相等（同一份上一版、队列、K 算两次，重写角色与释放落点集合相等） | 计划依赖隐藏状态 | 计划以外的副作用；记账行的值不在「角色与落点」里 |
| ② 已完成的删除再续做一次，一个写都不发 | 完成态上还在写 | 中间态上的重复施加（队列非空时它不触发） |
| ③ 每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同，每个崩溃态 checker 全绿 | 内容级的重复（extent、inode、队列） | 终态会抹平的量；checker 不读的量 |

### 4.2　三条都过的非幂等续做：有（待删占用减两次），同一落点放两次没有

**待删占用减两次，三条都过。** 构造：把「标签 6 退 P、标签 4 加 P − W_意图」这一步挂在续做入口而不是挂在意图那次发布上（入口见到队列里有号就做一次），每多一次中间态的挂载标签 4 就多加一次；最后一批把标签 4 写成 0。
- ① 过：角色与落点的计划不含记账行的值。
- ② 过：完成的删除队列已空，入口不触发。
- ③ 过：终态标签 4 = 0，与不崩溃的终态相同；每个崩溃态 checker 全绿，因为 checker 根本不读标签 4、6（取值点只有标签 12、1、2、5，`crates/singlefs-checker/src/walk.rs:116`「const STATISTIC_INODE_WATERMARK: u16 = 12;」一带；恢复只判条数与水位那一行在，`crates/singlefs-core/src/recovery.rs:2597`「InodeNumberWatermarkRowMissingFromTheAccountingTree」），`crates/singlefs-core/src/recovery.rs:3505`「记账条目数不是 3 + 6 × 盘数」一类只判条数。
所以 D8 第 4 条点名的「统计量减两次」对标签 4、6 没有任何判据盯着。判决第二节的跑前判据 V1「找到一种非幂等的续做而三条判据都过 ⇒ 判据要收严」字面满足。

**同一落点放两次，三条里 ③ 抓得住，不是都过。** 代码在施加时自己核：`crates/singlefs-core/src/transaction.rs:3220`「return Err(PublishError::ReleaseTargetAlreadyReleased {」、`crates/singlefs-core/src/allocator.rs:1302`「同一个落点释放了两次」。续做在这里报错，③ 的「续做到底」就走不完；若将来有人拿掉这两道核，defer 计数多加一次，I-3.11 红（推的，没跑；`.claude/kb/invariants.md:144`「| I-3.11 | 已分配减 defer 等于最新根走读 |」）。这一类是代码守住的，三条判据是它的第二道。

### 4.3　收严（我提的，零轮）

加第 ④ 条：**每个崩溃态、每次重开续做之后的每个发布之后，记账行的标签 4、6 等于从这个镜像重算的值**，由 checker 判（I-3.4 改写成「标签 4 == 重算值」，已承诺预留另立一条，3.5 第 6 条说明为什么要改写）。判别力自证：把续做入口的转换改成每次挂载做一次，④ 必须红。我没有做这个变异（checker 里没有这两条）。
再加 ①b（第 1.2 节）：同一镜像各自重挂载续做，盘上逐字节相同。

### 4.4　「逻辑内容」写死

| 含（逐项相等） | 不含 |
|---|---|
| inode 树每条记录的（号、对象出生代、size、nlink）；write_time 若由用户输入定则含 | 实例号、txg、改动计数（`research/prompts/m3-delete-r1-opus-model/run-final-2-e-lines.txt:9` 的 `change_count_first=6 change_count_redo=8`） |
| extent：每个 inode 的单元序号集与每个单元数据指针里的内容校验和 | 落点与槽号、位置条目、出生序号、nonce |
| 删除队列的 key 集 | 单元头里的诞生代号、写序、校验和（重写的节点逐个不同） |
| 墓碑记录的**多重集**（对象号、出生代、区间；不含容器头里的 txg 与写序）——多重集才看得出重复写 | journal 记录字节、根槽与系统配置槽、实例表的行 |
| 记账池级三行：标签 12、4、6 逐值 | 逐盘标签 1、2、5、10、11：随 txg 个数与根环推进（回收时机）变，改判它们在每个崩溃态满足 I-3.1、I-5.2、I-3.11（checker 已判） |
| 被逻辑释放的数据单元序号集 | 被释放落点的集合与释放代 |
| 读者的答案：冷走读、只读挂载 `open_file`、模型读回在每个崩溃态一致（从意图提交那一步起「没有文件」） | — |

**什么现象会推翻 4.2**：入库的 checker 已经读标签 4、6，或恢复已经把它们对着重算值判；那时「待删占用减两次」被 ③ 的 checker 全绿抓住。
**同类先例**：D8 第 4 条本身（上面引的）就是先例，说的正是统计量减两次；把它落成「记账行等于重算值」的已落地检查我没找到（I-3.4 状态是未实现，`.claude/kb/invariants.md:139`「| I-3.4 | 可用空间扣待删占用 |」）。

## 5　V5：删空形态与恢复

### 5.1　三棵树各取什么（我提的，零轮）

| 树 | 取 | 理由 |
|---|---|---|
| inode 树 | 零条目根（照 kb） | D8 已定项 6 已写（`.claude/kb/decisions/08-核心索引结构.md:169`「唯一的叶被删空时是不落盘、根变成零条目」），I-9.13 已实现（`.claude/kb/invariants.md:287`「唯一的叶被删空时不落盘、根变成零条目」）；`crates/singlefs-core/src/recovery.rs:2453`「what: "inode 树根没有条目",」要放行 |
| extent 树 | E-c 零条目根；退路 E-d | E-c 同 inode 树一条规则；要改 `crates/singlefs-core/src/extent_tree.rs:664`「"extent 树节点一条条目都没有（缺席的一段不写节点）",」（只对根放行）与规划那一支（没有条目 ⇒ 层级 0，推的）。E-d：上段叶条目留标签 0「没有单元」，格式里已有、读者今天就走（`crates/singlefs-core/src/mounted_read.rs:654`「Some(ExtentUpperLeafTarget::NoDataUnit) => DataUnitsOfTheInode::NoneWritten,」），读者零改动，代价是已死 inode 的叶条目永远留着 |
| `deleted_inodes` | **E-b 根指针置零** | 见下 |

**`deleted_inodes` 取 E-b 的理由（S1 第 5 条写的是零条目根，我不同意这一棵）**：
1. S1 第 3 条让它第一个文件版本就零根注册，可写挂载续做、恢复与 checker 每次都要读这棵树，所以读者的零根分支（`crates/singlefs-core/src/mounted_read.rs:414`「&& entry.root != NodePointer::empty_root())」今天对零根报 `TreeTableHasNoTreeOfKind`）本来就要加；E-b 不多一个分支，E-c 要再加一个「零条目根」的分支。
2. 空队列在盘上只有一种样子：E-c 下是「零根（建树后到第一次 unlink 前）」与「零条目根（第一次 unlink 删完之后）」两种，恢复与 checker 各认两种，I-9.5 与 I-4.7 的判定写两遍；E-b 回到建树时的零根，一种。
3. 最后一批少 1 个单元：n 22 → 20，两态闭式 4 194 315 → 1 048 587（`research/prompts/m3-delete-r2-sonnet-model/cells.out:18-19` 的 F1 总数差 3 145 728）。
4. 不用定头里 key 区间。
**先例**：livelist、稀疏旁表、deadlist 三棵树根指针恒零（`crates/singlefs-core/src/transaction.rs:6508`「TREE_KIND_LIVELIST,」起三条，值是 `NodePointer::empty_root()`），指针层面的定义在 `crates/singlefs-core/src/pointer.rs:226`「没有根的树：整条指针全 0」。
**反方**：第一轮正推 5.2 节「一条规则罩三棵」的整齐；E-b 下 inode 与 extent 是零条目根、`deleted_inodes` 是零根，两种空形态各归各的树。我没有量 E-b 的恢复改动面。
**什么现象会推翻**：`deleted_inodes` 改成惰性建（建树前树表里没有这一条）：那时「建树前」与「删空之后」本来就不是同一种盘上形态，理由 2 不再成立，E-c 也可；或读者零根分支要为别的原因另写、与「零条目根」共用。

### 5.2　零条目根的头里 key 区间写什么

- **extent 根**：不用另立约定。节点头的区间是位置规定的那一段（`crates/singlefs-core/src/extent_tree.rs:120`「节点头里写的 key 区间：」），上段层级 0 第 0 个节点罩 inode `[0, U − 1]`，U = 143（`crates/singlefs-format/src/lib.rs:163`「pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;」），即 `[(0, 0, 0), (0, 142, u64::MAX)]`；读时核的也是它（`crates/singlefs-core/src/extent_tree.rs:659`「"extent 树节点头里的 key 区间不是它的位置规定罩的那一段",」）。
- **inode 树根**：checker 对空节点的区间不设限（`crates/singlefs-checker/src/lib.rs:597`「区间贴紧首末条目（空节点区间任意）」），所以写什么都过；要选一个常量（字节才与历史无关，①b 才成立）。我提 `[0, u64::MAX]`：根罩整个 key 空间，先例是分配记录树的根（`crates/singlefs-core/src/transaction.rs:2026`「分配记录树的根（罩整个 key 空间」）。恢复读 inode 根时核不核区间我没读到，行号待查。
- **`deleted_inodes` 根**：E-b 下没有节点，没有区间。

### 5.3　恢复认「没有文件」的判定

写成：树表条目齐全（今天 7 种加 `deleted_inodes` 共 8 条）∧ inode 树根零条目 ∧ extent 上段根零条目（或只有标签 0 条目）∧ `deleted_inodes` 零根 ∧ 记账里的水位行在且 ≥ 2；中间态再加两半（I-4.7，`.claude/kb/invariants.md:172`「两者皆非的中间态不存在」）：有 extent 无队列条目、有队列条目无 nlink = 0 的记录，各是一个坏镜像。
与 I-9.13 对得上：I-9.13 管的是打包记录类型 2 的**容器**（叶）记录数 ≥ 1，空的是码 2 的根节点，不是容器；checker 本来就容忍空节点（上一条引的 `lib.rs:597`）。
**不能映射到今天的「没有文件」**：`RebuiltVersion::WithoutFile` 的注释是「树表 0 条」（`crates/singlefs-core/src/recovery.rs:2271`「/// 树表 0 条：所选根下面还没有发布过文件版本」），建在它上面的路径假定树表是 mkfs 写的（`crates/singlefs-core/src/mount.rs:1658`「根记录指着的实例表或树表不是 mkfs 写的那一版（诞生 txg 不是 0）」）、没有记账树（`crates/singlefs-core/src/mount.rs:3889` 同一行「这一版没有记账树」）。删完之后的一版有记账树、水位在里面，丢了水位号就重发（C479）。所以要第三个成员（有树无文件），落在 `RebuiltVersion`、`PreviousVersion` 与 `PoolVersion`（`crates/singlefs-core/src/mount.rs:3992`「PoolVersion::WithoutFile(_) => panic!(」那种 `match` 逐处加臂）。
**什么现象会推翻**：删完之后改成回到树表 0 条（水位另找地方放，格式改动）。

### 5.4　理想模型的第三态

`ModelRoot::file` 是 `Option`，注释写「树表 0 条（第 0 代根、暖机根、零单元发布）时没有文件」（`crates/singlefs-harness/src/model.rs:199`），于是 `file.is_none()` 同时当「没有树」用：`model.rs` 里 `file.is_none()` 与 `file.is_some()` 共 11 处（`grep -c` 输出 11），选角色表（`ModelPublishKind`）、答读回、比对观测都靠它。第三态不需要新角色表：`EmptyOnFileVersion` 的注释本来就是「树表不是 0 条时的空发布」（`crates/singlefs-harness/src/model.rs:106`「/// 树表不是 0 条时的空发布（暖机、抬 F）：四个固定点单元」）；要改的是选择键从「有没有文件」换成「树表是不是 0 条」，11 处逐个判。判决 U5 模型一行写「理想模型加第三态」，改动面按这个数写。

## 6　V7：其余零轮项

### 6.1　`deleted_inodes` 的态别与冻结层：站得住，带一个条件

判据是 D21 已定项 10：「重建不出这个值时，读者拿到的是保守答案还是错答案」（`.claude/kb/decisions/21-权威态与派生态的分界.md:216`「重建不出这个值时，读者拿到的是保守答案还是错答案」）。S1 第 1 条里意图那次同一发布写 nlink = 0 与队列 key，所以队列恒等于「inode 树里 nlink = 0 记录的号集合」，丢了能重算；重算不出时（假想）读者拿到的是「删除永远不完成」的泄漏，不是错答案。⇒ 派生态、第 3 / 4 层、退出冻结，与第一轮正推一致。
**条件**：重建规则要成文（一句：队列 = inode 树里 nlink = 0 的号集合）。livelist 三行判不动的原因就是这一句没人答过（`.claude/kb/freeze-layer-membership.md:85`「没有任何一条分项答过这一句」）；`deleted_inodes` 同样没有分项答过，不补这一句就落回判不动。
**推翻**：以后加第二种不带 inode 号的意图并复用这棵树——那时队列不再能从 nlink = 0 记录重算，态别要改（K-U1 选 L4 时已写明以后再加意图要再加一棵树，所以现在不成立）。
同类先例：livelist / 稀疏旁表 / deadlist 的三行（`.claude/kb/freeze-layer-membership.md:85`）是「判不动」的先例，没有「派生态」的先例可贴，找不到。

### 6.2　批大小产品默认 144

`crates/singlefs-format/src/lib.rs:156`「pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;」是「一片下段叶罩的单元数」，不是从成本推出来的：每批需求 K = 1 是 13、K = 144 连续是 18、散落是 264（第 3.1 节）。判：**自由选取，站得住**（旋钮才是硬要求）；不要在正文里把它写成「由 W 定」。推翻：把散落的 W 当产品场景去量，K = 144 的预付超过可接受的比（3.2 的 97.6%），那时要重选 K。

### 6.3　「只有一条路」与入口名带批号

- 只有一条路：站得住。代价：1 个单元的文件也是意图 + 1 批两次发布；N = 0（size 0 的 inode）也是两次。预付按这个算（3.5 最后一条）。
- 入口名带批号：站得住，名字是 `crates/singlefs-checker-tier/src/crash_amplification.rs:2389`「name: format!("{entry_name}.{period}.{step_in_period}"),」，入口名由测试给，「第几批」进名字。接口提醒：挂载里的续做批没有自己的入口名，`WritableMountPhase` 只有三个成员（`crates/singlefs-checker-tier/src/crash_injection.rs:251`「pub enum WritableMountPhase {」），第四个成员 `Continuation` 要逐处加臂（V3 归攻方）。

### 6.4　F1 流的组成与两态闭式（加墓碑单元之后）

3 单元文件、K = 1 的删除：意图 + 3 批。每次发布的单元数取第 3.1 节量到的角色表（行 14–17）加 S1 另加；每个单元一份落每块盘，写数 n = 2 × 单元数；流里一次带单元的普通发布增量是 2^n + 11（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:102`「= 2^n + 11」；n = 24 是 16 777 227）。

| 发布 | 单元（量到 + S1 另加） | n | 增量 2^n + 11 |
|---|---|---|---|
| 意图 | 10 + `deleted_inodes` 根 1 + 墓碑 1 = 12 | 24 | 16 777 227 |
| 批 1（留 2） | 12 | 24 | 16 777 227 |
| 批 2（留 1） | 11 | 22 | 4 194 315 |
| 批 3（留 0，删记录） | E-c：量到 11 − inode 叶 1 + `deleted_inodes` 根 1 = 11；E-b：11 − 1 = 10 | 22 / 20 | 4 194 315 / 1 048 587 |
| 合计 | | | E-c 41 943 084；E-b 38 797 356（`research/prompts/m3-delete-r2-sonnet-model/cells.out:18-19`） |

接在多记录流（全域 1 358 954 634，`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:21`「层 0 的枚举域 1358954634 个状态」）后面，全域 1 400 897 718（E-c）或 1 397 751 990（E-b），算的。
判决的 F1 增量写约 2 936 万（第一轮正推的 F-1，算式在 `research/prompts/m3-delete-r2-sonnet-model/cells.out` 第 20 行，复现为 29 360 172），那个数没有墓碑单元：意图那次 n 从 22 到 24，多 12 582 912。
单元数是我在 3 单元、单个文件的池上量的，不是在多记录流上量的；流里 A、B、C 之后的分配器状态不同，分配记录树节点数要在流上量一次才能钉。
**什么现象会推翻**：流上量出的某次发布单元数与上表不同（式子照用，数变）；墓碑形态（m3-tomb-r1）改成不是 1 个单元。

### 6.5　截断成 0 另立一条流

没复核，不改判。

## 7　没做什么、限度、清理

- 没在入库装置上量任何东西：全部量数在冻结副本 + 第一轮攻方原型 + 我的补丁上，进 kb 之前要在入库装置上重做。
- 没实现 S1：意图记录、`deleted_inodes`、墓碑、nlink 字段、预付式子、checker 的标签 4、6 判定都没有。第 3.4 节两张可用表与紧池表是用量到的各发布需求加 S1 另加的槽数**算的**；S1 另加的 3 槽（`deleted_inodes` 根 1、墓碑 2）与末批的「−2 + 1」是按角色表的槽数加的，没有落盘量过。
- 没量：(b) 取法（按落点现算 W）的重算代价；E-b 的恢复改动面；`ModelPublishKind` 选择键换掉之后 11 处各自怎么改；多记录流上 F1 各发布的真实单元数。
- 只有一个样本的量：同镜像重挂载续做逐字节相同（1.2 节）只做了 3 单元、留 2 一批、一次；散落那一格只造了一次（行 4，255 个分配记录树节点）。一次不算数。
- 不归我、没碰：V3（续做挂点）、V4（回退）、V6（nlink 与写入口）、墓碑形态（m3-tomb-r1）；第三节 U5 再建带数据的文件（X1）、U6 截断成 0 的流、U7 没复核。
- 复现：`bash research/scripts/run-with-memory-cap.sh 1900M bash research/scripts/capped.sh 8 bash research/prompts/m3-delete-r2-sonnet-model/rerun.sh <仓根> /tmp/claude-1000/m3-delete-r2/tree <草稿目录>`（峰值表里没有这条命令时需要 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`）；输出里 `D name=` 行与 `run-final-d-lines.txt` 逐行相同（草稿树上跑了三轮、逐步加用例，最后经 `rerun.sh` 从头跑一次：`test result: ok. 4 passed; 0 failed`，17 行排序后与草稿树最后一轮 `diff` 为空）。`python3 cells.py` 复现 `cells.out`。
- 负载：开跑前 `ps` 里有别的会话的 `cargo test`（单个 `cargo` 与 nextest），没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；每次都加了 `nice -n 19`，没有等锁。第一次以 8G 起被内存包装拒（退出码 251，峰值表 961M 的 2 倍），按它的提示改成 1900M；那一次命令一行没跑。
- 清理：草稿目录 `/tmp/claude-1000/m3-delete-r2-sonnet/` 里的仓副本 `tree/`、编译目录 `target/`、`rerun-work/`、`diffbase/` 交回前已删；留下日志与 `cells.py`。
