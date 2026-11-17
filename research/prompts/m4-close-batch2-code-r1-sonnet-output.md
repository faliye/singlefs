# m4-close-batch2-code-r1 正推腿（Sonnet）报告

批次：`records/2026-10-13-里程碑四预开发.md#收口修补二批`。腿立场：正推（从条款与 `crates/` 今天的实现推「应该是什么」，再与 diff 逐格比）。
被判对象：`research/prompts/_m4-close-batch2-code-r1-diff.md`（5 份产品代码）。没写模型、没起任何命令，只读；「要主 agent 跑的命令」在第四节。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1.1 暂停删意图（定案 1，D26 已定项 2 目标态 / R 排他） | 对 | 删意图发布落盘后才撤分配器侧意图与 R 排他，盘上没有「意图在、R 不排他」那一态，I-5.6 守住 |
| 1.2 一批出错停下删意图（定案 2） | 对，一处缺口 | 三处出错停下都删；后台入口判起跑时判出停着而还有活意图的不删（代码自己写明），与「停下都删」不齐 |
| 1.3 删意图那一次归删除类不判式子（定案 3） | 对，范围比字面宽 | 暂停与出错停下那一次免式子判，落点判照判；同一函数还放过「R 已空只删意图」「丢意图」「零搬迁的删意图批」 |
| 1.4 D4 已定项 4 读接全（r3 收尾） | 对，一处弱漏 | 各读点都接了计数层；抬 F 先写配置槽成功、随后第一次发布在落盘前被拒时，那一读留在随错丢掉的写入口上 |
| 1.5 D18 已定项 11 同实例点名窗口（`c1-same`） | 对 | 窗口、同实例、按计数器认、取不到全算缺都与条款一致；分叉盘取并集条款没写，偏保守 |
| 1.6 窗口外缺的按读到一份坏处理（D4 / D18） | 对 | 计数后经搬迁路径换落点；拒了分配器随错丢掉 |
| 1.7 D28 已定项 1 不判名单、落点照判 | 对（按定案） | 名单字面只写删除意图那一族，整理意图是定案加的；kb 还没写回 |
| 1.8 D26 已定项 13 第 4 条字面 | 与定案相左（待写回） | 字面「一个写都不发」「在任何写之前就报停着」被定案取代；实现对显式入口取定案、对后台入口取字面 |
| 1.9 I-5.6、I-1.11 | I-5.6 守住；I-1.11 diff 没碰 | |
| 面 1 暂停与出错停下删意图 | 站住，两处边角 | 崩溃三段都成立；边角：后台入口不删、满盘时删意图自己被拒 |
| 面 2 同实例点名窗口 | 站住 | 跨实例、记录槽坏、多盘落后三种都判得对 |
| 面 3 r3 收尾读接全 | 站住，一处弱漏 | 见 1.4 |
| 面 4 三处改动交叉 | 没找到互相踩的路径 | 追到的几条路径都不相交 |
| 格 1 走不到理由二次查实 | 成立 | `budget == 0` 那一支走不到；留着不与定案冲，但代码注释写的「预算用完就不删」比定案字面窄 |
| 格 2 类型与名字 | 成立，附小命中 | 旧名全仓零命中；一处新 `expect` 依赖常量 4096；注释里 5 处序数写法，形参 `devices` 装的是读者 |
| 格 3 不兼容旧代码 | 成立 | grep 零命中 |
| 格 4 不新建判定库 | 成立 | 读过的 diff 与六份测试源码没有判定存放处 |
| 格 5 多机可用 | 成立 | 六份测试源码与 `common_compaction_prefix/mod.rs` 不读环境、主机、路径 |
| 格 6 测试优先多线程 | 不成立（待主 agent 跑门禁坐实） | 两份新测试用 `std::thread::available_parallelism()`，不经登记位，排除表里没有，门禁同形判红 |
| 格 7 带 --item 的命令与项 | 成立（`--list-items` 待跑） | 两处登记行在、格式与邻行一致、测试函数名对得上 |
| 格 8 新崩溃流多罩了什么 | 成立，附两处射程限 | 两句都核得到；出错停下那一条只钉写盘相同，一趟开头停着时删意图没有崩溃枚举 |

## 一、格 1：代码做的是不是条款与定案说的

定案的原话只在背景材料第一节那一行（「用户 2026-11-10 定：整理暂停删意图、一批出错停下也删意图、删意图那一次归删除类不判式子」）与记录里一行（「整理暂停 删意图 下次重来」，`records/2026-10-13-里程碑四预开发.md:1589`）。下面「定案 1/2/3」指这三句。

### 1.1 暂停删意图（定案 1）：对

