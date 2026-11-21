#!/usr/bin/env python3
# admission: always 判的是此刻的登记表、源码与判定库，门禁阶段与实验每一次开跑之前都要现判
# run-condition: none 它要的 git 由它自己读，读不到的按它的约定判（算不出的那一条按拒算现判、前提没齐退 3），不交给调用方预判
"""准入：门禁与实验开跑之前判「这一条判定该不该现判」。门禁与实验共用这一个模块、一张登记表。

复用的单位是一条判定（一条测试函数、一份自证、复跑表的一行、一条崩溃枚举用例日志的一项），不是一道门禁、也不是一格：
  单元键相同 ⇒ 不跑，沿用那一条判定。门禁各道在道内按判定入库，这个模块只给它们算单元键、查库、记库；
  实验：装置入口单元闭包的单元键与这一行的参数，与 research/results/ 里某份产物头记的相同就拒重跑。
  登记表 .claude/gate.d/stage-inputs.tsv 只答「从哪些入口起算」，整张表与登记行都不进任何键（D13（验证路线） 已定项 17）。
  不比树、不比提交、不比 HEAD：refs/sop/staged-green 只是 scripts/gate/gate-staged.sh 记的「上次整轮全绿的树」，这里一处都不读。

单元键（compute_unit_key）：下面这些行各以换行收尾、排好序连起来取 sha256：
  entry <入口闭包的键>   同目录 lib_function_units.py 的 closure_key：入口目标的生效 cfg 集、闭包里每个单元（种类、去注释指纹）的多重集、数据单元摘要、入口单元
    版本一样都不进键（D13（验证路线） 已定项 17：键只由代码函数、屏障与数据流组成）：rustc -vV、cargo -V、目标、RUSTFLAGS 与别的构建环境
                          （KeyContext.rust_toolchain_lines、build_environment_setting_lines）、python3 与 bash 的版本、闭包里点名的外部程序（cargo、herd7、busybox、
                          qemu-system-*）的内容与版本、Cargo.lock 里的依赖版本（KeyContext.dependency_lines）都不读进键，闭包里的工具链询问也不出工具链单元
                          （lib_function_units.py 文件头「版本询问」）；Verus、herd7、QEMU 这类判定工具升级由人登记、整族删重判，不靠键作废。
                          编译档与线程数同样不进：请求带的 profile:<名> 只定入口目标的生效 cfg（release 这一类档的 cfg 不带 debug_assertions），
                          与 crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py 文件头「构建档与线程数不进键」同一口径
  judge <判法入口闭包的键>   请求带 judge:<入口> 的：判这一条结局的那几个函数（例 scripts/gate/lib_libtest_result_lines.py::result_line）的闭包键（解释器版本不进）；
                            判法闭包里按行进键的登记表只进它按这一条的入口文件查的那几行（lib_function_units.py 文件头「按行进键的登记表」，row_keyed_closure）
  program-arguments <json>  请求带 program-argument:<词> 的：程序入口（python 与 shell）带的参数照次序（闭包也照它剪，lib_function_units.py 文件头「程序入口带参数」；
                            弄坏开关 program-arguments-unkeyed 下不进）
  line <文本>               请求带 line:<文本> 的：这一条判定自己的内容输入（这一行的参数、GPU 开关与特性集这一类），调用方不许把名字、路径、上限、限时写进来
  file <内容 sha256>        请求带 file:<仓根起的路径> 的：数据（期望产物这一类）只进内容，不进路径；不在也是一种取值
  variable <名> <取值>      改判定域或结论的环境变量（DOMAIN_ENVIRONMENT_VARIABLES 登记的那几个）里闭包读到的：闭包里有一个单元的原文里有这个名字的字面串
                            （"<名>"；登记的是前缀的，有以这个前缀起头的字面串）才进，取值取请求的 variable:<名>=<值>（没有这一项写 unset），
                            按值登记的进值、按文件登记的进那份文件的内容 sha256（路径不进；不在写「不在」）。
                            闭包读不到的不进：没读这个变量的判定不因它作废（弄坏开关 domain-environment-unkeyed 下一律不进）
  不带路径、名字、行号、登记行、手写版本号、资源上限、限时、提交、树。组成清单（单元行：种类、限定名、文件:起止行、指纹；其余输入行）随键落盘，只当索引。
  按角色排除（.claude/rules/verification-reference.md「复用门禁的判定：按每条判定的单元键判，不按「改了哪个目录」或「哪棵树绿过」判」「单元键的组成与排除」文件级；not_judging_files_of_entry）：
    NOT_JUDGING_FILES 那两份（算键的库自己与函数单元库）管怎么算键、不管判对错：入口与 judge: 的闭包都不收它们的单元，也不从它们往下连（只经它们读到的数据、起的程序不进键）；
    入口本身在这两份里的照收，它调到的同一份里的单元也收（它们自己的判定、function-units-two-implementations-agree 的判法）。crates 变异判法不走这里（scripts/mutation/lib_mutation_units.py 的 judging_units 自己算，两份照进）。
  拒算（lib_function_units.py 文件头「拒算」：闭包收不全的形态；另有 key-uncomputable：入口找不到、在这一组 cfg 下不编）的那一条没有键，
  这一趟照跑、不复用、不入库；红句与成功句报拒算条数与头几处形态。

判定请求（verdict-plan 的请求文件一行一条，制表符分隔；unit-key 的参数同一套）：<名字><制表符><入口>[<制表符><项>…]
  名字是这一条判定在库里的索引（门禁写测试函数的入口原样、格名、自证的命令），不进键。入口照 lib_function_units.py 的写法，相对 root: 那个目录。
  入口写 -（JUDGE_ONLY_ENTRY）的：被判那棵树里没有程序入口，判的是判法拿被判那棵树当数据跑（门禁 code-tooling 的 function-units-two-implementations-agree、
    mutation-list-items、entry-scripts 三格）：键里的 entry 行是「entry -」，其余只由 judge:、line:、file: 组成；没有入口闭包，variable 行不进；
    一个 judge: 都没有的拒算（key-uncomputable）。弄坏开关 judge-only-entry-judges-unkeyed 下这种请求的 judge: 不进键。项：
    root:<仓根起的目录>   闭包从哪棵树算（research 工作区写 root:scripts/experiment-apparatus；不写是仓根）
    target:<目标号>        lib_function_units 的目标号 <包目录名>:<lib|bin|test|…>:<名>（不写照它的默认挑）
    feature:<包>/<特性>    no-default-features    cfg:<名或 名="值">    profile:<名>（cargo 的编译档，例 release）
    judge:<入口>   judge-root:<目录>（判法在哪棵树里算闭包：门禁写它自己所在的仓，样本档被判的是样本；不写是被判的根；只当索引，不进键）
    line:<文本>   file:<路径>
    program-argument:<词>  程序入口（只写 <文件> 的 python 与 shell 入口）带的参数，可给几次、照次序；闭包只收这组参数真走到的那一支（break-switch-replay、
                           research-script-selftests 这类「这份脚本带这组参数跑」的判定写它），这组参数另进键。弄坏开关 shell-program-arguments-dropped 下
                           shell 入口不把参数交给闭包（整份收）
    variable:<名>=<值>   这一趟起判定时这个环境变量的取值（名要是 DOMAIN_ENVIRONMENT_VARIABLES 登记的或以登记的前缀起头；没设的不写这一项）。
                         test-functions-request 照它起那一刻的环境逐个打（调用方在交给 cargo 的同一份环境里起它）；进不进键、怎么进见「单元键」的 variable 一行

判定库（VerdictLibrary）：在 prune-server 里（scripts/prune-server/prune_server_client.py 文件头「键空间」，族 unit-verdicts），一种类一份，
  名字是 <被判的根的 git common-dir>/singlefs-unit-verdicts/<种类>（本地不建、不读、不写；键里只用种类名，哪棵仓不进键，命名空间只在临时服务底下给，
  客户端文件头「键空间」），两台、各 worktree 与仓副本读写同一份、跨轮留着，不随判法换目录、没有版本目录、不按个数删。服务连不上客户端退 75，调用方不当「库里没有」现判。
  被判的根不是 git 工作树（取不到 git common-dir）、这棵树里没有客户端：退 2（LibraryUnavailable），一条都不判，没有「不读不写、全部现判」的那条路
  （弄坏开关 library-outside-git-tolerated 下不读不写）。样本档要在上一版的树上算键、记进样本自己的库的，把上一版的树放在样本根底下（取到的是同一个 common-dir）。
  记红、撤绿、下趟现判：verdict-record 收 --red-names 的那几条照计划里的键写一条 verdict=red 的 keys/<键>.json，盖掉这把键原有的绿；lookup 遇到键记着红答 judge、
  原因以「上次判红」起头；之后判绿照常盖回绿（弄坏开关 red-verdict-not-recorded 下判红不写）。
  种类由调用方定（门禁写登记键：阶段文件名或 <阶段>:<格>）。keys/<键>.json 是一条判定（verdict 是 green 或 red、key、item、request、how（跑法）、summary、
  judged（判定日期）、composition（组成清单）），items/<名字的 sha256>.json 是这个名字最近一次判绿的那一份，只用来在键对不上时比组成清单、报出变了的单元。
  键相同就复用，不看记下来的是哪个名字（两条判定内容相同，判定就相同）。
  同一种类在服务里另有 flow-lines/<组成键>.lines：一个测试二进制整份现判时打的覆盖行与计数行，
  由 scripts/crash/crash-flow-coverage.py collect 经 VerdictLibrary(…).store 写、读（组成键是那个测试二进制里编得进的测试函数的（名字，键）排好序取 sha256），
  这一份的 lookup / record 不碰它，没有只是下一趟整份复用的那几份报行不全。
  同族另两样各占一个种类，不经 VerdictLibrary 的 keys/ 写法，经 library-files 子命令读写（下面「library-files」）：
    replay-row（scripts/experiment-apparatus/replay.sh 按行复用）：keys/<行键>/verdict、line、product、finished、how、composition 各一条，items/<前三列的 sha256> 一条记那一行最近一次的键；
    e182-single-device、e182-single-device-units-kept（E182（被抛弃根读不出的三条岔路）两份长跑脚本的片结果）：<片键的 sha256>.shard 一条一片。
  这几样都只在服务里，本地种类目录不建、不读、不写。

library-files（unit-verdicts 族里按文件存的种类：replay-row、E182 的片结果；装置与入口读写目录的，经这一份在服务与这一趟自己的中转目录之间搬）：
  库名 = <git common-dir>/singlefs-unit-verdicts/<种类>（只当名字与命名空间，同 VerdictLibrary；父目录名不是 singlefs-unit-verdicts 的退 2），
  服务里的相对路径 = 中转目录里的相对路径（「/」分隔）；以 . 起头的路径段（原子写的临时文件）不搬。
  fetch 把服务里这个种类（给了前缀只取那几个前缀底下的）逐条写进中转目录；store 把中转目录里的文件（给了相对路径只写那几条，照给的次序）写进服务，
  逐字节相同的跳过；has 打出给的相对路径里服务有的那几条。中转目录是调用方这一趟自己的，并回服务之后由调用方删掉，不留给下一趟。
  服务连不上客户端退 75（弄坏开关 library-files-unreachable-as-absent 下 fetch 照「库里没有」交 0 条、退 0，改错的样子）。

复用哨兵（verdict-plan 与 test-plan 每一趟都插；规则照 scripts/gate/lib_reuse_checks.py 文件头「二」，判法是那里的 ReuseSentinel，这里不另写一份）：
  候选 = 此刻键算得出、库里 keys/<键>.json 记着判绿、记的键与此刻相同的请求（sentinel_candidates 直接读键文件，与 VerdictLibrary.lookup 分开查）。
  种子取 SINGLEFS_REUSE_SENTINEL_SEED（不设现取、打进输出），抽 SINGLEFS_REUSE_SENTINEL_COUNT 条（默认 3）当哨兵，再从剩下的抽 1 条强制现判：
    强制现判那一条不查库、决定写 judge、原因以「强制现判（复用哨兵抽中…」起头；哨兵查完库就记下它的结局，有一条不是 reuse ⇒「复用失灵」，verdict-plan 退 1；
    强制现判那一条的判定在 verdict-record 里比（判绿了与库里同为绿；没判绿的不在这里判）。状态存在 <计划文件>.reuse-sentinel.json（与 .observed），计划文件是 - 时不存。
  不抽（打一行「不抽哨兵」）：没有判定库、被判的根下有 .unit-verdict-sample/（门禁样本档：样本钉死了现判与复用的条数）、
    同目录没有 lib_reuse_checks.py（拷进不带它的小树里跑的这一份）。键相同的不足 K 条报「哨兵不足 K 条」，不红。
  自证：reuse-three-questions（复用三问四趟，规范输出 REUSE_SELFTEST script=scripts/gate/admission.py …）、reuse-sentinel（哨兵被现判判红），在 run_reuse_three_question_cells。

按条复用失灵（不能复用就判红；调 verdict-plan / test-plan 的几处共用这一份判法：门禁 crash-point-replay 按条复用的两格、code-tooling 的 research-script-selftests 格、
  model-differential-and-scenarios 各格、research-build-and-replay 的 research-unit-tests 格、scripts/crash/crash-point-replay.sh 按条复用的那一段）：
  调用方在 verdict-plan / test-plan 退非 0、退 0 而没打 PLAN 那一行、verdict-record 退非 0 时调 plan-failure，把那一趟的输出交给它；
  失灵 = 输出里有复用哨兵的红句（「复用失灵：」「判定不稳：」「…强制现判没起作用」那几行 ✗），或计划那两条退非 0、没打 PLAN。
  plan-failure 退 1 并打 ✗ 与出路（点名哪一条哨兵、或退了几，出路指到本文件头的哪一段）；调用方照旧全部现判跑完那一趟，那一道（那一格）结论判红。
  记红没成算失灵：verdict-record 照计划里的键记红没成（写帧出错、读不了计划或名字文件、样本档造的）的，每条打一行「✗ 判红没记进判定库：…」、末行 RECORD 带
    red-unrecorded=<条数>、退 1；调用方交 plan-failure 时带 --red <根> <种类> <计划文件> <判红的名字文件>（交 --red-names 的那一趟都带）。plan-failure 见了那一句，
    或带了 --red、名字文件不空、verdict-record 退非 0 而输出里没有 RECORD 那一行（没跑完：被停、崩了）的，改走撤绿：计划里那几个名字的键在库里记着绿的删掉
    （VerdictLibrary.withdraw_green，删完回读），打 ✗、退 1，那一道（那一格）判红；撤也撤不掉（没带 --red、读不了计划、开不了库、删不成）的打 ✗「整道判红」，
    出路写服务好了之后照键逐条删（prune_server_client.py delete-prefix）。这一条不归弄坏开关 plan-failure-not-red。
    样本档造记红没成：被判的根下有 .unit-verdict-sample/red-record-failure 时，verdict-record 照它当每条判红都没记成（记绿照常）。
  只交判绿的那一趟 verdict-record 退非 0 而输出里没有哨兵红句的（记绿记不进库）不算失灵：plan-failure 退 0，调用方照旧只报一句、下一趟照判（查不到绿，不会错复用）。
  prune-server 客户端退 3 不是记不进库：三条子命令哪一条退 3、输出里有客户端「✗ prune-server 客户端：…（退 3）」那一行（没写服务地址、多机配置读不了、服务拒了这一帧），
    plan-failure 打 ✗「prune-server 客户端退 3」与客户端给的出路、退 1，调用方照红办（配置错不当本次未跑，也不当服务不可用）。
  prune-server 不可用不是失灵：三条子命令哪一条退 75（scripts/prune-server/prune_server_client.py 文件头「退出码」），plan-failure 打 ✗「prune-server 不可用」与出路
    （python3 prune-server/control check、报给主 agent），退 75、不判复用失灵；调用方照「服务不可用」那一格退 75（不现判、不判红也不判绿），
    整道由 .claude/gate.d/lib/stage-cells.sh 汇总成 75（有一格真红仍整道退 1）。
  欠撤：判红那一趟记红时 prune-server 正好不可用（verdict-record 退 75，同一把键原有的绿还在），调用方带 --red 交的 plan-failure 照旧退 75，
    同时把判红那几条（计划里有键的）的键与种类记成「欠撤」：运行记录，不是判定库（.claude/rules/verification.md「判定库只有一份」里不进库的那一类），
    放在上次红的清单那一层（scripts/gate/lib_reuse_checks.py 的 last_runs_directory：SINGLEFS_LAST_RUNS_DIRECTORY 或共享数据根下的 last-runs）底下的
    withdrawals-owed/<库名的 sha256>/<键>.json，一把键一份（种类、库名、名字、键、日期），写完回读；记不下的打 ✗ 与手删的出路。
    下一趟 verdict-plan / test-plan 开跑、算计划之前先读这份库的欠撤：逐把键撤掉原有的绿（VerdictLibrary.withdraw_green，删完回读）、删掉那一份再核不在；
    有一份撤不掉（服务不可用、回读还是绿、记录读不了、删不掉）打 ✗、整道退 75、不算计划、不判绿，欠撤留到下一趟。上次红的清单那一层取不到的打一行 ! 照算（记的那一趟同样取不到、记不下）。
    样本档造：被判的根下有 .unit-verdict-sample/red-record-unavailable 时 verdict-record 记红那一帧照服务不可用退 75；有 .unit-verdict-sample/owed-withdrawal-unavailable
    时 verdict-plan / test-plan 撤欠撤那一帧照服务不可用、整道退 75。自证两格在 run_reuse_fix_cells（记红时服务不可用 ⇒ 记欠撤、下一趟撤成现判；撤不掉 ⇒ 整道 75、欠撤留着）。
  样本档造失灵：被判的根下有 .unit-verdict-sample/plan-failure（一行 exit-1 或 sentinel-reuse-failed）时，verdict-plan / test-plan 照它装失灵——
    exit-1 不算键、不查库、打一行 ✗ 退 1；exit-75 不连服务、打一行 ✗「样本档造的服务不可用」退 75（各调用方照「服务不可用」退 75 的样本用它）；
    sentinel-reuse-failed 照常算、照常打计划，再添一条「复用失灵：<头一条请求> 键相同却现判了（样本档造的假哨兵…）」退 1。
    只有门禁样本档放这份文件；写的不是这三样退 2。
  自证：plan-failure 那几格（run_plan_failure_cells：哨兵三种红句、退 1 / 2、没打 PLAN、记库退非 0 而没有红句、记红没成没带 --red、三条子命令退 75 判不可用、三种样本标记）；
    记红没成撤绿那两格在 run_reuse_fix_cells（样本标记 red-record-failure 下 verdict-record 退 1、plan-failure --red 撤绿、下一趟现判；verdict-record 没跑完、带 --red 照样撤绿）。

登记表是 .claude/gate.d/stage-inputs.tsv（唯一登记位，门禁阶段与实验各占几行；写法在它文件头）。一行一条，制表符分隔：
    <键> <制表符> <入口，空格分隔；没有就写 -> [<制表符> <准入条件与跑法字段，空格分隔>] <制表符> #<为什么是这几个入口>
  键：门禁阶段写文件名（crates-mutation-replay.sh）；阶段里的一格写 <阶段文件名>:<格名>；实验写实验号（E142），同一个实验另有一种调用方式的写
      实验号加斜杠加方式名（E142/p8g-crates-dump）；崩溃枚举用例写 crash-case:<名>。
  入口（第二列）：<文件>::<限定名>（测试函数全路径、bin 的 main、判法函数）、<文件>（Python / shell 脚本的程序入口、litmus 文件）、
      tests-of:<包>（这个包 lib 与各测试目标里全部不标 ignore 的测试函数，test-functions 现列）、vm:<档名>（虚机档，门禁 qemu-device-streams 认）、
      @<别的键>（那个键登记的全部入口）；路径从仓根起。实验的入口在复跑表那一行之外另加进行键（准入键登记的装置入口）。崩溃枚举用例写 -，入口就是第三列的 test=。
  准入条件（第三列）：实验行认三种：
      question-row=<文件>#<行号>:<正则>          那份文件里第一列是 <行号> 的表格行，最后一格（状态）要匹配正则
      product-field=<实验键>:<行名>:<字段>=<值>   scripts/experiment-apparatus/replay.sh 登记给那个实验键的产物里（准入键是它的那一行），
                                                  最后一行 `E7RESULT name=<行名>` 的 <字段> 要等于 <值>
      same-run-field=<行名>:<字段>=<值>           同一趟里先跑的那一段这一趟的输出（路径由调用方经环境变量 SINGLEFS_ADMISSION_SAME_RUN_PRODUCT 交来）里，
                                                  最后一行 `E7RESULT name=<行名>` 的 <字段> 要等于 <值>；不读 research/results/ 下的留存产物（同一趟现判：research/ 可以整个删掉）。变量没设、指的文件不在、没有那一行或字段、值不等，都算前提没齐（退 3）
    question-row 与 product-field 点名的文件（那份问题单、replay.sh 给那个实验键登记的留存产物）不在仓里，登记表自查（lint_registration_rows）判红：
      前提指的是可以被归档删掉的东西，登记就靠不住；上一段的结论改用 same-run-field 在同一趟里现判。
    门禁行认五种：
      command=<可执行名>                  PATH 里找得到它
      readwrite=<路径>                    那个路径在、可读也可写（相对路径从仓库根起）
      probe=<仓内脚本>[:<参数>…]          bash <根>/<脚本> <参数…> 退 0（工作目录是仓库根）
      environment=<仓内脚本>[:<参数>…]    不是前提也不进键（工具版本不进任何判定的键，「单元键」）：自查只核点名的脚本在不在
      copy-range=<路径>                   不是前提也不进键：门禁 crates-mutation-replay 拷进每片源码副本的范围（crates-mutation-rows.py 的 copied_paths
                                          经 resolve_input_paths 读它；目录写成带斜杠的前缀）
      正则、值、路径与参数里不许有空白（条件之间按空白切）。实验行写了门禁的种类、门禁行写了实验的种类，自查都判错。
  崩溃枚举用例行（键是 crash-case:<名>，门禁 crash-point-replay 逐条跑）第三列认十一种：
      test=<包>:<测试目标>:<用例函数>     恰好一条：跑的是 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数>；
                                         包是验证包 singlefs-verification 的，-p <包> 之后另带 --no-default-features（cargo 给集成测试编目标时把本包的二进制全编一遍，
                                         验证包 src/bin/ 下的装置二进制挂缺省开着的特性，关掉缺省特性就不编；别的包不带；弄坏开关 verification-default-features-kept）；
                                         用例函数的每一处定义都不许标 #[ignore]（不带 --ignored 的 cargo test 就是它的跑法，门禁与入口按条复用那一趟用 --exclude-registered 把它排掉）：
                                         同名的每一处 fn <用例函数>( 都算（cfg 二选一的、子模块里同名的也算），有一处没标就算没标；
                                         测试目标顺着 include!("<字面路径>")、#[path = "<字面路径>"] mod、mod <名>; 带进来的源文件一起判
                                         （include! 的参数不是字符串字面量、带进来的找不到、用 use 改名 include 族宏的，判不出，按没标算）；
                                         # 与 [ 之间许空白；找不到字面的定义（宏生成的用例）判错
      count-line=<前缀>                  日志里以「<前缀> 」开头的行恰好一行
      exhaustive=<前缀>                  那一行带 exhaustive=true（前缀要先登记成 count-line=）；环境里设了缩崩溃域的展开深度变量的那一趟（CRASH_CASE_NARROWING_VARIABLES）
                                         exhaustive=false 不判、成功句写缩过域、不算门禁通过，这一行没有 exhaustive= 字段照判红
      threads=<前缀>                     按那一行的 states= 找 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行判工作线程（前缀要先登记成 count-line=）；
                                         那一行自己带 worker_threads= 与 slices= 的（CRASH_INJECTION_FINISHED 这一类）拿它自己判；
                                         那一行带 shards= 的（双机分片 merge 那一趟）逐片判，见 judge_threads_of_each_shard
      threads-variable=<环境变量名>      至多一条：用例读线程数的环境变量（没登记是 SINGLEFS_CRASH_POINT_REPLAY_THREADS）；crash-case-command 把它设成配的线程数
      features=<特性,特性…>              至多一条：用例要编进的 cargo 特性；crash-case-command 在 -p <包> 之后带 --features。多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，
                                         按 git common-dir 找）写了 ENABLE_GPU=1 的一律另加 gpu；最后的特性里有 gpu 且 ENABLE_GPU=1 的（上卡的用例），命令不经清场包装：卡与清场归 GPU 批处理验证服务
                                         （.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」：判段写待判行、等它判），环境里照调用方环境交那一份 GPU_CARDS_BY_PRIORITY（调用方没有就不设，
                                         不从多机配置里取）与多机配置的路径；
                                         验证包常编 librocksdb-sys（prune-server 是它常在的依赖），crash-case-command 一律另设 BINDGEN_EXTRA_CLANG_ARGS
                                         （取 scripts/crash/crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS），带不带 features= 都设。
      red-blocks=judged-by-test          至多一条：这条用例故障注入之后的红块是用例自己认下的（用例自己分清单、断言清单外的失败为空），门禁不按库里的红块判，
                                         照全量起用例按 cargo test 的退出码判（块那一行照 crash-case-blocks --ran 核）；值只认 judged-by-test。
                                         crash-case-judged-by-test 子命令答登记了没有；门禁 crash-point-replay 对这种用例的发现表只打印、不判红（发现日志缺节同样不判）
      shards-at-least=<正整数>           至多一条：这条用例状态多、照卡数分的片一片跑不完（.claude/rules/long-run-split.md）时登记：经服务分片的那一趟至少分这么多片。
                                         crash-case-shards-at-least 子命令打点到的用例里最大的那个数；门禁 crash-point-replay 计划那一步的片数取它与卡数的大者。
      findings=not-written-by-the-case   至多一条：这条用例的环境守卫（crash_amplification.rs 的 CrashAmplificationEnvironment）清掉了门禁给的 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE，
                                         发现日志写在用例自己的路径上或不写：用例日志里有 CRASH_POINT_REPLAY_FINDINGS 行而门禁给的那份发现日志是空的，不判红；
                                         那份发现日志写了节的照判。值只认 not-written-by-the-case。crash-case-findings-not-written 子命令答登记了没有
      blocks-line=not-printed-by-the-case  至多一条：这条用例的枚举按设计 panic 在打 CRASH_CASE_BLOCKS 那一行之前（用例 catch_unwind 认下、断言消息）：日志里一行都没有不判，判定是 cargo test 的退出码；
                                         日志里有那一行的照判。值只认 not-printed-by-the-case。crash-case-blocks-not-printed 子命令答登记了没有
      separate-run=<理由>                至多一条：这条用例拆出来单独验证（全域跑得久，不随不指定范围那一趟跑）：门禁 crash-point-replay 没有 --item 点名崩溃枚举用例的那一趟
                                         不跑它、逐条报本次未跑，--item 点名了照常跑，--list-items 照列；按条复用那一趟照旧经 --exclude-registered 排掉它的用例函数。
                                         理由写为什么拆出来、什么时候并回，不带空白（条件之间按空白切），至少 8 个字。crash-cases <根> --separate-run 答登了它的是哪几条

崩溃枚举用例的复用不在这里：用例一级没有复用标记，复用全部落在 crates 侧的崩溃状态块判定库（块键与库在 crates/singlefs-verification/src/crash_identity.rs 文件头）。
  用例跑完每条流打一行 CRASH_CASE_BLOCKS blocks=<块数> reused=<键相同、库里有判定的块> fresh=<这一趟现判的块> red=<判红的块> refused=<键算不出、照判不入库的块>
  （每一行 blocks = reused + fresh + refused；多流的用例打几行，逐行收、按用例合计；接口照 crash-case-blocks 子命令判）：
    键相同的块复用、对不上的现判；red 不是 0 判红。
    登记了 red-blocks=judged-by-test 的用例不按红块判：照平常跑起用例、按 cargo test 的退出码判，块那一行只核在不在、数对不对得上（crash-case-blocks --ran）。
    登记了 blocks-line=not-printed-by-the-case 的用例没有那一行不判（crash-case-blocks --no-line-registered）。

复跑表按行的准入键（scripts/experiment-apparatus/replay.sh 里 ADMISSION_ROW_KEYS 那一段 heredoc；replay-row-keys、experiment 与 product-field= 都读它）：
  一行一条，制表符分隔：<准入键> <制表符> <复跑表那一行的前三列，原样抄：实验号|二进制|参数> <制表符> <产物头谁写>
    产物头谁写：device  那一行跑的装置自己问准入、把 E7INPUT 行打在产物最前面（装置入口传的键要与这里的键逐字相同）；
               replay  那一行跑的东西不接准入（crates 的导出 bin 这一类），replay.sh 把准入放行时给的 E7INPUT 行接在新产物最前面。
  没写在这里的行，准入键是实验号、产物头由装置写。键的实验号要与那一行的实验号相同；每一条恰好指到复跑表一行，同一行只许指一次。
  登记了准入的键被复跑表两行以上共用判错：准入只按一个键比产物，分不清哪一行对哪一份。

复跑表每一行的行键（replay-row-inputs；scripts/experiment-apparatus/replay.sh 按行复用的键，算法 replay_row_request）：一行就是一条判定请求，入口是：
  装置    二进制写名字的行：scripts/experiment-apparatus/Cargo.toml 里 [[bin]] name 是它的那一份 path 的 main（root:scripts/experiment-apparatus、target 那个 bin）；
          二进制写 @<驱动函数> 的行：replay.sh 里那个函数（shell 单元闭包），连同驱动闭包里字面点名的 research 二进制的 main 与
          `-p <包> … --bin <名>` 点名的仓根工作区 bin 的 main（包是 scripts/experiment-apparatus/Cargo.toml 那个研究包的，--bin 的名字解到研究工作区 [[bin]] 的源码，
          root:scripts/experiment-apparatus）；`-p <包>` 点名了包却点不出 bin 的那一行拒算（replay-package-without-binary）；--bin "$变量" 写在包装函数里的，bin 取那个函数每一处调用的头一个词（replay_binaries_mentioned）；
          device-code-key.py 的 --package-directory <目录> --binary <名> 同样认（replay_device_key_binaries）
  登记    这一行的准入键在登记表里登记的入口（入口另加进来，不是登记行原文）
  判法    judge:scripts/experiment-apparatus/replay.sh::replay_judge_fresh_product 与比对前删字段的几个函数（REPLAY_JUDGING_FUNCTIONS）
  内容    line:arguments=<这一行的参数>、line:compare=<exact|timing>、驱动闭包里点到 CHECKER_TIER_FEATURES 的行另加 line:gpu=<0|1>（replay.sh 开跑前读多机配置给）
          与 验证包的 feature:singlefs-verification/gpu；file:research/results/<留存产物>（期望产物的内容）；
          驱动里 "$(replay_frozen_root <名>)" 点名的冻结根（工作根下 replay-frozen-roots/<名>，工作根取 .claude/hooks/lib_work_root.py 的 work_root）各加一行
          line:frozen-root=<名> <内容摘要>（frozen_root_digest：目录底下除 .git、target、__pycache__ 外每个文件的相对路径与字节 sha256 排序后的 sha256，目录不在写「不在」；
          冻结根里的 cargo 与编出来的装置由这一行进键，不进驱动闭包），工作根定不出的拒算 replay-frozen-root-unreadable。
  实验号、产物文件名不进键。实验的准入（experiment）用的是同一把行键去掉期望产物那一项：产物头记着键，产物进了键就对不上自己。

子命令（<根> 是被判仓的根）：
  entries <根> <键>              列出登记给这个键的入口（@ 引用已展开），一行一条；paths 同参数，只打入口所在的文件（去重）
  test-functions <根> <包> [--root <目录>] [--feature <包>/<特性>]… [--profile <名>]
                                 这个包 lib、各 bin 与各测试目标里的测试函数，一条一行：<入口><制表符><目标号><制表符><目标根文件（包目录起）>
                                 <制表符><libtest 认的全名><制表符><compiled|not-compiled|ignored>（not-compiled：这一组特性下 cfg 关着）
  test-plan <根> <种类> <包> <计划文件> [--root <目录>] [--feature <包>/<特性>]… [--profile <名>] [--token <项>]… [--exclude-registered]
                                 test-functions 与 verdict-plan 在一个进程里做（闭包只收一次）：每条编得进的测试函数一条判定（请求带 target:<目标号> 与 --token 给的项），
                                 一条一行「<名字><制表符><reuse|judge|refused|not-compiled|registered><制表符><键或 -><制表符><目标号><制表符><目标根文件><制表符><libtest 全名><制表符><原因>」，
                                 末行「PLAN judge=… reuse=… refused=… not-compiled=… registered=… ignored=…」；
                                 --exclude-registered：登记成 crash-case: 的用例（stage-inputs.tsv 的 test=）与放量分项（scripts/crash/crash-amplification.sh 的 ITEMS 的测试目标与用例函数，
                                 读法 scripts/crash/lib_crash_point_replay_items.py）不进判定，各记一行 registered（不算键、不查库、不入库，原因写交按块那一路），plan-skip-names 把它们也列进 --skip；名字是入口，同一个函数编进几个目标的另接「@<目标号>」。
                                 门禁 crash-point-replay（按条复用的两格）与 research-build-and-replay（research-unit-tests）照它按条跑 cargo test
                                 并行（records/2026-10-23-subagent拆分提案.md 第一百零六节 ①：一个进程单核算几分钟，看门狗报核数过少）：工作区读一次，
                                 之后每条测试函数「编不编（收 Rust 闭包）、编得进就算键」按测试函数交错分到每台默认重活线程数（registered_heavy_threads：
                                 SINGLEFS_THREAD_CAP 设了照它，不然 .claude/heavy-thread-default）个 fork 出来的子进程里算，按原来的次序合回来，
                                 查库、写计划文件与逐条打照旧在本进程里一条一条做：键、stdout、计划文件与一个进程算的逐字相同。
                                 线程数是 1、要算的不到两条时在本进程里算；线程数取不了（登记文件写坏、SINGLEFS_THREAD_CAP 不是正整数）整次列不出、退 2，不退回 1 个子进程
                                 （.claude/heavy-thread-default 文件头：取不了就拒）。子进程按块领活（一块约为每个子进程十六分之一的量），干完领下一块。
                                 内存：子进程读到的对象写引用计数，页被抄成私有的，整棵树是父进程加子进程数乘各自抄的量，算完一条攒着的组成清单也按子进程数乘。所以 fork 之前依次做：
                                 父进程先算一条；铺开工作区全部 Rust 单元的组成清单行、源文件行与字面串（test_plan_parent_memory_shared），子进程不各读一份；词串进 sys.intern、行号共用一个对象；
                                 把进程里全部对象的引用计数设为永生值（test_plan_parent_objects_immortal，只在 64 位 CPython 3.12：解释器不写永生对象的引用计数，别的解释器版本这一步不做，只是抄的多，键不变）；
                                 冻进永久代（到进程退出不解冻：后面查库那一趟还要 fork）。子进程交回的组成清单压成行表下标（composition_packed），要用时才解开（composition_lines）。
                                 Cargo.toml、cargo 缓存里的清单与数据文件单元一趟里按路径只读一次（test_plan_memoized_readers）。
                                 查库与存组成清单分进子进程（test_plan_settle_chunk，客户端按进程号各连各的；服务不可用照客户端的退出码退），复用哨兵与逐条打照旧在本进程按原序做；
                                 计划文件里的文件摘要同一份文件一趟只算一次（composition_files）。这些都不改闭包与键的算法，键由闭包自己的成员定。
                                 这几个函数不在判法闭包里，也不加模块级语句与带缺省值或装饰器的 def 头：这份文件是复跑行闭包的一部分（build-environment-lines 那条子命令），模块级语句随 import 进那几行的键。
                                 ADMISSION_TEST_PLAN_WORKERS_FILE（给自证与量内存）设了就往里写两行：「processes=<子进程数，本进程里算是 0> items=<算了几条> rust-closures-left=<个>
                                 computed-before-fork=<条> frozen-before-fork=<个> prefilled-units=<个> tokens-shared=<个> immortal-objects=<个> compacted=<条> reader-memo-hits=<次>
                                 worker-private-peak-bytes=<子进程算完一块时 /proc/self/smaps_rollup 的 Private_Dirty 里最大的一个>」与「file-digest-reads=<次> lookups-in-children=<条>」
  unit-key <根> <入口> [<项>…]   打「key <键>」与组成清单；拒算打拒算行、退 4
  verdict-plan <根> <种类> <请求文件> <计划文件>
                                 逐条算键、查库：stdout 一条一行「<名字><制表符><reuse|judge|refused><制表符><键或 -><制表符><原因>」，末行
                                 「PLAN judge=<N> reuse=<M> refused=<R>」；计划文件记下请求与键，交给 verdict-record。库是 <根> 的 git common-dir 那一份（样本档要在上一版的树上算键、记进这一版的库的，把上一版的树放在样本根底下）。
                                 复用哨兵（下面「复用哨兵」）那几行打在逐条的行之后、PLAN 之前；哨兵被现判时 PLAN 之后另打 ✗ 与出路、退 1。
                                 开跑先撤这份库的欠撤（上面「按条复用失灵」欠撤，test-plan 同）：撤成打一行「· 欠撤：…」，撤不掉打 ✗、退 75、不打 PLAN
  verdict-record <根> <种类> <计划文件> <判绿的名字文件> --how <跑法> [--summary <一句>] [--red-names <判红的名字文件>]
                                 名字文件里每个名字（一行一个）照计划文件里的请求重算一次键：与计划里的相同才记（跑的过程中输入变了的不记、点名报出来）；
                                 --red-names 给的名字（一行一个，这一趟真跑到而判红的）照计划里的键记红、撤掉这把键的绿（不重算：记红只会多现判）；
                                 一个名字两份文件里都有的按红办；末行「RECORD recorded=<N> changed=<K> refused=<R> red=<记红的条数> red-unrecorded=<记红没成的条数> library=<库目录>」；
                                 记红没成的每条打「✗ 判红没记进判定库：…」、退 1（上面「按条复用失灵」）
                                 计划文件旁边有 verdict-plan 存的复用哨兵状态（<计划文件>.reuse-sentinel.json）就在 RECORD 之前打哨兵的判定：
                                 强制现判那一条判绿了报与库里同为绿；没判绿的（红了或没跑到）只说一句，它的红由门禁这一趟自己报
  plan-failure <调用方> <verdict-plan|test-plan|verdict-record> <退出码> [--red <根> <种类> <计划文件> <判红的名字文件>] [<输出文件>…]
                                 那一趟按条复用失灵没有（上面「按条复用失灵」）：失灵退 1、打 ✗ 与出路（<调用方> 写进红句，例「code-tooling 的 research-script-selftests 格」）；
                                 --red 只跟 verdict-record，给那一趟交的根、种类、计划文件与 --red-names 那一份：记红没成时照它撤掉那几把键原有的绿；
                                 那一趟退 75（服务不可用）时照旧退 75，照它记欠撤（上面「按条复用失灵」欠撤）；
                                 没失灵退 0、一个字不打；用法错退 2
  按条复用的共用段（门禁 crash-point-replay 按条复用的几格与 scripts/crash/crash-point-replay.sh 按条复用的那一段都只调这三条，判定种类、请求、跳过与记库的名单全仓一份）：
  test-functions-request <阶段文件名> <判法根>
                                 按条复用的请求：一行「kind<制表符><阶段文件名>:test-functions」（两边记进、读的同一类判定库）、一行「package<制表符>singlefs-verification」、
                                 再逐个打交给 test-plan 的参数「argument<制表符><参数>」（--profile release、--token judge:scripts/gate/lib_libtest_result_lines.py::recordable_results、
                                 --token judge-root:<判法根>，再按起它那一刻的环境给 DOMAIN_ENVIRONMENT_VARIABLES 里设了的每一个打 --token variable:<名>=<值>）；用法错退 2。
                                 调用方在交给 cargo test 的同一份环境里起它（入口带上它给按条复用那一组的 env，门禁的 cargo 继承门禁自己的环境），两处取法一致
  plan-skip-names <test-plan 的输出> [--target-root <目标根文件>]… [--except-target-roots-file <文件>]
                                 cargo test 要 --skip 的 libtest 全名，一行一个（照 C 字节序）：计划里 reuse、not-compiled、registered 的，减去同名而要现判（judge、refused）的（几个二进制里同名的测试都跑，宁多跑）；
                                 给了 --target-root 只看那几个测试目标（目标根文件照 test-plan 的第五列写，例 tests/alpha.rs）的行；
                                 给了 --except-target-roots-file（一行一个目标根文件）：这些目标这一趟不起，名单里不收它们的 reuse、not-compiled、registered 行带来的名字，
                                 「减去同名而要现判的」照旧看全部目标的 judge、refused 行（门禁分片的一片只起归它的测试目标，别的片的目标不起）；读不了退 2
  plan-green-names <test-plan 的输出> <清单文件> <不记的落进的文件>
                                 这一趟现判而判绿、可记库的名字（test-plan 的第一列），一行一个：清单文件是 lib_libtest_result_lines.py recordable 的输出，按（目标根文件，libtest 全名）
                                 对到计划里 judge 的那一行，green 的打出来；withheld 的一行「<名字><制表符><不记的原因>」落进第三个参数；读不了退 2
  gate-reuse <根> <键> [<请求文件>]
                                 门禁的复用答法：给了请求文件按它逐条算键，没给按库里那个名字最近一次判绿时的请求重算；登记给这个键的入口
                                 （tests-of: 展开成测试函数）每一条都有键相同的判定 ⇒ 退 10 可跳过；否则退 0 要跑，stdout 报要现判的入口清单与「键对不上的条数」。
                                 模块自己出错一律退 0（按要跑处理）。不读 refs/sop/staged-green、不比树
  gate-preconditions <阶段所在的仓根> <阶段文件名>
                                 门禁开跑前判第三列的前提（command=、readwrite=、probe=）：退 0 齐了（stdout 一个字不打）；
                                 退 1 没齐、退 2 登记有错，两种都在 stdout 打 ✗、明细与出路，阶段照判红
  experiment <根> <实验键>       实验的准入：退 0 放行，stdout 是要写在产物最前面的几行（E7INPUT 开头，sha256= 记的是这一行的实验键）；
                                 退 77 research/results/ 里有一份产物头上记着同一个键；退 3 前提没齐；退 2 登记或调用有错。说明一律打到 stderr
  keys <根>                      列出登记表里的实验键，一行一个
  build-environment-lines <根>   构建环境的生效值（build_environment_lines）：一行一项「<名字><制表符><内容的 sha256>」，UTF-8；E171 复用库的构建环境一节按它取
                                 （起程序带这条子命令，不载入本模块）；算不出退 2，说明打到 stderr
  replay-row-keys <根>           复跑表每一行打一行「<前三列><制表符><准入键><制表符><产物头谁写><制表符><登记了准入：yes|no>」，按表里的次序
  replay-row-inputs <根> [--gpu 0|1] [--library <库名>] [--compositions <目录>] [实验号…]
                                 复跑表每一行（给了实验号只算那几个实验的行）打一行「<前三列><制表符><行键或 refused><制表符><组成清单的单元数><制表符><要不要卡><制表符><说明>」，
                                 按表里的次序；要不要卡是 1（驱动闭包里点到 CHECKER_TIER_FEATURES，与行键里加 line:gpu 的同一判）或 0，replay.sh 只在要现跑的行里有 1 时试编 gpu 特性、要服务交的卡；
                                 说明是拒算的头几处，或给了 --library（replay-row 的库名，「library-files」那一段）时与服务里那一行上一次判过的组成清单比出来的变了的单元；服务连不上退 75。--compositions 给了就把每把键的组成清单写成 <目录>/<键>.tsv（replay.sh 记库时拷进那一条判定）
                                 并行（replay_row_shell_index_prefilled）：只点几行的也要全部脚本的 shell 索引（source 的边要看别的文件），这一段分到每台默认重活线程数个 fork 子进程里建
                                 （shell_index_filled，与 ShellIndex 建的逐项相同）；线程数是 1 或取不了、进程池没跑完（标准错误打一行 !）时在本进程里建，行键不变。
                                 ADMISSION_REPLAY_ROW_WORKERS_FILE（给自证与量时间）设了就往里写一行「processes=<子进程数，本进程里建是 0> files=<份> rounds=<趟>」
                                 脚本闭包（KeyContext.script_closure）按（根、入口、program-argument）在一个 KeyContext 里只收一次：每行的驱动闭包取驱动函数原文找 bin 与取行键各要一遍，
                                 一遍收完两处共用；共用只在这一个 KeyContext 里（一趟），下一趟新建的重收。
  library-files fetch <库名> <中转目录> [<相对前缀>…]
  library-files store <库名> <中转目录> [<相对路径>…]
  library-files has <库名> <相对路径>…
                                 文件头「library-files」：fetch 末行 LIBRARY_FILES_FETCH entries=<条数>，store 末行 LIBRARY_FILES_STORE written=<写了> skipped=<逐字节相同跳过>，
                                 has 一行打一条服务里有的相对路径；库名写错、中转目录不在（store）退 2，服务不可用退 75
  crash-cases <根> [--separate-run]
                                 核登记的崩溃枚举用例（第三列的写法、包与测试目标在不在、用例函数在不在、它有没有标 #[ignore]），每条打一行
                                 「<键><制表符><包><制表符><测试目标><制表符><用例函数>」；登记有错退 2，原因打在 stdout（不带 ✗，阶段自己打 ✗ 与出路）。
                                 带 --separate-run 时自查照旧，只给登了 separate-run= 的那几条（上面「崩溃枚举用例行」）各打一行「<键><制表符><理由>」，照登记表的次序，
                                 一条都没登记就一行不打、退 0
  crash-case-command <根> <键> [--share <片号>/<片数>] [--stage record|judge]
                                 跑这一条的命令与环境：stdout 以 NUL 分隔，本机核数、线程数、explicit|default，其后是整条命令
                                 （env 设 SINGLEFS_CRASH_POINT_REPLAY_THREADS，登记了 threads-variable= 的另把那个变量设成同一个数；
                                 清掉调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_SHARE 与给定状态引擎的
                                 SINGLEFS_CRASH_AMPLIFICATION_STAGE、SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS、SINGLEFS_CRASH_LIBRARY_RUN_KIND；再起 cargo test --release …，features= 与 GPU 照上面「崩溃枚举用例行」）。
                                 带 --share 的（门禁 crash-point-replay 的一片，它文件头「分片」）把 SINGLEFS_CRASH_AMPLIFICATION_SHARE 设成给的这一份，引擎只判这一片领的块、
                                 直接写进 prune-server；不带的整份（0/1），即合并那一步的核账。<片号> 小于 <片数>、都是十进制数，写错退 2。
                                 带 --stage 的（录入段与判段分两次起的调用方用；门禁 crash-point-replay 不带它，两段在同一趟里先后跑，它文件头「显卡」）在清掉调用方那一份之后把
                                 SINGLEFS_CRASH_AMPLIFICATION_STAGE 设成给的值：record 只跑录入段（CPU 上录、写进库），judge 只跑判段（从库里读回判）；不带的两段先后跑。
                                 值不是 record、judge 退 2。两个选项次序不限
                                 线程数：调用方设了 SINGLEFS_CRASH_POINT_REPLAY_THREADS 用它（explicit），没设取每台默认重活线程数（default：SINGLEFS_THREAD_CAP 设了照它，否则
                                 .claude/heavy-thread-default 的登记值）；本机核数另取 os.cpu_count() 与 CPU 亲和的小者（判「本机多于 1 核」），不认 OMP_NUM_THREADS。交不出退 2
  crash-case-judge <根> <键> <日志> <记录行文件>
                                 判跑那一条的日志：退 0 判绿（日志里认出的行写进记录行文件，stdout 是线程那一句，缩过域的那一趟另有逐个计数行不判 exhaustive=true 那一句）；退 1 判红（stdout 逐条原因）。
                                 本机核数、线程数与是不是显式设的由它现取；日志里线程来源是用例自己取的默认（worker_threads_source=available_parallelism）
                                 而配的线程数不等于每台默认重活线程数的，判红。缩没缩域由它从环境现取（CRASH_CASE_NARROWING_VARIABLES，设了就算，与门禁 crash-point-replay 收尾那一判同一个算法）
  crash-case-blocks <根> <日志> [--ran] [--no-line-registered] [--recorded-only]
                                 判日志里的 CRASH_CASE_BLOCKS 行（每条流一行，逐行收、按用例合计）：退 0 无红的块；
                                 --recorded-only（只录的那一趟，crash-case-command --stage record 起的）：已录、待判，不判红也不当判过——
                                 一行都没有、或每一行现判 0、红 0（数对得上）退 0，成功句写「只录：已录、待判」；有一行现判或红不是 0（录入段判了块）、缺字段、数对不上退 4。
                                 带它时 --ran、--no-line-registered 不起作用（判段那一趟照它们判）；
                                 --no-line-registered（跑登记了 blocks-line=not-printed-by-the-case 的用例那一趟）：一行都没有退 0，有的照下面判；
                                 退 1 有红的块；
                                 退 4 一行都没有、有一行缺字段或数对不上（每一行 blocks = reused + fresh + refused）、平常跑按用例合计 blocks = 0（引擎只录、没编进 gpu 特性、
                                 表全留给别的机器时打 blocks=0，一块都没判也没查，行不对）。
                                 --ran（跑登记了 red-blocks=judged-by-test 的用例那一趟）：只核那一行在、每一行数对得上、合计 blocks 不是 0，红块与现判块不判：
                                 退 0 对得上，退 4 一行都没有、缺字段、数对不上或合计 blocks = 0。
  crash-case-judged-by-test <根> <键>
                                 这条用例登记了 red-blocks=judged-by-test 没有：退 0 登记了、退 1 没登记（stdout 各一句）、退 2 登记有错
  crash-case-shards-at-least <根> [<键>…]
                                 点到的崩溃枚举用例（不给键是全部登记的）里登记的 shards-at-least= 最大是几：stdout 一行一个数，一条都没登记打 0，退 0；
                                 键不在或登记有错退 2
  crash-case-findings-not-written <根> <键>
                                 这条用例登记了 findings=not-written-by-the-case 没有：退 0 登记了、退 1 没登记（stdout 各一句）、退 2 登记有错
  crash-case-blocks-not-printed <根> <键>
                                 这条用例登记了 blocks-line=not-printed-by-the-case 没有：退 0 登记了、退 1 没登记（stdout 各一句）、退 2 登记有错
  --selftest                     自证（格数由成功行现算）
                                 选格：ADMISSION_SELFTEST_CELLS=<格名,…> python3 scripts/gate/admission.py --selftest（经临时服务起，同不选格的那一趟）只跑点名的格，单跑一格与它在全部格里跑判定相同；
                                 格是 run_selftest 的 cells 那张表的键，每一格是一个 run_*_cells 函数，只用自己建的临时仓与临时目录，失败行写成「✗ 自证：<格名>：<那一项的句子>：<说明>」；
                                 格名写错退 2 并列出认得的格。门禁 code-tooling 的 break-switch-replay 格指定范围时经它每条开关只起点名的格（下面开关表「自证：<格名>：」那一项）；
                                 下面开关表里没写「自证：<格名>：」的开关（点名的句子落在几格共用的函数里，读不出落在哪一格）那一趟整份起。不设这个变量跑全部格

没有强制重跑：实验键与上一份产物头相同就退 77，没有越过「输入没变」的变量或参数；要重跑就改输入（装置入口闭包、复跑表那一行的参数），键变了自然放行。
弄坏开关（ADMISSION_BREAK=<项>[,<项>…]，只给自证与样本档证红用：--selftest 本身与它起的子进程认，门禁样本根（当前目录有 .gate-cells 或 .gate-break-sample）里由阶段转来的认，
平常跑设了退 2、点名只在自证里认。逐条写开关名、弄坏哪一处，下一行起写打开后红的项：`ADMISSION_BREAK=<项> python3 scripts/gate/admission.py --selftest` 里逐条以 `✗ ` 起头的失败行（写成 `✗ 自证：<格名>：<句子>：<说明>`）里 `<句子>` 的句首，
「⇒」之前的那一段；里面再带「」的要配平，没配平的那一项连同同一条后面几项门禁 code-tooling 的 break-switch-replay 格都认不出）：
  ADMISSION_BREAK=selftest-cells-run-extra  点名了格时 runner-script 那一格也跑
      「自证：cell-selection：选格：点名 composition-packing」
  ADMISSION_BREAK=selftest-cells-unknown-accepted  ADMISSION_SELFTEST_CELLS 里写错的格名不退 2
      「自证：cell-selection：选格：写错的格名退 2」
  ADMISSION_BREAK=experiment-key-with-kb-documents  .claude/kb/decisions/ 下的文件整份进键（实验准入收 kb 决策文档）
      「reuse-sentinel：哨兵 1 条、种子 7 那一趟退 0、种子打进输出、一条强制现判（verdict-reco」
      「实验准入：T19（kb 决策文档改一句）⇒ 实验键不变，照判没变（退 77）：退 0」「实验准入：N22（登记表加注释、别的实验行多登记入口）⇒ 照判没变（退 77）：退 0」
      「给 doc-experiments：产物头的实验键对得上时状态是 matched：实际 stale」
  ADMISSION_BREAK=tree-compare-reuse  gate-reuse 退回按树比：refs/sop/staged-green 那棵树（没有它取 HEAD 的树）与工作区在登记入口的文件上逐字相同就答可跳过，不查判定库
      「自证：verdict-library：」
      「gate-reuse：H2（改了 helper 并提交、refs/sop/staged-green 也挪到这一棵）⇒ 退」
  ADMISSION_BREAK=skip-preconditions  实验不判前提
      「自证：experiment：」「自证：replay：」「自证：replay-row-key：」
      「实验准入：前提没齐时拒绝（退 3）并点名那一行：退 0，stderr：✓ 放行 E900/second：上一份产物头上没」「same-run-field：没交同一趟的输出时前提没齐（退 3）并点名那个环境变量：退 0，stderr：✓ 放行 E」
      「same-run-field：同一趟的输出里值不等时前提没齐（退 3），留存产物里的 control_ok=true 不」「same-run-field：同一趟的输出里值相等时放行，说明点名那份输出：退 0，stderr：✓ 放行 E900/t」
      「replay.sh：前提没齐时判跑不了（准入退 3）、装置一次都没跑：退 0，输出：」「复跑表按行的准入键：product-field=E903 只认键是 E903 的那一行产物（复跑表里 E903 两行），」
  ADMISSION_BREAK=skip-gate-preconditions  门禁不判前提
      「自证：gate-precondition：」
      「门禁前提 command= 缺了判红（退 1）：点名那个可执行名，带出路与那一行的注释：退 0，stdout「」」「门禁前提 readwrite= 不可写判红（退 1）并点名那个路径（以 root 跑这一格判不出）：退 0，stdout」
      「门禁前提 probe= 退非 0 判红（退 1），明细带出探针自己打的那几行：退 0，stdout「」」
  ADMISSION_BREAK=skip-unchanged  实验准入不比实验键，一律当输入变了
      「自证：experiment：」「自证：replay：」
      「实验准入：老产物头上没有实验键时放行并报判不了：退 0，stderr：✓ 放行 E900：（弄坏开关 skip-unch」「实验准入：存了带这把实验键的产物、复跑表指到它」
      「实验准入：装置 main 改一个字」「实验准入：装置 main 只加一行注释」
      「实验准入：T19（kb 决策文档改一句）⇒ 实验键不变，照判没变（退 77）：退 0」「实验准入：N22（登记表加注释、别的实验行多登记入口）⇒ 照判没变（退 77）：退 0」
      「replay.sh：留存产物头上的实验键与今天的相同时判「输入没变」、装置一次都没跑、汇总报没跑 1 行（只改复跑表里的」
  ADMISSION_BREAK=same-run-field-unchecked  same-run-field= 不读同一趟的输出、一律当齐了
      「自证：experiment：」
      「same-run-field：没交同一趟的输出时前提没齐（退 3）并点名那个环境变量：退 0，stderr：✓ 前提齐了」「same-run-field：同一趟的输出里值不等时前提没齐（退 3），留存产物里的 control_ok=true 不」
      「same-run-field：同一趟的输出里值相等时放行，说明点名那份输出：退 0，stderr：✓ 前提齐了：（弄坏开」
  ADMISSION_BREAK=condition-files-unchecked  登记表自查不核 question-row=、product-field= 点名的文件在不在
      「自证：experiment：」
      「登记表自查点得出前提点名的文件不在：question-row 的问题单、product-field 指的留存产物：自查结」
  ADMISSION_BREAK=skip-build-environment  构建环境的生效值不进工具链行（RUSTFLAGS 这一类）
      「单元键：RUSTFLAGS 加 -C target-cpu=native（E02）」「单元键：.cargo/config.toml 改 [build] rustflags 的值」
      「单元键：cargo 配置 runner 写 `bash tools/runner.sh`，参数里的脚本（.cargo 那一层下）改一个字」
      「单元键：cargo 配置 runner 写 `bash package-runner.sh`，脚本只在登记用例的包目录 crates/demo 下，改一个字」（runner 指的脚本的内容摘要也在构建环境那几行里）
  ADMISSION_BREAK=build-environment-lines-names-only  子命令 build-environment-lines 只打名字、不打内容的 sha256
      「自证：unit-key：」
      「子命令 build-environment-lines：每行「名字<制表符>内容的 sha256」，与 build_environment_lines 逐项相同、不空」
  ADMISSION_BREAK=ignore-row-keys  复跑表按行的准入键那一段不认
      「replay.sh：老产物没有产物头、新跑的有，比对前删掉产物头之后判字节一致：退 1，输出：」「弄坏 replay.sh 比对前删产物头那一步（换成 cat）判对不上：上一格分得出差别：退 1，输出：」
      「replay.sh：留存产物头上的实验键与今天的相同时判「输入没变」、装置一次都没跑、汇总报没跑 1 行（只改复跑表里的」「复跑表按行的准入键：两行各给键、各自登记时两行都跑、都判字节一致；第一行产物头是装置打的 key=E903，第二行是 r」
      「E903 登记了准入，而这张复跑表里它有 2 行：准入只按实验号比产物，分不清哪一行对哪一份」「定不下复跑表每一行的准入键（python3 scripts/gate/admission.py replay-row-k」
      「复跑表按行的准入键：--admission-only 逐行列出各自的键与判定，一行都没跑：退 2，跑了 []，输出：」「复跑表按行的准入键：两行各给键而其中一个没在登记表登记时整份退 2，点名那个键，一行都没跑：退 2，跑了 []，输出：」
      「复跑表按行的准入键：指到复跑表里没有的行时整份退 2，点名那一条：退 2，跑了 []，输出：」「复跑表按行的准入键：product-field=E903 只认键是 E903 的那一行产物（复跑表里 E903 两行），」
      「真仓 replay.sh 复跑表每一行定得下准入键：按行的准入键那一段写得对、指明的键都登记了、没有登记了准入的键被两行」
      「replay.sh：起自证的环境里设着多机配置变量（指到一份在的配置）时，样本仓的 replay.sh 照样跑、判字节一致」
  ADMISSION_BREAK=replay-build-tools-unlinked  自证的临时仓不链 scripts/hosts/two-logs.sh 与 two_logs.py（replay.sh 开跑前的构建经它们起），要真跑 replay.sh 的几格构建退 127、replay.sh 整份退 2；
      「自证：replay：」「自证：replay-row-key：」
      一行都不用跑的几格（准入没放行、复跑表按行的准入键写错、--admission-only）走不到构建，不红
      （失败行里输出那一段是 replay.sh 输出的末 600 字，起头随临时目录的长短挪，点名的原文只写到冒号为止）
      「replay.sh：老产物没有产物头、新跑的有，比对前删掉产物头之后判字节一致：退 2，输出：」「弄坏 replay.sh 比对前删产物头那一步（换成 cat）判对不上：上一格分得出差别：退 2，输出：」
      「replay.sh：留存产物头上的实验键与今天的相同时判「输入没变」、装置一次都没跑、汇总报没跑 1 行（只改复跑表里的」
      「replay.sh：登记了准入、装置却没打实验键行时判跑不了：退 2，输出：」
      「replay.sh：起自证的环境里设着多机配置变量（指到一份在的配置）时，样本仓的 replay.sh 照样跑、判字节一致：退 2，输出：」（同一棵样本仓、同一行 E900，构建照样退 127）
      「复跑表按行的准入键：两行各给键、各自登记时两行都跑、都判字节一致；第一行产物头是装置打的 key=E903，第二行是 r」
      「replay.sh：起自证的环境里设着多机配置变量（指到一份在的配置）时，样本仓的 replay.sh 照样跑、判字节一致」
  ADMISSION_BREAK=row-key-with-product-name  行键另加留存产物的文件名（门禁 research-build-and-replay 的样本 replay-row-key-samples 里 R03 红）
      「行键：R03 E23 产物改名（内容相同）」
  ADMISSION_BREAK=row-key-without-gpu  行键不加 GPU 开关
      「行键：E22 切 GPU 开关（--gpu 1）」
  ADMISSION_BREAK=frozen-root-unkeyed  驱动点名的冻结根（"$(replay_frozen_root <名>)"）的内容摘要不进行键
      「自证：row-key：」
      「行键：驱动点名冻结根 ⇒ 改冻结根里一个文件的一个字节，变的只有 E900」
  ADMISSION_BREAK=research-package-unresolved  驱动点名 -p <研究包> --bin <名> 时照仓根工作区去找 bin
      「自证：row-key：」
      「行键：驱动点名 -p e7-index-bench --bin e900（研究工作区的包）⇒ E900 算得出行键、不拒」
  ADMISSION_BREAK=test-key-with-whole-file  test-plan 给每条测试函数的键另加它所在的整份文件（门禁 crash-point-replay 的 GATE_CRASH_POINT_REPLAY_BREAK=setting-file-whole-closure 转成它，红在那一道的样本 unit-key-samples-green 里的 N19）
      「自证：test-plan-closure-key：」
      「test-plan：同一设点文件里只改 test_t 闭包里的 observer」
  ADMISSION_BREAK=exclude-registered-ignored  test-plan 不认 --exclude-registered（登记的用例与放量分项照判）
      「自证：test-plan-exclusion：」
      「test-plan --exclude-registered：登记成 crash-case: 的用例与放量分项记 registered」
  ADMISSION_BREAK=skip-names-of-dropped-targets-kept  plan-skip-names 不看 --except-target-roots-file：不起的目标的名字照收进 --skip 名单
      「自证：plan-skip-names：」
      「plan-skip-names --except-target-roots-file：不起的目标（tests/dropped.rs）的名字不收」
  ADMISSION_BREAK=test-plan-single-process  test-plan 算键在一个进程里一条一条算
      「自证：test-plan-parallel：」
      「test-plan：同一份输入单线程与并行算出的 stdout 与计划文件逐字相同，并行那一趟真分到了子进程：单线程退」
      「test-plan：并行之前父进程先算一条，再把对象冻进永久代（子进程共用判法索引与工具链行）」
      「test-plan：并行之前父进程铺好全部单元的行表、源文件行与字面串，子进程不各读一份」
      「test-plan：并行之前词串进驻留表、行号共用一个对象」
      「test-plan：并行之前把父进程的对象标成永生，子进程读它们不写引用计数」
      「test-plan：子进程交回的组成清单是压过的下标串」
      「test-plan：查库与存组成清单分进子进程」
  ADMISSION_BREAK=test-plan-closures-kept  test-plan 每条算完键不放掉 Rust 闭包，一趟的闭包全攒在 rust_closures 里
      「自证：test-plan-parallel：」
      「test-plan：每条算完键放掉它的 Rust 闭包，单线程那一趟算完 rust_closures 是空的」
  ADMISSION_BREAK=test-plan-parent-not-prewarmed  test-plan 并行之前父进程不先算一条、不把对象冻进永久代
      「自证：test-plan-parallel：」
      「test-plan：并行之前父进程先算一条，再把对象冻进永久代（子进程共用判法索引与工具链行）」
  ADMISSION_BREAK=test-plan-lines-not-prefilled  test-plan 并行之前父进程不铺开工作区全部单元的组成清单行、源文件行与字面串，子进程各读各算
      「自证：test-plan-parallel：」
      「test-plan：并行之前父进程铺好全部单元的行表、源文件行与字面串，子进程不各读一份」
  ADMISSION_BREAK=test-plan-tokens-unshared  test-plan 并行之前不把词串进驻留表、行号不共用对象
      「自证：test-plan-parallel：」
      「test-plan：并行之前词串进驻留表、行号共用一个对象」
  ADMISSION_BREAK=test-plan-objects-not-immortal  test-plan 并行之前不把父进程的对象标成永生，子进程读到就抄页
      「自证：test-plan-parallel：」
      「test-plan：并行之前把父进程的对象标成永生，子进程读它们不写引用计数」
  ADMISSION_BREAK=test-plan-compositions-whole  test-plan 的子进程交回整份组成清单（行的列表），不压成行表下标
      「自证：test-plan-parallel：」
      「test-plan：子进程交回的组成清单是压过的下标串」
  ADMISSION_BREAK=test-plan-packing-literals-moved  组成清单解开时行表外的那几行挪到最前面（次序错）
      「自证：composition-packing：」
      「test-plan：组成清单压完再解开逐行相同，文件名单也相同」
  ADMISSION_BREAK=test-plan-file-readers-unmemoized  test-plan 里 Cargo.toml、cargo 缓存里的清单与数据文件单元每条测试函数各读一遍
      「自证：test-plan-parallel：」
      「test-plan：一趟里清单与数据文件按路径只读一次」
  ADMISSION_BREAK=test-plan-file-digests-reread  test-plan 写计划文件时同一份文件每条测试函数各算一遍摘要
      「自证：test-plan-parallel：」
      「test-plan：计划文件里的文件摘要同一份文件一趟只算一次」
  ADMISSION_BREAK=test-plan-lookups-in-parent  test-plan 查库与存组成清单在本进程里一条一条做
      「自证：test-plan-parallel：」
      「test-plan：查库与存组成清单分进子进程」
  ADMISSION_BREAK=replay-row-shell-index-serial  replay-row-inputs 的 shell 索引在本进程里建
      「自证：replay-row-parallel：」
      「replay-row-inputs：单线程与并行建脚本索引行键逐字相同，并行那一趟真分到了子进程」
  ADMISSION_BREAK=shell-index-rounds-one  脚本索引那一格让进程池建的 source 的边只算一趟，不算到不再变（变量赋在被 source 的那份里的折不出）
      「自证：shell-index-filled：」
      「脚本索引：进程池建的与一个进程建的逐项相同，source 的边算到不再变」
  ADMISSION_BREAK=composition-lines-unshared  组成清单的单元行每份现拼一个新串，不按（前缀, 单元）共用
      「自证：composition-line-sharing：」
      「组成清单：两份闭包里同一个单元的那一行共用一个串对象，内容与现拼的相同，带前缀的另算」
  ADMISSION_BREAK=script-closures-recollected  KeyContext.script_closure 不记收过的闭包，同一个入口每问一次收一遍
      「自证：script-closure-sharing：」
      「脚本闭包：同一个 KeyContext 里同一个入口只收一次，再问交同一个对象」
  ADMISSION_BREAK=script-closures-process-wide  KeyContext.script_closure 收过的闭包记在进程里（按根、入口），不随 KeyContext，新建的 KeyContext 也拿旧的
      「自证：script-closure-sharing：」
      「脚本闭包：改了入口所在文件的内容、新建 KeyContext 再问，闭包里的单元与键跟着变」
  ADMISSION_BREAK=test-plan-jobs-fallback-one  test-plan 取不了每台默认重活线程数时退回 1 个子进程照算
      「自证：test-plan-parallel：」
      「test-plan：取不了每台默认重活线程数（SINGLEFS_THREAD_CAP=abc）⇒ 整次列不出、退 2，不退回 1 个子进程照算」
  ADMISSION_BREAK=dependencies-by-crate  依赖版本行（KeyContext.dependency_lines，不进单元键）里外部依赖照包收（闭包碰到一个工作区包就收它 Cargo.toml 写的全部外部依赖，不看单元点没点它的名字）
      「单元键：Cargo.lock 里 formats 用到的 itoa 抬版本（S14 形态）」
  ADMISSION_BREAK=rocksdb-entry-unchecked  crash-case-command 交命令之前不调 scripts/hosts/rocksdb-prebuilt.py entry
      「自证：crash-cases-listing：」
      「crash-case-command：验证包依赖 rocksdb 而进程环境里没有预编库的取值」
  ADMISSION_BREAK=blocks-count-unchecked  crash-case-blocks 不核 blocks = reused + fresh + refused
      「crash-case-blocks：多流用例里一行数对不上」「crash-case-blocks：blocks 不等于 reused + fresh + refused」
      「crash-case-blocks：--ran：blocks 不等于 reused + fresh + refused」
  ADMISSION_BREAK=zero-blocks-covered  crash-case-blocks 放行按用例合计 blocks = 0 的日志
      「crash-case-blocks：平常跑：blocks = 0（一块都没判）」
      「crash-case-blocks：多流用例两行合计 blocks = 0」「crash-case-blocks：--ran：blocks = 0（一块都没判）」
  ADMISSION_BREAK=keep-caller-stage-variables  crash-case-command 起用例时不清调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_STAGE 与 SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS
      「自证：crash-cases-listing：」
      「crash-case-command：调用方环境里设着给定状态引擎的阶段 record 与各台权重 1,3」
  ADMISSION_BREAK=case-stage-not-set  crash-case-command 收了 --stage 也不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE（只录那一趟照两段先后跑、录的时候就判）
      「自证：crash-cases-listing：」
      「crash-case-command --stage record」「crash-case-command --stage judge」
  ADMISSION_BREAK=recorded-only-judged  crash-case-blocks 不认 --recorded-only，只录那一趟照平常跑判（没有块那一行、合计 0 块都退 4）
      「自证：crash-case-blocks：」
      「crash-case-blocks：--recorded-only：日志里没有那一行」「crash-case-blocks：--recorded-only：块那一行现判 0、红 0」「crash-case-blocks：--recorded-only：块那一行现判 1（录入段判了块）」
  ADMISSION_BREAK=blocks-count-without-refused  crash-case-blocks 核 blocks = reused + fresh（拒算的块不算进去）
      「crash-case-blocks：平常跑：拒算 1 块计在 refused、数对得上」
  ADMISSION_BREAK=blocks-single-line-only  crash-case-blocks 要日志里恰好一行
      「crash-case-blocks：多流用例每条流一行：逐行收、按用例合计」
      「crash-case-blocks：多流用例里一条流有红」
      「crash-case-blocks：多流用例两行合计 blocks = 0」「crash-case-blocks：多流用例一条流 0 块、另一条流 2 块有判定：合计不是 0」
      「crash-case-blocks：多流用例的一句报两条流合计的块数（块 5、复用 4、现判 1）：日志里以」「crash-case-blocks：--ran：多流用例两行合计」
  ADMISSION_BREAK=ran-blocks-line-optional  crash-case-blocks --ran 日志里一行 CRASH_CASE_BLOCKS 都没有也退 0
      「crash-case-blocks：--ran：日志里没有那一行」
  ADMISSION_BREAK=blocks-line-required-when-marked  crash-case-blocks 不认 --no-line-registered，登记了 blocks-line=not-printed-by-the-case 的用例日志里没有那一行也退 4
      「crash-case-blocks：--no-line-registered：日志里没有那一行」
  ADMISSION_BREAK=exhaustive-judged-when-narrowed  crash-case-judge 不看缩域变量，缩过域的那一趟也判 exhaustive=true
      「crash-case-judge 缩域：设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5、那一行 exhaustive=false」「crash-case-judge 缩域：设了 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3、那一行 exhaustive=false」
      「crash-case-judge 缩域：设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5、那一行 exhaustive=true」
  ADMISSION_BREAK=exhaustive-unjudged-without-narrowing  判用例日志不看缩没缩域，登记了 exhaustive= 的计数行一律不判 exhaustive=true
      「判用例日志：不是全量（exhaustive=false）」「真仓登记的 crash-case:c561-sigma-full：计数行 exhaustive=false（状态数不等于闭式）」
      「crash-case-judge 缩域：没设缩域变量、那一行 exhaustive=false」「crash-case-judge 缩域：设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5、那一行没有 exhaustive= 字段」
  ADMISSION_BREAK=verdict-key-with-entry-file  verdict-plan 每条请求的键另加入口所在的整份文件（闭包外的改动也作废）
      「自证：reuse-three-question：」「自证：verdict-library：」
      「gate-reuse：登记的三条在库里都有键相同的判定」「gate-reuse：H2（改了 helper 并提交、refs/sop/staged-green 也挪到这一棵）⇒ 退」
      「gate-reuse：H3（空提交，HEAD 前移、内容回到判过的样子）⇒ 退 10 照复用：退 0：demo-gate」「reuse-three-questions：verdict-plan 头一趟三条现判、同一份输入第二趟全复用、闭包外加一」
      「gate-reuse：E01（只换工具链）⇒ 退 10 照复用（工具链版本不进键）：退 0：」
      「reuse-sentinel：哨兵 1 条、种子 7 那一趟退 0、种子打进输出、一条强制现判（verdict-reco」
  ADMISSION_BREAK=verdict-lookup-skipped  VerdictLibrary.lookup 不读库、一律现判（复用失灵的样子）
      「自证：reuse-fix：」「自证：reuse-three-question：」「自证：verdict-library：」
      「verdict-plan：记过之后第二趟三条都 reuse：{'test_t': 'judge', 'test_u':」「复用失灵：crates/demo/tests/setting.rs::test_v 键相同却现判了（复用哨兵（demo-」
      「verdict-plan：改了被调函数 helper」「gate-reuse：登记的三条在库里都有键相同的判定」
      「gate-reuse：不给请求文件（tests-of:demo 展开、按库里记的请求重算）⇒ 同样退 10：退 0：de」「gate-reuse：H2（改了 helper 并提交、refs/sop/staged-green 也挪到这一棵）⇒ 退」
      「gate-reuse：H3（空提交，HEAD 前移、内容回到判过的样子）⇒ 退 10 照复用：退 0：demo-gate」「reuse-three-questions：verdict-plan 头一趟三条现判、同一份输入第二趟全复用、闭包外加一」
      「gate-reuse：E01（只换工具链）⇒ 退 10 照复用（工具链版本不进键）：退 0：」
      「复用失灵：crates/demo/tests/setting.rs::test_v 键相同却现判了（复用哨兵（reuse」「reuse-sentinel：哨兵 1 条、种子 7 那一趟退 0、种子打进输出、一条强制现判（verdict-reco」
      「复用失灵：crates/demo/tests/setting.rs::test_u 键相同却现判了（复用哨兵（reuse」
      「判定库：同一把键判绿入库、后来判红（--red-names）⇒ 下一趟这一条现判、原因以「上次判红」起头」「判定库：红之后再判绿 ⇒ 下一趟复用」
      「判定库：记红没成（样本标记 red-record-failure）⇒ verdict-record 退 1、plan-failure --red 撤掉原有的绿、下一趟这一条现判」
      「判定库：verdict-record 没跑完（退 137、没有 RECORD 那一行）⇒ plan-failure --red 照样撤绿、判红、下一趟这一条现判」
      「判定库：记红时 prune-server 不可用（样本标记 red-record-unavailable）⇒ plan-failure --red 照旧退」（撤之前那一趟要全部复用、撤完那一趟要只 test_u 现判，不读库都对不上）
      「判定库：欠撤撤不掉（样本标记 owed-withdrawal-unavailable）⇒ verdict-plan 整道退 75、不打 PLAN、欠撤留着；」（撤得掉之后要只 test_u 现判，不读库三条都现判）
  ADMISSION_BREAK=judge-only-entry-judges-unkeyed  入口写 - 的请求 judge: 不进键（只按 line:、file: 算）
      「自证：unit-key：」
      「单元键：入口写 -（键只由判法与内容输入组成）」
      「单元键：判法入口就在 lib_function_units.py 里的：改它调到的函数键变（入口与 judge: 都算）」
      「单元键：改别的判定自己的判法：键变（入口与 judge: 都算）」
  ADMISSION_BREAK=program-arguments-unkeyed  单元键不加 program-arguments 那一行（参数不同、闭包相同的两条判定共用一把键）
      「自证：unit-key：」
      「单元键：program-argument: 进键：同一程序入口两组参数各算得出、键不同」
      「单元键：shell 入口不按参数分支、两组 program-argument: 闭包相同：各算得出、键不同」
  ADMISSION_BREAK=shell-program-arguments-dropped  shell 入口（只写 <文件>）的闭包不带请求的 program-argument:，脚本体整份收
      「自证：unit-key：」
      「单元键：shell 入口带 program-argument:run，--selftest 那一支起的脚本改了：带 run 的键不变、带 --selftest 的键变」
  ADMISSION_BREAK=red-verdict-reused  VerdictLibrary 读库时判定不是绿的记录也当判定读（scripts/crash/crash-point-replay.sh 的 CRASH_POINT_REPLAY_BREAK=red-reused-as-green 转成它，那一份自证的 red-rejudged 格也红）
      「自证：reuse-fix：」
      「判定库：同一把键判绿入库、后来判红（--red-names）⇒ 下一趟这一条现判、原因以「上次判红」起头」
  ADMISSION_BREAK=superseded-composition-kept  挤出最近几把、换下来的键的组成清单一把都不删（打开后组成清单只增不减）
      「组成清单的回收」
  ADMISSION_BREAK=red-verdict-not-recorded  verdict-record 收了 --red-names 也不记红、不撤绿（只记绿）
      「自证：reuse-fix：」
      「判定库：同一把键判绿入库、后来判红（--red-names）⇒ 下一趟这一条现判、原因以「上次判红」起头」
  ADMISSION_BREAK=replay-package-mention-dashes  驱动正文里 -p 后面的词首字符也认 -（mkdir -p -- 的 -- 当包名，那一行拒算 replay-package-without-binary）
      「自证：replay-package-mention：」
      「复跑表的行键：驱动里 mkdir -p -- 的 -- 不当包名，-p <包> 照认」
  ADMISSION_BREAK=replay-bin-wrappers-unread  --bin "$变量" 写在包装函数里的不认（那一行拒算 replay-package-without-binary）
      「自证：replay-package-mention：」
      「复跑表的行键：--bin "$变量" 写在包装函数里：认出每一处调用点名的 bin」
  ADMISSION_BREAK=replay-device-key-binaries-unread  device-code-key.py 的 --package-directory <目录> --binary <名> 不认（装置的 Rust 入口不进行键）
      「自证：replay-package-mention：」
      「复跑表的行键：device-code-key.py run --package-directory <目录> --binary <名>：认出包目录与 bin」
  ADMISSION_BREAK=not-judging-files-keyed  NOT_JUDGING_FILES 两份的单元照进别的判定的键，也从它们往下连
      「自证：unit-key：」
      「单元键：改 lib_function_units.py 里的代码、改只经它起的程序：别的判定的键不变（入口与 judge: 都算）」
  ADMISSION_BREAK=not-judging-entries-excluded  入口本身在 NOT_JUDGING_FILES 两份里的也排除（它调到的同一份里的函数不进键，它们自己的判定键不随它们变）
      「自证：unit-key：」
      「单元键：判法入口就在 lib_function_units.py 里的：改它调到的函数键变（入口与 judge: 都算）」
  ADMISSION_BREAK=replay-store-inside-judge  自证往拷来的 replay.sh 的 replay_judge_fresh_product 开头接回一句（判完当场起 admission.py library-files store 存库）
      「自证：replay-judging-closure：」
      「replay.sh 判法闭包：REPLAY_JUDGING_FUNCTIONS 每个都算得出键、没有拒算、闭包里没有 admission.py 的单元（存库不在判法里）」
  ADMISSION_BREAK=library-outside-git-tolerated  被判的根取不到 git common-dir 时 VerdictLibrary 不退 2、没有库（store 是 None）往下走
      「自证：reuse-fix：」
      「判定库：不是 git 工作树的副本里判，verdict-plan 与 verdict-record 都退 2、点名那棵树，一条都不判」
  ADMISSION_BREAK=domain-environment-unkeyed  test-functions-request 不打 variable: 那几项、单元键不加 variable 行（改判定域的环境变量不进键）
      「自证：reuse-fix：」
      「单元键：闭包读按段登记文件与片长两个变量的那条，登记文件的内容变、片长变、设与没设，键都变；闭包不读的那条键不变」
      「test-functions-request：起它的环境里设了的改判定域的变量打成 --token variable:<名>=<值>」
  ADMISSION_BREAK=entry-homonyms-pulled  Rust 入口自己名字的定义那一处当引用解，把同名（短名相同）的单元收进入口的闭包
      「自证：reuse-fix：」
      「单元键：同一目标里两条短名相同、不在同一模块的测试函数键不同，改其中一条只它的键变」
  ADMISSION_BREAK=plan-unavailable-as-failure  plan-failure 把计划与记库退 75 照别的非 0 判复用失灵、退 1
      「plan-failure：verdict-plan 退 75：判 prune-server 不可用、退 75、不说复用失灵」「plan-failure：test-plan 退 75：…」「plan-failure：verdict-record 退 75：…」
      「plan-failure：样本标记 exit-75 下 verdict-plan 退 75、plan-failure 判不可用、退 75」
      「判定库：记红时 prune-server 不可用（样本标记 red-record-unavailable）⇒ plan-failure --red 照旧退」（退 75 当复用失灵退 1，欠撤记的那一支在退 75 那一支里，一笔都不记）
      「判定库：欠撤撤不掉（样本标记 owed-withdrawal-unavailable）⇒ verdict-plan 整道退 75、不打 PLAN、欠撤留着；」（记红那一步同上一笔欠撤都没记，撤不掉那一趟无从说起）
  ADMISSION_BREAK=client-failure-swallowed  plan-failure 不认客户端退 3 那一行：verdict-record 退 3 照「记不进库」退 0，计划两条照别的非 0 判
      「plan-failure：verdict-record 退 3（prune-server 客户端没配服务地址）」「plan-failure：test-plan 退 3（prune-server 客户端没配服务地址）」
  ADMISSION_BREAK=plan-failure-not-red  plan-failure 认出失灵也退 0、只打一行 !（四处调用方退回全部现判、不判红）
      「plan-failure：哨兵报复用失灵：verdict-plan 退 1、红句点名那一项与「复用失灵」：退 0（要 1」「plan-failure：哨兵报判定不稳：verdict-record 退 1 也判红：退 0（要 1），输出：! 样本」
      「plan-failure：哨兵报强制现判没起作用：test-plan 退 1 判红：退 0（要 1），输出：! 样本调用」「plan-failure：verdict-plan 退 2（读不了请求文件）：红句写退 2 与那一行 ✗、出路指到子命令」
      「plan-failure：test-plan 退 0 却没打 PLAN 那一行：判红：退 0（要 1），输出：! 样本调」「plan-failure：样本标记 exit-1 下 verdict-plan 退 1、plan-failure 判红：」
      「plan-failure：样本标记 sentinel-reuse-failed 下 verdict-plan 退 1、p」「复用失灵：crates/demo/tests/setting.rs::test_t 键相同却现判了（样本档造的假哨兵：/」
  ADMISSION_BREAK=red-record-failure-swallowed  记红没成：verdict-record 只打一行 !、照退 0，plan-failure 不认「判红没记进判定库」、不照 --red 撤绿（同一把键原有的绿留着、下一趟照复用）
      「plan-failure：verdict-record 打了「判红没记进判定库」而没带 --red：判红、说撤不了」
      「判定库：记红没成（样本标记 red-record-failure）⇒ verdict-record 退 1、plan-failure --red 撤掉原有的绿、下一趟这一条现判」
      「判定库：verdict-record 没跑完（退 137、没有 RECORD 那一行）⇒ plan-failure --red 照样撤绿、判红、下一趟这一条现判」
  ADMISSION_BREAK=build-environment-all-settings  构建环境行（build_environment_setting_lines：工具链行的构建环境那一截与子命令 build-environment-lines，不进单元键）整份收：
      CARGO_BUILD_TARGET_DIR、CARGO_BUILD_INCREMENTAL 与 cargo 配置的每一张表（[alias]、[net]、[build] 的 incremental、[profile.<名>] 这类）都收
      「自证：unit-key：」
      「单元键：构建环境行（build_environment_lines）只收改结局的设置：CARGO_BUILD_INCREMENTAL 与 [build] incremental 换取值、[alias] 加一条、[profile.release] 改一项都不变」
  ADMISSION_BREAK=cargo-configuration-ancestors-unread  cargo 配置只读闭包根自己那一层与 CARGO_HOME 的，不读往上各层（构建环境行：工具链行的构建环境那一截与子命令 build-environment-lines）
      「自证：unit-key：」
      「单元键：祖先目录的 .cargo/config.toml 写 [build] rustflags ⇒ 构建环境行变（cargo 配置从被判那棵树的根往上每一层都读）、单元键都不变」
  ADMISSION_BREAK=withdrawal-owed-unrecorded  记红时 prune-server 不可用：plan-failure --red 收到 verdict-record 退 75 只退 75、不记欠撤，verdict-plan / test-plan 开跑也不读欠撤
      「自证：reuse-fix：」
      （同一把键原有的绿留着、下一趟照复用）
      「判定库：记红时 prune-server 不可用（样本标记 red-record-unavailable）⇒ plan-failure --red 照旧退 75、记一笔欠撤，下一趟开跑先撤同一把键的绿、回读，这一条现判」
      「判定库：欠撤撤不掉（样本标记 owed-withdrawal-unavailable）⇒ verdict-plan 整道退 75、不打 PLAN、欠撤留着；撤得掉之后这一条现判」
  ADMISSION_BREAK=threads-by-worker-count  线程数按实际起的工作线程个数判，不照每台默认重活线程数比
      「判用例日志：读回 63 片、只剩 1 片要跑、起 1 个线程」
  ADMISSION_BREAK=first-definition-only  crash-cases 自查同名函数只认第一份定义（cfg 二选一、子模块同名、带进来的源文件里的同名标的 ignore 都漏）
      「自证：crash-cases-listing：」
      「crash-cases 自查：同名函数 cfg 二选一：debug 那一份不标、release 那一份标 ignore」「crash-cases 自查：子模块里同名标 ignore 的写在顶层那一份（--exact the_case 跑的）后面」
      「crash-cases 自查（带进来的源文件）：debug 下不标、release 下 include!("common/release_case.rs") 进来一份标 ignore 的同名用例」「crash-cases 自查（带进来的源文件）：#[path = "common/p.rs"] mod p; 里有一份标 ignore 的同名用例」
  ADMISSION_BREAK=ignore-attribute-without-space  crash-cases 自查不认 `# [ignore]`（# 与 [ 之间有空格，rustc 认）
      「自证：crash-cases-listing：」
      「crash-cases 自查：# [ignore]（# 与 [ 之间有空格，rustc 认）」
  ADMISSION_BREAK=single-worker-threads-field  崩溃枚举用例日志的记录里不写 started_worker_threads= 那一行
      「自证：crash-case-log：」
      「判绿时记 started_worker_threads=：登记了 threads= 的记 CRASH_POINT_REP」「判绿时记 started_worker_threads=：没登记 threads= 的记「读不到」（门禁不读它起了几个）」
  ADMISSION_BREAK=threads-ignore-shards  跑完那一行带 shards 字段时不逐片判线程数
      「判用例日志：merge：两片各 31 个线程各跑 32 片」「判用例日志：merge：第 1 片那台 32 核、线程数没显式设成 1（来源 crash_amplification_threads_variable、配的 32），跑了 32 片却只起 1 个线程」
      「判用例日志：merge：第 1 片只起 1 个线程，那一片的线程数显式设成 1」「判用例日志：merge：第 1 片只起 1 个线程，那台只有 1 核」
      「判用例日志：merge：第 1 片跑了 32 片却报起了 0 个线程」「判用例日志：merge：shards=3 而逐片字段只有两片」
      「判用例日志：merge：两片线程数都没显式设（来源 available_parallelism）、各取每台默认重活线程数 31」「判用例日志：merge：第 1 片线程数没显式设（来源 available_parallelism）、按那台核数取了 32，不等于每台默认重活线程数 31」
  ADMISSION_BREAK=target-own-files-only  crash-cases 自查只看测试目标自己的文件，不顺着 include!、#[path] mod、mod <名>; 带进来的源文件
      「自证：crash-cases-listing：」
      「crash-cases 自查（带进来的源文件）：debug 下不标、release 下 include!("common/release_case.rs") 进来一份标 ignore 的同名用例」「crash-cases 自查（带进来的源文件）：include! 的参数是 concat!(…)（读的是哪一份判不出）」
      「crash-cases 自查（带进来的源文件）：#[path = "common/p.rs"] mod p; 里有一份标 ignore 的同名用例」「crash-cases 自查（带进来的源文件）：mod 指的文件找不到」
      「crash-cases 自查（带进来的源文件）：use core::include as pull; 把 include! 改名引进来」
  ADMISSION_BREAK=directory-target-every-file-as-root  目录式测试目标把目录里每份 .rs 都当根文件交给 files_brought_in_by_target，再嵌一层的 mod 按目录算成找不到
      「自证：crash-cases-listing：」
      「crash-cases 自查（目录式测试目标）：tests/own_case/main.rs 声明 mod flows;、flows.rs 里再声明 mod nested;」
  ADMISSION_BREAK=judge-takes-forwarded-threads  crash-case-judge 照旧收 crash-point-replay 转过来的 --machine-cores / --threads / --threads-origin
      「自证：thread-source：」
      「crash-case-judge 自己现取线程数：没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、转过来 --threads 1 --threads-origin explicit（crash-point-replay 改前的转法）」 history-kept: 引的是自证用例标签原文，按原文匹配
  ADMISSION_BREAK=cores-from-nproc  本机核数照旧取 nproc（认 OMP_NUM_THREADS），不取 os.cpu_count() 与 CPU 亲和的小者
      「自证：thread-source：」
      「本机核数：PATH 里的 nproc 打 1、OMP_NUM_THREADS=1」「没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、SINGLEFS_THREAD_CAP=3」
      「没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS 与 SINGLEFS_THREAD_CAP」
  ADMISSION_BREAK=thread-variable-ignored  崩溃枚举用例登记的 threads-variable= 不认
      「自证：real-crash-case-row：」「自证：thread-source：」
      「crash-case-command：登记了 threads-variable=SINGLEFS_SAMPLE_CASE_THREADS、调用方设着它等于 1」「标记里 configured_worker_threads= 记的是这条用例读的那个变量（SINGLEFS_SAMPLE」
      「真仓登记的 crash-case:crash-injection-default-batch 的 threads-variabl」
  ADMISSION_BREAK=shards-at-least-ignored  崩溃枚举用例登记的 shards-at-least= 不认，crash-case-shards-at-least 一律打 0
      「crash-case-shards-at-least：点名登记了 16 的那一条 ⇒ 16」「crash-case-shards-at-least：点名登记了 7 的与没登记的 ⇒ 7」「crash-case-shards-at-least：不给键 ⇒ 全部登记的里最大的 16」
  ADMISSION_BREAK=features-not-passed  崩溃枚举用例登记了 features= 也不往 cargo 参数里带 --features
      「自证：crash-case-features：」
      「crash-case-command：登记了 features=sample-engine 的行（GPU 关）：-p p」「crash-case-command：登记了 features=sample-engine 的行、多机配置 ENABLE」
  ADMISSION_BREAK=features-passed-to-every-case  没登记 features= 的用例也一律带 --features（带开关自己的占位特性 unregistered-feature）
      「自证：crash-case-features：」
      「crash-case-command：没登记 features= 的行、多机配置没开 GPU：命令照旧，不带 --fea」「crash-case-command：没登记 features= 的行、多机配置开着 GPU：一律带 --feature」
  ADMISSION_BREAK=gpu-case-quiesced-by-itself  上卡的用例整条前面接 bash scripts/crash/gpu-quiesce-run.sh --（清场归 GPU 批处理验证服务，用例自己不清场）
      「自证：crash-case-features：」
      「crash-case-command：没登记 features= 的行、多机配置开着 GPU：一律带 --feature」「crash-case-command：登记了 features=sample-engine 的行、多机配置 ENABLE」
      「crash-case-command：开着 GPU 而调用方环境里没有 GPU_CARDS_BY_PRIORITY」
  ADMISSION_BREAK=gpu-cards-from-configuration  上卡的用例从多机配置取显卡先后（GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED，没写取 GPU_CARDS_BY_PRIORITY），不认调用方环境里的那一份
      「自证：crash-case-features：」
      「crash-case-command：没登记 features= 的行、多机配置开着 GPU：一律带 --feature」「crash-case-command：登记了 features=sample-engine 的行、多机配置 ENABLE」
      「crash-case-command：开着 GPU 而调用方环境里没有 GPU_CARDS_BY_PRIORITY」
  ADMISSION_BREAK=gpu-test-threads-one  开着 GPU 的用例 cargo 参数收尾带 --test-threads 1
      「自证：crash-case-features：」
      「crash-case-command：没登记 features= 的行、多机配置开着 GPU：一律带 --feature」「crash-case-command：开着 GPU（带特性行与不带特性行）与 GPU 关的命令都不带 --te」
  ADMISSION_BREAK=verification-default-features-kept  验证包的崩溃枚举用例不带 --no-default-features（照编 src/bin/ 下的装置二进制）
      「自证：crash-case-features：」
      「crash-case-command：验证包的用例（GPU 关、不登记 features=）：-p singlefs-verification 之后带 --no-default-features」「crash-case-command：验证包的用例（登记了 features=sample-engine、GPU 开）」
  ADMISSION_BREAK=threads-skip-self-contained-finish-line  跑完那一行自己带线程数时不判线程数
      「自证：real-crash-case-row：」
      「真仓登记的 crash-case:crash-injection-default-batch：本机 32 核、线程数没显式设，跑完那一行报 1 个线程跑了 24 片」
  ADMISSION_BREAK=threads-default-unchecked  不核配的线程数等于每台默认重活线程数
      「判用例日志：线程数没显式设（来源 available_parallelism）、用例按核数取了 32，不等于每台默认重活线程数 31」「判用例日志：merge：第 1 片线程数没显式设（来源 available_parallelism）、按那台核数取了 32，不等于每台默认重活线程数 31」
      「真仓登记的 crash-case:c561-sigma-full：线程数没显式设，用例按本机核数取了 32 个（不是每台默认重活线程数）」
  ADMISSION_BREAK=threads-default-from-cores  没设线程变量时线程数按核数算，不取登记的每台默认重活线程数
      「自证：thread-source：」
      「crash-case-command：登记了 threads-variable=SINGLEFS_SAMPLE_CASE_THREADS、调用方设着它等于 1」「本机核数：PATH 里的 nproc 打 1、OMP_NUM_THREADS=1」
      「没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、SINGLEFS_THREAD_CAP=3」「没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、SINGLEFS_THREAD_CAP=abc」
      「没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS 与 SINGLEFS_THREAD_CAP」
  ADMISSION_BREAK=runner-program-only  cargo runner 配置只读程序本身，不读它的参数
      「单元键：cargo 配置 runner 写 `bash tools/runner.sh`，参数里的脚本（.cargo 那一层下）改一个字（runner-script-argument）」
      「单元键：cargo 配置 runner 写 `bash package-runner.sh`，脚本只在登记用例的包目录 crates/demo 下，改一个字（runner-script-in-package-directory）」
  ADMISSION_BREAK=runner-arguments-from-configuration-directory-only  runner 参数里的相对路径只从 .cargo 那一层 / 仓根解，不从各崩溃枚举用例的包目录解
      「单元键：cargo 配置 runner 写 `bash package-runner.sh`，脚本只在登记用例的包目录 crates/demo 下，改一个字（runner-script-in-package-directory）」
  ADMISSION_BREAK=library-files-unreachable-as-absent  library-files fetch 连不上服务时照「库里没有」交 0 条、退 0（改错的样子：调用方当全部没判过现判）
      「自证：library-files：」
      「library-files：服务连不上时 fetch 退 75、中转目录里一个文件都没写」
  ADMISSION_BREAK=multi-host-config-inherited  起自证的环境里多机配置变量设着的那一格，起样本仓的 replay.sh 时不清 SINGLEFS_MULTI_HOST_CONFIG（继承来的变量让样本里的
      replay.sh 去读不在的 scripts/crash/crash-amplification-lib.sh，一条实验都没跑）
      「replay.sh：起自证的环境里设着多机配置变量（指到一份在的配置）时，样本仓的 replay.sh 照样跑、判字节一致」
  ADMISSION_BREAK=function-units-library-by-prefix  function-units-library-copy 那一格拷算键库只按文件名前缀 lib_function_units 列，不照算键库实际 import 的同目录模块现算
      （清单里没有名字不以 lib_function_units 起头的 lib_lag_split_family.py，拷出来的 lib_function_units.py 载入报 ModuleNotFoundError）；别的格拷算键库不读这条开关
      「自证：function-units-library-copy：」
      「样本仓拷算键库：清单里有它在模块级 import 的 lib_lag_split_family.py」「样本仓拷算键库：拷出来的 lib_function_units.py 在只有这份拷贝的目录里载得进」
  ADMISSION_BREAK=function-units-runtime-data-not-copied  function-units-library-copy 那一格拷算键库只拷模块、不拷算键库运行期读的数据文件（模块级 RUNTIME_DATA_FILES 列的，例 scripts/gate/test-service-code-paths.tsv）
      （样本仓里没有那几个文件，拷出来的 lib_function_units.py 载得进、读测试服务代码登记表时拒算）；别的格拷算键库不读这条开关
      「自证：function-units-library-copy：」
      「样本仓拷算键库：算键库运行期读的数据文件拷到了样本仓里同一个相对路径」「样本仓拷算键库：拷出来的算键库读得到测试服务代码登记表」
  ADMISSION_BREAK=runtime-data-nonliteral-skipped  function-units-library-copy 那一格里夹具模块的 RUNTIME_DATA_FILES 写成拼出来的（不是字面的字符串元组）时算零份、不抛 RuntimeDataError
      「自证：function-units-library-copy：」
      「样本仓拷算键库：RUNTIME_DATA_FILES 写成非字面的字符串元组时抛 RuntimeDataError、不悄悄跳过」
  共用库的 REUSE_CHECKS_BREAK=rejudged-sentinel-passes（哨兵被现判不红）：自证 reuse-sentinel 那一格判错
  函数单元库的 FUNCTION_UNITS_BREAK=rust-manifest-directory-reads-unrefused（清单目录拼出的目录也不拒算 runtime-path）：
      自证「实验准入入口 scripts/experiment-apparatus/lib.rs::admit_experiment_run_or_exit 的闭包不拒算；…」那一格红
  函数单元库的 FUNCTION_UNITS_BREAK=python-parameter-callers-unresolved（别的模块里没有调用处的参数读的文件拒算）与 python-data-reads-ignored（读的文件不收）：
      自证「单元键：请求文件内容变而判法没变：这一份判法键不变、调用方那一条键变（file: 折进键）；判法自己读的规则表改了判法键变」那一格红
  函数单元库的 FUNCTION_UNITS_BREAK=shell-entry-arguments-unnarrowed（shell 入口给了参数也照整份收）：
      自证「单元键：shell 入口带 program-argument:run，--selftest 那一支起的脚本改了：带 run 的键不变、带 --selftest 的键变」那一格红
  函数单元库的 FUNCTION_UNITS_BREAK=row-keyed-tables-whole（按行进键的登记表照旧整份进键）：
      自证「单元键：判法经登记的取用函数按入口查 .claude/hooks/selftest-registry.tsv 的一行（lib_function_units.py 文件头「按行进键的登记表」）：改表里别的脚本那一行，这一条的键不变；改它自己那一行，键变」那一格红

代码分块：本文件留子命令分派表 COMMANDS、main、自证入口 run_selftest 与实验准入入口 admit_experiment / command_experiment（lib_function_units.py 的自证替换载入的这一份里的
  judge_inputs_unchanged 与 experiment_key，要调它们的 admit_experiment 在这一份里）；别的名字写在同目录的几份模块里，本文件按名字再导出，别处按路径载入本文件取名字，名字都取得到：
  admission_base.py             常量、登记表行与准入条件、复跑表行与产物头、崩溃枚举用例的行解析、多机配置与每台默认重活线程数、git 询问
  admission_unit_key.py         单元键（Request、UnitKey、KeyContext、compute_unit_key）、构建环境与工具链行、判定库 VerdictLibrary 与 library-files
  admission_crash_case.py       崩溃枚举用例：登记表自查、测试目标源码扫描、日志判定、线程数与特性、块账与各 crash-case-* 子命令
  admission_commands.py         verdict-plan、test-plan、verdict-record、plan-failure 等子命令，门禁复用答法、复跑表行的请求与行键、实验键、gate-preconditions、replay-row-*
  admission_selftest_support.py、admission_selftest_cells.py   自证的样本仓、记数与各格，只在 --selftest 里 import（run_selftest 的函数体里 import，模块级不 import）
下面「弄坏开关」的开关表仍写在本文件。读开关的代码在哪一份（按开关名带引号搜得到；自证块里的 BreakSwitch(…) 只是打开它，不在这里列）：
  admission.py：selftest-cells-run-extra、selftest-cells-unknown-accepted、skip-preconditions、skip-unchanged
  admission_base.py：same-run-field-unchecked、ignore-row-keys、build-environment-all-settings、cargo-configuration-ancestors-unread、cores-from-nproc、thread-variable-ignored、shards-at-least-ignored
  admission_unit_key.py：experiment-key-with-kb-documents、skip-build-environment、test-plan-packing-literals-moved、test-plan-file-readers-unmemoized、test-plan-file-digests-reread、composition-lines-unshared、script-closures-recollected、script-closures-process-wide、dependencies-by-crate、verdict-lookup-skipped、judge-only-entry-judges-unkeyed、program-arguments-unkeyed、shell-program-arguments-dropped、red-verdict-reused、superseded-composition-kept、red-verdict-not-recorded、not-judging-files-keyed、not-judging-entries-excluded、library-outside-git-tolerated、domain-environment-unkeyed、entry-homonyms-pulled、runner-program-only、runner-arguments-from-configuration-directory-only、library-files-unreachable-as-absent
  admission_crash_case.py：condition-files-unchecked、rocksdb-entry-unchecked、blocks-count-unchecked、zero-blocks-covered、keep-caller-stage-variables、case-stage-not-set、recorded-only-judged、blocks-count-without-refused、blocks-single-line-only、ran-blocks-line-optional、blocks-line-required-when-marked、exhaustive-judged-when-narrowed、exhaustive-unjudged-without-narrowing、threads-by-worker-count、first-definition-only、ignore-attribute-without-space、single-worker-threads-field、threads-ignore-shards、target-own-files-only、directory-target-every-file-as-root、judge-takes-forwarded-threads、features-not-passed、features-passed-to-every-case、gpu-case-quiesced-by-itself、gpu-cards-from-configuration、gpu-test-threads-one、verification-default-features-kept、threads-skip-self-contained-finish-line、threads-default-unchecked、threads-default-from-cores
  admission_commands.py：tree-compare-reuse、skip-gate-preconditions、row-key-with-product-name、row-key-without-gpu、frozen-root-unkeyed、research-package-unresolved、test-key-with-whole-file、exclude-registered-ignored、skip-names-of-dropped-targets-kept、test-plan-single-process、test-plan-closures-kept、test-plan-parent-not-prewarmed、test-plan-lines-not-prefilled、test-plan-tokens-unshared、test-plan-objects-not-immortal、test-plan-compositions-whole、test-plan-lookups-in-parent、replay-row-shell-index-serial、test-plan-jobs-fallback-one、verdict-key-with-entry-file、red-verdict-not-recorded、replay-package-mention-dashes、replay-bin-wrappers-unread、replay-device-key-binaries-unread、plan-unavailable-as-failure、client-failure-swallowed、plan-failure-not-red、red-record-failure-swallowed、withdrawal-owed-unrecorded
  admission_selftest_support.py：tree-compare-reuse、skip-build-environment、build-environment-lines-names-only、replay-build-tools-unlinked、test-key-with-whole-file、exclude-registered-ignored、skip-names-of-dropped-targets-kept、shell-index-rounds-one、dependencies-by-crate、verdict-key-with-entry-file、verdict-lookup-skipped、function-units-library-by-prefix、function-units-runtime-data-not-copied、runtime-data-nonliteral-skipped
  admission_selftest_cells.py：selftest-cells-run-extra、selftest-cells-unknown-accepted、experiment-key-with-kb-documents、skip-preconditions、skip-gate-preconditions、skip-unchanged、same-run-field-unchecked、condition-files-unchecked、row-key-with-product-name、row-key-without-gpu、frozen-root-unkeyed、research-package-unresolved、test-plan-packing-literals-moved、blocks-count-unchecked、zero-blocks-covered、recorded-only-judged、blocks-count-without-refused、blocks-single-line-only、ran-blocks-line-optional、replay-store-inside-judge、plan-failure-not-red、first-definition-only、ignore-attribute-without-space、target-own-files-only、judge-takes-forwarded-threads、multi-host-config-inherited

管不到的：跑的是不是按今天的源码编出来的二进制（键按源码算，跑的是 target/ 里的旧二进制时两边对不上，cargo test 与 replay.sh 开跑前都会重编）；
  闭包收全没收全以 lib_function_units.py 为准（它文件头「管不到的」）：收不全而没认出来的形态，改了那一处不让判定作废。
  换了工具链、外部判定工具或依赖版本（都不进键）而结局变了的，键照旧相同、照旧复用：由人登记、整族删重判。gate-reuse 不给请求文件时按库里记的请求重算，门禁这一趟的请求项变了（GPU 开关、特性）它看不见：
  调 gate-reuse 的门禁要给请求文件。
"""

