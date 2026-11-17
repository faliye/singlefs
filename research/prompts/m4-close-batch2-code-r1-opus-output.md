# m4-close-batch2-code-r1：攻方腿报告（Opus，2026-11-10）

立场：攻方（假设实现与条款、用户定案、文档串对得上是错的，造历史打穿它）。只攻背景材料 diff 里有的块，方案按 `crates/` 今天的实现谈（主工作区五份产品文件与 `research/prompts/m4-close-batch2-code-r1-snapshot/crates-sha256.txt` 逐份 sha256 相同，量的）。

## 复跑

模型目录 `research/prompts/m4-close-batch2-code-r1-opus-model/`，复跑：`bash research/prompts/m4-close-batch2-code-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新目录>`（整条交服务，命令在末尾「要主 agent 跑的命令」）。它经 `minimal-copy.py` 拷冻结副本、核五份产品文件 sha256、把两条探针追加进 `compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs` 末尾（要用那份文件里的私有辅助函数），只跑名字以 `attack_` 起头的两条。两条的断言都写在「打中」那一边：绿 = 打中、红 = 没打中，观测行以 `m4-close-batch2-attack:` 起头。

```
318dcdee277fdca8552f299f11dc36addc939c84e1e8dc1372e00295b0a35f86  probes/attack_background_entry_keeps_the_compensating_intent.rs
0a5b498d96be298001a3f76f629b47e3e49052d4177ed55a2ea51bde4217cd51  probes/attack_drop_refused_when_only_the_region_has_free_slots.rs
bdf0e8a7bea9bb341744090b3cc87d747046446d47f372ddb429a595e6671129  tools/rerun.sh
```

我什么都没编、没跑（子 agent 不交任务）；两条探针只过了 `rustfmt --check` 的语法解析（只报排版差异，没有解析错）。下面凡是没标「量的」的读数都是推的，判决要引的数等主 agent 复跑。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| 三·1 整理暂停与出错停下删意图 | **打中 A1** | 整理因出错停着、占用不超过阈值时，产品入口（后台入口）判起跑那一刻只判不写，补跑签下的意图与 R 排他一直留着；显式入口（没有产品调用方）同一刻却删。与「停下都删意图」、与文档串都对不上 | 读码（推的）＋探针一（待主 agent 跑） |
| 三·1 同上 | **打中 A2** | 暂停删意图那一次自己的落点也受 R 排他：R 之外没有空槽而 R 里有时，删意图那一次被拒，意图与 R 排他一直留着，前台被拒；放开 R 发同一次删意图就发得成 | 读码（推的）＋探针二（待主 agent 跑） |
| 三·1 `deletes_only_the_compaction_intent` 放没放过别的发布 | 没打中（一次抽样） | 产出 `DeleteTheCompactionIntent` 的只有整理那条发布路径；会话里能让这个合取改变判法的只有「logged_ops 里另有截断意图」，只在一趟开头停着删意图（删除续做之前）那一处出现，正是定案要免的那一次 | 读码 |
| 三·2 同实例点名窗口 | 没打中（一次抽样） | 跨实例、记录槽坏、两盘都落后、分叉盘、环转过一圈都落到「全部算缺」或「多点名」，只往严里偏；点名项罩全这次重写的每个角色 | 读码 |
| 三·3 r3 读接全 | **打中 A3**（轻） | 落后支的窗口放行在三处入口（会话读文件、抬 F／推抬 F、挂着回退）经裸读者调，窗口之外坏的那一份放行了却不记账；随后那次发布的轮换把这块盘的 tail 推平，那一份之后再也看不见 | 读码（推的），没造模型 |
| 三·4 三处交叉 | 打中（并进 A3） | 删意图那一次与整理别的发布一样不判落后支，发布末尾轮换照样把落后盘的 tail 推平；A3 那三处入口同形 | 读码（推的） |
| 四·1 走不到的理由二次查实 | `budget == 0` 走不到：**成立**；留着它与定案冲不冲突：**推不出**（算打中，指到 A1） | 三个调用点逐处追过；这一支本身走不到，但「停着而不删」那一形在后台入口照样走得到 | 读码 |
| 四·2 类型与名字 | 成立 | 删掉的四个名字仓里零命中（量的）；新名字说得出内容。`crates/singlefs-core/src/compaction_plan.rs:639` 那句文档串说停着就删，只对显式入口与续做成立（并进 A1） | 读码＋grep |
| 四·3 不兼容旧代码与旧格式 | 成立 | `grep -nE 'legacy|old_format|migrat'` 在 diff 上零命中（量的） | grep |
| 四·4 不新建判定库 | 成立 | diff 只在 `crates/singlefs-core/src/`，零命中 prune／rocksdb／sqlite／`.tsv`（量的） | grep |
| 四·5 所有任务多机可用 | 成立 | 三份新测试零命中 `/home`、`hostname`、`env::var`、`CARGO_MANIFEST_DIR`、读文件（量的） | grep |
| 四·6 测试优先多线程 | 合入那一版**不成立**；工作区已有没暂存的改动改掉 | 暂存区那一版两份新测试的线程数取本机核数 `available_parallelism()`，不取登记的每台默认重活线程数，排除表里也没有；写报告时工作区里这两份已改成 `default_heavy_threads()`（不是本腿改的） | `git show :<路径>` 与 grep（量的）；门禁那一格没跑 |
| 四·7 带 --item 的命令与项 | 成立（推的） | 合表第 78 行、`stage-inputs.tsv` 第 161 行都在，形与邻行同；`--list-items` 只读合表，没跑 | grep |
| 四·8 新崩溃流多罩了什么 | 两句成立；另有一处没罩 | 暂停流多罩那一段、出错停下流靠小用例钉住相同，都在；一趟开头停着删意图、一批开头不补跑删意图这两处删意图的发布，不在任何一条录制流里 | 读码 |

