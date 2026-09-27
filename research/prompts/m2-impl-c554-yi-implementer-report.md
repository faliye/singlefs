# 实 C554 乙：挂载时有更新的根读不出，先重读一次，还读不出就拒可写——实现员报告

写于 2026-09-27（UTC 00:29 开工；JST 09:29 起）。规格 `/tmp/claude-1000/impl-c554-yi/spec.md`；里程碑 `.claude/kb/milestone/02-second-txn.md` 收尾批「实 C554 乙」。
交补丁：在草稿目录的副本 `/tmp/claude-1000/impl-c554-yi/work/` 里改，补丁在 `/tmp/claude-1000/impl-c554-yi/patch/`，主工作区一个字没动。
副本取于 2026-09-27 00:29:14Z（`rsync -a --exclude target --exclude .git`，主工作区当时 HEAD `260fa60a`），基线 sha 见 `base/sha-at-copy.txt`。

## 一、结论

1. 乙照 E158 第 3 次跑登记第 347 行（判据 N-配置）、第 363 行「乙-配置」、第 34 行（R = 1）与第 4 次跑登记 5.2 第 1 条 / 5.5 第 1 条（读缓存只收读成且非全零）实现在 `crates/singlefs-core/src/mount.rs`：
   - 可写挂载的读阶段（择根、扫 journal、重放）经一层这次挂载内有效的读缓存读；读完判 N-配置（系统配置槽直接读盘）。
   - 判真就在同一个读缓存上重做读阶段一遍（R = 1，产品路径不等；只供测试的钩子 `BeforeTheOneReread::CallTheTestOnlyHookFirst` 在重读之前调），重读那一遍判假就用它的所选根、记录、所选那一版往下走；仍判真，取号之前交回新成员 `MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read, reread })`，盘上逐字节不变。
   - 代码审阅第 22 条：重建分配器判抛弃用的「最新那条根的实例表」读不出就重读一次，仍读不出交回同一成员的 `InstanceTableOfTheNewestRootForTheShadowLedger`，不再按「没有根被抛弃」往下走。
   - 挂载做成时 `MountOutput::rereads` 报读阶段在哪一遍判完、两遍的读数、实例表在哪一遍读出（分支可观测）。
2. 验收三条（规格「验收」第 2 条）在新测试文件与改过的两份测试里都钉住了：暂时读错撤在重读之前（读报错、读回全 0 两种造法）与只坏一次（产品路径立即重读）⇒ 择到最新的根、不抛弃、池级 checker 一条不红；持续读不出 ⇒ 拒可写、`DiskSnapshot` 不变、同一组故障下只读挂载照常；第 22 条那一格持续读不出 ⇒ 拒，撤在重读之前 ⇒ 照隔离 14 槽。
3. 门禁 74 号那两条在副本上照 74 号跑法各跑一次（第三节）：快档 I-7.4 的新发现从 28 段归零；换成 46 段 `ModelDisagreement { aspect: "模型说该成、实现拒了" }`（「崩溃恢复抛弃根」那一步被乙拒了，模型还按今天的规则答「该成」）。收口表第 43 行那条用例仍红，红在第 2 步那一步被拒。符合规格「新发现归零，或只剩别的签名」，模型要不要跟着改交主 agent（第六节 Q2）。
4. 乙罩不到的一格照实钉住：系统配置没见证到的最新根（它那次发布的系统配置轮换没落盘）暂时读不出，照旧被当成被抛弃，崩溃恢复那次挂载看不见它、把它的单元再发出去，池级 checker 的 I-7.4 照实红（`a_newest_root_the_system_configuration_never_witnessed_is_abandoned_without_a_reread`、formatted_pool 的 `crash_recovery_abandoning_the_unwitnessed_row_publish_…`）。这是判据 N-配置 自己的边界（登记 PC-554 那一句写的就是 -配置 各臂与今天同一结局），交主 agent（第六节 Q1）。
5. 超出派发文件清单：改了 8 份 crates 文件（清单列 6 份，其中 `recovery.rs`、`lib.rs` 没动；多出 `history.rs`、`model_comparison.rs`、`bin/first_transaction_on_device.rs`、`tests/common/mod.rs` 四份），理由见第二节。
6. 我的改动让两份不在我文件里的测试二进制各多红一条（checker_known_bad_images 一条、random_history 一条，基线都绿），它们的历史就是「系统配置见证过的根全读不出、恢复照常挂上」，乙之后被拒；没改，交主 agent（第六节 Q3）。

推翻条件：
- 在副本上把 `mount.rs` 的读阶段改回「不判 N-配置、不重读」，第四节列的新测试不红 ⇒ 这些测试没钉住乙（第四节逐条证过会红）。
- 门禁 74 号在打上补丁的主工作区上快档仍报 `CheckerViolations { invariants: ["I-7.4"] }` 签名 ⇒ 结论 3 不成立。

## 二、这一轮写过的文件（副本 `work/` 里；补丁按这些生成）

