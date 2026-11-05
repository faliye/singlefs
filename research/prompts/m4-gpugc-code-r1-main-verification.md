# m4-gpugc-code-r1：里程碑四七·3 GPU 垃圾回收，代码轮第一轮判决（2026-10-30）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 -->

被判的产品代码：七·3 第 39–51 批 18 份补丁合入之后的 28 份文件——`crates/singlefs-background-gpu/src/` 下的 `adapter_inventory.rs`、`background_gpu.rs`、`background_gpu_compute.rs`、`checksum_kernels.rs`、`checksum_kernel_bitwise.wgsl`、`checksum_kernel_table_driven.wgsl`、`gpus_of_this_host.rs`、`lib.rs`、`live_slot_kernels.rs`、`live_slot_kernel_searching_sorted_records.wgsl`、`live_slot_kernel_walking_every_record.wgsl`、`memory_limit_and_chunks.rs`、`region_record_kernels.rs`、`region_record_kernel_searching_sorted_indexes.wgsl`、`region_record_kernel_walking_the_mapping.wgsl`、`two_kernel_reconciliation.rs`、`waiting_for_the_gpu.rs`、`work_split_by_measured_cost.rs`，与 `crates/singlefs-core/src/` 下的 `admission.rs`、`background_compute.rs`、`block_device.rs`、`checksum.rs`、`compaction_plan.rs`、`lib.rs`、`mount.rs`、`mounted_session.rs`、`region_scan.rs`、`transaction.rs`（改动原文 `research/prompts/_m4-gpugc-code-r1-diff.md`）。

