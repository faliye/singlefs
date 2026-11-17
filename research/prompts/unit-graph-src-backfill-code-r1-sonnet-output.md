# 函数单元图 src 在途改动代码轮第一轮：正推腿报告（three-way-forward）

日期：2026-11-17。腿：Sonnet。草稿目录：`~/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward/`（夹具与改名核对脚本在里面，下文点名）。

## 0 开工核对

- 十六份被判文件开工时与交回前各跑一次 `sha256sum -c research/prompts/unit-graph-src-backfill-code-r1-snapshot/crates-sha256.txt`，十六行全 `OK`，没有被别的会话改过。kb 的 `.claude/kb/decisions/13-验证路线.md` sha256 与快照 `kb-sha256.txt` 一致（`4e322bf8…1835`）。
- 没写模型（正推腿这一轮读源码、读 patch、读 Python 那一份，只用只读命令与一段改名核对脚本），所以没有 `tools/rerun.sh`；要主 agent 跑的在末尾「要主 agent 跑的命令」。
- 整条读过的条款：`.claude/kb/decisions/13-验证路线.md` 已定项 17，第 428 行标题到第 457 行下一个标题之前。下文引它写行号：431 是键的组成与「工具链」那一句，432 是「一律不进键」与「键过窄」，433 是「收不全」拒算，434 是三种排除（含单元级「不沿它收」），437 是「改键让旧库作废不算代价」，440 是射程。这几行原样抄在附录 A。
- 判定词：「对得上」「对不上」「推不出（缺什么）」。读数一律是我读源码推的，没跑过任何测试或变异，一律标「推的」；要复跑的在末尾。

## 1 第 1 到 7 件：从条款正推到代码

条款的骨架（431 行）：键由「函数单元各自去注释后的指纹」「闭包里 cfg 真引到的原子带真值」「工具链（含 cargo 解析后的生效构建环境与外部工具版本）」「从 `Cargo.lock` 解出的闭包真引到的依赖版本」「判定自己的内容输入」组成；432 行「键过窄（引用的常量、类型、宏漏收）」「库放在会丢的位置」「改名丢库」算违反；433 行「收不全……的那一份拒算键、照跑，不静默少收」。多收只是多判，少收是违反，所以第 2、3、4、5 件（让闭包变小或让拒算变放行）每件的判据是「被排掉的东西运行时一定不会跑到」。

### 件 1 进键的 profile 设置由 8 项缩成 6 项（`crates/singlefs-unit-graph/src/build_toolchain.rs:20`）

- 键多含什么：无。少含什么：工具链段里 `profile.debug: …` 与 `profile.incremental: …` 两行没了，`profile.opt-level / debug-assertions / overflow-checks / lto / panic / codegen-units` 六行与次序不变；内置默认值数组 dev、release 各剥掉两个元素，我逐位对过：dev 六位 `["0","true","true","false","unwind","256"]` 对 opt-level、debug-assertions、overflow-checks、lto、panic、codegen-units，release `["3","false","false","false","unwind","16"]` 同序，没错位。
- 条款：「加工具链（含 cargo 解析后的生效构建环境与外部工具版本）」（`.claude/kb/decisions/13-验证路线.md:431`）没有一句说档设置里哪几项算「生效构建环境」；「改键让旧库作废不算代价」（`.claude/kb/decisions/13-验证路线.md:437`）管缩键后旧库作废这一头，对得上。
- 注释立的前提「`debug`（调试信息）与 `incremental`（增量编译）不改编出来的代码跑出的结果」（`crates/singlefs-unit-graph/src/build_toolchain.rs:17`），kb 里没有它的出处；唯一的出处是 `records/2026-11-14-门禁静态审核.md:390` 那一行「独立核对」，那一行自己同时写「`codegen-units` 会不会改结局没核」。注释又写「`codegen-units` 进键：它改代码分区与栈深」，这一半同样是没核的。
- 我自己能推的：`cfg(debug_assertions)` 由 `debug-assertions` 管、不由 `debug` 管，键里留着前者；仓里没有 `build.rs` 读 `DEBUG`（`grep -rn '"DEBUG"' crates --include=build.rs` 零命中，我跑过）；`crates/singlefs-verification/build.rs:123` 只读 `PROFILE` 取档名。推不出的几处：(a) 第三方依赖的构建脚本读 cargo 给的 `DEBUG`（`crates/singlefs-unit-graph/src/build_toolchain.rs:4` 自己写了 cargo 给构建脚本 `DEBUG`），它们不在闭包里，只靠依赖版本管；(b) cargo 的 `codegen-units` 缺省值随 `incremental` 变（256 对 16，我凭记忆写，本机没有离线 cargo 文档，没现查），若属实则「`codegen-units` 进键、`incremental` 不进」自相矛盾：开 `incremental` 会改有效 cgu 而键不动；(c) `strip`、`split-debuginfo`、`rpath`、`[profile.<名>.package.<包>]`、`build-override`、`.cargo/config.toml` 里的 profile 都不进键，`crates/singlefs-unit-graph/src/build_toolchain.rs:7` 自己写了后三样「不认」，前三样既不进键也没论证（这不是本轮新增，缩键没改变它）。
- 判定：**推不出**（缺：kb 里一句说明档设置哪几项算生效构建环境；缺 `debug`、`incremental` 不改运行结果的现查依据；缺 cargo 文档对 `codegen-units` 缺省与 `incremental` 关系的一句）。
- 什么现象推翻它：一个判定的结局随 `debug` 变（断言读到 backtrace 文本、第三方构建脚本按 `DEBUG` 出 cfg）；或 cargo 文档写 `incremental` 改 `codegen-units` 缺省（则「有效 cgu 不在键里」成立，键过窄）。
- 另：Python 三处（`scripts/gate/admission.py:867`、`scripts/mutation/lib_mutation_units.py:103`）用的是「去掉 `debug`、`incremental`，别的 profile 键照进」的黑名单，Rust 这一处是六项白名单；对 `strip`、`rpath` 这类 Python 进键、Rust 不进。三份各算各的工具链行，不是同一把键的两份实现，所以不触发门禁「两份实现对拍」，但同一个 manifest 改一行 `strip = true`，Python 两处键变、Rust 键不变。

### 件 2 局部绑定遮住同名函数（`crates/singlefs-unit-graph/src/closure.rs:910-1262`、调用处 `crates/singlefs-unit-graph/src/closure.rs:1656-1672`）

- 键少含什么：被遮名字的同名函数单元及它往下的闭包（只在 `unit.kind == Function`、候选 `kind == Function`、名字不是 use 改名的原名时）。多含：无。
- 条款：431 行「函数单元从这份判定的入口沿调用与引用收」，432 行「键过窄……算违反」。这件的判据是「被排掉的那个同名函数运行时一定调不到」，逐种绑定位置的论证在 §2。
- 判定：**对得上，条件是 §2 的论证**；§2 里有一处推不出（形参遮住「返回类型与 where 子句」里的 `名()`）。
- 什么现象推翻它：§2 点名的任何一种位置里，rustc 解析到的是外面的同名函数而这套扫描判成「只是局部绑定」。

### 件 3 use 路径点准候选（`crates/singlefs-unit-graph/src/closure.rs:926-970` 与 `crates/singlefs-unit-graph/src/closure.rs:1781-1872`）

- 键少含什么：use 引进 `名`（或 glob）时，只连「那一支路径恰好点到的模块」里的同名候选；点不准（该模块里没有这个名字的候选，名字是它经 `pub use` 转出来的、或任一支点不到）退回全连。多含：点不准的情形比旧判法多连（旧判法 `placed(&segments)` 把整条 use 里所有标识符混在一起、候选模块末段碰巧在里面就算点到；新判法要求恰好点到，点到时只会更窄，点不到时退回全连，比旧判法多连，这是它补的漏）。
- 条款同件 2。判定：**对得上，附两处推不出**（§2.3）；另 Rust 与 Python 在 `原名 as 名` 那一支上判法不同（§6，不影响单边的收全，影响两份对拍）。
- 什么现象推翻它：一个 use 引进的名字，rustc 解到的函数不在「路径恰好点到的模块」里、而点到的模块里又恰好有同名的别的候选。

### 件 4 程序起自己（`crates/singlefs-unit-graph/src/rust_items.rs:1647-1812`、`crates/singlefs-unit-graph/src/units.rs:281`、`crates/singlefs-unit-graph/src/closure.rs:2753-2884`）

- 键少含什么：`Command::new(current_exe()…)` 路径原样交出的写法，入口恰好是 bin 的 `fn main` 时由「就地拒算」改成「不拒算、不连边」；入口不是它时照旧拒算（现在由 `finish` 判，不再由扫描时就地判）。
- 此刻怎么判：见 §3。
- 判定：**对不上（两形）**：识别「路径一个字不改」的两个写法漏了一条路径改写——`unwrap_or_else` 的闭包里先 `return` 另一条路径、最后一句 `panic!` 收尾（`crates/singlefs-unit-graph/src/rust_items.rs:1700` 只看最后一个表达式不交值），子进程可能是别的程序；§3.3 给夹具。其余写法对得上。**另有「条款没写」**：`.claude/kb/checks-owed.md:522`（C889）把「`current-exe` 一类怎么处置」记成「要主 agent 定」，D13 已定项 17 里没有一句直接管「自起」。
- 什么现象推翻它：§3.3 夹具 f1 `key` 算得出键、不拒算 `current-exe`，而它的子进程可以是 `/bin/sh`——对不上成立；若算出拒算行，对不上撤回。

