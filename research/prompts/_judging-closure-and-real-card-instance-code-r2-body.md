# 判器闭包与真卡测试共用实例那一批：代码轮第二轮，攻第一轮判决的修补（2026-11-18）

这一轮攻的是第一轮判决（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md`）「三、改法」第 1 到 5 条的修补，已经合进主工作区（打补丁任务 20008、回读任务 20010），快照取在合入之后。方案按 `crates/` 今天的实现来谈。第一轮攻过而没打中的角度（两步配对会不会漏报真变了的单元）不重复。

## 范围

被判的三份文件与一张表：

- `crates/singlefs-verification/src/crash_identity.rs`（只动 `#[cfg(test)] mod tests`）
- `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs`
- `crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs`
- `crates/mutations.tsv`（追加 9 行、删 1 行；变异名在实现员报告里逐条列着）

实现员报告（逐条做了什么、每条测试配哪一行变异、认得出与认不出的写法）：`~/.local/share/singlefs/work/code-r1-judging-closure-repair/result.md`。

压着的条款：`.claude/kb/decisions/13-验证路线.md` 的 D13（验证路线） 已定项 17；第一轮判决「二、判决」「三、改法」。

## 实现今天的样子

主 agent 的观测（读的是合入之后主工作区的文件）：

- `crates/singlefs-verification/src/crash_identity.rs:1887` 新测试 `no_listed_unit_that_only_prints_the_gpu_lifecycle_line_can_panic_or_change_what_its_caller_holds`：被判的三个单元的源码经 `include_str!` 编进来，按文字判函数体里有没有会 panic 的写法、参数里有没有可变借用。它认得出与认不出的写法实现员逐类列了（报告第 1 条）：认不出的有移位、调到的函数内部的 panic、宏展开、经共享借用改内部可变的类型、参数之外的静态项。
- `:2680` `RepositoryCopy::of_the_repository`（仓根与另拷清单都是参数），`:2762` 新测试 `copying_the_tree_copies_each_listed_file_outside_the_crates_to_the_same_path_under_the_temporary_root` 在现造的小仓上核清单里那一份被拷到；`:2494` 带 gpu 特性的清单测试只留「清单里有 `prune-server/src/lib.rs`」；`:3636` 已有测试 `units_named_by_their_line_that_only_moved_are_not_named_as_changed_and_edited_ones_are` 多了一行点它的变异。
- `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:750` `not_run_line_of_the_test`、`:756` `handed_over_gpus_line_of_the_test`、`:769` `line_reporting_the_gpus_taken`，薄壳 `:791` `handed_over_gpus_or_report_not_run` 只剩打行；`:681` 进程级的 `static VULKAN_INSTANCE_OF_THIS_TEST_PROCESS`。新测试文件 `:203`、`:227`、`:248`、`:272` 四条不要卡的测试。同一模块里 `gpus_this_process_is_on_judged_against_the_handed_over` 另有两处记号行的打印没改用新的拼行函数（实现员写明：被两行旧变异锚住），记号行的拼法现在有两处。
- 量到的读数：写段点名测试 19873「test result: ok. 4 passed」、19874（带 gpu 特性）「test result: ok. 3 passed」、19875「test result: ok. 4 passed」；证红 19876、19921 各「跑了 4 条，抓到 4 条」；合入后验证 19945 退 1，红的只有带 gpu 特性的 lib 那 27 条，按测试名与上一批存下的名单逐条相同（任务里拿不到卡的那一类）；快照上点名 9 行变异 19955「计数：抓到 9 条、没红 0 条、无效 0 条、锚点失效 0 行」；合入后判器键表 20013 `group gpu_judge` 仍是 a051d9cd51cc337e…。
- 没罩住、实现员写明的两处：薄壳交进去的是不是进程级那一格、`println!` 打没打出拼好的那一行（要在真卡上量）；`RepositoryCopy::new` 里交清单那一处换成空清单，不带特性时没有东西判得出差别。

