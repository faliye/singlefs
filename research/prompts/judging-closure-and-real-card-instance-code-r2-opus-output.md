# judging-closure-and-real-card-instance-code-r2：攻方腿报告（2026-11-18）

背景 `research/prompts/_judging-closure-and-real-card-instance-code-r2-background.md`（正文 `research/prompts/_judging-closure-and-real-card-instance-code-r2-body.md`）。攻击面：第一轮的修补本身、产生结论的方法、从未被看过的地方。没写模型（这一题是代码轮，探针是仓副本上的源码改动），所以没有 `rerun.sh`。
开工核过：被判的三份文件此刻的 sha256 与开工快照 `crates-sha256.txt` 三行逐字相同；副本 `/home/fy5090/.local/share/singlefs/work/code-r2-judging-closure-attack/copy` 建完、施加探针之前，这三份加 `gpu_unit_checks.rs`、`crates/mutations.tsv` 与主工作区逐份同哈希。
所有「会红 / 照绿」的探针读数都是推的，等主 agent 跑「要主 agent 跑的命令」第 1 到 4 条；下面写成「推的，待主 agent 跑」。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 第 1 问 守护测试按文字认 panic | 推不出（算打中） | 今天三个单元真用到的认不出的写法有五样，都还走不到 panic（推的）；但与第一轮探针同义的改法换成路径调用 `Option::expect(thread.name(), …)` 就过得去守护，这一类不在实现员列的「认不出」里 |
| 第 1 问 反方向 | 不成立（没逼出过） | 三个函数体自 2026-11-07 那次提交之后没改过；误报会把人推向守护看不见的写法（推的） |
| 第 2 问 拷树薄壳 | 推不出（算打中） | `crash_identity.rs:2670` 换成 `&[]`，两种编法下都没有测试红（推的，待主 agent 跑）；实现员「带 gpu 特性那条清单测试照样红」那句不成立 |
| 第 3 问 行格式 | 成立 | 三条逐字钉的行与任务服务、变异判法、按条记库三处认法都对得上 |
| 第 3 问 两处手写打印会不会漂开 | 推不出（算打中） | `mod.rs:642` 记号后加一个冒号，三处读者都认不出、没有测试红（推的，待主 agent 跑） |
| 第 3 问 先读后建 | 成立（带一条限度） | 替身建了 0 次；它证的是「读不成时不建」，与「先读后建」在可注入的那一层等价 |
| 第 4 问 九行变异 | 成立（两处小的） | 名字与改法逐行对得上；F、G 是两处不同改法；删掉的旧行由 D 守（同一处改法）。D 行少 `--exact`；G 在 19955 是复用、不是现判 |
| 八格检查单 | 七格成立、一格推不出 | 「类型与名字」：守护测试又加了一处按 `::` 拆限定名的字符串约定（不打中，第一轮已记账的那一类） |

## 第 1 问：守护测试（`crash_identity.rs:1887`）按文字认 panic 写法

### 今天三个单元真用到、而这条测试认不出的写法

逐个读了 `crates/singlefs-verification/src/gpu_unit_checks.rs` 里三个函数体（行 1007 到 1057）：

