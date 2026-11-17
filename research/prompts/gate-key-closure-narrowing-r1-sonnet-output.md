# gate-key-closure-narrowing-r1 正推腿报告（Sonnet）

立场：从今天 `crates/` 与 `scripts/` 的实现推「收窄之后留在键里的东西够不够定结局」，与八条改法逐格比。行号都是我这一趟在文件里现查的；kb 条款写 `13-验证路线.md` 加那份文件自己的行号。没写模型；读了任务 16712 的日志（`task-server/task log 16712 --all`，落在草稿目录 `/home/fy5090/.local/share/singlefs/work/gate-key-closure-r1-sonnet/task16712.txt`）数单元，没有自己跑过任何项目代码，凡是推的都标「推」。

## 速览

- 八条里没有一条构造得出「改了结局而键不变」的反例的，只有改法 7 的 `codegen-units`、改法 8 的「只留数」这两处写法站不住，其余各条都带一个必须补进写法的条件（下面逐条）。
- 改法 1 的缓存目录名只罩 `rust_lex` 与帮手不够：缓存读写那几个函数（`load_rust_lex`、`store_rust_lex`）改了，旧缓存照取，推得出一个反例。
- 改法 4 点名的三样里，`.claude/heavy-thread-default` 还有第二条进闭包的路（python 的 `default_heavy_threads` 经 `run_cargo_command`），只改算键库那条路它不会离开判法指纹。
- 改法 5 的「登记词表」极性是错的：漏登 = 少收，不安全；改成「登记确认没人读的词」，漏登只是多收。
- 问题 6：八条之外还有同类的宽，主要三处，见问题 6 一节。

## 问题 1：八条逐条正推

记号：留 = 留在键里；出 = 被收出去；到 = 被收出去的东西经哪条路径到判定结局。

### 改法 1（算键库不读自己整份文件）

- 实现今天：`scripts/gate/lib_function_units.py:2032` `with open(os.path.realpath(__file__), "rb") as handle:` 把整份读进 sha256，当 `rust_lex_cache_namespace`（2028）的目录名。闭包里的 `source-text` 与 `data-file` 两个整文件单元（任务 16712 日志各一行，1-23685 与 1-23684）都出自这一处读。
- 留：闭包里所有按函数收到的单元（含 `rust_lex`、`is_identifier_start`、`is_identifier_continue`、`block_document_rows`、1628 行的 `IDENT, LITERAL, LIFETIME, PUNCT` 赋值）与新写的那几行取指纹的代码本身。
- 出：整份文件的字节：注释、文档串、闭包没到的 def、自证夹具。
- 到：缓存目录名只决定「取哪一批已切好的词法单元」。结局通过「取到的词法单元对不对」影响单元指纹，所以目录名必须随一切会改变缓存内容的代码而变。新写法罩住的集合若不全，就是旧缓存照取。
- 反例（构造得出）：改法写的是「`rust_lex` 与它调到的帮手」。缓存内容还经过 `store_rust_lex`（2068，写）与 `load_rust_lex`（2047，读）：把 `load_rust_lex` 里 `texts = [table[code] for code in text_codes]` 改成取表项的前三个字符，目录名（只罩词法函数）不变，旧缓存里的词法单元被截短读出，单元指纹塌成少数几个值，改一处被判函数键不变。今天靠整份文件摘要罩着这条路；自证只有一格 `RUST_LEX_CACHE_CELL`（19213）判「同一份文本第二次取缓存与现切的逐项相同」，别的格全设 off（23450 `os.environ[RUST_LEX_CACHE_DIRECTORY_VARIABLE] = "off"`），不罩读写代码的改动。
- 所以：集合要取 `rust_lex_cached`（2101）的整个闭包，含 `rust_lex_cache_path`（2037）、`load_rust_lex`、`store_rust_lex`、`RUST_LEX_CACHE_FORM`（1806）与 `rust_lex` 的帮手；手写清单不行，要机器算（或手写清单加一格「清单等于入口闭包」的自证，见问题 4、5 的表）。
- 推翻它：若 `rust_lex_cached` 的闭包比 `rust_lex` 的闭包多的单元一个也不影响缓存内容（读写路径是纯搬运、且自证格覆盖到），则只罩词法函数也够；我读 `load_rust_lex` / `store_rust_lex` 的代码（2047-2098）看，读写都在改写数据（`kinds` 经 `kind_codes` 编号、`texts` 经 `table` 编号），不是纯搬运。

### 改法 2（写 `__doc__` 不算读）

- 实现今天：`lib_function_units.py:9091` `if node.id == "__doc__":` 与 9095 `if node.attr == "__doc__":` 两支都不看 `ctx`；1469 的 `InlineScript.__doc__ = ` 是 `Store`。记下的 owner 是 `"InlineScript"`，不在 `module.docstrings` 里（它是 namedtuple 赋值、不是 def 或 class），也认不出绑定的模块，走 14530 那一行 `other = module.module_of_binding(...)` 的 else，交 `module.docstring_units("*")`（10499 起，`owner == "*"` 取全部 owner）。推：这就是 558 段文档串整批进闭包的来路；没跑，建议用「要主 agent 跑的命令」第 1 条核。
- 留：真有 `ast.Load` 读 `__doc__` 的单元所读的那一段文档串。我在五份进闭包的 python 文件（`lib_function_units.py`、`crates-mutation-rows.py`、`lib_mutation_units.py`、`lib_reuse_checks.py`、`lib_libtest_result_lines.py`）与 `.claude` 下两份预检脚本、`scripts/hosts` 下三份里 grep `__doc__`：只有 1469（写）与 15840（在一个夹具字符串里，不是代码）两处，所以今天闭包里没有真读。
- 出：558 段文档串（任务 16712 日志里 `docstring` 行 558，全在 `lib_function_units.py`）。
- 到：文档串不进任何单元指纹（`without_docstrings`，6510），只经 `__doc__` 读才成为数据；不读就到不了结局。
- 反例（边角，写法要补）：`ast.AugAssign`（`X.__doc__ += "…"`）的目标 `ctx` 是 `Store`，实际先读后写；只看 `Load` 会漏。今天的闭包里没有这种写法，但改法写成「只在 `ast.Load` 上记」就留了口子，写成「`Store` 且不是 `AugAssign` 的目标」才对；`del X.__doc__` 的 `ctx` 是 `Del`，今天的 `facts.names` 那一支已把 `Del` 当读（9089 `isinstance(node.ctx, (ast.Load, ast.Del))`），属性这一支同样处理。
- 推翻它：闭包里出现一处真读 `__doc__` 而文档串仍然不进键。

### 改法 3（自证夹具与格名表搬出被判法载入的模块）

