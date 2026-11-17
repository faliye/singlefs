# m4-close-batch2-code-r2：攻方承接腿报告，B2 改法的 A/B（Opus，2026-11-10）

<!-- doc-lint:not-numbers R -->

立场：攻方承接（承接 `research/prompts/m4-close-batch2-code-r2-opus-output.md`）。攻击面只有一样：判决 `research/prompts/m4-close-batch2-code-r2-main-verification.md` 第三节第 1 条的零轮改法（B2 与 H3 同一处改），照 `.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做` 里零轮改法先做 A/B 那一条，造两臂探针。不攻别的面。方案按 `crates/` 今天的实现谈：主工作区六份被判产品文件与 `research/prompts/m4-close-batch2-code-r2-snapshot/crates-sha256.txt` 逐份 `OK`（量的，`sha256sum -c`，本腿开工时）。

我什么都没编、没跑（子 agent 不交任务）。下面的读数全是推的，等主 agent 跑出「要主 agent 跑的命令」那一条再定。本腿原型跑的崩溃状态数：0（没写调枚举函数的原型，这一题不问崩溃）。

## 复跑

模型目录 `research/prompts/m4-close-batch2-code-r2-opus-s2-model/`。复跑命令（整条交服务，见「要主 agent 跑的命令」）：
`bash research/prompts/m4-close-batch2-code-r2-opus-s2-model/tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录（工作根下、以 -run 收尾）> <模型目录>`。
外层（与上一份同一份文件，sha256 相同）删掉那一个运行目录，再起同目录的 `tools/rerun.sh`（拷自 `scripts/agents/three-way-rerun-template.sh`，只换了 `rerun_steps` 那一段）。`rerun_steps` 在同一个任务里先后做两臂，每臂各建一份副本：

1. 经 `minimal-copy.py` 从冻结副本拷一份，核六份产品文件与开工快照的 sha256（`name=m4_close_batch2_r2_ab_snapshot arm=<臂> rc=`）。
2. 乙臂（`with-the-patch`）在副本里 `patch -p1` 打 `patches/stopped-drop-reads-the-mount-budget-and-judges-the-gate.patch`，再核打完的 `mounted_session.rs` 与 `patches/patched-sha256.txt` 相等（`name=m4_close_batch2_r2_ab_patch arm=with-the-patch rc=`）。甲臂（`without-the-patch`）不打。快照或补丁核不过的那一臂不编不跑，报 `rc=skipped`。
3. 两份副本放进同一份探针 `probes/attack_ab_stopped_drop_judges_the_rollback_gate.rs` 当新测试目标，只跑它；观测行前面加 `arm=<臂>` 落进运行目录的 `observations.txt`；副本当场删。
4. 末两行按跑前写下的期望对一遍：`name=m4_close_batch2_r2_ab_verdict …`、`name=m4_close_batch2_r2_ab_mount_budget …`。退出码只说两臂跑没跑成（0 = 两臂都编成、跑完、前提断言过），读数结局看这两行。

```
b2a2c9307c22576c8c0615d8fd01d5b99f0ce4380d26b2694093c696bdaeb0a9  patches/patched-sha256.txt
9a3a365d425d75848e306b131376febe42e1bd40ec7c693390aea3c69fc1a7c2  patches/stopped-drop-reads-the-mount-budget-and-judges-the-gate.patch
9c8af718ac878ddf1fa88958869f237c7c810e8626de41662349da35ec77a755  probes/attack_ab_stopped_drop_judges_the_rollback_gate.rs
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
2bf3d1d0c26657b50006b11779aa3f49bdde0799daafd58b4087efd871b22e7e  tools/rerun.sh
```

