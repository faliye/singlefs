# m4-gpugc-code-r1 攻方腿（Opus）报告（2026-10-30）

轮名 m4-gpugc-code-r1；攻击面 T1–T6；立场照正文第四节「云端攻方（Opus）」那一行。前几轮判决：无。
冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r1/repo/`（下称「冻结副本」），代码行号都是冻结副本里的；kb 行号取冻结副本里那份 kb 文件自己的行号。
反例装置在仓副本（草稿目录 `~/.local/share/singlefs/work/m4-gpugc-code-r1-opus/` 下）上编、跑；副本上的数注明是副本，不算入库装置上的数。

## 各格判定一览

| 格 | 判定 | 打中的几条（详见各节） |
|---|---|---|
| T1 | 打中 3 条 | T1-1 重交与等的上限按「一段」算，一批之内超时、重交可以多于 3 次而不报警（量的）；T1-2 没点名的独显缺总线地址，点名别的卡也拒挂载（量的）；T1-3 点名虚拟显卡照「不存在」拒（量的，弱） |
| T2 | 没打中 | 三种形状都造不出（T2-1 到 T2-3）；旁记计数按落点认（推的） |
| T3 | 打中 1 条 | T3-1 参与的卡按 972 字节判进来，一批里一个落点两条条目让乙段最小一片涨到 1048，整理停下而不是不用这张卡（量的，纯函数层）；4156 字节那一次修的是账（T3-4） |
| T4 | 打中 2 条 | T4-1 删除续做那一支进得去（回退到 unlink 意图那一版），停着补跑时也走，步数不算进预算（量的）；T4-2 只供测试的补跑开关经公开入口够得着，调一次整个挂载期不补跑（按代码推） |
| T5 | 打中 2 条 | T5-1 卸载泄漏那一支测试进不去、运行时看不出；T5-2 重交第 1–3 次运行时看不出；T5-3 推的 |
| T6 | 打中 2 条 | T6-1 泄漏边界改成全部卡，库用例 119 条全绿（量的）；T6-2 补跑判定锁存，补跑用例 9 条全绿（量的）；重交改 2、合并少一片都红 |

我提的改法都只在我的模型上量过（多数没实现），被攻过零轮。

## 复跑

- 模型目录 `research/prompts/m4-gpugc-code-r1-opus-model/`：`probes/`（接进仓副本的探针用例原文）、`mutations/`（T6 变异的原文与替换文）、`tools/`（`apply-probes.sh`、`mutate-once.py`、`rerun.sh`）、`outputs/`（这一次跑出来的原样输出）。各文件 sha256 见文末「模型目录文件的 sha256」。
- 复跑命令（工作目录随意；`<新的草稿目录>` 给一个不在的目录）：
  `bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-gpugc-code-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-gpugc-code-r1/repo --work <新的草稿目录>`
  它拷冻结副本、接上探针、逐条跑本报告里的每一件（每件一条 `cargo test`，各带范围声明），输出落 `<新的草稿目录>`，与 `outputs/` 下同名文件比 `ATTACK` 行与 `test result` 行。
- 本报告里「量的」数全是仓副本（`~/.local/share/singlefs/work/m4-gpugc-code-r1-opus/repo`，冻结副本拷来、`find crates -type f -exec touch {} +` 之后编）上跑出来的，不算入库装置上的数。

## T1　过度外推与越位

### T1-1 打中：等的上限按「一次提交」算、重交按「一段」算，一批之内超时与重交可以多于 3 次而不报警

- 条款：「挂着的时候一批等 GPU 超过挂载参数 `background_gpu_wait_limit`，放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警、判那张卡用不了」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）；挂载参数表这一行写「一批交给 GPU 之后至多等多久」（`.claude/kb/mount-parameters.md:25`）。
- 代码：重交那一圈套在一段的提交上，「`for _resubmission in 0..RESUBMISSIONS_AFTER_A_TIMEOUT {`」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:152`）；三段各在切好的每一段（几片）上各调一次，「`outcome_resubmitted_after_timeouts(`」（`crates/singlefs-background-gpu/src/background_gpu.rs:1302`，另两处在第 1414、1535 行）；一次提交等的期限从这一次交起算，「`let deadline = GpuWaitDeadline::starting_now(self.budget.wait_limit(), &clock);`」（`crates/singlefs-background-gpu/src/background_gpu.rs:716`）。补丁块在 `m4-gpugc-adaptive-split-writer` 那一节（切片之后每段各重交）。
- 反例（量的，仓副本，`outputs/probes-lib.out`）：一批四个单元切成四段，每段头一次提交等超过上限、重交一次做完。原样输出：
  `ATTACK name=per_chunk_resubmission batch_ok=true submissions=8 waits_for_earlier=4 timeouts_in_the_batch=4 resubmissions_allowed_per_submission=3`
  这一批超时 4 次、重交 4 次，照常交回、不报警。按条款的「一批」读法，第 4 次超时之后就该报警停下。等的总时长同理：一批切成 K 段、每段只用到上限的九成，就一次都不超时，这一批合起来等了约 0.9·K 个上限（推的，按第 716 行每段各起一个期限推）。三段（甲乙丙）是三次方法调用，各自再乘一遍。
- 四问：① 不涉臂，分辨的是条款「一批」与代码「一段」两种读法；② 系统看得到：`both_kernel_outcomes_of_the_batch` 在一次调用里拿着整批的切法（`background_gpu.rs` 第 1250 行起），按批计数做得到；③ 满足的是第 4 条「放弃的那一批……至多 3 次」那一分句；④ 跑前没有条款给改法。
- 我提的改法（只在我的模型上量过、被攻过零轮；改法本身没实现，「推的」）：甲，在一张卡上按批记超时次数，跨段累计，到 3 次就按 `UnusableAfterResubmissions` 报；乙，期限按批起算，一批里各段共用一个期限。甲只修「次数」那一格，乙只修「时长」那一格。
- 推翻条件：条款的「一批」被定成「一次提交」（用户或 kb 改字），或补丁另有按批计数的地方我没读到（`grep -n 'resubmission' crates/singlefs-background-gpu/src/*.rs` 只在 `waiting_for_the_gpu.rs` 有计数）。

### T1-2 打中：没点名的一张独显报不出总线地址（或读不到显存堆、两张报同一地址），点名别的卡照样拒挂载

- 条款：「点名了就只用点名的卡」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`）；第 4 条拒挂的六样里没有「没点名的卡缺标识」。
- 代码：开包先对本机全部独显判清单，「`let discrete_gpus = discrete_gpus_of_the_enumeration(&descriptions)`」（`crates/singlefs-background-gpu/src/background_gpu.rs:298`），判点名在它之后，「`let participating = participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus)`」（`crates/singlefs-background-gpu/src/background_gpu.rs:300`）；清单那一判只看是不是独显，不看点没点名（`crates/singlefs-background-gpu/src/adapter_inventory.rs` 第 97 行起 `discrete_gpus_of_the_enumeration`）。
- 反例（量的，仓副本，`outputs/probes-lib.out`）：
  `ATTACK name=unnamed_card_refuses_the_list listed=Err(DiscreteGpuWithoutAPciBusAddressIsNotSupported { name: "unnamed card" })`
  `ATTACK name=same_selection_without_the_unnamed_card admitted_ok=true`
  同一组设置（只点 `0000:01:00.0`），本机多一张没点名、没报地址的独显，开包就拒；去掉那张卡照收。
- 四问：① 不涉臂；② 系统看得到点名（`selection` 在第 300 行之前就在手里）；③ 「点名了就只用点名的卡」那一分句，代码管的比条款多；④ 改法（推的）：给了 `background_gpu_devices` 时清单那一判只判点名的卡，没点名的缺地址照「没开成」跳过（同第 275 行注释里给了列表而没点名的卡开不成的处置）。

### T1-3 打中（弱）：点名一张虚拟显卡（或类型认不出的适配器）照「点名的卡不存在」拒

- 条款只说「集成显卡与软件 Vulkan 适配器不算「显卡」、不进「全部」」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`）；代码另把虚拟显卡与认不出类型的也排出去，「`| AdapterDeviceKind::VirtualGpu`」（`crates/singlefs-background-gpu/src/adapter_inventory.rs:46`），点名它们就拒。
- 反例（量的，仓副本）：`ATTACK name=naming_a_virtual_gpu refused=Err(NamedGpuDeviceNotInTheInventory { device: GpuDeviceIdentifier([48, 48, 48, 48, 58, 48, 50, 58, 48, 48, 46, 48]) })`
- 后果：直通进虚拟机、驱动报 `VirtualGpu` 的卡，不给列表时不参与、点名也拒，`gpu` 在这种机器上开不成。条款没写，代码第一版拒；算越位里「代码管的比条款多」，不算违约。

