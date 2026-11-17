# unit-graph-src-backfill-code-r1：判决（2026-11-17）

正文 `research/prompts/_unit-graph-src-backfill-code-r1-body.md`，开工快照 `research/prompts/unit-graph-src-backfill-code-r1-snapshot/`。这是这一题的第一轮，补的是函数单元图 `src` 十六份在途改动没走过的三方。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推 | `research/prompts/unit-graph-src-backfill-code-r1-sonnet-output.md` | 第 2、3、6、7 件对得上；第 1 件推不出（并指 `codegen-units` 的缺省值可能随 `incremental` 变、Rust 是六项白名单而 Python 两处是黑名单）；第 4 件对不上且条款没写；第 5 件推不出；纯改名 12 块；80 行变异里第 79 行不红在名字说的那条断言上；检查单八格全成立 |
| 攻方 | `research/prompts/unit-graph-src-backfill-code-r1-opus-output.md` | 第 1、2、3、4、5 件各打中；第 6 到 9 件没打中；`use … as` 那一支 Rust 与 Python 不一致；80 行变异没罩全，五处改坏之后是少收；检查单「所有任务多机可用」不成立，其余七格成立 |

核查：

- 写判决之前 `sha256sum -c research/prompts/unit-graph-src-backfill-code-r1-snapshot/crates-sha256.txt research/prompts/unit-graph-src-backfill-code-r1-snapshot/kb-sha256.txt` 全部 OK，被判的十六份与点名的 kb 文件自开工没变，没倒推快照树。
- `python3 scripts/agents/cite-check.py research/prompts/unit-graph-src-backfill-code-r1-opus-output.md research/prompts/unit-graph-src-backfill-code-r1-sonnet-output.md --root . --background research/prompts/_unit-graph-src-backfill-code-r1-body.md` 末行「✓ 核了 34 处引文、132 处裸行号，对上 34 处，没判 132 处（其中裸行号没有引文可比 132 处；逐处列在上面）」。
- 主 agent 现读的行：`crates/singlefs-unit-graph/src/build_toolchain.rs:12-37`、`crates/singlefs-unit-graph/src/closure.rs:1118-1126`、`crates/singlefs-unit-graph/src/units.rs:281`。攻方引的 `closure.rs`、`rust_items.rs` 别的行主 agent 没有逐行读，靠下面的夹具读数坐实它的结论。
- 复跑的读数（都经任务服务，攻方交的 13 条命令里的 12 条；第 1 条单独编的那一趟编译目录随任务结束被删，改成在同一条任务里先编再喂）：
  - codegen-units 探针（任务 18228，退 0，`rustc 1.98.0 (88d9e12ae 2026-08-18)`）：`codegen-units-probe incremental objects=128`、`codegen-units-probe plain objects=1`。
  - 十棵夹具（任务 18233，退 0，末行「喂了 10 条夹具」）：第 2 条 `fshadow::entry_through_the_module_path`、第 3 条 `fshadow::entry_inside_a_nested_function` 都是「闭包 2 个单元…拒算 0 处」、清单里没有 `fshadow::helper_total`，对照第 4 条 `fshadow::entry_without_a_binding` 有 `fn	fshadow::helper_total	crates/fshadow/src/lib.rs:14-16`；第 5 条 `fswap::entry_point` 闭包 2 个单元、没有 `fswap::portable::checksum`；第 6 条 `falias::entry_point` 只有 `falias::other::short::helper_total`、没有 `falias::deep::real::helper_total`；第 7 条 `fdiverge::entry_point` 有 `fdiverge::first::helper_total` 与 `fdiverge::second::helper_total`；第 10 条 `frelaunch::main`、第 11 条 `ffallback::main`、第 12 条 `fexternattr::entry_point`、第 13 条 `fexternother::entry_point` 都是「拒算 0 处」、退 0。
  - Python 那一份对第 7 条同一棵夹具（任务 18232，退 0）：清单里只有 `fdiverge::first::helper_total`。
  - 夹具原样喂、没改（喂之前 `sha256sum -c fixtures.sha256` 全 OK）。
- 没核的：正推腿交的 7 条命令（夹具 f1 到 f3、单元图整包测试、两条变异行、cargo 手册那一句）没交，它要核的几处已被攻方的夹具罩住或并进改完之后的那一轮；80 行变异没有逐行现判，留到提交时的 crates-mutation-replay；`codegen-units` 没写时的缺省值 256 与 16 是攻方凭记忆写的文档数字，主 agent 没查文档，探针只坐实了「开不开增量，代码生成单元数不同」。

## 二、判决

