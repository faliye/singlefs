# 第 18 批代码轮第一轮：云端正推腿报告（格 K1、K3）

日期 2026-10-04。代码一律读冻结副本 `/tmp/claude-1000/m3-formal-code-b18-r1/tree/`（引文行号是那棵树里的行号）；kb 条款写 kb 文件自己的行号。
没有读别的腿的报告与实现员报告。checker 档没跑，凡是只能在 checker 档上判的，一律标「推的」。

## 一、开工前的观测

- 冻结副本里被判的三份源文件与主仓今天的字节相同：`sha256sum` 对 `crates/singlefs-harness/src/model.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs` 在仓里与副本里各自逐个相等（model.rs 8ba9b345…、mount.rs 717e108a…、crash_injection.rs 46bfe504…）。
- 我的副本：`/tmp/claude-1000/m3-formal-code-b18-r1-sonnet/ws/`（冻结树的 `crates/` 加仓根的 `Cargo.toml`、`Cargo.lock`），编译目录 `/tmp/claude-1000/m3-formal-code-b18-r1-sonnet/target/`（交回时的处置见末尾「没做什么」）。
- 开跑前 `ps` 看到别的会话在跑一条 `cargo test --offline --no-fail-fast -p singlefs-harness …`（无性能测量在跑），我的命令加 `nice -n 19` 并经 `capped.sh 6` 照常跑；没有等锁（我用自己的 `CARGO_TARGET_DIR`，不抢仓内 target 的锁）。内存包装在同会话已有别的腿在跑时拒绝再起（要设理由才并行），我每次都设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON=同会话里别的腿各跑各的，我只跑小测试`，没有绕开包装；包装自己的结局我遇到过：251（用法错：上限超过峰值表 2 倍、或写 auto 而表里没有这条，命令一行没跑）若干次，按它的提示改上限后重跑；250 一次（撞自己的上限：上限 38M 把「改了 core 之后的重新编译」也罩住了），那一次输出不算结果，我把编译挪到包装之外（`--no-run`，不跑编出来的代码）再经包装跑。这些被拒或作废的调用都没有产出任何被引的数。

- kb 在我开工之后被别的会话改过两份：`.claude/kb/decisions/16-发布语义.md` 与 `.claude/kb/checks-owed.md` 的 `sha256sum` 与快照 `research/prompts/m3-formal-code-b18-r1-snapshot/kb-sha256.txt` 记的不同（D23 一致）。我引的 C670 行（529）、D23 的 392、369 行与 D16 的 42 行，按 `git diff`（工作区对暂存区）看没动（暂存区对快照时刻我没比），D16 里「树表 0 条那一版上正常卸载」那一条从 76 行挪到了 77 行；下面引的是今天文件的行号。

## 二、K1　P1 第三截接判官

### K1-a　判官的五样输入，逐样对 C670 题面与 D23 已定项 14

C670 题面（该行很长，只引与这一格相关的三句，其余没引）：

- 「没有任何读故障的崩溃态，不许让 D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真」（`.claude/kb/checks-owed.md:529`）
- 「崩溃注入第二截在没注入读故障的崩溃镜像上复算见证判据，为真即判拒得不合法」（`.claude/kb/checks-owed.md:529`）
- 「第二截要知道这个崩溃态注没注入读故障」（`.claude/kb/checks-owed.md:529`）

D23 已定项 14 的判据与拒法（同样是一行长条，引三句）：

- 「判不出 ⇒ 按真」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）
- 「所选那一版那次发布带末条标志的记录读得出（任一份）⇒ c_见证 > 它的计数器即为真」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）
- 「仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）

判官 `judge_writable_mount_refusal_of_a_crash_state` 今天的判法（它是第 16 批就有的，第 18 批只是第三截多了一个调用点）：

- 「witnessed_counter_holds_on_a_crash_image_without_read_faults(」（`crates/singlefs-harness/src/mount_transparency.rs:1561`）只在「拒因是 N-配置、读故障认定为 NoneInjected」那一支判（`crates/singlefs-harness/src/mount_transparency.rs:1556-1559`）；注入过读故障、拒因是实例表读不出的，都落到空的那一支（`crates/singlefs-harness/src/mount_transparency.rs:1572-1579`）。
- 三支判据照 D23 写：c_见证 为 0 判假（`crates/singlefs-harness/src/mount_transparency.rs:1376`）；末条读得出比计数器（`crates/singlefs-harness/src/mount_transparency.rs:1389-1393`）；读不出按 c_见证 处那条记录比 (实例, txg)；两条都读不出按真（`crates/singlefs-harness/src/mount_transparency.rs:1403-1405`）。

第三截的五样输入怎么取、对不对得上（每一行的代码行都在冻结树里现 grep 过）：

| 输入 | 第三截代码 | 条款要的 | 判定 |
|---|---|---|---|
| 二次崩溃镜像 | 「let second_crash_image_materialized = materialized_crash_image(&image);」（`crates/singlefs-checker-tier/src/crash_injection.rs:2726`）；同一份交给再挂（`crates/singlefs-checker-tier/src/crash_injection.rs:2812-2815`）、判官第 1 参数（`crates/singlefs-checker-tier/src/crash_injection.rs:2943`）、盘变没变的对照（`crates/singlefs-checker-tier/src/crash_injection.rs:2937`） | 判据在「这个崩溃镜像」上复算；拒之后的池与它逐字节相同 | 一致。再挂、复算、比盘用的是同一个池值，没有第二份 |
| 拒之后的池 | `writable_mount_alone_on` 在拒的错误出来之后才把 `devices` 收成池（`crates/singlefs-checker-tier/src/crash_injection.rs:1611-1617`），`Refused` 带它（`crates/singlefs-checker-tier/src/crash_injection.rs:1619-1624`） | 「盘上逐字节不变」要比拒之后的盘 | 一致。盘在挂载那一趟之后才收，错误值先取走再收盘，没有先后错位 |
| 所选那一版 | 二次崩溃镜像上只读恢复的 `report.effective_root`（`crates/singlefs-checker-tier/src/crash_injection.rs:2692-2694`）交给「version_landed_on_after_the_crash: report.effective_root,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2733`），第二截同一写法（`crates/singlefs-checker-tier/src/crash_injection.rs:1322`） | 「所选那一版」 | 一致。落在 `None`（恢复没择到根）时判官在 `let selected_version = selected_version?;` 那一行（`crates/singlefs-harness/src/mount_transparency.rs:1379`）提前交回 None、不判，C670 在那种二次崩溃镜像上静默不判，第二截同样 |
| 只读读出在不在允许集合 | 第三截对这个二次崩溃问模型的结论 `disagreement.is_none()`（`crates/singlefs-checker-tier/src/crash_injection.rs:2734`），结论本身在 2706-2710 行算 | U6 第三条（这一条本身不是 C670） | 一致，同第二截口径 |
| 读故障认定 | 沿用第一次崩溃那个态：「read_faults: read_faults_in_the_first_crash_state,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2739`），那个值在「let read_faults_in_the_crash_state = marks.read_faults_in_a_crash_state(」（`crates/singlefs-checker-tier/src/crash_injection.rs:1279`）取、「read_faults_in_the_first_crash_state: read_faults_in_the_crash_state,」（`crates/singlefs-checker-tier/src/crash_injection.rs:1382`）交进第三截 | 「第二截要知道这个崩溃态注没注入读故障」；D13 射程只管不带故障的态 | 一致，理由见下 |

读故障认定沿用第一次崩溃态为什么对（我自己推的两步，各有代码依据）：

1. 第二截与第三截之前那次可写挂载在 `SparseBlockDevice` 外包 `RecordingBlockDevice` 的盘上起（`crates/singlefs-checker-tier/src/crash_injection.rs:1710-1725`），那一段没有任何故障注入的包装，所以挂载自己写出的字节不引入读故障。
2. 注入读故障的只有历史里那一类步：「HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => true,」（`crates/singlefs-checker-tier/src/crash_injection.rs:1033`），认定函数只看崩溃窗口末尾之前的步（`crates/singlefs-checker-tier/src/crash_injection.rs:1020`）。二次崩溃镜像 = 第一次崩溃镜像加上那次挂载写的一个子集，所以它「注没注入过」就是第一次崩溃态的。
反方向的代价：第一次崩溃态认定为 Injected 时，第三截整个不判 C670（判官在 Injected 那一支不加 C670 那一条），与 D13 的射程一致，是保守的。

K1-a 判定：**一致**（五样输入都对得上；一个已知的静默区：所选那一版为 None 时不判）。
会推翻它的现象：(1) 在某个第三截再挂被拒的态上，`pool_after` 与二次崩溃镜像在「拒之前盘上没写任何东西」时仍逐字节不等（说明收盘的先后有错位）；(2) 第二截那次挂载其实在注入读故障的包装上起（我读到的是没有，`crates/singlefs-checker-tier/src/crash_injection.rs:1710-1725`）。

### K1-b　与 I-8.4 那一判的先后

代码顺序（第三截 `judge_the_remount_after_a_second_crash`）：先计「再挂次数」，再看没崩过的那一次挂载有没有内容，再挂，被拒才进下面这一支——

- 「tally.remounts_after_a_second_crash += 1;」（`crates/singlefs-checker-tier/src/crash_injection.rs:2801`）
- 没崩过的那一次被拒（`once` 是 Err）时只记没判就返回，不再挂（`crates/singlefs-checker-tier/src/crash_injection.rs:2803`）。
- 再挂被拒那一支：先记 I-8.4 没判「.entry(RemountIdempotenceNotJudged::TheRemountWasRefused)」（`crates/singlefs-checker-tier/src/crash_injection.rs:2826`），之后才调「judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(」（`crates/singlefs-checker-tier/src/crash_injection.rs:2828`），再 `return`（2842 行）。
- 函数头注释自己写的也是这个次序：「再挂被拒时 I-8.4 照旧不判，另判 C670 那一条」（`crates/singlefs-checker-tier/src/crash_injection.rs:2783`）。
- 内联用例 `a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal`（`crates/singlefs-checker-tier/src/crash_injection.rs:5246`）同时钉了两样：I-8.4 没判计 1、C670 判过 1 不合法 1（改动全文 `research/prompts/_m3-formal-code-b18-r1-diff.md` 里那条用例的两处 `assert_eq!`）。

K1-b 判定：**一致**。I-8.4 先记没判、再判 C670，二者互不挤占；C670 红另走 `findings.settle`，记在 `RemountAfterASecondCrashInsideTheWritableMount(phase)` 那一截（`crates/singlefs-checker-tier/src/crash_injection.rs:2983`），与 I-8.4 的红同一截标签、靠「违例为空而带模型分歧」与 I-8.4 的「违例是 I-8.4」分开。
会推翻它的现象：一个再挂被拒的态上 `remount_idempotence_not_judged[TheRemountWasRefused]` 没记，而 C670 的计数记了（或反过来）。checker 档没跑，这一格是读代码推的。

### K1-c　第三截只判 C670 一条、不拿 U6 其余三条定红绿

代码：只在判官交回的没满足的条件里有 C670 那一条时才报红、才计不合法——「if !witness_criterion_true_without_read_faults {」（`crates/singlefs-checker-tier/src/crash_injection.rs:3028`）之后是 `return None`；函数头注释交代了这是有意的：「条款只管第二截，这一截不拿它们定红绿」（`crates/singlefs-checker-tier/src/crash_injection.rs:2920`）。盘是否变了在第三截算了（`crates/singlefs-checker-tier/src/crash_injection.rs:2937`），但只在 C670 红的时候随 `conditions_not_met` 打印出来。

并排两句：

- D23 已定项 14：「仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）
- 代码：「if !witness_criterion_true_without_read_faults {」（`crates/singlefs-checker-tier/src/crash_injection.rs:3028`）

差在哪：D23 这句管「拒了可写挂载」本身，不限第二截。第三截里一次拒了的再挂如果写过盘（`DiskAfterTheRefusal::Changed`）而 C670 那一条不成立（比如认定为 Injected），这一截既不计也不红。正文第二节已把「U6 不延到第三截」判不改、记欠，这一格我只记「范围窄于条款」，不替主 agent判。
K1-c 判定：**冲突（范围：D23 字面管所有拒，第三截只判 C670 一条），已知并记欠**。会推翻它的现象：第三截有一条别的路径也会对「再挂被拒而盘变了」报红（我没找到：第三截里读到 `Changed` 的只有 3028 行之前那一处）。

### K1-d　没崩过的那一次挂载被拒时第三截不判，漏的是哪一类态

代码：`content_after_the_uninterrupted_mount` 在没崩过的那一次挂载被拒时是 Err：「None => Err(RemountIdempotenceNotJudged::TheUninterruptedMountWasRefused),」（`crates/singlefs-checker-tier/src/crash_injection.rs:1345`）；第三截看到 Err 就在 2803 行返回，不起再挂，两样（I-8.4、C670）都不判。

哪些态被这一条挡掉要分两层看：

1. 没崩过的那一次被拒、而拒之前一个字节都没写：挂载那一段录制流是空的，「写行之前（取号之前的判、取号自己）就交回」那一类（`crates/singlefs-checker-tier/src/crash_injection.rs:1874`）。挂载写表空 ⇒ 没有段 ⇒ `draw_second_crash_points` 里每个阶段都是「if writes_of_the_phase.is_empty() {」（`crates/singlefs-checker-tier/src/crash_injection.rs:2559`）而跳过 ⇒ 不摆任何二次崩溃态。这一层是空集，不算漏。
2. 没崩过的那一次被拒、而拒之前写过东西：「被拒时是拒之前写出的」（`crates/singlefs-checker-tier/src/crash_injection.rs:1667`）；代码里点名带着已落盘写账的错只有两个——「MountError::Publish(failed) => {」（`crates/singlefs-checker-tier/src/crash_injection.rs:1877`）与「MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {」（`crates/singlefs-checker-tier/src/crash_injection.rs:1883`）；还有取号自己失败与 `RaiseFloorSequencePublishFailed` 也可能带写，代码没分（推的）。这些态有二次崩溃态（挂载写表非空），但第三截一个都不判。**漏的就是这一类**：「第一次崩溃之后可写挂载走到取号或写行之后才被拒」的态上的二次崩溃镜像。
正文第二节已记的「要多挂一次」是同一件事的另一面：C670 的复算并不需要 `once`（判官的输入只有二次崩溃镜像、所选版本、拒错、盘变没变、读出在不在集合、读故障），门是为 I-8.4 的「做一次」设的，C670 被它连带挡掉。

这一类今天有多少没量到：`CrashInjectionTally` 里被拒的挂载只分 `writable_mounts_after_the_crash_refused_at_the_unit_area_wall` 与 `writable_mounts_after_the_crash_refused` 两格（`crates/singlefs-checker-tier/src/crash_injection.rs:2162-2167` 一带），不分拒前写没写。要量，得在 tally 里加一格「没崩过的那次挂载拒前写表非空」再跑 checker 档（我没跑）。
K1-d 判定：**一致（代码做的就是注释与正文说的）；漏的类 = 第 2 层**，条款没写这一类要不要判（C670 题面只说第二截），按「规则没说」记。
会推翻它的现象：在 checker 档上，第 2 层的态数是 0（即没有任何一个第一次崩溃态上的可写挂载在写过东西之后被拒）——那就只剩空集。

### K1-e　V242u 在 checker 档用例上推得出红吗（抽样会不会抽不中）

V242u 是什么：只在带单元的发布上把「系统配置轮换」挪到「根槽 FUA」之前（改动全文里那一行：「只在带单元的发布上先轮换系统配置、后写根槽」（`research/prompts/_m3-formal-code-b18-r1-diff.md:665`)）；被改的是 `persist_publish_writes` 末尾那一段（「persist_the_root_then_rotate_the_system_configuration(」（`crates/singlefs-core/src/transaction.rs:1151`））。写行那次发布的单元非空：「.chain(allocation_built.rewritten_units.iter().cloned())」（`crates/singlefs-core/src/transaction.rs:1964`）；零单元发布「units: Vec::new(),」（`crates/singlefs-core/src/transaction.rs:1406`）不受 V242u 影响。

红要同时满足的条件（我从代码逐条推的）：

1. 红态在第三截：二次崩溃落在写行那一段的「轮换落了、根槽没落」。第一次崩溃镜像上看不见它，因为同一实例的记录会被重放、恢复落到新那一版；二次崩溃里写行那条记录属于新实例，前缀不跨实例：「前缀规则不跨实例边界：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停」（`.claude/kb/decisions/23-journal的角色与格式.md:369`）。所以所选那一版是旧的，而轮换写进去的 tail 是新记录的计数器，大于旧那一版末条的计数器，判据第一支为真（「所选那一版那次发布带末条标志的记录读得出（任一份）⇒ c_见证 > 它的计数器即为真」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）；判官同样的比法 `crates/singlefs-harness/src/mount_transparency.rs:1389-1393`）。实现的再挂同判据为真、重读仍真，拒，判官判不合法，计 1。
2. 红态要被抽中。写表里根槽 FUA 只发给一块盘：「WriteDurability::ForceUnitAccess,」（`crates/singlefs-core/src/transaction.rs:358`，目标盘由 `region_devices` 定，345-351 行）；FUA 只放行它那块盘：「FUA 写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写一起被放行，别的盘上的写不因它持久。」（`crates/singlefs-harness/src/segments.rs:103`），所以 V242u 下「轮换 dev0、轮换 dev1、根槽」三个写在同一段。

草稿探针（`research/prompts/m3-formal-code-b18-r1-sonnet-model/scratch_layout.rs`，只量切段与挂载写布局，不判任何东西，harness 档，经内存包装）在全持久的池上起一次可写挂载，量到写行那一段的写数 N 与 V242u 下的切段（原样输出在同目录 `layout-base.log`、`layout-v242u.log`）：

    历史写数 83，段数 18，候选段数 13            （V242u；基线是 段数 20，候选段数 15）
    候选段按长度的个数 {1: 2, 2: 7, 3: 2, 20: 2}
    挂载写数 55，段数 10
      挂载段 3 长 3：["system_configuration_slot@dev0", "system_configuration_slot@dev1", "root_record_fua@dev0"]
    写行那一段（含它的轮换）的写数 N = 19

3. 二次崩溃的抽法：每一阶段在这一阶段的 N 个写里均匀抽一个 w，w 之前的写都持久、w 不持久、w 之后的各按硬币：「std::cmp::Ordering::Greater => source.below(2) == 1,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2572`）。红态要「根槽没落且至少一个轮换落了」：w 是轮换 dev0 时要轮换 dev1 落、根槽没落（1/2 × 1/2）；w 是轮换 dev1 时只要根槽没落（1/2）；w 是根槽时必红（1）。每一个独立的抽样流命中概率 p = (1/4 + 1/2 + 1) / N = 7/76 ≈ 0.092（N = 19，草稿 `probability.txt`）。这个式子与正文第二节转述的「3 ÷ 写行那一段的写数 × 7/12」同值，我是从切段与抽法独立推出的。
4. 「流」不是崩溃态数：随机源按「历史种子 ^ 盐 ^ 第一次崩溃的段号」起：「let mut source = SeededRandomSource::from_seed(」（`crates/singlefs-checker-tier/src/crash_injection.rs:2541`）。同一个第一次崩溃段里的崩溃态，只要挂载写表布局相同，抽到的二次崩溃态逐个相同。所以用例里「崩溃态数以百计」（`crates/singlefs-checker-tier/src/crash_injection.rs:5332`，那条注释自己标「推的，没量过」）不是独立抽样数；独立流数 ≈ 候选段数（V242u 下 13，探针量的）乘每段里不同的挂载布局数（没量，段内恢复落到不同版本时 N 与段表不同）。
5. 抽样是定值：历史种子写死 `HistorySeed(0)`，随机源是确定的，所以这一条用例在 V242u 下要么红、要么不红，重跑不变；「抽不抽得中」是一个固定的 0/1，不是概率事件，概率只描述我事前推不出它。

推的数（独立流假设，布局数没量）：流数 13 时至少中一次 71.5%，26 时 91.9%，39 时 97.7%（`probability.txt`）。

K1-e 判定：**推得出红（路径成立），但命中是否发生我推不出**；下界 ≈ 71%（只算 13 条流）。要坐实只能在 checker 档把 V242u 那一行跑一遍（命令见改动全文 `research/prompts/_m3-formal-code-b18-r1-diff.md:665` 的第 5 列），我没跑、复核不了（checker 档）。
会推翻它的现象：(1) V242u 行在 checker 档跑出绿（那就是这一个固定种子没抽中，或第 2、3 条里我漏了一个红态要满足的条件）；(2) 探针以外的第一次崩溃态上写行那一段的 N 不是 19 且更大（p 变小）。
探针的边界：布局只在「历史全持久」的那个池上量，没逐个第一次崩溃态量；草稿探针没有复刻 `draw_second_crash_points`，也没有算出各段实际抽到什么。

## 三、K3　P3 卸载那一格

### K3-a　模型的答、D16 补进的那一句、实现，逐字段对

条款（两处都是同一个意思）：

- 「现行那一版树表 0 条（mkfs 之后只暖机、还没写过第一个文件）时一个字节都不写、F 不动、不推空发布，卸载照常做成」（`.claude/kb/decisions/16-发布语义.md:42`，「正常卸载」那一行的末句）
- 「树表 0 条那一版上正常卸载一个字节都不写、F 不动：用户定案 2026-10-04」（`.claude/kb/decisions/16-发布语义.md:77`，这一条在我开工时在第 76 行：D16 在开工之后被别的会话改过，`sha256sum` 现为 eeeb6563…，快照 `kb-sha256.txt` 记的是 0e43f344…，多出来的是「抬 F」一段里的一条依据，与卸载无关；表里「正常卸载」那一行仍在第 42 行）

模型：「let roots = if current.contents.has_trees() {」（`crates/singlefs-harness/src/model.rs:2508`），树表 0 条时 `roots` 取 `Vec::new()`；注释写「一个字节都不写、F 不动，答零条根、会话关」（`crates/singlefs-harness/src/model.rs:2496`）；会话关：「answer.session_after_success = ModelSessionAfterSuccess::Closed;」（`crates/singlefs-harness/src/model.rs:2525`）。
实现：「PoolVersion::WithoutFile(current_version_without_file) => {」（`crates/singlefs-core/src/mount.rs:2474`）一进来就「return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {」（`crates/singlefs-core/src/mount.rs:2475`），在「let raised = raise_the_floor_through(」（`crates/singlefs-core/src/mount.rs:2484`）之前，一次设备调用也没有。观测侧：「Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => Vec::new(),」（`crates/singlefs-harness/src/model_comparison.rs:653`）。

| 字段 | D16 补进的那一句 | 模型（`answer_unmount`） | 实现（`mount::unmount`） | 判定 |
|---|---|---|---|---|
| 写字节 | 一个字节都不写 | `expected_roots` 空；用例钉盘上写过的扇区前后逐字节相同（测试文件 360-372 行） | 早退，不碰设备（2474-2481 行） | 一致 |
| F | F 不动 | F 不另存，由环里的根现算（`crates/singlefs-harness/src/model.rs:1255-1275`）；没有新根，F 不变；用例钉前后都是 0（「树表 0 条的一版上卸载不抬 F」，`crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs:392`） | 早退，`allocator`、`current` 一个字没动 | 一致 |
| 空发布条数 | 不推 | 零条根；对拍 `roots_compared` 钉 0（用例钉 `(1, 0)`） | `Vec::new()`（`model_comparison.rs:653`） | 一致 |
| 卸载照常做成 | 做成、不报错 | 没有必须拒绝或允许拒绝的理由（`answer_for_publishes` 里两个集合为空）；整串预演的墙那一格因没有计划中的发布而无墙可拒 | 不报错、不核盘表与参数：「现行那一版树表 0 条时不报错、不核（一个字节都不写）」（`crates/singlefs-core/src/mount.rs:2463`） | 一致；条款没说「不核盘表」，模型也没拒，两边同口径，按「规则没说」记 |
| 会话 | （条款没写） | 会话关（2525 行）；用例钉 `has_writable_session()` 为假 | 写入口不再发布由调用方关（`unmount` 不碰会话） | 一致（条款没写） |
| 「树表 0 条」的划法 | mkfs 之后只暖机、还没写过第一个文件 | `has_trees()` 为假只有 `WithoutTrees`：「ModelVersionContents::WithoutTrees => false,」（`crates/singlefs-harness/src/model.rs:322`），`TreesAndNoFile`、`InodesWithoutTheFirstFile`、`FileBeingDeleted` 都是真 | 只有 `WithoutFile` 早退；有树的两种：「WithTreesAndNoFile(TransactionOutput),」（`crates/singlefs-core/src/transaction.rs:2042`）与 `WithFile` 一样推空发布 | 一致：两边的「0 条」同一个划法，删完那一版（有树无文件）两边都推 |

我复跑的（harness 档，经内存包装，副本在 `/tmp/claude-1000/m3-formal-code-b18-r1-sonnet/ws/`，命令与原样输出落在 `research/prompts/m3-formal-code-b18-r1-sonnet-model/harness-unmount-test.log`）：

    $ cargo test --offline -p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version
    running 5 tests
    test unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees ... ok
    test unmount_on_an_overwrite_at_txg_four_pushes_three_empty_publishes_carrying_floor_four_and_the_model_agrees ... ok
    test an_unmount_whose_roots_carry_the_floor_of_the_current_version_instead_of_its_txg_is_reported_at_the_rollback_floor ... ok
    test unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees ... ok
    test an_unmount_that_pushes_empty_publishes_on_a_version_without_trees_is_reported_at_the_publish_count ... ok
    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.61s

K3-a 判定：**一致**（字段逐个对上；两处「条款没写」：不核盘表、会话状态）。
会推翻它的现象：一个树表 0 条的池上，实现的 `unmount` 推了空发布或写了任何字节（M2 那一行会红）；或模型在 `has_trees()` 为假之外的版本上答零条根（M1 会红）。

### K3-b　删掉的那一行与新加三行各证什么

| 行 | 行文（改动全文里的那一行） | 它证的 | 我在副本上套了它的结果 |
|---|---|---|---|
| 删 | 「模型在树表 0 条的一版上也答卸载（条款没判的那一格不再停下）」（`research/prompts/_m3-formal-code-b18-r1-diff.md:857`） | 旧决定的反面：条款没判、模型必须停下报「模型答不了」。旧用例 `the_model_does_not_answer_unmount_on_a_version_without_trees` 已被换掉，这一行没有可红的用例了 | 不套（对应用例不在冻结树里；按新决定它描述的是已作废的行为），删它是对的 |
| 加 1 | 「模型在树表 0 条的一版上推空发布（用户 2026-10-04 定不推：一个字节都不写、答零条根）」（`research/prompts/_m3-formal-code-b18-r1-diff.md:853`） | 模型一侧：0 树版本上模型答零条根。`has_trees()` 改成恒真，模型推空发布，与实现的零条根对不上 | 红：退出码 101，红在 `unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees`，panic 在测试文件 57 行（`judged_by_the_model` 里「模型与实现对不上」）|
| 加 2 | 「实现在树表 0 条的一版上卸载写了东西（用户 2026-10-04 定一个字节都不写；盘上逐字节比）」（`research/prompts/_m3-formal-code-b18-r1-diff.md:854`） | 实现一侧：早退那一支里往设备 0 写 4096 字节，盘上写过的扇区前后比不等 | 红：退出码 101，同一条用例，panic 在测试文件 369 行（逐字节比那个 `assert_eq!`） |
| 加 3 | 「对拍不比根的条数（树表 0 条的一版上实现推了空发布不报）」（`research/prompts/_m3-formal-code-b18-r1-diff.md:855`） | 对拍器一侧：`judge_roots` 的条数比较（`crates/singlefs-harness/src/model.rs:2850`）。用例 2 拿别的池（带文件）上卸载交回的两条根，当成这一格实现交回的，模型答零条，期望对拍报 `PublishCount`（测试文件 420 行） | 红：退出码 101，红在 `an_unmount_that_pushes_empty_publishes_on_a_version_without_trees_is_reported_at_the_publish_count`，panic 在测试文件 413 行 |

三条变异各自套在副本上、跑各自点名的用例，原样输出在 `research/prompts/m3-formal-code-b18-r1-sonnet-model/mutations-18-2.log`，脚本 `research/prompts/m3-formal-code-b18-r1-sonnet-model/run-three-mutations.py`（每条套完即还原，`cmp` 过副本里 model.rs、mount.rs 与冻结树相同）：

    M1 model.rs has_trees -> true exit= 101       （红在 unmount_on_a_version_without_trees_… ，测试文件 57 行第 13 列）
    M2 mount.rs 卸载树表 0 条那一支写 4096 字节 exit= 101  （红在同一条，测试文件 369:5）
    M3 model.rs judge_roots 不比条数 exit= 101    （红在 an_unmount_that_pushes_empty_publishes_…，测试文件 413:5）

三行之间的分工：加 1 管模型、加 2 管实现、加 3 管对拍器，三方各一条，互不重复；用例 2 是「假想的实现错」，不跑真实现，它证的是对拍器认得出条数不对，不证实现。
没有独立变异行的断言（只记，不判）：「F 不动」那条断言（测试文件 392 行）与「会话关」那条（387 行）：F 是模型由环里根现算的，树表 0 条那一支没有新根，所以 F 不动是结构性的；会话关由 `answer_unmount` 末尾统一设（2525 行），带树的两条用例也钉了它。这两条没有只对它们红的变异行，也就是说它们是「钉住了、但没证过会红」。
K3-b 判定：**一致（三行各证各的，我套了三行都红）**；删掉的那行作废合理。
会推翻它的现象：把任一变异套在副本上，点名的用例仍绿——我这一次三条都红，抽样只一次（变异是确定的，不涉及抽样）。

### K3-c　同时合入的删除批 GA 改动，有没有让「树表 0 条那一版不写」不成立

GA 落在哪：卸载那一串回收出来的槽「释放代不高于每块盘上最新有效根」那一截当场可用：「RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice,」（`crates/singlefs-core/src/mount.rs:2346`）。它是 `raise_the_floor_through` 里的分支，而 `unmount` 在树表 0 条时于调用它之前就早退（2474-2481 行早退，2484 行才调）。所以 GA 够不到树表 0 条那一支，这句今天在实现上仍成立。

三条独立的核（互不共用同一段代码或同一次采样）：

1. 读代码：早退在 `raise_the_floor_through` 之前（上面两处行号）。
2. harness 档用例复跑：与卸载、GA、回收门槛相关的 8 个测试目标在冻结副本上全绿（命令与原样输出落在 `research/prompts/m3-formal-code-b18-r1-sonnet-model/harness-unmount-related-8-targets.log`；这 8 个目标取自我开跑前 `ps` 里看到的别的会话那条 cargo 命令点名的目标，我另核了每个文件存在）：

        a_lone_surviving_device_reads_back_what_it_read_before_the_unmount   2 passed
        admission_raises_the_floor_before_refusing                           25 passed; 1 ignored
        model_answers_unmount_with_the_floor_at_the_current_version          5 passed
        mount_transparency_judged_field_by_field_after_a_writable_mount      33 passed
        rebuilt_root_takes_every_new_root_segment_field_from_the_record      2 passed
        reuse_after_raising_the_floor                                        3 passed; 13 ignored
        unmount_marker_of_a_root_rebuilt_from_the_records_of_the_unmount_sequence 1 passed
        writable_mount_of_a_formatted_pool                                   15 passed

   （这些 `ignored` 是用例自己标的，我没动。）
3. 变异：把早退那一支改成往设备写 4096 字节（M2），点名的用例红（见 K3-b），说明「不写」这句有用例在看着，不是只靠读代码。

checker 档里另有一条同义用例 `unmount_of_a_version_without_file_writes_nothing_and_succeeds`（注释：「盘上逐字节不变（系统配置槽、根环、录制流一步不多），分配器与现行版本不动」，`crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs:573`），我没跑（checker 档）。
K3-c 判定：**一致，GA 没有让这句不成立**。
会推翻它的现象：GA 之后有第二个会改盘的入口在 `unmount` 早退之前被调（我读到的 `unmount` 里早退之前只有 `match current`）；或 8 个目标里任一个红（这一次全绿）。

### K3 附记（不是格的判定，只记下我读到的）

- 来历的措辞没跟上：实现里这一支的注释还写「主 agent 2026-09-26 定，实二交回第一问；推的，被攻过零轮」（`crates/singlefs-core/src/mount.rs:2299`），D16 已写成用户 2026-10-04 定案（`.claude/kb/decisions/16-发布语义.md:77`）。两句不矛盾（用户把前一条定案「写进条款」），只是注释没提用户这次定案。
- 装置里一句旧注释：「模型这一步不交答：取号、暖机（两次都返回之后一起推一步）与卸载（`model.rs` 里没有 `answer_unmount` 一族）。」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:237`）。`model.rs` 里今天有 `answer_unmount`（`crates/singlefs-harness/src/model.rs:2505`），这句不再成立；它属于 C673 ④ 没做的那一块（E171 的 O1 还没接卸载的答），不在这一批的补丁里。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-a 判官五样输入 | 一致 | 镜像、拒后池、所选版本、读出在不在集合、读故障认定都取对；读故障沿用第一次崩溃态有代码依据（第二截那次挂载不注入读故障） |
| K1-b 与 I-8.4 的先后 | 一致 | 先记 I-8.4 没判，再判 C670，二者互不挤占（读代码，用例钉了两样；checker 档没跑） |
| K1-c 只判 C670、不拿 U6 其余定红绿 | 冲突（范围，已记欠） | D23 字面管所有拒、盘上逐字节不变；第三截只在 C670 成立时才报红，再挂写了盘而 C670 不成立时不红 |
| K1-d 没崩过的那次被拒时不判 | 一致；漏的类是第 2 层 | 拒前一个字节没写的没有二次崩溃态（空集）；漏的是「写过东西才被拒」那一类（`Publish`、`FloorRaiseFailed…` 等），数量没量到；条款没写这一类要不要判，按规则没说记 |
| K1-e V242u 推得出红吗 | 推得出红，命中推不出 | 路径成立；同段共用抽样流，独立流数 ≈ 候选段数 13（探针量），每流命中 7/76，至少中一次 ≥ 71%（推的）；种子定值，要在 checker 档跑 V242u 那一行才坐实，我没跑 |
| K3-a 模型的答对 D16 与实现 | 一致 | 写字节、F、条数、做成、「0 条」划法逐字段对上；5 个用例复跑全绿 |
| K3-b 删一行与加三行 | 一致 | 加 1 管模型、加 2 管实现、加 3 管对拍器，三条在副本上套了都红；删掉的是已作废的旧行为；「F 不动」「会话关」两条断言没有只对它们红的变异行 |
| K3-c 删除批 GA | 一致，没有让这句不成立 | GA 在 `raise_the_floor_through` 里，`unmount` 在它之前早退；8 个相关目标全绿 |

## 五、复核不了与要主 agent 跑的

- 复核不了（checker 档）：K1-b 里「I-8.4 没判与 C670 判过各计 1」的内联用例是否绿；K1-e 的 V242u 行是否红、红在哪个崩溃态；基线上第三截会不会对合法态误红（第 18 批之前最近一次放量提交 `fcc4d340` 的标题写「26.16 亿态零红」，那是在没有第三截这一红源时量的，推的，我只读了提交标题）。
- 要主 agent 跑的命令（原样取自改动全文第 665 行的第 5 列；checker 档，我没跑）：`-p singlefs-checker-tier --lib -- crash_injection::tests::no_remount_after_a_second_crash_without_read_faults_is_refused_writable_for_a_witness_newer_than_the_selected_version`，套 V242u 之前先跑一次基线要绿；看判结果的是这条用例的 `test result` 行。
- 我复核得动、已复跑的：K3 的 5 个用例、3 条变异、8 个相关目标、C670 判官的 3 个 harness 档用例（`writable_mount_refusal_on_a_crash_state_without_read_faults`，3 passed，原样输出在 `harness-c670-judge-test.log`）。

## 六、没做什么

- 没跑 checker 档（`singlefs-checker-tier` 的测试、门禁 `checker-tier-*`），没跑变异整表，没跑任何全量；没复刻 `draw_second_crash_points` 去算各段实际抽到什么（那是攻方腿的活，也近乎把 checker 档那条用例换个写法再跑一遍）。
- 不判别的格：K2 不归我；没替主 agent 采纳或出判决；「U6 不延到第三截」「没崩过被拒时第三截不判」两件正文第二节已判不改、记欠的，我只给条款与代码的并排，没改它们的结论。
- 副本与编译目录已删（交回闸要求）：`/tmp/claude-1000/m3-formal-code-b18-r1-sonnet/` 整个目录，删前 `du -sh`：编译目录 `target/` 2.5G、副本 `ws/`（冻结树 `crates/` 加 `Cargo.toml`、`Cargo.lock`）16M、草稿 `scratch/` 36K。要复跑的材料都在模型目录 `research/prompts/m3-formal-code-b18-r1-sonnet-model/`（探针源、两个脚本、各日志的原样输出）；复跑照规则把它拷到草稿目录、在冻结树副本里经内存包装跑。变异脚本每条套完都还原过，当时 `cmp` 过副本里被改的三份源文件与冻结树逐字节相同。
- 包装脚本 `run-with-memory-cap.sh` 每次跑完会把峰值记进 `research/scripts/memory-peaks.tsv`（它自己的记账，不是我写的报告产物）；`git status` 里我没看到它被改（该文件的状态没有显示）。
