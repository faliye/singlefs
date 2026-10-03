# m3-delete-r2 攻方腿报告（Opus，2026-10-01）

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 V6 V7 S1 R3 L4 X1 F1 T1 M1 M2 M3 Z1 Z2 Z3 Z4 Z5 Z6 E-b E-c -->

副本上的数都是副本上的，不算入库装置上的数。原型与我提的改法（Z1–Z6）都被攻过零轮，只在我的模型上量过或没实现。

## 零、复跑与文件

复跑（从仓根；草稿目录必须还不存在，跑完自己删）：

```
RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<理由> nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G \
  bash research/scripts/capped.sh 8 bash research/prompts/m3-delete-r2-opus-model/rerun.sh /tmp/claude-1000/m3-delete-r2/tree <草稿目录>
```

输出里 `grep -o 'E name=.*'` 的行与 `run-final-e-lines.txt` 逐行比（254 行）。这一次从模型目录从头跑：「test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 106.23s」，254 行与之前分开跑的八次排序后 `diff` 为空。

```
881d3c1ad57d4c8d8f33721c67d2b1c58c5ce3431c123f108aadebf671f379e7  core-prototype-r2.diff
ddeafe47b7ff05d188c3ebdef9066caa0ab3ba774977a04707c148cb579adeb2  m3_delete_r2_attack_prototype.rs
e613ad6814e797d0d74961453522f4913b87551b2251cb80b4b583c283ba039f  rerun.sh
c42621cc8e3d8f40f4ca9f51eb488495332e3e08fd91d5df5f70d27f62de5a48  run-final-e-lines.txt
```

（都在 `research/prompts/m3-delete-r2-opus-model/`。）冻结副本开工时 `sha256sum -c research/prompts/m3-delete-r2-snapshot/crates-sha256.txt` 全 OK；第一轮的 `core-prototype.diff` 在这一轮副本上照样打得上（`patch --dry-run` 无失败），所以 `core-prototype-r2.diff` 是它加这一轮三处改动合成的一份。我引的 `crates/` 文件与主仓工作区此刻逐字节相同（`cmp` 八个文件 + `walk.rs`），行号两边通用。

**原型是什么**（只改 `admission.rs`、`records.rs`、`transaction.rs`）：

- 第一轮那一套：一次不写文件内容的发布把 inode 1 的 extent 从尾部截到前 M 个单元、size 跟着截、nlink 由线程局部开关写 0（`publish_unlink_batch_for_tests`）。「意图那次」= 留全部、size 不变、nlink = 0。
- 这一轮加的：① **测试当 R3 的账房**：线程局部 `LEDGER_FOR_TESTS` 给（待删占用, 已承诺预留）两个按全部副本之和记的字节，准入读数（`admission_reading_with_the_switch_reserve_of`）与记账行标签 4、6 都读它；② 两个只关准入的开关：式子那一判（`transaction.rs:5135` 那一处）、落点那一判（`:5089` 那一处），R3「删除各批不再按普通准入判」用前一个；③ `publish_rewrite_inode_record_for_tests`：把某个 inode 的记录原样重写一遍（空 inode 意图那次与最后一批的代用）。
- 没有：意图记录、`deleted_inodes` 树、墓碑、nlink 字段（nlink 仍是按 inode 号的线程局部开关）、续做挂点。凡说到它们的量都是推的，逐处标。

**几何**：两块内存稀疏盘、两副本、默认 journal 环；盘 4 GiB（单元区每盘 211 968 槽）；根环每区 8 槽（三区共 24 条根），环窗口那一格另跑每区 16 槽（最大值，48 条根）。「挂载之后」指 mkfs 那个进程关掉、`mount_writable` 重开（新实例、写行、暖机），这之后的分配器装着根环表、按环里最旧有效根回收；mkfs 那个进程里的分配器不回收（第一轮「一个槽都没回收」就是这个原因）。

**崩溃状态**：一个都没枚举。三单元文件、K = 1 的删除，四次发布的两态闭式各 16 777 223、16 777 223、4 194 311、4 194 311（`E name=closed_form` 四行），每段都超过约 10⁶，照定义不跑。这条腿跑过的全量状态数 0。

## 一、各格判定一览

| 格 | 攻的是什么 | 判定 | 量过 / 推的 |
|---|---|---|---|
| V2 W 是不是写入那一刻算得出的上界 | 同一个文件（145 单元、K = 144），写完之后用户做不同的事再删 | **打中**：删除三次发布新占的槽从 [15, 14, 14] 涨到 [19, 16, 14]（建 13 980 个空 inode 之后），合计 43 → 49；按写入那一刻的形状算 W，预付差 6 槽 / 盘 | 量过 |
| V2 W 随文件大小 | N = 1 … 1000、K ∈ {1, 2, 100, 144} | W 不是常数：每批新占 12 → 24 槽 / 盘，其中固定点 8 → 15；式子要按文件大小与树形状取上界 | 量过 |
| V2 K 是测试旋钮时按产品 K 预付 | 同一文件 K = 144 预付、K = 1 实走 | **打中**：3 单元文件预付 25 槽、实走 50；145 单元预付 43、实走 2116 | 实走量过；「按产品 K 预付」这个式子是推的 |
| V2 写入口漏了 `publish_new_inodes` | 空 inode 照样能按号 unlink | **打中**：可写满池上空 inode 的意图走普通准入被拒；预付它要每个 inode 约 22 槽 / 盘（代用量），4 GiB 盘整个单元区只装得下 9 634 个可删的空 inode，对照今天一个根装得下的 31 455 条记录 | 拒量过；22 槽是代用（没 `deleted_inodes` 节点、没墓碑） |
| V2 逐文件求和够不够、多不多 | 满池（式子可用 = 0）、账里一字节不给删除、各批不走式子那一判，删 100 / 200 单元（K = 1） | **逐文件求和远多于真要的**：删除最多借保留量 258 / 304 槽（24 条根）、712 槽（48 条根），借到环转一圈就不再涨、与文件大小无关；R3 逐文件预付要 1515 / 3817 槽 | 量过 |
| V2 W 含不含固定点 | 落点那一判要的量 | 每批普通分配 4–7 槽、固定点 8–15 槽；落点那一判要「普通 + ckpt_cost」= 540–559 槽（ckpt_cost 537，含整棵分配记录树） | 量过 |
| V2 同一批字节扣两次 | 预付现算、各批递减 | 没打中（按「每版现算」那种实现）；按「增量减」实现时见 V1 | 量过一种实现 |
| V4 回退把已承诺预留按 R_old 抬上去 | 满池上回退到更大的文件版本 | **打中**：回退那次发布不判准入、照成；之后式子可用每盘 −1 212 416 字节，其中 R3 那一份 −589 824；之后删 A 吃进保留量 41 槽 | 量过 |
| V4 三种情形之后 I-4.7、I-9.5、I-3.4 | 照 S1 第 8 条改 | 回退到删之前、回退进删除中间：按代码推都成立；跨第一个文件版本：候选集今天就排除，空格。另有「从删完那一版出发」的表示法问题（第七节） | 推的 |
| V3 续做挂点 | 挂载里续做会不会失败、会不会跳过 | 没找到「挂载因续做失败」的可达历史；三条推的约束（第四节） | 借保留量量过，其余推的 |
| V6 谁会重写删除中 inode 的那条记录 | 删除中一版上各种发布 | **打中**：`publish_new_inodes` 重写 inode 1 所在的叶；不带 nlink 的写者把它写回 1（还剩 4 个单元）；删除中顺序写照收、对象出生代不变 | 量过 |
| V1 三条判据 | 待删占用减两次 / 不减 | **打中**：两种非幂等的续做三条判据都过，中间三个版本的待删占用行与镜像现算不符，checker 0 红 | 量过（只取发布边界的崩溃点） |
| V5 「有树无文件」走哪一支 | 恢复认「没有文件」若落 `WithoutFile` | 撞四处今天按「树表 0 条」写的分支 | 推的 |

