# m4-gpugc-mount-runtime-code-r3：云端攻方（Opus）报告（2026-11-09）

## 复跑

探针没编过、没跑过（腿不交任务），下面凡标「待主 agent 跑」的都是读码推的。整条命令在「要主 agent 跑的命令」一节。

| 文件（模型目录 `research/prompts/m4-gpugc-mount-runtime-code-r3-opus-model/`） | sha256 |
|---|---|
| `src/attack_r3_accept_floor.rs` | `284379b729f376bfabcdde22d69fb909ede903c7d0d54d6e35c5d99a6f474bc9` |
| `tools/patch_the_copy.py` | `6aac4449bf94e7ccff720e3ef35782a158a9f518d8e5d8c4dbc4220b723d59d0` |
| `tools/rerun.sh` | `7ae57435853dae7be15c96466660903fd9bc69ea1dbe7c8f85ad4d090f31e989` |
| `tools/rerun-from-a-clean-work-directory.sh` | `407e31a9625e258ff43ad76278f6fcda0060cda6b06b809cbeb4fb859b72a48b` |

探针的内存与时间（推的）：一个测试函数按次序跑，`--test-threads 1`；填盘按任务 11298 的 `elapsed_seconds` 52–120 秒一次、至多 6 次；一段历史一次挂载，按任务 11298 整趟 367 秒推，一次不到几秒；总预算 2400 秒，每臂之前查，到了打 `name=stopped_by_the_time_budget` 就停。内存按「满盘一份约 0.7 GiB」的旧读数（`~/.local/share/singlefs/work/m4-gpugc-queue-walk-on-a-really-full-pool/result.md`，当时没复核）推：同时留 3 份填好的盘、1 份在用，加 256 MiB 的放不下的写，运行 5 到 8 GiB，不含编译。服务撞上限重排同一条命令是安全的：包的那一层只清带记号的草稿目录。

## 各格判定一览

| 格 | 判定 | 依据的成色 |
|---|---|---|
| T1 连续 `accept` 被拒时根环与状态数 | **打中（待量）**：状态数不足 4 之后，每次被拒的 `accept` 都推 5 到 7 次空发布，不收敛，环走完只剩 1 个状态 | 推的，待主 agent 跑 `name=crash_loop_*` |
| T1 主 agent「只删 1 个文件写不进是 D16 的结果」 | **说法不全，行为不违反条款**：差的 52 槽压在 defer 里是 D16 回收门槛，但 3392 个空槽拒写是 D28（挂载期承诺量）已定项 1 扣出来的；正文那个 -1017100 是推串之前的读数，推串之后是 -558348 | 读数量的（任务 11298、11320 日志）；拆账推的 |
| T1 回退门的账、预算、备忘、破例 | 没打中（读码） | 推的，翻面见第一节 |
| T1 D16 已定项 7、17（`accept` 写失败、冻结、待发放回） | 没打中（读码）；逐写位、逐屏障位扫描待量 | 推的，待主 agent 跑 `name=fault_durability` |
| T1 第二轮三处修补被 `accept` 抬 F 破坏 | 没打中（读码）；队列那一处在后台任务停着时够不着，是另一格 | 推的 |
| T1 串里写盘报错之后扣住位 | **打中（轻，待量）**：串断了之后 F 已到上限、不再推，扣住位留到下一串或卸载；C516、C546 那一格 | 读码；待跑 `held_slots` |
| T2 未定项清单 | 逐条答在第二节；**多出 4 样清单上没有的** | 读码 |
| T3 报出的数 | **四处要改口径**：-1017100、15.11 GiB、任务 11441、任务 11339 | 量的（`task-server/task status`、`log`） |
| 走不到的理由 | **成立** | 现查 |
| 类型与名字 | **不成立（低危，一句文档）**；第二轮两处修到了 | 现查 |
| 不兼容旧代码与旧格式 | **成立** | 现查 |
| 不新建判定库 | **成立** | 现查 |
| 所有任务多机可用 | **成立**；15.11 GiB 不当依据，这一批不收 | 现查 |
| 测试优先多线程 | **成立** | 现查 |
| 带 --item 的命令与项 | **成立**（崩溃点重放项）；形式化项 **推不出**（腿不跑 `--list-items`） | 现查 |
| 新崩溃流多罩了什么 | **成立**；第 115 批不加流，**够**（推的） | 现查 + 读码 |

这一轮新提的改法都算零轮，写在第一节末尾。

## 一、T1「已定」项撑不撑得住

### 1. 连续 `accept` 被拒：状态数不足 4 之后推不停

