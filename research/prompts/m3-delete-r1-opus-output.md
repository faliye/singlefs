# m3-delete-r1 攻方腿报告（Opus，2026-09-30）

<!-- doc-lint:not-numbers U1 U2 U3 U4 U5 U6 U7 L1 L2 L3 L4 K1 K2 K3 M1 M2 M3 -->

这是接续的腿：前一条同立场的腿撞限额中断，留下模型目录里 7 个用例与 64 行产物，没写报告。这一条先按 `rerun.sh` 从头复跑核过（64 行逐行相同），再补 3 个用例（只读挂载读到什么、回退进删除中间的一版、回退之后再做同一批），写这份报告。副本上的数都是副本上的，不算入库装置上的数；原型与我提的改法都被攻过零轮。

## 零、复跑与文件

复跑（从仓根，草稿目录必须还不存在；跑完自己删）：

```
RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<理由> nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G \
  bash research/scripts/capped.sh 6 bash research/prompts/m3-delete-r1-opus-model/rerun.sh /home/fy5090/code/singlefs <草稿目录>
```

输出里的 `E name=` 行与 `run-final-2-e-lines.txt` 逐行比。这一条腿跑了三次：接手核现场一次（7 个用例，与 `run-final-e-lines.txt` 64 行 `diff` 为空）、加用例后在副本上一次、从模型目录从头再来一次（10 个用例，与 `run-final-2-e-lines.txt` 71 行 `diff` 为空，`test result: ok. 10 passed; 0 failed`）。

```
7fb8e9fa1e4b4c873f25fdedc87240a65781555cb19e0c65af2cb0ae47764686  core-prototype.diff
35c587f828aad45f12f1debb435638d7d2f81344cd0ecb4305b724b533964695  m3_delete_r1_attack_prototype.rs
8a9bf2d80274bc68486cc1f3b9cf975ad59eda3670ed28a8ff7717256eeb4647  rerun.sh
7ed10bc07e279263c223a0bcec5d2194e72e7cce562855c06d29838afa4d2756  run-final-2-e-lines.txt
64b7c8e94cb32a6b7053f9c312cc5903bf0ae550928c82e960babecebc746749  run-final-e-lines.txt
```

（都在 `research/prompts/m3-delete-r1-opus-model/`；`run-final-e-lines.txt` 是前一条腿那一次的 64 行，排序之后是 71 行的子集。）

**原型是什么**（`core-prototype.diff`，177 行，只改 `records.rs` 与 `transaction.rs`）：一次不写文件内容的发布把 inode 1 的 extent 从尾部截到前 M 个数据单元，M 之后的单元经今天「文件变短」那一支的映射释放，inode 1 记录的 size 跟着改、nlink 由一个线程局部开关写成 0；M = 0 时 extent 上段那条叶条目写标签 0。原型**没有**意图记录、没有 `deleted_inodes`、没有墓碑：「意图那次」用一次「留 6、size 不变、nlink = 0」的发布代替。所以凡是说到意图住哪、墓碑的格，都是按代码推的，逐处标了。

**几何与历史**：两块 4 GiB 内存稀疏盘、两副本、默认环；mkfs → 取号 → 暖机 → 第一个文件 → 顺序写成 6 个数据单元（txg 4）→ 意图那次（txg 5）→ 三批各删 2 个单元（txg 6、7、8）。只这一组几何、一个文件大小、一种批大小。

**崩溃状态**：一个都没枚举。每次删除类发布的两态闭式是 16 777 223（末批 4 194 311），单独一段就超过约 10⁶ 的线，照定义那段不跑（第六节）。这条腿与前一条腿合起来跑过的全量状态数是 0。

## 一、各格判定一览