## 二、V2　R3 预付的式子

「新占的槽」= 这次发布分配代等于它自己 txg、未释放的分配记录的跨度之和（盘 0；两块盘每行都同值）。「普通 / 固定点」按 `crates/singlefs-core/src/admission.rs:919` 的 `space_budget_of_role` 把这次重写的角色分两格。

### 2.1 W 随文件大小与批大小：不是常数（量过）

`E name=w_sum` 十五行里取（原样行在 `run-final-e-lines.txt`）：

| N 单元 | K | 意图那次 | 各批 | 合计 | 固定点最多 |
|---|---|---|---|---|---|
| 1 | 144 | 12 | [12] | 24 | 8 |
| 3 | 1 | 13 | [13, 12, 12] | 50 | 8 |
| 145 | 144 | 15 | [14, 14] | 43 | 10 |
| 289 | 144 | 17 | [17, 14, 14] | 62 | 10 |
| 600 | 144 | 19 | [20, 21, 20, 17, 14] | 111 | 13 |
| 1000 | 144 | 23 | [24, 24, 23, 23, 21, 19, 16] | 173 | 15 |

```
E name=w_row units=1000 batch=144 step=batch_2 keep=712 txg=7 new_slots=24 ordinary_slots=10 fixed_point_slots=14 switch_reserve_slots=0 checkpoint_cost=547 landing_commit_generated_slots=557 landing_data_units=0
```

每批 W 随剩下的单元数降：extent 下段叶越多、映射树里这个文件的条目越多，一批要重写的 extent 节点、映射树与分配记录树叶越多。K = 1 时 145 单元那一条各批在 13–17 之间跳（`E name=w_sum units=145 batch=1` 那一行），跳的位置跟着分配记录树叶与映射叶的边界走。所以「W」只能是一个按文件单元数与几何取的上界函数，不是一个量出来的数。

### 2.2 写入那一刻算的 W 不是删除那一刻的上界（打中，量过）

同一个 145 单元文件、K = 144，写完之后、删之前做不同的事（用户的动作，放开扫了八种），删除三次发布新占的槽：

```
E name=w_state state=nothing units=145 batch=144 new_slots=[15, 14, 14] ordinary_slots=[7, 4, 4] fixed_point_slots=[8, 10, 10] checkpoint_cost=[537, 537, 537] sum_new_slots=43
E name=w_state state=new_inodes_13980 units=145 batch=144 new_slots=[19, 16, 14] ordinary_slots=[7, 4, 4] fixed_point_slots=[12, 12, 10] checkpoint_cost=[537, 537, 537] sum_new_slots=49
E name=w_state state=remount_then_30_rewrites units=145 batch=144 new_slots=[15, 16, 16] ordinary_slots=[7, 4, 4] fixed_point_slots=[8, 12, 12] checkpoint_cost=[537, 537, 537] sum_new_slots=47
```

另五种（建 233 / 932 / 4660 个空 inode、重挂一次、重挂五次）合计 43、45、45、45、45。普通分配那一半在八种状态下都是 [7, 4, 4]，变的全是固定点（分配记录树叶、映射树）。

后果有两条：

1. **预付不够**：W 若取写入那一刻这个文件的形状（43），删除那一刻要 49；差的 6 槽 / 盘没人预付，满池时从保留量里出（第 2.5 节）。
2. **已承诺预留会漂**：S1 第 6 条要「写入时加、意图那次转、文件变短时退」——三处各算一次 P。P 若含任何盘上状态（树深、别的文件、别的 inode），写入那次加进去的与意图那次转走的就不是同一个数，标签 6 一路漂、不会自己回 0。要不漂，P 只能是「这个 inode 的单元数 + 盘上几何常量」的纯函数，而它是上界、比实际 W 大（Z1）。

四句：① 分辨臂——分辨「P 用量出的形状」与「P 用纯函数上界」，不分辨 R3 以外的候选（R0 走普通准入、不预付）。② 系统看得到：写入时看得到这个文件的单元数与盘的几何，看不到将来别人建多少 inode。③ 满足跑前判据 V2「某次删除发布在已经预付的池上被准入拒」的前提一半：这一格量到的是「预付量 < 实际需求」；实际被拒要等保留量也被吃完，在 4 GiB 盘上没量到被拒（第 2.5 节）。④ 改法：Z1（纯函数上界）修这一格；「量出 W」那种写法不修。

### 2.3 K 是测试旋钮时按产品 K 预付（打中，实走量过）

```
E name=batch_knob units=3 product_batch=144 test_batch=1 prepaid_slots_with_the_product_batch=25 consumed_slots_with_the_test_batch=50 short_slots=25
E name=batch_knob units=145 product_batch=144 test_batch=1 prepaid_slots_with_the_product_batch=43 consumed_slots_with_the_test_batch=2116 short_slots=2073
```

