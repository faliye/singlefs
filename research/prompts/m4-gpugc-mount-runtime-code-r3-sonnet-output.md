# m4-gpugc-mount-runtime-code-r3：云端正推腿（Sonnet）报告（2026-11-09）

立场：核代码今天做的是不是判决 `research/prompts/m4-gpugc-mount-runtime-code-r2-main-verification.md` 第三节八条与用户定的「accept 自己抬 F」说的；T3 主攻（报出的数能不能核）；检查单八格逐格回成立 / 不成立 / 推不出。

约定：代码引文的路径相对冻结副本根 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3/repo/`；被判五份产品文件的 sha256 与 `research/prompts/m4-gpugc-mount-runtime-code-r3-snapshot/crates-sha256.txt` 逐份相同，主工作区那五份也相同（`sha256sum` 现算，量的，各自末段 `073479…`、`a52cbf…`、`854023…`、`19c505…`、`0a1370…`）。每个数标「量的」（附取法与出处）或「推的」。读到的任务日志是 `task-server/task log <号> --all` 的最后一次尝试，落在我的草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3-sonnet/log<号>.txt`。

一句话：T3 里的数大多读得对，但有四个数的「口径」比正文写的窄（续做 137 次里真走了步的只有 28 次；门槛 128 到 256 是倍增扫描的括号、摆法与真盘位置分布不同；「删够 4 个」只有 1 与 4 两个点；任务 11441 的终态是退 1）；T1 的几条已定项在代码里站得住，新冒出来一处（上限卡在读坏槽时，`accept` 重试每次最多推 7 次不抬 F 的空发布）；T2 没有哪一条必须这一批修。

## T3　报出的数能不能核（主攻）

### 满盘上队列走不动「走不到」：`refused_walks=0` 与续做 137 次

| 数 | 取法与读数（量的） | 能不能核 | 推翻它的现象 |
|---|---|---|---|
| `refused_walks=0` | 任务 11290（最后一次尝试，本机）`grep -o 'refused_walks=[0-9]*' log11290.txt \| sort \| uniq -c`：48 行 `=0`、1 行 `=1`；那一行 `=1` 是阳性对照 `name=control_marked_full`，事件里 `Failed(DeletionContinuation(SpaceAdmissionRefused(… judged_short_by: SlotsLeftForCommitGeneratedUnits, available: AvailableBytesOnOneDevice(0) …`。48 行 = 45 行 `background_arm_summary` + 3 行 `drive_summary`。任务 11298：14 行 `=0`。 | 成立：读数存在、阳性对照读得到拒绝，读数不是「什么都读不到」 | 任一臂 `refused_walks>0`；或阳性对照 `=0` |
| 续做 `WalkedToEmpty` 137 次 | 同一份日志 137 行 `name=mount`，137 行都是 `continuation_outcome=WalkedToEmpty`。**但 `continuation_steps` 的分布：0 → 109 行、1 → 24 行、8 → 4 行**（`grep -o 'continuation_steps=[0-9]*' \| sort \| uniq -c`）。 | 数对，口径窄：109 次是挂载时队列本来就空，续做一步没走；真走了步的只有 28 次（24 + 4）。正文与判决写「137 次」当「走得动」的证据会高估 3.9 倍 | 把 137 换成 28 之后任一次 `StillShortAfterTheFloorRaises` |
| 「填到被拒时还有 3400 多个空槽」 | `name=fill_finished`：DescendingSizes 与 DescendingSizesWithCompaction `slots=3457`，ThirtyTwoUnitFiles `slots=3731`（`room=[DeviceIdentity(0):pairs=…,slots=…`） | 对 DescendingSizes 两种成立；三种合起来是 3457 到 3731，「3400 多」只对前两种 | 任一填法 `slots` 小于 256 |
| 三种填法「写满」 | 填法在**第一次建 inode 被拒就整个停**：`stopped_by=new_inode_refused`（三种都是），`last_refusal` 是 `FormulaAvailableAgainstDemand`（`available=47708`、`demand=163840`）加 `FloorAlreadyAtTheCeiling{42,42}`（DescendingSizes 两种），或 `EmptyClusterSegmentsBelowTheFloor`（`empty_cluster_segments: 0`，ThirtyTwoUnitFiles）。建 inode 被拒时整个梯子停（写被拒只换下一档），所以停下之后不再用更小的写去吃剩下的槽（探针 `queue_walk_on_a_really_full_pool.rs` 填盘函数里 `stopped_by = "new_inode_refused"` 之后 `break 'ladder`）；测试里的断言也只要求「建不出 inode」（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs` 里 `was_refused_a_new_inode`） | 「写满」= 建不出新 inode，不是「再小的写也进不去」。更小需求的写（覆写已有文件一个单元）可能还放得下，余量没有量到最低 | 每种填法被拒之后改用覆写已有一单元文件继续写到被拒，再读 `slots` 小于 256 |

### 门槛 128 到 256 与「摆法与真盘同不同口径」

量的（任务 11290，`name=walk_summary arm=sweep`）：档位 `0,1,2,4,8,16,32,64,128,256,512,1024,4096`（探针常量 `SLOTS_LEFT_OF_THE_SWEEP`），两种文件大小都是 `slots_left=128` 时 `stopped_by=a_call_published_no_deletion_step`、`slots_left=256` 时 `queue_empty`（走了 1 步）。所以 128 到 256 是**倍增扫描的括号**，分辨率 128 槽，不是量到的门槛。

摆法与真盘的差别（读探针源码 `leave_this_many_unblocked_slots`，推的）：

- 摆法从最高槽号往下把空闲槽记成已分配，只改内存里的分配位、盘上不动，剩下的是最低号的那一片；真盘空闲槽散在各处（填满之后 ThirtyTwoUnitFiles 是 `pairs=19` 对 `slots=3731`：槽多、能放数据对的极少，说明散）。门槛读的是 `unblocked_slots`（提交内生块用的没挡的槽），摆法与真盘同一个量，不同的是位置分布。
- 摆法不动 defer 的槽（扫描里 `deferred=300` 不变），真盘 `deferred` 是 83 到 500。
- 所以「摆法与真盘同口径」：量一样、分布不同，**推不出**位置分布不影响门槛；探针二换三种摆法、128 到 288 步长 8 细扫。

结构理由（推的，读码）：准入式子最后一项环窗口 `(3S+1) × W`（`crates/singlefs-core/src/admission.rs:577`「let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;」），`W` 是一次删除发布最坏新占的槽；写入类发布都要先过这个式子（`admit_on_every_device`；删除类的意图与步不判式子，只判落点），可用(d)按 `crates/singlefs-core/src/admission.rs:797` 的「容量 − 已分配 − 不可回收 − 挂载期承诺量 − 被抛弃根独占量」起头一路扣到环窗口、保留池。所以前台永远吃不到那 25 个 `W`（S = 8），队列走不动的状态（空闲槽小于一步要的）要 `W` 小于一步真要的槽才走得到。这给「外推到大盘」一个不靠扫描的理由：它比的是 `W` 对「一步真要的槽」，两者都随树高变，不随盘大小线性变。但「`W` 不小于一步真要的」本身没人量过，门槛括号 (128, 256] 与 `W` 的数没并排。探针二在抬过意图的盘上打印准入读数各项（含 `ring_window`），与细扫出的门槛并排：门槛（槽）大于 `ring_window`（字节）÷ 16384 ÷ 25 就推翻「`W` 兜得住一步」。

### 「删够 4 个」与 `available` 为负

`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs` 里 `one_unlink_stops_at_the_floor_ceiling_and_four_go_through` 的函数体：先 `unlink_cleanly(runtime, *first)`、一拍维护、`accept` 断言停在 `FloorRaiseStop::FloorAlreadyAtTheCeiling`；再删 `the_others`（`files_to_unlink(.., 4)` 切出的其余 3 个）、一拍维护、`accept` 写得进。**只有 1 与 4 两个点，没有 2、3、5**（量的：函数体全文读过）。

「4」是 D16 的 4 个状态，不是文件数：一个文件的 unlink 先发意图再走删除步，量的 `name=walk_summary arm=steps … OneUnit files=1 calls=1 deletion_steps=1`（一个一单元文件走 1 步）、`… size=Largest files=1 calls=8 deletion_steps=8`（最大文件走 8 步）；每一步都改用户可见的树，所以一个小文件的 unlink 至少发 2 个不同状态（推的，没数过根环），大文件发 9 个。`accept` 推抬 F 抬到「第 4 新的非空根」（`.claude/kb/decisions/16-发布语义.md:38` 那一行），要第 1 个文件释放的槽可再分配，需要它的释放代不高于第 4 新状态的 txg。

- 3 个行不行：**没量，推的**。按上面的数 3 个小文件发 6 个状态，第 4 新是第 2 个文件的意图，第 1 个文件的释放代在它之前，理论上够；够不够放下 `demand` 要看槽数。
- 5 个是不是也可能不行：**没量，推的**。每删一个文件先扣槽：量的 `available`，填满被拒时 `AvailableBytesOnOneDevice(47708)`（任务 11290 `fill_finished` 的 `last_refusal`），删 1 个之后 `AvailableBytesOnOneDevice(-1017100)`（任务 11320，8 处；带整理那种 `-558348`，8 处，`demand` 都是 `294912`）；差 1,064,808 字节 = 64.99 槽（按 16384 字节一槽，`admission.rs` 里 `ckpt_cost × 16384`）。两个读数来自不同任务，相减是推的。一次小文件 unlink 先扣约 65 槽、要等它的状态滑出 4 个才还，所以 `available` 随删文件数不保证单调。
- 换填法、换文件大小：测试里 `Largest`（8 步）一个文件就写得进（量的：任务 11441 的 `descending_sizes_the_largest_file_unlinked_accept_goes_through_on_the_next_pass` 在那一趟 19 条全过，见下），说明「个数」不是量，是「状态数 + 释放量」。

**主 agent 那条「只删 1 个文件写不进是 D16 的结果、不是缺陷」站不站得住**：站得住一半。`available` 的式子是九项（`crates/singlefs-core/src/admission.rs:797`「容量 − 已分配 − 不可回收 − 挂载期承诺量 − 被抛弃根独占量」，后接待删占用、已承诺预留、checkpoint 保留池、环窗口）；与 4 状态窗口直接相关的只有「已分配」里还在 defer 窗口的已释放槽，别的几项（环窗口、保留池、实例切换预留、隔离）是 D28 的固定预留，和删多少文件无关。读数：前台被拒时 `room` 里还有 `free=3392`（量的，任务 11298 的 `background_arm_summary` 行，DescendingSizes OneUnit files=1），按 16 KiB 一槽约 53 MiB，`demand=294912` 才 18 槽。所以「有没有一个不违反 D16 的历史里本可以写进去却拒了」：有，且就是这一格——物理上放得下，式子（D28 的预留）不让写，D16 自己写明的代价是「删掉的空间要再发生 3 次改变用户可见状态的发布才回可分配集合，在那之前 `df` 不把它算空闲、按 `df` 写它是真 ENOSPC」（`.claude/kb/decisions/16-发布语义.md:59`）。负的 `available` 里窗口那一项占多少、固定预留占多少，**读不出**（日志只打总数）；探针一在 `accept` 之前打 `AdmissionReading` 各项，`reading_before_the_accept=terms=[DeviceAdmissionTerms{allocated,…,ring_window}…]`。

推翻「4 恰好」：探针一里某一条历史删 3 个就写得进（则 4 是保守数），或删 5 个反而被拒（则非单调）。推翻「窗口占的」：`reading_before_the_accept` 里 `allocated` 减去按 txg 数出的活槽不足负值的绝对值。

### 余量取 S 那一组读数带不带最坏的活意图

量的（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check/run/attack_r2_probes.out`，sha256 前缀 `8d325919c277`，与判决第 15 行对上）：