import ast
import collections
import copy
import glob
import gzip
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from admission_base import (
     REGISTRATION_TABLE, REPLAY_SCRIPT, RESULTS_DIRECTORY, GATE_GREEN_TREE_RECORD, BREAK_VARIABLE, HEADER_PREFIX, HEADER_LINES_READ_AT_MOST,
     FUNCTION_UNITS_LIBRARY, VERDICT_LIBRARY_DIRECTORY_NAME, RED_VERDICT_REASON, CHANGED_UNITS_SHOWN_AT_MOST, DOMAIN_ENVIRONMENT_VARIABLES,
     DOMAIN_VARIABLE_LITERAL, EXIT_ADMITTED, EXIT_REGISTRATION_ERROR, EXIT_PRECONDITION_MISSING, EXIT_INPUTS_UNCHANGED, EXIT_GATE_MUST_RUN,
     EXIT_GATE_MAY_SKIP, EXIT_GATE_PRECONDITION_MISSING, EXIT_UNIT_KEY_REFUSED, PROBE_TIMEOUT_SECONDS, GATE_PRECONDITION_KINDS, EXPERIMENT_KEY_FORM,
     QUESTION_ROW_FORM, PRODUCT_FIELD_FORM, REPLAY_ROW_FORM, REPLAY_FROZEN_ROOT_CALL, REPLAY_FROZEN_ROOTS_DIRECTORY, FROZEN_ROOT_SKIPPED_DIRECTORIES,
     REPLAY_ROW_KEYS_START, REPLAY_ROW_KEYS_END_LINE, REPLAY_ROW_HEADER_WRITERS, GATE_CONDITION_FORM, STAGE_CELL_KEY_FORM, ENTRY_FORM,
     CRASH_CASE_KEY_PREFIX, CRASH_CASE_KEY_FORM, CRASH_CASE_CONDITION_FORM, CRASH_CASE_SHARDS_AT_LEAST_FORM, CRASH_CASE_FEATURES_FORM,
     UNREGISTERED_FEATURE_OF_BREAK, GPU_FEATURE, MULTI_HOST_CONFIGURATION_VARIABLE, MULTI_HOST_CONFIGURATION_NAME, GPU_QUIESCE_SCRIPT,
     SERVICE_CARD_VARIABLE, BINDGEN_LIBRARY_SCRIPT, BINDGEN_VARIABLE, BINDGEN_ASSIGNMENT_FORM, THREADS_VARIABLE_FORM,
     CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST, CRASH_CASE_FINDINGS_NOT_WRITTEN, CRASH_CASE_BLOCKS_LINE_NOT_PRINTED, CRASH_CASE_NARROWING_VARIABLES,
     CRASH_SHARE_VARIABLE, CRASH_CASE_SHARE_FORM, CRASH_CASE_STAGES, CRASH_STAGE_VARIABLE, CRASH_HOST_WEIGHTS_VARIABLE,
     CRASH_LIBRARY_RUN_KIND_VARIABLE, CRASH_CASE_BLOCKS_PREFIX, CRASH_CASE_BLOCKS_FIELDS, EXIT_BLOCKS_RED, EXIT_BLOCKS_LINE_BAD, CRASH_CASE_TEST_FORM,
     COUNT_LINE_PREFIX_FORM, CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX, PASSED_ONE_TEST_FORM, POSITIVE_INTEGER_FORM,
     CRASH_POINT_REPLAY_THREADS_VARIABLE, HEAVY_THREAD_DEFAULT_FILE, THREAD_CAP_VARIABLE, HEAVY_THREAD_COMMENT_OR_BLANK_FORM,
     HEAVY_THREAD_REGISTRATION_FORM, DEFAULT_WORKER_THREAD_SOURCES, IGNORE_ATTRIBUTE_FORM, IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE, ATTRIBUTE_START,
     FUNCTION_QUALIFIER_AT_END, TEST_ATTRIBUTE_FORM, TEST_IGNORE_FORM, CARGO_CONFIGURATION_FILE_NAMES, TOOLCHAIN_FILE_NAMES,
     BUILD_ENVIRONMENT_EXACT_VARIABLES, BUILD_ENVIRONMENT_VARIABLE_FORMS, BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT, BUILD_CONFIGURATION_SECTIONS,
     BUILD_KEYS_LEFT_OUT, BUILD_ENVIRONMENT_ALL_SETTINGS_BREAK, CONFIGURATION_ANCESTORS_UNREAD_BREAK, BUILD_ENVIRONMENT_PROGRAM_VARIABLES,
     BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS, BUILD_ENVIRONMENT_COMPILER_VARIABLES, CONFIGURATION_PROGRAM_BUILD_KEYS, REPLAY_RESEARCH_DIRECTORY,
     REPLAY_JUDGING_FUNCTIONS, REPLAY_GPU_FEATURE_VARIABLE, REPLAY_SHELL_WORD, REPLAY_PACKAGE_MENTION, REPLAY_PACKAGE_MENTION_DASHES_UNDER_BREAK,
     REPLAY_BIN_MENTION, REPLAY_BIN_VARIABLE_MENTION, REPLAY_FUNCTION_HEADER, REPLAY_DEVICE_KEY_MENTION, VERIFICATION_PACKAGE, RegistrationError,
     InputManifestError, RegistrationRow, read_registration_rows, rows_of_key, resolve_entries, resolve_input_paths, entry_file,
     experiment_condition_file_problems, parse_gate_condition, FUNCTION_UNITS_MODULE, function_units, UnitKeyError, break_is_set, say,
     parse_condition, header_fields, header_lines_of, StoredFingerprint, stored_fingerprints, ReplayRow, replay_row_key_entries,
     replay_rows_with_keys, replay_products_of, markdown_row_status, product_field_value, SAME_RUN_FIELD_FORM, SAME_RUN_PRODUCT_VARIABLE,
     last_result_line_field, same_run_field_value, judge_preconditions, REGISTRY_LISTED_PROGRAMS, run_probe, probe_output_tail, gate_conditions_of,
     judge_gate_preconditions, git_output, CrashCase, parse_crash_case, crash_cases_of, crash_case_of_key,
     git_common_directory, fields_of_line, machine_core_count, registered_heavy_threads, multi_host_configuration_file, configuration_value,
     TEST_PLAN_SHARED, REPLAY_STRIP_HEADER_COMMAND,
)  # noqa: E402,F401
from admission_unit_key import (
     JUDGE_ONLY_ENTRY, Request, parse_request_line, UnitKey, entry_language, run_text, flattened_setting_lines, merged_configuration,
     build_environment_variable_left_out, configuration_affecting_output, SCRIPT_CLOSURES_UNDER_BREAK, KeyContext, resolve_lock_dependency,
     host_cfg_value, target_section_applies, manifest_dependency_names, registry_dependency_filter, rust_cfg_for_profile,
     closure_of_entry, UNIT_LINE_TEXTS, closure_unit_lines, domain_variable_form,
     domain_variable_tokens, closure_literals, domain_variable_lines, not_judging_files_of_entry, request_entry_file, compute_unit_key,
     changed_composition, LATEST_COMPOSITIONS_KEPT, VerdictLibrary, kind_directory_name, JUDGED_DATE, judged_date, write_atomically,
     LibraryUnavailable, composition_file_names, composition_packed, composition_unpacked, composition_lines, composition_files, REUSE_CHECKS_LIBRARY,
     SENTINEL_STATE_SUFFIX, UNIT_VERDICT_SAMPLE_DIRECTORY, _REUSE_CHECKS, reuse_checks_module, sentinel_candidates, plan_line, sha256_of_file,
     read_configuration_file, resolved_program, runner_argument_directories, program_contents, compiler_version, configured_program_lines,
     manifest_sections, workspace_package_directories, configuration_layers, build_environment_lines, command_build_environment_lines,
     build_environment_setting_lines, research_binary_sources, research_package_name, workspace_binary_entry, test_plan_memoized_readers,
     file_library_store, hidden_relative, transit_files, library_files_fetch, library_files_store, command_library_files,
)  # noqa: E402,F401
from admission_crash_case import (
     lint_registration_rows, test_target_source_files, INCLUDE_CODE_MACRO, MODULE_DECLARATION, PATH_ATTRIBUTE, USE_OF_INCLUDE_MACRO,
     TARGET_FILES_FOLLOWED_AT_MOST, string_literal_value, files_brought_in_by_target, crash_case_test_files, rust_code_without_comments,
     RAW_STRING_START, end_of_literal, attribute_spans, attributes_before_each_definition, attributes_mark_ignored, definitions_marked_ignored,
     judge_worker_threads, SHARD_FIELD_NAMES, judge_threads_of_each_shard, crash_case_narrowing_settings, judge_crash_case_log,
     started_worker_threads_text, configured_worker_threads_text, crash_case_worker_threads, crash_case_launch, crash_case_cargo_features,
     bindgen_include_setting, judge_crash_case_blocks, judge_recorded_only_blocks, command_crash_case_blocks, command_crash_cases, option_values,
     forwarded_worker_threads, command_crash_case_judge, ROCKSDB_PREBUILT_SCRIPT, rocksdb_entry_problem, command_crash_case_command,
     command_crash_case_shards_at_least, command_crash_case_judged_by_test, answer_crash_case_mark, command_crash_case_findings_not_written,
     command_crash_case_blocks_not_printed,
)  # noqa: E402,F401
from admission_commands import (
     plan_requests, TestFunction, unit_text_before_body, test_functions_of_package, posixpath_relative, expand_entries, gate_reuse,
     tree_compare_reuse, shell_function_text, unit_source_text, FROZEN_ROOT_DIGESTS, frozen_root_directory, tree_content_rows, frozen_root_digest,
     replay_packages_mentioned, replay_binaries_mentioned, replay_device_key_binaries, replay_row_request, row_unit_key, replay_table_rows,
     gpu_switch_of, shell_index_outputs_added, shell_index_file_parsed, shell_index_links_of_file, shell_index_filled,
     replay_row_shell_index_prefilled, replay_row_keys, experiment_key, judge_inputs_unchanged, stored_product_row_key_status, command_entries,
     command_test_functions, option_pairs, command_unit_key, read_requests, print_plan, command_verdict_plan, TEST_PLAN_WORKERS_FILE_VARIABLE,
     test_plan_resolve, test_plan_private_bytes, test_plan_resolve_chunk, test_plan_parent_memory_shared, test_plan_parent_objects_immortal,
     test_plan_settle_chunk, test_plan_settled_lookups, test_plan_resolve_all, REGISTERED_ITEMS_LIBRARY, registered_test_functions, command_test_plan,
     command_verdict_record, report_forced_rejudge, PLAN_FAILURE_SAMPLE_MARKER, PLAN_FAILURE_SAMPLE_FORMS, RED_RECORD_FAILURE_HEAD,
     RED_RECORD_FAILURE_FORM, RED_RECORD_FAILURE_SAMPLE_MARKER, RED_RECORD_FAILURE_BREAK, RED_RECORD_FAILURE_HOWTO, OWED_WITHDRAWALS_DIRECTORY,
     OWED_WITHDRAWAL_BREAK, RED_RECORD_UNAVAILABLE_SAMPLE_MARKER, OWED_WITHDRAWAL_UNAVAILABLE_SAMPLE_MARKER, verdict_library_name,
     owed_withdrawals_directory, record_owed_withdrawals, read_owed_withdrawal, settle_owed_withdrawals, PLAN_CLIENT_FAILURE_EXIT,
     PLAN_CLIENT_FAILURE_FORM, PLAN_SERVICE_UNAVAILABLE_EXIT, PLAN_SERVICE_UNAVAILABLE_HOWTO, PLAN_COMMANDS, SENTINEL_FAILURE_FORMS,
     plan_failure_sample, sample_plan_failure_exit, add_sample_sentinel_failure, plan_failure_findings, red_record_failure_findings,
     withdraw_red_greens, command_plan_failure, TEST_FUNCTIONS_VERDICT_KIND_SUFFIX, TEST_FUNCTIONS_PACKAGE, TEST_FUNCTIONS_PROFILE,
     TEST_FUNCTIONS_JUDGE_ENTRY, test_functions_request_lines, command_test_functions_request, read_plan_rows, plan_skip_names,
     command_plan_skip_names, plan_green_names, command_plan_green_names, command_gate_reuse, command_replay_row_inputs, replay_row_change_note,
     command_gate_preconditions, command_keys, replay_row_key_problems, command_replay_row_keys,
)  # noqa: E402,F401

