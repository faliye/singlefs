# 转述核对表：m2-supp3-item4-code-r1-local-attack（2026-09-21）

逐句核对 `research/prompts/m2-supp3-item4-code-r1-local-attack-k3.md`（K3：设备说谎那一类该不该进）与
`research/prompts/m2-supp3-item4-code-r1-local-attack-k6.md`（K6：收口表第 40 行那一格「只记不判」）
里每一条 fact 与中文/源码原文。kb 与 crates 的行号现查各自文件（工作区 2026-09-21 版本，`git status`
显示这些文件都是未提交的工作区改动，不是 HEAD 版本）；body 的行号按 Read 工具显示的
`research/prompts/_m2-supp3-item4-code-r1-body.md` 自己的行号。
格式：英文项（fact 编号）/ 原文文件:行 / 首稿缺的或改动的 / 定稿理由。

提示两份都不引用具体的文件名加行号（按共用约束「英文提示里的每一句转述」一节与主 agent
派发提示「提示里明令答复不写代码行号与文件行号」），只在这份核对表里写死来源文件与行号。

## K3（设备说谎那一类该不该进）

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| Fact 1: 故障注入逐字要求 | `.claude/kb/milestone/02-second-txn.md:429`（「故障注入：一个通用的设备包装，按种子让任意一次写、读、刷盘返回`BlockDeviceError`，或让一次读返回改坏的字节；实现要返回错误而不是panic，出错之后重开要恢复到模型允许的版本。各用例里手写的包装并进来。」） | 首稿把「重开」译成"a fresh remount"；「重开」在这个代码库里指冷启动整池重开（`recovery::recover`），与「mount」（`mount_writable` / `mount_rollback`）是两个不同的、这个代码库自己命名的操作，用"remount"会把两者混同 | 改成"a fresh cold reopen"，K3、K6 两份都统一用这个说法 |
| Fact 2: 六种故障的文档注释 | `crates/singlefs-harness/src/fault_injection.rs:110-123`（`WriteFails`「这一次写报块设备错，一个字节都不落盘」`WriteIsSwallowed`「这一次写报成功、其实一个字节都没落盘（写进了设备缓存又掉电的那一形）」`ReadFails`「这一次读报块设备错，缓冲区不动」`ReadReturnsCorruptedBytes`「这一次读报成功，读回的字节里翻掉一位」`BarrierFails`「这一次刷盘报块设备错」`BarrierIsSwallowed`「这一次刷盘报成功、其实没发给设备（漏一道屏障；虚机档拿它量『少一道屏障多出哪些崩溃状态』）」） | 有意不译 `BarrierIsSwallowed` 注释里「虚机档拿它量『少一道屏障多出哪些崩溃状态』」这半句到 Fact 2；这半句挪到 Fact 5（与 `WriteIsSwallowed` 那半句「丢一次写要配崩溃注入一起用」并列），避免同一句话在两条 fact 里出现两次 | Fact 2 只留六种各自「报什么、盘上发生了什么」的定义句；用途说明集中在 Fact 5 |
| Fact 3: 六种故障对底层设备的真实效果 | `crates/singlefs-harness/src/fault_injection.rs:610-698`（`write_at`/`barrier`/`read_at` 三个函数体，直接观察源码，非某一句中文的翻译） | 无中文原句可核对；核对方式是逐行读代码确认：`WriteFails`（:648）与 `BarrierFails`（:673）在返回错误之前不调用 `self.inner` 对应函数；`ReadReturnsCorruptedBytes`（:615-621）先调 `self.inner.read_at` 取真字节、只在调用方的 buffer 里翻位，不写回设备；`WriteIsSwallowed`（:650）、`BarrierIsSwallowed`（:674-677）同样不调用 `self.inner` 对应函数 | 按代码原样转述，不是翻译；已现查每一行 |
| Fact 4: 录制流恒等于真正落盘的那一串 | `crates/singlefs-harness/src/history.rs`（工作区改动，diff 见 `research/prompts/_m2-supp3-item4-code-r1-diff.md:424-427`：「故障注入（第4件）在**录制器外面**包了一层`crate::fault_injection::FaultInjectingBlockDevice`，注入计划由调用方给（`execute_history_with_faults`），不注入时它只数调用、每一次都原样交给录制器——报错的写、被吞掉的写与被吞掉的屏障都不进录制流，录制流因此恒等于真正落到盘上的那一串。」） | 首稿只译后半句（报错的写/被吞掉的写/被吞掉的屏障不进录制流 ⇒ 录制流恒等于真正落盘的那一串），不译前半句「在录制器外面包了一层…注入计划由调用方给」——这半句是模块接线方式，与 K3 的判断（该不该把说谎的设备纳入随机抽样）无关 | 只留持久性等价这一句的因果关系 |
| Fact 5: 随机抽样只抽四种 + 排除理由 + 保留理由 | `crates/singlefs-harness/src/fault_injection.rs:1082-1106`（`draw_injected_fault` 函数体与文档注释：「随机注入抽的就是里程碑…那几种…四种等概率」「`WriteIsSwallowed`与`BarrierIsSwallowed`不抽：它们是设备说谎…不是『返回错误』」「一次被吞掉的写在盘上无声地丢一份内容，任何文件系统都发现不了（写入口拿到的是Ok，校验和是它自己算的，读回来才看得见）——拿它判『实现要返回错误而不是panic』是在判一件实现做不到的事」「包装里留着这两种：漏一道屏障是真设备那一路量『少一道屏障多出哪些崩溃状态』的手段…丢一次写要配崩溃注入一起用（增补3第3件的枚举域）」） | 首稿把「丢一次写要配崩溃注入一起用」译完后自己加了一句"not yet connected"（尚未接上），这是本稿从 Fact 10 的 grep 结果反推出来的判断，原文这句话本身没有这层意思——发现后删掉，改稿只留原文字面的「意图配对」，不掺入 Fact 10 才有的独立观测，避免在给出 Fact 10 之前先泄露它的结论 | 删掉"not yet connected"，只保留"stated as intended to be paired with a separate crash injection mechanism" |
| Fact 6: 实测的量化声称 | `crates/singlefs-harness/src/fault_injection.rs:1089-1090`（Fact 5 同一段注释内：「实测：在快档上抽`WriteIsSwallowed`，24段里有2段的池级checker判出I-2.1/I-4.8/I-7.4红（根指着一份从没落盘的单元），与这一条判断相符」），与正文 `research/prompts/_m2-supp3-item4-code-r1-body.md:27` 同一数字复述一次 | 无遗漏，按字面译；从 Fact 5 单独拆出来是因为它是一条独立的可核实测量声称，不是排除理由本身 | 按字面译 |
| Fact 7: I-2.1 / I-4.8 / I-7.4 定义 | `.claude/kb/invariants.md:110`（I-2.1「任一被引用的块，其校验和与内容匹配」）、`:157`（I-4.8，见下）、`:53`（I-7.4，见下） | I-4.8、I-7.4 原文各自还有大段关于回退候选集精确定义（`D23`已定项14、`D16`已定项1、`F_生效`怎么跨盘算）、K代术语的历史沿革（2026-09-13起）、`C113`定案P3的候选集判据等——这些字段对 K3 的判断（2/24段判红是不是一个真信号）不改变结论，首稿即未纳入；I-4.8 保留了「判别力」那句（抓『本事务内释放的块被重新分配并写入』），因为它直接说明这条不变量测的是哪一类破坏；I-7.4 只保留谓词本身（块的物理范围不许被重新分配或抹头），略去「K代」术语说明与`D22`已定项14的鸽笼式下界解释 | 保留判据本身与 I-4.8 的判别力说明，略去候选集精确定义与术语沿革；这一处压缩已在此处登记，未逐字抄录全文 |
| Fact 8: 里程碑验收标准（故障注入这一条） | `.claude/kb/milestone/02-second-txn.md:446`（「故障注入：每个发布步骤与挂载步骤上各注入过至少一次，报每个注入点的命中次数，没命中的逐个列名；判别力：增补2收口表第40行那一格修之前判红、修之后转绿」） | 「增补2收口表第40行那一格」在 K3 里改写成「a specific known defect, tracked elsewhere and referenced only by its tracking name」，不点名 C381、不提「第40行」这个编号——K3 与 K6 是两份独立文档，C381 的具体内容只在 K6 的 Fact 8、Fact 9 里详细给出，K3 不需要也不应该带出 C381 的细节，避免同一轮两份文档互相牵连 | 只译「验收标准要求某个已登记的缺陷在修之前判红、修之后转绿」这一层结构，不展开是哪个缺陷 |
| Fact 9: 两个孤立单元测试 | `crates/singlefs-harness/src/fault_injection.rs:1925-1944`（`a_swallowed_write_reports_success_and_changes_nothing_on_the_device`：断言读回原字节、`plan.counts_of_device(...).writes`为0）、`:1946-1959`（`a_swallowed_barrier_reports_success_and_is_counted_apart`：断言`barriers_swallowed`加1、`barriers_forwarded`不变） | 无遗漏；直接引用两个测试函数各自的断言内容，不译测试名本身（测试名本身是标识符，按共用约束不作为「引用」处理，只描述断言在测什么） | 按断言内容转述 |
| Fact 10: 全仓搜索结果 | 现查命令（非某一句原文的翻译）：`grep -rn "WriteIsSwallowed\|BarrierIsSwallowed" crates/singlefs-harness/src/crash_injection.rs crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs` 零命中；`grep -rln "WriteIsSwallowed\|BarrierIsSwallowed" crates/ --include=*.rs` 只命中 `crates/singlefs-harness/src/fault_injection.rs` 与 `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`；后一份文件里只有 `BarrierIsSwallowed`（`swallow_the_next_barrier_on` 函数，diff 第157-165行），未使用 `WriteIsSwallowed`，且该二进制的故障是在固定脚本的一个固定点手动 `plan.arm(...)`，不经随机抽样 | 无中文原句可核；已现查两条 grep 命令与输出（本报告下方「命令与输出」一节留痕） | 如实转述两条命令的结果 |