### 件 5 extern 块只声明登记的系统库调用（`crates/singlefs-unit-graph/src/rust_items.rs:68`、`crates/singlefs-unit-graph/src/rust_items.rs:1818`、调用处）

- 键少含什么：无（块单元仍进闭包、指纹仍进键）。变化的是拒算：`extern "C"` 无 `#[link…]`、块里只有 `fn` 声明、符号（`link_name` 或函数名）都在 `["pwrite","fdatasync"]` 里的块不再拒算 `extern-function-interface`。
- 条款：433 行「收不全……拒算键」；FFI 本身 D13 已定项 17 没写。注释的理由「C 库的系统调用封装，标准库的文件读写走的是同一层，换 C 库影响它们的程度与 `std::fs` 相同」（`rust_items.rs` 常量上方注释）是代码自己立的前提（`crates/singlefs-unit-graph/src/rust_items.rs:65-66`）；libc 版本不在工具链行里（`rustc -vV`、`cargo -V`、目标、RUSTFLAGS、profile），`std::fs` 同样不在，所以前提「同一层」自洽，但条款没点它。
- 判定：**推不出（缺：条款里一句说明系统库调用的声明不算「收不全」；`.claude/kb/checks-owed.md:522` 写明这类处置要主 agent 定）**。代码本身与它的注释一致、与 Python 一致（§6）。
- 什么现象推翻它：这两个符号在被判的程序里被一个工程自己的 `#[no_mangle] pub extern "C" fn pwrite` 盖掉（Python 文件头明说「不判」；Rust 这一侧靠裸名连边把同名函数一起连进来，但若它在不可见的目标里就连不到）。

### 件 6 首边与 `why` 子命令（`crates/singlefs-unit-graph/src/closure.rs:1301-1319`、`crates/singlefs-unit-graph/src/output.rs:290-376`、`bin/unit_graph.rs`）

- 键少含/多含：无。我逐处核了：`first_edges`、`unit_being_visited`、`rule_being_followed` 只在 `record_the_first_edge` 里读（`grep -n 'first_edges\|unit_being_visited\|rule_being_followed' crates/singlefs-unit-graph/src/*.rs`，读处只有 `crates/singlefs-unit-graph/src/closure.rs:1302-1307` 与 `crates/singlefs-unit-graph/src/output.rs:325、344`）；`push` 与 `follow_bare_names`、`follow_methods`、`follow_qualified_paths` 里新加的都是 `record_the_first_edge(...)` 一行，不改 `seen_pairs` 与 `pending`；`reuse_key` 只读 `outcome.units`、`wgsl_units`、`data_units`、`condition_atoms_with_their_values`、工具链行与锁行，不读 `first_edges`。`why` 在 `lines_of` 里返回得比算锁行早（`bin/unit_graph.rs` 新加 9 行），不算键。
- 判定：**对得上**。限制：`rule_being_followed` 的值是上一回被设的，没设的路径上标签可能是上一个单元的（只影响 `why` 打的文字，不进键）；测试只钉了一条链（`why_prints_the_chain_of_first_edges_from_a_unit_back_to_the_entry`）。
- 什么现象推翻它：同一入口在这批改动前后 `unit-graph key` 的闭包行或键不同（`closure.rs` 有一处不是 `record_*` 的控制流改动：件 2 的 `continue`，不属本件）。

### 件 7 不沿点名单元收的入口组键（`crates/singlefs-unit-graph/src/closure.rs:1389-1393`、`crates/singlefs-unit-graph/src/closure.rs:3151-3256`、`entry_groups.rs`）

- 键少含什么：登记里的单元不进 `in_closure`、不往下收，其余不变；登记为空时 `contains` 恒假，与改前逐字相同（`compute_closure_of_entries_with_the_features_of_the_build` 现在以空登记转调新函数，`entry_group_key_with_the_features_of_the_build` 以空登记转调、`.map(|keyed| keyed.key)`）。
- 条款：434 行「单元级：Rust 一侧按判器组、按单元（限定名）登记「不沿它收」」一整段，里面点名了 `compute_closure_of_entries_not_following_the_listed_units` 与 `entry_group_key_not_following_the_listed_units`，也写了「登记是空的，键与不带登记的逐字相同」。本件的机制与条款写的一致。
- 被谁用：`crates/singlefs-verification/build.rs:185`（构建时算键）与 `crates/singlefs-verification/src/crash_identity.rs`（五条测试，条款写的那五条在验证包里，不在本轮）。「少收的那一截凭什么不进键」由条款 434 行的登记 + 五条测试担，这一包的代码本身没有守卫，只把「真够到的几个另外交出」（`units_reached_and_not_followed`）。
- 判定：**对得上**。**facts/正文写错一处**：「`entry_group_key_with_the_features_of_the_build` 的返回类型换成新类型」不对，它的签名仍是 `Result<EntryGroupKey, EntryError>`（`crates/singlefs-unit-graph/src/entry_groups.rs:116`），换成新类型的是新函数（§8）。
- 什么现象推翻它：空登记下与改前键不同；或一个登记过的单元改了而键变（那是验证包那五条测试管的）。

## 2 第 2、3 件：被排掉的同名函数运行时一定调不到吗

### 2.1 件 2 的判法，先写清它此刻怎么判（`is_bare_name_only_a_local_binding_of_the_function`，`crates/singlefs-unit-graph/src/closure.rs:1144-1262`）

名字 `N` 在函数单元 `U` 里「只是局部绑定」，当且仅当：`has_binding`（参数表里有 `N: …`，或有语句开头的 `let [mut] N`，且后接 `=`、`;` 或单个 `:`）；`U` 里 `N` 的每一处出现（排除函数自己的名字、单个 `.` 之后的字段与方法）都落在已绑定的作用域里（`if !innermost.is_name_bound {`，`crates/singlefs-unit-graph/src/closure.rs:1251`，下一行 `return false`）；`N` 不带路径（前后挨 `::`）、不是宏名；函数体里没有 `fn`、`impl`、`trait`、`mod`、`const`、`static`、`struct`、`enum`、`union`、`type`、`use`、`macro_rules`、`extern` 这十三个词（`crates/singlefs-unit-graph/src/closure.rs:910-924`，`crates/singlefs-unit-graph/src/closure.rs:1258` 判）。作用域按括号栈：`let` 到它的 `;` 才生效，生效到外层花括号收尾；参数从参数表的 `)` 起到单元收尾（`if position == signature.parameters_close && parameter_position.is_some() {`，`crates/singlefs-unit-graph/src/closure.rs:1200`）。调用处只在同名候选是 `Function`、`U` 是 `Function`、`N` 不是 use 改名的原名时才问（`crates/singlefs-unit-graph/src/closure.rs:1656-1672`）。

论证的底：Rust 值命名空间里，局部绑定（形参、`let`、模式、闭包形参、`for`、`match` 臂、`if let`）在它的词法作用域里压过同名的 item（函数）；rustc 对函数 item 与局部绑定同名不报错（只有常量、静态量、元组结构体、单元结构体、元组变体会让标识符成为模式或报 E0530，注释里 `crates/singlefs-unit-graph/src/closure.rs:28-29` 说的就是这一点，所以只遮函数）。扫描做的是「宁可多连」：任何扫不准的位置都把 `return false`。所以要找的反例是「扫描判成只是局部、而 rustc 解到外面函数」的位置。逐种：

