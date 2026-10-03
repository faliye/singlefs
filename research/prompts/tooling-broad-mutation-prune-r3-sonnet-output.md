# tooling-broad-mutation-prune-r3 云端正推腿（Sonnet）：U1、U3（2026-10-01）

立场：正推。从已定条款（用户 2026-10-01 的方向、C1–C6）与 `crates/`、驱动 `research/scripts/broad-mutation.py` 今天的实现推「要成立需要什么前提、要花多少」。
读了：第三轮正文 `research/prompts/_tooling-broad-mutation-prune-r3-body.md`、前两轮判决、第二轮辩方的报告与模型、库目录（只读）、`git show` 的历史源码（HEAD 是 fcc4d340）、`crates/` 工作区源码、驱动。没读禁读清单。没起驱动、没编译、没跑任何测试。
本轮模型与原样输出：`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/`（脚本 `c1_commits.py`、`c3_keys.py`、`t_changed.py`、`outside_kinds.py`、`u3.py`、`u3b.py`、`u3c.py`；`out/` 是这一次的输出；`lex2.py`、`rows.json` 拷自第二轮辩方模型）。
口径：「量的」= 库里 `results.tsv` 的 302 条已判、`baseline.tsv`、基线日志、`git` 历史的直接读数；「推的」= 在这些读数上做的算术或名字匹配的静态扫描（token 级，不是类型解析）。已判的 302 条只来自 5 个文件（allocator、allocation_record_tree、admission、address、checker 的 lib.rs），外推到整张清单只是量级。

## U1　C1–C6 推不推得出用户要的效果

用户要的效果（正文第一节）：「没改的沿用上次的结论」（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:11`，括注是「改了就测，没改就绿」）与「别处改动让没改的变异翻面这一类，靠里程碑末一次全量兜底。」（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:12`）。

### U1-1　「这次改到的变异都被重判」在 C1 的定义下成立吗

**判定：对 C1 的字面成立；「沿用的就是对的」推不出——沿用那一半每个提交都带着没被 C1 看见的改动。**

C1 的定义：「落在被改函数里的变异，加上下面两类：」（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:31`）。

1. 字面成立的部分（量的，14 个真实提交，脚本 `c1_commits.py`，第二轮辩方的 14 个，剔除整体导入的 53c9f384）：
   - 改到的条数（函数文本变、函数外面变则整文件、新函数）中位 2157，与第二轮辩方的数逐个相同：「median eff_s 2157.0 p25 198 p75 3920 min 2 max 4401」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:17`）。
   - 只按函数的条数中位 292：「median fn_s 292.0 p25 40 p75 1256 min 2 max 1812」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:19`）。**「函数外面变了就整文件」这一条把中位条数乘了 7.4 倍**；93 个（提交，文件）对里函数外面变了：「outside-changed (commit,file) pairs 93 by kind {'use': 68, 'const-static': 57, 'type': 77, 'impl': 40, 'test-mod': 18, 'other': 1}」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/outside_kinds.txt:15`，分类是 token 级近似）。
   - 所以条数的驱动量不是「改的函数数中位 38」：「median changed_fn_s 38.5 p25 6 p75 193 min 1 max 291」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:20`）。
2. 「改没改」比什么：比从属性与修饰符（`#[…]`、`pub`、`const`、`unsafe`）起到函数体末尾的文本。去空白对比、再去注释对比，14 个提交的中位条数一样（`out/c1_commits.txt:17`、`:18`，都是 2157）；两者只在 495bede6 一个提交里差 31 条（198 对 167）。第三轮正文把「去不去注释」留给 U2；从 U1 看，**不去最简单**（见 U1-3：逐字相同才能保证函数内的相对位置不变），代价是每个提交多重判几十条。
3. 函数身份键要用**限定名**（含 impl 的类型）：HEAD 清单 5091 条里，「(file, qualified) keys 1031 with >1 function start: 0 mutants in them: 0」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c3_keys.txt:2`）；只用裸函数名则「(file, bare fn name) keys 974 with >1 function start: 38 mutants in them: 217」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c3_keys.txt:3`）。第二轮的按裸名近似会把同名方法（`name`、`fmt` 这一类）混在一起。
4. 函数外面的 170 条（常量初始化式）没有函数，键与「改没改」都要另定：外面没变才沿用，键用（文件，所在项的序号，项内相对位置，替换文）（推）。

