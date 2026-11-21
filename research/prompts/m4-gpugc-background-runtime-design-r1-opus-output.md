# m4-gpugc-background-runtime-design-r1 攻方腿报告（云端攻方 Opus）

日期 2026-11-09。立场：假设「一把锁加周期线程、水位越线起跑」是错的。读的是主工作区 `crates/` 此刻的样子。数都标「量的」或「推的」；**这条腿没跑过任何代码，所有探针读数都待主 agent 跑（「要主 agent 跑的命令」一节），下面凡写「探针」的格，此刻都算「推的」。**

模型目录 `research/prompts/m4-gpugc-background-runtime-design-r1-opus-model/`。复跑（主 agent 交，详见「要主 agent 跑的命令」）：`task-server/task submit -- bash research/prompts/m4-gpugc-background-runtime-design-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <草稿目录>/copy --work <新目录>`。sha256：`src/runtime_design_attack_probes.rs` 6570eaa04a3937a36386b48d014e0f8b4514fe6bd513ea91f275b38c011d8969；`src/runtime_design_send_bounds_probe.rs` 380c80976c16d52ea911af482cbe7778cf7481dafae26e0f7b6cdc90b7cba02f；`tools/rerun.sh` 62acf47cdb12e9254733d104fb8d758132d651340abcb7081f68bc66db354efe。

## 各格判定一览

| 号 | 格 | 判定 | 一句话 | 证据状态 |
|---|---|---|---|---|
| A1 | Q1 Q3 Q6 | 打中，分辨不了「一把锁」与别的形态，是共用前提 | 产品路径批大小 `NotDecidedByTheClauses`：水位做完 GC 照样一批不发，而 K 的归属实验明写不出它 | 读码，代码与 kb 字面 |
| A2 | Q3 | 打中 | 不发布的那几种停不扣 `B_gc`、没有退避：满段池上每次叫起照做一遍 O(池) 的读（GPU 模式再加提交），「不需要清理就不跑」在这一支不成立 | 读码；探针 `name=wake_cost` 量随池大小 |
| A3 | Q3 | 打中 | 只要两次叫起之间有过用户改动，定时兜底那一支轮不到，一次叫起至多一批 | 读码；探针 `name=due_shadow` |
| A4 | Q3 | 打中 | 默认几何 `3S − 4 = 20`：一次用户改动至多 10 条意图、10 MiB（单盘），`T_dirty` 一次发布 256 MiB；闲置池扣完就永远停 | 算术加读码 |
| A5 | Q3 | 打中（依赖 K 取值） | 全空段到底额时前台写被拒 ⇒ 没有用户改动 ⇒ 预算回不来；一条意图做不完时前台与整理互相等，出路只有删东西、建 inode 这类别的改动或重挂 | 读码；探针 `name=floor_and_budget` 扫 K |
| A6 | Q3 Q2 | 打中 | 水位只在起跑查、循环里不再查：越线一点也跑到预算或无产出；F 那一条的已知轨迹是「整理越多 runs 越高」，每次用户改动都重新起跑 | 读码、kb；没量 F |
| B1 | Q5 | **打中，最重** | 整理发布挤掉回退候选集：`U G²⁰ U G¹⁰` 之后环里只剩 2 个用户状态，崩溃循环每次重挂给一份新预算，环里只剩 1 个；`B_gc` 立的时候防的就是这件事 | 读码；探针 `name=rollback_states`、`name=crash_loop_step` |
| B2 | Q5 | 打中（覆盖缺口，不是已证的坏） | 崩溃枚举流一条都不经后台入口、不经攒批会话进整理；活意图期间的用户改动、卸载带活意图再挂、挂载之后头一次叫起，这几种次序没有流 | grep 过；崩溃枚举没写原型 |
| B3 | Q5 Q1 | 打中 | 后台批写失败冻结发布：前台下一次写被拒，而重发没有会话入口，整理账也不跟着动；报警通道不管这一种 | 读码 |
| C1 | Q1 | **打中，分辨臂** | 锁内等 GPU：前台等锁等的就是 GPU，字面撞 D24 已定项 2「不许进前台路径」 | 读码；探针 `name=one_lock` |
| C2 | Q1 | 打中 | 只有 fsync 让得了路：`accept`、读、切换、卸载等一整趟（默认至多 20 次发布）；要全让路就得让调用方谎报 fsync | 读码；探针 `name=one_lock` |
| C3 | Q1 | 打中 | 卸载要 join 线程，条款说「卸载时不等还在跑的提交」；攒批会话没有切换入口，切换与卸载都要先排空，排空被拒时两样都做不成 | 读码 |
| C4 | Q1 Q3 | 打中（小） | 写带宽上限停下之后要等下一次叫起：配 L 字节/秒，定时那一支实际约 L/30 | 读码；探针 `name=bandwidth_per_wake` |
| D1 | Q2 | 打中 | 三字段对不上三量：`0` 一处读「内置默认」一处读「关」；不可回收字节非 zoned 恒 0；「停止线」与 D26 已定项 1「停机不靠水位」相左 | 读码、kb 字面 |
| D2 | Q2 Q6 | 打中 | 「腾出空段」两种读法：记录口径涨了，开段看的是另一个量；验收与水位读哪个要写死 | 读码；探针 `name=emptied_segment` |
| E1 | Q4 | 部分打中 | 开卡按卡串行、每张至多 5 分钟：只点一张卡也被没点名的卡拖，至多约 15 分钟；一进程一份实例加修好驱动重挂没有无代价的出路 | 读码；挂住的真驱动核不动 |
| F1 | Q6 | 打中 | 录制盘 `Rc<RefCell>`，过不了线程：崩溃枚举与块序列比对不能走真线程；现有用例全开 `ForcedToStartForTests` 与强制批大小，产品开关那条路没有一条用例 | 读码 |

没打中的写在「没打中的形状」。

## Q3　起跑之后空转与不收敛（主攻）

### A1　批大小：水位做完 GC 也不跑

正文六格没有一格问批大小，方案里也没有它。

- 历史：① 挂载走产品开关；② 水位真正实现，判「越线」；③ 周期线程调后台入口。走到这里停：「batch_size: CompactionBatchSize::NotDecidedByTheClauses,」（`crates/singlefs-core/src/compaction_plan.rs:570`）。一趟在 `run_compaction_on_with_the_region_check_of_the_intent_deletion` 开头停下：「CompactionRunOutcome::Stopped(CompactionRunStop::BatchSizeNotDecidedByTheClauses),」（`crates/singlefs-core/src/mounted_session.rs:3760`）。可写挂载续做同样：「产品路径批大小条款没写，签任何东西、发任何写之前停下」（`crates/singlefs-core/src/mount.rs:8172`）。
- 而 K 的归属是空的：代码注释说它归实验，「产品路径的批大小条款没写（判决 G2 的 K 归 E10（老化：结构抗老化 vs 事后整理））」（`crates/singlefs-core/src/compaction_plan.rs:428`）；实验页说它不出这个数，「这次登记只出整理底额 `N₀`；产品批大小与三条整理水位不归这次登记」（`.claude/kb/experiments/10-老化结构抗老化vs事后整理.md:23`）。
- 后果：用户定的目标「最终该功能可以正确的接入并运行」在 K 定下来之前不可能达成；现有测试全部强制批大小，所以测试全绿而产品路径一批不发（见 F1）。K 还不是一个能随便给的默认值：A5 给出它的下界。
- 四句：不分辨臂（任何运行库形态都撞，是共用前提，另立一笔账）。系统看得到（开关就在分配器上）。满足的是用户目标字面，不是某条判据。改法：把 K 列为这一轮要定的第七格，或交用户；只改运行库碰不到它。
- 什么现象推翻它：仓里已有一处产品路径把 `batch_size` 设成别的值（`grep` `ForcedForTests` 只在测试里）。这一条我核了 `crates/singlefs-core/src/` 的命中，没有；`crates/` 全包没逐个核。