「预付」这里取「意图那次 + ⌈N ÷ 144⌉ × 这个文件 K = 144 时最大的一批」（推的式子，量出的 W 代进去）；「实走」是同一个文件 K = 1 时量到的合计。按产品 K 预付、测试里用 K = 1 走：标签 4 在第二批就不够减，测试里「各批用已扣的那一份」这条路根本走不到它产品上的样子。反过来按实际 K 预付：K 不在盘上，checker 从镜像重算 I-3.4 时不知道 K（推的：这一类只供测试的旋钮——`JournalRecordNamedEntryCapacity::CappedForTests` 是先例——由调用方装进写入口，不落盘）。两条出路：K 写进盘（系统配置一个字段）或 checker 与写者都按产品常量算预付、测试旋钮只改分批不改预付（那样测试里标签 4 要允许「最后一批之前就减到 0」）。

四句：① 分辨「按产品 K」与「按实际 K」两种写法，不分辨 R3 以外的候选。② 写者看得到旋钮；checker 看不到。③ 满足 V2 的「I-3.4 的值不能只从镜像重算 ⇒ 记一格，写明还要什么输入」：还要 K。④ 改法 Z1 带上「K 进盘或预付只按产品 K」两选一。

### 2.4 写入口漏了 `publish_new_inodes`（打中，量过）

V2 题面点名的写入口是 `publish_first_file`、`publish_sequential_write`、覆盖写。`publish_new_inodes`（`crates/singlefs-core/src/transaction.rs:4830`）建出来的 size 0 的 inode 同样能按号 unlink：S1 第 1 条写的是「inode N」，X1 只限制删完之后不再建带数据的文件。删一个空 inode 至少要意图那次与最后一批两次发布。

```
E name=create_inodes count=233 watermark_after=468 leaves_rewritten=2 txg=8 new_slots=13 ordinary_slots=5 fixed_point_slots=8 switch_reserve_slots=0 checkpoint_cost=537 landing_commit_generated_slots=542 landing_data_units=0
E name=empty_inode_delete step=intent_proxy inode=2 txg=9 new_slots=11 ordinary_slots=3 fixed_point_slots=8 switch_reserve_slots=0 checkpoint_cost=537 landing_commit_generated_slots=540 landing_data_units=0
E name=empty_inode_prepay unit_area_slots_per_device=211968 prepay_slots_per_inode_measured_proxy=22 inodes_whose_deletion_fits_the_whole_unit_area=9634 inode_records_one_root_holds=31455 create_233_new_slots=13
E name=empty_inode_delete_on_a_full_pool admission=ordinary result=Err(SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_devices: [DeviceShortOfDemand { device: DeviceIdentity(0), available: AvailableBytesOnOneDevice(0), demand: BytesOnOneDevice(212992) }, DeviceShortOfDemand { device: DeviceIdentity(1), available: AvailableBytesOnOneDevice(0), demand: BytesOnOneDevice(212992) }] })) available_before=[0, 0]
```

- 不预付：满池（式子可用 = 0）上空 inode 的意图那次走普通准入被拒（上面最后一行）；走 R3「不判」则标签 4 里没有它的那一份，从保留量里出。两样都不是 S1 第 6 条说的「满池也删得动、用已扣下的那一份」。
- 预付：代用量每个空 inode 22 槽 / 盘（意图 11 + 最后一批 11，都是「重写它那片叶与根」的代用；还没算 `deleted_inodes` 的节点与意图那次一个码 3 墓碑单元，推的再加 3–4 槽）。建 233 个 inode 今天一次发布新占 13 槽；按每个预付 22 槽，同一次要再扣 5 126 槽 / 盘。整个 4 GiB 单元区只够 9 634 个空 inode 的预付，对照一个 inode 根今天装得下的 31 455 条记录。

四句：① 分辨「预付只挂在写数据的入口」与「建 inode 也预付」。② 写者看得到（建 inode 的发布知道建几个）。③ 满足 V2 的「某次删除发布在已经预付的池上被准入拒」：池对写数据的那一份预付过了，空 inode 那一份没有。④ 改法：Z4（建 inode 也预付，代价如上）或 Z2（不逐对象预付批的元数据，见下一节）；只改「写数据的入口」的那种写法在这一格上照样中。

### 2.5 删除各批真要多少新槽：一个根环窗口，与文件大小无关（量过）

装置：挂载之后（分配器按环回收），把池填到式子可用 = 0（在分配器拷贝上把空闲槽标成已分配，盘上不写），账里待删占用与已承诺预留都给 0，删除各批不走式子那一判、落点那一判照判，K = 1 删整个文件。「借」= 账为 0 时式子扣掉的保留量（挂载期承诺量 + checkpoint 保留池，每盘 6 989 槽）减此刻空闲槽，正数就是删除吃进了保留量。

```
E name=ring_window ring_slots_per_region=8 ring_roots=24 units=100 batch=1 publishes=101 result=Ok borrowed_before=0 max_borrowed_slots=258 at_publish=23 final_borrowed_slots=140 max_deferred_slots=391 intent_w=15 batch_w=15 ring_roots_times_batch_w=360 r3_per_file_prepay_slots=1515 reserve_slots_dev0=6989
E name=ring_window ring_slots_per_region=8 ring_roots=24 units=200 batch=1 publishes=201 result=Ok borrowed_before=0 max_borrowed_slots=304 at_publish=23 final_borrowed_slots=-60 max_deferred_slots=409 intent_w=17 batch_w=19 ring_roots_times_batch_w=456 r3_per_file_prepay_slots=3817 reserve_slots_dev0=6989
E name=ring_window ring_slots_per_region=16 ring_roots=48 units=200 batch=1 publishes=201 result=Ok borrowed_before=0 max_borrowed_slots=712 at_publish=56 final_borrowed_slots=356 max_deferred_slots=865 intent_w=17 batch_w=19 ring_roots_times_batch_w=912 r3_per_file_prepay_slots=3817 reserve_slots_dev0=6989
```

