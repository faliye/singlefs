#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Bash）：起验证脚本的命令要在开头带一份范围声明（跑什么、范围多大、为什么这么跑），种类与范围要与命令的形状对得上；
#   没声明、缺段、对不上、子 agent 起整轮的，执行前拒绝（退出 2，stderr 写给 agent 的说明）；放行的每一条把命令与四段声明追加进检出记录，
#   范围合不合适由主 agent 事后对着记录看。主 agent 与子 agent 一样判，子 agent 另拒 whole。
# hook-events: PreToolUse:Bash
# gate-similar: heavy-test-guard.sh 同挂 PreToolUse[Bash]、同样认门禁阶段与 cargo test、同样把执行的脚本读进去逐行判，切词与剥包装都从它用的 lib_shell_words.py、lib_heavy_tests.py 导入；它判的是重型测试谁能跑、带没带 SINGLEFS_HEAVY_TESTS、经没经内存包装，这里判的是每一次起验证（轻的也算）带没带范围声明、种类与范围对不对得上，放行条件与拒绝说明都不同，并进去会让一个钩子同时管两套规矩
# gate-similar: bash-command-detector.sh 同挂 PreToolUse[Bash]、同用 lib_shell_words.py 切词；它判命令的形状（等待循环、放出追踪、单独的 &、覆盖产物、终止进程、就地改文件），不认验证脚本也不看环境变量里的声明
# gate-similar: timer-only-remind-guard.sh 的 matcher 含 Bash、同用 lib_shell_words.py；它拒 remind.sh 之外的定时写法，与起验证脚本无关
# gate-similar: host-resources-reminder.sh 同挂 PreToolUse[Bash]；它认出重活或 GPU 命令时查各台资源出提醒、从不拒，这里拒没带范围声明的验证命令，不查外部资源
# gate-similar: pattern-process-guard.sh 上游的钩子，同挂 PreToolUse[Bash]；它拒按模式找进程，判据与 shell-lint 同一份，与验证脚本的范围无关，也不许在上游副本里加本工程的判据
#
# 为什么：用户 2026-10-15「该做的是红了只跑红的、改了只点名改到的。做成门禁。跑之前提醒，这个东西靠prompt 根本控制不住」，
#   「提醒 agent 是跑单个一条 还是跑整个 让agent解释清楚 选好参数才跑」「agent 启动脚本的时候 必须要解释清楚 跑什么 范围多大 为什么要这么跑。范围是不是合适。」
#   当天三个验证驱动都整套重跑、合入后验证每次跑全部 212 个测试二进制，而那一轮补丁只碰 unit-graph 与 checker 档（records/2026-10-13-复用键与全仓改名.md 第 686、689–691 行）。
#
# 声明的写法：环境赋值写在命令开头（写进 env 的参数、同一段命令文本前面的 export 也认）：
#   SINGLEFS_RUN_SCOPE='种类=<whole|items|quick|target>; 跑什么=<哪道门禁 / 入口 / 包与目标>; 范围=<N 项 / N 格 / N 个测试 / 整轮 / 整包>; 为什么=<为什么这么跑、这个范围为什么合适>'
#   四段用 ; 隔、各段 键=值，键只认这四个；每段都要有、值非空、同一段不许写两次；不以「键=」起头的那一截接到上一段后面（值里自带的 ;）。
#   为什么：去掉「要跑」「验证」「跑一下」「跑一遍」「看看」「确认一下」之后，字母、汉字与数字至少 12 个（空白与标点不算）；
#   整轮门禁（gate-staged.sh、gate.sh）的为什么里写了「提交前整轮」就放，不数字数。
# 要声明的命令（只认命令位置）。包在外面的照剥、剥出来的照同一张表判，外面写的声明带进里面那一条：bash -c、run-with-memory-cap.sh、capped.sh、env、nice、timeout、
#   stdbuf 这类由 lib_shell_words.py 剥；flock、/usr/bin/time、systemd-run、chrt、prlimit、strace、perf 与 scripts/hosts/detached-run.sh 的 start … -- 由 lib_heavy_tests.py
#   的 command_under_launcher 剥；scripts/crash/gpu-quiesce-run.sh -- 由 command_under_gpu_quiesce 剥。
#   门禁阶段    .claude/gate.d/ 顶层的 *.sh（按路径认，lib_heavy_tests.py 的 gate_stage_name；lib/ 下的不算）：带 --quick 是 quick（带不带 --item 都算）；
#               带 --check / --item 是 items，N = --check 点名的格（逗号隔的逐个算）与 --item 点名的项，各自去重后相加；都不带是 whole。根参数给不给一样算。
#   崩溃点重放  scripts/crash/crash-point-replay.sh：--quick 是 quick；带 --item 是 items（N = 项数，带不带 --full 一样）；不带 --item 的剪枝跑与 --full 是 whole。
#   崩溃放量    scripts/crash/crash-amplification.sh：带 --item 是 items（N = 分项数）；不带 --item 的（--expand-up-to 缩过规模的也算）是 whole。
#               scripts/crash/crash-amplification-item.sh、crash-amplification-two-hosts.sh 起一个测试目标：target，N = 1（范围写「1 个测试目标」，带 --test-filter 的写「1 个测试」）。
#   实验复跑    scripts/experiments/replay.sh：不带实验号是 whole，带实验号是 items（N = 实验号个数，范围写「N 个实验」）。
#   合入后验证  scripts/gate/merge-and-verify.sh：whole（挑包在脚本里做：只跑补丁改到的包与依赖它们的包，--all-packages 与它文件头写的另几种照旧全跑；
#               声明照旧写 whole，为什么写明这一批补丁碰了哪些包）。
#   整轮门禁    scripts/gate/gate-staged.sh、.claude/singlefs-ai-sop/scripts/gate.sh 与它的包装 .claude/scripts/gate.sh：whole。
#   cargo test  cargo test 与 cargo t（cargo 的全局选项与 +工具链照 lib_heavy_tests.py 的 cargo_subcommand 跳过）：带 --test / --bin / --example / --bench <名>、--lib、--doc，
#               又不带 --workspace / --all / --tests / --bins / --examples / --benches / --all-targets 的是 target，N = 目标数；`--` 之后带 --exact 又给了筛选词的，
#               N = 筛选词个数（范围写「N 个测试」）。别的（不挑目标的整包、整工作区）是 whole，范围写「整包」或「整轮」。
#   驱动脚本    命令位置上执行的别的 shell 脚本（bash / sh / source 起的；./x.sh、路径/x 直接执行而 #! 指到 shell、或没有 #! 而扩展名是 .sh 的；
#               同一条命令里前面 `cat > 文件 <<EOF`、`cat <<EOF > 文件`、`tee [-a] 文件 <<EOF` 写出的，拿写出的正文判）读进去，里面每一条要声明的命令照同一张表判。
#               外面那一条的声明不带进脚本：脚本里每一条要在自己那一行（或脚本里它前面的 export）带声明，缺的、对不上的逐条列出（脚本:行）拒。
#               读到第 6 层为止，绕回外层的不再读；超过 1 MiB、有 NUL 字节、不是 UTF-8 的不读。
#               仓里的工具脚本不读：路径落在某一层带 .claude/hooks/ 的目录下的 scripts/ 或 .claude/ 里（主工作区与仓副本都算），它们里面起的验证由调它们的那一条负责。
# 范围一段：items 与 target 的范围里第一个阿拉伯数字等于 N；whole 的范围里有「整轮」或「整包」；quick 只要求非空。种类对不上时不再核范围。
# 种类与命令的形状对不上就拒，说明写「声明的是点名的几项（items），命令跑的是整轮（whole）」这类。
# 子 agent（hook 输入带 agent_type）的 whole 一律拒，说明写「交回主 agent」；quick、items、target 带对的声明放。主 agent 带对的声明照放（重型的另由 heavy-test-guard.sh 判前缀）。
# 不要声明的形态（照常放行、不记）：
#   门禁阶段带 --list、--list-items、--selftest、--help、-h、--tool-versions（只打格名、项名、自证、帮助、工具版本）；
#   crash-point-replay.sh 与 crash-amplification.sh 带 --list-items（含 --with-keys）、--selftest、--help、-h；
#   crash-amplification-item.sh、crash-amplification-two-hosts.sh、merge-and-verify.sh、gate-staged.sh 带 --selftest、--help、-h；
#   replay.sh 带 --selftest、--admission-only（只问准入，不编、不跑、不比）、--help、-h；
#   cargo test 带 --no-run（只编不跑）或 libtest 的 --list（只列用例，判法同 lib_heavy_tests.py 的 libtest_lists_only）；cargo 的别的子命令（build、clippy、fmt、run、bench……）；
#   上面几类之外的命令：钩子与研究脚本的 --selftest、stage-selftest.sh 喂样本、python 起的脚本、仓里的工具脚本等。各入口的 --force 先摘掉再判（同它们的 preflight）。
# 拒绝说明（stderr，写给 agent）：逐条写命令（驱动脚本里的写 脚本:行）、跑的是什么、范围多大、哪里不合、上次这一道红的清单；
#   再给照抄的声明模板（种类、跑什么、范围按命令现算好，为什么留尖括号让 agent 填；驱动脚本里的写「在 <脚本> 那一行的命令前面加」，命令超过 2000 字的不整条贴），
#   只跑红的与改到的写法（门禁 --check <格> / --item <项>，崩溃入口 --item <项>，cargo test -p <包> --test <目标> -- <测试名> --exact 或
#   python3 scripts/gate/rerun-failed-tests.py <上次日志> -p <包>（它打出的每条命令开头已带 target 声明），merge-and-verify 写挑包在脚本里做、种类照旧写 whole），与声明的写法。
#   上次红的清单：读 <目录>/<脚本相对仓根的路径，/ 换成 __>.tsv（目录是 SINGLEFS_LAST_RUNS_DIRECTORY，没设是 ~/.local/share/singlefs/last-runs；
#   一行一个项名，空行与 # 开头的行不算）；没有这份文件或是空的写「没有上次红的记录」。各道跑完写这份文件归下一批。cargo test 没有这份文件，说明里指到 rerun-failed-tests.py。
# 检出记录（AGENT_HOOK_DETECTIONS，没设是 lib_work_root.py 的 default_detections_path）：
#   放行的每一条要声明的命令追加一行：hook、run_scope（在哪：命令文本或 脚本:行；剥完包装的词；命令的形状；跑的是什么；声明原文与切开的四段）、command、agent_type、agent_id、
#   transcript_path、outcome「放行」；会话号写在 scope_session_id，不写 session_id——看门狗 scripts/agents/agent-watch.py 的 read_detections 只读带本会话 session_id 的行，
#   带了它每一次放行都会当一条 hook 检出叫醒主 agent。要看门狗也把放行列进报告，先在它只记不叫醒的那一类（INFO_DETECTION_PREFIXES）里认这种行，再改成写 session_id。
#   被拒的整条命令照别的钩子记一行：session_id、findings（每条以「范围声明被拒」起头）、outcome「被拒」，看门狗只记不叫醒。
# 罩不住的：变量里拼出来的命令、eval、python 的 subprocess、make 与 xargs 起的验证；命令词写在变量里的；cargo nextest、cargo miri、cargo llvm-cov 这类别的测试入口；
#   .claude/scripts/check.sh 与 scripts/mutation/ 下的变异驱动（不在要声明的那张表里）；读不到的驱动脚本（不在、太大、不是文本）不判、照常放行；
#   同一条命令里用 printf、echo、cp、python 写出再执行的脚本，判的是盘上那一份（还不在就不判）；
#   声明里写的「跑什么」「为什么」说得对不对、范围合不合适（只判写没写、够不够长、数对不对得上），由主 agent 对着检出记录看。
# 判法在同目录的 lib_run_scope.py（切词与剥包装从 lib_shell_words.py、lib_heavy_tests.py 导入，不另写一份）。判的时候出异常、读不到判法文件，
#   照常放行并往 stderr 报一句：这个钩子挂在每一条 Bash 上，坏了不许把所有命令都拒掉。
#
# 用法：
#   注册：.claude/settings.json 的 hooks.PreToolUse 里 matcher "Bash" 那一条的 hooks 加 {"type": "command", "command": "bash \"$CLAUDE_PROJECT_DIR\"/.claude/hooks/run-scope-guard.sh"}
#   自证：bash .claude/hooks/run-scope-guard.sh --selftest（登记在 .claude/hooks/selftest-registry.tsv，light）
# 自证：在临时目录里造驱动脚本与一份仓的样子；每种写法一绿一红（门禁整道、--check、--item、--quick、查清单；崩溃点重放、崩溃放量与它的两份驱动；replay.sh；
#   merge-and-verify；gate-staged.sh 与 gate.sh；cargo test 整工作区、整包、点名目标、--exact；bash -c、capped.sh、env、nice 与 timeout、export、flock、gpu-quiesce-run.sh、
#   detached-run.sh start、/usr/bin/time；驱动脚本缺声明与都带、外面带声明、套两层、直接执行、heredoc 写出、仓里的工具脚本；缺段、值空、写两次、种类认不出、
#   种类对不上、范围数对不上、为什么太短；子 agent 的 whole 拒与 quick、items、target 放），在内存里逐格判；另起这份入口喂 hook 的 JSON，
#   核退出码、拒绝说明里的模板与只跑红的写法、上次红的清单、放行与被拒的检出记录、判法文件读不到时放行。末了开关轮：下面每个开关逐个打开再判一遍，点名的格都要红。
# 弄坏开关 RUN_SCOPE_GUARD_BREAK=<项>（几个用逗号连；只给自证判红用；认不出的项抛异常，钩子照常放行并报一句）。
#   设了开关再跑 --selftest：照它判一遍，下面写的格红、退 1，开关轮不跑；不设时末了跑开关轮，逐个打开、核红的格与这里写的逐格相等。逐项写开关、弄坏哪一处、红的格（写全）：
#   declaration-optional  没带声明也放
#       红：「门禁整道：没带声明拒」「崩溃点重放剪枝跑：没带声明拒」「崩溃放量全部分项：没带声明拒」「崩溃放量双机驱动：没带声明拒」「replay.sh 全部实验：没带声明拒」「gate-staged.sh 没带声明拒」「cargo test 整工作区：没带声明拒」「bash -c 里的门禁没带声明拒」「capped.sh 包着的 cargo test 没带声明拒」「env 带别的变量、没带声明拒」「nice 与 timeout 包着、没带声明拒」「flock 包着的门禁没带声明拒」「gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒」「detached-run.sh start 包着的门禁没带声明拒」「/usr/bin/time 包着的 cargo test 没带声明拒」「驱动脚本里一行缺声明拒」「外面带声明、驱动脚本里一行缺声明拒」「驱动脚本再起驱动脚本、里面缺声明拒」「直接执行的驱动脚本缺声明拒」「detached-run.sh start 起驱动脚本、里面缺声明拒」「同一条命令里 heredoc 写出的驱动脚本缺声明拒」「入口：没带声明拒（退出码 2）」「入口：拒绝说明写跑的是什么、照抄模板与只跑红的写法」「入口：拒绝说明附上次红的清单」「入口：没有上次红的记录时照写」「入口：被拒的记一行（session_id、范围声明被拒、outcome 被拒）」
#   segments-unchecked  不核四段齐不齐、值空不空、写没写两次、种类认不认得
#       红：「声明缺段：没写为什么拒」「声明值空：范围一段是空的拒」「声明写两次：种类写两次拒」「声明种类认不出拒」
#   kind-unchecked  不核声明的种类与命令的形状
#       红：「门禁 --item 声明成 whole：种类对不上拒」「门禁 --quick 声明成 items：种类对不上拒」「merge-and-verify 声明成 items：种类对不上拒」「cargo test 整包声明成 target：种类对不上拒」
#   count-unchecked  不核范围一段的数（items、target）与整轮整包（whole）
#       红：「门禁 --check 两格写成 1 格：范围数对不上拒」「whole 范围没写整轮或整包拒」「cargo test 两个目标写成 3 个：范围数对不上拒」「cargo test --exact 两个测试写成 1 个：范围数对不上拒」
#   reason-unchecked  不核为什么一段的字数
#       红：「gate.sh 为什么只写验证拒」「为什么太短拒」「为什么只写要跑验证拒」
#   subagent-whole-allowed  子 agent 的 whole 带对的声明也放
#       红：「子 agent 起崩溃点重放剪枝跑、带对的声明：whole 拒」「子 agent 起整道门禁、带对的声明：whole 拒」「入口：子 agent whole 拒、说明写交回主 agent」「入口：命令开头已有声明时模板只留一份」
#   scripts-unread  命令位置上执行的驱动脚本不读进去
#       红：「驱动脚本里一行缺声明拒」「外面带声明、驱动脚本里一行缺声明拒」「驱动脚本再起驱动脚本、里面缺声明拒」「直接执行的驱动脚本缺声明拒」「detached-run.sh start 起驱动脚本、里面缺声明拒」「同一条命令里 heredoc 写出的驱动脚本缺声明拒」
#   wrappers-opaque  flock、/usr/bin/time、detached-run.sh start、gpu-quiesce-run.sh 不剥，里面那条不判
#       红：「flock 包着的门禁没带声明拒」「gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒」「detached-run.sh start 包着的门禁没带声明拒」「/usr/bin/time 包着的 cargo test 没带声明拒」「detached-run.sh start 起驱动脚本、里面缺声明拒」
#   allowed-unrecorded  放行的不记进检出记录
#       红：「入口：放行的记进检出记录（四段与命令）」
#   last-runs-unread  拒绝说明不读上次红的清单
#       红：「入口：拒绝说明附上次红的清单」
#   query-forms-declared  只查清单、自证、只编不跑、只列用例的写法也要声明
#       红：「门禁 --list 不要声明」「门禁 --selftest 不要声明」「崩溃点重放 --list-items 不要声明」「replay.sh --admission-only 不要声明」「cargo test --no-run 不要声明」「cargo test -- --list 不要声明」
#   tooling-scripts-read  仓里的工具脚本（scripts/、.claude/ 下）也读进去
#       红：「仓里的工具脚本不读：放行」
#   cargo-targets-as-whole  cargo test 点名目标的也当整包
#       红：「cargo test 两个目标、带对的声明：放」「cargo test --lib --exact 两个测试、带对的声明：放」「子 agent cargo test 点名目标、带对的声明：放」「capped.sh 外面带声明放」「驱动脚本里每一行都带声明：放」
#   template-keeps-declaration  照抄模板时不摘命令开头已有的那份声明（照抄出来两份，bash 取后一份）
#       红：「入口：命令开头已有声明时模板只留一份」
#   开关与红的格以 lib_run_scope.py 的 BREAK_SWITCH_CELLS 为准，开关轮核每个开关名都写在这一段里。
# 退役判据：各道入口都认了范围参数、主 agent 连续四周对着检出记录没挑出不合适的范围，或要声明的那几道入口都不在了，删掉这个钩子与它的注册。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JUDGE="$HOOK_DIR/lib_run_scope.py"
if [[ ! -f "$JUDGE" ]]; then
  echo "  ! run-scope-guard.sh 读不到判法 $JUDGE，这条命令照常执行（范围声明没判）" >&2
  if [[ "${1:-}" == "--selftest" ]]; then
    echo "  ✗ 自检：判法文件 $JUDGE 不在" >&2
    echo "  → 怎么办：把 lib_run_scope.py 放回 .claude/hooks/，再跑 bash .claude/hooks/run-scope-guard.sh --selftest" >&2
    exit 1
  fi
  exit 0
fi
exec python3 "$JUDGE" "$HOOK_DIR" "$@"