### T1-4 打不中（推的或按代码读）

- 阈值恰好 100、上限恰好等于显存总量：都照收，与条款同（「超过」）。代码 「`if memory_limit > total_memory {`」（`crates/singlefs-core/src/background_compute.rs:874`）、「`if gc_cpu_compensation_threshold.allocated_percent_of_the_capacity > WHOLE_CAPACITY_IN_PERCENT {`」（`crates/singlefs-core/src/background_compute.rs:736`）。
- 空卡列表、同一张卡点两次、选 `gpu` 时等的上限给 0：代码拒、条款没写，但挂载参数表已登记成「条款没写，代码第一版拒」与「选 `gpu` 时给 0，拒绝挂载或切换」（`.claude/kb/mount-parameters.md` 第 23、25 行），不算新打中。
- `gpu` 而没递实现：拒成 `GpuNotPresent`，对第 4 条「GPU 不在」，造不出不一致。
- 切换在一批之内生效：换上只在一批开头，「`self.in_effect.hand_the_gpu_settings_to_the_implementation();`」（`crates/singlefs-core/src/background_compute.rs:1335`），一批结束断言设置没换（同文件第 1383 行起）；钩子在一批中途请求的切换只记成待生效。试了钩子请求切换、出错之前挂着切换、停着时切换三种次序，都造不出一批之内换了设置。
- 停着时切换「当场」解除：请求那一刻就清，「`self.stopped_after_a_compute_error.clear();`」（`crates/singlefs-core/src/background_compute.rs:1307`）。造不出。
- Vulkan 以外的后端混进来：开包只建 Vulkan 实例，「`backends: wgpu::Backends::VULKAN,`」（`crates/singlefs-background-gpu/src/background_gpu.rs:287`）。造不出。
- 「那个后台任务停下」与「GC 补跑」分不分得开：补跑那一判按任务分支（`crates/singlefs-core/src/background_compute.rs` 第 1237–1246 行，只有 GC 一臂），停着的表按任务记。今天只有一种任务，分不分得开要等第二种任务进来才看得出（见 T5-3）。
- 条款拒而代码放过（推的，没卡跑不了）：切换时的卡清单是挂载那一刻的那一份，「`self.discrete_gpus_after_the_opening`」（`crates/singlefs-background-gpu/src/background_gpu.rs:1611`）；挂载之后掉了的卡，切换照样过校验，下一批开头才出错停下。另一形：挂载时因上限装不下而没量代价的卡，切换给它装得下的上限，换设置那一刻量代价出错（`background_gpu.rs` 第 1173–1178 行），切换已经照收，下一批报成 `CostNotMeasuredAtTheSwitch` 停下。第 4 条写的是「挂载或切换那一刻……GPU 不在、驱动不对……就拒绝挂载（或拒绝切换）」，这两形都成了「收下切换、下一批停下」。没 GPU 造不出可跑的反例，记推的。