def admit_experiment(root, key):
    """返回 (退出码, 产物头的行)。说明打到 stderr。"""
    rows = read_registration_rows(root)
    own_rows = rows_of_key(rows, key)
    if not EXPERIMENT_KEY_FORM.match(key) or not own_rows:
        say(f"  ✗ 准入登记里没有实验键 {key}（{REGISTRATION_TABLE}）")
        say(f"     → 怎么办：在 {REGISTRATION_TABLE} 加一行「{key}<制表符><装置入口，空格分隔>[<制表符><准入条件>]<制表符>#<为什么>」，"
            "照 E142 那一行写；装置入口传的键与登记的键逐字相同")
        return EXIT_REGISTRATION_ERROR, []
    conditions = [token for row in own_rows for token in row.conditions]
    try:
        problems = lint_registration_rows(own_rows, root, rows)
        if problems:
            raise RegistrationError("；".join(problems))
        for token in conditions:
            parse_condition(token)
        replay_products_of(root, key)   # 复跑表按行的准入键那一段写错，在这里按登记有错报
        unit_key, basis = experiment_key(root, key)
        missing, satisfied = ([], []) if break_is_set("skip-preconditions") else judge_preconditions(root, key, conditions)
    except RegistrationError as error:
        say(f"  ✗ {key} 的准入登记有错：{error}")
        say(f"     → 怎么办：改 {REGISTRATION_TABLE} 里 {key} 那一行（入口写错、条件写错），或 {REPLAY_SCRIPT} 的 ADMISSION_ROW_KEYS；"
            "改到 python3 scripts/gate/admission.py entries <根> <键> 列得出、这里不再报错为止")
        return EXIT_REGISTRATION_ERROR, []
    for description in satisfied:
        say(f"  ✓ 前提齐了：{description}")
    if missing:
        say(f"  ✗ 前提没齐，不许开跑 {key}：共 {len(missing)} 条")  # gate-lint:summary
        for description in missing:
            say(f"       {description}")  # gate-lint:detail
        say(f"     → 怎么办：这几条前提登记在 {REGISTRATION_TABLE} 的 {key} 那一行（跑前登记写的「只在……之后跑」）；"
            "先把它们办齐（问题单那一行判完写成够判、上一段产物的判决字段是 true），再跑。"
            "条件本身写错了就改登记表那一行，不许绕过")
        return EXIT_PRECONDITION_MISSING, []
    unit_count = sum(1 for line in unit_key.composition if line.count("\t") == 3 and ":" in line.split("\t")[2])
    if unit_key.key is None:
        say(f"  ! {key} 的实验键拒算（{basis}）：{'；'.join(line.split(chr(9), 1)[1].replace(chr(9), ' ') for line in unit_key.refusals)}；照放行，下一趟也判不了「没变」")
        return EXIT_ADMITTED, [f"{HEADER_PREFIX}name=input_fingerprint key={key} sha256=refused units={unit_count}"]
    if break_is_set("skip-unchanged"):
        unchanged, description = False, "（弄坏开关 skip-unchanged：没比实验键）"
    else:
        unchanged, description = judge_inputs_unchanged(root, key, unit_key.key)
    header = [f"{HEADER_PREFIX}name=input_fingerprint key={key} sha256={unit_key.key} units={unit_count}"]
    if unchanged:
        say(f"  ✗ 拒绝开跑 {key}：实验键（{basis}）自上次产物以来没变，不必重跑——{description}")
        say("     → 怎么办：沿用那一份的结论；没有越过「输入没变」的开关，要重跑就改输入（装置入口闭包、复跑表那一行的参数），键变了自然放行。"
            "实验键只看装置入口单元闭包与这一行的参数，不看 kb 文档与登记行原文")
        return EXIT_INPUTS_UNCHANGED, []
    say(f"  ✓ 放行 {key}：{description}")
    return EXIT_ADMITTED, header


