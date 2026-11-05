#!/usr/bin/env bash
# admission: always 每一次调都逐条查按条判定库：键对得上的复用、对不上的现判（键与库写在文件头「按条入库」），整张表的结论不替哪一条作保
# run-condition: command cargo git python3
# 证明「这些测试会红」：逐条注入一个已知的破坏，跑测试，记下红了哪几个，然后还原。
# 红了哪几个（libtest 的「test <名字>[ - should panic] ... FAILED」行）与汇总行报没报失败，经 scripts/gate/lib_libtest_result_lines.py failed-names 取（全仓一份解析）：
#   #[should_panic] 的用例那一段「 - should panic」不进名字；汇总行报了失败而一个名字都没取到，它退 3，这里记「抓名字的规则有盲区」。
#
#   mutate.sh <bin 名> <源文件> <变异表>      # 在 scripts/experiment-apparatus/ 里跑（在仓根下起也行，见下面「在仓根下起」）；参数不够三个退 2，打用法与一条例子
#   认不出的选项（- 起头、又不是 --binary-of 的）退 2、点名它、一次 cargo 都不调
#
# 按条入库（判法在 scripts/mutation/lib_mutation_units.py 的 mutate_plan 与 mutate_record，单元的切分与闭包在 scripts/gate/lib_function_units.py）：
#   每条变异一条判定：平常跑记「抓到 / 没红 / 无效」，按键存一个文件（判定、要打的那一行、判定日期、跑法）；
#   键是原文、替换文、包住原文的 fn 单元与它依赖的单元指纹（入口是原文所在的最里层单元，lib_function_units.py 的 Rust 闭包）、cargo 参数与特性
#   （bin 名不进）、工具链行（rustc -vV、cargo -V、这里给每个 cargo 带的 RUSTFLAGS 与构建配置解析后的值）、解析出的依赖版本行（包住原文那几个单元的闭包里真点名的外部 crate 在 scripts/experiment-apparatus/Cargo.lock 里落到的包与传递依赖，
#   lib_mutation_units.py 文件头「依赖版本行」；落不到包的那一条拒算）；
#   不带变异名、表名、行号。平常跑复用另要：抓到的，红了的那几条测试此刻的闭包键与记录里的相同；没红、无效的，
#   这个 bin 全部 #[test] 的闭包键合起来与记录里的相同。测试进程没跑完、抓名字的规则有盲区那两种不记、每趟照跑。原文不落在任何单元上（只在注释里）、闭包里有拒算的那一条拒算：每趟照跑、不写库。
#   记红、撤绿、下趟现判（用户 2026-10-29 定案，D13 已定项 17）：判失败的那几档（没红，记录里 fails=1）也照键写 records/<键>（mutate_record），
#   盖掉同一把键原有的抓到；下一趟这一条现判、stderr 写「现判：[<名>] 上次判红（<档>）：键相同的判定是红，现判」，不当绿复用，也不进复用哨兵的候选；
#   现判又判绿（抓到）就盖回绿，再下一趟复用。判法在下面「按条入库」那段 python 里（复用得上的 fails 不是 0 的改成现判），弄坏开关 redreused 下照旧复用。
#   判定库在 prune-server 里（scripts/mutation/lib_mutation_units.py 的 Library，键前缀 mutation-units[@命名空间]/singlefs-mutate-library/，
#   scripts/prune-server/prune_server_client.py 文件头「键空间」）。库目录只当名字与命名空间、本地不建不读不写，只由被判那棵仓定：
#   被判那棵仓的 git common-dir 底下的 singlefs-mutate-library（git rev-parse --git-common-dir 取）；键里只用库名，哪棵仓不进键，
#   命名空间只在临时服务底下给（客户端文件头「键空间」）。服务没有删的操作，库不回收。
#   取不到 git common-dir（被判的不在 git 工作树里）就退 2、点名取不到的那棵树、一次 cargo 都不调，不退到别的库目录
#   （判定会写进服务里另一个键前缀）；出路：在 git 工作树里跑，rsync 拷的副本先在副本根 git init。弄坏开关 libraryrootfallback 下照旧退回（下面开关表）。
#   MUTATE_LIBRARY_DIRECTORY=<目录> 盖过库目录，只在临时 prune-server 底下认（客户端 selftest_service_problem 交空串：自证给每一例的临时目录、
#   门禁 research-build-and-replay 的样本给这一格的临时目录）；不在临时服务底下设了就退 2、点名它、一次 cargo 都不调（判定会写进服务里另一个键前缀）。
#   弄坏开关 libraryoverridehonored 下照旧认（下面开关表）。
#   服务连不上（客户端退 75）：查库那一段与每条判完的 record_in_library 都照 75 整轮退出（不每条现判、不当「没写进」放过），stderr 写「prune-server 不可用」与出路。
#   客户端别的错（退 3：没配服务地址、时限写坏、帧被拒这类）：查库那一段退 2、一次 cargo 都不起；每条判完的 record_in_library 退 3 照跑完别的、收尾整轮退 2；
#   都不每条现判、不当「没写进」放过，stderr 写「判定库客户端报错」、转打客户端那一句与出路。
#   全部复用时不起 cargo：基线与还原之后那一次都不跑、副本里的源码一个字没改，收尾打一行「全部复用」代替「已还原，基线仍全绿」；
#   有一条要现判就照原来的样子跑基线、派活、跑还原之后那一次。
#   复用哨兵（每一趟都插，规则照 scripts/gate/lib_reuse_checks.py 文件头「二」，判法是那里的 ReuseSentinel）：查完库、派活之前抽。候选是此刻键算得出、
#   records/<键> 记着可复用的档、复用要的另几样（红了的测试的闭包键、全部测试的闭包键）也对得上的那几条——直接读计划文件与记录文件，与 mutate_decision 分开查；
#   种子取 SINGLEFS_REUSE_SENTINEL_SEED（不设现取、打在 stderr），抽 SINGLEFS_REUSE_SENTINEL_COUNT 条（默认 3）当哨兵、再抽 1 条强制现判：
#   强制现判那一条不用库里的判定、交给工作进程照现判跑，stderr 写「现判：[<名>] 强制现判（复用哨兵抽中…）」，计数里算现判；收尾（finish_reuse_sentinel，「计数：」那一行之后）
#   拿它这一趟的档与库里的比、再判哨兵：哨兵有一条被现判「复用失灵」、强制现判的档与库里不同「判定不稳」，打 ✗ 与出路、整轮退 1（别的照判完）。
#   键相同的不足 K 条报「哨兵不足 K 条」，不红；拷进不带 lib_reuse_checks.py 的树里跑的这一份说一句、不抽。状态在这一轮自己的 SHARD_ROOT 里。收尾那行「计数：」末尾接「、锚点失效 A 条、现判 N 条、复用 M 条、拒算 R 条」，
#   stderr 逐条写现判与拒算的原因。判法（这份脚本判结局的那几支）不进键：改了判法，库里已有的判定照键复用、不会自己作废（这份 bash 的函数闭包在 lib_function_units.py 里拒算）。
#   mutate.sh --binary-of <源文件>            # 打 cargo 从这份源文件编出的 bin 名（与下面「改的文件与跑的二进制对得上」那道核对同一份读法），在 scripts/experiment-apparatus/ 里跑
#
# 在仓根下起：当前目录下有 scripts/experiment-apparatus/Cargo.toml、里面是 [workspace]（仓根自己的 Cargo.toml 也是 [workspace]，改前目录检查照样放过、
#   接着报成「基线就是红的」），就先 cd 进 scripts/experiment-apparatus/，参数里的路径从仓根起的写法改写成从那里起的：
#   scripts/experiment-apparatus/ 起头的剥掉这一段（./ 起头的同样），scripts/ 下别处的写成 ../<其余>（变异表 scripts/mutation/tables/x.tsv → ../mutation/tables/x.tsv），
#   别的相对路径接 ../../，绝对路径原样；改写的是 --binary-of 的源文件与平常跑的 <源文件>、<变异表>，bin 名不动。这样 kb 里按仓根写的复跑命令
#   （bash scripts/mutation/mutate.sh <bin 名> scripts/experiment-apparatus/<源文件> scripts/mutation/tables/<表>）照样能用。
#   stderr 打一行「在仓根下起的：cd 进了 …，参数「…」改写成「…」」（平常跑在头一行之后，--binary-of 与 bin 名对不上退 6 的那两支在打别的话之前）。
#   既不在仓根、也不在 scripts/experiment-apparatus/ 的，照旧在拷副本之前退 2（「要在 scripts/experiment-apparatus/ 下跑」）。
#
# 变异表每行三段，用制表符分隔：变异名 <TAB> 原文 <TAB> 替换文（原文/替换文里的 \n 表示换行）
# 替换必须恰好命中一次，不是的那一条照「锚点失效」记、不当成跑过 —— 静默失效的替换会让整份证明变成假的
# （singlefs-ai-sop/rules/command-safety.md「脚本改文件之后要回读确认」）。
#
# 锚点失效（.claude/rules/mutation-sampling.md 第七类）：原文在源码里命中 0 次或多次的那一条，派活之前就记成「⚓ [<名>] 锚点失效：原文在 <源文件> 里命中 N 次……」，
#   不起 cargo、不进按条判定库、不复用；别的条照跑照判、照入库。收尾「计数：」里一栏「锚点失效 A 条」（接在现判、复用、拒算那三栏前面），A 不为 0 整轮退 3（判完别的条之后）。
#   弄坏开关 MUTATE_BREAK=anchorstopsall 照改前的跑法：一条锚点失效就派活之前退 3、一条变异都不跑。
#
#   mutate.sh --selftest    拿一个假的 cargo（PATH 前面放它，直接起）走一遍各种结局（抓到、没红、无效），
#                           再走一遍并行的几样（进程数不同 stdout 逐字相同、各用各的副本、丢一份结果整道判红、锚点失效那一条记 ⚓ 而别的照跑、整轮退 3），
#                           工作进程数取每台默认重活线程数（SINGLEFS_THREAD_CAP 顶），取不到退 2、一次 cargo 都不调；
#                           再走一遍 required-features（挂了的 bin 带上 --features 跑得起来，没挂的 bin 的 cargo 参数与改前逐字相同），
#                           再走一遍平常跑带只供自证的变量（MUTATE_BREAK：退 2、一次 cargo 都不调），
#                           再走一遍不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY（临时服务的标记设成空串：退 2、点名它、一次 cargo 都不调），
#                           基线编不过退 2；--binary-of 显式 [[bin]] 打声明的名字、没写的打文件名去掉 .rs。
#                           再走一遍 deny 警告（假 cargo 记下每次收到的 RUSTFLAGS；替换文留死代码的那一条 RUSTFLAGS 里没有 --cap-lints warn 就编不过）：
#                           平常跑调用方没设、设了 RUSTFLAGS 各一例判抓到；基线、那一条与还原之后那一次收到的 RUSTFLAGS 逐字相同，调用方的接在前面。
#                           再喂环境里设了 CARGO_ENCODED_RUSTFLAGS 的两例（设空串、设了别的值各一例；假 cargo 照真 cargo 只读它、不读 RUSTFLAGS）：
#                           留死代码那一条照样判抓到，每个 cargo 收到的 CARGO_ENCODED_RUSTFLAGS 逐字是「调用方的（有就在前）+ 0x1f + --cap-lints + 0x1f + warn」，stderr 报的与收到的一致。
#                           再走一遍记红、撤绿、下趟现判（unit-library-red）：三条抓到的表（替换文各不相同、三把键）先判一趟入库，复用哨兵强制现判一条（哨兵取 2 条）、假 cargo 演测试全绿，那一条判没红、整轮退 1，
#                           下一趟那一条现判（stderr 写「上次判红（notred）」、cargo 照起）并判回抓到、别的两条复用，再下一趟三条都复用、一次 cargo 都不起；
#                           再走一遍按条入库（被测源码换成一份带 #[test] 的）：同一条第二趟复用、一次 cargo 都不起（「全部复用」），
#                           改了被改坏函数调到的函数之后那一条现判，原文只在注释里的那一条拒算照跑；
#                           依赖版本行只收闭包里真点名的外部 crate（夹具 Cargo.lock：头一趟逐字是 extdep 与传递依赖 inner 两行，不点外部 crate 的是空的，
#                           改闭包外点的 other 的版本照样复用，改 inner 的版本现判，认 package = 改名）。
#                           再走一遍副本范围：装置 include_str! 读仓根下别处的文件（../../.claude/heavy-thread-default）时副本根下链上、基线编得过、照常判抓到；
#                           认不出的写法（env! 给的路径）与落到仓根之外的路径退 2、逐行点名、一次 cargo 都不起（判法在 scripts/hosts/lib_repository_copy.py 的闭包）。
#                           再走一遍复用三问（reuse-three-questions，scripts/gate/lib_reuse_checks.py 文件头「一」）：两条抓到的表，头一趟两条现判、同一份输入第二趟全复用、
#                           加一个没人调的函数全复用、改「改孤函数」那一条闭包里的 lonely_helper 只现判它，末尾打规范输出 REUSE_SELFTEST script=scripts/mutation/mutate.sh …；
#                           再走一遍复用哨兵（reuse-sentinel，共用库文件头「二」）：种子 5、哨兵 1 条那一趟一条强制现判、与库里同为「caught」、退 0，不读库那一趟红「复用失灵」、退 1。
#                           再走一遍测试线程（test-threads 三例，文件头「并行」）：调用方没设 RUST_TEST_THREADS、SINGLEFS_THREAD_CAP=10 时每条 cargo test 收到的是 10 ÷ 工作进程数、
#                           上限 1 时取 1，调用方设了 7 的原样收到 7。
#                           再走一遍在仓根下起（from-root 四例，文件头「在仓根下起」，不起 cargo）：kb 的写法与 ./ 起头、scripts/ 之外两种写法各改写对、退 6 报的复跑命令是改写后的参数，
#                           --binary-of 照样打 bin 名，既不在仓根也不在 scripts/experiment-apparatus/ 的照旧退 2。
#                           每一例各用一份自己的临时目录：假 cargo、假 workspace、编译目录、按条判定库（MUTATE_LIBRARY_DIRECTORY）都在里面，不读别的例留下的东西；
#                           复用哨兵的状态在每一趟自己的临时根（SHARD_ROOT）里。经统一任务服务交，登记为 selftest（.claude/hooks/selftest-registry.tsv）。
#
# bin 在 Cargo.toml 的 [[bin]] 里挂了 required-features 的：每次 cargo test 带 `--features <包名>/<feature>,…`，
#   feature 从那一节现读，不在命令行上手敲、不写死在这里；没挂的 bin，cargo 的参数与改前逐字相同（`test --release --bin <名>`）。
#   不带的话 cargo 不编这个 bin、直接退 101，基线停在「基线就是红的」，一条变异都没跑。
#   那几个 feature 的依赖要的环境（E162 的 rocksdb 要 BINDGEN_EXTRA_CLANG_ARGS）由调用方设，原样传给 cargo。
#
# deny 警告：scripts/experiment-apparatus/Cargo.toml 设了 [lints.rust] warnings = "deny"，替换文多半故意删掉用法、留下死代码，deny 下那是编译错
#   （平常跑记「⏭ 无效」），不是被断言抓到。所以这里起的每个 cargo（基线、每条变异、还原之后那一次）
#   都带 RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }--cap-lints warn"：调用方设了的 RUSTFLAGS 在前，后面接 --cap-lints warn，lint 一律压成警告，别的编译错照样编不过。
#   这一份开跑前算一次（MUTATE_RUSTFLAGS），一张表里基线、每条变异与还原之后那一次逐字相同：RUSTFLAGS 进 cargo 的指纹，不同就来回重编。
#   开跑时 stderr 打一行「每个 cargo 都带 RUSTFLAGS=「…」」。只有 mutate.sh 加：replay.sh 的产物复跑、门禁 research-unit-tests 格与研究 bin 平常的编译照 deny。
#   调用方的 RUSTFLAGS 里自己带了 --cap-lints 的，rustc 认第一个（rustc 1.98 量的），以调用方的为准。
#   环境里设了 CARGO_ENCODED_RUSTFLAGS（空串也算）时 cargo 只读它、不读 RUSTFLAGS，上面那个 RUSTFLAGS 就悄悄失效：所以这时再给每个 cargo 同时设
#   CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS:+$CARGO_ENCODED_RUSTFLAGS<0x1f>}--cap-lints<0x1f>warn"（各项以 0x1f 分隔，调用方设了的在前），RUSTFLAGS 照旧也设；
#   开跑时 stderr 那一行改报 cargo 实收的那一个变量与值（「每个 cargo 都带 CARGO_ENCODED_RUSTFLAGS=「…」」，0x1f 显示成 ␟）。没设 CARGO_ENCODED_RUSTFLAGS 的，一个字不变。
#
# gate-overlap:copy-kept scripts/mutation/crates-mutation-rows.py 开几个工作进程（worker_count_and_head_line）照那边的 worker_count_for 写：各项取最小、至少 1、头一项是每台默认重活线程数；两边的项不同（这里 MUTATE_JOBS 只能压小、第二项是表里的条数、另算每个 cargo 的编译并行度；那边 GATE_MUTATION_WORKERS 顶掉核数那一项、第二项是要现跑的条数），派活一边是 bash 子 shell 按目录认领、一边是 python 进程池，共用的只剩一个取最小值，抽成共用要这里每趟多起一个 python 只为算一个数、两边头一行的字也不同
# 并行（command-safety.md「一个脚本里的检测项，能并行就并行」，形态照 .claude/gate.d/crates-mutation-replay.sh 与它的判法 scripts/mutation/crates-mutation-rows.py）：
#   每个工作进程一份被测装置的副本（scripts/experiment-apparatus/ 里编译要用的那部分）、一个自己的 CARGO_TARGET_DIR，不在同一份源码上并发改：
#   第 1 个用 MUTATE_TARGET_DIR，第 i 个用它后面加 -w<i>；MUTATE_TARGET_DIR 没设时取这一趟自己的 <分片根>/target（用户 2026-10-23 定案「自己用完自己删」，
#   不跨轮复用）：分片根（$TMPDIR 下这一趟建的临时目录）连同各份编译目录在 EXIT 上删，删之前停掉工作进程、核本机没有进程开着里面的文件
#   （scripts/hosts/run_build_directory.py holders），有就不删、报出进程、退出码改 1。调用方设了的照它、不删。
#   工作进程数取 min(每台默认重活线程数、表里的条数)，至少 1；每台默认重活线程数照登记取法取（scripts/hosts/heavy-threads.sh 的 heavy_thread_default：
#   SINGLEFS_THREAD_CAP 设了照它，不然读 .claude/heavy-thread-default），取不到退 2、一条变异都没跑，不退回按核数算；
#   MUTATE_JOBS 只能压小、不能加大（设成 1 就是原来的串行跑法）。
#   实际开几个、各项给几个、卡在哪一项、每个 cargo 的编译并行度，打在 stderr 的头一行。
#   内存不在这里判、不按内存收工作进程数：见下面「上限、限时与卡住」。
#   每个 cargo 的编译并行度：先用 MUTATE_CARGO_JOBS，其次调用方已经设了的 CARGO_BUILD_JOBS，都没有就取 核数 ÷ 工作进程数（至少 1），免得 N 个 cargo 各按核数开 rustc。
#   每条 cargo test 的测试线程数（基线、每条变异、还原之后那一次）：调用方设了 RUST_TEST_THREADS（非空）原样传、不覆盖；没设的给每次 cargo test 设
#   RUST_TEST_THREADS=max(1, 线程上限 ÷ 工作进程数)，线程上限与工作进程数那一项取同一个登记值；免得 N 个工作进程的测试二进制各按整机核数起线程。
#   只设在自己起的那几次 cargo test 的环境里、不 export。stderr 在头一行之后报一行设了几、怎么算的（或照调用方的）。
#   基线在每份副本上各跑一次（第 1 份那次就是原来的基线，其余几份顺带把各自的编译目录热起来），还原之后那一次也在每份副本上各跑一次。
#   锚点在派活之前逐条核（每条原文在源码里恰好命中一次），不是的那一条见「锚点失效」，别的照派。
#   活是动态领的：每条变异一个认领目录（mkdir 是原子的），谁先跑完谁领表里下一条还没人领的。
#   每条变异的判定写进自己的结果文件，父进程按表里的次序逐个读、逐行打到 stdout；工作进程一个字都不往 stdout 写。
#   stdout 与进程数无关：MUTATE_JOBS=1 与 =4 逐字相同；头一行在 stderr，是两次跑唯一不同的那一行。
#   派出去（表里）多少条、被认领多少条、收回多少份判定，数一遍对不上就退 8，不许少一条还报绿（command-safety.md「并行不许把失败吃掉」）。
#
# 退出码：0 每条都判完、没有判失败的；1 有没红或抓名字的规则有盲区；
#   2 用法、环境或基线不绿，或平常跑时带了只供自证的变量（见「只供自证」），或不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY（见「按条入库」），
#     或取不到每台默认重活线程数，或判定库客户端退 3（客户端别的错：没配服务地址、时限写坏、帧被拒这类；查库那一段退 3 一次 cargo 都不调，
#     判完写库那一步退 3 照跑完、收尾整轮退 2，见「按条入库」末尾）；
#   3 有锚点失效的条（别的条照跑完；同时有 1 那几种的也退 3，逐条的行照打）；
#   4 还原之后没回到全绿；5 跑的时候工作区里的源码被改过；6 源文件与二进制对不上；8 收回的判定与派出去的对不上。
#   测试进程退几都照输出判：点名的测试红了就是抓到，见 judge_mutation。
#   基线就是红的（退 2）：stderr 点出红的测试名（libtest 的「test <名字> ... FAILED」行）、打基线输出末尾 15 行，
#   整份基线输出拷到 ${TMPDIR:-/tmp}/singlefs-mutate-baseline-XXXXXX.log 留着（点出路径，看完自己删），再给单独重现的命令。
#
# 上限、限时与卡住（records/2026-10-30-编译绕过内存总上限.md SC02，用户定「照字面全删」「照旧交服务，服务加一条 工具里不限时」）：
#   每次 cargo test（基线、每条变异、还原之后那一次）直接起，不设内存上限、不限时；cargo 的 stdout 与 stderr 并成一路交判结局（bash -c 'exec "$@" 2>&1' 那一层）。
#   整条 mutate.sh 交统一任务服务（task-server/），一整条一个任务：内存上限由服务按记录给、撞了放大重排；死循环的变异由服务停整条任务
#   （用时远超记录而日志又满 10 分钟不涨就停掉、打回）。
# 只供自证（自证起的子进程里才认；平常跑时设了就退 2 拒跑、一次 cargo 都不调，--selftest 本身不拒）：MUTATE_BREAK（下面那张表），
#   MUTATE_SELFTEST_RECORD_UNREACHABLE=<地址>（每条判完写库那一步改连这个地址，查库那一段照常；自证 record-unavailable 那一例给没人听的 <地址:端口>，
#   record-client-error 那一例给不带端口的写法、客户端退 3）。
#   「自证起的子进程」的标记：MUTATE_SELFTEST_MARK=<文件>，那份文件在、内容是 SELFTEST_MARK_CONTENT 那一行；只有 run_selftest 设它，读完就清，不传给 cargo。
# 弄坏开关（几个用逗号连；自证里每次 run_case 起的里层 mutate.sh 都带着调用方设的值）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 run_selftest 里 run_case 的标签，括号里是那几条「✗ 自检：」的开头）：
#   MUTATE_BREAK=droprow         工作进程把表里第 2 条的判定丢掉不写
#       every-outcome、parallel（都退 8：「应当整轮判失败退 1」「照样让整轮退 1」那几条）；lost-result 本来就带着它跑，照样退 8
#       「✗ 自检：三种结局的表输出里」
#       「✗ 自检：复用三问 reuse-three-questions」
#       「✗ 自检：复用哨兵 reuse-sentinel」
#       workers-registry、test-threads、test-threads-floor、test-threads-caller（三条、四条的表丢了一份判定，整轮退 8，这几格要的是退 0）
#       「✗ 自检：工作进程数：SINGLEFS_THREAD_CAP=2」
#       「✗ 自检：测试线程：调用方没设 RUST_TEST_THREADS」
#       「✗ 自检：测试线程：线程上限 1」
#       「✗ 自检：测试线程：调用方设了的 RUST_TEST_THREADS=7」
#       unit-library-red（「✗ 自检：记红撤绿：同一把键经哨兵强制现判一条」：头一趟丢了第 2 条的判定没入库，库里只剩两条，哨兵取 2 条之后没有强制现判的那一条，后面「记红撤绿」两问跟着红）
#       「✗ 自检：记红撤绿：库里同一把键最近一次是没红」
#       「✗ 自检：记红撤绿：现判又判抓到之后再下一趟应当复用」
#   MUTATE_BREAK=unknownoptionaccepted  认不出的选项照旧当第一个位置参数往下走（没有这一道检查的样子）
#       unknown-option（「认不出的选项 --no-such-option 应当退 2、点名它、一次 cargo 都不调」：当 bin 名往下走，退的不是 2、也点不到「认不出的选项」）
#       「✗ 自检：认不出的选项」
#   MUTATE_BREAK=runtargetkept   没设 MUTATE_TARGET_DIR 时跑完不删这一趟的分片根与里面的编译目录（改前的样子：编译目录在 $TMPDIR 下跨轮留着）
#       own-target（「没设 MUTATE_TARGET_DIR：编译目录应当建在这一趟的分片根里、跑完删了」）
#   MUTATE_BREAK=countblind      父进程不数派出去与收回来的条数（只读在的那几份）
#       lost-result（「丢了一份判定时应当退 8」）
#   MUTATE_BREAK=sharedcopy      所有工作进程挤在第 1 份副本与第 1 个编译目录上（并行之前要防的那种并发改同一份源码）；
#       各工作进程逐条加锁轮流占那一份（run_worker），不真的并发改同一份源码：并发改坏的那几例红不红不定，开关只证挤在一份副本上那一格
#       parallel（「✗ 自检：MUTATE_JOBS=4 时变异应当落在至少 2 份副本上」，核数 ≥ 4 才判这一条）
#   MUTATE_BREAK=nofeatures      不读 required-features、cargo test 一律不带 --features（改前的跑法）
#       required-features（「挂了 required-features 的 bin 应当每次带」：基线退 101、整轮退 2）
#   MUTATE_BREAK=honorbreakoutsideselftest  平常跑时也认只供自证的变量，不拒（改前的跑法）；它是关守卫项，只在它是 MUTATE_BREAK 唯一一项时认（guard_off_honored），与别的项同写平常跑照拒并点名它
#       outside-selftest-MUTATE_BREAK 与 outside-selftest-MUTATE_SELFTEST_RECORD_UNREACHABLE 两格（『平常跑时带了只供自证的变量 … 应当退 2』：
#       自证起这两例的子进程从环境里接着这一项，守卫关了，两个变量都照跑、不拒）
#       「✗ 自检：平常跑时带了只供自证的变量 MUTATE_BREAK」「✗ 自检：平常跑时带了只供自证的变量 MUTATE_SELFTEST_RECORD_UNREACHABLE」
#   MUTATE_BREAK=honorbreakoutsideselftest-with-others  关守卫项与别的项同写也照认（改前的样子）；这一项只在自证进程本身里认，自证起的子进程与不带 --selftest 的跑法都不认它
#       进程里判关守卫项那一格（『关守卫项与别的项同写：guard_off_honored 应当只认单独一项』：同写的那一次认了）；
#       outside-selftest-guard-combined 不红（它起的是平常跑的子进程，那边不认这一项）
#       「✗ 自检：关守卫项与别的项同写：guard_off_honored 应当只认单独一项 honorbreakoutsideselftest」
#   MUTATE_BREAK=baselinelogdropped  基线输出丢进 /dev/null，红了只报一句「基线就是红的」（改前的跑法）
#       baseline-red（「基线就是红的时应当退 2、点出红的测试 tests::adds」）
#       「✗ 自检：基线里 should panic 的用例红了」
#   MUTATE_BREAK=bareusage  参数不够三个时不先判，set -u 报 unbound variable 退 1（改前的样子）
#       usage（「参数不够三个时应当退 2、打用法与一条例子」）
#   MUTATE_BREAK=ownnameparse  红了哪几个回到自己的 sed / awk（`test ` 与 ` ... FAILED` 之间整段算名字，改前的取法），不经 lib_libtest_result_lines.py
#       should-panic（「#[should_panic] 的用例红了应当记「✅ [抓到] 红：tests::answer_panics」……」：名字带上了「 - should panic」）、
#       baseline-red-should-panic（「基线里 should panic 的用例红了……」）
#   LIBTEST_RESULT_LINES_BREAK=should-panic-in-name  原样传给 lib_libtest_result_lines.py（它文件头）：「 - should panic」当成名字的一部分
#       should-panic（「✗ 自检：#[should_panic] 的用例红了应当记「✅ [抓到] 红：tests::answer_panics」」）、
#       baseline-red-should-panic（「✗ 自检：基线里 should panic 的用例红了」）
#   MUTATE_BREAK=libraryunread  不读按条判定库（改前的样子：每趟整张现判）
#       unit-library（「同一条第二趟应当复用……」）、
#       unit-library-callee（「改了被改坏函数调到的函数之后那一条应当现判……stderr 写库里没有这把键」：现判的原因写成了不读库）、
#       reuse-three-questions（「复用三问 reuse-three-questions」：①②③ 全红）、reuse-sentinel（「复用哨兵 reuse-sentinel」：照常那一趟就红「复用失灵」退 1）；
#       复用哨兵抽中的那几例（caught-only 第二趟这一类）另打「✗ 复用失灵」
#       「✗ 自检：依赖版本行：改了闭包没点名的外部包 other 的版本之后那一条」
#       「✗ 自检：依赖版本行：改了点名的 extdep 的传递依赖 inner 的版本之后那一条」
#       unit-library-red（先判红的库里同一把键，不读库时计划里没有复用行，哨兵没有候选、强制现判的是空，下一趟整张现判，三问都落空）
#       「✗ 自检：记红撤绿：同一把键经哨兵强制现判一条」
#       「✗ 自检：记红撤绿：库里同一把键最近一次是没红，下一趟应当现判」
#       「✗ 自检：记红撤绿：现判又判抓到之后再下一趟应当复用」
#   MUTATE_BREAK=libraryoverridehonored  不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY 也照旧认、拿它当库目录（改前的样子）
#       library-directory-outside-service（「不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY 应当退 2……」：照常跑完、调了 cargo）
#       「✗ 自检：不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY」
#   MUTATE_BREAK=libraryrootfallback  取不到 git common-dir 时照常跑完，判定写进开关分支里写死的键前缀 mutation-units/mutate-library/（改前的样子：退到另一个键前缀）
#       library-common-directory-unavailable（「取不到 git common-dir 时应当退 2……」：退到那个键前缀、照常跑完、调了 cargo）
#       「✗ 自检：取不到 git common-dir 时」
#   MUTATE_BREAK=recordunavailableignored  每条判完写库那一步客户端退 75（prune-server 不可用）当「没写进、下一趟照跑」放过（改前的样子），整轮照退 0
#       record-unavailable（「判完写库那一步 prune-server 不可用：应当整轮退 75……」）
#   MUTATE_BREAK=recordclienterrorignored  每条判完写库那一步客户端退 3（客户端别的错）当「没写进、下一趟照跑」放过（改前的样子），整轮照退 0
#       record-client-error（「判完写库那一步客户端退 3：应当整轮退 2……」）
#   MUTATE_BREAK=planclienterrorfallback  查库那一段客户端退 3（客户端别的错）照改前退回「每条现判、不写库」，照起 cargo
#       plan-client-error（「查库那一段客户端退 3：应当退 2……」）
#   MUTATE_BREAK=redreused  判红的（记录里 fails=1：没红、编不过）照旧当绿复用、进复用哨兵的候选（改前的样子：只要档在可复用的几种里就复用）
#       unit-library-red（「记红撤绿：库里同一把键最近一次是没红，下一趟应当现判……」：下一趟复用、cargo 一次都不起；同一格下一问「记红撤绿：现判又判抓到之后再下一趟应当复用……」跟着红）
#   MUTATE_BREAK=calleesignored  键里包住原文的那个函数只取它自己的指纹、不收它调到的单元（依赖版本行也只看它自己）
#       unit-library-callee（「改了被改坏函数调到的函数之后那一条应当现判……」：helper 改了照样复用、cargo 一次都不起）、
#       reuse-three-questions（「复用三问 reuse-three-questions」：③ 红，lonely_helper 改了「改孤函数」照样复用）；
#       unit-library-dependency（「依赖版本行：头一趟库里那一条应当逐字只记 extdep……」：adds 自己不点 extdep，记成空的）
#       「✗ 自检：依赖版本行：改了点名的 extdep 的传递依赖 inner 的版本之后那一条」
#       「✗ 自检：依赖版本行：helper 改点 alias_dep:: 之后库里那一条」
#   MUTATE_BREAK=dependenciesbycrate  依赖版本行按整包收：单元所在 crate 清单里的外部依赖不看点没点名全收（改前的收法）
#       unit-library-dependency（「依赖版本行：头一趟库里那一条应当逐字只记 extdep……」多出 aliased、other、「……改了闭包没点名的外部包 other 的版本之后那一条应当照样复用……」现判了、
#       「……helper 改点 alias_dep:: 之后……逐字只记 aliased 一行」多出 extdep、inner、other）
#   MUTATE_BREAK=includesunlinked  副本根下不链 lib_repository_copy.py 闭包里 scripts/experiment-apparatus/ 之外的仓内文件，也不判闭包算不算得成（改前的样子）
#       include-outside（「装置 include_str 读仓根下别处的文件……应当链上、基线编得过」：基线编不过、退 2）、
#       include-unrecognized（「认不出的 include_str! 写法……应当退 2、逐行点名、一次 cargo 都不起」：不判就起了 cargo）
#   REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库 scripts/gate/lib_reuse_checks.py 的开关，原样传下去）  哨兵被现判不红
#       reuse-sentinel（「复用哨兵 reuse-sentinel」：不读库那一趟退 0）
#   MUTATE_BREAK=no-cap-lints  起 cargo 时不加 --cap-lints warn，RUSTFLAGS 与 CARGO_ENCODED_RUSTFLAGS 都照调用方的原样（改前的跑法：装置的 deny 警告让留死代码的替换文编不过）
#       cap-lints、cap-lints-existing、encoded-empty、encoded-other（「deny 警告：平常跑……留死代码那一条应当判抓到、整轮退 0」：记成『⏭  [留死代码]』无效），
#       四例都另红「deny 警告：……每个 cargo 收到的 RUSTFLAGS 应当逐字是」与「……stderr 应当报」，encoded-empty、encoded-other 两例再红「……收到的 CARGO_ENCODED_RUSTFLAGS 应当逐字是」
#   MUTATE_BREAK=anchorstopsall  锚点失效一条就派活之前退 3、一条变异都不跑（改前的跑法）
#       stale-anchor（「锚点失效的那一条应当记「⚓ 锚点失效」……」：退 3、一次 cargo 都没调、stdout 是空的）
#   MUTATE_BREAK=rocksdb-entry-unchecked  bin 编得到 librocksdb-sys 也不在起 cargo 之前调 scripts/hosts/rocksdb-prebuilt.py entry（改前的跑法）
#       rocksdb-entry-refused（「bin 的 required-features 点到 rocksdb、进程环境里没有预编库的取值，应当在起 cargo 之前退 2……」：假 cargo 照起了）
#   MUTATE_BREAK=rootnotentered  不认仓根（改前的样子：仓根的 Cargo.toml 也是 [workspace]，目录检查照样放过，参数照仓根的写法原样用）
#       from-root-kb-form、from-root-other-form（「在仓根下起……」：没有『在仓根下起的』那一行，源文件没剥掉 scripts/experiment-apparatus/，
#       失败行接的 stderr 头一行就是 bin 名对不上那一句：「实际退 6……mutate: 源文件 scripts/experiment-apparatus/bench/src/bin/fake_bench.rs 在 Cargo.toml 里声明的二进制是 'fake-bench'」；
#       退 6 报的复跑命令是没改写的参数）、
#       from-root-binary-of（「在仓根下起 --binary-of ……」：stderr 没说 cd 进去）
#       「✗ 自检：在仓根下起（kb 的写法、bin 名写错）」
#       「✗ 自检：在仓根下起（./ 起头的源文件、scripts/ 之外的变异表）」
#       「✗ 自检：在仓根下起 --binary-of」
#   MUTATE_BREAK=workersbyprocessor  工作进程数头一项照改前取核数一半、至多 16，不取每台默认重活线程数、取不到也不拒
#       parallel（头一行报的头一项叫核数，不叫每台默认重活线程数）、workers-registry（SINGLEFS_THREAD_CAP=2、MUTATE_JOBS 没设，开的个数是核数那一项给的）、workers-unregistered（SINGLEFS_THREAD_CAP=abc 取不到每台默认重活线程数，照跑了、没退 2）
#       「✗ 自检：MUTATE_JOBS=4、三条的表，stderr 头一行应当报」
#       「✗ 自检：工作进程数：SINGLEFS_THREAD_CAP=2」
#       「✗ 自检：工作进程数：取不到每台默认重活线程数」
#   MUTATE_BREAK=testthreadsunset  不给 cargo test 设 RUST_TEST_THREADS（改前的跑法：每条 cargo test 照 libtest 默认按整机核数起线程）
#       test-threads（假 cargo 收到的是调用方那个空串）、test-threads-floor（同）、test-threads-caller（7 照样传到，但 stderr 没有「不覆盖」那一行）
#       「✗ 自检：测试线程：调用方没设 RUST_TEST_THREADS」
#       「✗ 自检：测试线程：线程上限 1」
#       「✗ 自检：测试线程：调用方设了的 RUST_TEST_THREADS=7」
#   MUTATE_BREAK=overridecallertestthreads  调用方设了的 RUST_TEST_THREADS 也换成算出来的
#       test-threads-caller（假 cargo 收到的是线程上限 ÷ 工作进程数，不是 7）
#       「✗ 自检：测试线程：调用方设了的 RUST_TEST_THREADS=7」
# RocksDB 预编库的取值：bin 编得到 librocksdb-sys（bin_builds_rocksdb：清单里 rocksdb 是不挂特性的依赖，或这个 bin 的 required-features 顺着 [features]
#   一层层点到 dep:rocksdb / rocksdb）时，起任何 cargo 之前判一次（scripts/hosts/rocksdb-prebuilt.py entry --root <scripts/experiment-apparatus 的上两层>）：
#   这台装了、此刻进程环境的取值对得上；判红退 2、一条变异都不跑。编不到的 bin（今天除 E162 那几个之外的全部）不判。
#   entry 的跳过办法：被判的树里没有依赖 rocksdb 的清单，或 PATH 上的 cargo 是带「# rocksdb-prebuilt-entry: selftest-fake-cargo」那一行的假 cargo（那份脚本文件头「entry」）。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SELF_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
BROKEN_JUDGEMENT="${MUTATE_BREAK:-}"
MAXIMUM_WORKERS_BY_PROCESSOR=16  # 只在弄坏开关 workersbyprocessor 下用：照改前取核数一半、至多这么多个
COLLECTION_POLL_SECONDS=0.5      # 父进程隔这么久看一次哪几条判完了，按表序接着往 stdout 打

