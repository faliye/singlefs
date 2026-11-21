# 判器闭包不沿只打日志的单元收、真卡测试一个进程共用一个实例、数据文件清单那条测试的修补：代码轮第一轮（2026-11-18）

这一轮攻的是已经合进主工作区的一批 `crates/` 改动（打补丁任务 19768、回读 19769），快照取在合入之后。方案按 `crates/` 今天的实现来谈。

## 范围

被判的四份文件与一张表：

- `crates/singlefs-verification/src/judging_units_entries.rs`
- `crates/singlefs-verification/src/crash_identity.rs`
- `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs`
- `crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs`（新）
- `crates/mutations.tsv`（追加 16 行、整行替换 1 行；变异名在三份实现员报告里逐条列着）

三份实现员报告（改了什么、每条测试配哪一行变异、证红期望）：

- `~/.local/share/singlefs/work/ledger-topdown-judging-closure-writer-second/result.md`
- `~/.local/share/singlefs/work/ledger-topdown-real-card-one-instance-writer/result.md`
- `~/.local/share/singlefs/work/ledger-topdown-watched-data-files-repair/result.md` 与同目录 `result-second.md`

压着的条款：`.claude/kb/decisions/13-验证路线.md` 的 D13（验证路线） 已定项 17（判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成）；`.claude/rules/verification.md`「一个门禁：两种跑法」里单元键那一条与「卡只归 GPU 服务，任务不拿卡」。

## 实现今天的样子

主 agent 的观测（读的是合入之后主工作区的文件）：

- `crates/singlefs-verification/src/judging_units_entries.rs:1178` 新常量 `UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW`（只打 GPU 阶段现场那一行的三个单元），`:1196` 新常量 `UNITS_REACHED_ONLY_THROUGH_THE_UNITS_THAT_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW`（只经那三个够得到的四个单元，其中一个名字带行号 `enum_listing_every_member!@78`）；`:1207` 的 `units_not_followed_of_the_group` 把这两批连同已有的两批一起交给 GPU 判器那一组。
- `crates/singlefs-verification/src/crash_identity.rs:483` 的 `units_changed_between` 改成两步配对（先配名字与指纹都相同的，再配去掉限定名里 `@<数字>` 之后名字相同、指纹也相同的），新私有函数在 `:510`、`:549`。测试模块里新加：`:1463`、`:1548`、`:1561`、`:1573`、`:1587` 五条挂 `gpu` 特性、读活树的测试；`:3097` 一条不挂特性的；`:1954` 的纯函数 `watched_files_outside_the_crates_and_the_root_manifests` 与 `:1980`、`:2002`（`cfg(not(gpu))`）、`:2016`（`cfg(gpu)`）三条测试，替掉了原来断言「清单里有 `.claude/heavy-thread-default`」的那一条。
- `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:676` 新的进程级 `static VULKAN_INSTANCE_OF_THIS_TEST_PROCESS: OnceLock<…>`，`:702` 的 `handed_over_gpus_through_the_instance_kept_for_this_process` 收一个 `&OnceLock<Instance>`；同文件 `:664` 已有先例 `CARDS_LEASED_ON_DEMAND_FOR_THIS_PROCESS`。新测试文件三条不要卡的测试（`:89`、`:147`、`:170`）拿一个数建了几次的替身当实例。
- 量到的读数：合入后验证 19725「测试全绿（8 个测试二进制，共 15 条测试…按条复用：现判 4、复用 1024、拒算 11）」，带 gpu 特性的 lib 目标「FAILED 的测试函数 27 条」与合入之前的基线逐条相同（都是任务里拿不到卡的那一类，不是这一批带的）；17 行变异在快照上 19748「抓到 17 条、没红 0 条、无效 0 条」；快照上量的判器键表 19764：`group gpu_judge` 从 27de3bf2… 变成 a051d9cd…，`judgement_of_the_crash_images`、`observer_conclusion_composition` 没变。
- 挂 `gpu` 特性、读活树的那五条新测试没登进拒算表：拒算表的清单只取不带特性那一趟的测试函数（任务 19740「拒算 12 条、登记表 12 行」里没有它们），同文件已有的同类挂特性测试也不在表里。

## 要腿判的

