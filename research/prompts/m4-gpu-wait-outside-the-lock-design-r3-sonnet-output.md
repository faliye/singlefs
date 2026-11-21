# m4-gpu-wait-outside-the-lock-design-r3 正推腿（Sonnet）报告（2026-11-18）

立场：照正文第六节「分工」正推那一行，把第四节形态核成能交实现员的样子；Q1、Q4、Q6 主攻，Q2、Q3、Q5 各回成立、不成立或推不出。方案按 `crates/` 今天的实现来谈。
没读禁读清单里的三处。这一腿一条项目代码都没跑（不起 cargo、不交任务）：文中「量的」只有 grep、读文件与读前两轮留在工作根里的产物；凡是读码推出来、没量过的都写「推的，没量过」。
引文：每一句引文都在被引文件里 `grep -nF` 过一次，行号取命中的那一行；kb 条款写 kb 文件名加那份文件自己的行号。设计轮没有冻结的树，读的是主工作区此刻的样子：本轮点名的八份 `crates/` 文件的 sha256 与 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-snapshot/crates-sha256.txt` 里的前 20 位逐个相同（量的，`sha256sum` 现取）；`crates/singlefs-core/src/allocator.rs` 不在那张清单里、`crates/singlefs-core/src/mount.rs` 有未提交改动（`git diff --stat` 量得 52 行），这两份我引的行都是此刻读的。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-model/`。它只补一件事：第二轮辩方腿的忙窗口基线探针（源文件原样拷来，与第二轮那份 `cmp` 逐字节相同）在今天的代码上再量三样第二轮没量的，用来回 Q3：替身卡每次调用睡 10、20 毫秒两档、每个配置连跑三趟看趟与趟之间差多少、CPU 路径连跑三趟。步骤零（主 agent 直接跑，不经服务；建副本带运行库用例的共用搭建）：

```text
python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-sonnet/copy --with-test-inputs
```

之后整条交任务服务（包装先删运行目录，运行目录名以 `-run` 收尾）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-sonnet/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-sonnet-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-model
```

| 文件 | sha256 |
|---|---|
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷来，只改 `rerun_steps`） | `21c577e73d565decf83325c53fca870e4ee33b1c079ad31ca129e3d475096240` |
| `tools/rerun-from-a-clean-work-directory.sh`（第二轮那份原样拷来） | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `probes/mount_runtime_busy_window_foreground_wait.rs`（第二轮辩方腿那份原样拷来） | `d7d2fdc481721ce5cc42e33fe59396ae3c736607ca4bf79eb46cf268f72b13b9` |

状态：`tools/rerun.sh` 过 `bash -n`；探针源文件第二轮跑过（第二轮判决 任务 19745 退 0），这一趟没改它；`rerun_steps` 里 30 条 cargo 都写字面参数，没编过、没跑过。脚本退 0 的条件：30 行 `name=busy_window_foreground_wait`、判法自证过（睡 50 毫秒三趟被挡最坏等待的中位数大于睡 0 毫秒的）、末行 `name=rerun_done busy_rows=30 expected_rows=30 failures=0`。内存与时间：推的，没量过，一个 cargo 测试进程、每次一个运行库带四条读线程，30 趟每趟 6 个 3 秒窗口加建池，合计十几分钟、内存几百 MB 以内。

## 一、Q1：第 5 条的前提撑不撑得住

**前提原文**（正文第四节第 5 条）：两版里起点在 R 里的未释放记录逐条相同（盘、槽、跨度、分配代）则这些单元的字节、分类与映射位置都没变。

**先把「分类」拆开**。今天一批里一个记录的分类由五样输入定，前提只用其中一样（记录的四元组）去代表其余四样：

| 输入 | 今天在哪读 | 四元组管得着吗 |
|---|---|---|
| R 里有哪些未释放记录 | 「record.device == device && !record.is_released」（`crates/singlefs-core/src/region_scan.rs:460`） | 就是它自己 |
| 钉住的槽 | 「pinned_slots.insert(SlotNumber(slot));」（`crates/singlefs-core/src/mounted_session.rs:4974`），从这一版 `version.units` 里豁免映射的几类角色算 | 管得着（推的）：能搬的单元在映射里有条目，豁免映射的角色没有，一个槽不会从能搬变成钉住 |
| 隔离集 | 「if quarantined_slots.range(covered_slots).next().is_some() {」（`crates/singlefs-core/src/region_scan.rs:717`），来自会话这一侧 `compaction.quarantined_slots()` | **管不着**，下面第二节 |
| 单元头字节 | 盘上读 | 管得着：一个槽的字节只在「未释放 → 已释放 → 已回收 → 被重新分配」之后才会变，重新分配要改写记录 |
| 映射条目（含留存） | 「mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(」（`crates/singlefs-core/src/mounted_session.rs:4925`） | **管不着**，同在第二节 |

**字节那一样的依据**（读码，推的，没量过）：新分配只发位图上空闲的槽，与新落点相交的记录必须是已回收的，起点就是这个槽的那一条被改写成这次分配、分配代换成新的——「assert!(existing.is_released, "回收过的落点的记录该是已释放");」（`crates/singlefs-core/src/allocator.rs:1580`）、「existing.generation = generation;」（`crates/singlefs-core/src/allocator.rs:1582`）；释放那一刻记录的分配代被换成释放代——「record.generation = release_generation;」（`crates/singlefs-core/src/allocator.rs:2828`）。所以「未释放且分配代没变」的记录，罩住的槽自分配以来没被重写过。唯一把记录改回原样的是回退复活，见下表第 6 行。


### 一·1　破前提的一处：释放前读盘核出对不上，记录留在已分配、分配代不变

读码顺序（都在今天的代码上，推的，没量过；记录那一半有现成的用例断言）：

1. 一个单元被换下（覆盖写、截断尾部、unlink 的批、删快照放 deadlist 条目、整理搬走源落点）要进释放清单，落盘之前先读盘核它每一份的校验和：`pub(crate) fn copies_of_a_released_unit_failing_the_release_checksum_check<`（`crates/singlefs-core/src/transaction.rs:5490`）。任一份读不出或对不上，两盘各一份的记录都留：「发布照常释放（映射条目去掉）」（`crates/singlefs-core/src/transaction.rs:5442`）。
2. 落盘那一步对每个释放的落点调 `release_leaving_the_record_allocated_on`：「allocator.release_leaving_the_record_allocated_on(」（`crates/singlefs-core/src/transaction.rs:15739`）；同一段里把隔离的落点记进会话这一侧的隔离集：「.note_quarantined_placement(copy.placement.slot, copy.placement.span);」（`crates/singlefs-core/src/transaction.rs:15754`）。
3. 分配器里，被点名的那块盘上的记录走 `continue`，没走改成已释放的那几行：「if devices_whose_record_stays_allocated.contains(&device.device) {」（`crates/singlefs-core/src/allocator.rs:2821`）、「record.quarantine_mark =」（`crates/singlefs-core/src/allocator.rs:2822`）、「continue;」（`crates/singlefs-core/src/allocator.rs:2824`）；把分配代换成释放代的那一行在 `continue` 之后：「record.generation = release_generation;」（`crates/singlefs-core/src/allocator.rs:2828`）。结果是：这条记录 `is_released` 仍是 false、`generation` 不变，只有 `quarantine_mark` 变了（`pub quarantine_mark: AllocationRecordQuarantineMark,` 在 `crates/singlefs-core/src/allocator.rs:81`，记录的 `derive` 含 `PartialEq`：`#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]` 在 `crates/singlefs-core/src/allocator.rs:71`）。
4. 现成用例断言的就是这一格：覆盖写换下一个一份副本坏了的数据单元之后，「两盘的分配记录都留在已分配、分配代照旧：账对称」（`crates/singlefs-verification/tests/release_checksum_quarantine.rs:270`）。
5. 发布路径只有一个落盘入口：`publish_admitted(` 全文件只有一处调用（`crates/singlefs-core/src/transaction.rs:13748`，在 `publish_version` 里，`pub fn publish_version<Device: BlockDevice>(` 在 `crates/singlefs-core/src/transaction.rs:13049`），所以上面两种改动（用户改动与整理自己的发布）都走它（量的：`grep -c 'publish_admitted(' crates/singlefs-core/src/transaction.rs` 得 1，别处出现的 `publish_admitted` 都是注释里的名字，定义是 `fn publish_admitted<Device: BlockDevice>(`，在 `crates/singlefs-core/src/transaction.rs:15681`）。

**对第 5 条前提的后果**：用户改动把 R 里一个单元换下时，只要它那一份读盘核没过，两版里这条记录的（盘、槽、跨度、分配代）逐条相同，而这个单元已经被释放：映射条目不再写它、会话隔离集里多了它的槽。锁外判的分类是能搬，现行那一版上的分类是隔离。「记录逐条相同则分类没变」**不成立**，破在隔离记号（以及它在会话这一侧的镜像）上。

