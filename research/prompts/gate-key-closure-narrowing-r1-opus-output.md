# gate-key-closure-narrowing-r1 攻方腿（Opus）报告（2026-11-14）

立场：假设八条改法收窄之后的键仍是 D13（验证路线） 已定项 17 要的那把键是错的，造「改了判定结局而键不变」的改动。攻击面：① 八条逐条找反例；② 第 7 条三项编译档设置会不会改测试的可观测行为；③ 八条里哪一条收错了方向。没读禁读清单里的两份文件。

## 复跑命令与文件

这一条腿没建模型目录（没有对被判系统建模）。用过两样东西，都在草稿目录 `/home/fy5090/.local/share/singlefs/work/gate-key-closure-r1-opus/`：

| 文件 | sha256 | 是什么、怎么跑 |
|---|---|---|
| `scan/selftest_constant_users.py` | `8b331f4e4d6d6d0155d7141a56b5a94af5f8bdfbff0ce150247e7985abf94e71` | 只读的 ast 扫描（不 import、不执行被扫的脚本）：列出名字以 SELFTEST 起头、却被名字不带 selftest 的函数或模块级语句读到的模块级常量。我在仓根直接起过：`python3 -I /home/fy5090/.local/share/singlefs/work/gate-key-closure-r1-opus/scan/selftest_constant_users.py scripts/mutation/crates-mutation-rows.py scripts/mutation/broad-mutation.py scripts/gate/lib_function_units.py scripts/mutation/lib_mutation_units.py`，输出落 `scan/selftest-users.txt` |
| `heavy-thread-probe/Cargo.toml` | `b8d1fc43fdb57babbb0115d49994be3a02c414d6ad3d2b5dc846a9417d56bc84` | 第 8 条的探针（独立小包，不在仓的工作区里），没跑，命令在「要主 agent 跑的命令」 |
| `heavy-thread-probe/src/main.rs` | `bb52131a1ef37fd50f15fc32f2cea4f20b8b0c43dd397a28edf5891d73ccced8` | 同上 |
| `heavy-thread-probe/src/copied_heavy_threads.rs` | `a6cf6e74aee9b1c446a539f2d058a62e1fa2cf2f9ed0487a0e892951d7a6685c` | `crates/singlefs-verification/src/heavy_threads.rs` 第 35–78 行逐字拷贝（`sed -n '35,78p' … | sha256sum` 与它同值） |
| `heavy-thread-probe/src/copied_e184.rs` | `b8b40067984f1a24b81a2897eab26f8f992cbe3b8b81a65b65a1f3a738db6567` | `crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts_write_dump/mod.rs` 第 75–106 行逐字拷贝（同法核过同值） |

报告里的读数除标「量的」的几处 grep 与 ast 扫描输出外，一律是「推的（按代码读）」：没有一处在入库装置或副本上跑过。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| ①-1 算键库不读自己整份文件 | **构造得出（条件式）** | 缓存目录名只取函数单元指纹时，决定 `rust_lex` 输出的模块级常量（词种名）与缓存编解码不在名里；改词种名常量，旧缓存交回旧词种，按名字连的边丢掉、键变窄，下一处改动键不变而结局变。推的 |
| ①-2 写 `__doc__` 不算读 | **今天的代码上推不出；构造得出两步反例** | 认读文档串只认 `__doc__` 这个词，`getattr(x, "__doc__")`、`inspect.getdoc` 认不出；第 1469 行那条写今天顺带把整份算键库的文档串带进闭包，罩住了这类读法，改 2 拿掉了这层罩子。病根在认读法，不在改 2 |
| ①-3 自证夹具与格名表搬出 | **推不出反例**（持久判定） | 名字带 SELFTEST 的模块级常量里，被名字不带 selftest 的运行期函数读到的只在广谱变异（3 个，都经 `test_mode` 门控）；crates 变异那 8 处读者都是自证用的造样本函数；开关表 `PRUNE_BREAKS` 被运行期 `prune_rules` 读，但弄坏开关只在 `test_mode` 下生效 |
| ①-4 判法指纹去掉被判树的数据文件 | **推不出反例**（三样各自在 unit-graph 缓存键那一处读点） | unit-graph 程序只改键的算法（产品行为视图的剥法、拒算），不改 cargo 跑出的判档；条件：别的读点上的同一份文件不能按路径一起排掉 |
| ①-5 kept-comments 只收登记词 | **构造得出（条件式）** | `#!` 行不归正则管、由 `kept_shell_comments` 另收；收窄时若连它一起收掉，按路径直接起的脚本改解释器键不变。另列程序读而不在示例表里的词 |
| ①-6 记账代码出判法闭包 | **构造得出** | 「写记录」那一段决定同一把行键下库里留哪一档、复用读的就是它；改写入条件（例：不是抓到的不写），重判出「没红」盖不掉旧「抓到」，下一趟照复用，键不变 |
| ①-7 编译档三项移出键 | **推不出确定性反例**；可观测的两处在不进键的那一类 | 见下面三格 |
| ①-8 登记文件只把数编进二进制 | **变体二构造得出（条件式）；变体一推不出键的反例** | 仓里两份 Rust 读法对「第一条非注释行」不一致（`split('\n')` 与 `.lines()`）；数据单元规则跟哪一份都漏另一份。插一行只有 `\r` 的行：一份 panic、一份照读 31 |
| ②-`debug` | **不改结局**（推的） | panic 位置来自编译期的 `Location`，不靠调试信息；`crates/` 下没有一处碰回溯（grep 0 命中，量的） |
| ②-`incremental` | **只经 codegen 分区与编译器缺陷改结局**（推的，没量） | 没写 `codegen-units` 时它把默认分区数从 16 改成 256（rustc 的默认，未在本仓核），效果并进下一格 |
| ②-`codegen-units` | **能改可观测行为：栈深与挂钟**（推的，没量） | 本仓有一处按挂钟比的断言（GPU 借据那一条），有一处显式加大线程栈；都属于「资源上限、限时」不进键那一类，不是确定性反例 |
| ③ 收错方向 | **有：第 6 条的写记录那一段**；另三处是实现时的陷阱 | 第 7 条 python 一侧按前缀滤 `debug` 会连 `debug-assertions` 一起滤；第 5 条收掉 `#!` 行；第 1 条名里只放函数单元 |