历史（只固定前缀与故障，写什么、崩几次放开）：写满的池上状态数掉到 4 以下，之后调用方一直重试一次写。

逐步对应被判对象：

1. 上限取法：「let fourth_newest = distinct_states_newest_first」（`crates/singlefs-core/src/mount.rs:4328`）接「.or(oldest_valid_root)?;」（`crates/singlefs-core/src/mount.rs:4331`），状态不足 4 个时上限是环里最旧有效根；条款同文「非空有效根不足 4 个时取最旧的有效根」（`.claude/kb/decisions/16-发布语义.md:38`）。
2. 环是满的时候，最旧有效根高于 F：抬 F 那一串新写的根盖掉最旧的几条，最旧有效根随之前移。于是「if ceiling <= current.root.rollback_floor {」（`crates/singlefs-core/src/mount.rs:4842`）那一判永远不成立，不会停在 `FloorAlreadyAtTheCeiling`。
3. `accept` 被空间拒就走同一个推法：「let pushed = push_one_floor_raise_for_a_refusal_for_space(」（`crates/singlefs-core/src/mounted_session.rs:6735`），循环到预算满。只有「let a_raise_exceeds_the_budget =」（`crates/singlefs-core/src/mount.rs:5006`）那一判停它，一次准入至多 7 次；串长 2 或 3，所以一次被拒推 5 到 7 次（推的）。
4. 下一次 `accept` 预算从 0 起。空发布不是状态，环走一圈之后只剩最新那个状态。

走到「不足 4」的路：`S = 8` 时门留的余量够同一次挂载里三次「推串加一次发布」（按串长 3、2、2 算，门关时 `当前 − t4 ≤ 2S − 1 = 15`，第 2 次之后 t2 距环尾 `15 + 7 − 2 = 20 ≤ 23`，第 3 次轮到 t1 那个状态，它的根一直延到门关那一条；推的）。所以挂着不崩走不到。要靠不经门的发布：崩了再挂载（写行加暖机至多 3 次，续做至多 5 次，「下一次可写挂载自己的发布不经门：写行加暖机至多 3 次，续做一条意图至多 5 次，合起来至多 8 次。S = 8 时一次挂载落在余量里；S 更小时一次挂载就可能用完余量。」（`crates/singlefs-core/src/compaction_plan.rs:1546`）），或破例那一次发布。那句余量算术没算 `accept` 的串：门关之后崩一次（3 次）再来两次推串的写，`S = 8` 时 `15 + 3 + 7 − 2 = 23` 刚好还在；崩两次，或崩一次带活意图（8 次），`S = 8` 就掉；`S = 7` 崩一次就是 `13 + 3 + 7 − 2 = 21 > 20`（推的）。mkfs 默认 S 随槽距取 5 到 8（`crates/singlefs-core/src/root_ring.rs:92`）。

探针臂 `arm_crash_loop`：写满（`DescendingSizes`），崩 8 次，每次挂载试 3 次，末次挂载再试 16 次；写的大小放开（放不下的 8192 单元、小覆盖写、只崩不写的对照）。

**跑前的翻面条件**：放不下的写那一臂，状态数不足 4（`after=[… states=Some((n,_,_))` 的 n 小于 4）之后的 `name=crash_loop_accept` 行 `empties=0`，或 `crash_loop_summary` 的 `refused_with_empties` 不超过 3，这一格撤回。状态数在整段历史里都不掉到 4 以下，这条路走不到，同样撤回。小覆盖写那一臂收得下（`went_through=true`）是回收够了，不算翻面：那一段状态数掉下去换到了写，要照实写成代价。

打中之后四句：

| 问 | 答 |
|---|---|
| 分不分辨臂 | **不分辨**「`accept` 自己抬 F」与不抬：`publish_user_change` 那条路（排空待发、屏障）用同一个推法，同一段历史照样走；`accept` 只把触发频率从偶发抬到每次重试。病根在上限式子与无条件推，另立一笔账，不拿它翻第 115 批的定案 |
| 系统当时看得到吗 | 看得到：算上限那一遍就读了整环与每条根的树表（`crates/singlefs-core/src/mount.rs:4083-4157`） |
| 满足的是哪个分句 | 定案句「回退候选集保留最近 4 个可退到的不同状态」（`.claude/kb/decisions/16-发布语义.md:31`）被挤掉；可上限分句字面许可这一次抬。边界句「根环容量是这句的边界」说的是两个状态交替；「反方向只让 F 停住，直到第 4 个状态的根离开根环」（`.claude/kb/decisions/16-发布语义.md:48`）说根离开之后没说接着推。**条款没写这一格**，不是代码违反字面 |
| 改法在这几格上还中不中 | 见下表 |

| 改法（都只在读码上推过、被攻过零轮） | 放不下的写重试 | 小写回收得到 | 崩溃循环里状态数 | 标记 |
|---|---|---|---|---|
| 甲 推之前先算：`available` 加 defer 里全部字节仍小于需求就不推（只读内存） | 修：一次都不推 | 不动：照推 | 对放不下的写保住，对写得下的照掉 | 推的 |
| 乙 把门搬到串上：这一串会盖掉某个状态在环里最后一条根、状态数又已不足 4，就停 | 修 | **变差**：回收不到，写被拒 | 保住 | 推的 |
| 丙 不足 4 个状态时一次准入只推一串 | 不修，只从 7 次降到 2 或 3 次 | 变慢 | 不保 | 推的 |
| 丁 照旧、写进 D16 射程 | 不修 | 不动 | 不保 | 条款动作，不用量 |

甲与乙可以叠。乙要用户定：状态数与这次写哪个优先。

### 2. 主 agent「只删 1 个文件写不进是 D16 的结果、不是缺陷」

读数（量的，任务 11320 第 5 次尝试的日志，四条失败用例同一组数；那一趟用例还写着「删 1 个写得进」，所以 panic 把拒绝全文打了出来）：`last_refusal` 的 `available: AvailableBytesOnOneDevice(-558348)`、`demand: BytesOnOneDevice(294912)`，`refusals_that_pushed_the_floor_raises` 里那一次是 `available: AvailableBytesOnOneDevice(-1017100)`，`stop: FloorAlreadyAtTheCeiling { floor: CheckpointTxg(47), ceiling: CheckpointTxg(47) }`。`OneUnit` 与 `ThirtyTwoUnits` 两种大小 `available` 逐字相同。

任务 11298（量的，`accept` 自己抬 F 之前的树）同一条历史：`base=DescendingSizes size=OneUnit files=1 … slots=3392,free=3392,deferred=147`，`checkpoint_cost=MetadataBlocks(130)`。

- **这一串是净赚的**：推之前 -1017100，推之后 -558348，多了 458752 字节（28 槽）。我起初怀疑空发布自己吃掉的比回收的多，这条历史上不成立，撤回；别的历史上还要看 `available_before`、`available_after`（翻面条件：任一行 `available_after < available_before` 且 `empties > 0`，就是这一串把可用压低了，单列一格）。
- **差的 52 槽（`294912 + 558348 = 853260` 字节）多半压在 defer 里**：147 槽 defer，放开就够（推的，147 是没抬 F 的树上量的；`gap_is_in_the_deferred` 读数回答）。这一半是 D16 回收门槛，主 agent 说得对；与删的文件多大无关也对得上。
- **可是「写不进」不是 D16 一条的结果。** 盘上 3392 个空槽，这次要 18 槽，写进去不挤掉任何一个状态。拒它的是 `FormulaAvailableAgainstDemand`，式子在「容量 − 已分配 − 不可回收 − 挂载期承诺量 − 被抛弃根独占量」（`crates/singlefs-core/src/admission.rs:797`）那一段：`3392 × 16384 + 558348 = 56132876` 字节共扣 3426 槽，其中切换预留 `4 × (2 + 3 × 130) = 1568` 槽、保留池 130 槽，余下 1728 槽是环窗口与已承诺预留（推的，按 `crates/singlefs-core/src/admission.rs:620`、`crates/singlefs-core/src/admission.rs:564` 的式子算，`c_max` 取 130）。

所以回答正文那一问：**有一个不违反 D16 已定项 1 的历史本可以写进去却拒了，就是这一条**；它违反不了的是 D28（挂载期承诺量）已定项 1。这不是缺陷，是两条已定项叠出来的。要改的是说法与用例文档：「放出来的槽还被回退窗口占着（D16（发布语义） 已定项 1 的保留最近 4 个状态）」（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:476`）只说了一半，写进判决时写「D16 回收门槛压着差额，D28 承诺量定下差额」。

**跑前的翻面条件**：`name=net_accept … size=OneUnit count=1` 的 `gap_is_in_the_deferred=false`，那就连「压在 defer 里」这一半也不成立，主 agent 的判断要撤回。

打中之后四句：不分辨臂（`accept` 抬不抬都是这个数）；系统看得到（三项都在内存）；满足的是「删掉的空间要再发生 3 次改变用户可见状态的发布才回可分配集合」（`.claude/kb/decisions/16-发布语义.md:59`）加 D28 的式子；没有跑前条款给的改法，只改说法。

### 3. 回退门的账、预算、备忘、破例：没打中

| 形状 | 读码 | 翻面条件（读 `name=steady_pass`、`name=crash_loop_*`） |
|---|---|---|
| 空发布不进 `publishes_since_the_last_user_change`，绕过门 | 门不读这个账，每一批之前现读根环（`crates/singlefs-core/src/mounted_session.rs:4274`），空发布占的槽读得到。预算多放的几次撞门 | 一拍 `compaction_publishes > 0`、`empty_segments` 不为 0、这一拍之后状态数掉到 4 以下 |
| 挂载那一份预算 | 只读一次（`crates/singlefs-core/src/compaction_plan.rs:1031`）。头一次判起跑之前推的串算进去，之后推的不算，口径不齐，只影响多放几次，还是撞门 | 同上 |
| 备忘 | 按 txg 认（`crates/singlefs-core/src/compaction_plan.rs:1026`），一串一推就失效，下一拍重判一次，再停。推到上限之后不再推，不循环；只有第 1 条那一格里每次都失效，多读几遍根环 | 一拍 `calls` 顶到 64 而 `compaction_publishes=0` |
| 破例判错 | 判的是记录口径的全空段数，抬 F 回收不改它（`crates/singlefs-core/src/mounted_session.rs:2044`）；串不会把破例打开或关上 | `empty_segments` 在一次 `accept` 前后变了 |
| 一次调用超过 B | 判过之后触发的那次发布从 0 重数（`publish_the_pending_changes` 进 `publish_user_change`），字面可到 7 + 7；可判与发共用一份判据，推成了再判过，发布那一次不再被空间拒 | `written_bytes` 对应超过 8 次发布的一行 |

### 4. D16 已定项 7、17 与第二轮三处修补：读码没打中，扫描待量

`accept` 自己写盘多出来的路只有「串里写盘报错」。逐条走：

1. 串里报错：冻结的种类是发布路径默认的 `NeitherOfTheCountedKinds`（`crates/singlefs-core/src/transaction.rs:1635`），`accept` 不改它。待发一个字都没动，`accept` 交回 `RefusedAfterRaisingTheFloorForSpace`。
2. fsync：运行库持锁先重发（`crates/singlefs-mount-runtime/src/lib.rs:923`），重发成了排空待发；重发报错交回错误。核心层入口也拒（`crates/singlefs-core/src/mounted_session.rs:6934`）。走不到答成功而待发丢掉。
3. 排空时报错：`publish_the_pending_changes` 只在种类是用户改动时不放回（`crates/singlefs-core/src/mounted_session.rs:6531`）。批自己那一次冻结记用户改动；为这一批推的串冻结记默认种类，待发放回。进门前就冻结着而种类是用户改动、待发又不空：冻结着时 `accept` 进门就拒（`crates/singlefs-core/src/mounted_session.rs:6568`），待发长不出来，那句「这里走不到」我查的这几条路上成立。
4. 修补「队列不空算起跑」：`accept` 判队列在推之前（`crates/singlefs-core/src/mounted_session.rs:6780`），队列不空不推，与起跑判据不打架。
5. 修补「重发成了照种类记账」：串的种类不记账，推的串不重给预算。对。

探针臂 `arm_fault_in_the_series`：起点是 F 低于上限、待发非空（摆法在 `pool_ready_for_a_fault_in_the_series`），再来一次放不下的写推串；写报错的位置从 0 逐个扫到串走完，屏障同样；之后三种走法：fsync 重发成、fsync 重发又报错、一拍之后接着写。

**跑前的翻面条件**：任一行 `name=fault_durability … violation=true`（fsync 答成功，崩了再挂载读不出收下的内容），这一格打中，最重。`name=fault_arm_invalid` 或对照臂 `empties_reported=0` 说明起点没摆对，这一组读数作废，不算没打中。

### 5. 串里写盘报错之后的扣住位（轻）

串断在第一次发布之后、重发成了：盘上那条根已带新 F，「if ceiling <= current.root.rollback_floor {」（`crates/singlefs-core/src/mount.rs:4842`）从此成立，不再推，「allocator.release_reclaim_holds();」（`crates/singlefs-core/src/mount.rs:5657`）走不到，代码自己写着「第二次起被拒的，前面已落盘的根带着新 F 与回收之后的账，扣住位照旧留在这个进程里，下一次抬 F 做成才放开」（`crates/singlefs-core/src/mount.rs:5613`）。第一次发布落盘途中失败那一支分配器也不退回（`crates/singlefs-core/src/mount.rs:5615`）。扣住的槽「空闲计数照加，位图照清」（`crates/singlefs-core/src/allocator.rs:789`），准入式子把它们当空闲（`crates/singlefs-core/src/admission.rs:750`），落点取不到，两边差着这些槽。

这一格在账上（C516、C546），`accept` 抬 F 之后写满时重试就走得到。写满的池上环窗口远大于扣住的量，推的结局是写照样进，只是真空闲比式子以为的少几十槽，到下一串或卸载放开。**翻面条件**：`name=fault_later_summary` 的 `went_through` 少于对照臂，或 `unmounted=false`，升成打中；`held_slots` 在下一串之后还不是 0，也升。

四句：不分辨臂（`publish_user_change` 那条路同样）；系统看得到（分配器知道扣着）；条款明写「那一格里扣住的槽怎么放没有条款，见 C546」（`.claude/kb/decisions/16-发布语义.md:42`）；改法（断串之后下一次准入先把串补完，只在分配器与根环读数上判，零轮、推的）。

## 二、T2 未定项清单

| 项 | 答 | 进哪 | 这一批修吗 |
|---|---|---|---|
| `accept` 持锁时长与「一批之内 GPU 持锁」 | 持锁一次 `accept` 至多 7 次空发布，加各串的系统配置写与屏障，发布之间不放锁（`crates/singlefs-mount-runtime/src/lib.rs:879` 的文档照实写了）。fsync 让路只管整理批，抢不了这一串。最坏：先等一批 GPU（默认等 10 秒，`.claude/kb/mount-parameters.md:25`，第 4 次超时才算用不了，推的 40 秒一段），再等这 7 次。真盘挂钟没量，内存盘上 `writes`、`barriers` 读数给写次数 | 里程碑四优化项（与判决登的 unlink 持锁同一行）加 D24（后台重活能不能卸给 GPU） 已定项 2 射程 | 不修 |
| `accept` 抬 F 在崩溃循环里的边界 | 第一节第 1 条。余量算术（`crates/singlefs-core/src/compaction_plan.rs:1546`）没算 `accept` 的串 | **交用户表**（D16 已定项 1 射程，零轮形态，甲、乙二选）；余量那句文档这一批可改 | 不自动修，交用户 |
| 前台 unlink 整段持锁 | 第 115 批没动：`grep -n '^[+-].*publish_the_queue_until_it_is_empty_raising_the_floor_for_space' research/prompts/_m4-gpugc-mount-runtime-code-r3-diff.md` 零行。这条路每落一步预算归零（`crates/singlefs-core/src/mounted_session.rs:5893`），满盘上一步至多再推 7 次，这是旧行为 | 已登的优化项，补一句「满盘上每步可再推 7 次」 | 不修 |
| `mark_allocated` 摆满那一格外推到大盘 | **站得住，理由换一个**（推的）。「填到被拒还有 3400 多个空槽」不是 320 MiB 的巧合，是准入式子扣出来的：第一节第 2 条那 3426 槽。其中环窗口按 `(3S + 1) × W` 现算（`crates/singlefs-core/src/admission.rs:564`），W 按树层数现算，门槛 128 到 256 量的是同一个池上走一圈根环的真占用，两边都随层数与 S 走。真盘余量下界因此是式子，不是盘大小。两处要写明：不经准入的发布（整理批、抬 F 的串）不算在 W 里；走不动时后台不推抬 F（`crates/singlefs-core/src/mounted_session.rs:2128`），`accept` 又在队列不空时拒，只写不删的调用方出不来 | D26 射程，连翻面条件：`name=crash_loop_accept` 的 `unblocked` 掉到 256 以下，或任一拍 `Failed(DeletionContinuation` | 不修 |
| 重发不退避 | 判决「不改」的理由还在（原位同字节）。第 113 批多一条路：每次 fsync 都持锁重发一遍（`crates/singlefs-mount-runtime/src/lib.rs:925`），坏盘上 fsync 的挂钟是重发的挂钟，别的入口排在后面 | D26 射程，加这一句 | 不修 |

**清单上没有、这一轮查出来的：**

| 项 | 依据 | 进哪 |
|---|---|---|
| 后台任务停着时队列不走，`accept` 一直拒 | 停着先返回（`crates/singlefs-core/src/mounted_session.rs:3325`），排在队列起跑（`crates/singlefs-core/src/mounted_session.rs:3391`）之前；`accept` 队列不空就拒（`crates/singlefs-core/src/mounted_session.rs:6780`）。历史：开 `gpu`、后台任务报警停着、占用不超过阈值，unlink 写盘报错、重发成了。第 4 条写了「删除续做也不走」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`），第 6 条写了「不因为它没删完挡住别的操作」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`），两句在这一格上相左。屏障类改动走得通（`publish_user_change` 自己走空队列）。推的，探针没开卡没摆这一臂；挂在 `crates/singlefs-verification/tests/compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs` 那一族上能量 | D26 射程，要用户定哪句优先；欠账 |
| 放不下的写也推 F | 甲那一判没有：一次远超容量的写先推一串、丢掉第 4 个状态之下的候选，才报 ENOSPC。状态数够时只推一次，有界；条款字面许可 | D16 射程或优化项，与甲同一个改法 |
| 观测与事件少数 | 事件文档说「调用方看不到发布点的接收方靠它知道根环走了几格」（`crates/singlefs-mount-runtime/src/lib.rs:179`），屏障的串不报（`crates/singlefs-core/src/mounted_session.rs:6890`），触发那一次发布自己推的串、断串已落盘的几次也不在里面 | 这一批可改文档；见类型与名字 |
| 拒绝的调试文字有 GiB | 任务 11320 日志 5.9 GiB，单行 1553573352 个字符（量的，我用脚本数的），是 `AcceptRefused` 带着推过的 `TransactionOutput`。用例已截断（`leading_text_of`），调用方写 `{:?}` 进日志照样炸；首个真调用方是 FUSE 守护进程 | 欠账：手写一份不带版本全文的调试输出，或拒绝只带 txg |

## 三、T3 报出的数

| 正文里的数 | 现查 | 要改成 |
|---|---|---|
| 「`accept` 推抬 F 之后 … `available: AvailableBytesOnOneDevice(-1017100)`」（`research/prompts/_m4-gpugc-mount-runtime-code-r3-body.md:22` 那一行） | -1017100 在 `refusals_that_pushed_the_floor_raises` 里，是**推之前**那一次；`last_refusal` 是 -558348（量的，第一节第 2 条） | 推之后 -558348，差 52 槽 |
| 「真写满那一族用例内存峰值 15.11 GiB（任务 11320 第 5 次尝试）」 | 那一次结局 `command-failed`、退 101，4 条用例 panic、把 1.5 GiB 的拒绝全文各打了一遍（量的：`task-server/task status 11320`，日志 5.9 GiB）。那一批用例后来改了、截断了，这个峰值不代表现在。任务 11441 跑四个包 23 个目标的峰值 12.87 GiB、第 4 次上限 13824M 撞过 | 不当依据。要数就单跑这一个目标量一次 |
| 「任务 11441 … ✓ 测试全绿 … ✓ fmt 过」 | 两行都在，但任务结局 `command-failed`、退 1：同一趟 `✗ clippy 红：2 个特性组合里 2 个红`、`✗ cargo build --all-targets 红`（`unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs` 还引 `ForegroundFsync`）。之后修了：任务 11523 退 0，任务 11519 退 0，冻结副本里 `compaction_plan::ForegroundFsync` 零命中 | 写明 11441 退 1、红的在哪两步、谁补的 |
| 「任务 11339 抓到 19 条」 | 点名 20 条、抓到 19 条、退 1，没红的是「根排在最后一道屏障之后」；任务 11400 补的就是这一条，抓到（量的） | 合计 38 条抓到不错，但 11339 不是全抓 |
| 「没有结局的」三个任务 | 写这份报告时都有了：11533 退 0；11519 退 0；**11530 退 1**，`mutation-tables` 红，三组重复行都在别的文件（`e171_three_oracle_collusion.rs`、`history.rs`、`model.rs`），不在这一批 | 判决里写结局 |
| `refused_walks=0`、门槛 128 到 256 | 真盘余量为什么大，第二节那一行给了来由；摆法与真盘同不同口径归正推腿主攻，这里不重复 | — |
| 「删够 4 个」恰好吗 | 探针 `arm_net_of_the_floor_raises` 放开扫：三种填法 × 一单元、32 单元各删 1 到 7 个、最大文件 1 个 × 小写，1、4、7 另加 32 单元的写。推不出 2、3 行不行：差额 52 槽，每删一个既放槽又抬上限，不是数状态能算的。**翻面条件**：`name=net_summary` 里同一填法同一大小 `went_through_at` 第一个不是 `None` 的 `count` 不是 4，或 count 变大之后又变回 `None`，用例名里的「四个」要改 | 待主 agent 跑 |
| `margin_crash_loop` 带不带最坏活意图 | 不带，判决已写。这一轮多一条：它也不带 `accept` 的串，第一节第 1 条的算术比它紧 | 写进 D26 射程 |

## 四、检查单

| 格 | 判定 | 现查了什么 |
|---|---|---|
| 走不到的理由 | **成立** | `grep -n '^+.*走不到\|^+.*expect(' research/prompts/_m4-gpugc-mount-runtime-code-r3-diff.md` 6 行：4 处 `expect` 是 `u64::try_from` 与计数；2 处注释。`crates/singlefs-core/src/mounted_session.rs:6530` 那句逐路走过（第一节第 4 条第 3 步）；`crates/singlefs-core/src/transaction.rs` 那句说树表与映射树根每次发布都换，读码相符。只查了这份 diff |
| 类型与名字 | **不成立（低危）** | 第二轮两处修到：枚举是 `AnswerOfTheCallerBeforeTheNextBatch`，停因拆出 `YieldedToTheCallerStoppingItsThread`。不成立的一句：`FloorRaisesPushedByAnAccept` 的文档（第二节「观测与事件少数」）。`RefusedAfterRaisingTheFloorForSpace` 在一串都没推时也用，文档写明了，不算 |
| 不兼容旧代码与旧格式 | **成立** | diff 里 `legacy`、`兼容`、`旧版`、`old_format` 零命中；`crates/` 下 `compaction_plan::ForegroundFsync`、`foreground_fsync`、`ForegroundFsync::` 只剩 `compaction_plan.rs:723` 一个字符串名 |
| 不新建判定库 | **成立** | diff 与三份新测试文件里没有 `sqlite`、`rocksdb`；崩溃流里的 `verdict` 是不变量判定的变量。只查了这几份 |
| 所有任务多机可用 | **成立** | 三份文件 `current_dir`、`std::fs::read`、`env!`、`include_str` 零命中，都在内存盘上。内存：不收，15.11 GiB 不当依据（第三节） |
| 测试优先多线程 | **成立** | `mount_runtime_accept_raises_the_floor_for_space.rs` 没限线程，填盘经 `OnceLock` 串一次、别的臂等它；崩溃流调 `enumerate_crash_states_with_every_kind_of_observer`（`crates/singlefs-verification/tests/crash_enumeration_frozen_publish_resent_stream.rs:428`） |
| 带 --item 的命令与项 | **成立**；形式化项**推不出** | `.claude/gate.d/stage-inputs.tsv` 第 161、162 行与 `.claude/gate.d/crash-point-replay-items.tsv` 第 78、79 行都在，点名的测试函数在第 592、603 行；任务 11533 退 0。形式化项要跑 `--list-items`，腿不跑 |
| 新崩溃流多罩了什么 | **成立**；不加流**够**（推的） | 录的是 `operations_before_the_failure` 之后的全部（第 274、311 行），失败那一遍与重发同段。甲二不丢状态：两个报错位置单元都已过屏障，重发逐字节同；只落一部分读出来与全落相同。`accept` 抬 F 之后待发只在内存，盘上就是串的前缀，用例钉了前缀（一条历史）。没罩的组合：串里冻结再重发。它的字节是串的字节重写一遍，同冻结流那一类，推的不算新状态；`name=fault_durability` 只取发布边界，不证这一句 |

## 要主 agent 跑的命令

1. 探针（编译加跑，整条可安全重排）。在仓根起：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-mount-runtime-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3-opus/run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-mount-runtime-code-r3-opus-model
```

跑完看：`TASK_SERVER_RESULT` 的 `type=`、`exit=`；`exit=0` 且日志末尾 `name=rerun_exit attack_r3_accept_floor=0`、`name=probe_finished` 算跑完（有 `name=stopped_by_the_time_budget` 就是预算到了，写明停在哪一臂）。全部读数在 `<run>/readings.out`。探针从没编过，编不过就看 `<run>/attack_r3_accept_floor.out` 的 `error[`，新派同立场的腿（带「承接：」）改探针。

读哪几行、翻面看什么：

| 问 | 行 | 判法 |
|---|---|---|
| 连续被拒推不停 | `name=crash_loop_summary` 三条；`name=crash_loop_accept` 与 `name=crash_loop_mount` | 第一节第 1 条的翻面条件。同一 `crash=` 上比 `write=Nothing` 与 `write=NeverFits` 的 `states`，差出来的才是 `accept` 的账 |
| 负数是谁占的 | `name=net_accept … size=OneUnit count=1` | `gap_is_in_the_deferred`、`available_before`、`available_after` |
| 删够几个 | `name=net_summary` | 第三节那一行 |
| fsync 答错 | `name=fault_durability` | `violation=true` 一行就打中。`name=fault_arm_invalid`、`name=fault_base_went_through went_through=false` 作废 |
| 扣住位 | `name=fault_after_the_pass`、`name=fault_later`、`name=fault_later_summary` | 第一节第 5 条 |
| 稳态与 S | `name=steady_summary`、`name=steady_after_a_crash_accept` | `least_states`、`cycles_below_four_states`：S 从 8 到 5、两种水位。推的是挂着不崩不掉到 4 以下，崩一次之后 S 小的先掉；对不上就是第一节的算术错了 |

判决要引的数由主 agent 复跑，上面都不算量过。

2. 报告引文（腿已跑，主 agent 判决时照规则另跑）：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-mount-runtime-code-r3-opus-output.md --root ~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3/repo --root .`。

## 没打中的形状

都是读码，没量；一条腿的「没打中」只抽了一次，不撑结论。

| 形状 | 取样范围 |
|---|---|
| 空发布不进账绕过回退门 | 读 `stop_of_the_rollback_gate` 与它的调用处；稳态臂 S 5 到 8 × 两种水位待量 |
| 备忘挡住该判的或该停的不停 | 读 `stop_still_holding_at`、`note_the_stop_of_a_run` 两处调用 |
| 破例被串开或关 | 读 `foreground_cannot_write_under_the_switches` 与 `refusal_is_short_of_space` 那段注释 |
| `accept` 在冻结、队列不空时推串 | 读进门次序；已有两条用例 |
| 进门之前冻结着、种类是用户改动、待发不空 | 逐个看 `publish_the_pending_changes` 的调用方与 `note_the_kind_of_a_publish_frozen_by_this_call` 的三处调用 |
| 一次改动推过 B | 判与发共用一份判据，读 `judge_the_publish_before_any_read_or_write`；两边分叉的只有读盘核对不上，没造 |
| 串自己把第 4 个状态挤掉（状态数够时） | 算术，串长 2 或 3，余量 S |
| 这一串净亏（空发布吃的比回收的多） | 只有任务 11320 一条历史（+28 槽），探针扫别的 |
| 读坏槽那一支（不抬 F 的空发布）挤掉状态 | 读码：读坏槽能住的 txg 不高于 F 才走这一支，盖到它之前盖的都是 F 之下的根。没造故障 |

## 这条腿自己的限度

- 探针一行都没编过、没跑过；公开接口按冻结副本与能编过的旧探针逐个核过，`rustfmt` 退 0，编不编得过不知道。第一节的数都是读码算的，串长、状态数的算术只手算过。
- 崩溃只取发布边界（丢运行库、盘原样留着）。这一轮的模型没有调枚举函数，也没有枚举崩溃状态：想问的是崩了之后状态数怎么走，不是某一写上崩了读出什么。「串里冻结再重发」不算新状态是推的，要证得加一条崩溃流，按崩溃点重放的枚举域跑。
- 只在 320 MiB、两块盘、日志环 64 MiB 上取样；稳态臂 S 只扫 5 到 8，`S = 4` 没扫（建池入口给得出、mkfs 默认给不出）。
- 第 1 条的路径要写满加崩溃（或破例）。整理真碰门的历史只有越线水位那一臂，产品水位上 `DescendingSizes` 全空段 47 个，整理不到期（量的，任务 11298）。
- 拆账里的 1568、130、1728 是按式子推的，`c_max` 取任务 11298 那棵树上的 130，没在 `accept` 抬 F 之后的树上读。
- T3 的摆法与真盘口径归正推腿，这里只给了余量的来由。
- 第 1 条的甲、乙、丙，第 5 条的补串，都没实现、没跑，只在读码上推过，被攻过零轮。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳。
- 没往统一任务服务交任务，判决要引的数没复跑（命令在上面）。
- 读了主 agent 给了号的任务日志：11290、11298、11320、11339、11400、11405、11441、11530、11533、11535，只取读数；11320 的日志 5.9 GiB 落在草稿目录里数过字符，已删。别的任务号只看 `status`。
- 没读禁读清单里的文件。

删的东西：草稿目录里没有编译目录与仓副本；`du` 过的只有任务日志拷贝（11320 那份 5.9 GiB，另两份 1.2 MiB），都删了。
