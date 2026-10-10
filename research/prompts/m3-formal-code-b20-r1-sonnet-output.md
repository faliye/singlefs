# 第 20 批代码轮第一轮：云端正推（Sonnet）报告（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 F1 F2 F3 F5 V242u U6 C670 -->

腿：云端正推；格 K1、K3。代码读冻结副本 `/tmp/claude-1000/m3-formal-code-b20-r1/tree/`（对快照清单 `research/prompts/m3-formal-code-b20-r1-snapshot/crates-sha256.txt` 跑 `sha256sum -c --quiet` 无不符），下面 `crates/…` 的行号都是那棵树里的。
「量过」是在我自己拷出的副本上、harness 档、经内存包装（线程上限 10）跑出来的；checker 档一个没跑，checker 档那三条新用例与 6 行变异的结论只能是「推的」，接线处的引擎层事实在 harness 档用公开入口复做量过。
模型、脚本与原样输出在 `research/prompts/m3-formal-code-b20-r1-sonnet-model/`（`README.md` 有清单）。

先说三件事：
1. K3 点名 20-1 原补丁 `/tmp/claude-1000/m3-formal-20-1/patch/crates.patch`，而禁读清单列了 `/tmp/claude-1000/m3-formal-20-1/` 这个目录。两句话在这一个文件上冲突，我只读了这一份补丁文件（它是代码，不是实现员的报告），目录里别的文件没读、没列。要按禁读清单严格算，K3-4 一格作废，主 agent 另核。
2. 跑 harness 档时包装拒了第二条日常命令（同一会话里 Opus 腿在跑一条），我设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（同一轮两腿各用各的副本与编译目录）放行，没绕开包装；包装自己记峰值表（`research/scripts/memory-peaks.tsv`，已在 `.gitignore` 里）。
3. 背景材料里的 5999 行是合入后主仓的行号；冻结副本里同一处在 `crates/singlefs-checker-tier/src/crash_injection.rs` 第 5999 至 6003 行（调 `writable_mount_phase_of_each_write` 那一句），下面都用冻结副本的行号。

## K1　F1、F2、F5 的用例证不证得到点名的接错

### K1-1　F1：第二截用例的两条条件断言

判定：站住（推的；判官一层量过）。

