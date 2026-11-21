# 辩方腿：判器闭包与真卡共用实例代码轮第三轮（2026-11-19）

复核对象：`research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md`（下称「第二轮判决」），依据它的两份腿报告（`research/prompts/judging-closure-and-real-card-instance-code-r2-opus-output.md`、`research/prompts/judging-closure-and-real-card-instance-code-r2-sonnet-output.md`）与三份实现员报告（路径在正文「范围」一节）。回的是正文「要腿判的」第 5 问与第 6 问八格。没写模型，没有 `rerun.sh`；一条命令都没交，任务日志用 `task-server/task status` 读，也只读地 grep 了任务服务的日志目录。

开工核了三件事：
- 开工快照 `sha256sum -c research/prompts/judging-closure-and-real-card-instance-code-r3-snapshot/crates-sha256.txt`：六行全是 OK（量的）。读的是主工作区此刻的那一版。
- 没读禁读清单里的文件。
- 引文行号都是此刻文件里 `grep -nF` 现取的；日志的读数只抄那一行的字，不写任务号。

## 判定一览：第 5 问（第二轮判决的 12 条）

判法：「判决站得住」是判决的诊断与它给的去处，在今天的文件与日志里都对得上；「判决够不着」是诊断对、但它给的改法或去处够不到那一格（或今天没落地）；「推不出」是现查查不出对错。「:N」是第二轮判决文件的第 N 行。

| 条目 | 辩方判定 | 一句话 |
|---|---|---|
| 坐实 1 守护换一种写法绕得过（:39） | 判决站得住 | 旧守护认方法调用要求词前带点，三种换写法的探针在两种编法下守护都绿；今天的白名单对照里三种都认得出，三行变异抓到 |
| 坐实 2 拷树薄壳交空清单没有测试红（:40） | 判决站得住；去处的后半句「登欠账」够不着 | 量的只有点名的 2 条与 3 条测试；今天清单里没有能分出薄壳的文件，是等价形态；欠账表里没有这一行 |
| 坐实 3 另两处手写记号行没有钉字（:41） | 判决站得住；「同包另 4 处」够不着 | 这两处已改走拼行函数并配了整行测试；同包另 4 处手写记号行今天原样在，正文「没做的」没列 |
| 坐实 4 `judging_units_entries.rs` 文档（:42） | 判决站得住 | 两处文档都改了；「在这一组闭包里」靠 `judge_one_dispatch` 是登记入口推的，没跑单元图 |
| 坐实 5 真卡上要量的没有登记位（:43） | 判决站得住；去处「写回 kb，不是代码」够不着 | 全仓没有登记位，量的；判决没点名谁写、何时写，调度记录里那句触发已满足而没做，这一条被留成了没人做的 |
| 认下 6 五样今天不 panic（:47） | 判决站得住；`format!` 那条放行理由推不出 | 五样都进了白名单；`format!` 的理由只管标准库类型，`{detail}` 带调用方的 `Display` |
| 认下 7 显存记账那一批有同一个缺口，改法「对两批都跑」（:48） | 缺口站得住；改法够不着 | 那一批的前提是行为（不改判定字），白名单的判据套不上；前一轮三方对同一批已判「设备丢失那一类写明做不成」，而辩方腿写明没读那份判决 |
| 认下 8 第一轮判决两句说宽了（:49） | 判决站得住 | 借用那一半在探针里量到，拷循环「有条件的守」写明了 |
| 认下 9 四件认下没改（:50） | 单元名与两步配对两件站得住；`checking_threads()` 那件够不着 | 钉格式的断言与两条配对用例都落了；转给管那一处的会话没有记录，行号又漂了一次 |
| 认下 10 红句「空表」措辞与 D13 补一句（:51） | 判决站得住 | 措辞没动，记账不是改；D13 写回有触发（第三轮收了再写），还没写 |
| 认下 11 D 行 `--exact`、G 行现判（:52） | 判决站得住 | D 行第五段末尾有 `--exact`；G 行在合入后验证的变异门禁里现判过，红了 |
| 不采 12 守护误报逼出绕法（:56） | 判决站得住 | 白名单对没登记的函数调用一律红，绕法要改名单，名单改动过三方；红句出路写了 |
| 两步配对不漏报、停（:58-60） | 判决站得住 | 补的两件已有用例，名字归属那件今天只会多点名一个 |

第二轮判决「三、改法」六行的落地：第 1 行白名单做了，「对两批登记都跑」没做（见条 7）；第 2 行「写明今天判不出」做了，「登欠账」没做；第 3 行薄壳拼行函数做了，同包另 4 处没做；第 4 行文档做了；第 5 行欠账表没登；第 6 行 D 行与 G 行做了。

## 判定一览：第 6 问（八格）

