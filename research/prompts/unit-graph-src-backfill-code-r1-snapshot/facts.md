# 事务员出的事实表（线索，主 agent 只现查了正文点名的那几处）

### unit-graph/src/bin/unit_graph.rs（+27 / -4，12 块）

**1 这一次改了什么**
为命令行加 why 子命令：L8 用法加一行 `unit-graph why <根> <入口 fn 路径>… --unit <子串>`；L55 引入 why_lines；Command 加 Why（L80）、Arguments 加 unit_pattern（L96）、--unit 选项解析（L139、L162）、位置参数个数报错文字加「与 why」（L197）、lines_of 里 Why 与 Closure/Key 同走（L291），L340 起 9 行在 why 且给了 --unit 时调 why_lines。

逐块：
- unit_graph.rs:5 +3/-0 〔〕 加：//!   unit-graph why <根> <入口 fn 路径> [<入口 fn 路径>…] --unit <子串> [与 closure 相同的选项]
- unit_graph.rs:52 +1/-1 〔use singlefs_unit_graph::conditional_compilation::〕 加：closure_lines, dump_lines, key_line, parse_time_refusals, refusal_lines, why_lin ｜ 删：closure_lines, dump_lines, key_line, parse_time_refusals, re
- unit_graph.rs:63 +2/-1 〔const USAGE: &str = concat!(〕 加："      unit-graph key <根> <入口 fn 路径> [<入口 fn 路径>…] [与 closure 相同的选项]\n", ｜ 删："      unit-graph key <根> <入口 fn 路径> [<入口 fn 路径>…] [与 closur
- unit_graph.rs:77 +1/-0 〔enum Command {〕 加：Why,
- unit_graph.rs:93 +2/-0 〔struct Arguments {〕 加：/// `why` 要查的单元：文件路径或限定名里的一段。
- unit_graph.rs:105 +1/-0 〔fn parse_arguments(raw: &[String]) -> Result<Argum〕 加："why" => Command::Why,
- unit_graph.rs:122 +1/-0 〔fn parse_arguments(raw: &[String]) -> Result<Argum〕 加：unit_pattern: None,
- unit_graph.rs:136 +1/-0 〔fn parse_arguments(raw: &[String]) -> Result<Argum〕 加：| "--unit"
- unit_graph.rs:159 +1/-0 〔fn parse_arguments(raw: &[String]) -> Result<Argum〕 加："--unit" => arguments.unit_pattern = Some(value.clone()),
- unit_graph.rs:194 +4/-1 〔fn parse_arguments(raw: &[String]) -> Result<Argum〕 加："dump 要 1 个位置参数（根），closure、key 与 why 要根与至少一个入口；给了 {} 个", ｜ 删："dump 要 1 个位置参数（根），closure 与 key 要根与至少一个入口；给了 {} 个",
- unit_graph.rs:288 +1/-1 〔fn lines_of(〕 加：Command::Closure | Command::Key | Command::Why => { ｜ 删：Command::Closure | Command::Key => {
- unit_graph.rs:337 +9/-0 〔fn lines_of(〕 加：if let (Command::Why, Some(pattern)) = (arguments.command, &arguments.unit_patte

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 3 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 1 个，两处都没引到（grep 名字）的 2 个：lines_of、parse_arguments。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 1 行（键新出现）、消失 0 行、行变 4 行（多为必红测试名随改名变）。 新增行的变异名：单元图 why：--unit 给了别的子命令也收

**5 线索，没核**
- 没有。

### unit-graph/src/build_features.rs（+75 / -15，15 块）

**1 这一次改了什么**
三类：L118–L155 形参名 features_with_their_cfg→features_with_their_conditional_compilation_flags；L270 在 features_enabled_on_the_dependencies 加 11 行注释，说明 singlefs-background-gpu 的构建脚本只在单测里编进来读特性表；测试里 8 个去 a_，L768–L816 新增 49 行：include_str 读 background-gpu 清单与 build.rs，两个测试核「构建脚本报告的键」与「清单里每个特性都在表里」。

逐块：
- build_features.rs:115 +2/-2 〔fn environment_name_of(name: &str) -> String {〕 加：/// 正在编的包自己开了哪些特性（清单 `[features]` 里、`features_with_their_conditional_compilation ｜ 删：/// 正在编的包自己开了哪些特性（清单 `[features]` 里、`features_with_their_cfg
- build_features.rs:124 +2/-2 〔fn environment_name_of(name: &str) -> String {〕 加：features_with_their_conditional_compilation_flags: &[(&str, bool)], ｜ 删：features_with_their_cfg: &[(&str, bool)],
- build_features.rs:152 +1/-1 〔pub fn features_of_the_crate_being_built(〕 加：features_with_their_conditional_compilation_flags ｜ 删：features_with_their_cfg
- build_features.rs:267 +11/-0 〔pub fn features_enabled_on_the_dependencies(〕 加：/// `singlefs-background-gpu` 的构建脚本本身，只在单测里编进来读它的特性表（`tests::the_background_gpu_
- L377–L695：8 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L377 fn a_fixture_workspace_is_gone_once_the_→fn fixture_workspace_is_gone_once_the_sc；L396 fn a_fixture_workspace_half_written_when→fn fixture_workspace_half_written_when_a
- build_features.rs:739 +1/-1 〔mod tests {〕 加：/// `singlefs-core` 清单 `[features]` 里的每个特性在它构建脚本的特性表（`FEATURES_OF_THIS_PACKAGE_W ｜ 删：/// `singlefs-core` 清单 `[features]` 里的每个特性在它构建脚本的特性表（`FEATUR
- build_features.rs:750 +1/-1 〔mod tests {〕 加：super::singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITION ｜ 删：super::singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_
- build_features.rs:764 +49/-0 〔mod tests {〕 加：/// `singlefs-background-gpu` 的清单与构建脚本，取法同 [`CORE_MANIFEST_TEXT`]。

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 6 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 3 个（非测试函数）：同文件测试模块引到的 2 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 1 个：environment_name_of。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 2 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/build_toolchain.rs（+80 / -29，3 块）

**1 这一次改了什么**
L16–L36 进键的 profile 设置由 8 项缩成 6 项：从 PROFILE_SETTING_KEYS 去掉 "debug" 与 "incremental"（L21、L25 两行删），builtin_profile 返回的数组长度 8→6、dev/release/test/bench 四条内置值各去两个元素（L33–L36），注释改写并引 D13（验证路线）已定项 17「工具链（含 cargo 解析后的生效构建环境与外部工具版本）」，写明 debug 与 incremental 不进键、根清单与 CARGO_PROFILE_<名>_<键> 里写了也不读，codegen-units 进键；L210 起新增 69 行两个测试（debug/incremental 不改 profile 行、debug-assertions/opt-level/codegen-units 各改 profile 行）与两个辅助 release_manifest_with、release_lines_of；L313 测试名 the_dev_… 改 the_development_…。

逐块：
- build_toolchain.rs:13 +10/-19 〔use std::collections::BTreeMap;〕 加：/// 进键的 profile 设置：会改编出来的代码跑出的结果的那几样（按 cargo 文档 Profiles 一节的键名）。 ｜ 删：/// 进键的 profile 设置：会改编出来的代码怎么跑的那几样（按 cargo 文档 Profiles 一节的键名
- build_toolchain.rs:207 +69/-9 〔mod tests {〕 加：// 根清单里的 `debug = true` 不出行；`codegen-units` 取内置默认 ｜ 删：assert!(
- build_toolchain.rs:310 +1/-1 〔mod tests {〕 加：fn the_development_profile_without_a_table_keeps_the_builtin_overflow_checks() { ｜ 删：fn the_dev_profile_without_a_table_keeps_the_builtin_overflo

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- build_toolchain.rs:18：D13（验证路线） 已定项 17 ｜ /// 里写了也不读。`codegen-units` 进键：它改代码分区与栈深。依据是 D13（验证路线） 已定项 17
去重：D13（验证路线） 已定项 17

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 8 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 2 个：const PROFILE_SETTING_KEYS、fn builtin_profile（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 1 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 1 个：builtin_profile。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 3 行（键新出现）、消失 0 行、行变 0 行（多为必红测试名随改名变）。 新增行的变异名：构建档键把 debug 放回进键的 profile 设置、构建档键把 incremental 放回进键的 profile 设置、构建档键把 codegen-units 移出进键的 profile 设置

**5 线索，没核**
- build_toolchain.rs:21-36 进键设置少两项，是改变「工具链」单元键的内容（键的组成在 .claude/rules/verification.md「一个门禁：两种跑法」），上线后凡带 debug 或 incremental 设置差异的键会合并、已记判定的键会整批变；没核这是不是预期的一次性作废，也没核 CARGO_PROFILE_*_DEBUG 一类环境变量确实不影响被测代码行为（注释自己的论断）

### unit-graph/src/closure.rs（+588 / -13，37 块）

**1 这一次改了什么**
最大一块。新增 588 行：L23、L44 头注释各加 4 行（use 路径点准候选；程序自己重起 SourceRead::OwnProgramRestart 在 finish 判）；L217 新增 FirstEdgeIntoTheClosure 与 ClosureOutcome 的首边索引（L239）、ClosureRun.first_edges（L889）、record_the_first_edge（L1301）；L905–L1260 新增 356 行：函数签名与参数/let 绑定位置扫描，判「裸名只是函数局部绑定」从而不遮同名函数（is_bare_name_only_a_local_binding_of_the_function 等，规则开关 LocalBindingsShadowSameNamedFunctions）；L1778–L1880 use 路径点准候选（use_path_leading_to）；各规则分支写 rule_being_followed；finish 加 entry_is_the_main_of_a_program 与 OwnProgramRestart 拒绝；L3131 起新增 compute_closure_of_entries_not_following_the_listed_units 与 units_not_followed 字段。

逐块（机器取：函数上下文、首个加行、首个删行；未逐块通读）：
- closure.rs:23 +4/-0 〔〕 加：//! use 引进的名字要 use 路径恰好点到候选所在的模块才算点准：路径点到的模块里没有这个名字的候选，名字是那个模块经 `pub use` 转出来的，落
- closure.rs:44 +4/-0 〔〕 加：//! 程序起自己（[`SourceRead::OwnProgramRestart`]，`finish` 判）：`current_exe()` 的路径原样交给 
- closure.rs:58 +4/-2 〔use crate::conditional_compilation::{〕 加：use crate::lexer::{ ｜ 删：use crate::lexer::{lex_source, token_text, DocumentationComm
- closure.rs:214 +9/-0 〔pub struct DataUnit {〕 加：/// 闭包里一个单元头一回被排进待收队列的那条边：正在收哪个单元（入口与收尾几步并进来的没有）、照哪条规则、经哪个名字（规则不按名字连的为空）。
- closure.rs:236 +4/-0 〔pub struct ClosureOutcome {〕 加：/// 闭包里每个单元头一回被排进来的那条边（[`FirstEdgeIntoTheClosure`]）：只当索引，不进键。
- closure.rs:886 +8/-0 〔struct ClosureRun<'workspace> {〕 加：/// 每个被排进队列的单元头一回是经哪条边排进来的（`record_the_first_edge`）；不改收什么，只当索引。
- closure.rs:905 +356/-0 〔const ALWAYS_NAMED_TYPES: [&str; 23] = [〕 加：/// 函数体里这几个词起头的可能是嵌套条目。嵌套条目看不见外层函数的局部绑定，里面的同名引用要往外解；哪一处在嵌套条目里不细分，
- closure.rs:1289 +20/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：first_edges: BTreeMap::new(),
- closure.rs:1374 +2/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.unit_being_visited = Some(unit);
- closure.rs:1386 +5/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：if self.units_not_followed.contains(&unit.qualified_name) {
- closure.rs:1414 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "R-开放";
- closure.rs:1424 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "R-隐式（类型被点名）";
- closure.rs:1436 +2/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "R-隐式（条目级宏调用生成的类型被点名）";
- closure.rs:1523 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "R-结构";
- closure.rs:1574 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "R-裸名";
- closure.rs:1590 +3/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：let local_bindings_shadow_same_named_functions = self
- closure.rs:1608 +2/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：// 只在这个名字真有同名函数要连时算一次（文件头「R-裸名局部绑定遮住同名函数」）；use 改名的原名不是这个单元写的名字，不算遮住
- closure.rs:1653 +18/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：if local_bindings_shadow_same_named_functions
- closure.rs:1715 +4/-4 〔impl<'workspace> ClosureRun<'workspace> {〕 加：/// `RustWorkspace.candidates_seen_from_module` 同一个判法）：与单元同一模块的；所在模块的 use 引进了 `n ｜ 删：/// `RustWorkspace.candidates_seen_from_module` 同一个判法）：与单元同一
- closure.rs:1778 +33/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：// use 的一支路径（不含名字本身）恰好点到的候选（下标）；一份都没点到交 None。
- closure.rs:1844 +22/-5 〔impl<'workspace> ClosureRun<'workspace> {〕 加：// 这条 use 里引进 name 的每一支（`*`、名字本身）的路径，花括号展开之后；`原名 as name` 那一支 `as` 前面不是 `段::`，路径 ｜ 删：let segments: Vec<&str> = (first_token..end_token)
- closure.rs:1962 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "R-方法";
- closure.rs:1993 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.record_the_first_edge(*member, method_name);
- closure.rs:2004 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "R-路径";
- closure.rs:2061 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.record_the_first_edge(*member, &format!("{qualifier}::{member_name}"));
- closure.rs:2088 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "全局分配器";
- closure.rs:2321 +2/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：// 拒不拒算要看入口，在 `finish` 判
- closure.rs:2504 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：| SourceRead::OwnProgramRestart { .. }
- closure.rs:2531 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "WGSL 外名的着落";
- closure.rs:2739 +1/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：self.rule_being_followed = "子进程测试";
- closure.rs:2750 +22/-1 〔impl<'workspace> ClosureRun<'workspace> {〕 加：fn finish( ｜ 删：fn finish(mut self, entry_target: TargetIndex, entry_units: 
- closure.rs:2846 +6/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：let in_closure = &self.in_closure;
- closure.rs:2860 +19/-0 〔impl<'workspace> ClosureRun<'workspace> {〕 加：first_edges,
- closure.rs:3131 +24/-0 〔pub fn compute_closure_of_entries_with_the_feature〕 加：) -> Result<ClosureOutcome, EntryError> {
- closure.rs:3207 +1/-0 〔pub fn compute_closure_of_entries_with_the_feature〕 加：run.units_not_followed.clone_from(units_not_followed);
- closure.rs:3218 +1/-0 〔pub fn compute_closure_of_entries_with_the_feature〕 加：run.rule_being_followed = "链进来的别的目标的 main";
- closure.rs:3233 +1/-1 〔pub fn compute_closure_of_entries_with_the_feature〕 加：let mut outcome = run.finish(entry_target, entries, entry_conditions.is_test); ｜ 删：let mut outcome = run.finish(entry_target, entries);

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- closure.rs:29：E053 ｜ //! 不是新绑定（rustc 对静态量与元组结构体报 E0530），这几种照连。
去重：E053

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段新加 4 处；测试段 0 处 assert/expect/panic。生产段逐处：
  - closure.rs:1169：.expect("最外面那一层不出栈（下面的出栈判了长度），栈不空");
  - closure.rs:1203：.expect("上面只在长度大于 1 时出栈，栈不空")
  - closure.rs:1210：.expect("最外面那一层不出栈（上面的出栈判了长度），栈不空");
  - closure.rs:1231：.expect("最外面那一层不出栈（上面的出栈判了长度），栈不空");
- `_ =>`：生产段 7 处，测试段 0 处：
  - closure.rs:961：_ => {}
  - closure.rs:1003：_ => {}
  - closure.rs:1017：_ => {}
  - closure.rs:1044：_ => {}
  - closure.rs:1078：_ => {}
  - closure.rs:1124：_ => false,
  - closure.rs:1216：_ => {}
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 13 个：struct FirstEdgeIntoTheClosure、const WORDS_STARTING_A_NESTED_ITEM、fn use_path_leading_to、struct FunctionSignaturePositions、fn function_signature_positions、fn parameter_binding_position、fn is_statement_let_binding_at、struct OpenDelimiterOfTheBindingScan、fn is_bare_name_only_a_local_binding_of_the_function、fn record_the_first_edge、fn finish、fn entry_is_the_main_of_a_program、fn compute_closure_of_entries_not_following_the_listed_units（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 11 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 2 个，两处都没引到（grep 名字）的 9 个：compute_closure_of_entries_not_following_the_listed_units、compute_closure_of_entries_with_the_features_of_the_build、entry_is_the_main_of_a_program、function_signature_positions、is_bare_name_only_a_local_binding_of_the_function、is_statement_let_binding_at、parameter_binding_position、record_the_first_edge、use_path_leading_to。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 54 行（键新出现）、消失 0 行、行变 58 行（多为必红测试名随改名变）。 新增行的变异名：单元图 R-裸名按模块筛：crates/ 下的包同一目标里改回按同名全连、单元图 R-裸名按模块筛：开关 crates-directory-names-by-module 不起作用、单元图 R-裸名按模块筛：use 引进的名字认不出（引进名单反着比）、单元图 R-裸名按模块筛：glob use 不当引进、单元图 R-裸名按模块筛：use 的根名是别的 crate 也当点本目标、单元图 R-裸名按模块筛：路径的根名是别的 crate 也当点本目标、单元图 R-裸名按模块筛：super::super 只往上数一层、单元图 R-裸名按模块筛：macro_rules! 体里的名字也按定义所在模块筛…另 46

**5 线索，没核**
- closure.rs 7 处新加 `_ =>`（L961、L1003、L1017、L1044、L1078、L1124、L1216）落在新增的词法位置扫描里，按「类型与名字」格不写 `_ =>` 的要求逐处要核是不是穷举了 Token 种类，没核
- closure.rs:1169、:1203、:1210、:1231 四处 expect 的消息说「栈不空」，依赖「最外面那一层不出栈」，是否真由前面的长度判断保证，没逐路追

### unit-graph/src/conditional_compilation.rs（+16 / -9，4 块）

**1 这一次改了什么**
纯改名：形参/局部变量 os→operating_system（L87、L97、L611-L619），一个测试名 a_cfg_inside_cfg_attr…→conditional_compilation_inside_conditional_compilation_attribute…（L732）。其中 L97 宏里的 `// naming-lint:external` 注释保留。

逐块：
- conditional_compilation.rs:84 +1/-1 〔pub struct TargetPlatform {〕 加：arch $arch:literal, os $operating_system:literal, family $family:literal, env $e ｜ 删：arch $arch:literal, os $os:literal, family $family:literal, 
- conditional_compilation.rs:94 +1/-1 〔macro_rules! target_platform_table {〕 加：("target_os", $operating_system), // naming-lint:external rustc 定的 cfg 名 ｜ 删：("target_os", $os), // naming-lint:external rustc 定的 cfg 名
- conditional_compilation.rs:608 +12/-6 〔mod tests {〕 加：let operating_system_is_linux = &conditions("cfg ( target_os = \"linux\" )")[0]; ｜ 删：let os_is_linux = &conditions("cfg ( target_os = \"linux\" )
- conditional_compilation.rs:729 +2/-1 〔mod tests {〕 加：fn conditional_compilation_inside_conditional_compilation_attribute_only_applies ｜ 删：fn a_cfg_inside_cfg_attr_only_applies_when_the_condition_hol

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 4 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 8 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/entry_groups.rs（+59 / -19，6 块）

**1 这一次改了什么**
L117–L240：新增 EntryGroupKeyNotFollowingTheListedUnits 与 entry_group_key_not_following_the_listed_units（L131、L141，33 行），原 entry_group_key_with_the_features_of_the_build 内部改调 compute_closure_of_entries_not_following_the_listed_units，返回值类型由 EntryGroupKey 改成新类型（L166、L239）；L31 引入名换成新函数；L640 一个测试去 a_。

逐块：
- entry_groups.rs:28 +1/-1 〔use std::collections::BTreeSet;〕 加：compute_closure_of_entries_not_following_the_listed_units, reuse_key, ClosureReq ｜ 删：compute_closure_of_entries_with_the_features_of_the_build, r
- entry_groups.rs:114 +33/-0 〔pub fn entry_group_key_with_the_features_of_the_bu〕 加：entry_group_key_not_following_the_listed_units(
- entry_groups.rs:163 +13/-10 〔pub fn entry_group_key_with_the_features_of_the_bu〕 加：return Ok(EntryGroupKeyNotFollowingTheListedUnits { ｜ 删：return Ok(EntryGroupKey {
- entry_groups.rs:190 +2/-1 〔pub fn entry_group_key_with_the_features_of_the_bu〕 加：match compute_closure_of_entries_not_following_the_listed_units( ｜ 删：match compute_closure_of_entries_with_the_features_of_the_bu
- entry_groups.rs:236 +9/-6 〔pub fn entry_group_key_with_the_features_of_the_bu〕 加：Ok(EntryGroupKeyNotFollowingTheListedUnits { ｜ 删：Ok(EntryGroupKey {
- entry_groups.rs:637 +1/-1 〔mod tests {〕 加：fn unit_line_before_its_group_and_a_malformed_key_are_refused() { ｜ 删：fn a_unit_line_before_its_group_and_a_malformed_key_are_refu

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 2 个：struct EntryGroupKeyNotFollowingTheListedUnits、fn entry_group_key_not_following_the_listed_units（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 2 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 2 个：entry_group_key_not_following_the_listed_units、entry_group_key_with_the_features_of_the_build。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 2 行（多为必红测试名随改名变）。

**5 线索，没核**
- entry_groups.rs:166、:239 返回类型换名，原 EntryGroupKey 是否仍被别处用，本表没追

### unit-graph/src/lexer.rs（+1 / -1，1 块）

**1 这一次改了什么**
一个测试去 a_（L479）。

逐块：
- lexer.rs:476 +1/-1 〔mod tests {〕 加：fn range_after_a_number_is_not_part_of_the_number() { ｜ 删：fn a_range_after_a_number_is_not_part_of_the_number() {

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 5 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/lock_file.rs（+4 / -4，3 块）

**1 这一次改了什么**
三个测试去 a_（L332、L356、L374）。

逐块：
- L329–L371：3 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L329 fn a_dependency_item_with_a_source_picks→fn dependency_item_with_a_source_picks_t；L353 fn a_dependency_item_with_a_version_pick→fn dependency_item_with_a_version_picks_

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 11 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/manifest.rs（+1 / -1，1 块）

**1 这一次改了什么**
一个测试改名：a_library_declaring_the_proc_macro_… → library_declaring_the_procedural_macro_…（L573）。

逐块：
- manifest.rs:570 +1/-1 〔mod tests {〕 加：fn library_declaring_the_procedural_macro_crate_type_is_a_procedure_macro_crate( ｜ 删：fn a_library_declaring_the_proc_macro_crate_type_is_a_proced

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 3 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/output.rs（+84 / -0，1 块）

**1 这一次改了什么**
L290 起新增 84 行：unit_position_text（L290）与 pub fn why_lines（L309），按 --unit 子串把闭包里的单元分成「命中单元自己」「由命中单元引进来」「引到命中单元」几组输出首边（规则、名字）。

逐块：
- output.rs:287 +84/-0 〔pub fn closure_lines(〕 加：fn unit_position_text(workspace: &Workspace, unit_index: UnitIndex) -> String {

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 2 个：fn unit_position_text、fn why_lines（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 3 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 1 个，两处都没引到（grep 名字）的 2 个：unit_position_text、why_lines。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 1 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/rule_switches.rs（+8 / -1，4 块）

**1 这一次改了什么**
加规则开关 LocalBindingsShadowSameNamedFunctions（L64）：EVERY_RULE 长度 28→29（L94）、文件头用法加两行（L11）、match 分支一支（L133，3 行）。

逐块：
- rule_switches.rs:8 +2/-0 〔〕 加：//!   local-bindings-shadow-same-named-functions  R-裸名：函数里的局部绑定不遮同名函数（名字只是 `let`
- rule_switches.rs:61 +1/-0 〔pub enum BrokenRule {〕 加：LocalBindingsShadowSameNamedFunctions,
- rule_switches.rs:91 +2/-1 〔pub enum BrokenRule {〕 加：pub const EVERY_RULE: [BrokenRule; 29] = [ ｜ 删：pub const EVERY_RULE: [BrokenRule; 28] = [
- rule_switches.rs:130 +3/-0 〔impl BrokenRule {〕 加：BrokenRule::LocalBindingsShadowSameNamedFunctions => {

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 1 个：const EVERY_RULE（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 1 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/rust_items.rs（+265 / -9，9 块）

**1 这一次改了什么**
三块：L53、L65 注释；L1646–L1880 新增 235 行：expression_ending_at_diverges（表达式一定不交值）、own_path_expression_range、is_command_new_open、own_program_path_is_handed_to_command_unchanged（识别 `current_exe()` 路径原样交 Command::new，产出 SourceRead::OwnProgramRestart）、extern_block_declares_only_listed_system_library_calls 与常量 SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS=["pwrite","fdatasync"]（L68）；L1991、L2501 两处调用；4 个测试改名。

逐块：
- rust_items.rs:50 +2/-2 〔const BUILD_SCRIPT_OUTPUT_DIRECTORY_VARIABLE: &str〕 加：/// 单元里出现这几个词算起了子进程：此时 `current_exe()` 折不出要点名的那一条测试、路径又不是一个字不改地交给 `Command::new` ｜ 删：/// 单元里出现这几个词算起了子进程：此时 `current_exe()` 折不出要点名的那一条测试就拒算 `curr
- rust_items.rs:62 +5/-0 〔const SPAWNING_WORDS: [&str; 7] = [〕 加：/// 不带 `#[link]` 的 `extern "C"` 块里只声明这几个符号时不拒算（`extern_block_declares_only_liste
- rust_items.rs:1643 +235/-0 〔impl<'scan> RustScanner<'scan> {〕 加：/// `last_token` 是一个表达式的最后一个词法单元时，这个表达式是否一定不交值：写成 `panic!(…)`、`…process::exit(…)
- rust_items.rs:1988 +9/-0 〔impl<'scan> RustScanner<'scan> {〕 加：// 路径一个字不改地交给 `Command::new`：起的是这个程序自己，拒不拒算看入口（闭包的 `finish`）
- rust_items.rs:2498 +10/-3 〔impl<'scan> RustScanner<'scan> {〕 加：if !self.extern_block_declares_only_listed_system_library_calls( ｜ 删：new_unit
- L3108–L3298：4 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L3108 fn a_use_with_an_alias_brings_only_the_a→fn use_with_an_alias_brings_only_the_ali；L3167 fn a_path_qualified_item_macro_invocatio→fn path_qualified_item_macro_invocation_

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段 2 处，测试段 0 处：
  - rust_items.rs:1673：_ => false,
  - rust_items.rs:1702：_ => false,
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 6 个：const SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS、fn expression_ending_at_diverges、fn own_path_expression_range、fn is_command_new_open、fn own_program_path_is_handed_to_command_unchanged、fn extern_block_declares_only_listed_system_library_calls（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 5 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 1 个，两处都没引到（grep 名字）的 4 个：expression_ending_at_diverges、is_command_new_open、own_path_expression_range、own_program_path_is_handed_to_command_unchanged。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 22 行（键新出现）、消失 0 行、行变 42 行（多为必红测试名随改名变）。 新增行的变异名：单元图 程序起自己：路径原样交给 Command::new 的也一律就地拒算、单元图 程序起自己：闭包一定不交值的 unwrap_or_else 不认、单元图 程序起自己：match 的 Err 分支 return 不认、单元图 程序起自己：问号取路径不认、单元图 程序起自己：lib 里起自己从 main 调到也拒算、单元图 程序起自己：Command::new 括号里路径之后又接方法也算、单元图 程序起自己：Command::new(名字.方法()) 也算、单元图 程序起自己：let 的右边路径之后又接方法也算…另 14

**5 线索，没核**
- 没有。

### unit-graph/src/shell_sources.rs（+5 / -2，1 块）

**1 这一次改了什么**
一个测试内局部变量 two_levels_up_then_a→path_climbing_two_levels_then_naming_one_file（L3268），改动 5 行。

逐块：
- shell_sources.rs:3265 +5/-2 〔mod tests {〕 加：let path_climbing_two_levels_then_naming_one_file = concat!("..", "/../a"); ｜ 删：let two_levels_up_then_a = concat!("..", "/../a");

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 1 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 12 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/units.rs（+4 / -0，1 块）

**1 这一次改了什么**
SourceRead 枚举加成员 OwnProgramRestart { line: u32, fragment: String }（L281）与 4 行文档注释。

逐块：
- units.rs:275 +4/-0 〔pub enum SourceRead {〕 加：/// `current_exe()` 的路径原样交给 `Command::new`（`rust_items` 的 `own_program_path_is_h

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 1 个（非测试函数）：同文件测试模块引到的 0 个，tests/ 目录文件引到的 1 个，两处都没引到（grep 名字）的 0 个。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 2 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/verus_ghost_code.rs（+1 / -1，1 块）

**1 这一次改了什么**
一个测试去 a_（L783）。

逐块：
- verus_ghost_code.rs:780 +1/-1 〔mod tests {〕 加：fn source_without_verus_blocks_comes_back_byte_for_byte() { ｜ 删：fn a_source_without_verus_blocks_comes_back_byte_for_byte() 

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 19 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### unit-graph/src/wgsl.rs（+1 / -1，1 块）

**1 这一次改了什么**
一个测试去 a_（L587）。

逐块：
- wgsl.rs:584 +1/-1 〔mod tests {〕 加：fn constant_named_in_a_switch_case_selector_is_a_reference_not_a_local_declarati ｜ 删：fn a_constant_named_in_a_switch_case_selector_is_a_reference

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 5 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

