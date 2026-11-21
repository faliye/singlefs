# selftest-serial: 几格在同一个进程里改环境变量（PATH 最前面的假 cargo、判定库命名空间、假入口），格与格不能同时跑
"""E171 逐行驱动（登记 5.3「研究一侧」、5.7）：坏实现表的每一行——拷仓、改坏、编、跑装置（`row` 模式）、跑冻结的 O3 执行器、留下导出目录与一份 `row-result.tsv`。

跑法（在仓根，整条命令交 `task-server/task submit --`；内存额度由服务按记录给，驱动里不套包装；编译、装置与执行器直接起，线程数变量按线道线程数设进各自的环境，`frozen_o3.thread_environment`）：
  python3 row_driver.py --repo-root <仓根> --work-root <草稿目录> --output-root <输出根> [--rows 1,6 | --first-slice] [--empty-mutation]
      [--threads <每线道，缺省 = 登记的重活默认并行度 ÷ --lanes>] [--lanes 1] [--positive-control O1-对拍|O1-崩溃判官|O2|O3] [--disable-oracle O1|O2]
      [--no-executor] [--o3-control-bad-states <E170 bad-states 的导出目录>] [--snapshot-from <冻住的树>]
      [--selected-rows <表>] [--other-selected-rows <别的表> …] [--row-range <起>-<止>] [--empty-mutation-only]
      [--lease-card-per-device-run]
  python3 row_driver.py --selected-rows <表> --print-shard-ranges <片数> [--shards-may-cut-groups]      # 只打行区间（默认按文件组对齐；加后只按行数均分），不跑
  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --selftest
      自证（下面「自证」）：只在临时 prune-server 底下跑；不起 cargo、编译、装置、统一任务服务
选行：不给 `--rows`、`--first-slice`、`--row-range` 就是表里全部行；给了 `--rows`、`--first-slice` 是指定范围，驱动往标准错误打一行
`row_scope scope=narrowed …`，缩过的那一趟不当正式产物。
分片：`--row-range <起>-<止>` 取表里序号落在闭区间里的行，打 `row_scope scope=shard rows=… of=… range=…`；各片写进同一个 `--output-root`
（每行一个 `row-<序号>/`，互不相交），各片各用自己的 `--work-root`，可以分给两台。区间不许把一个文件组切开（补位只在文件组内换行，
切开了两片会挑同一个补位）；`--keep-compile-failures-as-they-are` 不补位，不受这一条限。`--print-shard-ranges` 打的区间都对齐文件组。
合并：各片跑完，同一条整表命令（不给 `--row-range`）再起一趟：各片判完的行都在判定库里（下面「判定库」），键相同的从库里取回、解进这个输出根的
行目录，不重编不重跑；整表的 `row …` 与 `table …` 行由这一趟出，比对器读这个输出根的 `row-<序号>/` 和这一趟的标准输出。
只取（不现判；起法是 `python3 shared_steps.py take-rows <选行与键的参数>`，参数与这里同名：--repo-root、--work-root、--output-root、--selected-rows、--rows、
`--row-range`、--empty-mutation、--empty-mutation-only、--positive-control、--disable-oracle、--no-executor、--o3-control-bad-states、--snapshot-from，另加 --check-only）：
  take_rows_into_the_output_root 照这一趟的快照（<--work-root>/snapshot）与 row_request_lines 算出和现判那一趟逐字相同的请求，rows_from_the_library 逐行照键查库
  （只查不判：不起 plan、不抽复用哨兵）：键相同、判绿、产物读得回且 sha256 对得上的行解进 <--output-root>/row-<序号>/（先删那里留的；--check-only 只查不写），
  `row …` 与 `table …`（lanes=0，只取不起线道）照现判那一趟的形打进标准输出；库里没有的行一行都不编、不起装置，标准错误逐行打
  `E171_ROW_NOT_IN_THE_LIBRARY row=<序号> reason=<为什么>`（拒算的行、库里没有这把键、库里这把键记着红、产物读不回或对不上），末行
  `E171_ROWS_TAKEN taken=<取到几行> missing=<缺几行> of=<点到几行> check_only=<yes|no>`；有缺的那一趟不打 `row …`、`table …`。
  退出码（shared_steps.py take-rows 交的）：0 点到的行都取到；4 有行库里没有（逐行点名，出路是经服务交 e171-row-shard.sh 补判）；2 用法错、命令行设了弄坏开关；
  75 prune-server 连不上（照客户端的码）。
  这一路的代码只是本文件里不带缺省值的几个新函数加 shared_steps.py 的子命令：本文件的模块级语句（import 时执行的赋值、类头、带缺省值的 def 头）进每一行的判法闭包，
  main 经 shared_steps.py 的 run_control_row 起本文件那一条子进程边进共用前置每一项的判法闭包；这两处一处都没动，行键与项键不随只取这一路变。
`--empty-mutation-only` 只跑空变异（行目录 row-0），不跑表里的行：阳性对照的每一趟用它（比对器只读对照输出根的 `row-0/dump`）。
装置的 `row` 模式经给定状态引擎判 O2，引擎要环境里有 `CARGO`（记编这个二进制的工具链）；驱动给装置的环境补上 `CARGO`（没有就取 `PATH` 上的 cargo），
编装置带 `--features gpu`（配了 GPU 的环境里不带特性编的二进制判段报本次未跑）。装置的输出里有 `NOT_RUN`、工具链缺失或「起不了给定状态表的引擎」的句子，这一行记测量管子的缺陷（`device_output`），驱动退 5。
按需领卡（`--lease-card-per-device-run`；入口文件头写按需领卡声明、服务起跑不分卡的那一处给：e171-row-shard.sh 的各片与合并）：
每一行的装置那一步分录入、判两段起（run_device_on_a_leased_card 经 device_card_lease.record_then_judge_on_a_leased_card，两段的规矩在同目录 device_card_lease.py 文件头
「装置分录入、判两段起」）：录入段（`--stage record`）不持卡、环境里没有 `GPU_CARDS_BY_PRIORITY`；录入段退 0 才领一张卡、交给判段（`--stage judge`，参数与录入段逐字相同）
的 `GPU_CARDS_BY_PRIORITY`，判段跑完照卡串还掉（判段退非 0 照还）；录入段退非 0 不领卡、判段不起，这一行的装置退出码就是录入段的。录入段的标准输出转进 `device.err`，
`device.out` 只收判段的标准输出（与一段起时的 `device.out` 同一份内容）；两段合用这一行的限时，判段只给录入段剩下的。同步、编译、执行器这几步一张卡都不持，
几条线道的录入段可以同时跑，判段各领各的。环境里没有领卡通道：开跑前打一句、退 78，一行都不跑；
领卡没成：这一行不起判段、记测量管子的缺陷（`card_lease`，不写 `row-result.tsv`，重交同一条命令照没做完重做），各线道不再领新行，驱动退 5；还卡没成：标准错误打一行「!」，这一行照已跑的算。
不给这个参数不领卡、不分段，装置一段起，照此刻环境里的 `GPU_CARDS_BY_PRIORITY`。
F4 扩表（登记 5.2「扩表」）：`--selected-rows bad-implementations-step-4-only.tsv --other-selected-rows bad-implementations-step-8.tsv`；
补位跳过两张表里的每一个名字（「扩表与补位都不重挑已跑过的行」）。
一次跑里先把仓（或 `--snapshot-from` 给的树）拍一张快照（`Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`，不带 target，外加每台默认重活线程数的登记文件
`.claude/heavy-thread-default`：验证包（`crates/singlefs-verification`）经 include_str! 在编译时读它，线道的树里没有就编不过；快照摘要只算 `.rs` 与 `Cargo.*`，不含它），每条线道从快照同步出自己的树与编译目录（线道之间不共享，
同一条线道里跨行复用编译目录，所以第一行是冷编译、之后是增量），每行改一处、编、跑、再把改过的文件还原。
增量编译的前提（登记修订 5）：cargo 只在一个包的源文件修改时间晚于上一次编它的时刻时重编；还原（`copy2`）与同步（`rsync -a`）都把修改时间带回快照那一刻，
所以每条线道记一份 `build-record.tsv`（上一次编完的时刻、自那以后改过还没干净重编的文件），每行同步之后先把那几个文件的修改时间置成现在，编之前核它们内容与快照相同、
修改时间晚于上一次编完，编完核 cargo 报的这些文件所在的包都重编了（`fresh=false`）；三样任一不过是测量管子的缺陷，这一行不跑装置、各线道不再领新行、驱动退 5。
工作根里已有编译目录而没有这份记录（旧驱动建的），或快照与上一次在这个工作根里跑的不同，驱动一行都不跑、退 2：换一个新的 `--work-root`。
表里的行照名字取（登记 5.2：表若已变不重挑），
行文与登记 5.9 的 sha256 不同照今天的行跑、另记。编不过、锚点不唯一、找不到的行按补位规则换同一文件组里的下一行（编之前的失败与编不过合起来至多三次；
换上的行编不过再接着换），产物里带「补位自 <原名>」（`substituted_from`），试过而没用上的每个名字记成 `failed_attempt_<序>=<失败>:<名字>`。
判定库（.claude/rules/verification.md「判定库只有一份」；算键、查库、入库与复用哨兵都经 scripts/experiment-apparatus/long_run_item_verdicts.py 的
plan / record / finish，本地不建库目录）：
  一行一项（项名 row-<序号>），存进 prune-server 的 apparatus-items 族、种类 e171_three_oracle_collusion（LIBRARY_KIND）：items/<键>.out 是 pack_directory
  打的整个行目录（按相对路径排好序、修改时刻置 0，同一份内容同一串字节），items/<键>.verdict 是判定（比法 timing：行目录里带耗时）。
  键照 long_run_item_verdicts.py 文件头「请求文件」，由 admission.py 的 compute_unit_key 在这一趟的快照上算（快照当 --root；row_request_lines）：
    入口   装置二进制 e171_three_oracle_collusion 的 main（feature:singlefs-implementation-experiments/gpu、profile:release）：单元闭包、工具链、闭包真引到的依赖版本
    judge  本文件的 run_row_on_the_lane，`--positive-control O3` 另加 o3_bad_image_judge.py 的 main（judge-root 是本文件所在的仓，只当索引）
    line:  行号、名单那一行原文、改坏用的变异行原文（改坏之后的源码不进闭包，由这一行进键）、是不是空变异、--positive-control、--disable-oracle、
           --no-executor、判的路线（多机配置开没开 GPU，judging_route）、冻结 O3 的冻结行摘要（o3_freeze_digest；--no-executor 写 -）；原文照 JSON 转写
    file:  --o3-control-bad-states 那个目录里每个文件的内容（@files，路径不进）
    不进键的参数：--keep-compile-failures-as-they-are、--selected-rows 的别的行与 --other-selected-rows（只管编不过之后的补位，补位的行不入库）、
           --lease-card-per-device-run（只管卡从哪来）、--threads、--lanes，与只管挑哪几行的 --rows、--first-slice、--row-range、--empty-mutation-only
  拒算（照跑、不复用、不入库）：开跑前照名字取不到变异行、或锚点在快照里不是恰好一次的行（要补位，补位挑行跨行有关，NEEDS_SUBSTITUTION），
    改坏那一处的原文或替换文里有 MODULE_SET_TOKENS 之一的行（改坏之后连上或断开的模块闭包认不全，MUTATION_TOUCHES_THE_MODULE_SET），这两种不进请求
    （标准错误打 E171_ROW_ITEM row=<序号> refused=<哪一种>）；admission 算不出键的由 long_run_item_verdicts 记拒算。
  复用的行：开跑前从库里取回、解进 <输出根>/row-<序号>/（先删那里留的），`row …` 行照它的 row-result.tsv 打；比对器读行目录的方式不变。
  要现判的行：输出根里留着的行目录不当复用来源，一律删掉重做；线道判完一行当场经 record 记（中途停下重交同一条命令，判过的从库里复用、只补没判的）：
    结局可复用的（outcome_is_storable：status ok、没补位、装置不是超时）记绿、存行目录；编不过、补位换了行、超时、测量管子的缺陷、驱动出错，
    照同一把键记红（盖掉原有的绿），下一趟现判。
  plan 与 finish 的逐项行、总计「LONG_RUN_ITEMS … 现判 N、复用 M、拒算 R」只进标准错误。finish 判红（复用哨兵红、键中途变了）驱动退 1；
    prune-server 连不上照客户端的码退（75）：线道里记库出了错，各线道跑完手上那一行就不再领新行。
自证（--selftest，五格，逐格串行：几格共用 PATH 最前面那一份假 cargo，算键时 admission 问判法闭包里点名的 cargo 的版本，不起真 cargo）：
  只在临时 prune-server 底下跑（客户端 selftest_service_problem 判，不在退 2、一格都不跑）。
  reuse-three-questions  long_run_item_verdicts.py reuse-cells 那六格（复用三问、key-independent-of-directory、reuse-sentinel 与哨兵的三种红；打 REUSE_SELFTEST
      script=scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py …）：样板树是空的（reuse-cells 往每一格的树里写假装置入口），
      起法是本文件的 selftest-round <树>（只在临时 prune-server 底下认：row-1、row-2 两行在那棵树上走一趟 plan、线道、record、finish，
      判一行换成写一份只随行号变的 row-result.tsv；有一行没有 row-result.tsv 退 1）
  refused-rows-are-judged-outside-the-library  要补位的行与碰到模块集合的行两趟都现判、不进请求；能算键的两行再起那一趟复用
  unstorable-outcome-is-judged-again  头一趟 row-2 的装置超时（device_exit 253）：再起那一趟 row-2 现判、row-1 复用
  library-failure-in-a-lane-stops-the-run  线道里 record 抛客户端的读写失败（连不上）：驱动照客户端的码退 75
  take-only-names-the-rows-the-library-lacks  头一趟只判 row-1（记绿）；只取那一趟点 row-1、row-2：row-1 解进输出根，row-2 点名「库里没有这把键」、
      不解、判一行的函数（selftest_round 里换上的那一份）一次都没被叫；--check-only 那一趟两行都不写进输出根、照样点名 row-2
      （格表 SELFTEST_CELLS 是模块级赋值、进行键闭包，这一格接在 selftest 里，不改那张表）
  refused-rows-are-judged-outside-the-library 与 unstorable-outcome-is-judged-again 两格另核键变过（two_rounds）：两趟计划里同一项的键不同、
      或 finish 报了键在一趟中途变了。判法闭包照 frozen_o3.OWN_REPOSITORY_ROOT 那棵树（不设 E171_REPOSITORY_ROOT 就是本仓）算，自证跑着的时候那里闭包碰到的源码被改了，
      再起那一趟就对不上头一趟记的键、照现判；键变过的，这两格红、打「键变过：…」那一句，结局不算数，等改动定下来再起同一条命令。
  末行 ROW_DRIVER_SELFTEST cells=5 red=<红几格>；退 0 全绿、1 有红、2 不在临时服务底下。
每行一个目录 `<输出根>/row-<序号>/`：`row-result.tsv`（key<TAB>value）、`device.out`、`device.err`、`compile.log`（最后一次编的诊断）、
`compile-failed-<序>.log`（编不过的那几次的尾部）、`dump/`（装置导出目录）、
`o3.out`（冻结的执行器对这一趟导出的批 1、批 2 的判定，登记 5.4）、`o3.log`、`o3-control.out`（`--positive-control O3` 时）。
O3 一步：开跑前核一次冻结行（Z2，不过整趟不跑）；每行装置退 0、导出了至少一条流，就起冻结的执行器（只列导出了的流）；`--no-executor` 是 Q3-核「驱动不跑执行器」那一次。
线道从一条共用的队列里领行（先到先得），每条线道一次一行。耗时在这个进程里用 `time.monotonic_ns` 计：拷仓与改坏、编、装置、执行器各一段，整表一段（`table` 行）；装置自己的分段耗时在它的 `E7TIMING` 行里。
进度：各线道开跑之后每过 PROGRESS_INTERVAL_SECONDS（300 秒）往标准错误打一行 `row_progress done=<做完几行> of=<这一趟几行> running=<row-<序号>@lane<线道>,…|none> elapsed_s=<秒>`，
只进标准错误，标准输出（`row …`、`table …` 行）一个字节不多；一行跑得再久，日志也照这个间隔涨。
弄坏开关只在自证里认：证红的是 test_synced_entries.py（在进程里调 sync_tree）与 test_collusion_pipeline.py（在进程里调 run_with_time_limit）、在进程里调 main 的几份单测，
与临时 prune-server 底下的 --selftest、selftest-round；命令行别处设了 E171_ROW_DRIVER_BREAK 一律在参数解析之前退 2、点名它只在自证里认。
逐项写开关名、弄坏哪一处，下一行起写打开后红的测试（unittest 的失败行 「FAIL: <测试名> …」 的句首）：
  E171_ROW_DRIVER_BREAK=registry-not-synced  快照与线道的树都不带 `.claude/heavy-thread-default`
      「FAIL: test_the_thread_registry_reaches_the_snapshot_and_the_lane_tree」
  E171_ROW_DRIVER_BREAK=path-dependencies-not-synced  快照与线道的树不带 crates 经 path 依赖指到的仓根目录（prune-server/，第 3 次重跑任务 11511 编不过的原因）
      「FAIL: test_a_path_dependency_outside_crates_reaches_the_snapshot_and_the_lane_tree」
  E171_ROW_DRIVER_BREAK=timeout-not-enforced  行的限时不执行、一直等（驱动里去掉内存包装时把它的限时一并丢掉的样子）；证红的是 test_collusion_pipeline.py
      「FAIL: test_a_row_over_the_time_limit_is_killed_as_a_group_and_exits_253」
  E171_ROW_DRIVER_BREAK=timeout-kills-only-the-child  超时只杀直接起的那个进程、不杀整组（孙进程留着）
      「FAIL: test_a_row_over_the_time_limit_is_killed_as_a_group_and_exits_253」
  E171_ROW_DRIVER_BREAK=device-without-cargo-variable  装置的环境里不补 `CARGO`
      「FAIL: test_the_device_environment_names_the_cargo_that_records_the_toolchain」
  E171_ROW_DRIVER_BREAK=compile-without-gpu-feature  编装置不带 `--features gpu`
      「FAIL: test_the_device_is_built_with_the_gpu_feature」
  E171_ROW_DRIVER_BREAK=row-range-ignored  `--row-range` 不剪行（各片都跑整表）
      「FAIL: test_a_row_range_takes_only_the_rows_inside_it」
  E171_ROW_DRIVER_BREAK=empty-mutation-only-runs-the-table  `--empty-mutation-only` 照样跑表里的行
      「FAIL: test_the_empty_mutation_only_runs_row_zero」
  E171_ROW_DRIVER_BREAK=device-output-defect-ignored  装置输出里的 `NOT_RUN` 与工具链缺失不记测量管子的缺陷
      「FAIL: test_a_device_output_that_says_not_run_or_lacks_the_toolchain_is_a_defect」
  E171_ROW_DRIVER_BREAK=configuration-check-not-synced  快照与线道的树不带多机配置判法和它 source 的文件（CONFIGURATION_CHECK_ENTRIES；main 里缺了就拒的那一道也一并关掉）
      「FAIL: test_the_configuration_check_and_what_it_sources_reach_the_snapshot_and_the_lane_tree」
      「FAIL: test_a_snapshot_without_the_configuration_check_is_refused_before_any_row_runs」
  E171_ROW_DRIVER_BREAK=engine-start-refused-ignored  装置输出里「起不了给定状态表的引擎」不记测量管子的缺陷
      「FAIL: test_a_device_that_could_not_start_its_engine_is_a_defect_but_a_bad_implementation_that_crashes_is_not」
      「FAIL: test_an_engine_start_refusal_stops_the_row_as_a_measurement_pipe_defect_and_exits_5」
  E171_ROW_DRIVER_BREAK=device-run-twice  装置退非 0 时再跑一次（装置与执行器各只跑一次，不因退出码重跑）
      「FAIL: test_the_device_and_the_executor_run_once_whatever_their_exit_code」
  E171_ROW_DRIVER_BREAK=executor-run-twice  执行器退非 0 时再跑一次
      「FAIL: test_the_device_and_the_executor_run_once_whatever_their_exit_code」
  E171_ROW_DRIVER_BREAK=progress-not-reported  一行跑得比进度间隔久也不打 `row_progress`
      「FAIL: test_a_row_longer_than_the_progress_interval_reports_progress_on_stderr_only」
按需领卡的几处（证红的是 test_device_card_lease.py，在进程里调 main，编译与装置换成桩，领卡客户端是小仓里假的 task-server/task）：
  E171_ROW_DRIVER_BREAK=card-held-through-the-compile  一行开头（同步、编译之前）就领卡，持到判段跑完
      「FAIL: test_no_card_is_held_while_compiling_or_recording_and_one_is_taken_and_returned_around_each_judge_stage」
      「FAIL: test_a_refused_card_lease_is_a_measurement_pipe_defect_and_the_judge_stage_is_not_started」（拒领卡时一行开头就抛，记成 driver_error、退 1）
      「FAIL: test_a_failed_record_stage_takes_no_card_and_starts_no_judge_stage」「FAIL: test_a_failed_judge_stage_returns_its_card」（编之前多了一回领卡）
  E171_ROW_DRIVER_BREAK=card-kept-after-the-device  判段跑完不还卡
      「FAIL: test_no_card_is_held_while_compiling_or_recording_and_one_is_taken_and_returned_around_each_judge_stage」
      「FAIL: test_two_lanes_running_their_devices_at_once_hold_different_cards」
      「FAIL: test_a_failed_card_return_is_reported_and_the_row_stands」（不还就没有还卡那一句）
      「FAIL: test_a_failed_judge_stage_returns_its_card」
  E171_ROW_DRIVER_BREAK=lanes-share-one-card  几条线道同时到判段，判段都拿最先领到的那一张
      「FAIL: test_two_lanes_running_their_devices_at_once_hold_different_cards」
      「FAIL: test_no_card_is_held_while_compiling_or_recording_and_one_is_taken_and_returned_around_each_judge_stage」（一条线道的后一行也拿到最先那一张）
  E171_ROW_DRIVER_BREAK=card-lease-refusal-runs-the-device  领卡没成照起判段（不带卡），这一行照判
      「FAIL: test_a_refused_card_lease_is_a_measurement_pipe_defect_and_the_judge_stage_is_not_started」
  E171_ROW_DRIVER_BREAK=card-lease-channel-unchecked  开跑前不核领卡通道（没有通道照跑，到判段之前领卡才红）
      「FAIL: test_without_a_lease_channel_the_driver_exits_78_before_any_row」
  E171_ROW_DRIVER_BREAK=card-return-failure-unreported  还卡没成不打那一行「!」
      「FAIL: test_a_failed_card_return_is_reported_and_the_row_stands」
  E171_ROW_DRIVER_BREAK=card-held-through-the-record-stage  录入段之前就领卡，录入段拿着卡串跑
      「FAIL: test_no_card_is_held_while_compiling_or_recording_and_one_is_taken_and_returned_around_each_judge_stage」
      「FAIL: test_a_failed_record_stage_takes_no_card_and_starts_no_judge_stage」（录入段失败之前已经领了一回）
      「FAIL: test_a_failed_judge_stage_returns_its_card」
  E171_ROW_DRIVER_BREAK=cards-left-in-the-record-environment  录入段的环境里留着驱动环境里的 GPU_CARDS_BY_PRIORITY
      「FAIL: test_the_record_stage_environment_carries_no_cards」
  E171_ROW_DRIVER_BREAK=card-returned-before-the-judge-stage  领了卡先还、再起判段（判段跑的时候一张卡都不持）
      「FAIL: test_no_card_is_held_while_compiling_or_recording_and_one_is_taken_and_returned_around_each_judge_stage」
      「FAIL: test_a_failed_judge_stage_returns_its_card」
  E171_ROW_DRIVER_BREAK=judge-stage-after-a-failed-record  录入段退非 0 照领卡、起判段
      「FAIL: test_a_failed_record_stage_takes_no_card_and_starts_no_judge_stage」
  E171_ROW_DRIVER_BREAK=card-kept-after-a-failed-judge  判段退非 0 不还卡
      「FAIL: test_a_failed_judge_stage_returns_its_card」
判定库那一层（文件头「判定库」；证红的是 --selftest 的格，自证项「✗ <格名>」，带 FAIL 的另由 test_row_shards.py 证红）：
  E171_ROW_DRIVER_BREAK=item-finish-ignored  驱动不认 long_run_item_verdicts finish 的退出码
      自证项「✗ reuse-three-questions」（reuse-cells 里 sentinel-rejudged-red、forced-verdict-differs-red、forced-still-reused-red 三格要的退 1 变成退 0）
  E171_ROW_DRIVER_BREAK=refused-row-requested  要补位、碰到模块集合的行照常进请求
      自证项「✗ refused-rows-are-judged-outside-the-library」
      「FAIL: test_a_row_that_needs_a_substitute_is_kept_out_of_the_library」
  E171_ROW_DRIVER_BREAK=unstorable-outcome-recorded-green  结局不可复用的行（编不过、超时……）照绿记、存行目录
      自证项「✗ unstorable-outcome-is-judged-again」
      「FAIL: test_a_row_that_did_not_compile_is_judged_again_on_the_next_run」
  E171_ROW_DRIVER_BREAK=reused-row-not-unpacked  复用的行不从库里解进行目录
      自证项「✗ reuse-three-questions」（selftest-round 头一趟之后各趟有行没有 row-result.tsv、退 1）
      「FAIL: test_sharded_runs_merge_into_the_same_rows_and_lines_as_the_unsharded_run」
  E171_ROW_DRIVER_BREAK=lane-library-failure-swallowed  线道里记库出错照常往下领行，驱动不照客户端的码退
      自证项「✗ library-failure-in-a-lane-stops-the-run」
  E171_ROW_DRIVER_BREAK=selftest-keys-moved-between-rounds  两趟那两格在再起那一趟之前改现场里假装置入口 alpha_detail 的返回值（row-1 的入口 measure_alpha 的闭包）
      自证项「✗ refused-rows-are-judged-outside-the-library」「✗ unstorable-outcome-is-judged-again」（两格都打「键变过：row-1 …」那一句）
  E171_ROW_DRIVER_BREAK=take-only-judges-missing-rows  只取那一趟（rows_from_the_library）缺行就现判：库里没有的行交给现判那一套判完，当取到了报
      自证项「✗ take-only-names-the-rows-the-library-lacks」
      「FAIL: test_take_only_names_a_row_the_library_lacks_and_runs_nothing」
"""

