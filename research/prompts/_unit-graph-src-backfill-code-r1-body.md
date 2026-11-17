# 函数单元图 src 在途改动的代码轮（补轮）

日期：2026-11-16。

## 范围

被判的是 `crates/singlefs-unit-graph/src/` 下十六份文件工作区相对 HEAD 的改动（清单在快照的 `crates-sha256.txt`）。材料都在 `research/prompts/unit-graph-src-backfill-code-r1-snapshot/`：

- `changes.patch`：`git diff HEAD -- <这十六份>` 的原样输出（2151 行）。
- `crates-sha256.txt`：十六份文件开工时的 sha256。
- `facts.md`：事务员出的逐文件事实表。它是线索：`closure.rs` 的逐块一句是脚本从 diff 里取的「函数上下文加首个加行」，没有逐块通读。
- `mutation-rows-new.tsv`：`crates/mutations.tsv` 里指到这十六份、相对 HEAD 新出现的 80 行。`crates/mutations.tsv` 整份另开一题判，这一轮只判这 80 行罩不罩得住下面几件改动。

`crates/singlefs-unit-graph/tests/` 下有 21 份在途改动的测试文件，另开一题判；这一轮的腿可以读它们来核 src 的改动有没有测试，不逐份判它们。

方案按 `crates/` 今天的实现来谈。

## 实现今天的样子（主 agent 的观测）

读过 `crates/singlefs-unit-graph/src/build_toolchain.rs:12-37`、`crates/singlefs-unit-graph/src/closure.rs:1118-1126`、`grep -n OwnProgramRestart crates/singlefs-unit-graph/src/*.rs` 的输出与 `changes.patch` 的文件清单。其余几件照事务员的概述列，腿自己读源文件核：

1. **进键的 profile 设置由 8 项缩成 6 项**（`build_toolchain.rs:16-37`）：`PROFILE_SETTING_KEYS` 去掉 `debug` 与 `incremental`，`builtin_profile` 四条内置值各少两个元素；注释说这两样不改编出来的代码跑出的结果、根清单与 `CARGO_PROFILE_<名>_<键>` 里写了也不读，`codegen-units` 进键，依据是 D13（验证路线） 已定项 17「工具链（含 cargo 解析后的生效构建环境与外部工具版本）」那一句。这一件改的是单元键里工具链那一段的内容。
2. **裸名只是函数局部绑定时不连同名函数**（`closure.rs` 新增约 356 行：`function_signature_positions`、`parameter_binding_position`、`is_statement_let_binding_at`、`is_bare_name_only_a_local_binding_of_the_function`；规则开关 `LocalBindingsShadowSameNamedFunctions` 在 `rule_switches.rs`，`EVERY_RULE` 长度 28 变 29）。这一件让闭包变小：闭包少收一个真引到的单元，键就漏了输入。
3. **use 路径点准候选**（`closure.rs` 的 `use_path_leading_to`）。同样让闭包变小。
4. **程序起自己**（`rust_items.rs` 新增约 235 行：`expression_ending_at_diverges`、`own_path_expression_range`、`is_command_new_open`、`own_program_path_is_handed_to_command_unchanged`；`units.rs:281` 新成员 `SourceRead::OwnProgramRestart`；`closure.rs` 的 `finish` 加 `entry_is_the_main_of_a_program` 与拒算）：`current_exe()` 的路径原样交给 `Command::new` 的，由就地拒算改成只在入口是程序的 `main` 时怎样处理，腿读 `closure.rs:44-50` 的头注释与 `finish` 核它到底怎么判。
5. **extern 块只声明登记的系统库调用**（`rust_items.rs` 的 `extern_block_declares_only_listed_system_library_calls` 与常量 `SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS = ["pwrite", "fdatasync"]`）。
6. **首边与 why 子命令**（`closure.rs` 的 `FirstEdgeIntoTheClosure`、`record_the_first_edge`；`output.rs` 的 `why_lines`、`unit_position_text`；`bin/unit_graph.rs` 的 `why` 子命令与 `--unit`）：只为解释一个单元为什么进闭包，不该改闭包与键。
7. **不沿点名单元收的入口组键**（`closure.rs` 的 `compute_closure_of_entries_not_following_the_listed_units`；`entry_groups.rs` 的 `EntryGroupKeyNotFollowingTheListedUnits`、`entry_group_key_not_following_the_listed_units`，`entry_group_key_with_the_features_of_the_build` 的返回类型换成新类型）。
8. `build_features.rs`：形参改名、一段注释、两条读产品 GPU 包清单与构建脚本的新测试。
9. 一批纯改名（测试函数去 `a_` 前缀，`os` 改 `operating_system` 这一类），落在 `conditional_compilation.rs`、`lexer.rs`、`lock_file.rs`、`manifest.rs`、`shell_sources.rs`、`verus_ghost_code.rs`、`wgsl.rs`。

