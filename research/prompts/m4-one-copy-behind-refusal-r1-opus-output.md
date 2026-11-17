# m4-one-copy-behind-refusal-r1 攻方腿报告（2026-11-09）

批次：records/2026-10-13-里程碑四预开发.md#收口修补二批。行号都是主工作区现查的。**模型一行都没编、没跑**（腿不交任务），下面凡写「待跑」的读数都按推的算，命令在「要主 agent 跑的命令」。

## 各格判定一览

| 格 | 判定 | 依据的种类 |
|---|---|---|
| 候选 1、2 在不换盘的历史上放掉真缺单元 | **没打中**（一次抽样，不支撑「没打中」的结论） | 读码归纳，模型待跑 |
| 候选 1 字面（按计数器取窗口）放掉真缺单元 | **打中**：换盘的分叉盘面，窗口是空的 | 读码，模型 `fork-*` 待跑 |
| 候选 2 该放行的还拒 | **打中**：普通多单元写（一次发布几条记录）、两次断电的 tail 差二 | 读码，模型 `multi-record-*`、`lag-*` 待跑 |
| 候选 1、2 的窗口比字节在它们要修的那一形上起不起作用 | **不起作用**：点名单元坏时记录根本不施加，窗口里永远不缺 | 读码，模型 `named-*` 待跑 |
| 「tail 证明本盘收过」 | **字面不成立**，只有归纳的弱形站得住，有一条路径打破它（离线出路） | 读码，模型 `stale-disk-*` 待跑 |
| 放行之后坏副本由谁换 | **坏在第二条位置条目上接不上**：读不到它、不计数、不搬 | 读码，模型 `older-unit-on-1-*` 待跑 |
| 判真盘（放行三态与其余 150 态） | 没读数 | 坏盘用例在各臂下待跑 |
| 放行有没有份数上限 | **没有**；今天 tail 跟得上那一支同样没有，候选没有更差 | 读码，模型 `every-data-unit-*` 待跑 |
| 另一条路（攻方自提，只在我的模型上量、被攻过零轮） | 候选 3「重放施加不算落后」：不读环、打不中上面第二、三行 | 推的，待跑 |

本轮判法：要放掉真缺单元，必须有一个单元 U 在所选版本里、这块盘上没有、又不在窗口点名里。下面逐条找 U 的来路。

## 一、过度外推与越位

### 1.1 窗口比字节的理由，在要修的那一形上是空转的

调查员写候选 1 的理由是窗口里缺的可能从没写过、所以要比（`/home/fy5090/.local/share/singlefs/work/m4-close-partial-persist-investigator-2/result.md` 第 4 节）。这一条外推过了头，读码看：

1. 只落一份的那一形，所选版本是重放施加得来的（`crates/singlefs-core/src/mount.rs:8069`，根没写进根环那一支）。
2. 重放在读阶段带 `NamedUnitVerification::Verify`（`crates/singlefs-core/src/mount.rs:9227`），施加之前要求记录点名的每个单元两份都验过（`crates/singlefs-core/src/recovery.rs:5341`，`crates/singlefs-core/src/recovery.rs:5348`；D23 文件 `.claude/kb/decisions/23-journal的角色与格式.md` 第 367 行）。
3. 所以：被选中的那一版，它施加的记录点名的单元在这块盘上一定验得过。点名单元坏的那一份，记录不施加，退到上一版，tail 不再落后，今天与两个候选都不拒。

⇒ 不换盘、只断一次电的历史上，候选 1、2 的比字节**一个都比不出来**，等价于「施加了记录就不比」。模型 `named-unit-on-0-record-on-1` 跑前写死：五臂都该 `mounted`，`selected` 是上一版。**要是读数是拒**，这一节撤回。

根写进了根环的那一支：根 FUA 在记录那道屏障之后（`crates/singlefs-core/src/transaction.rs:1528`、`crates/singlefs-core/src/transaction.rs:1697`），屏障发给每块盘（`crates/singlefs-core/src/transaction.rs:619`），所以根落了，每块盘的记录都落了，乙' 接得住。这块盘没有那条记录，只剩两种来路：换了旧快照，或那一槽也坏了（第二个故障）。

### 1.2 候选 2 把「三行都差 1」外推成了常态

调查员的支撑是三行拒因都差 1（同一份报告第 4 节「紧邻一版」）。那条流只有建空 inode，一次发布一条记录。普通写不是这样：