## 攻击面①：八条改法逐条找反例

D13（验证路线） 已定项 17 里这一轮压着的几句（kb 文件自己的行号，现查）：`.claude/kb/decisions/13-验证路线.md:432`：「键过窄（引用的常量、类型、宏漏收）」；`.claude/kb/decisions/13-验证路线.md:433`：「收不全（动态分派、宏、」；`.claude/kb/decisions/13-验证路线.md:434`：「三种排除不进键，理由不同、分开记」；`.claude/kb/decisions/13-验证路线.md:436`：「红的也照键记」。

### 第 1 条：算键库不读自己整份文件

今天：`scripts/gate/lib_function_units.py:2032`：「with open(os.path.realpath(__file__), "rb") as handle:」——缓存目录名取整份库的 sha256。

这份缓存管什么：载入每份 `.rs` 走 `rust_lex_cached`（`scripts/gate/lib_function_units.py:2532`：「self.kinds, self.texts, self.lines, self.documents = rust_lex_cached(text)」），切出的词法单元是全部 Rust 单元、闭包与键的底料；crates 变异的行键（`UnitKeys.workspace`）、按条复用的测试函数、崩溃块都从它来。crates 变异的拷贝范围不走缓存：`scripts/hosts/lib_repository_copy.py:481`：「kinds, texts, lines, _documents = units.rust_lex(text)」。所以缓存只改键，不改 cargo 跑出的判档。

反例（条件式：改法照字面「rust_lex 与它调到的帮手的单元指纹」做，名里只有这几个 def 的指纹）：

1. 缓存里存的是 `rust_lex` 的输出，输出不只由 `rust_lex` 与帮手的函数体决定。还决定它的：
   - 模块级常量词种名，`scripts/gate/lib_function_units.py:1628`：「IDENT, LITERAL, LIFETIME, PUNCT = "ident", "literal", "lifetime", "punct"」。函数单元指纹是 `ast.dump`，里面只有名字 `IDENT`，没有它的值。缓存文件存的是词种名的字面串（`store_rust_lex` 存 `kind_names`）。
   - 缓存的编解码 `store_rust_lex` / `load_rust_lex` 与 `scripts/gate/lib_function_units.py:1806`：「RUST_LEX_CACHE_FORM = "singlefs-rust-lex-cache-v1"」：`rust_lex` 不调它们，不在「rust_lex 与它调到的帮手」里。
   - 解释器的 Unicode 表（`isspace`、`isdigit`、`isalnum`）：今天整份摘要也不含它，不归改 1，只记一笔。
2. 改动甲：把 `IDENT` 的值从 `"ident"` 改成别的串（同时这份库里 105 处 `== IDENT`/`!= PUNCT` 这类比较照新值比，`grep -c` 量的数）。函数单元指纹不变 → 缓存目录名不变 → 每份没改过的 `.rs` 取出旧缓存，词种是旧串 `"ident"`，与新的 `IDENT` 不等 → 标识符一个都认不出 → 按名字连的边（R-裸名、路径）全丢，闭包缩到只剩结构边。
   - 这一步本身的键会变（闭包成员变了），所以这一步重判一次，不是反例本身。
3. 改动乙：改一个只经名字边够得到的被调函数的函数体（例：某条测试经 `crate::foo()` 调到的 `foo`）。在缓存还在的那台（缓存放 git common-dir 底下，两台共用同一份共享根，推的）：闭包里没有 `foo`，键不变；`foo` 改了，测试结局变。这就是「改了判定结局而键不变」，落在第 432 行「键过窄」那一句。
   - 换一台没缓存、或缓存被挤掉（只留最近 3 个目录）的，算出的键含 `foo`、会变：同一棵树两种键，结局取决于本机缓存状态。
4. 今天为什么不中：整份摘要随第 1628 行变，改动甲那一刻旧目录整个不再取。

四句：分辨臂——分辨（只打「名里只放函数单元」那一种做法；名里放「`rust_lex`、`load_rust_lex`、`store_rust_lex` 三个入口的整个闭包键（含模块级语句）加解释器版本」的做法不中）；系统看不看得到——看得到（常量就在同一份文件里）；满足的分句——第 432 行「键过窄（引用的常量、类型、宏漏收）」；改法在打中的格上——见第三节改法表。