from __future__ import annotations

import argparse
import contextlib
import dataclasses
import gzip
import hashlib
import io
import json
import os
import queue
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import tempfile
import threading
import time

import device_card_lease
import frozen_o3
from run_directory import STREAM_NAMES
from bad_implementation_table import (MAXIMUM_SUBSTITUTIONS_PER_ROW, STATUS_ANCHOR_NOT_UNIQUE, STATUS_LOCATED, STATUS_NOT_FOUND,
                                       Located, MutationRow, SelectedRow, anchor_hit_count, apply_mutation, empty_mutation,
                                       group_names_of_file, locate_by_name, read_mutation_rows, read_selected_rows,
                                       substitute_for)

HERE = os.path.dirname(os.path.abspath(__file__))
DEVICE_BINARY_NAME = "e171_three_oracle_collusion"
DEVICE_PACKAGE = "singlefs-implementation-experiments"
DEVICE_FEATURES = ("gpu",)
TOOLCHAIN_MISSING_SENTENCE = "判法单元指纹要记编这个测试二进制的工具链"   # crash_point_replay_progress.rs 里环境没有 CARGO 时的 panic 句
ENGINE_START_REFUSED_SENTENCE = "起不了给定状态表的引擎"   # e171_three_oracle_collusion.rs 的 given_states_engine_of_this_experiment 起引擎失败的 panic 句（原因行跟在后面）
NOT_RUN_MARKER = "NOT_RUN"
STAGE_DEVICE_OUTPUT = "device_output"
STAGE_CARD_LEASE = "card_lease"   # 按需领卡没成：这一行的装置没起（文件头「按需领卡」）
THREAD_REGISTRY_ENTRY = ".claude/heavy-thread-default"   # 验证包经 include_str! 编进来的登记文件（crates/singlefs-verification/src/heavy_threads.rs）
# 装置起给定状态表的引擎要先起多机配置判法（crates/singlefs-verification/src/gpu_card_configuration.rs 的 emitted_by_the_configuration_check：
# bash <编这份二进制的树>/crates/singlefs-verification/../../scripts/crash/crash-point-replay-shard-configuration-check.sh），判法 source 的 peer-host-lib.sh 与它自证那一支 source 的 heavy-threads.sh（行键在快照上算，闭包静态收，缺它每行拒算），
# 与 .claude/scripts/preflight.sh 转到的规范副本里的 preflight.sh、preflight.py（它按路径载入同目录的 proc.py）；线道的树里没有这几份，
# 装置的引擎起不来（第 3 次重跑探路片：判法退 127，每行 streams_not_exported=2）。
CONFIGURATION_CHECK_ENTRIES = (
    "scripts/crash/crash-point-replay-shard-configuration-check.sh",
    "scripts/hosts/peer-host-lib.sh",
    "scripts/hosts/heavy-threads.sh",
    ".claude/scripts/preflight.sh",
    ".claude/singlefs-ai-sop/scripts/preflight.sh",
    ".claude/singlefs-ai-sop/scripts/preflight.py",
    ".claude/singlefs-ai-sop/scripts/proc.py",
)
SYNCED_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "crates", THREAD_REGISTRY_ENTRY, *CONFIGURATION_CHECK_ENTRIES)
BREAK_VARIABLE = "E171_ROW_DRIVER_BREAK"
# 只有自证在进程里调 main 时把它置真（test_row_shards.py 的 Run.main、test_device_card_lease.py 用 mock.patch.object）；命令行起的本脚本永远是假
BREAK_ACCEPTED_IN_PROCESS = False
MUTATIONS_TABLE = "crates/mutations.tsv"
ROW_TIME_LIMIT_SECONDS = 1800
PROGRESS_INTERVAL_SECONDS = 300   # 每过这么久打一行 row_progress（文件头「进度」）：任务服务看日志涨不涨判卡住
ROW_TIMEOUT_EXIT_CODE = 253  # 超过 ROW_TIME_LIMIT_SECONDS 由本驱动杀掉整组并记这个码：登记第一节读法 5 的「超时」一类，单列、不重跑（比对器按这个码认）
STATUS_OK = "ok"
STATUS_COMPILE_FAILED = "compile_failed"
COMPILE_LOG_TAIL_LINES = 60
O3_STATUS_RAN = "ran"
O3_STATUS_SKIPPED_BY_REQUEST = "skipped_by_request"
O3_STATUS_NO_EXPORTED_STREAM = "no_exported_stream"
O3_STATUS_EXECUTOR_FAILED = "executor_failed"
O3_STATUS_NOT_REACHED = "not_reached"
O3_CONTROL = "O3"
O3_CONTROL_ROW_NAME = "P1-F"
IMAGE_JUDGE_SCRIPT = os.path.join(HERE, "o3_bad_image_judge.py")
SUBSTITUTION_LOCK = threading.Lock()
LANE_BUILD_RECORD_FILE = "build-record.tsv"
LANE_COMPILE_ARTIFACTS_FILE = "compile-artifacts.jsonl"
SNAPSHOT_DIGEST_FILE = "snapshot-digest"
STATUS_MEASUREMENT_PIPE_DEFECT = "measurement_pipe_defect"
REBUILD_CHECK_FIELD = "packages_checked_rebuilt"  # row-result.tsv 里的键：编完核过重编了的包；编过的行没有这个键，就是没核过（旧驱动跑的）
EXIT_WORK_ROOT_REFUSED = 2
EXIT_MEASUREMENT_PIPE_DEFECT = 5
EXIT_FINISH_RED = 1
EXIT_NOT_UNDER_A_TEMPORARY_SERVICE = 2
# 判定库（文件头「判定库」）：种类、本文件的键名、装置入口、判一行的函数、拒算的两种、这一趟的状态目录（工作根下新建，跑完删）
LIBRARY_KIND = "e171_three_oracle_collusion"
SCRIPT_KEY = "scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py"
DEVICE_ENTRY = f"crates/{DEVICE_PACKAGE}/src/bin/{DEVICE_BINARY_NAME}.rs::main"
ROW_JUDGE = f"{SCRIPT_KEY}::run_row_on_the_lane"
IMAGE_JUDGE = "scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py::main"
NEEDS_SUBSTITUTION = "needs_substitution"
MUTATION_TOUCHES_THE_MODULE_SET = "mutation_touches_the_module_set"
MODULE_SET_TOKENS = ("mod ", "#[path", "include_str!", "include_bytes!", "include!")
ITEM_STATE_PREFIX = "item-verdicts-"
SELFTEST_ROUND_MODE = "selftest-round"
SELFTEST_ROWS = (1, 2)
SELFTEST_SOURCE_FILE = "crates/a/src/lib.rs"