- `name=margin_until_the_stop` 12 行：11 行 `live_intent=false intents_completed=1`，只有 `s=5 batch_units=4` 一行 `live_intent=true intents_completed=0`；`publishes_until_the_stop` 都是 2 或 3。
- `name=margin_crash_loop` 60 行，每次挂载推进的 txg（`txgs_advanced_by_the_mount`）：2 → 13 行、3 → 45 行、4 → 2 行。
- 代码注释写的最坏：一条整段意图 `1 + ⌈64 ÷ 16⌉ = 5` 次发布（`crates/singlefs-core/src/compaction_plan.rs:1545`），一次挂载「写行加暖机至多 3 次，续做一条意图至多 5 次，合起来至多 8 次」（同文件 1546 行）。

所以这组读数**不带最坏活意图**：最大一次挂载推进 4 个 txg、没有一次是 8；崩溃时活意图只有 1 行。S = 8 默认下 `margin_crash_loop s=8 batch_units=16`：第 1 到 4 次挂载之后 `states_after_the_mount=6`，第 5 次 `=3`（`batch_units=4` 在第 4 次就是 3、第 5 次是 1）。读法：S = 8 默认值下崩溃重挂 4 到 5 次之后掉到 4 以下，与 D26 已定项 13 第 5 条「挂载不再给新预算」同向，不是新缺口；最坏活意图那一档（每次挂载 8）没有读数，推的：掉得更快。判决里「这几条历史崩溃时都不带满额活意图，不算最坏」这句写对了，本腿认。补这一档的读数：造一条每次崩溃都带满额意图的历史（意图 5 次发布、每次挂载续做走满），读 `states_after_the_mount`；比现有读数同一次挂载的数大，就推翻「掉得更快」。

### 证红合计

量的（四个任务日志里 `抓到` / `没红` 行，我落成 `redlist.tsv`）：任务 11339 点名 20 条、抓到 19 条、`✗ … 没红` 1 条（`运行库第二轮修补 根排在最后一道屏障之后（最后一道屏障报错时根没落盘）`，任务退 1）；任务 11400 补跑那一条，抓到 1 条；任务 11405 抓到 10 条；任务 11535 抓到 8 条。19 + 1 + 10 + 8 = 38，与 `labels.txt` 里去重后的标签数 38 对上。口径：