开工快照：`research/prompts/m4-gpugc-code-r1-snapshot/`；冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r1/repo/`。

## 一、核查

| 核了什么 | 怎么核 | 结果 |
|---|---|---|
| 开工快照倒推 | `python3 scripts/agents/snapshot-tree.py m4-gpugc-code-r1 --out /tmp/claude-1000/-home-fy5090-code-singlefs/9980e6a0-efda-4d5d-b068-0b37cb82a711/scratchpad/check/m4-gpugc-code-r1-tree` | 32 份里倒推出 31 份、sha256 全对上；`.claude/kb/checks-owed.md` 倒推不出（快照之后别的会话提交改过它，diff 里没有它的基底）。冻结副本里那一份 sha256 `95949cd5d928…` 与快照相同，拷进树之后 `sha256sum -c` 32 份全对上 |
| 两条腿的引文 | `python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-code-r1-sonnet-output.md research/prompts/m4-gpugc-code-r1-opus-output.md --root /tmp/claude-1000/-home-fy5090-code-singlefs/9980e6a0-efda-4d5d-b068-0b37cb82a711/scratchpad/check/m4-gpugc-code-r1-tree --background research/prompts/_m4-gpugc-code-r1-background.md` | 末行「✓ 核了 53 处引文、239 处裸行号，对上 50 处，没判 242 处（其中裸行号没有引文可比 212 处；逐处列在上面）」，对不上 0 处。下面第二节用到的行，主 agent 在冻结副本里逐处读过（列在各格「主 agent 读码」里） |
| 攻方的复跑数 | 模型目录拷到 `~/.local/share/singlefs/work/m4-gpugc-code-r1-main-check/opus-model/`（22 份 sha256 与攻方报告文末那张表逐个相同），`bash scripts/hosts/run-with-memory-cap.sh 16G bash <那份>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-gpugc-code-r1/repo --work ~/.local/share/singlefs/work/m4-gpugc-code-r1-main-check/rerun --model <那份>`，不借腿的编译目录 | 退 0，六件与攻方 `outputs/` 逐行比「name=compare-total differing=0」。原样：「ATTACK name=per_chunk_resubmission batch_ok=true submissions=8 waits_for_earlier=4 timeouts_in_the_batch=4 resubmissions_allowed_per_submission=3」「ATTACK name=unnamed_card_refuses_the_list listed=Err(DiscreteGpuWithoutAPciBusAddressIsNotSupported { name: "unnamed card" })」「ATTACK name=batch_dependent_smallest_piece admitted_minimum=972 left_out_at_the_minimum=None one_entry_plan_ok=true two_entries_plan=Some(SmallestPieceLargerThanTheRoom { entry_position: 0, smallest_piece_bytes: 1048 })」「ATTACK name=compensating_run_after_rollback outcome=Stopped(PublishBudgetSpent { budget: 1 }) versions_published=2 txg_before=12 txgs_published=[13, 14] deleted_inodes_after_the_run=[] batches_recorded=1」「ATTACK name=left_stopped_run_after_rollback outcome=Stopped(BackgroundTaskStoppedAfterAnEarlierComputeError) versions_published=0 deleted_inodes_after_the_run=[InodeNumber(1)]」；泄漏边界改成全部卡「test result: ok. 119 passed; 0 failed」，补跑判定锁存「test result: ok. 9 passed; 0 failed」，重交次数改 2「test result: FAILED. 117 passed; 2 failed」，切片合并少带最后一片「test result: FAILED. 117 passed; 1 failed」 |
| 正推的「永远取第一块盘」变异 | 在复跑那棵副本树上照正推 T6.3 把 `crates/singlefs-core/src/background_compute.rs` 第 566 行 `Some(fullest) if !device.is_fuller_than(&fullest) => Some(fullest),` 换成 `Some(fullest) => Some(fullest),`，跑 `cargo test -p singlefs-harness --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure` | 「test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out」，没红（输出 `~/.local/share/singlefs/work/m4-gpugc-code-r1-main-check/variant-a.out`）；跑完源文件从冻结副本拷回 |

## 二、判定与去向

| 格 | 正推 | 攻方 | 判定 |
|---|---|---|---|
| T1 越位 | 拒的 13 支逐支对过；条款写了而代码放过：切换只读挂载那一刻的卡清单 | 打中 3 条：重交按片计数、没点名的卡缺地址也拒挂、点名虚拟显卡照不存在拒 | 见下面 T1 |
| T2 两侧约定 | 对照表一致；`TablesLargerThanABindingOnTheDevice` GPU 包不产、`input_bytes` 两侧含义不一；批内多段放弃时乙段计数先推进 | 没打中（三种形状造不出） | 见下面 T2 |
| T3 出错与资源边界 | 交活前先等前一次超时提交那一等一窗就报警；空闲显存不到上限照常跑没有实现；跨批攒没有实现；泄漏后再挂的账不扣泄漏的那片 | 打中 1 条：一批里最小一片随映射变大时整理停下，而不是不用这张卡 | 见下面 T3 |
| T4 CPU 侧 | 删除续做那一支不是死代码；切换到一张都用不上的设置时停着晚一步；只供测试的开关够得着、后果限于自己的会话 | 打中 2 条：删除续做那一支进得去、补跑时步数不进预算；只供测试的补跑开关够得着 | 见下面 T4 |
| T5 五条硬要求 | 切片与否、重交几次、这一批用了哪几张卡运行时都看不出；停着表写死长度 1 没有测试会红 | 打中 2 条：卸载泄漏测试进不去、看不出；重交第几次看不出 | 见下面 T5 |
| T6 测试与变异 | 「永远取第一块盘」补跑用例全绿；第 6104 行用例名写了「继续删除队列」而没造队列；第 6063 行红在装置前置断言；正文行号写错 | 打中 2 条：泄漏边界改成全部卡库用例全绿；补跑判定锁存补跑用例全绿 | 见下面 T6 |

### T1

- **重交按片计数**（攻方 T1-1、正推 T3.1 第 3 行）：**打中，坐实**。主 agent 读码：`crates/singlefs-background-gpu/src/background_gpu.rs` 第 1293–1306 行，`outcome_resubmitted_after_timeouts` 套在 `checksum_outcomes_of_the_units_cut_into_pieces` 的逐片闭包里；`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs` 第 152 行那一圈的计数是这一次调用的局部量，片与片之间不累计（乙、丙两段在第 1414、1535 行同形）。条款原文「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`），一批切成 K 片时代码至多重交 3K 次才报警。攻方复跑数见第一节。去向：第三节第 1 条（超时次数按批、按卡累计）；重交整批还是只重交超时的那一片交用户（第五节第 2 行，用户定重交单位不改）。
- **交活前先等前一次超时提交那一等一窗就报警**（正推 T3.1 第 4 行、打中第 7 条）：**打中，坐实**。主 agent 读码：同文件第 1272–1276 行 `room_once_the_submission_running_past_its_wait_completed` 在重交包装之外，出错直接交 `ChecksumBatchFailed::GpuFailed`。与上一条同一件事（一批里的超时次数），并进第三节第 1 条。
- **没点名的独显缺 PCI 地址也拒挂**（攻方 T1-2、正推 T1.1 第 11 行）：**打中，坐实**。主 agent 读码：`background_gpu.rs` 第 298 行先对本机全部独显跑 `discrete_gpus_of_the_enumeration`，第 300 行才按点名筛；`crates/singlefs-background-gpu/src/adapter_inventory.rs` 第 101–125 行三种拒都不看点没点名。条款「点名了就只用点名的卡」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`）。去向：第三节第 2 条。
- **点名虚拟显卡或认不出类型的适配器照「不存在」拒**（攻方 T1-3、正推 T1.3）：代码比条款严，条款只排集显与软件 Vulkan。是不是越位要看虚拟显卡该不该算「独立显卡」，交用户（第五节第 3 行）。
- **切换只读挂载那一刻的卡清单**（正推打中第 3 条、攻方 T1-4 末条）：**打中，坐实**（读码，没有卡造不出可跑的世界）。主 agent 读码：`background_gpu.rs` 第 1610–1613 行 `devices_with_total_memory` 交回 `self.discrete_gpus_after_the_opening`，没有现查。条款「挂载或切换那一刻……GPU 不在、驱动不对……就拒绝挂载（或拒绝切换）」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）。去向：第三节第 2 条。
- 其余（阈值恰好 100、上限恰好等于总量、`gpu` 而没递实现、一批之内切换、停着切换当场解除、只开 Vulkan、选 `cpu` 时只判阈值）：两条腿都判一致，**成立**。

### T2

- 三段方法两侧的入出类型、比对按条目不按片、`failure_under_the_settings_handed_in` 挂载与切换读同一个状态：两条腿都判一致，攻方三种形状都造不出，**成立**（没打中只抽了一次样，按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」只记「这一轮没打中」，不记「没有问题」）。
- **`TablesLargerThanABindingOnTheDevice` GPU 包不产、`input_bytes` 两侧含义不一**（正推打中第 5 条）：**打中，坐实**。主 agent 读码：`grep -rn TablesLargerThanABindingOnTheDevice crates` 只命中定义与一处测试里手造的值。去向：第三节第 3 条。
- 批内多段放弃时乙段的连续不确定计数先推进、计数按落点认（正推 T2.3、攻方 T2-3 旁记）：两条腿都判方向是早交 CPU、不漏判；条款「同一个条目连续 2 轮」没定义「轮」。**不改**，记进第五节第 5 行由用户看要不要写进条款。
- 共用把手的两个会话在交设置与读失败之间插队（正推 T2.4、T5 末段）：读码推，产品路径一个把手只给一个会话。**不改**，登记 C943（共用的后台 GPU 把手在交设置与读失败之间能被插队）。

### T3

- **一批里最小一片随映射变大时整理停下**（攻方 T3-1、正推 T3.3）：**打中，坐实**（纯函数层；真池里一个落点两条映射条目怎么来两条腿都没造出）。主 agent 读码：`crates/singlefs-background-gpu/src/gpus_of_this_host.rs` 第 396–400 行参与的卡只用与输入无关的 972 字节判，`crates/singlefs-background-gpu/src/region_record_kernels.rs` 第 663 行注释自己写「一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判」，逐批判出装不下交 `InputNotSupportedOnTheGpu`、整理停下。条款「参与的卡的上限连最小的一片都装不下，这张卡就不用……参与的卡因此一张都不剩时……后台任务照第 4 条进出错停着」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`）。去向：第三节第 4 条。
- **空闲显存不到上限时照常跑、只用空闲的，没有实现**（正推打中第 1 条）：**打中，坐实**。主 agent 读码：GPU 包只在开卡时读设备本地堆的总量，`grep -n -i -E 'memory_budget|free_memory|空闲' crates/singlefs-background-gpu/src/*.rs` 零命中。条款原文第 247 行「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」；七·3 验收（`.claude/kb/milestone/04-fourth-txn.md:145`）也列着这一句。去向：第三节第 5 条。
- **跨批攒没有实现**（正推打中第 2 条）：**打中，坐实**。条款第 247 行「GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管」；代码每一批各交三段（`crates/singlefs-core/src/mounted_session.rs` 第 3676–3692 行）。七·3 验收没有单列跨批攒，只在显存峰值那一句写「攒批、缓冲、暂存区一起算」。实现还是改条款交用户（第五节第 1 行）。
- **泄漏之后同一进程再挂，新的账不扣泄漏的那片**（正推 T3.4）：**打中，坐实**（读码）。条款「GPU 模式下全部后台任务在那张卡上合占的显存不超过它的那一个数……任何时刻都不许比它多」（第 247 行）。去向：第三节第 6 条。
- 4156 字节那一次红：两条腿都判修的是产品代码（第 48 批先拒、第 49 批量代价那一段改切片），用例两次改了输入、账没动。**成立**。
- 重交 3 次之内卡回来了而判用不了、超时提交回来之后账算错：攻方三种时序造不出，**这一轮没打中**。
- 账超上限是 `assert!`（正推打中第 9 条）：切批与断言两层守着账内不超上限，断言红是实现的缺陷，照 `.claude/singlefs-ai-sop/rules/code-discipline.md` 不变量破了就停。**不改**。