| 格 | 攻的是什么 | 判定 | 量过还是推的 |
|---|---|---|---|
| U4 扣两次 | 待删占用按「还没释放的 extent 字节」记 | **打中**：意图那一步每块盘多扣 196 608 字节，正好是 6 个单元的一份副本；需求等于空闲槽时这个口径拒、恒 0 的口径放行 | 量过 |
| U4 扣两次（另一种口径） | 按「还没回到空闲的被删 extent」记 | **打中，且更重**：删完（留 0）之后照样多扣 196 608 | 量过 |
| U4 删除自己要空间 | 删除的一批是有普通分配的发布 | **打中**：空间满到一次覆写被拒时，删除的一批同样被准入拒；四次发布下来每块盘可用少 835 584 字节，是被删数据一份副本的 4.25 倍，这段历史里一个槽都没回收 | 量过 |
| U5 删掉之后恢复拒 | 留 0 个单元、记录留树、树表 7 条 | **核实成立**：冷恢复与可写挂载都报 `UnitMalformed`；只读挂载那条路不拒，打开 inode 1 得 0 个单元 | 量过 |
| U5 另两种「没有文件」的形态 | 去掉 extent 叶条目；去掉 inode 记录 | 都会被拒 | 推的（按 `recovery.rs` 两处判法） |
| U5 删完之后的写入口 | 再建第一个文件；给 inode 1 顺序写；建新 inode | 再建被拒；**给已删的 inode 1 顺序写照收，对象出生代不变**；建新 inode 成、之后冷恢复拒 | 量过 |
| U5 size 不跟着截 | 只删 extent、记录等回收完再动 | **出局**：中间态冷恢复与可写挂载拒；池级 checker 不红（checker 的缺口） | 量过 |
| U3 续做两次字节相同 | 跑前判据第 2 条 | **打中，不分辨臂**：崩溃重开之后重做同一批，重写的 12 个单元没有一个逐字节相同，目标态相同；判据字面让所有形态一起出局 | 量过 |
| U3 只读挂载读到什么 | nlink = 0 留树 | **打中**：读者每一步都看得到、打得开 inode 1，读到的是一个从尾部变短的文件（6 → 4 → 2 → 0 个单元） | 量过 |
| U3 续做挂在可写挂载里做完 | 暖机之后、交回之前把剩下的批做完 | **打中**：批数无界而一次准入最多 8 次发布；近满时续做被准入拒，挂载要么失败要么跳过续做 | 拒那一半量过，其余推的 |
| U3 批的粒度按记录条数 | 一次发布 65 536 条记录 | 进不了测试：删除的一批不重写数据单元，记录条数不随被删单元数长 | 推的 |
| U1 意图或删除队列不随回退取 R_old | 住记账行，或住 extent / inode 之外的树表条目 | **打中，两个方向**：回退到删之前，续做把刚回来的文件再截一次；回退进删除中间的一版，nlink = 0 的记录带着 6 个单元回来 | 回退与再截量过；「意图留着」推的 |
| U1 惰性建树 | 第一次 unlink 才发树 ID | 打中：回退跨过建树那一次时，回退代码按种类在目标那一版里找条目，找不到 | 推的 |
| U1 意图住记账行或树表条目 | 一条条目一个意图 | 只在「同时最多一个意图」时放得下；U7 的两个消费者放不进 | 推的 |
| U2 墓碑 | 整个对象一条区间记录 | 反例：管理员回退把对象带回来之后，那条墓碑指着一个活对象 | 推的（crates 里没有墓碑代码） |
| U6 崩溃点与流 | 自动命名、状态数 | 每批一次发布四段；状态数见第六节；续做并进可写挂载入口时名字分不出第几批 | 状态数量过，命名推的 |
| U7 共用意图机制 | 第一版的意图形态留什么 | 反例在 U1 那一行：只认 inode 的形态装不下整理与稀疏旁表的意图 | 推的 |

## 二、U4　空间：待删占用与「已分配」扣两次

### 2.1 量到的逐格值

下面每行都是 `run-final-2-e-lines.txt` 的原样行（只取每块盘相同的那一对里的数，行没截）。`dev0_allocated`、`dev0_free`、`dev0_deferred` 的单位是分配器的槽（16 384 字节：3 471 982 592 ÷ 211 913 = 16 384）。

```
E name=state step=before_unlink txg=4 keep=6 size=179487 demand_bytes_both_devices=819200 dev0_allocated=42 dev0_free=211926 dev0_deferred=15 dev1_allocated=42 dev1_free=211926 dev1_deferred=15
E name=state step=intent txg=5 keep=6 size=179487 demand_bytes_both_devices=425984 dev0_allocated=55 dev0_free=211913 dev0_deferred=28 dev1_allocated=55 dev1_free=211913 dev1_deferred=28
E name=state step=batch_keep_4 txg=6 keep=4 size=130536 demand_bytes_both_devices=425984 dev0_allocated=68 dev0_free=211900 dev0_deferred=45 dev1_allocated=68 dev1_free=211900 dev1_deferred=45
E name=state step=batch_keep_2 txg=7 keep=2 size=65268 demand_bytes_both_devices=425984 dev0_allocated=81 dev0_free=211887 dev0_deferred=62 dev1_allocated=81 dev1_free=211887 dev1_deferred=62
E name=state step=batch_keep_0 txg=8 keep=0 size=0 demand_bytes_both_devices=393216 dev0_allocated=93 dev0_free=211875 dev0_deferred=79 dev1_allocated=93 dev1_free=211875 dev1_deferred=79
```

四种口径下式子算出的每块盘可用（字节；从产物的 `E name=available` 二十行里取的，两块盘同值）：

| 步 | A：还没释放的 extent | A2：还没回到空闲的被删 extent | B：恒 0 | C：剩下几批还要新占的 |
|---|---|---|---|---|
| 删之前（txg 4） | 3 472 195 584 | 3 472 195 584 | 3 472 195 584 | 3 472 195 584 |
| 意图（txg 5） | 3 471 785 984 | 3 471 785 984 | 3 471 982 592 | 3 471 360 000 |
| 留 4（txg 6） | 3 471 638 528 | 3 471 572 992 | 3 471 769 600 | 3 471 360 000 |
| 留 2（txg 7） | 3 471 491 072 | 3 471 360 000 | 3 471 556 608 | 3 471 360 000 |
| 留 0（txg 8） | 3 471 360 000 | 3 471 163 392 | 3 471 360 000 | 3 471 360 000 |

「待删占用」各口径那一行的值（全部副本之和，字节）：A 是 393 216、262 144、131 072、0；A2 四步都是 393 216；C 是 1 245 184、819 200、393 216、0。

扣两次那一格的原样行：

```
E name=double_count step=intent pending_kind=A_unreleased_extents demand_equals_free_slots=true admitted=false detail=Some([DeviceShortOfDemand { device: DeviceIdentity(0), available: AvailableBytesOnOneDevice(3471785984), demand: BytesOnOneDevice(3471982592) }, DeviceShortOfDemand { device: DeviceIdentity(1), available: AvailableBytesOnOneDevice(3471785984), demand: BytesOnOneDevice(3471982592) }])
E name=double_count step=intent pending_kind=B_zero demand_equals_free_slots=true admitted=true detail=None
```

