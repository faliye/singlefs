# tooling-broad-mutation-prune-r1：云端正推腿（Sonnet）报告，Q1 与 Q3（2026-10-01）

立场：正推，推 P1–P5 成立需要的前提与代价。方案按 `crates/` 与 `research/scripts/broad-mutation.py` 今天的实现谈。
没读禁读清单里的任何东西。没起广谱驱动、没跑 cargo、没跑任何测试；下面的数都来自结果表、清单、基线表、日志与 git log，算术在 `research/prompts/tooling-broad-mutation-prune-r1-sonnet-model/cost-model.py`（输出同目录 `cost-model.out`）。

## 0. 结论速览

1. P1–P5 推不出「复用的结论等于在这一版上重跑的结论」。缺口有三个推不出的前提（F3 覆盖只看没变异的那一遍、F4 测试确定性、F6 测试读的文件有一部分在导出树之外），另有四处今天实现与规则的写法对不上（F1、F2、F8、F10）。逐条见第 1 节表。
2. 按「推不出就补规则或一律重判」：抓到（caught）是存在性命题，只要上次抓到它的那条测试在新版变异树上仍红就成立，不需要 F3；所以抓到可以从「复用」改成「只重跑那一条测试确认」，等式因此推得出（仍受 F4 约束）。代价与好处见第 2 节。
3. 耗时：以文件 hash 为键，被改文件名下的条目一定重判，这一项就决定了量级。最近 23 个动过 `crates/` 的非合并提交里 15 个动了清单里的文件，每个提交自己名下的条数中位占清单 62.2%（`cost-model.out:32`）。照今天每条的耗时，中位提交单驱动要 28.5–55.4 小时，不是「变轻到每次提交都过」的量级；要变轻得动键的粒度（文件改成条目所在函数）或加测试选择，这两样都不在 P1–P5 里。
4. 覆盖那一遍的下限 23.8 分钟（基线各步相加），插桩倍数没量过；这一版第二段 checker 档基线是红的（`baseline.tsv:8`），原因是一条测试读导出树之外的文件，红着就没有「全部测试的覆盖」。
5. Q3：P5 会被别的原因误红的来源有六个（零覆盖先于编译判、覆盖看不见编译期求值、超时阈值、只过第一段的读法、第二段基线红、等价变异数未知），见第 3 节。

## 1. Q1 前提

### 1.1 要证的命题与三类结局各自的形状

记一条变异 m、版本 V，判得的结局 v(m, V)。要证：用 P1 的键判「可复用」时，v(m, V旧) = v(m, V新)。三类结局的逻辑形状不同，前提也不同：

| 结局 | 命题形状 | 在新版上要成立的是什么 |
|---|---|---|
| 抓到 | 存在：至少一条测试在变异树上红 | 只要有一条红就成立；上次那条 T 仍红就够 |
| 没红 | 全称：两段里每条测试在变异树上都绿 | 要对整套测试成立；任何一条测试变了、新增了都可能翻 |
| 无效 | 编译层：变异树编不过 | 依赖整个包的类型环境，不只是被改文件与执行到的文件 |

驱动今天的记法对得上这三种形状：无效在编译那一步判（`research/scripts/broad-mutation.py:1466` 「if log_shows_compile_error(log_path) or break_is("unviable-by-exit-code"):」），抓到取日志里第一条失败行（`research/scripts/broad-mutation.py:1389` 「return found.group(1)」），没红是两段都绿后才记。

### 1.2 前提表

「状态」列：今天 = 按 `crates/` 与驱动今天的样子；推得出 = 从 P1–P5 加今天的代码能推出；推不出 = 缺一个前提。

