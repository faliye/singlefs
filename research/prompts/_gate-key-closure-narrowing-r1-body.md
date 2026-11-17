# gate-key-closure-narrowing-r1：复用键的闭包与编译档设置能收窄到哪（2026-11-14）

## 一、题面

2026-11-13 起的六道提交门禁里，crates 变异表 7986 行有 7928 行要现判、6.4 小时只判了 2261 行；改一处门禁脚本或算键库，整类判定一起作废。静态审核（`records/2026-11-14-门禁静态审核.md`）把原因归到三样：判法闭包收得比「会改判定结局的代码」宽得多；键里带着不改判定结局的编译档设置；编译档本身让每次重判都慢。用户 2026-11-14 定「三样并成一次做」、只作废一次，闭包怎么收窄先走一轮三方。

这一轮判的是下面八条改法各自收窄之后，键还是不是 D13（验证路线） 已定项 17 要的那把键：有没有一种改动，改了判定结局而键不变。方案按 `crates/` 与 `scripts/` 今天的实现来谈。

## 二、实现今天的样子（主 agent 的观测，2026-11-14）

读过的路径与看到的事实：

- `crates/singlefs-unit-graph/src/build_toolchain.rs:17-26`：进键的 profile 设置是 `PROFILE_SETTING_KEYS` 八项——`opt-level`、`debug`、`debug-assertions`、`overflow-checks`、`lto`、`panic`、`incremental`、`codegen-units`。
- `scripts/gate/admission.py:847`：`BUILD_KEYS_LEFT_OUT = frozenset(("jobs", "target-dir", "incremental"))`，只管 cargo 配置 `[build]` 那张表；`admission.py:18-21` 文件头写 `[profile.<名>]` 整张表摊开进键。
- 仓根 `Cargo.toml:22-27`：`[profile.release]` 是 `opt-level = 3`、`debug = true`、`overflow-checks = true`、`debug-assertions = true`，没写 `incremental`（release 缺省关）。
- `crates/singlefs-verification/src/heavy_threads.rs:22`：`REGISTRATION_FILE_TEXT` 经 `include_str!(concat!(…))` 把仓根 `.claude/heavy-thread-default` 整份编进来。
- `scripts/gate/lib_function_units.py:2028-2034`：`rust_lex_cache_namespace` 用 `open(os.path.realpath(__file__), "rb")` 把这份库整份读进 sha256，当 Rust 词法缓存的目录名。
- `scripts/gate/lib_function_units.py:10513`、`14534`：有单元读自己原文时，整份文件字节收成一个 `source-text` 数据单元进闭包。
- `scripts/gate/lib_function_units.py:9091-9097`：碰到名字或属性 `__doc__` 就记一次「读文档串」，不看是读（`ast.Load`）还是写（`ast.Store`）；`lib_function_units.py:1469` 有一条模块级 `InlineScript.__doc__ = """…"""`，是写。
- `scripts/gate/lib_function_units.py:10539`、`14933-14934`：shell 文件里凡是形如「# 英文词:」的注释整批收成一个 `kept-comments` 单元，闭包碰到这份文件的任何单元就带上它。
- `scripts/gate/lib_function_units.py:13358`：`NOT_JUDGING_FILES` 把算键库与 `scripts/crash/lib_verdict_library.py` 排出一切判定的键，crates 变异判法除外（D13（验证路线） 已定项 17 定案第四条原文）。
- `scripts/mutation/crates-mutation-rows.py` 的判法入口闭包照「import 时执行的模块级语句随模块进闭包」把这份驱动与算键库的全部模块级赋值收进来（`.claude/rules/verification.md`「一个门禁：两种跑法」第一条原文：「python 模块有单元进闭包时，它在 import 时执行的模块级语句一起进闭包：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头」）。

## 三、量到的（主 agent 交的任务 16712 的输出，逐行读过）

crates 变异判法闭包此刻 2105 个单元：模块级赋值 948、文档串 558、def 230、def 头 166、import 128、class 48、数据文件 16、模块体 9、整份源文本 1、数据目录 1。按文件：`scripts/gate/lib_function_units.py` 1485、`scripts/mutation/crates-mutation-rows.py` 312、`scripts/mutation/lib_mutation_units.py` 87、`scripts/gate/lib_reuse_checks.py` 67。输出里有两行 `data-file scripts/gate/lib_function_units.py:1-23684` 与 `source-text scripts/gate/lib_function_units.py:1-23685`：这份库改任何一个字节（注释也算），7986 行的判法指纹都变。558 段文档串全部出自算键库。16 个数据文件里有被判树的 `Cargo.lock`、仓根与各 crate 的 `Cargo.toml`、`.claude/heavy-thread-default`，数据目录是 `crates/singlefs-unit-graph/src`。

