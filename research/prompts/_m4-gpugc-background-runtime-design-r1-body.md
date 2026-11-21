# m4-gpugc-background-runtime-design-r1：里程碑四七节 GPU 垃圾回收，挂载就起的后台运行库与整理水位，设计轮第一轮正文（2026-11-09）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 -->

## 一、题面

用户 2026-11-08 定：「mount的时候就启动开始按照周期启动。」主 agent 提的四条答「可以按照你的说 1 2 3 4做。」：① 新开一层运行库，挂载时按参数开卡、建实例、挂载、起周期线程，前台和后台经一把锁排队，核心层照旧不起线程、不读钟；② 周期默认 30 秒，是挂载参数；③ 三条水位真正实现，给默认值，挂载可调；④ 定时兜底的预算照旧与用户改动共用。用户 2026-11-09 补：「目标是最终该功能可以正确的接入并运行。」

这一轮不判要不要做，判**怎么做才不错**，交实现员之前把形态与会错的格定下来。全盘整理不在这一轮（D26（后台整理与放置回收） 未定项，用户定这一批不做）。

这是这一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 后台入口：`crates/singlefs-core/src/mounted_session.rs` 的 `run_compaction_in_the_background`（`MountedSession` 第 2842 行、`MountedSessionWithPendingChanges` 第 6021 行）收 `devices: &mut Vec<(DeviceIdentity, Device)>`、`CompactionPacing`、`&dyn CallerOfTheBackgroundCompaction`。文档写「核心层不起线程、不读钟（钟与前台的 fsync 由 `caller` 答）」。调用方全在 E202（GPU与CPU两种模式GC对照组） 的装置（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs`）与验证包测试里，产品里没有调用方。
- 起跑判据：`crates/singlefs-core/src/compaction_plan.rs` 第 1405 行 `compaction_starts_by_the_watermarks`，文档「第一版三条水位恒 0（关），产品路径恒不起跑；只供测试的开关强制」。`CompactionRunForcing`（第 440 行）只有 `FollowTheWatermarks`、`ForcedToStopForTests`、`ForcedToStartForTests`。
- 水位的两套说法：`.claude/kb/mount-parameters.md`「整理三个水位的名字与条款对不上」：系统配置字段表写低水位、高水位、停止线三个 8 字节字段（`crates/singlefs-core/src/system_configuration.rs:369` `compaction_watermarks`，0 = 内置默认，`.claude/kb/layout/01-first-txn.md:153`）；D26（后台整理与放置回收） 已定项 1 的三水位是全空聚簇段数、碎片度 `F = runs / L`、不可回收字节，任一越线起跑，都带设备维。哪个字段对哪个量没有条款。三个量的读法：`crates/singlefs-core/src/allocator.rs` 的 `empty_segments`（第 12、320 行）、`free_runs`（第 899 行）；不可回收字节的现读腿自己查。
- 周期：`CompactionTimerFallback`（`compaction_plan.rs` 第 1305 行起），默认 `Off`；`CompactionPacing` 每次调后台入口由调用方交。`publish_when_due`（`mounted_session.rs` 第 5941 行）同样要调用方按周期调，丢失窗口的 P 归调用方（D16（发布语义） 已定项 2、5）。
- 预算：定时兜底与用户改动共用上一次用户改动给的那一份 `B_gc`，闲置时合计不过一份（后台入口文档第 4 条）。这条保证「不需要清理就不一直跑」的另一半是停机判据（D26 已定项 1 第 2 条）。
- GPU：`crates/singlefs-background-gpu` 依赖核心层，核心层不依赖它。开卡要 `VulkanInstanceOfThisProcess`（调用方持有，一进程一份，`background_gpu.rs` 第 742 行起）与 `GpuMemoryLeakedInThisProcess`。挂载入口 `mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`（`crates/singlefs-core/src/mount.rs` 第 9587 行）收 `gpu_compute: Option<BackgroundGpuComputeHandle>`。
- 判决 `research/prompts/m4-gpugc-one-vulkan-instance-code-r1-main-verification.md` 留给这一轮两条：打中 1（枚举真挂住时等待上限管不到，挂载跟着挂住；任务 10414 `FUTURE_BUILT_BEFORE_THE_DEADLINE outcome=Ok(7) elapsed_millis=300`）；打中 6（进程级实例把驱动清单冻在建它的那一刻；任务 10428 `same_instance_after_the_drivers_came_back=0 new_instance_after_they_came_back=3`，第二台的驱动上同一进程新建实例到大约第 12 个就枚举不到独显，任务 9625、9626、9650）。
- 报警：后台入口交回 `BackgroundCompactionCall`，带 `alarm_of_the_compaction_stopped_after_a_compute_error`（`mounted_session.rs` 第 341、362 行），「调用方要处理，整份丢掉编译就警告」。今天没有产品一侧的接收方。
- 产品入口：`crates/` 下没有产品程序入口，`src/bin` 只有实验与真设备装置（`find crates -maxdepth 3 -name main.rs` 零命中）。首个真正的入口是里程碑十二的 FUSE 守护进程（`.claude/kb/milestone/12-twelfth-txn.md:9`）；并行提交在里程碑六（`.claude/kb/milestone/06-sixth-txn.md`）。
- 块设备：`crates/singlefs-core/src/block_device.rs` 有 `FileBackedBlockDevice`（第 787 行）、`DirectInputOutputBlockDevice`（第 1106 行）。
- 产品 GPU 包的 `wgpu` 与真卡测试挂特性 `tests-on-a-real-gpu`（`crates/singlefs-background-gpu/Cargo.toml` 第 27 行：不开时 `cargo test` 整包与整工作区都不碰 GPU）。
- 压着的条款：D26（后台整理与放置回收） 已定项 1、已定项 13（`.claude/kb/decisions/26-后台整理与放置回收.md`）；D16（发布语义） 已定项 2、5（`.claude/kb/decisions/16-发布语义.md`）；D24（后台重活能不能卸给 GPU） 已定项 2；`.claude/rules/fs-design.md`「一个事务层，所有结构共用」「记账是事务的副产品」「五条硬要求」；`.claude/rules/verification.md`「崩溃一致性只能靠崩溃点重放验证」「所有任务都要多机可用」；`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### Q1　运行库的形态
新包放哪、叫什么、依赖方向（它依赖核心层与产品 GPU 包：不开 GPU 的调用方要不要也拖上 wgpu，怎么用特性隔开）；谁持有会话与盘表（`MountedSessionWithPendingChanges` 还是 `MountedSession`）；前台接口（`accept`、`publish_when_due`、`publish_pending_now`、`publish_user_change`、卸载、切换 `background_compute`）怎么经那一把锁；后台线程怎么停、卸载与丢弃时怎么等它；线程 panic、前台持锁太久、卸载时有活意图各怎么办。写出接口草案（类型与函数签名），并说明与核心层那些 `&mut` 入口怎么接。判一把锁够不够：一批整理持锁多久，前台会被挡多久，第 6 条「前台有 fsync 在等时不发下一批」这一判别子在一把锁下还答不答得出、由谁答。

