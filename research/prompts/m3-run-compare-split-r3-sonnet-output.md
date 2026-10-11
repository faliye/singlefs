# 崩溃点重放拆成「跑数据 / 对比数据」两段：设计轮第三轮（最终轮）辩方腿报告（2026-10-10）

立场：辩方。复核第二轮判决（`research/prompts/m3-run-compare-split-r2-main-verification.md`）第二节逐格与第三节七条，替判输的一方（辩方 S7、被判「坐实」「采」的几格里站不稳的）和被第三节替掉的原形态找站得住的理由；只答正文第四节第 1、2 问。不替主 agent 撤销或维持判决。

## 0. 读法

- 证据标签：**[读]** 读代码；**[算]** 算的；**[量]** 跑出来的（只在我的模型上）；**[推]** 推的。
- 代码引用三种：`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/...` 是 E5 副本（主工作区加 V…E5，只读）；`crates/...` 没写绝对路径的是主工作区（`crash_amplification.rs`、`crash_injection.rs` 等被补丁动过的文件我只引 E5 副本）；E7 副本与 E5 在我引的几处逐字相同（`grep -nF` 比过 `given-states-input-v1` 与 `table_name: format!("seed=` 两行，行号同为 8776、2075）。
- 没读禁读清单里的东西。没编译、没跑 checker 档。唯一跑的是三份小 Python（`research/prompts/m3-run-compare-split-r3-sonnet-model/`，纯 std，经 `run-with-memory-cap.sh` 或秒级）。
- 日志行（`~/.local/share/singlefs/crash-amplification/` 下）cite-check 不一定认，我写文件名加行号。

## 1. 先说结论

第二轮判决第二节我逐格过了。没有一格该翻成相反方向，但有五处判决站的理由或补的办法够不着，一处判决理由被代码否掉：

1. **S7「不采」的理由被代码否掉。** 判决与辩方 S7 自己都说「执行器边跑边问模型来定删除步数」。`history.rs:3879` 起那一圈循环遍历的是 `published_in_txg_order`，即会话已经发完的版本，模型只是在事后挑「走空的一步」还是「这次改动本身」那份答案来比。执行器的操作序列是事先生成好的（`history.rs:5843`），模型对控制流唯一的影响是第一次对不上就停（判停在 `history.rs:5897`，原文见 3.1），而用户定的「红步之后照判」本来就要去掉这个停。S7 本体（模型与 RefFS 的答案在对比段预处理里重放）还活着，它同时消掉 R7 的整条链和 `14` 的「模型派生常量」缺口，并让 `12`/`14` 的录入版本不必罩 `model.rs`。它的遗留代价写在 3.1。
2. **R10「采」的前提在今天的代码上不成立。** 给定状态表的块键输入里有表名，表名里有 `seed=`（E5 `crash_injection.rs:2075`、`:3887`，读故障档测试 `:352`），所以块键不是纯内容键，不会跨历史去重；一把块键对应一把历史是函数关系。「（块键，历史键）」这个键仍有用，但理由是 R1（同一个种子、执行参数变了），不是 R10。
3. **R8 的数字没被取样点推翻，反而偏低。** 攻方三组权重里有两对是跨流、跨展开深度的反事实对。我从 9/29、9/30 两趟日志里另取四对（含一对同流同深度的真对），块换主 44–51%（3.2）。同时判决补的办法「块归属录入时写进库、权重只管没录的块」没回答：权重量的是谁的速度。`states_per_second_on_this_host` 是 GPU 短试判的速度（`crash_amplification.rs:11040`），拿它分 CPU 录入的活是个未定义。
4. **R2 的补法「现成的 `kernel_sources_digest`」是固定三份内核合一的编译缓存键；** 撤掉静态自证之后，新内核宿主忘记绑摘要没有任何检查兜着（原自证之二在负对照 k-amp 上是红的）。
5. **R7「第 m 步多半判绿」是推的，判决没靠它，站得住；** 但补法（第 7 条）可以直接沿用今天已有的弃权票形状 `TheFirstCrashWasNotJudgedGreen`，不用另造「单列」。
6. 第三节第 6 条「钉内容」有两种读法（钉操作序列＋执行参数／钉写表），判决没选；第 4 条「历史跑数据按历史键归台」与给定状态表按指纹分台冲突。交用户表五行的候选还缺一条（RefFS／模型放哪一段）。