- 实现今天：规则在 `13-验证路线.md:434`（「python 模块有单元进闭包时，它在 import 时执行的模块级语句一起收进来」）；`crates-mutation-rows.py:1236` 的注释已经写明代价：「模块级的 import 与赋值都在判法单元的闭包里，多一处整张变异表的行键全变」。任务 16712 日志里进闭包的模块级赋值单元：`lib_function_units.py` 622、`crates-mutation-rows.py` 197、`lib_mutation_units.py` 25。粗数能搬的：`crates-mutation-rows.py` 的 197 里名字以 `SELFTEST_`、`FAKE_`、`Selftest` 起头的 54 个；`lib_function_units.py` 的 622 里起行不小于 15700（夹具区）的 448 个（只按位置数，是上限，不是按「没人引用」数的）；`lib_mutation_units.py` 里 `MULTIPLE_PACKAGES_FIXTURE`、`MULTIPLE_PACKAGES_CELL`、`SPECIFICATION_LINE_*` 四五个。
- 留：被判法里的函数读到的模块级语句（含 `NOT_RUN_LINE`、`DEVICE_LOST_LINE`、`GPU_TEST_FEATURES`、`CARGO_DEBUG_SETTING`、`COVERAGE_FLAG`、`VERDICTS_*`、`REUSABLE_VERDICT` 这类真改结局的）。
- 出：只给 `--selftest` 与弄坏开关用的夹具文本、格名表、开关表。
- 到：只有「闭包里某个函数读到被搬走的名字」这一条路。python 里这条路必须有 import（模块级 import 则整个新模块的模块级语句又进闭包，等于没搬；函数里 import 则只在那个函数进闭包时带上），名字动态取（`globals()[…]`、非字面 `getattr`）闭包算键时拒算（`13-验证路线.md:433` 要求的就是不静默少收）。所以正推下来到不了，条件是下面两点。
- 条件一：分类要按「谁读」不按「名字像」。例：`crates-mutation-rows.py:1343` 的注释说 `STAGE_VARIABLES` 是「run_stage 读的环境变量（自证起每一格之前从调用方的环境里清掉…）」，两头都读；`BREAK_VARIABLES`（1385）被 `break_is` 读。名字像自证、实际生产也读的不许搬。
- 条件二：`lib_function_units.py` 的 `<module-body>` 单元（日志里 1263-18242，一份文件一个）装着所有既不是 def、赋值、import 也不是 class 的模块级语句，搬赋值缩不了它；它里面有没有自证区的语句我没查。
- 范围提醒：`scripts/mutation/broad-mutation.py` 在任务 16712 的日志里一行单元也没有（`grep -c broad-mutation` 0 命中）：它不在 crates 变异判法的闭包里，我对它那一份判法的闭包没有读数，改法 3 里点它的名字对这 7986 行没有作用。
- 反例：推不出（查过：夹具赋值名字的引用方向；import 规则 `13-验证路线.md:434`；动态引用拒算 `13-验证路线.md:433`）。没跑闭包对照，所以是推的。
- 推翻它：搬完后用「物理删掉新文件再算判法闭包」，闭包的单元集合与有新文件时不同（说明有闭包里的函数引了搬走的名字）。

### 改法 4（判法指纹里去掉被判树的数据文件）

- 实现今天：`lib_function_units.py:6077` `built_unit_graph(repository)` 读 `6082` `thread_registry = os.path.join(repository, ".claude", "heavy-thread-default")`、`Cargo.lock`、`crates/singlefs-unit-graph` 下每个文件，摘要当编出来的 unit-graph 的缓存目录名；`build_unit_graph`（6113）把这三样拷进临时工作区编。编出来的程序在 `product_view_dump`（6059）里出产品行为视图的单元行，改变有 verus 的文件的单元指纹。日志里对应三行：`data-file .claude/heavy-thread-default:1-7`、`data-file Cargo.lock:1-1402`、`data-directory crates/singlefs-unit-graph/src:1-20`。
- 留：每行键里的依赖版本栏（`lib_mutation_units.py:766` `read_lock_packages(root)` 读被判根的 `Cargo.lock`，路径来自参数，今天不当数据单元收，所以与这三个数据单元无关）、工具链栏、点名测试闭包键与被改坏单元指纹（含 unit-graph 程序交出的产品行为视图那些行）。
- 出：unit-graph 源码字节、整份 `Cargo.lock`、登记文件。
- 到：unit-graph 程序对结局的影响只能经它交出的单元行，而单元行已经进每行的键；程序改了、交出的行没变，键不变是对的；交出的行变了，键变。所以这一条路正推到不了。这与 `admission.py:40` 对别的判定的口径一致（「只经它们读到的数据、起的程序不进键」），也与 `lib_function_units.py:13372` 登记的边（「算键的工具 unit-graph（编出来的二进制）：管怎么算键、不管判对错，与 NOT_JUDGING_FILES 同一个角色」）同一个角色划分。
- 反例（构造得出，条件缺一就红）：目录名摘要（6084 `inputs = [thread_registry, lock_file]` 加 6085-6087 的源码遍历）是「陈旧程序别复用」的唯一保险。今天还有「数据单元进判法指纹」罩着它；改法 4 搬走数据单元以后，若有人把 `lock_file` 从 `inputs` 里去掉，`Cargo.lock` 里 `verus_syn` 升版本，缓存里旧程序照取，产品行为视图还是旧程序出的，键不变，旧绿被复用。所以改法 4 必须带「摘要输入集等于程序真读的全部输入」的自证（问题 4、5 表）。
- 反例（范围漏洞，实现事实）：`.claude/heavy-thread-default` 还有第二条进闭包的路：`crates-mutation-rows.py:1121` 模块级 `from heavy_threads import HeavyThreadDefaultError, default_heavy_threads`、`5114` `with_gpu_test_threads` 里 `5134` `threads = str(default_heavy_threads(environment))`、`5274` `run_cargo_command` 调它（`run_cargo_command` 是 `JUDGING_ENTRIES` 里的一个，1169），`scripts/hosts/heavy_threads.py:33` 的 `REGISTRATION_FILE` 读文件。日志里 `scripts/hosts/heavy_threads.py` 的 `default_heavy_threads` 是进闭包的 def。只改算键库那条路，登记文件仍整份进判法指纹。要它离开，这条路也要经引用取：`init_worker` 已有先例（`crates-mutation-rows.py:3526` `gpu_released_targets=gpu_released_test_targets`）。
- 附带的事实：这个数在 `--test-threads` 里（5133-5136，只对 `GPU_TEST_THREADS_RELEASED_TARGETS` 点名的测试目标）。该函数的文档串自己写「补的是起命令时的参数，不进行键」（5120），`13-验证路线.md:432` 也把「资源上限」列在不进键之列，所以搬走与定案一致；但今天它因为数据文件的路进了判法指纹，改完以后线程数变了不会让这类行重判。线程数会不会翻转 GPU 测试的结局没量过，是未知，也不在我这条腿的范围，归主 agent 定要不要量。
- 推翻它：unit-graph 改了而交出的行没变时，判法指纹不变是对的；若发现交出的行没变而某行结局变了，路径在别处。

### 改法 5（`kept-comments` 只收真被读的指令词）