**C1 没算进来、却会让沿用的结论变的五类**（量的：14 个提交里各类出现几次；脚本 `c1_commits.py`）：

| 类 | 谁变了 | 14 个提交里出现 | 原样输出行 |
|---|---|---|---|
| 测试函数自己变了（`#[cfg(test)] mod tests` 里与 `tests/` 下） | 抓到变异的那条测试被改弱、改名、删掉 | 动了 `crates/*/tests/` 下文件的 14 个；改了清单文件里测试函数的 12 个，中位 7.5 个 | 「commits with changed tests/ files: 14 of 14」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:23`）、「median changed_test_fn 7.5 p25 1 p75 13 min 0 max 45」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:21`） |
| harness 库（测试用的模型与断言辅助） | 断言的口径变了 | 13 个 | 「commits with harness src changed: 13 of 14」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:25`） |
| format 包（不在被变异的两个包里，core 引它的常量与函数） | 被测代码依赖的常量变了 | 8 个 | 「commits with format crate changed: 8 of 14」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:24`） |
| 构建配置（`Cargo.toml`、`Cargo.lock`、`.cargo/`） | 依赖版本、特性、编译参数 | 4 个 | 「commits with cfg changed: 4 of 14」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:26`） |
| 测试读的数据文件（`litmus/`、`.claude/gate.d/stage-inputs.tsv`） | 数据变了 | 4 个 | 「commits with data changed: 4 of 14」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:27`） |
| 以上任一类 | | **14 个全部** | 「commits with ANY of (test fn / tests dir / format / harness src / cfg / data): 14 of 14」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:28`） |

还有两类这一轮没法量、只能从机制推：被改函数的**调用方**（调用方里的变异，抓它的测试可能因被调函数变了而不再走到它）、给 trait 默认方法加覆盖（新 impl 里的函数是新函数会重判，默认方法本身的变异不重判，而测试此后走的是覆盖那份）。这两类与上表同属「别处改动让没改的变异翻面」。

**抓到变异的那条测试变了——可以直接对数据量一次**（量的，`t_changed.py`）：已判的 241 条抓到里，186 条是同一个源文件里 `mod tests` 的单元测试（「T class: core-lib 185, checker-lib 1, harness 55」，`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/t_changed.txt:19`）。对 14 个提交逐个看：变异所在函数没改（C1 不重判、C3 沿用）、而抓它的那条测试在这个提交里变了：「commits where >=1 carried judged-caught mutant lost/changed its catching test: 7 of 14」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/t_changed.txt:17`）；「sum carried 1304 sum stale 120」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/t_changed.txt:18`，同一条变异在几个提交里各算一次）。
限度：测试「变了」不等于「不再抓」；样本只有 5 个文件的 241 条；「改没改」按去空白后的函数文本比。这个数只说明「抓到它的测试被改」不是罕见事件，发生在一半的提交里。

**结论（U1-1）**：
- C1 对「改到的」是闭的（三类字面成立，键用限定名）。
- 对「沿用的」不闭：14 个提交每一个都动了上表至少一类。第三轮正文把它写成「靠里程碑末一次全量兜底」，等于每个提交的绿都欠着沿用条数那么多条没重判：中位 5091 − 2157 = 2934 条。
- **补规则候选（我提的，被攻过零轮）**：结论表同时存抓到它的测试名（`results.tsv` 的 `first_failing_test` 一列今天就有）与那条测试的函数文本哈希；沿用时「那条测试变了或没了」的重判。按上面的数，这一类每个提交多重判的量级是沿用条数的 9%（120 / 1304，样本内）。其余几类（harness 库、format、配置、数据、调用方）补不起来，交里程碑末，门禁每次输出「本次动了哪几类、沿用了多少条」。
- 推翻它的现象：在副本上把上表五类改动各施加一次，沿用的结论仍都等于重跑的结论；或 14 个提交以外的提交史里这五类很少出现（这 14 个是里程碑建设期的大提交，9–22 个文件一个提交）。

### U1-2　C2 的「不改结论」证明（第二轮判决第三节）在 C2 的写法下哪里会破

证明（`research/prompts/tooling-broad-mutation-prune-r2-main-verification.md` 第 33–37 行）要三个前提：（P1）测试是确定的；（P2）覆盖遍与判定遍里这条测试面对的是同一个环境（同一份二进制除被改函数外的语义、同样的输入与环境变量、同样的进程隔离）；（P3）「执行到」按被改函数记、没有漏记。C2 的写法：「零覆盖的记没红（即红）；」（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:38`）、「常量初始化式里的变异、只在常量上下文被叫的 const fn 跑全部测试。」（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:39`）。

| 项 | 今天仓里的事实（现查） | 破不破 | 要补的规则 |
|---|---|---|---|
| 测试的不确定性 | 被变异的两个包（core、checker）里 `Instant::now`、`SystemTime`、`thread::spawn`、`rand::`、`HashMap` 命中数都是 0；harness 里 `Instant::now`、`SystemTime`、`elapsed()` 合计 21 处（读到的是 `started.elapsed().as_secs_f64()` 这类报告耗时）、`thread::scope` 切片并行（并集与线程数无关）、`HashMap` 0 处；`crates/` 里（不含 `src/bin/`）没有按 `sleep(`、`deadline`、`elapsed() <` 分支的代码；checker 档（checker-tier）里 `HashMap`、`HashSet` 合计 33 处，迭代序是否决定调到哪个被测函数没逐处核 | 第一段（harness 档）不破；第二段 checker 档有一处没核的缺口 | 覆盖遍与判定遍用同一个 `--threads` 与同一组环境变量（`SINGLEFS_THREAD_CAP`、`SINGLEFS_RANDOM_HISTORY_*` 这一类；后者在大档用例里读，`crates/singlefs-harness/tests/random_histories.rs:1642` 起）；覆盖遍跑两次，覆盖集合不同的测试记「不稳」、进每条变异的集合（推，没量） |
| 共享状态 | 第一段是 `cargo nextest`，每条测试一个进程（nextest 的设计）；测试镜像文件名带进程号与计数器：`static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);`（`crates/singlefs-core/src/block_device.rs:743`）；core 与 checker 里的 `OnceLock` 是确定内容的查表缓存（`crates/singlefs-core/src/checksum.rs:11`）；第二段 checker 档是 `cargo test`（libtest），同一个二进制里的测试在同一进程的多个线程里跑 | 第一段不破；第二段**按测试名挑会破**（同进程里一条测试的状态可能影响另一条走到哪里） | 第二段 checker 档按**测试二进制**挑（覆盖遍里这个二进制的任一条测试走到被改函数，就把整个二进制跑全）；这个粒度对共享状态是对的，因为判定遍复现了覆盖遍的进程组成（推） |
| `#[path]` 共享模块 | checker-tier 的集成测试用 `#[path = "../../singlefs-harness/tests/common/mod.rs"]`（例：`crates/singlefs-checker-tier/tests/crash_enumeration_acquisition_barrier.rs:19`）；core 与 checker 的 `src` 里 `#[path` 命中 0 处；被变异的两个包里的函数都不住在共享模块里 | 对 C2 的「函数→测试」映射不破：被变异的函数在 core 与 checker 的 rlib 里，覆盖名字不带 `..`。对 **C1 破**：共享模块自己改了，几十个测试二进制同时变，C1 看不见（并入 U1-1 的「测试函数」类） | 共享测试模块的变动单独记一类：变了就把沿用降级为待里程碑 |
| 文档测试 | 整个 `crates/` 里 `//!` 或 `///` 的代码围栏只有 core 的 `address.rs` 第 3 行与第 8 行一对（一条 `compile_fail`）；驱动里文档测试是单独一步：`("two-test-doc", "two", "test"`（`research/scripts/broad-mutation.py:1219`），基线 1.5 秒（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/u3.txt` 的 `baseline.tsv steps` 行）；稳定版没有文档测试的覆盖（第一轮判决第 8 行） | C2 写法里没说文档测试，覆盖挑不出它；今天只有一条，不是破口，是一句要补的话 | 文档测试每条变异都跑（只在第二段、第一段没红才轮到它），不挑；`compile_fail` 只依赖签名，函数体变异抓不到它，常量初始化式变异除外（推） |
| 第二段 checker 档 | 第二段这一步是 `cargo test --release -p singlefs-checker-tier --lib --tests`（`research/scripts/broad-mutation.py:1202`）；它读导出树之外的文件：`std::fs::read_to_string(root.join(".claude/gate.d/stage-inputs.tsv")).expect("登记表");`（`crates/singlefs-checker-tier/src/crash_identity.rs:1324`）；这一版基线在这一步红：「two-test-checker-tier exit-101 47.3」（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/baseline.tsv:8`） | **破**：没有这一步的绿基线，它的覆盖遍不可信；这一步红在基线的 47.3 秒，而 `cargo test` 默认在第一个失败的测试二进制之后不再跑后面的二进制（推，没在本机验），这一步该有的覆盖就不全；C2 写的「零覆盖的记没红」会把只有 checker 档测试才抓得到的变异误记成没红（误红方向，安全但会让门禁红）。另：全量崩溃枚举（标了 `#[ignore]` 的）不在第二段：「checker 档标了 #[ignore] 的全量崩溃枚举不在这里（另列第三段，驱动还没做）。」（`research/scripts/broad-mutation.py:60`），C2 的「全部测试」不含它 | 先修绿（C6 已写）；修绿之前第二段对「第一段没红的」不出结论（今天就是退 6 不跑，见 U1-4）；修绿后覆盖遍对 checker 档按二进制记（见「共享状态」行） |