### Q2　三条水位
三个量（全空聚簇段数、碎片度、不可回收字节）的判据写成什么函数、读哪份记账、代价随不随盘容量涨（`.claude/rules/fs-design.md` 第一格）；带设备维时「任一越线」怎么算；系统配置那三个字段（低水位、高水位、停止线）与三个量怎么对，还是这一轮把字段拿掉或改名（第一个外部用户之前格式是软的，`.claude/rules/format-evolution.md`）；哪一边是挂载参数、存不存盘；D26 已定项 1「阈值数值不进格式」与盘上字段是不是已经矛盾。每个量给候选默认值与取法，写明是拍的还是量的，以及越线太早、太晚各会怎样（空盘不该起跑、满盘不该进死循环）。

### Q3　起跑之后会不会空转或不收敛
水位越线而选不出能腾出空段的区域（全是满的段、快照钉住、不够空洞签意图）时周期每次醒来做什么；预算扣完、水位仍越线时的行为；水位在线附近反复时要不要回滞。给可达的历史。判「不需要清理就不跑」在三个量上是不是都成立。

### Q4　GPU 开卡的时序与判决留下的两条
挂载那一刻在运行库里的次序（读参数、建实例、枚举、开卡、挂载、起线程），哪一步失败落什么结局；打中 1：枚举放线程加期限的形态，超时之后那个线程怎么办、算不算把实例弄坏；打中 6：一进程一份实例与「修好驱动重挂」怎么同时成立，运行库跨多次挂载持有实例时给什么出路（第二台约 12 个实例的界不许被重建撞上）；`GpuMemoryLeakedInThisProcess` 由谁持有；`gpu` 用不上时按第 4 条报警、停下、CPU 补跑的报警送到哪。

### Q5　前台与后台交错之后崩溃语义变没变
今天的崩溃枚举与模型对拍都是单线程调后台入口。有了真线程：前台发布与后台批的交错由一把锁串成一种次序，这个次序集合是不是已有测试罩到的（`crates/singlefs-verification/tests/compaction_off_the_calling_thread_and_admission.rs`、`crash_enumeration_compaction_relocation_stream_on_the_gpu_path.rs`）；哪些交错是新的；`publish_when_due` 的周期（D16 的丢失窗口 P）与整理周期要不要同一个线程；挂载续做意图与后台线程起跑的先后；要不要加新的崩溃点重放流，多罩什么（`.claude/agents/implementation-writer.md`「做什么」第 7 步）。

### Q6　「接入并运行」怎么验才算数
用户的目标是功能正确接入并运行。列验收：挂载之后不调任何后台入口，水位越线的池子上 GC 自己跑、腾出空段、卸载干净；CPU 模式与真卡 GPU 模式写出的块序列逐字节相同（七·4 的要求，验证包不依赖产品 GPU 包，今天走的是测试实现）；钟与负载怎么换成假的（`.claude/rules/fs-design.md`「每条分支必须能被测试强制进入」）；每条必红世界；两台都跑得起。判七·4、七·5 现有的测试与 E202 要不要改接运行库。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | Q1–Q6，Q1、Q2 主攻 | 给一份能照着写的形态：接口草案、水位函数、挂载次序、验收清单；每条落到 `crates/` 的文件与行；代价能量的写成模型与命令 |
| 云端攻方（Opus） | Q1–Q6，Q3、Q5 主攻 | 假设「一把锁加周期线程」是错的：造前台被饿死、不收敛空转、交错之后崩溃语义变了、卸载与活意图相撞的可达历史；由用户决定的动作放开扫 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写主工作区路径与行号（设计轮没有冻结副本，读的时候核一下那一行还在）。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷一份副本，不在主工作区改。读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不写进命令）。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 水位默认值与岔路写成候选加代价，不替用户定数。
- 回复与报告用中文。
