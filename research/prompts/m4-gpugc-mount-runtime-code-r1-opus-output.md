# m4-gpugc-mount-runtime-code-r1：云端攻方腿报告（2026-11-09）

立场：假设运行库这一批是错的。读的是冻结副本 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1/repo/`（被引的四份产品文件与主工作区逐字节相同，`cmp` 过）。**这条腿没跑过任何代码，全部打中都是读码推的，待主 agent 跑探针。**

复跑（主 agent 交，详见「要主 agent 跑的命令」）：`task-server/task submit -- bash <模型目录拷贝>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1/repo <草稿目录>/run <模型目录拷贝>`

模型目录 `research/prompts/m4-gpugc-mount-runtime-code-r1-opus-model/` 每个文件的 sha256：

```
205ea177a3995e90b1d83713c0edee7abb19e6c9caee965eed2376ef09b28a7b  src/attack_full_pool.rs
1220f778f820d421a16cd5a252fcc99c10f5e695e287daa10b5a832d595ae74e  src/attack_gpu_opener.rs
4754a9769cdf11e962f4f580038c44ee18ef0c93f34c35b17061fe62fe4e40c1  src/attack_probes.rs
ad1949d489405e2cce10ebc91ce67fcbdf9c7d0476e687c6bc5d4c5406b013d0  tools/patch_the_fixture.py
407e31a9625e258ff43ad76278f6fcda0060cda6b06b809cbeb4fb859b72a48b  tools/rerun-from-a-clean-work-directory.sh
e0dc92a26140c3b266981fd9214c012bc0c13911b5f0bf0429168543e6ac4e7b  tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 打中（读数名） | 证据状态 |
|---|---|---|---|
| T3 | 打中，最重 | 只用 accept 写的调用方在满盘上走不出来：不低于底额不破例，accept 不推抬 F，门与预算不够把环转满（`full_pool_*`） | 推的，待主 agent 跑 |
| T2 | 打中，可能是判据的问题 | 搬根记成状态：门与挂载预算把整理搬根当用户状态，崩溃循环每轮可再涨（`gate_close`、`crash_loop`） | 推的，待主 agent 跑；D16 口径字面不违反，「用户状态」取哪个口径要定 |
| T2 | 打中 | 破例排在让路之后：低于底额时空 fsync 挡住「至少发 1 次」，两条条款相顶（`fsync_of_nothing`） | 推的，待主 agent 跑 |
| T2、T4 | 打中 | 余量只算 K=16，K 是挂载参数（`margin_sweep`） | 推的，待主 agent 跑 |
| T2、T4 | 部分打中 | S=5 门放 3 次，多于 32 单元的意图闲着做不完；「永远走不完」不成立（`geometry_sweep`） | 推的，待主 agent 跑 |
| T3 | 打中 | accept 被插队：放锁之后前台照样排不上（`entry_overtaken`） | 推的，待主 agent 跑 |
| T3 | 打中 | 破例一个周期发一次：前台约等 150 秒（`below_the_floor`） | 推的，待主 agent 跑 |
| T3、T1 | 打中 | 重发只在整理到期那一支，同步一拍的用例看不到（`frozen_resend`） | 推的，待主 agent 跑 |
| T3 | 打中，低危 | 冻结着卸载变崩溃（`unmount_frozen`）；接收方 panic 判死好会话（没写探针） | 推的 |
| T3 | 没打中（推的） | 「没定的一处」：产品 `N₀ = 1` 走不到；这一批不修，给了理由（`no_yield_sweep` 量压过 `N₀` 时的出路） | 推的，待主 agent 跑 |
| T3 | 没打中（一次抽样） | 卸载与线程相撞、线程 panic | 读码 |
| T5 | 打中 | 工作线程过期之后 panic，闸永远关着（`opener_panic_after_the_deadline`） | 推的，待主 agent 跑 |
| T5 | 打中 | 建实例挂住时 `forget`、`instances_created` 一起挂（`opener_read_while_the_maker_is_hung`） | 推的，待主 agent 跑 |
| T5、T1 | 范围窄 | 产品构造 `of_this_process` 没有调用方也没有用例；被攻的那条钉法只钉一个 `clone` | 读码 |
| T1 | 部分成立 | 线程真自跑、入口都经锁；一拍绿外推不到线程的三处；E202 不走运行库；能开的段要用户改动转环 | 读码 |
| T4 | 没打中 | 正推腿主攻，腿没逐条量 | 读码 |
| 走不到的理由 | 不成立（字面），没有走得到的（推的） | 见检查单 | 读码 |
| 类型与名字 | 不成立（两处，低危） | 见检查单 | 读码 |
| 不兼容旧代码与旧格式 | 成立 | | `grep` |
| 不新建判定库 | 成立 | | `grep`，范围见检查单 |
| 所有任务多机可用 | 成立 | 线程体睡真时间 | `grep`、读码 |
| 测试优先多线程 | 成立，一问推不出 | | 读码 |
| 带 --item 的命令与项 | 成立（登记） | 原命令不放行；上限 4 是否全展开推不出 | 读登记 |
| 新崩溃流多罩了什么 | 成立 | 不走放锁入口与门 | 读码 |

## T2：回退门、余量、破例、备忘（主攻）

下面每个打中都写「推的，待主 agent 跑」：腿没有编译与运行权限，读数在「要主 agent 跑的命令」那一条跑完之后才有。每条写明什么读数推翻它。用户动作都放开扫（批大小、S、fsync 线程数、删几个文件、无产出之后的九种动作），只固定前缀与故障。