is_broken() { [[ ",$BROKEN_JUDGEMENT," == *",$1,"* ]]; }

# 关守卫项 honorbreakoutsideselftest（文件头开关表）整值比：只在它是 MUTATE_BREAK 唯一一项时认；与别的项同写，平常跑照拒并点名它。
# 弄坏开关 honorbreakoutsideselftest-with-others（改前的样子：同写也照认）只在自证进程本身里认（in_selftest_process，run_selftest 里设），
# 平常跑与自证起的子进程都不认它：平常跑同写着它照拒，它自己不成借道
GUARD_OFF_ITEM=honorbreakoutsideselftest
in_selftest_process=0
guard_off_honored() { # <MUTATE_BREAK 的值>：关守卫项照认就退 0
  [[ "$1" == "$GUARD_OFF_ITEM" ]] && return 0
  [[ ",$1," == *",$GUARD_OFF_ITEM,"* ]] && ((in_selftest_process)) && is_broken honorbreakoutsideselftest-with-others && return 0
  return 1
}

# 只供自证的变量（文件头「只供自证」）：判是不是自证起的子进程，标记读完就清；平常跑时有一个非空就拒（在 --selftest 那一支之后判）
SELFTEST_MARK_CONTENT="mutate.sh --selftest 起的子进程"
TEST_ONLY_VARIABLES=(MUTATE_BREAK MUTATE_SELFTEST_RECORD_UNREACHABLE)
running_under_selftest=0
if [[ -n "${MUTATE_SELFTEST_MARK:-}" && -f "${MUTATE_SELFTEST_MARK:-}" ]] \
   && [[ "$(cat -- "$MUTATE_SELFTEST_MARK")" == "$SELFTEST_MARK_CONTENT" ]]; then
  running_under_selftest=1
fi
unset MUTATE_SELFTEST_MARK
LIBTEST_RESULT_LINES="$(dirname "$SELF_PATH")/../gate/lib_libtest_result_lines.py"   # 红了哪几个的解析（文件头「红了哪几个」那两行）

# ── 自证：假的 cargo 按被测源码里写的 BEHAVIOR 演一种结局，不编译任何东西 ──
run_selftest() {
  local scratch failures=0 checked=0 output status
  local case_script="$SELF_PATH"   # run_case 跑哪一份 mutate.sh
  local -a case_leading_arguments=()   # run_case 放在 bin 名之前的参数：认不出的选项那一例是 --no-such-option
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/mutate-selftest-XXXXXX")"
  # 自证起的子进程的标记：只有这里设（文件头「只供自证」）；子进程读完就清
  printf '%s\n' "$SELFTEST_MARK_CONTENT" >"$scratch/mark"
  export MUTATE_SELFTEST_MARK="$scratch/mark"
  # 建一例自己的目录 $scratch/<标签>/：假 cargo、假 workspace（scripts/experiment-apparatus/）、几张变异表。$1 标签；
  # $2 被测源码里写的 BEHAVIOR（默认 PASS）；$3 bin 那一节追加的一行（默认不加，required-features 那一例给它挂 feature）
  prepare_case() {
    local case_directory="$scratch/$1" behavior="${2:-PASS}" bin_extra_line="${3:-}"
    mkdir -p "$case_directory/bin" "$case_directory/scripts/experiment-apparatus/bench/src/bin" "$case_directory/scripts/mutation/tables"
    cat >"$case_directory/bin/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
# 假 cargo：只认 mutate.sh 调的 `cargo test --release --bin <名字>`，按被测源码里的 BEHAVIOR 演一种结局；
# 每次被调记一行「当前目录|CARGO_TARGET_DIR|BEHAVIOR|参数」（并行的自证拿前两段数各用了几份副本、几个编译目录）
# 按条入库算工具链行时问的 `cargo -V` 只打一行版本、不记（FAKE_CARGO_VERSION 设了打它：工具链变了那一例）
if [[ "${1:-}" == -V || "${1:-}" == --version ]]; then echo "${FAKE_CARGO_VERSION:-cargo 0.0.0 (mutate.sh selftest)}"; exit 0; fi
behavior="$(sed -n 's/.*BEHAVIOR: \([A-Z]*\).*/\1/p' bench/src/bin/fake_bench.rs | head -1)"
# FAKE_CARGO_BEHAVIOR_OVERRIDE 设了就演它（不进键：同一把键这一趟判出另一档，记红撤绿那一例用）
behavior="${FAKE_CARGO_BEHAVIOR_OVERRIDE:-$behavior}"
echo "$PWD|${CARGO_TARGET_DIR:-}|$behavior|$*" >>"$FAKE_CARGO_LOG"
# 另记一行「BEHAVIOR|子命令|收到的 RUSTFLAGS」（没设记「（没设）」），deny 警告那几例拿它核每个 cargo 收到的 RUSTFLAGS
echo "$behavior|${1:-}|${RUSTFLAGS-（没设）}" >>"$FAKE_CARGO_LOG.rustflags"
# 同样另记收到的 CARGO_ENCODED_RUSTFLAGS（原样，含 0x1f；没设记「（没设）」，设了空串记空）
echo "$behavior|${1:-}|${CARGO_ENCODED_RUSTFLAGS-（没设）}" >>"$FAKE_CARGO_LOG.encoded"
# 另记收到的 RUST_TEST_THREADS（没设记「（没设）」），测试线程那几例拿它核每条 cargo test 的测试线程数
echo "$behavior|${1:-}|${RUST_TEST_THREADS-（没设）}" >>"$FAKE_CARGO_LOG.test-threads"
# 照真 cargo：CARGO_ENCODED_RUSTFLAGS 设了（空串也算）就只读它（各项以 0x1f 分隔）、不读 RUSTFLAGS
if [[ -v CARGO_ENCODED_RUSTFLAGS ]]; then effective_flags="${CARGO_ENCODED_RUSTFLAGS//$'\x1f'/ }"; else effective_flags="${RUSTFLAGS:-}"; fi
# 照 [lints.rust] warnings = "deny"：DEADCODE（替换文留下死代码）在 cargo 实读的 flags 里没有 --cap-lints warn 时编不过
if [[ "$behavior" == DEADCODE && " $effective_flags " != *" --cap-lints warn "* ]]; then
  printf 'error: function `left_unused` is never used\nerror: could not compile `bench` (bin "fake-bench" test) due to 1 previous error\n'; exit 101
fi
# 照真 cargo：源码里 include_str!("<路径>") 读不到（路径相对那份源文件）就编不过（include-outside 那一例：副本里要链上仓根下别处的文件）
while IFS= read -r included; do
  included_file="${included%%:*}"; included_path="${included#*:}"
  if [[ ! -r "$(dirname "$included_file")/$included_path" ]]; then
    printf 'error: couldn'"'"'t read `%s`: No such file or directory (os error 2)\nerror: could not compile `bench`\n' "$included_path"; exit 101
  fi
done < <(grep -rIo --include='*.rs' --exclude-dir=target 'include_str!("[^"]*")' . 2>/dev/null | sed 's/include_str!("\(.*\)")$/\1/')
passing_output() { printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts ... ok\n\ntest result: ok. 2 passed; 0 failed\n'; }
# 照真 cargo：bin 挂了 required-features 而 --features 里没给全（写成 feature 或 包名/feature 都认），不编、退 101
enabled=""; previous=""
for argument in "$@"; do
  [[ "$previous" == --features ]] && enabled+=",$argument"
  [[ "$argument" == --features=* ]] && enabled+=",${argument#--features=}"
  previous="$argument"
done
for feature in $(grep '^required-features' bench/Cargo.toml | grep -o '"[^"]*"' | tr -d '"'); do
  if [[ ",$enabled," != *",$feature,"* && ",$enabled," != *",bench/$feature,"* ]]; then
    printf 'error: target `fake-bench` in package `bench` requires the features: `%s`\n' "$feature"; exit 101
  fi
done
case "$behavior" in
  PASS) passing_output; exit 0 ;;
  RED|DEADCODE) printf 'running 2 tests\ntest tests::adds ... FAILED\ntest tests::subtracts ... ok\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
  PANICS) printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::answer_panics - should panic ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
  BROKEN) printf 'error[E0425]: cannot find value `missing` in this scope\nerror: could not compile `bench`\n'; exit 101 ;;
  *) echo "假 cargo 认不出 BEHAVIOR「$behavior」" >&2; exit 2 ;;