- 数据角色除最后一个外每个单独一条记录（`crates/singlefs-core/src/transaction.rs:14017`–`:14023`），tail 是发布末条的计数器（`crates/singlefs-core/src/transaction.rs:17433`）。写四个数据单元的一次发布，上一版的 tail 与这一版差 4，**不是差 1**。写法在 `UserChange::SequentialWriteOfAnInode`（`crates/singlefs-core/src/mounted_session.rs:183`）。
- 取号写的 tail 是见证值，不是所选版本的位置（`crates/singlefs-core/src/transaction.rs:1261`）。根与轮换都没落就断电，下一次挂载取号写 (新实例, n−1)，那次挂载第一条记录编 n+1（`crates/singlefs-core/src/mount.rs:9865`）。再断一次，差 2。

模型跑前写死：`multi-record-older-on-0` 与 `lag-older-on-0` 上 `c2` 拒，`c1`、`c1-same`、`c3` 挂得上。这两格打中的是候选 2 的放行范围（该放行的还拒），不是安全。

四句（候选 2 这一格）：分辨臂，候选 1 不中。系统看得到 (tail, 所选位置)，判别子观测得到。满足的是背景第四节第 1 问「该放行的还拒」那一分句。改法：把「差 1」放宽到「窗口」就是候选 1，能修；候选 3 也修。

### 1.3 越位：候选说「缺一份按读到一份坏处理」，那条路不存在

`crates/singlefs-core/src/recovery.rs:932` 的读法第一份验过就返回，只有前面那一份失败才记只坏一份（`crates/singlefs-core/src/recovery.rs:953`）。逐盘核那一遍不走它（`crates/singlefs-core/src/mount.rs:7078`）。于是坏在第二条位置条目（第一版盘 1）的那一份，放行之后：

- 建上一版读不到它，不计数；D4 文件 `.claude/kb/decisions/04-校验和位置.md` 第 73 行的搬迁计划里没有它。
- 调查员探针里盘 1 落后那一镜像（记录只落盘 0、弄坏盘 1 的一份）没量过；那 153 态弄坏的都是盘 0。

模型 `older-unit-on-1-record-on-0`：跑前写死放行臂 `relocation=NotNeeded`，卸载之后 `violations>0`。对照 `older-unit-on-1-all-persisted` 是**今天就有**的同一缺口（tail 跟得上不比字节）。要是对照绿而放行臂红，这一格改判候选带进来的；两边都红，就是另一笔账，候选不背，但用例「判真盘绿」只罩盘 0 的。

## 二、内部矛盾与循环依赖

### 2.1 拿被怀疑的盘自己的 tail 判要不要怀疑它

要分的两种：坏了一份，还是换了旧快照。候选 1 的窗口下界是这块盘自己写的 tail，比的又是这块盘。旧快照的 tail 是真的，所以多数形状下窗口正确；但窗口成立要求这块盘与所选版本是**同一条历史**。tail 只说计数器到哪，不说是哪条历史。下面就是这个循环能打穿的盘面。

### 2.2 真缺单元的盘：跨实例分叉，窗口为空（候选 1 字面）

历史，每一步指到许可它的那一句：

| 步 | 动作 | 盘面 | 许可 |
|---|---|---|---|
| 1 | 池在 (i, c0) | 两盘同 | 普通历史 |
| 2 | 管理员留一份快照 A，再挂载、发 k 次改动 | 实例 i+1，tail (i+1, c1) | D18 行 322 只判交进来的盘 |
| 3 | 把盘 0 取下存成 S | S 的 tail (i+1, c1) | 同上 |
| 4 | 两块盘都还原成 A | 环里没有 c0 之后的记录 | 计数器取环里读得出的最大（`crates/singlefs-core/src/mount.rs:9865`，D23 文件第 373 行） |
| 5 | 挂载两次、发若干次，使计数器回到 c1 | 实例 i+2，所选位置 (i+2, c1) | 同上 |
| 6 | 盘 0 换回 S，崩后可写挂载 | 见下 | 见下 |

判到的每一步：

