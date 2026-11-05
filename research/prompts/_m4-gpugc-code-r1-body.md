# m4-gpugc-code-r1：里程碑四七·3 GPU 垃圾回收，代码轮第一轮正文（2026-10-30）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 -->

## 一、题面

七·3（`.claude/kb/milestone/04-fourth-txn.md` 作业点表七·3 那一行）这一段的写段与验证段都已结束：第 39 到 51 批共 18 份补丁打进主工作区，合入后验证这一段的目标全绿，真卡对拍 20 条绿，这一段的变异行全部证红，形式化轻型 3 格绿，崩溃轻型整理四项绿（`records/2026-10-19-里程碑四GPU垃圾回收开工.md`「验证段收尾（这一轮的部分）」那一行）。这一轮照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 2 步攻这一段合入的产品代码：正推腿核「代码做的是不是 D26（后台整理与放置回收） 已定项 13 与七·3 那一行说的那件事」，攻方腿攻「哪一格会错」。第一轮的分工照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

改动全文：`research/prompts/_m4-gpugc-code-r1-diff.md`（18 份补丁里产品代码的块按打进主工作区的次序原样抄录，25744 行；测试、变异表与门禁的块不在里面，整份补丁在各自的目录里，路径写在每一节的标题上）。

## 二、实现今天的样子（主 agent 的观测，2026-10-30）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r1/repo/`（此刻主工作区，含这 18 份补丁与别的会话这期间合入、没提交的改动；去掉了 `.git`、编译目录、私有配置 `multi-host.env` 与 `.env`）。被判的 28 份产品代码文件的 sha256 在 `research/prompts/m4-gpugc-code-r1-snapshot/crates-sha256.txt`，冻结副本逐份对上。
- 产品代码改动落在两个包：新包 `crates/singlefs-background-gpu/src/`（18 份文件，全部是这一段写的）与核心层 `crates/singlefs-core/src/` 的 `background_compute.rs`（新文件）、`mount.rs`、`mounted_session.rs`、`compaction_plan.rs`、`region_scan.rs`、`checksum.rs`、`admission.rs`、`transaction.rs`、`block_device.rs`、`lib.rs`。同一份文件里不是这一段的改动：`mount.rs` 的未暂存块（`rollback_floor_ceiling_of_this_process`、`push_one_floor_raise_within_the_admission_budget` 一带，代码轮 m3-close-b19 那一路）、`block_device.rs` 除第 563 行附近一处注释之外的全部、`transaction.rs` 除挪线程那一批（`m4-gpugc-off-thread-writer`）碰的几处之外的全部、`mounted_session.rs` 里别的会话的发布触发补丁（`publish_user_change` 进门先排空待发、后台整理入口拆成「判起跑」「跑」两段）。腿只攻 `_m4-gpugc-code-r1-diff.md` 里有的块，别的块当上下文读、不攻。
- 冻结副本里的落点：
  - 核心层 trait 与状态：`crates/singlefs-core/src/background_compute.rs` 第 269 行 `pub trait BackgroundGpuCompute: GpuInventory`，第 306 行 `use_the_settings_from_the_next_batch`，第 313 行 trait 默认方法 `failure_under_the_settings_handed_in`（挂载时在第 1124 行 `of_a_mount` 读）；出错成员第 190 行 `GpuUnusableAfterResubmissions`、第 198 行 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`、第 1031 行 `GpuUnusableUnderTheSettingsHandedIn`（挂载与切换共用，第 1046 行经写死的 `BACKGROUND_TASKS_WITH_A_GPU_PATH`（第 1035 行，长度 1）逐个任务记停着）；补跑阈值第 514 行 `BackgroundGcCpuCompensationThreshold`（默认 85，第 521 行）、第 531 行 `AllocatedBytesAndCapacityOfADevice`（按盘）、第 590 行 `is_the_allocated_share_above_the_threshold`、第 1220 行起 `whether_the_task_runs_the_next_batch`；只供测试的开关第 1142 行 `judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests`；等的上限默认第 503 行 `BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT`（C926（后台 GPU 等的上限默认 10 秒没量过））。
  - 整理一趟里的分支：`crates/singlefs-core/src/mounted_session.rs` 第 3206 行起（停着而每块盘都不超过阈值就一个写都不发、第 3098 行 `compaction_stays_stopped_pausing_the_live_intent`），第 3230 行起「删除续做在整理批之前」那一支；后台入口 `run_compaction_in_the_background` 第 2334 行；挂载递实现进来 `crates/singlefs-core/src/mount.rs` 第 8946 行起（`gpu_compute: Option<BackgroundGpuComputeHandle>`，第 8960、8983 行）。
  - 产品 GPU 包：`crates/singlefs-background-gpu/src/background_gpu.rs` 第 1138 行 `use_the_settings_from_the_next_batch`、第 1593 行 `impl Drop for BackgroundGpu`（卸载时还记着一次超时提交的卡整张泄漏，经 `gpus_of_this_host.rs` 第 549 行 `gpus_dropped_leaking_those_still_running`）；等与重交 `waiting_for_the_gpu.rs` 第 117 行 `RESUBMISSIONS_AFTER_A_TIMEOUT = 3`、第 152 行循环；切片 `memory_limit_and_chunks.rs`，乙段头一片 `region_record_kernels.rs` 第 92 行 `UNIT_HEAD_PIECE_INPUT_BYTES = 628`，最小一片 `gpus_of_this_host.rs` 第 1190–1194 行（972）；trait 实现 `background_gpu_compute.rs` 第 508 行 `use_the_settings_from_the_next_batch`、第 513 行 `failure_under_the_settings_handed_in`；两份核比对 `two_kernel_reconciliation.rs`；按量出的代价分活 `work_split_by_measured_cost.rs`；卡清单 `adapter_inventory.rs`、`gpus_of_this_host.rs` 第 71 行。
  - CPU 侧两处快写法：`crates/singlefs-core/src/region_scan.rs`（按身份查映射条目）、`crates/singlefs-core/src/checksum.rs`（CRC-32C 硬件指令，checker 那一份按位实现照留）。
- 验证段的数（量的，`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 验证段各行）：合入后验证（第二台，`~/.local/share/singlefs/work/merge-and-verify/2026-10-29-811319/`）GPU 包库「ok. 116 passed」、`singlefs-core` 库「ok. 368 passed」、`compaction_on_the_cpu_while_stopped_after_a_gpu_failure`「ok. 9 passed」、`compaction_batches_on_the_gpu_path_through_the_background_gpu_compute`「ok. 11 passed」、`compaction_off_the_calling_thread_and_admission`「ok. 13 passed」；真卡对拍格「4 个对拍测试目标（特性 tests-on-a-real-gpu）每个都跑到了用例，共 20 条（含复用；按条：现判 20、复用 0、拒算 0、本次未跑 0）」；变异轻型「计数：抓到 325 条、没红 2 条、无效 1 条…（共 331 条）」加补跑「计数：抓到 298 条、没红 0 条、无效 0 条…（共 298 条）」，这一段的行全部红；崩溃轻型整理四项「CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=4 collected=4 green=4 red=0 not_run=0」。合入后验证里的红、崩溃轻型整套的红都落在别的会话没提交的文件上（池级 checker `walk.rs`、checker 档判器 WGSL、e183、e197、实例表），不当这一段的前提。
- 主 agent 已记下、要腿判的几处（报出来不算打中，由腿判后果）：① 只供测试的开关 `judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests` 在发布构建里够得着（C921（只供测试的开关在发布构建里够得着） 的射程）；② `mounted_session.rs` 第 3230 行起「停着先走删除续做」那一支，主 agent 造不出进得去的输入（会话里删除一次走完、挂载续做在建后台计算之前走空），问它是不是死代码；③ `BACKGROUND_TASKS_WITH_A_GPU_PATH` 写死长度 1，以后加带 GPU 路径的后台任务时编译器不逼着改它；④ 卸载时整张卡泄漏（用户 2026-10-28 定「卸载时同样不等、泄漏设备」）：泄漏的边界是不是只在「还记着一次超时提交」的卡；⑤ 重交 3 次用的是同一张卡、同一批，第 4 条写「放弃的那一批在同一张卡上重交」；⑥ 补跑按盘：任一块盘「已分配 ÷ 容量」超过阈值就补跑（用户 2026-10-28 定「任一块盘超过就补跑」），读数取分配器此刻最满的那一块盘；⑦ 别的会话把后台入口拆成「判起跑」「跑」两段之后，这一段的「用户改动返回之前一批整理都不发」与「前台 fsync 在等时不发下一批」还成不成立；⑧ 第 45、46 批的读法：参与的卡至少装得下三段里最大的那一片（972 字节）才参与、只切比头一片长的单元、一样满取排前面的那块盘。
- 压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 13 八条（第 245–252 行）与它的射程段；`.claude/kb/milestone/04-fourth-txn.md` 七·3 那一行的验收；`.claude/kb/mount-parameters.md` 挂载参数表；`.claude/rules/fs-design.md`「五条硬要求」（每条分支能被测试强制进入、可观测、分支变量不许在操作中途改变）与「记账是事务的副产品，不是事后的遍历」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　过度外推与越位：代码管的比条款多、还是比条款少
核：挂载参数四个（`background_compute`、`background_gpu_devices`、`background_gpu_memory_limit`、`background_gpu_wait_limit`）与第 49 批加的 `background_gc_cpu_compensation_threshold`，代码里拒挂、拒切换的每一支各对第 4 条哪一个分句，有没有条款没写而代码拒了的（例：阈值恰好 100、上限恰好等于显存总量、点名的卡重复、`gpu` 而没递实现）、条款写了而代码放过的；切换「从下一批开始生效、一批之内不换」在哪一行取生效（`use_the_settings_from_the_next_batch` 调在批的开头还是中间），停着时收到通过校验的切换是不是「当场」解除；第 2 条「不给就用本机全部独立显卡，集成显卡与软件 Vulkan 适配器不算」在 `adapter_inventory.rs` 里按什么字段判，Vulkan 以外的后端会不会混进来；补跑只管 GC、不管打包搬迁，代码里停着的任务表 `BACKGROUND_TASKS_WITH_A_GPU_PATH` 只有一项，第 4 条的「那个后台任务停下」与「GC 补跑」分不分得开。