条款：「意图的目标态」一行（D26 已定项 2）：
「**「`R` 内不存在分配代 ≤ `c0` 且未释放的落点」**，`c0` = 写下这条意图那一刻的分配代。**不写「清空区域 `R`」**」（`.claude/kb/decisions/26-后台整理与放置回收.md:55`）。
R 排他：「3. **`R` 在意图期内不可作分配目的地**（回落政策的第一条）」（`.claude/kb/decisions/26-后台整理与放置回收.md:30`）。
I-5.6：「| I-5.6 | 活整理意图的区域里没有签发之后分配的记录 |」（`crates/singlefs-checker/invariants/07-分配器.md:11`）。

diff 块：`mounted_session.rs` 的 `pause_by_dropping_the_live_intent_or_hand_up`（diff 1114–1145）与 `drop_the_live_intent_on_a_stop`（diff 1147–1192）。读 `crates/` 现状：
- 空间不够才调删意图，别的拒绝原样交回（`mounted_session.rs:5850` 起）。
- 删意图单发一次 `CompactionIntentStep::Drop`，`publish_the_compaction_step_once` 返回 `Ok` 之后才 `allocator.compaction_mut().withdraw_the_live_intent();`（`mounted_session.rs:5911`），再换 `current`。R 排他在删意图那一版落盘之前一直在，所以内存里不再出现「撤排他、意图留盘」。旧的 `pause_the_live_intent`、`resume_the_live_intent` 删净：`grep -rnE 'pause_the_live_intent|resume_the_live_intent|IntentDropOnThePauseRefusedShortOfSpaceUndecided|IntentDroppedOnAPause' crates` 零命中（量的）。
- 冻结重发路径会按重发成立的那一版的 logged_ops 重装或撤掉活意图：「活意图的 R 排他按那一版的 logged_ops 重装」（`mounted_session.rs:3286`），删意图那一次写盘中途失败再重发也对得上。

推翻条件：任何一版盘上 logged_ops 里有活意图而同一版分配记录里 R 内有分配代大于 `c0` 的未释放记录（I-5.6 红）；或删意图发布 `Ok` 之后分配器仍有活意图（`mounted_session.rs:5266` 的 `assert_eq!` 会先 panic）。

### 1.2 一批出错停下删意图（定案 2）：对，一处缺口

diff 块：`advance_compaction_by_one_publish` 的两处（diff 1001–1047）、一趟开头那一判（diff 842–919，现 `mounted_session.rs:4448`）。三处都调同一个 `drop_the_live_intent_on_a_stop`：
- `ComputeFailed` 一臂：先 `stop_the_task_after_a_compute_error`，再删意图，停在 `BatchAbandonedAfterABackgroundComputeError`；删意图那一次失败时任务照样已停下。
- 一批开头判出停着而不补跑：删，停在 `BackgroundTaskStoppedAfterAnEarlierComputeError`。
- 一趟开头判出停着：预算没用完就删（见格 1）。