| 文件 | 改了什么 | 在不在派发清单 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs` | 新成员 `MountError::NewerStateStillUnreadableAfterOneReread(Box<StillUnreadableAfterOneReread>)`；新类型 `StillUnreadableAfterOneReread`、`SelectedVersionAgainstTheWitness`、`NewerPublishWitness`、`WitnessedCounterComparison`、`BeforeTheOneReread`、`RereadsOfThisMount`、`ReadStageSettled`、`InstanceTableOfTheNewestRootRead`；私有 `ReadStageCache`（读缓存，`PoolReader` 包装）、`ReadStage`、`newer_publish_witness`、`read_stage`、`read_stage_settled_at_most_on_the_one_reread`、`instance_table_of_the_newest_root_read_at_most_twice`；`mount_writable_with_test_only_switches` 改成调新入口 `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread`（产品路径传 `RereadImmediately`）；`rebuilt_allocator` 多收钩子、按新函数取表；`MountOutput` 多一项 `rereads`；`floor_raise_refused_for_space` 补新成员的臂 | 在 |
| `crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` | 新测试 8 条（第四节） | 在（新文件） |
| `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs` | ① 判定读的序号常数 4 → 5（读阶段判 N-配置 每块盘多读一次槽 0）；② `the_first_file_version_after_a_recovery_dropped_an_earlier_one_…` 改成崩在实例 2 第一个文件版本的根槽 FUA 之后（录制流切到那一写），同名；③ 原 C554 用例拆成两条：`crash_recovery_that_cannot_read_the_witnessed_row_publish_…`（见证过的那一形：持续 ⇒ 拒，撤在重读之前 ⇒ 择 (2, 4)）与 `crash_recovery_abandoning_the_unwitnessed_row_publish_…`（崩在写行那次根槽 FUA 之后、系统配置没见证：照旧抛弃，保住影子账树表 0 条那一臂的判别力）；新辅助 `devices_crashed_right_after_the_root_slot_of` | 在 |
| `crates/singlefs-harness/tests/second_transaction_supplement_two_unreadable_abandoned_root_slot.rs` | 钉今天样子的那一条改名 `an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable`，加钉两条臂重开时都在第一遍判完（(3, 10)、jsn 10）；文件头注释改写 | 在 |
| `crates/singlefs-harness/src/history.rs` | `mount_error_member` 补新成员的臂（名字里带是哪一样） | 不在（规格点名「穷举 match 它的」） |
| `crates/singlefs-harness/src/bin/first_transaction_on_device.rs` | 两处穷举 match 补新成员的臂 | 不在（规格点名） |
| `crates/singlefs-harness/src/model_comparison.rs` | 三处穷举 match 补新成员的臂（`refusal_reason_of_mount_error` 归 `Unexplained`，另两处 `None`） | 不在：它也穷举 match `MountError`，不补编不过 |
| `crates/singlefs-harness/tests/common/mod.rs` | ① 帮手 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 多做一步：见证 `newest` 的那一槽系统配置（每块盘世代号最大那一槽）清零、不写回——乙之后见证在就拒可写、造不出被抛弃的根，7 个测试二进制靠它造被抛弃根；② 新增共用的读故障包装 `SharedUnreadableRanges` / `DeviceWithUnreadableRanges`（几段落点读报错或读回全 0、钩子里撤、逐段数读）与 `unreadable_root_slot_of` | 不在：不改它 7 个测试二进制当场 panic；包装两份测试文件都要用，放共用模块免得手抄 |
| `crates/mutations.tsv` | 末尾追加 14 行（第四节表）；整行替换 7 行（第四节） | —— |

`recovery.rs` 与 `lib.rs` 没动：读缓存是 `mount.rs` 里的私有 `PoolReader` 包装，`choose_root` / `scan_journal` / `replay_journal` 本来就收 `&dyn PoolReader`；新类型都在 `mount` 模块里、`pub`，不用在 `lib.rs` 另导出。

`crates/mutations.tsv` 追加的 14 行的变异名（原样）：
- C554 乙：读阶段第一遍判据 N-配置 为真也不重读、按第一遍往下走（系统配置见证过的最新根暂时读不出照旧被当成被抛弃）
- C554 乙（树表 0 条那一版上）：读阶段第一遍判据 N-配置 为真也不重读、按第一遍往下走
- C554 乙：重读那一遍判据为假也拒可写（重读读得出也不用）
- C554 乙（产品路径只坏一次的读）：重读那一遍判据为假也拒可写
- C554 乙：读阶段重读之前不调「重读一次」之前的钩子
- C554 乙：读缓存把读回全 0 的落点也收进去（重读那一遍还是全 0，E158 第 4 次跑登记 5.5 第 1 条）
- C554 乙：读缓存收了也不查（重读那一遍整遍打到盘上）
- C554 乙：c_见证 取系统配置槽 tail 的最小值（只剩较旧那一槽的见证）
- C554 乙：拿所选那一版的末条比时 c_见证 = c_E 也判真（系统配置没见证到的最新根也被拒）
- C554 乙：拿计数器等于 c_见证 的那条记录比时方向比反
- C554 乙：判不出（两条记录都读不出）按判据为假
- C554 乙：c_见证 = 0 不按没有见证判（只做过 mkfs 的池落到判不出、按真）
- 代码审阅第 22 条：重建分配器时最新那条根的实例表重读仍读不出，照改之前按空表往下走（无声关掉隔离）
- 代码审阅第 22 条：实例表重读之前不调「重读一次」之前的钩子

整行替换的 7 行（按名字）：`增补 2 收口第 27 行：影子账隔离只铺记录起点那一槽…`、`增补 2 收口第 27 行：只供测试的根槽点名判定恒不命中…`（必须红的测试名换成改名后的）；`实四乙（实三交回 Q3…`、`实四乙（同上一支）：树表 0 条的被抛弃根只认实例表与树表…`、`实审 B2b（formatted_pool 那段 C554 历史改判只红 I-7.4）…`（过滤串与测试名换成 `crash_recovery_abandoning_the_unwitnessed_row_publish_…`）；`C378（认了）…`（锚点里的 `InstanceStart` 多了 `rereads` 一项，原文与替换文同样补上）；`增补 2 第 9 行 P5（C499）…`（扫 journal 挪进了读阶段，锚点改成 `read_stage` 里经读缓存的那两行，替换文里的读者换成读缓存）。

## 六、停下交主 agent 的设计问题

**Q1 乙-配置罩不到「系统配置没见证到的最新根」，而这一形里有 fsync 已返回的发布（推的，没量）。** 判据只认系统配置槽里的 tail；发布的系统配置轮换是普通写，发布返回之前没有尾随屏障（代码审阅第 19 条原话「上一次发布的 RotateSystemConfigurationSlots 之后也没有尾随屏障」，`research/prompts/m2-code-review-6c/00-handover-message.md` 第 44 行），所以「根已 FUA、调用方拿到了返回、轮换还在缓存里」崩掉之后见证就没了；这之后重开时那条根暂时读不出，乙照旧把它当成被抛弃，那次挂载看不见它、把它的单元再发出去（池级 checker 的 I-7.4 照实红）。钉这一格的用例：新文件的 `a_newest_root_the_system_configuration_never_witnessed_is_abandoned_without_a_reread`（只钉「不重读、照旧抛弃」），formatted_pool 的 `crash_recovery_abandoning_the_unwitnessed_row_publish_…`（钉 I-7.4 照实红），`second_transaction_step_five_reuse` 的 `raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_ends_in_the_known_red_form_of_closeout_row_43` 基线红、乙之后照旧红（同一个 I-7.4，帮手改成没见证那一形）。补这一格的候选在登记里都有名字：N-配置续（只管两次取号）、N-槽（读回全 0 看不见）、轮换之后加屏障（动段序列，门禁 52 号）；要不要补、补哪个交主 agent / 用户。
**什么现象会推翻「有 fsync 已返回的发布」这一句**：发布路径在向调用方返回之前对系统配置轮换过了屏障。现查 `crates/singlefs-core/src/transaction.rs` 第 1040–1054 行 `persist_the_root_then_rotate_the_system_configuration`：根槽 FUA 写之后紧接 `CommitStep::RotateSystemConfigurationSlots`，函数到此返回，没有屏障；调用方之后有没有屏障我没逐条查，「fsync 返回时轮换可能还没持久」是推的，没在崩溃点重放里造过。

**Q2 随机历史的「崩溃恢复抛弃根」那一步乙之后一律被拒，模型还答「该成」。** 那一步把会话里最新那条根的根槽与它点名的单元在整次挂载里读成全 0（`history.rs` 的 `apply_crash_recovery_abandoning_the_newest_root`），而那条根是这个会话发的、系统配置见证过 ⇒ 乙重读仍全 0 ⇒ 拒。门禁 74 号快档因此 46 段新发现、签名 `ModelDisagreement { aspect: "模型说该成、实现拒了" }`；这一步再也造不出被抛弃的根，随机历史里影子账那一路就没人走了。三条路：① 改模型那一步的答案成「拒可写、盘上不变」（`model.rs` 的 `answer_mount_writable_with_the_newest_root_unreadable`）；② 那一步另把见证它的系统配置槽也藏掉（像 `tests/common` 的帮手那样），留住「造被抛弃根」，而 I-7.4 那一形（Q1）会跟着回来；③ 两者都要，拆成两种步。条款与模型归谁改由主 agent 定，我没动 `model.rs`。

**Q3 两份不在我文件里的测试被乙改了结局（基线绿、乙之后红），没改：**
- `checker_known_bad_images.rs` 的 `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`：把实例 2 的每一条根清零再可写挂载，要的是「落到实例 1、写中间实例行 (2, 0, 0)」。实例 2 的写行、暖机、第一个文件都被系统配置见证过 ⇒ 乙拒可写（原样：`NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: … (1, 2) … witnessed_journal_counter: 5 … selected_version_last_record_counter: 2 … })`）。「中间实例行后面还有实例 2 的码 2 节点在盘上」这个前提在乙之后造不出合法的一形（实例 2 发过第一个文件，它的写行与暖机就一定见证过），要换前提，是设计判断。
- `second_transaction_supplement_three_random_history.rs` 的 `rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline`：同 Q2，第 2 步「崩溃恢复抛弃根」被拒、模型对拍报新发现（原样见第三节）。同一个二进制里基线就红的 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与快档那一条照旧红，红法变了（第三节）。
- 我没跑、按静态搜可能受影响的：`second_transaction_supplement_three_fault_injection`（随机注入里抽得到一次性的读错，落在最新根槽上时乙会立即重读、读得出，结局与今天不同；门禁 59 号跑 `C378（认了）` 那一行时会跑它过滤出的那一条，证红时它的基线绿）、层 0 与崩溃注入各流（不跑，归提交时）。

**Q4 读缓存的代价。** 照第 4 次跑登记 5.5 第 1 条，读阶段第一遍就经缓存读，扫 journal 读成、非全零的每个记录槽都进缓存；环写满一圈之后是环长 × 盘数（默认 768 MiB 环、两块盘约 1.5 GiB，推的，没量），活到读阶段判完。要不要只在判出 N 为真之后才开缓存（第一遍不收，重读那一遍整遍打到盘上），或另立「乙-窄读」（主 agent 在第 4 次跑登记 5.5 第 1 条认定里立过这个名字），交主 agent；我照规格写的做了。

**Q5 判据自己会在一个单故障的合法状态上拒可写（照登记定义，没改）。** 所选那一版就是最新的、系统配置见证的就是它，而它那次发布的末条记录两份都读不出 ⇒ 第一支不适用、第二支找的「计数器等于 c_见证 的记录」就是那条读不出的 ⇒ 判不出 ⇒ 按真 ⇒ 重读仍读不出 ⇒ 拒可写。登记第 347 行原文就是「两条都读不出 ⇒ 判不出，按 N 为真处置」。只读挂载照常。用例没钉这一格。

**Q6 相邻的两处「读不出就无声放过」没动（不在第 22 条那一格里）：** `recovery::effective_rollback_floor` 自己再择一次根、读实例表，读不出就「不按表滤」（`recovery.rs` 第 1013 行文档注释原话「那张表读不出、解不开时不按表滤」）；挂着时抬 F 重算影子账用 `readable_roots`，根槽读不出的被抛弃根照旧不隔离、不计数（C554 前半段，第四节 supplement_two 那条钉着）。

## 七、条款：哪一句要改或补才说得上「乙」（交主 agent 派书记员；我不写 kb）

原文整行照抄（行号 2026-09-27 01:2xZ 在主工作区现取）。

`.claude/kb/decisions/23-journal的角色与格式.md` 第 400 行（已定项 14 射程）原文：

```
- ⚠️ **影子账与按实例表判抛弃（`abandoned_by_table`）留着，理由是崩溃恢复，不是管理员回退**：一段没有任何管理员回退的历史里，崩溃恢复因暂时读错落到旧根、实例表抛弃了较新的根，关掉影子账，被抛弃的根引用的数据单元被复用，之后恢复落在它上面读不出（第一轮 H6）。「只在崩溃恢复落到旧根的那次挂载里隔离」挡不住它：被复用的写发生在下一次干净的挂载里（第二轮辩方 G6，读代码的判断、没量）。落 [checks-owed.md](../checks-owed.md) C314（回退可以复用被抛弃的根引用的单元）。
```

改：「一段没有任何管理员回退的历史里，崩溃恢复因暂时读错落到旧根、实例表抛弃了较新的根」这一句之后补「——C554 乙（用户 2026-09-27 定）之后，系统配置见证过的较新的根暂时读不出不再被抛弃：可写挂载重读一次，仍读不出就拒可写；影子账留着的理由收窄成系统配置没见证到的那一形（那次发布的系统配置轮换没落盘）」。

补一段（放已定项 14「失败的处置」之前，或另立一条已定项，由书记员定位置）写乙本身，建议原句：「**可写挂载读到的更新状态读不出（C554 乙，用户 2026-09-27 定；判据 N-配置 照 E158 第 3 次跑登记第 347 行）**：读阶段（择根、扫 journal、重放）读完，取池里每块盘两槽里全部自证过的系统配置槽 journal tail 的最大值 c_见证；0 ⇒ 不判；所选那一版那次发布带末条标志的记录读得出（任一份）⇒ c_见证 > 它的计数器即为真；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) 大于所选那一版即为真；两条都读不出按真。为真就在这次挂载内有效的读缓存（只收读成且不是全零的落点）上把读阶段重做一遍（R = 1，取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」，两次读之间不等），重做那一遍为假就用它往下走，仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常。重建分配器判抛弃用的最新那条根的实例表读不出，同样重读一次、仍读不出拒可写（代码审阅第 22 条）。系统配置没见证到的最新根罩不到（C554）。」

第 401 行（已定项 14 射程「挂着时回退的已知边角三样」）原文：

```
- ⚠️ **挂着时回退的已知边角三样**：① 「R_old 仍分配的落点在 cur 的账里是同槽同分配代」这道核不在定案里，也补不上——已释放的记录记的是释放代（`crates/singlefs-core/src/allocator.rs:38`），核看不出「这个槽被复用过又释放了」；它放过的读不对的回退只在 F 回落时走得到（第三轮判决第二节 K1：F 生效取最大值的最小值时 15 格中 12 格，取 SysPre 时 0），靠 F 不回落挡。② 崩溃恢复抛弃的根在重开时全部暂时读不出，影子账读不到它们的账、隔离 0，之后别的实例复用了它们的槽，撤故障再把新实例的根全改坏，恢复落在被抛弃的根上读不出——与回退形态无关的多重故障，见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）。③ 两个状态来回回退超过根环一圈时，可退到的不同状态少于 4 个（D16（发布语义） 已定项 1 的根环容量边界）。
```

改：② 那一句末尾「见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）」之前补「C554 乙之后这一形只剩被抛弃根是系统配置没见证到的那一种（见证过的在崩溃恢复那次挂载里重读一次仍读不出就拒可写，抛弃不了）」。

第 486 行（已定项 18「语义」）原文：

```
- **语义**：这个数的含义是**一次恢复重放的前缀最多 N 条**。读到一条校验和不过的记录时，恢复**断链即止**——停在它之前那一条，不追问它是「撕裂」还是「损坏」，也不因此拒绝挂载。不是 XFS 那种「最后 N 条内的坏校验和算撕裂、之外算损坏拒绝挂载」。
```

改：「也不因此拒绝挂载」改成「也不因此拒绝只读挂载；可写挂载另按 C554 乙判（已定项 14）：系统配置见证过比断链处那一版新的发布、重读一次仍判真，拒可写」（E158 第 3 次跑登记 5.5 第 5 项主 agent 认定原话「D23 已定项 18「不因此拒绝挂载」那一句要改成「只读挂载照常、可写挂载按那条改法拒」」）。

已定项 15（由记录重建那次发布的根）：乙不碰它，不用改。

`.claude/kb/decisions/16-发布语义.md` 第 37 行（已定项 1 表「根槽这一次读坏」那一行）原文：

```
| 根槽这一次读坏 | 算准入抬 F 的上限时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根；挂载那一刻读得出、或这个进程写过且 FUA 返回过的槽，这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F、报它自己的错误成员，不按有根或没根猜（用户 2026-09-26 定，推的，被攻过零轮）。读根环的别的调用方（择根、回退候选集、「环里最旧有效根」）怎么处置这一格没判，见 C563（读根环一槽读坏就当没有根，别的调用方没判） |
```

改：「读根环的别的调用方（择根、回退候选集、「环里最旧有效根」）怎么处置这一格没判」改成「可写挂载的择根按 C554 乙处置（D23（journal 的角色与格式） 已定项 14：系统配置见证过比所选那一版新的发布就把读阶段重读一次，仍判真拒可写）；只读挂载的择根、回退候选集、「环里最旧有效根」怎么处置这一格没判」。

另外要跟着改的 kb 引用（旧测试名，不改会指向不存在的用例）：
- `.claude/kb/checks-owed.md` 第 475 行（C554 行）引 `second_transaction_supplement_two_unreadable_abandoned_root_slot.rs` 的 `an_abandoned_root_whose_own_root_slot_is_unreadable_is_neither_isolated_nor_counted`，新名 `an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable`；这一行的状态也要改：乙已实现、代码审阅第 22 条那一格已罩住、剩系统配置没见证的那一形（第六节 Q1）。
- `.claude/kb/milestone/02-second-txn.md` 第 361 行（收口表第 27 行）引同一个旧名。
- formatted_pool 的旧名 `crash_recovery_abandoning_the_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them` 在 kb 里有没有引用，我只按上面那条 grep 查了两份 kb 文件的命中（`grep -rln` 那一次命中列表里 kb 只有 checks-owed.md 与 milestone/02-second-txn.md 两份），书记员改之前再全仓搜一遍。

## 三、门禁 74 号那两条与最短复现（副本上照 74 号跑法；74 号不是重型）

命令（在副本根，今天 = `repo/` 未改动、乙 = `work/`；各用自己的 target）：
`nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash /tmp/claude-1000/impl-c554-yi/random-history.sh`，里面逐个跑
`cargo test --release --offline -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture`（经 `capped.sh 5`）与
`SINGLEFS_RANDOM_HISTORY_SHRINK_SEED=7463871032432355115 cargo test --release … -- --exact shrink_one_failing_seed_from_the_environment --ignored --nocapture`。日志 `logs/rh-repo.log`、`logs/rh-work.log`、`logs/shrink-repo.log`、`logs/shrink-work.log`。

快档那一行（原样）：

```
今天： 历史 96 段：跑完 67、以已知红收尾 {0: 1}、新发现 28；根环转过一圈的 62 段；最高 txg 44；一版里最多 746 条分配记录
今天： 新发现 CheckerViolations { invariants: ["I-7.4"] }：第一个种子 7463871032432355115（…28 个种子…）
乙：   历史 96 段：跑完 50、以已知红收尾 {}、新发现 46；根环转过一圈的 43 段；最高 txg 44；一版里最多 746 条分配记录
乙：   新发现 ModelDisagreement { aspect: "模型说该成、实现拒了" }：第一个种子 7463871032432355114（同签名的种子 […]
```

- I-7.4 签名的新发现：28 → 0（乙这一侧日志里 `CheckerViolations` 零命中，`grep -c 'CheckerViolations' logs/rh-work.log` 见第五节）。
- 换成 46 段 `ModelDisagreement`「模型说该成、实现拒了」，全在「崩溃恢复抛弃根」那一步（第六节 Q2）。
- 其余四个取样点两边都是 0 新发现（日志里「历史 48 段 … 新发现 0」「历史 32 段 … 新发现 0」各行两边相同）。

收口表第 43 行那条用例 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`：今天红在第 3 步 I-7.4；乙之后红在第 2 步，原样：
`崩溃恢复抛弃根那一步是一次做成的可写挂载、取号 2：Refused { member: "MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)" }`

