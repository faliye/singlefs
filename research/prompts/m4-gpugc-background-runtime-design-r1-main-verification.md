# m4-gpugc-background-runtime-design-r1：判决（2026-11-09）

正文 `research/prompts/_m4-gpugc-background-runtime-design-r1-body.md`，开工快照 `research/prompts/m4-gpugc-background-runtime-design-r1-snapshot/`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpugc-background-runtime-design-r1-sonnet-output.md`，模型 `research/prompts/m4-gpugc-background-runtime-design-r1-sonnet-model/` | 给接口草案、水位函数、挂载次序、验收表与 `crates/` 改动清单；批大小没有产品值、闲置预算死局是题面没点到的 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-background-runtime-design-r1-opus-output.md`，模型 `research/prompts/m4-gpugc-background-runtime-design-r1-opus-model/` | 打中 A1–A6、B1–B3、C1–C4、D1、D2、E1（部分）、F1；B1 最重 |

主 agent 的核查（量的标「量的」）：

1. 引文：两腿交回前各跑 `scripts/agents/cite-check.py`，末行分别「✓ 核了 63 处引文、90 处裸行号，对上 63 处，没判 90 处」「✓ 核了 65 处引文、31 处裸行号，对上 65 处，没判 31 处」（腿报告原话，主 agent 没另跑）。判决依赖的代码行主 agent 现读过：`crates/singlefs-core/src/compaction_plan.rs:429`–`:433`、`:570`（批大小）、`:1403`–`:1405`（水位）；`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:35`。设计轮不冻结代码，腿跑着的时候别的会话改过核心层，腿引的行以它们读的那一刻为准；这一轮不据此下语义判决，只定形态。
2. 攻方探针复跑，任务 10591 退 0（`~/.local/share/singlefs/work/m4-gpugc-background-runtime-design-r1-main-check/opus-model/tools/rerun-from-a-clean-work-directory.sh`，外层只多一步清草稿目录），`test result: ok. 8 passed`（量的）：
   - `name=send_bounds session=send pending_session=send file_device=send direct_device=send`；
   - `name=rollback_states wakes_after_each_change=1 … smallest_states_reachable=4`、`wakes_after_each_change=2 … smallest_states_reachable=2`；
   - `name=crash_loop_step wakes_after_each_mount=1 mount=1 compaction_publishes_of_this_mount=20 states_reachable=2`、`mount=2 … states_reachable=1`，醒 0 次那一臂全程 `states_reachable=6`；
   - `name=floor_and_budget batch_units=8 … foreground_refused_by_the_floor=40`、`batch_units=16 … foreground_refused_by_the_floor=0`；
   - `name=one_lock foreground=not_an_fsync lock_held_by_the_run_in_microseconds=79152 compaction_publishes_before_the_foreground_got_the_lock=20`、`foreground=fsync lock_held_by_the_run_in_microseconds=38`；
   - `name=wake_cost` 不发布的那几次叫起：`allocation_records=256` 4596 微秒、`=644` 5353、`=2222` 8395、`=8482` 21483。
3. 正推模型复跑，任务 10594 退 0，`name=model_summary failures=0`（量的）。其中 K 的下界是纯算术模型。
4. 没跑的：持锁各段在大盘与 GPU 模式下的挂钟（腿没写装置）；正推腿的 Send 断言（探针 `send_bounds` 已量同一件事）；真挂住的驱动（E1 核不动）。

「没打中」只各抽一份样本（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）：攻方「没打中的形状」不当依据。

## 二、判决

**用户 2026-11-09 弹窗定的四问**（问句里的事实见核查第 2 条）：

