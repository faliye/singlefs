# 崩溃点重放拆成「跑数据 / 对比数据」两段：设计轮第二轮，辩方腿报告（2026-10-10）

立场：辩方。复核第一轮判决（`research/prompts/m3-run-compare-split-r1-main-verification.md`）第二节逐格、第三节七条修订、第五节四行候选；替被判输的一边（读故障那一标的正推、RefFS 那一格的攻方、随机历史 60 步）找站得住的理由，也替被第三节替掉的原形态找判决有没有冤枉。不替主 agent 撤销或维持判决。

## 0. 读法

- 证据标签：**[读]** 读代码得出；**[算]** 算出来的，算式在文中；**[推]** 推的；**[没量]**。
- 引用写法：「原文」（`路径:行号`）的路径是主工作区 `/home/fy5090/code/singlefs`，这些文件在 E5 副本（`~/.local/share/singlefs/work/m3-merge-impl/e5/repo`）里逐字节相同，`diff -q` 比过：`crates/singlefs-harness/src/{history,model,reference_filesystem_translation}.rs`、`crates/singlefs-reference-filesystem/src/lib.rs`、`crates/singlefs-checker-tier/src/{crash_facts,crash_verify_gpu}.rs`、`crates/singlefs-checker-tier/Cargo.toml`。
- 「E5 副本」前缀的行号来自被补丁动过或新建的文件（`crash_injection.rs`、`crash_identity.rs`、`judging_closure.rs`、`crash_given_states.rs`、`crash_amplification.rs`、`crash.rs`、`crash_judge_tables.rs`、`tests/crash_injection_campaign.rs`），这些行号我都在 E5 副本里 `grep -nF` 现取，不带引号原文（主工作区上核不了）。
- 没读禁读清单里的东西。没编译、没跑测试；唯一跑的是 `research/prompts/m3-run-compare-split-r2-sonnet-model/rh-probe-stats.sh`（awk，读主 agent 的探针日志）。
- 规则条款写 `文件@小节标题`；`.claude/rules/verification.md` 的标题行号是现取的（`## checker 档自己分快档与全量` 在第 30 行）。

## 1. 先说结论

1. 第二节逐格：**没有一格被判错到该翻**。三处特别要看的：
   - 读故障那一标：「攻方打中」站得住；「正推那一行没有出处」对正推 Q2 表那一行成立，但正推第 149 行已有「跑数据段录标」的口径，只是没延伸到拒绝合法性这一支。**更要紧的是判决给的存法不够**：「步种序列与每步之后的流长」还差「写表下标 → 录制流下标」的映射（屏障位置），照它存，对比段仍算不出这一标；判决括号里的另一条（直接存每个崩溃态一位）才是够用的，应当当主条（第 3.1 节）。
   - RefFS 那一格：「攻方说重了」站得住。`lib.rs:440` 是公开口，而且 harness 翻译层今天就有一个事件变体专门调它（`reference_filesystem_translation.rs:507`、`:540`），攻方说「要改 RefFS 的口」是没看到这个变体。攻方「历史内一票住在跑数据段」那一半打中，不受影响（第 3.2 节）。
   - 随机历史约 60 步封顶：方向对、量级对，**不是 60 这个数撑得住，是主 agent 自己的探针日志撑得住**：24 步历史的写表最长 746 次写（31.08 次写/步），按这个速率 65.9 步满 2048 位；均值 20.05 次写/步，102.2 步。枚举写表还要加撕裂镜像与重放，所以真封顶只会更低（没量）。这个封顶来自 E5 的给定状态表设计，原形态与修订形态同样有，不是两段拆法造成的；快档与读故障档都是 24 步（第 3.3 节）。
2. 第三节七条修订：七条都没有「只打中原形态、放过别的候选」的毛病。有四处值得补（第 4 节）：读故障标改成每崩溃态一位；`12`、`14` 并成一把「历史族」；新内核的版本绑定有一条同类先例（`gpu_verifier_digest`），自证的「要么在闭包里、要么不判对错」两分法会把现有的 `crash_verify_gpu.rs` 误判；第五节第 2 行漏了一条不涉及挂钟的候选（红步之后模型依赖的判不再问），它在 `history.rs:3878` 有同形先例。
3. 第五节第 1 行（快档怎么读库）漏了一条候选：按文件挂特性，走快档今天就有的「第二趟」，不必整条带 `verdict-store`（第 4 节 S5）。
4. 我造不出「拆法本身站不住」的输入；也没替判决里没被攻过的零轮形态担保。

## 2. 第一问：第二节逐格

