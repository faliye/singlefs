# c996-code-r3：判决（2026-11-13）

正文 `research/prompts/_c996-code-r3-body.md`，开工快照 `research/prompts/c996-code-r3-snapshot/`。这是 C996 这一题代码轮的第三轮、最后一轮（前两轮判决 `research/prompts/c996-code-r1-main-verification.md`、`research/prompts/c996-code-r2-main-verification.md`），同时是产品 GPU 包审核修补那一题的第二轮（第一轮判决 `research/prompts/bggpu-audit-fix-code-r1-main-verification.md`）；两题动同一批文件，合用一份开工快照。这一轮之后不再开轮：这一轮里新冒出来的改法都是零轮，写进「三、改法」与「交用户的」。

写判决之前拿快照 `sha256sum -c research/prompts/c996-code-r3-snapshot/crates-sha256.txt`：15 份在主工作区与冻结副本 `~/.local/share/singlefs/work/c996-code-r3/repo/` 都对得上，腿跑着的时候没人改过。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（Sonnet） | `research/prompts/c996-code-r3-sonnet-output.md` | 两份判决的改法逐条「照写了」，其中新七「读法与判决字面相反」、壬「实验装置没共用」、癸「仓里没有非测试的读者」、改法 9「另有三处新分支没有行锚」、改法 10「另有残留」；甲「只打中这份实现」；清单外六条没交用户；条款写了代码没做两处、代码做了条款没写四处；「验证到的数」都核得出，变异行三趟现判各 1 条、其余复用；检查单「类型与名字」「带 --item」不成立，其余成立；正文说那处 `debug_assert!` 在 release 里不判是错的 |
| 攻方（Opus） | `research/prompts/c996-code-r3-opus-output.md`，模型 `research/prompts/c996-code-r3-opus-model/` | 前两轮判为站住的格都站住、十三格都「修掉了」；三·甲（第 4 次开卡没回来时切 cpu 再切回同一张卡，撞 `debug_assert!`、毒化锁、会话锁里 panic）打中、重；三·乙（进程级那张表把别的挂载正在开的卡算成没回来）打中、中；三·丙（选段打分那一段重复键压过真的 GPU 出错）打中、轻；三·丁、戊、己条款字面或两读、交用户；检查单「走不到的理由」「类型与名字」不成立，「多机」「带 --item」「新崩溃流」推不出 |

核查：