- 实现今天：`lib_function_units.py:10539` `KEPT_SHELL_COMMENT = re.compile(` 任何「`# 词:`」或 `# shellcheck` 都收，`kept_shell_comments`（10551）逐行取，闭包碰到这份文件任一单元就 `14934` `self.closure.add(shell_file.kept, unit, "kept-comments")`。crates 变异判法的闭包里没有 shell 单元，任务 16712 日志里 `kept-comments` 0 行，改法 5 对那 7986 行没有作用。
- 查到的事实（`git ls-files '*.sh'` 去掉 fixtures 共 208 份，取行首「# 词:」的写法数）：`gate-lint:` 897、`gate-cell:` 196、`gate-similar:` 163、`run-condition:` 94、`admission:` 84、`shellcheck` 38，另有 `gate-fixture`、`account-quota`、`task-server-card-lease`、`gate-stage`、`gate-category`、`task-server-compiles`、`task-server-shards`、`long-run-*`、`hook-events`、`process-safety`、`selftest-serial`、`change-range`、`positive-control`、`clock-times`、`stdin`、`noqa` 等共五十来个词。改法 5 列的七类名单（`admission`、`run-condition`、`selftest-serial`、`task-server-*`、`gate-cell`、`gate-covers`、`shellcheck`）没有 `gate-lint`（897 处）、`gate-similar`、`gate-overlap`、`hook-events`、`long-run-*` 等。其中真有读它的程序的，我只点到一处：`.claude/singlefs-ai-sop/scripts/gate-overlap.py:89` 的 `HOOK_EVENTS_RE`；别的词只数了仓里提到它的文件数，没有逐个核读法，是「没查」。
- 留：名单里的词的注释行与 `#!` 行。出：别的注释。
- 到：注释到结局要两步：有程序读它，且那个程序的行为在这条判定的闭包里。读法有三种：程序按路径读这份脚本（python 数据读：整份字节进键，kept 多余）；程序在脚本自己运行时读它头上的指令（`14938-14940` 的 `declared_checks(self.root, unit.file, shell_file.rows)` 那一路，`admission:`、`run-condition:` 在这里）；外部工具读（`# shellcheck`）。只有后两种靠 kept。
- 反例（构造得出，是设计性的，仓里今天有没有实例没核）：新加一个程序 X 在某脚本 S 运行时读 `# newword: 取值`，`newword` 不在名单；把取值改了，S 所在判定的闭包单元不变、kept 不变、键不变，X 的行为变了，旧绿被复用。这是名单极性（漏登 = 少收）造成的，今天「任何词都收」是安全的一边。
- 建议极性取反：登记的是「核实过没有程序读」的词（漏登只是多收、安全），不是「有人读」的词。判「没人读」的自证见问题 4、5 表。
- 推翻它：逐个词核完读法，名单七类之外所有词都确无读者、且新读者出现时有机器拦截，则名单极性也可以。

### 改法 6（记账代码出判法闭包）

- 实现今天：`crates-mutation-rows.py:5364` `judge_row`。核心是 5368-5393：读源码、`pristine.count(row.old) != 1` 的锚点判、改坏、`5382` 起 `run_cargo_command`、还原、`5390` `verdict_of_run`、`5392` `gpu_card_unused_reason` 把「没红」改成 `notrun`。记账是 5396-5420：`row_key` 组件、`fields`、`write_record_quietly`、用时表 `write_atomically`，加 5388 的 `measure_and_remove_temporary_directory`。
- 留：改坏、起 cargo、`verdict_of_run`、`notrun` 的转换、`with_shown_output`、`run_cargo_command`、`prepare_shard`（`JUDGING_ENTRIES`，1151-1171 的整组）。
- 出：写记录、用时表、占盘、说明文字。
- 到：三条。一，`verdict` 的值不经记账改动，只要核心把 `verdict` 交出去、外壳只读它；二，写不写、盖不盖旧记录的策略（5399-5404，含 5402 `keeps_counted = bool(components.get("row_key"))…`、`red-not-recorded`）决定库里存什么、下一趟复用什么，是「红的也照键记」（`13-验证路线.md:436`）的落点，策略放进外壳就脱离判法指纹，这一类毛病（例：改成红不撤绿）键不变、旧绿被复用；三，删临时目录那一步每行用自己的 TMPDIR，不串行。
- 第二条是边界：这是「怎么存」不是「判对错」，与 `NOT_JUDGING_FILES` 同一个角色划分，可以出，但保护要换成自证：`red-not-recorded`、`records-written-at-end`、`narrowed-verdicts-recorded` 这几个弄坏开关配的格要在拆完之后还跑到外壳（它们在 `--selftest` 里，不在判法闭包里，拆壳不会让它们消失，但要确认它们经引用调的外壳，不是调了一个替身）。
- 附带：改法 6 不让整份 `lib_reuse_checks.py`（日志里 67 个单元，只有 `write_atomically` 一个 def 被调）出闭包：`crates-mutation-rows.py:1135` 模块级 `_REUSE_CHECKS_SPECIFICATION = importlib.util.spec_from_file_location(…)` 载入它，模块级语句随模块进来（见问题 6）。
- 反例：推不出（查过：`verdict` 的流向、记录策略的位置、临时目录路径）。
- 推翻它：外壳里出现任何会改 `verdict` 或 `notrun` 转换的语句，或记录策略被改而没有自证红。

### 改法 7（编译档里不改运行结果的设置移出键）

- 实现今天，Rust 一侧：`crates/singlefs-unit-graph/src/build_toolchain.rs:17` `pub const PROFILE_SETTING_KEYS: [&str; 8] = [`，`builtin_profile` 的默认值数组按同一次序（32 行起 `dev`：`debug` 真、`incremental` 真、`codegen-units` 256；`release`：假、假、16），环境变量覆盖只循环这八个键（146-150）。python 一侧至少四处各自摊开：`scripts/mutation/lib_mutation_units.py:737` 凡是 `CARGO_PROFILE_` 起头的环境变量整条进键行、745-750 cargo 配置里 `profile` 一张表整张摊开、758 起根 `Cargo.toml` 的 `profile` 整张表摊开；`scripts/gate/admission.py:1568` `profile_lines` 摊开 `[profile.<名>]`、1189 `configuration_affecting_output` 的 `profile` 一节。只改 Rust 的 `PROFILE_SETTING_KEYS` 与 python 的「`[profile.<名>]` 摊开」两处不够，环境变量那一支（`CARGO_PROFILE_<名>_DEBUG|INCREMENTAL|CODEGEN_UNITS`）与配置文件那一支都要同步。Rust 与 python 两份算同一个键，门禁 code-tooling 的 function-units-two-implementations-agree 格逐项对拍，只改一边会红。
- 留：`opt-level`、`debug-assertions`、`overflow-checks`、`lto`、`panic`（连同 `RUSTFLAGS`、目标三元组、编译器版本、`[profile]` 里别的键与嵌套表）。出：`debug`、`incremental`、`codegen-units`。
- 已有的先例与矛盾：`incremental` 在 `[build]` 一侧今天已不进键（`admission.py:847`、`lib_mutation_units.py:97` 的 `BUILD_KEYS_NOT_AFFECTING_OUTPUT = frozenset(("jobs", "target-dir", "incremental"))`），改法 7 是把同一条口径推到 `[profile]`。`debug` 这一项反而是变异门禁自己显式放进键的：`crates-mutation-rows.py:1202-1203` `CARGO_DEBUG_PROFILES`、`CARGO_DEBUG_SETTING = ["false"]`，四档都设 `CARGO_PROFILE_*_DEBUG=false`，`key_toolchain_lines`（1409）照起 cargo 的环境算，注释（1201）说「自证『改了构建档的取值之后行键变了』那一格就地换它」。该格与弄坏开关 `debuginfo-unkeyed` 断言的正是改法 7 要推翻的东西，要改写，不是补一格。
- 一个事实对改法 7 有利：变异门禁起 cargo 时环境里的 `CARGO_PROFILE_*_DEBUG=false` 已盖掉 `Cargo.toml` 里的 `debug`（cargo 的优先次序：配置与环境盖清单，我没跑，是 cargo 文档的读法），所以仓根 `[profile.release] debug = true` 改成 `"line-tables-only"` 对变异门禁的编译没有影响，却今天会让整张变异表的行键全变（清单里 `profile` 整张表进行键，`lib_mutation_units.py:758`）。
- 到：
  - `debug`：panic 位置来自编译期的 `Location::caller()`（`crash_library.rs:1172`）与 `line!()`，不来自调试信息；`crates/` 下 `*.rs` 里没有 `backtrace` 字样（我 grep 过，零命中）；没有测试读自己的调试信息。推不出到得了结局的路。
  - `incremental`：不改语义，是编译器的缓存；已知错编是编译器缺陷。`dev` 档今天就带增量（编译目录里的 `incremental` 目录，`crates-mutation-rows.py:943` 起的占盘说明写到「debug/incremental 13G」），不是新引入的风险。
  - `codegen-units`：最弱的一项。有增量时 cargo 的默认值是 256 而不是 16，改法 7 的第二步「release 开增量」会让 release 的有效 `codegen-units` 从 16 变成 256 而键不变；它经内联与栈帧大小改栈深：`crates/singlefs-verification/tests/allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf.rs:39` `const STACK_BYTES_OF_THE_PUBLISHING_THREAD: usize = 64 << 20;` 是现成的栈深敏感用例（余量 64 MiB，是否够我没量）；`singlefs-core` 的 `records.rs`、`unit.rs`、`journal.rs`、`container_tree.rs` 与产品 GPU 包里有 `unsafe`（`crates` 下 20 份文件含它），优化器相关的未定义行为在这些文件里有暴露面。反例推不出具体的一个，也排除不了。