| 绑定位置 | 扫描怎么认 | 被排掉的函数运行时调得到吗 | 判 |
|---|---|---|---|
| 形参 `N: T`、`mut N: T` | 深度 0 上紧跟 `(` 或 `,` 的 `N`，后接单个 `:`；作用域从 `)` 起到单元收尾 | 函数体里 `N` 解到形参。**推不出的一处**：`)` 之后到 `{` 之间的返回类型与 `where` 里的 `N()`（如 `-> [u8; N()]`）也被当成「已绑定」。形参在返回类型里看不看得见我没现查：我读 rustc 对 `fn f(n: usize) -> [u8; n]` 报「不是常量」（E0435）的印象是它看得见（于是解到局部、也调不到函数），但这是记忆。夹具 f3 交主 agent 编一次。 | 对得上；返回类型位置推不出 |
| 语句 `let [mut] N [: T] = …;` / `let N;` | `let` 前是 `{`、`}`、`;`，在花括号层里、不在宏调用括号里；`;` 才绑 | 之后到花括号收尾，`N` 都解到局部；初始值里的 `N`、`let` 之前的 `N` 解到外面，扫描里这两处是「未绑定」，直接 `return false`（多连）。`let N = N();`、`let N = { N() };`、`let N = || N();` 都是 `false`。 | 对得上 |
| 模式（`let (a, N)`、`let S { N }`、`let N @ …`、`ref N`） | 不认成绑定；出现在 `N` 的那个位置算「一处出现」，未绑定就 `false` | 只可能在外面已有 `let/参数` 时落进「已绑定」；那时它本来就是又一个局部。没有「把模式当绑定」的方向，不会误排 | 对得上 |
| 闭包形参 `\|N\|` | 不认成绑定；`N` 出现按所在作用域算 | 同上：外面有绑定则闭包里的 `N` 在词法上被外面压着或被闭包形参压着，两种都是局部；外面没绑定则 `false` | 对得上 |
| `for N in …` | 不认成绑定，同上 | 同上 | 对得上 |
| `match` 臂 `Some(N) => …`、`N @ …` | 不认成绑定，同上 | 同上 | 对得上 |
| `if let` / `while let` / `&& let` | `let` 前不是 `{ } ;`，不认成语句 `let`；测试 `an_if_let_does_not_bind_the_name_for_the_later_statements` 钉 | `if let N = …` 只在 `if` 体里绑，体外 `N` 解到外面函数；这套扫描不把它当绑定，所以体外 `N()` 是「未绑定」`false` | 对得上 |
| 泛型形参同名（`fn f<N>(N: N)`、`const N: usize`） | 类型参数在类型命名空间，不碰值；`N` 在参数表里出现是「未绑定」`false`；const 泛型参数没被认成绑定，同样 `false` | 多连 | 对得上 |
| 嵌套 item（`fn`、`use`、`const` 等） | 十三个词一律不遮 | item 在块里压过形参（块的匿名模块在形参 rib 里面），所以必须整体不遮，这个词表正是为此；词表覆盖手写的 item | 对得上；**宏展开出来的 item**（`my_macro!();` 展开成 `use other::N;`）词表看不见，但 workspace 内的 `macro_rules!` 单元按「调用处解」全连（`crates/singlefs-unit-graph/src/closure.rs:1592`、调用处条件 `unit.kind == Function` 排除宏单元），所以它的体里那条 `use` 或 `fn N` 的裸名把同名函数连进来；工作区外的宏展开成 item 的，推不出 |
| 宏调用参数里的 `N`（`foo!(N)`、`format!("{N}")`） | 参数里的 `N` 按所在括号层的绑定算；格式串捕获按 `references_of_tokens` 认，未绑定则 `false` | 宏实参的标识符带调用处的卫生，解到调用处的局部；宏体里写的 `N` 带宏定义处的卫生，不是实参，在宏单元里连 | 对得上 |
| `r#N` | `identifier_name` 剥 `r#` 两边同口径 | `r#N` 与 `N` 同名 | 对得上 |
| 句点 | 单个 `.` 之后的不看；`..N`（范围）算一处出现 | 字段与方法不解到自由函数 | 对得上 |

小结：扫描里没有「靠模式、闭包形参、for、match 臂、if let 去证明遮住」的路，这五种位置只会让结果更保守；真正能误排的只有「参数或 `let` 这两种它认的绑定」，而它们在词法作用域里压过同名函数这件事是 Rust 的名字解析规则本身（局部 rib 在 item 之前）。推不出的只有两处：返回类型与 `where` 里的 `N()`，和工作区外宏展开出的 item。

### 2.2 为什么扫不会漏掉「绑定之后又回到外面函数」

没有：Rust 没有把一个局部绑定「取消」回外层 item 的写法；`let` 的作用域到花括号收尾，栈在 `}` 出栈（`crates/singlefs-unit-graph/src/closure.rs:1197-1199`），行为与词法作用域一致；`macro_rules!` 局部宏、`const { }`、嵌套 `fn` 都落在十三词里。

### 2.3 件 3 的论证（use 路径点准候选）

- 论证的底：Rust 没有跨模块的词法可见性，本模块里一个裸名只能经（本模块自己的 item）、（本模块的 `use`，含 glob）、（函数体里的 `use` 或 item）、（预导入）解到；`candidates_seen_from_the_module_of_the_unit` 的并集正是这几路，且函数体里有 `use`、`名!` 后接开括号、前缀点不到、use 的任一支点不到都退回全连。新判法在「use 引进 `N`」这一路上只把「路径点到的整条模块路径」纳入（`crate`、`self`、`super` 起头的要整条相等，别的起头的只要候选模块路径以这条路径收尾，`crates/singlefs-unit-graph/src/closure.rs:1783-1813`），并要求 use 里每一支都点到（`crates/singlefs-unit-graph/src/closure.rs:1860-1872`，`match placed_by_every_path {` 在 `crates/singlefs-unit-graph/src/closure.rs:1869`，任一支为 `None` 则 `is_placement_unknown`）。对 `crate`/`self`/`super` 起头的路径，点到的模块里自己定义的名字压过它 glob 转出来的，点到的模块里没有这个名字的候选则退回全连：这两句正是 Rust 的「显式定义压 glob」与「经 `pub use` 转出来的看不出」。
- **推不出的两处（构造得出，都不是本轮新增的弱点，旧判法同样漏）**：(a) 同一模块 `M` 里有同名的非值命名空间候选（`struct N {…}`、`trait N`、`macro_rules! N`、`type N`，它们是 `is_found_by_bare_name` 的候选种类，`crates/singlefs-unit-graph/src/units.rs:103-125`）又 `pub use other::N`（函数）：`use M::N` 同时引进两个命名空间，点到 `M` 里有候选（那个 struct），于是只连 `M` 里的，`other::N` 这个函数没被连。命名规矩让它少见（struct 驼峰、函数蛇形），但没有机器挡。(b) `use a::b as foo; use foo::N;`，工作区里另有一个叫 `…::foo` 的模块里恰有同名 `N`：相对路径 `foo` 按「候选模块路径以 `foo` 收尾」认，把那个无关模块当点到的，`a::b::N` 没连。
- 判：**对得上，附 (a)(b) 两处推不出**。什么现象推翻：构造出 (a) 或 (b) 的夹具，`unit-graph closure` 里缺 `other::N` 或 `a::b::N`。

## 3 第 4 件：程序起自己，此刻到底怎么判

读了 `closure.rs` 头注释 44-50 行、`finish`（`crates/singlefs-unit-graph/src/closure.rs:2753-2870`）、`entry_is_the_main_of_a_program`（`crates/singlefs-unit-graph/src/closure.rs:2873-2884`）、`rust_items.rs` 的 `own_program_path_is_handed_to_command_unchanged`（`crates/singlefs-unit-graph/src/rust_items.rs:1762-1813`）及其调用处（`crates/singlefs-unit-graph/src/rust_items.rs:1983-2010`）。

### 3.1 判的流程（每遇到一处 `current_exe(` 词）

`crates/singlefs-unit-graph/src/rust_items.rs:1985` 起一个 `match`，四种结局，先到先得：

1. `child_test_name_at` 折得出 `--exact` 后的测试名：记 `ChildTestProcess`，闭包里连到入口目标里那个 `#[test] fn`，解不到恰好一个的在 `finish` 拒算 `current-exe`（旧规则，没动）。
2. 否则 `own_program_path_is_handed_to_command_unchanged(index, end)`（`crates/singlefs-unit-graph/src/rust_items.rs:1993`）为真：记 `OwnProgramRestart`，**这里不拒算**；拒不拒看入口，在 `finish`。
3. 否则单元里有 `Command`、`spawn`、`posix_spawn`、`fork`、`execv`、`execve`、`execvp` 任一词（`crates/singlefs-unit-graph/src/rust_items.rs:2001`，`starts_a_process`）：就地拒算 `current-exe`。
4. 否则（只问路径，如读程序文件大小、`current_exe().is_ok()`）：不拒算、不连边（`crates/singlefs-unit-graph/src/rust_items.rs:2008`）。

### 3.2 第 2 步「路径一个字不改地交给 `Command::new`」认哪些写法

- 就地：`Command::new(current_exe()?)`、`….unwrap()`、`….expect("字面串")`、`….unwrap_or_else(|…| <以 panic!(…) 或 …process::exit(…) 收尾的块或表达式>)`，或 `match current_exe() { Ok(名) => 名, Err(…) => <以 return 起头，或一定不交值的单个臂> }`；这个表达式的右括号恰好是 `Command::new(` 配对的右括号（`crates/singlefs-unit-graph/src/rust_items.rs:1680-1746` 取范围、`crates/singlefs-unit-graph/src/rust_items.rs:1776-1781` 对括号）。
- 绑名字：同一个 `let [mut] 名 [: T] = <上面那种表达式>;`，同一单元里这个名字之后不再 `let`（`let is_bound_again` 在 `crates/singlefs-unit-graph/src/rust_items.rs:1785`），名字的每一处用法只能是 `Command::new(名)`、`Command::new(&名)`、`名.display()`（`.all(|use_index| is_handed_to_command(*use_index) || is_only_displayed(*use_index))`，`crates/singlefs-unit-graph/src/rust_items.rs:1812`），至少一处是前者。
- 拒算（走到第 3 步）：`.parent()`、`.with_file_name(…)`、`.join(…)` 接在后面；`Command::new("sh").arg(名)`；名字另交别的函数；`.unwrap_or_else` 闭包交回另一条路径（最后一个表达式不是 `panic!`/`exit`）；`match` 的 `Ok` 臂不返回绑出来的名；名字被再 `let`。`child_test` 与这一步的写法里，`Command` 前缀必须逐字写成 `Command::new`（`use … as C` 之后写 `C::new(current_exe()?)` 不认，落到第 3 步，单元里若没有上面七个词则落到第 4 步放行）。