### T2　内部矛盾与循环依赖：核心层与 GPU 包各自的约定对不对得上
核：核心层 trait `BackgroundGpuCompute` 的三段方法与 GPU 包 `background_gpu_compute.rs` 的实现，入出类型、错误成员、切片之后的合并（甲段校验和拆片合并、乙段头一片 628 字节之后带进来的 CRC 寄存器、丙段）各自的不变量；两份核比对 `two_kernel_reconciliation.rs` 与第 8 条：确认、不确定、连续 2 轮不确定交 CPU 判的计数住在哪、重挂清不清零、切片之后一个条目跨两片时两份核的比对是不是仍按条目；出错之后「正在算的那一批放弃」——意图留在盘上、目标态单调（已定项 2）——与会话里 `last_completed_intent`、回滚清意图（第 44 批）的次序；`failure_under_the_settings_handed_in` 这个 trait 默认方法在挂载（`of_a_mount`）与切换两处读的是不是同一个状态、GPU 包在 `use_the_settings_from_the_next_batch` 里更新它的时刻与核心层读它的时刻有没有错开一批。

### T3　哪一条最脆弱：出错与资源边界
核：等与重交——`waiting_for_the_gpu.rs` 第 152 行循环里，重交 3 次各自的等待上限是不是同一个 `wait_limit`，先前超时的提交还没回来时的新提交怎么算（`EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit`），一张卡用不了之后别的卡接不接它的活、还是整批放弃；显存上限——`memory_limit_and_chunks.rs` 的账：攒批、缓冲、暂存区是不是都记在里面、切片之后每片的峰值是不是仍不超上限（真卡用例 `a_memory_limit_smaller_than_the_batch_cuts_the_segments_into_chunks_with_the_same_live_slots` 在验证段曾因「本包在这张卡上占的缓冲 4156 字节超过了上限 500 字节」红过一次，第 50 批之后绿，问那一处是修了账还是修了用例）、「连一片都装不下就不用这张卡」与「参与的卡一张都不剩」的成员；卸载时的泄漏——`impl Drop for BackgroundGpu` 泄漏的那张卡持有的主机内存与显存有没有上限、泄漏之后进程里还能不能再挂一次。