本腿在草稿目录里核过的（量的）：补丁在主工作区那一份 `mounted_session.rs` 的拷贝上 `patch -p1 --forward` 退 0、打完 sha256 与 `patched-sha256.txt` 对上；打完的文件与探针都过 `rustfmt --edition 2021 --check`（探针配两份空占位模块，只解析、不编）；`rerun.sh` 与外层过 `bash -n`；`rerun.sh` 里读汇总字段的 `ab_field` 拿造出来的两行观测行试过，字段名是别的字段名前缀时取得对（`histories_with_the_intent_gone` 与 `…_gone_after_a_wake_held_at_the_gate`）、找不到时给空、不让脚本因 pipefail 退出。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| A/B 装置（第三节第 1 条改法） | 建好，待主 agent 跑 | 补丁、同一份探针、两臂各一份副本、同一批输入（门前用户改动 4、6、8 × 门关之后三种 × 门开前后的叫起） | 模型目录；没编过 |
| 乙臂门关时发不发（期望，推的） | 不发 | 停着那一趟单发删意图之前判门，门关着就停在门上：`publishes_of_the_wake=0`、意图留着 | 读补丁 |
| 乙臂门开之后删不删得掉（期望，推的） | 删得掉 | 门关的停不进 `start_while_the_compaction_stays_stopped` 的判，每次叫起都重判门；用户改动攒够、门开那一次删掉 | 读码 |
| 甲臂（期望，推的） | 照任务 12314 复现 | 切换的两种头一次叫起都发 1 次、意图没了；对照臂不发 | 任务 12314 第 4 次尝试日志 |
| H3 读数（期望，推的） | 乙臂头一次停着的后台调用就读根环，甲臂不读 | 观测行 `mount_budget_first_stopped_call` 写出调用前后的已过发布数 | 读补丁 |
| 补丁的副作用（推的，交主 agent 看） | 两处 | 门关而停着时每次叫起都读一遍根环（备忘不挡这一支）；停着时头一次后台调用读根环，「开头那几判一个盘都不读」那句文档串更不对 | 读码 |

## A/B：判决第三节第 1 条的改法

### 补丁（只在副本里打，主工作区不动）

`patches/stopped-drop-reads-the-mount-budget-and-judges-the-gate.patch`，对 `crates/singlefs-core/src/mounted_session.rs` 两处：