两条实现层的补充（推，算术）：

- **`-E` 表达式长度**。用 nextest 的筛选表达式挑测试，每条写成 `(binary_id(=…) & test(=…))` 用 `|` 连起来：库里基线日志的 776 个测试名拼出来是 133 410 字节（脚本内联在命令里算的，输出「expr bytes for all stage-one tests: 133410 MAX_ARG_STRLEN 131072」）。Linux 单个命令行参数上限是 131 072 字节（内核常量，我没在本机试 exec）：覆盖了几乎全部测试的函数（热函数）会拼出超限的参数。补法：选中的测试超过一半时改写成「全部减去没选中的」，或按整个测试二进制写 `binary_id(=…)`。
- **只在常量上下文里被叫的 const fn**：C2 写「只在常量上下文被叫的 const fn 跑全部测试」。今天的驱动没有调用图；第二轮辩方的名字匹配只找到 2 条（另 1 条找不到调用）。用不着调用图的写法：**所有 const fn 里的变异，覆盖遍里零覆盖的跑全部测试**（`rows.json` 里 const fn 里的变异 93 条，只有零覆盖的那几条要付全量代价）。

**结论（U1-2）**：C2 的证明在第一段（harness 档，nextest 每测试一进程）成立，前提是覆盖遍与判定遍用同一环境、同一线程数；在第二段不成立，要补三句（文档测试每条都跑、checker 档按测试二进制挑、覆盖遍跑两次比对）与一个前置（checker 档基线先绿）。「共享状态」「`#[path]`」对 C2 本身不是破口。
推翻它的现象：覆盖遍里同一条测试两次的覆盖集合不同（不确定性有实锤）；或副本上有一条变异，挑出来的测试没红、全部测试里有红（这一项归攻方 U2②）。