缺口：后台入口 `whether_the_background_compaction_starts` 判出停着就返回「没起跑」，一个写都不发（`crates/singlefs-core/src/mounted_session.rs:3379`），注释写明「这一判之前才停着而还有活意图的，意图与 R 排他照留」（`crates/singlefs-core/src/mounted_session.rs:3230`）。这条路能走到：意图在盘上活着（上一趟因预算、让路等停在意图中间），随后一次切换把设置换成「一张卡都用不上」，下一次叫起时 `put_the_switch_requested_from_the_next_batch_in_effect` 换上它、判出停着，后台入口不删。挂载运行库走的是后台入口：`session.run_compaction_in_the_background_for_at_most_one_publish(`（`crates/singlefs-mount-runtime/src/lib.rs:578`），所以产品路径上这一态会一直留到切换解除或下一次显式入口，期间 R 被排他、前台在 R 里取不到落点（实现员报告标「可能碰上假 ENOSPC（推的）」）。
条款字面站在代码这一边：「挂着切换到参与的卡一张都不剩的设置（第 3 条）：下一次整理入口与后台入口在任何写之前就报停着，删除续做也不走」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`）。定案「停下都删意图」与这句字面相左，代码在显式入口取定案、在后台入口取字面。这是写回时要二选一的一格，不是实现手滑。
推翻条件：在后台入口造出上述序列后，盘上意图被删（说明有别的路径补删）；或条款作者认定后台入口那一句不受定案影响。

### 1.3 删意图那一次归删除类不判式子（定案 3）：对，范围比字面宽

条款（D28 已定项 1「接线」）：
「删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子，删除类发布的落点那一判照判」（`.claude/kb/decisions/28-挂载期承诺量.md:38`）。
整理这一侧的条款（D26 已定项 3）：「2. **整理不是释放路径，它走常规准入**，批次最坏占用计入「已承诺预留」。」（`.claude/kb/decisions/26-后台整理与放置回收.md:103`）。
两句字面都没给「整理的删意图那一次」免式子；它由定案 3 加，kb 还没写回（背景材料也这么说）。代码按定案做：
`&& !deletes_only_the_compaction_intent(deletion_and_lineage, relocations)`（`crates/singlefs-core/src/transaction.rs:13405`）只摘掉式子那一判；落点那一判在它上面照判（同函数 `demand_is_judged` 之前的落点段，注释「落点那一判在上面照判」）。`deletes_only_the_compaction_intent`（`transaction.rs:13446`）要 logged_ops 那一步是 `DeleteTheCompactionIntent` 且 `relocations.is_empty()`；`DeleteTheCompactionIntent` 只在整理步的发布里构造（`grep -n 'LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent' crates/singlefs-core/src/*.rs`：构造点只有 `transaction.rs:10946`、`:10953`），用户改动与删除类发布不会漏进来（量的）。
签发那一次（`IssueTheCompactionIntent`）与搬了单元的批仍判式子，变异「带一批的也免判式子」钉着（`crates/mutations.tsv` 该行；证红命令在实现员报告 `~/.local/share/singlefs/work/m4-compaction-pause-deletes-intent-writer-2/result.md` 的「要主 agent 跑的命令」一节，我没跑）。

越出字面的一格：判据是结构（只删整理意图、不搬单元），不是来历（暂停 / 出错停下），所以还放过三种发布：
1. 意图达成而 R 已按记录全空、不带一批的 `DeleteOnceTheRegionIsEmptyByTheRecords`；
2. 区域扫描拒了 R、R 里有隔离槽、R 里一个能搬的都没有而 R 不空的丢意图（`CompactionIntentStep::Drop` 的其余调用点）；
3. 最后一批的计划里单元全是这次发布本来就重写的节点，于是 `relocations` 为空的那一批（`mounted_session.rs` 注释「删意图的那一批计划里的单元都是这次发布本来就重写的节点时，它就是一次只删意图的发布」）。
这三种盘上写的东西与暂停那一次相同（logged_ops 删一条、重写固定点），空间由 checkpoint 保留池出，式子那一判对它们没有用武之地，我推它无害；但定案原话是「删意图那一次」，要不要罩住这三种由主 agent 认。要收窄只能给发布路径另传一样标记（实现员报告「读法」第一条同说）。
推翻条件：这三种里有一种在满盘上因没判式子而写穿（落点那一判不会替它挡：它只判有普通分配的发布）。

### 1.4 D4 已定项 4 读接全（r3 收尾）：对，一处弱漏

条款：「写入口每次发布写系统配置槽之前读最新那条根的实例表，读到的记在写入口自己的账上，发布落盘之后转进分配器（`persist_the_publish_or_freeze_it`）」（`.claude/kb/decisions/04-校验和位置.md:73`）。
代码：`pool.count_the_reads_before_the_system_configuration_writes(allocator);` 在 `persist_publish_writes` 之后、判 `Err` 之前（`crates/singlefs-core/src/transaction.rs:1658`；冻结重发 `:1692`）：落盘中途失败也转进账（读是真发到盘上的），比条款字面「落盘之后」宽一步，方向保守。重算影子账、取号前取单元清单、建上一版之后的重建读（`mount::rebuilt_allocator`）、逐盘核落后支都接了计数层（diff 367–429、437–484、609–689）。读的是别的版本的只计数不搬，条款是：「读的是别的版本、认不出的只计数、不搬」（`.claude/kb/decisions/04-校验和位置.md:73`），diff 里只改了说明，行为是上一轮的。

弱漏：抬 F 那一串先写系统配置槽，成功时「这一读留在写入口上，这一串第一次空发布落盘时一并转交」（`crates/singlefs-core/src/mount.rs:5610`）；若第一次空发布在落盘前就被拒，这一串的写入口随错丢掉（`mount.rs` 同函数注释自己写了「第一次就在任何写之前被拒的，这一串什么都没落盘」），那一读没进账。后果有限：之后每次发布写配置槽前都会再读最新根的实例表，同一组还会被读到（推的，没造）。
另一处靠注释论证、没有用例：`publish_without_units`（`transaction.rs:1749`，落盘在 `:1764`，没有分配器参数）不转交；`mount.rs:8798` 的注释论证「取号写读到的由接着那次经分配器的发布转交……零单元发布……读的就是取号之前取单元清单读过的那一片」，把这一支说成已在账上。这是读码推的前提：「那一片」确是取号前取单元清单读过的，没有用例直接点名 `count_the_reads_before_the_system_configuration_writes`（`grep -rln 'count_the_reads_before_the_system_configuration_writes\|locations_read_with_one_copy_failing_before' crates/singlefs-verification/tests crates/singlefs-core/tests` 零命中，量的）；端到端看账的用例有 `unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs` 等，钉没钉到零单元发布那一支我没核。
推翻条件：造一个坏了一份的实例表片，让抬 F 的第一次空发布被拒，之后一直没有别的发布，卸载前账上仍缺这一组。

### 1.5 D18 已定项 11 同实例点名窗口：对

条款（`.claude/kb/decisions/18-块里携带什么信息.md:322`）：
「窗口 = 计数器在 (这块盘 tail 的计数器, 所选那一版末条记录的计数器] 的那几条记录，每条取池里任一块盘环槽上同计数器、自证过的那一份（只按计数器认，不另限记录的实例；实例的约束在 tail 与所选那一版末条记录同实例这一句上）」
「tail 的实例代号不是那一版末条记录的实例代号、窗口里有一条哪块盘上都取不到、或窗口比环的槽数还长，窗口当作覆盖那一版的全部单元、照旧拒」
代码逐项对：
- 区间 `(tail, 末条]`：`for counter in (tail.counter + 1)..=version_journal_position.counter {`（`crates/singlefs-core/src/mount.rs:7167`）。
- 跨实例：`if tail.instance_generation != version_journal_position.instance_generation {` 交回 `None`（`mount.rs:7158`），`None` 时 `is_none_or` 让全部差异单元算缺（`mount.rs:7126`）。
- 取不到：`if !is_the_counter_found {` 整个交回 `None`（`mount.rs:7191`）。
- 记录只按计数器认：`.filter(|record| record.counter == counter)`，不比实例。
- 窗口比环长：没有显式分支，靠环槽被盖掉让第一个计数器读不到、落到上一条；盘的环同步时与条款同结局（推的，没造分叉环）。
- 调用方保证 tail 小于末条：`if newest_system_configuration_journal_tail >= version_journal_position {` 先交回 `None`（`mount.rs:7090`）。
条款没写、代码多出的：同一计数器几块盘各有一条时取并集（`slots_named_by_the_records_after_the_tail` 文档），点名项两份位置条目的槽都收；都只多算缺，不多放行，方向保守，分叉盘本身另欠在 D18 的 C990（`.claude/kb/decisions/18-块里携带什么信息.md:322`）。
「点名是不是全集」这个前提：点名项取自这次发布重写的全部角色（`transaction.rs` 里 `named_unit_of` 对 `rewritten` 逐个取、搬迁副本另加），与写出的单元同源；判决那一格标「推的（两腿）」，反向检查池级 checker 还没有，不算本轮新漏。

### 1.6 窗口外缺的按读到一份坏处理（D4 / D18）：对

条款：「缺在窗口之外的单元按本盘收过处理，是落盘之后坏了的单份，与「系统配置跟得上、只缺几份单元」同一类，按读到一份坏处理、进搬迁路径」（`.claude/kb/decisions/18-块里携带什么信息.md:322`）。
代码：窗口外的差异单元各报 `reader.note_that_a_unit_was_read_with_one_copy_failing(`（`crates/singlefs-core/src/mount.rs:7130`）；取号之前逐盘核放行后才把那一层的组记进分配器账（`mount.rs:8239`，在 `!devices_without.is_empty()` 的拒绝返回之后，拒了随错丢，与文档一致）；会话改动入口经 `reading_and_counting_the_other_unit_reads_with_one_copy_failing` 走同一层（`crates/singlefs-core/src/mounted_session.rs:2970`）。
取号写会把跨盘见证值写到没收过单元的盘上，之后这块盘不再「落后」，窗口外的判不再发生；条款自己承认这是见证不全（D18 的 C991、C990），本轮不新增。
推翻条件：落后盘上窗口之外缺的单元，可写挂载放行后账里没有这一组（`other_unit_reads_with_one_copy_failing` 不加），或加了而下一次用户改动之后那一批没换落点。

### 1.7 D28 已定项 1 不判名单、落点照判：对（按定案）

见 1.3：式子不判、落点照判。D28 名单字面是删除意图那一族，整理的删意图是定案加的；写回 D28 时要把整理意图的删除那一次并进名单，否则条款与代码字面不符。环窗口 W 的上界是按删除批算的（D28 同一段），整理的删意图只重写 logged_ops 一条与固定点，新占槽数不超过删除批（推的，没量）。

### 1.8 D26 已定项 13 第 4 条字面与 D23 第 405 行：与定案相左，待写回

字面三处与定案不齐，实现按定案走，写回时要改：
- 「挂着的时候 GPU 出错（含重交 3 次都超时），正在算的那一批放弃（意图留在盘上，目标态单调，已定项 2），那个后台任务停下」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`）：定案 2 之后意图不再留盘。
- 「下一次整理入口与后台入口在任何写之前就报停着，删除续做也不走」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`）：显式入口现在在停着时先删意图（预算没用完），后台入口仍一个写不发。
- 「后台、可续做、非决策路径（重平衡、墓碑回收、scrub、整理）的分配失败 ⇒ **暂停那个活并记进意图**」（`.claude/kb/decisions/23-journal的角色与格式.md:405`）：整理这一支现在是「暂停 = 删意图」。
D26 已定项 3 第 2 句「整理不是释放路径，它走常规准入」对删意图那一次也要加例外。

### 1.9 I-5.6、I-1.11

I-5.6（`crates/singlefs-checker/invariants/07-分配器.md:11`）：停下的各路径（见 1.1、1.2）每一版盘上要么意图活着 R 排他，要么意图已不在；唯一会留「意图活着」的是后台入口不删与删意图自己被拒，这两种 R 仍排他，I-5.6 不破。
I-1.11（`crates/singlefs-checker/invariants/01-块自描述.md:17`）与 D21 已定项 10：diff 里没有 `recovery.rs`，也没有碰映射 key 集合的挂载核；读 `PreviousVersion::units` 改成泛型读者只换读者类型。我没发现冲突。

## 二、第三节四条攻击面

### 面 1 整理暂停与出错停下删意图：站住，两处边角

崩溃三段（删意图发布 = 现成的 `CompactionIntentStep::Drop` 单次发布，单元与记录写在屏障前，根槽 FUA 在后）：
- 之前、途中（根槽 FUA 之前）：盘上是旧版，意图活着，挂载续做照 D8「续做是每次恢复都跑的常规动作」把它做完（`crates/singlefs-core/src/mount.rs:8274` 起 `compaction_intent_continued_at_the_mount` 的文档）；R 排他靠读活意图推出，I-5.6 不破。
- 之后：盘上意图已不在，挂载不续做、不签，下次整理从头签发，`c0` 取新签发那一刻，比上一条大。
- 内存与盘不一致的窗口：删意图发布 `Err`（冻结）时分配器仍有活意图与 R 排他，`current` 没换；重发成功后 `resend_the_frozen_publish` 按那一版 logged_ops 重装或撤掉活意图（`mounted_session.rs:3286`）。
新崩溃流 `compaction-paused-short-of-space-dropping-the-intent` 枚举了这三段，登记在 `.claude/gate.d/stage-inputs.tsv:161`；一趟开头停着时删意图（补跑那几批签下的意图）没有崩溃枚举，只有普通用例（见格 8）。

「归删除类不判式子」有没有放过别的发布：只放过 `LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent` 且不搬单元的发布（1.3 的三种），构造点只在整理步（量的）；用户改动、删除类、快照类发布走不到这个合取。

一趟开头停着时删与不删的分界（逐入口追）：
- 显式入口与可写挂载续做：预算取 `SinceTheLastUserChange`，`publish_budget_of_a_compaction_run` 末尾 `.max(1)`（`crates/singlefs-core/src/compaction_plan.rs:1552`），恒大于 0，停着就删。
- 后台入口：判起跑时停着就不起跑（`mounted_session.rs:3379`），不删（1.2 的缺口）。
- 后台入口判过起跑、起跑前排空待发：排空发成用户改动会把 `publishes_since_the_last_user_change` 归零（`compaction_plan.rs:1026`），预算恒大于 0；排空没有待发则状态不变。
- 一趟之中：每批开头判出停着走 `advance_compaction_by_one_publish` 的 `else`，删意图并停。

边角一：后台入口不删（1.2）。
边角二：满盘上删意图自己被拒。删意图要重写固定点，落点判照判；logged_ops 删空后没有普通分配、式子与落点两判都不判，但固定点取不到落点（`PlacementRefused`）时 `pause_by_dropping_the_live_intent_or_hand_up` 把这个错交回，一趟结局变成 `Failed(Batch(..))` 而不是 `Stopped(PausedShortOfSpace)`，意图留盘、R 照排他（`mounted_session.rs` 的 `CompactionRunFailure` 文档已写这一态）。旧做法暂停不写盘，永远成功。这时前台在 R 里取不到落点，与本来满盘叠加。条款没写这一格（实现员报告第 2 条同说）。
推翻条件：造一个只剩零个自由槽的池，让一批被拒后暂停：若一趟结局仍是 `Stopped(PausedShortOfSpace)` 且意图已删，则边角二不成立。
再提一笔代价：被空间拒的暂停现在每趟多一次发布（签发 + 删意图各占一个根槽），受 `B_gc` 与回退门约束，有界。

### 面 2 同实例点名窗口放行：站住

逐个问法（代码行见 1.5）：
- 跨实例：tail 的实例代号与那一版末条记录不同，交回 `None`，全部差异单元算缺，拒。与条款同。tail 的实例代号大于那一版的（盘比所选版新）先被 `newest_system_configuration_journal_tail >= version_journal_position` 当作不落后（`mount.rs:7090`）。
- 记录槽自己坏：窗口里某个计数器在哪块盘上都解不出（读不出、自证不过、被环盖掉）整个交回 `None`，拒；只在落后盘上坏而别的盘有，照样取得到。这是失败关闭：一条窗口记录两盘都坏就永远拒可写，条款同意（「窗口里有一条哪块盘上都取不到……照旧拒」，`.claude/kb/decisions/18-块里携带什么信息.md:322`）。
- 两块盘都落后：每块盘用自己的 tail 各算一个窗口，记录读取走全部盘；两块盘都没有末条记录时，窗口的末几个计数器哪块盘都取不到，交回 `None`，拒。方向对。
- 窗口外缺的进计数与搬迁路径之后，卸载与下一次挂载：账只在内存；挂载之后若没有用户改动，不搬（D4 条款「挂载之后一直没有用户改动就不搬」，`.claude/kb/decisions/04-校验和位置.md:71`）；取号写已把见证值写到这块盘，下一次挂载它不再落后，那一份坏副本要等被读到才再计一次。与条款相符，不新增缺口。
推翻条件：造一块落后盘，窗口里某个计数器只在落后盘自己的环上有且被盖掉一圈（窗口比环长），看是否仍放行。

### 面 3 r3 收尾读接全：站住，一处弱漏

`transaction.rs` 写入口轮换那一读：计数层包住 `effective_rollback_floor_rereading_unreadable_reads_once` 的读者，读到的存进写入口 `RefCell`，由 `persist_the_publish_or_freeze_it`、`resend_the_frozen_publish`、抬 F 的错误路径转交（`transaction.rs:1658`、`:1692`，`mount.rs:5614`）。`mount.rs` 一族：`rebuilt_allocator`（最新根的实例表与各根的账）、`raise_the_floor_through`（影子账；错误路径把读记进换回去的分配器）、取号前取单元清单（`establish_instance`）、逐盘核（`devices_passing_the_check_before_the_acquisition`）、会话改动入口落后支。账各份分配器拷贝共用（`Arc<Mutex<..>>`，`crates/singlefs-core/src/allocator.rs:1945`），分配器换回去不退账。
读别的版本的只计数不搬：本批只改了文档串，行为沿上一轮；读 `allocator.rs` 的类别列表与 D4 的类别句一一对得上，我没发现多或少一类。
弱漏在 1.4：抬 F 先写配置槽成功、第一次空发布落盘前被拒；`publish_without_units` 不转交靠注释论证。

### 面 4 三处改动交叉：没找到互相踩的路径

追过的交叉：
- 删意图发布与可写挂载落后支：取号写把见证值写到每块盘，之后落后支判都落在「不落后」；整理的发布本来就不过落后支（落后支只在用户改动、卸载、抬 F、回退入口判），删意图不新增。
- 删意图发布与读到只坏一份另发的那一批：另发的那一批只在用户改动发成之后发；整理的删意图不是用户改动，不触发它，也不清账。账各份拷贝共用，删意图发布被拒、分配器换回时账里的读保留。
- 删意图时 R 排他仍在，另发那一批取落点绕开 R；删意图落盘之后排他撤，之后另发的落点可进 R，此时盘上已无意图，I-5.6 无从判。
- 删意图发布自己的配置槽轮换读也进账（走同一条 `persist_the_publish_or_freeze_it`），不丢。
这是一条腿单次读码的「没找到」，按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」不支撑「没打中」。
推翻条件：造一次挂载同时有落后盘窗口外坏的单元、停着的活意图和第一次用户改动，核账里的组、盘上意图、搬迁后落点三者不矛盾。

## 三、第四节八格

### 格 1 走不到的理由二次查实：成立

被问的那一支：`let dropped_on_the_stop = if budget == 0 {`（`crates/singlefs-core/src/mounted_session.rs:4450`）。追 `run_compaction_on_with_the_region_check_of_the_intent_deletion` 的全部调用点（`grep -n 'run_compaction_on_with_the_region_check_of_the_intent_deletion(' crates/singlefs-core/src/mounted_session.rs`：`:3192` 显式入口、`:3563` 后台入口、`:4308` 的 `run_compaction_on`，后者另被 `mount.rs:8296` 挂载续做调用）：
- 显式入口、挂载续做：预算规则 `SinceTheLastUserChange`，值 `.max(1)`（`compaction_plan.rs:1552`），不为 0。
- 后台入口：预算规则 `AtLeastOne…` 带 `.max(1)`（`mounted_session.rs:4445`）；只有 `LeftOfTheBudgetGivenByTheLastUserChange` 能为 0，它只由 `whether_the_background_compaction_starts` 在「删除队列不空」或「接着上一趟」里交出（`:3436`、`:3449`）。这个函数交出之前刚在 `:3379` 判过 `compaction_stays_stopped`，为真就返回没起跑。
- 判起跑与跑之间，同一次调用里只有排空待发会改分配器；排空发成的是用户改动，把已过去的发布数归零（`compaction_plan.rs:1026`），预算不再为 0；排空没有待发则 `compaction_stays_stopped` 的输入（`whether_the_task_runs_the_next_batch` 是 `&self` 的只读判，`background_compute.rs:1504`）不变。
所以「预算为 0 且一趟开头判出停着」走不到（读码推的，没造）。前提是上面这条「判起跑与跑之间无别的写者」，会话 `&mut self` 保证。
留着它与定案冲不冲：运行时不冲，因为走不到；但它在代码里写下一条行为（注释：「没有活意图或预算用完就一个写都不发」），比定案「停下都删意图」多一个条件。真走到了，结果是意图与 R 排他照留，I-5.6 仍守住，只是不齐定案。我的意见是留着当根环预算护栏可接受，写回 D26 时把「预算用完不删」写进条款，或把这支改成不判预算。
推翻条件：造出 `LeftOf…` 预算为 0、同一次叫起内 `compaction_stays_stopped` 从假变真的序列。

### 格 2 类型与名字：成立，附小命中

- 删掉的名字全仓零命中：`grep -rnE 'pause_the_live_intent|resume_the_live_intent|IntentDropOnThePauseRefusedShortOfSpaceUndecided|IntentDroppedOnAPause|compaction_stays_stopped_pausing_the_live_intent|pause_or_hand_up' crates --include=*.rs --include=*.tsv --include=*.md` 无输出（量的，含测试与 `crates/mutations.tsv`）。
- 新名各只定义一次：`drop_the_live_intent_on_a_stop`（`mounted_session.rs:5884`）、`compaction_stays_stopped`（`:4277`）、`write_bandwidth_limit_of_the_pacing`（`:565`）、`deletes_only_the_compaction_intent`（`transaction.rs:13446`）、`slots_named_by_the_records_after_the_tail`（`mount.rs:7152`）。
- 背景材料把 `CompactionIntentStep::Drop` 列在「新加」里不准：它在 HEAD 的 `transaction.rs:10653` 就有（`git show HEAD:crates/singlefs-core/src/transaction.rs` 量的），本批没动它；新的是 `CompactionPublishKind::IntentDroppedOnAStop { stop }`。
- 新 `expect`：`usize::try_from(singlefs_format::JOURNAL_RECORD_BYTES).expect("一条记录 4096 字节")`（`mount.rs:7162`），依赖常量恒为 4096、装得进 `usize`；同文件 `:6994`、`:7216` 同写法。依赖的不变量成立。实现员报告写「走不到的理由：无」，这一处是新增的 `expect`，该列进去。
- 小命中（不是门禁会拦的）：注释里有序数写法，规则字面「名字、注释、文档、输出文字都不用序数或位置代称」：`mount.rs:5610`「第一次空发布」、`:7074`「第二条位置条目……第一条」、`:7147`「第一个计数器」、`:7835`「第一次读」，`transaction.rs:13869`「第一段 16 个取样点」。门禁 ordinal-names 格剥掉注释不判（`.claude/gate.d/code-source-discipline.sh:8` 的说明），所以不会红，规则字面算命中。
- 形参名：`mount.rs:1740`、`:1946`、`:1978`、`:2046` 等七八处 `devices: &Reader`，名字说是盘表、类型是读者。

### 格 3 不兼容旧代码与旧格式：成立

`grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-batch2-code-r1-diff.md` 零命中（量的）。diff 里没有版本号常量、迁移、`legacy_ok`；删掉的两个函数与一个错误成员是本批功能的旧写法，不是兼容分支。

### 格 4 不新建判定库：成立

读过的 diff 全在 `crates/singlefs-core/src/`，没有判定存放；六份测试源码（`writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs`、`crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs`、`crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs`、`compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs`、`compaction_off_the_calling_thread_and_admission.rs`、`compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs`）里 `grep -nE 'rocksdb|sqlite|\.tsv|prune'` 只命中注释里引 `crates/mutations.tsv` 的「判别力」说明，没有读写。

### 格 5 所有任务多机可用：成立

同样那六份加 `common_compaction_prefix/mod.rs`：`grep -nE 'std::env|env!|hostname|/home/|/tmp|std::fs|File::'` 在后五份与公共模块零命中；前两份只命中注释里的 `crates/mutations.tsv`。线程数的问题单列格 6。

### 格 6 测试优先多线程：不成立（待主 agent 跑门禁坐实）

两份新用例确实按线程切片，但线程数取法不经登记位：
- `let threads = std::thread::available_parallelism()`（`crates/singlefs-verification/tests/writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs:1096`）。
- 同写法在 `crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:457` 与 `:682`。
- 同目录别的用例经登记位：`singlefs_verification::heavy_threads::default_heavy_threads(),`（`crates/singlefs-verification/tests/crash_enumeration_frozen_publish_resent_stream.rs:544`）。
- 门禁 heavy-thread-defaults 格把 `available_parallelism` 当「自己定并行度默认值的写法」：`thread_default_form("available_parallelism", r"\bavailable_parallelism\b"),`（`.claude/gate.d/code-tooling.sh:5251`），扫 crates/；排除表 `.claude/heavy-thread-exempt` 里 `available_parallelism` 的五行都是 `scripts/experiment-apparatus/` 下的装置，没有这两份（`grep -n 'available_parallelism' .claude/heavy-thread-exempt` 量的）。`crates/singlefs-verification/tests` 下用 `available_parallelism` 的只有这两份（`grep -rln` 量的：2）。
我没跑门禁，所以「判红」是读码推的；跑法在第四节第 1 条。
推翻条件：第四节第 1 条退 0。

### 格 7 带 --item 的命令与项：成立（`--list-items` 待跑）

登记行在：`compaction-paused-short-of-space-dropping-the-intent	crash-case:compaction-paused-short-of-space-dropping-the-intent-stream	-`（`.claude/gate.d/crash-point-replay-items.tsv:78` 前两列，量的），与邻行同形；`.claude/gate.d/stage-inputs.tsv:161` 的键 `crash-case:compaction-paused-short-of-space-dropping-the-intent-stream` 指的测试函数 `full_enumeration_of_the_compaction_paused_short_of_space_dropping_the_intent_stream_is_exhaustive_and_clean` 在测试文件里（`crates/singlefs-verification/tests/crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs:183`）。能不能被 `--list-items` 列出我没跑。

### 格 8 新崩溃流多罩了什么：成立，附两处射程限

两句都核到：
1. 暂停流多罩「最后一批被准入拒、暂停删意图、重新签发」：测试 `the_flow_drops_the_intent_on_the_pause_and_issues_it_again_on_the_same_region` 断言重挂之后的步序是签发、一批、`CompactionIntentDroppedOnAStop`、签发、最后一批，删意图那一版 `logged_operations` 空、不搬单元、R 里还有未释放记录，重新签发的 `c0` 比第一条大（该测试文件内，函数起于 `:93`）。HEAD 之前的流里没有 `CompactionIntentDroppedOnAStop` 这个步种（它是本批新增，`grep -rn 'CompactionIntentDropped' crates/singlefs-verification/tests` 只命中本批改到的几份）。
2. 出错停下流与它逐步相同：`the_flow_stopped_after_a_compute_error_writes_exactly_what_the_paused_flow_writes` 比两条流每步种类与 `retained_operations()`（该文件 `:151` 起）。
射程限：第一，只钉「写盘相同」，出错停下与暂停在内存里的停下状态（任务停着、等切换）不在枚举里，由普通用例管。第二，一趟开头停着而删补跑签下的意图（`BackgroundTaskStoppedAfterAnEarlierComputeError` 那一路，现在会写一个删意图发布）没有崩溃枚举；它的盘上形状与暂停那一次是同一种发布，但「删意图那一次可能发在删除队列不空的那一版上」（实现员报告「读法」最后一条）这一形没有枚举过。

## 四、要主 agent 跑的命令

没写模型，没有 `rerun.sh`。下面几条都没跑。

1. 核格 6：
`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 heavy-thread-defaults 一格; 范围=1 格; 为什么=核两份新用例取线程数的写法有没有被判' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults /home/fy5090/code/singlefs`
起在仓根。看 `TASK_SERVER_RESULT` 的 `exit=`：0 则格 6 的「不成立」撤回；1 并点名 `available_parallelism` 在那两份测试里则坐实。
2. 核格 7：`task-server/task submit -- bash scripts/crash/crash-point-replay.sh --list-items`（前面照 `.claude/hooks/run-scope-guard.sh` 文件头补范围声明：种类 items）。起在仓根。看输出里有没有 `compaction-paused-short-of-space-dropping-the-intent` 一行。
3. 命名检查（两份测试文件与产品代码都动了 `.rs`；这一轮我没改 `.rs`，由改动方交）：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; 范围=4 格; 为什么=核这一批改动的命名' task-server/task submit -- bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names /home/fy5090/code/singlefs` 与 `task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/naming-lint.sh /home/fy5090/code/singlefs`。起在仓根；`exit=0` 过。
4. 面 1 边角二（满盘删意图被拒）要造序列，我没写装置；若主 agent 要坐实，可让实现员在 `compaction_off_the_calling_thread_and_admission.rs` 里加一条「空闲槽数压到只够 0 个固定点槽」的用例，我只给出推翻条件。
5. 格 1 的 `budget == 0` 走不到只是读码推的，没有探针；要核，可在后台入口调用点前后加探针，打印 `compaction_stays_stopped` 与预算，跑 `mount_runtime_*` 那几份。命令我没拼。

## 五、没做什么

- 没起任何命令（门禁、测试、`--list-items`、变异证红都没跑）；格 6 的判红、格 7 的列项、变异行的证红都是待主 agent 跑。
- 没替主 agent 采纳或出判决；「由主 agent 认」的几格：1.3 的范围、1.2 的后台入口缺口、格 1 写回的措辞。
- 没读禁读清单里的文件；没读别的腿这一轮的产出。
- 没验背景材料第二节「验证到的数」（任务号 11954、11981、11982、11763、11992）：这些数我当作主 agent 的观测，没复核，也没引用。
- 引 kb 条款时，长行（`18-…md:322`、`26-…md:251`）我抄的是完整的句子，不是整行；整行在被引文件里。其余引文都带了现查的行号。
- 面 2、面 4 的「站住」「没找到」都是我一条腿读码的结果，按规则一次抽样不支撑「没打中」。
- 没核 `crash_judge_gpu` 的 GPU 对拍侧是否要为本批改动补变更；本批 diff 里没有它。
