#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Bash）：重型测试只在提交代码时、或用户要求时跑，子 agent 只跑自己那一份；不合的当场拒绝（退出 2，stderr 写原因与出路），同时记进检出记录交主 agent 看。
# hook-events: PreToolUse
# gate-similar: bash-command-detector.sh 同挂 PreToolUse[Bash]、判同一条命令，切词已抽成 lib_shell_words.py 两边共用；它判命令的形状（没超时的等待循环、后台里又放后台、起看门狗的写法），对谁都一样，起看门狗的错误写法与前台没超时的等待循环拒绝、其余只记不拦；这里判命令跑不跑重型测试，按 agent_type 与 SINGLEFS_HEAVY_TESTS 前缀放行、不合就拒，还把执行的脚本读进去逐层判，判定与看门狗共用 lib_heavy_tests.py
# gate-similar: pattern-process-guard.sh 同挂 PreToolUse[Bash]、也在执行前拒绝；它是上游 SOP 的钩子，判按模式找进程（pgrep / pkill 带 -f、killall），对谁都拒、没有放行条件，判据与 shell-lint 的 S2、S3 同一份；重型测试（层 0、QEMU、herd7）是本工程自己的装置，不归上游管，也不许在 .claude/singlefs-ai-sop/ 里就地改
# gate-similar: runner-dispatch-guard.sh 管同一条规矩（子 agent 不跑重型测试），但挂 PreToolUse[Agent|Task]、在派发那一刻判派发提示里的中文句子（只在「跑 / 复跑 / 执行 / 运行 / bash / 起」的宾语是重型阶段时才拒，否定、转述、引号里的都不判）；这里挂 PreToolUse[Bash]、在执行那一刻判真要跑的命令与它执行的脚本，一边判自然语言、一边判 shell 命令，判法没有能共用的
# gate-similar: write-guard.sh 同样按 agent_type 分谁能做什么，但挂 PreToolUse[Write|Edit]，判的是写哪个文件、写进什么字（整份覆盖未跟踪文件、写范围表、撇号类字符）；这里判 Bash 命令，放行表是各 agent 能跑哪几类重型测试，与写范围表没有共用的行
# gate-similar: continuation-guard.sh 挂 PreToolUse[SendMessage]，判的是收件的子 agent 在会话记录里交回过没有、被中断过没有；这里挂 PreToolUse[Bash]，判命令本身，两边的对象与输入没有交集
# gate-overlap:copy-kept bash-command-detector.sh 两边 main() 开头那十行（认 --selftest、从 stdin 读 hook 的 JSON、不是对象就放行）是入口，读不到共用模块时照样要跑：那时还要拿这份 JSON 带着 session_id 记一条检出，看门狗按 session_id 认本会话；抽进 lib_shell_words.py 或另一个同目录模块，模块读不到的那一刻入口跟着一起没了（两个 hook 自检里「hook 旁边没有共用模块」那一例走的就是这条路）
#
# 为什么：用户 2026-09-24 定「subagent任务派发的门禁里面写上 禁止跑0层测试等这种重测试 就跑自己相关的测试就好了」，
# 同日再定「项目内 全量崩溃 qemu herd7 等重型的测试任务 禁止平时调用 仅仅在每次代码提交时候跑 或者要求时候再跑 门禁写清楚。 提交代码必须跑。 subagent 严禁跑」、
# 更正「崩溃验证员和门禁分诊员 跑各自的部分就好了。不需要跑全量」、补「实在是任务需要跑的时候 弹窗」。定义里写一句「不跑」守不住：
# 同一天几个实现员在各自的副本里跑全量 `cargo test --all` 与层 0 快档，满载下每遍四五个小时（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 4、10、12、14 行）。
# 同日一个实现员把验证链写成 /tmp 下的脚本再用后台 Bash 起它，链里有一步属于「子 agent 一律拒」的一类，这道闸只看到「执行一个脚本」、放行了
# （同一张表第 15 行）：所以命令位置上执行的脚本文件要读进去判。
# 子 agent 自测时常在同一条命令里先用 heredoc 写出脚本、紧接着执行它，判的那一刻文件还不在，按「不存在」记检出，一小时里叫醒主 agent 六七次
# （同一张表第 18 行）：所以同一条命令里前面写出的脚本拿写出的内容判，不记「不存在」。
# 双引号里没转义的反引号把 `crates/…/walk.rs` 当命令替换执行，这个 `.rs` 被当 shell 脚本读进去、里面像路径的词再当脚本读，一路递归到第 6 层，
# 一条命令 100 秒没判完、拖慢每个 agent 的每条命令（同一张表第 18 行）：所以直接执行的只读 shell 脚本，同一份脚本一次判定里只读一遍、只判一遍。
#
# 重型测试，按类（只认命令位置；「cargo test」在下面各类里一样指 cargo t、cargo nextest run、cargo miri test、cargo llvm-cov、cargo hack test、
# cargo mutants 与 --config / CARGO_ALIAS_ 定的别名；libtest 参数里 --list 当选项出现的只列用例、一条都不跑，哪一类都不算，
# 跟在 --skip、--logfile 这类带值的选项后面的 --list 是那个选项的值，照算）：
#   checker 档    cargo test 的范围里有 checker 档包 singlefs-checker-tier（-p 点名它，认通配、@版本、pkgid URL；不挑包而范围含它；
#                 在它目录里裸跑；别名展开成这样的），不管挑哪个目标；直接执行它的测试二进制（tests/ 下的目标、src/bin/ 下的装置、库本身）；
#                 .claude/gate.d/checker-tier-layer0-replay.sh、research/scripts/layer0-shard-run.sh（按树根里键那一行登记的包判，登记的不是 checker 档包、
#                 没登记分片或没有这一行的不算，判法在 lib_heavy_tests.py 的 shard_driver_runs_checker_tier）。只按包判，不按测试名、不看 --ignored
#                 （.claude/rules/verification.md「定义与名字」；判定在 lib_heavy_tests.py）
#   QEMU          .claude/gate.d/checker-tier-qemu-device-streams.sh、qemu-system-*、research/scripts/vm-bench.sh（--selftest 也起虚机，照算）
#   herd7         .claude/gate.d/checker-tier-lkmm.sh、.claude/scripts/lkmm.sh、herd7
#   crates 变异整表 .claude/gate.d/checker-tier-crates-mutation-replay.sh、research/scripts/mutate.sh 的参数里有 crates/mutations.tsv、
#                 research/scripts/mutation-shard-run.sh（它的双机分片驱动）
#   全量测试      cargo test 带 --workspace / --all；在工作区根（清单有 [workspace] 没有 [package]：仓根与 research/ 都是）上
#                 不带 -p / --test / --lib / --bin 的 cargo test；不挑目标而包的范围是工作区全部成员的 cargo test
#                 （research/ 只有 e7-index-bench 一个成员，在它里面裸跑等于全量）；.claude/scripts/check.sh
#   全部实验复跑  .claude/gate.d/checker-tier-research-build-and-replay.sh，只跑它的单测那一格（--check research-unit-tests）的除外：那一格落到下一类
#   整轮门禁      gate.sh、research/scripts/gate-staged.sh
#   E152 装置     e152-file-system-benchmark（直接起、或 cargo run 它）、research/scripts/e152-run.sh
#   广谱变异      research/scripts/broad-mutation.py（python 脚本不读正文，按名字与位置、解到的真实路径、文件开头的签名行认：`python3 -W / -X <值> 脚本`、
#                 `python3 -m broad-mutation`、指到它的符号链接、带签名行的改名拷贝、`python3 -c` 的代码里点它的都认）。凡要起 cargo 或改库目录的都算：
#                 整趟、--shard / --from / --package、--item 点名（点几条、带不带 --skip-stage-two 都算）、--stage-two-only、--sync-tree（换全局的版本指针）、
#                 --import-results；只有带 --list-items / --summary / --check-ignored / --help（-h）的不算（仓内那一份单独一个 --selftest 照自证登记表判）。
#                 提交门禁那一道 .claude/gate.d/checker-tier-broad-mutation.sh 同算这一类（带 --list、--tool-versions 的不算，--check 只点名 gate-verdict 一格的不算）。子 agent 一律不跑；主 agent 带前缀能跑（提交时先起驱动
#                 --sync-tree、--changed-since HEAD，再跑那一道），不算的那几样谁都能跑。
#                 罩不住的写法在 lib_heavy_tests.py 文件头「广谱变异的逐条驱动」那一段
#   Verus         .claude/gate.d/checker-tier-formal-verification.sh（逐格起 Verus、空真与必红世界逐处起、编装置重放钉住的红例）。子 agent 一律不跑：
#                 主 agent 提交时带前缀单独跑，整轮 gate.sh 里也有它；直接起的 verus 与 cargo verus 不算（写证明的实现员要用，判法在 lib_heavy_tests.py）。
#                 这一道的两份判法脚本 research/scripts/verus-proof-check.py、research/scripts/e165-pin-red-examples.py 直接起的每一种用法同算这一类
#                 （python3 -m 加 PYTHONPATH、python3 -c、符号链接、参数放变量或 for 循环的都认），只有只读用法不算（lib_heavy_tests.py 的 verus_judge_use）：
#                 verus-proof-check.py 字面写明的 toolchain、fingerprint、tool-versions、list-items；e165-pin-red-examples.py 照它文件头的用法，起装置的只有 --replay，
#                 追加、--check、--backfill、--list-items 不算，参数里的变量只在可能展开成它认的选项名、或判不出落到哪种用法时才算（lib_heavy_tests.py 的 e165_pin_mode）；
#                 .claude/scripts/lkmm.sh 只带 --herd7-version 的不算
#   证红脚本      research/scripts/prove-red.sh 带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request 起、点名的变异行里有一行照 lib_heavy_tests.py 的 classify 判成重型的
#                 （它在 python 里逐行起 cargo test，读正文看不见；判法在 lib_heavy_tests.py 的 prove_red_use），算那一行的类；不带前缀的不算（它把重型的行报本次未跑）
#   崩溃放量      research/scripts/crash-amplification.sh（不带参数是崩溃放量全量；--item 点名分项、--expand-up-to 缩过规模的同算）、research/scripts/crash-amplification-item.sh
#                 （分项驱动）、research/scripts/crash-amplification-two-hosts.sh（双机驱动）：每一种跑法都起 checker 档 cargo（test-script-align-r3 攻方 A2）。按名字判、不读正文；
#                 只有 crash-amplification.sh --list-items（只列分项）与登记表里的单独 --selftest（三份都登记成 light，放行）不算。
#                 子 agent 一律不跑；主 agent 带 =commit 或 =user-request 都能跑（lib_heavy_tests.py 文件头「崩溃放量的三份入口」）
#   崩溃点重放    research/scripts/crash-point-replay.sh 的重型跑法：不带参数（剪枝）、--item 不带 --quick（逐条）、--full（全量），与不是轻型那种写法的每一种。
#                 按名字判、不读正文；只有单独的 --list-items、登记表里的单独 --selftest 与下面「轻型」那一段不算（lib_heavy_tests.py 文件头「两档里的轻型」）
#   真编 QEMU     research/scripts/build-patched-qemu.sh 除登记表里的单独 --selftest 之外的每一种起法（不带参数、--rebuild、别的参数）：取源码、在 TMPDIR 下
#                 约 1 GiB 的编译目录里按给的线程数编 QEMU、装进私有目录。按名字判、不读正文；子 agent 的放行表里没有这一类，只有主 agent 带前缀能跑
#                 （lib_heavy_tests.py 文件头「真编 QEMU」）
#   判不出的通配  命令词带方括号里的 POSIX 字符类、等价类、整理符号（`[[:alpha:]]`、`[[=h=]]`、`[[.h.]]`），交给 bash 展开而判不出展开成哪一份的
#                 （当前目录认不出、bash 起不来；lib_heavy_tests.py 文件头「命令词带通配」）：照拒，写成展开之后的字面路径再起
#   门禁阶段按文件名认，不按编号认；带 --list、--list-items、--tool-versions（只打格名表、逐项点名的清单、工具版本）的哪一道都不算；带 --write（生成）的不算。
# 提交时才跑的检查（不是重型，放行照同一张表；用户 2026-09-28 定「现在改到提交时候才验证的部分 都删掉」「门禁是靠 脚本和 hook来解决的 … 提示不执行 根本不靠谱」）：
#   上面几类之外的 .claude/gate.d/ 阶段；.claude/singlefs-ai-sop/scripts/ 下的 doc-lint.sh、gate-lint.sh、shell-lint.sh、preflight-lint.py、gate-overlap.py
#   （--list 查清单不算）、rules-lint.sh、hooks-registered.sh、stage-selftest.sh；research/scripts/gate-structure-check.py（不带 --selftest 的）；
#   研究脚本、钩子与门禁共用库的自证里登记成起重活的、没登记的、带了 --selftest 之外参数的那几份（下一段）。
#   这些都归提交前主 agent 跑的 research/scripts/gate-staged.sh：它带着前缀起整轮 gate.sh，
#   gate.sh 在里面起的这些不经这道闸（闸只看 Bash 工具的命令，看不见 gate.sh 的子进程）；看门狗 agent-watch.py 在进程这一层不报这一类
#   （Stop 钩子 gate-reuse-check.sh 每次收工也起 gate-overlap.py，那是钩子起的，不是哪个 agent 跑的）。
#   判定在 lib_heavy_tests.py 的 classify（COMMIT_CHECK_KINDS）；弄坏开关 HEAVY_TEST_GUARD_BREAK=ignore-commit-checks 时这一类一律放行，自检必须判红。
# 自证（上游 test-script-discipline.md「测试脚本自己也要被测试证明有效」：自证用假的驱动、不起编译与重活，拦重型测试的闸不拦自证）：
#   research/scripts/、.claude/hooks/ 下脚本的 `--selftest` 与 .claude/gate.d/lib/ 下共用库的直接执行，按登记表 .claude/hooks/selftest-registry.tsv 判
#   （键认本仓那一份文件：命令词解析后的真实路径要正是它，仓副本、worktree 里路径尾部相同的拷贝、路径写在变量里的照旧按提交时才跑的检查拒；
#   命令词带通配的——脚本里 case 的模式行切出来的也是——照字面不当自证，先照 bash 展开、按展开出来的那几份判，判法在 lib_heavy_tests.py 文件头「命令词带通配」那一段）
#   （一行一份、# 后写理由；判法在 lib_heavy_tests.py 的 classify_selftest，这里不另抄名单）：
#   登记成 light、命令写成单独一个 `--selftest`（门禁共用库：不带参数或单独一个 --selftest）的谁都能跑，不读脚本正文（正文里平常那条路上的 cargo 不是自证跑的；
#   lib_heavy_tests.py 的 selftest_judged_light）；登记成 heavy 的（自证真编、真跑 checker 档用例这类）与 heavy-if-set:<变量> 而命令里设了那个变量的，
#   照旧算提交时才跑的检查；表外的、带了别的参数的照旧拒，拒绝说明写怎么登记（vm-bench.sh --selftest 起虚机，按名字照旧算 QEMU）。
# 门禁样本（同一篇认的第三种自证形态：红绿样本目录）：同一张登记表里键是 .claude/gate.d/<阶段>.sh 的行登记那一道的样本档起不起重活
#   （samples-light、samples-light-marker:<假工具标记>、samples-heavy；判法与罩不住的写法在 lib_heavy_tests.py 文件头「门禁样本」，这里不另抄）。谁都能跑的两种：
#   ① .claude/singlefs-ai-sop/scripts/stage-selftest.sh 喂的阶段目录此刻在、里面有样本的每一道都登记成轻的、阶段文件与仓里那一份相同、
#      样本目录与仓 HEAD 加暂存区逐字节相同（多出的 setup.sh、没跟踪的新文件、改过内容或执行位的都算不同，样本改完要暂存之后才放行）；真仓的 .claude/gate.d 里有
#      登记成 samples-heavy 的几道（crates 变异表、Verus、research 构建与复跑），喂它照旧算提交时才跑的检查，拒绝说明点名那几道。放行时不读 stage-selftest.sh 的正文（按名字判）。
#      带 `--item <阶段文件名>`（可给几次）只判点名的那几道：每一道都登记成轻的、阶段文件与样本同上，才放行；
#      点名里有一道起重活、没登记或样本有改动照拒，拒绝说明点名那一道；点名里没有一道是配了样本的阶段的放行（stage-selftest.sh 自己退 2、不喂样本）；`--list-items` 放行。
#      带不带 --item、--list-items 都要命令词正是本仓那一份 stage-selftest.sh（别处树里路径尾部相同、改过正文的拷贝照拒）；阶段目录可写 $PWD/.claude/gate.d（当前目录认得出时换成它）。判法在 lib_heavy_tests.py 文件头「门禁样本」①。
#   ② 直接起登记成 samples-light-marker: 的那一道（checker-tier-layer0-replay、herd7、QEMU）：cd 进放了那个标记的样本根、根参数写同一个目录，
#      标记里的假工具（checker-tier-layer0-replay 的 cargo）可执行、与仓里那一道某份样本的逐字节相同（标记是文件的与仓里某份样本的标记相同；空标记目录、内容不同、不可执行的照拒）；
#      直接对真仓跑照旧按那一道拒，拒绝说明接一句样本档怎么跑。
# 轻型（.claude/rules/verification.md「两档：轻型与重型」）：research/scripts/crash-point-replay.sh --quick、.claude/gate.d/checker-tier-formal-verification.sh --quick [<根>]、
#   .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick [<根>]，三条都可再带 `--item <项>`（给几次）；不算哪一类，谁都能跑：子 agent 带不带前缀都放行、
#   主 agent 不带前缀也放行。夹了别的词（--full、--check、写成变量的项、多出的根参数）的照那一道或崩溃点重放的重型拒；认法在 lib_heavy_tests.py 的 light_tier_command。
#   三条在 lib_heavy_tests.runs_compiled_code 里算跑编译出来的代码：子 agent 要经 research/scripts/run-with-memory-cap.sh 起（下面「另一道」）；
#   要卡的经 research/scripts/gpu-quiesce-run.sh -- <命令> 起，它与 /usr/bin/time 那一类一样剥掉（lib_heavy_tests.py 的 command_under_gpu_quiesce），里面那条照这张表判。
# 谁、带什么才放行（都要带环境变量 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，别的值或没带一律拒）：
#   主 agent（输入里没有 agent_type）：上面每一类，连同提交时才跑的检查；
#   子 agent：一律拒，带不带前缀都拒（要跑的交回主 agent）；轻型三条不在这几类里，照放。
# 前缀认三种写法：写在命令前（`SINGLEFS_HEAVY_TESTS=commit bash …`）、写进 `env` 的参数、同一行前面的 `export`；
# 往 `bash -c '…'`、`capped.sh N …`、`nice`、`timeout` 这类包装里面传。git 的 pre-commit hook 由 git 起，不经这道闸。
# lib_shell_words 前缀表之外、包在命令外面照样起那条命令的 `/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、
#   `perf stat|record|trace`（lib_heavy_tests.py 的 LAUNCHER_OPTIONS_WITH_VALUE）剥掉之后，里面那条当一段命令文本接着判：前缀、内存包装、脚本照认。
#   research/scripts/detached-run.sh 的 `start … -- <命令>`（把命令挂到用户级 systemd 下脱离会话跑，调用方的 SINGLEFS_HEAVY_TESTS 带进去）同样剥掉，
#   不读它的正文：`--` 之后那条照这张表判，外面写的前缀与 `--` 之后写的前缀都认，谁能跑什么不因为经它起而变（lib_heavy_tests.py 的 command_under_detached_run）。
#   它的参数先照它开头的 preflight 摘掉每一个 `--force`（连 `--` 之后的），判的就是单元里真跑的那条（lib_shell_words.py 的 detached_run_arguments_after_preflight）。
#   认它按内容，不只按名字（lib_shell_words.py 的 detached_run_script）：名字叫 detached-run.sh、指到它的符号链接、开头带签名行
#   （行首 `# detached-run-script-signature:`）的改名拷贝，bash / sh / source / . 起的、直接执行的都认；同一条命令里前面 cp 拷来、heredoc 写出的，
#   按写出的内容找签名行（written_detached_run）。认签名行时另读命令词那份文件的开头 4 KiB（不经 JudgingState、不记进读过的文件）。
#   罩不住的写法在 research/scripts/detached-run.sh 文件头「谁能用」那一段。
# 写进脚本文件再执行的，读脚本正文、逐条照同一张表判：命令位置上是 `bash|sh 文件`（不带 -c）、`./x.sh` 或 `路径/x.sh`、`source 文件` / `. 文件`，
#   经 capped.sh N、nice、timeout、env、前缀变量包一层的同样认；脚本里再起脚本的递归读，读到第 MAXIMUM_SCRIPT_DEPTH（6）层为止，再往里判「看不全」。
#   外层命令带的前缀与 export 往脚本里传，脚本里某一行自己写的前缀同样认；谁能跑什么照上面那张表，不因为写在脚本里而变。
#   拒绝说明写出脚本路径与行号（`外层脚本:行 → 里一层的脚本:行`）。
#   按名字判的仓内脚本（.claude/gate.d/ 下的阶段，gate.sh、gate-staged.sh、mutate.sh、capped.sh、run-with-memory-cap.sh、崩溃放量的三份入口、build-patched-qemu.sh、上游副本里的 stage-selftest.sh 这几个在各自的仓内位置上）不读正文：
#   算不算重型、带什么参数才算，已经在名字那一格判完（lib_heavy_tests.py 的 judged_by_name）。
#   直接执行的（`./x`、`路径/x`，经包装的同样）只当两种文件是脚本：`#!` 指到 shell（SHELL_NAMES：bash / sh / dash / zsh / ksh，`#!/usr/bin/env` 转一层也算），
#   或没有 `#!`、扩展名是 `.sh`（执行的名字或它指到的文件）；开头是编译产物（ELF 魔数、NUL 字节）的也不算。别的一律不读、不记：
#   `.rs`、`.py`、`.md`、`.tsv` 这类、没扩展名又没 `#!` 的文本、`#!` 指到 python 这类别的解释器的。判它只看开头 SCRIPT_HEAD_BYTES（4 KiB），判在大小上限之前；
#   还不在的，只有扩展名是 `.sh` 的记「不存在」。`bash x`、`sh x`、`source x`、`. x` 显式交给 shell 的不看扩展名与 `#!`，照读。
#   读不到的——文件不存在、不是文本、超过 MAXIMUM_SCRIPT_BYTES（1 MiB）、嵌套超过上限看不全——这条命令照常执行，stderr 报一句并记一条检出；
#   不是普通文件的（目录、设备）不读、不记。脚本调回它自己（或绕一圈回来）不再读第二遍。
#   一次判定里（JudgingState）同一份文件按实际路径只从盘上读一遍；同一份脚本判过一遍，再遇到时直接用那一遍的结果，拒绝说明与检出照这一处的路径与行号写。
#   复用要这几样都相同：实际路径、执行时的当前目录、SINGLEFS_HEAVY_TESTS 的值、同一条命令里写出的文件自那一遍起没变过（written_version）、起它的那一条经没经内存包装；
#   那一遍撞上了嵌套上限、绕回了它外层的脚本、或自己写出了文件的，结果跟着从哪里起、什么时候起走，不复用，再遇到时重判（正文仍不重读）。
# 同一条命令里前面写出、后面执行的脚本（WrittenScript），执行时拿写出的内容判，先于盘上那一份（执行时盘上已经是写出的内容），不记「不存在」：
#   heredoc 喂给 cat 再输出重定向到那个文件（`cat > x.sh <<'EOF'`、`cat <<'EOF' > x.sh`；cat 带了文件参数或别的选项的不算），
#   heredoc 喂给 tee（参数里的文件，连同它的输出重定向），拿 heredoc 正文当脚本内容；
#   `cp 源 目的`（也认 `cp 源… 目录`、`-t 目录`），源在盘上就读源文件，源是同一条命令里更前面写出的就接着用那份内容；
#   追加（`>>`、`tee -a`）接在这条命令里前面写出的内容、或盘上那份（读得了的文本）后面；盘上那份读不了的不认，照旧读盘上。
#   脚本里再写出、再执行的同样认，写出的文件对外层后面的命令也算数。读法与判定同盘上的脚本（开头的 `#!`、大小上限、递归读）。
#   认不出的写法（printf、echo、python 写文件、mv、install、管道 `cat <<EOF | tee x.sh`、变量拼出的目标）照旧：还不在的按「不存在」记检出，已经在的读盘上那份。
#   正文不带引号的 heredoc（`<<EOF`）写进文件之前 shell 还要展开 `$` 与命令替换，这里判的是展开之前的原文。
# 命令位置上的词里带没展开的 `$`、反引号、括号、引号残片、重定向、通配字符、逗号或等号（NOT_A_LITERAL_PATH）的，不当脚本路径去读；
#   命令替换 `$(…)` 与反引号整段留在所在的词里，里面的命令按一条完整的命令另判（lib_shell_words.py）。
# 命令词带通配（`*`、`?`、`[`）的，判定照 bash 展开、按展开出来的那几份的名字判（重型、按名字判的重型脚本、门禁阶段、提交时才跑的检查，写成通配的包装照剥；
#   lib_heavy_tests.py 文件头「命令词带通配」），展开出来的脚本照样不读正文。
#   带方括号里的 POSIX 字符类、等价类、整理符号的（`[[:alpha:]]`、`[[=h=]]`、`[[.h.]]`）交给 bash 自己展开（python glob 不认），判不出展开成哪一份的命令词照拒。
# 命令词与参数里的花括号（`{a,b}`、`{1..3}`）照 bash 展开、参数里的通配照 bash 展开之后再判，展开出来有一条判成哪一类就照它拒
#   （`mutate.{sh,x}`、`checker-tier-layer0-replay.{sh,}`、`crates/mutation?.tsv`；lib_heavy_tests.py 文件头「花括号与参数里的通配」），花括号展开出来的脚本同样不读正文。
# 命令词没展开（`$b`、`$(ls …)`）的，连同它点到的变量在这段命令文本里的定义（`名=值`、`for 名 in 词…`）里有 checker 档包的测试目标名，
#   照直接执行它的测试二进制算（lib_heavy_tests.py 的 unexpanded_command_word_use；调度记录 records/2026-09-30-里程碑三开工第一轮.md 第 936 行那一次：
#   循环里 `b=$(ls -t $D/$t-* | …)` 再经内存包装与 capped.sh 起 `$b`）；读进去的脚本正文只用它自己的定义。
# 看不见的（照常放行、不记检出）：变量里拼出来的命令与脚本路径（命令词没展开而定义里点不到 checker 档测试目标名的）、通配与花括号写出来的脚本里的正文、eval、`<<<` 喂给 shell 的字符串；
#   喂给 python 这类非 shell 解释器的程序（`python3 x.py` 里用 subprocess 起的命令；heredoc 正文先剥掉）；
#   make（Makefile 里起的命令）与 xargs 起的命令；cd 到变量路径之后的裸 cargo test（认不出在不在工作区根）与相对路径的脚本；
#   命令词不带斜杠、从 PATH 里找到的脚本（直接执行的 `x.sh`）；同一条命令里用 WrittenScript 认不出的写法先写出再执行的脚本——
#   执行时文件还不在的按「不存在」放行并记检出，已经在的读到的是旧内容；source 进来的文件里 export 的变量对外层后面命令的影响；
#   .cargo/config.toml 与 `--config <文件>` 里定的别名与 runner；拷走改名、又不点名登记的用例函数的测试二进制（看门狗在进程这一层同样认不出）。
# 接受的误拒：`--include-ignored --exact <同一目标里的快用例>` 照拒（过滤之后剩下哪几条，执行前判不出）；别名展开之后不是 test 的照拒；
#   同一目标里别的模块有与登记的用例函数同名、合法不标 #[ignore] 的快用例时照拒；admission.py 导入不了、或导入得了而判的那一刻抛异常时，
#   点名登记目标的 cargo test 一律拒；`-- --ignored -- --list`（第二个 `--` 之后只剩过滤词，一条都不跑）照拒。
# 另一道，与重型不重型无关：子 agent 跑编译出来的代码——cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算）、
#   直接执行 cargo 编出来的二进制（lib_heavy_tests.runs_compiled_code）——要经 research/scripts/run-with-memory-cap.sh 跑，不经它的拒
#   （它先判整机放不放得下、放不下排队，撞了上限只杀这一条；records/2026-09-16-subagent拆分提案.md 第四十节第 30 行）。放在这里而不另起一个 hook：
#   判的是同一批对象（Bash 命令里的 cargo test / run 与测试二进制）、同一个触发点、同一套切词与读脚本。
#   算经它包着的：它剥出来的那条命令、它 `bash -c` 里的、它起的脚本里的（往里读时接着算，lib_shell_words 的 memory_capped）；
#   包装外面的命令替换 `$(…)` 先于包装执行，不算。主 agent 不判这一道。按名字判的仓内脚本（门禁阶段、mutate.sh 这些）不读正文，
#   它们里面起的 cargo 不在这一道的射程里（checker-tier-crates-mutation-replay 与 mutate.sh 自己每条经包装跑）。喂给 shell 的 heredoc 正文里的命令不算在挂它的那条命令的包装里：
#   `run-with-memory-cap.sh 4G bash <<EOF` 会被误拒，写成脚本文件或 bash -c。HEAVY_TEST_GUARD_BREAK=ignore-memory-cap 时不判这一道（只给自检证明它会红）。
# 判定（一条命令是不是重型、属于哪一类）在同目录的 lib_heavy_tests.py，看门狗在进程这一层复用同一份；
# 切词、切简单命令、认命令位置、剥前缀与包装、跟 cd 与 export 在同目录的 lib_shell_words.py（与 bash-command-detector.sh 共用），由 lib_heavy_tests.py 导入。
# 两份都按文件路径导入，这里只留谁能跑什么、读脚本与拒绝；lib_shell_words.py 的函数在 .claude/hooks/ 别的文件里再定义一份，门禁 code-tooling 的 agent-write-scope 格判红。
# 读不到它们时这条命令照常执行，stderr 报一句并记一条检出。
#
#   heavy-test-guard.sh             # 从 stdin 读 hook 的 JSON；拒绝退出 2，放行退出 0
#   heavy-test-guard.sh --selftest  # 在临时工作区里走一遍拒绝与放行（连同 lib_heavy_tests.py 的自检）；
#                                   # 弄坏开关 HEAVY_TEST_GUARD_BREAK=<项>（几个用逗号连；项名写错的，自检「HEAVY_TEST_GUARD_BREAK 里的项都认得」一格红），设了哪一项自检都必须判红。
#                                   # 逐项写开关名、弄坏哪一处、打开后自检哪几格红（格名是 selftest 里 cases 与 stdin 那几格的说明，与自检失败行「✗ 自检：」之后、「应当是」之前那一段相同；
#                                   # cases 里写了该记几条检出、该读进去几份脚本的，各另成一格，格名是说明后面接「：记几条检出」「：读进去几份脚本」）：
#                                   #   HEAVY_TEST_GUARD_BREAK=disable-check  decide() 一进来就放行，整道闸不判（重型、提交时才跑的检查、内存包装都不拒）
#                                   #       「实现员跑 cargo test --all」「实现员跑 checker 档包的测试（checker 档按包判）」「实现员跑 checker 档包点名一个快档目标」「实现员跑 checker 档包的库单测也拒」「主 agent 不带前缀跑 checker 档包」「实现员跑 checker 档的集成测试」「实现员跑 check.sh」「实现员经 capped.sh 跑 --workspace」
#                                   #       「实现员经 run-with-memory-cap.sh 跑 --workspace」「实现员在 bash -c 里跑 --all」「实现员在仓根裸跑 cargo test」「实现员 cd 进 harness 档裸跑、不经内存包装（拒的是包装，不是重型）」「实现员 -p harness 档不挑目标、不经内存包装（拒的是包装）」「实现员 -p checker 档 --test 通配」
#                                   #       「实现员前缀一串再跑 --workspace」「实现员在 research 工作区根裸跑」「实现员在 research 唯一的成员里裸跑」「实现员带前缀也拒」「实现员 mutate.sh 跑 crates 整表」「实现员起 qemu-system」「实现员 vm-bench.sh --selftest」「实现员跑 herd7」「实现员跑 lkmm.sh」「实现员跑 Verus 那一道门禁」
#                                   #       「主 agent 不带前缀跑 Verus 那一道」「实现员带前缀跑 Verus 那一道照拒」「实现员直接起判法脚本跑必红世界」「主 agent 不带前缀直接起判法脚本跑必红世界」「调查员经内存包装起红例重放」「主 agent不带前缀跑广谱变异的提交门禁：照拒」
#                                   #       「主 agent不带前缀起广谱变异驱动：照拒」「实现员带前缀跑广谱变异的提交门禁：照拒」「实现员跑 gate-staged.sh」「实现员喂给 bash 的 heredoc 里跑 --all」「实现员直接执行 checker 档的测试二进制」「书记员跑 research 构建与复跑那一道」「通用 agent 跑 checker-tier-layer0-replay」
#                                   #       「主 agent跑 checker-tier-layer0-replay 不带前缀」「主 agent不带前缀跑 gate.sh --staged」
#                                   #       「主 agent 裸跑 gate.sh --staged」「主 agent 带 =whatever」「主 agent 第二条命令没带前缀」「主 agent 裸跑 E152 的 run 脚本」「主 agent 裸 cargo run E152」「实现员跑登记的checker 档的用例（带 --include-ignored）」
#                                   #       「实现员跑 checker 档集成测试的快用例（不带 --ignored 也拒：按包判）」「主 agent不带前缀跑checker 档的用例」「主 agent 不带前缀直接执行checker 档的用例的测试二进制」「实现员 /usr/bin/time -v 包一层跑 --all」「实现员 systemd-run --scope 自设内存上限跑checker 档的用例」
#                                   #       「实现员 perf stat 包一层跑 --all」「主 agent 经 detached-run.sh start 起广谱变异、不带前缀拒」「实现员带前缀经 detached-run.sh start 跑 --all 照拒」「实现员经 detached-run.sh start、里面经内存包装跑 checker 档包照拒」「主 agent 不带前缀经改名拷贝起全量照拒」
#                                   #       「主 agent 不带前缀经指到它的符号链接起全量照拒」「主 agent 不带前缀同一条命令里先 cp 改名再起全量照拒」「实现员同一条命令里先 cp 改名再起全量照拒」「实现员 source 读进改名拷贝起 checker 档照拒」「主 agent 不带前缀同一条命令里 heredoc 写出带签名行的拷贝再起全量照拒」
#                                   #       「主 agent 不带前缀经 detached-run.sh start、-- 之后先写 --force 再起全量照拒」「主 agent 不带前缀经 detached-run.sh start、--label 后面先写 --force 再起全量照拒」「实现员 /usr/bin/time 包一层跑自己的测试目标、没经内存包装」「主 agent 不带前缀 flock 包一层跑 gate.sh」
#                                   #       「实现员内存包装里 strace -fo 包一层跑checker 档的用例」「实现员不经内存包装 strace -fo 包一层跑checker 档的用例」「实现员内存包装里 flock -xw 10 包一层跑checker 档的用例」「实现员内存包装里 -- --include-ignored --skip --list（--list 是 --skip 的值）」
#                                   #       「实现员内存包装里直接执行checker 档的用例的测试二进制 --include-ignored --skip --list」「实现员内存包装里 systemd-run --expand-environment no 包一层跑checker 档的用例」「实现员内存包装里 strace --output 文件包一层跑checker 档的用例」
#                                   #       「实现员内存包装里 -- --include-ignored the_full_case -- --list（第二个 -- 之后是过滤词）」「实现员经内存包装 cargo nextest run --run-ignored all 跑checker 档的用例」「实现员 cargo mutants 整个 checker 档」「实现员经内存包装、--config 定别名再用别名」
#                                   #       「执行员跑 doc-text（不重型的阶段：提交时才跑的检查）」「书记员跑 doc-decisions（提交时才跑的检查）」「通用 agent 跑 code-tooling.sh 的 agent-write-scope 格（提交时才跑的检查）」「主 agent 不带前缀跑 code-tooling.sh（提交时才跑的检查）」「主 agent不带前缀跑 gate-lint.sh」
#                                   #       「工具实现员带前缀跑 rules-lint.sh 照拒」「工具实现员拿样本跑 stage-selftest.sh、阶段目录此刻不在：判不出喂的是哪几道，照拒」「工具实现员 stage-selftest.sh 喂有起重活样本的阶段目录（crates 变异表）：照拒」「实现员拿别处树里改过的 stage-selftest.sh 拷贝、不带 --item 喂全登记成轻的阶段目录：照拒」
#                                   #       「实现员 stage-selftest.sh --item 点名里夹一道起重活的（crates 变异表）：照拒」「工具实现员 cd 进样本根、根参数指到真仓跑 checker-tier-layer0-replay：照拒」「工具实现员把 crates 变异表那一道指到样本根（样本档起真 cargo，登记成起重活）：照拒」「实现员在一棵树里只放 .layer0-sample-tools/quick-tier 再起 checker-tier-layer0-replay 快档：照拒」
#                                   #       「实现员在同一棵树里起 checker-tier-layer0-replay --full：照拒」「攻方腿在同一棵树里起 checker-tier-layer0-replay：照拒」「工具实现员把 checker-tier-layer0-replay 指到假 cargo 不可执行的样本根：照拒」「实现员 stage-selftest.sh 喂样本里多一份起 checker 档的 setup.sh 的阶段目录：照拒」
#                                   #       「实现员跑别处树里路径尾部相同的 claim-experiment.sh --selftest（正文起 checker 档）：照拒」「工具实现员起带 case 模式行 .claude/gate.d/lib/*.py) 的脚本：放行：读进去几份脚本」「实现员跑 gate-structure-check.py」「实现员跑登记表外的新脚本 --selftest，照拒」
#                                   #       「实现员 mutate.sh 带表再带 --selftest（只认单独一个 --selftest；这样跑的是 crates 整表），照拒」「实现员带前缀跑 layer0-shard-run.sh --selftest（设了 SINGLEFS_HEAVY_TESTS 换真 cargo 跑 checker 档包），照拒」「工具实现员跑 vm-bench.sh --selftest（起虚机），照拒」
#                                   #       「实现员只跑 research 单测那一格（不重型，照样是提交时才跑的检查）」「主 agent 不带前缀跑 hooks-registered.sh」「实现员起双机分片的驱动、树根里登记的是 checker 档包的分片用例」「实现员不经内存包装跑自己的测试目标」「执行员不经内存包装 cargo run 实验二进制」「实现员 cargo t 简写不经内存包装」「执行员直接执行实验二进制不经内存包装」
#                                   #       「实现员经 capped.sh 限了线程、没限内存」「实现员包装外面的命令替换不算在包装里」「实现员命令替换里不经内存包装跑」「实现员起的脚本里有不经内存包装的 cargo test」「实现员起的脚本里有不经内存包装的 cargo test：读进去几份脚本」「实现员经内存包装起那份脚本：读进去几份脚本」「实现员 heredoc 写出不经内存包装的 cargo run 再起它」
#                                   #       「实现员 heredoc 写出不经内存包装的 cargo run 再起它：读进去几份脚本」「实现员经内存包装跑 checker-tier-layer0-replay 照拒（重型那一道）」「实现员 bash 起含它的脚本」「实现员 bash 起含它的脚本：读进去几份脚本」「实现员 ./ 直接执行含它的脚本」
#                                   #       「实现员 ./ 直接执行含它的脚本：读进去几份脚本」「实现员经 capped.sh 4 bash 起含它的脚本」「实现员经 capped.sh 4 bash 起含它的脚本：读进去几份脚本」「实现员 source 含它的脚本」「实现员 source 含它的脚本：读进去几份脚本」「实现员 . 读进含它的脚本、nice 与 timeout 包一层」
#                                   #       「实现员 . 读进含它的脚本、nice 与 timeout 包一层：读进去几份脚本」「实现员 cd 之后按绝对路径起含它的脚本、输出落文件」「实现员 cd 之后按绝对路径起含它的脚本、输出落文件：读进去几份脚本」「实现员起的脚本里再调含它的脚本」「实现员起的脚本里再调含它的脚本：读进去几份脚本」「实现员起仓外同名的 mutate.sh（不按名字判，读正文）」
#                                   #       「实现员起仓外同名的 mutate.sh（不按名字判，读正文）：读进去几份脚本」「实现员起的脚本那一行自己带前缀也拒」「实现员起的脚本那一行自己带前缀也拒：读进去几份脚本」「主 agent 带放行前缀执行含它的脚本：读进去几份脚本」「主 agent export 之后执行含它的脚本：读进去几份脚本」「主 agent 不带前缀执行含它的脚本」「主 agent 不带前缀执行含它的脚本：读进去几份脚本」
#                                   #       「主 agent 执行的脚本那一行自己带前缀：读进去几份脚本」「实现员起不含重型命令的脚本：读进去几份脚本」「实现员起调回自己的脚本（只读一遍）：读进去几份脚本」「实现员 ./ 执行超过读的上限的 shell 脚本：放行并记检出：记几条检出」「实现员起 doc-lint.sh 第 43 行那种命令替换：片段不当路径：读进去几份脚本」
#                                   #       「实现员起 doc-lint.sh 第 70 行那种命令替换：片段不当路径：读进去几份脚本」「实现员起的脚本里命令替换、反引号、算术后面接路径：片段不当路径：读进去几份脚本」「实现员起的脚本里命令替换里跑 --all」「实现员起的脚本里命令替换里跑 --all：读进去几份脚本」「实现员起的脚本里 command -v herd7：查名字不是跑：读进去几份脚本」
#                                   #       「实现员起的脚本里数组赋值的元素：不当命令、不当路径：读进去几份脚本」「实现员 command herd7（不带 -v）照拒」「实现员 diff <(bash check.sh) x：括号里真执行全量测试，照拒」「实现员同一行里起 .sh 之后把 python 源码喂给 python3 -：正文不当命令：读进去几份脚本」
#                                   #       「实现员喂给 bash 的正文里有 source = 与带逗号的路径：不当脚本：读进去几份脚本」「实现员通配起 checker-tier-layer0-replay （绝对路径）：照拒」「实现员 cd 进仓根通配起 QEMU 那一道：照拒」「实现员通配起 mutate.sh 跑 crates 整表：照拒」「实现员通配起 vm-bench.sh --selftest：照拒」「实现员写成通配的 capped.sh 包全量：照拒」
#                                   #       「实现员 POSIX 字符类起 mutate.sh 跑 crates 整表：照拒」「实现员等价类起 checker-tier-layer0-replay：照拒」「实现员整理符号起 QEMU 那一道：照拒」「实现员 cd 到变量路径之后 POSIX 字符类起脚本：判不出，照拒」「实现员起崩溃放量全量：照拒」「实现员起崩溃放量的分项驱动：照拒」
#                                   #       「主 agent 不带前缀起崩溃放量全量：照拒」「实现员真编 QEMU（--rebuild）：照拒」「实现员经内存包装真编 QEMU（不带参数）：照拒」「主 agent 不带前缀经内存包装真编 QEMU：照拒」「主 agent不带前缀起崩溃放量全量：照拒」「主 agent不带前缀经 detached-run.sh start 起崩溃放量全量：照拒」
#                                   #       「实现员带 =user-request 起崩溃放量全量：照拒」「实现员花括号起 mutate.sh 跑 crates 整表：照拒」「实现员花括号起 checker-tier-layer0-replay：照拒」「实现员参数里的通配点到 crates 整表：照拒」
#                                   #       「实现员花括号写在包装上（{bash,-x}）：照拒」「实现员起不存在的脚本：放行并记检出：记几条检出」「实现员 ./ 执行不存在的脚本：放行并记检出：记几条检出」「实现员 bash 起不是文本的文件：放行并记检出：记几条检出」「实现员起超过上限的脚本：放行并记检出：记几条检出」「实现员起的脚本嵌套 7 层：读到第 6 层、看不全，放行并记检出：记几条检出」
#                                   #       「实现员起的脚本嵌套 7 层：读到第 6 层、看不全，放行并记检出：读进去几份脚本」「实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它」「实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它：读进去几份脚本」「实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行」
#                                   #       「实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行：读进去几份脚本」「实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source」「实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source：读进去几份脚本」「实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它」
#                                   #       「实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它：读进去几份脚本」「实现员 cd 进子目录写出、回来按相对路径起」「实现员 cd 进子目录写出、回来按相对路径起：读进去几份脚本」「实现员 heredoc 盖掉盘上不含重型命令的 light.sh 再起它：判写出的内容」
#                                   #       「实现员 heredoc 盖掉盘上不含重型命令的 light.sh 再起它：判写出的内容：读进去几份脚本」「实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本」「实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本：读进去几份脚本」「实现员 bash -c 里写出含 --all 的脚本再起它」
#                                   #       「实现员 bash -c 里写出含 --all 的脚本再起它：读进去几份脚本」「实现员命令替换里写出含 --all 的脚本再起它」「实现员命令替换里写出含 --all 的脚本再起它：读进去几份脚本」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份：读进去几份脚本」
#                                   #       「实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在」「实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在：读进去几份脚本」「实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在」「实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在：读进去几份脚本」
#                                   #       「实现员 cat -n 带选项写出：认不出，照旧记检出：记几条检出」「实现员 heredoc 写出不含重型命令的脚本再起它：放行、不记检出：读进去几份脚本」「主 agent 带前缀起 heredoc 写出的含 --all 的脚本：读进去几份脚本」「实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件」
#                                   #       「实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件：读进去几份脚本」「实现员 cp -f 到目录里再起：读源文件」「实现员 cp -f 到目录里再起：读源文件：读进去几份脚本」「实现员 cp 盘上不含重型命令的 light.sh 出来再起：放行、不记检出：读进去几份脚本」「实现员 cp 同一条命令里 heredoc 写出的脚本再起」
#                                   #       「实现员 cp 同一条命令里 heredoc 写出的脚本再起：读进去几份脚本」「实现员 cp 的源不在：照旧记检出：记几条检出」「实现员 printf 写出再起：认不出，照旧记检出：记几条检出」「实现员 ./ 直接执行没扩展名、#! 指到 bash 的脚本：读进去照拒」「实现员 ./ 直接执行没扩展名、#! 指到 bash 的脚本：读进去照拒：读进去几份脚本」
#                                   #       「实现员 ./ 直接执行 .py 名字、#! 指到 sh 的脚本：读进去照拒」「实现员 ./ 直接执行 .py 名字、#! 指到 sh 的脚本：读进去照拒：读进去几份脚本」「实现员 bash 起 .rs 文件：显式交给 shell 的照读，里面反引号括着的 .rs 不再读」「实现员 bash 起 .rs 文件：显式交给 shell 的照读，里面反引号括着的 .rs 不再读：读进去几份脚本」
#                                   #       「实现员复现输入那一条（反引号里的 walk.rs）：末尾的 doc-lint.sh 是提交时才跑的检查、读它之前就拒，不记检出，一份脚本都不读」「实现员同一条命令里起含 --all 的 s.sh 两遍：只判一遍」「实现员同一条命令里起含 --all 的 s.sh 两遍：只判一遍：读进去几份脚本」「实现员起的脚本里三种起法各起一遍 s.sh：s.sh 只判一遍」
#                                   #       「实现员起的脚本里三种起法各起一遍 s.sh：s.sh 只判一遍：读进去几份脚本」「主 agent 两遍都带前缀起 s.sh：只判一遍、放行：读进去几份脚本」「主 agent 先带前缀、再不带起 s.sh：前缀的值不同重判，第二遍照拒」「主 agent 先带前缀、再不带起 s.sh：前缀的值不同重判，第二遍照拒：读进去几份脚本」
#                                   #       「实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判」「实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判：读进去几份脚本」「实现员先起里面起了不存在脚本的 has-missing.sh、再经 outer-missing.sh 起它：检出照复用的那一遍记两条：记几条检出」
#                                   #       「实现员先起里面起了不存在脚本的 has-missing.sh、再经 outer-missing.sh 起它：检出照复用的那一遍记两条：读进去几份脚本」「实现员同一份 rel.sh 换个当前目录再起：按相对路径起的脚本变了，重判」「实现员同一份 rel.sh 换个当前目录再起：按相对路径起的脚本变了，重判：读进去几份脚本」
#                                   #       「实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判」「实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判：记几条检出」「实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判：读进去几份脚本」「实现员先起 cyc-a.sh（里面绕回自己）、再起 cyc-b.sh：cyc-b.sh 那一遍绕回外层，重判」
#                                   #       「实现员先起 cyc-a.sh（里面绕回自己）、再起 cyc-b.sh：cyc-b.sh 那一遍绕回外层，重判：读进去几份脚本」「造出来的登记表：实现员跑登记成 heavy 的 --selftest，照拒」「造出来的登记表：实现员 python3 通配起登记成 heavy 的自证，照拒」「同一条命令里起 s.sh 两遍：s.sh 从盘上读一遍（读盘次数）」
#                                   #       「起的脚本里三种起法各起一遍 s.sh：s.sh 从盘上读一遍（读盘次数）」「先带前缀、再不带起 s.sh：重判也不重读（读盘次数）」「直接执行 .rs 两遍：判它是不是脚本要看开头，开头也只读一遍（读盘次数）」「拒绝说明写出脚本路径与行号」「嵌套的拒绝说明写出每一层的路径与行号」「命令替换里的拒绝说明写出它所在那一行」「heredoc 写出的脚本，拒绝说明写出它的路径与正文里的行号」
#                                   #       「heredoc 写出的脚本里再写出的，逐层写出路径与行号」「复用上一遍的结果，拒绝说明照这一处的路径与行号写」「绕回外层的那一遍不复用，从别处起时照读回来」「复用上一遍的检出，照这一处的路径与行号写」「stdin:实现员跑 --all 拒绝（退出码 2）」「stdin:拒绝时 stderr 写了出路与前缀」「stdin:拒绝时 stderr 的「→ 规矩」那一串列出真编 QEMU」
#                                   #       「stdin:拒绝落进检出记录」「stdin:工具实现员跑 doc-lint.sh 拒绝（退出码 2）」「stdin:提交时才跑的检查被拒时 stderr 的出路点名 gate-staged.sh 与主 agent 照的那份做法」「stdin:实现员跑登记表外的 --selftest 拒绝（退出码 2）」
#                                   #       「stdin:登记表外的 --selftest 被拒时 stderr 点名登记表与登记的写法」「stdin:stage-selftest.sh 喂有起重活样本的阶段目录拒绝（退出码 2）」
#                                   #       「stdin:被拒时 stderr 点名起重活的那一道、写单跑一道样本的写法」「stdin:stage-selftest.sh --item 点名里夹起重活的拒绝（退出码 2）」「stdin:被拒时 stderr 点名起重活的那一道、写 --item 的写法」「stdin:实现员起含被拒命令的脚本拒绝（退出码 2）」「stdin:脚本里的拒绝 stderr 写出脚本路径与行号」
#                                   #       「stdin:实现员不经内存包装跑测试目标拒绝（退出码 2）」「stdin:不经内存包装的拒绝 stderr 写了出路」「stdin:脚本文件不存在时 stderr 点名它、记一条检出」「stdin:heredoc 写出的含 --all 的脚本拒绝（退出码 2）」「stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号」
#                                   #       「stdin:admission.py 判的那一刻抛异常时，点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 照拒（退出码 2）」（共 275 格）；
#                                   #       两档里的轻型那一批另有九格：「实现员不经内存包装起崩溃点重放 --quick：拒的是包装」「实现员起崩溃点重放入口不带参数（剪枝）：照拒」
#                                   #       「实现员起崩溃点重放 --item 不带 --quick（逐条）：照拒」「实现员带前缀起崩溃点重放 --full（全量）：照拒」「实现员经 gpu-quiesce-run.sh 起崩溃点重放不带参数：照拒」
#                                   #       「主 agent 不带前缀起崩溃点重放入口：照拒」「实现员起形式化那一道 --item（逐条）：照拒」「实现员起 crates 变异那一道 --quick 又带 --full：照拒」
#                                   #       「实现员经 gpu-quiesce-run.sh 起崩溃放量全量：照拒」
#                                   #   HEAVY_TEST_GUARD_BREAK=ignore-written-scripts  record_files_written() 不记，同一条命令里写出的脚本不认（执行时按盘上那份判，还不在的记「不存在」）
#                                   #       「主 agent 不带前缀同一条命令里先 cp 改名再起全量照拒」「主 agent 不带前缀同一条命令里先 cp 改名再起全量照拒：记几条检出」「实现员同一条命令里先 cp 改名再起全量照拒」「实现员同一条命令里先 cp 改名再起全量照拒：记几条检出」「主 agent 不带前缀同一条命令里 heredoc 写出带签名行的拷贝再起全量照拒」
#                                   #       「主 agent 不带前缀同一条命令里 heredoc 写出带签名行的拷贝再起全量照拒：记几条检出」「实现员 heredoc 写出不经内存包装的 cargo run 再起它」「实现员 heredoc 写出不经内存包装的 cargo run 再起它：记几条检出」「实现员 heredoc 写出不经内存包装的 cargo run 再起它：读进去几份脚本」
#                                   #       「实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它」「实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它：记几条检出」「实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它：读进去几份脚本」「实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行」
#                                   #       「实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行：记几条检出」「实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行：读进去几份脚本」「实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source」「实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source：记几条检出」
#                                   #       「实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source：读进去几份脚本」「实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它」「实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它：记几条检出」「实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它：读进去几份脚本」
#                                   #       「实现员 cd 进子目录写出、回来按相对路径起」「实现员 cd 进子目录写出、回来按相对路径起：记几条检出」「实现员 cd 进子目录写出、回来按相对路径起：读进去几份脚本」「实现员 heredoc 盖掉盘上不含重型命令的 light.sh 再起它：判写出的内容」「实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本」
#                                   #       「实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本：记几条检出」「实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本：读进去几份脚本」「实现员 bash -c 里写出含 --all 的脚本再起它」「实现员 bash -c 里写出含 --all 的脚本再起它：记几条检出」「实现员 bash -c 里写出含 --all 的脚本再起它：读进去几份脚本」
#                                   #       「实现员命令替换里写出含 --all 的脚本再起它」「实现员命令替换里写出含 --all 的脚本再起它：记几条检出」「实现员命令替换里写出含 --all 的脚本再起它：读进去几份脚本」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份：记几条检出」
#                                   #       「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份：读进去几份脚本」「实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在」「实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在：记几条检出」「实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在：读进去几份脚本」
#                                   #       「实现员 heredoc 写出不含重型命令的脚本再起它：放行、不记检出：记几条检出」「实现员 heredoc 写出不含重型命令的脚本再起它：放行、不记检出：读进去几份脚本」「主 agent 带前缀起 heredoc 写出的含 --all 的脚本：记几条检出」「主 agent 带前缀起 heredoc 写出的含 --all 的脚本：读进去几份脚本」
#                                   #       「实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件」「实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件：记几条检出」「实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件：读进去几份脚本」「实现员 cp -f 到目录里再起：读源文件」「实现员 cp -f 到目录里再起：读源文件：记几条检出」
#                                   #       「实现员 cp -f 到目录里再起：读源文件：读进去几份脚本」「实现员 cp 盘上不含重型命令的 light.sh 出来再起：放行、不记检出：记几条检出」「实现员 cp 盘上不含重型命令的 light.sh 出来再起：放行、不记检出：读进去几份脚本」「实现员 cp 同一条命令里 heredoc 写出的脚本再起」「实现员 cp 同一条命令里 heredoc 写出的脚本再起：记几条检出」
#                                   #       「实现员 cp 同一条命令里 heredoc 写出的脚本再起：读进去几份脚本」「实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判」「实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判：读进去几份脚本」「heredoc 写出的脚本，拒绝说明写出它的路径与正文里的行号」
#                                   #       「heredoc 写出的脚本里再写出的，逐层写出路径与行号」「stdin:heredoc 写出的轻脚本 stderr 不报「不存在」、不记检出」「stdin:heredoc 写出的含 --all 的脚本拒绝（退出码 2）」「stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号」（共 62 格）
#                                   #   HEAVY_TEST_GUARD_BREAK=ignore-memory-cap  decide() 不判子 agent 经没经内存包装跑编译出来的代码
#                                   #       「实现员 cd 进 harness 档裸跑、不经内存包装（拒的是包装，不是重型）」「实现员 -p harness 档不挑目标、不经内存包装（拒的是包装）」「实现员 /usr/bin/time 包一层跑自己的测试目标、没经内存包装」「实现员不经内存包装跑自己的测试目标」「执行员不经内存包装 cargo run 实验二进制」「实现员 cargo t 简写不经内存包装」
#                                   #       「执行员直接执行实验二进制不经内存包装」「实现员经 capped.sh 限了线程、没限内存」「实现员包装外面的命令替换不算在包装里」「实现员命令替换里不经内存包装跑」「实现员起的脚本里有不经内存包装的 cargo test」「实现员 heredoc 写出不经内存包装的 cargo run 再起它」
#                                   #       「stdin:实现员不经内存包装跑测试目标拒绝（退出码 2）」「stdin:不经内存包装的拒绝 stderr 写了出路」「实现员不经内存包装起崩溃点重放 --quick：拒的是包装」（共 15 格）
#                                   #   HEAVY_TEST_GUARD_BREAK=ignore-commit-checks  decide() 里提交时才跑的检查一律放行
#                                   #       「执行员跑 doc-text（不重型的阶段：提交时才跑的检查）」「书记员跑 doc-decisions（提交时才跑的检查）」「通用 agent 跑 code-tooling.sh 的 agent-write-scope 格（提交时才跑的检查）」「主 agent 不带前缀跑 code-tooling.sh（提交时才跑的检查）」「主 agent不带前缀跑 gate-lint.sh」
#                                   #       「工具实现员带前缀跑 rules-lint.sh 照拒」「工具实现员拿样本跑 stage-selftest.sh、阶段目录此刻不在：判不出喂的是哪几道，照拒」「工具实现员 stage-selftest.sh 喂有起重活样本的阶段目录（crates 变异表）：照拒」「实现员拿别处树里改过的 stage-selftest.sh 拷贝、不带 --item 喂全登记成轻的阶段目录：照拒」
#                                   #       「实现员 stage-selftest.sh --item 点名里夹一道起重活的（crates 变异表）：照拒」「实现员 stage-selftest.sh 喂样本里多一份起 checker 档的 setup.sh 的阶段目录：照拒」「实现员跑别处树里路径尾部相同的 claim-experiment.sh --selftest（正文起 checker 档）：照拒」
#                                   #       「实现员跑 gate-structure-check.py」「实现员跑登记表外的新脚本 --selftest，照拒」「实现员 mutate.sh 带表再带 --selftest（只认单独一个 --selftest；这样跑的是 crates 整表），照拒」
#                                   #       「实现员带前缀跑 layer0-shard-run.sh --selftest（设了 SINGLEFS_HEAVY_TESTS 换真 cargo 跑 checker 档包），照拒」「实现员只跑 research 单测那一格（不重型，照样是提交时才跑的检查）」「主 agent 不带前缀跑 hooks-registered.sh」
#                                   #       「实现员复现输入那一条（反引号里的 walk.rs）：末尾的 doc-lint.sh 是提交时才跑的检查、读它之前就拒，不记检出，一份脚本都不读」「造出来的登记表：实现员跑登记成 heavy 的 --selftest，照拒」「造出来的登记表：实现员 python3 通配起登记成 heavy 的自证，照拒」「stdin:工具实现员跑 doc-lint.sh 拒绝（退出码 2）」
#                                   #       「stdin:提交时才跑的检查被拒时 stderr 的出路点名 gate-staged.sh 与主 agent 照的那份做法」「stdin:实现员跑登记表外的 --selftest 拒绝（退出码 2）」「stdin:登记表外的 --selftest 被拒时 stderr 点名登记表与登记的写法」
#                                   #       「stdin:stage-selftest.sh 喂有起重活样本的阶段目录拒绝（退出码 2）」「stdin:被拒时 stderr 点名起重活的那一道、写单跑一道样本的写法」「stdin:stage-selftest.sh --item 点名里夹起重活的拒绝（退出码 2）」「stdin:被拒时 stderr 点名起重活的那一道、写 --item 的写法」（共 29 格）
#                                   #   HEAVY_TEST_GUARD_BREAK=ignore-heredoc-detached-run  written_detached_run() 不认同一条命令里 heredoc 写出的 detached-run.sh 拷贝（只认 cp 拷来的）
#                                   #       「主 agent 不带前缀同一条命令里 heredoc 写出带签名行的拷贝再起全量照拒」「主 agent 不带前缀同一条命令里 heredoc 写出带签名行的拷贝再起全量照拒：读进去几份脚本」（共 2 格）
#                                   #   HEAVY_TEST_GUARD_BREAK=read-light-selftest-body  heavy_uses() 把登记成不起重活的自证照样读进正文判
#                                   #       「工具实现员单独跑钩子的自证（登记成不起重活，放行）：读进去几份脚本」「实现员跑本仓那一份 claim-experiment.sh --selftest：放行：读进去几份脚本」「实现员直接执行门禁共用库（跑它的自证，登记成不起重活，放行、不读正文）：读进去几份脚本」（共 3 格）
#                                   #   HEAVY_TEST_GUARD_BREAK=drop-registry-hint  decide() 拒没登记的自证时不写怎么登记（拿掉 SELFTEST_REGISTRY_POLICY 那一句）
#                                   #       「stdin:登记表外的 --selftest 被拒时 stderr 点名登记表与登记的写法」（共 1 格）
#                                   #   HEAVY_TEST_GUARD_BREAK=main-full-policy  主 agent 没带前缀被拒时照旧整段贴规矩、不给加好前缀的那条命令
#                                   #       「stdin:主 agent 没带前缀被拒时最前面是能照抄的带前缀命令（照抄的那条再判放行）」「stdin:主 agent 被拒时规矩只留一行出处（不整段贴）」（共 2 格）
#                                   #   HEAVY_TEST_GUARD_BREAK=detection-outcome-dropped  检出记录不写结局（outcome：被拒 / 只记）
#                                   #       「stdin:被拒的检出记录写了结局「被拒」」「stdin:放行时的检出记录写了结局「只记」」（共 2 格）
#                                   #   （这四格在别的开关下也红：disable-check 下四格都红；ignore-commit-checks 下 main-full-policy 那两格红）
#                                   #   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（同目录 lib_work_root.py：工作根退回旧根）  检出记录的默认位置跟着退回旧根
#                                   #       「检出记录默认放在工作根下（lib_work_root.py）」
#                                   # 切词那一份（lib_shell_words.py）的弄坏开关下同样必须判红，照上面 HEAVY_TEST_GUARD_BREAK 那几个的写法逐个写（自检另原样转出
#                                   # lib_heavy_tests.py 自检的失败行，那几格写在 lib_heavy_tests.py 的文件头末尾；这里写的是 heavy-test-guard.sh 自己的格）：
#                                   #   LIB_SHELL_WORDS_BREAK=arguments-as-executed  `python3 -` 后面的词、`bash -n` 的目标当被执行的脚本
#                                   #       「工具实现员 bash -n 查门禁的语法」「工具实现员在门禁目录里 python3 - 读几份门禁（2026-09-28 误拦的那一条原样，目录换成自检的临时工作区）」
#                                   #       「实现员 bash -n 只查语法」三格（应当是 0，实际 2），「lib_heavy_tests.py 的自检」一格（应当是 0，实际 1）
#                                   #   LIB_SHELL_WORDS_BREAK=detached-run-force-kept  detached-run.sh 的参数不照它的 preflight 摘 --force（-- 之后、写在 --label 值位置上的不摘），`-- --force <重型命令>` 放过去
#                                   #       「主 agent 不带前缀经 detached-run.sh start、-- 之后先写 --force 再起全量照拒」「主 agent 不带前缀经 detached-run.sh start、--label 后面先写 --force 再起全量照拒」两格（应当是 2，实际 0），
#                                   #       「主 agent 不带前缀经 detached-run.sh start、--label 后面先写 --force 再起全量照拒：读进去几份脚本」一格（应当是 0，实际 1），「lib_heavy_tests.py 的自检」一格（应当是 0，实际 1）
#                                   # 判定那一半的弄坏开关（lib_heavy_tests.py 的 LIB_HEAVY_TESTS_BREAK、admission.py 的 ADMISSION_BREAK，写法在两份的文件头）设了哪一个也必须判红
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 write-guard.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import importlib.util, json, os, re, shlex, shutil, subprocess, sys, tempfile, time
from datetime import datetime, timezone
from typing import NamedTuple