esac
FAKE_CARGO
    chmod +x "$case_directory/bin/cargo"
    printf '[workspace]\nmembers = ["bench"]\n' >"$case_directory/scripts/experiment-apparatus/Cargo.toml"
    printf '[package]\nname = "bench"\n\n[[bin]]\nname = "fake-bench"\npath = "src/bin/fake_bench.rs"\n' >"$case_directory/scripts/experiment-apparatus/bench/Cargo.toml"
    [[ -n "$bin_extra_line" ]] && printf '%s\n' "$bin_extra_line" >>"$case_directory/scripts/experiment-apparatus/bench/Cargo.toml"
    printf '// 注释行\nfn main() {} // BEHAVIOR: %s\n' "$behavior" >"$case_directory/scripts/experiment-apparatus/bench/src/bin/fake_bench.rs"
    local tables="$case_directory/scripts/mutation/tables"
    printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n没红\t// 注释行\t// 改过的注释行\n无效\tBEHAVIOR: PASS\tBEHAVIOR: BROKEN\n' >"$tables/every-outcome.tsv"
    printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n' >"$tables/caught-only.tsv"
    printf '抓到一\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到二\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到三\tBEHAVIOR: PASS\tBEHAVIOR: RED\n' \
      >"$tables/three-caught.tsv"
    printf '抓到一\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到二\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到三\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到四\tBEHAVIOR: PASS\tBEHAVIOR: RED\n' \
      >"$tables/four-caught.tsv"
    # 三条替换文各不相同的抓到：键 = 原文 + 替换文 + …，名字不进键，替换文相同的几条共用一把键（记红撤绿那一例要「只翻强制现判的那一条」，三条得各是一把键）
    printf '抓到一\tBEHAVIOR: PASS\tBEHAVIOR: RED // 一\n抓到二\tBEHAVIOR: PASS\tBEHAVIOR: RED // 二\n抓到三\tBEHAVIOR: PASS\tBEHAVIOR: RED // 三\n' \
      >"$tables/three-distinct.tsv"
    printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n锚点腐化\tBEHAVIOR: GONE\tBEHAVIOR: RED\n' >"$tables/stale-anchor.tsv"
    printf '改注释\t// 注释行\t// 改过的注释行\n' >"$tables/comment-only.tsv"
    printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: PANICS\n' >"$tables/should-panic.tsv"
    printf '留死代码\tBEHAVIOR: PASS\tBEHAVIOR: DEADCODE\n' >"$tables/dead-code.tsv"
  }

  # 在 prepare_case 建好的那一例目录里跑一次 mutate.sh：$1 标签、$2 变异表，其余是额外的环境变量；
  # stdout 落 $scratch/<标签>.stdout、stderr 落 .stderr、两样接起来落 .out，退出码落 .rc，假 cargo 的调用记录落 .cargo
  run_case() {
    local label="$1" table="$2" case_directory="$scratch/$1"
    shift 2
    : >"$scratch/$label.cargo"
    if (cd "$case_directory/scripts/experiment-apparatus" && env -u CARGO_ENCODED_RUSTFLAGS PATH="$case_directory/bin:$PATH" FAKE_CARGO_LOG="$scratch/$label.cargo" \
          MUTATE_TARGET_DIR="$case_directory/target" MUTATE_LIBRARY_DIRECTORY="$case_directory/library" \
          "$@" bash "$case_script" ${case_leading_arguments[@]+"${case_leading_arguments[@]}"} fake-bench bench/src/bin/fake_bench.rs "$table" \
          >"$scratch/$label.stdout" 2>"$scratch/$label.stderr"); then
      echo 0 >"$scratch/$label.rc"
    else
      echo $? >"$scratch/$label.rc"
    fi
    cat "$scratch/$label.stderr" "$scratch/$label.stdout" >"$scratch/$label.out"
  }
  # 失败说明里转述的子进程输出（$(cat …out) 这类，多行）并成一行：换行换成「 | 」、续行去掉首尾空白、空行不要；
  #   转述里以 ✗ 起头的行（子进程的「✗ mutate: …」、客户端的「✗ prune-server 客户端：…」）不另起一行、不被门禁 code-tooling 的 break-switch-replay 格
  #   当成别的格（它按以 ✗ 起头的行认格；写法同 scripts/gate/verus-proof-check.py 的 one_line）
  fail() {
    local message
    message="$(awk 'NR == 1 { sub(/[ \t]+$/, ""); joined = $0; next } { gsub(/^[ \t]+|[ \t]+$/, "") } $0 != "" { joined = joined " | " $0 } END { print joined }' <<<"$1")"
    echo "  ✗ 自检：$message"; failures=$((failures + 1))   # gate-lint:detail
  }

  prepare_case every-outcome
  run_case every-outcome ../mutation/tables/every-outcome.tsv
  output="$(cat "$scratch/every-outcome.out")"; status="$(cat "$scratch/every-outcome.rc")"
  checked=$((checked + 1))
  [[ "$status" == 1 ]] || fail "三种结局各一条的表应当整轮判失败退 1（没红那一条），实际退 $status：$output"
  local wanted
  for wanted in "✅ [抓到] 红：tests::adds" "❌ [没红]" "⏭  [无效]" "计数：抓到 1 条、没红 1 条、无效 1 条、没跑完 0 条、抓名字有盲区 0 条、锚点失效 0 条" "已还原，基线仍全绿"; do
    checked=$((checked + 1))
    grep -qF -- "$wanted" <<<"$output" || fail "三种结局的表输出里应当有「$wanted」，实际：$output"
  done
  # 没挂 required-features 的 bin、正常的变异：照旧判抓到、整轮退 0，每次 cargo 的参数与改前逐字相同（这一例自己的调用记录）
  # own-target：没设 MUTATE_TARGET_DIR（文件头「并行」）：假 cargo 收到的编译目录在这一趟的分片根（这一例自己的 TMPDIR 下 singlefs-mutate-*）里，跑完连同分片根删了
  prepare_case own-target
  mkdir -p "$scratch/own-target-tmp"
  run_case own-target ../mutation/tables/caught-only.tsv MUTATE_TARGET_DIR= TMPDIR="$scratch/own-target-tmp"
  checked=$((checked + 1))
  local own_targets own_left
  own_targets="$(cut -d'|' -f2 "$scratch/own-target.cargo" | sort -u)"
  own_left="$(ls -A "$scratch/own-target-tmp")"
  if [[ "$(cat "$scratch/own-target.rc")" != 0 || -z "$own_targets" || -n "$own_left" ]] || grep -qvE "^$scratch/own-target-tmp/singlefs-mutate-[^/]+/target(-w[0-9]+)?\$" <<<"$own_targets"; then
    fail "没设 MUTATE_TARGET_DIR：编译目录应当建在这一趟的分片根里、跑完删了，实际退 $(cat "$scratch/own-target.rc")，假 cargo 收到的编译目录「$(tr '\n' ' ' <<<"$own_targets")」，跑完 TMPDIR 下剩「$own_left」"
  fi
  prepare_case caught-only
  run_case caught-only ../mutation/tables/caught-only.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/caught-only.rc")"
  output="$(cat "$scratch/caught-only.out")"
  [[ "$status" == 0 ]] && grep -qF "计数：抓到 1 条、没红 0 条、无效 0 条" <<<"$output" \
    || fail "正常的变异照旧判抓到、整轮退 0 并报「计数：抓到 1 条、没红 0 条、无效 0 条」，实际退 $status：$output"
  local unexpected_arguments
  checked=$((checked + 1))
  unexpected_arguments="$(cut -d'|' -f4- "$scratch/caught-only.cargo" | grep -vxF 'test --release --bin fake-bench')"
  if [[ ! -s "$scratch/caught-only.cargo" || -n "$unexpected_arguments" ]]; then
    fail "没挂 required-features 的 bin，cargo 的参数应当逐字是「test --release --bin fake-bench」，实际调了 $(wc -l <"$scratch/caught-only.cargo") 次、对不上的：${unexpected_arguments:-（一次都没调）}"
  fi

  # ── 并行 ──
  # MUTATE_JOBS=4、三条的表应当开几个：min(每台默认重活线程数、4、3)，至少 1；登记值照 heavy-threads.sh 取（与 mutate.sh 同一个取法）
  local registered_threads expected_workers mutated_runs distinct_copies distinct_targets distinct_pairs
  source "$(dirname "$SELF_PATH")/../hosts/heavy-threads.sh"
  if ! registered_threads="$(heavy_thread_default)"; then
    fail "取不到每台默认重活线程数（上面是原因），并行那几格的期望算不出"; registered_threads=1
  fi
  expected_workers=$((registered_threads > 4 ? 4 : registered_threads))
  ((expected_workers > 3)) && expected_workers=3
  ((expected_workers < 1)) && expected_workers=1
  # 同一个场景跑两次（MUTATE_JOBS=1 与 =4）再比对：各在自己的目录里跑，比的是两次的输出，不是一次读另一次留下的状态
  prepare_case serial
  run_case serial ../mutation/tables/every-outcome.tsv MUTATE_JOBS=1
  prepare_case parallel
  run_case parallel ../mutation/tables/every-outcome.tsv MUTATE_JOBS=4
  checked=$((checked + 1))
  cmp -s "$scratch/serial.stdout" "$scratch/parallel.stdout" \
    || fail "MUTATE_JOBS=1 与 =4 的 stdout 应当逐字相同（判定行按表序打），实际不同：$(diff "$scratch/serial.stdout" "$scratch/parallel.stdout")"
  checked=$((checked + 1))
  status="$(cat "$scratch/parallel.rc")"
  if [[ "$status" != 1 ]] || ! grep -qF "❌ [没红]" "$scratch/parallel.stdout" || ! grep -qF "✅ [抓到] 红：tests::adds" "$scratch/parallel.stdout"; then
    fail "MUTATE_JOBS=4 时没红的那一条照样让整轮退 1、抓到的那一条照样判抓到，实际退 $status：$(cat "$scratch/parallel.out")"
  fi
  checked=$((checked + 1))
  [[ "$(head -1 "$scratch/parallel.stderr")" == *"这一轮开 $expected_workers 个工作进程"*"每台默认重活线程数 $registered_threads"* ]] \
    || fail "MUTATE_JOBS=4、三条的表，stderr 头一行应当报「这一轮开 $expected_workers 个工作进程」、头一项是「每台默认重活线程数 $registered_threads」，实际：$(head -1 "$scratch/parallel.stderr")"
  # 工作进程数取每台默认重活线程数（文件头「并行」）：SINGLEFS_THREAD_CAP=2、MUTATE_JOBS 没设、三条的表开 2 个；取不到（SINGLEFS_THREAD_CAP=abc）退 2、一次 cargo 都不调。
  # 弄坏开关 workersbyprocessor（改前取核数一半至多 16）下这两格判错
  prepare_case workers-registry
  run_case workers-registry ../mutation/tables/three-caught.tsv MUTATE_JOBS= SINGLEFS_THREAD_CAP=2
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/workers-registry.rc")" != 0 || "$(head -1 "$scratch/workers-registry.stderr")" != *"这一轮开 2 个工作进程"*"卡在「每台默认重活线程数」这一项"* ]]; then
    fail "工作进程数：SINGLEFS_THREAD_CAP=2、MUTATE_JOBS 没设、三条的表应当开 2 个、卡在「每台默认重活线程数」、整轮退 0，实际退 $(cat "$scratch/workers-registry.rc")：$(head -1 "$scratch/workers-registry.stderr")"
  fi
  prepare_case workers-unregistered
  # RUST_TEST_THREADS 由调用方给：测试线程那一步不取登记值，退 2 只能来自工作进程数那一项
  run_case workers-unregistered ../mutation/tables/three-caught.tsv MUTATE_JOBS= SINGLEFS_THREAD_CAP=abc RUST_TEST_THREADS=3
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/workers-unregistered.rc")" != 2 || -s "$scratch/workers-unregistered.cargo" ]] || ! grep -qF "取不到每台默认重活线程数" "$scratch/workers-unregistered.stderr"; then
    fail "工作进程数：取不到每台默认重活线程数（SINGLEFS_THREAD_CAP=abc）应当退 2、报「取不到每台默认重活线程数」、一次 cargo 都不调，实际退 $(cat "$scratch/workers-unregistered.rc")、调了 $(grep -c . "$scratch/workers-unregistered.cargo") 次：$(cat "$scratch/workers-unregistered.out")"
  fi
  checked=$((checked + 1))
  [[ "$(head -1 "$scratch/serial.stderr")" == *"这一轮开 1 个工作进程"*"卡在「MUTATE_JOBS」这一项"* ]] \
    || fail "MUTATE_JOBS=1 时 stderr 头一行应当报开 1 个、卡在「MUTATE_JOBS」，实际：$(head -1 "$scratch/serial.stderr")"
  # 施加了变异的那几次（BEHAVIOR 不是 PASS）落在几份副本上：至少两份，才说明变异真的分给了不同的工作进程；
  # 每份副本只配一个编译目录、几份各不相同，才说明没有两个工作进程在同一份源码或同一个编译目录上并发
  if ((expected_workers >= 2)); then
    checked=$((checked + 1))
    mutated_runs="$(awk -F'|' '$3 != "PASS"' "$scratch/parallel.cargo")"
    distinct_copies="$(cut -d'|' -f1 <<<"$mutated_runs" | sort -u | wc -l)"
    distinct_targets="$(cut -d'|' -f2 "$scratch/parallel.cargo" | sort -u | wc -l)"
    distinct_pairs="$(cut -d'|' -f1,2 "$scratch/parallel.cargo" | sort -u | wc -l)"
    if ((distinct_copies < 2 || distinct_targets != expected_workers || distinct_pairs != expected_workers)); then
      fail "MUTATE_JOBS=4 时变异应当落在至少 2 份副本上、$expected_workers 个工作进程各用一份副本与一个编译目录，实际施加变异的副本 $distinct_copies 份、编译目录 $distinct_targets 个、(副本, 编译目录) 组合 $distinct_pairs 个：$(cat "$scratch/parallel.cargo")"
    fi
  fi

  # 丢一份判定：收回的比派出去的少，整道退 8，不许少一条还报绿；再拿掉那道闸（countblind），同一个丢法要静默判绿，才说明退 8 是那道闸给的
  prepare_case lost-result
  run_case lost-result ../mutation/tables/three-caught.tsv MUTATE_JOBS=3 MUTATE_BREAK="droprow${BROKEN_JUDGEMENT:+,$BROKEN_JUDGEMENT}"
  checked=$((checked + 1))
  status="$(cat "$scratch/lost-result.rc")"
  if [[ "$status" != 8 ]] || ! grep -qF "派出去 3 条" "$scratch/lost-result.stderr" || ! grep -qF "收回 2 份" "$scratch/lost-result.stderr"; then
    fail "丢了一份判定时应当退 8 并报「派出去 3 条……收回 2 份」，实际退 $status：$(cat "$scratch/lost-result.out")"
  fi
  prepare_case lost-result-unchecked
  run_case lost-result-unchecked ../mutation/tables/three-caught.tsv MUTATE_JOBS=3 MUTATE_BREAK=droprow,countblind
  checked=$((checked + 1))
  status="$(cat "$scratch/lost-result-unchecked.rc")"
  if [[ "$status" != 0 || "$(grep -c '^✅' "$scratch/lost-result-unchecked.stdout")" != 2 ]]; then
    fail "拿掉收束的计数闸之后，同一个丢法应当静默退 0、只报 2 条抓到（这样上一项的退 8 才是那道闸给的），实际退 $status：$(cat "$scratch/lost-result-unchecked.out")"
  fi

  # MUTATE_JOBS 不是正整数：退 2，一次 cargo 都不调
  prepare_case bad-jobs
  run_case bad-jobs ../mutation/tables/caught-only.tsv MUTATE_JOBS=0
  checked=$((checked + 1))
  status="$(cat "$scratch/bad-jobs.rc")"
  if [[ "$status" != 2 || -s "$scratch/bad-jobs.cargo" ]] || ! grep -qF "MUTATE_JOBS" "$scratch/bad-jobs.stderr"; then
    fail "MUTATE_JOBS=0 应当退 2、点名 MUTATE_JOBS、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/bad-jobs.cargo") 次：$(cat "$scratch/bad-jobs.out")"
  fi

  # 认不出的选项（文件头用法）：退 2、点名它、一次 cargo 都不调。弄坏开关 unknownoptionaccepted 下它当 bin 名往下走，这一格判错
  prepare_case unknown-option
  case_leading_arguments=(--no-such-option)
  run_case unknown-option ../mutation/tables/caught-only.tsv
  case_leading_arguments=()
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/unknown-option.rc")" != 2 || -s "$scratch/unknown-option.cargo" ]] || ! grep -qF "认不出的选项 --no-such-option" "$scratch/unknown-option.stderr"; then
    fail "认不出的选项 --no-such-option 应当退 2、点名它、一次 cargo 都不调，实际退 $(cat "$scratch/unknown-option.rc")、调了 $(wc -l <"$scratch/unknown-option.cargo") 次：$(cat "$scratch/unknown-option.out")"
  fi

  # 锚点失效按条判（文件头「锚点失效」）：两条的表，原文命中 0 次的那一条记「⚓ 锚点失效」、不入判定库，另一条照跑抓到；
  # 收尾「计数：」报锚点失效 1 条、整轮退 3。弄坏开关 anchorstopsall（改前的跑法：派活之前就退 3、一次 cargo 都不调）下这一格判错
  library_record_names() { # <按条判定库的目录>：库里 records/ 底下的名字，一行一个（判定库在临时 prune-server 里，库目录只当名字，本地没有文件）
    python3 - "$(dirname "$SELF_PATH")/lib_mutation_units.py" "$1" <<'PY'
import importlib.util, sys
specification = importlib.util.spec_from_file_location("lib_mutation_units_for_mutate_selftest", sys.argv[1])
units = importlib.util.module_from_spec(specification)
sys.modules[specification.name] = units
specification.loader.exec_module(units)
names = units.Library(sys.argv[2]).stored_names("records")
print("\n".join(names), end="\n" if names else "")
PY
  }
  prepare_case stale-anchor
  run_case stale-anchor ../mutation/tables/stale-anchor.tsv MUTATE_JOBS=2
  checked=$((checked + 1))
  status="$(cat "$scratch/stale-anchor.rc")"
  if [[ "$status" != 3 || ! -s "$scratch/stale-anchor.cargo" ]] \
     || ! grep -qxF "✅ [抓到] 红：tests::adds" "$scratch/stale-anchor.stdout" \
     || ! grep -qxF "⚓ [锚点腐化] 锚点失效：原文在 bench/src/bin/fake_bench.rs 里命中 0 次（要求恰好 1 次）—— 这一条没跑，不算抓到也不算没红" "$scratch/stale-anchor.stdout" \
     || ! grep -qE '^计数：.*、锚点失效 1 条、现判 [0-9]+ 条、复用 [0-9]+ 条、拒算 [0-9]+ 条$' "$scratch/stale-anchor.stdout" || ! grep -qF "已还原，基线仍全绿" "$scratch/stale-anchor.stdout" \
     || ! grep -qF "1 条锚点失效（上面 ⚓ 那几行）" "$scratch/stale-anchor.stderr" \
     || (( $(library_record_names "$scratch/stale-anchor/library" | grep -c .) > 1 )); then
    fail "锚点失效的那一条应当记「⚓ 锚点失效」、不入判定库，另一条照跑抓到，计数报锚点失效 1 条、整轮退 3，实际退 $status、调了 $(grep -c . "$scratch/stale-anchor.cargo") 次 cargo、判定库记录 $(library_record_names "$scratch/stale-anchor/library" | grep -c .) 份：$(cat "$scratch/stale-anchor.out")"
  fi

  # ── required-features ──
  # 挂了两个 feature 的 bin：从清单现读、带「包名/feature」跑，基线绿、变异照常判抓到；
  # 不读（MUTATE_BREAK=nofeatures，改前的跑法）假 cargo 照真 cargo 退 101，停在基线、退 2
  prepare_case required-features PASS 'required-features = ["fake-feature", "fake-other"]'
  run_case required-features ../mutation/tables/caught-only.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/required-features.rc")"
  unexpected_arguments="$(cut -d'|' -f4- "$scratch/required-features.cargo" | grep -vxF 'test --release --bin fake-bench --features bench/fake-feature,bench/fake-other')"
  if [[ "$status" != 0 || ! -s "$scratch/required-features.cargo" || -n "$unexpected_arguments" ]] \
     || ! grep -qF "✅ [抓到] 红：tests::adds" "$scratch/required-features.stdout"; then
    fail "挂了 required-features 的 bin 应当每次带「--features bench/fake-feature,bench/fake-other」跑、抓到那一条、退 0，实际退 $status、对不上的参数：${unexpected_arguments:-无}：$(cat "$scratch/required-features.out")"
  fi
  # ── RocksDB 预编库的取值（文件头「RocksDB 预编库的取值」）──
  # bin 的 required-features 经 [features] 点到 dep:rocksdb，那一例的仓根放一份依赖 rocksdb 的 验证包清单、进程环境里四个取值是空的，
  # 假 cargo 不带 rocksdb-prebuilt.py 认的标记：起 cargo 之前 entry 判红、退 2，假 cargo 一次都没起（上面那几例的 bin 编不到 rocksdb，不判）
  prepare_case rocksdb-entry-refused PASS 'required-features = ["fake-store"]'
  printf '\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n\n[features]\nfake-store = ["fake-inner"]\nfake-inner = ["dep:rocksdb"]\n' \
    >>"$scratch/rocksdb-entry-refused/scripts/experiment-apparatus/bench/Cargo.toml"
  mkdir -p "$scratch/rocksdb-entry-refused/crates/singlefs-verification"
  printf '[package]\nname = "singlefs-verification"\nversion = "0.0.0"\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n' \
    >"$scratch/rocksdb-entry-refused/crates/singlefs-verification/Cargo.toml"
  run_case rocksdb-entry-refused ../mutation/tables/caught-only.tsv ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC=
  checked=$((checked + 1))
  status="$(cat "$scratch/rocksdb-entry-refused.rc")"
  if [[ "$status" != 2 || -s "$scratch/rocksdb-entry-refused.cargo" ]] || ! grep -qF "rocksdb-prebuilt entry：" "$scratch/rocksdb-entry-refused.stderr"; then
    fail "bin 的 required-features 点到 rocksdb、进程环境里没有预编库的取值，应当在起 cargo 之前退 2、假 cargo 一次都不起、stderr 里有 rocksdb-prebuilt entry 的红句，实际退 $status、假 cargo 起了 $(wc -l <"$scratch/rocksdb-entry-refused.cargo") 次：$(cat "$scratch/rocksdb-entry-refused.out")"
  fi

  # 基线就是红的（没改的源码上假 cargo 演 RED）：退 2，点出红的测试名、打末尾几行，整份基线输出留在 TMPDIR 下并点出路径
  prepare_case baseline-red RED
  mkdir -p "$scratch/baseline-red/tmp"
  run_case baseline-red ../mutation/tables/comment-only.tsv TMPDIR="$scratch/baseline-red/tmp"
  checked=$((checked + 1))
  status="$(cat "$scratch/baseline-red.rc")"
  kept_logs=("$scratch/baseline-red/tmp"/singlefs-mutate-baseline-*.log)
  if [[ "$status" != 2 ]] || ! grep -qF "基线就是红的" "$scratch/baseline-red.stderr" || ! grep -qF "红的测试：tests::adds" "$scratch/baseline-red.stderr" \
     || [[ ! -s "${kept_logs[0]}" ]] || ! grep -qF "整份基线输出留在 ${kept_logs[0]}" "$scratch/baseline-red.stderr" \
     || ! grep -qF "test result: FAILED. 1 passed; 1 failed" "$scratch/baseline-red.stderr"; then
    fail "基线就是红的时应当退 2、点出红的测试 tests::adds、打基线输出末尾几行、整份输出留在 TMPDIR 下并点出路径，实际退 $status：$(cat "$scratch/baseline-red.out")"
  fi

  # #[should_panic] 的用例：结果行在名字与「 ... 」之间多一段「 - should panic」，取名字（lib_libtest_result_lines.py）时不进名字
  prepare_case should-panic
  run_case should-panic ../mutation/tables/should-panic.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/should-panic.rc")"
  if [[ "$status" != 0 ]] || ! grep -qxF "✅ [抓到] 红：tests::answer_panics" "$scratch/should-panic.stdout"; then
    fail "#[should_panic] 的用例红了应当记「✅ [抓到] 红：tests::answer_panics」（名字不带「 - should panic」）、整轮退 0，实际退 $status：$(cat "$scratch/should-panic.out")"
  fi
  prepare_case baseline-red-should-panic PANICS
  mkdir -p "$scratch/baseline-red-should-panic/tmp"
  run_case baseline-red-should-panic ../mutation/tables/comment-only.tsv TMPDIR="$scratch/baseline-red-should-panic/tmp"
  checked=$((checked + 1))
  status="$(cat "$scratch/baseline-red-should-panic.rc")"
  if [[ "$status" != 2 ]] || ! grep -qxF "        红的测试：tests::answer_panics" "$scratch/baseline-red-should-panic.stderr"; then
    fail "基线里 should panic 的用例红了，应当退 2、点出红的测试「tests::answer_panics」（不带「 - should panic」），实际退 $status：$(cat "$scratch/baseline-red-should-panic.out")"
  fi

  # 参数不够三个：退 2、打用法与一条例子，不报 unbound variable，一次 cargo 都不调
  prepare_case usage
  : >"$scratch/usage.cargo"
  if (cd "$scratch/usage/scripts/experiment-apparatus" && env PATH="$scratch/usage/bin:$PATH" FAKE_CARGO_LOG="$scratch/usage.cargo" \
        bash "$case_script" fake-bench >"$scratch/usage.stdout" 2>"$scratch/usage.stderr"); then
    echo 0 >"$scratch/usage.rc"
  else
    echo $? >"$scratch/usage.rc"
  fi
  checked=$((checked + 1))
  status="$(cat "$scratch/usage.rc")"
  if [[ "$status" != 2 ]] || ! grep -qF "要三个参数，给了 1 个" "$scratch/usage.stderr" \
     || ! grep -qF "→ 怎么办：在 scripts/experiment-apparatus/ 目录里照这个例子跑：bash ../mutation/mutate.sh " "$scratch/usage.stderr" \
     || grep -qF "unbound variable" "$scratch/usage.stderr" || [[ -s "$scratch/usage.cargo" ]]; then
    fail "参数不够三个时应当退 2、打用法与一条例子、不报 unbound variable、一次 cargo 都不调，实际退 $status：$(cat "$scratch/usage.stderr")"
  fi

  # ── 平常跑带只供自证的变量 ──
  # 不是自证起的子进程（标记清空）而带着只供自证的变量：每个变量各试一次，都退 2、点名那个变量、一次 cargo 都不调
  local variable variable_value outside_label
  for variable in "${TEST_ONLY_VARIABLES[@]}"; do
    case "$variable" in
      # 调用方设的开关接在后面传进去；关守卫项单独设着时原样单独传（同写会被拒，弄坏开关 honorbreakoutsideselftest 就红不出这一格）
      MUTATE_BREAK) if [[ "$BROKEN_JUDGEMENT" == "$GUARD_OFF_ITEM" ]]; then variable_value="$GUARD_OFF_ITEM"; else variable_value="countblind${BROKEN_JUDGEMENT:+,$BROKEN_JUDGEMENT}"; fi ;;
      *) variable_value=1 ;;
    esac
    outside_label="outside-selftest-$variable"
    prepare_case "$outside_label"
    run_case "$outside_label" ../mutation/tables/caught-only.tsv MUTATE_SELFTEST_MARK= "$variable=$variable_value"
    status="$(cat "$scratch/$outside_label.rc")"
    checked=$((checked + 1))
    if [[ "$status" != 2 || -s "$scratch/$outside_label.cargo" ]] || ! grep -qF "只给自证用的变量" "$scratch/$outside_label.stderr" \
       || ! grep -qF "$variable" "$scratch/$outside_label.stderr"; then
      fail "平常跑时带了只供自证的变量 $variable 应当退 2、点名它、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/$outside_label.cargo") 次：$(cat "$scratch/$outside_label.out")"
    fi
  done
  # 关守卫项与别的项同写（guard_off_honored）：进程里判一次（弄坏开关 honorbreakoutsideselftest-with-others 只在这里认），再起一次平常跑的子进程
  in_selftest_process=1
  checked=$((checked + 1))
  if guard_off_honored "$GUARD_OFF_ITEM,countblind" || ! guard_off_honored "$GUARD_OFF_ITEM" || guard_off_honored countblind; then
    fail "关守卫项与别的项同写：guard_off_honored 应当只认单独一项 $GUARD_OFF_ITEM，同写的「$GUARD_OFF_ITEM,countblind」与不带它的「countblind」都不认"
  fi
  in_selftest_process=0
  prepare_case outside-selftest-guard-combined
  run_case outside-selftest-guard-combined ../mutation/tables/caught-only.tsv MUTATE_SELFTEST_MARK= MUTATE_BREAK="$GUARD_OFF_ITEM,countblind"
  status="$(cat "$scratch/outside-selftest-guard-combined.rc")"
  checked=$((checked + 1))
  if [[ "$status" != 2 || -s "$scratch/outside-selftest-guard-combined.cargo" ]] \
     || ! grep -qF "关守卫项 $GUARD_OFF_ITEM 只在它是 MUTATE_BREAK 唯一一项时认" "$scratch/outside-selftest-guard-combined.stderr"; then
    fail "平常跑时关守卫项与别的项同写（MUTATE_BREAK=$GUARD_OFF_ITEM,countblind）应当退 2、点名关守卫项只在它是唯一一项时认、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/outside-selftest-guard-combined.cargo") 次：$(cat "$scratch/outside-selftest-guard-combined.out")"
  fi
  # 不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY（文件头「按条入库」）：自证起的子进程照常带着 run_case 给的库目录，只把临时服务的标记设成空串
  #   （selftest_service_problem 照「没设」答；连哪一处照旧指临时服务，放过了也不写真服务）：退 2、点名它、一次 cargo 都不调
  prepare_case library-directory-outside-service
  run_case library-directory-outside-service ../mutation/tables/caught-only.tsv PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET=
  status="$(cat "$scratch/library-directory-outside-service.rc")"
  checked=$((checked + 1))
  if [[ "$status" != 2 || -s "$scratch/library-directory-outside-service.cargo" ]] \
     || ! grep -qF "不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY（「$scratch/library-directory-outside-service/library」" "$scratch/library-directory-outside-service.stderr"; then
    fail "不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY 应当退 2、点名它、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/library-directory-outside-service.cargo") 次：$(cat "$scratch/library-directory-outside-service.out")"
  fi
  # 取不到 git common-dir（文件头「按条入库」）：不给 MUTATE_LIBRARY_DIRECTORY，GIT_CEILING_DIRECTORIES 挡在 $scratch，
  #   临时目录落在哪个 git 仓底下都找不到 common-dir：退 2、点名那棵树、一次 cargo 都不调（开关 libraryrootfallback 下退到另一个键前缀、照常跑完）
  prepare_case library-common-directory-unavailable
  run_case library-common-directory-unavailable ../mutation/tables/caught-only.tsv MUTATE_LIBRARY_DIRECTORY= GIT_CEILING_DIRECTORIES="$scratch"
  status="$(cat "$scratch/library-common-directory-unavailable.rc")"
  checked=$((checked + 1))
  if [[ "$status" != 2 || -s "$scratch/library-common-directory-unavailable.cargo" ]] \
     || ! grep -qF "取不到 $scratch/library-common-directory-unavailable 的 git common-dir" "$scratch/library-common-directory-unavailable.stderr" \
     || ! grep -qF "git init" "$scratch/library-common-directory-unavailable.stderr"; then
    fail "取不到 git common-dir 时应当退 2、点名 $scratch/library-common-directory-unavailable、出路写 git init、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/library-common-directory-unavailable.cargo") 次：$(cat "$scratch/library-common-directory-unavailable.out")"
  fi

  # ── deny 警告（文件头「deny 警告」那一段）──
  # 一条的表：替换文留死代码（DEADCODE），假 cargo 照 deny 判、RUSTFLAGS 里没有 --cap-lints warn 就编不过。MUTATE_JOBS=1：
  # 平常跑调 cargo 3 次（基线、那一条、还原之后）；每次收到的 RUSTFLAGS 逐字相同，调用方设了的接在前面
  # 后两例环境里设了 CARGO_ENCODED_RUSTFLAGS（空串也算）：假 cargo 照真 cargo 只读它，所以 mutate.sh 得把 cap 接进它（0x1f 分隔，调用方的在前）
  local rustflags_label rustflags_given rustflags_wanted wanted_line wanted_calls matched_rustflags
  local unit_separator=$'\x1f' encoded_set encoded_given encoded_wanted matched_encoded received_encoded
  for rustflags_label in cap-lints cap-lints-existing encoded-empty encoded-other; do
    encoded_set=0; encoded_given=""; encoded_wanted="（没设）"
    case "$rustflags_label" in
      cap-lints) rustflags_given=""; rustflags_wanted="--cap-lints warn"; wanted_line="✅ [留死代码] 红：tests::adds"; wanted_calls=3
                 case_leading_arguments=() ;;
      cap-lints-existing) rustflags_given="-Cdebuginfo=0"; rustflags_wanted="-Cdebuginfo=0 --cap-lints warn"; wanted_line="✅ [留死代码] 红：tests::adds"; wanted_calls=3
                          case_leading_arguments=() ;;
      encoded-empty) rustflags_given=""; rustflags_wanted="--cap-lints warn"; wanted_line="✅ [留死代码] 红：tests::adds"; wanted_calls=3
                     encoded_set=1; encoded_given=""; encoded_wanted="--cap-lints${unit_separator}warn"
                     case_leading_arguments=() ;;
      encoded-other) rustflags_given="-Cdebuginfo=0"; rustflags_wanted="-Cdebuginfo=0 --cap-lints warn"; wanted_line="✅ [留死代码] 红：tests::adds"; wanted_calls=3
                     encoded_set=1; encoded_given="-Copt-level=1${unit_separator}-Cdebuginfo=2"; encoded_wanted="-Copt-level=1${unit_separator}-Cdebuginfo=2${unit_separator}--cap-lints${unit_separator}warn"
                     case_leading_arguments=() ;;
    esac
    prepare_case "$rustflags_label"
    if ((encoded_set)); then
      run_case "$rustflags_label" ../mutation/tables/dead-code.tsv MUTATE_JOBS=1 RUSTFLAGS="$rustflags_given" CARGO_ENCODED_RUSTFLAGS="$encoded_given"
    else
      run_case "$rustflags_label" ../mutation/tables/dead-code.tsv MUTATE_JOBS=1 RUSTFLAGS="$rustflags_given"
    fi
    status="$(cat "$scratch/$rustflags_label.rc")"
    checked=$((checked + 1))
    if [[ "$status" != 0 ]] || ! grep -qxF -- "$wanted_line" "$scratch/$rustflags_label.stdout"; then
      fail "deny 警告：平常跑（调用方 RUSTFLAGS=「$rustflags_given」）留死代码那一条应当判抓到、整轮退 0（整行「$wanted_line」），实际退 $status：$(cat "$scratch/$rustflags_label.out")"
    fi
    checked=$((checked + 1))
    # 数逐字等于期望的那几行（不取「不等于的」再判空：收到空串的那几行取出来是空行，$( ) 一去尾就看不见了）
    matched_rustflags="$(cut -d'|' -f3- "$scratch/$rustflags_label.cargo.rustflags" 2>/dev/null | grep -cxF -- "$rustflags_wanted")"
    if [[ "$(grep -c . "$scratch/$rustflags_label.cargo.rustflags" 2>/dev/null)" != "$wanted_calls" || "$matched_rustflags" != "$wanted_calls" ]] \
       || [[ "$(cut -d'|' -f1 "$scratch/$rustflags_label.cargo.rustflags" | sort -u | tr '\n' ' ')" != "DEADCODE PASS " ]]; then
      fail "deny 警告：$rustflags_label 里每个 cargo 收到的 RUSTFLAGS 应当逐字是「$rustflags_wanted」、调 $wanted_calls 次、基线（PASS）与那一条（DEADCODE）都在，实际：$(cat "$scratch/$rustflags_label.cargo.rustflags" 2>/dev/null | tr '\n' ' ')"
    fi
    # CARGO_ENCODED_RUSTFLAGS：没设的两例每个 cargo 都收到「（没设）」（不凭空多出一个），设了的两例逐字是 encoded_wanted（0x1f 在 cat -v 里显示成 ^_）
    checked=$((checked + 1))
    matched_encoded="$(cut -d'|' -f3- "$scratch/$rustflags_label.cargo.encoded" 2>/dev/null | grep -cxF -- "$encoded_wanted")"
    if [[ "$(grep -c . "$scratch/$rustflags_label.cargo.encoded" 2>/dev/null)" != "$wanted_calls" || "$matched_encoded" != "$wanted_calls" ]]; then
      fail "deny 警告：$rustflags_label 里每个 cargo 收到的 CARGO_ENCODED_RUSTFLAGS 应当逐字是「$(printf '%s' "$encoded_wanted" | cat -v)」（0x1f 显示成 ^_）、调 $wanted_calls 次，实际：$(cat -v "$scratch/$rustflags_label.cargo.encoded" 2>/dev/null | tr '\n' ' ')"
    fi
    # stderr 开跑那一行报的与 cargo 实收的一致：设了 CARGO_ENCODED_RUSTFLAGS 的报它（0x1f 显示成 ␟）、不再说 RUSTFLAGS；没设的照旧报 RUSTFLAGS
    checked=$((checked + 1))
    if ((encoded_set)); then
      received_encoded="$(head -n 1 "$scratch/$rustflags_label.cargo.encoded" 2>/dev/null | cut -d'|' -f3- | sed "s/$unit_separator/␟/g")"
      if [[ -z "$received_encoded" ]] || ! grep -qF -- "mutate: 每个 cargo 都带 CARGO_ENCODED_RUSTFLAGS=「$received_encoded」" "$scratch/$rustflags_label.stderr" \
         || grep -qF -- "mutate: 每个 cargo 都带 RUSTFLAGS=" "$scratch/$rustflags_label.stderr"; then
        fail "deny 警告：$rustflags_label 里 stderr 应当报 cargo 实收的「CARGO_ENCODED_RUSTFLAGS=「$received_encoded」」（不再说 RUSTFLAGS），实际 stderr：$(cat "$scratch/$rustflags_label.stderr")"
      fi
    elif ! grep -qF -- "mutate: 每个 cargo 都带 RUSTFLAGS=「$rustflags_wanted」" "$scratch/$rustflags_label.stderr"; then
      fail "deny 警告：$rustflags_label 里 stderr 应当报「每个 cargo 都带 RUSTFLAGS=「$rustflags_wanted」」，实际 stderr：$(cat "$scratch/$rustflags_label.stderr")"
    fi
  done

  # --binary-of：显式 [[bin]] 的打声明的名字；没写 [[bin]] 的打文件名去掉 .rs；源文件不在退 2
  local binary_output binary_status
  prepare_case binary-of
  printf 'fn main() {}\n' >"$scratch/binary-of/scripts/experiment-apparatus/bench/src/bin/other_tool.rs"
  for wanted in "bench/src/bin/fake_bench.rs fake-bench" "bench/src/bin/other_tool.rs other_tool"; do
    checked=$((checked + 1))
    if binary_output="$(cd "$scratch/binary-of/scripts/experiment-apparatus" && bash "$case_script" --binary-of "${wanted%% *}" 2>&1)"; then binary_status=0; else binary_status=$?; fi
    [[ "$binary_status" == 0 && "$binary_output" == "${wanted##* }" ]] \
      || fail "--binary-of ${wanted%% *} 应当退 0、只打「${wanted##* }」，实际退 $binary_status：$binary_output"
  done
  checked=$((checked + 1))
  if binary_output="$(cd "$scratch/binary-of/scripts/experiment-apparatus" && bash "$case_script" --binary-of bench/src/bin/absent.rs 2>&1)"; then binary_status=0; else binary_status=$?; fi
  [[ "$binary_status" == 2 && "$binary_output" == *"→ 怎么办"* ]] || fail "--binary-of 给的源文件不在时应当退 2、给出路，实际退 $binary_status：$binary_output"

  # 按条入库（文件头「按条入库」）：被测源码换成一份带 #[test] 的（假 cargo 照旧只看 BEHAVIOR），抓到的那一条第二趟复用、一次 cargo 都不起（unit-library）；
  # 改了被改坏的函数调到的 helper 之后那一条现判（unit-library-callee）；原文只在注释里的那一条拒算、每趟照跑。
  # 弄坏开关 libraryunread（不读库）让「同一条第二趟复用」判错，calleesignored（键只取包住原文那个函数自己的指纹）让「改被改坏函数调到的函数后现判」判错
  write_unit_library_source() { # <例的目录>
    cat >"$1/scripts/experiment-apparatus/bench/src/bin/fake_bench.rs" <<'UNIT_SOURCE'
// 注释行
fn helper() -> u32 { 2 }
fn adds() -> u32 { 1 + helper() } // BEHAVIOR: PASS
fn main() { let _ = adds(); }

#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::adds(), 3); }
}
UNIT_SOURCE
  }
  real_cargo_calls() { awk -F'|' '$4 ~ /^(test|check)( |$)/ { count++ } END { print count + 0 }' "$1"; }   # <假 cargo 的记录>：test / check 起了几次
  prepare_case unit-library
  write_unit_library_source "$scratch/unit-library"
  run_case unit-library ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library.out")"; status="$(cat "$scratch/unit-library.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 ]] && grep -qF "✅ [抓到] 红：tests::adds" <<<"$output" && grep -qF "、现判 1 条、复用 0 条、拒算 0 条" <<<"$output" \
    || fail "按条入库头一趟应当判抓到、计数行报现判 1 条复用 0 条，实际退 $status：$output"
  run_case unit-library ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library.out")"; status="$(cat "$scratch/unit-library.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library.cargo")" == 0 ]] && grep -qF "✅ [抓到] 红：tests::adds" "$scratch/unit-library.stdout" \
     && grep -qF "、现判 0 条、复用 1 条、拒算 0 条" "$scratch/unit-library.stdout" && grep -qF "全部复用：表里 1 条都按判定库复用，没起 cargo" "$scratch/unit-library.stdout" \
    || fail "同一条第二趟应当复用：判定照旧是抓到、计数行报复用 1 条、打「全部复用」、一次 cargo 都不调，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library.cargo") 次：$output"
  run_case unit-library ../mutation/tables/comment-only.tsv
  output="$(cat "$scratch/unit-library.out")"
  checked=$((checked + 1))
  grep -qF "、现判 0 条、复用 0 条、拒算 1 条" <<<"$output" && grep -qF "拒算（照跑、不复用）：[改注释]" <<<"$output" && [[ "$(real_cargo_calls "$scratch/unit-library.cargo")" -ge 2 ]] \
    || fail "原文只在注释里的那一条应当拒算、照跑（计数行报拒算 1 条、stderr 写为什么），实际 cargo 起了 $(real_cargo_calls "$scratch/unit-library.cargo") 次：$output"
  # 记红、撤绿、下趟现判（文件头「按条入库」，unit-library-red）：三条抓到的表（替换文各不相同，三把键：名字不进键，替换文相同的几条共用一把，记红会翻掉全部）先判一趟入库；
  # 复用哨兵强制现判一条（哨兵取 2 条，剩下那一条强制现判，其余照复用），假 cargo 这一趟演测试全绿（FAKE_CARGO_BEHAVIOR_OVERRIDE=PASS，不进键）⇒ 那一条判没红、整轮退 1，照同一把键记红盖掉抓到；
  # 下一趟平常跑：那一条现判（stderr 写「上次判红（notred）」、cargo 照起）、判回抓到，别的两条复用；再下一趟三条都复用、一次 cargo 都不起。
  # 弄坏开关 redreused（判红的照旧复用）让「下一趟现判」判错
  prepare_case unit-library-red
  write_unit_library_source "$scratch/unit-library-red"
  run_case unit-library-red ../mutation/tables/three-distinct.tsv
  run_case unit-library-red ../mutation/tables/three-distinct.tsv SINGLEFS_REUSE_SENTINEL_COUNT=2 SINGLEFS_REUSE_SENTINEL_SEED=5 FAKE_CARGO_BEHAVIOR_OVERRIDE=PASS
  output="$(cat "$scratch/unit-library-red.out")"; status="$(cat "$scratch/unit-library-red.rc")"
  local red_name
  red_name="$(grep -oE '现判：\[[^]]+\] 强制现判' <<<"$output" | head -1 | sed -E 's/^现判：\[(.*)\] 强制现判$/\1/')"
  checked=$((checked + 1))
  [[ "$status" == 1 && -n "$red_name" ]] && grep -qF "❌ [$red_name]" <<<"$output" \
    || fail "记红撤绿：同一把键经哨兵强制现判一条、判出没红应当整轮退 1、打「❌ [<被强制现判的那一条>]」，实际退 $status、强制现判的是「$red_name」：$output"
  run_case unit-library-red ../mutation/tables/three-distinct.tsv
  output="$(cat "$scratch/unit-library-red.out")"; status="$(cat "$scratch/unit-library-red.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library-red.cargo")" -ge 2 ]] && grep -qF "现判：[$red_name] 上次判红（notred）" <<<"$output" \
     && grep -qF "、现判 1 条、复用 2 条、拒算 0 条" <<<"$output" && grep -qF "✅ [$red_name] 红：tests::adds" "$scratch/unit-library-red.stdout" \
    || fail "记红撤绿：库里同一把键最近一次是没红，下一趟应当现判（cargo 照起、stderr 写上次判红、计数行报现判 1 条复用 2 条）并判回抓到，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library-red.cargo") 次：$output"
  run_case unit-library-red ../mutation/tables/three-distinct.tsv
  output="$(cat "$scratch/unit-library-red.out")"; status="$(cat "$scratch/unit-library-red.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library-red.cargo")" == 0 ]] && grep -qF "、现判 0 条、复用 3 条、拒算 0 条" <<<"$output" \
    || fail "记红撤绿：现判又判抓到之后再下一趟应当复用（盖回绿）、一次 cargo 都不起，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library-red.cargo") 次：$output"
  # 改了被改坏的函数（adds）调到的 helper：包住原文那个函数的闭包变了，键跟着变，那一条现判
  prepare_case unit-library-callee
  write_unit_library_source "$scratch/unit-library-callee"
  run_case unit-library-callee ../mutation/tables/caught-only.tsv
  CALLEE_SOURCE="$scratch/unit-library-callee/scripts/experiment-apparatus/bench/src/bin/fake_bench.rs" python3 - <<'PY' || fail "改不了 helper（自证自己的样本出错）"
