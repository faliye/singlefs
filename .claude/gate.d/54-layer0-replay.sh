#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 research/scripts/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判（用户 2026-09-26 定：项目更严）
# gate-stage: 层 0 崩溃点重放与登记的崩溃枚举用例（整轮门禁与提交时跑快档：checker 档包（crates/singlefs-checker-tier，D13 已定项 15）不标 ignored 的用例，再逐条核 stage-inputs.tsv 里 crash-case: 那几条用例各自那一格全绿标记与它这批输入的指纹相等，不作数的报「本次未跑」、不判红；全量由用户要求或夜间在 HEAD + 暂存区的 worktree 里跑 --full，逐条用例照复用判定跑：那一格全绿标记在就复用，不在才在 release 下跑它、判绿写那一格；两条流的层 0 全量带断点续跑，新池新建文件与 E142 产物逐字比对、里程碑「覆盖写、释放、回退与复用」固定脚本到 E 与用例的闭式比对由用例自己断言；只做过 mkfs 的池可写挂载再发第一个文件版本那条流与新池新建文件逐项相同，由 cargo test 里的快用例钉住，不另枚举）
# gate-category: checker-tier 类
# gate-similar: code-source-discipline.sh 与 harness-model-differential-and-scenarios.sh 也调 research/scripts/crash-case-check.py，但调的是 file-names、one-scenario 两样，分归代码类与 harness 类；这里开跑前只调 placement、modules 两样，判的是崩溃枚举用例与 checker 档测试文件，红了就不起 cargo
# gate-covers: 崩溃点重放
# gate-cell: layer0-replay 快档跑 checker 档包不标 ignored 的用例、逐条核崩溃枚举用例的全绿标记；--full 逐条照复用判定跑
#
# 格结构只补了最小的一截（.claude/rules/verification.md「门禁的结构」；这一道的重设计归会话「里程碑3 放量 GPU加速」）：只有一格，
#   --list                 打格名表那一行（格名、制表符、判什么），不起 cargo、不起 python
#   --check layer0-replay  与不给一样；格名写错或缺格名退 2、列出可用的格名
#   --list-items           按崩溃枚举用例点名：逐行打被判那棵树的 .claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的那几条与它们的 test=，不起 cargo、不起 python
#   --item <用例名>        只看点名的崩溃枚举用例（crash-case: 可写可不写，给几次取并集，没登记退 2）：快档照跑 checker 档包、只核这几条的标记；--full 只跑这几条
# 不改用共用库 lib/stage-cells.sh：它的汇总把 0、77 之外的退出码并成 1，而样本档判绿退 3；research/scripts/admission.py --selftest 的「54 号」那几格
# 把这一份拷进临时仓的 .claude/stage-under-test/（没有 lib/）跑流程，source 不到共用库。解析参数的判法与共用库同一套（写错退 2、列出格名）。
#
# 分两档（用户 2026-09-19 定，原话「每次主 agent 执行完任务后统一执行」，records/2026-09-19-里程碑二遗留收拢.md「五之二」第 8 问；
# 用户 2026-09-26 定逐条用例复用，原话「下次肯定要接入提交时崩溃验证员， 并且以后跑也不能全量这么跑，改了只跑改了的部分。」，
# records/2026-09-24-里程碑二收尾调度.md 第三节「崩溃枚举的跑法」那一行；
# 用户 2026-09-27 定验证代码分两档、提交时默认只跑快档，原话「checker可以自己跑，但是默认只有在提交时候才跑」，records/2026-09-27-验证两档拆分.md）：
#
#   bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full [--start-over] <worktree>
#     主 agent 暂存之后（提交时由崩溃验证员），在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
#     逐条崩溃枚举用例（.claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的那几行，照登记表的次序）：先算这条用例这批输入的指纹
#     （research/scripts/admission.py crash-case-manifest：登记路径下的文件减去用例读不到的文件，加准入模块 admission.py 里崩溃枚举用例的判法摘要、
#     工具链、构建环境与这条用例的登记行；这一份 54 号不进指纹）；那一格全绿标记在、作数就复用，不跑；不在才照 admission.py crash-case-command
#     交出的命令与环境跑（`cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数> --nocapture`，续跑的变量与线程数也由它定，
#     那一段在判法摘要里），按登记行第三列判日志（crash-case-judge），开跑与跑完各算一次指纹，相同才写那一格（crash-case-record）。
#     全绿标记在 git common-dir：`singlefs-crash-case-green.<用例名>.<输入指纹>`，不进工作树，各 worktree 读写同一组；别的格不动。
#     一条判红删它这批输入那一格（先绿后红，前一趟那一格不再作数），接着跑下一条；「跑的过程中输入变了」那一支判红不删：它说不出开跑那一批的好坏。
#     两趟 --full 同时跑同一条用例、同一个指纹时这里不加锁：后跑完的那一趟判红会删掉先跑完的那一趟刚写的绿标记，只会假红、不会假绿
#     （崩溃验证员的定义里「另有 --full 在跑时不起」挡着这一种）。
#     断点续跑（crash-case-command 设）：跑用例时设 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=<common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>
#     （不随 worktree 删掉）、SINGLEFS_LAYER0_INPUT_FINGERPRINT=<这条用例的输入指纹>；--start-over 设 SINGLEFS_LAYER0_START_OVER=1（丢掉进度文件、从头跑），
#     不带它时从调用方的环境里清掉这个变量。续跑的判法（片方案、校验和、观察者计数、判红删进度文件）在 crates/singlefs-checker-tier/src/layer0_progress.rs。
#     双机分片（里程碑三第六项，用户 2026-09-27 定默认不分片）：本地配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，
#     模板是仓根 multi-host.env.example）在、research/scripts/layer0-shard-configuration-check.sh 判得过（键齐、第二台 ssh 连得上、它上面有 cargo）
#     才开；开着时登记了 shard=across-machines 的用例（admission.py crash-case-shardable）交给 research/scripts/layer0-shard-run.sh --merged-log
#     （本机 0/2、第二台 1/2、本机 merge/2；驱动脚本与配置判法按内容进这几条用例的指纹），它交回的 merge 那一趟日志照单机的判法判、写同一格标记；
#     别的用例、配置不在或判不过时，照 crash-case-command 单机跑。开没开、为什么，开跑时打一行。
#     发现日志（用户 2026-09-27 定「崩溃放量日志要全量与发现双份，不然读不过来」，records/2026-09-24-里程碑二收尾调度.md「层 0 放量的发现日志」那一行；
#     行格式以 crates/singlefs-checker-tier/src/crash.rs 的发现日志为准）：每趟 --full 的全量日志与发现日志放 <common-dir>/singlefs-layer0-logs/<开跑时刻>-<pid>/，
#     不随 worktree 删、跑完不删；一条用例的全量日志是 log.<用例名>，发现日志是它同名加 .findings.tsv。单机跑时在 crash-case-command 交的命令外面
#     设 SINGLEFS_LAYER0_FINDINGS_FILE=<发现日志>；双机分片时驱动脚本照同一个命名（<它交回的 merge 日志>.findings.tsv）给 merge 那一趟，两片各自的
#     发现日志在那一片的日志旁边（驱动脚本的输出里打出三份路径）。跑完逐节读发现日志（一节一趟枚举；这一趟的目录是新建的，里面的节都是这一趟的）：
#     有 layer0_findings_summary 的节打一行 signatures / red_states / states 与发现表（每签名一行：号、pass、violated、segment、publish、states、
#     sample_states；超过 30 个签名只打前 30 行加一行「其余 N 个见 <路径>」）；没有 summary 的节报「这一趟没跑完」判红；red_states 不是 0 判红
#     （用例自己也会红，这是第二道）；定稿行数与 signatures= 对不上判红。发现日志不在或是空的：全量日志里有 LAYER0_FINDINGS 行（枚举跑完了却没写）判红，
#     一行都没有（这条用例不走读这个变量的枚举入口）打一行说明、不判红。全量日志不转进阶段输出，只给路径（LAYER0_PROGRESS 照旧边跑边转，
#     cargo 退非 0 时照旧打它的尾部）；全绿标记不带发现表。快档不设、不读发现日志，还从起 cargo 的环境里清掉调用方的这个变量。
#     这一份 54 号不进指纹：改它（出路句、快档、次序）不废旧标记。本机核数、线程数不经它转：crash-case-judge 与 crash-case-record 自己现取
#     （与 crash-case-command 同一个算法、同一份环境），crash-case-command 交的前三个词这里只拿来打开跑那一行。它里面还定着结论的只剩流程的次序
#     （开跑与跑完各算一次指纹、先判日志再写标记、判红删那一格）与两处第二道判红（cargo 退非 0、发现日志那几条）——这两处的第一道
#     （test result 恰好 1 passed、登记的计数行）在准入模块的判法摘要里；由 admission.py --selftest 的「54 号」那几格核；改它时快档照样核标记
#     （范围那一问不摘掉它）。
#   bash .claude/gate.d/54-layer0-replay.sh [项目根]           整轮门禁的默认（gate.sh 只传项目根）
#     快档先核登记的每一条路径 git 至少列得出一个文件（git ls-files -co --exclude-standard -- <那一条>），有一条列不出判红，之后才问复用与改动范围。
#     checker 档包在 release 下只跑不标 ignored 的用例（cargo test --release -p singlefs-checker-tier --lib --tests：库与集成测试，崩溃枚举用例都住那个包的 tests/；装置二进制 src/bin/ 的内联单测不在快档里，归它们的变异表（checker-tier-crates-mutation-replay）与实验复跑），一条都没通过判红；
#     再逐条崩溃枚举用例算它这批输入的指纹、核那一格
#     （admission.py crash-case-marker-check：在、记的指纹与用例相同、test result 是 1 passed、登记的计数行各恰好一行、要 exhaustive=true 的带着），
#     全部作数成功句逐条原样带出那一格的计数行与时刻；有不作数的逐条列原因（没有那一格时比最近写的一格与这一次的清单）、每条往 GATE_NOT_RUN_FILE 报一行「本次未跑」，
#     不判红、退 0（全量默认不在提交时跑，D13 已定项 15；报过本次未跑的这一轮不算覆盖崩溃点重放），出路是跑 --full。
#     按整批输入分格的旧标记（singlefs-layer0-full-green.*）不再认。
#
# 输入：整道阶段的复用判定与改动范围按 stage-inputs.tsv 里本阶段那一行（唯一登记位）；每条崩溃枚举用例的输入按它自己那一行，
# 由 admission.py 按内容算（主工作区跑的与 `--staged` 临时 worktree 里的同一份内容算出同一个数）。
# 主工作区里跑的 --full 读的是工作区（连同别的会话没暂存的改动），罩不到这一批暂存内容；所以 --full 在 HEAD + 暂存区的 worktree 里跑。
#
# 全量（里程碑「新池新建文件」步 7）：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑恢复与 oracle、池级 checker、记录核对器，
# 计数由用例钉死。全量要跑很久，平时 `cargo test` 里那几条标 ignored；--full 在 release 下带 --include-ignored --exact 逐条跑。
# 用例的判别力在用例自己：对着产物与闭式的计数断言，oracle 的判别力由同文件的靶向阳性对照证明。
#
# 多线程（增补 2 收口表第 41 行；2026-09-18 用户定：测试与崩溃检测优先多线程）：crash.rs 按状态序号区间切片、多线程跑，
# 线程数由 SINGLEFS_LAYER0_THREADS 传进去——调用方没设就取本机核数（nproc），取法在 admission.py 的 crash_case_worker_threads。
# 每跑完一片，用例打一行 `LAYER0_PROGRESS`，这里边跑边转到本阶段的输出里。
# 全绿标记里线程分两格记：configured_worker_threads= 是配的（SINGLEFS_LAYER0_THREADS 与它怎么来的、本机几核），started_worker_threads= 是
# 登记了 threads= 的用例从 LAYER0_PARALLEL_FINISHED 读到的起了几个；没登记 threads= 的记「读不到」。
# 登记了 threads=<前缀> 的用例：按那一行计数的状态数找 `LAYER0_PARALLEL_FINISHED`，这一趟真跑了至少两片、却只起了 1 个工作线程、本机多于 1 核、
# SINGLEFS_LAYER0_THREADS 没显式设成 1，判红（多半是线程数没传进去）；全部片从进度文件读回（起 0 个线程）、只剩 1 片要跑（最多起 1 个）都不判红。
#
# 判别力：日志与标记怎么判、输入指纹怎么算，由 research/scripts/admission.py --selftest 的「崩溃枚举用例」那几格拿合成日志与临时仓核；
# 这一份脚本的流程（逐条复用、只重跑输入变了的那一条、续跑的三个环境变量、--start-over、快档缺一格判红、跑的过程中输入变了不写标记）
# 由同一份自证的「54 号」那几格核：把这一份拷进临时仓（不放在 .claude/gate.d/ 下）、拿打合成日志的假 cargo 跑。
# 发现日志那一段的判别力样本 fixtures/54-layer0-replay.sh/{red,green}（共享门禁的 .claude/singlefs-ai-sop/scripts/stage-selftest.sh 跑，setup.sh 在临时目录里 git init）：
# 样本根上放一个 .layer0-sample-tools/ 目录，本阶段就进样本档——把它放到 PATH 最前面（里面是假 cargo 与假 rustc：照 cases/<测试目标>.log 打日志，
# 设了 SINGLEFS_LAYER0_FINDINGS_FILE 时把 cases/<测试目标>.findings.tsv 追加进去），不管带没带 --full 都走 --full 那一路；判绿退 3（不退 0，
# 样本档不算真跑过），判红照常退 1。green：一条用例的发现日志一节、0 个签名，一条用例不走发现日志；red：两个签名 red_states=3、没有 summary、32 个签名三条用例。
# 弄坏开关（只给证红用）GATE_LAYER0_BREAK=<项>：no-findings-file 不设 SINGLEFS_LAYER0_FINDINGS_FILE、ignore-unfinished 没有 summary 的节当跑完了、
# ignore-red-states 不看 red_states、no-truncation 签名全打不截；各自打开时 red 或 green 样本判错。
# 开跑前的静态判据（placement、modules 两样）：样本 fixtures/54-layer0-replay.sh/static-check-red 放一条标了 #[ignore]、调 enumerate_layer0 却没登记 crash-case: 的
# checker 档用例与一份第一行没声明模块的 checker 档测试文件，判红、不起 cargo；red、green 两份的测试文件第一行都声明了「无」，静态判据判绿、往下走。
# 弄坏开关 GATE_LAYER0_BREAK=static-check-skipped 跳过这一段，static-check-red 报不出那两处，样本判错；
# 判法那一份的开关 CRASH_CASE_CHECK_BREAK=rows-ignored 让没登记那一条报不出来，样本同样判错。
set -uo pipefail
# 参数：`--full`、`--start-over`、`--list`、`--check <格名>`、`--list-items`、`--item <用例名>` 与项目根，顺序不限；gate.sh 只传项目根，于是整轮门禁走快档。
layer0_tier="quick"
layer0_start_over=0
root_argument=""
layer0_list_cells=0
layer0_list_items=0
layer0_named_cases=()
# 格名表（与文件头 `# gate-cell:` 那一行逐字相同；.claude/rules/verification.md「门禁的结构」）
LAYER0_CELL_NAME="layer0-replay"
LAYER0_CELL_TITLE="快档跑 checker 档包不标 ignored 的用例、逐条核崩溃枚举用例的全绿标记；--full 逐条照复用判定跑"
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
layer0_check_cells() { # <逗号隔开的格名>：只认 layer0-replay；写错、空着退 2 并列出可用的格名
  local requested_cell requested_cells=()
  IFS=, read -r -a requested_cells <<< "$1"
  ((${#requested_cells[@]})) || requested_cells=("")
  for requested_cell in "${requested_cells[@]}"; do
    if [[ "$requested_cell" != "$LAYER0_CELL_NAME" ]]; then
      echo "  ✗ 54-layer0-replay.sh：--check 里有没登记的格名：「$requested_cell」"
      echo "     → 怎么办：这一道只有一格，格名照下面写（也可以 bash .claude/gate.d/54-layer0-replay.sh --list 看）；按崩溃枚举用例点名用 --item <用例名>："
      printf '       %s  %s\n' "$LAYER0_CELL_NAME" "$LAYER0_CELL_TITLE"
      exit 2
    fi
  done
}
while (($#)); do
  stage_argument="$1"
  case "$stage_argument" in
    --full) layer0_tier="full" ;;
    --start-over) layer0_start_over=1 ;;
    --list) layer0_list_cells=1 ;;
    --list-items) layer0_list_items=1 ;;
    --check|--item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then
        echo "  ✗ $stage_argument 后面缺名字"
        echo "     → 怎么办：写成 --check $LAYER0_CELL_NAME，或 --item <崩溃枚举用例名>（crash-case: 可写可不写，几条就给几次；可点的用 --list-items 看）。"
        exit 2
      fi
      if [[ "$stage_argument" == --check ]]; then layer0_check_cells "$2"; else layer0_named_cases+=("crash-case:${2#crash-case:}"); fi
      shift ;;
    --check=*) layer0_check_cells "${stage_argument#--check=}" ;;
    --item=*) layer0_named_value="${stage_argument#--item=}"; layer0_named_cases+=("crash-case:${layer0_named_value#crash-case:}") ;;
    -*)
      echo "  ✗ 认不出的参数：$stage_argument"
      echo "     → 怎么办：只认 --full、--start-over、--list、--check $LAYER0_CELL_NAME、--list-items、--item <用例名> 与项目根，顺序不限：bash .claude/gate.d/54-layer0-replay.sh [--full [--start-over]] [--item <用例名>] [项目根]"
      exit 2 ;;
    *) root_argument="$stage_argument" ;;
  esac
  shift