**这一破不会写坏盘，但会把「剔掉」变成「整批被拒」**（推的，没量过）：计划里这个单元的 key 在现行那一版映射里已不在、或位置已不是源落点，发布路径逐条核计划时拒整批：「return Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { mapping_key });」（`crates/singlefs-core/src/transaction.rs:11411`）、「RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition {」（`crates/singlefs-core/src/transaction.rs:11457`）。会话那一边这两种都不是空间被拒，一路交上去：「Err(CompactionStepRefused::Refused(refused)) => {」（`crates/singlefs-core/src/mounted_session.rs:5704`），一拍里的运行库把 `Failed` 结局当「不再接着叫」：`CompactionRunOutcome::Failed(_) => false,`（`crates/singlefs-mount-runtime/src/lib.rs:678`）。下一拍从现行那一版重扫，在 R 里查到隔离槽就走今天的丢意图那一支：「if region_holds_a_quarantined_slot(region, &quarantined_slots) {」（`crates/singlefs-core/src/mounted_session.rs:5631`）。也就是每出一次隔离，多白算一批、多一个 `Failed` 结局；没看到写坏盘的路径。

**复核还要多比的三样**（回「破前提」那一格要写的）：

1. 比整条 `AllocationRecord`（含 `quarantine_mark`），不比四元组；并且比 R 里未释放记录的**集合**两边相等，不只比「票里每一条在现行那一版里还相同」：回退复活能让现行那一版的 R 里多出票里没有的未释放记录（下表第 6 行）。
2. 在现行那一版上重取隔离集（今天一批开头做的事：「compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());」在 `crates/singlefs-core/src/mounted_session.rs:5367`；锁外那一段期间前台的发布把新隔离的槽记在分配器那一侧，会话这一侧的集合要回锁后再取一次），先跑「if region_holds_a_quarantined_slot(region, &quarantined_slots) {」那一判，成立就丢意图，不走计划。
3. 映射位置、留存条目、字节不用另比：发布路径对现行那一版逐条核 key 在不在、源落点是不是映射位置、跨度（`crates/singlefs-core/src/transaction.rs:11411`、`crates/singlefs-core/src/transaction.rs:11457`），字节取现行那一版里的单元（「RelocatedUnitHolder::RoleOfThisVersion(role) => previous_read_by_key」在 `crates/singlefs-core/src/transaction.rs:11465`）或读盘核校验和（「.filter(|copy| crc32_castagnoli(copy) == location.unit_checksum);」在 `crates/singlefs-core/src/transaction.rs:11483`）。这是兜底，不是复核的替代：兜底拒的是整批。

### 一·2　逐条路径

判定列只写三种：不破前提、破前提、核不动。「记录」指 R 里那条未释放的分配记录。

| 路径 | 对 R 里的记录、映射条目、留存条目、钉住的槽、隔离集做什么（文件与行） | 判定 |
|---|---|---|
| `Truncate` | 入口 `pub fn publish_size_change<Device: BlockDevice>(`（`crates/singlefs-core/src/transaction.rs:11703`），截短时尾部数据单元与 extent 节点进释放清单、之后各批走 `pub fn walk_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(`（`crates/singlefs-core/src/transaction.rs:12872`）。换下的单元三种去向：释放（记录变成已释放、分配代换成释放代，`crates/singlefs-core/src/allocator.rs:2826`、`crates/singlefs-core/src/allocator.rs:2828`，计划里这个单元剔掉）；被活快照引用进 deadlist、映射条目挪进留存条目（「SnapshotLifetimeVerdict::KeptOnTheDeadlist => {」在 `crates/singlefs-core/src/transaction.rs:6785`，记录不变、位置不变，分类仍是能搬；今天的扫描就把留存条目并进映射：`crates/singlefs-core/src/mounted_session.rs:4925`，发布路径按现行那一版重新认持有者：`crates/singlefs-core/src/transaction.rs:11395` 起）；读盘核没过（一·1）。 | 释放、进 deadlist 两支不破；隔离那一支破前提 |
| `Unlink` | 意图那次发布只改 inode 记录与墓碑、「数据单元与 extent 树照抄」（`crates/singlefs-core/src/transaction.rs:11514`），`pub fn publish_unlink_intent<Device: BlockDevice>(`（`crates/singlefs-core/src/transaction.rs:11526`）；之后各批释放尾部，同 `Truncate` 的三种去向。 | 同上 |
| `DataChangesPublishedTogether` | 变体在 `crates/singlefs-core/src/mounted_session.rs:207`，落盘 `pub fn publish_data_changes_published_together<Device: BlockDevice>(`（`crates/singlefs-core/src/transaction.rs:10428`）。写新数据单元落新槽（R 排他时不会落进 R，见一·3），换下旧单元同上三种去向。**另**：每次用户改动发成之后会另发一批搬迁把「只坏一份」的单元换落点：「published.relocation_after_the_change_of_the_units_read_with_one_copy_failing =」（`crates/singlefs-core/src/mounted_session.rs:3043`，函数 `crates/singlefs-core/src/mounted_session.rs:2289`，计划取自 `pub fn relocations_of_old_data_units_kept_for_the_snapshots(` 在 `crates/singlefs-core/src/transaction.rs:11998`）。这一批改映射位置，源落点照释放之前读盘核：只坏一份就两份一起隔离，记录不变、映射位置已变。 | **破前提**：这是今天唯一在前台改映射位置、而 R 里记录可以不变的路径 |
| 建快照 | `pub fn publish_create_snapshot<Device: BlockDevice>(`（`crates/singlefs-core/src/transaction.rs:10509`）：不释放用户单元；改变的是活头的 `previous_snapshot_txg`，使之后被换下的单元走进 deadlist 而不是释放（`crates/singlefs-core/src/transaction.rs:6785`）。R 里的记录不变，单元留在留存条目里，位置不变。 | 不破前提 |
| 删快照 | 意图 `pub fn publish_snapshot_destruction_intent<Device: BlockDevice>(`（`crates/singlefs-core/src/transaction.rs:10587`）、批 `pub fn publish_snapshot_destruction_step<Device: BlockDevice>(`（`crates/singlefs-core/src/transaction.rs:10635`）：放掉 deadlist 条目、释放它们留存的映射条目指的落点，记录变成已释放，留存条目被删；读盘核没过时同一支隔离。 | 释放支不破；隔离支破前提 |
| 回退 | `pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(`（`crates/singlefs-core/src/mount.rs:10900`），会话内那一版 `pub fn roll_back_by_a_forward_publish_in_the_session<Device: BlockDevice>(`（`crates/singlefs-core/src/mount.rs:11236`）。把已释放的记录改回未释放、分配代写回目标版里的值：「record.is_released = false;」（`crates/singlefs-core/src/allocator.rs:2871`）。**记录可以回到票里那一条的原样**（释放、再复活，四元组相同，映射与位置项由回退那次发布按目标版重写，所以这一格不破）；**也可以在 R 里多出票里没有的未释放记录**（只比「票里每条」会漏，一·1 的第 1 样）。现行那一版有快照就拒回退：「if snapshots_in_the_current_version > 0 {」（`crates/singlefs-core/src/mount.rs:10917`）。运行库没有回退入口（`grep -rn 'roll_back' crates/singlefs-mount-runtime/src crates/singlefs-mount-runtime/tests` 零命中，量的），产品路径上只有会话内回滚函数与验证包的历史装置在调它（`crates/singlefs-verification/src/history.rs:8390`）。 | 记录相同那支不破；多出记录那支要集合比；回退落盘时丢掉复活槽的 discard 窗口核不动（缺：回收门槛与回退下限的对账，我只读到 `ReuseWindow::GatedByTheRollbackFloor`） |
| 删除续做 | 在整理批之前走：「if !deletion_queue_of_the_version_is_empty(current) {」（`crates/singlefs-core/src/mounted_session.rs:4618`），放锁的入口一次调用至多一步 `fn publish_at_most_one_step_of_the_queue<Device: BlockDevice>(`（`crates/singlefs-core/src/mounted_session.rs:2206`）。它们是 unlink、截断、删快照的各批，效果同那三行。 | 同上 |
| 抬 F 的空发布 | 一串空发布 `fn raise_the_floor_through<Device: BlockDevice>(`（`crates/singlefs-core/src/mount.rs:5327`），计划 `empty_publish_plan_after_file_version(&last_published, instance),`（`crates/singlefs-core/src/mounted_session.rs:4082`）：重写固定点单元，不碰用户树、实例表照抄（`crates/singlefs-core/src/mounted_session.rs:3859` 的说明）。换下的是豁免映射的根单元，它们是钉住类、本来不进计划；回收的是已释放记录的槽（`reclaim_released_up_to`），不碰未释放记录。 | 不破前提 |
| 整理自己在锁外那一段期间的发布 | 一个后台批在锁外时没有第二个整理批：显式入口 `pub fn run_compaction<Device: BlockDevice>(`（`crates/singlefs-core/src/mounted_session.rs:3088`）要 `&mut self`，运行库只能经 `with_the_session_and_devices` 里拿到会话再调，产品路径没人这样做（推的，没量过）；第二个后台调用者由第四节第 1 条的「会话上记有一批在锁外」拒。 | 不破前提，靠那一条拒 |


### 一·3　R 排他挡得住什么、挡不住什么

正文第五节 Q1 把 R 排他指到「已定项 2」。kb 里写排他的是已定项 1 第 3 条：`26-后台整理与放置回收.md:30` 「R 在意图期内不可作分配目的地」；已定项 2 只有目标态单调那一句：`26-后台整理与放置回收.md:58` 「前台在 R 内新分配的落点，其分配代必然 > c0」。代码里排他从签发那次发布起：「allocator.compaction_mut().issue(issued);」（`crates/singlefs-core/src/mounted_session.rs:5598`）。

- 挡得住：有活意图时前台与整理批的**新落点**不进 R。所以有活意图那一支里，R 里不会在锁外那一段期间冒出票里没有的新记录（回退复活除外，见一·2 第 6 行）。
- 挡不住：R 里已有记录被释放（记录变化）、被隔离（记录不变，一·1）、映射位置因另发的那批搬迁而变（一·1）、回退复活。
- 有一处明文放开：`26-后台整理与放置回收.md:30` 「取落点不排除 R」（删意图那一次，句子在同一行里）。这一次是整理自己的发布，不是前台。

### 一·4　GPU 判成「交 CPU 判」「这一轮不确定」的条目，随计划留到回锁之后还对不对

- 「交 CPU 判」（`JudgedByTheCpuReference`）与两份核确认的条目同样进分类结果：「| GpuEntryDecision::JudgedByTheCpuReference(class) => {」（`crates/singlefs-core/src/region_scan.rs:603`）。它们的有效性与别的判定一样，取决于一·1 补上的三样比对。
- 「这一轮不确定」的条目不进扫描的 `allocations`，只留一个落点（槽、跨度，没有分配代）：「pub records_uncertain_this_round: Vec<Placement>,」（`crates/singlefs-core/src/region_scan.rs:491`）。它在计划那一步只决定「照抄意图还是同一批删意图」（`fn compaction_publish_planned_with_the_records_uncertain_this_round(` 在 `crates/singlefs-core/src/mounted_session.rs:5737`）。锁外那一段期间这些记录被前台释放了的话，名单就是旧的：照名单走「照抄意图」那一支，发布路径在算好的账上核 R 已按记录全空就拒并换成同一批删意图（`fn publish_the_planned_compaction_step_falling_back_on_the_region_check` 在 `crates/singlefs-core/src/mounted_session.rs:5867`，拒绝成员 `RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried` 在 `crates/singlefs-core/src/transaction.rs:10723`）。不破（读码，推的）。
- 连续不确定的计数按落点认、住在 GPU 实现里（内存）：`26-后台整理与放置回收.md:259` 「一个条目是一个落点（盘、槽、跨度）」；实现被借出时 panic 的说明点名了它：「GPU 实现上一次借出时 panic 过：那一次已经往上报过，不带着可能半改的连续不确定计数再用它」（`crates/singlefs-core/src/background_compute.rs:746`）。落点被释放、回收、重新分配给另一个单元之后，新单元继承旧单元的计数：今天两批之间就有，锁外那一段拉长了窗口（推的，没量过）。**核不动**：缺的是计数的键带不带分配代、被释放的落点计数清不清，我没读到清零的那一处。

### 一·5　Q1 小结

| 格 | 判定 | 什么现象会推翻它 |
|---|---|---|
| 前提原样（四元组相同 ⇒ 分类、映射位置、字节都没变） | **不成立**，破在隔离记号与它在会话隔离集里的镜像（一·1）；另一处要求比集合不是比「票里每条」（回退复活，一·2 第 6 行） | 看到 `release_leaving_the_record_allocated_on` 在读盘核没过时仍把 `is_released` 改成 true、或 `compaction_mut().note_quarantined_placement` 不进 `quarantined_slots`：那就不破。现有用例 `crates/singlefs-verification/tests/release_checksum_quarantine.rs:270` 断言了记录那一半，会话那一半（扫描判隔离）没有用例，要实现员补 |
| 补上三样（整条记录与集合相等、现行那一版重取隔离集并先跑丢意图那一判、其余交发布路径逐条核） | 八条路径逐条读完没找到反例 | 找到一条在 R 里改单元字节或映射位置而不动整条记录、也不进隔离集的路径；或发布路径逐条核漏掉「key 在、位置是源落点、字节核得上」里的一样 |
| 字节、钉住 | 不破（读码） | 找到不经「已回收再分配」就重写一个未释放记录罩住的槽的写（我没读到；grep 面只覆盖 `allocator.rs` 的分配与释放，没有逐个读各发布变体里的写） |
| 回退的 discard 窗口、GPU 计数的键 | 核不动 | 见一·2 第 6 行、一·4 |

## 二、Q2：第 6 条签发那一支

先对一遍今天的代码，因为正文第五节 Q2 的头一句读的是「新进这一段的记录直接进计划」：

- **签发那次发布不带计划**：「CompactionIntentStep::Issue(issued),」（`crates/singlefs-core/src/mounted_session.rs:5605`）的上一行是 `&[],`（`crates/singlefs-core/src/mounted_session.rs:5604`，计划为空）。签发只把意图（R、`c0`、`segs0`）写进 logged_ops，一个单元都不搬。
- **每一批都对 R 重扫重核**：「let scanned = scan_of_the_batch(region, &*devices, compute)」（`crates/singlefs-core/src/mounted_session.rs:5634`，在有活意图那一支开头），说明写在 `crates/singlefs-core/src/mounted_session.rs:5333` 「对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）」。新进 R 的记录要进计划，只能经下一批的这次重扫过甲、乙两段。
- **`c0` 与 `segs0` 取签发所用那一版**：「allocation_generation_at_issue: version.root.checkpoint_txg,」（`crates/singlefs-core/src/mounted_session.rs:5591`）、「empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(」（`crates/singlefs-core/src/mounted_session.rs:5592`）。锁外扫的是旧一版、回锁里取现行那一版签发，`c0` 随现行那一版：旧一版到现行那一版之间落进 R 的记录（签发之前 R 还不排他）的分配代都不大于 `c0`，目标态把它们算进去，下一批重扫会看见。

| 子问 | 判定 | 依据与缺什么 |
|---|---|---|
| 没经过甲、乙核的记录直接进计划，发布路径逐条核兜不兜得住 | **不成立**（前提读错：它们不进计划） | 上面三条。它们只进「R 里要搬几个」那个计数（`let units_to_move = u64::try_from`，`crates/singlefs-core/src/mounted_session.rs:5569`），喂「if !holes_admit_the_issue(holes, units_to_move) {」（`crates/singlefs-core/src/mounted_session.rs:5581`）；按记录数的上界算，只会多拒、不会少拒（推的） |
| 签发之后第一批对 R 重扫，是不是本来就会重核 | **成立** | `crates/singlefs-core/src/mounted_session.rs:5634` |
| 所以签发这一步根本不需要锁外的扫描结果 | **推不出** | 签发要的有两样来自扫描：选段资格「R 里至少一个能搬的单元」（「Some(scan) if scan.movable_units().next().is_some() => {」，`crates/singlefs-core/src/compaction_plan.rs:248`）与 `units_to_move`。资格只能先扫再判，这一圈就是读盘候选甲、乙、丙在管的事（Q4 第 1 行）；`units_to_move` 能换成现行那一版上 R 的记录数的上界。缺的量：用上界后多拒的比例（要用例量，不是读码推得出） |
| 选到的段在新一版上已经全空 | **成立**，要重判 | 选段那一步把没有未释放记录的段当「已扫、没有能搬的」（`region_scans_of_the_segments_without_an_unreleased_record(` 在 `crates/singlefs-core/src/mounted_session.rs:5477`）。回锁里不在现行那一版上重判，就会给一个空的 R 签意图：下一批 `PlannedBatch::RegionEmptied`（`crates/singlefs-core/src/mounted_session.rs:5835`）删意图、记成「完成而没有批」，`compaction.last_completed_intent = Some(intent);`（`crates/singlefs-core/src/mounted_session.rs:5717`）使之后停在 `CompactionRunStop::NoYieldFromTheLastIntent {`（`crates/singlefs-core/src/mounted_session.rs:5433`），要等下一次用户改动才清：`compaction.last_completed_intent = None;`（`crates/singlefs-core/src/mounted_session.rs:3065`）。两个发布加一个停，不伤盘 |
| 选到的段已被前台填满 | **成立**，要重判 | 前台写只落进已开的聚簇段，已开的段不当候选：「is_opened_in_this_mount: cluster_segments.contains(&start_slot),」（`crates/singlefs-core/src/compaction_plan.rs:240`），读的是 `allocator.cluster_segments()`（`crates/singlefs-core/src/mounted_session.rs:5500`），分配器的状态，回锁里在现行那一版上再读一次 |

**要交实现员的重判清单**（回锁里签发之前、在现行那一版上各取一次，不用锁外的结果）：`allocator.cluster_segments()`、钉住 ∪ 隔离的槽（「let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = read_of_the_version」，`crates/singlefs-core/src/mounted_session.rs:5441`）、没有未释放记录的段、被区域扫描拒过的段表，之后才是签发前那一判。

## 三、Q3：前两轮的数能不能核，射程写没写够

### 三·1　「会丢已答成功的发布不采」的射程

第二轮判决写：「有活意图那一支、建快照与删快照的旧底没扫」（`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-main-verification.md:29`）。这一轮形态回锁之后在现行那一版上发布：发布用的是回锁里取的那一版（`publish_the_planned_compaction_step_falling_back_on_the_region_check(` 在 `crates/singlefs-core/src/mounted_session.rs:5867`，今天传的是一批开头克隆出来的 `version`：`crates/singlefs-core/src/mounted_session.rs:5686`），不再有「旧底上发布、靠释放目标核拒掉」这一件事。所以**这一句本身不再要紧**。

**要紧的是它换成的那一句**：锁外判的计划、对现行那一版对不上时，发布路径是不是都在写盘之前拒。文档写的是「计划对不上上一版 ⇒ … 那四种，在任何读写之前」（`crates/singlefs-core/src/transaction.rs:10755`）；逐条核的代码在 `crates/singlefs-core/src/transaction.rs:11370` 起，我读完的结论在一·1：对得上就写、对不上拒整批，没有写坏盘的路径，但隔离那一支让「剔掉」变成「整批被拒」。第二轮的 42 段历史（旧底 × 三种前台改动 × k 到 48，`stale_publish_made=0`）扫的是另一件事，**没覆盖这一件**：有活意图 × 前台改动的种类（覆盖写、顺序写、建 inode、截断、unlink、建快照、删快照、读坏一份副本之后的覆盖写）× 锁外期间前台发几次。

判定：这一格 **推不出**。缺的是一个探针（本腿没写：要先有第四节形态的实现才有可调的入口；今天的代码上能写的是「在会话里手工摆：取 R 的扫描与计划，放进前台改动，再对现行那一版发今天的发布入口」，读数是 `refused / published / wrote_nothing`）。什么现象会推翻「不写坏盘」：某一种前台改动之后，对现行那一版发出去的计划被发布路径收下、而 R 里那个单元的位置已不是源落点或字节已不是它。

### 三·2　忙窗口基线的睡眠是摆的，各一趟：「今天读最坏等 1.10 秒」站不站得住

第二轮探针的产物原样（`~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet-run/out/` 下六份，`grep -h -o 'name=busy_window_foreground_wait .*'` 现取的整行；每份各一趟，读线程四条紧循环、不睡）：

```text
busy-cpu_route_readers4_tight.out
name=busy_window_foreground_wait route=cpu sleep_ms_per_gpu_call=0 readers=4 writers=0 gap_us=0 samples=138891749 wait_micros_mean=0 wait_micros_p99=3 wait_micros_max=110895 maintenance_publishes_total=60 blocked_samples=1649 blocked_wait_micros_p50=1839 blocked_wait_micros_p99=64026 blocked_wait_micros_max=110895 blocked_publishes_none=1570 blocked_publishes_one=33 blocked_publishes_two_to_three=34 blocked_publishes_four_or_more=12 writer_calls=0 writer_call_micros_p99=0 writer_call_micros_max=0
busy-double_sleep0_readers4_tight.out
name=busy_window_foreground_wait route=gpu_double sleep_ms_per_gpu_call=0 readers=4 writers=0 gap_us=0 samples=191954718 wait_micros_mean=0 wait_micros_p99=2 wait_micros_max=130610 maintenance_publishes_total=60 blocked_samples=168 blocked_wait_micros_p50=2005 blocked_wait_micros_p99=115005 blocked_wait_micros_max=130610 blocked_publishes_none=136 blocked_publishes_one=8 blocked_publishes_two_to_three=10 blocked_publishes_four_or_more=14 writer_calls=0 writer_call_micros_p99=0 writer_call_micros_max=0
busy-double_sleep5_readers4_tight.out
name=busy_window_foreground_wait route=gpu_double sleep_ms_per_gpu_call=5 readers=4 writers=0 gap_us=0 samples=177687894 wait_micros_mean=0 wait_micros_p99=2 wait_micros_max=165487 maintenance_publishes_total=60 blocked_samples=83 blocked_wait_micros_p50=2068 blocked_wait_micros_p99=165487 blocked_wait_micros_max=165487 blocked_publishes_none=46 blocked_publishes_one=6 blocked_publishes_two_to_three=25 blocked_publishes_four_or_more=6 writer_calls=0 writer_call_micros_p99=0 writer_call_micros_max=0
busy-double_sleep50_readers4_tight.out
name=busy_window_foreground_wait route=gpu_double sleep_ms_per_gpu_call=50 readers=4 writers=0 gap_us=0 samples=104512640 wait_micros_mean=0 wait_micros_p99=1 wait_micros_max=1104695 maintenance_publishes_total=60 blocked_samples=1076 blocked_wait_micros_p50=1995 blocked_wait_micros_p99=551723 blocked_wait_micros_max=1104695 blocked_publishes_none=1003 blocked_publishes_one=28 blocked_publishes_two_to_three=34 blocked_publishes_four_or_more=11 writer_calls=0 writer_call_micros_p99=0 writer_call_micros_max=0
busy-double_sleep5_readers4_writer1.out
name=busy_window_foreground_wait route=gpu_double sleep_ms_per_gpu_call=5 readers=4 writers=1 gap_us=0 samples=2961152 wait_micros_mean=23 wait_micros_p99=0 wait_micros_max=401011 maintenance_publishes_total=330 blocked_samples=1075 blocked_wait_micros_p50=49312 blocked_wait_micros_p99=219264 blocked_wait_micros_max=401011 blocked_publishes_none=353 blocked_publishes_one=376 blocked_publishes_two_to_three=307 blocked_publishes_four_or_more=39 writer_calls=199 writer_call_micros_p99=247099 writer_call_micros_max=281520
busy-double_sleep50_readers4_writer1.out
name=busy_window_foreground_wait route=gpu_double sleep_ms_per_gpu_call=50 readers=4 writers=1 gap_us=0 samples=521701 wait_micros_mean=114 wait_micros_p99=0 wait_micros_max=1154895 maintenance_publishes_total=113 blocked_samples=229 blocked_wait_micros_p50=181417 blocked_wait_micros_p99=1154892 blocked_wait_micros_max=1154895 blocked_publishes_none=76 blocked_publishes_one=61 blocked_publishes_two_to_three=64 blocked_publishes_four_or_more=28 writer_calls=48 writer_call_micros_p99=1169025 writer_call_micros_max=1169025
```

第二轮判决自己也标了：「这是今天锁内等的代价的第一份量到的数（替身卡，睡眠是摆的）」（`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-main-verification.md:31`）。逐条对：

1. **睡 50 毫秒那一行是真的，但它说的是替身睡 50 毫秒**。E185 量的真卡：「第 4 次重跑一次 GPU 提交的挂钟约 5.3 ms」（`24-后台重活能不能卸给GPU.md:40`），同一行写明「量的是含核在内的整次提交、不是纯 PCIe 往返」。50 毫秒是它的约 9.4 倍（50 ÷ 5.3，推的算术）。「今天读最坏等 1.10 秒」只能写成「替身每次调用睡 50 毫秒、四条读线程紧循环、一趟」，不能写成真卡上的读数。
2. **替身睡一次、真卡一次调用不是一次提交**：替身在三个方法各睡一次（`std::thread::sleep(self.sleep_for_every_call);` 在 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-model/probes/mount_runtime_busy_window_foreground_wait.rs:154`、`:173`、`:203`，与第二轮那份逐字节相同）；真实现一次调用里是两份独立写的核各算一次：`26-后台整理与放置回收.md:259` 「两份独立写的 GPU 核（不共用核函数）对同一批输入各算一次」，再加按显存切片与超时重交。所以一次调用 ≥ 两次提交，约 10 毫秒量级（5.3 乘 2 约 10.6，推的，没量过）：睡 5 毫秒偏低、睡 50 毫秒偏高，**没有 10 毫秒那一档的读数**。本腿的复跑补 10、20 毫秒两档。
3. **最坏等待里不含 GPU 的那一块就有 111 到 131 毫秒**：CPU 路径被挡最坏 110895 微秒、替身睡 0 毫秒 130610 微秒。睡 5 毫秒是 165487 微秒，比睡 0 毫秒只多 34877 微秒；睡 50 毫秒多 974085 微秒。折成睡眠次数：34877 ÷ 5000 ≈ 7.0 次、974085 ÷ 50000 ≈ 19.5 次，最坏那一次持锁里的 GPU 调用数在两档上不是同一个数（推的；产物不记每次持锁里调了几次 GPU，这是它的缺口，下面第 4 条补）。在最接近真卡的那一行（睡 5 毫秒）上，拆到锁外能拿走的最坏等待是约 35 毫秒（165487 − 130610），剩下约 130 毫秒是锁内 CPU 活，拆 GPU 等拿不走。这一块里有每批一次整份版本克隆：`.clone();`（`crates/singlefs-core/src/mounted_session.rs:5371`，前面 `.version_of_trees()` 交回的是 `&TransactionOutput`，所以是深拷贝；大小随池走，没量过）。
4. **写一次调用的数**：睡 5 毫秒写线程 p99 247099 微秒、最长 281520；睡 50 毫秒 48 次调用 p99 等于最长 1169025。「写一次调用 p99 是 1.17 秒」同样是睡 50 毫秒的读数，而且 48 次里 p99 就是最长，没有分位数的意义。
5. **趟与趟之间差多少没有数**：每个配置各一趟。本腿的复跑把每个配置连跑三趟。

判定：「今天读最坏等 1.10 秒」**作为真卡上的数不成立**，作为替身睡 50 毫秒的一趟读数成立；与 E185 的真卡数对不上（量纲差 9.4 倍、调用与提交不是一对一）。复跑要看的行：`name=self_check` 与 `name=rerun_done`；判读：同一个睡眠档三趟的 `blocked_wait_micros_max` 极差；睡 10 与 20 毫秒两档相对睡 0 毫秒的增量是不是随睡眠线性。什么现象会推翻第 3 条：睡 0 毫秒三趟的 `blocked_wait_micros_max` 中位数落在 40 毫秒以下（那样 130610 是单趟毛刺）。

### 三·3　攻方第二轮的离散事件模型，哪些格对这一轮的形态还有效

读了第二轮攻方报告里模型一节与各格判定表（`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-output.md:23` 到 `:31`）与产物目录里五个场景的文件名与 `today` 臂的行；`form_model.py` 我没逐行读。按那份报告对 `form` 臂的描述（回锁复核 txg 相等且待发为空、不等整批作废、连着作废 k 批后回锁里算、破例批恒在锁里）：

| 场景（产物名） | 对这一轮形态 | 理由 |
|---|---|---|
| `hang_brought_back_into_the_lock` | **失效**（`today` 臂仍有效，当对照） | 它量的是 k 下限把挂住的那一批带回锁里；这一轮没有 k |
| `hang_with_accepts_only` | **失效** | 量的是「只收下不发」的写流让待发为空判据连着作废；这一轮不作废。换成的新效应（每次回锁先排空待发，先发一次用户改动，现行那一版几乎总比票新）模型里没有格 |
| `exception_batch_hang` | **失效** | 破例批恒在锁里；这一轮破例批照常走三段（第四节第 8 条） |
| `yield_and_floors` | **部分有效** | N 那一维（连着让路 N 次之后一批不让）有效；`yield_resets_k` 与 k 那两维没有对象 |
| `registration_window` | **有效** | try_lock 失败到登记之间的窗，第四节第 9 条原样保留 |
| `idle_pool_spin` | **对象换了** | 原来的空转是作废的新停因默认「接着叫」；这一轮没有作废，但新增的停因（冻结丢批、没资格换段、让路给前台等锁者）同样会落进反向匹配：「!matches!(」（`crates/singlefs-mount-runtime/src/lib.rs:671`）名单里只有 `YieldedToAForegroundFsync`、`YieldedToTheCallerStoppingItsThread`、`WriteBandwidthLimitReached` 三个不接着叫，别的停因默认接着叫，一拍至多 64 次：`pub const COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST: u32 = 64;`（`crates/singlefs-mount-runtime/src/lib.rs:61`）。新停因进不进那张名单要在第四节形态里写明（推的，没量过） |
| 模型自带的「今天」臂 | **有效** | 就是今天的做法；时长是拍的（一批 GPU 一段 10 毫秒、卡挂住一段 40 秒） |
| 缺 | — | 这一轮形态自己的 `form` 臂（取输入、锁外算、回锁重排、剔单元、让路 N、冻结丢批）与有活意图 / 签发两支。输入要量：前台改动命中 R 里单元的比率（决定每批剔掉几个）、卡挂住时一批的重交次数 |

## 四、Q4：岔路单与未定项清单

### 四·1　岔路单（这一轮形态里每条岔路一行；只写问题与候选定义，不写倾向、不写已有的数）

状态三种：开着 / 够判 / 用户已定。本腿不把任何一行标「够判」（够判要量过）。

| 候选（各自的定义） | 翻面观测（量出什么数会让选择换面） | 够判条件 | 状态 |
|---|---|---|---|
| **读盘·签发那一支每圈要读一段**（第四节第 6 条）<br>甲：选段循环每一圈回会话锁里读那一段，读完放锁，下一圈的 GPU 等在锁外<br>乙：盘表与会话分成两把锁，读在会话锁外（今天 `SessionAndDevices` 是一个结构、一把锁：`pub struct SessionAndDevices<Device> {`，`crates/singlefs-mount-runtime/src/lib.rs:387`）<br>丙：签发那一支（选段循环、判空洞、签发）整段留在会话锁里，只有有活意图那一支走三段 | 甲：一次签发的选段圈数（今天迭代上界是单元区整段数：`crates/singlefs-core/src/mounted_session.rs:5494` 的注释）乘每圈锁内读盘毫秒，对比每圈一次取放锁的前台等待；乙：盘表锁分成读与写之后，真盘块设备能不能一边读一边写（接口里读是 `&self`、写是 `&mut self`：`&self,` 在 `crates/singlefs-core/src/block_device.rs:298`，`&mut self,` 在 `crates/singlefs-core/src/block_device.rs:313`；真盘实现没读）；丙：签发批占一条意图全部批的比例（第二轮辩方读码推「至少 1/5」，没量过）与签发批里 GPU 等的挂钟 | 一次签发的选段圈数分布（代表性池、几个碎片度）；每圈 GPU 调用数；真卡一次调用毫秒（本腿的 10、20 毫秒两档补一部分）；乙还要真盘块设备并发读写的结论 | 开着 |
| **读盘·有活意图那一支读 R**（第四节第 2 条写死在锁里）<br>甲：取输入时在会话锁里把 R 里的单元读完放进票里（至多一段，64 槽、每槽 16 KiB 约 1 MiB）<br>乙：锁外读（同上一行乙的前提） | 一段读盘的锁内毫秒（真盘、几何）；前台读路径被它挡的最坏等待 | 一段读盘在真盘上的挂钟分布 | 开着 |
| **维护线程自己的发布怎么照跑**（第四节第 10 条）：`publish_when_due` 与重发冻结今天都在维护线程那一条线程上（`self.publish_when_due_once()?;` 在 `crates/singlefs-mount-runtime/src/lib.rs:606`、`:685`、`:706`），默认每 1 秒一次：`pub const PUBLISH_POLL_PERIOD_BY_DEFAULT: Duration = Duration::from_secs(1);`（`crates/singlefs-mount-runtime/src/lib.rs:59`）<br>甲：另起一条发布线程，专跑这两件事，与维护线程各自取会话锁<br>乙：锁外那一段拆成可轮询的等，每次短等之间回会话锁跑这两件事（GPU 包里的等是一个循环，今天每一轮等满剩下的全部时间：`pub fn wait_for_completion_before_the_deadline<Output>(` 在 `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:99`）<br>丙：不加，到期发布等锁外那一段结束再跑，丢失窗口（D16 的 P）被拉长到锁外那一段的时长 | 锁外那一段超过 P 的批占比（真卡、含超时重交；一次等上限默认 10 秒：`pub const BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT: Duration = Duration::from_secs(10);` 在 `crates/singlefs-core/src/background_compute.rs:905`，超时最多重交 3 次：`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:120`）；前台 `accept` 自带按钟的到期判（`crates/singlefs-core/src/mounted_session.rs:7076`、`:7020`），只有「最后一次收下之后安静」那一段靠线程 | 锁外那一段的时长分布；到期判被锁外那一段延迟的最坏值；乙与下一行卸载的乙是不是同一处改动（见 §四·3） | 开着 |
| **卸载与 Drop 等不等锁外那一批**（第四节第 11 条）<br>甲：照旧等（`stop_and_join_the_maintenance_thread` 的注释：`crates/singlefs-mount-runtime/src/lib.rs:1109`）<br>乙：GPU 包里等的那一处查停标记，卸载置标记，锁外那一批在下一轮短等醒来时放弃 | 卡挂住的替身上卸载的挂钟（甲：至多约 40 秒，一次等上限 10 秒乘一次加三次重交，推的，没量过；乙：一次短等）；乙放弃时卡上还在跑的提交泄漏的卡数 | 卡挂住的替身上的卸载挂钟（第一轮正文 Q6 验收清单里「卡挂住的替身上前台与卸载各等多久」那一项）；乙改动的调用点数（见 §四·3） | **用户已定**：2026-10-28 弹窗「卸载不等」（`26-后台整理与放置回收.md:269`），条款 `26-后台整理与放置回收.md:255` 「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」；今天的运行库没做到（上面甲的注释）。甲与已定相左，这一行只剩「怎么做」 |
| **冻结的发布回锁时**（第四节第 7 条）<br>甲：不发，这一批丢掉，记单独的停因<br>乙：回锁里先重发冻结的那一次（前台 fsync 与卸载今天都这样做），重发成了再比对再发；没成这一批丢 | 冻结时锁外有批在途的频率；丢批白算一批 GPU 的代价对多一条后台写路径；冻结时分配器已退回发布之前那一份（「冻结时分配器退回发布之前那一份」，`crates/singlefs-core/src/mounted_session.rs:4135`），票里读的分配器状态可能与它不一致 | 写失败冻结的实际频率（今天没量过，欠账里没有它的数）；乙的重发在后台路径上多出的分支数 | 开着 |
| **回锁里比对什么**（第四节第 5 条写成四元组）<br>甲：整条 `AllocationRecord`（含隔离记号）与 R 里未释放记录的集合相等，加回锁后重取隔离集（一·1）<br>乙：甲再加对计划里每个单元在会话锁里用 CPU 重判一次分类（读盘，至多一段） | 甲：随机历史里「剔掉」与「整批被拒」各多少次（`crates/singlefs-verification/tests/compaction_while_mounted_in_random_histories.rs` 是现成的历史装置）；乙：多出的锁内读盘毫秒 | 一·1 补的会话那一半用例；随机历史上甲零漏判（整批被拒的次数为 0） | 开着 |
| **签发那一支回锁里取什么**（第四节第 6 条；Q2）<br>甲：用票里的扫描数出「要搬几个」，记录变了的单元算要搬<br>乙：用现行那一版 R 的未释放记录数当上界<br>丙：回锁里 CPU 重扫选中那一段（读盘一段） | 上界比能搬数多拒的比例；丙的锁内读盘毫秒 | 签发批上「上界 − 能搬数」的分布（Q2 记的缺口） | 开着 |
| **签发没资格之后**（第四节第 6 条写死「下一批选别的段」）<br>甲：记一个不再选的段集合（新状态，范围：这一趟 / 这次挂载）<br>乙：不记，下一批照旧重选，可能还是同一段 | 同一段连续没资格的次数；每次没资格白算的 GPU 调用数 | 连续没资格次数的分布 | 开着 |
| **让路的 N**（第四节第 9 条：连着让路 N 次之后下一批不让，N 自己计、一批发成才清零）<br>甲：N 取固定小整数（常量或挂载参数）<br>乙：不计次数、计让路累计毫秒，超过 T 毫秒下一批不让 | 前台到达率从低到高扫，后台一趟发成的批数与前台最坏等待的两条曲线（第二轮攻方模型 `yield_and_floors` 里的 N 那一维有同形的格，时长是拍的） | 在这一轮形态的实现上的真线程用例里扫 N（或 T）；没有实现就没有这条曲线 | 开着 |
| **锁外那一段含多少**（第四节第 2、3 条没写清）<br>甲：只含 GPU 或 CPU 三段的核调用<br>乙：含全部纯计算：取输入只拷 `Arc`，建映射条目、钉住集、按身份索引、重扫分类、选段打分、算计划都在锁外从不可变的那一版算，回锁里只比对、重排与发布 | 锁内 CPU 时间里这几块各占多少（今天的读数是总量：睡 0 毫秒被挡最坏 130610 微秒、CPU 路径 110895 微秒，见 §三·2）；一批开头那次整份版本克隆 | 锁内各段（克隆、建映射条目、建索引、重扫、计划、发布）分段计时；本腿没有 | 开着 |
| **切换校验读什么**（第四节第 11 条写死「读卡清单快照」）<br>甲：读 GPU 把手带的卡清单快照（开卡、换卡时更新）<br>乙：`try_lock` GPU 把手，拿不到就拒这次切换<br>丙：校验推迟到下一批开头，由维护线程在会话锁外做 | 甲：快照陈旧的窗（卡在切换那一刻已不在，快照还列着）；乙：切换被拒的频率；丙：切换生效延迟的批数 | 卡清单实际变化的频率与锁外占比（今天校验在会话锁里取 GPU 把手锁：「let implementation = gpu_compute.lock();」，`crates/singlefs-core/src/background_compute.rs:1176`） | 开着 |

### 四·2　被写死而该是岔路的（正文第四节没写成候选的）

1. 回锁里比对什么：第 5 条写死四元组，一·1 打中，要改成上表那一行。
2. 锁外那一段含多少：第 2、3 条只说「锁外算」，GPU 路径与 CPU 路径边界各在哪没写（上表倒数第二行）。
3. 切换校验读卡清单快照（第 11 条）：第一轮判决里它是三个候选（缓存卡清单、`try_lock` 拒、校验推迟），这一轮写成了单一做法。
4. 冻结的发布回锁时丢批（第 7 条）、签发没资格选别的段（第 6 条）：各有一个没写的对面做法。
5. 有活意图那一支在会话锁里读 R（第 2 条）：第 6 条的读盘岔路只写了签发那一支。

### 四·3　被写成岔路而站得住的只有一个（或候选自己立不住）的

1. **卸载等不等**：候选甲（照旧等）与用户已定相左（`26-后台整理与放置回收.md:269`、`26-后台整理与放置回收.md:255`）。留下的候选乙要改 GPU 包里等的那一处：那个循环是 `wait_for_completion_before_the_deadline`（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:99`），它在包里有四处调用（`crates/singlefs-background-gpu/src/background_gpu.rs:2223`、`:2312`，`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:796`、`:1005`），`grep -n 'stopping\|cancel\|叫停' crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs` 零命中（量的）。这一处改动与「维护线程自己的发布」的乙是同一处（等的循环拆成短等）：卸载乙做了，发布乙几乎白送；反过来卸载乙不做，发布乙要单独改它。两行不是互相独立的岔路。
2. **读盘丙**（签发那一支整段留在会话锁里）：这一支的 GPU 等留在会话锁里，字面上与用户定的「等 GPU 拆到锁外」相左；除非用户放宽到「只拆有活意图那一支」，它不是候选。
3. **读盘乙**（盘表分锁）：能不能立，取决于真盘块设备能不能一边读一边写，这一点两轮都没读（第二轮判决 Q4 把它挂起）；在没读之前它不该和甲、丙并列写成候选（见 Q6 第 2 条）。

**每条岔路给得出翻面观测与够判条件吗**：读盘、发布线程、冻结、比对粒度、签发取什么、没资格、锁外含多少、切换校验这八行，翻面观测与够判条件都写得出，但够判条件里的量除「真卡一次调用毫秒」外今天都没有（要实现之后的探针或分段计时）；N 一行的观测要实现之后才有（没有曲线，不是没有定义）；卸载一行已定。

## 五、Q5：崩溃语义

**同一输入下，剔过的计划与今天锁内整批重算的发布逐项相同吗**：只在剔掉 0 个单元时相同。今天的计划是现行那一版上可搬的单元按槽升序取前 K 个（`plan_batch` 的说明：`crates/singlefs-core/src/compaction_plan.rs:307` 「扫描结果里可搬的单元按槽升序取前 `batch` 个」，K 默认 16）。新形态是旧一版上取前 K 个、再剔掉记录变了的：旧一版前 K 个里每剔一个，批就小一个，旧一版第 K+1 个及以后的单元不会补进来，而今天重算会补。所以剔掉 ≥ 1 个时，新批是今天那一批的真子集（次序仍是槽升序）。对计划的第二处读法也跟着变：「if planned.len() == scan.allocations.len() {」（`crates/singlefs-core/src/mounted_session.rs:5819`，判「这一批罩住 R 里每一条未释放记录就同一次发布删意图」）读的是旧一版的扫描；现行那一版上 R 的记录数不同时，发布路径在算好的账上再核，核不过就照同一批删意图或照抄再发（`RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried`，`crates/singlefs-core/src/transaction.rs:10723`；`RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted`，`crates/singlefs-core/src/transaction.rs:10718`）。

**盘上多出哪些写序列**（读码推的，没量过）：没有新的写种类。批的字节、journal 点名项与发布顺序都由同一个发布函数按「现行那一版 + 计划」算：`pub fn publish_relocation_batch<Device: BlockDevice>(`（`crates/singlefs-core/src/transaction.rs:10759`）进 `publish_version` 再进 `publish_admitted`，计划里每个单元的字节取现行那一版（或读盘核校验和过的那一份，`crates/singlefs-core/src/transaction.rs:11465`、`:11483`），不用锁外读到的字节。新代码只多一步「发布之前把计划剔小」。多出来的是「批大小 j < K 的、不是前缀的子集」与「分类在旧一版、发布在新一版」这两个组合：前一个今天已能到（发布路径拒「本来就重写的节点」、会话把它们并进跳过的集合再计划，`crates/singlefs-core/src/mounted_session.rs:5659` 起的那个循环，用例 `crates/singlefs-verification/tests/compaction_batches_skip_the_nodes_the_publish_rewrites_anyway.rs`），后一个今天到不了（今天分类与发布同一版）。

**现有崩溃流罩不罩得到**：整理搬迁那几条在 `.claude/gate.d/stage-inputs.tsv`：`crash-case:compaction-relocation-stream`（68）、`crash-case:compaction-relocation-on-the-gpu-path-stream`（69）、`crash-case:compaction-relocation-switching-backends-between-batches-stream`（70）、`crash-case:deletion-interleaved-with-compaction-across-a-rollback-stream`（71）、`crash-case:compaction-relocating-a-block-shared-with-a-snapshot-stream`（72）、`crash-case:compaction-deadlist-reclamation-stream`（107）、`crash-case:compaction-with-a-write-while-the-intent-is-live-stream`（160）、`crash-case:compaction-paused-short-of-space-dropping-the-intent-stream`（161）。其中 160 是活意图期间前台写，批与批之间 `V` 变了，但每一批的计划仍在它自己的现行那一版上算。罩没罩到「一批的子集计划 × 分类版本比发布版本旧」没法读出来：缺的是枚举那一格的计数（要新流的话，比已有流多罩的是「计划是旧版扫描的真子集、发布在比它新的一版上、其间前台发了 m 次」这一个组合）。

判定：**推不出**。读码推得出的是不新增写种类；推不出的是现有流罩不罩得到那个组合。什么现象会推翻「不新增写种类」：新实现在剔单元那一步之外又有绕过 `publish_admitted` 的写，或计划里的字节取自锁外读盘。要主 agent 看的：实现之后把 160 那条流改一份「批的计划取自旧一版、剔过」的变体，枚举计数对比原流（本腿没有新流）。

## 六、Q6：最终轮的总判

### 六·1　换到的（用前两轮量到的数说）

数全在 §三·2 的原样行里，这里只引算出来的差，标「推的算术」：

1. 最接近真卡一次提交（约 5.3 毫秒）的那一行是替身睡 5 毫秒：被挡最坏 165487 微秒，睡 0 毫秒 130610 微秒；差 34877 微秒（推的算术）。**拆 GPU 等出锁，在这一档上拿走的是约 35 毫秒的最坏等待**，不是 1.10 秒。
2. 睡 50 毫秒那一行 1104695 微秒，与睡 0 毫秒的差 974085 微秒（推的算术）：这是替身睡 50 毫秒时能拿走的，约是真卡一次提交的 9.4 倍。
3. 剩下的最坏等待 110895（CPU 路径）到 130610（睡 0 毫秒）微秒是锁内 CPU 活，拆 GPU 等拿不走；任何前台等锁都让路也只能让后台不再取下一批，缩不短已经在跑的那一批。
4. 卡挂住：今天一段持会话锁最多约 40 秒（一次等上限 10 秒，超时重交至多 3 次：`pub const BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT: Duration = Duration::from_secs(10);` 在 `crates/singlefs-core/src/background_compute.rs:905`，`pub const RESUBMISSIONS_AFTER_A_TIMEOUT: u32 = 3;` 在 `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:120`，读码算的）；第二轮攻方模型的 `today` 臂量到读等满约 40 秒，同量级（「读等满约 40 秒（攻方模型）」，`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-main-verification.md:33`；模型里的时长是拍的）。拆到锁外之后不再挂前台，这是推的，没量过；换到的最实在的一项。
5. 前台等锁让路：忙窗口基线里被挡的取样拿到锁之前后台连着发 4 次以上的，一趟 6 到 28 个（第二轮判决「缺口 2」一行，`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-main-verification.md:31`）；让路只在批与批之间，换到的是这一部分。

### 六·2　多出来的状态、分支、参数（逐条）

**状态（六样）**

1. 会话上「有一批在锁外」的标记，第二个后台调用者在取 GPU 锁之前被拒（第四节第 1 条）；它对应一个停因。
2. 票：现行那一版的 `Arc`（`pub fn shared_version_of_trees(&self) -> Option<&Arc<TransactionOutput>>` 在 `crates/singlefs-core/src/transaction.rs:2637`）、它的 txg、活意图与 R、隔离集与被拒段表的快照、只供测试的开关快照；有活意图那一支再加 R 里单元的字节（至多一段约 1 MiB）。GPU 的输入表（`CentralMappingIdentityIndex` 借着映射条目的切片）要么票自带，要么锁外按 `Arc` 重建。
3. 让路的两个计数：前台等锁者数（锁外的原子数，`try_lock` 失败才登记）与连着让路次数（N 自己计，一批发成才清零）。
4. 回锁时「剔掉了几个单元」「没资格换段」「冻结丢批」「让路」的观测量与停因成员。
5. 会话上「现行那一版的隔离集」要在回锁之后再取一次（一·1），分配器那一侧在锁外那一段期间会被前台的发布继续写。
6. 新停因进 `keeps_going` 的名单（`crates/singlefs-mount-runtime/src/lib.rs:671`）。`CompactionRunStop` 是封闭集合，加成员要改两处穷举匹配（`crates/singlefs-core/src/compaction_plan.rs:687`、`:728`）。

**分支（八样）**

1. 取输入时让路的问答：`AnswerOfTheCallerBeforeTheNextBatch` 今天三个成员（`pub enum AnswerOfTheCallerBeforeTheNextBatch {` 在 `crates/singlefs-core/src/compaction_plan.rs:1541`）加一个「有前台在等锁」，今天的问答点在 `crates/singlefs-core/src/mounted_session.rs:4732`。
2. 回锁里出错先于一切（第 4 条）。
3. 回锁里：冻结丢批、排空待发被拒丢批（`crates/singlefs-core/src/mounted_session.rs:7670` 起的那一支今天只管「起跑之前排空」）、有活意图那一支剔单元与剔完不发、签发那一支重判资格与没资格、一批发成。
4. CPU 路径走同样的三段：今天 CPU 与 GPU 是同一个函数里的两个臂（`enum CompactionComputeOfTheBatch<'gpu> {` 在 `crates/singlefs-core/src/mounted_session.rs:5217`），拆三段之后两个臂各要有锁外那一段的形状。
5. 破例批照常走三段，破例排在让路之前（`26-后台整理与放置回收.md:256` 「破例排在第 6 条的让路之前，前台有 fsync 在等也照发」）。
6. 切换校验读卡清单快照；GPU 把手锁中毒交回错误（今天是 panic：`crates/singlefs-core/src/background_compute.rs:746` 的 `expect`）。
7. 卸载：乙要的停标记；甲要的「卸载等锁外那一批」与已定条款相左（Q4 §四·3）。
8. 维护线程自己的发布：另一条线程或轮询。

**参数**：N（或 T）；乙的短等切片；甲的发布线程周期。没有 k。

### 六·3　用户定的两条，今天的 `crates/` 上做不做得成、要不要先做别的

**条一（等 GPU 拆到锁外）：做得成，不需要先做盘表分锁，也不需要先做块设备的并发。**

- GPU 把手今天就是 `Arc<Mutex<…>>` 的共享句柄：`pub fn of_a_shared_implementation(shared: Arc<Mutex<dyn BackgroundGpuCompute + Send>>) -> Self {`（`crates/singlefs-core/src/background_compute.rs:735`），一批取的是它的克隆：「let gpu_compute = batch_under_way.gpu_compute_of_the_route().cloned();」（`crates/singlefs-core/src/mounted_session.rs:5142`），借出在 `crates/singlefs-core/src/mounted_session.rs:5160`。借出可以挪到会话锁之外。
- 读盘留在锁里（读盘候选甲或丙，或有活意图那一支的 R 读进票里）就用不着盘表分锁：`SessionAndDevices` 一个结构一把锁（`crates/singlefs-mount-runtime/src/lib.rs:387`、`:396`）不用动。**只有读盘候选乙需要盘表分锁**，而候选乙要的块设备一边读一边写今天没核（读是 `&self`、写是 `&mut self`，`crates/singlefs-core/src/block_device.rs:298`、`:313`；真盘实现没读）——它是「要先做别的」，且不是条一必须的。
- **要先做的一处在 GPU 把手上，不是盘表也不是块设备**：切换校验今天在会话锁里取 GPU 把手锁：「let implementation = gpu_compute.lock();」（`crates/singlefs-core/src/background_compute.rs:1176`）。锁外那一段持着它时，前台切换持着会话锁在它上面等满整段（第一轮判决量到：`switch_waited_ms=1595`，持 1600 毫秒）。所以第 11 条的「读卡清单快照」（或 `try_lock` 拒、或校验推迟）是条一的先决，不是可选件。另三处取 GPU 把手锁的（`crates/singlefs-core/src/background_compute.rs:1103`、`:1105`、`:1124`）是两批之间维护线程自己调的，与锁外那一段同线程、先后不交叉（读码，推的）。
- **「GPU 包里等的那一处」条一不需要**。需要它的是卸载不等（已定条款，与条一无关但互相牵连）与发布线程的乙：`wait_for_completion_before_the_deadline`（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:99`）加停标记、四处调用点各按短等切片。

**条二（任何前台等锁都让路）：做得成，不需要先做别的；它换到的有限。**

- 前台入口都经同一个取锁处：`fn lock_the_session(&self)`（`crates/singlefs-mount-runtime/src/lib.rs:491`），前台五处在 `crates/singlefs-mount-runtime/src/lib.rs:963`、`:991`、`:1030`、`:1054`、`:1079`，维护线程自己三处在 `:562`、`:579`、`:613`。登记「前台等锁」只能加在前台五处（或给取锁处加一个区分调用者的参数）；今天只有 `publish_pending_now` 登记：「let _waiting = self.foreground_fsync_is_waiting();」（`crates/singlefs-mount-runtime/src/lib.rs:987`）。
- 核心层的问答与停因加成员即可（上面分支第 1 条）。
- 它换到的：让后台不再取下一批、前台拿得到锁；缩不短已经在跑的一批（六·1 第 3 条）。要缩短一批本身要靠条一加「锁外那一段含全部纯计算」（Q4 §四·1 倒数第二行），那一项不是用户定的两条里的。

**总判**：两条都做得成。真正要先做的只有三处，且都不在盘表、块设备、GPU 包等的循环上：切换校验不取 GPU 把手锁；回锁里在现行那一版上重取隔离集并比整条记录；新停因进 `keeps_going` 名单。盘表分锁与块设备并发只有读盘候选乙需要；GPU 包里等的循环只有卸载不等（已定）与发布乙需要。

什么现象会推翻「条一不需要先做盘表分锁」：实现读盘候选甲或丙时发现一次签发的选段圈数大到每圈回锁读让前台的等待超过今天（Q4 第 1 行的翻面观测）。什么现象会推翻「换到约 35 毫秒」：睡 10、20 毫秒两档的复跑里，睡 0 毫秒三趟被挡最坏的中位数低于 40 毫秒（那样今天那 130 毫秒是毛刺，换到的会比 35 毫秒大）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 前提原样 | **不成立** | 四元组管不着隔离记号：释放前读盘核没过，记录 `is_released` 与分配代都不变、只多隔离记号，单元已被释放或已被另发的一批搬走（`crates/singlefs-core/src/allocator.rs:2821` 到 `:2828`；用例 `release_checksum_quarantine.rs:270` 断言记录那一半） |
| Q1 补三样之后 | 八条路径读码没找到反例；两处核不动 | 补：整条记录加集合相等、回锁后在现行那一版重取隔离集并先跑丢意图那一判、其余交发布路径逐条核（它拒整批、不写坏盘）；核不动：回退落盘与 discard 窗口、GPU 连续不确定计数的键不带分配代 |
| Q1 R 排他 | 条款在已定项 1 第 3 条，不在已定项 2 | 只挡新落点进 R，挡不住释放、隔离、位置变化、回退复活 |
| Q1 GPU 交 CPU 判 / 不确定 | 交 CPU 判不破；不确定名单安全；计数键核不动 | 发布路径兜底把旧名单换成同一批删意图 |
| Q2 没核过的记录直接进计划 | **不成立** | 签发发布不带计划（`mounted_session.rs:5604`），新进 R 的记录只经下一批重扫进计划 |
| Q2 签发之后第一批本来就重核 | **成立** | `mounted_session.rs:5634` |
| Q2 签发不需要锁外扫描结果 | **推不出** | 选段资格与「要搬几个」要分类；缺：用记录数上界后多拒的比例 |
| Q2 选到的段已全空 / 已被填满 | **成立**，回锁要重判 | 不重判会签空意图（两发布一停）；要在现行那一版上重取已开聚簇段、钉住与隔离、没有未释放记录的段 |
| Q3 旧底射程那一句 | 不再要紧；换成的那一句**推不出** | 有活意图 × 前台改动种类（含隔离）的计划对现行那一版的探针没人做 |
| Q3 「今天读最坏等 1.10 秒」 | 作为真卡数**不成立**；作为替身睡 50 毫秒一趟**成立** | 最接近真卡的睡 5 毫秒一行是 165487 微秒、睡 0 毫秒 130610 微秒，GPU 等只占差的约 35 毫秒；与 E185 的 5.3 毫秒量纲差 9.4 倍；补 10、20 毫秒两档与三趟（待主 agent 跑） |
| Q3 攻方模型的格 | 部分有效 | 失效：k、破例批恒在锁里、作废；有效：今天臂、登记窗、N 那一维；对象换了：新停因进 `keeps_going` 名单 |
| Q4 岔路单 | 十一行（见 §四·1） | 被写死而该是岔路的五条，站得住只有一个的三条（卸载等不等已定、读盘丙与用户定的第一条相左、读盘乙要块设备并发） |
| Q5 崩溃语义 | **推不出** | 读码推得出不新增写种类、批是今天那一批的真子集；缺：现有流罩不罩得到「旧版分类、新版发布、子集计划」那个组合的枚举计数 |
| Q6 换到的 | 约 35 毫秒（睡 5 毫秒档，推的算术）；卡挂住时不再挂前台 | 剩下约 110 到 131 毫秒的锁内 CPU 活拿不走 |
| Q6 多出来的 | 六个状态、八个分支、一到三个参数（§六·2） | 没有 k |
| Q6 用户定的两条在今天的 `crates/` 上 | 都做得成 | 要先做的三处：切换校验不取 GPU 把手锁（`background_compute.rs:1176`）、回锁重取隔离集并比整条记录、新停因进 `keeps_going`；盘表分锁、块设备并发、GPU 包里等的循环都不是这两条的先决 |

## 要主 agent 跑的命令

每项三样：命令、在哪个目录起、跑完看什么。本腿没有往任务服务交任何任务。

1. 建副本（主 agent 直接跑，不经服务；用完 `du -sh` 记大小再删）。在仓根起。看它列出的副本里有 `crates/singlefs-mount-runtime/tests/common_pool_on_memory_devices`：

```text
python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-sonnet/copy --with-test-inputs
```

2. 忙窗口基线补量（睡 0、5、10、20、50 毫秒 × 读线程四条 × 写线程 0 或 1，每个配置三趟，加 CPU 路径三趟；共 30 趟）。在仓根起；命令里都是绝对路径；范围声明闸要它时照 `.claude/hooks/run-scope-guard.sh` 文件头补。跑完看 `TASK_SERVER_RESULT` 末行 `exit=0`；`task-server/task log <号> --all` 里 30 行 `name=busy_window_foreground_wait`、一行 `name=self_check median_blocked_max_sleep0=… median_blocked_max_sleep50=…`（后者大于前者）、末行 `name=rerun_done busy_rows=30 expected_rows=30 failures=0`。判读见 §三·2 末段（同一档三趟的 `blocked_wait_micros_max` 极差、10 与 20 毫秒两档相对睡 0 毫秒的增量；睡 0 毫秒三趟中位数低于 40 毫秒则 §三·2 第 3 条与 §六·1 第 1 条作废）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-sonnet/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-sonnet-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-model
```

3. 模板自证（`tools/rerun.sh` 只换了 `rerun_steps`；假冻结副本没有运行库测试目录时 `rerun_steps` 打 `name=rerun_skipped` 并退 0，只验传参）。在仓根起；看末行 `TASK_SERVER_RESULT` 的 `exit=0` 与日志里 `✓ 自检通过`：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-model/tools/rerun.sh --selftest
```

4. 现成用例（一·1 的记录那一半：释放前读盘核没过时记录留已分配、分配代不变）。在仓根起；跑完看 `TASK_SERVER_RESULT` 末行 `type=` 与 `exit=0`，`task-server/task log <号>` 里 `test result: ok. 2 passed`：

```text
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 release_checksum_quarantine 里的 2 条测试; 范围=2 个测试; 为什么=核 Q1 那一格：释放前读盘核没过时记录留在已分配、分配代不变' task-server/task submit -- cargo test -p singlefs-verification --test release_checksum_quarantine -- checksum_failure_on_only_one_copy_keeps_both_records_allocated_and_the_pool_stays_writable_and_readable released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool --exact
```

5. 命名检查：本腿没改 `crates/`、`scripts/` 下的 `.rs`（探针源文件在 `research/prompts/…-model/` 下、是第二轮那份的原样拷贝），不适用。

## 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数（第 2 条的 30 趟、第 4 条的用例）没复跑，命令在上一节。
- 没写会话那一半的用例（隔离记号之后扫描判隔离、计划被整批拒）、没写「计划对现行那一版」的探针、没写分段计时、没在探针里记每次持锁里的 GPU 调用数、没读真盘块设备的并发实现、没读 `form_model.py` 逐行、没逐个读各前台改动变体的释放清单算法（只读到它们共用的 `publish_admitted`）。这些都写在各格的「缺」里。
- 没量过：写失败冻结的频率；锁内 CPU 各段的分段耗时；一次签发的选段圈数分布；真卡一次调用（两份核）的挂钟。读码推出来的都标了「推的，没量过」。
- 回退落盘与 discard 窗口的对账、GPU 连续不确定计数的键与清零：核不动，原因写在一·2、一·4。
- 没改任何 `crates/`、kb 与规则；没做 git 写操作。

## 引文核对末行

`python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m4-gpu-wait-outside-the-lock-design-r3-background.md` 末行：

```text
  ✓ 核了 53 处引文、106 处裸行号，对上 53 处，没判 106 处（其中裸行号没有引文可比 105 处；逐处列在上面）
```
