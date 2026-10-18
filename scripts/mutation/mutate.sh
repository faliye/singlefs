#!/usr/bin/env bash
# admission: always 每一次调都逐条查按条判定库：键对得上的复用、对不上的现判（键与库写在文件头「按条入库」），整张表的结论不替哪一条作保
# run-condition: command cargo git python3
# 证明「这些测试会红」：逐条注入一个已知的破坏，跑测试，记下红了哪几个，然后还原。
# 红了哪几个（libtest 的「test <名字>[ - should panic] ... FAILED」行）与汇总行报没报失败，经 scripts/gate/lib_libtest_result_lines.py failed-names 取（全仓一份解析）：
#   #[should_panic] 的用例那一段「 - should panic」不进名字；汇总行报了失败而一个名字都没取到，它退 3，这里记「抓名字的规则有盲区」。
#
#   mutate.sh [--full] <bin 名> <源文件> <变异表>      # 在 scripts/experiments/ 里跑；参数不够三个退 2，打用法与一条例子
#   mutate.sh [--full] --compile-only <bin 名> <源文件> <变异表>   # 只编不跑（门禁 checker-tier-research-build-and-replay 的 mutation-tables-compile 格调它），见下面「只编不跑」
#   --full：只写不读（每条现判，照键写回同一份按条判定库），见下面「按条入库」
#
# 按条入库（判法在 scripts/mutation/lib_mutation_units.py 的 mutate_plan 与 mutate_record，单元的切分与闭包在 scripts/gate/lib_function_units.py）：
#   每条变异一条判定：平常跑记「抓到 / 没红 / 无效 / 超时」，只编不跑记「编得过 / 编不过」，按键存一个文件（判定、要打的那一行、判定日期、跑法）；
#   键是原文、替换文、包住原文的 fn 单元与它依赖的单元指纹（入口是原文所在的最里层单元，lib_function_units.py 的 Rust 闭包）、cargo 参数与特性
#   （bin 名不进）、工具链行（rustc -vV、cargo -V、这里给每个 cargo 带的 RUSTFLAGS 与构建配置解析后的值）、解析出的依赖版本行（包住原文那几个单元的闭包里真点名的外部 crate 在 scripts/experiments/Cargo.lock 里落到的包与传递依赖，
#   lib_mutation_units.py 文件头「依赖版本行」；落不到包的那一条拒算）；
#   不带变异名、表名、行号、内存上限、限时。平常跑复用另要：抓到的，红了的那几条测试此刻的闭包键与记录里的相同；没红、无效、超时的，
#   这个 bin 全部 #[test] 的闭包键合起来与记录里的相同；超时的另要这一趟的限时不比记录里的长。内存撞顶、排不上、被总上限挤掉、显存不够、
#   抓名字的规则有盲区那几档不记、每趟照跑。原文不落在任何单元上（只在注释里）、闭包里有拒算的那一条拒算：每趟照跑、不写库。
#   库在 MUTATE_LIBRARY_DIRECTORY（设了用它），不设是这个仓的 git common-dir 底下的 singlefs-mutate-library/
#   （取不到 git common-dir 时 ~/.local/share/singlefs/mutate-library），不进 /tmp；读到、写到的都摸成此刻，30 天里没被任何一趟引用的收尾删掉。
#   全部复用时不起 cargo：基线与还原之后那一次都不跑、副本里的源码一个字没改，收尾打一行「全部复用」代替「已还原，基线仍全绿」；
#   有一条要现判就照原来的样子跑基线、派活、跑还原之后那一次。
#   复用哨兵（每一趟都插，规则照 scripts/gate/lib_reuse_checks.py 文件头「二」，判法是那里的 ReuseSentinel）：查完库、派活之前抽。候选是此刻键算得出、
#   records/<键> 记着可复用的档、复用要的另几样（红了的测试的闭包键、全部测试的闭包键、超时的限时）也对得上的那几条——直接读计划文件与记录文件，与 mutate_decision 分开查；
#   种子取 SINGLEFS_REUSE_SENTINEL_SEED（不设现取、打在 stderr），抽 SINGLEFS_REUSE_SENTINEL_COUNT 条（默认 3）当哨兵、再抽 1 条强制现判：
#   强制现判那一条不用库里的判定、交给工作进程照现判跑，stderr 写「现判：[<名>] 强制现判（复用哨兵抽中…）」，计数里算现判；收尾（finish_reuse_sentinel，「计数：」那一行之后）
#   拿它这一趟的档与库里的比、再判哨兵：哨兵有一条被现判「复用失灵」、强制现判的档与库里不同「判定不稳」，打 ✗ 与出路、整轮退 1（别的照判完）。
#   --full 不抽（打「不抽哨兵」）；键相同的不足 K 条报「哨兵不足 K 条」，不红；拷进不带 lib_reuse_checks.py 的树里跑的这一份说一句、不抽。状态在这一轮自己的 SHARD_ROOT 里。收尾那行「计数：」末尾接「、锚点失效 A 条、现判 N 条、复用 M 条、拒算 R 条」（只编不跑那一行同样），
#   stderr 逐条写现判与拒算的原因。判法（这份脚本判结局的那几支）不进键：改了判法要 --full 跑一趟（这份 bash 的函数闭包在 lib_function_units.py 里拒算）。
#   mutate.sh --binary-of <源文件>            # 打 cargo 从这份源文件编出的 bin 名（与下面「改的文件与跑的二进制对得上」那道核对同一份读法），在 scripts/experiments/ 里跑
#
# 只编不跑（--compile-only）：锚点核对、施加替换文、bin 与源文件的核对、required-features 的读法都与平常跑同一套；
#   不同的只有每条跑什么：基线与每条变异只跑一次 `cargo check --profile test --bin <名>`（带同样的 --features），环境里 CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=false，不跑测试。
#   为什么是 test 档：平常跑的 cargo test --release 编的是带 cfg(test) 的那一份，`cargo check --release` 编的是不带的；只在测试外用不到的函数、常量
#   在后者报 dead_code，在前者不报（2026-10-13 量的：E142 第二十五次那张旧表，`--release` 判 34 条编不过、多出 M153、M225、M229 三条，
#   test 档判 31 条、与 research/results/e142-mutate-r25-2026-10-13.log 的 31 条 ⏭ 逐个相同）。test 档默认开 debug_assertions，release 不开，关掉它让 cfg 与 cargo test --release 相同。
#   串行、一份副本、一个编译目录（MUTATE_TARGET_DIR，没设取 ${TMPDIR:-/tmp}/singlefs-mutate-target），增量编；不经内存包装（cargo check 不跑编出来的代码），不开工作进程。
#   每条打一行：编得过「✅ [<名>] 编得过」，编不过「⏭  [<名>] 编不过：<cargo 输出里第一行 error>」；末尾打「计数：」一行，
#   派出去的条数（表里的）与收回的判定份数对不上退 8。退出码：0 全编得过；1 有编不过的；2 用法、环境或基线（没改过的源码）就编不过；3、5、6、8 同平常跑。
#
# 变异表每行三段，用制表符分隔：变异名 <TAB> 原文 <TAB> 替换文（原文/替换文里的 \n 表示换行）
# 替换必须恰好命中一次，不是的那一条照「锚点失效」记、不当成跑过 —— 静默失效的替换会让整份证明变成假的
# （singlefs-ai-sop/rules/command-safety.md「脚本改文件之后要回读确认」）。
#
# 锚点失效（.claude/rules/mutation-sampling.md 第七类）：原文在源码里命中 0 次或多次的那一条，派活之前就记成「⚓ [<名>] 锚点失效：原文在 <源文件> 里命中 N 次……」，
#   不起 cargo、不进按条判定库、不复用；别的条照跑照判、照入库。收尾「计数：」里一栏「锚点失效 A 条」（接在现判、复用、拒算那三栏前面），A 不为 0 整轮退 3（判完别的条之后）。
#   只编不跑同样按条。弄坏开关 MUTATE_BREAK=anchorstopsall 照改前的跑法：一条锚点失效就派活之前退 3、一条变异都不跑。
#
#   mutate.sh --selftest    拿一个假的 cargo 走一遍各种结局（抓到、没红、无效、超时、内存撞顶、基线撞顶、scope 起不来），
#                           再走一遍并行的几样（进程数不同 stdout 逐字相同、各用各的副本、丢一份结果整道判红、锚点失效那一条记 ⚓ 而别的照跑、整轮退 3，只编不跑同样），
#                           再走一遍 required-features（挂了的 bin 带上 --features 跑得起来，没挂的 bin 的 cargo 参数与改前逐字相同），
#                           再走一遍包装排队（包装每次放行前多停 5 秒，五种结局照样各归各类：限时不算排队的时间），
#                           再走一遍包装的另三种结局（假包装退 252 排不上、退 254 被总上限挤掉、退 245 显存不够：各归各类、整轮判失败），
#                           再走一遍平常跑带只供自证的变量（每个变量各一次，都退 2、一次 cargo 都不调），
#                           再走一遍会话里平常跑并行（不经自证的标记：一份原样副本配假包装，照包装的「日常一条」判，账上先放一条这个会话在跑的）：
#                           MUTATE_JOBS=4 照样各条判对、整轮退 0，每次包装带各自的并行理由、--check 不带，调用方设了的理由不覆盖，基线排不上报「基线排不上」。
#                           再走一遍只编不跑（--compile-only）的三种结局：编得过、编不过（整轮退 1、打第一行 error）、收回的判定与派出去的对不上（退 8）；
#                           只在测试外用不到的那一条照 test 档判编得过，cargo 只调 check、参数逐字是「check --profile test --bin fake-bench」、一次 test 都不调；
#                           基线编不过退 2；--binary-of 显式 [[bin]] 打声明的名字、没写的打文件名去掉 .rs。
#                           再走一遍 deny 警告（假 cargo 记下每次收到的 RUSTFLAGS；替换文留死代码的那一条 RUSTFLAGS 里没有 --cap-lints warn 就编不过）：
#                           平常跑调用方没设、设了 RUSTFLAGS 各一例判抓到，只编不跑一例判编得过；基线、那一条与还原之后那一次收到的 RUSTFLAGS 逐字相同，调用方的接在前面。
#                           再喂环境里设了 CARGO_ENCODED_RUSTFLAGS 的三例（设空串、设了别的值各一例平常跑，设空串一例只编不跑；假 cargo 照真 cargo 只读它、不读 RUSTFLAGS）：
#                           留死代码那一条照样判抓到 / 编得过，每个 cargo 收到的 CARGO_ENCODED_RUSTFLAGS 逐字是「调用方的（有就在前）+ 0x1f + --cap-lints + 0x1f + warn」，stderr 报的与收到的一致。
#                           再走一遍按条入库（被测源码换成一份带 #[test] 的）：同一条第二趟复用、一次 cargo 都不起（「全部复用」），
#                           改了被改坏函数调到的函数之后那一条现判，--full 只写不读，原文只在注释里的那一条拒算照跑，只编不跑第二趟同样复用；
#                           依赖版本行只收闭包里真点名的外部 crate（夹具 Cargo.lock：头一趟逐字是 extdep 与传递依赖 inner 两行，不点外部 crate 的是空的，
#                           改闭包外点的 other 的版本照样复用，改 inner 的版本现判，认 package = 改名）。
#                           再走一遍副本范围：装置 include_str! 读仓根下别处的文件（../../.claude/heavy-thread-default）时副本根下链上、基线编得过、照常判抓到；
#                           认不出的写法（concat!）与落到仓根之外的路径退 2、逐行点名、一次 cargo 都不起。
#                           再走一遍复用三问（reuse-three-questions，scripts/gate/lib_reuse_checks.py 文件头「一」）：两条抓到的表，头一趟两条现判、同一份输入第二趟全复用、
#                           加一个没人调的函数全复用、改「改孤函数」那一条闭包里的 lonely_helper 只现判它，末尾打规范输出 REUSE_SELFTEST script=scripts/mutation/mutate.sh …；
#                           再走一遍复用哨兵（reuse-sentinel，共用库文件头「二」）：种子 5、哨兵 1 条那一趟一条强制现判、与库里同为「caught」、退 0，不读库那一趟红「复用失灵」、退 1。
#                           每一例各用一份自己的临时目录：假 cargo、假 workspace、编译目录、按条判定库（MUTATE_LIBRARY_DIRECTORY）、包装的锁与账（RUN_WITH_MEMORY_CAP_STATE_DIR）
#                           与峰值表（RUN_WITH_MEMORY_CAP_PEAKS）都在里面，不读别的例留下的东西，也不读写主仓的峰值表；复用哨兵的状态在每一趟自己的临时根（SHARD_ROOT）里。
#                           真包装的 scope 都挂进这一趟自证自己的 slice（singlefs_mutate_selftest_<进程号>.slice，总上限 2G，收尾 stop 加 revert），
#                           不与别的会话、同时起的别的自证挤主仓那一个 slice 的总上限与排队；判超时的那几例限时 5 秒，别的 25 秒（机器忙时起 scope 慢，不把它算成超时）
#
# bin 在 Cargo.toml 的 [[bin]] 里挂了 required-features 的：每次 cargo test 带 `--features <包名>/<feature>,…`，
#   feature 从那一节现读，不在命令行上手敲、不写死在这里；没挂的 bin，cargo 的参数与改前逐字相同（`test --release --bin <名>`）。
#   不带的话 cargo 不编这个 bin、直接退 101，基线停在「基线就是红的」，一条变异都没跑。
#   那几个 feature 的依赖要的环境（E162 的 rocksdb 要 BINDGEN_EXTRA_CLANG_ARGS）由调用方设，原样传给 cargo。
#
# deny 警告：scripts/experiments/Cargo.toml 设了 [lints.rust] warnings = "deny"，替换文多半故意删掉用法、留下死代码，deny 下那是编译错
#   （平常跑记「⏭ 无效」、只编不跑记「编不过」），不是被断言抓到。所以这里起的每个 cargo（基线、每条变异、还原之后那一次、只编不跑的 cargo check）
#   都带 RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }--cap-lints warn"：调用方设了的 RUSTFLAGS 在前，后面接 --cap-lints warn，lint 一律压成警告，别的编译错照样编不过。
#   这一份开跑前算一次（MUTATE_RUSTFLAGS），一张表里基线、每条变异与还原之后那一次逐字相同：RUSTFLAGS 进 cargo 的指纹，不同就来回重编。
#   开跑时 stderr 打一行「每个 cargo 都带 RUSTFLAGS=「…」」。只有 mutate.sh 加：replay.sh 的产物复跑、门禁 research-unit-tests 格与研究 bin 平常的编译照 deny。
#   调用方的 RUSTFLAGS 里自己带了 --cap-lints 的，rustc 认第一个（rustc 1.98 量的），以调用方的为准。
#   环境里设了 CARGO_ENCODED_RUSTFLAGS（空串也算）时 cargo 只读它、不读 RUSTFLAGS，上面那个 RUSTFLAGS 就悄悄失效：所以这时再给每个 cargo 同时设
#   CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS:+$CARGO_ENCODED_RUSTFLAGS<0x1f>}--cap-lints<0x1f>warn"（各项以 0x1f 分隔，调用方设了的在前），RUSTFLAGS 照旧也设；
#   开跑时 stderr 那一行改报 cargo 实收的那一个变量与值（「每个 cargo 都带 CARGO_ENCODED_RUSTFLAGS=「…」」，0x1f 显示成 ␟）。没设 CARGO_ENCODED_RUSTFLAGS 的，一个字不变。
#
# gate-overlap:copy-kept scripts/mutation/crates-mutation-rows.py 开几个工作进程（worker_count_and_head_line）照那边的 worker_count_for 写：各项取最小、至少 1、核数一半至多 16；两边的项不同（这里 MUTATE_JOBS 只能压小、第二项是表里的条数、另算每个 cargo 的编译并行度；那边 GATE_MUTATION_WORKERS 顶掉核数那一项、第二项是要现跑的条数），派活一边是 bash 子 shell 按目录认领、一边是 python 进程池，共用的只剩一个取最小值，抽成共用要这里每趟多起一个 python 只为算一个数、两边头一行的字也不同
# 并行（command-safety.md「一个脚本里的检测项，能并行就并行」，形态照 .claude/gate.d/checker-tier-crates-mutation-replay.sh 与它的判法 scripts/mutation/crates-mutation-rows.py）：
#   每个工作进程一份被测装置的副本（scripts/experiments/ 里编译要用的那部分）、一个自己的 CARGO_TARGET_DIR，不在同一份源码上并发改：
#   第 1 个用 MUTATE_TARGET_DIR（跨轮热着），第 i 个用它后面加 -w<i>（跨轮复用，第一次要冷编译）。
#   工作进程数取 min(核数的一半且至多 16、表里的条数)，至少 1；MUTATE_JOBS 只能压小、不能加大（设成 1 就是原来的串行跑法）。
#   实际开几个、各项给几个、卡在哪一项、每个 cargo 的编译并行度，打在 stderr 的头一行。
#   内存不在这里判：每条 cargo test 照旧经 run-with-memory-cap.sh 跑，几条同时跑要多少、放不放得下由它管，这里不按内存收工作进程数。
#   每个 cargo 的编译并行度：先用 MUTATE_CARGO_JOBS，其次调用方已经设了的 CARGO_BUILD_JOBS，都没有就取 核数 ÷ 工作进程数（至少 1），免得 N 个 cargo 各按核数开 rustc。
#   基线在每份副本上各跑一次（第 1 份那次就是原来的基线，其余几份顺带把各自的编译目录热起来，不占每条变异的限时），还原之后那一次也在每份副本上各跑一次。
#   锚点在派活之前逐条核（每条原文在源码里恰好命中一次），不是的那一条见「锚点失效」，别的照派。
#   活是动态领的：每条变异一个认领目录（mkdir 是原子的），谁先跑完谁领表里下一条还没人领的。
#   每条变异的判定写进自己的结果文件，父进程按表里的次序逐个读、逐行打到 stdout；工作进程一个字都不往 stdout 写。
#   stdout 与进程数无关：MUTATE_JOBS=1 与 =4 逐字相同；头一行在 stderr，是两次跑唯一不同的那一行。
#   派出去（表里）多少条、被认领多少条、收回多少份判定，数一遍对不上就退 8，不许少一条还报绿（command-safety.md「并行不许把失败吃掉」）。
#   会话里平常跑（包装文件头「日常一条」：从 Claude Code 会话里起、不带 SINGLEFS_HEAVY_TESTS 的，同一个会话账上已有一条在跑、这一条没给至少 8 个字的理由就退 252）：
#   这里自己起的每次包装（基线、每条变异、还原之后那一次）都带 RUN_WITH_MEMORY_CAP_PARALLEL_REASON=「同一次 mutate.sh 的第 <i> 个工作进程，各用各的副本与编译目录……」，
#   不然第 2 个工作进程起就排不上。调用方已经设了（非空）的照原样传、不覆盖。只设在这几次包装的环境里、不 export：
#   --check 与别的命令看不见它，包装把它清掉之后才起 cargo，不给 mutate.sh 之外的命令放宽「日常一条」。
#
# 退出码：0 每条都判完、没有判失败的；1 有没红、超时、内存撞顶、排不上（包装退 252）、被总上限挤掉（包装退 254）、显存不够（包装退 245）或抓名字的规则有盲区；
#   2 用法、环境或基线不绿（基线排不上也退 2，报「基线排不上」），或平常跑时带了只供自证的变量（见「只供自证」）；
#   3 有锚点失效的条（别的条照跑完；同时有 1 那几种的也退 3，逐条的行照打）；
#   4 还原之后没回到全绿；5 跑的时候工作区里的源码被改过；6 源文件与二进制对不上；7 带上限的 scope 起不来；8 收回的判定与派出去的对不上；
#   254 基线被内存总上限挤掉：包装的结局原样转出，这一次不算数、一条变异都没跑，整张表重跑。
#   基线就是红的（退 2）：stderr 点出红的测试名（libtest 的「test <名字> ... FAILED」行）、打基线输出末尾 15 行，
#   整份基线输出拷到 ${TMPDIR:-/tmp}/singlefs-mutate-baseline-XXXXXX.log 留着（点出路径，看完自己删），再给单独重现的命令。
#
# 每次 cargo test（基线、每条变异、还原之后那一次）都放进内存上限里跑（scripts/hosts/run-with-memory-cap.sh：systemd 的临时 scope，
# MemoryMax=<上限>、MemorySwapMax=0），撞上限只杀这一次 cargo test 里的进程，不把整机拖进 OOM
# （2026-09-25 两次整机 OOM 连带杀掉本地模型服务与 Claude Code 会话，records/2026-09-16-subagent拆分提案.md 第四十节第 28 行）。
#   MUTATE_MEMORY_MAX  上限，systemd 写法，默认 16G。依据：本机 60 GiB 内存，本地模型服务连同会话常驻约 12 GiB
#                      （2026-09-25 `free -g` 的 used 列是 12），同时可能有 3 件重活 ⇒ 每件 (60 − 12) ÷ 3 = 16 GiB；
#                      一条变异按一件重活的份额给；同时跑几条时合计由 run-with-memory-cap.sh 管（见「并行」那一段）。换机器、同时跑的重活变多都要重算（这些数只在本机成立）。
#   撞了上限记「内存撞顶」，与「超时」同一类：这条破坏让被测代码无界分配，不是「没抓到」，也不算「抓到」；整轮判失败，收尾单列计数。
#   带上限的 scope 起不来（没有用户级 systemd、D-Bus 连不上）就退 7，一条都不跑，不退回无上限去跑。
# 每条变异限时 MUTATE_TIMEOUT（正整数秒，默认 120），交给包装的 RUN_WITH_MEMORY_CAP_TIME_LIMIT（scope 的 RuntimeMaxSec）：从命令起跑算、不算排队的时间；
#   到点 systemd 停掉 scope 里的进程，包装退 253，这一条记「超时」。不在包装外面套 timeout：套在外面的把排队拿锁、起 scope 的时间也算进去，
#   机器一忙，撞顶的、抓到的那几条先被记成超时（run-with-memory-cap.sh 文件头「限时」一段）。基线与还原之后那一次不限时。
# 包装的另三种结局（run-with-memory-cap.sh 文件头「退出码」一段）各单列一类，与超时、撞顶一样整轮判失败、收尾单列计数：
#   退 252「排不上」：内存不够排不上——等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 秒还放不下、这一条要的量比总上限还大（包装 2026-10-06 撤了「日常一条」那两道拒绝，
#     见包装文件头「日常一条」一段；这里给每次包装设的并行理由包装已经不看，设着无害），命令一行都没跑。记「⏳ 排不上」：这一条没判；不落进「编译失败、无效」那一支
#     （输出是空的，按「有没有 running N tests」判就成了无效、不判失败）。
#   退 254「被总上限挤掉」：整个 slice 满了，内核在 slice 里挑了这一条杀，不是它自己撞了上限；包装说它的结果不算数、要重跑。
#     记「🚧 被总上限挤掉」：不落进「测试进程没跑完、破坏被看见了」那一支（cargo 打的 process didn't exit successfully 是被挤掉造成的）。
#   有这两种的，收尾在 stderr 说要整张表重跑。基线退 252 报「基线排不上」、退 2（不说成「基线就是红的」）；基线退 254 报「基线被内存总上限挤掉」、
#   原样退 254（不说成「基线就是红的」，也不并进退 2）；
#   还原之后那一次退这两个码，按还原后不绿退 4。
#   退 245「显存不够」（包装 2026-10-06 加的，它文件头「显存不够」一段）：命令退了非 0、起止之间内核报了 NVRM … NV_ERR_NO_MEMORY，结果不算数、要重跑。
#     记「🎴 显存不够」：不落进下面「抓到」「没跑完」那几支（测试是被显存不够弄红的，不是破坏被看见了）；收尾同样说要整张表重跑。基线退 245 照「基线就是红的」退 2。
# 包装自己的话（排队时每 60 秒报一行在等什么、撞顶与超时的说明）打在 mutate.sh 开跑时的 stderr 上，工作进程里起的也一样，不混进判结局的输出：
#   cargo 的 stdout 与 stderr 在 scope 里并成一路（命令前面包一层 bash -c 'exec "$@" 2>&1'）交回判结局，包装的 stderr 另走。
#   包了这一层，峰值表的键照旧是「cargo test --release --bin <名>[ --features …]」：用 RUN_WITH_MEMORY_CAP_KEY 给成与包之前逐字相同。
# 只供自证（自证起的子进程里才认；平常跑时有一个非空就退 2 拒跑、一次 cargo 都不调，--selftest 本身不拒）：
#   MUTATE_BREAK（下面那张表）；MUTATE_SELFTEST_MEMORY_CAP_RUNNER=<脚本> 换掉内存包装（自证拿假包装演 252、254 两种结局）；
#   原样传给包装的 RUN_WITH_MEMORY_CAP_BREAK、RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE、RUN_WITH_MEMORY_CAP_SELFTEST_TARGET（包装文件头说这几个只给它的自证用）。
#   「自证起的子进程」的标记：MUTATE_SELFTEST_MARK=<文件>，那份文件在、内容是 SELFTEST_MARK_CONTENT 那一行；只有 run_selftest 设它，读完就清，不传给包装与 cargo。
# 弄坏开关（几个用逗号连；自证里每次 run_case 起的里层 mutate.sh 都带着调用方设的值）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 run_selftest 里 run_case 的标签，括号里是那几条「✗ 自检：」的开头）：
#   MUTATE_BREAK=memoryascaught  撞顶记成抓到
#       every-outcome（「五种结局的表输出里应当有」缺「🧱 [撞内存]」与「计数：内存撞顶 1 条」、「撞了内存上限的那一条不许记成抓到」）、
#       caught-and-memory（「只有一条抓到、一条内存撞顶的表应当判失败退 1」）、queued-admission（「包装每次放行前停 5 秒时……输出里应当有」缺「🧱 [撞内存]」）
#   MUTATE_BREAK=memorynotfail   撞顶不让整轮判失败
#       caught-and-memory（同上）
#   MUTATE_BREAK=droprow         工作进程把表里第 2 条的判定丢掉不写
#       every-outcome、caught-and-memory、parallel、queued-admission（都退 8：「应当整轮判失败退 1」「照样让整轮退 1」那几条）、daily-parallel（退 8：「四条应当都判抓到、整轮退 0」）；
#       lost-result 本来就带着它跑，照样退 8
#   MUTATE_BREAK=countblind      父进程不数派出去与收回来的条数（只读在的那几份）
#       lost-result（「丢了一份判定时应当退 8」）
#   MUTATE_BREAK=sharedcopy      所有工作进程挤在第 1 份副本与第 1 个编译目录上（并行之前要防的那种并发改同一份源码）
#       parallel（「MUTATE_JOBS=4 时变异应当落在至少 2 份副本上」，核数 ≥ 4 才判这一条）；几个工作进程并发改同一份源码，
#       every-outcome、caught-and-memory、daily-parallel 与「stdout 应当逐字相同」可能跟着红，红不红不定
#   MUTATE_BREAK=nofeatures      不读 required-features、cargo test 一律不带 --features（改前的跑法）
#       required-features（「挂了 required-features 的 bin 应当每次带」：基线退 101、整轮退 2）
#   MUTATE_BREAK=outertimeout    限时套回包装外面：timeout <秒> bash run-with-memory-cap.sh …，按退出码 124 判超时（改前的跑法）
#       queued-admission（「包装每次放行前停 5 秒时……」：抓到、没红、无效、撞内存那几条都被记成「⏱ 超时」）；
#       别的例里包装自己慢过限时（判超时的那几例 5 秒，别的 25 秒）的那几条也会被记成超时，红不红不定
#   MUTATE_BREAK=swallowwrapper  包装的 stderr 并进判结局的输出（改前的跑法：排队时每 60 秒那一行看不见）
#       every-outcome（「包装自己的话应当打在 mutate.sh 的 stderr 上」）
#   MUTATE_BREAK=queueasinvalid  包装退 252 不单列，落进「编译失败、无效」那一支（改前的判法）
#       queue-full（「包装退 252 的那一条应当记「⏳ [抓到] 内存包装排不上」……整轮退 1」：记成「⏭  [抓到]」无效、退 0）
#   MUTATE_BREAK=squeezedascrashed  包装退 254 不单列，落进「测试进程没跑完」那一支（改前的判法）
#       squeezed（「包装退 254 的那一条应当记「🚧 [抓到] 被内存总上限挤掉」……整轮退 1」：记成「💥 [抓到]」没跑完、退 0）
#   MUTATE_BREAK=gpumemoryascaught  包装退 245 不单列，照输出往下判（改前的判法：被显存不够弄红的测试记成抓到）
#       gpu-memory（「包装退 245 的那一条应当记「🎴 [抓到] 显存不够」……整轮退 1」：记成「✅ [抓到]」、退 0）
#   MUTATE_BREAK=honorbreakoutsideselftest  平常跑时也认只供自证的变量，不拒（改前的跑法）
#       outside-selftest-<变量名> 五格（「平常跑时带了只供自证的变量 <变量名> 应当退 2」：五个变量各一次，都照跑、退 0）
#   MUTATE_BREAK=noparallelreason  平常跑不给自己起的包装设并行理由（改前的跑法：只有自证替它设）
#       daily-parallel（「会话里平常跑、MUTATE_JOBS=4……四条应当都判抓到、整轮退 0」与「每次包装应当带各自的并行理由」：基线就被假包装拒、退 2）；
#       在会话里跑（有 CLAUDE_CODE_SESSION_ID）时，经真包装、开几个工作进程的那几例也跟着红：every-outcome、caught-and-memory、parallel、lost-result
#       （真包装照「日常一条」拒同一例里第二条同时起的基线，退 2）；不在会话里跑的不红
#   MUTATE_BREAK=overridecallerreason  调用方设了的并行理由也换成自己的
#       daily-caller-reason（「调用方设了的并行理由应当原样传给包装」）、daily-baseline-refused（调用方那句不够 8 个字的被换掉，基线照跑、退 0）
#   MUTATE_BREAK=baselinequeueasred  基线退 252 不单列，报成「基线就是红的」（改前的判法）
#       daily-baseline-refused（「基线排不上应当退 2、报「基线排不上」」）
#   MUTATE_BREAK=baselinesqueezedasred  基线退 254 不单列，报成「基线就是红的」、退 2（改前的判法）
#       baseline-squeezed（「基线被总上限挤掉应当原样退 254、报「基线被内存总上限挤掉」」）
#   MUTATE_BREAK=baselinelogdropped  基线输出丢进 /dev/null，红了只报一句「基线就是红的」（改前的跑法）
#       baseline-red（「基线就是红的时应当退 2、点出红的测试 tests::adds」）
#   MUTATE_BREAK=bareusage  参数不够三个时不先判，set -u 报 unbound variable 退 1（改前的样子）
#       usage（「参数不够三个时应当退 2、打用法与一条例子」）
#   MUTATE_BREAK=exportparallelreason  并行理由 export 给 mutate.sh 之后起的每一条命令（--check 也看得见），不只设在自己起的那几次包装上
#       daily-parallel（「--check 那一次不该带并行理由」）
#   MUTATE_BREAK=ownnameparse  红了哪几个回到自己的 sed / awk（`test ` 与 ` ... FAILED` 之间整段算名字，改前的取法），不经 lib_libtest_result_lines.py
#       should-panic（「#[should_panic] 的用例红了应当记「✅ [抓到] 红：tests::answer_panics」……」：名字带上了「 - should panic」）、
#       baseline-red-should-panic（「基线里 should panic 的用例红了……」）
#   LIBTEST_RESULT_LINES_BREAK=should-panic-in-name  原样传给 lib_libtest_result_lines.py（它文件头）：「 - should panic」当成名字的一部分
#       should-panic、baseline-red-should-panic（同上）
#   RUN_WITH_MEMORY_CAP_BREAK=nocap  原样传给 run-with-memory-cap.sh：不进 scope、不设上限也不限时直接跑（改前的跑法）
#       every-outcome（撞内存、超时那两条跑完了、记成没红）、unavailable（「D-Bus 连不上时应当退 7」：照跑了）、baseline-memory（「基线就撞上限时应当退 2」：基线没撞顶）、
#       queued-admission（同 every-outcome；不进 scope 也就不排队，HANG 那一条跑满 30 秒）
#   MUTATE_BREAK=compileassuccess  只编不跑时不看 cargo check 的退出码，每条都记编得过
#       compile-only（「只编不跑：三条的表应当……整轮退 1」：编不过那一条记成「✅ [编不过] 编得过」、退 0）
#   MUTATE_BREAK=compilenontest  只编不跑改回 `cargo check --release --bin <名>`（不带 cfg(test) 那一份，派发时写的跑法）
#       compile-only（「只编不跑：三条的表应当整行打出「✅ [测试外死码] 编得过」」与「只编不跑：cargo 应当调 4 次……参数逐字是「check --profile test --bin fake-bench」」）
#   MUTATE_BREAK=libraryunread  不读按条判定库（改前的样子：每趟整张现判）
#       unit-library（「同一条第二趟应当复用……」）、unit-library-compile（「只编不跑第二趟应当复用……」）、
#       unit-library-callee（「改了被改坏函数调到的函数之后那一条应当现判……stderr 写库里没有这把键」：现判的原因写成了不读库）、
#       reuse-three-questions（「复用三问 reuse-three-questions」：①②③ 全红）、reuse-sentinel（「复用哨兵 reuse-sentinel」：照常那一趟就红「复用失灵」退 1）；
#       复用哨兵抽中的那几例（caught-only 第二趟这一类）另打「✗ 复用失灵」
#   MUTATE_BREAK=calleesignored  键里包住原文的那个函数只取它自己的指纹、不收它调到的单元（依赖版本行也只看它自己）
#       unit-library-callee（「改了被改坏函数调到的函数之后那一条应当现判……」：helper 改了照样复用、cargo 一次都不起）、
#       reuse-three-questions（「复用三问 reuse-three-questions」：③ 红，lonely_helper 改了「改孤函数」照样复用）；
#       unit-library-dependency（「依赖版本行：头一趟库里那一条应当逐字只记 extdep……」：adds 自己不点 extdep，记成空的）
#   MUTATE_BREAK=dependenciesbycrate  依赖版本行按整包收：单元所在 crate 清单里的外部依赖不看点没点名全收（改前的收法）
#       unit-library-dependency（「依赖版本行：头一趟库里那一条应当逐字只记 extdep……」多出 aliased、other、「……改了闭包没点名的外部包 other 的版本之后那一条应当照样复用……」现判了、
#       「……helper 改点 alias_dep:: 之后……逐字只记 aliased 一行」多出 extdep、inner、other）
#   MUTATE_BREAK=includesunlinked  副本根下不链 include_str! / include_bytes! 读的 scripts/experiments/ 之外的仓内文件（改前的样子）
#       include-outside（「装置 include_str 读仓根下别处的文件……应当链上、基线编得过」：基线编不过、退 2）、
#       include-unrecognized（「认不出的 include_str! 写法……应当退 2、逐行点名、一次 cargo 都不起」：不判就起了 cargo）
#   REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库 scripts/gate/lib_reuse_checks.py 的开关，原样传下去）  哨兵被现判不红
#       reuse-sentinel（「复用哨兵 reuse-sentinel」：不读库那一趟退 0）
#   上面的 droprow 与 countblind 在只编不跑那一支同样起作用：droprow 让 compile-only 退 8（「只编不跑：三条的表……应当整轮退 1」），
#       compile-only-lost-result 本来就带着 droprow 跑，照样退 8；countblind 让 compile-only-lost-result 静默退 0（「只编不跑：丢了一份判定时应当退 8」）；
#       compile-only-lost-result-unchecked 带着 droprow,countblind 跑，静默退 0、只报 2 条
#   MUTATE_BREAK=no-cap-lints  起 cargo 时不加 --cap-lints warn，RUSTFLAGS 与 CARGO_ENCODED_RUSTFLAGS 都照调用方的原样（改前的跑法：装置的 deny 警告让留死代码的替换文编不过）
#       cap-lints、cap-lints-existing、encoded-empty、encoded-other（「deny 警告：平常跑……留死代码那一条应当判抓到、整轮退 0」：记成「⏭  [留死代码]」无效）、
#       compile-only-cap-lints、compile-only-encoded-empty（「deny 警告：只编不跑……应当判编得过、整轮退 0」：编不过、退 1），
#       六例都另红「deny 警告：……每个 cargo 收到的 RUSTFLAGS 应当逐字是」与「……stderr 应当报」，encoded-empty、encoded-other、compile-only-encoded-empty 三例再红「……收到的 CARGO_ENCODED_RUSTFLAGS 应当逐字是」
#   MUTATE_BREAK=anchorstopsall  锚点失效一条就派活之前退 3、一条变异都不跑（改前的跑法）
#       stale-anchor（「锚点失效的那一条应当记「⚓ 锚点失效」……」：退 3、一次 cargo 都没调、stdout 是空的）、
#       compile-only-stale-anchor（「只编不跑：锚点失效的那一条应当记「⚓ 锚点失效」……」：退 3、stdout 是空的）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SELF_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
MEMORY_CAP_HIT_EXIT=250          # run-with-memory-cap.sh：撞了上限
MEMORY_CAP_UNAVAILABLE_EXIT=251  # run-with-memory-cap.sh：带上限的 scope 起不来
MEMORY_ADMISSION_REFUSED_EXIT=252  # run-with-memory-cap.sh：内存不够排不上，命令一行都没跑
TIME_LIMIT_HIT_EXIT=253          # run-with-memory-cap.sh：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT
SLICE_TOTAL_HIT_EXIT=254         # run-with-memory-cap.sh：被总上限挤掉，结果不算数
GPU_MEMORY_HIT_EXIT=245          # run-with-memory-cap.sh：显存不够（命令起止之间内核报了 NV_ERR_NO_MEMORY），结果不算数
OUTER_TIMEOUT_EXIT=124           # timeout(1) 到点：只在弄坏开关 outertimeout 下出现
MEMORY_MAX="${MUTATE_MEMORY_MAX:-16G}"
ROW_TIME_LIMIT="${MUTATE_TIMEOUT:-120}"   # 每条变异的限时（秒），见文件头「每条变异限时」一段
BROKEN_JUDGEMENT="${MUTATE_BREAK:-}"
CALLER_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-}"   # 调用方设了的并行理由原样传、不覆盖（文件头「并行」那一段）
MAXIMUM_WORKERS_BY_PROCESSOR=16  # 核数那一项的上限，与 checker-tier-crates-mutation-replay 的 worker_count_for 相同
COLLECTION_POLL_SECONDS=0.5      # 父进程隔这么久看一次哪几条判完了，按表序接着往 stdout 打