- 这 38 是「标签 × 点名测试」行，不是 38 种改坏：`accept 抬 F：被空间拒不推抬 F（…）` 一族在 11339 里有 10 个臂标签。
- 这 38 个标签里只有 17 个是今天主工作区 `crates/mutations.tsv` 第一列的整行（`comm -12` 对 `cut -f1 crates/mutations.tsv`，量的）；21 个不在（如 `运行库第二轮修补 根排在最后一道屏障之后…` 在主工作区表里 `grep -cF` 为 0）。主工作区表里 `accept 抬 F：` 开头只有 7 行（`grep -c`），其中 `accept 抬 F：运行库不送抬 F 的事件`、`accept 抬 F：推成的那一版不接回现行那一版` 两行不在这四个任务的任何一条标签里。合入时表是不是改名、并行，**推不出**；证红合计 38 对不到主工作区表的行数，主 agent 判决里引数要说是「副本表」。
- 11339 的那一条 `没红` 是补跑 11400 才抓到的，所以「19 条」不能当 20 条点名全红。

### 任务终态：正文里引的几行绿

- **任务 11441**：背景写「✓ 测试全绿（23 个测试二进制，共 600 条测试…）✓ fmt 过」，两行在日志里（第 82、83 行）。但 `task-server/task status 11441` 的终态是 `command-failed、退 1`（第 5 次尝试，本机），同一份日志第 84 到 88 行：`✗ clippy 特性组合 default 红`、`✗ clippy 特性组合 --all-features 红`、`✗ cargo build --all-targets 红`。红因（`/home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-09-190828/clippy-default.problems.log`、`build.problems.log`，量的）：`mount_runtime_accept_raises_the_floor_for_space.rs` 524、532 行 `refused` 遮蔽，与 `crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:33` 还引着旧枚举名 `ForegroundFsync`（E0432）。这两处在冻结副本里已改（该文件 1982 行是 `AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting`），修后复核：任务 11523（clippy 四个包 `--all-targets --all-features`）退 0；任务 11519（那个测试目标 `--release`）`37 passed`。**没有哪个任务在修后的树上单独过一遍 `cargo build --all-targets`**，推的：clippy 全目标退 0 已含类型检查。
- 11441 的 `ITEM_DONE` 行 23 行，`judged` 之和 600、`reused` 之和 250（`awk` 求和，量的），与「共 600 条、复用 250」对上；`unit_read_with_one_copy_failing…` 不在这 23 项里（`grep -c` 为 0），由 11519 补。
- 任务 11532「拒算 12 条、登记表 12 行」：日志第 136 行原样。
- 背景写「没有结局」的三个任务，此刻（我读的时候）：11519 已 `succeeded`（上条），11530 `queued`，11533 `sharded`（还在跑）。崩溃流两项没有结局，下面检查单里「新崩溃流」那一格因此只能读码，不能引结局。

## T1　「已定」项撑不撑得住

逐格：定值或门 → 代码落点 → 判定（一致 / 冲突 / 规则没说）。

