# test-script-align-code-r3 正推腿报告（Sonnet，格 L2、L3、L5，2026-10-02）

读冻结副本 `/tmp/claude-1000/test-script-align-code-r3/tree/`（下文路径都是副本里的路径，引文用 `--root` 指它核）。模型目录 `research/prompts/test-script-align-code-r3-sonnet-model/`（`SHA256SUMS`、`rerun.sh`、`outputs/`）。立场：从已定条款推「应该是什么」，再核代码做没做到；不判 L1、L4。

## 0 从哪几条推

- 判决 `research/prompts/test-script-align-code-r1-main-verification.md:28`：「写错就 panic，带变量名与原值，与 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 同一判法」。
- 判决 `research/prompts/test-script-align-code-r2-main-verification.md:24`：「收成一份共用解析，写错一律 panic 带变量名与原值；线程数 0 的口径全仓一致」。
- 判决 `research/prompts/test-script-align-code-r1-main-verification.md:32`：「复现入口走 `Result`、展开上限走 panic 是两种调用方（入口报给人、库内不变量），不统一」。这一条第一轮判的是不采；第三轮前主 agent 另定了崩溃注入重放入口全 panic（实现员报告 `/tmp/claude-1000/impl-pre-r3-2/report.md` 第 1 条，转述），所以下文把它当作现状，不当作条款。
- C610（`.claude/kb/checks-owed.md:479`）：「一格门禁按结构判、不按词表：测试文件头的文档注释里一行带日期、同一行没有 kb 指路就判红」。
- 上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md:14`：「用例名写场景和预期（`code-discipline.md` 的「名字」），不带里程碑、步号、批次这类来历；来历进 commit message 或项目 kb。」
- `.claude/rules/verification.md:25`：「慢档用例：debug 下单条跑到 60 秒及以上的用例，标 `#[ignore = "harness 慢档用例：…"]`」。

## 1 方法与能复跑的命令