- 判：`debug`、`incremental` 推不出反例；`codegen-units` 不建议和前两项一起出键，除非先拿同一批测试在「有增量 256」与「无增量 16」两种下比一次结局（命令见「要主 agent 跑的命令」，属可选，问题 3 不在我这条腿）。
- 推翻它：同一批点名测试在两种编译档下有任何一条结局不同。

### 改法 8（重活线程数的登记文件只把数编进二进制）

- 实现今天：`crates/singlefs-verification/src/heavy_threads.rs:22` `pub const REGISTRATION_FILE_TEXT: &str = include_str!(concat!(` 整份编进来；`35` `heavy_threads_registered_in` 只认第一条非注释行；`59` `count_of_the_registration_line` 要「数、至少一个空白、`#`、非空理由」，`72` 行的 `let reason = after_the_blanks.strip_prefix` 起判理由非空。python 一份 `scripts/hosts/heavy_threads.py:41` `REGISTRATION_LINE = re.compile(` 同一个形状，shell 一份 `scripts/hosts/heavy-threads.sh` 同。`crates/singlefs-unit-graph/src/heavy_threads.rs:20` `first_registered_threads` 不一样：只取第一条非注释行的头一个词，不要理由。`crates/singlefs-verification/src/bin/e184_regular_layer_cost_counts_write_dump/mod.rs:44` 还有自己的一份 `include_str!`。登记文件头 5 行「写法：第一条非注释行」。
- 变体 a（只留数）：文件里只剩 `31`。四个读者里三个（verification 的 Rust、python、shell）会拒（要 `# 理由`），unit-graph 那一份接受。写成「第一条非注释行仍是 `31  # 理由`，说明挪走」才不红：留在键里的是数加理由那一行，理由文字改了仍然整批重判，只是少了头部说明与后面行的改动。
- 变体 b（只取第一条非注释行）：读的一侧本来就只取第一条非注释行（`heavy_threads.rs:35` 起），后面的行与头部注释对结局没有路。数据单元的指纹改成「第一条非注释行的文字」够用。取「解析出的数」不够：把 `31  # 理由` 改成 `31  #`（理由空）会让 verification、python、shell 三个读者从接受变成拒（`SINGLEFS_THREAD_CAP` 没设时测试读登记就 panic），解析出的数相同、键不变、结局变（红），是构造得出的反例；指纹必须含「是否合法」。四个读者对「合法」的口径又不一样（unit-graph 那份宽），所以取一个口径要写明：取最严的，或取第一条非注释行原文。
- 到：数本身经 `default_heavy_threads` 进线程数（工作线程、`--test-threads`，见改法 4）。头部说明、第一条之后的行不被任何读者读，到不了结局。`REGISTRATION_FILE_TEXT` 是 `pub const`，今天只有同文件内部与 `registered_heavy_threads` 用它（grep 全 crates：只有两份 `heavy_threads.rs` 与 e184 那份自己的常量）。
- 要同步的自证与代码：`crates/singlefs-verification/src/crash_identity.rs:1626` `the_data_files_outside_the_crates_include_the_heavy_thread_registration`（登记文件仍进被监视文件表）；`build.rs` 的 `rerun-if-changed` 仍盯整份（cargo 重编，不是键）；两份实现（Rust unit-graph 与 python 函数单元）的数据单元规则要同改，否则对拍格红。
- 反例：变体 a 照字面写（只留数）会让三个读者拒；变体 b 若指纹取解析出的数会漏「理由空」。其余推不出。
- 推翻它：任一读者读了第一条非注释行之外的东西。

## 问题 2：定案第四条「crates 变异判法除外」那一句

### 现行原句与实现的对账

`13-验证路线.md:434` 里的原句（连同上下文的半句）：「crates 变异判法（`scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/lib_mutation_units.py`）除外：那里这两份照进判法单元指纹，覆盖号也带判法单元指纹」。「这两份」指前面的 `NOT_JUDGING_FILES` 两份：`scripts/crash/lib_verdict_library.py` 与 `scripts/gate/lib_function_units.py`（`lib_function_units.py:13358` `NOT_JUDGING_FILES = ("scripts/crash/lib_verdict_library.py", "scripts/gate/lib_function_units.py")`）。

三处与实现对不上，收窄之前就存在：

1. 「这两份」里 `lib_verdict_library.py` 在 crates 变异判法的闭包里一个单元也没有：任务 16712 日志 `grep -c lib_verdict_library` 是 0；`crates-mutation-rows.py` 不载入它。真进闭包的只有 `lib_function_units.py`。
2. 句子说的是「单元」进判法指纹，实现里进的不止单元：整份 `lib_function_units.py` 的字节（`source-text` 与 `data-file` 两行，1-23685 与 1-23684）、558 段文档串、夹具赋值等。其中整份文件字节与 `13-验证路线.md:432` 的「整文件原文；文件级、包级、整表级的指纹或闭包」一律不进键直接冲突。也就是说，今天的实现既超出了这一句的字面，也踩着定案第二条。
3. 句子点名 `crates-mutation-rows.py` 与 `lib_mutation_units.py` 整份，实现里进闭包的是 `JUDGING_ENTRIES`（`crates-mutation-rows.py:1151-1171`）收到的那一部分（`crates-mutation-rows.py:148` 的原话：「lib_function_units.py 经 executed_units 管覆盖落进哪几个单元，覆盖决定基线复用，是判对错的一环，它的单元照进指纹、里面的拒算照算」，这是「除外」的理由）。