- 借的量在第 23 次（24 条根）/ 第 56 次（48 条根）发布到顶，之后往下走：每次发布换下的元数据与被删的数据单元进 defer，环转过一圈（`crates/singlefs-core/src/mount.rs:1158`：「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」）就回收，新一批的 W 由一圈之前那一批还回来。顶点不超过「环里根数 × 每批 W」（360 / 456 / 912），文件从 100 涨到 200 单元顶点只从 258 到 304（W 从 15 到 19）。
- R3 逐文件预付（意图 + ⌈N ÷ K⌉ × W）是 1 515 / 3 817 槽；真借的是它的 17% / 8% / 19%。按产品 K 推到大文件（推的，W 用 1000 单元那一行的 24 外推、没量更大的文件）：4 GiB 盘满盘一个文件约 105 984 单元、736 批，逐文件预付约 736 × 24 ≈ 17 664 槽 ≈ 276 MiB / 盘，而一圈 24 条根只要 24 × 24 = 576 槽。
- 这一段历史里账给 0、删除照样删完：保留量里本来就有 6 989 槽。也就是说在这个几何上，**「满池删得动」靠的是删除各批不走式子那一判，不是逐文件预付**；预付要罩的只是「一圈之内要的新槽」与「不回收的那几样」。不回收的那几样里，按接口说墓碑单元是唯一一样：它在意图那次写下、直到退役才放（推的，墓碑形态归 m3-tomb-r1），每删一个对象多一份。
- 反向：保留量小的池上「一圈」能不能超过保留量，我没量。保留量随 ckpt_cost（这里 537，含整棵分配记录树）走，一圈的量随 W 与环长走；保留量 = 13 × ckpt_cost + 8（4 GiB 盘上 13 × 537 + 8 = 6 989，与量到的对得上）；1 GiB 盘的 ckpt_cost 我没量，按分配记录树叶数粗估约 30（推的），保留量约 400 槽，而每区 16 槽时一圈约 48 × 15 = 720 槽，那时不预付就删不动、落点那一判拒。这一格要实验量。

四句：① 分辨「逐文件求和」与「全池一份环窗口」两种口径；R0 那条（各批走普通准入）在满池上被拒，第一轮量过。② 系统看得到：环长在系统配置里，W 的上界是几何的函数。③ 打中的是代价，不是跑前判据的哪一个失败分句；它说的是 R3 逐文件求和那一读法的预付量比真需求大一到两个数量级，用户 2026-10-01 定 R3 时这个比例没有数（第一轮判决第五节写的是「约为数据的 4.5%（推的，只一个几何点）」）。④ 改法 Z2：已承诺预留拆成「每对象一份墓碑（持久的那一份）」+「全池一份 (环里根数 + 1) × W 上界」（批的元数据，不随文件数与大小长）；只在我的模型上量过一半（环窗口那一半），被攻过零轮。

### 2.6 W 里算不算固定点；落点那一判要的是 ckpt_cost（量过）

每批新占的槽里，普通分配那一半 4–12 槽，固定点（分配记录树、记账树、映射树、树表）8–15 槽（2.1 节两行原样行）。S1 第 6 条「删除各批不再按普通准入判，用已扣下的那一份」若只把普通那一半算进 W，固定点每批从 checkpoint 保留池里出；保留池只按一次发布留（`admission.rs` 的 `checkpoint_reserve_pool`：ckpt_cost 个块），连删几批就把抬 F、暖机要用的那一份吃掉（一圈之内不回收，2.5 节）。所以 W 要连固定点一起算。

另一处：发布路径在式子那一判之前还有落点那一判（`crates/singlefs-core/src/transaction.rs:5089`：「if units_landing_is_judged {」），它要的提交内生槽是「普通那一半 + ckpt_cost」，而 ckpt_cost 拿整棵分配记录树顶（`crates/singlefs-core/src/admission.rs:506`：「（分配记录树那一项是 K1 整棵树）顶它（C545 的准入先拒，用户 2026-09-27 定）。」）。删除一批实际新占 14 槽，落点那一判要 541 槽（145 单元那几行的 `landing_commit_generated_slots`）。R3 若只关式子那一判、留着落点那一判，那么预付量不决定删不删得动——决定的是空闲槽够不够 541 槽，满池上靠的仍是保留量；若两判都关，只剩真取落点那一道。哪一判关、哪一判留，S1 没写，要写进 S1 第 6 条。

### 2.7 同一批字节扣两次：按「每版现算」实现时没打中

装置里账房按「剩下几批 × W」每版现算（第六节 `rule=Recomputed` 那一行），各批新占的槽进「已分配」时待删占用同一次发布减掉同样多的 W 上界，被删的数据单元进 defer、算在「已分配」里、不在待删占用里：没找到同一批字节同时在两项里的版本。按「增量减」实现时会错，那是第六节 V1 的格。

## 三、V4　回退

### 3.1 回退到更大的文件版本：已承诺预留按 R_old 抬上去，回退那次发布不判准入（打中，量过）

历史：挂载之后顺序写成 300 单元（A）→ 覆盖写成 1 单元（C）→ 把池填到式子可用 = 0 → 管理员回退到 A → 删 A（K = 144，各批不走式子那一判）。账房照 S1 第 8 条「记账标签 4、6 按 R_old 的状态重算」：回退那次把已承诺预留从 P(C) 改成 P(A)。P 用 2.1 节那种「意图 + ⌈N ÷ 144⌉ × 最大一批」代（这个文件量出来是 17 与 18 槽）。

```
E name=rollback_raise intent_w=17 batch_w=18 prepay_a=2326528 prepay_c=1146880 raise_bytes_all_replicas=1179648 available_at_c=[0, 0] new_inode_at_c=Err(SpaceAdmissionRefused[(0, 180224), (0, 180224)]) rollback_if_judged=Refused([(0, 589824), (0, 589824)]) rollback=Ok txg_after=8 units_after=300 available_after=[-1212416, -1212416] committed_row_after=2326528 dev0_allocated=204956 dev0_free=7012 dev0_deferred=61 dev1_allocated=204956 dev1_free=7012 dev1_deferred=61
E name=rollback_then_delete result=Ok(publishes=4) borrowed_slots_dev0=[3, 3, 22, 38, 27, 41] available_after_the_delete=[-671744, -671744] dev0_allocated=205020 dev0_free=6948 dev0_deferred=731 dev1_allocated=205020 dev1_free=6948 dev1_deferred=731
```

- 同一个池上建一个 inode（普通准入）被拒；回退照成（它没有普通分配，`crates/singlefs-core/src/transaction.rs:5131`：「publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles)」为假就不判；D28 已定项 1 接线：`.claude/kb/decisions/28-挂载期承诺量.md:34`：「判不判照旧：没有普通分配的发布（空发布、写行、暖机、抬 F）不判」）。回退之后标签 6 写成 2 326 528，式子可用每盘 −1 212 416 字节；其中 R3 那一份是 1 179 648 ÷ 2 = 589 824，其余 622 592（38 槽）今天就有：回退那次自己的固定点与 A 那一版更大的映射树抬高的 ckpt_cost（拆法是推的，没逐项量）。
- 回退若照「增加空间占用的操作」判（需求 = 预付之差摊到每盘），同一个池上拒（`rollback_if_judged=Refused`）。
- 之后删 A 删完了，但借保留量从 3 槽涨到 41 槽：标签 4 说预付过，空闲槽里没有那一份。这个池上保留量有 6 989 槽，借 41 不出事；按 2.5 节，借的顶点是「一圈的根数 × W」，与 P(A) − P(C) 无关。