| # | 依赖的前提 | 来自 | 今天的状态 | 推得出 | 推翻它的现象 |
|---|---|---|---|---|---|
| F1 | 覆盖那一遍跑的测试集与起法，和判变异时每一步一模一样（每一步、排除表、`--run-ignored only`、debug 与 release 各一份） | P2 | 判变异的一共七步：`one-build`、`one-test`、`two-build-harness`、`two-test-doc`、`two-test-slow-harness`、`two-build-checker-tier`、`two-test-checker-tier`（`baseline.tsv:1-8`）；checker 档那一步是 release（`research/scripts/broad-mutation.py:1191` 「checker_tier = ["--release", "-p", CHECKER_TIER_PACKAGE, "--lib", "--tests"]」）；本机没有覆盖工具（`rustup component list --installed` 无 llvm-tools，`~/.cargo/bin` 无 llvm-cov） | 推不出：今天没有任何覆盖产物；文档测试那一步覆盖要不要 nightly，我没在本机核（外部事实，未核实） | 覆盖遍里的某一步少跑或多跑了，某条判变异时红的测试在表里找不到 |
| F2 | 「测试 → 执行到的文件」要一条测试一份 profile | P2 | 第一段与耗时用例那一步是 nextest，一测试一进程；checker 档那一步是 `cargo test`，一个二进制里多条用例同进程（`research/scripts/broad-mutation.py:1212` 「("two-test-checker-tier", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *checker_tier], CARGO_FAILED_EXIT),」）；文档测试每条自成一个可执行文件（外部事实，未核实） | 第一段与耗时用例那步推得出（按进程取）；checker 档只能到二进制粒度、文档测试要另想（未核实）。nextest 能不能给每个测试进程一个独立的 `LLVM_PROFILE_FILE`，我没核（要看 nextest 0.9.146 的环境变量与包装脚本文档） | checker 档一个二进制里两条用例各抓各的变异，表里只有二进制一格 |
| F3 | 变异后的那一趟执行到的文件 ⊆ 没变异的那一趟执行到的文件（覆盖的是前者，依赖的是后者） | P2 | 推不出，见 1.3 | 推不出 | 一条变异让 T 走进基线没执行过的文件 G，改 G 之后 T 的结局变了 |
| F4 | 每条测试在同一份代码上结局不变（确定性）：不看机器核数、负载、环境变量、临时目录 | P1 全局项只列了工具链与 Cargo 文件；P3 把环境变量写成「一变就整版重判」 | 随机历史用固定种子段，但工作线程数取 `available_parallelism`（`crates/singlefs-harness/tests/random_histories.rs:113` 「fn worker_threads_by_default() -> usize {」）；故障注入线程数同样取环境变量或核数；`entries_after_mount_refuse_swapped_or_behind_devices.rs` 读 `SINGLEFS_THREAD_CAP`（`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:542` 「let thread_count = std::env::var("SINGLEFS_THREAD_CAP")」）；驱动起每一步经 `capped.sh <线程数>` 设这个变量，线程数是驱动参数，不进指纹 | 推不出：线程数不同时结局是否一样没有证据；同一条变异在同一台机器重复判的结果表里没有（只有第 3 条重复过一次，两行都是抓到，`cost-model.out:1`） | 同一条变异、同一份树、线程数不同，结局或抓到它的测试不同 |
| F5 | 测试之间不共享状态、与执行顺序无关 | P2、P3 没写 | 第一段与耗时用例那步：nextest 一测试一进程，进程内静态量不跨测试；文件系统上的临时路径带进程号与计数器（`crates/singlefs-harness/tests/common/mod.rs:25` 「static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);」），驱动每一步开跑前清空 TMPDIR。checker 档那步 `cargo test` 同进程多线程，计数器与目录名跨用例共享 | 第一段与耗时用例那步推得出（今天的代码、没发现跨进程共享量）；checker 档那步推不出 | 同一个二进制里换用例次序或加 `--test-threads=1`，结局不同 |
| F6 | 测试运行时读的所有文件都在键里 | P3 末条「后五样倾向一变就整版重判，或并进关联文件」 | 三种实例。一，litmus：`publish_order_matches_litmus.rs` 读 `litmus/` 下的文件（`crates/singlefs-harness/tests/publish_order_matches_litmus.rs:12` 「let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))」），这些文件在导出树里（`manifest.tsv` 里 `litmus/` 有 9 行）。二，`.rs` 文件被当文本读：`model_comparison.rs` 的测试 `include_str!("model.rs")`（`crates/singlefs-harness/src/model_comparison.rs:1281` 「let source = include_str!("model.rs");」），覆盖只记执行过的行，不记这个文件被读。三，读导出树之外的文件：checker 档的一条测试读 `.claude/gate.d/stage-inputs.tsv`（`crates/singlefs-checker-tier/src/crash_identity.rs:1324` 「std::fs::read_to_string(root.join(".claude/gate.d/stage-inputs.tsv")).expect("登记表");」），导出树只有 `crates/`、`litmus/`、`.cargo/`、`Cargo.*`，于是这一版第二段 checker 档基线就是红的（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/baseline.two-test-checker-tier.log:62` 「登记表: Os { code: 2, kind: NotFound, message: "No such file or directory" }」；`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/baseline.tsv:8` 「two-test-checker-tier exit-101 47.3」）。另有读宿主 sysfs 的探测（`crates/singlefs-core/src/block_device.rs:362` 「let text = std::fs::read_to_string(」），结局随宿主 | 一、二能并进键（一变就重判）；三推不出：导出树之外的文件进不了 manifest，也进不了指纹；宿主 sysfs 同理 | 改了 `litmus/` 或 `model.rs` 而 `.rs` 覆盖集没变，T 的结局变了；或在另一台机器上同一版同一条变异结局不同 |
| F7 | 编译层的依赖：变异树能不能编过，取决于整个包的类型环境 | P2「无效每次重判」 | 已经按「每次重判」处理（0.6 秒中位）。无效重判一遍的总量见第 2 节 | 推得出（只要无效不复用） | 某次提交只改了一个没执行到的文件，一条抓到的变异变成了无效或反过来 |
| F8 | 「一条测试都没覆盖到那一行」蕴含「一条测试都抓不到它」 | P2 | 不蕴含，两处反例都在今天的代码里：编译期求值不记覆盖（`crates/singlefs-core/src/root_ring.rs:44` 「pub const AT_MAKE_FILESYSTEM: Self = Self::at_make_filesystem_checked_at_compile_time(」，那个 `const fn` 只被这个常量调用）；无效变异也没有覆盖（编不过就没有运行）。清单 5 091 条里，最近的 `fn` 头带 `const` 的有 116 条（我用「往回找最近的 fn 头」粗扫的，嵌套函数与闭包会错归，只作量级） | 推不出（要先判编译，再看覆盖，见 3.1） | 零覆盖行的变异在真跑里被抓到，或判无效 |
| F9 | P3 说的宏、`include_*`、`cfg`、feature 在被变异的包里有没有 | P3 | 被变异的两个包（core、checker）里 `macro_rules!` 零处、`#[inline` 零处（`grep -rn` 数过）；非测试的 `cfg` 只有 core 里两处按 `target_arch` 的（`crates/singlefs-core/src/block_device.rs:335` 「#[cfg(target_arch = "x86_64")]」）；features 只有 checker 档包有（`verdict-store`、`gpu`，默认不开）；`build.rs` 零个。`include_str!` 一处（F6 二） | 宏与内联：今天空集，推得出「不用管」，但没有东西在以后出现时拦它 | 被变异的包里新出现 `macro_rules!`、`#[inline]`、`build.rs` 或默认开启的 feature |
| F10 | P1 的全局项覆盖了判定的全部非源码输入 | P1 | 指纹只含树里逐文件 sha256 与 `rustc -Vv`（`research/scripts/broad-mutation.py:540` 「def fingerprint_of_manifest(manifest):」；`research/scripts/broad-mutation.py:583` 「return hashlib.sha256(f"{source_fingerprint}\n{toolchain}".encode()).hexdigest()」）。判定还依赖：`cargo-nextest` 版本（本机 0.9.146）、cargo-mutants 版本（决定清单）、驱动的步骤表（`step_table`）、排除表 `research/scripts/broad-mutation-ignored-exclusions.tsv`、`--threads`、超时地板与倍数（`DEFAULT_TIMEOUT_MULTIPLIER = 5`）。这几样都不在指纹里；改了排除表或步骤表，旧结论照旧被认 | 推不出，要补进全局键 | 只改排除表（多排一条用例）而源码不变，旧版「抓到」里有一条原来靠被排掉的用例 |
| F11 | 「抓到它的第一条测试 T」是确定的一条 | P2 抓到条 | 驱动取日志里第一条失败行（`research/scripts/broad-mutation.py:1389` 「return found.group(1)」）；并行跑，谁先红由调度定；同一变异两次可得不同的 T | 推得出的方向是保守的（T 变了就多重判一次，不会少判）；但键里要存「所有失败过的测试」才有意义，不能只存第一条 | 同一变异重判，T 不同而结论照旧——说明键该是集合 |
| F12 | 清单项的身份稳定：「文件、行列范围、替换文」 | P1 | 同一文件里上方加一行，下面所有条的行号全变；但被改文件的 sha 也变了，按 P1 本来就要重判，所以身份漂移不造成误复用；cargo-mutants 加了新算子时，清单多出来的条没有旧结论，当新条判 | 推得出 | 文件没变而同一变异在清单里序号或行列变了 |

