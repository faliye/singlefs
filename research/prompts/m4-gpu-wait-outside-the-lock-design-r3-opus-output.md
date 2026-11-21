# m4-gpu-wait-outside-the-lock-design-r3 攻方腿（Opus）报告（2026-11-18）

立场：假设正文第四节十二条形态是错的；攻 Q1、Q2、Q3、Q5；第三轮三个角度：前两轮「已定」的撑不撑得住、未定项清单全不全、前两轮报出的数能不能核。前两轮攻过的八个角度不重复。
没读禁读清单里的三处。这一腿一条项目代码都没跑（不编、不跑探针）：文中「量的」只有 grep、读文件与读前两轮留在工作根的产物；探针写好了、没编过，凡写「探针该打出」的都是推的，读数等主 agent 跑「要主 agent 跑的命令」。
引代码写主工作区路径与行号：`git rev-parse HEAD` 是 4319aa2b035b2d8238d64aeb68b3ceb7674cc723；第三轮快照 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-snapshot/crates-sha256.txt` 点名的八份文件与 `kb-sha256.txt` 点名的两份 kb，`sha256sum -c` 全 OK（主工作区与快照同一份）。行号逐条 `grep -nF` 过。正文第四节的条目引 `research/prompts/_m4-gpu-wait-outside-the-lock-design-r3-body.md` 自己的行号。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-model/`。复跑（包装先删运行目录，再调同一模型目录里的 `tools/rerun.sh`；冻结副本那一格传这一腿在草稿目录建的仓副本，`.deliverables` 写成交接）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-opus/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-opus-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-model
```

| 文件 | sha256 |
|---|---|
| `probe/Cargo.toml` | `cda2de6fad3394150ac62c3fead7122e63ea0d7294fd077f58b1fa8fce225b71` |
| `probe/src/main.rs` | `ee48f802fd16c477a0d08fb40fceaf19a4f0faff8330e61bfaaf75025e24ed5b` |
| `probe/src/pool.rs` | `6aa86551b49aea85b64e084d908adc417142da73c46f92fd83cef294a3b43c9f` |
| `probe/src/issue_after_the_window.rs` | `c08ec5f365dea709948a215a08b16ead75d86d67d67653b44a309c5a9997d056` |
| `tools/patch_the_copy.py` | `7017c69377ad334bb0ffc521c23557ff2b7007f50534ff1496126903562e1d0c` |
| `tools/rerun.sh` | `074228a194b3c852cff354554a4bb09262882c795d00dd1bb29aa1300de0ef49` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

- `tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps` 那一段（连它上面那段「这一腿要跑的步骤」注释），`bash -n` 过；包装与第一、二轮那一份逐字相同（sha256 同上）。`patch_the_copy.py` 过 `python3 -m py_compile`。`out/` 是空的。
- **模型是什么**（读它的数之前先读这一段）：Rust 探针，只调 `singlefs-core`、`singlefs-checker` 与验证包内存盘的公开入口，核心层一行不改。起点池照第二轮辩方腿探针的 `pool.rs` 拷（有洞的数据段）。一段历史：拷一份起点池、可写挂载（V0）→ 前台改动（用户定的，放开扫：三种 × k）→ 在现行那一版（V_cur）上照会话签发那一支的写法签发今天在 V0 上会签的那一段，意图里的 `c0`、`segs0` 两臂（`ticket` 照 V0、`current` 照 V_cur）→ 对整个池跑池级 checker → 再走一趟今天的显式整理入口 → 再跑 checker → 拷一份可写挂载。选段不在探针里另写：起点池另拷一份、挂上、走今天的显式整理入口，读它签出的那条意图（就是今天在 V0 上会签的那一条）。
- 取样：前台改动 `one_byte_writes` k ∈ {0,1,2,4,8,16,24,32,48}、`sequential_write_of_the_second_inode` k ∈ {1,2,4,8,16,32}、`unlink_first_file` k = 1，各乘两臂，共 32 段。先跑一段打 `ESTIMATE`，估的全量超过 40 分钟退 3。推的：每段几次挂载、几十次内存盘上的发布与两遍池级 checker，全量几分钟；内存是两块 4 GiB 稀疏盘只占写过的块，几百 MB 以内。
- 崩溃状态：没取。打中的那一格判的是签发那一版本身（不崩也在盘上），不调枚举函数；这一腿与接手的腿合计 0 个状态。

## 各格判定一览

| 格 | 判定 | 打中的形（节） | 分辨候选或读法吗 | 量的还是推的 |
|---|---|---|---|---|
| Q1 前提「四元组相同 ⇒ 没变」 | 字面打中，后果被发布路径兜住 | 前台释放 R 里一个单元时 R 那块盘那一份读盘核不过：那条记录留在已分配、四元组不变、只多一个隔离记号，映射已离开它；剔过的计划照搬它，被 `PlannedUnitNotInTheVersion` 拒、这一趟记成 `Failed(Batch(..))`；今天同一段历史是丢意图（一·1） | 分辨「比四元组」与「比四元组加隔离记号」 | 读码推的 |
| Q1 七种前台路径 | 没打中 | `Truncate`、`Unlink`、`DataChangesPublishedTogether`、建删快照、回退、删除续做、抬 F 空发布：要么释放（记录变）、要么映射与字节都不动；建快照之后改写把映射条目挪进留存表、位置不变，发布路径按留存那一支读盘核（一·2） | — | 读码 |
| Q1 交 CPU 判与不确定的条目 | 没打中 | 交 CPU 的照旧由发布路径释放前读盘核；不确定的那几条过时，照抄而核 R / 删意图两条退路在算好的账上纠正（一·3） | — | 读码 |
| Q1 回锁不重判回退门 | 打中（未定项漏列） | 第 2、5 条没写回退门在哪判；照今天的次序它在取输入之前判，锁外期间前台的抬 F 空发布挤掉根环里的用户状态，回锁里发的那一次让状态少于 4 个而没人拦（一·4） | 分辨门判在取输入还是回锁 | 读码推的 |
| Q2 `c0`、`segs0` 照哪一版取 | 打中（未定项漏列；一种读法下 checker 红） | 候选段不排他，锁外期间前台的用户数据落进选中的段；`c0` 照票里那一版取时签发那一版里 R 有分配代大于 `c0` 的未释放记录，I-5.6 红（二·1） | 分辨 ticket / current 两种读法 | 推的，待主 agent 跑（探针已写） |
| Q2 没核过的记录被搬坏 | 没打中 | 签发那次发布不搬单元（计划是空的），签发之后第一批在有活意图那一支对 R 重扫（二·2） | — | 读码 |
| Q2 选中的段在新一版上全空 | 打中（弱，代价） | 删文件把选中的段腾空，回锁照旧签：签一次、删一次，两个根槽白用，`segs0` 照 V0 取时还把「什么都没做」记成有产出（二·3） | 分辨 `segs0` 两种读法 | 推的，待主 agent 跑 |
| Q2 资格只判「开成聚簇段」 | 打中（弱，措辞） | 提交内生块回落不开段、可落进选中的段；新进的是映射树节点（钉住）时照第 6 条括号里那一判还有资格，签了之后 R 腾不空，搬完能搬的再丢意图（二·4） | 分辨资格五条全判与只判一条 | 读码推的 |
| Q5 剔过的计划的写序列 | 没打中 | 同一个发布函数在现行那一版上发，变的只是搬哪几个单元；批小于 K、中间缺几个单元，写表的形状与今天最后一批相同（三·1） | — | 读码推的，没枚举 |
| Q5 `c0` 照票取的签发 | 打中（与二·1 同一格） | 签发那一版落盘之后的每个状态都红，现有整理搬迁那几条崩溃流里没有一条签出过「`c0` 早于上一版」的意图（三·2） | 同二·1 | 推的 |
| Q3 42 段旧底历史 | 射程写宽了 | 42 段里第一次整理发布都是签发（起点没有活意图）：40 段在记账树那一处被拒、2 段没签；「旧底上的整理发布不落盘」只量到签发那一次；这一轮的形态不再用旧底，这一句不承重（四·1） | — | 量的（读第二轮产物）+ 推的 |
| Q3 忙窗口基线 1.10 秒 | 撑不住那一句 | 50 毫秒一次是摆的，E185 真卡一次往返 4.5–10.3 毫秒截距；同一组产物里睡 0 与 5 毫秒两档最坏 130.6 与 165.5 毫秒，最坏等的大头不是等卡（四·2） | — | 量的（读判决与 E185 原行）+ 推的 |
| Q3 `lock_order` 不死锁 | 够得着，射程要写 | 靠的是锁序论证、不是那一次两线程的取样；候选甲若一批借一次 GPU 锁、每圈回锁读盘，或第 10 条拆出另一条线程，就出现「持 GPU 锁取会话锁」（四·3） | 分辨候选甲的借锁粒度 | 读码推的 |
| Q3 第二轮攻方离散事件模型 | 大半没有对象 | 只剩 `today` 臂各格与 `registration_window` 一格的定性结论对这一轮还有效（四·4） | — | 读模型推的 |

## 零、撤回这一腿前两轮给过、现在站不住的

1. 第二轮一·5 第 1 点「R 排他，前台只能让 R 里的记录变少（改写、删文件都是把记录释放）」：**撤回**。前台释放 R 里一个单元时，释放前读盘核核出 R 那块盘那一份对不上，那条记录不改成已释放：`crates/singlefs-core/src/transaction.rs:15730`：「释放之前读盘核校验和核出对不上（读不出也算）的那几份：逻辑上照样释放（映射条目这次不再写它），那一份在它那块盘上的分配记录」，下一行接「留在已分配、不改成已释放、分配代不变，跨度段置上隔离记号 0x4000」（转述第 15731 行）。改的是 `crates/singlefs-core/src/allocator.rs:2823`：「AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck;」，跨度字段不含它（`crates/singlefs-core/src/allocator.rs:75`：「/// 纯跨度，不含已释放标志与隔离记号。」）。所以前台还能做第三件事：记录四元组不变、只改隔离记号。后果见一·1（被发布路径兜住）。
2. 同一点括号「字节没变：记录没变、写时复制，那一槽没被重写过」之后「其余照锁外判过的分类」：**收窄**。字节那半句站得住；分类那半句不成立，同一个单元从「可搬」变成「隔离」而记录四元组不变（一·1）。
3. 第二轮一·5 第 3 点「停（`NoEligibleSegment` 这一类）照锁外那一版的 txg 记备忘」：**撤回**。备忘只在 txg 与现行那一版相等时算数（`crates/singlefs-core/src/compaction_plan.rs:1038`：「.filter(|noted| noted.checkpoint_txg == current_txg && noted.switches == switches)」）；锁外期间有发布时照 V0 的 txg 记，备忘永远对不上现行那一版、等于没记；照 V_cur 的 txg 记，又把 V0 上判的「没得选」安到 V_cur 上。站得住的只有「两版 txg 相同才记，不同就不记」。用户改动会把「该跑了」置上、绕过备忘（`mounted_session.rs` 第 3499 到 3506 行），所以哪一种记法都只影响定时兜底那一支，弱。
4. 第二轮一·5 末段「锁里多做的只有比记录、算账与今天本来就有的发布；不读计划外的盘、不交卡」：**收窄**。回退门要在回锁里重判（一·4），它读整个根环与每条有效根的树表，在锁里读盘。
5. 第二轮七节表一·5 那一行「二·1 空转 64 次：修（停照记）」：**收窄**，照上面第 3 条，停只在两版 txg 相同时记。
6. 第二轮四·1「候选乙读到复用槽：没打中」的理由「之前必经一次发布，txg 变、复核挡住」：**撤回理由，结论不动**。这一轮的形态不比 txg。换成的理由：槽被复用要先释放再分配，R 那块盘那条记录的分配代变了，第 5 条比记录剔掉它（有活意图那一支）；签发那一支锁外读到的候选段不进计划（二·2）。这一格是前两轮攻过的角度，只撤理由，不重攻。
7. 第二轮一·5 第 2 点「签发那一支……`c0` 取现行那一版的 txg」：**不撤**，第四节第 6 条把它丢了，见二·1。

## 一、Q1：第 5 条的前提

### 一·1　四元组不变而隔离状态与映射变了（打中前提字面，后果被兜住）

第 5 条原句：`research/prompts/_m4-gpu-wait-outside-the-lock-design-r3-body.md:36`：「拿现行那一版与票里那一版比起点在 R 里的未释放记录（盘、槽、跨度、分配代），记录变了的单元从计划里剔掉，其余照锁外判过的分类与 GPU 的核在现行那一版上算账、发布」。

**历史**（前缀固定：有活意图、R 里一个单元 X 在 R 那块盘那一份坏了；用户动作放开：前台什么时候、改写哪一个单元）：
1. 维护线程取输入（V0），锁外扫 R：X 判成可搬（分类先判钉住、再判隔离，隔离集是取输入那一刻会话上的那一份，`crates/singlefs-core/src/region_scan.rs:717`：「if quarantined_slots.range(covered_slots).next().is_some() {」；会话这一份在取输入时从分配器取走，`crates/singlefs-core/src/mounted_session.rs:5367`：「compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());」）。GPU 甲段读的是 R 那块盘那一份，坏了就判对不上、交 CPU 发布路径；坏在另一块盘则甲段看不出。计划里有 X。
2. 锁外期间前台改写 X 所在的文件并发布（`publish_user_change` 或 fsync 一次，用户定）：X 的映射条目这次不再写；释放前读盘核读 X 的两份，R 那块盘那一份对不上 ⇒ R 那块盘那条记录留在已分配、分配代不变、只置隔离记号，另一块盘那条释放；这一槽记进分配器上「发布记下、会话还没取走」的隔离表。许可它的是一·1 开头撤回第 1 条引的那两行。
3. 回锁：R 那块盘上 X 那条记录（盘、槽、跨度、分配代）与票里逐字相同 ⇒ 不剔；计划照搬 X。
4. 发布路径逐条核：X 的 key 不在现行那一版的映射与留存条目里 ⇒ `crates/singlefs-core/src/transaction.rs:11411`：「return Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { mapping_key });」（若 X 被快照留着、映射条目挪进留存表，位置是 X 那一槽，那一支读盘时两份里先核得上的那一份照搬，见一·2）。不是空间不够 ⇒ 这一趟 `crates/singlefs-core/src/mounted_session.rs:4884`：「CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),」，运行库停这一拍（`crates/singlefs-mount-runtime/src/lib.rs:678`：「CompactionRunOutcome::Failed(_) => false,」）。下一批取输入时取走隔离、R 里有隔离槽 ⇒ 丢意图（`crates/singlefs-core/src/mounted_session.rs:5631`：「if region_holds_a_quarantined_slot(region, &quarantined_slots) {」）。

今天同一段历史：那一批在锁里取输入时已经取走隔离，直接丢意图。差别：多一个 `Failed` 结局、一批白算；盘上一个写都没多（被拒在任何写之前）。

**四句**：分辨「比四元组」与「比四元组加隔离记号（或回锁时先取发布记下的隔离、R 里有隔离槽就丢意图）」两种读法；系统看得到（隔离记号在现行那一版的分配记录里，发布记下的隔离在分配器上）；满足 Q1 题面「这些单元的……钉住或隔离状态已经变了」那一分句；改法「比较加隔离记号」修这一格，改法「被逐条核拒掉的批记成单独的停因、不记整理失败」只修观测那一半。

**为什么后果被兜住**（推的，读码）：前台能让一个 R 里的未释放记录只改隔离记号的唯一一条路是释放它，而释放就意味着映射已离开那一槽；发布路径核源落点就是映射位置（`crates/singlefs-core/src/transaction.rs:11457`：「RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition {」），所以每一次都被拒在写之前。推翻它的观测：一段历史里四元组不变、映射仍指着那一槽、而那一槽已被隔离（我没找到这样的路）。

### 一·2　七种前台路径对 R 里的记录、映射、留存条目与钉住做什么（没打中，读码）

映射 key 是单元的身份（码 1 是树 ID、诞生代、写序；码 2、码 3 是出生树、诞生代、实例代号、出生序号，`region_scan.rs` 第 760 到 761 行的注释），不是「文件第几块」：改写一个文件换的是新 key，旧 key 的映射位置只在单元被搬（整理、或读到只坏一份之后会话另发的那一批搬迁）时变；搬就释放源槽。区域扫描把留存给快照的映射条目一并读进来（`crates/singlefs-core/src/mounted_session.rs:4925`：「mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(」）。

| 路径 | 对 R 里一条未释放记录 | 映射 / 留存条目 | 钉住 | 第 5 条剔不剔、发布路径兜不兜 |
|---|---|---|---|---|
| `Truncate`（N < S：意图 + 各批 + 最后一批，`crates/singlefs-core/src/mounted_session.rs:175`：「空间不够推抬 F 再试。N = S 只改记录、N > S 变长一次发布、N < S 意图 + 各批 + 最后一批。」） | 尾部单元释放（记录变）；读盘核不过时同一·1 | 尾部 key 删 | 不动 | 记录变的剔；一·1 那一种被拒 |
| `Unlink`（意图 + 从尾部全部分批，同一次调用） | 同上 | 同上；意图那一次只改 inode 记录，数据单元与映射照抄 | 不动 | 同上；意图之后、批之前的单元 key 仍在映射里，搬它与今天在那一版上算的计划相同 |
| `DataChangesPublishedTogether`（整份写与建 inode 并进一批） | 被换下的单元释放 | 新 key 进、旧 key 出 | 不动 | 同 `Truncate` |
| 建快照 | 不动 | 不动；之后改写时旧 key 从活映射挪进留存表、位置不变 | 不动 | 不剔；发布路径按留存那一支（`crates/singlefs-core/src/transaction.rs:11396`：「.kept_mapping_entry_of_key(&mapping_key)」）从盘上读两份、核位置项校验和、取先核得上的那一份（第 11483 行），与今天在 V_cur 上重扫算出的计划相同 |
| 删快照 | deadlist 条目放掉（记录变） | 留存条目删（`crates/singlefs-core/src/transaction.rs:10629`：「不多于一批就是最后一批：放掉剩下的全部、同一次发布删掉那一对码 14、15 条目。每条放之前按它留存的映射条目读盘核、逐盘核落点，」） | 不动 | 记录变的剔；读盘核不过时同一·1 |
| 回退 | 运行库没有回退入口，回退要交出会话、走卸载与重挂（第二轮二·3 读过，这一轮没变）；不在锁外那一段里发生 | — | — | 没有对象 |
| 删除续做 | 后台那一趟在整理批之前、锁里走（`mounted_session.rs` 第 4618 到 4644 行）；前台三种删除在自己的调用里走空 | 同 `Unlink` | 不动 | 同 `Unlink` |
| 抬 F 的空发布 | 不动（只动回收下界与 defer 队列，R 排他挡新落点） | 不动 | 不动 | 不剔，计划照旧对；但它挤根环，见一·4 |

钉住：豁免映射的三类（映射树各节点、树表、实例表链）住的槽要换成别的东西，先得释放再分配，R 排他挡分配；发布路径另核一遍（`PinnedUnitInThePlan`，`transaction.rs` 第 11419 行）。试过的形与范围见文末「没打中的形状」。

### 一·3　交 CPU 判与这一轮不确定的条目留到回锁之后（没打中，读码）

- 甲段判对不上而交 CPU 的：它们仍在扫描的可搬项里，CPU 那一判就是发布路径释放前读盘核，在回锁里、对现行那一版的落点做，与今天同一处。
- 这一轮不确定的：不进可搬项，只决定「照抄而核 R」还是「判定了的一个都没有就不发」（`mounted_session.rs` 第 5749 到 5767 行）。锁外期间它们被前台释放了，这份清单就过时：照抄而核 R 那一支在算好的账上发现 R 已空，照同一批删意图再发（第 5917 到 5949 行那一支），与今天重扫的结局相同；全被剔光就不发，下一批重扫。
- 第 8 条的连续不确定计数住在 GPU 实现里，这一轮形态不作废，第二轮二·2 那一格没有对象。

### 一·4　回锁不重判回退门（打中，未定项漏列；读码推的）

今天回退门在每一批之前、锁里、紧挨着推一批判（`crates/singlefs-core/src/mounted_session.rs:4777`：「if is_the_rollback_gate_judged && !is_this_publish_the_exception {」，下一行「if let Some(stop) = stop_of_the_rollback_gate(parameters, &*devices, allocator, current)」）。它判的是这次发布要写的那一槽与之后余量里的根走了之后根环里还剩几个不同用户状态（转述 `mount.rs` 第 4252 行的文档），不同状态按用户可见树的根指针去重。第四节第 2 条取输入的步骤与第 5 条回锁的步骤都没写它；照今天的次序它落在取输入之前。

**历史**：根环里不同用户状态正好压在门上（`states_after_the_next_publish_and_the_margin` = 4，取输入那一刻门开）；锁外期间前台一次写被空间拒、推一串抬 F（不改用户可见树的发布：占根槽、不添状态），再写成（添一个状态）；k 个抬 F 挤走环里最旧那几个状态的最后一条根就少 k 个状态、多 1 个。回锁：第 5 条照计划发，又挤走一条根，环里不同状态少于 4 个；今天的代码在锁里重判，停在 `CompactionPublishWouldLeaveFewerThanFourRollbackStates`。用户定的：前台写多大、池子多满、抬几次 F。

强弱：推的，没量；前台自己那一串抬 F 有没有被准入上限挡在「不少于 4 个状态」之上，我没读完 `push_one_floor_raise_within_the_admission_budget`，读完若前台那一串自己就守着 4 个状态、回锁那一次再挤一条仍可能到 3 个。破例批不判门，与今天同。

**四句**：分辨「门判在取输入」与「门在回锁里重判」；系统看得到（回锁时根环在锁里读得到，今天就在锁里读）；满足 Q4 角度「该是岔路却被写死 / 漏列」与 D26 已定项 13 第 5 条的加门（`.claude/kb/decisions/26-后台整理与放置回收.md:256` 那一条，转述：一次整理发布要写的根槽连同余量里的根一走、不同用户状态就少于 4 个时不发）；改法「回锁里、发布之前重判回退门，门关着就丢这一批、照门关记停」修，代价是回锁里多读一遍根环与有效根的树表。

## 二、Q2：第 6 条签发那一支

第 6 条原句：`research/prompts/_m4-gpu-wait-outside-the-lock-design-r3-body.md:37`：「回锁里对选中的段在现行那一版上重判资格（这期间被前台开成聚簇段就没资格，换段）与签发前那一判（空洞够不够装下 R 里要搬的），新进这一段的记录全算成要搬的；没资格算这一次没成，下一批选别的段。」

### 二·1　`c0`、`segs0` 照哪一版取，形态没写（打中：一种读法下 I-5.6 红）

今天签发那一支在同一版上建意图、发签发：`crates/singlefs-core/src/mounted_session.rs:5591`：「allocation_generation_at_issue: version.root.checkpoint_txg,」，下一行 `segs0` 也照这一版的分配记录数（第 5592 行），`version` 就是签发那次发布的上一版。三段之后「建意图」与「签发那次发布」分在锁外与回锁两处：意图若在锁外照票里那一版（V0）建好带回锁里签（`ticket` 读法），上一版却是现行那一版（V_cur）。条款要的是写下那一刻：`.claude/kb/decisions/26-后台整理与放置回收.md:76`：「| value | `c0` | 8 | 22 | 签发那一刻的分配代 |」；`.claude/kb/decisions/26-后台整理与放置回收.md:77`：「| value | `segs0` | 8 | 30 | 签发那次发布之前的记账第 11 项（全空聚簇段数，D5（快照 / 空间记账机制） 已定项 4）里 `R` 所在盘的值，只作观测量；不是签发发布写下的新账值 |」。第二轮我写过「`c0` 取现行那一版的 txg」，第四节第 6 条没收这一句。

**历史**（前缀：没有活意图、起跑判据为真；用户动作放开：锁外期间前台写多少、写什么）：
1. 取输入（V0）、锁外丙段打分选出段 R、扫描判出至少一个可搬、空洞够。R 不排他（意图还没签）。
2. 锁外期间前台写用户数据：用户数据的落点是这次挂载开过的聚簇段与活 R 之外最低的空槽（`crates/singlefs-core/src/allocator.rs:2930`：「let excluded_region = self.compaction.region_excluded_from_placement();」，此刻没有活意图、排除的是空；第 2937 行「device.lowest_user_data_slot_outside_the_region(」）。整理选的正是活槽最少、洞最多的段，R 里的洞低于别处的空槽时新数据就落进 R；R 没被开成聚簇段（用户数据不开段）。R 那块盘上多了分配代 = 那几次发布 txg（> V0 的 txg）的未释放记录。
3. 回锁：第 6 条重判资格——R 不在开过的聚簇段里，照旧有资格；新进的记录全算成要搬的，空洞够就签。`ticket` 读法：意图的 `c0` = V0 的 txg。
4. 签发那一版落盘：活意图的 R 里有分配代 > `c0` 的未释放记录。池级 checker 的 I-5.6（`crates/singlefs-checker/src/image.rs:424`：「pub const COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED: &str = "I-5.6";」；不变量表 `crates/singlefs-checker/invariants/07-分配器.md:11` 那一行，转述：每一版里活整理意图的 R 罩住的未释放记录，分配代都不大于 `c0`）判红。这一版不崩也在盘上，签发之后每个崩溃状态里只要它的根持久就红。

`current` 读法（`c0` = V_cur 的 txg，`segs0` 照 V_cur 的记录数）下新进的记录分配代 ≤ `c0`，不红；与今天在 V_cur 上锁内签的意图字段相同。

探针该打出（推的）：`ticket` 臂里 `new_records_in_region` > 0 的那几行 `i56_after_issue=violated`，`current` 臂全不是 `violated`；`RESULT … arm=ticket` 的 `i56_red_where_landed` 等于 `landed`，`arm=current` 的 `i56_red_after_issue=0`。翻面观测：`ticket` 臂有落进 R 的行而 I-5.6 没红（判法或我的读码错），或任何一行都没落进 R（这一份起点池上造不出，打中照旧是推的）。自检：`current` 臂签发之后 I-5.6 红、或没落进 R 而 I-5.6 红、或起点池本身有红项，都退 1。

**四句**：分辨 `ticket` 与 `current` 两种读法，不分辨「三段」与「今天」（`current` 读法下与今天同字段）；系统看得到（回锁时 V_cur 的 txg 与记录就在手里）；满足 Q2 题面「签到一个在现行那一版上已经没资格的段」的近邻——资格照判是有的，错在意图字段——与 Q5 题面「checker 判红」；改法「`c0` 取签发那次发布的上一版的 txg、`segs0` 照它的记录数」修这一格（与二·3 的 `segs0` 那一半同一个改法），别的改法都碰不到。

### 二·2　「没经甲、乙两段核过的记录被搬坏」：没打中（读码）

签发那次发布不搬单元：计划是空的（`mounted_session.rs` 第 5604 行 `&[],`，第 5605 行 `CompactionIntentStep::Issue(issued),`）。新进 R 的记录只进「R 里要搬几个」那一个数（空洞那一判）。签发之后第一批走有活意图那一支、在它自己的取输入上对 R 重扫（`crates/singlefs-core/src/mounted_session.rs:5634`：「let scanned = scan_of_the_batch(region, &*devices, compute)」），那时它们照常过甲、乙两段与发布路径的逐条核。所以 Q2 题面「签发之后第一批对 R 重扫，本来就会重核」成立；「签发这一步根本不需要锁外的扫描结果」只对一半：扫描还给三样——「R 里至少一个能搬的」那一条资格、这一轮不确定就不签（第 5559 行）、要搬几个的准数。全拿掉、在锁里把 R 那块盘上未释放记录全算成要搬的（保守），代价是空洞那一判更常不过，以及会签到全是钉住、已死、不一致单元的段（签一次、丢一次）。

### 二·3　选中的段在新一版上已经全空（打中，弱：代价）

**历史**：取输入（V0）选出 R；锁外期间前台删掉 R 里全部单元所在的文件（`Unlink`，一次调用走完各批）；回锁：R 不在聚簇段里、照旧有资格，第 6 条没写「R 里一条未释放记录都没有了就不签」，空洞那一判照「旧扫描的可搬数 + 新进的 0」判，够就签。签了之后下一批扫 R 全空，`IntentCompletedWithoutABatch` 删意图：两个根槽、两次发布白用，扣 `B_gc` 预算。之后签下一条之前那一判比「全空段数(现在) − `segs0`」：`segs0` 照 V0 取时 R 在 V0 上不空、现在空，差值 ≥ 1，把「什么都没做」算成有产出；照 V_cur 取时差 0，停在 `NoYieldFromTheLastIntent`。今天的代码在 V_cur 上选段：全空的段扫过没有可搬的，不是候选，不签。

探针该打出（推的）：`unlink_first_file` 两行 `records_in_region_now=0`（若 R 全是第一个文件的单元）、`issue=[published]`、`run_after=[…]` 里先删意图；`ticket` 与 `current` 两臂之后那一趟的结局在 `NoYieldFromTheLastIntent` 上分开。翻面观测：`records_in_region_now` > 0（R 里还有别的单元，这一格在这份起点池上造不出）。

**四句**：不分辨三段与今天之外的东西；系统看得到（V_cur 的记录在手里）；满足 Q2 题面「选到的段在新一版上已经全空」；改法「回锁重判资格时连同『R 里至少一个未释放记录』一并判，空了算没成」修。

### 二·4　资格只判「开成聚簇段」时，回落落进 R 的钉住单元不挡签（打中，弱：措辞；读码推的）

今天选段的资格五条里有一条是罩住的槽一个都不在钉住与隔离集里（转述；代码 `crates/singlefs-core/src/mounted_session.rs:5441`：「let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = read_of_the_version」那一段）。第 6 条括号里只写了「被前台开成聚簇段就没资格」。提交内生块（映射树节点、树表这些）在开放段装不下又没有全空段时回落，回落不开段、只排除活 R（`crates/singlefs-core/src/allocator.rs:1468`：「/// 同 [`Self::commit_generated_answer_without_open_segment`]，开段与回落都排除活整理意图的 `R`。」；`crates/singlefs-core/src/allocator.rs:4910`：「assert_eq!(preferring.open_segment(), None, "回落不开段");」）。池子满、正要整理的时候正是回落的时候：锁外期间前台一次发布把一个映射树节点回落进 R，回锁照括号那一判照旧有资格、签了；之后各批搬走能搬的，R 里留着那个钉住的节点腾不空，停在 `NothingMovableButTheRegionIsNotEmpty` 丢意图（直到哪次发布重写了那个节点）。今天在 V_cur 上选段，这一段没资格。

探针只报这一格（`exempt_units_in_region`），不造回落（起点池不满）。**四句**：分辨「资格五条全判」与「只判开没开段」；系统看得到；满足 Q2 题面「签到一个在现行那一版上已经没资格的段」；改法「回锁照 V_cur 判资格五条（聚簇段、活 R、钉住与隔离、被拒段、至少一个能搬的——最后一条照旧扫描判、新进的按保守算）」修。

## 三、Q5：崩溃语义

### 三·1　剔过的计划在现行那一版上的写序列（没打中，读码推的，没枚举）

剔过的计划交给同一个发布函数，上一版就是现行那一版（`publish_the_planned_compaction_step_falling_back_on_the_region_check`，`mounted_session.rs` 第 5867 行起），与今天锁内那一批走同一段：目的落点在发布时现取、照抄或删意图在算好的账上核、写表是「搬迁副本 + 这次重写的角色 + journal 记录 + 根槽 FUA + 系统配置轮换 + 屏障」。与今天锁内整批重算相比，同一输入（同一个 V_cur）下变的只有搬哪几个单元：

| 剔过的计划与今天的计划不同的地方 | 盘上多出什么写序列 | 现有崩溃流罩不罩 |
|---|---|---|
| 批小于 K 而 R 里还有能搬的（剔掉了几个、没补上） | 没有新形状：写表同一批 K' < K 个单元的那一批 | 罩：`.claude/gate.d/stage-inputs.tsv` 第 68 行 `crash-case:compaction-relocation-stream` 等整理搬迁那几条里每条意图的最后一批本来就小于 K；批大小只改副本个数 |
| 搬的那几个在 R 里不是槽号最前的那几个（中间缺） | 没有：次序照槽升序，只是集合不同 | 罩，同上；落点由发布时现取，与搬哪几个无关 |
| 剔掉之后 R 被这一批腾空、照同一批删意图 | 没有：今天照抄而核 R 那一支的退路 | 罩：第 68 行那几条的最后一批；腾空的退路另有只供测试的开关进 |
| 计划里有这次发布本来就重写的单元 | 没有：跳过集合在回锁里经发布路径核出，与今天同一圈 | 罩，同上 |
| 签发时 `c0` 照票取（二·1 `ticket` 读法） | 有：签发那一版本身违 I-5.6 | 不罩：现有流的签发都在同一版上建意图；这一格要的不是新流，是改法 |

推翻它的观测：在某个 V_cur 上剔过的计划发出一次今天的计划写不出的写（例如落点落进活 R、或源落点不是映射位置而写过去了）。这两样发布路径各有一判在写之前拒（落点取时排除活 R，`allocator.rs` 第 2930 行；源落点核，`transaction.rs` 第 11457 行）。

### 三·2　没取崩溃状态的理由

- 二·1 打中的那一格不靠崩溃：签发那一版落盘就红，崩溃状态只会多出「那一版的根没持久」那几个（不红）与「持久了」那几个（同样红）。
- 三·1 是写序列形状的比较，探针里摆崩溃枚举要先在验证包的来历表登一行（`crates/singlefs-verification/src/crash_point_replay_provenance.rs` 文件头：没登的名字一跑就拒）、写崩溃判定库；这一腿不改验证包。要量的话，现成的整理搬迁那几条流（第 68 到 72、107、160 到 162 行）覆盖了批大小与腾空两种形状。

## 四、Q3：前两轮的数撑不撑得住它那一句

### 四·1　第二轮 42 段旧底历史 ⇒「旧底上的整理发布不落盘」：射程写宽了，这一轮不承重

正文第三节第 3 条：`research/prompts/_m4-gpu-wait-outside-the-lock-design-r3-body.md:24`：「3. 旧底上的整理发布不落盘：第二轮 42 段历史 `stale_publish_made=0`，被发布路径的释放目标核拒掉（射程：没有活意图、三种前台改动、k 不超过 48）。」

第二轮辩方的起点池从没跑过整理，挂载也没有活意图可续做；所以每段历史在旧底上的第一次整理发布都是签发（不搬单元）。读第二轮那一趟留在工作根的产物（量的）：

```text
$ f=/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet-run/out/stale-base-kinds.out
$ grep -c '^HISTORY name=stale_base_kinds arm=stale_weak ' $f
42
$ grep '^HISTORY name=stale_base_kinds arm=stale_weak ' $f | grep -o 'compaction=\[[^]]*\]' | sort | uniq -c
     40 compaction=[published=0 outcome=Failed(Batch(Publish(ReleaseTargetAlreadyReleased_{_unit:_AccountingTree,_device:_DeviceIdentity(0),_slot:_SlotNumber(51219)_})))]
      2 compaction=[published=0 outcome=Stopped(HolesOutsideTheRegionFewerThanItsUnitsToMove_{_holes:_0,_units_to_move:_1_})]
