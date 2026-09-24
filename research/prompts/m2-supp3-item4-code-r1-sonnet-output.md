# 正推腿（Sonnet）判决：m2-supp3-item4-code-r1，问题 K4、K5

分工：本腿只判 K4（三条判定够不够）、K5（起点那一段不注入）。K1、K2、K3、K6 不判。

方法：读 `crates/` 今天的实现（`crates/singlefs-harness/src/fault_injection.rs`、`history.rs`、`model.rs`、`model_comparison.rs`；`crates/singlefs-core/src/transaction.rs`、`mount.rs`、`make_filesystem.rs`），标出「实现今天的样子」，与里程碑设想实现、`.claude/kb/decisions/13-验证路线.md`、`.claude/kb/invariants.md`、`.claude/kb/checks-owed.md` 的已定条款逐格比。所有代码行号、产物读数在写进本节之前都用 `grep -n` 或本次编译运行现取，不从背景材料数。

## K5 的设备调用计数：怎么量的

`HistoryPool::start`（`crates/singlefs-harness/src/history.rs:806-901`）没有暴露成可以从外部单独调用的公开函数，我另写了一个独立的一次性程序（不改工作区任何文件，草稿放 `/tmp/claude-1000/m2s3i4-r1-sonnet/count-start-calls/`，`Cargo.toml` 用 `path =` 依赖指回 `crates/singlefs-core`、`crates/singlefs-harness`、`crates/singlefs-checker`、`crates/singlefs-format`），逐字照抄 `HistoryPool::start` 的 `AfterFirstFile` 分支（与 `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs:326-355` 的 C381 用例、`crates/singlefs-harness/src/history.rs:832-895` 同一串调用），用同一个 `FaultInjectingBlockDevice` + `SharedFaultPlan::unarmed(...)` 包住两块内存盘，在 `make_filesystem`、`acquire_instance`、`warm_up`、`publish_first_file` 四步之后各打印一次 `plan.calls()`。

命令与原样输出（`cargo run --release`，两次独立重跑逐字节相同，见下）：

```
$ cd /tmp/claude-1000/m2s3i4-r1-sonnet/count-start-calls && cargo run --release
mkfs 之后: reads=0 writes=11 barriers=4 total=15
取号之后: reads=36 writes=13 barriers=6 total=55
暖机之后: reads=44 writes=23 barriers=12 total=79
第一个文件之后: reads=48 writes=44 barriers=16 total=108
first.root.checkpoint_txg = CheckpointTxg(3)
盘 0：reads=28 writes=23 written_bytes=251904 fua_writes=4 barriers_forwarded=8
盘 1：reads=20 writes=21 written_bytes=250880 fua_writes=2 barriers_forwarded=8
```

第二次独立重跑（同一个二进制，重新执行）：与上面逐字节相同（mkfs 15 / 取号 55 / 暖机 79 / 第一个文件 108，两盘计数同上），这是确定性回放（内存盘、SplitMix64 未涉及随机源，无并发），不算一次统计意义上的独立观测，只作「同一版代码同一个数」的复核。

逐步之差（各步单独新增的调用数）：
- `make_filesystem`：15 次（0 读 / 11 写 / 4 屏障）
- `acquire_instance`（取号）：+40 次（36 读 / 2 写 / 2 屏障）
- `warm_up`（暖机）：+24 次（8 读 / 10 写 / 6 屏障）
- `publish_first_file`（第一个文件）：+29 次（4 读 / 21 写 / 4 屏障）
- 起点段合计（`HistoryStartingPoint::AfterFirstFile` 分支）：**108 次**
- 起点段合计（`HistoryStartingPoint::AfterMakeFilesystem` 分支，只到 mkfs）：**15 次**

推翻条件：这份计数依赖我复制的调用序列与 `HistoryPool::start` 逐字相同；若有人指出 `history.rs:832-895` 与我 `main.rs` 里的某一步参数不同（比如 allocator 初始化、`FirstFile` 的 `write_time_seconds`），需要重新对照两段代码逐行核对，我这份数就要重量。

## K5：起点段今天由谁罩，不罩缺口有多大