### 1.3 F3 为什么推不出（P2 最核心的一步）

P2 抓到条的复用条件是：被改文件、T 所在文件、T 在没变异的树上执行到的全部文件都没变。要让它蕴含「T 在新版变异树上仍红」，需要「变异树上 T 执行到的文件 ⊆ 没变异树上 T 执行到的文件」。这个包含不成立：变异改变控制流，T 在变异树上可能走进基线从没执行过的错误处理或回退分支，那里的代码住在别的文件 G。G 改了、T 的红绿可能随之变。覆盖表是基线那一遍的产物，看不见变异树上才走到的 G。

两点收窄（推的，没造反例；造反例是攻方腿 Q2 的活）：
- 要翻面，G 要在 T 的失败路径上起作用（吞掉错误、改写断言读到的值）。这需要 G 的改动恰好落在变异引出的那条冷路径上，概率低，但「低」不是「推出」。
- 单步就能关死的版本：抓到本身是存在命题，用 T 在新版变异树上再跑一次就是证明，不需要 F3。代价是每条抓到的变异付一次编译加一条测试，不付整套测试。

### 1.4 小结：P1–P5 推得出什么

| 命题 | 推得出吗 | 要补什么 |
|---|---|---|
| 无效条每次重判，等于重跑 | 推得出（F7，驱动今天就是每次重编） | 无 |
| 抓到条复用等于重跑 | 推不出：F3、F4、F6 都参与；F1、F2 没有覆盖产物就谈不上 | 二选一：A 把「T 在新版变异树上仍红」做成复用的验证步（等式不再依赖 F3，仍依赖 F4）；B 关联文件取整个包（F3 关死，但见第 2 节：最近的提交几乎都动了 core，复用收益归零）。 |
| 没红条复用等于重跑 | 推不出：全称命题依赖整套测试在新版变异树上都绿，F3、F4、F5 都参与；而且 P5 下没红本身就是红，只在开发回路里有用 | 开发回路里可以复用（每条省第二段的 1 100 秒以上，见第 3 节 R4），提交那一遍必须在这一版上现判，不复用 |
| 零覆盖直接记没红 | 推不出（F8） | 先判编译（0.6 秒），再看覆盖；const fn 里的条目、被变异的函数只被常量初始化器调用的，归「一律重判」 |
| 键里的全局项够不够 | 推不出（F10） | 把 nextest 与 cargo-mutants 版本、步骤表、排除表、超时参数并进全局键 |
| 文件读取类（F6） | 一、二推得出；三推不出 | 一、二并进键；三是导出树的范围问题：要么把 stage-inputs.tsv 这类文件并进导出范围并进指纹，要么这类测试不进判变异的步骤 |