### T4　CPU 一侧：补跑、挪线程与两处快写法
核：补跑的读数 `AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator` 取的是分配器此刻的数还是上一次发布的数，「已分配」含不含保留与整理暂存；只供测试的开关 `judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests` 在发布构建里被调到的后果；`mounted_session.rs` 第 3206 行起停着不补跑时「一个写都不发（删除续做也不走）」与第 3230 行「删除续做在整理批之前」两支的先后——第二支在停着时进不进得去，进得去时删除续做发的写算不算「一个写都不发」的违约；挪出调用线程的三样保证（一趟发布预算有上限、签下一条意图那一判跨趟留着、整理批不插进用户改动的抬 F 与再判之间）在别的会话把入口拆成「判起跑」「跑」两段之后各由哪一行守着；`region_scan.rs` 按身份查映射条目与线性找、`checksum.rs` 硬件 CRC-32C 与软件写法、与 checker 按位实现，三样对拍各在哪条用例里、强制进入另一条分支的开关在哪。

### T5　五条硬要求与可观测
核：`.claude/rules/fs-design.md`「五条硬要求」逐条：每一条分支（cpu / gpu、停着 / 补跑 / 照设置走、切片 / 不切、重交第 1–3 次、两份核相同 / 不同 / 交 CPU 判）能不能被测试强制进入，运行时从 `background_compute_of_each_batch`、每批报的三个数看不看得出走了哪条；分支变量（设置、阈值、参与的卡）在一批之内会不会变；`BACKGROUND_TASKS_WITH_A_GPU_PATH` 写死长度 1 时，加一个任务而没改它的世界里哪一条测试会红。