### T4

- **「停着先走删除续做」那一支不是死代码**（攻方 T4-1、正推 T4.3）：**打中，坐实**，推翻主 agent 在正文第二节第 ② 处的猜测。攻方复跑见第一节；正推另给两条进入路径（挂载交回时队列非空、会话里走空半路被拒），主 agent 读码 `crates/singlefs-core/src/mounted_session.rs` 第 3238–3260 行。停着而不超过阈值时第 3207 行在前、一个写都不发，与条款一致；停着而补跑时照常推，与条款「超过就照常推」一致。
- 删除续做的步数不进这一趟的发布预算（攻方 T4-1 后半）：主 agent 现查，`publishes_since_the_last_user_change` 只在第 3353 行（整理发布）加一，删除续做那一支是 2026-10-21 提交 `0b926bd3f`（判决 G8）就有的、不在这一段的补丁里（`grep -c -F 'if !queue_is_empty {' research/prompts/_m4-gpugc-code-r1-diff.md` 输出 0）。**不归这一段**，登记 C942（整理一趟里删除续做的步数不进发布预算）。
- **切换到一张卡都用不上的设置时，停着晚到下一批开头才记**（正推打中第 4 条、T1.2）：**打中，坐实**。主 agent 读码：`crates/singlefs-core/src/background_compute.rs` 第 1333–1337 行只在 `begin_a_batch` 里记停着，挂载在 `of_a_mount`（第 1124–1129 行）当场记；`mounted_session.rs` 第 3207 行与第 2388 行两道闸在那之前判「没停着」，于是那一趟先走删除续做，后台入口在第 2395 行取走「该跑了」不放回。代码自己的注释（第 3205–3206 行）写「停着……一个写都不发（删除续做也不走）」。去向：第三节第 7 条。
- **只供测试的开关在发布构建里够得着**（主 agent 第 ① 处，两条腿都判成立）：`run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch`（`mounted_session.rs` 第 2193 行）是 `pub`、没有 cfg 也没有特性门控，把 `&mut BackgroundComputeOfThisMount` 交给调用方的闭包。后果限于调用方自己的会话，是 C921（只供测试的开关在发布构建里够得着） 的射程；**不在这一段改**，C921 那一行补点名这两个开关与这个入口（第三节第 9 条的 kb 那一半）。
- 补跑读数取分配器此刻的增量计数，两条腿都判对，**成立**。
- 挪线程三样保证在拆成「判起跑」「跑」两段之后：两条腿都判仍成立，**成立**。正推另记「用户改动发成就清上一条完成的意图」是 2026-10-26 用户定的、kb 没写，「前台 fsync 在等时不发下一批」只约束整理批：并进第三节第 9 条的 kb 那一半。