最短复现（种子 …115 收缩，3 步）：今天第 2 步 `CrashRecoveryAbandoningTheNewestRoot → Applied(Mounted { instance: InstanceGeneration(2), … })`（这一段在今天判 I-7.4 红）；乙之后原样：
`2. CrashRecoveryAbandoningTheNewestRoot → Refused { member: "MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)" }`，签名 `ModelDisagreement { aspect: "模型说该成、实现拒了" }`，在 Operation(3) 之后。

随机历史这个二进制两边的测试结局：今天 `22 passed; 2 failed; 2 ignored`（快档、第 43 行那条）；乙 `21 passed; 3 failed; 2 ignored`，多红 `rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline`（第 2 步同样被拒，第六节 Q3）。

规格验收第 1 条「在今天的代码上造出会红的状态」：上面「今天」三样都是 `repo/`（取副本那一刻主工作区的原样）上跑出来的；formatted_pool 那条 C554 用例今天绿（它钉的就是「只红 I-7.4」），supplement_two 那条今天绿（钉着「隔离 0、计数 0」），都在 `logs/base-*.log`：
```
== logs/base-second_transaction_step_three_formatted_pool.log
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.67s
== logs/base-second_transaction_supplement_two_unreadable_abandoned_root_slot.log
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.83s
```