同一机理还有一条推的路：崩溃恢复落到较旧的根（被抛弃的根留在环里）。所选那一版的标签 6 是那一版写的值，若那一版里文件更大（之后的被抛弃时间线里文件变短了），恢复之后同样「账里预付了、空闲槽里没有」，而崩溃恢复不能被拒。

四句：① 分辨「R_old 重算时回退判准入」「不判」两种改法；第一轮的 R0 不预付、这一格不存在。② 系统看得到：回退那次在内存里同时有 cur 与 R_old 两版的账。③ 满足的是 V4 跑前判据「三种回退之后 I-3.4 不成立」吗——**不满足字面**：I-3.4（`.claude/kb/invariants.md:139`：「可用空间统计已扣除」）只要求扣了，扣成负数它仍成立。满足的是 R3 自己的承诺（D3 已定项 2：`.claude/kb/decisions/03-空间分配.md:52`：「解法不是给删除路径开口子，是把那一笔提前到写入那一刻收掉」）在回退之后不再由空闲槽兑现，归错判据的风险我标在这里：它不是 I-3.4 的打中。④ 改法：Z5a 回退那次判准入、需求 = 预付之差（代价：满池上管理员回退会被空间拒，今天回退从不因空间被拒——今天它没有普通分配、不判）；Z5b 不判、允许可用为负、靠保留量兜（代价：保留量要罩住 2.5 节那一圈）；Z2 让这一格消失（预付不随单个文件变，回退没有东西可抬）。三个都只在我的模型上量过 Z5a 的拒那一半，被攻过零轮。

### 3.2 三种回退之后 I-4.7、I-9.5、I-3.4（推的，按代码与 S1 第 8 条）

