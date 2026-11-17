# m4-gpugc-mount-runtime-code-r1：里程碑四七节 GPU 垃圾回收，挂载就起的后台运行库、水位、批大小与回退门，代码轮第一轮正文（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 -->

## 一、题面

设计轮判决 `research/prompts/m4-gpugc-background-runtime-design-r1-main-verification.md` 第二节定了形态与用户四问（K=16；E 线 5%、下限 2，F、U 先关，不存盘；加门保住 4 个状态、挂载不给新预算、低于底额破例；每次发布之间放锁、一批之内 GPU 持锁），用户 2026-11-09 另定「门留 8 槽余量」。第 105–108 批实现合入，补丁目录 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-merged/patch/`。第 102 批修了判决 `research/prompts/m4-gpugc-one-vulkan-instance-code-r1-main-verification.md` 打中的几处，并进这一轮一起攻。

用户的目标：「最终该功能可以正确的接入并运行」。

这是这一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

改动全文：`research/prompts/_m4-gpugc-mount-runtime-code-r1-diff.md`（产品代码的块，测试与变异表的块在补丁目录）。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1/repo/`（全部补丁打进主工作区之后那一刻的拷贝）。被判文件的 sha256 在 `research/prompts/m4-gpugc-mount-runtime-code-r1-snapshot/crates-sha256.txt`。这几份文件里不是这一批的块（别的会话的改动）当上下文读、不攻。
- 被判的产品代码：`crates/singlefs-mount-runtime/src/lib.rs`（新，1029 行）；`crates/singlefs-core/src/` 的 `compaction_plan.rs`、`mounted_session.rs`、`mount.rs`、`system_configuration.rs`、`allocator.rs`；`crates/singlefs-background-gpu/src/` 的 `gpu_opener_of_this_process.rs`（新）、`background_gpu.rs`、`lib.rs`、`adapter_inventory.rs`、`bin/e202_gpu_cpu_gc_comparison.rs`。
- 量到的（在实现员副本上，合入之前）：
  - 运行库 15 条、开卡器 3 条绿（任务 10966、10959）。
  - 回退门目标 14 条绿（任务 10976）；验证包其余 10 个目标绿（任务 10960、10965）。
  - 真卡端到端：`the_runtime_on_the_real_gpus_writes_the_same_block_sequence_as_on_the_cpu ... ok`，本机 `0000:10:00.0`（任务 10967），是小池、一张卡。
  - 运行库 16 行变异抓到（任务 10995）。
  - clippy 四个包绿（任务 10977）。
  - 合入后验证与活意图期间写类改动那条崩溃流这一轮写正文时还没有结局（任务 11000、11011），不当过。
- 已知红，不归这一批：可写挂载回退相关 15 条拒在 `NewerStateStillUnreadableAfterOneReread`，别的会话在修。
- 实现员「停下交主 agent 的设计问题」（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-writer/result.md` 同名一节）里主 agent 判过的：门不判挂载续做；`ForcedToStartForTests` 绕门留着，这一轮攻。
- 没定的一处（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-writer-r3/result.md` 第 3 条）：被拒的改动不清 `last_completed_intent`，低于底额时一条无产出的意图做完之后只有重挂解得开。
- 压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 1、已定项 13；`.claude/kb/decisions/16-发布语义.md` D16（发布语义） 已定项 1、2、5、17；`.claude/kb/decisions/24-后台重活能不能卸给GPU.md` D24（后台重活能不能卸给 GPU） 已定项 2；`.claude/rules/fs-design.md`「五条硬要求」「一个事务层，所有结构共用」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　过度外推与越位：运行库真的「接上了」吗
挂载之后调用方不碰后台入口，GC 是不是真自己跑、跑到腾出能开的空段；前台全部入口（`accept`、`publish_when_due`、`publish_pending_now`、`publish_user_change`、读、切换、卸载）是不是都经那把锁；`run_one_maintenance_pass` 与线程体是不是同一段代码，测试绿能不能外推到线程；真卡用例是小池、一张卡，能外推到几张卡、大盘吗；E202 与七·4 的测试走不走运行库，走不走有没有差别。

### T2　内部矛盾：回退门、余量、破例、备忘四样放一起
门数的状态（`newest_first_distinct_states`，D16 已定项 1 的「非空」，搬根那一批算一个状态）与用户定的「保住 4 个用户状态」是不是一件事；8 槽余量在 S≠8 的几何（S 是 5–8，3S=15–24）上还成不成立，会不会让一条意图永远走不完；破例不判门与余量的先后；备忘（`mounted_session.rs` 第 3055 行附近，低于底额不认停在门上的备忘）与门、破例的所有组合；挂载预算把写行、暖机算已过发布之后第一次叫起的行为；`ForcedToStartForTests` 绕过门、备忘、破例与挂载预算，现有测试因此测的是不是产品路径。