- 见证：`crates/singlefs-core/src/mount.rs:9137` 只比计数器，c1 等于所选末条，不拒（c1 大于就拒，所以分叉要恰好等号）。
- 择根：`crates/singlefs-core/src/mount.rs:6952` 这类序是 (txg, 实例)。txg 的根落在盘 1 所属区域（`parameters_of_the_common_test_pool` 的区域归属 0、1、0，txg 模 3 余 1）时 S 上没有它。
- 逐盘核：(i+1, c1) < (i+2, c1)，算落后（`crates/singlefs-core/src/mount.rs:7061`）；环槽里不是那一版（`crates/singlefs-core/src/mount.rs:7121`）；今天比全部单元，缺的全记下，**拒**。
- 候选 1 字面：计数器窗口 (c1, c1] 是空的，缺的全算窗口外，**放行**。S 上真没有第二条历史写的任何单元。

模型 `counter_fork_sweep` 把 k 放开扫 49 个组合，只固定前缀，不写死哪一个 k（「攻方腿的装置把用户动作写死」那一条）。判据：`arm=c1` 行 `mounted` 且 `differing_after_mount_on_0>0`，同一行 `arm=today` 拒。

四句：分辨臂（候选 2 同实例那一判不中；`c1-same`、`c3` 不中）。系统看得到实例代号不同，看得到。满足的是第四节第 1 问前半句。改法：同实例才放宽，或按 (实例, 计数器) 取窗口，都能修，都零轮。

**这一格的份量要说清**：要还原两块盘、再把一块换成另一条历史的快照，三步都是管理员操作，不是普通断电。我没找到不换盘走到这个等号的路径（第 2.4 节）。

### 2.3 窗口是否站得住：同实例时

同实例、窗口非空：所选版本末次发布写的单元在窗口里点名、也在版本的单元表里，旧快照没有就拒。靠的是「末次发布写的单元都在 `units` 里」，零单元发布例外（`crates/singlefs-core/src/transaction.rs:1486`）；零单元那一版只改根，没有要缺的。推的，没造出反例。

### 2.4 没有实例切换也能不能走到 tail 超过单元

`next_counter` 取环里最大（`crates/singlefs-core/src/mount.rs:9865`），不换盘时计数器不回头。唯一的回头是所选根那条记录两份都读不出（D23 文件第 373 行），新实例落回原槽：tail (i, n)、取号写 (i+1, n)，取号在任何单元之前过屏障（`crates/singlefs-core/src/transaction.rs:1325`），盘的 tail 已是新实例。今天这种形状算不落后。不产生窗口问题。一次抽样，推的。

## 三、哪一条最脆弱：「tail 证明本盘收过」

背景第二节第一处没证。逐条写 tail 被谁写、写到哪：

| 写 tail 的 | 写什么值 | 写到哪 | 单元到了没有 | 位置 |
|---|---|---|---|---|
| 发布末尾轮换 | 这次发布末条计数器 | 每块盘 | 到了：前面两道屏障都发给每块盘 | `crates/singlefs-core/src/transaction.rs:608`、`crates/singlefs-core/src/transaction.rs:1521` |
| 取号写两轮 | 所有盘所有自证槽 tail 的最大值（跨盘） | 每块盘 | **不一定**：见下 | `crates/singlefs-core/src/transaction.rs:1010`、`crates/singlefs-core/src/transaction.rs:1261`、`crates/singlefs-core/src/transaction.rs:1308` |
| 取号回卷 | 同一个见证值 | 写过的盘 | 同上 | `crates/singlefs-core/src/transaction.rs:772` |
| 抬 F 那一步 | 调用方给的现行那一对 | 每块盘 | 挂着，入口已核 | `crates/singlefs-core/src/transaction.rs:861` |
| 丙' 补写 | 取号见证值 | 两槽全废的盘 | 闸一比所选版本全部单元 | `crates/singlefs-core/src/mount.rs:6859` |
| 离线出路 α' | 断言那一版末条计数器 | 不自证的槽，**哪块盘都行** | **完全不核单元**，只有单元区 birth_txg 闸 | `crates/singlefs-core/src/system_configuration_repair.rs:268`、`crates/singlefs-core/src/system_configuration_repair.rs:294`、`crates/singlefs-core/src/system_configuration_repair.rs:382` |

调查员问的「tail 会不会写到没收过单元的盘上」（同一份报告第 4 节 ③）：**会**，两条路。