| 写法 | 在哪 | 能不能 panic 或改调用方状态 | 今天走不走得到 |
|---|---|---|---|
| `format!` 宏展开 | `crates/singlefs-verification/src/gpu_unit_checks.rs:1051`「format!(」 | 标准库 `alloc::fmt::format` 在任一个 `Display` / `Debug` 实现交回 `Err` 时用 `expect` panic（我印象里的标准库实现，本机没装 rust-src，没核源码） | 走不到（推的）：`{detail}` 里调用方交的是整数、`&str`、`Vec` 与 wgpu 类型的派生 `Debug`，23 处调用逐个看过格式串（`grep -rn 'gpu_lifecycle_trace(\|gpu_work_trace(' crates/` 命中 23 行） |
| `format_args!`、`writeln!` 宏展开 | `crates/singlefs-verification/src/gpu_unit_checks.rs:1010`「format_args!(」；`:1030`「std::io::stderr().lock(),」 | `writeln!` 先取 stderr 的锁、再求 `gpu_lifecycle_line(…)` 的值，`detail` 的格式化在持锁时跑：会卡住，不是 panic | 走不到（推的）：今天的 `detail` 里没有取锁的格式化 |
| 调标准库函数 `std::thread::current()` | `crates/singlefs-verification/src/gpu_unit_checks.rs:1035`「&std::thread::current(),」 | 线程局部数据析构之后再调会 panic（我印象里的标准库行为，没核源码）；`gpu_lifecycle_trace` 的文档写着它在析构期间被调 | 走不到（推的）：`gpu_unit_checks.rs` 里没有 `thread_local`，卡不放在线程局部里 |
| 参数之外的静态项 | `crates/singlefs-verification/src/gpu_unit_checks.rs:1020`「static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();」 | 只定时钟起点，不改判定（实现员已列） | 用到，无害 |
| 共享借用 `card: &GpuCard`，类型里有内部可变 | `crates/singlefs-verification/src/gpu_unit_checks.rs:1007`「pub(crate) fn gpu_lifecycle_trace(card: &GpuCard」；字段 `crates/singlefs-verification/src/gpu_unit_checks.rs:323`「device_condition: std::sync::Arc<DeviceConditionOfACard>,」；那个类型 `crates/singlefs-verification/src/gpu_memory_ledger.rs:2101`「out_of_memory_reports: Mutex<Vec<OutOfMemoryReport>>,」与 `:2174`「pub fn record_an_out_of_memory(&self, what: String) {」 | 经这个参数能记一次显存不够、改卡的设备情况（判器读它），守护只看参数表里的 `&GpuCard` 字样 | 今天只读 `crates/singlefs-verification/src/gpu_unit_checks.rs:1012`「card.index, card.pci_bus_identifier」，走不到 |

结论：五样都用到了、都认不出，今天都还走不到 panic 或改状态（推的）。这条守护对这五样是空的，它的文档也只认了其中三样（宏展开、调到的函数内部、静态项），共享借用那一类认了，没点名 `GpuCard` 本身带 `Mutex`。

### 守护对与第一轮探针同义的改法是空的（打中）

认方法调用那一句要求词前面是点：`crates/singlefs-verification/src/crash_identity.rs:1817`「if before.ends_with('.')」。所以下面三种与第一轮探针同义或更重的改法过得去：

1. 路径调用：`gpu_unit_checks.rs:1055` 的 `thread.name(),` 换成 `Option::expect(thread.name(), "GPU 阶段现场那一行只在有名字的测试线程上打"),`——没名字的线程上 panic，与变异行 A 的效果相同，`expect` 前面是 `::` 不是点，不报。`Option::unwrap(x)`、`RefCell::borrow_mut(&c)` 同理。
2. 会 panic 的函数而不是宏：`gpu_work_trace` 里 `.unwrap_or_default()`（`:1027`）换成 `.unwrap_or_else(|_| std::panic::panic_any("时钟早于 1970 年"))`——`panic_any` 后面接 `(` 不接 `!`，不报。`std::process::exit`、`abort` 同样不报（不是 panic，比 panic 还重：判器进程整个没了）。
3. 共享借用改状态：`gpu_lifecycle_trace` 开头加一句 `card.device_condition.record_an_out_of_memory(String::new());`——参数表还是 `&GpuCard`，函数体里没有认的写法。

这三处一起放在副本里（`probe.patch`，sha256 `81d751ef02adeae14785a7f0ee8261aa671615545e3e1cc1524d8f38521904ec`），守护测试照绿：推的，待主 agent 跑第 1、2 条。第 2 条带 gpu 特性编，同时证这三处是编得过的 Rust。键不变不另量：三个单元在 GPU 判器那一组里不沿它收，第一轮任务 19825 已量过改其中一个键不变。

实现员列的「认不出」（`crates/singlefs-verification/src/crash_identity.rs:1810`「认不出的形态：移位」那一段）没有路径调用与会 panic 的函数这两类，所以这不是「已知的空」，是没被列出来的空。

打中之后的四句（`.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）：

- 分不分辨臂：这一轮没有臂；打中的是守护测试本身，第一轮改法 1 只给了一种写法（文字认法），没有对照臂可分。
- 被判的系统看不看得到判别它的东西：路径调用与 `panic_any` 的字就在它读的函数体里，看得到，是认法窄；共享借用那一类，`GpuCard` 有没有内部可变不在它读的参数表里，按它只读参数表的设计看不到（判别子观测不到）。
- 满足字面哪一分句：第一轮判决改法 1「登记的每一个单元的函数体里没有会 panic 的写法（`expect`、`unwrap`、`panic!`、`assert!` 一族、下标取值），参数里没有可变借用」——路径调用 `Option::expect` 是这一句点名的 `expect`，修补后仍认不出，满足的是改法 1 的前一分句；共享借用改状态不在改法 1 的字面里（它只写了可变借用），归正文第 1 问「会改调用方状态的写法」。
- 跑前给的改法在打中的格上还中不中：第一轮只给了改法 1 这一个，修补照它做了，这几格照旧中。

### 反方向：判红而不会 panic 的写法今天逼没逼出绕法

不成立（没逼出）。三个函数体（`gpu_unit_checks.rs` 行 1005 到 1057）最后一次改动是 2026-11-07 的提交 baefc645a（`git log -L1005,1057:…` 的首行）；这一批的 `git diff HEAD` 在这份文件里只碰行 2188 之后的块。修补没碰产品代码，守护落地时三个函数体里没有一处被它报出（实现员用 Python 照搬认法核过，量的；Rust 那一份在 19873、19874 里绿）。

往后会不会逼出（推的）：守护把整数算术与 `[` 一律判红（`u128` 除以非零常数也红），红句没给出路（`crates/singlefs-verification/src/crash_identity.rs:2055` 一带的断言只说「会 panic 或能改调用方的东西」）。改的人最省事的绕法正好是上面第 1、2 条那几种看不见的写法，或把那一段挪进一个没登记的小函数——后者会让 `not_following_the_listed_units_drops_only_the_listed_ones`（`crates/singlefs-verification/src/crash_identity.rs:1351`，只在带 gpu 特性时编）红，红了之后按文档最顺手的去处是登进「只经它们够得到」那一批（`crates/singlefs-verification/src/judging_units_entries.rs:1196`「pub const UNITS_REACHED_ONLY_THROUGH_THE_UNITS_THAT_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW:」），而守护只读另一批（`crates/singlefs-verification/src/crash_identity.rs:2011`「let listed_units = crate::judging_units_entries::UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW;」）。登进那一批的新函数里的 panic，守护看不到。

## 第 2 问：拷树那一条测的是 `of_the_repository`，真用的是薄壳 `new`

### 交错清单：给出一处改坏、今天没有测试红（打中，推的）

改法：`crates/singlefs-verification/src/crash_identity.rs:2670`「&data_files_outside_the_crates_the_build_read(),」换成 `&[],`（薄壳交空清单）。

为什么照绿（推的）：

- 不带特性：编进去的清单本来就空（`the_build_without_the_gpu_feature_reads_no_file_outside_the_crates` 守着），交不交都一样。
- 带 gpu 特性：清单里是 prune-server 库的文件。拷树那一步先整目录拷了 path 依赖收进来的 crate 目录（`crates/singlefs-verification/src/crash_identity.rs:2706`「for crate_directory in crate_directories_outside_the_crates_directory(repository) {」，那个集合钉成 `crates/singlefs-verification/src/crash_identity.rs:2645`「BTreeSet::from([PathBuf::from("prune-server")]),」），清单里那几份已经在拷贝里了；比摘要那一步读拷贝（`crates/singlefs-verification/src/crash_identity.rs:2735`「&|path| std::fs::read(self.root.join(path)).ok(),」）读得到，`the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree` 照绿。
- 带 gpu 的清单测试读的是 `data_files_outside_the_crates_the_build_read()` 本身（`crates/singlefs-verification/src/crash_identity.rs:2497`「data_files.contains(&"prune-server/src/lib.rs"),」），不经 `new`，改薄壳碰不到它。

所以实现员报告第 2 条「`new` 里交清单那一处换成空清单的改法，与改名后的那一行是同一个效果，带 gpu 特性那条清单测试照样红」不成立：D 行改的是 `:2450` 取清单的函数（`new` 与清单测试共用），薄壳那一处只有 `new` 用，两者不是同一个效果。正文「不带特性时没有东西判得出差别」只说了一半，带特性时同样判不出（推的，待主 agent 跑第 1、2 条；推翻它的读数：第 2 条里键那一条红、红句是「树在编译后变了」，那就是带 gpu 特性的清单里有一份不在 prune-server 目录里的文件）。

旁证（量的，读的是本机 `target/` 里 2026-11-17 留下的两份构建产物，不是今天这棵树编的）：带 prune-server 那一份构建表的 `watched` 行里 `crates/` 之外的是 `.claude/heavy-thread-default` 与 `prune-server/src/` 下 34 份；不带的那一份只有 `.claude/heavy-thread-default`。今天不带特性的那条测试判清单为空，说明 `.claude/heavy-thread-default` 那一行在这之后已经出了清单；构建产物是旧的，只当线索。

### 交错仓根：逐条排除

- 少取一层上级（仓根变成 `crates/`）、取成当前目录（`cargo test` 的当前目录是包目录）：`repository.join("crates")` 不在，`copy_directory` 里 `read_dir(…).expect("读得了目录")` panic，凡经 `RepositoryCopy::new` 的都红（`crates/singlefs-verification/src/crash_identity.rs:2810`「let copy = RepositoryCopy::new(label);」那一族与 `:2834`「let copy = RepositoryCopy::new("compiled-keys");」）。推的，按代码读。
- 取成另一棵真仓（例如别的副本）：编进去的摘要表是这一棵的，比摘要那一步红「树在编译后变了」。推的。
- 没找到「交错仓根而今天没有测试红」的写法。

### 要不要一条变异或用例钉住

变异行钉不住：变异行要今天红，而这一处今天在两种编法下都判不出（推的）。它的后果只在「清单里出现一份不在 path 依赖 crate 目录里的文件」那天才有；那天比摘要那一步会红，可红句是「树在编译后变了…重编再跑」（`crates/singlefs-verification/src/crash_identity.rs:2739`），重编再跑照样红，指不到薄壳。我提的改法（推的、被攻过零轮）：比摘要那一步把「拷贝里读不了」与「内容改过」分开报，读不了的写「拷树没拷到」，指到 `RepositoryCopy::new` 交的清单。这只改红句，不加用例。

## 第 3 问：真卡共用模块的四条新测试

### 逐字钉整行的三条，与读这一行的各方是不是同一个格式：成立

新函数拼的行：`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:751`「format!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}")」。三处读者：

| 读者 | 认法 | 对新函数拼的行 |
|---|---|---|
| 任务服务 | `task-server/task_server.py:1124`「if line.startswith(GPU_NOT_RUN_MARK):」，记号后面 `task-server/task_server.py:1149`「if rest and not rest.startswith(b" "):」，再找 ` test=(\S+)` | 认得：行首记号、后接空格、`test=real_gpu_case` |
| 变异判法 | `scripts/mutation/crates-mutation-rows.py:1399`「NOT_RUN_LINE = re.compile(r"^\s*NOT_RUN_NO_GPU_ADAPTER test=(\S+)(?: reason=(.*))?$", re.M)」 | 认得：用例名不带空白，原因到行尾 |
| 按条记库 | `scripts/gate/lib_libtest_result_lines.py:104`「NOT_RUN_MARKER_FORM = re.compile(r"NOT_RUN_[A-Z][A-Z0-9_]*(?=\s|$)")」 | 认得 |

两条本次未跑的期望行里的原因都不含换行、不含 `NOT_RUN_`，按条记库截「到下一处 NOT_RUN_ 之前」也截不坏。注：任务服务的这一段在我开工之后被别的会话改过（多认了行首空白与 `test <名> ... ` 之后两处起点），「记号后面要接空格或行尾」那一条没变，上表引的是此刻的行号。

限度（不算打中）：三条 Rust 测试比的是字面量，没有东西把这串字面量与三处 Python 认法连起来；哪一处读者改了认法，Rust 这边照绿。三处读者各自的自证用的是同一形态的样本行（例：`scripts/gate/lib_libtest_result_lines.py:508`），只能说今天对得上。

### 另两处没改用拼行函数的打印，会不会与新函数拼的行漂开：会，没有测试红（打中，推的）

两处：`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:634`「"NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=nvidia-smi 列的计算进程里没有这个进程」与 `:642`「"NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=读不成这个进程在哪几张卡上：{reason}"」。

改法：`:642` 的记号后面加一个冒号，写成 `"NOT_RUN_NO_GPU_ADAPTER: test={test_name} reason=读不成这个进程在哪几张卡上：{reason}"`。

- 三处读者都不认这一行：任务服务要求记号后面是空格或行尾（`:1149`），变异判法要求记号后面是 ` test=`，按条记库的正则要求记号后面是空白或行尾。
- 没有测试红（推的）：管这个函数的两条测试 `a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller`、`a_process_seen_on_no_card_is_reported_not_run_to_the_caller`（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs` 的 `tests` 模块）只比交回的枚举值，不读打出的行；变异行 7937 的锚点从 `reason=` 起，记号那一截不在锚点里，锚点照样恰好命中一次。
- 后果：这条用例照样交回「本次未跑」给调用方，测试进程退 0；任务服务不推「有用例本次未跑」，按条记库把它当一条绿记下（推的，按 `scripts/gate/lib_libtest_result_lines.py` 文件头「按条记库」读）。读日志的人看到的是一趟真跑过的绿。

量法（推的，待主 agent 跑第 3、4 条）：副本里施加这一处，点名跑 `tests::a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller` 并带 `--show-output`；第 4 条在主工作区跑同一条作对照。对照里任务结局说明有「这一趟有 1 条用例本次未跑」，副本那一趟没有、而测试照样 `1 passed`，就坐实。

四句：不分辨臂（没有臂）；任务服务看得到这一行的字，是写的人把字写坏了，不是判别子观测不到；满足的是正文第 3 问「另两处没改用拼行函数的打印与新函数拼出的行会不会漂开，有没有东西守着它们相同」的后半句「没有」；第一轮改法 3 只点了薄壳打的那两行（第一轮判决「二、判决」第 5 条引的是那时的 `mod.rs:766`、`:771`），这两处不在它的射程里，修补没碰它们，所以这一格是「从未被看过的地方」。

实现员不改它们的理由「被变异行 7939、7940 锚住，改了锚点会失效」：此刻表里锚这两处的是 7937、7938（`grep -n -F '真卡取卡模块：读' crates/mutations.tsv`），实现员写的是 7939、7940（中间别处的增删挪了行，没追是哪一处）；改用拼行函数时这两行的原文一起改即可（锚点是这张表里的字，不是要保的东西），这条理由挡不住改法。

### `unreadable_gpus_of_this_host_are_judged_red_before_the_instance_is_created` 证的是不是「先读后建」：成立，带一条限度

- 替身建了几次：0 次（`crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs:218`「assert_eq!(creations.get(), 0, "读不成本机的卡时一个实例都不建");」），进程那一格也是空的。
- 在可注入的那一层，「先读后建」唯一看得见的后果就是「读不成时不建」：读成了的路径上两种次序交回的东西逐个相同。变异行 I 把两句对调，这条红（19921 抓到）。所以成立。
- 限度：判红那一句只看有没有 panic（`:217`「assert!(taking.is_err(), "交了这一台的卡而读不成本机的卡：取卡判红");」），不看 panic 的是哪一句。读 `nvidia-smi` 之前那一步（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:717`「.unwrap_or_else(|reason| panic!("服务交的卡认不成：{reason}"));」）若对 `192.0.2.1:0:16G` 也判红，这条照绿而根本没走到读卡那一步。单看这一处不会让次序反过来的改法漏过（那种改法同时让「连取两次」那条红），只是这条测试自己证不出它红在哪。我提的改法（推的、零轮）：`catch_unwind` 之后取 panic 的字，核它含「读不成本机的卡」。
- 薄壳那一层（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:803`「&VULKAN_INSTANCE_OF_THIS_TEST_PROCESS,」与同一份文件 `:809`「println!("{}", line_reporting_the_gpus_taken(test_name, &taken));」）照旧没罩住，实现员已认、列进「合入后要在真卡上量的」。具体改坏的样子补两例（推的，不要卡的测试都不红）：`:803` 换成 `&std::sync::OnceLock::new()`（每次取卡另建一个）；薄壳里调取卡之前先 `VULKAN_INSTANCE_OF_THIS_TEST_PROCESS.get_or_init(VulkanInstanceOfThisProcess::created_for_this_process);`（没交卡也建、先建后读）。

## 第 4 问：新变异行 9 行

读的是此刻 `crates/mutations.tsv` 第 8223 到 8231 行（`grep -n -F` 按名字定的位），逐行拆六段核。

| 行 | 标 | 名字说的 | 原文 → 替换文改坏的 | 一回事吗 |
|---|---|---|---|---|
| 8223 | A | 取线程名改成 expect，没名字的线程上 panic | `gpu_unit_checks.rs:1055` 的 `thread.name(),` → `.expect(…)`（原文在文件里恰好一处） | 是。它是守护唯一的判别行，正好是第一轮探针的原字；同义的路径调用写法见第 1 问，今天加成变异行会是「没红」 |
| 8224 | C | 删掉逐份拷清单的循环 | `of_the_repository` 的 `for data_file in …` 整段 → `let _not_copied = …;` | 是 |
| 8225 | D | 带 gpu 特性编时清单取成空 | `:2450`「&compiled_judging_units_keys().watched_files,」→ 加 `[..0]` | 是。改的是取清单的函数，两种编法都受影响，只在带 gpu 时看得出，名字写「带 gpu 特性编时」说的是哪一编法看得出，不算错。小处：第五段没带 `--exact`（另八行都带），今天只有一条测试名含这串，不影响判 |
| 8226 | E | 去掉行号之后配对不看指纹 | 第二次 `pair_off_units_with_the_same_compared_name_and_fingerprint` → 只按去行号名字配 | 是 |
| 8227 | F | 记号行写错记号 | `mod.rs:751` 的 `NOT_RUN_NO_GPU_ADAPTER` → `NOT_RUN` | 是。三处读者都不认 `NOT_RUN test=`（任务服务要整个记号、变异判法要整个记号、按条记库要 `NOT_RUN_` 后接大写字母） |
| 8228 | G | 没跑成的结局打成交了卡的那一行 | `mod.rs:775` 的 `not_run_line_of_the_test(test_name, reason)` → `handed_over_gpus_line_of_the_test(test_name, &[])` | 是 |
| 8229 | J | 记号行不带原因 | 同一处原文 → `not_run_line_of_the_test(test_name, &reason[..0])` | 是 |
| 8230 | H | 交了卡的那一行不写序号 | `mod.rs:760` 的 `format!("{}@{}", …)` → 只交总线地址 | 是 |
| 8231 | I | 取实例挪到读 nvidia-smi 之前 | `mod.rs:724` 到 `:727` 两句对调 | 是 |

F 与 G 是不是两处不同的改法：是。F 改拼行函数里的字（`mod.rs:751`，凡是拼这一行的都坏），G 改结局到哪个拼行函数的分派（`mod.rs:775`，拼行函数本身不动）；同一条测试抓两者，是因为它比整行。G 与 J 的原文是同一句、替换文不同，表头「一处改法（文件、原文、替换文三样）只留一行」许这样。

G 的判据来源：主 agent 给的 19955 里 G 是复用、不是这一趟现判（`task-server/task log 19955` 第 170 行原样「✓ 真卡取卡模块：没跑成的结局打成交了卡的那一行（本次未跑的记号行没打）：no_gpu_handed_over_reports_the_not_run_marker_line_with_the_test_name_and_the_reason 红了（复用）」），证红 19876、19921 也没点它。它头一次在哪一趟现判成「抓到」，我核不动（判定库在合入后验证的运行目录里），交主 agent 查。

删掉的旧行「拷树重算 漏拷带 gpu 特性编时构建读到的 crates 之外的文件」守的东西现在由哪一行守：它的改法（原文、替换文）原样落到 D 行（实现员草稿 `make_rows.py` 里 `renamed_old` 与 D 行的原文、替换文是同一份，读的是那份草稿，旧行原文我没从 git 里找到：它是这一批之前没提交的行），守的是「带 gpu 特性时清单不空」，D 照守。它名字里说的「漏拷」（活树上拷漏了）今天谁也不守：C 只在现造小仓上守拷贝循环，活树上的漏拷就是第 2 问那一处薄壳，今天判不出。

## 八格检查单

| 格 | 判定 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 实现员补丁（`code-r1-judging-closure-repair/patch/crates.patch`）里 `crash_identity.rs` 的块都在行 1578 之后，`mod tests` 从 `:930` 起；另两份在 `tests/` 下。新加的 `expect`、`panic!` 都在测试代码里：`of_the_repository` 里的「仓内相对路径拼到临时根底下，总有上一级目录」靠临时根非空，成立；真卡测试文件里的 `panic!` 是替身的绊线 |
| 类型与名字 | 推不出（算打中，不新） | 命名两道在快照上绿（`task-server/task log 19952` 末段「✓ code-source-discipline.sh：4 格里 4 格绿」，19953「✓ 命名纪律通过：查了 1064 个 .rs 文件、313259 个声明的名字」）。守护测试按 `rsplit_once("::")` 拆登记的限定名（`crates/singlefs-verification/src/crash_identity.rs:2014`），单元名格式只在字符串里约定这件事又多了一处读者；拆不对时它报一句、判红，不会放过，属第一轮已记账的那一类 |
| 不兼容旧代码与旧格式 | 成立 | 三份补丁里没有认旧版本的分支 |
| 不新建判定库 | 成立 | 新测试只建 `ExclusiveTemporaryDirectory`（目录名带进程号，`crates/singlefs-verification/src/crash_library.rs` 的 `create_in`），不存判定 |
| 所有任务多机可用 | 成立 | 守护的输入经 `include_str!` 编进来；小仓在 `std::env::temp_dir()`；真卡四条的地址用文档地址段现造 |
| 测试优先多线程 | 成立 | 每条用自己的 `OnceLock` 与临时目录；`catch_unwind` 那条只打一行 panic 到 stderr，不碰共用状态 |
| 带 --item 的命令与项 | 成立 | 19955 点 9 行、计数行「计数：抓到 9 条、没红 0 条、无效 0 条、锚点失效 0 行（共 9 条）」（该任务日志第 174 行）；D 行少 `--exact` 见上 |
| 新崩溃流多罩了什么 | 成立 | 没有新流 |

## 我提的改法（都只在读代码上推过、被攻过零轮，没有一格量过）

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 守护测试改成白名单：登记单元的函数体里出现在调用位置（后接 `(` 或 `!`）的名字，只许一张写死的名单（今天三个函数体用到的 `format`、`format_args`、`writeln`、`gpu_work_trace`、`gpu_lifecycle_line`、`get_or_init`、`elapsed`、`as_micros`、`now`、`duration_since`、`unwrap_or_default`、`stderr`、`lock`、`current`、`id`、`name`、`process` 这一类），名单外的一律报、由人判再加名单 | 第 1 问的路径调用、`panic_any`、经共享借用调方法改状态、调到新的小函数，四类一起变红；算术与下标的误报也不再逼人绕 | 推的，待主 agent 跑（要实现员写，探针副本第 1、2 条的三处改动当它的判别行） |
| 守护同时读「只经它们够得到」那一批（`judging_units_entries.rs:1196`）里是函数的那几个 | 第 1 问反方向：挪进新小函数、登进那一批之后的空 | 推的 |
| 比摘要那一步把「拷贝里读不了」与「内容改过」分开报，读不了的指到 `RepositoryCopy::new` 交的清单 | 第 2 问：薄壳交错清单那天的红句指得到病根（今天这一格判不出，不加变异行） | 推的 |
| `gpus_this_process_is_on_judged_against_the_handed_over` 的两处打印改用 `not_run_line_of_the_test`，变异行 7937、7938 的原文跟着改 | 第 3 问漂开那一格：记号只剩一处拼，F 行连带守住这两处 | 推的，待主 agent 跑第 3、4 条坐实缺口 |
| `unreadable_gpus_…` 取 panic 的字，核它含「读不成本机的卡」 | 第 3 问「先读后建」的限度：证得出红在哪一步 | 推的 |

## 没打中的形状

- 守护的注释与字面量剥法：逐个试读了生命周期 `'_`、`'a>`、字符字面量 `'"'`、`'\''`、原始标识符 `r#type`、不嵌套的块注释；剥法对这几种都不误伤函数体的花括号配对（嵌套块注释会多报，宁红不放）。
- 守护的找函数：同名 `fn` 多于一个、泛型函数、`#[cfg(test)]` 里的同名辅助都判红，不放过。
- 拷树交错仓根：少一层、当前目录、别的真仓三种都红（推的）。
- 三条逐字钉的行与三处读者：没找到格式对不上的。
- 九行变异：没找到名字与改法不一致的行。
- 取样范围：被判的三份文件逐行读了修补涉及的段；`gpu_unit_checks.rs` 读了行 985 到 1074 与 `GpuCard` 的字段；三处读者各读认记号的那一段。

## 这条腿自己的限度

- 一条命令都没跑：探针的「照绿」全是推的，等第 1 到 4 条。
- 标准库里 `format!` 与 `std::thread::current()` 的 panic 路径是我印象里的，本机没装 rust-src，没核源码。
- 拷树那一格的旁证读的是 2026-11-17 的旧构建产物，不是今天这棵树编的。
- 没读辩方腿的报告（禁读），也没复核第一轮判决本身（归辩方腿）。

## 要主 agent 跑的命令

副本 `/home/fy5090/.local/share/singlefs/work/code-r2-judging-closure-attack/copy`（`.deliverables` 登成交接；施加的改动在同目录 `probe.patch`，五处：`gpu_unit_checks.rs` 三处、`crash_identity.rs:2670` 一处、真卡模块 `:642` 一处）。四条都按「点到的目标不走崩溃枚举」写；被范围声明闸拒、点名要缩域的，照拒绝说明改。

1. 不带特性，验证包库内 2 条：守护测试与活树键测试。起点副本。看 `TASK_SERVER_RESULT` 的 `exit=0` 与日志「test result: ok. 2 passed」——两条都绿就坐实第 1 问（三处改动守护认不出）与第 2 问不带特性那一半；守护红了看红句点名的是哪一处，那一处就是我推错了。
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的库内单测里点名的 2 条测试; 范围=2 个测试; 为什么=攻方腿副本探针：守护测试认不出路径调用与共享借用改状态、拷树薄壳交空清单判不出' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/code-r2-judging-closure-attack/copy -- cargo test -p singlefs-verification --lib -- crash_identity::tests::no_listed_unit_that_only_prints_the_gpu_lifecycle_line_can_panic_or_change_what_its_caller_holds crash_identity::tests::the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree --exact
2. 带 gpu 特性，验证包库内 3 条。起点副本。看 `exit=0` 与「test result: ok. 3 passed」：坐实三处改动编得过而守护照绿、带特性时薄壳交空清单也判不出；键那一条红而红句是「树在编译后变了」，就推翻第 2 问带特性那一半。
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 带 gpu 特性的库内单测里点名的 3 条测试; 范围=3 个测试; 为什么=攻方腿副本探针：带 gpu 特性编时守护照绿、薄壳交空清单判不出' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/code-r2-judging-closure-attack/copy -- cargo test -p singlefs-verification --features gpu --lib --release -- crash_identity::tests::no_listed_unit_that_only_prints_the_gpu_lifecycle_line_can_panic_or_change_what_its_caller_holds crash_identity::tests::the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree crash_identity::tests::the_build_with_the_gpu_feature_reads_the_prune_server_library_files_outside_the_crates --exact
3. 真卡测试包，记号行漂开的那一条，带 `--show-output`。起点副本。看 `exit=0`、「test result: ok. 1 passed」、日志里那一段输出是 `NOT_RUN_NO_GPU_ADAPTER: test=…`，且任务结局说明里没有「本次未跑」一句。
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 e202_gpu_cpu_gc_comparison 里的 1 条测试; 范围=1 个测试; 为什么=攻方腿副本探针：手写的本次未跑记号行写坏而测试照绿、任务服务认不出' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/code-r2-judging-closure-attack/copy -- cargo test -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- tests::a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller --exact --show-output
4. 对照：同一条在主工作区跑。起点仓根。看 `exit=0`、「test result: ok. 1 passed」，且任务结局说明有「这一趟有 1 条用例本次未跑」——与第 3 条只差那一个冒号。
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 e202_gpu_cpu_gc_comparison 里的 1 条测试; 范围=1 个测试; 为什么=攻方腿探针的对照：没改坏的记号行任务服务认得出本次未跑' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- tests::a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller --exact --show-output

## 只读到、没攻到的

- 同一个包里另有 4 处手写的本次未跑记号行，不在被判的三份里：`crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs` 第 575、599 行，`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs` 第 8924、12415 行；漂开的风险与第 3 问那两处同形。
- 守护对「会卡住」（持 stderr 锁时格式化）这一类没有判法，正文与第一轮改法都没提，只读到。
- G 行头一次现判在哪一趟，核不动。

## 没做什么

- 不判辩方那几格（第 5 问）；不替主 agent 采纳；副本上的数不算入库装置上的数；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没写模型，没有 `rerun.sh`。

## 草稿与清理

- 留着：副本 `copy/`（57M，交接，第 1 到 3 条要用；主 agent 用完删）、`probe.patch`（补丁）、`originals/`（施加探针前的三份原文，备份），都登在草稿目录的 `.deliverables`。没有编译目录。
- 草稿里另有读过的任务日志摘录（`log-<号>.txt`）、施加探针的草稿脚本 `apply_probe_edits.py` 与 `cite-check.txt`，都是小文件。