1. **第 1 件，进键的 profile 设置由 8 项缩成 6 项（`build_toolchain.rs`）**：打中。`codegen-units` 进键、`incremental` 不进，而 `codegen-units` 没写时生效的切法随增量变（探针 128 对 1）；两种编法出同一把键，D13（验证路线） 已定项 17「工具链（含 cargo 解析后的生效构建环境与外部工具版本）」那一句管的生效构建环境却不同。新测试 `build_toolchain.rs:249` 把这一对钉成必须相同，方向反了。`debug` 本身攻方没找到改运行结果的路，记「没打中」。检查单「所有任务多机可用」因此不成立：两台 `CARGO_INCREMENTAL` 不同时键相同。
2. **第 2 件，裸名只是函数局部绑定时不连同名函数（`closure.rs`、`rule_switches.rs`）**：打中。宏调用实参里的名字一律按局部算，宏把它展开成 `self::$f()` 或放进嵌套 `fn` 体时运行时真调到同名函数而闭包不收（夹具第 2、3 条对照第 4 条）；违的是已定项 17 里宏「不静默少收」那一句。正推腿判这一件对得上，它没看宏实参这一形；两条腿不一致的这一格以夹具读数为准。
3. **第 3 件，use 路径点准候选（`closure.rs`）**：打中两形（夹具第 5、6 条）：点到的模块里那一份候选被 cfg 关掉、名字其实由 cfg 互斥的 `pub use` 转来；相对路径的首段是 `use … as` 改出来的模块名。攻方说这两形在 HEAD 的判法下同样漏；主 agent 没在 HEAD 那一版上喂过夹具，这句是推的。违的是已定项 17「键过窄…同样算违反」。
4. **第 4 件，程序起自己（`rust_items.rs`、`units.rs`、`closure.rs`）**：打中判形状的那一半（夹具第 10、11 条：块里另一个同名绑定交给 `Command::new`、`unwrap_or_else` 的闭包以 `if … else { panic!() }` 收尾，都没拒算）。入口那一半攻方没攻出来。正推腿指 Rust 与 Python 入口判法有别（Rust 比目标根文件里第一个顶层 `main`，Python 比入口本身），没量。
5. **第 5 件，extern 块只声明登记的系统库调用（`rust_items.rs`）**：打中（夹具第 12、13 条：`cfg_attr` 包着的 `link`、同一个 crate 里另一个带 `#[link]` 的块，都没拒算）。
6. **第 6 件首边与 `why`、第 7 件不沿点名单元收、第 8 件 `build_features.rs`、第 9 件纯改名**：两条腿都没打中，记「没打中」。正文说「`entry_group_key_with_the_features_of_the_build` 的返回类型换成新类型」是错的：它仍返回 `Result<EntryGroupKey, EntryError>`（正推腿引 `entry_groups.rs:116`），换类型的是新函数。
7. **Rust 与 Python 两份实现**：`use … as` 改名那一支不一致，坐实（任务 18233 第 7 条对任务 18232）。第 1 件上 Rust 是六项白名单、Python 两处是去掉 `debug`、`incremental` 的黑名单（正推腿读码，没量）。
8. **新加的 80 行变异**：没罩全。攻方列了没有变异行的判定分支，其中改坏之后是少收的五处（`closure.rs:1190-1191`、`:1659`、`:1236`、`:1865`、`:1861`）与起自己、extern 各一处「不拒」的；正推腿另指第 79 行红在索引越界 panic 上、不红在名字说的那条断言上。都是读码推的。
9. **检查单八格**：「所有任务多机可用」不成立（第 1 条）；其余七格两条腿都判成立。四处 `expect` 的「栈不空」两条腿各自逐路追过、成立；九处 `_ =>` 匹配的都是词法记号文本（开放集合）。
10. **事实表与正文写错的**：`fn finish`、`EVERY_RULE`、`PROFILE_SETTING_KEYS`、`builtin_profile` 都不是新加的；事实表的「E053」是 rustc 错误码 E0530 被截短；`use_path_leading_to` 在第 928 行。判决里的事实以这一节与第 6 条为准。

## 三、改法

改法都是攻方腿提的、没实现，**零轮**；单元图是测试基础设施不是产品代码，A/B 由攻方那十棵夹具顶（改前的读数在第一节，改完每棵夹具的结局要翻过来、对照那几棵不变），改完照代码轮收法再攻一轮。用户定案在 `records/2026-11-16-三方豁免取消与存量补轮.md` 第 38 到 40 条。