### U1-3　C3 按描述存，变异名的行列漂了怎么对上

**判定：对得上，条件是「没改」取逐字相同；行列漂是常态，不能用变异名里的行列。**

- 漂移量（量的，`c1_commits.py`）：14 个提交里，改过的文件中函数文本没变的变异共 19361 条（同一变异跨提交各算一次），其中函数起始行变了的 11015 条：「carried-in-changed-files total 19361 shifted-start-line 11015 56.9%」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c1_commits.txt:29`；条数按 HEAD 清单里该函数的条数近似）。变异名里的 `文件:行:列` 前缀过半会变，所以「按变异名存」不行，正文写「不按行号存」是对的。
- 对得上的办法：函数文本逐字没变 ⇒ cargo-mutants 在它里面生成的变异（位置、替换文）逐条没变 ⇒ 相对于函数起点的起止位置没变。键 = （文件，限定名，相对起始行列，相对终止行列，替换文）。HEAD 清单里函数内的 4921 条：「mutants in functions 4921 distinct (fn, rel start, rel end, replacement) keys 4921 dup keys 0」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/c3_keys.txt:4`），没有重键。
- 如果「没改」取「去空白与注释」：函数内相对位置会因为空白与注释的增减而挪，上面的键就对不上，要改成「函数内第 k 条」配替换文（依赖 cargo-mutants 的生成次序稳定，没验证）。逐字相同没有这个问题；代价是只差空白与注释的函数多重判（14 个提交合计 144 条，`out/c1_commits.txt` 的 wsOnlyMut 一列加起来：1+74+32+6+31，`ws_only_mut`）。
- 载体：驱动今天的结果表只读本版：「table = read_table(self.version_file("results.tsv"), len(RESULT_COLUMNS))」（`research/scripts/broad-mutation.py:704`）、「def item_states(self, results=None):」（`research/scripts/broad-mutation.py:722`）；库目录里只剩一个版本目录（`ls versions` 只有 19c45e02…），版本目录里 `manifest.tsv` 只有逐文件的 sha256，**没有逐函数的文本哈希**。沿用要新做两样：跨版本的结论表（键如上，存结局、抓到它的测试名、测试的函数哈希、判它的提交）；每版 `--sync-tree` 时存一份逐函数哈希（不能靠 `tree.source` 里的提交号回头 `git show`：提交被改写就没了）。
- 只沿用 caught、missed、unviable 三种；`timeout` 与 `memory-cap` 是环境型结局（驱动自己区分，见 `research/scripts/broad-mutation.py` 里 `ENVIRONMENTAL_OUTCOMES`），按 C4 超时算抓到时要连同「CPU 时间限」那条记录一起存，不沿用墙钟超时。
- 冷启动：今天只有 302 条有定论，4789 条没有：「沿用的结论都在；有任何一条没红就红。」（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:44`）——第一次采纳时沿用表是空的，第一个提交门禁就等于一次全量（费用见 U3）。
- 推翻它的现象：在改过文件的真实提交里，函数文本逐字没变而生成的变异集合变了（cargo-mutants 的生成依赖函数外面的东西，比如同文件里的类型）；或键在新版清单里出现重复。

### U1-4　C4、C5、C6 各自要的前提

**判定：三条都推得出，但各缺一个今天驱动里没有的东西。**

- **C4（单条测试 CPU 时限）**：驱动今天只有「每一步」的墙钟限：`return int(max(self.arguments.timeout_floor, self.arguments.timeout_multiplier * self.baseline_seconds.get(step_name, 0)))`（`research/scripts/broad-mutation.py:1298`），经内存包装的 `RUN_WITH_MEMORY_CAP_TIME_LIMIT` 生效：`environment["RUN_WITH_MEMORY_CAP_TIME_LIMIT"] = str(time_limit)`（`research/scripts/broad-mutation.py:1364`）。驱动里 `rusage`、`cpu`、`RLIMIT`、`ulimit` 的命中是 0 处（grep 原样输出为空）；基线里每条测试的耗时只有 nextest 的 `PASS [ x s]` 墙钟。C4 要的「基线里那条测试的 CPU 时间」今天没有来源：要在基线遍里给每条测试进程记 rusage（不是多一遍，是改基线遍的起法）。多线程测试的口径：harness 的随机历史测试默认线程数 `.min(16)`（`crates/singlefs-harness/tests/random_histories.rs:116`），CPU 时间可以到墙钟的十几倍，倍数与下限要按 CPU 基线定，口径细节归攻方 U2③。
- **C5（里程碑末全量）**：能做；它只在「里程碑末」才看得见 U1-1 里那五类造成的翻面（抓到变没红），也就是翻面的发现点比造成它的提交晚整个里程碑；结论表存了「判它的提交」，只能把翻面缩到那一段提交里，不能自动指到哪个提交。
- **C6（提交门禁的红）**：「沿用的结论都在；有任何一条没红就红。」（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:44`）。两个前提今天都不具备：①沿用的结论表不存在（U1-3）；②第二段基线红：基线那一步红了驱动退 6 不跑，所以第一段没红的变异在今天得不到定论：驱动里第一段没红先落 `stage-one-only`，文件头写明它不算定论（「只跑第一段；第一段没红的记」，`research/scripts/broad-mutation.py:23`，接着写「不算」没红定论）。
- 推翻它的现象：驱动加了 CPU 时限之后，基线里同一条测试多遍的 CPU 时间离散到倍数下限罩不住（负载下 CPU 时间也会因缓存、频率变）；或第二段基线修绿之后仍退 101。