键算法另有一份 Python 实现 `scripts/gate/lib_function_units.py`，两份要算出同一个键（门禁 code-tooling 的 function-units-two-implementations-agree 格）；`rust_items.rs:54` 的注释自己说与它的 `SPAWNING_WORDS`「同一份」。Python 那一份不在这一轮被判的文件里，腿读它只为核两份对不对得上。

## 要两条腿答的

- 正推：第 1 到 7 件各自做的是不是 D13（验证路线） 已定项 17 说的（单元键由判定依赖的函数单元闭包、闭包真引到的 cfg 原子、工具链、依赖版本组成；条款整条现读，引 kb 文件自己的行号）；第 9 件是不是纯改名。
- 攻方：哪一格会错。闭包只许多收、不许少收：第 2、3 件各构造一段 Rust 源码，让新规则判成「不连」而运行时真调到那个同名函数（宏体里的名字、闭包捕获、模式里的常量与元组结构体、嵌套函数与嵌套块里的遮蔽、`let` 之前的使用、形参与泛型同名、方法调用与关联函数）；第 4 件构造一段起自己而没被拒算、也没被收进闭包的写法；第 5 件登记名单之外的 extern 声明怎么判、名单里两个调用凭什么不拒；第 1 件 `debug` 与 `incremental` 有没有改运行结果的路（`debug` 影响 `cfg(debug_assertions)` 吗、影响 panic 信息与回溯里测试断言读到的东西吗、`split-debuginfo`、`strip`、`rpath` 这类没进键的设置凭什么不进）；第 6 件会不会改到闭包或键；第 7 件「不沿点名单元收」的键被谁用、少收的那一截凭什么不进键；Rust 与 Python 两份实现在这几件上还一致不一致；新加的 80 行变异罩不罩得住每条新规则的每个判定分支。
- 两条腿都逐格判下面的检查单，回「成立」「不成立」或「推不出」。

### 检查单：走不到的理由二次查实

这批改动没有实现员报告（补轮），照 `facts.md` 各文件第 3 样列：生产段新加的 `expect` 四处，都在 `closure.rs` 的绑定位置扫描里——`closure.rs:1169`「最外面那一层不出栈（下面的出栈判了长度），栈不空」、`closure.rs:1203`「上面只在长度大于 1 时出栈，栈不空」、`closure.rs:1210` 与 `closure.rs:1231`「最外面那一层不出栈（上面的出栈判了长度），栈不空」。腿逐处追出栈的每条路，判「栈不空」成不成立。别的文件生产段没有新加 `unreachable!`、`panic!`、`expect`、`unwrap`、`assert!`（事实表按 `mod tests` 之前机械划分生产段，腿在 `changes.patch` 里再数一遍）。

### 检查单：类型与名字

新加的名字列在 `facts.md` 各文件「类型与名字」一行（`closure.rs` 13 个、`rust_items.rs` 6 个、`entry_groups.rs` 2 个、`output.rs` 2 个、`build_toolchain.rs` 2 个）。生产段新加的 `_ =>` 九处：`closure.rs:961`、`:1003`、`:1017`、`:1044`、`:1078`、`:1124`、`:1216` 与 `rust_items.rs:1673`、`:1702`；主 agent 读了 `closure.rs:1118-1126` 那一处，是对词法记号文本的 `match`（`"=" | ";" => true`、`":" => …`、`_ => false`），被匹配的是字符串、不是封闭枚举；其余八处腿逐处判被匹配的是不是封闭集合。`build_toolchain.rs:36` 的 `_ => None` 是 HEAD 里已有的。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：事实表按关键词查十六份的加行零命中。第 1 件改了键的内容、库里按旧键存的判定不再命中，没有读旧键的分支。

### 检查单：不新建判定库

本轮改动没有新的判定存放处：事实表按关键词（rocksdb、sqlite、prune-server）查加行零命中；`why` 子命令只往标准输出打行。

### 检查单：所有任务多机可用

事实表按关键词（hostname、`/proc/`、`/sys/`、`env::var`）查加行，腿核 `facts.md` 各文件「多机」一行；第 1 件的注释说 `CARGO_PROFILE_<名>_<键>` 里的 `debug`、`incremental`「写了也不读」，腿核别的 `CARGO_PROFILE_*` 环境变量照旧读、两台设得不一样时键跟着不一样。

### 检查单：测试优先多线程

新测试是单元图的库单测与 `tests/` 下的小用例，没有重活；没有写死或覆盖线程数的地方（事实表查 `--test-threads`、`available_parallelism`，腿核 `facts.md` 各文件「多线程」一行）。

### 检查单：带 --item 的命令与项

本轮改动没有新加可点名的项：`.claude/gate.d/stage-inputs.tsv` 与崩溃用例登记表没有指到单元图的行；新测试随测试目标点名，新加的变异行由门禁 crates-mutation-replay 按行点名。

### 检查单：新崩溃流多罩了什么

本轮改动没有新加崩溃点重放流或崩溃枚举用例（事实表查十六份的加行，非注释行里提到崩溃的 0 处）。
