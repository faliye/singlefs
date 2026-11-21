# 判器闭包与真卡测试共用实例那一批：代码轮第三轮（最后一轮），攻第二轮判决的修补（2026-11-19）

这一轮攻的是第二轮判决（`research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md`）「三、改法」的修补，另加用户在合入前定过要另修的两条串库对拍。修补已合进主工作区（合入后验证第 1 步只红带 gpu 特性那一步 24 条，全在用户定过照已知红合的那一组里；第 2 步在快照上全绿；打补丁的结局行「打上了：merge-patch」，回读「打全了…6 份文件现在的 sha256 都等于回读的改后」），快照取在合入之后。方案按 `crates/` 今天的实现来谈。前两轮攻过而没打中的角度不重复：两步配对会不会漏报真变了的单元、拼行函数的行格式与读它的三方认法、四条不要卡的测试证明的是不是名字说的事。这一轮是最后一轮，这一轮的修补在判决里标零轮。

## 范围

被判的五份文件与一张表：

- `crates/singlefs-verification/src/crash_identity.rs`（只动 `#[cfg(test)] mod tests`）
- `crates/singlefs-verification/src/crash_judge_gpu.rs`（只动 `#[cfg(test)] mod tests`）
- `crates/singlefs-verification/src/judging_units_entries.rs`（只动两处文档注释）
- `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs`
- `crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs`
- `crates/mutations.tsv`（合起来追加 11 行、换 3 行；变异名在三份实现员报告里逐条列着）

实现员报告三份，依次承接：`~/.local/share/singlefs/work/code-r2-judging-closure-repair/result.md`（白名单、两处手写记号行、两处文档、单元名格式断言、D 行）、`~/.local/share/singlefs/work/code-r2-judging-closure-repair-second/result.md`（判定与打行拆开、两行旧变异换锚点）、`~/.local/share/singlefs/work/code-r2-judging-closure-repair-third/result.md`（`process_id` 改名、两条串库对拍各开一次性库）。

压着的条款：`.claude/kb/decisions/13-验证路线.md` 的 D13（验证路线） 已定项 17；第二轮判决「二、判决」「三、改法」。

## 实现今天的样子

主 agent 的观测（读的是合入之后主工作区的文件）：

- `crates/singlefs-verification/src/crash_identity.rs:2120` 守护测试 `no_listed_unit_that_only_prints_the_gpu_lifecycle_line_can_panic_or_change_what_its_caller_holds` 从黑名单改成白名单：写法名单 `FORMS_ALLOWED_IN_THE_UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE`（37 样）、参数类型名单（5 样），函数体切成字样，不在名单里的一律红；红句给出路「把单元放回闭包，不为过守护改写法」。只罩新登记的那一批；显存记账那一批的登记实现员停下交主 agent（第一份报告「停下交主 agent 的」第 1 条：今天过不了白名单）。
- 拷树薄壳交的清单：实现员停下交主 agent（第一份报告同一节第 2 条：今天的清单里没有一份不在 `crates/` 之外整目录拷底下的文件，判不出）。`crash_identity.rs:2682` `data_files_outside_the_crates_the_build_read`，`:2716`、`:2904` 两处交它。
- `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs`：判定与打行拆开，判定 `gpus_this_process_is_on_and_the_line_to_print` 交回结构 `GpusThisProcessIsOnAndTheLineToPrint { outcome, line_to_print }`，打行那一层 `gpus_this_process_is_on_judged_against_the_handed_over` 只剩调判定、`println!`、交回；两处记号行都经 `not_run_line_of_the_test` 拼；参数名 `process_id` 改成 `process_identifier`。上了交来的卡之外的卡照旧 panic，改在判定里。
- 取卡测试文件：两条测试经判定核交回的整个结构，变异下失败话里的那一行在 `Debug` 引号里、不在行首，变异判法认不成「本次未跑」。e202 装置里两条调打行那一层的测试没改。
- 两条串库对拍（`crash_identity.rs` 测试模块的 `verdict_words_of_every_state_are_the_same_at_a_dispatch_width_of_one_of_seven_and_of_every_state_and_agree_with_the_cpu_judge`、`crash_judge_gpu.rs` 测试模块的 `verdict_words_of_the_new_pool_file_creation_are_the_same_whatever_the_accounting_slices_and_agree_with_the_cpu_judge`）：开库之前建自己的一次性库，在一个块里经 `DisposableLibrary::make_it_the_library_of_this_process` 设成这个进程的那一份，块里判表过库与 CPU 参照取样，块后断言自己的库里读得到判表，之后才借卡上 GPU 判。CPU 参照取样因此挪到借卡之前。两行新变异各把那一句换成不设。
- `judging_units_entries.rs` 两处文档注释：第二批常量文档改成「方法名 `name` 在这一组留下的单元里本来就算见过（`crash_judge_dispatch` 的 `judge_one_dispatch` 里 `pass.name()`）」；另一处「两条前提」改成「三条前提」并点名守护测试。
- 量到的读数：合入后验证第 1 步只红带 gpu 特性那一步 24 条，第二轮修补前那一趟的 26 条里少掉的正是这两条串库对拍；第 2 步在快照上命名四格与 naming-lint 都绿，点名 14 行变异「抓到 14 条、没红 0 条、无效 0 条、锚点失效 0 行」；写段的改后不复现那一条「2 passed; 1 failed」，失败的只有第一版就不支持的那一条，两条对拍各开了自己的一次性库。
- 没做的：第二轮改法第 5 条「欠账表登一行，真卡取卡薄壳交的是不是进程级那一格、打没打出拼好的那一行」，`.claude/kb/checks-owed.md` 里还没有（`grep -n 薄壳` 零命中），归 kb 写回、不在这一轮的代码里。三份报告「停下交主 agent 的」里另有：命名三处没有会红的变异行；变异下拿掉守卫的那一趟会写真服务；集成测试里经卡池对拍的那些用例开的是全仓那一份。