### 搬根记成状态：门与挂载那一份预算都把「整理搬根」当一个用户状态

可达历史（`name=gate_close`、`name=crash_loop`）：

1. 一次挂载里 6 次用户改动，崩溃。用户根此刻在上一次挂载开的段里。
2. 运行库挂载（`crates/singlefs-mount-runtime/src/lib.rs:722`）。inode 根、extent 根不是钉住的角色：钉住的只有映射树、树表、实例表（`crates/singlefs-core/src/mounted_session.rs:4504`，`InodeRoot` 落在不钉那一支 `crates/singlefs-core/src/mounted_session.rs:4516`）。上一次挂载开的段不在本次 `cluster_segments`（`crates/singlefs-core/src/mounted_session.rs:5049`），选得上。
3. 一批搬了根：树表写新位置，`user_visible_trees_changed`（`crates/singlefs-core/src/mount.rs:3841`）判非空。
4. 门每次发布之前数 `distinct_states_of_the_valid_roots`（`crates/singlefs-core/src/mount.rs:4225`、`crates/singlefs-core/src/mount.rs:4291`），搬根那一批算一个状态。环里实际 3 个用户状态加 1 个搬根状态时门判够 4，照放。
5. 崩溃：挂载头一次判起跑读 `publishes_after_the_newest_state`（`crates/singlefs-core/src/mount.rs:4295`），最新状态落在搬根那一批，预算从那里数，「挂载不再给新预算」对这段历史不成立。每次挂载都有上一次挂载开的段，崩溃循环每轮都可能再搬一次根，门的计数跟着涨，真用户状态被推出环。

判别量与对照臂：同一段历史只差强制选段。对照臂取入库用例的满数据段，`user_states_left_after_the_margin` 应是 4（入库 `crates/singlefs-verification/tests/compaction_rollback_gate_keeps_four_user_states.rs:345` 钉的同一个量）；带根的段那一臂推的是 3 或更少。崩溃循环再加一臂产品选段，看 `root_relocating_publishes` 非 0 才算产品路径可达。

**推翻条件**：带根那一臂读数也是 4，或产品选段那一臂六次挂载 `root_relocating_publishes` 全是 0（那样只剩强制选段可达，降为「强制才到」）。

四句：

- 分不分辨臂：分辨「门数 D16 的状态」与「门数用户改动」两种读法；不打中所有改法。
- 系统当时看不看得到：`UserVisibleTreeRootPointers` 只存盘上原样的指针字节（`crates/singlefs-core/src/recovery.rs:4953`），只比指针看不到。搬迁写原样字节，读两边根单元的字节比得出，要多读（推的，没量）。
- 满足哪个分句：`.claude/kb/decisions/26-后台整理与放置回收.md:252`：「一次整理发布若会让根环里不同用户状态少于 4 个就不发」。D16 的认法把搬根算非空（`crates/singlefs-core/src/mount.rs:3841` 上的文档，用户 2026-10-13 定），照这个认法门的计数字面没错；入库回退门用例自己的判法（`user_states_among_the_roots_from`）按用户改动分组，两个口径不是一件事。**这一处可能是判据写错而不是代码写错**，要主 agent 或用户定「用户状态」取哪个口径。
- 改法在这几格上还中不中：改法一，门与预算读根环时比根单元内容、不比指针：这几格不再中（只在我的模型上想过、被攻过零轮，没实现，推的，待主 agent 跑）。改法二，保持 D16 口径、把余量加大：这几格照中，只推迟一格。

### 余量只算 K=16

`crates/singlefs-core/src/compaction_plan.rs:1525`：「续做一条意图在产品批大小 16 下至多」，常量写死 8（`crates/singlefs-core/src/compaction_plan.rs:1527`）。批大小却是挂载参数，`RuntimeParameters.compaction`（`crates/singlefs-mount-runtime/src/lib.rs:63`）不校验下界，挂载续做不判门（`crates/singlefs-core/src/mounted_session.rs:4269`）、预算是 `3S−4`。

历史（`name=margin_sweep`，批大小放开扫 16、8、4、2、1）：6 次用户改动，K 取小，门关时带活意图，崩溃，同一 K 再挂。推的：K=1 时续做到 20 次（S=8 的 `3S−4`），加写行暖机，环几乎整圈换掉，一次崩溃就少于 4 个用户状态；入库用例挂载都走 `mount_writable`（K=16），扫不到这一格。

**推翻条件**：`margin_sweep` 各 K 的 `states_after_the_mount` 都不少于 4。

四句：分辨臂（K=16 不中、K 小中）；系统看得到（挂载知道 K）；分句同上，且 `crates/singlefs-core/src/compaction_plan.rs:1525` 自己写明前提是 16；改法：余量按 K 算，或挂载拒绝太小的 K。按 K 算时 `3 + 1 + ⌈64/K⌉`，S=8 环 24 槽，要留 4 个状态，K 小到 4 已放不下（推的），所以两样都要：算余量并拒掉放不下的 K。这一格只在我的模型上想过、被攻过零轮。

### 余量与 S

门关的位置：4 次相邻用户改动之后，第 c+1 次整理发布要求第 4 新的状态距现行 `c+3 ≤ 3S−10`，所以门放 `3S−12` 次（推的算术；入库用例在 S=8 量过放 12 次的那一头）。

| S | 门放几次（推的） | 一条意图要几次（K=16，推的） | 判定 |
|---|---|---|---|
| 8 | 12 | 数据段 3，最坏 5 | 够 |
| 7 | 9 | 同上 | 够 |
| 6 | 6 | 同上 | 够 |
| 5 | 3 | 同上 | 数据段刚好；多于 32 个单元的段（提交内生块多的段）做不完，闲着时活意图一直挂着 |

