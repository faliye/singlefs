# m4-gpugc-mount-runtime-code-r3：判决（2026-11-09）

正文 `research/prompts/_m4-gpugc-mount-runtime-code-r3-body.md`，改动全文 `research/prompts/_m4-gpugc-mount-runtime-code-r3-diff.md`，开工快照 `research/prompts/m4-gpugc-mount-runtime-code-r3-snapshot/`。这是这一题的第三轮、最后一轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpugc-mount-runtime-code-r3-sonnet-output.md`，探针 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3-main-check/probe_r3_unlink_count_and_deletion_step_threshold.rs` | T1 一致；读坏槽那一支磨环（推的）；T3 几个数的口径比正文窄 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-mount-runtime-code-r3-opus-output.md`，模型 `research/prompts/m4-gpugc-mount-runtime-code-r3-opus-model/` | 连续 accept 被拒推不停；「只删 1 个写不进」说法不全；清单外四项 |

主 agent 的核查（量的标「量的」）：

1. 快照倒推任务 11667 退 0，树在 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3-main-check/m4-gpugc-mount-runtime-code-r3-tree`。`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-mount-runtime-code-r3-opus-output.md research/prompts/m4-gpugc-mount-runtime-code-r3-sonnet-output.md --root <那棵树> --background research/prompts/_m4-gpugc-mount-runtime-code-r3-background.md` 末行「✓ 核了 49 处引文、51 处裸行号，对上 33 处，没判 67 处（其中裸行号没有引文可比 36 处；逐处列在上面）」，没有对不上的；没判的是快照树之外的路径，判决依赖的读数主 agent 现读过。
2. 正推腿探针任务 11606 退 0，日志 sha256 前缀 `223b74f4ee313a34`；攻方探针任务 11617 退 0（主 agent 拷模型到 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3-main-check/opus-model` 跑），读数 `.../run/readings.out` sha256 前缀 `2ff9d52840520b95`（量的）。两份探针腿都没编过，在主 agent 这里第一次编过、第一次出数。
3. 攻方探针到了腿设的 2400 秒预算（`name=probe_finished elapsed_seconds=2411`）：`net` 臂部分行、`steady_writes`、`steady_fill` 没跑完（`name=stopped_by_the_time_budget` 6 行）。稳态那一臂没读数，下面不据它下判。

「没打中」只各抽一份样本，只当线索。

## 二、判决

逐条，读数都是量的：

| 条 | 读数 | 判决 |
|---|---|---|
| fsync 答错（D16 已定项 7、17） | `name=fault_durability` 164 行，`violation=false` 164 行；串里写盘报错与屏障报错的位置逐个扫 | **没打中**。第二轮三处修补没被 `accept` 抬 F 破坏，限这个池与崩溃取发布边界 |
| 串断之后扣住位 | `name=fault_later_summary` 82 行全是 `went_through=8 of=8`，`unmounted=true` 82 个 | **没打中**，翻面条件「`went_through` 少于对照臂或 `unmounted=false`」不成立 |
| 连续 `accept` 被拒推不停 | `crash_loop_summary … write=NeverFits empties_in_total=186 refused_with_empties=31`；第 4 次崩溃起每次 `outcome=no_space:PublishesPerAdmissionWouldBeExceeded … series=2 empties=6`，F 从 65 一路往前 | **打中推不停**，翻面条件「`refused_with_empties` 不超过 3」不成立 |
| 状态数是 `accept` 磨掉的 | `write=Nothing`（完全不写）与 `write=NeverFits` 逐次挂载 `states` 第一项都是 4、4、4、4、1、1、1、1、1，同在第 4 次崩溃掉到 1 | **撤回**：同一段历史上差出来的是 0，掉状态是挂载自己的发布，判决 m4-gpugc-mount-runtime-code-r1 接受过的崩溃循环边界。`accept` 的代价是白写：`writes=40 barriers=16 written_bytes=500736` 一次 |
| 改法 | 用户 2026-11-09 弹窗定「改：推之前先算」：可用加 defer 里全部字节仍小于需求就不推 | 第三节。腿提的形态，**被攻过零轮**，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」另立一题 |
| 「只删 1 个写不进是 D16 的结果」 | `name=net_accept shape=DescendingSizes size=OneUnit count=1 … available_before=Some(-952248) available_after=Some(-608184) … gap_is_in_the_deferred=Some(true)` | **主 agent 的说法只对一半**。差额压在 defer 里（D16 回收门槛），差额的大小由 D28（挂载期承诺量） 已定项 1 的式子定（攻方拆账，推的）。不是缺陷。用例文档改成两半都写 |
| 「删够 4 个」 | 正推探针 `unlinks_needed`：`DescendingSizes` 与带整理的填法删 2 个就写得进（`f t t t t t t t`，两种次序相同）；`ThirtyTwoUnitFiles` 删 1 个就写得进。攻方探针同填法（挂载与拍数不同）`OneUnit` 第 4 个、`ThirtyTwoUnits` 第 3 个起写得进 | **「4」是用例选的保守数，不是门槛**。门槛随历史走，判决与文档不写死 |
| 走队列的门槛 | `deletion_step_threshold`：`HighestSlotsFirst`、`LowestSlotsFirst` 摆法 `min_ok=136 max_no=128`；`EveryFourthSlotFirst` 到 288 都走不动 | **改口径**：判决第二轮记的「128 到 256」只是连续摆法。碎片比槽数要紧；「走不到」只说「这几条填法的真盘历史上 `refused_walks=0`」，不外推到碎片盘，写进 D26 射程并登欠账 |
| 「续做 137 次 `WalkedToEmpty`」 | 正推腿数：真走了步的 28 次，109 次队列本来就空（腿量的，主 agent 没另数） | **改口径**：写回时写 28 次走了步 |
| 证红合计 | 正推腿：标签 38 行，主工作区表里 17 行在、21 行对不上 | **不是缺陷**：主 agent 按「一处改法留一行」收过表（调度记录那一节），证明会红的记录留在证红日志 11339、11400、11405、11535 |
| 读坏槽那一支磨环 | 没造故障，两腿读码不一致：正推腿判 S=8 至多 4 次调用磨穿，攻方腿判盖到的都是 F 之下的根 | **没量**：登欠账，不下判 |
| 后台停着时队列不走 | 读码，没量 | 用户定「先记账，以后定」：进 D26 未定项与欠账 |
| 持锁时长、重发不退避、前台 unlink 持锁、重发成了之后队列多等一个周期 | 读码 | 不改：进 D26 射程与里程碑四优化项 |
| 拒绝的调试文字有 GiB | 任务 11320 日志单行 1553573352 个字符（攻方腿量的） | 欠账；用例已截断 |

检查单：

| 格 | 判决 |
|---|---|
| 走不到的理由 | 成立。正推腿查到新代码 4 个 `expect`、1 处「走不到」注释、1 处无符号减，逐处追过站得住，只是报告写「无新增」不准 |
| 类型与名字 | 第二轮两处修到。攻方腿报一句事件文档与实情不符（屏障的串不报），第三节改文档；`ForegroundFsyncWaiting` 与 `NoForegroundFsyncWaiting` 只差一个 `No`，低危不改 |
| 不兼容旧代码与旧格式、不新建判定库、测试优先多线程 | 成立（两腿一致） |
| 所有任务多机可用 | 成立；真写满那一族内存峰值 15 GiB 量级，登优化项，不这一批收 |
| 带 --item 的命令与项 | 崩溃点重放项成立；形式化项主 agent 在提交前列 |
| 新崩溃流多罩了什么 | 成立；任务 11533 缩域两项不红（「缩过域、不算门禁通过」），全量归提交时 |

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-core/src/mounted_session.rs` | **要改**：第三节第 1 条 |
| `crates/singlefs-core/src/compaction_plan.rs` | **要改**：第三节第 2 条（只改文档） |
| `crates/singlefs-mount-runtime/src/lib.rs` | **要改**：第三节第 2 条（只改文档） |
| `crates/singlefs-core/src/transaction.rs` | 照旧 |
| `crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs` | 照旧 |