1. H3：后台入口判起跑那一段里，停没停那一判「        if compaction_stays_stopped(allocator, background_compute) {」（`crates/singlefs-core/src/mounted_session.rs:3394`）连同它的 `return Ok(start_while_the_compaction_stays_stopped(…))` 挪到挂载那一份预算读根环「            && compaction.take_the_reading_of_the_publishes_of_the_mount_from_the_ring()」（`crates/singlefs-core/src/mounted_session.rs:3409`）那一段之后。`start_while_the_compaction_stays_stopped` 判「扣完」用的就是读过根环、取过 max 的已过发布数。
2. B2：一趟开头停着那一支「    if compaction_stays_stopped(allocator, background_compute) {」（`crates/singlefs-core/src/mounted_session.rs:4505`）里，预算不为 0 时先判回退门（`stop_of_the_rollback_gate`，与循环里「        if is_the_rollback_gate_judged && !is_this_publish_the_exception {」（`crates/singlefs-core/src/mounted_session.rs:4728`）同一个函数），门关着就以门的停收这一趟、0 次发布、不删意图，意图与 R 排他都不动。判不判门的条件照循环那一处：`is_the_rollback_gate_judged` 那一句「let is_the_rollback_gate_judged = matches!(pacing, PacingOfTheRun::PacedByTheCaller { .. })」（`crates/singlefs-core/src/mounted_session.rs:4627`）连同注释整段挪到停着那一支之前（循环里照用同一个变量）；破例同循环里每趟的第一次发布：`budget_rule` 是全空段低于底额、前台写不进那一档时不判。

没改的：停着那一趟的别的行为（预算为 0 不发、删成之后记账、出错冻结），触发名与文档串（`crates/singlefs-core/src/mounted_session.rs:516` 「/// [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；删完之后的调用照旧不起跑。预算照后台入口别的起跑给：那一份扣完而前台写得进就不起跑。」，最后一句打补丁之后在门上也成立了，句子本身没动），挂载续做与显式入口（都是 `PacingOfTheRun::NotPaced`，不判门，同 `crates/singlefs-core/src/mount.rs:8288` 「回退门不判这一趟：续做只推盘上那一条、不签新的，发布数有上界（1 + ⌈64 ÷ 批大小⌉）。」）。

### 探针与喂的输入（两臂同一份）

`probes/attack_ab_stopped_drop_judges_the_rollback_gate.rs`，两个测试：

- `ab_the_stopped_drop_at_a_closed_rollback_gate`：前缀与上一份探针 `attack_stopped_drop_skips_the_rollback_gate.rs` 相同（另一次挂载里顺序写 40 个数据单元；重挂选 `gpu`、两张卡都用得上；开关照水位判、越线靠只供测试的开关、选段强制、一批一个；建 `user_changes_before` 个 inode；定时兜底叫起到门关）。门前用户改动取 4、6、8 三档；门关之后三种：切到一张卡都用不上再叫起（`SwitchToNoUsableGpuThenWake`）、切换之后先建一个 inode 再叫起（`SwitchToNoUsableGpuThenAUserChangeThenWake`）、不切换再叫起（`WakeWithoutASwitch`，对照）。比上一份多的：切换的两种在头一次叫起之后不改动再叫起一次，再一次建一个 inode、每建一个叫起一次，直到意图没了或门关之后建满 8 个。
- `reading_of_the_publishes_of_the_mount_at_the_first_background_call_left_stopped`（H3）：同一前缀（4 个 inode）推到门关、意图活着时丢掉会话；在同一组盘上重挂（两张卡都用得上，挂载续做照常推那一条意图）；切到一张卡都用不上；读头一次后台调用之前（挂载交回时）与之后会话记的已过发布数，另写探针自己数的「最后一次用户改动之后根环里读得出、不低于 F 的根数」。

盘是内存稀疏盘：起点（顺序写之后）经 `build_pool` 建一次、把录制流施加到内存镜像上，每段历史从那一份 `MemoryPool` 拷两块 `SparseBlockDevice`（上一份探针每段历史各建一份文件镜像）。判门的独立读法照上一份：根环里读得出、不低于 F 的根按「用户改动」分状态；产品按用户可见树根指针分状态（整理搬了树根单元的那几版也算一个状态，任务 12314 里产品报 `states_now: 6` 时探针数 4），两种读法在门的边上可能不一致，观测行把两样都写出来（探针的 `gate_refuses_by_the_probe` 与产品停在哪一条的 `outcome`）。

### 跑前写下的期望（推的）与推翻条件

观测行（每行前面有 `arm=<臂> m4-close-batch2-r2-ab:`）：

- `stopped_drop_gate_wake`：每次叫起一行，字段 `wake=`（`first_after_the_closure`、`again_without_a_change`、`after_one_more_user_change`）、`user_changes_after_the_closure`、`states_now_before`、`states_left_past_the_next_publish_and_the_margin_before`、`gate_refuses_by_the_probe`、`started`、`trigger`、`outcome`、`publishes_of_the_wake`、`states_left_past_the_margin_after`、`live_intent_after`。
- `stopped_drop_gate`：每段历史一行，头一次叫起的 `publishes_of_the_wake`、`states_left_past_the_next_publish_and_the_margin`、`live_intent_after_the_wake`，与 `user_changes_until_the_intent_is_gone`（门关之后第几次用户改动之后意图没了，`never` 是建满 8 个还在）、`wakes_publishing_past_a_closed_gate`。
- `stopped_drop_gate_ab_summary`：每臂一行，九个计数（见 `rerun.sh` 末段）。
- `mount_budget_first_stopped_call`：H3，每臂一行。

| 臂 | 期望 | 推翻条件 |
|---|---|---|
| 甲（不打补丁） | 三档都判到（`geometries_judged=3`）；切换的两种头一次叫起 `trigger=LiveIntentWhileStoppedAfterAComputeError`、`publishes_of_the_wake=1`、`live_intent_after=false`，探针读法认为门关的那几次都算越门（`first_wakes_publishing_past_a_closed_gate>=3`）；对照臂 `publishes_of_the_wake=0`、`outcome=NotStarted(UnchangedSinceAStopThatPublishedNothing …)` | 与任务 12314 那九行对不上（换了内存盘之后读数变了，要先查装置） |
| 乙（打补丁）门关时 | 切换的两种头一次叫起 `started=true`、`outcome=Stopped(CompactionPublishWouldLeaveFewerThanFourRollbackStates { … })`、`publishes_of_the_wake=0`、`live_intent_after=true`；不改动再叫起一次同样 0 次 | 头一次叫起仍发（`first_wakes_publishing_past_a_closed_gate>0`）：改法没碰到这一格 |
| 乙（打补丁）门开之后 | 建 inode 到门开（每档推的要 1 到 4 个：门关之后建满 4 个，余量之外那几槽之后的根全是用户状态），那一次叫起 `publishes_of_the_wake=1`、`live_intent_after=false`、`outcome=Stopped(BackgroundTaskStoppedAfterAnEarlierComputeError)`；`histories_left_with_the_intent=0` | 建满 8 个仍 `never`：改法卡死 |
| 对照臂（两臂） | 两臂逐字相同（补丁不碰没停着的那一支） | 两臂不同：补丁碰到了别的路 |
| H3 | 甲臂 `publishes_since_the_last_user_change_after_the_first_call` 等于 `publishes_counted_at_the_mount`；乙臂不小于它，根环里比最新状态新的根多于续做那几次时更大 | 乙臂也等于、而探针数的根数明显更大：读根环那一段没走到（或产品认的最新状态是续做那一版，见下） |

`rerun.sh` 末行 `name=m4_close_batch2_r2_ab_verdict` 照上表前三行机算：`without_the_patch_reproduces`、`with_the_patch_holds_at_a_closed_gate`（三档都判到、门关的头一次叫起至少 3 次、一次都不发、每次都留着意图）、`with_the_patch_drops_once_the_gate_opens`（没有一段留着意图、至少一段是先被门挡住再删的）。

### 打中之后先答的四句（对这一组 A/B）

- 分不分辨臂：分辨。同一段前缀、同一刻（门关、意图活着、切到零卡），甲臂发、乙臂不发；对照臂两臂同，说明差只来自停着那一支。
- 系统当时看不看得到判别它的东西：看得到。停着那一支手里有参数、盘表、分配器、现行那一版，`stop_of_the_rollback_gate` 要的四样都在，补丁就是在那里调它。
- 满足的是判据字面的哪一个分句：D26（后台整理与放置回收） 已定项 13 第 5 条管回退门的那一分句（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 252 行；原文判决第一节已抄、主 agent 已现读，这里不另摘）；破例分句（全空段低于底额、前台写不进）在这组输入里不成立，补丁里照留。
- 改法在打中的格上还中不中：推的不中，等乙臂读数。

### 用户动作放开扫了哪些、没扫哪些

扫了：门前用户改动 4、6、8；门关之后切换与否、切换之后先改动与否；门关之后再叫起而不改动；之后用户改动 1 到 8 次、每次之后叫起。固定的：前缀（顺序写 40 个数据单元、强制选段、一批一个、照水位判）与「门关」这一刻；用户改动只用「建一个 inode」一种；后台入口只用整趟那一种（`run_compaction_in_the_background`，`SpanOfTheBackgroundCall::WholeTrip`），放锁的后台入口（一次调用至多一次发布）没扫，它走同一个 `run_compaction_on_with_the_region_check_of_the_intent_deletion`，补丁在那一支里，推的同样生效；攒批会话没扫。破例那一档（前台写不进）没造。

### H3：挂载那一份预算在停着的头一次后台调用时读不读得到

补丁第 1 处之后，挂载之后头一次判后台起跑就读根环（`publishes_of_the_mount_are_still_to_be_read_from_the_ring` 起步是真，「            publishes_of_the_mount_are_still_to_be_read_from_the_ring: true,」（`crates/singlefs-core/src/compaction_plan.rs:1008`）），不管停没停。探针那一行写出：`publishes_counted_by_the_previous_mount`（丢掉会话之前上一次挂载记的数）、`publishes_counted_at_the_mount`（重挂交回时会话记的数，就是挂载续做那几次）、`roots_after_the_last_user_change_by_the_probe`、头一次调用的 `started`／`outcome`、`publishes_since_the_last_user_change_after_the_first_call` 与 `read_at_the_first_call`（调用前后不等）。

推的读数：上一次挂载推到门关时已过发布数十几次（门关那一刻再发 1 次就只剩 3 个状态），重挂续做把那条意图推完（产品批大小 16，几次发布），甲臂调用前后都是续做那几次；乙臂取 max 之后变成根环里「最新状态之后的根数」。一处没把握（推的）：产品按用户可见树根指针认状态，续做那几批要是搬了 inode 树或 extent 树的根单元，那一版就算一个新状态，「之后的根数」只数它之后那几条，乙臂可能与甲臂相等；这时读根环那一段其实走到了、只是读出来的不比续做数大，`read_at_the_first_call=false` 不说明补丁没生效。这一格要定性，得另造一段续做不搬树根单元的前缀，本腿没造。

H3 改变行为的那一形（停着、意图活着、头一次调用就判扣完）这里没造：挂载续做在停着时一趟开头就删意图、不停着时推完意图，重挂之后头一次调用手里都没有活意图（上一份报告「挂载那一份预算（B1）」那一段同判，没找到不靠写盘报错的产品前缀）。所以 H3 这里只给读数，不给行为差。

### 补丁的副作用（推的，交主 agent 看）

- 门关而停着时，每一次叫起都起跑一趟、读一遍根环与每条有效根的树表，再停在门上。门关的停记进了不发布的备忘，可后台入口停着那一判直接走 `start_while_the_compaction_stays_stopped`，那里不看备忘（只看「有没有活意图」与预算，「    if !has_a_live_intent || (is_the_budget_spent && !foreground_cannot_write) {」（`crates/singlefs-core/src/mounted_session.rs:4331`））。没停着那一支的定时兜底会先查备忘（`stop_still_holding_at`）、不读盘。代价是一次根环读（随根环槽数，不随盘容量），不写盘；要省就在 `start_while_the_compaction_stays_stopped` 里也查备忘，那是实现员的事，探针里第二次叫起（`wake=again_without_a_change`）的 `outcome` 看得出读没读（又停在门上就是又判了一次）。
- 补丁第 1 处让停着时的头一次后台调用读根环；`whether_the_background_compaction_starts` 的文档串「开头那几判……一个盘都不读、一个写都不发」在没停着那一支本来就因为读根环不成立，补丁之后停着那一支也不成立。要是验证包里有用例在停着的头一次调用上数读盘（「停着时一个盘都不读」这一类），会红；本腿没 grep 全，这条交实现员合入时跑相关测试目标看。
- 门关时那一趟的结局从 `Stopped(BackgroundTaskStoppedAfterAnEarlierComputeError)` 变成 `Stopped(CompactionPublishWouldLeaveFewerThanFourRollbackStates { … })`，触发名照旧是 `LiveIntentWhileStoppedAfterAComputeError`；后台入口的报警（停着的那一个）由别处按停着判，补丁没碰，推的不变。

### 改法表

| 改法 | B2（门关时停着那一趟越门） | 门开之后删得掉 | H3（停着的头一次调用读挂载那一份预算） | 标注 |
|---|---|---|---|---|
| 判决第三节第 1 条（本补丁） | 推的：修，待主 agent 跑 | 推的：删得掉，待主 agent 跑 | 推的：读得到（读数可能与续做数相等，见上），待主 agent 跑 | 只在我的模型上量过（还没量）、被攻过零轮 |
| 另加：`start_while_the_compaction_stays_stopped` 查不发布的备忘 | 不改结局 | 不改（任何一次发布、用户改动都让备忘失效） | 不改 | 推的；只省门关期间每次叫起那一遍根环读；没实现 |

## 没打中的形状

这一腿只造 A/B，不攻别的面，没有「打中」可报。试过的形状都在上面「探针与喂的输入」：3 档门前改动 × 3 种门关之后 × 门开前后至多 10 次叫起，两臂同一批；H3 一段。读数都等主 agent 跑。

## 这一条腿自己的限度

- 探针与补丁都没编过。探针的 API 照上一份探针（任务 12314 编成、跑过）与 `crates/singlefs-verification/tests/common/mod.rs` 现读的签名写；新用到的只有 `MemoryPool`／`SparseBlockDevice`（字段 `image`、`devices` 都是 `pub`）、`BuiltPool::memory_pool`、`IMAGE_BYTES`、`singlefs_core::block_device::PhysicalBlockSizeInBytes`，与 `common::crash_state_devices` 拷盘的写法相同。补丁里新加的判法照循环里同一段写，用到的名字都在同一个函数里已有。编不过时 `^error` 行会打出来，交承接的攻方腿改。
- 起点那一次仍经 `build_pool` 建文件镜像（mkfs 的现成装置只有这一份），之后每段历史都在内存里；换盘之后甲臂要是与任务 12314 对不上，先查装置。
- 补丁是本腿照判决原文写的一种实现：门放在「预算不为 0」之后、删意图之前；破例照循环里第一次发布那一档。实现员写进产品代码时次序与破例要是另取，这一组读数不直接替它作证。
- 判门的独立读法按用户改动分状态，与产品按树根指针分状态不是同一个数；`gate_refuses_by_the_probe` 与产品的 `outcome` 在门边上可能不一致，`rerun.sh` 的期望只拿头一次叫起判（那一刻两种读法在任务 12314 里都判门关）。
- `patch` 是系统命令：派到的那一台没有它，乙臂报 `name=m4_close_batch2_r2_ab_patch … rc=` 非 0、`rc=skipped`，不会把没打上的代码当乙臂读。
- 冻结副本给的是仓根（六份被判文件此刻与快照逐份相同，两臂各自先核）；主工作区这几份在主 agent 交任务之前改了，快照那一步报 `rc=1`、那一臂不跑。两臂在同一个任务里先后拷，别的文件就算有人在改，两臂拷到的也是同一刻前后的树（推的，没另核）。
- 内存与时间（推的，没量）：任务 12314 第 4 次尝试（一臂、本机）用时 137.703 秒、内存峰值 5.16 GiB（`task-server/task status 12314` 量的）；这一次两臂先后各编一份、各跑一次，推的挂钟约两倍（5 分钟上下），峰值与一臂相当（两臂不并行）；探针每臂的叫起比上一份多（切换的两种各多 2 到 9 次叫起），推的每臂几十秒。远低于 40 分钟，没缩取样。没先跑一小段估时：子 agent 不起项目代码。

## 要主 agent 跑的命令

1. A/B 两臂（一条命令、一个任务，两臂各建一份副本、先后跑；在草稿目录起，冻结副本给仓根）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 attack_ab_stopped_drop_judges_the_rollback_gate（攻方 A/B 探针，甲乙两份副本里各新加一份）; 范围=1 个测试目标; 为什么=判决第三节第 1 条零轮改法写进产品代码之前的两臂读数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r2-opus-s2 -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-batch2-code-r2-opus-s2-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r2-opus-s2/ab-run /home/fy5090/code/singlefs/research/prompts/m4-close-batch2-code-r2-opus-s2-model`
   - 在哪起：`--cwd /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r2-opus-s2`（运行目录 `…/ab-run` 是新目录，外层先删它再起；判决复跑照规则把模型目录拷到主 agent 自己的草稿目录、末参数换成那一份）。
   - 看什么：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=`（0 = 两臂都核过快照、乙臂补丁打上且 sha256 对上、两臂都编成跑完、前提断言过；1 = 有一臂没跑成，看下面哪一行不是 0；别的码是脚本那一步）。日志里依次：`name=m4_close_batch2_r2_ab_snapshot arm=without-the-patch rc=0`、`name=m4_close_batch2_r2_ab_probes arm=without-the-patch rc=0`、`name=m4_close_batch2_r2_ab_snapshot arm=with-the-patch rc=0`、`name=m4_close_batch2_r2_ab_patch arm=with-the-patch rc=0`、`name=m4_close_batch2_r2_ab_probes arm=with-the-patch rc=0`；观测行 `arm=<臂> m4-close-batch2-r2-ab: stopped_drop_gate_wake …`（每次叫起一行）、`… stopped_drop_gate …`（每段历史一行，每臂 9 行）、`… stopped_drop_gate_ab_summary …`（每臂 1 行）、`… mount_budget_first_stopped_call …`（每臂 1 行）；末两行 `name=m4_close_batch2_r2_ab_verdict without_the_patch_reproduces=… with_the_patch_holds_at_a_closed_gate=… with_the_patch_drops_once_the_gate_opens=… arms_failed=0` 与 `name=m4_close_batch2_r2_ab_mount_budget …`。产物落在运行目录：`observations.txt`、`probes-without-the-patch.log`、`probes-with-the-patch.log`、`snapshot-check-*.txt`、`patch-with-the-patch.txt`；副本在 `ab-run.attempts/attempt.*/copy-<臂>`，每臂跑完脚本自己删，编译目录归服务（`task-server/task clean <号>`）。
   - 接着做：主 agent 交、等到结局之后新派同立场的腿（「承接：」这一份，指到任务号与 `ab-run/observations.txt`），读两臂读数逐格对上面「跑前写下的期望」那张表，写结论；编不过时由那条腿改探针或补丁。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，上面那一条与判决要引的数都没跑，报告里的读数全是推的。
- 副本上量出的数（跑出来之后）不算入库装置上的数；补丁只在副本里打，主工作区 `crates/` 没动。
- 没造 H3 改变行为的那一形（停着、意图活着、挂载之后头一次调用就判扣完），见 H3 一节；破例那一档（前台写不进）、攒批会话与放锁的后台入口没扫。
- 没 grep 验证包里会不会有用例被补丁第 1 处（停着时头一次调用读根环）弄红，交实现员合入时跑相关测试目标。
- 没写调崩溃枚举函数的原型，崩溃状态数 0；这一腿与前面的腿合起来原型跑的崩溃状态数仍是 0。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r2-opus-s2` 里只留文本：`syntax/`（探针拷贝与两份空占位，给 rustfmt 解析用）、`rerun_steps.sh`、`rerun.sh`（拼脚本的草稿）、`ab_field_check.sh`、`ab_field_check.txt`、`patchwork/apply_edits.py`（在拷贝上做补丁那四处替换的脚本）；补丁用的两份 `mounted_session.rs` 拷贝与试打补丁的那一份已删（见下一节）。没建仓副本、没建编译目录。

## 删了的草稿

- `/home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r2-opus-s2/patchwork/a`（476K，补丁前的 `mounted_session.rs` 一份拷贝）、`…/patchwork/b`（480K，打补丁后的一份）、`…/patchwork/applytest`（480K，试打补丁的一份）与两份补丁草稿（各 8.0K）：补丁与打完的 sha256 已进模型目录 `patches/`，这几份拷贝删了（`du -sh` 量的）。