is_broken() { [[ ",$BROKEN_JUDGEMENT," == *",$1,"* ]]; }

# 只供自证的变量（文件头「只供自证」）：判是不是自证起的子进程，标记读完就清；平常跑时有一个非空就拒（在 --selftest 那一支之后判）
SELFTEST_MARK_CONTENT="mutate.sh --selftest 起的子进程"
TEST_ONLY_VARIABLES=(MUTATE_BREAK MUTATE_SELFTEST_MEMORY_CAP_RUNNER RUN_WITH_MEMORY_CAP_BREAK RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE RUN_WITH_MEMORY_CAP_SELFTEST_TARGET)
running_under_selftest=0
if [[ -n "${MUTATE_SELFTEST_MARK:-}" && -f "${MUTATE_SELFTEST_MARK:-}" ]] \
   && [[ "$(cat -- "$MUTATE_SELFTEST_MARK")" == "$SELFTEST_MARK_CONTENT" ]]; then
  running_under_selftest=1
fi
unset MUTATE_SELFTEST_MARK
MEMORY_CAP_RUNNER="$(dirname "$SELF_PATH")/../hosts/run-with-memory-cap.sh"
LIBTEST_RESULT_LINES="$(dirname "$SELF_PATH")/../gate/lib_libtest_result_lines.py"   # 红了哪几个的解析（文件头「红了哪几个」那两行）
if ((running_under_selftest)) && [[ -n "${MUTATE_SELFTEST_MEMORY_CAP_RUNNER:-}" ]]; then
  MEMORY_CAP_RUNNER="$MUTATE_SELFTEST_MEMORY_CAP_RUNNER"   # 自证拿假包装演 252、254（文件头「只供自证」）
fi