哪一条最脆弱：A2，见末节。

## 攻击面一：整理暂停与出错停下删意图

### A1：后台入口停着时不删意图（打中，内部矛盾＋越位）

**历史**（每一步指到许可它的那一句）：

1. 可写挂载选 `gpu`，一批算着出错 ⇒ 那一批放弃、单发一次只删意图的发布，整理停下。这一步照定案做了。
2. 停着而最满那块盘超过补跑阈值 ⇒ 整理照常起跑、每批在 CPU 上补跑，从头签发：照 `.claude/kb/decisions/26-后台整理与放置回收.md:251`（已定项 13 第 4 条）。用户改动之后那一趟至多一批，签发之后搬一批，R 没搬空，意图活着、R 排他。
3. 用户删文件，读数降到不超过阈值（同文件用例 `a_compensating_compaction_stays_stopped_from_the_next_batch_once_a_user_deletion_brings_the_reading_down` 就是这样造读数的）。
4. 调用方照常一次次调后台入口（产品里唯一驱动整理的入口；显式入口「回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方」（`crates/singlefs-core/src/mounted_session.rs:4569`））。判起跑的第一判「if compaction_stays_stopped(allocator, background_compute) {」（`crates/singlefs-core/src/mounted_session.rs:3379`）为真就交回 `NotStarted`，一个写都不发。
5. 之后每一次都是第 4 步，直到占用重新越线或一次切换解除停下。这一段时间里补跑签下的意图一直在盘上、R 一直排他。

**它与哪几句对不上**：

- 定案「整理暂停 删意图 下次重来」（`records/2026-10-13-里程碑四预开发.md:1589`）与背景材料写的「一批出错停下也删意图」：停着的整理在产品入口上不删。
- `compaction_stays_stopped` 的文档串「只判，分配器与盘一样都不动：停着时还有活意图的由推整理的那一步删」（`crates/singlefs-core/src/mounted_session.rs:4276`）：后台入口判出停着就不推，「推整理的那一步」在产品路径上走不到。
- 后台入口文档串把停着而还有活意图的来历只写成两种，「这一判之前才停着而还有活意图的，意图与 R 排他照留，等下一趟显式入口、补跑或解除停下」（`crates/singlefs-core/src/mounted_session.rs:3230`）：上面这段历史不是「这一判之前才停着」，是一直停着、补跑签下的意图，文档串漏了产品上最常见的那一种；「等下一趟显式入口」在产品里等不来。
- `BackgroundTaskStoppedAfterAnEarlierComputeError` 的文档串「停在这一刻：现行那一版里有活意图就单发一次只删意图的发布丢掉它」（`crates/singlefs-core/src/compaction_plan.rs:639`）：后台入口从不走到这个停机原因，它走的是 `BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError`。
- 两个入口对同一刻同一态判得不一样：显式入口与可写挂载续做（`CompactionRunBudget::SinceTheLastUserChange`，预算至少 1）一趟开头就删；后台入口不删。条款那一边，`.claude/kb/decisions/26-后台整理与放置回收.md:251` 写挂着切到一张卡都用不上的设置时「下一次整理入口与后台入口在任何写之前就报停着，删除续做也不走」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`）——那一刻要是有活意图（一趟留下的、上一趟预算或节奏截住的），后台入口照条款不写，显式入口照定案写一次删意图，两边各守一句，没有一处写明这一格归哪一句。

**这一批让它变了什么**：这一批之前同一刻后台入口撤掉 R 排他、意图留着（diff 里删掉的 `compaction_stays_stopped_pausing_the_live_intent`）；现在 R 一直排他。R 里的空槽在整理停着、占用不超过阈值的整段时间里不给前台用。实现员自己报过这一句（它报告「读法」一节「后台入口判起跑时不删」，交主 agent 定），背景材料第四节没把它列成一格。

**四句**：
- 分不分辨臂：分辨。被判的是「停下都删意图」这一个定案在产品入口上落没落地；显式入口那一臂删，后台入口那一臂不删，同一段历史上两臂判得不一样。
- 系统当时看不看得到判别它的东西：看得到。后台入口判起跑时手里就有现行那一版（`current`），有没有活意图读 logged_ops 即知，与显式入口一趟开头那一判读的是同一样东西。
- 满足的是哪一个分句：定案「整理暂停 删意图」管的是停下那一刻；这里的意图是停着期间补跑签下的，停下那一刻已删的是另一条。所以严格按字面，这一形打中的是文档串三处（4276、3230、639 那三句）与 D26 第 251 行两句之间没写明归属，不是定案字面——判成「内部矛盾」，对定案是「推不出」。
- 改法在这几格上中不中：「后台入口判出停着而现行那一版有活意图时起跑一趟只删意图」（实现员提过的另一种办法）修得到第 4、5 步，但与 D26 第 251 行「在任何写之前就报停着」那一句相撞，要用户定这一格归哪一句；「只改文档串写实（后台入口不删）」修不到 R 排他一直压着前台那一点。

**探针一**（`probes/attack_background_entry_keeps_the_compensating_intent.rs`）：用同文件已有的辅助函数造第 1、2 步（阈值 0%，显式入口推一次、钩子里调只供测试的「之后都当不超过」开关代第 3 步的读数下降，与 `every_wake_…` 那条用例同一种造法），然后后台入口连调三次、夹一次用户改动（建一个 inode）再调一次；最后显式入口推一趟作对照。期望（推的，待主 agent 跑）：后台入口四次都是 `NotStarted(BackgroundTaskStoppedAfterAComputeError)`，盘上意图与分配器上的 R 排他都还在；显式入口那一趟恰好发一版删意图。

用户动作放开扫：第 3 步之后用户做什么（写、建 inode、删、什么都不做）都不改变第 4 步，因为第 4 步那一判在任何用户动作之前、只看读数与停没停；推的，探针只取了「建一个 inode」这一种。

推翻条件：探针一红在「前提：补跑那一趟在 R 上签下意图」——说明补跑一趟预算 1 没签下意图，这一形要改造法；红在最后那条 `assert!`、观测行里后台入口有一次不是 `NotStarted` 或意图不在了——A1 不成立。

### A2：暂停删意图那一次的落点也被 R 排他挡住（打中，越位）

**历史**：

1. 整理在 R 上签发，R 排他（D26 已定项 1 第 3 条；I-5.6「活整理意图的区域里没有签发之后分配的记录」（`crates/singlefs-checker/invariants/07-分配器.md:11`））。
2. 意图活着期间前台的写都落在 R 之外（R 排他），一批一批整理把 R 里的单元搬到 R 之外，R 里搬空的槽回收之后成了空槽；R 之外被写满。
3. 下一批取目的地取不到 ⇒ 被空间拒 ⇒ 暂停：`pause_by_dropping_the_live_intent_or_hand_up` ⇒ `drop_the_live_intent_on_a_stop`。
4. 删意图那一次要重写固定点（记账树、分配记录树、中央映射树、树表；logged_ops 里另有条目时还有 logged_ops 根），要取新落点。文档串写「R 排他一直留到删意图那一版落盘」（`crates/singlefs-core/src/mounted_session.rs:5875`），落点只能在 R 之外取 ⇒ 取不到 ⇒ 被拒：「取落点被拒在任何写之前，盘上逐字节不变，意图留在盘上、R 照旧排他，」（`crates/singlefs-core/src/mounted_session.rs:5882`）。
5. 下一趟：活意图还在 ⇒ 重扫、计划、那一批又被拒 ⇒ 再删意图又被拒。每一趟都是 `Failed(Batch(..))`。前台的写同样只能落在 R 之外 ⇒ 被拒；推抬 F 的空发布同样要固定点落点。R 里的空槽统计上算空闲，谁都用不上。

**为什么算越位**：R 排他守的是 I-5.6——「同一版」里活意图的 R 内没有签发之后分配的记录。删意图那一次发出的那一版里意图已经不在，I-5.6 对那一版不适用（「没有这样的意图时报不适用」，同一行）；那一次写进 R 的固定点，在它落盘之前崩溃时只是上一版里没人记的空槽上的字节，上一版的分配记录里没有它们，I-5.6 照样成立（推的，没造崩溃状态）。所以「R 排他一直留到删意图那一版落盘」对删意图那一次自己的落点多挡了一层，文档串里「盘上每一版要么意图活着而 R 排他着，要么意图已不在」（`crates/singlefs-core/src/mounted_session.rs:5876`）推不出这一层非挡不可。而触发暂停的正是「空间不够」：删意图偏偏在最可能取不到落点的那一刻发。这一批之前同一刻是撤 R 排他、意图留着（I-5.6 那一红就出在这里，用户因此定删意图）；现在不撤也删不掉，空间只在 R 里的那一形从「I-5.6 红」换成「整理与前台一起卡死」。

**四句**：
- 分不分辨臂：分辨。「删意图那一次照 R 排他取落点」与「删意图那一次放开 R 取落点」两臂在这段历史上一臂卡死一臂发成。
- 系统看不看得到：看得到。发布路径知道这一次是 `CompactionIntentStep::Drop`（`drop_the_live_intent_on_a_stop` 自己就交的是它）。
- 哪一个分句：定案「整理暂停 删意图 下次重来」（`records/2026-10-13-里程碑四预开发.md:1589`）的「删意图」在这一形上做不到；文档串那一句「删意图那一次没发成…下一次整理再走到停下时再删」承诺的「再删」在这一形上永远删不成。
- 改法在这几格上中不中：「删意图那一次放开 R」修得到；「删意图被拒时退回撤 R 排他、意图留着」修得到卡死但回到 I-5.6 那一红，正是用户否掉的那一态；「不改」修不到。

**探针二**（`probes/attack_drop_refused_when_only_the_region_has_free_slots.rs`）：CPU 挂载，R 取第一块盘上有数据单元、两块盘同槽号空槽最多的那一段（I-5.6 那一行写 R 罩住每块盘上的同号槽，所以两块盘都留着 R 那段槽号），签发；然后在分配器里把 R 那段槽号之外的每一个空槽标成已分配（只供测试的直接标法，同文件 `allocate_the_tail_of_a_device_just_above_percent` 用的同一个方法），代第 2 步。显式入口推两趟（每趟预算 20）；再发一次用户改动；然后对照臂：撤掉分配器这一侧的 R 排他，经公开的 `transaction::publish_compaction_step(…, CompactionIntentStep::Drop, …)` 发同一次删意图，发成就换上那一版、再发一次用户改动。期望（推的）：两趟都是 `Failed`、一版不发、意图与 R 排他都在；用户改动被拒；对照臂删意图发成、之后用户改动发成。对照臂是我提的改法，**只在我的模型上量过（还没量，待主 agent 跑）、被攻过零轮**。

用户动作放开扫（推的）：第 2 步是「R 之外被写满」，由什么写满（数据写、建 inode、快照钉住的旧单元）不改变第 4 步，因为删意图那一次只要固定点落点；第 2 步里用户删文件、而释放的槽还没被抬 F 回收，同样写不进。我只造了「直接标满」这一种。

推翻条件：观测行 `drop_refused_setup` 里 `free_slots_in_the_region` 有 0（R 里本来就没空槽，这一形退化成真满，不分辨臂）；`drop_refused_trip` 的 `outcome` 不是 `Failed` 或意图不在（删意图那一次另有落点来源，A2 不成立）；对照臂也被拒（放开 R 也不够，改法碰不到这一格）。

### 改法表（每一格修哪一个打中）

| 改法 | A1 | A2 | 标注 |
|---|---|---|---|
| 后台入口判出停着而现行那一版有活意图时，起跑一趟只删意图（与显式入口一趟开头同一段） | 修 | 不修 | 推的；与 D26 第 251 行「在任何写之前就报停着」相撞，交用户定归属 |
| 只把三处文档串改成写实（后台入口停着不删） | 不修（只消矛盾） | 不修 | 推的 |
| 删意图那一次（`CompactionIntentStep::Drop` 与达成删意图）取落点时不排除 R | 不修 | 修 | 推的，待主 agent 跑探针二的对照臂；只在我的模型上、被攻过零轮 |
| 删意图被拒时退回撤 R 排他、意图留着 | 不修 | 修卡死、回到 I-5.6 红 | 推的；用户已否 |

### `deletes_only_the_compaction_intent` 放没放过别的发布（没打中，一次抽样）

试过的形状：
- 谁产出 `LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent`：只有 `publish_compaction_step_in_the_session` 里 `DeleteOnceTheRegionIsEmptyByTheRecords` 与 `Drop` 两臂（`grep -n DeleteTheCompactionIntent transaction.rs` 五处命中，两处产出、一处定义、一处消费、一处这个函数），用户改动、回退、另发那一批（意图照抄）、删除续做都不产出它。
- 合取「deletes_the_compaction_intent && relocations.is_empty()」（`crates/singlefs-core/src/transaction.rs:13458`）只在 `publish_has_an_ordinary_allocation` 为真时改判法；只删意图的发布有普通分配，只能是 logged_ops 里另有条目、要重写 logged_ops 根。会话里一趟的删除续做在整理发布之前走空，循环里的删意图（达成、K-c 丢、无可搬丢、暂停与出错放弃）都在它之后；只有一趟开头停着那一处删意图排在删除续做之前（文档串自己写了），而那一处正是定案要免的「停下删意图」。所以实现员把免判罩到达成与 K-c 丢那几种，在会话里不改变任何一次的判法（推的）。
- 有搬迁的一批不进：`relocations` 非空就不免，带一批的删意图照判式子（变异行「带一批的也免判式子」盯它）。
没试：回退把截断意图带回来之后、同一趟里先回退再整理（回退是挂着的公开入口，`roll_back_by_a_forward_publish`；实现员的截断用例就是这样造两条意图同在，但它直接走发布路径，不经会话一趟）。

### 删意图之前、途中、之后崩溃（没造，交代为什么）

这一格要崩溃状态枚举，本腿没写调枚举函数的原型：三条打中都不靠崩溃状态。读码看（推的）：删意图那一次是一次普通发布（固定点 COW、记录、根槽、轮换），崩在根槽之前落到意图活着、R 排他着的上一版，挂载续做（`compaction_intent_continued_at_the_mount`，`CompactionRunReach::ContinueTheLiveIntentAtTheMount,`（`crates/singlefs-core/src/mount.rs:8303`））接着推它；续做那一趟要是又被空间拒，就在挂载里再删一次；A2 那一形下挂载续做同样删不掉，挂载照常完成（整理不挡挂载），意图留着。这一处要不要另开一条录制流，见第四节第 8 格。本腿原型跑的崩溃状态数：0。

## 攻击面二：同实例点名窗口放行（没打中，一次抽样）

试过的形状（全读码）：
- 跨实例：「if tail.instance_generation != version_journal_position.instance_generation {」（`crates/singlefs-core/src/mount.rs:7158`）交回 `None`，调用方全部算缺，与这一批之前一样严。取号写的 tail 是（新实例，见证值），见证值与环里最大计数器之间住着上一实例没来得及轮换的记录：窗口按计数器认、实例不限，这几条的点名也算进去，只多算缺。
- 记录槽自己坏：某个计数器哪块盘上都解不出 ⇒ 整个 `None` ⇒ 全部算缺（往严里偏，C593 那一形照旧拒）。一块盘上坏、另一块盘上好 ⇒ 取到好的那一条。环转过一圈 ⇒ 槽里是别的计数器，「.filter(|record| record.counter == counter);」（`crates/singlefs-core/src/mount.rs:7179`）滤掉 ⇒ 解不出 ⇒ `None`。
- 分叉盘：同一计数器两块盘各一条不同的记录 ⇒ 并集，只多点名。
- 两块盘都落后、tail 各不同：逐盘各算各的窗口；本盘环里有那一版末条记录就不算落后（这一支没改）。
- 点名漏罩：窗口靠 `record.named` 判「这次发布写过哪些槽」，新根段里的树表、映射根、实例表、分配记录树根不在 `named` 里也没关系——点名项是「这次重写的角色各一项」（`transaction.rs` 第 2415 行附近 `named: roles_of_this_record` 那一段与整理一批那一段都按 `rewritten` 全列），树表、映射树节点、实例表片在重写的角色里。没找到写了而不点名的单元。
- 落后盘上的槽被窗口里的发布释放又复用：复用那一次的记录点名了它，进窗口，算缺。
- 窗口之外坏的那几份进计数与搬迁之后：卸载之前没有用户改动就不搬（账只在内存里）；下一次挂载时这块盘的 tail 已被取号与轮换推平、不再落后，第二条位置条目上那一份再也没有读会碰到它，直到释放之前读盘核。这是「一次性的检出」，但与平常第二份默默坏掉、读到第一份就停的基线一样看不见，不比基线差，我不算打中（推的）。

## 攻击面三：r3 收尾读接全

### A3：窗口放行在三处入口经裸读者调，放行的那一份不记账（打中，轻）

`units_of_the_version_missing_on_a_device_behind_it` 把窗口之外坏的那一份「各按读到只坏一份报给读者」，记账靠读者是数只坏一份的那一层。裸读者（盘表本身，`[(DeviceIdentity, Device)]` 与 `Vec<…>` 的 `PoolReader` 实现）用的是默认实现「fn note_that_a_unit_was_read_with_one_copy_failing(&self, _locations: &[LocationEntry; 2]) {}」（`crates/singlefs-core/src/recovery.rs:193`），什么都不记。diff 只把挂载取号之前那一判与会话改动入口那一判换成了数的那一层；另外两处调 `devices_behind_the_current_version` 的照旧给裸盘表：

- 会话读文件之前那一判：「devices_behind_the_current_version(devices, parameters, current_version_of_trees);」（`crates/singlefs-core/src/mounted_session.rs:2624`），在 `refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount` 里，`read_the_file` 先调它。
- 调用方输入对盘那一判：「devices_behind_the_current_version(devices, &parameters_of_the_pool, current);」（`crates/singlefs-core/src/mount.rs:7732`），在 `caller_inputs_agreeing_with_the_disk` 里；它的调用方是 `raise_rollback_floor_to_the_admission_ceiling`、`push_one_floor_raise_within_the_admission_budget`、`raise_the_floor_through`、`parameters_of_the_pool_agreeing_with_the_caller_inputs`、`roll_back_by_a_forward_publish`（`awk` 按行号找外层函数，量的）。

**历史**：挂着时一块盘落后（C585 那一形：发布途中取下的这块盘的旧快照换回来），它上面有一份单元落在窗口之外、坏了。接着调用方没有先发用户改动，而是挂着回退（或 `accept` 被空间拒时推抬 F）。这一判以前把这块盘判成落后、整次拒；这一批之后放行、不记账，那次回退（或抬 F 那一串）的发布末尾轮换把这块盘的系统配置推到新 tail，它不再落后；之后用户改动入口那一判（记账的那一处）看它不落后、不读它，坏的那一份再也不进账、不搬。读文件那一处同形，只是读不发布、不推平 tail，之后的用户改动入口照样记得到。

**对不上的句子**：D18 已定项 11 的用户定案 `c1-same`「窗口之外缺的按读到一份坏处理」（背景材料第一节表格那一行转述，原话我没去 records 里核）与 D4 已定项 4「可写挂载里的读都计数并搬」；diff 里 `allocator.rs` 那段文档串把会话改动入口的落后支列进「走它的读」、把取号之前逐盘核列进「读完另经…进账」，没列这两处，也没列进「没接这份账的是可写挂载之外的读」——这两处是可写挂载里的读，落在哪一类都没写。

**四句**：分辨（数的那一层与裸读者两臂在这段历史上一臂记一臂不记）；看得到（这一判当场读出了那一份坏）；满足的是「计数并搬」的「计数」分句；改法「这两处同样经 `reading_and_counting_the_other_unit_reads_with_one_copy_failing` 包一层」修得到（推的，`caller_inputs_agreeing_with_the_disk` 手里没有分配器，要从调用方传进来）。

没造模型：要在挂着时换一块盘的旧快照、再让它窗口之外坏一份，造法同 `writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs` 那一族，本腿没有余力写第三条探针。判成「打中（推的）」，等主 agent 现查那两行与 `crates/singlefs-core/src/recovery.rs` 第 193 行坐实。

### 别的读接全那几处（没打中）

- 写入口轮换那一读：落在写入口上，`persist_the_publish_or_freeze_it` 与 `resend_the_frozen_publish` 落盘之后（成与不成）都转交；零单元发布 `publish_without_units` 直接调 `persist_publish_writes`，不转交——它的调用点只在暖机与取号之后零单元写行，`mount.rs` 那段注释的理由（读的是取单元清单时读过的同一片、或这次挂载自己写下的那张表）我按读码认，没造反例。
- 抬 F 那一串：系统配置写成了、第一次空发布在任何写之前被拒时，写入口随错丢掉，那一读不进账；但下一次任何发布的轮换都再读同一张实例表、再记一次，只差这一次的计数（推的，不算打中）。
- `rebuilt_allocator`、`raise_the_floor_through`、取号之前逐盘核、整理回退门、后台入口读根环：数的那一层建了、出口都并账，出错早退的那几处整次挂载或整串抬 F 被拒、分配器随错换回（账各份拷贝共用），读码没找到漏并。

## 攻击面四：三处改动之间的交叉

- 删意图那一次 × 可写挂载落后支：整理的发布（含这一批新加的删意图那几次）不判落后支，发布末尾的轮换照样把落后盘推平。整理批这样做在这一批之前就有，这一批新加的是停下那几处多了一次发布（推的，并进 A3 那一句「推平之后再也看不见」）。
- 删意图那一次 × 读到只坏一份另发的那一批：另发那一批意图照抄（`publish_relocation_batch`），不产出 `DeleteTheCompactionIntent`，不进免判；它的目的地同样避开 R。A2 那一形下另发那一批也取不到 R 之外的落点、被拒，「推抬 F 再试一遍」同样要固定点落点（推的）。
- 一趟开头停着删意图（排在删除续做之前）× 删除队列不空：那一次发布照抄队列，不在任何一条录制流里（见第四节第 8 格）。

## 第四节八格

1. **走不到的理由二次查实**。`budget == 0` 那一支「let dropped_on_the_stop = if budget == 0 {」（`crates/singlefs-core/src/mounted_session.rs:4450`）逐处追调用点：
   - 显式入口（3192 行那一调）与 `run_compaction_on`（可写挂载续做唯一的调用点，`mount.rs` 第 8296 行）都给「CompactionRunBudget::SinceTheLastUserChange,」（`crates/singlefs-core/src/mounted_session.rs:3200`），预算是 `3S − 4 − 已过去的发布数` 再「.max(1)」（`crates/singlefs-core/src/compaction_plan.rs:1552`），恒 ≥ 1。
   - 后台入口给 `LeftOfTheBudgetGivenByTheLastUserChange`（可为 0，「publish_budget_of_a_compaction_run(root_ring_slots_per_region, 0)」（`crates/singlefs-core/src/compaction_plan.rs:1564`）减已过去的发布数）或 `AtLeastOne…`（`.max(1)`）。可为 0 的只有前者；而后台入口判起跑时第一判就是停没停，停着就 `NotStarted`，走不到这一趟。判起跑与这一趟开头之间：普通会话什么都不写，两次判读同一个分配器、同一份后台计算状态（待生效的切换在第一判之前已换上，第二次换是空操作），判得一样；攒批会话中间排空待发（「let pending_changes_published_before_the_run = match self.publish_the_pending_changes(」（`crates/singlefs-core/src/mounted_session.rs:7427`）），待发空就什么都不写，不空就发一次用户改动、「compaction.note_a_user_change_published();」（`crates/singlefs-core/src/mounted_session.rs:3023`）把已过去的发布数归零，预算又 ≥ 1；排空被拒就整次返回、不跑这一趟。
   - 只供测试的开关也到不了：强制「不超过阈值」的开关在后台入口第一判就生效，同样 `NotStarted`。
   结论：走不到**成立**（读码，三个入口都追了）。留着它与「停下都删意图」冲不冲突：这一支自己没有行为，不冲突；但它写下的那条规矩（停着、预算 0 ⇒ 意图与 R 排他照留）在产品上照样发生，只是换在后台入口判起跑那一刻（A1）。所以这一格答「推不出」：护栏护的那一格走不到，真正留着意图的那一格不在护栏里。变异行 7571（`budget == 0 || budget > 0`，抓得到）与 7579（换成按共用 `B_gc` 判，抓得到）都动的是另一个方向；把这一支删掉（恒删）没有用例会红，因为走不到。
2. **类型与名字**：`pause_the_live_intent`、`resume_the_live_intent`、`IntentDropOnThePauseRefusedShortOfSpaceUndecided`、`compaction_stays_stopped_pausing_the_live_intent` 在 `crates` 的 `.rs` 里零命中，旧步种名 `drop_on_the_pause`／`dropped_on_the_pause`／`DroppedOnThePause` 在 `crates`、`scripts`、`.claude/gate.d` 零命中（量的）。新名字都说得出内容；`deletes_only_the_compaction_intent` 罩到达成与 K-c 丢那几种，名字照实。diff 里 `expect` 只有 `JOURNAL_RECORD_BYTES` 转 `usize`（常量 4096）。成立；文档串的矛盾并进 A1。
3. **不兼容旧代码与旧格式**：成立（grep 零命中，量的）。
4. **不新建判定库**：成立。
5. **所有任务多机可用**：成立（量的：三份新测试零命中本机路径、主机名、环境变量、读文件）。
6. **测试优先多线程**：合入的那一版**不成立**，工作区里已经有人在改。暂存区那一版（`git show :<路径>`，量的）两份新测试的线程数取本机核数：`writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs` 第 1096 行、`crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs` 第 456、678 行都是 `let threads = std::thread::available_parallelism()`。登记位写「写坏或删掉这份文件，上面几个取法一律拒绝，不退回按核数算」（`.claude/heavy-thread-default:5`），检查单原文「该并行而没并行、线程数写死或覆盖，打中」（`.claude/rules/implementation-workflow.md:40`）；`.claude/heavy-thread-exempt` 没登这两份（grep 零命中，量的）。我开工读的时候工作区也是这样；写报告时再读，工作区里这两份已有没暂存的改动（`git status` 报 `AM`，`git diff --stat` 两份共 16 行增、12 行删，量的），改成「let threads = singlefs_verification::heavy_threads::default_heavy_threads()」（`crates/singlefs-verification/tests/writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs:1096`），另一份在第 457、682 行同改。不是我改的，不知道是哪个会话；主 agent 核这份改动是不是要并进这一批。门禁 code-tooling 的 heavy-thread-defaults 格管 crates 里这种写法（它的文件头点名 `available_parallelism`），命令在下面。
7. **带 --item 的命令与项**：`.claude/gate.d/crash-point-replay-items.tsv` 第 78 行（四列，形同第 77 行）与 `.claude/gate.d/stage-inputs.tsv` 第 161 行都在；`--list-items` 照它文件头「只读合表，不编、不跑」，推的是列得出。没跑，命令在下面。成立（推的）。
8. **新崩溃流多罩了什么**：两句都核到——暂停删意图流文件头写明比已有整理流多罩「R 里还有未释放的记录时单发一次只删意图的发布」与同一 R 上第二条意图；出错停下流只靠小用例 `the_flow_stopped_after_a_compute_error_writes_exactly_what_the_paused_flow_writes` 钉住录制流逐步相同。另有两处删意图的发布不在任何录制流里：一趟开头判出停着那一次（排在删除续做之前，删除队列不空时照抄队列），与补跑过一批之后下一批开头不补跑那一次（`advance_compaction_by_one_publish` 的 `else` 那一支）。它们写的是同一种只删意图的发布，崩溃状态的形状多半同暂停流（推的）；队列不空那一种的恢复要先走删除续做、再看有没有意图，与暂停流不同，没有枚举罩着。

## 哪一条最脆弱

**A2：暂停删意图那一次照 R 排他取落点。** 依据：暂停只在「空间不够」时触发，删意图那一次恰好在最可能取不到落点的那一刻发；它一旦被拒没有退路（文档串只承诺「下一次停下时再删」，而下一次的条件只会更紧），整理与前台一起卡在「意图在、R 排他、R 里有空槽没人能用」，正是用户为 I-5.6 否掉「撤排他留意图」之后换来的那一态的兄弟；I-5.6 本身并不要求删意图那一次避开 R。推翻它的现象：探针二的观测行里删意图那一趟发成（说明固定点另有落点来源），或对照臂同样被拒（说明放开 R 也救不了，脆的是别处）。

## 没打中的形状

- 三·1：`deletes_only_the_compaction_intent` 的罩法（见上，三种形状）；删意图之后紧接着在同一个 R 上重签：暂停流就是这一形，读码没找到 `c0` 取错。
- 三·2：六种形状（见上）。
- 三·3：写入口轮换、零单元发布、抬 F 那一串、重建分配器、取号之前逐盘核、回退门、后台入口读根环——除 A3 两处之外都并账（读码）。
- 三·4：另发那一批进不进免判（不进）。
取样范围：全部是读码，一次抽样；按规则「没打中」一次不算。

## 这一条腿自己的限度

- 两条探针都没编过：API 照同文件已有用例的写法抄，`rustfmt --check` 只证了能解析。编不过的话，最可能是探针二里 `dropped.units` 与 `dropped.rewritten` 的字段名（照 `compaction_off_the_calling_thread_and_admission.rs` 里 `dropped.rewritten` 的用法推的）。
- 探针一用只供测试的「之后都当不超过」开关代用户删文件降读数；探针二用直接在分配器里标已分配代「R 之外被写满」，标的槽没有分配记录，池级 checker 会判 I-3.1 不对（探针不跑 checker）。两处都是造前提，不进被判路径。
- A3 没有模型；A1 的切换那一形（挂着切到一张卡都用不上、手里有活意图）没写探针，只读码。
- 用户 2026-11-10 三条定案与 `c1-same` 的原话我只核了 records 第 1589 行那一句，别的照背景材料转述。

## 要主 agent 跑的命令

1. 两条探针（判决要引 A1、A2 的数就靠它）。在仓根起，`--frozen` 给开工那一刻的树（主工作区此刻五份产品文件与快照 sha256 相同，量的；脚本里先核一遍，对不上 `name=m4_close_batch2_attack_snapshot rc` 不是 0）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 compaction_on_the_cpu_while_stopped_after_a_gpu_failure 里攻方追加的两条 attack_ 探针; 范围=1 个测试目标里 2 条; 为什么=攻方腿核整理停着时后台入口留不留意图、暂停删意图在只剩 R 有空槽时删不删得掉' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-batch2-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r1-opus/rerun-1`
   看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=`（0 = 两条都打中；1 = 至少一条没打中或快照核不过；别的码是脚本或编译那一步出的错）；日志里 `name=m4_close_batch2_attack_snapshot rc=0`、`name=m4_close_batch2_attack_probes rc=`、`name=m4_close_batch2_attack_summary both_hits_reproduced=`，以及 `m4-close-batch2-attack:` 起头的观测行（`compensating_run`、`background_entry_while_stopped`、`explicit_entry_while_stopped`、`drop_refused_setup`、`drop_refused_trip` 两行、`drop_refused_user_change_with_the_region_excluded`、`drop_contrast_with_the_region_open`、`drop_contrast_user_change`）。日志与观测落在 `--work` 那个目录（`probes.log`、`observations.txt`、`snapshot-check.txt`），每次尝试的副本在 `rerun-1.attempts/attempt.*/copy`，脚本跑完自己删。编不过的话 `^error` 行会打出来，交承接的攻方腿改探针。
2. 第四节第 6 格（仓根起）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 heavy-thread-defaults 格; 范围=1 格; 为什么=核两份新测试取本机核数当线程数判不判红' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults /home/fy5090/code/singlefs`
   看：工作区此刻已改成 `default_heavy_threads()`，在工作区上跑期望 `exit=0`；要坐实「合入那一版不成立」，在暂存区那一版的树上跑（`exit=1` 且 `✗` 行点名这两份测试文件）。工作区那份改动要不要并进这一批由主 agent 定。
3. 第四节第 7 格（仓根起）：`task-server/task submit -- bash scripts/crash/crash-point-replay.sh --list-items`
   看：输出里有一行以 `compaction-paused-short-of-space-dropping-the-intent` 起头。被范围声明闸拒的话照拒绝说明补范围声明再交。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，上面三条与判决要引的数都没复跑，报告里没标「量的」的读数都是推的。
- 没写调崩溃枚举函数的原型，本腿原型跑的崩溃状态数 0；删意图之前、途中、之后崩溃那一格只读码。
- A3、A1 的切换那一形、三·2 与三·3 的没打中都只读码，一次抽样。
- 副本上的数（探针跑出来之后）不算入库装置上的数。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r1-opus` 里只有两份文本（`syntax-check.rs` 是两条探针拼起来给 `rustfmt --check` 解析用的、`rustfmt.txt` 是它的输出），没建仓副本、没建编译目录，没有要删的。