### 第 1、3、4、6 条收窄之后，这一句还成不成立

- 理由还成立：覆盖落进哪几个单元（`UnitKeys.executed_units`）决定基线复用，是判对错的一环，所以 `lib_function_units.py` 被它们调到的函数单元仍然必须进。收窄后进的恰好是「单元」，句子才名副其实。
- 字面不再成立的部分：
  - 「这两份」要改成只指 `lib_function_units.py`，因为 `lib_verdict_library.py` 没单元进。
  - 「照进」要写清进的是什么、不进什么，否则读者会把今天的整份字节读成合规。改法 1（整份字节不进）、改法 3（只给自证用的模块级语句不进）、改法 4（算键库为自己编程序与缓存读的数据不进）、改法 6（记账与派活代码不进）都落在「不进」那一边，需要在条款里点出，不然下一个改动又会把它们带回去。
  - 点名文件要改成点名入口：进的是 `JUDGING_ENTRIES` 收到的单元，不是两份文件整份。

### 改写后的整句（建议，供主 agent 采纳；我不替主 agent采纳）

「crates 变异判法除外：判「抓没抓到」的入口（`scripts/mutation/crates-mutation-rows.py` 的 `JUDGING_ENTRIES` 登记的那一组）从 `scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/lib_mutation_units.py` 与 `scripts/gate/lib_function_units.py` 收到的函数单元，连同 import 时执行的、被这些函数读到的模块级语句，照进判法单元指纹，覆盖号也带判法单元指纹；这三份的整份原文、文档串、只给 `--selftest` 与弄坏开关用的模块级语句、写记录与派活与报数的代码、算键库为自己编程序与缓存而读的数据文件与源码目录，不进。`scripts/crash/lib_verdict_library.py` 照上面的按角色排除，crates 变异判法的闭包里今天没有它的单元。」

### 对着定案第二条的「键过窄」核一遍（`13-验证路线.md:432`）

第二条末句原文：「键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反」。

| 第二条点名的四类 | 改写后一句与四条收窄是否落进 | 依据 |
|---|---|---|
| 引用的常量、类型、宏漏收 | 改法 3 有这个风险：搬走的名字若被闭包里的函数引到，就是「引用的常量漏收」；静态上到不了（引用要有 import，动态引用拒算），条件是分类按「谁读」。改法 1 同类：缓存读写函数漏进目录名集合（问题 1 改法 1 的反例） | 问题 1 改法 1、3 |
| 库放在会丢的位置 | 不涉及：缓存目录丢了只是重算，判定库只有 prune-server 那一份，改法没动 | `13-验证路线.md:435` |
| 改名丢库 | 键算法一变所有行键一次性变，旧库作废；不是「改名丢库」，是 `13-验证路线.md:437` 的「改键让旧库作废不算代价」，改完重跑一次 | `13-验证路线.md:437` |
| 两份库重复判同一批 | 不涉及 | 同上 |

还要对第三条（`13-验证路线.md:433`）：「收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收」。整份字节出键以后，今天靠整份字节兜底的那一类漏（闭包分析器漏连了算键库里的某个函数）失去兜底：分析器漏连一个就是静默少收。python 闭包分析只有 `lib_function_units.py` 一份实现，没有像 Rust 那一份有第二条独立实现对拍（文件头 9-11 行写的两条不共用代码的路子是 Rust）。所以改写后的句子要配一格「动态 ⊆ 静态」的独立校验（问题 4、5 表里改法 1、3 那两行），否则收窄就是把兜底换成了单一来源的信任。

结论：改写后的句子不落进「键过窄」，条件是问题 4、5 表里的三类自证格都在；没有这些格，改法 1、3、4 都有构造得出的反例。

## 问题 4、5：登记词表漏登、八条各自的自证格

### 漏登一个真被读的指令词会怎样（改法 5）

- 词表是「有人读」的登记（允许名单）时：漏登的词，它的注释行从 `kept-comments` 单元里消失；读它的程序若在某条判定的闭包里，改这个词的取值，闭包里所有单元的指纹不变、键不变，程序行为变了，旧结论被复用。就是 `13-验证路线.md:432` 末句的「键过窄」，也违反 `13-验证路线.md:433` 的「不静默少收」。今天任何词都收（10539），是安全的一边。
- 让漏登必红有两条路，我推荐第二条：
  1. 允许名单加扫描：一格扫全仓 `.py`、`.sh`、`.rs` 里把「`# 词:`」当模式读的写法（`re.compile(r"…#\s*词:")`、`grep -E '^# 词:'`、`startswith("# 词:")`），每个词都必须在词表里。字面模式能扫；模式由变量、`f"# {name}:"` 拼出来的扫不到，那一处要当「读者未知」直接红（与 `13-验证路线.md:433` 同一个口径）。漏扫一个动态读者就漏登。
  2. 极性取反：登记的是「核实过没有读者」的词（否定名单），不在名单里的词一律收。漏登只是多收、不是少收；错的只可能是「误登」，由一格扫描判：名单里每个词，全仓扫不到任何读它的模式。误登的反例输入：一份夹具脚本里写 `# newword: a`，夹具里另有一份 python 带 `re.compile(r"^#\s*newword:")`，名单里登了 `newword`，这一格必红。
- 两条路都要配「改词的取值，键变不变」的正反两格（登记 / 不登记 × 有读者 / 无读者）。

### 要先说明的三类自证

- 「机器算的集合等于手写集合」：改法 1 的目录名集合、改法 5 的名单、改法 3 搬走的名字集合，凡是手写的，都配一格「手写集合 == 从入口收出的闭包」，红的输入是往被收的函数里加一个新帮手调用而不更新集合。
- 「动态 ⊆ 静态」（独立校验，不共用分析器）：改法 1、2、3、6 把整份文件字节这道兜底拿走以后，python 闭包只剩分析器一路（Rust 那一份有第二实现对拍，python 没有）。自证里已有假 cargo 的夹具（`crates-mutation-rows.py` 的 `FAKE_CARGO`、`SELFTEST_*` 一族）：在夹具上跑 `judge_row`、`judge_batch`、`UnitKeys.executed_units`，用运行期跟踪记下真被执行的函数，必须都在静态闭包里；红的输入是把核心里一处直接调用改成 `getattr(库, 变量名)(…)`，静态应当拒算，跟踪应当抓到被调用的函数不在闭包里。
- 现有自证格要改写的：`lib_function_units.py:17118` `EXCLUDED_FILE_UNSET_CELL`（「排除文件：不给排除（crates 变异判法那一路）时它读的数据照进键（只改判法文件一行注释键变）」）与 `17117` `EXCLUDED_FILE_KEPT_CELL`（「排除文件：判法自己的单元照进键（改它键变）」）：前一格今天断言「改判法文件一行注释键变」，改法 1 之后要断言「改注释键不变、改它调到的函数键变」；`crates-mutation-rows.py:1201` 说的「改了构建档的取值之后行键变了」那一格与弄坏开关 `debuginfo-unkeyed`（改法 7）；`lib_function_units.py:20996` 一族「真仓：改 … 一个模块级 import，形式化入口的键变」不动（规则不动）。

