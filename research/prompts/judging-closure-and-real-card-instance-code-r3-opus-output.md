# judging-closure-and-real-card-instance-code-r3：攻方腿报告（2026-11-19）

攻的是第二轮判决（`research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md`）「三、改法」合入之后的修补，正文「要腿判的」第 1 到 4 问与第 6 问的八格。没写计数模型，没有 `rerun.sh`；要跑的都是副本上的探针，在末尾「要主 agent 跑的命令」。

读的范围（主工作区此刻那一版）：正文「范围」列的五份文件与 `crates/mutations.tsv` 里这一批的 14 行；它们的 sha256 与开工快照 `research/prompts/judging-closure-and-real-card-instance-code-r3-snapshot/crates-sha256.txt` 逐份相同（`sha256sum` 现算，六份都对上）。另读了被守的 `crates/singlefs-verification/src/gpu_unit_checks.rs`（三个打行单元与 `GpuCard` 定义）、`gpu_memory_ledger.rs`（`is_lost`、`books_locked`）、`crash_library.rs`（一次性库那几段与 `judge_tables_through_the_pairing_library`）、`scripts/mutation/crates-mutation-rows.py` 与 `scripts/mutation/prove-red.sh` 起 cargo 的那几段、`task-server/task_server.py` 的 `gpu_not_run_test`、三份实现员报告、调度记录第 390、399 行。第二轮、第一轮的判决读了全文；第二轮攻方腿报告只读了探针命令那几行。禁读的另一条腿的报告没读。

## 各格判定一览

| 格 | 判 | 一句话 | 依据 |
|---|---|---|---|
| 第 1 问 白名单 37 样 | 不成立（打中） | 名单里的 `gpu_work_trace(`、`gpu_lifecycle_line(` 不看是谁在调，只用名单里的写法就写得出两个登记单元互调：判器所在进程栈溢出倒下，键不变 | 读码（推的）；探针 3 的头两行待主 agent 跑 |
| 第 1 问 切字样 | 成立 | 多字运算符、路径、宏、字面量与注释的切法试了 11 种形状，没有切错而漏过名单外写法的 | 读码 |
| 第 1 问 显存记账那一批 | 成立（缺口照旧在） | 守护只读 `gpu_unit_checks.rs` 一份源码、只走打行那一批登记；显存记账那一批改成会 panic 而键不变，没有会红的测试。另判第二轮改法「对两批都跑」对这一批不该做，见第 1 问第三节 | 读码 |
| 第 2 问 打行那一层 | 不成立（打中） | 删掉打行那一层的 `println!`：两条钉字测试与 e202 两条照绿，变异判法与任务服务都看不到本次未跑。第二轮第 3 条要守的「服务认得出的那一行」从拼字挪到了打印，打印没人守 | 探针 1、2 |
| 第 2 问 两支没有不要卡的测试 | 成立（另一格空着，不属第 3 条） | 「上了别的卡判红」那一处 `assert!` 没有不要卡的测试，也没有变异行；变异表里点 `not_handed_over` 的 0 行 | 读码、`grep -c` |
| 第 3 问 块后那一句断言 | 推不出（打中一半） | 断言只证「判表那一处开的是自己的库」；CPU 参照取样那一处开哪一份没有断言，把取样挪出设着一次性库的那一段照绿。「没写进全仓那一份」它证不了 | 读码；探针 3 的第三行待主 agent 跑 |
| 第 3 问 GPU 那一段读不读库 | 成立 | 借卡之后只用内存里读回的 `packed`，GPU 那几份源码里没有开库的调用 | `grep` 零命中 |
| 第 3 问 CPU 参照挪前，比法变没变 | 成立 | 比的还是同一个 `cpu` 与最后的 GPU 结论；挪前之后取样一定现判（新的一次性库里没有可复用的），只更严 | 读码（推的） |
| 第 4 问 11 行追加、3 行改写的名字 | 成立 | 逐行对了改坏的地方与名字，见第 4 问 | 读码 |
| 第 4 问 拿掉守卫会不会写真服务 | 不成立（打中，条件式） | 门禁那一路每次起自己的临时 prune-server，不写；写段证红用的 `prove-red.sh` 不起临时服务，拿掉守卫那一趟判表与取样写进真服务 | 读码 |
| 第 4 问 命名三处要不要另一种检查 | 成立（不要） | 上游命名纪律缩写表里有 `id`，`gate.sh` 每趟都起它，改回 `process_id` 当场红 | 读码 |
| 第 6 问 走不到的理由 | 成立 | 新的 `expect` 都只在临时服务起不来时走到；判定里那一处 `assert!` 走得到，只是没有不要卡的测试（并进第 2 问） | 读码 |
| 第 6 问 类型与名字 | 成立 | 三张名单的长度由数组类型钉住；守护测试名只说 panic，文档写的危害更宽（附注） | 读码 |
| 第 6 问 不兼容旧代码 | 成立 | 五份文件改到的地方没有认旧版本的分支 | 读码 |
| 第 6 问 不新建判定库 | 成立 | 一次性库是测试对自己起的临时服务，用完丢，早就是验证包测试的通行做法 | 读码 |
| 第 6 问 多机可用 | 成立 | 输入现造、一次性库在临时目录、单进程内用 | 读码 |
| 第 6 问 测试优先多线程 | 成立（附代价，推的） | 写锁罩到整条流的 CPU 参照取样，期间同一个库内测试二进制里设一次性库的别的用例都等 | 读码 |
| 第 6 问 带 --item 的命令与项 | 推不出（打中） | 记录写的是「这一批新写与改写的 14 行变异」：11 加 3，第二轮改法第 11 条要的「G 行验证段点名现判」不在里面，这一轮的读数里找不到它 | 记录第 390 行 |
| 第 6 问 新崩溃流 | 成立 | 两条对拍是原有的流，没有新加 | 读码 |
| 未定项清单 | 不完整（打中） | 正文「没做的」漏了四样，见「未定项清单」一节 | `grep` |
| 这一轮报的数 | 一处推不出 | 「24 条、少掉的正是那两条串库对拍」分不出修好与没卡：没卡时那两条开头就报本次未跑、返回成功 | 读码 |