## U3　代价（量的与推的分开）

### 单价（量的）：库里已判的 302 条，第一段

`u3b.py` 把 302 条已判的实测总秒数按结局拆开（原样输出）：

```
rows 302 total measured seconds (first stage only) 37799 = 10.5 h
  unviable        n= 39 (12.9% of rows) seconds     33 (0.1% of seconds)
  caught          n=241 (79.8% of rows) seconds  17460 (46.2% of seconds)
  stage-one-only  n= 15 (5.0% of rows) seconds   9730 (25.7% of seconds)
  timeout         n=  7 (2.3% of rows) seconds  10574 (28.0% of seconds)
measured mean per mutant, first stage only: 125.2 s
```

- **第二轮辩方的稳态单价 62.4 秒（37.4 小时）只含抓到与无效，丢掉了尾巴**：占 7.3% 的条（超时与幸存）花了 53.7% 的秒数（`out/u3b.txt`）。含尾巴的实测均值是 125.2 秒，是前者的两倍。这些秒数是当时的机器负载下量的（第二轮辩方量到过同一测试耗时与基线之比 0.17–30.59），不是空闲机器的值。
- 幸存的 15 条（`stage-one-only`）连第二段都还没有跑：第二段基线秒数 `two-test-doc` 1.5 + `two-test-slow-harness` 936.0 + `two-test-checker-tier` 47.3 + 两个编译步 95.7 + 43.5 = 1124.0（量的基线秒数，`out/u3c.txt` 第一行；`two-test-checker-tier` 那一步基线是红的，47.3 是它红掉的时刻）。