1. 「不沿它收」的两批登记对不对：三个只打日志的单元（`gpu_lifecycle_line`、`gpu_lifecycle_trace`、`gpu_work_trace`）真的不改任何崩溃状态的判定字吗——它们交不交值、交的值有没有被留下的单元读到、有没有副作用（写全局、改卡的状态、panic 的路径）会改判定结局。四个「只经它们够得到」的单元（`<impl RecordedOperationKind>`、`<impl RecordedOperationKind>::name`、`enum_listing_every_member!@78`、`history::enum_listing_every_member`）是不是真的只经那三个够得到：实现员说两条进来的边都是同名撞上（函数体里的 `use std::io::Write` 连到成员里写着 `Write` 的宏调用；`thread.name()` 让方法名 `name` 算见过），这是读 `crates/singlefs-unit-graph/src/closure.rs` 推的、没跑闭包打边。判这两句成不成立；判「把它们登进不沿它收」会不会把留下的单元真要的东西切掉（键过窄）。
2. 带行号的登记名（`enum_listing_every_member!@78`）：`lib.rs` 那次宏调用挪一行，这条登记就成陈旧的。实现员说届时 `not_following_the_listed_units_drops_only_the_listed_ones` 当场红。判这句成不成立，以及陈旧登记在那条测试红之前会不会先让键悄悄变宽或变窄。
3. `units_changed_between` 的两步配对：只挪行的单元不再报，会不会把「挪了行又改了内容」「两条同名单元互换了内容」「同一名字去行号后撞名」这些情形配错、漏报真变了的单元。红句是给人判哪几个单元变了用的，漏报算打中。
4. 数据文件清单那条测试的修补：原测试断言清单里有 `.claude/heavy-thread-default`，改成了（a）对现造清单钉过滤式、（b）不带特性时清单为空、（c）带 gpu 时清单里有 `prune-server/src/lib.rs` 且都在 `prune-server/src/` 底下。前提是「线程数只决定盘切成几段、不改判定结论」（实现员读代码推的）。判这个前提；判改完之后「拷树重算漏拷 `crates/` 之外的数据文件就拒算」这件事还有没有东西守着（实现员自己写了：删掉拷数据文件那个循环，今天两种编法下都不会红）；（b）（c）两条把今天的清单长什么样钉死，别的会话合一批让清单变了它们就红，这是该有的警报还是会成为噪声。
5. 真卡测试共用实例：进程级 `OnceLock` 是一个单例，`.claude/rules/code-discipline-by-review.md`「类型、数据与封装」留下了「不用全局可变状态；不用单例」。照同文件已有先例做的。判这一处该不该有；判三条不要卡的测试罩没罩住真改动——实现员自己写了没罩到的一处：薄壳里交进程级那一格的地方若改成每次新建一格，这三条不红。
6. 八格检查单逐格判（下面每格是这一轮的答案与证据，判它成不成立）。

### 检查单：走不到的理由二次查实

三份实现员报告的「走不到的理由」一节都是没有：补丁只改验证包的 `src/`（`crates/singlefs-verification/src/judging_units_entries.rs`、`crash_identity.rs`）与产品 GPU 测试包的 `tests/`，产品代码一行没动，没有新加的 `todo!`、`unreachable!`、产品路径上的 `assert!`，也没有条款没定的错误成员。`units_changed_between`（`crash_identity.rs:483`）是验证包里的库函数，里面有没有走不到的分支、`expect`，腿自己读一遍核。

### 检查单：类型与名字

新名字：两个常量（`judging_units_entries.rs:1178`、`:1196`）、两个私有函数 `pair_off_units_with_the_same_compared_name_and_fingerprint`（`crash_identity.rs:510`）与 `unit_name_without_line_numbers`（`:549`）、测试模块里的纯函数 `watched_files_outside_the_crates_and_the_root_manifests`（`:1954`）、`static VULKAN_INSTANCE_OF_THIS_TEST_PROCESS`（`mod.rs:676`）、函数 `handed_over_gpus_through_the_instance_kept_for_this_process`（`mod.rs:702`）、新测试文件里的 `InstanceStandInNumberedByItsCreation` 等替身类型与九条新测试名。命名检查在快照上过了（门禁 code-source-discipline 的四格 19746、naming-lint 19747）。单元名在代码里是字符串（限定名），没有新立 newtype：它们本来就是字符串键，这一轮没改这一点。每处 `expect` 依赖哪条不变量，腿逐处读。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：没有认旧版本的分支、旧布局号、迁移路径。查过的范围是这四份文件的 diff。GPU 判器那一组的键变了一次（27de3bf2… → a051d9cd…），旧键下的判定不迁移、下一趟现判；GPU 服务已照新键重编换上。

### 检查单：不新建判定库

本轮改动没有这一类：没有新的判定存放处。新测试里拷活树进临时目录重算键（`crash_identity.rs` 里 `:1548` 一带的三条），临时目录用完即删，不存判定。

### 检查单：所有任务多机可用

新测试不读本机独有状态：挂 `gpu` 特性的五条读的是仓里的源文件与编进二进制的键表；产品 GPU 测试包的三条新测试不要卡、输入在用例里现造（替身实例、现造的两张卡的总线号）。`mod.rs` 里取卡照旧只认服务交的卡。腿核一处：`crash_identity.rs:2002`、`:2016` 两条断言的清单内容随编法变，两台的编法相同时结局相同；有没有哪台的路径、主机名进了断言。

### 检查单：测试优先多线程

新加的都是单元测试，每条一个场景、毫秒到几秒：三条拷活树重算键的各自拷一份、互不共用状态，可以由测试框架并行；没有新的崩溃检测循环。没有写死线程数的地方。

### 检查单：带 --item 的命令与项

三份报告里带 `--item` 的只有验证段在快照上点名变异行的那一条（门禁 crates-mutation-replay，17 个 `--item`，任务 19748），范围声明与 `<根>` 都写了。没有新加崩溃枚举用例与放量分项，所以没有新项要进 `--list-items`。合入后验证的 8 个测试目标项由 `--list-items` 列出（任务 19724）。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放流或崩溃枚举用例。