## 2. 第一问：第二轮判决第二节逐格

判定三种：站得住 / 够不着（判决引的东西说的不是那件事，或补的办法没罩到）/ 同样打中别的臂。证据详情在第 3 节。

| 判决条目 | 辩方判定 | 证据（要点） |
|---|---|---|
| 辩方：第一轮第二节一格不翻 | 站得住 | 我没找到该翻的一格 |
| 读故障那一标改每个崩溃态存一位 | 站得住 | 这一位只由步种、每步之后的流长与窗口末尾的录制流下标定（E5 `crash_injection.rs:1098` 起的 `read_faults_in_a_crash_state`），在 `:1651` 算一次，`:1659`、`:1715`、`:1736`、`:1784` 读同一个变量 [读] |
| 随机历史封顶：攻方量 71–136 步 | 站得住 | 取样点是 BROAD 比重、两块 4 GiB 盘、不跑每步 checker；快档与读故障档都是 24 步，离 2048 远，判决没有一条依赖它 |
| R1 历史键靠手写串、录入版本不含设点测试文件 | 站得住 | `crash_amplification.rs:8776` 的块键只含项名、种子与规模串、表名、基线、写表、接缝、持久集合，不含执行参数；改法（历史键按内容取＋录入版本并设点闭包）罩得住 |
| R2 自证之二放过「在错的那一把里」 | **站得住，补法够不着一半** | 现成函数是固定三份合一的编译缓存键；静态自证撤掉后新宿主没有兜底（3.4） |
| R3 入口取写表函数所在文件，闭包含 WGSL 与判器 | 站得住 | 主 agent 已复跑；与 S2、S6 合成一条无异议 |
| R4 自证之一只看一层调用方（推的） | 站得住 | 判决只撤它，没拿它当打中 |
| R5 簇 0 验收词法多重集两头错 | 站得住 | 取样点只有两棵手改的树（move 没编译），判决已把它自报的漏处另配名字解析检查 |
| R6 己的检查与 J6、退回撞 | 站得住 | 我多核了一处：GPU 判不了的状态「照 H6 判红」，往下走的结论取 CPU 的（E5 `crash_amplification.rs:9145`），所以「当结论用的那份必须来自 GPU」在退回那一路上本来就红，不与改法冲突 [读] |
| R7 RefFS 历史内读出越界后的票 | **结构站得住；「多半判绿」不是判决的依据，也没被证；补法可以更省** | 3.5 |
| R8 四成块换主 | **站得住，数偏低；补法没定义权重** | 3.2 |
| R9 P12、P17 没落处 | 站得住 | 我没动它 |
| R10 块键跨历史去重，单值「属于」装不下 | **够不着** | 块键里有 `seed=`，不跨历史（3.3） |
| N1 层 0 按片分台、片长小于一块 | 站得住 | 我没重算；判决已复跑相同 |
| N2 钉回快档按历史键漂 | 站得住，判决补的「钉内容」没定义 | 3.6 |
| N3 `12`/`14` 没有归属规则 | 站得住，判决给的两个候选里第一个与今天的分台冲突 | 3.6 |
| Q3 态数基数是下界 | 站得住 | 日志行我没重核，主 agent 已现查 |
| S4、S5 采 | 站得住 | 我没动它 |
| S7 不采 | **够不着** | 3.1 |
| 第三节第 7 条、交用户表 | 见第二问（第 4 节） | |

## 3. 展开：判决够不着或补法没罩到的几处

### 3.1 S7 模型与 RefFS 在对比段重放：判决的理由被代码否掉 [读]

判决原句：「不采：执行器边跑边问模型定删除步数（`history.rs:3883` 一带），辩方自己已否」（`research/prompts/m3-run-compare-split-r2-main-verification.md:45`）。辩方 S7 的否定理由与它是同一句，我在这里替 S7 重查。

