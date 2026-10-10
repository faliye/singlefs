# m3-c694-r2 云端辩方（Sonnet）报告：K1、K4（2026-10-07）

立场：云端辩方。复核第一轮判决 `research/prompts/m3-c694-r1-main-verification.md`，替被判出局的一方（U2、A、只做 C、只做 B）辩护，判它们是不是被同样的理由打中。
结论先行：①K4：13 行里 C 的挂钩走到且非空的是 43（unlink 多记录流）与 47（崩溃注入快档），40 的挂钩空，B 的拒一行都走不到，恢复补法的前提在 13 行上都不成立，登记判定不变（量到 + 推的）；②K1：第一轮判决站得住，但有三处够不着或理由挂错（第六节）。

引用写法：kb 写 `文件@小节标题`（原文整行抄）或 kb 文件自己的行号；代码写冻结副本全路径加行号，根目录 `~/.local/share/singlefs/work/m3-c694-r2/tree/`（下文写 `crates/…`，cite-check 的 `--root` 指它）。草稿目录 `~/.local/share/singlefs/work/m3-c694-r2-sonnet/`：交回前已把仓副本与各个编译目录、复跑工作目录删掉（它们的内容由 `research/prompts/m3-c694-r2-sonnet-model/rerun.sh` 从冻结树重新造得出），留下的是各批分片日志（`b*.log`）、`e2` / `e3` 的输出日志、`tools/`、写报告用的分段草稿。
我在冻结副本的拷贝上加了一份只数不改行为的探针补丁（`research/prompts/m3-c694-r2-sonnet-model/probe.diff`：影子账那一函数的各支加计数、C 的挂钩加计数，C 的隔离由开关控制，开关与计数口径照第一轮正推腿 `research/prompts/m3-c694-r1-sonnet-model/candidates.diff` 的 W4 / W1 / W2），再写了一条 harness 档探针（`research/prompts/m3-c694-r2-sonnet-model/tests/m3c694_k4_reach.rs`）。方案都按 `crates/` 今天的实现谈。

## 一、K4　回扫（静态读码：13 条已登记的崩溃枚举流各走到哪几支）

`.claude/gate.d/stage-inputs.tsv` 在我开工之后被改过一次（文件时间 2026-10-07 03:32，第 43 行新加了 `crash-case:layer0-unlink-after-the-multi-record-publish-stream`）：我先读到 12 行、核引文时是 13 行，下面按 13 行写，行号是今天这份的（`grep -n '^crash-case:' .claude/gate.d/stage-inputs.tsv` 出 38–50）。