```

40 段全在同一处被拒：签发那一次要释放 V0 的记账树（槽 51219），前台的发布已经把它换下。2 段在签之前就停了。所以：
- 那一句量到的只是「旧底上的签发」，不是「旧底上搬一批」；判决那一格应加「射程：只到每段第一次整理发布、即签发」。
- 推广到搬一批（推的）：记账树每次发布都重写，任何一次前台发布之后旧底上的整理发布都要释放一个已释放的记账树槽，被同一判拒。这条推理没量过；推翻它的观测：一次不改记账的前台发布（我没找到）。
- 这一轮的形态不在旧底上发布（第 5、6 条都在现行那一版上发），这一句不再承重；承重的换成发布路径对现行那一版的逐条核（一·1）与 `c0` 的取法（二·1）。

### 四·2　忙窗口基线 ⇒「今天读最坏等 1.10 秒」：撑不住那一句

正文第三节第 5 条：`research/prompts/_m4-gpu-wait-outside-the-lock-design-r3-body.md:26`：「5. 今天锁内等的代价：替身卡一次调用睡 50 毫秒时读最坏等 1.10 秒、写一次调用 p99 是 1.17 秒（第二轮忙窗口基线，一趟）。」它带着「替身卡一次调用睡 50 毫秒时」，可第二轮判决「交用户的」与第三节都把它当「今天锁内等的代价」用。三处够不着：

1. **50 毫秒是摆的，比真卡大 4 到 10 倍**。E185 量的真卡一次交卡：`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:201`：「| Q11 a、Q12 b | r1 第三段第 54、136 行 | 卡 0 `E7RESULT name=q11_q12 index=0 card_valid=true intercept_seconds=0.004492 slope_seconds_per_byte=4.117289e-10 one_range_median_seconds=0.005244 largest_two_points_slope_seconds_per_byte=4.172332e-10`；卡 1 `E7RESULT name=q11_q12 index=1 card_valid=true intercept_seconds=0.010300 slope_seconds_per_byte=1.266004e-9 one_range_median_seconds=0.005393 largest_two_points_slope_seconds_per_byte=1.191920e-9` | 不单判 |」。按 E185 量到的最大一批 786432 字节算（算术）：卡 0 约 4.8 毫秒、卡 1 约 11.3 毫秒一次。与它对得上的是睡 5 毫秒那一档，不是 50 毫秒那一档。
2. **同一组产物里，最坏等的大头不是等卡**。第二轮判决同一节：`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-main-verification.md:22`：「睡眠 0 与 5 毫秒两档、四个读线程：被挡的取样最坏等 130610 与 165487 微秒」。睡 0 毫秒（替身卡不等，只剩 CPU 上算与锁里的发布）最坏已经 130.6 毫秒；睡 5 毫秒只多约 35 毫秒（各一趟，推的差值）。第三段（回锁里发布）照旧在锁里，所以「拆到锁外」在 E185 那一档的卡上换到的读最坏等，照这两行看是几十毫秒，不是 1 秒；卡挂住时（约 40 秒）才是大头，那一格前两轮是读码算的、没量。
3. **最坏那一个取样不是「一批的锁内等」**：50 毫秒四读线程那一行 `blocked_publishes_four_or_more=11`（第 20 行），被挡的取样里有 11 个在拿到锁之前后台连着发了 4 次以上：被维护线程连着越过，等的是几批，不是一批。最坏那一个属于哪一桶，产物没打。一趟、一个池子、一种选段圈数。

所以那一句要写成「替身卡每次调用睡 50 毫秒这一档，读最坏等 1.10 秒（一趟，含连着被越过几批）」，交用户时与 E185 那一档（睡 5 毫秒，最坏 165 毫秒；睡 0 毫秒，最坏 131 毫秒）并列；写线程 p99 1.17 秒同理。

### 四·3　第一轮 `lock_order` ⇒「放掉 GPU 锁再回会话锁不死锁」：够得着，靠的是锁序，射程要写

第一轮探针只摆了两条线程：一条持 GPU 实现那把锁一段时间、放掉再取会话锁，一条前台切到 `gpu`（会话锁里取 GPU 锁）。一次取样说明不了「不死锁」；撑住这一句的是锁序：这一轮形态下会话锁 → GPU 锁的嵌套只剩维护线程自己判起跑与开批（`crates/singlefs-core/src/mounted_session.rs:3419`：「.put_the_switch_requested_from_the_next_batch_in_effect();」那一处问实现，第 11 条把前台切换的校验换成读快照），GPU 锁 → 会话锁的嵌套没有（第 3 条算完先放）。两样都在维护线程上、先后不交叉，所以不成环。射程要写进那一句：
- 候选甲（选段每圈回锁里读）若一批只借一次 GPU 锁、借到算完（今天就是这样借：`crates/singlefs-core/src/mounted_session.rs:5160`：「let mut implementation = gpu_compute.lock();」罩住整批），每圈回锁读盘就是持 GPU 锁取会话锁；要每圈放、每圈借。
- 第 10 条候选「另一条线程」若是把锁外那一段挪到另一条线程、维护线程照常一拍，维护线程判起跑那一处在会话锁里问实现、等锁外那条线程放 GPU 锁：不死锁，但会话锁里等满锁外那一段（缺口 3 回来，第二轮六·3 同形）。
这两条是读码推的，没量。

### 四·4　第二轮攻方离散事件模型对这一轮还有哪几格有效

它是照第二轮正文第三节（比 txg、作废、k 与 N 两个下限）写的（`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-model/model/form_model.py`）。这一轮不作废、没有 k：`form` 臂的 `hang_brought_back_into_the_lock`、`yield_and_floors`、`idle_pool_spin`、`hang_with_accepts_only` 各格都量的是作废与 k，没有对象；`exception_batch_hang` 的 `form` 臂把破例批放在锁里，这一轮第 8 条让它出锁，也没有对象。还有效的：各场景 `today` 臂（卡挂住时读等满约 40 秒，今天的代价那一侧，时长是拍的）；`registration_window` 的定性结论（`try_lock` 失败到登记之间后台至多再取锁两次）与作废无关，第 9 条照用，但那一格的 `form` 臂是带作废跑的（`form_model.py` 第 489 到 491 行：k 不设、前台每秒 100 次 fsync），要引数得在不作废的模型上重跑。

## 五、改法各修哪一格（都只在我的读码与我的探针上过、被攻过零轮；这一腿没有一格量过）

| 改法 | 一·1 隔离记号 | 一·4 回退门 | 二·1 `c0` | 二·3 R 已空 | 二·4 钉住进 R | 撤回第 3 条 备忘 | 四·2 那一句 | 四·3 候选甲 |
|---|---|---|---|---|---|---|---|---|
| 意图的 `c0`、`segs0` 取签发那次发布的上一版（V_cur） | 不碰 | 不碰 | 修（推的，待主 agent 跑：探针 `current` 臂） | 修 `segs0` 那一半（推的，待主 agent 跑） | 不碰 | 不碰 | 不碰 | 不碰 |
| 回锁照 V_cur 判资格五条，另加「R 里至少一条未释放记录」 | 不碰 | 不碰 | 不修 | 修（推的） | 修（推的） | 不碰 | 不碰 | 不碰 |
| 第 5 条比较加隔离记号，或回锁先取发布记下的隔离、R 里有隔离槽就丢意图 | 修：照今天丢意图（推的） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 被逐条核拒掉的批记成单独的停因、不记整理失败 | 修观测那一半（推的） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 回锁里、发布之前重判回退门，门关就丢这一批 | 不碰 | 修（推的） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 停只在两版 txg 相同时记备忘 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） | 不碰 | 不碰 |
| 那一句改写成带档位的，与 E185 那一档并列 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（措辞） | 不碰 |
| 候选甲每圈借、每圈放 GPU 锁；第 10 条不把锁外那一段挪到另一条线程 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） |

## 没打中的形状

- Q1 七种前台路径：一·2 那张表，读码；取样范围是第四节第 5 条比的那四个字段、区域扫描的分类五类（`region_scan.rs` 第 700 到 757 行）与发布路径逐条核的四种计划错（`transaction.rs` 第 11370 到 11510 行）。
- Q1 映射不动记录就改指向：映射 key 是单元身份，指向只在搬时变；搬只有整理与会话另发的那一批，都释放源槽。没找到「映射仍指着那一槽、那一槽被隔离」的路。
- Q1 钉住状态变：钉住的三类要换槽先得释放再分配，R 排他挡分配。
- Q1 冻结的发布在锁外那一段里重发：重发成了现行那一版跟着换、分配器整份换成那一次成立之后的那一份（`resend_the_frozen_publish`），冻结的若是删意图那一次，回锁时没有活意图，发布路径照 `NoLiveCompactionIntent` 拒在写之前；若是搬一批，那几个单元的记录变了，第 5 条剔掉。
- Q2 签发搬坏没核过的记录：二·2。
- Q5 剔过的计划写出今天写不出的序列：三·1 那张表，读码，没枚举。
- 由用户决定的动作：Q1 放开的是改写哪一个单元、什么时候、几次；Q2 探针放开三种前台改动 × k（32 段）；一·4 放开写多大、抬几次 F，没进探针。

## 这条腿自己的限度

- 探针没编过、没跑过：子 agent 不起 cargo。API 名与签名逐个 `grep` 过公开性，编不过由承接的腿改。`issue_after_the_window.rs` 有一次追加写了 157 行，超过「每次写进文件不超过 150 行」那一条。
- 探针造签发用的是 `transaction::publish_compaction_step` 照会话那一支的写法，不是这一轮形态的代码（形态还没有代码）；`ticket` 臂的意图是今天在 V0 上会签的那一条原样带过来，代表「锁外建好意图带回锁里签」那一种实现。
- 二·1 打不打得中取决于这一份起点池上前台的用户数据落不落进选中的段；没落进的话那一格照旧是推的（探针按行报 `new_records_in_region`）。
- 一·4 的历史没进探针，前台抬 F 那一串的准入上限没读完。
- 四·2 的「大头不是等卡」只拿两行各一趟的最坏值比，最坏值噪声大。
- 没调崩溃枚举函数，0 个崩溃状态（三·2）。

## 要主 agent 跑的命令

1. 探针整份复跑（包装先删运行目录 `…-r3-opus-run`，再调同一模型目录里的 `tools/rerun.sh`；`tools/rerun.sh` 里拷仓副本、拼探针包、`cargo build --offline --release -p gpu-wait-outside-the-lock-r3-probe`、跑编出来的二进制）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-opus/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-opus-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-model
```

   - 在哪起：仓根（命令里都是绝对路径），新 shell。冻结副本那一格是这一腿建的仓副本（`.deliverables` 写成交接），`rerun.sh` 把它拷进运行目录再改，副本本身不动。不调枚举函数，0 个崩溃状态。范围声明闸要它时照 `.claude/hooks/run-scope-guard.sh` 文件头补（实验复跑）。
   - 跑完看：`TASK_SERVER_RESULT` 那一行 `exit=0` 算跑完（退非 0 的件数为 0）；`task-server/task log <号> --all` 里依次有 `name=rerun_roots`、五个文件的 sha256（该与上面那张表相同）、`name=patched`、`name=build done`、`name=exit_codes issue_after_the_window=0`、`BASELINE`（`start_pool_violated=[]`）、`TICKET`（有 `region_start=`）、`ESTIMATE`、两行 `RESULT name=issue_after_the_window arm=…`、`WALL_SECONDS`、`SELF name=self_checks checked=3 failed=0`、`name=rerun_done`。逐段 `HISTORY` 行在运行目录 `out/issue-after-the-window.out`。
   - 判这一腿的话：二·1 看 `arm=ticket` 的 `landed` 与 `i56_red_where_landed`（相等且大于 0 算打中坐实）、`arm=current` 的 `i56_red_after_issue`（该是 0）；二·3 看 `kind=unlink_first_file` 两行的 `records_in_region_now`、`issue=`、`run_after=`；二·4 看各行 `exempt_units_in_region`。翻面条件照二·1、二·3 写的判。
   - 退 3：估时超过 40 分钟，交承接的腿缩 k 的取样。退 1 而有 `✗ 自检：` 行，或 `TICKET … intent=none`：探针判不出它要判的，数作废，交承接的腿修。编不过：`out/build.log` 末 80 行打在日志里，交承接的腿修 API。
   - 内存与时间：推的，一个进程，两块稀疏盘与几份拷贝几百 MB 以内；编译 release 几分钟，跑几分钟。
   - 承接：新派同立场的腿，派发提示写「承接：research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-output.md」与任务号，它读 `task-server/task log <号> --all` 与运行目录 `out/`，把二·1、二·3「探针该打出」换成读数、按翻面条件改判定。

2. 引文核对（文本工具，交回前这一腿自己跑过，末行贴在文末）：`python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-output.md`。

## 没做什么

- 不判正推那几格（Q4、Q6 归正推腿），不替主 agent 采纳；五节的改法都是零轮，进产品代码之前要照 `.claude/rules/three-way-inference.md` 在探针里做 A/B。
- 没往统一任务服务交任务，探针没编没跑，判决要引的数都没复跑（命令在上一节）。
- 副本上没有量出的数；仓副本只建了、没改（探针拼进的是运行目录里的拷贝）。
- 没删 /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-opus/copy：交主 agent 复跑探针当冻结副本用，`.deliverables` 写成交接；复跑完由主 agent 删。
- 草稿目录里另有 `rerun-template.sh`、`rerun.sh`、`rerun-steps.txt`、`wrapper-draft.sh`、`minimal-copy.txt`（几十 KB，脚本草稿与建副本的输出），不是副本、不是编译目录。

## 引文核对末行

`python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-output.md` 退 0，末行（本节写进之前那一版跑的）：

```text
  ✓ 核了 32 处引文、3 处裸行号，对上 32 处，没判 3 处（其中裸行号没有引文可比 3 处；逐处列在上面）
```