import os
path = os.environ["CALLEE_SOURCE"]
text = open(path, encoding="utf-8").read()
assert text.count("fn helper() -> u32 { 2 }") == 1
open(path, "w", encoding="utf-8").write(text.replace("fn helper() -> u32 { 2 }", "fn helper() -> u32 { 3 }"))
PY
  run_case unit-library-callee ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library-callee.out")"; status="$(cat "$scratch/unit-library-callee.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library-callee.cargo")" -ge 2 ]] && grep -qF "、现判 1 条、复用 0 条、拒算 0 条" <<<"$output" \
     && grep -qF "现判：[抓到] 库里没有这把键" <<<"$output" \
    || fail "改了被改坏函数调到的函数之后那一条应当现判（cargo 照起、计数行报现判 1 条、stderr 写库里没有这把键），实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library-callee.cargo") 次：$output"

  # 依赖版本行（lib_mutation_units.py 文件头「依赖版本行」）：只收包住原文那个函数的闭包里真点名的外部 crate 与它们的传递依赖。
  # prune-server 不可用（文件头「按条入库」）：查库那一段连得上临时服务，每条判完写库那一步改连一个没人听的地址（只供自证的 MUTATE_SELFTEST_RECORD_UNREACHABLE）
  # ⇒ 整轮退 75、stderr 写「prune-server 不可用」，不当「没写进、下一趟照跑」放过。弄坏开关 recordunavailableignored（75 照「没写进」放过）下退 0，这一格判错
  local unreachable_address
  unreachable_address="$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print("127.0.0.1:%d" % s.getsockname()[1]); s.close()')"
  prepare_case record-unavailable
  write_unit_library_source "$scratch/record-unavailable"
  run_case record-unavailable ../mutation/tables/caught-only.tsv MUTATE_SELFTEST_RECORD_UNREACHABLE="$unreachable_address"
  status="$(cat "$scratch/record-unavailable.rc")"
  checked=$((checked + 1))
  [[ "$status" == 75 ]] && grep -qF "prune-server 不可用" "$scratch/record-unavailable.stderr" \
    || fail "判完写库那一步 prune-server 不可用：应当整轮退 75、stderr 写「prune-server 不可用」，实际退 $status：$(tail -n 4 "$scratch/record-unavailable.stderr")"
  # 判完写库那一步客户端退 3（客户端别的错，文件头「退出码」2）：写库那一步改连一个不带端口的地址（客户端判「TCP 地址要写成 <地址>:<端口>」、退 3）
  # ⇒ 别的照跑完、整轮退 2、stderr 写「判定库客户端报错」并转打客户端原话。弄坏开关 recordclienterrorignored（3 照「没写进」放过）下退 0，这一格判错
  prepare_case record-client-error
  write_unit_library_source "$scratch/record-client-error"
  run_case record-client-error ../mutation/tables/caught-only.tsv MUTATE_SELFTEST_RECORD_UNREACHABLE=selftest-address-without-port
  status="$(cat "$scratch/record-client-error.rc")"
  checked=$((checked + 1))
  [[ "$status" == 2 ]] && grep -qF "判定库客户端报错" "$scratch/record-client-error.stderr" && grep -qF "要写成 <地址>:<端口>" "$scratch/record-client-error.stderr" \
    || fail "判完写库那一步客户端退 3：应当整轮退 2、stderr 写「判定库客户端报错」并转打客户端那一句「要写成 <地址>:<端口>」，实际退 $status：$(tail -n 4 "$scratch/record-client-error.stderr")"
  # 查库那一段客户端退 3：重连总时长写成不是数（临时服务底下才认这个变量，客户端判「要是正数（秒）」、退 3）
  # ⇒ 退 2、stderr 写「判定库客户端报错」，一次 cargo test / check 都不起（不退回每条现判）。弄坏开关 planclienterrorfallback 下照起 cargo，这一格判错
  prepare_case plan-client-error
  write_unit_library_source "$scratch/plan-client-error"
  run_case plan-client-error ../mutation/tables/caught-only.tsv SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=selftest-not-a-number
  status="$(cat "$scratch/plan-client-error.rc")"
  checked=$((checked + 1))
  [[ "$status" == 2 && "$(real_cargo_calls "$scratch/plan-client-error.cargo")" == 0 ]] && grep -qF "判定库客户端报错（退 3" "$scratch/plan-client-error.stderr" \
    || fail "查库那一段客户端退 3：应当退 2、stderr 写「判定库客户端报错（退 3」、一次 cargo 都不调，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/plan-client-error.cargo") 次：$(tail -n 4 "$scratch/plan-client-error.stderr")"
  # 夹具：bench 声明 extdep、other、alias-dep（package = "aliased"）三个外部依赖，Cargo.lock 里 extdep 带一个传递依赖 inner；
  # adds 调 helper、helper 点 extdep::，other 只在闭包外的 unrelated 里点。头一趟库里那一条的依赖版本行逐字是 extdep、inner 两行；
  # 不点外部 crate 的 unit-library 那一例是空的；改 other 的版本第二趟照样复用、一次 cargo 都不起；改 inner 的版本现判；
  # helper 改点 alias_dep:: 之后依赖版本行逐字是 aliased 一行（认 package = 改名）。
  # 弄坏开关 dependenciesbycrate（按整包收）让头一趟与改名那一趟多出别的行、改 other 之后现判
  library_dependency_lines() { # <按条判定库的目录> [<之前已有的记录名清单>]：库里新写下的那一条判定（不在清单里的恰好一条；不给清单是库里恰好一条）
    # 记的依赖版本行，一行一条；空的打「（空）」。判定库在临时 prune-server 里（没有修改时刻可比，按「之前有哪几条」认新写下的那一条）
    python3 - "$(dirname "$SELF_PATH")/lib_mutation_units.py" "$1" "${2:-}" <<'PY'
import importlib.util, sys
specification = importlib.util.spec_from_file_location("lib_mutation_units_for_mutate_selftest", sys.argv[1])
units = importlib.util.module_from_spec(specification)
sys.modules[specification.name] = units
specification.loader.exec_module(units)
library = units.Library(sys.argv[2])
earlier = set(open(sys.argv[3], encoding="utf-8").read().split()) if sys.argv[3] else set()
names = [name for name in library.stored_names("records") if name not in earlier]
if len(names) != 1:
    sys.exit(f"库里新写下的判定要恰好一条，实际 {len(names)} 条：{names[:3]}")
fields = library.read("records", names[0])
lines = (library.read_json("compositions", fields.get("dependency", "")) or {}).get("lines", [])
print("\n".join(lines) if lines else "（空）")
PY
  }
  write_dependency_lock() { # <Cargo.lock 的路径> <other 的版本> <inner 的版本>
    local registry='registry+https://example.invalid/index'
    printf 'version = 4\n\n[[package]]\nname = "aliased"\nversion = "3.0.0"\nsource = "%s"\nchecksum = "33"\n\n[[package]]\nname = "bench"\nversion = "0.0.0"\ndependencies = [\n "aliased",\n "extdep",\n "other",\n]\n\n[[package]]\nname = "extdep"\nversion = "1.0.0"\nsource = "%s"\nchecksum = "aa"\ndependencies = [\n "inner",\n]\n\n[[package]]\nname = "inner"\nversion = "%s"\nsource = "%s"\nchecksum = "inner-%s"\n\n[[package]]\nname = "other"\nversion = "%s"\nsource = "%s"\nchecksum = "other-%s"\n' \
      "$registry" "$registry" "$3" "$registry" "$3" "$2" "$registry" "$2" >"$1"
  }
  write_dependency_source() { # <例的目录> <helper 点的外部 crate 的代码名>
    cat >"$1/scripts/experiment-apparatus/bench/src/bin/fake_bench.rs" <<DEPENDENCY_SOURCE
// 注释行
fn helper() -> u32 { $2::shift(2) }
fn adds() -> u32 { 1 + helper() } // BEHAVIOR: PASS
fn unrelated() -> u32 { other::value() }
fn main() { let _ = adds(); let _ = unrelated(); }

#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::adds(), 3); }
}
DEPENDENCY_SOURCE
  }
  local dependency_case="$scratch/unit-library-dependency" dependency_registry='registry+https://example.invalid/index' expected_dependency_lines recorded_dependency_lines
  prepare_case unit-library-dependency
  printf '\n[dependencies]\nextdep = "1.0"\nother = "2.0"\nalias-dep = { package = "aliased", version = "3.0" }\n' >>"$dependency_case/scripts/experiment-apparatus/bench/Cargo.toml"
  write_dependency_lock "$dependency_case/scripts/experiment-apparatus/Cargo.lock" 2.0.0 0.5.0
  write_dependency_source "$dependency_case" extdep
  run_case unit-library-dependency ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library-dependency.out")"; status="$(cat "$scratch/unit-library-dependency.rc")"
  expected_dependency_lines="$(printf 'extdep\t1.0.0\t%s\taa\ninner\t0.5.0\t%s\tinner-0.5.0' "$dependency_registry" "$dependency_registry")"
  recorded_dependency_lines="$(library_dependency_lines "$dependency_case/library" 2>&1)"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$recorded_dependency_lines" == "$expected_dependency_lines" ]] && grep -qF "、现判 1 条、复用 0 条、拒算 0 条" <<<"$output" \
    || fail "依赖版本行：头一趟库里那一条应当逐字只记 extdep 与它的传递依赖 inner 两行（other 只在闭包外点、alias-dep 没人点），实际退 $status、记的是「$recorded_dependency_lines」：$output"
  recorded_dependency_lines="$(library_dependency_lines "$scratch/unit-library/library" 2>&1)"
  checked=$((checked + 1))
  [[ "$recorded_dependency_lines" == "（空）" ]] || fail "依赖版本行：闭包不点外部 crate 时（unit-library 那一例）应当是空的，实际记的是「$recorded_dependency_lines」"
  write_dependency_lock "$dependency_case/scripts/experiment-apparatus/Cargo.lock" 2.0.1 0.5.0
  run_case unit-library-dependency ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library-dependency.out")"; status="$(cat "$scratch/unit-library-dependency.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library-dependency.cargo")" == 0 ]] && grep -qF "、现判 0 条、复用 1 条、拒算 0 条" <<<"$output" \
    || fail "依赖版本行：改了闭包没点名的外部包 other 的版本之后那一条应当照样复用、一次 cargo 都不起，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library-dependency.cargo") 次：$output"
  write_dependency_lock "$dependency_case/scripts/experiment-apparatus/Cargo.lock" 2.0.1 0.5.1
  run_case unit-library-dependency ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library-dependency.out")"; status="$(cat "$scratch/unit-library-dependency.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library-dependency.cargo")" -ge 2 ]] && grep -qF "、现判 1 条、复用 0 条、拒算 0 条" <<<"$output" \
     && grep -qF "现判：[抓到] 库里没有这把键" <<<"$output" \
    || fail "依赖版本行：改了点名的 extdep 的传递依赖 inner 的版本之后那一条应当现判（stderr 写库里没有这把键），实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library-dependency.cargo") 次：$output"
  write_dependency_source "$dependency_case" alias_dep
  library_record_names "$dependency_case/library" >"$scratch/unit-library-dependency.records-before-alias"
  run_case unit-library-dependency ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library-dependency.out")"; status="$(cat "$scratch/unit-library-dependency.rc")"
  expected_dependency_lines="$(printf 'aliased\t3.0.0\t%s\t33' "$dependency_registry")"
  recorded_dependency_lines="$(library_dependency_lines "$dependency_case/library" "$scratch/unit-library-dependency.records-before-alias" 2>&1)"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$recorded_dependency_lines" == "$expected_dependency_lines" ]] \
    || fail "依赖版本行：helper 改点 alias_dep:: 之后库里那一条应当逐字只记 aliased 一行（清单里 alias-dep 写了 package = \"aliased\"），实际退 $status、记的是「$recorded_dependency_lines」：$output"

  # 复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」，格名 reuse-three-questions）：一例自己的目录与判定库，表里两条抓到——
  # 「改加法」改 adds（闭包含 helper）、「改孤函数」改 lonely（闭包含 lonely_helper，测试都不调它）。四趟：库是空的（两条现判）、同一份输入（①）、
  # 加一个没人调的函数（闭包外，②）、改 lonely_helper（只在「改孤函数」的闭包里，③ affected 改孤函数）；这四趟哨兵数设 1000（键相同的全当哨兵、不抽强制现判）。
  # 每趟的结局从 stderr 取：「现判：[名]」是现判（原因以「强制现判」起头的是强制现判）、「拒算（照跑、不复用）：[名]」是拒算，表里别的是复用。
  # 弄坏开关 libraryunread（不读库）让 ①②③ 全红，calleesignored（键不收被调的单元）让 ③ 红
  write_questions_source() { # <例的目录>
    cat >"$1/scripts/experiment-apparatus/bench/src/bin/fake_bench.rs" <<'QUESTIONS_SOURCE'
// 注释行
fn helper() -> u32 { 2 }
fn lonely_helper() -> u32 { 4 }
fn lonely() -> u32 { lonely_helper() }
fn adds() -> u32 { 1 + helper() } // BEHAVIOR: PASS
fn main() { let _ = adds(); let _ = lonely(); }

#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::adds(), 3); }
}
QUESTIONS_SOURCE
  }
  run_states() { # <标签> <结局文件>：这一例上一趟的 stderr 按表里每一条写「名<TAB>结局」
    awk -F'\t' -v stderr_file="$scratch/$1.stderr" '
      BEGIN { while ((getline text < stderr_file) > 0) {
                kind = (text ~ /^mutate: 现判：\[/) ? "现判" : (text ~ /^mutate: 拒算（照跑、不复用）：\[/) ? "拒算" : ""
                if (kind == "") continue
                split(text, parts, "["); item = parts[2]; sub(/\].*/, "", item)
                state[item] = (kind == "现判" && index(text, "] 强制现判")) ? "强制现判" : kind } }
      { print $1 "\t" ($1 in state ? state[$1] : "复用") }' "$scratch/$1/scripts/mutation/tables/questions.tsv" >"$2"
  }
  local question_case="$scratch/reuse-three-questions" question_source
  prepare_case reuse-three-questions
  write_questions_source "$question_case"
  question_source="$question_case/scripts/experiment-apparatus/bench/src/bin/fake_bench.rs"
  printf '改加法\tBEHAVIOR: PASS\tBEHAVIOR: RED\n改孤函数\tfn lonely() -> u32 { lonely_helper() }\tfn lonely() -> u32 { lonely_helper() + 1 } // BEHAVIOR: RED\n' \
    >"$question_case/scripts/mutation/tables/questions.tsv"
  run_case reuse-three-questions ../mutation/tables/questions.tsv SINGLEFS_REUSE_SENTINEL_COUNT=1000
  run_states reuse-three-questions "$scratch/questions.first"
  run_case reuse-three-questions ../mutation/tables/questions.tsv SINGLEFS_REUSE_SENTINEL_COUNT=1000
  run_states reuse-three-questions "$scratch/questions.second"
  QUESTION_SOURCE="$question_source" python3 - <<'PY' || fail "复用三问：改不了样本（加没人调的函数）"
import os
path = os.environ["QUESTION_SOURCE"]
text = open(path, encoding="utf-8").read()
assert text.count("\n#[cfg(test)]") == 1
open(path, "w", encoding="utf-8").write(text.replace("\n#[cfg(test)]", "fn spare_unused() -> u32 { 9 }\n\n#[cfg(test)]"))
PY
  run_case reuse-three-questions ../mutation/tables/questions.tsv SINGLEFS_REUSE_SENTINEL_COUNT=1000
  run_states reuse-three-questions "$scratch/questions.unrelated"
  QUESTION_SOURCE="$question_source" python3 - <<'PY' || fail "复用三问：改不了样本（改 lonely_helper）"
import os
path = os.environ["QUESTION_SOURCE"]
text = open(path, encoding="utf-8").read()
assert text.count("fn lonely_helper() -> u32 { 4 }") == 1
open(path, "w", encoding="utf-8").write(text.replace("fn lonely_helper() -> u32 { 4 }", "fn lonely_helper() -> u32 { 5 }"))
PY
  run_case reuse-three-questions ../mutation/tables/questions.tsv SINGLEFS_REUSE_SENTINEL_COUNT=1000
  run_states reuse-three-questions "$scratch/questions.unit-change"
  checked=$((checked + 1))
  if ! python3 "$(dirname "$SELF_PATH")/../gate/lib_reuse_checks.py" three-questions --script scripts/mutation/mutate.sh --first "$scratch/questions.first" \
       --second "$scratch/questions.second" --unrelated "$scratch/questions.unrelated" --unit-change "$scratch/questions.unit-change" --affected 改孤函数; then
    fail "复用三问 reuse-three-questions：上面 REUSE_SELFTEST 那一行点名了红在哪一问；四趟的结局：$(for run in first second unrelated unit-change; do printf '%s「%s」 ' "$run" "$(tr '\t\n' ':,' <"$scratch/questions.$run")"; done)；末趟 stderr：$(cat "$scratch/reuse-three-questions.stderr")"
  fi

  # 复用哨兵（lib_reuse_checks.py 文件头「二」，格名 reuse-sentinel）：一例自己的目录与判定库，三条抓到的表先跑一趟记库；
  # 哨兵 1 条、种子 5 那一趟：一条当哨兵复用、一条强制现判（判完与库里同为「caught」）、整轮退 0、种子打在 stderr；
  # 开着弄坏开关 libraryunread（不读库）再跑同一个种子：哨兵被现判，红「复用失灵」、整轮退 1（共用库的 REUSE_CHECKS_BREAK=rejudged-sentinel-passes 下这一格判错）
  prepare_case reuse-sentinel
  write_unit_library_source "$scratch/reuse-sentinel"
  run_case reuse-sentinel ../mutation/tables/three-caught.tsv
  run_case reuse-sentinel ../mutation/tables/three-caught.tsv SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5
  local sentinel_green_status sentinel_green_stderr
  sentinel_green_status="$(cat "$scratch/reuse-sentinel.rc")"; sentinel_green_stderr="$(cat "$scratch/reuse-sentinel.stderr")"
  run_case reuse-sentinel ../mutation/tables/three-caught.tsv SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5 \
    MUTATE_BREAK="libraryunread${BROKEN_JUDGEMENT:+,$BROKEN_JUDGEMENT}"
  status="$(cat "$scratch/reuse-sentinel.rc")"
  checked=$((checked + 1))
  if [[ "$sentinel_green_status" != 0 ]] || ! grep -qF "复用哨兵（变异表 three-caught.tsv）：种子 5（SINGLEFS_REUSE_SENTINEL_SEED=5 复现）" <<<"$sentinel_green_stderr" \
     || ! grep -qF "] 强制现判（复用哨兵抽中，种子 5" <<<"$sentinel_green_stderr" || ! grep -qF "与库里同为「caught」" <<<"$sentinel_green_stderr" \
     || [[ "$status" != 1 ]] || ! grep -qF "复用失灵：" "$scratch/reuse-sentinel.stderr"; then
    fail "复用哨兵 reuse-sentinel：哨兵 1 条、种子 5 那一趟应当退 0、种子打在 stderr、一条强制现判且与库里同为「caught」；不读库那一趟应当退 1、红「复用失灵」。照常那一趟退 $sentinel_green_status：$(grep -E '复用哨兵|强制现判|复用失灵' <<<"$sentinel_green_stderr" | tr '\n' ' ')；不读库那一趟退 $status：$(grep -E '复用哨兵|复用失灵' "$scratch/reuse-sentinel.stderr" | tr '\n' ' ')"
  fi

  # 副本范围（文件头「变异只改副本」一段）：装置用 include_str! 读 scripts/experiment-apparatus/ 之外的仓内文件（../../.claude/heavy-thread-default），
  # 副本根下链上、基线编得过、照常判抓到（include-outside）；认不出的写法与落到仓根之外的路径退 2 点名那一行、一次 cargo 都不起（include-unrecognized）。
  # 弄坏开关 includesunlinked（不链）让两例都判错
  prepare_case include-outside
  mkdir -p "$scratch/include-outside/.claude"
  printf '7\n' >"$scratch/include-outside/.claude/heavy-thread-default"
  printf 'pub const THREADS: &str = include_str!("../../.claude/heavy-thread-default");\n' >"$scratch/include-outside/scripts/experiment-apparatus/threads_probe.rs"
  run_case include-outside ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/include-outside.out")"; status="$(cat "$scratch/include-outside.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 ]] && grep -qF "✅ [抓到] 红：tests::adds" <<<"$output" && grep -qF "副本根下链了 1 份 scripts/experiment-apparatus/ 之外的仓内文件" <<<"$output" \
    || fail "装置 include_str 读仓根下别处的文件（../../.claude/heavy-thread-default）时副本根下应当链上、基线编得过、照常判抓到，实际退 $status：$output"
  prepare_case include-unrecognized
  printf 'pub const PIECE: &str = include_str!(env!("PIECE_PATH"));\npub const FAR: &str = include_str!("../../../../outside.txt");\n' \
    >"$scratch/include-unrecognized/scripts/experiment-apparatus/threads_probe.rs"
  run_case include-unrecognized ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/include-unrecognized.out")"; status="$(cat "$scratch/include-unrecognized.rc")"
  checked=$((checked + 1))
  [[ "$status" == 2 && "$(real_cargo_calls "$scratch/include-unrecognized.cargo")" == 0 ]] \
     && grep -qF "scripts/experiment-apparatus/threads_probe.rs:1：include_str! 的参数不是一个串字面量" <<<"$output" \
     && grep -qF 'scripts/experiment-apparatus/threads_probe.rs:2：编译期读的' <<<"$output" && grep -qF '落到仓根之外' <<<"$output" \
    || fail "认不出的 include_str! 写法与落到仓根之外的路径应当退 2、逐行点名、一次 cargo 都不起，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/include-unrecognized.cargo") 次：$output"

  # ── 每条 cargo test 的测试线程数（文件头「并行」）：调用方没设 RUST_TEST_THREADS 时取 max(1, 线程上限 ÷ 工作进程数)，线程上限照登记取法（这几例用 SINGLEFS_THREAD_CAP 顶）；
  # 调用方设了的原样传。假 cargo 把每次收到的 RUST_TEST_THREADS 记进 .cargo.test-threads
  local test_thread_lines test_thread_wanted floor_workers
  # test-threads：MUTATE_JOBS=4、四条的表，线程上限 10（工作进程数头一项同一个登记值 10）：开 4 个，每条 10 ÷ 4 = 2
  local thread_workers=4
  test_thread_wanted=$((10 / thread_workers))
  prepare_case test-threads
  run_case test-threads ../mutation/tables/four-caught.tsv MUTATE_JOBS=4 SINGLEFS_THREAD_CAP=10 RUST_TEST_THREADS=
  test_thread_lines="$(cat "$scratch/test-threads.cargo.test-threads" 2>/dev/null)"
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/test-threads.rc")" != 0 || "$(grep -c "^[A-Z]*|test|$test_thread_wanted\$" <<<"$test_thread_lines")" != $((2 * thread_workers + 4)) \
        || "$(grep -c . <<<"$test_thread_lines")" != $((2 * thread_workers + 4)) ]] \
     || ! grep -qF "每条 cargo test 设 RUST_TEST_THREADS=$test_thread_wanted（线程上限 10 ÷ 工作进程 $thread_workers 个" "$scratch/test-threads.stderr"; then
    fail "测试线程：调用方没设 RUST_TEST_THREADS、线程上限 10、开 $thread_workers 个工作进程，每条 cargo test（共 $((2 * thread_workers + 4)) 次）收到的 RUST_TEST_THREADS 应当都是 $test_thread_wanted、stderr 报出来，实际退 $(cat "$scratch/test-threads.rc")，假 cargo 收到的（BEHAVIOR|子命令|值）：$(tr '\n' ' ' <<<"$test_thread_lines")；stderr：$(cat "$scratch/test-threads.stderr")"
  fi
  # test-threads-floor：线程上限 1、MUTATE_JOBS=2（工作进程数头一项同一个登记值 1：开 1 个），1 ÷ 1 取 1
  floor_workers=1
  prepare_case test-threads-floor
  run_case test-threads-floor ../mutation/tables/three-caught.tsv MUTATE_JOBS=2 SINGLEFS_THREAD_CAP=1 RUST_TEST_THREADS=
  test_thread_lines="$(cat "$scratch/test-threads-floor.cargo.test-threads" 2>/dev/null)"
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/test-threads-floor.rc")" != 0 || -z "$test_thread_lines" || -n "$(grep -v '^[A-Z]*|test|1$' <<<"$test_thread_lines")" ]]; then
    fail "测试线程：线程上限 1、开 $floor_workers 个工作进程，每条 cargo test 收到的 RUST_TEST_THREADS 应当都是 1（至少 1），实际退 $(cat "$scratch/test-threads-floor.rc")，假 cargo 收到的：$(tr '\n' ' ' <<<"$test_thread_lines")"
  fi
  # test-threads-caller：调用方设了 RUST_TEST_THREADS=7（线程上限 10 ÷ 1、2、3、4 个工作进程都算不出 7），原样传、不覆盖
  prepare_case test-threads-caller
  run_case test-threads-caller ../mutation/tables/four-caught.tsv MUTATE_JOBS=4 SINGLEFS_THREAD_CAP=10 RUST_TEST_THREADS=7
  test_thread_lines="$(cat "$scratch/test-threads-caller.cargo.test-threads" 2>/dev/null)"
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/test-threads-caller.rc")" != 0 || -z "$test_thread_lines" || -n "$(grep -v '^[A-Z]*|test|7$' <<<"$test_thread_lines")" ]] \
     || ! grep -qF "照调用方设的 RUST_TEST_THREADS=7 跑（不覆盖）" "$scratch/test-threads-caller.stderr"; then
    fail "测试线程：调用方设了的 RUST_TEST_THREADS=7 应当原样传给每条 cargo test、stderr 说不覆盖，实际退 $(cat "$scratch/test-threads-caller.rc")，假 cargo 收到的：$(tr '\n' ' ' <<<"$test_thread_lines")"
  fi

  # ── 在仓根下起（文件头「在仓根下起」）：只核目录判定与参数改写，不起 cargo（bin 名对不上退 6、--binary-of 只打名字，都在起 cargo 之前）──
  # run_from <标签> <从这一例目录起的子目录> <参数…>：在那里起 mutate.sh，stdout 落 .stdout、stderr 落 .stderr、退出码落 .rc，假 cargo 的调用记录落 .cargo
  run_from() {
    local label="$1" directory="$scratch/from-root/$2"
    shift 2
    : >"$scratch/$label.cargo"
    if (cd "$directory" && env PATH="$scratch/from-root/bin:$PATH" FAKE_CARGO_LOG="$scratch/$label.cargo" MUTATE_LIBRARY_DIRECTORY="$scratch/from-root/library" \
          bash "$case_script" "$@" >"$scratch/$label.stdout" 2>"$scratch/$label.stderr"); then
      echo 0 >"$scratch/$label.rc"
    else
      echo $? >"$scratch/$label.rc"
    fi
  }
  prepare_case from-root
  local root_apparatus="$scratch/from-root/scripts/experiment-apparatus"
  # kb 里的写法：bin 名故意写错，退 6 时报的复跑命令是改写过的参数，声明的 bin 名是 cd 进去之后才读得到的 bench/Cargo.toml 里的那一个
  run_from from-root-kb-form . wrong-name scripts/experiment-apparatus/bench/src/bin/fake_bench.rs scripts/mutation/tables/caught-only.tsv
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/from-root-kb-form.rc")" != 6 || -s "$scratch/from-root-kb-form.cargo" ]] \
     || ! grep -qF "在仓根下起的：cd 进了 $root_apparatus，参数「wrong-name scripts/experiment-apparatus/bench/src/bin/fake_bench.rs scripts/mutation/tables/caught-only.tsv」改写成「wrong-name bench/src/bin/fake_bench.rs ../mutation/tables/caught-only.tsv」" "$scratch/from-root-kb-form.stderr" \
     || ! grep -qF "在 Cargo.toml 里声明的二进制是 'fake-bench'" "$scratch/from-root-kb-form.stderr" \
     || ! grep -qF "用： mutate.sh fake-bench bench/src/bin/fake_bench.rs ../mutation/tables/caught-only.tsv" "$scratch/from-root-kb-form.stderr"; then
    fail "在仓根下起（kb 的写法、bin 名写错）：应当 cd 进 scripts/experiment-apparatus/、源文件剥掉 scripts/experiment-apparatus/、变异表改成 ../mutation/tables/…，读到声明的 fake-bench 后退 6、一次 cargo 都不起，实际退 $(cat "$scratch/from-root-kb-form.rc")、cargo 起了 $(grep -c . "$scratch/from-root-kb-form.cargo") 次：$(cat "$scratch/from-root-kb-form.stderr")"
  fi
  # 别的写法：./ 起头的源文件同样剥掉，scripts/ 之外的变异表接 ../../
  run_from from-root-other-form . wrong-name ./scripts/experiment-apparatus/bench/src/bin/fake_bench.rs elsewhere/table.tsv
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/from-root-other-form.rc")" != 6 || -s "$scratch/from-root-other-form.cargo" ]] \
     || ! grep -qF "用： mutate.sh fake-bench bench/src/bin/fake_bench.rs ../../elsewhere/table.tsv" "$scratch/from-root-other-form.stderr"; then
    fail "在仓根下起（./ 起头的源文件、scripts/ 之外的变异表）：参数应当改写成 bench/src/bin/fake_bench.rs 与 ../../elsewhere/table.tsv、退 6、一次 cargo 都不起，实际退 $(cat "$scratch/from-root-other-form.rc")：$(cat "$scratch/from-root-other-form.stderr")"
  fi
  # --binary-of：仓根起的源文件照样打出 bin 名，stderr 说 cd 进去了
  run_from from-root-binary-of . --binary-of scripts/experiment-apparatus/bench/src/bin/fake_bench.rs
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/from-root-binary-of.rc")" != 0 || "$(cat "$scratch/from-root-binary-of.stdout")" != fake-bench ]] \
     || ! grep -qF "在仓根下起的：cd 进了 $root_apparatus" "$scratch/from-root-binary-of.stderr"; then
    fail "在仓根下起 --binary-of scripts/experiment-apparatus/bench/src/bin/fake_bench.rs：应当退 0、stdout 只打 fake-bench、stderr 说 cd 进了 scripts/experiment-apparatus/，实际退 $(cat "$scratch/from-root-binary-of.rc")：$(cat "$scratch/from-root-binary-of.stdout" "$scratch/from-root-binary-of.stderr")"
  fi
  # 既不在仓根、也不在 scripts/experiment-apparatus/（这一例的 scripts/ 下）：照旧拒，退 2、一次 cargo 都不起
  run_from from-neither scripts fake_bench bench/src/bin/fake_bench.rs mutation/tables/caught-only.tsv
  checked=$((checked + 1))
  if [[ "$(cat "$scratch/from-neither.rc")" != 2 || -s "$scratch/from-neither.cargo" ]] \
     || ! grep -qF "要在 scripts/experiment-apparatus/ 下跑" "$scratch/from-neither.stderr" || grep -qF "在仓根下起的" "$scratch/from-neither.stderr"; then
    fail "既不在仓根、也不在 scripts/experiment-apparatus/ 下起：应当照旧退 2、报「要在 scripts/experiment-apparatus/ 下跑」、不认成仓根、一次 cargo 都不起，实际退 $(cat "$scratch/from-neither.rc")：$(cat "$scratch/from-neither.stderr")"
  fi

  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ mutate.sh 自检 $failures 处不对（查了 $checked 项）"
    echo "  → 怎么办：照上面逐条改——判结局的在 judge_mutation，起 cargo test 的在 run_cargo_test，基线与收尾计数在主流程，"
    echo "    并行的在 worker_count_and_head_line、run_worker 与 print_judged_rows_in_table_order，required-features 在 read_required_features；"
    echo "    平常跑拒只供自证的变量在 --selftest 那一支后面（running_under_selftest），工作进程数的头一项（每台默认重活线程数）在 worker_count_and_head_line；"
    echo "    --binary-of 在 find_declared_binary；"
    echo "    每个 cargo 带的 RUSTFLAGS 与 CARGO_ENCODED_RUSTFLAGS 在 MUTATE_RUSTFLAGS、MUTATE_ENCODED_RUSTFLAGS 与 RUSTFLAGS_ENVIRONMENT（run_cargo_test 用它）；"
    echo "    每条 cargo test 的 RUST_TEST_THREADS 在 TEST_THREADS_ENVIRONMENT（worker_count_and_head_line 之后算），在仓根下起的认法与参数改写在 relative_to_apparatus 那一段；"
    echo "    MUTATE_BREAK 设着的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ mutate.sh 自检通过（查了 $checked 项：抓到、没红、无效三种结局各归各类、计数行各栏对得上，\
正常的变异照旧判抓到退 0；\
基线红退 2 并点出红的测试、末尾几行与留下的整份日志，should panic 的用例红了（变异那一趟与基线那一趟）名字不带「 - should panic」，参数不够退 2 打用法与例子、一次 cargo 都不调；\
并行：MUTATE_JOBS=1 与 =4 的 stdout 逐字相同、没红的照样判失败、头一行报开了几个、各工作进程各用一份副本与编译目录，\
工作进程数头一项取每台默认重活线程数、取不到退 2，丢一份判定退 8 而拿掉那道闸就静默判绿，MUTATE_JOBS 写错退 2，锚点失效那一条记 ⚓、不入库而别的照跑、计数报锚点失效、整轮退 3；\
required-features：没挂的 bin 的 cargo 参数逐字不变，挂了两个的带「包名/feature」跑得起来、照常判抓到；\
平常跑带只供自证的 ${#TEST_ONLY_VARIABLES[@]} 个变量各一次都退 2，关守卫项只认单独一项、与别的项同写平常跑退 2 并点名它；\
不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY 退 2、点名它、一次 cargo 都不调；\
判定库客户端退 3：查库那一段退 2、一次 cargo 都不调，判完写库那一步照跑完、整轮退 2，都转打客户端那一句；\
取不到 git common-dir 退 2、点名那棵树、不退到别的库目录、一次 cargo 都不调；\
deny 警告：留死代码的那一条平常跑判抓到，基线、那一条与还原之后那一次的 RUSTFLAGS 逐字相同、都带 --cap-lints warn、调用方的接在前面；\
环境里设了 CARGO_ENCODED_RUSTFLAGS（空串、别的值）时 cap 接进它、0x1f 分隔，cargo 实收的与 stderr 报的一致，没设的不凭空多出一个；\
--binary-of 显式 [[bin]] 与没写的两种各打对名字、源文件不在退 2；\
按条入库：同一条第二趟复用、一次 cargo 都不起，改了被改坏函数调到的函数之后那一条现判，原文只在注释里的那一条拒算照跑；\
依赖版本行只收闭包里真点名的外部 crate 与传递依赖（逐字对得上夹具、不点的为空、改闭包外点的包照样复用、改传递依赖现判、认 package = 改名）；\
副本根下链上装置 include_str! 读的仓内别处文件、基线编得过，认不出的写法与落到仓根之外的路径退 2 逐行点名；\
测试线程：调用方没设时每条 cargo test 收到 max(1, 线程上限 ÷ 工作进程数)、调用方设了的原样收到；\
在仓根下起：cd 进 scripts/experiment-apparatus/、参数改写成从那里起的写法（不起 cargo），既不在仓根也不在那里的照旧退 2）"
}