- 操作序列事先生成，执行器逐步照做，不看模型：「for (step_index, operation) in history.operations.iter().enumerate() {」（`crates/singlefs-harness/src/history.rs:5843`），「} = apply_operation(pool, operation, step_index);」（`crates/singlefs-harness/src/history.rs:5850`）。我把 `history.rs` 里全部 `model.` 命中行过了一遍（`grep -n 'model\.' history.rs | wc -l` 得 24 行：1452、1646、1647 与 3281 起 21 行）：没有一处拿模型的答案去决定「接着对实现做什么」，全是取答案、事后比、推进。
- 判决点名的 `:3883` 一带是事后循环。unlink 先让真会话发完：「let change_result = mounted_session.publish_user_change(」（`crates/singlefs-harness/src/history.rs:3823`），再交模型逐版比：「settle_user_change_comparing_every_version_published(」（`crates/singlefs-harness/src/history.rs:3842`）。比的循环遍历的是已发成的版本：「for published in &published_in_txg_order {」（`crates/singlefs-harness/src/history.rs:3879`）。模型状态只用来挑这一版按哪份答案比：「let walking = model.current_version_has_a_file_being_deleted();」（`crates/singlefs-harness/src/history.rs:3895`）。走空走几步是会话自己定的，不是模型。
- 模型对执行器控制流唯一的影响是第一次对不上就停：「if !violations.is_empty() || harness_judgement.is_some() || model_disagreement.is_some()」（`crates/singlefs-harness/src/history.rs:5897`），文档也这么写：「第一次失败（违例、执行器判出的、模型对不上、panic）就停」（`crates/singlefs-harness/src/history.rs:5745`）。用户定的「红步之后照判」本来就要把这个停去掉；去掉之后记录段根本不需要模型。
- 模型的比与进是焊在一起的（判决第二节「分界：模型比与走焊在一起」那一格）：「/// 拿实现的结局与模型的答案比；对得上就按模型自己算的结果往前走（不按实现交回的），对不上交回哪一格。」（`crates/singlefs-harness/src/model.rs:2870`）。记录段要「只推进」就得给模型开一个不比较的口并先定对不上之后怎么走（交用户表第 2 行）；S7 形态下记录段不碰模型，这一行的四个候选都变成对比段预处理里的重放策略，换候选不用重录。

S7 活着，判决没有理由不采它；但它不是免费的，遗留四处，都是判决要选的而不是我替它选：

1. 观察者与崩溃注入今天在记录时读活模型：「for (key, file) in observation.model.committed_versions() {」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_injection.rs:1420`），回调里直接给模型引用：「model: &pool.model,」（`crates/singlefs-harness/src/history.rs:5917`）。S7 要把「模型提交过的每一版」改成对比段按记录的观察重放得出，观察者接口要换 [读]。
2. `14` 要存的是观察到的结局（发成的根、报告的上限这类 `ObservedOutcome`），不是模型派生常量；这一点比判决第 2 条更省，但存的字段要核全 [推]。
3. 重放放在对比段的 CPU 预处理（判决第 1 条第三列已有这一格）；RefFS 今天每个崩溃时刻就是从头重放：「let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:981`）。随机历史 24 步量级便宜，多记录流这类固定流要不要逐段重放没量 [推]。
4. 录入版本不再罩 `model.rs`、RefFS 与翻译（`12` 的族缩成只罩 core 读路径与录制器），结论版本要罩它们——版本表要改，但改模型不再触发重录 [推]。

**它同时回答 R7（3.5）和 `14` 缺模型派生常量**：翻译层照今天的写法在对比段重放，读出越界就停、之后的票沿用今天的 `stopped` 机制，录入段不存在「照越界读出重起头」这一步。推翻 S7 的现象：找到记录段里有一处拿模型答案去决定对实现做什么（那它就必须在记录段跑），或观察者那一处不能由重放得出。

### 3.2 R8 四成块换主：取样点与补法 [算＋读]

- 攻方三组权重来路：`runs/` 下三行 `CRASH_AMPLIFICATION_HOST_WEIGHTS`，其中 `140427,62390` 与 `136700,45275` 同是 unlink 那条流但一趟展开 30、一趟展开 28（待判块 22124 对 5463），`642805,291558` 是另一条流（multi-record-publish）。块号空间不同的两趟比块归属是反事实：同一个块号在两趟里不是同一批态。真正「同一条流同一展开深度」的一对要去 9/29、9/30 两趟全量日志里找。
- 我取的真对（`research/prompts/m3-run-compare-split-r3-sonnet-model/extract-weights.out`，原样）：9/29-f 第 45 行 `local,peer=1393344,988839` 与 9/30-i 第 52 行 `local,peer=1427848,980112`，同是 multi-record-publish、展开 28。再取三对：同流预热行 `35453,20156`→`52822,20838`；`52937,20951`→`119814,130951`（position_addressed_ExtentInlineToLowerSegment）；`4544035,5880702`→`611669,813458`（tree_split_CentralMappingRootSplit）。
- 用攻方的模型（`weighted_shares_drift.py` 原样拷贝，自检 `ok=True`）跑，`drift-real-pairs.out` 原样：

```
OWNER_DRIFT kind=pipeline before=1393344,988839 after=1427848,980112 blocks=39917 moved=19302 moved_fraction=0.4836
OWNER_DRIFT kind=pipeline before=35453,20156 after=52822,20838 blocks=39917 moved=17580 moved_fraction=0.4404
OWNER_DRIFT kind=pipeline before=52937,20951 after=119814,130951 blocks=39917 moved=20347 moved_fraction=0.5097
OWNER_DRIFT kind=pipeline before=4544035,5880702 after=611669,813458 blocks=39917 moved=19600 moved_fraction=0.4910
```

  给定状态那一路四对是 0.4798、0.4412、0.5111、0.4880。**4 对全在 44–51%**，所以攻方的 40–43% 不是取样点碰巧偏高，反而偏低。机理读代码：每一块的归属取 `block_sequence % total`，`total` 是两个权重之和且不约分：「let position = block_sequence % total;」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:984`），权重差 2.5% 周期就整个重排，趋近于随机重分（两台各约一半）。这些块数是 `39917` 个块的替身，不是真态数。
- **补法没定义权重量的是谁。** 权重来自排计划时每张卡一小段计时试判：「states_per_second_on_this_host += states_timed as f64 / seconds;」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:11040`），卡量完派活宽度之后再计时：「// 先判一小段让卡量出派活宽度（量宽度那几次派活不算速度），再计时判几次派活那么多」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:11012`）。即它是 GPU 判速。判决第 4 条「块归哪台在录入那一刻写进库，之后照表分，权重只管还没录的块」里被分的是 CPU 录入的活；日志里本机与第二台的 GPU 判速比从 0.75 到 3.0 变（上面四对加攻方三对），CPU 一边是机器常量。拿 GPU 判速去分 CPU 录入，只有「录入与对比同一台、对比 GPU 与录入 CPU 同比例」才平衡。这是未定义，不是已有的数。
- 我另算的候选（不是判决里的）：把权重量化成总和固定的小整数再轮转（`quantized.py`，`quantized.out` 原样）。总和 8、16、32 时同流同深度的真对和 tree_split 那对 0% 换主；总和 64 时又回到 48%；比例真变了的对（`52937,20951`→`119814,130951`）总和 8–32 仍换 50–75%。**只在比例稳定时有用，不是通解**；它说明「权重量一次写进配置、改了才重排」（攻方给的另一个办法）比量化更靠谱，不是我的新办法。

### 3.3 R10 块键跨历史去重：今天的块键里有种子 [读]

攻方原句：「给定状态表的块键只看内容」（`research/prompts/m3-run-compare-split-r2-opus-output.md:269`），接着说「项名与种子规模串在里面，历史序号不在」。它漏看了表名。

- 块键输入：「"given-states-input-v1\nitem={}\nseeds_and_scale={}\ntable={}\nbase={}\n",」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:8776`），文档写明含表名：「输入指纹是项（名字、种子与规模）、表名、基线镜像、写表」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:8760`）。
- 随机历史三截的表名都带种子：「table_name: format!("seed={}/first-crash/power-losses={power_losses:?}", seed.0),」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_injection.rs:2075`）；「"seed={}/second-crash/segment={}/first-crash-fingerprint={:016x}",」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_injection.rs:3887`）；读故障档测试自己的表：「table_name: format!("seed={seed}/first-crash"),」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs:352`）。E7 副本两行行号与原文相同 [读]。
- 所以两段不同种子的历史，表名不同、块键不同，不会共用一块；一把块键到一把历史是函数关系。攻方举的两例（都在 mkfs 之后第一步就崩、第三截在逐字相同的崩溃镜像上挂载）表名里种子不同，进不了同一块。没带种子的表名只在固定流里（E5 `crash_injection.rs:7912` 的 `first-file-and-one-overwrite/...`），那些表没有 `12`/`14`。
- 剩下能撞同一把块键的只有同种子、同规模、不同执行参数（R1 那一种）；判决第 2 条「键按内容取」已经罩它。
- 判定：R10 判「采」的理由（块键跨历史）够不着；第 2 条「读这一族的结论种类，键用（块键，历史键）」不必撤，但理由是 R1：同一把块键下换了执行参数，历史键内容变了，结论按新键重算。攻方原话自己也写了「可达性没量」。推翻条件：找到一处表构造不带种子、又有读 `12`/`14` 的结论。

### 3.4 R2 的补法与撤掉的自证 [读]

判决第 3 条：「结论版本的 `extra_inputs` 绑它读的录入版本，并在派活那一刻拼进实际编进管线的源码摘要（现成的 `kernel_sources_digest`）。」（`research/prompts/m3-run-compare-split-r2-main-verification.md:58`）与「撤掉第一轮的两条自证（调用方在闭包里、读 `*.wgsl` 的文件在某一把闭包里）。」（`research/prompts/m3-run-compare-split-r2-main-verification.md:59`）。

- 方向对：今天的结论键用的是静态摘要「判法代码的摘要 [`gpu_judge_digest`] 进判定表的键：内核改一个字，旧行留着当历史，新行另起。」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:19`），它靠约定「判定入库的路径不调它」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:525`）保证编进管线的就是摘要罩的；派活那一刻现算的摘要把约定变成事实。
- 但「现成的」只对今天那三个内核成立：「/// 三份内核源码合起来的摘要：同一张卡上同一套源码只编一次（[`GpuCard::pipelines_compiled_once_on_this_device`]）的键。」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:161`）。它是编译缓存键，固定三个标签：「("recovery", &kernel_sources.recovery),」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:166`），入参结构只有三个字段（`crash_judge_gpu.rs:535` 起的 `JudgeKernelSources`），管线数组定长三条（`crash_judge_dispatch.rs:573`）。判决第 5 条的新比较内核与 P14 内核是另外的管线，没有现成摘要；要么新写同形函数，要么塞进这个结构——塞进去就是三份合一，改任何一个内核，所有绑它的结论种类一起换版本（S3 当初指出的副作用）。判决没选粒度，建议写成「每种结论只绑它派的那几个内核」。
- 撤自证有损：原自证之二在攻方自己的负对照 k-amp 上是红的：「SELFPROOF2 checked=5 red=1 verdict=red」（`research/prompts/m3-run-compare-split-r2-opus-model/outputs/selfproof2.out:20`），攻方也认：「第 3 条的自证照字面判红（探针 k-amp），抓得住」（`research/prompts/m3-run-compare-split-r2-opus-output.md:352`）。它漏的是「在错的那一把里」（k-gpu 绿），不是「哪一把里都不在」。运行时绑摘要只罩走了会绑摘要的路径；新来的一个宿主文件自建管线、没人给它绑，就没有任何东西会红。**弱形的静态自证（读 `*.wgsl` 或自建管线的文件：在某把闭包里，或登记不判对错，或自己带摘要进自己表的键）值得留作兜底**，三个桶正是 S3 提的（`crash_verify_gpu.rs` 在 k-amp 树上就属第三桶：`registered=true`）。这与运行时绑摘要不冲突，判决写的是「撤掉」，我认为只该撤强形的那条量词。推翻条件：运行时绑摘要被做成派活函数的必经路径（拿不到管线就拿不到摘要、编不过），那静态兜底可以省。

### 3.5 R7 第 7 条的 RefFS 一半：判决没靠「多半判绿」，站得住；补法沿用现成的弃权票 [读＋推]

- 机制 [读]：现成的口「// 只核 hi 判不判得了；读出的内容在不在允许集合里不判」（`crates/singlefs-reference-filesystem/src/lib.rs:445`）之后无条件重起头并把读出推进版本表：「// 恢复读出、以它重新起头的那一份也是一个版本：之后可写挂载写的根映到它（r6 用户定案 H1-收严）」（`crates/singlefs-reference-filesystem/src/lib.rs:463`）。翻译层的事件今天只在读故障那一类恢复上用：「RestartAfterARecoveryThatAbandonedTheNewestRoot(VisibleState),」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:507`）。判决第 1 条「RefFS 只录」要把它用在每一次历史内恢复上，读出越界那一次也会照样重起头。
- 「第 m 步多半判绿」[推]：实现自己的盘面若与读出一致，之后在这个起点上接着走，读出落在从 V 起算的集合里，判绿；若读出本身与盘面不一致（读错了字节），之后可能红。两头都没证，我没造出带 bug 的实现去量（要编 checker 档，不跑）。判决第 7 条的分法不依赖这个推断：不管 m 步判绿判红，它都是相对一个已知越界的起点判的。
- 现成先例：今天第二次崩溃的 RefFS 票已经按「第一次没判绿就弃权」处理：「let Some(restarted) = &first_crash.restarted else {」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:920`），弃权原因「ReferenceFileSystemAbstention::TheFirstCrashWasNotJudgedGreen,」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:922`）。RefFS 自己红之后的票沿用这个形状（新增一个弃权原因）比另起「单列、不进总数、不登记新发现」的机制小；弃权票今天是怎么汇总的我没读 [没读]。
- 今天历史内读出越界的真实行为：翻译停住，之后每个崩溃时刻都拿到同一条红：「StoppedVerdict::Red { reason } => ReferenceFileSystemVote::Red {」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:994`），即同一个发现被数了 N 次。判决第 7 条「不进总数」是在改这件事，应该在实现规格里写成改动而不是保持原状 [读]。
- 与 S7 的关系：S7 形态（3.1）把翻译层放在对比段重放，今天的 `stopped` 机制原样保留，记录段没有「照读出重起头」，R7 的链不出现。

### 3.6 N2「钉内容」与 N3「按历史键归台」判决没定义清楚 [读＋推]

- N2：判决第 6 条「钉回快档钉内容不钉历史键」。「内容」有两种读法：(a) 操作序列加整份执行参数，几百字节，在当时的代码上重新执行得出写表；(b) 写表加持久集合，攻方自己说「钉表要自带写内容，体量没算」（`research/prompts/m3-run-compare-split-r2-opus-output.md:333`）。(a) 钉的是输入，代码改了写表跟着变，但钉住的就是「这一段历史的红」，核的是在出红的那份代码上同样红；(b) 把记录段整个绕过去，钉不到录入的错。判决没有选，实现员会各读各的。建议选 (a)，并写明只在 GPU 上红的那一类进开 GPU 才判的组（判决已写）。推翻条件：生成器或执行器改动让 (a) 的重新执行得出另一张表而红消失——那正是回归测试该抓的，不算推翻。
- N3：判决第 4 条「按历史键归台；或两台都做，导入时逐行比相同」（`research/prompts/m3-run-compare-split-r2-main-verification.md:63`）。给定状态表按指纹分台：「if !self.owns_the_table(&key) {」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:9253`），不归自己的表记一笔就交占位结论：「self.note(|tally| tally.tables_left_to_the_other_hosts += 1);」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:9260`）。同一段历史的各张表表名不同、指纹不同，散在两台上；所以「`12`/`14` 按历史键只在一台录」会让另一台的表读不到它——要么表的归属改成跟着历史键走（一段历史的全部表归一台），要么把 `12`/`14` 拷过去（破坏「录入不跨机」）。「两台都做」才是今天的行为（每台都把整批历史跑一遍），代价就是今天的代价。两个候选不是等价的，应拆成两条。

## 4. 第二问：第三节七条是不是同样打中别的候选；交用户表五行是否列全、定义写不写得出一句可判的话

### 4.1 第三节七条

| 条 | 辩方判定 | 说明 |
|---|---|---|
| 1 分界（读故障一位，录入段算） | 站得住 | 对所有候选都成立：不管模型与 RefFS 放哪一段，窗口末尾所在步都得有一个来源。「模型只推进、RefFS 只录」这一半有判决没比较的候选 S7（3.1） |
| 2 表与键 | 站得住，一处换理由 | `12`/`14` 并族对 S7 同样成立（字段更少）；「（块键，历史键）」留，理由由 R10 换成 R1（3.3） |
| 3 版本 | 方向站得住，两处够不着 | 叶子文件加族闭包检查对所有候选成立；`kernel_sources_digest` 粒度没选、撤自证过头（3.4） |
| 4 库与双机 | 站得住，两处没定义 | 「另起一份库」对「同库另一列族」同样解 `cp -a`；归属写进库对「录入不跨机」成立，对「录入随块拷」候选无意义。权重量什么没定义（3.2）；历史跑数据两个候选不等价（3.6） |
| 5 GPU | 站得住 | 我没动 |
| 6 簇 | 站得住，一处没选 | 己的检查改判「当结论用的那份来自 GPU」我核过退回那一路（第 2 节 R6 行）；「钉内容」两种读法（3.6） |
| 7 红步之后 | 站得住 | 实现形状可沿用弃权票（3.5）；RefFS 那一半在 S7 形态下不出现 |

### 4.2 交用户表五行：候选定义写得出一句可判的话吗

下面每个候选我给一句可判的话（写不出的标出）。判据：句子里的每个词在今天的代码或表里有对应物。

| 行 | 候选 | 一句可判的话 | 缺口 |
|---|---|---|---|
| 1 快档怎么读库 | 甲 | 快档 `cargo test` 整条带 `--features verdict-store` | 无 |
| 　 | 乙 | 库是一个 trait，RocksDB 与内存各一份实现，内存那份只给不链 RocksDB 的用例 | 无 |
| 　 | 丙 | 迁到库上的用例按文件挂 `cfg(feature = "verdict-store")`，进快档第二趟 | 无 |
| 2 模型对不上之后怎么走 | 不走 | 对不上那一步之后该段历史的模型比一律不再做，模型状态不动 | 与「之后依赖模型的判不再问」只差「状态动不动」，要写清 |
| 　 | 照实现走 | 对不上之后模型按实现交回的结局推进 | **写不出现成函数**：`judge_and_advance` 的对不上分支在 `write_root` 之前就返回 |
| 　 | 照模型自己的答走 | 对不上之后模型按 `answer.expected_roots` 推进 | **同上**：今天只有对得上才按模型自己算的往前走（`model.rs:2870` 文档），对不上交回哪一格，不推进 |
| 　 | 之后依赖模型的判不再问 | 对不上之后模型比与理想模型派生的判不做，不依赖的照判 | 有先例（`history.rs:3878` 注释），可判 |
| 3 按 `recover` 读到的字节记忆化 | 用／不用 | 用＝按（恢复读到的字节）去重，每态仍等价一次真实恢复；不用＝每态都真跑 | 无 |
| 4 观察者那块 CPU 账 | 只有一个候选 | 共享前缀的每个状态都挂载 | **这一行不是岔路**：用户已定「共享前缀的每个状态都做，不得例外」（`research/prompts/m3-run-compare-split-r1-sonnet-output.md:234`），待量的只有 M2，该挪出弹窗表放量的清单 |
| 5 历史内读出越界那一次 | 录入段照读出重起头、之后单列 | 录入段对越界读出也调现成的重起头口 | 可判 |
| 　 | 录入段当场判、照今天停 | 录入段调 `crash_and_recover`，越界就记停并让之后的票继承 | 可判，但这是在录入段做一次集合成员比较 |

验证「写不出」那两格（`model.rs`）：「/// 拿实现的结局与模型的答案比；对得上就按模型自己算的结果往前走（不按实现交回的），对不上交回哪一格。」（`crates/singlefs-harness/src/model.rs:2870`），拒绝分支的对不上直接返回：「return Err(disagreement);」（`crates/singlefs-harness/src/model.rs:2923`），推进模型的 `write_root` 在它之后：「self.write_root(root.clone());」（`crates/singlefs-harness/src/model.rs:2957`）。所以第 2 行前三个候选里有两个要先新写一个「对不上也推进」的函数，才算有定义。

### 4.3 没列进去的候选与行

1. **第 5 行缺第三个候选**：翻译层整个放对比段重放（3.1、3.5），记录段不碰 RefFS。它不需要用户的「照读出重起头」或「当场判」二选一，今天的 `stopped` 机制原样保留。
2. **第 2 行缺 S7 形态**：模型与 RefFS 都在对比段预处理重放，对不上之后怎么走是对比段的重放策略，换了不重录。
3. **缺一行双机录入去向**：录入留本机钉归属（判决第 4 条）／录入随块拷。判决只写了前者。哪个划算取决于 M3（压缩比）；3.2 的 44–51% 换主若不钉归属，后者就是每块拷录入。候选定义：钉归属＝块归属在录入那刻写进库、之后照表分；随块拷＝对比段在哪台跑，录入就先拷到哪台。缺的代价数：M3 与一趟 CPU 录入的线程时（第一轮 M1）。**零轮**。
4. 判决第 4 条的权重定义（3.2）和第 6 条的「钉内容」读法（3.6）是实现层的二选一，不上弹窗，但要在实现规格里选定。**零轮**。

## 5. 本腿提的改法，一律零轮

| 改法 | 修哪一格 | 标记 |
|---|---|---|
| 采 S7：模型与 RefFS 的答案在对比段预处理重放，`14` 存观察到的结局 | S7、R7 的 RefFS 一半、`14` 模型派生常量、`12`/`14` 版本不罩 `model.rs` | 推的；遗留四处见 3.1，零轮 |
| 块键与历史键的关系理由改为 R1 | R10 | 读代码，零轮 |
| 运行时摘要按种结论只绑它用的内核；弱形静态自证留作兜底（三个桶） | R2 | 推的，零轮 |
| RefFS 自己红之后的票沿用弃权票形状 | R7（若不采 S7） | 推的，零轮 |
| 「钉内容」选操作序列加执行参数 | N2 | 推的，零轮 |
| 历史跑数据：两台都做；归属跟着历史键走则表归属也要跟 | N3 | 读加推，零轮 |
| 加权量化到小整数 | R8 | 算的（我的模型），只在比例稳定时有用，不推荐，零轮 |

## 6. 每条结论什么现象会推翻它

| 结论 | 来源 | 推翻它的现象 |
|---|---|---|
| S7 的否定理由被代码否掉 | 读代码（3.1 的行号） | 找到一处拿模型答案决定「对实现接着做什么」的代码；或观察者那一处不能由重放得出 |
| R10 的前提在今天不成立 | 读代码（3.3） | 找到一张不带种子的表，又有读 `12`/`14` 的结论 |
| R8 的 44–51% | 算（我的模型，权重来自日志） | 入库装置上用真实块号重做；同一条流同一展开深度连跑两趟的权重落在量化格里 |
| 权重是 GPU 判速 | 读代码（`crash_amplification.rs:11040`） | 判决第 4 条实际用的是另一个权重（CPU 录入速度）——那要在规格里写出来 |
| R2 补法的粒度没选、撤自证过头 | 读代码＋攻方负对照 | 运行时摘要被做成派活的必经路径；或判决明确按种结论绑 |
| R7 沿用弃权票 | 读（先例）加推 | 弃权票的汇总方式容不下「RefFS 自己已红」这一类（我没读汇总处） |
| N2、N3 | 读加推 | 实现员只有一种读法时无事 |

## 7. 没做什么

- 没编译、没跑任何 checker 档或 harness 档测试；只跑了三份纯 Python（`weighted_shares_drift.py`、`quantized.py`、`extract-weights.sh` 的 awk）。模型目录 `research/prompts/m3-run-compare-split-r3-sonnet-model/`；`weighted_shares_drift.py` 是攻方第二轮模型的原样拷贝（sha256 与它相同：ce3f6faf…），自检 `ok=True`，我没改它。
- 没读禁读清单里的三处；没碰 kb、records、`.claude/`、`crates/`；没改副本。
- 没造带 bug 的实现去量 R7 的「第 m 步判绿还是判红」；没量 S7 形态下重放的 CPU 代价、观察者改接口的工作量。
- 没读弃权票的汇总处，所以 3.5 的「沿用弃权票比另起机制小」是推的。
- 没重核攻方的 N1（层 0 切片）、Q3（态数基数）、R3 的闭包计数、R5 的探针；它们判决已复跑或现查。
- 没核 kb 条款；本报告没引 kb 行。
- 草稿目录 `~/.local/share/singlefs/work/m3-run-compare-split-r3-defense/` 里是 `weighted_shares_drift.py`、`quantized.py` 及其输出的副本（小文件），没建仓副本与编译目录，交回闸要求删的东西无。
- 我的数都是我的模型上的数，不进 kb，要在入库装置上重做才能引。

## 8. 要主 agent 跑的命令

没有。若要把 3.2 的数做成入库装置上的数：在入库的 `WeightedShares::owner_of_block` 上加一个只读子命令，对 `extract-weights.out` 里那四对权重各跑 39917 个块号，与 `drift-real-pairs.out` 比。

## 9. cite-check

`python3 scripts/cite-check.py research/prompts/m3-run-compare-split-r3-sonnet-output.md --root . --root /home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo --background research/prompts/_m3-run-compare-split-r3-background.md` 末行（交回闸用默认根再跑一遍；E5 副本的文件我都写了绝对路径）：

```
  ✓ 核了 44 处引文，对上 44 处，没判 0 处（逐处列在上面）
```