# ── 自证：假的 cargo 按被测源码里写的 BEHAVIOR 演一种结局，不编译任何东西 ──
run_selftest() {
  local scratch failures=0 checked=0 output status
  # 每条变异的限时（秒）：要判超时的那几例（表里有 HANG 那一条的 every-outcome、serial、parallel、queued-admission）给 selftest_time_limit，
  # 由 run_case 的调用方显式带 MUTATE_TIMEOUT；queued-admission 那一例让包装每次放行前停同样的秒数，限时套在包装外面（outertimeout）时那几条必然先到点。
  # 别的例不判超时，给 selftest_roomy_time_limit：机器忙的时候（几份自证同时起、负载几十）起 scope 慢过几秒，短限时会把它们记成超时或 scope 起不来，
  # 那不是被测的东西红了。HANG 那一条睡 30 秒，两个限时都要比它短
  local selftest_time_limit=5 selftest_roomy_time_limit=25
  # 自证里的 scope 都挂进这一趟自己的 slice（名字不带 -：- 是 systemd 的层级），总上限 2G：不与别的会话、别的自证挤主仓那一个 slice 的总上限与排队；收尾 stop 加 revert 掉
  local selftest_slice="singlefs_mutate_selftest_$$.slice"
  # 自证里的包装：每例自己的锁、账与峰值表，与主仓的峰值表和别的会话的账互不读写。
  # 峰值表从空的起，基线记下的峰值只有几 M，之后给 64M 就超过它的 2 倍：撞内存那一条要的正是 64M，理由写在这里。
  # 几个工作进程在同一例里同时起包装：同一个会话里起的「日常一条」靠 mutate.sh 自己给每次包装设的并行理由放行，这里不替它设（设了就把它盖住）；
  # 排队等得下就等，不因为调用方是不是会话而改判。
  local -a selftest_wrapper_environment=(
    "RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON=自证里假 cargo 一律给 64M 上限，撞内存那一条就是要撞它"
    "RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600"
    "RUN_WITH_MEMORY_CAP_SLICE=$selftest_slice" "RUN_WITH_MEMORY_CAP_SLICE_TOTAL=2G"
  )
  local case_script="$SELF_PATH"   # run_case 跑哪一份 mutate.sh：会话里平常跑并行那一组换成自己的副本
  local -a case_leading_arguments=()   # run_case 放在 bin 名之前的参数：只编不跑那几例是 --compile-only
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/mutate-selftest-XXXXXX")"
  # 自证起的子进程的标记：只有这里设（文件头「只供自证」）；子进程读完就清
  printf '%s\n' "$SELFTEST_MARK_CONTENT" >"$scratch/mark"
  export MUTATE_SELFTEST_MARK="$scratch/mark"
  # 假包装：施加了变异的那几次（被测源码里的 BEHAVIOR 不是 PASS）照 FAKE_WRAPPER_OUTCOME 演包装的一种结局，
  # 别的（--check、基线、还原之后）原样交给真包装 FAKE_WRAPPER_REAL
  cat >"$scratch/fake-memory-cap.sh" <<'FAKE_WRAPPER'
#!/usr/bin/env bash
behavior="$(sed -n 's/.*BEHAVIOR: \([A-Z]*\).*/\1/p' bench/src/bin/fake_bench.rs 2>/dev/null | head -1)"
if [[ "${1:-}" == --check ]]; then exec bash "$FAKE_WRAPPER_REAL" "$@"; fi
# baseline-squeezed：没施加变异的那几次（基线）就被总上限挤掉
if [[ "$behavior" == PASS && "${FAKE_WRAPPER_OUTCOME:-}" == baseline-squeezed ]]; then
  echo "run-with-memory-cap: 被总上限挤掉（假包装，基线），退 254" >&2; exit 254
fi
if [[ "$behavior" == PASS ]]; then exec bash "$FAKE_WRAPPER_REAL" "$@"; fi
case "${FAKE_WRAPPER_OUTCOME:-}" in
  queue-full) echo "run-with-memory-cap: 等满还放不下（假包装），退 252" >&2; exit 252 ;;
  squeezed) printf 'running 2 tests\nerror: test failed, to rerun pass `--bin fake-bench`\n\nCaused by:\n  process didn'"'"'t exit successfully: `target/release/deps/fake_bench-0123456789abcdef` (signal: 9, SIGKILL: kill)\n'
            echo "run-with-memory-cap: 被总上限挤掉（假包装），退 254" >&2; exit 254 ;;
  gpu-memory) printf 'running 2 tests\ntest tests::fake_bench_case ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\nerror: test failed, to rerun pass `--bin fake-bench`\n'
              echo "run-with-memory-cap: 显存不够（假包装），退 245" >&2; exit 245 ;;
  *) echo "假包装认不出 FAKE_WRAPPER_OUTCOME「${FAKE_WRAPPER_OUTCOME:-}」" >&2; exit 2 ;;
esac
FAKE_WRAPPER
  chmod +x "$scratch/fake-memory-cap.sh"
  # 假包装（会话里平常跑并行那一组）：照 run-with-memory-cap.sh 文件头「日常一条」判——有 CLAUDE_CODE_SESSION_ID、不带 SINGLEFS_HEAVY_TESTS 的，
  # 同一个会话在账上（FAKE_DAILY_LEDGER）已有一条活着的而这一条的理由不够 8 个字，退 252、命令不跑；放行的记账、不进 scope 直接跑、跑完销账。
  # 每次被调往 FAKE_DAILY_LOG 记一行「check / admitted / refused|理由」
  cat >"$scratch/fake-daily-memory-cap.sh" <<'FAKE_DAILY_WRAPPER'
#!/usr/bin/env bash
reason="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-}"
if [[ "${1:-}" == --check ]]; then printf 'check|%s\n' "$reason" >>"$FAKE_DAILY_LOG"; exit 0; fi
shift
exec {lock_fd}>>"$FAKE_DAILY_LEDGER/lock"
flock "$lock_fd"
if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" && -n "${CLAUDE_CODE_SESSION_ID:-}" ]]; then
  running=0
  for entry in "$FAKE_DAILY_LEDGER"/*.entry; do
    [[ -f "$entry" ]] || continue
    read -r entry_session entry_pid <"$entry"
    if [[ "$entry_session" == "$CLAUDE_CODE_SESSION_ID" ]] && kill -0 "$entry_pid" 2>/dev/null; then running=$((running + 1)); fi
  done
  if ((running > 0 && ${#reason} < 8)); then
    printf 'refused|%s\n' "$reason" >>"$FAKE_DAILY_LOG"
    echo "run-with-memory-cap（假包装）: 这个会话已经有 $running 条日常的在跑，同一个会话同时只许一条，命令没跑，退 252" >&2
    exit 252
  fi
fi
printf '%s %s\n' "${CLAUDE_CODE_SESSION_ID:-}" "$$" >"$FAKE_DAILY_LEDGER/$$.entry"
printf 'admitted|%s\n' "$reason" >>"$FAKE_DAILY_LOG"
flock -u "$lock_fd"
exec {lock_fd}>&-
env -u RUN_WITH_MEMORY_CAP_PARALLEL_REASON -u RUN_WITH_MEMORY_CAP_TIME_LIMIT -u RUN_WITH_MEMORY_CAP_KEY "$@"
status=$?
rm -f -- "$FAKE_DAILY_LEDGER/$$.entry"
exit "$status"
FAKE_DAILY_WRAPPER

  # 建一例自己的目录 $scratch/<标签>/：假 cargo、假 workspace（scripts/experiments/）、几张变异表、包装的锁与账。$1 标签；
  # $2 被测源码里写的 BEHAVIOR（默认 PASS）；$3 bin 那一节追加的一行（默认不加，required-features 那一例给它挂 feature）
  prepare_case() {
    local case_directory="$scratch/$1" behavior="${2:-PASS}" bin_extra_line="${3:-}"
    mkdir -p "$case_directory/bin" "$case_directory/scripts/experiments/bench/src/bin" "$case_directory/scripts/mutation/tables" "$case_directory/memory-cap-state"
    cat >"$case_directory/bin/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
# 假 cargo：只认 mutate.sh 调的 `cargo test --release --bin <名字>`，按被测源码里的 BEHAVIOR 演一种结局；
# 每次被调记一行「当前目录|CARGO_TARGET_DIR|BEHAVIOR|参数」（并行的自证拿前两段数各用了几份副本、几个编译目录）
# 按条入库算工具链行时问的 `cargo -V` 只打一行版本、不记（FAKE_CARGO_VERSION 设了打它：工具链变了那一例）
if [[ "${1:-}" == -V || "${1:-}" == --version ]]; then echo "${FAKE_CARGO_VERSION:-cargo 0.0.0 (mutate.sh selftest)}"; exit 0; fi
behavior="$(sed -n 's/.*BEHAVIOR: \([A-Z]*\).*/\1/p' bench/src/bin/fake_bench.rs | head -1)"
echo "$PWD|${CARGO_TARGET_DIR:-}|$behavior|$*" >>"$FAKE_CARGO_LOG"
# 另记一行「BEHAVIOR|子命令|收到的 RUSTFLAGS」（没设记「（没设）」），deny 警告那几例拿它核每个 cargo 收到的 RUSTFLAGS
echo "$behavior|${1:-}|${RUSTFLAGS-（没设）}" >>"$FAKE_CARGO_LOG.rustflags"
# 同样另记收到的 CARGO_ENCODED_RUSTFLAGS（原样，含 0x1f；没设记「（没设）」，设了空串记空）
echo "$behavior|${1:-}|${CARGO_ENCODED_RUSTFLAGS-（没设）}" >>"$FAKE_CARGO_LOG.encoded"
# 照真 cargo：CARGO_ENCODED_RUSTFLAGS 设了（空串也算）就只读它（各项以 0x1f 分隔）、不读 RUSTFLAGS
if [[ -v CARGO_ENCODED_RUSTFLAGS ]]; then effective_flags="${CARGO_ENCODED_RUSTFLAGS//$'\x1f'/ }"; else effective_flags="${RUSTFLAGS:-}"; fi
# 照 [lints.rust] warnings = "deny"：DEADCODE（替换文留下死代码）在 cargo 实读的 flags 里没有 --cap-lints warn 时编不过，check 与 test 一样
if [[ "$behavior" == DEADCODE && " $effective_flags " != *" --cap-lints warn "* ]]; then
  printf 'error: function `left_unused` is never used\nerror: could not compile `bench` (bin "fake-bench" test) due to 1 previous error\n'; exit 101
fi
# 照真 cargo：源码里 include_str!("<路径>") 读不到（路径相对那份源文件）就编不过，check 与 test 一样（include-outside 那一例：副本里要链上仓根下别处的文件）
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
# 只编不跑调的 cargo check：test 档要关 debug_assertions（照 cargo test --release 的 cfg）；BROKEN 编不过，
# DEADOUTSIDETESTS 只在不带 cfg(test) 时编不过（只在测试外用不到的函数报 dead_code），别的都编得过
if [[ "${1:-}" == check ]]; then
  if [[ " $* " == *" --profile test "* && "${CARGO_PROFILE_TEST_DEBUG_ASSERTIONS:-}" != false ]]; then
    printf 'error: 假 cargo：test 档没关 debug_assertions（CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=false），cfg 与 cargo test --release 不同\n'; exit 101
  fi
  case "$behavior" in
    BROKEN) printf 'error[E0425]: cannot find value `missing` in this scope\nerror: could not compile `bench`\n'; exit 101 ;;
    DEADOUTSIDETESTS)
      if [[ " $* " == *" --profile test "* ]]; then printf '    Finished `test` profile\n'; exit 0; fi
      printf 'error: function `only_tests_use_it` is never used\nerror: could not compile `bench`\n'; exit 101 ;;
    *) printf '    Finished\n'; exit 0 ;;
  esac
fi
case "$behavior" in
  PASS) passing_output; exit 0 ;;
  RED|DEADCODE) printf 'running 2 tests\ntest tests::adds ... FAILED\ntest tests::subtracts ... ok\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
  PANICS) printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::answer_panics - should panic ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
  # 每次拿 16 MiB 写满、拿到 512 MiB 就停：上限 64M 时半路被杀；没有上限时分配完照常通过（改前被记成「没红」），吃不满机器
  HOG) python3 -c 'held = [b"\x01" * (16 * 1024 * 1024) for _ in range(32)]' || exit 101; passing_output; exit 0 ;;
  HANG) sleep 30; passing_output; exit 0 ;;
  BROKEN) printf 'error[E0425]: cannot find value `missing` in this scope\nerror: could not compile `bench`\n'; exit 101 ;;
  *) echo "假 cargo 认不出 BEHAVIOR「$behavior」" >&2; exit 2 ;;
