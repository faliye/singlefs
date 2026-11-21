# m4-gpugc-mount-runtime-code-r1：判决（2026-11-09）

正文 `research/prompts/_m4-gpugc-mount-runtime-code-r1-body.md`，改动全文 `research/prompts/_m4-gpugc-mount-runtime-code-r1-diff.md`，开工快照 `research/prompts/m4-gpugc-mount-runtime-code-r1-snapshot/`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-output.md`，模型 `research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-model/` | T1 部分成立；8 槽余量在 S≤5 不成立；删除走空那一遍整段持锁；检查单两格不成立 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-mount-runtime-code-r1-opus-output.md`、`research/prompts/m4-gpugc-mount-runtime-code-r1-opus-s2-output.md`，模型同名目录 | 打中九条加低危两条，判最重的是只用 accept 写的走不出来；检查单两格不成立 |

主 agent 的核查（量的标「量的」）：

1. 开工快照：实现合入之后主工作区的被判文件腿跑着的时候没被这一轮改；腿引的行以冻结副本 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1/repo` 为准。两腿交回前各跑 cite-check，末行分别「✓ 核了 5 处引文、105 处裸行号，对上 5 处，没判 105 处」「✓ 核了 8 处引文、94 处裸行号，对上 8 处，没判 94 处」（腿报告原话）。判决依赖的是下面量到的读数。
2. 攻方探针任务 11092（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/opus-model/`），`attack_probes` 11 条、`attack_gpu_opener` 2 条跑完（量的）；`attack_full_pool` 建池炸，第二条同立场腿修过，任务 11116 退 0（量的）。读数在同目录 `run/readings.out`、`s2-run/readings.out`。
3. 正推模型任务 11091 退 0（量的），其余读数未引。

## 二、判决

逐条，读数都是量的：

| 条 | 读数 | 判决 |
|---|---|---|
| 只用 accept 写的走不出来 | 六组 `full_pool_pass` 全在 `pass=1`、`total_publishes` 2–10 时 `accept_went_through=true`；`accept_before_unlink_went_through=false` 三组 | **撤回**，攻方报告写在跑前的翻面条件「40 拍内过了，且过之前 `total_publishes` 非 0」打中。运行库在满盘上救得出来 |
| 余量与几何 | `geometry_sweep s=5 … publishes_until_the_stop=0`，`s=6` 3–4 次，`s=7` 7–8 次，`s=8` 4 次到无产出停 | **打中**。用户 2026-11-09 弹窗定「余量取 S 槽 (Recommended)」 |
| 破例排在让路之后 | `fsync_of_nothing threads=0 compaction_publishes=10`，`threads=1` `compaction_publishes=0 yields=950` | **打中**。低于底额的破例改排在让路之前：前台已写不进，让路换不来什么 |
| 重发只在到期那一支 | `frozen_resend compaction_period_seconds=3600 resent_after_half_a_second=0 accept_after_half_a_second_went_through=false`，周期 0 那一行 1、`true` | **打中**。每一拍先重发冻结的发布，与整理到不到期无关 |
| 冻结着卸载 | `unmount_frozen … verdict=unmount_failed RaiseFloorSequencePublishFailed(… PublishFrozenAfterAWriteFailureIsNotResentYet` | **打中**。卸载先试一次重发；还失败就交回错误，文档写明等价崩溃 |
| 开卡闸关死 | `opener_panic_after_the_deadline panics=true … hung_after_the_worker_ended=true second=Some(AnEarlierOpeningIsStillHung)` | **打中**。工作线程 panic 也要放闸 |
| 实例锁挂住读 | `opener_read_while_the_maker_is_hung … returned_within_a_second=false`（两行） | **打中**。建实例不持那把锁 |
| 搬根记成状态 | `gate_close` 四臂 `user_states_left_after_the_margin=6`；`crash_loop` 带根臂比对照臂早一次挂载掉（第 4 次 2 个、对照第 5 次 3 个） | **照旧**，门取 D16（发布语义） 已定项 1 的「非空」，用户定过搬根算非空（`crates/singlefs-verification/tests/compaction_batch_relocating_a_user_visible_root_is_non_empty.rs:5`）。写进 D26（后台整理与放置回收） 射程 |
| 崩溃循环总有尽头 | `crash_loop` 三臂整理只在前两次挂载发（4–6 次），第 5、6 次挂载降到 3、1 | **用户接受的边界**（问句里写明）。余量撑的是有限次连崩 |
| accept 被插队 | `entry_overtaken_summary entry=accept … [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2]`，中位数 1 | **撤回**（翻面条件「中位数不超过 2」） |
| 反向饿线程 | `foreground_spinning threads=8 maintenance_passes=1900` | **撤回** |
| 破例一个周期发一次 | `below_the_floor pass=1 simulated_seconds=30 publishes_of_the_pass=1` | **不改**：前台可达的情形由满盘探针量过第 1 拍就过；登进下面的去向 |
| 无产出的出路 | `no_yield_sweep`：`crash_and_mount_again` 之后发 2 次；`accept_overwrite`、`publish_*` 之后 `NoEligibleSegment` | **这一批不修**：`N₀ = 1` 产品走不到（攻方报告）；留作交用户那一行 |
| 删除走空整段持锁 | 正推腿读码，`crates/singlefs-core/src/mounted_session.rs` 第 4207–4240 行，没量 | **采**：不守判决 m4-gpugc-background-runtime-design-r1「每次发布之间放锁」；窄改法见正推腿报告，配会红的用例 |
| 产品构造 `of_this_process` 没调用方没用例 | 正推腿与攻方腿都报，读码 | **采**：配一条用例；另补删掉那一行变异的缺口（把手递给挂载） |
| 真卡不能外推到线程 | 读码，真卡用例走同步一拍 | **采**：加真线程、真卡加前台并发的用例，挂 `tests-on-a-real-gpu`；起线程的用例核「腾出能开的段」 |
| E202 不走运行库 | 读码 | **照旧**：实验要强制选段；写进 E202 页不接运行库的理由，七·5 收口时再判 |

