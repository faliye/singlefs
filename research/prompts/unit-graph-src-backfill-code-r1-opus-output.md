# 函数单元图 src 在途改动代码轮第一轮：攻方腿报告

日期：2026-11-17（`TZ=Asia/Tokyo date +%F`）。攻击面照派发提示：正文「要两条腿答的」攻方那一条逐处攻，再逐格判检查单八格。

开工核了被判文件：`sha256sum -c research/prompts/unit-graph-src-backfill-code-r1-snapshot/crates-sha256.txt` 十六份全 `OK`，下面引的行号都是这十六份此刻（即快照）的行号。
没写模型目录：要量的只是拿几段源码喂今天的单元图，夹具与一个小探针写在草稿目录 `~/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/`（`fixtures/` 八棵小树、`codegen-units-probe.sh`），各文件的 sha256 在同目录 `fixtures.sha256`。命令在「要主 agent 跑的命令」一节；没复跑出的读数一律是推的。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 第 1 件 profile 去掉 debug、incremental | 打中 | `codegen-units` 没写时的生效值随 incremental 变（增量 256、非增量 16），`codegen-units` 进键、incremental 不进：两种编法同一把键、代码分区不同；`debug` 在仓里没找到改结果的路（推的） |
| 第 2 件 局部绑定遮同名函数 | 打中 | 宏调用实参里的名字一律按局部算；宏把它放进 `self::$f()` 或嵌套 `fn` 体时，运行时调的是同名函数，闭包不收 |
| 第 3 件 use 路径点准候选 | 打中两形（HEAD 的判法同样漏，不是这次新添的） | 点到的模块里有一份被 cfg 关掉的候选、名字其实经 cfg 互斥的 `pub use` 转来；相对路径首段是 `use … as` 改名的模块、别处另有同末段的模块 |
| 第 4 件 程序起自己 | 打中（判形状的字面） | 块里另一个同名绑定交给 `Command::new`；`unwrap_or_else` 的闭包以 `if … else { panic!() }` 收尾也当「一定不交值」。实害取决于既有缺口（起别的程序本来就不判），见那一节 |
| 第 5 件 extern 块名单 | 打中 | `#[cfg_attr(…, link(…))]` 绕过「没有 `#[link]`」；同 crate 另一个带 `#[link]` 的块不在闭包里也照样改链接 |
| 第 6 件 首边与 why | 没打中 | 只写索引，`in_closure` 与 `units` 的来路没动 |
| 第 7 件 不沿点名单元收 | 没打中（src 这一侧） | 少收的那一截由调用方的五条测试担，src 里只有一处进闭包的路，点名单元截在它前面 |
| 第 8 件 build_features | 没打中 | 形参改名、一段注释、两条读产品 GPU 包清单的测试，不碰闭包与键 |
| 第 9 件 纯改名 | 没打中（是纯改名） | 删掉的 24 个函数名在仓里（research、records 之外）零引用 |
| Rust 与 Python 两份一致 | 打中一处不一致 | use 改名那一支：Rust 退回全连，Python 顺原名点准 |
| 80 行变异罩不罩得住 | 不罩全 | 列了没行的判定分支，其中五处改坏的方向是少收 |
| 检查单：走不到的理由二次查实 | 成立 | 栈起始一层、唯一的出栈判了长度 |
| 检查单：类型与名字 | 成立（事实表名字清单有错） | 九处 `_ =>` 匹配的都是 `&str` |
| 检查单：不兼容旧代码与旧格式 | 成立 | 两个保留的旧入口函数仍有调用方，不是兼容垫片 |
| 检查单：不新建判定库 | 成立 | |
| 检查单：所有任务多机可用 | 不成立（推的，探针待跑） | 另六个 `CARGO_PROFILE_*` 照读成立；`CARGO_INCREMENTAL` 与 `CARGO_PROFILE_<名>_INCREMENTAL` 两台不同时键相同、生效的 codegen-units 不同 |
| 检查单：测试优先多线程 | 成立 | |
| 检查单：带 --item 的命令与项 | 成立 | |
| 检查单：新崩溃流多罩了什么 | 成立 | |

打中的几格按「判据自己也会写错」四问答过，各写在该节末尾。这一轮是代码轮，没有跑前登记的改法条款，四问第四句一律答「无跑前条款；下面自提的改法只在我的读法上推过、被攻过零轮」。

## 第 1 件：进键的 profile 设置由 8 项缩成 6 项

依据条款整条读过（`.claude/kb/decisions/13-验证路线.md@已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成`，那一节第 428 行起到第 456 行的下一个标题）。键里工具链那一段是「加工具链（含 cargo 解析后的生效构建环境与外部工具版本）」（`.claude/kb/decisions/13-验证路线.md:431`）。

### 打中：incremental 决定 codegen-units 的生效值，而 codegen-units 在键里

- 代码自己的判据：「`codegen-units` 进键：它改代码分区与栈深。」（`crates/singlefs-unit-graph/src/build_toolchain.rs:18`）。
- 内置默认把 dev 写死成 256、release 写死成 16：「"dev" => Some((None, ["0", "true", "true", "false", "unwind", "256"])),」（`crates/singlefs-unit-graph/src/build_toolchain.rs:33`）、「"release" => Some((None, ["3", "false", "false", "false", "unwind", "16"])),」（`crates/singlefs-unit-graph/src/build_toolchain.rs:34`）。
- 外部事实（未在本项目验证；转述 cargo 文档 Profiles 一节 codegen-units 条与 rustc 文档 codegen-units 条）：`codegen-units` 没写时由 rustc 定，增量编译默认 256、非增量默认 16；cargo 的 `incremental` 默认 dev / test 开、release / bench 关，`CARGO_INCREMENTAL` 环境变量与 `CARGO_PROFILE_<名>_INCREMENTAL` 盖它，非路径依赖一律不增量。和本项目的差异：本项目的键不调 cargo、自己照文档现算生效值（`build_toolchain.rs` 文件头第 4 到 7 行），所以 256 / 16 这两个数是它照「默认增量」抄死的。
- 反例（两种编法，键行逐字相同、生效 codegen-units 不同）：
  1. 根清单 `[profile.release]` 写 `incremental = true`（或环境 `CARGO_PROFILE_RELEASE_INCREMENTAL=true`）。键行照旧 `profile.codegen-units: 16`；rustc 实际按增量默认 256 切。这一对正是新测试钉成「必须相同」的那一对：「release_lines_of("debug = 2\nincremental = true\n"),」（`crates/singlefs-unit-graph/src/build_toolchain.rs:249`）。
  2. 一台的进程环境有 `CARGO_INCREMENTAL=0`（CI 常见），另一台没有，跑同一条 `cargo test`（test 继承 dev）。两台键行都是 `profile.codegen-units: 256`；设了的那台工作区包实际 16。`CARGO_INCREMENTAL` 这个名字在 `build_toolchain.rs` 里零命中，HEAD 也不读它；`CARGO_PROFILE_DEV_INCREMENTAL` 是这次改动起才不读的。
