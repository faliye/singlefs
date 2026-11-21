# gpu-cases-without-cards-code-r1 攻方腿（Opus）报告（2026-11-21）

被判的三份文件今天的 sha256 与开工快照 `research/prompts/gpu-cases-without-cards-code-r1-snapshot/judged-sha256.txt` 一致（`gpu_test_start.rs` `73a42931…`、`gpu_test_start/tests.rs` `08bfd0dc…`、`crates/mutations.tsv` `c7e78682…`）。引代码一律主工作区今天的行号；第三、四节里不带文件名的「第 N 行」是 `mutation-rows-replaced-by-the-batch.tsv` 的行号，写「今天第 N 行」的是 `crates/mutations.tsv` 的。全部结论是读码推的，量法在「要主 agent 跑的命令」。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A1 没交卡串本该录而没录 | 打中（推的，分辨读法） | 17 处 GPU 对拍只把卡池把手当闸、GPU 那一趟走给定状态引擎；引擎没交卡串时本来写待判行交 GPU 服务判，现在在拿把手那一步就报本次未跑，一行待判行都不写（第一节） |
| 第 2 条 `Err(_)` 并臂 | 没打中，附一条 | 拿把手时交了卡串一律走 `Err`，`HandedOverByTheService(_)` 那一支在那个函数里走不到；写错的卡串在有卡那一台判红，对得上条款 |
| 第 3 条 `given_the_hardware` | 没打中（今天） | 两处调用都在要池里的卡之前返回；它是 `pub`、没有守新调用方的 |
| B1 公开入口没人钉 | 打中（推的） | 入口那十行的三种改坏——读此刻的环境、退回只判硬件、未跑不打那一行——四条单测与四行变异都不红；后两种分别是「panic」与「静默算过」 |
| B2 拿把手不起池不罩入口了 | 打中（推的，影响小） | 守它的集成测试与第 7877 行经 `given_the_hardware`，合入之后入口不走那里 |
| 第 8 条 单测测条款还是实现 | 部分测实现 | 「判硬件一回不调」「接线读一回」「原因措辞」三处不是条款要的；判法本身的五种改坏都抓得到 |
| 第 6 条 2102 行替换 | 没打中这一批；抽到两类旧问题 | 全量机检：原文都命中一次，改坏的意思 2102 行都没变（8 行只差可见性与花括号），筛选词都对得上；48 行分层抽查同。抽到 P1（改 WGSL、点名经引擎的场景用例，没交卡串时抓不到）与 P2（模块前缀筛选词把没红记成本次未跑），都是前像里就有 |
| 第 5 条 点名要卡的行 | 194 行（上界，按名字闭包），下界 109 | 其中 169 行改 WGSL，没有不用卡的测试守；第 934 行这类 CPU 函数只被要卡的用例守着 |
| 第 7 条 删掉的那一行 | 删得对，记录的依据点错了行 | 它要造的改坏已是现行写法；今天由第 8594 行与观察者那条单测守；记录第 475 行点的 6794、7931 两行不守特性 |
| 检查单九格 | 七格成立、一格成立附前提、一格推不出 | 见第六节 |

## 一、没交卡串的路（攻击面第 1、2、3 条与检查单「走不到的理由二次查实」）

### 打中 A1：本该写待判行、交 GPU 服务判的 17 处对拍，现在在拿把手那一步就返回

调用链（主工作区今天的行号）：