### 四种读法（`u3c.py`，原样输出在 `out/u3c.txt`）

每条变异平均秒数：

| 读法 | 内容 | 秒/条 | 量的 / 推的 |
|---|---|---|---|
| (i) 今天，第一段 | 库里 302 条实测均值 | 125.2 | 量的 |
| (ii) C2 的 T-first + C4 把超时压到 L | 抓到的条：增量编译 + 抓到它的那条测试在基线里的耗时（241 条里 241 条在基线日志里找得到，均值 22.8）；超时的条：编译 + L；幸存条不省 | L=60 时 53.3；L=300 时 58.8 | 推的（C2 挑出的集合大小没覆盖产物，T-first 是它的下界；L 是参数，C4 没定倍数与下限） |
| (iii) 只算编译 | 每条一次增量编译，测试不花时间 | 18.6 | 推的（硬下界：C2 不减编译） |
| 幸存条的第二段 | 幸存条占比 × 第二段基线秒数 | 再加 55.8 | 推的（上界：C2 在第二段能挑掉多少没数） |

代进 C1 的条数（`out/u3c.txt`）：

| | C1 条数 | (i) 小时 | (ii) L=300 小时 | (iii) 小时 | 再加第二段（上界）小时 |
|---|---|---|---|---|---|
| 14 个真实提交的中位 | 2157 | 75.0 | 35.3 | 11.2 | 33.5 |
| 14 个里最小的（8186d5b5） | 2 | 0.1 | 0.0 | 0.0 | — |
| 14 个里最大的（346f5e6c） | 4401 | 153.0 | 71.9 | 22.8 | — |
| 里程碑末全量（C5，5091 条） | 5091 | 177.0 | 83.2 | 26.4 | 78.9 |

14 个提交里 C1 条数不超过 200 的有 4 个（每个至多 7.0 小时，(i) 读法）；不少于 3000 的有 7 个（「commits with C1 count >= 3000: 7 of 14」，`out/u3c.txt` 末行）。

说明：`out/u3.txt` 是更早一版算术，其中「today」列 = (i) 加第二段、「tfirst」列 = (ii) 加第二段、「floor」列 = (iii)；上面两张表以 `out/u3c.txt` 为准。

### 每个版本都要付的固定量