补规则候选只是推出来的条件，不是采纳：采纳由主 agent 判。

### 1.5 对 P4 的前提（正推看它能不能自证会红）

P4 的八个样本里，「新测试覆盖到那一行」「工具链变了」「被改文件变了」是键的直接比较，写得出会红形态。「测试执行到的另一个文件变了」「常量定义文件变了」「宏定义文件变了」「测试读的数据文件变了」这四个样本，要在一份真有覆盖产物的版本上才造得出（没有覆盖产物就没有「执行到的另一个文件」）；宏定义文件今天零个（F9），这个样本没有真实对象，只能造一个假的。样本能不能抓到 F3 这类缺口：F3 的缺口是「覆盖表漏掉变异树上才走到的文件」，造这样的样本要先有一条真实的反例（攻方腿 Q2 的活），P4 的八项里没有对应的一项。

## 2. Q1 耗时

### 2.1 采样的局限（先于数）

已判的 303 行是 302 个不同序号（第 3 条重复了一行，`cost-model.out:1`），不是 303 条。它们来自清单序号 1–588 加第 3331 条，只落在五个文件：`allocator.rs` 162、`allocation_record_tree.rs` 74、`admission.rs` 60、`address.rs` 5、`checker` 的 `lib.rs` 1（`cost-model.out:2`）。清单共 30 个文件、5 091 条，最大的文件（`crates/singlefs-checker/src/walk.rs`）1 157 条、一个也没判过。所以下面把采样的结局占比与每条耗时外推到整张清单，只是量级，不是估计值。

| 结局 | 条数 | 占比 | 均值秒 | 中位秒 | 合计秒 |
|---|---|---|---|---|---|
| 无效 | 39 | 0.129 | 0.9 | 0.6 | 33 |
| 抓到（去重后） | 241 | 0.798 | 72.4 | 19.0 | 17 460 |
| 只过了第一段 | 15 | 0.050 | 648.7 | 647.1 | 9 730 |
| 超时 | 7 | 0.023 | 1 510.6 | 1 545.5 | 10 574 |

抓到那一行的均值 72.4 与中位 19.0 差得远：编译均值 20.4、测试均值 52.0（中位 1.0）；测试超过 10 秒的 60 条占测试总秒的 98.9%（`cost-model.out:10`）。第一条失败的测试在 core 自己的单元测试里的 185 条，测试均值 17.0 秒；在 harness 集成测试里的 55 条，测试均值 170.7 秒（`cost-model.out:11-12`）。慢的一部分原因在位置：第一条红的测试在调度里排得靠后，要先等前面的跑完（调度次序的规则我没核；例：第 422 条，抓到它的 `unreadable_abandoned_root_slot` 是 776 条里的第 769 条，整趟 885.559 秒，`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/422.one-test.log:1114` 「Summary [ 885.559s] 776 tests run: 775 passed (176 slow), 1 failed, 64 skipped」）。

### 2.2 可提交状态里没有慢档

P5 之下，可提交的版本里每条都是抓到、无效或已登记的等价排除，没有「只过了第一段」、超时、没红。采样里这两个慢档占采样总秒数的 53.7%，外推到整张清单是 371 条、约 95 小时（`cost-model.out:8-9`）。这些条要先被消掉（补测试，或登记等价排除），第一次全量才走到稳态。所以下面的每次提交耗时用「稳态」的每条均值：只含抓到与无效，62.5 秒（`cost-model.out:8`）；另给一个「先只重跑上次抓到它的那条测试」的粗估 31.9 秒（`cost-model.out:13`，粗估的算法写在那一行里：core 类取编译均值加测试中位，harness 类取编译均值加 47.7 秒当该测试自身时长的替身；T 自身时长要实测，见 2.5）。

### 2.3 每次提交的耗时

算式（`cost-model.py` 里的 `scenario`）：`C0 + 被改文件名下条数 × 每条秒数 + 其余条数 × 无效占比 × 无效均值`。三项分别是：覆盖那一遍的下限（2.4）；P1「被改文件的 sha256」决定的必重判条；P2「无效每次重判」的 563 秒（657 条 × 0.86 秒，`cost-model.out:16`）。上界另加「被改的是 core 或 checker，就把整个包都重判」（关联文件取包的最坏）。

