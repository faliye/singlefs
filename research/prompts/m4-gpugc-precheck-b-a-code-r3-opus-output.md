# m4-gpugc-precheck-b-a-code-r3 攻方腿（Opus）报告（2026-11-13）

立场：假设乙、甲两轮站住的样子放到今天这棵树上是错的，找打穿它的历史；另找三轮下来没有去向的形。主攻 T1、T2。
引代码一律是冻结副本 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3/repo/` 里的路径与行号；下面点名的 `crates/` 文件此刻主工作区与冻结副本 sha256 逐份相同（`allocator.rs` `eb822f09…`、`mounted_session.rs` `cf3db681…`、`admission.rs` `7cc59211…`、`transaction.rs` `d6b2083d…`、`mount.rs` `ae249401…`、`accept_raises_the_floor_for_space.rs` `4d98ac04…`、`mutations.tsv` `8473fc55…`，量的）。
这一腿没交任务、没起 cargo：探针的读数全在「要主 agent 跑的命令」那一节等主 agent 跑，报告里凡没跑出的数都标「推的」。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpugc-precheck-b-a-code-r3-opus-model/`（第二轮攻方探针 `research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model/` 拷来改）。复跑（整条交服务；运行目录在工作根下、以 `-run` 收尾，包一层的脚本先删它再调 `tools/rerun.sh`）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3-opus-probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r3-opus-model
```

每个文件的 sha256（`sha256sum` 现算，量的）：

```
de8133f1af23c60c211b3548d984f9e8f357f17d3b3c21b1df2382872ffd1f98  ./probe/Cargo.toml
d0fe00ef0ddf73c65744c5d4d123a021640e628b6649f8d47935a9ead1ab5ab3  ./probe/core_probe_module.rs
81a8ae9934428a5a09ec1f59cc0f0567e49eb41aa7135ab584d9f956338959a3  ./probe/src/device.rs
61e34d34fe88c78f638e4d48b92dc700b1f05a85167abfee6803accf0c962a8d  ./probe/src/main.rs
e4125643126be91539e6a445988eb0719c53d82f89c8429260d2dc3ddeb7b4ac  ./probe/src/pool.rs
105a7b2a5cbef3f0f41b5ff1f2f28c158ffcea12f35281cee984c9d63385d758  ./probe/src/ruler.rs
1c99eba7816d7cfe8912559af1dfb817476c3e159177e0aef5d8eabaae3ab7b6  ./probe/src/shapes.rs
ca60aa3eb94485a3939a1a75237fc111235574a8eb93afec6ff2acd8d8beb574  ./probe/src/tally.rs
422da5ea0c01a6fcb954e9537c3c04511eacab7d20daf32392b8157875d887be  ./tools/patch_the_copy.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  ./tools/rerun-from-a-clean-work-directory.sh
a032f5570e4896a3a40f92aa7d5e7dfde335111c278bc7d9a04c65fd70dc8a5f  ./tools/rerun.sh
```

与第二轮那一份比改了什么：
- `tools/patch_the_copy.py` 三处锚点（推串之前合取 7 那一判、放不下的盘那一判、推一串抬 F）与工作区成员那一行，在这一轮冻结副本里现查各恰好一处（`mounted_session.rs` 第 6211、6121、6328 行起，`Cargo.toml` 第 17 行），原文没动，只改文件头。
- 新加 `probe/src/ruler.rs`：每一次改动之前与之后，每块盘比一次乙读的 `slot_pairs_without_an_unreleased_record_outside_the_cluster_segments` 与落得下那一判读的 `unblocked_slot_pairs_outside_the_cluster_segments`（两样都排会话分配器此刻的聚簇段），前者小于后者记 `ruler bound-below-landing`；判出放不下的每一格另按被拒的那一次分 `last-refusal=`（`SpaceAdmissionRefused`、`PlacementRefused:<落点拒绝的成员>`、别的）与 `last-refusal-class=`，O 那一臂的结局与误拒也按类分；O 那一臂推没推串分 `oracle-series=0`、`1+`。
- `tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps`：仓副本拷到任务所在那台的 `$TMPDIR`（照第二轮主 agent 核查时的改法），跑两份种子——从 1 起 155 个（与第二轮攻方 `30a9b0ce…` 那一份同一批种子，比两棵树）、从 1001 起 155 个（新取样），各 250 步；每个配置往标准错误打一行 `PROGRESS`。
- 估时上限照正文第五节写 100 分钟（`main.rs` 的 `WALL_CLOCK_LIMIT_SECONDS`）；第二轮同一族 155 个配置在服务上 `WALL_SECONDS 512.3`（`~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2-main-check-opus-run/out/oracle.out`，量的），两份合起来推的约 17 分钟加编译。
- 这一探针不走崩溃枚举（不调 `enumerate_crash_states`，剪刀那一条路在 `shapes.rs` 里没用），定义第 3b 步的状态数与约 10⁶、10⁷ 两条线不适用；崩溃状态一个没缩。

跑前写下的（结果反过来我怎么改）：
- 预期从 1 起那一份 `PROBE_RESULT` 里与第二轮同名的字段逐字段等于 `30a9b0ce…` 那一行（今天的树只换了用户数据落点的取法与回落的名字，按 `allocator.rs` 第 1313–1316 行它与逐个候选看答得一样）。任一同名字段不同，T3 那一格改判「旧树上的读数不能搬过来」，不同的字段逐个交主 agent。
- 预期 `ruler_bound_below_landing=0`（下面 T1-1 的不变式）。非 0 即乙在今天的树上读的不是落得下那一判的同一把尺，T1-1 翻成打中。
- 预期 `misrefused_slot_pairs=0`、`misrefused_conjunct_before_any_series=0`。非 0 即误拒，样例行 `EXAMPLE MISREFUSED` 是反例。

## 各格判定一览

| 格 | 判定 | 依据的种类 |
|---|---|---|
| T1-1 乙的上界与落得下那一判是不是同一把尺 | **站住**：同一个聚簇段集合、同一种「偶数起点、不跨段」的对；乙数的对是落得下那一判数的对的超集，上界不会小于落得下的读数 | 读码推的不变式；探针 `ruler_*` 逐步核，待主 agent 跑 |
| T1-2 甲读的全空段数与发布路径合取 7 读的是不是同一个数 | **站住**：同一个函数、同一个 `N₀`；判不判的范围只在安全一边不同（建快照、改 size 甲不先判） | 读码 |
| T1-3 今天的树上重跑两个误拒读数 | **待主 agent 跑**：模型与命令已交；跑前预期写在上面 | 推的 |
| T1-4 甲也在「取落点被拒」上判出（落点那一道拒在合取 7 之前） | **形站住、没有条款与用例**：代码文档写了，kb 射程与用例都没写；判出格数待探针 | 读码；读数推的 |
| T1-5 第 131 批四条修补 | 四条都对得上代码（文档、`SlotPairCount`、断言、重建用例）；文档里「头一次被拒就判到这里时它们是空的」仍是 C1003 记着的那一句 | 读码 |
| T2 未定项清单 | 逐问判在 T2 一节；没去向的三形：甲在取落点被拒上判出、落得下那一判与乙都数活意图 `R` 里的槽对、第一轮判决「登欠账量代价」那一条没单独落账（C970 罩着一半） | 读码、grep |
| T3 读数能不能核 | 第二轮判决「有效分母是 1082 与 1039」两个数不是同一口径：1082 是 O 那一臂推过串的，1039 是第二族全部判出格，没拆；3643 同样没拆。探针加了拆法 | 产物 grep，量的 |
| 检查单八格 | 走不到的理由 成立；类型与名字 产品代码成立、全仓照字面不成立；不兼容旧 成立；判定库 成立；多机 成立；多线程 成立；--item 成立；新崩溃流 不需要 | 读码、grep |

## T1　「已定」项撑不撑得住（主攻）

### T1-1　乙的上界与落得下那一判：同一把尺，乙数的是超集（站住，推的不变式，探针逐步核）

两个读数读的是同一个聚簇段集合。落得下那一判：`crates/singlefs-core/src/admission.rs:996`：「.unblocked_slot_pairs_outside_the_cluster_segments(」，参数是 `crates/singlefs-core/src/admission.rs:997`：「allocator.cluster_segments(),」；乙：`crates/singlefs-core/src/mounted_session.rs:6069`：「.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(」，聚簇段取 `crates/singlefs-core/src/mounted_session.rs:6138`：「allocator.cluster_segments(),」。两处读的都是被拒之后、推串之前同一个分配器（发布路径在动分配器之前判落得下，`crates/singlefs-core/src/transaction.rs:13412`：「let units_landing_is_judged = match allocator.space_admission() {」，之后才 `crates/singlefs-core/src/transaction.rs:13435`：「let settled = settle_the_allocation_record_tree(」）。聚簇段集合只增不减：全文件只有 `crates/singlefs-core/src/allocator.rs:3075`：「self.cluster_segments.insert(segment_start);」一处改它（`grep -n 'cluster_segments\.\(insert\|remove\|clear\|retain\)\|cluster_segments = '` 现查，量的），重挂才清空。

乙数的对是落得下那一判数的对的超集，依据是一条不变式「罩在未释放记录下 ⇒ 分配位置着」：
- 置「罩在未释放记录下」只在分配与复活：`mark_allocated` 先置分配位（`crates/singlefs-core/src/allocator.rs:1287`：「self.set_blocking_bit(index, SlotBlockingBit::Allocated, true);」）再记进未释放记录；复活先断言分配位置着。
- 清分配位只在回收：`crates/singlefs-core/src/allocator.rs:1193`：「self.set_blocking_bit(index, SlotBlockingBit::Allocated, false);」，调它的只有回收已释放记录那一处（`crates/singlefs-core/src/allocator.rs:3337`：「singlefs_replay_prefix::gc_crash_proofs::released_record_can_be_reclaimed(」筛过，`crates/singlefs-core/src/allocator.rs:3355`：「device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));」），已释放的那几槽在释放时已清掉罩在未释放记录下那一位，见 `crates/singlefs-core/src/allocator.rs:996`：「UnreleasedRecordTransition::SlotsLeaveTheirUnreleasedRecord => false,」；同一槽不会同时罩在两条记录下（`mark_allocated` 断言跨度事先全空）。
- 所以落得下那一判认的一对（两槽三种位都没置）两槽都不罩在未释放记录下，乙认它；乙另认 defer、隔离、扣住那几种。上界读数 ≥ 落得下读数，两判在 `<` 上比的都是 `SlotPairCount`，乙按一个数据单元的槽数换成字节、需求也按同一个数换（`crates/singlefs-core/src/mounted_session.rs:6094` 与 `crates/singlefs-core/src/admission.rs:1077`），「落得下放行而乙判放不下」在这条不变式下走不到；乙只在落得下那一判拒之后才判（`crates/singlefs-core/src/mounted_session.rs:6122`），本来也不会碰到落得下放行的格。

别的会话换的两样不碰这条尺：用户数据落点按段取只换取法（按有槽对的段号集合走），认的对还是 `is_unblocked_pair_at_index` 那一种；回落改名与新的拒绝成员 `CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice` 都是 `PlacementRefused`，乙只读 `SpaceAdmissionRefused`（`crates/singlefs-core/src/mounted_session.rs:6122`），不受影响。

反推：乙错的话，今天的树上应当看到某一步某块盘乙的读数小于落得下的读数。探针每次改动前后各核一遍，记 `ruler_bound_below_landing`（待主 agent 跑，推的 0）。推翻它的现象：这个数非 0，或 `EXAMPLE RULER_BOUND_BELOW_LANDING` 有行。

同一把尺之外看到的一处（不分辨乙的对错，记进 T2）：两判都不排活整理意图的 `R`，而用户数据落点排 `R`（`crates/singlefs-core/src/allocator.rs:1330`：「&& !excluded_region.is_some_and(|region| region.contains_slot(segment_start));」）。`R` 里的槽对落得下那一判与乙都数、落点不取：上界只会更大，是白推一边，不是误拒一边。

### T1-2　甲读的全空段数与发布路径合取 7：同一个数（站住，读码）

- 读数：甲 `crates/singlefs-core/src/mounted_session.rs:6209`：「.map(|device_map| (device_map.device, device_map.empty_segments()))」，发布路径 `crates/singlefs-core/src/transaction.rs:13476`：「.map(|device_map| (device_map.device, device_map.empty_segments()))」；`N₀` 两处都取 `allocator.compaction().switches.empty_cluster_segments_floor.segments()`，判法同一个函数 `admit_the_empty_cluster_segments_floor_on_every_device`。
- 准入开关：甲 `crates/singlefs-core/src/mounted_session.rs:6197`：「SpaceAdmission::SkippedByTheTestOnlySwitch => false,」，发布路径同一个开关关掉合取 7。
- 判不判的范围：发布路径判「前台写入 ∧ 不是回退 ∧ 没有搬迁 ∧ 有普通分配」；甲按用户改动的种类判，建快照与改 size 甲归「发之前不知道」照推（`crates/singlefs-core/src/mounted_session.rs:6167`–`6170`），发布路径把建快照、变长与只改记录算前台写入。差在安全一边：甲少判，只多白推，不误拒。用户改动里没有回退、搬迁另走 `publish_the_relocation_batch_raising_the_floor_for_space`（不经先算，见 T2-1）。
- `N₀` 只有产品值与只供测试的 `ForcedForTests`；整理关着（`run_forcing`）不改 `N₀`，甲与发布路径都不读 `run_forcing`。可写挂载把产品那一档原样装上（`crates/singlefs-core/src/mount.rs` 第 9758 行起解构、第 9775 行装回）。

反推：甲读的若不是同一个数，应当看到甲判出、O 那一臂推一串之后合取 7 照样放行的格。第二轮第二族 3643 格里 O 那一臂结局是 `NoSpaceAfterRaisingTheFloor` 3399、`Publish:EmptyClusterSegmentsBelowTheFloor` 244、写进 0（`…-r2-main-check-opus-run/out/oracle.out` 第 66、67 行，量的，旧树）；今天的树上重取待主 agent 跑。

### T1-3　今天的树上重跑两个误拒读数（待主 agent 跑）

派发第 1 条要的两个读数（槽对那一判、合取 7 那一判各自的判出格数与 O 那一臂写进格数）在探针 `PROBE_RESULT` 里是 `fired_by_slot_pairs`、`misrefused_slot_pairs`、`fired_by_conjunct_before_any_series`、`misrefused_conjunct_before_any_series`，另加有效分母 `fired_by_slot_pairs_oracle_pushed`、`fired_by_conjunct_oracle_pushed`（O 那一臂推过至少一串的格）。两份种子各一行，命令见「要主 agent 跑的命令」。第二轮旧树上同一批种子（从 1 起）的读数：`fired_by_slot_pairs=1039`、`fired_by_conjunct_before_any_series=3643`、两个误拒都是 0（`…-r2-main-check-opus-run/out/oracle.out`，量的，旧树）。这一格的结论等读数：两份都是 0 才记这一轮的「没打中」；有一格非 0 就照 `EXAMPLE MISREFUSED` 那一行当反例交主 agent。

### T1-4　甲也在「取落点被拒」上判出（形站住；kb 射程与用例都没有）

- 甲判不判看的是 `refusal_is_short_of_space`（`crates/singlefs-core/src/mounted_session.rs:6200`），它把取落点被拒也算空间不够：`crates/singlefs-core/src/mounted_session.rs:2120`：「PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,」。
- 发布路径上取落点被拒发生在走固定点那一步（`crates/singlefs-core/src/transaction.rs:13435`），排在合取 7（`crates/singlefs-core/src/transaction.rs:13461`：「let empty_cluster_segments_conjunct_is_judged = match (」）之前：这一次合取 7 没判过。甲替它判的是「推了之后合取 7 也会拒」，与它在落得下那一判先拒时的理由同一条（推串不改写之前那一刻的全空段数，推高越不越过 `N₀` 不另估）。
- 全空段少于 `N₀`（第一版 `N₀` 是 1，即有一块盘全空段是 0）时提交内生块开不了新段，只能回落；回落取不到每块盘都空的落点正是今天的树新分出来的那个成员，它的文档说推抬 F 之后同一次写可能落得下：`crates/singlefs-core/src/allocator.rs:296`：「与段对齐那一条分开报：这一条不是没有条款的池形状，是各盘的空槽错开了——腾出空间（推抬 F、整理、删）之后同一次写可能落得下。」甲在这一形上一串不推就拒——推了落点放得下、合取 7 照拒，结论不变，所以这不是误拒（推的）；但它是先算在一种拒绝上替合取 7 预判，代码文档写了（`crates/singlefs-core/src/mounted_session.rs:699`：「判出这几块盘的那一次被拒（`SpaceAdmissionRefused`，合取 7 那一判判出时也可以是 `PlacementRefused`）：这一次发布没有推抬 F。」），kb 没写：`.claude/kb/decisions/16-发布语义.md:276` 射程里写先算的那一段只说「推抬 F 放得出的全放出来也放不下」与三处入口，没提取落点被拒（读的是这一行全文，量的）。
- 用例：`mounted_session.rs` 里甲的五条单测都用落得下那一判的拒绝造（`allocator_without_an_empty_cluster_segment_and_its_refusal` 调 `refusal_of_the_landing_judgement`，第 8715–8727 行），验证包的真池用例也是落得下与合取 7 那两种；变异表里「抬 F 先算：不是空间不够的被拒也先判合取 7」（`crates/mutations.tsv` 第 7694 行）点的是另一边。没有一条用例或变异行点「取落点被拒时甲判、照合取 7 的理由不推」这一支（`mounted_session.rs` 的 `everything_a_floor_raise_could_release_tests` 模块从第 8429 行到文件末尾，`PlacementRefused` 0 处，量的；`.claude/kb/decisions/16-发布语义.md:276` 整行里「落点」0 处，量的）。
- 四句：不分辨乙甲之外的臂（只问甲的射程）；被判的系统看得到（拒绝的种类就在手上）；满足的是 D16 已定项 10 射程「抬 F 之前先算」那一句没写到的分句；这一轮没有跑前条款给的改法。
- 读数：探针 `fired_by_conjunct_on_placement`、`misrefused_conjunct_on_placement` 给这一形的判出格数与误拒（待主 agent 跑）。去向见 T2-2。

### T1-5　第 131 批四条修补逐条对代码

1. 四处文档（`mounted_session.rs` 第 658–660、687–691、2939–2940、6832–6833 行）与代码逐句对：两形分开写，与 `judge_before_pushing_a_floor_raise_for_a_refusal`（第 6228 行起：先判 `devices_short`、再判合取 7）一致；「这次调用还一串没推」对 `series_pushed_in_this_change > 0` 就不判（第 6199 行）；「写之前那一刻」对甲读的是被拒那一刻的分配器。对不上的一句仍是第 660 行的「头一次被拒就判到这里时它们是空的，盘上逐字节不变」：调用开头先走空发成几步、走空没推串，之后改动自己被拒、甲判出，这时 `versions_published` 非空（C1003 已记这一句，见 T2-1）。
2. `SlotPairCount` 换齐之后 `admission.rs` 的比较语义不变：`<` 两边都是 `SlotPairCount`（派生的 `PartialOrd` 只比那一个字段），需求从 `units.data_units` 包一层，换字节那一处先 `.0` 再乘一个数据单元的槽数（第 1077–1082 行），与改前逐值相同（读码）。
3. 走空那条真池用例补的三句断言（`stop` 是抬 F 那一串自己放不下、`floor_raises` 与推它们的被拒都空、`versions_published` 空）与用例造的池（单元区空闲槽全记成已分配）对得上，读码。
4. 重建之后比计数的用例钉的是挂载真走的那条路：可写挂载重建分配器调的正是 `rebuild_from_records`（`crates/singlefs-core/src/mount.rs:3248`：「PoolAllocator::rebuild_from_records(device_maps, output.allocation_records.clone()),」、`crates/singlefs-core/src/mount.rs:3762`）；用例比的三样（逐槽位、按段槽对数、全盘槽对数）只随 `mark_allocated` 与 `mark_released` 变，隔离与扣住不进这三样，所以随机历史里重建不重放隔离、扣住也不会让它误红。它没钉的：挂载在 `rebuild_from_records` 之后另装的隔离、扣住、根环那几样（不在这三样里，本来不归它）。

## T2　未定项清单的完整性（主攻）

### T2-1　已登的去向逐条判

| 形 | 去向在哪 | 判 |
|---|---|---|
| C1002（`.claude/kb/checks-owed.md:606`） | 行文还写旧上界「按「每个放得开的槽各补一对」算上界」与「（改法乙，零轮）」「（改法甲，零轮）」，去向 `05-fifth-txn.md` 十一·4；第一轮判决第三节第 3 条要三轮判完由写回员把去向改成实现的那两处、验收形写成真池用例 | **有去向**（写回员，三轮之后）。此刻行文与代码对不上，是写回没做，不是没去向 |
| C1003（`.claude/kb/checks-owed.md:607`）与卸载重挂查询 | 用户定「做成单独的查询」、排进里程碑五，记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 1051 行写明「去向就是 C1003 那一行已写的去向」，读数与这句定案随写回补进；`05-fifth-txn.md` 第 185 行十一·4 那一行的编号表里有 C1002、C1003（`awk 'NR==185'` 取编号，量的），这一行的题目是根环，没写「查询」 | **有去向**（写回员补进 C1003 那一行）。另：C1003 写「头一次被拒就判出时盘上逐字节不变」在开头先走空发成几步时字面不成立、「可达要一次瞬时读写错」；我推还有一条路不要读写错：两次调用之间后台整理跑一批（`MountedSession::run_compaction_in_the_background`，`mounted_session.rs` 第 3296 行）腾出地方，下一次调用开头走空不推串就发成几步，接着改动自己被拒、甲判出，`versions_published` 非空。推的，没造（探针不跑整理）。写回 C1003 时把「可达要一次瞬时读写错」放宽成「要两次调用之间分配器变过（瞬时读写错、后台整理）」 |
| 第一轮判决「交用户的」第 2 条（误拒只一份取样，登欠账） | 第二轮判决「交用户的」写「第一轮登的那条欠账可以关」；`checks-owed.md` 里没找到这一行（`grep -n '误拒.*取样\|一份取样'` 0 行，量的） | **有去向**（第二轮判它可关）；只是从没开过，写回时不用找它关 |
| 第一轮判决表里「每次发布克隆分配器多一张每槽一字节的位图……登欠账量代价」（`research/prompts/m4-gpugc-precheck-b-a-code-r1-main-verification.md:34`） | 没单独落账（`grep -n '每槽一字节\|克隆分配器\|位图'` 在 `checks-owed.md` 只命中不相干的两行，量的）；C970（`.claude/kb/checks-owed.md:576`）写「调用方与固定点每轮整份拷分配器」的步数与挂钟没有数，罩着这一份多出来的位图 | **有去向一半**（C970 罩着「整份拷分配器」，推的）；第一轮算的「按 1 TiB、每槽 16 KiB 每次每盘多 64 MiB」没进任何一行。该登：写回时并进 C970 那一行的依据格，点名乙加的 `is_under_an_unreleased_record` 位图 |
| D16 已定项 10 射程「不罩改动发成之后另发的搬迁批」（`.claude/kb/decisions/16-发布语义.md:276`） | 今天的树上搬迁批被空间拒直接推串：`publish_the_relocation_batch_raising_the_floor_for_space`（`mounted_session.rs` 第 2432 行）里调 `push_one_floor_raise_within_the_admission_budget`（第 2490 行），不经 `judge_before_pushing_a_floor_raise_for_a_refusal` | **成立**，射程本身是去向 |
| 同一句「也不判提交内生块那一判（照推）」 | `crates/singlefs-core/src/mounted_session.rs:6097`：「JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,」 | **成立**；这一判只数槽不数对的病根另在 C1017，不归先算 |

### T2-2　没列过的形

| 形 | 读码依据 | 去向 | 该登哪里 |
|---|---|---|---|
| **甲在取落点被拒上判出**（T1-4） | 判不判看 `refusal_is_short_of_space`，取落点被拒算空间不够；那一次合取 7 在发布路径上没判过 | **没去向**：代码文档有（第 699 行），kb 射程、用例、变异行都没有 | D16 已定项 10 射程写回时并进先算那一句（「合取 7 那一形在落得下那一判或取落点被拒时都判」），验证包补一条真池用例：全空段为 0、回落取不到每块盘都空的落点（今天树上的新成员），先算不推、报合取 7 那一份；变异行「取落点被拒时甲不判、照推」 |
| **落得下那一判与乙都数活意图 `R` 里的槽对，用户数据落点不取 `R`** | 落点 `crates/singlefs-core/src/allocator.rs:2930`：「let excluded_region = self.compaction.region_excluded_from_placement();」，按段取时跳过 `R` 那一段（第 1330 行）；`PlacementRoomOnOneDevice::of_every_device_of_the_allocator` 与乙都只排聚簇段。`R` 是签发时挑的稀的段，里面空的槽对多 | **没去向**（`grep -n 落得下 .claude/kb/checks-owed.md` 的命中里再筛 `` `R` `` 0 行、再筛「R 里|R 那一段|活意图」0 行，量的）。只往白推一边偏：落得下放行、取落点被拒、推串；全空段已少于 `N₀` 时甲接住（T1-4） | `checks-owed.md` 新一行，挂 D3（空间分配） 已定项 8 与 D26（后台整理与放置回收） 已定项 1 第 3 条，与 C545（空间准入罩不住分裂与聚簇段层）、C1017（准入只数槽不数对） 并排；不归乙、甲这一题（不分辨它们） |
| 只读挂载 | 只读挂载走 `mounted_read.rs` 第 651 行 `mount_read_only`，没有发布入口 | 不成形 | 无 |
| 冻结 | 冻结之后发布报 `PublishFrozenAfterAWriteFailureIsNotResentYet`，不算空间不够（`mounted_session.rs` 第 2141 行在不算的那一串里）；乙只读 `SpaceAdmissionRefused`、甲先判空间不够，两样都不判出，原样交回 `Publish`；`accept` 冻结之后在任何读写之前拒是已定项 10 定案原文 | **有去向**：条款有；单测 `a_refusal_not_short_of_space_is_not_judged_against_the_empty_cluster_segments_floor`（第 8808 行）与变异行第 7694 行罩「不是空间不够不判」那一类，冻结那一种没有专门用例（推的，结构上判不出） | 无需另登 |
| 准入关着（只供测试的开关） | 甲 `mounted_session.rs` 第 6197 行不判；乙判不出——准入关着发布路径不判落得下与式子（`transaction.rs` 第 13416–13418 行），产不出 `SpaceAdmissionRefused` | **有去向**：单测第 8830 行、变异行第 7695 行 | 无 |
| 整理关着或换开关时甲判不判 | `N₀` 只读开关里的 `empty_cluster_segments_floor`，产品恒第一版的 1，`ForcedForTests` 只供测试；`run_forcing` 两边都不读 | 不成形：两边读同一个字段 | 无 |
| 整理停着（后台计算出错之后）而全空段已少于 `N₀` | 合取 7 拒每一次前台写入，甲只是先一步不推；出路只有整理（停着）或卸载重挂 | 不归这一题（合取 7 自己的性质，不分辨甲有没有） | D26（后台整理与放置回收） 已定项 13 第 4 条补跑那一格（推的，没核那一格写没写这一形） |

## T3　报出的数能不能核（不是主攻，只记一条口径与搬不搬得过来）

现核的（量的，旧树上的产物）：两份 `oracle.out` 的 sha256 前缀对得上判决：`…-r2-main-check-opus-run/out/oracle.out` 是 `30a9b0ce5b288e41`、155 个配置，`…-r2-main-check-sonnet-run/out/oracle.out` 是 `32c69feb2c70caaa`、310 个配置。后者第 95、96 行 `oracle COUNT fired by=slot-pairs oracle-series=0 825`、`oracle COUNT fired by=slot-pairs oracle-series=1 1082`；第 87 行 `… count-below-floor=yes 62`；第 45、51 行 `413`；第 125、131 行 `short-before` 362、59，第 126、132 行 `short-before empty-segments=same` 同为 362、59（推高 0，0/421 算得对）；`conjunct_first_forced_published=0`。

**打中一处口径**：第二轮判决第二节把 1082 与 1039 并称「有效分母」（`research/prompts/m4-gpugc-precheck-b-a-code-r2-main-verification.md:34`：「**更正分母**：有效分母是 1082 与 1039，不是 19351。」）。1082 是第一族槽对那一判判出格里 O 那一臂真推过至少一串的（825 格 O 那一臂一串没推）；1039 是第二族槽对那一判全部判出格，第二族那份产物里没有按 O 那一臂推没推串拆（`grep -c 'oracle-series'` 在 `…-opus-run/out/oracle.out` 是 0，量的）。两数不同口径。甲第二族的 3643 同样没拆。
- 四句：不分辨臂（是读数口径）；判别子看得到（探针里就有 O 那一臂推了几串）；满足的是判决那一句「有效分母」的定义分句（第一族那一个照「O 那一臂真推了至少一串」更正过，第二族没照同一定义）；改法是拆开重数——这一轮探针加了 `oracle-series=0/1+`，`fired_by_slot_pairs_oracle_pushed`、`fired_by_conjunct_oracle_pushed` 就是两族同一口径的有效分母（从 1 起那一份种子与第二轮第二族相同，待主 agent 跑）。
- 后果：「乙、甲误拒各两份取样都是 0」不变（写进 0 不随分母变），变的是能引的分母；第二族 O 那一臂结局全是 `NoSpaceAfterRaisingTheFloor`（第 91 行 1039 格，量的），其中有多少一串没推、等于没试，要等拆开的读数。

搬不搬得到今天的树：今天的树换了用户数据落点的取法（答案按代码文档与对拍用例相同）、回落的名字与一个新拒绝成员（仍是 `PlacementRefused`），乙、甲读的三样没动（T1-1、T1-2）。我推第二轮两族的读数在今天的树上逐字段重现；判据是这一轮从 1 起那一份 `PROBE_RESULT` 与 `30a9b0ce…` 那一行同名字段逐个相等（跑前写死在报告开头）。不等的字段标「旧树上量的」。记录第 1047、1050 行点名的四个任务的产物这一腿没核（归正推腿）。

## 检查单八格

| 格 | 判 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | **成立** | 改动全文加号行里 `assert`、`expect`、`unreachable`、减号逐个看（`grep -n '^+' … | grep 'assert\|expect\|unreachable\|panic\|unwrap\| - \|-='`，5 处）：一处是 `allocator.rs` 那一个减法换行包进 `SlotPairCount(…)`、与删掉的那一行同式；四处是测试里的 `assert_eq!` / `assert!`。产品代码没有新断言、新 `expect`、新减法 |
| 类型与名字 | **产品代码成立；全仓照字面不成立** | 产品代码里槽对数都带 `SlotPairCount`（`UnitsOfAPublishToLand::data_units` 是数据单元个数，比的时候包一层）。验证包与装置在读的地方 `.0` 拆回裸 `u64`：`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1082`（`bytes_of_slot_pairs(pairs: u64)`）、`:1091`（`slot_pairs_counting_one_pair_for_every_slot_a_floor_raise_could_release` 交 `u64`，把槽对数与 defer、隔离、扣住的槽数加在一起——故意造旧上界，测试本意）、`:1322`（`slot_pairs_left: u64`）、`crates/singlefs-core/src/mounted_session.rs:8575`（测试模块里同名函数）、`crates/singlefs-implementation-experiments/src/bin/e180_admission_reading_versus_writable_bytes/probed_families.rs:18`、`crates/singlefs-implementation-experiments/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound/fixed_point_upper_bound.rs:239`。低危、只在测试与装置里；第二轮判决第三节第 2 条要「读它们的地方跟着换」，实现员换成了在读处拆箱，照字面没换齐。新用例名 `slot_pairs_once_everything_is_released_are_the_same_after_rebuilding_the_allocator_from_its_records` 没有缩写 |
| 不兼容旧代码与旧格式 | **成立** | 改动只有类型包装、文档、测试与变异行，没碰盘上字节、没有版本分支（读改动全文） |
| 不新建判定库 | **成立** | 同上，没有新的存放处 |
| 所有任务多机可用 | **成立** | 新用例与补的断言都在内存里现造（`DeviceFreeMap::new` 与内存稀疏盘），不读本机状态 |
| 测试优先多线程 | **成立** | 新用例写了不并行的理由（`crates/singlefs-core/src/allocator.rs:4226`：「/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。」）；理由罩的是历史，每一步那一次重建与比对本可以拷状态并行，但一次重建是两块 385 槽的盘，整条 600 步推的毫秒级，不值得 |
| 带 --item 的命令与项 | **成立** | 删掉的 4 个旧名（第二轮冻结副本有、这一轮没有的首列，`comm -23` 现算：「实审 A4d C545：落得下那一判不判数据单元（段外槽对不够照样放行，256 槽那一格走到固定点、交回 PlacementRefused）」与三条「…（槽对数类型）」的旧名）在这一轮冻结副本的 `crates/mutations.tsv` 首列里 0 处；新加的 5 行在第 7761–7765 行。首列不同的其余几十行是别的会话的批，不归这一批 |
| 新崩溃流多罩了什么 | **不需要（推的）** | 乙、甲新拒的那几形一个写都不发（`judge_before_pushing_a_floor_raise_for_a_refusal` 只读分配器），没有自己的崩溃点；它们省下的推串已有崩溃流。唯一写盘的边角是「开头走空发成几步之后甲判出」，写的是走空那几步，已有走空的崩溃流 |

## 这一腿提的改法各修哪一格

全是只在读码上推的、被攻过零轮，没在模型上量过（探针只量读数，不实现改法）。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| D16 已定项 10 射程写回时，先算那一句补「合取 7 那一形在落得下那一判拒或取落点被拒时都判」；验证包补一条真池用例（全空段 0、回落取不到每块盘都空的落点，先算不推、报合取 7 那一份）与一行变异「取落点被拒时甲不判、照推」 | T1-4、T2-2 第一行 | 推的 |
| 写回 C1003 时把「可达要一次瞬时读写错」放宽成「两次调用之间分配器变过（瞬时读写错、后台整理）」 | T2-1 C1003 那一行 | 推的，没造 |
| `checks-owed.md` 新一行：落得下那一判与乙数活意图 `R` 里的槽对、落点不取 | T2-2 第二行 | 推的，没造 |
| C970 那一行依据格补第一轮判决算的「克隆多一张每槽一字节位图」 | T2-1 第四行 | 推的 |
| 第二轮判决第二族那两个分母照第一族的定义拆开重引（用这一轮探针的 `fired_by_slot_pairs_oracle_pushed`、`fired_by_conjunct_oracle_pushed`） | T3 口径那一条 | 推的，待主 agent 跑 |

## 要主 agent 跑的命令

1. 探针复跑（两份种子，这一腿的全部读数都在里面）。在仓根起：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3-opus-probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r3-opus-model
```

   - 在哪起：仓根 `/home/fy5090/code/singlefs`（命令里全是绝对路径，`--cwd` 不用给）。仓副本由 `rerun.sh` 拷进任务那台的 `$TMPDIR`、跑完删；输出只写运行目录 `…/m4-gpugc-precheck-b-a-code-r3-opus-probe-run/out/`（`patch.out`、`build.log`、`oracle-seeds-from-1.out`、`oracle-seeds-from-1001.out`）。主 agent 照规则拷模型目录到自己的草稿目录跑时，第四个参数换成拷贝的路径。
   - 看哪几行判结果：`TASK_SERVER_RESULT` 那一行 `exit=0` 算跑完；`task-server/task log <号>` 里 `name=exit_code family=seeds-from-1 oracle=0`、`name=exit_code family=seeds-from-1001 oracle=0`、两份 `name=self_checks bad=0`、末行 `name=rerun_done`；读数看两份 `oracle PROBE_RESULT` 行里的 `fired_by_slot_pairs`、`misrefused_slot_pairs`、`fired_by_conjunct_before_any_series`、`misrefused_conjunct_before_any_series`、`fired_by_slot_pairs_oracle_pushed`、`fired_by_conjunct_oracle_pushed`、`ruler_checked`、`ruler_bound_below_landing`、`fired_by_conjunct_on_placement`、`misrefused_conjunct_on_placement`，与 `COUNT fired by=… last-refusal=…` 各行；`EXAMPLE MISREFUSED`、`EXAMPLE RULER_BOUND_BELOW_LANDING` 有行就是反例。退 3 是探针估时超过 100 分钟没跑全量（`oracle ESTIMATE` 那一行有估时），退 1 是自证不合（`✗ name=self_check` 那一行点名哪一格）。
   - 内存与时间：推的，第二轮同一族 155 个配置在服务上 512 秒、31 线程；两份约 17 分钟加编译。内存这一腿量不到（没跑），服务按上限重排时 `rerun-from-a-clean-work-directory.sh` 先删运行目录再跑。
   - 跑完要接着读：新派同立场的腿带「承接：research/prompts/m4-gpugc-precheck-b-a-code-r3-opus-output.md」与任务号，照报告开头写死的三条预期判 T1-1、T1-3、T1-4 与 T3 那一条口径。