## 四、每条新测试与改过的测试：改坏哪一行 → 哪条断言红

证红一律 `bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-c554-yi/prove singlefs-harness <变异名…>`（`prove/` 是 `work/` 的另一份副本、自己的 target；经 `capped.sh 5`；日志 `logs/prove-red/NNN.log`，汇总 `logs/prove-all.out`）。31 条：30 抓到，1 没红（见表末）。每组参数先跑了基线（不改源码）且绿，这是 prove-red 自己的第 ④ 步。
被测代码里没有 `debug_assert`，debug 下红的都是测试自己的断言（或测试里的 `expect` 看到了错误成员）。同一份日志里别的测试一起红的不记：每条变异都按过滤串只跑点名的那一条（或那一个二进制里 1–3 条）。

| 变异（新增或换行） | 改坏哪一行（`mount.rs` 里，除非另写） | 红的测试 → 断言（日志行） |
|---|---|---|
| C554 乙：读阶段第一遍判据为真也不重读 | `if !first_read.witnesses_a_publish_newer_than_the_selected_version() {` 前加 `true \|\|` | 新文件 `a_witnessed_newest_root_still_unreadable_after_the_one_reread_…` → 第 196 行「两遍都落到 B、系统配置见证到 C：None」（挂载照常做成、没拒）（018） |
| 同上（树表 0 条那一版上） | 同一行 | formatted_pool `crash_recovery_that_cannot_read_the_witnessed_row_publish_…` → 第 1688 行「M3：两遍都落到 (1, 2)、系统配置见证到 jsn 4：None」（019） |
| C554 乙：重读那一遍判据为假也拒 | `if reread_against_the_witness.witnesses_…() {` 前加 `true \|\|` | `a_witnessed_newest_root_that_reads_back_an_error_until_just_before_the_one_reread_…` → 第 278 行 `expect("重读读得出：照常可写挂载")` 拿到拒绝成员（020） |
| 同上（产品路径只坏一次） | 同一行 | `a_witnessed_newest_root_whose_reads_fail_only_once_…` → 第 378 行 `expect("立即重读读得出")`（021） |
| C554 乙：读阶段重读之前不调钩子 | 删 `before_the_one_reread.before_rereading();`（读阶段那一处） | `…reads_back_an_error_until_just_before_the_one_reread…` → 第 278 行（故障没撤，重读仍拒）（022） |
| C554 乙：读缓存把全 0 也收进去 | `if bytes.iter().any(\|byte\| *byte != 0) {` → `if !bytes.is_empty() {` | `…reads_back_zeros_until_just_before_the_one_reread…` → 第 278 行（重读拿到缓存里的全 0，仍拒）（023） |
| C554 乙：读缓存收了也不查 | `.get(&placement)` 后加 `.filter(\|_\| false)` | `a_witnessed_newest_root_still_unreadable_…` → 第 214 行「B 的根槽第一遍读成、进了读缓存：重读那一遍不再打到盘上」（024） |
| C554 乙：c_见证 取最小值 | `.map(\|slot\| slot.quantities.journal_tail)\n.max()` → `.min()` | `a_witnessed_newest_root_still_unreadable_…` → 第 196 行（只剩较旧槽的见证 = B，判假，挂载做成）（025） |
| C554 乙：c_见证 = c_E 也判真 | `> selected_version_last_record_counter` → `>=` | `a_newest_root_the_system_configuration_never_witnessed_…` → `tests/common/mod.rs` 第 512 行帮手里的 `expect("…照常可写挂载")` 拿到拒绝成员（026） |
| C554 乙：记录比较方向比反 | `(record.instance, record.checkpoint_txg)\n    > (` → `<= (` | `without_the_last_record_of_the_selected_version_…` → 第 469 行「c554-yi-record-at-the-witnessed-counter：None」（027） |
| C554 乙：判不出按假 | `WitnessedCounterComparison::Undecidable => true,` → `false` | 同一条 → 第 469 行「c554-yi-undecidable：None」（028） |
| C554 乙：c_见证 = 0 不按没见证判 | `let comparison = if witnessed_journal_counter == 0 {` → `if false {` | `the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_…` → 第 495 行 `expect("第一次可写挂载")` 拿到拒绝成员（mkfs 池落到判不出、按真）（029） |
| 第 22 条：重读仍读不出按空表往下走 | `instance_table_of_the_newest_root_read_at_most_twice(…)?` → `.unwrap_or((空表, OnTheOneReread))` | `a_newest_instance_table_unreadable_when_the_shadow_ledger_reads_it_…` → 第 608 行 `matches!(… InstanceTableOfTheNewestRootForTheShadowLedger …)`，实际 None（挂载做成）（030） |
| 第 22 条：实例表重读之前不调钩子 | 删 `before_the_one_reread.before_rereading();`（重建分配器那一处） | 同一条 → 第 656 行 `expect("重读读得出：照常可写挂载")` 拿到 `InstanceTableOfTheNewestRootForTheShadowLedger`（031） |
| 实四乙 Q3（换行：测试改名） | 旧变异原样（树表 0 条那一臂交回 None） | formatted_pool `crash_recovery_abandoning_the_unwitnessed_row_publish_…` → 第 1956 行「被抛弃的那条根认得出它引用的落点」（015） |
| 实四乙（同上一支）（换行） | 旧变异原样 | 同一条 → 第 1979 行「盘 0：槽 SlotNumber(50246) 仍被环里的被抛弃根引用，回收之后隔离着」（016） |
| 实审 B2b 只红 I-7.4（换行） | 旧变异原样（checker） | 同一条 → 第 2049 行「卸载之后：只该红 I-7.4：[]」（017） |
| 增补 2 第 27 行两条（换行：测试改名） | 旧变异原样 | supplement_two 改名那一条 → 第 359 行「重开那一刻影子账两块盘各罩住 14 个槽」一类（002、003：003 那条红在第 271 行「点名的槽上真的注入过：0 次」） |
| C378（换行：锚点多一项） | 旧变异原样，锚点补 `rereads` | fault_injection `write_error_after_the_acquisition_…` → 第 792 行（004） |
| C499（换行：锚点挪进读阶段） | 旧变异改写：读阶段里只取 tail 那一条、不扫全环 | presumed_clause_checks `c499_remount_after_a_clean_close_…` → 第 466 行「干净关闭再挂载：盘 0 上只读了 360 次，不够扫一遍整环（196608 个记录槽）」（008） |