### 2.2 读法

- **扣两次成立**。意图那一步 6 个数据单元一个都没释放，它们在「已分配」里（`allocated` 55 槽里有它们的 12 槽）；口径 A 又在「待删占用」里记它们的 393 216 字节，式子按 `crates/singlefs-core/src/admission.rs:359`：「− 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数（D28（挂载期承诺量） 已定项 1）。」每块盘再扣 196 608。A 比 B 少的正好是这个数（3 471 982 592 − 3 471 785 984 = 196 608）。需求取每块盘此刻的空闲字节（分配器发得出这么多）时，A 拒、B 放行。
- **不是取样造出来的**：A 比 B 少的量在每一步都是「留下的单元数 × 32 768」，A2 比 B 少的量从意图起每一步都是 196 608，与需求取多少无关；落在两者之间的任何需求都是 A（或 A2）拒、B 放行。放开扫的是四步；文件大小、批大小、几何各只有一个点（见限度）。
- **A2 更重**：被删的单元释放之后进 defer，仍算在「已分配」里（释放不减 `allocated`，回收才减，事实表 (b) 第 8 步）；按「还没回到空闲」记，删完之后（留 0）照样多扣一份，直到回收。
- **C 扣的是别的量**：它是剩下几批要新占的元数据字节，不在「已分配」里，不算扣两次。它让可用从意图那一步起一步降到删完之后的值（3 471 360 000）、之后四步不动。它的毛病在别处：这个数是写者的计划（批大小 × 每批重写几个单元），checker 要判 I-3.4 就得照写者的计划再算一遍，批大小是只供测试的旋钮时两边还得用同一个旋钮值（推的）。
- **删除自己要空间，而且在这段历史里只出不进**。意图加三批四次发布，每块盘可用从 3 472 195 584 降到 3 471 360 000，少 835 584 字节（51 槽）；被删数据一份副本是 196 608 字节（12 槽）。四次发布里 `deferred` 从 15 涨到 79、`free` 只降不升：一个槽都没回收（回收要等根环转过去或 F 抬上来，这段历史没走到）。
- **满池删不动**（原样行）：

```
E name=full_pool demand_slots_of_the_batch=[13, 13] dev0_allocated=211956 dev0_free=12 dev0_deferred=28 dev1_allocated=211956 dev1_free=12 dev1_deferred=28
E name=full_pool_delete_batch result=Err(SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_devices: [DeviceShortOfDemand { device: DeviceIdentity(0), available: AvailableBytesOnOneDevice(196608), demand: BytesOnOneDevice(8880128) }, DeviceShortOfDemand { device: DeviceIdentity(1), available: AvailableBytesOnOneDevice(196608), demand: BytesOnOneDevice(8880128) }] }))
E name=full_pool_overwrite result=Err(SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_devices: [DeviceShortOfDemand { device: DeviceIdentity(0), available: AvailableBytesOnOneDevice(163840), demand: BytesOnOneDevice(8863744) }, DeviceShortOfDemand { device: DeviceIdentity(1), available: AvailableBytesOnOneDevice(163840), demand: BytesOnOneDevice(8863744) }] }))
```

  删除的一批的需求（8 880 128）比同一个池上覆写一个单元的需求（8 863 744）还多一个槽。`.claude/kb/checks-owed.md:311` 那一行引的「删除路径仍然不申请空间」在原型这种「每批一次普通发布」的形态上不兑现：满是在分配器拷贝上把空闲槽标成已分配造的（盘上不写），留的空闲比这一批少一个槽。这一格不在 U4 的题面里，单列交主 agent：删除的批要不要走准入、走的话从哪个池出（checkpoint 保留池今天在式子里是被扣的一项），今天没有条款。

### 2.3 打中之后的四句（U4 跑前判据「量出扣两次 ⇒ 待删占用的口径必须改，写明改成什么」）

1. 分辨臂：分辨。A、A2 中，B、C 不中。
2. 系统看不看得到：看得到。「已分配」与「待删占用」在同一份准入读数里，释放不减「已分配」是分配器自己的行为。
3. 满足的分句：「量出扣两次」。A、A2 两个口径下成立；题面正文第二节关于扣两次的那句推断坐实。
4. 改法在打中的格上还中不中：

| 改法 | 意图那一步「需求 = 空闲槽」那一格 | 删完那一格 | 标 |
|---|---|---|---|
| B：这一行对 unlink 恒 0 | 不中（`admitted=true`，上面第二行） | 不中（与 A、C 同值 3 471 360 000） | 量过 |
| C：记剩下几批要新占的元数据 | 这一格没单独判准入；可用比 B 少 622 592，扣的不是已分配里的字节 | 不中 | 可用值量过，准入那一判推的 |
| E（我提的）：这一行照 A 记「删除中的 inode 还没释放的 extent 字节」给 df 与 checker 用，准入式子不扣它 | 准入上与 B 同值 | 同 B | 推的：没实现；只在我的模型上量过 B 那一列，被攻过零轮 |

E 的理由：A 这个量 checker 从盘上算得出来（删除队列里每个 inode 在 extent 树里还有几个单元），I-3.4 有对象可判；B 让那一行恒 0，I-3.4 在 unlink 上没有东西可判；`.claude/kb/decisions/08-核心索引结构.md:141` 那一段要的是 df 反映「删了还没放完」，它没说准入要再扣一遍。E 要动 D28 已定项 1 的式子，是改已定项，归主 agent 与用户。