done
if ((layer0_list_cells)); then
  printf '%s\t%s\n' "$LAYER0_CELL_NAME" "$LAYER0_CELL_TITLE"
  exit 0
fi
if ((layer0_list_items)); then
  # 可点名的崩溃枚举用例：被判那棵树的登记表里键是 crash-case: 的行，照登记表的次序；不起 cargo、不起 python
  layer0_list_table="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}/.claude/gate.d/stage-inputs.tsv"
  if [[ ! -f "$layer0_list_table" ]]; then
    echo "  ✗ 读不到 $layer0_list_table：列不出可点名的崩溃枚举用例"
    echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"
    exit 1
  fi
  awk -F'\t' '$1 ~ /^crash-case:/ { test = ""; n = split($3, tokens, " "); for (i = 1; i <= n; i++) if (tokens[i] ~ /^test=/) test = substr(tokens[i], 6); printf "%s\t%s\n", $1, test }' "$layer0_list_table"
  exit 0
fi
if [[ "$layer0_start_over" == 1 && "$layer0_tier" != full ]]; then
  echo "  ✗ --start-over 只跟 --full 一起用：快档不跑全量，没有进度文件可丢"
  echo "     → 怎么办：要丢掉进度文件、从头跑全量，写成 bash .claude/gate.d/54-layer0-replay.sh --full --start-over <根>。"
  exit 2
