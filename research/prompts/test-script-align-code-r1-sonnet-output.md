# test-script-align-code-r1 正推腿（Sonnet）报告

批次：`records/2026-10-01-对接上游测试脚本规范.md#3`。轮：test-script-align-code-r1。格：K1、K4、K5。读的是冻结副本 `/tmp/claude-1000/test-script-align-code-r1/tree/`（下文代码路径都相对它，`--root` 取它），没读主仓 `crates/`。
引用规矩：kb 条款写 `.claude/kb/checks-owed.md` 自己的行号（都现 grep 过），代码写冻结副本里的路径与行号；补丁里的新增行我另抽成 `/tmp/claude-1000/test-script-align-code-r1-sonnet/added-*.txt` 当检索用，不当依据。
立场：从条款往下推「代码该有什么」，再逐格与冻结副本比；条款是 C601、C604–C608、C611、C620（`.claude/kb/checks-owed.md`）加上游 `test-script-discipline.md` 的几节。
模型目录 `research/prompts/test-script-align-code-r1-sonnet-model/`：一个只用 std 的探针（把冻结副本里 `common_gpu_adapter/mod.rs` 原样编进来，喂八种适配器枚举），加一组对冻结副本的 grep；`rerun.sh` 从仓根跑，`SHA256SUMS` 在同目录。

## 一、K1　代码做的是不是条款说的

### 1.1　C620（GPU 对拍「没卡」只看 Vulkan 枚举）——冲突

条款（怎么拦那一格）：
- 「PCI 上有卡而 Vulkan 枚举不出判红」（`.claude/kb/checks-owed.md:489`）