## T3　哪一条最脆弱：出错与资源边界

### T3-1 打中：参与的卡在挂载与切换时按「最小的一片 972 字节」判进来，一批里一个落点上两条条目，乙段最小的一片涨到 1048 字节，这一批交成 `InputNotSupportedOnTheGpu`、整理停下

- 条款：「参与的卡的上限连最小的一片都装不下，这张卡就不用（第 2 条「参与的卡都用」的例外）：照开设备、不分活、报出来，挂载不因此拒绝」，同一条前面「装不下不拒绝」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`）。
- 代码：挂载与换设置只拿一个与输入无关的数判这张卡参不参与（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs` 第 396–400 行，972）；乙段自己承认最小的一片随映射变，「一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判」（`crates/singlefs-background-gpu/src/region_record_kernels.rs:663`）；逐批判出装不下时，交给核心层的是「`core_failure_of_the_smallest_piece_not_held(smallest_piece_not_held)`」（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:379`），它造的是「`InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {`」（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:336`），核心层照「一批算着出错」放弃这一批、整理停下。几张卡时，分到活的那一张切不出，整批就交回出错，别的卡不接（`background_gpu.rs` 第 1386 行起，先把各卡的切法都算完，有一张切不出就一张都不交）。
- 反例（量的，仓副本，`outputs/probes-lib.out`）：上限正好 972 字节的卡，一个落点一条条目切得出，两条条目切不出：
  `ATTACK name=batch_dependent_smallest_piece admitted_minimum=972 left_out_at_the_minimum=None one_entry_plan_ok=true two_entries_plan=Some(SmallestPieceLargerThanTheRoom { entry_position: 0, smallest_piece_bytes: 1048 })`
- 可达性：反例在纯函数层；真池里一个落点上两条中央映射条目怎么来，我没在池上造出来（推的：映射坏了、或别的路径让两把 key 指同一槽时才有；区域扫描把这种记录分成「不一致」，见本模块文件头第 7 行）。所以这一格是「条款要不用这张卡、代码停 GC」的形状成立，真池上的取样没做。
- 四问：① 不涉臂；② 系统看得到：切法在交活之前算完，「一张卡都还没交活」（`background_gpu.rs` 第 180 行注释）；③ 满足第 3 条「这张卡就不用……挂载不因此拒绝」与「装不下不拒绝」两分句（停 GC 比拒挂载更重）；④ 改法（推的）：甲，逐批判出装不下的卡这一批不分活、活分给别的卡，全都装不下才报 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`；乙，挂载那一刻的最小一片照「一个落点上最多几条」的上界取。甲修「多卡时别的卡接活」，单卡时仍停；乙只在上界定得出来时修。

### T3-2 打不中：「重交 3 次之内卡回来了而代码仍判卡用不了」

- 试了三种时序（按 `waiting_for_the_gpu.rs` 第 142–164 行与现有用例推，替身已在 `waiting_for_the_gpu.rs` 的用例里覆盖）：前一次在第 k 轮等前一次时做完 ⇒ 接着重交、做完照常交回；前一次一直不做完 ⇒ 三轮都花在等前一次上，一次都没重交，报 `UnusableAfterResubmissions { resubmissions: 3 }`；每次都做完但都慢于上限 ⇒ 四次都超时、报警。第二种里卡没回来，报警合条款；只是报的「重交 3 次」实际重交 0 次（见 T5-2）。造不出卡回来了还判用不了。

### T3-3 打不中：先前超时的提交回来之后账算错

- 超时那一段的字节记成还在跑的那次提交，做完才放，「`self.release(completed.bytes_held_by_its_chunk);`」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:404`）；切下一批之前先等它（第 413 行起），再按上限扣掉还占着的切。试了「超时之后换小上限」：先等再扣，扣的是 0。造不出。
- 没量的：wgpu 建缓冲时自己另开的暂存（`create_buffer_init` 那一条路）记不记在账上，我没 GPU、没读 wgpu 源码，记「没核」。

### T3-4 「4156 字节超过上限 500 字节」那一次红：修的是账（量代价那一段改走切片），不是用例

- 出处：「5822 的 4156 字节是挂载那一刻量代价那一段（校验和段，4096 + 12 + 4 × 8 + 16），没经切批直接记账」（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:541`）。第 48 批（补丁 `m4-gpugc-gpu-reds-investigator`）先改成装不下就拒开卡，第 49 批（补丁 `m4-gpugc-adaptive-split-writer`）再改成照上限切片量，「`.pieces_under_the_bounds(self.budget.chunk_bounds())`」（`crates/singlefs-background-gpu/src/background_gpu.rs:611`）。两次改的都是产品代码的记账路径。

### T3-5 卸载时的泄漏（推的，没卡跑不了）

- 泄漏的边界：只看这张卡上记没记着一次等超过上限、没做完的提交，「`self.submission_running_past_its_wait.is_some()`」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:308`），记着就整张不析构，「`std::mem::forget(gpu);`」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:556`）。条款「有这种提交的那张卡整张泄漏」，边界对得上。
- 泄漏的量：设备、队列、六条管线，加上那一段的缓冲。给了显存上限时那一段不超过上限；没给上限时那一段只受绑定上限管（`background_gpu.rs` 第 96 行，2^31 字节），一次泄漏可以到 2 GiB 显存（推的，按切段的界推）。同一进程里挂了卸、卸了挂，卡一直卡着，每一轮都再泄漏一张设备，没有上限（推的）。
- 再挂一次拿不拿得到卡：新挂载开一个新的 wgpu 实例、在同一张物理卡上再起一个逻辑设备（Vulkan 许可）；卡若真卡死，量代价那几次提交超时，照第 4 条「算那张卡没开成」，点了名就拒挂。没卡，没跑，不知道驱动在一张卡上已有一个忙着的逻辑设备时 `request_device` 会不会自己卡住（那一步也在等的上限之内，`background_gpu.rs` 第 377–389 行，超时报出错）。造不出「拿不到卡且不报错」的形状。

## T4　CPU 一侧：补跑、挪线程与两处快写法

### T4-1 打中：第 3238 行「删除续做在整理批之前」那一支进得去（主 agent 的第 ② 处），停着补跑时它也走；它发的步数不算进这一趟的发布预算

- 代码：停着而不补跑先在第 3207 行返回，「`if compaction_stays_stopped_pausing_the_live_intent(allocator, background_compute) {`」（`crates/singlefs-core/src/mounted_session.rs:3207`）；补跑时往下走到「`if !queue_is_empty {`」（`crates/singlefs-core/src/mounted_session.rs:3238`），走空队列；预算计数在它之后才起，「`let mut publishes_of_the_run: u64 = 0;`」（`crates/singlefs-core/src/mounted_session.rs:3274`）。
- 进入路径（主 agent 说的两条都断，第三条不断）：会话里 unlink 一次走完、挂载续做在建后台计算之前走空，这两条成立；但挂着时回退到 unlink 意图那一版，队列随那一版回来，会话的 `current` 与 `allocator` 是公开字段（`crates/singlefs-core/src/mounted_session.rs` 第 106、107 行），回退之后直接跑整理就进这一支。回退到意图那一版是仓里已有用例走过的路（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` 第 1112 行起那一条用例的做法，「回退带回队列」）。
- 反例（量的，仓副本，`outputs/deletion-continuation.out`，用例 `probes/harness_deletion_continuation_test.rs`）：池同 `compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs`，挂 `gpu`、签意图、注入甲段出错停下 → unlink 第一个文件 → 回退到 unlink 意图那一版 → 显式入口推一趟（预算压到 1）。原样输出：
  `ATTACK name=rolled_back_queue tag=attack-deletion-continuation-compensating deleted_inodes_after_the_rollback=[InodeNumber(1)] stopped=1`
  `ATTACK name=compensating_run_after_rollback outcome=Stopped(PublishBudgetSpent { budget: 1 }) versions_published=2 txg_before=12 txgs_published=[13, 14] deleted_inodes_after_the_run=[] batches_recorded=1`
  `ATTACK name=left_stopped_run_after_rollback outcome=Stopped(BackgroundTaskStoppedAfterAnEarlierComputeError) versions_published=0 deleted_inodes_after_the_run=[InodeNumber(1)]`
  停着且超过阈值（0%）：这一趟先走删除续做（一步）、再补跑一批，共发两版，预算是 1；停着且不超过（默认 85%）：一个写都不发、队列照旧在。后一半与条款「都不超过就照旧停着」一致，「一个写都不发」没违约。
- 四问：① 不涉臂；② 系统看得到队列不空（第 3232–3237 行就判它）；③ 两处：主 agent 问的「是不是死代码」——不是；第 6 条「一趟发布预算有上限」（`.claude/kb/decisions/26-后台整理与放置回收.md:250`）——删除续做那几步不在 `publishes_of_the_run` 里，一趟实发的版数可以多于预算，多多少看队列里有几步（这个反例多 1；文件大、删除批小时多得多，推的）。这两步算不算「整理的发布」条款没写，判归由主 agent 定；④ 跑前没有条款给改法。我提的改法（推的、被攻过零轮）：删除续做的步数也记进这一趟的预算，或删除续做撞到预算就停、留给下一趟。

### T4-2 打中（主 agent 第 ① 处的后果）：只供测试的补跑开关在发布构建里够得着，调过一次之后整个挂载期补跑都关着，切换也解不开

- 够得着的路：`pub fn run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch`（`crates/singlefs-core/src/mounted_session.rs` 第 2193 行起，没有 `cfg(test)`）把 `&mut BackgroundComputeOfThisMount` 交给调用方的钩子，钩子里调 `pub fn judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests`（`crates/singlefs-core/src/background_compute.rs:1142`）或 `report_a_compute_error_in_the_batch_under_way_for_tests`（同文件第 1163 行）。
- 后果（按代码推，没另写用例）：前者把判法改掉，「`self.gc_cpu_compensation_judgement =`」（`crates/singlefs-core/src/background_compute.rs:1143`）；切换（第 1287–1309 行）与一批开头（第 1324–1369 行）都不碰这个字段，这次挂载剩下的时间里 GC 每次因 GPU 出错停着，最满的盘过了阈值也不补跑，空间只减不增，只有重挂能清。后者让当批照「算着出错」走、整理停下，报的错是 `InjectedByTheTestOnlyHook`。
- 四问：② 系统看不出调用方是测试还是产品；③ 第 4 条「任一块盘……超过……这一批在 CPU 上补跑」；④ 改法（推的）：两个开关与带钩子的入口加 `#[cfg(any(test, feature = "…"))]`，或开关随切换复位。

### T4-3 打不中：补跑读数取到旧值

- 每一批开头现读分配器的计数，「`AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator(allocator),`」（`crates/singlefs-core/src/mounted_session.rs:3646`），一趟开头另读一次（同文件第 3104 行）；计数是分配器增量维护的此刻值（含 defer 窗口里已释放的，`background_compute.rs` 第 527–529 行注释），不是上一次发布的。造不出读到旧值的时序。用例罩不罩得住「读旧值」见 T6-2。

### T4-4 第 ⑦ 处：拆成「判起跑」「跑」两段之后三样保证由谁守（按代码读，没造反例）

- 「整理批不插进用户改动的抬 F 与再判之间」：`publish_user_change` 整次调用拿着 `&mut self`（`mounted_session.rs` 第 2123–2126 行注释），攒批的会话在后台入口里先排空待发、再跑（第 5471 行起），排空那一次发布与之后的整理在同一个 `&mut self` 调用里串着，插不进。
- 「用户改动返回之前一批整理都不发」：攒批会话的 `accept` 只收不发，整理只在后台入口里跑。造不出。
- 「签下一条意图那一判跨趟留着」：`last_completed_intent` 住会话，用户改动发成时清（第 2132 行），会话内回滚经 `roll_back_by_a_forward_publish_in_the_session` 清（`mount.rs` 第 10324 行起）。注意：T4-1 那条路用的是不清它的 `roll_back_by_a_forward_publish`，两个回滚入口都公开，清不清看调用方挑哪一个（推的风险，没造反例）。
- 「前台 fsync 在等时不发下一批」：每一批之前问调用方（第 3298 行起）。造不出。

## T2　内部矛盾与循环依赖

### T2-1 打不中：切片前后两份核的比对结果不同

- 甲段：一个单元切成几片时，两份核各自的片值各按自己的合法合成整单元（`crates/singlefs-background-gpu/src/checksum_kernels.rs` 第 441–461 行），合成之后才逐条比，比的单位是单元不是片。乙段：头一片之后的几片先交、各核从自己上一片算完的寄存器接着算（`region_record_kernels.rs` 文件头第 40 行起），两份寄存器分开带（`CrcRegistersOfTheTwoKernels`）。丙段按段一条，不切条目。试了「片边界落在 628 字节内外」「最后一片只有 1 字节」两种取样点（按代码读，没另写用例；现有用例 `units_cut_into_pieces_under_a_small_memory_limit_match_the_cpu_reference_on_every_unit` 用 20000 字节载荷），造不出切与不切比对结果不同。
- 「少带一片」的改坏现有用例抓得到（T6-3）。

### T2-2 打不中：`failure_under_the_settings_handed_in` 在挂载与切换两处读到不同状态

- 两处走同一个函数 `tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in`（`crates/singlefs-core/src/background_compute.rs` 第 1041 行起），挂载在第 1125–1129 行、切换生效在第 1335–1337 行，都是「先交设置、紧接着读」，中间没有别的调用，读的是同一份实现在同一组设置下的状态。GPU 包那一侧在 `use_the_settings_from_the_next_batch` 里同步算完参与的卡与晾下的卡（`background_gpu.rs` 第 1138–1186 行），没有错开一批。造不出。

### T2-3 打不中：「正在算的那一批放弃」与回滚清意图、`last_completed_intent` 的次序

- 出错放弃时撤 R 排他、意图留盘（`mounted_session.rs` 第 3646 行那一处之后的出错分支），`last_completed_intent` 只在一条意图做完时记、用户改动发成与会话内回滚时清。试了「出错停下之后回滚」「回滚之后补跑」两种次序（后一种就是 T4-1 的反例，补跑照常推完那一批），没造出次序错乱。
- 旁证（推的）：连续不确定计数按（盘、槽、跨几槽）记（`background_compute.rs` 第 99–103 行），槽位释放又被别的单元以同样跨度占用时，计数接着上一个单元数，那一个新单元可能头一轮不确定就交 CPU 判。方向是早交 CPU，不漏判；条款说「同一个条目」，这里的「同一个」按落点认，记一笔。

## T5　五条硬要求与可观测

### T5-1 打中：卸载时「泄漏哪几张卡」那一支，测试强制不进去、运行时也看不出走了哪条

- 泄漏了几张交回给一个丢掉的绑定，「`let _gpus_leaked =`」（`crates/singlefs-background-gpu/src/background_gpu.rs:1596`）；`Drop` 只在真开了卡的 `BackgroundGpu` 上跑，库用例不开卡，一次都不进。T6-1 把判据改成「全部泄漏」，库用例 119 条全绿（量的）。真卡用例也读不到泄漏了几张（交回值丢了），所以真卡上同样看不出。`fs-design.md`「五条硬要求」的「每条分支能被测试强制进入」「可观测」两条都不满足。

### T5-2 打中：重交第 1–3 次那几支运行时看不出走了哪条

- 一批的记账只有三段各自的确认、不确定、交 CPU 判与甲段对不上的单元数（`crates/singlefs-core/src/background_compute.rs` 第 422–429 行），重交成功的那一批与一次就做完的那一批记账逐字段相同；报警时报的 `resubmissions` 恒是 3（`waiting_for_the_gpu.rs` 第 160–163 行），三轮都卡在「等前一次」、实际一次都没重交时也报 3（现有用例 `a_resubmission_whose_wait_for_the_earlier_submission_times_out_counts_as_failed_without_submitting` 钉的就是「等不到算这一次重交」）。用例能强制进入（替身脚本化），运行时看不出。

### T5-3 推的：`BACKGROUND_TASKS_WITH_A_GPU_PATH` 加一个任务而别处不改

- `BackgroundTask` 加一个成员，`route_of_the_task`（第 973 行起）与补跑那一判（第 1237 行起）的 `match` 编不过、逼着改；写死长度 1 的数组（第 1035–1036 行）不逼。新任务照 `route_of_the_task` 走 GPU，而挂载或切换时 GPU 用不上，它不在「当场停着」的表里，要等它自己的头一批出错才停。哪条用例会红：我在仓副本里没加成员试编（加一个成员要改的 `match` 散在几个包里，这一轮没做），按 `grep -rn 'BACKGROUND_TASKS_WITH_A_GPU_PATH' crates` 只命中 `background_compute.rs` 两处、没有一条用例点名它推：一条都不会红。记推的。

## T6　测试与变异判的是不是那件事

| 编号 | 改坏的是什么 | 改的哪一行 | 跑了哪些用例 | 结果 | 量的 / 推的 |
|---|---|---|---|---|---|
| T6-1 | 泄漏的边界改成全部卡 | `background_gpu.rs` 第 1597–1602 行的判据换成 `|_gpu_in_use| true`（`mutations/drop-leaks-every-card.*`） | `cargo test -p singlefs-background-gpu --lib`（仓副本） | `test result: ok. 119 passed; 0 failed`（`outputs/mutation-drop-leaks-every-card.out`），没红 | 量的 |
| T6-2 | 补跑读数用旧的那一判：停下以来补跑过一批，之后不看读数照补 | `background_compute.rs` 第 1240 行前插一判（`mutations/compensation-latched-after-a-compensating-batch.*`） | `cargo test -p singlefs-harness --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure`（仓副本） | `test result: ok. 9 passed; 0 failed`（`outputs/mutation-compensation-latched.out`），没红 | 量的 |
| T6-3 | 切片合并少带最后一片 | `checksum_kernels.rs` 第 443 行 `[1..]` 改成 `[1..len - 1]` | `cargo test -p singlefs-background-gpu --lib`（仓副本） | `test result: FAILED. 117 passed; 1 failed`，红在 `units_cut_into_pieces_under_a_small_memory_limit_match_the_cpu_reference_on_every_unit`（`outputs/mutation-combine-drops-the-last-piece.out`） | 量的 |
| T6-4 | 重交次数改 2 | `waiting_for_the_gpu.rs` 第 117 行 | `cargo test -p singlefs-background-gpu --lib`（仓副本） | `test result: FAILED. 117 passed; 2 failed`（`outputs/mutation-resubmissions-two.out`） | 量的 |
| T6-5 | 两份核比对按片不按条目 | 没改：甲段合成之后才比、乙段按记录出判定，找不到一处「按片比」的写法可改 | — | — | 推的 |

- T6-1 红不了的原因：`Drop` 在库用例里一次都不跑（见 T5-1），`crates/mutations.tsv` 里那一行（名字「丢后台 GPU 时还在跑的那张卡照样丢」）改的是 `gpus_of_this_host.rs` 的泛型函数，不是 `Drop` 里给它的判据。
- T6-2 红不了的原因：用例里「补跑途中降到不超过、下一批起照旧停着」那一格经只供测试的开关强制进入（`compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs` 文件头第 13 行起），开关在判法里排在读数前面（`background_compute.rs` 第 1231–1236 行），读数真降下来的那一格没有用例。条款「补跑途中降到不超过，下一批起照旧停着」因此没有判别力。只跑了这一个目标；`compaction_batches_on_the_gpu_path_through_the_background_gpu_compute` 与 `singlefs-core` 库用例没跑，它们抓不抓得到没量（推的：两处都不造「补跑过、读数又降下来」的池）。
- 总数说明：T6-1、T6-4 两次跑时仓副本带着三条探针（119 条）；T6-3 那一次之前 `waiting_for_the_gpu.rs` 已从冻结副本拷回、少了那一条探针，总数 118。

## 主 agent 记下的八处，各自的去向

| 处 | 去向 |
|---|---|
| ① 只供测试的补跑开关在发布构建里够得着 | 后果造出来了（按代码推，T4-2）：经公开的带钩子入口，一次调用让整个挂载期 GC 停着时不补跑，切换解不开 |
| ② 第 3230 行起删除续做那一支 | 不是死代码（量的，T4-1）：回退到 unlink 意图那一版之后进得去；停着补跑时也走，它的步数不算进预算 |
| ③ `BACKGROUND_TASKS_WITH_A_GPU_PATH` 写死长度 1 | 推的（T5-3）：加成员时编译器不逼、用例一条都不红；没在副本上加成员试编 |
| ④ 卸载时泄漏的边界 | 边界对得上「还记着一次超时提交」（T3-5）；但这一支用例进不去、运行时看不出（T5-1），改成「全部泄漏」库用例全绿（T6-1） |
| ⑤ 重交 3 次同一张卡同一批 | 同卡对；「同一批」没对上：重交按段，一批之内可以超时、重交多于 3 次（量的，T1-1） |
| ⑥ 补跑按盘、读最满那一块 | 读的是分配器此刻值（T4-3，造不出旧值）；但「降到不超过就停」那一格用例只经测试开关进，锁存的改坏不红（量的，T6-2） |
| ⑦ 拆成两段之后的两样保证 | 按代码读仍由 `&mut self` 守着（T4-4），没造出插队的时序 |
| ⑧ 第 45、46 批的读法 | 「至少装得下 972 字节才参与」与乙段逐批的最小一片对不上（量的，T3-1）；「一样满取排前面」照代码第 566 行，造不出不一致 |

## 没打中的形状

- T1：阈值 100、上限等于显存总量、`gpu` 而没递实现、一批之内切换生效、停着切换当场解除、Vulkan 以外的后端（T1-4，各按代码读或现有用例）。
- T2：切片前后比对不同、挂载与切换读到不同状态、出错放弃与回滚的次序（T2-1 到 T2-3）。
- T3：重交 3 次之内卡回来了仍判用不了（三种时序，T3-2）；超时提交回来之后账算错（T3-3）；泄漏的卡让再挂一次拿不到卡而不报错（T3-5，没卡）。
- T4：补跑读到旧值（T4-3）；拆两段之后整理批插进抬 F 与再判之间、用户改动返回之前发整理（T4-4）。`region_scan.rs` 按身份查与线性找、`checksum.rs` 硬件 CRC 与软件写法的对拍与强制分支：这一轮没读到，没攻。
- T6：重交次数改 2、切片合并少带一片，现有用例都红（T6-3、T6-4）；比对按片不按条目找不到可改的写法（T6-5）。
- 取样范围：探针各取一个点（4 段各超时一次；一张没点名缺地址的卡；一个落点两条条目、上限 972；回退到意图那一版之后推一趟、阈值 0% 与 85% 各一次）。没扫段数、卡数、条目数、阈值。

## 这条腿自己的限度

- 没 GPU：真卡用例一条没跑；T1-4 末一条、T3-3 的 wgpu 暂存、T3-5 全是推的。
- T3-1 的反例在纯函数层，真池里「一个落点两条映射条目」怎么来没造出。
- T6-2 只跑了一个测试目标；别的目标抓不抓得到没量。
- T5-3 没在副本上加成员试编。
- 这一轮没读 `region_scan.rs`、`checksum.rs`、`live_slot_kernels.rs`、`work_split_by_measured_cost.rs` 与两份 WGSL，T4 的两处快写法与丙段没攻。
- 负载：开跑前 `ps` 看到别的会话的 checker 档 `cargo test --release` 八条与一条 harness 用例在跑，没有性能测量；资源提醒说本机内存紧、第二台空。没走第二台，本机加 `nice -n 19` 跑；第一次跑 harness 探针在内存包装里排队 6 分钟没排上，我停了自己那一条的排队进程（`proc.py stop`），把上限从 16G 降到 3G 重起，排上就跑完。

## 撤回

- 先提「会话内回滚那个清 `last_completed_intent` 的入口没有调用方」：现查 `crates/singlefs-harness/src/history.rs` 第 8066 行与 `compaction_rollback_in_the_session_clears_the_last_completed_intent.rs` 都调它，撤回；只留「两个回滚入口都公开、清不清看调用方挑哪一个」（T4-4，推的）。
- 先提「补跑读数取旧值」能直接造时序：读数每批现取分配器（T4-3），撤回；改成「用例罩不住锁存那一种改坏」（T6-2）。
- 先想「重交 3 次之内卡回来了仍判用不了」能用「等前一次」那一轮的上限造出：三种时序都不成（T3-2），撤回。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没起 GPU、没跑真卡用例、没起门禁阶段、没跑 checker 档测试。
- 改法都没实现、没跑。

## 模型目录文件的 sha256

```text
a5958c34a963d11f5e33d59424603e32569ce0d2422ce6cdbcb59e397499c6f0  ./mutations/combine-drops-the-last-piece.original
9886422caef68f555a564cdceedfea59a069f41c4eaf6664d60b15c3f6bd8500  ./mutations/combine-drops-the-last-piece.replacement
9cad31861f6acaef08350df68ec7624cdad5d41cb07a4bc1feb9e63f69cd9a26  ./mutations/compensation-latched-after-a-compensating-batch.original
8bf03e27d5f5a2a50891277b31a68cb7f23d5fc8851979fd8cd477952c64387b  ./mutations/compensation-latched-after-a-compensating-batch.replacement
ea5f92b2143b15c1d89b81a294585bee461610d2ca5d43c29511be4840e562f0  ./mutations/drop-leaks-every-card.original
5a9efa072f69fa65f75b80bef5c301b562499fae10189d9873e83e4bcd8be6ca  ./mutations/drop-leaks-every-card.replacement
eecb052c6963467566f7a0c9b092e530d2acd70a18577b46c61ae3d4ed593763  ./mutations/resubmissions-two.original
013986aea005120a80a37e6078f94cdc7e2664f64b50a85df0f15ceb5267812b  ./mutations/resubmissions-two.replacement
dc36038aeeb84e63941e5b59ebdd21c14d316cbb18919c01b50ee9705c37784c  ./outputs/deletion-continuation.out
3f95ed41c6ec024cc38630f944f63e0a2319a894981fbf2d074b586ea193b004  ./outputs/mutation-combine-drops-the-last-piece.out
5e7875271bf37be85ee2f9befe46d3bbad06ff87189787810c45b4157dcf6838  ./outputs/mutation-compensation-latched.out
7f1f5cafded8961da6e6644e5c0459bab753b38179fb19e4e59049d5e9327062  ./outputs/mutation-drop-leaks-every-card.out
e702a02b3a3d0ad3dcc98a1146da77339d21f0f73b7b94f2e29b32ec60795568  ./outputs/mutation-resubmissions-two.out
eb5c008742f422acbe5d535858a530251db74965177a860cb7721c7211847a00  ./outputs/probes-lib.out
cc3b7ba121733b10f26be370fd30a25134cfc84b1bc3fabcf277ef75c722fa3f  ./probes/adapter_inventory_probe.rs
5538261a35ef8e978649c66a45e5be4d8b9092d8b7c360214de95a42f998c1d8  ./probes/harness_deletion_continuation_test.rs
20946da74d9a1bf5e9334c8b5fe76232ad0912317541bdeea384bb6e5d117117  ./probes/region_record_kernels_probe.rs
5b51db025a7b7c90f24e53a19b1c55fb582b492f6466d536e98244c63c17810c  ./probes/waiting_for_the_gpu_probe.rs
92592626a8aebfb8d61f2b35110504e99c770614fb400d8e4234ef053d392cd9  ./tools/apply-probes.sh
87d86d131a1956a076ebd4934f926a66ab4c128fd845de70609a263feefdc61b  ./tools/mutate-once.py
c733104b3da86b1e69182b44e69d5e8ad2675906e7f5f44ef42b741f85f36552  ./tools/rerun.sh
```

## 交回前的整份复跑

在新起的一条 Bash 里跑（编译目录借我草稿目录里那一份，`RERUN_CARGO_TARGET_DIR` 指过去，省掉第三方依赖的重编；不设它就用 `<草稿目录>/target` 从头编）：
`RERUN_CARGO_TARGET_DIR=~/.local/share/singlefs/work/m4-gpugc-code-r1-opus/target bash scripts/hosts/run-with-memory-cap.sh 12G bash research/prompts/m4-gpugc-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-gpugc-code-r1/repo --work ~/.local/share/singlefs/work/m4-gpugc-code-r1-opus/rerun-check`
退 0，末尾原样输出：

```text
name=probes-lib rc=0
name=mutation-drop-leaks-every-card rc=0
name=mutation-resubmissions-two rc=101
name=mutation-combine-drops-the-last-piece rc=101
name=deletion-continuation rc=0
name=mutation-compensation-latched rc=0
name=compare-probes-lib same=yes
name=compare-mutation-drop-leaks-every-card same=yes
name=compare-mutation-resubmissions-two same=yes
name=compare-mutation-combine-drops-the-last-piece same=yes
name=compare-deletion-continuation same=yes
name=compare-mutation-compensation-latched same=yes
name=compare-total differing=0
```

两条 `rc=101` 是那两个变异被现有用例抓到（T6-3、T6-4），是预期的红；比对按 `ATTACK` 行与 `test result` 行（去掉耗时、排序）逐行比，六份都一样。

## 清理

交回前删了草稿目录里自己建的三样：仓副本 `repo/`（44M）、编译目录 `target/`（3.9G）、复跑用的 `rerun-check/`（44M，里面是又一份仓副本与复跑输出）。草稿目录里留下的只有几份文本日志（共 104K）。