| 改几个文件 | 选法 | 自己名下条数 | 整套重判 下界–上界（小时） | 先跑 T 下界–上界（小时） |
|---|---|---|---|---|
| 1 | 最小的（`address.rs`） | 5 | 0.6–58.2 | 0.6–29.9 |
| 1 | 居中的 | 122 | 2.7–58.2 | 1.6–29.9 |
| 1 | 最大的（`walk.rs`） | 1 157 | 20.6–31.1 | 10.8–16.1 |
| 5 | 最小的五个 | 83 | 2.0–58.2 | 1.3–29.9 |
| 5 | 居中的五个 | 577 | 10.5–58.2 | 5.6–29.9 |
| 5 | 最大的五个 | 2 778 | 48.7–88.7 | 25.0–45.4 |
| 20 | 最小的二十个 | 1 319 | 23.4–88.7 | 12.2–45.4 |
| 20 | 居中的二十个 | 2 230 | 39.2–88.7 | 20.2–45.4 |
| 20 | 最大的二十个 | 4 813 | 83.9–88.7 | 43.0–45.4 |

（`cost-model.out:21-29`；单驱动串行；用户那两台机器各跑一半就再除二。）不复用的整张清单：稳态混合 5 091 × 62.5 秒 ≈ 88.4 小时，全混合（含慢档）≈ 177 小时（7.4 天，`cost-model.out:9`）。

下界只在「关联文件的覆盖集小到不外溢」时成立；上界是包级。这条线靠最近的提交说话：最近 150 个非合并提交里动了 `crates/` 的 23 个，其中 15 个动了清单里的文件，每个提交动 1–22 个，中位 10 个（`cost-model.out:31`）；这 15 个提交自己名下的条数占清单中位 62.2%、最大 91.7%，动了 core 或 checker 的提交整包条数占清单中位 100%（`cost-model.out:32`）。按中位自己名下 3 166 条：整套重判 55.4 小时，先跑 T 28.5 小时（`cost-model.out:33`），再加覆盖遍。仓里有一处已经量到过同一类事：`crates/singlefs-checker-tier/src/crash_identity.rs:1317` 「（按 crate 闭包取会把整个 harness 收进来，E164 W5 量到崩溃枚举节点 20 个提交全部重跑；按模块引用取就不会）。」这句话是 checker 档崩溃枚举复用的键，不是广谱变异的键，但说的是同一种事：包级的键量到过「全部重跑」。

core 的 26 个模块文件，我用 `crate::<模块>` 与 `super::<模块>` 扫了静态依赖的传递闭包：14 个模块的闭包就是全部 26 个（互相成环），其余 12 个是 6、5、5、5、5、5、5、1、1、0、0、0（`cost-model.out:35`，分布写成 `[(26, 14), (6, 1), (5, 6), (1, 2), (0, 3)]`）。扫法是粗的，只认 `crate::` 与 `super::` 两种写法，漏掉别的路径会让闭包偏小，所以这是下界：静态依赖给不出比「整个 core」更小的关联集；要比整包小只能靠覆盖，而覆盖的 F3 缺口就在这里。

### 2.4 覆盖那一遍要多久

下限 = 判变异时那七步在没变异的树上的基线秒数相加：编译 23.5 + 95.7 + 43.5 = 162.7 秒，测试 278.5 + 1.5 + 936.0 + 47.3 = 1 263.3 秒，合计 1 426.0 秒（23.8 分钟，`cost-model.out:15`；`baseline.tsv:2-8`）。这个下限有四处不是真值：
1. 没插桩；插桩倍数、profile 文件写盘与合并的时间都没量过。
2. 基线编译用的是库目录里的 target，那一趟冷热我没核（23.5 秒像是热的，推的）；覆盖遍要另一个 target 目录从头编（插桩换了编译参数，不能共用），冷编译时间没量过。
3. 第七步 `two-test-checker-tier` 在这一版是红的，47.3 秒是它红掉的时刻，不是跑完的时间（`baseline.tsv:8`）。
4. 同一套第一段测试，在第 422 条的变异树上整趟 885.559 秒（那一趟机器上有没有别的负载没有记录），基线是 278.5 秒；已判的「只过了第一段」15 条里，第一段整趟通过的测试秒数最小 136.6、中位 620.5、最大 1 101.8（`cost-model.out:34`）。基线与实际能差到三倍以上，其中有多少是负载、多少是变异让测试变慢，这张表分不开（没有同一条变异在不同负载下重复判的记录）。

要量的怎么量：在草稿目录的仓副本里，`RUSTFLAGS="-C instrument-coverage"`、独立的 `CARGO_TARGET_DIR`、`LLVM_PROFILE_FILE` 带 `%p`，把驱动 `step_table` 的七步各跑一次，每步经 `bash research/scripts/run-with-memory-cap.sh`；对照 `baseline.tsv` 同一步的秒数；装 `llvm-tools` 要多久也要记。checker 档那一步属于重型测试，只能主 agent 在提交里或用户要求时跑（`.claude/agent-common.md`「不做」一节）。每步至少跑五次取中位数再谈倍数（单次观测不算数，`.claude/singlefs-ai-sop/rules/test-discipline.md@单次观测不算数`）。

