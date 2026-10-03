# test-script-align-code-r2 辩方腿（Sonnet）报告：J5 为主，J3、J4（2026-10-01）

批次：`records/2026-10-01-对接上游测试脚本规范.md#3`。被复核的判决：`research/prompts/test-script-align-code-r1-main-verification.md`；它依据的腿报告：`research/prompts/test-script-align-code-r1-sonnet-output.md`（正推）、`research/prompts/test-script-align-code-r1-opus-output.md`（攻方）。
读的是冻结副本 `/tmp/claude-1000/test-script-align-code-r2/tree/`（下文 `crates/…` 路径都相对它，cite-check 用 `--root` 指它）；第一轮冻结副本只拿变异表对照。没读主仓 `crates/`、没读禁读清单里的文件。
立场：替第一轮判决里被判「不采」、被判「只抽一次」的一方找它站得住的理由；对判决采的每一处改法，问它是不是同样打中了别的写法。
探针：全是只读的 grep、awk 与两份 python 静态核，没编译、没跑任何测试。`research/prompts/test-script-align-code-r2-sonnet-model/`，`bash rerun.sh` 复跑，`SHA256SUMS` 同目录。

## 一、判定一览

「站得住」：判决那一条我复核后同意，它不会被同形的别的写法推翻。「够不着」：判决那一条射程不够，同一形状还住在判决没碰的地方。「同样打中别的臂」：判决的理由对替代方案同样成立或同样不成立。

| 判决条目 | 辩方判定 | 一句话 |
|---|---|---|
| K4 `ExpandUpToWritesPerSegment` 的 `.0`，「不采」 | 站得住 | 编码纪律自己的例子就是 `pub struct LogicalAddress(pub u64)`，仓里 38 个 `pub struct X(pub …)` 同形；这个类型没有不变量 |
| K4 `ALL`，「不采」 | 站得住 | 仓里 14 处 `const ALL`，宏生成的也叫它；类型名带着它，说得出是什么 |
| K4 `a_line_was_printed_at`，「不采」 | 够不着 | 判决的理由是「名字说得出它是什么」；这个名字说的是「打印了一行」，函数实际记下的是「收到一条消息的时刻」，CPU 块那一支收消息时并没有打印；这个方法没有文档注释，名字之外读者没有地方知道 |
| K4 错误口径（`Result` 对 panic），「不采」 | 站得住（口径）；够不着（重复） | 写错就 panic 在仓里是主流（至少 8 处）；但判决没碰的是「解析一个数值环境变量」这同一件事被写了至少 9 份，判决采 C606 时只写「同一判法」没写「同一个函数」，第一轮之后又多了一份 |
| K2「只抽一次」 | 站得住（规则）；够不着（清单不全） | 规则依据对；但攻方「没打中的形状」表 10 行，判决只点了 5 行，另 4 行（驱动行全复用不叫停、挂钟进度行、GPU 非 NVIDIA 独显与集显、崩溃注入复现新发现那一类）没点名也没标「只抽一次」 |
| K2 五格我各看一遍 | 同结论 4 格；1 格缺样本 | E165 五对列、零态、ignore 全跳过、展开上限 panic 被吞，我独立读与核，得到与攻方相同的「没打中」；36 段随机历史复现缺「真红」的样本，不能当没打中 |
| 采的 C606（GPU 对拍展开上限写错退回默认） | 够不着 | 同一个形状还有 `checking_threads()` 与 `SINGLEFS_THREAD_CAP` 两处悄悄退回默认；线程数 0 在这两处是「取默认」，在新共用函数里是 panic |
| 采的 C611（挂钟进度那一臂补假时钟用例） | 够不着 | 三个发射点只有 CPU 块那一处有站点级假时钟用例；GPU 块只核了行的写法，层 0 那一臂一条用例都没有；C611 条款要的是每一处各一条 |
| 采的 K5（环境守卫） | 够不着（低险） | 守卫只用在流水线用例与公共模块；同形的 `set_var` 与 `remove_var` 还有库内两条用例与两份固定脚本用例没守 |
| 采的 C604（取卡一律经入口） | 站得住 | 仓里再没有 `.expect("本机…")` 取卡；库内计时用例、σ 对拍、量显存都经 `first_usable_card_or_report_not_run` / `usable_cards_or_report_not_run` |
| 采的 G1（PCI 判红） | 站得住 | 产品路径 `gpu_cards_in_use` 没卡一律返回错误，不退回 CPU，本来就不是「当成本次未跑」的形状 |
| 采的 S2（重放行带照哪一档断言判） | 站得住 | 坏盘输入的重放行只附在大档入口同样判的两条断言上，没有 S2 的形状 |
| K3「其余 33 行推的会红」 | 站得住 | 第一轮之后变动与新增的 82 行我静态核了一遍：80 行全过，2 行是同名用例落在两个包里，由 `-p … --test …` 分开，不算问题 |
| K3 第 1575 行红在防御断言上，「记一笔」 | 站得住 | 红在 panic 上也算红；防御断言哪天删了，下标越界照样 panic |
| J3 M3 第 1587 行的推断 | 成立，且比推断更稳 | 变异在前置断言 `assert_ne!(复用键)` 上就红，不靠「改坏 magic 后判红」那一推 |
| J3 C604 没留变异的理由 | 站得住（就变异这一形）；够不着（「留不了守法」） | 有卡机器上改回去走同一条路，变异分不出；但源码扫描的用例与硬件无关，仓里已有同形先例 |
| J4 宏挪进核心层 | 站得住（字面）；可议（放哪） | 宏做的是生成、不藏逻辑，合「重复要生成」；D17 没有话说；但宏住在讲空间准入的模块里、有三个名字，核心层自己手写 `ALL` 有先例 |
| J4 展开上限解析与 history.rs 共用函数 | 是同一件事写了两份 | `crates/singlefs-checker-tier/src/crash_amplification.rs:85` 与 `crates/singlefs-harness/src/history.rs:4308`；检查包依赖 harness，没有理由不共用；另有一个共用函数没有调用方 |