def heavy_tests_library_path(hook_dir):
    return os.path.join(hook_dir, "lib_heavy_tests.py")

def load_heavy_tests(library_path):
    """判定的共用模块（它再导入同目录的 lib_shell_words.py）；读不到或导入出错就抛异常，由调用方处理。"""
    spec = importlib.util.spec_from_file_location("lib_heavy_tests", library_path)
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library

try:
    heavy_tests = load_heavy_tests(heavy_tests_library_path(sys.argv[1]))
    shell_words, shared_library_error = heavy_tests.shell_words, None
except Exception as error:  # 文件不在、语法错、导入时抛的都算读不到
    heavy_tests, shell_words, shared_library_error = None, None, error

def shared_library_problem(hook_dir):
    return f"读不到共用模块 {heavy_tests_library_path(hook_dir)}（它再导入同目录的 lib_shell_words.py）：{shared_library_error!r}"

try:   # 检出记录的默认位置在工作根下，工作根只在同目录的 lib_work_root.py 里定义（读不到时检出记录不写）
    sys.dont_write_bytecode = True
    sys.path.insert(0, sys.argv[1]); import lib_work_root as work_roots
    DEFAULT_DETECTIONS = work_roots.default_detections_path()
except Exception:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, DEFAULT_DETECTIONS = None, None
OCCASION_VARIABLE = "SINGLEFS_HEAVY_TESTS"
OCCASIONS = {"commit", "user-request"}
MAXIMUM_SCRIPT_DEPTH = 6        # 从命令文本起一层层读进去的脚本最多几层；再往里判「看不全」
MAXIMUM_SCRIPT_BYTES = 1 << 20  # 脚本超过这么大不读，判「读不到」
# 直接执行的文件先读开头这么多判是不是 shell 脚本（ELF 魔数、NUL 字节、`#!` 指到别的解释器、没有 `#!` 而扩展名不是 .sh），不是就不读也不记，之后才判大小上限：
# 编译产物常有几 MB，先判大小会把它记成「超过上限、没判」（2026-09-24 现场：research/target/release/e49_chain_width 5124256 字节）
SCRIPT_HEAD_BYTES = 4096
SHELL_SCRIPT_EXTENSION = ".sh"  # 直接执行、没有 `#!` 的文件，只有这个扩展名的当 shell 脚本读