没换行、点名的用例我改过或靠改过的帮手的旧行也复证了：`步 3：取号不核判定时算出的号`（001，formatted_pool 常数 4 → 5 之后仍红在 `mount.rs:3326` 的断言）、D8 已定项 8 ② 两条与 `publish_first_file 退回按水位判`、`只读挂载认中央映射树…写死 15`（005、006、007、014，改过的 `the_first_file_version_after_a_recovery_dropped_…`）、C503 两条、C518、第 27 行 ④（009、011、012、013）。

**没红的一条**：`C503（隔离位清零的时机条文与实现说反话）：清隔离位时不看环里别的被抛弃根还引用不引用（多清）`（010）。在 `repo/`（今天的代码、没改动）上同样没红（`logs/prove-red-base/`，命令原样 `bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-c554-yi/repo singlefs-harness "C503（…）：清隔离位时不看环里别的被抛弃根还引用不引用（多清）"`，输出「✗ … 没红」）——不是这一轮带进来的（主 agent 01:3xZ 来消息：那一行是 A2c 换过锚点的既有变异），在我的副本与今天的代码上都没红，没改它，留给门禁 59 号。

留给门禁 59 号的：追加的 14 行每行都证过（上表），没有「只追加、没证」的行。

规格验收第 2 条三件对上的用例：暂时读错、重读成功 ⇒ 择对根、I-7.4 不红（新文件两条「until_just_before_the_one_reread」+ 产品路径那条，formatted_pool 被见证那条的撤故障一臂）；重读仍读不出 ⇒ 拒可写、只读照常（新文件 `…still_unreadable_after_the_one_reread…`、判据另两支那条、formatted_pool 被见证那条的持续一臂）；第 22 条那一格不再无声关（新文件 `a_newest_instance_table_unreadable_…`）。