### 3.3 `finish` 对 `OwnProgramRestart` 的判（`crates/singlefs-unit-graph/src/closure.rs:2766-2778`）

对闭包里每个单元的每条 `OwnProgramRestart`：`if !entry_is_the_main_of_a_program {`（`crates/singlefs-unit-graph/src/closure.rs:2770`）才拒算 `current-exe`；入口是 main 时没有拒算也没有边。`entry_is_the_main_of_a_program` 同时要：入口目标是 `Binary`（`workspace.targets[entry_target.0].kind == TargetKind::Binary`，`crates/singlefs-unit-graph/src/closure.rs:2879`）、这一趟不按测试编（`entry_conditions.is_test` 是 `finish` 的第三个参数，`crates/singlefs-unit-graph/src/closure.rs:3236`；目标种类是测试，或请求里带 `--cfg test`，都算按测试编）、`main_function_of(entry_target)`（`crates/singlefs-unit-graph/src/workspace.rs:1693`，目标根文件顶层、无容器、无内联模块路径的第一个叫 `main` 的函数）等于唯一的入口单元（`.is_some_and(|main| entry_units == [main])`，`crates/singlefs-unit-graph/src/closure.rs:2883`）。一组入口里另有别的入口、入口是 `tests::…`、bin 里的辅助函数、模块里另一个 `main`、测试目标，一律拒算。

### 3.4 放行之后子进程跑的代码进没进闭包

进了，条件是「子进程是同一个二进制、从它的 `main` 起」：这一趟入口就是该 bin 的 `main`，闭包就是从 `main` 沿全部边收出来的，子进程换参数只在 `main` 里选另一条路，选到的单元都已在闭包里；`main` 之后读的数据文件也已按闭包里每个单元的 `reads` 收进数据单元。额外的两条：(1) 经 `CARGO_BIN_EXE_*` 请求进来的别的 bin 目标，它的 `main` 也被推进闭包（`crates/singlefs-unit-graph/src/closure.rs:3222-3226`，标签「链进来的别的目标的 main」），那个目标里的自起同样落回它自己的 `main`，也在闭包里；(2) `Command::new(current_exe())` 与 `--exact` 同时出现且测试名折得出的走第 1 步，不走这里。

### 3.5 对条款

- 条款 433 行「收不全……拒算键、照跑，不静默少收」：本件放行的前提是「子进程跑的是入口 `main` 本身」，闭包因此不少收。在这个前提成立的写法上对得上。
- 前提不成立的写法：**对不上**，构造如下（夹具 f1，已写好，等主 agent 跑，我没跑）：

```rust
let program = std::env::current_exe().unwrap_or_else(|_| {
    if std::env::args().count() > 5 { return std::path::PathBuf::from("/bin/sh"); }
    panic!("no exe")
});
let _ = std::process::Command::new(&program).arg("-c").arg("true").status();
```

  读法：`unwrap_or_else` 的闭包里先 `return` 另一条路径，最后一句 `panic!(…)`；`expression_ending_at_diverges(close - 1, open + 1)`（`crates/singlefs-unit-graph/src/rust_items.rs:1700`）只看闭包体的最后一个表达式，`}` 剥开、取前一个词，前一个词是 `panic!(…)` 的 `)`，判「一定不交值」；而闭包里的 `return` 是闭包的返回，会把 `/bin/sh` 当 `unwrap_or_else` 的值交出来。于是 `own_path_expression_range` 认这个写法，`program` 只被 `Command::new(&program)` 用，入口是 `main` 时不拒算：子进程可以是 `/bin/sh`，不在闭包里。Python 那一份同一算法（`scripts/gate/lib_function_units.py:3365`），同样放行。
  对 `match` 形的 `Err(…) => { … return …; … panic!() }`：`return` 是函数的返回、不是值，没有这个洞。
- 路径经别的单元拐一圈（`fn exe() -> PathBuf { current_exe().unwrap() }` 与 `Command::new(exe())` 分在两个函数）：`exe()` 里没有七个词、走第 4 步，起 `Command` 的函数里没有 `current_exe` 词，两边都不报：这是改动前就在的缺口（`SPAWNING_WORDS` 只看同一个单元），不是本轮新增；`crates/singlefs-unit-graph/src/rust_items.rs:53-54` 的注释写的就是「同一个单元」。
- 条款没写：D13 已定项 17 没有一句管「自起」；`.claude/kb/checks-owed.md:522`（C889）写「三类怎么处置要主 agent 定」。本件因此在条款层面是「推不出」，在代码与自己的头注释之间是上面那一处「对不上」。
- 什么现象推翻我的「对不上」：夹具 f1 `unit-graph key` 退 4 并打 `current-exe` 拒算行（那就是我读错了 `expression_ending_at_diverges`）。

## 4 第 9 件：纯改名那一批，逐块核

做法：把 `changes.patch` 每个块（`@@` 起的一块）的删行与加行各切成词（标识符、数字、单个符号），用最长公共子序列对齐；块「是纯改名」当且仅当所有不相同的段都是「标识符换标识符」、个数相等、其余词（含符号与字符串内容）逐个相同。脚本 `rename_check.py` 与它的 sha256 在草稿目录（`8da2ce01…72ec`），输出是 `changes.patch` 全部块的逐块判。改名前后的旧名在整个 `crates`、`scripts`、`.claude/kb` 里再没被引：24 个旧测试函数名我逐个 `grep -rlF` 过，零命中（`$os`、`os_is_*`、`features_with_their_cfg`、`FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG`、`two_levels_up_then_a` 也零命中，别的包里的构建脚本与测试已用新名）。

- 七份点名文件：`conditional_compilation.rs` 4 块、`lexer.rs` 1 块、`lock_file.rs` 3 块、`manifest.rs` 1 块、`shell_sources.rs` 1 块、`verus_ghost_code.rs` 1 块、`wgsl.rs` 1 块，合计 **12 块，12 块是纯改名，不是纯改名的块：没有**。
- 其中两块落在生产段：`conditional_compilation.rs` 里宏 `target_platform_table!` 的元变量 `$os` 改 `$operating_system`（匹配器里字面词 `os` 不变，体里 `("target_os", $operating_system)` 跟着改）。它是代码里的名字，改了之后宏单元的指纹变（去注释后文本不同），键里含它的闭包会换一次键，这是「改键让旧库作废不算代价」那一条管的（`.claude/kb/decisions/13-验证路线.md:437`）。别的 10 块是测试里的名字。`manifest.rs` 那一块是「proc_macro」换成「procedural_macro」，仍是标识符对标识符。
- 同一批里不在点名七份里的纯改名块另有 19 块，一并核过都是纯改名：`build_features.rs` 13 块（形参 `features_with_their_cfg` 换长名 3 块，其中含 1 块文档注释；测试改名 8 块；对 `FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS` 的引用 2 块）、`build_toolchain.rs` 1 块（`the_dev_…` 改 `the_development_…`）、`entry_groups.rs` 1 块（测试名）、`rust_items.rs` 4 块（测试名）。**一处要提醒**：脚本把 `entry_groups.rs` 的 `use` 块（`compute_closure_of_entries_with_the_features_of_the_build` 换成 `compute_closure_of_entries_not_following_the_listed_units`）也标成了「标识符对标识符」，那不是改名，是换了调的函数，属第 7 件；我没把它算进 19 块。
- 它们不是第 9 件的：`build_features.rs` 两块（新加 `#[path]` 的测试用模块与两条新测试，第 8 件）、`build_toolchain.rs` 两块（第 1 件）、`entry_groups.rs` 5 块（第 7 件）、`rust_items.rs` 5 块（第 4、5 件与注释）、`closure.rs` 37 块、`output.rs` 1 块、`rule_switches.rs` 4 块、`units.rs` 1 块、`bin/unit_graph.rs` 12 块。
- 对测试改名的副作用：测试函数名是 `crates/mutations.tsv` 第六段、门禁「按条复用的测试函数」认单元的名字；旧名在 `crates/mutations.tsv` 里零命中（上面 grep 含 `crates`），所以没有行指向已不存在的名字。

## 5 新加 80 行变异：按件分组，每个判定分支有没有行改坏它

先核可复用性：80 行每行的「原文」在被点名的文件里恰好命中一次（我用 `str.count` 对当下的树数过，80 行全是 1，没有锚点失效、没有「原文＝替换文」的行），77 个不同的「必须红的测试名」逐个在 `tests/` 或 `src/` 里找得到 `fn <名>`（`crates/singlefs-verification/src/crash_identity.rs:1268` 那一条在 `--features gpu` 下编，参数也带 `--features gpu`）。行号指 `mutation-rows-new.tsv` 里的第几行。读数都是读出来的，没跑变异。