## 3. Q3 P5 的代价，与会被别的原因误红的来源

用户要的效果（新代码一行没被测试覆盖到、那一行的变异就红、提交被挡）这里不评。下面只列「红了、而原因不是测试没抓到」的来源，每条带证据与推翻它的现象。

### R1 零覆盖先于编译判，会把「无效」判成红

P2 写「一条测试都没覆盖到那一行的变异，直接记没红，不跑」。无效（编不过）的变异也没有任何测试执行到它，这条规则先于编译就会把它们判红。清单里有两条今天的实例（按代码推，没真编过）：`RootRingSlotsPerRegion::at_make_filesystem_checked_at_compile_time` 只被常量初始化器调用（`crates/singlefs-core/src/root_ring.rs:44` 「pub const AT_MAKE_FILESYSTEM: Self = Self::at_make_filesystem_checked_at_compile_time(」），类型不实现 `Default`（`crates/singlefs-core/src/root_ring.rs:25` 「也不实现 `Default`」），清单里它只有一条 `Default::default()` 替换；`PublishShape::row_publish_rewriting_instance_table_pages` 同形（`crates/singlefs-core/src/transaction.rs:3604` 「PublishShape::row_publish_rewriting_instance_table_pages(1);」，类型只 `derive(Clone, Copy, Debug, PartialEq, Eq)`，`crates/singlefs-core/src/transaction.rs:3590` 「#[derive(Clone, Copy, Debug, PartialEq, Eq)]」）。两条按今天的代码都该判无效，零覆盖规则会判红。无效占采样的 12.9%（`cost-model.out:4`），所以这不是边角。
判法：先过编译那一步（中位 0.6 秒），编不过就记无效，编得过再谈覆盖。推翻它的现象：这两条的变异树真的编得过。

### R2 覆盖看不见的几类

| 类 | 今天的实例 | 后果 |
|---|---|---|
| 编译期求值（`const fn` 在常量里被调） | R1 两条；`new_pool_file_creation_regions.rs` 的常量调 `to_device_offset`（`crates/singlefs-harness/src/new_pool_file_creation_regions.rs:127` 「SlotNumber(ROOT_RING_BASE_SLOT).to_device_offset().0」） | 被变异的 `const fn` 只在编译期执行时零覆盖，变异改了常量的值，测试读常量时才红 |
| 编译期才有结局的测试 | core 里唯一的文档测试是 `compile_fail`（`crates/singlefs-core/src/address.rs:3` 「//! ```compile_fail」） | 它没有运行时覆盖；变异只改函数体时它不受影响，但「文档测试那一步」要是不进覆盖遍，覆盖集与判变异的步骤集就不一致（F1） |
| 子进程 | 第一段所在的四个包（harness、checker、core、format）里 `Command::new`、`process::exit`、`process::abort` 都是零处（`grep -rnE … | wc -l` 数出 0）；checker 档有（`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:870` 「let output = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))」） | 第一段不受影响；checker 档那一步子进程的覆盖要靠 profile 环境变量继承，没核 |
| 行级覆盖比区域级宽 | 一行里有多个区域（`&&` 右边、闭包）时，行有计数而区域没跑到 | 只会让条目照常去跑，不会误红（推的；方向是保守的） |

按 `.claude/singlefs-ai-sop/rules/test-discipline.md` 末一节（阴性结果要能和代码没跑到分开）的精神，零覆盖判红的那一支要有一条会红的自证：往一个没被任何测试跑到的函数里塞变异，规则必须判红；反过来，往一个被 const 求值的函数里塞变异，规则不许判红。

### R3 超时阈值