class MeasurementPipeDefect(Exception):
    """测量管子自己的缺陷：这一行编出来的装置可能带着同一条线道上别的行的变异。`stage` 是哪一道核对没过，`details` 是没过的文件或包。"""

    def __init__(self, stage: str, details: list[str]) -> None:
        super().__init__(f"{stage}:{','.join(details)}")
        self.stage = stage
        self.details = details


@dataclasses.dataclass(frozen=True)
class LaneBuildRecord:
    """一条线道的编译记录：上一次 cargo 编完的时刻（`time.time_ns`，墙上时间，与文件修改时间同一个钟），与自那以后这条线道上改过、还没有一次
    编过并核过「重编了」的文件（相对树根的路径）。编不过或没核过的那一次之后，文件留在记录里。"""

    last_compile_finished_ns: int
    files_to_rebuild: tuple[str, ...]


def now_ns() -> int:
    return time.monotonic_ns()


def read_lane_build_record(lane_directory: str) -> LaneBuildRecord | None:
    path = os.path.join(lane_directory, LANE_BUILD_RECORD_FILE)
    if not os.path.exists(path):
        return None
    finished_ns, files = 0, []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            key, value = line.rstrip("\n").split("\t", 1)
            if key == "last_compile_finished_ns":
                finished_ns = int(value)
            elif key == "file_to_rebuild":
                files.append(value)
            else:
                raise ValueError(f"{path} 里有认不得的行：{line!r}")
    return LaneBuildRecord(finished_ns, tuple(files))


def write_lane_build_record(lane_directory: str, record: LaneBuildRecord) -> None:
    """先写临时文件再换上：驱动在写的时候被停，留下的仍是上一份完整的记录。"""
    os.makedirs(lane_directory, exist_ok=True)
    path = os.path.join(lane_directory, LANE_BUILD_RECORD_FILE)
    with open(path + ".partial", "w", encoding="utf-8") as handle:
        handle.write(f"last_compile_finished_ns\t{record.last_compile_finished_ns}\n")
        for file in record.files_to_rebuild:
            handle.write(f"file_to_rebuild\t{file}\n")
    os.replace(path + ".partial", path)


def mark_files_newer_than_every_build(lane_tree: str, relative_paths: tuple[str, ...]) -> None:
    """把修改时间置成现在（墙上时间、纳秒）：cargo 只在源文件的修改时间晚于上一次编那个包的时刻时重编它（cargo 1.98 实测，登记修订 5），
    而还原用的 `copy2` 与同步用的 `rsync -a` 都把修改时间带回快照那一刻——不置，上一行改过的包这一行被当成新的、留着上一行的变异。"""
    for relative_path in relative_paths:
        moment_ns = time.time_ns()
        os.utime(os.path.join(lane_tree, relative_path), ns=(moment_ns, moment_ns))


def files_that_would_not_be_rebuilt(lane_tree: str, snapshot_root: str, record: LaneBuildRecord) -> list[str]:
    """编之前的核对：记录里每个文件在线道树里的内容与快照逐字节相同，修改时间晚于上一次编完。交回没过的（`<文件>:<哪一样>`）。"""
    problems = []
    for relative_path in record.files_to_rebuild:
        lane_path = os.path.join(lane_tree, relative_path)
        with open(lane_path, "rb") as lane_handle, open(os.path.join(snapshot_root, relative_path), "rb") as snapshot_handle:
            if lane_handle.read() != snapshot_handle.read():
                problems.append(f"{relative_path}:content_differs_from_the_snapshot")
        if os.stat(lane_path).st_mtime_ns <= record.last_compile_finished_ns:
            problems.append(f"{relative_path}:not_newer_than_the_last_compile")
    return problems


def package_of_file(relative_path: str) -> str:
    """`crates/<包目录>/…` 的包目录（坏实现表只改 `crates/singlefs-core/src/` 与 `crates/singlefs-replay-prefix/src/lib.rs`）。"""
    parts = relative_path.split("/")
    if len(parts) < 3 or parts[0] != "crates":
        raise ValueError(f"不在 crates/<包>/ 下的文件：{relative_path}")
    return "/".join(parts[:2])


def library_freshness_by_package(artifacts_path: str, lane_tree: str) -> dict[str, bool]:
    """读 `cargo build --message-format=json-render-diagnostics` 的标准输出：每个包的库目标那一条 `compiler-artifact` 的 `fresh`
    （true = 这次没重编、用的是编译目录里已有的产物）。键是包目录相对树根的路径。"""
    freshness: dict[str, bool] = {}
    with open(artifacts_path, encoding="utf-8") as handle:
        for line in handle:
            if not line.startswith("{"):
                continue
            message = json.loads(line)
            if message.get("reason") != "compiler-artifact" or "lib" not in message.get("target", {}).get("kind", []):
                continue
            package = os.path.relpath(os.path.dirname(message["manifest_path"]), lane_tree)
            freshness[package] = bool(message["fresh"])
    return freshness


def packages_left_stale(freshness: dict[str, bool], relative_paths: tuple[str, ...]) -> list[str]:
    """编完的核对：这些文件所在的每个包，这一次 cargo 都重编了它的库（`fresh=false`）。交回没过的（`<包>:<哪一样>`）。"""
    problems = []
    for package in sorted({package_of_file(relative_path) for relative_path in relative_paths}):
        if package not in freshness:
            problems.append(f"{package}:no_library_artifact")
        elif freshness[package]:
            problems.append(f"{package}:fresh_not_rebuilt")
    return problems


def work_root_refusals(work_root: str, lanes: int, snapshot_digest_now: str) -> list[str]:
    """开跑前对工作根的两样核对：编过的线道（有编译目录）都有编译记录（没有就是旧驱动建的，还原过的文件没置过时间）；
    工作根上一次的快照摘要与这一次的相同（快照变了，变了的文件带着快照的旧修改时间进线道树，cargo 不重编）。交回拒跑的理由。"""
    refusals = []
    built_lanes = [lane for lane in range(lanes) if os.path.isdir(os.path.join(work_root, f"lane{lane}", "target"))]
    for lane in built_lanes:
        if read_lane_build_record(os.path.join(work_root, f"lane{lane}")) is None:
            refusals.append(f"lane{lane}:built_without_a_build_record")
    if built_lanes:
        digest_path = os.path.join(work_root, SNAPSHOT_DIGEST_FILE)
        if not os.path.exists(digest_path):
            refusals.append("snapshot_digest_missing")
        else:
            with open(digest_path, encoding="utf-8") as handle:
                previous_digest = handle.read().strip()
            if previous_digest != snapshot_digest_now:
                refusals.append(f"snapshot_changed:{previous_digest}->{snapshot_digest_now}")
    return refusals


PATH_DEPENDENCY_PATTERN = re.compile(r'^\s*[\w.-]+\s*=\s*\{[^}]*\bpath\s*=\s*"([^"]+)"', re.MULTILINE)


def path_dependency_entries(source_root: str) -> list[str]:
    """`crates/*/Cargo.toml` 里 `path = "…"` 指到的、落在仓根下 crates/ 之外的第一层目录（验证包依赖仓根的 prune-server/：不带它线道的树编不过）。
    现读源树，不手抄名单；这份驱动在第二台上单独拷在套件目录里，不引 scripts/hosts 的共用库。"""
    crates_directory = os.path.join(source_root, "crates")
    entries: set[str] = set()
    if not os.path.isdir(crates_directory):
        return []
    for crate in sorted(os.listdir(crates_directory)):
        manifest = os.path.join(crates_directory, crate, "Cargo.toml")
        if not os.path.isfile(manifest):
            continue
        with open(manifest, encoding="utf-8") as handle:
            for target in PATH_DEPENDENCY_PATTERN.findall(handle.read()):
                relative = os.path.relpath(os.path.normpath(os.path.join(crates_directory, crate, target)), source_root)
                top = relative.split(os.sep)[0]
                if top not in ("..", ".", "crates") and os.path.isdir(os.path.join(source_root, top)):
                    entries.add(top)
    return sorted(entries)


def sync_tree(source_root: str, destination_root: str) -> None:
    """`rsync -a --delete`：源与目的逐项同步，不动 target；没变的文件保留修改时间，所以编译目录里没变的包不重编。
    SYNCED_ENTRIES 之外再同步 crates 经 path 依赖指到的仓根目录（path_dependency_entries）。"""
    os.makedirs(destination_root, exist_ok=True)
    for entry in (*SYNCED_ENTRIES, *path_dependency_entries(source_root)):
        if entry == THREAD_REGISTRY_ENTRY and os.environ.get(BREAK_VARIABLE) == "registry-not-synced":
            continue
        if entry in CONFIGURATION_CHECK_ENTRIES and os.environ.get(BREAK_VARIABLE) == "configuration-check-not-synced":
            continue
        if entry not in SYNCED_ENTRIES and os.environ.get(BREAK_VARIABLE) == "path-dependencies-not-synced":
            continue
        source = os.path.join(source_root, entry)
        if not os.path.exists(source):
            continue
        destination = os.path.join(destination_root, entry)
        if os.path.isdir(source):
            subprocess.run(["rsync", "-a", "--delete", "--exclude", "target", source + "/", destination + "/"], check=True)
        else:
            os.makedirs(os.path.dirname(destination), exist_ok=True)
            subprocess.run(["rsync", "-a", source, destination], check=True)


def snapshot_digest(root: str) -> tuple[str, int]:
    """快照里每个 `.rs` 与 `Cargo.*` 文件的 (相对路径, sha256) 排好序再算一个总摘要，连同文件数记进产物（行跑的是同一棵树的证据）。"""
    entries = []
    for directory, _, files in os.walk(root):
        for file_name in files:
            if file_name.endswith(".rs") or file_name.startswith("Cargo."):
                path = os.path.join(directory, file_name)
                with open(path, "rb") as handle:
                    entries.append((os.path.relpath(path, root), hashlib.sha256(handle.read()).hexdigest()))
    entries.sort()
    return hashlib.sha256("\n".join(f"{path} {digest}" for path, digest in entries).encode("utf-8")).hexdigest(), len(entries)


def write_result(directory: str, fields: dict[str, object]) -> None:
    with open(os.path.join(directory, "row-result.tsv"), "w", encoding="utf-8") as handle:
        for key, value in fields.items():
            handle.write(f"{key}\t{value}\n")


def compile_command() -> list[str]:
    """编装置的命令。装置的 `row` 模式经给定状态引擎判 O2：配了 GPU 的环境里，不带 `gpu` 特性编的二进制判段报本次未跑、不判（引擎的写法），
    所以带 `--features gpu`，与 `replay.sh` 里 `cargo run` 起的基线、已知坏镜像各趟同一条判路。"""
    features = [] if os.environ.get(BREAK_VARIABLE) == "compile-without-gpu-feature" else ["--features", ",".join(DEVICE_FEATURES)]
    return ["nice", "-n", "19", "cargo", "build", "--release", "--offline", "--message-format=json-render-diagnostics",
            "-p", DEVICE_PACKAGE, *features, "--bin", DEVICE_BINARY_NAME]


def run_compile(lane_tree: str, lane_target: str, repo_root: str, threads: int, log_path: str, artifacts_path: str) -> tuple[int, int]:
    """`cargo build --release --offline --message-format=json-render-diagnostics -p singlefs-implementation-experiments --features gpu --bin e171_three_oracle_collusion`
    （只编不跑，不是重型测试）：诊断照常写进 `log_path`，每个编译单元的 JSON 消息（带 `fresh`）写进 `artifacts_path`。交回 (退出码, 耗时 ns)。"""
    environment = frozen_o3.thread_environment(threads, dict(os.environ, CARGO_TARGET_DIR=lane_target))
    command = compile_command()
    started = now_ns()
    with open(log_path, "w", encoding="utf-8") as log, open(artifacts_path, "w", encoding="utf-8") as artifacts:
        completed = subprocess.run(command, cwd=lane_tree, env=environment, stdout=artifacts, stderr=log)
    return completed.returncode, now_ns() - started


def run_with_time_limit(command: list[str], limit_seconds: int, **popen_arguments) -> int:
    """命令放进自己的进程组跑；超过 limit_seconds 对整组发 KILL，退出码记 ROW_TIMEOUT_EXIT_CODE。出任何异常也先杀整组再抛，不留孤儿。"""
    process = subprocess.Popen(command, start_new_session=True, **popen_arguments)
    broken = os.environ.get(BREAK_VARIABLE)
    try:
        return process.wait(timeout=None if broken == "timeout-not-enforced" else limit_seconds)
    except subprocess.TimeoutExpired:
        if broken == "timeout-kills-only-the-child":
            process.kill()
        else:
            os.killpg(process.pid, signal.SIGKILL)
        process.wait()
        return ROW_TIMEOUT_EXIT_CODE
    except BaseException:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()
        raise


def device_command(binary: str, dump_directory: str, extra: list[str], stage: str | None = None) -> list[str]:
    """装置直接起（线程数变量由 run_device 设进环境）。给了 `stage`（device_card_lease 的 RECORD_STAGE、JUDGE_STAGE）就在末尾点这一段。"""
    return [binary, "row", "--output-directory", dump_directory] + extra + (device_card_lease.stage_arguments(stage) if stage else [])


def device_environment(threads: int, cards: str | None = None, stage: str | None = None) -> dict[str, str]:
    """装置的环境：线程数变量都设成这一条线道的 `threads`；给定状态引擎要环境里有 `CARGO`（记编这个二进制的工具链，不是经 cargo 起的进程里没有它），
    没有就取 `PATH` 上的 cargo。给了 `cards`（这一步按需领到的卡串）就交给装置的 `GPU_CARDS_BY_PRIORITY`；录入段的环境里没有 `GPU_CARDS_BY_PRIORITY`
    （device_card_lease.environment_of_the_stage）。"""
    environment = device_card_lease.environment_of_the_stage(frozen_o3.thread_environment(threads), stage, cards, broken)
    if os.environ.get(BREAK_VARIABLE) != "device-without-cargo-variable" and not environment.get("CARGO"):
        cargo = shutil.which("cargo")
        if cargo:
            environment["CARGO"] = cargo
    return environment


def device_output_defects(row_directory: str) -> list[str]:
    """装置这一趟的输出（`device.out`、`device.err`）里不该有的句子：`NOT_RUN`（判段没跑）与环境里没有 CARGO 的 panic 句（判段没判成）。
    有就是这一行没判成，不是实现的结局：交回 `<文件>:<哪一样>`。"""
    if os.environ.get(BREAK_VARIABLE) == "device-output-defect-ignored":
        return []
    defects = []
    for name in ("device.out", "device.err"):
        path = os.path.join(row_directory, name)
        if not os.path.exists(path):  # run_device 总会建这两个文件；换掉 run_device 的单测桩不建
            continue
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
        if NOT_RUN_MARKER in text:
            defects.append(f"{name}:not_run")
        if TOOLCHAIN_MISSING_SENTENCE in text:
            defects.append(f"{name}:toolchain_missing")
        if ENGINE_START_REFUSED_SENTENCE in text and os.environ.get(BREAK_VARIABLE) != "engine-start-refused-ignored":
            defects.append(f"{name}:engine_start_refused")
    return defects