- **基线**（量的）：七步相加 1426.0 秒 = 0.40 小时（`u3.txt` 的 `baseline.tsv steps` 行：23.5 + 278.5 + 95.7 + 1.5 + 936.0 + 43.5 + 47.3）。第七步（checker 档）在这一版是红的，修绿后更长。
- **覆盖遍**：每条测试一个进程，进程数（量的）：第一段 776 条、耗时用例 61 条（日志头「Starting 776 tests across 93 binaries (64 tests skipped)」「Starting 61 tests across 93 binaries (779 tests skipped)」，`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/baseline.one-test.log:4` 与 `…/logs/baseline.two-test-slow-harness.log:6`）；checker 档 `#[test]` 436 条（`grep -rE '^\s*#\[test\]' crates/singlefs-checker-tier` 计数），合计约 1273 个进程。**插桩倍数没量**（本机没有覆盖工具、按定义不编译）：覆盖遍的下界是未插桩的基线 0.40 小时，倍数取 1.5、2、3 是 0.59、0.79、1.19 小时（假设，`out/u3.txt` 的 `fixed per version` 行把基线并进去：0.99、1.19、1.58 小时）。
- **覆盖后处理**（每条测试的 profraw 合并与导出）：进程数量的，每个多少秒没量；取 1、3、10 秒是 0.35、1.06、3.54 小时（假设，`out/u3.txt` 的 `post-processing` 三行）。
- **每个提交都要重做**：覆盖对的是这一版没改过的代码，代码一变就不能沿用上一版的。所以固定量是每个提交的：约 0.8–1.6 小时（基线加覆盖遍）加 0.35–3.5 小时（后处理）；这些只有倍数与单个后处理耗时两个没量的参数。
- **CPU 基线**（C4）：并进基线遍（见 U1-4），不另加一遍，改的是基线遍的起法。

### 每次提交

= 固定量（上面，约 1–5 小时，没量的两个参数）+ C1 条数 × 单价。

- 中位提交（C1 条数 2157）：(ii) 35.3 小时，加第二段上界 33.5 小时；硬下界（只算编译）11.2 小时。
- 「改的函数数中位 38」对应的条数不是 38 × 每函数条数：条数由函数外面变了的文件驱动（U1-1 第 1 点）。**真实提交里 7 个超过 3000 条**，那几个提交没有哪种读法能压到几小时：(iii) 的硬下界已经是 15.6–22.8 小时。
- 只改一两个函数、函数外面没动的提交（4 个提交 C1 条数不超过 200）：6.9 小时以内（(i)），(ii) 3.2 小时以内。
- 这 14 个是里程碑建设期的大提交，不代表以后的提交习惯；若以后提交都是只动几个函数，C1 的条数会落到几十到几百，耗时与它成正比。

### 里程碑末全量（C5）

= 固定量 + 5091 × 单价。(ii) 83.2 小时、再加第二段上界 78.9 小时；硬下界 26.4 小时（5091 × 0.871 可编 × 21.3 秒编译，`out/u3.txt` 的 `compile-only floor` 行：26.2 小时）。(i) 今天的读法 177.0 小时（第一段，量的）。
- 若采纳后第一次提交就要补齐沿用表（今天只有 302 条有定论，U1-3），第一个提交门禁的代价就是这一项。
- 样本偏差：302 条里 301 条在 core 的 4 个文件，checker 只有 1 条；checker 的 `walk.rs` 占清单 1157 条，一条没判过。外推到 5091 只是量级；单价与幸存率（5.0%）、超时率（2.3%）都可能变。
- 里程碑末可以用 `--shard 0/2`、`--shard 1/2` 分两台机器跑（`research/scripts/broad-mutation.py` 文件头「--shard <k>/<n>」，已有），按墙钟除以台数；本轮没算。

### 推翻

- 幸存条（5.0%）与超时条（2.3%）在别的文件里占比低得多：均值会落向 (ii)(iii) 之间；
- 覆盖产物量出来：C2 挑出的集合中位远小于全部测试、且幸存条的第二段能被挑掉大半：(ii) 的幸存条项（32.2 秒/条）与第二段上界会降；
- 插桩倍数大于 3 或覆盖后处理每个超过 10 秒：固定量超过 5 小时，对小提交（几十条）它成为主项。

### U3 数字的出处行（原样）