- **取号写**：见证值可以高于这块盘自己的 tail。那块盘只保证收过**所选版本的单元**（取号之前逐盘核过），没收过 (自己 tail, 见证值] 里、所选版本不再引用的单元。这些单元之后不会被要，所以弱形成立：**对任何之后的版本 V，V 里写在 tail 之前的单元在这块盘上，除非后来坏了**。按发布归纳，取号这一步的归纳假设靠逐盘核。候选 1 把逐盘核收窄之后，窗口外的单元靠上一步的假设；归纳不断。这是推的，模型没验归纳本身。
- **α'**：直接破。一块旧快照的盘两槽都坏，α' 断言现行那一版，把 tail 写上去（birth_txg 闸只挡比断言新的单元，旧快照没有）。之后 `crates/singlefs-core/src/mount.rs:7061` 判不落后，比字节那一步根本不走。**今天就放行**，与候选无关；候选只是不再在下一次崩溃时碰巧拒它。模型 `stale-disk-control`（拒）、`stale-disk-after-alpha`（跑前写死：`arm=today` `mounted`、`differing_after_mount_on_0>0`）。

⇒ 判决第 23 行的前提改写时不能写「tail 证明本盘收过」。能写的只有弱形，并且要把 α' 列成例外或收紧 α'。

同样脆的一处：比较是 (实例, 计数器) 的字典序（`crates/singlefs-core/src/address.rs:40`），实例新就算不落后，不管计数器。今天 (i+1, w) 永远不落后于 (i, n)，w 可以小于 n。同实例分叉的盘（两盘还原、两边各挂一次）tail 相等时**今天就不比字节**。模型 `same-instance-fork-*` 量：`arm=today` 行 `differing_after_mount_on_0>0` 就坐实今天的判也挡不住这一族。这一条说明候选 1 的分叉形状不是它独有的那一类毛病，只是多一个入口。

## 四、背景第二节三处没证的逐条答

| 处 | 答 | 读数 |
|---|---|---|
| tail 取哪块盘 | 判的时候取**本盘**两槽世代号最大那份（`crates/singlefs-core/src/mount.rs:6767`、`crates/singlefs-core/src/mount.rs:7056`）。写的时候见第三节表。会写到没收过的盘上，取号写与 α' | 读码，推的 |
| 记录的点名是不是全集 | `PublishWrites` 全仓只在三处装（`crates/singlefs-core/src/transaction.rs:1815`、`crates/singlefs-core/src/transaction.rs:2412`、`crates/singlefs-core/src/transaction.rs:17421`），设备写只走它（`crates/singlefs-core/src/transaction.rs:1514`）。零单元：不写单元、不点名（暖机、抬 F 的空发布）。写行：写的是实例表各片加分配记录树节点（`crates/singlefs-core/src/transaction.rs:2395`–`:2410`），点名是同一组角色（`crates/singlefs-core/src/transaction.rs:2248`、`crates/singlefs-core/src/transaction.rs:2309`）。带文件：写的是搬迁副本加重写角色（`crates/singlefs-core/src/transaction.rs:17284`），点名同组（`crates/singlefs-core/src/transaction.rs:17336`–`:17357`）；切记录不丢角色（`crates/singlefs-core/src/transaction.rs:14017`–`:14043`）。取号只写系统配置，不写单元。**按构造是全集**。例外：mkfs 写的树表与实例表第 0 片没有记录点名（`crates/singlefs-core/src/mount.rs:1331`），只在树表 0 条那一版；它们写在第一个 tail 之前，算窗口外，方向是对的 | 读码，推的；调查员探针量到建 inode 五次发布全等，别的发布没量 |
| 放行之后的后果 | 第五节第 3 问 | 待跑 |

没证完的一处：实例表第 1 片起不进版本的单元表（`crates/singlefs-core/src/mount.rs:1308`），第 0 片在、后面某片坏时今天与候选都看不见。不是候选带进来的。

## 五、第四节五问

### 第 1 问：判得对不对

第 1、2 节。不换盘：候选 1 没打中，候选 2 只多拒。换盘：候选 1 字面打中一形，改成同实例能修。**候选 2 不放掉真缺单元的形状，但放行范围窄到普通写的崩溃都不罩**（第 1.2 节）。

### 第 3 问：放行之后接着可写