S=5 只在 mkfs 算出槽距落在 512–614 KiB 时取到（`crates/singlefs-core/src/root_ring.rs:92`，I/O 最小单位不是 2 的幂才进这一格，推的）。「永远走不完」不成立：用户改动继续发，门跟着开。入库 `crates/singlefs-verification/tests/compaction_rollback_gate_keeps_four_user_states.rs:405` 只钉 S=8。读数 `name=geometry_sweep`。

**推翻条件**：S=5 带根段那一臂 `intents_completed` 为 1，或 `publishes_until_the_stop` 不是 3。

### 破例排在让路之后

`.claude/kb/decisions/26-后台整理与放置回收.md:252`：「破例，每趟至少发 1 次」。代码次序：先问 fsync（`crates/singlefs-core/src/mounted_session.rs:4321`）、让路就停，破例判在后面（`crates/singlefs-core/src/mounted_session.rs:4353`）。待发是空的 fsync 也算在等（`crates/singlefs-mount-runtime/src/lib.rs:839`、`crates/singlefs-mount-runtime/src/lib.rs:364`）。一拍只问一次、让路就结束这一拍（`crates/singlefs-mount-runtime/src/lib.rs:572`）。

历史（`name=fsync_of_nothing`，线程数放开扫 0、1、2、4）：低于底额，写被拒的应用反复 fsync。整理这一趟一次不发，前台等整理、整理等前台。两条条款在这一格互相顶：`.claude/kb/decisions/26-后台整理与放置回收.md:253`：「前台有 fsync 在等时不发下一批」。

**推翻条件**：4 个线程那一臂 `compaction_publishes` 与 0 个线程同一量级。

四句：分辨臂（这一格 0 线程不中）；系统看得到（低于底额与破例的判别都在同一函数里）；分句是破例那一句；改法：破例那一次发布不问 fsync（把 `crates/singlefs-core/src/mounted_session.rs:4353` 的判提到 `:4321` 之前）修这一格，代价是一次 fsync 多等一批，而那一刻写本来就被拒（推的，待主 agent 跑，被攻过零轮）。

### 备忘的组合、挂载预算、`ForcedToStartForTests`

- 备忘只放过停在门上的那一种（`crates/singlefs-core/src/mounted_session.rs:3293`）。低于底额而备忘记着 `NoEligibleSegment`、`HolesOutsideTheRegionFewerThanItsUnitsToMove`、`NoYieldFromTheLastIntent` 时不重判，要等一次发布。删除类不判合取 7，是出路（`name=no_yield_sweep` 逐动作量）。
- `ForcedToStartForTests` 绕过门、备忘、破例、从根环读预算（`crates/singlefs-core/src/mounted_session.rs:4269`）。用它的在冻结副本三个包（验证包测试、GPU 包、运行库包）里 `grep` 到 19 行、10 个文件，含 E202 装置（`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:3347`）。这些用例测的是一趟怎么推，不是产品路径。产品路径由回退门目标与运行库目标钉，两者走的是全趟入口与一拍入口，组合都盖到（`AtMostOnePublish` 加破例：第二次调用走 `crates/singlefs-core/src/mounted_session.rs:3237` 那一支，结果与全趟相同，读码推的）。

## T3：前台饿死、死局、停不下来（主攻）

### accept 被插队

`accept` 进门不登记在等（`crates/singlefs-mount-runtime/src/lib.rs:814`，登记只在 `crates/singlefs-mount-runtime/src/lib.rs:839`）。维护线程放锁之后几微秒内重新取锁（`crates/singlefs-mount-runtime/src/lib.rs:585`、`crates/singlefs-mount-runtime/src/lib.rs:518`），std 的 `Mutex` 不交接，等的线程醒来时锁多半又被拿走（对 std 锁行为的判断是读码推的，没在本项目量过）。一拍至多 64 次调用（`crates/singlefs-mount-runtime/src/lib.rs:55`），实际上界是门与预算。

历史（`name=entry_overtaken`）：真线程，后台一趟正在发，前台 `accept`。判决「每次发布之间放锁」要的是前台插得进去；设计轮量到不放锁时前台前面发了 20 次。推的：放了锁，`publishes_overtaking` 仍接近一趟的发布数。对照臂 `publish_pending_now` 应是 0–2。

**推翻条件**：`accept` 那一臂排序之后中位数不超过 2。

四句：分辨臂（fsync 臂不中）；系统看得到（登记一个计数就够）；分句是 `.claude/kb/decisions/26-后台整理与放置回收.md:252`：「前台与后台经一把锁排队」，字面只说排队、没说次序，所以这条是「放锁买不到它要买的东西」，不是违反字面；改法：全部前台入口登记一个等待计数，维护线程两次调用之间看到非 0 就让出去等它减到 0（不结束这一趟）。修这一格，代价一个原子量加几行（只在我的模型上想过、被攻过零轮，推的，待主 agent 跑）。

反向（`name=foreground_spinning`）：几个线程短持锁打转时维护线程走不走得动。没有预期结论，量了才知道。

### 只用 accept 写的调用方在满盘上走不出来（最脆弱的一格）