阈值是 `max(超时地板 900, 倍数 5 × 这一版基线里这一步的秒数)`（`research/scripts/broad-mutation.py:1267` 「return int(max(self.arguments.timeout_floor, self.arguments.timeout_multiplier * self.baseline_seconds.get(step_name, 0)))」）。这一版 `one-test` 基线 278.5 秒，所以限时 1392 秒。
- 超时 7 条，占采样 2.3%（`cost-model.out:7`），整个清单外推约 118 条，每条至少 1 392 秒；它们不被复用（`ENVIRONMENTAL_OUTCOMES`，`research/scripts/broad-mutation.py:167`），P5 又把超时算红：在这些条被消掉之前，每次提交的尝试都要重付这个数。
- 阈值两边都不干净。抓到的最大测试秒数是 1 469.1，超过限时 1 392 秒（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:221` 「422 singlefs-core crates/singlefs-core/src/allocator.rs:607:33: replace -= with += in DeviceFreeMap::clear_isolation_of_slot caught one」）；超时的最小是 1 392.2 秒（`cost-model.out` 无此行，第 556 条）。两个范围重叠，说明分界不是由变异定的。重叠的一个解释（推的，没验）：驱动量的秒数含内存包装排队的时间，而限时从起跑算、不含排队（`research/scripts/run-with-memory-cap.sh:86` 起那段）。
- 第 102 条记着「超过 1392 秒」，实际量到 1 638.2 秒（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:61` 「timeout one 5.1 1638.2」），差 246 秒；我没查出这段差是不是排队，不下结论。
- 仓里没有 nextest 配置（`git ls-files | grep -i nextest` 零行，导出树 `manifest.tsv` 里 `nextest` 零行），所以没有单个测试的超时：一条挂住的变异只能撞整步限时。第 454 条的日志是实例：一条测试 432 秒后被 scope 的 TERM 杀掉，83 条没来得及跑（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/454.one-test.log:1535` 「warning: 83/776 tests were not run due to signal」）。
- 「挂住」与「慢」分不开。挂住的变异在效果上是被抓到（套件不可能绿）；P5 把它记红是误红还是真红，是个要用户定的岔路，不是这条腿能判的。驱动的失败行正则已经认 nextest 自己报的 `TIMEOUT`（`research/scripts/broad-mutation.py:188` 「NEXTEST_FAILURE_LINE = re.compile(r"^\s*(?:FAIL|TIMEOUT|ABORT|LEAK-FAIL|SIG[A-Z0-9]+)\s+\[[^\]]*\]\s+(?:\([^)]*\)\s+)?(\S.*?)\s*$")」），只是没人配出单测超时来产生这样的行；配法与 `slow-timeout` 的字段我没核（外部事实，未核实）。
判法候选（只是推出的条件）：超时分两种记，整步限时与单测限时各一个口径，口径写进指纹。推翻它的现象：同一条超时变异在空闲机器上重判，落在限时之内。

### R4 「只过了第一段」不是「第二段没抓到」

P5 写「只过了第一段就红」。结果表里的 15 条（采样 5.0%，整个清单外推约 253 条）是驱动按 `--skip-stage-two` 记的：`row = record(STAGE_ONE_ONLY, "one", note_text="第一段没红；第二段接着跑" if with_stage_two else "第一段没红；这一趟没跑第二段")`（`research/scripts/broad-mutation.py:1486`），备注是「第一段没红；这一趟没跑第二段」。它说的是「第二段还没跑」，不是「第二段没抓到」。把它记红，是把调度状态当成了测试结局。
这 15 条要真判完，每条还要付第二段：基线里 `two-build-harness` 95.7、`two-test-doc` 1.5、`two-test-slow-harness` 936.0、`two-build-checker-tier` 43.5、`two-test-checker-tier` 47.3（红着跑到 47.3 秒），合计 1 124.0 秒（`baseline.tsv:4-8`）；外推 253 条 × 1 124 秒 ≈ 79 小时，只是第二段、不含第一段的 647 秒中位数。
判法候选：只过了第一段的条，提交闸要求补完第二段，补不完不得记红也不得记绿，单列「没判完」。推翻它的现象：补跑第二段后这一类的结局分布与一段式没有差别。

### R5 第二段基线红：环境红与没红要分开

驱动在基线不绿时退 6（`research/scripts/broad-mutation.py:187` 「EXIT_INCONSISTENT, EXIT_USAGE, EXIT_INFRASTRUCTURE, EXIT_LOCKED, EXIT_DIRTY_TREE, EXIT_BASELINE_RED, EXIT_TOOLCHAIN, EXIT_STOPPED = 1, 2, 3, 4, 5, 6, 7, 143」），这一版 `two-test-checker-tier` 的基线就是红的（`baseline.tsv:8`），原因是 F6 三：导出树里没有 `.claude/gate.d/stage-inputs.tsv`。红着的基线下，没有任何一条变异能得到第二段的结局，这也是上面 15 条只记第一段的原因之一（备注写的是「这一趟没跑第二段」，我没有证据说是不是因为基线红，不下结论）。
P5 要求每一条都有这一版的结论，在基线红时无法满足。判法候选：提交闸先判基线，基线红记「环境红」，原因另报，不混进「有一条没红」。导出树要么把这类测试读的文件并进导出范围并进指纹，要么这类测试不进判变异的步骤，二选一。推翻它的现象：把 `stage-inputs.tsv` 拷进导出树后基线仍红。

### R6 等价变异与登记表的键

P5 末句照 `.claude/singlefs-ai-sop/rules/test-discipline.md@变异测试证明的是断言会红，不是覆盖` 第 2 条：「等价变异不算盲区，但要分开记。」「把等价性写成一条测试留档」。采样里没有没红的条（`research/prompts/tooling-broad-mutation-prune-r1-sonnet-model/cost-model.out:4-7` 四个结局里没有 missed），但采样里有 22 条（7.3%）没得出结论，最坏它们全是等价或没被盯的，整个清单外推 371 条要补测试或登记，第一次全量才能绿。这个数没有下界证据，也不能当上界（采样只来自五个文件）。
登记表的键是个今天就有的坑：变异名里带行列（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:3` 「address.rs:84:36: replace * with + in SlotNumber::to_device_offset」）。同一文件里这一行上方加一行，名字就变，登记行指不到它，条目回到没登记、红。P1 的「变异描述：文件、行列范围、替换文」同一个毛病；登记表要用不含行号的键（文件、所在函数、算子种类与出现序号），键怎么取没有试过。推翻它的现象：改一行无关的注释，登记表里的行仍然全部指得到。