| 问 | 走到什么 | 读数 |
|---|---|---|
| 盘 0 的记录缺着 | 放行之后取号写 (新实例, 见证值)，之后每块盘的 tail 不再落后这一版；环里那一槽是洞。扫环「任一份在即在」，后面不读它。这个洞不是候选带来的：不坏盘的流上 `never-damaged` 那两态就有 | 读码；`control-record-on-1-no-damage` 待跑（判 checker） |
| 下一次发布 | 走会话 `publish_user_change`；入口核 `devices_behind_the_current_version`（`crates/singlefs-core/src/mounted_session.rs:2918`）读新的 tail，不落后 | 读码；`older-unit-on-0-record-on-1` 待跑 |
| 取号之后、第一次发布之前断电 | (新实例, n−1) 字典序不落后于 (旧实例, n)，再挂不比字节，放行 | 读码，推的 |
| 取号之后、第一次发布写记录时断电 | tail (新实例, n−1)、所选 (新实例, n+1)。候选 1 窗口含 n、n+1；候选 2 拒；候选 3 放 | `lag-older-on-0` 待跑 |
| 卸载之后判池级 checker | 三态（与其余 150 态）今天没读数。候选臂下由坏盘用例判 | 待跑，`guards-*.txt` |
| 坏的那一份由谁换 | 坏在第一条位置条目（盘 0）：建上一版那一读计数（`crates/singlefs-core/src/recovery.rs:954`），第一次改动之后另发的那一批搬（D4 文件第 73 行）。**坏在第二条位置条目：接不上**（第 1.3 节） | 读码；`older-unit-on-1-*` 待跑 |

D4 文件第 73 行写豁免映射的三类不进计划，副本用例却断言那一批换下映射树叶与实例表（`crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs` 的 `pool_with_the_batch_replacing_the_exempt_units_published`）。我没核那一批是补丁改的还是条文落后，交主 agent。

### 第 4 问

| 盘面 | 判什么 | 依据 |
|---|---|---|
| 两块盘都落后 | 两块盘的池上不会有两块都落后又都没记录：记录在哪一块，哪一块过乙'。记录两份都读不出时，所选位置拿环里最大的那条顶（`crates/singlefs-core/src/mount.rs:9270`），常常就等于 tail，都不算落后 | 读码，推的。多于两块盘第一版没有（`crates/singlefs-core/src/mount.rs:1302`「两盘同槽」） |
| 坏的不止一份，同一块盘 | 窗口外的全放，没有上限。`every-data-unit-on-0-record-on-1`。tail 跟得上那一支今天同样没有上限（`every-data-unit-on-0-all-persisted`），D18 行 322 那句「几份」代码里没有数 | 读码，待跑 |
| 坏的两份在不同盘、不同单元 | 放行；盘 1 那一份接不上搬迁（第 1.3 节） | `older-on-0-other-on-1-record-on-1` 待跑 |
| 同一单元两份都坏 | 建上一版读不出，这一拒在逐盘核之前（`crates/singlefs-core/src/mount.rs:9841`），候选管不到 | `same-unit-both-copies-record-on-1` 待跑 |
| 坏的一份正好是这次发布写的 | **不拒**：记录不施加，退上一版（第 1.1 节）；那次发布没返回过，没丢已确认的。若根也落了、盘 0 那条记录也坏（两个故障），所选靠根，三个候选都拒：跟「发布途中取下的快照」逐字节分不开，判别子观测不到 | `root-landed-record-bad-on-0-named-on-0` 待跑 |

### 第 5 问：改动面（步数都是推的）