- 探针只调库的公开函数、不起 checker 档测试二进制：`probe/src/main.rs`（解析入口的结局）、`probe/src/bin/closure.rs`（复现 `crash_identity` 那条依赖断言的取数，对冻结副本与两份改过一行的副本各跑一遍）、`silent/gen.sh`（从 E161/E165/E168 原样抽三份线程数解析函数）。
- 整套复跑：`bash research/prompts/test-script-align-code-r3-sonnet-model/rerun.sh`（我跑了一遍，退 0；输出落 `outputs/`，与我第一次手跑的逐字相同：`probe-l2.out` 与 `silent.out` 各 `diff` 一次无差）。命令都经 `run-with-memory-cap.sh 8G`（峰值表里已有记录之后写了 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`）与 `capped.sh 8`，`nice -n 19`。
- 开跑前 `ps` 没有 `qemu-system`、`vm-bench`、`e152`、`fio`、别的 cargo。
- 没有改冻结副本：探针靠路径依赖编它，目标目录在草稿目录；改一行的两份副本拷在草稿目录里，跑完删。

## 2 L2　数值环境变量解析收成一份之后

### 2.1 共用解析本体与全部走它的调用方：一致

条款推出来该是：写错、写 0（线程数）、空串、全空白、非 UTF-8 都 panic 且带变量名与原值；前后空白去掉再认；没设是 `None` 交调用方取默认。

- 本体：`crates/singlefs-harness/src/numeric_environment_variable.rs:51`：「match text.trim().parse::<Number>() {」，认不出的 `panic!` 带 `{variable}={text:?}`；`:50`：「let text = value?;」没设是 `None`。
- 读进程环境：`crates/singlefs-harness/src/numeric_environment_variable.rs:30`：「panic!("{variable} 的值不是 UTF-8：{raw:?}；不当成没设")」。
- 探针 A（同一取值喂两种解析）与 E（经三个公开入口读真环境变量），原样输出在 `outputs/probe-l2.out`：
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:46`：「E value="0" 崩溃注入 = PANIC SINGLEFS_CRASH_INJECTION_THREADS="0" 认不出」
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:47`：「E value="0" 故障注入 = PANIC SINGLEFS_FAULT_INJECTION_WORKER_THREADS="0" 认不出」
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:48`：「E value="" 崩溃注入 = PANIC SINGLEFS_CRASH_INJECTION_THREADS="" 认不出」
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:50`：「E value=" 4" 崩溃注入 = OK 4」
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:43`：「D CrashInjectionWorkerThreads::from_the_environment = PANIC SINGLEFS_CRASH_INJECTION_THREADS 的值不是 UTF-8」
- 走共用解析的 Rust 调用方（`grep -rnE 'number_from_the_variable_value_panicking_when_unreadable|process_environment_variable' crates` 数出）：`crash_amplification.rs`（展开上限、核对线程数）、`crash.rs`（层 0 线程数）、`crash_injection.rs`（线程数与重放入口四个数）、`fault_injection.rs`、`tests/random_histories.rs`、`tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`、`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`tests/record_checker_judges_absence_by_the_persisted_set.rs`。写 0 的线程数在这几处都是 panic。

推翻它的现象：任一上述调用方对「写错 / 写 0 / 空串」给出非 panic 的结局，或探针 E 里某行不是 PANIC。

### 2.2 前后空白：同一入口里两种口径（打中 L2「口径两处不一」，响的方向，不静默）

- 共用解析去前后空白（上面 `:51`）；`history` 的 `Result` 版不去：`crates/singlefs-harness/src/history.rs:4095`：「text.parse()」。
- 崩溃注入重放入口 `crash_injection_campaign_from_environment`（`crates/singlefs-checker-tier/src/crash_injection.rs:2464`：「pub fn crash_injection_campaign_from_environment(」）把种子起点、种子数、崩溃点数、短段写数交共用解析，每段步数、比重、checker、盘宽、空间准入交 `HistorySampling::from_environment`（`:2503`），同一次调用里两种口径。探针 B：
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:27`：「B SINGLEFS_CRASH_INJECTION_FIRST_SEED=" 11" -> OK (11, 2, 3」
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:30`：「B SINGLEFS_CRASH_INJECTION_OPERATIONS=" 24" -> PANIC SINGLEFS_CRASH_INJECTION_OPERATIONS= 24：只认 非负整数」
- 同名同义的变量跨入口也不一：坏盘输入 `crates/singlefs-checker-tier/src/bad_disk_input.rs:905`：「let first_seed = number_from_reproduction_environment(」走 `Result` 版，探针 C：
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:37`：「C SINGLEFS_BAD_DISK_FIRST_SEED=" 11" -> OK Err("SINGLEFS_BAD_DISK_FIRST_SEED= 11：只认 非负整数")」
- 故障注入大档同样走 `Result` 版：`crates/singlefs-harness/tests/fault_injection_fast_tier.rs:133`：「let first_seed = number_from_reproduction_environment(」；随机历史大档的种子起点走共用解析（`crates/singlefs-harness/tests/random_histories.rs:1653`：「let first_seed = number_from_environment(」），而「收缩一个种子」那条入口的种子走 `Result` 版（`crates/singlefs-harness/src/history.rs:4117`：「Some(_given) => number_from_reproduction_environment(lookup, variable, 0),」）。
- `Result` 版的渲染不给值加引号，前导空白看不出来：`research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:9`：「A lead-space     history::number_from_reproduction_environment(Result版) = OK Err("V= 8：只认 非负整数")」。人看到「= 8：只认非负整数」会摸不着头脑；共用解析的消息用 `{text:?}`，空白看得见（`outputs/probe-l2.out:31`）。
- 严重度：两种结局都不是静默放过（一边接受并去空白，一边响着拒绝），不会让红的变绿。算「口径两处不一」，不算「该 panic 不 panic」。
- 推翻它的现象：探针 B 第 27、30 行不是上面那样；或 `history.rs:4095` 改成先 `trim()`。

### 2.3 非 UTF-8：数值入口都 panic，但还有一份没收、三处消息写法

- 经共用读法的入口都 panic（探针 D，`outputs/probe-l2.out:42`、`:43`）。
- 故障注入线程数还留着自己的一臂，没经 `process_environment_variable`：`crates/singlefs-harness/src/fault_injection.rs:2615`：「panic!("{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE} 不是 UTF-8：{raw:?}")」，与共用那一句（`numeric_environment_variable.rs:30`）是同一件事写了两遍；调用处 `crates/singlefs-harness/src/fault_injection.rs:2596`：「std::env::var(FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE),」。结局同是 panic，消息不同：`research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:44`：「D FaultInjectionWorkerThreads::from_the_environment = PANIC SINGLEFS_FAULT_INJECTION_WORKER_THREADS 不是 UTF-8」。「收成一份」在这一处没收完，不改变结局。
- 不在共用解析名下、同一条流水线里的三处把非 UTF-8 当没设（`Err(_)` / `.ok()`）：
  - `crates/singlefs-checker-tier/src/crash_amplification.rs:3927`：「Err(_) => HostShare { index: 0, count: 1 },」（`SINGLEFS_CRASH_AMPLIFICATION_SHARE`，写成非 UTF-8 就悄悄不分片）；
  - `crates/singlefs-checker-tier/src/crash_amplification.rs:3949`：「Err(_) => vec![1; share.count],」（主机权重悄悄取等权）；
  - `crates/singlefs-checker-tier/src/crash_amplification.rs:3998`：「let import_list = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_IMPORT").ok();」（悄悄不导入）。
  - 方向都保守（多判、不少判），不会让红变绿；它们的值是「i/n」「w1,w2」「路径表」，不是单个数，不在共用解析的射程里，列为线索。
- 三份装置在 L5 第 4 条里量。

### 2.4 写 0：线程数全仓 panic，规模变量全仓照收 0（口径一致，留一条没人拦的空跑）

- 线程数（探针 E、E161/E165/E168 的 `silent.out` 第 7–9 行）写 0 都 panic。
- 规模变量（种子数、崩溃点数、每段步数）经共用解析是 `u64` / `usize`，写 0 照收；`Result` 版同样收 0（`research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:18`：「A zero           history::number_from_reproduction_environment(Result版) = OK Ok(0)」）。两条路一致。
- 空跑：崩溃注入大档默认只判「清单外失败」——`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:187`：「CrashInjectionAssertions::NewFindingsOnly => {}」，`match` 之前只断言 `new_findings.is_empty()`（`:181` 起），`NewFindingsOnly` 一臂是空的（`:188` 起各臂另判）。所以 `SINGLEFS_CRASH_INJECTION_SEEDS=0` 会跑 0 段、判 0 个崩溃状态、报绿。推的，没跑（要起 checker 档测试二进制，不在我的许可里）；不是第二、三轮引入的，第二轮之前就收 0。记作线索，不判。
- 推翻它的现象：探针 E 或 `silent.out` 里某个线程数变量写 0 不 panic；或崩溃注入大档跑 0 段时有断言拦住。

### 2.5 复现行喂回入口（C607、S2）有没有因为全 panic 丢了「写错报给人看」那一层：没丢

- 第二轮之后的旧样子：入口返回 `Result`，唯一的调用方 `large_tier_campaign_from` 马上把 `Err` 渲染成 panic。证据：`research/prompts/test-script-align-code-r3-snapshot/r2-to-r3.diff:2502`：「-    .unwrap_or_else(|error| panic!("{}", error.render()))」（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs` 的 `large_tier_campaign_from` 里删掉的那一行）。
- 今天：同样的渲染挪进库函数：`crates/singlefs-checker-tier/src/crash_injection.rs:2502`：「.unwrap_or_else(|unrecognized| panic!("{}", unrecognized.render()));」（断言组）与 `:2512` 同一句（取样参数）；数值那几个经共用解析 panic，带变量名与原值。调用方只剩 `crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1006`：「fn large_tier_campaign_from(lookup: &dyn Fn(&str) -> Option<String>) -> CrashInjectionCampaign {」。
- 报给人的内容没丢：断言组与盘宽写错的消息里仍有变量名、原值、认得的取值（探针 B 第 35 行：`research/prompts/test-script-align-code-r3-sonnet-model/outputs/probe-l2.out:35`：「B SINGLEFS_CRASH_INJECTION_ASSERTIONS="" -> PANIC SINGLEFS_CRASH_INJECTION_ASSERTIONS=：只认 new-findings、small-fast-tier、fast-tier、rollback-sampling」）；库内用例改成 `#[should_panic(expected = …)]` 钉这一句。
- 变了的只有两样：一，原来能断言 `Err` 的结构（变量、值、认得的列表三个字段），现在只能断言消息里含一段；二，崩溃注入这一个入口变成 panic，另三个入口（坏盘输入 `bad_disk_input.rs:905`、故障注入 `fault_injection_fast_tier.rs:133`、随机历史 `history.rs:4117` 那一路）仍返回 `Result`，各自的调用方都在末尾渲染成 panic（`tests/bad_disk_input_campaign.rs:1044`、`tests/fault_injection_fast_tier.rs:337`、`tests/random_histories.rs:1664` 与 `:1710`：每处都是「.unwrap_or_else(|error| panic!("{}", error.render()))」）。两种形态并存；结局一致（都 panic、都带变量与值）。
- 推翻它的现象：找到一个入口把 `ReproductionEnvironmentError` 吞掉（`.ok()`、`unwrap_or(默认)`）；上面四处调用方我逐个看了末尾，没有。

### 2.6 依赖判法在新模块下照样成立（且断言有牙）

- 断言：`crates/singlefs-checker-tier/src/crash_identity.rs:1445`：「.get("crates/singlefs-harness/src/history.rs")」，`:1448`：「reason.starts_with("referenced by crates/singlefs-checker-tier/tests/"),」。用例 `:1392`：「fn every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries(」。
- 复现它的取数（`closure` 探针，对每条登记的崩溃枚举用例调 `judging_code_by_module_references`）：冻结副本上 8 条登记用例，`guard-violations: 0`，新模块都在闭包里：`research/prompts/test-script-align-code-r3-sonnet-model/outputs/closure-base.out:73`：「guard-violations: 0」。
- 有牙：把 `crash_amplification.rs:77` 改成经 `singlefs_harness::history::` 引（正是第二轮追加的那条变异），三条用例的 history.rs 理由变成「referenced by crates/singlefs-checker-tier/src/crash_amplification.rs」，断言会红：`research/prompts/test-script-align-code-r3-sonnet-model/outputs/closure-mut-crash-amplification-via-history.out:10`：「guard-violations: 3」。这是用库函数复现取数，不是真起那条测试；真起归提交时 checker 档。
- 新模块不引 `history`（`numeric_environment_variable.rs` 全文只有 `std`），所以它被收进闭包不会带进脚手架；`history.rs` 只在 `:4290`：「pub use crate::numeric_environment_variable::process_environment_variable;」单向转出。
- 盲区（L5 第 2 条展开）：这条断言只看 history.rs 的收录理由，而测试文件自己直接引 history 的目标，理由恒为「referenced by …/tests/…」，断言对它们判不出。
- 推翻它的现象：`closure-base.out` 里有 `guard_ok=false`；或把 `numeric_environment_variable.rs` 加一行 `use crate::history::…` 后三条用例不红（我没加这个变异，只用上面那条等价的）。

## 3 L3　C610 回扫与术语改名

### 3.1 删掉的来历里有没有删掉「文件测什么」的现状句或指到 kb 的指路：没有

做法：实现员的替换规格（`/tmp/claude-1000/impl-pre-r3/c610-spec.json`，83 处旧串 / 新串）逐处对；再按「旧串里有、新串里没有的指路」扫一遍（正则：`[DECI]\d{2,4}`、`kb/`、`records/`、`research/`、`.md`、反引号片段）。

- 83 处在冻结副本里：新串 83 处都在、旧串 83 处都不在（脚本核，`problems 0`）。
- 删掉的全是「用户 / 主 agent YYYY-MM-DD 定」「写于」「实测」「同日定」这类半句；D / E / C / I 编号、已定项、判决与报告路径都留着。
- 指路被删的只有两处，都是 `/tmp` 下的规格路径，换成了仓里存在的实现员报告：`/tmp/claude-1000/impl-rev-a3c/spec.md`、`/tmp/claude-1000/impl-rev-a3b/spec.md` 换成 `research/prompts/m2-rev-a3c-implementer-report.md`、`research/prompts/m2-rev-a3b-implementer-report.md`（`ls` 两份都在，`git log` 见 `dbcc0d2f`）。
- 两处删得略多、不算现状：`crash_injection_campaign.rs` 头上删了「定案第 7 条」的出处（剩下「随机说的是一个测试周期与下一个之间，不是每次跑重抽」这句现状还在）；`tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 头上删了「立不变量并同步」（I-9.16 的指路还在）。
- 只动了 `//!`：76 份「改前」副本（`/tmp/claude-1000/impl-pre-r3/before/`）逐份比非 `//!` 行（先把 `harness 耗时用例` 换成 `harness 慢档用例`），只有 12 份有差，正好是实现员报告里写明动过代码的 12 份（`crash_injection_campaign.rs`、两份 GPU 对拍、`gpu_adapter_probe_…`、`crash_amplification.rs`、`gpu_unit_checks.rs`、`crash_injection.rs`、`random_histories.rs`、`entries_after_mount_refuse_…`、`fault_injection.rs`、`history.rs`、`lib.rs`）；其余 64 份逐字相同。
- 推翻它的现象：在 83 处里找到一处旧串带 `D\d+` / `C\d+` / kb 路径、新串里没有（我的扫描只报了上面两处 `/tmp` 路径）。

### 3.2 剩下 7 行带日期：都是路径名的一部分

`grep -nE '^//!.*[0-9]{4}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs` 在冻结副本里 7 行（`grep -c` 数文件 7 份）：

- `crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs:10`：「records/2026-09-22-panic面普查-走得到的那些.md」
- `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs:6`：「closeout-recheck-2026-09-28/c393-investigator-report.md」
- `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:2`：「e142-new-pool-file-creation-dry-run-2026-09-16-instance-boundary.out」
- `crates/singlefs-harness/tests/recovery_reads_the_created_file_after_reopen.rs:2`：「e142-new-pool-file-creation-dry-run-2026-09-16-instance-boundary.out」
- `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:1`：「records/2026-09-24-里程碑二收尾调度.md」
- `crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs:2`：「records/2026-09-27-代码审阅38条去向.md」
- `crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs:5`：「closeout-recheck-2026-09-28/row43-investigator-report.md」

别的日期写法（`N 月 N 日`、`2026/09`、`2026.09`、`09-24`）在 `crates/*/tests/*.rs` 的 `//!` 里零命中（同一次 `grep -nE` 的多式样，只命中上面 7 行里路径名那几处）。

### 3.3 线索（不判，不是判据里的两种）

- C610 回扫的 `grep` 通配是 `crates/*/tests/*.rs`，管不到测试子目录里的辅助模块。`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:2`：「别的流直接复用（用户 2026-09-28 定：前缀共享是剪枝的基础，没有它每次放量都从头核）。」是同一类来历，还在。C610 的门禁按结构判（`.claude/kb/checks-owed.md:479`：「测试文件头的文档注释里一行带日期、同一行没有 kb 指路就判红」），要是那格门禁只扫 `crates/*/tests/*.rs`，这一行它也看不见。
- 没带日期的来历半句还在 8 行（`grep -nE '^//!.*(用户|主 agent)[^，。]{0,12}(定|认|嘱咐)' crates/*/tests/*.rs | wc -l` 数出 8）：例如 `crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs:3` 的「主 agent 定走「拒」」。C610 按日期判，这几行在门禁外；实现员报告也写了没扫。
- 一处 `/tmp` 路径还留着：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:16`：「合入后验证一（规格 `/tmp/claude-1000/impl-merge-verify-1/spec.md`）改了三样：」；`/tmp` 下的文件会被清掉，这是个会悬空的指路，不带日期。

### 3.4 `#[ignore = "harness 慢档用例：…"]` 字面认的脚本与门禁：全认新串，没有一处还按旧串认

- 冻结副本里 harness 档四个包 + checker 包的 `#[ignore` 属性行 64 条（`grep -rE '^\s*#\[ignore' …| wc -l`）；理由以新串开头的 61 条，`#[ignore = "harness 慢档用例：` 的 `grep` 数 61；其余 3 条是 `research/scripts/broad-mutation-ignored-exclusions.tsv` 排除表里的三条大档 / 工具（随机历史两条、故障注入一条）。61 + 3 = 64，不多不少。
- 全仓按字面认这个理由前缀的只有 `research/scripts/broad-mutation.py:287`：「TIMED_CASE_REASON_PREFIX = "harness 慢档用例："」，分类函数 `:1401` 用 `startswith` 它。它的自证样本里的 `#[ignore = "harness 慢档用例：…"]`（`:2843`、`:2845`）也是新串。
- 别的脚本与门禁不看理由文字，只认 `#[ignore`：`research/scripts/crash-case-check.py:182`、`.claude/gate.d/code-source-discipline.sh:1091`：「if found_tests and all("#[ignore" in attributes for attributes, _full_name in found_tests):」。
- 旧串 `耗时用例`、`轻用例`、`harness 耗时` 在 `crates/`、`research/scripts/`、`.claude/gate.d/`（含样本）、`.claude/rules/` 里零命中（`grep -rnE` 现查）；只剩 `.claude/kb/term-renames.md` 与 `.claude/kb/decisions-history.md` 里作为改名记录。`research/prompts/test-script-align-code-r3-snapshot/r2-to-r3.diff` 里 61 处旧串对 61 处新串（`grep -c '^-.*harness 耗时用例'` 与 `'^+.*harness 慢档用例'` 各 61）。
- 一处无害的怪字：六份样本 `.claude/gate.d/fixtures/code-source-discipline.sh/*/crates/demo/tests/heavy.rs` 第 2 行是 `#[ignore = "样本：慢档用例用例"]`，「用例用例」是 HEAD 上「耗时用例用例」留下来的，没有任何脚本按它认。
- 没做：`broad-mutation.py --selftest` 与 `--check-ignored`（前者起样本、后者读库目录里的专用树，不是冻结副本），所以「自证仍绿」不是我复核的；我只核了字面对得上。
- 推翻它的现象：在 `research/scripts/`、`.claude/gate.d/` 里找到一处正则或 `startswith` 认 `耗时用例` / `轻用例`；或冻结副本里有一条 harness 档 `#[ignore` 理由既不以新串开头、又不在排除表里。

## 4 L5　交来的四样

条款推出来该是：每一样留下的东西，不得让「写错 / 该红」的输入绿着过去；绿着过去的要有可见的原因（日志、消息）。四样逐个量。

### 4.1 取样参数的每段步数仍走 `history` 的 `Result` 版解析：不会让红变绿

- 它走的路：`crates/singlefs-harness/src/history.rs:4087`：「pub fn number_from_reproduction_environment(」，认不出交 `Err(Unrecognized)`；四个调用方末尾都渲染成 panic（见 2.5）。没有一处吞掉。
- 与共用解析的差只有两样：不去前后空白（2.2）、消息不加引号。探针 B 第 30、34 行：写成 `" 24"`、`""` 都停下。
- 结论：口径问题，不是正确性问题。
- 推翻它的现象：找到一个调用方对 `Err` 取 `.ok()` / 默认。

### 4.2 `history.rs` 留一行 `pub use`：不会让红变绿；但它让一条登记的崩溃枚举用例的判法闭包多收 5 份文件

- `crates/singlefs-harness/src/history.rs:4290`：「pub use crate::numeric_environment_variable::process_environment_variable;」；经它引的测试文件里，有一份是登记的崩溃枚举用例 `crash-case:c561-sigma-full`（`.claude/gate.d/stage-inputs.tsv:41`：「crash-case:c561-sigma-full」）的入口：`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1117`：「singlefs_harness::history::process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE)」。这是该文件对 `history` 的唯一引用（`grep -n 'singlefs_harness::history'` 只此一行；别的 `use` 是 `memory_pool`、`segments` 与 crate 根上的类型）。
- 量：把这一行改引 `numeric_environment_variable`（草稿副本，改一行），闭包从 63 份降到 58 份，少的正是 `fault_injection.rs`、`history.rs`、`model.rs`、`model_comparison.rs`、`scenario.rs`：
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/closure-diff-record-checker.txt:4`：「< F crates/singlefs-harness/src/history.rs」
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/closure-base.out:68`：「T record_checker_judges_absence_by_the_persisted_set: files=63」
  - `research/prompts/test-script-align-code-r3-sonnet-model/outputs/closure-mut2-record-checker-direct.out:63`：「T record_checker_judges_absence_by_the_persisted_set: files=58」
- 为什么依赖断言放过它：断言只拒「经 impl 头带进来」，测试文件自己直接引的理由恒为「referenced by …/tests/…」（`research/prompts/test-script-align-code-r3-sonnet-model/outputs/closure-base.out:68` 的 `history_reason`）。所以这条用例的判法摘要现在含随机历史脚手架，`history.rs` 一改，`c561-sigma-full` 的判定复用键就变、整格重判。
- 方向：多收文件只会多重判，不会少重判；不是「该红的绿着过去」。它违背的是 `crash_identity.rs:1392` 那条用例要拦的意图（E164 W5 量到的「崩溃枚举节点 20 个提交全部重跑」，见该用例上方文档），只是被字面放过。
- 另五份经 `history::` 引的文件（`tests/bad_disk_input_campaign.rs:40`、`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:48`、`tests/random_histories.rs`、`tests/fault_injection_fast_tier.rs`、`tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`）都不是登记的崩溃枚举用例（探针里 8 条登记目标没有它们），不受影响。
- 结论：不是正确性问题（不放过红）；是复用键变宽的维护问题，有量。一行改动（`:1117` 改引新模块）就能去掉这五份；改完 `pub use` 才能删。
- 推翻它的现象：`stage-inputs.tsv` 里 c561 的入口不是这个测试目标；或我改一行的副本上 `closure` 探针的文件数不是 58。

### 4.3 `numeric_environment_variable` 模块装了读非数值变量原文的函数：不会让红变绿，只是名字

- 读原文的函数 `crates/singlefs-harness/src/numeric_environment_variable.rs:14`：「pub fn process_environment_variable(variable: &str) -> Option<String> {」被不少调用方拿来读取值是名字的变量（`random_histories.rs:1665` 读 `SHRINK`、重放入口读断言组名）。行为只有「没设 / 读到 / 非 UTF-8 panic」，与名字里的「数值」无关。
- 条款上没有「模块名要和内容相符」那一条，上游 `code-discipline.md`「名字」那一节要的是名字说得出它是什么；模块文档已补一句（`crates/singlefs-harness/src/numeric_environment_variable.rs:5`：「变量的原文从进程环境怎么读（[`process_environment_variable`]）也只有这里一份」）。风格，不是正确性。

### 4.4 E158 / E161 / E165 / E168 装置与 `layer0_progress.rs` 同形副本没改：没有会让红变绿的，有一处口径不一

量法：`silent/gen.sh` 从三份装置原样抽线程数解析（用 `grep -nF` 核函数头行号，再抽到下一个顶格 `}`），喂非 UTF-8、`" 4"`、`"0"`、`""`、没设。原样输出 `outputs/silent.out`：

- `research/prompts/test-script-align-code-r3-sonnet-model/outputs/silent.out:1`：「E161 non-utf8   -> OK 32」
- `research/prompts/test-script-align-code-r3-sonnet-model/outputs/silent.out:2`：「E165 non-utf8   -> OK 32」
- `research/prompts/test-script-align-code-r3-sonnet-model/outputs/silent.out:3`：「E168 non-utf8   (装置调用处是 std::env::var(..).ok()：非 UTF-8 先变 None) -> OK 7」
- `outputs/silent.out` 第 4–6 行：`" 4"` 三份都 PANIC（共用解析接受、装置不接受）；第 7–12 行：`"0"`、`""` 都 PANIC。

源码位置：`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:2644`：「Err(_not_set) => {」；`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7447`：「if let Ok(text) = env::var(variable) {」；`crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:621`：「let read = |variable: &str| std::env::var(variable).ok();」。E158 的 `parse_local_system_configuration_bytes` 非 UTF-8 也 panic（`Err(error) => panic!`），不属于这一类。

- 这三份是线程数变量：E168 自己的文档说「线程数不进产物：产物与线程数无关」，E161、E165 同理（线程数只影响跑多快）。非 UTF-8 悄悄退回默认，不会改判定，不会让红变绿。
- 它是 C606 那一类（「写错的规模悄悄退回默认」，`research/prompts/test-script-align-code-r1-main-verification.md:28`）剩下的一处：三份装置对非 UTF-8 与共用解析口径不一，对前后空白也不一（装置不去空白）。
- `crates/singlefs-checker-tier/src/layer0_progress.rs:361`：「panic!("{LAYER0_SHARD_ENVIRONMENT_VARIABLE} 读到的不是 UTF-8：{raw:?}")」；`:375` 进度目录同样 panic。四个变量走各自的 `Result` 读法，非 UTF-8 一律 panic；分片写法 `i/n` 不是单个数，不在这一类。
- 推翻它的现象：找到一个装置变量，其值会改变产物行（不只是线程数），而装置对它的非 UTF-8 / 空串悄悄取默认。我只看了 E161、E165、E168 的线程数与 E158 的几处，没有逐变量扫全部装置。

### 4.5 哪一样其实是正确性问题

四样里没有一样会让该红的东西绿着过去。最接近「正确性」的是 4.4（C606 同类、非 UTF-8 静默退默认，影响只到线程数）；对判法复用键有实测影响的是 4.2（多收 5 份，方向是多重判）。其余两样是口径与名字。以上结论的支点是：每个把 `Err` 吞掉才会变绿的路径我都没找到——我读过的是四个 `Result` 调用方、共用解析的八处调用方、三份装置。

## 5 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| L2 共用解析本体、走它的调用方 | 一致 | 写错、写 0（线程数）、空串、全空白、非 UTF-8 都 panic 带变量名与原值，前后空白去掉再认；探针 A、D、E 对上 |
| L2 前后空白口径 | 打中（口径两处不一，响的方向） | 共用解析去空白、`history` 的 `Result` 版不去；同一个崩溃注入入口里 `FIRST_SEED=" 11"` 收、`OPERATIONS=" 24"` 拒（探针 B），坏盘输入与故障注入的种子拒（探针 C）；不静默 |
| L2 非 UTF-8 | 部分一致 | 数值入口都 panic；`fault_injection.rs:2615` 还留一份自己的一臂（结局同、消息异）；`crash_amplification.rs` 的 SHARE / 主机权重 / IMPORT 三处悄悄当没设（方向保守，线索） |
| L2 写 0 | 一致（留线索） | 线程数全 panic；规模变量全收 0，`NewFindingsOnly` 下 0 段空跑报绿，推的没跑，非本轮引入 |
| L2 全 panic 后复现行喂回入口（C607、S2）的「报给人」层 | 没丢 | 旧调用方本来就把 `Err` 渲染成 panic（`r2-to-r3.diff:2502`），渲染挪进库函数；四个入口两种形态并存，结局一致 |
| L2 依赖判法 | 成立、有牙 | 8 条登记用例 `guard-violations: 0`；把 `crash_amplification.rs:77` 改经 `history::`，3 条红；新模块不引 `history` |
| L3 删掉的来历里有没有现状句 / 指路 | 没有 | 83 处旧串 / 新串逐处核，只有两处 `/tmp` 规格路径被删、换成仓里存在的报告；非 `//!` 行 64 份逐字相同，12 份是报告里写明动代码的 |
| L3 剩下 7 行带日期 | 都是路径名的一部分 | 3 份 `records/`、2 份 `closeout-recheck-…/`、2 份 `research/results/…-2026-09-16-…out` |
| L3 脚本与门禁认新串 | 全认、无旧串残留 | 全仓只有 `broad-mutation.py:287` 按字面认；61 + 3 = 64 条 `#[ignore`；样本里「慢档用例用例」是 HEAD 留下的怪字，没有脚本按它认；`--selftest`、`--check-ignored` 没跑 |
| L3 线索 | 不判 | `tests/common_crash_points/mod.rs:2` 的 `用户 2026-09-28 定` 在 `crates/*/tests/*.rs` 通配之外；8 行无日期的来历半句在门禁之外；`unit_area_start_…rs:16` 一个悬空 `/tmp` 路径 |
| L5 每段步数走 `Result` 版 | 不会让红变绿 | 四个调用方末尾都渲染成 panic；只剩空白口径与消息不加引号 |
| L5 `history.rs` 留 `pub use` | 不会让红变绿；有实测影响 | `record_checker_judges_absence_by_the_persisted_set.rs:1117` 经它引 `history`，登记用例 `c561-sigma-full` 的判法闭包 63 份、改引新模块 58 份，多收 `history.rs`、`model.rs`、`model_comparison.rs`、`scenario.rs`、`fault_injection.rs`；依赖断言字面放过（测试文件自己引）；方向是多重判 |
| L5 模块名 | 不会让红变绿 | 风格 |
| L5 装置同形副本 | 不会让红变绿；一处口径不一 | E161、E165、E168 非 UTF-8 悄悄取默认线程数（`silent.out:1-3`），空白不去；变量只管线程数 |
| L5 哪一样是正确性问题 | 没有 | 最近的是装置副本（C606 同类、只到线程数）；有实测影响的是 `pub use`（复用键变宽，多重判） |

## 6 没做什么

- 不判 L1、L4（攻方的格）；不替主 agent 采纳或出判决。
- 没跑 checker 档测试二进制（`cargo test -p singlefs-checker-tier`、登记的崩溃枚举用例、门禁 `checker-tier-layer0-replay`）：探针只调库的公开函数与 `judging_code_by_module_references`。主 agent 的 `r3b-libint.log` 我没读，那一格的绿红我没复核。
- 没跑 `research/scripts/broad-mutation.py` 的任何用法（`--selftest`、`--check-ignored` 也没跑）；没跑 `crash-case-check.py`、门禁 `code-source-discipline` 与 `harness-model-differential-and-scenarios` 的样本；L3 第 4 点只核了字面对得上。
- 没逐变量扫全部 E158 / E161 / E165 / E168 装置的环境变量，只看了线程数那几处与 E158 的一处。
- `SEEDS=0` 空跑报绿是读代码推的，没跑。
- 实现员报告（`/tmp/claude-1000/impl-pre-r3*/report.md`）里 `/tmp` 下的文件是转述，不是引文；`c610-spec.json` 与 `before/` 在 `/tmp/claude-1000/impl-pre-r3/`，我读它们没改。
- 没改冻结副本、主仓 `crates/`、`.claude/`；没做 git 写；没起性能测量。

## 7 草稿目录与清理

- 草稿目录 `/tmp/claude-1000/test-script-align-code-r3-sonnet/`：留着日志与输出（`build.log`、`closure-*.out`、`probe-l2.out*`、`silent*`、`rerun-out/` 里的几份输出）。
- 已删（交回前）：编译目录 `/tmp/claude-1000/test-script-align-code-r3-sonnet/target`（1023M）、`/tmp/claude-1000/test-script-align-code-r3-sonnet/rerun-out/target`（1019M）、改一行的仓副本 `/tmp/claude-1000/test-script-align-code-r3-sonnet/mut-tree` 与 `/tmp/claude-1000/test-script-align-code-r3-sonnet/mut2-tree`（各 13M）。`rerun.sh` 会重建它们并在末尾删掉两份副本，编译目录留在它的输出目录里，用完自己删。