| 格 | 判定 | 一句话 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 两份验证包源文件的改动都落在 `mod tests` 里，`mod.rs` 与取卡测试文件在 `tests/` 下；新加的 `expect` 各依赖什么见第 6 问逐格 |
| 类型与名字 | 成立 | 命名两道在合入快照上绿；守护又多一处字符串约定（`rsplit_once` 拆限定名、按模块名字符串找源码），不放过只会红，属已记账那一类 |
| 不兼容旧代码与旧格式 | 成立 | 五份文件新加的行里没有认旧版本、迁移与放宽的写法；格式断言对不认得的名字报红 |
| 不新建判定库 | 成立 | 一次性库是测试进程里起的临时服务里的一段名字空间，不连真服务，落在共用约束写明不算的那一类；拿掉守卫的那条变异会回到写全仓那一份，写哪一份我没读到 |
| 所有任务多机可用 | 成立 | 输入编进二进制或在用例里现造，临时目录取 `std::env::temp_dir()`；读数只在第二台出过，本机的是推的 |
| 测试优先多线程 | 推不出 | 写锁只罩开库与 CPU 参照取样、不跨借卡，对；但一次性库占的临时服务名额从建库一直占到用例结束，跨借卡等待，两条用例都没有提前放手；名额数是推的 |
| 带 --item 的命令与项 | 成立 | 点名 14 行日志「计数：抓到 14 条、没红 0 条、无效 0 条、锚点失效 0 行（共 14 条）」；其中现判 1 条、复用 13 条 |
| 新崩溃流多罩了什么 | 成立 | 两条对拍不在登记的崩溃枚举用例与放量分项里（`stage-inputs.tsv`、`crash-amplification.sh` 零命中） |

另：「放的位置」一格今天不要求（`.claude/gate.d/doc-process-records.sh:736` 第九格截止日是 2026-11-20），正文改写落款在 11-20 及以后要补。

## 第 5 问逐条（证据）

### 坐实 1 守护换一种写法绕得过：判决站得住

- 旧守护的认法：开工快照倒推树 `~/.local/share/singlefs/work/code-r2-verify/tree` 里 `crates/singlefs-verification/src/crash_identity.rs` 第 1817 行是 `if before.ends_with('.')`（`grep -nF` 命中，量的）：会 panic 的方法名前面要紧接一个点才认。
- 三种换写法的探针读数（任务日志里的字，量的）：不带特性那条 `cargo test -p singlefs-verification --lib -- crash_identity::tests::no_listed_unit_that_only_prints_the_gpu_lifecycle_line_can_panic_or_change_what_its_caller_holds crash_identity::tests::the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree --exact` 日志「test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 894 filtered out」；带 gpu 特性那条「test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1073 filtered out」，编译行指向攻方腿副本（「Compiling singlefs-verification v0.1.0 (/home/fy5090/.local/share/singlefs/work/code-r2-judging-closure-attack/copy/crates/singlefs-verification)」）。三处改动在 `gpu_unit_checks.rs` 里，那个模块只在带 gpu 特性时编（「pub mod gpu_unit_checks;」(`crates/singlefs-verification/src/lib.rs:55`)），所以带特性那趟绿，同时说明三种写法编得过。
- 今天的样子：白名单守护 `crates/singlefs-verification/src/crash_identity.rs:2120` 对三种写法各现造一份，逐字认得出：「对照：取线程名写成 Option::expect(…) 认得出」(`crates/singlefs-verification/src/crash_identity.rs:2135`)、「对照：时钟早于 1970 年时 panic_any 认得出」(`crates/singlefs-verification/src/crash_identity.rs:2156`)、「对照：经共享借用的卡记一笔显存不够认得出」(`crates/singlefs-verification/src/crash_identity.rs:2173`)；三行变异：「只打日志的单元会 panic：GPU 阶段现场那一行取线程名改成 Option::expect」(`crates/mutations.tsv:8369`)、「只打日志的单元会 panic：GPU 阶段现场那一行的墙上时钟早于 1970 年时 panic_any」(`crates/mutations.tsv:8370`)、「只打日志的单元改调用方的东西：GPU 阶段现场那一行带卡号时经共享借用给卡记一笔显存不够」(`crates/mutations.tsv:8371`)。合入后验证的变异门禁日志里这三行都是「红了」（后两行有一趟「红了（现判：判法单元变了）」，最终那趟是「红了（复用）」）。
- 替「不登记、让三个单元留在闭包里」写的最强理由：守护机器从 `SOURCES_OF_THE_MODULES_…` 到守护测试尾共 686 行（`crash_identity.rs` 1584 到 2269，两个行号相减，量的），外加 37 样写法与 5 样参数类型；今后改这三个单元里任何一个字样都要先过名单，名单改动「由代码轮三方判」：「新写法确实不 panic、不改调用方的东西的，往名单里加一行并写明为什么放行，由代码轮三方判」(`crates/singlefs-verification/src/crash_identity.rs:2267`)。留在闭包里则键跟着函数体走，不要任何文本判法。这条理由的另一半是代价：日志改一个字让 GPU 判器那一组现判多少块，第一轮辩方腿就写了「推翻它的现象」，两轮以来没人量过（正文与第二轮判决里都没有这个数）。
- 判定：判决只写「守的检查不够」，没写「该不登」，站得住；「登记加守护」与「不登」哪个便宜，今天推不出。推翻判决的现象：白名单对今天三个函数体报出一处误红（今天报空，实现员用 Python 照搬认法核过，推的，没复跑），或三种写法里任一种在今天的白名单下仍绿。

### 坐实 2 拷树薄壳交空清单：判决站得住，去处「登欠账」没落