## 三、改法（下一批的条款）

1. 抬 F 之前先算（用户定）：`crates/singlefs-core/src/mounted_session.rs` 的 `push_one_floor_raise_for_a_refusal_for_space` 判可用加 defer 里全部字节仍小于需求就不推，报空间不够；`accept` 与 `publish_user_change` 同一处。代码轮另立一题 `m4-gpugc-floor-raise-precheck-code-r1`，快照取在改完之后。
2. 只改文档：余量算术（`crates/singlefs-core/src/compaction_plan.rs` 那一句）写明不算 `accept` 的串；运行库事件文档照实写哪几种串不报；用例 `one_unlink_stops_at_the_floor_ceiling_and_four_go_through` 的文档写「D16 回收门槛压着差额，D28 承诺量定下差额，4 是保守数」。

## 交用户的

- 「推之前先算」被攻过零轮，用户已知；另立一题走一轮。
- 后台停着时队列不走：用户定记账。
- 写满时只删 1 个文件写不进、要再发几次状态才放得出来：D16 与 D28 叠出来的，不是缺陷，写进 D26 射程。

## 这一轮被判过的文件

- crates/singlefs-core/src/mounted_session.rs：判了第三节第 1 条（推之前先算），要改
- crates/singlefs-core/src/compaction_plan.rs：判了第三节第 2 条（余量算术的文档），要改文档
- crates/singlefs-mount-runtime/src/lib.rs：判了第三节第 2 条（事件文档），要改文档
- crates/singlefs-core/src/transaction.rs：判了这一批的改动，照旧
- crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs：判了这一批的改动，照旧

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-09 不受影响：状态数掉下去是挂载的发布，条款不动；推之前先算写进射程 |
| D16（发布语义） 已定项 10 | 改了 | 2026-11-09 改了：`accept` 被空间拒时自己抬 F（用户定） |
| D26（后台整理与放置回收） 已定项 13 | 改了 | 2026-11-09 改了：写回第二轮与这一轮定的射程 |