### T5

- **卸载泄漏那一支测试进不去、泄漏了几张看不出**（攻方 T5-1、正推 T3.4 末条）：**打中，坐实**。主 agent 读码：`background_gpu.rs` 第 1596 行 `let _gpus_leaked =` 把交回值丢掉。去向：第三节第 8 条。
- **重交第几次、切没切片、这一批用了哪几张卡运行时看不出**（攻方 T5-2、正推 T5 表第 6、7、9 行）：**打中，坐实**。主 agent 读码：一批的记账 `GpuWorkOfTheBatch`（`crates/singlefs-core/src/background_compute.rs` 第 422–429 行）只数三段的确认、不确定、交 CPU 判与甲段对不上的单元数。`.claude/rules/fs-design.md`「五条硬要求」第 4 条「分支必须可观测」。去向：第三节第 8 条。
- **`BACKGROUND_TASKS_WITH_A_GPU_PATH` 写死长度 1**（主 agent 第 ③ 处）：两条腿都判加任务时编译器不逼、没有测试会红（攻方推的，没加成员试编）。**打中，坐实**（读码：`background_compute.rs` 第 1035 行是私有常量、数组字面量长度写死）。去向：第三节第 3 条。

### T6

- **补跑判定锁存改坏不红**（攻方 T6-2）：**打中，坐实**，复跑数见第一节。「补跑途中降到不超过、下一批起照旧停着」只经只供测试的开关进入。去向：第三节第 10 条。
- **「永远取第一块盘」改坏不红**（正推 T6.3）：**打中，坐实**，复跑数见第一节。`.claude/rules/mutation-sampling.md` 第三类（取样点不敏感）。去向：第三节第 10 条。
- **泄漏边界改成全部卡不红**（攻方 T6-1）：**打中，坐实**，复跑数见第一节；与 T5 卸载泄漏同一件，去向第三节第 8 条。
- **第 6104 行用例名写了「继续删除队列」而没有造队列**（正推 T6.2）：**打中，坐实**。主 agent 读码：`grep -c -E 'unlink|deleted_inodes|publish_unlink' crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs` 输出 0。去向：第三节第 10 条。
- **第 6063 行红在装置的前置断言**（正推 T6.2）：正推跑出来的。去向：第三节第 10 条（换一行红在「停着且超过阈值」断言上的变异）。
- 外壳三处重交包装、三处交活前先等与 `Drop` 的接线没有变异行（正推 T6.5）：并进第三节第 1、8 条。
- 重交次数改 2、切片合并少带一片：现有用例都红（攻方复跑），**成立**。
- 正文第二节与第三节 T6 写的变异行号 6131、6137、6138 是主 agent 写错的：那三行是 C773 的，乙段单元切片三行是第 6121–6123 行（正推 T6.0，主 agent 现查 `sed -n 6121,6123p crates/mutations.tsv` 三行名字都以「七·3 乙段单元切片」起头）。两条腿都按真实行号核了，不影响判定。

