#!/usr/bin/env bash
# 崩溃放量的两个驱动共用的那几段：读配置的开关与显卡配置、组 cargo 的参数与环境、在本机跑一趟用例、从日志里取派活计划的数。
# 被 source 的函数库，只定义函数、不改 shell 选项；单独执行只认 --selftest（文件头「自证」一段）；scripts/crash/crash-amplification-item.sh（一个分项的一趟）与
# scripts/crash/crash-amplification-two-hosts.sh（两台各领一份）source 它。
# gate-similar: peer-host-lib.sh 它管第二台那几段（读多机配置、在第二台上跑命令、清场，不复原）；这里管本机那一趟与派活计划的数，不连第二台
#
# 调用方先设好这几个变量再调 crash_amplification_prepare：
#   repository_root  仓根
#   test_target      crates/singlefs-verification/tests/ 下的测试目标；test_filter 用例函数名（可空）
# crash_amplification_prepare 之后可用：
#   configuration_file（多机配置的路径，文件头「配置」一段怎么找）、gpu_switch（0 / 1）、across_machines_switch（0 / 1）、features（开了 GPU 是 gpu，没开是空串、cargo 不带 --features）、
#   gpu_cards_by_priority（环境里的 GPU_CARDS_BY_PRIORITY，有就原样交给本机那一趟；没有是空串，照样往下跑）、
#   common_environment、cargo_build_arguments、cargo_test_arguments（数组）
#   卡只归 GPU 批处理验证服务（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：这里不领卡、不挑卡、不清场，不读多机配置里的显卡先后。
#     开了 GPU 的判段（验证包 crash_amplification 的 judge_my_blocks_on_gpu）把要判的块写成判定库里的待判行、等 GPU 批处理服务判完；服务读不了时判段一直等，卡住由任务服务判。
#     弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=cards-read-from-configuration 下从多机配置读 GPU_CARDS_BY_PRIORITY（不看环境），
#     由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红（红在哪一格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#   crash_amplification_recorded_blocks <日志>        打日志里 CRASH_AMPLIFICATION_RECORDED 行 blocks= 之和（CRASH_LIBRARY_RECORDED 行不带块数，记 0）。
#   先编后判（开了 GPU 的分项驱动在录入段之后、判段之前先编好判段要的测试二进制，判段那几趟直接起它、不再经 cargo）：
#   crash_amplification_build_judge_binary <日志>     判段之前调。照 cargo_build_arguments（带判段那一趟的特性、--no-run）在本机编一趟
#       （同 crash_amplification_run_on_this_host 的 RocksDB 预编库核对与 TERM 转发，只起编的那一步），编成了从 <日志> 里 cargo 打的 Executable 行取这个测试目标的二进制
#       （crash_amplification_test_binary_in_log），放进 crash_amplification_judge_binary、往标准错误打一行「· 判段要的测试二进制编好了」；取不到打一行「!」、留空。
#       交回编那一趟的退出码（非 0 时 crash_amplification_judge_binary 留空，调用方照 crash_amplification_exit_code_of_a_failed_run 分类）。
#   crash_amplification_judge_binary                   不空时 crash_amplification_run_on_this_host 直接起它（工作目录是验证包目录，补上 cargo 给测试进程设的 CARGO_MANIFEST_DIR、
#       CARGO_PKG_NAME、CARGO_CRATE_NAME、CARGO_PRIMARY_PACKAGE、CARGO；libtest 参数取 cargo_test_arguments 里 -- 之后那几个），不起 cargo；空着照两步 cargo。
#       调用方也可以直接设成别处编好的那一份（分项驱动的 --judge-binary）。弄坏开关 CRASH_AMPLIFICATION_LIB_BREAK=judge-binary-unused 下不认它、照两步 cargo。
# 交给本机那一趟的环境由 crash_amplification_environment_for_this_host 组（数组 this_host_environment，env 的参数）；
#   驱动自证里换掉本机那一趟时也经它组，只把起 cargo 换成假的。
#
# 运行条件与失败分类（崩溃放量入口与分项驱动共用）：
#   crash_amplification_run_condition_holds [<仓根>]   两个驱动文件头 run-condition: check 调它（preflight 用 bash -c 在仓根起）；只判配置：
#       开了 GPU 就要在多机配置里写了显卡、点名的卡与双机开关对得上。不连第二台、不碰卡：那两样跑起来才知道，用不上时驱动在运行中退 78。
#   crash_amplification_cards_match_the_hosts <双机 0/1> <显卡配置的值> <这一台的地址> <另一台的地址> <HOSTS>
#       显卡先后每一项写 <HOSTS 里的地址>:<卡号>:<额度>（从右切两刀，地址可以是 IPv6；crash_amplification_card_list_problem 判写法，地址不在 HOSTS 里的拒）；
#       双机开着两台都要有点名的卡，双机没开不许点名另一台的卡。
#   这一台与另一台（两台对称，同一份多机配置两台判出同一个结局）：只照 scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses 认
#       （crash_amplification_host_addresses：它把 HOSTS 与这一台网卡上的地址求交，恰好一项是这一台），不读随台变的角色变量；哪一台起活、哪一台是第二台都不改判法。
#       那一份判不过（退非 0）时照它的退出码与它打的「✗ …」那一句原样报：那一句说的是这一台网卡上的地址（HOSTS 里没有、有几项、读不出）才说认不出这一台，
#       别的（缺键、写法不对）只报那一句；弄坏开关 emit-failure-said-host-unrecognized、host-unrecognized-unsaid 由本份 --selftest 证红（下面「弄坏开关」表）。
#       弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=this-host-from-hosts-order 下不问那一份、一律当 HOSTS 头一项是这一台；card-address-outside-hosts-accepted 下
#       显卡先后里地址不在 HOSTS 的项照认、当成另一台的卡。两条由 scripts/crash/crash-amplification-item.sh --selftest 证红。
#   crash_amplification_failure_kind <日志…>           一趟没跑成，日志里说的是哪一类：observer-images-without-a-card（观察者交回的盘要在卡上判、取不到卡：验证包打的 CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS 行，
#       任务不拿卡、这一族待判行 GPU 批处理服务还不收，记本次未跑，调用方退 78；经 cargo test 出来的退出码是 101，只能按这一行认）、
#       device（显卡或显存用不上，含给定状态表的判器借不到显存那一行，运行条件，调用方退 78）、
#       build（这份二进制跑不了这一趟：结论的版本与树对不上、配置开了 GPU 而没编进 gpu 特性，运行条件，调用方退 78）、
#       algorithm（算法上判不了：有状态 GPU 判不了、内核展开超预算、建不出表，调用方退 248），都不是打空串；
#       几类都有时先 observer-images-without-a-card、再 device、再 build
#   crash_amplification_exit_code_of_a_failed_run <那一趟的退出码> <日志…>   打调用方该退的码：observer-images-without-a-card、device 与 build 78、
#     algorithm 248，别的原样（内存包装退的 245 显存不够照原样，出路由 crash_amplification_explain_exit_code 的 245 那一支打）
#   crash_amplification_explain_exit_code <要退的码> [<日志…>]   打出路（标准错误）：给了日志而日志里是 observer-images-without-a-card 那一类时 78 打
#       「这条用例的观察者交回了盘、要上卡判，任务不拿卡；等 GPU 批处理服务收这一族待判行」（crash_amplification_observer_images_without_a_card_howto）；别的照码打。
#   弄坏开关读 CRASH_AMPLIFICATION_ITEM_BREAK（开关表与该红的格在 scripts/crash/crash-amplification-item.sh 文件头）：
#   accept-missing-peer-cards、accept-peer-cards-without-across、accept-missing-card-list、device-unusable-as-run-failure、algorithm-refusal-as-run-failure、
#   version-differs-as-run-failure、gpu-feature-missing-as-run-failure、this-host-from-hosts-order、card-address-outside-hosts-accepted。
#   given-states-run-condition-as-run-failure（给定状态表借不到显存那一行不认、不退 78）只由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红
#   （红在哪一格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#   其中 device-unusable-as-run-failure、algorithm-refusal-as-run-failure 打开时 scripts/crash/crash-amplification-two-hosts.sh --selftest 也红
#   （红在哪几格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#   配置那一处的开关 configuration-from-tree-root 由 scripts/crash/crash-amplification-item.sh --selftest 证红（表与该红的格在那份文件头）。
#   运行目录那一处的开关 home-under-home-directory 由 scripts/crash/crash-amplification-item.sh --selftest 证红（表与该红的格在那份文件头）。
#
# 弄坏开关轮（崩溃放量入口与双机驱动的 --selftest 末尾共用；.claude/rules/test-script-discipline.md「测试脚本自己也要被测试证明有效」：
# 复跑的一方逐个打开开关、看它红在点名的那一项）：
#   crash_amplification_read_switch_table <脚本文件> <变量名…>
#       读那份文件头的开关表（第一行代码之前的注释块），一个开关打一行「<变量>=<开关名> <TAB> <点名的格，空格隔开>」，照表里的次序。表的写法（B 写法，
#       scripts/mutation/mutate.sh 文件头「弄坏开关」一段是样板）：一行「#   <变量>=<开关名>  <弄坏哪一处>」（变量是给的变量名之一），下一行起以「#       」
#       开头写点名的格（、隔开；括号里是说明与失败行的句首，不读，括号外的每一段都算一格）。「第 N 格」读成 N（崩溃放量入口的格是编号的）；
#       写错的格名照样读进来，开关轮比格时红。崩溃放量入口与双机驱动的 --selftest 都经它读表。
#   crash_amplification_switch_wheel <放输出的目录> <取格的正则> <只比退出码 0/1> -- <自证命令…>
#       调用方先从自己文件头的开关表现读出开关（crash_amplification_read_switch_table，不在代码里另抄名单），填两个同序数组：switch_wheel_assignments 每个开关一项「<变量>=<开关名>」；
#       switch_wheel_expected 那个开关点名的格，空白隔开，写成「-」表示表里没列它、打开后应当退 0、一格不红。
#       每个开关一个子进程（子 shell 里 exec）：先 unset 数组里出现过的每个变量、只 export 这一个，再设 CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER（指到这一轮写的标记文件；
#       调用方用 crash_amplification_switch_wheel_inner 认它，认出来就不再起开关轮），跑自证命令，标准输出与标准错误并进 <目录>/switch-wheel-<序号>.out，
#       一批一批起、逐个 wait 收退出码。并行度：CRASH_AMPLIFICATION_SELFTEST_JOBS（正整数），没给取 SINGLEFS_THREAD_CAP（服务交下来的），再没给取登记的重活默认并行度（.claude/heavy-thread-default）。
#       对上的逐个打 ·（标准输出），对不上的逐个打 ✗（标准错误，出路由调用方的汇总句给）；交回之后 switch_wheel_failures 是对不上的处数
#       （含派出收回对不上）、switch_wheel_collected 是收回的个数。
#   crash_amplification_switch_wheel_verdict <点名的格> <退出码> <输出文件> <取格的正则> <只比退出码 0/1>
#       一个开关对不对得上：打实测红的格（取格的正则是 sed -E 的写法、一组括号括住格名；去重、sort -V 排好、空格隔开）。点名了格的要退 1、
#       红的格与点名的逐格相等；「-」的要退 0、一格不红；点名的格是空串（表里那一行没写格）一律对不上。
#       第五个参数是 1 时不比格、只比退出码：调用方的弄坏开关用（双机驱动的 wheel-ignores-cells），由它自证的 switch-wheel-judges 格拿造出来的输出证红。
#
# 源码摘要（两个驱动共用；崩溃放量入口与崩溃点重放入口经它们用）：
#   crash_amplification_source_digest <树根>   打「<摘要> files=<文件数>」，读不了、一个源文件都没有交回 1。分项驱动在录入段跑成后记一份、判段起跑前与收尾后各算一次，
#       双机驱动在计划那一步记、合并那一步核账之前算一次；对不上都是「跑的过程中源码变了」：打 CRASH_AMPLIFICATION_NOT_RUN 行、退 77（本次未跑），不落红。各驱动的开关与自证在各自文件头。
#
# 判器输入核对（崩溃点重放入口的测试函数那一段与门禁 crash-point-replay 各片起计划那一步放的测试二进制之前用）：
#   crash_amplification_judging_inputs_unchanged <测试二进制> <树根>   编这个二进制时算判器键所读的文件（构建脚本写在构建输出目录 watched_file_digests.tsv 里的每一份）
#       在树根下此刻的内容还是不是编的那一刻读到的：打「same watched=<份数>」交 0（键必然没变，直接起这个二进制）、「changed watched=… changed=<份数>：<路径>」交 1（键可能变了）、
#       「unknown <原因>」交 2（认不出）。写法与判法在函数前的注释；弄坏开关 judging-inputs-digest-unread 见下面「弄坏开关」表。
#   crash_amplification_judging_inputs_of_binaries <树根> <并行数> <测试二进制>…   上一条对一批二进制一次做完（一个 python 进程、同一个构建输出目录的键表与摘要答案只读只算一遍），
#       每个二进制一行「<交回值><TAB><上一条会打的那一行>」，次序同给的次序；门禁 crash-point-replay 的片对计划那一步放进去的几百个测试二进制用它。弄坏开关 judging-inputs-batch-answers-shared。
#   crash_amplification_place_binary_without_debug_info <源> <目标>   拷测试二进制（跨文件系统放不成硬链接时）：objcopy --strip-debug 写出去掉调试段的一份，原件不动，用不上整份 cp
#       （交 0 去了调试段、3 整份拷、1 拷不成）。弄坏开关 debug-info-kept-on-copy。
#
# 线程数的缺省（所有的默认都跑 31 线程，双机各台各 31，机器忙时固定、不看忙闲）：
#   线程只读统一任务服务交的：crash_amplification_default_threads 打服务交的 SINGLEFS_THREAD_CAP，没交取登记的重活默认并行度
#   （.claude/heavy-thread-default，经 scripts/hosts/heavy-threads.sh）。引擎的核对线程（crash_amplification_prepare 交的 SINGLEFS_CRASH_AMPLIFICATION_THREADS）取它；
#   本机那一趟两步 cargo 的线程不在这里定（下面「包装、内存上限与线程」）。
#   弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=threads-default-cores：缺省取本机核数减一，由 scripts/crash/crash-amplification-item.sh --selftest 证红（表在那份文件头）。
#
# 包装、内存上限与线程：本机那一趟不套任何包装（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03：一整条一个任务，上限由 task-server 按记录给、撞了放大重排；
#   records/2026-11-01-判定库服务化.md：包装只认服务起尝试时套的最外一层），两步 cargo 直接起，编译与测试线程照服务交下来的
#   CARGO_BUILD_JOBS、RUST_TEST_THREADS（SINGLEFS_THREAD_CAP 同）原样继承。
#   弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=this-host-service-threads-dropped：两步 cargo 起之前清掉 CARGO_BUILD_JOBS、RUST_TEST_THREADS，吃 cargo 自己的缺省，
#   由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红（红在哪几格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#
# 被停（crash_amplification_run_on_this_host；欠账 C797、C801）：本机那一趟的编、跑两步各放后台起、进程号记下、wait 它；收到 TERM 只把 TERM 转给记下的那一个进程号
#   （env exec 成的 cargo；不按名字找，不遍历它的子孙）、不再起下一步、
#   等记下的进程号退出（等的上限 CRASH_AMPLIFICATION_TERM_WAIT_SECONDS，默认 100 秒，没退完打 ✗ 与出路）、
#   放回原来的 TERM 处理，再对自己发 TERM：调用方没设 TERM 处理（崩溃放量分项驱动的主线、放后台的子 shell）就此按被停退 143；设了的，它的处理跑过之后这一趟交回 143。
#   没被停时退出码照旧转出：编的那一步退非 0 就不起跑的那一步，交回的是那一步的退出码。日志仍是一份：编的输出在前、跑的输出在后。
#   crash_amplification_judge_binary 不空（文件头「先编后判」）时只一步：直接起那份二进制，记下的进程号是它；crash_amplification_build_judge_binary 只起编的那一步。
#
# 自证（单独执行 bash scripts/crash/crash-amplification-lib.sh --selftest；被 source 时不跑）：调真的 crash_amplification_run_on_this_host，
#   仓根换成临时目录下的假树，cargo 是 PATH 前面的假脚本（跑着时占着一个文件、收到 TERM 慢一点放掉再退），不起重活；
#   失败行是「✗ 自证 <格名>：…」。格：term-reaches-cargo（放后台的子 shell 里被停）、term-reaches-cargo-in-foreground（调用方的 shell 里没设 TERM 处理）、
#   term-returns-143-under-caller-trap（调用方设了 TERM 处理）、term-during-build-skips-test、exit-code-passes-through、build-failure-skips-test、caller-term-trap-kept、
#   rocksdb-entry-refused（假树的 验证包依赖 rocksdb、进程环境里没有预编库的取值：交回 78、两步 cargo 都不起；别的格假树里没有这份清单，entry 报不判）。
#   judging-inputs-batch-agrees（同一份假树与假二进制，按键所读的文件没变、一份不在了、摘要表是空的三种状态，把有表、没表、有表三个二进制一批交给
#   crash_amplification_judging_inputs_of_binaries，并行 1 与 3 各一遍：每一行与逐个调时的交回值与打的一行相同）、
#   place-binary-without-debug-info（PATH 前面放假 objcopy：成了就用它写出目标、交 0、原件不动；退非 0 就整份拷、交 3）两格也直接调库里的函数。
#   三格直接调 crash_amplification_judging_inputs_unchanged（假的构建输出目录与带键表的假二进制）：judging-inputs-unchanged（键所读的文件没变、闭包外的文件变了都打 same 交 0）、
#   judging-inputs-changed（键所读的一份文件内容变了、或不在了，打 changed 点名那份交 1）、judging-inputs-unknown（二进制里找不到对应的键表、摘要表是空的，打 unknown 交 2）。
#   另两格直接调 crash_amplification_host_addresses、起真的多机配置判法（假配置，这一台的地址经判法只在自证里认的 CONFIGURATION_SELFTEST_LOCAL_ADDRESSES 给成 127.0.0.1）：
#   emit-addresses-missing-key-named（写全的配置判得过、认出 127.0.0.1；开了 GPU 而缺 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 的那一份报出缺键那一句、不说认不出这一台）、
#   emit-addresses-host-absent-named（HOSTS 里没有 127.0.0.1 的那一份说认不出这一台、连着判法的原句）。
#   judge-binary-built-before-the-judge-runs（调真的 crash_amplification_build_judge_binary 与 crash_amplification_run_on_this_host：假 cargo 在 --no-run 那一步写一份假的测试二进制、打 Executable 行；
#   外壳编完记一笔判段开始，再起两趟本机那一趟：调用记录里只有判段开始之前那一回编，判段两趟都是直接起那份二进制（工作目录是验证包目录、libtest 参数照 -- 之后的），没有 cargo）。
#   observer-images-without-a-card（库里的函数直接调：接在「test <名> ... 」后面的 CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS 行
#   认成 observer-images-without-a-card、那一趟的 101 改退 78、78 的出路是等 GPU 批处理服务收这一族待判行；同一个记号不在行首的一句话不认、101 原样退）。
#   只在临时 prune-server 上跑（下面「判定库」：引擎的判定只在服务里，自证不写真服务）：开跑先调 crash_amplification_selftest_service_problem，
#   不是 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 起的临时服务就退 2、一格都不跑。
#   末尾一轮（开关轮）从下面的表现读开关与点名的格，逐个打开再跑一遍 --selftest，核红的格与点名的逐格相等（判法是同一份库的 crash_amplification_switch_wheel）。
# 弄坏开关 CRASH_AMPLIFICATION_LIB_BREAK=<项>（几个用逗号连；只给 --selftest 证红用）。写法同上面「弄坏开关轮」一段：一行「#   <变量>=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格。
#   CRASH_AMPLIFICATION_LIB_BREAK=term-not-forwarded  收到 TERM 只记一笔、不转给记下的那一趟（cargo 收不到，一直跑到它自己跑完）
#       term-reaches-cargo（「✗ 自证 term-reaches-cargo：」）、term-reaches-cargo-in-foreground（「✗ 自证 term-reaches-cargo-in-foreground：」）、term-returns-143-under-caller-trap（「✗ 自证 term-returns-143-under-caller-trap：」）、term-during-build-skips-test（「✗ 自证 term-during-build-skips-test：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=term-not-waited  转完 TERM 不等它退出，当场按被停退
#       term-reaches-cargo（「✗ 自证 term-reaches-cargo：」）、term-reaches-cargo-in-foreground（「✗ 自证 term-reaches-cargo-in-foreground：」）、term-returns-143-under-caller-trap（「✗ 自证 term-returns-143-under-caller-trap：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=term-trap-not-restored  那一趟返回之后不放回调用方原来的 TERM 处理
#       term-returns-143-under-caller-trap（「✗ 自证 term-returns-143-under-caller-trap：」）、caller-term-trap-kept（「✗ 自证 caller-term-trap-kept：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=rocksdb-entry-unchecked  起这一趟的 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
#       rocksdb-entry-refused（「✗ 自证 rocksdb-entry-refused：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=judging-inputs-digest-unread  判器输入核对认出构建输出目录之后不比摘要，树根下的文件变了也打 same
#       judging-inputs-changed（「✗ 自证 judging-inputs-changed：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=judging-inputs-batch-answers-shared  一批二进制的判器输入核对每个都打同一个答案（批里排在最前那个二进制的）
#       judging-inputs-batch-agrees（「✗ 自证 judging-inputs-batch-agrees：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=debug-info-kept-on-copy  拷测试二进制时不去调试段、整份拷
#       place-binary-without-debug-info（「✗ 自证 place-binary-without-debug-info：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=test-line-markers-unread  判行首记号之前不去掉 libtest 的「test <名> ... 」（接在它后面的录入行与判类别的记号认不出，没跑成的当跑成或退错码）
#       test-line-markers（「✗ 自证 test-line-markers：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=emit-failure-said-host-unrecognized  多机配置判法 --emit-addresses 退非 0 一律报「认不出这一台是 HOSTS 里哪一项」，不看它 ✗ 那一句说的是什么
#       emit-addresses-missing-key-named（「✗ 自证 emit-addresses-missing-key-named：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=host-unrecognized-unsaid  判法 ✗ 那一句确是认不出这一台（说的是这一台网卡上的地址）也不说认不出这一台
#       emit-addresses-host-absent-named（「✗ 自证 emit-addresses-host-absent-named：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=judge-binary-unused  本机那一趟不认判段之前编好的测试二进制，照两步 cargo 起（判段那几趟又经 cargo 编）
#       judge-binary-built-before-the-judge-runs（「✗ 自证 judge-binary-built-before-the-judge-runs：」）
#   CRASH_AMPLIFICATION_LIB_BREAK=observer-images-without-a-card-unrecognized  crash_amplification_failure_kind 不认观察者交回的盘取不到卡那一行（那一趟照退 101 原样退、当红）
#       observer-images-without-a-card（「✗ 自证 observer-images-without-a-card：」）、test-line-markers（「✗ 自证 test-line-markers：」）
#
# 配置（多机配置合成一份，不零散配置）：
#   只读一份多机配置 ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}（git 忽略、这台机器一份；模板是仓根的 multi-host.env.example）。
#   主工作树的根按 git common-dir 取（crash_amplification_multi_host_configuration_file，找法与 scripts/crash/crash-point-replay-shard-configuration-check.sh、
#   scripts/crash/gpu-quiesce-run.sh 相同）：在 HEAD + 暂存区的临时 worktree 里跑也读主工作树那一份；树不是 git 工作树时退回 <树根>/multi-host.env。
#   开关 ENABLE_GPU、ENABLE_ACROSS_MACHINES 的判法不改（没写或写 0 算关，键的定义只在 crash-point-replay-shard-configuration-check.sh）。
#   显卡的两个键也写在这一份里：GPU_CARDS_BY_PRIORITY（不清场时显卡的先后与每张卡的显存额度，读法在
#   crates/singlefs-verification/src/gpu_card_configuration.rs 的 parse_cards_by_priority）、GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（清场之后的先后与额度，同一种写法）。
#   两个键由 GPU 批处理验证服务读来开卡（gpu-verification-server/README.md）；这里只在运行条件（crash_amplification_run_condition_holds）里核它们写得对不对，
#   交给用例的只有环境里的 GPU_CARDS_BY_PRIORITY（文件头「crash_amplification_prepare 之后可用」那一段），不从这里挑。
# 走 GPU 的计算：开了 GPU（ENABLE_GPU=1）引擎就三类全走 GPU，不经环境变量交、不登正表。
#   不走 GPU 的那几截计算登记在仓内豁免表 .claude/gate.d/gpu-exemptions.tsv（每行用户点头），门禁 code-source-discipline 的 gpu-exemptions 格核它与代码标记逐项对上。
#   本机那一趟另交 SINGLEFS_MULTI_HOST_CONFIG=<这一份的绝对路径>：用例不设它时按自己的树根找 multi-host.env，在 worktree 里会找不到主工作树那一份
#   （派到第二台的那一趟同样在这棵共享的树里起，照同一种找法取到同一份配置）。
# 运行目录（分项驱动、双机驱动、崩溃放量入口、崩溃点重放入口与它的分片驱动共用 crash_amplification_home_directory）：日志与这一项的编译目录放在
#   ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-<工作根>/crash-amplification}/runs/<标签>/；工作根照 .claude/hooks/lib_work_root.py（数据根下的 work，
#   落在多机配置 NFS_SHARED_ROOTS 点名的共享根里、两台同一绝对路径），不经 ~ 与 $HOME 取：第二台的家目录是另一个，日志两台要读得到同一份。
#   取不到工作根（数据根没配）交回 1、打原因与出路，调用方退 2；不退回家目录（弄坏开关 home-under-home-directory 下取 $HOME 底下那一份）。
# 判定库（分项驱动与双机驱动共用；崩溃放量入口与崩溃点重放入口经它们用）：崩溃判定只在 prune-server 里那一份库（.claude/rules/verification.md「判定库只有一份」），
#   引擎经 prune-server 的客户端直接读写，连哪一处照客户端的取法（本机的 unix socket；没有 socket 时多机配置的 PRUNE_SERVER_TCP_ADDRESS，第二台走它）。
#   本机没有崩溃判定库的目录：这里不拼库目录、不往引擎送库的变量、不删库、不拷库；两台、几个进程判的块都直接写进服务里同一份，判过的块照键复用。
#   服务连不上时引擎退 75（prune-server 不可用），调用方照 75 整趟退出、不当「库里没有」现判。