## 没打中的形状

- 乙的上界小于落得下读数：试了全部改「罩在未释放记录下」与分配位的路（`mark_allocated`、`mark_released`、`mark_resurrected_out_of_the_defer_queue`、`mark_reclaimed`、重建），找不到一条让一槽「没置分配位而罩在未释放记录下」的路（读码，范围是 `allocator.rs` 里这四个函数与 `rebuild_from_records`、`reclaim_released_records_up_to`）。
- 两判读不同的聚簇段集合：集合只增（全文件一处 `insert`），两判读同一个分配器、同一时刻（读码）。
- 按段取的新落点让乙或落得下的计数走岔：两判读的计数与段号集合同在 `set_blocking_bit` 维护，按段取不改计数；它跳过 `R` 那一处见 T2-2。
- 回落的新拒绝成员绕过先算：它是 `PlacementRefused`，乙不判（照推）、甲照判，与改名前的段对齐那一条同一类（读码）。
- 甲与发布路径读不同的 `N₀`：两处都读分配器上开关那一个字段，可写挂载装产品值（读码）。
- 重建用例钉的不是挂载那条路：挂载两处都调 `rebuild_from_records`（读码）。
- 取样范围：读码；探针两份种子各 155 个配置、各 250 步、8 个起点状态（4 档盘几何乘 2 档文件大小），只走 `publish_user_change`，不走 `accept`、不跑整理、不造读故障。