`accept` 收之前那一判被空间拒时不推抬 F（`crates/singlefs-core/src/transaction.rs:10326`：「被空间拒时这里不推抬 F」；`crates/singlefs-core/src/mounted_session.rs:6220` 的 `accept` 拒成 `ChangeRefusedByThePublishJudgement`），推抬 F 只在 `publish_user_change`（`crates/singlefs-core/src/mounted_session.rs:3436`）。运行库文档把 `accept` 当写入口，没写被拒之后要换入口。

历史（`name=full_pool_*`，产品 `N₀`、产品水位、一个只供测试的开关都不碰；删几个文件放开扫 1、4、16）：

1. 写满，删几个文件。放掉的槽进 defer，全空段按记录口径涨（`crates/singlefs-core/src/allocator.rs:908`），开段照旧算占着（`crates/singlefs-core/src/allocator.rs:994`）。
2. 全空段不低于 `N₀`，破例不来（`crates/singlefs-core/src/mounted_session.rs:3234`）。
3. `accept` 判落点或准入被拒，没有发布，defer 不走。
4. 维护线程：水位越线就整理，至多 12 次（门）到 20 次（预算），根环转不满一圈，defer 里的槽放不出来；之后备忘或预算挡住，不再发。
5. 前台一直被拒。只有调用方改走 `publish_user_change` 才推抬 F 走出来（`name=full_pool_publish_user_change` 量这一步）。

**推翻条件**：`full_pool_pass` 里 `accept_went_through=true` 在 40 拍内出现，或填盘先被别的拒绝挡住、`full_pool_unlink` 之后 `openable=true`。这条探针第一次跑可能要调（1 GiB 的盘腿没跑过），挂了只丢这一个目标。

四句：分辨臂（`publish_user_change` 走得出、`accept` 走不出）；系统看得到（拒绝成员就是空间不够）；分句：`.claude/kb/decisions/26-后台整理与放置回收.md:252` 的破例只管低于底额，这一格不低于底额，条款没罩到，算条款的洞；改法：甲，运行库 `accept` 被空间拒时自己转 `publish_user_change`（调用线程上推抬 F，语义与单发一样；修这一格，代价是这一次写在调用线程上多几次空发布）；乙，维护线程看到「前台被空间拒过」就推一次抬 F（要一个新计数，推抬 F 砍回退候选）；丙，把破例条件从记录口径改成开段口径（修不了只差落点那一支）。都没实现，推的，待主 agent 跑。

### 「没定的一处」：低于底额、无产出、预算花光

背景材料写「只有重挂解得开」。腿的判断（推的，没量）：产品 `N₀ = 1` 时这一格走不到。低于底额是全空段 0；一条意图做完时 R 按记录全空，全空段至少 1，不再低于底额（`crates/singlefs-core/src/compaction_plan.rs:1417` 比的是签发时的段数，无产出只说明没涨过签发时）。入库用例要 `N₀` 压到「此刻全空段加一」才造得出（`crates/singlefs-verification/tests/compaction_rollback_gate_keeps_four_user_states.rs:679`）。