- 新规则怎么判：两边 `effective_profile_lines` 交出同一组行，`toolchain_lines_of_the_build` 同一组行，键相同、判定复用。运行时真跑的：分区不同的两份代码；照代码自己那一句，栈深可以不同（深递归的用例一边过一边栈溢出）。
- 分量：HEAD 把 `incremental` 放在键里，第一种（清单或 `CARGO_PROFILE_*_INCREMENTAL`）在 HEAD 键会变，这次改动之后不变，是这次改动新开的口子；第二种（`CARGO_INCREMENTAL`）HEAD 同样漏。
- 能放进测试的一格（`build_toolchain.rs` 的 `mod tests` 里，照现有辅助写）：`assert_ne!(release_lines_of("incremental = true\n"), release_lines_of("incremental = false\n"))`——今天这条会红，与第 249 行那条正相反；改法落地之后它该绿。
- 量法：草稿目录的 `codegen-units-probe.sh` 拿 64 个模块的同一份源码，开增量与不开各编一次，数代码生成单元的目标文件个数（命令在「要主 agent 跑的命令」第 9 条）。我推它打出增量一侧明显多于 16、不增量一侧不超过 16；若两边一样，这一格的外部事实前提就不成立、撤回。

四问：分不分辨臂——分辨「6 项」与 HEAD 的「8 项」（第一种只有 6 项漏）；系统看不看得到——看得到，`incremental` 就在同一张根清单表与同一套 `CARGO_PROFILE_*` 里，只是被丢掉；满足的分句——已定项 17 那一句的「cargo 解析后的生效构建环境」：生效的 codegen-units 变了、键没变；改法——无跑前条款。

自提改法（只在我的读法上推过、被攻过零轮）：甲、`incremental` 放回键里（等于撤回一半改动）；乙、`incremental` 不出行，但用生效的 `incremental`（清单、`CARGO_PROFILE_<名>_INCREMENTAL`、`CARGO_INCREMENTAL` 依次盖）去定 `codegen-units` 没写时的默认值，只出 `codegen-units` 一行。乙修两种反例，甲只修第一种（`CARGO_INCREMENTAL` 甲也不读）。两者都是推的，待主 agent 跑。

### 没打中：debug 本身

- `cfg(debug_assertions)` 跟 `debug-assertions` 走、不跟 `debug` 走（外部事实，cargo 文档 Profiles 一节，未在本项目验证），`debug-assertions` 在键里。
- `debug` 只改调试信息：panic 消息里的文件与行来自 `Location`，不靠调试信息；回溯里的文件与行靠它。仓里 `crates/` 下的 `.rs` 读 `Backtrace`、`backtrace`、`RUST_BACKTRACE`、`RUST_LIB_BACKTRACE` 的零命中（`grep -rln 'Backtrace\|backtrace' crates --include=*.rs; echo rc=$?` 打 `rc=1`，另一条同样 `rc=1`）；四份构建脚本读 cargo 给的 `DEBUG` 的零命中（只读 `PROFILE`）。所以在今天的仓里，`debug` 改结果的路我没找到；对「断言读回溯原文」的测试这条会失效，今天没有这样的测试（推的，只核了 `crates/` 下的 `.rs`）。

### 没进键的别的设置

- `strip`、`split-debuginfo`、`rpath`、`trim-paths`：根清单与 `crates/*/Cargo.toml` 里零命中（同一条 grep 打 `rc=1`）。`strip = "symbols"` 会让回溯丢函数名、`rpath` 改动态库找法，今天都没用到，不进键说得过去；哪天写进清单，键不变、也不报，没有守卫（`build_toolchain.rs` 只摊这 6 个键名，别的键静默丢）。
- 「不认的：`.cargo/config.toml` 与 `--config` 里写的 profile、`[profile.<名>.package.<包>]` 与 `build-override`（今天工作区里一处都没有）。」（`crates/singlefs-unit-graph/src/build_toolchain.rs:7`）：这几处写了同样静默不进键、不拒算，是 HEAD 就有的，这次没动。两台 `CARGO_HOME` 下的 `config.toml` 里 profile 写得不同时键相同。
- 同一口径在别的脚本：「PROFILE_SETTINGS_NOT_AFFECTING_OUTPUT = frozenset(("debug", "incremental"))」（`scripts/mutation/lib_mutation_units.py:103`），变异行键同样丢 `incremental`；它不在这一轮被判的文件里，改法乙落地时要一起看。

## 第 2 件：裸名只是函数局部绑定时不连同名函数

判法在 `is_bare_name_only_a_local_binding_of_the_function`（`crates/singlefs-unit-graph/src/closure.rs:1144`），调用处在 R-裸名（`crates/singlefs-unit-graph/src/closure.rs:1663`）。逐行通读过它与三个辅助函数。

### 打中：宏调用实参里的名字一律按局部算