**什么现象会推翻 2.2**：`mark_released` 改成释放时就减 `allocated_slots`，A 就不再扣两次（A2 照旧）；或者式子里「待删占用」那一项改成加回。

## 三、U5　恢复与写入口

### 3.1 「删掉 inode 1 之后恢复拒」：成立

```
E name=recover step=batch_keep_0 outcome=Failed root=Some((InstanceGeneration(1), CheckpointTxg(8))) failure=UnitMalformed { what: "extent 树里第一个文件一个数据单元都没有" }
E name=read_only_mount step=batch_keep_0 result=Ok
E name=writable_mount step=batch_keep_0 result=Err(Recovery(UnitMalformed { what: "extent 树里第一个文件一个数据单元都没有" }))
E name=checker step=batch_keep_0 red=[]
E name=reader step=batch_keep_0 inode_records=1 inode_1_present=true raw_nlink_on_disk=0 open_file_inode_1=Ok(units=0 size=0)
```

拒的那一句在 `crates/singlefs-core/src/recovery.rs:3365`：「extent 树里第一个文件一个数据单元都没有」。三条读路的答案不一样：冷恢复与可写挂载拒；只读挂载照开，打开 inode 1 得 0 个单元、size 0（`crates/singlefs-core/src/mounted_read.rs:654` 那一支把标签 0 与缺席都当「一个单元都没有」）；池级 checker 不红。所以要改的是「重建成内存里一版」那一条路与它的形状（一版只有一个带数据的文件、至少一个单元），只读挂载那条路今天已经认这个形态。

另两种「没有文件」的盘上形态（**推的**，原型没做）：整条去掉 extent 上段那条叶条目，撞 `recovery.rs:3354` 的「不是恰好一条叶条目」；回收完成时去掉 inode 记录，撞 `recovery.rs:2505` 的「inode 树里没有第一个文件那条记录」。三种形态都被拒，这一格不分辨「没有文件长什么样」的候选，病根是共用的前提（恢复只认一个带数据的文件）。照跑前判据第 5 条，它是三·2 实现清单里的一项，不拿来判臂。

### 3.2 删完之后今天的写入口

```
E name=after_delete op=publish_first_file result=Err(FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries: 7 })
E name=after_delete op=sequential_write_to_inode_1 result=Ok txg=9 object_birth_deleted=3 object_birth_written=3 recover=FileRead root=(InstanceGeneration(1), CheckpointTxg(9)) bytes=16317
E name=after_delete op=publish_new_inodes_1 result=Ok watermark=3 recover=Failed root=Some((InstanceGeneration(1), CheckpointTxg(9))) failure=UnitMalformed { what: "extent 树里第一个文件一个数据单元都没有" }
```

- 再建第一个文件被拒（`crates/singlefs-core/src/transaction.rs:4642`），题面第二节那一句坐实。
- **给已经删完的 inode 1 顺序写照收**：同一个 (inode 1, 对象出生代 3) 又有了数据，冷恢复读回 16 317 字节。写入口不知道这个 inode 在删：`InodeRecord` 没有 nlink（`crates/singlefs-core/src/records.rs:56` 写死、`:85` 跳过）。这一格进三·2 清单：写入口要拒 nlink = 0 的 inode；它还是三·3 的一格坏镜像（nlink = 0 的 inode 名下的 extent 比上一版多）。原型里这一步之前把 nlink 开关关了，盘上那条记录被写回 nlink = 1，是原型的样子，不是结论。
- 建 inode 2 成（水位 3，号不复用照水位走），之后冷恢复还是因为 inode 1 那条空条目被拒。

### 3.3 size 不跟着截的中间态：出局

```
E name=size_kept step=batch_keep_4 size=179487 units=4 recover=Failed root=Some((InstanceGeneration(1), CheckpointTxg(6))) failure=InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" } writable_mount=Err(Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" })) checker_red=[]
```

「只删 extent、inode 记录等回收完成再动」这个形态的每个中间态今天都被冷恢复与可写挂载拒（`recovery.rs:3748`），按跑前判据第 1 条出局；要它不出局就得放宽恢复那一判（nlink = 0 的 inode 不核 size 与单元数），那是另一个形态。同一个镜像池级 checker 不红：checker 今天没有「单元数对得上 size」这一条，I-4.6 光靠 checker 判不出这一格，三·3 要记。从头部删（留下的单元不从 0 起）同样出局，撞 `recovery.rs:3374`（推的，没做）。

**什么现象会推翻 3.1**：`extents_of_the_first_file` 不再拒空的单元表，或重建一版不再经它。

## 四、U3　分批与续做

### 4.1 「续做同一批两次盘上字节不同 ⇒ 那个形态出局」：打中，但不分辨臂

```
E name=redo_same_instance images_byte_equal=true outputs_equal=true
E name=redo_after_crash instance_first=1 instance_redo=2 txg_first=6 txg_redo=8 rewritten_units_first=12 rewritten_units_redo=12 rewritten_units_byte_equal=0 data_pointers_equal=true inode_record_equal_but_change_count=true change_count_first=6 change_count_redo=8
E name=redo_after_crash_recover first=FileRead root=(InstanceGeneration(1), CheckpointTxg(6)) bytes=130536 redo=FileRead root=(InstanceGeneration(2), CheckpointTxg(8)) bytes=130536
```