三支改动落点（读冻结副本）：
- C：可写挂载读阶段之后、重建分配器返回之后加一个挂钩，每次可写挂载都过它。第一轮正推腿的副本落点在 `crates/singlefs-core/src/mount.rs` 的 `rebuilt?` 之后，用读阶段已有的 `records` 与 `effective_root`；它只在「水位之上、没被施加的记录」存在时有东西可隔离。
- B：只在 `crates/singlefs-core/src/mount.rs:1542`「None => return Err(」那一支改。能走到它的前提是：有被抛弃根、账读不出、从映射现算不成、重读仍不成。
- 恢复一侧补法：`recover()` 择到的根走读失败那一支（`RecoveryOutcome::Failed`）。

层 0 流的每个状态上做什么：`crates/singlefs-checker-tier/src/crash.rs:1374`「let consulted = recover(image, JournalPolicy::Consult);」；全文件没有 `mount_writable`、没有 `mount_read_only`（`grep -n 'mount_writable\|mount_read_only' crates/singlefs-checker-tier/src/crash.rs` 无输出）。所以层 0 流（下表 38、39、42、44、45、46 行；43 行在观察者里另起可写挂载，见该行）只走恢复这一支，C 与 B 的挂载代码在它们的崩溃状态上一行都没跑。

oracle 对「恢复落到旧根」的口径：`crates/singlefs-checker-tier/src/crash.rs:1111`「if (effective_txg, effective_instance) < (newest_txg, newest_instance) {」，报的违例种类是 `RecoveredToAnOlderRoot`（同文件 1115 行「根槽已持久而恢复到旧态（盘上最新的根槽是实例 {} 第 {} 代，走的是实例 {} 第 {} 代）」）：枚举流的状态里，根槽已持久的那条根在 `image.writes` 里时，恢复落到更旧的版本一律判违例。

| 行（`.claude/gate.d/stage-inputs.tsv`） | 每个状态上做什么（读码） | C | B | 恢复补法 | 判定会不会变 |
|---|---|---|---|---|---|
| 38 layer0-first-stream | 恢复 + oracle、池级 checker、记录核对器（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:2` 「每个状态跑三件事——」）；挂载只用来造起点流 | 不走（起点盘上没有水位之上的记录） | 不走 | 走到恢复；走读失败数断言 0，补法的前提「所选根走不通」不成立 | 不变 |
| 39 layer0-second-stream | 同上，多版本 oracle（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:4` 「每个状态跑恢复 + 多版本 oracle（实际走的根是哪一代就得读出那一代的内容）、池级 checker、记录核对器。」）；挂载只在造流时用（该文件 273、1127 行的 `mount_writable`） | 不走 | 不走 | 同上 | 不变 |
| 40 floor-raise-pushed-by-the-session | 每个崩溃状态恢复，再起一次可写挂载（`crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs:245` 「// 每个崩溃状态可写挂载一次、挂载之后的镜像跑池级 checker：按线程分片并行（观察者在调用线程上，逐个挂载太慢）。」），挂载被拒记失败（同文件 289 行「Err(error) => failures.push(format!("挂载失败：{error:?}")),」） | 走到挂钩；有没有东西可隔离见第二节 E2 | 不走（没有被抛弃根的造法） | 走到恢复；失败数断言 0 | 见 E2：不变 |
| 41 c561-sigma-full | 记录核对器按持久集合判缺席；`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:470`「let effective_root = recover(&image, JournalPolicy::Consult).effective_root;」；挂载在造流时用 | 不走 | 不走 | 只取 `effective_root`，不判走读 | 不变 |
| 42 layer0-multi-record-publish-stream | 恢复 + oracle + checker + 发布边界（`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:18` 「多跑的一步只有恢复本身（这条流上没有重开、挂载）。」）；这条流有多条记录的发布，状态里有「第一条记录落了、末条没落」 | 不走（没有挂载），但它造出的正是 C 要隔离的盘面（水位之上没施加的记录） | 不走 | 走到恢复；失败数断言 0 | 不变 |
| 43 layer0-unlink-after-the-multi-record-publish-stream | 多记录流之后 unlink（意图 → 两批 → 最后一批）的层 0 全量；每个状态恢复 + oracle + checker + 记录核对器，**再在每个状态上可写挂载一次、核续做走到空**（`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:16`「多跑的一步是每个状态上的可写挂载（取号、写行、暖机、续做）。」；挂不上就 panic：`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:565`「.unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));」） | **走到，而且有东西可隔离**：多记录发布的状态里有「第一条记录落了、末条没落」（整次不施加，点名的单元已持久），也有新实例写行发布的跨实例记录；盘宽 4 GiB 级（`tests/common` 的文件镜像） | 不走（没有被抛弃根的造法） | 走到恢复；失败数断言 0 | 判定看的是挂载做成、续做结局是 `WalkedToEmpty`（`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:569`「DeletionQueueContinuationOutcome::WalkedToEmpty」）、续做步数与挂完那一版的形状，都不看落点；C 多隔离的槽（第二节量到 6 至 28 槽）在这个盘宽上不挡路 → 不变（推的；它是 checker 档，我没跑） |
| 44 layer0-tree-split-streams | 同上（`crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs:22`「多跑的一步只有恢复本身（这几条流上没有重开、挂载）。」） | 不走 | 不走 | 同上 | 不变 |
| 45 layer0-position-addressed-tree-streams | 单元写段全展开、每个两遍恢复 + checker；起点用 `mount_writable` 造（`crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs:304`「mount_writable(&parameters(), &mut devices).expect("mkfs 之后第一次可写挂载");」） | 不走 | 不走 | 同上 | 不变 |
| 46 writable-mount-of-an-io-min-8192-pool | 枚举的是那次首次可写挂载自己的写（33 态），每态恢复 + checker；起点是只做过 mkfs 的池（`crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs:99`「mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");」） | 挂钩在造流那次挂载里过一次，盘上没有记录，空 | 不走 | 同上 | 不变 |
| 47 crash-injection-fast-tier | 随机历史（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:295`「weights: GenerationWeights::BROAD,」，宽 `crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:59`「device_width: HistoryDeviceWidth::FourGibibytes,」）每段摆崩溃态，每态只读恢复 + 判定、再起可写挂载发一次布跑 checker、挂载途中取号 / 写行 / 暖机各一个二次崩溃（`.claude/gate.d/stage-inputs.tsv:47`「随机崩溃注入快档」） | **走到**：二次崩溃落在新实例写行那次发布的记录之后、根之前，再挂时水位之上有跨实例的记录，前缀规则不施加（`.claude/kb/decisions/23-journal的角色与格式.md:369`「所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。」是它的一个特例）。结局见第二节 E1 | 不走：随机历史里没有被抛弃的根（E1 三档 `abandoned_roots_seen=0`；`crates/singlefs-harness/src/history.rs:492`「/// 这一步不再造出被抛弃的根；乙罩不到的」） | 走到恢复；「恢复三种结局见过两种（失败一次都不许有）」（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:129`「/// 截回到最后一版之前过、恢复三种结局见过两种（失败一次都不许有）、journal 记录真的施加过、模型真的比过内容。」），补法的前提不成立 | 见第二节 E1：C 开着，走到挂钩的状态 Ok 变 Ok；4 GiB 一档没跑池级 checker，只比挂载结局 |
| 48 read-faulted-states-never-hold-the-witness-criterion | 逐个摆崩溃态，只读复算判据 W（`crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs:64`「let fields = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion);」），每段最长前缀上起一次可写挂载录它的写（`crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs:108`「let _mounted = mount_writable_with_space_admission(」） | 走到挂钩；C 开了会改挂载选的落点、录下的挂载写表因而不同；W 只看恢复字段与系统配置、记录计数，不看落点（读码） | 不走 | 走到恢复；失败不是它判的东西 | 不变（推的：W 不依赖落点；没在 C 开着时跑这一条，它是 checker 档） |
| 49 instance-table-sweep-mount | 清扫那次挂载（抹头、清扫发布）的全部写按层 0 展开，恢复 + oracle + checker + F5；挂载在造流时用（`crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs:349`「let mounted = mount_writable_with_instance_table_sweep_configuration(」） | 造流那次挂载里挂钩过一次；起点盘上记录都在水位之下（推的） | 不走 | 走到恢复；失败数断言 0 | 不变 |
| 50 instance-table-sweep-phases-remount | 清扫两段上每个崩溃态再起一次可写挂载（`crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs:584`「let remounted = mount_writable_with_instance_table_sweep_configuration(」），被拒记失败（`crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs:594`「failures.push(format!("再挂被拒：{error:?}"));」） | 走到挂钩；清扫发布是同一个实例的发布，它的记录崩在根之前时按 D23 已定项 15 由记录重建根，记录被施加、水位之上不剩（推的，没造盘面验） | 不走 | 走到恢复；失败数断言 0 | 不变（推的） |

读法：
- 13 行里**真正会走到 C 的挂钩并且挂钩里有东西可隔离**的是 43 与 47；40 的挂钩是空的（第二节 E2 量到）；38、39、41、42、44、45、46、49 不起可写挂载或只在造流时起一次；48 走到挂钩但判定不看落点；50 走到挂钩、挂钩里应当是空（推的）。
- **B 的拒走不到任何一行**：能造出「被抛弃根 + 账读不出」的只有 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 一个造法（`crates/singlefs-harness/tests/common/mod.rs:741`「/// 崩溃恢复造出一条被抛弃的根（第一轮判决 H6 那一形；D23（journal 的角色与格式） 已定项 14 射程：影子账与按实例表判抛弃留着，」）。它的调用方是 harness 档 14 个测试文件（`grep -rl 'abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before' crates/singlefs-harness/tests` 去掉 `common/mod.rs` 数出 14），checker 档里只有 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 调它，`src/bin` 下 e158、e171 三个实验二进制提到它，都不是登记的崩溃枚举用例。B 改的那一支被判的用例是 harness 档的（第一轮正推腿量过其中 4 个目标 31 条在 W1 下全绿，`research/prompts/m3-c694-r1-sonnet-output.md:95`「**甲 / 乙 / 丙之下 31 条相关既有用例的结局**：」）。
- **恢复一侧补法**：13 行每个状态都走 `recover`，补法的前提是「所选根走不通」。登记用例自己的断言是 0 违例（走读失败是一种违例，`crates/singlefs-checker-tier/src/crash.rs:1125`「(RecoveryOutcome::Failed { failure, .. }, _) => Some(violation(」），E1 三档与 E2 的状态里 `recover_failed_on_a_chosen_root=0`（第二节）：前提在登记用例的状态上不成立。会让前提成立的是同文件里靶向的阳性对照（普通 `#[test]`，不是登记行那条 `#[ignore]` 用例）：它们从全部持久起手摘掉单元——`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:771`「let all_persisted = vec![true; prepared.writes.len()];」、`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:949`「let all = vec![true; prepared.writes.len()];」——全部持久含该次发布的系统配置轮换，即系统配置见证到了那条根（读码推的），补法按它的条件不退，它们照旧判红。所以补法在这 13 行上**不走**，登记流的判定不变。
- 补法在这 13 行之外才会改判定，分两类（读码推的，没造盘面）：① 被判的那条根的根槽写在流的写表里、它的单元写被后来的写盖掉或没落、系统配置轮换没落：补法让恢复退到见证到的旧版，oracle 先走 `crates/singlefs-checker-tier/src/crash.rs:1111`「if (effective_txg, effective_instance) < (newest_txg, newest_instance) {」那一支，判 `RecoveredToAnOlderRoot`（仍红，种类名从 `ReadFailed` 换了；按种类计数的发现表 `Layer0OracleViolationKind::EVERY_KIND` 与进度文件里那一类的计数会变）；② 根槽在起点镜像里、流是取新实例那次挂载自己的写（第一轮攻方 H5 的造法）：`newest_persisted_root` 只看写表，不进 1111 那一支，退到见证版 P 之后交给版本表、P 在表里、内容对上，判绿——这正是补法要的结局，第一轮 L0TALLY 的 violations 4 / 9 在这里变 0（`research/prompts/m3-c694-r1-opus-output.md:102`「L0TALLY m1 W=384 k=5 arm=today states=4128 root_persisted=4116 violations=4 failed=4 no_file=0」）。

## 二、K4 实测（harness 档缩小复刻：C 的挂钩、B 的拒、恢复补法各走到几次）

做法：冻结树拷贝上打 `research/prompts/m3-c694-r2-sonnet-model/probe.diff`（影子账函数各支、C 的挂钩加计数；C 的隔离由开关控制，默认关 = 今天），三条探针在 `research/prompts/m3-c694-r2-sonnet-model/tests/`。复跑与期望文件在同目录（`rerun.sh`、`compare.sh`、`expected-quick/`、`expected-full/`）。这里的数全是**副本上量的**，按 `.claude/rules/three-way-inference.md` 不进 kb，要引由主 agent 在入库装置上重做。

**E1　随机历史的每个段内前缀当崩溃状态**（种子基、24 段 × 24 步、`GenerationWeights::BROAD` 与崩溃注入快档同一组；快档每段只摆 4 个抽样点，这里取每个段的每个前缀；每个状态上今天与 C 两臂各挂一次可写挂载）。`extracted` 里取 `K4TOTAL` / `K4ARM` 行：

| 盘宽 | 状态数 | 挂载走到挂钩的状态 | 所选根走读失败 | 没择到根（mkfs 中途） | 被抛弃根 `abandoned_roots_seen` | 拒 `refused` | 挂钩非空的挂载 | 挂钩点名的落点（两盘合计） | C 与今天结局不同 | 其中 Ok 变 Ok | 准入结局翻了 | 挂载之后池级 checker 红 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 384 槽 | 9790 | 9358 | 0 | 432 | 0 | 0 | 168（1.8%） | 2652 | 168 | 168 | 0 | 0 |
| 256 槽 | 10558 | 10126 | 0 | 432 | 0 | 0 | 168（1.7%） | 2660 | 168 | 168 | 6 | 0 |
| 4 GiB（6 段 × 12 步，没跑池级 checker） | 1523 | 1415 | 0 | 108 | 0 | 0 | 26（1.8%） | 408 | 26 | 26 | 0 | 未跑 |

读法：
- **B 的拒与恢复补法的前提在三档上一次都没出现**：`abandoned_roots_seen=0`、`refused=0`、所选根走读失败 0（「没择到根」不是「所选根走不通」，是 mkfs 做到一半、盘上还没有根）。
- **C 的挂钩非空的状态占 1.7% 至 1.8%，每个状态恰一条记录在水位之上**（`c_hook_records_above_water` 等于非空挂载数）。这批历史里每个有记录写的段都是 2 个记录写（一条记录的两份）：单记录发布，所以没有「第一条落了、末条没落」（额外探针 `research/prompts/m3-c694-r2-sonnet-model/extra/m3c694_k4_record_histogram.rs` 量的，不在 `rerun.sh` 里，原样输出一行：`K4RECORDSEGMENTS record_writes_per_record_segment_histogram {2: 464}`）；水位之上没施加的记录是别的实例的——新实例写行那次发布的记录落了、根没落，前缀规则不跨实例边界（`.claude/kb/decisions/23-journal的角色与格式.md:369`「前缀规则不跨实例边界」）。直接的一支：384 槽 168 个状态里 48 个所选根是 mkfs 的第 0 代根（`eff=(0,0)`，`isolated=6`），那一行的原文是 `.claude/kb/decisions/23-journal的角色与格式.md:369`「所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。」。其余是同一规则在后来的实例上。
- C 在这些状态上多隔离 6 至 24 个槽（两盘合计，3 至 12 个 / 盘）；今天一个都不隔离。挂载 Ok 变 Ok，没有一个状态因此挂不上、也没有一个状态挂载之后池级 checker 变红。
- **256 槽盘上有 6 个状态准入结局翻了**：4 个从「取号之前判过、没推」变成「推一串抬 F 之后仍不够」（`still_short_raises1`），2 个变成「推一串抬 F 之后够了」（`after_raises1`）；挂载都照样做成可写（`.claude/kb/decisions/16-发布语义.md:42`「挂载处推满仍不够时挂载照样做成可写」），之后的发布照发布准入判。384 槽与 4 GiB 上没有翻。这是 K2 的格（窄盘多隔离的代价），我只报量：6 / 168，不是登记流的判定。
- 4 GiB 一档只有 6 段 × 12 步、没跑池级 checker：单次挂载约 0.33 秒（release，不跑池级 checker 量的：6 段里 seed_offset=16 那段 234 个状态 × 2 臂 157 秒），24 段 × 24 步的 4 GiB 全扫一遍要几十分钟，我没做。快档在 4 GiB 上摆 96 个抽样点；按 1.8% 估约 1 至 2 个抽样点会走到挂钩（推的，没跑快档）。

**E2　登记用例 floor-raise-pushed-by-the-session 的缩小复刻**（`tests/m3c694_k4_floor_raise.rs`：同一条历史——384 槽小盘、第一个文件之后崩了再挂、会话覆盖写 61 次、直接发布到准入先拒、推一串抬 F、只录推的那一串；崩溃状态取每个段的每个前缀，不枚举段内子集与系统配置槽撕裂态；每个状态四臂各挂一次）。原样：`K4FR admission_refused_at_overwrite=63`、`K4FR raise_publishes=2 writes=36 segments=9 record_writes=4`、`K4FRTOTAL states=37 hook_mounts_with_unapplied_records_under_none=0 hook_placements=0`，四臂 `mount_err=0 mount_ok_with_red_checker=0 states_differing_from_none=0`。**挂钩在 37 个状态上都是空的**：推的那一串是同一个实例的两次空发布，记录写在屏障之后，记录落了点名的单元就已落；记录落了根没落时恢复由记录重建根（`.claude/kb/decisions/23-journal的角色与格式.md@已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根`「定案：由记录重建那次发布的根：记录头加」），水位之上不剩。所以 C 在这一条上不改判定。

**E3　被抛弃根的三种造法**（`tests/m3c694_k4_unwitnessed_abandon.rs`：崩溃镜像里那条根持久、它之后的系统配置轮换没落；第二次挂载读它的根槽与它那次发布的每个单元读回全 0；第三次挂载什么都不藏。四臂各自从头造）。`K4E3` 行：

| 造法（R = 被抛弃的那条根） | 臂 | R 那次发布写的单元数 | 第二次挂载把其中几个写掉 | R 的树表单元第二次挂载之后 | 第三次：被抛弃根数 / 账读不出 / 拒 | 行 |
|---|---|---|---|---|---|---|
| 树表 0 条池的暖机第二条根 | 四臂同 | 0 | 0 | 读得出（0 条） | 0 / 0 / 0（第二次挂载按记录重建出 R，R 没被抛弃） | `research/prompts/m3-c694-r2-sonnet-model/expected-quick/unwitnessed.txt:6`「arm=none unit_writes=0 clobbered_by_the_second_mount=0 r_tree_table_entries_after_the_second_mount=Ok(0)」 |
| 带文件池的第一个文件那条根 | 今天、W1、W2 | 20 | 6 | 读得出（11 条） | 1 / 0 / 0 | `research/prompts/m3-c694-r2-sonnet-model/expected-quick/unwitnessed.txt:10`「arm=none unit_writes=20 clobbered_by_the_second_mount=6 r_tree_table_entries_after_the_second_mount=Ok(11)」 |
| 带文件池的第一个文件那条根 | C | 20 | 0（挂钩隔离 28 槽） | 读得出 | 1 / 0 / 0 | `research/prompts/m3-c694-r2-sonnet-model/expected-quick/unwitnessed.txt:9`「arm=c unit_writes=20 clobbered_by_the_second_mount=0」 |
| 树表 0 条池第二次挂载（实例 2）的写行那条根 | 今天、W1、W2 | 8 | 8 | 读得出（0 条） | 1 / 0 / 0 | `research/prompts/m3-c694-r2-sonnet-model/expected-quick/unwitnessed.txt:2`「arm=none unit_writes=8 clobbered_by_the_second_mount=8 r_tree_table_entries_after_the_second_mount=Ok(0)」 |
| 同上 | C | 8 | 0（挂钩隔离 12 槽） | 读得出 | 1 / 0 / 0 | `research/prompts/m3-c694-r2-sonnet-model/expected-quick/unwitnessed.txt:1`「arm=c unit_writes=8 clobbered_by_the_second_mount=0」 |

读法：
- **C 把 C694 那一形的第一步（被抛弃根的单元被新实例的挂载写掉）挡住了**：两种起点上今天写掉 6 个与 8 个、C 一个都不写掉；W1 与 W2 不挡（它们只改拒的那一支）。这复现了第一轮两条腿「C 修好拒与 K1-2」的方向，在我的造法上也成立。
- **三种造法里第三次挂载一次都没有拒**（`third_refused=0`）：被抛弃根的账都还读得出，因为写掉的单元不含树表与映射根——这与第一轮拒的那 11 个起点不同（那里是覆盖写 5 次之后树表与映射根两个都被写掉）。我没有在这条探针里接会话覆盖写，所以没有复现拒本身；拒的复现用第一轮的模型（`research/prompts/m3-c694-r1-sonnet-model/`）。
- **树表 0 条的根在我的造法里走不到映射现算那一支**：第二次挂载把 R（树表 0 条池第二次挂载的写行根）那次发布写的 8 个单元全覆盖之后，R 的树表单元仍读得出（`Ok(0)`），所以这 8 个单元里没有树表单元——树表 0 条的根指着共用的第 0 代树表（`crates/singlefs-core/src/recovery.rs:2266`「/// 一条根指着的树表是不是 0 条（mkfs 的第 0 代树表，第一次可写挂载的暖机根照抄它）：」是暖机根的说法，写行根这一条我是观察到的，没找到条款）。`tree_table_has_no_entries` 那一臂只读这一个单元、其余指针取根记录里的，账读得出，走不到映射现算。这就是 K1 那一节里「怀疑的映射现算那一处量下来走不到」的依据，范围限于这一种造法。

**K4 汇总：C、B、恢复补法在 13 行登记流上各走到哪、判定变不变**

| 改动 | 走到的行 | 走到时判定 |
|---|---|---|
| C | 挂钩非空：43、47；挂钩空：40（E2 量）、50（推的）；走到但判定不看落点：48；其余不起可写挂载或只在造流时起一次 | 量到的（E1 / E2）：挂载 Ok 变 Ok、池级 checker 不红；准入结局只在 256 槽盘上翻（6 / 168）。43 的挂载断言是做成、续做走到空、步数与形状（`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:569`「DeletionQueueContinuationOutcome::WalkedToEmpty」），不看落点，盘宽 4 GiB 级 → 不变（推的，checker 档没跑）；47 在 4 GiB 上 → 不变（推的） |
| B | 无 | 这 13 行里一次都不走到；走到它的是 harness 档 14 个测试文件 |
| 恢复补法 | 13 行都走 `recover` | 前提「所选根走不通」不成立 → 不变；补法之外才会改的两类见第一节末条 |

## 三、K1　第一轮判决站不站得住（辩方）

判决文件：`research/prompts/m3-c694-r1-main-verification.md`（下文「判决」）。被辩护的一方：U2、A（裸跳过）、只做 C、只做 B。每行先写它怎么站、再写它够得着哪里；「同样打中」指同一个理由对别的候选一样成立。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| K1「采 U1」：U2 后半句（拒得合法、之后某一次卸载做得成）不成立 | **站得住**。U2 能替自己说的只有一句：卸载被拒时盘一字节不动，拒本身不丢数据——这句是真的，但 U1 的理由本来就是闩，不是损坏 | 拒的那一支回滚分配器就返回（`crates/singlefs-core/src/mount.rs:3523`「*allocator = allocator_before_the_raise;」），建写者在它之后（`crates/singlefs-core/src/mount.rs:3563`「let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());」），整串预演与先写系统配置更在后面，所以拒的时候盘上一个字节没动（读码）。出口：`ShadowLedger::Off` 只供测试（`crates/singlefs-core/src/mount.rs:834`「/// 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：关掉影子账」），非测试代码里只有 `crates/singlefs-core/src/mount.rs:850` 与 `crates/singlefs-core/src/mount.rs:2228` 两处写到它，产品入口恒开（`crates/singlefs-core/src/mount.rs:5701`「mount_writable_with_test_only_switches(parameters, devices, space_admission, ShadowLedger::On)」）。再挂那一路的拒在取号之前：`crates/singlefs-core/src/mount.rs:6097`「} = rebuilt?;」排在 `crates/singlefs-core/src/mount.rs:6104`「establish_instance(」之前。第一轮量到 11 / 11 再挂被拒（`research/prompts/m3-c694-r1-sonnet-output.md:52`「U2 的后半句（拒得合法、之后某一次卸载做得成）在产品路径上不成立，是个自己锁死自己的闩」） |
| K2 行里 U2 出局的理由「推空发布把它推出根环，碰不到 K1-2」 | **这句理由够不着 U2 一家**：K1-2 同样打中 A、B、today，不分辨它们（`research/prompts/m3-c694-r1-opus-output.md:158`「不分（A、B、C、today、U2 同中）」）。U2 出局站得住，但该写的理由是另两条 | 判决：`research/prompts/m3-c694-r1-main-verification.md:25`「U2 推空发布把它推出根环，碰不到 K1-2」。U2 的改进形（卸载撞上拒时推空发布直到那条根离开根环）只覆盖卸载一路：会话写那一路的拒在准入推抬 F 的同一个函数里（`crates/singlefs-core/src/mount.rs:3511`「match isolate_slots_referenced_only_by_abandoned_roots(」），再挂那一路的拒在取号之前（上一行）；它还要多发至多 24 次空发布（`research/prompts/m3-c694-r1-opus-output.md:163`「R 离开根环最多再发 24 次」） |
| K1「拒不护任何东西」 | **站得住**。我把 Opus 的 33 / 36 独立复核了一遍，且 33 个的第一次读不出全在第 1 步 | 判决：`research/prompts/m3-c694-r1-main-verification.md:22`「**两条腿一致，采 U1**」。复核脚本 `research/prompts/m3-c694-r2-sonnet-model/check_r1_steps.py`（只读 Opus 的逐步表 `research/prompts/m3-c694-r1-opus-model/out/h1-grid-steps.txt`），原样输出两行：`today_starts=42 refused_starts=6 not_refused=36 not_refused_with_a_step_where_R_is_unreadable=33` 与 `first_step_where_R_is_unreadable_histogram={1: 33}`。判决把它说成「第一次覆盖写就」，对 33 个都成立 |
| 同一条的证据分两档 | **站得住，但该写明**：池级 checker 红 I-7.4 是单故障下的证据；Opus 的「R 可读性探针」把更高实例的根槽全写零再恢复，是第二个故障 | 探针定义在 `research/prompts/m3-c694-r1-opus-output.md:69` 那一长段的 (c) 项（「R 的可读性探针」几个字在那一行）。这不动 U1：U1 的前提是被拒的那一刻 R 已经不可挽，单故障证据（checker 红）已够；探针只增加「R 对用户有什么用」那一问上的证据，那一问在 U1 之外 |
| K1-2：取新实例那次挂载里写行根落盘之前断电，恢复落到被抛弃的根上走读失败 | **站得住，而且它在注册的故障模型之内**；它同样打中 A、B、today、U2，所以它是选 C 的理由，不是废掉 A / B / U2 的理由 | 同中：`research/prompts/m3-c694-r1-opus-output.md:105`「A、B 的数与 today 逐行相同」。故障模型：随机历史里注入的就是「最新那条根的根槽与它那次发布的记录点名的单元整次挂载读不出」：`crates/singlefs-harness/src/history.rs:489`「下一次可写挂载时根环里最新那条根（这个会话的现行版本）的根槽与它那次发布的记录点名的单元整次挂载读不出（读回全 0；」。崩溃注入把这一类步标成读故障步：`crates/singlefs-checker-tier/src/crash_injection.rs:1141`「最新那条根的根槽与它点名的单元读回全 0），每块盘最新那一槽系统配置读回全 0 的可写挂载」。**被注入的故障里记录本身不在被藏之列**：C 的盲区那一形（记录也读回全 0，`research/prompts/m3-c694-r1-opus-output.md:151`「**打中 C 的一形**：R 的 journal 记录也在挂载那一刻读回全 0（故障多藏两个记录写）。」）超出注入框架今天造的故障 |
| 「条款空白在 D16 已定项 1「根槽这一次读坏」那一行」（写回 D16 已定项 1 或 D23 已定项 14） | **站得住一半，漏了一处**。D16 那一行的词汇是对的（它管全零槽），但 C 要落的机制还牵到 D28 已定项 1 第九项：C 隔离的槽不是「被抛弃根引用的槽」，而准入式子里第九项直接取分配器实际隔离的槽数 | D16 那一行只管回收门槛（`.claude/kb/decisions/16-发布语义.md:37`「**读坏的槽与读回全零的槽**：挂载那一刻读坏（读不出，或读得出不是全零而自证不过）与读回全零的槽，回收门槛三处（挂载时含 m 取不到那一支、进程内、抬 F 那一遍）钉在它可能住过的最旧 txg，」）。D23 已定项 14 已写「没见证到」那一半留着：`.claude/kb/decisions/23-journal的角色与格式.md:407`「影子账留着的理由收窄成系统配置没见证到的那一形」、`.claude/kb/decisions/23-journal的角色与格式.md:392`「系统配置没见证到的最新根，…落地之后由下一次挂载在自己的 c_见证 里看到」。D28：`.claude/kb/decisions/28-挂载期承诺量.md:27`「**被抛弃根独占量**（第九项）：取值 = 只被被抛弃根引用的槽数 = 被抛弃根引用的槽 − 回退候选集里的根引用的槽（下界 0），与分配器实际隔离的集合是同一个数；」；代码把分配器的隔离数直接当第九项：`crates/singlefs-core/src/admission.rs:583`「abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),」。C 一上，第九项的定义与「同一个数」的断言有一边要改；E1 在 256 槽盘上 168 个 C 有隔离的状态里 6 个把准入结局翻了（第二节） |
| C695「次序修法」够不够 | **够它自己要修的**；我怀疑过它够不着影子账的映射现算那一处，量下来走不到；剩一处同样打中所有候选 | 够：判决写的 16 处在 `open_pool_for_read` 先读空映射根（`crates/singlefs-core/src/mounted_read.rs:376`「let mapping_tree = read_code_two_tree(」），`research/prompts/m3-c694-r1-main-verification.md:36`「C695 甲在攻方的同类改法上被扫过 16 处」。**怀疑过的**：同一个习惯（空映射根当真指针读）还在影子账的映射现算里（`crates/singlefs-core/src/mount.rs:1581`「/// 树表自己的落点取根记录里那条指针。」那一行的后半句自己写了「映射根是空指针的 mkfs 第 0 代、树表 0 条的一版也在内」），B 的正推甲凭据对空映射根永远不放行（`research/prompts/m3-c694-r1-sonnet-model/candidates.diff:53`「if *pointer == crate::pointer::NodePointer::empty_root() {」，下一行 `return false;`）。若被抛弃的是树表 0 条那一版、树表单元又被写掉，影子账就走到映射现算、现算交 `None`、拒。**量下来走不到**：树表 0 条的暖机根照抄第 0 代树表（`crates/singlefs-core/src/recovery.rs:2266`「/// 一条根指着的树表是不是 0 条（mkfs 的第 0 代树表，第一次可写挂载的暖机根照抄它）：」），E3 里树表 0 条的写行根（我观察到的，没找到条款）在第二次挂载把它那次发布写的 8 个单元写全覆盖之后，它的树表仍读得出（`r_tree_table_entries_after_the_second_mount=Ok(0)`），第三次挂载没有拒（第二节 E3）。**剩下一处**：`TreeTableHasNoEntries` 的文档写「不是坏盘」（`crates/singlefs-core/src/mounted_read.rs:195`「/// 树表 0 条：这一版还没发布过文件版本，一个文件都打不开（不是坏盘）。」），丢 0 号盘那一形里已确认的 txg 3 在幸存盘上读得出（判决 `research/prompts/m3-c694-r1-main-verification.md:26`「**打中**「报 `TreeTableHasNoEntries` 就是对的」」），它对甲 / 乙 / 丙一样成立，是同样打中所有候选，用户已定归 C683 |
| K2 行里 A（裸跳过）出局：「放过 36 行今天拒得对的」 | **出局站得住，但那 36 行不支撑它**；支撑它的是第一轮正推腿的丙，判决把丙挂在 K1 行、没挂到 A 上 | 36 行的结局：`research/prompts/m3-c694-r1-opus-output.md:133`「这一格对 A 是**没打中误放**」，`research/prompts/m3-c694-r1-opus-output.md:201`「- A 的误放：H2 两种瞬时读错 × 240 槽 k = 2、3、4 × 覆盖写 0..8 次，36 行放过，R 在环里时被写掉 0 个」。丙（= A，读不出一律放行）：`research/prompts/m3-c694-r1-sonnet-output.md:91`「TEMPORARY writes=1 W1=false W2=true … mount=Ok abandoned_unreadable=1 session_writes_done=1 account_unit_copies_overwritten_by_the_session=4/4 reds_after_the_fault_is_gone=[I-7.4]」，今天与甲都拒、盘不动（同文件 93 行）。再加用户定案：`.claude/kb/decisions/23-journal的角色与格式.md:390`「被抛弃根的账（树表或分配记录树）读不出时不跳过」（C393）。A 最强的一句辩护是单故障契约：今天的拒把一次瞬时读错放大成拒可写，A 放过它。但丙形要 R 的两个账单元（四份副本）整次挂载都读不出，写掉之后 R 能不能害到用户还要第二个故障（新实例的根全坏）才有机会（推的，没造盘面），所以是设计层违例（I-7.4 红），不是单故障丢数据；A 把一个故障的可用性损失换成两个故障的暴露，这个换法是 C393 用户定案不选的 |
| 「C 与 B 都做」下的两个辩护：只做 C、只做 B | **只做 C 站得住的范围**：新发生的损坏。**只做 B 站不住**：K1-2 不修。B 在 C 之上多给的只有两样：已经写坏的盘、C 的盲区之后的拒 | 只做 B：K1-2 同中（上面）。只做 C：C 不碰拒的那一支（`research/prompts/m3-c694-r2-sonnet-model/probe.diff` 里 C 的挂钩只加在 `rebuilt?` 之后），所以丙那一形的保护原样保留；E3 里 C 臂在第二次挂载里把被抛弃根的 20 个单元一个都没让写掉（今天 6 个、W1 / W2 同今天），第三次挂载没有拒。已写坏的盘上只有 B（或 A）放得了行；没查到有存量池要救的条款（`grep -rn '存量' CLAUDE.md .claude/rules .claude/kb/decisions.md` 无输出），所以 B 的必要性靠的是「注册的故障模型之外还有盘会坏」，用户已定都做（`research/prompts/m3-c694-r1-main-verification.md:42`「| C694 方向：C 与 B 都做 / 只做 C / 只做 B | 2026-10-07「C 与 B 都做」 |」） |
| K1-2 与 C 盲区谁都碰不到，只有恢复一侧补法碰得到 | **站得住**（读码推的，没实现） | `research/prompts/m3-c694-r1-opus-output.md:158`「我能想到碰得到的改法是恢复一侧的：所选根走不通、且系统配置没见证到它」。K4 回扫的结论补一条：补法在 13 行登记流上不走，只会改 oracle 对失败那一类的名字（第一节末条） |

「同样打中所有候选」的汇总：K1-2（打中 A、B、today、U2，只有 C 不中）；C 的盲区（打中 A、B、C、today、U2，只有恢复一侧补法碰得到）；C695 的 `TreeTableHasNoEntries` 文档与单盘幸存（打中甲、乙、丙，归 C683）。
够不着：判决 K2 行给 U2 的理由（它不分辨 U2）；判决 K2 行给 A 的 36 行证据（0 次误放）；判决点名的 D16 / D23 两处落点（漏了 D28 第九项）。C695 次序修法没有够不着的新处（怀疑的映射现算那一处量下来走不到）。

## 四、没做什么

- 没跑任何 checker 档的东西：13 条登记的崩溃枚举用例一条都没跑（E1 / E2 是它们在 harness 档的缩小复刻：只取段内前缀，不枚举段内子集，不枚举系统配置槽的撕裂态；43、47 的真流没复刻；48、49、50 与 41 没复刻）；没跑 `gate.sh`、门禁阶段、全量测试、变异表。
- E1 的 4 GiB 一档只有 6 段 × 12 步、没跑池级 checker；24 段 × 24 步的 4 GiB 全扫没做（单次挂载约 0.33 秒，几十分钟）。
- 没实现 U2 的改进形（卸载撞上拒时推空发布）、没实现恢复一侧补法、没实现攻方 B；候选开关只有第一轮正推腿的 W1 / W2 / W4 三个（`research/prompts/m3-c694-r1-sonnet-model/candidates.diff`），我的补丁在它上面加计数。
- 没复现拒本身（E3 没接会话覆盖写；拒的复现用第一轮的模型）；没造出「树表 0 条的被抛弃根账读不出」这一形的拒，只量到它的树表读得出（K1 那一节）。
- 没做 K2、K3 两格（攻方的）；没替主 agent 撤销或维持判决。
- 副本上量的数没有一个在入库装置上重做过，不进 kb。
- 没改 `crates/`、kb、规则、`.claude/`；只写了报告、模型目录和草稿目录。
- 第一轮正推腿报的 31 条既有用例结局（该报告第 95 行）与它 K4 那张 9 处读根环的表（该报告第 142 至 152 行）我没重跑、没重核；`.claude/gate.d/stage-inputs.tsv` 在我开工后被别处改过一次（第一节开头已写）。

## 五、什么现象会推翻

- 「B 的拒走不到任何登记行」：任何一条登记的崩溃枚举用例的运行日志里 `abandoned_roots_unreadable` 或 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot` 出现；我这边 E1 三档计数为 0。
- 「恢复补法在 13 行上不走」：任何一条登记流的计数行 `failed=` 非 0 且那个状态的系统配置轮换没落。
- 「C 在登记流上不改判定」：43 或 47 在 C 开着时有一个状态挂载被拒、或续做不到空、或挂载之后池级 checker 红；或 40 的挂钩在枚举全集（不只是段内前缀）里有一个非空状态。
- 「K1-2 同样打中 A / B / U2、不是废掉它们的理由」：任何一个 A、B、U2 的实现把第一轮那 4 个状态（`research/prompts/m3-c694-r1-opus-output.md:105`）变成 0。
- 「A 出局的依据是丙与 C393」：找到单故障（只有一次瞬时读错、没有第二个故障）下 A 造成用户可见丢数据而今天不丢的盘面，则 A 反过来该重判。
- 「第一轮判决漏了 D28 第九项」：C 的实现把隔离的槽另记一个集合、不进分配器的 `isolated_slots`（`crates/singlefs-core/src/admission.rs:583`「abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),」那一行读的数）。

## 六、判定一览（判决条目 / 辩方判定 / 证据位置）

| 判决条目（`research/prompts/m3-c694-r1-main-verification.md`） | 辩方判定 | 证据 |
|---|---|---|
| K1 采 U1、U2 后半句不成立 | 站得住 | 第三节第 1 行 |
| K2 行给 U2 的理由「碰不到 K1-2」 | **同样打中别的臂**（A、B、today 一样）：理由够不着 U2 一家；U2 出局另有两条（只覆盖一路、24 次空发布） | 第三节第 2 行 |
| K1「拒不护任何东西」 | 站得住（我复核了 33 / 36）；证据分单故障（checker 红）与双故障（探针）两档 | 第三节第 3、4 行，`research/prompts/m3-c694-r2-sonnet-model/check_r1_steps.py` |
| K1-2（取新实例那次挂载里写行根落盘之前断电） | 站得住；**同样打中 A、B、U2、today**，只有 C 不中；在注册的故障模型之内 | 第三节第 5 行 |
| C 的盲区（记录也读回全 0） | 同样打中 A、B、C、today、U2；只有恢复一侧补法碰得到；超出注入框架今天造的故障 | 第三节第 5、10 行 |
| 条款空白在 D16 已定项 1 那一行 | 站得住一半；**够不着 D28 已定项 1 第九项**（C 的隔离集不是「被抛弃根引用的槽」，准入式子直接读分配器的隔离数） | 第三节第 6 行 |
| C695 次序修法 | 站得住（它自己要修的 16 处）；怀疑的映射现算那一处量下来走不到；`TreeTableHasNoEntries` 文档与单盘幸存**同样打中甲、乙、丙**，归 C683 | 第三节第 7 行，第二节 E3 |
| A 出局 | 站得住；但判决 K2 行引的 36 行（0 次误放）**够不着**，该引丙与 C393 | 第三节第 8 行 |
| 只做 C / 只做 B（用户已定都做） | 只做 C 站得住于新发生的损坏、保留丙那一形的保护；只做 B 站不住（K1-2 不修）；B 在 C 之上多给的是已写坏的盘与 C 的盲区之后的拒 | 第三节第 9 行 |
| K4 回扫：同形只有择根 → 影子账看不见 → 重建分配器钉回收门槛这一条链 | 本轮 K4：C 走到 13 行里的 43、47（挂钩非空），B 一行都不走，补法 13 行都走 `recover` 但前提不成立；登记判定不变（量到 + 推的） | 第一、二节 |

## 七、模型与复跑

模型目录 `research/prompts/m3-c694-r2-sonnet-model/`：`probe.diff`、`tests/`（三条 harness 档探针）、`extra/`（一条额外探针，不在 `rerun.sh` 里）、`check_r1_steps.py`、`rerun.sh`、`compare.sh`、`expected-quick/`、`expected-full/`、`README.md`。
- **quick 档**：命令 `SINGLEFS_REPO=/home/fy5090/code/singlefs THREADS=6 bash research/prompts/m3-c694-r2-sonnet-model/rerun.sh <空目录>`。`expected-quick/` 来自空目录 `~/.local/share/singlefs/work/m3-c694-r2-sonnet/work-expected/` 里的一次从头跑；再在另一个开跑前不存在的空目录 `…/work-rerun/` 里从头跑一次，用时 `real 2m11.359s`，`bash research/prompts/m3-c694-r2-sonnet-model/compare.sh compare …/work-rerun/out/extracted research/prompts/m3-c694-r2-sonnet-model/expected-quick` 原样输出：`全同（4 个文件）`。
- **full 档**（quick 全部 + 384 / 256 槽各 24 段 × 24 步 + 4 GiB 6 段 × 12 步，六片并行）：同一条命令加 `PROFILE=full`，在开跑前不存在的空目录 `…/work-full/` 里从头跑通一次，`real 16m15.806s`，三批分片的退出码计数各 `6 0`；`expected-full/` 就是这一次的抽出结果，它与 `expected-quick/` 重合的四个文件逐个相同，`compare.sh compare` 对 `expected-full` 原样输出：`全同（7 个文件）`。full 档的数与我分批手跑三次的汇总数逐项相同（384 槽 168 / 2652，256 槽 168 / 2660 与 6 个准入翻转，4 GiB 26 / 408），所以三次独立跑出的结局行没有一行不同。
- 编译经 6G、跑经 4G 内存包装，线程上限 6，自己的编译目录；cargo 同一时刻只起一条（full 档的分片直接跑编出来的二进制，每个也经 4G 包装）。`compare.sh` 抽取时去掉 `seconds=` 字段、按字典序排序，不读 `SINGLEFS_REPO`。只跑 harness 档。
- 开工时机器上别人的 cargo：`ps` 看到别的会话的一条 `cargo test --no-fail-fast --release -p singlefs-checker-tier …`（经 32G 包装，线程 31），没有 qemu、vm-bench、e152、fio；我按「只有别的 cargo 在跑：加 nice 19 照常跑」做。