## 第 1 问：白名单守护

### 一、名单里两样调用不看调用方：两个登记单元互调

名单把两个登记单元的调用按名字放行，理由只写「自己也受这条守护」：

- `crates/singlefs-verification/src/crash_identity.rs:1829`：「("gpu_work_trace(", "同一批登记的单元，自己也受这条守护"),」
- `crates/singlefs-verification/src/crash_identity.rs:1830`：「("gpu_lifecycle_line(", "同一批登记的单元，自己也受这条守护"),」

认法里一段名字接 `(` 就交 `名(`，不看出现在哪个函数里（`crates/singlefs-verification/src/crash_identity.rs:1971`：「} else if punctuation_at(index + 1) == Some("(") {」）。于是下面这一处改动，每个字样都在名单里：

```rust
// crates/singlefs-verification/src/gpu_unit_checks.rs 的 gpu_lifecycle_line，format! 之前插一行
) -> String {
    gpu_work_trace(phase, detail);   // 字样：gpu_work_trace(  (  ,  )  ;   参数名 phase、detail 不算写法
    format!(
```

`fmt::Arguments` 是 `Copy`，`detail` 用两次编得过（推的，没编）。`gpu_work_trace` 调 `gpu_lifecycle_line`（`crates/singlefs-verification/src/gpu_unit_checks.rs:1032`：「gpu_lifecycle_line(」），后者又调回前者，无条件互调：进程里头一次打 GPU 阶段现场那一行就栈溢出，整个进程中止（不是 panic，接不住）。这两个单元都在登记里、都不沿它收，互调不添新单元，闭包与键都不变（推的，探针 3 第二行核闭包那条测试）。

守护文档写的危害是「判器在那一处倒下而键不变」（`crates/singlefs-verification/src/crash_identity.rs:2113`：「判器在那一处倒下而键不变；改了调用方的卡或判表，判定字跟着变而键不变」），名单的放行判据却只写不 panic、不改调用方（`crates/singlefs-verification/src/crash_identity.rs:1803`：「放行的判据：这一处不 panic，也不改调用方手里的东西」）。栈溢出、死循环这一类「倒下」不在放行判据里，名单也没挡。

四句：

- 分不分辨改法：分。第二轮撤掉的黑名单同样漏（互调里没有一个会 panic 的词）；把三个单元放回闭包的那条出路不漏（改一个字键就变、现判时当场倒下）。
- 守护当时看不看得到判别它的东西：看得到。`gpu_work_trace(` 出现在 `gpu_lifecycle_line` 的函数体里，守护读的正是这一段；缺的是按单元分的调用名单。
- 满足的是哪一分句：守护文档「判器在那一处倒下而键不变」那一句，不是测试名字面的「panic」。
- 第二轮给的改法（白名单）在这一格上：照中。

改法（只在我读码推出来、没实现没跑，被攻过零轮）：调用按单元分名单，`gpu_lifecycle_trace` 只许调 `gpu_work_trace(`，`gpu_work_trace` 只许调 `gpu_lifecycle_line(`，`gpu_lifecycle_line` 一个登记单元都不许调；放行判据补一句「不发散（不递归、不循环）」。修的是这一格；在第二节、第三节那几格上不起作用。

这一处的分量：互调是无条件的，一旦现判就当场倒下，看得见。它坐实的是「名单够不够」这一问：只用名单里的写法照样写得出判器倒下而键不变的代码。

### 二、切字样的认法：没找到漏的

试过的形状与结果写在「没打中的形状」第 1 条。这里只记两处认法以外、名单理由所依赖而守护不核的前提，今天都没有可用的打法：

- `format!`、`format_args!`、`writeln!` 按宏名放行（`crates/singlefs-verification/src/crash_identity.rs:1806` 那一行写的理由依赖参数类型）。格式串在比之前被换成空格（字面量一律抹掉），格式串里内联捕获的名字（`{名}`）守护看不见；`gpu_unit_checks.rs` 里没有手写的 `Display`、`Debug` 实现（`grep` 零命中），今天捕获不到会 panic 的格式化（推的）。
- 宏名按文本作用域解析，模块里先定义的同名 `macro_rules!` 会顶替标准库的 `writeln!`；同一份文件里卡池那条守护就是防这个（`crates/singlefs-verification/src/gpu_unit_checks.rs:4733`：「宏名按文本作用域解析，卡池模块外面先定义的同名 `macro_rules!` 会顶替标准库那一个」）。白名单守护不看模块里有没有这样的宏；但这样的宏只经登记单元够到时，`not_following_the_listed_units_drops_only_the_listed_ones` 会把它当「跟着被切掉、没逐个判过」判红，经留下的单元够到时它进键（推的，没跑）。

### 三、显存记账那一批：缺口照旧在；第二轮改法对这一批该撤

守护只编进一份源码（`crates/singlefs-verification/src/crash_identity.rs:1589`：「include_str!("gpu_unit_checks.rs"),」），只走打行那一批登记（`crates/singlefs-verification/src/crash_identity.rs:2225`：「let listed_units = crate::judging_units_entries::UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW;」）。显存记账那一批的 47 个登记一个都不过它。可照喂的改坏：

- `crates/singlefs-verification/src/gpu_memory_ledger.rs:2294` 的 `is_lost` 里 `.unwrap_or_else(PoisonError::into_inner)` 换成 `.expect("设备情况那把锁没中毒")`：别的线程持锁时 panic 过，判器线程读设备丢没丢就 panic。这个单元在登记里，`an_edit_of_a_listed_unit_of_the_memory_ledger_keeps_the_gpu_judge_key` 证的正是改它键不变；守它的三条测试（够到的恰好是登记的、登记名恰好一个单元、改了键不变）都不看函数体（推的）。

所以第二轮第 1 条那个缺口（不沿它收的单元会 panic、键不变、没有测试红）在这一批上照旧在，欠账表里也没有一行罩它：C1033（两个显存记账单元没有会红的检查）只管两个单元的前提，不管「会 panic」。

撤回：第二轮攻方腿给的改法「对两批「不沿它收」的登记都跑」，对显存记账那一批不该做。那一批的常量文档写明它们的本职就是改记账（`crates/singlefs-verification/src/judging_units_entries.rs:841`：「都不改崩溃状态的判定字：设备丢失与显存不够的记账、」），「不改调用方手里的东西」这条判据套上去，要么整批红、要么名单放宽到失去意义（实现员量的：18 个函数单元全有名单外的写法）。该做的是另一条：登一行欠账「显存记账那一批登记的单元改成会 panic 而键不变，没有会红的检查」，再由条款一侧定那一批的判据（例：只许改账本与借据、不许 panic），定之前不往白名单里加东西。

## 第 2 问：判定与打行拆开

### 一、打行那一层：删掉 println 没有测试红，服务看不到本次未跑

打行那一层只剩一句打印（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:701`：「println!("{}", judged.line_to_print);」）。两条钉字的新测试调的是不打行的判定，e202 那两条调打行这一层、只核交回的结局；变异表里点这一句的 0 行（`grep -c -F 'judged.line_to_print' crates/mutations.tsv` 输出 0）。删掉这一句：

- 四条测试照绿（推的，探针 1 核 e202 那一条）；
- 结局照旧是本次未跑，调用方不入库、照返回成功；但日志里一行 `NOT_RUN_NO_GPU_ADAPTER` 都没有，任务服务（`task-server/task_server.py` 的 `gpu_not_run_test` 按记号认）与变异判法都记不成本次未跑：要卡的用例在没认出卡的一趟里报成绿。

这正是第二轮第 3 条坐实的那种后果（「写坏了任务服务认不出」），只是坏的那一处从拼字挪到了打印。拆开之后字被钉住，「打没打出来」没人钉：第 3 条要守的东西又空了一块。实现员在第二份报告里写明了这一点（「被删掉不会有测试红」），正文也列了；这里补的是它为什么属于第 3 条，以及量法（探针 1、2）。

四句：

- 分不分辨改法：第二轮第 3 条的改法（改用拼行函数、钉字的测试）与拆开之后的做法都不防删；把打印改成写进调用方给的输出口（标准输出或测试里的一块内存），测试读那块内存，能防。
- 测试当时看不看得到：看不到。同一个进程里 libtest 截走了标准输出，测试读不回自己打的行；打行那一层不收输出口，就只能靠读日志的那三方。
- 满足的是哪一分句：第二轮判决第二节第 3 条「写坏了任务服务认不出」。
- 第二轮改法在这一格上：照中。

改法（推的，没实现没跑，被攻过零轮）：打行那一层收一个 `&mut impl std::io::Write`，真卡路径传标准输出的锁，测试传 `Vec<u8>` 再逐字比；e202 那两条改调判定（它们今天每跑一次都真打一行本次未跑记号，探针 2 那一趟的读数就是这个）。

### 二、「读到了、只在交来的卡上」与「上了别的卡判红」两支

上了别的卡那一处判红（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:677`：「not_handed_over.is_empty(),」）是服务判跑错卡之外的第二道。调判定的四处不要卡的调用（取卡测试文件两条、e202 两条）交的都是读不成或空集合，走不到这两支；变异表里点 `not_handed_over` 的 0 行、点 `GPUS_THIS_PROCESS_IS_ON` 的 0 行（`grep -c -F` 输出各 0）。把 `assert!` 的条件改成恒真，四条照绿（推的）。

这一支守的是「跑错卡判红」，不是第二轮第 3 条的记号行，不算第 3 条又空；但拆开之后它已经不要卡就测得到（交一个含不在交来的卡里的序号的集合，`#[should_panic]`），今天没有。记作另一格，交主 agent 定登不登。

## 第 3 问：两条串库对拍的一次性库

### 一、块后那一句断言证的是什么

段里两处经产品入口开库：判表过库与 CPU 参照取样（`crates/singlefs-verification/src/crash_identity.rs:2464`：「let Some(judged_on_the_cpu) = sample_crash_states_on_the_cpu(」）。块后的断言只读判表（`crates/singlefs-verification/src/crash_identity.rs:2495`：「.read_packed_judge_tables(&sha256_digest(&packed_tables.to_bytes()))」），话写的是「判表过库写进的是这条用例自己的一次性库（不是别的用例设着的那一份、也不是全仓那一份）」（`crates/singlefs-verification/src/crash_identity.rs:2498`）。

- 它证的是「判表那一处开的是自己的库」。读的是自己那一份、键是内容摘要，判表进了别处这一份里就没有，单跑也红、不靠跑序。
- 「没写进全仓那一份」它证不了：没有一处读全仓那一份；同一份判表两处都写，这一句照绿。差在这里：前一句管「这一份里有」，后一句管「别处没有」，只有前一句有断言。
- 文档说两处都在借卡之前开自己的库（`crates/singlefs-verification/src/crash_identity.rs:2339`：「判表过库与 CPU 参照取样在借卡之前、都开这条用例自己的一次性库，过库之后核判表写在这一份里」），判别力那一句只说判表（`crates/singlefs-verification/src/crash_identity.rs:2340`：「判别力（`crates/mutations.tsv`）：不把一次性库设成这个进程开的那一份，判表写到别处，核判表那一句红。」）。CPU 参照取样那一处开到哪一份，没有断言、也没有变异行。

可照喂的改坏：段里 `let mut cpu: Vec<CrossCheckWords> = Vec::new();`（`crates/singlefs-verification/src/crash_identity.rs:2463`）之前插 `drop(_the_library_of_this_process);`。判表照旧进自己的库、断言照绿；取样开到此刻设着的那一份（没人设着时是全仓那一份），恰是这一改要防的原样（推的，探针 3 第三行量）。两条变异行把整个守卫拿掉，两处一起坏，红在判表那一句，名字写「判表过库与 CPU 参照取样前」也对；单坏取样那一处没有行接。

四句：

- 分不分辨改法：分。只核判表（今天）不防；补一句读自己库里这条用例的取样记录，防。
- 测试当时看不看得到：看得到。块后自己的库还开着，取样记录按项名 `TEST_NAME` 读得到。
- 满足的是哪一分句：文档第 2339 行「都开这条用例自己的一次性库」里的「都」。
- 第三份报告的改法在这一格上：照中（断言只罩判表）。

改法（推的，被攻过零轮）：块后另加一句，开自己的库读这条用例取样的结论，读得到才过；探针 3 第三行的那一改做成必须红的变异行。

### 二、GPU 那一段读不读库

借卡之后只用内存里读回的 `packed` 建判器（`crates/singlefs-verification/src/crash_identity.rs:2518`：「GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, *width);」）。`crash_judge_dispatch.rs`、`gpu_unit_checks.rs`、`crash_judge_gpu.rs`、`gpu_memory_ledger.rs` 四份里 `CrashLibrary::open`、`open_or_share` 零命中；`crash_judge_dispatch.rs` 里调 `judge_tables_through_the_pairing_library` 的只有 `judge_every_state_on_a_card_leased_from_the_pool`，验证包库内没有调它的（`grep` 只命中定义那一行）。块不罩 GPU 那一段，不缺什么（读码，传递调用没逐个跟）。

### 三、CPU 参照挪到借卡之前，比法变没变

比的仍是段里收齐的 `cpu` 与借卡之后的 GPU 结论，逐状态、同一个比法，没变。变的一处：取样开的是新建的一次性库，里面没有可复用的结论，一定现判；挪前那一版开全仓那一份时可能读到旧结论（推的，复用路径没跟）。只更严。

另一处要知道的：段里遇到只录那一趟（`StageOutcome::RecordedOnly`）就返回；只录的结论写进一次性库，随用例丢掉，判那一趟读不到。今天带特性那一趟清掉了分段的环境变量，这两条也不在分段的登记里，走不到（第三份报告的读法，我没另核）。

## 第 4 问：变异行

### 一、名字与改坏的地方

这一批的 14 行在 `crates/mutations.tsv` 第 7934、7935、8221 行（改写）与第 8369 到 8379 行（追加，`sed -n` 数得 11 行）。逐行对了原文、替换文、点名的测试与名字：

| 行 | 改坏的地方 | 名字说的 | 对不对得上 |
|---|---|---|---|
| 8369、8370、8371 | `gpu_unit_checks.rs` 三个打行单元各插一处 `Option::expect`、`panic_any`、经共享借用记一笔显存不够 | 单元会 panic、改调用方的东西、键不变 | 对得上。附注：这三行点的是不带特性的 `--lib`，`gpu_unit_checks` 模块整个是 `#[cfg(feature = "gpu")]`，改坏的那份源码只被守护经 `include_str!` 读，从来不编；替换文我读着都编得过（推的） |
| 8372、8373 | 组成清单行 `@` 两边的空格、同名序号从几起 | 同 | 对得上 |
| 8374、8375 | 配对后两边没配上的都报、配上的拿掉 | 同 | 对得上 |
| 8376、8377 | 两个拼行函数丢掉读不成的那一句、不写进程号 | 同 | 对得上 |
| 8378、8379 | 两条对拍里设一次性库那一句换成只借用 | 判表过库与 CPU 参照取样前不设 | 对得上（两处一起坏）；单坏取样那一处没有行，见第 3 问 |
| 7934、7935 | 判定里两支的结局换成只在交来的卡上，要打的那一行不动 | 交回只在交来的卡上（本次未跑被盖掉） | 对得上：调用方当成绿，日志里那一行照旧是本次未跑 |
| 8221 | 编进去的被监视文件清单取成空的 | 弄坏的是清单，不是拷贝 | 对得上；它守的是清单测试，第二轮第 2 条那个薄壳照旧没有行接（正文已列） |

### 二、拿掉守卫那两行会不会写真服务

分两条路：

- 变异门禁（这一轮验证段那一趟走的这一路）：不写。每次起 cargo test 真跑都起自己的临时 prune-server（`scripts/mutation/crates-mutation-rows.py:1069`：「变异复跑的崩溃判定：起 cargo test 真跑测试的每一次」，`:1072`：「跑完停服务、删目录：走判定库的 验证包用例在变异版上判出的崩溃结论只落进那一份临时库，不读真库、不写真库」）。拿掉守卫，两处开到的「全仓那一份」就是那一份临时库。
- 写段证红 `scripts/mutation/prove-red.sh`：写。它起 cargo 时直接用调用方的环境（`scripts/mutation/prove-red.sh:516`：「environment = REPOSITORY_COPY.environment_for_commands_in_the_copy(MAIN_ROOT)」，`scripts/hosts/lib_repository_copy.py:918`：「copied = dict(os.environ if environment is None else environment)」，只另设多机配置），整份脚本只有自证那一段要临时服务。拿掉守卫那一趟没人设一次性库，两处开到真服务，写进判表与取样的结论（推的；那一趟真跑没跑过，我核不动，第三份报告把它列在「要主 agent 跑的命令」第 6 条）。

违不违反：产品源码自己写着测试不许写真服务（`crates/singlefs-verification/src/crash_library.rs:849`：「没人设着时开到全仓那一份（测试不许写真服务）」）。所以在 `prove-red.sh` 那一路是违反。写进去的判表按内容摘要存、取样结论按判法单元的键存，内容与没改坏时相同，不会污染判定（推的）；违反的是规矩，不是结论。这一处不是这一批的代码造成的：`prove-red.sh` 跑任何一行「点名测试经产品入口开库」的变异行都一样，属那份脚本，交主 agent 转给管它的会话。

### 三、命名三处要不要另一种检查

不要。上游命名纪律的缩写表里有 `id`（`.claude/singlefs-ai-sop/scripts/naming-lint.sh:277`：「id identifier」），`gate.sh` 每趟都起这一道（`.claude/singlefs-ai-sop/scripts/gate.sh:509`：「if stage_admitted "命名纪律" "$SCRIPTS_DIRECTORY/naming-lint.sh" "$ROOT"; then」）。改回 `process_id`，提交时那一道就红（第三份报告第 11 条拿 naming-lint 核那三处，照它推的，没跑）。变异表管的是行为，名字改回去行为不变，本来就不该进变异表。

## 第 6 问：八格检查单

1. 走不到的理由二次查实：成立。这一批新加的 `expect` 与 `unwrap_or_else(… panic!)` 都在测试代码里：两条对拍的 `DisposableLibrary::new`、`open_store`、`read_packed_judge_tables` 三处只在临时 prune-server 起不来、连不上时走到，那是环境坏了，该红；守护里那一处只在现造的源码写错时走到。判定里那一处 `assert!` 走得到（进程上了别的卡），由第 2 问第二节接着。
2. 类型与名字：成立。`FORMS_ALLOWED_…` 37 样、参数类型 5 样、多字标点 24 样都由数组类型钉住长度，写错个数编不过。附注：守护测试名说的是 `can_panic_or_change_what_its_caller_holds`，文档写的危害是「倒下而键不变」，比名字宽；第 1 问第一节那一格就落在这个差上。
3. 不兼容旧代码与旧格式：成立。读了五份文件改到的地方，没有认旧版本的分支、旧布局号或迁移。
4. 不新建判定库：成立。`DisposableLibrary` 是测试在自己进程里起的临时 prune-server、命名空间带进程号与时刻，用例收尾停服务删目录；验证包库内测试早就这样用（`crash_library.rs` 测试模块里设一次性库的十几处），属共用约束放过的「对临时服务起的自证库」那一类。块后那一句 `open_store` 只是再开同一份。
5. 所有任务多机可用：成立。守护的输入编进二进制；取卡判定的输入现造；一次性库在本进程起、本进程用，换哪一台都一样。
6. 测试优先多线程：成立，附一笔代价（推的，没量）：写锁从判表过库罩到整条流的 CPU 参照取样（状态数上限 1000），期间库内测试二进制里设一次性库或拿全仓那一份守卫的用例都等。不跨借卡的理由（第三份报告）成立；取样是在产品入口里开库，罩不短。
7. 带 --item 的命令与项：推不出（打中）。第二轮改法第 11 条后半要「G 行验证段点名现判」（`research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md:75`：「D 行补 `--exact`；G 行验证段点名现判」）。调度记录写的验证段那一趟是「这一批新写与改写的 14 行变异」（`records/2026-11-17-subagent账本清欠调度记录.md:390`：「这一批新写与改写的 14 行变异」），14 等于追加 11 加改写 3；G 行（第 8224 行「真卡取卡模块：没跑成的结局打成交了卡的那一行（本次未跑的记号行没打）」）这三份报告都没改，不在里面。头两份报告的门禁命令点了它（13 行），第三份合起来之后没点。这一轮报出的读数里找不到 G 行现判的那一趟；要么补一趟，要么在判决里写明 G 行留给提交时整表那一趟（`mod.rs` 改了，它的键会变、会现判）。
8. 新崩溃流多罩了什么：成立。两条对拍是原有的流，没有新加崩溃点重放流或枚举用例。

## 未定项清单：漏了四样

正文「没做的」列了欠账表那一行、命名三处没有变异行、拿掉守卫写真服务、集成测试开全仓那一份。对着第二轮判决「三、改法」逐行查，另有四样没做又没列：

1. 同包另 4 处手写的本次未跑记号行（第二轮改法表第 3 行后半，`research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md:72`：「同包另 4 处同形的一起改或各配钉字的测试」）。今天还在：`crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:575`、`:599`，`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/construction_phases_and_timing/tests.rs:53`、`:13745`（e202 那两处从判决记的 8924、12415 漂到了这里，`grep -n` 现查）。第一份报告把它放进「下一批」（那两份文件别的会话在改），正文没提。
2. G 行验证段点名现判（第二轮改法第 11 条后半），见第 6 问第 7 格。
3. 显存记账那一批的欠账：第二轮第 7 条「记账：第 1 条的改法做成对两批登记都跑」没做成，也没登欠账；C1033 不罩它。见第 1 问第三节。
4. D13（验证路线） 已定项 17 补一句「登记名里的行号只用来认单元、不进键」（第二轮第 10 条）：`grep -c` 在 `.claude/kb/decisions/13-验证路线.md` 里 0 处。第二轮「回看决策」写了等第三轮收了再写，属已排期；正文的清单里该有一行。

## 这一轮报的数：一处推不出

「合入后验证第 1 步只红带 gpu 特性那一步 24 条，第二轮修补前那一趟的 26 条里少掉的正是这两条串库对拍」——条数少两条，分不出「修好了」与「这一趟没卡」：两条对拍头一句就是取卡池，取不到就报本次未跑、返回成功（`crates/singlefs-verification/src/crash_identity.rs:2373`、`crates/singlefs-verification/src/crash_judge_gpu.rs:2567` 两处 `let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {`），照样不在 FAILED 里。能分辨的是那一趟日志里这两条有没有打本次未跑记号、`CRASH_LIBRARY_RECORDED … item=verdict_words_…` 两行的 `disposable-library=` 是不是各自的标签。正文「写段的改后…两条对拍各开了自己的一次性库」那一句靠的是后一种读数，判决引的时候引那两行原样，不引 24 与 26 的条数。验证段 14 行变异里两行守卫「抓到」，说明那一趟卡池起来了、断言走到了，但那是另一趟。

## 没打中的形状

1. 白名单的切法（第 1 问第二节），试了 11 种：多字运算符（`<<`、`!=`、`==`、`&&`，各认成一个记号、都不在名单）；带 turbofish 的路径 `std::sync::OnceLock::<…>::new`（前半截交出、`::` 单独交出而红）；带 turbofish 的方法 `.get_or_init::<…>(`（交成字段 `.get_or_init` 而红）；路径中间夹空白与注释（注释先换空格、记号跳空白，照拼回整条路径）；原始标识符 `r#expect`（`r` 与 `#` 各交而红）；字符字面量与生命周期（`'_`、`'\''`、`'é'`）；嵌套块注释（守护比编译器早收口、多看到代码，只会多红）；字节串、原始串与 `c"…"`（`c` 当词交出而红）；数字字段 `.0`；`let mut` 与按值 `mut` 参数；`let` 绑一个与名单里调用同名的局部（`let gpu_work_trace = std::thread::current;` 之后 `gpu_work_trace(` 照名单放行，但够得到的函数值只有名单里那几个读时钟、读进程号的，打不出 panic）。
2. 按名字放行的方法与字段落到别的接收者上：`&GpuCard` 没有叫 `name`、`id`、`lock`、`elapsed` 的方法，也不 `Deref` 到别的东西；`.index` 是 `u32`、`.pci_bus_identifier` 是 `String`（`gpu_unit_checks.rs` 第 300 到 332 行 `GpuCard` 的定义）。
3. 函数头之外守护不看的部分：`fn` 之前的属性（验证包的依赖里没有属性宏）、`unsafe fn`（名单里没有一样不安全的操作）、`async fn`（调用方当语句调，未用的 future 会被 `must_use` 报出来）、返回类型。
4. `Instant::duration_since`、`elapsed` 标准库文档保留了将来再 panic 的说法：工具链进键，换工具链就现判，不是键不变的那一类。
5. 一次性库的新受害者：两条对拍从「没设库的一方」变成「设库的一方」，同一个库内测试二进制里没设库、经产品入口开库的用例，可能在它们设着的那一段开到它们的库、之后连不上。`judge_every_state_on_a_card_leased_from_the_pool` 在库内没有调用方；`crash_library.rs` 测试模块里抽查五处开库都先设了自己的一份。没扫全库内测试，见「限度」。
6. 拼行函数的行格式、两步配对漏报、四条不要卡的测试名实相符：前两轮攻过，不重复。

## 这条腿自己的限度

- Rust 一行没编没跑。第 1 问互调能不能编过、编译器报不报互调，第 2 问四条照绿，第 3 问取样挪出段外照绿，都是推的，靠下一节的探针。
- `prove-red.sh` 那一路写没写过真服务，取决于那一趟跑没跑，我没有任务号、核不动。
- 显存记账那一批「判器线程上会调 `is_lost`」是照常量文档「被这一组的入口够到」推的，没跟调用链。
- 库内测试里还有没有别的「没设库就经产品入口开库」的用例，只抽查了五处。
- 守护的白名单只在「今天能写出什么」上试；名单理由里那几句标准库行为（`writeln!` 写失败交 `Err`、`OnceLock` 初值不 panic）照文档读，没另核。

## 要主 agent 跑的命令

副本 `/home/fy5090/.local/share/singlefs/work/code-r3-judging-closure-attack/copy`（`minimal-copy.py --with-test-inputs --with-gate-tables` 从主工作区此刻建，`.deliverables` 写成交接）里改了两处，别的与主工作区相同：

- `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs` 删掉第 701 行那一句 `println!("{}", judged.line_to_print);`（改后 sha256 2e58bf51d96b9cd5e28a061e386d249bba177a849c9ed9885cc1d93140f7243a）；
- `crates/mutations.tsv` 末尾追加三行探针变异（改后 sha256 dcb8904d9c437265e5d394fcdc5de1225e0560c387e68a8feb1d68702ffe65e1；改前原样在草稿 `mutations-copy-before.tsv`，sha256 与主工作区那一份相同，24faa7e6 起头）：
  - 「攻方腿探针：GPU 阶段现场那一行拼行之前先调打行的单元（两个登记的单元互调、栈溢出，只用白名单里的写法）」：`gpu_lifecycle_line` 的 `format!(` 之前插 `gpu_work_trace(phase, detail);`，点守护测试（不带特性）；
  - 「攻方腿探针：GPU 阶段现场那一行拼行之前先经 format_args 调打行的单元（带 gpu 特性编，登记的单元互调、闭包不变）」：同一处插 `gpu_work_trace(phase, format_args!("{detail}"));`（替换文与上一行不同，免得同一处改法两行），带 gpu 特性点 `not_following_the_listed_units_drops_only_the_listed_ones`，顺带核这一改在 gpu 特性下编不编得过；
  - 「攻方腿探针：GPU 派活宽度对拍的 CPU 参照取样挪出设着一次性库的那一段（判表过库照旧在段里）」：`let mut cpu: Vec<CrossCheckWords> = Vec::new();` 之前插 `drop(_the_library_of_this_process);`，带 gpu 特性点派活宽度那条对拍。

这三行在副本里是「要看它不红」：没红就是打中。不经 `prove-red.sh` 跑（第三行拿掉一半守卫，`prove-red.sh` 那一路会写真服务，第 4 问第二节），走变异门禁（每次起自己的临时 prune-server）。

1. `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 e202_gpu_cpu_gc_comparison 里的 1 条测试; 范围=1 个测试; 为什么=攻方腿副本探针：打行那一层删掉 println 之后测试照绿、任务服务与变异判法都看不到本次未跑' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/code-r3-judging-closure-attack/copy -- cargo test -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- tests::a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller --exact --show-output`
   - 目录：副本 `/home/fy5090/.local/share/singlefs/work/code-r3-judging-closure-attack/copy`。
   - 看：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里「test result: ok. 1 passed」，且没有一行含 `NOT_RUN_NO_GPU_ADAPTER test=a_process_whose_cards_cannot_be_read`；`task-server/task status <号>` 里「本次未跑」0 处、服务不推 gpu-cases-not-run 提醒。三样都是，第 2 问第一节打中坐实；有那一行或「本次未跑」1 处，打中不成立。
2. `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 e202_gpu_cpu_gc_comparison 里的 1 条测试; 范围=1 个测试; 为什么=攻方腿探针的对照：打行那一层没删时任务服务认得出本次未跑' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- tests::a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller --exact --show-output`
   - 目录：仓根 `/home/fy5090/code/singlefs`。
   - 看：`exit=0`；日志里「test result: ok. 1 passed」与一行以 `NOT_RUN_NO_GPU_ADAPTER test=a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller reason=` 起头的行；`task-server/task status <号>` 里「本次未跑」1 处。对照不是这样，第 1 条的读数作废。
3. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 带 --item 点名攻方腿副本里追加的 3 行探针变异; 范围=3 行; 为什么=攻方腿副本探针：两个登记的打行单元互调照过白名单守护与闭包测试，派活宽度对拍的 CPU 参照取样挪出一次性库那一段照绿；全域理由：探针看的是点名测试红不红，与这一批验证段那一趟同一个域才比得上' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/crates-mutation-replay.sh --item '攻方腿探针：GPU 阶段现场那一行拼行之前先调打行的单元（两个登记的单元互调、栈溢出，只用白名单里的写法）' --item '攻方腿探针：GPU 阶段现场那一行拼行之前先经 format_args 调打行的单元（带 gpu 特性编，登记的单元互调、闭包不变）' --item '攻方腿探针：GPU 派活宽度对拍的 CPU 参照取样挪出设着一次性库的那一段（判表过库照旧在段里）' /home/fy5090/.local/share/singlefs/work/code-r3-judging-closure-attack/copy`
   - 目录：仓根 `/home/fy5090/code/singlefs`（门禁脚本取仓里那一份，判的树是副本）。
   - 看：日志里「计数：」那一行与三行各自的判档；退 1 是预期（有没红的行）。头一行「没红」：第 1 问第一节守护那一半坐实；第二行「没红」：编得过、闭包不变，互调那一格坐实，「无效」（编不过）则第 1 问第一节撤回；第三行「没红」：第 3 问第一节坐实，「抓到」则撤回，「本次未跑」（借不到卡、卡池没起来）则照旧推不出。被范围声明闸拒、说明要缩域的，照说明改一次再交。

另有一件不用交任务、只读日志的：判决引「两条串库对拍修好了」时，取合入后验证第 1 步那一趟（或写段改后那一趟）`task-server/task log <号> --all` 里两条对拍的结果行、有没有 `NOT_RUN_NO_GPU_ADAPTER test=verdict_words_` 与两行 `CRASH_LIBRARY_RECORDED … item=verdict_words_…` 的 `disposable-library=` 原样（「这一轮报的数」一节）。

## 没做什么

- 不判辩方那一问（第 5 问）；不替主 agent 采纳。
- 没往统一任务服务交任务；上面三条与读日志那一件都由主 agent 做，判决要引的数由主 agent 跑出。副本上的数不算入库装置上的数。
- 没写计数模型，没有 `rerun.sh`（这一轮没有要算的崩溃状态）。
- 我提的改法（按单元分调用名单、打行收输出口、块后补读取样结论、显存记账那一批另登欠账）都只读码推出来、没实现没跑，被攻过零轮。

## 草稿与清理

- 留着：`/home/fy5090/.local/share/singlefs/work/code-r3-judging-closure-attack/copy`（168M，交接，主 agent 在里面跑上面第 1、3 条）；`mutations-copy-before.tsv`（改前副本变异表原样，备份）、`minimal-copy.txt`（建副本的输出）。
- 没建编译目录。
- 没删 /home/fy5090/.local/share/singlefs/work/code-r3-judging-closure-attack/copy：主 agent 要在里面跑探针，`.deliverables` 写成交接，用完由主 agent 删。