| 条款与定值 | 代码落点（冻结副本） | 判定 |
|---|---|---|
| D16 已定项 1「回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算」（`.claude/kb/decisions/16-发布语义.md:31`）：抬 F 的空发布占根环、不进状态 | 回退门读盘上根环，不读内存计数：「Ok(states) => (states.states_after_the_next_publish_and_the_margin」（`crates/singlefs-core/src/mounted_session.rs:4285`），数法在 `crates/singlefs-core/src/mount.rs:4260` 的 `rollback_states_of_the_ring`（按树根指针去重、空发布不算状态）。`accept` 推抬 F 的函数体（`mounted_session.rs` 6645 到 6760 行）里没有 `compaction`、`publishes_since`、`start_judgement`、`last_completed_intent` 一个字（`awk` 在 6645 到 6770 行里数这五个模式，量的：只命中我自己加进模式的 6657 行 `publishes_pushed_in_this_admission = 0`，那一行是推的数，不是整理的账） | 一致：空发布自动进门的读数，**门的计数绕不过去**，因为门不用计数；`publishes_since_the_last_user_change` 只在整理发布与重发整理那一种里加（`crates/singlefs-core/src/mounted_session.rs:3275`、`4616`），`accept` 的推不碰它 |
| 连续 `accept` 被拒、每次都推抬 F：根环与状态数怎么走（上限高于 F 的那一支） | 推一串后 F 到上限；再被拒时 `FloorAlreadyAtTheCeiling`、一串都不推：量的，任务 11290 `fill_finished` 的 `last_refusal` 里「floor_raises: [], refusals_that_pushed_the_floor_raises: [], stop: FloorAlreadyAtTheCeiling { floor: CheckpointTxg(42), ceiling: CheckpointTxg(42) }」。要再推，必须先有新状态把上限推高；每个新状态之后至多一串（2 或 3 次空发布，D16 已定项 1「正常卸载」那一行的几何，`.claude/kb/decisions/16-发布语义.md:43`）。根环 3S = 24 槽（S = 8）放得下 4 个状态各带 3 次空发布 + 状态根自己（4 × 4 = 16） | 一致。点上有用例：`the_floor_raises_of_an_accept_leave_four_user_states_in_the_root_ring`（一个点，不是连续被拒的扫） |
| 同上：上限卡在挂载时读坏或读回全零的槽上的那一支 | 「let pushed = if ceilings.are_held_by_ring_slots_bad_since_the_mount(current.root.rollback_floor)」（`crates/singlefs-core/src/mount.rs:5030`）走 `push_one_empty_publish_keeping_the_floor`：发一次**不抬 F** 的空发布；预算「publishes_pushed + 1 + 1 > PUBLISHES_PER_ADMISSION_AT_MOST;」（`crates/singlefs-core/src/mount.rs:5005`，常量 8，`crates/singlefs-core/src/mount.rs:4732`）；`accept` 每次调用从零数：「let mut publishes_pushed_in_this_admission = 0;」（`crates/singlefs-core/src/mounted_session.rs:6657`）。所以**每次被拒的 `accept` 至多推 7 次空发布、F 不动**，下一次调用又从零推，直到这个进程写掉那一槽 | 条款已认（`.claude/kb/decisions/16-发布语义.md:38`「下一次请求从零预算接着推，推到这个进程写掉那一槽」，用户 2026-11-02 定「推空」）；条款没写的是：**`accept` 这个入口也算「请求」，一个重试 ENOSPC 的调用方循环就能走到这一支**。推的、没量过：S = 8、3S = 24 次空发布把环里全部状态根换成同一个状态的空发布根，至多 ⌈24 ÷ 7⌉ = 4 次被拒的 `accept`，期间 F 不抬、候选集掉到 1 个状态。前提：挂载时有一槽读坏或读回全零、之后这个进程没写过，同时池满。规则没说（新的零轮形态，见 T2） |
| D26 已定项 13 第 5 条「挂载不再给新预算」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`）与破例「只有全空聚簇段数低于底额（已定项 3）、前台已写不进时破例，每趟至少发 1 次」 | 预算 `3S − 4 − 已过去的发布数`（`crates/singlefs-core/src/compaction_plan.rs:1562`「(3 * root_ring_slots_per_region)」起的函数）只数整理发布；破例的判 `foreground_cannot_write_under_the_switches(allocator)` 只读分配器；备忘 `StopThatPublishesNothingUntilTheNextPublish` 的键含 `checkpoint_txg`（`crates/singlefs-core/src/compaction_plan.rs:955` 起三个字段）：`accept` 推空发布让 txg 变，下一次定时兜底重判，只多判一次，不会误放行 | 一致：`accept` 的推不会让破例、备忘、预算判错；推的串不改任何一个这几处 |
| 「每次用户改动发成就清掉它」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）：`accept` 收下不算发成 | `accept` 只把改动收进待发，`note_a_user_change_published` 在排空发成时才调（重发那一支 `crates/singlefs-core/src/mounted_session.rs:3264`「KindOfTheFrozenPublish::UserChange => {」同一份账） | 一致 |
| D16 已定项 7「fsync 等系统配置轮换持久之后才返回」（`.claude/kb/decisions/16-发布语义.md:215`） | 运行库 `publish_pending_now`：「let was_frozen = session.session().allocator.frozen_publish().is_some();」（`crates/singlefs-mount-runtime/src/lib.rs:923`）、冻结着先重发；核心层 `publish_pending_now` 进门：「if let Some(frozen) = frozen_publish {」（`crates/singlefs-core/src/mounted_session.rs:6935`）拒，待发空着也不答 `None`。重发走 `persist_publish_writes(pool, &frozen.writes)`（`crates/singlefs-core/src/transaction.rs:1661`），与首次发布共用最后三步（`persist_the_root_then_rotate_the_system_configuration`，根槽 FUA → 系统配置轮换 → 屏障） | 一致：`accept` 自己写的是空发布，写失败冻结（种类 `NeitherOfTheCountedKinds`）后，`fsync` 先重发这一次、成了才排空待发、答成功；重发又报错交回错误，不答成功 |
| D16 已定项 17「报错之后不许假设可以照旧复用同一份内存态重试」（`.claude/kb/decisions/16-发布语义.md:377`）：待发放回那几条路 | `publish_the_pending_changes` 失败时：「frozen.kind() == KindOfTheFrozenPublish::UserChange);」（`crates/singlefs-core/src/mounted_session.rs:6535`）——冻结的是这一批自己才不放回；`accept` 推空发布冻结（种类 `Neither`）时待发放回，因为那一次不含待发 | 一致；这一格有变异行点它（任务 11405 抓到 `抬 F 冻结的那一次也当待发并了进去（accept 之后排空被拒）` 与 `（排空时）`） |

### 第二轮三个修补有没有被 `accept` 抬 F 破坏

- 冻结时 fsync 拒：没有（上表）；`accept` 进门冻结着照拒：「if let Some(frozen) = self.session.allocator.frozen_publish() {」（`crates/singlefs-core/src/mounted_session.rs:6568`）在推抬 F 之前，所以冻结着不推（用例 `an_accept_while_a_publish_is_frozen_is_refused_without_raising_the_floor` 任务 11441 全过）。
- 队列不空算起跑：`trigger: BackgroundCompactionTrigger::DeletionQueueNotEmpty,`（`crates/singlefs-core/src/mounted_session.rs:3393`）没被碰。一个相关事实：`accept` 在队列没走空时拒在推抬 F 之前（文档「冻结着、队列没走空时拒的是那几样，不推」），后台走队列不推抬 F（只有前台屏障类调用与挂载续做推）；所以「队列一步被空间拒、后台每拍重试、前台数据写又被拒」这个状态里没有一条路会抬 F——这是 T3 里那个「走不到」的状态，量过 48 个读数没走到（49 行 `refused_walks` 里去掉阳性对照），结构理由见上；走到了，出路是前台调一次屏障类入口或重挂。
- 重发成了照种类记账：`accept` 推空发布冻结的种类是 `Neither`，重发成了不记预算、不清完成的意图，与「推抬 F 的一次发布不算用户改动」一致。
- 没有专门用例的一支：`AcceptedButTheTriggeredPublishRefused { floor_raises_pushed_for_space, refusal }`：`grep -rn 'AcceptedButTheTriggeredPublishRefused' crates --include=*.rs` 在 `mounted_session.rs` 之外 0 处命中（量的）；字段 `floor_raises_pushed_for_space` 的取值只由 `floor_raises_pushed_by_this_accept()` 的匹配臂（`mounted_session.rs` 里）读到，没有测试走它。读码：匹配臂把它交出去、运行库 `crates/singlefs-mount-runtime/src/lib.rs:900` 记观测，没有算术，低危。

推翻 T1「一致」各格的现象：门的读数在 `accept` 推完之后与盘上根环不符（门读盘，不会）；一次被拒的 `accept` 之后 `publishes_since_the_last_user_change` 变了；冻结着时 `fsync` 答 `Ok`；上限在 F 之上的那一支连续被拒时空发布数随被拒次数增长。

## T2　未定项清单的完整性

逐条答：该进哪张表，还是这一批就要修。结论先说：没有一条必须这一批修；「`accept` 持锁时长」「崩溃循环里的边界」「重发不退避」「重发之后队列多等一周期」四行是条款没写、该进 D26 射程或欠账的；「前台 unlink 整段持锁」「`mark_allocated` 外推」两行是已有清单里的，不用新开。

| 项 | 读到什么 | 进哪张表 | 这一批修不修 |
|---|---|---|---|
| `accept` 持锁时长（前台自己发空发布）与「一批之内 GPU 持锁」放一起的最坏 | `accept` 整个调用持会话锁（`crates/singlefs-mount-runtime/src/lib.rs` 的 `accept` 在 `drop(locked)` 之前调核心层 `accept`），核心层每次调用至多推 B − 1 = 7 次空发布（`crates/singlefs-core/src/mount.rs:5005` 的预算判），每次发布四个序点（`.claude/kb/decisions/16-发布语义.md:220` 的「每次发布四个序点」），一串另多一遍系统配置；后台一批在锁里至多一次整批（含 GPU 提交与超时重交至多 3 次，`26-后台整理与放置回收.md` 已定项 13 第 4 条）。前台等的最坏 = 当时那一批 + 排在它后面的 `accept` 推的 7 次。**墙钟没量过**（accept 持锁时长在任何日志里没打）；序点数 7 × 4 = 28 加串内系统配置是推的 | D26 已定项 13 射程补一句「前台 `accept` 为空间推抬 F 时持锁至多 7 次空发布，与一批之内 GPU 持锁串行叠加」；里程碑四优化项（拆成每次空发布之间放锁，与前台 unlink 整段持锁同一条） | 不修：不违反已定项；D8「一次调用走完」 |
| `accept` 抬 F 在崩溃循环里的边界 | 上限在 F 之上的那一支：F 一到上限就不再推（T1 表），每个新状态至多一串，与 `publish_user_change` 里推的同一个函数同一个预算，槽数不比原来多。新增的只有上限卡在读坏槽的那一支（T1 表里「上限卡在读坏槽那一支」那一行）：每次被拒的 `accept` 至多 7 次不抬 F 的空发布，环被磨穿至多 ⌈3S ÷ 7⌉ = 4 次调用（S = 8，推的，没量） | D16 已定项 1 射程补一句：读坏槽卡住上限时 `accept` 入口也算「请求」；欠账登一条「读坏槽卡住上限时被拒的 `accept` 重试把环里状态磨到少于 4 个的次数」，探针见下 | 不修，标「零轮」：前提要读坏槽加池满同时成立 |
| 前台 unlink 整段持锁（判决登优化项）在抬 F 之后变没变 | 没变：unlink 走 `publish_user_change`，走队列与推抬 F 本来就在同一次调用同一把锁里（`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`）；`accept` 的屏障分支 `publish_a_barrier_alone` 走同一个函数；数据写那一支多的只有上一行那 7 次 | 优化项那一条不用改，可补一句「`accept` 推抬 F 也在其内」 | 不修 |
| `mark_allocated` 摆满才走得到的那一格（只在 320 MiB 上判走不到）外推到大盘 | T3 已答：量只在一个池（`room` 里 `slots=3457` 到 `3731`），括号 (128, 256] 粗；结构理由（环窗口 `(3S+1) × W` 兜一步）不随盘线性变，但 `W` 对一步真要的槽没有并排的数 | D26 射程已知边角一条；欠账一条「一步删除要的没挡槽数与环窗口 `W` 并排，换盘大小再量」 | 不修；探针二 |
| 重发不退避 | 判决已写进 D26 射程。这一批多一个触发源：`fsync` 每次调用持锁时冻结着就先重发一次（`crates/singlefs-mount-runtime/src/lib.rs:923`、`925`），设备持续报错时每个 `fsync` 调用都把整次发布再写一遍（写量 = 调用数 × 一次发布，R2 读数一次一单元的发布 `writes_of_the_pass=28 bytes_of_the_pass=524288`，量的，R2 判决「重发没有退避」那一行）。原位同字节，盘上状态无害 | D26 射程「重发不退避」那一条补「前台 `fsync` 每次调用也重发」 | 不修（前台本来写不进，重发越快恢复越快，同 R2 判决理由） |
| 重发成了之后队列最多等一个 `compaction_period` | 读码：线程体里重发在「不是整理到期」的那一支（`crates/singlefs-mount-runtime/src/lib.rs:666` 的 `else if !self.resend_the_frozen_publish_if_any()?`）；重发成了之后整理要等 `next_compaction_at`（`crates/singlefs-mount-runtime/src/lib.rs:660` 起，默认 30 秒，`.claude/kb/decisions/26-后台整理与放置回收.md:252` 的「间隔是挂载参数，默认 30 秒」）。这段里前台数据写被「队列没走空」拒（`AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`）。实现员报了这一条，在判决与正文的清单里没有 | D26 已定项 13 射程补一句「重发成了之后队列最坏多等一个整理周期」 | 不修：不违反条款，一个调用屏障类入口或下一拍就走 |
| 新冒出来的零轮形态（不开轮） | 读坏槽卡上限时 `accept` 重试磨环（见「`accept` 抬 F 在崩溃循环里的边界」那一行）；`fsync` 多一个重发的触发源（见「重发不退避」那一行）；重发成了之后队列多等一周期；`AcceptedButTheTriggeredPublishRefused` 带的串没有用例 | 写进判决的交用户表，标「零轮」；前三条是射程补句，带串没有用例那条是欠账一行 | — |

推翻 T2 的「不用这一批修」：量到 `accept` 持锁墙钟在默认参数下超过 `publish_poll_period` 的几倍而前台 `fsync` 延迟目标有写明数（现在条款没有数，推的）；或探针一里 `accept` 读坏槽那一支的环磨穿次数小于 4。

## 检查单

### 走不到的理由二次查实

实现员报告「无新增」与 `awk` 扫 `crates.patch` 里 `src/` 加行 0 处；我扫同一批的加行（改动全文 `research/prompts/_m4-gpugc-mount-runtime-code-r3-diff.md` 里 `+` 开头、去掉 `+++` 的 995 行，落成 `added.txt`），`grep -n 'expect(\|unwrap()\|unreachable\|走不到\|不会\|不可能\|恒\|panic\|总是\|绝不'`：

- `expect`：4 处。`crates/singlefs-mount-runtime/src/lib.rs:488`「u64::try_from(floor_raises.len()).expect("一次准入至多推几串");」与 `crates/singlefs-mount-runtime/src/lib.rs:490` 的「…expect("一次准入至多推几次空发布");」（usize 转 u64，64 位平台恒成立）；`crates/singlefs-mount-runtime/src/lib.rs:619`「.expect("一次调用的发布数")」，紧接着减 `deletion_steps`：`versions_published` 是 `walk.steps_published` 并进去的（`crates/singlefs-core/src/mounted_session.rs:4423` 「.set(u64::try_from(walk.steps_published.len()).expect("一趟的删除步数"));」之后马上 `versions_published.extend(walk.steps_published…)`），设值只此一处，不会减成负数；
- 写成注释的「走不到」：新加 1 处，`crates/singlefs-core/src/mounted_session.rs:6530`「进门之前就冻结着的那一次不含这一批：种类是用户改动的那一次冻结时待发是空的（排空发成之后、或待发并进了它），这里走不到；别的种类不是用户改动。」读码追：待发只有 `accept` 能加，而 `accept` 在冻结着时进门就拒（`crates/singlefs-core/src/mounted_session.rs:6568`）；冻结种类是 `UserChange` 的进法（待发整个并成的那次发布、屏障自己的发布、用户改动发成之后接着走的删除步）发生时待发都已清空或并进了冻结的那一次；`accept` 推空发布冻结的种类是 `Neither`，不是 `UserChange`。成立，读码推的，没有用例直接造「冻结种类是用户改动而待发非空」；
- 另一处 `crates/singlefs-core/src/transaction.rs:11957`「树表与映射树根每次发布都换，走不到这里」：是别的会话同时段的改动，当上下文，不判；
- 一处无符号减：`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:72`「.instances_created -= 1;」：守卫在 `instances_created += 1`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:210`）之后才建（213 行），建完标 `is_the_instance_made = true`（218 行），下溢要有人在中间把计数清掉；同文件 `instances_forgotten` 是另一个计数。

判定：**成立，但「无新增」不对**：新代码里有 4 个 `expect`、1 处「走不到」注释、1 处无符号减，不是 0；逐处追过都站得住。

### 类型与名字

第二轮打中的两处：枚举名与停因。现查（冻结副本 `crates/` 下 `*.rs`）：

- `grep -rnw 'ForegroundFsync' crates --include=*.rs`：0 处（量的）。含这个子串的标识符只剩 `AForegroundFsyncIsWaiting`（20 处）、`NoForegroundFsyncWaiting`（34 处）、`YieldedToAForegroundFsync`（13 处）、`ForegroundFsyncWaiting`（4 处）与三个测试里的结构名 `CallerAnsweringTheForegroundFsync`、`CallerWithNoForegroundFsync`、`CallerWithoutAForegroundFsync`（`grep -rhoE '[A-Za-z_]*ForegroundFsync[A-Za-z_]*' \| sort \| uniq -c`）。
- 停因拆开了：`YieldedToTheCallerStoppingItsThread`（8 处）与 `YieldedToAForegroundFsync` 并列，枚举 `AnswerOfTheCallerBeforeTheNextBatch` 的三个成员与 `answer_before_the_next_batch`（38 处）都在；检查单点名的其余十一个名字逐个 `grep -rnw` 命中数：`KindOfTheFrozenPublish` 26、`DeletionQueueNotEmpty` 12、`OnlyTheDeletionQueue` 5、`DeletionQueueWalkedWithoutACompactionPublish` 6、`publish_the_next_step_of_the_deletion_queue` 6、`RefusedAfterRaisingTheFloorForSpace` 11、`floor_raises_pushed_by_this_accept` 9、`floor_raises_pushed_for_space` 22、`deletion_steps_published` 7、`deletion_continuation_publishes` 6、`LimitGivenBackIfTheMakingPanics` 3（量的）。
- 低危残留：运行库里前台 `fsync` 的守卫类型还叫 `ForegroundFsyncWaiting`（`crates/singlefs-mount-runtime/src/lib.rs:402`「pub struct ForegroundFsyncWaiting<'runtime> {」），与枚举成员 `NoForegroundFsyncWaiting` 只差一个 `No`，读的人会以为是同一个东西；它表示的确实是「前台有 fsync 在等」，名字不错，只是近。`crates/mutations.tsv`（冻结副本）里 `grep -cw 'ForegroundFsync'` 是 0，含这个子串的只有 `AForegroundFsyncIsWaiting`（4 处）、`NoForegroundFsyncWaiting`（1 处）、`YieldedToAForegroundFsync`（2 处）。

判定：**成立**（两处都修到了）；低危一条见上。

### 不兼容旧代码与旧格式

- `grep -rnw ForegroundFsync`：0 处（枚举没留别名，也没有 `pub type`）。
- 加行里的历史词：`grep -n '此前\|原先\|起初\|最初\|后来\|改成了\|换成了\|修过\|旧值\|以前\|兼容\|legacy\|旧版\|旧库\|旧镜像\|迁移\|用户 2026\|第二轮\|第三轮\|判决' added.txt` 只命中 1 行：`AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement` 的文档里「判决第二节「一次发布里攒什么」那一格」（冻结副本 `crates/singlefs-core/src/mounted_session.rs` 6307 行）。这一句是存量（HEAD 里同一句有 4 处，冻结副本里也是 4 处：6307、6544、6998、7198 行），这一批改了那一行的别的字、保留了它；属 `comment-history-lag.tsv` 的存量，不是新加。删掉的加行里有「用户 2026-10-26 定」这类，已不在。
- 没有认旧版本的分支、旧布局号、迁移脚本。

判定：**成立**。

### 不新建判定库

- 加行里 `grep -n -i 'rocksdb\|sqlite\|\.tsv\|File::create\|fs::write\|OpenOptions\|std::fs\|env::var\|hostname'`：0 处。
- 新增与改动的测试 `mount_runtime_accept_raises_the_floor_for_space.rs`、`crash_enumeration_frozen_publish_resent_stream.rs`、`mount_runtime_frozen_publish_answers_and_accounting.rs`、`mount_runtime_rollback_gate_margin_follows_the_root_ring_slots.rs`、`mount_runtime_walks_the_deletion_queue_by_itself.rs`：`grep -n 'std::env\|env::var\|/home\|HOME\|std::fs\|read_to_string\|Command::new'` 只命中 `SizeOfTheUnlinkedFile::…`（模式里的 `File::` 假阳性，逐行看过），没有读文件与环境变量。
- 两条崩溃流登记在已有的 `.claude/gate.d/stage-inputs.tsv`（161、162 行，量的 `grep -nF`）与 `crash-point-replay-items.tsv`（78、79 行，末尾两行），是已有的表，不是新判定存放处。

判定：**成立**。

### 所有任务多机可用

- 新用例都在内存盘上现造、不读活树：成立（上一条的 `grep`）。没有写死主机名与路径。
- 真写满那一族内存峰值 15.11 GiB：量的，任务 11320 第 5 次尝试（本机，上限 20736M，用时 325.969 秒）；那一次退 101（当时有两条臂红，测试没跑完整，峰值只是那一次的下界，推的）；前 4 次尝试都撞上限（`memory-cap-hit`，上限 4096M、6144M、9216M、13824M，用时 112.558、71.085、81.249、147.128 秒，合计 412 秒）。19 条 `#[test]` 并行各拷一份盘（`grep -c '^#\[test\]'` 19，量的）加三份填盘的 `OnceLock`：按每份 0.7 GiB 算 22 份 ≈ 15 GiB，与 15.11 GiB 对上（0.7 是探针交接里写的数，推的）。
- 第二台能不能给到 15.2 GiB：**推不出**（第二台上最大试过 13824M 那一档，撞了；没有更大一档在第二台上的记录）。
- 要不要这一批收：**不收**，推的。要降峰值只有把同一填法的臂并成一条循环用例，那条违反 `.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」；服务按记录给上限，下一趟起步就是记下的峰值，浪费的是这一次的 412 秒。

判定：**成立**（不读活树、不绑机器）；第二台给得出 15.2 GiB 这一点**推不出**。

### 测试优先多线程

- `mount_runtime_accept_raises_the_floor_for_space.rs`：19 条 `#[test]`，`grep -n 'test-threads\|serial\|Mutex<()>'` 无命中，libtest 默认并行；三种填法各一个 `OnceLock`，首次需要的线程填、别的线程等（`filled_pool`）。「14 条臂并行」是 19 条里拷盘的臂；另 5 条也各拷一份。
- `crash_enumeration_frozen_publish_resent_stream.rs`：整盘枚举用例走 `enumerate_crash_states_with_every_kind_of_observer`（`crates/singlefs-verification/tests/crash_enumeration_frozen_publish_resent_stream.rs:428` 调它），文件头写判段的工作线程取登记值；CPU 判器小用例传 `singlefs_verification::heavy_threads::default_heavy_threads()`（544 行）。
- 线程数的效果没有量。

判定：**成立**（读码）。

### 带 --item 的命令与项

- 崩溃流两项 `frozen-publish-resent-after-a-failed-journal-write`、`frozen-publish-resent-after-a-failed-last-barrier`：`.claude/gate.d/crash-point-replay-items.tsv` 78、79 行与 `.claude/gate.d/stage-inputs.tsv` 161、162 行（量的 `grep -nF`，都是各自文件的末尾两行），`test=` 点名的用例名与流文件里的 `#[test]` 名逐字一致（`every_crash_state_of_the_resend_after_a_failed_journal_write_recovers_a_published_version_and_remounts_clean`、`…_last_barrier_…`）。
- 形式化项：这五份产品文件里 `verus!` 的 `grep -c` 都是 0（量的），但门禁按函数单元键选项、`transaction.rs` 的改动会不会进某个被验区域的闭包，要 `--list-items` 现算：**推不出**，由主 agent 列。
- 任务 11533 `sharded`（还在跑）、11530 `queued`：此刻没有结局，两条崩溃流这一格不引结局。

判定：登记**成立**；形式化项**推不出**。

### 新崩溃流多罩了什么

文件 `crates/singlefs-verification/tests/crash_enumeration_frozen_publish_resent_stream.rs`。

- **流里真有那几步**：读码。`record_the_failed_publish_and_its_resend` 摆好故障后断言发布报错、冻结、重发成了：「"落盘途中报错：那一次冻结在分配器上"」（`crates/singlefs-verification/tests/crash_enumeration_frozen_publish_resent_stream.rs:302`）、「assert!(is_resent, "冻结着一次发布");」（307 行）；最后一道屏障那一条用「.store(barriers_of_one_publish - 1, Ordering::SeqCst);」（284 行）让第 n 道屏障报错，n 是基线那次覆盖写量出来的一次发布的屏障数；录下的根槽写数断言「"失败那一遍走到根槽写的多一次"」（323 行）：日志记录写报错那条 1 次（失败那一遍没走到根槽）、最后一道屏障报错那条 2 次（失败那一遍根已写）。第二块盘日志写报错由 `next_journal_write_on_the_second_device_fails`（107 行）按日志环范围过滤。
- **甲二展开丢不丢状态**：不丢，推的、读码。理由是文件头写的那条：重发的单元写逐字节同失败那一遍，失败那一遍的单元写在已过的屏障之前，所以重发的单元写落一部分与全落读出来相同；这个前提（重发的字节与位置不变）由另一个测试目标 `publish_failure_resent_unchanged` 核，我没读那一份。日志记录写与根槽写、系统配置槽是原地写，仍按「原地写任意组合」展开；日志条那一条里第一块盘的记录在失败那一遍里没过屏障，所以落不落都在展开里。`judge_every_crash_state_of_the_resend` 里断言的是「状态数 = 按同一个展开另算的闭式」（`assert_the_counted_states_equal_the_closed_form_unless_narrowed`），它核枚举引擎的计数，不核展开够不够，展开够不够只靠上面这条理由。
- **第 115 批「不加流」够不够**：够，推的。`an_accept_writes_the_same_operations_as_the_floor_raises_of_a_publish_of_the_same_change`（`mount_runtime_accept_raises_the_floor_for_space.rs` 777 行起）断言 `accept` 的录下的写是 `publish_user_change` 的前缀，任务 11441 里那个文件 19 条全过；抬 F 那一串自己的崩溃状态有 `crash_enumeration_floor_raise_pushed_by_admission.rs`（文件头写「只录会话推的那一串抬 F」，终点是「全部持久」那一态）。`accept` 抬 F 之后待发只在内存里，崩了只剩那一串抬 F 的盘面，用户那次改动没了，与 D16 已定项 10 的「收进待发」不承诺持久一致。盘上状态里已有流没罩到的：我追到一种看似新的，追完不是。抬 F 的串做完之后、待发排空之前，盘上能不能先插进别的发布？后台入口起跑之前先排空待发（`mounted_session.rs` 里「那一趟起跑之前先把待发排空」那一句），屏障类先排空，`publish_when_due` 排空的就是待发本身，冻结着时 `accept` 进门拒；所以收下了的那次 `accept` 之后，下一个落盘的发布一定带着它，中间不会隔整理批。收下之后被拒的那种（推过抬 F、仍放不下），之后可以接整理批，但这个盘面与 `publish_user_change` 被拒在推过之后的盘面同一个，原来就有。结论：盘面序列与原来相同、只是时间上拉开，**推的：不丢状态**，没有读数。
- 结局：任务 11533 此刻 `sharded`，没有结局，这一格不引。

判定：**成立**（读码），登记与读码都对；两条流的结局没有。

## 要主 agent 跑的命令

模型：无（没有写模型目录，也没有 `tools/rerun.sh`）；要量的数是两条探针，写在冻结副本的一份拷贝里，只打 `name=` 读数、不断言结论、不写盘外的东西，重排重跑安全。

- 探针文件：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3-sonnet/probe-copy/crates/singlefs-mount-runtime/tests/probe_r3_unlink_count_and_deletion_step_threshold.rs`，sha256 `962d8de8d8d11d142d4d7f525b01aa130c51e4dcbb2311a8b3aab9f709c8f550`，1210 行：前 971 行是 `mount_runtime_accept_raises_the_floor_for_space.rs` 的逐字拷贝，后面是新加的两条探针。副本 `probe-copy` 在草稿目录 `.deliverables` 里写了「交接」。
- **没编过**：我没有起 cargo；`rustfmt --edition 2024 --check` 解析过、`^error` 0 行（只有样式差），类型对不对要编了才知道。编不过的由新派同立场的腿改探针。

**命令一**（探针一与探针二同跑；过滤词 `probe_` 只选新加的两条，拷来的 19 条不跑）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-mount-runtime 的测试目标 probe_r3_unlink_count_and_deletion_step_threshold; 范围=1 个测试目标里的 2 条探针; 为什么=量删几个文件之后 accept 才写得进与一步删除要的没挡槽数的细扫，核判决里「删够 4 个」「门槛 128 到 256」两个数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3-sonnet/probe-copy -- cargo test -p singlefs-mount-runtime --test probe_r3_unlink_count_and_deletion_step_threshold -- probe_ --nocapture
```

- 在哪起：仓根（`submit` 的 `--cwd` 是上面的副本）。
- 看哪几行判结果：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；`task-server/task log <号> --all` 里 `test result: ok. 2 passed`。`exit=1` 先找 `error[`（编不过）与 `panicked at`（`unlink_cleanly` 的「写满之后 unlink 过得去」panic 说明那一臂摆不对）。
- 预期读数行数（数不对就是没跑全）：`name=unlinks_needed` 48 行（3 种填法 × 2 种次序 × 删 1 到 8 个）；`name=deletion_step_reading_terms` 6 行（3 种摆法 × 2 种文件大小）；`name=deletion_step_threshold` 150 行（3 × 2 × 25 档，`slots_left` 96 到 288 步长 8）。
- 内存与时间（推的，没量过）：三份填盘静态各约 0.7 GiB、两条探针各同时持一份拷贝，合计约 3.5 GiB，起步上限 4096M 大概率撞一次由服务放大；时间：三种填法在任务 11290 里的 `elapsed_seconds` 是 106、52、59（量的），探针一内三种填法依次填约 217 秒，后面 48 次与 150 次各是拷盘、几次发布、一拍维护，估 10 到 15 分钟。
- 跑完用 `task-server/task clean <号>` 删编译目录；副本 `probe-copy` 由主 agent 在用完后删（`du -sh` 量过 52M，加探针 1210 行）。

怎么读（推翻条件都在这里）：

- `unlinks_needed`：每一行 `went_through=` 与 `count=`。删 1 个 `went_through=false` 且删 4 个 `true`（与测试一致）为前提；3 个就 `true` ⇒「4 恰好」被推翻、4 是保守数；有 `count` 更大而 `false` ⇒ 非单调。`reading_before_the_accept` 里 `allocated`（含 defer 窗口的已释放槽）、`ring_window`、`mount_time_commitment`、`checkpoint_reserve_pool`（在 `pool_wide`）并排，回答「`available` 为负里窗口占多少」。
- `deletion_step_threshold`：每种摆法（`order=`）、每种大小，`deletion_steps_of_the_pass` 从 0 翻成 1 的最小 `slots_left` 就是门槛，分辨率 8 槽。三种摆法的门槛相差大于 1 个档 ⇒ 摆法与真盘位置分布有关，括号 (128, 256] 不能当真盘门槛。`name=deletion_step_reading_terms` 里的 `ring_window`（字节）÷ 16384 ÷ 25 是 `W`（槽）：门槛大于 `W` ⇒ 环窗口兜不住一步，「走不到」只剩 25 倍的余量。

**读数只读、已有数的复核命令**（只读，主 agent 直接起，不交服务）：

```
D=/home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3-sonnet
grep -o 'refused_walks=[0-9]*' $D/log11290.txt | sort | uniq -c
grep 'name=mount' $D/log11290.txt | grep -o 'continuation_steps=[0-9]*' | sort | uniq -c
grep 'name=walk_summary arm=sweep' $D/log11290.txt | grep 'size=OneUnit' | cut -c1-170
grep '^ITEM_DONE' $D/log11441.txt | sed 's/.*judged=\([0-9]*\) reused=\([0-9]*\).*/\1 \2/' | awk '{j+=$1; r+=$2} END {print j, r}'
```

日志是我取的副本；主 agent 要独立核，用 `task-server/task log <号> --all` 重取。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 D16 已定项 1 保留 4 个状态、门的计数 | 一致 | 门读盘上根环不读内存计数，`accept` 推的空发布自动进门的读数；`accept` 推抬 F 的函数体不碰整理的任何账 |
| T1 连续被拒、上限在 F 之上那一支 | 一致 | F 到上限就不再推（`FloorAlreadyAtTheCeiling`），要再推得先有新状态；只有一个点的用例，没有连续被拒的扫 |
| T1 上限卡在读坏槽那一支（`accept` 重试） | 规则没说 | 每次被拒的 `accept` 至多推 7 次不抬 F 的空发布、下次从零，S = 8 时至多 4 次调用磨穿环里的状态；条款认「请求」没点 `accept`；推的，没量 |
| T1 D26 已定项 13 第 5 条破例与挂载不给新预算 | 一致 | 预算、破例、备忘、完成的意图都不被 `accept` 的推碰 |
| T1 D16 已定项 7、17（fsync、待发放回） | 一致 | 冻结着 `fsync` 先重发再排空、重发又报错不答成功；冻结的是这一批自己才不放回待发；有变异行点它 |
| T1 第二轮三个修补有没有被破坏 | 一致 | 一支（`AcceptedButTheTriggeredPublishRefused` 带串）没有用例；「队列走不动」状态里没有路抬 F，靠「走不到」 |
| T1「只删 1 个文件写不进是 D16 的结果」 | 推不出（站得住一半） | 物理上放得下（`free=3392`）、式子不让写是对的；负的 `available` 里窗口占多少读不出，探针一打各项 |
| T2 `accept` 持锁时长 + 一批之内 GPU 持锁 | 规则没说 | 进 D26 射程与优化项；不修；墙钟没量 |
| T2 崩溃循环里的边界 | 规则没说 | 上限在 F 之上的支槽数不增；读坏槽支见 T1；进 D16 射程与欠账；不修 |
| T2 前台 unlink 整段持锁 | 一致 | 没变，优化项不用改 |
| T2 `mark_allocated` 外推大盘 | 推不出 | 量只在一个池，括号粗；结构理由（环窗口）成立但 `W` 对一步没并排；探针二 |
| T2 重发不退避 | 规则没说 | 多一个触发源（`fsync`）；进 D26 射程；不修 |
| T2 重发之后队列多等一个周期 | 规则没说 | 实现员报、读码确认；进 D26 射程；不修 |
| T3 `refused_walks=0` | 成立 | 48 个 0、阳性对照 1；读数在 |
| T3 续做 137 次 `WalkedToEmpty` | 口径窄 | 真走了步的 28 次，109 次队列本来就空 |
| T3 填到被拒还有 3400 多个空槽 | 部分成立 | 3457 与 3731；「写满」= 建不出 inode，不是再小的写也进不去 |
| T3 门槛 128 到 256 | 推不出 | 倍增扫描的括号；摆法与真盘同量不同位置分布；探针二细扫 |
| T3「删够 4 个」恰好 | 推不出 | 只有 1 与 4 两个点；3 个、5 个没量，每删一个先扣约 65 槽（两份读数相减，推的） |
| T3 余量取 S 读数带不带最坏活意图 | 不带 | 12 行里 11 行没有活意图，每次挂载推进最多 4 个 txg，最坏是 8；判决那句「不带满额，不算最坏」对 |
| T3 证红合计 38 | 口径窄 | 是 19 + 1 + 10 + 8 个标签行；38 个标签里 17 个是主工作区表的整行，21 个不是；11339 有 1 条没红靠 11400 补 |
| T3 任务 11441 的绿行 | 口径窄 | 两行绿是真的，终态退 1（clippy 与 build 当时红，修后 11523、11519 过）；没有单独的 build |
| 检查单 走不到的理由 | 成立 | 「无新增」不对：4 个 `expect`、1 处「走不到」注释、1 处无符号减，逐处追过站得住 |
| 检查单 类型与名字 | 成立 | 两处都修到；守卫类型 `ForegroundFsyncWaiting` 与成员 `NoForegroundFsyncWaiting` 只差一个 `No`（低） |
| 检查单 不兼容旧代码与旧格式 | 成立 | 枚举旧名 0 处、无别名；加行里只有存量的一句「判决第二节」 |
| 检查单 不新建判定库 | 成立 | 加行与五份测试里没有文件、环境变量、判定存放 |
| 检查单 所有任务多机可用 | 成立 / 推不出 | 不读活树、不绑机器成立；第二台给不给得出 15.2 GiB 推不出；不收（原子化规则） |
| 检查单 测试优先多线程 | 成立 | 19 条并行；崩溃流用引擎线程与默认重活线程数；读码 |
| 检查单 带 --item 的命令与项 | 成立 / 推不出 | 两项登记在两张表末尾；形式化项由主 agent 现列 |
| 检查单 新崩溃流多罩了什么 | 成立 | 流里真有失败那一遍与重发那一遍同段；甲二展开不丢状态（推的，依赖另一个目标核重发字节）；不加流够；结局没有 |

## 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务、没起 cargo、没跑任何门禁阶段；判决要引的数都没复跑，凡是我在日志里读到的数标「量的」只是读了任务日志，主 agent 引之前用 `task-server/task log <号> --all` 重取。
- 探针一与探针二（`probe-copy` 里）没编过、没跑过；命令在上面「要主 agent 跑的命令」。
- 禁读清单里的攻方腿报告与模型目录没读。
- 没读 `publish_failure_resent_unchanged` 那个目标的内容（崩溃流「甲二展开不丢状态」的前提）；没读别的会话同时段在 `transaction.rs` 里的改动（只当上下文）；没逐条追开卡器 panic 退额度的行为，只在「走不到」格里看了那一处无符号减。
- 没量 `accept` 持锁墙钟；没造「读坏槽 + 池满」的探针，只给出条件与步数的算术。
- 形式化项（`.claude/gate.d/formal-verification.sh --list-items`）没列。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3-sonnet`：`probe-copy`（52M，`du -sh` 量的，交接，等主 agent 编并跑探针后删）与十来份任务日志副本（合计 2 MB 量级）、`added.txt`、`redlist.tsv`、`labels.txt`。取日志时落过一份 6 GB 的任务日志（任务 11320 的第一版测试把整个拒绝打成一行），取完核数后已删。没有编译目录。
- 报告里有一次定点替换起初用 python 就地改了我自己的报告文件（我没有 Edit 工具），之后改用了 `scripts/kb/replace-once.py`。