def command_experiment(arguments):
    if len(arguments) != 2:
        say("  ✗ 用法：admission.py experiment <项目根> <实验键>")
        say("     → 怎么办：实验键照 .claude/gate.d/stage-inputs.tsv 的第一列写，例 E142、E142/p8g-crates-dump")
        return EXIT_REGISTRATION_ERROR
    exit_code, header = admit_experiment(os.path.abspath(arguments[0]), arguments[1])
    for header_line in header:
        print(header_line)
    return exit_code


def run_selftest():
    from admission_selftest_support import (
         SELFTEST_HERE, run_unit_key_cells, run_runner_script_cells, run_test_plan_parallel_cells, run_composition_packing_cell,
         run_shell_index_filled_cell, run_replay_row_parallel_cell, run_script_closure_sharing_cell, run_composition_line_sharing_cell,
         run_test_plan_exclusion_cells, run_plan_skip_names_cells, run_test_plan_closure_key_cells, run_verdict_library_cells, run_reuse_fix_cells,
         run_reuse_three_question_cells, run_function_units_library_copy_cell, Selftest,
    )
    from admission_selftest_cells import (
         run_experiment_cells, run_row_key_cells, run_crash_case_blocks_cells, run_gate_precondition_cells, run_crash_cases_listing_cells,
         run_crash_case_narrowing_cells, run_crash_case_log_cells, run_crash_case_features_cells, run_real_crash_case_features_cells,
         run_thread_source_cells, run_real_crash_case_row_cells, run_replay_cells, run_replay_package_mention_cell, run_replay_judging_closure_cell,
         run_replay_row_key_cells, run_plan_failure_cells, run_experiment_admission_entry_cells, run_library_files_cells, run_cell_selection_cells,
    )
    # 判定库那几格写进 prune-server：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），
    # 不是就一格都不跑，不写真服务
    client = VerdictLibrary.client()
    service_problem = client.selftest_service_problem() if client is not None else "这棵树里没有 scripts/prune-server/prune_server_client.py"
    if service_problem:
        say(f"  ✗ admission.py 自证没跑：{service_problem}")
        say(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY if client is not None else '在带着 scripts/prune-server/ 的仓里跑'}")
        return EXIT_REGISTRATION_ERROR
    selftest = Selftest()
    module = os.path.abspath(__file__)
    repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))

    def real_registration_table():
        # 真仓的登记表自己写对没有（键、入口的写法与文件在不在、条件的种类分门禁与实验、@ 引用、probe= / environment= 指的脚本在不在）
        real_problems = lint_registration_rows(read_registration_rows(repository_root), repository_root)
        selftest.expect(f"真仓的登记表 {REGISTRATION_TABLE} 自查没有问题", not real_problems, "；".join(real_problems))

    def real_replay_row_keys():
        _real_rows, _registered, real_row_key_problems, real_shared_keys = replay_row_key_problems(repository_root)
        selftest.expect("真仓 replay.sh 复跑表每一行定得下准入键：按行的准入键那一段写得对、指明的键都登记了、没有登记了准入的键被两行共用",
                        not real_row_key_problems and not real_shared_keys,
                        "；".join(real_row_key_problems + [f"{key} 被 {count} 行共用" for key, count in real_shared_keys]))

    # 格（文件头「选格」）：跑的次序就是这张表的次序；一格只用自己建的临时仓与临时目录，单跑与全部格里跑判定相同
    cells = {
        "unit-key": lambda: run_unit_key_cells(selftest, module),
        "runner-script": lambda: run_runner_script_cells(selftest),
        "test-plan-parallel": lambda: run_test_plan_parallel_cells(selftest, module),
        "composition-packing": lambda: run_composition_packing_cell(selftest),
        "shell-index-filled": lambda: run_shell_index_filled_cell(selftest),
        "function-units-library-copy": lambda: run_function_units_library_copy_cell(selftest),
        "replay-row-parallel": lambda: run_replay_row_parallel_cell(selftest),
        "composition-line-sharing": lambda: run_composition_line_sharing_cell(selftest),
        "script-closure-sharing": lambda: run_script_closure_sharing_cell(selftest),
        "test-plan-exclusion": lambda: run_test_plan_exclusion_cells(selftest, module),
        "plan-skip-names": lambda: run_plan_skip_names_cells(selftest, module),
        "test-plan-closure-key": lambda: run_test_plan_closure_key_cells(selftest, module),
        "verdict-library": lambda: run_verdict_library_cells(selftest, module),
        "reuse-fix": lambda: run_reuse_fix_cells(selftest, module),
        "reuse-three-question": lambda: run_reuse_three_question_cells(selftest, module),
        "plan-failure": lambda: run_plan_failure_cells(selftest, module),
        "experiment": lambda: run_experiment_cells(selftest, module),
        "row-key": lambda: run_row_key_cells(selftest, module),
        "replay": lambda: run_replay_cells(selftest, module),
        "replay-package-mention": lambda: run_replay_package_mention_cell(selftest),
        "replay-judging-closure": lambda: run_replay_judging_closure_cell(selftest, module),
        "replay-row-key": lambda: run_replay_row_key_cells(selftest, module),
        "library-files": lambda: run_library_files_cells(selftest, module),
        "gate-precondition": lambda: run_gate_precondition_cells(selftest, module),
        "crash-cases-listing": lambda: run_crash_cases_listing_cells(selftest, module),
        "crash-case-log": lambda: run_crash_case_log_cells(selftest),
        "crash-case-narrowing": lambda: run_crash_case_narrowing_cells(selftest, module),
        "thread-source": lambda: run_thread_source_cells(selftest, module),
        "crash-case-features": lambda: run_crash_case_features_cells(selftest),
        "crash-case-blocks": lambda: run_crash_case_blocks_cells(selftest, module),
        "experiment-admission-entry": lambda: run_experiment_admission_entry_cells(selftest),
        "cell-selection": lambda: run_cell_selection_cells(selftest, module),
        "real-registration-table": real_registration_table,
        "real-crash-case-row": lambda: run_real_crash_case_row_cells(selftest, read_registration_rows(repository_root)),
        "real-crash-case-features": lambda: run_real_crash_case_features_cells(selftest, read_registration_rows(repository_root), repository_root),
        "real-replay-row-keys": real_replay_row_keys,
    }
    named_text = os.environ.pop("ADMISSION_SELFTEST_CELLS", "")
    named = [word for word in named_text.split(",") if word]
    unknown = [word for word in named if word not in cells] or ([named_text] if named_text and not named else [])
    if unknown and not break_is_set("selftest-cells-unknown-accepted"):
        print(f"  ✗ admission.py 自证：ADMISSION_SELFTEST_CELLS 里点名的 {'、'.join(unknown)} 不是这份自证的格")
        print(f"     → 怎么办：格名照失败行「✗ 自证：<格名>：」写，逗号隔开：{'、'.join(cells)}")
        return EXIT_REGISTRATION_ERROR
    for name, run_cell in cells.items():
        if not named or name in named or (name == "runner-script" and break_is_set("selftest-cells-run-extra")):
            selftest.cell = name
            run_cell()

    if selftest.failures:
        print(f"  ✗ admission.py 自证没过：{selftest.failures} 格判错（共 {selftest.checked} 格）")
        print("     → 怎么办：照上面每一格的说明改被测的那一段；判据与为什么这么定见文件头。弄坏开关那几格红不了，说明那一格分不出差别")
        return 1
    if named:
        print(f"  ✓ admission.py 自证只跑点名的 {len(named)} 格（{named_text}）：{selftest.checked} 项都对；没点名的格这一趟没跑，不算门禁通过")
        return 0
    print(f"  ✓ admission.py 自证通过：{selftest.checked} 格都对（单元键的阳性与阴性对照、判定库的现判 / 复用 / 只写不读、门禁复用答法的 H2 / H3 / E01、"
          "verdict-plan 的复用三问与复用哨兵、实验键的 T19 / N22、设了删掉的强制重跑变量退 2、多机配置变量设着时样本仓的 replay.sh 照跑、实验准入入口不拒算、复跑表行键的 R01–R05 / T18 / E22、崩溃枚举用例的登记 / 日志 / 线程 / 特性 / 块判定那一行 / red-blocks= 标记与 --ran；"
          "含弄坏开关 superseded-composition-kept、verdict-key-with-entry-file、verdict-lookup-skipped、tree-compare-reuse、red-verdict-not-recorded、library-outside-git-tolerated、domain-environment-unkeyed、entry-homonyms-pulled、"
          "experiment-key-with-kb-documents、skip-build-environment、build-environment-lines-names-only、row-key-with-product-name、row-key-without-gpu、blocks-count-unchecked、zero-blocks-covered、keep-caller-stage-variables、ran-blocks-line-optional、skip-unchanged、"
          "skip-preconditions、skip-gate-preconditions、runner-program-only、runner-arguments-from-configuration-directory-only、test-key-with-whole-file 与崩溃枚举用例那几格的老开关下各自那一格转红）")
    return 0