**为什么这 108（或 15）次调用今天绝对进不了随机注入**：`crates/singlefs-harness/src/fault_injection.rs:1108-1156` 的 `draw_faults` 只在 `marks[1..]` 里选候选步（`for index in 1..marks.len()`，比较 `marks[index]` 与 `marks[index - 1]` 的调用计数），`marks[0]` 恒为 `StepPosition::StartingPoint` 那一份（`crates/singlefs-harness/src/history.rs:2844` `let position = Cell::new(StepPosition::StartingPoint)`；`fault_injection.rs:1190-1196` 测量跑把 `HistoryPool::start` 返回后的第一份 `StepMark` 推进 `marks`），因此起点段的调用**只能充当 `marks[0]` 这个基线，永远不会被选中做注入目标**。代码注释原文（`fault_injection.rs:1123`）：「起点那一段不摆：起点（mkfs、取号、暖机、第一个文件）在执行器里是 `expect`，注入在那里只会撞出执行器自己的 panic，不是 `singlefs-core` 的缺口」。

逐函数核「谁罩着」：

1. **`make_filesystem`（15 次调用）**：`crates/singlefs-core/src/make_filesystem.rs:57-82` 的 `MakeFilesystemError` 里确有 `BlockDevice(BlockDeviceError)` 成员（79 行），说明这条函数的设计上是**会**处理块设备错误并返回 `Err` 的。但 `grep -n "make_filesystem(&parameters" crates/*/tests/*.rs crates/*/src/*.rs` 命中的全部 9 处调用（`first_transaction_step_one_mkfs.rs:77,317,378`、`first_transaction_step_five_publish.rs:146`、`second_transaction_supplement_one_write_accounting.rs:239`、`second_transaction_supplement_two_unequal_devices.rs:110`、`second_transaction_supplement_two_warm_up_counter.rs:36`、`first_transaction_step_two_data_unit.rs:84`、`second_transaction_supplement_three_fault_injection.rs:326`）没有一处把 `make_filesystem` 包在带 `arm()` 的 `FaultInjectingBlockDevice` 里；`first_transaction_step_one_mkfs.rs:317,378` 那两处唯一测的 `Err` 分支是参数校验错误（`RegionDevicesNotTheFirstVersionLayout`、`JournalRingTooLargeForDevice`），不是设备 I/O 错误。**结论：`make_filesystem` 的 `BlockDevice(_)` 分支今天全仓零覆盖**——不只是随机注入路走不到，是没有任何一条测试、任何一种手段走到过。

2. **`acquire_instance`（取号，起点段这一次 40 次调用）**：这个函数本身**在别处被专门的手写故障注入测过**——`crates/singlefs-harness/tests/instance_acquisition.rs:128-153,155-175` 两条用例分别对同一个 `acquire_instance` 调用注入「每道屏障都报错」（`fail_every_barrier`）与「盘 1 每次系统配置槽写都报错」，钉住回卷行为（世代号不重发、盘 0/盘 1 各自的槽内容）。而且 `acquire_instance`（`transaction.rs:374-379`）与 `establish_instance`（`crates/singlefs-core/src/mount.rs:983-1024`，被每一次 `mount_writable`/`mount_rollback` 调用，也就是随机历史里的 `CloseAndMountWritable`/`CloseAndMountRollback` 操作）用的 `acquire_expected_instance`（`transaction.rs:398-413`）共用同一个写入口 `write_acquired_instance`（`transaction.rs:416`起）。随机历史生成器里 `CloseAndMountWritable` 权重 70/6/12/90/50/95/90（`history.rs:389,396,405,424,459,492,517`，各段权重表），这些操作是「步」，会进 `draw_faults` 的候选步——只要一段历史里抽中了这一种操作，取号那次写/屏障就可能被注入。**结论：这次调用（起点段那一次）本身没被随机注入或专门用例罩住，但同一份代码在随机历史的绝大多数段里都会在别处（第二、三……次取号）被注入到；缺口窄，是「这一次」不是「这个函数」。**

