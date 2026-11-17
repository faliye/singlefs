# m4-close-batch2-code-r2：攻方腿报告（Opus，2026-11-10）

<!-- doc-lint:not-numbers R -->

立场：攻方。第二轮分工：第一轮三处修补本身、产生结论的方法、从未被看过的 `mounted_read.rs` 改动。第一轮攻方报告与判决列过的形（后台入口停着不删、删意图落点受 R 排他、落后支两处经裸读者、`budget == 0` 走不到、同实例窗口六形）不再攻。方案按 `crates/` 今天的实现谈：主工作区六份被判产品文件与 `research/prompts/m4-close-batch2-code-r2-snapshot/crates-sha256.txt` 逐份 `OK`（量的，`sha256sum -c`）。

## 复跑

模型目录 `research/prompts/m4-close-batch2-code-r2-opus-model/`。复跑命令（整条交服务，见末节「要主 agent 跑的命令」）：
`bash research/prompts/m4-close-batch2-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录（工作根下、以 -run 收尾）> <模型目录>`。
它删掉那一个运行目录，再起同目录的 `tools/rerun.sh`（拷自 `scripts/agents/three-way-rerun-template.sh`，只换了 `rerun_steps`）：经 `minimal-copy.py` 拷冻结副本、核六份产品文件 sha256、把探针放进副本验证包的 `tests/` 当新测试目标 `attack_stopped_drop_skips_the_rollback_gate`，只跑它。断言写在打中那一边：绿 = 打中、红 = 没打中，观测行以 `m4-close-batch2-r2-attack:` 起头。

```
5ac7547cd4c073babdd0a1ec9b00baf021ba3207727529ca127639cf5816f732  probes/attack_stopped_drop_skips_the_rollback_gate.rs
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
fd648d76e06921056e1070bd6726cdcee9a4ac4fd919216e0010069701e7e790  tools/rerun.sh
```

我什么都没编、没跑（子 agent 不交任务）。探针只过了 `rustfmt --edition 2021 --check`（草稿目录里配空的 `common.rs` 两份占位，解析过、排版已照 rustfmt 改齐）。下面没标「量的」的读数都是推的，判决要引的数等主 agent 复跑。本腿原型跑的崩溃状态数：0（没写调枚举函数的原型）。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| 三·1 A2 修补本身 | 没打中（一次抽样） | 放开与放回在三条返回路上都成对；放开的那一窗口里只有这次发布自己的节点；不数自己新落点那一核放不过 R 里上一版的记录 | 读码（推的） |
| 三·1 附：崩溃之后的续做 | 推不出（轻，算打中） | 新流的崩溃镜像不带「R 之外满」（隔离只在会话内存里），恢复后续做在同样空间紧的盘上有没有走到放开 R 那一支，没有枚举罩着 | 读码＋读流文件头 |
| 三·2 A1 修补本身 | **打中 B2** | 后台入口停着而有活意图起跑的那一趟，删意图排在一趟开头、回退门之前，门不判：门关着、意图活着时切到一张卡都用不上的设置，下一次叫起照发一次删意图，根环里留的不同用户状态（算余量）掉到 3 个 | 读码＋探针（待主 agent 跑） |
| 三·2 附：挂载那一份预算 | 推不出（窄，算打中） | 停着那一判排在「挂载不给新预算」读根环之前，起跑判预算用的是没读根环的已过发布数；产品上要先有挂载留下的活意图而挂载之后头一次叫起就停着才走得到 | 读码（推的），没造 |
| 三·2 附：攒批排空、切到零卡 | 没打中（一次抽样） | 排空发成一次用户改动，预算与门都新；切到零卡有活意图时删，与定案同 | 读码 |
| 三·3 A3 修补本身 | 没打中（一次抽样） | 五个调用方都把手里那一份分配器传进来，账各份拷贝共用、按位置条目去重；拒了账留着；新用例那一核之后到 `NotInRing` 之间只读实例表与根环 | 读码 |
| 三·4 `mounted_read.rs` | 没打中（一次抽样）；一句注释字面不成立 | 新类型实参次序与函数签名一致；带 fsid 的入口只换系统配置那一择；「一次调用里一个单元只读一遍」在提示不算数、转查映射时不成立（计数结论照样成立） | 读码 |
| 四·1 走不到的理由（放回排他） | 成立 | 发布出错返回时排他一样放回：放开在 `?` 之后、放回无条件；被拒与冻结两路分配器都换回放开之后的那一份 | 读码 |
| 四·2 类型与名字 | 成立；一处文档串推不出 | `LiveIntentWhileStoppedAfterAComputeError` 的文档串说「预算照后台入口别的起跑给」，门不照给（B2） | 读码 |
| 四·3 不兼容旧代码 | 成立 | 现跑 `grep -nE 'legacy|old_format|migrat'` 在 diff 上零命中（量的） | grep |
| 四·4 不新建判定库 | 成立 | diff 新加行零命中 prune／rocksdb／sqlite／`.tsv`（量的） | grep |
| 四·5 多机可用 | 成立 | 四份测试源零命中 `std::env::var`、`hostname`、`/home/`、`CARGO_MANIFEST_DIR`、读文件（量的） | grep |
| 四·6 测试优先多线程 | 成立（推的） | 落后支那份测试的分叉扫描取 `default_heavy_threads()`；崩溃流走枚举函数自己的线程；停着那份两臂串行，量小 | grep＋读码 |
| 四·7 带 --item 的命令与项 | 成立（读登记行） | `.claude/gate.d/stage-inputs.tsv` 第 162 行、`.claude/gate.d/crash-point-replay-items.tsv` 第 78 行都在、四列；`--list-items` 没跑 | awk |
| 四·8 新崩溃流多罩了什么 | 那一句成立；终点够不够推不出 | 流到删意图落盘为止；之后在同一个 R 上重签（R 里装着删意图那一次的节点）与恢复后在空间紧的盘上续做，都不在里面 | 读流文件头 |

