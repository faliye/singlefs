# 崩溃点重放拆成「跑数据 / 对比数据」两段：设计轮第一轮，正推腿报告（2026-10-10）

立场：正推。从已定条款与 `crates/` 今天的实现（加六份补丁）推「应该是什么」，再与正文第三节逐格比。不替主 agent 采纳或出判决。

## 0. 读法

- 证据标签：**[读]** 读代码或文件得出；**[算]** 算出来的，算式在文中；**[推]** 推的，没有直接观测；**[复核不了]** 说明原因。
- 底子：副本 = 主工作区 + v、e、e2、e3、e4、e5 六份补丁，`apply-writer-patch.py` 依次退出码 1、0、0、0、0、0（第一份的 1 是 code-source-discipline 的 mutation-tables 格，与事实表 0 节所记相同）。六份补丁的 `crates.patch` 里 `.wgsl` 文件的 `diff --git` 行数各为 0 [读，`grep '\.wgsl'` 逐份数过]。
- 引用写法：路径带 `copy/` 的是副本（`crates/singlefs-checker-tier/` 下被补丁动过的文件）；不带 `copy/` 的是主工作区（`crates/singlefs-harness`、`singlefs-core`、`singlefs-reference-filesystem`、`.wgsl`、kb、脚本都没被六份补丁动过）。副本在交回前已删，重建法：`rsync -a --exclude target --exclude 'target-*'` 拷主工作区（带 `.git`），再依次 `apply-writer-patch.py ~/.local/share/singlefs/work/m3-merge-impl/<名>/patch --root <副本>`。
- **事实表里 `crash_injection.rs` 的行号在 E5 之后整体偏移约 −180**（E5 把给定状态表的类型、trait 与 CPU 就地判搬进了 `copy/crates/singlefs-checker-tier/src/crash_given_states.rs`）：例如 `// gpu-exempt: writable-mount` 事实表记 3289，副本里是 `copy/crates/singlefs-checker-tier/src/crash_injection.rs:3109`。下文都用副本的行号。
- 正文第一节引的用户原话，我回调度记录核过出处在 `records/2026-09-30-里程碑三开工第一轮.md` 第 994、996、997、1003 行（逐句 grep 命中）。

## 1. 先说结论（每条在后文有出处）

1. **形态方向做得出来，但正文有七处要补或要改**（下面逐条，细节在各节）：
   - 3.1 分界缺一类「对比段内部的 CPU 预处理」（建判表、抽事实表）和一类「判器自证与手摆镜像用例」；没说它们归哪边。
   - P1–P18 漏了历史执行器里的三类比较（分配记录代数、冷启动读回报错、每步池级 checker 的停历史判据）、历史翻译里 RefFS 的「历史内恢复一票」、规划阶段的 B0 断言、「拒后盘逐字节不变」这一整盘相等比较。
   - 事实表 D14 把「拒后盘变没变」放进跑数据段由 CPU 录布尔值，这一条与用户「生成物的对比可以交给gpu的都交给gpu」的分界冲突。
   - 3.2 的「录入版本另起一族」在 `crates/` 今天的闭包算法下**单靠起一族做不到**：闭包按文件收、流水线五份是停止文件、`crash.rs` 与 `crash_injection.rs` 把「跑」和「比」放在同一个文件里（Q3）。
   - 对比结论的版本没有绑定它读的录入版本（`crash_identity.rs:619` 对 GPU 与 CPU 判器的额外输入写死 `"-"`），重录之后旧结论会被当成已判过复用，出「过期的绿」（Q3）。
   - 3.4 的簇漏三份用 `crash_injection` 公开接口的集成测试、一份调旧引擎的测试文件；「丁二 迁完就删旧引擎」不成立，`enumerate_layer0_*` 还被基础设施用例、实验装置 `e161` 与门禁脚本引用（Q7）。
   - 丙（新内核）的对拍需要丁一、戊的 CPU 判那一半当参照，3.4 把它们写成「不交」是漏的依赖（Q7）。
2. **RefFS 拆得开**（事实表 P11 写「没核」）：第一次崩溃的允许集合是只读函数 `allowed_set_for_a_crash_in` 的返回值，不依赖读回；判绿之后重起头的 RefFS，其第二次崩溃的允许集合恰是单元素 {R1}（Q2）。
3. **每态定长 160 字节的账**：按 26.16 亿态是 418.6 GB，但**今天登记的全量深度是 30 不是 28**（`scripts/crash/crash-amplification.sh:222`），去重后的下界是 3 743 376 012 态，160 字节是 598.9 GB；磁盘余量开工时 514G，交回时 1.8T，会变（Q4）。
4. **CPU 代价是大头**：GPU 一侧约 7.2×10⁵ 态/秒，旧记录里 CPU 判器两台合计约 8 千态/秒（上界，含 checker）；观察者（P5）要在约 1.36×10⁹ 态上真挂载，一次轻型跑里每态墙钟约 66 ms，量级以天计，且这一块与拆不拆无关（Q5）。
5. 九个问题的答案在第 4–12 节；P1–P18 逐处核对表在第 3 节；末尾是判定一览与「没做什么」。

## 2. 3.1 分界：每格归得对不对

用户原话三句（`records/2026-09-30-里程碑三开工第一轮.md` 里的位置）：

- `records/2026-09-30-里程碑三开工第一轮.md:996`：「cpu跑 录入数据 gpu跑对比运算数据。 严格来说开启了gpu的时候，所有的对比都需要用gpu对比。」
- `records/2026-09-30-里程碑三开工第一轮.md:997`：「这个流程就是两部分 跑数据和对比数据 跑出的数据放入rockdb中，然后读出来对比。 有GPU的时候可以发GPU，没有GPU的时候就等数据跑完再对比。」
- `records/2026-09-30-里程碑三开工第一轮.md:1003`：「初期的环境检测 指纹检测让cpu做。生成物的对比可以交给gpu的都交给gpu」