| 问 | 用户选 |
|---|---|
| 批大小 K | 「K=16 (Recommended)」：挂载参数，数是拍的 |
| 水位 | 「E 5%，F、U 先关，不存盘 (Recommended)」：E 线取整段数 5%、下限 2；F、U 代码接上、默认关；三条都是挂载参数不存盘，盘上 24 字节留作全 0 预留，字节不变 |
| 回退与预算 | 「加门：保住 4 个状态 (Recommended)」：预算照旧共用；这次整理发布会让根环里不同用户状态少于 4 个就不发；挂载不再给新预算；只有全空段低于底额、前台已写不进时破例每趟至少发 1 次 |
| 锁与 GPU | 「每次发布放锁，GPU 在锁内先接受 (Recommended)」：一批之内 GPU 仍持锁，写进 D24（后台重活能不能卸给 GPU） 已定项 2 射程并登欠账，里程碑六再拆 |

格判：

| 格 | 判决 |
|---|---|
| Q1 形态 | 采正推腿 1.1–1.5：新包 `crates/singlefs-mount-runtime`，只依赖核心层，GPU 经特质递进；攒批的会话加盘表同在一把锁；「前台有 fsync 在等」是锁外的原子计数；核心层给后台入口加「至多一次发布就返回」的跨度（正推腿 F-1 乙，用户选的放锁要它）；攒批会话补挂着切换的转发（攻方 C3）；Drop 只停线程（F-2 甲，用户没另定，取不在析构里写盘的那一支）；panic 等价崩溃。线程体之外另给同步的 `run_one_maintenance_pass`，录制盘带 `Rc<RefCell>` 过不了线程（攻方 F1），确定性测试走它 |
| Q2 水位 | 用户定值。F 的 L 没定义归 C326（条款没写），F 这一支开了要带绝对下限 |
| Q3 空转与收敛 | 两腿一致：不发布的停不扣预算、每次叫起现读（核查第 2 条 `wake_cost` 随记录数涨）。**采**正推腿的备忘：记现行 txg 与停因，txg 没变、停因是不发布的那几种就不重判。被攻过零轮。不加回滞（停机不靠水位）。攻方 A3（两次叫起之间有改动，定时那一支轮不到）、A6（循环里不再查水位）写进实现员的注意项 |
| Q4 开卡 | 开卡在挂载之前；枚举放工作线程加期限，超时之后闸住新开卡（判决 m4-gpugc-one-vulkan-instance-code-r1 打中 1）；开卡器持有进程级实例、泄漏账与实例计数，新建实例有上限（打中 6）。E1：只点一张卡也被没点名的卡拖，开卡至多约 15 分钟（推的），第一版接受、写进文档 |
| Q5 崩溃语义 | **B1 采**（用户定加门）。B3 采：后台批写失败冻结前台，运行库要把重发接通或报出去。B2：两腿不一致：正推腿判不新开流，攻方判活意图期间的写类改动、卸载带活意图再挂、排空接着整理都没有流。**判加**活意图期间写类改动一条崩溃流（多罩写放掉 `R` 里落点之后下一批重扫的状态）；另两样先加小用例，随机历史字母表加后台入口 |
| Q6 验收 | 采正推腿 6.1 与攻方 13 条的并集。D2 进验收：「腾出空段」按记录口径与开段能用两种量都核 |

被判的不是已有产品代码，没有文件要判「照旧」或「要改」；改动全部进下一批实现。

### 交用户的

- K、水位、加门、挂载不给新预算、备忘、破例这几样都是腿提的形态，被攻过零轮，下一批的代码轮照常攻。
- 全盘整理已登 D26（后台整理与放置回收） 未定项 14，不在这一批。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 1 | 改了 | 2026-11-09 改了：水位默认线与批大小照用户定值写回 |
| D26（后台整理与放置回收） 已定项 13 | 改了 | 2026-11-09 改了：第 5 条挂载就起之外，加门与挂载不给新预算写回 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-09 不受影响：加门是为了守它「回退候选集保留最近 4 个」，条款不动 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 改了 | 2026-11-09 改了：射程写明第一版一批之内持锁等 GPU，里程碑六拆到锁外 |