- 开工快照那一刻的树，任务 15498 退 0，命令原样：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/c996-code-r3-main -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py c996-code-r3 --out /home/fy5090/.local/share/singlefs/work/c996-code-r3-main/c996-code-r3-tree --repository /home/fy5090/code/singlefs
```

  末行「✓ 快照 c996-code-r3：清单点名 17 份，倒推出 17 份，sha256 对上 17 份」（17 份都取工作区此刻那一份：快照之后没人改过）。
- 引文核对，命令原样：

```
python3 scripts/agents/cite-check.py research/prompts/c996-code-r3-opus-output.md research/prompts/c996-code-r3-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/c996-code-r3-main/c996-code-r3-tree --background research/prompts/_c996-code-r3-background.md
```

  退 0，末行「✓ 核了 32 处引文、165 处裸行号，对上 29 处，没判 168 处（其中裸行号没有引文可比 140 处；逐处列在上面）」。没判的 3 处引文：攻方引仓根 `Cargo.toml` 第 25、27 行两处（只写文件名），主 agent 在冻结副本现读：第 25 行「overflow-checks = true」、第 27 行「debug-assertions = true」，对得上；辩方引 `crates/singlefs-core/src/mounted_session.rs:5192` 一处（那份文件不在快照里），不靠它下判决。
- 攻方模型复跑：模型目录拷到 `~/.local/share/singlefs/work/c996-code-r3-main/model`（7 份与原目录 sha256 逐份相同），在拷贝上交任务 15497，退 0。读数行原样存在 `research/prompts/c996-code-r3-main-checks/attack-model-lines-task-15497.txt`（99 行）：「name=apply_model inserted=3」「name=prediction_check predicted=27 ran=27 matched=27」，两个包的「test result:」行各一（产品 GPU 包「13 passed; 2 failed」，核心层「12 passed; 0 failed」）。红的两条就是三·甲、三·乙。
- 辩方点名、合入后验证没跑到的三个测试目标，在主工作区交：`singlefs-verification` 的 `compaction_on_the_cpu_while_stopped_after_a_gpu_failure` 任务 15499「test result: ok. 25 passed」；`singlefs-background-gpu` 的 `gpu_gc_formal_contracts_through_the_host` 任务 15500「test result: ok. 2 passed」；文档测试 `--doc memory_limit_and_chunks` 任务 15501「test result: ok. 6 passed」。
- 主 agent 在冻结副本里读过、判决要靠的行：`crates/singlefs-background-gpu/src/background_gpu.rs:1432`–`:1438`（算这张卡剩几次与那处 `debug_assert!`）、`:1451`（叫停只在每一次开卡之前看）、`:1470`（开完那一次记进次数）、`:2635`–`:2650`（交设置那一刻判试满、`opening_at_a_switch` 为空就起线程）、`:2666`–`:2667`（交 `cpu` 时取走还在开的那一次并叫停）、`:1276`–`:1296`（一开卡就记进那张表、线程结束才划掉）。
- 核不动的：两处打中的窗口在真驱动上有多长（开一次卡几秒还是到 5 分钟的点），模型用替身卡与用例放开的时刻造；丙的改法在真卡上的行为（上一轮就没验）。

## 二、判决

| 格 | 判决 | 依据 |
|---|---|---|
| 前两轮判为站住、修掉的格（c996-code-r2 的甲、乙、丙、戊 ①②、己、庚、设计问题 ③；bggpu-audit-fix-code-r1 的 G01、G03 两段、G04 算术） | 站住 | 两条腿各推一遍都没打中；攻方移植的原模型用例 25 条 ok（任务 15497）。丁的「每次交设置各数 4 次」已被新七与用户定案改写，不再是现状读数 |
| 十三格改法换没换到东西（新一、新二、新三、新四、新七、壬、癸、新五、新六；甲、乙、丙、重复键只放弃这一批） | 修掉了；壬与新七各带下面一处打中 | 任务 15497：「name=attack_model_sweep_summary rows=10 panicked=0」（甲，原来 4 行 panic）；「name=attack_model_live_slot_summary rows=9 rows_not_returning_the_repeated_key=0」（乙）；壬的共用表那一臂「later_mount_started=false starts_after_the_later_mount=1」；癸、新五、新六、丙是读码，辩方给了文件:行 |
| 换类型（字节数、条数、输入长度） | 站住 | 攻方逐处对过约 1,600 行算术没找到换反的量；「name=attack_model_staging_arithmetic fit_cases=200000 fit_disagreements=0 cut_cases=200000 chunks_over_the_limit=0 pieces_not_largest=0 cut_refusals_wrong=0」（任务 15497，换类型之后的签名上重算）；文档测试 6 段在主工作区过（任务 15501） |
| 换了算法的那句断言 | 站住 | 攻方：左边只在某份缓冲内容长 0 或长度不是 4 的整数倍时与原来不同，产品三段的建法都补齐、非空；合入后验证与任务 15499、15500 没红 |
| 三·甲：第 4 次开卡还没回来时切到 `cpu` 再切回同一张卡 | **打中（重）** | 任务 15497 第 44、48 行：`held_opening=4 … switch_back_while_held=true … poll_panicked_with=Some("连着试满的卡在交设置那一刻、为在等的那一组起开卡之前就答开不成，不为它起开卡线程") … later_switch_panicked_with=Some("开卡那一步只在开卡线程上锁着用…锁没被毒化: PoisonError { .. }")`；同一前缀下 `switch_back_while_held=false`（第 45、49 行）当场答 `OpeningsInThisMountUsedUp { failed_openings: 4 }`、不 panic，`held_opening=3` 的 6 段都不 panic：分辨臂是用户切回的时机。辩方判这处断言「走不到」是漏了「交 `cpu` 时取走在开的那一次」之后 `opening_at_a_switch` 为空这一支。产品构建开着断言（`Cargo.toml:27`） |
| 三·乙：同一进程两次挂载同时开同一张卡，后到的一方被算成「前一次开卡没回来」 | **打中（中）** | 任务 15497 第 27 行（`arm=shared_table`）四个原因都是 `EarlierOpeningOfTheDeviceNotReturned`、另一次挂载那一次开成；第 36 行（`arm=table_of_each_mount`）两次都 `PutInEffect`。壬的改法（表挪到进程级）带出来的；成员文档「出路只有重启进程」对这一种是假话 |
| 三·丙：选段打分那一段重复键压过同一批真的 GPU 出错 | 打中（轻；要调用方组批先有 bug） | 任务 15497 第 25 行「gpu_failures_reported_as_the_repeated_key=3」；校验和、区域扫描两段次序相反（卡出错先报）。三段不一致 |
| 辩方未交二：前一次开卡没回来也记进连着开不成，满 4 次答「试满、要卸载重挂」，而真出路是重启进程 | 成立（读码，没模型） | `crates/singlefs-background-gpu/src/background_gpu.rs:1383`–`:1389`（`EarlierOpeningOfTheDeviceNotReturned` 算开不成）；随三·乙的改法一起处理 |
| 新七的读法（连着数、开成清零）与 c996-code-r2 判决第 70 行（累计、卸载重挂清零）相反 | 用户定「连着数，开成清零」；上一份判决那一行的说法作废，以这一份为准 | 用户 2026-11-13 弹窗 |
| 检查单：走不到的理由二次查实 | 不成立 | 三·甲走到批次 3 报告列的那处 `debug_assert!`；`batch_not_split_over_the_gpus` 那一处断言两条腿都判走不到。正文写那处 `debug_assert!`「在 release 里不判」是主 agent 写错了，仓根开着 `debug-assertions = true` |
| 检查单：类型与名字 | 不成立 | `EarlierOpeningOfTheDeviceNotReturned` 的文档对三·乙是假话；新类型内值都是 `pub`、取内值再包编得过（混用只在签名处拦得住）；`FailedOpeningsOfEachGpuInThisMount`、`OpeningsInThisMountUsedUp` 两条腿读法不一（辩方：名字说预算、语义是连败数；攻方：到 4 之后不会再开成，「这一次挂载里用完了」是真话），不改名 |
| 检查单：不兼容旧代码与旧格式、不新建判定库、测试优先多线程 | 成立 | 两条腿各自的 grep 零命中；进程级那张表不是静态量，用例各建各的 |
| 检查单：所有任务多机可用 | 成立 | 辩方读了任务日志：15315 在第二台、15444 在本机都过；真卡对拍「本次未跑」的两处是三卡与两卡的 `checksum_batches_over_…_real_gpus` 目标，各只拿到 1 张卡，留给提交时那一趟 |
| 检查单：带 --item 的命令与项 | 不成立 | 137 个变异行名在 `crates/mutations.tsv` 第一列各恰好 1 次（辩方量的）；新分支里三处没有变异行锚着：`crates/singlefs-core/src/background_compute.rs:1787`、`crates/singlefs-mount-runtime/src/lib.rs:946` 与 `:1193`、`crates/singlefs-background-gpu/src/background_gpu.rs:1432`–`:1434` |
| 检查单：新崩溃流多罩了什么 | 推不出，按打中办 | 重复键放弃一批走的是已有的同一条只删意图的发布；「删意图之后紧接着再签」连着几轮这一串，已有崩溃流罩没罩两条腿都没核，归提交时的崩溃门禁那一趟，另记欠账 |
| 「验证到的数」 | 都核得出 | 辩方逐字对上任务日志。要照实写的两样：三趟变异行复跑「抓到 51、31、55」里现判各 1 条、其余是库里键相同的判定复用（按键复用的正常结果）；任务 15382 的 31 行里 6 行是「七·3」起头，正文称「C996 第一轮修补的变异行里剩下的」不准。15185 与 15315 两趟总数差 5 条的来源日志里没有，没核出 |

没打中的那几格只抽了一次样，照规矩不撑否定结论；这一轮是最后一轮，不再补抽。

## 三、改法

都是腿提的或主 agent 照用户定案推的，零轮。A/B 里没有改法的那一臂是任务 15497 的读数；实现员把模型用例收成回归用例，改完之后在同一批输入上读数翻面才算换到东西，任务号由合入后验证补在各条里。一个实现员一次改完；不碰 `crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs`（别的会话正在把它搬进测试包，第 6 条交给它）。

1. 三·甲：`gpus_opened_at_a_switch` 里算出这张卡剩 0 次时不断言，照试满当场记成它开不成（`OpeningsInThisMountUsedUp`）、不开；交设置那一刻与线程开头那一刻读的是同一个判据。回归用例取模型 12 段：`held_opening=4`、`switch_back_while_held=true` 的 2 段翻成 `poll_panicked_with=None`、答开不成、`openings_of_forty_one=4`。零轮。
2. 三·乙（用户 2026-11-13 定「等池乙开完再开」）：那张进程级的表只记到点之后还没回来的开卡；没到点、正在开的并发开卡，后到的一方在自己的开卡期限之内排队等前一方开完再开，不算一次开不成。回归用例取模型两臂：`arm=shared_table` 翻成 `PutInEffect`。壬原来那一条（到点之后挂住）照旧中。零轮。
3. 未交二：前一次开卡到点没回来（`EarlierOpeningOfTheDeviceNotReturned`）不记进连着开不成的次数，报警里照旧报这个原因与「出路只有重启进程」；它不会把这张卡判成试满。零轮；这是主 agent 照第 4 条两句出路推的，交用户表里列着。
4. 三·丙：三段都在交卡之前查重复键，重复就一片都不交、交回重复键；同一批里卡出错与重复键不再同时出现。回归用例取模型 `kernels=gpu_failed` 三行：不再出现。零轮。
5. 没有变异行罩着的三处新分支各补一行变异与用例；第 1 条改完之后 `background_gpu.rs:1432`–`:1434` 那一处随它一起罩。
6. e202 四处开卡各现建一张空的「没回来」表、没共用进程级那一张（`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:4266`、`:4280`、`:7873`、`:7975`，辩方量的）：采纳，由搬 e202 的那个会话在新位置重写取卡时一并做，这一批不碰。
7. 文字：`crates/singlefs-core/src/background_compute.rs:13`、`:745` 的「同日弹窗定」与第 213 行「意图留在盘上、任务停下」改成现状；`crates/singlefs-background-gpu/src/background_gpu.rs:3130` 把旧条款「照新上限重新量代价」当第 3 条原话引的那一句改成现状；`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:302` 与 `crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:2203` 两处被注释历史那一格命中的行改写。`crates/singlefs-core/src/mounted_session.rs:5099` 的函数文档与新四之后的现状不全合：那份文件别的会话在改，发给它。
8. 写回 kb：D26（后台整理与放置回收） 已定项 13 第 4 条照改法 1、2、3 补（线程开头再判一次试满；进程级那张表只记到点之后的、没到点的并发开卡排队等；没回来不记进连着的次数），并补四句代码做了而条款没写的（叫停没开的不记进次数；起不了开卡线程那一回记在头一张要开的卡上；冲掉待写的空提交等不到算一次超时、不算重交；开卡期间停着之后新卡答开不成算另一回停下、报警的两个数从头数——最后一句是用户 2026-11-13 定的「分开算」）；第 3 条补切换时对请回来的卡再量一次代价、量不成这张卡不参与并报出来（甲的改法落在条款上）；第 8 条补同一批别的段已交回的照算一轮、重复键每一批都出现时照旧不停不报警（用户 2026-11-13 定「维持现状」）。变更史记用户这四条定案的原话。欠账表加一条：重复键放弃一批之后紧接着再签意图连着几轮的落盘，崩溃流罩没罩没核。

## 交用户的

- 用户 2026-11-13 弹窗已定、这一轮照办的五条：试满的 4 次「连着数，开成清零」；重复键每一批都出现时「维持现状」（不设上限、不停、不报警）；开卡期间停着之后新卡答开不成，报警的两个数「分开算（现状）」；新的一组仍点着刚出错的那张卡、换上了「维持现状（照样解除）」；两个池同时开同一张卡，后到的「等池乙开完再开」。
- 主 agent 照已有定案推的、没单独问的（可以推翻）：前一次开卡到点没回来不记进连着开不成的次数（改法 3）；切换时对请回来的卡再量一次代价这一步留着、写进条款（辩方指出它是旧条款留下的，现行条款只写「重新参与分活」；不量的话每张参与的卡报不出代价，要另一种实现）。
- 知道一下：癸的读口（运行库读切换关卡的泄漏）仓里只有测试在读，产品代码没有调用方；它是给宿主程序读的。
- 知道一下：新类型的内值是公开的，取出内值再包成另一个类型编得过；混用只在函数签名处被类型拦住。
- 知道一下：三·甲在产品里的后果是后台维护线程在会话锁里 panic，与上一轮新一同一级；窗口是第 4 次开卡在跑的那一段（开卡到点的情形下至多 5 分钟），没在真驱动上量。

## 这一轮被判过的文件

- crates/singlefs-background-gpu/src/background_gpu.rs：切换开卡的次数、叫停、进程级那张表、量代价之前先等、三段的记账与那句换了算法的断言；三·甲、三·乙打中，改法 1、2、3、7
- crates/singlefs-background-gpu/src/background_gpu_compute.rs：五个原因成员与重复键到核心层的转法照写了；字段改名之后的构造点
- crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs：四处开卡各现建一张空表、不共用进程级那一张，改法 6 交搬它的会话
- crates/singlefs-background-gpu/src/checksum_kernels.rs：重复键先于比对、后于交卡；份数换类型；改法 4 把查重复挪到交卡之前
- crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs：开卡器持有进程级那张表、把切换关卡的泄漏接给运行库；改法 2 落点之一
- crates/singlefs-background-gpu/src/gpus_of_this_host.rs：接住 panic 挪进挂载与切换共用的开一张卡那一步，照写了
- crates/singlefs-background-gpu/src/lib.rs：与上一轮快照相同，导出随新类型核过
- crates/singlefs-background-gpu/src/live_slot_kernels.rs：选段打分那一段先查重复再断言整段清单（乙修掉了）；重复键压过 GPU 出错（三·丙），改法 4
- crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs：字节数、条数、输入长度的类型与算术，站住
- crates/singlefs-background-gpu/src/region_record_kernels.rs：占用计算换类型、`minus` 的两处上游挡住下溢，站住；改法 4
- crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs：先整批查重复、后动计数，站住
- crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs：冲掉待写的空提交等不到算进超时数、判它不算交到卡上（丙修掉了）
- crates/singlefs-core/src/background_compute.rs：开不成不记成生效、开卡期间出错停下与换上了解除、重复键只放弃这一批、`GpuMemoryInBytes` 的算术；三处注释改法 7、一处没有变异行改法 5
- crates/singlefs-core/src/compaction_plan.rs：第 631 到 650 行切换生效时刻的文档改成现状，照写了
- crates/singlefs-mount-runtime/src/lib.rs：读切换关卡泄漏的两处读口，照写了；两处没有变异行改法 5

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-13 改了：第 3、4、8 条照这一轮「三、改法」第 8 条补写，等改法 1 到 4 合入之后写回 |