3. **`warm_up`（暖机，起点段这一次 +24 次调用）**：与 `acquire_instance` 不同，这是**全仓唯一**的缺口——`grep -n "warm_up(" crates/*/tests/*.rs crates/*/src/*.rs` 命中的 9 处（`first_transaction_step_five_publish.rs:173`、`scenario.rs:104`、`second_transaction_supplement_two_unequal_devices.rs:138`、`second_transaction_supplement_three_fault_injection.rs:343`、`history.rs:864`、`second_transaction_supplement_one_write_accounting.rs:258`、`second_transaction_step_three_formatted_pool.rs:231,345`）**全部**是 `.expect("暖机")`，逐一核对每处调用前的设备变量：`instance_acquisition.rs` 不调用 `warm_up`；`second_transaction_step_three_formatted_pool.rs:231,345` 两处用的是 `PoolWriter::new(&publish_parameters, open_devices.as_mut_slice())` 里的 `open_devices`，来自 `format_pool(...)` 帮助函数，不含 `FaultInjectingBlockDevice`；`second_transaction_supplement_one_write_accounting.rs:258` 与 `second_transaction_supplement_three_fault_injection.rs:343` 虽然设备类型是 `FaultInjectingBlockDevice`，但两处调用 `warm_up` 时计划都还没 `arm()`（前者第一次 `.arm()` 在 `publish_through_overwrite()` 建好的镜像之上，晚于 `warm_up`；后者 `plan.arm(...)` 在第 359 行，晚于第 343 行的 `warm_up`）。**production 里也没有第二条路径调用它**：`mount.rs` 里可写挂载与回退挂载的暖机发布走的是 `publish_empty_after`（`mount.rs:762-805`），它与 `warm_up_after_journal_counter`（`transaction.rs:473-499`）是两个不同的函数——后者的文档注释原文（`transaction.rs:469`）「今天只有 `warm_up` 调它，环是空的、两个量按构造相等」，明确说这条函数只服务于 mkfs 之后的第一次暖机这一种起点态。**结论：`warm_up`/`warm_up_after_journal_counter` 这两个函数的错误返回路径（`BlockDeviceError` 原样交回，`transaction.rs:451`「块设备报的错原样交回」）今天在全仓任何测试、任何注入手段下都**从未被触发过一次**——比「这一次不罩」更严重：是「这个函数从不被注入」。

4. **`publish_first_file`（第一个文件，起点段这一次 +29 次调用）**：与 `acquire_instance` 同类——同一个函数在随机历史里作为 `HistoryOperationKind::PublishFirstFile` 操作（`history.rs:2070-2110` 的 `apply_publish_first_file` 在 2094 行调用同一个 `publish_first_file`；权重表 `with_session_open_without_file` 里占 60/95（`history.rs:394,492`）等），是候选步、会被 `draw_faults` 选中。**缺口窄，是「这一次」不是「这个函数」。**

**汇总**：108 次调用里，40（取号）+ 29（第一个文件）= 69 次属于「同一份代码在随机历史别处会被注入到，只是这一次不会」；15（mkfs）+ 24（暖机）= 39 次属于「这份代码今天在全仓任何地方都没被故障注入测过」，这 39 次里又以 `warm_up` 的 24 次最直接——它甚至没有一条手写用例像 `instance_acquisition.rs` 那样单独盯着它的失败路径。

**C378 是不是同一处、今天开着还是关着**：`.claude/kb/checks-owed.md:333` 现查（`grep -n "^| C378" .claude/kb/checks-owed.md`）原文：「还开着的是取号之后因块设备报错失败的那一路：写行或暖机那几次写报错时代号已经烧了，回卷只管取号自己那几次写（`mount.rs` 第 849 行注释）」。核实它今天在「已还清」表内外：`grep -n "^### 已还清" .claude/kb/checks-owed.md` 命中 416 行，`awk '/^### 已还清/{f=1} f' .claude/kb/checks-owed.md | grep -n "C378"` 零命中——C378 只出现在 333 行（已还清表之前的主表），**今天仍是开着的**。