1. 用例开头只把卡池把手当闸用、把手丢掉：`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out/judges_and_libraries_of_the_given_states_engine.rs:241`：「if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {」。
2. 公开入口交这个进程的配置：`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start.rs:868`：「GpuConfigurationOfTheCaller::of_this_process,」。判法没交卡串就交回未跑、不判硬件：`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start.rs:911`：「Ok(CardsOfTheCallerForTheGpuTests::NotHandedOver { reason_not_run }) => {」。
3. 同一条用例往下本来要走的是给定状态引擎配了 GPU 的那一趟；引擎在调用方没交卡时不借卡、把判表交 GPU 批处理服务判：`crates/singlefs-verification/src/crash_amplification/given_states_engine.rs:370`：「调用方没交卡（起步时环境里没有服务交的卡串，取卡报没经服务给卡）的表交 GPU 批处理服务判」；那一支：`crates/singlefs-verification/src/crash_amplification/given_states_engine.rs:956`：「WayAfterTheCardsAreNotTaken::ThroughTheGpuVerificationServer => {」；枚举的定义：`crates/singlefs-verification/src/crash_amplification/gpu_pending_rows_and_packed_tables.rs:34`：「调用方没交卡：判表交 GPU 批处理服务判。」。
4. 规则给任务的那一份活正是这一条：`.claude/rules/verification.md:52`：「交的任务只跑录入、往判定库写待判行、读结论，不领卡、不分卡、不清场」。

合入之前（`merge-patch/crates.patch` 前像）公开入口只判硬件：`gpu_card_pool_or_report_not_run_given_the_hardware(test_name, &nvidia_hardware_of_this_machine())`。所以合入之前，在有 NVIDIA 卡的那台上、没交卡串的任务里，这 17 处走得到引擎、写待判行、由 GPU 服务判（推的，没量过：没读到合入之前那一趟的日志）；合入之后，两台上都在第 1 步返回，打一行「本次未跑」，一行待判行都不写。

把手只当闸用的 17 处（判法：`tools/classify_call_sites.py`，调用写在 `if … .is_none()` 里、把手不留；另 1 处 `gpu_unit_checks_the_given_states_engine_takes_its_cards_from_the_card_pool_of_the_process.rs:68` 在引擎已经判不了之后才调，不算）：

| 文件 | 行 |
|---|---|
| `crates/singlefs-verification/tests/crash_amplification_observer_images_judged_on_the_gpu_name_the_card.rs` | 224、346 |
| `crates/singlefs-verification/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs` | 1497、1581 |
| `crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out/judges_and_libraries_of_the_given_states_engine.rs` | 241、306、512、735、812、989 |
| `crates/singlefs-verification/tests/crash_injection_campaign/gpu_and_cpu_runs_stop_files_shares_and_other_hosts.rs` | 66、120、304 |
| `crates/singlefs-verification/tests/crash_injection_campaign/pool_after_the_writable_mount_that_follows_a_crash.rs` | 215、597 |
| `crates/singlefs-verification/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs` | 1150、1216 |

（量的：`call-sites.tsv` 里 `gate-only` 18 行，减去上面那 1 处；把手留着往下借卡的 150 处另算，它们借卡那一步才要卡，不在此列。）

四句：
- 分不分辨臂：分。两种读法——「GPU 用例」指所有挂 `gpu` 特性、开头取把手的用例（这一批的读法），或只指要在本进程借卡的那一类（引擎那一路不借卡，照规则第 52 行写待判行）——在这 17 处结局不同：前一种读法下报本次未跑是对的，后一种读法下是漏录。
- 系统当时看不看得到判别它的东西：看不到。公开入口只收用例名（`gpu_test_start.rs:863`），不知道调用方往下是借卡还是走引擎。
- 满足的是判据字面的哪一个分句：条款第一句「GPU 用例没交卡串的报本次未跑」字面上满足；与同一条里「只跑录入、往判定库写待判行、读结论」（第 52 行）的任务分工对不上。条款没写清「GPU 用例」管不管经引擎交服务判的那一类，交主 agent 判（判据本身可能要改，`.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」的「判别子观测不到」那一格）。
- 跑前给的改法在这几格上还中不中：这一批没给改法。我提的改法见第七节 G1（只在我的推理上走过、被攻过零轮）。

推翻条件：在没交卡串的任务里，把上表任一条（例如 `judges_and_libraries_of_the_given_states_engine::the_images_of_the_paired_harness_scenarios_judged_on_the_gpu_and_on_the_cpu_give_the_same_words`）开头那道闸拿掉再跑，日志里没有报 GPU 判的 `CRASH_POINT_REPLAY_GIVEN_STATES` 行、或在取卡与交服务那一步报错退出，A1 的「闸后面走得到服务」那一半就不成立（那时 A1 降为「17 处一直没覆盖，这一批没改变」）。量法写在「要主 agent 跑的命令」第 1、2 条。

### 第 2 条：交了卡串而卡项读不成（`Err(_)` 那一支）

- 拿把手那一步交给 `cards_for_the_gpu_tests_given` 的「这一台是哪一项」恒报错（`gpu_test_start.rs:907-909`），而 `cards_for_the_gpu_tests_given` 读卡串之前先调它：`crates/singlefs-verification/src/gpu_card_configuration.rs:948`：「Some(card_string) => parse_cards_by_priority(card_string, &addresses_of_the_hosts()?)」。所以拿把手时只要交了卡串（写得对也好、写错也好）结局一律是 `Err(_)`，`Ok(CardsOfTheCallerForTheGpuTests::HandedOverByTheService(_))`（`gpu_test_start.rs:916`）那一支在这个函数里走不到（推的，读码）。四条新单测里交卡串的两条（`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:229-231`、`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:259-261`）走的也是 `Err(_)`，不是名字说的「交了卡串」那一支。后果：改坏只落在 `HandedOverByTheService(_)` 那一支的变异抓不到（等价变异）；这一处不算违反条款，算注释与单测名字说的比代码多。
- 起池那一步照同一份配置读：`Err(reason)` 交成一张卡都不取（`gpu_test_start.rs:1699`），`decide_gpu_test_start(…, 0)` 在有 NVIDIA 独显的机器上交 `AdapterPresentButNoCardStarted`（`gpu_test_start.rs:229-234`），起池结局 `CardsPresentButNotUsable`，借卡与要池里的卡都 panic（`gpu_test_start.rs:1759-1763`）：判红，带配置那一句。与条款对得上（配置写错不算本次未跑）。
- 没有 NVIDIA 卡的机器上交了写错的卡串：拿把手那一步判硬件交 `Absent`，报本次未跑，写错的那一句不报（`gpu_test_start.rs:919-921`）。GPU 服务不往没卡的机器交卡串，这条路在今天的分工里到不了（推的）。

### 第 3 条：`gpu_card_pool_or_report_not_run_given_the_hardware` 给「有」

调用方只有两份集成测试（量的：`grep -rn gpu_card_pool_or_report_not_run_given_the_hardware crates/` 去掉注释与定义，2 个文件各 1 处调用）：
- `gpu_unit_checks_the_card_pool_handle_starts_no_card_pool_and_leases_no_card_until_its_first_lease.rs:40`：之后只走 `lease_card_or_the_not_run_line`（第 61 行），起池判成没卡交 `NotRun`，不 panic。
- `gpu_unit_checks_a_recording_only_pairing_through_the_card_pool_handle_starts_no_card_pool.rs:66`：把手交给 `judge_every_state_on_a_card_leased_from_the_pool`，只录那一趟在要池里的卡之前就返回：`crates/singlefs-verification/src/crash_judge_dispatch.rs:1515`：「StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {」，要池里的卡在其后：`crates/singlefs-verification/src/crash_judge_dispatch.rs:1523`：「states_per_scratch_lane(cards.smallest_maximum_storage_buffer_binding_size());」。用例自己把选段设成 `record`（第 62-65 行），不看调用方环境。
- 结论：今天两处调用都走不到 `card_pool_started_now` 的 panic（推的，读码）。这个函数是 `pub`、文档写明给「有」就绕过卡串，以后谁在只判那一趟（选段 `judge` 或不设）里用它，就会在没卡串的任务里撞上 `gpu_test_start.rs:1017` 的 panic；没有测试或门禁拦「新调用方」（推的）。

### 检查单「走不到的理由二次查实」：「只剩硬件前后不一致那一种」

- 经公开入口：拿把手时没交卡串的已经返回（第 911 行）；交了卡串的拿到把手，起池时照同一份 `of_this_process`（`gpu_test_start.rs:1678`）判。起池判成没卡（`NotRunNoGpuAdapter`）只有两处来源：`NotHandedOver`（第 1691 行，同一份配置在拿把手时已判成交了，到不了）与 `decide_gpu_test_start` 交 `NotRunNoGpuAdapter`（第 1726 行）——后者要 Vulkan 没有 NVIDIA 独显且 PCI 没有 NVIDIA 控制器或看不见总线，而拿把手时同一判法判过「有」，起池那一步重新枚举（第 1701-1705 行，不读 `nvidia_hardware_of_this_machine` 缓存的那一份），所以只剩两次枚举不一致：成立（推的，读码）。
- 经 `given_the_hardware`：文档已列为第二种来源（`gpu_test_start.rs:1010-1011`），今天调用方走不到（见上）。
- 那一句成立的前提是公开入口读的是 `of_this_process`。把它换成此刻的环境（第二节 B1），拿把手与起池可能读到不同的两份，第三种来源就出来了，而四条单测没有一条红。

## 二、公开入口读哪一份配置、四条新单测测的是条款还是实现（攻击面第 4、8 条）

### 打中 B1：公开入口这一层的三种改坏，四条单测与四行变异一条都不红

四条单测各自直调判法或接线（`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:186`、`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:227`、`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:257` 调 `gpu_card_pool_of_the_configuration_or_the_not_run_line`，`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:296` 调 `gpu_card_pool_of_the_configuration_of_the_caller_or_the_not_run_line`），没有一条调公开入口 `gpu_card_pool_or_report_not_run`（`gpu_test_start.rs:863-872`）。四行新变异的原文都在判法或接线里，没有一行落在第 866-871 行。所以公开入口那十行里的每一处都没人钉（推的，读码；量法在「要主 agent 跑的命令」第 3、4、5 条）：

| 改法 | 改在哪 | 违反条款的哪一句 | 四条单测 | 四行新变异 | 别的守着的 |
|---|---|---|---|---|---|
| B1a 读此刻的环境 | 第 868 行 `GpuConfigurationOfTheCaller::of_this_process,` 换成 `\|\| -> &'static GpuConfigurationOfTheCaller { Box::leak(Box::new(GpuConfigurationOfTheCaller::read_from_the_environment_now())) },`（`read_from_the_environment_now` 是 `pub(crate)`，`gpu_card_configuration.rs:858`，编得过） | 别的线程拿着改环境的锁把卡串换成临时那一份时，拿把手读到「交了」、起池照 `of_this_process`（`gpu_test_start.rs:1678`）读到「没交」：有卡那一台上要池里的卡就 panic（`gpu_test_start.rs:1017`），「本该报本次未跑而 panic」（要那种交错，单测帮手 `while_another_thread_holds_a_temporary_gpu_configuration` 造得出；今天集成测试里设回的是调用方那一份，推的） | 都绿（不经入口） | 都不碰 | 没有（推的；`the_card_pool_started_while_another_thread_holds_a_temporary_configuration_picks_from_the_configuration_of_the_caller` 只钉起池那一步，`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:395`） |
| B1b 入口退回只判硬件 | 第 866-871 行整段换成 `gpu_card_pool_or_report_not_run_given_the_hardware(test_name, &nvidia_hardware_of_this_machine())`（合入之前那一版） | 没交卡串、有卡那一台上交把手，要池里的卡 panic：这一批要修的那个红原样回来 | 都绿 | 都不碰（第四行改的是接线，`mutation-rows-appended-by-the-batch.tsv` 第 4 行，原文在第 883-887 行） | 只有在没交卡串、有卡那一台上跑一条要池里的卡的 GPU 对拍才红；没卡那一台上看不出（多机口径不一，推的） |
| B1c 未跑那一行不打 | 第 871 行 `.into_option_printing_the_not_run_line()` 换成 `.into_option_without_the_line()` 一类（`match … { Taken(t) => Some(t), NotRun { .. } => None }`） | 没交卡串的 GPU 用例照样返回成功、却不打 `NOT_RUN_NO_GPU_ADAPTER`：门禁当它判绿记库，「静默算过」 | 都绿 | 都不碰 | 没找到会红的（查过：`gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs` 只扫名字出现没出现；`gpu_unit_checks_the_given_states_engine_takes_its_cards_from_the_card_pool_of_the_process.rs:68` 只在引擎判不了之后调、返回值不断言打没打） |

四句：分辨臂——这里没有两条臂，被判的是「四条单测加四行变异钉住了公开入口」这一句，三种改坏都让它不成立；系统看得到判别它的东西——看得到（改的就是入口自己）；满足的判据分句——`.claude/rules/mutation-sampling.md` 与条款「GPU 用例没交卡串的报本次未跑」：B1c 让该打的那一行不打，B1a、B1b 让该报本次未跑的变成 panic；跑前改法——这一批没给，实现员第三份报告把「读的是这个进程的配置没被钉住」交了主 agent（`gpu-cases-without-cards-report-not-run-third/result.md`），我的改法见第七节 G2（被攻过零轮）。

为什么钉不住：入口打那一行就会让单测本身被门禁当成本次未跑（`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:174` 注释写的就是这条约束），所以实现员只钉了不打那一行的接线。B1a 到 B1c 三种都落在「接线之外、打那一行之前」这一截里。

### 打中 B2：合入之后，「拿把手时不起池、不领卡」不再罩着公开入口

- 守这条的那份集成测试经 `gpu_card_pool_or_report_not_run_given_the_hardware` 拿把手（`gpu_unit_checks_the_card_pool_handle_starts_no_card_pool_and_leases_no_card_until_its_first_lease.rs:40`），钉它的变异行（今天 `crates/mutations.tsv` 第 7877 行「GPU 卡池把手 拿到把手时就起池」）原文在那个函数里：`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start.rs:945`：「Some(GpuCardPoolStartedAtTheFirstLease {」。
- 合入之前公开入口就是调 `given_the_hardware`，那一行与那份测试连带罩着入口；合入之后入口交把手的是另一处（`gpu_test_start.rs:922-926`），两份代码分开了。把第 923 行那一支改成交把手之前先 `card_pool_or_the_not_run_line()` 起一次池：四条单测里交把手的那一条（`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:221`）只断言交了把手，照绿；第 7877、7878 行的测试不经这一支，照绿（推的）。
- 影响：交了卡串的任务里，GPU 用例开头就起池（起池只开账、不起设备，`gpu_test_start.rs:1627`），只录那一趟的「不起池」由没交卡串那一步先挡掉，所以今天的条款不被打破；被打破的是文件头写的「拿把手不起池」这一句（推的）。

### 攻击面第 8 条：四条单测把实现的哪几处当成了条款

条款只要「没交卡串的报本次未跑」，下面三处不是条款要的，换一种同样满足条款的写法会无故红：
- 没交卡串时「判硬件的那一步一回都不调」：`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:211`：「!is_the_hardware_judged.get(),」，同样的断言在 `crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:326`。先判硬件（进程里缓存着，便宜）、再照卡串报本次未跑的写法满足条款，这两条照红。
- 「接线读一回调用方那一份」：`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:323`：「"取卡池的接线读一回调用方那一份"」。`of_this_process` 是 `OnceLock`，读两回交的是同一份，读几回不改结局。
- 原因那一句点名「调用方没交卡串」（`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:202`、`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:313`）：钉的是 `gpu_card_configuration.rs:952` 那一句的措辞，不是本文件的判法。
这三处我判为「测实现」；它们让第一、四行变异抓得更稳，代价是换写法要连单测一起改。不算打中，交主 agent 定要不要放宽。

把判法改坏成不满足条款的写法而四条照绿的，只找到 B1 那三种（都在入口，不在判法里）。判法本身（第 902-930 行）里试过的改坏：没交卡串那一支往下判硬件（第一行变异，红）、`Err(_)` 并进未跑（交卡串那两条走的就是 `Err`，第二条红）、空白卡串当交了（第一条的 `Some("  ")` 红）、没卡那一支交把手（第三行变异，红）、有卡那一支报未跑（第二行变异，红）：都抓得到（推的，读码）。

## 三、2102 行整行替换：改没改意思、点名测试抓不抓得到（攻击面第 6 条）

### 全量机检（量的：读文件的脚本，不编不跑；脚本在 `<草稿目录>/tools/`）

| 查什么 | 判法 | 结果（2102 行） |
|---|---|---|
| 原文今天在源码里恰好命中一次 | 原文 `\n` 换成换行，在第 2 段点名的文件里数（`replaced_rows_profile_indexed.py`） | 2102 行都是 1 次 |
| 哪几段变了 | 前六段对后六段逐段比 | 只改第 5 段 1108 行；只改第 2 段 735 行；第 2、3、4 段 155 行；第 2、5 段 86 行；第 2-5 段 15 行；第 2、3 段 2 行；第 3、4 段 1 行。第 1 段（名字）与第 6 段（必须红的测试名）一行都没变 |
| 「改坏成什么」变没变 | 原文与替换文去掉共同前后缀，剩下的那一截前后比 | 2057 行逐字同；45 行不同，去掉空白之后剩 8 行（第 296、826、990、1474、1782、2018、2029、2079 行），逐行读过：都是代码搬进子模块后补的 `pub(super)`、花括号与末尾逗号，或第 2018 行装置源文件变成目录（`…/e202_gpu_cpu_gc_comparison.rs` 换成 `…/e202_gpu_cpu_gc_comparison`），改坏的意思没变 |
| 第 5 段的改法 | 筛选词前后比（`--release`、`--features` 等 cargo 参数另比） | 筛选词前面补模块路径 666 行、模块路径改了 543 行；cargo 参数一行都没变 |
| 筛选词够不够得着第 6 段那条测试 | 找定义那条测试函数的文件、算模块路径（加文件里至多两层 `mod … {`），带 `--exact` 的要相等、不带的要是子串（`substring_filter_paths.py`） | 带 `--exact` 700 行、不带 1402 行，都对得上 |

### 分层抽 48 行（层按 cargo 参数点名的包 × 变了哪几段；层内按行号等距取，`stratified_sample.py`）

每层抽几行：验证包只改第 5 段 10、只改第 2 段 4、第 2、5 段 4、第 2-4 段 4、第 2-5 段 3、第 2、3 段 1、第 3、4 段 1；实验包只改第 2 段 6、第 2-4 段 4；产品 GPU 测试包只改第 2 段 4、第 2、3 段 1、第 2-4 段 2；核心层只改第 5 段 1；prune-server 只改第 2 段 2、第 2、5 段 1。「够不够得着」一列：测试体或同一份文件里直接点到原文所在的函数的写「直接点到」；没直接点到的逐行读过一跳，写「推的」的是经产品路径间接走到、没逐跳追到底。

| 行（替换文件里的行号） | 层（包 \| 变了哪几段） | 原文今天命中 | 改坏的那一段（去空白后）前后 | 筛选词对得上点名测试 | 点名测试够不够得着 | 判定 |
|---|---|---|---|---|---|---|
| 57 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 经场景用例写数据单元走到（推的） | 没改意思 |
| 129 | singlefs-implementation-experiments\|file,original,replacement | 1 次 | 同 | 对得上 | 测试体里直接点到 `run_a_crash_branch` | 没改意思 |
| 186 | singlefs-verification\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `judge_a_pinned_red_example_replay` | 没改意思 |
| 238 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 测试体里直接点到 `receive_the_pieces_of_one_block` | 没改意思 |
| 248 | singlefs-verification\|file,original,replacement,arguments | 1 次 | 同 | 对得上 | 筛选词是整个模块前缀（不带 --exact），同一批里有打本次未跑的真卡用例：没红会被记成本次未跑 | 没改意思；没红会被记成本次未跑（P2） |
| 272 | singlefs-verification\|file | 1 次 | 同 | 对得上 | 同一装置的建表路径（推的） | 没改意思 |
| 329 | singlefs-verification\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `promised_violations_from_the_split_rows` | 没改意思 |
| 344 | singlefs-core\|arguments | 1 次 | 同 | 对得上 | 测试体里直接点到 `key_width_for_kind` | 没改意思 |
| 447 | singlefs-implementation-experiments\|file | 1 次 | 同 | 对得上 | 装置常量，同文件探针读它（推的） | 没改意思 |
| 463 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 经场景用例的挂载重读走到（推的） | 没改意思 |
| 596 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 测试体里直接点到 `most_states_whose_need_fits_the_capacity` | 没改意思 |
| 664 | singlefs-verification\|file,arguments | 1 次 | 同 | 对得上 | 测试体里直接点到 `started_with_its_owner_recorded` | 没改意思 |
| 716 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 经账本借还走到（推的） | 没改意思 |
| 748 | singlefs-verification\|file,original,replacement | 1 次 | 只差空白或可见性写法 | 对得上 | 测试体里直接点到 `lease_card` | 没改意思 |
| 806 | singlefs-verification\|file,original,replacement | 1 次 | 只差空白或可见性写法 | 对得上 | 源码扫描用例，扫这一模块（够得着） | 没改意思 |
| 839 | singlefs-verification\|file,original,replacement | 1 次 | 同 | 对得上 | 测试体里直接点到 `lines_read_past_comments_and_literals` | 没改意思 |
| 868 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 经场景用例推环入口走到（推的） | 没改意思 |
| 918 | singlefs-verification\|original,replacement | 1 次 | 同 | 对得上 | 测试体里直接点到 `journal_record_offsets_hint` | 没改意思 |
| 934 | singlefs-verification\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `check_unit_checksums_on_the_cpu` | 没改意思；点名测试要卡，没交卡串的任务里恒本次未跑（第三节） |
| 995 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 同文件测试经登记向量编号走到（推的） | 没改意思 |
| 1078 | singlefs-implementation-experiments\|file | 1 次 | 同 | 对得上 | 同装置拒因分类（推的） | 没改意思 |
| 1154 | singlefs-implementation-experiments\|file | 1 次 | 同 | 对得上 | 同装置拒因说明（推的） | 没改意思 |
| 1296 | singlefs-implementation-experiments\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `overwrite_order_of` | 没改意思 |
| 1356 | singlefs-implementation-experiments\|file,original,replacement | 1 次 | 同 | 对得上 | 测试体里直接点到 `submission_units_of_the_intent` | 没改意思 |
| 1386 | singlefs-implementation-experiments\|file,original,replacement | 1 次 | 同 | 对得上 | 测试体里直接点到 `comparison_control_holds` | 没改意思 |
| 1409 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 源码扫描用例（够得着） | 没改意思 |
| 1433 | singlefs-implementation-experiments\|file,original,replacement | 1 次 | 只差空白或可见性写法 | 对得上 | 测试体里直接点到 `unstable_ratio_judgement` | 没改意思 |
| 1457 | singlefs-implementation-experiments\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `rank_correlation` | 没改意思 |
| 1473 | singlefs-verification\|file,original,replacement,arguments | 1 次 | 同 | 对得上 | 测试体里直接点到 `drop` | 没改意思 |
| 1521 | prune-server\|file | 1 次 | 同 | 对得上 | 经容错客户端点读走到（推的） | 没改意思 |
| 1567 | singlefs-background-gpu-tests\|file,original,replacement | 1 次 | 同 | 对得上 | 测试体里直接点到 `small_pool_image` | 没改意思 |
| 1570 | singlefs-background-gpu-tests\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `batches_are_equal_and_at_least_the_minimum` | 没改意思 |
| 1602 | prune-server\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `replace_claim_lease` | 没改意思 |
| 1609 | singlefs-background-gpu-tests\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `fragment_segments` | 没改意思 |
| 1711 | singlefs-verification\|file,arguments | 1 次 | 同 | 对得上 | 测试体里直接点到 `a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels` | 没改意思；点名测试要卡，没交卡串的任务里恒本次未跑（第三节） |
| 1730 | prune-server\|file,arguments | 1 次 | 同 | 对得上 | 同文件测试起监听（推的） | 没改意思 |
| 1762 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 经可写挂载修复走到（推的） | 没改意思 |
| 1782 | singlefs-verification\|file,original,replacement,arguments | 1 次 | 只差空白、可见性或花括号 | 对得上 | 够得着，但没交卡串时用例在第 92-97 行先打本次未跑返回：恒本次未跑 | 没改意思；恒本次未跑（第三节） |
| 1846 | singlefs-verification\|file,arguments | 1 次 | 同 | 对得上 | 测试体里直接点到 `dispatch_width_after_a_dispatch` | 没改意思 |
| 1862 | singlefs-verification\|file,original | 1 次 | 只差空白或可见性写法 | 对得上 | 源码扫描（include_str 读这份文件，够得着） | 没改意思 |
| 1881 | singlefs-implementation-experiments\|file | 1 次 | 同 | 对得上 | 同装置并行判（推的） | 没改意思 |
| 1926 | singlefs-verification\|arguments | 1 次 | 同 | 对得上 | 改的是 WGSL；没交卡串时引擎把判表交 GPU 服务，服务用装好的内核、不用副本里改坏的那一份：抓不到（推的） | 没改意思；点名测试抓不到（P1） |
| 1953 | singlefs-background-gpu-tests\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment` | 没改意思 |
| 1986 | singlefs-verification\|file,arguments | 1 次 | 同 | 对得上 | 同文件派活加宽规则（推的） | 没改意思 |
| 2005 | singlefs-verification\|file,original,replacement | 1 次 | 同 | 对得上 | 测试体里直接点到 `workspace_loading_threads_wanted` | 没改意思 |
| 2018 | singlefs-background-gpu-tests\|file,original | 1 次 | 只差空白、可见性或花括号 | 对得上 | 摘要用例读这张目录表（够得着） | 没改意思 |
| 2025 | singlefs-background-gpu-tests\|file,original,replacement | 1 次 | 同 | 对得上 | 测试体里直接点到 `with_the_one_library_kept_for_this_process` | 没改意思 |
| 2043 | singlefs-background-gpu-tests\|file | 1 次 | 同 | 对得上 | 测试体里直接点到 `of_the_pool` | 没改意思 |

抽查结论：48 行里没有一行改了「改坏成什么」的意思，原文都恰好命中一次，筛选词都对得上第 6 段那条测试（推的部分是「够不够得着」那一列标「推的」的 15 行）。抽中的行里有两类问题，都不是这一批引进来的（前像里同样有），但这一批把这些行整行重写过一遍、没有顺手拦下：

**P1（第 1926 行，同类另有第 1477 行）：改的是 WGSL 内核，点名的是经给定状态引擎判的场景用例，没交卡串的任务里抓不到。** 引擎在没交卡串时把判表交 GPU 批处理服务判（第一节 A1 第 3 步）；待判行只带判表与旋钮栏，不带内核源码（`crates/singlefs-verification/src/crash_amplification/gpu_pending_rows_and_packed_tables.rs:249-255` 的参数里没有内核源码），服务用它自己编进去的那一份（`gpu-verification-server/Cargo.toml:14` 依赖验证包），副本里改坏的那份 WGSL 一次都不跑：用例照绿，这一行记没红（推的，没量过）。4 句：不分臂（判的是「这一行抓得到」）；系统看不到（服务不知道副本改了内核）；满足 `.claude/rules/verification.md`「测试不分档，红了只重跑红的」里「变异行点名的测试要跑得到」那一条的反面；没有跑前改法。量法见「要主 agent 跑的命令」第 6 条。

**P2（第 248 行，同形共 4 行：今天 `crates/mutations.tsv` 第 1528-1531 行）：筛选词是整个模块前缀 `gpu_unit_checks::gpu_test_start::tests::`、不带 `--exact`。** 这个模块里有一条在没交卡串时打本次未跑的用例（`crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs:95`：「not_run_line(test_name, "服务没经 GPU_CARDS_BY_PRIORITY 交卡")」），而变异门禁认本次未跑时不看是哪条用例打的：`scripts/mutation/crates-mutation-rows.py:366`：「点名的测试结果是 ok、输出里有这一族本次未跑行的，这一条记「本次未跑」」（`test=` 对不上点名测试名的照样记，同文件第 369-370 行）。所以这 4 行在没交卡串的任务里没抓到时记本次未跑、不记没红：没红被藏起来（推的）。抓到时照记抓到，不受影响。

## 四、点名要卡的用例的变异行（攻击面第 5 条）

判法（`tools/card_needing_closure.py`，量的，读文件、按名字连，会多算）：验证包里一个函数的体里调了取卡入口（`gpu_card_pool_or_report_not_run`、`usable_cards_or_report_not_run`、`first_usable_card_or_report_not_run`；`_given_the_hardware` 不算），或调了已判「要卡」的函数名，就记「要卡」，迭代到不动；第 6 段的测试名在这个集合里的行记「点名要卡」。没交卡串的任务里，这类用例在取卡那一步打本次未跑返回，这一行恒记本次未跑。

| 2102 行里（验证包 1406 行） | 行数 |
|---|---|
| 点名要卡、带 `--features gpu`、改的是 WGSL 内核 | 169 |
| 点名要卡、带 `--features gpu`、改的是 Rust | 25 |
| 合计点名要卡 | 194 |
| 带 `--features gpu`、点名不要卡（源码扫描、纯函数单测、经引擎的场景用例） | 238（其中改 WGSL 的 16） |
| 另：产品 GPU 测试包挂 `tests-on-a-real-gpu` 的（产品例外，照产品挂载开卡） | 2（第 2026、2027 行） |

只按直调与一跳判（`tools/card_needing_rows.py`）得 109 行，是这 194 行的下界；两种判法之间的 85 行是经两跳以上的帮手取卡的（例：今天 `crates/mutations.tsv` 第 2388 行那一类，点名 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 目标里的对拍）。

它们守的行为另外有没有不用卡的测试守着：
- WGSL 内核（169 行）：没有。WGSL 只在卡上执行；CPU 一侧另有池级 checker 自己的变异行，守的是 CPU 判法，不是内核。同一份 WGSL 里不用卡就抓得到的只有源码扫描那一类（16 行，例：第 1409 行「GPU 扫描内核 I-5.4 排分配记录的次序表改回借 C_ROW_ORDER」，点名 `scan_kernel_sources_of_the_record_tables::…`）。
- Rust 一侧（25 行）：例第 934 行「两段课题 4-2：单元校验和的 CPU 参照把头那一段当载荷算」，改的是 CPU 函数 `check_unit_checksums_on_the_cpu`（`crates/singlefs-verification/src/gpu_unit_checks/unit_checksums.rs:162`），而全仓调它的只有要卡的那一份测试（`grep -rn check_unit_checksums_on_the_cpu crates/` 去掉注释：`gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs` 第 130、190 行），那份测试开头就取卡池把手（同文件第 102 行）。CPU 代码只被要卡的用例守着：没交卡串的任务里这一处等于没覆盖。
- 同一份文件、同一处原文、点名不要卡的另一行（换一种改坏）：194 行里只有 4 行有（第 552、1288、1549、1990 行）。
- 「本次未跑」不判红，也不进「没红」：这 194 行此刻在没交卡串的任务里都是零覆盖；规则写的出路「崩溃判段之外用卡的测试交 GPU 服务代跑」还是「接口待定」（`.claude/rules/verification.md:52`），所以今天哪一台、哪一种任务都跑不出它们的结局（推的：没找到交卡串给验证包测试的入口）。这一条不是这一批引进来的；这一批让它从「有卡那一台照硬件判、没卡串时 panic」变成「两台都本次未跑」。

## 五、删掉的那一行原先守什么、现在谁守（攻击面第 7 条）

删掉的是「判器键表重算：依赖包并上来的特性不认（重算改回只照本包特性，与 entry_group_key 相同）」：原文是测试一侧重算键时传这一次编译的依赖包特性，改坏成传空表；点名 `the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree`。

- 它原先守的行为（测试重算照构建的依赖包特性收）今天不存在了：构建脚本两组都只按空表收（`crates/singlefs-verification/build.rs:185`：「&FeaturesEnabledByTheBuild::default(),」，同样在第 209 行），测试重算也是空表：`crates/singlefs-verification/src/crash_identity/compiled_judging_units_key_table/tests.rs:211`：「self.product_code_key_under(request, &FeaturesEnabledByTheBuild::default())」。那一行要造的改坏就是现行写法，删掉成立。
- 今天守「两种构建同一把键」的：被检测产品代码那一组是 `crates/mutations.tsv` 第 8594 行「判器引到的被检测产品代码那一组照这一次编译的依赖包特性收（带不带计数的两种构建两把键）」，点名 `the_judged_product_code_key_is_the_same_whether_the_build_turns_on_the_lookup_work_counters`（`judged_product_code.rs:70`）；观察者那几组由 `the_observer_product_code_keys_are_the_same_under_the_dependency_features_of_either_build`（`observer_keys.rs:187`）判两种特性下同一把键，构建脚本第 185 行那一处没有单独的变异行（改成照这一次的特性收，这条单测说两遍算出同一把，改了也等价，推的）。构建脚本读依赖包特性进表那一步另有第 4925 行（今天的行号）。
- 记录里删的依据点错了行：`records/2026-11-20-拆分带出的门禁与变异红.md:475`：「的是第 6794、7931 行」（那一句前半是「守「两种构建同一把键」」）。按删前那份表（`mutation-rows-after-judging-keys/mutations.before-deleting-4925.tsv`）读，第 6794 行是「判器键测试拷树：不拷 crates 之外的 path 依赖」，第 7931 行是「判器键表构建脚本：按组并行算键时每条线程都算第一组」，都不守特性；守它的是删前第 8595 行（今天第 8594 行）。结论（删得对）不变，依据那一句要改（记录不在被判的三份文件里，交主 agent）。

## 六、检查单九格

| 格 | 答 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立（推的），有一个前提没人钉 | 「只剩硬件前后不一致那一种」经公开入口成立（第一节末），前提是入口读 `of_this_process`；换成此刻的环境（B1a）就多出第三种来源，四条单测与四行变异都不红（第二节 B1）。`given_the_hardware` 那一种文档已列，今天两处调用都走不到 panic |
| 类型与名字 | 成立，附一条 | 新名字没有序数与略写。`Ok(CardsOfTheCallerForTheGpuTests::HandedOverByTheService(_))` 那一支在 `gpu_card_pool_of_the_configuration_or_the_not_run_line` 里走不到（交了卡串一律是 `Err`，第一节第 2 条）；`Err(_)` 丢掉的不只是「读不成的原因」，写对的卡串也走这一支。名字为 `…with_a_card_string…` 的两条单测实际钉的是 `Err` 那一支 |
| 不兼容旧代码与旧格式 | 成立 | `merge-patch/crates.patch` 源码那一段（第 1-140 行）与四条新单测逐行读过，没有认旧版本的分支与旧格式 |
| 不新建判定库 | 成立 | 改动不写库；A1 说的是少写了待判行，不是多了存放处 |
| 所有任务多机可用 | 成立，附一条 | 四条单测不读本机状态。附：合入之前「没交卡串」的结局随这台有没有卡而变（有卡那一台 panic 或走引擎、没卡那一台本次未跑），合入之后两台同一结局；B1b 那种改坏只在有卡那一台现形（第二节表） |
| 测试优先多线程 | 成立 | 四条是纯函数判法，毫秒级 |
| 带 --item 的命令与项 | 推不出 | 背景材料给了四行「抓到 4 条」的结局句，我没读到那几趟的日志（没给任务号），没核 |
| 新崩溃流多罩了什么 | 成立 | 没有新流 |
| 放的位置 | 成立 | 新单测在被测模块自己的 `#[cfg(test)]` 文件 |

## 七、我提的改法（只在我的读码与副本行上走过、被攻过零轮）

- G1（对 A1）：把只当闸用的那 17 处与要在本进程借卡的那 150 处分开：前者不调 `gpu_card_pool_or_report_not_run`（引擎在没交卡串时本来就交 GPU 服务判、不借卡），或另立一个入口只判「这台机器开不开 GPU 判段」；连带要把条款「GPU 用例没交卡串的报本次未跑」写清管不管经引擎的那一类。注意 P1：经服务判的是服务装好的那一份内核，所以 G1 换回的是「服务的内核对 CPU 判器」的对拍，换不回副本里改坏 WGSL 的变异覆盖。
- G2（对 B1a、B1b）：入口拆成不打那一行的 `pub(crate)` 一层（照 `of_this_process` 与 `nvidia_hardware_of_this_machine` 取）加打那一行的一层；单测在 `while_another_thread_holds_a_temporary_gpu_configuration` 里调不打那一行的那一层，断言结局与照 `of_this_process()` 判的相同、原因点名调用方没交卡串（不交卡串的单测进程里）。B1c 要另钉：`into_option_printing_the_not_run_line` 打没打那一行，或门禁一侧认「GPU 用例返回成功而没打本次未跑」。
- G3（对 P2）：今天第 1528-1531 行的筛选词改成 `--exact` 全名。
- P1 没有在这一批能做的改法：副本里的内核只有在本进程上卡时才跑，而条款不许验证包借卡；交主 agent 定这类行（改 WGSL、点名经引擎的用例）记什么。

| 改法 | A1 | B1a | B1b | B1c | B2 | P1 | P2 |
|---|---|---|---|---|---|---|---|
| G1 | 推的 | — | — | — | — | 不修（推的） | — |
| G2 | — | 推的 | 推的 | 不修（推的） | — | — | — |
| G3 | — | — | — | — | — | — | 推的 |

（全部「推的，待主 agent 跑」：副本上的阴性与阳性对照行已写好，见下一节第 3-6 条。）

## 要主 agent 跑的命令

副本 `/home/fy5090/.local/share/singlefs/work/gpu-cases-without-cards-code-r1-opus/copy` 在草稿目录的 `.deliverables` 写成「交接」。副本相对主工作区只改两处：A1 那条对拍拿掉拿把手的闸（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out/judges_and_libraries_of_the_given_states_engine.rs`，sha256 `74fe28b275190f9225708facd96d1019670bbffe0b299ab9e7f7d5003a5b3479`），`crates/mutations.tsv` 末尾加 5 行「攻方 …」（sha256 `7d9840945f431e6433b50cc62bee2349e7a4dd8259099b77b75be751dfebc4d6`）。副本上的数只算副本上的。

1. A1 今天的结局（主工作区、任务里不交卡串）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_given_states_of_the_images_harness_scenarios_hand_out 里的 1 条测试; 范围=1 个测试; 为什么=核攻方 A1：没交卡串时经引擎的对拍在拿把手那一步就报本次未跑、不写待判行；全域理由：看的是这条对拍整条走到哪一步，缩域会改掉要看的那一段' task-server/task submit -- cargo test --release -p singlefs-verification --features gpu --test crash_given_states_of_the_images_harness_scenarios_hand_out -- judges_and_libraries_of_the_given_states_engine::the_images_of_the_paired_harness_scenarios_judged_on_the_gpu_and_on_the_cpu_give_the_same_words --exact --nocapture`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看什么：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里有一行 `NOT_RUN_NO_GPU_ADAPTER test=the_images_of_the_paired_harness_scenarios_judged_on_the_gpu_and_on_the_cpu_give_the_same_words` 且 reason 含「调用方没交卡串」，没有 `CRASH_POINT_REPLAY_GIVEN_STATES` 行：A1 的「今天不录」那一半成立。
2. A1 的「闸拿掉就经服务判」那一半（副本）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_given_states_of_the_images_harness_scenarios_hand_out 里的 1 条测试; 范围=1 个测试; 为什么=核攻方 A1：拿掉拿把手那道闸之后没交卡串的对拍是否写待判行、由 GPU 服务判完；全域理由：看的是这条对拍整条走到哪一步，缩域会改掉要看的那一段' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-cases-without-cards-code-r1-opus/copy -- cargo test --release -p singlefs-verification --features gpu --test crash_given_states_of_the_images_harness_scenarios_hand_out -- judges_and_libraries_of_the_given_states_engine::the_images_of_the_paired_harness_scenarios_judged_on_the_gpu_and_on_the_cpu_give_the_same_words --exact --nocapture`
   - 在哪起：副本（上面那条路径）
   - 看什么：`exit=0`，日志里有 `CRASH_POINT_REPLAY_GIVEN_STATES` 行带 `judged_by=` 是 GPU 的那一种、`states_judged_by_the_gpu_here=` 大于 0，测试行 `ok`：A1 成立（这条对拍不借卡就走得通）。测试 FAILED 在取卡或交服务那一步、或卡住被服务停掉：A1 那一半不成立，A1 降为「这 17 处一直没覆盖」。要 GPU 服务在跑。
3. B1 三种改坏与阳性对照（副本上证红）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=证红 攻方 B1a、B1b、B1c 与第 8626 行阳性对照; 范围=4 行; 为什么=核攻方 B1：公开入口这一层改坏时四条新单测红不红；全域理由：点名的是验证包库单测，照全部崩溃状态判、不缩域，免得结局被缩域改掉' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/gpu-cases-without-cards-code-r1-opus/copy singlefs-verification '攻方 B1a GPU 卡池把手 公开入口读此刻的环境、不读调用方那一份' '攻方 B1b GPU 卡池把手 公开入口退回只判硬件（不看卡串）' '攻方 B1c GPU 卡池把手 公开入口未跑时不打那一行（静默交 None）' 'GPU 卡池把手 取卡池的接线不把调用方那一份交给判法、只看硬件（没交卡串也交把手）'`
   - 在哪起：仓根（`--copy` 指副本）
   - 看什么：三行「攻方 B1…」各记「没红」、第四行记「抓到」：B1 成立。三行里哪一行记「抓到」，B1 那一种不成立。「无效」（编不过）照无效记、不算推翻。
4. P2 宽筛选词把没红记成本次未跑（副本上证红）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=证红 攻方 P2 两行; 范围=2 行; 为什么=核攻方 P2：模块前缀筛选词下抓不到的改动记成本次未跑；全域理由：点名的是验证包库单测，照全部崩溃状态判、不缩域，免得结局被缩域改掉' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/gpu-cases-without-cards-code-r1-opus/copy singlefs-verification '攻方 P2 宽筛选词 点名的用例抓不到的改动（应记没红）' '攻方 P2 全名筛选词 点名的用例抓不到的改动（应记没红）'`
   - 在哪起：仓根
   - 看什么：「宽筛选词」那一行记「本次未跑」、「全名筛选词」那一行记「没红」：P2 成立。两行都记「没红」：P2 不成立。
5. P1 改 WGSL、点名经引擎的场景用例（副本上证红）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=证红 GPU 判器走读内核环长判据那一行; 范围=1 行; 为什么=核攻方 P1：没交卡串时改坏的内核不在本进程跑，点名的场景用例抓不到；全域理由：点名的场景用例经给定状态引擎判，照全部崩溃状态判才看得到它在哪一步交服务' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/gpu-cases-without-cards-code-r1-opus/copy singlefs-verification 'GPU 判器 走读内核环长判据去掉「在飞上限装进 4 字节」一格（给定状态用例带 GPU）'`
   - 在哪起：仓根
   - 看什么：记「没红」（或等 GPU 服务被停）：P1 成立；记「抓到」：P1 不成立（那时要看是哪一路判了副本里的内核）。

跑完判决引完数，副本由主 agent 删：`/home/fy5090/.local/share/singlefs/work/gpu-cases-without-cards-code-r1-opus/copy`（交接，交回时没删）。

## 没打中的形状

- 没交卡串时经公开入口 panic：试了把手交给 `card_pool_started_now`、`smallest_maximum_storage_buffer_binding_size`、`lease_card_or_report_not_run` 三条去路（`gpu_test_start.rs:968-1022`），没交卡串的在第 911 行已经返回，三条都走不到；交了卡串的只剩两次硬件枚举不一致。
- `Err(_)` 那一支在有卡那一台上：配置写错照判红（起池 `CardsPresentButNotUsable`），不静默。
- 只录那一趟起池领卡：两份用 `given_the_hardware` 的集成测试都在要池里的卡之前返回（第一节第 3 条）。
- 判法本身（第 902-930 行）里五种改坏都有单测抓（第二节末）。
- 2102 行替换：全量机检三项（命中一次、改坏的意思、筛选词）零命中；48 行分层抽查没有一行改了意思。抽查只覆盖 2.3%，「够不够得着」一列有 15 行是推的。
- 删掉的那一行：它要造的改坏是现行写法，删得对；守着今天那条不变量的行在（第五节）。

## 这条腿自己的限度

- 一处都没量：所有结论是读码推的；第二节的三种改坏、P1、P2 的阴性对照只写进了副本，结局等「要主 agent 跑的命令」第 1-5 条。
- A1 的「合入之前走得到 GPU 服务」那一半没核：没读到合入之前没交卡串那一趟的日志；第 2 条命令量的是今天的代码去掉闸之后，不是合入之前那一版。
- 「要卡」的判法按名字连函数，会多算（重名函数）；直调一跳的下界 109 行、按名字闭包的上界 194 行，真值在中间，没逐行核。
- 分层抽查每层取等距的几行，层内同类改法（只改模块路径）占大头；够不够得着只追了一跳。
- 检查单「带 --item 的命令与项」没核（没给任务号，读不到那几趟日志）。
- 没核 gpu-verification-server 的内核是不是真的只从它自己那一份编（P1 的前提），只读了 `gpu-verification-server/Cargo.toml:14` 与 `gpu_pending_rows_and_packed_tables.rs` 的参数表。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没往统一任务服务交任务，判决要引的数一条都没跑（命令在「要主 agent 跑的命令」）。
- 没写模型目录：这条腿只用了读文件的分析脚本（`<草稿目录>/tools/` 下 9 份，直接以 `python3 -I` 跑、不编不起项目代码），没有要复跑的模型，所以没有 `rerun.sh`。
- 记录 `records/2026-11-20-拆分带出的门禁与变异红.md:475` 那一句依据点错了行，不在写范围里，交主 agent。