### 八条各自要配的自证格与弄坏开关（该红的输入）

弄坏开关的名字是我给的描述性名字，主 agent 或实现员可改，不是已有名字；写法照现有的：环境变量 `FUNCTION_UNITS_BREAK`（算键库）、`GATE_MUTATION_BREAK` 或 `CRATES_MUTATION_ROWS_BREAK`（变异门禁）、`ADMISSION_BREAK`。

| 条 | 自证格与该红的输入 | 弄坏开关（打开后哪一格该红） |
|---|---|---|
| 改法 1 缓存目录名 | 格 a：集合里每个成员各改一个词法单元（`rust_lex` 里 `quoted_end` 的返回值、`is_identifier_continue`、`load_rust_lex` 的解码、`store_rust_lex` 的编码、`RUST_LEX_CACHE_FORM`），目录名每次都变；格 b：改注释、文档串、集合外的自证夹具，目录名不变（证明收窄成立）；格 c：集合等于 `rust_lex_cached` 入口闭包，红的输入是给 `rust_lex` 加一个新帮手调用；格 d：闭包里不再有本文件的 `source-text` 与 `data-file`，红的输入是把 `open(os.path.realpath(__file__))` 放回去 | `rust-lex-cache-namespace-lexer-only`（集合只罩 `rust_lex` 与词法帮手，格 a 对 `load_rust_lex`、`store_rust_lex` 的那几次红）；`rust-lex-cache-namespace-reads-own-file`（放回整份读，格 d 红） |
| 改法 2 写 `__doc__` 不算读 | 格 a：夹具模块只有 `Foo.__doc__ = "x"`，别的函数的闭包里没有 `docstring` 单元，改任一文档串键不变；格 b：`return helper.__doc__` 仍收（沿用 22855 行那一格）；格 c：`Foo.__doc__ += "x"` 与 `del Foo.__doc__` 记为读：改 `Foo` 的文档串键变，红的输入是只按 `Load` 判 | `python-docstring-writes-counted`（写也记，格 a 红）；`python-docstring-augmented-assignment-not-read`（只看 `Load`，格 c 红） |
| 改法 3 夹具搬出 | 格 a：判法入口的闭包在「新文件物理删掉」与「在」两种下单元集合相同，红的输入是让闭包里的一个函数引搬走的一个名字；格 b：`--selftest` 仍找得到夹具，红的输入是新文件缺失；格 c：改搬走的夹具键不变、改一个真读的生产常量（`NOT_RUN_LINE`）键变；格 d：搬走的名字没有被生产函数引用，红的输入是 `STAGE_VARIABLES` 这类两头读的名字被误搬 | `judging-function-reads-moved-fixture-name`（核心里一个函数读一个搬走的名字，格 a、d 红）；`selftest-fixtures-file-absent`（格 b 红） |
| 改法 4 数据文件出判法指纹 | 格 a：`built_unit_graph` 的摘要输入覆盖 `Cargo.lock`、登记文件、源码目录里每一份文件（各改一个字节，目录名变）；格 b：改这三样，判法指纹不变；格 c：换一个会交出不同单元指纹的假 unit-graph 程序，有 verus 夹具文件的行键变（程序输出进键）；格 d：改整份 `Cargo.lock` 里与闭包无关的包，行键的依赖栏不变 | `unit-graph-binary-digest-skips-lock-file`（摘要少一份输入，格 a 红）；`unit-graph-inputs-kept-as-data-units`（放回数据单元，格 b 红）；`product-view-output-ignored`（行键不取程序输出，格 c 红） |
| 改法 5 `kept-comments` | 格 a：读者扫描（问题 4、5 上面第二条路的误登格）；格 b：未登记的词改取值键变；格 c：登记的词（无读者）改取值键不变；格 d：登记后新加一个读它的程序，格 a 变红；格 e：读者模式动态拼出，红 | `kept-comments-deny-list-skips-reader-scan`（不扫读者，格 a、d 红）；`kept-comments-allow-list-only`（取反回允许名单，格 b 红） |
| 改法 6 记账出闭包 | 格 a：改 `RECORD_HEADER`、`how_text` 文字、用时表写法，判法指纹不变；格 b：改 `pristine.replace(row.old, row.new)` 那一步、`gpu_card_unused_reason` 的分支、`verdict_of_run`，指纹变；格 c：外壳替身改 `verdict` 或 `notrun` 转换，核心交出的结局不变（外壳只读）；格 d：`red-not-recorded`、`records-written-at-end`、`narrowed-verdicts-recorded` 三个现有开关打开后，它们的格仍然红 | `judge-row-shell-may-alter-verdict`（外壳能改结局，格 c 红）；`judge-row-records-inside-core`（记账放回核心，格 a 红）；现有三个开关不动 |
| 改法 7 编译档三项 | 格 a：`debug`、`incremental`、`codegen-units` 各自在 `Cargo.toml`、环境变量、cargo 配置三处改，Rust 一侧与 python 一侧（`lib_mutation_units.py`、`admission.py` 各处）键都不变；格 b：`opt-level`、`debug-assertions`、`overflow-checks`、`lto`、`panic` 五项各改键变；格 c：两份实现对拍同一批输入逐行相同；格 d：改写「改了构建档的取值之后行键变了」那一格，改用 `opt-level` | `profile-keys-all-eight`（八项都进，格 a 红）；`profile-keys-python-side-only-manifest`（python 只收清单、漏环境变量与配置，格 a 的环境变量与配置两处红） |
| 改法 8 登记文件 | 格 a：改头部说明与第一条之后的行，键不变；改第一条非注释行的文字，键变；格 b：把 `31  # 理由` 改成 `31  #`、`31`、`0  # 理由`，键变（且读者拒）；格 c：四个读者（verification 的 Rust、unit-graph 的 Rust、python、shell）对这几种登记的合法性逐个对拍，红的输入是 `31  #`（unit-graph 那份接受、别的拒）；格 d：两份实现对同一份夹具 `include_str!` 文件的数据单元指纹相同 | `registration-fingerprint-parsed-number-only`（指纹取解析出的数，格 b 红）；`registration-fingerprint-whole-file`（放回整份，格 a 红） |

## 问题 6（前半）：同一份闭包清单里还有没有同类的宽

有。查过的范围：任务 16712 日志 2105 个单元按文件与种类数一遍，再对着源码核其中大头的来路；没有跑 `--explain`，来路是读代码推的，标「推」。