判定：站得住 / 够不着 / 同样打中别的臂。「够不着」= 判决引的代码说的不是那件事。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 历史内 RefFS 恢复一票：两腿一致坐实 | 站得住 | 翻译层 `apply_event` 里：「match model.crash_and_recover(crash, read_out.clone()) {」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:532`）判成员，不在就走 `stopped_verdict_of_a_refused_recovery` 记红停翻译；之后每个崩溃时刻在「if let Some(stopped) = self」（同文件 `:983`）那一支直接交回这一句红 [读] |
| 　其中「拆法用 `lib.rs:440` 现成的口，攻方说重了」 | **站得住，攻方被冤枉的只有这一句** | 「pub fn restart_after_a_recovery_that_abandoned_the_newest_root(」（`crates/singlefs-reference-filesystem/src/lib.rs:440`）公开，里面「只核 hi 判不判得了；读出的内容在不在允许集合里不判」（同文件 `:445`）后调私有 `restart_from_the_recovered_state`（`:451`）。翻译层已有对应事件：「RestartAfterARecoveryThatAbandonedTheNewestRoot(VisibleState),」（`reference_filesystem_translation.rs:507`）与调用「.restart_after_a_recovery_that_abandoned_the_newest_root(crash, read_out.clone())」（`:540`）。攻方 A1/B3 的「漏」打中不变，F2 的「RefFS 包没人管」随之落空 [读] |
| 历史执行器的 harness 判（分配记录的代、冷启动读回报错）：一致坐实 | 站得住 | 「fn allocation_generation_judgement(」（`crates/singlefs-harness/src/history.rs:3242`）、「pub fn harness_judgement_of_outcome(outcome: &StepOutcome) -> Option<HarnessJudgement> {」（`:1926`）；判红同样停历史：「if !violations.is_empty() || harness_judgement.is_some() || model_disagreement.is_some()」（`:5897`）。崩溃注入里每步池级 checker 是 Skipped：E5 副本 `tests/crash_injection_campaign.rs:192` `per_step_checker: PerStepChecker::Skipped` [读]。**同样打中别的臂**：这两样与模型比在同一处停历史，拆开后「红步之后」的定义对它们同样要给（判决第 7 条已按依不依赖模型分，它们属不依赖模型的一类，`history.rs:3878` 注释「模型对不上之后不再问模型（分配代照判）」就是这个分法的现成形态）|
| B0 断言、拒后盘整盘相等：坐实 | 站得住，B0 的归属有一条加固理由 | B0 函数 E5 副本 `crash_amplification.rs:626`，调用 `:703`，文档 `:622` 写明它防的是「判定不同的两条流会共用块键」。拆开后录入行也按块键存，B0 防的东西更重要，留在 CPU、不依赖 GPU 开关，判决归法对。整盘相等在 E5 副本 `crash_injection.rs:3128` [读] |
| 读故障那一标：攻方打中、正推没有出处 | 站得住（有两处补） | 第 3.1 节 |
| `14` 的键：两条都坐实合成一条 | 站得住 | 正推「`14` 缺模型派生常量」：`capacity_wall_is_permitted` 在 `model.rs:2835` 起，读 `unit_area_slots_per_device()`（`:2854`），`judge_effect` 读 `effective_rollback_floor()`（`:3036`）[读]。攻方「摆不出崩溃状态的历史无键」：E5 副本 `crash_injection.rs:1480` `tally.histories_without_any_crash_point = 1;`，该历史没有给定状态表，但每步模型比照样做 [读]；表名里的 `seed=` 在 E5 副本 `crash_injection.rs:2075`，所以「给定状态表块键记它属于哪一把历史键」只差项名与规模两项 |
| RefFS 允许集合：一致，第二次崩溃 = {R1} | 站得住 | 第二次崩溃：「let crash = restarted.crash_after_every_operation_returned();」（`reference_filesystem_translation.rs:925`），重起头的 RefFS 操作序列已清空（`lib.rs` `restart_from_the_recovered_state` 里 `operations_since_the_last_recovery.clear()`），`lower_bound` 取 `StateIndex(0)`，循环不转，集合只剩起点 = R1 [读，我读了 `lib.rs:208-250`、`:286-300`] |
| 录入版本四处互补 | 站得住，但「新内核不在闭包」不是拆法才有的洞 | 第 4 节 S3：`crash_verify_gpu.rs` 今天就是一个不在任何闭包里的 GPU 内核，靠自带摘要进自己表的键 |
| 体量：双机拷库是闸 | 站得住 | 「cp -a -- "$local_library" "$library_for_peer"」（`scripts/crash-amplification-two-hosts.sh:1040`）每项一份整库；「FULL_EXPAND_UP_TO=30」（`scripts/crash-amplification.sh:222`）[读]。攻方「压缩比 5 时 3.3 项满」依赖假设的压缩比与 160 字节，两者都没量：判决写的是条件句，没越界 |
| CPU 代价：量级一致、要量 | 站得住 | 两腿都是旧数外推，判决没引数 |
| GPU 展开：P14 单独成内核、采攻方的量 | 站得住，量的是下界 | 攻方的 62660 / 387189 是探针（只摆调用点、判据本体一行没写，攻方报告自己写了），所以「并进恢复内核到顶」是下界得出的结论，「单独成内核」无论本体多大都不违预算；正推估 7–25 万，现有内核 33–40 万已编得过，单独成内核落在已知编得过的区间里。M4 才定是一个还是两个内核 |
| 分界：模型比与走焊在一起 | 站得住 | 「/// 拿实现的结局与模型的答案比；对得上就按模型自己算的结果往前走（不按实现交回的），对不上交回哪一格。」（`crates/singlefs-harness/src/model.rs:2870`）[读] |
| 红步之后：按依不依赖模型分开报 | 站得住，加一句不要丢 | 用户弹窗原话是「照判，报告里标在红步之后」。判决的分法是在「标」之上再分两类，不是换掉它：红步之后的每一条判定都仍然带「红步之后」这一标，只是依赖模型的不进总数、不登记新发现。写进实现规格时要保留这一标，否则只剩第二类的「单列」，第一类（只读镜像、RefFS）会失去用户要的标记 [推] |
| 旧引擎用户 17 份、3 份测旧引擎自己的功能 | 站得住 | 我数过：E5 副本 `tests/` 下 `grep -lF 'enumerate_layer0' *.rs \| wc -l` 得 17，主工作区同为 17 [读，命令原样] |
| 快档构建：拆开后要带 `verdict-store` | **够不着一半**：不是非要「整条带」 | 第 4 节 S5。判决引的 Cargo.toml 事实对：「verdict-store = ["dep:rocksdb"]」（`crates/singlefs-checker-tier/Cargo.toml:27`）默认不编；但快档今天已经分两趟，第二趟带特性（「再一趟 `cargo test --release -p singlefs-checker-tier --features」（`.claude/rules/verification.md:34`）），E5 副本 `tests/` 下 `grep -rln 'verdict-store' tests | wc -l` 得 19，其中 `grep -rln 'cfg(feature = "verdict-store")' tests | wc -l` 得 14 份文件按特性挂了（含 `crash_enumeration_multi_record_publish_stream.rs`、`crash_injection_campaign.rs` 这样的流水线与随机历史用例）。所以那一格的候选不止甲、乙两条 |
| 随机历史开 GPU 约 60 步封顶：零轮、推的 | 站得住（方向与量级），60 这个数的来路不对 | 第 3.3 节 |
| J6 跳过时全 0：不判 3.1，另记环境检测 | 站得住 | 攻方自己写的「不分辨臂」：J6 跳过时今天就抓不到，P7 不新增；判决把它从 3.1 摘出去、另记一条开跑前检测，没越界。它不在第三节七条里，也没被攻过，算零轮 |
| 改名先做 | 站得住 | 两腿一致；理由是录入行孤儿，不用我再辩 |

## 3. 三处特别要看的

### 3.1 读故障那一标

**判决的原文**：「读故障那一标 \| 表里写「重建得出」，没写存哪 \| B1 重建不出 \| 攻方打中：入参是步种与每步流长，3.2 六张表都没有；正推那一行没有出处」。

**攻方打中，这一半我查实了**。E5 副本 `crash_injection.rs:1098` 的 `read_faults_in_a_crash_state` 有三个输入，不是两个：
1. `step_kinds`：`history.operations` 映射出来的步种，E5 副本 `crash_injection.rs:1466` 起算；
2. `self.after_each_step`：`StreamMarks` 的字段（E5 副本 `crash_injection.rs:1081`），每步跑完时录制流里有几步；
3. `last_stream_index_of_the_crash_window`：窗口末尾那个写在**录制流**里的下标，调用处 E5 副本 `crash_injection.rs:1651`，由 `stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, first_write_of_the_segment, writes_in_the_segment)`（E5 副本 `crash_injection.rs:1653`）从「写表下标 → 录制流下标」映射算出。

**判决给的存法不够**：「`12` 多存步种序列与每步之后的流长」只给了前两样。第三样要的 `stream_indexes` 是写表每个写在录制流里的位置，**屏障占录制流的位置、不占写表的位置**。E5 副本 `crash_injection.rs:1156` 的文档注释记着这里出过错：「拿写表下标去比会把窗口末尾认早（攻方 K2-c，…）」。3.2 的 `0F` 存「基线、写表、段、被判根、版本表」，写表是不带屏障位置的。所以照判决第一条存法，对比段拿到步种和每步流长，仍然算不出窗口末尾落在哪一步，只能把 K2-c 那个错再犯一次 [读]。

**判决括号里的另一条够用，应当当主条**：「或直接存每个崩溃态的读故障标」。理由：
- 这一标是跑出来的事实（哪几步在注入的读故障下往盘上发了写），不是比较，归跑数据段，不踩用户的分界；
- 它只有一位，三处读它：E5 副本 `crash_injection.rs:1737`（第一次崩溃之后的挂载透明）、`:3095`（拒绝合法性的判官入参）、`:3789`（二次崩溃之后再挂，沿用第一次崩溃那一位），一个来源；
- 对比段不用重新实现 K2-c 修过的那段映射。存「步种 + 流长 + 写表到流的映射」等于把这段微妙的下标算术复制到第二个地方 [推]。

**正推那一半**：判决说「正推那一行没有出处」，对的是正推 Q2 表里「拒绝合法性的输入」那一行（`m3-run-compare-split-r1-sonnet-output.md:143`）：读故障标记列在「是（重建得出）」里，标 [读]，但没引任何一行代码说它从哪来。不过同一份报告第 149 行写了「弃权的原因（翻译停在某步、读故障、写后报错后又起操作）都由事件或标记定，不依赖读回，跑数据段录一个码」。那一行说的是 RefFS 一票的弃权码（翻译层 `TheCrashIsInsideAStepWhoseRecoveryHadReadFaults`，`reference_filesystem_translation.rs:743`），是另一个量，但它给出的口径（跑数据段录标，对比段只读标）正是判决最后采的口径。所以：「没有出处」站得住；「正推没看见需要录标」说重了，缺的是把口径从 RefFS 弃权延伸到拒绝合法性与挂载透明，并给出处。

**攻方的构造可达**：`Injected` 这一支在随机历史里要靠「每块盘最新那一槽系统配置读回全 0 的可写挂载」这一类步，E5 副本 `crash_injection.rs:1173` 起的文档写明它是随机历史里唯一真往盘上发写的一种；有一个登记项专门造它：`.claude/gate.d/crash-point-replay-items.tsv` 里的 `read-faulted-states-never-hold-the-witness-criterion`（读故障档，24 步，`tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs` 的 `OPERATIONS_PER_HISTORY = 24`）[读]。

**推翻条件**：找到对比段不用屏障位置也能算出窗口末尾所在步的办法（例如 `0F` 的写表其实带录制流下标）→ 第一条存法也够；或这一位在三处用法里有一处读的不是同一个值 → 「一位」不够。

### 3.2 RefFS 那一格

**判决的原文**：「拆法用 `lib.rs:440` 现成的口，不用新开接口（攻方「要改 RefFS 的口」说重了）」。

站得住，而且比判决写的更硬：
- 公开口的行为：「只核 hi 判不判得了；读出的内容在不在允许集合里不判」（`crates/singlefs-reference-filesystem/src/lib.rs:445`）后以读出内容重起头，与 `crash_and_recover` 判绿之后做的是同一个 `restart_from_the_recovered_state` [读，`lib.rs:419-460`]。
- harness 翻译层已经把它接成事件：「RestartAfterARecoveryThatAbandonedTheNewestRoot(VisibleState),」（`reference_filesystem_translation.rs:507`）与调用（`:540`）。所以跑数据段要的「录而不判」是翻译层 `apply_event` 里给 `RecoverWithEveryOperationReturned` 加一条不判的执行路径，改的是 harness（簇戊的文件），RefFS 包一个字不动。
- 攻方 F2 那一行「没有一簇管 `crates/singlefs-reference-filesystem/src/lib.rs`」随之作废；判决已写成「只用现成公开口，不改 RefFS 包」。

**攻方没被冤枉的部分**：A1「历史内一票住在跑数据段」、B3「第三截要先判第一截」都是真的（上表第一行），判决也这么记了。被判说重了的只有「要改 RefFS 的口」。

**一处命名的坑，不是错**：`lib.rs:440` 的文档把它限定为「恢复时有读故障」的那一种。跑数据段对每次历史内恢复都用它（不管有没有读故障），事件名、文档会和用法对不上；翻译层给这条新路径起个新事件名就够，不用动 RefFS 的文档 [推]。

**推翻条件**：`restart_after_a_recovery_that_abandoned_the_newest_root` 与 `restart_from_the_recovered_state` 之间有一处只在「抛弃根」时才做的副作用（我读到的是没有：两者都先调 `allowed_set_for_a_crash_in(crash_falls_in)?` 核 hi，再调同一个 `restart_from_the_recovered_state`，差别只是 `crash_and_recover` 在中间多一道 `allowed.contains(&recovered)`，`lib.rs:419-460`）→ 现成口不够。

### 3.3 随机历史开 GPU 约 60 步封顶

**判决的原文**：「随机历史开 GPU 约 60 步封顶 \| 推的 \| 没提 \| 零轮、推的，进第二轮攻」。正推的来路是「一次发布约 24–30 次写，2048 ÷ 30 ≈ 68，快档 24 步 ≈ 900」，其中 24–30 是固定流（多记录流 A、B、C 三段）的数，不是随机历史的数；这一步的推法不对。

**撑得住它的是主 agent 自己的探针日志**（`~/.local/share/singlefs/work/m3-merge-impl/rh-probe/probe.log`，sha256 `12c37890…4e06`，第 24 行起是崩溃注入 24 段历史各自的 `RANDOM_HISTORY_PROBE` 行）。我的小脚本把它算了一遍：

```
$ nice -n 19 bash research/prompts/m3-run-compare-split-r2-sonnet-model/rh-probe-stats.sh
n=24 mean_writes=481.083 max=746 min=86 max_per_step=31.08 mean_per_step=20.05 cap_at_max_rate=65.9 cap_at_mean_rate=102.2 max_scaled_to_60_steps=1865
```

- 每段 24 步；写表最长 746 次写（种子 7463871032432355134，日志第 24 行：`writes=746 mkfs_writes=21 after_mkfs=725`），均值 481.083 [算]。
- 掩码上限：「pub const MAXIMUM_ENUMERATED_WRITES: usize = 2048;」（`crates/singlefs-checker-tier/src/crash_facts.rs:287`），建表时多于它交 `EnumeratedWritesPastTheMaskWidth`（E5 副本 `crash_judge_tables.rs:388`）。E6 补丁没动这个常量（`grep -c 'MAXIMUM_ENUMERATED_WRITES\|PERSISTED_MASK_WORDS' e6/patch/crates.patch` 得 0）。
- 给定状态表的写表是**整段历史 mkfs 之后的全部写**：E5 副本 `crash_injection.rs:2077` `writes: history_after_the_make_filesystem.writes,`，持久集合按整张写表给（`persisted_over_the_whole_write_table`，E5 副本 `crash_injection.rs:1879`）。所以封顶是历史长度的函数，不是崩溃窗口的函数。
- 掩码装的是**枚举写表**：录制流的写在前，后面接每个原地覆写的撕裂镜像与重放（E5 副本 `crash.rs:1596` `enumerated_writes: writes_with_torn_images.writes.clone(),`；建表处 E5 副本 `crash_judge_tables.rs:387`）。探针日志只数录制流的写，没数撕裂镜像与重放，所以 2048 位的真实封顶只会比下面的数更低 [读，**没量**]。

**这个数怎么说才站得住**：

| 速率 | 每步写数 | 2048 位在第几步满 | 依据 |
|---|---|---|---|
| 样本里最坏的一段 | 31.08 | 65.9 步 | 746 ÷ 24 |
| 24 段均值 | 20.05 | 102.2 步 | 481.083 ÷ 24 |
| 把最坏的一段线性放到 60 步 | — | 1865 < 2048，还没满 | 746 × 60 ÷ 24 |

所以「约 60 步」是最坏种子速率下的下限附近，不是均值，更不是硬数；写成「约 60–100 步封顶，视种子，撕裂镜像另算」才是读得出的。**线性放大是推的**（写数随步数不一定线性），用到的数里只有 24 步这一档是量的。

**它打中谁**：
- 不打中原形态与修订形态的区别：这个封顶来自 E5 的给定状态表（掩码 2048 位 + 整段历史做写表），拆不拆两段它都在。判决把它放进零轮清单不影响第三节任何一条。
- 打中谁：只有把随机历史步数抬过封顶且开 GPU 的跑法。登记里的两项（快档、读故障档）都是 24 步（E5 副本 `tests/crash_injection_campaign.rs:180` `FAST_TIER_OPERATIONS_PER_HISTORY: usize = 24`；读故障档 `OPERATIONS_PER_HISTORY: usize = 24`，其测试文件第 57 行）。harness 档的 150 步档（`crates/singlefs-harness/tests/random_histories.rs:76` `WALL_SAMPLING_OPERATIONS_PER_HISTORY: usize = 150`）不开 GPU；`.claude/gate.d/crash-point-replay-items.tsv` 里登的随机历史项只有 `crash-injection-fast-tier` 与 `read-faulted-states-never-hold-the-witness-criterion` 两行（`grep -n random_histories` 在该表里 0 命中）。复现入口可以传别的步数（`operations_per_history`），那一支才会撞 [读，我没跑]。
- 建表建不出时给定状态表的约定是判红、不退回别处（E5 副本 `crash_given_states.rs:130`，`judge_the_given_states` 的文档：判不了这张表时「调用方照它判红，不退回别处判」），所以撞上是红灯不是静默漏判；这是它该留零轮、不该挡第三节的理由 [读]。

**推翻条件**：量出 `enumerated_writes.len()`（含撕裂镜像与重放）在 24 步历史上的分布（要编 checker 档、重型，归主 agent）；若最坏段已超过 2048，则快档本身就不过，不是 60 步的问题。

## 4. 第二问：第三节七条修订 + 第五节四行

判定三种：站得住 / 够不着 / 同样打中别的臂。最后一列写有没有更简单、判别力不丢的形态；有的编号 S1–S7，细节在下面。

| 修订 | 辩方判定 | 同样打中别的候选吗 | 更简单的替代 |
|---|---|---|---|
| 1 分界三列 | 站得住 | 不打中别的候选：加的是两格，正推 §2 的三个缺格与攻方 E1/E2 都落在里面。判决把「模型只推进」放进跑数据列，**这一点有代码撑**：执行器边跑边问模型，`model.current_version_has_a_file_being_deleted()` 的答案决定还要走几步删除（`crates/singlefs-harness/src/history.rs:3883`、`:3895`、`:3964`），所以模型不能挪到对比段重放 | 无（S7 记「模型挪去对比段」被这几行否掉） |
| 2 表与键 | 站得住，但存法要换主条 | 历史键对「正推的按块键存」和「攻方的历史键」都成立；`12`、`14` 的键之外，`10`、`11` 的块键沿用不变 | S1：读故障改每崩溃态一位；S2：`12`、`14` 并一把族 |
| 3 版本 | 站得住，三处要补 | 「结论版本绑录入版本」对正推的 `extra_inputs` 改法与攻方的并版本表同样成立；「内核要罩进版本」同样打中现有的 `crash_verify_gpu.rs` 与 `gpu_unit_checks.rs` | S2、S3、S6 |
| 4 库与双机 | 站得住 | 「另起一份库」与「同库另一个列族」对「cp -a 整库」同样解；按块亲和对两种都成立 | 无 |
| 5 GPU | 站得住 | 「P14 并进恢复内核」被攻方探针打到只剩余量 12811；「P14 并进新内核」没量（M4）。判决选单独成内核，对所有候选都不违预算，没有只打中一支 | 现有先例：第 ② 段内核 + CPU 参照 `verify_state_from_facts` 逐状态相等就是自证（`crates/singlefs-checker-tier/src/crash_verify_gpu.rs:3` 起的文档「判据与 `crate::crash_facts::verify_state_from_facts` 逐条相同，两份逐状态相等是这一份的判别力自证」），新内核照这个形态 |
| 6 簇 | 站得住，一处要加 | 簇 0 的纯搬家对「录入分族」和「结论分族」都是前提 | S6 |
| 7 红步之后 | 站得住 | 按依不依赖模型分，对正推的「按模型时间线分」与攻方的「按依不依赖分」同样成立 | S4 |
| 五·1 快档怎么读库 | 候选不全 | 甲、乙对同一件事（17 份旧引擎用例 + 13 处注入调用要读库）解的是同一个问题 | S5 |
| 五·2 模型对不上怎么走 | 候选不全 | 三条（不走／照实现走／照模型自己的答走）都要先定义「对不上之后模型的状态」 | S4 |
| 五·3 记忆化 | 站得住 | E161 的去重命中率没有判过：「**G1–G5 每一行都未判**」（`.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:221`），所以没有更简单的数可补，只剩 M1 | 无 |
| 五·4 观察者 CPU 账 | 站得住 | 与拆不拆无关，判决已写 | 无 |

### 4.1 漏掉的更简单形态（S1–S7）

每条标：证据是读代码 / 算 / 推；判别力丢不丢；什么会推翻。S1 在 3.1 节，这里不重写。

**S2 `12`、`14` 并成一把「历史族」，而不是三把**
- 事实 [读]：`10`、`11` 的行按块键存，块键里已经有基线、写表、每个持久集合的摘要（E5 副本 `crash_amplification.rs` 的 `block_key_of_the_table`，攻方 B2 引在 `:8765`），所以这两张表的内容只是「块键 + core 读路径 + 提取代码」的函数；`12`、`14` 的内容是「种子与规模 + 执行器 + 模型 + RefFS + 翻译 + 实现交回的结局」的函数。
- 判决三族的写法里 `14` 罩「core 与模型」、`12` 罩「模型、RefFS、翻译」，**两处都漏了历史执行器**（`history.rs` 里产出 `ObservedOutcome` 与 harness 判的那一段，如 `harness_judgement_of_outcome`，`history.rs:1926`）。入口若按「写各表的函数所在文件」取，闭包会自己带上它；括号里手列的名单是示意，容易当成清单照抄。
- 代价账 [算/量]：重录一整族历史行的上界是整个 24 段崩溃注入跑完：「CRASH_INJECTION_FINISHED seeds=[7463871032432355113,7463871032432355137) slices=24 worker_threads=24 elapsed_seconds=16.3」（`rh-probe/probe.log` 第 55 行，含判定、挂载、二次崩溃，所以是 `12`/`14` 重录的上界）；`01` 的重录是 3.7–5.4 天的 CPU 上界（正推 8.1 节，旧数）。两个量级差五个数量级，把 `12`、`14` 拆成两族换不来东西。
- 分两族：**内容键族**（`10`、`11`：core 读路径 + 提取代码）与**历史键族**（`12`、`14`：前者 + 执行器 + 模型 + RefFS + 翻译）。判别力：不丢（任何一处改动仍然换版本），只是模型改动会多重录 `14` 里本来不依赖它的部分（16.3 秒量级）。
- 推翻：历史项扩到上万段（那时重录不再是秒级）→ 回到三族。

**S3 新内核的版本：先例是「自带摘要进自己表的键」，自证要有第三个桶**
- 判决第 3 条的自证写「`src/` 下 `include_str!` 读 `*.wgsl` 或自建管线的文件要么在某一把闭包里、要么登记不判对错」。今天有一个文件两边都不是：`crash_verify_gpu.rs`（第 ② 段内核）。E5 副本 `crash_identity.rs` 的自检把它列在「不在判器闭包里」（`:927`）与「改它结论版本不该换」（`:1077`、`:1083` 的 `expect_changes(unchanged, RUST_PROBE, false, …)`）两处，同时它的内核摘要进自己表的键：「核对代码的摘要 [`gpu_verifier_digest`] 进核对表的键：着色器改一个字，旧行留着当历史，新行另起」（`crates/singlefs-checker-tier/src/crash_verify_gpu.rs:8`），「pub fn gpu_verifier_digest() -> [u8; DIGEST_BYTES] {」（`:316`）。登记成「不判对错」会说谎（它的位喂 J7 的第 ③ 段），放进闭包会和 `:1077` 那条自检打架。
- 所以自证要三个桶：在某把闭包里 / 自带摘要进自己表的键 / 登记不判对错。
- 新比较内核（丙）照这个先例：每个内核的源码摘要进它那张结论表的键，**不必**把内核文件塞进 J8 的 GPU 入口。后一种做法（攻方 C1 的改法）有副作用：GPU 判器入口一变，三个恢复/扫描/走读内核的全部 GPU 结论都换版本（GPU 全量约 1.0–1.4 小时，正推 8.1 节旧数），而改的只是一个小比较内核 [推]。
- 判别力：不丢（内核改一个字，那张结论表的行就换键，攻方探针正对照 kernel3/kernel4 的同一件事），且不波及别的结论。
- 推翻：新比较结论读的是已经由 GPU 判器入口罩住的行（则本来就换）；或「摘要进键」那条路径本身出过假绿（`gpu_verifier_digest` 摘的是 `verifier_shader_source()`，文档写它是整份源码、段表宽与展开方式码的常量接在它前面，所以常量段在摘要里，`crash_verify_gpu.rs:316-319`；别的不经它进内核的输入我没查 [没查]）。

**S4 五·2 的第四条：红步之后，模型依赖的判不再问，不依赖的照判**
- 先例 [读]：同一个执行器里，一步中间出现第一处模型对不上之后就不再问模型，harness 判照做：「模型对不上之后不再问模型（分配代照判）」（`crates/singlefs-harness/src/history.rs:3878`），循环里每处问模型前都先看 `if verdict.disagreement.is_none() {`（`:3887`、`:3902`）。判决第 7 条的分法（依赖模型的 / 不依赖的）把这条先例从一步里推到整段历史。
- 这条候选的效果：红步之后模型依赖的判（每步模型比、B1/B9 与模型比）不出，所以不需要定义「对不上之后模型怎么往前走」，五·2 的三条候选都不用选；不依赖模型的判（只读镜像、RefFS、harness 的分配代判）照判、仍带「红步之后」一标。
- 丢什么：「红步之后依赖模型的判单列」那一类数。判决第 7 条与正推 12.2 节、攻方第七节自己的分析是：这类判「绿不证明任何东西、红多半是连锁」，所以丢的是噪声 [推，来自两条腿的读码分析，我没另量]。
- **要明说的冲突**：用户弹窗选的是「照判，报告里标在红步之后」，对依赖模型的判也是照判。这一条候选改了这半句，只能交用户，不能由主 agent 代采。
- 推翻：红步之后依赖模型的判里有一类在分叉后仍有独立含义（例如实现多拒了一次发布、模型照实现的完成发布数推进那一支，`model.rs:2956` 附近）→ 不能一刀切不问。

**S5 五·1 的第三条：按文件挂特性，走快档今天就有的第二趟**
- 事实 [读]：快档今天已经两趟，第二趟带特性：「再一趟 `cargo test --release -p singlefs-checker-tier --features」（`.claude/rules/verification.md:34`，在 `## checker 档自己分快档与全量` 一节里）；E5 副本 `tests/` 下 19 份文件提到 `verdict-store`、14 份按 `cfg(feature = "verdict-store")` 挂（第 2 节「快档构建」行）。`verdict-store = ["dep:rocksdb"]` 默认不编（`crates/singlefs-checker-tier/Cargo.toml:27`）。
- 候选丙：迁到库上的用例同样按文件挂特性，进第二趟；第一趟（不带特性）不动。甲（整条带）的账「每个编译目录多编一遍 C++」，在快档门禁里有一半今天就付着（第二趟已在编）；变异整表的拷贝目录只在被点名的行带特性时才编 [推]。
- 丢什么：迁走的 17 份用例不再在不带特性的第一趟里跑（第一趟更快，第二趟承担）；日常 `cargo test --lib --tests` 不再罩它们。乙（内存实现）则丢「快档不测 RocksDB 本身」。
- 推翻：迁走的用例里有一份要被 `--lib` 的单测（没挂特性）直接调 → 那一份只能走乙。

**S6 录入代码要住小文件：簇 0 要多拆一个**
- 事实 [读]：流水线文件 `crash_amplification.rs` 在 E5 副本是 11094 行（`wc -l`），且在停止表里，停止表给它的理由（转述，E5 副本 `judging_closure.rs:681`）是：先录后核的流水线，一个状态判成什么由判器定，录制器只把写表交给它。
- 拆开后录入代码（`record_crash_points_needing`、各表的写函数）就在这个文件里，两条路都坏：停在门外 → 提取代码不进录入版本（攻方 C2）；收进闭包 → 调度、对账、报告的任何一次编辑都换录入版本，重录 `01`。判决第 3 条「入口取写各表的函数所在文件」要成立，这些函数必须先住进一个只含录入与提取的小文件。
- 判决第 6 条的簇 0 只写了拆 `crash.rs`、`crash_injection.rs`、挪 `GivenStatesEngine`，没有这一件。它与甲（先做版本机制）耦合：不先拆，甲的入口清单没有地方指。攻方 F1 写的环，换个方向就是这里。
- 丢什么：无；是搬家。验收可以沿用判决的「词法单元多重集不变」（`lexical_tokens`，E5 副本 `judging_closure.rs:21`）。
- 推翻：录入代码其实已经在 `crash_given_states.rs` 这类小文件里（我只看到类型与 CPU 就地判在那里，`record_crash_points_needing` 在 E5 副本 `crash_amplification.rs:1113`，不在 [读]）。

**S7 考虑过、被代码否掉的：把模型挪到对比段重放**
- 想法：模型和 RefFS 的答案都是确定的纯函数，对比段预处理里重放就行，`12`、`14` 不用存模型派生值，也没有「模型派生常量」缺口。
- 否掉：执行器边跑边问模型来决定下一步做什么：`model.current_version_has_a_file_being_deleted()` 决定还走不走一步删除（`history.rs:3883`、`:3895`、`:3964`）。模型在跑数据段里就是执行器的一部分，所以判决放进左列对。RefFS 不同（它只吃观察到的结局，不控制执行），那一半对比段重放可行，判决已经这么拆。

## 5. 小结：每条结论什么会推翻它

| 结论 | 来源 | 推翻它的现象 |
|---|---|---|
| 第二节没有一格该翻 | 读代码（第 2 节每行的引文） | 任一行里引的代码行说的不是那件事；我读过的行：`lib.rs:208-250`、`:286-300`、`:419-460`，`reference_filesystem_translation.rs:500-560`、`:890-1010`，`crash_injection.rs:1060-1180`、`:1440-1500`、`:1636-1670`、`:1890-2110` |
| 读故障标判决的存法不够，一位才够 | 读代码 + 推 | 对比段不用屏障位置也能算窗口末尾所在步（见 3.1 推翻条件） |
| RefFS 攻方说重了站得住 | 读代码 | 现成口与私有函数之间有只在抛弃根时才做的副作用 |
| 60 步封顶方向对、数要改成 60–100、撕裂镜像另算 | 算（探针日志）+ 读 | 量出的枚举写表长度分布与此不符（3.3 推翻条件） |
| S2 并一把历史族 | 读 + 量（16.3 秒是别人的探针） | 历史项扩到上万段 |
| S3 摘要进键 + 三个桶 | 读代码（`crash_verify_gpu.rs`、`crash_identity.rs` 自检） | 摘要进键出过假绿，或新比较结论读的是已被入口罩住的行 |
| S4 第四条候选 | 读代码（`history.rs:3878`）+ 推 | 红步之后依赖模型的判有独立含义的一类 |
| S5 第三条候选 | 读 `verification.md` + 推 cargo 行为 | 迁走的用例要被没挂特性的 `--lib` 单测直接调；或 rocksdb 的 C++ 并不跨趟复用（第二趟与第一趟要分开编） |
| S6 簇 0 多拆一件 | 读（`wc -l`、停止表理由） | 录入代码已经住在小文件里 |
| S7 模型不能挪去对比段 | 读代码（`history.rs:3883`） | 执行器问模型的那几处可以换成不问模型的等价写法 |

## 6. 没做什么

- 没编译、没跑任何 checker 档或 harness 档测试；只跑了 `rh-probe-stats.sh`（awk，只读日志）。重型一律没起。
- 没重跑攻方的三个模型（体量、内核展开、闭包探针），所以 62660 / 387189 / 闭包探针的数我没复核，是引主 agent 判决里的说法；「探针没写判据本体」是攻方自己报告里写的。
- 没量：24 步历史的枚举写表长度（含撕裂镜像与重放）、观察者挂载写分组数、新表压缩比、P14 内核本体展开量（M1–M4 仍在主 agent 手里）。
- 没碰 kb、records、`.claude/`、`crates/`；没改副本；没读禁读清单里的任何文件。
- 没替主 agent 撤销或维持判决；S1–S7 是候选，S4 与 S5 的一部分要用户定，S1、S2、S3、S6 是实现细节层面的建议。
- 没判第三节七条的攻击面（修补本身、方法、没被看过的地方）：那是同轮攻方腿的活。
- 草稿目录 `~/.local/share/singlefs/work/m3-run-compare-split-r2-defense/` 建了、没放东西；没建仓副本与编译目录，所以交回闸要求的「删副本」无事可做。

## 7. cite-check

`python3 scripts/cite-check.py research/prompts/m3-run-compare-split-r2-sonnet-output.md --background research/prompts/_m3-run-compare-split-r2-background.md` 末行：

```
  ✓ 核了 17 处引文，对上 17 处，没判 0 处（逐处列在上面）
```

补一条：我另用 `--root` 指到 E5 副本再跑过一遍，唯一对不上的是 `scripts/crash-amplification-two-hosts.sh:1040`——E5 副本里这份脚本是旧版（那句在第 1031 行），主工作区才是 1040；报告引的是主工作区，交回闸用默认根，不受影响。