- 薄壳交清单：「&data_files_outside_the_crates_the_build_read(),」(`crates/singlefs-verification/src/crash_identity.rs:2904`)；整目录拷在清单之前：「for crate_directory in crate_directories_outside_the_crates_directory(repository) {」(`crates/singlefs-verification/src/crash_identity.rs:2940`)；带 gpu 的清单测试读清单本身、不经薄壳：「data_files.contains(&"prune-server/src/lib.rs"),」(`crates/singlefs-verification/src/crash_identity.rs:2731`)；不带特性清单钉成空：「不带 gpu 特性编时构建读到的 crates 之外的文件」(`crates/singlefs-verification/src/crash_identity.rs:2718`)。
- 探针读数的范围：把薄壳换成 `&[]` 之后，量到的只有点名的 2 条（不带特性）与 3 条（带 gpu）测试照绿。经薄壳的测试在今天的文件里有 4 处（`crates/singlefs-verification/src/crash_identity.rs:3044`、`:3068`、`:3503`、`:3678`，如「let copy = RepositoryCopy::new(label);」(`crates/singlefs-verification/src/crash_identity.rs:3044`)），探针没跑它们；它们也照绿是推的（整目录拷已把清单里的文件拷了）。判决写「没有测试红」时没有把范围缩到点名的那几条，但结论不受影响。
- 为什么今天判不出：不带特性清单是空的，带 gpu 清单里的文件全在被整目录拷的 `prune-server/src/` 底下，换成空清单与不换，拷出来的树逐份相同，是等价形态。判决的去处写了这一种：「没有就写明今天判不出、登欠账」（:71）。实现员「停下交主 agent 的」第 2 条写明了今天判不出，欠账表没登：
```
$ grep -n '薄壳' .claude/kb/checks-owed.md; echo rc=$?
rc=1
```
- 判定：诊断站得住；去处的后半句在今天的欠账表里没有，归条 5 一起说。推翻判决的现象：清单里有一份不在任何被整目录拷的包目录底下的文件，而换成空清单时活树键测试仍绿。

### 坐实 3 另两处手写记号行：判决站得住，「同包另 4 处」没做

- 这两处今天走拼行函数：「not_run_line_of_the_test(」(`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:802`)、同一句在 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:815`；记号只剩一处拼：「format!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}")」(`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:792`)。取卡测试文件逐字钉整行：「读不成这个进程在哪几张卡上：nvidia-smi 起不来（用例现造）」(`crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs:318`)。
- 判决的探针读数（任务日志里的字，量的）：记号后多一个冒号，日志里那一行是「NOT_RUN_NO_GPU_ADAPTER: test=a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller reason=读不成这个进程在哪几张卡上：nvidia-smi 起不来」，结局说明里没有「本次未跑」；对照那条的结局说明有「这一趟有 1 条用例本次未跑（日志里 1 行行首是 NOT_RUN_NO_GPU_ADAPTER）」。
- 够不着的：判决「三、改法」第 3 行要「同包另 4 处同形的一起改或各配钉字的测试」（:72）。今天 4 处原样在：「"NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=这张卡一个绑定至多」(`crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:575`)、同文件 `:599`、「"NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={REASON_WITHOUT_CARDS_FROM_THE_TASK_SERVER}"」(`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/construction_phases_and_timing/tests.rs:53`)、`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:13745`（判决里写的 8924、12415 行号已漂）。第一份实现员报告「下一批」一节与调度记录都记成「下一批」：「同包另 4 处同形的手写记号行在别的会话改着的两份文件里，这一件不动、进下一批」(`records/2026-11-17-subagent账本清欠调度记录.md:306`)。可这是代码轮的最后一轮，「下一批」的代码要走新的三方轮，而判决与正文「没做的」一节只列了欠账那一行，没列这四处。
- 新出的同类：拆开之后打行那一层有一句没有任何测试罩的打印：「println!("{}", judged.line_to_print);」(`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:701`)，与薄壳里的「println!("{}", line_reporting_the_gpus_taken(test_name, &taken));」(`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:878`)同类；实现员写明删掉它不会有测试红。「这一行打没打」只有真卡上量得出，归条 5 的欠账行，那一行要点名这两处。
- 判定：两处站得住；同包另 4 处判决的改法没做，且没进今天的记账。推翻「够不着」的现象：主 agent 在最终判决里把这 4 处另记了去向（谁改、走哪一轮三方）；调度记录那句只说「进下一批」，没有轮次。

### 坐实 4 文档：判决站得住

- 「方法名 name 在这一组留下的单元里本来就算见过」(`crates/singlefs-verification/src/judging_units_entries.rs:1191`)；「三条前提由 crate::crash_identity 的测试守着：」(`crates/singlefs-verification/src/judging_units_entries.rs:1174`)。
- `pass.name()` 在 `judge_one_dispatch` 里：「self.submit_one_pass_and_wait(&passes, pass.name(), pipeline, None),」(`crates/singlefs-verification/src/crash_judge_dispatch.rs:1726`)，同一句另在 `crates/singlefs-verification/src/crash_judge_dispatch.rs:1811`；它是登记的入口：「"singlefs_verification::crash_judge_dispatch::GpuCrashJudge::judge_one_dispatch",」(`crates/singlefs-verification/src/judging_units_entries.rs:68`)。「在这一组闭包里」要单元图 `why` 才证得了，没跑（推的）。
- 文档不进指纹：「文档注释不进条目指纹」(`crates/singlefs-unit-graph/src/rust_items.rs:10`)；「if treatment == DocumentationCommentTreatment::KeptAsTokens {」(`crates/singlefs-unit-graph/src/lexer.rs:277`)。

### 坐实 5 真卡上要量的没有登记位：诊断站得住，去处「写回 kb，不是代码」把它留成了没人做的

- 诊断：全仓没有登记位。`.claude/` 与 `records/` 下按「薄壳」「进程级」「真卡取卡」搜欠账表，零命中；「合入后要在真卡上量」这句话在全仓 `.md`、`.tsv`、`.txt` 里只出现在 `research/prompts/` 下的判决与腿报告里（`grep -rn '合入后要在真卡上量' --include=*.md --include=*.tsv --include=*.txt .` 去掉 `target/`，按文件数是 5 处，量的）。
```
$ grep -n '薄壳' .claude/kb/checks-owed.md; echo rc=$?
rc=1
$ grep -n '进程级\|真卡取卡' .claude/kb/checks-owed.md
（无输出）
```
- 判决给的去处只有一格：「写回 kb，不是代码」(`research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md:74`)，没写谁写、什么时候写。同一份判决里 D13 那一行有触发：「写回时点名的常量与守护测试等第三轮收了再写」(`research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md:95`)；欠账那一行没有。
- 触发其实在调度记录里：「判决第 5 条（真卡上要量的登进欠账表）等这一批落了一起写回」(`records/2026-11-17-subagent账本清欠调度记录.md:306`)；这一批落进了主工作区，同一份记录的后一条已经开了第三轮：「接着开这一题代码轮第三轮（最后一轮）」(`records/2026-11-17-subagent账本清欠调度记录.md:399`)。触发满足、欠账表里没有这一行（上面的 grep），正文「没做的」又写成「归 kb 写回」。这是同一件事第三次被推给「写回」而没有落到某一次派发。
- 规则对「推后」的要求：「只写进调度记录不算延后，算没去向」(`.claude/main-agent.md:25`)；欠账表属于「要另登一行才有人接的」：「要另登一行才有人接的（欠账表、拆分提案、决策正文）不进这张表」(`.claude/gate.d/deferral-registries.tsv:3`)。调度记录那句只是等待，不是登记。
- 要写回的不止判决那一行。写回清单（给主 agent；我不写 kb，行的格式见欠账表表头「编号 | 简称 | 要拦什么 | 怎么拦（会红的形态） | 前置 | 出处」）：
  1. 真卡取卡：三处没有要卡的测试罩着——打行那一层的 `println!`（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:701`）、取卡薄壳的 `println!`（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:878`）、薄壳交进去的是不是进程级那一格：「&VULKAN_INSTANCE_OF_THIS_TEST_PROCESS,」(`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:872`)；前置是 GPU 服务代跑测试的接口。
  2. 薄壳交的清单（条 2）：今天判不出，等清单里出现不在被整目录拷的包目录底下的文件。
  3. 同包另 4 处手写记号行（条 3）。
  4. 已有的 C1033：派活宽度那一条今天有了两条用例，该行没更新（条 7）。
  5. D13 已定项 17 的两句（条 10）与第二轮判决回看决策点名的常量与守护测试。
- 判定：诊断站得住；去处够不着（无主、触发满足而未做）。推翻的现象：欠账表或 `deferral-registries.tsv` 登记的流程文件里已有一行点名这一件，或有会话已认领。

### 认下 6 五样今天不 panic：判决站得住，`format!` 的放行理由推不出

- 五样都进了名单，每样一句理由：「标准库给这几样的格式化不 panic，也不改它们」(`crates/singlefs-verification/src/crash_identity.rs:1806`)、「写失败交 Err，由 let _ = 吞掉，不 panic」(`crates/singlefs-verification/src/crash_identity.rs:1808`)、「取当前线程的句柄，函数体里只读它的名字与号」(`crates/singlefs-verification/src/crash_identity.rs:1817`)、「这个进程头一次打这一行的时钟起点，只管 elapsed_microseconds 那一截，不进判定字」(`crates/singlefs-verification/src/crash_identity.rs:1833`)、「共享借用卡；函数体里只读名单里的两个字段」(`crates/singlefs-verification/src/crash_identity.rs:1851`)。
- `format!` 那条理由够不着的地方：`{detail}` 是调用方交的 `fmt::Arguments`，格式化时跑调用方实参的 `Display` 与 `Debug`。今天 25 处调用点（`grep -rnE 'gpu_lifecycle_trace\(|gpu_work_trace\(' crates --include=*.rs` 去掉注释行与 `fn` 定义行，量的）的实参是整数、`&str`、`Vec` 的 Debug 与一个轮询结果的 Debug，如「format_args!("states={ordinals:?} pass=readback result={polled:?}"),」(`crates/singlefs-verification/src/crash_judge_dispatch.rs:1862`)；`polled` 那个类型的 Debug 实现我没读（推的）。守护读不到调用方，也读不到格式串里的 `{名字}`：字面量的内容被换成空白（「fn rust_source_without_comments_and_literals(source: &str) -> String {」(`crates/singlefs-verification/src/crash_identity.rs:1696`)）。单元图把 `{名字}` 当裸名连边：「含格式串里 {名字} 捕获的名字」(`crates/singlefs-unit-graph/src/units.rs:359`)，只经登记单元够到的新单元会被「少掉而没登记」那条测试点名；那是第二道，不是白名单。
- 判定：五样都按判决要求列进去并写了理由；`format!` 那句「标准库给这几样的格式化不 panic」对 `{detail}` 只在今天的实参上成立。推翻：某个调用点的实参是调用方自己的 `Display` 并会 panic，而守护与「少掉而没登记」两条都绿。

### 认下 7 显存记账那一批：缺口站得住，改法「对两批登记都跑」够不着

- 登记今天有四张：「pub const UNITS_OF_THE_MEMORY_LEDGER_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW: &[&str] = &[」(`crates/singlefs-verification/src/judging_units_entries.rs:848`，47 条，量的：`sed -n 848,906p` 接 `grep -c '^    "'`)、「pub const UNITS_REACHED_ONLY_THROUGH_WRAPPER_DEVICES_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW: &[&str] = &[」(`crates/singlefs-verification/src/judging_units_entries.rs:908`，27 条)、只打日志的 3 条（`crates/singlefs-verification/src/judging_units_entries.rs:1180`）与只经它们够到的 4 条（`crates/singlefs-verification/src/judging_units_entries.rs:1198`）。判决说的「两批」是显存记账与只打日志两张；包装设备那张的前提是够不够得到，由点名的测试守：「no_unit_the_gpu_judge_group_keeps_names_a_wrapper_type_it_does_not_follow」(`crates/singlefs-verification/src/judging_units_entries.rs:904`)，不读函数体。
- 显存记账那张的前提是行为：「都不改崩溃状态的判定字：设备丢失与显存不够的记账、」(`crates/singlefs-verification/src/judging_units_entries.rs:841`)。它的函数单元本职就改账本与借据，「不 panic、不改调用方的东西」的白名单判据套不上：实现员把 47 个登记名逐个对（Python 照白名单认法读 `gpu_memory_ledger.rs`，推的，我没复跑）：28 个没有函数体，1 个带泛型参数表，18 个函数单元每个都有名单之外的写法，接进守护今天就红。
- 先例：同一批在前一轮三方里判过。「没有会红的检查守着」(`research/prompts/m3-close-unit-graph-wrapper-impls-r3-main-verification.md:27`)，改法乙的结论里写了：「设备丢失那一类写明做不成」(`research/prompts/m3-close-unit-graph-wrapper-impls-r3-main-verification.md:37`)；欠账表里是 C1033，怎么拦那一格同样写了：「设备丢失那一类做不成，写明不做」(`.claude/kb/checks-owed.md:635`)。那份判决数的是 52 个登记名、今天登记 47 条，差的 5 条我没查（推的）。
- C1033 今天有一半已还：派活宽度的蜕变断言有了两条用例（「两个显存记账单元没有会红的检查」(`crates/singlefs-verification/src/crash_identity.rs:2333`) 点名了它），编内核透传签名那条没有：`grep -rn 'compile_for_the_work_of_this_thread' crates --include=*.rs` 去掉 `gpu_memory_ledger.rs` 只命中 `crates/singlefs-verification/src/judging_units_entries.rs:874` 与 `crates/singlefs-verification/src/gpu_unit_checks.rs:1128`、`:1144`，没有测试（量的）。C1033 那一行没随这一轮更新。
- 这一条的来历：辩方腿写明没读那份先例判决：「没读：显存记账那一批与包装设备那一批登记时的三方判决」(`research/prompts/judging-closure-and-real-card-instance-code-r2-sonnet-output.md:263`)，并写了「要不要扩到更早那一批，是主 agent 的取舍，我不替它判」(`research/prompts/judging-closure-and-real-card-instance-code-r2-sonnet-output.md:52`)；「对两批登记都跑」是判决自己的改法，判决条 7 没引这份先例。
- 判定：缺口（登记的前提没有读函数体的检查）站得住；改法够不着：判据不同，且前一轮已判「设备丢失那一类做不成」。今天该做的是更新 C1033、写明记账那一批留什么检查。推翻「够不着」的现象：另给一条今天过得了的、适用显存记账那一批的判据（例如只许改账本与借据、不碰判表与判定字），并有探针证它红。

### 认下 8 到 11 与不采 12、两步配对

- 条 8（:49）站得住。借用那一半的探针里，经共享借用记显存不够的那处改动在带 gpu 特性时编过、守护照绿（条 1 的读数）；「拷循环有条件的守」指的是摘要比对：「fn assert_the_watched_files_are_the_ones_the_build_read(&self) {」(`crates/singlefs-verification/src/crash_identity.rs:2966`)。
- 条 9（:50）：单元名那件做了钉格式的断言：「fn unit_names_in_the_compiled_compositions_keep_line_numbers_as_digits_after_an_at_sign_and_the_file_after_a_spaced_at_sign(」(`crates/singlefs-verification/src/crash_identity.rs:4061`)，错形态报红：「限定名里有 @ 后面紧跟的不是数字」(`crates/singlefs-verification/src/crash_identity.rs:4068`)。两步配对补的两件有用例：「fn a_unit_named_by_its_line_that_is_removed_or_added_is_named_and_one_that_only_moved_is_not() {」(`crates/singlefs-verification/src/crash_identity.rs:3954`)、「fn units_with_the_same_content_named_by_their_line_pair_off_one_for_one_and_the_name_left_is_one_of_them(」(`crates/singlefs-verification/src/crash_identity.rs:3991`)。
- 条 9 的 `checking_threads()`：今天产品代码里 4 处，被改的那处在「crate::crash_library::InnerThreadsTaken::take(checking_threads().min(image_count).max(1));」(`crates/singlefs-verification/src/crash_amplification.rs:34192`)，另三处是「checking_threads(),」(`crates/singlefs-verification/src/crash_amplification.rs:2495`)与「let threads = checking_threads();」(`crates/singlefs-verification/src/crash_amplification.rs:26265`)、同一句在 `crates/singlefs-verification/src/crash_amplification.rs:33188`；判决写的 :32438 又漂到 :34192。判决要求「转给管那一处的会话时给今天的行号」，调度记录里只有第一轮那条：「同一条消息告诉它 checking_threads() 另外 3 处调用没人核」(`records/2026-11-17-subagent账本清欠调度记录.md:264`)，`grep -n 'checking_threads' records/2026-11-17-subagent账本清欠调度记录.md` 只命中 263 与 264 两行（量的）。第二轮之后有没有发、带没带今天的行号，推不出；这一件判决的去处够不着。
- 条 10（:51）站得住：措辞没动：「变了的单元：（组成清单比不出：库里那一把或这一趟的组成清单是空表）」(`crates/singlefs-verification/src/crash_identity.rs:566`)；D13 的「名字、种子串、行号、资源上限、限时」(`.claude/kb/decisions/13-验证路线.md:436`) 一行还在，补的那句今天没有（`grep -n '登记名里的行号' .claude/kb/decisions/13-验证路线.md` 无输出），触发是第二轮判决回看决策那一行的「等第三轮收了再写」。
- 条 11（:52）站得住：D 行「拷树重算 带 gpu 特性编时另拷的 crates 之外的文件清单取成空的」(`crates/mutations.tsv:8221`)第五段是 `-p singlefs-verification --features gpu --lib --release -- crash_identity::tests::the_build_with_the_gpu_feature_reads_the_prune_server_library_files_outside_the_crates --exact`（`awk -F'\t' 'NR==8221 {print $5}'`，量的）。G 行「真卡取卡模块：没跑成的结局打成交了卡的那一行」(`crates/mutations.tsv:8224`)在合入后验证的变异门禁里有一趟日志「✓ 真卡取卡模块：没跑成的结局打成交了卡的那一行（本次未跑的记号行没打）：no_gpu_handed_over_reports_the_not_run_marker_line_with_the_test_name_and_the_reason 红了（现判：判法单元变了）」，同一趟「计数：抓到 3 条、没红 0 条、无效 0 条、锚点失效 0 行（共 3 条）」。最终那趟点名的 14 行里没有 G 行；G 的键是否因后来的 `process_identifier` 改名变了，我没查（推的）。
- 不采 12（:56）站得住：红句出路写了：「不为过这条守护改写法绕开名单」(`crates/singlefs-verification/src/crash_identity.rs:2266`)；攻方腿说的绕法，把代码挪进新小函数再登记，调用点一律出字样：「push_form(format!("{word}("));」(`crates/singlefs-verification/src/crash_identity.rs:1972`)，名单里没有的函数名一律红，改名单要过三方。推翻：有一处只靠白名单已有字样就把新函数体接进登记单元、而那个新函数体不受任何守护。
- 两步配对不漏报、停（:58-60）站得住：补的两件见条 9；名字归属那件只会多点名一个内容相同的单元，今天没有变，不会少点名。

## 第 6 问逐格

### 走不到的理由二次查实：成立

- 验证包两份源文件的改动都在 `mod tests` 里：`crash_identity.rs` 的 `mod tests {`（`crates/singlefs-verification/src/crash_identity.rs:930`）、守护机器从 `crates/singlefs-verification/src/crash_identity.rs:1584` 起；`crash_judge_gpu.rs` 的 `mod tests {`（`crates/singlefs-verification/src/crash_judge_gpu.rs:1193`）、`git diff HEAD -U0` 里它的改动块最小行号是 2534（量的）。`mod.rs` 与取卡测试文件在 `tests/` 下。
- 新加的 `expect` 与 `panic!` 各依赖什么：
  - `DisposableLibrary::new(…).unwrap_or_else(|refusal| panic!(…))`：`Err` 只在建不了一次性临时目录、临时 prune-server 起不来时有（那个函数文档的 `# Errors` 一段）；单测进程里名额满了是等、不是 `Err`。
  - 「.expect("开得了这条用例自己的一次性库")」(`crates/singlefs-verification/src/crash_identity.rs:2494`)、「.expect("读得了这条用例自己的一次性库")」(`crates/singlefs-verification/src/crash_identity.rs:2496`)、`crates/singlefs-verification/src/crash_judge_gpu.rs:2685` 同一句：依赖那份临时服务在用例结束前还活着；它由 `disposable_library` 持有到函数尾（`drop(disposable_library` 在两份文件里零命中，量的）。
  - `u64::try_from(cpu.len()).expect("装得进 u64")` 与 `usize::try_from(state_count).expect("装得进 usize")`：状态数上限是 1000：「const MOST_STATES_OF_THE_DISPATCH_WIDTH_FLOW: u64 = 1000;」(`crates/singlefs-verification/src/crash_identity.rs:2325`)。
  - 判定里的 `assert!`：「not_handed_over.is_empty(),」(`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:677`)，从打行那一层原样挪进判定。挪动的副作用：`GPUS_THIS_PROCESS_IS_ON` 那一行以前先打到标准输出再 panic，今天写在 panic 的话里；`scripts`、`task-server`、`gpu-verification-server`、`.claude/gate.d`、`.claude/hooks`、`.claude/scripts`、`prune-server` 里搜这个词零命中（量的），没有读者。
  - 守护测试的现造样本：`.unwrap_or_else(|reason| panic!("现造的函数认不出（{reason}）：{source}"))` 只在现造样本写错时走（`crates/singlefs-verification/src/crash_identity.rs:2096`）。

### 类型与名字：成立

- 命名两道在合入快照上绿：日志「✓ 命名纪律通过：查了 1071 个 .rs 文件、316704 个声明的名字」，门禁四格日志「4 格里 4 格绿」（量的）。
- 字符串约定又多了：「let Some((module, function_name)) = listed_unit.rsplit_once("::") else {」(`crates/singlefs-verification/src/crash_identity.rs:2228`)、按模块名字符串找源码的表（「include_str!("gpu_unit_checks.rs"),」(`crates/singlefs-verification/src/crash_identity.rs:1589`)）、名单里的 `static ORIGIN` 字样（`crates/singlefs-verification/src/crash_identity.rs:1833`）。它们对不上时都是红，不放过；第二轮判决把同一形状记成「推不出」：「守护里又多一处按 :: 拆限定名，属第一轮记账的那一类」(`research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md:88`)。我按命名纪律本身判成立，这一条限度照同样记账。

### 不兼容旧代码与旧格式：成立

- 五份文件新加的行里没有这类字样：`git diff HEAD -U0 -- <五份文件> | grep -E '^\+' | grep -niE 'legacy|compat|deprecated|migrat|版本号|旧版|旧格式|兼容|迁移|v1|v2'` 零输出（量的）。格式断言对不认得的名字报红、不放宽：「限定名里有 @ 后面紧跟的不是数字」(`crates/singlefs-verification/src/crash_identity.rs:4050`)。

### 不新建判定库：成立

- 一次性库的定义：「不读连哪一处的环境变量、不连真服务：测试写不到真服务里去」(`crates/singlefs-verification/src/crash_library.rs:527`)，是它自己在这个进程里起的临时 prune-server 里的一段名字空间。共用约束写明不算：「并回即删的中转库与对临时服务起的自证库不算」(`.claude/agent-common.md:29`)。复现那趟的日志里两条对拍各开一份：`disposable-library=disposable-crash-library/dispatch-width-pairing/…` 与 `disposable-library=disposable-crash-library/accounting-slice-pairing/…`，socket 在本进程的目录下（`singlefs-prune-server-<进程号>-<号>/socket`，量的）。
- 限度：拿掉守卫的两行变异（「GPU 派活宽度对拍：判表过库与 CPU 参照取样前不把这条用例的一次性库设成这个进程开的那一份」(`crates/mutations.tsv:8378`)与同表 `crates/mutations.tsv:8379`）回到「单跑时开到全仓那一份」，实现员写明那一趟判表与取样结论写进了真服务。变异跑在任务里时「全仓那一份」是哪一份、写进了谁的库，我没读到，推不出；这是第 4 问，攻方腿判。

### 所有任务多机可用：成立

- 守护的输入编进二进制（上一格的 `include_str!`）；取卡测试的输入在用例里现造：「const THIS_HOST: &str = "192.0.2.1";」(`crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs:27`)；一次性库建在 `std::env::temp_dir()`：「Self::new_under(label, &std::env::temp_dir())」(`crates/singlefs-verification/src/crash_library.rs:746`)。
- 限度：读数（复现那趟 3 条里 2 条绿）状态行是第二台；本机上的结果是推的，代码里没有读本机独有状态。

### 测试优先多线程：推不出

- 写锁那一半对：「设着的那一段只罩这两处、不罩借卡」(`crates/singlefs-verification/src/crash_identity.rs:2448`)，守卫在块尾（第 2490 行）丢掉。但块里第二处是 `sample_crash_states_on_the_cpu` 把整条流在 CPU 上判一遍：「let Some(judged_on_the_cpu) = sample_crash_states_on_the_cpu(」(`crates/singlefs-verification/src/crash_identity.rs:2464`)，至多 1000 个状态，都在写锁里；同一个库内测试进程里设一次性库的用例有 40 处（`grep -rn 'make_it_the_library_of_this_process()' crates/singlefs-verification/src --include=*.rs | grep -v '///' | wc -l`，量的，含 1 处 `bin/`），它们要排在这一段后面。常量文档写这一条流「单跑几秒」：「几百个以内的小流，单跑几秒」(`crates/singlefs-verification/src/crash_identity.rs:2323`)。
- 名额那一半没人提：一次性库在单测进程里占一份临时服务名额，满了就等：「let place_in_the_test_process = TemporaryServerPlaceInTheTestProcess::take();」(`crates/singlefs-verification/src/crash_library.rs:756`)、「while in_use.temporary_servers >= temporary_servers_of_the_share {」(`crates/singlefs-verification/src/crash_library.rs:706`)。份额：「let temporary_servers = (heavy_threads / 2 / THREADS_OF_A_TEMPORARY_SERVER).max(1);」(`crates/singlefs-verification/src/crash_library.rs:567`)，一份服务常驻线程数「3 + prune_server::server::WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS + 2;」(`crates/singlefs-verification/src/crash_library.rs:555`)、「pub const WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS: usize = 2;」(`prune-server/src/server.rs:1038`)，每台默认重活线程数 31：「所有重活默认每台 31 线程」(`.claude/heavy-thread-default:7`)，所以是 31 ÷ 2 ÷ 7 取整得 2 份（推的：算术，没跑）。
- 两条对拍的名额从建库一直占到用例结束：「let disposable_library = DisposableLibrary::new("dispatch-width-pairing")」(`crates/singlefs-verification/src/crash_identity.rs:2449`)、「let disposable_library = DisposableLibrary::new("accounting-slice-pairing")」(`crates/singlefs-verification/src/crash_judge_gpu.rs:2640`)；中间借卡等别的线程还卡：「let Some(lease) = cards.lease_card_or_report_not_run(」(`crates/singlefs-verification/src/crash_identity.rs:2505`)。借的卡有 `drop(lease)`：「drop(lease);」(`crates/singlefs-verification/src/crash_judge_gpu.rs:2713`)；一次性库没有，`drop(disposable_library` 在两份文件里零命中（量的）。过库之后的断言一做完这份库就不再用（我读到的范围）。
- 读数：复现那趟三条测试一起跑，日志「test result: FAILED. 2 passed; 1 failed」，两条对拍 ok，没有「No such file or directory」（量的）；没有挂住的读数。两条对拍同时占满名额时别的设一次性库的用例等多久，没量（推的）。
- 规则 `.claude/rules/implementation-workflow.md@测试与崩溃检测优先多线程`：「写法：切片并行、线程数、合并要确定、报进度、不能并行的写明为什么」。写锁那条「不能并行」写了为什么，名额这一条没写。
- 判定：推不出（算打中）。推翻的现象：在一个进程里放这两条对拍加若干条别的设一次性库的用例，分别在现在的树与断言之后 `drop(disposable_library)` 的副本上跑，别的用例的等待时长相同。

### 带 --item 的命令与项：成立

- 合入后验证的变异门禁日志：「计数：抓到 14 条、没红 0 条、无效 0 条、锚点失效 0 行（共 14 条）」；「复用：现判 1 条、复用 13 条、拒算 0 条、本次未跑 0 条」（量的）。那一条现判是复用哨兵强制现判的，不是新行现判；两条新加的对拍行在写段证红里现判过：「✓ 点名 2 条：现判 2 条、复用 0 条、拒算 0 条；跑了 2 条，抓到 2 条（其中复用 0 条），本次未跑 0 条，跑了的都抓到了」。
- 14 的来历：表里那 15 个变异名各恰好一行（量的），11 追加加 3 改写是 14，不含 G 行（条 11）。

### 新崩溃流多罩了什么：成立

- 两条对拍不在登记的崩溃枚举用例与放量分项里：`grep -c 'verdict_words_of_every_state_are_the_same_at_a_dispatch_width\|verdict_words_of_the_new_pool_file_creation' .claude/gate.d/stage-inputs.tsv scripts/crash/crash-amplification.sh` 两个文件各 0（量的）；两份源文件里不读 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 与 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（零命中，量的）。它们自己的展开上限是测试内常量：「const EXPAND_UP_TO_IN_THE_DISPATCH_WIDTH_FLOW: usize = 3;」(`crates/singlefs-verification/src/crash_identity.rs:2329`)，不是崩溃放量的域。

## 同类先例（贴出处）

- 条 7 的先例是前一轮三方对同一批的判决与它登的账：`research/prompts/m3-close-unit-graph-wrapper-impls-r3-main-verification.md:27`、`:37` 与 `.claude/kb/checks-owed.md:635`（引文见条 7）。
- 条 5 的「推后要另登一行」：`.claude/main-agent.md:25` 与 `.claude/gate.d/deferral-registries.tsv:3`（引文见条 5）。
- 条 2「等价形态、今天判不出、写明不造测试」的先例：C1033 那一格「设备丢失那一类做不成，写明不做」是同一种处理；别的我没找到，不用重新论证顶替。

## 第 6 问各格推翻的现象

- 走不到的理由：新加的 `expect` 在测试里红了，且红点落在产品代码的路径上。
- 类型与名字：合入后的树上命名两道红，或守护与格式断言的字符串约定对不上时仍绿。
- 不兼容旧代码与旧格式：五份文件新加的行里出现认旧版本、迁移或放宽的分支。
- 不新建判定库：一次性库连的是真服务，或丢掉之后还留着行。
- 所有任务多机可用：本机跑这两条对拍与取卡测试的结果与第二台不同。
- 带 --item：最终那趟日志里「复用」的 13 行，复用到的那次现判与这一趟键不同（日志里「现判的按栏」非 0）。
- 新崩溃流：这两条对拍出现在 `stage-inputs.tsv` 的 `crash-case:` 行，或放量入口的 `ITEMS`。

## 要主 agent 跑的命令

无。判决要引的数都是读文件与读日志得到的，没有要编、要跑才量得出的项。下面是我读数用的只读命令，主 agent 现查时可原样再跑（不交任务）：

```
grep -n '薄壳' .claude/kb/checks-owed.md
grep -rn 'drop(disposable_library' crates/singlefs-verification/src/crash_identity.rs crates/singlefs-verification/src/crash_judge_gpu.rs
grep -rn 'NOT_RUN_NO_GPU_ADAPTER test={test_name}' crates/singlefs-background-gpu-tests --include=*.rs
grep -n 'checking_threads' records/2026-11-17-subagent账本清欠调度记录.md
sed -n 848,906p crates/singlefs-verification/src/judging_units_entries.rs | grep -c '^    "'
```

条 6 的多线程一格要量名额占用，得在副本里加一句 `drop(disposable_library)` 做前后对照，是一次实验，不是这一轮判决要引的数，没写命令。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没做第 1 到 4 问（攻方腿的活）：没逐样证明白名单 37 样的放行理由，没试切字样的边角，只在条 6 读到 `format!` 那一条；没判拿掉守卫的变异写哪一份库。
- 没跑单元图 `why`（条 4 的「在这一组闭包里」）；没复跑实现员的 Python 镜像（条 7 的 28 / 1 / 18）；没读 `polled` 那个类型的 Debug 实现（条 6）；没查先例的 52 与今天 47 差的 5 条（条 7）；没核 G 行的键是否因 `process_identifier` 改名变了（条 11）；名额数（条 6 多线程）是算术，没跑。
- 没读 `gpu_memory_ledger.rs` 的函数体，没读欠账表别的行，没读禁读清单里的文件。
- 日志读数来自 `task-server/task status` 与任务服务日志目录里的文件（只读）；合入后验证快照已删，那趟日志里「现判的按栏」只读了计数行。

## 清理

没建仓副本与编译目录；草稿目录 `~/.local/share/singlefs/work/code-r3-judging-closure-defense` 里只有我写报告用的几份小文件，交回前删。