推的、没量过的：每条改法各自能让闭包少多少单元；文档串整批进闭包是不是只因为第 1469 行那一条赋值。

## 四、八条改法（每条写成对实现的改动）

1. **算键库不读自己整份文件**：`rust_lex_cache_namespace` 的缓存目录名改取切 Rust 词法单元那几个函数（`rust_lex` 与它调到的帮手）的单元指纹加弄坏开关，不 `open(__file__)`。闭包里那两个整文件单元随之消失。
2. **写 `__doc__` 不算读**：`lib_function_units.py:9091-9097` 只在 `ast.Load` 上记读文档串；属性那一支同样看 `node.ctx`。
3. **自证的夹具与格名表搬出被判法载入的模块**：算键库、`crates-mutation-rows.py`、`scripts/mutation/broad-mutation.py` 里只给 `--selftest` 与弄坏开关用的模块级常量（夹具文本、格名、开关表）挪到另一份只在 `--selftest` 里载入的文件。「模块级语句随模块进闭包」的规则不动。
4. **判法指纹里去掉被判树的数据文件**：算键库给自己编 unit-graph 程序算缓存键时读的 `Cargo.lock`、`.claude/heavy-thread-default` 与 `crates/singlefs-unit-graph/src`，改成经不进闭包的引用取（同 `WORKER_STATE` 的做法），不当判法的数据单元。依赖版本、特性、工具链照旧各在每行键的自己那一栏里。
5. **`kept-comments` 只收真被读的指令词**：`KEPT_SHELL_COMMENT` 从「任何英文词加冒号」收到一张登记的词表（`admission`、`run-condition`、`selftest-serial`、`task-server-*`、`gate-cell`、`gate-covers`、`shellcheck` 这类有程序读的），别的注释照普通注释去掉。
6. **记账代码出判法闭包**：`crates-mutation-rows.py` 的 `judge_row` 拆成「改坏源码、起 cargo、照输出判档」的核心与「写记录、写用时表、量占盘、拼说明文字」的外壳，外壳经不进闭包的引用调。
7. **编译档里不改运行结果的设置移出键**：Rust 一侧 `PROFILE_SETTING_KEYS` 去掉 `debug`、`incremental`、`codegen-units`；python 一侧 `[profile.<名>]` 摊开时去掉同三项。随后仓根 `Cargo.toml` 的 `[profile.release]` 改 `debug = "line-tables-only"`、门禁起 cargo 时开 release 增量。
8. **重活线程数的登记文件只把数编进二进制**：`.claude/heavy-thread-default` 只留数、说明挪到不被 `include_str!` 的另一份；或 `heavy_threads.rs` 读的那一侧只取第一条非注释行（后一种要改 Rust 两份实现的数据单元规则）。

## 五、要回答的问题

1. 逐条：收窄之后，构造得出一种改动——它改了某一份判定的结局（绿变红、红变绿、抓到变没红），而这份判定的键不变吗？构造得出就给出改哪个文件哪一行、哪一份判定的结局怎么变；构造不出写「推不出反例」并写查过哪些路径。
2. 第 1、3、4、6 条碰的是 D13（验证路线） 已定项 17 定案第四条里「crates 变异判法除外：那里这两份照进判法单元指纹」那一句。收窄之后这一句还成不成立，要不要改写；改写成什么才不落进同一条定案第二条的「键过窄」。
3. 第 7 条：`debug`、`incremental`、`codegen-units` 三项里，有没有哪一项会改测试的可观测行为（panic 信息里的位置、`#[track_caller]`、回溯文本被断言比对、`codegen-units` 影响浮点或内联从而改结果、增量编译的已知错编）。本仓的测试有没有断言这类输出。
4. 第 5 条：登记的词表漏一个真被读的指令词时，会怎样；用什么自证让漏登必红。
5. 八条里哪几条要配新的自证格与弄坏开关才算立住，各自「该红的输入」是什么。
6. 八条之外，同一份闭包清单里还有没有同类的宽（收了不改结局的东西），或者这八条里有哪一条收错了方向（把会改结局的东西收掉了）。

## 六、压着的条款

D13（验证路线） 已定项 17「判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成」，整条在附录；尤其定案第二条末句「键过窄（引用的常量、类型、宏漏收）……同样算违反」、第三条「收不全……的那一份拒算键、照跑，不静默少收」、第四条三种排除与边级排除的三格自证、第七条「改键让旧库作废不算代价」。
