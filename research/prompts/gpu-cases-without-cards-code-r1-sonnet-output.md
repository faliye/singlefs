# gpu-cases-without-cards-code-r1 正推腿（Sonnet）报告

日期：2026-11-21。立场：从条款与 `crates/` 今天的实现推「应该是什么」，再与被判的三份文件逐格比。
被判：`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start.rs`、`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs`、`crates/mutations.tsv`（这一批追加的 4 行在 8623-8626 行，替换的 2102 行与删掉的 1 行归攻方腿，这一份不判）。
没起模型：这一份报告没有模型目录，没有要报 sha256 的文件；读数都来自只读命令（`grep`、读文件、`python3 -I` 读文件计数），命令原样放在「要主 agent 跑的命令」一节前后。草稿都在 `~/.local/share/singlefs/work/gpu-cases-without-cards-code-r1-sonnet/`（调用点表的原始输出 `sites.tsv`、`calls-pool-real.txt`）。
标注：「量的」= 这一趟我跑只读命令得到；「推的，没量过」= 读码推出来、没起任何测试。

## 一、两句条款，代码做的是不是它们说的

条款原文（`.claude/rules/verification.md` 第 52 行，同一段里的两句）：

- 「验证包的测试与崩溃判段不借卡：GPU 用例没交卡串的报本次未跑」（`.claude/rules/verification.md:52`）
- 「崩溃判段之外用卡的测试交 GPU 服务代跑（接口待定）」（`.claude/rules/verification.md:52`）

用户定案原话（背景材料引的两句，我在记录里现查了行号）：
- 「E202 真卡验证才能借卡 其他的拒绝。 白名单借卡」（`records/2026-11-13-全量提交门禁前收尾调度.md:1645`）
- 「除了白名单，其他一律走统一的gpuserver。没有其他的逻辑 删掉这部分无用的逻辑」（`records/2026-11-13-全量提交门禁前收尾调度.md:1647`）

### 1.1 「GPU 用例没交卡串的报本次未跑」——成立（限定在 `gpu_card_pool_or_report_not_run` 这个入口）

追到底的路（主工作区，行号现 `grep -nF` 过）：