| 处 | 候选 1（同实例） | 候选 2 | 候选 3（攻方自提，零轮） |
|---|---|---|---|
| `crates/singlefs-core/src/mount.rs` | `units_of_the_version_missing_on_a_device_behind_it` 里约 40 行：读窗口、取并集、读不到退回；不动签名（`:7049`） | 约 10 行，同一函数，只读一条记录 | `devices_without_the_selected_version` 加一个入参（择到的根那一版的 jsn）约 10 行，读阶段的 `records` 算好传进去（`:9757`）；不读环 |
| `crates/singlefs-core/src/mounted_session.rs` 与 `mount.rs:7621` 的同一判 | 不改代码，判法跟着变：挂着的三个入口（`crates/singlefs-core/src/mounted_session.rs:2575`、`crates/singlefs-core/src/mounted_session.rs:2918`、`crates/singlefs-core/src/mount.rs:7621`）一起放宽，落后的盘多读环 | 同左 | **不动**，挂着的入口照今天 |
| 「按读到一份坏处理」 | 要新接：`units_missing` 只有角色与槽，`record_the_units_read_with_one_copy_failing_while_building_on_it`（`crates/singlefs-core/src/mount.rs:1260`）要位置条目；逐盘核在建上一版之后，要从 `start.previous` 改路 | 同左 | 同左 |
| GPU 与 CPU 判器 | **不用改**：`units_of_the_version_missing_on_a_device_behind_it`、`what_a_device_behind_the_version_lacks`、`BehindTheSelectedVersionAndMissingItsUnits` 三个名字在 `crates/` 下只命中 5 份文件：`crates/singlefs-core/src/mount.rs`、三份用例、`crates/mutations.tsv`（`grep -rln` 现查）；判器判的是卸载之后的盘 | 同左 | 同左 |
| `crates/singlefs-verification/src/mount_transparency.rs` | 坏盘用例上若一个都不拒，不动也过（第 2330 行映成 `None`）。还拒的盘面（根落了、记录也坏）走到它就红，要加单子一行加复算，约 50 行，连 D23 单子 | 还拒的盘面更多，**这一行躲不掉**：多记录与差二的拒要进单子 | 同候选 1 |
| kb 与变异 | D18 文件第 322 行与英日两份、判决第 23 行那句前提、C593（`.claude/kb/checks-owed.md` 第 355 行）、里程碑四出口 9；`mutations.tsv` 现有 2 行锚这一判，要各核一遍再加行 | 同左，C593 要写明还留一半 | 同候选 1 |

### 几个改法各修哪一格（都是「推的，待主 agent 跑」）

| 盘面（模型的 shape 名） | 今天 | c1 字面 | c1-same | c2 | c3 |
|---|---|---|---|---|---|
| `older-unit-on-0-record-on-1`（要修的那一形） | 拒 | 放 | 放 | 放 | 放 |
| `multi-record-older-on-0` | 拒 | 放 | 放 | **拒** | 放 |
| `lag-older-on-0` | 拒 | 放 | 放 | **拒** | 放 |
| `root-landed-record-bad-on-0-older-on-0`（两个故障） | 拒 | 放 | 放 | 放 | **拒** |
| `root-landed-record-bad-on-0-named-on-0`（两个故障） | 拒 | 拒 | 拒 | 拒 | 拒 |
| `fork-*`，等号那几格（真缺单元） | 拒 | **放（打中）** | 拒 | 拒 | 拒 |
| `stale-disk-after-alpha`（真缺单元） | **放** | 放 | 放 | 放 | 放 |
| `same-instance-fork-*`（真缺单元） | **放**（tail 相等那格） | 放 | 放 | 放 | 放 |

候选 3 的定义：所选版本是重放施加得来的（所选 != 择到的根），且这块盘 tail 不小于择到的根那一版的 jsn，就不算落后。理由：择到的根之前的单元归 tail 的弱形管，之后的单元都由重放验过两份。**只在我的模型上量过（其实还没量）、被攻过零轮。** 它不罩记录也坏的两故障，那一格留只读。

## 六、线索（没造到盘面，交主 agent 定要不要追）

- 整理搬数据单元不重写指针（`crates/singlefs-core/src/transaction.rs:17334`），版本单元表的数据单元落点取指针里的位置（`crates/singlefs-core/src/recovery.rs:4808`）。旧落点释放、复用之后，逐盘核比那一槽，字节必不同。**不坏盘也可能被判缺**，只要这块盘落后、又没记录；窗口里复用的，候选 1、2 同样拒。这条推的，要强制整理，模型没造。什么现象推翻：造出整理、复用、只落一份之后，今天的臂挂得上。
- `stale-disk-after-alpha` 之后再发一次、只落一份断电，候选臂放行、今天拒：这是候选相对今天多放的唯一一类，模型没跑这一步。

## 没打中的形状（一次抽样，不支撑「没打中」）

试过的是读码推演，模型只罩表里那些，范围：两块盘、`parameters_of_the_common_test_pool`、一个 7 单元左右的文件、发布全是建 inode 与一次顺序写。

- tail 超过单元：发布轮换（屏障之后）、取号回卷、抬 F、丙'。没走通。
- 单元写了不点名：三处装配逐个看，没走通（第四节）。
- 窗口里读到别的实例的记录把真点名盖掉：候选写「任一块盘一份」，我的臂取并集，不盖。实现成先到先得才有这一形（`crates/singlefs-core/src/recovery.rs:5256` 就是先到先得），实现时要写明。
- 环转圈让窗口读到旧圈记录：比了计数器，读到旧圈算读不到，退回拒。
- 单元区里被放弃的发布写过的槽：它们不在所选版本里，比不到。
- 位置拿环里最大的那条顶（`crates/singlefs-core/src/mount.rs:9270`）：窗口变大，方向保守。
- 回退：回退目标的单元在 F 之内不被复用（条款照 D23 文件已定项 14），窗口外的旧快照盘有它们。推的，没造。