esac
FAKE_CARGO
    chmod +x "$case_directory/bin/cargo"
    printf '[workspace]\nmembers = ["bench"]\n' >"$case_directory/scripts/experiments/Cargo.toml"
    printf '[package]\nname = "bench"\n\n[[bin]]\nname = "fake-bench"\npath = "src/bin/fake_bench.rs"\n' >"$case_directory/scripts/experiments/bench/Cargo.toml"
    [[ -n "$bin_extra_line" ]] && printf '%s\n' "$bin_extra_line" >>"$case_directory/scripts/experiments/bench/Cargo.toml"
    printf '// 注释行\nfn main() {} // BEHAVIOR: %s\n' "$behavior" >"$case_directory/scripts/experiments/bench/src/bin/fake_bench.rs"
    local tables="$case_directory/scripts/mutation/tables"
    printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n没红\t// 注释行\t// 改过的注释行\n撞内存\tBEHAVIOR: PASS\tBEHAVIOR: HOG\n超时\tBEHAVIOR: PASS\tBEHAVIOR: HANG\n无效\tBEHAVIOR: PASS\tBEHAVIOR: BROKEN\n' \
      >"$tables/every-outcome.tsv"
    printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n撞内存\tBEHAVIOR: PASS\tBEHAVIOR: HOG\n' >"$tables/caught-and-memory.tsv"
    printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n' >"$tables/caught-only.tsv"
    printf '抓到一\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到二\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到三\tBEHAVIOR: PASS\tBEHAVIOR: RED\n' \
      >"$tables/three-caught.tsv"
    printf '抓到一\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到二\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到三\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到四\tBEHAVIOR: PASS\tBEHAVIOR: RED\n' \
      >"$tables/four-caught.tsv"
    printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n锚点腐化\tBEHAVIOR: GONE\tBEHAVIOR: RED\n' >"$tables/stale-anchor.tsv"
    printf '改注释\t// 注释行\t// 改过的注释行\n' >"$tables/comment-only.tsv"
    printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: PANICS\n' >"$tables/should-panic.tsv"
    printf '编得过\tBEHAVIOR: PASS\tBEHAVIOR: RED\n编不过\tBEHAVIOR: PASS\tBEHAVIOR: BROKEN\n测试外死码\tBEHAVIOR: PASS\tBEHAVIOR: DEADOUTSIDETESTS\n' \
      >"$tables/compile-only.tsv"
    printf '留死代码\tBEHAVIOR: PASS\tBEHAVIOR: DEADCODE\n' >"$tables/dead-code.tsv"
  }

  # 在 prepare_case 建好的那一例目录里跑一次 mutate.sh：$1 标签、$2 变异表，其余是额外的环境变量；
  # stdout 落 $scratch/<标签>.stdout、stderr 落 .stderr、两样接起来落 .out，退出码落 .rc，假 cargo 的调用记录落 .cargo
  run_case() {
    local label="$1" table="$2" case_directory="$scratch/$1"
    shift 2
    : >"$scratch/$label.cargo"
    if (cd "$case_directory/scripts/experiments" && env -u CARGO_ENCODED_RUSTFLAGS PATH="$case_directory/bin:$PATH" FAKE_CARGO_LOG="$scratch/$label.cargo" MUTATE_MEMORY_MAX=64M \
          MUTATE_TIMEOUT="$selftest_roomy_time_limit" MUTATE_TARGET_DIR="$case_directory/target" MUTATE_LIBRARY_DIRECTORY="$case_directory/library" \
          RUN_WITH_MEMORY_CAP_STATE_DIR="$case_directory/memory-cap-state" RUN_WITH_MEMORY_CAP_PEAKS="$case_directory/memory-peaks.tsv" \
          "${selftest_wrapper_environment[@]}" "$@" bash "$case_script" ${case_leading_arguments[@]+"${case_leading_arguments[@]}"} fake-bench bench/src/bin/fake_bench.rs "$table" \
          >"$scratch/$label.stdout" 2>"$scratch/$label.stderr"); then
      echo 0 >"$scratch/$label.rc"
    else
      echo $? >"$scratch/$label.rc"
    fi
    cat "$scratch/$label.stderr" "$scratch/$label.stdout" >"$scratch/$label.out"
  }
  fail() { echo "  ✗ 自检：$1"; failures=$((failures + 1)); }   # gate-lint:detail

  prepare_case every-outcome
  run_case every-outcome ../mutation/tables/every-outcome.tsv MUTATE_TIMEOUT="$selftest_time_limit"
  output="$(cat "$scratch/every-outcome.out")"; status="$(cat "$scratch/every-outcome.rc")"
  checked=$((checked + 1))
  [[ "$status" == 1 ]] || fail "五种结局各一条的表应当整轮判失败退 1，实际退 $status：$output"
  local wanted
  for wanted in "✅ [抓到] 红：tests::adds" "❌ [没红]" "⏭  [无效]" "⏱  [超时]" "🧱 [撞内存] 撞了内存上限 64M（内存撞顶）" \
                "计数：内存撞顶 1 条（上限 64M）、超时 1 条" "已还原，基线仍全绿"; do
    checked=$((checked + 1))
    grep -qF -- "$wanted" <<<"$output" || fail "五种结局的表输出里应当有「$wanted」，实际：$output"
  done
  checked=$((checked + 1))
  if grep -E '^(✅|❌|💥|⏭|⚠️) +\[撞内存\]' <<<"$output" >/dev/null; then
    fail "撞了内存上限的那一条不许记成抓到、没红、没跑完或无效：$(grep -F '[撞内存]' <<<"$output")"
  fi
  # 包装自己的话（排队时每 60 秒那一行与撞顶的说明走同一个 stderr）：工作进程里起的包装，它的 stderr 也要打到 mutate.sh 的 stderr 上，
  # 不并进判结局的输出（并进去就看不见，改前就是这样）。撞内存那一条的包装必然说一句「撞了内存上限」
  checked=$((checked + 1))
  grep -qF "run-with-memory-cap: 撞了内存上限 64M" "$scratch/every-outcome.stderr" \
    || fail "包装自己的话应当打在 mutate.sh 的 stderr 上（撞内存那一条的包装说的「run-with-memory-cap: 撞了内存上限 64M」），实际 stderr：$(cat "$scratch/every-outcome.stderr")"

  prepare_case caught-and-memory
  run_case caught-and-memory ../mutation/tables/caught-and-memory.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/caught-and-memory.rc")"
  [[ "$status" == 1 ]] || fail "只有一条抓到、一条内存撞顶的表应当判失败退 1（撞顶与超时同类），实际退 $status：$(cat "$scratch/caught-and-memory.out")"

  # 没挂 required-features 的 bin、上限之内正常的变异：照旧判抓到、整轮退 0，每次 cargo 的参数与改前逐字相同（这一例自己的调用记录）
  prepare_case caught-only
  run_case caught-only ../mutation/tables/caught-only.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/caught-only.rc")"
  output="$(cat "$scratch/caught-only.out")"
  [[ "$status" == 0 ]] && grep -qF "计数：内存撞顶 0 条（上限 64M）、超时 0 条" <<<"$output" \
    || fail "上限之内正常的变异照旧判抓到、整轮退 0 并报「内存撞顶 0 条」，实际退 $status：$output"
  local unexpected_arguments
  checked=$((checked + 1))
  unexpected_arguments="$(cut -d'|' -f4- "$scratch/caught-only.cargo" | grep -vxF 'test --release --bin fake-bench')"
  if [[ ! -s "$scratch/caught-only.cargo" || -n "$unexpected_arguments" ]]; then
    fail "没挂 required-features 的 bin，cargo 的参数应当逐字是「test --release --bin fake-bench」，实际调了 $(wc -l <"$scratch/caught-only.cargo") 次、对不上的：${unexpected_arguments:-（一次都没调）}"
  fi

  # 带上限的 scope 起不来：退 7，假 cargo 一次都没被调（连基线都没跑，不退回无上限）
  prepare_case unavailable
  run_case unavailable ../mutation/tables/caught-only.tsv DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent
  checked=$((checked + 1))
  status="$(cat "$scratch/unavailable.rc")"
  if [[ "$status" != 7 || -s "$scratch/unavailable.cargo" ]] || ! grep -q '起不来' "$scratch/unavailable.out"; then
    fail "D-Bus 连不上时应当退 7、报「起不来」、假 cargo 一次都不调，实际退 $status、调了 $(wc -l <"$scratch/unavailable.cargo") 次：$(cat "$scratch/unavailable.out")"
  fi

  # ── 并行 ──
  # 这台机器上 MUTATE_JOBS=4、五条的表应当开几个：min(核数的一半且至多 16、4、5)，至少 1
  local processor_count expected_workers mutated_runs distinct_copies distinct_targets distinct_pairs
  processor_count="$(nproc 2>/dev/null || echo 2)"
  expected_workers=$((processor_count / 2))
  ((expected_workers > MAXIMUM_WORKERS_BY_PROCESSOR)) && expected_workers=$MAXIMUM_WORKERS_BY_PROCESSOR
  ((expected_workers > 4)) && expected_workers=4
  ((expected_workers < 1)) && expected_workers=1
  # 同一个场景跑两次（MUTATE_JOBS=1 与 =4）再比对：各在自己的目录里跑，比的是两次的输出，不是一次读另一次留下的状态
  prepare_case serial
  run_case serial ../mutation/tables/every-outcome.tsv MUTATE_JOBS=1 MUTATE_TIMEOUT="$selftest_time_limit"
  prepare_case parallel
  run_case parallel ../mutation/tables/every-outcome.tsv MUTATE_JOBS=4 MUTATE_TIMEOUT="$selftest_time_limit"
  checked=$((checked + 1))
  cmp -s "$scratch/serial.stdout" "$scratch/parallel.stdout" \
    || fail "MUTATE_JOBS=1 与 =4 的 stdout 应当逐字相同（判定行按表序打），实际不同：$(diff "$scratch/serial.stdout" "$scratch/parallel.stdout")"
  checked=$((checked + 1))
  status="$(cat "$scratch/parallel.rc")"
  if [[ "$status" != 1 ]] || ! grep -qF "❌ [没红]" "$scratch/parallel.stdout" || ! grep -qF "✅ [抓到] 红：tests::adds" "$scratch/parallel.stdout"; then
    fail "MUTATE_JOBS=4 时没红的那一条照样让整轮退 1、抓到的那一条照样判抓到，实际退 $status：$(cat "$scratch/parallel.out")"
  fi
  checked=$((checked + 1))
  [[ "$(head -1 "$scratch/parallel.stderr")" == *"这一轮开 $expected_workers 个工作进程"* ]] \
    || fail "MUTATE_JOBS=4、五条的表，stderr 头一行应当报「这一轮开 $expected_workers 个工作进程」，实际：$(head -1 "$scratch/parallel.stderr")"
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

  # MUTATE_TIMEOUT 不是正整数秒（包装的 RUN_WITH_MEMORY_CAP_TIME_LIMIT 只认正整数秒）：开跑之前退 2，一次 cargo 都不调，
  # 不等基线跑完、第一条变异时才被包装按用法错退 251、报成「scope 起不来」
  prepare_case bad-timeout
  run_case bad-timeout ../mutation/tables/caught-only.tsv MUTATE_TIMEOUT=1.5
  checked=$((checked + 1))
  status="$(cat "$scratch/bad-timeout.rc")"
  if [[ "$status" != 2 || -s "$scratch/bad-timeout.cargo" ]] || ! grep -qF "MUTATE_TIMEOUT" "$scratch/bad-timeout.stderr"; then
    fail "MUTATE_TIMEOUT=1.5 应当退 2、点名 MUTATE_TIMEOUT、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/bad-timeout.cargo") 次：$(cat "$scratch/bad-timeout.out")"
  fi

  # 锚点失效按条判（文件头「锚点失效」）：两条的表，原文命中 0 次的那一条记「⚓ 锚点失效」、不入判定库，另一条照跑抓到；
  # 收尾「计数：」报锚点失效 1 条、整轮退 3。弄坏开关 anchorstopsall（改前的跑法：派活之前就退 3、一次 cargo 都不调）下这一格判错
  prepare_case stale-anchor
  run_case stale-anchor ../mutation/tables/stale-anchor.tsv MUTATE_JOBS=2
  checked=$((checked + 1))
  status="$(cat "$scratch/stale-anchor.rc")"
  if [[ "$status" != 3 || ! -s "$scratch/stale-anchor.cargo" ]] \
     || ! grep -qxF "✅ [抓到] 红：tests::adds" "$scratch/stale-anchor.stdout" \
     || ! grep -qxF "⚓ [锚点腐化] 锚点失效：原文在 bench/src/bin/fake_bench.rs 里命中 0 次（要求恰好 1 次）—— 这一条没跑，不算抓到也不算没红" "$scratch/stale-anchor.stdout" \
     || ! grep -qE '^计数：.*、锚点失效 1 条、现判 [0-9]+ 条、复用 [0-9]+ 条、拒算 [0-9]+ 条$' "$scratch/stale-anchor.stdout" || ! grep -qF "已还原，基线仍全绿" "$scratch/stale-anchor.stdout" \
     || ! grep -qF "1 条锚点失效（上面 ⚓ 那几行）" "$scratch/stale-anchor.stderr" \
     || (( $(find "$scratch/stale-anchor/library" -type f -path '*records*' 2>/dev/null | grep -c .) > 1 )); then
    fail "锚点失效的那一条应当记「⚓ 锚点失效」、不入判定库，另一条照跑抓到，计数报锚点失效 1 条、整轮退 3，实际退 $status、调了 $(grep -c . "$scratch/stale-anchor.cargo") 次 cargo、判定库记录 $(find "$scratch/stale-anchor/library" -type f -path '*records*' 2>/dev/null | grep -c .) 份：$(cat "$scratch/stale-anchor.out")"
  fi
  # 只编不跑同样按条：锚点失效那一条记「⚓」、另一条照编，计数报锚点失效 1 条、退 3
  case_leading_arguments=(--compile-only)
  prepare_case compile-only-stale-anchor
  run_case compile-only-stale-anchor ../mutation/tables/stale-anchor.tsv
  case_leading_arguments=()
  checked=$((checked + 1))
  status="$(cat "$scratch/compile-only-stale-anchor.rc")"
  if [[ "$status" != 3 ]] || ! grep -qxF "✅ [抓到] 编得过" "$scratch/compile-only-stale-anchor.stdout" \
     || ! grep -qF "⚓ [锚点腐化] 锚点失效：原文在 bench/src/bin/fake_bench.rs 里命中 0 次" "$scratch/compile-only-stale-anchor.stdout" \
     || ! grep -qE '^计数：派出去 2 条，收回 2 份判定：编得过 1 条、编不过 0 条、锚点失效 1 条、现判 [0-9]+ 条、复用 [0-9]+ 条、拒算 [0-9]+ 条$' "$scratch/compile-only-stale-anchor.stdout"; then
    fail "只编不跑：锚点失效的那一条应当记「⚓ 锚点失效」、另一条照编，计数报锚点失效 1 条、退 3，实际退 $status：$(cat "$scratch/compile-only-stale-anchor.out")"
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

  # 基线（没改的源码）就撞上限：退 2 并说是内存上限，不说成「基线就是红的」。
  # 表里的原文要在 HOG 版源码里命中得上（锚点在基线之前核，命中不上的那一条只记锚点失效、不跑），所以用只改注释的那张表
  prepare_case baseline-memory HOG
  run_case baseline-memory ../mutation/tables/comment-only.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/baseline-memory.rc")"
  if [[ "$status" != 2 ]] || ! grep -q '基线.*撞了内存上限 64M' "$scratch/baseline-memory.out"; then
    fail "基线就撞上限时应当退 2 并报「基线……撞了内存上限 64M」，实际退 $status：$(cat "$scratch/baseline-memory.out")"
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
  if (cd "$scratch/usage/scripts/experiments" && env PATH="$scratch/usage/bin:$PATH" FAKE_CARGO_LOG="$scratch/usage.cargo" \
        bash "$case_script" fake-bench >"$scratch/usage.stdout" 2>"$scratch/usage.stderr"); then
    echo 0 >"$scratch/usage.rc"
  else
    echo $? >"$scratch/usage.rc"
  fi
  checked=$((checked + 1))
  status="$(cat "$scratch/usage.rc")"
  if [[ "$status" != 2 ]] || ! grep -qF "要三个参数，给了 1 个" "$scratch/usage.stderr" \
     || ! grep -qF "→ 怎么办：在 scripts/experiments/ 目录里照这个例子跑：bash ../mutation/mutate.sh " "$scratch/usage.stderr" \
     || grep -qF "unbound variable" "$scratch/usage.stderr" || [[ -s "$scratch/usage.cargo" ]]; then
    fail "参数不够三个时应当退 2、打用法与一条例子、不报 unbound variable、一次 cargo 都不调，实际退 $status：$(cat "$scratch/usage.stderr")"
  fi

  # ── 包装排队 ──
  # 包装每次放行前多停 $selftest_time_limit 秒（RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE，拿着这一例自己的锁停），与每条变异的限时一样长：
  # 限时从命令起跑算、不算排队，五种结局照样各归各类，只有 HANG 那一条记超时；限时套在包装外面（outertimeout，改前的跑法）时排队就吃满限时，
  # 抓到、没红、无效、撞内存几条全被记成超时。一个工作进程，免得几条在同一把锁上串着停、把这一例拖长
  prepare_case queued-admission
  run_case queued-admission ../mutation/tables/every-outcome.tsv MUTATE_JOBS=1 RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE="$selftest_time_limit" \
    MUTATE_TIMEOUT="$selftest_time_limit"
  output="$(cat "$scratch/queued-admission.out")"; status="$(cat "$scratch/queued-admission.rc")"
  checked=$((checked + 1))
  [[ "$status" == 1 ]] || fail "包装每次放行前停 $selftest_time_limit 秒时，五种结局各一条的表照样整轮判失败退 1，实际退 $status：$output"
  for wanted in "✅ [抓到] 红：tests::adds" "❌ [没红]" "⏭  [无效]" "⏱  [超时]" "🧱 [撞内存] 撞了内存上限 64M（内存撞顶）" \
                "计数：内存撞顶 1 条（上限 64M）、超时 1 条" "已还原，基线仍全绿"; do
    checked=$((checked + 1))
    grep -qF -- "$wanted" <<<"$output" || fail "包装每次放行前停 $selftest_time_limit 秒时（限时不算排队），输出里应当有「$wanted」，实际：$output"
  done
  checked=$((checked + 1))
  if grep -E '^⏱ +\[' <<<"$output" | grep -vF '⏱  [超时]' >/dev/null; then
    fail "包装每次放行前停 $selftest_time_limit 秒时，只有 HANG 那一条该记超时，排队的时间不许把别的几条算成超时：$(grep -E '^⏱ +\[' <<<"$output")"
  fi

  # ── 包装的另两种结局 ──
  # 假包装在施加了变异的那一次退 252（排不上：命令没跑、输出是空的）：记「⏳ 排不上」、整轮退 1、单列计数，不记成「⏭  无效」；
  # 退 254（被总上限挤掉：输出里有 cargo 的 process didn't exit successfully）：记「🚧 被总上限挤掉」、整轮退 1、单列计数，不记成「💥 没跑完」
  local real_memory_cap_runner outcome_label wrapper_exit expected_line expected_count wrong_symbol
  real_memory_cap_runner="$(dirname "$SELF_PATH")/../hosts/run-with-memory-cap.sh"
  for outcome_label in queue-full squeezed gpu-memory; do
    case "$outcome_label" in
      queue-full) wrapper_exit=252; expected_line="⏳ [抓到] 内存包装排不上（run-with-memory-cap.sh 退 252）"
                  expected_count="排不上 1 条、被总上限挤掉 0 条"; wrong_symbol="⏭" ;;
      squeezed) wrapper_exit=254; expected_line="🚧 [抓到] 被内存总上限挤掉（run-with-memory-cap.sh 退 254"
                expected_count="排不上 0 条、被总上限挤掉 1 条"; wrong_symbol="💥" ;;
      gpu-memory) wrapper_exit=245; expected_line="🎴 [抓到] 显存不够（run-with-memory-cap.sh 退 245"
                  expected_count="被总上限挤掉 0 条、显存不够 1 条"; wrong_symbol="✅" ;;
    esac
    prepare_case "$outcome_label"
    run_case "$outcome_label" ../mutation/tables/caught-only.tsv MUTATE_SELFTEST_MEMORY_CAP_RUNNER="$scratch/fake-memory-cap.sh" \
      FAKE_WRAPPER_REAL="$real_memory_cap_runner" FAKE_WRAPPER_OUTCOME="$outcome_label"
    status="$(cat "$scratch/$outcome_label.rc")"
    checked=$((checked + 1))
    if [[ "$status" != 1 ]] || ! grep -qF -- "$expected_line" "$scratch/$outcome_label.stdout" \
       || ! grep -qF -- "$expected_count" "$scratch/$outcome_label.stdout" || grep -E "^$wrong_symbol +\[" "$scratch/$outcome_label.stdout" >/dev/null \
       || ! grep -qF "整张表重跑" "$scratch/$outcome_label.stderr"; then
      fail "包装退 $wrapper_exit 的那一条应当记「$expected_line」、计数里有「$expected_count」、不记成「$wrong_symbol」、stderr 说整张表重跑、整轮退 1，实际退 $status：$(cat "$scratch/$outcome_label.out")"
    fi
  done

  # 基线就被总上限挤掉（假包装在没施加变异的那几次退 254）：原样退 254、报「基线被内存总上限挤掉」、一条变异都不跑，不说成「基线就是红的」
  prepare_case baseline-squeezed
  run_case baseline-squeezed ../mutation/tables/caught-only.tsv MUTATE_SELFTEST_MEMORY_CAP_RUNNER="$scratch/fake-memory-cap.sh" \
    FAKE_WRAPPER_REAL="$real_memory_cap_runner" FAKE_WRAPPER_OUTCOME=baseline-squeezed
  status="$(cat "$scratch/baseline-squeezed.rc")"
  checked=$((checked + 1))
  if [[ "$status" != "$SLICE_TOTAL_HIT_EXIT" ]] || ! grep -qF "基线被内存总上限挤掉" "$scratch/baseline-squeezed.stderr" \
     || grep -qF "基线就是红的" "$scratch/baseline-squeezed.stderr" || grep -qF "[抓到]" "$scratch/baseline-squeezed.stdout"; then
    fail "基线被总上限挤掉应当原样退 $SLICE_TOTAL_HIT_EXIT、报「基线被内存总上限挤掉」、一条变异都不跑，不说成「基线就是红的」，实际退 $status：$(cat "$scratch/baseline-squeezed.out")"
  fi

  # ── 会话里平常跑并行 ──
  # 包装的「日常一条」（run-with-memory-cap.sh 文件头）：同一个会话账上已有一条日常的在跑、这一条没给至少 8 个字的理由，退 252、命令不跑。
  # 这一组不经自证的标记（平常跑），跑的是这一份 mutate.sh 的原样副本（设了弄坏开关的话，只把 BROKEN_JUDGEMENT 那一行写死成同一个值：平常跑不认 MUTATE_BREAK），
  # 放在这一例自己的目录里，同目录的 run-with-memory-cap.sh 换成上面的假「日常一条」包装。
  # 账上先放一条这个会话的、进程号是自证自己的（一直活着）：每一次都要带理由才放行，不靠几个工作进程碰巧同时起
  local repository_root test_only_variable daily_label daily_log admitted_reasons unmatched_reasons distinct_reasons check_reasons
  local caller_reason="调用方自己给的理由：派出去的两个执行员各跑各的" short_caller_reason="不够八字"
  local -a daily_environment=(MUTATE_SELFTEST_MARK= CLAUDE_CODE_SESSION_ID=mutate-selftest-session SINGLEFS_HEAVY_TESTS= RUN_WITH_MEMORY_CAP_PARALLEL_REASON=)
  for test_only_variable in "${TEST_ONLY_VARIABLES[@]}"; do
    daily_environment+=("$test_only_variable=")   # 调用方环境里设着的弄坏开关不漏进平常跑（漏进去就退 2，红的不是这一格要判的）
  done
  repository_root="$(cd "$(dirname "$SELF_PATH")/../.." && pwd)"
  # 建一例：prepare_case 那一套，加上 tooling/scripts/ 下的 mutate.sh 副本与假包装、tooling/.claude 指回仓里（preflight 从那里找）、账上那一条
  prepare_daily_case() {
    local label="$1" tooling="$scratch/$1/tooling"
    prepare_case "$label"
    mkdir -p "$tooling/scripts" "$tooling/scripts/gate" "$tooling/scripts/hosts" "$tooling/scripts/mutation" "$scratch/$label/daily-ledger"
    ln -s "$repository_root/.claude" "$tooling/.claude"
    if [[ -n "$BROKEN_JUDGEMENT" ]]; then
      BAKED="$BROKEN_JUDGEMENT" SOURCE="$SELF_PATH" DESTINATION="$tooling/scripts/mutation/mutate.sh" python3 - <<'PY' || return 1
import os, shlex, sys
old = 'BROKEN_JUDGEMENT="${MUTATE_BREAK:-}"\n'
with open(os.environ["SOURCE"], encoding="utf-8") as handle:
    text = handle.read()
if text.count(old) != 1:
    sys.stderr.write("mutate.sh 里 %r 应当恰好一行，实际 %d 行\n" % (old, text.count(old))); sys.exit(1)
with open(os.environ["DESTINATION"], "w", encoding="utf-8") as handle:
    handle.write(text.replace(old, "BROKEN_JUDGEMENT=%s\n" % shlex.quote(os.environ["BAKED"])))
PY
    else
      cp -- "$SELF_PATH" "$tooling/scripts/mutation/mutate.sh" || return 1
    fi
    cp -- "$scratch/fake-daily-memory-cap.sh" "$tooling/scripts/hosts/run-with-memory-cap.sh" || return 1
    cp -- "$LIBTEST_RESULT_LINES" "$tooling/scripts/gate/lib_libtest_result_lines.py" || return 1   # 副本取红了哪几个要它
    cp -- "$(dirname "$SELF_PATH")/lib_mutation_units.py" "$tooling/scripts/mutation/" || return 1   # 副本按条入库要它（文件头「按条入库」）
    cp -- "$(dirname "$SELF_PATH")/../gate/lib_function_units.py" "$tooling/scripts/gate/" || return 1
    printf 'mutate-selftest-session %s\n' "$$" >"$scratch/$label/daily-ledger/already-running.entry"
  }
  run_daily_case() {
    local label="$1" table="$2"
    shift 2
    case_script="$scratch/$label/tooling/scripts/mutation/mutate.sh"
    run_case "$label" "$table" "${daily_environment[@]}" FAKE_DAILY_LEDGER="$scratch/$label/daily-ledger" FAKE_DAILY_LOG="$scratch/$label.daily" "$@"
    case_script="$SELF_PATH"
    touch "$scratch/$label.daily"
  }
  for daily_label in daily-parallel daily-caller-reason daily-baseline-refused; do
    prepare_daily_case "$daily_label" || fail "建「$daily_label」那一例的副本失败（上面是原因）"
  done

  # MUTATE_JOBS=4、四条都抓得到的表：调用方没给理由，mutate.sh 自己给每次包装设，照样各条判对、整轮退 0
  run_daily_case daily-parallel ../mutation/tables/four-caught.tsv MUTATE_JOBS=4
  status="$(cat "$scratch/daily-parallel.rc")"
  daily_log="$scratch/daily-parallel.daily"
  checked=$((checked + 1))
  if [[ "$status" != 0 || "$(grep -c '^✅ \[抓到[一二三四]\] 红：tests::adds' "$scratch/daily-parallel.stdout")" != 4 ]] \
     || ! grep -qF "已还原，基线仍全绿" "$scratch/daily-parallel.stdout"; then
    fail "会话里平常跑、MUTATE_JOBS=4（假包装照「日常一条」判，账上已有这个会话的一条在跑）：四条应当都判抓到、整轮退 0，实际退 $status：$(cat "$scratch/daily-parallel.out")"
  fi
  # 每次包装（基线 $expected_workers 次、四条变异、还原之后 $expected_workers 次）都放行、各带「第 <i> 个工作进程」的理由，几个工作进程的理由各不相同
  checked=$((checked + 1))
  admitted_reasons="$(sed -n 's/^admitted|//p' "$daily_log")"
  unmatched_reasons="$(grep -v '^同一次 mutate.sh 的第 [0-9]* 个工作进程，各用各的副本' <<<"$admitted_reasons")"
  distinct_reasons="$(sort -u <<<"$admitted_reasons" | grep -c .)"
  if grep -q '^refused|' "$daily_log" || [[ "$(grep -c '^admitted|' "$daily_log")" != $((2 * expected_workers + 4)) ]] \
     || [[ -n "$unmatched_reasons" || "$distinct_reasons" != "$expected_workers" ]]; then
    fail "会话里平常跑、MUTATE_JOBS=4 时每次包装应当带各自的并行理由（「同一次 mutate.sh 的第 <i> 个工作进程，各用各的副本……」，$expected_workers 个工作进程各一句）、一次都不被拒、共放行 $((2 * expected_workers + 4)) 次，实际假包装的记录：$(cat "$daily_log")"
  fi
  # 理由只设在自己起的那几次包装上：--check 那一次不带（export 出去就看得见）
  checked=$((checked + 1))
  check_reasons="$(sed -n 's/^check|//p' "$daily_log")"
  if [[ "$(grep -c '^check|' "$daily_log")" != 1 || -n "$check_reasons" ]]; then
    fail "--check 那一次不该带并行理由（理由只给 mutate.sh 自己起的那几次包装，不放宽别的命令），实际假包装的记录：$(cat "$daily_log")"
  fi

  # 调用方设了的理由原样传、不覆盖
  run_daily_case daily-caller-reason ../mutation/tables/caught-only.tsv MUTATE_JOBS=4 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="$caller_reason"
  status="$(cat "$scratch/daily-caller-reason.rc")"
  daily_log="$scratch/daily-caller-reason.daily"
  checked=$((checked + 1))
  admitted_reasons="$(sed -n 's/^admitted|//p' "$daily_log")"
  unmatched_reasons="$(grep -vxF -- "$caller_reason" <<<"$admitted_reasons")"
  if [[ "$status" != 0 || -z "$admitted_reasons" || -n "$unmatched_reasons" ]] || grep -q '^refused|' "$daily_log"; then
    fail "调用方设了的并行理由应当原样传给包装（每次都是「$caller_reason」）、照常判抓到退 0，实际退 $status、假包装的记录：$(cat "$daily_log")"
  fi

  # 调用方给的理由不够 8 个字（不覆盖）：基线就被拒，报「基线排不上」、退 2、一次 cargo 都不调，不说成「基线就是红的」
  run_daily_case daily-baseline-refused ../mutation/tables/caught-only.tsv RUN_WITH_MEMORY_CAP_PARALLEL_REASON="$short_caller_reason"
  status="$(cat "$scratch/daily-baseline-refused.rc")"
  checked=$((checked + 1))
  if [[ "$status" != 2 || -s "$scratch/daily-baseline-refused.cargo" ]] || ! grep -qF "基线排不上" "$scratch/daily-baseline-refused.stderr" \
     || grep -qF "基线就是红的" "$scratch/daily-baseline-refused.stderr"; then
    fail "基线排不上应当退 2、报「基线排不上」、一次 cargo 都不调，不说成「基线就是红的」，实际退 $status、调了 $(wc -l <"$scratch/daily-baseline-refused.cargo") 次：$(cat "$scratch/daily-baseline-refused.out")"
  fi

  # ── 平常跑带只供自证的变量 ──
  # 不是自证起的子进程（标记清空）而带着只供自证的变量：每个变量各试一次，都退 2、点名那个变量、一次 cargo 都不调
  local variable variable_value outside_label
  for variable in "${TEST_ONLY_VARIABLES[@]}"; do
    case "$variable" in
      MUTATE_BREAK) variable_value="memoryascaught${BROKEN_JUDGEMENT:+,$BROKEN_JUDGEMENT}" ;;
      MUTATE_SELFTEST_MEMORY_CAP_RUNNER) variable_value="$scratch/fake-memory-cap.sh" ;;
      RUN_WITH_MEMORY_CAP_BREAK) variable_value=nocap ;;
      RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE) variable_value=1 ;;
      *) variable_value="$scratch/not-a-wrapper.sh" ;;
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

  # ── 只编不跑（--compile-only）──
  # 三条的表：编得过一条、编不过一条（打第一行 error、整轮退 1）、只在测试外用不到的一条（test 档编得过）；
  # cargo 只调 check、参数逐字是 test 档那一句（基线一次加每条一次），一次 test 都不调
  case_leading_arguments=(--compile-only)
  prepare_case compile-only
  run_case compile-only ../mutation/tables/compile-only.tsv
  output="$(cat "$scratch/compile-only.out")"; status="$(cat "$scratch/compile-only.rc")"
  checked=$((checked + 1))
  [[ "$status" == 1 ]] || fail "只编不跑：三条的表（编得过、编不过、测试外死码各一条）应当整轮退 1，实际退 $status：$output"
  for wanted in "基线：编得过" "✅ [编得过] 编得过" "⏭  [编不过] 编不过：error[E0425]: cannot find value \`missing\` in this scope" "✅ [测试外死码] 编得过" \
                "计数：派出去 3 条，收回 3 份判定：编得过 2 条、编不过 1 条、锚点失效 0 条、现判 3 条、复用 0 条、拒算 0 条"; do
    checked=$((checked + 1))
    grep -qxF -- "$wanted" "$scratch/compile-only.stdout" || fail "只编不跑：三条的表应当整行打出「$wanted」，实际 stdout：$(cat "$scratch/compile-only.stdout")"
  done
  checked=$((checked + 1))
  unexpected_arguments="$(cut -d'|' -f4- "$scratch/compile-only.cargo" | grep -vxF 'check --profile test --bin fake-bench')"
  if [[ "$(grep -c . "$scratch/compile-only.cargo")" != 4 || -n "$unexpected_arguments" ]]; then
    fail "只编不跑：cargo 应当调 4 次（基线一次、每条一次）、参数逐字是「check --profile test --bin fake-bench」，实际调了 $(grep -c . "$scratch/compile-only.cargo") 次、对不上的：${unexpected_arguments:-无}"
  fi
  checked=$((checked + 1))
  [[ "$(head -1 "$scratch/compile-only.stderr")" == *"卡在「只编不跑」这一项"* ]] \
    || fail "只编不跑：stderr 头一行应当报开 1 个工作进程、卡在「只编不跑」，实际：$(head -1 "$scratch/compile-only.stderr")"

  # 丢一份判定：派出去 3 条、收回 2 份，退 8；再拿掉那道闸（countblind），同一个丢法静默退 0、只报 2 条
  prepare_case compile-only-lost-result
  run_case compile-only-lost-result ../mutation/tables/three-caught.tsv MUTATE_BREAK="droprow${BROKEN_JUDGEMENT:+,$BROKEN_JUDGEMENT}"
  status="$(cat "$scratch/compile-only-lost-result.rc")"
  checked=$((checked + 1))
  if [[ "$status" != 8 ]] || ! grep -qF "派出去 3 条" "$scratch/compile-only-lost-result.stderr" || ! grep -qF "收回 2 份" "$scratch/compile-only-lost-result.stderr"; then
    fail "只编不跑：丢了一份判定时应当退 8 并报「派出去 3 条……收回 2 份」，实际退 $status：$(cat "$scratch/compile-only-lost-result.out")"
  fi
  prepare_case compile-only-lost-result-unchecked
  run_case compile-only-lost-result-unchecked ../mutation/tables/three-caught.tsv MUTATE_BREAK=droprow,countblind
  status="$(cat "$scratch/compile-only-lost-result-unchecked.rc")"
  checked=$((checked + 1))
  if [[ "$status" != 0 || "$(grep -c '^✅' "$scratch/compile-only-lost-result-unchecked.stdout")" != 2 ]]; then
    fail "只编不跑：拿掉计数闸之后同一个丢法应当静默退 0、只报 2 条编得过（这样上一项的退 8 才是那道闸给的），实际退 $status：$(cat "$scratch/compile-only-lost-result-unchecked.out")"
  fi

  # 基线（没改过的源码）就编不过：退 2、点出第一行 error，基线之后一次 cargo 都不调
  prepare_case compile-only-baseline BROKEN
  run_case compile-only-baseline ../mutation/tables/comment-only.tsv
  status="$(cat "$scratch/compile-only-baseline.rc")"
  checked=$((checked + 1))
  if [[ "$status" != 2 || "$(grep -c . "$scratch/compile-only-baseline.cargo")" != 1 ]] \
     || ! grep -qF "基线（没改过的源码）就编不过" "$scratch/compile-only-baseline.stderr" \
     || ! grep -qF "第一处错：error[E0425]" "$scratch/compile-only-baseline.stderr"; then
    fail "只编不跑：基线编不过时应当退 2、报「基线（没改过的源码）就编不过」与第一处错、cargo 只调基线那一次，实际退 $status、调了 $(grep -c . "$scratch/compile-only-baseline.cargo") 次：$(cat "$scratch/compile-only-baseline.out")"
  fi
  case_leading_arguments=()

  # ── deny 警告（文件头「deny 警告」那一段）──
  # 一条的表：替换文留死代码（DEADCODE），假 cargo 照 deny 判、RUSTFLAGS 里没有 --cap-lints warn 就编不过。MUTATE_JOBS=1：
  # 平常跑调 cargo 3 次（基线、那一条、还原之后），只编不跑 2 次（基线、那一条）；每次收到的 RUSTFLAGS 逐字相同，调用方设了的接在前面
  # 后三例环境里设了 CARGO_ENCODED_RUSTFLAGS（空串也算）：假 cargo 照真 cargo 只读它，所以 mutate.sh 得把 cap 接进它（0x1f 分隔，调用方的在前）
  local rustflags_label rustflags_given rustflags_wanted wanted_line wanted_calls matched_rustflags
  local unit_separator=$'\x1f' encoded_set encoded_given encoded_wanted matched_encoded received_encoded
  for rustflags_label in cap-lints cap-lints-existing compile-only-cap-lints encoded-empty encoded-other compile-only-encoded-empty; do
    encoded_set=0; encoded_given=""; encoded_wanted="（没设）"
    case "$rustflags_label" in
      cap-lints) rustflags_given=""; rustflags_wanted="--cap-lints warn"; wanted_line="✅ [留死代码] 红：tests::adds"; wanted_calls=3
                 case_leading_arguments=() ;;
      cap-lints-existing) rustflags_given="-Cdebuginfo=0"; rustflags_wanted="-Cdebuginfo=0 --cap-lints warn"; wanted_line="✅ [留死代码] 红：tests::adds"; wanted_calls=3
                          case_leading_arguments=() ;;
      compile-only-cap-lints) rustflags_given="-Cdebuginfo=0"; rustflags_wanted="-Cdebuginfo=0 --cap-lints warn"; wanted_line="✅ [留死代码] 编得过"; wanted_calls=2
                              case_leading_arguments=(--compile-only) ;;
      encoded-empty) rustflags_given=""; rustflags_wanted="--cap-lints warn"; wanted_line="✅ [留死代码] 红：tests::adds"; wanted_calls=3
                     encoded_set=1; encoded_given=""; encoded_wanted="--cap-lints${unit_separator}warn"
                     case_leading_arguments=() ;;
      encoded-other) rustflags_given="-Cdebuginfo=0"; rustflags_wanted="-Cdebuginfo=0 --cap-lints warn"; wanted_line="✅ [留死代码] 红：tests::adds"; wanted_calls=3
                     encoded_set=1; encoded_given="-Copt-level=1${unit_separator}-Cdebuginfo=2"; encoded_wanted="-Copt-level=1${unit_separator}-Cdebuginfo=2${unit_separator}--cap-lints${unit_separator}warn"
                     case_leading_arguments=() ;;
      compile-only-encoded-empty) rustflags_given=""; rustflags_wanted="--cap-lints warn"; wanted_line="✅ [留死代码] 编得过"; wanted_calls=2
                                  encoded_set=1; encoded_given=""; encoded_wanted="--cap-lints${unit_separator}warn"
                                  case_leading_arguments=(--compile-only) ;;
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
      fail "deny 警告：$( ((${#case_leading_arguments[@]})) && echo 只编不跑 || echo 平常跑)（调用方 RUSTFLAGS=「$rustflags_given」）留死代码那一条应当$( ((${#case_leading_arguments[@]})) && echo 判编得过 || echo 判抓到)、整轮退 0（整行「$wanted_line」），实际退 $status：$(cat "$scratch/$rustflags_label.out")"
    fi
    checked=$((checked + 1))
    # 数逐字等于期望的那几行（不取「不等于的」再判空：收到空串的那几行取出来是空行，$( ) 一去尾就看不见了）
    matched_rustflags="$(cut -d'|' -f3- "$scratch/$rustflags_label.cargo.rustflags" 2>/dev/null | grep -cxF -- "$rustflags_wanted")"
    if [[ "$(grep -c . "$scratch/$rustflags_label.cargo.rustflags" 2>/dev/null)" != "$wanted_calls" || "$matched_rustflags" != "$wanted_calls" ]] \
       || [[ "$(cut -d'|' -f1 "$scratch/$rustflags_label.cargo.rustflags" | sort -u | tr '\n' ' ')" != "DEADCODE PASS " ]]; then
      fail "deny 警告：$rustflags_label 里每个 cargo 收到的 RUSTFLAGS 应当逐字是「$rustflags_wanted」、调 $wanted_calls 次、基线（PASS）与那一条（DEADCODE）都在，实际：$(cat "$scratch/$rustflags_label.cargo.rustflags" 2>/dev/null | tr '\n' ' ')"
    fi
    # CARGO_ENCODED_RUSTFLAGS：没设的三例每个 cargo 都收到「（没设）」（不凭空多出一个），设了的三例逐字是 encoded_wanted（0x1f 在 cat -v 里显示成 ^_）
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
  case_leading_arguments=()

  # --binary-of：显式 [[bin]] 的打声明的名字；没写 [[bin]] 的打文件名去掉 .rs；源文件不在退 2
  local binary_output binary_status
  prepare_case binary-of
  printf 'fn main() {}\n' >"$scratch/binary-of/scripts/experiments/bench/src/bin/other_tool.rs"
  for wanted in "bench/src/bin/fake_bench.rs fake-bench" "bench/src/bin/other_tool.rs other_tool"; do
    checked=$((checked + 1))
    if binary_output="$(cd "$scratch/binary-of/scripts/experiments" && bash "$case_script" --binary-of "${wanted%% *}" 2>&1)"; then binary_status=0; else binary_status=$?; fi
    [[ "$binary_status" == 0 && "$binary_output" == "${wanted##* }" ]] \
      || fail "--binary-of ${wanted%% *} 应当退 0、只打「${wanted##* }」，实际退 $binary_status：$binary_output"
  done
  checked=$((checked + 1))
  if binary_output="$(cd "$scratch/binary-of/scripts/experiments" && bash "$case_script" --binary-of bench/src/bin/absent.rs 2>&1)"; then binary_status=0; else binary_status=$?; fi
  [[ "$binary_status" == 2 && "$binary_output" == *"→ 怎么办"* ]] || fail "--binary-of 给的源文件不在时应当退 2、给出路，实际退 $binary_status：$binary_output"

  # 按条入库（文件头「按条入库」）：被测源码换成一份带 #[test] 的（假 cargo 照旧只看 BEHAVIOR），抓到的那一条第二趟复用、一次 cargo 都不起（unit-library）；
  # 改了被改坏的函数调到的 helper 之后那一条现判（unit-library-callee）；--full 只写不读；原文只在注释里的那一条拒算、每趟照跑；只编不跑同样按条复用。
  # 弄坏开关 libraryunread（不读库）让「同一条第二趟复用」与「只编不跑第二趟复用」判错，calleesignored（键只取包住原文那个函数自己的指纹）让「改被改坏函数调到的函数后现判」判错
  write_unit_library_source() { # <例的目录>
    cat >"$1/scripts/experiments/bench/src/bin/fake_bench.rs" <<'UNIT_SOURCE'
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
  local saved_leading_arguments=(${case_leading_arguments[@]+"${case_leading_arguments[@]}"})
  case_leading_arguments=(--full)
  run_case unit-library ../mutation/tables/caught-only.tsv
  case_leading_arguments=(${saved_leading_arguments[@]+"${saved_leading_arguments[@]}"})
  output="$(cat "$scratch/unit-library.out")"; status="$(cat "$scratch/unit-library.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library.cargo")" -ge 2 ]] && grep -qF "、现判 1 条、复用 0 条、拒算 0 条" <<<"$output" && grep -qF "（--full：只写不读）" <<<"$output" \
    || fail "--full 应当只写不读：库里有这把键照样现判（cargo 照起），计数行报现判 1 条，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library.cargo") 次：$output"
  # 改了被改坏的函数（adds）调到的 helper：包住原文那个函数的闭包变了，键跟着变，那一条现判
  prepare_case unit-library-callee
  write_unit_library_source "$scratch/unit-library-callee"
  run_case unit-library-callee ../mutation/tables/caught-only.tsv
  CALLEE_SOURCE="$scratch/unit-library-callee/scripts/experiments/bench/src/bin/fake_bench.rs" python3 - <<'PY' || fail "改不了 helper（自证自己的样本出错）"
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
  # 只编不跑：编得过的那一条第二趟复用，一次 cargo check 都不起（基线也不编）
  prepare_case unit-library-compile
  write_unit_library_source "$scratch/unit-library-compile"
  saved_leading_arguments=(${case_leading_arguments[@]+"${case_leading_arguments[@]}"})
  case_leading_arguments=(--compile-only)
  run_case unit-library-compile ../mutation/tables/caught-only.tsv
  run_case unit-library-compile ../mutation/tables/caught-only.tsv
  case_leading_arguments=(${saved_leading_arguments[@]+"${saved_leading_arguments[@]}"})
  output="$(cat "$scratch/unit-library-compile.out")"; status="$(cat "$scratch/unit-library-compile.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library-compile.cargo")" == 0 ]] && grep -qF "✅ [抓到] 编得过" "$scratch/unit-library-compile.stdout" \
     && grep -qF "基线：没编" "$scratch/unit-library-compile.stdout" && grep -qF "编得过 1 条、编不过 0 条、锚点失效 0 条、现判 0 条、复用 1 条、拒算 0 条" "$scratch/unit-library-compile.stdout" \
    || fail "只编不跑第二趟应当复用：判定照旧是编得过、计数行报复用 1 条、基线也不编、一次 cargo 都不调，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library-compile.cargo") 次：$output"

  # 依赖版本行（lib_mutation_units.py 文件头「依赖版本行」）：只收包住原文那个函数的闭包里真点名的外部 crate 与它们的传递依赖。
  # 夹具：bench 声明 extdep、other、alias-dep（package = "aliased"）三个外部依赖，Cargo.lock 里 extdep 带一个传递依赖 inner；
  # adds 调 helper、helper 点 extdep::，other 只在闭包外的 unrelated 里点。头一趟库里那一条的依赖版本行逐字是 extdep、inner 两行；
  # 不点外部 crate 的 unit-library 那一例是空的；改 other 的版本第二趟照样复用、一次 cargo 都不起；改 inner 的版本现判；
  # helper 改点 alias_dep:: 之后依赖版本行逐字是 aliased 一行（认 package = 改名）。
  # 弄坏开关 dependenciesbycrate（按整包收）让头一趟与改名那一趟多出别的行、改 other 之后现判
  library_dependency_lines() { # <按条判定库的目录>：最近写下的那一条判定记的依赖版本行，一行一条；空的打「（空）」
    python3 - "$1" <<'PY'
import gzip, json, os, sys
records = os.path.join(sys.argv[1], "records")
newest = max((os.path.join(records, name) for name in os.listdir(records) if not name.startswith(".")), key=os.path.getmtime)
with open(newest, encoding="utf-8") as handle:
    fields = dict(line.split("=", 1) for line in handle.read().splitlines() if "=" in line and not line.startswith("#"))
with gzip.open(os.path.join(sys.argv[1], "compositions", fields["dependency"] + ".json.gz"), "rt", encoding="utf-8") as handle:
    lines = json.load(handle).get("lines", [])
print("\n".join(lines) if lines else "（空）")
PY
  }
  write_dependency_lock() { # <Cargo.lock 的路径> <other 的版本> <inner 的版本>
    local registry='registry+https://example.invalid/index'
    printf 'version = 4\n\n[[package]]\nname = "aliased"\nversion = "3.0.0"\nsource = "%s"\nchecksum = "33"\n\n[[package]]\nname = "bench"\nversion = "0.0.0"\ndependencies = [\n "aliased",\n "extdep",\n "other",\n]\n\n[[package]]\nname = "extdep"\nversion = "1.0.0"\nsource = "%s"\nchecksum = "aa"\ndependencies = [\n "inner",\n]\n\n[[package]]\nname = "inner"\nversion = "%s"\nsource = "%s"\nchecksum = "inner-%s"\n\n[[package]]\nname = "other"\nversion = "%s"\nsource = "%s"\nchecksum = "other-%s"\n' \
      "$registry" "$registry" "$3" "$registry" "$3" "$2" "$registry" "$2" >"$1"
  }
  write_dependency_source() { # <例的目录> <helper 点的外部 crate 的代码名>
    cat >"$1/scripts/experiments/bench/src/bin/fake_bench.rs" <<DEPENDENCY_SOURCE
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
  printf '\n[dependencies]\nextdep = "1.0"\nother = "2.0"\nalias-dep = { package = "aliased", version = "3.0" }\n' >>"$dependency_case/scripts/experiments/bench/Cargo.toml"
  write_dependency_lock "$dependency_case/scripts/experiments/Cargo.lock" 2.0.0 0.5.0
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
  write_dependency_lock "$dependency_case/scripts/experiments/Cargo.lock" 2.0.1 0.5.0
  run_case unit-library-dependency ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library-dependency.out")"; status="$(cat "$scratch/unit-library-dependency.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library-dependency.cargo")" == 0 ]] && grep -qF "、现判 0 条、复用 1 条、拒算 0 条" <<<"$output" \
    || fail "依赖版本行：改了闭包没点名的外部包 other 的版本之后那一条应当照样复用、一次 cargo 都不起，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library-dependency.cargo") 次：$output"
  write_dependency_lock "$dependency_case/scripts/experiments/Cargo.lock" 2.0.1 0.5.1
  run_case unit-library-dependency ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library-dependency.out")"; status="$(cat "$scratch/unit-library-dependency.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$(real_cargo_calls "$scratch/unit-library-dependency.cargo")" -ge 2 ]] && grep -qF "、现判 1 条、复用 0 条、拒算 0 条" <<<"$output" \
     && grep -qF "现判：[抓到] 库里没有这把键" <<<"$output" \
    || fail "依赖版本行：改了点名的 extdep 的传递依赖 inner 的版本之后那一条应当现判（stderr 写库里没有这把键），实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/unit-library-dependency.cargo") 次：$output"
  write_dependency_source "$dependency_case" alias_dep
  run_case unit-library-dependency ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/unit-library-dependency.out")"; status="$(cat "$scratch/unit-library-dependency.rc")"
  expected_dependency_lines="$(printf 'aliased\t3.0.0\t%s\t33' "$dependency_registry")"
  recorded_dependency_lines="$(library_dependency_lines "$dependency_case/library" 2>&1)"
  checked=$((checked + 1))
  [[ "$status" == 0 && "$recorded_dependency_lines" == "$expected_dependency_lines" ]] \
    || fail "依赖版本行：helper 改点 alias_dep:: 之后库里那一条应当逐字只记 aliased 一行（清单里 alias-dep 写了 package = \"aliased\"），实际退 $status、记的是「$recorded_dependency_lines」：$output"

  # 复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」，格名 reuse-three-questions）：一例自己的目录与判定库，表里两条抓到——
  # 「改加法」改 adds（闭包含 helper）、「改孤函数」改 lonely（闭包含 lonely_helper，测试都不调它）。四趟：库是空的（两条现判）、同一份输入（①）、
  # 加一个没人调的函数（闭包外，②）、改 lonely_helper（只在「改孤函数」的闭包里，③ affected 改孤函数）；这四趟哨兵数设 1000（键相同的全当哨兵、不抽强制现判）。
  # 每趟的结局从 stderr 取：「现判：[名]」是现判（原因以「强制现判」起头的是强制现判）、「拒算（照跑、不复用）：[名]」是拒算，表里别的是复用。
  # 弄坏开关 libraryunread（不读库）让 ①②③ 全红，calleesignored（键不收被调的单元）让 ③ 红
  write_questions_source() { # <例的目录>
    cat >"$1/scripts/experiments/bench/src/bin/fake_bench.rs" <<'QUESTIONS_SOURCE'
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
  question_source="$question_case/scripts/experiments/bench/src/bin/fake_bench.rs"
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

  # 副本范围（文件头「变异只改副本」一段）：装置用 include_str! 读 scripts/experiments/ 之外的仓内文件（../../.claude/heavy-thread-default），
  # 副本根下链上、基线编得过、照常判抓到（include-outside）；认不出的写法与落到仓根之外的路径退 2 点名那一行、一次 cargo 都不起（include-unrecognized）。
  # 弄坏开关 includesunlinked（不链）让两例都判错
  prepare_case include-outside
  mkdir -p "$scratch/include-outside/.claude"
  printf '7\n' >"$scratch/include-outside/.claude/heavy-thread-default"
  printf 'pub const THREADS: &str = include_str!("../../.claude/heavy-thread-default");\n' >"$scratch/include-outside/scripts/experiments/threads_probe.rs"
  run_case include-outside ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/include-outside.out")"; status="$(cat "$scratch/include-outside.rc")"
  checked=$((checked + 1))
  [[ "$status" == 0 ]] && grep -qF "✅ [抓到] 红：tests::adds" <<<"$output" && grep -qF "副本根下链了 1 份 scripts/experiments/ 之外的仓内文件" <<<"$output" \
    || fail "装置 include_str 读仓根下别处的文件（../../.claude/heavy-thread-default）时副本根下应当链上、基线编得过、照常判抓到，实际退 $status：$output"
  prepare_case include-unrecognized
  printf 'pub const PIECE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/piece.txt"));\npub const FAR: &str = include_str!("../../../../outside.txt");\n' \
    >"$scratch/include-unrecognized/scripts/experiments/threads_probe.rs"
  run_case include-unrecognized ../mutation/tables/caught-only.tsv
  output="$(cat "$scratch/include-unrecognized.out")"; status="$(cat "$scratch/include-unrecognized.rc")"
  checked=$((checked + 1))
  [[ "$status" == 2 && "$(real_cargo_calls "$scratch/include-unrecognized.cargo")" == 0 ]] \
     && grep -qF "scripts/experiments/threads_probe.rs:1：include_str! 的参数不是一个串字面量" <<<"$output" \
     && grep -qF 'scripts/experiments/threads_probe.rs:2：include_str!("../../../../outside.txt") 落到仓根之外' <<<"$output" \
    || fail "认不出的 include_str! 写法与落到仓根之外的路径应当退 2、逐行点名、一次 cargo 都不起，实际退 $status、cargo 起了 $(real_cargo_calls "$scratch/include-unrecognized.cargo") 次：$output"

  systemctl --user stop "$selftest_slice" >/dev/null 2>&1
  systemctl --user revert "$selftest_slice" >/dev/null 2>&1
  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ mutate.sh 自检 $failures 处不对（查了 $checked 项）"
    echo "  → 怎么办：照上面逐条改——判结局的在 judge_mutation（退出码 $TIME_LIMIT_HIT_EXIT / $MEMORY_CAP_HIT_EXIT / $MEMORY_CAP_UNAVAILABLE_EXIT / $MEMORY_ADMISSION_REFUSED_EXIT / $SLICE_TOTAL_HIT_EXIT 的分支），"
    echo "    限时与包装 stderr 的去向在 run_capped_cargo_test，基线与收尾计数在主流程，"
    echo "    并行的在 worker_count_and_head_line、run_worker 与 print_judged_rows_in_table_order，required-features 在 read_required_features；"
    echo "    平常跑拒只供自证的变量在 --selftest 那一支后面（running_under_selftest），并行理由在 run_capped_cargo_test（CALLER_PARALLEL_REASON），基线排不上、基线被总上限挤掉在基线那一段；"
    echo "    只编不跑在 run_compile_only（cargo 的参数在 CARGO_CHECK_COMMAND），--binary-of 在 find_declared_binary；"
    echo "    每个 cargo 带的 RUSTFLAGS 与 CARGO_ENCODED_RUSTFLAGS 在 MUTATE_RUSTFLAGS、MUTATE_ENCODED_RUSTFLAGS 与 RUSTFLAGS_ENVIRONMENT（check_in_copy、run_capped_cargo_test 都用它）；"
    echo "    MUTATE_BREAK 或 RUN_WITH_MEMORY_CAP_BREAK 设着的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ mutate.sh 自检通过（查了 $checked 项：抓到、没红、无效、超时、内存撞顶五种结局各归各类，撞顶不记成抓到或没红、整轮判失败并单列计数，\
包装自己的话打在 stderr 上，上限之内正常的变异照旧判抓到退 0，scope 起不来退 7 且一次 cargo 都没跑，基线撞顶退 2 并说是内存上限，基线被总上限挤掉原样退 254；\
基线红退 2 并点出红的测试、末尾几行与留下的整份日志，should panic 的用例红了（变异那一趟与基线那一趟）名字不带「 - should panic」，参数不够退 2 打用法与例子、一次 cargo 都不调；\
并行：MUTATE_JOBS=1 与 =4 的 stdout 逐字相同、没红的照样判失败、头一行报开了几个、各工作进程各用一份副本与编译目录，\
丢一份判定退 8 而拿掉那道闸就静默判绿，MUTATE_JOBS、MUTATE_TIMEOUT 写错退 2，锚点失效那一条记 ⚓、不入库而别的照跑、计数报锚点失效、整轮退 3（只编不跑同样）；\
required-features：没挂的 bin 的 cargo 参数逐字不变，挂了两个的带「包名/feature」跑得起来、照常判抓到；\
包装每次放行前停 $selftest_time_limit 秒，五种结局照样各归各类、只有 HANG 那一条记超时；\
包装退 252 记排不上、退 254 记被总上限挤掉、退 245 记显存不够，都整轮判失败并单列计数；平常跑带只供自证的 ${#TEST_ONLY_VARIABLES[@]} 个变量各一次都退 2；\
会话里平常跑并行（假包装照「日常一条」判）：MUTATE_JOBS=4 各条判对退 0、每次包装带各自的并行理由而 --check 不带，调用方的理由不覆盖，基线排不上退 2 报「基线排不上」；\
只编不跑：编得过、编不过（退 1、打第一行 error）、测试外死码按 test 档编得过，cargo 只调 check --profile test，丢一份判定退 8 而拿掉那道闸就静默退 0，基线编不过退 2；\
deny 警告：留死代码的那一条平常跑判抓到、只编不跑判编得过，基线、那一条与还原之后那一次的 RUSTFLAGS 逐字相同、都带 --cap-lints warn、调用方的接在前面；\
环境里设了 CARGO_ENCODED_RUSTFLAGS（空串、别的值、只编不跑）时 cap 接进它、0x1f 分隔，cargo 实收的与 stderr 报的一致，没设的不凭空多出一个；\
--binary-of 显式 [[bin]] 与没写的两种各打对名字、源文件不在退 2；\
按条入库：同一条第二趟复用、一次 cargo 都不起，改了被改坏函数调到的函数之后那一条现判，--full 只写不读，原文只在注释里的那一条拒算照跑，只编不跑第二趟同样复用；\
依赖版本行只收闭包里真点名的外部 crate 与传递依赖（逐字对得上夹具、不点的为空、改闭包外点的包照样复用、改传递依赖现判、认 package = 改名）；\
副本根下链上装置 include_str! 读的仓内别处文件、基线编得过，认不出的写法与落到仓根之外的路径退 2 逐行点名）"
}