`name=no_yield_sweep` 在压过的 `N₀` 上把用户动作全扫一遍（什么都不做、fsync、accept、覆盖写、建 inode、截断、unlink、建快照、崩溃再挂），看哪几种解得开。条款已写「被拒、没发成的改动不清」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`：「被拒、没发成的改动不清」）。

**判这一批修不修**：不修。窄改法（低于底额时备忘与无产出都放行）要拿「选得出的只有这一段时再搬也腾不出」换一条永远发布的循环，每 30 秒一次把回退状态全挤掉，代价大于它防的那一格。上面 accept 那一格才是同一族里产品路径走得到的死局。

**推翻条件**：造出一段 `N₀ = 1` 的历史，意图做完之后全空段仍是 0。

### 破例一个周期发一次

低于底额、预算花光：第一次发布之后这一趟按 `PublishBudgetSpent{budget:1}` 结束（`crates/singlefs-core/src/mounted_session.rs:4279`），一趟不留（`crates/singlefs-core/src/mounted_session.rs:3237` 那一支也只在有一趟时走），下一次调用定时兜底没到，这一拍结束；`is_a_trip_under_way` 是假，线程隔一个周期再来（`crates/singlefs-mount-runtime/src/lib.rs:599`）。一条意图至多 5 次发布，前台每次写都被拒，等到 5 个周期，默认约 150 秒（推的，`name=below_the_floor` 量拍数与模拟秒数）。

这几次发布都不判门：`user_states_now` 同一行打出来，看破例这一段挤掉几个。

**推翻条件**：`below_the_floor` 第一二拍 `foreground_write_went_through=true`。

四句：分辨臂（预算没花光时一拍推到门）；系统看得到（`foreground_cannot_write` 就在 `crates/singlefs-core/src/mounted_session.rs:3234`）；分句字面满足「每趟至少发 1 次」，这是条款没写「多久一趟」；改法：低于底额时线程隔 P 不隔周期，核心层定时兜底那一判也放行。代价是破例挤状态的速度从每 30 秒一次变每秒一次，总次数不变（被攻过零轮，推的，待主 agent 跑）。

### 重发只在整理到期那一支

`crates/singlefs-mount-runtime/src/lib.rs:597` 到期才走一拍（重发在一拍里，`crates/singlefs-mount-runtime/src/lib.rs:509`），`crates/singlefs-mount-runtime/src/lib.rs:606` 那一支只有 `publish_when_due`。写盘报错一次，前台 `accept`、`publish_user_change`、fsync 全拒，至多等一个周期（默认 30 秒；周期是挂载参数）。入库用例 `crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:384` 走同步一拍、手动拨 30 秒，看不到这一段：T1 问的「测试绿能不能外推到线程」在这一格不能。

重发又失败：照旧冻结，下一次再等一个周期，前台一直见 `PublishFrozenAfterAWriteFailureIsNotResentYet`，维护线程事件 `FrozenPublishResendFailed`，没有退避，也没有上报上限（读码）。

读数 `name=frozen_resend`。**推翻条件**：周期 3600 秒那一臂 `resent_after_half_a_second` 不是 0。改法：`crates/singlefs-mount-runtime/src/lib.rs:606` 那一支也调重发。冻结时才有写，代价一个判断（推的，待主 agent 跑，被攻过零轮）。

### 卸载与冻结、卸载与线程

- 卸载与线程相撞：`unmount(self)` 拿走所有权，前台入口都借着 self，`ForegroundFsyncWaiting` 也借着，类型上排除并发；线程在拿锁之外才 join（`crates/singlefs-mount-runtime/src/lib.rs:954`）。没打中（读码，一次抽样）。
- 冻结着卸载：线程已停，没人重发，待发空时直接走核心层卸载，那一步要发布（`crates/singlefs-core/src/mount.rs:4557`），推的是报 `UnmountFailed`、盘交回、不能再试；文档只在 `PendingChangesNotPublished` 那一支说可以重发。一次瞬时写错加一次卸载，干净卸载变成崩溃（不丢已确认的：冻结的那次没确认）。读数 `name=unmount_frozen`。**推翻条件**：`verdict=unmounted`。改法：卸载之前重发一次（被攻过零轮，推的）。

### panic、Drop

- 线程在锁里 panic：会话锁中毒，之后每个入口经 `crates/singlefs-mount-runtime/src/lib.rs:424` 报 `Failed`，卸载交回盘（入库两条用例钉着）。没打中。
- 接收方 panic（`deliver` 在锁外，`crates/singlefs-mount-runtime/src/lib.rs:445`）：被 `catch_unwind` 接住，同样 `note_the_failure`，会话明明完好也判死、待发不排空（读码，推的，没写探针）。接收方是 FUSE 守护进程写的，一个日志函数 panic 就丢待发。改法：`deliver` 单独 `catch_unwind`，记一次、不判坏。
- Drop「丢最多一个 T_time + P」：还要加前台等的那一段：维护线程 `publish_when_due` 也排同一把锁，一批之内 GPU 持锁（D24 已定项 2 的射程），窗口是 T_time + P + 一批，GPU 路径上一批受等待上限管（推的）。条款的口径没写这一项，交主 agent 看要不要写回。Drop 自己要等 join，一批卡住 Drop 也卡住（推的）。
- fsync 计数读到 0 之后马上来：至多等一批加两次短持锁（`crates/singlefs-mount-runtime/src/lib.rs:585` 与下一次调用），之后让路。`name=entry_overtaken` 的 fsync 臂量。

## T1：运行库真的接上了吗

- 自己跑：线程在挂载里起（`crates/singlefs-mount-runtime/src/lib.rs:671`），周期到调一拍（`crates/singlefs-mount-runtime/src/lib.rs:598`）。成立（读码；入库线程用例钉过，腿没另跑）。
- 「跑到腾出能开的空段」：要分两半。记录口径的段一条意图就涨；开段能用要 defer 过窗口，根环要转满一圈。闲置池上门放 12 次、预算 20 次，转不满 24 槽（S=8，推的）。入库用例 `maintenance_passes_on_the_product_switches_empty_a_segment_that_can_be_opened_after_the_ring_turns` 靠后面 32 次用户改动转环，用户不写时这一段走不完。不算缺陷（D2 进验收的就是这两种量），但「GC 自己腾出能开的段」只在有用户改动时成立。和上面 accept 那一格连着。
- 前台入口都经锁：`accept`、`publish_pending_now`、`publish_user_change`、切换、`with_the_session_and_devices` 都走 `lock_the_session`。成立（读码）。读路径要持这把锁（闭包），读与整理批互等；`read_at` 没有 fsync 那样的让路（推的，没量）。
- 一拍与线程体：同一段代码，差三处：周期节奏（`crates/singlefs-mount-runtime/src/lib.rs:599`，用例一律拨 30 秒，1 秒那一支没用例）；`crates/singlefs-mount-runtime/src/lib.rs:606` 那一支（重发那一格打中）；锁上的竞争（一拍用例单线程，`accept` 插队那一格打中）。绿不能外推到这三处。
- 真卡小池一张卡：外推不到几张卡、大盘。门每次发布读整环（`crates/singlefs-core/src/mount.rs:4258` 起，3S 根加每条根的树表）在一把锁里，内存盘上量不到挂钟；`name=gate_close` 打 `reads_of_the_passes` 给读次数，机械盘上一次发布多几十次随机读（推的）。开卡期限默认三张卡（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:33`），四张卡都慢时健康也拒（推的，极端）。
- E202 与七·4：不走运行库，走全趟入口加 `ForcedToStartForTests`（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/gc_call_windows.rs:223`）。差别：没有门的读、没有放锁、没有一拍上限。E202 量到的后台 CPU 时间不含门的读。结论要引到产品路径时要写这一条（推的，没量差多少）。

## T4：水位与默认值

| 问 | 判 | 依据 |
|---|---|---|
| 线夹到整段数 | 没打中 | `crates/singlefs-core/src/compaction_plan.rs:1603` 整段数 0、1 时线 0、1，整段数 1 时线等于 `N₀`，与底额同时越，不早（推的，极小盘不实用） |
| 大盘 5% | 没打中 | 读数是增量计数，O(1)（`crates/singlefs-core/src/allocator.rs:908`）。5% 是拍的；每份预算至多几条意图，大盘上越线之后追不上，属于拍的值，交实验，不算错 |
| F、U 关着真不读 | 成立 | 计数照读（`of_the_device_map`），判线不进（读码）。「不读」是不进判断，代价 O(1) |
| E 线与 N₀ | 成立 | 线下限 2 大于 1，线与底额之间那一格整理先起（读码，不随记录口径差这一格：开段那一半见 accept 那一格） |
| K=16 与 S | 见「余量与 S」 | S=5 不够，S≥6 够（推的） |
| 盘上 24 字节 | 成立 | 改名前后都是三个 `put_u64(0)`（`crates/singlefs-core/src/system_configuration.rs` 改动块），字节不变（读码；`mount_transparency` 腿没跑） |

没有比第一轮分工更新的打中。正推腿主攻这一格，腿没逐条量。

## T5：开卡器与第 102 批

### 工作线程过期之后 panic，闸永远关着

闸只在工作线程走到末尾才放开（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:215`）。到点之后调用线程已走（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:227`），`Disconnected` 那一支（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:231`）不再有人走。驱动回来时 panic（wgpu 的校验 panic 走这条路，推的）：闸一直关着，每次开卡报 `AnEarlierOpeningIsStillHung`，而工作线程早没了，文案是错的，出路只有重启进程。