## 五、交回前的验证（末尾原样；全在副本 `work/` 上，除非另写）

负载：开跑前 `ps` 看到别的会话的 `cargo build --release --bin e158_root_choice_repair` 与 `cargo test --release … e158_root_choice_repair`（E158 执行员），后来还有 `/tmp/claude-1000/impl-rev-a4d/copy` 上的随机历史；没有 qemu / fio / 性能测量。我用自己副本的 target，没等锁。内存包装上限一律 8G（派发没给，取 `replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值）；线程经 `capped.sh 5`。

**测试二进制（debug，整个二进制；`logs/final-*.log`；基线 `logs/base-*.log` 是 `repo/` 上同一条）**：

```
a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable exit 0   test result: ok. 8 passed; 0 failed
second_transaction_step_three_formatted_pool exit 0                              test result: ok. 14 passed; 0 failed   （基线 13 passed）
second_transaction_supplement_two_unreadable_abandoned_root_slot exit 0          test result: ok. 3 passed; 0 failed
checker_known_bad_images exit 101                                                test result: FAILED. 38 passed; 1 failed   （基线 39 passed；红的是 published_nodes_behind_an_intermediate_row_…，第六节 Q3）
checker_narrow_invariants_and_abandoned_roots exit 0                             test result: ok. 19 passed; 0 failed
second_transaction_step_five_reuse exit 101                                      test result: FAILED. 13 passed; 1 failed   （基线同样 13 / 1，红的是同一条 closeout_row_43）
second_transaction_step_four_rollback exit 0                                     test result: ok. 15 passed; 0 failed
second_transaction_supplement_two_admission_formula exit 0                       test result: ok. 3 passed; 0 failed
second_transaction_supplement_three_fault_injection exit 101                     test result: FAILED. 12 passed; 1 failed; 1 ignored   （基线同样 12 / 1：快档新发现 今天 38、乙 30）
second_transaction_supplement_two_presumed_clause_checks exit 0                  test result: ok. 3 passed; 0 failed
core-lib exit 0                                                                  test result: ok. 127 passed; 0 failed
second_transaction_supplement_three_random_history（release，74 号跑法）          今天 22 passed; 2 failed；乙 21 passed; 3 failed（第三节）
```

fault_injection 快档两边都红：今天那一份清单外的签名是 `CheckerViolations { invariants: ["I-7.4"] }`（与门禁 74 号同一形），乙之后是 `ModelDisagreement { aspect: "模型说该成、实现拒了" }`（同第六节 Q2），测量跑提前停的段数 0 → 13。都是 `CrashRecoveryAbandoningTheNewestRoot` 那一步引起的，我没改这个二进制。

**`cargo fmt --check`**：退 1，67 处 `Diff in …/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`，别的文件 0 处（`grep '^Diff in' logs/final-fmt.log | sed 's/:[0-9]*:$//' | sort | uniq -c` 原样 `67 Diff in /tmp/claude-1000/impl-c554-yi/work/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`）。那个 bin 是 E158 执行员在改的，副本里是取副本那一刻的样子，不是我的；我动过的 8 份 `rustfmt --edition 2021 --check` 退 0。

**`cargo clippy --offline --keep-going --all-targets --all-features -- -D warnings` 加 check.sh 那七条 lint**：退 101，红的只在三处，今天的代码（`repo/`）上同一条命令红在同样三处（`logs/base-clippy-all.log`）：
```
work/：
      1 error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts") due to 3 previous errors
      1 error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts" test) due to 3 previous errors
      1 error: could not compile `singlefs-harness` (bin "e158_root_choice_repair") due to 3 previous errors
      1 error: could not compile `singlefs-harness` (bin "e158_root_choice_repair" test) due to 5 previous errors
      1 error: could not compile `singlefs-harness` (test "checker_narrow_invariants_and_abandoned_roots") due to 2 previous errors