def run_device(repo_root: str, binary: str, row_directory: str, threads: int, extra: list[str], cards: str | None = None, stage: str | None = None,
               time_limit_seconds: int = ROW_TIME_LIMIT_SECONDS) -> tuple[int, int]:
    """装置的 `row` 模式（限时缺省 1800 秒，登记第一节读法 5 的超时：超时退出码记 253，比对器记进超时一类并单列）。只跑一次。交回 (退出码, 总耗时 ns)。
    线程数变量都设成这一条线道的 `threads`（frozen_o3.thread_environment）；`cards`、`stage` 见 device_environment。不给 `stage` 是一段起；
    录入段新写 `device.out`（空着）与 `device.err`，标准输出也转进 `device.err`；判段往两份文件后面接（文件头「按需领卡」）。"""
    dump_directory = os.path.join(row_directory, "dump")
    command = device_command(binary, dump_directory, extra, stage)
    environment = device_environment(threads, cards, stage)
    mode = "a" if stage == device_card_lease.JUDGE_STAGE else "w"
    started = now_ns()
    with open(os.path.join(row_directory, "device.out"), mode, encoding="utf-8") as out, \
            open(os.path.join(row_directory, "device.err"), mode, encoding="utf-8") as err:
        standard_output = err if stage == device_card_lease.RECORD_STAGE else out
        returncode = run_with_time_limit(command, time_limit_seconds, cwd=repo_root, env=environment, stdout=standard_output, stderr=err)
        if returncode != 0 and os.environ.get(BREAK_VARIABLE) == "device-run-twice":
            returncode = run_with_time_limit(command, time_limit_seconds, cwd=repo_root, env=environment, stdout=standard_output, stderr=err)
    return returncode, now_ns() - started


CARDS_SHARED_BY_THE_BREAK: list[str] = []   # 只给弄坏开关 lanes-share-one-card 用：各线道领到的卡依次记下，判段都拿最先领到的那一张
NANOSECONDS_PER_SECOND = 1_000_000_000


