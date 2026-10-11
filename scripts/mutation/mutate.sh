#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command cargo git python3
# 证明「这些测试会红」：逐条注入一个已知的破坏，跑测试，记下红了哪几个，然后还原。
# 红了哪几个（libtest 的「test <名字>[ - should panic] ... FAILED」行）与汇总行报没报失败，经 scripts/gate/lib_libtest_result_lines.py failed-names 取（全仓一份解析）：
#   #[should_panic] 的用例那一段「 - should panic」不进名字；汇总行报了失败而一个名字都没取到，它退 3，这里记「抓名字的规则有盲区」。
#
#   mutate.sh <bin 名> <源文件> <变异表>      # 在 research/ 里跑；参数不够三个退 2，打用法与一条例子
#
# 变异表每行三段，用制表符分隔：变异名 <TAB> 原文 <TAB> 替换文（原文/替换文里的 \n 表示换行）
# 替换必须命中，命中数为 0 直接报错退出 —— 静默失效的替换会让整份证明变成假的
# （singlefs-ai-sop/rules/command-safety.md「脚本改文件之后要回读确认」）。
#
#   mutate.sh --selftest    拿一个假的 cargo 走一遍各种结局（抓到、没红、无效、超时、内存撞顶、基线撞顶、scope 起不来），
#                           再走一遍并行的几样（进程数不同 stdout 逐字相同、各用各的副本、丢一份结果整道判红、锚点腐化派活之前退 3），
#                           再走一遍 required-features（挂了的 bin 带上 --features 跑得起来，没挂的 bin 的 cargo 参数与改前逐字相同），
#                           再走一遍包装排队（包装每次放行前多停 2 秒，五种结局照样各归各类：限时不算排队的时间），
#                           再走一遍包装的另三种结局（假包装退 252 排不上、退 254 被总上限挤掉、退 245 显存不够：各归各类、整轮判失败），
#                           再走一遍平常跑带只供自证的变量（每个变量各一次，都退 2、一次 cargo 都不调），
#                           再走一遍会话里平常跑并行（不经自证的标记：一份原样副本配假包装，照包装的「日常一条」判，账上先放一条这个会话在跑的）：
#                           MUTATE_JOBS=4 照样各条判对、整轮退 0，每次包装带各自的并行理由、--check 不带，调用方设了的理由不覆盖，基线排不上报「基线排不上」。
#                           每一例各用一份自己的临时目录：假 cargo、假 workspace、编译目录、包装的锁与账（RUN_WITH_MEMORY_CAP_STATE_DIR）
#                           与峰值表（RUN_WITH_MEMORY_CAP_PEAKS）都在里面，不读别的例留下的东西，也不读写主仓的峰值表
#
# bin 在 Cargo.toml 的 [[bin]] 里挂了 required-features 的：每次 cargo test 带 `--features <包名>/<feature>,…`，
#   feature 从那一节现读，不在命令行上手敲、不写死在这里；没挂的 bin，cargo 的参数与改前逐字相同（`test --release --bin <名>`）。
#   不带的话 cargo 不编这个 bin、直接退 101，基线停在「基线就是红的」，一条变异都没跑。
#   那几个 feature 的依赖要的环境（E162 的 rocksdb 要 BINDGEN_EXTRA_CLANG_ARGS）由调用方设，原样传给 cargo。
#
# gate-overlap:copy-kept scripts/mutation/crates-mutation-rows.py 开几个工作进程（worker_count_and_head_line）照那边的 worker_count_for 写：各项取最小、至少 1、核数一半至多 16；两边的项不同（这里 MUTATE_JOBS 只能压小、第二项是表里的条数、另算每个 cargo 的编译并行度；那边 GATE_MUTATION_WORKERS 顶掉核数那一项、第二项是要现跑的条数），派活一边是 bash 子 shell 按目录认领、一边是 python 进程池，共用的只剩一个取最小值，抽成共用要这里每趟多起一个 python 只为算一个数、两边头一行的字也不同
# 并行（command-safety.md「一个脚本里的检测项，能并行就并行」，形态照 .claude/gate.d/checker-tier-crates-mutation-replay.sh 与它的判法 scripts/mutation/crates-mutation-rows.py）：
#   每个工作进程一份被测装置的副本（research/ 里编译要用的那部分）、一个自己的 CARGO_TARGET_DIR，不在同一份源码上并发改：
#   第 1 个用 MUTATE_TARGET_DIR（跨轮热着），第 i 个用它后面加 -w<i>（跨轮复用，第一次要冷编译）。
#   工作进程数取 min(核数的一半且至多 16、表里的条数)，至少 1；MUTATE_JOBS 只能压小、不能加大（设成 1 就是原来的串行跑法）。
#   实际开几个、各项给几个、卡在哪一项、每个 cargo 的编译并行度，打在 stderr 的头一行。
#   内存不在这里判：每条 cargo test 照旧经 run-with-memory-cap.sh 跑，几条同时跑要多少、放不放得下由它管，这里不按内存收工作进程数。
#   每个 cargo 的编译并行度：先用 MUTATE_CARGO_JOBS，其次调用方已经设了的 CARGO_BUILD_JOBS，都没有就取 核数 ÷ 工作进程数（至少 1），免得 N 个 cargo 各按核数开 rustc。
#   基线在每份副本上各跑一次（第 1 份那次就是原来的基线，其余几份顺带把各自的编译目录热起来，不占每条变异的限时），还原之后那一次也在每份副本上各跑一次。
#   锚点在派活之前逐条核（每条原文在源码里恰好命中一次），有一条不是就退 3，一条变异都不跑。
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
#   2 用法、环境或基线不绿（基线排不上也退 2，报「基线排不上」），或平常跑时带了只供自证的变量（见「只供自证」）；3 锚点不是恰好命中一次；
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
#       caught-and-memory（「只有一条抓到、一条内存撞顶的表应当判失败退 1」）、queued-admission（「包装每次放行前停 2 秒时……输出里应当有」缺「🧱 [撞内存]」）
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
#       queued-admission（「包装每次放行前停 2 秒时……」：抓到、没红、无效、撞内存那几条都被记成「⏱ 超时」）；
#       别的例里包装自己慢过 2 秒的那几条也会被记成超时，红不红不定
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
  # 每条变异的限时（秒）；queued-admission 那一例让包装每次放行前停同样的秒数，限时套在包装外面（outertimeout）时那几条必然先到点
  local selftest_time_limit=2
  # 自证里的包装：每例自己的锁、账与峰值表，与主仓的峰值表和别的会话的账互不读写。
  # 峰值表从空的起，基线记下的峰值只有几 M，之后给 64M 就超过它的 2 倍：撞内存那一条要的正是 64M，理由写在这里。
  # 几个工作进程在同一例里同时起包装：同一个会话里起的「日常一条」靠 mutate.sh 自己给每次包装设的并行理由放行，这里不替它设（设了就把它盖住）；
  # 排队等得下就等，不因为调用方是不是会话而改判。
  local -a selftest_wrapper_environment=(
    "RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON=自证里假 cargo 一律给 64M 上限，撞内存那一条就是要撞它"
    "RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600"
  )
  local case_script="$SELF_PATH"   # run_case 跑哪一份 mutate.sh：会话里平常跑并行那一组换成自己的副本
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

  # 建一例自己的目录 $scratch/<标签>/：假 cargo、假 workspace（research/）、几张变异表、包装的锁与账。$1 标签；
  # $2 被测源码里写的 BEHAVIOR（默认 PASS）；$3 bin 那一节追加的一行（默认不加，required-features 那一例给它挂 feature）
  prepare_case() {
    local case_directory="$scratch/$1" behavior="${2:-PASS}" bin_extra_line="${3:-}"
    mkdir -p "$case_directory/bin" "$case_directory/research/bench/src/bin" "$case_directory/scripts/mutation/tables" "$case_directory/memory-cap-state"
    cat >"$case_directory/bin/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
# 假 cargo：只认 mutate.sh 调的 `cargo test --release --bin <名字>`，按被测源码里的 BEHAVIOR 演一种结局；
# 每次被调记一行「当前目录|CARGO_TARGET_DIR|BEHAVIOR|参数」（并行的自证拿前两段数各用了几份副本、几个编译目录）
behavior="$(sed -n 's/.*BEHAVIOR: \([A-Z]*\).*/\1/p' bench/src/bin/fake_bench.rs | head -1)"
echo "$PWD|${CARGO_TARGET_DIR:-}|$behavior|$*" >>"$FAKE_CARGO_LOG"
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
  RED) printf 'running 2 tests\ntest tests::adds ... FAILED\ntest tests::subtracts ... ok\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
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
    printf '[package]\nname = "bench"\n\n[[bin]]\nname = "fake-bench"\npath = "src/bin/fake_bench.rs"\n' >"$case_directory/research/bench/Cargo.toml"
    [[ -n "$bin_extra_line" ]] && printf '%s\n' "$bin_extra_line" >>"$case_directory/research/bench/Cargo.toml"
    printf '// 注释行\nfn main() {} // BEHAVIOR: %s\n' "$behavior" >"$case_directory/research/bench/src/bin/fake_bench.rs"
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
  }

  # 在 prepare_case 建好的那一例目录里跑一次 mutate.sh：$1 标签、$2 变异表，其余是额外的环境变量；
  # stdout 落 $scratch/<标签>.stdout、stderr 落 .stderr、两样接起来落 .out，退出码落 .rc，假 cargo 的调用记录落 .cargo
  run_case() {
    local label="$1" table="$2" case_directory="$scratch/$1"
    shift 2
    : >"$scratch/$label.cargo"
    if (cd "$case_directory/research" && env PATH="$case_directory/bin:$PATH" FAKE_CARGO_LOG="$scratch/$label.cargo" MUTATE_MEMORY_MAX=64M \
          MUTATE_TIMEOUT="$selftest_time_limit" MUTATE_TARGET_DIR="$case_directory/target" \
          RUN_WITH_MEMORY_CAP_STATE_DIR="$case_directory/memory-cap-state" RUN_WITH_MEMORY_CAP_PEAKS="$case_directory/memory-peaks.tsv" \
          "${selftest_wrapper_environment[@]}" "$@" bash "$case_script" fake-bench bench/src/bin/fake_bench.rs "$table" \
          >"$scratch/$label.stdout" 2>"$scratch/$label.stderr"); then
      echo 0 >"$scratch/$label.rc"
    else
      echo $? >"$scratch/$label.rc"
    fi
    cat "$scratch/$label.stderr" "$scratch/$label.stdout" >"$scratch/$label.out"
  }
  fail() { echo "  ✗ 自检：$1"; failures=$((failures + 1)); }   # gate-lint:detail

  prepare_case every-outcome
  run_case every-outcome mutations/every-outcome.tsv
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
  run_case caught-and-memory mutations/caught-and-memory.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/caught-and-memory.rc")"
  [[ "$status" == 1 ]] || fail "只有一条抓到、一条内存撞顶的表应当判失败退 1（撞顶与超时同类），实际退 $status：$(cat "$scratch/caught-and-memory.out")"

  # 没挂 required-features 的 bin、上限之内正常的变异：照旧判抓到、整轮退 0，每次 cargo 的参数与改前逐字相同（这一例自己的调用记录）
  prepare_case caught-only
  run_case caught-only mutations/caught-only.tsv
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
  run_case unavailable mutations/caught-only.tsv DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent
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
  run_case serial mutations/every-outcome.tsv MUTATE_JOBS=1
  prepare_case parallel
  run_case parallel mutations/every-outcome.tsv MUTATE_JOBS=4
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
  run_case lost-result mutations/three-caught.tsv MUTATE_JOBS=3 MUTATE_BREAK="droprow${BROKEN_JUDGEMENT:+,$BROKEN_JUDGEMENT}"
  checked=$((checked + 1))
  status="$(cat "$scratch/lost-result.rc")"
  if [[ "$status" != 8 ]] || ! grep -qF "派出去 3 条" "$scratch/lost-result.stderr" || ! grep -qF "收回 2 份" "$scratch/lost-result.stderr"; then
    fail "丢了一份判定时应当退 8 并报「派出去 3 条……收回 2 份」，实际退 $status：$(cat "$scratch/lost-result.out")"
  fi
  prepare_case lost-result-unchecked
  run_case lost-result-unchecked mutations/three-caught.tsv MUTATE_JOBS=3 MUTATE_BREAK=droprow,countblind
  checked=$((checked + 1))
  status="$(cat "$scratch/lost-result-unchecked.rc")"
  if [[ "$status" != 0 || "$(grep -c '^✅' "$scratch/lost-result-unchecked.stdout")" != 2 ]]; then
    fail "拿掉收束的计数闸之后，同一个丢法应当静默退 0、只报 2 条抓到（这样上一项的退 8 才是那道闸给的），实际退 $status：$(cat "$scratch/lost-result-unchecked.out")"
  fi

  # MUTATE_JOBS 不是正整数：退 2，一次 cargo 都不调
  prepare_case bad-jobs
  run_case bad-jobs mutations/caught-only.tsv MUTATE_JOBS=0
  checked=$((checked + 1))
  status="$(cat "$scratch/bad-jobs.rc")"
  if [[ "$status" != 2 || -s "$scratch/bad-jobs.cargo" ]] || ! grep -qF "MUTATE_JOBS" "$scratch/bad-jobs.stderr"; then
    fail "MUTATE_JOBS=0 应当退 2、点名 MUTATE_JOBS、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/bad-jobs.cargo") 次：$(cat "$scratch/bad-jobs.out")"
  fi

  # MUTATE_TIMEOUT 不是正整数秒（包装的 RUN_WITH_MEMORY_CAP_TIME_LIMIT 只认正整数秒）：开跑之前退 2，一次 cargo 都不调，
  # 不等基线跑完、第一条变异时才被包装按用法错退 251、报成「scope 起不来」
  prepare_case bad-timeout
  run_case bad-timeout mutations/caught-only.tsv MUTATE_TIMEOUT=1.5
  checked=$((checked + 1))
  status="$(cat "$scratch/bad-timeout.rc")"
  if [[ "$status" != 2 || -s "$scratch/bad-timeout.cargo" ]] || ! grep -qF "MUTATE_TIMEOUT" "$scratch/bad-timeout.stderr"; then
    fail "MUTATE_TIMEOUT=1.5 应当退 2、点名 MUTATE_TIMEOUT、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/bad-timeout.cargo") 次：$(cat "$scratch/bad-timeout.out")"
  fi

  # 锚点腐化：派活之前就退 3，连基线都不跑、一条判定都不打（并行下不能让前面几条先跑了、后面那条才炸）
  prepare_case stale-anchor
  run_case stale-anchor mutations/stale-anchor.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/stale-anchor.rc")"
  if [[ "$status" != 3 || -s "$scratch/stale-anchor.cargo" || -s "$scratch/stale-anchor.stdout" ]] \
     || ! grep -qF "[锚点腐化] 替换没命中" "$scratch/stale-anchor.stderr"; then
    fail "锚点腐化的表应当在派活之前退 3、点名那一条、一次 cargo 都不调、stdout 一行都没有，实际退 $status、调了 $(wc -l <"$scratch/stale-anchor.cargo") 次：$(cat "$scratch/stale-anchor.out")"
  fi

  # ── required-features ──
  # 挂了两个 feature 的 bin：从清单现读、带「包名/feature」跑，基线绿、变异照常判抓到；
  # 不读（MUTATE_BREAK=nofeatures，改前的跑法）假 cargo 照真 cargo 退 101，停在基线、退 2
  prepare_case required-features PASS 'required-features = ["fake-feature", "fake-other"]'
  run_case required-features mutations/caught-only.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/required-features.rc")"
  unexpected_arguments="$(cut -d'|' -f4- "$scratch/required-features.cargo" | grep -vxF 'test --release --bin fake-bench --features bench/fake-feature,bench/fake-other')"
  if [[ "$status" != 0 || ! -s "$scratch/required-features.cargo" || -n "$unexpected_arguments" ]] \
     || ! grep -qF "✅ [抓到] 红：tests::adds" "$scratch/required-features.stdout"; then
    fail "挂了 required-features 的 bin 应当每次带「--features bench/fake-feature,bench/fake-other」跑、抓到那一条、退 0，实际退 $status、对不上的参数：${unexpected_arguments:-无}：$(cat "$scratch/required-features.out")"
  fi

  # 基线（没改的源码）就撞上限：退 2 并说是内存上限，不说成「基线就是红的」。
  # 表里的原文要在 HOG 版源码里命中得上（锚点在基线之前核，命中不上就先退 3 了），所以用只改注释的那张表
  prepare_case baseline-memory HOG
  run_case baseline-memory mutations/comment-only.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/baseline-memory.rc")"
  if [[ "$status" != 2 ]] || ! grep -q '基线.*撞了内存上限 64M' "$scratch/baseline-memory.out"; then
    fail "基线就撞上限时应当退 2 并报「基线……撞了内存上限 64M」，实际退 $status：$(cat "$scratch/baseline-memory.out")"
  fi

  # 基线就是红的（没改的源码上假 cargo 演 RED）：退 2，点出红的测试名、打末尾几行，整份基线输出留在 TMPDIR 下并点出路径
  prepare_case baseline-red RED
  mkdir -p "$scratch/baseline-red/tmp"
  run_case baseline-red mutations/comment-only.tsv TMPDIR="$scratch/baseline-red/tmp"
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
  run_case should-panic mutations/should-panic.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/should-panic.rc")"
  if [[ "$status" != 0 ]] || ! grep -qxF "✅ [抓到] 红：tests::answer_panics" "$scratch/should-panic.stdout"; then
    fail "#[should_panic] 的用例红了应当记「✅ [抓到] 红：tests::answer_panics」（名字不带「 - should panic」）、整轮退 0，实际退 $status：$(cat "$scratch/should-panic.out")"
  fi
  prepare_case baseline-red-should-panic PANICS
  mkdir -p "$scratch/baseline-red-should-panic/tmp"
  run_case baseline-red-should-panic mutations/comment-only.tsv TMPDIR="$scratch/baseline-red-should-panic/tmp"
  checked=$((checked + 1))
  status="$(cat "$scratch/baseline-red-should-panic.rc")"
  if [[ "$status" != 2 ]] || ! grep -qxF "        红的测试：tests::answer_panics" "$scratch/baseline-red-should-panic.stderr"; then
    fail "基线里 should panic 的用例红了，应当退 2、点出红的测试「tests::answer_panics」（不带「 - should panic」），实际退 $status：$(cat "$scratch/baseline-red-should-panic.out")"
  fi

  # 参数不够三个：退 2、打用法与一条例子，不报 unbound variable，一次 cargo 都不调
  prepare_case usage
  : >"$scratch/usage.cargo"
  if (cd "$scratch/usage/research" && env PATH="$scratch/usage/bin:$PATH" FAKE_CARGO_LOG="$scratch/usage.cargo" \
        bash "$case_script" fake-bench >"$scratch/usage.stdout" 2>"$scratch/usage.stderr"); then
    echo 0 >"$scratch/usage.rc"
  else
    echo $? >"$scratch/usage.rc"
  fi
  checked=$((checked + 1))
  status="$(cat "$scratch/usage.rc")"
  if [[ "$status" != 2 ]] || ! grep -qF "要三个参数，给了 1 个" "$scratch/usage.stderr" \
     || ! grep -qF "→ 怎么办：在 research/ 目录里照这个例子跑：bash scripts/mutation/mutate.sh " "$scratch/usage.stderr" \
     || grep -qF "unbound variable" "$scratch/usage.stderr" || [[ -s "$scratch/usage.cargo" ]]; then
    fail "参数不够三个时应当退 2、打用法与一条例子、不报 unbound variable、一次 cargo 都不调，实际退 $status：$(cat "$scratch/usage.stderr")"
  fi

  # ── 包装排队 ──
  # 包装每次放行前多停 $selftest_time_limit 秒（RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE，拿着这一例自己的锁停），与每条变异的限时一样长：
  # 限时从命令起跑算、不算排队，五种结局照样各归各类，只有 HANG 那一条记超时；限时套在包装外面（outertimeout，改前的跑法）时排队就吃满限时，
  # 抓到、没红、无效、撞内存几条全被记成超时。一个工作进程，免得几条在同一把锁上串着停、把这一例拖长
  prepare_case queued-admission
  run_case queued-admission mutations/every-outcome.tsv MUTATE_JOBS=1 RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE="$selftest_time_limit"
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
    run_case "$outcome_label" mutations/caught-only.tsv MUTATE_SELFTEST_MEMORY_CAP_RUNNER="$scratch/fake-memory-cap.sh" \
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
  run_case baseline-squeezed mutations/caught-only.tsv MUTATE_SELFTEST_MEMORY_CAP_RUNNER="$scratch/fake-memory-cap.sh" \
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
    mkdir -p "$tooling/scripts" "$scratch/$label/daily-ledger"
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
  run_daily_case daily-parallel mutations/four-caught.tsv MUTATE_JOBS=4
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
  run_daily_case daily-caller-reason mutations/caught-only.tsv MUTATE_JOBS=4 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="$caller_reason"
  status="$(cat "$scratch/daily-caller-reason.rc")"
  daily_log="$scratch/daily-caller-reason.daily"
  checked=$((checked + 1))
  admitted_reasons="$(sed -n 's/^admitted|//p' "$daily_log")"
  unmatched_reasons="$(grep -vxF -- "$caller_reason" <<<"$admitted_reasons")"
  if [[ "$status" != 0 || -z "$admitted_reasons" || -n "$unmatched_reasons" ]] || grep -q '^refused|' "$daily_log"; then
    fail "调用方设了的并行理由应当原样传给包装（每次都是「$caller_reason」）、照常判抓到退 0，实际退 $status、假包装的记录：$(cat "$daily_log")"
  fi

  # 调用方给的理由不够 8 个字（不覆盖）：基线就被拒，报「基线排不上」、退 2、一次 cargo 都不调，不说成「基线就是红的」
  run_daily_case daily-baseline-refused mutations/caught-only.tsv RUN_WITH_MEMORY_CAP_PARALLEL_REASON="$short_caller_reason"
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
    run_case "$outside_label" mutations/caught-only.tsv MUTATE_SELFTEST_MARK= "$variable=$variable_value"
    status="$(cat "$scratch/$outside_label.rc")"
    checked=$((checked + 1))
    if [[ "$status" != 2 || -s "$scratch/$outside_label.cargo" ]] || ! grep -qF "只给自证用的变量" "$scratch/$outside_label.stderr" \
       || ! grep -qF "$variable" "$scratch/$outside_label.stderr"; then
      fail "平常跑时带了只供自证的变量 $variable 应当退 2、点名它、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/$outside_label.cargo") 次：$(cat "$scratch/$outside_label.out")"
    fi
  done

  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ mutate.sh 自检 $failures 处不对（查了 $checked 项）"
    echo "  → 怎么办：照上面逐条改——判结局的在 judge_mutation（退出码 $TIME_LIMIT_HIT_EXIT / $MEMORY_CAP_HIT_EXIT / $MEMORY_CAP_UNAVAILABLE_EXIT / $MEMORY_ADMISSION_REFUSED_EXIT / $SLICE_TOTAL_HIT_EXIT 的分支），"
    echo "    限时与包装 stderr 的去向在 run_capped_cargo_test，基线与收尾计数在主流程，"
    echo "    并行的在 worker_count_and_head_line、run_worker 与 print_judged_rows_in_table_order，required-features 在 read_required_features；"
    echo "    平常跑拒只供自证的变量在 --selftest 那一支后面（running_under_selftest），并行理由在 run_capped_cargo_test（CALLER_PARALLEL_REASON），基线排不上、基线被总上限挤掉在基线那一段；"
    echo "    MUTATE_BREAK 或 RUN_WITH_MEMORY_CAP_BREAK 设着的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ mutate.sh 自检通过（查了 $checked 项：抓到、没红、无效、超时、内存撞顶五种结局各归各类，撞顶不记成抓到或没红、整轮判失败并单列计数，\
包装自己的话打在 stderr 上，上限之内正常的变异照旧判抓到退 0，scope 起不来退 7 且一次 cargo 都没跑，基线撞顶退 2 并说是内存上限，基线被总上限挤掉原样退 254；\
基线红退 2 并点出红的测试、末尾几行与留下的整份日志，should panic 的用例红了（变异那一趟与基线那一趟）名字不带「 - should panic」，参数不够退 2 打用法与例子、一次 cargo 都不调；\
并行：MUTATE_JOBS=1 与 =4 的 stdout 逐字相同、没红的照样判失败、头一行报开了几个、各工作进程各用一份副本与编译目录，\
丢一份判定退 8 而拿掉那道闸就静默判绿，MUTATE_JOBS、MUTATE_TIMEOUT 写错退 2，锚点腐化派活之前退 3；\
required-features：没挂的 bin 的 cargo 参数逐字不变，挂了两个的带「包名/feature」跑得起来、照常判抓到；\
包装每次放行前停 $selftest_time_limit 秒，五种结局照样各归各类、只有 HANG 那一条记超时；\
包装退 252 记排不上、退 254 记被总上限挤掉、退 245 记显存不够，都整轮判失败并单列计数；平常跑带只供自证的 ${#TEST_ONLY_VARIABLES[@]} 个变量各一次都退 2；\
会话里平常跑并行（假包装照「日常一条」判）：MUTATE_JOBS=4 各条判对退 0、每次包装带各自的并行理由而 --check 不带，调用方的理由不覆盖，基线排不上退 2 报「基线排不上」）"
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
# 参数不够三个：打用法与一条例子，退 2（文件头「退出码」：2 是用法错），不让 set -u 报 unbound variable
if (($# < 3)) && ! is_broken bareusage; then
  echo "mutate: 要三个参数，给了 $# 个；用法：mutate.sh <bin 名> <源文件> <变异表>" >&2
  echo "        <bin 名> 是 Cargo.toml 里 [[bin]] 的 name（没写 [[bin]] 的是源文件名去掉 .rs）；<源文件> 与 <变异表> 都相对当前目录（research/）" >&2
  echo "        → 怎么办：在 research/ 目录里照这个例子跑：bash scripts/mutation/mutate.sh e172-deletion-write-dry-run scripts/experiments/e172_deletion_write_dry_run.rs mutations/e172_deletion_write_dry_run.tsv" >&2
  exit 2
fi
BIN="$1"; SRC="$2"; TABLE="$3"

# ⚠️ **先确认「改的文件」与「跑的二进制」对得上。**
# 不确认的话，变异会改 X 而跑 Y 的测试：一条真能被抓的破坏被报成盲区
# （2026-08-29 实测踩过：`e7_index.rs` 配上 crate 主二进制的名字，
# 三条变异全被误报成盲区，换对名字后三条全被抓）。
# 反方向更坏：Y 的测试恰好因别的原因红了，会被记成「变异被抓」。
# 清单：从源文件所在目录一层层往上找最近的 Cargo.toml（实验工作区根是 scripts/experiments，src/bin 布局与自带清单的子 crate 同样认得）
_manifest="" _manifest_directory="$(dirname "$SRC")"
while [[ -z "$_manifest" ]]; do
  if [[ -f "$_manifest_directory/Cargo.toml" ]]; then _manifest="$_manifest_directory/Cargo.toml"
  elif [[ "$_manifest_directory" == . || "$_manifest_directory" == / ]]; then _manifest="Cargo.toml"
  else _manifest_directory="$(dirname "$_manifest_directory")"; fi
done
_declared="$(awk -v src="$SRC" '
  /^\[\[bin\]\]/ { name=""; path=""; next }
  /^name *=/ { gsub(/.*= *"|"/,""); name=$0; next }
  /^path *=/ { gsub(/.*= *"|"/,""); path=$0;
               if (src ~ path"$") print name; next }
' "$_manifest" 2>/dev/null | head -1)"
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
# 写成带包名的形式：research/ 是虚拟 workspace，在它的根上不带包名的 --features 要靠被选中的包碰巧有那个 feature
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
# 这几分钟里仓里那份源码是坏的——几个会话共写一个仓时，别的会话此刻跑门禁（checker-tier-research-build-and-replay 阶段会编译并跑 research 的全部单测）
# 就编到被改坏的源码，红得莫名其妙、还可能当成自己改坏的；此刻有人整份提交这个文件，提交进去的就是变异。
# ⇒ 把 research/ 里编译要用的部分（workspace、crate 源码、include_str! 读的 results/、data/）拷到临时目录，
#   在那里改、在那里编；CARGO_TARGET_DIR 单独一份，不碰 scripts/experiments/target 里别人要用的二进制。跑完删掉副本。
#   并行时每个工作进程各一份这样的副本、各一个编译目录（文件头「并行」那一段）。
if [[ ! -f Cargo.toml ]] || ! grep -q '^\[workspace\]' Cargo.toml; then
  echo "mutate: 要在 research/ 下跑（那里才有 workspace 的 Cargo.toml）" >&2; exit 2
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
# 开跑之前先试一次带上限的 scope 起不起得来：起不来就一条都不跑，不退回无上限（run-with-memory-cap.sh 自己把原因打到 stderr）
if bash "$MEMORY_CAP_RUNNER" --check "$MEMORY_MAX"; then memory_cap_check=0; else memory_cap_check=$?; fi
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
export CARGO_BUILD_JOBS="$CARGO_JOBS_PER_WORKER"

SHARD_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-mutate-XXXXXX")"
PRISTINE="$SHARD_ROOT/pristine-source"   # 开跑时那一份被测源码，每条变异都从它改、每次还原都拷它
CLAIMS="$SHARD_ROOT/claims"              # <行号>/：那一条被哪个工作进程认领了（mkdir 建成的那个领走）
RESULTS="$SHARD_ROOT/results"            # <行号>.line（要打到 stdout 的那一行）、.stderr、.verdict（「档 要不要判失败」，最后写）
STOP_FLAG="$SHARD_ROOT/stop"             # 有一条判出「scope 起不来」或锚点没命中：其余工作进程领完手上那条就不再领
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
copy_directory_of() { echo "$SHARD_ROOT/copy-$1"; }
target_directory_of() { if (($1 == 1)); then echo "$MUTATE_TARGET"; else echo "$MUTATE_TARGET-w$1"; fi; }

if ! rsync -a --exclude target --exclude prompts --exclude mutations --exclude scripts ./ "$(copy_directory_of 1)/"; then
  echo "mutate: 拷副本失败" >&2; exit 2
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
# 锚点在派活之前逐条核：并行下不能让前面几条先跑了、后面那条才炸（mutation-sampling.md 第七类：锚点腐化那一轮一条都不算数）
for ((row_index = 0; row_index < mutation_count; row_index++)); do
  if ! write_mutated_source "${mutation_originals[row_index]}" "${mutation_replacements[row_index]}" ""; then
    echo "mutate: [${mutation_names[row_index]}] 替换没命中，证明作废（锚点在派活之前逐条核，这一轮一条变异都没跑）" >&2
    echo "        → 怎么办：把变异表里这一条的原文改到源码今天的写法、让它恰好命中一次，再整张表重跑" >&2
    exit 3
  fi
done

# 第 2 份起的副本从第 1 份拷：同一份源码快照，不在拷的间隙里读到别人刚改的工作区
for ((worker_index = 2; worker_index <= WORKER_COUNT; worker_index++)); do
  if ! rsync -a "$(copy_directory_of 1)/" "$(copy_directory_of "$worker_index")/"; then
    echo "mutate: 拷第 $worker_index 份副本失败" >&2; exit 2
  fi
done
# ⚠️ rsync -a 保留源码的旧 mtime，而编译目录跨轮共用：上一轮中途退出（替换没命中 exit 3）时
# 那里留着一份**打着变异**编出来的二进制，它比副本里的源码新 ⇒ cargo 判「不用重编」，基线直接跑变异版。
# 实测（2026-09-12）：e114 在 M6 处退出后，下一轮基线连报两次「基线就是红的」，而单测直接跑 14 条全绿。
# 先 touch 每份副本里的被测源码，逼 cargo 按这一轮的副本重编（每个工作进程的编译目录都跨轮留着，每份都要 touch）。
for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  touch "$(copy_directory_of "$worker_index")/$SRC"
done
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
      CARGO_TARGET_DIR="$target_directory" RUN_WITH_MEMORY_CAP_KEY="${CARGO_TEST_COMMAND[*]}" \
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
  echo "        → 怎么办：在 research/ 里对没改过的源码单独跑一遍 ${CARGO_TEST_COMMAND[*]}（经 run-with-memory-cap.sh），照上面红的那几条修好，绿了再整张表重跑" >&2
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
    touch "$STOP_FLAG"
    record_verdict "$row_index" anchor 1 "" "$(cat "$RESULTS/$row_index.anchor")
mutate: [$name] 替换没命中，证明作废
        → 怎么办：把变异表里这一条的原文改到源码今天的写法、让它恰好命中一次，再整张表重跑"
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
    record_verdict "$row_index" caught 0 "✅ [$name] 红：$red" ""
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
# 碰到「scope 起不来」或锚点没命中那一条就停：记下是哪一种，后面的一条都不再打
fail=0
timeout_count=0
memory_hit_count=0
queue_full_count=0
squeezed_count=0
gpu_memory_count=0
next_row=0
stopped_verdict=""
print_judged_rows_in_table_order() {
  local verdict_category verdict_fails
  while ((next_row < mutation_count)) && [[ -z "$stopped_verdict" && -f "$RESULTS/$next_row.verdict" ]]; do
    read -r verdict_category verdict_fails <"$RESULTS/$next_row.verdict"
    if [[ "$verdict_category" == unavailable || "$verdict_category" == anchor ]]; then
      stopped_verdict="$verdict_category"
      return
    fi
    [[ -f "$RESULTS/$next_row.line" ]] && cat "$RESULTS/$next_row.line"
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
if [[ "$stopped_verdict" == anchor ]]; then
  cat "$RESULTS/$next_row.stderr" >&2
  exit 3
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
echo "计数：内存撞顶 $memory_hit_count 条（上限 $MEMORY_MAX）、超时 $timeout_count 条、排不上 $queue_full_count 条、被总上限挤掉 $squeezed_count 条、显存不够 $gpu_memory_count 条"
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
exit $fail