repo/（今天）：同样这五行，逐字相同
```
`-p singlefs-format -p singlefs-core -p singlefs-checker --all-targets` 那一次退 0（`Finished`）。harness 里我动过的 lib、`first_transaction_on_device` bin 与三份测试文件、`tests/common` 都不在告警里。

**`cargo build --offline --all-targets`**：退 0，末行 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 35.27s`。

**登记给我的门禁阶段**（在副本上跑：`cd work && bash .claude/gate.d/<阶段> /tmp/claude-1000/impl-c554-yi/work`；日志 `logs/gate-*.log`）：
```
33-mutation-tables.sh exit 0   ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1135 条的原文各命中源码一次；…
53-format-const-placeholders.sh exit 0   ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）
92-layout-checker-sync.sh exit 77   ! /tmp/claude-1000/impl-c554-yi/work 不是 git 仓，本阶段跳过   （本次未跑）
94-checker-implementation-disjoint.sh exit 0   ✓ checker 与实现只共享常量模块 `singlefs-format`（…checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）…
93-feature-bits.sh exit 0   ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，…）
89-closeout-row27-preconditions.sh exit 77   ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）；主工作区上同样退 77
74-model-differential.sh exit 1   ✗ 随机历史的测试二进制判红（签名是 ModelDisagreement，第三节；`repo/`——取副本那一刻的主工作区——上照 74 号跑法同样红，签名是 I-7.4，`logs/rh-repo.log`）
```
补丁对主工作区的核（2026-09-27 01:3xZ，主工作区那一刻的样子）：`git apply --check --verbose /tmp/claude-1000/impl-c554-yi/patch/crates.patch` 八份逐个 `Checking patch …`、退 0；`python3 research/scripts/apply-writer-patch.py /tmp/claude-1000/impl-c554-yi/patch --dry-run` 原样 `✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1151 行`、退 0。

