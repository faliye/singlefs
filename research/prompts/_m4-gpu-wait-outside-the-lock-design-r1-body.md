# m4-gpu-wait-outside-the-lock-design-r1：后台整理一批等 GPU 拆到会话锁外，前台等锁都让路，设计轮第一轮正文（2026-11-18）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 -->

## 一、题面

用户 2026-11-18 弹窗定两条：锁与 GPU 选「现在拆到锁外 (Recommended)」；让路范围选「任何前台等锁都让 (Recommended)」。前一条把 2026-11-09 定的「每次发布放锁，GPU 在锁内先接受」里「里程碑六再拆」那一半提前到现在做（`research/prompts/m4-gpugc-background-runtime-design-r1-main-verification.md` 第 36 行；欠账 C998（一批之内持锁等 GPU））；后一条改 D26（后台整理与放置回收） 已定项 13 第 6 条「前台有 fsync 在等时不发下一批」的让路范围。

这一轮不判要不要做，判**怎么拆才不错**：交实现员之前把形态与会错的格定下来。并行提交（里程碑六一·1 到一·3）不在这一轮：发布照旧串行、照旧在会话锁里。

这是这一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

## 二、实现今天的样子（主 agent 的观测，2026-11-18）

- 一把锁：`crates/singlefs-mount-runtime/src/lib.rs` 第 387 行 `SessionAndDevices`（攒批的会话 `MountedSessionWithPendingChanges` 加盘表）住在第 396 行 `locked: Mutex<SessionAndDevices<Device>>` 里；取锁只经第 491 行 `lock_the_session`，锁中毒或坏过就报 `RuntimeFailed`。文件头第 5 行写「每次发布之间放锁……一批之内 GPU 仍在锁内等（D24（后台重活能不能卸给 GPU） 已定项 2 的射程，里程碑六拆）」。
- 前台入口全走这把锁：`accept`（第 956 行）、`publish_pending_now`（第 981 行，fsync）、`publish_user_change`（第 1020 行）、`switch_background_compute`（第 1046 行）、`with_the_session_and_devices`（第 1075 行，读路径 `read_at` 要盘表，只能走它）。锁是 `std::sync::Mutex`，读与读也互斥。
- 让路：只有 `publish_pending_now` 在取锁之前登记 `fsync_waiters`（第 987 行、第 1102 行）。`CallerOfTheRuntime::answer_before_the_next_batch`（第 428 行）只看 `stopping` 与 `fsync_waiters`；核心层 `crates/singlefs-core/src/mounted_session.rs` 第 4732 行每一批之前问一次，答有 fsync 在等就停在 `CompactionRunStop::YieldedToAForegroundFsync`；全空段低于底额、这一趟第一次发布的破例不让（第 4720 到 4734 行）。
- 维护线程的一拍：`run_one_maintenance_pass`（`lib.rs` 第 600 行）循环至多 `COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST`（第 61 行，64）次，每次取锁调 `run_compaction_in_the_background_for_at_most_one_publish`（`mounted_session.rs` 第 7559 行）、放锁、记观测、送事件、再取锁调 `publish_when_due_once`，接着下一次。两次取锁之间没有等前台的一步。运行库的测试 `crates/singlefs-mount-runtime/tests/` 五份文件里 `grep -n "thread::spawn"` 零命中：没有真线程用例验非 fsync 的前台在两次发布之间拿不拿得到锁（「拿不到」是推的，没量过）。
- 一批怎么算：`mounted_session.rs` 第 5110 行 `advance_compaction_by_one_publish` 进门 `begin_a_batch` 读一次这次挂载的后台计算设置，走 GPU 时第 5160 行 `gpu_compute.lock()` 取 GPU 实现那把锁（`BackgroundGpuComputeHandle`，`crates/singlefs-core/src/background_compute.rs` 第 735 行 `of_a_shared_implementation` 收的是 `Arc<Mutex<dyn BackgroundGpuCompute + Send>>`），整段 `advance_compaction_by_one_publish_computing_on`（第 5355 行）都持着它，也都在会话锁里。
- 一批里读什么、GPU 调在哪：第 5368 行取现行那一版的树 `version`（克隆），第 5392 行 `version_as_the_compaction_reads_it`，第 5396 行每读一版建一次映射条目的按身份索引。没有活意图时：丙段 `live_slots_per_cluster_segment_computing_on`（第 5487 行，调 `count_live_slots_per_cluster_segment`）数一次，然后一个循环（第 5495 行）：`choose_region_over_the_live_slots` 答「先扫」就 `scan_of_the_batch`（读盘上那一段的单元，甲段 `verify_unit_checksums`、乙段 `classify_region_records` 上 GPU）再问，迭代上界是单元区的整段数；选中之后在锁里判空洞、签发意图、发布（第 5599 行）。有活意图时：对 R 重扫一次（第 5634 行，同样甲、乙两段上 GPU），再计划、发布。GPU 的输入来自 `version`、盘上读到的字节、分配器的 `cluster_segments()` 与 `is_free`、会话这一侧的隔离集与被拒段表；交回出错在任何一次发布之前（第 5350 行）。
- GPU 一侧等多久：等的上限默认 10 秒（`background_compute.rs` 第 905 行 `BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT`，注释写「默认值是推的，待量」，欠账 C926（后台 GPU 等的上限默认 10 秒没量过））；超时在同一张卡上重交，`RESUBMISSIONS_AFTER_A_TIMEOUT` 是 3（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs` 第 120 行），一段一次调用里同一张卡第 4 次等超过上限才报这张卡用不了：卡挂住时一段持会话锁最多约 40 秒（读码算的，没在真卡上量）。`grep -n "stopping\|cancel\|叫停" crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs` 零命中：等的那一处不查停线程的标记。
- 卸载与丢弃：`stop_and_join_the_maintenance_thread`（`lib.rs` 第 1110 行）与 `Drop`（第 1200 行）都 join 维护线程，注释写「一批之内 GPU 还在等时要等到那一批出来」。
- 开卡已经在会话锁外：`switch_background_compute` 的文档（`lib.rs` 第 1038 到 1042 行）写沿用这次挂载那一份实现而点到没开着的卡时，实现在会话锁外开卡、开完在两批之间换上（`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`，`background_compute.rs` 第 646 行起）。等 GPU 算一批没有这样的锁外路径：`grep -rn "锁外" crates/singlefs-core/src/mounted_session.rs` 命中的都是开卡。
- 核心层的约定：`mounted_session.rs` 第 7555 行「挂着的会话这一层不起线程、不持锁，放锁是调用方的事」。
- 2026-11-09 那一轮攻方已打中的：C1「锁内等 GPU：前台等锁等的就是 GPU」、C2「只有 fsync 让得了路」与反向饿死（前台 fsync 不断时一批都不发），`research/prompts/m4-gpugc-background-runtime-design-r1-opus-output.md` 第 20、21、212 行；当时的处置是用户选锁内先接受。
- 压着的条款：D24（后台重活能不能卸给 GPU） 已定项 2；D26（后台整理与放置回收） 已定项 13（第 1、4、5、6、8 条）、已定项 2；D16（发布语义） 已定项 1、2、5、7；D23（journal 的角色与格式） 已定项 14；欠账 C998（一批之内持锁等 GPU）、C943（共用的后台 GPU 把手在交设置与读失败之间能被插队）、C926（后台 GPU 等的上限默认 10 秒没量过）、C1030（等提交时卡真丢了会不会持锁 panic 没量过）；`.claude/kb/milestone/06-sixth-txn.md` 一·4；`.claude/rules/fs-design.md`「一个事务层，所有结构共用」「五条硬要求」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### Q1　拆法的形态

核心层不起线程、不持锁。等 GPU 拆到锁外，核心层的后台入口要拆成几段、每段交回什么：锁里取输入（哪些要克隆、哪些读盘）、锁外交 GPU 等结果、回锁里接着判与发布。没有活意图那一支的循环（选段答「先扫」→ 读盘 → GPU → 再选）每一圈都要读盘：读盘在锁里还是锁外、一圈放一次锁还是一批放一次。GPU 实现那把锁（`BackgroundGpuComputeHandle` 的 `Mutex`）在锁外等的这段时间谁持着、切换与卸载碰它时怎样。CPU 那一条路径拆不拆成同样的段。

### Q2　回到锁里之后复核什么才够

锁外等的这段时间前台可能发了任意多次发布、切换了设置、冻结了发布、panic 过。回锁里拿什么判「锁外算的结果还能用」：现行那一版的 txg、分配器的状态、隔离集与被拒段表、活意图、后台计算设置（一批之内不换，`fs-design.md` 五条硬要求第 5 条）。对不上时怎么办：整批丢掉重算、还是只重算变了的那一部分；丢掉的那一批算不算进预算、第 8 条的连续不确定计数动不动、GPU 记账怎么报。前台改动不断时后台会不会每一批都被作废、永远发不出（活锁），有没有下限。

### Q3　任何前台等锁都让路

用户定了范围，这一格判怎么做才不错：锁外的计数登记在哪几处入口（`accept`、读、`publish_user_change`、`switch_background_compute`、`with_the_session_and_devices`），停因另起成员、不冒充 fsync（`fs-design.md` 五条硬要求第 4 条）；让路是在两次调用之间由运行库判、还是照旧由核心层每批之前问；`std::sync::Mutex` 不保证公平，放锁之后等锁的前台怎么保证先拿到；全空段低于底额的破例照旧不让。反向饿死：让路范围放宽之后前台不断时后台一批都不发，第 6 条要不要「至少推一批」的下限、下限取什么观测。D26（后台整理与放置回收） 已定项 13 第 6 条的改句。

### Q4　崩溃语义与一个事务层变没变

发布照旧在会话锁里、照旧一次一个。拆开之后前台的发布插在「取输入」与「整理发布」之间：盘上能出现的写序列多了哪些，崩溃点重放要不要新流，新流比已有的流多罩什么状态；意图签发与 R 排他（`allocator.compaction_mut().issue`）在锁外那一段是什么态；有没有哪一步变成了绕开事务状态机的第二条发布路径（`fs-design.md`「一个事务层，所有结构共用」）。

### Q5　卸载、丢弃、切换、出错与 panic

锁外等 GPU 时：卸载与 Drop 还等不等这一批（等多久、不等的话那一批与那张卡怎么办，对 D26（后台整理与放置回收） 已定项 13 第 4 条「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」）；切到 `cpu` 或换一组卡的请求什么时候生效；GPU 交回出错、等超时重交 3 次、等的那一处 panic（欠账 C1030）各在锁外还是锁里处置，停下与报警、按占用在 CPU 上补跑走不走得到；前台在锁里 panic、锁中毒时锁外那一批怎么收。欠账 C943（把手几处共用时交设置与读失败之间的窗）在拆开之后变大还是变小。

### Q6　怎么验才算数

验收清单：真线程用例量非 fsync 的前台在后台一拍里等多久、拿到锁之前后台发了几次布（先在今天的代码上量出基线）；C998 计划的检查（真卡上后台一批等 GPU 时前台取到锁之前没有等一次 GPU 提交）扩到读与 `accept`；卡挂住的替身上前台与卸载各等多久；回锁复核每一种对不上各一条用例；GPU 与 CPU 两条路径在同一组输入上逐项相同照旧成立。哪些用替身卡验得了、哪些非真卡不可。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | Q1–Q6，Q1、Q2 主攻 | 给一份能照着写的形态：核心层分段入口的接口草案、回锁复核的判据表、运行库一拍的新次序、条款改句、验收清单；每条落到 `crates/` 的文件与行；代价能量的写成模型与命令 |
| 云端攻方（Opus） | Q1–Q6，Q2、Q4、Q5 主攻 | 假设「锁里取输入、锁外等、回锁复核」是错的：造回锁复核漏判的可达历史（锁外算的结果过期却被用了）、后台被前台作废到永远发不出的历史、崩溃语义变了的写序列、卸载与切换撞上锁外那一批的历史；由用户决定的动作放开扫 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写主工作区路径与行号（设计轮没有冻结副本，读的时候核一下那一行还在）。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷一份副本，不在主工作区改。读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不写进命令）。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 岔路写成候选加代价，不替用户定。
- 回复与报告用中文。