检查单：

| 格 | 判决 |
|---|---|
| 走不到的理由 | **不成立**：运行库 2 处 `panic!`、核心层 5 处 `expect` 没写进报告「走不到的理由」。修补那一批逐处列、判走不到或改成错误成员 |
| 类型与名字 | **不成立**：`WatermarkVerdict::Crossed { reading, line }` 三种单位共用 `u64`，改成各自的类型；`Waiting` 兼表停线程改名 |
| 不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、带 --item、新崩溃流多罩了什么 | 成立（两腿一致，读码） |
| 测试优先多线程 | 运行库四条真线程用例补一句不并行的理由 |

kb：正推腿报两处还写旧名字（D22（单元原子性怎么合成） 已定项 26、`.claude/kb/decisions/13-验证路线.md` 第 48 行），并进下一次写回。

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-mount-runtime/src/lib.rs` | **要改**（重发、卸载、不让路的判法调用、用例） |
| `crates/singlefs-core/src/compaction_plan.rs` | **要改**（余量取 S、`WatermarkVerdict`） |
| `crates/singlefs-core/src/mounted_session.rs` | **要改**（破例排在让路之前、删除走空放锁） |
| `crates/singlefs-core/src/mount.rs` | **要改**（门读槽数的余量） |
| `crates/singlefs-core/src/system_configuration.rs` | 照旧 |
| `crates/singlefs-core/src/allocator.rs` | 照旧（只改注释） |
| `crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs` | **要改**（放闸、不持锁建实例） |
| `crates/singlefs-background-gpu/src/background_gpu.rs` | 照旧 |
| `crates/singlefs-background-gpu/src/lib.rs` | 照旧 |
| `crates/singlefs-background-gpu/src/adapter_inventory.rs` | 照旧（只改注释） |
| `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs` | 照旧（只改文档） |

打中语义的格有，修完开第二轮，攻这一轮的修补。

### 交用户的

- 余量取 S、破例不让路、每拍先重发都是腿提的形态，被攻过零轮，第二轮攻。
- 无产出死局（`no_yield_sweep`）这一批不修：产品 `N₀ = 1` 走不到，写进 D26 射程。

## 这一轮被判过的文件

- crates/singlefs-mount-runtime/src/lib.rs：判了重发、卸载、让路判法的调用与用例，要改
- crates/singlefs-core/src/compaction_plan.rs：判了余量取 S 与 `WatermarkVerdict`，要改
- crates/singlefs-core/src/mounted_session.rs：判了破例与让路的次序、删除走空放锁，要改
- crates/singlefs-core/src/mount.rs：判了门读槽数的余量，要改
- crates/singlefs-core/src/system_configuration.rs：判了这一批的改动，照旧
- crates/singlefs-core/src/allocator.rs：判了这一批的改动（只改注释），照旧
- crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs：判了放闸与建实例时的锁，要改
- crates/singlefs-background-gpu/src/background_gpu.rs：判了这一批的改动，照旧
- crates/singlefs-background-gpu/src/lib.rs：判了这一批的改动，照旧
- crates/singlefs-background-gpu/src/adapter_inventory.rs：判了这一批的改动（只改注释），照旧
- crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs：判了这一批的改动（只改文档），照旧

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 改了 | 2026-11-09 改了：第 5 条余量取 S 槽，破例排在让路之前，崩溃循环的边界写进射程 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-09 不受影响：门取它的「非空」口径，条款不动 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 不影响 | 2026-11-09 不受影响：设计轮已写进射程，这一轮没再动 |