fi
ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
# 跑的这一份 54 号进每条用例的输入清单；相对的 $0 在 cd 之后会指错，所以在 cd 之前取成绝对路径
layer0_stage_script_path="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
# 门禁与实验共用的准入模块：读登记表、算输入清单、判前提、判日志与读写全绿标记都经它（research/scripts/admission.py，文件头写全了各子命令）
layer0_stage_repository="$(cd "$(dirname "$0")/../.." && pwd)"
layer0_admission_module="$layer0_stage_repository/research/scripts/admission.py"
cd "$ROOT" 2>/dev/null || exit 2
# 样本档：样本根上有 .layer0-sample-tools/ 时，假 cargo 与假 rustc 放到 PATH 最前面、走 --full 那一路，判绿退 3（写法见文件头「判别力」那一段）
layer0_sample_mode=0
if [[ -d "$PWD/.layer0-sample-tools" ]]; then
  layer0_sample_mode=1
  layer0_tier="full"
  PATH="$PWD/.layer0-sample-tools:$PATH"
  export PATH
  echo "  ! 样本档：$PWD/.layer0-sample-tools 在，起用例的是那里面的假 cargo，走 --full 那一路；这一趟不算真跑过层 0"
fi
# 弄坏开关（只给证红用，文件头「判别力」那一段）
layer0_break="${GATE_LAYER0_BREAK:-}"

# 开跑前的静态判据（原是测试粒度那一道的两样；判法只在 research/scripts/crash-case-check.py，静态、几秒跑完）：
# placement 崩溃枚举用例住 checker 档包、标了 #[ignore] 的都登记成 crash-case:（没登记的 --full 不跑，谁都不跑）；
# modules checker 档每个测试文件第一行声明它测哪几个模块。快档、--full、样本档都先判这一次，在复用判定之前：红了判红、不起 cargo。
# 没有 crates/ 时不判（后面没有 checker 档包那一句退 77）；两样都无对象可判（crash-case-check 退 77）照往下走；
# 判法脚本不在（admission.py --selftest 把这一份拷进不带 research/scripts/ 的临时仓跑）往 $GATE_NOT_RUN_FILE 报一行没跑、往下走，不判绿也不判红。
layer0_static_judge="$layer0_stage_repository/research/scripts/crash-case-check.py"
if [[ -d crates && "$layer0_break" != static-check-skipped && ! -f "$layer0_static_judge" ]]; then
  echo "  ! 开跑前的静态判据没跑：判法脚本 $layer0_static_judge 不在（这一份 54 号被拷到了不带 research/scripts/ 的树里），往 GATE_NOT_RUN_FILE 报一行"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    printf '%s\n' "54-layer0-replay.sh 开跑前的静态判据（placement、modules）没跑：$layer0_static_judge 不在" >> "$GATE_NOT_RUN_FILE"
  fi
elif [[ -d crates && "$layer0_break" != static-check-skipped ]]; then
  echo "── 开跑前的静态判据：崩溃枚举用例住哪、登记没有，checker 档测试文件声明模块"
  if python3 "$layer0_static_judge" --only placement,modules "$ROOT"; then layer0_static_exit=0; else layer0_static_exit=$?; fi
  if [[ "$layer0_static_exit" != 0 && "$layer0_static_exit" != 77 ]]; then
    echo "  ✗ 开跑前的静态判据：crash-case-check --only placement,modules 退 $layer0_static_exit（逐处列在上面），这一趟不起 cargo"
    echo "     → 怎么办：照上面那一句出路改（崩溃枚举用例挪进 crates/singlefs-checker-tier/tests/、全量那条登记进 .claude/gate.d/stage-inputs.tsv 的 crash-case: 行；"
    echo "                checker 档测试文件第一行写「//! checker 档模块：…」）；改完先单跑 python3 research/scripts/crash-case-check.py --only placement,modules 到它退 0，再跑这一道。"
    exit 1
  fi
fi