### R7 不稳定的测试

没有数据：一个第一段整趟 776 条测试（`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/422.one-test.log:1114` 同一行）、加第二段各步，每一步基线只跑过一遍（`baseline.tsv` 每步一行）。单次基线绿不证明测试不抖。两个方向的后果不同：基线抖红，整次提交被挡且原因不是变异；变异那一趟里一条抖红的测试被记成抓到，是假抓到，复用会把它钉住（F4）。要量的：同一版、同一台机器，七步各跑至少五次，记每次是否绿、每步秒数。

### 3.8 代价合计（只列推得出的）

| 项 | 量 | 出处 |
|---|---|---|
| 覆盖遍，每个版本一次 | ≥ 23.8 分钟（未插桩、增量编译的下限） | 2.4 |
| 无效全部重判，每次提交 | 约 563 秒 | `cost-model.out:16` |
| 被改文件名下的条目，每次提交 | 最近提交中位 62.2% 的清单，单驱动 28.5–55.4 小时 | `cost-model.out:32-33` |
| 第一次全量要先消掉的未定条 | 采样 22 条（7.3%），外推约 371 条；超时与只过第一段各自的补法见 R3、R4 | `cost-model.out:9` |
| 基线红时 | 整次提交无结论，直到导出树与基线修好 | R5 |

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 复用等于重跑（整体） | 推不出 | F3、F4、F6 推不出；F1、F2、F8、F10 今天实现与规则写法对不上 |
| Q1 P1 键 | 推不出 | 全局项缺 nextest 与 cargo-mutants 版本、步骤表、排除表、线程数、超时参数（F10） |
| Q1 P2 抓到 | 推不出 | F3；可改成「T 在新版变异树上仍红」的存在性确认，等式就不依赖 F3 |
| Q1 P2 没红 | 推不出 | 全称命题依赖整套测试；P5 下提交时本就不复用 |
| Q1 P2 零覆盖判红 | 冲突 | 先于编译判会把无效与编译期求值判红（R1、R2） |
| Q1 P2 无效每次重判 | 一致 | 编译层依赖整个包，每次重判是对的，约 563 秒 |
| Q1 P3 常量 | 冲突 | 并进关联文件不够，零覆盖规则本身要先过编译（F8） |
| Q1 P3 宏、内联、feature、build.rs | 规则没说 | 被变异的包今天是空集，没有东西拦以后出现 |
| Q1 P3 非 `.rs` 文件 | 部分冲突 | litmus 与 `include_str!` 能并进键；导出树之外的文件不能，且已经让第二段基线红 |
| Q1 P4 | 规则没说 | 八个样本里四个要真覆盖产物才造得出，F3 没有对应样本 |
| Q1 耗时 1、5、20 个文件 | 已算 | 见 2.3 表；最近提交中位 55.4 小时整套重判、28.5 小时先跑 T |
| Q1 覆盖遍耗时 | 下限 23.8 分钟 | 插桩倍数没量，要量的写在 2.4 |
| Q3 零覆盖先于编译 | 冲突 | R1 |
| Q3 覆盖看不见的几类 | 冲突 | R2 |
| Q3 超时阈值 | 冲突 | 阈值两侧重叠，挂住与慢分不开（R3） |
| Q3 只过第一段 | 冲突 | 「没跑第二段」被读成「没抓到」（R4） |
| Q3 第二段基线红 | 冲突 | 无法满足「每条都有结论」（R5） |
| Q3 等价登记表 | 规则没说 | 键里带行号会失效（R6） |
| Q3 不稳定测试 | 没数据 | 每步只跑过一遍（R7） |

## 没做什么

- 没造反例、没在副本上真跑 cargo-mutants：那是 Q2（攻方）。F3 的「推不出」是逻辑上缺前提，不是造出了一条翻面的例子。
- 没装覆盖工具、没量插桩倍数、没量 nextest 的 per-test profile 环境变量与 `slow-timeout` 的字段、没核文档测试覆盖要不要 nightly：这些是外部事实，已逐处标「未核实」；要量的做法写在 2.4 与 R7。
- 没在副本上量「先只跑上次抓到它的那条测试」的真实秒数：31.9 秒是粗估（`cost-model.out:13`），要量的做法是在副本里对点名的几条抓到的变异，用 `cargo nextest run -p <包> --test <二进制> -E 'test(=<T>)'` 单独判，经内存包装，每条至少五次。
- 没读禁读清单里的文件；没判别、没替主 agent 采纳补规则候选；各判法候选只是推出的条件。
- 采样只覆盖五个文件，外推到整张清单只是量级；`walk.rs` 1 157 条一条没判过。
- 清单里「最近的 `fn` 头带 `const`」的 116 条是粗扫的量级，没一条条核。