1. `lib_function_units.py` 里 `<module-body>` 一个单元就占日志里 1263-18242 行，一份文件一个。所有既不是 def、赋值、import 也不是 class 的模块级语句（`InlineScript.__doc__ = …` 是其中一条）都在它里面；改法 1、2、3 缩不了它。是否含自证区的语句没查，建议跑下面第 1 条命令后按行区间核。
2. 整个模块被载入、只用到其中一个函数的：
   - `crates-mutation-rows.py:1119` `from project_preflight import preflight`、`1121` `from heavy_threads import HeavyThreadDefaultError, default_heavy_threads`、`1122` `import multi_host_addresses`、`1135` `_REUSE_CHECKS_SPECIFICATION = importlib.util.spec_from_file_location(…)`；`lib_function_units.py` 的 1261、1265 也有同样的 `preflight` 载入。
   - 日志里这几份进来的单元数：`.claude/singlefs-ai-sop/scripts/preflight.py` 43（没有一个 def）、`scripts/hosts/multi_host_addresses.py` 26（没有一个 def）、`scripts/gate/lib_reuse_checks.py` 67（一个 def `write_atomically`）、`scripts/hosts/heavy_threads.py` 21（含 `SELFTEST_CELLS`，94-108）、`.claude/scripts/project_preflight.py` 9，共 166。它们对结局的影响最多是 `write_atomically`（改法 6 之后只在外壳里）与 `default_heavy_threads`（改法 4、8）。同类的先例在 `crates-mutation-rows.py:1236` 的注释里：对 `task_memory` 已经改成函数体里 import，说明这一类做法站得住。`SELFTEST_CELLS` 与改法 3 同类，但在 `heavy_threads.py` 里，改法 3 没点名。
3. `crates-mutation-rows.py` 里不是自证的模块级赋值：197 个赋值减去 `SELFTEST_`、`FAKE_`、`Selftest` 起头的 54 个是 143 个，里面大头是调度、内存、报数、判定库存放与判法指纹缓存：`WORKER_PEAKS_RECENT`、`WORKER_PEAK_SAMPLE_SECONDS`、`DEFAULT_TARGET_DIRECTORY_CAP`、`DEFAULT_DISK_RESERVE`、`DEFAULT_LONG_ROW_SECONDS`、`STOP_GRACE_SECONDS`、`REASONS_SHOWN`、`REUSE_COLUMNS`、`COLUMN_TITLES`、`RECORD_HEADER`、`CGROUP_LISTING`、`JUDGING_CACHE_*`、`REACHABILITY_*`、`RELEASE_HOWTO`、`ITEM_*`、`SEED_*`。改法 3 只搬自证夹具，改法 6 只拆 `judge_row`，这 143 个模块级赋值仍随模块进闭包，改一个就让整张变异表重判（`crates-mutation-rows.py:1236` 的注释已承认）。真改结局的有（`NOT_RUN_LINE`、`DEVICE_LOST_LINE`、`GPU_TEST_FEATURES`、`COVERAGE_FLAG`、`CARGO_DEBUG_SETTING`、`VERDICTS_*`），不能一刀切，要逐个按「谁读」分。
4. `lib_function_units.py` 生产区的模块级常量：夹具区之前（起行小于 15700）还有 174 个赋值（622 减 448），含缓存与程序保留数（`RUST_LEX_CACHE_KEPT_NAMESPACES = 3` 在 1809，只管留几份旧目录）、`UNIT_GRAPH_*` 一族的编程序旋钮；这类「只管留几份、等几次」的常量属于怎么算键、不属于判对错，与改法 1 同类，改法里没有。
5. 13 份 `Cargo.toml`（根 1 份、各 crate 11 份、`prune-server` 1 份）整份字节作数据单元进判法指纹（日志里 `data-file` 共 16 行，另有 1 行 `data-directory`）。改法 4 只点 `Cargo.lock`、登记文件与 unit-graph 源码目录，没点它们。`13-验证路线.md:432` 把「整文件原文」列在不进键里，而依赖版本、特性已经各在行键自己的栏里（`scripts/mutation/lib_mutation_units.py:724` 起的 `toolchain_lines`、依赖栏）。谁读这些清单没核出来（`prepare_shard` 拷哪些的计算是我的猜测之一），要用下面的 `--explain` 核来路。
6. 条款自己对不上的一处（不算宽，算说多了）：见问题 2 第 1 点，`NOT_JUDGING_FILES` 的另一份 `lib_verdict_library.py` 在这份闭包里没有单元。

## 冲突整行并排（两边各抄原文）

1. 改法 1 与现有自证格：
   - 现有格（`scripts/gate/lib_function_units.py:17118`）：`EXCLUDED_FILE_UNSET_CELL = "排除文件：不给排除（crates 变异判法那一路）时它读的数据照进键（只改判法文件一行注释键变）"`
   - 改法 1 的后果（背景材料第四节改法 1）：闭包里那两个整文件单元随之消失，改判法文件一行注释键不变。两边相反，这一格要改写。
2. 改法 7 与变异门禁自己的约定：
   - 现有注释（`scripts/mutation/crates-mutation-rows.py:1201`）：`# 取值放在一格的列表里：自证「改了构建档的取值之后行键变了」那一格就地换它`
   - 改法 7（背景材料第四节改法 7）：`debug`、`incremental`、`codegen-units` 移出键，键里不再带构建档的这几项取值。两边相反。
3. 改法 8「只留数」与读者：
   - 读者（`crates/singlefs-verification/src/heavy_threads.rs:72`）：`    let reason = after_the_blanks.strip_prefix('#')?;`，其后（73 行起）要求理由非空；`scripts/hosts/heavy_threads.py:41` 的 `REGISTRATION_LINE` 同。
   - 改法 8（背景材料第四节改法 8）：`.claude/heavy-thread-default` 只留数、说明挪到另一份。只剩 `31` 时这些读者拒。
4. `13-验证路线.md:434` 的原句与日志：
   - 条款（`13-验证路线.md:434`）：「那里这两份照进判法单元指纹」，两份指 `NOT_JUDGING_FILES`（`lib_function_units.py:13358`）。
   - 日志（任务 16712，草稿目录 `task16712.txt` 第 71、72 行）：一行种类 `data-file`、限定名是 `lib_function_units.py` 的路径，起止是第 1 到 23684 行；一行种类 `source-text`、限定名 `<source-text>`，起止是第 1 到 23685 行；两行指纹同为 `9588a52e5bbf14f5598a66af7c567b8740c89a618a1cba1cf7fc52b038136d84`：进的是整份字节，不是单元；`lib_verdict_library` 在日志里 0 行。

## 要主 agent 跑的命令

我一条都没起。下面四条是只读的闭包诊断（算键库自己的 `closure --explain`，没有范围声明的种类，照任务 16712 那一条的形态，不带范围声明），目的是把本报告里标「推」的三件事核成「量到的」：558 段文档串的来路是不是 `InlineScript.__doc__ = …`；`.claude/heavy-thread-default` 是不是两条路进；13 份清单谁读。