COMMANDS = {
    "entries": command_entries,
    "paths": lambda arguments: command_entries(arguments, files_only=True),
    "test-functions": command_test_functions,
    "test-plan": command_test_plan,
    "unit-key": command_unit_key,
    "verdict-plan": command_verdict_plan,
    "verdict-record": command_verdict_record,
    "plan-failure": command_plan_failure,
    "test-functions-request": command_test_functions_request,
    "plan-skip-names": command_plan_skip_names,
    "plan-green-names": command_plan_green_names,
    "gate-reuse": command_gate_reuse,
    "gate-preconditions": command_gate_preconditions,
    "experiment": command_experiment,
    "keys": command_keys,
    "build-environment-lines": command_build_environment_lines,
    "replay-row-keys": command_replay_row_keys,
    "replay-row-inputs": command_replay_row_inputs,
    "library-files": command_library_files,
    "crash-cases": command_crash_cases,
    "crash-case-command": command_crash_case_command,
    "crash-case-judge": command_crash_case_judge,
    "crash-case-blocks": command_crash_case_blocks,

    "crash-case-judged-by-test": command_crash_case_judged_by_test,
    "crash-case-shards-at-least": command_crash_case_shards_at_least,
    "crash-case-findings-not-written": command_crash_case_findings_not_written,
    "crash-case-blocks-not-printed": command_crash_case_blocks_not_printed,
}


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return run_selftest()
    if not arguments or arguments[0] not in COMMANDS:
        say(f"  ✗ 用法：admission.py {{{'|'.join(COMMANDS)}}} … 或 --selftest")
        say("     → 怎么办：各子命令的参数见文件头；整道全绿标记、用例全绿标记、输入清单与环境记录那几个子命令删了，复用改由各道按判定入库（verdict-plan / verdict-record）")
        return EXIT_REGISTRATION_ERROR
    return COMMANDS[arguments[0]](arguments[1:])


BREAK_VARIABLES = ("ADMISSION_BREAK",)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；门禁样本根（当前目录有 .gate-cells 或 .gate-break-sample、没设 GATE_STAGED_FROM）里由阶段转来的；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    sample_root = (os.path.exists(".gate-cells") or os.path.exists(".gate-break-sample")) and not os.environ.get("GATE_STAGED_FROM")
    if not setting or arguments[:1] == ["--selftest"] or sample_root or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）和门禁样本根里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/admission.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