## 这条腿自己的限度

- 一个数都没跑出：T1-1 的「站住」是读码推的不变式，探针那一核等主 agent 跑；T1-3 整格待跑。
- 探针只走会话单发一条入口；`accept` 与走空两处的先算这一轮没取样（第二轮攻过 `accept` 的分类，这一轮不重复）。
- 探针不跑整理，`R` 永远不活，T2-2 第二行与 C1003 那条放宽都造不出；读故障的宽护也没造（C1003 原有的缺口照旧）。
- `last_refusal_kind` 按拒绝的 Debug 文字认成员名，成员改名时分类会落进 `other`（只影响分类计数，不影响误拒计数）。

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳。没往统一任务服务交任务、没起 cargo：探针读数、判决要引的数都等主 agent 复跑（命令在上一节）。
- 没核记录第 1047、1050 行点名的四个任务的产物（T3 主攻归正推腿）。
- 模型目录里的源码我只读过、没编过：编不过的话 `build.log` 末 80 行会打进任务日志，主 agent 新派同立场腿接着改。
- `tools/rerun.sh` 是从模板拷的、换了 `rerun_steps`，模板自带的自证跑的是模板那一段，对这一份不适用，没跑。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3-opus/` 里只有读码用的两份 diff、两份变异表首列清单与拼 `rerun.sh` 的片段，没有仓副本、编译目录与库拷贝。