## 要腿判的

攻方腿攻修补本身，辩方腿复核第二轮判决。

1. 白名单守护：名单里 37 样逐样放行的理由成不成立，有没有一样放行之后照样能写出会 panic 或改调用方东西的代码（给得出一份过得了白名单的改坏写法）；切字样的认法（多字运算符、路径、宏）有没有切错让名单外的写法漏过去的。显存记账那一批今天过不了白名单、停下没改，判这一批是不是照旧有第二轮第 1 条那个缺口。
2. 判定与打行拆开：打行那一层的 `println!` 被删掉不会有测试红（实现员写明）；判定里「读到了、只在交来的卡上」与「上了别的卡判红」两支没有不要卡的测试。判这两处是不是第二轮第 3 条要守的东西又空了一块。
3. 两条串库对拍的一次性库：块只罩两处开库，不罩借卡之后 GPU 判与比；GPU 那一段读不读库、读的是哪一份。块后那一句断言证明的是「判表写进了自己的库」还是「没写进全仓那一份」，两者差在哪。CPU 参照取样挪到借卡之前，结论比法有没有因此变。
4. 变异行：11 行追加与 3 行改写各自改坏的地方与名字说的是不是一回事；拿掉守卫那两行在变异下会不会写真服务的库，是不是违反「测试不许写真服务」。命名三处没有会红的变异行，判要不要另一种检查。
5. 辩方腿：第二轮坐实的 5 条、认下记账的 6 条、不采的 1 条，判第二轮的判法够不够得着；第二轮改法第 5 条的欠账表那一行没登，判第二轮判决写的「写回 kb，不是代码」是不是把它留成了没人做的。
6. 八格检查单逐格判。

### 检查单：走不到的理由二次查实

三份实现员报告「走不到的理由」都是无：`crates/<包>/src/` 的产品代码没改，`crash_identity.rs`、`crash_judge_gpu.rs` 只改测试模块。判定里那一处 `assert!`（上了交来的卡之外的卡）在产品 GPU 测试包的 `tests/` 下，从打行那一层原样挪进判定。新加的 `expect` 在测试代码里，腿自己读一遍各依赖什么。

### 检查单：类型与名字

新名字：`FORMS_ALLOWED_IN_THE_UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE`、`PARAMETER_TYPES_ALLOWED_IN_THE_UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE`、`tokens_of_the_body`、`forms_in_the_function_body`、`forms_outside_the_allowed_ones`、`unit_names_in_the_compiled_compositions_keep_line_numbers_as_digits_after_an_at_sign_and_the_file_after_a_spaced_at_sign`（`crash_identity.rs`）；`gpus_this_process_is_on_and_the_line_to_print`、`GpusThisProcessIsOnAndTheLineToPrint`、`not_run_line_when_the_gpus_of_the_process_are_not_read`、`not_run_line_when_the_process_is_seen_on_no_gpu`、参数 `process_identifier`（`mod.rs`）；取卡测试文件里两条改名的测试。命名检查在快照上过了（门禁 code-source-discipline 四格「4 格里 4 格绿」、naming-lint「命名纪律通过：查了 1071 个 .rs 文件」）。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：没有认旧版本的分支、旧布局号、迁移路径。查过的范围是合起来那一份补丁的五份文件。

### 检查单：不新建判定库

两条串库对拍各建一份一次性库（`DisposableLibrary`，标签 `dispatch-width-pairing`、`accounting-slice-pairing`），用完丢掉；它是不是「并回即删的中转库与自证临时服务的库」之外的新判定存放处，腿判。

### 检查单：所有任务多机可用

新测试不读本机独有状态：守护测试的输入编进二进制；取卡测试的输入（读不成的原因、空的卡集合、一张现造的卡、进程号 4242）在用例里现造；两条对拍的一次性库在临时目录。

### 检查单：测试优先多线程

新加与改写的都是单元测试，每条一个场景。两条对拍在一个进程里设一次性库要拿写锁，块只罩开库那两处、不跨借卡，实现员写的理由是持锁借卡会让别的设一次性库的用例互等。没有写死线程数的地方。

### 检查单：带 --item 的命令与项

带 `--item` 的是合入后验证点名的 8 个测试目标（重列项列出）与快照上点名 14 行变异的那一条（范围声明与根都写了）。没有新加崩溃枚举用例与放量分项。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放流或崩溃枚举用例。