source "$(dirname "${BASH_SOURCE[0]}")/../hosts/heavy-threads.sh"
# crash_amplification_clip_text <字节数>：标准输入每行取前 <字节数> 个字节，再丢掉被切在中间的半个字符，输出总是合法的 UTF-8（GNU 的 cut -c 与 cut -b 都按字节切，三个字节的汉字会被切断；给人看的失败行与日志摘录用它截）
crash_amplification_clip_text() { cut -b "1-$1" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null; }
CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"  # librocksdb-sys 的 bindgen 找 stdbool.h；哪个版本的目录在就用哪个，不在的 -I 无害
# 给定状态表的判器借不到显存时引擎打的那一行的行首（crates/singlefs-verification/src/crash_amplification.rs 的 GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX）：
# 运行条件不满足，调用方退 78；crash_amplification_failure_kind 的 device 那一组与崩溃点重放入口 scripts/crash/crash-point-replay.sh 按条复用的那一段都照这一份认
CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX="CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET"
# 观察者交回的盘要在卡上判、取不到卡时验证包打的那一行的行首（crates/singlefs-verification/src/crash_amplification.rs 的
# IMAGES_AFTER_THE_OBSERVERS_WITHOUT_A_CARD_LINE_PREFIX）：任务不拿卡，记本次未跑、调用方退 78（文件头「运行条件与失败分类」）；
# 门禁 crash-point-replay 的崩溃枚举用例那一路照同一份认
CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS_LINE_PREFIX="CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS"

# 自证只在临时 prune-server 上跑（文件头「自证」；scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service）：
#   打客户端 selftest_service_problem() 的那一句，是临时服务时打空串；读不了客户端照读不了说。分项驱动、双机驱动、崩溃放量入口的自证开跑前都调它，不是空串就退 2、一格都不跑
crash_amplification_selftest_service_problem() {
  local problem
  if ! problem="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' \
       "$(dirname "${BASH_SOURCE[0]}")/../prune-server" 2>&1)"; then
    printf '%s\n' "读不了 scripts/prune-server/prune_server_client.py：${problem:-没有输出}"
    return 0
  fi
  printf '%s\n' "$problem"
}

crash_amplification_configuration_value() { # <配置文件> <键>：打那个键的值（去掉两头成对的一层引号）；文件不在、键没写打空串
  [[ -f "$1" ]] || return 0
  python3 - "$1" "$2" <<'PYTHON'
import sys
path, key = sys.argv[1], sys.argv[2]
for line in open(path, encoding="utf-8"):
    line = line.strip()
    if not line.startswith(key + "="):
        continue
    value = line[len(key) + 1:].strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
        value = value[1:-1]
    print(value)
    break
PYTHON
}

# 多机配置在哪（文件头「配置」）：打路径。SINGLEFS_MULTI_HOST_CONFIG 设了照它；没设取 <树根> 的 git common-dir 的上一层（主工作树的根）下的 multi-host.env，
# 取不到 common-dir（不是 git 工作树）退回 <树根>/multi-host.env。文件在不在不判（不在就是两个开关都关、一张卡都没写）。
# 弄坏开关 configuration-from-tree-root：按 <树根>/multi-host.env 找（在 worktree 里读不到主工作树那一份）。
crash_amplification_multi_host_configuration_file() { # <树根>
  local root="$1" common_directory=""
  if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then printf '%s\n' "$SINGLEFS_MULTI_HOST_CONFIG"; return 0; fi
  if ! crash_amplification_item_break_is configuration-from-tree-root \
     && common_directory="$(git -C "$root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    printf '%s/multi-host.env\n' "$(dirname "$common_directory")"
    return 0
  fi
  printf '%s/multi-host.env\n' "$root"
}

# 崩溃放量的运行目录（文件头「运行目录」）：打一行；取不到工作根打原因与出路、交回 1
crash_amplification_home_directory() {
  local work_root
  if [[ -n "${SINGLEFS_CRASH_AMPLIFICATION_HOME:-}" ]]; then
    printf '%s\n' "$SINGLEFS_CRASH_AMPLIFICATION_HOME"
    return 0
  fi
  if crash_amplification_item_break_is home-under-home-directory; then
    printf '%s\n' "$HOME/.local/share/singlefs/crash-amplification"
    return 0
  fi
  if ! work_root="$(python3 "$(dirname "${BASH_SOURCE[0]}")/../../.claude/hooks/lib_work_root.py")" || [[ -z "$work_root" ]]; then
    echo "  ✗ 取不到工作根（.claude/hooks/lib_work_root.py 没打出来），崩溃放量的运行目录没处放" >&2
    echo "  → 怎么办：在多机配置（multi-host.env）里写 SHARED_DATA_ROOT=<落在 NFS_SHARED_ROOTS 某一段下的绝对路径>，或设 SINGLEFS_CRASH_AMPLIFICATION_HOME=<两台都读得到的目录>" >&2
    return 1
  fi
  printf '%s/crash-amplification\n' "$work_root"
}

crash_amplification_prepare() {
  configuration_file="$(crash_amplification_multi_host_configuration_file "$repository_root")"
  gpu_switch=0; across_machines_switch=0; features=""
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)" == 1 ]] && gpu_switch=1
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_ACROSS_MACHINES)" == 1 ]] && across_machines_switch=1
  (( gpu_switch )) && features="gpu"
  # 交给本机那一趟的卡（文件头「crash_amplification_prepare 之后可用」那一段）：环境里的 GPU_CARDS_BY_PRIORITY，不读多机配置里的显卡先后；没有也不拒、不领。
  #   弄坏开关 cards-read-from-configuration 下从多机配置读（不看环境）
  gpu_cards_by_priority="${GPU_CARDS_BY_PRIORITY:-}"
  if crash_amplification_item_break_is cards-read-from-configuration; then
    gpu_cards_by_priority="$(crash_amplification_configuration_value "$configuration_file" GPU_CARDS_BY_PRIORITY)"
  fi
  common_environment=("BINDGEN_EXTRA_CLANG_ARGS=$CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS")
  local cores
  cores="$(crash_amplification_default_threads)"
  common_environment+=("SINGLEFS_CRASH_AMPLIFICATION_THREADS=$cores")
  cargo_build_arguments=(test --release -p singlefs-verification ${features:+--features "$features"} --test "$test_target" --no-run)
  cargo_test_arguments=(test --release -p singlefs-verification ${features:+--features "$features"} --test "$test_target" -- --nocapture)
  if [[ -n "${test_filter:-}" ]]; then cargo_test_arguments+=(--exact "$test_filter"); fi
  return 0
}