## 二、J5：第一轮判决里「可议，不采」的 K4

判决那一行：`research/prompts/test-script-align-code-r1-main-verification.md:32`，理由「名字说得出它是什么」（`research/prompts/test-script-align-code-r1-main-verification.md:32`）与「是两种调用方（入口报给人、库内不变量），不统一」（`research/prompts/test-script-align-code-r1-main-verification.md:32`）。三处名字与口径不一，我逐个替「该采」的一方辩一遍。

### 2.1　`.0` 与 `ALL`：不采站得住

- `.0`：编码纪律的取舍是「有不变量的私有，没有的直接公开」，它自己的例子就是元组 newtype：「没有不变量的字段直接公开，不写样板 getter / setter。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:133`）与「#[derive(Clone, Copy, PartialEq, Eq)] pub struct LogicalAddress(pub u64);」（`.claude/singlefs-ai-sop/rules/code-discipline.md:116`）。「pub struct ExpandUpToWritesPerSegment(pub usize);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:61`）任何 `usize` 都是合法取值，没有不变量；仓里 `pub struct X(pub …)` 的形状数出来 38 个（`outputs/j4_j5.out` 倒数第二段）。读 `.0` 要回头看定义，不是规则点名的毛病。
- `ALL`：仓里 `const ALL` 数出来 14 处（同一份输出），分布在核心层、harness、检查包的库与测试里，例如「const ALL: [Self; 3] = [」（`crates/singlefs-core/src/transaction.rs:4064`）。这是一个成体系的名字：枚举类型名带着它，`WritableMountPhase::ALL` 读出来就是「这个枚举的全部成员」。要改只能全仓一起改，一个枚举改了反而破坏「一个语义概念全仓一个名字」（`.claude/singlefs-ai-sop/rules/code-discipline.md:31`）。
- 会推翻：有人指出 `ALL` 在某处被当成「全部」之外的东西用；或 `ExpandUpToWritesPerSegment` 后来长出不变量（例如上限不许超过 28）。

### 2.2　`a_line_was_printed_at`：「不采」够不着

判决的理由是名字「说得出它是什么」。编码纪律的判据是「光看这个名字，说不说得出它是什么、做什么、不做什么」（`.claude/singlefs-ai-sop/rules/code-discipline.md:15`）；方法的名字另有一条：「类型用名词、方法用动词」（`.claude/singlefs-ai-sop/rules/code-discipline.md:218`）。

1. 它不是动词，是一句过去时的事件陈述，却是个 `&mut self` 的改状态方法：「pub fn a_line_was_printed_at(&mut self, now: std::time::Duration) {」（`crates/singlefs-checker-tier/src/crash.rs:2789`）。同一个 `impl` 里的兄弟是「pub fn wall_clock_line_is_due(&self, now: std::time::Duration) -> bool {」（`crates/singlefs-checker-tier/src/crash.rs:2785`），一个问、一个记，名字却是两种文法。
2. 更要紧的是名字断言了一件函数自己不知道的事。它在两处被调：收到一条消息之后「progress.a_line_was_printed_at(clock());」（`crates/singlefs-checker-tier/src/crash.rs:2815`），到期之后调用方打了一行再记一次。第一处记下的是**收到消息的时刻**，名字说的是**打印了一行的时刻**；两者只在调用方「收到一条就打一行」这个约定下才相等。
3. 约定在 CPU 块那一支（`crates/singlefs-checker-tier/src/crash_amplification.rs`）靠的是别处：收到一段时「ReceivedOrWallClockLineDue::Received(piece) => arrived.push(piece),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1438`）什么都不打印；那一段的行是核对线程在发消息之前打的：「"CRASH_AMPLIFICATION_BLOCK point={} block_start={} states={} red={}",」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1229`）。所以这一支里「记下一行打印的时刻」记的是收到的时刻，比真打印晚一个排队的时间。语义上不致错（只会让下一行最多晚出一点），但名字说的与做的差了这一截，「做什么、不做什么」说不出来。
4. 方法没有文档注释，约定只写在它的调用函数的文档里，看名字的人拿不到。
- 判定：这一处「不采」够不着判决给的理由；改名成「记下上一次进度行的时刻」一类（例如 `remember_the_instant_of_the_last_progress_line`）才说得出它做什么。严重度低，不改不影响任何判定。
- 会推翻：CPU 块那一支收到一段时同样打印一行（则名字说的就是实际发生的）；或调用方契约进了这个方法的文档注释。

### 2.3　错误口径：口径站得住，同一件事写了多份判决没碰

判决写「复现入口走 `Result`、展开上限走 panic 是两种调用方」。我替「要统一」一方找最强的理由，找不到，因为仓里写错就停才是主流：

- 写错（含 0）就 panic 的：「环境变量设了却不是正整数：配错了就停，不悄悄退回单线程。」（`crates/singlefs-checker-tier/src/crash.rs:1603`）、「环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程。」（`crates/singlefs-harness/src/fault_injection.rs:2591`）、`e161` 与 `e165`、`e168` 三个装置（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:2643`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7452`、`crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:601`）、随机历史用例「fn number_from_environment(name: &str, default: u64) -> u64 {」（`crates/singlefs-harness/tests/random_histories.rs:1873`）、展开上限「pub fn from_value_of_the_variable(value: Option<&str>, default_when_unset: usize) -> Self {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:85`）、第一轮之后新写的「pub fn number_from_the_variable_value_panicking_when_unreadable<Number>(」（`crates/singlefs-harness/src/history.rs:4308`）：共 8 处。
- 返回 `Result` 的只有复现入口那一族，它的调用方是把错误报给跑重放行的人：「fn usize_from_reproduction_environment(」（`crates/singlefs-checker-tier/src/crash_injection.rs:2532`）。
- 编码纪律的分界是「能恢复的失败」（`.claude/singlefs-ai-sop/rules/code-discipline.md:141`）走 `Result`，与「不变量被破坏是 bug，用断言」（`.claude/singlefs-ai-sop/rules/code-discipline.md:146`）。环境变量配错既不是 I/O 错也不是盘上数据，不在「能恢复」那一类；两种口径各有调用方，判决「不统一」站得住。

判决没碰、而我认为该记的是**重复**，不是口径：「解析一个数值环境变量、认不出就报」这同一件事，在仓里至少 9 份各写各的（上面 8 处加复现入口那一份 `Result` 版）。判决采 C606 时写的是「与 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 同一判法」（`research/prompts/test-script-align-code-r1-main-verification.md:28`），只说判法、没说同一个函数；于是实现员在 GPU 测试文件里照抄了一份，第一轮收尾又抽成 `history.rs` 的共用函数，而 `crash_amplification.rs` 那一份留着（收尾报告 `/tmp/claude-1000/impl-post-merge-code-r1/report.md` 第 5 节第 3 条自己写了）。「重复要生成，不许手抄。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:179`）管的就是这个。这是 J4 第二问的出处，放到第五节判。
- 会推翻：把这 9 份逐份核，发现各自对 0、空串、前后空白的处理确有不同的业务理由（本节只核了 panic 文案与调用方，没逐份核边角）。

## 三、J5：第一轮判决里「只抽一次」的 K2

判决那一行：`research/prompts/test-script-align-code-r1-main-verification.md:35`，去向「第二轮换攻击面再攻」（`research/prompts/test-script-align-code-r1-main-verification.md:35`）。规则依据：`.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`，其中「一次不算」（`.claude/rules/three-way-inference.md:159`）。

### 3.1　规则用对了，清单不全

规则的条件是「拿一条腿的没打中去支撑任何结论之前，至少再抽一次」。判决没拿它支撑任何结论，只是不让它当「没打中」用，这一步站得住。

但判决点名的是 5 格，攻方「没打中的形状」表有 10 行（`research/prompts/test-script-align-code-r1-opus-output.md:245` 起；我数的是表里以「|」起头的数据行）。没点名的 4 格同样只有攻方一次抽样，同样不该被当成「没打中」：驱动行全复用不叫停（「同一份库第二趟」，`research/prompts/test-script-align-code-r1-opus-output.md:249`）、挂钟进度行（`research/prompts/test-script-align-code-r1-opus-output.md:252`）、GPU 非 NVIDIA 独显与集显、崩溃注入复现新发现那一类（`research/prompts/test-script-align-code-r1-opus-output.md:255`）。判决今天没有拿它们支撑结论，不是错，但留着没标，下一轮的人会以为它们已经被两次打过。

### 3.2　我对判决点名的 5 格各独立看一遍

我是辩方、不是同立场再抽，所以下面是「第二次观测」而不是「第二个样本」：方法不同（我用静态核，攻方用副本原型），能核到的格我都用命令核，核不到的写明。

| 格 | 我看到的 | 判定 |
|---|---|---|
| E165 五对列 | 钉表 195 行，21 段，五对（账第 3、4、5、12、13 段对钉表第 2、3、4、7、8 段）逐行 0 行不同；原样输出在 `outputs/j5_e165.out`：「rows 195 nf!=21 0 pairdiff 0」。同一张对列表在 Rust 装置与 Python 脚本各写了一份（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 的 `LEDGER_COLUMNS_THAT_MUST_EQUAL_A_PINNED_COLUMN` 与 `research/scripts/e165-pin-red-examples.py` 的 `COPIED_PAIRS`），今天两份一致 | 同结论（没打中）；两份对列表是手抄，记一笔 |
| 零态 | `plan_crash_points` 末尾核「各点的序号区间加起来要正好是整条流的状态数」（`crates/singlefs-checker-tier/src/crash_amplification.rs:347`）；没有崩溃点的流这一句会 panic，造不出零态的计划。读代码，没造流跑 | 同结论，但是靠一处断言，不是靠构造；断言哪天删了这一格就开了 |
| ignore 全跳过 | 「let exhaustive = expand_up_to.covers_every_segment(flow.segments);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3571`）按展开深度判，不看 ignore；`ignored_points` 与 `states_skipped_by_ignore` 另报，入口脚本按「按登记整批不判：N 个点、M 态」（`research/scripts/crash-amplification.sh:83`）写进成功句，上游的要求是「按登记整批不判的一类不算剪枝」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:62`） | 同结论；类名仍「取不出」，脚本头自己写明，不属本格 |
| 展开上限 panic 被吞 | 读这个变量的三处：「let limit = ExpandUpToWritesPerSegment::from_environment_or(DEFAULT_EXPAND_UP_TO);」（`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs:328`）、回滚那份、公共模块「let limit = ExpandUpToWritesPerSegment::from_environment_or(default_expand_up_to);」（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:292`），都在用例主线程上，全仓 `catch_unwind` 的命中里没有它们的路径（`grep -rn catch_unwind` 那一批都在别的函数里）| 同结论 |
| 36 段随机历史复现 | 攻方自己写明 36 段「结局都是完成」，没有一段落在已知红或新发现上（`research/prompts/test-script-align-code-r1-opus-output.md:254`）。所以被比的是历史与跑法逐项相同，「复现出同一个红」这一半一次都没看到 | **缺样本，不能记没打中**；判决点它是对的 |

- 会推翻：钉表新增一行五对不等；或有一条流让 `plan_crash_points` 末尾那句断言不触发而返回零态计划；或展开上限的读取挪进了被 `catch_unwind` 包住的线程。

### 3.3　挂钟进度行：我替判决没点名的一格补一次观测

攻方的结论是「读代码，没找到「重置了计时却没打行」的路径」（`research/prompts/test-script-align-code-r1-opus-output.md:252`）。我按它说的两处各读一遍：CPU 块每一段判完在核对线程里打一行（`crates/singlefs-checker-tier/src/crash_amplification.rs:1229`）再发消息，层 0 的 `Received` 一支先 `println!` 再合并（`crates/singlefs-checker-tier/src/crash.rs:3265` 一带），所以「收到消息即重置」在这两处都不吞空档。与攻方同结论。副作用是 2.2 节说的：记下的是收到的时刻，不是打印的时刻，名字与它不符。

## 四、J5：采的改法是不是同样打中别的写法

下面每一行：判决采的那一条、同一形状我在冻结副本里还找到的落点、判定。落点都是 grep 命令数出来的，`outputs/j4_j5.out` 是原样输出。

### 4.1　C606：同一形状还有两处悄悄退回默认（够不着）

判决只点了一个变量：「改代码：写错就 panic，带变量名与原值，与 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 同一判法」（`research/prompts/test-script-align-code-r1-main-verification.md:28`）；条款是「规模变量的值认不出判红、不退回默认」（`.claude/kb/checks-owed.md:475`）。第一轮收尾把同族的三个变量（展开上限、显存额度、CPU 线程数）也改了，是按变量列的清单，不是按形状。冻结副本里同形的另外两处没动：

- 「std::env::var("SINGLEFS_CRASH_AMPLIFICATION_THREADS")」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1157`）后面接 `.ok()`、`.and_then(parse.ok())`、「.filter(|count| *count >= 1)」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1160`）、`unwrap_or_else(取本机并行度)`：写错与写 0 都悄悄退回本机并行度。入口脚本的 `--threads` 只要求带一个值（`research/scripts/crash-amplification-item.sh:100`，我没看到正整数校验），再经 `research/scripts/crash-amplification-lib.sh:129` 原样送进这个变量。
- 「let thread_count = std::env::var("SINGLEFS_THREAD_CAP")」（`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:632`）同样 `.ok().and_then(parse.ok()).unwrap_or(取默认)`。

线程数不是「规模变量」，但第一轮收尾已经把 CPU 线程数当同族改了；而且这一个仓里别处对「线程数写 0」的口径是 panic（`crates/singlefs-checker-tier/src/crash.rs:1603`、`crates/singlefs-harness/src/fault_injection.rs:2591`）。结果是同一个 crate 里线程数写 0 有两种命运：`SINGLEFS_CRASH_JUDGE_CPU_THREADS=0` panic，`SINGLEFS_CRASH_AMPLIFICATION_THREADS=0` 取默认。这条正好回答 J2 里「有没有调用方拿 0 当取默认」：`checking_threads()` 就是把 0 当取默认的那一个；脚本侧没有校验挡住 0，仓里的脚本与记录里 `grep -rnE` 不到 `--threads 0`、`CRASH_AMPLIFICATION_THREADS=0`、`CPU_THREADS=0` 这样的调用，所以今天没有调用方拿 0 当取默认，只是同一个 crate 里两种口径并存。
- 会推翻：核对线程数写错就悄悄取默认是有意的设计，且写在条款里（我没找到这样的条款）。

### 4.2　C611：三个发射点只有一个有站点级用例（够不着）

条款：「每一处各一条快档用例用假时钟核」（`.claude/kb/checks-owed.md:480`）。判决：「改代码：补用例（假时钟）与变异」（`research/prompts/test-script-align-code-r1-main-verification.md:30`），写的是「那一臂」单数。

冻结副本里 `by=wall_clock` 的发射点三处：CPU 块「"CRASH_AMPLIFICATION_BLOCK by=wall_clock point={}…」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1440`）有假时钟用例（用例在同文件，行串在 `crates/singlefs-checker-tier/src/crash_amplification.rs:2918`）；GPU 块只有行的写法抽成函数后核（`crates/singlefs-checker-tier/src/crash_amplification.rs:1466` 与 `…:2929`），打印那一臂没有，实现员报告第 6 条自己写了「没核到」；层 0 的「LAYER0_PROGRESS by=wall_clock」（`crates/singlefs-checker-tier/src/crash.rs:3282`）全仓一条用例都没引它（`grep -rn 'by=wall_clock' crates` 只命中两处发射点与一处注释，在 `outputs/j4_j5.out`）。共用的接收函数有两条用例，所以「站点那一臂被删」这类改坏三处都看不见。
- 判定：判决「那一臂」只关了三分之一；GPU 一处推不动（要卡），层 0 一处纯属漏掉。
- 会推翻：站点只是 `handle` 里一行 `println!`，共用接收函数的用例已经算罩住（那么条款「每一处各一条」本身就过严，该改条款）。

### 4.3　K5：环境守卫只守了两处（够不着，低险）

判决：「改代码：用例自己清、自己还原（守卫），不读别的用例与调用方留下的环境」（`research/prompts/test-script-align-code-r1-main-verification.md:34`）；上游：「不依赖跑序，不读别的用例留下的状态：文件、全局变量、环境变量、临时目录。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:12`）。守卫（`EnvironmentRestoredOnDrop`）用在流水线用例与 `common_crash_points`。同形没守的：库内「std::env::set_var("ENABLE_ACROSS_MACHINES", "1");」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2745`，到 2759 才 `remove_var`，中间 panic 就不还原）；库内另一条先「std::env::remove_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY");」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2650`）再 `set_var`、再 `remove_var`（`crates/singlefs-checker-tier/src/crash_amplification.rs:2679`）；两份固定脚本用例「std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY", &directory);」（`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs:268`）及 unmount 那份，设了不删。
- 险有多大：库内两条用例今天没有第三个读这几个变量的用例，没有真竞态，只是 panic 时不还原；两份固定脚本用例是由入口脚本带环境驱动的、一个二进制一条用例，读调用方环境是设计。所以我标低险，不标「该采」。
- 会推翻：这两条库内用例所在的二进制里，有别的用例读 `ENABLE_ACROSS_MACHINES` 或 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY`（我数的是 `grep` 命中，没跑）。

### 4.4　C604、G1、S2：站得住

- C604：`grep -rn '本机有一张\|本机要有一张' crates --include=*.rs` 在冻结副本里零命中；取卡的落点全部经入口：库内计时用例（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 997、1126 行）、σ 对拍、量显存（`usable_cards_or_report_not_run`）、两份对拍。直接调 `usable_gpu_cards()` 的只剩「pub mod gpu_test_start {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:500`）里面与 `gpu_cards_by_priority`。
- G1：产品路径 `gpu_cards_in_use` 没卡一律 `Err`，不退回 CPU，所以 G1 的「驱动坏了被当本次未跑」形状不在那里。PCI 读不出判红（容器里要不要改本次未跑）是实现员留给主 agent 的一条，不是判决的错。
- S2：坏盘输入的重放行只附在大档入口同样判的两条断言上（`crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs` 的 `how_to_replay` 在 202、214 行，大档「fn the_large_tier_of_bad_disk_inputs() {」在 1050 行，同样两条），结构断言消息里没带重放行，所以没有「照抄重放行判绿」的形状。

## 五、J3：变异表

冻结副本 `crates/mutations.tsv` 共 1721 行；对第一轮冻结副本变动的行是第 1585、1587、1590、1595、1609 行加第 1645–1721 行新增的 77 行，合计 82 行。第 1595 行（`.wgsl` 里的锚点）不在四份报告写明的改动里，是别的会话改了那份文件，我只核了锚点与点名，不判它。

### 5.1　82 行静态核（`k3_static_check.py`，原样输出 `outputs/j3.out`）

每行查：六段齐、文件在、原文恰好命中一次、替换文与原文不同、点名的测试在 `crates/` 里恰好一处 `fn` 定义、不带 `#[ignore]`、参数里的 `--features` 罩得住测试文件头的 `#![cfg(feature …)]`、`--test` 对得上文件名。输出：

```
BAD row 1709 test_file=None :: C607 故障注入重放行漏写比重（大档入口退回 broad，偏重取样的一批复现不出） :: test_defs=2
BAD row 1710 test_file=None :: C607 坏盘输入重放行漏写比重（大档入口退回 broad，偏重取样的一批复现不出） :: test_defs=2
rows_checked=82 rows_with_problems=2
```

两行「BAD」是同一个用例名「replay_line_of_a_batch_unlike_the_large_tier_defaults_runs_the_same_campaign_on_the_large_tier_entry」落在两个包里（`crates/singlefs-harness/tests/fault_injection_fast_tier.rs:431`、`crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs:1129`）。两行参数都带 `-p <包> --test <文件>`，门禁只在那一次 cargo test 的输出里找点名的完整名，找到恰好一个就够（`research/scripts/crates-mutation-rows.py:659` 的文档串「这一行的点名测试在基线输出里不是」恰好一个用例、结果都是 ok 时才报问题）。所以不是问题；但是「一个语义概念全仓一个名字」（`.claude/singlefs-ai-sop/rules/code-discipline.md:31`）会把这两个同名用例记一笔：改名成带「fault_injection」与「bad_disk」的各一个，两行变异的筛选词就不必再靠 `--test` 分开。
- 会推翻：门禁按用例名不带包去重（我读的是 `results_by_test` 一个函数，没跑门禁）。

### 5.2　筛选词匹配条数（`k3_filter_check.py`）

67 行带筛选词，数筛选词在目标里匹配到几条 `fn`：

```
BAD row 1609 filter='replay_lines_of' matches=3 [三条 replay_lines_of_* 用例] named='replay_lines_of_the_fast_tier_run_the_same_campaign_on_the_large_tier_entry'
filter_rows_checked=67 bad=1
```

第 1609 行筛选词 `replay_lines_of` 匹配三条（小快档、快档、回退取样），点名的是中间那条。门禁只认点名的完整名在输出里出现一次（`research/scripts/crates-mutation-rows.py:626` 的正则按完整名匹配），三条都跑、红的至少含点名那条就抓到。我进一步核了它红不红：三条共用的「the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_large_tier_entry」里「let finding_seed = HistorySeed(campaign.first_seed + 5);」（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1105`），与替换文写的 `self.first_seed` 不同，「复现这一段」那行的种子会错、`replayed_finding == original_finding` 红。站得住。

### 5.3　M3：第 1587 行的推断成立，而且变异红在前置断言上

问题：实现员推「改坏根槽写的 magic 之后那个点有状态判红」（`/tmp/claude-1000/impl-code-r1-crash-amp/report.md` 第 8 条）；这个推断错了，第 1587 行会不会红。

读用例「fn a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts() {」（`crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:861`）：先断言管根槽写的点有状态、判器输入相同、计划哈希相同，然后「"这一步的写表不同，复用键不同"」（`crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:897`）。第 1587 行的替换文把每个点的 `write_table_with_barriers` 换成全零，这一点上两条流的父镜像相同，复用键 = f(写表摘要, 父镜像摘要)，两个摘要此时都相同，复用键必然相等，`assert_ne!` 当场红。红在读回对照之前。

所以：(1) 变异红不靠「改坏 magic 后判红」那一推；(2) 那一推若错，最后的「"这一对在管根槽写的那个点上判得不同：根槽写改坏的那一条有红"」（`crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:911`）在**没改坏的代码**上就红，基线不绿，门禁报基线问题、不会悄悄绿。代价：变异是被前置条件抓住的，「读回逐态相同」那一判没有被这一行变异罩住；那一判由第 1588 行（块键不接判法版本）与版本表那几行罩。
- 会推翻：某个点的写表摘要不是 `reuse_key` 的输入（读 `plan_crash_points` 里 `StepAttributes` 的构造，`write_table_with_barriers` 与 `parent_output` 两个字段进 `reuse_key()`，我没逐行核 `reuse_key()` 的实现）。

### 5.4　C604 没留变异的理由

GPU 簇实现员的理由：有卡机器上把 σ 对拍、量显存、计时用例改回 `.expect` 或直接 `usable_gpu_cards()`，走同一条「起好卡、照跑」的路，没有用例会红，只有没卡机器分得出（`/tmp/claude-1000/impl-code-r1-gpu/report.md` 第 2 条）。就「变异」这一形式它成立：门禁在本机跑，本机有卡。

但「留不了守法」不成立：改回去会被一个与硬件无关的**源码扫描用例**抓住，仓里已有同形先例：库内用例扫本文件里每一处 `env::var("…")`、名字都得在表里（`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 2933 行起的 K5 用例），变异表里也有它的行。照这个形：扫 `tests/*.rs` 与 `src/crash_judge_dispatch.rs`，除 `gpu_unit_checks.rs` 之外的任何文件出现 `usable_gpu_cards(` 或 `.expect("本机` 判红，加一行变异把某个对拍改回直接取卡，这行在有卡的机器上同样红。第一轮判决对 C604 没有要变异，实现员定义第 6 步要每条改法留一行，所以这是实现员遇到的真缺口，不是判决的错。
- 会推翻：某个 GPU 用例有正当理由直接调 `usable_gpu_cards()`（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 里的计时用例不在此列，它们经入口）；那样扫描要带白名单。

### 5.5　`#[should_panic]` 与门禁

全仓 `#[should_panic(expected = …)]` 14 处，全带 `expected`；`#[should_panic]` 裸的只命中下面那条注释自己。第一轮之后新写的 8 处：σ 对拍文件 1、GPU 对拍文件 3、`crash_amplification.rs` 1、`history.rs` 2、`crash_injection_campaign.rs` 1。仓里另有一条相反的说法：「不写成 `#[should_panic]`」（`crates/singlefs-checker-tier/src/crash.rs:4178` 的文档注释）理由是门禁 59 号认不出这条测试红没红。读门禁今天的判法：结果行的正则「(?: - should panic)?」在点名之后可有可无（`research/scripts/crates-mutation-rows.py:626`），文档字符串明写「点名可以省掉模块路径。」（`research/scripts/crates-mutation-rows.py:625`）。所以 `crates/` 这张表认 `#[should_panic]`，`crash.rs:4178` 那条注释是过期的；它让同一个仓里同一类用例有两种写法（`catch_unwind` 与 `should_panic`）。`research/scripts/mutate.sh:1028` 抓红名字的写法会把「- should panic」带进名字，但它不是 `crates/` 表用的判法。
- 会推翻：`crates/mutations.tsv` 实际由 `mutate.sh` 而不是 `crates-mutation-rows.py` 判（门禁 `checker-tier-crates-mutation-replay.sh` 第 23–26 行写的是前者的行格式与基线判法，我没读它调用哪个脚本）。

## 六、J4：编码纪律

### 6.1　`enum_listing_every_member!` 宏挪进核心层

宏定义：「macro_rules! enum_listing_every_member {」（`crates/singlefs-core/src/admission.rs:605`）；harness 里留一个转发宏，再加一个 `pub use … as enum_listing_every_member_defined_in_the_core_layer`，全仓三个名字指同一个宏。

- 合编码纪律「重复要生成」：「重复要生成，不许手抄。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:179`）同一条往下是「宏和 `build.rs` 是生成的正路，只做生成、不藏逻辑，生成出来的东西要有测试盯着」；这个宏只生成枚举与 `ALL`，有测试盯着（`the_generated_member_table_lists_every_member_once_in_declaration_order`，变异表第 1708 行）。站得住。
- 合 D17：我读了 `.claude/kb/decisions/17-实现分层与第三方管道.md` 的 8 个已定项标题与连锁表（`grep -n '核心层\|测试'` 命中的几行），没有一条管「核心层能不能放只给测试生成清单的东西」；已定项 1 管的是「核心层规定单一失败模型」。所以 D17 既不禁也不许，靠编码纪律判。核心层本来就有只供测试的东西：「只供测试的开关」（`crates/singlefs-core/src/admission.rs:626`，`SpaceAdmission` 自己）。
- 可议（放哪）：(1) 宏住在讲空间准入的 `admission.rs` 里，而它是个通用的枚举生成宏，读 `admission.rs` 找一个与准入无关的宏不自然；(2) 核心层自己早有手写 `ALL`：「const ALL: [Self; 3] = [」（`crates/singlefs-core/src/transaction.rs:4064`）。若不挪，给 `SpaceAdmission` 手写 `ALL` 加一条穷举 `match` 的核对用例，是核心层已有的先例，不需要给核心层的公开面多一个 `#[macro_export]`。(3) `e165` 装置里另有一个同类宏「macro_rules! labelled_enumeration {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1531`），生成的也是枚举加 `ALL`：「重复要生成」只做了一半。
- 判定：不违字面，可议的是放哪；这一格我不替任何一边下重手，不算打中。
- 会推翻：「核心层不放测试用宏」被写进某个决策或规则（我没找到）。

### 6.2　展开上限的解析是不是同一件事写了两份

是。两个函数都是「读一个数值环境变量、设了而认不出就 panic、消息带变量名与原值、没设交调用方」：「pub fn from_value_of_the_variable(value: Option<&str>, default_when_unset: usize) -> Self {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:85`）与「pub fn number_from_the_variable_value_panicking_when_unreadable<Number>(」（`crates/singlefs-harness/src/history.rs:4308`）；前者 `trim().parse::<usize>()`，后者 `trim().parse::<Number>()`，除了固定的变量名与默认值外逐项同判法。检查包的库依赖 harness（`crates/singlefs-checker-tier/Cargo.toml` 第 12 行有 `singlefs-harness = { path = "../singlefs-harness" }`，该文件第 6 行描述里写明「脚手架在 singlefs-harness」），且检查包的测试已经在从 `singlefs_harness::history` 引它；`crash_amplification.rs` 没有理由不引。

同一族里还有两样要记：
- 「pub fn number_from_process_environment_panicking_when_unreadable<Number>(」（`crates/singlefs-harness/src/history.rs:4331`）全仓没有调用方（`outputs/j4_j5.out` 的「死代码」一段只命中它自己的定义与文档），收尾报告写「这一轮没有调用方」。「用不到的代码删掉」（`.claude/singlefs-ai-sop/rules/code-discipline.md:181`）。
- 这个通用的数值解析函数住在 `history.rs`（随机历史生成模块）里，被 GPU 用例与记录核对器用例引；放哪不是「说不出名字」的毛病，但读者去 `history` 里找环境变量解析不自然。

### 6.3　新名字、`#[should_panic]`、环境守卫

- 新名字：`decide_gpu_test_start`、`first_usable_card_or_report_not_run`、`reuse_and_scale_fields_of_the_line`、`number_from_the_variable_value_panicking_when_unreadable` 都说得出做什么、不做什么。仅 `a_line_was_printed_at` 一个（第二节）。
- `#[should_panic]`：新写的 8 处全部带 `expected = …`，消息里带变量名与原值，没有裸的。与门禁的关系见 5.5。
- 环境守卫：`EnvironmentRestoredOnDrop` 记原值、清掉、丢掉还原；`set` 断言变量在记下的那一批里；改环境的用例再拿一把二进制内的锁。形状对；覆盖面见 4.3。

## 七、没做什么

- 没跑任何测试、没编译；所有「会不会红」都是读代码与静态核推的，已逐条写明推翻条件。
- 没判 J1、J2（攻方格）；4.1 的线程数与 4.2 的挂钟进度只是 J5「同形别处」的出处，不是替攻方判 J2。
- 没核 R1 的两个推的前提（根槽写 txg 严格递增、基镜像只有第 0 代根），那是 J1。
- 没逐份核 6.2 里「至少 9 份」各自对 0、空串、前后空白的处理；没核门禁 `checker-tier-crates-mutation-replay.sh` 实际调用哪个脚本（5.5 的推翻条件）。
- 没去有卡与没卡的机器上看 G1；容器里读不出 PCI 判红那一条交主 agent，是实现员已报的。
- 没读攻方腿报告与它的模型目录、本轮判决（禁读清单）；没读主仓 `crates/`。
- 不替主 agent 撤销或维持判决，只交复核结果。

## 八、交回前核

模型目录 `research/prompts/test-script-align-code-r2-sonnet-model/`：`rerun.sh`、`k3_static_check.py`、`k3_filter_check.py`、`outputs/j3.out`、`outputs/j4_j5.out`、`outputs/j5_e165.out`、`SHA256SUMS`。

```
$ cd research/prompts/test-script-align-code-r2-sonnet-model && sha256sum -c SHA256SUMS
rerun.sh: OK
k3_static_check.py: OK
k3_filter_check.py: OK
outputs/j3.out: OK
outputs/j4_j5.out: OK
outputs/j5_e165.out: OK
$ python3 research/scripts/cite-check.py research/prompts/test-script-align-code-r2-sonnet-output.md --root /tmp/claude-1000/test-script-align-code-r2/tree --root . --background research/prompts/_test-script-align-code-r2-background.md
  ✓ 核了 63 处引文，对上 63 处，没判 0 处（逐处列在上面）
$ python3 research/scripts/cite-check.py research/prompts/test-script-align-code-r2-sonnet-output.md
  ✓ 核了 63 处引文，对上 63 处，没判 0 处（逐处列在上面）
```

草稿目录 `/tmp/claude-1000/test-script-align-code-r2-sonnet/` 里只有 grep 输出与核对文本（`envvars.txt`、`k3_static.out`、`k3_filter.out`、`sha-check.txt`），没有编译目录与仓副本。