### T3　哪一条最脆弱：前台饿死、死局、停不下来
一把锁之下谁饿谁：fsync 计数在锁外的时序（计数读到 0 之后马上来 fsync）；`accept` 被后台一批挡多久；周期线程在前台持续写时拿不拿得到锁；低于底额、无产出、预算花光同时成立时的死局（第二节「没定的一处」）：判这一处要不要在这一批修，给窄改法与代价；后台批写失败的冻结（`a_frozen_publish_is_visible_to_the_foreground_and_resent_by_the_next_pass`）重发又失败时前台见到什么；卸载与线程正在持锁相撞；panic 之后每个入口的结局；Drop 只停线程与「丢最多一个 T_time + P」。

### T4　水位与默认值
E 线 5% 与「线夹到整段数」在极小盘与大盘上的行为；F、U 关着时是不是真不读；E 线高于底额 N₀ 的关系；K=16 在 S=5、S=8 上一份预算做不做得完一条意图（门留 8 槽之后）；盘上 24 字节改名之后写的字节有没有变（`system_configuration.rs`、`mount_transparency.rs`）。

### T5　开卡器与第 102 批
`gpu_opener_of_this_process.rs`：枚举工作线程超时之后实例的状态、闸住新开卡之后能不能恢复、实例上限 4 够不够、修好驱动重挂的路径（判决打中 6）；上限 0 在枚举之前判（第 102 批）与核心层校验次序的差别（空列表加上限 0 拒成哪个成员）；`the_product_hands_the_opening_the_instance_it_holds_instead_of_a_new_one` 这条钉法被攻过零轮，攻它。

### 检查单：走不到的理由二次查实
实现员报告「走不到的理由」一节原文：「无：产品代码 `crates/<包>/src/` 这一批没有新加 `todo!`、`unreachable!`、`assert!`，也没有条款没定的错误成员。」主 agent 现查到运行库新加一处 `expect`：`crates/singlefs-mount-runtime/src/lib.rs` 第 562 行 `u64::try_from(run.versions_published.len()).expect("一次调用的发布数")`。腿另查核心层与开卡器有没有写成注释的「走不到」。

### 检查单：类型与名字
新名字：`ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH`、`COMPACTION_BATCH_UNITS_BY_DEFAULT`、`CompactionBatchSize::OfTheMount`、`CompactionPublishWouldLeaveFewerThanFourRollbackStates`、`NoYieldFromTheLastIntent`、`run_one_maintenance_pass`、`gpu_opener_of_this_process`、水位判决 `WatermarkVerdict` 一族（腿现查全表）。判同一个量有没有不同语义共用裸整数、缩写、`_ =>`。

### 检查单：不兼容旧代码与旧格式
`CompactionBatchSize::NotDecidedByTheClauses` 已删；盘上 24 字节改称全 0 预留、字节不变（腿核）。腿查有没有认旧开关、旧名字的分支。

### 检查单：不新建判定库
主 agent 读补丁文件清单，没看到新的判定存放处。腿现查。

### 检查单：所有任务多机可用
真卡端到端用服务交的卡；腿查新测试与运行库有没有读本机独有状态、写死地址；钟与负载能不能换成假的。

### 检查单：测试优先多线程
新崩溃流（`crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs`）切片并行没有、线程数取登记值没有；运行库用例有真线程的那几条不并行的理由写没写。

### 检查单：带 --item 的命令与项
实现员报告的 `crash-point-replay.sh --item compaction-with-a-write-while-the-intent-is-live --item compaction-relocation`；任务 10968 退 79「指定范围（--item）没缩域，点名的 2 项这一趟要现判多少态算不出，不放行（上限 200000 态）」，主 agent 改带 `--crash-expand-up-to 4`（任务 11011）。新项登记在 `.claude/gate.d/stage-inputs.tsv` 与 `.claude/gate.d/crash-point-replay-items.tsv` 末行。腿核能被 `--list-items` 列出（读登记，不跑）。

### 检查单：新崩溃流多罩了什么
新流 `compaction-with-a-write-while-the-intent-is-live`：多记录流 → 重挂 → 整理签发 → 一批 → 活意图期间覆盖写。设计轮判它多罩「写放掉 `R` 里落点之后下一批重扫」的状态。腿核流里真有那一步，与已有的 `crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs` 比多了什么。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T5 与检查单八格，T1、T4 主攻 | 核「代码今天做的是不是判决说的」：每条定值、每个门落到文件:行；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1–T5 与检查单八格，T2、T3 主攻 | 在冻结副本的拷贝上造反例：门被绕过、余量吃掉意图、前台饿死、死局、卸载相撞、崩溃后状态少于 4 的历史；用户动作放开扫；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改；复跑脚本要能安全重跑（服务撞内存上限会重排同一条命令）。读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不写进命令）。cargo 命令都点名包。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