- 「measured mean per mutant, first stage only: 125.2 s」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/u3b.txt:6`）
- 「median commit hours: (i) 75.0 | (ii) L=60 31.9 L=300 35.3 | (iii) 11.2 | stage-two add-on 33.5」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/u3c.txt:4`）
- 「all 5091 hours: (i) 177.0 | (ii) L=60 75.3 L=300 83.2 | (iii) 26.4 | stage-two add-on 78.9」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/u3c.txt:5`）
- 「commits with C1 count >= 3000: 7 of 14」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/u3c.txt:22`）
- 「compile-only floor if every viable mutant compiles once: viable share 0.871 x 21.3 s x 5091 = 26.2 h」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/u3.txt:35`）
- 「post-processing at 10 s per test process, ~1273 processes: 3.54 h」（`research/prompts/tooling-broad-mutation-prune-r3-sonnet-model/out/u3.txt:43`）

## 判定一览

| 格 | 判定 | 一句话 | 什么现象推翻它 |
|---|---|---|---|
| U1-1 改到的都重判 | 对 C1 字面成立；沿用的结论不闭 | 键用限定名；函数外面变了整文件重判把中位条数乘 7.4 倍（292 到 2157）；14 个提交每个都动了测试函数、harness 库、format 包、配置、数据里至少一类，7 个提交里抓到变异的那条测试被改 | 副本上逐类施加改动后沿用结论仍等于重跑；或别的提交史里这五类罕见 |
| U1-2 C2 证明 | 第一段成立，第二段要补 | 前提是覆盖遍与判定遍同环境同线程数；文档测试每条跑；checker 档按测试二进制挑、基线先绿；`-E` 表达式热函数会超 131072 字节；const fn 零覆盖跑全部 | 同一测试两次覆盖集合不同；或挑出的测试没红而全部测试抓到（U2②） |
| U1-3 C3 对账 | 对得上，条件是逐字相同 | 改过的文件里 56.9% 的沿用变异所在函数起始行变了，不能用变异名里的行列；键（文件，限定名，相对起止，替换文）在 HEAD 清单 4921 条上无重键；今天没有跨版本表也没有逐函数哈希；冷启动 302 / 5091 | 逐字相同的函数生成的变异集合变了；新清单里键重复 |
| U1-4 C4、C5、C6 | 推得出，各缺一样 | C4 没有 CPU 基线与单条限时；C5 翻面的发现点晚整个里程碑；C6 的两个前提（沿用表、第二段基线）今天都不具备 | 加了 CPU 时限后基线 CPU 时间离散过大；第二段基线修绿后仍红 |
| U3 每次提交 | 推的：(ii) 中位提交 35.3 小时，下界 11.2 小时；固定量约 1–5 小时（两个没量的参数） | 单价必须含超时与幸存的尾巴（7.3% 的条花 53.7% 的秒）；7 个提交 C1 条数超过 3000，下界已 15.6 小时以上 | 覆盖产物量出挑出集合很小且幸存条第二段能挑掉大半；另一批提交的函数外面变得少 |
| U3 里程碑末全量 | 推的：(ii) 83.2 小时加第二段上界 78.9 小时，下界 26.4 小时；量的：今天读法第一段 177.0 小时 | 样本只来自 core 的 4 个文件，checker 包 1765 条只判了 1 条 | 同上 |

## 没做什么

- 没起驱动、没编译、没跑测试（含 nextest）、没量插桩倍数与覆盖后处理的耗时；覆盖遍、挑出的测试集大小、幸存条的第二段能省多少，都是空的，U3 里相应的数一律标推的。
- 没有造「挑出来的测试没红、全部测试抓到」的反例，也没有找 C1 漏算的写法：那是攻方 U2 的格。U1-2 只推前提与今天仓里的事实。
- 函数边界、测试函数归属、「抓到变异的测试改没改」都是 token 级静态扫描（`lex2.py`），不是类型解析；`#[cfg(test)] mod` 的识别只认 `#[cfg(test)] mod x {` 一种写法；函数外面的变更分类（`outside_kinds.py`）是近似，只用来说明「条数由哪类变更驱动」。
- 没核 checker 档里 33 处 `HashMap|HashSet` 的迭代序是否决定调到哪个被测函数。
- 没替主 agent 采纳或出判决；「补规则候选」（测试名与测试函数哈希进结论表、覆盖遍跑两次、checker 档按二进制挑、文档测试每条跑、const fn 零覆盖跑全部、`-E` 超长改写）是我推出的条件，被攻过零轮。
- 14 个提交是里程碑建设期的大提交；302 条已判只覆盖 5 个文件；两处外推都只给量级。
- 工作区的 `crates/` 与 HEAD 不同（暂存区有改动）；提交史的数一律读 `git show <提交>:<文件>`，工作区的 grep 数（`#[path`、`HashMap`、`Instant` 等）读的是工作区。两者是否一致没逐处比。