# 提交时才跑的检查那一类的 kind（与 lib_heavy_tests.COMMIT_CHECK_KINDS 同一组；共用模块读不到时这里照样要有，自检核两边相同）
COMMIT_CHECK_KINDS = {"commit-check-stage", "commit-check-lint", "commit-check-selftest", "commit-check-selftest-unregistered", "commit-check-structure"}
# 没登记的自证（lib_heavy_tests.UNREGISTERED_SELFTEST_KIND）：拒绝时另写怎么登记
UNREGISTERED_SELFTEST_KIND = "commit-check-selftest-unregistered"
# 弄坏开关 HEAVY_TEST_GUARD_BREAK=<项>（几个用逗号连；只给自检证明点名的几格会红，项名与红的格写在文件头）
GUARD_BREAK_VARIABLE = "HEAVY_TEST_GUARD_BREAK"
KNOWN_GUARD_BREAK_ITEMS = ("disable-check", "ignore-written-scripts", "ignore-memory-cap", "ignore-commit-checks",
                           "ignore-heredoc-detached-run", "read-light-selftest-body", "drop-registry-hint", "main-full-policy",
                           "detection-outcome-dropped")
def guard_break_is_set(item):
    return item in os.environ.get(GUARD_BREAK_VARIABLE, "").split(",")

class HeavyUse(NamedTuple):
    test: object           # lib_heavy_tests.HeavyTest
    occasion: str | None   # 这一条带的 SINGLEFS_HEAVY_TESTS 值
    frames: tuple          # 读到它经过的脚本：((脚本路径, 行号或 None), …)，最后一格是它自己所在的那一行；命令文本里直接写的是 ()

class Notice(NamedTuple):
    frames: tuple          # 同 HeavyUse.frames：出在哪个脚本的哪一行，命令文本里直接写的是 ()
    text: str              # 读不到、看不全的是什么

class HeavyScan(NamedTuple):
    uses: list             # [HeavyUse]
    notices: list          # [Notice]：读不到、看不全的脚本，这条命令照常执行，记检出
    scripts_read: list     # 读进去判过的脚本路径，判一遍记一次，复用上一遍的结果不记（放行时分得出「读了、里面没有」与「根本没读」）
    hit_depth_limit: bool  # 这一层或更里面有脚本因为嵌套超过上限没读：结果跟着从第几层起走
    lowest_cycle_frame: int | None  # 这一层或更里面绕回外层脚本而没读时，绕回的那一格在帧里的最小下标；没绕回是 None
    uncapped: list = []    # [UncappedRun]：不经内存包装跑编译出来的代码的每一处（子 agent 要拒的那一道）

class UncappedRun(NamedTuple):
    detail: str            # lib_heavy_tests.runs_compiled_code 的说明（cargo test、直接执行 cargo 编出来的二进制 …）
    frames: tuple          # 同 HeavyUse.frames

class WrittenScript(NamedTuple):
    text: str | None          # heredoc 写出的正文；cp 拷来的是 None
    copied_from: str | None   # cp 拷来的：源文件的绝对路径；heredoc 写出的是 None

class FileOnDisk(NamedTuple):
    head: bytes            # 开头 SCRIPT_HEAD_BYTES 字节（判是不是 shell 脚本用）；读不了是 b""
    data: bytes | None     # 整份；超过 MAXIMUM_SCRIPT_BYTES 或读不了是 None
    size: int
    error: str | None      # 读不了的原因（OSError 的 strerror）；读得了是 None

class JudgingState:
    """一次判定里命令文本与读进去的各层脚本共用的：写出的文件、从盘上读过的文件、判过的脚本。"""
    def __init__(self):
        self.written = {}          # 实际路径 → WrittenScript：同一条命令里到这里为止写出的文件（record_files_written）
        self.written_version = 0   # written 每变一次加一；判过的脚本只在它自那一遍起没变时复用
        self.files = {}            # 实际路径 → FileOnDisk：一次判定里同一份文件只从盘上读一遍
        self.files_read = []       # 从盘上读过的文件的实际路径，按读的次序，每份一次
        self.judged = {}           # (实际路径, 当前目录, SINGLEFS_HEAVY_TESTS 的值, written_version, 经没经内存包装) → HeavyScan，
                                   # 帧去掉了起它那一行之上的几格（复用时接上这一处的）

class Decision(NamedTuple):
    code: int              # 0 放行，2 拒绝
    message: str | None    # 拒绝时写给 stderr 的原因与出路
    notices: list          # 读不到、看不全的脚本（写好的句子，出在脚本里的带路径与行号）
    scripts_read: list     # 同 HeavyScan.scripts_read
    files_read: list       # 同 JudgingState.files_read

# 命令位置上的词里有这些字符，就是没展开的变量、命令替换、通配、引号残片或别的语言的源码残片（逗号、等号），不是一个能读的路径
NOT_A_LITERAL_PATH = set("$`()'\"<>|&;*?[]{}=,\n")

LAUNCHER_WORDS = {"shell": "bash / sh 起的", "source": "source 读进来的", None: "直接执行的"}

def frames_text(frames):
    return " → ".join(f"{path}:{line}" if line else f"{path}（行号认不出）" for path, line in frames)

def command_lines(script_text):
    """脚本里每条命令（剥掉前缀与包装之后的词序列）出现在哪几行，按出现次序；
    逐个逻辑行（反斜杠续行接上）单独切词，喂给非 shell 命令的 heredoc 正文先换成同样多的空行，行号不变。"""
    lines = shell_words.strip_data_heredocs(script_text, keep_line_count=True).split("\n")
    found, number = {}, 0
    while number < len(lines):
        first_line, logical = number + 1, lines[number]
        while logical.endswith("\\") and number + 1 < len(lines):
            number += 1
            logical += "\n" + lines[number]
        number += 1
        for command in shell_words.commands_at_command_position(logical).commands:
            found.setdefault(tuple(command.words), []).append(first_line)
    return found

def literal_path(command, word):
    """命令里的一个词当文件路径（按这条命令的当前目录解开、取实际路径）；带没展开的变量、通配这类的，或目录认不出，交 None。"""
    if not word or NOT_A_LITERAL_PATH & set(word):
        return None
    resolved = shell_words.resolve_path(command.directory, os.path.expanduser(word))
    return None if resolved is None else os.path.realpath(resolved)

def heredoc_write_targets(command):
    """这一条 cat / tee 把喂给它的 heredoc 写进了哪些文件：[(目标词, 是不是追加)]；不是这种写法交 []。"""
    name, arguments = os.path.basename(command.words[0]), command.words[1:]
    if name not in ("cat", "tee") or not command.standard_input_heredocs:
        return []
    targets = []
    if name == "cat":
        if any(argument != "-" for argument in arguments):  # 带了文件参数（写出的是那些文件）或别的选项（-n 这类改了内容）
            return []
    else:
        only_files = False
        for argument in arguments:
            if not only_files and argument == "--":
                only_files = True
            elif only_files or not argument.startswith("-") or argument == "-":
                if argument != "-":
                    targets.append(argument)
        appends = any(argument in ("-a", "--append") or (argument.startswith("-") and not argument.startswith("--") and "a" in argument[1:])
                      for argument in arguments if argument != "--")
        targets = [(target, appends) for target in targets]
    for operator, target in command.output_redirections:
        if operator == ">&" and (target.isdigit() or target == "-"):
            continue
        targets.append((target, operator in (">>", "&>>")))
    return targets

def copy_pairs(command):
    """`cp` 这一条拷了哪些文件：[(源词, 目的词)]；不是 cp、或参数认不出，交 []。"""
    if os.path.basename(command.words[0]) != "cp":
        return []
    operands, target_directory, no_target_directory, only_operands = [], None, False, False
    arguments, position = command.words[1:], 0
    while position < len(arguments):
        argument = arguments[position]
        if not only_operands and argument == "--":
            only_operands = True
        elif not only_operands and argument in ("-t", "--target-directory", "-S", "--suffix"):
            if argument in ("-t", "--target-directory") and position + 1 < len(arguments):
                target_directory = arguments[position + 1]
            position += 1
        elif not only_operands and argument.startswith("--target-directory="):
            target_directory = argument.split("=", 1)[1]
        elif not only_operands and argument.startswith("-") and argument != "-":
            no_target_directory = no_target_directory or argument == "--no-target-directory" or (
                not argument.startswith("--") and "T" in argument[1:])
        else:
            operands.append(argument)
        position += 1
    if target_directory is not None:
        return [(source, os.path.join(target_directory, os.path.basename(source))) for source in operands]
    if len(operands) < 2:
        return []
    sources, destination = operands[:-1], operands[-1]
    destination_path = literal_path(command, destination)
    into_directory = not no_target_directory and (len(sources) > 1 or destination.endswith("/")
                                                   or (destination_path is not None and os.path.isdir(destination_path)))
    if into_directory:
        return [(source, os.path.join(destination, os.path.basename(source))) for source in sources]
    return [(sources[0], destination)]

def file_on_disk(path, state):
    """盘上一份普通文件的内容：一次判定里同一份（按实际路径认）只读一遍，读过的记进 state.files_read。"""
    real_path = os.path.realpath(path)
    known = state.files.get(real_path)
    if known is not None:
        return known
    try:
        size = os.path.getsize(real_path)
        with open(real_path, "rb") as handle:
            data = handle.read(MAXIMUM_SCRIPT_BYTES + 1) if size <= MAXIMUM_SCRIPT_BYTES else None
            head = handle.read(SCRIPT_HEAD_BYTES) if data is None else data[:SCRIPT_HEAD_BYTES]
        if data is not None and len(data) > MAXIMUM_SCRIPT_BYTES:  # 量过大小之后才长大的
            size, data = len(data), None
        known = FileOnDisk(head, data, size, None)
    except OSError as error:
        known = FileOnDisk(b"", None, 0, error.strerror)
    state.files[real_path] = known
    state.files_read.append(real_path)
    return known

def text_on_disk(path, state):
    """追加之前盘上那份的正文：还不在交 ""，不是普通文件、超过读的上限、读不了、不是 UTF-8 文本交 None。"""
    if not os.path.lexists(path):
        return ""
    if not os.path.isfile(path):
        return None
    found = file_on_disk(path, state)
    if found.data is None:
        return None
    try:
        return found.data.decode("utf-8")
    except UnicodeDecodeError:
        return None

def record_files_written(command, state):
    """这一条命令写出了哪些文件，记进 state.written（实际路径 → WrittenScript），后面的命令执行它们时拿这份内容判；每变一处 written_version 加一。"""
    if guard_break_is_set("ignore-written-scripts"):
        return
    written = state.written
    def remember(path, entry):
        written[path] = entry
        state.written_version += 1
    def forget(path):
        if written.pop(path, None) is not None:
            state.written_version += 1
    body = command.standard_input_heredocs[-1].text if command.standard_input_heredocs else None
    for target, appends in heredoc_write_targets(command):
        path = literal_path(command, target)
        if path is None:
            continue
        if not appends:
            remember(path, WrittenScript(body, None))
            continue
        entry = written.get(path)
        before = (text_on_disk(path, state) if entry is None else entry.text if entry.text is not None
                  else text_on_disk(entry.copied_from, state))
        if before is None:
            forget(path)  # 追加到读不了的文件上：不认，照旧读盘上那份
        else:
            remember(path, WrittenScript(before + body, None))
    for source, destination in copy_pairs(command):
        source_path, destination_path = literal_path(command, source), literal_path(command, destination)
        if source_path is None or destination_path is None:
            continue
        if source_path in written:
            remember(destination_path, written[source_path])
        elif os.path.isfile(source_path):
            remember(destination_path, WrittenScript(None, source_path))
        else:
            forget(destination_path)  # 源不在：认不出，执行时照旧按盘上有没有判

def has_shell_script_extension(path):
    """执行的名字、或它指到的文件，扩展名是 .sh。"""
    return path.endswith(SHELL_SCRIPT_EXTENSION) or os.path.realpath(path).endswith(SHELL_SCRIPT_EXTENSION)