def run_device_on_a_leased_card(arguments: argparse.Namespace, binary: str, row_directory: str, extra: list[str],
                                cards_taken_earlier: str | None = None) -> tuple[int, int]:
    """装置那一步（文件头「按需领卡」）：给了 `--lease-card-per-device-run` 就分录入、判两段起（device_card_lease.record_then_judge_on_a_leased_card）：
    录入段不持卡，录入段退 0 才领一张卡、交给判段，判段跑完照卡串还掉；两段合用 ROW_TIME_LIMIT_SECONDS，判段只给录入段剩下的。没给照 run_device 一段起、不领卡。
    交回 (这一步的退出码, 两段合计耗时 ns)。领卡没成抛 MeasurementPipeDefect（`card_lease`，判段没起）。
    `cards_taken_earlier` 只在弄坏开关 card-held-through-the-compile 下给（一行开头就领了的那张）。"""
    if not getattr(arguments, "lease_card_per_device_run", False):
        return run_device(arguments.repo_root, binary, row_directory, arguments.threads, extra)
    client = device_card_lease.client_of(arguments.repo_root)
    step = f"装置 {DEVICE_BINARY_NAME} {os.path.basename(row_directory)}"
    spent = {"ns": 0}

    def run_stage(stage: str, cards: str | None) -> int:
        if stage == device_card_lease.JUDGE_STAGE and cards is not None and broken("lanes-share-one-card"):
            CARDS_SHARED_BY_THE_BREAK.append(cards)
            cards = CARDS_SHARED_BY_THE_BREAK[0]
        remaining_seconds = max(1, ROW_TIME_LIMIT_SECONDS - spent["ns"] // NANOSECONDS_PER_SECOND)
        exit_code, stage_ns = run_device(arguments.repo_root, binary, row_directory, arguments.threads, extra, cards=cards, stage=stage,
                                         time_limit_seconds=remaining_seconds)
        spent["ns"] += stage_ns
        return exit_code

    def take() -> str:
        return cards_taken_earlier if cards_taken_earlier is not None else device_card_lease.take_card(client, arguments.repo_root, step)

    def give_back(cards: str) -> None:
        if not broken("card-kept-after-the-device"):
            device_card_lease.return_card(client, arguments.repo_root, cards, step, report_failure=not broken("card-return-failure-unreported"))

    try:
        exit_code = device_card_lease.record_then_judge_on_a_leased_card(run_stage, take, give_back, broken)
    except device_card_lease.CardLeaseRefused as refused:
        if not broken("card-lease-refusal-runs-the-device"):
            raise MeasurementPipeDefect(STAGE_CARD_LEASE, [f"take_exit={refused.exit_code}"]) from refused
        exit_code = run_stage(device_card_lease.JUDGE_STAGE, None)
    return exit_code, spent["ns"]


def exported_streams(dump_directory: str) -> list[str]:
    """装置导出了判定表的流（`oracle-verdicts.tsv` 在）；实现自崩没导出的流没有文件可读，不交给执行器。"""
    return [name for name in STREAM_NAMES if os.path.exists(os.path.join(dump_directory, name, "oracle-verdicts.tsv"))]


def run_image_control(arguments: argparse.Namespace, row_directory: str) -> tuple[int, int]:
    """O3 阳性对照（登记 5.6）：这一趟流一导出上，全持久态 F 叠 P1 补丁交冻结的执行器要判「不在」；补丁取 `--o3-control-bad-states` 那份 E170 导出。
    交回 (退出码, 耗时 ns)。"""
    environment = frozen_o3.thread_environment(arguments.threads, dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
    command = ["nice", "-n", "19", sys.executable,
               IMAGE_JUDGE_SCRIPT, "control", "--repo-root", arguments.repo_root, "--dumps", os.path.join(row_directory, "dump"),
               "--e170-dumps", arguments.o3_control_bad_states, "--row-name", O3_CONTROL_ROW_NAME, "--output", os.path.join(row_directory, "o3-control.out")]
    started = now_ns()
    with open(os.path.join(row_directory, "o3-control.log"), "w", encoding="utf-8") as log:
        completed = subprocess.run(command, cwd=arguments.repo_root, env=environment, stdout=log, stderr=subprocess.STDOUT)
    return completed.returncode, now_ns() - started


def run_o3_step(arguments: argparse.Namespace, row_directory: str) -> dict[str, object]:
    """装置跑完之后的 O3 一步。交回要并进 `row-result.tsv` 的字段：状态、退出码、墙钟（驱动进程里 `monotonic_ns` 计）、导出了哪几条流。"""
    if arguments.no_executor:
        return {"o3_status": O3_STATUS_SKIPPED_BY_REQUEST, "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"}
    dump_directory = os.path.join(row_directory, "dump")
    streams = exported_streams(dump_directory)
    if not streams:
        return {"o3_status": O3_STATUS_NO_EXPORTED_STREAM, "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"}
    output = os.path.join(row_directory, "o3.out")
    exit_code, wall_ns = frozen_o3.run_executor(arguments.repo_root, dump_directory, streams, arguments.threads, output, os.path.join(row_directory, "o3.log"))
    if exit_code != 0 and os.environ.get(BREAK_VARIABLE) == "executor-run-twice":
        exit_code, wall_ns = frozen_o3.run_executor(arguments.repo_root, dump_directory, streams, arguments.threads, output, os.path.join(row_directory, "o3.log"))
    complete = os.path.exists(output) and frozen_o3.read_executor_output(output).complete
    fields: dict[str, object] = {"o3_status": O3_STATUS_RAN if exit_code == 0 and complete else O3_STATUS_EXECUTOR_FAILED, "o3_exit": exit_code,
                                 "o3_ns": wall_ns, "o3_streams": ",".join(streams)}
    if arguments.positive_control == O3_CONTROL:
        control_exit, control_ns = run_image_control(arguments, row_directory)
        fields.update({"o3_control_exit": control_exit, "o3_control_ns": control_ns})
    return fields


@dataclasses.dataclass(frozen=True)
class RowChoice:
    """一行这一次要跑的变异：`mutation` 是 None 时补不上。`substituted_from` 是表里那一行的原名（没补位是 `-`），`substitutions_used` 是这一行
    已经换过几次（编之前的失败与编不过合起来至多 `MAXIMUM_SUBSTITUTIONS_PER_ROW` 次），`failed_attempts` 是试过而没用上的每个名字与它的失败（按先后）。"""

    mutation: MutationRow | None
    status: str
    substituted_from: str
    line_matches_the_registration: bool
    substitutions_used: int
    failed_attempts: tuple[tuple[str, str], ...]


def prepare_row(selected: SelectedRow | None, table: list[MutationRow], rows_in_the_table: set[str], used_substitutes: set[str],
                snapshot_root: str, empty_of: MutationRow | None, start_name: str | None = None, substituted_from: str = "-",
                substitutions_used: int = 0, failed_attempts: tuple[tuple[str, str], ...] = ()) -> RowChoice:
    """把表里的一行变成要跑的变异：照名字取、核锚点；编之前的失败（找不到、名字不唯一、锚点不是恰好一次）按补位规则换行。
    编不过之后接着补位时由 `substitute_after_a_compile_failure` 带着已经换过的次数与试过的名字再进来（`start_name` 是换上的那一行）。"""
    if empty_of is not None:
        return RowChoice(empty_mutation(empty_of), STATUS_LOCATED, "-", True, 0, ())
    assert selected is not None
    current_name = start_name or selected.name
    attempts = list(failed_attempts)
    while True:
        located: Located = locate_by_name(SelectedRow(selected.number, selected.slice_number, selected.file, selected.group_index,
                                                      current_name, selected.line_digest_prefix), table)
        status = located.status
        if status == STATUS_LOCATED and located.row is not None:
            with open(os.path.join(snapshot_root, located.row.file), encoding="utf-8") as handle:
                hits = anchor_hit_count(handle.read(), located.row)
            if hits == 1:
                matches = located.line_matches_the_registration if substituted_from == "-" else False
                return RowChoice(located.row, STATUS_LOCATED, substituted_from, matches, substitutions_used, tuple(attempts))
            status = STATUS_ANCHOR_NOT_UNIQUE
        attempts.append((current_name, status))
        if substitutions_used == MAXIMUM_SUBSTITUTIONS_PER_ROW:
            return RowChoice(None, status, substituted_from, False, substitutions_used, tuple(attempts))
        substitute = substitute_for(group_names_of_file(table, selected.file), current_name, rows_in_the_table, used_substitutes)
        if substitute is None:
            return RowChoice(None, status, substituted_from, False, substitutions_used, tuple(attempts))
        used_substitutes.add(substitute)
        if substituted_from == "-":
            substituted_from = selected.name
        substitutions_used += 1
        current_name = substitute


def substitute_after_a_compile_failure(selected: SelectedRow | None, table: list[MutationRow], rows_in_the_table: set[str],
                                       used_substitutes: set[str], snapshot_root: str, failed: RowChoice) -> RowChoice:
    """编不过之后的补位（登记第一节读法 5「替换之后编不过（第八类，按 5.2 的补位规则换行）」）：换同一文件组里编不过那一行的下标 j + 1、
    两张表里都没有、这一次跑里别的补位也没用过的那一行，换上的行编之前照样核锚点（不过再换）；与编之前的补位合起来一行至多换三次，补不上交回 `mutation=None`、
    状态 `compile_failed`。空变异（`selected` 是 None）不补位。"""
    assert failed.mutation is not None
    attempts = failed.failed_attempts + ((failed.mutation.name, STATUS_COMPILE_FAILED),)
    if selected is None or failed.substitutions_used == MAXIMUM_SUBSTITUTIONS_PER_ROW:
        return RowChoice(None, STATUS_COMPILE_FAILED, failed.substituted_from, False, failed.substitutions_used, attempts)
    substitute = substitute_for(group_names_of_file(table, selected.file), failed.mutation.name, rows_in_the_table, used_substitutes)
    if substitute is None:
        return RowChoice(None, STATUS_COMPILE_FAILED, failed.substituted_from, False, failed.substitutions_used, attempts)
    used_substitutes.add(substitute)
    chosen = prepare_row(selected, table, rows_in_the_table, used_substitutes, snapshot_root, None, start_name=substitute,
                         substituted_from=selected.name if failed.substituted_from == "-" else failed.substituted_from,
                         substitutions_used=failed.substitutions_used + 1, failed_attempts=attempts)
    if chosen.mutation is None:  # 换上的行编之前就不过、又补不上：这一行的结局仍记编不过（最早的失败是编不过），每一次试的都在 failed_attempts 里
        return dataclasses.replace(chosen, status=STATUS_COMPILE_FAILED)
    return chosen


def compile_one_mutation(mutation: MutationRow, lane_tree: str, lane_target: str, snapshot_root: str, lane_directory: str, row_directory: str,
                         arguments: argparse.Namespace) -> tuple[int, int, str]:
    """一次改坏、编、还原（登记修订 5 的编译记录与两道核对在每一次编之前与之后都做）。交回 (cargo 退出码, 编译耗时 ns, 编过时核过重编了的包，
    逗号隔开；编不过是 `-`)。核对不过抛 `MeasurementPipeDefect`。编译日志写进行目录的 `compile.log`（下一次编会覆盖它，编不过的那一份由调用方先挪走）。"""
    record = read_lane_build_record(lane_directory)
    if record is None:
        if os.path.isdir(lane_target):
            raise MeasurementPipeDefect("lane_without_a_build_record", [lane_target])
        record = LaneBuildRecord(0, ())
    mark_files_newer_than_every_build(lane_tree, record.files_to_rebuild)
    problems_before = files_that_would_not_be_rebuilt(lane_tree, snapshot_root, record)
    if problems_before:
        raise MeasurementPipeDefect("before_the_compile", problems_before)
    files_to_rebuild = tuple(sorted(set(record.files_to_rebuild) | {mutation.file}))
    write_lane_build_record(lane_directory, LaneBuildRecord(record.last_compile_finished_ns, files_to_rebuild))
    mutated_path = os.path.join(lane_tree, mutation.file)
    with open(mutated_path, encoding="utf-8") as handle:
        original_text = handle.read()
    with open(mutated_path, "w", encoding="utf-8") as handle:
        handle.write(apply_mutation(original_text, mutation))
    artifacts_path = os.path.join(lane_directory, LANE_COMPILE_ARTIFACTS_FILE)
    exit_code, compile_ns = run_compile(lane_tree, lane_target, arguments.repo_root, arguments.threads, os.path.join(row_directory, "compile.log"),
                                        artifacts_path)
    compile_finished_ns = time.time_ns()
    shutil.copy2(os.path.join(snapshot_root, mutation.file), mutated_path)
    if exit_code != 0:  # 编不过：这一次改过的文件留在记录里，下一次编（这一行的补位或下一行）之前照样置时间、核对
        write_lane_build_record(lane_directory, LaneBuildRecord(compile_finished_ns, files_to_rebuild))
        return exit_code, compile_ns, "-"
    problems_after = packages_left_stale(library_freshness_by_package(artifacts_path, lane_tree), files_to_rebuild)
    if problems_after:
        write_lane_build_record(lane_directory, LaneBuildRecord(compile_finished_ns, files_to_rebuild))
        raise MeasurementPipeDefect("after_the_compile", problems_after)
    write_lane_build_record(lane_directory, LaneBuildRecord(compile_finished_ns, (mutation.file,)))
    return exit_code, compile_ns, ",".join(sorted({package_of_file(file) for file in files_to_rebuild}))


def keep_the_tail_of_the_compile_log(row_directory: str, failed_attempt_number: int) -> None:
    """编不过的那一次的编译日志只留尾部，改名成 `compile-failed-<序>.log`，序与 `row-result.tsv` 里记这一次的 `failed_attempt_<序>` 同号
    （这一行补位之后再编会写新的 `compile.log`）。"""
    with open(os.path.join(row_directory, "compile.log"), encoding="utf-8") as handle:
        tail = handle.readlines()[-COMPILE_LOG_TAIL_LINES:]
    with open(os.path.join(row_directory, f"compile-failed-{failed_attempt_number}.log"), "w", encoding="utf-8") as handle:
        handle.writelines(tail)
    os.remove(os.path.join(row_directory, "compile.log"))


def failed_attempt_fields(failed_attempts: tuple[tuple[str, str], ...]) -> dict[str, object]:
    """`row-result.tsv` 里试过而没用上的名字：`failed_attempts=<个数>`，每一个一行 `failed_attempt_<序>=<失败>:<变异名>`（序从 1 起，按先后）。"""
    fields: dict[str, object] = {"failed_attempts": len(failed_attempts)}
    for position, (name, status) in enumerate(failed_attempts, start=1):
        fields[f"failed_attempt_{position}"] = f"{status}:{name}"
    return fields


def names_in_the_tables(selected_rows: list[SelectedRow], other_table_paths: list[str]) -> set[str]:
    """补位要跳过的名字：这一次跑的表与别的几张表（扩表那一次是 117 行那张）里的每一个变异名。"""
    names = {row.name for row in selected_rows}
    for path in other_table_paths:
        names |= {row.name for row in read_selected_rows(path)}
    return names


def read_result(directory: str) -> dict[str, object]:
    with open(os.path.join(directory, "row-result.tsv"), encoding="utf-8") as handle:
        return dict(line.rstrip("\n").split("\t", 1) for line in handle if "\t" in line)


def broken(name: str) -> bool:
    return os.environ.get(BREAK_VARIABLE) == name


def outcome_is_storable(fields: dict[str, object]) -> bool:
    """一行的结局能不能存进库给下一趟复用（文件头「判定库」）：status ok、没补位、装置不是超时。编不过、补位换了行（补位挑行跨行有关）、
    超时、测量管子的缺陷、驱动出错都不存，照同一把键记红、下一趟现判。"""
    return (fields.get("status") == STATUS_OK and str(fields.get("substituted_from", "-")) == "-"
            and str(fields.get("device_exit")) != str(ROW_TIMEOUT_EXIT_CODE))


# ── 判定库（文件头「判定库」）：打包、请求、计划、复用的行取回、判完的行记库、收尾 ──

def pack_directory(directory: str) -> bytes:
    """整个目录打成 tar.gz：子目录、普通文件与符号链接按相对路径排好序，修改时刻、属主写 0，权限写固定值，gzip 的时刻也置 0，
    同一份内容同一串字节。别的文件种类（管道、套接字）抛 ValueError。"""
    members = []
    for base, directories, names in os.walk(directory):
        directories.sort()
        members.extend(os.path.relpath(os.path.join(base, name), directory) for name in (*directories, *names))
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode="w", format=tarfile.PAX_FORMAT) as archive:
        for relative in sorted(members):
            path = os.path.join(directory, relative)
            information = tarfile.TarInfo(relative)
            information.mtime, information.uid, information.gid, information.uname, information.gname = 0, 0, 0, "", ""
            if os.path.islink(path):
                information.type, information.linkname, information.mode = tarfile.SYMTYPE, os.readlink(path), 0o777
                archive.addfile(information)
            elif os.path.isdir(path):
                information.type, information.mode = tarfile.DIRTYPE, 0o755
                archive.addfile(information)
            elif os.path.isfile(path):
                with open(path, "rb") as handle:
                    content = handle.read()
                information.size, information.mode = len(content), 0o644
                archive.addfile(information, io.BytesIO(content))
            else:
                raise ValueError(f"{path} 不是目录、普通文件或符号链接，打不进库")
    return gzip.compress(buffer.getvalue(), compresslevel=6, mtime=0)


def unpack_directory(data: bytes, directory: str) -> None:
    """pack_directory 打的那一包解进一个新目录（目录已在抛 FileExistsError）。"""
    os.makedirs(directory, exist_ok=False)
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        archive.extractall(directory, filter="data")


def item_verdicts_module():
    """scripts/experiment-apparatus/long_run_item_verdicts.py，按本目录所在的仓找（frozen_o3.OWN_REPOSITORY_ROOT：变异驱动把本目录拷到仓外跑单测时
    仓根经 E171_REPOSITORY_ROOT 交过来）；放在 import 路径最后，不遮本目录的模块。"""
    directory = os.path.join(frozen_o3.OWN_REPOSITORY_ROOT, "scripts", "experiment-apparatus")
    if directory not in sys.path:
        sys.path.append(directory)
    import long_run_item_verdicts
    return long_run_item_verdicts


def item_name(number: int) -> str:
    return f"row-{number}"


def number_of_item(item: str) -> int:
    return int(item.removeprefix("row-"))


def print_to_standard_error(line: str) -> None:
    print(line, file=sys.stderr, flush=True)


@dataclasses.dataclass(frozen=True)
class RowRequest:
    """一行进请求的那几样：序号、名单那一行原文（空变异写 -）、改坏用的变异行原文（开跑前照名字取到、锚点恰好一次的那一行；要补位的行写 -）、
    是不是空变异，与拒算的理由（不拒算是空串；NEEDS_SUBSTITUTION、MUTATION_TOUCHES_THE_MODULE_SET 两种，文件头「判定库」的拒算）。"""

    number: int
    list_line: str
    mutation_line: str
    empty_mutation: bool
    refusal: str


@dataclasses.dataclass(frozen=True)
class RowKeySettings:
    """这一趟各行共用、改得了一行结局的那几样（文件头「判定库」的 line: 与 file:）；o3_control_bad_states 是绝对路径或 None。"""

    positive_control: str
    disable_oracle: str
    no_executor: bool
    o3_control_bad_states: str | None
    judging_route: str
    o3_freeze: str


def selected_row_line(selected: SelectedRow) -> str:
    """名单那一行原文：read_selected_rows 读的六列照原次序接回去。"""
    return "\t".join(str(value) for value in (selected.number, selected.slice_number, selected.file, selected.group_index, selected.name,
                                              selected.line_digest_prefix))


def touches_the_module_set(mutation: MutationRow) -> bool:
    """改坏那一处的原文或替换文里有 MODULE_SET_TOKENS 之一：改坏之后连上或断开的模块，入口闭包（在没改坏的快照上算）认不全。"""
    return any(token in mutation.original_text() + mutation.replacement_text() for token in MODULE_SET_TOKENS)


def row_request_of(number: int, selected: SelectedRow | None, empty_of: MutationRow | None, table: list[MutationRow],
                   snapshot_root: str) -> RowRequest:
    """一行开跑前的请求：照名字在快照的变异表里取、核锚点（同 prepare_row 不补位的那一步）；取不到或锚点不是恰好一次的要补位、拒算。"""
    if empty_of is not None:
        return RowRequest(number, "-", empty_of.raw_line, True, "")
    assert selected is not None
    list_line = selected_row_line(selected)
    located = locate_by_name(selected, table)
    if located.status != STATUS_LOCATED or located.row is None:
        return RowRequest(number, list_line, "-", False, NEEDS_SUBSTITUTION)
    try:
        with open(os.path.join(snapshot_root, located.row.file), encoding="utf-8") as handle:
            hits = anchor_hit_count(handle.read(), located.row)
    except OSError:
        hits = 0
    if hits != 1:
        return RowRequest(number, list_line, "-", False, NEEDS_SUBSTITUTION)
    refusal = MUTATION_TOUCHES_THE_MODULE_SET if touches_the_module_set(located.row) else ""
    return RowRequest(number, list_line, located.row.raw_line, False, refusal)


def judging_route(repository_root: str) -> str:
    """给定状态引擎这一趟的判器路线：多机配置开着 GPU 记 gpu，没开或没有多机配置记 cpu（取法同 e171-row-shard.sh：SINGLEFS_MULTI_HOST_CONFIG，
    没设读仓根的 multi-host.env）。"""
    path = os.environ.get("SINGLEFS_MULTI_HOST_CONFIG") or os.path.join(repository_root, "multi-host.env")
    if not os.path.isfile(path):
        return "cpu"
    with open(path, encoding="utf-8") as handle:
        values = dict(line.strip().split("=", 1) for line in handle if "=" in line and not line.lstrip().startswith("#"))
    return "gpu" if values.get("ENABLE_GPU", "0").strip().strip('"') == "1" else "cpu"


def o3_freeze_digest(repo_root: str) -> str:
    """冻结 O3 的冻结行（frozen_o3.FREEZE_REFERENCE_RELATIVE 那份参照产物里的文件行与种子行，各片开跑前照它核冻结）连起来的 sha256；参照产物不在写 missing。"""
    path = os.path.join(repo_root, frozen_o3.FREEZE_REFERENCE_RELATIVE)
    if not os.path.isfile(path):
        return "missing"
    return hashlib.sha256("\n".join(frozen_o3.frozen_lines_of(path)).encode("utf-8")).hexdigest()


def empty_mutation_line_of(repo_root: str, selected_rows_path: str | None = None) -> str:
    """只跑空变异的那一行（row-0）改坏用的变异行原文：选行表（默认步长 8 那张）的第一行照名字在 <树>/crates/mutations.tsv 里取到的那一行；取不到写 -。"""
    try:
        rows = read_selected_rows(selected_rows_path or os.path.join(HERE, "bad-implementations-step-8.tsv"))
        table = read_mutation_rows(os.path.join(repo_root, MUTATIONS_TABLE))
    except OSError:
        return "-"
    located = locate_by_name(rows[0], table).row if rows else None
    return located.raw_line if located is not None else "-"


def key_settings_of(arguments: argparse.Namespace) -> RowKeySettings:
    return RowKeySettings(arguments.positive_control or "-", arguments.disable_oracle or "-", bool(arguments.no_executor),
                          os.path.abspath(arguments.o3_control_bad_states) if arguments.o3_control_bad_states else None,
                          judging_route(frozen_o3.OWN_REPOSITORY_ROOT), "-" if arguments.no_executor else o3_freeze_digest(arguments.repo_root))


def row_request_lines(request: RowRequest, settings: RowKeySettings) -> list[str]:
    """一行的判定请求（long_run_item_verdicts.py 文件头「请求文件」，本文件头「判定库」）；拒算的行不进请求，交空表。"""
    if request.refusal and not broken("refused-row-requested"):
        return []
    item = item_name(request.number)
    judges = [ROW_JUDGE, *([IMAGE_JUDGE] if settings.positive_control == O3_CONTROL else [])]
    tokens = [item, DEVICE_ENTRY, *(f"feature:{DEVICE_PACKAGE}/{feature}" for feature in DEVICE_FEATURES), "profile:release",
              *(f"judge:{judge}" for judge in judges), f"judge-root:{frozen_o3.OWN_REPOSITORY_ROOT}", f"line:row={request.number}",
              f"line:list-line={json.dumps(request.list_line, ensure_ascii=False)}",
              f"line:mutation-line={json.dumps(request.mutation_line, ensure_ascii=False)}",
              f"line:empty-mutation={'yes' if request.empty_mutation else 'no'}", f"line:positive-control={settings.positive_control}",
              f"line:disable-oracle={settings.disable_oracle}", f"line:no-executor={'yes' if settings.no_executor else 'no'}",
              f"line:judging-route={settings.judging_route}", f"line:o3-freeze={settings.o3_freeze}", "line:compare=timing"]
    lines = ["\t".join(tokens)]
    if settings.o3_control_bad_states:
        lines.append(f"@files\t{item}\t{settings.o3_control_bad_states}")
    return lines


class RowLibrary:
    """这一趟经 long_run_item_verdicts 的计划：各行的决定（进了请求的行：judge、reuse、refused；不进请求的拒算行不在里面）、复用的行取回、
    判完的行记库（线道之间一把锁排开）、收尾。记库出的客户端错存进 failure，由主线程照它的码退。"""

    def __init__(self, verdicts, state_directory: str, decisions: dict[int, str], printer) -> None:
        self.verdicts = verdicts
        self.state_directory = state_directory
        self.decisions = decisions
        self.printer = printer
        self.lock = threading.Lock()
        self.failure: BaseException | None = None

    def reused(self, number: int) -> bool:
        return self.decisions.get(number) == self.verdicts.REUSE

    def take_reused_row(self, number: int, row_directory: str) -> dict[str, object]:
        """复用的行：计划那一步从库里取回、放进状态目录的那一包解进行目录（先删那里留的），交它的 row-result.tsv。"""
        _exit_code, packed = self.verdicts.reused_output(self.state_directory, item_name(number))
        if os.path.isdir(row_directory):
            shutil.rmtree(row_directory)
        if broken("reused-row-not-unpacked"):
            return {"row_number": number}
        unpack_directory(packed, row_directory)
        return read_result(row_directory)

    def record_row(self, number: int, row_directory: str, fields: dict[str, object]) -> bool:
        """一行判完（含测量管子的缺陷与驱动出错）：进了请求、不是复用的行经 record 记（文件头「判定库」）。交 False 是记库出错（failure 记着客户端抛的那一个）。"""
        if number not in self.decisions or self.reused(number):
            return True
        storable = outcome_is_storable(fields) or broken("unstorable-outcome-recorded-green")
        try:
            packed = pack_directory(row_directory) if storable else b""
            with self.lock:
                self.verdicts.record(self.state_directory, item_name(number), 0 if storable else 1, packed, storable, self.printer)
        except SystemExit as failure:   # prune_server_client.PruneServerFailure 是 SystemExit：线道线程里不接，这条线程就悄悄没了
            if broken("lane-library-failure-swallowed"):
                return True
            self.failure = failure
            return False
        return True

    def finish(self) -> int:
        """long_run_item_verdicts 的 finish（重算现判过的行的键、判复用哨兵、逐项报、打总计）的退出码；一行都没进请求时没有计划，交 0。"""
        if not os.path.isfile(os.path.join(self.state_directory, self.verdicts.PLAN_FILE)):
            self.printer("  · 一行都没进请求（都拒算）：没有 long_run_item_verdicts 的计划，不收尾")
            return 0
        code = self.verdicts.finish(self.state_directory, self.printer)
        return 0 if broken("item-finish-ignored") else code


def plan_rows(state_directory: str, tree_root: str, requests: list[RowRequest], settings: RowKeySettings, printer=print_to_standard_error) -> RowLibrary:
    """计划那一步：各行的请求交 long_run_item_verdicts 的 plan（在 tree_root 上算键、查库、抽复用哨兵，状态写进 state_directory）。
    不进请求的拒算行打 E171_ROW_ITEM 一行；一行都没进请求时不起 plan。"""
    verdicts = item_verdicts_module()
    text = ""
    for request in requests:
        lines = row_request_lines(request, settings)
        if not lines:
            printer(f"E171_ROW_ITEM row={request.number} refused={request.refusal}")
        text += "".join(line + "\n" for line in lines)
    decisions: dict[int, str] = {}
    if text:
        verdicts.plan(state_directory, SCRIPT_KEY, LIBRARY_KIND, tree_root, text, "E171 各行的请求", printer)
        for item, entry in verdicts.read_plan(state_directory)["items"].items():
            decisions[number_of_item(item)] = entry["decision"]
    return RowLibrary(verdicts, state_directory, decisions, printer)


def parse_row_range(text: str) -> tuple[int, int]:
    """`<起>-<止>`（闭区间，表里的序号）。"""
    match = re.fullmatch(r"(\d+)-(\d+)", text)
    if match is None or int(match.group(1)) > int(match.group(2)):
        raise ValueError(f"--row-range 写成 <起>-<止>（起 ≤ 止，表里的序号）：{text!r}")
    return int(match.group(1)), int(match.group(2))


def rows_in_the_range(selected_rows: list[SelectedRow], low: int, high: int) -> list[SelectedRow]:
    """表里序号落在闭区间 [low, high] 里的行（表里的次序）。"""
    if os.environ.get(BREAK_VARIABLE) == "row-range-ignored":
        return list(selected_rows)
    return [row for row in selected_rows if low <= row.number <= high]


def files_cut_by_the_range(selected_rows: list[SelectedRow], low: int, high: int) -> list[str]:
    """区间切开的文件组：同一个文件的行有的在区间里、有的在区间外。补位只在文件组内换行，两片切开同一组会各自挑同一个补位。"""
    inside = {row.file for row in selected_rows if low <= row.number <= high}
    outside = {row.file for row in selected_rows if not low <= row.number <= high}
    return sorted(inside & outside)


def shard_ranges(selected_rows: list[SelectedRow], count: int, may_cut_groups: bool = False) -> list[tuple[int, int]]:
    """把表按次序切成至多 `count` 片连续的行区间，边界落在文件组之间（不切开一个文件组），每片的行数尽量接近。交回每片的 (起序号, 止序号)。
    `may_cut_groups`：不管文件组，只按行数均分（各片行数最多差 1）；只给不补位的跑法用（`--keep-compile-failures-as-they-are`，补位才怕切开文件组）。
    文件组 = 表里文件相同的连续行；表里的序号要严格递增、同一个文件的行要连在一起，否则抛 ValueError（区间按序号取，散开的文件组切不对齐）。"""
    if count < 1:
        raise ValueError("片数至少 1")
    if may_cut_groups:
        evenly: list[tuple[int, int]] = []
        start = 0
        for index in range(count):
            end = len(selected_rows) * (index + 1) // count
            if end > start:
                evenly.append((selected_rows[start].number, selected_rows[end - 1].number))
                start = end
        return evenly
    groups: list[list[SelectedRow]] = []
    for row in selected_rows:
        if groups and groups[-1][-1].file == row.file:
            if row.number <= groups[-1][-1].number:
                raise ValueError(f"表里的序号要严格递增：{row.number} 接在 {groups[-1][-1].number} 后面")
            groups[-1].append(row)
        else:
            if any(group[0].file == row.file for group in groups):
                raise ValueError(f"同一个文件的行没有连在一起：{row.file}")
            if groups and row.number <= groups[-1][-1].number:
                raise ValueError(f"表里的序号要严格递增：{row.number} 接在 {groups[-1][-1].number} 后面")
            groups.append([row])
    ranges: list[tuple[int, int]] = []
    closed_rows = 0
    current: list[SelectedRow] = []
    for group in groups:
        shards_open = count - len(ranges)
        if current and shards_open > 1:
            target = -(-(len(selected_rows) - closed_rows) // shards_open)
            with_group = len(current) + len(group)
            if len(current) >= target or (with_group > target and target - len(current) < with_group - target):
                ranges.append((current[0].number, current[-1].number))
                closed_rows += len(current)
                current = []
        current.extend(group)
    if current:
        ranges.append((current[0].number, current[-1].number))
    return ranges


def choose_rows(arguments: argparse.Namespace, selected_rows: list[SelectedRow]) -> list[SelectedRow]:
    """选行：给了 --empty-mutation-only 一行都不取（只跑空变异），给了 --first-slice 取第一刀，给了 --rows 取点名的序号，
    给了 --row-range 取区间里的行，都没给取全部行。"""
    if arguments.empty_mutation_only and os.environ.get(BREAK_VARIABLE) != "empty-mutation-only-runs-the-table":
        return []
    if arguments.first_slice:
        return [row for row in selected_rows if row.slice_number == 1]
    if arguments.rows is not None:
        wanted = {int(token) for token in arguments.rows.split(",") if token}
        return [row for row in selected_rows if row.number in wanted]
    if arguments.row_range is not None:
        return rows_in_the_range(selected_rows, *parse_row_range(arguments.row_range))
    return list(selected_rows)


def run_row_on_the_lane(number: int, selected: SelectedRow | None, empty_of: MutationRow | None, table: list[MutationRow],
                        rows_in_the_table: set[str], used_substitutes: set[str], arguments: argparse.Namespace,
                        lane: int) -> dict[str, object]:
    """一条线道一次只跑一行（同一条线道共用一棵树与一个编译目录）。"""
    row_directory = os.path.join(arguments.output_root, f"row-{number}")
    row_name = selected.name if selected else (empty_mutation(empty_of).name if empty_of else "-")
    if os.path.isdir(row_directory):
        shutil.rmtree(row_directory)  # 输出根里留着的行目录不当复用来源（复用只经判定库，文件头「判定库」）：删掉重做
    os.makedirs(row_directory, exist_ok=False)
    lane_tree = os.path.join(arguments.work_root, f"lane{lane}", "tree")
    lane_target = os.path.join(arguments.work_root, f"lane{lane}", "target")
    snapshot_root = os.path.join(arguments.work_root, "snapshot")
    fields: dict[str, object] = {"row_number": number, "row_name": row_name, "lane": lane}
    started = now_ns()
    compile_kind = "incremental" if os.path.isdir(lane_target) else "cold"
    cards_taken_earlier = None
    if getattr(arguments, "lease_card_per_device_run", False) and os.environ.get(BREAK_VARIABLE) == "card-held-through-the-compile":
        cards_taken_earlier = device_card_lease.take_card(device_card_lease.client_of(arguments.repo_root), arguments.repo_root, f"row-{number}")
    sync_tree(snapshot_root, lane_tree)
    with SUBSTITUTION_LOCK:  # 补位挑行读写同一张「已用过的补位」集合，线道之间不能同时挑
        choice = prepare_row(selected, table, rows_in_the_table, used_substitutes, snapshot_root, empty_of)
    copy_ns, compile_ns_of_every_attempt = now_ns() - started, 0
    while choice.mutation is not None:
        attempt_started = now_ns()
        exit_code, compile_ns, rebuilt_packages = compile_one_mutation(choice.mutation, lane_tree, lane_target, snapshot_root,
                                                                       os.path.join(arguments.work_root, f"lane{lane}"), row_directory, arguments)
        copy_ns += now_ns() - attempt_started - compile_ns
        compile_ns_of_every_attempt += compile_ns
        if exit_code == 0:
            fields[REBUILD_CHECK_FIELD] = rebuilt_packages
            break
        keep_the_tail_of_the_compile_log(row_directory, len(choice.failed_attempts) + 1)
        if getattr(arguments, "keep_compile_failures_as_they_are", False):  # 只给复跑 s4、F4 两份产物用：它们是编不过不补位那一版驱动跑的
            choice = RowChoice(None, STATUS_COMPILE_FAILED, choice.substituted_from, False, choice.substitutions_used,
                               choice.failed_attempts + ((choice.mutation.name, STATUS_COMPILE_FAILED),))
            break
        with SUBSTITUTION_LOCK:
            choice = substitute_after_a_compile_failure(selected, table, rows_in_the_table, used_substitutes, snapshot_root, choice)
    fields.update({"substituted_from": choice.substituted_from, "line_matches_the_registration": str(choice.line_matches_the_registration).lower(),
                   "mutated_row_name": choice.mutation.name if choice.mutation else "-"})
    fields.update(failed_attempt_fields(choice.failed_attempts))
    if choice.mutation is None:
        compiled_at_least_once = any(status == STATUS_COMPILE_FAILED for _, status in choice.failed_attempts)
        fields.update({"status": choice.status, "device_exit": "-", "copy_ns": copy_ns,
                       "compile_ns": compile_ns_of_every_attempt if compiled_at_least_once else "-",
                       "compile_kind": compile_kind if compiled_at_least_once else "-", "device_ns": "-", "o3_ns": "-", "o3_status": O3_STATUS_NOT_REACHED})
        write_result(row_directory, fields)
        return fields
    fields.update({"mutation_line_sha256": hashlib.sha256(choice.mutation.raw_line.encode("utf-8")).hexdigest(), "copy_ns": copy_ns,
                   "compile_ns": compile_ns_of_every_attempt, "compile_kind": compile_kind})
    extra = []
    if arguments.positive_control and arguments.positive_control != O3_CONTROL:
        extra += ["--positive-control", arguments.positive_control]
    if arguments.disable_oracle:
        extra += ["--disable-oracle", arguments.disable_oracle]
    binary = os.path.join(lane_target, "release", DEVICE_BINARY_NAME)
    device_exit, device_ns = run_device_on_a_leased_card(arguments, binary, row_directory, extra, cards_taken_earlier)
    defects = device_output_defects(row_directory)
    if defects:
        raise MeasurementPipeDefect(STAGE_DEVICE_OUTPUT, defects)
    fields.update({"status": STATUS_OK, "device_exit": device_exit, "device_ns": device_ns})
    if device_exit == 0:
        fields.update(run_o3_step(arguments, row_directory))
    else:
        fields.update({"o3_status": O3_STATUS_NOT_REACHED, "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"})
    write_result(row_directory, fields)
    return fields


def lane_worker(lane: int, jobs: "queue.Queue[tuple[int, int, SelectedRow | None, MutationRow | None]]", results: dict[int, dict[str, object]],
                table: list[MutationRow], rows_in_the_table: set[str], used_substitutes: set[str], arguments: argparse.Namespace,
                stop_taking_rows: threading.Event | None = None, running: dict[int, int] | None = None,
                library: RowLibrary | None = None, row_runner=None) -> None:
    """一条线道：从共用队列里领行（先到先得），一次一行，直到队列空。队列里的项是 (在作业表里的位置, 登记 5.9 表的序号, 表里的行, 空变异的行)。
    任一条线道报测量管子的缺陷或记库出错，就设 `stop_taking_rows`：各线道跑完手上那一行之后不再领新行。`running`（给了的话）记这条线道手上那一行的序号，进度行读它。
    给了 `library`，判完的每一行当场经它记库（RowLibrary.record_row）。`row_runner` 换掉判一行的函数（只给自证的 selftest_round），不给照 run_row_on_the_lane。"""
    while stop_taking_rows is None or not stop_taking_rows.is_set():
        try:
            position, number, selected, empty_of = jobs.get_nowait()
        except queue.Empty:
            return
        if running is not None:
            running[lane] = number
        runner = row_runner or run_row_on_the_lane
        try:
            results[position] = runner(number, selected, empty_of, table, rows_in_the_table, used_substitutes, arguments, lane)
        except MeasurementPipeDefect as defect:
            results[position] = {"row_number": number, "lane": lane, "status": STATUS_MEASUREMENT_PIPE_DEFECT, "stage": defect.stage,
                                 "detail": ",".join(defect.details)}
            if stop_taking_rows is not None:
                stop_taking_rows.set()
        except Exception as error:  # 一行的驱动出错不拖垮别的线道；记下来、最后退非 0
            results[position] = {"row_number": number, "status": "driver_error", "error": f"{type(error).__name__}:{error}"}
        finally:
            if running is not None:
                running.pop(lane, None)
        if library is not None and not library.record_row(number, os.path.join(arguments.output_root, f"row-{number}"), results[position]):
            if stop_taking_rows is not None:
                stop_taking_rows.set()
            return


def report_progress(stop: threading.Event, started_ns: int, total: int, results: dict[int, dict[str, object]], running: dict[int, int]) -> None:
    """进度（文件头「进度」）：直到 `stop` 被设，每过 PROGRESS_INTERVAL_SECONDS 往标准错误打一行 row_progress（做完几行、哪几条线道在做哪一行、开跑了几秒）。"""
    while not stop.wait(PROGRESS_INTERVAL_SECONDS):
        if os.environ.get(BREAK_VARIABLE) == "progress-not-reported":
            continue
        in_flight = ",".join(f"row-{number}@lane{lane}" for lane, number in sorted(dict(running).items())) or "none"
        print(f"row_progress done={len(results)} of={total} running={in_flight} elapsed_s={(now_ns() - started_ns) // 1_000_000_000}",
              file=sys.stderr, flush=True)


def run_rows_through_the_library(library: RowLibrary, jobs: list[tuple[int, SelectedRow | None, MutationRow | None]], table: list[MutationRow],
                                 rows_in_the_table: set[str], used_substitutes: set[str], arguments: argparse.Namespace,
                                 row_runner=None) -> dict[int, dict[str, object]]:
    """复用的行从库里取回、解进行目录；别的行交 `arguments.lanes` 条线道跑，判完当场记库（文件头「判定库」）。交 {在作业表里的位置: 那一行的字段}。
    线道里记库出了客户端的错（RowLibrary.failure），各线道停下之后照原样抛出去（照客户端的码退）。"""
    results: dict[int, dict[str, object]] = {}
    work: "queue.Queue[tuple[int, int, SelectedRow | None, MutationRow | None]]" = queue.Queue()
    for position, (number, selected, empty_of) in enumerate(jobs):
        if library.reused(number):
            results[position] = library.take_reused_row(number, os.path.join(arguments.output_root, f"row-{number}"))
        else:
            work.put((position, number, selected, empty_of))
    stop_taking_rows = threading.Event()
    running: dict[int, int] = {}
    threads = [threading.Thread(target=lane_worker, args=(lane, work, results, table, rows_in_the_table, used_substitutes, arguments, stop_taking_rows,
                                                          running, library, row_runner))
               for lane in range(arguments.lanes)]
    lanes_done = threading.Event()
    progress = threading.Thread(target=report_progress, args=(lanes_done, now_ns(), len(jobs), results, running), daemon=True)
    progress.start()
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    lanes_done.set()
    progress.join()
    if library.failure is not None:
        raise library.failure
    return results


def threads_per_lane(threads: int | None, lanes: int, environment=os.environ) -> int:
    """每条线道的线程数：给了照它；没给取登记的重活默认并行度（SINGLEFS_THREAD_CAP 设了取它）÷ 线道数，向下取整，至少 1。"""
    if threads is not None:
        return threads
    # 现用现引：第二台上这份驱动单独拷在套件目录里，那里没有 scripts/hosts，不能在导入时引
    sys.path.insert(0, os.path.join(HERE, "..", "..", "hosts"))
    from heavy_threads import default_heavy_threads
    return max(1, default_heavy_threads(environment) // max(1, lanes))


def main(argument_list: list[str]) -> int:
    # --selftest 与 selftest-round 先判在不在临时 prune-server 底下（不在退 2），在就认弄坏开关（文件头「自证」）
    if argument_list and argument_list[0] in ("--selftest", SELFTEST_ROUND_MODE):
        return selftest_entry(argument_list)
    # 弄坏开关只在自证里认：别的命令行设了 E171_ROW_DRIVER_BREAK 一律退 2，在参数解析之前判；
    # 证红的是 test_synced_entries.py，它在进程里调 sync_tree，不经这个命令行；
    # test_row_shards.py、test_device_card_lease.py 在进程里调本函数，先把 BREAK_ACCEPTED_IN_PROCESS 置真，命令行起的永远是假
    if os.environ.get(BREAK_VARIABLE) and not BREAK_ACCEPTED_IN_PROCESS:
        print(f"  ✗ {BREAK_VARIABLE} 只在自证里认（弄坏开关只给 test_synced_entries.py、test_collusion_pipeline.py、test_row_shards.py、test_device_card_lease.py 证红用），这一趟不是自证", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 -m unittest discover -s scripts/experiment-apparatus/e171_three_oracle_collusion -p 'test_*.py'", file=sys.stderr)
        return 2
    parser = argparse.ArgumentParser(description="E171 逐行驱动")
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--work-root", required=True, help="草稿目录：快照与每条线道的树与编译目录放这里")
    parser.add_argument("--output-root", required=True, help="每行一个 row-<序号>/ 目录；已存在的行目录拒绝覆盖")
    parser.add_argument("--selected-rows", default=os.path.join(HERE, "bad-implementations-step-8.tsv"))
    parser.add_argument("--other-selected-rows", action="append", default=[],
                        help="别的表（同一格式）：它们的名字补位时一样跳过；扩表那一次给步长 8 那张")
    parser.add_argument("--rows", default=None, help="指定范围：登记 5.9 表里的序号，逗号隔开（给空串是不跑表里的行，只跑 --empty-mutation）；不给 --rows、--first-slice 就是表里全部行")
    parser.add_argument("--first-slice", action="store_true", help="指定范围：第一刀，每个文件 i = 0 那一行（24 行）")
    parser.add_argument("--row-range", default=None, help="分片：表里序号落在 <起>-<止>（闭区间）的行；各片写同一个 --output-root，不许切开一个文件组（见文件头「分片」）")
    parser.add_argument("--print-shard-ranges", type=int, default=None, metavar="片数",
                        help="只打 --selected-rows 那张表按文件组对齐的 <片数> 片行区间（一行一片），不跑")
    parser.add_argument("--shards-may-cut-groups", action="store_true",
                        help="与 --print-shard-ranges 一起：只按行数均分、不管文件组（只给带 --keep-compile-failures-as-they-are 的跑法用）")
    parser.add_argument("--empty-mutation-only", action="store_true", help="只跑空变异（行目录 row-0），不跑表里的行；阳性对照的每一趟用它")
    parser.add_argument("--empty-mutation", action="store_true", help="另跑空变异：拿表里第一行、把替换文换成原文本身（行目录 row-0）")
    parser.add_argument("--threads", type=int, default=None, help="每条线道的线程数；不给取登记的重活默认并行度 ÷ --lanes（向下取整，至少 1）")
    parser.add_argument("--lanes", type=int, default=1)
    parser.add_argument("--positive-control", choices=["O1-对拍", "O1-崩溃判官", "O2", O3_CONTROL])
    parser.add_argument("--disable-oracle", choices=["O1", "O2"])
    parser.add_argument("--no-executor", action="store_true", help="不跑 O3 执行器（Q3-核「驱动不跑执行器」那一次）")
    parser.add_argument("--o3-control-bad-states", help="`--positive-control O3` 要的 E170 `bad-states` 导出目录（里面有每条流的 bad-states.tsv 与 bad-patches.bin）")
    parser.add_argument("--snapshot-from", help="快照从哪棵树同步（默认 --repo-root）：几趟要编同一棵树时，先把 Cargo.toml、Cargo.lock、.cargo、crates 拷一份冻住，各趟都指它")
    parser.add_argument("--keep-compile-failures-as-they-are", action="store_true",
                        help="编不过的行不补位、记编不过（只给 replay.sh 复跑 s4、F4 两份产物用：那两份是编不过之后不补位的那一版驱动跑的，登记第一节读法 5 要补位）")
    parser.add_argument("--lease-card-per-device-run", action="store_true",
                        help="按需领卡：每一行的装置分录入、判两段起，录入段不持卡，录入段退 0 才领一张卡起判段、跑完还（文件头「按需领卡」）；环境里没有领卡通道退 78")
    arguments = parser.parse_args(argument_list)
    if arguments.print_shard_ranges is not None:
        table_rows = read_selected_rows(arguments.selected_rows)
        for index, (low, high) in enumerate(shard_ranges(table_rows, arguments.print_shard_ranges, arguments.shards_may_cut_groups), start=1):
            print(f"shard_range index={index} low={low} high={high} rows={sum(low <= row.number <= high for row in table_rows)}")
        return 0
    channel_problem = device_card_lease.channel_problem() if arguments.lease_card_per_device_run else None
    if channel_problem is not None and os.environ.get(BREAK_VARIABLE) != "card-lease-channel-unchecked":
        device_card_lease.report_missing_channel("row_driver.py", channel_problem,
                                                 "bash scripts/experiment-apparatus/e171-row-shard.sh …（或 replay.sh 的 E171 那几行）")
        return device_card_lease.EXIT_NO_CHANNEL
    arguments.threads = threads_per_lane(arguments.threads, arguments.lanes)
    if sum(option is not None and option is not False for option in (arguments.rows, arguments.first_slice, arguments.row_range, arguments.empty_mutation_only)) > 1:
        parser.error("--rows、--first-slice、--row-range、--empty-mutation-only 只给一样")
    if arguments.empty_mutation_only:
        arguments.empty_mutation = True
    if arguments.positive_control == O3_CONTROL and not arguments.o3_control_bad_states:
        parser.error("--positive-control O3 要同时给 --o3-control-bad-states")
    if arguments.positive_control == O3_CONTROL and arguments.no_executor:
        parser.error("--positive-control O3 与 --no-executor 不能一起")
    selected_rows = read_selected_rows(arguments.selected_rows)
    if arguments.row_range is not None:
        try:
            low, high = parse_row_range(arguments.row_range)
        except ValueError as error:
            parser.error(str(error))
        cut = files_cut_by_the_range(selected_rows, low, high)
        if cut and not arguments.keep_compile_failures_as_they_are:
            print(f"row_range_refused range={arguments.row_range} files_cut={','.join(cut)}", file=sys.stderr)
            print("区间切开了文件组（补位只在文件组内换行，两片会挑同一个补位）：用 --print-shard-ranges 打对齐的区间，或加 --keep-compile-failures-as-they-are", file=sys.stderr)
            return EXIT_WORK_ROOT_REFUSED
    chosen = choose_rows(arguments, selected_rows)
    if arguments.row_range is not None:
        scope = f"shard rows={len(chosen)} of={len(selected_rows)} range={arguments.row_range}"
    else:
        scope = f"{'narrowed' if arguments.rows is not None or arguments.first_slice or arguments.empty_mutation_only else 'full_table'} rows={len(chosen)} of={len(selected_rows)}"
    print(f"row_scope scope={scope}", file=sys.stderr)
    started = now_ns()
    if not arguments.no_executor and frozen_o3.report_missing_freeze_reference(arguments.repo_root, "row_driver.py"):
        return frozen_o3.NOT_RUN_EXIT_CODE   # 冻结行的参照产物不在：本次未跑，一行都不跑
    if not arguments.no_executor:
        differences = frozen_o3.freeze_differences(arguments.repo_root)
        print(f"freeze_check before=true differences={len(differences)} detail={','.join(differences) or 'none'}")
        if differences:
            print("冻结的执行器与冻结行对不上（作废条款 Z2），这一趟不跑", file=sys.stderr)
            return 4
    snapshot_root = os.path.join(arguments.work_root, "snapshot")
    sync_tree(arguments.snapshot_from or arguments.repo_root, snapshot_root)
    missing_entries = [entry for entry in CONFIGURATION_CHECK_ENTRIES if not os.path.isfile(os.path.join(snapshot_root, entry))]
    if missing_entries and os.environ.get(BREAK_VARIABLE) != "configuration-check-not-synced":
        print(f"configuration_check_missing entries={','.join(missing_entries)}")
        print("快照里缺多机配置判法或它 source 的文件（装置起不了给定状态表的引擎），一行都不跑：从带这几份的树起，或用 --snapshot-from 指一棵带的树", file=sys.stderr)
        return EXIT_WORK_ROOT_REFUSED
    digest, file_count = snapshot_digest(snapshot_root)
    refusals = work_root_refusals(arguments.work_root, arguments.lanes, digest)
    if refusals:
        print(f"work_root_refused reasons={','.join(refusals)}")
        print("工作根里的编译目录不是这一版驱动在这张快照上建的（登记修订 5），换一个新的 --work-root", file=sys.stderr)
        return EXIT_WORK_ROOT_REFUSED
    with open(os.path.join(arguments.work_root, SNAPSHOT_DIGEST_FILE), "w", encoding="utf-8") as handle:
        handle.write(digest + "\n")
    os.makedirs(arguments.output_root, exist_ok=True)
    table = read_mutation_rows(os.path.join(snapshot_root, MUTATIONS_TABLE))
    rows_in_the_table = names_in_the_tables(selected_rows, arguments.other_selected_rows)
    used_substitutes: set[str] = set()
    jobs: list[tuple[int, SelectedRow | None, MutationRow | None]] = []
    if arguments.empty_mutation:
        first_row = locate_by_name(selected_rows[0], table).row
        if first_row is None:
            print("空变异要的表里第一行找不到", file=sys.stderr)
            return 2
        jobs.append((0, None, first_row))
    jobs.extend((row.number, row, None) for row in chosen)
    print(f"snapshot_digest={digest} files={file_count} snapshot_ns={now_ns() - started} rows={len(jobs)}")
    requests = [row_request_of(number, selected, empty_of, table, snapshot_root) for number, selected, empty_of in jobs]
    state_directory = tempfile.mkdtemp(prefix=ITEM_STATE_PREFIX, dir=arguments.work_root)
    try:
        try:
            library = plan_rows(state_directory, snapshot_root, requests, key_settings_of(arguments))
        except item_verdicts_module().UsageError as error:
            print(f"  ✗ 各行的判定请求写不成：{error}", file=sys.stderr)
            print("     → 怎么办：照 scripts/experiment-apparatus/long_run_item_verdicts.py 文件头「请求文件」看 row_request_lines 拼的那几行", file=sys.stderr)
            return 2
        table_started = now_ns()
        results = run_rows_through_the_library(library, jobs, table, rows_in_the_table, used_substitutes, arguments)
        table_ns = now_ns() - table_started
        for position in sorted(results):
            print("row " + " ".join(f"{key}={str(value).replace(' ', '_')}" for key, value in results[position].items()))
        print(f"table lanes={arguments.lanes} threads_per_lane={arguments.threads} rows={len(jobs)} table_wall_ns={table_ns} "
              f"sum_of_row_ns={sum(int(value) for fields in results.values() for key, value in fields.items() if key in ('copy_ns', 'compile_ns', 'device_ns', 'o3_ns') and str(value).isdigit())}")
        finish_code = library.finish()
    finally:
        shutil.rmtree(state_directory, ignore_errors=True)
    if not arguments.no_executor:
        after = frozen_o3.freeze_differences(arguments.repo_root)
        print(f"freeze_check before=false differences={len(after)} detail={','.join(after) or 'none'}")
    if any(fields.get("status") == STATUS_MEASUREMENT_PIPE_DEFECT for fields in results.values()):
        return EXIT_MEASUREMENT_PIPE_DEFECT
    if finish_code != 0:
        return EXIT_FINISH_RED
    return 1 if len(results) != len(jobs) or any(fields.get("status") == "driver_error" for fields in results.values()) else 0


# ── 只取（文件头「只取」）：这几个函数都不带缺省值、不加模块级语句，不进行键与项键的判法闭包 ──

def rows_from_the_library(requests: list[RowRequest], settings: RowKeySettings, tree_root: str, output_root: str, check_only: bool,
                          judge_missing) -> tuple[dict[int, dict[str, object]], list[tuple[int, str]]]:
    """照键从库里取这几行（只查不判：不起 long_run_item_verdicts 的 plan、不抽复用哨兵）。请求与键的算法同 plan_rows（row_request_lines、item_keys 在 tree_root 上算），
    库里认的也同它：.verdict 记着绿、.out 读得回且 sha256 与记的相同。取到的解进 <output_root>/row-<序号>/（check_only 不写）。
    交 ({序号: 行目录里 row-result.tsv 的字段，check_only 时只有序号}, [(没取到的序号, 为什么)…]，按序号排)。
    judge_missing(序号表) 只在弄坏开关 take-only-judges-missing-rows 下叫（缺行就现判的样子），交 {序号: 字段}。"""
    verdicts = item_verdicts_module()
    verdicts.require_selftest_variables_allowed()
    missing: dict[int, str] = {}
    text = ""
    for request in requests:
        lines = row_request_lines(request, settings)
        if not lines:
            missing[request.number] = f"拒算（{request.refusal}）：这一行不进判定库"
        text += "".join(line + "\n" for line in lines)
    taken: dict[int, dict[str, object]] = {}
    if text:
        parsed = verdicts.parse_requests(text, "E171 各行的请求")
        if os.environ.get(verdicts.SUBSTITUTION_VARIABLE):
            verdicts.substitute_entries(parsed, os.environ[verdicts.SUBSTITUTION_VARIABLE])
        keyed = verdicts.item_keys(tree_root, parsed)
        store = verdicts.library_of(LIBRARY_KIND, verdicts.library_directory(tree_root))
        for item in parsed.order:
            number = number_of_item(item)
            key, _lines, reason = keyed[item]
            if key is None:
                missing[number] = f"拒算：{reason}"
                continue
            raw_verdict, output = store.read_many([verdicts.verdict_relative(key), verdicts.output_relative(key)])
            record = verdicts.parsed_record(raw_verdict, key)
            if record is None:
                missing[number] = f"库里没有这把键（{key[:12]}）"
            elif record.get("verdict") != "green":
                missing[number] = f"库里这把键记着红（{key[:12]}），要现判"
            elif output is None or hashlib.sha256(output).hexdigest() != record.get("output_sha256"):
                missing[number] = f"库里这把键的产物读不回或对不上（{key[:12]}）"
            elif check_only:
                taken[number] = {"row_number": number}
            else:
                row_directory = os.path.join(output_root, item_name(number))
                if os.path.isdir(row_directory):
                    shutil.rmtree(row_directory)
                unpack_directory(output, row_directory)
                taken[number] = read_result(row_directory)
    if missing and broken("take-only-judges-missing-rows"):
        taken.update(judge_missing(sorted(missing)))
        missing = {}
    return taken, sorted(missing.items())


def take_rows_into_the_output_root(arguments: argparse.Namespace, printer) -> list[tuple[int, str]]:
    """只取那一趟（文件头「只取」，shared_steps.py take-rows 起）：选行、快照、请求与现判那一趟（main）逐字同一套，取库里有的行；交没取到的 [(序号, 为什么)…]。
    一行都不缺才往标准输出打 `row …` 与 `table …`；缺的逐行点名、打出路（经服务交分片入口补判）。"""
    started = now_ns()
    selected_rows = read_selected_rows(arguments.selected_rows)
    if arguments.empty_mutation_only:
        arguments.empty_mutation = True
    chosen = choose_rows(arguments, selected_rows)
    snapshot_root = os.path.join(arguments.work_root, "snapshot")
    sync_tree(arguments.snapshot_from or arguments.repo_root, snapshot_root)
    table = read_mutation_rows(os.path.join(snapshot_root, MUTATIONS_TABLE))
    jobs: list[tuple[int, SelectedRow | None, MutationRow | None]] = []
    if arguments.empty_mutation:
        first_row = locate_by_name(selected_rows[0], table).row if selected_rows else None
        if first_row is None:
            missing = [(0, "空变异要的表里第一行在快照的变异表里找不到")]
            printer(f"E171_ROW_NOT_IN_THE_LIBRARY row=0 reason={missing[0][1]}")
            printer(f"E171_ROWS_TAKEN taken=0 missing=1 of=1 check_only={'yes' if arguments.check_only else 'no'} output_root={arguments.output_root}")
            return missing
        jobs.append((0, None, first_row))
    jobs.extend((row.number, row, None) for row in chosen)
    requests = [row_request_of(number, selected, empty_of, table, snapshot_root) for number, selected, empty_of in jobs]
    settings = key_settings_of(arguments)

    def judge_missing(numbers: list[int]) -> dict[int, dict[str, object]]:
        """弄坏开关 take-only-judges-missing-rows 下才叫：缺的行交现判那一套（plan_rows、线道、记库、收尾）判。"""
        wanted = set(numbers)
        arguments.lanes, arguments.threads = 1, threads_per_lane(None, 1)
        state_directory = tempfile.mkdtemp(prefix=ITEM_STATE_PREFIX, dir=arguments.work_root)
        try:
            library = plan_rows(state_directory, snapshot_root, [request for request in requests if request.number in wanted], settings, printer)
            results = run_rows_through_the_library(library, [job for job in jobs if job[0] in wanted], table, names_in_the_tables(selected_rows, []),
                                                   set(), arguments)
            library.finish()
        finally:
            shutil.rmtree(state_directory, ignore_errors=True)
        return {int(fields["row_number"]): fields for fields in results.values()}

    taken, missing = rows_from_the_library(requests, settings, snapshot_root, arguments.output_root, arguments.check_only, judge_missing)
    for number, reason in missing:
        printer(f"E171_ROW_NOT_IN_THE_LIBRARY row={number} reason={reason}")
    printer(f"E171_ROWS_TAKEN taken={len(taken)} missing={len(missing)} of={len(jobs)} check_only={'yes' if arguments.check_only else 'no'} "
            f"output_root={arguments.output_root}")
    if missing:
        printer(f"  ✗ E171 逐行驱动只取：{len(missing)} 行库里没有键相同、判绿的那一条（上面逐行 E171_ROW_NOT_IN_THE_LIBRARY），这一趟一行都不现判")
        printer("     → 怎么办：经统一任务服务交分片入口补判：task-server/task submit -- bash scripts/experiment-apparatus/e171-row-shard.sh --root <共用输出根>"
                "（在这棵树的根上起；它文件头「分片」），判完再起这一趟")
        return missing
    if not arguments.check_only:
        for number, _selected, _empty_of in jobs:
            print("row " + " ".join(f"{key}={str(value).replace(' ', '_')}" for key, value in taken[number].items()))
        print(f"table lanes=0 threads_per_lane=0 rows={len(jobs)} table_wall_ns={now_ns() - started} "
              f"sum_of_row_ns={sum(int(value) for fields in taken.values() for key, value in fields.items() if key in ('copy_ns', 'compile_ns', 'device_ns', 'o3_ns') and str(value).isdigit())}")
    return []


# ── 自证（文件头「自证」） ──

SELFTEST_KEY_SETTINGS = RowKeySettings("-", "-", True, None, "cpu", "-")


def silent(_line: str) -> None:
    return None


def selftest_requests(numbers: tuple[int, ...] = SELFTEST_ROWS, refusals: dict[int, str] | None = None) -> list[RowRequest]:
    refusals = refusals or {}
    return [RowRequest(number, f"{number}\t2\t{SELFTEST_SOURCE_FILE}\t0\tm{number}\t{'0' * 12}", f"m{number}\t{SELFTEST_SOURCE_FILE}\tfn a()\tfn b()\t-p a\tt",
                       False, refusals.get(number, "")) for number in numbers]


def selftest_round(tree_root: str, work: str, requests: list[RowRequest], device_exits: dict[int, int] | None = None,
                   printer=print_to_standard_error) -> tuple[int, dict[int, int], list[str]]:
    """一趟（计划、线道、记库、收尾在一个进程里依次走，走的是 plan_rows、run_rows_through_the_library、RowLibrary.finish）：判一行换成写一份只随行号变的
    row-result.tsv（device_exits 点名的行照给的装置退出码写）。work 是这一趟新建的空目录。交 (finish 的退出码, {行号: 这一趟判了几次}, 问题)。"""
    output_root = os.path.join(work, "rows")
    judged: dict[int, int] = {}
    guard = threading.Lock()
    exits = device_exits or {}

    def written_row(number, selected, empty_of, table, rows_in_the_table, used_substitutes, arguments, lane):
        with guard:
            judged[number] = judged.get(number, 0) + 1
        row_directory = os.path.join(output_root, f"row-{number}")
        if os.path.isdir(row_directory):
            shutil.rmtree(row_directory)
        os.makedirs(os.path.join(row_directory, "dump"))
        fields: dict[str, object] = {"row_number": number, "row_name": f"m{number}", "status": STATUS_OK, "substituted_from": "-",
                                     "device_exit": exits.get(number, 0)}
        write_result(row_directory, fields)
        return fields

    library = plan_rows(os.path.join(work, "items"), tree_root, requests, SELFTEST_KEY_SETTINGS, printer)
    jobs = [(request.number, None, None) for request in requests]
    run_rows_through_the_library(library, jobs, [], set(), set(), argparse.Namespace(output_root=output_root, lanes=2), written_row)
    code = library.finish()
    problems = [f"row-{request.number}:no_row_result" for request in requests
                if not os.path.isfile(os.path.join(output_root, f"row-{request.number}", "row-result.tsv"))]
    return code, judged, problems


def run_selftest_round(tree_root: str) -> int:
    """selftest-round（自证 reuse-three-questions 的起法）：row-1、row-2 在 tree_root 上走一趟 selftest_round；退 finish 的码，有行没有 row-result.tsv 退 1。"""
    work = tempfile.mkdtemp(prefix="e171-row-driver-selftest-round-")
    try:
        code, _judged, problems = selftest_round(tree_root, work, selftest_requests())
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for problem in problems:
        print(f"  ✗ {SELFTEST_ROUND_MODE}：{problem}", file=sys.stderr)
    return EXIT_FINISH_RED if problems else code


@contextlib.contextmanager
def selftest_library_scene(numbers: tuple[int, ...]):
    """一棵只有假装置入口的树（long_run_item_verdicts 的 plant_fake_module），这几行的装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成 measure_alpha
    （行号最小那一行）与 measure_beta（其余几行）；命名空间是这份现场自己的；键相同的全当复用哨兵、不抽强制现判。交 (现场根, 树)，出来时环境换回原样。"""
    from unittest import mock
    verdicts = item_verdicts_module()
    checks = verdicts.lib_reuse_checks
    with tempfile.TemporaryDirectory(prefix="e171-row-driver-selftest-") as root:
        tree = os.path.join(root, "tree")
        verdicts.plant_fake_module(tree)
        entries = os.path.join(root, "entries.tsv")
        with open(entries, "w", encoding="utf-8") as handle:
            for position, number in enumerate(sorted(numbers)):
                handle.write(f"{item_name(number)}\t{verdicts.FAKE_MODULE}::{'measure_alpha' if position == 0 else 'measure_beta'}\n")
        settings = {verdicts.LIBRARY_DIRECTORY_VARIABLE: os.path.join(root, "library"), verdicts.SUBSTITUTION_VARIABLE: entries,
                    checks.SENTINEL_COUNT_VARIABLE: verdicts.ALL_SENTINELS}
        with mock.patch.dict(os.environ, settings):
            for name in (verdicts.FAULT_VARIABLE, verdicts.FAKE_JUDGE_VARIABLE, checks.SENTINEL_SEED_VARIABLE):
                os.environ.pop(name, None)
            yield root, tree


@contextlib.contextmanager
def fake_cargo_ahead_on_the_path():
    """算键时判法闭包里点名的 cargo 要问 --version（admission.py 的 KeyContext.external）：PATH 最前面放一份只打一行版本的假 cargo，不起真 cargo。"""
    from unittest import mock
    with tempfile.TemporaryDirectory(prefix="e171-row-driver-fake-cargo-") as directory:
        path = os.path.join(directory, "cargo")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write('#!/usr/bin/env bash\necho "cargo 0.0.0 (E171 逐行驱动自证的假 cargo)"\n')
        os.chmod(path, 0o755)
        with mock.patch.dict(os.environ, {"PATH": directory + os.pathsep + os.environ.get("PATH", "")}):
            yield


def cell_reuse_three_questions(printer=print) -> bool:
    """复用三问与复用哨兵的三种红（.claude/rules/test-script-discipline.md「复用也要自证」）：long_run_item_verdicts.py reuse-cells 那六格，
    样板树是空的，起法是本文件的 selftest-round。"""
    verdicts = item_verdicts_module()
    with tempfile.TemporaryDirectory(prefix="e171-row-driver-reuse-cells-") as work:
        template = os.path.join(work, "template")
        os.makedirs(template)
        command = [sys.executable, "-B", os.path.realpath(__file__), SELFTEST_ROUND_MODE, "{tree}"]
        red = verdicts.reuse_cells(SCRIPT_KEY, template, [item_name(number) for number in SELFTEST_ROWS], os.path.join(work, "cells"), 0, command,
                                   printer=printer)
    return red == 0


def planned_keys(state_directory: str) -> dict[str, str | None]:
    """一趟 selftest_round 的计划里各项的键（state_directory 是那一趟的 <work>/items；一行都没进请求、没起 plan 的交空表）。"""
    verdicts = item_verdicts_module()
    if not os.path.isfile(os.path.join(state_directory, verdicts.PLAN_FILE)):
        return {}
    return {item: entry["key"] for item, entry in verdicts.read_plan(state_directory)["items"].items()}


def two_rounds(numbers: tuple[int, ...], requests: list[RowRequest], exits_of_the_earlier_round: dict[int, int] | None = None):
    """同一份现场里接连两趟 selftest_round（头一趟 exits_of_the_earlier_round 点名的行照给的装置退出码写）；交两趟各自的 (退出码, 判了几次, 问题)，
    与一句键变过的话（文件头「自证」末尾「键变过」）：两趟计划里同一项的键不同，或 finish 报了键在一趟中途变了；没变过交空串。"""
    verdicts = item_verdicts_module()
    with selftest_library_scene(numbers) as (root, tree):
        rounds, keys, midway = [], [], []
        for name, exits in (("earlier", exits_of_the_earlier_round), ("later", None)):
            work = os.path.join(root, name)
            os.makedirs(work)
            if name == "later" and broken("selftest-keys-moved-between-rounds"):
                verdicts.edit_once(os.path.join(tree, verdicts.FAKE_MODULE), *verdicts.FAKE_UNIT_CHANGE)
            lines: list[str] = []
            rounds.append(selftest_round(tree, work, requests, exits, lines.append))
            keys.append(planned_keys(os.path.join(work, "items")))
            midway += [line.strip() for line in lines if "中途变了" in line]
    earlier_keys, later_keys = keys
    moved = sorted(item for item in earlier_keys.keys() & later_keys.keys() if earlier_keys[item] != later_keys[item])
    if not moved and not midway:
        return rounds[0], rounds[1], ""
    shown = "、".join(f"{item} {(earlier_keys[item] or '拒算')[:12]}→{(later_keys[item] or '拒算')[:12]}" for item in moved)
    return rounds[0], rounds[1], (f"键变过：{shown or '两趟之间没变'}；{'；'.join(midway) or '一趟之内没变'}（判法闭包照 frozen_o3.OWN_REPOSITORY_ROOT 那棵树算，"
                                  "自证跑着的时候那里闭包碰到的源码、或现场里的装置入口被改了），这一格的结局不算数；→ 等改动定下来再起同一条命令")


def cell_refused_rows_are_judged_outside_the_library(printer=print) -> bool:
    numbers = (1, 2, 3, 4)
    earlier, later, moved = two_rounds(numbers, selftest_requests(numbers, {3: NEEDS_SUBSTITUTION, 4: MUTATION_TOUCHES_THE_MODULE_SET}))
    passed = not moved and earlier == (0, {1: 1, 2: 1, 3: 1, 4: 1}, []) and later == (0, {3: 1, 4: 1}, [])
    if moved:
        printer(f"     {moved}")
    if not passed:
        printer(f"     头一趟（退出码、各行判了几次、问题）{earlier}；再起那一趟 {later}（要 (0, {{3: 1, 4: 1}}, [])：拒算的两行现判、能算键的两行复用）")
    return passed


def cell_unstorable_outcome_is_judged_again(printer=print) -> bool:
    earlier, later, moved = two_rounds(SELFTEST_ROWS, selftest_requests(), {2: ROW_TIMEOUT_EXIT_CODE})
    passed = not moved and earlier == (0, {1: 1, 2: 1}, []) and later == (0, {2: 1}, [])
    if moved:
        printer(f"     {moved}")
    if not passed:
        printer(f"     头一趟 {earlier}；再起那一趟 {later}（要 (0, {{2: 1}}, [])：超时的 row-2 记红、现判，row-1 复用）")
    return passed


def cell_library_failure_in_a_lane_stops_the_run(printer=print) -> bool:
    from unittest import mock
    verdicts = item_verdicts_module()
    client = verdicts.prune_server_client
    outcome = None
    with selftest_library_scene(SELFTEST_ROWS) as (root, tree):
        work = os.path.join(root, "unreachable")
        os.makedirs(work)
        with mock.patch.object(verdicts, "record", side_effect=client.PruneServerFailure("unreachable", "E171 逐行驱动自证造的：记库连不上")):
            try:
                selftest_round(tree, work, selftest_requests(), printer=silent)
            except SystemExit as raised:
                outcome = raised.code
    if outcome != client.EXIT_UNAVAILABLE:
        printer(f"     线道里记库连不上，这一趟抛出的退出码是 {outcome}（要 {client.EXIT_UNAVAILABLE}）")
    return outcome == client.EXIT_UNAVAILABLE


def cell_take_only_names_the_rows_the_library_lacks(printer) -> bool:
    """只取（文件头「只取」）：头一趟只判 row-1；只取 row-1、row-2 时 row-1 解进输出根、row-2 点名、判一行的函数一次都没被叫；只查（check_only）一行都不写。
    不带缺省值：带缺省值的 def 头是 import 时执行的语句，会进每一行的判法闭包。"""
    judged_while_taking: list[int] = []

    def judge_missing(numbers: list[int]) -> dict[int, dict[str, object]]:
        judged_while_taking.extend(numbers)
        return {number: {"row_number": number, "status": STATUS_OK} for number in numbers}

    with selftest_library_scene(SELFTEST_ROWS) as (root, tree):
        work = os.path.join(root, "judged")
        os.makedirs(work)
        earlier_code, earlier_judged, _problems = selftest_round(tree, work, selftest_requests((SELFTEST_ROWS[0],)), printer=silent)
        output_root = os.path.join(root, "taken")
        taken, missing = rows_from_the_library(selftest_requests(), SELFTEST_KEY_SETTINGS, tree, output_root, False, judge_missing)
        written = sorted(os.listdir(output_root)) if os.path.isdir(output_root) else []
        checked_root = os.path.join(root, "checked")
        checked, checked_missing = rows_from_the_library(selftest_requests(), SELFTEST_KEY_SETTINGS, tree, checked_root, True, judge_missing)
        checked_written = os.path.exists(checked_root)
    passed = (earlier_code == 0 and earlier_judged == {SELFTEST_ROWS[0]: 1} and sorted(taken) == [SELFTEST_ROWS[0]] and [number for number, _ in missing] == [SELFTEST_ROWS[1]]
              and written == [item_name(SELFTEST_ROWS[0])] and sorted(checked) == [SELFTEST_ROWS[0]]
              and [number for number, _ in checked_missing] == [SELFTEST_ROWS[1]] and not checked_written and not judged_while_taking)
    if not passed:
        printer(f"     头一趟退 {earlier_code}、判了 {earlier_judged}（要 0、{{{SELFTEST_ROWS[0]}: 1}}）；只取那一趟取到 {sorted(taken)}、缺 {missing}、输出根里 {written}；"
                f"只查那一趟取到 {sorted(checked)}、缺 {checked_missing}、输出根{'建了' if checked_written else '没建'}；只取时判一行的函数被叫了 {judged_while_taking}"
                f"（要：取到 row-{SELFTEST_ROWS[0]}、点名 row-{SELFTEST_ROWS[1]}、只查不写、一行都不现判）")
    return passed


SELFTEST_CELLS = (("reuse-three-questions", cell_reuse_three_questions),
                  ("refused-rows-are-judged-outside-the-library", cell_refused_rows_are_judged_outside_the_library),
                  ("unstorable-outcome-is-judged-again", cell_unstorable_outcome_is_judged_again),
                  ("library-failure-in-a-lane-stops-the-run", cell_library_failure_in_a_lane_stops_the_run))


def selftest() -> int:
    red = []
    # 只取那一格接在这里、不进 SELFTEST_CELLS：那张表是模块级赋值，进每一行的判法闭包（文件头「只取」）
    cells = (*SELFTEST_CELLS, ("take-only-names-the-rows-the-library-lacks", cell_take_only_names_the_rows_the_library_lacks))
    with fake_cargo_ahead_on_the_path():
        for name, cell in cells:
            if cell(print):
                print(f"  ✓ {name}")
                continue
            red.append(name)
            print(f"  ✗ {name}")
            print(f"     → 怎么办：照文件头「自证」这一格的条件看 RowLibrary、plan_rows、run_rows_through_the_library 与 lane_worker 记库那一段，"
                  f"再在临时服务底下单独起 {SCRIPT_KEY} 的 --selftest 到它退 0")
    print(f"ROW_DRIVER_SELFTEST cells={len(cells)} red={len(red)}{' red_cells=' + ','.join(red) if red else ''}")
    return EXIT_FINISH_RED if red else 0


def selftest_entry(argument_list: list[str]) -> int:
    """--selftest 与 selftest-round <树>：只在临时 prune-server 底下认（客户端 selftest_service_problem），不在退 2。"""
    client = item_verdicts_module().prune_server_client
    problem = client.selftest_service_problem()
    if problem:
        print(f"  ✗ {argument_list[0]} 只在临时 prune-server 底下的自证里起：{problem}", file=sys.stderr)
        print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY}", file=sys.stderr)
        return EXIT_NOT_UNDER_A_TEMPORARY_SERVICE
    if argument_list == ["--selftest"]:
        return selftest()
    if len(argument_list) == 2 and argument_list[0] == SELFTEST_ROUND_MODE:
        return run_selftest_round(argument_list[1])
    print(f"  ✗ 用法：row_driver.py --selftest，或 row_driver.py {SELFTEST_ROUND_MODE} <树>（只给 reuse-cells 起）", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