代码做的是：
- 第二截把「所选那一版」交给判官（「version_landed_on_after_the_crash: report.effective_root,」（`crates/singlefs-checker-tier/src/crash_injection.rs:1384`））。判官在所选那一版是 None 时，判据复算为假：「if !criterion_holds_on_the_crash_image {」（`crates/singlefs-harness/src/mount_transparency.rs:1680`）把 `CriterionFalseWhenRecomputedOnTheCrashImage` 列进没满足的条件；C670 那一支拿不到分支，「if let Some(branch) = witnessed_counter_holds_on_a_crash_image_without_read_faults(」（`crates/singlefs-harness/src/mount_transparency.rs:1640`）不进，因为「let selected_version = selected_version?;」（`crates/singlefs-harness/src/mount_transparency.rs:1457`）直接交回 None。
- 用例新加的两条断言：「conditions_not_met.contains(&witness_criterion_true_without_read_faults),」（`crates/singlefs-checker-tier/src/crash_injection.rs:5761`）与「!conditions_not_met.contains(&criterion_false_when_recomputed),」（`crates/singlefs-checker-tier/src/crash_injection.rs:5765`）。条件文本取自第二截发现里实现那一半：「拒得不合法（{conditions_not_met:?}）：{member}」（`crates/singlefs-checker-tier/src/crash_injection.rs:2289`），用例的取法按这一句切（`split_once("拒得不合法（")`、`split_once("）：")`）。
- 已知红清单是空的：「pub const KNOWN_RED_FORMS: [KnownRedForm; 0] = [];」（`crates/singlefs-harness/src/history.rs:2038`），所以这条拒一定记成新发现，`new_findings` 里恰好一条（`settle` 一个崩溃态一截只记一条）。
- 变异行：「C670（20-1 F1）接线：第二截调用点把所选那一版交成 None」（`crates/mutations.tsv:2362`），原文两行在冻结副本里恰好命中一次，变异后的源码 `cargo check --tests` 过（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/check-mutants-summary.txt:1`）。

量过（harness 档，同一份「只落了覆盖写那次轮换」的盘面，调的就是第二截调的那个判官 `judge_writable_mount_refusal_of_a_crash_state`；输出 `research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log`）：
- 接对：「(合法,不合法)=(0,1) 条件文本="[WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterAboveTheLastRecordOfTheSelectedVersion)]"」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:10`），两条断言：「断言①含 C670 = true；断言②不含「复算为假」 = true」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:11`）。绿。
- 所选那一版交成 None：「(合法,不合法)=(0,1) 条件文本="[CriterionFalseWhenRecomputedOnTheCrashImage]"」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:12`），计数仍是 (0, 1)（所以原来的计数断言抓不到），两条新断言：「断言①含 C670 = false；断言②不含「复算为假」 = false」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:13`）。两条都红。
- 读故障恒交 Injected（旧变异行）：「(合法,不合法)=(1,0) 条件文本=""」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:14`），原来的计数断言就红。

推翻条件：在冻结副本上真跑这条 checker 档用例，基线上它红（比如发现文本被别的发现挤掉、`split_once` 切不出），或变异行 2362 下它仍绿。checker 档没跑，这是推的。

### K1-2　靶向二次崩溃摆法 `exactly_one_second_crash_point` 摆的是不是它文档说的态

判定：站住（推的；纯函数照原样抄出来量过）。

文档说的五点与代码逐句对：
1. 持久集合从挂载第 `segment_index` 段的第一个写起数，可以越过段尾：「let first_write_of_the_segment = mount_segments」（`crates/singlefs-checker-tier/src/crash_injection.rs:2745`）取段首；「persisted_within_the_segment: (0..writes_from_the_segment_on)」（`crates/singlefs-checker-tier/src/crash_injection.rs:2777`）给窗口里每个写一个标志，长度是 `writes_from_the_segment_on`，不是段长。
2. 扣下的按下标：「.map(|within_the_state| !withheld_among_them.contains(&within_the_state))」（`crates/singlefs-checker-tier/src/crash_injection.rs:2778`）。
3. 更早的段整段持久、窗口之后一个都没持久：落在第三截里「persisted_in_the_mount[..first_write_of_the_segment].fill(true);」（`crates/singlefs-checker-tier/src/crash_injection.rs:2949`），窗口按这一段标志拷进去，其余保持 `false`。
4. 崩在挂载的哪一段按第一个被扣下的写取：「phase: phase_of_each_write[first_write_of_the_segment + first_withheld],」（`crates/singlefs-checker-tier/src/crash_injection.rs:2780`），`first_withheld` 是扣下下标里的最小值（`crates/singlefs-checker-tier/src/crash_injection.rs:2761`）。
5. 写死的输入写错就 panic：段号不在、窗口 0、越过写表末尾（「writes_from_the_segment_on > 0」（`crates/singlefs-checker-tier/src/crash_injection.rs:2755`）、「<= phase_of_each_write.len(),」（`crates/singlefs-checker-tier/src/crash_injection.rs:2757`））、扣下的为空或越界。

量过（`research/prompts/m3-formal-code-b20-r1-sonnet-model/pure/pure_base.rs` 把函数体一个字不改抄出来，写表与切段取 harness 探针量到的真挂载写表：38 个写、9 段，写行根槽是写 18 独占段 3、其后两个轮换写 19、20 是段 4）：
- F2 用例的入参 (段 3，窗口 3，扣下 [0]) 摆出 persisted = [假, 真, 真]、相位取写 18 的：「F2 越过段尾：窗口 [18,19,20] 里 18 落段 3、19、20 是段 4 的写；持久 [false, true, true]」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/pure-base.out:7`）。
- 扣下 [1,2] 与无序的 [2,1]，相位表里单独把写 19 记成 Continuation，都取到 Continuation（取第一个被扣下的写）：（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/pure-base.out:8`、`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/pure-base.out:9`）。
- 六种坏输入（段号越界、窗口 0、越过写表末尾、扣下为空、扣下越界、一个在内一个越界）都 panic，恰好到写表末尾不 panic（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/pure-base.out:10`–`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/pure-base.out:16`）。
- 窗口等于段长、扣下 [0, 5] 时退化成段内真子集（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/pure-base.out:17`）。

推翻条件：真挂载的写表与我探针量到的不同（切段规则改了，根槽 FUA 不再独占一段），或第三截里把窗口拷进 `persisted_in_the_mount` 时长度对不上（现在 `copy_from_slice` 两边都是窗口长度）。

### K1-3　F2 用例经第三截调用点，三行接线变异各自红不红

判定：站住（推的；调用点那几行接线逐行在 harness 档用公开入口复做，量过）。

用例做了什么：第一次崩溃摆在覆盖写第一个写上、它一个写都没落（「writes_from_the_segment_on: 1,」（`crates/singlefs-checker-tier/src/crash_injection.rs:6027`）、「withheld_among_them: &[0],」（`crates/singlefs-checker-tier/src/crash_injection.rs:6028`）），第二截那次可写挂载写行那次发布上摆二次崩溃：根槽 FUA 扣下、其后紧跟的两槽轮换落了；经 `inject_crashes_into_history_drawing_second_crashes` 跑，断言第三截判过 1、不合法 1：「tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,」（`crates/singlefs-checker-tier/src/crash_injection.rs:6068`）、「tally.remount_refusals_with_the_witness_criterion_true_without_read_faults」（`crates/singlefs-checker-tier/src/crash_injection.rs:6069`），对 (1, 1)。

引擎层复做（`research/prompts/m3-formal-code-b20-r1-sonnet-model/f1f2/probe_f1f2.rs`：用 `RecordingBlockDevice` 录一次挂载，挂载写表按同一份几何切段，按用例的摆法造二次崩溃镜像，再按第三截判再挂被拒的同一个计数口径数，输出 `research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log`）：
- 摆出的态与用例文档说的一样：「PROBE F2 靶向二次崩溃：段 3（段内写 [18]），窗口 3 个写，扣下窗口里第 0 个」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:28`）；段 3 只有根槽 FUA、段 4 只有两个轮换写（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:21`、`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:22`），所以「越过段尾」是必须的。
- 接对：「[接对] 再挂被拒 → 判过 1、不合法 1」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:30`）。绿，与用例的 (1, 1) 一致。
- 行 2363（读故障恒交 Injected，对应调用点「read_faults: read_faults_in_the_first_crash_state,」（`crates/singlefs-checker-tier/src/crash_injection.rs:3011`））：「[读故障恒交 Injected（tsv 2363）] 再挂被拒 → 判过 1、不合法 0」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:31`）。第二个数 0，红在断言 (1, 1)。
- 行 2364（再挂的盘交成第一次崩溃的镜像）：「再挂做成（没被拒）→ 判过 0、不合法 0」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:32`）。再挂的目标就是这个字段：「second_crash_image.crash_image_materialized,」（`crates/singlefs-checker-tier/src/crash_injection.rs:3085`），它是起点那一版，挂得上。红。
- 行 2365（交成没叠这一段的基线 `second_base`）：「再挂做成（没被拒）→ 判过 0、不合法 0」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:33`）。红。
- 三行变异的原文在冻结副本里各恰好命中一次，变异后的源码 `cargo check --tests` 全过（只有「unused variable」警告）（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/check-mutants-summary.txt:2`–`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/check-mutants-summary.txt:4`）。

第三截里「所选那一版」这一根线（「version_landed_on_after_the_crash: report.effective_root,」（`crates/singlefs-checker-tier/src/crash_injection.rs:3005`））的两种接错，用例的抓法不同，没有变异行：
- 交成 None：「[所选那一版交成 None（没有变异行）] 再挂被拒 → 判过 1、不合法 0」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:34`），用例红，但没有变异行钉它。
- 交成第一次崩溃那一个：「所选那一版交成第一次崩溃那一个（没有变异行）」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:35`）仍是 (1, 1)：这一例里两个崩溃镜像上只读恢复都落在同一版（「两者相同 = true」，`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/f1f2-probe.log:29`），用例分不出，这一例分不出；`crates/mutations.tsv` 里没有变异行覆盖第 3005 行（下面「没钉住」第 1 点）。判决 K2 点名的是「三种接错」，这第四种不在点名里，记一笔。

推翻条件：在冻结副本上真跑这条用例，基线上它红，或 2363、2364、2365 任一行下它仍绿（要在 checker 档跑，这里推的）。

### K1-4　F5：录制流下标抽成函数、内联单测

判定：函数一层站住（量过）；调用点一层没钉住（推的）。

- 函数：「stream_indexes[first_write_of_the_segment + last_write_within_the_crash_window]」（`crates/singlefs-checker-tier/src/crash_injection.rs:1062`）。读故障认定拿的就是录制流下标：「&& operations_before_the_step <= last_stream_index_of_the_crash_window;」（`crates/singlefs-checker-tier/src/crash_injection.rs:1036`），不是写表下标。
- 单测给的写表：「let stream_indexes = [0, 1, 3, 4, 6, 9];」（`crates/singlefs-checker-tier/src/crash_injection.rs:5496`），三个调用 (2, 3)、(4, 2)、(3, 1)：「stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 2, 3),」（`crates/singlefs-checker-tier/src/crash_injection.rs:5498`）等三行。三个调用要的 6、9、4 与写表下标 4、5、3 全不同。
- 量过：原样抄出的函数跑三个断言全绿（「F5 单测整体：绿」，`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/pure-base.out:4`）；换上变异行 2366（「C670（20-1 F5）读故障认定的崩溃窗口末尾交成写表下标（不是录制流下标）」（`crates/mutations.tsv:2366`））三个断言全红，得 4、5、3：「F5 单测整体：红」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/pure-row2366.out:4`）。变异后源码编得过（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/check-mutants-summary.txt:5`）。
- 调用点一层：「stream_index_of_the_last_write_of_the_crash_window(」（`crates/singlefs-checker-tier/src/crash_injection.rs:1339`）的三个实参（`&stream_indexes`、`first_write_of_the_segment`、`writes_in_the_segment`）接错，单测不经过，改成别的实参这条单测仍绿。b18 腿量过的「随机 60 段注入过读故障的态 0 个」仍是今天的状态（`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2 那一行写的就是它），所以调用点实参接错今天在任何跑法上观测不到。判决 K2 要的是「抽成函数单独钉」，这一点做到了；调用点实参没钉是这条路的固有边界，不是写漏。

推翻条件：函数一层，副本上编出的单测在基线上红，或变异行 2366 下仍绿；调用点一层，找到一段注入过读故障的历史（`operation_reads_under_injected_faults` 为真的步）上改实参就红的用例。

### K1-5　薄壳改造有没有让原入口行为变

判定：没变（推的；逐行对）。

- 薄壳只转调：「inject_crashes_into_history_drawing_second_crashes(」（`crates/singlefs-checker-tier/src/crash_injection.rs:1096`）、第二截摆法实参固定为「SecondCrashPointDraw::OneInEachMountPhaseBySeed,」（`crates/singlefs-checker-tier/src/crash_injection.rs:1100`）；签名（`crates/singlefs-checker-tier/src/crash_injection.rs:1090`）与 `#[must_use]` 不变，战役与所有老调用（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs` 等）不用改。
- 老摆法分支的调用与改前逐字相同：「draw_second_crash_points(seed, crash_point, mount_segments, &phase_of_each_write)」（`crates/singlefs-checker-tier/src/crash_injection.rs:2910`）；种子的三项异或（`^ SECOND_CRASH_POINT_SEED_SALT`（`crates/singlefs-checker-tier/src/crash_injection.rs:2794`））不在补丁里。
- 新的字段「second_crash_draw: SecondCrashPointDraw,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2867`）只在 `SecondCrashInputs` 里加、解构里取出（补丁里的这一块在合入后被拆成两块，逐行比过，见 K3-4 第 3 点）；老入口永远传 `OneInEachMountPhaseBySeed`，计数口径没有第二处会读它。

推翻条件：在同一批种子上，改前与改后的 `inject_crashes_into_history` 交回的 `HistoryCrashInjection` 有一处不同（checker 档跑，这里没跑）。

### K1 没钉住的几处（不判决，列给主 agent）

1. 第三截「所选那一版」一线（`crates/singlefs-checker-tier/src/crash_injection.rs:3005`）交成第一次崩溃那一个：F2 用例分不出（K1-3）。`crates/mutations.tsv` 里没有哪一行的原文覆盖第 3005 行（我按每行原文在冻结副本里的命中位置核过，覆盖第 3004 行的只有 2364、2365）。
2. F5 调用点三个实参（K1-4）。

## K3　18-2 两行变异、注释与常量、重定基

### K3-1　「F 不动」「会话关」两行变异各自只让点名的那条断言红

判定：站住（量过，harness 档）。

- 两行：「C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型答树表 0 条那一版上卸载之后会话照旧开着」（`crates/mutations.tsv:2368`）改的是「answer.session_after_success = ModelSessionAfterSuccess::Closed;」（`crates/singlefs-harness/src/model.rs:2525`）；「C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型关会话时把现行那一版那条根带的 F 记成它的 txg」（`crates/mutations.tsv:2369`）改的是「ModelSessionAfterSuccess::Closed => self.session = None,」（`crates/singlefs-harness/src/model.rs:3140`）。两行原文各恰好命中一次（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-anchors.txt:1`、`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-anchors.txt:2`）。
- 条款与实现一致：「一个字节都不写、F 不动，答零条根、会话关」（`crates/singlefs-harness/src/model.rs:2496`）；「树表 0 条那一版上正常卸载一个字节都不写、F 不动」（`.claude/kb/decisions/16-发布语义.md:78`）。
- 用例里对应的两条断言：会话关「!pool.model.has_writable_session(),」（`crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs:386`）、F 不动「pool.model.effective_rollback_floor(),」（`crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs:390`，消息「"树表 0 条的一版上卸载不抬 F"」在第 392 行）。
- 量法：在副本里把 `model.rs` 换成变异后的源码（`research/prompts/m3-formal-code-b20-r1-sonnet-model/anchor_check.py --emit`），各跑两样——真用例（看第一条红的是哪一句），和我加的探针（`research/prompts/m3-formal-code-b20-r1-sonnet-model/k3/probe_k3.rs`：同一条用例的每条断言各自求值、打印真假、不 panic，免得第一条红盖住后面的）。
  - 基线：「test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.64s」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-baseline-real-test.log:13`），探针五项全真（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-baseline-probe.log:10`–`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-baseline-probe.log:14`）。
  - 行 2368（会话照旧开着）：真用例 panic 在会话关那条（panic 位置第 385 行是 `assert!(` 起头那一行）：「卸载之后这个进程没有可写会话」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2368-real-test.log:11`），同一文件里别的四条用例绿（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2368-real-test.log:22`）。探针：「PROBE A2 会话关（!has_writable_session） = false」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2368-probe.log:12`），而「PROBE A3 F 不动（卸载后 F == 卸载前 F） = true（前 ModelCheckpointTxg(0)，后 ModelCheckpointTxg(0)）」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2368-probe.log:13`）；盘上不变、对拍 (1, 0)、F 初值 0 三条仍真（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2368-probe.log:10`、`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2368-probe.log:11`、`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2368-probe.log:14`）。只红「会话关」。
  - 行 2369（关会话时把现行根的 F 记成它的 txg）：真用例 panic 在 F 不动那条：「assertion `left == right` failed: 树表 0 条的一版上卸载不抬 F」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2369-real-test.log:11`）、左 2 右 0（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2369-real-test.log:12`、`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2369-real-test.log:13`）。探针：「PROBE A2 会话关（!has_writable_session） = true」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2369-probe.log:12`）、「PROBE A3 F 不动（卸载后 F == 卸载前 F） = false（前 ModelCheckpointTxg(0)，后 ModelCheckpointTxg(2)）」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/k3-row2369-probe.log:13`），别的三条仍真。只红「F 不动」。
- 范围说明：「只让点名的那条断言红」量的是这一条用例里的断言；这两行变异让别的测试文件红不红我没量（变异表的判法是「抓到」，不要求只红一条）。

推翻条件：在入库的装置上经 `research/scripts/mutate.sh` 重跑这两行，红的断言不是上面那一条；或这条用例里的某一条别的断言在变异下也红。

### K3-2　`crates/singlefs-checker/src/image.rs` 的 I-5.5 注释与 `.claude/kb/invariants.md` 的 I-5.5 一致

判定：一致（逐句对）。

| 项 | `image.rs` 注释 | `invariants.md` 第 199 行 I-5.5 |
|---|---|---|
| 认哪一版 | 「I-5.5：从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条诞生 txg 都等于这一版的 `checkpoint_txg`）引用的数据单元里，」（`crates/singlefs-checker/src/image.rs:198`） | 「树表非空，且种类 1（extent 树）条目的树 ID 等于 65536」（`.claude/kb/invariants.md:199`），再加每条条目的诞生 txg 等于 V 的 `checkpoint_txg` |
| 比什么 | 「每块盘上落点槽号最小的那一个等于这一版分配记录照 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、」（`crates/singlefs-checker/src/image.rs:199`） | 「槽号等于这一版分配记录里 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、两槽都空、不在开放段里）」（`.claude/kb/invariants.md:199`） |
| U + 4 与回收 | 「两槽都空、不在开放段里；新池新建文件那一次是单元区起始槽号 U + 4，回收之后不再是）（D3（空间分配） 已定项 8 射程，判决」（`crates/singlefs-checker/src/image.rs:200`） | 「新池新建文件那一次是 U + 4」「回收之后（例如 mkfs 的实例表 U、U + 1 那两槽被回收）不再是 U + 4」（`.claude/kb/invariants.md:199`） |
| 依据 | 注释末句点名 b17 判决第二节 K2（按分配记录现算） | 「判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2」（`.claude/kb/invariants.md:199`） |

D3 已定项 8 第 1 条原文是「起点槽号最小、起点 32768 对齐、跨度内两槽都空、不在任何开放的聚簇段里」（`.claude/kb/decisions/03-空间分配.md:163`），后面还有「整跨度都过分配准入合取」一截；注释与 kb 行都压成四样、没带准入那一截，两边压得一样，不构成不一致（kb 行自己写了已知限度）。注释比 kb 行少的一句是「也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件」，注释的「从 mkfs 水位第一次建树的那一版」把它含在里面。池级 checker 的实现注释也同口径（`crates/singlefs-checker/src/walk.rs:4166` 起，不写死 U + 4）。

推翻条件：kb 行 I-5.5 的哪一个字改了而注释没跟，或实现（`walk.rs` 的判法）与这句注释对不上（我没跑池级 checker 的新用例，只核了文字）。

### K3-3　wgsl 删掉的常量，全仓没有别的引用

判定：站住（grep 层；GPU 内核编译没跑，推的）。

- 删掉的是 `FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START`（背景材料里的 diff 删了它与上面一行注释）。冻结副本里 `crates/` 下 `.wgsl` 命中 0 个；`crates/` 下只剩一处同名的 Rust 常量，在 harness 测试里自己定义自己用：「const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u64 = 4;」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:54`），它上面的注释写的就是「测试这边另写一份」（同一文件第 53 行），不引用那个 wgsl 常量。仓里别处的命中是 `records/` 的一条历史记录与 `research/prompts/` 下冻结的材料。完整命中表：`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/const-refs.txt`。
- 三份 wgsl 在主仓与冻结副本里逐字节相同（同一文件）。
- 内核源码是几份 wgsl 用 `include_str!` 接起来的（「pub const GPU_JUDGE_CHECKER_COMMON_SOURCE: &str = include_str!("crash_judge_checker_common.wgsl");」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:28`）），所以删一个常量只有被别的 wgsl 引用才会编不过；零引用。I-5.5 认法 ① 还在用的水位常量没动：「const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u32 = 65536u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:161`）、「u64_of(TREE_IDENTIFIER_WATERMARK_AT_MKFS)」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1878`）。
- 删掉的那条注释点名的 `walk.rs` 同名常量，冻结副本里 `crates/singlefs-checker/src/walk.rs` 已经没有它（19-1 起判法按分配记录现算，见 K3-2 末句），所以这个 wgsl 常量是悬空的、删得对。
- `crates/mutations.tsv` 里指向 `crash_judge_checker_common.wgsl` 的 3 行原文都恰好命中一次（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/anchor-all.out`），没有哪一行锚在被删的那几行上。

推翻条件：某份没在 `crates/` 下的 WGSL 源（或运行时拼出来的内核源）按名字引用它；或真起一次 GPU 内核编译（naga 解析）报未声明的标识符——checker 档，这里没跑。

### K3-4　主 agent 手工接的两段与第 5999 行，与 20-1 原补丁意图一致、没丢根环那条线的东西

判定：一致、没丢（量过：补丁对 HEAD 复做、逐行比；根环那一边只能证到「碰过的地方都在」，见末尾的限度）。
读的是 `/tmp/claude-1000/m3-formal-20-1/patch/crates.patch` 这一份（见开头第 1 条）。

做法与结果（脚本 `research/prompts/m3-formal-code-b20-r1-sonnet-model/patch_vs_merged.sh`、`research/prompts/m3-formal-code-b20-r1-sonnet-model/hunk_context_diff.py`、`research/prompts/m3-formal-code-b20-r1-sonnet-model/anchor_all.py`，输出在 `outputs/`）：
1. 把补丁打到 git HEAD 上，得「HEAD + 20-1」：补丁自己写的底版 blob 与 HEAD 里那份相同、打完与补丁写的结果 blob 相同：「补丁里 crash_injection.rs 的底版 blob：index bf66bc6..06a0c10 100644；HEAD 里那份的 blob：bf66bc608dd88d841bb6a536a45d9a12492d3991」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/patch-vs-merged.out:1`）、「打完之后 crash_injection.rs 的 blob：06a0c10e166f9bcf4b03927d3ce0551cb7ffec7e」（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/patch-vs-merged.out:5`）。补丁没有手工改过。
2. 「HEAD + 20-1」对合入后冻结副本逐行 diff，被合入删掉的行共 18 + 2 + 1：全是别的批（根环那条线的相位表改写、`ALL` 数组、`IMPLEMENTED_INVARIANTS` 长度、wgsl 的 `SCRATCH_END`）改写的行，没有一行是 20-1 自己加的（唯一的字面撞车是 `    ) {` 一行，它是通用的括号行，合入后文件里它的出现次数仍不少于补丁加的次数）；20-1 加的不同非空行（crash_injection.rs 377 条、image.rs 3 条）在合入后出现次数都不少于补丁加的次数、20-1 删掉的行没有一行回来（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/patch-vs-merged.out:30`–`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/patch-vs-merged.out:32`）。
3. 补丁 23 块与背景材料 diff 的对应块逐行比（含上下文行）：20-1 自己加删的行 0 处不同；有差别的只有三种——(a) 导入行：合入后文件里有根环那条线加的「use singlefs_core::mount::{mount_writable_with_space_admission, MountError, MountOutput};」（`crates/singlefs-checker-tier/src/crash_injection.rs:46`）、`instance_table_sweep` 那组导入（第 42 行起），也有 20-1 的「use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;」（`crates/singlefs-checker-tier/src/crash_injection.rs:47`）与「publish_first_file, publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter,」（`crates/singlefs-checker-tier/src/crash_injection.rs:53`）后面的 `TransactionOutput,`，两边都在；(b) 解构入参：根环加的 `publishes_before_the_instance_table_sweep,`（`crates/singlefs-checker-tier/src/crash_injection.rs:2892`）与 20-1 加的「second_crash_draw,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2900`）并存，`SecondCrashInputs` 里两个字段并存（`crates/singlefs-checker-tier/src/crash_injection.rs:2851`、`crates/singlefs-checker-tier/src/crash_injection.rs:2867`），调用点也两个都传（`crates/singlefs-checker-tier/src/crash_injection.rs:1441`–`crates/singlefs-checker-tier/src/crash_injection.rs:1447`）；(c) 第 5999 行：补丁原写的是两个实参，合入后多一行「mount_run.publishes_before_the_instance_table_sweep,」（`crates/singlefs-checker-tier/src/crash_injection.rs:6002`），函数现在三个形参（「publishes_before_the_instance_table_sweep: Option<usize>,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2609`）），挂载交回值里这一项就是 `Option<usize>`（`crates/singlefs-checker-tier/src/crash_injection.rs:1748`），与第三截里同一个函数的正式调用（第 2903 至 2907 行，三个实参）同形。其余的上下文差别（`.collect()` 对 `by_phase`、`}` 对 `})`、image.rs 里多出来的 I-9.28 常量）都是根环那条线改过的行，在合入后文件里原样在。
4. `crates/mutations.tsv` 里指向合入后 crash_injection.rs 的 66 行（含根环那条线的）原文各恰好命中一次，`crash_judge_checker_common.wgsl` 3 行、`image.rs` 46 行、`model.rs` 50 行、`mount.rs` 252 行也都是一次（`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/anchor-all.out:1`、`research/prompts/m3-formal-code-b20-r1-sonnet-model/outputs/anchor-all.out:6`）。
5. 根环那条线自己的检查文件用到的 `crash_injection` 项都在：`writes_of_the_instance_table_sweep_phases`、`WritableMountPhase`（`crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs:33`）。

限度：我没拿到合入前（只含根环那条线）的 crash_injection.rs，所以「根环那条线没丢东西」靠的是：补丁的每块上下文与合入后逐行比过、补丁以外的删行全是别批的、66 条锚点全命中。要证到「根环那条线的每一行都在」，要有合入前备份对合入后的 diff（背景材料那份就是），我只核了它与补丁的关系，没核它与备份的关系。

推翻条件：合入前备份与合入后文件之间，除补丁的 513 条加删行外还有一行差别（我没有备份，没法查）；或 20-1 的任一块在合入后文件里行序变了（我比的是行集合，不是行序，行序由编译与 clippy 通过间接保证，是主 agent 的观测，不是我的）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1　F1 两条条件断言 | 站住（推的；判官层量过） | 所选那一版交 None 时计数仍 (0, 1)、判官只列「复算为假」，两条新断言都红；接对时都绿 |
| K1-2　`exactly_one_second_crash_point` 摆的态 | 站住（推的；纯函数量过） | 窗口从段首起数、可越段尾、扣下按下标、相位取第一个被扣下的写、六种坏输入 panic，都与文档一致 |
| K1-3　F2 用例与三行接线变异 | 站住（推的；接线层复做量过） | 接对 (1, 1)；2363 得 (1, 0)、2364 与 2365 得 (0, 0)，三行都红；第三截「所选那一版」一线没有变异行，且在这一例上与第一次崩溃那一个等价 |
| K1-4　F5 抽函数与单测 | 函数一层站住（量过）；调用点实参没钉 | 单测三条在基线绿、变异 2366 下三条全红；调用点三个实参接错单测与任何跑法都观测不到（今天没有注入读故障的态） |
| K1-5　薄壳 | 没变（推的） | 老入口只传 `OneInEachMountPhaseBySeed`，老分支调用与种子逐字不动 |
| K3-1　18-2 两行变异 | 站住（量过） | 2368 只红「会话关」，2369 只红「F 不动」，别的断言与同文件别的四条用例不动 |
| K3-2　`image.rs` 注释对 I-5.5 | 一致 | 四项逐句对上，D3 已定项 8 第 1 条的准入一截两边都没带 |
| K3-3　wgsl 常量 | 站住（grep；内核编译没跑） | `crates/` 下 wgsl 零引用，同名 Rust 常量是 harness 测试自己的 |
| K3-4　手工接的两段与 5999 行 | 一致、没丢（量过；根环一边有限度） | 补丁对 HEAD 复做 blob 对得上，补丁的加删行合入后一条不少，手工接的三处与补丁意图同形 |

## 没做什么

- checker 档一个没跑：三条新用例（F1 加的断言、F2、F5、发布臂固定历史用例）与 6 行 checker 档变异（`crates/mutations.tsv` 第 2362 至 2367 行）的基线绿、变异红都是推的；我只在 harness 档复做了接线处的引擎层事实，并核了六行变异的原文各命中一次、变异后源码 `cargo check --tests` 过。
- 没经 `research/scripts/mutate.sh` 在入库的装置上跑变异表；K3-1 的两行是在副本上把变异后的 `model.rs` 换进去跑真用例与探针，数不进 kb（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」那一条：副本上的数要在入库装置上重做才能引）。
- K2（发布臂改法）不归我，没判；`first_file_is_a_live_record` 只在 K1 里作为「编得过」出现。
- F3（二次崩溃种子混进第一次崩溃那个子集）、V242u 那一行的抽样口径没碰。
- 没读 Opus 腿的报告与模型目录、`/tmp/claude-1000/m3-formal-code-b20-r1-opus/`；没读 `/tmp/claude-1000/m3-formal-20-1/` 里除 `patch/crates.patch` 以外的任何文件。
- 没替主 agent 采纳或出判决。

## 要主 agent 跑的命令（checker 档，我不跑）

工作目录仓根，环境变量前缀 `SINGLEFS_HEAVY_TESTS=commit`，内存上限 8G、线程 10，命令都带 `bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10`：
1. 基线（四条各应 `1 passed`）：`cargo test -p singlefs-checker-tier --lib -- crash_injection::tests::<名>`，名依次是：`a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash`（F1 加了断言）、`a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site`（F2）、`the_end_of_a_crash_window_is_named_by_its_stream_index_and_not_by_its_write_index`（F5）、`the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published`（K2 的，顺带）。
2. 变异：`crates/mutations.tsv` 第 2362 至 2367 行，用 `research/scripts/mutate.sh` 带那几行，判每一行红的是第 5 列那条用例。

跑完看：基线四行 `test result: ok. 1 passed`；变异每行判「抓到」。F1 那行还要看红的是不是条件那两条断言（报错消息里有「拒得不合法要由 C670 那一条」或「不许列出 CriterionFalseWhenRecomputedOnTheCrashImage」）。

## 临时目录

- `/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/`：草稿目录（各次跑的日志与我的小模型源码，约 9 MB），不删，主 agent 要看原始日志时在；里面没有编译目录与仓副本。
- 副本 `/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/work/` 与编译目录 `/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/target/`（21 GB）、比对用的 `cmp/`、`cmp2/`、变异源码 `check-mutants/`：都已删。重做用 `research/prompts/m3-formal-code-b20-r1-sonnet-model/setup_work.sh` 重建副本。