前提：回退那次发布对 `deleted_inodes` 取 R_old 的根，与今天 `crates/singlefs-core/src/mount.rs:4972`：「root: if entry.kind == TREE_KIND_EXTENT || entry.kind == TREE_KIND_INODE {」里两种同形加一种；记账行今天只有水位那一行另算（`crates/singlefs-core/src/mount.rs:4989`：「value: if entry.statistic == STATISTIC_INODE_WATERMARK {」），标签 4、6 照抄 cur（第一轮量过的「回退之后待删占用照现行那一版」），要改成按 R_old 现算。

| 情形 | I-4.7 | I-9.5 | I-3.4 | 再截一次 |
|---|---|---|---|---|
| 回退到意图之前 | 成立：N 不在删除中（nlink 1、队列里没有） | 成立：R_old 的队列里没有 N | 字面成立；R3 那一份见 3.1 | 不会：队列随 R_old 空 |
| 回退进删除中间 | 成立：N 在 R_old 的队列里、nlink 0、extent 是那一版的 | 成立 | 字面成立；R_old 剩的批比 cur 多，待删占用往上调，与 3.1 同形 | 续做按 R_old 接着截，是管理员选的那一版的目标态，不算「再截一次」 |
| 跨第一个文件版本 | 空格：树表 0 条的版本今天不是回退候选（`crates/singlefs-core/src/mount.rs:4755`：「RollbackCandidateExclusion::VersionWithoutFile,」），树 ID 不同的目标也拒（`crates/singlefs-core/src/mount.rs:5131`） | 同左 | 同左 | 同左 |

没找到这三种的反例；它们都依赖「三棵树与两行账都按 R_old 现算」一起做，少一样第一轮两个方向的后果就回来（第一轮判决第三节 U1 管理员回退那一行，量过）。

另一种起点没进题面：**cur 是删完的那一版**（S1 第 5 条「有树无文件」）。今天回退的入口只收带文件的一版（`crates/singlefs-core/src/mount.rs:5042`：「回退的调用方要带文件的现行版本：cur 树表 0 条时它的时间线上还没有带文件的版本，」），「有树无文件」若在内存里表示成 `PoolVersion::WithoutFile`，删完之后管理员就回退不回删之前——数据还在 defer 里、候选也还在环里，正是最需要回退的时候。见第七节。

### 3.3 墓碑退役与回退（只答交互，推的）

回退到意图之前要退役意图那次写的墓碑。它写在回退那次发布里，回退那次就有了一个新写的单元——今天回退那次没有普通分配、不判准入（3.1）；墓碑容器若是普通分配，回退就从「不判」变成「判」，满池上回退可能被空间拒，这与 Z5a 是同一个后果。退役要的槽是不是也该预付（每对象一份），交 m3-tomb-r1。回退进删除中间、回退到删完之后都不退役。

## 四、V3　续做挂点

候选沿用第一轮的名字：M1 挂载里把队列走到空；M2 挂载里只走一批，其余在会话里每次用户发布之前接着走；M3 挂载里不走，会话第一件事走。

- **「挂载因续做失败而失败」没找到可达历史（这一几何上）**。R3 下删除各批不走式子那一判，能拒它的只剩落点那一判（要「普通 + ckpt_cost」≈ 541 槽空闲）与真取落点。2.5 节那三段历史是「满池、账里一字节不给删除」这一最坏账面，100 / 200 单元每一批都做成（`result=Ok`），借保留量的顶点 258–712 槽，离 6 989 槽远。所以 M1 在 4 GiB 盘上不会因空间失败；小盘、长环那一格（2.5 节最后一条）没量，那里 M1 可能在挂载里撞落点那一判，M2 / M3 只是推迟。
- **一次准入最多 8 次发布（`crates/singlefs-core/src/mount.rs:2123`：「pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;」）罩不到续做批**（推的）：它数的是一次准入推的抬 F 加为之判的那一次发布（`crates/singlefs-core/src/mount.rs:2311` 那一行）；R3 下续做批不走式子那一判、也不是被准入推出来的，不进这个数。所以它不限制 M1 的批数，也不保护什么：续做批吃的是保留量（2.5 节），保留量的上界由环长定、不由 8 定。
- **取号前预演**（推的）：真发之后有一道断言，比预演取的落点与真发取的落点（`crates/singlefs-core/src/mount.rs:3957`：「取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作」），只比写行与暖机那一串。续做批若排在写行与暖机之间，那道断言就比错了一串；排在暖机之后则不用进预演。进预演的唯一好处是让落点那一判在取号之前拒（挂载不留半截）；代价是 M1 的整串续做在拷贝上先走一遍。
- **C1 的字面**（`.claude/kb/checks-owed.md:17`：「崩溃点重放里，任一含未完成意图的镜像正常挂载后没跑续做即判红」）：M1、M2 过；M3 只在「正常挂载」包括会话的第一次调用时过——今天崩溃重放里的挂载就是 `mount_writable` 交回，M3 在那一刻一批都没走，按字面红。
- **挂载里续做与抬 F 的次序**（推的）：挂载前那一判不够（例如 3.1 那种回退或崩溃恢复之后账面为负）时，写行之后会推抬 F（第一轮事实表 (d) 第 17 步），F 抬到准入上限，删之前那几版就不再是回退候选。续做排在抬 F 之前还是之后，空间上都做得成（保留量罩得住），但抬 F 之后管理员就回退不回删之前了；这一条在 Z2 下不出现（账面不会因回退变负）。
- **会话里的队列**（推的，M2 / M3）：管理员回退走 `roll_back_by_a_forward_publish`，直接改现行那一版，不经会话。会话若在挂载时把队列读进内存、之后照它续做，回退到删之前之后会把刚带回来的文件再截一次（第一轮量过的那个后果，换了一条路）。队列要每次从现行那一版的 `deleted_inodes` 读。
- **二次续做**：挂载途中崩溃、再挂载，计划只读现行那一版（第六节 `plan_of_the_continuation` 两次相等），没找到反例。

挂点这一格我的排序（推的，被攻过零轮）：M1 在 4 GiB 盘上没有打中它的历史；M3 撞 C1 字面；M2 介于两者，代价是删除进度依赖用户发布。

## 五、V6　nlink 与写入口（量过）

删除中（意图之后留 4 单元）的一版上，把原型的 nlink 开关关掉，模拟「`InodeRecord` 里没有 nlink、重写时按内存里的结构序列化」的写者：

```
E name=v6 op=state_mid_delete raw_nlink=0
E name=v6 op=writable_mount row_publish_rewrites_inode_leaf=false warm_up_rewrites_inode_leaf=false raw_nlink_after=0
E name=v6 op=publish_new_inodes_1 rewrites_inode_leaf=true raw_nlink_after=1 units_after=4
E name=v6 op=sequential_write_mid_delete result=Ok(units=2) raw_nlink_after=1 object_birth_kept=true
E name=v6 op=rollback_to_the_intent result=true rewrites_inode_leaf=false raw_nlink_after=0
```

- 挂载的写行与暖机、回退都不重写 inode 叶，nlink 保持 0（V6 题面问的「写行与暖机会不会把 nlink 写回 1」：不会，量过）。
- **`publish_new_inodes` 会**：新号追加到最右那片叶（`crates/singlefs-core/src/inode_tree.rs:158`：「它的 inode 号大于树里每一个 key ⇒ 追加到最右那片叶」），inode 1 那条在同一片里就被整片重写；写者不带 nlink 时它回到 1，此时还有 4 个单元、队列里（S1 的）还有它——I-9.5（`.claude/kb/invariants.md:279`：「里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0」）当场为假，删除中的文件对读者复活成一个 4 单元的文件。V6 题面把 `publish_new_inodes` 列为「不涉及」：它不用拒，但它是一个重写删除中记录的写者。要的是 `parse` 读 nlink（今天 `crates/singlefs-core/src/records.rs:85`：「reader.skip(8 + 16);」把它跳过）、`to_bytes` 写结构里的值（今天 `crates/singlefs-core/src/records.rs:56`：「writer.put_u32(1); // nlink」），两头一起改；只改写入口那一半，这一格照样中。
- **删除中顺序写照收**、对象出生代不变、nlink 回 1（第一轮量的是删完之后，这一格是删除中）。顺序写与覆盖写都经 `inode_record_of_file_version` 现造一条记录（`crates/singlefs-core/src/transaction.rs:4595`：「inode: FIRST_INODE_NUMBER,」那一段），它不读上一版那条记录的 nlink；拒要在走到它之前。
- **删完之后 nlink 判不到**（推的）：S1 第 2 条最后一批删掉 inode 记录。之后再顺序写，上一版里没有 inode 1 的记录可看 nlink；拒法只能是「没有这条记录」，与 nlink = 0 是两个错误成员。今天 `TransactionOutput` 的 `inode_record` 不是 `Option`，类型改动会逼出这一处。
- 只读挂载 `open_file` 遇到 nlink = 0：原型没有 nlink 字段，没量。

四句：① 不分辨 S1 的候选（凡 nlink 留在记录里都一样），是实现清单项。② 系统看得到（那条记录就在它要重写的叶里）。③ 满足 V6 题面「写入口在某一步写进已删的 inode」的一种更宽的形态：不是写入口，是同一片叶的别的写者。④ 改法：nlink 进 `InodeRecord` 且 `parse` 读它（S1 第 4 条已写「加 nlink」，没写 `parse` 那一头）；量过的只有「不带 nlink 时会回 1」这一半。

## 六、V1　三条幂等判据（打中，量过）

装置：挂载之后一个 3 单元文件，K = 1（意图 + 三批）；账房三种规则：现算（待删占用 = 剩下几批 × W，W = 15 槽）、每批减两次（饱和到 0、最后一批清零）、不减（最后一批清零）。两条路径：不崩溃；第一批整个落盘之后崩溃、重开可写挂载（新实例、写行、暖机，这几次照抄镜像上的两行账）、续做到底。判据照第一轮判决第二节第 2 行：① 同一版算两次计划相等；② 删完再续做，计划为空；③ 终态逻辑内容（单元数、size、盘上 nlink、标签 4、标签 6）两条路径相同，且每一版镜像池级 checker 0 红。

```
E name=v1_criteria rule=SubtractedTwice batch_w=15 criterion_1_plan_equal=true criterion_2_done_writes_nothing=true criterion_3_final_equal_and_checker_green=true final_straight=(0, 0, 0, 0, 0) final_crashed=(0, 0, 0, 0, 0) checker_red_total=0 pending_row_vs_recomputed_straight=[(1474560, 1474560), (491520, 983040), (0, 491520), (0, 0)] pending_row_vs_recomputed_after_crash=[(0, 491520), (0, 0)] rows_not_equal_to_the_recompute=3
```

`rule=NeverSubtracted` 那一行同样三条都过、3 行对不上；`rule=Recomputed` 三条都过、0 行对不上（三行都在 `run-final-e-lines.txt`）。

- 两种非幂等的续做三条都过。原因：①② 只看计划，计划不读账；③ 只比终态，终态最后一批清零；池级 checker 不读标签 4、6（这一轮事实表 (g1) 的 grep 输出 0 行）。中间那几版的待删占用写错了，镜像上就是错的，I-3.4 现算会红，可今天没有人现算。
- 崩溃点只取了「一批整个落盘之后」这一格（发布边界）。批内的写点没枚举：四次发布的两态闭式 16 777 223、16 777 223、4 194 311、4 194 311（`E name=closed_form` 四行），每段超过约 10⁶。批内崩溃态恢复到的仍是某个发布边界那一版（或施加记录得到的那一版），账行随那一版，推的结论不变。
- 「同一落点放两次」那一种没造：今天释放前每盘核「在册、未释放、跨度对」（第一轮事实表 (b) 第 5 步），放两次会在动分配器之前报错、被 ③ 的「续做到底」看见。D8 已定项 5 第 4 条（`.claude/kb/decisions/08-核心索引结构.md:124`：「一批里每个动作施加时先核它要删的东西还在」）在这一格上今天的代码已经兑现。

四句：① 不分辨 S1 的候选，是判据写松了。② 系统看得到：标签 4、6 与 inode、队列在同一版镜像里。③ 满足跑前判据 V1 的字面：「找到一种非幂等的续做而三条判据都过」。④ 收严 Z3（只在我的模型上量过、被攻过零轮）：加第 ④ 条——**每一个被判的状态（不止终态）上，标签 4、6 等于从这一版镜像现算的值**（标签 4 = 队列里每个 inode 剩下的预付之和，标签 6 = 每个 nlink = 1 的 inode 的预付之和；Z2 之下标签 6 是一个常量加墓碑份数）；这一条就是把 I-3.4 落到 checker 里，前提是 P 是只读镜像就算得出的纯函数（2.2、2.3 两节）。「逻辑内容」该含：inode 集合与每条的 nlink、size、对象出生代，extent，`deleted_inodes` 的 key 集合，标签 4、6、12；不含：实例号、txg、落点、改动计数、标签 1、2、5（随落点与实例变）、墓碑容器的落点。

## 七、V5　「有树无文件」在内存里走哪一支（找反例，推的）

S1 第 5 条：删完之后树表仍 8 条、三棵是零条目根，恢复认它为「没有文件」。今天「没有文件」只有一种：树表 0 条（`RebuiltVersion::WithoutFile` / `PoolVersion::WithoutFile`）。删完那一版若也落进这一支，按今天的代码会撞四处：

| 处 | 今天按「树表 0 条」写的 | 删完那一版落进来之后 |
|---|---|---|
| 会话 | `crates/singlefs-core/src/mounted_session.rs:261`：「PoolVersion::WithoutFile(_) => return Err(UserChangeRefused::NoFileVersionToChange),」 | 建 inode（`UserChange::NewInodes`）被拒，而 S1 第 5 条说删完之后能做的写有 `publish_new_inodes` |
| 回退入口与候选 | `crates/singlefs-core/src/mount.rs:5042` 只收带文件的现行版本；候选按树表条数判（`crates/singlefs-core/src/mount.rs:4753`），条数 8 进了候选，重建出 `WithoutFile` 再报 TargetNotACandidate（`crates/singlefs-core/src/mount.rs:5109`，那一支的注释说是「两次读之间的瞬时读错」） | 删完之后回退不回删之前；回退到删完那一版被误报成读错 |
| 卸载 | `crates/singlefs-core/src/mount.rs:2080`：「return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {」 | 删完之后卸载一个字节不写、F 不抬 |
| 挂载前那一判 | `crates/singlefs-core/src/mount.rs:3753`：「return Err(MountError::SpaceAdmissionRefusedBeforeAcquisition {」 | 账面不够时不推抬 F、当场拒挂载 |

所以 S1 第 5 条的「没有文件」要写成「带树、文件集合为空」的一版（第一轮判决 U5 那一行说的「`TransactionOutput` 的文件那两个字段变 `Option`」），不能借 `WithoutFile`；读者对外答「没有文件」，内部仍走带文件那一支。E-c（零条目根）与 E-b（根指针置零）：checker 读码 2 节点时全零指针直接不走（`crates/singlefs-checker/src/walk.rs` 的 `read_index_node` 开头 `if pointer.all_zero`），零条目根要过 I-1.1 的 key 区间判（首末两条条目的 key），零条目时首末不存在——两种写法哪一种让 checker 改得少，没量，是实现时的一格。I-9.6 在 inode 树没有记录时报不适用（`walk.rs` 那一段 `(Some(_), None)` 分支），删完之后不红。

## 八、我提的改法各修哪一格

全部只在我的模型上量过或没实现，被攻过零轮。「量过」= 副本上的原样输出在本报告里贴了；「推的」= 按代码推、没实现没跑。

| 改法 | 修哪一格 | 不修哪一格 | 标 |
|---|---|---|---|
| Z1：P 取「单元数、产品 K、盘上几何常量」的纯函数上界；K 要么进盘、要么预付只按产品 K | 2.2 写入那一刻的 W 不是上界、标签 6 漂；2.3 checker 不知道 K | 2.4 空 inode；3.1 回退抬预留 | 推的（量到的是不这样做的后果） |
| Z2：已承诺预留拆成「每对象一份墓碑」+「全池一份 (环里根数 + 1) × W 上界」 | 2.5 逐文件求和过量；2.4 空 inode 只预付墓碑那一份；3.1 回退无东西可抬 | 小盘长环（2.5 末条）要不要更大的那一份；墓碑的量归 m3-tomb-r1 | 环窗口那一半量过（`ring_window` 三行），其余推的 |
| Z3：判据加第 ④ 条，每个状态标签 4、6 等于镜像现算 | 第六节两种非幂等续做 | 前提是 Z1（P 可从镜像算） | 量过（`v1_criteria` 三行：现算 0 行不符、另两种 3 行） |
| Z4：建 inode 也预付（逐对象口径下） | 2.4 | 代价：每个空 inode 约 22 槽 / 盘，4 GiB 盘只够 9 634 个 | 代价量过（代用），改法推的 |
| Z5a：回退照增加占用判准入，需求 = 预付之差 | 3.1 账面负 | 满池上管理员回退会被空间拒（新的失败方式）；崩溃恢复那一条路不能拒 | 拒那一半量过（`rollback_if_judged=Refused`） |
| Z5b：回退不判、允许账面负、靠保留量兜 | 3.1 的空间后果（借 41 槽） | 账面负期间别的发布都被拒；保留量要罩一圈 | 量过（`rollback_then_delete`） |
| Z6：`InodeRecord` 加 nlink 且 `parse` 读它，所有现造记录的地方（`inode_record_of_file_version`）之前先拒 nlink = 0 与「没有记录」两种 | 第五节 `publish_new_inodes` 写回 1、删除中顺序写照收 | 只读挂载的错误成员 | 后果量过，改法推的 |
| 「有树无文件」不借 `WithoutFile` | 第七节四处 | E-c / E-b 选哪个 | 推的 |
| R3 写明哪一判关：式子那一判关、落点那一判留（或两判都关） | 2.6：预付不决定删不删得动 | — | 推的 |

## 九、没打中的形状

- 「挂载因续做失败」：满池、账里一字节不给删除、K = 1 删 100 / 200 单元，三种环长（每区 8、16）都删完（`result=Ok`）。取样只有 4 GiB 盘、一个文件；小盘没跑。
- 同一批字节扣两次：「每版现算」那种账没找到（2.7 节）。
- 回退三种情形之后 I-4.7、I-9.5：按 S1 第 8 条推都成立（3.2 节表），没跑——原型没有 `deleted_inodes`。
- 挂载的写行、暖机、回退重写删除中 inode 的记录：都不重写（第五节，量过）。
- 续做计划两次相等、删完续做不发写：第六节三种账都过（这是判据的格，不是 S1 的反例）。
- 二次续做（挂载途中崩溃再挂）：只取了发布边界，计划只读现行那一版，没找到反例。
- 「没打中」都只有这一条腿的一次取样，照规则不能单独支撑结论。

## 十、这条腿自己的限度与没做什么

- 原型没有意图记录、`deleted_inodes`、墓碑、nlink 字段、续做挂点。意图那次在原型里会重写 extent 路径（145 单元那一行意图的普通分配 7 槽里有 extent 节点），真实现的意图那次可能不重写 extent，意图的 W 会小一些；各批的 W 不受影响。
- 「满」是在分配器拷贝上标出来的（盘上不写、没有分配记录）：这样的池不能重挂（重挂按记录重建、标出来的槽就没了），所以「回退之后重挂会不会推抬 F」那一条（第四节）没量。
- 空 inode 删除的 W 是「重写它那片叶」的代用，缺 `deleted_inodes` 节点与墓碑单元。
- W 的取样：N ≤ 1000 单元、一个盘大小、两种环长；大文件的外推（2.5 节 276 MiB）是推的。
- 崩溃状态一个没枚举（每段超过约 10⁶）；这条腿跑过的全量状态数 0。
- 第一轮攻过的角度（扣两次的 A / A2 口径、size 不跟着截、续做两次字节相同、只读挂载看到变短、删 inode 1 之后恢复拒、回退两个方向）没重攻；第六节第 ③ 条与第五节删除中顺序写是在 S1 的改法上攻的新形态。
- 没判辩方那几格（V2 的逐格值表、V5 的 key 区间、V7）；没替主 agent 采纳；副本上的数不算入库装置上的数。
- 负载与跑法：开工 `ps` 看到别的 `cargo test`（harness / core）与一个远端变异在跑，没有性能测量；全部编译与运行经 `run-with-memory-cap.sh 8G` 与 `capped.sh 8`、`nice -n 19`。第一次起被包装拒（同会话已有一条在跑，退出码 252，那次一行没跑），之后带 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 起；另有一次 `--no-run` 被拒（上限超过峰值表 2 倍，退 251，没跑），不影响结果。没跑 checker 档、没跑门禁。
- 没读禁读清单里的文件；包装报错里出现过另一轮一个在跑命令的路径，没去读。

## 十一、什么现象会推翻这份报告的打中

- 2.2：入库装置上同一个文件在八种状态下删除各次新占的槽都相同，或 S1 的 P 本来就定成纯函数上界。
- 2.4：S1 明写空 inode 不能 unlink，或建 inode 已在别处预付。
- 2.5：入库装置上满池删除借保留量的顶点随文件大小涨、不在一圈根数 × W 附近停。
- 3.1：回退那次发布按 R_old 现算标签 6 时被判了准入（代码改了），或 S1 改成预付不随单个文件变。
- 第五节：`publish_new_inodes` 不再重写装着删除中 inode 的那片叶，或 `parse` 已经读 nlink。
- 第六节：池级 checker 开始判标签 4、6（I-3.4 落地），那时两种非幂等续做会在中间版本上红。

## 十二、草稿目录

交回前删了整个草稿目录 `/tmp/claude-1000/m3-delete-r2-opus/`（1.5G）：仓副本 `tree/`（12M）、编译目录 `target/`（738M）、`rerun.sh` 从头复跑的工作目录 `final-run/`（722M）、模型与脚本的草稿（`model/` 188K、`tools/` 12K，内容已逐字节拷进模型目录）。复跑材料只在 `research/prompts/m3-delete-r2-opus-model/`。