# 这一趟在不在临时 prune-server 底下（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service；判据只在客户端的 selftest_service_problem 一处）：
#   在就退 0、什么都不打；不在退 1，stdout 打「为什么<TAB>出路」。--selftest 那一支与 MUTATE_LIBRARY_DIRECTORY 那一道都问它
temporary_service_problem() {
  python3 - "$(dirname "$SELF_PATH")/../prune-server/prune_server_client.py" <<'PY'
import importlib.util, sys
specification = importlib.util.spec_from_file_location("prune_server_client_for_mutate_selftest", sys.argv[1])
client = importlib.util.module_from_spec(specification)
specification.loader.exec_module(client)
problem = client.selftest_service_problem()
print(f"{problem}\t{client.SELFTEST_SERVICE_REMEDY}" if problem else "")
sys.exit(1 if problem else 0)
PY
}

if [[ "${1:-}" == "--selftest" ]]; then
  # 按条入库那几例写进 prune-server：只在临时服务上跑，不是就一例都不跑，不写真服务
  if ! service_problem="$(temporary_service_problem)"; then
    echo "  ✗ mutate.sh 自检没跑：${service_problem%%$'\t'*}"
    echo "  → 怎么办：${service_problem#*$'\t'}"
    exit 2
  fi
  run_selftest
  exit 0