- 判法自己的说法：「作用域里写在宏调用实参里的名字照算局部的」（`crates/singlefs-unit-graph/src/closure.rs:1142`）。实现上，宏调用的开括号只把「在宏调用里」往下传，绑定状态照抄外层：「is_inside_a_macro_call: innermost.is_inside_a_macro_call」（`crates/singlefs-unit-graph/src/closure.rs:1190`）；实参里的那个名字只要外层已绑定，就走不到「if !innermost.is_name_bound {」（`crates/singlefs-unit-graph/src/closure.rs:1251`）那一处拒遮。
- 可是宏把实参放到哪里，这个函数单元的词法单元里看不出。放到路径后面或嵌套条目里的，Rust 照条目解，不看局部绑定。条款是「收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收」（`.claude/kb/decisions/13-验证路线.md:433`）。
- 反例（夹具 `fixtures/shadow-through-macro/crates/fshadow/src/lib.rs`，整份就是能放进测试的源码）：

```rust
macro_rules! call_through_the_module_path {
    ($function:ident) => {
        self::$function()
    };
}

macro_rules! call_inside_a_nested_function {
    ($body:block) => {{
        fn nested() -> u32 $body
        nested()
    }};
}

fn helper_total() -> u32 {
    7
}

pub fn entry_through_the_module_path() -> u32 {
    let helper_total = 1u32;
    call_through_the_module_path!(helper_total) + helper_total
}

pub fn entry_inside_a_nested_function() -> u32 {
    let helper_total = 1u32;
    helper_total + call_inside_a_nested_function!({ helper_total() })
}

pub fn entry_without_a_binding() -> u32 {
    helper_total()
}
```

- 新规则怎么判（推的，待第 2、3 条命令）：两个 `entry_…` 单元里，`let helper_total` 在第一个 `;` 之后把花括号那一层标成已绑定；`call_…!(` 的开括号是宏调用，往里抄已绑定；`helper_total` 不挨 `::`、后面不接 `!`，算局部；两个函数体里都没有 `fn`、`mod` 这类词（`fn nested` 在宏定义单元里，不在这两个单元的词法单元里）。于是 `is_only_a_local_binding` 为真，候选 `fn fshadow::helper_total` 被 `continue` 跳过。宏定义单元里只有 `$function`、`$body`、`self`、`nested`，也连不到它。两个入口的闭包都没有 `helper_total`，改它的函数体键不变。对照 `entry_without_a_binding` 的闭包有它。
- 运行时真调到谁：第一个展开成 `self::helper_total()`，带路径的只解条目，调 `fn helper_total`（返回 7，整个函数返回 8）；第二个展开出 `fn nested() -> u32 { helper_total() }`，嵌套条目看不见外层局部，调 `fn helper_total`（返回 8）。两段都编得过（推的，没编；局部 `helper_total: u32` 不可调用，若它解到局部就编不过，这也反过来说明运行时解到的是函数）。
- 一样能写进测试的期望：`entry_through_the_module_path` 与 `entry_inside_a_nested_function` 改 `helper_total` 的函数体之后键要变，今天不变（推的）。

四问：分不分辨臂——分辨这次改动与 HEAD（HEAD 没有这条遮法，照连）；系统看不看得到——看得到，宏定义单元就在工作区，展开式里 `$function` 挨着 `::`、`$body` 落在 `fn` 后面，判法没去看；满足的分句——已定项 17「宏…认不出的…不静默少收」；改法——无跑前条款。

自提改法（零轮）：甲、名字落在宏调用括号里的一律不算遮住（返回 false），只对 std 那几个格式化与断言宏（`format!`、`println!`、`write!`、`assert!` 一族、`vec!`）照今天算局部；乙、只对展开式里 `$` 实参挨 `::` 或落在 `fn` / `impl` 这类词之后的工作区宏不算遮住。甲简单、照顾得到外部 crate 的宏；乙要读宏定义单元。两者对真仓那一形（文件头说的 `let main = SamplingPoint::Main;` 写进宏实参与闭包）的影响要量：std 宏那一支照算局部，推它不动；别的宏要量，推的。

### 没打中的形状（第 2 件）

逐个按判法走过，都是「返回 false、照连」或语义上本来就解到局部：闭包捕获（`let c = || helper_total();` 在 `let` 之前或初始值里，外层未绑定，拒遮；绑定之后的捕获解到局部，对）；`let` 之前的使用与初始值里的使用（`is_let_waiting_for_its_semicolon` 等到 `;`）；块收尾之后的使用（出栈回到外层的未绑定）；嵌套函数、`impl`、`const`、`static`、函数体里的 `use`（`WORDS_STARTING_A_NESTED_ITEM`，见到就整个不遮）；模式里的常量与元组结构体（只遮 `UnitKind::Function` 候选，常量、结构体照连；`let 常量名 = …` 时名字不是新绑定，函数候选也不会同名同命名空间）；`if let`、`match`、`for`、闭包参数、解构绑定的同名（这几处的名字按作用域外，拒遮）；`Self::名`、`模块::名`、`名::…`（拒遮）；`名!(`（拒遮）；方法调用 `.名(`（R-方法另管，不算裸名）；区间 `..名`（不当字段，照判）；参数前带属性或 `ref`（不认成参数，拒遮）；形参与泛型同名（泛型在类型命名空间；`const` 泛型参数不是 `let` 也不是参数，没有绑定就不遮）；函数指针类型里的具名参数 `fn(名: u8)`（不在深度 0，不认成参数，拒遮）；`let … else` 的 `else` 块（块开在 `;` 之前，继承未绑定，拒遮）；格式串捕获 `{名}`（作用域外拒遮，作用域内解到局部，对）；数组长度、`const {}` 里用局部（编不过，E0435）。取样是读代码逐形推，没喂单元图。

## 第 3 件：use 路径点准候选

判法在 `candidates_seen_from_the_module_of_the_unit` 里的 `placed_by_the_use_path` 与 `use_path_leading_to`（`crates/singlefs-unit-graph/src/closure.rs:928`）。前提写在注释里：「恰好点到才算：点到的模块里自己定义的名字压过它 glob 转出来的；模块里没有这个名字的候选，名字可能是它经 `pub use` 转出来的」（`crates/singlefs-unit-graph/src/closure.rs:1782`）。下面两形都打这条前提；两形在 HEAD 的「末段出现在 use 里」判法下同样漏（我按 `changes.patch` 删掉的 `placed(&segments)` 那一支推的），所以是这条新规则没补上、不是它新开的。

### 打中一：点到的模块里有一份被 cfg 关掉的同名候选

- 候选按名字建索引时不看 cfg（`workspace.rs` 第 1276 到 1285 行只看种类与名字），被 cfg 关掉的单元要到收它时才「Activity::Inactive => return,」（`crates/singlefs-unit-graph/src/closure.rs:1395`）。于是「模块里有这个名字的候选」在 cfg 互斥的两种写法下成立，名字却是另一支 `pub use` 转来的。
- 反例（夹具 `fixtures/use-cfg-swapped-re-export/`，清单 `[features] fast = []`，`src/lib.rs` 如下）：

```rust
pub mod backend {
    #[cfg(feature = "fast")]
    pub fn checksum(data: &[u8]) -> u32 {
        data.len() as u32
    }

    #[cfg(not(feature = "fast"))]
    pub use crate::portable::checksum;
}

pub mod portable {
    pub fn checksum(data: &[u8]) -> u32 {
        data.iter().map(|byte| u32::from(*byte)).sum()
    }
}

use crate::backend::checksum;

pub fn entry_point(data: &[u8]) -> u32 {
    checksum(data)
}
```

- 新规则怎么判（推的，待第 5 条命令）：不开 `fast` 收 `fswap::entry_point`。裸名 `checksum` 的候选是 `backend::checksum` 与 `portable::checksum`；根模块的 use 那一支路径 `[crate, backend]`，`exact = [backend]`，点到 `backend::checksum`，不退回全连；`backend::checksum` 收的时候是 Inactive、不进闭包。`backend` 里那条 `pub use` 是 use 单元，不按裸名找、里面的名字也不往下收（`units.rs` 的 `is_found_by_bare_name` 对 `UnitKind::Use` 交 false）。闭包里没有 `portable::checksum`，改它的函数体键不变。
- 运行时：不开 `fast` 时 `crate::backend::checksum` 就是 `crate::portable::checksum`，调它。
- 能写进测试的期望：不开 `fast`，改 `portable::checksum` 的函数体，`fswap::entry_point` 的键要变；今天不变（推的）。

### 打中二：相对路径的首段是 `use … as` 改名的模块

- 别的起头的相对路径按后缀比：「None => !path.is_empty() && module_words.ends_with(path),」（`crates/singlefs-unit-graph/src/closure.rs:1807`）。首段若是同一作用域里 `use 甲 as 乙;` 改出来的名字，真去处是甲，后缀比的却是乙。
- 反例（夹具 `fixtures/use-relative-path-through-module-alias/crates/falias/src/lib.rs`）：

```rust
pub mod deep {
    pub mod real {
        pub fn helper_total() -> u32 {
            7
        }
    }
}

pub mod other {
    pub mod short {
        pub fn helper_total() -> u32 {
            8
        }
    }
}

use crate::deep::real as short;
use short::helper_total;

pub fn entry_point() -> u32 {
    helper_total()
}
```

- 新规则怎么判（推的，待第 6 条命令）：`use short::helper_total;` 那一支路径 `[short]`，不是 `crate`、`self`、`super` 起头，按后缀比，`other::short::helper_total` 的模块 `[other, short]` 以 `short` 收尾、点到，不退回全连；`deep::real::helper_total` 的 `[deep, real]` 不以它收尾，不连。运行时 `short` 解到根模块里那条改名（2018 起 use 路径可以从作用域里的名字起头），调 `deep::real::helper_total`。
- 能写进测试的期望：改 `deep::real::helper_total` 的函数体，键要变；今天不变（推的）。

四问（两形一并）：分不分辨臂——不分辨这次改动与 HEAD（两边都漏），分辨「点准」与「一律全连」；系统看不看得到——看得到：第一形候选单元自己带着 cfg 条件，第二形改名的 use 单元就在 `uses_by_scope` 同一组里；满足的分句——已定项 17「键过窄（引用的常量、类型、宏漏收）…同样算违反」（`.claude/kb/decisions/13-验证路线.md:432`）；改法——无跑前条款。

自提改法（零轮）：一形，点到的模块里候选带 cfg 条件、或那个模块的 `uses_by_scope` 里有引进同名的 use 时退回全连；二形，相对路径的首段在本作用域被某条 use 引进（含 `as` 改名）时退回全连，或先把首段换成那条 use 的路径再比。

### 没打中的形状（第 3 件）

`crate::` / `self::` / `super::` 起头而中间一段是被 `pub use` 转来的模块（整条比、比不上就退回全连）；同一条 use 里一支 glob 点不到、一支点得到（`collect::<Option<…>>` 一支点不到就整条退回全连）；花括号两层嵌套与兄弟支里带花括号（`use_path_leading_to` 往回数花括号，逐例走过）；开头带 `::`（根名取 `::` 之后那个词，是别的 crate 的跳过）；函数体里的 use（另有 use 就退回全连）；`use 名 as 名`（见下一小节，Rust 退回全连）；`self::甲::名` 而甲是 use 引进的（`exact` 比不上，退回全连）。读代码逐形推，没喂单元图。

### Rust 与 Python 在这一件上不一致：use 改名那一支

- Rust 的叶子只排掉后面接 `:` 的：「&& text_at(*position + 1) != ":")」（`crates/singlefs-unit-graph/src/closure.rs:1856`），`as` 后面那个别名照当叶子，从它往回收路径，前一个词是 `as`、不是 `段::`，路径为空，整条退回全连；注释自己认了：「`原名 as name` 那一支 `as` 前面不是 `段::`，路径只到外层花括号，多半点不到，退回全连」（`crates/singlefs-unit-graph/src/closure.rs:1847`）。
- Python 排掉后面接 `as` 的，别名那一支从原名起收路径：「and rust_file.t(index + 1) not in (":", "as"))]」（`scripts/gate/lib_function_units.py:4583`）、「hits = [placed_by_the_use_path(use_path_leading_to(rust_file, use_keyword, index - 2 if rust_file.t(index - 1) == "as" else index)) for index in leaves]」（`scripts/gate/lib_function_units.py:4584`）。
- 反例（夹具 `fixtures/use-alias-leaf-divergence/crates/fdiverge/src/lib.rs`）：`first` 里有 `old_total` 与 `helper_total`，`second` 里有 `helper_total`，根模块 `use crate::first::old_total as helper_total;`，`entry_point` 调 `helper_total()`。推的结局：Rust 的闭包有 `first::helper_total` 与 `second::helper_total`（退回全连），Python 只有 `first::helper_total`（点到 `[first]`）；两边另外都经改名原名连 `first::old_total`。两份闭包不同，键就不同。两边都只是多收（真调的是 `old_total`），不漏，但门禁 code-tooling 的两份实现一致那一格在这种源码上会红（待第 7、8 条命令）。