# 交给本机那一趟的环境（env 的参数）放进数组 this_host_environment：common_environment、附加环境、份、多机配置的路径（文件在才交，文件头「配置」），
# 挑中的那份显卡先后与额度（空的不交）。
crash_amplification_environment_for_this_host() { # <share> [<附加环境 键=值>…]
  local share="$1"
  shift
  this_host_environment=("${common_environment[@]}" "$@" "SINGLEFS_CRASH_AMPLIFICATION_SHARE=$share")
  [[ -n "${configuration_file:-}" && -f "${configuration_file:-}" ]] && this_host_environment+=("SINGLEFS_MULTI_HOST_CONFIG=$configuration_file")
  [[ -n "${gpu_cards_by_priority:-}" ]] && this_host_environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_by_priority")
  return 0
}

# 日志里录入的块数：CRASH_AMPLIFICATION_RECORDED 行 blocks= 之和；没有这种行打 0
crash_amplification_recorded_blocks() { # <日志>
  crash_amplification_lines_without_test_prefix "$1" \
    | awk '$1 == "CRASH_AMPLIFICATION_RECORDED" { for (i = 2; i <= NF; i++) if ($i ~ /^blocks=[0-9]+$/) total += substr($i, 8) } END { print total + 0 }'
}

# 判段要的测试二进制（文件头「先编后判」）：不空时 crash_amplification_run_on_this_host 直接起它、不经 cargo
crash_amplification_judge_binary="${crash_amplification_judge_binary:-}"