同一个实例、同一份分配器拷贝上把同一批跑两次，两份盘逐字节相同。崩在这一批落盘之前、重开可写挂载（新实例 2，写行加暖机占掉 txg 6、7）之后再做同一批：留下的数据指针相同、inode 记录除改动计数外相同、读回长度相同，而这一批重写的 12 个单元**没有一个**逐字节相同（单元头里的写序带实例代号与事务号，出生代是这次的 txg，落点也换了）。

四句：

1. 分辨臂：**不分辨**。差别来自新实例与新 txg，任何「一批 = 一次发布」的形态在崩溃重开之后重做都一样；不崩溃就没有「续做」。照判据字面，全部候选一起出局。
2. 系统看不看得到：看得到，但这不是它能避开的东西（COW 加写序）。
3. 满足的分句：「续做同一批两次盘上字节不同」字面满足。它想判的是幂等（`.claude/kb/decisions/08-核心索引结构.md:121`：「意图声明目标状态，绝不声明增量。」），字面写成了盘上字节。
4. 改法：跑前条款没给改法。

照 `evidence-discipline.md` 那张表，这是「打中不分辨臂」：不拿它判臂，共用的前提另立一笔。要改的有两处字面，都交主 agent：正文第五节第 2 条，与 `.claude/kb/milestone/03-third-txn.md:84` 三·2 验收里的「同一批续做两次盘上字节相同（幂等）」。我提的收严（只在我的模型上量过、被攻过零轮）：比目标态而不是比字节——留下的 extent 集、inode 记录去掉改动计数、释放的落点集、记账里与实例和 txg 无关的行；「同一实例同一份分配器上重跑逐字节相同」留作另一条（量过，成立），它判的是发布的确定性，不是续做。

分辨得出臂的幂等攻击我只找到一个，是推的：意图记录里若带进度（「已删到第 K 个单元」），重做时进度与树对不上就是增量字段，撞 I-4.4（`.claude/kb/invariants.md:168`）。从尾部截、size 跟着截时进度就是 inode 记录的 size，意图里不用带。

### 4.2 只读挂载读到什么：读者看到一个变短的文件

```
E name=reader step=before_unlink inode_records=1 inode_1_present=true raw_nlink_on_disk=1 open_file_inode_1=Ok(units=6 size=179487)
E name=reader step=intent inode_records=1 inode_1_present=true raw_nlink_on_disk=0 open_file_inode_1=Ok(units=6 size=179487)
E name=reader step=batch_keep_4 inode_records=1 inode_1_present=true raw_nlink_on_disk=0 open_file_inode_1=Ok(units=4 size=130536)
E name=reader step=batch_keep_2 inode_records=1 inode_1_present=true raw_nlink_on_disk=0 open_file_inode_1=Ok(units=2 size=65268)
E name=reader step=batch_keep_0 inode_records=1 inode_1_present=true raw_nlink_on_disk=0 open_file_inode_1=Ok(units=0 size=0)
```

盘上 nlink 从意图那一步起是 0，读者照样给出文件，内容是 6、4、2、0 个单元；冷恢复同样（产物里 `E name=recover step=batch_keep_4 outcome=FileRead root=(InstanceGeneration(1), CheckpointTxg(6)) bytes=130536` 那一行）。这些中间态结构上都过得了今天的恢复与 checker，但「用户从没写过一个 130 536 字节的文件」：中间态合法（`.claude/kb/decisions/08-核心索引结构.md:123`：「中间状态必须每一个都是合法的文件系统状态」）只在读者把 nlink = 0 当「没有这个文件」时才说得通。读者今天分不出：记录里的 nlink 被跳过。进三·2 清单：`InodeRecord` 加 nlink；只读挂载的 `open_file`、冷恢复的读回、模型的读回答案从意图提交那一步起就是「没有文件」，不是等删完。这一格对 U3 的候选不分辨（只要 nlink = 0 留树都一样），是实现清单项。

### 4.3 续做挂在可写挂载里

事务员推的挂点是暖机循环之后、交回之前。三个形态（名字是我起的）：M1 挂载里把剩下的批全做完；M2 挂载里只做一批，其余在会话里每次用户发布之前接着做；M3 挂载里不做、只读出意图，会话里做。

- **M1 打中**（拒那一半量过，其余推的）。批数随文件大小无界；取号前的准入与预演要罩住取号之后的每一次发布（事实表 (d) 第 11、12 步），而 `crates/singlefs-core/src/mount.rs:2123`：「pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;」。每批还要新占 13 个槽、这段历史里不回收（第二节），近满时某一批被准入拒（第二节满池那一格）：挂载到这里要么失败（带着未完成意图的近满池挂不上可写），要么跳过续做往下走，后者撞 `.claude/kb/checks-owed.md:17` C1 那一行的「恢复路径里不许存在「跳过续做」的分支」。
- M2、M3 被拒时只是推迟，挂载照成。M3 撞 C1 同一行的前半句（「正常挂载后没跑续做即判红」），除非把「跑续做」定义成会话的第一件事；M2 在挂载里至少跑一批，C1 的字面过得去，取号前的预演只多算一次发布。这是我推的排序，被攻过零轮；三个形态各要多少次挂载期发布是一个数（M1：批数；M2：1；M3：0），没量别的代价。
- 原型上中间态的可写挂载都成（`E name=writable_mount step=batch_keep_4 result=Ok(WithFile units=4 size=130536)`）。挂载自己的写行与暖机会不会把 inode 1 的记录重写成 nlink = 1，原型量不了（nlink 是线程局部开关，不是记录的字段）；按 `records.rs:56` 推，今天任何一次重写那条记录的发布都会写回 1，`InodeRecord` 加字段之前这一格是真的。