## 第 4 件：程序起自己

逐行读了 `closure.rs` 文件头那一段（第 47 到 50 行）、`finish` 里的拒算（`OwnProgramRestart` 且入口不是程序的 `main` 才拒）、`entry_is_the_main_of_a_program`（目标是 Binary、不按测试编、入口恰好是 `[main]`：「.is_some_and(|main| entry_units == [main])」（`crates/singlefs-unit-graph/src/closure.rs:2883`）），与 `rust_items.rs` 新加的四个函数。入口一侧我没攻出「起的确实是自己、子进程却跑到闭包外」的写法：同一个程序文件从同一个 `main` 进，闭包按静态可达收，跑哪条路都在里面；从 bin 的 `main` 收的闭包里也够不到别的程序的 `main`（`CARGO_BIN_EXE_*` 只给集成测试与 bench）。打中的是判「路径一个字不改地交给 `Command::new`」的那一半：判成「起自己」而起的其实是别的程序。

### 打中一：块里另一个同名绑定交给 `Command::new`

- 「再 `let` 同名」只往后看：「let is_bound_again = (expression_end..unit_end).any(|index| {」（`crates/singlefs-unit-graph/src/rust_items.rs:1785`），用法也只从绑定之后数到单元收尾，不看块收尾、不看前面的同名绑定。
- 反例（夹具 `fixtures/relaunch-shadowed-binding/crates/frelaunch/src/main.rs`；同包另有 `src/bin/worker.rs`）：

```rust
fn worker_path() -> std::path::PathBuf {
    std::env::current_exe().unwrap().with_file_name("worker")
}

fn main() {
    let exe = worker_path();
    {
        let exe = std::env::current_exe().unwrap();
        eprintln!("launcher at {}", exe.display());
    }
    std::process::Command::new(&exe).status().unwrap();
}
```

- 新规则怎么判（推的，待第 10 条命令）：`main` 里那个 `current_exe()` 后接 `.unwrap()`，是 `let exe = …;` 的右边；之后没有 `let exe`；之后的用法是 `exe.display()` 与 `Command::new(&exe)`，前者「只是显示」、后者「交给 Command::new」，记 `OwnProgramRestart`。入口是 bin `frelaunch` 的 `main`、不按测试编，`finish` 不拒算、不连边。`worker_path` 里的 `current_exe()` 不交给 `Command::new`、单元里也没有起子进程的词，不拒算。键算得出，闭包里没有 `worker` 的 `main` 与 `worker_role`。
- 运行时：块里那个 `exe` 出块就没了，`Command::new(&exe)` 用的是外面的 `worker_path()`，起的是同目录的 `worker` 程序。HEAD 在同一份源码上：`main` 单元有 `current_exe()` 又有 `Command`，折不出测试名，就地拒算 `current-exe`（推的，按 `changes.patch` 里 HEAD 那一支读）。
- 分量要说清：起别的程序这件事本身，今天在一个不含 `current_exe()` 的单元里写 `Command::new(&path)` 照样不拒算（`RefusalForm` 里没有「起了算不出的程序」这一形，`SPAWNING_WORDS` 只在有 `current_exe()` 时才用）。所以这一格的实害是：HEAD 顺带给的那一条拒算没了，而新规则的判语「起的就是在跑的这个程序」在这份源码上是错的。漏收 `worker` 的那个缺口是既有的，不归这次改动。

### 打中二：`unwrap_or_else` 的闭包以 `if … else { panic!() }` 收尾也当「一定不交值」

- `expression_ending_at_diverges` 碰到 `}` 只看块里最后一个表达式：「"}" => {」（`crates/singlefs-unit-graph/src/rust_items.rs:1649`）往里递归，`if … { 甲 } else { panic!(…) }` 的最后一个 `}` 里是 `panic!(…)`，判成一定不交值；`if` 那一支交的值没看。`match` 的 `Err` 分支同一个判法，同样中。
- 反例（夹具 `fixtures/restart-with-argument-fallback/crates/ffallback/src/main.rs`）：

```rust
fn main() {
    let this_program = std::env::current_exe().unwrap_or_else(|_| {
        if let Some(first_argument) = std::env::args_os().next() {
            std::path::PathBuf::from(first_argument)
        } else {
            panic!("拿不到程序路径")
        }
    });
    if std::env::args().count() > 1 {
        return;
    }
    std::process::Command::new(&this_program)
        .arg("role")
        .status()
        .unwrap();
}
```

- 新规则怎么判（推的，待第 11 条命令）：`.unwrap_or_else(` 后面是 `|`，闭包体最后一个词是 `}`，里面最后是 `else` 块的 `}`，再里面是 `panic!(…)` 的 `)`，判一定不交值；`let this_program = …;`、唯一的用法是 `Command::new(&this_program)`，记 `OwnProgramRestart`，从 `main` 收不拒算。
- 运行时：`current_exe()` 失败时（没有 `/proc` 的沙箱这类）退回 `argv[0]`，那可能是 `PATH` 里同名的另一个程序。只在出错那一支，概率低；判语「一定不交值」仍是错的。

四问（两形一并）：分不分辨臂——分辨这次改动与 HEAD（HEAD 两份都拒算）；系统看不看得到——看得到，块的花括号与 `if` 都在这个单元的词法单元里；满足的分句——判据是新规则自己的「起的就是在跑的这个程序」（`crates/singlefs-unit-graph/src/units.rs:279`），不是已定项 17 的哪一句，已定项 17 那边的实害见上面「分量」；改法——无跑前条款。

自提改法（零轮）：一形，「再 `let` 同名」与用法都在整个单元里数（不止绑定之后），或要求每一处用法与那条 `let` 同在一层花括号、中间不出块；二形，`}` 那一支只认闭包体本身的块（开括号紧跟在闭包参数的 `|` 后面）与 `=>` 后面的块，块前面是 `else`、`)`、`if` 条件的不认。Python 一份同写（`scripts/gate/lib_function_units.py` 第 3337 到 3432 行，两形逐句同义），要一起改。

### 没打中的形状（第 4 件）

`for exe in …`、`|exe|`、`Some(exe)` 这类别的绑定（那一处名字本身算一处用法，不是交给 `Command::new`，整个不认、拒算）；`exe = 别的;` 重新赋值、`exe.push(…)`、`&mut exe`（都算别的用法）；路径交给宏、交给辅助函数、当 `.arg(…)`（同上）；`current_exe()?.parent()`、`.with_file_name(…)`（表达式之后不是 `;` 也不是 `Command::new` 的闭括号）；入口是 bin 里的辅助函数、模块里的 `main`、`tests::…`、按测试编的 `main`、一组入口里另有别的（`entry_is_the_main_of_a_program` 交 false，拒算）；`harness = false` 的测试目标（不是 Binary，拒算，只多拒）；lib 里起自己从 bin 的 `main` 调到（子进程仍是这个 bin，对）。读代码逐形推。按名字认 `current_exe`、`Command`、`panic`、`process::exit`，用户自己定义同名函数、宏或类型的写法我没往下攻（都要写一个故意骗判法的同名条目，HEAD 同样按名字认）。

## 第 5 件：extern 块只声明登记的系统库调用

- 名单之外的怎么判：符号（`#[link_name]` 的串，没有就是函数名）不在 `SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS` 里、ABI 不是 `"C"`、块前有 `#[link`、块里有 `static`、别的属性、`unsafe fn` / `safe fn`、空块，一律照旧拒算 `extern-function-interface`（第 1825 到 1880 行逐分支走过）。
- 名单里两个凭什么不拒：代码给的理由是标准库的文件读写走同一层 C 库，键本来不跟 C 库变。这条理由我认：`std::fs` 的 `sync_data` 同样调 C 库的 `fdatasync`，换 C 库时两者同进退，这两个符号没有多出一类依赖。Python 一侧还写了一条不判的：「不判：工程里自己定义同名导出符号盖掉 C 库」（`scripts/gate/lib_function_units.py:357`）——这一形对 `std::fs` 一样成立，同一个既有缺口，不算这次的。

### 打中：「块前没有 `#[link`」只看紧挨着的三个词，也只看这一个块

- 判法：「let has_link_attribute = (item_start..keyword_index).any(|index| {」（`crates/singlefs-unit-graph/src/rust_items.rs:1830`）里要 `#`、`[`、`link` 三个词紧挨着：「&& self.text_of(index + 2) == "link"」（`crates/singlefs-unit-graph/src/rust_items.rs:1833`）。Python 同写（`scripts/gate/lib_function_units.py:3444`）。
- 反例一（夹具 `fixtures/extern-link-through-cfg-attr/crates/fexternattr/src/lib.rs`，属性写成 `cfg_attr` 是外面常见的写法）：

```rust
#[cfg_attr(target_os = "linux", link(name = "fakesync"))]
extern "C" {
    fn fdatasync(fd: i32) -> i32;
}

pub fn entry_point() -> i32 {
    unsafe { fdatasync(3) }
}
```

- 反例二（夹具 `fixtures/extern-link-on-another-block/crates/fexternother/src/lib.rs`）：

```rust
#[link(name = "fakesync")]
extern "C" {}

extern "C" {
    fn fdatasync(fd: i32) -> i32;
}

pub fn entry_point() -> i32 {
    unsafe { fdatasync(3) }
}
```

- 新规则怎么判（推的，待第 12、13 条命令）：反例一，`#` 之后是 `[`、`cfg_attr`，不是 `link`，不算带 `#[link]`；块里只有 `fn fdatasync`，在名单里，不拒算。反例二，带 `#[link]` 的那个空块自己会拒算，可是没有谁点名它（它不声明名字），不进闭包；第二个块不拒算。两份都算出键，键里没有 `libfakesync` 的任何东西。
- 运行时：在 Linux 上链接时多一个 `-l fakesync`，它排在 C 库前面；它要是导出了 `fdatasync`，这一处（连同 `std::fs` 的那一处）调的是它。`#[link]` 在 Rust 里对整个产物生效、不只对写它的那个块，所以「这个块前面有没有」不是判别式。HEAD 在这两份上都拒算（每个 extern 块一律拒）。
- 能写进测试的期望：两份都该拒算 `extern-function-interface`；今天都算出键（推的）。

四问：分不分辨臂——分辨这次改动与 HEAD；系统看不看得到——一形看得到（`link` 就在属性的词法单元里），二形要看同 crate 别的 extern 块（同一份文件或同一个目标的单元表里都有）；构建脚本的 `cargo::rustc-link-lib` 那一路这里看不到；满足的分句——已定项 17「收不全…的那一份拒算键、照跑，不静默少收」；改法——无跑前条款。

自提改法（零轮）：一形，属性里任何位置出现 `link` 这个词（不只紧挨 `#[`）就算带；二形，同一个目标里任一个 extern 块带 `link`（含 `cfg_attr` 里的）、或那个包的构建脚本里出现 `rustc-link-lib`，就不放行名单里的块。

## 第 6 件：首边与 why 子命令

没打中。逐处核了：`record_the_first_edge` 只往 `first_edges` 里插（已有的不改），`push` 里先记边再照旧判 `seen_pairs`；`unit_being_visited`、`rule_being_followed` 只是被记的值；进闭包的唯一一处 `self.in_closure.insert(unit_index)` 在 `visit_unit_for_target` 里（`grep -n 'in_closure.insert'` 只命中第 1407 行），收尾 `first_edges` 只按 `in_closure` 过滤、不反过来改它；键由 `reuse_key` 从 `units`、数据单元与 cfg 原子拼，`first_edges` 与 `units_reached_and_not_followed` 两个新字段的名字在 `reuse_key` 那一路里零命中（`grep -n 'first_edges\|unit_being_visited\|rule_being_followed'` 的命中全在记录处、`finish` 与 `output.rs` 的 `why_lines`）。`why` 子命令与 `closure`、`key` 走同一条收闭包的路，收完只打 `why_lines`、退 0，不往别处写。`--unit` 给了 `why` 之外的子命令是用法错。附带一处：`rule` 是自由写的 `&'static str`（「R-裸名」「R-方法」……），拼错了没有类型挡，只影响 `why` 的输出。

## 第 7 件：不沿点名单元收的入口组键

- 谁用：验证包的构建脚本（GPU 判器那一组，两张登记表）与 `crash_identity` 的测试；产品 GPU 测试包的构建脚本仍调不带登记的那一个。
- 少收的那一截凭什么不进键：已定项 17 的「单元级」排除（`.claude/kb/decisions/13-验证路线.md:434` 那一条，整条读过），要求配五条测试，五条都在调用方（另开一题判）。
- src 这一侧没打中：截断点「if self.units_not_followed.contains(&unit.qualified_name) {」（`crates/singlefs-unit-graph/src/closure.rs:1389`）在 `in_closure.insert` 之前，进闭包只有这一处，所以点名单元连同它的读、拒算、cfg 原子与往下的边一起不收，和条款「不进这一组的闭包、不从它往下收」一致；登记空的时候这一行恒假，键与不带登记的逐字相同。两处与条款的测试相关的细节：截断在 `activity` 之前，被 cfg 关掉的点名单元也记进「够到了」（调用方那条「够到的恰好等于登记的」会因此误红，不会少收）；点名单元自己带的拒算（`current-exe`、`extern-function-interface` 这类）一并被跳过，键照算——这是「不沿它收」的本义，靠调用方「改一个登记了的单元，这一组的键不变」那条测试的前提（登记的都不改判定字）担着。

## 第 8 件：build_features.rs

没打中。形参改名不改行为；新加的 `#[cfg(test)] #[path = "../../singlefs-background-gpu/build.rs"] mod …` 只在单测里编，两条新测试只比清单 `[features]` 的名字与构建脚本里的特性表、元数据键的原文，不碰闭包与键。读别的包的清单与构建脚本用的 `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), …))` 与同文件里早有的 `CORE_MANIFEST_TEXT` 同一写法。

## 第 9 件：纯改名

没打中，是纯改名：`conditional_compilation.rs` 的宏元变量 `$os` 换 `$operating_system`（定义与唯一一处用处一起换，展开式不变）；`shell_sources.rs` 一个测试局部变量；其余是测试函数去 `a_` 前缀。`changes.patch` 里删掉的 24 个函数名逐个 `git grep -n -w` 过 research、records 之外的仓：改名的 22 个零命中，`builtin_profile`、`finish` 是签名改了的同名函数、不是改名。

## 新加的 80 行变异罩不罩得住每条新规则的每个判定分支

按 `mutation-rows-new.tsv` 第三、四段（原文与替换文）逐行对到判定分支（行号是那张表的行次）。罩到的：局部绑定第 9 到 34 行、use 路径第 35 到 42 行、程序起自己第 43 到 64 行、extern 第 65 到 72 行、首边与 why 第 73 到 76 行、不沿点名单元收第 77 行、profile 第 78 到 80 行。没有一行改到的判定分支如下；「方向」写那一处改坏之后闭包是多收还是少收（少收的才会让键漏输入）。都是读代码推的，没跑变异。

| 规则 | 没行的判定分支（`文件:行`） | 改坏的方向 |
|---|---|---|
| 局部绑定 | 宏调用标记往里传：`crates/singlefs-unit-graph/src/closure.rs:1190-1191` 开括号把「在宏调用里」往里传的两个条件（第 26 行只改了 `let` 那一处的判断） | 少收：嵌套在宏调用里的花括号里的 `let` 被当成绑定 |
| 局部绑定 | 改名原名不看遮法：`crates/singlefs-unit-graph/src/closure.rs:1659` | 少收：原名在这个单元里另有局部绑定时也被遮 |
| 局部绑定 | 区间 `..名` 不当字段：`crates/singlefs-unit-graph/src/closure.rs:1236` 那一句的第二个条件 | 少收：绑定之前的 `0..名()` 被跳过 |
| 局部绑定 | `let` 要在花括号层（`innermost.is_brace`）、名字后接 `=` / `;`、`let` 词本身、参数表里 `,` 复位与 `名::` 排除、签名里 `->` 的 `>` 不出尖括号、`fn` 要在深度 0（`crates/singlefs-unit-graph/src/closure.rs:999`）、只在长度大于 1 时出栈（`crates/singlefs-unit-graph/src/closure.rs:1197`） | 多是多收或编不过的写法；出栈守卫去掉会让下一个 `expect` panic |
| use 路径 | 每一支都点到才算点到：`crates/singlefs-unit-graph/src/closure.rs:1865` 的 `collect::<Option<…>>`（换成只收点到的那几支） | 少收：一支 glob 经 `pub use` 转来点不到、另一支点得到时不退回全连 |
| use 路径 | 一支都没找到时退回全连：`crates/singlefs-unit-graph/src/closure.rs:1861` 那一句前面的 `(!paths_bringing_in_the_name.is_empty())` | 少收：引进名单里有、词法单元里找不到叶子（`r#名`）时一份都不连 |
| use 路径 | 叶子认 `*`、叶子后面不接 `:`（`crates/singlefs-unit-graph/src/closure.rs:1856`）、`use_path_leading_to` 的 `use_keyword + 4` 与花括号嵌套计数 | 多是退回全连（多收） |
| 程序起自己 | 「至少起一次」：`crates/singlefs-unit-graph/src/rust_items.rs:1809` | 不拒：只显示路径、`Command::new` 起的是字面别的程序时也记成起自己 |
| 程序起自己 | `expression_ending_at_diverges` 里 `}` 的递归与 `;`、`panic` 与 `process::exit` 各自的判断（第 44、58 行换的是整个调用）；`match` 的单分支判断；`unwrap` 空括号与 `expect` 字面串；`&名` 那一形；目标是 Binary（`crates/singlefs-unit-graph/src/closure.rs:2879`） | 前两样改坏是不拒（把交值的当不交值）；后几样多是多拒 |
| extern | 块里出现 `link_name` 之外的属性就拒（`crates/singlefs-unit-graph/src/rust_items.rs:1843` 起那几句）、`link_name` 不是字面串、`pub` 跳过、ABI 之后紧跟 `{`、空块（`declared_any`）、找不到 `;` | 第一样改坏是不拒：`#[cfg_attr(…, link_name = "别的")]` 的声明被放行 |
| 首边、不沿点名单元收 | 记进 `units_reached_and_not_followed` 那一句 | 只影响调用方的测试 |

另两处要主 agent 注意：第 78、79 行把 `PROFILE_SETTING_KEYS` 加到 7 项而 `builtin_profile` 仍交 6 个默认值，`zip` 之后少一项，没写那个键的测试在 `effective[key]` 处索引越界 panic（推的）；这两行会红，但多半红在 panic 上，不是红在「debug / incremental 不改行」那条断言上。第 77 行的测试在验证包（`--release --features gpu --lib`），跑它要编验证包。

## 检查单八格

1. **走不到的理由二次查实：成立。** 栈起始就有一层：「let mut open_delimiters = vec![OpenDelimiterOfTheBindingScan {」（`crates/singlefs-unit-graph/src/closure.rs:1159`）；函数里只有一处出栈「open_delimiters.pop();」（`crates/singlefs-unit-graph/src/closure.rs:1198`），套在「if open_delimiters.len() > 1 {」（`crates/singlefs-unit-graph/src/closure.rs:1197`）里；`push` 只加。四处 `expect`（第 1169、1203、1210、1231 行）都在这之后取最后一层，栈至少一层。别的文件：`changes.patch` 加行里 `unreachable!`、`panic!(`、`.expect(`、`.unwrap()`、`assert` 一族，按 awk 数出来 `closure.rs` 4 处（就是这四处）、`build_toolchain.rs` 7 行、`build_features.rs` 6 行、`conditional_compilation.rs` 4 行、`shell_sources.rs` 1 行；后四份的行号都大于各自 `mod tests` 或 `#[cfg(test)]` 的起始行（195、261、577、3254），在测试段。新代码里另有几处下标减法（`position - 3`、`call_start - 1`、`close - 1`、`arm_end - 1`、`use_index - 2`），前面都有守卫，逐处核过。
2. **类型与名字：成立；事实表的名字清单有错（见下一节）。** 九处 `_ =>`（`closure.rs` 第 961、1003、1017、1044、1078、1124、1216 行，`rust_items.rs` 第 1673、1702 行）匹配的都是 `token_text` / `text_of` 交的 `&str`，开放集合，`_` 是必需的；对封闭枚举 `TokenKind` 的那一处「TokenKind::Punctuation => match text {」（`crates/singlefs-unit-graph/src/closure.rs:1183`）所在的外层 `match token.kind` 四个成员逐个写、没有 `_`。名字：新类型、新函数都说出内容；`FirstEdgeIntoTheClosure.rule` 是自由写的串，见第 6 件。
3. **不兼容旧代码与旧格式：成立。** 加行里没有认旧键的分支。保留的 `compute_closure_of_entries_with_the_features_of_the_build` 与 `entry_group_key_with_the_features_of_the_build` 是空登记的包装，仍有调用方（前者 `build_features.rs` 的测试，后者 `singlefs-background-gpu-tests/build.rs` 与 `singlefs-implementation-experiments/src/bin/e182_unreadable_abandoned_root_forks/main.rs`），不是为旧调用方留的垫片。
4. **不新建判定库：成立。** 加行里没有落盘的路；`why` 只往标准输出打。
5. **所有任务多机可用：不成立（推的，探针待跑）。** 窄的那句成立：另六个键照旧读 `CARGO_PROFILE_<名>_<键>`（`crates/singlefs-unit-graph/src/build_toolchain.rs:137` 那一段），两台设得不同键就不同。宽的不成立：照第 1 件，两台 `CARGO_INCREMENTAL` 或 `CARGO_PROFILE_<名>_INCREMENTAL` 不同时，键相同而生效的 codegen-units 不同；两台 `CARGO_HOME` 下 `config.toml` 的 profile 不同，键同样不变（既有）。加行里没有主机名、`/proc/`、`/sys/`、`env::var`。
6. **测试优先多线程：成立。** 加行里没有 `--test-threads`、`available_parallelism`；新测试是切词夹具，不编不跑重活。
7. **带 --item 的命令与项：成立。** 没有新加可点名的项；新测试随测试目标点名，新变异行由 crates-mutation-replay 按行点名。
8. **新崩溃流多罩了什么：成立。** 加行里没有崩溃点重放流与崩溃枚举用例。

## 事实表 facts.md 写错的地方

1. `closure.rs`「类型与名字」13 个里的 `fn finish` 不是新加的：HEAD 里就有（`git show HEAD:crates/singlefs-unit-graph/src/closure.rs | grep -n 'fn finish'` 打 `2250:    fn finish(mut self, entry_target: TargetIndex, entry_units: Vec<UnitIndex>) -> ClosureOutcome {`），这次只改了签名。
2. `rule_switches.rs`「新加的条目名 1 个：const EVERY_RULE」不是新加的（HEAD 第 91 行 `pub const EVERY_RULE: [BrokenRule; 28] = [`），只改了长度；真正新加的是枚举成员 `LocalBindingsShadowSameNamedFunctions`，它没进这一栏。
3. `build_toolchain.rs`「新加的条目名 2 个：const PROFILE_SETTING_KEYS、fn builtin_profile」两个都不是新加的（HEAD 第 17、29 行就有，`[&str; 8]`），只改了长度与内容；正文「`build_toolchain.rs` 2 个」随之不对，这一份新加的名字是 0 个（新测试与两个测试辅助在 `mod tests` 里）。
4. `closure.rs` 第 1 样「L1778–L1880 use 路径点准候选（use_path_leading_to）」：`use_path_leading_to` 是第 928 行的自由函数；L1778 起那一段是 `candidates_seen_from_the_module_of_the_unit` 里的 `placed_by_the_use_path` 闭包与它的调用处。
5. `closure.rs` 第 2 样「closure.rs:29：E053」是正则把 rustc 的错误码 `E0530` 截掉一位，不是项目的实验编号。
6. 各文件第 4 样「两处都没引到」只搜了单元图这一个包：`entry_group_key_not_following_the_listed_units` 由 `crates/singlefs-verification/src/crash_identity.rs` 的测试调（第 1049、1570、2301 行），`entry_group_key_with_the_features_of_the_build` 由两份构建脚本与一个 bin 调，`compute_closure_of_entries_not_following_the_listed_units` 经前者被同一批测试间接测到；`unit_graph.rs` 的 `lines_of`、`parse_arguments` 由 `tests/unit_graph_command_line_prints_one_item_per_line.rs` 起二进制测到。「没引到」不等于「没测」。
7. `closure.rs` 逐块一句（脚本取的「函数上下文加首个加行」）对这几件判断够用，没发现与源码对不上的；但它看不出第 2、3 件的判定分支，本报告这两件都是重读源文件得的。

## 要主 agent 跑的命令

全部在仓根起（`--cwd` 不给即仓根，或写 `/home/fy5090/code/singlefs`）。第 1 条编出二进制，之后各条把 `<build=>` 换成第 1 条 `task wait` 末行 `build=` 给的目录。第 2 到 8、10 到 13 条读的夹具在草稿目录，原样喂、不改。每条看末行 `TASK_SERVER_RESULT` 的 `exit=`，再看 `task-server/task log <号> --all` 里点名的行。

1. `task-server/task submit -- cargo build -p singlefs-unit-graph --bin unit-graph`。看：`exit=0`，末行有 `build=`。
2. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/shadow-through-macro fshadow::entry_through_the_module_path --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：`exit=0`；组成清单里有没有 `fshadow::helper_total` 那一行。我推没有（打中）；有就撤回第 2 件「打中」的这一形。
3. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/shadow-through-macro fshadow::entry_inside_a_nested_function --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：同上，我推没有 `fshadow::helper_total`。
4. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/shadow-through-macro fshadow::entry_without_a_binding --toolchain-line 'host: x86_64-unknown-linux-gnu'`。对照：我推有 `fshadow::helper_total`；没有就说明夹具本身连不上，第 2、3 条不算数。
5. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/use-cfg-swapped-re-export fswap::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：`exit=0`；我推组成清单里没有 `fswap::portable::checksum`（打中）。
6. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/use-relative-path-through-module-alias falias::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：我推有 `falias::other::short::helper_total`、没有 `falias::deep::real::helper_total`（打中）。
7. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/use-alias-leaf-divergence fdiverge::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：我推 `fdiverge::first::helper_total` 与 `fdiverge::second::helper_total` 都在。
8. `task-server/task submit -- python3 scripts/gate/lib_function_units.py closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/use-alias-leaf-divergence fdiverge::entry_point`。看：`exit=0`；我推只有 `first::helper_total`、没有 `second::helper_total`。与第 7 条不同即两份实现不一致（打中）；相同就撤回那一小节。
9. `task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/codegen-units-probe.sh /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack`。看：`exit=0`；日志里 `codegen-units-probe incremental objects=` 与 `codegen-units-probe plain objects=` 两行。我推前者大于 16、后者不大于 16；两数相同就撤回第 1 件「打中」的前提。
10. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/relaunch-shadowed-binding frelaunch::main --target bin:frelaunch --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：我推 `exit=0`、没有 `refused` 行（打中）；`exit=3` 且拒算行点名 `current-exe` 就撤回这一形。
11. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/restart-with-argument-fallback ffallback::main --target bin:ffallback --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：同上，我推 `exit=0`。
12. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/extern-link-through-cfg-attr fexternattr::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：我推 `exit=0`、没有 `extern-function-interface` 拒算行（打中）。
13. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/extern-link-on-another-block fexternother::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：同上，我推 `exit=0`。

第 1 条之后编译目录归服务，主 agent 用完 `task-server/task clean <号>`。第 8 条的 Python 入口写法照它文件头「Rust 另认 fn 路径」；它要是认不出这个入口（退 2），换成 `crates/fdiverge/src/lib.rs::entry_point` 再交。

## 没打中的形状（汇总）

- 第 2 件：见那一节末的清单（十几形，读代码逐形推，没喂单元图）。
- 第 3 件：见那一节末的清单。
- 第 4 件：入口一侧（「起的确实是自己、子进程跑到闭包外」）没攻出来；形状一侧见那一节末。
- 第 5 件：名单外的符号、别的 ABI、`static`、别的属性、`safe fn`、空块都照拒；`#[no_mangle]` 同名导出盖掉 C 库对 `std::fs` 同样成立，不算这次的。
- 第 6、7、8、9 件：见各节。

## 这条腿自己的限度与没做什么

- 所有「新规则怎么判」都是按源码推的，没喂单元图；判决要引的结局在第 2 到 13 条命令里，没出结局之前一律当「推的」。
- 第 1 件的 256 / 16 与 incremental 的关系是 cargo 与 rustc 文档里的事实，我没在本机现查文档原文（凭的是我对这两份文档的记忆，按「我印象里」算），第 9 条探针是这一格唯一的本项目观测。
- 第 2 件之外的反例都要求写法略偏（块里同名遮蔽、`cfg` 互斥的 `pub use`、模块改名再走相对路径、`cfg_attr` 包 `link`）；今天的仓里有没有这些写法我没扫。
- 没跑变异、没跑测试；80 行变异的覆盖是按原文与替换文对分支推的。
- 没判正推、辩方那几格；没替主 agent 采纳；没往任务服务交任务。
- 没建仓副本与编译目录。草稿目录里留着 `fixtures/`（八棵夹具树，没有根 `Cargo.toml`，不是仓副本）、`codegen-units-probe.sh`、`fixtures.sha256`、`removed-fn-names.txt`，都是第 2 到 13 条命令的输入或核对用的清单。