这与 K5 问的**不是同一处代码，但是同一类损伤**：C378 说的「写行或暖机那几次写报错」指 `mount.rs:983-1105` 的 `establish_instance`（每次 `CloseAndMountWritable`/`CloseAndMountRollback` 都会走到，走的是 `publish_empty_after`，`mount.rs:762-805`），不是 `history.rs` 起点段直接调的 `warm_up`（`transaction.rs:452`）——两个函数不同，我在上面第 3 点已核实。**但两者共享同一个「取号之后再失败，代号不回卷」的行为**（`mount.rs:849` 注释：`acquire_expected_instance` 失败会回卷，但它之后的写行/暖机发布失败不回卷；`history.rs` 起点段的 `acquire_instance` 同理只在自己那步失败时回卷，之后 `warm_up`/`publish_first_file` 失败同样不回卷）。区别在于：C378 描述的那条路径（`establish_instance` 里的写行/暖机）**理论上够得着随机注入**（它是 `CloseAndMountWritable` 这个「步」的一部分，`draw_faults` 选得到），但**判定这一格今天没有任何断言**——checks-owed 那一行的第 4 列（怎么拦）写的是「造……历史，按用户定的那一边断言」，第 5 列（前置）写「用户定「回卷还是认了」」，说明**断言还没写**，用户还没在「回卷」与「认了」之间选。也就是说，即便随机注入的种子恰好把故障砸在某次后续 `CloseAndMountWritable` 的写行/暖机步骤上，触发了「代号已烧、不回卷」，`model_comparison.rs` 里这类块设备错误一律映射到 `ObservedRefusalReason::Unexplained`（`model_comparison.rs:162,177,201,243`，现查命中 4 处），执行器只会把它当成「模型没料到、这次失败到此为止」处理并归入 `SurfacedAsAnError`（详见下面 K4 部分），**不会去检查代号回没回卷**——今天没有专门的断言像 `instance_acquisition.rs` 那样盯着「取号之后再失败，系统配置槽的实例代号该是什么」。所以 C378 与 K5 的关系是：**K5 问的起点段是「进不去」，C378 问的是同一类损伤在「进得去」的地方「进去了也没人判」——两处都开着，缺口的性质不同（前者是抽样射程缺口，后者是判据缺口），互不覆盖对方。**

推翻条件：若有人证明 `mount.rs:849` 那条注释描述的回卷范围其实也覆盖了写行/暖机失败（即代号确实会回卷），则 C378 这半的现状陈述本身错了，需要重新读 `mount.rs` 的 `establish_instance` 找回卷调用点；我现查过 `establish_instance`（`mount.rs:983-1105`）里只有 `acquire_expected_instance`（1011 行）一次调用带着显式的 `ExpectedInstanceAcquisitionFailed::Acquisition` 处理，之后的 `publish_rows_on_file_version`/`publish_without_units`/`publish_empty_after` 报错都直接 `?` 向上传播（1032-1044、1053-1066、1085 行），没有看到任何针对已取号但后续发布失败的回卷调用，与 checks-owed 的现状陈述一致。

## K4：三条判定够不够

**三条判定字面是什么**：`fault_injection.rs:11-15`（模块文档注释）：① 「整段历史里一个 panic 都没有」；② 「注入那一步的结局是「入口返回 Err」或「冷启动读回报错」这两种形态之一」；③ 「重开（拿录制流重建镜像、跑 `recovery::recover`）走到的那一版在模型允许的集合里，读回的内容与模型记的逐字节相同」。

**这三条实际怎么算出来**（`inject_one_fault`，`fault_injection.rs:1268-1465`）：
- ①（无 panic）在 `history.rs:2846` 的 `with_panic_capture` 里判，`fault_injection.rs:1426-1429` 把 `observation.panic.is_some()` 计进 `faults_that_panicked`。
- ②（surfaced）由 `the_fault_surfaced_as_error`（`fault_injection.rs:1240-1265`）判：要求 `observation.position == StepPosition::Operation(step_index)` 且无 panic、无 checker 违例，且 `run.outcomes.last()` 是 `Refused` 或 `Recovered{ read_back: Failed }`。
- ③（reopen 在模型允许集合里）由 `crash_recovery_disagreement`（`crates/singlefs-harness/src/model.rs:1756` 起）判：只比对「择到的根是不是模型提交过的某一版」「读回内容与模型记的那一版逐字节相同」两件事（`model.rs:1772-1799` 起，逐分支核对：`chosen < persisted` 判红、`committed_versions.get(&chosen)` 找不到判红、内容 `None`/`Some` 不对等判红）。