1. 公开入口 `gpu_card_pool_or_report_not_run` 起在 `gpu_test_start.rs:863`（「pub fn gpu_card_pool_or_report_not_run(」），读的配置是 `gpu_test_start.rs:868`（「GpuConfigurationOfTheCaller::of_this_process,」），这一份在进程里只记一次：`gpu_card_configuration.rs:854`（「GPU_CONFIGURATION_OF_THE_CALLER.get_or_init(Self::read_from_the_environment_now)」）。
2. 判法在 `gpu_test_start.rs:907` 起先问交没交卡串（「let cards_of_the_caller = configuration.cards_for_the_gpu_tests_given(&|| {」）；没设、空串、只有空白都判成没交：`gpu_card_configuration.rs:742`（「environment_value.filter(|card_string| !card_string.as_ref().trim().is_empty())」），再进 `NotHandedOver`：`gpu_card_configuration.rs:950`（「None => Ok(CardsOfTheCallerForTheGpuTests::NotHandedOver {」）。
3. `NotHandedOver` 臂在 `gpu_test_start.rs:911`（「Ok(CardsOfTheCallerForTheGpuTests::NotHandedOver { reason_not_run }) => {」）直接交回 `NotRun`，同一臂里没有 `hardware_of_this_machine()` 的调用（判硬件的闭包只在 `gpu_test_start.rs:918` 那一臂里调）。
4. 入口用 `.into_option_printing_the_not_run_line()` 收（`gpu_test_start.rs:456-472` 一带，`TakenOrNotRun::NotRun` 臂打 `println!("{line}")`、交 `None`），所以没交卡串的进程里，调用方拿到 `None`、标准输出有那一行 `NOT_RUN_NO_GPU_ADAPTER … reason=调用方没交卡串…`。
5. 调用方照 `None` 返回：全仓 173 处调用点里 168 处是 `else { return; }` 或 `.is_none()` 后 `return;`，4 处是返回 `Option` 的帮手里 `?` 往上交，1 处是 `.expect`（见第三节表与第 3.4 条）。

限定：
- 只核了 `gpu_card_pool_or_report_not_run` 一族。`gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs:100`（「const TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL」）登记的、还经 `usable_cards_or_report_not_run` / `first_usable_card_or_report_not_run` 取卡的测试文件，不在这一批改动里，我没追（没核）。
- 「崩溃判段不借卡」那半句这一批没改，我没判。

### 1.2 「交了卡串而卡项读不成」与条款第一句的关系——推不出（条款没说这一种）

条款第一句只管「没交卡串」。交了卡串但读不成时，新代码拿把手那一步照样往下判硬件（`gpu_test_start.rs:916`，「Ok(CardsOfTheCallerForTheGpuTests::HandedOverByTheService(_)) | Err(_) => {」），硬件有就交把手，起池时才 panic（第三节 3.3 条）。这不是「本次未跑」，也不违反第一句；是否该改成别的结局，条款里找不到依据，只有 `gpu_test_start.rs` 起池处文档注释写的 panic 清单（见 3.3 条）。

### 1.3 「判段之外用卡的测试交 GPU 服务代跑（接口待定）」——推不出（没有接口，条款自己写了待定）

- 代码里没有任何把用卡的测试交给 GPU 服务代跑的调用：`grep -rn '代跑' crates/singlefs-verification/src --include=*.rs` 在 `gpu_card_configuration.rs` 里命中 8 行（771、806、1149、1190、1195、1226、1255、1258，量的），我读过 771 与 806：是报错文字里的出路句子；其余 6 行是同一文件里测试对这句话的断言与说明（读了行首，没逐行追）。
- 这一批的做法是：没交卡串的测试打「本次未跑」返回，不借卡，不另起代跑。这与「接口待定」不冲突；也意味着这一类测试在没交卡串的任务里**没有任何覆盖**，覆盖要等那个接口。
- 与用户定案「删掉这部分无用的逻辑」（`records/2026-11-13-全量提交门禁前收尾调度.md:1647`）对照：验证包里借卡通道的残留（`CardLeaseChannelOfTheCaller`、`CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY`，被 `tests/common_gpu_cards_of_the_caller/mod.rs` 引用）这一批没动。我没追它是不是「无用」，写进「没做什么」。

## 二、检查单九格

格名与判法照 `.claude/skills/three-way/SKILL.md`「代码轮正文的检查单」。每格先给判定，再给证据。

### 2.1 走不到的理由二次查实——成立（带三条限定）

要判的一句：`card_pool_started_now` 的 panic「只剩硬件前后不一致那一种」（文档注释在 `gpu_test_start.rs:899-900`，panic 在 `gpu_test_start.rs:1017`，「TakenOrNotRun::NotRun { line } => panic!(」）。

追的是「怎样才走到 1017 那一臂」：它要 `card_pool_or_the_not_run_line`（`gpu_test_start.rs:1015`，「match self.card_pool_or_the_not_run_line() {」）交回 `NotRun`，即起池结局是 `NotRunNoGpuAdapter`。起池结局在 `gpu_test_start.rs:1753`（「let start = GPU_CARD_POOL.get_or_init(start_the_gpu_card_pool);」）按 `card_pool_start_outcome_on` 定，只有两个来源产生 `NotRunNoGpuAdapter`：

- 来源甲：调用方没交卡串。`gpu_test_start.rs:1691`（「Ok(CardsOfTheCallerForTheGpuTests::NotHandedOver { reason_not_run }) => {」）起，返回 `NotRunNoGpuAdapter`。经公开入口拿到的把手到不了这里：拿把手读 `of_this_process`（`gpu_test_start.rs:868`），起池读同一个 `of_this_process`（`gpu_test_start.rs:1678`，「let configuration = GpuConfigurationOfTheCaller::of_this_process();」），两处是同一份 `OnceLock`（`gpu_card_configuration.rs:854`），进程里卡串的有无前后相同；拿把手时没交卡串的早在 863 的入口返回了。（推的，没量过）
- 来源乙：交了卡串、起池时一张点名的卡都没取到（`named.cards.len()` 为 0，`gpu_test_start.rs:1706`），`decide_gpu_test_start` 用同样的 Vulkan 枚举与 PCI 读数判，枚举里没有 NVIDIA 独显且 PCI 上也没有（或看不见 PCI）才交 `NotRunNoGpuAdapter`（`gpu_test_start.rs:241-243`、`:265-270` 一带）。这与拿把手时 `nvidia_hardware_of_this_machine`（`gpu_test_start.rs:804`）判 `Absent` 的条件逐字相同（`gpu_test_start.rs:816`，「GpuTestStart::NotRunNoGpuAdapter { reason } => {」）；拿把手时判 `Absent` 的已经在 919 返回 `NotRun`，没交把手。所以走到乙只能是拿把手那一刻与起池那一刻两次枚举不一致——就是文档写的那一种。

限定（都在文档注释的射程里，或者是要说清楚的）：

1. 经 `gpu_card_pool_or_report_not_run_given_the_hardware`（`gpu_test_start.rs:935`）拿到的把手不受上面约束：调用方给「有」，没交卡串时起池判成没卡，`card_pool_started_now` / `smallest_maximum_storage_buffer_binding_size`（`gpu_test_start.rs:1002`）/ `judge_every_state_on_a_card_leased_from_the_pool`（`crash_judge_dispatch.rs:1523` 调 `smallest_maximum_…`）会 panic。这一条文档里写了（`gpu_test_start.rs:1009-1012`），今天全仓只有 2 个调用文件，两个都走不到（见 3.2 条）。
2. 同一个 `card_pool_started_now` 还会经 `gpu_card_pool_or_the_not_run_line` 在 `gpu_test_start.rs:1759-1764`（「GpuCardPoolStartOutcome::CardsPresentButNotUsable { reason } => {」后接 `panic!(`）panic：交了卡串而读不成、卡串没给这台机器点卡、点名的卡少一张。这是原来就有的「配置写错判红」，文档 `gpu_test_start.rs:1010-1011` 一句「起池那一步的 panic 同 `card_pool::gpu_card_pool_or_the_not_run_line`」指到了。「只剩那一种」只对 1017 那一臂成立，不是对「`card_pool_started_now` 整个函数不会 panic」成立。
3. 三份实现员报告写「没有新加 `todo!`、`unreachable!`、`assert!`」：产品代码侧属实（补丁里新加的 `assert!` 与 `panic!` 全在 `tests.rs` 里，「验证包、池级 checker、函数单元图的 `src/` 不列」（`.claude/agents/implementation-writer.md:57`））。量的：`grep -nE '^\+.*(panic!|assert!|unreachable!|todo!)' crates.patch` 命中补丁的第 174、181、184、186、213、217、249、285、292、301 行，都在 `gpu_test_start/tests.rs` 那一段（那一段从补丁第 139 行起）。

推翻它的现象：在没交卡串、用公开入口拿到把手的进程里，`card_pool_started_now` 照 1017 panic（来源甲被打破：例如有别的路径在拿把手与起池之间改了 `of_this_process` 记的那份）；或者交了卡串、`nvidia_hardware_of_this_machine` 判 `PresentOrToBeJudgedByTheCardPool` 之后起池交 `NotRunNoGpuAdapter` 而两次枚举其实没变（来源乙以外还有路）。

### 2.2 类型与名字——成立（带一条要写清的事实）

- 新名字：`gpu_card_pool_of_the_configuration_of_the_caller_or_the_not_run_line`（`gpu_test_start.rs:878`）、`gpu_card_pool_of_the_configuration_or_the_not_run_line`（`gpu_test_start.rs:902`）、`HOSTS_NOT_READ_WHEN_TAKING_THE_CARD_POOL_HANDLE`（`gpu_test_start.rs:892`）；都是说内容的名字，没有缩写、没有序数。没有新类型、没有新错误成员。同文件里这三个名字各只定义一次（`grep -nE 'fn gpu_card_pool_of_the_configuration' gpu_test_start.rs` 命中 878、902 各一次，量的）。
- `match` 里 `Err(_)`（`gpu_test_start.rs:916`）是或模式里的一支，不是 `_ =>` 通配臂；整份补丁没有新加 `expect`、`unwrap`（`grep -nE '^\+.*(\.expect\(|\.unwrap\()' crates.patch` 零命中，量的）。
- 要写清的事实（推的，没量过）：`cards_for_the_gpu_tests_given` 在 `gpu_card_configuration.rs:948`（「Some(card_string) => parse_cards_by_priority(card_string, &addresses_of_the_hosts()?)」）先调 `addresses_of_the_hosts()?` 再解析。拿把手交进去的闭包（`gpu_test_start.rs:907-909`）总是交 `Err`，所以**交了卡串的情形一律以 `Err(_)` 到 916 那一臂，写法对不对都一样**；`HandedOverByTheService(_)` 这一支在这个闭包下走不到，只是让 `match` 穷尽。916 行下面那句注释「交了卡串而读不成（`Err`）也往下判硬件」把 `Err` 说成了特例，实际是交了卡串的唯一形状。结果上没有错（两支同一个动作），但读代码的人会误以为存在「读成了」的另一支。单测里交的卡串 `"127.0.0.1:0:4G"`（`tests.rs:230`、`tests.rs:260`）也是走 `Err`。

推翻它的现象：新加的函数里出现 `_ =>` 通配臂、裸整数充当不同语义的量、或补丁里出现 `expect`/`unwrap`。

### 2.3 不兼容旧代码与旧格式——成立

量的：`grep -nE '^\+' crates.patch | grep -nE '兼容|legacy|旧版|旧布局|迁移|此前|原先|起初|最初|后来|改成了|换成了|修过'` 零命中（补丁文件 `~/.local/share/singlefs/work/merge-and-verify/2026-11-21-931572/merge-patch/crates.patch`，309 行）。命中的只有 `gpu_test_start.rs:900` 的「照旧 panic」：描述运行时行为（这一臂 panic 不变），不是旧版本分支；`照旧` 不在 `.claude/agent-common.md` 列的历史词里，门禁 doc-text 的 comment-history 格怎么判我没跑（交主 agent 的命令里不单列，它不归这一格）。

推翻它的现象：补丁里出现认旧版本号、旧布局号、迁移入口。

### 2.4 不新建判定库——成立

没交卡串时 `gpu_card_pool_of_the_configuration_or_the_not_run_line` 不写任何库（`gpu_test_start.rs:911-914` 只拼那一行字符串）；整份补丁没有 `CrashLibrary`、`two_stages`、`DisposableLibrary` 的新引用（`grep -nE '^\+.*(CrashLibrary|two_stages|DisposableLibrary|RocksDB|sqlite)' crates.patch` 零命中，量的）。要说明的副作用：没交卡串的进程里，用例在拿把手这一步返回，**不再往对拍那一类库写 CPU 参照与判表**（第三节 3.1 条逐个看过，没有发现有谁读这些行）。

推翻它的现象：补丁里出现新的存放目录或库开法。

### 2.5 所有任务多机可用——成立

- 四条新单测的配置由 `GpuConfigurationOfTheCaller::for_the_unit_tests(…)`（`gpu_card_configuration.rs:871`，多机配置路径不设、没清场）造，硬件由闭包给，不读本机环境、不枚举 Vulkan（`tests.rs:178-250` 一带，转述；两条交了卡串的用 `"127.0.0.1:0:4G"` 这类字面地址，没有主机名）。
- 公开入口今天读这个进程的环境与本机的 Vulkan/PCI，只在交了卡串的进程里读硬件（`gpu_test_start.rs:918`）；没交卡串的进程不读，两台判得一样。这条没变。
- 变异行 `crates/mutations.tsv:8623-8626` 的第三、第四段是源码改写，不带主机名与本机路径（读过这四行的全文，量的）。

推翻它的现象：新单测在两台里有一台红；或新行里出现主机名。

### 2.6 测试优先多线程——成立（这一格不适用）

四条新单测是纯函数判法（`tests.rs:178-330` 读过），没有循环可切，没有新加崩溃检测；没有写死线程数。推的，没量过。

### 2.7 带 --item 的命令与项——成立（限定：列项与点名结局是主 agent 量的，我没复跑）

- 四行都在：`crates/mutations.tsv:8623`、`:8624`、`:8625`、`:8626`（行首是「GPU 卡池把手 没交卡串也往下判硬件、交把手（要池里的卡时 panic）」「GPU 卡池把手 交了卡串、有卡也报本次未跑」「GPU 卡池把手 交了卡串、没卡也交把手」「GPU 卡池把手 取卡池的接线不把调用方那一份交给判法、只看硬件（没交卡串也交把手）」）。
- 四行第三段（锚点）在主工作区 `gpu_test_start.rs` 里各恰好命中一次（量的：`python3 -I` 把 `\n` 还原后 `str.count` 得 1、1、1、1）。
- 四行第五段点名 `--lib --features gpu -- gpu_unit_checks::gpu_test_start::tests::<名> --exact`，四个名字对应的 `fn` 都在 `tests.rs`（量的：`tests.rs:178`、`tests.rs:221`、`tests.rs:251`、`tests.rs:286` 起的函数名与第五段的名字逐字相同）。
- 「`--list-items` 能列出新项」与「点名跑出抓到 4 条」是背景材料里主 agent 的观测，我没复跑。

推翻它的现象：`bash .claude/gate.d/crates-mutation-replay.sh --list-items` 列不出这四个名字，或点名的四行有一行「没红」。

### 2.8 新崩溃流多罩了什么——成立（本轮没有这一类）

补丁没有新加崩溃点重放的流或用例（`grep -nE '^\+.*(crash_point|CrashPointReplay|crash-case)' crates.patch` 零命中，量的）。

### 2.9 放的位置——成立

新单测放在 `gpu_test_start.rs:1817`（「mod tests;」）引进来的 `gpu_test_start/tests.rs`（`git ls-files` 里本来就有这份文件，量的）；包是 `singlefs-verification`，`.claude/gate.d/crate-roles.tsv:12` 登的角色是「验证」。没有新开包、没有跨包引源码。

## 三、全仓调用点逐个追到底（「要攻的地方」第 1、2、3 条）

### 3.0 三个入口各有几条路（读码，推的，没量过）

| 进程里的情形 | `gpu_card_pool_or_report_not_run`（`gpu_test_start.rs:863`） |
|---|---|
| 没交卡串（没设、空串、只有空白） | `gpu_test_start.rs:911` 交 `NotRun`，不调判硬件的闭包；`gpu_test_start.rs:470` 打 `println!("{line}")`，交 `None`；调用方返回，退 0 |
| 交了卡串、硬件判 `Absent` | `gpu_test_start.rs:919` 交 `NotRun`，同样打那一行、交 `None` |
| 交了卡串、硬件判 `PresentOrToBeJudgedByTheCardPool` | `gpu_test_start.rs:923` 交卡池把手，不读卡项、不起池；第一次借卡或要池里的卡时才起池（见 3.3） |

### 3.1 `gpu_card_pool_or_report_not_run` 的调用点（量的：计数；分支：读过代码）

计数命令与原样输出（量的）：

```
$ grep -rnE 'gpu_card_pool_or_report_not_run\(' --include=*.rs crates gpu-verification-server | grep -vE '^[^:]*:[0-9]+:\s*(//|use )' | wc -l
174
```

174 = 173 处调用点 + 定义行 `gpu_test_start.rs:863`。173 处在 73 个文件里；用只读脚本（草稿目录 `sites.py`）按「调用行往上找最近的 `fn`、往下看 `else {` 后一行」分：

| None 之后做什么 | 处数 | 走到哪一支 |
|---|---|---|
| `let Some(x) = … else { return; }` 或 `if …is_none() { return; }` | 168 | 没交卡串：拿把手一步 `NotRun`，用例当场返回，之后什么都不跑；交了卡串且有卡：往下用把手 |
| 返回 `Option` 的帮手里 `…(test_name)?` 往上交 | 4（`crash_judge_gpu_judges_cached_references_and_journal_conflicts_like_the_cpu_checker.rs:452`、`:486`，`crash_judge_gpu_writer_identity_allocation_record_spans_and_new_root_segments_of_every_record_like_the_cpu_checker.rs:370`，`crash_judge_gpu_rollback_floor_ceiling_rebuilt_from_the_journal.rs:712`） | 往上交 `None`；这四个帮手的调用处我逐个看了上下文：`let Some(…) = … else { return; }` 或把 `Option` 当语句丢掉，没有 `expect`/`unwrap`（读了每一处调用的前后各几行，不是全文） |
| `.expect(…)` | 1（`gpu_unit_checks_the_given_states_engine_takes_its_cards_from_the_card_pool_of_the_process.rs:83`，`.expect` 在 `:84`） | 前面 `:68` 已经 `if …is_none() { return; }`；`:83` 的第二次调用与第一次同配置同硬件（进程里各只记一份），第一次是 `Some` 它就是 `Some`，不会 panic（推的） |

全部 173 处，文件与行号（量的：脚本从 `grep -rnE` 的输出算出；`(is_none 后 return)` 是 `if gpu_card_pool_or_report_not_run(` 多行写法、行里 `.is_none()` 在后面）。路径相对 `crates/singlefs-verification/tests/`，`src/…` 相对 `crates/singlefs-verification/`，`gpu-verification-server/…` 相对仓根：

```
crash_judge_gpu_judges_the_subtree_coverage_key_range_like_the_cpu_judge.rs: 408
crash_judge_gpu_agrees_with_the_cpu_judge_on_deadlist_entries_outside_every_side_and_missing_from_the_mapping.rs: 386
crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs: 1150(is_none 后 return), 1216(is_none 后 return)
crash_judge_gpu_agrees_with_the_cpu_judge_on_the_pending_release_bound.rs: 391
bad_disk_input_campaign.rs: 1283
gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs: 284
gpu_unit_checks_the_given_states_engine_takes_its_cards_from_the_card_pool_of_the_process.rs: 68, 83(expect)
gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs: 184
record_checker_judges_absence_by_the_persisted_set.rs: 1635
crash_amplification_observer_images_judged_on_the_gpu_name_the_card.rs: 224, 346
crash_judge_gpu_secondary_reads_after_the_source_segment_is_discarded.rs: 661, 771, 926
crash_judge_fall_back_on_the_union_of_readers_matches_the_walk_alone.rs: 139, 330
crash_judge_gpu_judges_cached_references_and_journal_conflicts_like_the_cpu_checker.rs: 452(?), 486(?)
crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs: 442
crash_judge_gpu_judges_files_written_by_inode_number_like_the_cpu_judge.rs: 433
crash_judge_gpu_agrees_with_the_cpu_judge_on_root_ring_stride_refusals.rs: 116
crash_judge_gpu_judges_size_changes_of_inodes_other_than_the_first_like_the_cpu_judge.rs: 343
crash_judge_gpu_writer_identity_allocation_record_spans_and_new_root_segments_of_every_record_like_the_cpu_checker.rs: 370(?), 1160
crash_judge_gpu_judges_the_number_acquisition_skip_like_the_cpu_judge.rs: 350
crash_judge_gpu_rollback_floor_ceiling_rebuilt_from_the_journal.rs: 712(?)
crash_judge_gpu_reads_long_files_and_lineages_block_by_block.rs: 682, 710, 733, 760, 1444, 1483, 1519, 1562, 1593
crash_judge_gpu_agrees_with_the_cpu_judge_on_data_unit_total_order_keys.rs: 412
crash_judge_gpu_recovery_finds_central_mapping_entries_at_mapping_leaf_boundaries.rs: 777
gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs: 102, 148
crash_judge_gpu_agrees_with_the_cpu_judge_on_a_make_filesystem_over_a_used_pool.rs: 381
gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs: 300
crash_given_states_of_every_step_of_the_compaction_and_rollback_histories.rs: 503
crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs: 1497, 1581
gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs: 238
crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs: 202, 411, 463, 544
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/unit_copies_and_quarantine_marks.rs: 295, 340
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/snapshots_relocations_discards_and_compactions.rs: 157, 466
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/versions_applied_by_their_records.rs: 359, 402, 443
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/tree_kinds_and_root_record_bytes.rs: 536, 624, 695
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/system_configuration_slots_found_by_the_judge_tables.rs: 87, 392
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/reference_intervals_after_rollbacks_and_snapshots.rs: 170, 280, 315, 349, 425
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/retirement_records_and_an_unlink_rolled_back.rs: 293
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/container_tree_children_not_walked_into.rs: 301
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/pairing_blocks_judged_on_both_sides.rs: 722
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/journal_groups_and_last_record_flags.rs: 297
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/roots_abandoned_by_a_crash_recovery_and_instance_table_chains.rs: 1192, 1309, 1376, 1515
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/first_publish_replayed_by_the_next_mount.rs: 333, 1115
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/every_state_of_the_six_flows_and_their_resealed_fields.rs: 284, 381
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/truncations_holes_and_the_truncation_field.rs: 138, 205, 440, 593
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/deletion_in_progress.rs: 197, 443, 482, 613
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/walked_locations_verified_copies_and_unwritten_slots.rs: 456, 538, 574
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/expected_bits_and_examined_states.rs: 12, 64
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/children_not_walked_into_and_their_second_copies.rs: 335, 404, 492, 578, 659, 697
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/fallback_on_the_union_of_the_readers.rs: 215, 243, 567, 597, 714, 741, 769, 795, 822, 849
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/recovery_kernel_falling_back_to_a_witnessed_version.rs: 109, 169, 267, 564, 708, 792, 960, 1019
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/root_ring_candidates_and_floor_raise_windows.rs: 140, 208, 335, 401, 437, 514, 682
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/tree_table_entries_and_lineage_kinds.rs: 376, 459
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/accounting_verdicts_across_slicings.rs: 175
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/tombstone_lineage_and_inode_tree_entries.rs: 89, 971, 1012, 1106
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/container_capacity_knobs_in_the_scan_kernel.rs: 54, 89, 439, 549, 625, 916, 1307
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/kernel_sources_and_planted_bits.rs: 346, 518
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/last_pairing_group_and_unrecognized_incompat_bytes.rs: 75, 318, 441
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/pairing_judges_on_the_cards.rs: 92, 501
crash_given_states_of_the_images_harness_scenarios_hand_out/judges_and_libraries_of_the_given_states_engine.rs: 241, 306, 512, 735, 812, 989
crash_injection_campaign/gpu_and_cpu_runs_stop_files_shares_and_other_hosts.rs: 66(is_none 后 return), 120(is_none 后 return), 304
crash_injection_campaign/pool_after_the_writable_mount_that_follows_a_crash.rs: 215, 597
crash_judge_gpu_tables_at_their_capacity/inode_and_object_birth_tables_at_capacity.rs: 92
crash_judge_gpu_tables_at_their_capacity/examined_table_full_and_long_histories.rs: 137, 278, 470
crash_judge_gpu_tables_at_their_capacity/allocation_record_tables_at_capacity.rs: 103, 220
crash_judge_gpu_tables_at_their_capacity/instance_table_rows_at_capacity.rs: 235, 312
crash_judge_gpu_tables_at_their_capacity/main.rs: 656
src/crash_judge_dispatch/tests.rs: 513
src/crash_judge_gpu/tests.rs: 1468
src/crash_identity/compiled_judging_units_key_table/tests/dispatch_widths_judge_the_same_verdict_words.rs: 55
gpu-verification-server/tests/packed_tables_service_path_agrees_with_the_direct_judge.rs: 58
gpu-verification-server/tests/library_service_path_agrees_with_the_direct_judge.rs: 63
gpu-verification-server/tests/gpu_service_path_agrees_with_the_direct_judge.rs: 194, 237
gpu-verification-server/tests/facts_verifier_service_path_agrees_with_the_direct_verifier.rs: 79
```

### 3.2 要攻第 1 条：没交卡串的任务里，原先「先在 CPU 上录、往判定库写待判行」的流程现在断没断

结论：**没发现靠这条路干活的用例或流程**；但这一批确实让一类行为消失，列在下面，由主 agent 判算不算要紧。

消失的行为（读码，推的）：没交卡串的进程里，用例在拿把手那一步返回，从此不再往对拍那一类的库写这两样东西：
- CPU 参照那张表（`cpu_reference_table_read_back_from_the_pairing_library`，`gpu_test_start.rs:32`，经 `two_stages` 写库、录入段打 `CPU_REFERENCE_RECORDED_ONLY`）；
- GPU 对拍判表（`judge_every_state_on_a_card_leased_from_the_pool` 里经 `judge_tables_through_the_pairing_library` 写库，只录那一趟打 `JUDGE_TABLES_RECORDED_ONLY`，`crash_judge_dispatch.rs:1515` 起「StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {」，返回 `None`）。

这一批之前，没交卡串、硬件有卡的进程里这两样的实际结局（读码，推的）：
- 不设只录那一段（门禁起验证包的常规一趟；举例，走 `judge_every_state_on_a_card_leased_from_the_pool` 的用例）：写库之后 `crash_judge_dispatch.rs:1523`（「states_per_scratch_lane(cards.smallest_maximum_storage_buffer_binding_size());」）→ `gpu_test_start.rs:1002` → `card_pool_started_now` → 起池判成没卡 → `gpu_test_start.rs:1017` panic。这就是背景材料里「基线就红」的来路。
- 只录那一段（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）：写库、打 `RECORDED_ONLY` 返回。

那么有没有人在没交卡串的情形下，靠只录那一段给后面的判段留行？我查了所有会设只录那一段的发起方（量的，命令见「要主 agent 跑的命令」一节的只读复核命令，我这一趟跑的是同一批 `grep`）：

| 发起方 | 位置 | 与 `gpu_card_pool_or_report_not_run` 的关系 |
|---|---|---|
| 产品 GPU 包真显卡对拍那一格，只录那一趟摘掉 `GPU_CARDS_BY_PRIORITY` | `.claude/gate.d/lib/crash-point-replay-feature-card-and-audit-cells.sh:620`（「env "${cleared_environment[@]}" -u GPU_CARDS_BY_PRIORITY」）；文件头说明 `.claude/gate.d/crash-point-replay.sh:271`（「先只录（env 里 SINGLEFS_CRASH_AMPLIFICATION_STAGE=record、摘掉 GPU_CARDS_BY_PRIORITY，不持卡」） | 起的是 `singlefs-background-gpu-tests`，那个包与 `singlefs-background-gpu` 里没有任何对 `gpu_test_start` 或 `GpuCardPoolStartedAtTheFirstLease` 的引用（量的：`grep -rln 'gpu_test_start\|GpuCardPoolStartedAtTheFirstLease' crates --include=*.rs` 在验证包之外只命中 `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/resident_gpu_kernels/tests.rs`，它用的是 `first_usable_card_or_report_not_run`，不是这一批改的入口）。所以这一格的只录那一趟不受影响 |
| 崩溃放量入口 `scripts/crash/crash-amplification-item.sh` 的只录、只判两趟 | 脚本里 `--stage record`/`--stage judge` | 点名的是 `scripts/crash/crash-amplification.sh` 里登记的放量分项函数；登记的函数名与 173 处调用点所在的 167 个函数名不重合（量的：逐个 `grep -lF`，零命中，见下）；登记函数经帮手间接走到调用点的，没逐个追（没核） |
| 登记的崩溃枚举用例（`crash-case:` 行）的只录/只判 | `scripts/gate/admission.py` 的 `crash-case-command --stage`；门禁 `crash-point-replay` 不带它 | `stage-inputs.tsv` 里 `crash-case:` 点名的函数与 173 处调用点所在的函数不同；调用点所在的三份文件 `crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`、`crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs`、`record_checker_judges_absence_by_the_persisted_set.rs` 也在 `stage-inputs.tsv` 里有别的函数的登记，但那些登记的函数里没有调用点（调用点在 `:1497`、`:1581`、`:1150`、`:1216`、`:1635`，各在别的测试函数里，量的） |
| 用例自己设只录那一段 | `crash_amplification_block_range_selection.rs:720`、`gpu_unit_checks_a_recording_only_pairing_through_the_card_pool_handle_starts_no_card_pool.rs`（用 `…_given_the_hardware`，见 3.4） | 前者不走这个入口；后者是 3.4 的第一个调用文件 |

比对命令的原样结局（量的）：登记表五份与 `scripts/crash/*.sh` 对 167 个函数名逐个 `grep -lF`，输出文件 `fn-in-registries.txt` 只有 1 行 `? :: scripts/crash/…`；那 1 行是「函数名没解析出来」的那一处（`expected_bits_and_examined_states.rs:12`，帮手的签名分几行写，`?` 被当字面串搜，匹配任何含 `?` 的行），不是真命中。该帮手被哪些测试调、那些测试在不在登记表里，我没追（没核）。

仍然在的「取卡之后才录」约定：`pairing_blocks_judged_on_both_sides.rs:716`（「取卡之后才录，没卡的机器上不录」）说明这个测试目录里的帮手本来就按「没卡的机器上不录」写，没交卡串的进程现在按同一口径走。拿把手之前先录的只有一处：`first_publish_replayed_by_the_next_mount.rs:1078` 在 `:1115` 的取卡之前经 `image_with_every_write_landed_read_back_from_the_pairing_library` 写库，这一处没交卡串也照录、然后在 `:1115` 返回，行为没变。

推翻它的现象：有一个起只录那一段的发起方（脚本、门禁格、人工命令）的目标含 173 处调用点所在的测试，并且它的判段那一趟读那些行；或者 `fn-in-registries.txt` 那一格换成逐个函数追，查出登记的放量分项/崩溃枚举用例经帮手调到了这些入口。

### 3.3 要攻第 2 条：`HandedOverByTheService(_) | Err(_)` 并一臂，起池时走到哪里

先说一个事实（推的，没量过）：这一臂在拿把手时**只有 `Err` 一种形状**。闭包 `gpu_test_start.rs:907-909` 总是交 `Err`，`gpu_card_configuration.rs:948` 先求值 `addresses_of_the_hosts()?`，所以交了卡串的进程里 `cards_for_the_gpu_tests_given` 总是 `Err`（卡串写得对不对都一样）。

交了卡串、拿把手时硬件判 `Present` 的进程，起池时：

1. 第一次借卡 / 要池里的卡 / 要最小缓冲上限 → `gpu_test_start.rs:988`（「match self.card_pool_or_the_not_run_line() {」）或 `:1015`（同一句）→ `gpu_test_start.rs:1753`（「let start = GPU_CARD_POOL.get_or_init(start_the_gpu_card_pool);」）。
2. 起池照真多机配置读卡项：`gpu_test_start.rs:1690`（「let named = match configuration.cards_for_the_gpu_tests() {」），分三种：
   - 卡串写对、这一台有卡、点名的卡都找得到：`Started`，没有问题。
   - 卡串读不成（`gpu_test_start.rs:1699`，「Err(reason) => CardsNamedForThisMachine::refused(reason),」；多机配置找不到也是这一种）、或读成了而没给这一台点卡（`gpu_test_start.rs:1600`，「if mine.is_empty() {」）：一张卡都没取，`decide_gpu_test_start` 用 0 张卡判，硬件有 → `AdapterPresentButNoCardStarted` / `NvidiaCardOnPciButVulkanEnumeratesNone` / `PciBusVisibleButListingUnreadable` → `CardsPresentButNotUsable` → `gpu_test_start.rs:1759` 起的 `panic!`（消息是原因加「卡池照调用方给这个进程的显卡配置挑的卡」）。
   - 点名的卡少一张：同样 `CardsPresentButNotUsable`、panic。
3. `lease_card_or_report_not_run`（`gpu_test_start.rs:968`）与 `card_pool_started_now` 都经这一步，所以都 panic；`lease_card_or_the_not_run_line`（`:983`）也是（panic 发生在 `card_pool_or_the_not_run_line` 里面，不是它的 `NotRun` 臂）。

答：交了卡串而卡项读不成、硬件有卡 → **panic（用例红），不是本次未跑，也不是断言判红**；硬件判 `Absent` 的机器上拿把手就报本次未跑，到不了起池。与条款：第一句只管没交卡串，这一种条款没说，推不出对不对得上；与代码自己的文档对得上（`gpu_test_start.rs:1746-1748` 的「显卡配置写错、没给这台机器点卡、点名的卡少一张找不到」）。

两件值得主 agent 知道的（推的）：
- 这一种 panic 发生在用例已经建好表、`judge_every_state_on_a_card_leased_from_the_pool` 已经把判表写进库之后（写库在 `crash_judge_dispatch.rs:1513` 一带，panic 在 `:1523`）：用例红，但库里留着写过的行。
- 没有单测钉这一支：`tests.rs` 里起池的单测只有空卡串那一条（`tests.rs:152` 起的那条，断言交 `NotRunNoGpuAdapter`），交了卡串而读不成的起池结局（`CardsPresentButNotUsable`）没有单测；两条交了卡串的新单测只钉到 `Taken` / `NotRun`（拿把手这一步），不起池。变异行 `crates/mutations.tsv:8624`、`:8625` 守的是拿把手的有卡/没卡两支，不守这一支。

### 3.4 要攻第 3 条：`…_given_the_hardware` 与 `card_pool_started_now` 的调用点各走到哪一支

`gpu_card_pool_or_report_not_run_given_the_hardware`（定义 `gpu_test_start.rs:935`）只看调用方给的硬件、不看卡串（文档注释 `gpu_test_start.rs:932-934`）：给 `Absent` 打那一行、交 `None`；给 `PresentOrToBeJudgedByTheCardPool` 直接交把手。全仓调用文件两个（量的：`grep -rnE 'gpu_card_pool_or_report_not_run_given_the_hardware\(' --include=*.rs crates gpu-verification-server` 在定义行之外命中 2 处）：

| 调用点 | 进程里卡串 | 给的硬件 | 把手之后走到哪一支 |
|---|---|---|---|
| `gpu_unit_checks_a_recording_only_pairing_through_the_card_pool_handle_starts_no_card_pool.rs:66`（「let cards = gpu_card_pool_or_report_not_run_given_the_hardware(」） | 没交（假通道设进环境之前问过一次，核记下的那一份没有卡串，`tests/common_gpu_cards_of_the_caller/mod.rs` 的断言） | `Present` | 只录那一段（`cleared_then_set` 设 `record`）：`judge_every_state_on_a_card_leased_from_the_pool` 在 `crash_judge_dispatch.rs:1515` 起的 `RecordedOnly` 臂返回 `None`，**不到** `:1523` 的 `smallest_maximum_…`，也不借卡、不起池；用例断言 `card_pool_started_in_this_process().is_none()` 与 `Ok(None)`。没有 panic、没有判红 |
| `gpu_unit_checks_the_card_pool_handle_starts_no_card_pool_and_leases_no_card_until_its_first_lease.rs:40` | 没交（同上，假通道） | `Present` | 拿把手不起池；`:61`（「match handle.lease_card_or_the_not_run_line(TEST_NAME, MEBIBYTES_LEASED) {」）借卡：起池 `NotHandedOver` → `NotRunNoGpuAdapter`（`gpu_test_start.rs:1691`）→ `gpu_card_pool_or_the_not_run_line` 交 `NotRun` → `gpu_test_start.rs:992` 原样交回；用例断言交回的是未跑与点名用例的那一行、原因含「调用方没交卡串」。没有 panic |

「没交卡串而调用方给『有』，后面还有没有会 panic 的路」：**API 上有，今天没有调用点走到**。会 panic 的三条路（推的，没量过）：
- `card_pool_started_now`（`gpu_test_start.rs:1014`）：起池判成没卡 → `:1017` panic；
- `smallest_maximum_storage_buffer_binding_size`（`gpu_test_start.rs:1002`）：经上一条；
- `judge_every_state_on_a_card_leased_from_the_pool`（`crash_judge_dispatch.rs:1523`）：不设只录那一段时，写库之后经上一条。

不会 panic 的：`lease_card_or_the_not_run_line`（`gpu_test_start.rs:983`）与 `lease_card_or_report_not_run`（`:968`）都交 `NotRun`（后者打那一行、交 `None`）。文档 `gpu_test_start.rs:1009-1012` 把「用例经 …_given_the_hardware 在没卡或没交卡串的机器上给了『有』」列为 panic 来源之一，与上面一致。以后再有第三个调用方给「有」又走前三条路的，没有任何东西拦它（没有门禁格、没有单测钉「`…_given_the_hardware` 的调用方不碰这三条路」）。

`card_pool_started_now` 的调用点（量的：`grep -rnE '\.card_pool_started_now\(\)' --include=*.rs crates gpu-verification-server` 4 处）：

| 调用点 | 把手来自 | 没交卡串 | 交了卡串且读得成、有卡 | 交了卡串而读不成、有卡 | 硬件前后不一致 |
|---|---|---|---|---|---|
| `gpu_test_start.rs:1002`（`smallest_maximum_…`） | 调用它的函数拿到的把手（`crash_judge_dispatch.rs:1523`） | 拿把手时已返回，到不了 | 起池成功，取最小上限 | `:1759` 起的 panic | `:1017` panic |
| `gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs:231` | `:184`（「let Some(pool) = gpu_card_pool_or_report_not_run(TEST_NAME) else {」） | 到不了 | 照常查账 | 同上 | 同上 |
| `comparison_on_every_state_and_room_on_the_host.rs:574`（`pairing_cards_of_the_pool`）、`:605`（`smallest_card_ceiling_of_the_pool_mebibytes`） | 同目录的帮手交进来的把手：`comparison_on_every_state_and_room_on_the_host.rs:566` 与 `:626`、`pairing_blocks_judged_on_both_sides.rs:102`、`pairing_judges_on_the_cards.rs:525` 调它们，把手来自各用例开头的入口调用（如 `pairing_judges_on_the_cards.rs:92`、`:501`；读了调用行，没沿帮手链逐条追到底，推的） | 到不了 | 照常 | 同上 | 同上 |

### 3.5 小结（三条要攻的地方）

- 第 1 条：没交卡串的进程里这一批让「CPU 参照表与 GPU 判表写进对拍类库」不再发生；我找不到靠它的流程（见 3.2），要紧不要紧由主 agent 判。
- 第 2 条：`Err(_)` 并进来的是交了卡串的**全部**情形（见 3.3 第一句），不只是读不成的；读不成且硬件有卡，到起池 panic，不是本次未跑；没有单测钉这一支。
- 第 3 条：`…_given_the_hardware` 今天 2 个调用点都走不到 panic 路；API 上 panic 路仍在，文档写明，没有东西拦以后的调用方。

## 四、要主 agent 跑的命令

这一份没有模型，也没有起过探针，没有内存与时间的读数。下面三条是读码推出结论之后想让主 agent 量的；跑完之前，凡是写了「推的，没量过」的都按推的办。范围声明的写法照 `.claude/hooks/run-scope-guard.sh` 文件头；被范围声明闸拒了，照它的说明改（这几条点的是 `--lib` 与不走崩溃枚举的测试目标，我先照「不会走崩溃域」写）。

1. 四条新单测在没交卡串的任务里绿（验 3.0 表第一行与 2.7、2.5 的「不读本机状态」）。在仓根起：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的库测试 4 条; 范围=4 个测试; 为什么=核拿卡池把手的四条单测在没交卡串的任务里绿' task-server/task submit -- cargo test -p singlefs-verification --release --features gpu --lib -- gpu_unit_checks::gpu_test_start::tests::the_card_pool_handle_without_a_card_string_of_the_caller_is_not_run_before_the_hardware_is_judged gpu_unit_checks::gpu_test_start::tests::the_card_pool_handle_with_a_card_string_of_the_caller_on_a_machine_with_nvidia_cards_is_handed_out gpu_unit_checks::gpu_test_start::tests::the_card_pool_handle_with_a_card_string_of_the_caller_on_a_machine_without_nvidia_cards_is_not_run gpu_unit_checks::gpu_test_start::tests::the_card_pool_entry_hands_the_configuration_of_the_caller_to_the_judgement_and_without_a_card_string_hands_out_no_handle --exact
```

   看：`TASK_SERVER_RESULT` 行 `exit=0`，`task-server/task log <号>` 里 `test result: ok. 4 passed`。

2. 没交卡串时，一个调用点返回并打本次未跑那一行、退 0（验 1.1 与 3.1 表）。在仓根起（任务不交卡串）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card 里的 1 条测试; 范围=1 个测试; 为什么=核没交卡串时调用点返回并打本次未跑一行' task-server/task submit -- cargo test -p singlefs-verification --release --features gpu --test gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card -- a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading --exact --nocapture
```

   看：`exit=0`；日志里有 `NOT_RUN_NO_GPU_ADAPTER test=a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading reason=调用方没交卡串…`（同一行以 `gpu_memory_ledger::GPU_NOT_RUN_MARKER` 起头）；没有 `panicked`。

3. 交了卡串而读不成、硬件有卡时 panic、不是本次未跑（验 3.3；要在有 NVIDIA 卡的机器上才分得出，没卡的机器上这一条会退 0 并打 `NOT_RUN_…`，那一次不算数）。在仓根起；它让用例枚举一次本机显卡适配器、不领卡、不借卡，panic 发生在借卡之前，要不要在任务里这样跑由主 agent 定：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card 里的 1 条测试; 范围=1 个测试; 为什么=核交了读不成的卡串、硬件有卡时起池 panic 而不是报本次未跑' task-server/task submit -- env GPU_CARDS_BY_PRIORITY=not-a-card-string cargo test -p singlefs-verification --release --features gpu --test gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card -- a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading --exact --nocapture
```

   看：`exit` 非 0；日志里有 `panicked at crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start.rs:1760`，消息里点名「卡池照调用方给这个进程的显卡配置挑的卡」；没有 `NOT_RUN_` 那一行。读到别的结局（退 0、或 panic 在别的行）就推翻 3.3。

只读复核命令（不交服务，在仓根直接跑，输出能与第三节对上；都是 `grep`）：

```
grep -rnE 'gpu_card_pool_or_report_not_run\(' --include=*.rs crates gpu-verification-server | grep -vE '^[^:]*:[0-9]+:\s*(//|use )' | wc -l
grep -rnE 'gpu_card_pool_or_report_not_run_given_the_hardware\(' --include=*.rs crates gpu-verification-server
grep -rnE '\.card_pool_started_now\(\)' --include=*.rs crates gpu-verification-server
grep -rnE 'SINGLEFS_CRASH_AMPLIFICATION_STAGE=record|RUN_STAGE_ENVIRONMENT_VARIABLE' scripts .claude/gate.d crates --include=*.sh --include=*.py --include=*.rs
grep -rln 'gpu_test_start\|GpuCardPoolStartedAtTheFirstLease' crates --include=*.rs
```

## 五、什么现象会推翻各条结论（汇总）

| 结论 | 什么现象会推翻它 |
|---|---|
| 1.1 没交卡串报本次未跑 | 命令 2 退非 0，或日志里没有那一行，或有 `panicked` |
| 3.2 没发现靠「先录」的流程 | 有发起方把 173 处调用点所在的测试放进只录那一趟，且判段那一趟读它写的行 |
| 3.3 交了卡串读不成 → panic | 命令 3 在有卡的机器上退 0，或 panic 在别的位置 |
| 3.4 `…_given_the_hardware` 两个调用点走不到 panic | 命令 1 之外，这两个测试文件在没交卡串的任务里红在 `gpu_test_start.rs:1017` |
| 2.1 panic 只剩硬件前后不一致 | 公开入口拿到的把手在没交卡串的进程里照 `:1017` panic |
| 2.2 916 那一臂实际只有 `Err` | 有人把 `cards_for_the_gpu_tests_given` 改成不先求值地址，使 `Ok(HandedOverByTheService(_))` 变得可达；两支动作相同，行为不变，只是 916 下面那句注释要改 |

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 条款「GPU 用例没交卡串的报本次未跑」 | 成立 | 限 `gpu_card_pool_or_report_not_run`：没交卡串 `gpu_test_start.rs:911` 报未跑、不判硬件；173 处调用点 168 处照 `None` 返回，4 处 `?` 上交，1 处 `.expect` 前有 `:68` 的返回挡着（推的，没量过） |
| 条款「判段之外用卡的测试交 GPU 服务代跑（接口待定）」 | 推不出 | 代码里没有代跑接口，条款自带待定；没交卡串的这类测试现在只报未跑、没有别的覆盖 |
| 九格：走不到的理由二次查实 | 成立 | 1017 那一臂只剩硬件前后不一致；限定三条：`…_given_the_hardware` 会走到它（今天无调用点）、`card_pool_started_now` 另有 1759 起的配置错 panic、产品代码侧没新加 `assert!` |
| 九格：类型与名字 | 成立 | 名字说内容、无 `_ =>`、无新 `expect`/`unwrap`；916 的 `Err(_)` 实际是交了卡串的唯一形状，`HandedOverByTheService(_)` 在这个闭包下走不到（推的） |
| 九格：不兼容旧代码与旧格式 | 成立 | 补丁零命中旧版本、迁移类词 |
| 九格：不新建判定库 | 成立 | 没新存放处；副作用是没交卡串的进程不再往对拍类库写参照表与判表 |
| 九格：所有任务多机可用 | 成立 | 新单测用 `for_the_unit_tests` 与闭包给的硬件，不读本机状态 |
| 九格：测试优先多线程 | 成立 | 纯函数单测，无循环可切（推的，没量过） |
| 九格：带 --item 的命令与项 | 成立 | 四行 `crates/mutations.tsv:8623-8626`，锚点各命中一次，点名的四条单测都在；列项与点名结局是主 agent 的量，我没复跑 |
| 九格：新崩溃流多罩了什么 | 成立 | 补丁没有新崩溃流或用例 |
| 九格：放的位置 | 成立 | `gpu_test_start/tests.rs` 由 `gpu_test_start.rs:1817` 的 `mod tests;` 引进，包角色「验证」 |
| 要攻 1：没交卡串先录的流程是否断了 | 没发现 | 没交卡串的进程不再写对拍类库；找不到起只录那一趟而读这些行的发起方；只录发起方（产品 GPU 包格、放量分项、崩溃枚举用例）与调用点所在函数名不重合（量的）；经帮手间接的没逐个追 |
| 要攻 2：`HandedOverByTheService(_) \| Err(_)` 并臂 | 推不出（条款没说） | 交了卡串而读不成、硬件有卡：起池 panic（`gpu_test_start.rs:1759` 起），不是本次未跑也不是断言；无单测钉；与代码自己的 `# Panics` 文档对得上 |
| 要攻 3：`…_given_the_hardware` 仍不看卡串 | 没发现走到 panic 的调用点 | 2 个调用文件分别走只录返回与借卡交 `NotRun`；API 上三条 panic 路仍在，文档写明，无物拦以后的调用方 |

## 七、没做什么

- 没替主 agent 采纳或出判决；这一份只是正推腿的读码结果。
- 没往统一任务服务交任务、没起 cargo、没跑门禁；上面所有「推的，没量过」都没有测试结局支撑，命令在第四节。判决要引的数没有复跑过。
- 没判攻方腿的格：2102 行替换文、删掉的 1 行、变异行抓不抓得到被改坏的函数、公开入口读的是不是「这个进程的配置」有没有单测钉住（`tests.rs:286` 起的那条单测由闭包给配置，不经 `of_this_process`，这一点我只是看到，没判）。
- 没追：`TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL`（`gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs:100`）里还经别的入口取卡的测试；验证包里借卡通道的残留代码（`CardLeaseChannelOfTheCaller` 一族）是不是用户定案要删的「无用的逻辑」；`gpu_card_pool_or_report_not_run` 的 4 个返回 `Option` 的帮手以外、经帮手间接到登记函数的路径。
- 没看 `.claude/gate.d/` 里门禁 doc-text 的 comment-history 格会不会判 `gpu_test_start.rs:900` 的「照旧」；没跑命名检查（本报告不改 `crates/`）。
- 草稿目录 `~/.local/share/singlefs/work/gpu-cases-without-cards-code-r1-sonnet/` 里没有仓副本与编译目录（量的：`du -sh` 是 200K，都是文本与小脚本），交回前不用删。