1. 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 scripts/gate/lib_function_units.py closure /home/fy5090/code/singlefs scripts/mutation/crates-mutation-rows.py::judge_row --explain`
   - 在哪个目录起：`/home/fy5090/code/singlefs`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；`task-server/task log <号> --all` 里 `via` 行：规则名是 `docstring` 的各行，「从哪个单元来」是不是只有 `<module-body>`（落在 `lib_function_units.py` 的 1263-18242）或 `InlineScript`；规则名是 `source-text` 的那一行从哪个单元来（预期是 `rust_lex_cache_namespace`）。
2. 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 scripts/gate/lib_function_units.py closure /home/fy5090/code/singlefs scripts/mutation/crates-mutation-rows.py::run_cargo_command --explain`
   - 在哪个目录起：`/home/fy5090/code/singlefs`。
   - 看哪几行：同上的退出行；`via` 行里涉及 `.claude/heavy-thread-default` 的那一行（预期规则名 `data-read`，来自 `scripts/hosts/heavy_threads.py` 的 `default_heavy_threads`）。
3. 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 scripts/gate/lib_function_units.py closure /home/fy5090/code/singlefs scripts/mutation/crates-mutation-rows.py::prepare_shard --explain`
   - 在哪个目录起：`/home/fy5090/code/singlefs`。
   - 看哪几行：同上的退出行；`via` 行里 `…/Cargo.toml` 与 `Cargo.lock` 各行从哪个单元来、什么规则。
4. 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 scripts/gate/lib_function_units.py closure /home/fy5090/code/singlefs scripts/mutation/lib_mutation_units.py::UnitKeys.executed_units --explain`
   - 在哪个目录起：`/home/fy5090/code/singlefs`。
   - 看哪几行：同上的退出行；这一个入口收到的 `lib_function_units.py` 的 def 有几个、是不是都不含自证区的函数（对着「覆盖决定基线复用」那一句核改写后的条款说得对不对）。

可选（改法 7 的 `codegen-units`、`incremental` 敏感性，问题 3 不在我这条腿，列在这里供主 agent 取舍；若范围声明闸拒了，照它的拒绝说明改）：

5. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf; 范围=1 个测试目标; 为什么=核改法 7 把 codegen-units 与 incremental 移出键之前，栈深敏感的用例在有增量 256 个编译单元与无增量 16 个两种下结局是否相同，只点名一个目标' task-server/task submit -- env CARGO_PROFILE_RELEASE_INCREMENTAL=true CARGO_PROFILE_RELEASE_CODEGEN_UNITS=256 cargo test -p singlefs-verification --release --test allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf`
   - 在哪个目录起：`/home/fy5090/code/singlefs`（不给 `--cwd` 取此刻目录）。
   - 看哪几行：`test result:` 一行是 ok 还是 FAILED。
6. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf; 范围=1 个测试目标; 为什么=上一条的对照臂，缺省 release 档，同一个目标' task-server/task submit -- cargo test -p singlefs-verification --release --test allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf`
   - 在哪个目录起：`/home/fy5090/code/singlefs`。
   - 看哪几行：同上；两臂结局不同才是反例，相同只算这一个用例上推不出反例，不外推。

没有模型、没有 `tools/rerun.sh`，所以没有复跑命令与 sha256。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 问题 1 改法 1 | 冲突（要补条件） | 目录名集合只罩 `rust_lex` 与帮手不够，`load_rust_lex`、`store_rust_lex` 改了旧缓存照取，有反例；与现有格 `EXCLUDED_FILE_UNSET_CELL` 相反 |
| 问题 1 改法 2 | 一致（要补条件） | 只看 `Load` 会漏 `AugAssign`；今天闭包里没有真读，推不出反例 |
| 问题 1 改法 3 | 一致（要补条件） | 搬走的名字闭包里没人引，条件是按「谁读」分类；`broad-mutation.py` 不在这份闭包里；`<module-body>` 缩不了 |
| 问题 1 改法 4 | 冲突（范围漏洞） | 输出进键所以出键成立，但摘要输入集要自证；登记文件还有 python 一条路没被点到 |
| 问题 1 改法 5 | 冲突 | 允许名单极性不安全，建议取反；对 crates 变异那 7986 行没有作用 |
| 问题 1 改法 6 | 一致（要补条件） | `verdict` 不经记账改动；记录策略是「怎么存」，保护换成自证 |
| 问题 1 改法 7 | 冲突（`codegen-units`） | `debug`、`incremental` 推不出反例（`incremental` 在 `[build]` 一侧已有先例）；`codegen-units` 与有增量时的缺省 256 有栈深与优化器暴露面；变异门禁自己的「调试信息」一组自证与它相反 |
| 问题 1 改法 8 | 冲突（写法） | 「只留数」让三个读者拒；指纹取解析出的数会漏「理由空」；取第一条非注释行原文才对 |
| 问题 2 | 冲突 | 原句今天就说多了（`lib_verdict_library.py` 没单元进）也说少了（整份字节与文档串也进）；改写后整句已给；对「键过窄」条件是自证齐 |
| 问题 4（漏登） | 规则没说 | 漏登 = 少收，建议极性取反；`13-验证路线.md:432`、`:433` 对此只有总则 |
| 问题 5（八行表） | 规则没说 | 八行自证格与弄坏开关已给，开关名是描述性新名字 |
| 问题 6（前半） | 冲突 | 有同类的宽：`<module-body>` 单元、整模块载入（166 个单元）、143 个调度与报数常量、174 个生产区常量、13 份清单整份字节 |

## 没做什么

- 不判别的腿的格（问题 3 与问题 6 后半）；不替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数都没复跑，上面所有「推」的都等主 agent 的命令；改法 7 的可观测行为（问题 3）只列了一对可选命令，没有做。
- 没跑任何项目代码，没写模型，没有 `tools/rerun.sh`；任务 16712 的日志读的是 `task-server/task log 16712 --all` 的输出副本，不是我重新算的。
- 改法 5 的「哪些词真有读者」只点到一处（`gate-overlap.py` 的 `hook-events`），其余词没逐个核读法。
- `lib_function_units.py` 的 `<module-body>` 里有没有自证区语句、13 份清单谁读，没核。

## 落盘后改过什么

报告第一次落盘后、交回前，在原处改了五处（核对用，不影响结论）：改法 4 末尾一句的指代；改法 5 里 `declared_checks` 的行号写成 `14938-14940`；问题 6 第 5 点的清单份数改成 13 份并删掉「17 行」一句；「要主 agent 跑的命令」引言里的份数；冲突并排第 4 条里把日志行的起止写成文字（引文检查把「路径:起-止」当文件行号读）。

## 其余结论的推翻条件

- 问题 2（原句说多也说少、改写后整句）：若命令 1 到 4 的 `--explain` 输出显示 `lib_verdict_library.py` 的单元经别的路进了闭包，或整份字节那两个单元不是出自 `rust_lex_cache_namespace` 那一处读，则「说多了」那一点要改；若改写后的整句里「被这些函数读到的模块级语句」在实现里分不出（分析器没有按「谁读」切模块级语句的能力），则句子要退回「模块级语句随模块进」并单列不进的清单。
- 问题 4、5（极性取反、八行自证格）：若仓里查得出一条已经存在的、读法不是字面模式的读者，那么两条路都扫不到，要换成登记时让读者自己登记的办法；若某一格的弄坏开关打开后该格没红，那一行自证作废。
- 问题 6（同类的宽）：每一条的数字来自任务 16712 的日志，来路是读代码推的；命令 1 到 4 的输出若显示这些单元的来路不是我推的那一处，对应一条作废。