| 组 | 行 | 行数 |
|---|---|---|
| 件 1 | 78、79、80 | 3 |
| 件 2 | 9 到 34 | 26 |
| 件 3 | 35 到 42 | 8 |
| 件 4 | 43 到 64 | 22 |
| 件 5 | 65 到 72 | 8 |
| 件 6 | 73 到 76 | 4 |
| 件 7 | 77 | 1 |
| 不属第 1–7 件 | 1 到 8（`R-裸名按模块筛` 旧规则；原文在 HEAD 里就有，HEAD 的 `crates/mutations.tsv` 里这个名字前缀零行） | 8 |

### 件 1（行 78–80）
- 罩住：`debug` 进键（78，在 `debug_and_incremental_leave_the_profile_lines_byte_for_byte_unchanged` 第一条断言处红）、`codegen-units` 出键（80，`debug_assertions_opt_level_and_codegen_units_each_change_the_profile_lines` 的 `assert_ne` 红）。
- **79 不真断在它名字说的那一处**：它把 `incremental` 加进键数组但没给 `builtin_profile` 补第七个默认值，`zip` 截成六个，`effective_profile_lines` 末尾 `effective[key]` 对 `incremental` 取不到值而 panic（`crates/singlefs-unit-graph/src/build_toolchain.rs:146` 的 `effective[key]`），测试在第一条断言之前就 panic；「只差 incremental」那条断言与「环境变量 `CARGO_PROFILE_RELEASE_DEBUG`/`_INCREMENTAL` 不影响」那条断言（第三条）没有一行变异是红在它们上的。（推的：若 79 的替换文把默认值补齐，红的位置才是第二条断言。）
- 没有变异罩着：`lto`、`panic`、`overflow-checks` 三项出键（只靠 `the_release_profile_takes_the_root_manifest_over_the_builtin_defaults` 的整行相等，没有行）；`builtin_profile` 里 dev 的 `debug-assertions`、`lto`、`panic`、`codegen-units` 默认值与 release 的 `debug-assertions`、`overflow-checks` 默认值（测试里这两项被根清单盖住了）；`test`、`bench` 的 `[""; 6]`；环境变量六项照读（只有 `OVERFLOW_CHECKS` 一项被钉，`crates/singlefs-unit-graph/src/build_toolchain.rs:302-309`）。

### 件 2（行 9–34）
- 罩住：总开关两种弄坏（9、10）、候选种类只遮函数（25）、单元种类（31）、同目标私有函数照连（11）、跨目标 `pub`（32）、泛型尖括号（18）、形参 `mut`（17）、形参表收尾才算绑定（16）、`let` 的 `mut`（19）与 `:` 臂（20）与前驱词（27）、`let` 等到 `;`（13、15）、块收尾出栈（14）、内层继承（29）、宏调用里的 `let`（26）、绑定之前的出现（12）、单个 `.` 之后（30）、路径前后与宏名三个并列条件各一行（21、22、23）、格式串捕获（28）、嵌套词（24，整个检查）、函数自己名字（33）、`has_binding`（34）。
- 没有变异罩着的分支：调用处 `!alias_originals.contains(name)`；`unit.token_range` 为空；`function_signature_positions` 的深度 0 找 `fn`、`->` 不当尖括号收尾（`text_at(parameters_open - 1) != "-"`）、`(` 配对与 `body_open` 的找法；`parameter_binding_position` 的「深度 0 上的 `,` 才是参数分隔」、单个 `:`（`text_at(position + 2) != ":"`）、标识符种类；`is_statement_let_binding_at` 的 `=`、`;` 两臂；`innermost.is_brace` 才认 `let`；`is_inside_a_macro_call: innermost.is_inside_a_macro_call || follows_a_macro_name` 的传递；单个 `.` 判里的 `..` 例外；`Some(position) == parameter_position` 的跳过；十三个嵌套词逐词；`Lifetime` 一支；栈的 `len() > 1` 守卫。
- 没有行钉的东西里最要紧的一条：「`let` 前驱词只认 `{ } ;`」只有 27 一行，而它被 27 钉的那一格（`if let` 是否算语句 `let`）正是 §2.1 表里 `if let` 一行的全部依据。

### 件 3（行 35–42）
- 罩住：`crate` 起头只点到根（35、36、37 三个形态各一行）、中间模块不算点到（38）、跨花括号往前收（39）、`super` 往上数层（40）、`self` 起头（41）、相对路径按后缀认（42）。
- 没有变异罩着的分支：use 里每一支都点到才算（任一支为 `None` 则退回全连；有测试 `branch_through_a_prelude_inside_braces_links_the_re_exported_item` 逼它，但没有行改坏这一步）；`paths_bringing_in_the_name` 为空时退回全连；叶子识别里的 `*` 与 `name`、`text_at(*position + 1) != ":"`；`use_path_leading_to` 的 `position >= use_keyword + 4`；`use_keyword` 找不到时的 `unwrap_or(first_token)`；`as` 改名那一支（Rust 退回全连，没有测试，Python 不同，见 §6）。件 3 在 Rust 这边没有弄坏开关（`rule_switches.rs` 里只加了件 2 的那一个），所以这 8 行是它唯一的机械守卫。

### 件 4（行 43–64）
- 罩住：就地写法放行（43、47，这两行改的是同一行、效果相同，`false &&` 与 `!true &&`）、`unwrap_or_else`（44、58）、`match` 的 `return` 臂与 `Err` 臂交别的路径（45、59）与 `Ok` 臂（60）、`?`（46）、入口五条件（48 到 54，其中 48 与 49 与 50 改的是同一行的三种替换）、`Command::new(…)` 的右括号对得上（55）、`is_handed_to_command` 的括号对得上（56）与「必须是 `Command::new`」（61）、`let` 右边整个是路径（57）、「每一处用法」（62）、再 `let` 同名（63）、`--exact` 点名的测试不在照旧拒（64）。
- 没有变异罩着的分支：`.unwrap()` 一支（测试里没有 `.unwrap()` 的夹具，`grep -n 'unwrap()' tests/unit_graph_own_program_restart_…rs` 只命中 `unwrap_or_else` 行）；`.expect` 的「参数是字面串」（`plain_string_content(…).is_some()`）；`panic!` 一支（夹具里只有 `process::exit`）；`expression_ending_at_diverges` 的 `;` 跳过与 `first_allowed`；`match` 形的 `Ok(`、`)`、`=>`、`,`、`Err(` 各偏移与 `close > unit_end`、`is_one_arm`；`is_only_displayed`（有夹具用到但没有行）；`any(is_handed_to_command)`（「至少起一次」）；`TargetKind::Binary`（`crates/singlefs-unit-graph/src/closure.rs:2879`；48 钉 lib、51 钉按测试编）；`main_function_of` 自身。§3.5 的 `unwrap_or_else` 闭包里 `return` 的写法没有测试也没有行。

### 件 5（行 65–72）
- 罩住：整块不拒算（65，改的是调用处）、函数名当符号（66）、名单外符号（67）、`link_name` 当符号（68）、第一条之后也核（69）、`#[link…]`（70）、别的 ABI（71）、块里的 `static`（72）。
- 没有变异罩着的分支：至少声明一条（`declared_any`，空块 `extern "C" {}`）；`pub` 跳过；`;` 找不到则不算；`keyword_index + 2 != open`；块里出现除 `#[link_name]` 之外的属性即不算；`is_identifier(position + 1)` 与 `(` 的检查。

### 件 6（行 73–76）
- 罩住：头一条边「已记不改」（73）、`unit_being_visited` 赋值（74）、只交闭包里的单元（75）、`--unit` 只给 `why`（76）。
- 没有变异罩着的分支：各处 `rule_being_followed = "…"` 标签的值与 `record_the_first_edge` 里传的名字（`methods` 传方法名、`paths` 传 `限定::成员`、`bare names` 传裸名；测试只钉了一条链的三行）；`why_lines` 的 `inside` 与跨入两支各自、链步数封顶 `0..=outcome.units.len()`、`name_text` 的 `-`；`drain` 末尾的 `unit_being_visited = None`。「件 6 不改闭包与键」这条没有变异行（改坏它的办法是让 `push` 因记边而不入队，现有靠整套键测试的回归，不是变异）。

### 件 7（行 77）
- 罩住：`units_not_followed.contains(&unit.qualified_name)` 这一个判断（77）。
- 没有变异罩着的分支：`run.units_not_followed.clone_from(units_not_followed)`（`crates/singlefs-unit-graph/src/closure.rs:3210`）；`units_reached_and_not_followed.insert`（够到的清单，验证包 `crates/singlefs-verification/src/crash_identity.rs:1049-1061` 的测试读它，但没有行）；`entry_groups.rs` 里把登记交给闭包、把 `units_reached_and_not_followed` 回传（两处）；`NotCompiled` 分支回传空表；判断放在 `activity` 之前、`return` 在 `in_closure.insert` 之前的次序。

## 6 Rust 与 Python 在第 1 到 5 件上是不是同一个判法

我把 Python 那份 `scripts/gate/lib_function_units.py` 对应函数逐段读过，没有跑。