def script_path_of(command, written):
    """命令位置上这一条执行的是不是一个要读进去的脚本文件：返回 (路径, None) 要读；(None, 找不到的说明) 记检出；(None, None) 不读也不记。
    written 是同一条命令里前面写出的文件（record_files_written），在里面的算找得到；直接执行的还不在，扩展名不是 .sh 的不读也不记。"""
    word = command.words[0]
    if command.launcher == "python" or word.startswith("-") or NOT_A_LITERAL_PATH & set(word):
        return None, None
    expanded = os.path.expanduser(word)
    if command.launcher is None and "/" not in expanded:
        return None, None
    resolved = shell_words.resolve_path(command.directory, expanded)
    if resolved is None:
        return None, None
    if os.path.realpath(resolved) in written or os.path.lexists(resolved):
        return resolved, None
    if command.launcher is not None and "/" not in expanded:
        found = shutil.which(expanded, mode=os.F_OK)
        if found:
            return os.path.abspath(found), None
    if command.launcher is None and not has_shell_script_extension(resolved):
        return None, None
    return None, f"{LAUNCHER_WORDS[command.launcher]}脚本 {resolved} 不存在"

def shebang_interpreter(data):
    """`#!` 那一行指到的解释器名（`#!/usr/bin/env bash` 取 bash）；没有 `#!` 交 None。"""
    if not data.startswith(b"#!"):
        return None
    end = data.find(b"\n")
    words = data[2:end if end >= 0 else len(data)].decode("utf-8", "replace").split()
    if not words:
        return ""
    interpreter = os.path.basename(words[0])
    if interpreter == "env":
        rest = [word for word in words[1:] if not word.startswith("-") and "=" not in word]
        interpreter = os.path.basename(rest[0]) if rest else ""
    return interpreter

def executed_file_is_shell_script(executed_path, head):
    """直接执行的文件当不当 shell 脚本读，只看开头：编译产物（ELF 魔数、NUL 字节）不算；有 `#!` 的看它指不指到 shell；
    没有 `#!` 的看扩展名是不是 .sh（`.rs`、`.py`、`.md`、没扩展名的文本都不算）。"""
    if head.startswith(b"\x7fELF") or b"\0" in head:
        return False
    interpreter = shebang_interpreter(head)
    if interpreter is not None:
        return interpreter in shell_words.SHELL_NAMES
    return has_shell_script_extension(executed_path)

def decoded_script_text(data, path, launcher, executed_path):
    """已经读到手的脚本字节（盘上读的、或同一条命令里 heredoc 写出的）：返回值同 script_text。"""
    how = LAUNCHER_WORDS[launcher]
    if launcher is None and not executed_file_is_shell_script(executed_path, data[:SCRIPT_HEAD_BYTES]):
        return None, None
    if len(data) > MAXIMUM_SCRIPT_BYTES:
        return None, f"{how}脚本 {path} 有 {len(data)} 字节，超过读的上限 {MAXIMUM_SCRIPT_BYTES} 字节"
    if b"\0" in data:
        return None, f"{how}脚本 {path} 不是文本（有 NUL 字节）"
    try:
        return data.decode("utf-8"), None
    except UnicodeDecodeError:
        return None, f"{how}脚本 {path} 不是文本（不是 UTF-8）"

def script_text(path, launcher, state, executed_path):
    """盘上 path 那份当脚本读（executed_path 是执行的名字，直接执行的按它的扩展名判）：返回 (正文, None) 要读；(None, 读不到的原因) 记检出；
    (None, None) 不是普通文件（目录、设备）或直接执行的不是 shell 脚本，不读也不记。"""
    how = LAUNCHER_WORDS[launcher]
    if not os.path.isfile(path):
        return None, None
    found = file_on_disk(path, state)
    if found.error is not None:
        return None, f"{how}脚本 {path} 读不了（{found.error}）"
    if launcher is None and not executed_file_is_shell_script(executed_path, found.head):
        return None, None
    if found.data is None:
        return None, f"{how}脚本 {path} 有 {found.size} 字节，超过读的上限 {MAXIMUM_SCRIPT_BYTES} 字节"
    return decoded_script_text(found.data, path, launcher, executed_path)

def script_text_to_judge(path, launcher, state):
    """执行时这个脚本里是什么：同一条命令里前面写出的拿写出的内容（cp 拷来的读源文件），其余读盘上那份；返回值同 script_text。"""
    entry = state.written.get(os.path.realpath(path))
    if entry is None:
        return script_text(path, launcher, state, path)
    if entry.copied_from is not None:
        return script_text(entry.copied_from, launcher, state, path)
    return decoded_script_text(entry.text.encode("utf-8"), path, launcher, path)

def written_detached_run(command, state):
    """命令词是同一条命令里前面写出的文件（cp 拷来的、heredoc 写出的），写出的内容开头带 detached-run.sh 的签名行，而参数是 `start … -- <命令>`：
    交回 (<命令> 的词, None)，同 command_under_launcher；别的交 None。弄坏开关 LIB_HEAVY_TESTS_BREAK=detached-run-by-name-only 下一律交 None。"""
    if heavy_tests.break_is_set("detached-run-by-name-only"):
        return None
    path = literal_path(command, command.words[0])
    entry = state.written.get(path) if path is not None else None
    if entry is not None and entry.copied_from is None and guard_break_is_set("ignore-heredoc-detached-run"):
        return None   # 弄坏开关：heredoc 写出的拷贝不认（只认 cp 拷来的）
    if entry is None:
        return None
    if entry.copied_from is not None:
        head = file_on_disk(entry.copied_from, state).head
    else:
        head = (entry.text or "")[:shell_words.DETACHED_RUN_HEAD_BYTES].encode("utf-8")
    if not shell_words.carries_detached_run_signature(head):
        return None
    return heavy_tests.command_under_detached_run(command.words, recognized=True)

def lower_cycle_frame(first, second):
    """两个「绕回外层的那一格」取靠外的那个；None 是没绕回。"""
    if first is None:
        return second
    return first if second is None else min(first, second)

def heavy_uses(text, directory, environment=None, frames_above=(), script=None, state=None, memory_capped=False, definitions=None):
    """text 里命令位置上的每一处重型用法；执行的是脚本文件就读进去，用同一套切词与判定递归判。
    definitions 是外层命令文本里的变量定义（剥掉 /usr/bin/time 这一类之后里面那条接着判时带进来）；命令词没展开的，连同这一段自己的定义交给
    lib_heavy_tests.unexpanded_command_word_use 判（点到 checker 档测试目标名的照直接执行它的测试二进制算）。读进去的脚本正文只用它自己的定义。
    script 是 text 所在的脚本（命令文本本身是 None），frames_above 是读到它为止经过的外层脚本与起它的那一行；
    state 是这一次判定里外各层共用的 JudgingState：脚本里写出的，外层后面的命令也看得见；同一份脚本只读一遍，判过的按键复用。
    memory_capped：text 整个跑在 run-with-memory-cap.sh 里（起它所在脚本的那一条经它包着）；不经它跑编译出来的代码的每一处记进 uncapped。"""
    if state is None:
        state = JudgingState()
    own_definitions = heavy_tests.variable_definitions(text)
    for name, values in (definitions or {}).items():
        own_definitions.setdefault(name, []).extend(values)
    uses, notices, scripts_read, occurrences, uncapped = [], [], [], {}, []
    hit_depth_limit, lowest_cycle_frame = False, None
    line_index = None
    def frames_of(key, occurrence):
        nonlocal line_index
        if script is None:
            return frames_above
        if line_index is None:
            line_index = command_lines(text)
        lines = line_index.get(key, [])
        return frames_above + ((script, lines[occurrence] if occurrence < len(lines) else None),)
    for command in shell_words.commands_at_command_position(text, directory, environment, memory_capped=memory_capped).commands:
        key = tuple(command.words)
        occurrence = occurrences.get(key, 0)
        occurrences[key] = occurrence + 1
        record_files_written(command, state)
        # /usr/bin/time、flock、systemd-run 这一类包在外面照样起那条命令的：剥掉它，里面那条按一段命令文本接着判（包装、脚本、内存包装照认）；
        # detached-run.sh start 同样剥（同一条命令里前面写出的拷贝按写出的内容认，其余按名字、符号链接、签名行认）
        launched = written_detached_run(command, state)
        if launched is None:
            launched = heavy_tests.command_under_launcher(command.words, command.directory) or heavy_tests.command_under_gpu_quiesce(command.words, command.directory)
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            inner = heavy_uses(shlex.join(inner_words), inner_start, command.environment, frames_of(key, occurrence), None, state, command.memory_capped,
                               own_definitions)
            uses += inner.uses
            notices += inner.notices
            uncapped += inner.uncapped
            scripts_read += inner.scripts_read
            hit_depth_limit = hit_depth_limit or inner.hit_depth_limit
            lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, inner.lowest_cycle_frame)
            continue
        compiled = heavy_tests.runs_compiled_code(command.words, command.directory)
        if compiled and not command.memory_capped:
            uncapped.append(UncappedRun(compiled, frames_of(key, occurrence)))
        test = (heavy_tests.classify(command.words, command.directory, command.environment)
                or heavy_tests.unexpanded_command_word_use(command.words, command.directory, command.environment, own_definitions))
        if test:
            uses.append(HeavyUse(test, command.environment.get(OCCASION_VARIABLE), frames_of(key, occurrence)))
            continue
        if heavy_tests.judged_by_name(command.words[0], command.directory):
            continue
        # 登记成不起重活的自证（单独一个 --selftest）判完就是判完：正文里平常那条路上起的 cargo 不是自证跑的，不读进去
        if (not guard_break_is_set("read-light-selftest-body")
                and heavy_tests.selftest_judged_light(command.words, command.directory, command.environment)):
            continue
        path, missing = script_path_of(command, state.written)
        if path is None and missing is None:
            continue
        here = frames_of(key, occurrence)
        if missing:
            notices.append(Notice(here, f"{missing}，里面的命令没判"))
            continue
        real_path = os.path.realpath(path)
        enclosing = [os.path.realpath(frame_path) for frame_path, _ in here]
        if real_path in enclosing:
            lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, enclosing.index(real_path))
            continue
        if len(here) + 1 > MAXIMUM_SCRIPT_DEPTH:
            hit_depth_limit = True
            notices.append(Notice(here, f"{LAUNCHER_WORDS[command.launcher]}脚本 {path} 是第 {len(here) + 1} 层，"
                                        f"超过读的上限 {MAXIMUM_SCRIPT_DEPTH} 层，看不全，里面的命令没判"))
            continue
        body, problem = script_text_to_judge(path, command.launcher, state)
        if problem:
            notices.append(Notice(here, f"{problem}，里面的命令没判"))
            continue
        if body is None:
            continue
        judged_key = (real_path, command.directory, command.environment.get(OCCASION_VARIABLE), state.written_version, command.memory_capped)
        earlier = state.judged.get(judged_key)
        if earlier is not None:
            uses += [use._replace(frames=here + use.frames) for use in earlier.uses]
            notices += [notice._replace(frames=here + notice.frames) for notice in earlier.notices]
            uncapped += [run._replace(frames=here + run.frames) for run in earlier.uncapped]
            continue
        scripts_read.append(path)
        inner = heavy_uses(body, command.directory, command.environment, here, path, state, command.memory_capped)
        uses += inner.uses
        notices += inner.notices
        uncapped += inner.uncapped
        scripts_read += inner.scripts_read
        hit_depth_limit = hit_depth_limit or inner.hit_depth_limit
        lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, inner.lowest_cycle_frame)
        # 它自己在帧里的下标是 len(here)：绕回比它更外的、撞上嵌套上限的、判的时候写出了文件的，换一处起就可能不一样，不复用
        depends_on_where_started = inner.hit_depth_limit or (inner.lowest_cycle_frame is not None and inner.lowest_cycle_frame < len(here))
        if not depends_on_where_started and state.written_version == judged_key[3]:
            state.judged[judged_key] = HeavyScan([use._replace(frames=use.frames[len(here):]) for use in inner.uses],
                                                 [notice._replace(frames=notice.frames[len(here):]) for notice in inner.notices],
                                                 [], False, None,
                                                 [run._replace(frames=run.frames[len(here):]) for run in inner.uncapped])
    return HeavyScan(uses, notices, scripts_read, hit_depth_limit, lowest_cycle_frame, uncapped)

POLICY = ("→ 规矩：重型测试就是 checker 档那一批（.claude/rules/verification.md）：checker 档包 singlefs-checker-tier 的测试（按包判，挑哪个目标都算）与 checker-tier-layer0-replay、QEMU、herd7、crates 变异整表、"
          "Verus 那一道门禁（checker-tier-formal-verification.sh，连同直接起它的判法脚本 research/scripts/verus-proof-check.py、research/scripts/e165-pin-red-examples.py "
          "的每一种用法，只有字面写明的 toolchain / fingerprint / tool-versions / list-items 与红例钉表的追加、--check、--backfill、--list-items 不算；直接起 verus 验证明、只打 --tool-versions 不算；"
          "research/scripts/prove-red.sh 带前缀起而点名行里有 checker 档的同算）、"
          "全量测试、整轮门禁、全部实验复跑、E152 装置、广谱变异（research/scripts/broad-mutation.py 起 cargo 或改库目录的每一种用法，只有 --list-items / --summary / --check-ignored / --help（-h）不算）、真编 QEMU（research/scripts/build-patched-qemu.sh 除 --selftest 之外的用法），"
          "崩溃点重放的重型（research/scripts/crash-point-replay.sh 不带参数、--item、--full），"
          "只在提交代码时跑一次、或用户要求时跑；"
          "子 agent 一律不跑，只跑 harness 档——自己动到的 singlefs-harness 与别的非 checker 档包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）与 fmt / clippy / build——"
          "与轻型三条（`bash research/scripts/run-with-memory-cap.sh <上限> bash research/scripts/crash-point-replay.sh --quick`、"
          "`… bash .claude/gate.d/checker-tier-formal-verification.sh --quick <根>`、`… bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick <根>`，"
          "都可再带 `--item <项>`，要卡的套 `bash research/scripts/gpu-quiesce-run.sh --`）；"
          "主 agent 在提交流程里跑要带 `SINGLEFS_HEAVY_TESTS=commit`，用户要求时带 `SINGLEFS_HEAVY_TESTS=user-request`。\n"
          "→ 主 agent 怎么跑：提交时的六道重阶段照 .claude/skills/crash-test/commit-stages.md，整轮门禁照 .claude/skills/gate/staged-gate.md；都带那个前缀，都不跑全量 `cargo test`。\n"
          "→ 提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑；"
          "子 agent 在交回报告「要主 agent 跑的命令」一节写明要跑什么、为什么，交主 agent（派发提示里点名要你跑的也一样，写明被这道闸拒了）。")
COMMIT_CHECK_POLICY = ("→ 规矩：提交时才跑的检查——扫仓的门禁阶段（重型那几道之外的）、上游的 doc-lint / gate-lint / shell-lint / preflight-lint / gate-overlap / "
                       "rules-lint / hooks-registered / stage-selftest、research/scripts/gate-structure-check.py、"
                       "研究脚本 / 钩子 / `.claude/gate.d/lib/` 共用库的自证里登记成起重活的、没登记的、带了 `--selftest` 之外参数的"
                       "——归提交前主 agent 照 .claude/skills/gate/staged-gate.md 跑的 research/scripts/gate-staged.sh（它起的整轮 gate.sh 在里面起这些，不经这道闸）；"
                       "主 agent 带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）跑，子 agent 不跑，主 agent 不带前缀不跑。\n"
                       "→ 交回前不跑它们：改了门禁、脚本或钩子的，在交回里逐份写明改了什么、各自的样本与自证格和弄坏开关，交主 agent 排进提交前那一趟；"
                       "只查清单的放行（阶段的 `--list` / `--list-items`、`gate-overlap.py --list`），阶段的 `--write`（生成）放行；"
                       "登记表 .claude/hooks/selftest-registry.tsv 里记成 light 的自证，单独写 `<脚本> --selftest` 谁都能跑；"
                       "门禁样本：stage-selftest.sh 喂的阶段目录里有样本的每一道在同一张表里都登记成 samples-light / samples-light-marker: 的谁都能跑，"
                       "只喂几道写 `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item <阶段文件名>`（可给几次），点名的每一道都登记成轻的谁都能跑，"
                       "`--list-items` 列可点名的阶段；"
                       "登记了假工具标记的那一道（checker-tier-layer0-replay、herd7、QEMU）cd 进放了标记的样本根、根参数写同一个目录直接起也谁都能跑。")
SELFTEST_REGISTRY_POLICY = ("→ 自证的轻重按登记表 .claude/hooks/selftest-registry.tsv 判（判法在 .claude/hooks/lib_heavy_tests.py 的 classify_selftest）："
                            "只管单独一个 `--selftest`（门禁共用库：单独执行或单独带 `--selftest`），带了别的参数的写成单独一条再跑。"
                            "表外的新脚本要登记：读它 `--selftest` 走的代码，起的 cargo / rustc / qemu / herd7 / verus 都换成了假的、或根本不起，"
                            "就加一行 `<相对仓根的路径><制表符>light<制表符># <理由：起的是什么、怎么换成假的>`；自证本身起这类重活的写 `heavy`，"
                            "只在设了某个变量时才换真家伙的写 `heavy-if-set:<变量>`。表归 tooling-writer 改，别的子 agent 在交回里写明要登记哪一份、理由，交主 agent 派。"
                            "登记表认的是本仓那一份文件：仓副本、worktree 里路径尾部相同的拷贝与写在变量里的路径不按登记表放行，跑本仓那一份写它在本仓的字面路径。")
MEMORY_CAP_POLICY = ("→ 规矩：子 agent 跑编译出来的代码（cargo test / cargo run / cargo bench、直接执行 cargo 编出来的二进制）要经内存包装跑："
                     "`bash research/scripts/run-with-memory-cap.sh <上限> <命令>`（上限例 4G）——它先判整机放不放得下、放不下就排队，撞了上限只杀这一条，"
                     "不把整机拖进 OOM；写在脚本里的，整条脚本经它跑：`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`。"
                     "只编不跑的（cargo build / clippy、cargo test --no-run）不用经它；要限时设 RUN_WITH_MEMORY_CAP_TIME_LIMIT=<秒>，别在包装外面套 timeout（排队的时间也会算进去）。")
SCRIPT_POLICY = ("→ 这一步写在脚本里：命令位置上执行的脚本（bash / sh / ./ / source 起的，经 capped.sh / nice / timeout / env 包一层的，脚本里再起的脚本）"
                 "读进去逐条照同一张表判。把那一步从脚本里拿掉再跑，或照上一条交主 agent。")
# 主 agent 没带前缀被拒：最前面给加好前缀、再判放行的那条命令，规矩只贴出处一行（规矩正文在下面这几份里，不在拒绝信息里整段贴）
MAIN_AGENT_RULE_POINTER = ("→ 规矩只贴出处：.claude/rules/implementation-workflow.md「checker 档只在提交时跑，harness 随时跑」"
                           "（重型测试与提交时才跑的检查各是哪些、谁跑、什么时候跑）；整轮门禁照 .claude/skills/gate/staged-gate.md 跑 research/scripts/gate-staged.sh，"
                           "提交时的六道重阶段照 .claude/skills/crash-test/commit-stages.md。")
COPY_COMMAND_LIMIT = 2000   # 加好前缀的命令超过这么多字就不整条贴，只写前缀加在哪
LEADING_OCCASION = re.compile(r"\A\s*" + OCCASION_VARIABLE + r"=\S*[ \t]+")

def main_agent_still_refused(command, directory):
    """主 agent 跑这一条，照同一张表还有一处要拒就交 True（给加好前缀的那条再判一遍用）。"""
    return any(refusal_reason(use.test.kind, use.occasion, None) for use in heavy_uses(command, directory, state=JudgingState()).uses)

def prefixed_command(command, directory):
    """主 agent 被拒的那条加好 SINGLEFS_HEAVY_TESTS=commit 的写法与它的式样（"prefix" 写在命令前面 / "export" 开头 export 一句）：
    开头已有这个变量的先摘掉；两种都照同一张表再判一遍，放行的才交，都还拒（前缀到不了被拒的那一步）交 (None, None)。"""
    bare = LEADING_OCCASION.sub("", command, count=1)
    for form, candidate in (("prefix", f"{OCCASION_VARIABLE}=commit {bare}"), ("export", f"export {OCCASION_VARIABLE}=commit; {bare}")):
        if not main_agent_still_refused(candidate, directory):
            return candidate, form
    return None, None

def main_agent_copy_lines(command, directory):
    """主 agent 被拒时放在最前面的几行：能照抄的那条命令（太长的只写前缀加在哪），或照抄不了的原因。"""
    candidate, form = prefixed_command(command, directory)
    if candidate is None:
        return ["→ 照抄不了：命令最前面加前缀也到不了被拒的那一步（写在 env -i 之后、另起的登录 shell 里这类），"
                f"把 {OCCASION_VARIABLE}=commit 写在起那一步的那条命令前面再跑；用户要求时写 =user-request。"]
    if len(candidate) > COPY_COMMAND_LIMIT:
        written = f"`{OCCASION_VARIABLE}=commit `" if form == "prefix" else f"`export {OCCASION_VARIABLE}=commit; `"
        return [f"→ 照抄：这条命令 {len(candidate)} 字，不整条贴；在它最前面加 {written}（加上之后照同一张表再判过，放行）；用户要求时把 commit 换成 user-request。"]
    return ["→ 照抄（提交流程里跑；用户要求时把 commit 换成 user-request，提交之外要跑先弹窗问用户）：", candidate]

def refusal_reason(kind, occasion, agent_type):
    """放行返回 None；拒绝返回一句原因。子 agent 一样都不跑，主 agent 要带前缀。"""
    category = heavy_tests.KIND_CATEGORY[kind]
    if agent_type:
        return f"{agent_type} 不跑「{category}」"
    if occasion not in OCCASIONS:
        carried = "没带" if occasion is None else f"带的是 {OCCASION_VARIABLE}={occasion}"
        return f"主 agent 跑「{category}」要带 {OCCASION_VARIABLE}=commit 或 =user-request，这一条{carried}"
    return None

def decide(hook_input, project_root):
    """返回 Decision：code 0 放行、2 拒绝；notices 是读不到、看不全的脚本，放不放行都照记。"""
    if guard_break_is_set("disable-check"):
        return Decision(0, None, [], [], [])
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    agent_type = hook_input.get("agent_type") or None
    directory = hook_input.get("cwd") or project_root
    state = JudgingState()
    scan = heavy_uses(command, directory, state=state)
    notices = [f"脚本 {frames_text(notice.frames)} 里{notice.text}" if notice.frames else notice.text for notice in scan.notices]
    refused, commit_check_refused, in_script, unregistered_selftest = [], [], False, False
    for use in scan.uses:
        reason = refusal_reason(use.test.kind, use.occasion, agent_type)
        if reason:
            commit_check = use.test.kind in COMMIT_CHECK_KINDS
            if commit_check and guard_break_is_set("ignore-commit-checks"):
                continue   # 弄坏开关：提交时才跑的检查一律放行（只给自检证明它会红）
            where = f"脚本 {frames_text(use.frames)} 里的 " if use.frames else ""
            in_script = in_script or bool(use.frames)
            unregistered_selftest = unregistered_selftest or use.test.kind == UNREGISTERED_SELFTEST_KIND
            (commit_check_refused if commit_check else refused).append(f"{where}{use.test.detail}（{use.test.category}）：{reason}")
    uncapped_refused = []
    if agent_type and not guard_break_is_set("ignore-memory-cap"):
        for run in scan.uncapped:
            where = f"脚本 {frames_text(run.frames)} 里的 " if run.frames else ""
            in_script = in_script or bool(run.frames)
            uncapped_refused.append(f"{where}{run.detail}（{agent_type} 不经 run-with-memory-cap.sh）")
    if not refused and not commit_check_refused and not uncapped_refused:
        return Decision(0, None, notices, scan.scripts_read, state.files_read)
    lines, policies = [], []
    if refused:
        lines += [f"✗ 重型测试被拒：{refused[0]}"] + [f"  另有：{entry}" for entry in refused[1:]]
        policies.append(POLICY)
    if commit_check_refused:
        lines += [f"✗ 提交时才跑的检查被拒：{commit_check_refused[0]}"] + [f"  另有：{entry}" for entry in commit_check_refused[1:]]
        policies.append(COMMIT_CHECK_POLICY)
        if unregistered_selftest and not guard_break_is_set("drop-registry-hint"):
            policies.append(SELFTEST_REGISTRY_POLICY)   # 弄坏开关 HEAVY_TEST_GUARD_BREAK=drop-registry-hint 拿掉这一句（只给自检证明它会红）
    if uncapped_refused:
        lines += [f"✗ 不经内存包装跑编译出来的代码被拒：{uncapped_refused[0]}"] + [f"  另有：{entry}" for entry in uncapped_refused[1:]]
        policies.append(MEMORY_CAP_POLICY)
    if in_script:
        policies.append(SCRIPT_POLICY)
    if agent_type is None and not guard_break_is_set("main-full-policy"):
        # 主 agent 被拒的只有「没带前缀」这一种：照抄的那条放最前面，规矩只留出处（弄坏开关 main-full-policy 下照旧整段贴）
        message = "\n".join(main_agent_copy_lines(command, directory) + lines + [MAIN_AGENT_RULE_POINTER])
    else:
        message = "\n".join(lines + policies)
    return Decision(2, message, notices, scan.scripts_read, state.files_read)