### 第 2 条：写 `__doc__` 不算读

机制（按代码读，推的）：第 1469 行 `InlineScript.__doc__ = """…"""` 的属性节点进 `doc_reads`，主人记成 `InlineScript`；`InlineScript` 是 `scripts/gate/lib_function_units.py:1468`：「InlineScript = collections.namedtuple(」，不是带文档串的 class，不在 `module.docstrings` 里，于是走 `scripts/gate/lib_function_units.py:14531`：「module.docstring_units("*")」——整份库的文档串全进。背景材料「推的」那一句（文档串整批进闭包是不是只因第 1469 行）在代码上成立；只因这一条、还是另有别的读者，要主 agent 量（改 2 之后闭包里文档串单元数是不是 0）。

今天判法闭包里真读文档串的：`grep -n '__doc__\|getdoc\|"__doc__"\|pydoc\|get_docstring'` 在 `scripts/gate/lib_function_units.py`、`scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/lib_mutation_units.py`、`scripts/gate/lib_reuse_checks.py` 四份里，读的只有第 1469 行那条写、文件头说明与第 15840 行（那一行在夹具文本里），量的。所以今天的代码上推不出反例。

两步反例（构造的，推的）：
1. 改动甲：在判法闭包里某个会改判档的函数里加一句 `text = inspect.getdoc(some_function)` 或 `getattr(some_function, "__doc__")`，判档按 `text` 走。认读法：`getattr(x, <字面串>)` 进的是属性表、不进 `doc_reads`（`scripts/gate/lib_function_units.py:9387`：「facts.attributes.append((dotted_name(node.args[0]), attribute.value))」）；`inspect.getdoc` 只有 `inspect.getsource`/`getsourcelines` 两个名字被认（`SOURCE_READERS`），它不在。这一步改了函数体，键变，重判一次。
2. 改动乙：只改 `some_function` 的文档串。函数指纹去掉文档串；文档串单元没人「读」→ 不在闭包 → 键不变；判档变了。
3. 今天为什么不中：`some_function` 若在算键库里，第 1469 行那条写把整份库的文档串都带进来，改动乙改了一个文档串单元，键变。改 2 拿掉这层罩子；`crates-mutation-rows.py` 里的函数今天就没罩（它没有 `__doc__` 写），所以这是认读法的老缺口，改 2 只把它扩到算键库。

四句：分辨臂——不分辨（同一个缺口打中改 2 前后的 `crates-mutation-rows.py`，病根在共用的认读法）；看不看得到——看得到（`getattr` 的第二个参数是字面串）；分句——第 433 行「收不全（动态分派、宏、」那一类应拒算而没拒；改法——改 2 照做不修这一格，要另补「`getattr(…, "__doc__")`、`inspect.getdoc`、`pydoc`、`help` 记读文档串或拒算」。

### 第 3 条：自证夹具与格名表搬出

扫描（量的，`scan/selftest-users.txt`）：名字以 SELFTEST 起头、被名字不带 selftest 的地方读到的——

- `scripts/mutation/crates-mutation-rows.py`：8 处，读者是 `build_seed_repository`、`build_coverage_repository`（自证造样本仓的函数，只被 `--selftest` 那一路调）与别的 `SELFTEST_*` 模块级定义。
- `scripts/mutation/broad-mutation.py`：`SELFTEST_WORK_VARIABLE`（读者 `Runner`、`check_test_only_variables`）、`SELFTEST_CELLS_VARIABLE`（`main`）、`SELFTEST_STEP_DRIVER_NAME`（`Runner`）三个是运行期读的，外加三处模块级 `SELFTEST_*` 互引。
- 算键库与 `lib_mutation_units.py`：各 1 个与 0 个，没有运行期读者。

运行期那几处都经 `test_mode` 门控：`scripts/mutation/broad-mutation.py:685`：「return os.environ.get(name) if test_mode() else None」，`break_is` 同样先判 `test_mode()`；真跑时这几个变量设了也读不到，判档不受它们左右。开关表 `PRUNE_BREAKS` 被运行期 `prune_rules` 读：`scripts/mutation/broad-mutation.py:1378`：「PRUNE_MODULE.BREAKS = frozenset(name for name in PRUNE_BREAKS if break_is(name))」——真跑时 `break_is` 一律假，交空集。

判定：持久判定（crates 变异每行、广谱变异每条）推不出反例。条件两条：搬走的只许是上面扫描里没有运行期读者的那些；`PRUNE_BREAKS` 这类开关表若搬走，`prune_rules` 不能再在模块级以外读它（否则搬了也要经 import 拉回闭包，等于没搬）。查过的路径：`test_variable`、`break_is`、`check_test_only_variables_before_parsing`、`prune_rules`、`judging_units_key`（广谱变异）；`cargo_debug_overrides`、`break_is`（crates 变异，开关名在用处写成字面串，没有开关表常量）。

### 第 4 条：判法指纹里去掉被判树的数据文件

三样今天的读点：`unit-graph` 程序的跨轮缓存摘要（`scripts/gate/lib_function_units.py` 文件头「unit-graph 从哪来」那一段，第 457–460 行）。`unit-graph` 程序在 crates 变异里只用在产品行为视图：剥 Verus 规格与证明、换指纹、记 `ghost_only`、配不上记 `product-view-unmatched` 拒算（文件头第 435–455 行）。它改的是键怎么算与拒不拒算；拒算的行照跑，判档来自 cargo test。

逐样查（推的，按代码读）：
- `crates/singlefs-unit-graph/src` 改了：剥得多了 → 被剥单元出闭包 → 含它的行键这一步就变（重判一次）；之后改被剥掉的那段执行代码键不变——但这一步（改动乙）把 `unit-graph/src` 放进判法指纹也挡不住（指纹在改动甲那一刻变过一次之后就稳定了）。所以判法指纹里带不带它，对「结局变而键不变」没有保护作用。剥得少了 → 键变，保守。
- `Cargo.lock` 在这一处只决定编 `unit-graph` 用的 `verus_syn`、`proc-macro2` 这些版本，同上，只改键的算法。被测代码的依赖版本照旧在行键自己那一栏，规则带传递依赖：`scripts/mutation/lib_mutation_units.py:53`：「再顺 dependencies 收传递依赖」。
- `.claude/heavy-thread-default` 在这一处只决定编 `unit-graph` 时的线程数，不改它的输出（两份实现对拍时逐字相同，文件头第 9–11 行）。

判定：推不出反例。条件（实现时的陷阱，归攻击面③）：
- 改法只拿掉「编 unit-graph 算缓存键」这一个读点。同一份文件在判法闭包里别的读点照收：`scripts/mutation/lib_mutation_units.py:766` 起的 `read_lock_packages`（读根 `Cargo.lock` 出依赖行）、`scripts/mutation/lib_mutation_units.py:751` 起读根 `Cargo.toml` 的 `[profile]`、`[lints]`，`scripts/hosts/heavy_threads.py` 取并行度。按路径整份排除会连这几处一起拿掉。
- 背景材料说 16 个数据文件里还有「仓根与各 crate 的 Cargo.toml」。改 4 没点它们，我也没查它们各从哪个读点进来。要是实现时连它们一起当「被判树的数据文件」排掉：工作区清单里外部依赖开的特性（例 `crates/singlefs-background-gpu/Cargo.toml:17` 的 wgpu 特性表）不在行键里（行键的特性只收工作区包自己的，`scripts/mutation/lib_mutation_units.py:507` 起 `features_of`；锁文件的行不带特性），删一个不增减可选依赖包的特性，`Cargo.lock` 不变、行键不变，外部 crate 编法变、结局可能变。今天它们整份在判法指纹里，碰巧罩着。这一句是推的，没构造到具体会翻的测试。

### 第 5 条：`kept-comments` 只收登记的指令词

今天的收法有两件：正则（第 10539 行，任何「# 英文词:」与 `# shellcheck`）与另一件——`#!` 行：`scripts/gate/lib_function_units.py:10553`：「kept = [rows[0]] if rows and rows[0].startswith("#!") else []」。闭包碰到这份文件的任何单元就带上整批：`scripts/gate/lib_function_units.py:14934`：「self.closure.add(shell_file.kept, unit, "kept-comments")」。

反例（条件式：收窄时把 `kept_shell_comments` 整个换成「只收登记词」，`#!` 行跟着没了）：
- 改动：把一份按路径直接起的脚本第一行从 `#!/usr/bin/env bash` 改成 `#!/bin/sh`。`declare -f` 的规范文本不含注释，`#!` 行不在任何函数或脚本体单元里 → 键不变；直接执行时换了解释器，bash 专有写法（`[[`、数组、`mapfile`）在 dash 下出错 → 结局变。
- 要被直接起：仓里有经路径直接执行的程序（`task-server/task` 在自证夹具里按 `[os.path.join(repo_root, "task-server", "task"), …]` 起，`scripts/gate/lib_function_units.py:21374`）；门禁阶段由 `gate.sh` 经 `bash` 起，不受影响。哪一份 shell 脚本既按路径直接起、又在某份判定的闭包里，我没逐个列（推的）。
- 照改法字面「`KEPT_SHELL_COMMENT` 从任何英文词加冒号收到一张登记的词表」只改正则，`#!` 那一行不动，不中。

程序读、而不在改法示例表（admission、run-condition、selftest-serial、task-server-*、gate-cell、gate-covers、shellcheck）里的词（漏登会怎样归问题 4，这里只交我查到的线索）：
- `# gate-stage:`：`.claude/singlefs-ai-sop/scripts/gate.sh:670`：「s/^# gate-stage:[[:space:]]*//p」，`gate.sh` 起阶段前从被起的那份脚本里读阶段名；还有 `.claude/singlefs-ai-sop/scripts/gate-overlap.py:310`。阶段名进标签、跳过名单与汇总，我没找到它改哪一格判档。
- 仓里 shell 文件（`git ls-files '*.sh'`，去掉门禁样本目录）里出现的「# 词:」共 48 种（量的，`scan/kept-words.txt`），频次最高的是 `gate-lint:`（897）、`gate-cell:`（196）、`gate-similar:`（163）、`run-condition:`（94）、`admission:`（84），另有 `account-quota:`、`long-run-shards:`、`long-run-hosts:`、`long-run-resume:`、`hook-events:`、`shell-lint:`、`clock-times:`、`change-range:`、`rocksdb-prebuilt-entry:`。其中有程序按字面读的我只核了 `gate-stage`、`gate-covers`（`gate.sh` 同一段）、`rocksdb-prebuilt-entry`（`scripts/hosts/rocksdb-prebuilt.py` 读假 cargo 的头 4096 字节）、`long-run-*`（我没找到运行期读者，只有装置脚本的注释引它）。

四句（`#!` 那一格）：分辨臂——分辨（只打「连 `#!` 一起收」的实现）；看不看得到——看得到（第一行就在文件里）；分句——第 432 行「键过窄」；改法——「`#!` 行照收」修它。

### 第 6 条：记账代码出判法闭包

复用读什么：`scripts/mutation/crates-mutation-rows.py:3000`：「if record.get("verdict") == REUSABLE_VERDICT["mutation"]:」——只认 `records/<行键>` 里那一条的 `verdict` 字段。写它的就是改法要搬出的「写记录」：

- `scripts/mutation/crates-mutation-rows.py:5402`：「keeps_counted = bool(components.get("row_key")) and verdict in VERDICTS_UNCOUNTED」，不算数的结局（`scripts/mutation/crates-mutation-rows.py:1272`：「VERDICTS_UNCOUNTED = ("notrun",)」）不盖原有的算数结论；
- `scripts/mutation/crates-mutation-rows.py:5404` 决定写不写，弄坏开关就是另一种写法：「not break_is(context["environment"], "red-not-recorded")」；
- `scripts/mutation/crates-mutation-rows.py:5406` 写哪一档：「verdict=verdict, text=text,」；基线那一批另在 `scripts/mutation/crates-mutation-rows.py:5358` 写。

反例（构造的，推的）：
1. 外壳经不进闭包的引用调之后，改外壳的写入条件，让「不是抓到的」不写（就是 `red-not-recorded` 的行为当常态）。外壳不在判法指纹里 → 判法指纹不变 → 每行的行键不变。
2. 同一把行键下要出现「抓到」之后又判出「没红」：照这一道自己的文件头，会翻的有 GPU 行（`verdict_of_run` 那一段：显存不够而红的照记抓到，空闲时同一条测试不红）、复用哨兵强制现判（`plan_reuse_sentinel`）、基线不绿那几行。今天：重判出「没红」盖掉旧的「抓到」，下一趟现判。改了外壳之后：盖不掉，下一趟照复用「抓到」。
3. 库里这把键下的结局从「没红」变成「抓到」，键不变。落在第 436 行「红的也照键记」那一条：它要的「撤掉同一把键已有的绿」正是外壳这一段做的事。

不改结局的另外三件（推的，查过）：用时表（`timings/<六段原样的 sha256>`）只排派活次序；量占盘那几个数进记录，复用时读不出才改现判（`scripts/mutation/crates-mutation-rows.py:7176-7180`），只多判不少判；说明文字复用时现拼（`caught_text(row, "（复用）")`），存的那份不回读。外壳里删每格 TMPDIR 的那一步删没删成进「跑完有 N 个没删掉」整道判红，这一格是整道的，不是行的判定。

四句：分辨臂——分辨（只打「写记录」搬出去的那一半；用时表、量占盘、说明文字搬出去不中）；看不看得到——看得到（写入条件就在外壳代码里）；分句——第 436 行；改法——把「写哪一档、写不写、盖不盖」留在核心里（与判档同一边），外壳只拿到写好的字段去落库。

### 第 7 条：编译档里不改运行结果的设置移出键

确定性的反例推不出（三项各自的推理在攻击面②）。实现上的两处现状（量的，读代码）：
- 进键的 profile 设置有三条路：Rust 一侧 `crates/singlefs-unit-graph/src/build_toolchain.rs:17`：「pub const PROFILE_SETTING_KEYS: [&str; 8] = [」；python 一侧的清单摊开（`scripts/gate/admission.py:1569` 起 `profile_lines`、`scripts/mutation/lib_mutation_units.py:758` 起）；python 一侧的环境变量整族：`scripts/mutation/lib_mutation_units.py:737`：「if name in BUILD_VARIABLES or name.startswith("CARGO_PROFILE_") or target_variable:」。改 7 只点了前两条；crates 变异起 cargo 时自己设的 `CARGO_PROFILE_<档>_DEBUG=false`（`scripts/mutation/crates-mutation-rows.py:952`：「顶掉调用方设的同名变量与仓根 Cargo.toml 的 [profile.release] debug = true」）走第三条，照旧进键。这只让键比改法说的宽，不是反例。
- 「门禁起 cargo 时开 release 增量」要是用 `CARGO_INCREMENTAL=1` 开，它今天就不进任何一边的键（不在 `BUILD_VARIABLES`，也不是 `CARGO_PROFILE_` 起头）；用 `CARGO_PROFILE_RELEASE_INCREMENTAL` 开，python 一侧照第三条进键。

### 第 8 条：重活线程数的登记文件只把数编进二进制

仓里有两份 Rust 读法，各自 `include_str!` 同一份文件（`crates/singlefs-verification/src/heavy_threads.rs:25` 与 `crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts_write_dump/mod.rs:47`），对「第一条非注释行」的认法不同：
- `crates/singlefs-verification/src/heavy_threads.rs:36`：「let mut lines: Vec<&str> = registration_text.split('\n').collect();」——只按 `\n` 切，行尾的 `\r` 留在行里；只去行首空格与制表符；那一行还要带理由：`crates/singlefs-verification/src/heavy_threads.rs:72`：「let reason = after_the_blanks.strip_prefix('#')?;」，不合就 panic。
- `crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts_write_dump/mod.rs:95`：「.lines()」——Rust 的 `str::lines` 去掉行尾一个 `\r`；不要理由，取行首数字。

变体二（数据单元只取第一条非注释行）的反例（条件式，推的，探针待跑）：
1. 数据单元规则照 e184 那一份（或 python 的 `splitlines`、`strip`）写。
2. 改动：在 `31  # …` 之前插一行只有 `\r` 的行（例：有人用 CRLF 的编辑器加了一个空行）。
3. 数据单元：`\r` 那一行算空行跳过，第一条非注释行仍是 `31  # …` → 键不变。
4. `heavy_threads_registered_in`：`\r` 那一行去掉行首空白后不空、不以 `#` 起头 → 当成第一条非注释行 → 不是「数 + 空白 + # + 理由」→ panic。验证包里经 `default_heavy_threads` 取线程数的测试与装置全红；e184 那一份照读 31。
5. 规则改照 `heavy_threads.rs` 那一份写：插 `\r` 行时键变（保守）。反过来让 e184 变而 `heavy_threads.rs` 不变的改动，我只找到第一条非注释行本身变的那种（键照变），推不出反例。

变体一（文件只留数、说明挪走）：整份文件仍进键，键这一头推不出反例；但第 72 行那条「要理由」会让只留 `31` 的文件在 `heavy_threads_registered_in` 里 panic，而 e184 照读——改 8 变体一要连 `heavy_threads.rs`、`scripts/hosts/heavy_threads.py`、`scripts/hosts/heavy-threads.sh` 三份读法一起改（探针的第三格 `number-without-reason` 量这一句）。

四句（变体二）：分辨臂——分辨（只打「数据单元规则与 `heavy_threads.rs` 不同」的实现）；看不看得到——看得到；分句——第 432 行「键过窄」；改法——数据单元规则逐字照 `heavy_threads_registered_in` 的切法（只按 `\n` 切、只去空格与制表符），或者把 e184 那一份改成调 `singlefs_verification::heavy_threads`、仓里只留一种读法；或者干脆走变体一。

## 攻击面②：第 7 条三项会不会改测试的可观测行为

grep 命令与命中（量的，在仓根跑，`crates/` 下全部 `.rs`）：

```
$ grep -rn -e 'Backtrace' -e 'backtrace' -e 'RUST_BACKTRACE' crates --include=*.rs ; echo rc=$?
rc=1
$ grep -rn -e 'Location::caller' -e 'panic::Location' -e '\.location()' crates --include=*.rs ; echo rc=$?
crates/singlefs-verification/src/crash_library.rs:890:        site: &'static std::panic::Location<'static>,
crates/singlefs-verification/src/crash_library.rs:957:    site: &'static std::panic::Location<'static>,
crates/singlefs-verification/src/crash_library.rs:1172:        let site = std::panic::Location::caller();
crates/singlefs-verification/src/history.rs:3624:                let location = panic_details.location().map_or_else(
rc=0
$ grep -rn 'should_panic(expected = "[^"]*\.rs' crates --include=*.rs ; echo rc=$?
rc=1
```

（`#[track_caller]` 另有 `crates/singlefs-verification/src/crash_state_enumeration.rs` 的 8 处，都是断言帮手报调用处，不比文本。）

断言 panic 位置的只有一处，按文件加消息片段比、不按行号：`crates/singlefs-verification/src/bad_disk_input.rs:948`：「captured.location.contains(site.file) && captured.message.contains(site.message_fragment)」；位置串来自 `crates/singlefs-verification/src/history.rs:3626`：「format!("{}:{}", location.file(), location.line())」。`crash_library.rs` 那三处记持锁时长的调用处，不进判档。

| 项 | 能不能改测试结局 | 依据与什么会推翻它 |
|---|---|---|
| `debug` | 不能（推的） | panic 位置、`#[track_caller]` 的文件与行是编译期常量，不靠 DWARF；回溯文本没人断言（上面 rc=1）；依赖的构建脚本读 `DEBUG` 的只有 `cc`，它只加 `-g` 与帧指针（`~/.cargo/registry/src/*/cc-1.5.1/src/lib.rs` 第 2445、4342 行，读过），rocksdb 那一份无条件定义 `NDEBUG`（`librocksdb-sys-0.19.0+11.8.1/build.rs` 第 148、399 行）。推翻它：找到一条测试或构建脚本按 `DEBUG` 或调试信息分支 |
| `incremental` | 不经语义改（推的，没量） | 没写 `codegen-units` 时开增量把 rustc 的默认分区数从 16 改成 256（rustc 的默认值，我没在本仓核，未核实），效果并进下一行；增量缓存在每个工作进程自己的编译目录里，是本机状态、不进键，只经编译器缺陷改结局（外部线索，未在本项目验证）。推翻它：同一把键、开关增量两趟判档不同 |
| `codegen-units` | 能改挂钟与栈深（推的，没量） | 分区数改内联，改主机侧代码的快慢与栈帧大小。本仓有按挂钟比的断言：`crates/singlefs-verification/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs:225`：「submitted_to_the_lease_dropped.as_micros() * 100」（提交到丢借据的主机侧用时要小于 GPU 空转的一个比例）；有显式加大线程栈的：`crates/singlefs-verification/tests/allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf.rs:39`（64 MiB）。这两样属于 D13（验证路线） 已定项 17「资源上限、限时」不进键那一类，结局本来就随机器与负载变，不算确定性反例。浮点：Rust 不自动融合乘加，分区数不改浮点结果（推的）。推翻它：同一棵树、只改 `codegen-units`，某条不依赖挂钟与栈的测试判档变 |

## 攻击面③：八条里哪一条收错了方向

| 改法 | 收错没有 | 收掉的、会改结局的东西（文件与行） |
|---|---|---|
| 第 6 条 | **收错了**（照改法字面就中） | 「写记录」不是记账，是判定的一半：复用读 `scripts/mutation/crates-mutation-rows.py:3000` 那一句，写它的是第 5402–5412 行与基线那一批的第 5358 行；改法把它与用时表、量占盘、说明文字归成一类搬出判法闭包 |
| 第 1 条 | 条件式 | 改法字面「rust_lex 与它调到的帮手的单元指纹」把 `scripts/gate/lib_function_units.py:1628` 的词种名常量、第 1806 行的缓存格式号与 `store_rust_lex`/`load_rust_lex` 留在名外；它们决定缓存里存的、取出的是什么 |
| 第 5 条 | 条件式 | 收窄若换掉整个 `kept_shell_comments`，`scripts/gate/lib_function_units.py:10553` 收的 `#!` 行跟着没了 |
| 第 7 条 | 条件式（实现陷阱） | python 一侧摊开成一行一项（写法见 `scripts/mutation/lib_mutation_units.py:697` 与 `scripts/gate/admission.py:1167`），摊出的行形如 `profile.release.debug=…` 与 `profile.release.debug-assertions=…`，按前缀 `profile.<名>.debug` 滤会连 `debug-assertions` 一起滤掉，那一项改结局。Rust 一侧按键名精确比，没有这个问题 |
| 第 8 条 变体二 | 条件式 | 数据单元规则与 `heavy_threads.rs` 的切法不同时，收掉了「`\r` 行算不算第一条非注释行」这一件 |
| 第 2、3、4 条 | 推不出 | 查过的范围：第 2 条——四份判法文件里全部读文档串的写法（见上）；第 3 条——ast 扫描的四份文件里全部 SELFTEST 名字的读者与 `PRUNE_BREAKS`、`TEST_ONLY_VARIABLES` 的读者；第 4 条——`unit-graph` 程序在 crates 变异里的用处（产品行为视图；`static-check` 可达性经同一份视图算闭包，但截过的闭包不算键、不进判定库，第 432 行那一节），拷贝范围（`scripts/hosts/lib_repository_copy.py`）只用 `rust_lex` 与 `rust_string_value` |

## 改法（只在我的模型上量过、被攻过零轮——这一条腿没有模型，下面全是推的）

| 改法 | 修哪一格 | 依据 |
|---|---|---|
| 甲：第 1 条的缓存目录名取「`rust_lex`、`load_rust_lex`、`store_rust_lex` 三个入口的闭包键（照这份库自己的 python 闭包算，含模块级语句）加解释器版本加弄坏开关」，不取单个 def 的指纹 | ①-1 | 推的，待主 agent 跑；闭包带模块级语句会把第 1628、1806 行收进来，但也把这份库全部模块级赋值收进来——收窄的幅度要量（今天判法闭包里模块级赋值 948 个，背景材料第三节） |
| 乙：认读文档串加三种写法：`getattr(<x>, "__doc__")`、`inspect.getdoc(<x>)`、`pydoc`/`help` 一族；认不出主人的照「*」收或拒算 | ①-2 | 推的；与改 2 正交，改 2 照做 |
| 丙：第 6 条只搬用时表、量占盘、说明文字；「写哪一档、写不写、盖不盖」（第 5402–5412 行与第 5358 行那一段，连同 `uncounted_verdict_keeps_counted_record`）留在判法闭包里 | ①-6、③ | 推的 |
| 丁：第 5 条只换正则，`#!` 行照收；自证加一格「改 `#!` 行键变」 | ①-5 | 推的 |
| 戊：第 7 条 python 一侧按键名精确去掉三项（摊开之前在 dict 上删键），自证加一格「改 `debug-assertions` 键变、改 `debug` 键不变」 | ③ | 推的 |
| 己：第 8 条走变体一，三份读法一起改成不要理由；或走变体二时数据单元规则逐字照 `heavy_threads_registered_in`，并把 e184 那一份读法换成调验证包的函数 | ①-8 | 推的，探针待主 agent 跑 |

## 要主 agent 跑的命令

1. 第 8 条的探针（可选：上面的判定按代码读就成立，探针把它变成量的）。不是仓里的代码，是草稿目录里一个独立的小包（`[workspace]` 自成一个工作区，不依赖仓里任何包）。
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gate-key-closure-r1-opus/heavy-thread-probe -- cargo run --offline`
   - 在哪个目录起：`/home/fy5090/.local/share/singlefs/work/gate-key-closure-r1-opus/heavy-thread-probe`（已在认领目录的 `.deliverables` 写成「交接」）
   - 看哪几行：`TASK_SERVER_RESULT` 那一行 `exit=0`；`task-server/task log <号>` 里三行 `case=`。预言（跑前写下）：`case=today` 两份都是 31；`case=carriage-return-blank-line-before-the-number` 的 `unit_line` 是 `Some("31  # 理由")`、`heavy_threads_rs=panic`、`e184=31`；`case=number-without-reason` 的 `heavy_threads_rs=panic`、`e184=31`。第二行若 `heavy_threads_rs=31`，变体二那一格反例撤回。
   - 被服务的范围声明闸拒（它把点名 cargo 的命令当验证命令）时，照拒绝说明补 `SINGLEFS_RUN_SCOPE`，种类写 `target`，范围写「草稿目录里一个探针包」。

2. 判决要用的两个数（这一条腿没量）：改 2 之后 crates 变异判法闭包里文档串单元剩几个（预言 0，见第 2 条）；改 1 照改法甲之后判法闭包里还剩不剩 `source-text`、`data-file scripts/gate/lib_function_units.py` 两行。两样都是主 agent 交过的任务 16712 那一条命令在改过的副本上再跑一次，我不知道那条命令的原文，没写成整条。

## 没打中的形状

| 试过的形状 | 对哪一条 | 取样范围 | 为什么没中 |
|---|---|---|---|
| 缓存编解码改了而格式号没升，旧文件被新读法读歪 | 1 | `store_rust_lex` / `load_rust_lex` 两个函数 | 读法核三张表等长、文档项五元组，改编码的大多数写法（换数组宽度、换次序）被这道核挡下、照常现切；只剩「等长而语义变」那一种，我没造出会改键的具体写法 |
| 拷贝范围经缓存词法改判档 | 1 | `scripts/hosts/lib_repository_copy.py`、crates 变异驱动里调 `rust_lex` 的三处 | 都调不带缓存的 `rust_lex` |
| 文档串被 `__doc__` 之外的写法读，今天就在判法闭包里 | 2 | 四份判法文件全文 grep | 0 处 |
| 名字带 SELFTEST 的常量在真跑时改判档 | 3 | 四份文件的 ast 扫描（共 76 个 SELFTEST 名字） | 运行期读者都经 `test_mode` 门控 |
| 只给自证用的变量名单（`TEST_ONLY_VARIABLES`）搬走后真跑放进假一步 | 3 | 广谱变异 `check_test_only_variables*`、`test_variable` | `test_variable` 自己先判 `test_mode`，名单只是第二道 |
| `unit-graph` 改动直接改判档 | 4 | `unit-graph` 在 crates 变异里的用处 | 只改键的算法与拒算，拒算照跑 |
| 依赖版本：锁文件改了而行键没变 | 4 | `dependency_lines` | 收传递依赖；落不到包的拒算 |
| 第三方构建脚本按 `DEBUG` 改语义 | 7 | 注册表里本仓用到的带 C 代码的包：`cc`、`librocksdb-sys`、`zstd-sys`、`lz4-sys`、`bzip2-sys`、`libz-sys` 的构建脚本 grep `DEBUG`/`NDEBUG` | `cc` 只加 `-g` 与帧指针；rocksdb 无条件 `NDEBUG` |
| 测试断言回溯文本、按行号断言 panic 位置、`should_panic` 期望串带文件路径 | 7 | `crates/` 全部 `.rs` | 0 处；唯一按位置认的按文件加消息片段 |
| 让 e184 那一份读法变而 `heavy_threads.rs` 不变 | 8 | 两份读法逐行对 | 能让 e184 变的都先改了第一条非注释行本身 |

## 这条腿自己的限度

- 没有一个数是在装置上跑出来的；「量的」只有 grep 与 ast 扫描的计数。全部反例是按代码读出来的构造，主 agent 要现查。
- 第 1、5、7、8 条的反例都是条件式：打的是改法的某一种实现，不是改法字面的每一种实现；第 6 条照字面就中。
- 第 4 条只查了改法点名的三样在「编 unit-graph」那一个读点上的作用；背景材料里另外那些 Cargo.toml 数据单元各从哪个读点进来，我没查。
- 第 5 条程序读的词只核了五种；48 种里其余的有没有运行期读者没逐个核。
- 第 7 条关于 rustc 默认分区数（开增量时 256、不开 16）与「Rust 不自动融合乘加」是我对外部工具的印象，没在本仓核；`codegen-units` 改栈深、挂钟能不能真让本仓某条测试翻，没量。
- 外部实现（rustc、cargo、`cc`）的行为在这里只当线索，没有一条当证据用。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没有副本上的数。
- 没往统一任务服务交任务；探针没编没跑（命令在「要主 agent 跑的命令」）；判决要引的两个闭包计数没量。
- 没建仓副本、没编过东西：草稿目录里只有 `scan/`（扫描脚本与输出）与 `heavy-thread-probe/`（探针源码，交接），没有编译目录与仓副本要删。
- 没回答问题 2、4、5 与问题 6 的前半（不在分给这条腿的攻击面里）。