## 三、交给下一批的实现员（按文件聚簇）

一簇，文件有交集：`crates/singlefs-background-gpu/src/`（`background_gpu.rs`、`waiting_for_the_gpu.rs`、`memory_limit_and_chunks.rs`、`gpus_of_this_host.rs`、`adapter_inventory.rs`、`background_gpu_compute.rs`、`region_record_kernels.rs`）、`crates/singlefs-core/src/background_compute.rs`、`crates/singlefs-core/src/mounted_session.rs`、测试与 `crates/mutations.tsv`。第五节交用户的几行已定（第五节末列），都不加实现员的活：跨批攒删条款、虚拟显卡改条款、「轮」写进条款，三处都是 kb 改动，并进第 9 条。

| # | 问题 | 验收 |
|---|---|---|
| 1 | 一批里的超时次数按片计、交活前先等那一等不在重交里（T1） | 同一张卡、同一批里的超时（含交活前先等前一次超时提交那一等）跨片累计，第 4 次超时就报 `GpuUnusableAfterResubmissions`；替身用例「一批切成 4 片、每片超时一次」报警、「一批 1 片超时 3 次后做完」不报警；外壳三段各自的接线（套重交、交活前先等）各有一行变异会红 |
| 2 | 没点名的独显缺标识也拒挂；切换只读挂载那一刻的卡清单（T1） | 给了 `background_gpu_devices` 时只判点名的卡，没点名的卡缺 PCI 地址、缺设备本地堆、地址重复都不拒；切换那一刻现查卡清单，挂载之后掉了的卡切换时照「GPU 不在」拒；两条各一条用例（清单判法在 `adapter_inventory.rs` 的纯函数上，现查在实现里经一个能换成假的枚举） |
| 3 | 核心层不产的成员与两侧含义不一的字段；停着表写死长度 1（T2、T5） | 删掉 `TablesLargerThanABindingOnTheDevice` 或让 GPU 包产它并有用例钉映射；`input_bytes` 的文档与 GPU 包填的数同义；停着表由 `BackgroundTask` 的成员现推（加成员不改它就编不过，或有一条对每个成员断言的用例） |
| 4 | 一批里最小一片随映射变大时整理停下（T3） | 一张卡这一批切不出就这一批不分给它、活分给别的卡，参与的卡这一批全都切不出才报 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`；纯函数层两条用例：两张卡一张切不出照常交回、全都切不出报那一个成员 |
| 5 | 空闲显存不到上限时照常跑、只用空闲的（T3） | 卡上建缓冲因显存不足失败时，按更小的片重切重交、不报 GPU 出错；替身用例「上限 1 MiB、实际只放得下 256 KiB」那一批照常交回、判定与 CPU 参照逐条相同；真卡用例另交主 agent 跑 |
| 6 | 泄漏之后同一进程再挂的账（T3） | 进程里记每张卡泄漏的字节，新一次挂载这张卡的可用余量先扣掉它；纯函数层用例 |
| 7 | 切换到一张都用不上的设置时停着晚一步（T4） | 挂着切换到参与的卡一张都不剩的设置，下一次整理入口与后台入口在任何写之前就报停着、不走删除续做、不取走「该跑了」，与挂载那一刻同；一条用例（造非空的删除队列，断言盘上逐字节不变、「该跑了」还在） |
| 8 | 卸载泄漏测试进不去、看不出；重交第几次、切没切片、用了哪几张卡看不出（T5） | 泄漏了几张、哪几张交到调用方看得见的地方（例：卸载那一刻的报告），`Drop` 传给判泄漏的那一位有用例与变异行会红（泄漏边界改成全部卡要红）；一批的记账另记每段交了几片、重交了几次、用了哪几张卡，各有用例钉绝对值 |
| 9 | kb 那一半（主 agent 写回，不派实现员） | `.claude/kb/mount-parameters.md` 落点行号现查重写、补三种整个拒挂的情形与卡标识的写法（正推打中第 6 条，主 agent 读码核过 `mount.rs:8658`、`mounted_session.rs:2241` 已不是入口）；C921 那一行点名两个新开关与带钩子的入口；D26（后台整理与放置回收） 已定项 13 第 4、6 条补「切到一张都用不上当场停着、报同一个成员」「用户改动发成清上一条完成的意图」「前台 fsync 只约束整理批」；用户 2026-10-30 定的三处：第 3 条删「GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管」与显存账里的「攒批」字样，第 2 条补虚拟显卡与认不出类型的适配器不算独显，第 8 条补「轮」的口径（这个落点出现在一批里且不确定；放弃的那一批里已算完的段照算一轮）；变更史与三语同步 |
| 10 | 测试与变异缺口（T6） | 补跑用例加一格「补跑途中真删文件让读数降到不超过、下一批起照旧停着」，不经只供测试的开关，攻方锁存那一种改坏要红；加一格最满的是第二块盘，正推「永远取第一块」那一种改坏要红；第 6104 行点名的用例造非空的删除队列（攻方 T4-1 回退那条路），断言停着时队列照旧在；第 6063 行换成红在「停着且超过阈值」那条断言上的变异 |

## 四、复跑与核不动的

- 正推量的数（抽样重跑 7 行变异都红、6104 与 6063 红在哪一行）没有单独复跑：6104 那一行的用例文件主 agent 读码核过不造队列；6063 只当线索用在第三节第 10 条。
- 攻方 T3-1 在纯函数层，真池里一个落点两条映射条目没造出来；T1-4 末条、T3-5 全部按代码推（没有 GPU）。
- 真卡与 WGSL：两条腿都没起 GPU，29 行真卡变异行红在哪只读码推。
- `region_scan.rs`、`checksum.rs` 两处快写法：攻方没攻；正推只列了对拍用例与开关（T4.5），判一致。这两处这一轮只算抽了一次样。

## 五、交用户

| 岔路 | 事实 | 候选 | 用户定（2026-10-30 弹窗） |
|---|---|---|---|
| 跨批攒（第 247 行「GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管」） | 代码每一批各交三段，没有跨批攒（`mounted_session.rs` 第 3676–3692 行）；七·2 设计轮判「GC 留在调用线程里开 GPU…跨批攒不成」（第 261 行依据），之后 GC 已挪出调用线程；七·3 验收没单列跨批攒 | 实现（改 `crates/`，量收益另立实验）/ 删掉条款这一句（改 kb） | 「删掉条款这一句 (推荐)」 |
| 超时之后重交什么 | 条款「放弃的那一批在同一张卡上重交」，用户 2026-10-28 原话「重试是把那一批重新交一次」；代码重交的是超时的那一片，同一批已做完的片留着 | 只重交超时的那一片（今天的写法，次数按批累计）/ 整批重交 | 先答「不再设置超时了 一直排队。 一个脚本执行完后  查看后一个任务有没有接着跑 没有的话就提醒对方的对话，跑了的话就不管了。」；再弹窗确认这句管的是「② 验证脚本的排队」，重交单位答「和GPU后台任务无关不改 说的是测试脚本」：重交单位照今天的写法 |
| 虚拟显卡与认不出类型的适配器 | 条款只排集显与软件 Vulkan；代码另排虚拟显卡（`adapter_inventory.rs` 第 46 行）与 `Other`，点名它们照「不存在」拒 | 照代码排除（改条款补上这两类）/ 照条款纳入 | 「照代码排除，改条款 (推荐)」 |
| 只供测试的开关 | 两个开关与带钩子的入口都 `pub`、没有特性门控；同包 `lookup-work-counters`、`test-only-publish-thresholds` 是特性门控的；后果限于调用方自己的会话 | 照 C921 延后（只补登记）/ 这一批就加特性门控 | 没问：照 C921（只供测试的开关在发布构建里够得着） 已有的延后，只补登记 |
| 「连续 2 轮」的轮 | 计数按落点（盘、槽、跨度）认，没出现在这一批的条目计数不动；一批放弃时已交回的段计数先推进；两样都是早交 CPU 的方向 | 照代码写进条款 / 改代码按条目身份与按批回滚计数 | 「照代码写进条款 (推荐)」 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-10-30 改了：第 3 条删跨批攒那一句、第 2 条补虚拟显卡与认不出类型的不算、第 8 条补「轮」的口径（用户 2026-10-30 定，第五节）；第 4、6 条补三句（第三节第 9 条），随第 54 批合入之后的写回一起改 |