读数 `name=opener_panic_after_the_deadline`。**推翻条件**：`panics=true` 那一行 `hung_after_the_worker_ended=false`。改法：工作线程里用 Drop 守卫放闸（几行，被攻过零轮，推的，待主 agent 跑）。

### 建实例挂住时，修好驱动的那一步也挂住

模块文档说建实例「驱动也可能挂在这里」，`the_instance_or_a_new_one_under_the_limit` 拿着实例锁建（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:163`）。到点拒之后，`forget_the_instance_so_the_next_opening_creates_one`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:151`）与只读的 `instances_created`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:137`）都要这把锁，调用线程一起挂住。判决打中 6 要的「修好驱动重挂」路径第一步就挂。

读数 `name=opener_read_while_the_maker_is_hung`。**推翻条件**：`returned_within_a_second=true`。改法：建实例放到锁外（两次开卡已被 `one_opening_at_a_time` 串行，建实例不用这把锁护）。

### 其余几问

- 超时之后实例状态：读不到，模块文档已写，核不动。
- 闸住之后恢复：驱动回来就恢复；驱动一直不回来，只有重启进程，`forget` 不解闸（读码）。这是判决许的。
- 实例上限 4：只有 `forget` 让计数涨，够（读码；数是拍的）。到上限是进程级不可恢复，开卡器文档写了。
- 上限 0 的次序：开卡器一支拒成 `RefusedByTheOpening`，`description` 里才有 `GpuWaitLimitOfZero`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:110`）；不给开卡器时核心层拒成 `GpuNotPresent`（`crates/singlefs-core/src/background_compute.rs:885`）。阈值配错要等开卡回来之后核心层才拒（`crates/singlefs-mount-runtime/src/lib.rs:743` 在 `:755` 之前），白等一次开卡。同一份配错，经运行库两种形状，调用方分不出「改参数」与「卡不在」（`BackgroundGpuNotOpened` 文档说成员按调用方要做的决定分）。空列表加上限 0：开卡器一支是 `GpuWaitLimitOfZero`（入库钉了）；给了实现之后核心层那一支先判卡清单、再判上限（`crates/singlefs-core/src/background_compute.rs:1024`，读码，没量）。低危。
- `the_product_hands_the_opening_the_instance_it_holds_instead_of_a_new_one`（`crates/singlefs-background-gpu/src/background_gpu.rs:4783`）：钉的是 `vulkan_instance_for_this_opening` 那个 `self.clone()`，改那一处会红；名字说「产品」，但没走 `GpuOpenerOfThisProcess`。链上真正要钉的是开卡器把实例交到 `open_background_gpu_compute`：产品构造 `of_this_process`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:72`）在冻结副本里没有调用方也没有用例（`grep` 过测试与装置），真卡用例走 `over_the_vulkan_instance`（`crates/singlefs-background-gpu/tests/mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs:158`）。这条钉法不算错，范围比名字窄：**产品构造走到真卡这一格没有测试**。改法：真卡用例改走 `of_this_process`，但它自建实例，会多建一个，受「一个进程只建一个」的量牵制（推的）。

## 检查单八格

| 格 | 判 | 依据 |
|---|---|---|
| 走不到的理由 | 不成立（字面），没有走得到的（推的） | 实现员报「无」。运行库除 `crates/singlefs-mount-runtime/src/lib.rs:562` 的 `expect`，还有两处 `panic!`：`crates/singlefs-mount-runtime/src/lib.rs:700`、`crates/singlefs-mount-runtime/src/lib.rs:966`。核心层新加 `expect`：`crates/singlefs-core/src/compaction_plan.rs:1603`、`crates/singlefs-core/src/compaction_plan.rs:1764`、`crates/singlefs-core/src/mount.rs:4300`、`crates/singlefs-core/src/mount.rs:4304`、`crates/singlefs-core/src/mount.rs:4306`。逐个读过，前提都成立：`:700` 线程起不来闭包已丢，`:966` join 过，`:1764` 在 `is_above_the_line` 之后。开卡器 `Disconnected` 文案写「panic 了」，到点之后那一支走不到它，见 T5 |
| 类型与名字 | 不成立（两处，低危） | `CallerOfTheRuntime::foreground_fsync`（`crates/singlefs-mount-runtime/src/lib.rs:363`）把「停线程」也答成 `Waiting`：同一成员两种语义，卸载时 `last_stop` 报 `YieldedToAForegroundFsync`。`WatermarkVerdict::Crossed` 的 `reading`、`line`（`crates/singlefs-core/src/compaction_plan.rs:1707`）是裸 `u64`，三个量三种单位（段、空闲段数、字节），靠文档分。新加产品代码块里没有 `_ =>`（`awk` 扫改动全文）；定时兜底间隔取周期一半（`crates/singlefs-mount-runtime/src/lib.rs:87`）有文档说明，不算 |
| 不兼容旧代码与旧格式 | 成立 | `NotDecidedByTheClauses`、`BatchSizeNotDecided`、`compaction_watermarks()` 在冻结副本 `crates/` 下 `grep` 命中 0 处。24 字节见 T4 |
| 不新建判定库 | 成立 | 新文件里 `rocksdb`、`sqlite`、`.tsv`、`prune` 命中 0 处（运行库 `lib.rs`、开卡器、运行库 `Cargo.toml`）。只核了这三份与补丁文件清单的说法 |
| 所有任务多机可用 | 成立，一处说明 | 真卡用例走服务交的卡（`handed_over_gpus_or_report_not_run`）。运行库与开卡器没有本机路径、地址（`grep` `0000:`、`/home/`、`env::var` 命中 0）。钟可换假的，但线程体睡的是真 `Duration`（`crates/singlefs-mount-runtime/src/lib.rs:618`），假钟只驱得动同步一拍，线程用例靠真时间与 300 秒防挂上限（`THREAD_DEADLINE`），派到慢的那台也不判错结局（推的） |
| 测试优先多线程 | 成立，一处推不出 | 新崩溃流用枚举库（`enumerate_crash_states_with_every_kind_of_observer`），工作线程取登记值（`crates/singlefs-verification/src/crash_state_enumeration.rs:248`）。运行库用例有真线程的那几条：文件头只写「起线程的只留几条与时间无关的」，**没写不并行的理由**；cargo 默认并行跑它们，互不共享，所以没有要写的理由，这一问推不出缺陷 |
| 带 --item 的命令与项 | 成立（登记），原命令不放行 | 新项在 `.claude/gate.d/crash-point-replay-items.tsv:77`、`.claude/gate.d/stage-inputs.tsv:158`（读登记，没跑 `--list-items`）。实现员报告的命令原样退 79（背景材料第二节观测，主 agent 改带 `--crash-expand-up-to 4`）；上限 4 那一趟是不是整条流全展开：推不出，由任务 11011 的计数行判 |
| 新崩溃流多罩了什么 | 成立 | 流里真有那一步：`crates/singlefs-verification/tests/crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs:78` 那条用例按 `OverwriteWhileTheIntentIsLive` 找位置。已有流活意图期间只有 unlink（该文件头与 `crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs` 文件头对读）。没罩的：流走显式入口（`crates/singlefs-verification/tests/common_compaction_prefix/mod.rs:396`）、批大小 2，不走运行库的放锁入口与门。门只读、不写，崩溃状态不因此多（推的）；放锁入口多的是发布之间的前台交错，流里那一次覆盖写罩的就是它的一种 |

## 没打中的形状

都是一次抽样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不当依据。取样方式都是读码，没量。

- 排空待发被拒挡住整理（待发里的改动发不出、整理要先排空）：`accept` 判过的那一批到排空之间没有别的发布（整理先排空、屏障先排空），底额与准入不翻面。试过：accept 后紧跟整理、accept 之间夹 `publish_when_due`、低于底额时 accept。走不到。
- 冻结时待发被吞（`crates/singlefs-core/src/mounted_session.rs` `publish_the_pending_changes` 冻结时不放回）：整理批冻结时待发一定是空的（同一次持锁先排空）。走不到。
- 门的 `after < now` 分句放过丢状态：此刻不足 4 个时只放不丢的那几次，试过 1–3 个状态的几种相邻形状，放的都没挤掉状态。
- 门读根环槽的编号算错：`target_for_publish` 按 txg 算，新实例 txg 跳号也照它落槽，算的是真落点。
- 整理开头那一遍删除绕门、一次调用多发：条款 `.claude/kb/decisions/26-后台整理与放置回收.md:253`：「它只约束整理批」，也写明不占预算，**照条款不算打中**。它违反的只有运行库文档「至多一次发布」的字面（`crates/singlefs-core/src/mounted_session.rs:4214`），且只在删除被空间拒留下队列时才有（unlink 那一次调用自己就走空）。大文件 unlink 一次调用挤掉整环，是前台路径本来的样子，分辨不出这一批的任何臂，另记。
- 盘表、退出次序、`Arc::try_unwrap` 的 panic、卸载返回运行库之后线程状态：读码都对得上。

## 这条腿自己的限度

- **没跑过一行**：腿没有编译权限，三份探针连编译都没验过，只过了 `rustfmt` 语法检查。第一次跑可能要改；全部读数交主 agent。上面所有「打中」此刻都是读码推出来的线索。
- 崩溃只取发布边界（丢运行库、盘留着），没有调崩溃枚举（`enumerate_crash_states` 一族）：段内写子集的崩溃状态这一腿一个都没枚举。「崩溃后状态少于 4」的结论只在发布边界上成立。门只读不写，推的是段内状态不多丢，没量；要量得在崩溃点重放里加流，这一轮没做。
- 估时：腿跑不了短样，估不出全量挂钟（推的：`attack_probes` 单线程几分钟，`attack_full_pool` 写满 1 GiB 盘两块在内存里，可能几分钟到几十分钟，内存也可能撞上限由服务放大）。估出超 40 分钟的办法：把 `FILES_AT_MOST` 或 `files_unlinked` 那一扫缩掉，只留 4。崩溃状态数：探针不枚举，全量状态数 0。
- 起线程的那几条量的是这台机器一次的锁交错，单次观测；`accept` 插队要看分布，不是一个数。
- 只读了补丁里产品代码的块、回退门用例、运行库用例与本报告引到的段落，别的崩溃流、E202 装置没通读。
- S=5 用显式 S 建的池（`from_system_configuration_field`），不是 mkfs 按槽距算出来的那条路。

## 要主 agent 跑的命令

没有一条跑过。盲写，可能编不过：编不过或要读读数，都新派同立场的腿（行首「承接：」本报告，带任务号）。

**整份复跑（两台都行，不碰卡，三个测试目标：`attack_probes`、`attack_gpu_opener`、`attack_full_pool`）**

命令原样（`<草稿目录>` 取工作根下一个新目录；模型目录照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」核查第 3 条先拷进草稿目录再跑）：

```
task-server/task submit -- bash <模型目录拷贝>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1/repo <草稿目录>/run <模型目录拷贝>
```

在哪个目录起：仓根（脚本只认参数给的路径）。服务撞内存上限重排同一条命令时，这一层看到上一次留的记号就清掉 `--work` 再跑，能安全重跑；没记号的目录不碰。

跑完看：

- `task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=`：0 是三个目标都跑完；1 是有目标编不过或探针 panic，看 `task-server/task log <号> --all` 的 `panicked at`、`error` 与 `name=rerun_exit` 那一行分得出是哪个目标。三个目标互不挡，一个挂了别的读数照出。
- 读数全在日志与 `<草稿目录>/run/readings.out`（`name=` 开头）。每条结论对哪一行，见下表。

| 结论 | 读数 | 打中的样子 | 推翻的样子 |
|---|---|---|---|
| 搬根记成状态 | `name=gate_close`、`name=crash_loop` | 带根臂 `user_states_left_after_the_margin` 少于对照臂；崩溃循环带根臂 `publishes_counted_against_the_budget` 不随挂载累积、用户状态比对照臂早少于 4；产品选段臂 `root_relocating_publishes` 非 0 | 两臂相同；产品选段六次全 0 则降为只有强制选段可达 |
| 余量只算 K=16 | `name=margin_sweep` | K 小的几行 `states_after_the_mount` 少于 4 | 每一行不少于 4 |
| 余量与 S | `name=geometry_sweep` | S=5 带根段 `live_intent=true`、四拍闲置发 0 次 | S=5 做完 |
| 破例排在让路之后 | `name=fsync_of_nothing` | 线程多的几行 `compaction_publishes` 远少于 0 线程、`yields` 高 | 同量级 |
| accept 被插队 | `name=entry_overtaken`、`name=entry_overtaken_summary` | `accept` 的排序中位数明显大于 `publish_pending_now` | 中位数不超过 2 |
| 反向饿线程 | `name=foreground_spinning` | 线程多时 `maintenance_passes` 掉到个位数 | 不掉 |
| 只用 accept 写的走不出来 | `name=full_pool_*` | 40 拍 `accept_went_through=false`，之后 `name=full_pool_publish_user_change` `went_through=true` | 40 拍内 accept 过了；或填盘挡在别处 |
| 破例一个周期发一次 | `name=below_the_floor` | 前台写得进的那一拍 `simulated_seconds` 在几十到几百秒，每拍 `publishes_of_the_pass` 是 1 | 头一两拍就写得进 |
| 无产出的出路 | `name=no_yield_sweep` | 列出哪几个动作解得开（没有预设结论） | 不适用 |
| 重发只在到期那一支 | `name=frozen_resend` | 周期 3600 秒那一行 `resent_after_half_a_second=0`、对照行非 0 | 那一行非 0 |
| 冻结着卸载 | `name=unmount_frozen` | `verdict=unmount_failed …` | `verdict=unmounted` |
| 开卡闸关死 | `name=opener_panic_after_the_deadline` | `panics=true` 那行 `hung_after_the_worker_ended=true`，`second` 是 `AnEarlierOpeningIsStillHung`；`panics=false` 那行 `false`、`second` 是 `TimedOut` | 两行相同 |
| 实例锁挂住读 | `name=opener_read_while_the_maker_is_hung` | `returned_within_a_second=false` | `true` |

模型目录：`research/prompts/m4-gpugc-mount-runtime-code-r1-opus-model/`。`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改了 `rerun_steps`。模板的 `--selftest` 腿没跑（照「子 agent 不交任务」）。要跑：`整条交 `task-server/task submit -- bash <模型目录拷贝>/tools/rerun.sh --selftest`。

## 没做什么

- 没跑任何代码，没往统一任务服务交任务。判决要引的数都没有。副本上的数不算入库装置上的数，改法都没实现、都被攻过零轮。
- 没枚举崩溃状态（「这条腿自己的限度」）。
- 不判正推、辩方那几格，不替主 agent 采纳。
- 没建仓副本、编译目录（草稿目录里只有 `.owner` 与写者锁），没有要删的。
- 「用户状态」的口径、Drop 窗口要不要多写一项、满盘 accept 选哪条改法，都要主 agent 或用户定，腿没定。