| 3.1 的格 | 我的判定 | 依据 |
|---|---|---|
| 左栏「真跑 core：录制历史、只读恢复两遍、可写挂载、续做、再挂、读回」 | 一致 | 这些都是调 `singlefs_core` 产出，不判对错：`recover` 在 `copy/crates/singlefs-checker-tier/src/crash.rs:1374`、`:1375`，挂载在 `copy/crates/singlefs-checker-tier/tests/common_crash_point_observers/mod.rs:169` [读] |
| 左栏「真跑理想模型、RefFS（出各版与允许集合）」 | 一致，但模型要新加一个「只推进、不比较」的口子 | 现在模型推进和比较是同一个函数 `judge_and_advance`（`crates/singlefs-harness/src/model.rs:2874`），对得上才推进（Q9） [读] |
| 左栏「显卡与配置检测、清场；输入指纹、判法版本、录入版本的现算与核」 | 一致 | `conclusion_version_checked_against_the_build`（`copy/crates/singlefs-checker-tier/src/crash_identity.rs:721`）、`computation_routes`/`CardHost`（`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:7596`、`:7604`）都是这一类 [读] |
| 右栏 oracle、池级 checker、记录核对器 | 一致 | `copy/crates/singlefs-checker-tier/src/crash.rs:1376`、`:1388`、`:1389` [读] |
| 右栏「与理想模型比」「与 RefFS 允许集合比」 | 一致 | 理想模型那一比是三参数纯函数（`crates/singlefs-harness/src/model.rs:3433`）；RefFS 的集合见 Q2 [读] |
| 右栏「拒绝合法性与 W 的判据复算」 | 一致（用户 1003 行的话），但这一块最重 | 判据复算 535 行 Rust（`crates/singlefs-harness/src/mount_transparency.rs:2352` 到 `:2886`）要在 GPU 上重写，见 Q6 [读] |
| 右栏「第 ② 段事实核对」 | 一致 | 但**抽事实表**（`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:3503`）是对比段内的 CPU 预处理，3.1 没点名它归哪边 [规则没说] |
| 右栏「第 ③ 段、J7、判红之后归类」 | 一致，但「归类」要拆开 | 红态的签名位可上 GPU；下写的文字正文与落镜像文件（`findings.settle` 带 `image_directory`，`copy/crates/singlefs-checker-tier/src/crash_injection.rs:1634`、`:1644`）只能 CPU [读] |
| 右栏「J6 对照：开 GPU 时 CPU 判器在同一批数据上也判，从库里读」 | 一致 | 两份结论今天已分别存库（kind `GpuJudge`、`CpuJudge`，`copy/crates/singlefs-checker-tier/src/crash_identity.rs:541` 起）[读] |
| **没有的格**：对比段内的 CPU 预处理（建判表 `build_judge_tables_*`、抽事实表、把序号解成持久集合的枚举表 `layer0_enumeration_tables`） | 规则没说 | 它们只读库里的流程输入、不跑被测代码，应写成「对比段内 CPU 预处理」；见 `copy/crates/singlefs-checker-tier/src/crash_judge_tables.rs:359`、`copy/crates/singlefs-checker-tier/src/crash.rs:1580` [读] |
| **没有的格**：账与报告（对账 `unjudged_blocks_of_the_flow`、复用计数、覆盖报告、红态正文） | 规则没说 | 既不是跑也不是对生成物的比较，应写明留 CPU、排在对比之后 [推] |
| **没有的格**：判器自证与手摆镜像用例（GPU 对拍自证、`record_checker_breaks_the_history_at_the_power_loss_seam.rs` 这类自己造镜像的） | 规则没说 | 它们**就是** CPU 参照本身（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:81`），不能搬走；要在 3.1 写成留 CPU 的第三类 [推] |

## 3. P1–P18 逐处核：跑与比拆不拆得开，比的那一半 GPU 一侧接不接得上

「拆得开」= 比的那一半只需要库里的东西（`0F`–`14` 加已有表）和对比段自己的 CPU 预处理。路径前缀 `C=copy/crates/singlefs-checker-tier/src/`、`H=crates/singlefs-harness/src/`。行号都是上面说的两个根里 grep 过的 [读]；「接得上」「卡点」列里带 [推] 的是推的。

| P | 跑（CPU，录什么） | 比（今天在哪） | 拆得开 | GPU 一侧接哪 | 卡点 |
|---|---|---|---|---|---|
| P1 层 0 旧引擎 | `recover` 两遍：`C crash.rs:1374`、`:1375` | `C crash.rs:1376`、`:1382`（oracle）、`:1388`、`:1389` | 是：`Layer0StateJudgement`（`C crash.rs:1261`）里 Rust 恢复的产物只有 `consulted`、`ignored` 两份，别的字段由镜像算 | 恢复内核 `classify_oracle`（`C crash_judge_recovery.wgsl:294`）、`check_records`（`:539`）；判表加 Rust 恢复表（`JudgeTablesWords` `C crash_judge_tables.rs:1908`，`JUDGE_TABLE_COUNT` 20 在 `:2002`，形态版本 `:47` 要抬） | 旧引擎不经判表也不经库，「迁走」的量见 Q7 |
| P2 流水线第 ① 段 CPU 判 | `record_crash_points_needing`（`C crash_amplification.rs:1113`）今天只记块与状态数；要加逐状态 `recover` | `judge_one_block`（`C crash_amplification.rs:1442`） | 是，机械 | 同 P1 | 无 |
| P3 开 GPU 的流水线 | 同 P2（开不开 GPU 都录） | `judge_my_blocks_on_gpu`（`C crash_amplification.rs:10685`、`:10713`）今天一次 `recover` 都不调（`C crash_judge_gpu.rs:2`） | 是 | 比一致 = 已有 J6 的 `CrossCheckWords`（`C crash_judge_conclusions.rs:315`）里落点、结局、失败点三项，**缺读回摘要**；读回摘要与版本表里的期望摘要直接比相等，WGSL 里不算 SHA-256 | J6 可被用户授权跳过（`C crash_amplification.rs:7599`），P3 的 Rust 对 WGSL 比一致不该跟着跳：要从 J6 里拆出来恒跑 [推] |
| P4 单元校验和对拍 | 无可录（输入是镜像） | `gpu_unit_checksum_items`（`C crash_amplification.rs:1963`，调用 `:1474`）GPU 算 CPU 比 | 不用拆：它是 GPU 对 CPU 的自证，同 J6 一类 | `gpu_unit_checks.rs` 的 CRC32C 内核，自己建流水线、不过展开预算（`C crash_judge_dispatch.rs:263` 的量法只管 `compute_pipeline` 编的内核） | 留 CPU 参照，没有缺口 |
| P5 观察者 | 挂载（`C tests/common_crash_point_observers/mod.rs:169`）+ `recover`（`C crash_amplification.rs:10048`）；要加挂载写录制层（`mod.rs:155` 起现用 `SparseBlockDevice`，没有录制） | 断言 `mod.rs:178`、`:192`、`:202`、`:219`；挂完的盘 `check_pool_image`（`mod.rs:231`） | 是，要把四条断言读的量压成标量记录；`assert_the_continuation_publishes_take_the_next_numbers_of_the_instance`（`mod.rs:219`）读一串发布的（实例，txg，事务号） | 池级 checker：走读内核、扫描内核（挂载写并进判表）；四条断言：新小内核 | 判表上限：每落点 64 次写（`C crash_judge_tables.rs:51`）、持久掩码 2048 位（`C crash_facts.rs:287`），推的用量远低于上限（Q6）；**CPU 代价才是卡点**（Q5） |
| P6 第 ② 段读回 | `store_flow_facts` 已写（`C verdict_store.rs:1330`，调用 `C crash_amplification.rs:3536`） | 核对用内存里刚抽的那份（`:3503`），`read_flow_facts` 存在（`C verdict_store.rs:1347`）而没人调 | 是，几乎免费：改成读回 | `GpuFactsVerifier`（`C crash_verify_gpu.rs:337`）已在 | 抽事实表要归「对比段 CPU 预处理」（第 2 节） |
| P7 第 ③ 段与 J7 | 无 | `compare_the_conclusions_and_the_verifier`（`C crash_amplification.rs:3660`）逐态一字节对一位；`count_vector_disagreements_of_the_flow`（`:10295`）逐块算术，按段区间闭式现算（`:10249` 起） | 已经只读库 | 新小内核，每态 1 字节编号 + 1 个核对字 | GPU 上没有吞吐收益（CPU 逐态比 2.6×10⁹ 态是秒到分钟量级 [推]），是照用户口径做 |
| P8 每步模型比 | 每步 (实现交回值 `ObservedOutcome`, 模型的答 `ModelAnswer`)，**加模型在这一步的派生常量**：`capacity_wall_is_permitted` 读 `unit_area_slots_per_device()`（`H model.rs:2854`），`judge_effect` 读 `effective_rollback_floor()`（`H model.rs:3036`）与 `geometry` | `judge_by_model`（`H history.rs:1447`）19 处调用；`judge_and_advance`（`H model.rs:2874` 到 `:2985`） | 是，但**要新加「不比较、按模型自己的答推进」的口子**（现在对不上就不推进，`H model.rs:2870`） | 无现成内核；要新写 | 变长嵌套的 `expected_roots` 要压平；说明文字留 CPU（Q6、Q9） |
| P9 给定状态表 | 持久集合、写表进 `0F`/`10`（`C crash_given_states.rs:27` 的 `GivenStatesOfATable`） | 引擎 `C crash_amplification.rs:8361`；默认入口 `GivenStatesJudgedOnTheCpuInPlace`（`C crash_given_states.rs:138`，用在 `C crash_injection.rs:1237`、`:1285`、`:5067`）不经库 | 是 | 现成：`build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams`（`C crash_judge_tables.rs:359`）+ 三内核 | 就地判路径要么删、要么也走库；删了则每个调 `inject_crashes_into_history` 的测试（13 处）都要链 RocksDB（Q8） |
| P10 理想模型 | 读回（R1）、模型各版 `committed_versions`（`C crash_injection.rs:1410`、`:1420`） | `disagreement_with_the_ideal_model`（`C crash_injection.rs:1573`）→ `crash_recovery_disagreement`（`H model.rs:3433`） | 是：三个入参，四条判据，无状态 | 新小内核：选到的根对「比最新持久根旧」「没提交过」两项是 (txg,实例) 对比与有序表查找，内容一项比摘要 | 无 |
| P11 RefFS | 第一次崩溃的允许集合（只读）、历史内每次恢复的 (hi, 允许集合, 读出) | `C crash_injection.rs:1581` → `H reference_filesystem_translation.rs:1022`；第二次 `:916` | **是**（Q2 展开） | 新小内核：读出摘要属于摘要集合 | 历史内恢复一票要挪（Q1） |
| P12 第二截记录核对器 | 挂载与发布的写（`11`、`13`） | `record_check_after_the_writable_mount`（`C crash_injection.rs:3023`，调用 `:1700`） | 是 | 并进第三截那张给定状态表（`C crash_injection.rs:2991` 起的 `persisted_with_the_mount`） | 现成恢复内核的记录核对器只认一份镜像，要改成每态两份掩码（崩溃态、恢复后）[推] |
| P13 第二截池级 checker | 同 P12 | `checker_violations_of(&mount_run.pool_after)`（`C crash_injection.rs:3184`） | 是 | 同张表，现成走读、扫描内核 | 同 P12 |
| P14 拒绝合法性与 W | 拒因（错误枚举）、错误文本；`refusal_reason_on_the_d23_list`（`H mount_transparency.rs:2280`）的结果是跑数据 | `C crash_injection.rs:3133` → `H mount_transparency.rs:3025`；W 在 `C crash_injection.rs:4168` 起 | 判据复算（`H mount_transparency.rs:2352`–`:2886`）读的是崩溃镜像，可拆；**「拒后盘逐字节不变」是整盘相等**（`C crash_injection.rs:3128`），事实表 D14 放进跑数据段由 CPU 录布尔值——这一格与用户分界冲突（第 4 节） | 新内核，复用 `crash_judge_common.wgsl` 里系统配置、根环、journal 的解析 | 535 行 Rust 重写；展开量未知，推 7–25 万，落在「27 万到 98 万之间没量过」的不确定带下沿（Q6） |
| P15 挂载透明 | R1、R2、二次崩溃 R（`RecoveryFieldsRead`，`H mount_transparency.rs:113`）、续做根指针、抬 F、清扫删的行 | `C crash_injection.rs:3381`、`:3465`；`H mount_transparency.rs:1659` | 是 | 新内核：逐字段按字节区间比 | R 含变长条目（树表条目原字节 `Vec<Vec<u8>>`），要定长化或按摘要比 |
| P16 I-8.4 | 再挂读出摘要、没崩过那次的摘要 | `C crash_injection.rs:2372`（调用 `:4086`） | 是 | 摘要相等，可并进 P15 内核 | 无 |
| P17 判红后归类 | 判红那份镜像由库重建 | `findings.settle`（`C crash_injection.rs:1634`、`:3208`、`:3435`、`:4003`、`:4133`） | 要拆成两半 | 签名位可上 GPU；正文文字与落镜像文件只能 CPU | 3.3「P17 上 GPU」只能说成「红态判定上、文字不上」 |
| P18 J6 | 无 | 流水线一路 `C crash_amplification.rs:9547`；给定状态一路当场在内存比（`:9202`、`:9074`） | 是：两份结论都已分别存库 | 无（J6 就是 CPU 参照） | 给定状态那一路要从「当场比」改成「读库比」 |

## 4. Q1 全不全：对比有没有漏在 P1–P18 之外，有没有归错边

### 4.1 我自己的 grep 与事实表的 117 处

事实表 117 处的模式表里有「oracle 两族」「挂载透明三族」「事实核对两族」这种聚合名，我复刻不出同一个数，所以**没有核 117 这个总数** [复核不了：模式表不是逐字给的]。我做的是拿产品路径上的几份源（下面命令的 `<文件>`）逐处对到 P 号 [读]：

```
cd copy/crates/singlefs-checker-tier
grep -nE 'check_pool_image|check_records\(|check_records_against\(|record_check_after_the_writable_mount\(|record_check_after_a_second_crash\(|disagreement_with_the_ideal_model\(|crash_recovery_disagreement\(|reference_filesystem_judgement_of_a_crash_image\(|reference_filesystem_vote_on_a_second_crash\(|\.judge_a_crash\(|judge_a_second_crash_after_the_restart\(|criteria_true_without_read_faults|checker_violations_of\(|classified_oracle_violation_for_the_version_table\(|oracle_violation_for_versions\(|judge_mount_transparency|judge_writable_mount_refusal|compare_content_read_once_and_twice\(|instance_table_rows_against_the_sweep_publish\(|compare_the_conclusions_and_the_verifier\(|compare_judge_and_verifier\(|compare_the_gpu_and_the_cpu_conclusions\(|count_vector_disagreements_of_the_flow\(|judge_a_given_state_on_the_cpu\(|judge_crash_image|verify_states_from_facts|verify_blocks_from|judge_by_model\(' <文件> | grep -vE '^[0-9]+:\s*(//|///|//!)' | grep -vE '^[0-9]+:\s*(pub(\([a-z]+\))? )?fn ' | grep -vE '^[0-9]+:\s*use '
（只看产品部分时，把开头的 `grep -nE '…' <文件>` 换成 `head -n <单测模块起始行 − 1> <文件> | grep -nE '…'`：`crash.rs` 单测模块起于 3977 行，`crash_injection.rs` 起于 5249 行。）
```

命中数 [算，产品部分 / 整份]：`src/crash.rs` 10 / 17、`src/crash_injection.rs` 24 / 29、`src/crash_amplification.rs` 9 / 9、`src/crash_given_states.rs` 3 / 3、`src/bad_disk_input.rs` 1 / 1；harness 的 `src/history.rs` 20、`src/fault_injection.rs` 4、`src/mount_transparency.rs` 1、`src/reference_filesystem_translation.rs` 1。checker 档产品部分共 46 处，我逐处对过，都落在事实表 A1–A9、B1–B15 里（含 `crash.rs:961` 一族单版本 oracle，属 A1）；**命中里没有找到不在表里的比较入口**。下面七类是**名字不在模式表里、但就是比较**的东西，是读上下文找到的，不是 grep 命中的。

### 4.2 漏在 P1–P18 之外的比较

| # | 是什么 | 在哪 | 为什么算比较 | 该归哪 |
|---|---|---|---|---|
| G1 | 规划阶段的 B0 断言：对基镜像跑一次 Rust 恢复，落到的根要等于从基镜像扇区里数出的最新根 | `copy/crates/singlefs-checker-tier/src/crash_amplification.rs:626`（定义，`recover` 在 `:635`），调用在 `:703`，每条流一次 | `assert_eq!` 一份 Rust 恢复产物对一份由镜像算出的值 | 归 CPU「输入前提检查」，与指纹检测同类；3.1 要加这一格 [推] |
| G2 | 历史执行器自己的三类判断：分配记录的代不是发布的 txg；冷启动读回报错；每步池级 checker 与第一次失败就停的判停 | `allocation_generation_judgement`（`crates/singlefs-harness/src/history.rs:3242`，8 处调用 `:3338`、`:3581`、`:3598`、`:3925`、`:4216`、`:5093`、`:5133`、`:7220`）；`harness_judgement_of_outcome`（`crates/singlefs-harness/src/history.rs:1926`）；`violations_on`（`:5240`，由 `per_step_checker.runs_the_checker()` 控制 `:5790`、`:5815`、`:5885`） | 它们和 `judge_by_model` 一样在历史里当场判、判红就停（`crates/singlefs-harness/src/history.rs:5745` 文档注释） | P8 把历史改成「跑到实现自己停」时必须连它们一起改；崩溃注入用 `PerStepChecker::Skipped`（`copy/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:192`），所以崩溃点重放里真起作用的是前两类 [读] |
| G3 | 「拒后盘逐字节不变」：`mount_run.pool_after == *refusal_on_the_crash_image.crash_image_materialized` | `copy/crates/singlefs-checker-tier/src/crash_injection.rs:3128` | 两盘整份相等比较，结果喂给拒绝合法性判官 | 对比段（事实表把它放进跑数据，冲突，见 4.4） |
| G4 | 历史翻译里 RefFS 的「历史内恢复一票」：`RecoverWithEveryOperationReturned` → `crash_and_recover`，读出不在集合里就记 `StoppedVerdict::Red` | `crates/singlefs-harness/src/reference_filesystem_translation.rs:530`、`:532`、`:563`；`translate` 在跑历史时调（`copy/crates/singlefs-checker-tier/src/crash_injection.rs:1460`） | 这是 RefFS 对读出的内容做成员判断，发生在「跑数据」段里，红了会让之后所有崩溃时刻都判红 | 对比段；拆法见 Q2（run 侧不判、无条件重起头，比较侧传播「停在第 k 步」） |
| G5 | 对比段内部的先后：B5 吃 B1 的输出（`read_only_read_back: if disagreement.is_none() {`） | `copy/crates/singlefs-checker-tier/src/crash_injection.rs:1710`、`:3955` | 拒绝合法性要用 B1 已有的结论：只读读出在不在模型的允许集合里 | GPU 上 P14 内核必须在 P10 内核之后派，或把 P10 的结论位作为 P14 的输入列 [推] |
| G6 | 坏盘输入战役：`check_pool_image` 在 CPU 上判被改坏的镜像 | `copy/crates/singlefs-checker-tier/src/bad_disk_input.rs:1308` | checker 档的战役（`.claude/kb/decisions/13-验证路线.md:395` 列了「崩溃注入与坏盘输入两场战役」），没有 GPU 路 | 它不是崩溃点重放（没有崩溃状态），3.1 没表态；要主 agent 定射程 |
| G7 | 判器自证与手摆镜像用例（6 份：`crash_points_tree_identifier_watermark_orphans.rs`、`crash_points_of_the_rollback_publish.rs`、`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`、`crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs`、`record_checker_breaks_the_history_at_the_power_loss_seam.rs`、`crash_segments_per_device_and_torn_in_place_overwrites.rs`） | `copy/crates/singlefs-checker-tier/tests/` | 自己造镜像、当场判；是 CPU 参照 | 留 CPU（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:81`），3.1 要写成一格 |

### 4.3 射程判据：harness 档 tests 与 src/bin 装置在不在

我给的判据：**一处比较在射程里，当且仅当它被下面任一入口跑到**——放量分项（`scripts/crash/crash-amplification.sh:230` 起 `ITEMS` 的 8 项）；合表 12 项第 ② ③ ④ 列点名的测试目标（`.claude/gate.d/crash-point-replay-items.tsv`）；它们直接调的 checker 档库函数与 harness 函数。据此：

- `singlefs-harness/tests/` 不在：harness 档随时跑、不分 GPU，且 harness 档不许依赖 checker 档（`.claude/kb/decisions/13-验证路线.md:395`）。
- `src/bin/` 的 E 系列装置不在：它们是独立验证装置（`.claude/rules/implementation-first.md:14`「`research/` 下的装置与模型只用来独立验证」、`:15` 的例外），由 experiment-replay 复跑。但 `e161` 调着旧引擎 `enumerate_layer0_in_state_slices`（`copy/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:3023`），这一条决定旧引擎能不能删（Q7）。
- 故障注入战役的比较（`crates/singlefs-harness/src/fault_injection.rs` 里调 `judge_a_reopen_after_an_injected_fault`，`:2561`）在 harness 档，不在 [读：checker 档只有两份测试 `use` 它的 `injected_block_device_error` 等工具]。
- 不在合表里、却在 checker 档快档里跑的 6 份（G7）：在射程外的是**用例自己的判**（它们是参照），在射程内的是它们调的库函数（改了库函数它们跟着改）。

### 4.4 归错的边（逐处对 3.1 与用户 1003 行的话）

- **事实表 D14 一格与用户的分界冲突**。事实表 D14 把「拒后盘变没变」放进跑数据段的 `11`；用户 `records/2026-09-30-里程碑三开工第一轮.md:1003` 说「生成物的对比可以交给gpu的都交给gpu」。并排整行：

```
事实表 report.md:157（整行）：| D14 | 拒绝合法性 B5、W B12 | 边跑边比、CPU | 拒因、错误、拒后盘变没变进 `11`；崩溃镜像由 `0F`+`10` 重建 | 要新写：判据在崩溃镜像上复算（`singlefs-harness/src/mount_transparency.rs:2386`、`:2446`、`:2498`、`:2542`），系统配置槽、根环、journal 的读法可复用 `crash_judge_common.wgsl`。另一种分法请主 agent 定：判据复算算跑数据（CPU 录判据值），GPU 只判「拒因 × 判据值」 | 判那一段调 `judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount` 读录下的 | 几十字节 |
用户 records:1003 | …「初期的环境检测 指纹检测让cpu做。生成物的对比可以交给gpu的都交给gpu」…
```

  做得到的改法 [推]：`11` 只录挂载写表（`13` 去重），「盘变没变」由对比段对「写的字节是否异于崩溃镜像同位置字节」现算；GPU 上这是对每次挂载写按落点查崩溃镜像版本再比字节，要新表（写的字节摘要 × 落点版本摘要），不是现成表。
- 事实表 D17 / 3.3 的「P17 上 GPU」只对红态签名位成立（见第 3 节 P17 行）。
- 3.1 左栏「读回」只该含「读回 + 算摘要」，摘要是对生成物的**归纳**不是比较，归跑数据没问题 [推]。
- 没有发现「该归 CPU 的步被划进对比」的其它格：环境检测、指纹检测、清场、版本核都在左栏。

## 5. Q2 对比那一段只读库，每一样输入重建得出来吗

结论：**RefFS、模型各版、读回内容、镜像都重建得出来；缺的是四件小东西**——观察者另读的流程输入、观察者挂载的写录制层、模型的「只推进不比较」口子、红态正文里 `{failure:?}` 的 Debug 文字。下面逐样推。

| 输入 | 重建得出吗 | 依据与缺什么 |
|---|---|---|
| **镜像：基线** | 是 | `MemoryPool` 就是稀疏扇区表加盘字节数（`crates/singlefs-harness/src/memory_pool.rs:222`、`:30`），`0F` 装得下 [读] |
| **镜像：写内容** | 是 | `WrittenContents` 只有字节与整段清零两种（`crates/singlefs-harness/src/memory_pool.rs:230`、`:234`）。判表建表**拒收**枚举写表里的清零写（`copy/crates/singlefs-checker-tier/src/crash_judge_tables.rs:228`、`:408`），所以清零必须落在基线里：E5 之后给定状态表正是这样（`copy/crates/singlefs-checker-tier/src/crash_given_states.rs:22`）[读] |
| **镜像：持久集合** | 是，**不用逐态存** | 镜像是视图：基线 + 写表 + `persisted: Vec<bool>`（`crates/singlefs-harness/src/memory_pool.rs:431`、`:434`），不拷贝。流水线的持久集合是序号的确定函数（`copy/crates/singlefs-checker-tier/src/crash.rs:1580` 的 `layer0_enumeration_tables`，撕裂镜像与重放由基线和写表派生），`0F` 存段、每段展开方式、写表就够；给定状态表每态存显式掩码，≤ 2048 位 = 256 字节（`copy/crates/singlefs-checker-tier/src/crash_facts.rs:287`）进 `10/03` [读]。**条件**：序号到掩码的那份枚举代码要两段共用、且住在对比段的闭包里（Q3） |
| **恢复报告字段** | 是，但 `0C` 的 8 个字不够 | `0C` 每态八字、txg 只有低 32 位（`copy/crates/singlefs-checker-tier/src/verdict_store.rs:34`、`copy/crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:331`）。比较要的是：每遍结局种类、`effective_root`（`crates/singlefs-core/src/recovery.rs:344`）全宽、失败点码、读回长度、末字节、读回 SHA-256。journal 六个计数（`crates/singlefs-core/src/recovery.rs:325`）对结论没用，只有崩溃注入的计数项用到 `prefix_applied > 0`（`copy/crates/singlefs-checker-tier/src/crash_injection.rs:1567`），记成 `10/03` 里的小整数 [读] |
| 　其中红态正文 | **缺** | oracle 的违例原文是 `format!("走读失败：{failure:?}")`（`copy/crates/singlefs-checker-tier/src/crash.rs:1127`），Debug 文字带枚举的载荷，定长失败点码装不下。红态现场再跑一次 `recover` 补正文，同形先例是 `POOL_CHECKER_VIOLATION_WITHOUT_ITS_DETAIL`（`copy/crates/singlefs-checker-tier/src/crash_given_states.rs:60`）[推] |
| **读回内容** | 是，存摘要 | oracle 比的是 `content != version.content`（`copy/crates/singlefs-checker-tier/src/crash.rs:1158`），模型比的是字节相等（`crates/singlefs-harness/src/model.rs:3496`），RefFS 与挂载透明也是相等比较；摘要相等等价于字节相等（SHA-256 碰撞忽略 [推]）。违例说明文字用到长度与末字节（`crates/singlefs-harness/src/model.rs:3501` 起），一并录。版本表一侧存同一函数的摘要 |
| **模型各版** | 是 | `committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>`（`copy/crates/singlefs-checker-tier/src/crash_injection.rs:1410`）→ 行（根键，有无文件，摘要，长度，末字节）；`newest_persisted` 由写表和持久集合算（`crates/singlefs-harness/src/memory_pool.rs:821`），对比段自己算 [读] |
| **挂载写** | 崩溃注入是，观察者**缺录制层** | 崩溃注入的挂载已经录成写表（`mount_operations`、`publish_operations`，`copy/crates/singlefs-checker-tier/src/crash_injection.rs:1684`、`:1690`）。观察者的挂载跑在没有录制的 `SparseBlockDevice` 上（`copy/crates/singlefs-checker-tier/tests/common_crash_point_observers/mod.rs:155` 起），要套录制层才有写表 [读] |
| **续做输出** | 是 | 观察者断言读的量都是标量：结局枚举、`deletion_queue_continuation.len()`、`PoolVersion` 种类、`deleted_inodes` 是否空、一串发布的（实例，txg，事务号）（`mod.rs:178`、`:192`、`:202`、`:219`、`:295`）。`TransactionOutput` 其余字段不需要 [读] |
| 　其中观察者另读的流程输入 | **3.2 的 `0F` 没列** | 观察者读 `MountContinuationInputs` 的 `versions_with_a_file`、`deletion_publishes`（`copy/crates/singlefs-checker-tier/tests/common_crash_point_observers/mod.rs:137`、`:131`），它是测试代码按发布输出算的流程常量，不在基线、写表、段、被判根、版本表里。`0F` 要加一行，或并进版本表 [读，缺] |
| **挂完的盘** | 是 | 由崩溃镜像加挂载写重建（`mod.rs:224` 的 `mounted_disks` 就是这样得到的）|
| **R 字段** | 是 | `RecoveryFieldsRead`（`crates/singlefs-harness/src/mount_transparency.rs:113`）= 恢复报告 + 被比那一版的根记录原字节、内容、实例表行、树表条目原字节 + 489 字节系统配置 + 若干标量。变长的两项（`instance_table_rows`、`tree_table_entries`）存「条数 + 定长条目」[读] |
| **拒绝合法性的输入** | 是 | 崩溃镜像、R1 落到的那一版、拒因码、「只读读出在不在允许集合里」（B1 的结论）、读故障标记；「盘变没变」对比段现算（第 4.4 节）[读] |

### 5.1 RefFS：是集合，不是状态机（事实表 P11 写「拆不拆得开没核」，这里核了）

- **第一次崩溃**：`allowed_set_for_a_crash_in` 是只读函数（`crates/singlefs-reference-filesystem/src/lib.rs:375` 文档注释「只读，不动模型」），入参只有上一次恢复之后进了序列的操作（字段注释在 `crates/singlefs-reference-filesystem/src/lib.rs:260`）和崩溃号 hi；返回 `AllowedSet`，元素是 {lo，以及 lo 到 hi 之间每个发布末尾的状态}（`crates/singlefs-reference-filesystem/src/lib.rs:229` 起）。事件序列到崩溃时刻为止是翻译好的（`crates/singlefs-harness/src/reference_filesystem_translation.rs:977` 的 `replay_up_to`），**与读回无关**。一个历史上它至多「发布数 + 1」个状态 [读]。
- **重起头之后的第二次崩溃**：判绿后 `restart_from_the_recovered_state` 把操作序列清空、起点设为读出的内容（`crates/singlefs-reference-filesystem/src/lib.rs:467`、`:468`）；第二次崩溃取 `crash_after_every_operation_returned`（序列长度 + 1 = 1，`crates/singlefs-reference-filesystem/src/lib.rs:293`），序列为空时 `lower_bound` 取 `StateIndex(0)`（`:221`）、循环 `.take(upper_bound.0)`（`:236`）一圈都不转，允许集合就是 {起点} = {R1}。所以第二次崩溃的 RefFS 一票 = 「第一次判绿」且「第二次读出 == R1」；第一次没判绿时弃权（`crates/singlefs-harness/src/reference_filesystem_translation.rs:920`、`:922`）[读，推导]。**不必把「重起头的 RefFS」存进库**。
- **弃权的原因**（翻译停在某步、读故障、写后报错后又起操作）都由事件或标记定，不依赖读回，跑数据段录一个码 [读]。
- **要改的一处语义**：历史里的每次恢复现在是边翻译边判成员（`crates/singlefs-harness/src/reference_filesystem_translation.rs:530`、`:532`），不在集合里就停翻译、之后所有崩溃时刻判红（`:563`）。拆开后跑数据段改用 `restart_after_a_recovery_that_abandoned_the_newest_root`（`crates/singlefs-reference-filesystem/src/lib.rs:440`，不判、无条件重起头），同时录 (hi，允许集合，读出)；对比段判成员，并把「停在第 k 步」传给 k 之后的崩溃时刻。这样改后与今天判定逐项相同需要一条新旧对拍用例（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:63` 的剪枝自证同形）[推]。

## 6. Q3 键与版本：改了什么，作废哪几张表的哪些行

### 6.1 今天的键与版本是怎么算的 [读]

- 块键 = 输入指纹 ‖ 流名或节点名 ‖ 枚举计划哈希 ‖ 块起点（`copy/crates/singlefs-checker-tier/src/verdict_store.rs:22`）；「块键只由流程定」「判器改了不动块键」（`:23`）。计划哈希把基镜像、写表前缀摘要、根槽写、各版本的内容摘要都收了（`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:589` 的 `JudgeInputsOfTheFlow` 与 `:768`–`:795` 对 `versions` 的逐行摘要），所以**写表或期望内容变了，块键就变，旧行成孤儿、不会被误复用**。
- 结论行的键 = 块键 ‖ 结论种类（`GpuJudge`、`CpuJudge`、观察者，`copy/crates/singlefs-checker-tier/src/crash_identity.rs:541`）‖ 版本。版本 = SHA-256(头、种类、**代码摘要**、rustc、cargo、target、`extra_inputs`)（`copy/crates/singlefs-checker-tier/src/crash_identity.rs:626`）；`extra_inputs` 只有观察者非空，GPU 与 CPU 判器写死 `"-"`（`copy/crates/singlefs-checker-tier/src/crash_identity.rs:619`）。
- 代码摘要 = 从入口文件顺着 `use`、`mod`、`#[path]`、`include_str!` **按文件**收的闭包的词法单元串摘要（注释不进摘要）（`copy/crates/singlefs-checker-tier/src/judging_closure.rs:637` 起的入口与停止表；摘要形态 `crash_identity.rs:488`–`:491` 的文档注释）。入口：`crash.rs` 与 `crash_judge_gpu.rs`（`judging_closure.rs:638`、`:639`）；停止文件：流水线五份，其中 `crash_amplification.rs` 的理由是「先录后核的流水线：…一个状态判成什么由判器定，录制器只把写表交给它」（`copy/crates/singlefs-checker-tier/src/judging_closure.rs:681`）。

### 6.2 两句并排（冲突 / 缺口）

```
正文 body:39（整行）：
`0F` 流程输入（基线、写表、段、被判根、版本表）、`10` 状态跑数据（录入种类 `01` Rust 只读恢复两遍、`02` 观察者、`03` 崩溃注入各截）、`11` 挂载跑数据、`12` 模型与 RefFS 跑数据、`13` 写内容（按 SHA-256 去重）、`14` 每步跑数据；布局号 2 → 3；录入版本另起一族（录入那一段跑的代码闭包摘要：core、录制器、理想模型、RefFS），块键照旧只由流程定。
judging_closure.rs:681（整行）：
        "先录后核的流水线：录写表、排计划、领块、分工、对账、存库；一个状态判成什么由判器定，录制器只把写表交给它",
crash_identity.rs:619（整行）：
            Self::GpuJudge | Self::CpuJudge { .. } => "-".to_string(),
```

- **与 `judging_closure.rs:681` 的冲突**：录入段的代码（循环里调 `recover`、录库）若落在 `crash_amplification.rs` 里，它属于停止文件，**不进任何版本**，改它不换录入版本。所以录入版本的入口必须是一个**不在停止表里**的新文件（例：`crash_run_record.rs`），录入代码住在那里 [推]。
- **与 `crash_identity.rs:619` 的缺口**：3.2 没说对比结论的版本怎么绑它读的录入。按今天的写法，重录之后（核心的读路径改了，块键不变，录入版本变）旧结论行的键（块键 ‖ 种类 ‖ 判器版本）一模一样，`conclusions_missing_for_block`（`copy/crates/singlefs-checker-tier/src/verdict_store.rs:1929`）认为不缺，**新录入的数据一眼没看就复用了旧绿**。这就是「判出错的绿」。要补：对比结论版本的 `extra_inputs` 填它读的那几族录入版本的摘要（`crash_identity.rs:618`–`:624` 的 `version_of_code`），对比段读库只认当前录入版本的行，没有就退 78，不读旧行 [推]。
- 反过来（旧录入配新判法）不会出错：录入行的键带录入版本，对比段只读当前版本那一族；判法版本变只重比、不重录 [推]。

### 6.3 改了什么，作废什么

录入版本拆成三族（见 6.4 的理由）：R10 = 核心（读路径、挂载）+ 录制器，管 `10`、`11`；R12 = 理想模型 + RefFS + 翻译，管 `12`；R14 = 核心 + 理想模型，管 `14`。`0F`（流程输入）与 `13`（写内容）是内容寻址的，不带版本。

| 改了 | 块键 | `10`/`11` | `12` | `14` | GPU 对比结论 | CPU 对比结论 | 重做的代价 |
|---|---|---|---|---|---|---|---|
| 核心**写路径**（mkfs、事务写出的字节） | 变（写表变）：全部旧行成孤儿 | 新键下全录 | 新键 | 新键 | 新键 | 新键 | 整域重录重比 |
| 核心**读路径/挂载**（`recovery.rs`、`mount.rs`） | 不变 | **作废**（R10 变） | 不变 | **作废**（R14 变） | 作废（绑 R10、R14） | 作废 | 重跑 Rust 恢复：上界 3.7–5.4 天（Q5） |
| 录制器（切段、录制流） | 写表不变则不变；变则同上 | 作废 | 不变 | 不变 | 作废 | 作废 | 同上 |
| 判器：GPU 内核、CPU 对比代码 | 不变 | 不变 | 不变 | 不变 | 只作废对应种类 | 只作废对应种类 | 只重比，GPU 一小时量级（Q5） |
| 理想模型 `model.rs` | 不变 | 不变 | **作废**（R12 变） | **作废**（R14 变） | 作废（绑 R12、R14） | 作废（随机历史两项的 CPU 对比经 `crates/singlefs-harness/src/model.rs:3433` 引到它；放量各项含不含它取决于设点文件的闭包，我没逐项算 [复核不了：要编译或跑 build.rs 的摘要表]） | 重跑模型与翻译，不重跑 Rust 恢复 |
| RefFS 包 | 不变 | 不变 | **作废** | 不变 | 作废（绑 R12） | 作废 | 同上 |

GPU 对比版本的闭包里没有 `model.rs`（WGSL 不引 Rust），所以**模型的比较规则改了而 WGSL 里的那份没跟着改**时，版本不会提醒；这条靠 CPU 对 GPU 的对拍（J6）逐态抓，因此新内核的对拍要进快档，且要有「往加速那一侧塞一个已知故障，对拍要红」的自证（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:82`）[推]。

### 6.4 必须同批做的两件事（否则版本分族等于没分）

1. **按文件拆 `crash.rs` 与 `crash_injection.rs`**。闭包按文件收，`crash.rs`（5538 行）今天同时住着：把序号解成持久集合的枚举表（`copy/crates/singlefs-checker-tier/src/crash.rs:1506`、`:1580`）、判（`:1367`）、旧引擎（`:1773` 起的 `enumerate_layer0_*`）。录入段要引枚举表，若不拆，**对比代码的任何一处改动都会换录入版本，等于白白重跑几天的 CPU** [推]。拆法：枚举表放一个只被两段引用的叶子文件；录入入口一个文件、CPU 对比入口一个文件。`crash_injection.rs`（9059 行，单测从 `:5249` 起）同理；`model.rs` 里模型推进与 `crash_recovery_disagreement`（`:3433`）也可挪进 `model_comparison.rs`，这样模型推进代码改了不换对比版本 [推]。
2. **录入族拆成三族，不要一个摘要**。一个摘要意味着改理想模型也要重跑 Rust 恢复；拆三族，模型改了只重跑 `12`、`14`。

## 7. Q4 体量：每态 160 字节的账、去重、压缩、三档

### 7.1 状态数：问题里的 26.16 亿不是今天登记的全量

两句并排：

```
正文 body:23：- 规模：CLAUDE.md 记展开 28 全枚举七个分项 26.16 亿态，GPU 判一趟 7 小时 37 分；每态在 CPU 上跑 Rust 恢复两遍的速率在今天的树上没量过。
scripts/crash/crash-amplification.sh:222：FULL_EXPAND_UP_TO=30
.claude/kb/checks-owed.md:610（C761 行，整行）：| C761 | 9/30 崩溃放量全量「26.16 亿态零违例」里多记录流没穷举 | 2026-09-30 那一趟崩溃放量全量（展开 28、七个分项含 σ）报的「26.16 亿态零违例」，多记录流那一项是 `exhaustive=false`（最长一段 30 写，第 32 批 Q 探针 `PROBE_SEGMENT_SIZES`；那时入口还没有「不穷举判红」这一判）；2026-10-08 展开 30 重跑多记录流与删除流两项都 `exhaustive=true`、0 红（日志 `~/.local/share/singlefs/work/m3-3-4-amplification/run-gpu-expand30.log`）。别的五项在展开 30 下穷不穷举没核。全仓还在引那句的地方（CLAUDE.md 当前里程碑一段、里程碑二记录、kb）要回扫 | 回扫按旧说法搜「26.16 亿」「2 616」「全枚举七个分项」，逐处判现状句还是那一次的事；入口 `scripts/crash/crash-amplification.sh` 全量里一项不穷举就判红已经立了（2026-10-08） | 下一次阶段同步；另外五项要一趟展开 30 的全量 | records/2026-10-01-删除与并发事务.md 2026-10-08 那两条 |
```

- 今天的全量深度是 30（`scripts/crash/crash-amplification.sh:222`），登记 8 项（`:230` 起的 `ITEMS`，比 9/30 那趟的 7 项多了删除流）。
- 9/30 那趟 28 深的账：合计 2 615 501 475 态是各项状态数直接相加（`briefs/2026-09-28.md:86`），到 D 那段在到 E 里数了两遍；实判 2 563 727 926 态（`README.md:60` 的括注「这一趟实判 2 563 727 926 态」）[读]。
- 10/08 那趟 30 深：多记录流 1 342 177 375 态、删除流 1 448 083 638 态（`records/2026-10-01-删除与并发事务.md:1292`），删除流把多记录流的 A、B、C 三段当前缀，块键相同，库里只记一份 [读，前缀关系读测试头注释 `copy/crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:3`]。
- **30 深去重后的下界** = 2 563 727 926 − 268 435 552（多记录流 28 深那份）+ 1 448 083 638（删除流 30 深，已含多记录流）= **3 743 376 012 态** [算]。下界，因为别的五项在 30 深下的态数没量过（C761 说「没核」）。

### 7.2 每态多少字节

事实表记 160 字节 [读 `report.md:136`]，其里每遍 72 字节含 journal 三个计数。我按比较真要的字段重算 [算]：每遍 = 结局种类 1 + `effective_root`（有无 1、实例 4、txg 8）+ 失败点码 4 + 读回长度 4 + 读回 SHA-256 32 + 末字节 1 = 55，两遍 **110 字节**；去掉有无标志的对齐，取 106–112。两个口径都列：160（事实表）、106（最小字段）。一块 `STATES_PER_BLOCK` = 65 536 态（`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:153`）：160 字节 = 10 MiB 一行，一趟 GPU 读库带宽 7.2×10⁵ 态/秒 × 160 = 115 MB/s [算]。

### 7.3 原始体量（未压缩）[算]

| 域 | 态数 | ×160 B | ×106 B | 调色板形态（2 B/态）|
|---|---|---|---|---|
| 问题口径 26.16 亿 | 2 616 000 000 | 418.6 GB（389.8 GiB） | 277.3 GB | 5.2 GB |
| 9/30 实判 28 深 | 2 563 727 926 | 410.2 GB | 271.8 GB | 5.1 GB |
| 今天全量 30 深（下界） | 3 743 376 012 | **598.9 GB**（557.8 GiB） | 396.8 GB | 7.5 GB |

磁盘余量会变，不能当前提：开工时 `df -h ~` 是 `/dev/nvme0n1p2  3.6T  2.9T  514G  86% /`，交回前 `df -h ~/.local/share/singlefs` 是 `/dev/nvme0n1p2  3.6T  1.7T  1.8T  50% /`（同一个盘，两次相隔我这一趟的工作时间）[读，命令原样输出]。背景材料写的「余 569G」是更早一刻的数。按 569G 算 30 深下界的 160 字节形态（598.9 GB）**落不下**，按 1.8T 落得下；这个判断要在开跑前现查。

背景材料另一句「判定库今天 4.4G」：`du -sb ~/.local/share/singlefs/crash-amplification` 是 4 657 296 813 字节，其中 `runs/`（日志与取回的对端库）4 372 436 324，库目录 `library/` 只有 283 992 607 字节 [读，命令原样]。而且那份 `library/` 是**按结论种类存之前的旧布局**：入口默认的新库目录是 `library-by-conclusion-kind`（`scripts/crash/crash-amplification-lib.sh:93`），盘上不存在 [读：`ls -d ~/.local/share/singlefs/crash-amplification/library*` 只出 `library`]。所以现有库的字节数**不能当作 `0A`–`0E` 或新表的压缩比样本**。

### 7.4 写内容 `13` 去重之后多大

多记录流 A、B、C 三段：12、14、15 个单元，每单元两份（24、28、30 写，`copy/crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:3`、`:20`），数据单元 32768 字节、索引节点 16384 字节（`crates/singlefs-core/src/recovery.rs:365`–`:367`）：A 212 992、B 262 144、C 294 912 字节每份，两份共 1 540 096 字节 [算]。再加 journal 记录（4096）、根槽、系统配置槽写，一条流 ≲ 2–3 MiB，十二项按前缀共享去重后总共 ≲ 30 MiB [推]。**`13` 比 `10` 小四到五个数量级，去重与否都不是体量问题**。GPU 判表本身是派生数据不入库：`tables_mebibytes=12`（`~/.local/share/singlefs/work/m3-3-4-amplification/run-gpu-expand30.log` 的 PLACEMENT 行）。

### 7.5 观察者与随机历史

- 观察者挂在标准前缀三个节点上（`copy/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:206` 的 `OBSERVERS_ON_THE_STANDARD_PREFIX`），用户定「共享前缀的每个状态都做，不得例外」（`:190`）。多记录流这三个节点是 A（24 写）、B（28 写）、C（30 写）三个单元段，30 深全展开是 2²⁴ + 2²⁸ + 2³⁰ = **1 358 954 496 态** [算]，与测试头注释写的「枚举域 1358954634 个状态」几乎重合（`…multi_record_publish_stream.rs:21`），即**这条流几乎全部状态都要挂载**。事实表的 `11` 估法「头约 100 字节 + 每次写 50 字节」，按每态 10 次写 = 600 字节、1.359×10⁹ 态 = **815 GB 未压缩** [算，写数是推的]。同一份挂载写在很多态里重复，只有「挂载写表的分组」（第 9.3 节最后一条）是压得下去的路。
- 随机历史快档：24 段历史（`copy/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:179`）× 每段 4 个崩溃点（`:182`）= 96 个崩溃点，按事实表每点 6–12 KiB（推的），≤ 1.2 MB [算]，不是体量问题。

### 7.6 RocksDB 的压缩与写放大

- 压缩：`options.set_compression_type(rocksdb::DBCompressionType::Lz4)`（`copy/crates/singlefs-checker-tier/src/verdict_store.rs:811`）。每批同步写、WAL 开着（`:819`）。
- 压缩比我**没有可用的样本**：旧布局库没有 `0C`（上一小节）。一个旁证（不是样本）：10/08 那趟取回的对端库 169 270 644 字节，对应对端判的 409 862 144 态，即 0.41 字节/态；本机送出的 187 508 594 字节对应本机判的 903 610 463 态，0.21 字节/态（哪份库对哪份态数是我从 share-0、share-1 日志推的）（`~/.local/share/singlefs/crash-amplification/runs/crash-amplification-2026-10-08-2361592-two-hosts/` 下 `fetched`、`library-for-peer` 的 `du -sb`，日志 `share-1.log`）[算]。这些是每态 1 字节的判定向量编号加键、WAL，说明**同值成片的块压得很狠**，但 `10` 是 106–160 字节、同值成片程度取决于一块里不同落点的个数，不能外推 [推]。
- 写放大：没量过。E162（崩溃放量判定块存储选型）只量了持续写入速率与占用比，对应产物已归档，不在今天的树上。一般经验（WAL 一遍、刷盘一遍、各层合并若干遍）**未在本项目验证**，不进结论。量法写在第 13 节 M3。

### 7.7 三档各要多少、能落下的形态、各丢什么

| 档 | 域 | 160 字节原始 | 说明 |
|---|---|---|---|
| 轻型 | 「甲二」∪ 展开 6（`scripts/crash/crash-point-replay.sh:162` `LIGHT_EXPAND_UP_TO=6`） | 我读到的展开 6 日志行（`runs/` 下几趟的 `CRASH_AMPLIFICATION flow=…` 行）每项十几到几百态，十三行共约 480 态，固定脚本两项那趟没出数；< 10⁴ 态 → < 2 MB [推] | 新建空库，跑完删（`scripts/crash/crash-point-replay.sh:532`、`:29`） |
| 重型·剪枝 / 逐条 | 只补键不同的块 | 首次等于全量；之后只有变了的项 | 判器改了只重比；核心读路径改了重录 |
| 重型·全量（清空复用） | 今天登记深度 30，3.74×10⁹ 态下界 | 598.9 GB | 新库放 `crash-point-replay-full/<时戳>-<进程号>/library`（`scripts/crash/crash-point-replay.sh:542`），跑完不删 |

| 形态 | 30 深下界的大小 | 丢什么判别力 |
|---|---|---|
| F0 逐态定长 160 B，只靠 LZ4 | ≤ 598.9 GB，LZ4 后不知道 | 不丢 |
| F1 只存比较用到的字段（106 B） | ≤ 396.8 GB | 红态正文里 `{failure:?}` 的载荷与 tally 的 journal 计数；红态现场重跑 `recover` 补 |
| F2 块内调色板：每块一张去重的记录表 + 每态 u16 下标 | 约 7.5 GB（2 B/态）加每块的表；前提是一块里不同记录 ≤ 65 536（`0A` 的编号是 1 字节、上限 256 种，`copy/crates/singlefs-checker-tier/src/verdict_store.rs:1042` 的 `register_verdict_vector`，这里要 u16） | **不丢**（无损），代价是读要先读调色板 |
| F3 比完一块就删该块 `10` 行，只留红块 | 峰值 = 在飞块数 × 10 MiB（几百 MiB） | **丢复用**：判器一改要重跑几天的 CPU；红块之外的态没法在不重跑 Rust 的情况下再比。违背 `.claude/singlefs-ai-sop/rules/test-script-discipline.md:58`「每一份判定按它依赖的全部输入算键存下」的初衷 |
| F4 按 Rust 读到的字节记忆化 | 取决于命中率，没量过 | 不丢结论；但不是「每个状态都跑」的字面，要用户点头；剪枝自证要进快档（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:61`、`:63`） |

我的推荐顺序 [推]：F2 加 LZ4 为默认；`11`（观察者挂载写）先做「按挂载写表分组」；F3 只当盘不够时的应急，且报告里写明丢了复用。

## 8. Q5 CPU 代价：量法、分片、与 GPU 对比段怎么流水

### 8.1 已有的数（都是旧数，只当参考，不当依据）

- GPU 一侧：10/08 展开 30 那趟，本机两张卡 `gpu_states_per_second=338821`、第二台三张卡 `380989`（`~/.local/share/singlefs/work/m3-3-4-amplification/run-gpu-expand30.log` 第 18、23 行）；合起来约 **7.2×10⁵ 态/秒** [算]。9/28 汇总里到 E 那一项均值 9.9 万态/秒是按该项总用时算的均值（`briefs/2026-09-28.md:88`）。
- CPU 一侧：旧记录「CPU 判器两台合计每秒约 8 千态，按那个速度七项要近四天」（`briefs/2026-09-28.md:88`）。这个速率是**整个 CPU 判器**（两遍 `recover`、两份 oracle、池级 checker、记录核对器）的，不是 `recover` 单独的；两台各 31 线程（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:62`）折成每线程每态约 62/8000 = 7.75 ms [算]。本机 `nproc` 是 32（`lscpu`：Ryzen 9 9950X，实测）；第二台的线程数 [复核不了：要登第二台]。
- 所以拆开后：对比段（GPU）约一小时（2.56×10⁹ 态 ÷ 7.2×10⁵ = 1.0 h；30 深下界 3.74×10⁹ 态 = 1.44 h）[算]；跑数据段（CPU）的**上界**是整个旧 CPU 判器的速率，2.56×10⁹ 态要 3.7 天、3.74×10⁹ 态要 5.4 天 [算]。`recover` 单独占旧 CPU 判器多大比例没量过，可能远小于 1（池级 checker 走树，更重）[推]。**不管比例多少，墙钟由 CPU 定**，GPU 先天空等，对比段没有流水压力。

### 8.2 观察者那一块（P5）是另一笔账，与拆不拆无关

`seconds_observing=9.2`、`states_observed_here=140`、`blocks_observed_here=16`（`~/.local/share/singlefs/crash-amplification/runs/crash-point-replay-2026-10-10-3703484/fast-tier.crash_enumeration_multi_record_publish_stream.log` 第 182、184 行；另一条流 9.5 秒、140 态）：这是展开 6 的小域，墙钟 9.2 秒除以 140 态 = **每态约 66 ms**，含存库、起线程的固定开销，线程数没记，不是吞吐量测量，只说明量级 [算]。观察者在多记录流上挂的三个节点覆盖 1.359×10⁹ 态（第 7.5 节）；按 66 ms × 1.359×10⁹ ÷ 62 线程 = 约 17 天 [算，纯外推，可能高估也可能低估]。**这一块和「跑数据 / 对比数据」拆不拆无关，是 J4 观察者「共享前缀的每个状态都做」自带的 CPU 账**，只是拆开后挂载的写还要额外录库。需要先量（8.3），再谈要不要对用户摊开。

### 8.3 量法：哪条命令、在哪棵树上、量到哪算够

在**副本**上（六份补丁之后的树，不动主工作区）加计时，主 agent 或实现员跑：

1. 在 `copy/crates/singlefs-checker-tier/src/crash.rs:1374`、`:1375` 两个 `recover` 前后各取一次 `Instant`，按线程累加纳秒与调用数；块末打一行 `RUN_COST pass=consulted|ignored ns=… calls=…`。观察者在 `observe_one_piece`（`crash_amplification.rs:10025`）里对挂载与 `check_pool_image` 分开计时。
2. 命令（重型，主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request`）：`bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/hosts/capped.sh <线程> bash scripts/crash/crash-amplification.sh --item multi-record-publish --expand-up-to 16`，`ENABLE_GPU=0`（要量的是 CPU），环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=<新建的空目录，不放 /tmp>`（入口读它，`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:7609`）；再换 `--item fixed-script-through-unmount` 与 `--item position-addressed-trees` 各一次。线程数取 1、8、32 三档。
3. 「量到哪算够」[推]：同一项连跑 3 次，每态纳秒的相对极差 < 10%；落到 `FileRead`、`NoFile`、`Failed` 三种结局的态各 ≥ 10⁴ 个（读回内容的代价依结局不同）；三项各 ≥ 2×10⁵ 态；线程数 1→8→32 近似线性（`recover` 无共享锁）；报 `recover` 两遍占旧 CPU 判器总用时的比例，乘 7.75 ms 得每线程每态的预测，再与新写的跑数据段整段墙钟对一次。

### 8.4 分片与流水

- 单位是块（65 536 态）。两台按权重分块：`host_shares.owner_of_block(*block) == share.index`（`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:7692`）。建议**块亲和**：哪台跑某块，就在哪台比某块，对端库只把结论表带回主库，`10`–`14` 不回传（省下百 GB 量级的网络与盘（7.3 节的数））；这样 `import_from`（`copy/crates/singlefs-checker-tier/src/verdict_store.rs:1566`）只要认 `0F` 和结论表，`10`–`14` 可选 [推]。
- 线程：跑数据段沿用 `checking_threads()`（`:1368`）把一块切成线程数段并行、按块序单线程写回，输出与线程数无关（今天的做法，`:1524` 的文档注释）。
- 与 GPU 流水：生产者 = CPU 线程，每块写完（`set_sync(true)` 同步落盘）即对 GPU 可见；消费者 = GPU 对比线程，按「库里有 `10` 行而没有当前版本结论行」取块——现成的 `pending_blocks`（`:2130`）就是「录了没判」的对账函数，换成「录入完成没比」即可 [推]。K 个在飞块的内存 = K × 10 MiB，K 取卡数的小倍数即可。
- J6 的 CPU 对照（`check_the_cross_check_blocks_beside_the_library`，`:9704`）今天在旁边那份库里**自己再跑一遍 `recover`**，和跑数据段抢同一批核。拆开后 J6 的 CPU 对比改读 `10`，不再重跑 `recover`，这是净省的一笔 [推]。
- 不许拿挂钟长当收窄枚举域的理由（`.claude/main-agent.md:10`「挂钟长不是收窄它的理由」）：下面只列**不收窄域**的省法，要不要用交用户——F4（按 `recover` 读到的字节记忆化，每个状态的结果仍是一次真实 `recover` 的结果或与某次真实结果的读集相同的复用）：证明义务是「`recover` 是读到字节的纯函数」，成立的理由是镜像是视图、读只经 `PoolReader::read`（`crates/singlefs-harness/src/memory_pool.rs:445`）；用法是记录第一次的读集（设备，偏移，长度，字节），后面的态只比读集上的字节。**它不等于「每个状态都跑」的字面**，要用户点头，且必须带一条关掉记忆化跑一遍、与开着时逐项相同的对拍进快档（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:61`、`:63`）。

## 9. Q6 GPU 一侧做不做得出

### 9.1 预算的账 [读、算]

- 预算 `LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL` = 400 000（`copy/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:247`），**按单个内核**算：`refuse_a_kernel_too_large_for_the_driver(label, source)`（`:391`）每个内核源码各过一次。钉值（`:1502`–`:1514`）：恢复内核 334 042、扫描内核 395 566、走读内核 352 481；余量 400 000 − 334 042 = 65 958、400 000 − 395 566 = 4 434、400 000 − 352 481 = 47 519。三个内核的源码都是常量前言、`crash_judge_common.wgsl`（3 730 行）、各自的一份拼成（恢复内核：`copy/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:420`–`:422`，一共 3 730 + 656 = 4 386 行 → 334 042，**平均每个源码行展开约 76 个表达式**，含调用点逐处展开）[算]。
- **余量只对「往这三个已有内核里加东西」有意义**。新建的「录入比对」内核是独立源文件，自己有 400 000，不占这三个的余量 [读]。
- 编译边界没量死：注释写「编不编得过的真实边界在 27 万与 98 万之间，没量过」（`copy/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:246`），第二台 580 版驱动曾在 98 万时 48G 里编不完（`:244`）。现有三个内核（33–40 万）已经越过那条注释里已知编得过的 27 万，所以新内核最好留在 27 万以内 [推]。

### 9.2 逐类

| 要加的 | 加在哪 | 展开量 [推] | 做得出吗 |
|---|---|---|---|
| P1–P3：读 Rust 恢复表，比落点、结局、失败点、读回摘要 | 恢复内核（`C crash_judge_recovery.wgsl`）里一处比较；**不再多调一次** `check_records` | 几千：`classify_oracle` 只有 58 行（`C crash_judge_recovery.wgsl:294` 到 `:351`），按平均 76 表达式/行上界 ≈ 4.4 千；比相等本身几十个 | 做得出，余量 65 958 够。理由：记录核对器只读 `consulted.effective_root` 与镜像（`copy/crates/singlefs-checker-tier/src/crash.rs:1389`），Rust 落点与 WGSL 落点相等时，WGSL 那一次的记录核对结果就等于以 Rust 落点算的结果，**不必再调一遍**；不等直接判红。若硬要按录下的落点再调一次 `check_records`（`:539` 起，带 `load_instance_table_of_the_effective_root` 等整串），就是多一个大调用点，会超或逼近余量 |
| P5 四条断言、P7、P10、P11、P15、P16 | 新文件 `crash_record_compare.wgsl`（丙），**不带** `crash_judge_common.wgsl` | 每样几百到几千，全并起来 ≲ 2 万，占预算 5% 以内 | 做得出。它们读的都是定长小表，做字段相等、摘要相等、集合成员、(txg，实例) 序比较；P15 逐字段比的是 489 字节系统配置与根记录原字节的区间，不解析 |
| P12–P13 的记录核对器第二份掩码 | 恢复内核的 `write_in_place`（`:352`）、`unit_copy_is_missing`（`:389`）两个小函数的第二份掩码变体 | 几百到几千 | 做得出，条件是每态两份掩码（崩溃态、恢复后），表加一列；`check_records` 源码自己注明「两份镜像同一份，一条录制流」（`:538`），所以第二截不是现成路径 |
| **P14 判据复算** | 新内核，复用 `crash_judge_common.wgsl` 的系统配置、根环、journal、实例表解析 | **7–25 万**：Rust 是 535 行（`H mount_transparency.rs:2352` 到 `:2886`），写成 WGSL 约 600–800 行新代码，按 20–76 表达式/行 = 1.2–6 万，再加它调的 common 解析的展开（恢复内核整份 common + recovery 才 33 万），上限我估到 25 万 | 做得出，**风险最大**：最重的几个要读账树节点（`H mount_transparency.rs:2683`、`:2721`、`:2796`、`:2839`、`:2868`），若连带把走读内核的树读法拉进来会逼近 27 万的不确定带。先写最重的两个，用 `kernel_inlined_size`（`copy/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:263`）量，再决定拆成一个内核还是两个 |
| P8 每步模型比 | 新小内核，或不上 GPU | 见下 | 见下 |

### 9.3 P5 判表形态上限：用量 [推]

- **每落点 64 次写**（`copy/crates/singlefs-checker-tier/src/crash_judge_tables.rs:51`）。系统配置槽是被写次数最多的落点：多记录流从取号到 C 的系统配置槽写，点名里是 `acquire_instance_and_warm_up.system_configuration_slot.1`/`.2`/`.3`、`publish_first_file.system_configuration_slot.1`、`sequential_write.system_configuration_slot.1`…（那一趟日志的 `CRASH_AMPLIFICATION_BLOCK` 行），两盘各两个槽轮换，每个落点个位数；一次挂载再加取号写行、暖机两次、续做每步一次，约 8–10。合计十几次。最坏的是 σ 流，系统配置 0 号槽写了 34 次（`briefs/2026-09-28.md:85`）加挂载十来次，约 44 < 64。**不会顶到**。
- **持久掩码 2048 位**（`copy/crates/singlefs-checker-tier/src/crash_facts.rs:287`），超了判表建不出（`EnumeratedWritesPastTheMaskWidth`，`copy/crates/singlefs-checker-tier/src/crash_judge_tables.rs:242`、`:389`）。多记录流写表约 150 次、加撕裂镜像与重放，再加一个挂载组的 10–120 次写，≈ 300–500，远低于上限。**随机历史是另一回事**：一次发布约 24–30 次写（多记录流 A、B、C 三段 24、28、30 写，`…multi_record_publish_stream.rs:20`），2048 ÷ 30 ≈ 68 次发布。快档 24 步历史（`copy/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:180`）≈ 24 × 30 + 挂载与之后一次发布 ≈ 900，够；**历史长过约 60 步的大档，开 GPU 时判表建不出**，而 P9 要求建不出就判红、不退回 CPU。这是 3.3 没写的 GPU 路对随机历史规模的硬上限（内核容量，不是挂钟）。
- **挂载写的分组数**：挂载写由（落到的版本，实例代号与 txg 这几个标量，journal 前缀长度）决定，组数不超过这几个量的乘积，估几十 [推]；没量过。量法：在观察者里对每个态的挂载写表取 SHA-256，数一个展开 16 的域里有几个不同值；接近状态数就说明退化成一态一表，要改形态。

### 9.4 P8 每步模型比：搬上 GPU 做不做得出，量从哪来

- 能搬的是**比**，不是 `answer_*`。`answer_*`（`H model.rs:2030` 起）是模型的「步进」，属跑数据；比较是 `judge_and_advance`（`H model.rs:2874` 到 `:2985`）与 `judge_effect`（`:3014` 起）约 510 行。它还会读模型状态：`capacity_wall_is_permitted` 读 `unit_area_slots_per_device()`（`H model.rs:2854`），`judge_effect` 读 `effective_rollback_floor()`（`:3036`）与 `geometry`。这些常量要由跑数据段在每步随答案一起录下来，**事实表 D8 只写了「实现交回值与模型的答」，漏了这几个模型派生常量** [读]。
- 卡点：`ModelAnswer.expected_roots` 是变长、嵌套（`ModelVersionContents` 含 `FileBeingDeleted`，`H model.rs:2976`），要定长化；违例说明文字（`describe_answer`，`H model.rs:3513`）留 CPU。快档 24 步 × 24 段 = 576 步，GPU 没有吞吐收益，做它只为口径一致 [推]。
- 量从哪来：`HistoryRun.outcomes` 与模型每步的 `answer_*` 返回值；数据量是步数 × 几十到几百字节（事实表推的，没量）。
- 无论搬不搬，都**要先给模型加一个「按它自己的答推进、不比较」的口子**（现在对不上就不推进，`H model.rs:2870`；成功分支调 `advance_by_success`，`:3393`）。

## 10. Q7 次序与簇

### 10.1 改名放在两段改造之前：对，但理由要换

- **判据**：结论版本的代码摘要取词法单元串，注释不进、标识符与字符串字面量进（`copy/crates/singlefs-checker-tier/src/judging_closure.rs:21` 的 `lexical_tokens`）。我数了 core、harness、checker 三个包的 `src/` 里「layer0 / 层 0」的**非注释**命中：只有一处，`crates/singlefs-harness/src/model.rs:3656` 的字符串「第 1 层 0..=1」，是误命中 [读，`grep -rniE 'layer0|层 0' … | grep -vE '^[^:]+:[0-9]+:\s*(//|///|//!)'`]。所以**改名不会换核心、录制器、理想模型、RefFS 这条线上的任何摘要**；会换的是 `crates/singlefs-checker-tier` 里 `Layer0*` 类型与 `layer0_progress` 模块名所在闭包——这些在对比段的闭包里（`crash.rs` 是入口，`judging_closure.rs:638`），不在录入段。
- 但 6.4 要求把 `crash.rs` 拆开后，**枚举表那一份叶子文件两段共用**，里面有 `Layer0SegmentExpansion` 之类的名字（`copy/crates/singlefs-checker-tier/src/crash.rs:1506`、`:1580` 一带），改名会换录入段的版本。结论：**改名必须早于第一趟非一次性的新布局整域录入**（几天的 CPU），其余顺序随意；现在新布局还一趟整域都没跑过（`library-by-conclusion-kind` 不存在，第 7.3 节），所以现在改名零代价。
- 另一个理由是合并冲突：改名碰 1 751 + 1 886 份文件，主工作区还有 8 个别的会话在改（`records/2026-09-30-里程碑三开工第一轮.md:1000`）；簇的补丁都按改名后的名字写，就不用一份份重放改名。3.4 的次序（先合 V…E6，再在主工作区一次改名，再派簇）成立 [推]。
- 前提：改名那一趟不动 `tests/crash_enumeration_*` 与 `common_crash_points/mod.rs` 的**路径与崩溃点名**——节点身份是 `<仓内路径>::<崩溃点名称>::<复用键>`（`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:255`），进块键；这些路径一动，块键全变，旧行成孤儿。改名清单（`~/.local/share/singlefs/work/m3-merge-impl/rename-sweep/report.md` 第二节）里我没看到这些路径被列进去 [读，只扫了第 83–130 行]，这一条请主 agent 在改名前用 `git mv` 清单对一遍。

### 10.2 簇之间漏的依赖与文件 [读、推]

1. **丙依赖丁一、戊**：事实表第 6 节（`~/.local/share/singlefs/work/m3-merge-impl/gpu-split-survey/report.md:209`）把丙与别簇的交集写成「不交」，3.4 照它排。但新内核必须有 CPU 参照并逐态对拍、且要证明对拍会红（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:81`、`:82`），而 CPU 对比那一半归丁一（P7）和戊（P10–P16）。对拍用例要引两边，丙的验收只能排在它们的 CPU 判那一半之后，或这几个 CPU 参照同批交丙。
2. **甲的接口得是「通用的」**：甲只给「表标签 + 键 + 值字节 + 编解码」的框架，每种记录的字段由各簇在自己的文件里定义。否则每个簇的字段一变都要回头改 `verdict_store.rs`（今天 3 219 行、是最热的文件）。
3. **文件拆分是甲之前的「簇 0」**（6.4）：`crash.rs`、`crash_injection.rs` 按「枚举（叶子）/ 录入入口 / CPU 对比入口」拆开，`GivenStatesEngine` 那一节（`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:8252` 到 `:9293`，约 1 040 行）挪走。纯搬家、判定不变，用「判法摘要里的词法单元串多重集不变」当验收。这一步也解掉丁一与戊撞 `GivenStatesEngine` 的问题（E5 只挪了类型、trait 与 CPU 就地判，`GivenStatesEngine` 还在 `crash_amplification.rs:8361`，**3.4 的这一句今天仍然成立**）。
4. **戊的文件表漏了**：`crates/singlefs-harness/src/model.rs`（P8 要的「只推进不比较」口子在 `:2874` 到 `:3393`）；三份用 `crash_injection` 公开接口的集成测试——`crash_injection_writable_mount_after_the_crash.rs`、`crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs`、`crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs`（`inject_crashes_into_history(` 在测试里共 13 处调用，`HistoryThenWritableMountRecords`、`mount_and_publish_once_on_the_crash_image` 的签名一变它们就编不过）。
5. **`lib.rs` 是每个加模块的簇的撞点**（36 行，丙加内核模块、簇 0 加新文件都要改它）：`apply-writer-patch.py` 只对 `mutations.tsv` 按名字合并（事实表第 6 节末），`lib.rs` 的撞要在派发提示的「要动的文件」里错开。
6. **门禁与脚本侧没有簇**：`scripts/gate/admission.py`、`crash-case-check.py`、`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 登记、`crash-point-replay.sh`、`lib_crash_point_replay_items.py`，以及 `.claude/rules/verification.md:34` 的快档命令（要加 `--features verdict-store`，Q8）都得改；它们不在 `crates/`，实现员写不了，归工具员，3.4 没排 [推]。
7. **己的替代检查没写**：删掉豁免表后，原来的 gpu-exemptions 格只核「标记与表行对得上」（`scripts/crash/gpu-exemptions-check.py` 文件头），没有它就没有任何东西拦「开了 GPU 仍在 CPU 上对比」。记录 `records/2026-09-30-里程碑三开工第一轮.md:996` 同一条里已说要「另立会红的检查」，3.4 的己只写了删。形态建议 [推]：CPU 对比入口在 `routes.crash_judge == Gpu`（`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:7596`）且不在 J6 参照上下文里被调用时直接 panic，配一条「开 GPU、毒化 CPU 对比入口」的用例进快档（有卡时跑，没卡报本次未跑）。

### 10.3 丁二迁完 14 份测试文件，旧引擎 `enumerate_layer0_*` 留不留：留

并排：

```
正文 body:47（整行）：
先合 V…E6；再在主工作区一次做完全仓改名；再按事实表第 6 节的簇派：甲（库布局与录入版本）先做，之后乙（判表与判器内核）、丙（新写「录入比对」内核）、丁一（流水线与观察者）、戊（崩溃注入与 harness 拆口；与丁一撞 `GivenStatesEngine`，先把那一节挪成单独文件）、丁二（层 0 旧引擎用例迁移，14 份测试文件），最后己（豁免表与八处标记同批删）。
grep -c 'enumerate_layer0' copy/crates/singlefs-checker-tier/tests/*.rs | grep -v ':0$'（17 行，原样）：
crash_enumeration_acquisition_barrier.rs:2
crash_enumeration_floor_raise_pushed_by_admission.rs:2
crash_enumeration_multi_record_publish_stream.rs:5
crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs:2
crash_enumeration_writable_mount_of_a_formatted_pool.rs:2
crash_enumeration_position_addressed_trees.rs:2
crash_enumeration_sharded_across_processes.rs:3
crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs:3
crash_enumeration_fixed_script_stream.rs:7
crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:3
crash_enumeration_new_pool_file_creation_stream.rs:4
crash_points_of_the_instance_table_sweep.rs:3
crash_segments_per_device_and_torn_in_place_overwrites.rs:3
crash_enumeration_resumes_from_its_progress_file.rs:3
crash_enumeration_record_spill_over_stream.rs:2
crash_enumeration_tree_split_streams.rs:2
record_checker_judges_absence_by_the_persisted_set.rs:1
```

- 17 份不是 14 份：多的 `crash_segments_per_device_and_torn_in_place_overwrites.rs` 不在丁二名单里。
- 这 17 份里有三份测的是**旧引擎自己的功能**——断点续跑（`crash_enumeration_resumes_from_its_progress_file.rs`）、跨进程分片（`crash_enumeration_sharded_across_processes.rs`）、发现日志按签名去重并续跑（`crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs`）。流水线的对应物是判定库续跑、`host_shares` 分片、判红清单；这三份是「迁」还是「删」没人定，迁不了的话旧引擎不能删。
- 旧引擎还被这些地方引：`copy/crates/singlefs-checker-tier/src/layer0_progress.rs`（进度文件、分片账本、发现日志，文件头 `:12` 指到 `enumerate_layer0_in_state_slices`）、`copy/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:3023`（实验装置，`scripts/experiments/replay.sh:309` 登记了 E161 的复跑）、`scripts/gate/admission.py`、`scripts/crash/crash-case-check.py`、`.claude/gate.d/checker-tier-layer0-replay.sh`、`.claude/gate.d/stage-inputs.tsv` [读]。
- 结论：丁二之后至少 `enumerate_layer0_in_state_slices`、`Layer0*` 计数类型与 `layer0_progress` 要留到 E161 的装置收掉（或把装置里那一处改成走流水线并重登记它的产物）、三份基础设施用例有了去处为止 [推]。

## 11. Q8 两档：轻型 ⊂ 重型还成不成立，轻型的库放哪

- **成立，且不用改合表**。`.claude/rules/verification.md:50` 写的是「轻型是重型的子集：轻型里有的每一项，重型里都有」，入口自证 `light-within-heavy` 是拿真合表与小仓两份计划逐件核（`scripts/crash/crash-point-replay.sh` 文件头「轻型是重型的子集」一段）。比的是（项，测试目标，展开域）这一层，拆成两段后每一项仍是同一个入口、同一组测试目标，只是每个入口内部先跑数据后比；轻型在小域（「甲二」∪ 展开 6）、重型在整域，包含关系不变 [读、推]。观察者也一样：展开 6 域里 140 态（那一趟日志），30 深全展开 1.359×10⁹ 态，前者在后者里。
- **会破的是构建，不是包含关系**。快档命令是 `cargo test --release -p singlefs-checker-tier --lib --tests`（`.claude/rules/verification.md:34`），不带 `verdict-store`；RocksDB 是可选依赖、平常不编，因为「librocksdb-sys 在每个编译目录里现编一遍 C++」（`copy/crates/singlefs-checker-tier/Cargo.toml:18`、`:20`、`:26`–`:27`）。拆成先跑完全部数据、再从库读以后，每一个要判的测试（17 份调旧引擎的、13 处调 `inject_crashes_into_history`）都得读库，**快档整条要带这个特性**；`copy/crates/singlefs-checker-tier/src/crash_given_states.rs:119`：「`verdict-store` 特性的二进制只有它」，指不经库的就地判那一支（前半句在 `:118`）。连带：crates 变异整表每个拷贝目录都要现编 C++。两条路，交主 agent 或用户定 [推]：
  - 路 A：快档整条带特性，成本是每个编译目录多编一遍 C++（我没量，只知道 Cargo.toml 注释的说法，以及 `records/2026-09-30-里程碑三开工第一轮.md:986` 记的「编译两次撞 235 秒限时」）。
  - 路 B：对比段读一个 trait（库），实现两个——RocksDB 和内存 `BTreeMap`；内存那个只给不链 RocksDB 的小用例，**同样严格先写全部、后读**，对比段闭包里不引 RocksDB。代价：用户原话说的是「放入rockdb中」（`records/2026-09-30-里程碑三开工第一轮.md:997`），内存实现要用户点头。
- **轻型的库放哪、删不删**：沿用。轻型每趟新建空库，放 `<放量的家目录>/crash-point-replay-quick/<时戳>/library`，跑完删（`scripts/crash/crash-point-replay.sh:532`、`:29`），入口代码里 `delete_after` 一律删（`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:8213`）。轻型域 < 10⁴ 态，两段的库 < 数 MB（第 7.7 节），两段在同一趟同一进程里，不需要跨进程续跑。**建议改的一处**：判红时不删，留着那一块的 `10`–`14` 行，方便看红态现场 [推]。
- **轻型要加的守门**（`BOTH_TIERS_GROUPS` 那几类，`scripts/crash/crash-point-replay.sh` 文件头）：
  1. 每个新内核（P5 断言、P7、P10、P11、P14、P15、P16）一条 CPU 对 GPU 的逐态对拍，进快档，并各有「往加速那一侧塞一个已知故障，对拍要红」的自证（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:82`；既有先例是 `FaultPutOnTheGpuSide`，`copy/crates/singlefs-checker-tier/src/crash_amplification.rs:8266`）。
  2. 剪枝自证分两层：录入行复用、结论行复用，各自「在小规模输入上关掉剪枝与复用跑一遍，判定与开着时逐项相同」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:63`）。
  3. 红态正文现场重跑补细节的那条路，要有一条钉回来的红例（`.claude/singlefs-ai-sop/rules/test-script-discipline.md@全量的红，钉回快档`）。
- **重型三种跑法在两层上的含义**（`.claude/rules/verification.md:47`）：剪枝 = 录入行键同则不重录、结论行（键 + 版本）同则不重比；逐条 `--item` = 点名项同上；`--full` = 清空复用，两层都清，等于重跑 Rust 恢复，6.3 节表里「读路径改了」那一行的 3.7–5.4 天。**弹窗写明「全量 = 重录 + 重比」**，用户若想要一个只重比不重录的档（判器改了的日常），是一个新档，交用户定；它不能证明录入没错，所以不能替代 `--full` [推]。
- 重型红了要修好或查清原因（`.claude/rules/verification.md:52`）这一条不变。

## 12. Q9 红步之后照判：含义与单列写法

### 12.1 事实 [读]

- 今天历史跑到「第一次失败（违例、执行器判出的、模型对不上、panic）就停」（`crates/singlefs-harness/src/history.rs:5745`）；崩溃点只在跑出来的那一截流上抽（`copy/crates/singlefs-checker-tier/src/crash_injection.rs:1471` 的 `draw_crash_points(history.seed, &segments, …)`）。
- 模型「对得上就按模型自己算的结果往前走（不按实现交回的）」（`crates/singlefs-harness/src/model.rs:2870`）；对不上返回错、不推进；拒绝分支只按实现报的完成发布数推进到那一步（`:2956`、`:2957`）。拆开后跑数据段不比较，所以要无条件推进：成功按模型的答推（`advance_by_success`，`:3393`），拒绝按观察到的发布数推。
- 因此红步 k* 之后：**模型沿它自己的时间线走，实现已经离开它**。后面每一步比的是「实现真实的结局」对「模型按自己走到这里预言的答」。
- 模型各版（`committed_versions`，`copy/crates/singlefs-checker-tier/src/crash_injection.rs:1410`）也是这条时间线的产物；RefFS 翻译吃的是**实现观察到的**每步结局（`crates/singlefs-harness/src/reference_filesystem_translation.rs:99` 的 `note_an_observed_step`），不吃模型；池级 checker 与记录核对器判的是镜像本身。

### 12.2 每类判定在 k* 之后的含义

| 判定 | k* 之后还说明什么 | 不能说明什么 |
|---|---|---|
| 每步模型比（P8） | 实现这一步的结局与「模型假设前面都对」时的预言是否一致；一致说明实现自己拐回来了 | 不是独立的新缺陷：前面的分叉会级联（例：实现多拒了一次发布，之后每个根的 txg 都错位），后面的红多半是同一个根因 |
| 崩溃点上与理想模型比（B1/B9，P10） | 读回的根是否在**模型这条时间线**的 committed 里 | 模型从没提交过实现真写下的根（报「没提交过这一版」），或模型提交过实现根本没写的根（该红不红）；两个方向都会错 |
| RefFS 一票（P11） | 读回在不在**实现实际做过的**操作序列允许的集合里 | —— 不受模型分叉影响，k* 之后照常有效 |
| 池级 checker、记录核对器（P12–P13） | 镜像结构是否合法 | —— 不受影响 |
| 联合判（`JointVerdictOfTheReferenceFileSystemAndTheModel`，`crates/singlefs-harness/src/reference_filesystem_translation.rs:791`；文档注释在 `:789`：「任一红即红」） | RefFS 一侧的红 | 分类名「只 O1 红」（`:822`）、「两者都红」（`:823`）这类需要模型一侧当独立证据的，在 k* 之后不成立 |

### 12.3 单列怎么写才不误导 [推]

1. 对比段判出 k*（第一个模型比判红的步）后，历史得一条记录：`first_red_step`、「红在 k*，其后 N 步仅作参考」。历史的红绿由 k* 及之前定，之后的步**不改红绿**。
2. k* 之后的每步模型比、每个崩溃点上的 P10 结论，各带一个 `after_first_red: true` 字段，**单列计数**（比了多少、对上多少、对不上多少），报告里放在一个独立标题下，标题写「分叉之后：模型沿自己的时间线、实现已离开，只供参考」，**不与红步之前的数相加，不进「判红 / 新发现」总数**。
3. 发现签名（`FailureSignature`，`crates/singlefs-harness/src/history.rs:2241`）里加这一位，k* 之后由模型一侧单独判红的，**不登记成新发现**，只登记成「级联」；RefFS、池级 checker、记录核对器在 k* 之后判出的红仍照常登记新发现（它们不依赖模型）。
4. 联合判在 k* 之后只报 RefFS 一侧，模型一侧写「分叉后不判联合」。
5. 副作用要写进报告：历史不再停在 k*，流更长，崩溃点在更长的流上抽，**同一个种子今天抽到的崩溃点与拆开后抽到的不同**（抽取以段序列为入参）；今天因早停而只覆盖到 k* 之前的种子，拆开后覆盖变多，但两趟的计数不可直接对比，已知红清单里依赖「停在第几步」的签名要重核 [推]。
6. 跑到「实现自己停」的停法是 panic、报错、走完三种：panic 本身是不依赖模型的缺陷，照常登记。

## 13. 要主 agent 量或跑的东西（我没编译、没跑，不是子 agent 前台跑得完的）

| 编号 | 量什么 | 命令与环境 | 看哪几行 |
|---|---|---|---|
| M1 | 每态 `recover` 两遍的 CPU 代价（Q5） | 先让实现员在副本 `crash.rs:1374`、`:1375` 的两个 `recover` 前后加计时累加，块末打 `RUN_COST` 行。然后在副本根：`SINGLEFS_HEAVY_TESTS=user-request bash scripts/hosts/run-with-memory-cap.sh 16G bash scripts/hosts/capped.sh <1\|8\|32> bash scripts/crash/crash-amplification.sh --item multi-record-publish --expand-up-to 16`；再各换 `--item fixed-script-through-unmount`、`--item position-addressed-trees`。`ENABLE_GPU` 以 `multi-host.env` 为准，现在是 0（`/home/fy5090/code/singlefs/multi-host.env`）。重型，只由主 agent 带前缀起 | `RUN_COST` 行；`CRASH_AMPLIFICATION_PHASES` 的 `seconds_judging` 与 `states_checked_here`；三次的相对极差 |
| M2 | 观察者每态挂载成本、挂载写表分组数（Q5、Q6） | 同 M1 命令（`--item multi-record-publish`），多记录分项的测试名见 `scripts/crash/crash-amplification.sh:233`；另在观察者里对每态挂载写表取 SHA-256 数不同值（实现员活） | `CRASH_POINT_REPLAY_OBSERVERS` 的 `states_observed_here`、`CRASH_AMPLIFICATION_PHASES` 的 `seconds_observing`、不同挂载写表个数 |
| M3 | 新表压缩比与 RocksDB 写放大（Q4） | 新布局落地后同 M1 命令，新建空库；跑完 `du -sb <库目录>`；跑的同时读 `/proc/<pid>/io` 的 `write_bytes`（或开 RocksDB 统计读 `rocksdb.stats`） | 库字节 ÷ 态数；`write_bytes` ÷ 库字节 |
| M4 | P14 与「录入比对」内核的展开量（Q6） | 内核写出后在副本里写一条单测调 `kernel_inlined_size(&source)`（`copy/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:263`），`bash scripts/hosts/capped.sh 8 cargo test --release -p singlefs-checker-tier --features gpu --lib <测试名>`（经 `run-with-memory-cap.sh`；带 `gpu` 的经 `gpu-quiesce-run.sh`） | `expressions` 与 `largest_contributors` |
| M5 | 盘余量（Q4） | `df -h ~/.local/share/singlefs`，开跑前现查 | `Avail` |
| M6 | 重建我的副本 | `rsync -a --exclude target --exclude 'target-*' /home/fy5090/code/singlefs/ <新目录>/`，再依次 `python3 scripts/gate/apply-writer-patch.py ~/.local/share/singlefs/work/m3-merge-impl/<v e e2 e3 e4 e5>/patch --root <新目录>`；我这一趟拷贝加六份补丁共约 7 分半（`rsync` 慢在盘上，不在补丁） | 退出码 1、0、0、0、0、0 |

## 14. 各结论的推翻条件

1. 「3.1 缺三格」：附录或正文别处已经把建判表、账与报告、判器自证用例归了边 → 撤。
2. 「G1–G7 漏在 P 号之外」：事实表或正文里有一行把 `allocation_generation_judgement`、B0 断言、`pool_after == crash_image` 点了名 → 撤对应一条。
3. 「D14 的『盘变没变』与分界冲突」：用户把它认作「CPU 实测的跑数据」→ 撤。
4. 「RefFS 拆得开」：找到 `allowed_set_for_a_crash_in` 的入参含读回，或重起头之后序列非空（第二次允许集合不止 {R1}）→ 撤。便宜的验证：副本里写一条单测，对随机历史比新旧两种判法逐崩溃点相同（同时是改法的验收）。
5. 「录入版本单靠起一族做不到」：`crash.rs` 里的枚举、录入、对比其实已经分在三个文件 → 撤；或闭包算法改成按函数收 → 撤。
6. 「对比结论版本没绑录入」：在 `conclusions_missing_for_block`（`copy/crates/singlefs-checker-tier/src/verdict_store.rs:1929`）之外另有一道检查录入版本 → 撤。
7. 「30 深下界 3 743 376 012 态」：别的五项在 30 深下态数更少（不可能，深度只增不减）或登记表里 `FULL_EXPAND_UP_TO` 已不是 30 → 撤；若别的项更大，则是更大的下界。
8. 「CPU 上界 3.7–5.4 天」：旧速率 8 千态/秒本就是旧数（9/30 的版本）；M1 量出的 `recover` 两遍占比乘它得新数，两者不符以 M1 为准。
9. 「改名早于第一趟整域录入」：录入段闭包里不含被改名的标识符 → 顺序不再重要；我只在 core、harness、checker 的 `src/` 里 grep 了非注释命中。
10. 「丁二后旧引擎留」：E161 装置收掉、三份基础设施用例有去处 → 撤。
11. 「P14 展开量 7–25 万」：M4 量出 > 27 万或 < 5 万 → 改。
12. 「随机历史开 GPU 时约 60 步封顶」：每步写数比我估的少（读 M2 同样的方法数一条历史的枚举写表长度）→ 改。
13. 「快档要带 `verdict-store`」：对比段改读 trait 且内存实现被用户认 → 撤（路 B）。

## 15. 判定一览

判定三种：一致 / 冲突 / 规则没说。「一致」带条件的写在一句话里。

| 格 | 判定 | 一句话 |
|---|---|---|
| 3.1 左栏（真跑 core、模型、RefFS；环境、指纹、版本） | 一致 | 都是产出或前置检查，不判生成物对错（第 2 节） |
| 3.1 右栏（oracle、checker、记录核对器、与模型比、与 RefFS 比、判据复算、第 ②③ 段、J7、归类、J6） | 一致 | 「归类」要拆成签名位（可上 GPU）与文字、落文件（只能 CPU） |
| 3.1 缺格：对比段内 CPU 预处理（建判表、抽事实表、解枚举表） | 规则没说 | 应写成对比段内的 CPU 预处理 |
| 3.1 缺格：账与报告 | 规则没说 | 对账、复用计数、覆盖报告、红态正文，留 CPU、排在对比之后 |
| 3.1 缺格：判器自证与手摆镜像用例 | 规则没说 | 它们就是 CPU 参照，留 CPU（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:81`） |
| 3.2 `0F` 流程输入 | 规则没说 | 缺观察者另读的流程输入（`versions_with_a_file`、`deletion_publishes`） |
| 3.2 `10` 状态跑数据 | 一致 | 每态 160 字节偏大，最小字段账 106 字节；红态正文的 Debug 载荷装不下，现场重跑补 |
| 3.2 `11` 挂载跑数据 | 一致 | 观察者的挂载要加写录制层；「盘变没变」不该在这里（见 P14） |
| 3.2 `12` 模型与 RefFS | 一致 | 允许集合是集合；另要录历史内每次恢复的 (hi，允许集合，读出) |
| 3.2 `13` 写内容 | 一致 | 一条流 ≲ 2–3 MiB，不是体量问题 |
| 3.2 `14` 每步跑数据 | 规则没说 | 漏了模型派生常量（容量墙、有效回退下界、几何）与「只推进不比较」口子 |
| 3.2「录入版本另起一族」 | **冲突** | 与 `judging_closure.rs:681` 的停止表（录入代码若落在流水线文件里不进版本）和 `crash.rs` 的混放冲突；对比结论版本也没绑录入版本（`crash_identity.rs:619`）；要拆三族、拆文件 |
| 3.2 布局号 2 → 3 | 一致 | 旧布局库被拒、换新目录，新布局整域还一趟没跑过，零代价（`verdict_store.rs:514`、`:523`） |
| P1 层 0 旧引擎 | 一致 | 拆得开，接恢复内核加 Rust 恢复表；迁移量见 Q7 |
| P2 流水线 CPU 判 | 一致 | 机械拆 |
| P3 开 GPU 的流水线 | 一致 | 比一致就是 J6 `CrossCheckWords` 加读回摘要；要从可跳过的 J6 里拆出来恒跑 |
| P4 单元校验和对拍 | 一致 | GPU 对 CPU 的自证，留 CPU |
| P5 观察者 | 规则没说 | 缺录制层；判表上限够用；CPU 代价（每态约 66 ms 的量级）才是卡点 |
| P6 第 ② 段读回 | 一致 | `read_flow_facts` 已存在、没人调，改成读回 |
| P7 第 ③ 段与 J7 | 一致 | 已读库；GPU 上没有吞吐收益，照用户口径做 |
| P8 每步模型比 | 规则没说 | 漏执行器另外三类判断、模型派生常量、推进口子；做得出，无吞吐收益 |
| P9 给定状态表 | 一致 | 现成表与内核；就地判路径要删或也走库；随机历史开 GPU 约 60 步封顶 |
| P10 理想模型 | 一致 | 三参数纯函数，新小内核 |
| P11 RefFS | 一致 | **拆得开**；第二次崩溃集合 = {R1}；历史内恢复一票要挪 |
| P12–P13 第二截 | 一致 | 并进第三截表，记录核对器要两份掩码 |
| P14 拒绝合法性与 W | **冲突**（一格） | 事实表 D14 把「盘变没变」放跑数据，与 1003 行分界冲突；判据复算做得出但最重（7–25 万展开，未量） |
| P15 挂载透明 | 一致 | 新内核逐字段按字节区间比；变长条目要定长化 |
| P16 I-8.4 | 一致 | 摘要相等，并进 P15 内核 |
| P17 判红后归类 | 一致 | 只对红态签名位成立，文字与落文件留 CPU |
| P18 J6 | 一致 | 给定状态那一路要从「当场比」改成「读库比」 |
| 3.3 用户四件定案对 P3、P7、P17、P8、P14 的改动 | 一致 | 逐条见上；P8「红步之后照判」的含义见 Q9 |
| 3.4 次序：改名在两段改造之前 | 一致 | 理由换成：枚举叶子文件里有 `Layer0*` 名字，改名晚于第一趟整域录入会让录入孤儿；顺带免掉补丁重放改名 |
| 3.4 簇的交集与依赖 | **冲突** | 丙依赖丁一、戊的 CPU 参照（事实表把丙写成「不交」）；戊文件表漏 `model.rs` 与三份集成测试；丁二漏 `crash_segments_per_device…`；缺「簇 0」按文件拆 `crash.rs`/`crash_injection.rs`；己缺替代检查；门禁脚本侧没人排 |
| 3.4 丁二「14 份」 | **冲突** | 调旧引擎的测试文件 17 份，其中 3 份测旧引擎自己的功能 |
| Q1 全不全 | 冲突 + 规则没说 | 七类漏在 P 号外（第 4.2 节）；射程判据已给；事实表 117 处总数我复核不了 |
| Q2 只读库重建输入 | 一致（带四处缺口） | 缺观察者另读输入、观察者挂载写录制层、模型推进口子、红态正文 Debug 载荷 |
| Q3 键与版本 | 冲突 | 见 3.2 录入版本那一行；结论版本要绑录入版本；录入拆三族；按文件拆 |
| Q4 体量 | 冲突（数） | 今天全量深度是 30，30 深去重下界 3.74×10⁹ 态、160 字节 = 598.9 GB；盘余量 514G 与 1.8T 都读到过；背景的「判定库 4.4G」是整个目录，库本身 284 MB 且是旧布局 |
| Q5 CPU 代价 | 规则没说 | 墙钟由 CPU 定；旧速率给出上界 3.7–5.4 天；`recover` 单独的代价没量，量法见 8.3 |
| Q6 GPU 一侧 | 一致（带两个上限） | P14 展开量未知；随机历史开 GPU 约 60 步封顶；其余都在预算内 |
| Q7 次序与簇 | 冲突 | 见 3.4 两行 |
| Q8 两档 | 一致 + 规则没说 | 包含关系不变；快档要带 `verdict-store` 或给库一个 trait，交主 agent 或用户定 |
| Q9 红步之后 | 规则没说 | 单列写法见 12.3；副作用是同一种子抽到的崩溃点会变 |

## 16. 没做什么

- 没编译、没跑任何测试（重型、轻型都没跑）；没提交；没改主工作区（副本只用来读，交回前已删）。
- 所有「没量过」的数仍然没量过：`recover` 两遍的单态 CPU 代价、观察者每态挂载成本与挂载写分组数、新表压缩比、RocksDB 写放大、P14 与新内核的展开量、一次发布的写数。能算的只给了算式。
- 事实表的 117 处总数没复核（模式表有聚合名，复刻不出）。`singlefs-harness/tests/` 与 `src/bin/` 没逐处列。
- 第二台机器的线程数没登录核：用的是旧记录「两台各 31 线程」。
- 改名清单（`~/.local/share/singlefs/work/m3-merge-impl/rename-sweep/report.md`）只读了第 83–130 行与几处 grep；没确认 `tests/crash_enumeration_*` 与 `common_crash_points/mod.rs` 的路径不在改名清单里。
- 没替主 agent 采纳或出判决；不判别的腿的格没判；没读禁读清单里的任何文件。
- 没碰 kb、records、`.claude/`。要写进 kb 的东西（如 `verification.md` 快档命令要加特性）只在 Q8、Q7 里点了名。
- 草稿与临时文件在 `~/.local/share/singlefs/work/m3-run-compare-split-r1-forward/`（`g/` 是 grep 落盘，`tools/ln.sh` 是我查行号的小脚本，`notes.md`，六份 `apply-*.log`）；副本 `copy/`（2.4G）已删，原因是只读副本用完即删（`.claude/agent-common.md` 与交回闸的要求）。
- cite-check：副本还在时 `python3 scripts/agents/cite-check.py <本报告> --root ~/.local/share/singlefs/work/m3-run-compare-split-r1-forward --root /home/fy5090/code/singlefs` 末行「✓ 核了 15 处引文，对上 15 处，没判 0 处」；副本删后按交回闸的默认根再跑，末行「✓ 核了 15 处引文，对上 11 处，没判 4 处」（4 处是 `copy/…` 路径，副本已删所以认不出，不是对不上）。报告里大多数引用不是「引号加位置」的写法，cite-check 看不到；这些行号我都用 `grep -nF` 现取过（小脚本 `tools/ln.sh`），并另用一段脚本核过 164 处 `路径:行号` 都没越过文件末尾。