哪一条最脆弱：B2（A1 修补那一趟不过回退门），见末节。

## 攻击面二：A1 的修补本身

### B2：停着删意图那一趟不过回退门（打中，越位）

**历史**（每一步指到许可它的那一句）：

1. 可写挂载选 `gpu`（两张卡都用得上），整理照水位判（回退门只在这一支判：「let is_the_rollback_gate_judged = matches!(pacing, PacingOfTheRun::PacedByTheCaller { .. })」（`crates/singlefs-core/src/mounted_session.rs:4627`））。
2. 用户连着建几个 inode（几个不同用户状态），之后不再改；调用方照常一次次调后台入口（定时兜底）。整理签发、一批一批推，推到门关：`CompactionPublishWouldLeaveFewerThanFourRollbackStates`，意图还活着、R 排他（`compaction_rollback_gate_keeps_four_user_states.rs` 的 `the_gate_closes_exactly_where_the_next_eight_publishes_still_keep_four_user_states` 就是这样造到门关的，那一刻「再发 1 次就只剩 3 个」）。
3. 调用方挂着切到一张卡都用不上的那一组（D26 已定项 13 第 3、4 条许可，切换不拒）。
4. 下一次后台入口：切换在判停没停之前换上，整理停着而不超过补跑阈值，判起跑那一判走「return Ok(start_while_the_compaction_stays_stopped(」（`crates/singlefs-core/src/mounted_session.rs:3395`）：有活意图、上一次用户改动给的那一份 `B_gc`（20）只用了十来次 ⇒ 起跑 `LiveIntentWhileStoppedAfterAComputeError`。
5. 那一趟开头「    if compaction_stays_stopped(allocator, background_compute) {」（`crates/singlefs-core/src/mounted_session.rs:4505`）为真，预算不为 0，直接 `drop_the_live_intent_on_a_stop`、发成、返回。回退门那一判「if is_the_rollback_gate_judged && !is_this_publish_the_exception {」（`crates/singlefs-core/src/mounted_session.rs:4728`）在这段之后的循环里，走不到。
6. 这一次删意图写的根槽连同之后 S 槽余量一走，根环里不同用户状态从 4 个掉到 3 个；第 2 步那一刻门刚拒过同一次发布。

**对不上的句子**：

- D26 已定项 13 第 5 条管门的那一句，`.claude/kb/decisions/26-后台整理与放置回收.md:252`：「一次整理发布要写的根槽连同它之后的余量里的根一走、根环里不同用户状态就少于 4 个时，这次整理发布不发」；那一行整行（内含书名号，不进引文核对）：5. **什么时候起跑**：挂载就起、按周期查。每次发布之后按已定项 1 的三水位查一次；另有定时兜底，每隔一段时间再查一次，间隔是挂载参数，默认 30 秒。周期线程住在新开的运行库一层：挂载时按参数开卡、建实例、挂载、起周期线程，前台与后台经一把锁排队；核心层照旧不起线程、不读钟。定时兜底的预算照旧与用户改动共用同一份（第 6 条），挂载不再给新预算。一次整理发布要写的根槽连同它之后的余量里的根一走、根环里不同用户状态就少于 4 个时，这次整理发布不发（守 D16（发布语义） 已定项 1 的「回退候选集保留最近 4 个可退到的不同状态」）；余量取 S 槽，S 是每区根槽数（根环共 3S 槽）。只有全空聚簇段数低于底额（已定项 3）、前台已写不进时破例，每趟至少发 1 次；破例排在第 6 条的让路之前，前台有 fsync 在等也照发（前台已写不进，让路换不来什么）。删除队列不空也算起跑：这一趟只走队列、不发整理，不看水位，也不被发布预算挡。两种模式起跑规则相同。（整行到此）删意图是一次整理发布：「/// 再照预算一次一次推（[`advance_compaction_by_one_publish`]）。预算 `B_gc` 数的是整理的每一次发布（签发、搬一批、删意图：每一次都用掉一个根槽）。」（`crates/singlefs-core/src/mounted_session.rs:4349`）。破例的条件（全空段低于底额、前台写不进）在这段历史里不成立。
- 门的豁免只写了两处：「// 回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方，可写挂载续做只推盘上那一条，发布数有上界）。」（`crates/singlefs-core/src/mounted_session.rs:4626`）。这一趟是后台入口、照水位判，不在豁免里。
- 新触发的文档串「/// [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；删完之后的调用照旧不起跑。预算照后台入口别的起跑给：那一份扣完而前台写得进就不起跑。」（`crates/singlefs-core/src/mounted_session.rs:516`）：只照给了预算，后台入口别的起跑每一次整理发布之前都判门。
- 用户 2026-11-10 定案（正文第二节转述）「那一趟吃发布预算，预算扣完就等下一次用户改动之后那一次调用再删」只说预算，没说免门；我没在 records 里核定案原话（我印象里没有免门的说法，没核）。

**这一批让它变了什么**：第一轮之前后台入口停着时一个写都不发，门的问题不在；A1 修补把一次整理发布放进后台入口，却放在门之前。暂停与一批出错放弃那两处删意图都在循环里、门之后（门判的「下一次发布」就是它），停着这一处是后台入口上唯一不经门的整理发布。

**四句**：
- 分不分辨臂：分辨。同一段历史、同一刻（门刚关），「停着那一趟判门」那一臂不发、「不判门」那一臂发；探针里不切换的对照臂就是门照判的那一边，期望 0 次发布。
- 系统当时看不看得到判别它的东西：看得到。那一趟手里有参数、盘表、分配器与现行那一版，`stop_of_the_rollback_gate` 要的四样都在；后台入口别的起跑在同一个函数里就是这样判的。
- 满足的是判据字面的哪一个分句：第 5 条「一次整理发布要写的根槽连同它之后的余量里的根一走、根环里不同用户状态就少于 4 个时，这次整理发布不发」；不落在「只有全空聚簇段数低于底额、前台已写不进时破例」那一分句。
- 跑前条款给的改法在打中的格上还中不中：判决第三节第 1 条给的改法（后台入口起跑一趟只删意图）就是造出这一格的那一改，它在这一格上中；下表的改法甲修得到。

**探针**（`probes/attack_stopped_drop_skips_the_rollback_gate.rs`，一个测试 `attack_the_background_trip_left_stopped_drops_the_intent_past_a_closed_rollback_gate`）：池、开关与门的读法照 `compaction_rollback_gate_keeps_four_user_states.rs`（顺序写 40 个数据单元、一批一个、选段强制、开关照水位判、越线靠只供测试的开关），切到零卡的假实现照 `compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs`（只点名 card-eight 时答一张都用不上）。判门用测试自己的读法（`readable_roots` ＋ `effective_rollback_floor` 按用户改动分状态），不经产品的门。

用户动作放开扫（只固定前缀与「门关」这一刻）：门关之前建 inode 的个数取 4、6、8；门关之后三臂——切到零卡再叫起、切到零卡之后先建一个 inode 再叫起、不切换再叫起（对照）。期望（推的）：门关着的每一种个数下，第一臂起跑 `LiveIntentWhileStoppedAfterAComputeError`、发 1 次、意图没了、之后「算余量剩几个」< 4；第二臂也发 1 次，但那一次用户改动给了新状态，门本来就放（这一臂不算打中，它说明打中只在门关到叫起之间没有用户改动时）；第三臂 0 次。断言只拿第一臂与第三臂判。

推翻条件：观测行 `stopped_drop_gate` 里第一臂 `publishes_of_the_wake=0` 或 `started=false`（停着那一趟另有门，B2 不成立）；`gate_refuses_the_next_publish=false`（门关那一刻独立读法不认为下一次发布会挤掉状态，这一形退化）；三种个数下 `gate_closed_before=false` 或 `live_intent_before=false`（造法要改，`stopped_drop_gate_summary geometries_judged=0`，测试红在「前提不成立」，不算没打中）。

### 同一趟里另两样没过的判（推的，不单列打中）

- 节奏：停着那一趟的删意图也排在「前台有 fsync 在等就不发」与写带宽那一判之前（两判都在第 5 步之后的循环里）。D26 已定项 13 第 6 条说节奏「只约束整理批（每一批之前问调用方一次）」，删意图不是一批，字面放得过；但写带宽那一窗口照记这一次的字节（`note_the_bytes_written_by_a_compaction_publish` 在停着那一段里照调），记而不判。交主 agent 判这一格归不归第 6 条。
- 挂载那一份预算（B1）：「&& compaction.take_the_reading_of_the_publishes_of_the_mount_from_the_ring()」（`crates/singlefs-core/src/mounted_session.rs:3409`）在停着那一判（第 3394 行）之后。挂载起步时已过发布数是 0、要读根环（「publishes_of_the_mount_are_still_to_be_read_from_the_ring: true,」（`crates/singlefs-core/src/compaction_plan.rs:1008`）），挂载之后头一次叫起就停着、手里又有活意图时，`start_while_the_compaction_stays_stopped` 拿没读根环的那个数判「扣完」，按根环本来已扣完也照起跑删一次。走得到的前缀要「挂载留下活意图、而挂载之后头一次叫起之前没有用户改动就停着」：挂载续做本身在停着时一趟开头就删、发成就没有意图，所以要续做那一次删没发成（落盘报错冻结，之后重发把意图重装）或续做被截在别处；我没找到不靠写盘报错的产品前缀，判成窄、推的，没造探针。

### 攒批会话先排空再删、切到一张卡都不剩（没打中，一次抽样）

- 攒批：判起跑说起跑之后先排空待发；排空发成就是一次用户改动，已过发布数归零、根环里多一个新状态，门与预算都新，删意图那一次照常；排空被拒整次返回、不跑这一趟（读码）。边角（推的）：排空的那次改动把最满的盘推过补跑阈值时，这一趟开头不再停着、照常补跑一批，报出来的触发却仍是 `LiveIntentWhileStoppedAfterAComputeError`，与它文档串「这一趟只单发一次删意图的发布」对不上；只是观测名不准，不改盘上状态。
- 切到零卡、手里有活意图：后台入口照新触发删一次，没有活意图才在任何写之前停着。D26 第 251 行原文「挂着切换到参与的卡一张都不剩的设置（第 3 条）：下一次整理入口与后台入口在任何写之前就报停着，删除续做也不走，报的是挂载那一刻同一个成员（`GpuUnusableUnderTheSettingsHandedIn`）。」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`）与实现不同，正文第二节已说定案还没写回 kb、以定案为准，不另算。

### 改法表（每一格修哪一个打中）

| 改法 | B2（门） | 节奏两判 | B1（挂载那一份预算） | 标注 |
|---|---|---|---|---|
| 甲：停着那一趟删意图之前，后台入口、照水位判时先判回退门（破例同别的起跑：全空段低于底额、前台写不进时不判）；门关就停在门上，意图与 R 排他留到下一次用户改动 | 修 | 不修 | 不修 | 推的；只在我的模型上量过（还没量）、被攻过零轮；行为与定案「预算扣完就等下一次用户改动」同形 |
| 乙：停着那一判挪到读根环那一步之后（`start_while_the_compaction_stays_stopped` 拿 max 之后的已过发布数） | 不修 | 不修 | 修 | 推的；被攻过零轮 |
| 丙：只改文档串写明停着删意图不经门 | 不修（与第 5 条冲突，要用户定） | 不修 | 不修 | 推的 |
| 丁：删意图之前也问调用方 fsync 与写带宽 | 不修 | 修 | 不修 | 推的；第 6 条字面不要求，交用户定 |

## 攻击面一：A2 的修补本身（没打中，一次抽样）

试过的形状（全读码）：

1. 放回在每一条返回路上都走到：放开在「let relocations = relocations_checked_against_the_version(pool, allocator, previous, planned)?;」（`crates/singlefs-core/src/transaction.rs:10964`）之后，那一个 `?` 早退时还没放开；之后到「.put_back_the_region_exclusion(exclusion_before_the_publish);」（`crates/singlefs-core/src/transaction.rs:10994`）之间没有 `?`，发成、被拒都过它。被拒时发布路径把分配器换回「let allocator_before_this_publish = allocator.clone();」（`crates/singlefs-core/src/transaction.rs:13170`）拍下的那一份——拍在放开之后，换回来照样是放开的，随后放回；落盘报错冻结同一份换回。重发冻结的删意图那一次：会话按重发那一版的 logged_ops 撤、重装活意图（`resend_the_frozen_publish` 文档串，第 3301 行），删意图那一版里没有意图 ⇒ 撤。
2. 放开的那一窗口里有没有别的分配：这一次 `planned` 空、`relocations` 空、豁免换单元那一张传 `&[]`、实例表照抄、删除队列照抄，落进 R 的只能是这一版自己的固定点（记账树、分配记录树、中央映射树、树表、logged_ops 根）。另发那一批、前台、抬 F 的空发布都是别的发布，放回之后才发。
3. 不数自己新落点那一核放不放过 R 里真有别的记录：滤掉的只是「未释放而分配代等于这一次 txg」的记录；上一版就在 R 里的记录分配代都不大于上一版 txg，照算；R 排他之下上一版 R 里不会有分配代 > `c0` 的记录。所以「R 里真有别的记录」那一形照样判 R 不空。
4. 开放段：提交内生块先从开放段 bump，「        if let Some(open) = self.open_segment {」（`crates/singlefs-core/src/allocator.rs:2586`）那一支不看排他。删意图那一次放开 R、R 是最低的全空段时，它开 R 当新开放段；之后同一次挂载里选段的资格第一条就是「/// 挑一个整理区域 R。资格五条全成立的段才算：不在这次挂载开过的聚簇段里（`cluster_segments`，段起点）、」（`crates/singlefs-core/src/compaction_plan.rs:84`），R 不会再被选成新意图的区域，开放段与活 R 不重合。只有只供测试的强制选段能在同一个 R 上重签而开放段正是 R，那时 bump 会落进活 R、池级 checker 判 I-5.6 红：产品走不到，提醒以后接这条流的用例别用强制选段在同一 R 上重签。
5. 崩溃（推的，没枚举）：删意图落盘之前崩，落到意图活着、R 排他的上一版，R 里那几个槽在那一版里是空槽，I-5.6 只看记录，成立；落盘之后崩，那一版里没有意图，I-5.6 不适用。新流（stage-inputs 第 162 行）罩这两段崩溃状态，任务 12167 缩到展开深度 4、`exhaustive=false`，全量留给提交时的整轮门禁（正文第二节转述，我没读那份日志）。

### A2 附：崩溃之后在同样空间紧的盘上续做（推不出，轻）

新流文件头写「之后在会话的分配器上把 R 之外的每一个空槽隔离（不写盘、不进录制流）」，主流那一段另写「压余量只动会话分配器的隔离位、不写盘：录制流里没有它，崩溃镜像上的可写挂载不带它。」（两句都在 `crates/singlefs-verification/tests/crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs` 文件头）。所以删意图落盘之前崩的那几个状态，恢复后挂载续做看到的是 R 之外又有空槽的盘，续做照常搬完；真盘上 R 之外是被真数据写满的，恢复后续做那一批照样取不到落点、暂停、在挂载里放开 R 删意图，盘上 R 那几个槽里还留着崩掉的那一次写了一半的字节。这一支（挂载续做里的暂停删意图、落点落在 R 里、覆写半截字节）没有枚举罩着，正文第三节第 1 条问的「恢复后续做成不成立」在这一支上推不出。读码看它成立（那几个槽在上一版里是空槽，覆写不碰任何记录），所以只算轻；要钉住得在录制流里用真写满（写盘的分配）代隔离，或另开一条从崩溃镜像起、R 之外真满的续做流。

### A2 附：不带一批的达成删意图也放开 R（推的，交用户看）

定案把「不带一批的 `DeleteOnceTheRegionIsEmptyByTheRecords`」一起放开。R 按记录全空、R 里的槽也过了复用窗口时，那一次的固定点取落点照「最低的全空段，没有就回落到槽号最小的空槽」：R 是最低的全空段时它开 R 当新开放段，R 当场不再全空。别处还有全空段时，排他与放开都要开一段，全空段数一样；只有 R 是池里唯一的全空段时两者不同——排他时回落到 R 之外最小的空槽、R 保持全空，放开时 R 被这一次自己用掉。这一形下这条意图的产出被它自己的达成删意图吃掉，下一条签不签照「上一条无产出就不签」判。正确性不受影响，没造模型；交主 agent 看要不要把放开收窄到 `Drop`。

## 攻击面三：A3 的修补本身（没打中，一次抽样）

试过的形状（全读码）：

- 五个调用方（抬 F 到准入上限、准入里再推一串、`raise_the_floor_through`、推环、挂着回退）传的都是它手里那一份分配器；预演在拷贝上走时账也进同一份：「    other_unit_reads_with_one_copy_failing: Arc<Mutex<OtherUnitReadsWithOneCopyFailing>>,」（`crates/singlefs-core/src/allocator.rs:1945`）各份拷贝共用，记账按位置条目去重，真跑与预演读到同一份不重计。
- 拒了随错：这一核报落后而缺单元时，前面读到只坏一份的那几组照进账；之后入口照样拒落后盘，账里那几组等的「下一次用户改动」在落后解除之前发不成，不伤；解除之后照常换下。账里的位置条目在下一批里按改动之后那一版认（`units_at_these_locations`），回退之后已不在那一版里的认不出、照丢，不会搬错单元。
- 正常卸载那一处也记：卸载之后这次挂载没有下一次用户改动，账随会话丢掉；与挂载那一刻读到的同形，不算漏。
- 新取样点用例 `a_device_behind_at_a_rollback_refused_after_the_check_of_the_caller_inputs_counts_the_copy`：那一核之后到 `NotInRing` 之间，回退只读现行那一版的实例表链与根环（`rollback_candidate`），不碰第一个文件的数据单元；账里那 1 组只能是那一核记的。分辨力成立（读码），用例自己的文档串也写了另一条做成的用例分辨不出。

## 攻击面四：`mounted_read.rs` 相对 HEAD 的改动（没打中，一次抽样）

- 三个新类型：实参次序（记录号、分隔 key、容器号）与 `recovery.rs` 里那个函数的形参次序一致；`identity_in_the_entry.container` 在调用之前已核过与叶头 `container.identity` 逐字段相等，与冷走读传 `identity.container` 同值。
- `mount_read_only_of_the_pool`：只把择系统配置换成按 fsid 筛的那一个，之后同一条路（`mount_read_only_from_the_system_configuration_choice`）。别的池的盘被当两槽都不可择、列进 `devices_without_a_mountable_system_configuration_slot`；它的根环槽与单元头带别的 fsid，按择到的系统配置的 fsid 读根、按根的 fsid 核单元头，读不进来，读到别的池的字节走第二份。没找到经它读到别的池的一形（读码，没造双池盘表）。
- 新注释「        // 读是真发到盘上的、坏的那一份是真读到的，计不计不看这一次调用成没成。这个累计不去重：一次调用里一个单元只读一遍，跨调用相加。」（`crates/singlefs-core/src/mounted_read.rs:1029`）：「一个单元只读一遍」字面不成立——提示两条都不算数时「        observation.stale_location_hint_hops += 1;」（`crates/singlefs-core/src/mounted_read.rs:1166`）之后按映射那一组再读一遍，提示没过期时两组是同一个单元的同两条位置条目。计数的结论照样成立：按提示那一组只在读成时计、读成就返回，按映射那一组至多计一次，一次调用里一个单元至多计 1。改法：把那半句写成「一次调用里一个单元至多计一次」。不改行为。
- 新文档串（第 372–373 行）说只读那一条路只计数、不搬：与 `ReadPathObservation` 字段文档「只读挂载只计数、不搬」同；可写挂载的会话读文件另经计数层进账、去重，同一个量在两条路上一个按次、一个按位置条目，口径不同但各自写明了，不算打中。

## 第四节八格

1. **走不到的理由二次查实（放回排他）**：成立。逐处追的调用点：`publish_compaction_step_in_the_session` 是放开与放回唯一的一处（`grep -n 'open_the_region_to_the_publish_deleting_the_intent\|put_back_the_region_exclusion'` 在 `crates/singlefs-core/src/` 下只命中 `compaction_plan.rs` 的定义与文档、`transaction.rs` 第 10979、10994 行，量的）；它的直接调用点两处（`grep -n` 量的）：`transaction.rs` 第 10827 行的公开一族（会话删意图走的 `publish_the_compaction_step_once` 经它），`mounted_session.rs` 第 5831 行带计划的那一批。返回路见攻击面一第 1 条。
2. **类型与名字**：成立；一处文档串推不出（`mounted_session.rs` 第 516 行「预算照后台入口别的起跑给」，门不照给，并进 B2）。新方法名、新枚举 `PlacementsOfThisPublishInTheRegion` 两个成员名说得出内容；`CountedAsRecordsInTheRegion` 文档串「只供测试的开关关掉排他时走得到」之外，开放段 bump 不看排他那一支在强制选段下也走得到（攻击面一第 4 条），不改它的判法。diff 的产品代码新加行里零个 `expect(`、`unwrap(`、`unreachable!`、`panic!`（量的）。
3. **不兼容旧代码与旧格式**：成立（`grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-batch2-code-r2-diff.md` 退 1、零命中，量的）。
4. **不新建判定库**：成立（diff 新加行零命中 prune／rocksdb／sqlite／`.tsv`，量的）。
5. **所有任务多机可用**：成立（四份测试源零命中本机路径、主机名、环境变量、读文件，量的）。
6. **测试优先多线程**：成立（推的）。`writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs` 第 1292 行取 `default_heavy_threads()`、第 1296 行 `std::thread::scope`；崩溃流文件走枚举函数自己的线程取法；停着那份新用例里 `the_background_entry_left_stopped_writes_exactly_what_the_explicit_entry_left_stopped_writes` 两臂串行，各一段短历史，不算该并行而没并行。
7. **带 --item 的命令与项**：成立（读登记行）。`--list-items` 没跑，命令在下面。
8. **新崩溃流多罩了什么**：文件头那一句（多罩删意图那一次的写落在活意图 R 里的崩溃状态）成立；终点够不够推不出——流到删意图落盘就结束，(a) 之后在同一个 R 上重签、R 里装着删意图那一次的固定点那一段，(b) 崩溃镜像上 R 之外真满时挂载续做里的暂停删意图，两段都不在里面（见 A2 附）。另：第一轮判决第三节第 3 条要补的「一趟开头停着删意图、删除队列不空时照抄队列」那一形，这一批新用例里删意图之前队列都是空的（`a_run_left_stopped_at_its_start_reports_the_stop_before_judging_the_batch_size_or_continuing_the_deletion_queue` 那一条队列不空，但活意图已在出错停下那一刻删了、这一趟一个写都不发），仍没有录制流或小用例罩着（读码）。

## 哪一条最脆弱

**B2：A1 修补那一趟在回退门之前删意图。** 依据：它是后台入口上唯一一处不经门的整理发布，而停着有活意图这一态最常见的来历（补跑推到门关或预算截住之后读数降下来、切到零卡）恰好就是门刚关、或贴着门的那一刻；一旦越门，挤掉的是用户能回退到的状态，事后补不回来。条款（D26 已定项 13 第 5 条）没给删意图开破例，定案只说预算。推翻它的现象：探针第一臂 `publishes_of_the_wake=0`（停着那一趟另有一处判门），或对照臂同一刻也发了 1 次（门本来就放，`gate_refuses_the_next_publish` 那一判是我的读法错了）。

## 没打中的形状

- 攻击面一：放回的三条返回路、放开窗口里的别的分配、不数自己落点那一核、开放段与活 R 重合、冻结重发、落盘前后崩溃的 I-5.6（六形，读码）。
- 攻击面二：攒批排空、切到零卡有活意图、`budget == 0` 在新触发下仍走不到（新触发只在预算不为 0 或前台写不进时起跑，前台写不进那一档给 `.max(1)`），三形读码；B1 推的、窄。
- 攻击面三：五个调用方传的分配器、拒了随错、卸载、回退之后的旧位置、新用例分辨力（五形，读码）。
- 攻击面四：新类型次序、带 fsid 入口读不读别的池、计数去重口径（三形，读码）。
取样范围：除 B2 的探针之外全是读码，一次抽样；按规则「没打中」一次不算。

## 这一条腿自己的限度

- 探针没编过：API 照 `compaction_rollback_gate_keeps_four_user_states.rs` 与 `compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs` 的写法抄，只过了 rustfmt 的解析。最可能编不过的：GPU 挂载下强制选段、照水位判这一组开关是不是与那份 CPU 用例一样走到门关（不一样会红在「前提不成立」）。
- B2 的「挤掉的状态」用测试自己的根环读法数，不是产品的 `stop_of_the_rollback_gate`（它是 `pub(crate)`）；两者按同一条款写，我没核它们逐格相等。
- B1、节奏两判、A2 两条附、`mounted_read` 注释都只读码；双池盘表没造。
- 用户 2026-11-10 定案原话我没去 records 里核，按正文第二节转述。
- 本腿原型跑的崩溃状态数：0；没写调枚举函数的原型，A2 的崩溃那一格只读码、只读流文件头。

## 要主 agent 跑的命令

1. 探针（判决要引 B2 的数靠它；在仓根起，冻结副本给仓根——六份产品文件此刻与快照逐份相同，脚本里先核一遍）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 attack_stopped_drop_skips_the_rollback_gate（攻方探针，副本里新加）; 范围=1 个测试目标; 为什么=攻方腿核后台入口停着删意图那一趟越不越回退门' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-batch2-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r2-opus/attack-run /home/fy5090/code/singlefs/research/prompts/m4-close-batch2-code-r2-opus-model`
   看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=`（0 = 打中且快照核过；1 = 没打中、前提不成立或快照核不过；别的码是脚本或编译那一步）；日志里 `name=m4_close_batch2_r2_attack_snapshot rc=0`、`name=m4_close_batch2_r2_attack_probes rc=`、`name=m4_close_batch2_r2_attack_summary hit_reproduced=`，与 `m4-close-batch2-r2-attack:` 起头的 `stopped_drop_gate`（九行）、`stopped_drop_gate_verdict`（至多三行）、`stopped_drop_gate_summary` 观测行。产物落在运行目录（`probes.log`、`observations.txt`、`snapshot-check.txt`），副本在 `attack-run.attempts/attempt.*/copy`、脚本跑完自己删。编不过时 `^error` 行会打出来，交承接的攻方腿改探针。内存与时间（推的，没量）：九段历史各是一份内存池、几十次发布，串行跑；挂钟主要是编验证包这一个测试目标。
2. 第四节第 7 格（仓根起）：`task-server/task submit -- bash scripts/crash/crash-point-replay.sh --list-items`
   看：输出里有一行以 `compaction-paused-short-of-space-dropping-the-intent` 起头。被范围声明闸拒就照拒绝说明补声明再交。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，上面两条与判决要引的数都没复跑；报告里没标「量的」的读数都是推的。
- 副本上量出的数（探针跑出来之后）不算入库装置上的数。
- 没写调崩溃枚举函数的原型，崩溃状态数 0。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r2-opus` 里只有文本：`syntax/`（探针拷贝与两份空占位，给 rustfmt 解析用）、`rustfmt.txt`、`rerun_steps.sh`、`rerun.sh`、`rerun-from-a-clean-work-directory.sh`（拼脚本的草稿）；没建仓副本、没建编译目录，没有要删的。