### T6　测试与变异判的是不是那件事
核：这一段的变异行（`crates/mutations.tsv` 里名字以「七·3」起头的各行，加第 51 批改过输入的乙段切片三行 6131、6137、6138）各自红在哪条断言、换一种同样坏的写法还红不红；真卡用例只在有卡的机器上跑，没卡时 `NOT_RUN_NO_GPU_ADAPTER` 之后返回成功，这一段里哪些行为只有真卡用例罩着、替身（`crates/singlefs-harness` 的假 GPU）罩不到；第 51 批把切片用例的载荷改成 20000 字节、内容各异之后，还有没有别的切片用例的输入落在「非零字节都在头一片之内」这一类取样点不敏感里（`.claude/rules/mutation-sampling.md` 第三类）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T6 | 核「代码做的是不是 D26（后台整理与放置回收） 已定项 13 八条与七·3 验收说的那件事」：逐格对补丁块与冻结副本的行，每条结论写文件:行号；T1 列出每一支拒挂、拒切换对第 4 条哪一个分句；T2 列出三段方法在核心层与 GPU 包两侧的类型与错误成员对照表；T6 逐条列这一段的变异行改的是哪一行、点名的用例红在哪条断言、为什么那条断言判的是这一处 |
| 云端攻方（Opus） | T1–T6 | 在冻结副本的拷贝上造反例：T1 造一组挂载参数让代码拒而条款不拒、或条款拒而代码放过；T2 造一批输入让切片前后两份核的比对结果不同、或让 `failure_under_the_settings_handed_in` 在挂载与切换两处读到不同状态；T3 用替身造一个重交 3 次之内卡回来了而代码仍判卡用不了的时序、或一个切片之后峰值超上限的账；T4 造一个进得去「停着先走删除续做」那一支的世界，造不出就证明它是死代码；T5 找一条测试强制不进去的分支；T6 造一个改坏（切片合并少带一片、比对按片不按条目、补跑读数取旧值）而这一段的变异行与用例都不红的变异 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；引补丁块写 `_m4-gpugc-code-r1-diff.md` 那一节的补丁目录名，不写那份文件的行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷过去的源文件先 `find crates -type f -exec touch {} +` 再编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo；用自己的编译目录。本机资源紧张就尝试其他可用的机器（`scripts/hosts/multi-host-run.sh --peer`）。
- 不起 GPU：真卡用例（特性 `tests-on-a-real-gpu`）不跑，用替身与库单测；不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