# 判段之前编好判段要的测试二进制（文件头「先编后判」）：只起编的那一步，取 Executable 行里的二进制放进 crash_amplification_judge_binary；交回编那一趟的退出码
crash_amplification_build_judge_binary() { # <日志>
  local build_exit built
  crash_amplification_judge_binary=""
  if crash_amplification_this_host_build_only=1 crash_amplification_run_on_this_host 0/1 "$1"; then build_exit=0; else build_exit=$?; fi
  (( build_exit == 0 )) || return "$build_exit"
  # cargo 在仓根起（crash_amplification_run_step），编译目录落在仓根底下时 Executable 行写的是相对仓根的路径：在仓根认、再补成绝对路径
  if built="$(cd "$repository_root" && crash_amplification_test_binary_in_log "$1" "${test_target:-}")"; then
    [[ "$built" == /* ]] || built="$repository_root/$built"
    crash_amplification_judge_binary="$built"
    echo "  · 判段要的测试二进制编好了（判段之前，cargo ${cargo_build_arguments[*]}）：$built；判段直接起它，不经 cargo、不重编" >&2
  else
    echo "  ! 编完了，日志 $1 里却没有 ${test_target:-} 的 Executable 行（或那份文件不在）：判段照旧经 cargo test 起，树在编完之后改过会在判段重编" >&2
  fi
  return 0
}

# 一台机器用几个核（文件头「线程数的缺省」）：服务交的 SINGLEFS_THREAD_CAP，没交取登记的重活默认并行度（.claude/heavy-thread-default）；
# 弄坏开关 threads-default-cores 下取本机核数减一
crash_amplification_default_threads() {
  if crash_amplification_item_break_is threads-default-cores; then
    echo $(( $(nproc) > 1 ? $(nproc) - 1 : 1 ))
    return 0
  fi
  heavy_thread_default
}

# crash_amplification_test_binary_in_log <日志> <测试目标>：cargo test 在日志里打的那个测试目标的测试二进制的路径（只编不跑的 --no-run 打「Executable tests/<目标>.rs (<路径>)」，
#   编了再跑的打「Running tests/<目标>.rs (<路径>)」；目标住在 tests/<目标>/main.rs 的写 tests/<目标>/main.rs）。文件在且可执行才打、交 0；
#   日志读不了、没有这一行或文件不在交 1、什么都不打。分项驱动的判段、崩溃点重放入口的测试函数那一段与门禁 crash-point-replay 计划那一步放测试二进制时用它取编好的那一份、
#   之后直接起它、不再经 cargo：编完之后树被别的会话改过，cargo 按文件修改时间再来一趟就要重编几分钟
crash_amplification_test_binary_in_log() {
  local built_path
  built_path="$(awk -v target="$2" '
    ($1 == "Running" || $1 == "Executable") && ($2 == "tests/" target ".rs" || $2 == "tests/" target "/main.rs") {
      line = $0; sub(/^[^(]*\(/, "", line); sub(/\)[[:space:]]*$/, "", line); print line; exit
    }' "$1" 2>/dev/null)"
  [[ -n "$built_path" && -f "$built_path" && -x "$built_path" ]] || return 1
  printf '%s\n' "$built_path"
}

# crash_amplification_cargo_path <树根>：cargo test 起测试二进制时给它的环境变量 CARGO 的值（真 cargo 的路径，不是 rustup 的代理）：验证包的工具链指纹
#   （crash_point_replay_progress.rs 的 of_the_cargo_running_this_test）读它与它旁边的 rustc。取 <树根里 rustc --print sysroot>/bin/cargo，取不到退回 PATH 上的 cargo
crash_amplification_cargo_path() {
  local sysroot
  if sysroot="$(cd "$1" && rustc --print sysroot 2>/dev/null)" && [[ -x "$sysroot/bin/cargo" ]]; then printf '%s\n' "$sysroot/bin/cargo"; else type -P cargo; fi
}

# crash_amplification_judging_inputs_unchanged <测试二进制> <树根>：编这个测试二进制时算判器键所读的文件，此刻在树根下的内容还是不是编的那一刻读到的。
#   验证包的构建脚本（crates/singlefs-verification/build.rs）逐组算判器键写进 OUT_DIR 的 judging_units_keys.tsv（库经 include_str! 编进二进制），同时把键所读的每份文件
#   （与它的 rerun-if-changed 清单同一批）的 SHA-256 写进同目录的 watched_file_digests.tsv（每行「摘要<TAB>相对仓根的路径」）。这里认出编进这份二进制的那一份键表对应的构建输出目录
#   （二进制所在目录的上一层的 build/singlefs-verification-*/out，键表整份在二进制里找得到的那一个），把摘要表里的每份文件按树根下此刻的内容重算一遍比：
#   全相同，键必然相同（键只由这些文件、工具链与 Cargo.lock 定，构建脚本自己也只在这批文件变了才重算）。有一份不同或不在了，键可能变了（也可能只是闭包外的单元或注释变了），
#   不在这里现算键：现算当前树的键没有只读的办法（缺的东西写在入口文件头「GPU」）。
#   打一行「same watched=<份数>」交 0；「changed watched=<份数> changed=<不同的份数>：<头几份的路径>」交 1；认不出（二进制或构建输出目录读不了、键表不在二进制里、摘要表是空的）
#   打「unknown <原因>」交 2。弄坏开关 CRASH_AMPLIFICATION_LIB_BREAK=judging-inputs-digest-unread 下认出构建输出目录之后不比摘要、一律打 same。
#   判法只在 crash_amplification_judging_inputs_of_binaries 里：这一条是它只给一个二进制的调法。
crash_amplification_judging_inputs_unchanged() {
  local line code
  line="$(crash_amplification_judging_inputs_of_binaries "$2" 1 "$1")" || return $?
  code="${line%%$'\t'*}"
  [[ "$code" =~ ^[0-9]+$ ]] || return 1
  printf '%s\n' "${line#*$'\t'}"
  return "$code"
}

# crash_amplification_judging_inputs_of_binaries <树根> <并行数> <测试二进制>…：上一条对一批二进制一次做完，一个 python 进程：每个构建输出目录的键表与摘要表只读一次、
#   摘要表里的文件对同一个构建输出目录只重算一遍（同一份构建输出编出来的二进制答案相同），各二进制在键表里的查找按 <并行数> 个进程分着做（1 不分）。
#   每个二进制打一行「<交回值><TAB><上一条会打的那一行>」，次序同给的次序，每一行与上一条单给这个二进制时打的、交的相同；起不来（python 退非 0）交回它的退出码、一行不打。
#   弄坏开关 CRASH_AMPLIFICATION_LIB_BREAK=judging-inputs-digest-unread 同上一条；judging-inputs-batch-answers-shared 下每个二进制都打同一个答案（批里排在最前那个二进制的，批量与逐个对不上）。
crash_amplification_judging_inputs_of_binaries() {
  local root="$1" jobs="$2" digest_unread=0 shared_answer=0
  shift 2
  crash_amplification_lib_break_is judging-inputs-digest-unread && digest_unread=1
  crash_amplification_lib_break_is judging-inputs-batch-answers-shared && shared_answer=1
  (( $# > 0 )) || return 0
  python3 -I - "$root" "$digest_unread" "$jobs" "$shared_answer" "$@" <<'JUDGING_INPUTS_PYTHON'
import concurrent.futures
import glob
import hashlib
import mmap
import multiprocessing
import os
import sys

root, digest_unread, jobs, shared_answer, binaries = sys.argv[1], sys.argv[2] == "1", int(sys.argv[3]), sys.argv[4] == "1", sys.argv[5:]
KEYS_FILE, DIGESTS_FILE = "judging_units_keys.tsv", "watched_file_digests.tsv"
SHOWN_PATHS = 5
build_outputs = {}   # 二进制上一层的 build 目录 → [(键表的修改时刻, 构建输出目录, 键表开头的一段, 键表)…]：fork 之前读好，各进程共用


def build_parent_of(binary):
    return os.path.join(os.path.dirname(os.path.abspath(binary)), os.pardir, "build")


def build_outputs_of(build_parent):
    found = []
    for directory in sorted(glob.glob(os.path.join(glob.escape(build_parent), "singlefs-verification-*", "out"))):
        keys_path, digests_path = os.path.join(directory, KEYS_FILE), os.path.join(directory, DIGESTS_FILE)
        try:
            with open(keys_path, "rb") as handle:
                table = handle.read()
            modified = os.path.getmtime(keys_path)
            if not table or not os.path.isfile(digests_path):
                continue
        except OSError:
            continue
        found.append((modified, directory, table[:1 << 20], table))
    return found


def locate(binary):
    """交 (交回值, 打的一行, 构建输出目录)：键表整份在这个二进制里找得到的构建输出目录里取修改时刻最晚的；找不到、读不了交 2 与 unknown 那一行。"""
    try:
        with open(binary, "rb") as handle:
            view = mmap.mmap(handle.fileno(), 0, access=mmap.ACCESS_READ)
    except (OSError, ValueError) as error:
        return 2, f"unknown 读不了测试二进制 {binary}：{error}", None
    build_parent = build_parent_of(binary)
    matched = []
    for modified, directory, head, table in build_outputs[build_parent]:
        at = view.find(head)
        while at >= 0 and view[at:at + len(table)] != table:
            at = view.find(head, at + 1)
        if at >= 0:
            matched.append((modified, directory))
    if not matched:
        return 2, f"unknown 编进 {binary} 的判器键表在 {build_parent}/singlefs-verification-*/out 里找不到对应的构建输出", None
    return 0, "", max(matched)[1]


def digests_answer(directory):
    """摘要表里每份文件按树根下此刻的内容重算再比：交 (交回值, 打的一行)。"""
    try:
        with open(os.path.join(directory, DIGESTS_FILE), encoding="utf-8") as handle:
            rows = [line.rstrip("\n").split("\t", 1) for line in handle if line.strip()]
    except OSError as error:
        return 2, f"unknown 读不了 {directory}/{DIGESTS_FILE}：{error}"
    if not rows or any(len(row) != 2 for row in rows):
        return 2, f"unknown {directory}/{DIGESTS_FILE} 是空的或某一行不是「摘要<TAB>路径」"
    changed = []
    for digest, path in rows:
        try:
            with open(os.path.join(root, path), "rb") as handle:
                now = hashlib.sha256(handle.read()).hexdigest()
        except OSError:
            now = None
        if now != digest and not digest_unread:
            changed.append(path)
    if changed:
        return 1, f"changed watched={len(rows)} changed={len(changed)}：{' '.join(changed[:SHOWN_PATHS])}{' …' if len(changed) > SHOWN_PATHS else ''}"
    return 0, f"same watched={len(rows)}"


for parent in dict.fromkeys(build_parent_of(binary) for binary in binaries):
    build_outputs[parent] = build_outputs_of(parent)
if jobs > 1 and len(binaries) > 1:
    with concurrent.futures.ProcessPoolExecutor(max_workers=min(jobs, len(binaries)), mp_context=multiprocessing.get_context("fork")) as pool:
        located = list(pool.map(locate, binaries, chunksize=max(1, len(binaries) // (jobs * 4))))
else:
    located = [locate(binary) for binary in binaries]
answers = {}   # 构建输出目录 → 摘要表那一遍的答案，同一个目录只算一遍
results = []
for code, text, directory in located:
    if directory is not None:
        if directory not in answers:
            answers[directory] = digests_answer(directory)
        code, text = answers[directory]
    results.append((code, text))
if shared_answer:
    results = [results[0]] * len(results)
for code, text in results:
    print(f"{code}\t{text}")
JUDGING_INPUTS_PYTHON
}

# crash_amplification_place_binary_without_debug_info <源> <目标>：把测试二进制拷成 <目标>（跨文件系统放不成硬链接时用）：objcopy --strip-debug 直接写出去掉调试段的一份，原件不动
#   （release 档带完整调试信息，调试段占二进制的九成以上；判器输入核对读的键表在 .rodata、不在调试段里）。objcopy 不在 PATH 上、退非 0、写出空文件时整份 cp。
#   交 0：去了调试段；交 3：整份拷（objcopy 用不上，原因打到标准错误）；交 1：拷也不成。弄坏开关 CRASH_AMPLIFICATION_LIB_BREAK=debug-info-kept-on-copy 下不去调试段、整份拷（交 3）。
crash_amplification_place_binary_without_debug_info() {
  local source="$1" destination="$2" strip_output
  if crash_amplification_lib_break_is debug-info-kept-on-copy; then
    :
  elif ! type -P objcopy >/dev/null; then
    echo "  ! objcopy 不在 PATH 上：$source 整份拷、不去调试段" >&2
  elif strip_output="$(objcopy --strip-debug "$source" "$destination" 2>&1)" && [[ -s "$destination" ]]; then
    return 0
  else
    echo "  ! objcopy --strip-debug 没成（${strip_output:-写出的是空文件}）：$source 整份拷、不去调试段" >&2
    rm -f -- "$destination"
  fi
  cp -- "$source" "$destination" || return 1
  return 3
}

# 本机的一趟：先只编、再跑，两步 cargo 直接起，编译与测试线程照服务交下来的（文件头「包装、内存上限与线程」；librocksdb-sys 的 C++ 按并行度吃内存，
# 由 task-server 给这一整条的上限兜着）。弄坏开关 this-host-service-threads-dropped 下两步起之前清掉 CARGO_BUILD_JOBS、RUST_TEST_THREADS。
# 要这一趟中途停下：附加环境里给 SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=<文件>，建出那个文件它就不再领新的块、手里那一块判完写进 prune-server、
# 打一行汇总退 0；判过的块在服务里，下一趟接着判。
#
# 被 TERM 停（文件头「被停」）：编与跑两步各放后台起（起法见 crash_amplification_run_step）、进程号记下、wait 它；收到 TERM 时 trap 当场就跑——
# 只把 TERM 转给记下的那一个进程号（env exec 成的 cargo）、不再起下一步、等记下的那个进程号退出、
# 放回原来的 TERM 处理，
# 再对自己发 TERM：调用方没设 TERM 处理的（放后台的子 shell）就此按被停退 143，设了的（分项驱动与双机驱动的主线：trap 'exit 143' TERM）它的处理跑过之后交回 143。
# 弄坏开关 CRASH_AMPLIFICATION_LIB_BREAK（文件头「弄坏开关」）：term-not-forwarded、term-not-waited、term-trap-not-restored、rocksdb-entry-unchecked、judge-binary-unused。
crash_amplification_lib_break_is() { [[ ",${CRASH_AMPLIFICATION_LIB_BREAK:-}," == *",$1,"* ]]; }

crash_amplification_process_is_running() { # <进程号>：在跑（没有、或已是僵尸都算没在跑）交回 0
  local process_status=""
  { read -r process_status <"/proc/$1/stat"; } 2>/dev/null || return 1
  process_status="${process_status##*) }"   # 进程名可以带空格与括号：取最后一个「) 」之后，第一段是状态
  [[ "${process_status%% *}" != Z ]]
}

# 记下的进程号（crash_amplification_own_run_pid）是这个 shell 的子进程才发（收过之后进程号可能被别的进程重用），只给它这一个发 TERM；
# 它记进 crash_amplification_term_forwarded，等它退出时用
crash_amplification_forward_term_to_own_run() {
  local root="$crash_amplification_own_run_pid" process_status=""
  [[ -n "$root" ]] || return 0
  { read -r process_status <"/proc/$root/stat"; } 2>/dev/null || return 0
  process_status="${process_status##*) }"
  process_status="${process_status#* }"
  [[ "${process_status%% *}" == "$BASHPID" ]] || return 0
  kill -TERM "$root" 2>/dev/null
  crash_amplification_term_forwarded+=("$root")
  return 0
}

crash_amplification_on_term() { # TERM 的 trap：记一笔、打断正在等的 wait、转给记下的那一趟
  crash_amplification_term_received=1
  crash_amplification_term_interrupted=1
  crash_amplification_lib_break_is term-not-forwarded || crash_amplification_forward_term_to_own_run
}

# 起一步：放后台（子 shell 里先进仓根再 exec 那条命令，cd 不成就退 1、不起命令），进程号记在 crash_amplification_own_run_pid，wait 它；
# 标准输出与标准错误追加进 <日志>（调用方先清空）。交回那一步的退出码；wait 被 TERM 的 trap 打断就接着等（命令没退之前不放手）。
crash_amplification_run_step() { # <日志> <命令与参数…>
  local log="$1" step_status=0 reaped=""
  shift
  ( cd "$repository_root" && exec "$@" ) >>"$log" 2>&1 &
  crash_amplification_own_run_pid=$!
  # 记下进程号之前 TERM 就到了（trap 跑时没有可转的）：补发
  if (( crash_amplification_term_received )); then crash_amplification_forward_term_to_own_run; fi
  while :; do
    crash_amplification_term_interrupted=0
    if wait "$crash_amplification_own_run_pid"; then step_status=0; else step_status=$?; fi
    # 接着等时它已经收过（wait 退 127）就留上一次的码
    if [[ -n "$reaped" && "$step_status" == 127 ]]; then step_status="$reaped"; break; fi
    reaped="$step_status"
    (( crash_amplification_term_interrupted )) || break
    ! crash_amplification_lib_break_is term-not-waited || break
  done
  crash_amplification_own_run_pid=""
  return "$step_status"
}

crash_amplification_wait_until_the_forwarded_are_gone() { # 转过 TERM 的那个进程号退出交回 0；等满 CRASH_AMPLIFICATION_TERM_WAIT_SECONDS（默认 100，在统一任务服务停一次尝试的单元时给的 TimeoutStopSec 150 秒之内：task-server/task_server.py:277 的 UNIT_STOP_SECONDS）还有在跑的，打 ✗ 与出路、交回 1
  local limit="${CRASH_AMPLIFICATION_TERM_WAIT_SECONDS:-100}" ticks=0 pid alive=()
  [[ "$limit" =~ ^[1-9][0-9]*$ ]] || limit=100
  while :; do
    alive=()
    for pid in ${crash_amplification_term_forwarded[@]+"${crash_amplification_term_forwarded[@]}"}; do
      ! crash_amplification_process_is_running "$pid" || alive+=("$pid")
    done
    (( ${#alive[@]} == 0 )) && return 0
    if (( ticks >= limit * 10 )); then
      echo "  ✗ 收到 TERM 转给本机那一趟的 cargo 后等了 $limit 秒，进程号 ${alive[*]} 还在跑" >&2
      echo "  → 怎么办：看它为什么不退（ps -o pid,ppid,args -p ${alive[*]}）；确认是这一趟自己起的之后 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号> 停它，再重跑同一条命令接着判" >&2
      return 1
    fi
    sleep 0.1
    ticks=$((ticks + 1))
  done
}

crash_amplification_restore_term_trap() { # <起头记下的 trap -p TERM 输出，没有是空串>
  crash_amplification_lib_break_is term-trap-not-restored && return 0
  if [[ -n "$1" ]]; then eval "$1"; else trap - TERM; fi
}

crash_amplification_run_on_this_host() { # <share> <日志> [<附加环境 键=值>…]
  local share="$1" log="$2"
  shift 2
  # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：起这一趟的两步 cargo 之前判 repository_root 那棵树这台装了、此刻进程环境的取值对得上，
  #   判红交回 78（entry 的红句写进 <日志>、也转打到标准错误），两步都不起。崩溃放量分项驱动与双机驱动本机那几趟都经这里起 cargo。
  #   树里没有依赖 rocksdb 的清单（自证的假树）、PATH 上是带标记的假 cargo，entry 报不判。弄坏开关 rocksdb-entry-unchecked 下不判（自证 rocksdb-entry-refused 判错）
  if ! crash_amplification_lib_break_is rocksdb-entry-unchecked \
     && ! python3 "$(dirname "${BASH_SOURCE[0]}")/../hosts/rocksdb-prebuilt.py" entry --root "$repository_root" >"$log" 2>&1; then
    cat -- "$log" >&2
    return 78
  fi
  crash_amplification_environment_for_this_host "$share" "$@"
  local local_environment=("${this_host_environment[@]}")
  crash_amplification_item_break_is this-host-service-threads-dropped && local_environment=(-u CARGO_BUILD_JOBS -u RUST_TEST_THREADS "${local_environment[@]}")
  local step_status=0 previous_term_trap=""
  crash_amplification_term_received=0; crash_amplification_term_interrupted=0; crash_amplification_own_run_pid=""; crash_amplification_term_forwarded=()
  # 原来的 TERM 处理：放后台的子 shell（BASHPID 不是 $$）里 trap -p 显示的是父 shell 的，不是它自己的，那里原来就是默认
  if [[ "$BASHPID" == "$$" ]]; then previous_term_trap="$(trap -p TERM)"; fi
  : >"$log"
  trap 'crash_amplification_on_term' TERM
  if [[ -n "${crash_amplification_judge_binary:-}" && "${crash_amplification_this_host_build_only:-0}" != 1 ]] && ! crash_amplification_lib_break_is judge-binary-unused; then
    # 判段之前编好的测试二进制（文件头「先编后判」）：直接起它，不经 cargo；libtest 参数取 cargo_test_arguments 里 -- 之后那几个
    local word after_separator=0 package_directory="$repository_root/crates/singlefs-verification"
    local -a libtest_arguments=()
    for word in "${cargo_test_arguments[@]}"; do
      if (( after_separator )); then libtest_arguments+=("$word"); elif [[ "$word" == -- ]]; then after_separator=1; fi
    done
    crash_amplification_run_step "$log" env -C "$package_directory" "${local_environment[@]}" \
      "CARGO_MANIFEST_DIR=$package_directory" CARGO_PKG_NAME=singlefs-verification "CARGO_CRATE_NAME=${test_target:-}" CARGO_PRIMARY_PACKAGE=1 "CARGO=$(crash_amplification_cargo_path "$repository_root")" \
      "$crash_amplification_judge_binary" ${libtest_arguments[@]+"${libtest_arguments[@]}"}; step_status=$?
  else
    crash_amplification_run_step "$log" env "${local_environment[@]}" cargo "${cargo_build_arguments[@]}"; step_status=$?
    if (( step_status == 0 && ! crash_amplification_term_received )) && [[ "${crash_amplification_this_host_build_only:-0}" != 1 ]]; then
      crash_amplification_run_step "$log" env "${local_environment[@]}" cargo "${cargo_test_arguments[@]}"; step_status=$?
    fi
  fi
  if (( crash_amplification_term_received )); then
    echo "  · 收到 TERM：已转给本机那一趟（记下的进程号），等它退出后按被停退 143" >&2
    crash_amplification_lib_break_is term-not-waited || crash_amplification_wait_until_the_forwarded_are_gone || true
    crash_amplification_restore_term_trap "$previous_term_trap"
    kill -TERM "$BASHPID"   # 调用方没设 TERM 处理就在这里按被停退；设了的，它的处理跑过之后走下面的交回
    return 143
  fi
  crash_amplification_restore_term_trap "$previous_term_trap"
  return "$step_status"
}

# libtest 先打「test <名> ... 」、测试跑完才补结局，等着的时候别的测试打的行就接在这一截后面（并行、--nocapture），记号不在行首：
# 判行首记号的地方都先去掉这一截（可连着几截）。弄坏开关 test-line-markers-unread 下不去掉，红在这份文件 --selftest 的 test-line-markers。
crash_amplification_lines_without_test_prefix() { # <日志…>：打每份日志的每一行，行首的「test <名> ... 」去掉
  if crash_amplification_lib_break_is test-line-markers-unread; then
    cat -- "$@"
    return
  fi
  LC_ALL=C sed -E 's/^(test [^ ]+ \.\.\. )+//' -- "$@"
}

# 打录入段记的那几行（行首的「test <名> ... 」去掉），一行都没有退 1。两种录入行与开关 library-marker-not-accepted 写在 crash_amplification_record_on_this_host 上面那一段。
crash_amplification_recorded_lines() { # <日志>
  local recorded_line_pattern='^(CRASH_AMPLIFICATION_RECORDED|CRASH_LIBRARY_RECORDED) '
  if crash_amplification_item_break_is library-marker-not-accepted; then
    recorded_line_pattern='^CRASH_AMPLIFICATION_RECORDED '
  fi
  grep -E "$recorded_line_pattern" < <(crash_amplification_lines_without_test_prefix "$1")
}

# 录入段（录入段在 CPU 上跑数据、写进判定库（prune-server）；判段只从服务里读，段与段之间不在内存里交数据）：本机起一趟
# SINGLEFS_CRASH_AMPLIFICATION_STAGE=record，日志写 <日志>；跑成而且打了录入行交回 0 并把那几行转打出来。录入行两种都算：崩溃放量的分项打
# CRASH_AMPLIFICATION_RECORDED（crates/singlefs-verification/src/crash_amplification.rs），崩溃枚举用例经判定库两段跑的（崩溃点重放双机那几条）打
# CRASH_LIBRARY_RECORDED（crates/singlefs-verification/src/crash_library.rs 的 record_the_item）；弄坏开关 library-marker-not-accepted 只认前一种，
# 由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红。
# 之后的每一趟照 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge 起（调用方 export）。退了非 0 交回照日志分类之后该退的码；退 0 而一行都没打交回 1。
# 份：不给是 0/1（整份）。引擎录入段带 SHARE=k/n 时，清单、流程、事实表与判表（配了 GPU）照旧整条录（录入键不含份，crates/singlefs-verification/src/crash_state_enumeration.rs
#   的 recording_inputs_digest），只有观察者的块按份筛（record_the_observers_of_the_flow 的 is_mine，与判段同一个 blocks_of_this_host：份加
#   SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS、按块序号平滑轮转）。所以两片观察者录入各录各的那一份时，钉在本机那一片的录入段给 0/2（双机驱动
#   crash-amplification-two-hosts.sh 的两片：不带权重、等分），录入段与判段的分法才是同一个；判段读到录入段没录的观察者块会报「先跑录入段」。
crash_amplification_record_on_this_host() { # <日志> [<份> [<附加环境 键=值>…]]
  local log="$1" share="${2:-0/1}" record_exit exit_code
  shift $(( $# < 2 ? $# : 2 ))
  crash_amplification_run_on_this_host "$share" "$log" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "$@"; record_exit=$?
  if [[ "$record_exit" != 0 ]]; then
    exit_code="$(crash_amplification_exit_code_of_a_failed_run "$record_exit" "$log")"
    echo "  ✗ 录入段（SINGLEFS_CRASH_AMPLIFICATION_STAGE=record）退 $record_exit；日志 $log" >&2
    echo "  → 怎么办：看日志里 panicked 那几行，修好再重跑同一条命令；录入段没跑成，判段一趟都不起" >&2
    return "$exit_code"
  fi
  if ! crash_amplification_recorded_lines "$log"; then
    echo "  ✗ 录入段退 0，日志 $log 里却一行 CRASH_AMPLIFICATION_RECORDED 或 CRASH_LIBRARY_RECORDED 都没有（引擎没跑到录入段，或用例没经 run_from_environment 起）" >&2
    echo "  → 怎么办：看那份日志跑的是不是这一项的用例、用例有没有在只跑录入段的那一趟提早返回" >&2
    return 1
  fi
  return 0
}

# 日志里 CRASH_AMPLIFICATION_PLACEMENT 行某个字段的合计（一条用例里几条流各打一行）；一行都没有打空串。
crash_amplification_placement_total() { # <日志> <字段名>
  python3 - "$1" "$2" <<'PYTHON'
import re, sys
log, field = sys.argv[1], sys.argv[2]
total, lines = 0, 0
for line in open(log, encoding="utf-8", errors="replace"):
    if not line.startswith("CRASH_AMPLIFICATION_PLACEMENT "):
        continue
    lines += 1
    found = re.search(r"(?:^| )" + re.escape(field) + r"=([0-9]+)(?: |$)", line.strip())
    if found:
        total += int(found.group(1))
if lines:
    print(total)
PYTHON
}

crash_amplification_item_break_is() { [[ ",${CRASH_AMPLIFICATION_ITEM_BREAK:-}," == *",$1,"* ]]; }

# 被测源码的摘要（文件头「源码摘要」）。树根下 Cargo.toml、Cargo.lock、.cargo/、crates/ 里每个普通文件（不进 .git、target、target-*、__pycache__，
#   不跟符号链接；与 scripts/hosts/source-stamp.py 的 DEFAULT_INCLUDES、skipped_directory 同一批），相对路径与内容的 sha256 排好序再取 sha256。
#   打一行「<摘要> files=<文件数>」；读不了、一个文件都没有交回 1，原因打标准错误
crash_amplification_source_digest() { # <树根>
  python3 -I - "$1" <<'SOURCE_DIGEST_PYTHON'
import hashlib
import os
import sys

root = sys.argv[1]
includes = ("Cargo.toml", "Cargo.lock", ".cargo", "crates")


def skipped(name):
    return name in (".git", "__pycache__") or name == "target" or name.startswith("target-")


entries = []
try:
    for include in includes:
        path = os.path.join(root, include)
        if os.path.isfile(path) and not os.path.islink(path):
            entries.append(include)
            continue
        if not os.path.isdir(path) or os.path.islink(path):
            continue
        for directory, subdirectories, files in os.walk(path):
            subdirectories[:] = [name for name in subdirectories if not skipped(name)]
            for name in files:
                full = os.path.join(directory, name)
                if os.path.isfile(full) and not os.path.islink(full):
                    entries.append(os.path.relpath(full, root))
    if not entries:
        print(f"树根 {root} 下 Cargo.toml、Cargo.lock、.cargo/、crates/ 里一个源文件都没有", file=sys.stderr)
        sys.exit(1)
    total = hashlib.sha256()
    for relative in sorted(entries):
        with open(os.path.join(root, relative), "rb") as handle:
            total.update(hashlib.sha256(handle.read()).hexdigest().encode() + b"  " + relative.encode() + b"\n")
except OSError as error:
    print(f"读不了树根 {root} 下的源文件：{error}", file=sys.stderr)
    sys.exit(1)
print(f"{total.hexdigest()} files={len(entries)}")
SOURCE_DIGEST_PYTHON
}

# 显卡先后一项的写法（文件头「这一台与另一台」）：拒绝说明里照抄这一句
CRASH_AMPLIFICATION_CARD_ENTRY_HOW_TO_WRITE='每项写 <HOSTS 里的地址>:<卡号>:<额度>（从右切两刀，地址可以是 IPv6；额度是正整数带 G 或 M，例 192.0.2.1:0:24G）'

# 多机配置判法判不过的那一句里有这几个字，才是认不出这一台（那一份认主机数组里哪一项是自己的三句 refuse：读不出、HOSTS 里一项都没有、有几项，
#   都写「这一台网卡上的地址」；别的 refuse 不写这几个字）。那一份改了这三句的说法，这里与本份 --selftest 的 emit-addresses-host-absent-named 跟着改
CRASH_AMPLIFICATION_HOST_UNRECOGNIZED_MARK='这一台网卡上的地址'

# 这一台与另一台的地址（文件头「这一台与另一台」）：只照 crash-point-replay-shard-configuration-check.sh --emit-addresses 认。
# 置 crash_amplification_hosts、crash_amplification_this_host_address、crash_amplification_peer_host_address 与 crash_amplification_serial_compile_hosts
#   （多机配置的 GPU_KERNEL_SERIAL_COMPILE_HOSTS，没写是空串；那一份文件头「GPU 内核一个一个编的台」）；那一份判不过交回 1，标准错误先打一行 ✗：
#   照它的退出码与它打的头一行「✗ …」原样报（那一句说的是这一台网卡上的地址才冠「认不出这一台是 HOSTS 里哪一项」，别的不冠），再打出路，再把它的原话缩进转出。
# 第二个参数是 1 时（调用方的弄坏开关 this-host-from-hosts-order）照那一份取 HOSTS，这一台一律当头一项、另一台当第二项（写死次序的样子）
crash_amplification_host_addresses() { # <多机配置> [<1：照 HOSTS 次序认>]
  local output line value emit_exit=0 refusal="" advice="" headline
  crash_amplification_hosts="" crash_amplification_this_host_address="" crash_amplification_peer_host_address="" crash_amplification_serial_compile_hosts=""
  output="$(bash "$(dirname "${BASH_SOURCE[0]}")/crash-point-replay-shard-configuration-check.sh" --configuration "$1" --emit-addresses 2>&1)" || emit_exit=$?
  if (( emit_exit != 0 )); then
    while IFS= read -r line; do
      if [[ -z "$refusal" && "$line" =~ ^[[:space:]]*✗[[:space:]]*(.*)$ ]]; then refusal="${BASH_REMATCH[1]}"
      elif [[ -z "$advice" && "$line" =~ ^[[:space:]]*→[[:space:]]*(.*)$ ]]; then advice="${BASH_REMATCH[1]}"
      fi
    done <<<"$output"
    headline="多机配置判法 --emit-addresses 退 $emit_exit：${refusal:-（它没打「✗」那一行，原话见下）}"
    if crash_amplification_lib_break_is emit-failure-said-host-unrecognized \
       || { [[ "$refusal" == *"$CRASH_AMPLIFICATION_HOST_UNRECOGNIZED_MARK"* ]] && ! crash_amplification_lib_break_is host-unrecognized-unsaid; }; then
      headline="认不出这一台是 HOSTS 里哪一项：$headline"
    fi
    echo "  ✗ $headline（多机配置：$1）" >&2
    echo "  → 怎么办：${advice:+$advice；}照上面那一句改多机配置（或修好那一句点名的设备），单跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh --configuration $1 --emit-addresses 核" >&2
    printf '%s\n' "$output" | sed 's/^/       | /' >&2
    return 1
  fi
  while IFS= read -r line; do
    [[ "$line" =~ ^(HOSTS|THIS_HOST_ADDRESS|PEER_HOST_ADDRESS|GPU_KERNEL_SERIAL_COMPILE_HOSTS)=(.*)$ ]] || continue
    eval "value=${BASH_REMATCH[2]}"   # 那一份照 printf %q 打的值
    case "${BASH_REMATCH[1]}" in
      HOSTS) crash_amplification_hosts="$value" ;;
      GPU_KERNEL_SERIAL_COMPILE_HOSTS) crash_amplification_serial_compile_hosts="$value" ;;
      THIS_HOST_ADDRESS) crash_amplification_this_host_address="$value" ;;
      PEER_HOST_ADDRESS) crash_amplification_peer_host_address="$value" ;;
    esac
  done <<<"$output"
  if [[ "${2:-}" == 1 ]]; then
    read -r crash_amplification_this_host_address crash_amplification_peer_host_address _ <<<"$crash_amplification_hosts"
  fi
  if [[ -z "$crash_amplification_this_host_address" || -z "$crash_amplification_peer_host_address" ]]; then
    echo "  ✗ 多机配置判法 --emit-addresses 退 0 却没打出这一台与另一台的地址（读到：$(tr '\n' ' ' <<<"$output" | crash_amplification_clip_text 200)）" >&2
    echo "  → 怎么办：看 scripts/crash/crash-point-replay-shard-configuration-check.sh 的 --emit-addresses 那一段（要打 HOSTS、THIS_HOST_ADDRESS、PEER_HOST_ADDRESS 三行）" >&2
    return 1
  fi
  return 0
}

# 显卡先后写得对不对（文件头「这一台与另一台」）：每一项从右切两刀成 <地址>:<卡号>:<额度>，卡号是十进制、额度是正整数带 G 或 M、地址是 HOSTS 里的一项。
# 都对交回 0、什么都不打；有一项写坏打一句原因（不带换行）、交回 1。第三个参数是 1 时（调用方的弄坏开关 card-address-outside-hosts-accepted）地址不在 HOSTS 也照认
crash_amplification_card_list_problem() { # <显卡先后的值> <HOSTS> [<1：地址不在 HOSTS 也认>]
  local entry rest card quota address index=0
  for entry in $1; do
    index=$((index + 1))
    quota="${entry##*:}" rest="${entry%:*}"
    card="${rest##*:}" address="${rest%:*}"
    if [[ "$entry" != *:*:* || ! "$card" =~ ^[0-9]{1,9}$ || ! "$quota" =~ ^[0-9]{1,15}[GM]$ ]] || (( 10#${quota%[GM]} == 0 )); then
      printf '显卡先后第 %s 项「%s」写坏了：%s' "$index" "$entry" "$CRASH_AMPLIFICATION_CARD_ENTRY_HOW_TO_WRITE"
      return 1
    fi
    [[ "${3:-}" == 1 ]] && continue
    if [[ " $2 " != *" $address "* ]]; then
      printf '显卡先后第 %s 项「%s」的地址「%s」不在 HOSTS（「%s」）里：%s' "$index" "$entry" "$address" "$2" "$CRASH_AMPLIFICATION_CARD_ENTRY_HOW_TO_WRITE"
      return 1
    fi
  done
  return 0
}

# 显卡先后里点名某一台的那几项（写法先经 crash_amplification_card_list_problem 核过）：打卡号（逗号隔开，按出现次序、去重）；一项都没有打 none
crash_amplification_cards_on_address() { # <显卡先后的值> <地址>
  local entry rest cards=","
  for entry in $1; do
    rest="${entry%:*}"
    [[ "${rest%:*}" == "$2" ]] || continue
    [[ "$cards" == *",$((10#${rest##*:})),"* ]] || cards+="$((10#${rest##*:})),"
  done
  cards="${cards#,}"; cards="${cards%,}"
  printf '%s' "${cards:-none}"
}

# 显卡配置与双机开关对不对得上（文件头「这一台与另一台」）：每一项写得对、地址在 HOSTS 里；双机开着两台都要有点名的卡，双机没开不许点名另一台的卡。
# 对得上交回 0，对不上打原因与出路、交回 1
crash_amplification_cards_match_the_hosts() { # <双机开关 0/1> <显卡配置的值> <这一台的地址> <另一台的地址> <HOSTS>
  local across="$1" cards="$2" this_host="$3" other_host="$4" problem outside_accepted=0
  crash_amplification_item_break_is card-address-outside-hosts-accepted && outside_accepted=1
  if ! problem="$(crash_amplification_card_list_problem "$cards" "$5" "$outside_accepted")"; then
    echo "  ✗ 配置开了 GPU，$problem" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 改多机配置的显卡先后（两个键 GPU_CARDS_BY_PRIORITY、GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 一样写）" >&2
    return 1
  fi
  if [[ "$across" == 1 ]] && [[ "$(crash_amplification_cards_on_address "$cards" "$this_host")" == none || "$(crash_amplification_cards_on_address "$cards" "$other_host")" == none ]] \
     && ! crash_amplification_item_break_is accept-missing-peer-cards; then
    echo "  ✗ 配置开了双机与 GPU，显卡先后「$cards」却没有两台（这一台 $this_host、另一台 $other_host）各点名至少一张卡" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 在多机配置里给两台各点名卡（<这一台的地址>:<卡号>:<额度> 与 <另一台的地址>:<卡号>:<额度>）；不想用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 0" >&2
    return 1
  fi
  if [[ "$across" != 1 ]] && [[ "$(crash_amplification_cards_on_address "$cards" "$other_host")" != none ]] && ! crash_amplification_item_break_is accept-peer-cards-without-across; then
    echo "  ✗ 多机配置没开双机，显卡先后「$cards」却点名了另一台（$other_host）的卡：点名的卡用不上" >&2
    echo "  → 怎么办：修好设备，或改配置：要用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 1；不用就从多机配置的显卡先后里删掉地址是 $other_host 的那几项" >&2
    return 1
  fi
  return 0
}

# 运行条件：配置里开了的硬件在配置上对不对得上。开关与显卡都读多机配置（文件头「配置」；CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机> 只供测试，
# 只在分项驱动的自证里认（CRASH_AMPLIFICATION_ITEM_SELFTEST=1），给了照它），显卡读两份先后（不清场的与清场之后的，这里不知道调用方清没清场，写了的每一份都要对得上；
# CRASH_AMPLIFICATION_ITEM_CARDS 只供测试、同样只在那份自证里认，给了只判它）。
crash_amplification_run_condition_holds() { # [<仓根>]：对得上交回 0；对不上打原因与出路、交回 1
  local root="${1:-$PWD}" configuration gpu=0 across=0 cards_file key list lists=()
  configuration="$(crash_amplification_multi_host_configuration_file "$root")"
  [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_GPU)" == 1 ]] && gpu=1
  [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_ACROSS_MACHINES)" == 1 ]] && across=1
  if [[ "${CRASH_AMPLIFICATION_ITEM_SELFTEST:-}" == 1 && -n "${CRASH_AMPLIFICATION_ITEM_SWITCHES:-}" ]]; then IFS=, read -r gpu across <<<"$CRASH_AMPLIFICATION_ITEM_SWITCHES"; fi
  [[ "$gpu" == 1 ]] || return 0
  cards_file="$configuration"
  if [[ "${CRASH_AMPLIFICATION_ITEM_SELFTEST:-}" == 1 && -n "${CRASH_AMPLIFICATION_ITEM_CARDS:-}" ]]; then
    lists=("$CRASH_AMPLIFICATION_ITEM_CARDS")
  else
    for key in GPU_CARDS_BY_PRIORITY GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED; do
      list="$(crash_amplification_configuration_value "$cards_file" "$key")"
      [[ -n "$list" ]] && lists+=("$list")
    done
  fi
  if (( ${#lists[@]} == 0 )) && ! crash_amplification_item_break_is accept-missing-card-list; then
    echo "  ✗ 配置开了 GPU（ENABLE_GPU=1），多机配置里 GPU_CARDS_BY_PRIORITY 与 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 一份都没写（多机配置：$cards_file）" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 在多机配置里写每台机器用哪几张卡、各多少显存；不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
    return 1
  fi
  # 这一台与另一台照多机配置判法 --emit-addresses 认（文件头「这一台与另一台」）；弄坏开关 this-host-from-hosts-order 下照 HOSTS 次序认
  local first_is_this=0
  crash_amplification_item_break_is this-host-from-hosts-order && first_is_this=1
  crash_amplification_host_addresses "$configuration" "$first_is_this" || return 1
  for list in ${lists[@]+"${lists[@]}"}; do
    crash_amplification_cards_match_the_hosts "$across" "$list" "$crash_amplification_this_host_address" "$crash_amplification_peer_host_address" "$crash_amplification_hosts" || return 1
  done
  return 0
}

# 一趟没跑成，日志里说的是哪一类（判据是用例与判器报错的原话，出处见各串旁边）：
#   observer-images-without-a-card：行首是 CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS 的那一行（观察者交回的盘要在卡上判、取不到卡；
#     CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS_LINE_PREFIX）。弄坏开关 CRASH_AMPLIFICATION_LIB_BREAK=observer-images-without-a-card-unrecognized 下不认
#   device：显卡或显存用不上——GPU_CARD_NOT_USABLE 行（crates/singlefs-verification/src/gpu_unit_checks.rs）、
#     「这台机器却没有一张用得了的卡」「没有一张卡装得下这条流的表」「GPU_CARDS_BY_PRIORITY 里没有」（crates/singlefs-verification/src/crash_amplification.rs）、
#     行首（可带空白）是 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 的那一行（给定状态表的判器起过卡池而借不到显存，引擎打这一行后报错；
#     CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX）
#   build：这份二进制跑不了这一趟——行首是 CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD 的那一行（结论的版本编译时与跑时对不上、
#     整趟不写库；crates/singlefs-verification/src/crash_amplification.rs 的 CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX）、
#     「这份二进制没有编进 gpu 特性」（多机配置开了 GPU 而编的不带 gpu 特性，crash_amplification.rs）
#   algorithm：算法上判不了——「表越过了内核的容量」（有状态 GPU 判不了）、「全部调用展开之后 <数> 个表达式，超过预算」（内核展开超预算，crash_judge_dispatch.rs）、
#     「GPU 判器建不出这条流的表」（crash_amplification.rs）
# 不在的日志跳过；几类都有时先 device、再 build（这两类时那一趟判的不算数）；都不是打空串。行首的两种记号都先去掉「test <名> ... 」再认（crash_amplification_lines_without_test_prefix）。
# 弄坏开关 version-differs-as-run-failure、gpu-feature-missing-as-run-failure：build 那两句各自不认（表与该红的格在 crash-amplification-item.sh 文件头）。
# 弄坏开关 given-states-run-condition-as-run-failure：给定状态表借不到显存那一行不认（红在 crash-amplification-two-hosts.sh --selftest 的 shard-given-states-run-condition）。
crash_amplification_failure_kind() { # <日志…>
  local observer_images_without_a_card_pattern="^[[:space:]]*${CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS_LINE_PREFIX}( |\$)"
  local device_pattern='GPU_CARD_NOT_USABLE|这台机器却没有一张用得了的卡|没有一张卡装得下这条流的表|GPU_CARDS_BY_PRIORITY 里没有'
  local given_states_run_condition_pattern="^[[:space:]]*${CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX}( |\$)"
  local version_differs_pattern='^CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD( |$)'
  local gpu_feature_missing_pattern='这份二进制没有编进 gpu 特性'
  local algorithm_pattern='表越过了内核的容量|全部调用展开之后 [0-9]+ 个表达式，超过预算|GPU 判器建不出这条流的表'
  local logs=() log
  for log in "$@"; do [[ -f "$log" ]] && logs+=("$log"); done
  (( ${#logs[@]} > 0 )) || return 0
  if ! crash_amplification_lib_break_is observer-images-without-a-card-unrecognized \
     && grep -qE -- "$observer_images_without_a_card_pattern" < <(crash_amplification_lines_without_test_prefix "${logs[@]}"); then echo observer-images-without-a-card; return 0; fi
  if ! crash_amplification_item_break_is device-unusable-as-run-failure && grep -qE -- "$device_pattern" "${logs[@]}"; then echo device; return 0; fi
  if ! crash_amplification_item_break_is given-states-run-condition-as-run-failure && grep -qE -- "$given_states_run_condition_pattern" < <(crash_amplification_lines_without_test_prefix "${logs[@]}"); then echo device; return 0; fi
  if ! crash_amplification_item_break_is version-differs-as-run-failure && grep -qE -- "$version_differs_pattern" < <(crash_amplification_lines_without_test_prefix "${logs[@]}"); then echo build; return 0; fi
  if ! crash_amplification_item_break_is gpu-feature-missing-as-run-failure && grep -qF -- "$gpu_feature_missing_pattern" "${logs[@]}"; then echo build; return 0; fi
  if ! crash_amplification_item_break_is algorithm-refusal-as-run-failure && grep -qE -- "$algorithm_pattern" "${logs[@]}"; then echo algorithm; return 0; fi
  return 0
}

crash_amplification_exit_code_of_a_failed_run() { # <那一趟的退出码> <日志…>：打调用方该退的码
  local run_exit="$1" kind
  shift
  kind="$(crash_amplification_failure_kind "$@")"
  case "$kind" in
    observer-images-without-a-card|device|build) echo 78 ;;
    algorithm) echo 248 ;;
    *) echo "$run_exit" ;;
  esac
}

# crash_amplification_observer_images_without_a_card_howto：观察者交回的盘取不到卡那一类（文件头「运行条件与失败分类」）的出路，打到标准输出（调用方定往哪儿打）
crash_amplification_observer_images_without_a_card_howto() {
  echo "这条用例的观察者交回了盘、要上卡判，任务不拿卡；等 GPU 批处理服务收这一族待判行（日志里 ${CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS_LINE_PREFIX} 那一行的 reason= 是取卡报的原因）。这一条记本次未跑（退 78），不是红"
}

crash_amplification_explain_exit_code() { # <调用方要退的码> [<日志…>]：78、248 与 245（内存包装认出的显存不够）打对应的出路（标准错误），别的不打
  local failure_kind=""
  if [[ "$1" == 78 ]] && (( $# > 1 )); then failure_kind="$(crash_amplification_failure_kind "${@:2}")"; fi
  if [[ "$failure_kind" == observer-images-without-a-card ]]; then
    echo "  → 怎么办：$(crash_amplification_observer_images_without_a_card_howto)" >&2
    return 0
  fi
  case "$1" in
    245) echo "  → 怎么办：显存不够（那一趟退了非 0，内存包装认出它起止之间内核报了 NV_ERR_NO_MEMORY，退 245），这一趟的结果不算数：nvidia-smi 看谁在用卡（同一段时间别的进程撞的也算进来），等它跑完、或少给这一趟显存，再重跑同一条命令接着判（判过的块已经在库里）" >&2 ;;
    78) echo "  → 怎么办：修好设备，或改配置：日志里 GPU_CARD_NOT_USABLE 行点名的卡用不了（没有这个序号、空闲显存不够、上下文起不来）、显存装不下这条流的表、给定状态表的判器借不到显存（CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 那一行，等卡上占着显存的跑完再跑），或第二台连不上；查那张卡或那台机器，或改显卡配置与多机配置；日志里有 CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD 那一行的，是二进制与盘上的树对不上、整趟没写库，重编再跑（或在提交那一趟的 worktree 里跑）；有「这份二进制没有编进 gpu 特性」的，带 --features gpu 重编，或把多机配置的 ENABLE_GPU 改成 0（退 78：运行条件不满足）" >&2 ;;
    248) echo "  → 怎么办：算法上判不了（有状态 GPU 判不了、内核展开超预算、建不出表），不是设备的事、也不是红：照日志里 panicked 那几行改判器的表或内核的容量再跑（退 248）" >&2 ;;
  esac
}

# 弄坏开关轮（文件头「弄坏开关轮」一段）
CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT="crash_amplification_switch_wheel 起的子进程"

crash_amplification_read_switch_table() { # <脚本文件> <变量名…>：读文件头的开关表（文件头「弄坏开关轮」一段的写法），一个开关打一行「<变量>=<开关名> <TAB> <点名的格>」
  python3 - "$@" <<'PYTHON'
import re, sys
variables = "|".join(re.escape(name) for name in sys.argv[2:])
start = re.compile(r"^#   (" + variables + r")=([a-z0-9-]+)(\s|$)")
rows, current = [], None
with open(sys.argv[1], encoding="utf-8") as handle:
    for line in handle:
        line = line.rstrip("\n")
        if not line.startswith("#"):
            break
        found = start.match(line)
        if found:
            current = [found.group(1) + "=" + found.group(2), ""]
            rows.append(current)
        elif current is not None and line.startswith("#       "):
            current[1] += " " + line[1:]
        else:
            current = None
for assignment, text in rows:
    previous = None
    while previous != text:
        previous = text
        text = re.sub(r"（[^（）]*）", " ", text)
        text = re.sub(r"\([^()]*\)", " ", text)
    text = re.sub(r"第\s*([0-9]+)\s*格", r" \1 ", text)
    print(assignment + "\t" + " ".join(part for part in re.split(r"[\s、，,；;]+", text) if part))
PYTHON
}

crash_amplification_switch_wheel_inner() { # 这一趟是不是开关轮起的子进程：标记文件在、内容对得上交回 0
  [[ -n "${CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER:-}" && -f "${CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER:-}" ]] \
    && [[ "$(cat -- "$CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER")" == "$CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT" ]]
}

crash_amplification_switch_wheel_verdict() { # <点名的格> <退出码> <输出文件> <取格的正则> <只比退出码 0/1>：打实测红的格，对得上交回 0
  local expected="$1" exit_code="$2" output="$3" pattern="$4" exit_only="$5" observed="" wanted=""
  if [[ -f "$output" ]]; then
    observed="$(sed -nE "s/.*${pattern}.*/\\1/p" -- "$output" | LC_ALL=C sort -V | uniq | tr '\n' ' ')"
  fi
  printf '%s' "${observed% }"
  if [[ "$expected" == - ]]; then
    [[ "$exit_code" == 0 ]] && { [[ "$exit_only" == 1 ]] || [[ -z "$observed" ]]; }
    return
  fi
  wanted="$(tr -s ' \t' '\n\n' <<<"$expected" | sed '/^$/d' | LC_ALL=C sort -V | uniq | tr '\n' ' ')"
  [[ -n "$wanted" && "$exit_code" == 1 ]] || return 1
  [[ "$exit_only" == 1 || "$observed" == "$wanted" ]]
}

crash_amplification_switch_wheel() { # <放输出的目录> <取格的正则> <只比退出码 0/1> -- <自证命令…>（两个数组与交回的变量见文件头）
  local directory="$1" pattern="$2" exit_only="$3" jobs batch index count assignment variable observed shown
  local variables=" " unset_names=() pids=() exits=()
  shift 3
  [[ "${1:-}" == -- ]] && shift
  switch_wheel_failures=0; switch_wheel_collected=0
  count="${#switch_wheel_assignments[@]}"
  jobs="${CRASH_AMPLIFICATION_SELFTEST_JOBS:-$(heavy_thread_default)}"
  if [[ ! "$jobs" =~ ^[1-9][0-9]*$ ]]; then   # 取不出就拒，不退回 1（.claude/heavy-thread-default 文件头：取不了就拒）
    echo "  ✗ 开关轮的并行度取不出（CRASH_AMPLIFICATION_SELFTEST_JOBS 或每台默认重活线程数给的是「$jobs」），一个开关都没起" >&2
    echo "     → 怎么办：CRASH_AMPLIFICATION_SELFTEST_JOBS 写成正整数或不设；不设时照 .claude/heavy-thread-default 文件头写好登记再跑" >&2
    switch_wheel_failures=$((count > 0 ? count : 1))
    return 0
  fi
  for assignment in ${switch_wheel_assignments[@]+"${switch_wheel_assignments[@]}"}; do
    variable="${assignment%%=*}"
    [[ "$variables" == *" $variable "* ]] && continue
    variables+="$variable "; unset_names+=("$variable")
  done
  printf '%s\n' "$CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT" >"$directory/switch-wheel-mark"
  for ((batch = 0; batch < count; batch += jobs)); do
    for ((index = batch; index < batch + jobs && index < count; index++)); do
      # 子 shell 里清变量、设这一个开关再 exec 自证命令（不写成 env "${数组[@]}" …：函数单元键折不开数组里的 -u 与赋值，整个闭包拒算）
      (
        unset ${unset_names[@]+"${unset_names[@]}"}
        export "${switch_wheel_assignments[index]}" CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER="$directory/switch-wheel-mark"
        exec "$@"
      ) >"$directory/switch-wheel-$index.out" 2>&1 &
      pids[index]=$!
    done
    for ((index = batch; index < batch + jobs && index < count; index++)); do
      wait "${pids[index]}"; exits[index]=$?
    done
  done
  for ((index = 0; index < count; index++)); do
    [[ -n "${exits[index]:-}" ]] && switch_wheel_collected=$((switch_wheel_collected + 1))
    if observed="$(crash_amplification_switch_wheel_verdict "${switch_wheel_expected[index]:-}" "${exits[index]:-}" "$directory/switch-wheel-$index.out" "$pattern" "$exit_only")"; then
      shown="${observed// /、}"
      echo "  · 开关轮：${switch_wheel_assignments[index]} 红在「${shown:-（一格不红，表里没列它）}」，与文件头相同"
    else
      shown="${switch_wheel_expected[index]:-}"
      [[ "$shown" == - ]] && shown="（表里没列它：应当退 0、一格不红）"
      echo "  ✗ 开关轮：${switch_wheel_assignments[index]} 打开后 --selftest 退 ${exits[index]:-（没收到）}、红在「${observed:-（没有）}」，文件头点名的是「${shown:-（没写）}」" >&2 # gate-lint:detail
      switch_wheel_failures=$((switch_wheel_failures + 1))
    fi
  done
  if (( switch_wheel_collected != count )); then
    echo "  ✗ 开关轮：派出 $count 个开关、收回 $switch_wheel_collected 个" >&2 # gate-lint:detail
    switch_wheel_failures=$((switch_wheel_failures + 1))
  fi
  return 0
}

# 自证（文件头「自证」）：单独执行带 --selftest 才跑，被 source 时不跑。调真的 crash_amplification_run_on_this_host，
# 仓根换成格目录下的假树，cargo 是 PATH 前面的一个假脚本
# （跑着时占着一个文件、收到 TERM 慢一点放掉、再退：判它在那一趟返回之前退干净了）。失败行都是「✗ 自证 <格名>：<细节>」，开关轮靠它认红的格。
crash_amplification_lib_selftest() {
  local lib_script work driver_text failures=0 cells=0 cell_exit=0 cell_directory="" service_problem
  lib_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  # 只在临时 prune-server 上跑（文件头「自证」），不是就退 2、一格都不跑
  service_problem="$(crash_amplification_selftest_service_problem)"
  if [[ -n "$service_problem" ]]; then
    echo "  ✗ crash-amplification-lib.sh 自证没跑：$service_problem" >&2
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）" >&2
    return 2
  fi
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-lib-selftest.XXXXXX")" || {
    echo "  ✗ --selftest：建不了临时目录（TMPDIR=${TMPDIR:-/tmp}）" >&2
    echo "  → 怎么办：看 TMPDIR 可不可写" >&2
    return 2
  }
  trap "rm -rf -- '$work'" EXIT   # 现在展开：函数返回之后局部变量 work 已经没了
  # 外壳的正文放进变量、run_cell 用 bash -c 起（不写进文件再 bash 那份文件：写出来的脚本里 source "$2" 折不开，函数单元键拒算；
  # bash -c 的串之后的词照 $0 $1 … 折进去）
  driver_text="$(cat <<'DRIVER'
#!/usr/bin/env bash
# 自证里调真的 crash_amplification_run_on_this_host 的外壳：$1 仓根（假树）$2 共用库 $3 格目录 $4 形态（foreground 在本 shell 里调、background 放后台的子 shell 里调）
repository_root="$1"; source "$2" || exit 3
cell="$3"; shape="$4"
common_environment=(); gpu_cards_by_priority=""
cargo_build_arguments=(test --no-run); cargo_test_arguments=(test)
export PATH="$cell/bin:$PATH"
lock_state() { if [[ -e "$STUB_LOCK" ]]; then echo present; else echo absent; fi; }
if [[ "${STUB_CALLER_TRAP:-0}" == 1 ]]; then trap 'echo caller-term-trap >>"$STUB_CALLS"' TERM; fi
# 形态 judge-binary（格 judge-binary-built-before-the-judge-runs）：先编判段要的测试二进制（只编的那一步），记一笔判段开始，再起两趟本机那一趟（排计划与本机判）
if [[ "$shape" == judge-binary ]]; then
  test_target=fake_target; cargo_test_arguments=(test --test fake_target -- --nocapture)
  if crash_amplification_build_judge_binary "$cell/build.log"; then build_exit=0; else build_exit=$?; fi
  echo "judge-runs-begin" >>"$STUB_CALLS"
  for run_name in placement run; do
    if crash_amplification_run_on_this_host 0/1 "$cell/$run_name.log"; then run_exit=0; else run_exit=$?; fi
    echo "run-returned rc=$run_exit lock=$(lock_state)" >>"$STUB_CALLS"
  done
  echo "build-returned rc=$build_exit binary=[$crash_amplification_judge_binary]" >>"$STUB_CALLS"
  exit 0
fi
if [[ "$shape" == background ]]; then
  crash_amplification_run_on_this_host 0/1 "$cell/run.log" &
  run_pid=$!
  echo "$run_pid" >"$cell/run-pid.partial"; mv -- "$cell/run-pid.partial" "$cell/run-pid"
  if wait "$run_pid"; then run_exit=0; else run_exit=$?; fi
  echo "run-returned rc=$run_exit lock=$(lock_state)" >>"$STUB_CALLS"
  exit 0
fi
if crash_amplification_run_on_this_host 0/1 "$cell/run.log"; then run_exit=0; else run_exit=$?; fi
echo "run-returned rc=$run_exit lock=$(lock_state)" >>"$STUB_CALLS"
trap -p TERM >"$cell/trap-after"
exit "$run_exit"
DRIVER
)"
  cat >"$work/cargo" <<'CARGO'
#!/usr/bin/env bash
step=test; [[ " $* " == *" --no-run "* ]] && step=build
echo "cargo-$step started" >>"${STUB_CALLS:?}"
echo "out-$step"; echo "err-$step" >&2
: >"${STUB_LOCK:?}"
release() { sleep 0.3; rm -f -- "$STUB_LOCK"; echo "cargo-$step lock-released" >>"$STUB_CALLS"; }
trap 'echo "cargo-$step got-TERM" >>"$STUB_CALLS"; release; exit 143' TERM
: >"${STUB_DIR:?}/started.$step"
# 给了 STUB_BINARY 的（格 judge-binary-built-before-the-judge-runs）：编的那一步在那里写一份假的测试二进制（被起时往调用记录记一笔工作目录、参数与 CARGO_MANIFEST_DIR 的末一段），打 cargo 那种 Executable 行
if [[ "$step" == build && -n "${STUB_BINARY:-}" ]]; then
  printf '%s\n' '#!/usr/bin/env bash' 'echo "binary-run directory=${PWD##*/} arguments=$* manifest=${CARGO_MANIFEST_DIR##*/}" >>"$STUB_CALLS"' >"$STUB_BINARY"
  chmod +x "$STUB_BINARY"
  echo "     Executable tests/fake_target.rs ($STUB_BINARY)"
fi
tick=0
if [[ "$step" == "${STUB_HOLD_STEP:-}" ]]; then while (( tick < 50 )); do sleep 0.1; tick=$((tick + 1)); done; fi
echo "cargo-$step done" >>"$STUB_CALLS"
release
exit_variable="STUB_${step^^}_EXIT"
exit "${!exit_variable:-0}"
CARGO
  run_cell() { # <格名> <形态 foreground|background> <给 TERM 的时机 none|build|test> [<环境 键=值>…]：交回值放 cell_exit，调用记录放 <格目录>/calls
    local name="$1" shape="$2" term_at="$3" waited=0 driver_pid hold_step="" target=""
    shift 3
    [[ "$term_at" == none ]] || hold_step="$term_at"
    cell_directory="$work/$name"
    mkdir -p "$cell_directory/root" "$cell_directory/bin"
    : >"$cell_directory/calls"
    cells=$((cells + 1))
    cp -- "$work/cargo" "$cell_directory/bin/cargo"; chmod +x "$cell_directory/bin/cargo"
    env -u CRASH_AMPLIFICATION_TERM_WAIT_SECONDS STUB_CALLS="$cell_directory/calls" STUB_LOCK="$cell_directory/cargo-running" STUB_DIR="$cell_directory" STUB_HOLD_STEP="$hold_step" \
      STUB_CALLER_TRAP=0 STUB_BUILD_EXIT=0 STUB_TEST_EXIT=0 "$@" bash -c "$driver_text" driver "$cell_directory/root" "$lib_script" "$cell_directory" "$shape" >"$cell_directory/output" 2>&1 &
    driver_pid=$!
    if [[ "$term_at" != none ]]; then
      while (( waited < 100 )) && { [[ ! -e "$cell_directory/started.$term_at" ]] || { [[ "$shape" == background && ! -e "$cell_directory/run-pid" ]]; }; }; do
        sleep 0.1; waited=$((waited + 1))
      done
      target="$driver_pid"
      [[ "$shape" == background ]] && target="$(cat -- "$cell_directory/run-pid" 2>/dev/null)"
      [[ "$target" =~ ^[0-9]+$ ]] && kill -TERM "$target" 2>/dev/null
    fi
    if wait "$driver_pid"; then cell_exit=0; else cell_exit=$?; fi
    echo "driver-exited rc=$cell_exit lock=$([[ -e "$cell_directory/cargo-running" ]] && echo present || echo absent)" >>"$cell_directory/calls"
    # 被停的格：驱动退了之后假 cargo 的 TERM 处理（记 got-TERM、放锁）可能还在跑；等它放掉（至多 8 秒）再判，判的是它收没收到 TERM，不是赶不赶得上驱动退出
    if [[ "$term_at" != none ]]; then
      for ((waited = 0; waited < 80; waited++)); do
        [[ -e "$cell_directory/cargo-running" ]] || break
        sleep 0.1
      done
    fi
  }
  cell_failed() { # <格名> <判错的一句> <出路>
    echo "  ✗ 自证 $1：$2（退 $cell_exit；调用记录「$(tr '\n' ';' <"$work/$1/calls")」）" >&2
    echo "     → 怎么办：$3" >&2
    tail -n 8 "$work/$1/output" | sed 's/^/       | /' >&2
    failures=$((failures + 1))
  }
  calls_have() { grep -qE -- "$2" "$work/$1/calls"; }   # <格名> <正则>
  line_of() { grep -nE -m1 -- "$2" "$work/$1/calls" | cut -d: -f1; }   # <格名> <正则>：第一次出现的行号，没有是空串
  before() { local first second; first="$(line_of "$1" "$2")"; second="$(line_of "$1" "$3")"; [[ -n "$first" && -n "$second" ]] && (( first < second )); }   # <格名> <正则> <正则>：前者先出现

  # 被 TERM 停（C797）：放后台的子 shell 里，跑的那一步途中给它 TERM
  run_cell term-reaches-cargo background test
  if [[ "$cell_exit" != 0 ]] || ! calls_have term-reaches-cargo '^cargo-test got-TERM$' || calls_have term-reaches-cargo '^cargo-test done$' \
     || ! before term-reaches-cargo '^cargo-test lock-released$' '^run-returned rc=143 lock=absent$' || ! calls_have term-reaches-cargo '^driver-exited rc=0 lock=absent$'; then
    cell_failed term-reaches-cargo "跑 cargo test 那一步时给本机那一趟（放后台的子 shell）TERM：假 cargo 应当收到 TERM、没跑到底，在那一趟返回之前退干净，那一趟退 143" \
      "看 crash_amplification_run_on_this_host 与 crash_amplification_on_term：TERM 要转给记下的那一个进程号（env exec 成的 cargo）、等它退出、放回原来的 TERM 处理，再对自己发 TERM"
  fi
  # 同样的事发生在调用方自己的 shell 里（崩溃放量分项驱动的主线）：没设 TERM 处理，假 cargo 退干净之后自己按被停退 143
  run_cell term-reaches-cargo-in-foreground foreground test
  if [[ "$cell_exit" != 143 ]] || ! calls_have term-reaches-cargo-in-foreground '^cargo-test got-TERM$' || calls_have term-reaches-cargo-in-foreground '^cargo-test done$' \
     || ! before term-reaches-cargo-in-foreground '^cargo-test lock-released$' '^driver-exited rc=143 lock=absent$'; then
    cell_failed term-reaches-cargo-in-foreground "跑 cargo test 那一步时给调用方的 shell（没设 TERM 处理）TERM：假 cargo 应当收到 TERM、在调用方退出之前退干净，调用方退 143" \
      "看 crash_amplification_run_on_this_host 收尾：放回原来的 TERM 处理（没有就是默认）之后对自己发 TERM"
  fi
  # 调用方设了 TERM 处理（分项驱动与双机驱动的主线）：它的处理在假 cargo 退干净之后跑一次，那一趟交回 143
  run_cell term-returns-143-under-caller-trap foreground test STUB_CALLER_TRAP=1
  if [[ "$cell_exit" != 143 ]] || ! calls_have term-returns-143-under-caller-trap '^cargo-test got-TERM$' || [[ "$(grep -c '^caller-term-trap$' "$work/term-returns-143-under-caller-trap/calls")" != 1 ]] \
     || ! before term-returns-143-under-caller-trap '^cargo-test lock-released$' '^caller-term-trap$' || ! calls_have term-returns-143-under-caller-trap '^run-returned rc=143 lock=absent$'; then
    cell_failed term-returns-143-under-caller-trap "调用方设了 TERM 处理：假 cargo 收到 TERM，退干净之后调用方的处理跑恰好一次，那一趟交回 143" \
      "看 crash_amplification_run_on_this_host 收尾：先放回调用方原来的 TERM 处理、再对自己发 TERM，它的处理才轮得到"
  fi
  # 编的那一步被停：不起跑的那一步
  run_cell term-during-build-skips-test foreground build
  if [[ "$cell_exit" != 143 ]] || ! calls_have term-during-build-skips-test '^cargo-build got-TERM$' || calls_have term-during-build-skips-test '^cargo-test started$'; then
    cell_failed term-during-build-skips-test "只编那一步时给调用方 TERM：假 cargo 收到 TERM，跑的那一步不起，调用方退 143" \
      "看 crash_amplification_run_on_this_host：收到 TERM 之后不再起下一步（crash_amplification_term_received）"
  fi
  # 没被停：两步按序各起一次、日志里两步的输出按序都在，退出码原样转出；编的那一步红了不起跑的那一步
  run_cell exit-code-passes-through foreground none STUB_TEST_EXIT=7
  if [[ "$cell_exit" != 7 ]] || ! before exit-code-passes-through '^cargo-build done$' '^cargo-test started$' || ! calls_have exit-code-passes-through '^cargo-test done$' \
     || [[ "$(grep -cE '^(out|err)-(build|test)$' "$work/exit-code-passes-through/run.log")" != 4 ]] \
     || [[ "$(grep -nE '^out-test$' "$work/exit-code-passes-through/run.log" | cut -d: -f1)" -le "$(grep -nE '^out-build$' "$work/exit-code-passes-through/run.log" | cut -d: -f1)" ]]; then
    cell_failed exit-code-passes-through "没被停：编、跑两步按序各起一次，日志里两步的标准输出与标准错误按序都在，跑的那一步退 7 就原样转出 7" \
      "看 crash_amplification_run_step 与 crash_amplification_run_on_this_host：日志追加（调用方先清空）、wait 的退出码原样交回"
  fi
  run_cell build-failure-skips-test foreground none STUB_BUILD_EXIT=5
  if [[ "$cell_exit" != 5 ]] || calls_have build-failure-skips-test '^cargo-test started$'; then
    cell_failed build-failure-skips-test "编的那一步退 5：跑的那一步不起，原样转出 5" \
      "看 crash_amplification_run_on_this_host：编的那一步退出码非 0 就不起下一步（原来的 && 链）"
  fi
  # 没被停：调用方设的 TERM 处理原样放回
  run_cell caller-term-trap-kept foreground none STUB_CALLER_TRAP=1
  if [[ "$cell_exit" != 0 ]] || ! grep -qF 'caller-term-trap' "$work/caller-term-trap-kept/trap-after" || grep -qF 'crash_amplification_on_term' "$work/caller-term-trap-kept/trap-after"; then
    cell_failed caller-term-trap-kept "没被停：那一趟返回之后调用方原来的 TERM 处理还在、本库的处理已拆掉（实有「$(cat -- "$work/caller-term-trap-kept/trap-after" 2>/dev/null)」）" \
      "看 crash_amplification_restore_term_trap 与 crash_amplification_run_on_this_host 起头记下的 trap -p TERM"
  fi
  # 起 cargo 之前判 RocksDB 预编库的取值：这一格的假树里 验证包依赖 rocksdb、进程环境里四个取值是空的、PATH 前面的假 cargo 不带 entry 认的标记
  #   ⇒ 那一趟交回 78，两步 cargo 都没起，输出里有 rocksdb-prebuilt entry 的红句
  mkdir -p "$work/rocksdb-entry-refused/root/crates/singlefs-verification"
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' '[dependencies]' 'rocksdb = { version = "=0.25.0", optional = true }' \
    >"$work/rocksdb-entry-refused/root/crates/singlefs-verification/Cargo.toml"
  run_cell rocksdb-entry-refused foreground none ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC=
  if [[ "$cell_exit" != 78 ]] || calls_have rocksdb-entry-refused '^cargo-' || ! grep -qF 'rocksdb-prebuilt entry：' "$work/rocksdb-entry-refused/output"; then
    cell_failed rocksdb-entry-refused "树里的 验证包依赖 rocksdb 而进程环境里没有预编库的取值，那一趟应当交回 78、两步 cargo 都不起、输出里有 rocksdb-prebuilt entry 的红句" \
      "看 crash_amplification_run_on_this_host 里两步 cargo 之前 rocksdb-prebuilt.py entry 那一段"
  fi
  # 先编后判（文件头「先编后判」）：外壳先调 crash_amplification_build_judge_binary（假 cargo 的 --no-run 那一步写一份假的测试二进制、打 Executable 行），记一笔判段开始，
  #   再起两趟本机那一趟 ⇒ 调用记录里 cargo 只起过一回、是编的那一步、在判段开始之前；判段两趟都直接起那份二进制（工作目录与 CARGO_MANIFEST_DIR 是验证包目录、参数是 -- 之后的 --nocapture）
  local judge_cell="$work/judge-binary-built-before-the-judge-runs" judge_cargo_starts judge_last_cargo judge_runs_line
  mkdir -p "$judge_cell/root/crates/singlefs-verification"
  run_cell judge-binary-built-before-the-judge-runs judge-binary none STUB_BINARY="$judge_cell/fake-judge-binary"
  judge_cargo_starts="$(grep -E '^cargo-[a-z]+ started$' "$judge_cell/calls" | tr '\n' ' ')"
  judge_last_cargo="$(grep -nE '^cargo-' "$judge_cell/calls" | tail -n 1 | cut -d: -f1)"
  judge_runs_line="$(line_of judge-binary-built-before-the-judge-runs '^judge-runs-begin$')"
  if [[ "$cell_exit" != 0 || "$judge_cargo_starts" != "cargo-build started " || -z "$judge_runs_line" || -z "$judge_last_cargo" ]] || (( judge_last_cargo > judge_runs_line )) \
     || [[ "$(grep -cxF 'binary-run directory=singlefs-verification arguments=--nocapture manifest=singlefs-verification' "$judge_cell/calls")" != 2 ]] \
     || ! grep -qxF "build-returned rc=0 binary=[$judge_cell/fake-judge-binary]" "$judge_cell/calls"; then
    cell_failed judge-binary-built-before-the-judge-runs "判段开始之前应当只编一回（cargo 起过的只有「cargo-build started」，实有「$judge_cargo_starts」）、判段开始之后不再起 cargo、两趟都直接起编好的二进制（工作目录与 CARGO_MANIFEST_DIR 是验证包目录、参数 --nocapture）" \
      "看 crash_amplification_build_judge_binary（只起编的那一步、从 Executable 行取二进制）与 crash_amplification_run_on_this_host 里 crash_amplification_judge_binary 不空那一支"
  fi

  # 别的测试打的行接在 libtest 还没打结局的「test <名> ... 」后面：录入行与判类别的两种行首记号照认；不是行首的提到记号的一句话不认（库里的函数直接调，不起假 cargo）
  local marker_directory="$work/test-line-markers" recorded_lines
  mkdir -p "$marker_directory"; : >"$marker_directory/calls"; : >"$marker_directory/output"
  cells=$((cells + 1)); cell_exit=0
  printf '%s\n' 'test crash_given_states_case ... CRASH_LIBRARY_RECORDED blocks=3' 'test one ... test other ... CRASH_AMPLIFICATION_RECORDED blocks=4' >"$marker_directory/recorded.log"
  printf '%s\n' 'test one ... ok' 'test other ... 提到 CRASH_LIBRARY_RECORDED 的一句话' >"$marker_directory/prose.log"
  printf '%s\n' "test crash_given_states_case ... $CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX 借不到显存" >"$marker_directory/given-states.log"
  printf '%s\n' 'test crash_given_states_case ... CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD built=1 run=2' >"$marker_directory/version.log"
  printf '%s\n' "test crash_observer_case ... $CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS_LINE_PREFIX reason=取卡报的原因" >"$marker_directory/observer-images.log"
  recorded_lines="$(crash_amplification_recorded_lines "$marker_directory/recorded.log")"
  if [[ "$recorded_lines" != $'CRASH_LIBRARY_RECORDED blocks=3\nCRASH_AMPLIFICATION_RECORDED blocks=4' ]] \
     || crash_amplification_recorded_lines "$marker_directory/prose.log" >/dev/null \
     || [[ "$(crash_amplification_failure_kind "$marker_directory/given-states.log")" != device ]] \
     || [[ "$(crash_amplification_failure_kind "$marker_directory/version.log")" != build ]] \
     || [[ "$(crash_amplification_failure_kind "$marker_directory/observer-images.log")" != observer-images-without-a-card ]]; then
    cell_failed test-line-markers "记号接在「test <名> ... 」后面：录入行应当认出、去掉前缀打出来（实有「$recorded_lines」），给定状态表借不到显存判 device，结论版本对不上判 build，观察者交回的盘取不到卡判 observer-images-without-a-card；不在行首的一句话不认" \
      "看 crash_amplification_lines_without_test_prefix 与 crash_amplification_recorded_lines、crash_amplification_failure_kind 里几处行首记号"
  fi

  # 观察者交回的盘取不到卡（文件头「运行条件与失败分类」）：行首是那个记号的认成 observer-images-without-a-card，那一趟的 101 改退 78，78 的出路说等 GPU 批处理服务收这一族待判行；
  #   同一个记号不在行首的一句话不认、101 原样退（库里的函数直接调，不起假 cargo；接在「test <名> ... 」后面的那一种由 test-line-markers 那一格核）
  local no_card_directory="$work/observer-images-without-a-card" no_card_kind no_card_exit no_card_howto no_card_prose_kind no_card_prose_exit
  mkdir -p "$no_card_directory"; : >"$no_card_directory/calls"; : >"$no_card_directory/output"
  cells=$((cells + 1)); cell_exit=0
  printf '%s\n' "$CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS_LINE_PREFIX reason=取卡报的原因：观察者交回了盘" 'test result: FAILED. 0 passed; 1 failed' >"$no_card_directory/no-card.log"
  printf '%s\n' "test one ... 提到 $CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS_LINE_PREFIX 的一句话" >"$no_card_directory/prose.log"
  no_card_kind="$(crash_amplification_failure_kind "$no_card_directory/no-card.log")"
  no_card_exit="$(crash_amplification_exit_code_of_a_failed_run 101 "$no_card_directory/no-card.log")"
  no_card_howto="$(crash_amplification_explain_exit_code 78 "$no_card_directory/no-card.log" 2>&1)"
  no_card_prose_kind="$(crash_amplification_failure_kind "$no_card_directory/prose.log")"
  no_card_prose_exit="$(crash_amplification_exit_code_of_a_failed_run 101 "$no_card_directory/prose.log")"
  printf '%s\n' "kind=$no_card_kind exit=$no_card_exit prose-kind=$no_card_prose_kind prose-exit=$no_card_prose_exit" "$no_card_howto" >"$no_card_directory/output"
  if [[ "$no_card_kind" != observer-images-without-a-card || "$no_card_exit" != 78 || "$no_card_howto" != *"等 GPU 批处理服务收这一族待判行"* \
        || -n "$no_card_prose_kind" || "$no_card_prose_exit" != 101 ]]; then
    cell_failed observer-images-without-a-card "行首是 $CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS_LINE_PREFIX 的日志应当判 observer-images-without-a-card（实有「$no_card_kind」）、101 改退 78（实有 $no_card_exit）、出路说等 GPU 批处理服务收这一族待判行；不在行首的一句话不认（实有「$no_card_prose_kind」）、101 原样退（实有 $no_card_prose_exit）" \
      "看 crash_amplification_failure_kind 里 observer_images_without_a_card_pattern 那一支、crash_amplification_exit_code_of_a_failed_run 与 crash_amplification_explain_exit_code 的 observer-images-without-a-card 那一支"
  fi

  # 判器输入核对（文件头「判器输入核对」）：假树里一份键所读的文件 watched.rs、一份闭包外的文件 other.rs，假构建输出目录里放键表与摘要表，假二进制 = 一段前缀 + 键表 + 一段后缀。
  #   库里的函数直接调，不起假 cargo
  local inputs_directory="$work/judging-inputs" inputs_name inputs_output inputs_exit inputs_tree inputs_out
  inputs_tree="$inputs_directory/root"; inputs_out="$inputs_directory/target/release/build/singlefs-verification-0000000000000000/out"
  for inputs_name in judging-inputs-unchanged judging-inputs-changed judging-inputs-unknown judging-inputs-batch-agrees place-binary-without-debug-info; do
    mkdir -p "$work/$inputs_name"; : >"$work/$inputs_name/calls"; : >"$work/$inputs_name/output"
  done
  mkdir -p "$inputs_tree/crates/x/src" "$inputs_out" "$inputs_directory/target/release/deps"
  printf '%s\n' 'pub fn watched() {}' >"$inputs_tree/crates/x/src/watched.rs"
  printf '%s\n' 'pub fn other() {}' >"$inputs_tree/crates/x/src/other.rs"
  printf '%s\n' '# singlefs entry group keys' $'toolchain\tselftest' $'group\tselftest\tkeyed\t00ff' >"$inputs_out/judging_units_keys.tsv"
  printf '%s\t%s\n' "$(sha256sum <"$inputs_tree/crates/x/src/watched.rs" | cut -d' ' -f1)" crates/x/src/watched.rs >"$inputs_out/watched_file_digests.tsv"
  { printf 'before-the-table\n'; cat -- "$inputs_out/judging_units_keys.tsv"; printf 'after-the-table\n'; } >"$inputs_directory/target/release/deps/with-table"
  printf 'a binary that carries no key table\n' >"$inputs_directory/target/release/deps/without-table"
  inputs_call() { # <二进制的文件名>：交回值放 inputs_exit，打的一行放 inputs_output
    inputs_exit=0
    inputs_output="$(crash_amplification_judging_inputs_unchanged "$inputs_directory/target/release/deps/$1" "$inputs_tree")" || inputs_exit=$?
  }
  cells=$((cells + 1)); cell_exit=0
  inputs_call with-table
  if [[ "$inputs_exit" != 0 || "$inputs_output" != "same watched=1" ]]; then
    cell_failed judging-inputs-unchanged "树没变：应当打 same watched=1 交 0（得到 $inputs_exit「$inputs_output」）" \
      "看 crash_amplification_judging_inputs_unchanged：摘要表里每份文件都没变就是 same"
  fi
  inputs_batch_check() { # <树的状态> ：三个二进制（有表、没表、有表）一批交过去（并行 1 与 3 各一遍），每一行都与逐个调时的「交回值<TAB>打的一行」相同；不同记这一格判错
    local jobs name single="" batch deps="$inputs_directory/target/release/deps"
    for name in with-table without-table with-table; do
      inputs_call "$name"
      single+="${inputs_exit}"$'\t'"${inputs_output}"$'\n'
    done
    for jobs in 1 3; do
      batch="$(crash_amplification_judging_inputs_of_binaries "$inputs_tree" "$jobs" "$deps/with-table" "$deps/without-table" "$deps/with-table")"$'\n'
      if [[ "$batch" != "$single" ]]; then
        cell_failed judging-inputs-batch-agrees "$1：一批三个二进制（有表、没表、有表，并行 $jobs）每一行应当与逐个调时的「交回值<TAB>打的一行」相同（逐个得到「${single//$'\n'/｜}」，批量得到「${batch//$'\n'/｜}」）" \
          "看 crash_amplification_judging_inputs_of_binaries：每个二进制各答各的、按给的次序打，同一个构建输出目录的摘要答案共用，不把一个二进制的答案给批里所有二进制"
      fi
    done
  }
  cells=$((cells + 1)); cell_exit=0
  inputs_batch_check "键所读的文件没变"
  printf '%s\n' 'pub fn other_changed() {}' >"$inputs_tree/crates/x/src/other.rs"
  inputs_call with-table
  if [[ "$inputs_exit" != 0 || "$inputs_output" != "same watched=1" ]]; then
    cell_failed judging-inputs-unchanged "键所读的文件没变、闭包外的文件变了：应当打 same watched=1 交 0（得到 $inputs_exit「$inputs_output」）" \
      "看 crash_amplification_judging_inputs_unchanged：只比摘要表里列的那几份，表外的文件不看"
  fi
  cells=$((cells + 1)); cell_exit=0
  printf '%s\n' 'pub fn watched_changed() {}' >"$inputs_tree/crates/x/src/watched.rs"
  inputs_call with-table
  if [[ "$inputs_exit" != 1 || "$inputs_output" != "changed watched=1 changed=1：crates/x/src/watched.rs" ]]; then
    cell_failed judging-inputs-changed "键所读的一份文件内容变了：应当打 changed watched=1 changed=1：crates/x/src/watched.rs 交 1（得到 $inputs_exit「$inputs_output」）" \
      "看 crash_amplification_judging_inputs_unchanged：摘要表里每份文件按树根下此刻的内容重算再比"
  fi
  rm -f -- "$inputs_tree/crates/x/src/watched.rs"
  inputs_call with-table
  if [[ "$inputs_exit" != 1 || "$inputs_output" != "changed watched=1 changed=1：crates/x/src/watched.rs" ]]; then
    cell_failed judging-inputs-changed "键所读的一份文件不在了：应当同样打 changed 交 1（得到 $inputs_exit「$inputs_output」）" \
      "看 crash_amplification_judging_inputs_unchanged：读不了的文件算变了"
  fi
  cell_exit=0
  inputs_batch_check "键所读的一份文件不在了"
  cells=$((cells + 1)); cell_exit=0
  inputs_call without-table
  if [[ "$inputs_exit" != 2 || "$inputs_output" != unknown\ * ]]; then
    cell_failed judging-inputs-unknown "二进制里找不到对应的键表：应当打 unknown 交 2、不当没变（得到 $inputs_exit「$inputs_output」）" \
      "看 crash_amplification_judging_inputs_unchanged：键表在二进制里找不到的构建输出目录不算数"
  fi
  : >"$inputs_out/watched_file_digests.tsv"
  inputs_call with-table
  if [[ "$inputs_exit" != 2 || "$inputs_output" != unknown\ * ]]; then
    cell_failed judging-inputs-unknown "摘要表是空的：应当打 unknown 交 2（得到 $inputs_exit「$inputs_output」）" \
      "看 crash_amplification_judging_inputs_unchanged：空的摘要表证明不了什么"
  fi
  cell_exit=0
  inputs_batch_check "摘要表是空的"

  # 拷测试二进制去掉调试段（文件头「自证」）：PATH 前面放一份假 objcopy（目标 = 源再加一行「# 去掉了调试段」；另一份退 1），库里的函数直接调
  local place_directory="$work/place-binary-without-debug-info" place_exit
  mkdir -p "$place_directory/bin" "$place_directory/failing"
  printf '%s\n' '#!/usr/bin/env bash' '{ cat -- "$2"; echo "# 去掉了调试段"; } >"$3"' >"$place_directory/bin/objcopy"
  printf '%s\n' '#!/usr/bin/env bash' 'exit 1' >"$place_directory/failing/objcopy"
  chmod +x "$place_directory/bin/objcopy" "$place_directory/failing/objcopy"
  printf 'the binary\n' >"$place_directory/source"
  cells=$((cells + 1)); cell_exit=0
  place_exit=0
  PATH="$place_directory/bin:$PATH" crash_amplification_place_binary_without_debug_info "$place_directory/source" "$place_directory/stripped" 2>>"$place_directory/output" || place_exit=$?
  if [[ "$place_exit" != 0 || "$(tail -n 1 -- "$place_directory/stripped" 2>/dev/null)" != "# 去掉了调试段" || "$(cat -- "$place_directory/source")" != "the binary" ]]; then
    cell_failed place-binary-without-debug-info "objcopy 在时应当用 objcopy --strip-debug 写出目标、交 0、原件不动（交 $place_exit，目标末行「$(tail -n 1 -- "$place_directory/stripped" 2>/dev/null)」）" \
      "看 crash_amplification_place_binary_without_debug_info：objcopy --strip-debug <源> <目标> 成了就交 0，不再 cp"
  fi
  place_exit=0
  PATH="$place_directory/failing:$PATH" crash_amplification_place_binary_without_debug_info "$place_directory/source" "$place_directory/copied" 2>>"$place_directory/output" || place_exit=$?
  if [[ "$place_exit" != 3 || "$(cat -- "$place_directory/copied" 2>/dev/null)" != "the binary" ]]; then
    cell_failed place-binary-without-debug-info "objcopy 退非 0 时应当整份拷、交 3、目标与源相同（交 $place_exit，目标「$(cat -- "$place_directory/copied" 2>/dev/null)」）" \
      "看 crash_amplification_place_binary_without_debug_info：objcopy 没成就删掉写出的半份、整份 cp、交 3"
  fi

  # 多机配置判法 --emit-addresses 判不过时照它的 ✗ 那一句报（文件头「这一台与另一台」）：直接调 crash_amplification_host_addresses、起真的判法，
  #   这一台的地址经判法只在自证里认的 CONFIGURATION_SELFTEST_LOCAL_ADDRESSES 给成 127.0.0.1（本份是 --selftest 起的，判法沿祖先认得出）。
  #   写全的那一份（两个显卡键都写）先要判得过、认出 127.0.0.1，缺键那一份红就只红在缺键上；HOSTS 里没有 127.0.0.1 的那一份把地址整份换成 192.0.2.1
  local addresses_name addresses_exit
  addresses_call() { # <格名> <配置文件> <放输出的文件名>：交回值放 addresses_exit；标准输出打认出的这一台，标准错误是库打的
    addresses_exit=0
    ( export CONFIGURATION_SELFTEST_LOCAL_ADDRESSES=127.0.0.1
      crash_amplification_host_addresses "$2" || exit $?
      echo "this=$crash_amplification_this_host_address" ) >"$work/$1/$3.out" 2>"$work/$1/$3.err" || addresses_exit=$?
    cat -- "$work/$1/$3.out" "$work/$1/$3.err" >>"$work/$1/output"
  }
  for addresses_name in emit-addresses-missing-key-named emit-addresses-host-absent-named; do
    mkdir -p "$work/$addresses_name"; : >"$work/$addresses_name/calls"; : >"$work/$addresses_name/output"
  done
  printf '%s\n' "HOSTS=127.0.0.1 192.0.2.66" "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.66=/selftest/other-cargo" \
    "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.66=/selftest/other-builds" "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.66=2G" \
    QUIESCE_STOP_COMMAND= QUIESCE_STOPPED_CHECK_COMMAND= QUIESCE_START_COMMAND= QUIESCE_STARTED_CHECK_COMMAND= ENABLE_ACROSS_MACHINES=0 ENABLE_GPU=1 \
    'GPU_CARDS_BY_PRIORITY="127.0.0.1:0:8G"' "NFS_SHARED_ROOTS=$work" >"$work/emit-addresses-missing-key-named/missing-key.env"
  { cat -- "$work/emit-addresses-missing-key-named/missing-key.env"; echo 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="127.0.0.1:0:24G"'; } >"$work/emit-addresses-missing-key-named/whole.env"
  sed 's/127\.0\.0\.1/192.0.2.1/g' "$work/emit-addresses-missing-key-named/whole.env" >"$work/emit-addresses-host-absent-named/host-absent.env"

  cells=$((cells + 1))
  addresses_call emit-addresses-missing-key-named "$work/emit-addresses-missing-key-named/whole.env" whole
  cell_exit="$addresses_exit"
  if [[ "$addresses_exit" != 0 || "$(cat -- "$work/emit-addresses-missing-key-named/whole.out")" != this=127.0.0.1 ]]; then
    cell_failed emit-addresses-missing-key-named "两个显卡键都写全的假配置应当判得过、认出这一台是 127.0.0.1（实有「$(head -n 1 -- "$work/emit-addresses-missing-key-named/whole.err")」）" \
      "先看这一格造的假配置是不是缺了判法要的别的键（判法的原话在上面输出里），再看 crash_amplification_host_addresses 解析 --emit-addresses 那几行"
  else
    addresses_call emit-addresses-missing-key-named "$work/emit-addresses-missing-key-named/missing-key.env" missing-key
    cell_exit="$addresses_exit"
    if [[ "$addresses_exit" != 1 ]] || ! head -n 1 -- "$work/emit-addresses-missing-key-named/missing-key.err" | grep -qE '^  ✗ 多机配置判法 --emit-addresses 退 1：多机配置判不过：.*GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 没写' \
       || grep -qF '认不出这一台' "$work/emit-addresses-missing-key-named/missing-key.err"; then
      cell_failed emit-addresses-missing-key-named "开了 GPU 而缺 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED：应当交回 1，头一行报退出码与判法缺键那一句、整段不说认不出这一台（实有头一行「$(head -n 1 -- "$work/emit-addresses-missing-key-named/missing-key.err")」）" \
        "看 crash_amplification_host_addresses 判不过那一支：取判法头一行「✗ …」原样报，只有那一句里有 CRASH_AMPLIFICATION_HOST_UNRECOGNIZED_MARK 才冠认不出这一台"
    fi
  fi
  cells=$((cells + 1))
  addresses_call emit-addresses-host-absent-named "$work/emit-addresses-host-absent-named/host-absent.env" host-absent
  cell_exit="$addresses_exit"
  if [[ "$addresses_exit" != 1 ]] \
     || ! head -n 1 -- "$work/emit-addresses-host-absent-named/host-absent.err" | grep -qE '^  ✗ 认不出这一台是 HOSTS 里哪一项：多机配置判法 --emit-addresses 退 1：多机配置判不过：HOSTS 里没有一项是这一台网卡上的地址'; then
    cell_failed emit-addresses-host-absent-named "HOSTS 里没有这一台（127.0.0.1）：应当交回 1，头一行说认不出这一台、连着退出码与判法那一句（实有头一行「$(head -n 1 -- "$work/emit-addresses-host-absent-named/host-absent.err")」）" \
      "看 crash_amplification_host_addresses 判不过那一支与 CRASH_AMPLIFICATION_HOST_UNRECOGNIZED_MARK：判法那三句说法改了的，这个常量跟着改"
  fi

  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 bash $lib_script --selftest" >&2
    return 1
  fi
  if crash_amplification_switch_wheel_inner; then echo "  ✓ --selftest（开关轮里起的一份，不再起开关轮）：$cells 格都对"; return 0; fi

  # 开关轮（文件头「弄坏开关」）：从文件头现读开关与点名的格、与代码里认的开关比齐，逐个打开再跑一遍 --selftest，红的格要与点名的逐格相等
  local table assignment named in_table="" in_code duplicated round_failures=0 listed_count
  switch_wheel_assignments=(); switch_wheel_expected=()
  table="$(crash_amplification_read_switch_table "$lib_script" CRASH_AMPLIFICATION_LIB_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named")
    in_table+="${assignment#*=}"$'\n'
  done <<<"$table"
  listed_count="${#switch_wheel_assignments[@]}"
  if (( listed_count == 0 )); then
    echo "  ✗ 开关轮：文件头「弄坏开关」那张表一个开关都没读到" >&2
    echo "     → 怎么办：表里一行「#   CRASH_AMPLIFICATION_LIB_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（文件头「弄坏开关轮」一段的写法）" >&2
    return 1
  fi
  duplicated="$(printf '%s\n' "${switch_wheel_assignments[@]}" | LC_ALL=C sort | uniq -d | tr '\n' ' ')"
  if [[ -n "$duplicated" ]]; then
    echo "  ✗ 开关轮：文件头表里同一个开关列了不止一次：$duplicated" >&2
    echo "     → 怎么办：一个开关只写一行，点名的格都写在它下面" >&2
    round_failures=$((round_failures + 1))
  fi
  in_code="$(grep -oE 'crash_amplification_lib_break_is [a-z0-9-]+' "$lib_script" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table="$(printf '%s' "$in_table" | LC_ALL=C sort -u)"
  if [[ "$in_code" != "$in_table" ]]; then
    echo "  ✗ 开关轮：代码里 crash_amplification_lib_break_is 认的开关与文件头表里列的不一样（只在代码里：$(LC_ALL=C comm -23 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table") | tr '\n' ' ')；只在表里：$(LC_ALL=C comm -13 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table") | tr '\n' ' ')）" >&2
    echo "     → 怎么办：代码里每一处认的开关都在表里写一行（连同点名的格），表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  crash_amplification_switch_wheel "$work" '✗ 自证 ([a-z0-9-]+)：' 0 -- bash "$lib_script" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里点名的不同，要么改那一处判法让开关红在点名的格，要么照实改文件头那一行；单看一个开关：CRASH_AMPLIFICATION_LIB_BREAK=<开关> bash $lib_script --selftest" >&2
    return 1
  fi
  echo "  ✓ --selftest：本机那一趟被 TERM 停时（放后台的子 shell 里、调用方的 shell 里没设与设了 TERM 处理）TERM 到得了假 cargo、假 cargo 在那一趟返回之前退干净、退 143，编的那一步被停不起跑的那一步，没被停时两步按序起、日志按序、退出码原样转出、调用方的 TERM 处理原样放回，先编后判（判段开始之前只编一回、判段直接起编好的二进制、不起 cargo），多机配置判法判不过时照它那一句报（缺键报缺键、认不出这一台才说认不出），$cells 格都对（cargo 是假的，不起重活）；"
  echo "    开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格"
  return 0
}

# 单独执行：只认 --selftest；被 source 时（调用方的 $0 不是这个文件）什么都不做
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -uo pipefail
  # 弄坏开关只在自证里认：单独执行而命令行不是单独一个 --selftest，两个变量（文件头「弄坏开关」读的 CRASH_AMPLIFICATION_ITEM_BREAK、本库自己的 CRASH_AMPLIFICATION_LIB_BREAK）有一个非空就退 2，在参数解析之前
  if [[ "${1:-}" != --selftest || $# -ne 1 ]]; then
    for break_variable in CRASH_AMPLIFICATION_ITEM_BREAK CRASH_AMPLIFICATION_LIB_BREAK; do
      if [[ -n "${!break_variable:-}" ]]; then
        echo "  ✗ $break_variable 只在自证里认（弄坏开关只给 --selftest 证红用），这一趟不是自证" >&2
        echo "     → 怎么办：去掉 $break_variable 再跑；要证红就跑 $break_variable=<项> bash $0 --selftest" >&2
        exit 2
      fi
    done
  fi
  if [[ "${1:-}" == --selftest && $# -eq 1 ]]; then crash_amplification_lib_selftest; exit $?; fi
  echo "  ✗ 用法：$0 --selftest（这是被 source 的函数库，单独执行只认 --selftest）" >&2
  echo "  → 怎么办：要用库里的函数就在脚本里 source 它；要自证本机那一趟起 cargo 的那一段就跑 bash $0 --selftest" >&2
  exit 2
fi