`git diff --stat -- crates litmus`：我没改主工作区，这条命令在主工作区上报的是别的会话的改动，不代表这一件；补丁的 stat 见下（`git apply --stat` 原样）。
     crates/singlefs-core/src/mount.rs                  |  440 +++++++++++++
     .../src/bin/first_transaction_on_device.rs         |    8 
     crates/singlefs-harness/src/history.rs             |   14 
     crates/singlefs-harness/src/model_comparison.rs    |   12 
     crates/singlefs-harness/tests/common/mod.rs        |  303 +++++++++
     ...second_transaction_step_three_formatted_pool.rs |  504 +++++++++++----
     ...upplement_two_unreadable_abandoned_root_slot.rs |   44 +
     ...ewer_root_rereads_once_then_refuses_writable.rs |  670 ++++++++++++++++++++
     8 files changed, 1838 insertions(+), 157 deletions(-)

## 八、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交；主工作区一个字没改（交补丁）。
- 重型测试一条没跑：名字带 layer0 的测试二进制、崩溃注入快档（`second_transaction_supplement_three_crash_injection` 那几条）、全量 `cargo test`。我的改动改了每一次可写挂载的读法，其余 60 多个 `mod common` 的测试二进制只编过（`cargo build --all-targets` 绿）、没跑；按静态搜「挂载之前藏根槽」的只有第五节跑过的那几份，崩溃注入与层 0 里「根已落盘、系统配置见证、之后读不出」的崩溃状态会不会被乙拒、钉值会不会变，没验，留给提交时的层 0 与崩溃注入。
- 补丁不动 `crates/singlefs-checker/src/`，没有「受影响的层 0 流与崩溃枚举用例」那一节的对象；但 formatted_pool 那条改过的用例历史变了（崩在根槽 FUA 之后、少一次暖机），它不是层 0 流。
- 没加新层 0 流、没加崩溃点重放用例，`crash-case-check` 那一条无对象。
- 没改 `model.rs`、`history.rs` 的执行器、`checker_known_bad_images.rs`、随机历史测试文件（第六节 Q2、Q3）；没改 kb（第七节交书记员）。
- 门禁 92 号在副本上退 77（副本不是 git 仓），没判；89 号退 77（无对象）。74 号红，原因见第三节。
- 读缓存的内存代价、判据在「所选那一版末条两份都读不出」时的多拒，都没量（第六节 Q4、Q5）。

## 九、草稿与副本

- 补丁目录 `/tmp/claude-1000/impl-c554-yi/patch/`：`crates.patch`、`mutations-append.tsv`（14 行）、`mutations-replacements.tsv`（7 行）、`report.md`（本报告的拷贝）。
- 日志 `/tmp/claude-1000/impl-c554-yi/logs/`（基线、每批测试、证红 `prove-red/`、`prove-red-base/`、随机历史、clippy、门禁），进度 `progress.md`，辅助脚本 `baseline.sh`、`run-work.sh`、`random-history.sh`、`prove-all.sh`、`final.sh`、`final2.sh`，快照 `base/`（只有 `crates/` 源码与 sha，没有编译目录）：都留着，主 agent 核复跑用；没入库，因为派发要的是补丁，产物由主 agent 定去留。
- 删掉的仓副本与编译目录（交回之前，`du -sh` 量的）：`/tmp/claude-1000/impl-c554-yi/prove`（2.8G，含 target 2.6G）、`/tmp/claude-1000/impl-c554-yi/repo`（4.3G，含 target 4.1G）、`/tmp/claude-1000/impl-c554-yi/work`（16G，含 target 16G）。改动全在 `patch/crates.patch` 与两份变异 tsv 里；`base/crates/` 是取副本那一刻的源码（7.6M，没有编译目录），留着对补丁用。