| 件 | Python 对应 | 判 |
|---|---|---|
| 1 | `lib_function_units.py` 里没有 profile 行。Python 另有两处算工具链的地方：`scripts/gate/admission.py:867` 与 `scripts/mutation/lib_mutation_units.py:103`，都是「去掉 `debug`、`incremental`，别的 profile 键进键」的**黑名单**；Rust 是六项**白名单** | 不是同一个判法：`strip`、`rpath`、`split-debuginfo`、`package.<包>` 子表 Python 进键、Rust 不进。不触发门禁对拍（三份各算各的工具链行），但同一份清单加一行 `strip = true`，Python 两处键变、Rust 键不变 |
| 2 | `bare_name_only_a_local_binding`、`function_signature_positions`、`parameter_binding_position`（`scripts/gate/lib_function_units.py:2319-2444`）与 Rust 逐步同构：同一个括号栈（四个字段）、同样的前驱词、同样的十三个词（`scripts/gate/lib_function_units.py:2316`）、同一个并列的路径与宏名条件、同一个调用处条件（`scripts/gate/lib_function_units.py:5351`） | 同一个判法。我没逐字比的一处：格式串捕获两边用不同的函数（Python `format_captures` 加 `text[:1] in ('"','r')`，Rust `references_of_tokens`），`b"…"`、`c"…"` 这类字面量两边认法可能不同，推不出 |
| 3 | `placed_by_the_use_path`、`use_path_leading_to`（`scripts/gate/lib_function_units.py:2447-2468`、`4556-4564`）与 Rust 同构；「每一支都点到、任一支为 `None` 则退回全连」同样 | **不同的一支**：叶子是 `原名 as 名` 里的 `名` 时，Python 取它前面的原名位置往前收路径（`hits = [placed_by_the_use_path(use_path_leading_to(rust_file, use_keyword, index - 2 if rust_file.t(index - 1) == "as" else index)) …]`，`scripts/gate/lib_function_units.py:4584`），并把「`名 as` 的原名」排出叶子（`scripts/gate/lib_function_units.py:4582` 一行里的 `not in (":", "as")`）；Rust 叶子只排 `名::`（`crates/singlefs-unit-graph/src/closure.rs:1856`），`as` 之前不是 `段::`，路径是空，退回全连。夹具 f2：Rust 的闭包含 `b::aliased`，Python 的不含（我读出来的，没跑）。不影响哪一边单独少收（真正被引的原名在 `alias_originals` 里不筛），影响两份对拍。Python 另有 `rust-use-paths-flattened` 弄坏开关，Rust 没有 |
| 4 | `own_program_path_is_handed_to_command_unchanged`、`own_path_expression_range`、`expression_ending_at_diverges`、`let_binding_name`（`scripts/gate/lib_function_units.py:3337-3432`）与 Rust 逐句同构，`SPAWNING_WORDS` 同样七个词（`scripts/gate/lib_function_units.py:1332`）；入口判 `rust_entry_is_the_main_of_a_program`（`scripts/gate/lib_function_units.py:4996-5002`） | 写法判同一个（§3.5 那个 `unwrap_or_else` 洞两份同有）。入口判有一处不同：Python 比的是入口本身（`roots[0] is entry`、`entry.short == "main"`、无容器、无内联模块、`entry.file == target.root_file`），Rust 比的是「目标根文件里第一个顶层 `main`」`== [入口]`（`crates/singlefs-unit-graph/src/workspace.rs:1693`）。目标根文件里有两个顶层 `main`（`#[cfg]` 分着写）时，入口是第二个：Rust 拒算、Python 放行（Rust 偏严，不少收） |
| 5 | `extern_block_declares_only_listed_c_library_calls`（`scripts/gate/lib_function_units.py:3434-3467`）、名单 `scripts/gate/lib_function_units.py:1334` 与 Rust `crates/singlefs-unit-graph/src/rust_items.rs:68` 同一份 | 同一个判法。形态名不同：Rust 是 `extern-function-interface`，Python 是 `extern-block`（Python 文件头 354 行自己写了这一点） |

## 7 检查单八格

### 7.1 走不到的理由二次查实：**成立**（四处全成立）

`open_delimiters` 的全部操作在 `crates/singlefs-unit-graph/src/closure.rs:1159-1262`：建一层（`let mut open_delimiters = vec![…]`，`crates/singlefs-unit-graph/src/closure.rs:1159`），唯一的 `push` 在 `crates/singlefs-unit-graph/src/closure.rs:1188`，唯一的 `pop` 在 `crates/singlefs-unit-graph/src/closure.rs:1198` 且被 `if open_delimiters.len() > 1 {`（`crates/singlefs-unit-graph/src/closure.rs:1197`）守着（我用 `awk 'NR>=1144 && NR<=1262 && /open_delimiters/'` 列出，只有这八行）。没有 `clear`、`truncate`、`remove`、`drain`。所以不变式「长度至少 1」成立，逐处：
- `crates/singlefs-unit-graph/src/closure.rs:1169`（`.last()` 在每个词法单元的开头）：这一轮之前的所有出栈都被 `len > 1` 守着，成立。
- `crates/singlefs-unit-graph/src/closure.rs:1203`（`)`、`]`、`}` 臂里 `pop` 之后立刻 `last_mut()`）：`pop` 只在 `len > 1` 时做，做完 `len >= 1`，成立。
- `crates/singlefs-unit-graph/src/closure.rs:1210`（`;` 臂）：这一臂里没有出栈，进臂前 `len >= 1`，成立。
- `crates/singlefs-unit-graph/src/closure.rs:1231`（标识符臂，认出语句 `let` 之后）：同上，成立。
别的文件：生产段新加的 `unreachable!`、`panic!`、`expect`、`unwrap`、`assert!`、`debug_assert`、`expect_err`：我对 `git diff HEAD -U0` 十六份的加行、按「`mod tests {` 之前」划生产段各数了一遍，只有这四处（`crates/singlefs-unit-graph/src/closure.rs:1169、1203、1210、1231`），其余十五份零处，与事实表一致。附带：新代码里的下标运算我逐处看过守卫，`crates/singlefs-unit-graph/src/closure.rs:945`（`tokens[position - 3]`，守着 `position >= use_keyword + 4`）、`crates/singlefs-unit-graph/src/closure.rs:1187`（`tokens[position - 2]`，守着 `position >= 2`）、`crates/singlefs-unit-graph/src/closure.rs:1258`（`tokens[signature.body_open..]`，`body_open` 是从同一切片里找出的位置）、`rust_items.rs` 里 `use_index - 1/-2`（`use_index` 取自 `expression_end` 之后，`expression_end` 在 `let` 之后，不会小于 3）都安全；`close_of` 的 `expect`（`crates/singlefs-unit-graph/src/rust_items.rs:737`）是旧函数，新调用点都在「该位置是 `(` 或 `{`」的判断之后。

### 7.2 类型与名字：**成立**（附事实表几处小错）

- 生产段新加的 `_ =>` 九处我逐处看了被匹配的值：`crates/singlefs-unit-graph/src/closure.rs:961`（`match text_at(cursor)`）、`:1003`（`match text_at(position)`）、`:1017`（`match text_at(parameters_open)`）、`:1044`（`match text_at(position)`）、`:1078`（`match word`）、`:1124`（`match text_at(name_position + 1)`）、`:1216`（`TokenKind::Punctuation => match text`）、`crates/singlefs-unit-graph/src/rust_items.rs:1673`（`match self.text_of(last_token)`）、`:1702`（`match self.text_of(after_call + 1)`）：被匹配的全是词法记号的文本（`&str`），不是封闭枚举，没有可穷举的集合。外层 `match token.kind`（`crates/singlefs-unit-graph/src/closure.rs:1170`）列全了 `Lifetime`、`Literal`、`Punctuation`、`Identifier` 四种（`crates/singlefs-unit-graph/src/lexer.rs:9-14` 的 `TokenKind` 恰好这四种），没有 `_`。
- 没有新加 `as` 转换（对 `changes.patch` 加行 grep ` as (u8|…|usize|…|f64)` 零命中）。
- 名字：新加的函数、类型、常量，不含缩写，名字与职责对得上；事实表「新加的条目名」把几个**不是新加的**也列进去了：`closure.rs` 的 13 个里 `fn finish` 是旧函数改了签名，只有 12 个新加；`build_toolchain.rs` 的 2 个（`PROFILE_SETTING_KEYS`、`builtin_profile`）与 `rule_switches.rs` 的 `EVERY_RULE` 都是旧名改了类型或长度。

### 7.3 不兼容旧代码与旧格式：**成立**

- 加行（含注释）对「旧版|旧布局|旧格式|legacy|migrat|兼容旧|backward|迁移」grep 零命中（我对 `changes.patch` 加行跑了 `grep -iE`，零行）。第 1 件改键的内容让旧库按旧键存的判定不再命中，没有读旧键的分支：`profile.debug:`、`profile.incremental:` 这两个行前缀在仓里没有别的读者（`grep -rn 'profile\.debug\b\|profile\.incremental' crates scripts .claude/gate.d` 只命中 `profile-debug-keyed` 这个弄坏开关名）。条款 437 行写明「改键让旧库作废不算代价：不做兼容迁移」，对得上。

### 7.4 不新建判定库：**成立**

- 加行对 `rocksdb|sqlite|prune.server` grep 零命中；`why` 只往标准输出打行（`bin/unit_graph.rs` 里 `return (lines, summary, 0)`）；`first_edges`、`units_reached_and_not_followed` 是内存里的 `BTreeMap`、`Vec`，随 `ClosureOutcome` 交回，不落盘。