代码（整行，取卡判定的入口与「没卡」那一支）：
- 「pub fn decide_gpu_test_start(」（`crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs:53`）
- 「if nvidia_discrete_adapter_count == 0 {」（`crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs:64`）

这个纯函数只有两个输入：`adapters`（Vulkan 枚举）与 `usable_card_count`。条款要的第三个输入（PCI 上厂商号 0x10DE、class 前缀 0x03 的设备）没有地方进来。冻结副本里：

```
$ grep -rnF '/sys/bus/pci' crates          # 零命中，rc=1
$ grep -nE '0x03|class' common_gpu_adapter/mod.rs gpu_unit_checks.rs   # 零命中，rc=1
```
（原样输出在模型目录 `rerun-output.txt` 的「探针 2」。）

探针 1（`rerun-output.txt` 第 2 行起，八种枚举都经 `decide_gpu_test_start` 现算）：
```
case=A 没有任何适配器（机器上没卡；也是 PCI 上插着 NVIDIA 卡而 Vulkan 驱动坏了枚举不出来的那一种） outcome=NotRunNoGpuAdapter
case=F NVIDIA 独显枚举出来而一张卡都没起来 outcome=AdapterPresentButNoCardStarted
```
A 与「PCI 上有 NVIDIA 卡、Vulkan 驱动坏了」喂给函数的是同一对参数 `(&[], 0)`，结局只能是 `NotRunNoGpuAdapter`，即条款 C620 题面说的那件事（驱动坏了被当成本次未跑）原样还在。B 集显、C AMD 独显、D NVIDIA 虚拟 GPU、E 软件渲染都是 `NotRunNoGpuAdapter`（只认 NVIDIA 独显这一点与 `usable_gpu_cards` 筛卡同口径，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:17`）；H 集显加 NVIDIA 独显且独显没起来判红。
上游依据：「配置开了的硬件或机器是整个脚本离不开的，写成」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:80`）同一条往下是「用不上（设备不在、驱动不对、内核编不过、显存不够、第二台连不上）就退 78」。
会推翻这条的现象：冻结副本里有任何一处读 `/sys/bus/pci` 或 `lspci` 并进了 `decide_gpu_test_start` 的输入；或者主 agent 在一台插着 NVIDIA 卡、Vulkan 枚举为空的机器上跑到 GPU 对拍判红而不是 `NOT_RUN_NO_GPU_ADAPTER`（这台机器上的 PCI 现状我没查，`research/prompts/` 里没有这种机器的数据，复核不了）。

### 1.2　C604（没卡时 GPU 对拍打 NOT_RUN 并返回成功，不 panic）——部分一致

条款：
- 「并返回成功，带特性那一趟逐条报」（`.claude/kb/checks-owed.md:473`）
- 「没卡的机器上开 gpu 特性仍是 panic 退 101（这一格红，不是假绿）」（`.claude/kb/checks-owed.md:473`）

做到的：三份被判文件的取卡都改走 `first_usable_card_or_report_not_run`——
- 「GpuTestStart::NotRunNoGpuAdapter { reason } => {」后面一行是「println!("{}", not_run_line(test_name, &reason));」（`crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs:130`），随后 `None`，用例 `let Some(card) = … else { return; }` 当场返回成功（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:63`、`crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:38`）。
- 门禁读这一行靠 `--nocapture`：「cargo test --release -p singlefs-checker-tier --features "$feature_list" --lib "${target_arguments[@]}" -- --nocapture 2>&1 \」（`.claude/gate.d/checker-tier-layer0-replay.sh:724`）。不带 `--nocapture` 直接跑，libtest 吞掉通过用例的 stdout，人看到的是 ok、看不到「本次未跑」（推的，没跑）。

没做到的（冲突三处，都是 GPU 取卡但没走新入口）：
1. GPU 对 CPU 的 σ 对拍，条款 C601、C608 点名的那一条：
   - 条款：「并返回成功，带特性那一趟逐条报」（`.claude/kb/checks-owed.md:473`）
   - 代码整行：「.expect("本机要有一张空闲显存够的 NVIDIA 独显；没有就是这条测试的环境没搭好，不悄悄跳过");」（`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1084`），取卡那一行是「let card = usable_gpu_cards()」（同文件 `:1081`），用例名在 `:1017`，挂 `#[cfg(feature = "gpu")]`。没卡的机器上带 gpu 特性跑到它是 panic，不是 `NOT_RUN_NO_GPU_ADAPTER`；这正是 C604 题面「仍欠」的那一格（红，不是假绿），补丁没碰这份文件。
2. 量显存的 `#[ignore]` 用例，没卡时静默通过（假绿）：
   - 代码：「#[ignore = "量显存上限用：要清场之后空着的卡，每张卡开十几档草稿区"]」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1592`）；用例体里「let every_card: Vec<(u32, u64)> = usable_gpu_cards()」（同文件 `:1614`）零张卡时 `for` 一圈不进、没有断言、没有 NOT_RUN 行，返回成功。上游「用编译开关（特性、`cfg` 这类）分档的」那一条（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:54`）要求某一类编进来 0 条报本次未跑，这里是编进来了、跑了、什么也没判却是 ok。带 `--include-ignored` 的全量会跑到它。
3. 库内两条 `#[ignore]` 的编译计时用例，没卡时 panic：「let card = cards.first().expect("本机有一张能用的卡");」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:977` 与 `:1101`）。
对拍判定本身（绿、红）不受影响；受影响的是没卡的机器上报什么。
会推翻这条的现象：σ 对拍在没卡的机器上打出 `NOT_RUN_NO_GPU_ADAPTER` 并退 0（现读代码，不会）；或量显存那条用例在零卡时有断言（冻结副本 `:1593` 起整个函数体没有 `assert`，我读过）。

### 1.3　C608（GPU 变异要罩内核与 GPU 对 CPU 的对拍）——转 K3，K1 一格不单判

条款写的检查是「变异进表」，不是实现里的一处判法；这一格归攻方的变异表格（K3）。我只确认：被点名的 GPU 对拍用例在没卡时会打 `NOT_RUN_NO_GPU_ADAPTER`（1.2 的前两条），门禁认这一行记「本次未跑」（`.claude/gate.d/checker-tier-crates-mutation-replay.sh:58`）；σ 那条没卡时 panic，变异整表跑到它会是「红」而不是「本次未跑」，与 C608 写的「没卡的机器上这几行变异记本次未跑」对不上（同 1.2 第 1 点）。

### 1.4　C601（只差高位的两个版本让恢复落到不同根的流）——一致（会不会红要 GPU，复核不了）

条款：
- 「造一条一个落点超过 32 次写、且只差高位的两个版本让恢复落到不同根的流，加进对拍」（`.claude/kb/checks-owed.md:470`）

代码：新用例 `the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write`（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1179`）。输入由「const FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT: usize = 32;」（同文件 `:229`）在发布根槽前垫 32 次写造出，用例先在取卡之前断言判器表里有「低 32 位相同、只差高位、字节不同」的一对版本（`:1220` 起的 `told_apart_only_by_the_high_bits`），再取卡，「let comparison = compare_on_every_state(&card, &flow, EXPAND_UP_TO);」（同文件 `:1241`）逐状态比 GPU 与 CPU。条款要的三样（一个落点超过 32 次写、两个只差高位的版本、让恢复落到不同根）在输入上都有，断言了前提而不是假设它成立。
会推翻这条的现象：把 `crash_judge_common.wgsl` 的 `version_of_location` 改成只比低 32 位之后，这条用例在有卡的机器上仍绿（这要 GPU 与 checker 档的重型测试，我没跑，也复核不了；`crates/mutations.tsv:1591` 有一行点它，归 K3 判）。

### 1.5　C605（复用与剪枝关掉再判的对照）——一致，做法与条款写的开关不同

条款：
- 「先拿输入 A 填库，再拿与 A 只差一项被键漏掉的输入 B 在同一个库上跑，与 B 关掉复用的结果逐态比，不同判红」（`.claude/kb/checks-owed.md:474`）

代码：`reused_verdicts_equal_state_by_state_the_verdicts_judged_without_the_library_or_the_prefix_split`（`crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:473`）。三份判定：开着复用与前缀共享（共库里先放 A、再放旧判法版本下的 B、最后 B）、关掉复用（B 独自进空库）、关掉前缀共享（不进库，`judge_layer0_state_range` 整条流逐态现判），逐态相同。A 与 B 的尾巴同名同父镜像、只差内容，A 的版本表写错判红、B 健康，所以 B 复用到 A 的红会让对照红；旧判法版本下的 B 也故意判红，判法版本漏出键同样会让对照红。对照不空：断言前缀块数大于 0 且 `blocks_already_judged` 恰等于前缀块数。
与条款字面的差别：条款说「关复用与剪枝的开关要先加」，补丁没加开关，「关掉」是用空库与整流直判这两个现成路径做的，不是同一条代码路径上拨一个开关。我判它覆盖了条款要判的差异（键漏一项、判法版本漏出键），只是「关前缀共享」那一臂走的是另一份实现（`judge_layer0_state_range`），所以它核的是「流水线的共享前缀结果等于整条流直判」，不核「流水线内部开关拨到关」。键层那一行变异（`each_attribute_changes_the_reuse_key`）归 K3。
会推翻这条的现象：把复用键里写表（或判法版本）去掉之后这条用例仍绿；我没法在不跑 checker 档的前提下量，复核不了。

### 1.6　C606（展开深度写进驱动行，展开上限认不出就 panic）——一致，附同族没改的两处

条款：
- 「规模变量的值认不出判红、不退回默认」（`.claude/kb/checks-owed.md:475`）
- 「展开上限变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 的值写错时悄悄退回默认」（`.claude/kb/checks-owed.md:475`）

代码：`ExpandUpToWritesPerSegment::from_value_of_the_variable`（`crates/singlefs-checker-tier/src/crash_amplification.rs:78`）对设了而不是非负整数的值 panic，消息带变量名与原值；`from_environment_or` 的不是 UTF-8 一支也 panic；单元测试「fn an_unreadable_expand_up_to_value_panics_naming_the_variable_and_the_value() {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2370`）。三个原先各自 `.ok().and_then(parse).unwrap_or(default)` 的调用点（两份固定脚本用例与公共模块）都换成这个入口。驱动行末尾接 `states_reused= exhaustive= expand_up_to= ignored_points= states_skipped_by_ignore=` 由「pub fn reuse_and_scale_fields_of_the_line(run: &EnvironmentRun) -> String {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2678`）一处拼，`run_from_environment` 另外在录入之前核流自己的展开方式与上限逐段相同，不同就拒（`crates/singlefs-checker-tier/src/crash_amplification.rs:2794`）。
同族没改的（不是 C606 点名的那个变量，但是同一种毛病）：
- GPU 判器对拍用例读 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO` 仍是「std::env::var("SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO")」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:54`）接 `.ok().and_then(…parse().ok())` 再「.unwrap_or(DEFAULT_EXPAND_UP_TO)」（同文件 `:57`），写错悄悄退回 8。补丁动了这份文件但没动这个函数。
- 条款里「直接带 SHARE=i/n 跑、只判了一部分也照样绿」那一样：`exhaustive` 只比每段写数与上限，不看分到哪一份（`crates/singlefs-checker-tier/src/crash_amplification.rs:2682` 的 `reuse_and_scale_fields_of_the_line` 只写 `run.exhaustive`）；汇总行里的 `share=i/n` 是补丁之前就有的、`flow=` 行没有。条款把「成功句不说全量、完成标记不认它」放在入口脚本一侧，crates 这一侧我判不属于这次补丁的出口，只记一笔。

### 1.7　驱动行字段与上游「每项结束打一行」——一致（单行与整项两层），附一处写 0 的口径

上游：
- 「两种都不写 0）、用了几秒；末行是总计。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:31`）

对得上：判了多少（`states_checked_here`）、红了多少（`red`）、复用了多少（新增 `states_reused`）、用了几秒（`seconds=`/`total_seconds=`）在同一行；项名、退出码、总计行在入口脚本（`research/scripts/crash-amplification.sh:35`），不归 crates。
「不复用」与「取不出」两种写法：crates 里两者都不写（`grep -rnE '取不出|不复用' crates/singlefs-checker-tier` 只命中 `verdict_store.rs:1860` 与两份测试文件里的注释和断言消息，都不是驱动行字段）；取不出的判断放在入口脚本一侧（`research/scripts/crash-amplification.sh:59` 起那一段，`reuse_unknown`）。crates 一侧的口径是「每一行带自己那一份、几行相加就是一项」，所以有三处行内写 `states_reused=0` 但不是「这一趟没复用」：
- 被叫停的那一趟：「states_reused: 0,」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3063`），而前面 `:2925` 的 `states_reused_unless_stopped` 已经数出来了，是特意不报；
- 设了 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 的导入那一趟：`:2925` 起的 `if imports_another_library {` 一支给 0；
- 只排派活计划的那一趟（`:2995`），不打 `CRASH_AMPLIFICATION` 行。
按「一项的几行相加不重复计」这个设计它们不错；但单看这几行，读者读到的是「复用 0」，上游要的是「取不出」。入口脚本拿 `states_reused=` 直接取这几行时（`research/scripts/crash-amplification.sh:61`）得到的和是对的，这条我记「设计差异，不判错」。
会推翻这条的现象：入口脚本按行单独上报（而不是按项求和）时读到被叫停行或导入行的 0，就成了上游点名不许的「写 0」。

### 1.8　C607（随机取样点用例红了凭打出的种子复现不出）——大体一致，一处装配无自证

条款：
- 「每种随机用例在快档里有一条自检：拿它判红时打出的复现行原样重跑，逐态与原趟相同，不同判红」（`.claude/kb/checks-owed.md:476`）
上游：
- 「随机输入带种子。红了打印种子，凭种子能单独复现那一条。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:13`）

对得上的：
- 随机历史：复现行由「pub fn random_history_reproduction_line(seed: HistorySeed, sampling: &HistorySampling) -> String {」一处拼，赋值那一截与入口读回共用同一张变量表 `HistorySamplingVariable`（`crates/singlefs-harness/src/history.rs:3959`）；`NewFindingReport::render` 在每个新发现后打「"  复现：{}",」（`crates/singlefs-harness/src/history.rs:4901`）。入口 `shrink_one_failing_seed_from_the_environment`（`crates/singlefs-harness/tests/random_histories.rs:1721`）与六条自证（例：「fn reproduction_line_of_the_fast_tier_regenerates_the_same_history_and_execution() {」，`crates/singlefs-harness/tests/random_histories.rs:1829`）都经同一个 `history_to_reproduce_from`，六条一值一条、公共函数带参数，符合上游「按参数循环……拆成一个带参数的函数加每个取值一条用例」。
- 崩溃注入：重放行 `crash_injection_replay_line` 带种子区间、每段几个崩溃状态、短段整段枚举的写数上限、步数、比重、checker、盘宽、空间准入，大档入口 `crash_injection_campaign_from_environment` 照同一张表读回；单元自证覆盖两种摆法（「fn every_crash_point_draw_reads_back_from_its_own_replay_line() {」，`crates/singlefs-checker-tier/src/crash_injection.rs:2342`），集成自证两条（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1179` 起）。
- 与条款的差别：自证核的是「复现行读回的取样参数、重新生成的历史与原趟逐步相同」，没有把历史重跑一遍；崩溃注入那两条自证的文档注释里明说不跑崩溃状态（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1068`），摆出来的崩溃状态由种子定这一环靠别处的 `crash_points_are_reproducible_proper_subsets_sorted_by_segment`（`crates/singlefs-checker-tier/src/crash_injection.rs:2455`）。条款要的「逐态与原趟相同」落成「生成的历史逐步相同加跑法参数相同」。

没对上的：
- 「sampling: HistorySampling {」这一处装配（`crates/singlefs-harness/src/history.rs:5067`）是 `run_history_campaign` 把自己的步数、比重、跑法交给 `NewFindingReport` 的唯一地方。六条自证与新单测都是手工造一个 `NewFindingReport { …, sampling }` 再渲染，没有一条经过 `run_history_campaign`；`grep -c 'weights: \*weights' crates/mutations.tsv` 输出 0，变异表里也没有一行改这一处（探针 7，`rerun-output.txt` 末段）。这一处写错（例如固定写成 `BROAD`）时，所有取样点用例判红时打出的复现行都错、自证全绿（推的，没跑；补丁里 8 条 C607 变异行按名字看改的是读写变量表的那几处，没有改这一处）。
- 另外两个随机装置的重放行没动：「重放：SINGLEFS_BAD_DISK_FIRST_SEED={} SINGLEFS_BAD_DISK_SEEDS={} SINGLEFS_BAD_DISK_OPERATIONS={} 跑大档那条 #[ignore] 用例」（`crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs:83`）与故障注入那一份（`crates/singlefs-harness/tests/fault_injection_fast_tier.rs:75`）。C607 条款名下只点了随机历史与崩溃注入，这两个是否也漏比重、盘宽我没量，只记「没改」。
会推翻这条的现象：在 `run_history_campaign` 里把 `weights: *weights` 改成 `GenerationWeights::BROAD`，六条自证与 `random_histories` 其余用例仍全绿（我没在副本上量，攻方腿或主 agent 可量；它是 harness 档，轻）。

### 1.9　C611（长跑的项按挂钟打进度）——大体一致，站点层没有自证

条款：
- 「按挂钟每 5 分钟至少一行；每一处各一条快档用例用假时钟核」（`.claude/kb/checks-owed.md:480`）

代码：
- 间隔常量「pub const PROGRESS_LINE_INTERVAL_IN_SECONDS: u64 = 300;」（`crates/singlefs-checker-tier/src/crash.rs:2746`），一处定义，三个发射点共用。
- 时钟可换：「pub fn receive_until_the_senders_are_gone_with_wall_clock_lines<Message>(」（`crates/singlefs-checker-tier/src/crash.rs:2804`）收 `clock: &dyn Fn() -> Duration`，用例换成走 301 秒一步的假时钟。
- 三个发射点：层 0 的「"LAYER0_PROGRESS by=wall_clock finished_slices」（`crates/singlefs-checker-tier/src/crash.rs:3282`），内层 CPU 一块的「"CRASH_AMPLIFICATION_BLOCK by=wall_clock point=」（`crates/singlefs-checker-tier/src/crash_amplification.rs:972`），GPU 一块的「"CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1616`）。
- 重置时刻的口径我核过：每收到一条消息重置「上一行打在什么时候」，前提是收到一条消息之前对应的行已经打过。CPU 块的行由 `judge_one_block` 末尾打（`crates/singlefs-checker-tier/src/crash_amplification.rs:841`，在发消息之前），GPU 块的行由卡线程打（`:1432`），层 0 的行由接收一侧自己打，三处成立，收到消息不会把没打过行的空档吞掉。
没对上的：条款写「每一处各一条快档用例」。补丁只给辅助函数写了两条用例（「fn a_wall_clock_line_is_due_whenever_a_whole_interval_passes_without_a_message() {」，`crates/singlefs-checker-tier/src/crash.rs:4671`，另一条是消息按时到不多打），三个站点里 `WallClockLineDue` 那一臂的打印没有任何用例核：把站点里 `println!`/`eprintln!` 那一臂删掉、或把间隔写成别的，辅助函数的两条用例仍绿（推的，没跑）；变异表里与它相关的一行改的是辅助函数里的比较（`crates/mutations.tsv:1589`）。
会推翻这条的现象：有一条用例让站点在假时钟下打出 `by=wall_clock` 行；冻结副本里 `grep -rn 'by=wall_clock' crates --include=*.rs` 只有三处发射点与一处注释，没有测试引它（我读过这几处）。

## 二、K4　编码纪律

方法：把四份补丁的新增行抽到草稿目录（`added-*.txt`，共 2843 行）逐条 grep；每条命令都现跑过。
```
$ cd /tmp/claude-1000/test-script-align-code-r1-sonnet; wc -l added-*.txt | tail -1
$ grep -nE 'unwrap\(\)|\bas (u8|u16|u32|u64|usize|i32|i64)\b|^\s*_ =>' added-*.txt        # 零命中
$ grep -nE '_ =>' added-*.txt                                                               # 零命中
```
（输出见第四节「命令输出」。）

### 2.1　通配臂——一致
新增代码里没有 `_ =>`。新增的 `match` 都穷举：`HistorySamplingVariable::suffix`（`crates/singlefs-harness/src/history.rs:3959` 起）、`CrashInjectionVariable::suffix`、`draw_environment_values`、`adapter_device_kind` 把 `wgpu::DeviceType` 五个成员逐个列出（`crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs:87`）。依据：「封闭集合的枚举，match 不写 _ =>。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:100`）。

### 2.2　unwrap / 错误——一致，附一处不统一
- 新增行里没有 `unwrap()`，用的是 `expect("…")`，消息带前提（依据：「不写 unwrap()，写 expect」，`.claude/singlefs-ai-sop/rules/code-discipline.md:147`）。
- 能恢复的失败返回 `Result`：复现入口认不得的值走 `ReproductionEnvironmentError`（`crates/singlefs-harness/src/history.rs` 的 `from_environment`），不 panic。
- 不统一：同一批里另一处同类输入（展开上限变量写错）走 panic——「pub fn from_value_of_the_variable(value: Option<&str>, default_when_unset: usize) -> Self {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:78`），库公开函数、有 `# Panics` 文档。规则那一条是「能恢复的失败（I/O 错、盘上数据损坏、资源不够）走 Result」（`.claude/singlefs-ai-sop/rules/code-discipline.md:141`）；环境变量写错算不算「能恢复」，规则没单写，C606 的条款本身要的是「认不出判红」，panic 满足条款。我记「两处口径不一，不判违规」，主 agent 要统一就挑一边。

### 2.3　魔法数字——有三处与规则「一个数只许有一处定义」不合
规则：「魔法数字写成命名常量：一个数只许有一处定义，名字或类型里带上单位。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:178`）
1. NVIDIA 厂商号两处定义：
   - 「pub const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;」（`crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs:12`）
   - 「const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:17`）
   补丁在第一处上面自己写明原因是后者私有、取不到。后者所在模块挂 `gpu` 特性，不挂特性的探针单测编不进它，所以不是随手抄；但两处写着同一个数，改一处不改另一处时没有东西会红（`grep -rn 'const NVIDIA_VENDOR_IDENTIFIER' crates` 命中两行，探针 5）。
2. 单测里的组合数与步数写成字面量：「assert_eq!(combinations, 5 * 2 * 4 * 2, "每一种组合都走过一遍");」（`crates/singlefs-harness/src/history.rs:5141`）与「operations_per_history: 150 + combinations,」（`crates/singlefs-harness/src/history.rs:5118`）。五、二、四、二分别是 `GenerationWeights::NAMED`、`PerStepChecker::ALL`、`HistoryDeviceWidth::ALL`、`SPACE_ADMISSION_CHOICES` 的个数，在断言里再写一遍；表里加一档、断言不改就红，红得对，但这是手抄一份个数而不是取 `.len()`。
3. 大档默认值与回退取样点的字面量：`large_tier_campaign_from` 里 `500`、`40`、`8`（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs` 的 `large_tier_campaign_from`），`rollback_sampling_campaign` 里 `12`、`20`、`6`；原先也是字面量，补丁搬了位置、没起名。快档那一份（`FAST_TIER_SEEDS` 等）是命名常量，同一文件两种写法并存。
这三条是 review 档的小问题，不挡补丁。

### 2.4　名字——大体清楚，三处可议
判据：「判据：不看注释、不看调用点、不看实现，光看这个名字，说不说得出它是什么、做什么、不做什么。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:15`）。
新名字多数说得清（`reuse_and_scale_fields_of_the_line`、`states_skipped_by_ignore`、`decide_gpu_test_start`、`environment_assignments_in_line`、`large_tier_campaign_from`）。可议的三处：
- 「pub fn a_line_was_printed_at(&mut self, now: std::time::Duration) {」（`crates/singlefs-checker-tier/src/crash.rs:2789`）：名字像事件陈述，不是动词；它做的是「记下上一行打在 now」并改状态，名字没说出「改了谁」。
- `ExpandUpToWritesPerSegment(pub usize)` 的元组字段到处以 `.0` 取（例如 `run.expand_up_to.0` 在 `crates/singlefs-checker-tier/src/crash_amplification.rs:2683`），字段没有名字；这个类型没有不变量，`pub` 字段合规，只是读 `.0` 要回头看定义。
- `PerStepChecker::ALL`、`HistoryDeviceWidth::ALL` 的 `ALL` 说的是「全部成员」，没说是干什么用的（复现入口认的取值全集）；`GenerationWeights::NAMED` 说得清，`ALL` 次之。
缩写：新增行里没找到 `cfg/ctx/msg/idx/cnt/buf/tmp` 之类（`grep -nE` 零命中，同 `added-*.txt`），单字母名零命中（闭包参数 `_` 不算名字）。

### 2.5　`history.rs` 那张变量表：一处定义、两个装置共用——一致，附一处手抄
- 表在 `crates/singlefs-harness/src/history.rs`：枚举「pub enum HistorySamplingVariable {」（`:3959`），前缀类型「pub struct ReproductionVariablePrefix(pub &'static str);」（`:3940`）。
- 两个装置各一个前缀：随机历史 `RANDOM_HISTORY_VARIABLE_PREFIX`（`crates/singlefs-harness/src/history.rs:3952`），崩溃注入「pub const CRASH_INJECTION_VARIABLE_PREFIX: ReproductionVariablePrefix =」（`crates/singlefs-checker-tier/src/crash_injection.rs:1983`）；取样那五项的后缀、取值、读回都只经 `HistorySampling` 一处，没有第二份。
- 崩溃注入自己的四个变量另有一个枚举（`crates/singlefs-checker-tier/src/crash_injection.rs:1988`），不与随机历史共用，也没有必要共用。
- 手抄一处：`GenerationWeights::NAMED` 手写列出五个常量（「pub const NAMED: [GenerationWeights; 5] = [」，`crates/singlefs-harness/src/history.rs:636`）。以后新增一组比重而忘了加进 `NAMED`，复现入口就认不得它的 `environment_value`；自证 `every_sampling_choice_reads_back_from_its_own_reproduction_assignments` 遍历的就是 `NAMED` 自己（`crates/singlefs-harness/src/history.rs:5106`），漏掉的常量它看不见。规则：「重复要生成，不许手抄。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:179`）。Rust 里没法枚举关联常量，只能靠宏或一条核对测试，记「review 档」。
- 还有一处读法留在表外：大档入口的 `SINGLEFS_RANDOM_HISTORY_FIRST_SEED`/`_SEEDS`/`_THREADS`/`_SHRINK` 用字面量字符串（`crates/singlefs-harness/tests/random_histories.rs:1653` 到 `:1668`），不在表里；它们不进复现行（复现行只要种子与取样参数），所以不影响「复现行与入口读同一张表」，只是前缀字符串在 `random_histories.rs` 与 `history.rs` 各写一遍。
会推翻 2.5 的现象：新增一个 `GenerationWeights` 常量、不进 `NAMED`，全仓测试仍绿（主 agent 可在 harness 档副本上量，轻）。

## 三、K5　测试原子化与快慢档

### 3.1　GPU 用例没卡时报「本次未跑」而不是假绿——部分一致
见 1.2。补充三点：
- 新取卡入口把三种结局分开：起好了卡照跑、没有 NVIDIA 独显适配器打 `NOT_RUN_NO_GPU_ADAPTER` 返回、有适配器而一张卡没起来 panic（`crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs:133` 的 `AdapterPresentButNoCardStarted` 一臂）。上游：「要特定硬件的那几条，在没有那种硬件的机器上报」「本次未跑」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:53`），三份被判文件里走新入口的用例对得上。
- 新增的 `gpu_adapter_probe_…` 自证只喂了两种判定：`decide_gpu_test_start(&[], 0)`（`crates/singlefs-checker-tier/tests/gpu_adapter_probe_reports_not_run_without_an_adapter_and_red_when_no_card_starts.rs:14`）与 `decide_gpu_test_start(&[nvidia_discrete_gpu], 0)`（同文件 `:31`），即本次未跑与红；绿（`RunOnTheFirstUsableCard`，`usable_card_count > 0`）一支没喂。上游自证那一条是「每一种判定（绿、红、本次未跑、用法错）至少喂一个答案已知的输入，判出来与答案相同」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:39`）。非 NVIDIA 独显、集显、虚拟 GPU 这几种枚举也没有喂。我的探针 1 把这几种都喂了一遍，结局都对（`rerun-output.txt` 第 2–9 行），即这是自证覆盖不全，不是判法错。
- 同一个用例里连做两个场景（先零适配器、再 NVIDIA 独显没起来），中间没有「确是一个场景」的注释：上游「确是一个场景的（同一份状态上按次序做几步），在用例旁写一行注释说明为什么」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:11`）。两个场景的状态互不相干（各喂一个枚举），按上游应拆成两条。
会推翻这条的现象：把 `decide_gpu_test_start` 里 `usable_card_count > 0` 一支改成恒不进，这条自证仍绿（我读到的代码是会绿；没量，因为它是一条 checker 档测试，不跑）。

### 3.2　新用例一个场景、不读别的用例留下的状态
- 种子复现（harness 档，`random_histories.rs`）：六条自证一值一条，公共函数带参数，每条自己造取样、自己生成历史，不读文件，不碰环境（读回用的是传进去的 `lookup` 闭包，不是进程环境），一致。
- 崩溃注入重放行两条（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1179` 起）：同形，一致。
- 复用对照（`crash_amplification_two_flows_share_their_prefix.rs:473`）：库目录以 `std::env::temp_dir()` 加进程号起名、先删后建、用完 drop，一个场景三份判定逐态比，一致。
- 驱动行字段用例有一处与上游「不读别的用例留下的状态：…环境变量」不合：「fn a_driver_line_reports_reuse_exhaustiveness_and_depth_and_its_reuse_equals_the_states_judged_in_the_library(」（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:747`）。它改进程环境：「std::env::set_var("SINGLEFS_MULTI_HOST_CONFIG", &configuration);」（同文件 `:756`，另两个变量在 `:757`、`:758`），三个变量在用例末尾 `remove_var`（`:773` 到 `:775`），中间任何一步 panic 就不还原，没有 drop 守卫。同时它调的 `run_from_environment` 还读一批它没设也没清的变量：`SINGLEFS_CRASH_AMPLIFICATION_SHARE`（`crates/singlefs-checker-tier/src/crash_amplification.rs:2848`）、`…_IMPORT`（`:2921`）、`…_PLACEMENT_ONLY`（`:2846`）、`…_EXPAND_UP_TO`（经 `from_environment_or`）、`…_STOP_FILE`、`…_VERIFY`、`SINGLEFS_GPU_CARDS_CONFIG` 等。调用方环境里设了其中一个（例如一台机器分到 `SHARE=1/2` 或设了 `IMPORT`），这条用例的「第二趟复用等于整条流已判态数」会变；同一份文件里别的用例不经 `run_from_environment`，所以进程内被它污染的概率低，但「点名单跑与全量判定相同」这条在带环境变量的调用下不成立（推的，没跑，checker 档重型）。
上游：「不依赖跑序，不读别的用例留下的状态：文件、全局变量、环境变量、临时目录。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:12`）。
会推翻这条的现象：这条用例在 `SINGLEFS_CRASH_AMPLIFICATION_SHARE=1/2` 之类的环境下仍绿，或它在开头清掉了这些变量（读得到的代码没有）。

### 3.3　新用例归哪一档——放对，附一处「内联单测不在快档」
归类判据：「归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档。」（`.claude/rules/verification.md:16`）；「拿不准的放 checker 档，再由代码三方判要不要挪回。」（`.claude/rules/verification.md:16`）。
- 种子复现的随机历史六条与 `history.rs` 两条单测：不枚举崩溃状态，只生成历史，归 harness 档，实际在 `crates/singlefs-harness`，对。耗时的取样点用例原本就标 `#[ignore = "harness 耗时用例…"]`（`random_histories.rs:325`），新自证不标，是轻用例，对。
- 崩溃注入两条重放行自证：只生成历史、不跑崩溃状态，按判据可以归 harness 档，但它们测的函数在 `singlefs-checker-tier` 的 `crash_injection.rs`，harness 档不许依赖 checker 档（`.claude/rules/verification.md@定义与名字` 同一节第二段），放在 checker 档的 `crash_injection_campaign.rs` 里是唯一可放处；它们不标 `#[ignore]`，在快档里跑到。对。
- `gpu_adapter_probe_…`：纯函数，不依赖特性，在快档（不带 `--features`）里编进来跑到，写了「checker 档模块：无（…）」那一行（`crates/singlefs-checker-tier/tests/gpu_adapter_probe_reports_not_run_without_an_adapter_and_red_when_no_card_starts.rs:1`）；放 checker 档的原因同上（被测的 `common_gpu_adapter` 在 checker 档 `tests/` 里）。对。
- 复用对照与 C601 新用例：要 `verdict-store`（C601 那条要 `verdict-store`+`gpu`）特性，进「带特性那一趟」，不在不带特性的快档。上游：「用编译开关（特性、`cfg` 这类）分档的，快档里至少有一趟把它编进来」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:54`），门禁 54 号另起一趟，在门禁一侧，不归 crates。
- E165 读表七条单测（「fn a_pinned_table_whose_rows_carry_the_whole_ledger_row_parses_every_row() {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6906`）在 `src/bin/` 的 `mod tests` 里。这一类不在快档：「装置二进制 `src/bin/` 的内联单测不在快档里，归它们的变异表与实验复跑」（`.claude/rules/verification.md:33`）。上游要求自证接进门禁或快档（「测试脚本自己也要被测试证明有效」那一节）；这七条经变异表被门禁 checker-tier-crates-mutation-replay 跑到（`crates/mutations.tsv:1573` 一行点名 `-p singlefs-checker-tier --bin e165_replay_bounded_enumeration`），属「接进门禁」，但不是「改了判法立刻能跑的快档」；读表是纯函数，小输入，若要进快档要把 `parse_the_pinned_table` 挪到库里。记「放在可接受处，没有进快档」，归不归主 agent 定。
- 挂钟进度的两条（`crates/singlefs-checker-tier/src/crash.rs:4671`）在库的 `mod` 里，`--lib` 快档跑到，对。

## 四、命令输出（这一次真跑的原样）

K4 的几条 grep（草稿目录 `/tmp/claude-1000/test-script-align-code-r1-sonnet/cmd-output-k4.txt`，补丁新增行抽出来的 `added-*.txt`）：
```
$ wc -l added-*.txt | tail -1
  2843 total
$ grep -cE 'unwrap\(\)|\bas (u8|u16|u32|u64|usize|i32|i64)\b|^\s*_ =>' added-*.txt     # 四份各 0
$ grep -cE '_ =>' added-*.txt                                                           # 四份各 0
$ grep -nE '\b(let|for|mut|fn)\s+\(?[a-z]\b|\|[a-z]\||\b(cfg|ctx|msg|cmd|idx|cnt|buf|tmp|val|res|ret|req|resp)\b' added-*.txt | grep -vE '#\[cfg|cfg\(|//' | wc -l
0
```
模型目录 `research/prompts/test-script-align-code-r1-sonnet-model/` 的 `rerun.sh`（从仓根起，探针编译在 `/tmp/claude-1000/test-script-align-code-r1-sonnet/probe-build/`，经 `run-with-memory-cap.sh` 跑）连跑两遍输出逐字节相同，`rerun-output.txt` 即其中一遍，`SHA256SUMS` 同目录：
```
$ sha256sum -c SHA256SUMS
```
（贴在报告末尾「交回前核」一节。）

## 五、复核不了的

- 任何会红与否的断言要 GPU 或 checker 档重型测试才量得出（1.4 的 C601 对拍、1.5 的复用键漏一项、3.2 的环境依赖、3.1 的 `usable_card_count > 0` 改坏）：这轮不跑重型测试，我只读了代码、断言与变异表，没有量红。
- 没卡机器上 σ 对拍与量显存用例的真实表现：本机我没起带 `gpu` 特性的构建，1.2 的两条结论是读代码得来，不是跑出来的。
- 门禁一侧读 `NOT_RUN_NO_GPU_ADAPTER` 的行为（`.claude/gate.d/checker-tier-layer0-replay.sh`）：我只核了它用 `--nocapture` 取日志（`:724`），没跑那一格。
- 一台「PCI 上有 NVIDIA 卡、Vulkan 驱动坏」的机器：没有，1.1 的冲突是从函数签名与探针推的。
- K2、K3 的格不是我的（攻方腿），变异行锚点与点名测试跑不跑得到我没核。

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 C620 | 冲突 | `decide_gpu_test_start` 只有 Vulkan 枚举与起卡数两个输入，没有 PCI 一项；`(&[],0)` 恒是本次未跑，C620 题面那件事还在（探针 1 case A、探针 2 零命中） |
| K1 C604 | 部分一致 | 三份文件走新入口、没卡打 NOT_RUN 返回；σ 对拍仍 `.expect` panic（`record_checker_judges_absence_by_the_persisted_set.rs:1084`），量显存 `#[ignore]` 用例零卡静默通过（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1614`），库内两条 `#[ignore]` 计时用例 panic |
| K1 C608 | 转 K3 | 只确认被点名用例没卡时打 NOT_RUN；σ 那条会 panic 不会记本次未跑 |
| K1 C601 | 一致 | 输入造出了「一个落点 33 次写、低 32 位相同只差高位」的版本对并先在表上断言；会不会红要 GPU，复核不了 |
| K1 C605 | 一致 | 三份判定逐态比（开复用、空库、整流直判），A 红 B 绿、旧判法版本红；没加开关，用空库与整流直判代替 |
| K1 C606 | 一致 | 展开上限写错 panic 带变量名与原值，三个调用点都换；GPU 对拍用例的 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO` 仍悄悄退回默认；SHARE 一半在脚本侧 |
| K1 每项一行 | 一致（附设计差异） | 判了、红了、复用、用时同在一行；被叫停、导入、只排计划三处行内写 `states_reused=0`，按项求和不错，单看是「写 0」 |
| K1 C607 | 大体一致 | 复现行与入口共用一张表、六加二条自证；`run_history_campaign` 装配 `sampling`（`history.rs:5067`）无自证无变异；另两个随机装置的重放行没动 |
| K1 C611 | 大体一致 | 一个常量三个发射点、假时钟自证；站点那一臂的打印没有用例核 |
| K4 名字 | 大体一致 | 三处可议：`a_line_was_printed_at`、`.0` 元组字段、`ALL` |
| K4 分支 | 一致 | 新增行零 `_ =>`，`wgpu::DeviceType` 五个成员逐个列 |
| K4 错误 | 一致，附不统一 | 零 `unwrap()`；复现入口走 `Result`，展开上限走 panic，两处口径不一 |
| K4 魔法数字 | 三处不合（小） | `0x10DE` 两处定义；组合数 `5 * 2 * 4 * 2` 手写；大档默认值字面量 |
| K4 变量表 | 一致，附手抄 | `HistorySamplingVariable` 一处定义两个装置共用；`GenerationWeights::NAMED` 手抄五个常量 |
| K5 GPU 没卡 | 部分一致 | 见 K1 C604；自证没喂绿、一个用例做两个场景 |
| K5 原子化 | 一处不合 | 驱动行字段用例改进程环境且读调用方环境，不还原的路径无守卫 |
| K5 档位 | 放对 | 种子复现在 harness 档、崩溃注入重放与 GPU 探针在 checker 档快档、E165 读表单测在 `src/bin/`（不在快档，经变异整表跑） |

## 七、没做什么

- 不判 K2（攻方造反例）、K3（变异表锚点与点名测试），也没替主 agent 采纳或出判决；上面「推翻它的现象」是给复核的人用的。
- 没跑重型测试（checker 档包的测试、GPU、`checker-tier-layer0-replay`、crates 变异整表）；没动主仓 `crates/`，没读主仓 `crates/` 的任何文件（引用路径都在冻结副本里；`.claude/gate.d/checker-tier-layer0-replay.sh`、`research/scripts/crash-amplification.sh` 是仓里非 `crates/` 的文件，按需读过）。
- 没核补丁原文与冻结副本是否逐字一致（读的是冻结副本；新增行抽取是从补丁抽的，两份我只在行号上对过，没做逐字节比对）。
- 没读四份实现员报告（`/tmp/claude-1000/impl-*/patch/report.md`）。
- 没量红：1.4、1.5、3.1、3.2 里的「会不会红」都是读代码推的。

## 八、交回前核
```
$ cd research/prompts/test-script-align-code-r1-sonnet-model && sha256sum -c SHA256SUMS
common_gpu_adapter.rs: OK
gpu_adapter_decision_probe.rs: OK
rerun.sh: OK
rerun-output.txt: OK
$ python3 research/scripts/cite-check.py research/prompts/test-script-align-code-r1-sonnet-output.md --root /tmp/claude-1000/test-script-align-code-r1/tree --root /home/fy5090/code/singlefs --background research/prompts/_test-script-align-code-r1-background.md
  ✓ 核了 53 处引文，对上 53 处，没判 0 处（逐处列在上面）
```
探针编译目录 `/tmp/claude-1000/test-script-align-code-r1-sonnet/probe-build/` 已删；草稿目录里留的是新增行抽取与几份命令输出（`added-*.txt`、`cmd-output-k4.txt`、`rerun-*.txt`），没有编译目录与仓副本。