## 要腿判的

攻方腿攻修补本身，辩方腿复核第一轮判决。

1. 守护测试（`crash_identity.rs:1887`）按文字认 panic 写法：它认不出的那几类里，今天这三个单元真用到了哪几样（例：格式化宏里调到的 `Display`、`gpu_work_trace` 的静态项、锁）；有一样用到了而认不出，这条守护对它就是空的。再判反方向：它把不会 panic 的写法判红的（整数算术、下标）会不会让以后改这三个单元的人为了过测试去改登记而不是改代码。
2. 拷树那一条（`:2762`）：现造小仓上测的是 `of_the_repository`，真用的是薄壳 `new`。薄壳里交错清单、交错仓根，哪条测试会红；实现员自己写了不带特性时判不出，判这个缺口要不要一条变异或用例钉住。
3. 真卡共用模块的四条新测试：逐字钉整行的三条，与任务服务、门禁读这一行的认法是不是同一个格式（`NOT_RUN_NO_GPU_ADAPTER test=<名> reason=<…>`）；另两处没改用拼行函数的打印与新函数拼出的行会不会漂开，有没有东西守着它们相同。`unreadable_gpus_of_this_host_are_judged_red_before_the_instance_is_created` 用替身数建了几次实例，判它证明的是不是「先读后建」。
4. 变异行：9 行各自改坏的地方与名字说的是不是一回事；G 行（没跑成的结局打成交了卡的那一行）与 F 行点同一条测试，判它们是不是两处不同的改法。删掉的那一行旧变异，它原来守的东西现在由哪一行守。
5. 辩方腿：第一轮判决认下而没改的四件（带行号的登记名的代价、红句措辞、单元名只在字符串里约定、`checking_threads()` 另外 3 处调用没人核）与它撤回实现员的两句，判第一轮的判法够不够得着；第一轮判「两步配对不漏报、停」这一格只凭两条腿各一次，判这个停成不成立。
6. 八格检查单逐格判。

### 检查单：走不到的理由二次查实

实现员报告「走不到的理由」一节是无：产品代码一行没改，验证包只改测试模块，产品 GPU 测试包只改 `tests/`。新加的 `expect`、`unreachable!` 在测试代码里有没有、依赖什么，腿自己读一遍。

### 检查单：类型与名字

新名字：`RepositoryCopy::of_the_repository`（`crash_identity.rs:2680`）、三条新测试名（`:1887`、`:2762` 与改过的 `:2494`）、`not_run_line_of_the_test`、`handed_over_gpus_line_of_the_test`、`line_reporting_the_gpus_taken`（`mod.rs:750`、`:756`、`:769`）、新测试文件里的四条测试名与替身的辅助函数。命名检查在快照上过了（门禁 code-source-discipline 四格 19952、naming-lint 19953）。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：没有认旧版本的分支、旧布局号、迁移路径。查过的范围是三份文件的补丁。

### 检查单：不新建判定库

本轮改动没有这一类。新测试 2 在临时目录里现造小仓，用完即删，不存判定。

### 检查单：所有任务多机可用

新测试不读本机独有状态：守护测试的输入编进二进制；拷树那一条在临时目录现造；真卡包四条的输入（两张卡的总线号、取卡结局）在用例里现造。证红两条一条派在本机、一条派在第二台，都抓到 4 条（19876、19921）。

### 检查单：测试优先多线程

新加的都是单元测试，每条一个场景，各自建临时目录或只读编进来的源码，互不共用状态，可以由测试框架并行。没有写死线程数的地方。

### 检查单：带 --item 的命令与项

带 `--item` 的是验证段在快照上点名 9 行变异的那一条（任务 19955，范围声明与根都写了）与合入后验证点名的 8 个测试目标（列项任务 19937）。没有新加崩溃枚举用例与放量分项。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放流或崩溃枚举用例。