**逐条核合起来能推出什么**：三条合起来证明的是——「这一次注入没有让 `singlefs-core` 崩溃、注入那一步的调用方确实收到了一个可处理的错误、而且事后冷启动能选中一条模型认可的根、读到与模型记的完全相同的内容」。这证明的是**这一次故障之后系统仍然可以正常冷启动到一个合法版本**，不证明**这次故障发生期间与发生之后，整个镜像的其余结构仍然自洽**。实际实现里还叠了第四道检查（`checker_violations_on`，`fault_injection.rs:1400,1410-1425`）把池级 checker 的违例也塞进 `failures`——但背景材料与模块文档只数「三样」，checker 那一道没有被列进「三样」里，是文档口径本身漏记了一道正在生效的检查（这个漏记不改变下面两类反例，因为下面两类反例连 checker 那道也过得了）。

**反例一（举得出、可复算）：checker 自己的实现只判了 69 条不变量里的 29 条**。`.claude/kb/invariants.md:12` 现查原文：「池级 checker（`walk::check_pool_image`）判 29 条……状态列写「已实现」的就是这 29 条」；`invariants.md:17`：「现共 69 条在用……」。也就是说三样（含隐藏的 checker 那道）里，checker 判绿只能担保「这 29 条不违反」，另外 40 条（例如 I-4 类里除 I-4.8 外全部「未实现」——`I-4.1`「重放后树可遍历」、`I-4.2`「无被引用未提交块」、`I-4.3`「提交原子」、`I-4.4`、`I-4.6`、`I-4.7` 均标「未实现」；I-3.4、I-3.6、I-3.7 同样「未实现」；I-7.3、I-7.5 同样「未实现」）不管镜像上实际发生了什么，checker 都不会报违例。这正是 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「文件系统特有的反推缺口」里点名的那条：「「checker 没报错」不等于「镜像是好的」」——某次注入若恰好只破坏了 40 条未实现不变量之一描述的性质（比如让某个块被引用却没提交完整，对应 I-4.2），三条判定与那道隐藏的 checker 检查会**全部通过**（无 panic、注入那步确实报错或冷启动确实报错、reopen 选中的根与内容与模型一致、checker 29 条全绿），而实际上违反了一条已经写进 `invariants.md` 但还没实现检查的不变量。

推翻条件：若有人证明这 40 条「未实现」的不变量里，凡是能被「一次设备错误 + 冷启动恢复」触发的场景其实都会连带违反某条已实现的 29 条之一（即两个集合在故障注入能触达的范围内其实重合），这条反例就不成立；我现查 `invariants.md:487-494`（I-4 类表格）与 `I-3.4/I-3.6/I-3.7`（`invariants.md:456-459`）没有看到这类交叉断言，且各条「未实现」旁边没有「已被 I-x.y 间接覆盖」的说明，所以今天看不出重合。

**反例二（结构性的，来自代码本身的控制流）：一次注入之后历史不会再往下跑，导致的损伤要等下一次操作才现形**。`model_comparison.rs` 把块设备错误一律映射到 `ObservedRefusalReason::Unexplained`（现查命中 `model_comparison.rs:162,177,201,243` 四处），而 `model.rs:1399-1402` 对 `Unexplained` 一律判 `accepted = None`，`model.rs` 紧接着把它变成 `Err(ModelDisagreement)`；`history.rs:2896` 把它接成 `model_disagreement`，`history.rs:2935`（`if !violations.is_empty() || harness_judgement.is_some() || model_disagreement.is_some() { ... return Some(observation); }`）在**这一步就让整条历史提前结束**（不会再执行 `history.operations` 里排在它后面的操作）。`fault_injection.rs:1240-1265` 的 `the_fault_surfaced_as_error` 恰好把这种「因为模型没见过设备错、判成分歧、历史提前结束」的形态识别成「surfaced」，`fault_injection.rs:1405-1409`（`if !surfaced { failures.push(...) }`）据此把它从失败列表里剔除——也就是说，**这一次注入所在的历史，注定不会有任何一步排在故障之后**：三条判定（含 checker 那一道）永远只能检查「故障发生那一刻起、到 `execute_history_with_faults` 返回为止」这一个静态镜像，检查不到「如果这次失败之后调用方又发起下一次写，会不会因为分配器/实例代号/回退下界这类内存态没被正确回滚而写坏东西」。这正是 `.claude/kb/checks-owed.md:333`（C378）今天开着的那一类：「写行或暖机那几次写报错时代号已经烧了」——即便随机历史后续段落真的抽中了「取号成功、后续写行/暖机写失败」这个组合（技术上够得着，因为它是某个 `CloseAndMountWritable` 步骤内部的调用），三条判定加 checker 也不会去看「代号回没回卷」，因为历史在那一步就已经按「surfaced，不算失败」结束，压根不会有后续写操作把「代号没回卷」这件事写坏、暴露出来。

