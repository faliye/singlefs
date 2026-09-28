# 实审 B3b 报告（implementation-writer）：崩溃注入补可写挂载与二次崩溃（审阅第 2 条）、对拍双向与每一版比内容和实例表（第 12 条）、c561 用例打 exhaustive 与线程行

## 一、写过的文件

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-harness/src/crash_injection.rs` | 第 2 条：每个崩溃状态判三截（`CrashStateStage`）。第一截（崩溃镜像本身）照旧；第二截在同一份崩溃后镜像的副本上起 `mount_writable_with_space_admission`（取号、写行、暖机），挂载交回带文件的一版就 `publish_overwrite`、树表 0 条就 `publish_first_file`，之后跑池级 checker；第三截在挂载那一段录制流上摆二次崩溃，取号 / 写行 / 暖机（`WritableMountPhase`）各一个，每个上只读恢复、问模型、池级 checker、记录核对器。计数进 `CrashInjectionTally`（`absorb` 按字段拆开写全，`render` 多三行）；新发现带 `stage`，按 (截, 签名) 去重；判红的镜像目录名带截名。模块头与 `inject_crashes_into_history` 的文档注释跟着改 |
| `crates/singlefs-harness/src/model.rs` | 第 12 条：`ObservedRoot` 的 `has_file` 换成三个字段 `file: ObservedFile`、`instance_table: ObservedInstanceTable`、`unit_allocation_records: ObservedUnitAllocationRecords`；`judge_roots` 每一版都比文件内容（`judge_file_content`，新格 `FileContent`）与整张实例表（`judge_instance_table`，新格 `InstanceTableOfTheVersion`）；`judge_allocation_generations` 两个方向都比（模型这一版有、实现没交回的角色也报），树表 0 条的一版比「这次重写了哪几个角色」两个方向；`ModelJudgementCounts` 加四个计数，`add` 按字段拆开写全；`ModelDisagreement::new` 改成 `pub`（崩溃注入第二截报拒绝要用）；四条新单测，旧单测 `carried_unit_keeps_the_generation_of_the_publish_that_wrote_it` 改成交回写行那一版的九个角色 |
| `crates/singlefs-harness/src/model_comparison.rs` | 胶水：从 `TransactionOutput.units` 的数据单元解出内容（`parse_data_unit` + `data_unit_payload`），从实例表各片按链解出整张表（`InstanceTablePage::parse`），`units` 里没有实例表时按根记录里那条实例表指针交回它的分配记录；树表 0 条的一版交回 `rewritten` 换成的角色集合；一条新单测跑四段带挂载的短历史 |
| `crates/singlefs-harness/src/fault_injection.rs` | 只在 `left_by_the_lying_device` 那个穷举 `match` 的 `=> false` 臂里补两个新格名（`FileContent`、`InstanceTableOfTheVersion`）。这份不在派发给的文件单里：`ModelDisagreementAspect` 加成员不补这一处编译不过，改动只有这两行 |
| `crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs` | c561：σ 全量改成按掩码区间切片（max(64, 16 × 线程数) 片）、工作线程领片、调用线程逐片打 `C561_SIGMA_PROGRESS`、按片号并；打 `C561_SIGMA_FULL states=… closed_form_states=… exhaustive=… record_claimed_state_missing_unit=…`（评过的状态数等于 `closed_form_state_count(&[σ])` 时 `exhaustive=true`）与一行 `LAYER0_PARALLEL_FINISHED` 同形的线程行；加一条不标 ignore 的快用例拿 σ 前面一段 2 写的小段核两行的打法 |
| `crates/singlefs-harness/tests/crash_injection_writable_mount_after_the_crash.rs`（新建） | 一条用例：两段短历史各两个崩溃状态，钉第二、三截每一样都跑到、三段都摆到、挂载与发布一次都没被拒、一条新发现都没有 |
| `crates/mutations.tsv` | 没改（派发提示定的）。13 行变异写在 `/tmp/claude-1000/impl-rev-b3b/mutations-append.tsv`，同列同格式，主 agent 追加 |

我这几份相对开工时的快照（`/tmp/claude-1000/impl-rev-b3b/baseline/`）的增删行数（`git diff --no-index --numstat`，这一次跑出来的）：

```
770 增 41 删 crates/singlefs-harness/src/crash_injection.rs
460 增 26 删 crates/singlefs-harness/src/model.rs
193 增 19 删 crates/singlefs-harness/src/model_comparison.rs
2 增 0 删 crates/singlefs-harness/src/fault_injection.rs
257 增 44 删 crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs
109 crates/singlefs-harness/tests/crash_injection_writable_mount_after_the_crash.rs（新文件，wc -l）
```

`crash_injection.rs` 里有一次改动（二次崩溃那一截把 `read_back` 改名成 `read_back_after_the_second_crash`，避开 `crates/mutations.tsv` 第 152 行的锚点被我的新代码撞成两次命中）是用 python 写临时文件再 `mv` 换上的，没走 Edit；写范围闸看不见那一次。其余手改都走的 Edit。

## 二、改法落到代码上的几处取舍（主 agent 定的改法之外我定的、或定不了的）

1. **文件内容带整份，不带摘要**：规格写「文件内容摘要」。模型模块只许 `std` 与 `singlefs_format`（门禁判的是 `model.rs` 的 use 行，D13（验证路线） 已定项 5），没有与胶水共用的哈希；一版至多一个数据单元（≤ 32634 字节），整份比逐字节，比摘要严。
2. **比不了的格照计数，不判**（`ObservedInstanceTable::NotInTheOutput`、`ObservedUnitAllocationRecords::RewrittenRolesOnly`）：
   - 树表 0 条的一版（`VersionWithoutFilePublishOutput`）输出里只有根记录、记录与 `rewritten`，没有分配记录、没有实例表单元的字节。这一版只比「这次重写了哪几个角色」（两个方向），实例表与分配代都比不了。要比得让输出带上这两样（`crates/singlefs-core/src/transaction.rs`，A1 在改），或者在 `history.rs` 里从挂着的分配器与盘上读（不在我的文件单里）。**交主 agent。**
   - 带文件的一版在 mkfs 之后第一次重写实例表之前（`units` 里没有实例表），与从盘上重建、实例表多于一片的一版（`recovery.rs` 的重建只装第 0 片），实例表同样比不了、照计数。分配记录那一半不缺：`units` 里没有实例表时按根记录里那条实例表指针的两条位置条目找记录。
3. **崩溃后的可写挂载或发布被单元区墙拒（成员映射成 `UnitAreaWall`）只计数、不判**：模型在容量墙上只答「允许拒绝的区间」，区间要崩溃点上的模型状态才算得出来，崩溃注入这一路没有重建它。4 GiB 的盘上走不到（快档、新用例两块 4 GiB）；大档配小盘宽时这一格没判。别的理由被拒按 `RefusedWhenModelRequiresSuccess` 报（模型的 `answer_mount_writable` 从不要求拒）。
4. **二次崩溃状态上的记录核对器只核挂载自己写出的那几次发布**（写表只给挂载那一段）：把历史写表接在前面时，第一次崩溃截在一次发布中间，那次发布没落根的单元写与记录写会被 `crash::publishes_in` 归进写行那次发布（按根槽写分），核出假红；把那几条剪掉又会丢掉它们对更早单元扇区的「合法复用」解释。要核历史那几次发布在二次崩溃镜像上还在不在，`publishes_in` 得认「一段流从这里断开」（`crash.rs`，B3a 在改）。**交主 agent。**
5. **第二截只跑 checker，没接记录核对器**：D13（验证路线） 已定项 7 说「比对的对象是实现恢复后的镜像」；`check_records_against` 只收一个读盘口子，没有「崩溃态镜像判择根与前缀、实现恢复后的镜像判在不在」两份入参（`crash.rs`，B3a 在改）。规格只要 checker，照规格做。
6. **二次崩溃状态上不再起可写挂载**（不递归到第三层）；每个第一次崩溃状态摆三个二次崩溃（取号、写行、暖机各一，按种子挑那一段里的一个写 w：w 所在段里 w 之前都落、w 不落、w 之后各抽）。「至少各一处」是整批的下限；这样摆每个崩溃状态三段都有。
7. **二次崩溃上问模型时，允许的版本 = 模型提交过的每一版 + 这次挂载写出的根**，挂载写出的根的内容取第一次崩溃之后只读恢复读回的那一版（写行与暖机照抄那一版的文件；那一次恢复失败时一条都不加，落到挂载的根上就判「模型没提交过」）。挂载的有效根与只读恢复的有效根要是分了叉，这里会红。
8. **c561 没改走 `enumerate_layer0_in_state_slices`**：那一路每个状态多跑一遍不看 journal 的恢复与整份池级 checker、要一张版本表给 oracle，状态集合也差一个（它有「全部持久」那一个、没有「σ 全落、之后不落」那一个），等于换了这条全量测的东西、全量的挂钟也要重量。照规格第二选择自己打：计数行带 `closed_form_states=` 与 `exhaustive=`，线程行字段、次序与层 0 那一行相同（`resumed_slices=0`、`progress_file_after_completion=none`，不留进度文件）。`worker_threads_source=` 的名字在用例里照 `crash.rs` 的 `Layer0WorkerThreadsSource::name` 抄了一份穷举 `match`：那个方法不是 `pub`（`crash.rs`，B3a 在改），改成 `pub` 之后这份可以删。
9. `fault_injection.rs` 那两行见第一节表格。

## 三、新用例与证红（每条先看它红，再改）

先红：第 12 条的四条模型单测先写、跑在「只做形状适配、还没加新比较」的那一版上，4 条都红（`/tmp/claude-1000/impl-rev-b3b/red-model.log`：`test result: FAILED. 11 passed; 4 failed`，红在 `expect_err` 与三处计数 `left: 0 right: 1`），加上比较之后 15 条绿（`green-model.log`）。第 2 条与 c561 的新用例用到的计数字段、函数改前不存在（编不过）；它们会不会红，靠下面的变异证。

变异证红：在仓副本（`rsync -a --exclude target --exclude .git`，自己的 target）上跑，**release**，每条跑那条用例所在的整个测试二进制；先跑不改动的副本：`--lib` 87 绿、新用例二进制 1 绿、c561 二进制 6 绿 1 ignored，**基线红集为空**。13 行全部证过（没有留给 59 号的行），行在 `/tmp/claude-1000/impl-rev-b3b/mutations-append.tsv`（与 `crates/mutations.tsv` 同六段；原文在现在的源码里各命中一次，脚本核过 `checked=13 bad=0`）。

| # | 改坏哪一行 | 哪条断言红（文件:行是副本里那一份，与工作区逐字相同） | 同一个二进制里同时红的 |
|---|---|---|---|
| 1 | `model.rs` `.find(|role| !handed_in.contains(role))` → `.find(|_role| false)`（模型这一版有、实现没交的角色不报） | `a_role_of_the_version_that_the_implementation_did_not_hand_in_is_reported`：`model.rs:2685` `expect_err("inode 根这一版有，实现没交")` | 无 |
| 2 | `model.rs` `if rewritten_in_the_model == *rewritten {` → `.is_superset(rewritten)` | `a_version_without_file_compares_the_rewritten_roles_both_ways`：`model.rs:2744` `expect_err("重写的角色少一个或多一个")` | 无 |
| 3 | `model.rs` `if file.content.as_ref() == content.as_slice() {` → 只比长度 | `each_version_compares_its_file_content`：`model.rs:2781` `expect_err("内容不是 [3]")` | 无 |
| 4 | `model.rs` `if expected.instance_table_rows.as_slice() == rows.as_slice() {` → `len() >= len()` | `each_version_compares_its_instance_table`：`model.rs:2813` `expect_err("实例表丢了那一行")` | 无 |
| 5 | `model_comparison.rs` 解出的内容倒过来 | `every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways`：`model_comparison.rs:752`，种子 0 在起点停在「这一版的文件内容」 | `crash_injection::tests` 的 `crash_points_fall_inside_the_starting_point_too`、`crash_points_are_reproducible_proper_subsets_sorted_by_segment`、`crash_points_withhold_every_kind_of_write_and_leave_holes_inside_a_segment`（历史在起点就停，摆不出崩溃状态） |
| 6 | `model_comparison.rs` `units` 里没有实例表时不按根记录的实例表指针交回（`if … {` → `if false {`） | 同一条：`model_comparison.rs:749`，种子 0 在起点停在「单元的分配代」（模型这一版有 mkfs 的实例表、胶水没交） | 同 5 那三条 |
| 7 | `model_comparison.rs` 解实例表时每片丢第一行 | 同一条：`model_comparison.rs:752`，种子 0 第 2 步（可写挂载）停在「这一版的实例表」 | 无 |
| 8 | `crash_injection.rs` `device.image = sectors.clone();` → 空盘 | `every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase`：新用例 `:52` 「两块 4 GiB 的盘上崩溃之后的可写挂载都该成」 | 这个二进制只有这一条 |
| 9 | `crash_injection.rs` 取号的系统配置槽写归进写行 | 同一条：`:65`「挂载途中的二次崩溃一次都没崩在 acquisition」 | — |
| 10 | `crash_injection.rs` 二次崩溃允许的版本不并进挂载写出的根 | 同一条：`:101` 新发现「冷启动读回」（second_crash_inside_warm_up，种子 3 第 26 段，恢复落在实例 3 第 8 代——挂载写的根） | — |
| 11 | c561 用例 `enumerated.states == enumerated.closed_form_states,` → `<=` | `the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape`：`:801`「少评一个状态就不是全量」 | 无 |
| 12 | c561 用例线程行 `freshly_run_slices` 报成片数 + 1 | 同一条：`:790` 读回 + 跑的 ≠ 总片数（`freshly_run_slices=5`、`slices=4`） | 无 |
| 13 | c561 用例闭式按 σ 两遍算 | 同一条：`:766` 计数行 `exhaustive=false`（`closed_form_states=7`） | 无 |

第 10 条同时说明二次崩溃之后的恢复真的会落到挂载写出的根上（允许集合那一半不是摆设）。第 1 条只有单测红：真历史里实现交回了全部角色，漏交那一格没有实际发生过——胶水那一半由第 6 条证。

全量 `every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present` 没跑（派发定的）。它打的两行拿合成日志喂 `research/scripts/admission.py` 的 `judge_crash_case_log` 核过（用第五节那行登记）：合规的那份判绿、线程说明「4 个工作线程跑了 64 片、从进度文件读回 0 片（共 64 片）」；`exhaustive=false` 那份判红「不带 exhaustive=true」；`worker_threads=1` 那份判红「只起了 1 个工作线程」。

## 四、新两截在快档上判出的新发现、快档时长（交主 agent）

快档所在的二进制 `second_transaction_supplement_three_crash_injection`（不在我的文件单里，没改它）在改后的工作区上跑（debug，`capped.sh 4`，`/tmp/claude-1000/impl-rev-b3b/crash-injection-binary-1.log`）：

```
test result: FAILED. 7 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 253.62s
```

红的是 `crash_injection_fast_tier_recovers_only_into_versions_the_model_committed`（`:142`「崩溃状态上「已知红」清单外的失败」）。快档报告里新两截跑满了：

```
崩溃后镜像上的可写挂载 94 次：做成 94、单元区墙上被拒 0、别的理由被拒 0；之后的一次发布做成 94、单元区墙上被拒 0、别的理由被拒 0；之后的池上 checker 跑了 94 次
挂载途中的二次崩溃 282 个：恢复读回文件 225 次、没有文件 57 次、失败 0 次；问模型 282 次、checker 282 次、记录核对器 282 次
```

新发现按 (截, 签名) 去重三条：

| 截 | 签名 | 种子 / 崩溃状态 | 读数（原样摘自报告行） |
|---|---|---|---|
| crash_image（第一截，没改过） | `RecordCheck { aspects: ["claimed_state_missing_unit"] }` | 7463871032432355115，第 56 段（2 个写，持久 1 个，扣下段内第 [0] 个），`Operation(14)／Some(CrashRecoveryAbandoningTheNewestRoot)` | 「记录核对器判红：claimed_state_missing_unit」 |
| after_the_writable_mount_and_one_publish（新） | `CheckerViolations { invariants: ["I-3.1"] }` | 7463871032432355136，第 91 段（2 个写，持久 1 个，扣下段内第 [1] 个），`Operation(21)／Some(PublishOverwrite)` | 「I-3.1：盘 0：记账的已分配 Some(4915200)，遍历全部有效根得到 4882432（其中隔离豁免 0）；机理：根环槽数 24、最新根 txg 29、环里自证过的根槽 24 个、最老的自证过的根 txg 2、遍历的候选根槽 23 个、并进遍历的由记录施加出来的版本 3 个、被实例表判抛弃的根槽 0 个、回退下界 F 3、低于 F 的根槽 1 个」 |
| second_crash_inside_warm_up（新） | `CheckerViolations { invariants: ["I-3.1"] }` | 同上一行那个崩溃状态，二次崩溃崩在暖机 | 「I-3.1：盘 0：记账的已分配 Some(4521984)，遍历全部有效根得到 4489216（其中隔离豁免 0）；机理：…并进遍历的由记录施加出来的版本 2 个、被实例表判抛弃的根槽 1 个、回退下界 F 3、低于 F 的根槽 1 个」 |

- **第一行不是这一轮带进来的**：基线副本（开工那一刻我这 5 份的快照换回去、新用例删掉，别的会话的改动照带）上同一个二进制照样红，同一条、同一个种子与段（`/tmp/claude-1000/impl-rev-b3b/baseline-crash-injection-binary.log` 第 253 行，`test result: FAILED. 7 passed; 1 failed; 1 ignored; … finished in 32.50s`）。
- **后两行是新两截第一次判出来的**：记账比遍历多整一个数据单元（32768 字节），出在一次覆盖写截在它的最后一段、起可写挂载与覆盖写之后（以及挂载暖机途中再崩之后），盘上有抬过的 F（3）、一个低于 F 的根槽。它与「已知红」清单第 0 条（增补 2 收口表第 43 行，也是 I-3.1 记账大于遍历）同向，但清单那一形要根环没转圈，这里最新根 txg 29 ≥ 24 槽，没匹配上。是实现错还是 checker 口径（低于 F、由记录施加出来的版本怎么进遍历），我没有往下查。复现：`SINGLEFS_CRASH_INJECTION_FIRST_SEED=7463871032432355136 SINGLEFS_CRASH_INJECTION_SEEDS=1 SINGLEFS_CRASH_INJECTION_OPERATIONS=24 SINGLEFS_CRASH_INJECTION_POINTS=4`（种子基之外与快档同规模）跑大档那条 ignore 用例。**什么现象会推翻「这是新截判出的真红」**：同一个崩溃状态上不起挂载、只在崩溃镜像上跑 checker 也红（那就是第一截的旧红，没被这一截新找到）——快档第一截在这个状态上没有判红，所以今天的读数不支持这一种。
- **时长**（都是 debug、`capped.sh 4`；check.sh 按 debug、16 线程跑）：

| | 基线副本 | 改后 |
|---|---|---|
| 快档一条（报告里的「用时」） | 19.2 秒 | 107.0 秒 |
| 这个测试二进制整个（`finished in`） | 32.50 秒 | 253.62 秒 |
| `--lib`（含 `crash_injection::tests` 那几条） | 没量 | 131.49 秒（`run-lib.log`） |

  计时副本上量的一个崩溃状态的分解（debug，8 步的短历史）：只读那一截之外，可写挂载约 180 ms（块设备没有 journal 提示，挂载整环扫描 768 MiB）、之后那次发布约 15 ms、挂载之后的 checker 约 200–300 ms、每个二次崩溃约 90–200 ms（主要是 checker）。`crash_states_of_the_history_that_raised_the_floor_into_a_rollback_gap_are_clean_under_the_forward_rollback` 与 `every_crash_state_of_a_written_out_history_recovers_into_a_committed_version` 段内真子集全枚举、单线程，改后是这个二进制里最长的两条。
- **要不要把快档标 ignore、登记 crash-case**：规格说长了要标并给出登记行；那份测试文件不在我的文件单里，我没标。要标的话，登记行照这个写（快档只调一次 `run_crash_injection_campaign`，恰好一行 `CRASH_INJECTION_FINISHED`；它不打 `LAYER0_PARALLEL_FINISHED`，登记不了 `threads=`；抽样不是全量，没有 `exhaustive=`）：

```
crash-case:crash-injection-fast-tier	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_supplement_three_crash_injection:crash_injection_fast_tier_recovers_only_into_versions_the_model_committed count-line=CRASH_INJECTION_FINISHED	# 随机崩溃注入快档：每个崩溃状态上只读恢复与判定、再起可写挂载发一次布跑 checker、挂载途中取号/写行/暖机各一个二次崩溃（代码审阅第 2 条）；标了 ignore，release 跑。计数行 CRASH_INJECTION_FINISHED 只调一次、恰好一行；不打 LAYER0_PARALLEL_FINISHED，没登记 threads=；抽样，没登记 exhaustive=
```

## 五、要改的 kb 句子与 `stage-inputs.tsv` 第 37 行（我都没改，交主 agent / kb-scribe）

**D13（验证路线） 已定项 7 的射程**（`.claude/kb/decisions/13-验证路线.md` 第 133 行那一段），在末尾接一句（规格要写明「层 0 不覆盖可写挂载、由崩溃注入覆盖」）：

> 层 0 不覆盖可写挂载：层 0 在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在层 0 的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-harness/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器（只核挂载自己写出的那几次发布）。「比对的对象是实现恢复后的镜像」那一半，今天只有可写挂载之后的池级 checker 落在实现恢复后的镜像上；记录核对器在崩溃注入里与层 0 一样，入参的崩溃后镜像取的是崩溃态那一份。

**`.claude/gate.d/stage-inputs.tsv` 第 37 行**（`crash-case:c561-sigma-full`）第三列改成：

```
test=singlefs-harness:record_checker_judges_absence_by_the_persisted_set:every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present count-line=C561_SIGMA_FULL exhaustive=C561_SIGMA_FULL threads=C561_SIGMA_FULL
```

第四列注释里「没登记 exhaustive= 与 threads=：计数行 C561_SIGMA_FULL 不带 exhaustive=true（状态数由用例断言等于 262144），用例自己起线程、不经 enumerate_layer0_in_state_slices，不打 LAYER0_PARALLEL_FINISHED」那一半换成：

```
计数行 C561_SIGMA_FULL 在评过的状态数等于闭式 2^18（crash::closed_form_state_count 只对 σ 算）时带 exhaustive=true；用例按掩码区间切片多线程跑、自己打一行与 LAYER0_PARALLEL_FINISHED 同形的线程行（不留进度文件：resumed_slices=0），threads= 按它判；不经 enumerate_layer0_in_state_slices（那一路每个状态多跑一遍不看 journal 的恢复与池级 checker、状态集合差一个）
```

这一行登记上之后，下一次提交跑 54 号 `--full` 时这一格的输入指纹变了（用例文件改了），全量要重跑一次；挂钟改前改后都没量（派发定的不跑全量），每个状态做的事没变（一遍看 journal 的恢复加记录核对器），只是切片与打行。

### 4.1 B3a 与 B1 交回之后重跑（主 agent 知会之后）

`crash.rs`（B3a）与 checker（B1）的最终版进了工作区之后，我在工作区上重跑了自己动到的三个二进制与快档那个二进制，又刷新了基线副本（现在的工作区、我那 5 份换回开工快照、新用例删掉）重跑快档那个二进制与随机历史那个二进制。**没有改判**：

| 跑的是什么（debug 除注明外，`capped.sh 4`） | 结果（原样） |
|---|---|
| 工作区 `--lib` | `test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 91.75s` |
| 工作区 `--test crash_injection_writable_mount_after_the_crash` | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.70s` |
| 工作区 `--test record_checker_judges_absence_by_the_persisted_set` | `test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 10.05s` |
| 工作区 `--test second_transaction_supplement_three_crash_injection` | `test result: FAILED. 7 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 181.85s`（快档「用时 58.3 秒」，第四节那三条新发现逐字不变） |
| 基线副本 同上那个二进制 | `test result: FAILED. 7 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 29.28s`（快档「用时 17.6 秒」，只红第一截那一条） |
| 基线副本 `--release --test second_transaction_supplement_three_random_history` | `test result: FAILED. 23 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 35.77s`（红 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`，与门禁 74 号在工作区上红的是同一条，见第六节） |

第四节表里的 107.0 / 253.62 秒是 B3a、B1 交回之前、机器上别的活更多时量的；这一趟是 58.3 / 181.85 秒。两次都比基线长 3–6 倍。
## 六、验证的原样末尾输出

| 哪一样 | 退出码 | 末行（原样） |
|---|---|---|
| `cargo fmt --all -- --check`（整个工作区） | 0 | （没有输出） |
| `cargo clippy --offline --keep-going --all-targets --all-features -- -D warnings` 加 check.sh 那七条编码纪律 lint | 101 | error: could not compile `singlefs-harness` (bin "e158_root_choice_repair" test) due to 4 previous errors |
| `cargo build --offline --all-targets` | 0 | Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.40s |
| 门禁 33-mutation-tables.sh | 0 | ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 898 条的原文各命中源码一次；没有同名实验二进制、锚点不在本阶段射程的表 5 张：research/mutations/e125_zoned_wp.tsv research/mutations/e129_tear_injector.tsv research/mutations/e129_thin_neighbour.tsv research/mutations/e129_thin_rmw.tsv research/mutations/e158_arms.tsv （本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判） |
| 门禁 53-format-const-placeholders.sh | 0 | ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款）） |
| 门禁 92-layout-checker-sync.sh | 0 | 第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）：.claude/kb/layout/02-second-txn.md |
| 门禁 94-checker-implementation-disjoint.sh | 0 | 这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑 |
| 门禁 93-feature-bits.sh | 0 | ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值）） |
| 门禁 89-closeout-row27-preconditions.sh | 77 | 「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐 |
| 门禁 74-model-differential.sh（`SINGLEFS_GATE_FULL=1`，release） | 1 | `test result: FAILED. 23 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 36.12s`；阶段末行「✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）」 |

- clippy 的红全在两个别人正在改的实验 bin 上（`e156_allocation_basis_counts.rs` 6 处、`e158_root_choice_repair.rs` 8 处，都是 `shadow_unrelated`，按 `-->` 行数的），我动的六份文件一条都没有（`final-clippy.log`，`grep -E '^\s+--> '` 数的）。
- 门禁 74 号红的那一条 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`（`:418`「覆盖写、第一个文件一次都没被「每块盘上都没有」拒过」）在刷新过的基线副本上照样红（4.1 表最后一行），不是这一轮带进来的；74 号要的「模型对拍」行每一段都打了，新发现 0（第 12 条加的每一版比较在随机历史的五段取样上没有判出对不上）。
- 89 号退 77（本次未跑），原因与这一轮的改动无关。
- 动到的测试二进制：见 4.1 表前三行（工作区上跑的最后一趟）。门禁 54、55、57、59 与层 0 没跑（按定义不归我）。

## 七、`git diff --stat -- crates litmus`（原样）

别的会话同时在改 `crates/`，这张表分不出谁改的；我这一轮的改动见第一节。两个用例文件都没进过 git（`git status --short` 报 `??`），不在这张表里：`record_checker_judges_absence_by_the_persisted_set.rs` 是前面的会话建的、还没提交，`crash_injection_writable_mount_after_the_crash.rs` 是这一轮新建的。表里 `model.rs`、`model_comparison.rs`、`crash_injection.rs`、`fault_injection.rs` 的数含开工之前别的会话没提交的改动。

```
 crates/mutations.tsv                               |  566 +-
 crates/singlefs-checker/src/image.rs               |   77 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                |  847 ++-
 crates/singlefs-core/src/admission.rs              |  272 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  132 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   10 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |   11 +-
 crates/singlefs-core/src/mount.rs                  | 2182 ++++--
 crates/singlefs-core/src/mounted_read.rs           |    7 +-
 crates/singlefs-core/src/recovery.rs               |  413 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  433 +-
 crates/singlefs-core/src/write_request_split.rs    |    2 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 3339 ++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 1633 ++++-
 crates/singlefs-harness/src/crash_injection.rs     |  820 ++-
 crates/singlefs-harness/src/device_log.rs          |    2 +-
 crates/singlefs-harness/src/fault_injection.rs     |  749 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             | 1083 ++-
 crates/singlefs-harness/src/lib.rs                 |   44 +-
 crates/singlefs-harness/src/model.rs               |  964 ++-
 crates/singlefs-harness/src/model_comparison.rs    |  338 +-
 crates/singlefs-harness/src/on_device_modes.rs     |    4 +-
 crates/singlefs-harness/src/read_tally.rs          |    2 +-
 crates/singlefs-harness/src/segments.rs            |    2 +-
 .../tests/checker_known_bad_images.rs              |  805 ++-
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/common_tree_split/mod.rs                 |  138 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 .../tests/first_transaction_step_seven_layer0.rs   |  156 +-
 .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 +-
 .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
 ...action_step_three_acquisition_barrier_layer0.rs |   29 +-
 ...second_transaction_step_three_formatted_pool.rs |  566 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  563 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  196 +-
 ...transaction_supplement_three_fault_injection.rs |  412 +-
 ..._transaction_supplement_three_random_history.rs |  434 +-
 ...transaction_supplement_two_admission_formula.rs |  342 +-
 ...ent_two_c519_whole_device_loss_after_warm_up.rs |   29 +-
 ...two_c533_row_publish_record_without_its_root.rs |    7 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   26 +-
 ...rop_and_devices_without_the_selected_version.rs |  258 +-
 ...nsaction_supplement_two_instance_table_chain.rs |  100 +-
 ...tion_supplement_two_instance_table_page_full.rs |   77 +-
 ...plement_two_instance_table_second_page_write.rs |    4 +-
 ..._two_multi_record_transaction_zero_publishes.rs |    5 +-
 ...action_supplement_two_presumed_clause_checks.rs |  105 +-
 ...plement_two_publish_failure_resent_unchanged.rs |    2 +-
 ...n_supplement_two_release_checksum_quarantine.rs |   88 +-
 ..._transaction_supplement_two_rollback_witness.rs | 1079 ---
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 --
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    2 +-
 ...nt_two_row_publish_checks_before_acquisition.rs |    2 +-
 ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |   10 +-
 ...second_transaction_supplement_two_tree_split.rs |  147 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 85 files changed, 22084 insertions(+), 10583 deletions(-)
```

## 八、草稿目录里删了什么、留了什么

- 删了：`/tmp/claude-1000/impl-rev-b3b/copy-timing`（1.6G，计时副本）、`/tmp/claude-1000/impl-rev-b3b/copy-mutate`（1.3G，变异副本）、`/tmp/claude-1000/impl-rev-b3b/copy-baseline`（2.6G，基线副本）、`/tmp/claude-1000/impl-rev-b3b/target-main`（15G，主工作区用的编译目录）。大小都是删之前 `du -sh` 量的。
- 留着：报告、`mutations-append.tsv`（主 agent 要追加进 `crates/mutations.tsv`）、`progress.md`、各次跑的日志（`mutation-logs/`、`rerun-*.log`、`crash-injection-binary-1.log`、`baseline-crash-injection-binary.log`、`gate-*.log`、`final-*.log`）、开工快照 `baseline/` 与变异用的原件 `pristine/`（几份源码，不是仓副本）、几个一次性脚本。这些都不进仓；日志是主 agent 核这份报告的出处。
- 快档跑时写过判红镜像的临时目录（`/tmp/singlefs-crash-injection-<pid>-seedbase-…`）是用例自己跑完就删的（快档那一档 `DeleteTheImageFilesWhenTheRunFinishes`）。

## 九、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交。
- 没改 `crates/mutations.tsv`（派发定的），13 行在 `mutations-append.tsv`，都已证红，没有留给 59 号的行。
- 没跑 c561 全量（262144 个状态，派发定的）；两行的打法拿小段与合成日志核过（第三节末段）。登记行没改（`stage-inputs.tsv` 不在写范围），要补的字段在第五节。
- 没改 kb（写不了），D13（验证路线） 已定项 7 射程要接的句子在第五节。
- 没改快档那份测试文件（不在文件单里）：没给快档标 ignore，也没把第二、三截的计数断言加进 `assert_every_crash_injection_path_was_exercised`（快档今天只在报告里打出这几行，不断言；第二、三截跑满与否由新用例钉，快档那一档没钉）。
- 快档今天红：第一截那一条是开工前就有的（基线副本照样红），后两条是新两截判出的 I-3.1（第四节）；都没修，也没往「已知红」清单里加（`history.rs`，不在文件单里）。
- 第 12 条在树表 0 条的一版上只比到「重写了哪几个角色」，实例表与分配代比不了；二次崩溃上记录核对器只核挂载自己的发布；第二截没接记录核对器；二次崩溃上不再起挂载；崩溃后被单元区墙拒只计数——都在第二节，交主 agent。
- 门禁 74 号那条红（`unit_area_wall_sampling_…`）没查原因：基线副本上照样红，不在这一轮的改动里。
- 大档（`crash_injection_large_tier_from_the_environment`，ignore）没跑；它走同一个入口，第二、三截会跟着跑。