## K6（收口表第 40 行那一格「只记不判」）

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| Fact 1: 故障注入逐字要求 | 同 K3 Fact 1，`.claude/kb/milestone/02-second-txn.md:429` | 同 K3 Fact 1 的「remount→reopen」修正 | 同 K3 |
| Fact 2: 三种归类的文档注释 | `crates/singlefs-harness/src/fault_injection.rs:776-785`（`CommittedByTheModel`「走到的是模型认下来的某一版（注入那一段历史跑完时模型根环里的那几条，`HistoryRun::committed_versions_at_the_end`）」`TheVersionTheFaultedStepWasWriting`「模型没认这一版，但它正是不注入时失败那一步会写出的那一版（增补2收口表第40行/C381在争的那一格；这里只记不判）」`OutsideEverythingTheModelAllows`「两样都不是：`crash_recovery_disagreement`判出对不上」） | 「增补2收口表第40行/C381在争的那一格」译成"a separately tracked known defect is arguing over"，不点名编号——编号本身在这份文档里不重要，Fact 8/9 会把 C381 的实质内容完整给出 | 隐去编号，保留「这正是一个已登记缺陷在争的那一格」这层事实 |
| Fact 3: 计数字段与唯一的断言 | `crates/singlefs-harness/src/fault_injection.rs:860-882`（`FaultInjectionTally`结构体字段与注释，尤其880行「重开走到了失败那一步正在写、模型没提交的那一版（收口表第40行那一格；只记不判）」）；`crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs:176-181`（`assert_eq!(tally.reopens_outside_everything_the_model_allows, 0, ...)`） | 首稿加了一句「No assertion anywhere in that same test file places any bound at all on the count for TheVersionTheFaultedStepWasWriting」——这是从`grep -n "reopened_into_the_version_the_faulted_step_was_writing" crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs`零命中反推出的观测，不是某一句原文的翻译，核对时确认这条 grep 确实零命中（退出码1）后予以保留 | 保留，并在本报告「命令与输出」一节留痕这条 grep 的原样结果 |
| Fact 4: inject_one_fault 的评分逻辑 | `crates/singlefs-harness/src/fault_injection.rs:1268-1425`（函数体，直接观察源码，非翻译） | 无中文原句可核对；已逐行确认：:1371-1379构造`allowed`（`committed_by_the_model`并上`marks[index].committed_versions`）、:1380对`allowed`调一次`crash_recovery_disagreement`（此结果控制:1410是否进`failures`）、:1382对`committed_by_the_model`单独再调一次（只用于:1381-1399选三个标签之一，不影响`failures`）、:1400-1425 checker 的 violations 独立于以上两次调用、无条件并入`failures` | 按代码原样转述，不是翻译；已现查每一行 |
| Fact 5: crash_recovery_disagreement 的四条判据 | `crates/singlefs-harness/src/model.rs:1742-1754`（文档注释）与:1756-1784起的函数体（①走读不许失败②择到的根必须是模型提交过的某一版③按`(txg,实例)`不许比已知最新持久根旧④内容逐字节相同） | 首稿漏译④后半句「那一版树表0条时只许报没有文件」这个特例；核对时发现原文④有这半句，判断它对 K6 的三道判断不改变结论（不影响 Fact 4 的机制描述本身），予以保留不译，在此登记 | 未译入 Fact 5，登记为已知的、有意的省略 |
| Fact 6: C381 判别力用例 | `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs:292-298`（用例上方注释：「发布在最后一步（系统配置槽）失败时根已FUA落盘、分配器照样退回，同一个写入口拿同一个上一版再发一次，就把那条根指着的单元原地盖掉」「C381一旦按哪条候选改掉…这条用例会转绿在别处、这里的断言会红——那时候连同『已知红』的账一起改，不许把断言改松」）、:300（函数名）、:435-444（断言：`violations.contains(&"I-7.4") && violations.contains(&"I-7.2")`、`recovery.outcome`为`Failed`） | 无遗漏，按字面译；未译函数名本身（标识符），只译注释与断言内容 | 按字面译 |
| Fact 7: 快档一轮的实测计数 | `research/prompts/_m2-supp3-item4-code-r1-body.md:18`（「快档一轮（24段×20步×每段4次=95个注入点…）：panic0次、新发现0条、重开走到模型都不允许的版本0次…重开95次里走到模型认下的某一版77、走到失败那一步正在写的那一版18…」） | 无遗漏；「新发现0条」与「模型都不允许的版本0次」两句合并译成「0 panics, and 0 failures outside a separately kept list of already known, already accepted failures」，其中「已知的、已接受的失败清单」这个说法是本稿对「已知红清单」的翻译选择，与 Fact 6 注释里「已知红」呼应，未改变数字本身 | 数字原样保留（95/77/18/0/0），「已知红」译成"already known, already accepted failures"并在 Fact 6 里用同一措辞 |
| Fact 8: C381 缺陷登记 | `.claude/kb/checks-owed.md:336`（「发布的持久顺序是根记录FUA之后再转系统配置槽；系统配置槽那一步失败时根已落盘，恢复会选中它，而发布路径把分配器整个退回到这次发布之前…」「第二、三轮都判完了（2026-09-21）…改法待用户定」） | 无遗漏；「三轮三方判完，改法待用户定」译成「three separate rounds of independent review have now finished…the actual fix itself is still awaiting a decision from the project's own human owner」，按字面对应 | 按字面译 |
| Fact 9: 候选甲的内容与状态 | `research/prompts/c381-r3-main-verification.md:37`（「候选甲（第2题：不立条款，明写『fsync报错之后数据可以出现也可以不出现』）——站住，但要加两样」）、:19（POSIX缺「不许假设可以照旧复用同一份内存态重试」这条禁止性条款）、:42（K4-e：候选甲那句语义只在「转只读」与「重做又失败」那几格上生效）、:61（「甲本身攻过一轮…加的那两样零轮」） | 首稿把「候选甲」译成"candidate A"（无实质改动，只是把中文的「甲」换成英文序数代号，避免用中文字符）；「两样」拆成两句分别译（禁止性条款、射程限定），未合并成一句，保持原文列出两条的结构 | "candidate A"对应「候选甲」；两处改动各自成句，与原文两点式结构一致 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| K3 Fact 3 整条 | `crates/singlefs-harness/src/fault_injection.rs:610-698` | 这整条 Fact 本身是本稿从代码控制流归纳出来的陈述，不对应任何一句中文注释 | K3 的判断（该不该把说谎的设备纳入随机抽样）依赖「四种已抽的故障是否都不构成『设备真的什么都没做却报成功』」这一事实，而这一点在中文注释里没有被显式写出来，只能从代码本身读出；不给这条陈述，模型没有依据去判断四种已抽的故障是不是都还给调用方一个诚实的信号 |
| K3 Judgment 1/2/3、K6 Judgment 1/2/3 全部三条 | `research/prompts/_m2-supp3-item4-code-r1-body.md:27`（K3）、`:33`（K6） | 三道判断本身是本地攻方腿按主 agent 派发提示「分工」一节给的问题框架拆出来的任务说明，不是某一句中文的逐字翻译 | 正文原句「这是不是一个真缺陷、该不该立『说谎的设备』这一类故障模型、不立的话这一格今天谁在罩」（K3）与「放行这个读法会让哪几类真错误也被放行；C381的改法一旦定下来，这一格要改成什么、快档会不会立刻红」（K6）各自天然拆成三问，本稿按这三问的顺序原样立三道 Judgment，未增删问题数量 |
| K6 Fact 3："that count is printed in a report but nothing today requires it to be any particular value" | `crates/singlefs-harness/src/fault_injection.rs:1010-1014`（`render()`函数把这个计数打进报告字符串） | 「打进报告」这件事原文（880行注释）没有提，是本稿额外确认的一个事实 | 这条事实用来支撑「这个数只被记下来、没有任何断言约束它」这个判断的完整性——只说「没有断言」还不够具体，读者可能以为这个数字根本没被算出来或没被看到；补上「它确实被打印在报告里，只是没有断言」更准确地描述了「只记不判」的字面含义 |

## 命令与输出（本报告依据的现查，非某句原文的翻译，留痕核验）

K3 Fact 10 依据：
```
$ grep -rn "WriteIsSwallowed\|BarrierIsSwallowed" crates/singlefs-harness/src/crash_injection.rs crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs
（零命中，退出码 1）
$ grep -rln "WriteIsSwallowed\|BarrierIsSwallowed" crates/ --include=*.rs
crates/singlefs-harness/src/bin/first_transaction_on_device.rs
crates/singlefs-harness/src/fault_injection.rs
```

K6 Fact 3 依据：
```
$ grep -n "reopened_into_the_version_the_faulted_step_was_writing" crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs
（零命中，退出码 1）
```

## 历史版本

（暂无历史，这是本轮第一次交这两份提示。）