# 这一道读的路径：`.claude/gate.d/stage-inputs.tsv` 里登记给本阶段的那几条（唯一登记位），经准入模块的 paths 读。
# 快档的复用判定与改动范围按它算；每条崩溃枚举用例的输入另按它自己那一行算。
layer0_stage_file_name="$(basename "$0")"
layer0_input_table="$ROOT/.claude/gate.d/stage-inputs.tsv"
layer0_registered_input_paths=()
if layer0_registered_paths_text="$(python3 "$layer0_admission_module" paths "$ROOT" "$layer0_stage_file_name" 2>/dev/null)" \
   && [[ -n "$layer0_registered_paths_text" ]]; then
  mapfile -t layer0_registered_input_paths <<< "$layer0_registered_paths_text"
fi
if (( ${#layer0_registered_input_paths[@]} == 0 )); then
  echo "  ✗ $layer0_input_table 里没有 $layer0_stage_file_name 这一行（或读不到这份表）：判不出这一道读哪些路径"
  echo "     → 怎么办：在 .claude/gate.d/stage-inputs.tsv 里给本阶段登记它读的路径（制表符分隔，照别的行写）。"
  exit 1
fi
layer0_input_paths_text="${layer0_registered_input_paths[*]}"

# 快档先问两件事，任一答「可跳过」就退 77（本次未跑），不退 0——`exit 0` 的跳过在汇总里与「判过了」
# 一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。--full 是收尾时点名要跑的，这两问都不问（逐条用例的复用另判）。
# 一问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
# 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
# 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
# 二问这次改动碰没碰登记给本阶段的那几条路径，再加登记表本身、跑的这一份 54 号与准入模块：登记行与准入模块的判法摘要进每条崩溃枚举用例的指纹，
# 54 号定着流程的次序，只改它们的改动（新登记一条用例、改一条的第三列、改判法、改流程）同样要核标记，不能在这一问被摘掉；
# 核下来标记照样作数（只改了准入模块判法之外的部分、只改了 54 号）就判绿。
# 这是 C8（范围判定）的粗粒度前身：它只摘得掉「零行输入的改动」，摘不出别的，C8 照旧欠着。
# 两问之前先核登记的每一条路径 git 至少列得出一个文件：写错的路径两问都拿它答「没碰」，这一道就一直退 77、一次都不跑。
if [[ "$layer0_tier" == quick ]]; then
  layer0_unlisted_input_paths=()
  for registered_input_path in "${layer0_registered_input_paths[@]}"; do
    registered_input_listing="$(git -c core.quotepath=false -C "$ROOT" ls-files -co --exclude-standard -- "$registered_input_path" 2>/dev/null)"
    if [[ -z "$registered_input_listing" ]]; then layer0_unlisted_input_paths+=("$registered_input_path"); fi
  done
  if (( ${#layer0_unlisted_input_paths[@]} > 0 )); then
    echo "  ✗ stage-inputs.tsv 登记给 $layer0_stage_file_name 的路径里，有 ${#layer0_unlisted_input_paths[@]} 条 git 一个文件都列不出来：${layer0_unlisted_input_paths[*]}（登记的是 ${layer0_input_paths_text}）"
    echo "     → 怎么办：多半是 .claude/gate.d/stage-inputs.tsv 里那一条写错了（拼错、目录少了斜杠、指到不存在的路径）；"
    echo "                在项目根跑 git ls-files -co --exclude-standard -- <那一条>，改到列得出文件为止。全部列不出时先看这里是不是 git 工作树。"
    exit 1
  fi
  source "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
  stage_run_or_skip "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")"
  layer0_judge_paths=(.claude/gate.d/stage-inputs.tsv "$(realpath -m --relative-to="$ROOT" "$layer0_stage_script_path")" "$(realpath -m --relative-to="$ROOT" "$layer0_admission_module")")
  stage_run_or_skip "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "这次改动没碰它判的东西" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${layer0_input_paths_text}，加 ${layer0_judge_paths[*]}，判据见 research/scripts/change-touches-crates.sh。" \
    -- "$ROOT" "${layer0_registered_input_paths[@]}" "${layer0_judge_paths[@]}"
fi
[[ -f Cargo.toml && -d crates/singlefs-checker-tier ]] || { echo "  ! 没有 crates/singlefs-checker-tier（checker 档包），本阶段跳过"; exit 77; }
# 前提（cargo、rustc）登记在 stage-inputs.tsv 本阶段那一行第三列，经准入模块判，没齐判红（不退 77）
python3 "$layer0_admission_module" gate-preconditions "$layer0_stage_repository" "$layer0_stage_file_name" || exit 1

# 全绿标记与续跑的进度文件放 git 的 common-dir：不进工作树，主工作树与 `gate.sh --staged` 的临时 worktree 读写的是同一份。
if ! git_common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
  echo "  ✗ $ROOT 不是 git 工作树（取不到 git common-dir）：崩溃枚举用例的全绿标记与续跑的进度文件没处放、也没处读"
  echo "     → 怎么办：在这个项目的 git 仓里跑，或把仓库根作为参数传进来；标记在 \$(git rev-parse --git-common-dir)/singlefs-crash-case-green.<用例名>.<输入指纹>。"
  exit 1
fi
layer0_progress_root="$git_common_directory/singlefs-layer0-progress"
layer0_scratch_directory="$(mktemp -d)"
trap 'rm -rf -- "${layer0_scratch_directory:?}"' EXIT

# 登记的崩溃枚举用例，一条一行「键、包、测试目标、用例函数」（制表符分隔），照登记表的次序；登记有错判红。
if ! crash_case_listing="$(python3 "$layer0_admission_module" crash-cases "$ROOT")"; then
  printf '%s\n' "$crash_case_listing" | sed 's/^/       /'
  echo "  ✗ .claude/gate.d/stage-inputs.tsv 里 crash-case: 那几行登记有错（上面逐条列出）：判不出要跑、要核哪几条崩溃枚举用例"
  echo "     → 怎么办：照 research/scripts/admission.py 文件头「崩溃枚举用例行」改那几行（test=<包>:<测试目标>:<用例函数> 恰好一条，用例函数在测试目标里找得到）；"
  echo "                单跑 python3 research/scripts/admission.py crash-cases <项目根>，改到它退 0 为止。"
  exit 1
fi
crash_case_rows=()
if [[ -n "$crash_case_listing" ]]; then mapfile -t crash_case_rows <<< "$crash_case_listing"; fi
if (( ${#crash_case_rows[@]} == 0 )); then
  echo "  ✗ .claude/gate.d/stage-inputs.tsv 里一条崩溃枚举用例（键是 crash-case: 的行）都没登记：层 0 全量没有东西可跑、可核"
  echo "     → 怎么办：两条流的层 0 全量至少各登记一行，写法见 research/scripts/admission.py 文件头「崩溃枚举用例行」。"
  exit 1
fi
# --item 点名的崩溃枚举用例：快档只核、--full 只跑这几条（照登记表的次序）；点名的不在登记表里退 2
if (( ${#layer0_named_cases[@]} > 0 )); then
  layer0_kept_rows=()
  layer0_unknown_cases=()
  for layer0_named_case in "${layer0_named_cases[@]}"; do
    layer0_found=0
    for crash_case_row in "${crash_case_rows[@]}"; do [[ "${crash_case_row%%$'\t'*}" == "$layer0_named_case" ]] && layer0_found=1; done
    (( layer0_found )) || layer0_unknown_cases+=("$layer0_named_case")
  done
  if (( ${#layer0_unknown_cases[@]} > 0 )); then
    echo "  ✗ --item 点名的崩溃枚举用例没登记：${layer0_unknown_cases[*]}"
    echo "     → 怎么办：用例名照 bash .claude/gate.d/54-layer0-replay.sh --list-items 列的写（登记表里有 ${#crash_case_rows[@]} 条）。"
    exit 2
  fi
  for crash_case_row in "${crash_case_rows[@]}"; do
    for layer0_named_case in "${layer0_named_cases[@]}"; do
      if [[ "${crash_case_row%%$'\t'*}" == "$layer0_named_case" ]]; then layer0_kept_rows+=("$crash_case_row"); break; fi
    done
  done
  echo "  · 只看 --item 点名的 ${#layer0_kept_rows[@]} 条崩溃枚举用例（登记表里共 ${#crash_case_rows[@]} 条，别的这一趟不跑、不核）"
  crash_case_rows=("${layer0_kept_rows[@]}")
fi

# write_crash_case_manifest <键> <清单文件>：这条用例这批输入的逐文件清单写进清单文件，指纹、文件数、减去的文件数放进
# case_fingerprint / case_file_count / case_excluded_count。算不出返回 1，原因放进 case_manifest_problem。
write_crash_case_manifest() {
  local manifest_summary
  if ! manifest_summary="$(python3 "$layer0_admission_module" crash-case-manifest "$ROOT" "$1" "$2" \
      --judging-digest --toolchain --build-environment)"; then
    case_manifest_problem="$manifest_summary"
    return 1
  fi
  read -r case_fingerprint case_file_count case_excluded_count <<< "$manifest_summary"
  if [[ ! "$case_fingerprint" =~ ^[0-9a-f]{64}$ || ! "$case_file_count" =~ ^[1-9][0-9]*$ || ! "$case_excluded_count" =~ ^[0-9]+$ ]]; then
    case_manifest_problem="准入模块打的不是「<指纹> <文件数> <减去的文件数>」：$manifest_summary"
    return 1
  fi
  return 0
}

# delete_crash_case_marker <键> <指纹>：这条用例这批输入那一格全绿标记删掉（判红时；前一趟那一格不再作数）。
delete_crash_case_marker() {
  local marker_path
  marker_path="$(python3 "$layer0_admission_module" crash-case-marker-path "$ROOT" "$1" "$2")" || return 0
  if [[ -n "$marker_path" ]]; then rm -f -- "${marker_path:?}"; fi
  return 0
}

# run_checker_package_tests <日志>：快档跑 checker 档包不标 ignored 的全部用例（崩溃枚举用例住那个包的 tests/，D13 已定项 15）；cargo 的整段输出进日志。
# 快档不写发现日志：调用方环境里的 SINGLEFS_LAYER0_FINDINGS_FILE 清掉，不漏给快用例。
run_checker_package_tests() {
  env -u SINGLEFS_LAYER0_FINDINGS_FILE cargo test --release -p singlefs-checker-tier --lib --tests -- --nocapture 2>&1 \
    | tee "$1" \
    | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
    | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# read_crash_case_command <键> <输入指纹>：准入模块 crash-case-command 交出的这一条的命令与环境（以 NUL 分隔）读进 case_machine_cores、
# case_threads、case_threads_origin（这三个只打进开跑那一行，不转给 crash-case-judge / record）、case_progress_directory 与数组 case_command。
# 交不出、写法不对返回 1，原因放进 case_command_problem。
read_crash_case_command() {
  local command_file="$layer0_scratch_directory/command.${1#crash-case:}" command_output
  local -a start_over_option=()
  if [[ "$layer0_start_over" == 1 ]]; then start_over_option=(--start-over); fi
  if ! python3 "$layer0_admission_module" crash-case-command "$ROOT" "$1" "$2" "${start_over_option[@]}" > "$command_file"; then
    case_command_problem="$(tr '\0' ' ' < "$command_file")"
    return 1
  fi
  local -a command_words=()
  mapfile -d '' -t command_words < "$command_file"
  if (( ${#command_words[@]} < 5 )) || [[ ! "${command_words[0]}" =~ ^[1-9][0-9]*$ || ! "${command_words[1]}" =~ ^[1-9][0-9]*$ \
      || ! "${command_words[2]}" =~ ^(explicit|default)$ ]]; then
    command_output="$(tr '\0' ' ' < "$command_file")"
    case_command_problem="准入模块交的不是「核数、线程数、explicit|default、进度目录、命令…」：${command_output}"
    return 1
  fi
  case_machine_cores="${command_words[0]}"
  case_threads="${command_words[1]}"
  case_threads_origin="${command_words[2]}"
  case_progress_directory="${command_words[3]}"
  case_command=("${command_words[@]:4}")
  return 0
}

# run_crash_case <日志> <发现日志> <命令的词…>：--full 照准入模块交的命令跑一条崩溃枚举用例，环境里设 SINGLEFS_LAYER0_FINDINGS_FILE=<发现日志>；
# 整段输出进日志，`LAYER0_PROGRESS` 行边跑边转到本阶段的输出（跑全量时这里一直在涨）。
run_crash_case() {
  local log_file="$1"
  local -a findings_setting=(SINGLEFS_LAYER0_FINDINGS_FILE="$2")
  if [[ "$layer0_break" == no-findings-file ]]; then findings_setting=(-u SINGLEFS_LAYER0_FINDINGS_FILE); fi
  shift 2
  env "${findings_setting[@]}" "$@" 2>&1 \
    | tee "$log_file" \
    | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
    | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# run_crash_case_in_two_shards <键> <日志> <输入指纹>：双机分片跑一条（layer0-shard-run.sh --merged-log），merge 那一趟的整段输出进日志；
# 两片各自的 LAYER0_PROGRESS 由驱动脚本边跑边转出来。--start-over 照样交给两片（merge 那一趟驱动脚本自己不带）。
run_crash_case_in_two_shards() {
  local -a start_over_setting=(-u SINGLEFS_LAYER0_START_OVER)
  if [[ "$layer0_start_over" == 1 ]]; then start_over_setting=(SINGLEFS_LAYER0_START_OVER=1); fi
  env "${start_over_setting[@]}" bash "$layer0_shard_driver" --merged-log "$1" "$ROOT" "$3" "$2" 2>&1 | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# report_crash_case_findings <发现日志> <全量日志>：逐节打发现表（行格式与节的写法以 crates/singlefs-checker-tier/src/crash.rs 的发现日志为准：
# 字段制表符分，第一个字段是行的种类，其余 key=value）。每一处不对各打一行「问题：…」，有问题返回 1；发现日志不在又不该有时返回 0。
report_crash_case_findings() {
  local findings_file="$1" log_file="$2" stdout_findings_lines
  stdout_findings_lines="$(grep -c '^LAYER0_FINDINGS ' "$log_file" 2>/dev/null)"
  stdout_findings_lines="${stdout_findings_lines:-0}"
  if [[ ! -s "$findings_file" ]]; then
    if [[ "$stdout_findings_lines" == 0 ]]; then
      echo "    发现日志：没有（$findings_file 不在或是空的，全量日志里也没有 LAYER0_FINDINGS 行：这条用例没走读 SINGLEFS_LAYER0_FINDINGS_FILE 的枚举入口）"
      return 0
    fi
    echo "    发现日志：$findings_file 不在或是空的"
    echo "      问题：全量日志里有 ${stdout_findings_lines} 行 LAYER0_FINDINGS（枚举跑完了 ${stdout_findings_lines} 趟），发现日志却一节都没写：SINGLEFS_LAYER0_FINDINGS_FILE 没传进用例"
    return 1
  fi
  echo "    发现日志：$findings_file（全量日志里 LAYER0_FINDINGS ${stdout_findings_lines} 行）"
  awk -F'\t' -v shown_limit=30 -v findings_path="$findings_file" -v break_item="$layer0_break" '
    function field(name,   position, equals) {
      for (position = 2; position <= NF; position++) {
        equals = index($position, "=")
        if (equals > 0 && substr($position, 1, equals - 1) == name) return substr($position, equals + 1)
      }
      return ""
    }
    function signature_text(tail_name) {
      return "#" field("finding") " pass=" field("pass") " violated=" field("violated") " segment=" field("segment") " publish=" field("publish") " " tail_name
    }
    $1 == "layer0_findings_begin" {
      sections++
      section_name[sections] = "stream=" field("stream") (field("shard") != "" ? " shard=" field("shard") : "") " states=" field("states")
      next
    }
    sections == 0 { lines_before_first_section++; next }
    $1 == "layer0_finding" { final_rows[sections, ++final_count[sections]] = signature_text("states=" field("states") " sample_states=" field("sample_states")); next }
    $1 == "layer0_finding_new" { new_rows[sections, ++new_count[sections]] = signature_text("first_state=" field("first_state")); next }
    $1 == "layer0_finding_threshold" { next }
    $1 == "layer0_findings_summary" {
      summary_count[sections]++
      summary_signatures[sections] = field("signatures"); summary_red_states[sections] = field("red_states"); summary_states[sections] = field("states")
      next
    }
    { unknown_kinds[$1]++ }
    function print_rows(kind, section, count,   row, limit) {
      limit = (break_item == "no-truncation") ? count : shown_limit
      for (row = 1; row <= count && row <= limit; row++) print "        " (kind == "final" ? final_rows[section, row] : new_rows[section, row])
      if (count > limit) print "        其余 " (count - limit) " 个见 " findings_path
    }
    END {
      problems = 0
      if (sections == 0) { print "      问题：一行 layer0_findings_begin 都没有，认不出节"; problems++ }
      if (lines_before_first_section > 0) { print "      问题：第一行 layer0_findings_begin 之前有 " lines_before_first_section " 行"; problems++ }
      for (kind in unknown_kinds) print "      （认不出的行种类 " kind "：" unknown_kinds[kind] " 行，没判）"
      for (section = 1; section <= sections; section++) {
        if (summary_count[section] == 0) {
          if (break_item == "ignore-unfinished") continue
          print "      第 " section " 节（" section_name[section] "）：这一趟没跑完（没有 layer0_findings_summary 行）；死之前找到 " (new_count[section] + 0) " 个签名："
          print_rows("new", section, new_count[section] + 0)
          print "      问题：第 " section " 节这一趟没跑完（没有 layer0_findings_summary 行：还在跑、被杀或 panic 判红）"
          problems++
          continue
        }
        print "      第 " section " 节（" section_name[section] "）：signatures=" summary_signatures[section] " red_states=" summary_red_states[section] " states=" summary_states[section]
        if (final_count[section] + 0 == 0) print "        没有签名"
        print_rows("final", section, final_count[section] + 0)
        if (summary_count[section] > 1) { print "      问题：第 " section " 节有 " summary_count[section] " 行 layer0_findings_summary"; problems++ }
        if (summary_signatures[section] != ((final_count[section] + 0) "")) {
          print "      问题：第 " section " 节定稿 " (final_count[section] + 0) " 行 layer0_finding，summary 却说 signatures=" summary_signatures[section]; problems++
        }
        if (break_item != "ignore-red-states" && summary_red_states[section] != "0") {
          print "      问题：第 " section " 节 red_states=" summary_red_states[section] "：有状态判红（不是 0 都判红；读不出数也判红）"; problems++
        }
      }
      exit (problems > 0 ? 1 : 0)
    }
  ' "$findings_file"
}

# report_manifest_differences <前一份清单> <后一份清单> <前一份的叫法> <后一份的叫法>：逐个列出两份清单里不同的文件，最多 20 个，另报总数。
report_manifest_differences() {
  local difference_lines difference_count
  difference_lines="$(awk -v earlier_name="$3" -v later_name="$4" '
    FNR == NR { earlier_hash[substr($0, 67)] = substr($0, 1, 64); next }
    {
      later_path = substr($0, 67)
      if (!(later_path in earlier_hash)) print "只在" later_name "里：" later_path
      else if (earlier_hash[later_path] != substr($0, 1, 64)) print "内容不同：" later_path
      delete earlier_hash[later_path]
    }
    END { for (earlier_path in earlier_hash) print "只在" earlier_name "里：" earlier_path }
  ' "$1" "$2" | LC_ALL=C sort)"
  difference_count="$(grep -c . <<< "$difference_lines")"
  echo "       不同的文件共 ${difference_count} 个（最多列 20 个）："
  sed -n '1,20p' <<< "$difference_lines" | sed 's/^/         /'
}

# print_staged_worktree_full_commands：出路里建「HEAD + 暂存区」worktree、在里面用那棵树里的 54 号跑 --full 的命令。
# 建法与共享 gate.sh --staged 相同：worktree add --detach HEAD，再 apply --index 暂存区的 diff（diff 为空就不套）。
print_staged_worktree_full_commands() {
  echo '                在项目根、暂存之后，把下面三行命令放进同一次 Bash 调用（三行共用 layer0_full_base 这个变量，分开跑它就是空的；这次调用的退出码是经内存包装的那条 --full 命令的，250–254 是包装自己的结局）：'
  echo '                layer0_tree_ready=; layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"'
  echo '                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; } && layer0_tree_ready=1'
  echo '                if [ "$layer0_tree_ready" = 1 ]; then SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash "${layer0_full_base:?}/tree/.claude/gate.d/54-layer0-replay.sh" --full "${layer0_full_base:?}/tree"; layer0_full_rc=$?; else echo "worktree 没建好或暂存区的 diff 套不上，--full 没跑"; layer0_full_rc=1; fi; git worktree remove --force "${layer0_full_base:?}/tree" 2>/dev/null; rm -rf "${layer0_full_base:?}"; ( exit "$layer0_full_rc" )'
}

# run_checker_quick_tier：快档跑 checker 档包不标 ignored 的用例。判红打出路、返回 1；判绿把每个测试二进制那一行 test result 的 passed / ignored 加总进 quick_tier_report。
run_checker_quick_tier() {
  local quick_log result_lines passed_count ignored_count passed_total ignored_total binary_count
  quick_log="$layer0_scratch_directory/quick-singlefs-checker-tier.log"
  if ! run_checker_package_tests "$quick_log"; then
    tail -40 "$quick_log"
    echo "  ✗ checker 档快档判红（上面是 cargo test 的尾部）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-checker-tier -- --nocapture（只看一个测试二进制加 --test <名>）；"
    echo "                断言消息里是第一处对不上的计数或违例；改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    return 1
  fi
  result_lines="$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$quick_log")"
  passed_total=0; ignored_total=0; binary_count=0
  while read -r passed_count ignored_count; do
    [[ -n "${passed_count:-}" ]] || continue
    passed_total=$((passed_total + passed_count)); ignored_total=$((ignored_total + ignored_count)); binary_count=$((binary_count + 1))
  done <<< "$result_lines"
  if (( passed_total == 0 )); then
    echo "  ✗ checker 档快档跑过了，却读不到 cargo 的 test result 行，或一条用例都没通过：扫到 0 条不是通过"
    echo "     → 怎么办：cargo test --release -p singlefs-checker-tier -- --list 看这个包里还剩几条不标 ignored 的用例；"
    echo "                一条都没有，就是快用例被整批标了 ignored 或删掉了，补回来。"
    return 1
  fi
  quick_tier_report="checker 档包 ${binary_count} 个测试二进制，${passed_total} 条通过、${ignored_total} 条 ignored（全量那几条留给 --full）"
  return 0
}

# ── 快档：checker 档包不标 ignored 的用例，再逐条核崩溃枚举用例这批输入那一格全绿标记 ─────────
if [[ "$layer0_tier" == quick ]]; then
  quick_tier_report=""
  run_checker_quick_tier || exit 1
  present_report="$layer0_scratch_directory/present-report"
  missing_report="$layer0_scratch_directory/missing-report"
  : > "$present_report"
  : > "$missing_report"
  missing_cases=()
  for crash_case_row in "${crash_case_rows[@]}"; do
    IFS=$'\t' read -r case_key _case_package _case_target _case_function <<< "$crash_case_row"
    case_manifest="$layer0_scratch_directory/quick-manifest.${case_key#crash-case:}"
    if ! write_crash_case_manifest "$case_key" "$case_manifest"; then
      missing_cases+=("$case_key")
      echo "       $case_key：算不出这批输入的指纹：$case_manifest_problem" >> "$missing_report"
      continue
    fi
    if marker_check_output="$(python3 "$layer0_admission_module" crash-case-marker-check "$ROOT" "$case_key" "$case_fingerprint" "$case_manifest")"; then
      read -r _ok_word _marker_path marker_finished_utc <<< "$(head -1 <<< "$marker_check_output")"
      {
        echo "    $case_key：那一格跑完于 ${marker_finished_utc}（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去用例读不到的 ${case_excluded_count} 个）"
        tail -n +2 <<< "$marker_check_output" | sed 's/^/      /'
      } >> "$present_report"
    else
      missing_cases+=("$case_key")
      {
        echo "       $case_key（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去用例读不到的 ${case_excluded_count} 个）："
        sed 's/^/         /' <<< "$marker_check_output"
      } >> "$missing_report"
    fi
  done
  echo "  ✓ checker 档快档跑完（release，只跑不标 ignored 的用例，全量留给 --full）：${quick_tier_report}"
  if (( ${#missing_cases[@]} > 0 )); then
    echo "  ! ${#missing_cases[@]} 条崩溃枚举用例没有作数的全绿标记，全量这一轮没跑，记「本次未跑」、不判红（全量默认不在提交时跑，D13 已定项 15）：${missing_cases[*]}"
    cat "$missing_report"
    legacy_marker_total="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green*' 2>/dev/null | grep -c .)"
    if [[ "$legacy_marker_total" -gt 0 ]]; then
      echo "       common-dir 里还有 ${legacy_marker_total} 格按整批输入分格的旧标记（singlefs-layer0-full-green*）：分成逐条用例之后不再认，可以删掉。"
    fi
    echo "     → 要跑全量（用户要求或夜间）：暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>"
    echo "                （与 gate.sh --staged 同一建法；它只跑没有作数标记的那几条，别的复用）："
    print_staged_worktree_full_commands
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
      for missing_case in "${missing_cases[@]}"; do
        echo "崩溃枚举用例 ${missing_case} 这批输入没有作数的全绿标记，全量没跑（要跑：54 号 --full）" >> "$GATE_NOT_RUN_FILE"
      done
    fi
  fi
  echo "  ✓ 登记 ${#crash_case_rows[@]} 条崩溃枚举用例，$(( ${#crash_case_rows[@]} - ${#missing_cases[@]} )) 条的全绿标记与各自这批输入的指纹相同（登记路径 ${layer0_input_paths_text}，逐条减去用例读不到的文件），标记里的计数行原样："
  cat "$present_report"
  exit 0
fi

# ── --full：逐条崩溃枚举用例照复用判定跑；那一格在就复用，不在才跑，判绿写那一格 ─────────
full_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
start_over_note="不带 --start-over：有进度文件就接着跑"
if [[ "$layer0_start_over" == 1 ]]; then start_over_note="带 --start-over：进度文件整份丢掉、从头跑"; fi
echo "  · --full 开跑（${full_started_utc}）：${#crash_case_rows[@]} 条崩溃枚举用例逐条照复用判定跑（这批输入那一格全绿标记在就复用）；续跑的进度文件在 ${layer0_progress_root}/<输入指纹>/，${start_over_note}"
# 这一趟的全量日志与发现日志：common-dir 下新建一个目录（不随 worktree 删，跑完不删；每趟一个新目录，不拿上一趟的输出顶上）
layer0_log_directory="$git_common_directory/singlefs-layer0-logs/$(date -u +%Y%m%dT%H%M%SZ)-$$"
if ! mkdir -p -- "$layer0_log_directory"; then
  echo "  ✗ 建不了这一趟的日志目录 $layer0_log_directory：全量日志与发现日志没处放"
  echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full。"
  exit 1
fi
echo "  · 这一趟的全量日志与发现日志放 ${layer0_log_directory}/（log.<用例名> 与 log.<用例名>.findings.tsv，跑完不删）"
# 双机分片开不开：本地配置在、判得过才开（判法在 layer0-shard-configuration-check.sh，与驱动脚本的运行条件同一份）
layer0_shard_driver="$ROOT/research/scripts/layer0-shard-run.sh"
if layer0_shard_configuration_note="$(bash "$ROOT/research/scripts/layer0-shard-configuration-check.sh" "$ROOT" 2>&1)"; then
  layer0_sharded=1
  echo "  · 双机分片：开（${layer0_shard_configuration_note//$'\n'/；}）；登记了 shard=across-machines 的用例两台各跑一片、本机 merge，别的单机跑"
else
  layer0_sharded=0
  echo "  · 双机分片：关（${layer0_shard_configuration_note//$'\n'/；}）；每条用例单机跑"
fi
red_cases=()
green_cases=()
reused_cases=()
for crash_case_row in "${crash_case_rows[@]}"; do
  IFS=$'\t' read -r case_key case_package case_target case_function <<< "$crash_case_row"
  case_label="${case_key#crash-case:}"
  manifest_at_start="$layer0_scratch_directory/manifest-at-start.$case_label"
  if ! write_crash_case_manifest "$case_key" "$manifest_at_start"; then
    echo "  ✗ $case_key：算不出这批输入的指纹：$case_manifest_problem"
    echo "     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- ${layer0_input_paths_text} 看列不列得出文件，再跑 cargo -V && rustc -V；"
    echo "                单跑 python3 research/scripts/admission.py crash-case-manifest <根> $case_key <清单文件> --judging-digest --toolchain --build-environment 看它报什么。"
    red_cases+=("$case_key")
    continue
  fi
  fingerprint_at_start="$case_fingerprint"
  file_count_at_start="$case_file_count"
  excluded_count_at_start="$case_excluded_count"
  if marker_check_output="$(python3 "$layer0_admission_module" crash-case-marker-check "$ROOT" "$case_key" "$fingerprint_at_start" "$manifest_at_start")"; then
    read -r _ok_word reused_marker_path reused_finished_utc <<< "$(head -1 <<< "$marker_check_output")"
    echo "  · $case_key 复用：这批输入（指纹 ${fingerprint_at_start:0:16}…）那一格全绿标记跑完于 ${reused_finished_utc}，这一趟不跑；要重跑就删掉 ${reused_marker_path}"
    reused_cases+=("$case_key")
    continue
  fi
  if ! read_crash_case_command "$case_key" "$fingerprint_at_start"; then
    echo "  ✗ $case_key：准入模块交不出起用例的命令：$case_command_problem"
    echo "     → 怎么办：单跑 python3 research/scripts/admission.py crash-case-command <根> $case_key <指纹> 看它报什么（取不到 git common-dir、"
    echo "                nproc 起不来、SINGLEFS_LAYER0_THREADS 不是正整数都在这里报）；修好之后重跑 --full。"
    red_cases+=("$case_key")
    continue
  fi
  case_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  case_log="$layer0_log_directory/log.$case_label"
  case_findings_file="$case_log.findings.tsv"
  rm -f -- "$case_log" "$case_findings_file"
  case_threads_origin_text="没设，取本机核数"
  if [[ "$case_threads_origin" == explicit ]]; then case_threads_origin_text="显式设的"; fi
  case_threads_note="SINGLEFS_LAYER0_THREADS=${case_threads}（${case_threads_origin_text}），本机 ${case_machine_cores} 核"
  case_sharded=0
  if [[ "$layer0_sharded" == 1 ]] && python3 "$layer0_admission_module" crash-case-shardable "$ROOT" "$case_key" >/dev/null; then case_sharded=1; fi
  case_way="单机跑：${case_command[*]}"
  if [[ "$case_sharded" == 1 ]]; then
    case_way="双机分片跑（本机 0/2、第二台 1/2，本机 merge/2；bash research/scripts/layer0-shard-run.sh --merged-log $case_key <树根> <指纹> <日志>）"
    case_threads_note="本机那一片 ${case_threads_note}，第二台那一片取第二台的核数（逐片的数在 merge 那一行 LAYER0_PARALLEL_FINISHED 的 shard_…= 里）"
  fi
  echo "  · $case_key 开跑（${case_started_utc}；${case_way}；${case_threads_note}；这批输入的指纹 ${fingerprint_at_start:0:16}…，${file_count_at_start} 个文件，减去用例读不到的 ${excluded_count_at_start} 个）："
  sed 's/^/      /' <<< "$marker_check_output"
  echo "    全量日志：${case_log}；发现日志：${case_findings_file}"
  if [[ "$case_sharded" == 1 ]]; then
    run_crash_case_in_two_shards "$case_key" "$case_log" "$fingerprint_at_start"
  else
    run_crash_case "$case_log" "$case_findings_file" "${case_command[@]}"
  fi
  case_run_exit=$?
  # 发现表跑完就打（判红的那一趟最要看它）；判定留到下面，cargo 退非 0、日志判不绿先报
  if report_crash_case_findings "$case_findings_file" "$case_log"; then case_findings_exit=0; else case_findings_exit=$?; fi
  if [[ "$case_run_exit" != 0 ]]; then
    [[ -f "$case_log" ]] && tail -40 "$case_log"
    delete_crash_case_marker "$case_key" "$fingerprint_at_start"
    if [[ "$case_sharded" == 1 ]]; then
      echo "  ✗ $case_key 判红：双机分片那一趟退 $case_run_exit（上面是驱动脚本的输出与 merge 那一趟日志的尾部）"
      echo "     → 怎么办：驱动脚本输出里判红的那一句说清卡在哪一步（工具链、指纹、某一片、账本、merge）；要单机复核，挪开本地配置 multi-host.env 再跑 --full"
      red_cases+=("$case_key")
      continue
    fi
    echo "  ✗ $case_key 判红：cargo test 退非 0（上面是它的尾部）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p $case_package --test $case_target -- --include-ignored --exact $case_function --nocapture"
    echo "                断言消息里是第一处对不上的计数或违例。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    red_cases+=("$case_key")
    continue
  fi
  judged_lines_file="$layer0_scratch_directory/judged.$case_label"
  if ! judge_output="$(python3 "$layer0_admission_module" crash-case-judge "$ROOT" "$case_key" "$case_log" "$judged_lines_file")"; then
    delete_crash_case_marker "$case_key" "$fingerprint_at_start"
    printf '%s\n' "$judge_output" | sed 's/^/       /'
    echo "  ✗ $case_key 的用例跑过了，日志却判不绿（上面逐条列出）"
    echo "     → 怎么办：计数行不是恰好一行、过滤之后没跑到恰好一条用例，对一对 stage-inputs.tsv 里 $case_key 那一行第三列与用例打印的行；"
    echo "                不是全量去 crates/singlefs-checker-tier/src/crash.rs 的 enumerate_layer0 看；只起了 1 个线程看 Layer0Parallelism::from_environment 读没读到 SINGLEFS_LAYER0_THREADS，"
    echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_LAYER0_THREADS=1 bash .claude/gate.d/54-layer0-replay.sh --full <根>。"
    red_cases+=("$case_key")
    continue
  fi
  if [[ "$case_findings_exit" != 0 ]]; then
    delete_crash_case_marker "$case_key" "$fingerprint_at_start"
    echo "  ✗ $case_key 的用例跑过了、日志也判得绿，发现日志却判红（上面发现表里的「问题：」逐条列出）"
    echo "     → 怎么办：「这一趟没跑完」看全量日志 $case_log 的尾部是 panic 还是被杀；red_states 不是 0 是有状态判红而用例没红，"
    echo "                对一对用例的计数断言与 crates/singlefs-checker-tier/src/crash.rs 的发现表；发现日志不在而全量日志里有 LAYER0_FINDINGS 行，"
    echo "                是 SINGLEFS_LAYER0_FINDINGS_FILE 没传进用例（单机看本阶段 run_crash_case，双机看 research/scripts/layer0-shard-run.sh 给 merge 那一趟的设置）。"
    red_cases+=("$case_key")
    continue
  fi
  manifest_at_finish="$layer0_scratch_directory/manifest-at-finish.$case_label"
  if ! write_crash_case_manifest "$case_key" "$manifest_at_finish"; then
    echo "  ✗ $case_key 跑完之后算不出这批输入的指纹：$case_manifest_problem；不写全绿标记"
    echo "     → 怎么办：多半是跑的过程中有人删了登记路径下的文件或工具链坏了；等改动停下，在 HEAD + 暂存区的 worktree 里重跑 --full。"
    red_cases+=("$case_key")
    continue
  fi
  if [[ "$case_fingerprint" != "$fingerprint_at_start" ]]; then
    echo "  ✗ $case_key 跑的过程中它的输入变了（开跑 ${fingerprint_at_start:0:16}…，跑完 ${case_fingerprint:0:16}…）：读到的不一定是同一版，不写全绿标记；开跑那一批已有的那一格不删"
    report_manifest_differences "$manifest_at_start" "$manifest_at_finish" "开跑时" "跑完时"
    echo "     → 怎么办：别在有人改这些路径的树里跑。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
    print_staged_worktree_full_commands
    red_cases+=("$case_key")
    continue
  fi
  threads_text="${judge_output//$'\n'/；}${judge_output:+；}${case_threads_note}"
  if ! record_output="$(python3 "$layer0_admission_module" crash-case-record "$ROOT" "$case_key" "$fingerprint_at_start" "$manifest_at_start" "$judged_lines_file" \
      --files "$file_count_at_start" --excluded "$excluded_count_at_start" --started "$case_started_utc" --judged-root "$ROOT")"; then
    echo "  ✗ $case_key 判绿，全绿标记却没写成：$record_output"
    echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full（没有标记，整轮门禁的快档会一直红）。"
    red_cases+=("$case_key")
    continue
  fi
  rmdir -- "$case_progress_directory" 2>/dev/null
  echo "  ✓ $case_key 判绿（${threads_text}）：全绿标记写进 ${record_output}，记下的行原样："
  sed 's/^/      /' "$judged_lines_file"
  green_cases+=("$case_key")
done
full_finished_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if (( ${#red_cases[@]} > 0 )); then
  echo "  ✗ --full 有 ${#red_cases[@]} 条崩溃枚举用例判红：${red_cases[*]}（这一趟判绿 ${#green_cases[@]} 条、复用 ${#reused_cases[@]} 条；开跑 ${full_started_utc}，跑完 ${full_finished_utc}）"
  echo "     → 怎么办：逐条照它自己那一句「→ 怎么办」改；改完暂存，再在 HEAD + 暂存区的 worktree 里跑 --full（这一趟判绿的与复用的那几条下一趟照样复用）。"
  exit 1
fi
echo "  ✓ --full 跑完（开跑 ${full_started_utc}，跑完 ${full_finished_utc}）：${#crash_case_rows[@]} 条崩溃枚举用例，这一趟跑了判绿 ${#green_cases[@]} 条（${green_cases[*]:-无}），复用 ${#reused_cases[@]} 条（${reused_cases[*]:-无}）；日志在 ${layer0_log_directory}/"
if [[ "$layer0_sample_mode" == 1 ]]; then
  echo "  ! 样本档（.layer0-sample-tools，假 cargo）：流程判绿，退 3 不退 0——这一趟没跑过真的层 0"
  echo "     → 真跑要在不带 .layer0-sample-tools 的树里跑 --full（建法见快档的出路句）。"
  exit 3
fi