1. 第 1 件（用户定案第 38 条）：`incremental` 不单独出行，用生效的 `incremental`（根清单、`CARGO_PROFILE_<名>_INCREMENTAL`、`CARGO_INCREMENTAL` 依次盖）定 `codegen-units` 没写时的默认值，键里只出 `codegen-units` 一行；`build_toolchain.rs:249` 那条测试改成断言两种编法的行不同。Python 那一份（`scripts/gate/lib_function_units.py`）与 `scripts/mutation/lib_mutation_units.py:103` 的 `PROFILE_SETTINGS_NOT_AFFECTING_OUTPUT` 同一口径一起改。
2. 第 2 件（用户定案第 39 条）：名字落在宏调用括号里的照连同名函数，只对 std 的格式化与断言那几个宏（`format!`、`println!`、`write!`、`assert!` 一族、`vec!`）照今天按局部算。
3. 第 3 件（条款写了，坐实就改）：点到的模块里候选带 cfg 条件、或那个模块里有引进同名的 use 时退回全连；相对路径的首段在本作用域被某条 use 引进（含 `as` 改名）时退回全连。`use … as` 那一支 Rust 与 Python 对齐到同一个判法（取多收的那一个）。
4. 第 4、5 件（用户定案第 40 条）：起自己那一条要求每处用法与那条 `let` 同在一层花括号、闭包体只认紧跟参数的块与 `=>` 后面的块；extern 那一条属性里任何位置出现 `link`、同一个目标里任一 extern 块带 `link` 或那个包的构建脚本里出现 `rustc-link-lib` 就不放行。Python 那一份同步改。
5. 第 8 条列的没有变异行的判定分支，少收与不拒方向的逐处补一行变异（改完之后还在的）；第 78、79 行变异改到红在名字说的那条断言上。
6. Rust 与 Python 在第 1、4 件入口判法上的差别：改的时候对齐，夹具两边各喂一遍、清单逐行相同。

## 交用户的

| 事 | 状态 |
|---|---|
| profile 那一件怎么改 | 用户已定（第 38 条） |
| 宏实参那一件怎么改 | 用户已定（第 39 条） |
| 起自己与 extern 两件怎么改 | 用户已定（第 40 条） |

## 这一轮被判过的文件

- crates/singlefs-unit-graph/src/bin/unit_graph.rs：判了 `why` 子命令与 `--unit` 选项，两条腿都没打中。
- crates/singlefs-unit-graph/src/build_features.rs：判了形参改名、一段注释与两条读产品 GPU 包清单的新测试，两条腿都没打中。
- crates/singlefs-unit-graph/src/build_toolchain.rs：判了进键的 profile 设置由 8 项缩成 6 项，打中 `incremental` 定 `codegen-units` 生效值那一处，照第三节第 1 条改。
- crates/singlefs-unit-graph/src/closure.rs：判了局部绑定遮同名函数、use 路径点准候选、程序起自己的入口判法、首边、不沿点名单元收五件；前三件打中，照第三节第 2、3、4 条改；四处 `expect` 与七处 `_ =>` 判成立。
- crates/singlefs-unit-graph/src/conditional_compilation.rs：判了形参与测试改名，纯改名。
- crates/singlefs-unit-graph/src/entry_groups.rs：判了新类型 `EntryGroupKeyNotFollowingTheListedUnits` 与新函数，两条腿都没打中；正文关于返回类型的那一句更正在第二节第 6 条。
- crates/singlefs-unit-graph/src/lexer.rs：判了一处测试改名，纯改名。
- crates/singlefs-unit-graph/src/lock_file.rs：判了三处测试改名，纯改名。
- crates/singlefs-unit-graph/src/manifest.rs：判了一处测试改名，纯改名。
- crates/singlefs-unit-graph/src/output.rs：判了 `why_lines` 与 `unit_position_text`，只往标准输出打行、不改闭包与键，两条腿都没打中。
- crates/singlefs-unit-graph/src/rule_switches.rs：判了规则开关 `LocalBindingsShadowSameNamedFunctions` 与 `EVERY_RULE` 长度 28 变 29，随第 2 件一起改。
- crates/singlefs-unit-graph/src/rust_items.rs：判了程序起自己的形状判法与 extern 块名单，各打中两形，照第三节第 4 条改；两处 `_ =>` 判成立。
- crates/singlefs-unit-graph/src/shell_sources.rs：判了一处测试局部变量改名，纯改名。
- crates/singlefs-unit-graph/src/units.rs：判了新成员 `SourceRead::OwnProgramRestart`，随第 4 件一起看。
- crates/singlefs-unit-graph/src/verus_ghost_code.rs：判了一处测试改名，纯改名。
- crates/singlefs-unit-graph/src/wgsl.rs：判了一处测试改名，纯改名。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 不影响 | 2026-11-17 不受影响：条款没被推翻，打中的是实现比条款窄（生效构建环境、宏不静默少收、键过窄三句都照原文用） |