### 7.5 所有任务多机可用：**成立**

- 加行对 `hostname|/proc/|/sys/|env::var|std::env` grep 零命中。第 1 件：别的 `CARGO_PROFILE_*` 照旧读：`effective_profile_lines` 对六个键各取一次环境变量（`crates/singlefs-unit-graph/src/build_toolchain.rs:137-141`），调用方 `crates/singlefs-verification/build.rs:127` 交的是 `std::env::var(name).ok()`，并且构建脚本对同一六个键各发一次 `rerun-if-env-changed`（`crates/singlefs-verification/build.rs:286-289`，它迭代的就是缩过的 `PROFILE_SETTING_KEYS`）。所以两台设得不一样时（比如一台设了 `CARGO_PROFILE_RELEASE_OPT_LEVEL`）键跟着不一样，这是旧行为没变；两台只在 `CARGO_PROFILE_RELEASE_DEBUG` 或 `_INCREMENTAL` 上不一样时键不再不一样，这是第 1 件的本意，不是多机缺陷。测试里被钉的只有 `OVERFLOW_CHECKS` 一项环境变量（`crates/singlefs-unit-graph/src/build_toolchain.rs:302-309`），其余五项靠同一个循环。

### 7.6 测试优先多线程：**成立**

- 加行对 `available_parallelism|test-threads|RAYON|thread::` 零命中；新测试是库单测与 `tests/` 小夹具（只切词、不编译），没有重活。

### 7.7 带 `--item` 的命令与项：**成立**

- `grep -c 'unit-graph\|unit_graph' .claude/gate.d/stage-inputs.tsv .claude/gate.d/crash-point-replay-items.tsv` 两份都是 0。新加的 80 行变异由门禁 `crates-mutation-replay.sh --item <行号或变异名>` 按行点名（该脚本文件头写明 `--item` 收行号或变异名原样）；新测试随测试目标点名。

### 7.8 新崩溃流多罩了什么：**成立**

- 加行（含注释）对「crash|崩溃」grep 零命中；十六份没有新加崩溃点重放流或崩溃枚举用例。

## 8 事实表 `facts.md` 写错的地方（线索，不是依据）与正文里一处观测

1. `facts.md` 224 行与正文第 7 件：「`entry_group_key_with_the_features_of_the_build` 的返回类型换成新类型」（事实表写「返回值类型由 `EntryGroupKey` 改成新类型（L166、L239）」）**不对**。它的签名仍是 `-> Result<EntryGroupKey, EntryError> {`（`crates/singlefs-unit-graph/src/entry_groups.rs:116`），改成转调新函数后 `.map(|keyed| keyed.key)`；L166、L239 属于新函数 `entry_group_key_not_following_the_listed_units`。仓里别处调它的（`crates/singlefs-background-gpu-tests/build.rs:94`、`crates/singlefs-verification/src/bin/e182_unreadable_abandoned_root_forks.rs:2087`）不用改。
2. `facts.md` 73 行 `build_features.rs`「改到/新加的函数名 3 个……两处都没引到的 1 个：`environment_name_of`」：`environment_name_of` 没改，它只是块头（`@@ … @@ fn environment_name_of`）里「上一个函数」的上下文；真正改的是它后面的 `features_of_the_crate_being_built`（形参改名）。同类的误取（块头函数当改了的函数）在事实表别处可能还有，我只核了这一处。
3. `facts.md` 156 行 `crates/singlefs-unit-graph/src/closure.rs:29：E053`：把注释里的 rustc 错误码 `E0530` 当成 kb 的 E 编号并且还截成 `E053`；它不是 kb 编号，也没有这个编号的实验。
4. `facts.md` 174 行「类型与名字：新加的条目名 13 个」里 `fn finish` 不是新加（§7.2）；`facts.md` 97 行 `build_toolchain.rs` 的 2 个与 `rule_switches.rs` 的 `EVERY_RULE` 同理，都是旧名改了类型或长度。
5. `facts.md` 109 行把件 1 的键组成指到「`.claude/rules/verification.md`「一个门禁：两种跑法」」：键的组成在 `.claude/kb/decisions/13-验证路线.md:431`（已定项 17），不在那一节；我没打开 `verification.md` 逐字查那一节有没有同样的句子，只说引的位置不是条款原处。
6. `facts.md` 182 行「两处都没引到（grep 名字）的 9 个」是按函数名在 `tests/` 与同文件测试模块里 grep 的结果，不等于没被测到：`is_bare_name_only_a_local_binding_of_the_function` 这类私有函数经 `unit-graph` 的公开入口被 26 行变异（件 2）与 `tests/unit_graph_local_bindings_shadow_same_named_functions.rs` 的用例逼到。读它时别把「没引到」读成「没测」。
7. 事实表其余我核过对得上的：每份文件的块数（`git diff HEAD -- <文件> | grep -c '^@@'` 与表头一致：`bin/unit_graph.rs` 12、`build_features.rs` 15、`build_toolchain.rs` 3、`closure.rs` 37、`conditional_compilation.rs` 4、`entry_groups.rs` 6、`lexer.rs` 1、`lock_file.rs` 3、`manifest.rs` 1、`output.rs` 1、`rule_switches.rs` 4、`rust_items.rs` 9、`shell_sources.rs` 1、`units.rs` 1、`verus_ghost_code.rs` 1、`wgsl.rs` 1）与加减行数（`git diff HEAD --numstat` 对得上）；生产段新加 `expect` 四处的行号；`_ =>` 九处的行号；新增变异 80 行的文件分布（`closure.rs` 54、`rust_items.rs` 22、`build_toolchain.rs` 3、`bin/unit_graph.rs` 1，我从 `mutation-rows-new.tsv` 第二列数了：与表一致）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 件 1 profile 8 项缩 6 项 | 推不出 | 条款 431 行没说档设置哪几项算「生效构建环境」；「`debug`、`incremental` 不改结局」只有审核记录一句，`codegen-units` 那半句同样没核；`codegen-units` 缺省随 `incremental` 变（凭记忆）会让「cgu 进键、incremental 不进」自相矛盾 |
| 1 件 2 局部绑定遮同名函数 | 对得上 | 靠 Rust 的 rib 次序（局部压 item）；模式、闭包形参、`for`、`match` 臂、`if let` 都不被当绑定、只会更保守；一处推不出：返回类型与 `where` 里的 `N()` |
| 1 件 3 use 路径点准候选 | 对得上 | 只把「路径恰好点到的模块」纳入，任一支点不到退回全连；两处推不出：同模块非值命名空间同名候选加 `pub use` 转出函数、`use a::b as foo; use foo::N;` 撞名 |
| 1 件 4 程序起自己 | 对不上（一形）+ 条款没写 | `unwrap_or_else` 闭包里先 `return` 另一条路径、末句 `panic!`，被认成「一个字不改」，入口是 `main` 时放行，子进程可以不是自己；Python 同洞；D13 已定项 17 没有一句管自起（C889 记「要主 agent 定」） |
| 1 件 5 extern 块登记两个系统库调用 | 推不出 | 条款没写 FFI；代码前提「同 `std::fs` 一层」自洽但是自立的；与 Python 同一份 |
| 1 件 6 首边与 `why` | 对得上 | 记边的字段只在 `why` 里读，不进 `reuse_key`；`push` 与三个规则里只加记录一行 |
| 1 件 7 不沿点名单元收 | 对得上 | 条款 434 行已点名这两个函数；空登记逐字相同；少收那一截由验证包五条测试担，不在本轮；事实表与正文把返回类型说错 |
| 2 件 2 逐种绑定位置 | 对得上，一处推不出 | 形参、`let` 认；模式、闭包形参、`for`、`match` 臂、`if let` 不认且只会更保守；推不出：形参作用域是否含返回类型 |
| 2 件 3 | 对得上，两处推不出 | 同上行 |
| 3 件 4 此刻怎么判 | 写清（§3.1 到 §3.4） | 四结局；放行的只有「入口是 bin 的唯一 `main`、不按测试编、路径一个字不改」；放行后子进程跑的代码都在 `main` 的闭包里 |
| 4 第 9 件纯改名 | 七份点名文件 12 块全是纯改名，不是的：没有 | 同批别处另有 19 块纯改名；脚本把 `entry_groups.rs` 的 `use` 块误标为纯改名，那是第 7 件 |
| 5 新 80 行变异 | 罩不全 | 80 行锚点都在、各点名测试都在；79 不红在它名字说的那条断言上；各件没行罩的分支逐条列在 §5 |
| 6 Rust 与 Python | 件 2、5 同；件 4 写法同、入口判有别；件 3 的 `as` 支不同；件 1 不是同一个判法 | 夹具 f2 待主 agent 跑 |
| 7 检查单八格 | 八格全成立 | 走不到的四处 `expect` 都成立；九处 `_ =>` 全是对词法记号文本的 `match` |
| 8 事实表 | 写错 6 处（§8） | 含正文第 7 件返回类型那一句 |

## 要主 agent 跑的命令