def record_detection(hook_input, findings, detections_path, outcome):
    """检出记录追加一行；outcome 是这条命令的结局：「被拒」（执行前拒了、没执行）或「只记」（照常执行）。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": (tool_input.get("command") or "")[:500],
        "findings": findings,
    }
    if not guard_break_is_set("detection-outcome-dropped"):
        entry["outcome"] = outcome
    if not detections_path:   # 读不到 lib_work_root.py、又没设 AGENT_HOOK_DETECTIONS：不知道往哪写
        return
    try:
        os.makedirs(os.path.dirname(detections_path), exist_ok=True)
        with open(detections_path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass

def build_sample_scripts(work):
    """自检用的脚本：被拒的那一条（cargo test --all）写在 s.sh 第 4 行；另有嵌套、只含轻命令、读不到、看不全与不读的几种。"""
    def write(relative, content, mode="w"):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, mode) as handle:
            handle.write(content)
    write("s.sh", "#!/usr/bin/env bash\n# 验证链：先编，再跑\nset -euo pipefail\ncargo test --all\n")
    write("research/scripts/prove-red.sh", "#!/usr/bin/env bash\n# 样本（证红脚本的替身，正文只有一行轻命令）\necho sample\n")
    write("outer.sh", "#!/usr/bin/env bash\necho 起里一层\nbash s.sh\n")
    write("own-prefix.sh", "SINGLEFS_HEAVY_TESTS=commit cargo test --all\n")
    write("light.sh", "#!/usr/bin/env bash\nbash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract\necho \"cargo test --all\"\n"
                      "cat > note.md <<'EOF'\ncargo test --all\nEOF\ncargo clippy --all-targets -- -D warnings\n")
    write("loop.sh", "#!/usr/bin/env bash\n[ -n \"${AGAIN:-}\" ] || AGAIN=1 bash loop.sh\n")
    for level in range(1, 8):
        write(f"deep{level}.sh", f"bash deep{level + 1}.sh\n" if level < 7 else "cargo test --all\n")
    write("blob.bin", b"\x00\x01cargo test --all\n", "wb")
    write("big.sh", "# 填充\n" * (MAXIMUM_SCRIPT_BYTES // 8 + 1) + "cargo test --all\n")
    write("tool", "#!/usr/bin/env python3\nimport subprocess\nsubprocess.run(['cargo', 'test', '--all'])\ncargo test --all\n")
    write("prog", b"\x7fELF\x02\x01\x01\x00\x00\ncargo test --all\n", "wb")
    # 超过读的上限的编译产物与 python 程序：先认出不是 shell 脚本、不读也不记，轮不到大小上限（2026-09-24 现场误记过 e49_chain_width）
    write("bigprog", b"\x7fELF\x02\x01\x01\x00" + b"\x00" * (MAXIMUM_SCRIPT_BYTES + 64) + b"\ncargo test --all\n", "wb")
    write("bigtool", "#!/usr/bin/env python3\n" + "# 填充\n" * (MAXIMUM_SCRIPT_BYTES // 8 + 1) + "cargo test --all\n")
    write("elsewhere/mutate.sh", "#!/usr/bin/env bash\ncargo test --all\n")
    write("research/scripts/gate-staged.sh", "#!/usr/bin/env bash\nbash .claude/scripts/gate.sh --staged\n")
    # 查名字与数组赋值：照 .claude/scripts/fetch-deps.sh 第 44 行、research/scripts/change-touches-crates.sh 第 61 行原样
    write("command-lookup.sh", "#!/usr/bin/env bash\n  if command -v herd7 >/dev/null 2>&1; then ok \"herd7 已在\"; return 0; fi\n")
    write("array-assignment.sh", "#!/usr/bin/env bash\n  ((${#prefixes[@]})) || prefixes=(\"crates/\")\nchains=(./s.sh ./outer.sh)\n")
    # 命令替换的两种写法，照 .claude/singlefs-ai-sop/scripts/doc-lint.sh 第 43 行、第 70 行原样：切碎之后的片段不许当成脚本路径去读
    write("doc-lint-line-43.sh", "#!/usr/bin/env bash\n  set -- \"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\" && pwd)\"\n")
    write("doc-lint-line-70.sh", "#!/usr/bin/env bash\n"
          "DOC_LINT_FAMILY=\"$(sed -n 's/^family=//p' \"$(dirname \"${BASH_SOURCE[0]}\")/../I18N\" 2>/dev/null || true)\"\n")
    write("substitution-fragments.sh", "#!/usr/bin/env bash\ncd $(dirname $0)/.. && echo $((total/2)) `dirname $0`/x.sh\n"
          "$(dirname \"$0\")/helper.sh --flag\n`dirname $0`/helper.sh\n"
          "out=\"$(bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract 2>&1)\"\n")
    # 不经内存包装跑一个测试目标：子 agent 起它要拒（经包装起它放行）
    write("unwrapped.sh", "#!/usr/bin/env bash\ncargo test -p singlefs-core --test core_contract\n")
    write("substitution-heavy.sh", "#!/usr/bin/env bash\n  set -- \"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\" && pwd)\"\n"
          "summary=\"$(cargo test --all 2>&1 | tail -3)\"\n")
    # 直接执行时不当脚本读的文本：Rust 源码的文档注释里反引号括着彼此的路径与 `cargo test --all`，当 shell 读就一路互相读下去（2026-09-25 现场一条命令 100 秒没判完）
    checker_sources = ["walk.rs", "image.rs", "lib.rs"]
    for name in checker_sources:
        others = "` 与 `".join(f"crates/singlefs-checker/src/{other}" for other in checker_sources if other != name)
        write(f"crates/singlefs-checker/src/{name}", f"//! 池级 checker：见 `{others}`\n/// 跑法：`cargo test --all`\npub fn check_pool_image() {{}}\n")
    write("notes.md", "# 验证链\ncargo test --all\n")
    write("plain-text", "cargo test --all\n")
    write(".claude/singlefs-ai-sop/scripts/doc-lint.sh", "#!/usr/bin/env bash\necho doc-lint\n")
    # 直接执行时当脚本读的：没扩展名而 `#!` 指到 shell 的，扩展名不对而 `#!` 指到 shell 的
    write("runner", "#!/usr/bin/env bash\ncargo test --all\n")
    write("mislabeled.py", "#!/bin/sh\ncargo test --all\n")
    # 一条命令里同一份脚本起几遍：twice.sh 三种起法各起一遍 s.sh；cyc-a.sh 与 cyc-b.sh 互相调；rel.sh 按相对路径起的脚本随当前目录变
    write("twice.sh", "#!/usr/bin/env bash\nbash s.sh\n./s.sh\nsource s.sh\n")
    write("cyc-a.sh", "bash cyc-b.sh\ncargo test --all\n")
    write("cyc-b.sh", "bash cyc-a.sh\n")
    write("rel.sh", "bash ./inner-rel.sh\n")
    write("inner-rel.sh", "echo 轻\n")
    write("sub/inner-rel.sh", "cargo test --all\n")
    write("has-missing.sh", "bash missing.sh\n")
    write("outer-missing.sh", "echo 起里一层\nbash has-missing.sh\n")

# 2026-09-25 现场一条命令 100 秒没判完：双引号里没转义的反引号把 `crates/singlefs-checker/src/walk.rs` 当命令替换执行（原样，cwd 换成自检的临时工作区）
SLOW_REPRODUCTION_COMMAND = 'python3 research/scripts/replace-once.py .claude/kb/verification-build.md "池级 checker（\\`crates/singlefs-checker\\` 的 \\`walk::check_pool_image\\`，24 条）" "池级 checker（\\`crates/singlefs-checker\\` 的 \\`walk::check_pool_image\\`，26 条：第一版 23 条加 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽））" && python3 research/scripts/replace-once.py .claude/kb/milestone/02-second-txn.md "checker 新接的只有 I-3.8（实例表行唯一且低于挂载根），I-3.1（已分配统计对得上） 与 I-2.1（校验和与内容匹配） 改按回退候选集判，预想清单里其余各条（I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽） 那一族起）还没有 checker；" "checker 新接了 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽）（后两条按候选集里每条根各判一格：走读它引用的单元时校验和对不上或头用不了就红，坏镜像是 mkfs 树表单元——只有第 0 代根还引用——改内容重封 / 抹成零；`crates/singlefs-checker/src/walk.rs`、`image.rs` 清单 26 条），I-3.1（已分配统计对得上） 与 I-2.1（校验和与内容匹配） 改按回退候选集判；预想清单里其余各条（I-3.4（可用空间扣待删占用） 起）还没有 checker；" && bash .claude/singlefs-ai-sop/scripts/doc-lint.sh . 2>&1 | tail -1'
SLOW_REPRODUCTION_SECONDS = 1.0  # 复现输入那一条要在这么多秒内判完

def selftest(hook_dir):
    if heavy_tests is None:
        print(f"  ✗ 自检：{shared_library_problem(hook_dir)}")
        print("    → 怎么办：恢复 .claude/hooks/lib_heavy_tests.py 与 lib_shell_words.py（判定与切词只有那两份，别在 hook 里再抄一份），再跑 --selftest")
        return 1
    work = tempfile.mkdtemp(prefix="heavy-test-guard-")
    results = []
    # 登记表那几格起的是本仓里那几份脚本（键按解析后的真实路径认）；喂 stage-selftest.sh 的样本拿临时样本仓的暂存区比（lib_heavy_tests 的 SAMPLE_INDEX_REPOSITORY）
    repo, real_sample_index = heavy_tests.HOOK_REPOSITORY, heavy_tests.SAMPLE_INDEX_REPOSITORY
    try:
        heavy_tests.build_sample_workspace(work)
        heavy_tests.SAMPLE_INDEX_REPOSITORY = os.path.join(work, heavy_tests.SAMPLE_INDEX_REFERENCE_NAME)
        build_sample_scripts(work)
        writer, crash, triage, scribe = "implementation-writer", "crash-verifier", "gate-triage", "kb-scribe"
        commit, request = "SINGLEFS_HEAVY_TESTS=commit ", "SINGLEFS_HEAVY_TESTS=user-request "
        tier_binary = "./target/release/deps/crash_enumeration_new_pool_file_creation_stream-0123456789abcdef"
        # (说明, agent_type（None 是主 agent）, 命令, 该拒 2 / 该放 0[, 该记几条检出, 该读进去几份脚本]；后两列不写是 0、0)
        cases = [
            ("实现员跑 cargo test --all", writer, "cargo test --all", 2),
            ("实现员跑 checker 档包的测试（checker 档按包判）", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier", 2),
            ("实现员跑 checker 档包点名一个快档目标", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier --test checker_quick_stream", 2),
            ("实现员跑判决器库 singlefs-checker 的单测不算", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker --lib", 0),
            ("实现员跑 checker 档包的库单测也拒", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier --lib", 2),
            ("主 agent带前缀经包装跑 checker 档包快档", None, commit + "nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker-tier", 0),
            ("主 agent 不带前缀跑 checker 档包", None, "bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker-tier", 2),
            ("主 agent 带 =user-request 跑 checker 档包", None, request + "bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker-tier", 0),
            ("实现员跑 checker 档的集成测试", writer, "cargo test --release -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream", 2),
            ("实现员跑 check.sh", writer, "bash .claude/scripts/check.sh", 2),
            ("实现员经 capped.sh 跑 --workspace", writer, "bash research/scripts/capped.sh 4 cargo test --workspace", 2),
            ("实现员经 run-with-memory-cap.sh 跑 --workspace", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test --workspace", 2),
            ("实现员单独跑 run-with-memory-cap.sh --selftest（登记成不起重活，放行）", writer, f"bash {repo}/research/scripts/run-with-memory-cap.sh --selftest", 0),
            ("实现员在 bash -c 里跑 --all", writer, "bash -c 'cargo test --all'", 2),
            ("实现员在仓根裸跑 cargo test", writer, "cargo test", 2),
            ("实现员 cd 进 harness 档裸跑、不经内存包装（拒的是包装，不是重型）", writer, "cd crates/singlefs-harness && cargo test --release", 2),
            ("实现员 cd 进 harness 档裸跑、经内存包装放行（harness 档随时跑）", writer, "cd crates/singlefs-harness && bash ../../research/scripts/run-with-memory-cap.sh 4G cargo test --release", 0),
            ("实现员 -p harness 档不挑目标、不经内存包装（拒的是包装）", writer, "cargo test -p singlefs-harness", 2),
            ("实现员 -p harness 档不挑目标、经内存包装放行", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness", 0),
            ("实现员 -p checker 档 --test 通配", writer, "cargo test -p singlefs-checker-tier --test '*'", 2),
            ("实现员前缀一串再跑 --workspace", writer, "nice -n 19 timeout 600 env A=1 cargo +stable --offline test --workspace", 2),
            ("实现员在 research 工作区根裸跑", writer, "cd research && cargo test --release", 2),
            ("实现员在 research 唯一的成员里裸跑", writer, "cd research/e7-index-bench && cargo test", 2),
            ("实现员带前缀也拒", writer, commit + "cargo test --all", 2),
            ("实现员 mutate.sh 跑 crates 整表", writer, "cd research && bash scripts/mutate.sh x y ../crates/mutations.tsv", 2),
            ("实现员起 qemu-system", writer, "qemu-system-x86_64 -enable-kvm -m 512", 2),
            ("实现员 vm-bench.sh --selftest", writer, "bash research/scripts/vm-bench.sh --selftest", 2),
            ("实现员跑 herd7", writer, "herd7 -conf linux-kernel.cfg litmus/a.litmus", 2),
            ("实现员跑 lkmm.sh", writer, "bash .claude/scripts/lkmm.sh", 2),
            ("实现员跑 Verus 那一道门禁", writer, "bash .claude/gate.d/checker-tier-formal-verification.sh --check verus-proof", 2),
            ("主 agent 不带前缀跑 Verus 那一道", None, "bash .claude/gate.d/checker-tier-formal-verification.sh", 2),
            ("主 agent 带前缀跑 Verus 那一道", None, commit + "bash .claude/gate.d/checker-tier-formal-verification.sh", 0),
            ("实现员带前缀跑 Verus 那一道照拒", writer, commit + "bash .claude/gate.d/checker-tier-formal-verification.sh", 2),
            ("实现员直接起 verus 验自己写的证明：不算重型", writer, "verus --crate-type=lib --no-cheating crates/singlefs-core/src/lib.rs", 0),
            # Verus 那一道的两份判法脚本直接起：干活的用法照那一道判（lib_heavy_tests.py 的 verus-judges-by-stage-only 下前四格红）；
            # 只打工具版本的谁都能跑（tool-versions-counted 下末两格红）
            ("实现员直接起判法脚本跑必红世界", writer, "python3 research/scripts/verus-proof-check.py must-red-worlds .", 2),
            ("主 agent 不带前缀直接起判法脚本跑必红世界", None, "python3 research/scripts/verus-proof-check.py must-red-worlds .", 2),
            ("调查员经内存包装起红例重放", "investigator", "bash research/scripts/run-with-memory-cap.sh 4G python3 research/scripts/e165-pin-red-examples.py --replay --root .", 2),
            # 红例钉表只有 --replay 起装置；参数里的变量拼进 --table、--root 的值不算（LIB_HEAVY_TESTS_BREAK=e165-pin-unexpanded-as-replay 下前两格红）
            ("工具实现员红例钉表 --check --table 的值是变量：放行", "tooling-writer",
             "D=/tmp/x; python3 research/scripts/e165-pin-red-examples.py --check --table $D/dry-2c.tsv research/results/e165-a-red-examples.tsv", 0),
            ("工具实现员 for 循环里红例钉表 --check --root：放行", "tooling-writer",
             "for s in green red; do nice -n 19 python3 research/scripts/e165-pin-red-examples.py --check --root $W/fixture-copies/$s; done", 0),
            ("工具实现员 for 循环先 --check 再 --replay：照拒", "tooling-writer",
             "for m in --check --replay; do python3 research/scripts/e165-pin-red-examples.py $m; done", 2),
            # prove-red.sh 带前缀起照跑点名行里重型的那几行（LIB_HEAVY_TESTS_BREAK=prove-red-unjudged 下第一格红）
            ("实现员带前缀起 prove-red、点名 checker 档包的行：照拒", writer, commit + "bash research/scripts/prove-red.sh --copy . singlefs-checker-tier tier-row", 2),
            ("实现员不带前缀起 prove-red、点名 checker 档包的行：放行（重型的行它报本次未跑）", writer,
             "bash research/scripts/prove-red.sh --copy . singlefs-checker-tier tier-row", 0, 0, 1),
            ("主 agent带前缀起 prove-red、点名 checker 档包的行：放行", None, commit + "bash research/scripts/prove-red.sh --copy . singlefs-checker-tier tier-row", 0),
            ("实现员带前缀起 prove-red、点名的只有 harness 档包的行：放行", writer, commit + "bash research/scripts/prove-red.sh --copy . singlefs-harness harness-row", 0, 0, 1),
            # 命令词是变量、循环里 ls 出来的 checker 档测试二进制（调度记录 records/2026-09-30-里程碑三开工第一轮.md 第 936 行那一次的形；
            # LIB_HEAVY_TESTS_BREAK=unexpanded-binary-unjudged 下前两格红）
            ("三方腿经内存包装与 capped.sh 起循环里 ls 出来的 checker 档测试二进制：照拒", "three-way-defense",
             "D=./target/release/deps; for t in crash_enumeration_new_pool_file_creation_stream checker_quick_stream; do b=$(ls -t $D/$t-* | grep -v '\\.d$' | head -1); "
             "nice -n 19 bash research/scripts/run-with-memory-cap.sh 6G bash research/scripts/capped.sh 7 $b zz_probe --nocapture; done", 2),
            ("主 agent 不带前缀起循环里 ls 出来的 checker 档测试二进制：照拒", None,
             "for t in checker_quick_stream; do b=$(ls -t ./target/release/deps/$t-* | head -1); bash research/scripts/run-with-memory-cap.sh 16G $b; done", 2),
            ("主 agent带前缀起循环里 ls 出来的 checker 档测试二进制：放行", None,
             commit + "bash research/scripts/run-with-memory-cap.sh 16G $(ls -t ./target/release/deps/checker_quick_stream-* | head -1)", 0),
            ("实现员经内存包装起循环里 ls 出来的 harness 档测试二进制：放行", writer,
             "b=$(ls -t ./target/release/deps/overwrite_in_one_instance-* | head -1); bash research/scripts/run-with-memory-cap.sh 4G $b", 0),
            ("主 agent带前缀直接起判法脚本验证明", None, commit + "python3 research/scripts/verus-proof-check.py proof .", 0),
            ("实现员直接起判法脚本核版本一致（只读安装与缓存）", writer, "python3 research/scripts/verus-proof-check.py toolchain .", 0),
            ("实现员只打 Verus 那一道的工具版本", writer, "bash .claude/gate.d/checker-tier-formal-verification.sh --tool-versions", 0),
            ("主 agent 不带前缀只打 Verus 那一道的工具版本", None, "bash .claude/gate.d/checker-tier-formal-verification.sh --tool-versions", 0),
            ("主 agent带 =commit 把广谱变异库目录换到暂存树那一版：放行", None,
             commit + "nice -n 19 python3 research/scripts/broad-mutation.py --sync-tree --commit 0123456789abcdef0123456789abcdef01234567", 0),
            ("主 agent带 =commit 起广谱变异驱动只现判改到的：放行", None,
             commit + "nice -n 19 python3 research/scripts/broad-mutation.py --changed-since HEAD --threads 8 --memory-cap 16G", 0),
            ("主 agent带 =commit 跑广谱变异的提交门禁：放行", None, commit + "nice -n 19 bash .claude/gate.d/checker-tier-broad-mutation.sh", 0),
            ("主 agent不带前缀跑广谱变异的提交门禁：照拒", None, "bash .claude/gate.d/checker-tier-broad-mutation.sh", 2),
            ("主 agent不带前缀起广谱变异驱动：照拒", None, "python3 research/scripts/broad-mutation.py --changed-since HEAD", 2),
            ("实现员带前缀跑广谱变异的提交门禁：照拒", writer, commit + "bash .claude/gate.d/checker-tier-broad-mutation.sh", 2),
            ("实现员只打广谱变异提交门禁的格名表：放行", writer, "bash .claude/gate.d/checker-tier-broad-mutation.sh --list", 0),
            ("实现员只跑广谱变异提交门禁的判定那一格：放行", writer, "bash .claude/gate.d/checker-tier-broad-mutation.sh --check gate-verdict", 0),
            ("实现员只判广谱变异驱动的忽略用例登记：放行", writer, "python3 research/scripts/broad-mutation.py --check-ignored", 0),
            ("实现员跑 gate-staged.sh", writer, "bash research/scripts/gate-staged.sh", 2),
            ("实现员喂给 bash 的 heredoc 里跑 --all", writer, "bash <<'EOF'\ncargo test --all\nEOF", 2),
            ("实现员直接执行 checker 档的测试二进制", writer, tier_binary + " --test-threads 4", 2),
            ("书记员跑 research 构建与复跑那一道", scribe, "nice -n 19 bash .claude/gate.d/checker-tier-research-build-and-replay.sh", 2),
            ("通用 agent 跑 checker-tier-layer0-replay", "general-purpose", "bash .claude/gate.d/checker-tier-layer0-replay.sh", 2),
            ("主 agent跑 checker-tier-layer0-replay 不带前缀", None, "bash .claude/gate.d/checker-tier-layer0-replay.sh", 2),
            ("主 agent不带前缀跑 gate.sh --staged", None, "bash .claude/scripts/gate.sh --staged", 2),
            ("主 agent 裸跑 gate.sh --staged", None, "bash .claude/scripts/gate.sh --staged", 2),
            ("主 agent 带 =whatever", None, "SINGLEFS_HEAVY_TESTS=whatever bash .claude/scripts/gate.sh --staged", 2),
            ("主 agent 第二条命令没带前缀", None, commit + "cargo test --all; bash .claude/scripts/gate.sh", 2),
            ("主 agent 裸跑 E152 的 run 脚本", None, "bash research/scripts/e152-run.sh /tmp/x", 2),
            ("主 agent 裸 cargo run E152", None, "cd research && cargo run --release --bin e152-file-system-benchmark", 2),
            ("主 agent 带前缀跑 --all", None, commit + "cargo test --all", 0),
            ("主 agent 带 =user-request 跑 gate.sh", None, request + "bash .claude/scripts/gate.sh --staged", 0),
            ("主 agent export 之后跑 checker-tier-layer0-replay 全量", None, "export SINGLEFS_HEAVY_TESTS=commit && bash /tmp/wt/.claude/gate.d/checker-tier-layer0-replay.sh --full /tmp/wt", 0),
            ("主 agent 前缀往包装里传", None, "nice -n 19 env SINGLEFS_HEAVY_TESTS=commit bash research/scripts/capped.sh 8 bash -c 'cargo test --workspace'", 0),
            ("主 agent 跑一个测试目标不是重型", None, "cargo test -p singlefs-core --test core_contract", 0),
            ("主 agent带前缀跑 checker-tier-layer0-replay 全量", None, commit + "bash .claude/gate.d/checker-tier-layer0-replay.sh --full /tmp/wt", 0),
            ("主 agent带前缀跑 checker 档的集成测试", None,
             commit + "nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream", 0),
            ("主 agent带前缀直接执行 checker 档的测试二进制", None, commit + "bash research/scripts/run-with-memory-cap.sh 16G " + tier_binary, 0),
            ("实现员跑登记的checker 档的用例（带 --include-ignored）", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G cargo test --release -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --include-ignored --exact the_full_case", 2),
            ("实现员跑 checker 档集成测试的快用例（不带 --ignored 也拒：按包判）", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream", 2),
            ("主 agent不带前缀跑checker 档的用例", None,
             "bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --ignored", 2),
            ("主 agent带前缀跑checker 档的用例", None,
             commit + "bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --include-ignored --exact the_full_case", 0),
            ("主 agent 不带前缀直接执行checker 档的用例的测试二进制", None, "./target/release/deps/crash_enumeration_new_pool_file_creation_stream-0123456789abcdef --ignored", 2),
            ("主 agent 带前缀直接执行checker 档的用例的测试二进制", None, commit + "./target/release/deps/crash_enumeration_new_pool_file_creation_stream-0123456789abcdef --ignored", 0),
            # 包在外面照样起那条命令的（/usr/bin/time、flock、systemd-run……）剥掉之后照判；里面经内存包装的算经包装
            ("实现员 /usr/bin/time -v 包一层跑 --all", writer, "/usr/bin/time -v cargo test --all", 2),
            ("实现员 systemd-run --scope 自设内存上限跑checker 档的用例", writer,
             "systemd-run --user --scope -q -p MemoryMax=8G cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --ignored", 2),
            ("实现员 perf stat 包一层跑 --all", writer, "perf stat -e cycles cargo test --all", 2),
            # research/scripts/detached-run.sh start … -- <命令> 看穿：里面那条照表判、前缀照认、按角色拒（LIB_HEAVY_TESTS_BREAK=detached-run-opaque 下这几格红）
            ("主 agent 经 detached-run.sh start 起广谱变异、不带前缀拒", None,
             "bash /abs/research/scripts/detached-run.sh start --label m -- nice -n 19 python3 research/scripts/broad-mutation.py --threads 31 --skip-stage-two --shard 0/2", 2),
            ("主 agent 带 =user-request 经 detached-run.sh start 起广谱变异放行", None,
             request + "bash /abs/research/scripts/detached-run.sh start --label m -- nice -n 19 python3 research/scripts/broad-mutation.py --threads 31 --skip-stage-two --shard 0/2", 0),
            ("主 agent 前缀写在 -- 之后那条命令上也认", None,
             "bash /abs/research/scripts/detached-run.sh start --label m -- SINGLEFS_HEAVY_TESTS=user-request nice -n 19 python3 research/scripts/broad-mutation.py --shard 0/2", 0),
            ("主 agent cd 进 worktree、带前缀经 detached-run.sh start 起广谱变异放行", None,
             "cd /tmp/wt && " + request + "bash /abs/research/scripts/detached-run.sh start --label m -- nice -n 19 python3 research/scripts/broad-mutation.py --shard 0/2", 0),
            ("实现员带前缀经 detached-run.sh start 跑 --all 照拒", writer, commit + "bash research/scripts/detached-run.sh start --label m -- cargo test --all", 2),
            ("实现员经 detached-run.sh start、里面经内存包装跑 checker 档包照拒", writer,
             "bash research/scripts/detached-run.sh start --label m -- bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier", 2),
            # 按内容认 detached-run.sh：改名拷贝（开头带签名行）、符号链接、同一条命令里先 cp 再起、source 读进来的（LIB_HEAVY_TESTS_BREAK=detached-run-by-name-only 下前五格红）
            ("主 agent 不带前缀经改名拷贝起全量照拒", None, "bash research/scripts/dr-renamed.sh start --label p -- cargo test --all", 2),
            ("主 agent 不带前缀经指到它的符号链接起全量照拒", None, "bash elsewhere/dr-link.sh start --label p -- cargo test --all", 2),
            ("主 agent 不带前缀同一条命令里先 cp 改名再起全量照拒", None,
             "cp research/scripts/detached-run.sh dr2.sh && bash dr2.sh start --label p -- cargo test --all", 2),
            ("实现员同一条命令里先 cp 改名再起全量照拒", writer, "cp research/scripts/detached-run.sh dr2.sh && bash dr2.sh start --label p -- cargo test --all", 2),
            ("实现员 source 读进改名拷贝起 checker 档照拒", writer,
             "source elsewhere/dr-renamed.sh start --label p -- bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier", 2),
            ("主 agent 带前缀经改名拷贝起全量放行", None, request + "bash research/scripts/dr-renamed.sh start --label p -- cargo test --all", 0),
            # 同一条命令里 heredoc 写出带签名行的拷贝再起（HEAVY_TEST_GUARD_BREAK=ignore-heredoc-detached-run 下这一格与它的「：读进去几份脚本」一格红）
            ("主 agent 不带前缀同一条命令里 heredoc 写出带签名行的拷贝再起全量照拒", None,
             "cat > dr-heredoc.sh <<'EOF'\n#!/usr/bin/env bash\n# detached-run-script-signature: 样本（heavy-test-guard 自检造的）\necho sample\nEOF\n"
             "bash dr-heredoc.sh start --label p -- cargo test --all", 2),
            # --force 照 detached-run.sh 的 preflight 摘：-- 之后先写的、写在 --label 值位置上的（LIB_SHELL_WORDS_BREAK=detached-run-force-kept 下这两格与第二格的「：读进去几份脚本」一格红；这个开关下红的全部四格逐格写在文件头）
            ("主 agent 不带前缀经 detached-run.sh start、-- 之后先写 --force 再起全量照拒", None,
             "bash research/scripts/detached-run.sh start --label p -- --force cargo test --all", 2),
            ("主 agent 不带前缀经 detached-run.sh start、--label 后面先写 --force 再起全量照拒", None,
             "bash research/scripts/detached-run.sh start --label --force p -- cargo test --all", 2),
            ("实现员 /usr/bin/time 包一层跑自己的测试目标、没经内存包装", writer, "/usr/bin/time -v cargo test -p singlefs-core --test core_contract", 2),
            ("实现员内存包装里 /usr/bin/time 包一层跑自己的测试目标", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G /usr/bin/time -v cargo test -p singlefs-core --test core_contract", 0),
            ("实现员 /usr/bin/time 包在内存包装外面跑自己的测试目标", writer,
             "/usr/bin/time -v bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract", 0),
            ("主 agent带前缀 /usr/bin/time 包一层经内存包装跑checker 档的用例", None,
             commit + "/usr/bin/time -v bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream "
             "-- --include-ignored --exact the_full_case", 0),
            ("主 agent 不带前缀 flock 包一层跑 gate.sh", None, "flock /tmp/gate.lock bash .claude/scripts/gate.sh --staged", 2),
            # 短选项合写、systemd-run -E 设的 runner、--list 是别的选项的值、--config 里带引号的 runner 键、用例函数有一处没标或判不出：
            # lib_heavy_tests.py 的 LIB_HEAVY_TESTS_BREAK 与 admission.py 的 ADMISSION_BREAK 那几个开关下这几格红（开关写在两份的文件头）
            ("实现员内存包装里 strace -fo 包一层跑checker 档的用例", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G strace -fo /tmp/s cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --ignored", 2),
            ("实现员不经内存包装 strace -fo 包一层跑checker 档的用例", writer,
             "strace -fo /tmp/s cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --ignored", 2),
            ("实现员内存包装里 flock -xw 10 包一层跑checker 档的用例", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G flock -xw 10 /tmp/l cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --ignored", 2),
            ("实现员内存包装里 -- --include-ignored --skip --list（--list 是 --skip 的值）", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --include-ignored --skip --list", 2),
            ("实现员内存包装里直接执行checker 档的用例的测试二进制 --include-ignored --skip --list", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G ./target/release/deps/crash_enumeration_new_pool_file_creation_stream-0123456789abcdef --include-ignored --skip --list", 2),
            # 门禁批第三轮的 F1、F2、F3、F10：要值的长选项值另起一个词、systemd-run -p Environment=、第二个 -- 之后的 --list、写在 test 之后的 --config
            # （lib_heavy_tests.py 的 launcher-long-options-partial、launcher-drops-property-environment、list-past-separator、
            # configuration-before-subcommand-only 各自打开时对应那几格红）
            ("实现员内存包装里 systemd-run --expand-environment no 包一层跑checker 档的用例", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G systemd-run --user --scope --expand-environment no cargo test -p singlefs-checker-tier "
             "--test crash_enumeration_new_pool_file_creation_stream -- --ignored", 2),
            ("实现员内存包装里 strace --output 文件包一层跑checker 档的用例", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G strace --output /tmp/s cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --ignored", 2),
            ("实现员内存包装里 -- --include-ignored the_full_case -- --list（第二个 -- 之后是过滤词）", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --include-ignored the_full_case -- --list", 2),
            # 起测试的外部子命令、别名与 runner、只列不跑、没标 #[ignore] 的登记用例
            ("实现员经内存包装 cargo nextest run --run-ignored all 跑checker 档的用例", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G cargo nextest run -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream --run-ignored all", 2),
            ("实现员 cargo mutants 整个 checker 档", writer, "cargo mutants -p singlefs-checker-tier", 2),
            ("实现员 cargo mutants 整个 harness 档（随时跑）", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo mutants -p singlefs-harness", 0),
            ("实现员经内存包装、--config 定别名再用别名", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G cargo --config 'alias.xt=\"test\"' xt -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream", 2),
            ("实现员经内存包装列checker 档的用例（--ignored --list）", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- --ignored --list", 0),
            ("主 agent带 =user-request 跑 QEMU 那一道", None, request + "bash .claude/gate.d/checker-tier-qemu-device-streams.sh", 0),
            ("主 agent带前缀跑 crates 变异表那一道", None, "GATE_MUTATION_TARGET_DIR=/tmp/t " + commit + "nice -n 19 bash .claude/gate.d/checker-tier-crates-mutation-replay.sh", 0),
            ("主 agent带前缀跑 gate.sh --staged", None, commit + "nice -n 19 bash .claude/scripts/gate.sh --staged", 0),
            ("主 agent带前缀跑 research 构建与复跑那一道", None, commit + "bash .claude/gate.d/checker-tier-research-build-and-replay.sh", 0),
            # 提交时才跑的检查：同一套放行（lib_heavy_tests.py 的 commit-checks-ignored、这里的 HEAVY_TEST_GUARD_BREAK=ignore-commit-checks 下该拒的这几格红）
            ("执行员跑 doc-text（不重型的阶段：提交时才跑的检查）", "experiment-runner", "nice -n 19 bash .claude/gate.d/doc-text.sh", 2),
            ("书记员跑 doc-decisions（提交时才跑的检查）", scribe, "nice -n 19 bash .claude/gate.d/doc-decisions.sh", 2),
            ("通用 agent 跑 code-tooling.sh 的 agent-write-scope 格（提交时才跑的检查）", "general-purpose", "bash .claude/gate.d/code-tooling.sh --check agent-write-scope", 2),
            ("主 agent 不带前缀跑 code-tooling.sh（提交时才跑的检查）", None, "bash .claude/gate.d/code-tooling.sh", 2),
            ("主 agent 带前缀跑 code-tooling.sh", None, commit + "bash .claude/gate.d/code-tooling.sh", 0),
            ("主 agent带前缀跑不重型的阶段", None, commit + "bash .claude/gate.d/code-tooling.sh . --check research-gate-lint", 0),
            ("主 agent不带前缀跑 gate-lint.sh", None, "GATE_LINT_DIR=research/scripts bash .claude/singlefs-ai-sop/scripts/gate-lint.sh", 2),
            ("主 agent带前缀跑 doc-lint.sh", None, commit + "bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .", 0),
            ("工具实现员单独跑钩子的自证（登记成不起重活，放行）", "tooling-writer", f"bash {repo}/.claude/hooks/heavy-test-guard.sh --selftest", 0),
            ("工具实现员带前缀跑 rules-lint.sh 照拒", "tooling-writer", commit + "bash .claude/singlefs-ai-sop/scripts/rules-lint.sh .", 2),
            ("工具实现员拿样本跑 stage-selftest.sh、阶段目录此刻不在：判不出喂的是哪几道，照拒", "tooling-writer",
             f"nice -n 19 bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh /tmp/d/.claude/gate.d", 2),
            # 门禁样本（lib_heavy_tests.py 文件头「门禁样本」；样本根与阶段目录由 build_sample_workspace 造，登记行读仓里那份表）：
            # 放行的三格在 LIB_HEAVY_TESTS_BREAK=stage-samples-ignored 下红，拒的几格各对应 stage-samples-root-unchecked、stage-samples-heavy-as-light
            ("工具实现员 stage-selftest.sh 喂样本档全登记成轻的阶段目录：放行、不读它的正文", "tooling-writer",
             f"nice -n 19 bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-light/.claude/gate.d", 0, 0, 0),
            ("工具实现员 stage-selftest.sh 喂有起重活样本的阶段目录（crates 变异表）：照拒", "tooling-writer",
             f"nice -n 19 bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-heavy/.claude/gate.d", 2),
            # 不带 --item 也只信本仓那一份 stage-selftest.sh（test-script-align-r2 攻方 U1-2；LIB_HEAVY_TESTS_BREAK=stage-feed-script-by-name 下这一格红）
            ("实现员拿别处树里改过的 stage-selftest.sh 拷贝、不带 --item 喂全登记成轻的阶段目录：照拒", writer,
             f"bash {work}/evil-tree/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-light/.claude/gate.d", 2),
            # --item 只判点名的几道（lib_heavy_tests.py 文件头「门禁样本」①）：放行的四格在 LIB_HEAVY_TESTS_BREAK=stage-items-as-directory 下红（$PWD 那一格另在
            # stage-directory-pwd-unexpanded 下红），拒的一格在 stage-items-unjudged 下红
            ("工具实现员 stage-selftest.sh --item 点名一道轻的（目录里另有起重活的那一道）：放行、不读正文", "tooling-writer",
             f"nice -n 19 bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-heavy/.claude/gate.d --item doc-text.sh", 0, 0, 0),
            ("工具实现员 cd 进阶段目录的上一层、阶段目录写 \"$PWD/.claude/gate.d\"、--item 点名一道轻的：放行", "tooling-writer",
             f"cd {work}/gd-heavy && bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh \"$PWD/.claude/gate.d\" --item doc-text.sh", 0),
            ("实现员 stage-selftest.sh --item 点名里夹一道起重活的（crates 变异表）：照拒", writer,
             f"bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-heavy/.claude/gate.d --item doc-text.sh --item checker-tier-crates-mutation-replay.sh", 2),
            ("工具实现员 stage-selftest.sh --item 点名不存在的阶段（它自己退 2、不喂样本）：放行", "tooling-writer",
             f"bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-heavy/.claude/gate.d --item no-such-stage.sh", 0),
            ("实现员 stage-selftest.sh --list-items：放行", writer, f"bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-heavy/.claude/gate.d --list-items", 0),
            ("工具实现员把 checker-tier-layer0-replay 指到放了 .layer0-sample-tools 的样本根（2026-10-01 被拒的那一种）：放行", "tooling-writer",
             f"cd {work}/sample-layer0 && nice -n 19 bash {work}/.claude/gate.d/checker-tier-layer0-replay.sh {work}/sample-layer0 --force", 0),
            ("主 agent 不带前缀把 checker-tier-layer0-replay 指到样本根：放行", None, f"cd {work}/sample-layer0 && bash {work}/.claude/gate.d/checker-tier-layer0-replay.sh {work}/sample-layer0", 0),
            ("工具实现员 cd 进样本根、根参数指到真仓跑 checker-tier-layer0-replay：照拒", "tooling-writer", f"cd {work}/sample-layer0 && bash {work}/.claude/gate.d/checker-tier-layer0-replay.sh {work}", 2),
            ("工具实现员把 crates 变异表那一道指到样本根（样本档起真 cargo，登记成起重活）：照拒", "tooling-writer",
             f"cd {work}/sample-crates && bash {work}/.claude/gate.d/checker-tier-crates-mutation-replay.sh {work}/sample-crates --force", 2),
            # 标记里装的是不是仓里样本放的那种（攻方 2026-10-01 的造法：一棵树里只放 .layer0-sample-tools/quick-tier；LIB_HEAVY_TESTS_BREAK=stage-samples-marker-unchecked 下这四格红）
            ("实现员在一棵树里只放 .layer0-sample-tools/quick-tier 再起 checker-tier-layer0-replay 快档：照拒", writer,
             f"cd {work}/sample-layer0-quick-tier-only && bash {work}/.claude/gate.d/checker-tier-layer0-replay.sh {work}/sample-layer0-quick-tier-only", 2),
            ("实现员在同一棵树里起 checker-tier-layer0-replay --full：照拒", writer,
             f"cd {work}/sample-layer0-quick-tier-only && bash {work}/.claude/gate.d/checker-tier-layer0-replay.sh --full {work}/sample-layer0-quick-tier-only", 2),
            ("攻方腿在同一棵树里起 checker-tier-layer0-replay：照拒", "three-way-attack",
             f"cd {work}/sample-layer0-quick-tier-only && bash {work}/.claude/gate.d/checker-tier-layer0-replay.sh {work}/sample-layer0-quick-tier-only", 2),
            ("工具实现员把 checker-tier-layer0-replay 指到假 cargo 不可执行的样本根：照拒", "tooling-writer", f"cd {work}/sample-layer0-noexec && bash {work}/.claude/gate.d/checker-tier-layer0-replay.sh .", 2),
            # 喂的样本与暂存区比（攻方 2026-10-01 的造法：样本的 setup.sh 起 checker 档；LIB_HEAVY_TESTS_BREAK=stage-samples-fixtures-unchecked 下这一格红）
            ("实现员 stage-selftest.sh 喂样本里多一份起 checker 档的 setup.sh 的阶段目录：照拒", writer,
             f"bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-sample-setup/.claude/gate.d", 2),
            # 登记表的键按真实路径认（攻方 2026-10-01 的造法；LIB_HEAVY_TESTS_BREAK=selftest-registry-by-path-tail 下第一格红）
            ("实现员跑别处树里路径尾部相同的 claim-experiment.sh --selftest（正文起 checker 档）：照拒", writer,
             f"bash {work}/evil-tree/research/scripts/claim-experiment.sh --selftest", 2),
            ("实现员跑本仓那一份 claim-experiment.sh --selftest：放行", writer, f"bash {repo}/research/scripts/claim-experiment.sh --selftest", 0),
            # case 的模式行不当起门禁共用库（攻方扫描脚本那一行；LIB_HEAVY_TESTS_BREAK=selftest-pattern-words-judged 下这一格红）
            ("工具实现员起带 case 模式行 .claude/gate.d/lib/*.py) 的脚本：放行", "tooling-writer", "bash case-pattern.sh", 0, 0, 1),
            ("实现员单独跑研究脚本的自证（登记成不起重活，放行）", writer, f"python3 {repo}/research/scripts/admission.py --selftest", 0),
            ("实现员跑 gate-structure-check.py", writer, "python3 research/scripts/gate-structure-check.py", 2),
            ("实现员直接执行门禁共用库（跑它的自证，登记成不起重活，放行、不读正文）", writer, f"bash {repo}/.claude/gate.d/lib/stage-cells.sh", 0, 0, 0),
            # 自证照登记表 .claude/hooks/selftest-registry.tsv 判（读仓里那份表）：登记成轻的单独跑放行、不读正文（HEAVY_TEST_GUARD_BREAK=read-light-selftest-body
            # 下红的几格写在文件头），表外的、带别的参数的、设了条件变量的照拒，主 agent 带前缀放行；登记成重的拿造出来的登记表判（heavy_only_cases）
            ("工具实现员单独跑 crash-amplification.sh --selftest（假分项驱动，登记成不起重活；2026-10-01 被拒的那一条）", "tooling-writer",
             f"bash {repo}/research/scripts/crash-amplification.sh --selftest", 0, 0, 0),
            # 自证换成假驱动、登记成不起重活的两份（LIB_HEAVY_TESTS_BREAK=selftest-registry-ignored 下这两格红，commit-checks-ignored 下第二格红）
            ("实现员单独跑 compile-then-swap.py --selftest（登记成不起重活：每项一份临时仓、PATH 最前面放假 cargo），放行", writer,
             f"python3 {repo}/research/scripts/compile-then-swap.py --selftest", 0),
            ("实现员单独跑 crash-amplification-two-hosts.sh --selftest（登记成不起重活：本机、第二台、清场与用例全是假的），放行、不读正文", writer,
             f"bash {repo}/research/scripts/crash-amplification-two-hosts.sh --selftest", 0, 0, 0),
            ("实现员跑登记表外的新脚本 --selftest，照拒", writer, "python3 research/scripts/brand-new-tool.py --selftest", 2),
            ("主 agent带前缀跑登记表外的新脚本 --selftest（提交时才跑的检查），放行", None, commit + "python3 research/scripts/brand-new-tool.py --selftest", 0),
            ("实现员 mutate.sh 带表再带 --selftest（只认单独一个 --selftest；这样跑的是 crates 整表），照拒", writer,
             "cd research && bash scripts/mutate.sh x y ../crates/mutations.tsv --selftest", 2),
            ("实现员单独跑 layer0-shard-run.sh --selftest（没设 SINGLEFS_HEAVY_TESTS，假 cargo），放行", writer, f"bash {repo}/research/scripts/layer0-shard-run.sh --selftest", 0),
            ("实现员带前缀跑 layer0-shard-run.sh --selftest（设了 SINGLEFS_HEAVY_TESTS 换真 cargo 跑 checker 档包），照拒", writer,
             commit + f"bash {repo}/research/scripts/layer0-shard-run.sh --selftest", 2),
            ("工具实现员跑 vm-bench.sh --selftest（起虚机），照拒", "tooling-writer", "bash research/scripts/vm-bench.sh --selftest", 2),
            ("实现员只跑 research 单测那一格（不重型，照样是提交时才跑的检查）", writer, "bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests", 2),
            ("主 agent带前缀只跑 research 单测那一格", None, commit + "bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests", 0),
            ("主 agent 带前缀跑研究脚本的自证", None, commit + "python3 research/scripts/admission.py --selftest", 0),
            ("主 agent 不带前缀跑 hooks-registered.sh", None, "bash .claude/singlefs-ai-sop/scripts/hooks-registered.sh .", 2),
            # 只查清单、只生成、只读、只查语法的放行（lib_shell_words.py 的 LIB_SHELL_WORDS_BREAK=arguments-as-executed 下最后两格红；
            # 这个开关下红的全部四格（另两格是「实现员 bash -n 只查语法」「lib_heavy_tests.py 的自检」）逐格写在文件头）
            ("实现员 gate-overlap.py --list 查清单", writer, "python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list", 0),
            ("实现员看门禁阶段的格名表", writer, "bash .claude/gate.d/code-tooling.sh --list", 0),
            ("实现员列重型那一道可点名的变异", writer, "bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --list-items", 0),
            ("书记员跑阶段的 --write（生成）", scribe, "bash .claude/gate.d/doc-decisions.sh --write", 0),
            ("实现员 grep 门禁文件", writer, "grep -n gate-cell .claude/gate.d/checker-tier-lkmm.sh", 0),
            ("工具实现员 bash -n 查门禁的语法", "tooling-writer", "bash -n .claude/gate.d/checker-tier-crates-mutation-replay.sh", 0),
            ("工具实现员在门禁目录里 python3 - 读几份门禁（2026-09-28 误拦的那一条原样，目录换成自检的临时工作区）", "tooling-writer",
             f"cd {work}/.claude/gate.d && python3 - doc-text.sh doc-process-records.sh doc-experiments.sh <<'EOF'\nimport re, sys\n"
             "for path in sys.argv[1:]:\n    text = open(path, encoding=\"utf-8\").read()\nEOF", 0),
            # 双机分片的驱动按树根里键那一行判（lib_heavy_tests.py 的 shard-driver-by-name 打开时第一格红）
            ("实现员起双机分片的驱动、树根里登记的是 harness 档包的替身用例（驱动自检那样）", writer, "bash research/scripts/layer0-shard-run.sh crash-case:sharded-selftest .", 0),
            ("实现员起双机分片的驱动、树根里登记的是 checker 档包的分片用例", writer, "bash research/scripts/layer0-shard-run.sh crash-case:sharded-checker .", 2),
            ("实现员跑自己动到的测试目标", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test overwrite_in_one_instance", 0),
            ("实现员 clippy --all-targets", writer, "cargo clippy --all-targets -- -D warnings", 0),
            ("实现员 build --all-targets", writer, "cargo build --offline --all-targets", 0),
            ("实现员 grep 层 0 的名字", writer, "grep -n layer0 x", 0),
            ("实现员读 checker-tier-layer0-replay 脚本", writer, "cat .claude/gate.d/checker-tier-layer0-replay.sh", 0),
            ("实现员 echo 引号里的命令", writer, 'echo "cargo test --all"', 0),
            ("实现员写进笔记的 heredoc 正文", writer, "cat > note.md <<'EOF'\ncargo test --all\nEOF", 0),
            ("实现员 bash -n 只查语法", writer, "bash -n .claude/scripts/check.sh", 0),
            ("实现员 -p harness --lib", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib", 0),
            ("实现员 mutate.sh 跑 research 的表", writer, "cd research && bash scripts/mutate.sh e1 e7-index-bench/src/bin/e1.rs mutations/e1.tsv", 0),
            ("实现员单独跑 capped.sh --selftest（登记成不起重活，放行）", writer, f"bash {repo}/research/scripts/capped.sh --selftest", 0),
            ("实现员 cd 进没有层 0 的包裸跑", writer, "cd crates/singlefs-core && bash ../../research/scripts/run-with-memory-cap.sh 4G cargo test", 0),
            ("实现员 git diff 点名重型脚本", writer, "git diff -- crates/mutations.tsv .claude/gate.d/checker-tier-layer0-replay.sh", 0),
            ("执行员在 research 里跑自己的 bin", "experiment-runner",
             "cd research && bash scripts/run-with-memory-cap.sh 16G cargo test --release --bin e160-random-small-read-share", 0),
            # 跑编译出来的代码经没经内存包装（与重型不重型无关）：子 agent 不经它的拒，经它包着的（连同 bash -c、它起的脚本）放行，主 agent 不判
            ("实现员不经内存包装跑自己的测试目标", writer, "cargo test -p singlefs-harness --test overwrite_in_one_instance", 2),
            ("执行员不经内存包装 cargo run 实验二进制", "experiment-runner", "cd research && cargo run --release --bin e160-random-small-read-share", 2),
            ("实现员 cargo t 简写不经内存包装", writer, "cargo t -p singlefs-core --lib", 2),
            ("执行员直接执行实验二进制不经内存包装", "experiment-runner", "./research/target/release/e142_region_diff_independent", 2),
            ("实现员经 capped.sh 限了线程、没限内存", writer, "bash research/scripts/capped.sh 4 cargo test -p singlefs-core --lib", 2),
            ("实现员包装外面的命令替换不算在包装里", writer, "bash research/scripts/run-with-memory-cap.sh 4G echo \"$(cargo test -p singlefs-core --lib)\"", 2),
            ("实现员命令替换里不经内存包装跑", writer, "out=\"$(cargo test -p singlefs-core --lib 2>&1)\"", 2),
            ("实现员经内存包装再经 capped.sh", writer, "bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh 4 cargo test -p singlefs-core --lib", 0),
            ("实现员 capped.sh 在外、内存包装在里", writer, "capped.sh 4 run-with-memory-cap.sh 4G cargo test -p singlefs-core --lib", 0),
            ("实现员 timeout 套在内存包装外面", writer, "timeout 900 bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --lib", 0),
            ("实现员内存包装里 bash -c 跑", writer, "bash research/scripts/run-with-memory-cap.sh 4G bash -c 'cd crates/singlefs-core && cargo test --lib'", 0),
            ("实现员经内存包装直接执行测试二进制", writer, "bash research/scripts/run-with-memory-cap.sh 4G ./target/debug/deps/core_contract-0123456789abcdef", 0),
            ("实现员只编不跑 cargo test --no-run", writer, "cargo test --no-run -p singlefs-core", 0),
            ("实现员 cargo build 不用经内存包装", writer, "cargo build --release -p singlefs-core", 0),
            ("实现员起的脚本里有不经内存包装的 cargo test", writer, "bash unwrapped.sh", 2, 0, 1),
            ("实现员经内存包装起那份脚本", writer, "bash research/scripts/run-with-memory-cap.sh 4G bash unwrapped.sh", 0, 0, 1),
            ("实现员 heredoc 写出不经内存包装的 cargo run 再起它", writer,
             "cat > gen-unwrapped.sh <<'EOF'\ncargo run --release --bin e160-random-small-read-share\nEOF\nbash gen-unwrapped.sh", 2, 0, 1),
            ("主 agent 不经内存包装跑测试目标：这一道不判主 agent", None, "cargo test -p singlefs-core --lib", 0),
            ("主 agent带前缀经内存包装跑 QEMU 那一道", None, commit + "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/checker-tier-qemu-device-streams.sh", 0),
            ("主 agent带前缀经内存包装跑 herd7 那一道", None, commit + "bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/checker-tier-lkmm.sh", 0),
            ("主 agent带前缀经内存包装跑 gate.sh --staged", None, commit + "bash research/scripts/run-with-memory-cap.sh 24G bash .claude/scripts/gate.sh --staged", 0),
            ("主 agent 带前缀经内存包装跑 checker-tier-layer0-replay 全量", None,
             commit + "bash research/scripts/run-with-memory-cap.sh 16G bash /tmp/wt/.claude/gate.d/checker-tier-layer0-replay.sh --full /tmp/wt", 0),
            ("实现员经内存包装跑 checker-tier-layer0-replay 照拒（重型那一道）", writer, "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/checker-tier-layer0-replay.sh", 2),
            ("实现员单独跑 gate-staged.sh --selftest（登记成不起重活：假 gate.sh；仓内那份按名字判，不读正文），放行", writer, f"bash {repo}/research/scripts/gate-staged.sh --selftest", 0),
            # 写进脚本文件再执行：s.sh 第 4 行是 cargo test --all（上面第一例，实现员会被拒的那一条）。末两列是该记几条检出、该读进去几份脚本
            ("实现员 bash 起含它的脚本", writer, "bash s.sh", 2, 0, 1),
            ("实现员 ./ 直接执行含它的脚本", writer, "./s.sh", 2, 0, 1),
            ("实现员经 capped.sh 4 bash 起含它的脚本", writer, "capped.sh 4 bash s.sh", 2, 0, 1),
            ("实现员 source 含它的脚本", writer, "source s.sh", 2, 0, 1),
            ("实现员 . 读进含它的脚本、nice 与 timeout 包一层", writer, "nice -n 19 timeout 600 . s.sh", 2, 0, 1),
            ("实现员 cd 之后按绝对路径起含它的脚本、输出落文件", writer, f"cd {work} && bash {work}/s.sh > {work}/chain.log 2>&1", 2, 0, 1),
            ("实现员起的脚本里再调含它的脚本", writer, "bash outer.sh", 2, 0, 2),
            ("实现员起仓外同名的 mutate.sh（不按名字判，读正文）", writer, "bash elsewhere/mutate.sh x y mutations/e1.tsv", 2, 0, 1),
            ("实现员起的脚本那一行自己带前缀也拒", writer, "bash own-prefix.sh", 2, 0, 1),
            ("主 agent 带放行前缀执行含它的脚本", None, commit + "bash s.sh", 0, 0, 1),
            ("主 agent export 之后执行含它的脚本", None, "export SINGLEFS_HEAVY_TESTS=commit; ./s.sh", 0, 0, 1),
            ("主 agent 不带前缀执行含它的脚本", None, "bash s.sh", 2, 0, 1),
            ("主 agent 执行的脚本那一行自己带前缀", None, "bash own-prefix.sh", 0, 0, 1),
            ("实现员起不含重型命令的脚本", writer, "bash light.sh", 0, 0, 1),
            ("实现员起调回自己的脚本（只读一遍）", writer, "bash loop.sh", 0, 0, 1),
            ("实现员 capped.sh 后面不带斜杠的命令词从 PATH 找，不读当前目录的同名脚本", writer, "bash research/scripts/capped.sh 4 s.sh", 0, 0, 0),
            ("实现员 ./ 执行 python 程序不读（射程之外）", writer, "./tool", 0, 0, 0),
            ("实现员 ./ 执行编译出来的程序不读", writer, "./prog", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的编译产物：不读也不记", writer, "./bigprog", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的 python 程序：不读也不记", writer, "./bigtool", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的 shell 脚本：放行并记检出", writer, "./big.sh", 0, 1, 0),
            ("实现员起 doc-lint.sh 第 43 行那种命令替换：片段不当路径", writer, "bash doc-lint-line-43.sh", 0, 0, 1),
            ("实现员起 doc-lint.sh 第 70 行那种命令替换：片段不当路径", writer, "bash doc-lint-line-70.sh", 0, 0, 1),
            ("实现员起的脚本里命令替换、反引号、算术后面接路径：片段不当路径", writer, "bash substitution-fragments.sh", 0, 0, 1),
            ("实现员起的脚本里命令替换里跑 --all", writer, "bash substitution-heavy.sh", 2, 0, 1),
            ("实现员起的脚本里 command -v herd7：查名字不是跑", writer, "bash command-lookup.sh", 0, 0, 1),
            ("实现员起的脚本里数组赋值的元素：不当命令、不当路径", writer, "bash array-assignment.sh", 0, 0, 1),
            ("实现员直接跑 command -v herd7", writer, "command -v herd7 >/dev/null && echo 在", 0),
            ("实现员 command herd7（不带 -v）照拒", writer, "command herd7 -version", 2),
            ("实现员 bash 起一个目录：不读也不记", writer, "bash crates", 0),
            # 进程替换 `<(…)`：括号里的命令另按一条完整的命令判，`)` 之后的词是外层命令的参数（第一例是现场误拒的原句）
            ("实现员 diff <(git show …check.sh) check.sh：) 之后的 check.sh 是 diff 的参数，放行、不记检出", writer,
             "diff <(git show HEAD:.claude/scripts/check.sh) .claude/scripts/check.sh", 0, 0, 0),
            ("实现员 diff <(bash check.sh) x：括号里真执行全量测试，照拒", writer, "diff <(bash .claude/scripts/check.sh) x", 2),
            # 喂给 python 的 heredoc 正文不扫：那一行别处出现 x.sh、前一条是 bash，都不算喂给 shell；bash light.sh 那一条证明这一例真的判过
            ("实现员同一行里起 .sh 之后把 python 源码喂给 python3 -：正文不当命令", writer,
             "sed -n 1p light.sh > body.py && bash light.sh && python3 - <<'PY'\nsource = open(\"body.py\", encoding=\"utf-8\").read()\n"
             "body = open(\"/tmp/claude-1000/nowhere/hook_body.py\", encoding=\"utf-8\")\nusage = \"\"\"\ncargo test --all\n\"\"\"\nPY", 0, 0, 1),
            # 喂给 bash 的 heredoc 里出现 python 那种写法：`source =` 不算读脚本，带逗号的词不当路径
            ("实现员喂给 bash 的正文里有 source = 与带逗号的路径：不当脚本", writer,
             "bash light.sh; bash <<'EOF'\nsource = open(\"body.py\", encoding=\"utf-8\").read()\n"
             "body = open(\"/tmp/claude-1000/nowhere/hook_body.py\", encoding=\"utf-8\")\nEOF", 0, 0, 1),
            ("实现员 bash 起通配写出来的脚本路径：不当路径、不记检出", writer, "bash ./chain-*.sh", 0, 0, 0),
            # 命令词带通配的照 bash 展开、按展开的那几份判（test-script-align-r2 攻方 U1-1；LIB_HEAVY_TESTS_BREAK=pattern-command-words-literal 下前五格红，
            # pattern-expansions-bare 下「写成通配的 capped.sh」那一格红）
            ("实现员通配起 checker-tier-layer0-replay （绝对路径）：照拒", writer, f"bash {repo}/.claude/gate.d/checker-tier-layer0-replay.s? {repo}", 2),
            ("实现员 cd 进仓根通配起 QEMU 那一道：照拒", writer, f"cd {repo} && bash .claude/gate.d/checker-tier-qemu-device-streams.s? {repo}", 2),
            ("实现员通配起 mutate.sh 跑 crates 整表：照拒", writer, f"cd {repo} && bash research/scripts/mutate.s? crates/mutations.tsv", 2),
            ("实现员通配起 vm-bench.sh --selftest：照拒", writer, f"cd {repo} && bash research/scripts/vm-bench*.sh --selftest", 2),
            ("实现员写成通配的 capped.sh 包全量：照拒", writer, f"cd {repo} && bash research/scripts/capped.s? 4 cargo test --all", 2),
            ("主 agent 带前缀通配起 checker-tier-layer0-replay：放行", None, commit + f"bash {repo}/.claude/gate.d/checker-tier-layer0-replay.s? {repo}", 0),
            ("实现员合法的通配 ls research/scripts/*.sh：放行", writer, f"cd {repo} && ls research/scripts/*.sh", 0, 0, 0),
            ("实现员通配起登记成轻的自证：放行", writer, f"cd {repo} && python3 research/scripts/admission*.py --selftest", 0, 0, 0),
            # 方括号里的 POSIX 字符类、等价类、整理符号交给 bash 展开（test-script-align-r3 攻方 A1；LIB_HEAVY_TESTS_BREAK=posix-brackets-by-python-glob 下前四格红，
            # posix-brackets-unresolved-literal 下第四格红）
            ("实现员 POSIX 字符类起 mutate.sh 跑 crates 整表：照拒", writer, f"cd {repo} && bash research/scripts/mutat[[:alpha:]].sh crates/mutations.tsv", 2),
            ("实现员等价类起 checker-tier-layer0-replay：照拒", writer, f"bash {repo}/.claude/gate.d/checker-tier-layer0-replay.s[[=h=]] {repo}", 2),
            ("实现员整理符号起 QEMU 那一道：照拒", writer, f"cd {repo} && bash .claude/gate.d/checker-tier-qemu-device-streams.s[[.h.]] {repo}", 2),
            ("实现员 cd 到变量路径之后 POSIX 字符类起脚本：判不出，照拒", writer, 'cd "$X" && bash research/scripts/mutat[[:alpha:]].sh crates/mutations.tsv', 2),
            ("实现员合法的 POSIX 字符类 ls research/scripts/[[:alpha:]]*.sh：放行", writer, f"cd {repo} && ls research/scripts/[[:alpha:]]*.sh", 0, 0, 0),
            # 崩溃放量的三份入口按名字算重型、不读正文（test-script-align-r3 攻方 A2；LIB_HEAVY_TESTS_BREAK=crash-amplification-unnamed 下前四格红）
            ("实现员起崩溃放量全量：照拒", writer, f"bash {repo}/research/scripts/crash-amplification.sh", 2),
            ("实现员起崩溃放量的分项驱动：照拒", writer, f"bash {repo}/research/scripts/crash-amplification-item.sh crash_amplification_fixed_script_through_rollback", 2),
            ("主 agent 不带前缀起崩溃放量全量：照拒", None, f"bash {repo}/research/scripts/crash-amplification.sh", 2),
            ("主 agent 带前缀起崩溃放量全量：放行", None, commit + f"bash {repo}/research/scripts/crash-amplification.sh", 0),
            ("实现员列崩溃放量的分项：放行、不读正文", writer, f"bash {repo}/research/scripts/crash-amplification.sh --list-items", 0, 0, 0),
            ("实现员单独跑分项驱动的自证（登记成不起重活）：放行、不读正文", writer, f"bash {repo}/research/scripts/crash-amplification-item.sh --selftest", 0, 0, 0),
            # 两档里的轻型（lib_heavy_tests.py 文件头「两档里的轻型」）：三条带 --quick 的子 agent 带不带前缀都放行、要经内存包装；重型的剪枝、逐条、全量照拒
            # （LIB_HEAVY_TESTS_BREAK=light-tier-unrecognized 下前四格与「主 agent 不带前缀起崩溃点重放 --quick」红，light-tier-memory-cap-unrequired 下「不经内存包装」一格红，
            # crash-point-replay-unnamed 下「剪枝」到「主 agent 不带前缀起崩溃点重放入口」五格红，light-tier-any-arguments 下「--quick 又带 --full」一格红，
            # gpu-quiesce-opaque 下第二格的「：读进去几份脚本」与 gpu-quiesce-run.sh 起重型的两格红）
            ("实现员经内存包装起崩溃点重放入口 --quick：放行、不读正文", writer,
             f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/research/scripts/crash-point-replay.sh --quick", 0, 0, 0),
            ("实现员经内存包装与 gpu-quiesce-run.sh 起崩溃点重放 --quick 加点名一项：放行", writer,
             f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/research/scripts/gpu-quiesce-run.sh -- "
             f"bash {repo}/research/scripts/crash-point-replay.sh --quick --item fixed-script", 0, 0, 0),
            ("调查员经内存包装起形式化那一道 --quick：放行", "investigator",
             f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/.claude/gate.d/checker-tier-formal-verification.sh --quick {repo}", 0, 0, 0),
            ("实现员带前缀经内存包装起 crates 变异那一道 --quick --item：放行", writer,
             commit + f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/.claude/gate.d/checker-tier-crates-mutation-replay.sh --quick {repo} --item core-row",
             0, 0, 0),
            ("主 agent 不带前缀起崩溃点重放 --quick：放行", None, f"bash {repo}/research/scripts/crash-point-replay.sh --quick", 0),
            ("实现员不经内存包装起崩溃点重放 --quick：拒的是包装", writer, f"bash {repo}/research/scripts/crash-point-replay.sh --quick", 2),
            ("实现员起崩溃点重放入口不带参数（剪枝）：照拒", writer,
             f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/research/scripts/crash-point-replay.sh", 2),
            ("实现员起崩溃点重放 --item 不带 --quick（逐条）：照拒", writer,
             f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/research/scripts/crash-point-replay.sh --item fixed-script", 2),
            ("实现员带前缀起崩溃点重放 --full（全量）：照拒", writer,
             commit + f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/research/scripts/crash-point-replay.sh --full", 2),
            ("实现员经 gpu-quiesce-run.sh 起崩溃点重放不带参数：照拒", writer,
             f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/research/scripts/gpu-quiesce-run.sh -- bash {repo}/research/scripts/crash-point-replay.sh", 2),
            ("主 agent 不带前缀起崩溃点重放入口：照拒", None, f"bash {repo}/research/scripts/crash-point-replay.sh", 2),
            ("主 agent 带前缀起崩溃点重放 --full：放行", None, commit + f"bash {repo}/research/scripts/crash-point-replay.sh --full", 0),
            ("实现员起形式化那一道 --item（逐条）：照拒", writer,
             f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/.claude/gate.d/checker-tier-formal-verification.sh --item proof {repo}", 2),
            ("实现员起 crates 变异那一道 --quick 又带 --full：照拒", writer,
             f"bash {repo}/research/scripts/run-with-memory-cap.sh 4G bash {repo}/.claude/gate.d/checker-tier-crates-mutation-replay.sh --quick --full {repo}", 2),
            ("实现员经 gpu-quiesce-run.sh 起崩溃放量全量：照拒", writer,
             f"bash {repo}/research/scripts/gpu-quiesce-run.sh -- bash {repo}/research/scripts/crash-amplification.sh", 2),
            # 看门狗记下的那一次（records/2026-09-30-里程碑三开工第一轮.md 第 949 行；LIB_HEAVY_TESTS_BREAK=verus-read-only-needs-literal-arguments 下这一格红）
            ("工具实现员判法脚本 list-items $PWD（看门狗记下的那一次）：放行", "tooling-writer", "python3 research/scripts/verus-proof-check.py list-items $PWD", 0),
            # 真编 QEMU 按名字算重型、不读正文（LIB_HEAVY_TESTS_BREAK=qemu-build-unnamed 下前四格红；commit-checks-ignored、selftest-registry-ignored 下末格红）
            ("实现员真编 QEMU（--rebuild）：照拒", writer, f"bash {repo}/research/scripts/build-patched-qemu.sh --rebuild", 2),
            ("实现员经内存包装真编 QEMU（不带参数）：照拒", writer, f"bash {repo}/research/scripts/run-with-memory-cap.sh 8G bash {repo}/research/scripts/build-patched-qemu.sh", 2),
            ("主 agent 不带前缀经内存包装真编 QEMU：照拒", None, f"bash {repo}/research/scripts/run-with-memory-cap.sh 8G bash {repo}/research/scripts/build-patched-qemu.sh --rebuild", 2),
            ("主 agent 带前缀真编 QEMU：放行", None, commit + f"bash {repo}/research/scripts/build-patched-qemu.sh --rebuild", 0),
            ("实现员单独跑 build-patched-qemu.sh 的自证（登记成不起重活）：放行、不读正文", writer, f"bash {repo}/research/scripts/build-patched-qemu.sh --selftest", 0, 0, 0),
            # 主 agent 跑崩溃放量：直接起与经 detached-run.sh start … -- 起（前缀写在外面、写在 -- 之后）一样判，不带前缀拒
            ("主 agent带 =user-request 起 worktree 里的崩溃放量全量：放行", None, request + "bash /tmp/wt-crash/research/scripts/crash-amplification.sh", 0),
            ("主 agent经 detached-run.sh start、前缀写在外面起崩溃放量点名分项：放行", None, request + f"bash {repo}/research/scripts/detached-run.sh start "
             "--label crash-amplification-full -- bash /tmp/wt-crash/research/scripts/crash-amplification.sh --item fixed-script", 0),
            ("主 agent经 detached-run.sh start、前缀写在 -- 之后起崩溃放量全量：放行", None, f"bash {repo}/research/scripts/detached-run.sh start "
             "--label crash-amplification-full -- " + request + "bash /tmp/wt-crash/research/scripts/crash-amplification.sh", 0),
            ("主 agent不带前缀起崩溃放量全量：照拒", None, "bash /tmp/wt-crash/research/scripts/crash-amplification.sh", 2),
            ("主 agent不带前缀经 detached-run.sh start 起崩溃放量全量：照拒", None, f"bash {repo}/research/scripts/detached-run.sh start "
             "--label crash-amplification-full -- bash /tmp/wt-crash/research/scripts/crash-amplification.sh", 2),
            ("实现员带 =user-request 起崩溃放量全量：照拒", writer, request + "bash /tmp/wt-crash/research/scripts/crash-amplification.sh", 2),
            # 花括号与参数里的通配照 bash 展开再判（2026-10-01 主 agent 喂本仓钩子：前三格那样写都放行；LIB_HEAVY_TESTS_BREAK=brace-expansion-literal 下
            # 前两格与「花括号写在包装上」红，argument-patterns-literal 下第三格红，brace-command-words-bare 下「花括号写在包装上」红）
            ("实现员花括号起 mutate.sh 跑 crates 整表：照拒", writer, f"cd {repo} && bash research/scripts/mutate.{{sh,x}} crates/mutations.tsv", 2),
            ("实现员花括号起 checker-tier-layer0-replay：照拒", writer, f"cd {repo} && bash .claude/gate.d/checker-tier-layer0-replay.{{sh,}} {repo}", 2),
            ("实现员参数里的通配点到 crates 整表：照拒", writer, f"cd {repo} && bash research/scripts/mutate.sh crates/mutation?.tsv", 2),
            ("实现员花括号写在包装上（{bash,-x}）：照拒", writer, f"cd {repo} && {{bash,-x}} research/scripts/mutate.sh crates/mutations.tsv", 2),
            ("实现员合法的花括号 ls research/scripts/{a,b}.sh：放行", writer, f"cd {repo} && ls research/scripts/{{a,b}}.sh", 0, 0, 0),
            ("实现员合法的参数通配 cat crates/*.toml：放行", writer, f"cd {repo} && cat crates/*.toml", 0, 0, 0),
            ("实现员起不存在的脚本：放行并记检出", writer, "bash missing.sh", 0, 1, 0),
            ("实现员 ./ 执行不存在的脚本：放行并记检出", writer, "./missing.sh", 0, 1, 0),
            ("实现员 bash 起不是文本的文件：放行并记检出", writer, "bash blob.bin", 0, 1, 0),
            ("实现员起超过上限的脚本：放行并记检出", writer, "bash big.sh", 0, 1, 0),
            ("实现员起的脚本嵌套 7 层：读到第 6 层、看不全，放行并记检出", writer, "bash deep1.sh", 0, 1, 6),
            # 同一条命令里先写出、再执行的脚本：拿写出的内容判，不记「不存在」。heredoc 正文里是第一例那条被拒的 cargo test --all
            ("实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它", writer,
             "cat > gen.sh <<'EOF'\n#!/usr/bin/env bash\ncargo test --all\nEOF\nbash gen.sh", 2, 0, 1),
            ("实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行", writer,
             "cat <<'EOF' > gen2.sh\ncargo test --all\nEOF\nchmod +x gen2.sh && ./gen2.sh", 2, 0, 1),
            ("实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source", writer,
             "tee gen3.sh <<'EOF' >/dev/null\ncargo test --all\nEOF\nsource gen3.sh", 2, 0, 1),
            ("实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它", writer,
             "cat > gen4.sh <<-EOF\n\tcargo test --all\n\tEOF\nbash gen4.sh", 2, 0, 1),
            ("实现员 cd 进子目录写出、回来按相对路径起", writer,
             "mkdir -p gen-dir && cd gen-dir && cat > here.sh <<'EOF'\ncargo test --all\nEOF\ncd .. && bash gen-dir/here.sh", 2, 0, 1),
            ("实现员 heredoc 盖掉盘上不含重型命令的 light.sh 再起它：判写出的内容", writer,
             "cat > light.sh <<'EOF'\ncargo test --all\nEOF\nbash light.sh", 2, 0, 1),
            ("实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本", writer,
             "cat > gen-outer.sh <<'EOF'\ncat > gen-inner.sh <<'IN'\ncargo test --all\nIN\nbash gen-inner.sh\nEOF\nbash gen-outer.sh", 2, 0, 2),
            ("实现员 bash -c 里写出含 --all 的脚本再起它", writer,
             "bash -c 'cat > gen-c.sh <<EOF\ncargo test --all\nEOF\nbash gen-c.sh'", 2, 0, 1),
            ("实现员命令替换里写出含 --all 的脚本再起它", writer,
             "out=\"$(cat > gen-sub.sh <<'EOF'\ncargo test --all\nEOF\nbash gen-sub.sh)\"", 2, 0, 1),
            ("实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份", writer,
             "cat > gen-maker.sh <<'EOF'\ncat > gen-made.sh <<'IN'\ncargo test --all\nIN\nEOF\nbash gen-maker.sh && bash gen-made.sh", 2, 0, 2),
            ("实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在", writer,
             "cat > gen8.sh <<'EOF'\ncargo test --all\nEOF\ncat >> gen8.sh <<'EOF'\necho 尾\nEOF\nbash gen8.sh", 2, 0, 1),
            ("实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在", writer,
             "tee -a s.sh <<'EOF'\necho 尾\nEOF\nbash s.sh", 2, 0, 1),
            ("实现员 cat -n 带选项写出：认不出，照旧记检出", writer,
             "cat -n > gen-numbered.sh <<'EOF'\ncargo test --all\nEOF\nbash gen-numbered.sh", 0, 1, 0),
            ("实现员 heredoc 写出不含重型命令的脚本再起它：放行、不记检出", writer,
             "cat > gen-light.sh <<'EOF'\nbash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract\necho \"cargo test --all\"\nEOF\n"
             "bash gen-light.sh", 0, 0, 1),
            ("主 agent 带前缀起 heredoc 写出的含 --all 的脚本", None,
             "cat > gen5.sh <<'EOF'\ncargo test --all\nEOF\n" + commit + "bash gen5.sh", 0, 0, 1),
            ("实现员 heredoc 写出 python 程序再 ./ 执行：不读也不记", writer,
             "cat > gen.py <<'EOF'\n#!/usr/bin/env python3\nprint('cargo test --all')\nEOF\n./gen.py", 0, 0, 0),
            ("实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件", writer, "cp s.sh copied.sh && bash copied.sh", 2, 0, 1),
            ("实现员 cp -f 到目录里再起：读源文件", writer, "cp -f s.sh elsewhere/ && bash elsewhere/s.sh", 2, 0, 1),
            ("实现员 cp 盘上不含重型命令的 light.sh 出来再起：放行、不记检出", writer, "cp light.sh copied-light.sh && ./copied-light.sh", 0, 0, 1),
            ("实现员 cp 同一条命令里 heredoc 写出的脚本再起", writer,
             "cat > gen6.sh <<'EOF'\ncargo test --all\nEOF\ncp gen6.sh gen7.sh && bash gen7.sh", 2, 0, 1),
            ("实现员 cp 的源不在：照旧记检出", writer, "cp nowhere.sh copied-missing.sh && bash copied-missing.sh", 0, 1, 0),
            ("实现员 printf 写出再起：认不出，照旧记检出", writer, "printf 'cargo test --all\\n' > printed.sh && bash printed.sh", 0, 1, 0),
            # 直接执行的只读 shell 脚本：`#!` 指到 shell，或没有 `#!` 而扩展名是 .sh；别的不读、不记
            ("实现员 ./ 直接执行 .rs 文件：不当脚本读、不记", writer, "./crates/singlefs-checker/src/walk.rs", 0, 0, 0),
            ("实现员 ./ 直接执行 .md 文件：不当脚本读、不记", writer, "./notes.md", 0, 0, 0),
            ("实现员 ./ 直接执行没扩展名、没 #! 的文本：不当脚本读、不记", writer, "./plain-text", 0, 0, 0),
            ("实现员 ./ 直接执行还不在的 .rs：不记「不存在」", writer, "./crates/singlefs-checker/src/missing.rs", 0, 0, 0),
            ("实现员 ./ 直接执行还不在、没扩展名的：不记「不存在」", writer, "bash research/scripts/run-with-memory-cap.sh 4G ./target/release/not-built-yet", 0, 0, 0),
            ("实现员 ./ 直接执行没扩展名、#! 指到 bash 的脚本：读进去照拒", writer, "./runner", 2, 0, 1),
            ("实现员 ./ 直接执行 .py 名字、#! 指到 sh 的脚本：读进去照拒", writer, "./mislabeled.py", 2, 0, 1),
            ("实现员 bash 起 .rs 文件：显式交给 shell 的照读，里面反引号括着的 .rs 不再读", writer, "bash crates/singlefs-checker/src/walk.rs", 2, 0, 1),
            ("实现员复现输入那一条（反引号里的 walk.rs）：末尾的 doc-lint.sh 是提交时才跑的检查、读它之前就拒，不记检出，一份脚本都不读", writer, SLOW_REPRODUCTION_COMMAND, 2, 0, 0),
            # 同一份脚本一次判定里只读一遍、只判一遍；换了当前目录、前缀的值、写出的内容，或那一遍撞上嵌套上限的，重判
            ("实现员同一条命令里起含 --all 的 s.sh 两遍：只判一遍", writer, "bash s.sh; ./s.sh", 2, 0, 1),
            ("实现员起的脚本里三种起法各起一遍 s.sh：s.sh 只判一遍", writer, "bash twice.sh", 2, 0, 2),
            ("主 agent 两遍都带前缀起 s.sh：只判一遍、放行", None, commit + "bash s.sh; " + commit + "bash s.sh", 0, 0, 1),
            ("主 agent 先带前缀、再不带起 s.sh：前缀的值不同重判，第二遍照拒", None, commit + "bash s.sh; bash s.sh", 2, 0, 2),
            ("实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判", writer,
             "bash inner-rel.sh; cat > inner-rel.sh <<'EOF'\ncargo test --all\nEOF\nbash inner-rel.sh", 2, 0, 2),
            ("实现员先起里面起了不存在脚本的 has-missing.sh、再经 outer-missing.sh 起它：检出照复用的那一遍记两条", writer,
             "bash has-missing.sh; bash outer-missing.sh", 0, 2, 2),
            ("实现员同一份 rel.sh 换个当前目录再起：按相对路径起的脚本变了，重判", writer, "bash rel.sh; cd sub && bash ../rel.sh", 2, 0, 4),
            ("实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判", writer, "bash deep1.sh; bash deep6.sh", 2, 1, 8),
            ("实现员先起 cyc-a.sh（里面绕回自己）、再起 cyc-b.sh：cyc-b.sh 那一遍绕回外层，重判", writer, "bash cyc-a.sh; bash cyc-b.sh", 2, 0, 3),
        ]
        # 登记成起重活的自证拿造出来的登记表判（只含一行假 heavy 脚本，lib_heavy_tests.build_heavy_only_registry）：实现员照拒、主 agent 带前缀放行。
        # LIB_HEAVY_TESTS_BREAK=selftest-heavy-rows-as-light 下第一、三格红，pattern-command-words-literal 下第三格红；LIB_HEAVY_TESTS_BREAK=commit-checks-ignored 与 HEAVY_TEST_GUARD_BREAK=ignore-commit-checks 下第一、三格红
        heavy_only_root = os.path.join(work, "heavy-only-registry")
        heavy_only_registry = heavy_tests.build_heavy_only_registry(heavy_only_root)
        heavy_only_script = os.path.join(heavy_only_root, heavy_tests.HEAVY_ONLY_SCRIPT)
        heavy_only_cases = [
            ("造出来的登记表：实现员跑登记成 heavy 的 --selftest，照拒", writer, f"python3 {heavy_only_script} --selftest", 2),
            ("造出来的登记表：主 agent带前缀跑登记成 heavy 的 --selftest（提交时才跑的检查），放行", None, commit + f"python3 {heavy_only_script} --selftest", 0),
            ("造出来的登记表：实现员 python3 通配起登记成 heavy 的自证，照拒", writer,
             f"cd {heavy_only_root} && python3 {heavy_tests.HEAVY_ONLY_SCRIPT[:-1]}? --selftest", 2),
        ]

        def judge_cases(case_list):
            for case in case_list:
                label, agent_type, command, want = case[:4]
                want_notices, want_read = case[4:6] if len(case) > 4 else (0, 0)
                hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
                if agent_type:
                    hook_input["agent_type"] = agent_type
                decision = decide(hook_input, work)
                results.append((label, want, decision.code))
                results.append((f"{label}：记几条检出", want_notices, len(decision.notices)))
                results.append((f"{label}：读进去几份脚本", want_read, len(decision.scripts_read)))
        judge_cases(cases)
        with heavy_tests.selftest_registry_swapped(heavy_only_registry, heavy_only_root):
            judge_cases(heavy_only_cases)
        # 同一份文件一次判定里只从盘上读一遍；直接执行的 .rs 要读开头判是不是脚本，判完不当脚本读，它里面括着的文件不碰
        def disk_reads(command, relative, agent_type=writer):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            return decide(hook_input, work).files_read.count(os.path.realpath(os.path.join(work, relative)))
        for label, command, relative, agent_type, want in [
                ("同一条命令里起 s.sh 两遍：s.sh 从盘上读一遍", "bash s.sh; ./s.sh", "s.sh", writer, 1),
                ("起的脚本里三种起法各起一遍 s.sh：s.sh 从盘上读一遍", "bash twice.sh", "s.sh", writer, 1),
                ("先带前缀、再不带起 s.sh：重判也不重读", commit + "bash s.sh; bash s.sh", "s.sh", None, 1),
                ("直接执行 .rs 两遍：判它是不是脚本要看开头，开头也只读一遍", "./crates/singlefs-checker/src/walk.rs; ./crates/singlefs-checker/src/walk.rs",
                 "crates/singlefs-checker/src/walk.rs", writer, 1),
                ("复现输入那一条：walk.rs 只看开头，它括着的 image.rs 不碰", SLOW_REPRODUCTION_COMMAND, "crates/singlefs-checker/src/image.rs", writer, 0)]:
            results.append((f"{label}（读盘次数）", want, disk_reads(command, relative, agent_type)))
        # 复现输入那一条要在 SLOW_REPRODUCTION_SECONDS 秒内判完
        started = time.monotonic()
        decide({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": SLOW_REPRODUCTION_COMMAND}}, work)
        elapsed = time.monotonic() - started
        results.append((f"复现输入那一条在 {SLOW_REPRODUCTION_SECONDS} 秒内判完（这一遍 {elapsed:.3f} 秒）", 1, int(elapsed < SLOW_REPRODUCTION_SECONDS)))
        # 拒绝说明写出脚本路径与行号：直接起的写到 s.sh 第 4 行，嵌套的连外层起它的那一行一起写
        for label, command, location in [("拒绝说明写出脚本路径与行号", "bash s.sh", f"脚本 {work}/s.sh:4 里的 cargo test"),
                                         ("嵌套的拒绝说明写出每一层的路径与行号", "bash outer.sh", f"脚本 {work}/outer.sh:3 → {work}/s.sh:4 里的 cargo test"),
                                         ("命令替换里的拒绝说明写出它所在那一行", "bash substitution-heavy.sh", f"脚本 {work}/substitution-heavy.sh:3 里的 cargo test"),
                                         ("heredoc 写出的脚本，拒绝说明写出它的路径与正文里的行号",
                                          "cat > gen.sh <<'EOF'\n#!/usr/bin/env bash\ncargo test --all\nEOF\nbash gen.sh", f"脚本 {work}/gen.sh:2 里的 cargo test"),
                                         ("heredoc 写出的脚本里再写出的，逐层写出路径与行号",
                                          "cat > gen-outer.sh <<'EOF'\ncat > gen-inner.sh <<'IN'\ncargo test --all\nIN\nbash gen-inner.sh\nEOF\nbash gen-outer.sh",
                                          f"脚本 {work}/gen-outer.sh:4 → {work}/gen-inner.sh:1 里的 cargo test"),
                                         ("复用上一遍的结果，拒绝说明照这一处的路径与行号写",
                                          "bash s.sh; bash outer.sh", f"脚本 {work}/outer.sh:3 → {work}/s.sh:4 里的 cargo test"),
                                         ("绕回外层的那一遍不复用，从别处起时照读回来", "bash cyc-a.sh; bash cyc-b.sh",
                                          f"脚本 {work}/cyc-b.sh:1 → {work}/cyc-a.sh:2 里的 cargo test")]:
            message = decide({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": command}}, work).message or ""
            results.append((label, 1, int(location in message and "这一步写在脚本里" in message)))
        # 复用上一遍的检出，照这一处的路径与行号写
        reused_notices = decide({"tool_name": "Bash", "cwd": work, "agent_type": writer,
                                 "tool_input": {"command": "bash has-missing.sh; bash outer-missing.sh"}}, work).notices
        results.append(("复用上一遍的检出，照这一处的路径与行号写", 1,
                        int(any(f"脚本 {work}/outer-missing.sh:2 → {work}/has-missing.sh:1 里" in notice for notice in reused_notices))))
        # 切词：source / . 后面跟的像一个路径才算读脚本（python 源码里 `source = …` 那种不算）
        for label, command, want in [("切词：source = … 不算读脚本", "source = open", [("source", None)]),
                                     ("切词：. = … 不算读脚本", ". = x", [(".", None)]),
                                     ("切词：. ./env.sh 算读脚本", ". ./env.sh", [("./env.sh", "source")])]:
            got = [(found.words[0], found.launcher) for found in shell_words.commands_at_command_position(command, work).commands]
            results.append((label, want, got))
        # 切词：进程替换的另一种写法 `>(…)` 同 `<(…)`：里面的命令另判，`)` 之后的词是外层命令的参数
        got = [(found.words[0], found.launcher) for found in shell_words.commands_at_command_position("tee >(bash run.sh) out.log", work).commands]
        results.append(("切词：tee >(bash run.sh) out.log 里 run.sh 另判、out.log 是 tee 的参数", [("run.sh", "shell"), ("tee", None)], got))
        # 切词：heredoc 正文留不留，看它挂在哪条命令上（从标准输入读命令的 bash / sh 才留），不看同一行别处有没有 shell 的名字
        for label, command, kept in [("切词：前一条起了 x.sh、heredoc 喂给 python3 -：正文剥掉", "bash light.sh && python3 - <<'PY'\nBODY\nPY", False),
                                     ("切词：heredoc 喂给 bash：正文留着", "bash <<'EOF'\nBODY\nEOF", True),
                                     ("切词：heredoc 喂给 capped.sh 4 bash -s：正文留着", "capped.sh 4 bash -s -- a <<'EOF'\nBODY\nEOF", True),
                                     ("切词：heredoc 喂给 bash -c：正文剥掉", "bash -c 'cat' <<'EOF'\nBODY\nEOF", False),
                                     ("切词：heredoc 喂给 bash 起的脚本：正文剥掉", "bash light.sh <<'EOF'\nBODY\nEOF", False)]:
            results.append((label, kept, "BODY" in shell_words.strip_data_heredocs(command)))
        # 共用判定模块自己的自检（cargo 命令行与直接执行的测试二进制两种输入）
        results.append(("lib_heavy_tests.py 的自检", 0, heavy_tests.selftest()))
        results.append(("检出记录默认放在工作根下（lib_work_root.py）",
                        os.path.join(work_roots.configured_work_root(), "agent-hook-detections.jsonl") if work_roots else "读得到 lib_work_root.py", DEFAULT_DETECTIONS))
        # 走真实入口：从标准输入喂 JSON，拒绝退出码 2、stderr 带出路，放行退出码 0；拒绝落进检出记录
        script = os.path.join(hook_dir, "heavy-test-guard.sh")
        detections = os.path.join(work, "detections.jsonl")
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=work, AGENT_HOOK_DETECTIONS=detections)
        def count_detections():
            return sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
        refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "cargo test --all"}}))
        results.append(("stdin:实现员跑 --all 拒绝（退出码 2）", 2, refused.returncode))
        results.append(("stdin:拒绝时 stderr 写了出路与前缀", 1,
                        int("→" in refused.stderr and "SINGLEFS_HEAVY_TESTS=commit" in refused.stderr and "弹窗" in refused.stderr)))
        # 「→ 规矩」那一串列的重型测试带真编 QEMU（POLICY 里删掉这一项，这一格红）
        results.append(("stdin:拒绝时 stderr 的「→ 规矩」那一串列出真编 QEMU", 1,
                        int(any(line.startswith("→ 规矩：重型测试") and "真编 QEMU" in line for line in refused.stderr.splitlines()))))
        allowed = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": commit + "cargo test --all"}}))
        results.append(("stdin:主 agent 带前缀放行（退出码 0）", 0, allowed.returncode))
        recorded = count_detections()
        results.append(("stdin:拒绝落进检出记录", 1, recorded))
        # 走真实入口：提交时才跑的检查被拒，退出码 2、stderr 的出路点名 gate-staged.sh 与放行的两个子 agent
        commit_check_refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                              input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": "tooling-writer",
                                                                "tool_input": {"command": "bash .claude/singlefs-ai-sop/scripts/doc-lint.sh ."}}))
        results.append(("stdin:工具实现员跑 doc-lint.sh 拒绝（退出码 2）", 2, commit_check_refused.returncode))
        results.append(("stdin:提交时才跑的检查被拒时 stderr 的出路点名 gate-staged.sh 与主 agent 照的那份做法", 1,
                        int("✗ 提交时才跑的检查被拒" in commit_check_refused.stderr and "research/scripts/gate-staged.sh" in commit_check_refused.stderr
                            and ".claude/skills/gate/staged-gate.md" in commit_check_refused.stderr)))
        # 走真实入口：主 agent 没带前缀被拒，stderr 最前面是照抄的说明与加好前缀的那条（照它再喂一遍放行），规矩只留一行出处、不整段贴；
        # 检出记录写结局「被拒」（HEAVY_TEST_GUARD_BREAK=main-full-policy 下前两格红，detection-outcome-dropped 下第三格红）
        def last_detection():
            """检出记录的最后一条；还没有记录时交 {}（那几格照样判红，不让自检抛异常）。"""
            if not os.path.exists(detections):
                return {}
            with open(detections, encoding="utf-8") as handle:
                lines = handle.read().splitlines()
            return json.loads(lines[-1]) if lines else {}
        main_command = "bash .claude/singlefs-ai-sop/scripts/doc-lint.sh ."
        main_refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                      input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": main_command}}))
        main_lines = main_refused.stderr.splitlines()
        main_outcome = last_detection().get("outcome")
        copied = main_lines[1] if len(main_lines) > 1 and main_lines[0].startswith("→ 照抄") else ""
        copied_judged = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": copied}}))
        results.append(("stdin:主 agent 没带前缀被拒时最前面是能照抄的带前缀命令（照抄的那条再判放行）", (2, "SINGLEFS_HEAVY_TESTS=commit " + main_command, 0),
                        (main_refused.returncode, copied, copied_judged.returncode)))
        results.append(("stdin:主 agent 被拒时规矩只留一行出处（不整段贴）", True,
                        "implementation-workflow.md「checker 档只在提交时跑，harness 随时跑」" in main_refused.stderr
                        and "→ 交回前不跑它们" not in main_refused.stderr and len(main_lines) <= 4))
        results.append(("stdin:被拒的检出记录写了结局「被拒」", "被拒", main_outcome))
        # 两份表里提交时才跑的检查是同一组 kind（共用模块读不到时这里的 COMMIT_CHECK_KINDS 照样要认得这一类）
        results.append(("这里的 COMMIT_CHECK_KINDS 与 lib_heavy_tests.COMMIT_CHECK_KINDS 相同", True,
                        COMMIT_CHECK_KINDS == set(heavy_tests.COMMIT_CHECK_KINDS)))
        results.append(("这里的 UNREGISTERED_SELFTEST_KIND 与 lib_heavy_tests.UNREGISTERED_SELFTEST_KIND 相同", True,
                        UNREGISTERED_SELFTEST_KIND == heavy_tests.UNREGISTERED_SELFTEST_KIND))
        # 走真实入口：登记表外的 --selftest 被拒，退出码 2、stderr 点名登记表与登记的写法（HEAVY_TEST_GUARD_BREAK=drop-registry-hint 下第二格红）；
        # 登记成不起重活的单独跑放行，退出码 0、不记检出
        unregistered_refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                              input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer,
                                                                "tool_input": {"command": "python3 research/scripts/brand-new-tool.py --selftest"}}))
        results.append(("stdin:实现员跑登记表外的 --selftest 拒绝（退出码 2）", 2, unregistered_refused.returncode))
        results.append(("stdin:登记表外的 --selftest 被拒时 stderr 点名登记表与登记的写法", 1,
                        int(".claude/hooks/selftest-registry.tsv" in unregistered_refused.stderr and "<制表符>light<制表符>#" in unregistered_refused.stderr
                            and "classify_selftest" in unregistered_refused.stderr)))
        recorded = count_detections()
        light_selftest = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                        input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": "tooling-writer",
                                                          "tool_input": {"command": f"bash {repo}/research/scripts/crash-amplification.sh --selftest"}}))
        results.append(("stdin:工具实现员单独跑登记成不起重活的 crash-amplification.sh --selftest 放行（退出码 0）、不记检出", (0, recorded),
                        (light_selftest.returncode, count_detections())))
        unknown_breaks = [item for item in os.environ.get(GUARD_BREAK_VARIABLE, "").split(",") if item and item not in KNOWN_GUARD_BREAK_ITEMS]
        results.append((f"{GUARD_BREAK_VARIABLE} 里的项都认得（认得的：{'、'.join(KNOWN_GUARD_BREAK_ITEMS)}）", [], unknown_breaks))
        # 走真实入口：stage-selftest.sh 喂有起重活样本的阶段目录被拒，退出码 2、stderr 点名那一道与单跑一道样本的写法（LIB_HEAVY_TESTS_BREAK=stage-samples-ignored 下第二格红）
        heavy_samples = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": "tooling-writer", "tool_input": {
                                           "command": f"bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-heavy/.claude/gate.d"}}))
        results.append(("stdin:stage-selftest.sh 喂有起重活样本的阶段目录拒绝（退出码 2）", 2, heavy_samples.returncode))
        results.append(("stdin:被拒时 stderr 点名起重活的那一道、写单跑一道样本的写法", 1,
                        int("checker-tier-crates-mutation-replay.sh 的样本档登记成起重活" in heavy_samples.stderr and "cd 进样本根、根参数写同一个目录" in heavy_samples.stderr)))
        # 走真实入口：--item 点名里夹一道起重活的被拒，stderr 点名那一道、写 --item 的写法（LIB_HEAVY_TESTS_BREAK=stage-items-unjudged 下两格都红）
        heavy_item = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                    input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": "tooling-writer", "tool_input": {
                                        "command": f"bash {repo}/.claude/singlefs-ai-sop/scripts/stage-selftest.sh {work}/gd-heavy/.claude/gate.d "
                                                   "--item doc-text.sh --item checker-tier-crates-mutation-replay.sh"}}))
        results.append(("stdin:stage-selftest.sh --item 点名里夹起重活的拒绝（退出码 2）", 2, heavy_item.returncode))
        results.append(("stdin:被拒时 stderr 点名起重活的那一道、写 --item 的写法", 1,
                        int("checker-tier-crates-mutation-replay.sh 的样本档登记成起重活" in heavy_item.stderr and "--item <阶段文件名>" in heavy_item.stderr)))
        # 走真实入口：实现员起含被拒命令的脚本，退出码 2、stderr 写出脚本路径与行号
        in_script = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                   input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "bash s.sh"}}))
        results.append(("stdin:实现员起含被拒命令的脚本拒绝（退出码 2）", 2, in_script.returncode))
        results.append(("stdin:脚本里的拒绝 stderr 写出脚本路径与行号", 1, int(f"{work}/s.sh:4" in in_script.stderr)))
        # 走真实入口：实现员不经内存包装跑测试目标，退出码 2、stderr 写出经包装跑的写法
        uncapped_run = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                      input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer,
                                                        "tool_input": {"command": "cargo test -p singlefs-core --lib"}}))
        results.append(("stdin:实现员不经内存包装跑测试目标拒绝（退出码 2）", 2, uncapped_run.returncode))
        results.append(("stdin:不经内存包装的拒绝 stderr 写了出路", 1,
                        int("→" in uncapped_run.stderr and "run-with-memory-cap.sh <上限> <命令>" in uncapped_run.stderr
                            and "不经内存包装跑编译出来的代码被拒" in uncapped_run.stderr)))
        recorded = count_detections()
        # 走真实入口：脚本文件不存在时照常放行（退出码 0），stderr 点名那个文件，并记一条检出
        missing = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "bash missing.sh"}}))
        results.append(("stdin:脚本文件不存在照常放行（退出码 0）", 0, missing.returncode))
        results.append(("stdin:脚本文件不存在时 stderr 点名它、记一条检出", 1,
                        int(f"{work}/missing.sh" in missing.stderr and count_detections() == recorded + 1)))
        results.append(("stdin:放行时的检出记录写了结局「只记」", "只记", last_detection().get("outcome")))
        recorded = count_detections()
        # 走真实入口：同一条命令里 heredoc 写出不含重型命令的脚本再起它，照常放行（退出码 0），stderr 不报、不记检出
        written_light = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                           "command": "cat > gen-light.sh <<'EOF'\necho 轻\nEOF\nbash gen-light.sh"}}))
        results.append(("stdin:heredoc 写出的轻脚本照常放行（退出码 0）", 0, written_light.returncode))
        results.append(("stdin:heredoc 写出的轻脚本 stderr 不报「不存在」、不记检出", 1,
                        int("不存在" not in written_light.stderr and count_detections() == recorded)))
        written_heavy = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                           "command": "cat > gen.sh <<'EOF'\ncargo test --all\nEOF\nbash gen.sh"}}))
        results.append(("stdin:heredoc 写出的含 --all 的脚本拒绝（退出码 2）", 2, written_heavy.returncode))
        results.append(("stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号", 1, int(f"{work}/gen.sh:1" in written_heavy.stderr)))
        recorded = count_detections()
        # 走真实入口：hook 旁边没有共用模块时照常放行（退出码 0），stderr 点名两份模块，并记一条检出
        lonely_hook_directory = os.path.join(work, "hook-without-shared-modules")
        os.makedirs(lonely_hook_directory)
        shutil.copy(script, lonely_hook_directory)
        without_library = subprocess.run(["bash", os.path.join(lonely_hook_directory, "heavy-test-guard.sh")], capture_output=True, text=True, env=environment,
                                         input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "cargo test --all"}}))
        results.append(("stdin:读不到共用模块照常放行（退出码 0）", 0, without_library.returncode))
        results.append(("stdin:读不到共用模块时 stderr 点名两份模块、记一条检出", 1,
                        int("lib_heavy_tests.py" in without_library.stderr and "lib_shell_words.py" in without_library.stderr
                            and count_detections() == recorded + 1)))
        # 走真实入口：admission.py 导入得了、判的那一刻抛异常（判法里调的名字没跟上）——判不出，点名登记目标的 cargo test 照拒，不落到入口的「没判成、照常执行」
        # （lib_heavy_tests.py 的 judging-error-propagates 打开时这一格红）
        raising_repository = os.path.join(work, "repository-with-raising-admission")
        os.makedirs(os.path.join(raising_repository, ".claude", "hooks"))
        os.makedirs(os.path.join(raising_repository, "research", "scripts"))
        for name in ("heavy-test-guard.sh", "lib_heavy_tests.py", "lib_shell_words.py"):
            shutil.copy(os.path.join(hook_dir, name), os.path.join(raising_repository, ".claude", "hooks", name))
        for name in ("scripts", "singlefs-ai-sop"):
            os.symlink(os.path.join(os.path.dirname(hook_dir), name), os.path.join(raising_repository, ".claude", name))
        with open(os.path.join(raising_repository, "research", "scripts", "admission.py"), "w", encoding="utf-8") as handle:
            handle.write("def test_function_is_marked_ignored(*arguments):\n    return definitions_marked_ignored(*arguments)\n")
        raising = subprocess.run(["bash", os.path.join(raising_repository, ".claude", "hooks", "heavy-test-guard.sh")], capture_output=True, text=True,
                                 env=environment, input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                     "command": "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream"}}))
        results.append(("stdin:admission.py 判的那一刻抛异常时，点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 照拒（退出码 2）", 2, raising.returncode))
        results.append(("stdin:admission.py 判的那一刻抛异常时 stderr 不是「没判成、照常执行」", 1, int("没判成" not in raising.stderr)))
    finally:
        heavy_tests.SAMPLE_INDEX_REPOSITORY = real_sample_index
        shutil.rmtree(work)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 heavy_uses()、script_path_of()、executed_file_is_shell_script()、file_on_disk()、script_text()、command_lines()、refusal_reason() "
              "与共用的 lib_heavy_tests.py、lib_shell_words.py；"
              "HEAVY_TEST_GUARD_BREAK 设着的话（项与红的格见文件头）这里本来就该红")
        return 1
    refused_count = sum(1 for item in results if item[1] == 2)
    print(f"  ✓ 自检通过（查了 {len(results)} 种，其中该拒 {refused_count} 种）：子 agent 跑 checker 档（按包判）、全量测试、check.sh、整轮门禁、QEMU、herd7、crates 变异整表、Verus 那一道与它的判法脚本拒绝，harness 档经内存包装放行，"
          "子 agent 带不带前缀都拒绝，主 agent 带 commit / user-request 放行、不带或带别的值拒绝；"
          "写进脚本再执行的（bash / ./ / capped.sh / source / 嵌套）读进去照判、拒绝写出路径与行号，脚本读不到或看不全的放行并记检出；"
          "同一条命令里 heredoc（cat / tee）写出或 cp 拷出再执行的拿写出的内容判、不记「不存在」，认不出的写法照旧记检出；"
          "直接执行的只读 `#!` 指到 shell 的与没 `#!` 的 .sh（.rs、.md、没扩展名的文本不读不记），同一份脚本一次判定里只读一遍、只判一遍；"
          "子 agent 不经 run-with-memory-cap.sh 跑 cargo test / run / bench 与 cargo 编出来的二进制拒绝、经它包着的（连同 bash -c 与它起的脚本）放行；"
          "只跑动到的测试目标、fmt / clippy / build、把名字当参数的、只查清单与只查语法的放行；"
          "提交时才跑的检查（不重型的门禁阶段、上游几道 lint 与样本自检、登记成起重活或没登记的自证）只放行带前缀的主 agent，拒绝写出路 gate-staged.sh；"
          "登记表 selftest-registry.tsv 里记成不起重活的自证单独跑谁都放行、不读正文，没登记的拒绝写怎么登记；"
          "门禁样本经 stage-selftest.sh 喂、阶段目录里样本档全登记成轻的放行，--item 只判点名的几道、--list-items 放行，登记了假工具标记的那一道 cd 进样本根直接起放行，起重活的样本与直接对真仓跑照拒、别处树里的 stage-selftest.sh 拷贝照拒；"
          "命令词带通配的照 bash 展开、按展开出来的那几份判，带 POSIX 字符类、等价类、整理符号的交给 bash 自己展开、判不出的命令词照拒；"
          "命令词与参数里的花括号、参数里的通配照 bash 展开再判；崩溃放量的三份入口按名字算重型、只列分项与登记成轻的自证放行，"
          "子 agent 一律拒，主 agent 带前缀能跑（直接起与经 detached-run.sh start 起一样判）；"
          "主 agent 没带前缀被拒时最前面给加好前缀、照同一张表再判放行的那条命令，规矩只留出处一行；检出记录写结局（被拒 / 只记）")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except Exception:
        return 0
    if not isinstance(hook_input, dict):
        return 0
    detections_path = os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS
    if heavy_tests is None:
        warning = f"heavy-test-guard.sh {shared_library_problem(hook_dir)}，这条命令没判、照常执行"
        print(f"  ! {warning}", file=sys.stderr)
        record_detection(hook_input, [warning], detections_path, "只记")
        return 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    try:
        decision = decide(hook_input, project_root)
    except Exception as error:
        print(f"  ! heavy-test-guard.sh 没判成（{error!r}），这条命令照常执行", file=sys.stderr)
        return 0
    findings = [f"重型测试闸没判全：{notice}" for notice in decision.notices]
    for notice in decision.notices:
        print(f"  ! heavy-test-guard.sh：{notice}；{'' if decision.code else '这条命令照常执行，'}记了一条检出", file=sys.stderr)
    if decision.code:
        print(decision.message, file=sys.stderr)
        message_lines = decision.message.splitlines()
        findings.append(next((line for line in message_lines if line.startswith("✗ ")), message_lines[0]).removeprefix("✗ "))
    if findings:
        record_detection(hook_input, findings, detections_path, "被拒" if decision.code else "只记")
    return decision.code

sys.exit(main())
PY