什么现象推翻「不换盘打不中」：任一条不换盘、不用 α' 的历史上，`arm=c1-same` 或 `arm=c3` 挂上，而 `differing_after_mount_on_0` 里有不是手工弄坏的单元。

## 这条腿自己的限度

- **没编过。** 模型与补丁只过了 `rustfmt` 语法这一关与替换点计数（5 处各 1）；第一次编很可能要修。
- 模型的盘面是手挑的，不是枚举。挑的理由是这次发布的段里单元写 20 多个，整写子集超过约 10⁶ 上限（`MODEL_STREAM` 会打出段大小与闭式状态数），按规矩那段不跑。枚举那一半只有坏盘用例自己的取法（少于十个写的段每个真子集，别的段按前缀），**不是全域**，也不含原地覆写撕裂。
- 分叉要换盘，判它算不算射程内是主 agent 的事。D18 文件第 322 行把旧快照当要挡的东西（落后支的来由），我按射程内算。
- 模型里有两次写超了每次 150 行（第一段 224 行、第二段 161 行），已落盘、内容完整，照实记。

## 要主 agent 跑的命令

模型文件与 sha256 在下面，复跑只这一条。

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的 one_copy_behind_refusal_model 与四个既有测试目标; 范围=5 个测试目标; 为什么=坏副本放行第一轮攻方腿逐臂读数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-one-copy-behind-refusal-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-opus/copy --work /home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-opus/rerun-1
```

- 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-opus`（`--cwd`）；副本 `copy/` 在 `.deliverables` 写成交接。它是 r3 副本（带坏盘用例）照 `minimal-copy.py` 建的，**没打臂**，脚本在 `rerun-1/tree` 里打。
- 看什么：`task wait` 末行 `exit=0` 只说明模型编出来跑完了。`task-server/task log <号>` 里 `name=model rc=` 与四行 `name=guards arm=… rc=`（放宽臂下用例红是读数）。读数在 `rerun-1/model.txt`：`grep '^MODEL_'`，有 `MODEL_SCENARIO_PANICKED` 的那个场景要修，末行要有 `MODEL_DONE`。`rerun-1/guards-*.txt`：`grep -E '^test |^test result|panicked at'`。
- 编不过时：把 `rustc` 报错交新一条同立场腿（带「承接：」），它改模型目录，不改副本。
- 估时：分叉那一扫自己估，超过 40 分钟自己停（`MODEL_FORK estimate_seconds=`）。四个既有目标各跑一遍乘四臂，坏盘用例要走给定状态引擎、写判定库：**判定按盘字节进键，臂不进键**，同字节的盘两臂结论相同，这是对的，但要主 agent 知道。整条的挂钟我没估。

接着要做的（等上面的结果）：照「各格判定一览」与第 1.1、1.2、1.3、2.2、三节写下的跑前判据逐格对；对不上的格当场改判、撤回。主 agent 新派同立场的腿，带「承接：<本报告>」与任务号。

模型文件 sha256：

```
8889498e838e680d573b9244489f45615484ceab7bc4ecf7892d91b63afa7ae9  src/apply_arms_patch.py
44784dd8f5e88552560096726a59f32371cd5218cf897cdc19470719d0c7a0e9  src/one_copy_behind_refusal_model.rs
cea901ef65836050a7aac4484e722d24d9cea2d3648e3c9aa2ca62718eea7967  tools/rerun.sh
```

## 没做什么

- 不判正推那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没往统一任务服务交任务，模型与用例一次没跑，判决要引的数全部没复跑。上面所有「放 / 拒」都是跑前写死的预期，不是读数。
- 没读禁读清单里的腿。没读 `research/prompts/m3-close-sysconf-r3-*` 的腿，只读判决第 23 行。
- 英日两份 kb 没读。D23 已定项 14 与 D18 已定项 11 只读了引到的行。
- `mutations.tsv` 那 2 行在候选下还红不红，没核。
- 副本 `/home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-opus/copy` 没删：交主 agent 跑（`.deliverables` 已写交接）。