我没有往统一任务服务交任务，下面都是整条的字面命令，读数一律是「推的」。路径里的 `<草稿目录>` 就是 `/home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward`，下面已写成全路径。

1. 件 4 的洞（夹具 f1，我的读法下 `key` 应当算得出键、不拒算；若退 4 并打 `refused	current-exe`，则 §3.5 撤回）：
   `cargo run -q -p singlefs-unit-graph --bin unit-graph -- key /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward/fixtures/f1-restart-closure-return frestart::main --target bin:frestart --toolchain-line 'host: x86_64-unknown-linux-gnu'`
2. 件 3 的 `as` 分歧（夹具 f2；我的读法下 Rust 一行有 `fn	fuse::b::aliased`、Python 没有，其余相同）：
   `cargo run -q -p singlefs-unit-graph --bin unit-graph -- closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward/fixtures/f2-use-alias fuse::user::read --toolchain-line 'host: x86_64-unknown-linux-gnu'`
   `python3 scripts/gate/lib_function_units.py closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward/fixtures/f2-use-alias fuse::user::read --toolchain 'host: x86_64-unknown-linux-gnu'`
3. 件 2 返回类型位置（夹具 f3；编过＝返回类型里的 `name()` 指外面的 const fn、形参不在返回类型的作用域里，件 2 对这一位置多排了；报错＝形参看得见，件 2 的这一格推得出）：
   `mkdir -p /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward/f3-out`
   `rustc --edition 2021 --crate-type lib --emit=metadata --out-dir /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward/f3-out /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward/fixtures/f3-return-type/return_type_scope.rs`
4. 这批改动的单元与集成测试（读数：全绿）：
   `task-server/task submit -- cargo test -p singlefs-unit-graph`
5. 变异行（点名我标「不红在名字那一处」的 79 与有分支没罩的几行，看红的位置；整表另判）：
   `task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --item "构建档键把 incremental 放回进键的 profile 设置"`
   件 7 那一行（带 `--features gpu`，在验证包里编）：
   `task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --item "判器键：登记了不沿它收的单元照收（单元图不认登记）"`
6. 只读复核（不要服务）：
   `sha256sum -c research/prompts/unit-graph-src-backfill-code-r1-snapshot/crates-sha256.txt`
   `python3 -I /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward/rename_check.py research/prompts/unit-graph-src-backfill-code-r1-snapshot/changes.patch`（§4 的逐块判，31 行 `PURE-RENAME`，其中 `entry_groups.rs` 的 `use` 块要人看）
   `python3 -I /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-forward/anchor_check.py /home/fy5090/code/singlefs research/prompts/unit-graph-src-backfill-code-r1-snapshot/mutation-rows-new.tsv`（§5 的锚点与测试名计数，末行 `distinct tests 77`）
7. 件 1 的「cgu 缺省随 incremental」（我凭记忆写的）：读 cargo 参考手册 Profiles 一节的 `codegen-units` 与 `incremental` 两条；本机 `~/.rustup/toolchains/*/share/doc/cargo` 里没有离线手册，没现查。

草稿目录里文件的 sha256：`rename_check.py` `8da2ce018fd0b9026893dc55450bba38563b95d740bb5eace31aa6a995a972ec`；`anchor_check.py` `d8a2207e820c669f8d59ad436475e1ad235622ecd18d620e8c872ebf4179fbfc`；`fixtures/f1-restart-closure-return/crates/frestart/src/main.rs` `f1e8bd776a1954b234910b8c03d7aba582c177b9a8d998fb3b5081c6a2f6de72`；`fixtures/f2-use-alias/crates/fuse/src/lib.rs` `929734f09a3fbcb7f0154aed4e205159ac2e26a5c956f05448e373e5abee110f`；`fixtures/f3-return-type/return_type_scope.rs` `1791b61d86b8a7c58314fb807c58cad237cbdceaf365270a115d16ca533bf8a2`。

## 没做什么

- 没跑任何测试、变异、装置、`cargo`、`rustc`；没往统一任务服务交任务；§1 到 §8 里凡写「读出来的」「推的」的都没有复跑。
- 不替主 agent 采纳或出判决；没读攻方腿的报告与别的轮的判决。
- 没逐份判 `tests/` 下 21 份测试，只读了与 src 改动对口的几份（`unit_graph_local_bindings_shadow_same_named_functions.rs`、`unit_graph_own_program_restart_is_keyed_only_from_the_main_of_a_bin.rs`、`unit_graph_use_paths_link_only_the_module_they_name_and_fall_back_through_re_exports.rs`、`unit_graph_command_line_prints_one_item_per_line.rs` 的 `why` 三条，以及 `crates/singlefs-verification/src/crash_identity.rs:1268` 那一条）来核变异点名的测试断在哪。
- 没核 Python `format_captures` 与 Rust `references_of_tokens` 对 `b"…"`、`c"…"` 字面量的逐字等价（§6 件 2 标了推不出）；没打开 `.claude/rules/verification.md` 查 §8 第 5 条；没核 `rule_being_followed` 各标签值的对错。
- 没写模型，所以没有 `tools/rerun.sh` 与包装它的复跑脚本。

## 附录 A：引的 kb 行原样（`.claude/kb/decisions/13-验证路线.md`，已定项 17 从第 428 行标题到第 457 行下一个标题之前整条读过；下面是正文引到的几行）

行 431：
- 门禁、崩溃判定库与实验准入的每一份判定（崩溃状态块、按条复用的每条测试函数、crates 变异每行、广谱变异每条、形式化每条、QEMU 每档、litmus 每份、研究复跑每行、实验变异表每条、实验准入）沿不沿用，只看它的单元键：它依赖的函数单元各自去注释后的指纹，加闭包里各单元（含 mod 头）的 `#[cfg]`、`#[cfg_attr]`、`cfg!` 真引到的 cfg 原子、各带这一次目标下的真值（闭包没引到的原子不进键：带不带 `gpu` 特性编的两趟，闭包不提 `gpu` 的判定键相同、共用一份库），加工具链（含 cargo 解析后的生效构建环境与外部工具版本）、从 `Cargo.lock` 解析出的闭包真引到的依赖版本，加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）。函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。

行 432：
- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。

行 433：
- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。

行 434：
- 三种排除不进键，理由不同、分开记。文件级：`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份（算键的库自己与单元算法 `scripts/gate/lib_function_units.py`）按角色排，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键，crates 变异判法（`scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/lib_mutation_units.py`）除外：那里这两份照进判法单元指纹，覆盖号也带判法单元指纹。边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；边的清单以 `scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES` 登记为准，登记了的都只出提示，按这些边排出键。边级排除要配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达。另有一格「键不变而退出码变」必红。单元级：Rust 一侧按判器组、按单元（限定名）登记「不沿它收」，收闭包时够到登记的单元，它不进这一组的闭包、不从它往下收（经别的边够得到的单元照进），这一趟真够到的几个另外交出；登记放在判器组入口登记表旁（`crates/singlefs-verification/src/judging_units_entries.rs` 的 `units_not_followed_of_the_group`），今天只有 GPU 判器那一组有登记（`UNITS_OF_THE_MEMORY_LEDGER_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW` 与 `UNITS_REACHED_ONLY_THROUGH_WRAPPER_DEVICES_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW`），别的组是空的；收法在 `crates/singlefs-unit-graph/src/closure.rs` 的 `compute_closure_of_entries_not_following_the_listed_units` 与 `crates/singlefs-unit-graph/src/entry_groups.rs` 的 `entry_group_key_not_following_the_listed_units`，登记是空的，键与不带登记的逐字相同。登记要配五条测试（`crates/singlefs-verification/src/crash_identity.rs`）：不带登记收时够到的恰好等于登记的（陈旧的登记与新进来的单元都红）；登记的每个名字在工作区里恰好一个单元；不沿它收少掉的恰好是登记的那些；留下的单元不写登记里的包装类型名；改一个登记了的单元，这一组的键不变。python 模块有单元进闭包时，它在 import 时执行的模块级语句一起收进来：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头。

行 437：
- 改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。

行 440：
**射程**：管门禁、崩溃判定库与实验准入的复用键；不管静态门禁「只判新加行」的改动范围（`scripts/gate/changed-paths.sh`，那是判据语义）与 preflight 的 `admission: inputs-changed`；不管枚举域（已定项 4）、分层与提交时跑什么（已定项 9）。镜像判定与给定状态判定的版本是判定定义那一组（CPU 参照判器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` 的闭包）连同 GPU 判器那一组（含 WGSL 内核）的单元一起算出的键，不论这一块在 CPU 还是 GPU 上判的，键相同，改 GPU 判器那一组的任一单元，这一类的每一块（CPU 判过的也在内）都作废、现判：GPU 判器是加速路径，它的准入靠与定义那一组的对拍（GPU 与 CPU 对拍用例），块不按设备分种类；显卡驱动、卡型、几张卡怎么分活留作块的属性或跑的参数，不进键。键算法里不设「包装类型的 impl 等类型被点名」这类通用规则：三轮三方都被打中，已撤回，键算法本身没动；GPU 判器那一组要切的几类（故障注入、读计数、核心层挂载这类包装设备）走上面的单元级登记。