### A2　不发布的停不扣预算，也没有退避

正文第四条依赖「闲置时合计不过一份」。那一份数的是**发布**，不是**叫起**：

「compaction.publishes_since_the_last_user_change += 1;」（`crates/singlefs-core/src/mounted_session.rs:3890`）只在发成时走。

历史（放开的是池的形状，不写死）：

1. 只写过顺序写的池，段都是满的（正文 Q3 自己点名的「全是满的段」）。水位越线（比如全空段数低于线）。
2. 周期线程每 30 秒叫一次。预算判只看发布数，过了：「publishes_left_of_the_budget_given_by_the_last_user_change(」（`crates/singlefs-core/src/mounted_session.rs:2950`）。
3. 进 `advance_compaction_by_one_publish_computing_on`，先做这几样，都在任何判停之前：拷一整版「.clone();」（`crates/singlefs-core/src/mounted_session.rs:4418`）、读这一版「let read_of_the_version = version_as_the_compaction_reads_it(&version);」（`crates/singlefs-core/src/mounted_session.rs:4439`）、建索引「CentralMappingIdentityIndex::of_the_mapping_entries(&read_of_the_version.mapping_entries);」（`crates/singlefs-core/src/mounted_session.rs:4443`）。
4. 选段；GPU 模式这里交一次提交「live_slots_per_cluster_segment_computing_on(」（`crates/singlefs-core/src/mounted_session.rs:4535`），要扫的段每段再交 GPU「let scanned = scan_of_the_batch(region, &*devices, compute)」（`crates/singlefs-core/src/mounted_session.rs:4579`）。
5. 停在「CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove {」（`crates/singlefs-core/src/mounted_session.rs:4630`）。零次发布，预算没动。
6. 30 秒后从第 2 步再来，读的还是同一版，判的还是同一个结果。盘上没变，判据不可能翻面。

同样不扣预算的停还有 `NoYieldFromTheLastIntent`（第 3 步的读照做之后才判，`crates/singlefs-core/src/mounted_session.rs:4475` 在 `:4439` 之后）、`NoEligibleSegment`、`PausedShortOfSpace`、`IssueRefusedShortOfSpace`。其中无产出那一支要等用户改动才解：「compaction.last_completed_intent = None;」（`crates/singlefs-core/src/mounted_session.rs:2626`）。

- 判定：正文要判的「不需要清理就不跑」在**写**上成立，在**读与算**上不成立。这几步全在那把锁里，见 C1。代价随池线性涨（`allocation_records` 是整张表）：`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历` 把后台放在「无戒律」，所以不违反那一格；违反的是用户要的「不需要清理就不一直跑」的另一半。
- 量：探针 `name=wake_cost` 打每次叫起的挂钟与 `budget_used`，扫 64 到 4096 个数据单元。七·6 的副本实验「P = 4096 一批 217.47 → 62.38 ms」是引的旧数，当线索（推的，线性外推到 2²⁰ 约 16 秒，**没量过**）。
- 四句：分辨臂（「水位越线就跑」这个形态打中，加退避或「盘上没变就不重判」的臂不中）。系统看得到判别子：上一次停因与现行 txg 都在内存里，txg 没变、停因是这几种就不用重判，这一点今天没人判。满足判据「不需要清理就不跑」的字面否定分句。改法：见文末「改法」第 1 条。
- 什么现象推翻它：探针 `name=wake_cost` 显示 `budget_used` 随叫起涨，或挂钟不随 `allocation_records` 涨。

### A3　用户改动那一支挡住定时兜底

```text
match compaction.start_judgement { DueAfterAUserChange => 至多一批, NotDue => 才看定时兜底 }
```

这是我概括的结构，原文：「CompactionStartJudgement::DueAfterAUserChange => {」（`crates/singlefs-core/src/mounted_session.rs:2914`）后面接「CompactionRunReach::AtMostOneBatchAfterAUserChange,」（`crates/singlefs-core/src/mounted_session.rs:2918`），「CompactionStartJudgement::NotDue => {」（`crates/singlefs-core/src/mounted_session.rs:2921`）里才查定时兜底。到一批就停：「if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {」（`crates/singlefs-core/src/mounted_session.rs:3825`）。

历史：水位越线，用户每 30 秒内至少发成一次改动（任何改动，T_time 5 秒一次的攒批发布就算）；线程 30 秒一醒。每次醒来 `start_judgement` 都是「该跑了」，于是每次醒来至多一批，「推到停或预算」那一支永远进不去。一批 K 个单元，段 64 槽、数据单元两槽：32 个数据单元的段要 ⌈32/K⌉ 批，也就是 ⌈32/K⌉ × 30 秒腾一段。

- 放开的动作：探针扫「每隔 n 次叫起一次用户改动」n = 0、1、2、3。推的读数：n = 1 时 `timer_runs=0`，n = 0 时一趟推完。n = 2、3 时两支交替，定时那一支才跑得动。
- 醒得更勤能缓解、去不掉：线程每秒醒一次（只判、不跑）时定时那一支 30 秒轮到一次，上界变成 A4 那个数；再多就只能动判据。
- 这一支 G8 丁当初是**在用户改动的调用线程里**限前台代价的（「丁（`publish_user_change` 成功之后水位越线再跑至多一批）」，`research/prompts/m4-gc-r1-main-verification.md:39`）。挪出调用线程之后它限的是后台自己，理由不在了，上限还在。同一行还写了「丙（线程，归里程碑六）」：线程当时不在设计里。
- 四句：分辨臂（周期线程的节奏打中，改判据的臂不中）。系统看得到（水位读数 O(1)）。满足 Q3「预算扣完、水位仍越线时的行为」那一问下的不收敛。改法第 2 条。

### A4　预算本身的量级

- `S = 8`：「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」（`crates/singlefs-format/src/lib.rs:308`）。预算「.saturating_sub(publishes_since_the_last_user_change)」（`crates/singlefs-core/src/compaction_plan.rs:1399`）从 `3S − 4 = 20` 扣。
- 一条意图至少两次发布（签发、搬一批兼删意图），所以一次用户改动至多 10 条意图。一段 64 × 16384 字节 = 1 MiB（`SLOT_BYTES` 16384，推的算术；kb 写「至多 2 MiB」，差的是不是两盘我没核）。**一次用户改动至多腾 10 段、约 10 MiB。**
- 对面：`T_dirty` 默认环下有效值「默认环下有效值 = min(2 GiB, 256 MiB) = **256 MiB**」（`.claude/kb/decisions/16-发布语义.md:165`，一次发布的 journal 字节，「journal 恒占用户数据 12.5%」同一节）。持续写时写入速度与腾段速度差两个数量级以上（推的，**没量过，本机写带宽 D16 已定项 5 自己写着没实测**）。
- 闲置池：挂载那一份用完就停在 `PublishBudgetOfTheLastUserChangeSpent`，水位仍越线，直到下一次用户改动。1 TiB 的盘上腾 10 段对水位没有意义。正文 Q6 第一条验收「水位越线的池子上 GC 自己跑、腾出空段」只有在线很贴近时才过得了，过了也不能说明收敛。
- 这不是「预算太小」能一句话改的：`3S − 4` 是回退候选集的那根线（B1）。收敛要的预算与回退要的预算方向相反，这是一条要交用户的岔路，不是实现细节。
- 四句：不分辨「一把锁」与别的线程形态（预算是共用前提）。满足「不收敛」。改法第 3 条只能选一边。

### A5　底额、预算、无产出三样连成死锁

合取 7 只拦前台写：「合取 7（D3（空间分配） 已定项 7 登记表第 7 行：`全空聚簇段数 ≥ N₀`，只拦前台写入类发布）」（`crates/singlefs-core/src/transaction.rs:12883`），被拒原样交回、不推抬 F（`crates/singlefs-core/src/mounted_session.rs:2000`）。没发成的改动不归零预算、不清无产出：`publish_user_change` 里 `published?`（`crates/singlefs-core/src/mounted_session.rs:2596`）在归零（`:2621`）之前。

历史一（预算）：K 小到一条意图要超过 20 次发布（现有用例就是这样造的：「一条意图要发 1 次签发加 29 批，比一份 `B_gc` 多。」（`crates/singlefs-verification/tests/compaction_off_the_calling_thread_and_admission.rs:61`））。全空段到 `N₀` 之下；前台写被拒；整理发满 20 次，意图没做完，段没腾出来；预算扣完；前台还被拒；没有用户改动，预算回不来。**前台与整理互相等。**出路只有不受合取 7 拦的改动，或重挂（重挂给新预算，B1）。

历史二（无产出）：一条意图的元数据发布开新段比它腾出的多（`compaction_rollback_in_the_session_clears_the_last_completed_intent.rs` 文件头造的就是「搬空只抵掉那个新段」），全空段落到 `N₀` 之下，`last_completed_intent` 留着；前台写被拒，清不掉它；整理每次停在「无产出」。同样互相等。

- 由用户决定的动作放开：探针 `name=floor_and_budget` 扫 K = 1、2、3、4、8、16、32，卡住之后 `name=floor_and_budget_way_out` 放开「建 inode」「建快照」。`N₀` 用只供测试的开关压在此刻全空段数加一，代替真把盘写满。**这是造的前提**：真盘写到全空段为 0 这段历史探针没走。
- 由此给 K 一个下界（推的）：S 取最小 4 时预算 8，32 个数据单元的段要 K ≥ 5；64 个一槽单元的段要 K ≥ 10。S = 8 时 K ≥ 2 与 K ≥ 4。批里遇到本来就重写的节点时计划要跳，下界只紧不松。
- 四句：不分辨「一把锁」（共用前提），分辨 K 的取值。系统看得到判别子（拒绝成员就是 `EmptyClusterSegmentsBelowTheFloor`，注释写的是「调用方分得出「先整理」与「真没空间」」，而今天的调用方没有「先整理」这个动作）。满足 Q3「选不出能腾出空段的区域」之外的新形状。改法第 4 条。
- 什么现象推翻它：K 取到下界之上时探针的 `first_foreground_accepted_at_wake` 全有值；无产出那一支探针没造，推的。

### A6　水位只在起跑时查一次

判起跑只在一趟开头（`crates/singlefs-core/src/mounted_session.rs:2937`）与用户改动发成时（`crates/singlefs-core/src/mounted_session.rs:2627`）。循环体 `crates/singlefs-core/src/mounted_session.rs:3815` 起再也不看水位，只看预算、一批上限、节奏与意图判据。

- 后果：越线一点，定时那一趟照样推到预算：搬 10 段。没有「回到线上就停」，所以正文问的回滞没有落点，「停止线」字段也没有消费者（D1）。D26 已定项 1 第 2 条定了停机是 `R` 本地谓词，「不靠水位」是定案，不能随手加：要加就是改定案，交用户。
- F 那一条有正反馈：「预算最高那条整理臂的 `runs` 轨迹单调上行」（`.claude/kb/decisions/26-后台整理与放置回收.md:38`）。F 越线时整理把 runs 抬得更高，F 不会回线下，每次用户改动都起跑、都发满。那是 E93 的模型读数，不是产品路径的量；**当线索，没量过**。
- 判「不需要清理就不跑」三个量：全空段数在写上成立（腾够了就不越线）、但不在 A2 的读上成立；F 不成立（可能永远越线）；不可回收字节恒 0，永不越线（D1）。

## Q5　交错之后崩溃语义变没变（主攻）

### B1　回退候选集被周期整理挤掉

条款：「**定案**：回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算」（`.claude/kb/decisions/16-发布语义.md:31`）。`B_gc` 立的时候防的正是这件事：「GC 发布不改变用户可见状态，连发会把回退候选集挤成 1 个状态」（`research/prompts/m4-gc-r1-main-verification.md:39`）。

根环 3S = 24 槽。`3S − 4` 这根线保证的是：**紧接在 4 次连续用户改动之后**整理发满，环里还留得下那 4 个。它不保证周期整理之下的任何一刻。

历史甲（有写入，用户动作放开）：

1. `U₁`（用户改动），归零：「compaction.publishes_since_the_last_user_change = 0;」（`crates/singlefs-core/src/mounted_session.rs:2621`）。
2. 下一次醒：「该跑了」，至多一批（2 次发布）。再下一次醒：定时那一支，把这一份预算发满，合计 G²⁰。今天只有调用方才走得到这一步，有了周期线程它就是常态。
3. `U₂`，又归零，又给一份 20。
4. 醒两次，G¹⁰（探针的 R 只够 30 次发布；真盘有的是段）。
5. 环里 24 条：13 条 `U₁` 那个状态的整理根、`U₂`、10 条 `U₂` 那个状态的整理根。**只剩 2 个用户状态**，`U₁` 之前的全没了。

历史乙（一次用户改动都没有，只有崩溃）：

1. 六次用户改动，之后不再写。崩。
2. 挂载。预算是新的：「publishes_since_the_last_user_change: 0,」（`crates/singlefs-core/src/compaction_plan.rs:877`），经「let mut compaction_state_of_this_mount = CompactionOfThisMount::of_a_new_mount();」（`crates/singlefs-core/src/mount.rs:8183`）。
3. 头一次叫起定时兜底立刻到期：「None => true,」（`crates/singlefs-core/src/compaction_plan.rs:1054`）。水位越线就发满 20 次。
4. 崩（比如是某个 bug 让进程隔一会儿就死，**这正是用户最想回退的时候**）。回到第 2 步。
5. 第二轮发满之后环里 24 条全是最后那个用户状态：**只剩 1 个**，就是 O11 说的那个结果。这时要回退，候选集里没有。

- 今天为什么碰不到：产品路径没有调用方。正文说的「方案按今天的实现来谈」在这一格给的是错的安全感：有了周期线程，这两段历史都成了默认行为。
- 加重项：整理搬了 inode 根或 extent 根的那一批**算非空**，「只搬别的单元的批、签发与删意图那几次根指针照抄，不算非空。」（`crates/singlefs-verification/tests/compaction_batch_relocating_a_user_visible_root_is_non_empty.rs:5`）。于是「第 4 新的非空根」可以是整理根，抬 F 的上限因此跨过真正的用户状态（推的，探针没造抬 F）。
- 边界条款的读法：D16 已定项 1 写了「根环容量是这句的边界」，但那一句举的是「两个状态来回交替超过根环一圈」。周期整理不是用户在来回交替。这是不是在射程外要主 agent 判，不是我能定的；打中报成「字面在边界句之外，O11 的立意之内」。
- 放开的动作：探针 `name=rollback_states` 扫「一次用户改动之后醒 0、1、2、3 次」，0 次是对照臂，报每一步之后的最小值；`name=crash_loop_step` 扫「每次挂上之后醒 0、1、2 次」。**写死的只有**前缀（40 个数据单元的顺序写）、一批一个、强制选段。一批一个是为了一条意图够发满预算；K 大时要换成多段，代价相同（推的）。
- 反过来我接受什么：对照臂（醒 0 次）与其余臂的最小值相同，这一条就撤回。
- 四句：不分辨「一把锁」（线程形态都中），分辨「定时那一支与挂载给不给新预算」。系统看得到判别子：环里还剩几个不同状态，判抬 F 上限时本来就在数。满足的是 D16 已定项 1 定案第一句的否定；边界句是否豁免待判。正文④「定时兜底的预算照旧与用户改动共用」在这一格**不起作用**：共用只限一次用户改动内的总量，挡不住多次用户改动各发一份，也挡不住重挂。改法第 5 条。

### B2　崩溃枚举罩不到的次序

现查（`grep` 过 `crates/singlefs-verification/tests/` 的 `crash_enumeration_*.rs` 与 `common_compaction_prefix`、`common_compaction_relocation_on_the_background_compute`）：`run_compaction_in_the_background` 在这些文件里 0 处，整理流都经显式入口「let run = session.run_compaction(devices).expect("整理一趟");」（`crates/singlefs-verification/tests/common_compaction_prefix/mod.rs:357`），每趟把已过发布数压成一次预算。攒批会话进后台入口只在非崩溃用例里（`batching_across_requests_and_barriers.rs`）。

已有的交错：`crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs` 有「会话里 unlink」夹在批中间；`LifecycleClosure` 的用户改动在意图删了之后。

| 新次序 | 今天有没有流 | 为什么可能不同（都是推的，没造出坏） |
|---|---|---|
| 排空待发 + 同一次叫起接着签发、搬批（`crates/singlefs-core/src/mounted_session.rs:6053`、`crates/singlefs-core/src/mounted_session.rs:6079`） | 没有 | 排空那次发布就是用户改动，归零预算、记「该跑了」，紧接着整理；崩在两者之间的状态集没枚举过 |
| 活意图期间的**写类**用户改动（批与批之间的覆盖写、攒批发布，含改到 R 里单元） | 没有（只有 unlink 那一种） | 覆盖写放掉 R 里的落点，下一批重扫 |
| 活意图期间的建快照、删快照 | 没核到 | 钉住 R 里的单元 |
| 卸载带活意图、再可写挂载续做 | 没有（同时出现 `run_compaction` 与 `unmount(` 的崩溃流 `grep` 到两条，没逐条读，名字上不相干） | 抬 F 那一串与活意图共存；卸载之后 F = 现行，再挂续做 |
| 挂载续做之后头一次叫起立刻到期 | 没有 | 续做那一条做完马上签新意图 |
| 定时那一趟发满预算（多条意图连发），连带发 discard | 没有（流里一趟一次发布） | 第二条意图的签发读「上一条完成」 |

判法：这是**覆盖缺口**，不是已证的坏；我没造出恢复后判红的状态，因为崩溃枚举原型这一轮没写（见「这条腿自己的限度」）。

**一把锁把交错串成一种次序，这句话对；「这个次序集合已被罩到」不对。**串行之后的集合是 {前台入口, 后台一趟} 的任意序列，今天罩到的只是几条写死的序列。正文问「要不要加新的崩溃点重放流」：要，至少上表头四行；按 `.claude/agents/implementation-writer.md`「做什么」第 7 步写流时每条写清多罩什么。

**一把锁下不变的**（核过，算没打中）：整理批插不进抬 F 与再判之间，`publish_user_change` 从头到尾持 `&mut self`；待发不并进整理（后台入口先排空）。

### B3　后台批写失败冻结前台

冻结机制：「每一次发布都要交分配器进来，冻结着的时候发布路径在任何读写之前拒绝」（`crates/singlefs-core/src/transaction.rs:1529`），「只有 [`resend_the_frozen_publish`] 清得掉它。」（`crates/singlefs-core/src/transaction.rs:1530`）。

历史：

1. 周期线程的一批写盘报一次瞬时错；发布冻结在分配器上。那一趟交回「CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),」（`crates/singlefs-core/src/mounted_session.rs:3946`）。
2. 这不是报警通道里的东西：报警只管「这一趟结束时整理因一批出错停着」（`crates/singlefs-core/src/mounted_session.rs:358`），只有「Err(CompactionBatchNotCompleted::ComputeFailed(error)) => {」（`crates/singlefs-core/src/mounted_session.rs:4240`）才进停下那一支，写盘失败不走它。运行库不看 `outcome` 就没人知道。
3. 前台线程下一次写：「AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {」（`crates/singlefs-core/src/mounted_session.rs:5727`）。前台写的报错来自它没做过的一次发布。
4. 叫起照跑：每 30 秒签发一次、被拒一次，「allocator.compaction_mut().issue(issued);」（`crates/singlefs-core/src/mounted_session.rs:4645`）、「allocator.compaction_mut().withdraw_the_live_intent();」（`crates/singlefs-core/src/mounted_session.rs:4664`），「PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }」不算空间不够（`crates/singlefs-core/src/mounted_session.rs:2017`），没有退避。
5. 重发在哪：会话没有入口（`grep -n resend crates/singlefs-core/src/mounted_session.rs` 只有文档）。调用方要交出会话、走 `transaction::resend_the_frozen_publish`、自己把交回的版本装进 `current`。
6. 重发成功之后整理账没动：冻结那一批没走到发布计数（A2 那一行），若它是完成意图的那一批，「compaction.last_completed_intent = Some(intent);」（`crates/singlefs-core/src/mounted_session.rs:4769`）也没走。之后多发一次、少判一次无产出（推的，量级小）。

- D16 已定项 17 的射程是「fsync 报错之后调用方能假设什么」。这里 fsync 甚至可能返回成功（待发空就什么都不发，「if self.pending.is_empty() {」，`crates/singlefs-core/src/mounted_session.rs:5684`），下一次写才报错。语义没错，**但没有条款说前台要不要把后台的写失败算自己的**。交主 agent。
- 四句：分辨臂（任何把写放到后台线程的形态都中，单线程泵不中）。系统看得到（`frozen_publish()`）。满足 Q1「线程 panic、前台持锁太久…各怎么办」之外的漏项。改法第 6 条。
- 查过、没打中的一半：`publish_the_pending_changes` 看到「冻结着」就不放回待发（`crates/singlefs-core/src/mounted_session.rs:5697`），它假定冻结是这一次造成的。后台批冻结时那一刻待发一定是空的吗？是：后台入口先排空才跑、持锁期间 `accept` 进不来、冻结之后 `accept` 拒。**这条保证靠运行库只走攒批会话的入口**：内层字段私有，类型上调不到（「待发与会话都是私有字段」，`crates/singlefs-core/src/mounted_session.rs:5196`），这一点是好的。

### 同一个线程还是两个

`publish_when_due` 的周期就是丢失窗口里的 P：「**由此崩溃最多丢的是 T_time + P + d + 排队时长 + N_switch × (T_retry + T_redo)，不是恰好 5 秒**」（`.claude/kb/decisions/16-发布语义.md:167`）。挂在 30 秒的周期上，默认丢失窗口从约 5 秒变成约 35 秒。**不能同一个周期**；同一个线程可以，但要按 min(T_time 判定周期, 水位周期) 醒，整理判据自己按 `timer_fallback` 滤（`check_the_timer_fallback` 已经做了这一层）。

查过、没打中：整理跑着时待发会不会超龄：不会，整理前已排空，持锁时 `accept` 进不来，所以整理不延长丢失窗口，它把代价转成了前台写的延迟（C1、C2）。

## Q1　一把锁够不够

### C1　锁内等 GPU（分辨臂）

条款撞条款。D26 已定项 13 第 5 条：「前台与后台经一把锁排队」（`.claude/kb/decisions/26-后台整理与放置回收.md:249`）。D24 已定项 2 第 2 条：「**不许进前台路径。** PCIe 往返是几十微秒量级，前台读写延迟吃不起这个；它只能服务已经攒批的后台重活。」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:35`）。

后台入口收 `&mut self`，GPU 那几段在 `advance_compaction_by_one_publish_computing_on` 里面。没有拆分，整趟都在锁内。

历史（卡的负载由用户决定，放开）：

1. `background_compute=gpu`，卡同时跑别的活（训练、渲染）。
2. 周期线程进锁。一批：数活槽交一次（`crates/singlefs-core/src/mounted_session.rs:4535`）；扫段每段核校验和与解单元头各交一次；每段没到排在最前的就再扫一段（`crates/singlefs-core/src/mounted_session.rs:4578`）。
3. 每次提交没超时（默认上限「未定：代码现取 10 秒」，`.claude/kb/mount-parameters.md:29`，只要低于它就不算错），挨着慢。
4. 前台 `publish_pending_now`（fsync）举旗。旗只在**每一批之前**看：「match caller.foreground_fsync() {」（`crates/singlefs-core/src/mounted_session.rs:3841`）。这一批里面谁也不看。前台等锁等的是 GPU。
5. 卡真挂住时：一段的超时跨片累计，「这一批在这张卡上第 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次超时就交回」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:8`），默认 4 × 10 秒 = 40 秒（推的算术），全程持锁。

- 第 6 条「前台有 fsync 在等时不发下一批」在一把锁下**答得出、由谁答**：调用方用一个不进锁的原子量答（探针 `name=one_lock` 就这样写）。答得出不等于有用：它的粒度是一批，而一批里有 GPU。
- 四句：**分辨臂**（「算在锁外、发在锁内」的臂不中）。系统看得到（提交是在算还是在等）。满足 D24 已定项 2 第 2 条字面；这条是「满足不了任何一条就不许做」（同文件第 38 行射程）。D26 已定项 13 第 5 条是用户 2026-11-08 定的，**对不上的是这两条之间**，要用户看。
- 按 `.claude/singlefs-ai-sop/rules/pushback-discipline.md@对不上就当场说` 三样：对不上的是上面两句；按提议做的现象是 GPU 模式下前台 fsync 延迟的尾部等于 GPU 一批的挂钟（七·5 要报的正是这个量）；第三条路：一把锁管会话与写，算在锁外，改法第 7 条。要动核心层入口，「核心层不起线程」不受影响。

### C2　只有 fsync 让得了路

正文列的前台接口里，`accept`、`publish_when_due`、切换、卸载都不是 fsync，还有正文没列的**读**：「pub fn read_at(」（`crates/singlefs-core/src/mounted_read.rs:959`）要盘表，盘表在锁里。

历史：定时那一趟 `UntilAStopOrTheBudget`，默认 20 次发布，一趟不出锁。前台 `write()` 进 `accept` 等完整一趟。探针 `name=one_lock foreground=not_an_fsync` 打前台拿到锁时整理发了几次布（推的：20），`foreground=fsync` 对照（推的：至多 1）。

- 要让所有前台都让路只有一个办法：调用方在 `foreground_fsync()` 里对任何等锁的前台都答 `Waiting`。一趟就停在 `YieldedToAForegroundFsync`，停因是假的，撞 `.claude/rules/fs-design.md@五条硬要求` 第 4 条「分支必须可观测」；这时饿死的换成后台。
- **反向饿死**：前台 fsync 不断（两个以上 fsync 线程，总有一个在等），`Waiting` 恒真，第一批之前就停，一批都不发。定时那一支连「该跑了」都不留、下一次要等一个间隔。全空段见底时撞 A5：前台被合取 7 拒，后台被前台让路。第 6 条没有「至少推一批」的下限，也没有按占用放开的一支。
- 探针没造反向饿死的并发版；核心层判据一读就是（`crates/singlefs-core/src/mounted_session.rs:3842`）。用例 `the_background_entry_sends_no_batch_while_a_foreground_fsync_is_waiting` 钉的就是这个行为，它是按设计的。

### C3　卸载、切换与那把锁

- 「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构。」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）。而卸载要拿会话：`mount::unmount` 收 `allocator`、`current`（`crates/singlefs-core/src/mount.rs:4599`）；会话在线程那一趟的 `&mut` 里，要等它从 GPU 等里出来。线程 join 就是等提交，至多 40 秒每段；核心层给 `BackgroundGpuCompute` 的调用没有取消。**一把锁加 join 与这一句字面相反。** 不 join、直接走：会话出不来，写不了卸载那一串，就是崩溃。驱动在 `poll` 里挂住（`background_gpu.rs` 第 24 行「用 wgpu 自己带限时的 `poll`」，限时靠驱动守）时 join 不返回；真挂住的驱动我核不动。
- 活意图撞卸载**没打中**：活意图留在盘上（提示类，D8 已定项 5），卸载走空发布、不碰它，再挂续做。缺的只是 B2 那条流。
- 攒批会话没有切换入口：`switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch` 在 `MountedSession` 上（`crates/singlefs-core/src/mounted_session.rs:2741`），包装只给只读的「pub fn session(&self) -> &MountedSession {」（`crates/singlefs-core/src/mounted_session.rs:5619`）。回退、推环、要出路同样。要调就得交出会话：「排空没发成：[`HandingBackTheSessionRefused`]，攒批的会话原样交回」（`crates/singlefs-core/src/mounted_session.rs:6101`），换回来时 `reading_of_the_last_publish: None`（`crates/singlefs-core/src/mounted_session.rs:5613`）。
- 由此的历史：GC 因 GPU 出错停着；占用不到 85%，不补跑；前台写到空间紧；攒批发布被拒（`AcceptedButTheTriggeredPublishRefused`，待发放回）。用户要切回 `cpu` 解除停下（第 4 条「当场解除」），运行库交出会话先排空，被拒，切不了。卸载同一条路，同样被拒。**停着的 GC 等切换，切换等排空，排空等空间，空间等 GC。** `accept` 过了判才进待发，落点取不到是唯一的缝（`crates/singlefs-core/src/mounted_session.rs:1997` 把 `PlacementRefused` 算空间不够）；这段历史要几步配齐，探针没造，推的。
- 改法第 8 条：包装上加切换入口，不排空。卸载被排空拒时的出路要有条款：今天只有「丢掉就是崩溃」。
- 线程 panic：会话是半改的（比如 `crates/singlefs-core/src/mounted_session.rs:4645` 装上了活意图、发布还没回）。`Mutex` 中毒之后用 `into_inner()` 接着卸载就是拿半改的分配器写盘。这一格定：中毒即停、不再写、当崩溃。不算打中，是接口要写死的一句。

### C4　写带宽上限按秒、叫起按间隔

「所以一个窗口里整理至多写上限加一批的字节。」（`crates/singlefs-core/src/compaction_plan.rs:1066`）。窗口一秒。到了上限这一趟停，定时那一支不把那个记号放回（`crates/singlefs-core/src/mounted_session.rs:3016`），下一趟要等一个间隔。配 L 字节/秒，定时那一支实际约 L/30，参数名说的是每秒。探针 `name=bandwidth_per_wake` 扫三档。线程要在停因是节奏时早醒，或这一支要留「该跑了」。

## Q2　三条水位

### D1　字段与量

三件事不是一回事：

1. **`0` 两种读法**：「| 可调值 | 整理低水位 / 高水位 / 停止线 | 8 × 3 | 0 = 内置默认 |」（`.claude/kb/layout/01-first-txn.md:153`）；代码「0 = 关，所以这一支在产品路径上不起跑」（`crates/singlefs-core/src/compaction_plan.rs:442`）。mkfs 恒写 0。照 kb 读，水位一实现，所有池立刻按内置默认起跑；照代码读，不改字段就永远不跑。
2. **不可回收字节恒 0**：「式子里的「不可回收」：只在 zoned 上非零」（`crates/singlefs-core/src/admission.rs:248`），「第一版不跑 zoned ⇒ 恒 0。」（`crates/singlefs-core/src/admission.rs:249`）。这条水位第一版永不越线，等于两条水位。阈值若取 0 又用 `≥`，就永远越线。
3. **三字段是一个量的回滞，三量是三个量**：低、高、停止线是一维的；D26 的三量各要各的线。「停止线」对上的是「**整理三个水位的名字与条款对不上**」（`.claude/kb/mount-parameters.md:35`）那一行里的「停机取 `R` 本地谓词、不靠水位」：没有消费者（A6）。硬对只能把低水位当全空段、高水位当 F、停止线当不可回收，名字全错。

正文问「阈值数值不进格式」（`.claude/kb/decisions/26-后台整理与放置回收.md:32`）与盘上字段是不是已矛盾：**没打中**。同一种写法 D16 已定项 5 有先例：「系统配置里那个字段是格式，字段里放什么数不是。」（`.claude/kb/decisions/16-发布语义.md:165`）。矛盾的是形状，不是「进不进格式」。

选项（数交用户，不替定）：

| 候选 | 代价 | 越线太早 / 太晚 |
|---|---|---|
| 字段拿掉，两条水位做不存盘的挂载参数，走 D26 已定项 13 第 1 条那一类 | 改格式（第一个外部用户之前是软的）；重启不记 | 见下 |
| 字段留着改名、只放全空段与 F 两个线 | 改名加一个废字段；每次挂载回写多写系统配置 | 同上 |

全空段线若取绝对段数：4 GiB 的盘与 1 TiB 的盘差 256 倍，小盘恒越线（空转，A2）或大盘太晚（前台撞 `N₀`）。取比例才随盘走（推的，拍的数）。线不能低到 `N₀` 附近：A5。F 的线我不给数：A6 那条轨迹没在产品路径上量过，线怎么取都可能永远越线。

### D2　「腾出空段」读哪个

「defer 窗口里的已释放槽不算占用：」（`crates/singlefs-core/src/allocator.rs:903`），「一段里只剩 defer 槽时这里算它全空，[`Self::segment_is_empty`] 与开段（[`Self::lowest_empty_segment`]）照旧算它占着。」（`crates/singlefs-core/src/allocator.rs:904`）。

历史：闲置池，整理搬空 R；记录口径 +1，水位回线上，整理停。R 里的槽在 defer 里：释放代要低于环里最旧有效根，环 24 条，整理一次用户改动只发 20 条，**自己转不完一圈**。接着写，开段开不到 R，走别处（推的；探针 `name=emptied_segment` 打记录口径、R 里空闲槽数、`lowest_empty_segment` 随后续用户改动怎么变）。

- 所以 Q6「腾出空段」若读 `empty_segments()` 就是自证；读「开得出」要写明在哪一步之后。抬 F 能放开，但抬 F 只在被拒时推。
- 这一条同样影响水位：记录口径的线回去了，真正能开的段没回去。

## Q4　开卡时序

- 次序只能是「读参数 → 建或取实例 → 枚举 → 开卡 → 挂载 → 起线程」：挂载入口要 `gpu_compute`，续做意图在挂载里就要算（`crates/singlefs-core/src/mount.rs:8191`）。代价是开卡成功了挂载才发现盘有问题：丢掉 GPU 时量代价的提交若还在跑，那张卡整张泄漏、记进进程那一份。反复挂一个坏盘会把卡的可用余量吃掉，之后的挂载「参与的卡一张都不剩」、后台任务进停着（D26 已定项 13 第 3 条），表现成 GPU 问题（推的，要真卡，核不动）。
- **E1 串行开卡**：「for discrete_gpu in discrete_gpus {」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:828`），每张至多五分钟，「没参与的卡也开」（`crates/singlefs-background-gpu/src/background_gpu.rs:836`）。循环里参与的卡一张没开成就当场拒（「return Err(BackgroundGpuOpenRefused::GpuUnusable {」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:856`）），所以不点名时第一张挂住 5 分钟就拒；点名排在后面、前面没点名的卡挂住时才累加，三张卡的第二台上至多约 15 分钟（推的算术）。「只点一张卡却要等别的卡开卡」是这一条的形状：点名的卡好好的，挂载被没点名的卡拖住。
- **打中 1 的形态**：枚举放线程加期限，现成的模子是开卡那条线程（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:282`）。超时之后线程留在驱动里，它拿着实例的克隆。「算不算把实例弄坏」**核不动**：要真挂在枚举里的驱动，今天量到的只有造 future 那一步同步（任务 10414 是别人的数，我没复跑）。可以推的是重试：每次挂载起一条，挂住几次留几条。要单飞：枚举在跑就不再起，这次挂载等它或按期限拒。
- **打中 6 的形态**：三个候选，没有一个没代价：
  | 候选 | 打中它的历史 |
  |---|---|
  | 进程一份、永不重建，报错写「重启进程」 | 长进程里第一次选 `gpu` 时驱动没装好：实例照建、枚举恒空（「此刻驱动不在也交回实例、不报错，这个实例枚举一直是空的」，`crates/singlefs-background-gpu/src/background_gpu.rs:762`），修好也认不出，报的还是「卡不在」 |
  | 枚举不到独显时重建，限 N 次 | N 取几：「7 「8 小于 12」只对一套驱动成立」（`research/prompts/m4-gpugc-one-vulkan-instance-code-r1-main-verification.md:42`）。12 这个界在第二台那一套驱动上量的、丢掉的实例也算；没有 GPU 的机器每次挂载都白耗一次 |
  | 一次挂载一个进程（里程碑十二的守护进程这样起） | 打中 6 不存在了，但运行库「跨多次挂载持有实例」这一条前提也没了。今天 `crates/` 没有入口，这是要用户定的岔路 |
- `GpuMemoryLeakedInThisProcess`：运行库持有，跨挂载。陷阱：短入口「&GpuMemoryLeakedInThisProcess::with_nothing_leaked(),」（`crates/singlefs-background-gpu/src/background_gpu.rs:824`）每次给空账，运行库走它就把泄漏账丢了。接口上只暴露带账的那一个。
- **报警**：叫起一次报一次（第 4 条）。「同一个线程按 min 醒」若每醒一次都调后台入口，`wakes_while_stopped` 跟着涨，报警按判定周期发。两种办法：只按水位周期调后台入口（用户改动之后那一批就要等到 30 秒，A3 更重），或判定周期里自己先判「离上次叫起够不够一个间隔」（核心层只对定时那一支滤，「该跑了」那一支每次都起）。可写挂载续做的报警恒 `None`（`crates/singlefs-core/src/mounted_session.rs:361`），`MountOutput` 里那一趟的 `outcome` 没人看就丢。报警要送到运行库暴露的接收方；`#[must_use]` 只管编译，`let _ =` 就过，要一条变异行。

## Q6　怎么验才算数

### F1　今天的用例验不到接入

- 录制盘过不了线程：「pub struct SharedStream(Rc<RefCell<StreamState>>);」（`crates/singlefs-verification/src/lib.rs:252`）。崩溃枚举与块序列比对都靠它。真线程上录不了流。探针另核 `MountedSession`、攒批会话、两种产品盘过不过 `Send`（`name=send_bounds`，编不过就是答案）。
- 所以运行库要有一个**不起线程、由调用方一拍一拍推**的入口（钟、前台 fsync、要不要起跑都可摆），周期线程只是它外面的一个循环。崩溃枚举、块序列比对、必红世界走拍子；真线程只做少数冒烟（真盘、真卡、真钟）。线程循环那几行没有崩溃点要枚举，但要有用例钉它：停得下、panic 之后前台拿到错误、卸载等得到。
- 块序列逐字节相同只在拍子下有意义：真线程下 GC 与前台的次序取决于调度，两次跑本来就不同。七·4 的比对继续走拍子，CPU 与 GPU 用同一串拍。真卡一臂放真卡测试（特性 `tests-on-a-real-gpu`）。验证包不依赖产品 GPU 包这条不动，今天走测试替身；要真卡比块序列得放在产品 GPU 包的测试或 `src/bin` 装置里。
- 现有用例全强制开关，产品值那条路没有一条用例：`CompactionSwitches::PRODUCT` 不改、水位真越线、拍子推，GC 发布。**A1 就靠它变红**。
- E202 已经有一份手写的线程与调用方：「struct CallerOfTheConcurrentRun<'run> {」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/command_opening_and_construction_lines.rs:108`）。它量的前台 fsync 延迟走不走那把锁我没读完。要改接运行库：七·5 要报的「前台 fsync 延迟」若不含等锁，C1 那个尾部量不到，对照组的结论就是错的。

### 验收清单（缺一条算没接上）

| # | 验收 | 必红世界 |
|---|---|---|
| 1 | 产品开关、水位越线、挂载之后不调任何后台入口：拍子推，GC 发布 | 批大小留 `NotDecidedByTheClauses` |
| 2 | 同上，真线程真钟：等到全空段（**写明读哪个，D2**）涨，卸载干净 | 线程不起；线程起了不调后台入口 |
| 3 | 满段池上叫起 N 次：盘上逐字节不变，叫起代价不随 N 涨（A2） | 去掉退避 |
| 4 | 持续用户改动下定时那一支轮得到（A3） | 把判据改回去 |
| 5 | 持续周期整理、崩溃循环下回退候选集个数钉绝对值（B1，数交用户定） | 重挂预算归零 |
| 6 | 全空段见底、K 取下界时前台不永久被拒（A5） | K 取 1 |
| 7 | 后台批写失败：前台看得到、重发之后接着跑（B3） | 不重发 |
| 8 | 前台等锁上限钉绝对值，GPU 模式下慢卡假实现（C1） | 算放回锁内 |
| 9 | 卸载：带活意图、线程在 GPU 等里、待发被拒，各一条 | join 无限等 |
| 10 | B2 表头四行各一条崩溃枚举流，每个状态恢复、checker、再挂续做 | 现有那几个整理必红世界接上 |
| 11 | CPU 与测试替身 GPU 走同一串拍，块序列逐字节相同；真卡一臂在第二台 | GPU 计划改一个源落点 |
| 12 | 假钟：钟与前台 fsync 走已有的 `CallerOfTheBackgroundCompaction`，运行库另要一个「下一拍什么时候」的钟，真线程的 `sleep` 也经它 | 真 `sleep` 进了拍子入口 |
| 13 | 两台：拍子用例不碰卡；真卡用例按卡清单跑，没有卡的台照 D26 已定项 13 拒挂载那一支 | 写死主机 |

## 改法（只在读码上推的，没实现、没跑；被攻过零轮）

每一格都是「推的，待主 agent 跑」，没有一格量过。

| 改法 | A2 | A3 | A4 | A5 | B1 | B3 | C1 | C2 | C3 | 代价与它自己会撞什么 |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 记下（现行 txg、停因），txg 没变、停因是不发布的那几种就不重判 | 修 | | | | | | 减 | | | 无产出、全满这些判据会因别的量翻面吗要逐个核；隔离集、不合格段表也在变 |
| 2 后台入口里「该跑了」那一支不再限一批 | | 修 | | | 加重 | | | 加重 | | G8 丁的上限是挪出线程之前立的，要重判；发布更密 |
| 3 预算改按环里不同用户状态数现算（少于 4 个就不发） | | | **加重** | **加重** | 修 | | | | | 用户改动一慢整理就停：收敛更差。与 A4 方向相反，**这是岔路，交用户** |
| 3 的对面：放开预算、认回退候选集会缩 | | | 修 | 修 | **不修** | | | | | 改 D16 已定项 1 的射程，用户要知情 |
| 4 K 取下界之上；被合取 7 拒也算一次需求（给预算、清无产出） | | | | 修 | 加重 | | | | | 字面改第 6 条「被拒、没发成的改动不清」，要用户定 |
| 5 挂载给的预算按环里现状算，不从 0 起 | | | | 减 | 修历史乙 | | | | | 挂载要多读一遍根环（挂载本来就读） |
| 6 运行库持锁入口先判 `frozen_publish()`、重发；写失败进报警通道 | | | | | | 修 | | | | 整理账补计数要动核心层 |
| 7 算在锁外：只读版本与盘读锁外做、算完进锁核 txg，没变才发 | | | | | | | 修 | 减 | 修一半 | 要动核心层入口与盘的共享；版本变了算的就白算；用户定的「一把锁」要改说法 |
| 8 包装加切换入口；前台等锁也算让路、停因分开报 | | | | | | | | 修，换成反向饿死 | 修 | 第 6 条要加下限，又是一条用户的话 |

## 没打中的形状

每条写试过什么、范围多大；**全部是读码，没有一条跑过**，按规则「没打中」一次不算数。

- **整理批插进抬 F 与再判之间**：`publish_user_change` 全程 `&mut self`，线程或锁下都插不进。范围：`crates/singlefs-core/src/mounted_session.rs:2515` 起那一个入口。
- **待发并进整理发布**：后台入口先排空，排空被拒不跑。范围：攒批会话的后台入口。
- **整理延长丢失窗口**：见 Q5 末尾，不延长。
- **待发被冻结吞掉**：B3 末尾，今天类型上走不到。
- **「阈值不进格式」与盘上字段矛盾**：见 D1，有 `T_dirty` 先例，不算矛盾。
- **带设备维的「任一越线」**：两盘同槽（「两盘同槽，选段与判类按第一块盘的记录做」，`crates/singlefs-core/src/mounted_session.rs:4432`），记录一样，今天没造出两盘读数不同而整理判错的历史。隔离位不进记录口径，没逐条核。
- **水位读数的代价随盘涨**：`empty_segments`、`free_runs` 都是 O(1) 增量，起跑判据这一步不涨；涨的是起跑之后（A2）。
- **`B_gc` 被定时兜底绕过**：同一次用户改动内绕不过（用例钉着）；绕得过的是多次用户改动与重挂（B1）。
- **钟倒退、钟不走**：`saturating_sub` 按没过间隔算。卡住的钟让定时那一支永不到期，但这是调用方的钟；运行库用单调钟。
- **水位在线附近来回**：A6 之外没有独立的新形状：每次用户改动本来就重判，水位来回不比水位恒越线多花什么。回滞要不要，取决于 A6 那条定案改不改。
- **`Send`**：读到 `BackgroundGpuComputeHandle` 带 `Mutex<dyn … + Send>`，会话字段没看到 `Rc`。盘不确定，等 `name=send_bounds`。
- **活意图撞卸载**：见 C3。

## 这条腿自己的限度

- **一行代码都没跑**。探针是盲写的，没编过：API 从现有用例读来，可能编不过，编不过要新派同立场的腿修。主 agent 复跑之前，报告里每个 `name=` 的数都是推的。
- **Q5 没写崩溃枚举原型**：B2 只是覆盖缺口，**没有造出一个恢复后判红的状态**。这是这一轮最大的缺口：正文要求主攻 Q5，我拿到的 Q5 打中是语义（B1）与工程（B3），不是崩溃枚举红。接手的腿要写的话，第 3b 步的量都还没用（0 个状态），优先写 B2 表第二行（活意图期间的写类改动）。
- 探针的水位越线全靠 `ForcedToStartForTests`：核心层今天只能这样。真水位函数下的历史没走。
- A5 的底额靠只供测试的开关压，不是真把盘写满。A2 的「满段」靠只写顺序写的小池，最大 4096 个数据单元，不是真满盘。
- 没读 `e202_gpu_cpu_gc_comparison.rs` 的并发那一半。`mounted_read.rs` 只看了入口。
- 真挂住的驱动、真拔卡一条都核不动。任务 10414、10428、9625、9626、9650 的数是正文转述，我没复跑，当「推的」。
- 只抽了一次样，「没打中」不算数。
- 引的外部实现：无。

## 模型

只有两份探针与一份复跑脚本。没有 `out/` 里的基准输出：这条腿没跑过。

```
6570eaa04a3937a36386b48d014e0f8b4514fe6bd513ea91f275b38c011d8969  research/prompts/m4-gpugc-background-runtime-design-r1-opus-model/src/runtime_design_attack_probes.rs
380c80976c16d52ea911af482cbe7778cf7481dafae26e0f7b6cdc90b7cba02f  research/prompts/m4-gpugc-background-runtime-design-r1-opus-model/src/runtime_design_send_bounds_probe.rs
62acf47cdb12e9254733d104fb8d758132d651340abcb7081f68bc66db354efe  research/prompts/m4-gpugc-background-runtime-design-r1-opus-model/tools/rerun.sh
```

探针逐条打的 `name=`：

| `name=` | 判哪一条 | 放开的是什么 | 写死的是什么 | 翻面（跑之前写下） |
|---|---|---|---|---|
| `due_shadow` | A3 | 每隔 n 次叫起一次用户改动，n = 0、1、2、3 | 一批一个、强制选段、12 次叫起 | n = 1 时 `timer_runs > 0` ⇒ A3 撤回 |
| `rollback_states` | B1 甲 | 一次用户改动之后醒 0、1、2、3 次 | 前四次连续用户改动、之后四轮 | 醒 2、3 次的 `smallest_states_reachable` 不小于醒 0 次 ⇒ B1 甲撤回 |
| `crash_loop_step` | B1 乙 | 每次挂上之后醒 0、1、2 次；崩三次 | 六次用户改动之后不再写 | 醒 1、2 次的 `states_reachable` 不小于醒 0 次 ⇒ B1 乙撤回 |
| `floor_and_budget`、`..._way_out` | A5 | K = 1、2、3、4、8、16、32；卡住之后建 inode 或建快照 | `N₀` 压在全空段数加一 | K = 1 时 `first_foreground_accepted_at_wake` 有值 ⇒ A5 撤回 |
| `wake_cost` | A2 | 64、256、1024、4096 个数据单元（下一档按建池时长乘 4 估，超过 20 分钟不跑，打 `wake_cost_skipped`） | 一批 32、不强制选段、8 次叫起 | 不发布的那几次叫起挂钟不随 `allocation_records` 涨 ⇒ A2 的「随池涨」撤回，「不退避」不撤 |
| `one_lock` | C1、C2 | 前台是不是 fsync | 真线程、`Mutex`、一批一个 | `not_an_fsync` 那一臂整理发布数小于 20 ⇒ C2 撤回 |
| `bandwidth_per_wake` | C4 | 上限 64 KiB/s、1 MiB/s、16 MiB/s | 钟走 30 秒一次 | 一次叫起发布数不受上限影响 ⇒ C4 撤回 |
| `emptied_segment` | D2 | 搬空之后接着 0 到 6 次用户改动 | 一批 32、强制选段 | 搬空当下 `free_slots_of_the_region` = 64 ⇒ D2 撤回 |
| `send_bounds` | Q1、F1 | 无 | 五个类型 | 编不过说明哪个类型过不了线程 |

估时（推的）：起点只建一次、每格从内存拷。`wake_cost` 4096 那一档建池可能几分钟，有自限；其余每格几秒到几十秒。全量估不到 40 分钟，**没有实测，主 agent 跑到超时之前先看 `name=wake_cost_pool` 那几行**。没有用崩溃枚举函数，0 个崩溃状态。

## 要主 agent 跑的命令

没有一条跑过。盲写，可能编不过：编不过或想看读数都新派同立场的腿（「承接：」本报告，带任务号）。

**整份复跑（两台都行，不碰卡）**

命令原样：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-background-runtime-design-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-background-runtime-design-r1-opus/copy --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-background-runtime-design-r1-opus/rerun-one
```

在哪起：仓根。rerun.sh 拷副本 `<草稿目录>/copy`（`.deliverables` 写成交接）进 `--work`、把两份探针放进 `crates/singlefs-verification/tests/`、各编一个测试目标，主工作区不动。

看什么：`TASK_SERVER_RESULT` 的 `type=`、`exit=`（0 是跑完）；`--work` 下 `send-bounds.out`、`probes.out` 的 `name=` 行与 `test result` 行。`name=send_bounds_failed` 就是有类型过不了 `Send`，读 `send-bounds.out` 的编译错。探针不断言「应当怎样」，`test result: ok` 只说跑完，判定看上表「翻面」一列。

**单跑探针（整份复跑的拆法，二选一）**

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 runtime_design_attack_probes; 范围=1 个测试目标; 为什么=运行库设计轮攻方腿量后台入口在周期线程次序下的读数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-background-runtime-design-r1-opus/copy -- cargo test -p singlefs-verification --test runtime_design_attack_probes -- --nocapture --test-threads=1
```

要先把 `src/runtime_design_attack_probes.rs` 拷进副本的 `crates/singlefs-verification/tests/`（整份复跑由 rerun.sh 做了）。判决要引的数按 `.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做` 由主 agent 复跑。

**不要跑的**：门禁阶段一条都没写进来；真卡特性没开。

## 没做什么

- 没判正推腿的格，没替主 agent 采纳；Q1、Q2 的接口草案与水位函数归正推腿，我只打。
- 水位默认值没定数，岔路写的是候选。
- 没往统一任务服务交任务；模型、探针、rerun.sh 都没跑（命令在上一节）。副本上的数不算入库装置上的数，判决要引的数要主 agent 复跑。
- 没写崩溃枚举原型（限度一节）。
- 正文 Q 之外的：A1 的批大小、B3 的写失败、C2 的读、C3 的切换入口、Q4 的 N 张卡，都是正文没问、我多报的，要不要算进这一轮归主 agent。
- 没删的副本：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-background-runtime-design-r1-opus/copy`：主 agent 整份复跑时 `--frozen` 指它，`.deliverables` 写成交接，跑完由主 agent 删。

**什么现象会推翻这份报告的主结论**（「一把锁加周期线程这个形态照现在的预算、判据与锁内 GPU 接上去会错」）：探针翻面列全部撤回，且主 agent 判 D24 已定项 2 第 2 条管不到等锁、D16 已定项 1 的边界句豁免周期整理。