if [[ "${1:-}" == "--selftest" ]]; then
  run_selftest
  exit 0
fi
# 平常跑：只供自证的变量有一个非空就拒，一次 cargo 都不调（自证起的子进程除外，文件头「只供自证」）
if ((!running_under_selftest)) && ! is_broken honorbreakoutsideselftest; then
  test_only_given=()
  for variable in "${TEST_ONLY_VARIABLES[@]}"; do
    [[ -n "${!variable:-}" ]] && test_only_given+=("$variable")
  done
  if ((${#test_only_given[@]})); then
    echo "mutate: 平常跑时带了只给自证用的变量：${test_only_given[*]}；拒跑，一条变异都没跑" >&2
    echo "        → 怎么办：unset ${test_only_given[*]} 再跑；要证红就跑 bash $SELF_PATH --selftest（它起的子进程才认这几个变量）" >&2
    exit 2
  fi
fi
# 头一个参数是 --compile-only（只编不跑）或 --binary-of（只打 bin 名）的，拿掉之后照下面解析（文件头用法那几行）
COMPILE_ONLY=0
PRINT_BINARY_ONLY=0
FULL_RUN=0   # --full：只写不读（文件头「按条入库」）
while (($#)); do
  case "$1" in
    --compile-only) COMPILE_ONLY=1; shift ;;
    --binary-of) PRINT_BINARY_ONLY=1; shift; break ;;
    --full) FULL_RUN=1; shift ;;
    *) break ;;
  esac
done

# ⚠️ **先确认「改的文件」与「跑的二进制」对得上。**
# 不确认的话，变异会改 X 而跑 Y 的测试：一条真能被抓的破坏被报成盲区
# （2026-08-29 实测踩过：`e7_index.rs` 配上 crate 主二进制的名字，
# 三条变异全被误报成盲区，换对名字后三条全被抓）。
# 反方向更坏：Y 的测试恰好因别的原因红了，会被记成「变异被抓」。
# 清单：从源文件所在目录一层层往上找最近的 Cargo.toml（实验工作区根是 scripts/experiments，src/bin 布局与自带清单的子 crate 同样认得）。
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
    echo "        → 怎么办：在 scripts/experiments/ 目录里照这个例子跑：bash ../mutation/mutate.sh --binary-of e172_deletion_write_dry_run.rs" >&2
    exit 2
  fi
  find_declared_binary "$1"
  echo "${_declared:-$(basename "$1" .rs)}"   # 没写 [[bin]] 的按 cargo 自动发现的规则：文件名去掉 .rs
  exit 0
fi

# 参数不够三个：打用法与一条例子，退 2（文件头「退出码」：2 是用法错），不让 set -u 报 unbound variable
if (($# < 3)) && ! is_broken bareusage; then
  echo "mutate: 要三个参数，给了 $# 个；用法：mutate.sh [--full] [--compile-only] <bin 名> <源文件> <变异表>" >&2
  echo "        <bin 名> 是 Cargo.toml 里 [[bin]] 的 name（没写 [[bin]] 的是源文件名去掉 .rs，可以用 mutate.sh --binary-of <源文件> 查）；<源文件> 与 <变异表> 都相对当前目录（scripts/experiments/）" >&2
  echo "        → 怎么办：在 scripts/experiments/ 目录里照这个例子跑：bash ../mutation/mutate.sh e172-deletion-write-dry-run e172_deletion_write_dry_run.rs ../mutation/tables/e172_deletion_write_dry_run.tsv" >&2
  exit 2
fi
BIN="$1"; SRC="$2"; TABLE="$3"

find_declared_binary "$SRC"
if [[ -n "$_declared" ]]; then
  if [[ "$_declared" != "$BIN" ]]; then
    echo "mutate: 源文件 $SRC 在 Cargo.toml 里声明的二进制是 '$_declared'，不是 '$BIN'" >&2
    echo "        改的文件与跑的测试对不上，整份证明作废。用： mutate.sh $_declared $SRC $TABLE" >&2
    exit 6
  fi
else
  # 没有显式 [[bin]] ⇒ cargo 自动发现，名字就是文件名去掉扩展名
  _stem="$(basename "$SRC" .rs)"
  if [[ "$_stem" != "$BIN" ]]; then
    echo "mutate: $SRC 没有显式 [[bin]]，自动发现的名字是 '$_stem'，不是 '$BIN'" >&2
    echo "        改的文件与跑的测试对不上，整份证明作废。用： mutate.sh $_stem $SRC $TABLE" >&2
    exit 6
  fi
fi

# bin 挂了 required-features 的，cargo test 要带上那几个 feature（文件头 required-features 那一段）。
# 从 $_manifest 里名字是 $BIN 的那一节 [[bin]] 现读，写成「包名/feature」逗号连起来打到 stdout；没挂、或清单里没有这一节，打空行。
# 写成带包名的形式：scripts/experiments/ 是 workspace 根，在它的根上不带包名的 --features 要靠被选中的包碰巧有那个 feature
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

# ⚠️ **变异只改副本，不碰工作区里的源文件**（2026-09-12 改）。此前是就地改 $SRC 再还原：一轮变异要几分钟，
# 这几分钟里仓里那份源码是坏的——几个会话共写一个仓时，别的会话此刻跑门禁（checker-tier-research-build-and-replay 阶段会编译并跑实验装置的全部单测）
# 就编到被改坏的源码，红得莫名其妙、还可能当成自己改坏的；此刻有人整份提交这个文件，提交进去的就是变异。
# ⇒ 把 scripts/experiments/ 整个工作区（不带 target）拷到临时目录，include_str! 读的 research/results/ 与别的仓内文件（.claude/heavy-thread-default 这类）
#   在副本根下照层级链回仓里那一份（认不出的写法退 2，见拷副本那一段），
#   在那里改、在那里编；CARGO_TARGET_DIR 单独一份，不碰 scripts/experiments/target 里别人要用的二进制。跑完删掉副本。
#   并行时每个工作进程各一份这样的副本、各一个编译目录（文件头「并行」那一段）。
if [[ ! -f Cargo.toml ]] || ! grep -q '^\[workspace\]' Cargo.toml; then
  echo "mutate: 要在 scripts/experiments/ 下跑（那里才有 workspace 的 Cargo.toml）" >&2; exit 2
fi
REQUESTED_JOBS="${MUTATE_JOBS:-}"
if [[ -n "$REQUESTED_JOBS" && ! "$REQUESTED_JOBS" =~ ^[1-9][0-9]*$ ]]; then
  echo "mutate: MUTATE_JOBS 写成了「$REQUESTED_JOBS」，要写正整数（它只能把工作进程数压小），一条变异都没跑" >&2
  echo "        → 怎么办：写成 1、2、4 这样的正整数再跑；不设就按核数的一半（至多 $MAXIMUM_WORKERS_BY_PROCESSOR）与表里的条数取小" >&2
  exit 2
fi
if [[ ! "$ROW_TIME_LIMIT" =~ ^[1-9][0-9]*$ ]]; then
  echo "mutate: MUTATE_TIMEOUT 写成了「$ROW_TIME_LIMIT」，要写正整数秒（交给包装的 RUN_WITH_MEMORY_CAP_TIME_LIMIT，它只认正整数秒），一条变异都没跑" >&2
  echo "        → 怎么办：写成 120、600 这样的正整数再跑；不设就是 120 秒" >&2
  exit 2
fi
REQUESTED_CARGO_JOBS="${MUTATE_CARGO_JOBS:-}"
if [[ -n "$REQUESTED_CARGO_JOBS" && ! "$REQUESTED_CARGO_JOBS" =~ ^[1-9][0-9]*$ ]]; then
  echo "mutate: MUTATE_CARGO_JOBS 写成了「$REQUESTED_CARGO_JOBS」，要写正整数，一条变异都没跑" >&2
  echo "        → 怎么办：写成正整数再跑；不设就用调用方的 CARGO_BUILD_JOBS，那也没设就取 核数 ÷ 工作进程数" >&2
  exit 2
fi
if is_broken exportparallelreason && [[ -z "${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-}" ]]; then
  # 弄坏开关：并行理由 export 给之后起的每一条命令（--check 也看得见），不只设在自己起的那几次包装上
  export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="mutate.sh 起的每一条命令都照并行放行（弄坏开关 exportparallelreason）"
fi
# 开跑之前先试一次带上限的 scope 起不起得来：起不来就一条都不跑，不退回无上限（run-with-memory-cap.sh 自己把原因打到 stderr）。
# 只编不跑不经包装（cargo check 不跑编出来的代码），不试
memory_cap_check=0
if ((!COMPILE_ONLY)) && ! bash "$MEMORY_CAP_RUNNER" --check "$MEMORY_MAX"; then memory_cap_check=1; fi
if [[ $memory_cap_check -ne 0 ]]; then
  echo "mutate: 带内存上限（MUTATE_MEMORY_MAX=$MEMORY_MAX）的 systemd scope 起不来，一条变异都没跑" >&2
  echo "        → 怎么办：看上面 run-with-memory-cap.sh 的报错，修好用户级 systemd / D-Bus 或上限的写法再跑；不许拿掉上限去跑（无界分配的变异会把整机拖进 OOM）" >&2
  exit 7
fi

# 读表：一行三段、制表符分隔，空行与 # 开头的行跳过（与改成并行之前逐行读的口径相同），按表序进三个数组
mutation_names=(); mutation_originals=(); mutation_replacements=()
while IFS=$'\t' read -r name from to; do
  [[ -z "${name:-}" || "${name:0:1}" == "#" ]] && continue
  mutation_names+=("$name"); mutation_originals+=("$from"); mutation_replacements+=("$to")
done < "$TABLE"
mutation_count=${#mutation_names[@]}

# 开几个工作进程：各项取最小、至少 1，连同各项给几个、卡在哪一项、每个 cargo 的编译并行度一起打在 stderr 的头一行（文件头「并行」那一段）
worker_count_and_head_line() {
  local processor_count by_processor smallest binding="" floor_note="" bound_index
  local -a bound_names=() bound_counts=() bound_texts=()
  processor_count="$(nproc 2>/dev/null || echo 2)"
  by_processor=$((processor_count / 2))
  ((by_processor > MAXIMUM_WORKERS_BY_PROCESSOR)) && by_processor=$MAXIMUM_WORKERS_BY_PROCESSOR
  ((by_processor < 1)) && by_processor=1
  bound_names+=("核数"); bound_counts+=("$by_processor"); bound_texts+=("核数 $processor_count 的一半、至多 $MAXIMUM_WORKERS_BY_PROCESSOR，给 $by_processor 个")
  if [[ -n "$REQUESTED_JOBS" ]]; then
    bound_names+=("MUTATE_JOBS"); bound_counts+=("$REQUESTED_JOBS"); bound_texts+=("MUTATE_JOBS 给 $REQUESTED_JOBS 个")
  fi
  if ((COMPILE_ONLY)); then
    bound_names+=("只编不跑"); bound_counts+=(1); bound_texts+=("只编不跑串行、一份副本、一个编译目录，给 1 个")
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
  echo "mutate: $BIN 在 $_manifest 里挂了 required-features，每次 cargo $( ((COMPILE_ONLY)) && echo check || echo test) 带 ${cargo_feature_arguments[*]}" >&2
fi
export CARGO_BUILD_JOBS="$CARGO_JOBS_PER_WORKER"

SHARD_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-mutate-XXXXXX")"
PRISTINE="$SHARD_ROOT/pristine-source"   # 开跑时那一份被测源码，每条变异都从它改、每次还原都拷它
CLAIMS="$SHARD_ROOT/claims"              # <行号>/：那一条被哪个工作进程认领了（mkdir 建成的那个领走）
RESULTS="$SHARD_ROOT/results"            # <行号>.line（要打到 stdout 的那一行）、.stderr、.verdict（「档 要不要判失败」，最后写）
STOP_FLAG="$SHARD_ROOT/stop"             # 有一条判出「scope 起不来」：其余工作进程领完手上那条就不再领
mkdir -p "$CLAIMS" "$RESULTS"
worker_pids=()
cleanup() {
  local pid
  for pid in "${worker_pids[@]}"; do
    if kill -0 "$pid" 2>/dev/null; then
      kill -TERM "$pid" 2>/dev/null
      wait "$pid" 2>/dev/null
    fi
  done
  rm -rf "${SHARD_ROOT:?}"
}
trap cleanup EXIT
MUTATE_TARGET="${MUTATE_TARGET_DIR:-${TMPDIR:-/tmp}/singlefs-mutate-target}"
# 副本照仓里的层级放（copy-<i>/scripts/experiments/）：装置用 include_str!("../../research/results/…") 读留存产物，
# 副本根下放一个符号链接指回仓里那一份（只读，变异不改它）
copy_root_of() { echo "$SHARD_ROOT/copy-$1"; }
copy_directory_of() { echo "$SHARD_ROOT/copy-$1/scripts/experiments"; }
REPOSITORY_OF_WORKSPACE="$(cd ../.. && pwd)"
target_directory_of() { if (($1 == 1)); then echo "$MUTATE_TARGET"; else echo "$MUTATE_TARGET-w$1"; fi; }

if ! mkdir -p "$(copy_directory_of 1)" "$(copy_root_of 1)/research" \
   || ! ln -s "$REPOSITORY_OF_WORKSPACE/research/results" "$(copy_root_of 1)/research/results" \
   || ! rsync -a --exclude target ./ "$(copy_directory_of 1)/"; then
  echo "mutate: 拷副本失败" >&2; exit 2
fi
# 装置用 include_str! / include_bytes! 读 scripts/experiments/ 之外的仓内文件（../../.claude/heavy-thread-default 这类）：副本根下照仓里的层级
# 放一个符号链接指回仓里那一份（只读，变异不改它；research/results 上面已经链了）。认法：副本里每份 .rs 的词法单元（lib_function_units.py 的
# rust_lex，注释与串里的不算）里 include_str / include_bytes ! 加括号，括号里恰好一个串字面量（后面可带一个逗号），路径相对那份源文件。
# 认不出的写法（concat!、env! 拼的路径）、绝对路径、落到仓根之外的、仓里没有那份文件的，退 2 点名那一行，一条变异都不跑。
# 弄坏开关 includesunlinked：不链（改前的样子，副本里读不到就编不过）
INCLUDE_LINKER="$(dirname "$SELF_PATH")/../gate/lib_function_units.py"
if ! is_broken includesunlinked && ! python3 - "$INCLUDE_LINKER" "$(copy_root_of 1)" "$REPOSITORY_OF_WORKSPACE" <<'PY'
import importlib.util, os, sys
library_path, copy_root, repository = sys.argv[1:4]
specification = importlib.util.spec_from_file_location("lib_function_units_for_mutate_includes", library_path)
units = importlib.util.module_from_spec(specification)
specification.loader.exec_module(units)
closing = {"(": ")", "[": "]", "{": "}"}
problems, linked = [], []
for directory, subdirectories, files in os.walk(os.path.join(copy_root, "scripts", "experiments")):
    subdirectories[:] = sorted(name for name in subdirectories if name != "target")
    for name in sorted(files):
        if not name.endswith(".rs"):
            continue
        path = os.path.join(directory, name)
        shown = os.path.relpath(path, copy_root)
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            text = handle.read()
        if "include_str" not in text and "include_bytes" not in text:
            continue
        kinds, texts, lines, _documents = units.rust_lex(text)
        for index, word in enumerate(texts):
            if kinds[index] != units.IDENT or word not in ("include_str", "include_bytes") or texts[index + 1:index + 2] != ["!"] \
                    or texts[index + 2:index + 3] not in (["("], ["["], ["{"]):
                continue
            where = f"{shown}:{lines[index]}"
            argument = texts[index + 3] if index + 3 < len(texts) else ""
            value = units.rust_string_value(argument) if kinds[index + 3:index + 4] == [units.LITERAL] else None
            after = texts[index + 4:index + 6]
            if value is None or not (after[:1] == [closing[texts[index + 2]]] or after == [",", closing[texts[index + 2]]]):
                problems.append(f"{where}：{word}! 的参数不是一个串字面量（concat!、env! 拼的路径这里认不出）")
                continue
            if os.path.isabs(value):
                problems.append(f"{where}：{word}!(\"{value}\") 是绝对路径")
                continue
            relative = os.path.relpath(os.path.normpath(os.path.join(directory, value)), copy_root)
            if relative == ".." or relative.startswith("../"):
                problems.append(f"{where}：{word}!(\"{value}\") 落到仓根之外")
                continue
            parts = relative.split(os.sep)
            if any(os.path.islink(os.path.join(copy_root, *parts[:depth])) for depth in range(1, len(parts) + 1)) \
                    or os.path.exists(os.path.join(copy_root, relative)):
                continue   # 副本里本来就有（scripts/experiments/ 里的、research/results 链过去的、前面链过的）
            source = os.path.join(repository, relative)
            if not os.path.isfile(source):
                problems.append(f"{where}：{word}!(\"{value}\") 指的 {relative} 在仓里没有")
                continue
            os.makedirs(os.path.dirname(os.path.join(copy_root, relative)), exist_ok=True)
            os.symlink(source, os.path.join(copy_root, relative))
            linked.append(relative)
for problem in problems:
    print(f"mutate: 副本里读不到装置要的文件：{problem}", file=sys.stderr)
if problems:
    sys.exit(2)
if linked:
    print(f"mutate: 副本根下链了 {len(linked)} 份 scripts/experiments/ 之外的仓内文件（include_str! / include_bytes! 读的）：{'、'.join(linked)}", file=sys.stderr)
PY
then
  echo "        → 怎么办：把那一行改成 include_str!(\"<相对这份源文件、落在仓里的路径>\") 这种一个串字面量的写法，或先让那份文件进仓；一条变异都没跑" >&2
  exit 2
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
# 开跑前算一次，基线、每条变异、还原之后那一次与只编不跑的 check 都经 RUSTFLAGS_ENVIRONMENT 带它
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
if [[ -n "${MUTATE_LIBRARY_DIRECTORY:-}" ]]; then
  MUTATE_LIBRARY="$MUTATE_LIBRARY_DIRECTORY"
elif library_common_directory="$(git -C "$REPOSITORY_OF_WORKSPACE" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$library_common_directory" ]]; then
  MUTATE_LIBRARY="$library_common_directory/singlefs-mutate-library"
else
  MUTATE_LIBRARY="${HOME:?}/.local/share/singlefs/mutate-library"
fi
if ((COMPILE_ONLY)); then
  library_mode=compile; library_cargo_words=(check --profile test --bin "$BIN" ${cargo_feature_arguments[@]+"${cargo_feature_arguments[@]}"})
else
  library_mode=run; library_cargo_words=(test --release --bin "$BIN" ${cargo_feature_arguments[@]+"${cargo_feature_arguments[@]}"})
fi
library_breaks=""
for library_break in libraryunread calleesignored dependenciesbycrate; do is_broken "$library_break" && library_breaks+="${library_breaks:+,}$library_break"; done
# 复用哨兵（文件头「复用哨兵」）：状态放这一轮自己的临时根；判法是 scripts/gate/lib_reuse_checks.py（拷进不带它的树里跑的这一份不抽）
REUSE_CHECKS="$(dirname "$SELF_PATH")/../gate/lib_reuse_checks.py"
REUSE_SENTINEL_STATE="$SHARD_ROOT/reuse-sentinel.json"
if library_summary="$(env ${RUSTFLAGS_ENVIRONMENT[@]+"${RUSTFLAGS_ENVIRONMENT[@]}"} MUTATE_ANCHOR_BROKEN_INDEXES="$ANCHOR_BROKEN_INDEXES" \
      MUTATE_REUSE_CHECKS="$REUSE_CHECKS" MUTATE_REUSE_SENTINEL_STATE="$REUSE_SENTINEL_STATE" \
      python3 - "$MUTATE_LIBRARY_HELPER" "$PWD" "$SRC" "$BIN" "$TABLE" "$library_mode" \
      "$MUTATE_LIBRARY" "$FULL_RUN" "$PLAN_FILE" "$RESULTS" "$CLAIMS" "$ROW_TIME_LIMIT" "$library_breaks" "${library_cargo_words[@]}" <<'PY'
import importlib.util, json, os, sys
helper, root, source, binary, table, mode, library, full, plan_path, results, claims, time_limit, breaks = sys.argv[1:14]
words = sys.argv[14:]
specification = importlib.util.spec_from_file_location("lib_mutation_units_for_mutate", helper)
units = importlib.util.module_from_spec(specification)
sys.modules[specification.name] = units
specification.loader.exec_module(units)
try:
    decisions = units.mutate_plan(root, source, binary, table, mode, words, dict(os.environ), library, full == "1", plan_path, time_limit,
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
    与复用判法（lib_mutation_units.py 的 mutate_decision）分开查：直接读计划文件与记录文件，不经 Library。"""
    with open(plan_path, encoding="utf-8") as handle:
        plan = json.load(handle)
    reusable = {"run": ("caught", "notred", "invalid", "timeout"), "compile": ("compiled", "uncompiled")}[plan["mode"]]
    found = {}
    for index, entry in enumerate(plan["rows"]):
        if index in anchor_broken or not entry.get("key"):
            continue
        try:
            with open(os.path.join(library, "records", entry["key"]), encoding="utf-8", errors="surrogateescape") as handle:
                record = dict(text.split("=", 1) for text in handle.read().splitlines() if "=" in text and not text.startswith("#"))
        except OSError:
            continue
        verdict = record.get("verdict", "")
        if verdict not in reusable or record.get("key") != entry["key"]:
            continue
        if plan["mode"] == "run":
            if verdict == "caught":
                red_keys = dict(item.split("=", 1) for item in record.get("red_test_keys", "").split(",") if "=" in item)
                if not red_keys or any(plan["tests"].get(test) != key for test, key in red_keys.items()):
                    continue
            elif not plan.get("all_tests") or plan["all_tests"] != record.get("all_tests"):
                continue
            elif verdict == "timeout" and plan.get("time_limit") and record.get("time_limit") and int(plan["time_limit"]) > int(record["time_limit"]):
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
    sentinel.plan({} if full == "1" else sentinel_candidates(), reuse=full != "1", off_reason="--full：只写不读",
                  printer=lambda text: print(f"mutate: {text.strip()}", file=sys.stderr))
forced_index = None
for index, name, reuse, verdict, fails, line, reason in decisions:
    if int(index) in anchor_broken:
        continue
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
  echo "mutate: 判定库那一段 python 没跑成（上面是原因），这一趟每条现判、不写库" >&2
  LIBRARY_REUSED=0; LIBRARY_FRESH=0; LIBRARY_REFUSED=0
  rm -f -- "$PLAN_FILE"
fi
if ((LIBRARY_REUSED + LIBRARY_FRESH + LIBRARY_REFUSED + ANCHOR_BROKEN_COUNT != mutation_count)); then
  LIBRARY_FRESH=$((mutation_count - LIBRARY_REUSED - LIBRARY_REFUSED - ANCHOR_BROKEN_COUNT))   # 算不出键的那一趟：没进库的都按现判数（锚点失效的不算）
fi
echo "mutate: 判定库 $MUTATE_LIBRARY：复用 $LIBRARY_REUSED 条、现判 $LIBRARY_FRESH 条、拒算 $LIBRARY_REFUSED 条、锚点失效没跑 $ANCHOR_BROKEN_COUNT 条$( ((FULL_RUN)) && echo "（--full：只写不读）")" >&2

# 一条判完（现判的）写进判定库：档在可复用的那几种里、这一条有键才写（mutate_record 判）；写不成记进 LIBRARY_RECORD_ERRORS，收尾报，不改这一条的判定
record_in_library() { # <行下标> <档> <要不要判失败> <那一行> [红了的测试名，逗号分隔]
  [[ -f "$PLAN_FILE" ]] || return 0
  if ! python3 - "$MUTATE_LIBRARY_HELPER" "$PLAN_FILE" "$@" 2>>"$LIBRARY_RECORD_ERRORS" <<'PY'
import importlib.util, sys
helper, plan, index, verdict, fails, line = sys.argv[1:7]
specification = importlib.util.spec_from_file_location("lib_mutation_units_for_mutate", helper)
units = importlib.util.module_from_spec(specification)
sys.modules[specification.name] = units
specification.loader.exec_module(units)
units.mutate_record(plan, index, verdict, fails, line, sys.argv[7] if len(sys.argv) > 7 else "")
PY
  then
    echo "第 $(($1 + 1)) 条（$2）没写进判定库" >>"$LIBRARY_RECORD_ERRORS"
  fi
}
report_library_record_errors() {
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
      caught|notred|invalid|timeout|compiled|uncompiled) ;;
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
# 不起 cargo 的那一支：剩下的条都按判定库复用（锚点失效的那几条本来就不跑）
ALL_REMAINING_REUSED=0
((LIBRARY_REUSED + ANCHOR_BROKEN_COUNT == mutation_count && mutation_count > 0)) && ALL_REMAINING_REUSED=1

# ── 只编不跑（文件头「只编不跑」）：一份副本、一个编译目录、串行；每条只跑 cargo check，跑完退出，不往下走 cargo test 那一支 ──
CARGO_CHECK_COMMAND=(cargo check --profile test --bin "$BIN" ${cargo_feature_arguments[@]+"${cargo_feature_arguments[@]}"})
if is_broken compilenontest; then   # 弄坏开关：不带 cfg(test) 的那一份（只在测试外用不到的函数报 dead_code，平常跑的 cargo test 不报）
  CARGO_CHECK_COMMAND=(cargo check --release --bin "$BIN" ${cargo_feature_arguments[@]+"${cargo_feature_arguments[@]}"})
fi
# 在第 1 份副本上跑一次 cargo check，cargo 的 stdout 与 stderr 并进这里的 stdout；test 档关 debug_assertions，cfg 与 cargo test --release 相同
check_in_copy() {
  ( cd "$(copy_directory_of 1)" && exec env CARGO_TARGET_DIR="$(target_directory_of 1)" ${RUSTFLAGS_ENVIRONMENT[@]+"${RUSTFLAGS_ENVIRONMENT[@]}"} \
      CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=false "${CARGO_CHECK_COMMAND[@]}" 2>&1 )
}
# 一份 cargo 输出（$1 文件）里第一行 error：rustc 的「error[E…]: …」或「error: …」，打到 stdout，没有打空行
first_error_line() { grep -m1 -E '^error(\[[A-Za-z0-9]+\])?:' -- "$1"; }
run_compile_only() {
  local row_index name row_exit first_error compiled_count=0 invalid_count=0 collected_count=0 baseline_exit verdict_category verdict_fails row_line
  local row_log="$SHARD_ROOT/compile-row.log" anchor_count=0
  if ((ALL_REMAINING_REUSED)); then
    echo "基线：没编（表里 $LIBRARY_REUSED 条按判定库复用、锚点失效 $ANCHOR_BROKEN_COUNT 条不跑，一条都不编）"
  else
    if check_in_copy >"$SHARD_ROOT/baseline-1.log"; then baseline_exit=0; else baseline_exit=$?; fi
    if ((baseline_exit != 0)); then
      echo "mutate: 基线（没改过的源码）就编不过：${CARGO_CHECK_COMMAND[*]} 退 $baseline_exit，一条变异都没编" >&2
      first_error="$(first_error_line "$SHARD_ROOT/baseline-1.log")"
      echo "        第一处错：${first_error:-（输出里没有 error 开头的行）}" >&2
      echo "        输出末尾 15 行：" >&2
      tail -n 15 -- "$SHARD_ROOT/baseline-1.log" | sed 's/^/          /' >&2
      echo "        → 怎么办：在 scripts/experiments/ 里对没改过的源码单独跑 $MUTATE_FLAGS_HINT CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=false ${CARGO_CHECK_COMMAND[*]}，照第一处错修好装置（或补上 feature 依赖要的环境），编得过再只编不跑整张表重跑" >&2
      exit 2
    fi
    echo "基线：编得过"
  fi
  for ((row_index = 0; row_index < mutation_count; row_index++)); do
    name="${mutation_names[row_index]}"
    # 判定库复用得上的那一条（文件头「按条入库」）：打记录里那一行，不编
    if [[ -f "$RESULTS/$row_index.verdict" ]]; then
      [[ -f "$RESULTS/$row_index.line" ]] && cat "$RESULTS/$row_index.line"
      read -r verdict_category verdict_fails <"$RESULTS/$row_index.verdict"
      collected_count=$((collected_count + 1))
      case "$verdict_category" in
        compiled) compiled_count=$((compiled_count + 1)) ;;
        anchor) anchor_count=$((anchor_count + 1)) ;;   # 派活之前就记成锚点失效的那一条（文件头「锚点失效」）
        *) invalid_count=$((invalid_count + 1)) ;;
      esac
      continue
    fi
    if ! anchor_hits="$(write_mutated_source "${mutation_originals[row_index]}" "${mutation_replacements[row_index]}" "$(copy_directory_of 1)/$SRC" 2>&1)"; then
      # 派活之前核过的锚点在这里又对不上：同样记锚点失效、接着编下一条
      row_line="⚓ [$name] 锚点失效：原文在 $SRC 里${anchor_hits%%：*}—— 这一条没编，不算编得过也不算编不过"
      echo "$row_line"
      collected_count=$((collected_count + 1)); anchor_count=$((anchor_count + 1))
      continue
    fi
    if check_in_copy >"$row_log"; then row_exit=0; else row_exit=$?; fi
    cp "$PRISTINE" "$(copy_directory_of 1)/$SRC"   # 还原：下一条从开跑时那一份改
    if is_broken droprow && ((row_index == 1)); then continue; fi   # 弄坏开关：第 2 条的判定丢掉不记
    collected_count=$((collected_count + 1))
    if ((row_exit == 0)) || is_broken compileassuccess; then
      row_line="✅ [$name] 编得过"
      echo "$row_line"
      compiled_count=$((compiled_count + 1))
      record_in_library "$row_index" compiled 0 "$row_line"
      printf 'compiled 0\n' >"$RESULTS/$row_index.verdict"   # 复用哨兵收尾读它（强制现判那一条这一趟的档）
    else
      first_error="$(first_error_line "$row_log")"
      row_line="⏭  [$name] 编不过：${first_error:-（cargo check 退 $row_exit，输出里没有 error 开头的行）}"
      echo "$row_line"
      invalid_count=$((invalid_count + 1))
      record_in_library "$row_index" uncompiled 1 "$row_line"
      printf 'uncompiled 1\n' >"$RESULTS/$row_index.verdict"   # 复用哨兵收尾读它
    fi
  done
  # 派出去（表里）多少条就要收回多少份判定，对不上退 8（文件头「并行」那一段同一道闸）
  if ((collected_count != mutation_count)) && ! is_broken countblind; then
    echo "mutate: 只编不跑：派出去 $mutation_count 条变异，收回 $collected_count 份判定，这一轮的判定不算数" >&2
    echo "        → 怎么办：这是收束的完整性闸红了，去查 run_compile_only 那一段为什么有一条没记判定，修好整张表重跑" >&2
    exit 8
  fi
  echo "计数：派出去 $mutation_count 条，收回 $collected_count 份判定：编得过 $compiled_count 条、编不过 $invalid_count 条、锚点失效 $anchor_count 条$LIBRARY_COUNT_TEXT"
  report_library_record_errors
  finish_reuse_sentinel
  if [[ "$(sha256sum "$SRC" | cut -d' ' -f1)" != "$ORIGINAL_SUM" ]]; then
    echo "mutate: $SRC 在这一轮只编不跑的时候被改过；上面的结果量的是开跑时那一份，改完之后重跑" >&2
    exit 5
  fi
  if ((invalid_count)); then
    echo "mutate: $invalid_count 条替换文编不过（上面 ⏭ 那几行），这几条在平常跑里是无效变异，整轮判失败" >&2
    echo "        → 怎么办：改那几行替换文，让它编得过、仍改被测行为；或把装置改回能编。改完只编不跑再整张表跑一遍：bash ../mutation/mutate.sh --compile-only $BIN $SRC $TABLE" >&2
  fi
  if ((anchor_count)); then
    ANCHOR_BROKEN_COUNT="$anchor_count"
    report_anchor_broken
    exit 3
  fi
  ((invalid_count || REUSE_SENTINEL_RED)) && exit 1
  exit 0
}
if ((COMPILE_ONLY)); then
  run_compile_only
fi

# 全部复用（文件头「按条入库」）：不起 cargo——基线与还原之后那一次都不跑，副本里的源码一个字没改；按表序打记录里那几行，照它们的档判退出码
# （锚点失效的那几条派活之前就记好了，一起按表序打，不为它们起 cargo）
if ((ALL_REMAINING_REUSED)); then
  reused_fail=0; reused_timeout_count=0
  for ((row_index = 0; row_index < mutation_count; row_index++)); do
    [[ -f "$RESULTS/$row_index.line" ]] && cat "$RESULTS/$row_index.line"
    read -r verdict_category verdict_fails <"$RESULTS/$row_index.verdict"
    [[ "$verdict_category" == timeout ]] && reused_timeout_count=$((reused_timeout_count + 1))
    ((verdict_fails)) && reused_fail=1
  done
  echo "计数：内存撞顶 0 条（上限 $MEMORY_MAX）、超时 $reused_timeout_count 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条$ANCHOR_COUNT_TEXT$LIBRARY_COUNT_TEXT"
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

# 经内存包装跑一次 cargo test：$1 副本目录、$2 编译目录、$3 限时秒数（空串不限时：基线与还原之后那一次）、$4 第几个工作进程（写进并行理由）。
# cargo 的 stdout 与 stderr 在 scope 里并成一路打到这里的 stdout（判结局读它）；包装自己的 stderr（排队时每 60 秒那一行、撞顶与超时的说明）
# 打到开跑时的 stderr（WRAPPER_STDERR_FD），工作进程里起的也看得见，不混进判结局的输出。
# 限时交给包装的 RUN_WITH_MEMORY_CAP_TIME_LIMIT：从起跑算、不算排队，到点包装退 253（文件头「每条变异限时」一段）
CARGO_TEST_COMMAND=(cargo test --release --bin "$BIN" ${cargo_feature_arguments[@]+"${cargo_feature_arguments[@]}"})
exec {WRAPPER_STDERR_FD}>&2
run_capped_cargo_test() {
  local copy_directory="$1" target_directory="$2" time_limit="$3" worker_index="$4" wrapper_stderr_fd="$WRAPPER_STDERR_FD"
  local -a limit_environment=(-u RUN_WITH_MEMORY_CAP_TIME_LIMIT) outer_timeout=() parallel_environment=()
  # 会话里的「日常一条」：自己起的每次包装带一句并行理由，调用方设了的原样传、不覆盖；只设在这一次包装的环境里（文件头「并行」那一段）
  if [[ -z "$CALLER_PARALLEL_REASON" ]] || is_broken overridecallerreason; then
    is_broken noparallelreason \
      || parallel_environment=("RUN_WITH_MEMORY_CAP_PARALLEL_REASON=同一次 mutate.sh 的第 $worker_index 个工作进程，各用各的副本与编译目录（mutate.sh 进程 $$、这一轮共 $WORKER_COUNT 个，被测 $BIN）")
  fi
  if [[ -n "$time_limit" ]]; then
    if is_broken outertimeout; then
      outer_timeout=(timeout "$time_limit")      # 弄坏开关：改前的跑法，排队拿锁、起 scope 的时间也算进限时
    else
      limit_environment=("RUN_WITH_MEMORY_CAP_TIME_LIMIT=$time_limit")
    fi
    is_broken swallowwrapper && wrapper_stderr_fd=1   # 弄坏开关：改前的跑法，包装的话并进判结局的输出、看不见
  fi
  ( cd "$copy_directory" && exec env "${limit_environment[@]}" ${parallel_environment[@]+"${parallel_environment[@]}"} \
      ${RUSTFLAGS_ENVIRONMENT[@]+"${RUSTFLAGS_ENVIRONMENT[@]}"} CARGO_TARGET_DIR="$target_directory" RUN_WITH_MEMORY_CAP_KEY="${CARGO_TEST_COMMAND[*]}" \
      ${outer_timeout[@]+"${outer_timeout[@]}"} bash "$MEMORY_CAP_RUNNER" "$MEMORY_MAX" bash -c 'exec "$@" 2>&1' mutate-cargo-test "${CARGO_TEST_COMMAND[@]}" ) \
    2>&"$wrapper_stderr_fd"
}
# 在第 $1 个工作进程的副本上、用它的编译目录跑一次不限时的 cargo test（基线与还原之后那一次）
run_tests_in_copy() { run_capped_cargo_test "$(copy_directory_of "$1")" "$(target_directory_of "$1")" "" "$1"; }

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
  echo "        → 怎么办：在 scripts/experiments/ 里对没改过的源码单独跑一遍 $MUTATE_FLAGS_HINT ${CARGO_TEST_COMMAND[*]}（经 run-with-memory-cap.sh），照上面红的那几条修好，绿了再整张表重跑" >&2
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
  if [[ $baseline_exit -eq $MEMORY_CAP_HIT_EXIT ]]; then
    echo "mutate: 基线（没改过的源码）就撞了内存上限 $MEMORY_MAX${copy_note}：先查被测代码为什么吃这么多，确实要这么多就调大 MUTATE_MEMORY_MAX 再来" >&2; exit 2
  elif [[ $baseline_exit -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]]; then
    echo "mutate: 跑基线时带内存上限的 scope 起不来了${copy_note}，一条变异都没跑；修好用户级 systemd / D-Bus 再来" >&2; exit 7
  elif [[ $baseline_exit -eq $MEMORY_ADMISSION_REFUSED_EXIT ]] && ! is_broken baselinequeueasred; then
    echo "mutate: 基线排不上：第 $worker_index 份副本上内存包装退 $MEMORY_ADMISSION_REFUSED_EXIT，命令一行都没跑（包装的话在上面），一条变异都没跑" >&2
    echo "        → 怎么办：看上面包装列出的占着的那几条——同一个会话已有一条日常的在跑而调用方给的 RUN_WITH_MEMORY_CAP_PARALLEL_REASON 不够 8 个字的，" >&2
    echo "          改长或 unset 它（mutate.sh 自己给每个工作进程设）；内存不够排不上的，等它们跑完再整张表重跑，不许拿掉上限去跑" >&2
    exit 2
  elif [[ $baseline_exit -eq $SLICE_TOTAL_HIT_EXIT ]] && ! is_broken baselinesqueezedasred; then
    echo "mutate: 基线被内存总上限挤掉：第 $worker_index 份副本上内存包装退 $SLICE_TOTAL_HIT_EXIT（包装的话在上面），这一次不算数、一条变异都没跑，原样退 $SLICE_TOTAL_HIT_EXIT" >&2
    echo "        → 怎么办：等占着总上限的那几条跑完，再整张表重跑；不许拿掉上限去跑" >&2
    exit "$SLICE_TOTAL_HIT_EXIT"
  elif [[ $baseline_exit -ne 0 ]]; then
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
  # 抓到、没红、无效、超时四档进按条判定库（文件头「按条入库」；$6 是抓到那一档红了的测试名，逗号分隔）；写在 .verdict 之前：父进程看见 .verdict 时库里已经有它
  case "$2" in
    caught|notred|invalid|timeout) record_in_library "$row_index" "$2" "$3" "$4" "${6:-}" ;;
  esac
  printf '%s %s\n' "$2" "$3" >"$RESULTS/$row_index.verdict.partial"
  mv "$RESULTS/$row_index.verdict.partial" "$RESULTS/$row_index.verdict"
}

# 第 $1 个工作进程判表里第 $2 条：从开跑时那一份源码施加变异、限时跑、判结局，写进结果文件（record_verdict）
judge_mutation() {
  local worker_index="$1" row_index="$2" name copy_directory target_directory out row_exit sig red red_names names_status memory_fails
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
  # ⚠️ **第五种结局：测试根本不终止。** 一条破坏可以让被测代码进死循环——
  # 那时 `cargo test` 永远不返回，整轮变异**挂在这里**，既没有红也没有绿。
  # **挂住不是判红**：它看起来像脚本坏了，而实际发生的事是「这条破坏让代码不终止」。
  # 实测踩过（2026-09-01，E73）：`tree_height` 的循环无界，变异把扇出下界的 guard 改松之后
  # 扇出为 1 时 `cap` 不增长，整轮挂死，只能靠 `kill` 收场。
  # ⇒ 每条变异单独限时。默认 120 秒——本仓最慢的单测不到 1 秒，撞到它就是真挂了。
  #   限时交给包装（RUN_WITH_MEMORY_CAP_TIME_LIMIT，从起跑算），不在包装外面套 timeout：套在外面的把排队的时间也算进去（文件头「每条变异限时」一段）。
  # ⚠️ **第六种结局：内存先于超时撞顶。** 无界分配的破坏几十秒就能吃满整机，等不到 120 秒；2026-09-25 E142 的一条变异两次把整机拖进 OOM。
  # ⇒ 每条变异同时放进内存上限（MUTATE_MEMORY_MAX，见文件头），撞顶与超时同一类记。
  out="$(run_capped_cargo_test "$copy_directory" "$target_directory" "$ROW_TIME_LIMIT" "$worker_index")"
  row_exit=$?
  cp "$PRISTINE" "$copy_directory/$SRC"   # 还原：下一条从开跑时那一份改，领完之后那一次测试也跑在还原过的源码上
  if [[ $row_exit -eq $TIME_LIMIT_HIT_EXIT ]] || { is_broken outertimeout && [[ $row_exit -eq $OUTER_TIMEOUT_EXIT ]]; }; then
    record_verdict "$row_index" timeout 1 "⏱  [$name] 起跑之后 $ROW_TIME_LIMIT 秒没跑完（排队的时间不算）—— 这条破坏让被测代码不终止了，不是「没抓到」" ""
    return
  fi
  if [[ $row_exit -eq $MEMORY_CAP_HIT_EXIT ]]; then
    if is_broken memoryascaught; then
      record_verdict "$row_index" caught 0 "✅ [$name] 红：（弄坏开关 memoryascaught）" ""
      return
    fi
    memory_fails=1
    is_broken memorynotfail && memory_fails=0
    record_verdict "$row_index" memory "$memory_fails" "🧱 [$name] 撞了内存上限 $MEMORY_MAX（内存撞顶）—— 这条破坏让被测代码无界分配，不是「没抓到」，也不算抓到" ""
    return
  fi
  if [[ $row_exit -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]]; then
    touch "$STOP_FLAG"
    record_verdict "$row_index" unavailable 1 "" "mutate: [$name] 跑到一半带内存上限的 scope 起不来了（run-with-memory-cap.sh 的报错在这一轮 stderr 的前面几行），后面的变异不跑
        → 怎么办：修好用户级 systemd / D-Bus 整张表重跑；不许拿掉上限去跑"
    return
  fi
  # 包装退 252：内存不够排不上，命令一行都没跑；退 254：被总上限挤掉，结果不算数（文件头「包装的另两种结局」一段）。
  # 都在下面两支之前判：252 的输出是空的，会落进「编译失败、无效」；254 的输出里有 process didn't exit successfully，会落进「没跑完」
  if [[ $row_exit -eq $MEMORY_ADMISSION_REFUSED_EXIT ]] && ! is_broken queueasinvalid; then
    record_verdict "$row_index" queuefull 1 "⏳ [$name] 内存包装排不上（run-with-memory-cap.sh 退 $MEMORY_ADMISSION_REFUSED_EXIT），命令一行都没跑 —— 这一条没判，不是「无效」，也不算盲区或命中" ""
    return
  fi
  if [[ $row_exit -eq $SLICE_TOTAL_HIT_EXIT ]] && ! is_broken squeezedascrashed; then
    record_verdict "$row_index" squeezed 1 "🚧 [$name] 被内存总上限挤掉（run-with-memory-cap.sh 退 $SLICE_TOTAL_HIT_EXIT：整个 slice 满了，内核挑了这一条杀）—— 结果不算数，不是「破坏被看见了」" ""
    return
  fi
  # 包装退 245：显存不够，结果不算数（文件头「包装的另三种结局」一段）；也在下面几支之前判：测试是被显存不够弄红的，会落进「抓到」
  if [[ $row_exit -eq $GPU_MEMORY_HIT_EXIT ]] && ! is_broken gpumemoryascaught; then
    record_verdict "$row_index" gpumemory 1 "🎴 [$name] 显存不够（run-with-memory-cap.sh 退 $GPU_MEMORY_HIT_EXIT：命令起止之间内核报了 NV_ERR_NO_MEMORY）—— 结果不算数，不是「破坏被看见了」" ""
    return
  fi
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
    [[ -e "$STOP_FLAG" ]] && break
    mkdir "$CLAIMS/$row_index" 2>/dev/null || continue   # 认领：mkdir 是原子的，同一条只有一个工作进程建得成
    judge_mutation "$worker_index" "$row_index"
  done
  [[ -e "$STOP_FLAG" ]] && return 0
  if run_tests_in_copy "$worker_index" >/dev/null; then restore_exit=0; else restore_exit=$?; fi
  echo "$restore_exit" >"$SHARD_ROOT/restore-$worker_index.exit"
}

for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  run_worker "$worker_index" >"$SHARD_ROOT/worker-$worker_index.log" 2>&1 &
  worker_pids+=($!)
done

# 父进程：按表序把已经判完的那几条接着打到 stdout（前面有一条还没判完就停下等它），顺手计数。
# 碰到「scope 起不来」那一条就停，后面的一条都不再打；锚点失效的照打、单数一栏（文件头「锚点失效」）
fail=0
timeout_count=0
memory_hit_count=0
queue_full_count=0
squeezed_count=0
gpu_memory_count=0
anchor_count=0
next_row=0
stopped_verdict=""
print_judged_rows_in_table_order() {
  local verdict_category verdict_fails
  while ((next_row < mutation_count)) && [[ -z "$stopped_verdict" && -f "$RESULTS/$next_row.verdict" ]]; do
    read -r verdict_category verdict_fails <"$RESULTS/$next_row.verdict"
    if [[ "$verdict_category" == unavailable ]]; then
      stopped_verdict="$verdict_category"
      return
    fi
    [[ -f "$RESULTS/$next_row.line" ]] && cat "$RESULTS/$next_row.line"
    [[ "$verdict_category" == anchor ]] && anchor_count=$((anchor_count + 1))
    [[ "$verdict_category" == timeout ]] && timeout_count=$((timeout_count + 1))
    [[ "$verdict_category" == memory ]] && memory_hit_count=$((memory_hit_count + 1))
    [[ "$verdict_category" == queuefull ]] && queue_full_count=$((queue_full_count + 1))
    [[ "$verdict_category" == squeezed ]] && squeezed_count=$((squeezed_count + 1))
    [[ "$verdict_category" == gpumemory ]] && gpu_memory_count=$((gpu_memory_count + 1))
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

if [[ "$stopped_verdict" == unavailable ]]; then
  cat "$RESULTS/$next_row.stderr" >&2
  exit 7
fi

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
# 与超时同一类的四种结局单列计数：它们既不是抓到也不是没红，整轮已经判失败
echo "计数：内存撞顶 $memory_hit_count 条（上限 $MEMORY_MAX）、超时 $timeout_count 条、排不上 $queue_full_count 条、被总上限挤掉 $squeezed_count 条、显存不够 $gpu_memory_count 条、锚点失效 $anchor_count 条$LIBRARY_COUNT_TEXT"
report_library_record_errors
finish_reuse_sentinel
((REUSE_SENTINEL_RED)) && fail=1
if ((queue_full_count + squeezed_count + gpu_memory_count)); then
  echo "mutate: $queue_full_count 条排不上（包装退 $MEMORY_ADMISSION_REFUSED_EXIT）、$squeezed_count 条被总上限挤掉（包装退 $SLICE_TOTAL_HIT_EXIT）、$gpu_memory_count 条显存不够（包装退 $GPU_MEMORY_HIT_EXIT），这几条没判，整轮判失败" >&2
  echo "        → 怎么办：bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 里谁占着，等它们跑完再整张表重跑；不许拿掉上限去跑" >&2
fi

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
    echo "mutate: 还原后没回到全绿（第 $worker_index 份副本，退出码 $restore_exit；$MEMORY_CAP_HIT_EXIT 是撞了内存上限，$MEMORY_CAP_UNAVAILABLE_EXIT 是带上限的 scope 起不来，$MEMORY_ADMISSION_REFUSED_EXIT 是排不上，$SLICE_TOTAL_HIT_EXIT 是被总上限挤掉）" >&2
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