fi
# 平常跑：只供自证的变量有一个非空就拒，一次 cargo 都不调（自证起的子进程除外，文件头「只供自证」）
if ((!running_under_selftest)) && ! guard_off_honored "${MUTATE_BREAK:-}"; then
  test_only_given=()
  for variable in "${TEST_ONLY_VARIABLES[@]}"; do
    [[ -n "${!variable:-}" ]] && test_only_given+=("$variable")
  done
  if ((${#test_only_given[@]})); then
    echo "mutate: 平常跑时带了只给自证用的变量：${test_only_given[*]}；拒跑，一条变异都没跑" >&2
    if [[ ",${MUTATE_BREAK:-}," == *",$GUARD_OFF_ITEM,"* ]]; then
      echo "        关守卫项 $GUARD_OFF_ITEM 只在它是 MUTATE_BREAK 唯一一项时认，与别的项同写（MUTATE_BREAK=$MUTATE_BREAK）照拒；要它就单独设 MUTATE_BREAK=$GUARD_OFF_ITEM" >&2
    fi
    echo "        → 怎么办：unset ${test_only_given[*]} 再跑；要证红就跑 bash $SELF_PATH --selftest（它起的子进程才认这几个变量）" >&2
    exit 2
  fi
fi
# 库目录只由被判那棵仓定（文件头「按条入库」）：MUTATE_LIBRARY_DIRECTORY 只在临时 prune-server 底下认，不在就拒，一次 cargo 都不调；
#   弄坏开关 libraryoverridehonored 下照旧认（改前的样子）
if [[ -n "${MUTATE_LIBRARY_DIRECTORY:-}" ]] && ! is_broken libraryoverridehonored && ! service_problem="$(temporary_service_problem)"; then
  echo "mutate: 不在临时 prune-server 底下设了 MUTATE_LIBRARY_DIRECTORY（「$MUTATE_LIBRARY_DIRECTORY」；${service_problem%%$'\t'*}）：库目录只由被判那棵仓定，拒跑，一条变异都没跑" >&2
  echo "        → 怎么办：unset MUTATE_LIBRARY_DIRECTORY 再跑（判定记进被判那棵仓的 git common-dir 底下那一份，键里不带哪棵仓）；自证与样本要另一份库：${service_problem#*$'\t'}" >&2
  exit 2
fi
# 头一个参数是 --binary-of（只打 bin 名）的，拿掉之后照下面解析（文件头用法那几行）
PRINT_BINARY_ONLY=0
while (($#)); do
  case "$1" in
    --binary-of) PRINT_BINARY_ONLY=1; shift; break ;;
    -*)
      # 认不出的选项（- 起头、不是上面两个）：退 2，不当 bin 名往下走。弄坏开关 unknownoptionaccepted：不判（没有这一道检查的样子）
      if ! is_broken unknownoptionaccepted; then
        echo "mutate: 认不出的选项 $1" >&2
        echo "        → 怎么办：用法 mutate.sh <bin 名> <源文件> <变异表>，或 mutate.sh --binary-of <源文件>（bin 名不以 - 起头）" >&2
        exit 2
      fi
      break ;;
    *) break ;;
  esac
done
# 在仓根下起的（文件头「在仓根下起」）：当前目录下有 scripts/experiment-apparatus/Cargo.toml、里面是 [workspace]，就 cd 进去，
# 参数里的路径（--binary-of 的源文件；平常跑的 <源文件> 与 <变异表>）从仓根起的写法改写成从 scripts/experiment-apparatus/ 起；bin 名不动。
# 弄坏开关 rootnotentered：不认仓根（改前的样子，仓根的 Cargo.toml 也有 [workspace]，下面的目录检查照样放过）
ROOT_INVOCATION_NOTE=""
# relative_to_apparatus <从仓根起的路径>：打从 scripts/experiment-apparatus/ 起的写法（绝对路径原样；scripts/experiment-apparatus/ 起头的剥掉这一段，
# scripts/ 下别处的写成 ../<其余>，别的接 ../../）
relative_to_apparatus() {
  local path="${1#./}"
  if [[ "$path" == /* ]]; then printf '%s\n' "$1"
  elif [[ "$path" == scripts/experiment-apparatus/* ]]; then printf '%s\n' "${path#scripts/experiment-apparatus/}"
  elif [[ "$path" == scripts/* ]]; then printf '../%s\n' "${path#scripts/}"
  else printf '../../%s\n' "$path"; fi
}
if ! is_broken rootnotentered && [[ -f scripts/experiment-apparatus/Cargo.toml ]] && grep -q '^\[workspace\]' scripts/experiment-apparatus/Cargo.toml; then
  root_arguments="$*"
  if ((PRINT_BINARY_ONLY)) && (($# >= 1)); then
    set -- "$(relative_to_apparatus "$1")" "${@:2}"
  elif ((!PRINT_BINARY_ONLY)) && (($# >= 3)); then
    set -- "$1" "$(relative_to_apparatus "$2")" "$(relative_to_apparatus "$3")" "${@:4}"
  fi
  if ! cd scripts/experiment-apparatus; then
    echo "mutate: 在仓根 $PWD 下起的，cd 进 scripts/experiment-apparatus/ 失败，一条变异都没跑" >&2
    echo "        → 怎么办：看上面 cd 的报错（权限、符号链接），修好再跑；或自己 cd 进 scripts/experiment-apparatus/、参数照那里起的写法再跑" >&2
    exit 2
  fi
  ROOT_INVOCATION_NOTE="mutate: 在仓根下起的：cd 进了 $PWD，参数「$root_arguments」改写成「$*」（文件头「在仓根下起」）"
fi
# 平常跑打在 stderr 头一行（开几个工作进程）之后；--binary-of 与退 6 的那两支在打别的话之前
print_root_invocation_note() { [[ -z "$ROOT_INVOCATION_NOTE" ]] || echo "$ROOT_INVOCATION_NOTE" >&2; }

# ⚠️ **先确认「改的文件」与「跑的二进制」对得上。**
# 不确认的话，变异会改 X 而跑 Y 的测试：一条真能被抓的破坏被报成盲区
# （2026-08-29 实测踩过：`e7_index.rs` 配上 crate 主二进制的名字，
# 三条变异全被误报成盲区，换对名字后三条全被抓）。
# 反方向更坏：Y 的测试恰好因别的原因红了，会被记成「变异被抓」。
# 清单：从源文件所在目录一层层往上找最近的 Cargo.toml（实验工作区根是 scripts/experiment-apparatus，src/bin 布局与自带清单的子 crate 同样认得）。
# find_declared_binary <源文件>：设 _manifest（那份清单）与 _declared（[[bin]] 里 path 对得上这份源文件的 name，没有是空串）；--binary-of 与下面的核对共用这一份读法
find_declared_binary() {
  local source_file="$1" manifest_directory
  _manifest="" manifest_directory="$(dirname "$source_file")"
  while [[ -z "$_manifest" ]]; do
    if [[ -f "$manifest_directory/Cargo.toml" ]]; then _manifest="$manifest_directory/Cargo.toml"
    elif [[ "$manifest_directory" == . || "$manifest_directory" == / ]]; then _manifest="Cargo.toml"
    else manifest_directory="$(dirname "$manifest_directory")"; fi
  done
  _declared="$(awk -v src="$source_file" '
    /^\[\[bin\]\]/ { name=""; path=""; next }
    /^name *=/ { gsub(/.*= *"|"/,""); name=$0; next }
    /^path *=/ { gsub(/.*= *"|"/,""); path=$0;
                 if (src ~ path"$") print name; next }
  ' "$_manifest" 2>/dev/null | head -1)"
}
if ((PRINT_BINARY_ONLY)); then
  if (($# != 1)) || [[ ! -f "$1" ]]; then
    echo "mutate: --binary-of 要一个参数、是在当前目录下找得到的源文件，给了 $# 个（${1:-无}）" >&2
    echo "        → 怎么办：在 scripts/experiment-apparatus/ 目录里照这个例子跑：bash ../mutation/mutate.sh --binary-of e172_deletion_write_dry_run.rs" >&2
    exit 2
  fi
  print_root_invocation_note
  find_declared_binary "$1"
  echo "${_declared:-$(basename "$1" .rs)}"   # 没写 [[bin]] 的按 cargo 自动发现的规则：文件名去掉 .rs
  exit 0
fi

# 参数不够三个：打用法与一条例子，退 2（文件头「退出码」：2 是用法错），不让 set -u 报 unbound variable
if (($# < 3)) && ! is_broken bareusage; then
  echo "mutate: 要三个参数，给了 $# 个；用法：mutate.sh <bin 名> <源文件> <变异表>" >&2
  echo "        <bin 名> 是 Cargo.toml 里 [[bin]] 的 name（没写 [[bin]] 的是源文件名去掉 .rs，可以用 mutate.sh --binary-of <源文件> 查）；<源文件> 与 <变异表> 都相对当前目录（scripts/experiment-apparatus/）" >&2
  echo "        → 怎么办：在 scripts/experiment-apparatus/ 目录里照这个例子跑：bash ../mutation/mutate.sh e172-deletion-write-dry-run e172_deletion_write_dry_run.rs ../mutation/tables/e172_deletion_write_dry_run.tsv" >&2
  exit 2
fi
BIN="$1"; SRC="$2"; TABLE="$3"

find_declared_binary "$SRC"
if [[ -n "$_declared" ]]; then
  if [[ "$_declared" != "$BIN" ]]; then
    print_root_invocation_note
    echo "mutate: 源文件 $SRC 在 Cargo.toml 里声明的二进制是 '$_declared'，不是 '$BIN'" >&2
    echo "        改的文件与跑的测试对不上，整份证明作废。用： mutate.sh $_declared $SRC $TABLE" >&2
    exit 6
  fi
else
  # 没有显式 [[bin]] ⇒ cargo 自动发现，名字就是文件名去掉扩展名
  _stem="$(basename "$SRC" .rs)"
  if [[ "$_stem" != "$BIN" ]]; then
    print_root_invocation_note
    echo "mutate: $SRC 没有显式 [[bin]]，自动发现的名字是 '$_stem'，不是 '$BIN'" >&2
    echo "        改的文件与跑的测试对不上，整份证明作废。用： mutate.sh $_stem $SRC $TABLE" >&2
    exit 6
  fi
fi

# bin 挂了 required-features 的，cargo test 要带上那几个 feature（文件头 required-features 那一段）。
# 从 $_manifest 里名字是 $BIN 的那一节 [[bin]] 现读，写成「包名/feature」逗号连起来打到 stdout；没挂、或清单里没有这一节，打空行。
# 写成带包名的形式：scripts/experiment-apparatus/ 是 workspace 根，在它的根上不带包名的 --features 要靠被选中的包碰巧有那个 feature
read_required_features() {
  MANIFEST="$_manifest" BIN="$BIN" python3 - <<'PY'
import os, sys, tomllib
manifest_path = os.environ["MANIFEST"]; binary_name = os.environ["BIN"]
if not os.path.isfile(manifest_path):
    print(""); sys.exit(0)
try:
    with open(manifest_path, "rb") as handle:
        manifest = tomllib.load(handle)
except tomllib.TOMLDecodeError as error:
    sys.stderr.write("读不懂 %s：%s\n" % (manifest_path, error)); sys.exit(1)
features = []
for target in manifest.get("bin", []):
    if target.get("name") == binary_name:
        features = target.get("required-features", [])
        break
if features:
    package_name = manifest.get("package", {}).get("name")
    if not package_name:
        sys.stderr.write("%s 里 %s 挂了 required-features，清单却没有 [package] name\n" % (manifest_path, binary_name)); sys.exit(1)
    print(",".join("%s/%s" % (package_name, feature) for feature in features))
else:
    print("")
PY
}
cargo_feature_arguments=()
if ! is_broken nofeatures; then
  if ! required_features="$(read_required_features)"; then
    echo "mutate: 读不出 $_manifest 里 $BIN 的 required-features（上面是原因），一条变异都没跑" >&2
    echo "        → 怎么办：先让 cargo metadata --manifest-path $_manifest 读得通这份清单，再整张表重跑" >&2
    exit 2
  fi
  if [[ -n "$required_features" ]]; then
    cargo_feature_arguments=(--features "$required_features")
  fi
fi
# 这个 bin 编不编到 librocksdb-sys（文件头「RocksDB 预编库的取值」）：$_manifest 里 rocksdb 是不挂特性的依赖，或 $BIN 的 required-features 顺着 [features]
# 一层层点到 dep:rocksdb、rocksdb、rocksdb/<特性> 或 rocksdb?/<特性>，退 0；别的退 1（清单读不了也是 1：上面 read_required_features 已经判过读不读得了）
bin_builds_rocksdb() {
  MANIFEST="$_manifest" BIN="$BIN" python3 - <<'PY'
import os, sys, tomllib
try:
    with open(os.environ["MANIFEST"], "rb") as handle:
        manifest = tomllib.load(handle)
except (OSError, tomllib.TOMLDecodeError):
    sys.exit(1)
dependency = manifest.get("dependencies", {}).get("rocksdb")
if dependency is not None and not (isinstance(dependency, dict) and dependency.get("optional")):
    sys.exit(0)
features = manifest.get("features", {})
pending = list(next((target.get("required-features", []) for target in manifest.get("bin", []) if target.get("name") == os.environ["BIN"]), []))
seen = set()
while pending:
    feature = pending.pop()
    if feature in ("dep:rocksdb", "rocksdb") or feature.startswith(("rocksdb/", "rocksdb?/")):
        sys.exit(0)
    if feature not in seen:
        seen.add(feature)
        pending.extend(features.get(feature, []))
sys.exit(1)
PY
}
# 编得到就在起任何 cargo 之前判 RocksDB 预编库的取值（文件头「RocksDB 预编库的取值」），判红退 2、一条变异都不跑；弄坏开关 rocksdb-entry-unchecked 下不判
if ! is_broken rocksdb-entry-unchecked && bin_builds_rocksdb; then
  if ! python3 "$(dirname "$SELF_PATH")/../hosts/rocksdb-prebuilt.py" entry --root "$(cd ../.. && pwd)" >&2; then
    echo "mutate: $BIN 编得到 librocksdb-sys，起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条变异都没跑" >&2
    echo "        → 怎么办：照上面的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再整张表重跑" >&2
    exit 2
  fi
fi

# ⚠️ **变异只改副本，不碰工作区里的源文件**（2026-09-12 改）。此前是就地改 $SRC 再还原：一轮变异要几分钟，
# 这几分钟里仓里那份源码是坏的——几个会话共写一个仓时，别的会话此刻跑门禁（research-build-and-replay 阶段会编译并跑实验装置的全部单测）
# 就编到被改坏的源码，红得莫名其妙、还可能当成自己改坏的；此刻有人整份提交这个文件，提交进去的就是变异。
# ⇒ 把 scripts/experiment-apparatus/ 整个工作区（不带 target）拷到临时目录，include_str! 读的 research/results/ 与别的仓内文件（.claude/heavy-thread-default 这类）
#   在副本根下照层级链回仓里那一份（认不出的写法退 2，见拷副本那一段），
#   在那里改、在那里编；CARGO_TARGET_DIR 单独一份，不碰 scripts/experiment-apparatus/target 里别人要用的二进制。跑完删掉副本。
#   并行时每个工作进程各一份这样的副本、各一个编译目录（文件头「并行」那一段）。
if [[ ! -f Cargo.toml ]] || ! grep -q '^\[workspace\]' Cargo.toml; then
  echo "mutate: 要在 scripts/experiment-apparatus/ 下跑（那里才有 workspace 的 Cargo.toml）" >&2; exit 2
fi
REQUESTED_JOBS="${MUTATE_JOBS:-}"
if [[ -n "$REQUESTED_JOBS" && ! "$REQUESTED_JOBS" =~ ^[1-9][0-9]*$ ]]; then
  echo "mutate: MUTATE_JOBS 写成了「$REQUESTED_JOBS」，要写正整数（它只能把工作进程数压小），一条变异都没跑" >&2
  echo "        → 怎么办：写成 1、2、4 这样的正整数再跑；不设就按每台默认重活线程数与表里的条数取小" >&2
  exit 2
fi
REQUESTED_CARGO_JOBS="${MUTATE_CARGO_JOBS:-}"
if [[ -n "$REQUESTED_CARGO_JOBS" && ! "$REQUESTED_CARGO_JOBS" =~ ^[1-9][0-9]*$ ]]; then
  echo "mutate: MUTATE_CARGO_JOBS 写成了「$REQUESTED_CARGO_JOBS」，要写正整数，一条变异都没跑" >&2
  echo "        → 怎么办：写成正整数再跑；不设就用调用方的 CARGO_BUILD_JOBS，那也没设就取 核数 ÷ 工作进程数" >&2
  exit 2
fi

# 读表：一行三段、制表符分隔，空行与 # 开头的行跳过（与改成并行之前逐行读的口径相同），按表序进三个数组
mutation_names=(); mutation_originals=(); mutation_replacements=()
while IFS=$'\t' read -r name from to; do
  [[ -z "${name:-}" || "${name:0:1}" == "#" ]] && continue
  mutation_names+=("$name"); mutation_originals+=("$from"); mutation_replacements+=("$to")
done < "$TABLE"
mutation_count=${#mutation_names[@]}

# 开几个工作进程：各项取最小、至少 1，连同各项给几个、卡在哪一项、每个 cargo 的编译并行度一起打在 stderr 的头一行（文件头「并行」那一段）
# 每台默认重活线程数（文件头「并行」）：工作进程数的头一项与每条 cargo test 的测试线程数都取它；取不到退 2、一条变异都没跑，不退回按核数算
source "$(dirname "$SELF_PATH")/../hosts/heavy-threads.sh"
HOST_HEAVY_THREADS=""
if ! is_broken workersbyprocessor && ! HOST_HEAVY_THREADS="$(heavy_thread_default)"; then
  echo "mutate: 取不到每台默认重活线程数（上面是原因），定不了开几个工作进程，一条变异都没跑" >&2
  echo "        → 怎么办：照上面的出路修 .claude/heavy-thread-default 再跑；不退回按核数算" >&2
  exit 2
fi
worker_count_and_head_line() {
  local processor_count by_processor smallest binding="" floor_note="" bound_index
  local -a bound_names=() bound_counts=() bound_texts=()
  processor_count="$(nproc 2>/dev/null || echo 2)"
  if is_broken workersbyprocessor; then   # 弄坏开关（文件头）：照改前取核数一半、至多 16
    by_processor=$((processor_count / 2))
    ((by_processor > MAXIMUM_WORKERS_BY_PROCESSOR)) && by_processor=$MAXIMUM_WORKERS_BY_PROCESSOR
    ((by_processor < 1)) && by_processor=1
    bound_names+=("核数"); bound_counts+=("$by_processor"); bound_texts+=("核数 $processor_count 的一半、至多 $MAXIMUM_WORKERS_BY_PROCESSOR，给 $by_processor 个")
  else
    bound_names+=("每台默认重活线程数"); bound_counts+=("$HOST_HEAVY_THREADS"); bound_texts+=("每台默认重活线程数 $HOST_HEAVY_THREADS（SINGLEFS_THREAD_CAP 或 .claude/heavy-thread-default）")
  fi
  if [[ -n "$REQUESTED_JOBS" ]]; then
    bound_names+=("MUTATE_JOBS"); bound_counts+=("$REQUESTED_JOBS"); bound_texts+=("MUTATE_JOBS 给 $REQUESTED_JOBS 个")
  fi
  bound_names+=("表里的条数"); bound_counts+=("$mutation_count"); bound_texts+=("表里 $mutation_count 条")
  smallest="${bound_counts[0]}"
  for bound_index in "${!bound_counts[@]}"; do
    ((bound_counts[bound_index] < smallest)) && smallest="${bound_counts[bound_index]}"
  done
  for bound_index in "${!bound_counts[@]}"; do
    ((bound_counts[bound_index] == smallest)) && binding+="${binding:+、}「${bound_names[bound_index]}」"
  done
  ((smallest < 1)) && floor_note="（那一项不足 1，按 1 个开）"
  WORKER_COUNT=$((smallest < 1 ? 1 : smallest))
  if [[ -n "$REQUESTED_CARGO_JOBS" ]]; then
    CARGO_JOBS_PER_WORKER="$REQUESTED_CARGO_JOBS"; CARGO_JOBS_SOURCE="MUTATE_CARGO_JOBS"
  elif [[ -n "${CARGO_BUILD_JOBS:-}" ]]; then
    CARGO_JOBS_PER_WORKER="$CARGO_BUILD_JOBS"; CARGO_JOBS_SOURCE="调用方的 CARGO_BUILD_JOBS"
  else
    CARGO_JOBS_PER_WORKER=$((processor_count / WORKER_COUNT < 1 ? 1 : processor_count / WORKER_COUNT)); CARGO_JOBS_SOURCE="核数 ÷ 工作进程数"
  fi
  local joined_texts
  joined_texts="$(printf '%s；' "${bound_texts[@]}")"
  echo "mutate: 这一轮开 $WORKER_COUNT 个工作进程（各项取最小、至少 1）：${joined_texts%；}。卡在${binding}这一项${floor_note}；每个 cargo 编译并行度 $CARGO_JOBS_PER_WORKER（${CARGO_JOBS_SOURCE}）" >&2
}
worker_count_and_head_line
# 开几个工作进程那一行仍是 stderr 的头一行；挂了 required-features 的 bin 跟在它后面报带了哪几个
if ((${#cargo_feature_arguments[@]})); then
  echo "mutate: $BIN 在 $_manifest 里挂了 required-features，每次 cargo test 带 ${cargo_feature_arguments[*]}" >&2
fi
print_root_invocation_note
# 每条 cargo test 的测试线程数（文件头「并行」）：调用方设了 RUST_TEST_THREADS（非空）原样传、不覆盖；没设的取 max(1, 线程上限 ÷ 工作进程数)，
# 线程上限是工作进程数那一项的同一个登记值（HOST_HEAVY_THREADS；弄坏开关 workersbyprocessor 下现取，取不到退 2）。
# 只设在 run_cargo_test 起的那几次 cargo test 的环境里（TEST_THREADS_ENVIRONMENT），不 export
TEST_THREADS_ENVIRONMENT=()
if ! is_broken testthreadsunset; then   # 弄坏开关 testthreadsunset：不设（改前的跑法）
  if [[ -n "${RUST_TEST_THREADS:-}" ]] && ! is_broken overridecallertestthreads; then   # 弄坏开关 overridecallertestthreads：调用方设了的也换成算出来的
    echo "mutate: 每条 cargo test 照调用方设的 RUST_TEST_THREADS=$RUST_TEST_THREADS 跑（不覆盖）" >&2
  else
    thread_cap_of_host="$HOST_HEAVY_THREADS"
    if [[ -z "$thread_cap_of_host" ]] && ! thread_cap_of_host="$(heavy_thread_default)"; then
      echo "mutate: 取不到每台默认重活线程数（上面是原因），定不了每条 cargo test 的 RUST_TEST_THREADS，一条变异都没跑" >&2
      echo "        → 怎么办：照上面的出路修 .claude/heavy-thread-default 再跑；或调用方自己设 RUST_TEST_THREADS（原样传、不覆盖）" >&2
      exit 2
    fi
    test_threads_per_worker=$((thread_cap_of_host / WORKER_COUNT < 1 ? 1 : thread_cap_of_host / WORKER_COUNT))
    TEST_THREADS_ENVIRONMENT=("RUST_TEST_THREADS=$test_threads_per_worker")
    echo "mutate: 每条 cargo test 设 RUST_TEST_THREADS=$test_threads_per_worker（线程上限 $thread_cap_of_host ÷ 工作进程 $WORKER_COUNT 个，至少 1）" >&2
  fi
fi
export CARGO_BUILD_JOBS="$CARGO_JOBS_PER_WORKER"

SHARD_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-mutate-XXXXXX")"
PRISTINE="$SHARD_ROOT/pristine-source"   # 开跑时那一份被测源码，每条变异都从它改、每次还原都拷它
CLAIMS="$SHARD_ROOT/claims"              # <行号>/：那一条被哪个工作进程认领了（mkdir 建成的那个领走）
RESULTS="$SHARD_ROOT/results"            # <行号>.line（要打到 stdout 的那一行）、.stderr、.verdict（「档 要不要判失败」，最后写）
mkdir -p "$CLAIMS" "$RESULTS"
worker_pids=()
# 收尾时停一个自己起的工作进程：一次只处理点名的这一个，而且只在它的父进程还是这一趟时才发 TERM（进程号被别的进程复用了不误伤），再等它退
stop_own_worker() { # stop_own_worker <进程号>
  local stat_line parent
  IFS= read -r stat_line 2>/dev/null <"/proc/$1/stat" || return 0
  read -r _ parent _ <<<"${stat_line##*) }"
  [[ "$parent" == "$$" ]] || return 0
  kill -TERM "$1" 2>/dev/null
  wait "$1" 2>/dev/null
  return 0
}
cleanup() {
  local pid cleanup_exit=$?
  for pid in "${worker_pids[@]}"; do stop_own_worker "$pid"; done
  # 这一趟自己的编译目录在分片根里（文件头「并行」）：删之前核没有进程开着里面的文件，有就连同分片根留着、报出进程
  if is_broken runtargetkept && [[ -z "${MUTATE_TARGET_DIR:-}" ]]; then return 0; fi   # 弄坏开关（文件头）：没设 MUTATE_TARGET_DIR 时分片根与编译目录都不删
  if [[ -z "${MUTATE_TARGET_DIR:-}" ]] && ! python3 "$(dirname "$SELF_PATH")/../hosts/run_build_directory.py" holders "$SHARD_ROOT" >"$SHARD_ROOT.holders" 2>&1; then
    echo "  ✗ 这一趟的分片根 $SHARD_ROOT（连同里面的编译目录）还有进程开着里面的文件，没删：" >&2
    sed 's/^/      /' "$SHARD_ROOT.holders" >&2
    echo "  → 怎么办：照上面的进程号逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>，再 rm -rf -- '$SHARD_ROOT'" >&2
    rm -f -- "$SHARD_ROOT.holders"
    exit $(( cleanup_exit ? cleanup_exit : 1 ))
  fi
  rm -f -- "$SHARD_ROOT.holders"
  rm -rf "${SHARD_ROOT:?}"
}
trap cleanup EXIT
MUTATE_TARGET="${MUTATE_TARGET_DIR:-$SHARD_ROOT/target}"
# 副本照仓里的层级放（copy-<i>/scripts/experiment-apparatus/）：装置用 include_str!("../../research/results/…") 读留存产物，
# 副本根下放一个符号链接指回仓里那一份（只读，变异不改它）
copy_root_of() { echo "$SHARD_ROOT/copy-$1"; }
copy_directory_of() { echo "$SHARD_ROOT/copy-$1/scripts/experiment-apparatus"; }
REPOSITORY_OF_WORKSPACE="$(cd ../.. && pwd)"
target_directory_of() { if (($1 == 1)); then echo "$MUTATE_TARGET"; else echo "$MUTATE_TARGET-w$1"; fi; }

if ! mkdir -p "$(copy_directory_of 1)" "$(copy_root_of 1)/research" \
   || ! ln -s "$REPOSITORY_OF_WORKSPACE/research/results" "$(copy_root_of 1)/research/results" \
   || ! rsync -a --exclude target ./ "$(copy_directory_of 1)/"; then
  echo "mutate: 拷副本失败" >&2; exit 2
fi
# 副本范围（文件头「变异只改副本」一段）：拷贝范围不在这里写，取 scripts/hosts/lib_repository_copy.py 的闭包（只拷这一个工作区的那一种：
# --workspace scripts/experiment-apparatus，连测试代码一起扫——变异要跑测试），闭包里副本根下还没有的仓根条目（编译期 include_str! / include_bytes! / #[path]
# 读的、运行期读的、它们 source 与起的、path 依赖指到的）逐个放一个符号链接指回仓里那一份（只读，变异不改它；research/results 上面已经链了，经 --present 点名）。
# 闭包算不成的（认不出的 include 写法、绝对路径、落到仓根之外的、仓里没有那份编译期读的文件、path 依赖指到仓根之外）退 2 逐处点名，一条变异都不跑。
# 弄坏开关 includesunlinked：不链（改前的样子，副本里读不到就编不过）
REPOSITORY_COPY_LIBRARY="$(dirname "$SELF_PATH")/../hosts/lib_repository_copy.py"
if ! is_broken includesunlinked && ! python3 "$REPOSITORY_COPY_LIBRARY" link-into "$(copy_root_of 1)" --root "$REPOSITORY_OF_WORKSPACE" \
    --workspace scripts/experiment-apparatus --present research/results --test-inputs; then
  echo "        → 怎么办：照上面逐处改：编译期读的写成 include_str!(\"<相对这份源文件、落在仓里的路径>\") 或 concat!(env!(\"CARGO_MANIFEST_DIR\"), \"…\")，或先让那份文件进仓；一条变异都没跑" >&2
  exit 2
fi
# 副本里起的 cargo 与测试再起的判法用的多机配置显式交过去（副本不是 git 工作树，判法照仓位置找不到它）：调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它，
# 没设取 lib_repository_copy.py configuration-path 打的主工作区那一份
if [[ -z "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then
  if ! MUTATE_COPY_CONFIGURATION="$(python3 "$REPOSITORY_COPY_LIBRARY" configuration-path "$REPOSITORY_OF_WORKSPACE")" || [[ -z "$MUTATE_COPY_CONFIGURATION" ]]; then
    echo "mutate: 取不到副本里起命令要用的多机配置路径（lib_repository_copy.py configuration-path 没打出来），一条变异都没跑" >&2
    echo "        → 怎么办：设 SINGLEFS_MULTI_HOST_CONFIG 指到主工作区那一份 multi-host.env 再跑，或先修 lib_repository_copy.py" >&2
    exit 2
  fi
  SINGLEFS_MULTI_HOST_CONFIG="$MUTATE_COPY_CONFIGURATION"
  export SINGLEFS_MULTI_HOST_CONFIG
fi
ORIGINAL_SUM="$(sha256sum "$SRC" | cut -d' ' -f1)"
cp "$(copy_directory_of 1)/$SRC" "$PRISTINE"

# 从开跑时那一份源码施加一条变异：$1 原文、$2 替换文、$3 写到哪（空串只核命中、不写）。原文要恰好命中一次，否则退 1 并把命中数打到 stderr
write_mutated_source() {
  FROM="$1" TO="$2" PRISTINE="$PRISTINE" DESTINATION="$3" python3 - <<'PY'
import os,sys
s=open(os.environ["PRISTINE"]).read()
f=os.environ["FROM"].replace("\\n","\n"); t=os.environ["TO"].replace("\\n","\n")
n=s.count(f)
if n!=1:
    sys.stderr.write("命中 %d 次（要求恰好 1 次）：%r\n"%(n,f)); sys.exit(1)
if os.environ["DESTINATION"]:
    open(os.environ["DESTINATION"],"w").write(s.replace(f,t))
PY
}
# 锚点在派活之前逐条核（文件头「锚点失效」）：不是恰好命中一次的那一条记「⚓ 锚点失效」，判定先写进结果文件、认领目录先建好，
# 工作进程不领它、判定库不记它；别的条照跑，收尾计数单列、整轮退 3（.claude/rules/mutation-sampling.md 第七类）
ANCHOR_BROKEN_INDEXES="$SHARD_ROOT/anchor-broken.indexes"
: >"$ANCHOR_BROKEN_INDEXES"
ANCHOR_BROKEN_COUNT=0
for ((row_index = 0; row_index < mutation_count; row_index++)); do
  if ! anchor_hits="$(write_mutated_source "${mutation_originals[row_index]}" "${mutation_replacements[row_index]}" "" 2>&1)"; then
    if is_broken anchorstopsall; then   # 弄坏开关：改前的跑法，一条锚点失效整轮退 3、一条变异都不跑
      echo "mutate: [${mutation_names[row_index]}] 替换没命中，证明作废（锚点在派活之前逐条核，这一轮一条变异都没跑）" >&2
      echo "        → 怎么办：把变异表里这一条的原文改到源码今天的写法、让它恰好命中一次，再整张表重跑" >&2
      exit 3
    fi
    printf '%s\n' "⚓ [${mutation_names[row_index]}] 锚点失效：原文在 $SRC 里${anchor_hits%%：*}—— 这一条没跑，不算抓到也不算没红" >"$RESULTS/$row_index.line"
    printf 'anchor 1\n' >"$RESULTS/$row_index.verdict"
    mkdir -p "$CLAIMS/$row_index"
    echo "$row_index" >>"$ANCHOR_BROKEN_INDEXES"
    ANCHOR_BROKEN_COUNT=$((ANCHOR_BROKEN_COUNT + 1))
  fi
done
# 收尾那一句：锚点失效的条数不为 0 整轮退 3（判完别的条之后；4、5、7、8 那几种照它们自己的码先退）
report_anchor_broken() {
  ((ANCHOR_BROKEN_COUNT)) || return 0
  echo "mutate: $ANCHOR_BROKEN_COUNT 条锚点失效（上面 ⚓ 那几行），这几条没跑、不入判定库，整轮判失败（退 3）；别的条照跑完了" >&2
  echo "        → 怎么办：把变异表里那几条的原文改到源码今天的写法、让它恰好命中一次（原文里的换行写成 \\n），再跑这张表" >&2
}

# 第 2 份起的副本从第 1 份拷：同一份源码快照，不在拷的间隙里读到别人刚改的工作区
for ((worker_index = 2; worker_index <= WORKER_COUNT; worker_index++)); do
  if ! rsync -a "$(copy_root_of 1)/" "$(copy_root_of "$worker_index")/"; then
    echo "mutate: 拷第 $worker_index 份副本失败" >&2; exit 2
  fi
done
# ⚠️ rsync -a 保留源码的旧 mtime，而编译目录跨轮共用：上一轮在一条变异写进副本之后中途退出时
# 那里留着一份**打着变异**编出来的二进制，它比副本里的源码新 ⇒ cargo 判「不用重编」，基线直接跑变异版。
# 实测（2026-09-12）：e114 在 M6 处退出后，下一轮基线连报两次「基线就是红的」，而单测直接跑 14 条全绿。
# 先 touch 每份副本里的被测源码，逼 cargo 按这一轮的副本重编（每个工作进程的编译目录都跨轮留着，每份都要 touch）。
for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  touch "$(copy_directory_of "$worker_index")/$SRC"
done

# 每个 cargo 一份同样的 RUSTFLAGS（文件头「deny 警告」那一段）：调用方设了的在前，后面接 --cap-lints warn；
# 开跑前算一次，基线、每条变异、还原之后那一次都经 RUSTFLAGS_ENVIRONMENT 带它
MUTATE_RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }--cap-lints warn"
RUSTFLAGS_ENVIRONMENT=("RUSTFLAGS=$MUTATE_RUSTFLAGS")
MUTATE_FLAGS_HINT="RUSTFLAGS='$MUTATE_RUSTFLAGS'"   # 出路里让人照抄的那一个变量赋值
MUTATE_FLAGS_REPORT="RUSTFLAGS=「$MUTATE_RUSTFLAGS」（调用方设了的在前；--cap-lints warn 把装置的 deny 警告压成警告，替换文留下的死代码不算编不过）"
# 环境里设了 CARGO_ENCODED_RUSTFLAGS（空串也算）时 cargo 只读它、不读 RUSTFLAGS：cap 也接进它（各项以 0x1f 分隔，调用方设了的在前），RUSTFLAGS 照旧也设
MUTATE_ENCODED_RUSTFLAGS=""
if [[ -v CARGO_ENCODED_RUSTFLAGS ]]; then
  MUTATE_UNIT_SEPARATOR=$'\x1f'
  MUTATE_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS:+$CARGO_ENCODED_RUSTFLAGS$MUTATE_UNIT_SEPARATOR}--cap-lints${MUTATE_UNIT_SEPARATOR}warn"
  RUSTFLAGS_ENVIRONMENT+=("CARGO_ENCODED_RUSTFLAGS=$MUTATE_ENCODED_RUSTFLAGS")
  MUTATE_FLAGS_HINT="CARGO_ENCODED_RUSTFLAGS=$(printf '%q' "$MUTATE_ENCODED_RUSTFLAGS")"
  MUTATE_FLAGS_REPORT="CARGO_ENCODED_RUSTFLAGS=「${MUTATE_ENCODED_RUSTFLAGS//$MUTATE_UNIT_SEPARATOR/␟}」（环境里设了它，cargo 只读它、不读 RUSTFLAGS，所以 cap 接在它里面，RUSTFLAGS 也同样带着；各项以 0x1f 分隔、这里显示成 ␟；调用方设了的在前；--cap-lints warn 把装置的 deny 警告压成警告，替换文留下的死代码不算编不过）"
fi
if is_broken no-cap-lints; then   # 弄坏开关：改前的跑法，RUSTFLAGS 与 CARGO_ENCODED_RUSTFLAGS 都照调用方的原样传（没设就不设）
  MUTATE_RUSTFLAGS="${RUSTFLAGS:-}"; RUSTFLAGS_ENVIRONMENT=(); MUTATE_ENCODED_RUSTFLAGS=""
  MUTATE_FLAGS_HINT="RUSTFLAGS='$MUTATE_RUSTFLAGS'"
  MUTATE_FLAGS_REPORT="RUSTFLAGS=「$MUTATE_RUSTFLAGS」（调用方设了的在前；--cap-lints warn 把装置的 deny 警告压成警告，替换文留下的死代码不算编不过）"
fi
echo "mutate: 每个 cargo 都带 $MUTATE_FLAGS_REPORT" >&2

# ── 按条入库（文件头「按条入库」）：逐条算键、查库；复用得上的那几条判定先写进结果文件、认领目录先建好，工作进程不再领它们 ──
MUTATE_LIBRARY_HELPER="$(dirname "$SELF_PATH")/lib_mutation_units.py"
PLAN_FILE="$SHARD_ROOT/library-plan.json"
LIBRARY_RECORD_ERRORS="$SHARD_ROOT/library-record.errors"
LIBRARY_UNAVAILABLE="$SHARD_ROOT/library-unavailable"   # record_in_library 收到客户端退 75（prune-server 不可用）的那几条，收尾照它整轮退 75
LIBRARY_CLIENT_ERRORS="$SHARD_ROOT/library-client-errors"   # record_in_library 收到客户端退 3（客户端别的错）的那几条，收尾照它整轮退 2
if [[ -n "${MUTATE_LIBRARY_DIRECTORY:-}" ]]; then
  # 走到这里只有两种：临时 prune-server 底下（自证、样本），或弄坏开关 libraryoverridehonored（参数解析之前那一道已经拒了别的）
  MUTATE_LIBRARY="$MUTATE_LIBRARY_DIRECTORY"
elif library_common_directory="$(git -C "$REPOSITORY_OF_WORKSPACE" rev-parse --path-format=absolute --git-common-dir 2>"$SHARD_ROOT/library-common-directory.stderr")" \
     && [[ -n "$library_common_directory" ]]; then
  # 库目录：被判那棵仓的 git common-dir 底下那一份（文件头「按条入库」），只当名字
  MUTATE_LIBRARY="$library_common_directory/singlefs-mutate-library"
elif is_broken libraryrootfallback; then
  # 弄坏开关 libraryrootfallback：改前的样子，取不到 git common-dir 也照常跑完、判定写进另一个键前缀。库名写死在这里、不经共享目录名单取：
  #   名单只登记真在用的目录。Library 只拿末段当种类、本地不建，不在临时目录底下不加命名空间，键前缀是 mutation-units/mutate-library/
  MUTATE_LIBRARY=/libraryrootfallback/mutate-library
else
  # 库目录只由被判那棵仓定（文件头「按条入库」）：取不到库根就停，不退到别的库目录
  library_common_directory_said="$(tail -n 3 "$SHARD_ROOT/library-common-directory.stderr" 2>/dev/null | tr '\n' ' ')"
  echo "mutate: 取不到 $REPOSITORY_OF_WORKSPACE 的 git common-dir（判定库的名字；git rev-parse 打的是「$library_common_directory_said」）：库目录只由被判那棵仓定，拒跑，一条变异都没跑" >&2
  echo "        → 怎么办：在 git 工作树里跑；rsync 拷的副本先在副本根 git init（键里不带哪棵仓，副本与主仓读写同一份库）" >&2
  exit 2
fi
library_cargo_words=(test --release --bin "$BIN" ${cargo_feature_arguments[@]+"${cargo_feature_arguments[@]}"})
library_breaks=""
for library_break in libraryunread calleesignored dependenciesbycrate redreused; do is_broken "$library_break" && library_breaks+="${library_breaks:+,}$library_break"; done
# 复用哨兵（文件头「复用哨兵」）：状态放这一轮自己的临时根；判法是 scripts/gate/lib_reuse_checks.py（拷进不带它的树里跑的这一份不抽）
REUSE_CHECKS="$(dirname "$SELF_PATH")/../gate/lib_reuse_checks.py"
REUSE_SENTINEL_STATE="$SHARD_ROOT/reuse-sentinel.json"
if library_summary="$(env ${RUSTFLAGS_ENVIRONMENT[@]+"${RUSTFLAGS_ENVIRONMENT[@]}"} MUTATE_ANCHOR_BROKEN_INDEXES="$ANCHOR_BROKEN_INDEXES" \
      MUTATE_REUSE_CHECKS="$REUSE_CHECKS" MUTATE_REUSE_SENTINEL_STATE="$REUSE_SENTINEL_STATE" \
      python3 - "$MUTATE_LIBRARY_HELPER" "$PWD" "$SRC" "$BIN" "$TABLE" \
      "$MUTATE_LIBRARY" "$PLAN_FILE" "$RESULTS" "$CLAIMS" "$library_breaks" "${library_cargo_words[@]}" <<'PY'
import importlib.util, json, os, sys
helper, root, source, binary, table, library, plan_path, results, claims, breaks = sys.argv[1:11]
words = sys.argv[11:]
specification = importlib.util.spec_from_file_location("lib_mutation_units_for_mutate", helper)
units = importlib.util.module_from_spec(specification)
sys.modules[specification.name] = units
specification.loader.exec_module(units)
try:
    decisions = units.mutate_plan(root, source, binary, table, words, dict(os.environ), library, plan_path,
                                  tuple(item for item in breaks.split(",") if item))
except Exception as error:   # 算不出键（工具链起不来、切不开、Cargo.lock 读不懂这一类）：这一趟每条现判、不写库，不让变异跑不起来
    print(f"mutate: 判定库：算不出键（{type(error).__name__}：{error}），这一趟每条现判、不写库", file=sys.stderr)
    print("0 0 0")
    sys.exit(0)
reused = fresh = refused = 0
# 锚点失效的那几条（派活之前那一段记好了「⚓ 锚点失效」）：不复用、不现判、不拒算，判定库不碰
with open(os.environ["MUTATE_ANCHOR_BROKEN_INDEXES"], encoding="utf-8") as handle:
    anchor_broken = {int(line) for line in handle if line.strip()}


def sentinel_candidates():
    """复用哨兵的候选（文件头「复用哨兵」）：此刻键算得出、库里 records/<键> 记着可复用的档、复用要的另几样也对得上的那几条 → {变异名: 档}。
    与复用判法（lib_mutation_units.py 的 mutate_decision）分开查：直接读计划文件与记录文件的原文、自己解，不经 mutate_decision；
    原文经 Library.read_text 取（prune-server 里那一份；服务不可用客户端退 75）。"""
    with open(plan_path, encoding="utf-8") as handle:
        plan = json.load(handle)
    reusable = ("caught", "notred", "invalid")
    found = {}
    stored = units.Library(library)
    for index, entry in enumerate(plan["rows"]):
        if index in anchor_broken or not entry.get("key"):
            continue
        text = stored.read_text("records", entry["key"])
        if text is None:
            continue
        record = dict(line.split("=", 1) for line in text.splitlines() if "=" in line and not line.startswith("#"))
        verdict = record.get("verdict", "")
        if verdict not in reusable or record.get("key") != entry["key"]:
            continue
        if record.get("fails", "0") != "0" and "redreused" not in breaks.split(","):
            continue   # 判红的（fails=1：没红、编不过）不当绿复用，也不进候选（文件头「按条入库」记红、撤绿、下趟现判）
        if verdict == "caught":
            red_keys = dict(item.split("=", 1) for item in record.get("red_test_keys", "").split(",") if "=" in item)
            if not red_keys or any(plan["tests"].get(test) != key for test, key in red_keys.items()):
                continue
        elif not plan.get("all_tests") or plan["all_tests"] != record.get("all_tests"):
            continue
        found[entry["name"]] = verdict
    return found


sentinel = None
state_path, checks_path = os.environ["MUTATE_REUSE_SENTINEL_STATE"], os.environ["MUTATE_REUSE_CHECKS"]
if not os.path.isfile(checks_path):
    print(f"mutate: 复用哨兵：找不到 {checks_path}（这一份 mutate.sh 跑在不带它的树里），这一趟不抽哨兵", file=sys.stderr)
else:
    checks_specification = importlib.util.spec_from_file_location("lib_reuse_checks_for_mutate", checks_path)
    checks = importlib.util.module_from_spec(checks_specification)
    checks_specification.loader.exec_module(checks)
    sentinel = checks.ReuseSentinel("scripts/mutation/mutate.sh", f"变异表 {os.path.basename(table)}")
    sentinel.plan(sentinel_candidates(), printer=lambda text: print(f"mutate: {text.strip()}", file=sys.stderr))
forced_index = None
for index, name, reuse, verdict, fails, line, reason in decisions:
    if int(index) in anchor_broken:
        continue
    if reuse and str(fails) != "0" and "redreused" not in breaks.split(","):
        # 记红、撤绿、下趟现判（文件头「按条入库」）：库里键相同的判定是红（判失败的那几档：没红、编不过），不当绿复用，这一条现判；判绿了就盖回绿
        reuse, reason = False, f"上次判红（{verdict}）：键相同的判定是红，现判"
    if sentinel is not None and sentinel.forced(name):
        # 抽中强制现判的那一条：不用库里的判定，交给工作进程照现判跑（跑完照键写回同一份库）；收尾拿它这一趟的档与库里的比
        forced_index = index
        if reuse:
            fresh += 1
            print(f"mutate: 现判：[{name}] 强制现判（复用哨兵抽中，种子 {sentinel.seed}；库里键相同的判定是「{verdict}」，这一趟不读库）", file=sys.stderr)
            continue
    if sentinel is not None and name in sentinel.sentinels:
        sentinel.observe(name, "复用" if reuse else "现判", verdict if reuse else "-")
    if reuse:
        reused += 1
        if line:
            with open(os.path.join(results, f"{index}.line"), "w", encoding="utf-8") as handle:
                handle.write(line + "\n")
        with open(os.path.join(results, f"{index}.verdict"), "w", encoding="utf-8") as handle:
            handle.write(f"{verdict} {fails}\n")
        os.makedirs(os.path.join(claims, str(index)), exist_ok=True)
    elif reason.startswith("拒算"):
        refused += 1
        print(f"mutate: 拒算（照跑、不复用）：[{name}] {reason[len('拒算：'):]}", file=sys.stderr)
    else:
        fresh += 1
        print(f"mutate: 现判：[{name}] {reason}", file=sys.stderr)
if sentinel is not None:
    # 状态与哨兵的结局交给收尾（finish_reuse_sentinel）：强制现判那一条判完才比；lib_reuse_checks.py sentinel-observe / sentinel-finish 读的就是这两份
    with open(state_path, "w", encoding="utf-8") as handle:
        json.dump(sentinel.to_json(), handle, ensure_ascii=False)
    with open(state_path + ".observed", "w", encoding="utf-8") as handle:
        handle.write("".join(f"{item}\t{state}\t{verdict}\n" for item, (state, verdict) in sentinel.observed.items()))
    if forced_index is not None:
        with open(state_path + ".forced", "w", encoding="utf-8") as handle:
            handle.write(f"{forced_index}\t{sentinel.forced_item}\n")
print(reused, fresh, refused)
PY
)"; then
  read -r LIBRARY_REUSED LIBRARY_FRESH LIBRARY_REFUSED <<<"$library_summary"
else
  library_status=$?
  # 判定库在 prune-server 里而服务不可用（scripts/prune-server/prune_server_client.py 文件头「退出码」）：读失败不当「库里没有」，
  # 不退回每条现判（会把整张表重跑一遍、判出的也写不回去），照客户端的码退
  if ((library_status == 75)); then
    echo "  ✗ mutate: prune-server 不可用（退 $library_status，上面是原因）：判定库读不了，不每条现判，这一趟停在这里" >&2
    echo "     → 怎么办：python3 prune-server/control check 看服务在不在、第二台连的地址通不通；服务回来之后重跑同一条命令" >&2
    exit "$library_status"
  fi
  # 客户端退 3（客户端别的错：没配服务地址、时限写坏、帧被拒这类，客户端文件头「退出码」）：同样读不了库，不退回每条现判，退 2（文件头「退出码」）。
  #   弄坏开关 planclienterrorfallback 下照改前退回每条现判
  if ((library_status == 3)) && ! is_broken planclienterrorfallback; then
    echo "  ✗ mutate: 判定库客户端报错（退 3，上面「✗ prune-server 客户端」那一行是原因）：判定库读不了，不每条现判，一次 cargo 都没起" >&2
    echo "     → 怎么办：照上面那一行的出路改（本机要有服务的 socket，第二台要有多机配置里的 PRUNE_SERVER_TCP_ADDRESS；帧被拒是客户端或服务的缺陷，交主 agent）；改好之后重跑同一条命令" >&2
    exit 2
  fi
  echo "mutate: 判定库那一段 python 没跑成（上面是原因），这一趟每条现判、不写库" >&2
  LIBRARY_REUSED=0; LIBRARY_FRESH=0; LIBRARY_REFUSED=0
  rm -f -- "$PLAN_FILE"
fi
if ((LIBRARY_REUSED + LIBRARY_FRESH + LIBRARY_REFUSED + ANCHOR_BROKEN_COUNT != mutation_count)); then
  LIBRARY_FRESH=$((mutation_count - LIBRARY_REUSED - LIBRARY_REFUSED - ANCHOR_BROKEN_COUNT))   # 算不出键的那一趟：没进库的都按现判数（锚点失效的不算）
fi
echo "mutate: 判定库 $MUTATE_LIBRARY：复用 $LIBRARY_REUSED 条、现判 $LIBRARY_FRESH 条、拒算 $LIBRARY_REFUSED 条、锚点失效没跑 $ANCHOR_BROKEN_COUNT 条" >&2

# 一条判完（现判的）写进判定库：档在可复用的那几种里、这一条有键才写（mutate_record 判）；写不成记进 LIBRARY_RECORD_ERRORS，收尾报，不改这一条的判定；
# 客户端退 75（prune-server 不可用）的记进 LIBRARY_UNAVAILABLE，收尾整轮退 75；退 3（客户端别的错）的记进 LIBRARY_CLIENT_ERRORS，收尾整轮退 2
# （report_library_record_errors）；两样都不当「没写进」放过
record_in_library() { # <行下标> <档> <要不要判失败> <那一行> [红了的测试名，逗号分隔]
  [[ -f "$PLAN_FILE" ]] || return 0
  local record_status=0
  local -a record_connection=()
  # 只供自证（文件头「只供自证」）：写库这一步连一个没人听的地址（查库那一段照常连得上），证客户端退 75 时整轮退 75
  [[ -n "${MUTATE_SELFTEST_RECORD_UNREACHABLE:-}" ]] && record_connection=(SINGLEFS_PRUNE_SERVER_SOCKET="$SHARD_ROOT/no-such.sock"
    SINGLEFS_PRUNE_SERVER_TCP_ADDRESS="$MUTATE_SELFTEST_RECORD_UNREACHABLE" SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=3)
  env ${record_connection[@]+"${record_connection[@]}"} python3 - "$MUTATE_LIBRARY_HELPER" "$PLAN_FILE" "$@" 2>>"$LIBRARY_RECORD_ERRORS" <<'PY' || record_status=$?
import importlib.util, sys
helper, plan, index, verdict, fails, line = sys.argv[1:7]
specification = importlib.util.spec_from_file_location("lib_mutation_units_for_mutate", helper)
units = importlib.util.module_from_spec(specification)
sys.modules[specification.name] = units
specification.loader.exec_module(units)
units.mutate_record(plan, index, verdict, fails, line, sys.argv[7] if len(sys.argv) > 7 else "")
PY
  if ((record_status == 75)) && ! is_broken recordunavailableignored; then   # 弄坏开关 recordunavailableignored：75 当「没写进」放过（改前的样子）
    echo "第 $(($1 + 1)) 条（$2）：prune-server 不可用（退 75）" >>"$LIBRARY_UNAVAILABLE"
  elif ((record_status == 3)) && ! is_broken recordclienterrorignored; then   # 弄坏开关 recordclienterrorignored：3 当「没写进」放过（改前的样子）
    echo "第 $(($1 + 1)) 条（$2）：判定库客户端报错（退 3）" >>"$LIBRARY_CLIENT_ERRORS"
  elif ((record_status != 0)); then
    echo "第 $(($1 + 1)) 条（$2）没写进判定库" >>"$LIBRARY_RECORD_ERRORS"
  fi
}
report_library_record_errors() {
  if [[ -s "$LIBRARY_UNAVAILABLE" ]]; then
    echo "  ✗ mutate: prune-server 不可用：$(grep -c . "$LIBRARY_UNAVAILABLE") 条判定写不进判定库（$(head -n 3 "$LIBRARY_UNAVAILABLE" | tr '\n' ' ')）；这一趟整轮退 75，不当「没写进、下一趟照跑」放过" >&2
    echo "     → 怎么办：python3 prune-server/control check 看服务在不在、第二台连的地址通不通；服务回来之后重跑同一条命令" >&2
    exit 75
  fi
  if [[ -s "$LIBRARY_CLIENT_ERRORS" ]]; then
    echo "  ✗ mutate: 判定库客户端报错：$(grep -c . "$LIBRARY_CLIENT_ERRORS") 条判定写不进判定库（$(head -n 3 "$LIBRARY_CLIENT_ERRORS" | tr '\n' ' ')；客户端原话：$(grep -m1 'prune-server 客户端' "$LIBRARY_RECORD_ERRORS")）；这一趟整轮退 2，不当「没写进、下一趟照跑」放过" >&2
    echo "     → 怎么办：照客户端那一句的出路改（本机要有服务的 socket，第二台要有多机配置里的 PRUNE_SERVER_TCP_ADDRESS；帧被拒是客户端或服务的缺陷，交主 agent）；改好之后重跑同一条命令" >&2
    exit 2
  fi
  if [[ -s "$LIBRARY_RECORD_ERRORS" ]]; then
    echo "mutate: 有几条判定没写进判定库（下一趟它们照跑，不影响这一趟的判定）：$(grep -c '没写进判定库' "$LIBRARY_RECORD_ERRORS") 条；原因：$(tail -n 3 "$LIBRARY_RECORD_ERRORS" | tr '\n' ' ')" >&2
  fi
}
# 复用哨兵收尾（文件头「复用哨兵」）：强制现判那一条判完的档记进状态，再判哨兵与它；红了打 ✗ 与出路、REUSE_SENTINEL_RED=1（整轮退 1，判完别的之后）
REUSE_SENTINEL_RED=0
finish_reuse_sentinel() {
  local forced_index forced_name forced_category
  [[ -f "$REUSE_SENTINEL_STATE" ]] || return 0
  if [[ -f "$REUSE_SENTINEL_STATE.forced" ]] && IFS=$'\t' read -r forced_index forced_name <"$REUSE_SENTINEL_STATE.forced" \
     && [[ -f "$RESULTS/$forced_index.verdict" ]] && read -r forced_category _ <"$RESULTS/$forced_index.verdict"; then
    case "$forced_category" in
      caught|notred|invalid) ;;
      *) echo "mutate: 复用哨兵：强制现判的 [$forced_name] 这一趟的档是 $forced_category（不进库的那几档：结果不算数或没判），不与库里的比" >&2; forced_category="" ;;
    esac
    [[ -z "$forced_category" ]] || python3 "$REUSE_CHECKS" sentinel-observe "$REUSE_SENTINEL_STATE" "$forced_name" 强制现判 "$forced_category" >/dev/null 2>&1 \
      || echo "mutate: 复用哨兵：强制现判的 [$forced_name] 这一趟的档没记进状态（$REUSE_CHECKS sentinel-observe 没成），收尾当它没跑到" >&2
  fi
  # set -o pipefail：sentinel-finish 退 1（红）时这一条管道判失败
  if ! python3 "$REUSE_CHECKS" sentinel-finish "$REUSE_SENTINEL_STATE" | sed 's/^/mutate: /' >&2; then
    REUSE_SENTINEL_RED=1
  fi
  return 0
}
LIBRARY_COUNT_TEXT="、现判 $LIBRARY_FRESH 条、复用 $LIBRARY_REUSED 条、拒算 $LIBRARY_REFUSED 条"
ANCHOR_COUNT_TEXT="、锚点失效 $ANCHOR_BROKEN_COUNT 条"   # 「计数：」里那一栏，接在现判、复用、拒算那三栏前面（文件头「锚点失效」）
# 平常跑「计数：」那一行的各栏：
# 抓到 ✅、没红 ❌、无效 ⏭、没跑完 💥、抓名字有盲区 ⚠️、锚点失效 ⚓，几栏加起来等于表的条数
caught_count=0; notred_count=0; invalid_run_count=0; crashed_count=0; nameblind_count=0; anchor_count=0
count_verdict() { # <档>
  case "$1" in
    caught) caught_count=$((caught_count + 1)) ;;
    notred) notred_count=$((notred_count + 1)) ;;
    invalid) invalid_run_count=$((invalid_run_count + 1)) ;;
    crashed) crashed_count=$((crashed_count + 1)) ;;
    nameblind) nameblind_count=$((nameblind_count + 1)) ;;
    anchor) anchor_count=$((anchor_count + 1)) ;;
  esac
}
verdict_count_text() {
  printf '抓到 %s 条、没红 %s 条、无效 %s 条、没跑完 %s 条、抓名字有盲区 %s 条、锚点失效 %s 条' \
    "$caught_count" "$notred_count" "$invalid_run_count" "$crashed_count" "$nameblind_count" "$anchor_count"
  return 0
}
# 不起 cargo 的那一支：剩下的条都按判定库复用（锚点失效的那几条本来就不跑）
ALL_REMAINING_REUSED=0
((LIBRARY_REUSED + ANCHOR_BROKEN_COUNT == mutation_count && mutation_count > 0)) && ALL_REMAINING_REUSED=1


# 全部复用（文件头「按条入库」）：不起 cargo——基线与还原之后那一次都不跑，副本里的源码一个字没改；按表序打记录里那几行，照它们的档判退出码
# （锚点失效的那几条派活之前就记好了，一起按表序打，不为它们起 cargo）
if ((ALL_REMAINING_REUSED)); then
  reused_fail=0
  for ((row_index = 0; row_index < mutation_count; row_index++)); do
    [[ -f "$RESULTS/$row_index.line" ]] && cat "$RESULTS/$row_index.line"
    read -r verdict_category verdict_fails <"$RESULTS/$row_index.verdict"
    count_verdict "$verdict_category"
    ((verdict_fails)) && reused_fail=1
  done
  echo "计数：$(verdict_count_text)$LIBRARY_COUNT_TEXT"
  if ((ANCHOR_BROKEN_COUNT)); then reused_scope="$LIBRARY_REUSED 条按判定库复用、锚点失效 $ANCHOR_BROKEN_COUNT 条没跑"; else reused_scope="$mutation_count 条都按判定库复用"; fi
  echo "全部复用：表里 $reused_scope，没起 cargo（基线与还原之后那一次都没跑，副本里的源码一个字没改；库在 $MUTATE_LIBRARY）"
  finish_reuse_sentinel
  ((REUSE_SENTINEL_RED)) && reused_fail=1
  if ((ANCHOR_BROKEN_COUNT)); then
    report_anchor_broken
    exit 3
  fi
  exit "$reused_fail"
fi

# 跑一次 cargo test：$1 副本目录、$2 编译目录。直接起，不设内存上限、不限时（文件头「上限、限时与卡住」）；
# cargo 的 stdout 与 stderr 并成一路打到这里的 stdout（判结局读它）
CARGO_TEST_COMMAND=(cargo test --release --bin "$BIN" ${cargo_feature_arguments[@]+"${cargo_feature_arguments[@]}"})
run_cargo_test() {
  ( cd "$1" && exec env ${RUSTFLAGS_ENVIRONMENT[@]+"${RUSTFLAGS_ENVIRONMENT[@]}"} ${TEST_THREADS_ENVIRONMENT[@]+"${TEST_THREADS_ENVIRONMENT[@]}"} \
      CARGO_TARGET_DIR="$2" bash -c 'exec "$@" 2>&1' mutate-cargo-test "${CARGO_TEST_COMMAND[@]}" )
}
# 在第 $1 个工作进程的副本上、用它的编译目录跑一次 cargo test（基线与还原之后那一次）
run_tests_in_copy() { run_cargo_test "$(copy_directory_of "$1")" "$(target_directory_of "$1")"; }

# 从一份 cargo test 输出（$1 是文件，- 是标准输入）取红了的用例名，一行一个；退出码照 lib_libtest_result_lines.py failed-names：
# 0 取完了（可以一个都没有），3 汇总行报了失败而一个名字都没取到，别的是解析没跑成
failed_test_names() {
  if is_broken ownnameparse; then   # 弄坏开关：改前自己的取法，「 - should panic」算进名字
    local text names
    text="$(cat -- "$1")"
    names="$(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' <<<"$text")"
    [[ -n "$names" ]] && printf '%s\n' "$names"
    if [[ -z "$names" ]] && grep -q '^test result: FAILED' <<<"$text"; then return 3; fi
    return 0
  fi
  python3 "$LIBTEST_RESULT_LINES" failed-names "$1"
}

# 基线红了（$1 第几份副本）：这份副本的基线输出从 $SHARD_ROOT 拷到它外面留着（$SHARD_ROOT 退出时整个删），
# 打红的测试名（libtest 的「test <名字> ... FAILED」行）、末尾 15 行与留下的日志路径，再给单独重现的命令
report_red_baseline() {
  local log="$SHARD_ROOT/baseline-$1.log" kept failed
  is_broken baselinelogdropped && return 0   # 弄坏开关：改前的样子，基线输出丢进 /dev/null、什么都不留
  if [[ ! -s "$log" ]]; then
    echo "        这份副本的基线一行输出都没有（cargo 没起来就退了）" >&2
    return 0
  fi
  kept="$(mktemp "${TMPDIR:-/tmp}/singlefs-mutate-baseline-XXXXXX.log")"
  cp -- "$log" "$kept"
  failed="$(failed_test_names "$log" | awk '{ names = names (count++ ? "、" : "") $0 } END { print names }')"
  if [[ -n "$failed" ]]; then
    echo "        红的测试：$failed" >&2
  else
    echo "        输出里没有「test … FAILED」那种行：多半是没编过、或测试进程没跑完，看下面末尾几行" >&2
  fi
  echo "        基线输出末尾 15 行：" >&2
  tail -n 15 -- "$kept" | sed 's/^/          /' >&2
  echo "        整份基线输出留在 $kept（看完自己删）" >&2
  echo "        → 怎么办：在 scripts/experiment-apparatus/ 里对没改过的源码单独跑一遍 $MUTATE_FLAGS_HINT ${CARGO_TEST_COMMAND[*]}，照上面红的那几条修好，绿了再整张表重跑" >&2
}

# 基线必须全绿，否则后面「红了」分不清是变异造成的还是本来就红。每份副本各跑一次、一起跑，按副本号逐个收退出码；
# 每份的输出落进 $SHARD_ROOT/baseline-<副本号>.log，红了由 report_red_baseline 点出来
baseline_pids=()
for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  if is_broken baselinelogdropped; then
    run_tests_in_copy "$worker_index" >/dev/null &
  else
    run_tests_in_copy "$worker_index" >"$SHARD_ROOT/baseline-$worker_index.log" &
  fi
  baseline_pids+=($!)
done
baseline_exits=()
for pid in "${baseline_pids[@]}"; do
  if wait "$pid"; then baseline_exits+=(0); else baseline_exits+=($?); fi
done
for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  baseline_exit="${baseline_exits[worker_index - 1]}"
  copy_note=""
  ((worker_index > 1)) && copy_note="（第 $worker_index 份副本上；第 1 份是绿的就多半是这份的编译目录坏了，删掉 $(target_directory_of "$worker_index") 再来）"
  if [[ $baseline_exit -ne 0 ]]; then
    echo "mutate: 基线就是红的${copy_note}（第 $worker_index 份副本上 cargo test 退 $baseline_exit），先修好再来" >&2
    report_red_baseline "$worker_index"
    exit 2
  fi
done
echo "基线：全绿"

# 一条变异的判定写进结果文件：$1 行号、$2 档、$3 要不要判失败（0 / 1）、$4 要打到 stdout 的那一行（可空）、$5 要打到 stderr 的话（可空）。
# .verdict 最后写、先写临时名再改名：父进程看见它就说明这一条的几份都齐了
record_verdict() {
  local row_index="$1"
  if is_broken droprow && ((row_index == 1)); then
    return
  fi
  [[ -n "$4" ]] && printf '%s\n' "$4" >"$RESULTS/$row_index.line"
  [[ -n "$5" ]] && printf '%s\n' "$5" >"$RESULTS/$row_index.stderr"
  # 抓到、没红、无效三档进按条判定库（文件头「按条入库」；$6 是抓到那一档红了的测试名，逗号分隔）；写在 .verdict 之前：父进程看见 .verdict 时库里已经有它
  case "$2" in
    caught|notred|invalid) record_in_library "$row_index" "$2" "$3" "$4" "${6:-}" ;;
  esac
  printf '%s %s\n' "$2" "$3" >"$RESULTS/$row_index.verdict.partial"
  mv "$RESULTS/$row_index.verdict.partial" "$RESULTS/$row_index.verdict"
}

# 第 $1 个工作进程判表里第 $2 条：从开跑时那一份源码施加变异、跑、判结局，写进结果文件（record_verdict）
judge_mutation() {
  local worker_index="$1" row_index="$2" name copy_directory target_directory out sig red red_names names_status
  name="${mutation_names[row_index]}"
  copy_directory="$(copy_directory_of "$worker_index")"
  target_directory="$(target_directory_of "$worker_index")"
  if is_broken sharedcopy; then
    copy_directory="$(copy_directory_of 1)"; target_directory="$(target_directory_of 1)"
  fi
  if ! write_mutated_source "${mutation_originals[row_index]}" "${mutation_replacements[row_index]}" "$copy_directory/$SRC" 2>"$RESULTS/$row_index.anchor"; then
    # 派活之前核过的锚点在这里又对不上：同样记锚点失效（文件头「锚点失效」），不停别的工作进程
    record_verdict "$row_index" anchor 1 "⚓ [$name] 锚点失效：原文在 $SRC 里$(sed -n '1s/：.*//p' "$RESULTS/$row_index.anchor")—— 这一条没跑，不算抓到也不算没红" ""
    return
  fi
  # ⚠️ **测试根本不终止、或无界分配：** 一条破坏可以让被测代码进死循环或吃满内存（2026-09-01 E73 整轮挂死，2026-09-25 E142 两次把整机拖进 OOM）。
  # ⇒ 这里不限时、不设内存上限（用户 2026-10-31 定「照旧交服务，服务加一条 工具里不限时」）：整条 mutate.sh 交统一任务服务，
  #   内存上限由服务给、撞了放大重排，用时远超记录而日志又满 10 分钟不涨由服务停掉、打回（文件头「上限、限时与卡住」）。
  out="$(run_cargo_test "$copy_directory" "$target_directory")"
  cp "$PRISTINE" "$copy_directory/$SRC"   # 还原：下一条从开跑时那一份改，领完之后那一次测试也跑在还原过的源码上
  # 测试进程退几都照输出判（文件头「退出码」）：点名的测试红了就是抓到。
  # ⚠️ **编译失败不等于「没抓到」。** 一个改坏了语法或穷尽性的变异根本跑不到测试，
  # 那时既不能记成「测试抓到了」，也不能记成「测试没抓到」——它是一条**无效变异**。
  # 混成一类的话，一个编译不过的变异会被报成测试盲区，把人引去改测试（实测踩过）。
  # ⚠️ 判据是「测试进程有没有跑起来」，**不是「输出里有没有 error」**——
  # `cargo test` 在测试变红时也会打 `error: test failed`，
  # 拿它当编译失败的判据会把每一条成功的变异都误判成无效（实测踩过，全套复跑才发现）。
  # ⚠️ **不许写成 `printf ... | grep -q ...`。** `grep -q` 命中后立刻退出并关闭管道，
  # 上游 printf 收到 SIGPIPE；本脚本开了 `pipefail` ⇒ **整条管道判失败，命中被读成没命中**。
  # 实测：那样写会让每一条「测试确实变红了」的变异都被误报成「编译失败」。
  # 这正是 `.claude/singlefs-ai-sop/rules/command-safety.md`「管道里的退出码不是你想要的那个」。
  # ⇒ 用 here-string，根本不建管道。
  if ! grep -q '^running [0-9]* test' <<<"$out"; then
    record_verdict "$row_index" invalid 0 "⏭  [$name] 变异导致编译失败，本条无效（不计入盲区，也不算命中）" ""
    return
  fi
  # ⚠️ **第四种结局：测试进程根本没跑完**（挂死被 OOM 杀、段错误、abort）。
  # 它既不是「编译失败」也不是「一个测试都没红」——报成后者会把人引去改测试，
  # 而实际发生的事是「这条破坏让被测代码不终止了」。实测踩过：
  # 摘掉一条循环终止条件之后 recover 无限接受记录，进程 SIGKILL，
  # 当时被报成「没有被任何检查看见」。
  if grep -q "process didn't exit successfully" <<<"$out"; then
    sig="$(sed -n 's/.*(signal: \([0-9]*\).*/\1/p' <<<"$out" | head -1)"
    record_verdict "$row_index" crashed 0 "💥 [$name] 测试进程没跑完（signal ${sig:-?}）——破坏被看见了，但不是断言抓到的" ""
    return
  fi
  # ⚠️ **不许枚举字符类——这个坑犯过两次，形状一模一样。**
  # 2026-08-29 写成 `[a-z_]*`，名字带数字的测试匹配不上；
  # 2026-08-31 写成 `[A-Za-z0-9_]*`，**中文测试名**匹配不上——E57 的 9 条变异
  # 全被报成「一个测试都没红」，实际 9 条全红。两次都是**谎报盲区**（把人引去补
  # 一条本来就有的检查），而不是漏报命中，所以两次都很难自己发现。
  # ⇒ 改成「`...` 之前的都算名字」：Rust 的测试名可以是任意 Unicode 标识符，
  #   枚举允许的字符永远追不上。下面那条旧注释留着，它记的是同一个坑的第一次。
  # ⚠️ **字符类必须含数字与大写。** 写成 `[a-z_]*` 时，任何名字里带数字的测试
  # （例如 `..._analytic_io_per_op_of_1_9375`、`..._reads_exactly_122`）匹配不上，
  # `red` 为空 ⇒ **一条确实红了的变异被报成「一个测试都没红」**。
  # 方向是谎报盲区，会把人引去补一条本来就有的检查。2026-08-29 实测踩过并修。
  # ⚠️ **第三次同形：模块名也不许写死。** 2026-09-17 写的是 `^test tests::`，E154 的单测分在
  # `gate_tests` 与 `integration_tests` 两个模块里，7 条变异全被报成「一个测试都没红」，合并成 `mod tests` 之后 7 条全红。
  # ⇒ `test ` 与 ` ... FAILED` 之间的整段都算名字（含模块路径）；另加一道兜底：测试进程报了失败、名字却一个没抓到，
  #   说明抓取规则又有盲区，单独报，不报成「没被看见」。
  #   名字的解析在 lib_libtest_result_lines.py（failed_test_names）：「 - should panic」不进名字，汇总行报了失败而名字一个没取到时它退 3。
  if red_names="$(failed_test_names - <<<"$out")"; then names_status=0; else names_status=$?; fi
  red="$(paste -sd, - <<<"$red_names")"
  if ((names_status != 0 && names_status != 3)); then
    record_verdict "$row_index" nameblind 1 "⚠️  [$name] 取红了哪几个没跑成（lib_libtest_result_lines.py failed-names 退 $names_status）—— 这一条不算盲区也不算命中" ""
  elif ((names_status == 3)); then
    record_verdict "$row_index" nameblind 1 "⚠️  [$name] 测试进程报了失败，但一个失败的测试名都没抓到 —— mutate.sh 抓名字的规则有盲区，这一条不算盲区也不算命中" ""
  elif [[ -z "$red" ]]; then
    record_verdict "$row_index" notred 1 "❌ [$name] 一个测试都没红 —— 这条破坏没有被任何检查看见" ""
  else
    record_verdict "$row_index" caught 0 "✅ [$name] 红：$red" "" "$red"
  fi
}

# 第 $1 个工作进程：按表序认领还没人领的变异、逐条判；领完在自己的副本上跑一次还原之后的测试，退出码写进 restore-<号>.exit。
# 它的 stdout 与 stderr 都进自己的日志（worker-<号>.log），一个字都不往这个脚本的 stdout 写
run_worker() {
  local worker_index="$1" row_index restore_exit
  trap - EXIT
  for ((row_index = 0; row_index < mutation_count; row_index++)); do
    mkdir "$CLAIMS/$row_index" 2>/dev/null || continue   # 认领：mkdir 是原子的，同一条只有一个工作进程建得成
    if is_broken sharedcopy; then
      # 开关 sharedcopy 下各工作进程都用第 1 份副本：逐条加锁轮流用（文件头那一条），并发改同一份源码会把别的例弄得红不红不定
      { flock 9; judge_mutation "$worker_index" "$row_index"; } 9>>"$SHARD_ROOT/sharedcopy.lock"
    else
      judge_mutation "$worker_index" "$row_index"
    fi
  done
  if is_broken sharedcopy; then
    if { flock 9; run_tests_in_copy "$worker_index" >/dev/null; } 9>>"$SHARD_ROOT/sharedcopy.lock"; then restore_exit=0; else restore_exit=$?; fi
  elif run_tests_in_copy "$worker_index" >/dev/null; then restore_exit=0; else restore_exit=$?; fi
  echo "$restore_exit" >"$SHARD_ROOT/restore-$worker_index.exit"
}

for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  run_worker "$worker_index" >"$SHARD_ROOT/worker-$worker_index.log" 2>&1 &
  worker_pids+=($!)
done

# 父进程：按表序把已经判完的那几条接着打到 stdout（前面有一条还没判完就停下等它），顺手计数；锚点失效的照打、单数一栏（文件头「锚点失效」）
fail=0
next_row=0
print_judged_rows_in_table_order() {
  local verdict_category verdict_fails
  while ((next_row < mutation_count)) && [[ -f "$RESULTS/$next_row.verdict" ]]; do
    read -r verdict_category verdict_fails <"$RESULTS/$next_row.verdict"
    [[ -f "$RESULTS/$next_row.line" ]] && cat "$RESULTS/$next_row.line"
    count_verdict "$verdict_category"
    ((verdict_fails)) && fail=1
    next_row=$((next_row + 1))
  done
}
any_worker_alive() {
  local pid
  for pid in "${worker_pids[@]}"; do
    kill -0 "$pid" 2>/dev/null && return 0
  done
  return 1
}
while any_worker_alive; do
  print_judged_rows_in_table_order
  sleep "$COLLECTION_POLL_SECONDS"
done
worker_exits=()
for pid in "${worker_pids[@]}"; do
  if wait "$pid"; then worker_exits+=(0); else worker_exits+=($?); fi
done
print_judged_rows_in_table_order

# 派出去多少条就要收回来多少份判定：表里的条数、被认领的条数、收回的判定份数、按表序打到的位置，四个数要对得上
claimed_count=0
collected_count=0
missing_names=()
for ((row_index = 0; row_index < mutation_count; row_index++)); do
  [[ -d "$CLAIMS/$row_index" ]] && claimed_count=$((claimed_count + 1))
  if [[ -f "$RESULTS/$row_index.verdict" ]]; then
    collected_count=$((collected_count + 1))
  else
    missing_names+=("${mutation_names[row_index]}")
  fi
done
unhealthy_workers=""
for worker_index in "${!worker_exits[@]}"; do
  ((worker_exits[worker_index] != 0)) && unhealthy_workers+="${unhealthy_workers:+、}第 $((worker_index + 1)) 个退 ${worker_exits[worker_index]}"
done
if is_broken countblind; then
  # 弄坏开关：不数，只把在的那几份按表序打出来（并行化之后最容易写成的那种收法）
  while ((next_row < mutation_count)); do
    if [[ -f "$RESULTS/$next_row.verdict" ]]; then print_judged_rows_in_table_order; else next_row=$((next_row + 1)); fi
  done
elif ((claimed_count != mutation_count || collected_count != mutation_count || next_row != mutation_count)) || [[ -n "$unhealthy_workers" ]]; then
  echo "mutate: 派出去 $mutation_count 条变异，被认领 $claimed_count 条、收回 $collected_count 份判定，按表序打到第 $next_row 条；工作进程退出码不是 0 的：${unhealthy_workers:-无}" >&2
  if ((${#missing_names[@]})); then
    echo "        没收回判定的：$(printf '%s、' "${missing_names[@]}" | sed 's/、$//')" >&2
  fi
  for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
    if [[ -s "$SHARD_ROOT/worker-$worker_index.log" ]]; then
      echo "        第 $worker_index 个工作进程的日志末尾：" >&2
      tail -n 5 "$SHARD_ROOT/worker-$worker_index.log" | sed 's/^/          /' >&2
    fi
  done
  echo "        → 怎么办：这是并行收束自己的完整性闸红了，这一轮的判定不算数；MUTATE_JOBS=1 再跑一遍看串行下齐不齐，" >&2
  echo "          齐就去查认领与收束那一段（run_worker、record_verdict、print_judged_rows_in_table_order）；工作进程被信号杀的，先查是不是被 OOM 或人停了" >&2
  exit 8
fi
# 「计数：」按档各数各的（count_verdict），几栏加起来等于表的条数
echo "计数：$(verdict_count_text)$LIBRARY_COUNT_TEXT"
report_library_record_errors
finish_reuse_sentinel
((REUSE_SENTINEL_RED)) && fail=1

# 还原之后那一次：每个工作进程领完之后在自己的副本上各跑了一次
for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  restore_file="$SHARD_ROOT/restore-$worker_index.exit"
  if [[ ! -f "$restore_file" ]]; then
    echo "mutate: 第 $worker_index 份副本还原之后那一次没跑成（工作进程没留下退出码）" >&2
    echo "        → 怎么办：MUTATE_JOBS=1 再跑一遍；还是没跑成就去查 run_worker 收尾那几行" >&2
    exit 4
  fi
  restore_exit="$(cat "$restore_file")"
  if [[ "$restore_exit" != 0 ]]; then
    echo "mutate: 还原后没回到全绿（第 $worker_index 份副本，退出码 $restore_exit）" >&2
    exit 4
  fi
done
echo "已还原，基线仍全绿"
# 这一轮量的是开跑时那一份源码：跑的这几分钟里有人改了工作区里的原件，结果就不算数，要重跑。
if [[ "$(sha256sum "$SRC" | cut -d' ' -f1)" != "$ORIGINAL_SUM" ]]; then
  echo "mutate: $SRC 在这一轮变异跑的时候被改过；上面的结果量的是开跑时那一份，改完之后重跑" >&2
  exit 5
fi
# 锚点失效的不为 0：别的条照跑完了，整轮退 3（文件头「锚点失效」）
if ((anchor_count)); then
  ANCHOR_BROKEN_COUNT="$anchor_count"
  report_anchor_broken
  exit 3
fi
exit $fail