### 4.4 批的粒度与测试旋钮

原型的批就是「留前 M 个单元」，M 由调用方给，固定脚本里造两批不要新机制；这与点名项容量那个只供测试的旋钮同形。按「一次发布的记录条数上限」定批大小的候选进不了测试（推的）：删除的一批不重写数据单元，被删的单元不进点名项，原型里每批重写 12 个单元、与删几个无关；上限 65 536 条在单文件上碰不到。按单元数定、旋钮直接给「每批删几个单元」的形态收得进（量过：三批各 2 个单元）。

## 五、U1　格式：`deleted_inodes` 与意图住哪

候选（名字是我起的）。意图住处：L1 单独一棵 logged_ops 树；L2 记账里一行；L3 树表里一条条目；L4 不另立意图，`deleted_inodes` 里有这个号加 nlink = 0 就是「inode N：无 extent、记录删除」这个目标态。建树时机：K1 mkfs；K2 第一个文件版本那次（照 deadlist 三棵零根的先例）；K3 第一次 unlink。

### 5.1 管理员回退：意图与删除队列都得随 R_old，两个方向都量到了

回退那次发布今天只换两种树的根，`crates/singlefs-core/src/mount.rs:4972`：「root: if entry.kind == TREE_KIND_EXTENT || entry.kind == TREE_KIND_INODE {」；记账行只有水位那一行另算（`mount.rs:4989`），别的照现行那一版。

方向一，删到一半回退到删之前：

```
E name=rollback_mid_delete before=FileRead root=(InstanceGeneration(1), CheckpointTxg(6)) bytes=130536 rollback_ok=true txg_after=7 units_after=6 size_after=179487 recover_after=FileRead root=(InstanceGeneration(1), CheckpointTxg(7)) bytes=179487 checker_red=[] accounting_rows_equal_to_the_deleting_version_except_watermark=true
E name=redo_after_rollback recover_after_rollback=FileRead root=(InstanceGeneration(1), CheckpointTxg(7)) bytes=179487 raw_nlink_after_rollback=1 redo=Ok(txg=8 units=4) recover_after_redo=FileRead root=(InstanceGeneration(1), CheckpointTxg(8)) bytes=130536 checker_red=[]
```

文件整个回来（6 个单元、nlink = 1）。意图若住在回退不换的地方（L2 的记账行；L1、L3、L4 的树若不加进那个 `if`），它照现行那一版留着，续做照它再删一批：第二行量的就是这一批做得成，管理员刚带回来的文件又短了，checker 不红。「意图留着」是按上面那一行代码推的（原型没有意图记录）；第一行末尾那个字段名起大了，它只核了待删占用那一行是 0。

方向二，回退进删除中间的一版：

```
E name=rollback_into_mid_delete target_txg=5 rollback=Ok txg_after=8 units_after=6 size_after=179487 raw_nlink_before=0 raw_nlink_after=0 inode_leaf_rewritten_by_the_rollback=false recover_after=FileRead root=(InstanceGeneration(1), CheckpointTxg(8)) bytes=179487 checker_red=[]
```

从留 2 回退到意图那一版：inode 叶不重写、直接取 R_old 那一片，nlink = 0 的记录带着 6 个单元回来。删除队列与意图若照现行那一版，而现行那一版已经删完、条目已去掉（推的，这一次回退的起点是留 2、还没删完），这个 inode 就是「nlink = 0、不在队列里、extent 没放完」，正是 `.claude/kb/invariants.md:172` I-4.7 说不存在的那种中间态，没人再回收它。

四句：分辨臂——**不分辨「要不要改回退」**（四个住处都要改），分辨「怎么改」：住树的（L1、L3、L4）在那个 `if` 里加种类、R_old 那棵树的节点走复活那条路，与 extent、inode 两棵同形；住记账行的（L2）要在照抄现行记账行时把那一行换成 R_old 的，水位是先例但它取的是 max、不是取 R_old。系统看得到（两版都在内存里）。这一格坐实的是 `08-核心索引结构.md:171` 那一句「`deleted_inodes` 也取 R_old 那一版的」（那一句标着推的、被攻过零轮），并把它推到意图上：**意图与删除队列要同进同退**。「待删占用」那一行若不恒 0，回退之后也要按 R_old 的状态重算。

### 5.2 建树时机

- K3（第一次 unlink 才建）打中，推的：回退的目标是建树之前的一版时，现行树表有这条条目、目标那一版没有，`mount.rs:4963` 那个按种类在目标那一版里找条目的 `expect` 是照「树表里 extent 树与 inode 树各有一条」写的，新种类照抄这个写法就是 panic；要么回退时把这条条目的根写成零，要么树表条目数随历史变（7 或 8），而 `crates/singlefs-core/src/recovery.rs:2187`：「const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 7] = [」按固定的发号次序核。
- K1 与「mkfs 一棵树都不建」对不上（树表第 0 版 0 条），要改 mkfs 的字节。
- K2 没找到反例：每个带文件的版本都有这条条目、根为零，回退两头都有；代价是新池新建文件多一条 200 字节树表条目、水位 19 变 20，动 `layout/01-first-txn.md` 登记的字节与断言 7 条的那几处（事实表 a3）。空的时候根回到零（不留空节点）时，「没有删除在进行」在盘上只有一种样子（推的）。

### 5.3 每个住处的代价（全是推的，按格式常量与量到的段形状算）

| 住处 | 多写什么 | 意图那次与末批各多几个状态 |
|---|---|---|
| L1 单独的 logged_ops 树，另有 `deleted_inodes` 树 | 两条树表条目各 200 字节；两棵各一个 16 KiB 节点 × 2 副本；各一条映射条目 55 字节、两条分配记录 | 主段 24 次写变 28 次：两态闭式 2²⁸ 量级，是今天 2²⁴ 的 16 倍 |
| L4 只有 `deleted_inodes` 树 | 一条树表条目；一个节点 × 2 副本 | 24 变 26：4 倍 |
| L2 记账里一行 | 34 字节，落在本来就重写的记账单元里；0 个新单元 | 不变 |
| L3 树表里一条条目 | 200 字节，树表单元本来就重写；0 个新单元 | 不变 |

中间的批不动意图与队列（意图只写目标态），多出来的只在意图那次与末批。L2、L3 便宜，但一条条目只放得下一个意图：记账 key 没有 inode 维（标签 2 + 树 ID 8 + 设备 4 + 代 8，value 8），同时两个待删 inode、或 U7 的另两个消费者都放不进（第七节）。

## 六、U6　崩溃点与流（找反例）

```
E name=closed_form publish=intent_keep_6 txg=5 writes=29 segments=4 segment_sizes=[24, 2, 1, 2] two_state_closed_form=16777223
E name=closed_form publish=batch_keep_4 txg=6 writes=29 segments=4 segment_sizes=[24, 2, 1, 2] two_state_closed_form=16777223
E name=closed_form publish=batch_keep_2 txg=7 writes=29 segments=4 segment_sizes=[24, 2, 1, 2] two_state_closed_form=16777223
E name=closed_form publish=batch_keep_0 txg=8 writes=27 segments=4 segment_sizes=[22, 2, 1, 2] two_state_closed_form=4194311
E name=closed_form_sum publishes=4 sum_of_states_minus_one_each=54525976
E name=closed_form publish=sequential_write_one_unit_after_the_delete result_ok=true writes=29 segments=4 segment_sizes=[24, 2, 1, 2] two_state_closed_form=16777223
```

- **代价数**（量过，两态闭式，不含原地覆写的第三态；`crates/singlefs-harness/src/memory_pool.rs:611` 的 `closed_form_state_count` 算的）：原型形态下每批一次发布、四段，前三次各 16 777 223，末批 4 194 311；意图加三批合计 54 525 976。对照层 0 全量那条流的 1 662 648 564（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:100`，那是三态数，口径不同，只能看量级）：加一段「意图 + 两批」大约多 3%。这个数是文件单元数的函数（6 个单元、extent 有下段），固定脚本那条流的文件更小，没扫。住处取 L1 或 L4 时意图那次与末批各乘 16 或 4（第五节表，推的）。
- **这条腿没枚举**：每次发布单独就是 1.7 × 10⁷，超过一段约 10⁶ 的线，照定义不跑。「崩在一批中间」的每个状态恢复之后 checker 红不红，这条腿没有数；交主 agent 排进崩溃放量（那边有前缀共享与 GPU 判器）。
- **自动命名分不出第几批的一种情形**（推的）：名字是 `crates/singlefs-checker-tier/src/crash_amplification.rs:2389` 的入口名、写种类、同一入口里同种的第几段。每批的四段种类序列相同（上面前三行的段形状一样）。续做若并进 `mount_writable` 这个入口（第四节的 M1、M2），删除的段与写行、暖机的段同名同种、序号接着数，暖机推几次随历史变，光看名字分不出哪一段是第几批。续做单独起一个入口名（挂载里调它也照记成它的段）就分得出；每批一次入口调用时名字相同、靠树上位置分，够用。
- **截断成 0 那一格并不并进来**：没找到并的理由，也没找到不并的反例。末批（留 0）的段形状是 [22, 2, 1, 2]，安全轮那一格是覆写类发布；两者写表不同，一个替不了另一个（只比了段形状，推的）。

## 七、U2、U7（找反例，都是推的：原型不写墓碑、没有意图记录）

**U2 墓碑**。

- 反例一，回退之后墓碑指着活对象。整个对象一条区间记录（对象 ID + 对象出生代 + 全区间）写下之后，管理员回退到删之前那一版（第五节方向一，量过：同一个 (inode 1, 出生代 3) 整个回来）。回退是向前的一次发布，先前写下的墓碑单元还在盘上、属于一条已发布的历史；扫盘重建读到它，会把一个活着的对象判死。D18 已定项 10 写的「同号的两个已发布对象的墓碑靠它分开」罩的是号被重发，罩不到同一个对象被回退带回来。要么回退那次发布给这条墓碑写退役，要么墓碑记录带死亡那次的 txg、重建时与回退的那条根比。
- 反例二，墓碑写在哪一批。写在末批：意图到末批之间扫盘重建，被删的数据单元还在盘上（在 defer 里）、自描述头齐全，没有任何输入说它死了，文件整个复活。写在意图那一次：从用户的 `rm` 返回起盘上就有死讯。按 D18 已定项 8 给墓碑的理由（删除在盘上不留痕迹时扫盘重建没有输入知道它死过），只有后一种站得住。
- 代价：墓碑容器是一个 32 KiB 码 3 单元 × 2 副本，意图那次多 2 次写，两态闭式再乘 4；扫盘重建今天没有实现，没有读者读它。第一版写不写，是「今天没人读」对「盘上形态以后补要改格式」的取舍，交用户。
- inode 记录、映射条目不写墓碑：没找到反例（D8 已定项 6 已答 inode 记录那一半）。

**U7 共用意图机制**。

- 反例：L4（意图就是 `deleted_inodes` 里的号）与 L2、L3（一条条目一个意图）都装不下另两个消费者。整理的目标态是「`R` 内不存在分配代 ≤ `c0` 且未释放的落点」，要的字段是区域与 `c0`，没有 inode；稀疏旁表的删除按映射 key。以后加一棵意图树：不认它的旧实现挂上可写、不续做就往下写，撞 C1，所以那一次只能是不兼容的格式位，不是加一个 keyspace 就完。
- 留法的下限（推的）：意图记录带种类码（unlink 是其中一种，0 保留），目标态字段随种类定长；第一版只登记 unlink 一种，但读者遇到不认识的种类要拒可写挂载。L1 自带这个位置；L4 要在 `deleted_inodes` 之外另留。第 5.3 节的状态数倍数是这个位置的代价。

## 八、我提的改法各修哪一格

全部只在我的模型上量过或没实现，被攻过零轮。

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 待删占用对 unlink 恒 0（B） | U4 扣两次 | 量过（第二节 `double_count` 第二行与 B 列） |
| 待删占用照记、准入不扣（E） | U4 扣两次，且 I-3.4 有对象可判 | 推的 |
| 从尾部截、size 跟着截 | U5 size 不跟着截出局那一格；4.1 的进度字段 | 量过（中间态冷恢复 `FileRead`、checker 不红，第二节引的 `state` 行对应的 `recover` 行） |
| 读者与写入口认 nlink = 0 | 4.2 读到变短的文件；3.2 给已删 inode 写照收 | 推的 |
| 回退时意图与删除队列一起取 R_old | 5.1 两个方向 | 推的（量到的是不这样做时的后果的一半） |
| K2：第一个文件版本那次建零根条目 | 5.2 回退跨过建树 | 推的 |
| 续做判据改成比目标态 | 4.1 | 推的（量到的是字节不同、目标态相同） |
| 续做单独起入口名 | 第六节命名 | 推的 |
| 墓碑写在意图那一次、回退时退役 | 第七节两个反例 | 推的 |

## 九、没打中的形状

- 中间态（留 6、4、2，size 跟着截）：冷恢复、只读挂载、可写挂载、池级 checker 四样在每一步都过（第二节那批行，加 `E name=checker step=… red=[]` 五行）。取样只有一段历史、每批 2 个单元；没崩在一批中间。
- 同一落点放两次：原型走今天「文件变短」那一支，没造出重复释放；没试两个意图重叠（第一版只有一个文件）。
- `deleted_inodes` 只存号、不带对象出生代：没找到反例，前提是它随所选根、回退时取 R_old 两条都守。
- 不写墓碑的那几类（inode 记录、映射条目）：没找到反例。
- K2 建树时机：没找到反例。
- 「没打中」都只有这一条腿的一次取样，照规则不能单独支撑结论。

## 十、这条腿自己的限度与没做什么

- 原型没有意图记录、`deleted_inodes`、墓碑；凡涉及它们的格都是按代码推的，逐处标了。nlink 是线程局部开关、不是记录字段，挂载与回退里「记录被重写时 nlink 变成什么」在原型上没有意义。
- 只有一组几何（两块 4 GiB 盘、两副本、默认环）、一个文件大小（6 个单元）、一种批大小（2 个单元）、一段历史。第二节那张表与第六节的状态数都是这一个点上的值，是文件单元数、批大小、extent 树形状的函数，没扫。
- 崩溃状态一个没枚举（每段超过约 10⁶）。这条腿与前一条腿合起来跑过的全量状态数 0，离约 10⁷ 的线还有全额。
- 满池那一格的「满」是在分配器拷贝上标出来的，盘上没写；式子里需求那一边的 8 880 128 怎么构成没拆。
- 回退进删除中间的一版只量了「现行那一版还没删完」的起点；「现行那一版已经删完」那一个起点是推的。
- 第五节第一行产物里 `accounting_rows_equal_to_the_deleting_version_except_watermark` 这个字段名不副实：它只核了待删占用那一行等于 0。
- 第 5.3 节与第七节的代价数是算的，不是量的。
- 没读禁读清单里的文件；没跑 checker 档测试、没跑门禁；没改冻结副本与仓里 `crates/`；不判正推那几格，不替主 agent 采纳。
- 负载：开跑前 `ps` 里没有性能测量、没有别的 cargo；三次跑都经内存包装（16G）与线程上限 6，没有等锁，退出码都是 0。
- 留在草稿目录的东西见交回。

## 十一、什么现象会推翻这份报告的打中

- U4：入库装置上照同一段历史重做，A 与 B 的差不是「留下的单元数 × 一份副本」。
- U5：入库装置上删到 0 个单元之后 `recover` 不报 `UnitMalformed`。
- 4.1：崩溃重开之后重做同一批，有单元逐字节相同到足以让某个形态过、另一个不过。
- 4.2：只读挂载在 nlink = 0 的镜像上不给出文件。
- 5.1：回退那次发布对新树的种类也取目标那一版的根（代码改了），或意图不住在回退照抄的结构里。