推翻条件：若未来把 `execute_history_with_faults` 改成「注入触发的 `Unexplained` 分歧不再提前结束历史，而是把这一步标记为「合法失败」后继续跑后面的步骤」，上面这条反例就会失效（历史会继续跑，后续操作有机会把潜藏的损伤写成能被 checker 或 `crash_recovery_disagreement` 抓到的形态）；今天的代码没有这个继续逻辑，`history.rs:2935` 那一行只在 `per_step_checker == RunContinuingPastTheRingTurnForm` 且命中「I-3.1 记账多算、根环转过」这一个特定形态时才不提前结束（`history.rs:2950-2957`），对模型分歧没有这个例外。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K5：起点段（mkfs、取号、暖机、第一个文件）今天调用几次设备 | 已量（命令数出） | `AfterFirstFile` 分支 108 次（mkfs 15 / 取号 40 / 暖机 24 / 第一个文件 29），`AfterMakeFilesystem` 分支 15 次；`draw_faults` 结构性排除 `marks[0]` 那一份基线，永远选不到 |
| K5：这些调用今天由谁罩 | 冲突（逐函数不同） | mkfs（15 次）与 warm_up（24 次）今天全仓零故障覆盖；取号（40 次）与第一个文件（29 次）这个函数在随机历史别处会被注入到，只是起点这一次不会 |
| K5：不罩的缺口有多大 | 已判（39/108 属于「函数级」缺口） | mkfs 的 `MakeFilesystemError::BlockDevice(_)` 分支与 `warm_up`/`warm_up_after_journal_counter` 的错误路径今天没有任何测试、任何手段触发过 |
| K5 与 C378 是不是同一处 | 冲突（同类不同处，都开着） | C378 说的是 `establish_instance`（`mount.rs`）里写行/暖机失败不回卷，够得着随机注入但没断言；K5 说的是 `history.rs` 起点段直接调 `warm_up`，注入进不去；两处都开着，缺口性质不同 |
| K4：三条判定合起来能不能推出「验到了它该验的东西」 | 冲突（部分能，部分不能） | 能证明「这次注入没让实现崩溃、错误被正确处理、冷启动能选中模型认可的根且内容一致」；不能证明「镜像其余结构仍自洽」「这次失败不会在后续操作里现形」 |
| K4：有没有一类失败三样全过而实际是错的 | 已举出两类，均可复算 | ① checker 只实现 69 条不变量里的 29 条，损伤若只违反未实现的 40 条之一，三样全过；② 注入触发的分歧一律让历史提前结束（`history.rs:2935`），「这次失败污染了内存态、下次操作才现形」这类损伤没有下一步可看 |

## 没做什么

- 不判 K1、K2、K3、K6（分给云端攻方与本地腿）。
- 没有重新跑 `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs` 的快档（16.2 秒那一档）——K4、K5 都是对代码结构与既有产物读数的核验，不需要重新触发随机注入；K5 的设备调用计数改用独立复刻程序现算，已在上面给出命令与原样输出。
- 反例二里「继续跑历史会不会把潜藏损伤暴露出来」没有做实验验证（那是一次改动 + 重新证明会红的活，归实现员/下一轮），本报告只证明「今天的代码结构上做不到」。
- 没有判断 C381（收口表第 40 行）本身该怎么改，只在 K5 需要区分「同一处」时读了 C378，没有连带判 C381。
