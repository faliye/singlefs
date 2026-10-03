#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 research/scripts/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: crates 变异表复跑（每条改坏一处、点名的测试必须红）
# gate-category: checker-tier 类
# gate-similar: code-source-discipline.sh 它的实验与变异源码纪律那一格经同一份判法（research/scripts/crates-mutation-rows.py static-check）静态判变异表成形与拷贝范围，不改坏、不起 cargo；这里真改坏每一处、真跑点名的测试，是重型，两边共用判法模块、不共用阶段
# gate-similar: checker-tier-research-build-and-replay.sh 它的复跑那一格也按输入复用、也逐条跑编出来的代码，但跑的是 research 的实验二进制、比的是产物；这里改坏的是 crates 的源码、判的是点名的测试红不红
#
# gate-cell: crates-mutation-rows crates/mutations.tsv 每条改坏一处、点名的测试都红，基线在没改坏的副本上绿
# 参数解析、格汇总与退出码经共用库 lib/stage-cells.sh（写法在它的文件头）。
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh [项目根]                                整张表
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --list                                  逐行打格名与判什么，不起 cargo
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --check crates-mutation-rows [项目根]     只跑这一格（格名写错退 2）
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --list-items [项目根]                    逐行打变异表里可点名的变异名、行号与改哪个文件，不起 cargo
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --item <变异名> [--item <变异名>…] [项目根]   只判点名的几行
#   --item：名字原样写、名字里的逗号不拆，不在表里退 2；换成行号走共用模块的选行模式（不问整道的复用判定、不调双机驱动、不写全绿标记，照常写按条记录）。
#     选中的全抓到这一格绿（模块退 3，这里换成 0，成功行写明只判了这几行）；有本次未跑的，成功行写明抓到几条、本次未跑几条；全是本次未跑的退 77。
#   GATE_MUTATION_ROW_SELECTION=<文件>（一行一个变异表行号，只由双机分片的驱动设）：只判这些行（它们所在的基线批照跑），照常写按条记录，
#     不写全绿标记、不问整道的复用判定，末行写明「只判了选中的 N 行，不是整道结论」；不经格汇总，选中的全抓到退 3、有红退 1
#     （汇总把 0、77 之外的码并成 1，驱动就分不出这两样）。
#
# 判据（判法、派活、按条记录与全绿标记都在 research/scripts/crates-mutation-rows.py；拷贝范围、底座指纹、行键、记录放哪、几时复用、弄坏开关见它的文件头）：
#   crates/mutations.tsv 每行六段（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），替换文可以空（删掉原文），其余五段不许空；
#   原文在文件里恰好命中一次，必须红的测试名是一个用例名，只有 \n 一种反斜杠转义，文件落在每片拷的范围（stage-inputs.tsv 本阶段那一行）之内：
#   这几样派活之前判（row_problems，门禁 code-source-discipline 经 static-check 静态判同一份）；
#   点名的测试在没改坏的副本上恰好对上一个用例、结果是 ok（基线）；改坏那一处之后它必须红。一条不过整道红。
# 为什么：show-me-test.md「新增的测试必须先证明它会红」要变异清单进仓、交给门禁反复复跑；每处三方打中之后的改法留一条「改回去它就红」的变异，撤回时这里先响。
#
# 并发（派活与进程数的判法在 crates-mutation-rows.py 的 worker_count_for 与 dispatch_order）：每片各一份源码副本与一个编译目录（共用一份会被 cargo 的文件锁重新串行化）；
#   按 crate 归堆派（增量编译的命中率才稳），活动态领（等分之下拿到几十秒重测试的那一片会独自拖住整道），上一次用时 ≥ GATE_MUTATION_LONG_ROW_SECONDS（默认 300）秒或超时的先派。
#   工作进程数取 ① 核数的一半、至多 16（设了 GATE_MUTATION_WORKERS 就顶掉 ①）与 ② 这一趟要现跑的条数的小者、至少 1；不按内存收：开跑那一刻看不见别的重活来抢。
#   ⚠️ stdout 与进程数、复用了多少都无关：判定按变异表的行号排序再打，GATE_MUTATION_WORKERS=1 与 =8 逐字相同（并发之后重新证红的那一半）；进度、复用、前移只进 stderr。
#
# 编译目录：${GATE_MUTATION_TARGET_DIR:-${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-crates-mutation-target}，分片各加 -w<片号>、各自冷编一次（prepare_shard）。
#   放 GATE_CROSS_RUN_TMPDIR 不放 TMPDIR：门禁每轮给阶段的私有 TMPDIR 跑完就删，放进去每轮都要从头编。
# 临时目录：每条变异与每批基线各一个 TMPDIR，结束时不论退出码（含限时、撞顶被杀）都整个删掉：被杀的测试 Drop 守卫不跑，镜像会堆在里面；
#   全部判完总目录下还剩哪一条的目录，整道判红。
#
# 内存：每条 cargo test 经 research/scripts/run-with-memory-cap.sh（systemd 临时 scope，MemoryMax=<上限>、MemorySwapMax=0）跑，撞上限只杀这一条。
#   上限先到先用：变异表里单起一行「# 每条变异的内存上限：<上限>」、GATE_MUTATION_MEMORY_MAX、默认 4G。
#   默认 4G 的依据：本机 60 GiB、常驻约 12 GiB、同时可能 3 件重活，每件 16 GiB 里再分给几个工作进程；四个 crate 全部测试目标冷编的匿名内存峰值 0.60 GiB。
#   点名的测试跑起来要多少没量，按条记在 research/scripts/memory-peaks.tsv。这些数只在本机成立，换机器要重算。
#   包装退 250 记「内存撞顶」，254 记「被总上限挤掉」，252 记「排不上没跑」：都不算抓到也不算没红，单列、整道判红。
#   scope 或 slice 的总上限设不上（没有用户级 systemd、D-Bus 连不上）整道判红、一条都不跑，不退回无上限去跑。
# 限时：交给包装（RUN_WITH_MEMORY_CAP_TIME_LIMIT → scope 的 RuntimeMaxSec，从起跑算、含重编译）；不在包装外面套 timeout：排队的时间会被算成超时。
#   秒数先到先用：变异表里单起一行「# 每条变异的超时秒数：<秒>」、GATE_MUTATION_TIMEOUT、默认 1800。包装退 253 记「超时」：单列一栏，整道判红。
#   默认 1800 的依据：点名随机历史测试的变异在 16 个工作进程并发时一条超过 6–8 分钟（C457（被测试自己的线程池与外层进程池叠加超订阅）），约 4 倍留余地；
#   代价是一条真挂死的变异让一个工作进程多占 30 分钟。本机的数，换机器要重算。
#
# 基线（判法见 crates-mutation-rows.py 文件头「基线」）：点名的测试在没改坏的副本上本来就红时，改坏之后照样红、记「抓到」什么都没证明，
#   所以派变异之前先跑基线，不是恰好一个 ok 的行记「基线不绿」、整道判红。「计数：」那一行的格式不动（mutation-triage 照栏读），基线另起一行「基线：…」报。
# 按条记录（写法见 crates-mutation-rows.py 文件头「按条记录」）：下一趟只跑没有作数记录的行；登记输入里任何一份变了（含这份阶段脚本）整张重跑。
#   只有「抓到」与基线 ok 复用；SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 一律不复用。判绿之后写全绿标记（admission.py stage-marker-write），写不成只报不红。
# 双机分片：只在本地多机配置写了 ENABLE_ACROSS_MACHINES=1 时开（判法 research/scripts/layer0-shard-configuration-check.sh：退 0 开、退 3 关、退 1 判不过），不认环境变量开。
#   整道跑、没设 SINGLEFS_GATE_FULL=1 与 GATE_MUTATION_START_OVER=1（这两样下最后那一趟不复用，分出去的白跑）、判法退 0 时，
#   先调 research/scripts/mutation-shard-run.sh 把没有作数记录的行分到两台各判一份（步骤与弄坏开关在它文件头），输出进 stderr；
#   驱动退几都照常整张判一遍：命中记录的复用、缺的现跑，stdout 与单机逐字相同，全绿标记由本机写。
# 本次未跑（判法在 crates-mutation-rows.py 文件头「本次未跑」）：点名的测试打 NOT_RUN_NO_GPU_ADAPTER 后返回成功的那一条不算抓到、不算没红，整道不因它判红；
#   只有本次未跑的这一格退 77。模块写进 GATE_MUTATION_NOT_RUN_RECORD，这里转进 $GATE_NOT_RUN_FILE（驱动起的那两份不转，由本机整张那一趟再报）。
#
# 弄坏开关（只给证红用）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/checker-tier-crates-mutation-replay.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首；样本档登记为 samples-heavy，句首都是逐个打开开关实跑样本档、照原样输出写的）：
#   GATE_MUTATION_BREAK=keepmutationtmp  拿掉删临时目录那一步
#       red（输出里找不到「endless-walk：20 秒没跑完（超时），点名的测试 walking_to_ten_ends_at_ten 没来得及红；它的临时目录已整个删掉（删前留有 1 个文件」）、
#       green（期望退出 0，实测 1）、not-run（期望退出 0，实测 1）
#   GATE_MUTATION_BREAK=nobaseline  跳过基线，读副本外文件的那一行误判成抓到
#       red（输出里找不到「计数：抓到 0 条、没红 1 条、无效 1 条、内存撞顶 1 条、超时 1 条…」）、
#       green（输出里找不到「基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 5 行，基线不绿 0 行…」）、
#       not-run（输出里找不到「基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 2 行，基线不绿 0 行…」）
#   GATE_MUTATION_BREAK=emptyreplacementrefused  替换文空着也判不成形
#       green（期望退出 0，实测 1）
#   GATE_MUTATION_BREAK=notrunasmissed  把本次未跑当没红记
#       not-run（期望退出 0，实测 1）
#   MUTATION_SHARD_RUN_BREAK 的几项只在 research/scripts/mutation-shard-run.sh --selftest 起的进程里认（那份自证放的标记文件 MUTATION_SHARD_RUN_SELFTEST_MARK，
#   内容与驱动的 MUTATION_SHARD_SELFTEST_MARK_CONTENT 同一行）；别处设了，这一格判红、什么都不判：
#   MUTATION_SHARD_RUN_BREAK=break-honored-anywhere  别处设了也认
#       本道样本不判错；红在 research/scripts/mutation-shard-run.sh --selftest 的一格（「✗ ⑩ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外…」）
#   MUTATION_SHARD_RUN_BREAK=no-final-pass  调完驱动就按驱动的退出码退，不再整张判
#       本道样本不开双机、不判错；红在 research/scripts/mutation-shard-run.sh --selftest 的四格（「✗ ① checker-tier-crates-mutation-replay 接驱动那一趟判绿…」、
#       「✗ ① 最后那一趟全部复用…」、「✗ ③ 第二台那一份判完 1 条就停…」、「✗ ④ 驱动拒了之后 checker-tier-crates-mutation-replay 在本机整张判…」）
#   MUTATION_SHARD_RUN_BREAK=environment-opens  GATE_MUTATION_ACROSS_MACHINES=1 照旧把双机打开、不看配置的开关
#       本道样本不判错；红在 research/scripts/mutation-shard-run.sh --selftest 的一格（「✗ 单机那一趟（配置没开双机、环境变量 GATE_MUTATION_ACROSS_MACHINES=1）判绿…」）
#   复用、续跑、只改一行只跑那一行、改源码全部重跑、久的先派、选行、导入这几样要连跑好几趟，在 research/scripts/crates-mutation-rows.py --selftest 里
#   用假 cargo 跑（门禁 code-tooling 替它复跑），弄坏开关 basewithoutsource、timingsignored、importunchecked、nobaseline、emptyreplacementrefused 下各有一格必须判错。
# 样本（fixtures/checker-tier-crates-mutation-replay.sh/；各放 .gate-cells 点名 crates-mutation-rows，setup.sh 在样本目录里 git init）：
#   red      变异表把上限压到 512M、限时压到 20 秒：1 MiB 的缓冲改成 1536 MiB 的那条报「内存撞顶」；步长 1 改成 0、循环走不到上界的那条报「超时」，
#            它点名的测试先往 TMPDIR 写一个 4 KiB 的文件，判定里报「临时目录已整个删掉（删前留有 1 个文件」（0 个说明 TMPDIR 没传到测试进程）；
#            点名的测试读 fixture-data/answer.txt（样本仓里有、不在拷贝范围里）的一行与只写名字、两个模块各有一个同名用例的一行都报「基线不绿」
#            （基线那一行报 4 行 ok、2 行不绿）；「临时目录」那一行报 4 条都删掉了；计数行里没红、无效、内存撞顶、超时各数各的
#   green    同样压到 512M，留一行撤掉了的「# 给整机留的内存余量：1T」：工作进程数那一行报开 5 个、卡在「这一趟要现跑的条数」，不按内存收；
#            五条正常变异照旧红在点名的测试上，其中一条改 litmus/sample.litmus（拷贝范围要含 litmus/），一条替换文空着、删掉 add_twice 里第二次相加；
#            基线那一行报 5 行都 ok，「临时目录」那一行报 5 条都删掉了，stderr 头一行报复用 0 条
#   not-run  一条点名的测试只打 NOT_RUN_NO_GPU_ADAPTER 那一行就返回、一条正常变异：报本次未跑 1 条、抓到 1 条、这一格绿
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell crates-mutation-rows cell_crates_mutation_rows "crates/mutations.tsv 每条改坏一处、点名的测试都红，基线在没改坏的副本上绿" \
  "照上面逐条的判定改：没红的补测试或改变异，锚点腐化的改原文，基线不绿的先让点名的测试在没改坏的副本上绿；只重跑某几行：bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --item <变异名>"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

list_items=0; root_argument=""; named_mutations=()
while (($#)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" ]]; then
        echo "  ✗ --item 后面缺变异名"
        echo "     → 怎么办：写成 --item <变异名>（变异表第一列，原样照抄，名字里的逗号不拆），几行就给几次；可点的名字用 --list-items 看。"
        exit 2
      fi
      named_mutations+=("$2"); shift 2 ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check crates-mutation-rows、--list-items、--item <变异名> 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2
MUTATION_TABLE="crates/mutations.tsv"

if ((list_items)); then
  [[ -f "$MUTATION_TABLE" ]] || { echo "  ✗ 没有 $ROOT/$MUTATION_TABLE：列不出可点名的变异"; echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"; exit 1; }
  # 一行一条变异（# 起头的行与空行不算）：变异名<制表符>第几行、改哪个文件
  awk -F'\t' '/^#/ || /^[[:space:]]*$/ { next } { printf "%s\t第 %d 行，改 %s\n", $1, NR, $2 }' "$MUTATION_TABLE"
  exit 0
fi

item_selection_file=""
if ((${#named_mutations[@]})); then
  if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
    echo "  ✗ --item 与环境里的 GATE_MUTATION_ROW_SELECTION 一起给了：两份选行不知道听哪一份"
    echo "     → 怎么办：手跑只用 --item；GATE_MUTATION_ROW_SELECTION 只由双机分片的驱动设。"
    exit 2
  fi
  [[ -f "$MUTATION_TABLE" ]] || { echo "  ✗ 没有 $ROOT/$MUTATION_TABLE：--item 点名的变异无从找行"; echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"; exit 1; }
  item_selection_file="$(mktemp)" || { echo "  ✗ 建不了选行文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  trap 'rm -f -- "${item_selection_file:?}"' EXIT
  unknown_mutations=()
  for named in "${named_mutations[@]}"; do
    line_numbers="$(awk -F'\t' -v wanted="$named" '/^#/ || /^[[:space:]]*$/ { next } $1 == wanted { print NR }' "$MUTATION_TABLE")"
    if [[ -z "$line_numbers" ]]; then unknown_mutations+=("$named"); else printf '%s\n' "$line_numbers" >> "$item_selection_file"; fi
  done
  if ((${#unknown_mutations[@]})); then
    printf '  ✗ --item 点名的变异不在 %s 里：%s\n' "$MUTATION_TABLE" "$(printf '「%s」' "${unknown_mutations[@]}")"
    echo "     → 怎么办：变异名照 bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --list-items 列的第一列原样写。"
    exit 2
  fi
  GATE_MUTATION_ROW_SELECTION="$item_selection_file"
  export GATE_MUTATION_ROW_SELECTION
fi

# ── crates-mutation-rows 格 ──
cell_crates_mutation_rows() {
# 弄坏开关 MUTATION_SHARD_RUN_BREAK 只在 research/scripts/mutation-shard-run.sh --selftest 起的进程里认（文件头弄坏开关那一段）：别处设了就判红、什么都不判
if [[ -n "${MUTATION_SHARD_RUN_BREAK:-}" && ",${MUTATION_SHARD_RUN_BREAK}," != *,break-honored-anywhere,* ]] \
   && ! { [[ -n "${MUTATION_SHARD_RUN_SELFTEST_MARK:-}" && -f "$MUTATION_SHARD_RUN_SELFTEST_MARK" ]] \
          && [[ "$(head -n 1 -- "$MUTATION_SHARD_RUN_SELFTEST_MARK")" == "mutation-shard-run.sh --selftest 起的进程" ]]; }; then
  echo "  ✗ MUTATION_SHARD_RUN_BREAK=$MUTATION_SHARD_RUN_BREAK 设在 research/scripts/mutation-shard-run.sh --selftest 之外：弄坏开关只在那份自证起的进程里生效，这一趟什么都没判"
  echo "     → 怎么办：去掉它再跑（env -u MUTATION_SHARD_RUN_BREAK …）；要证红就跑 MUTATION_SHARD_RUN_BREAK=<项> bash research/scripts/mutation-shard-run.sh --selftest"
  exit 1
fi
# 先问能不能复用上一次整轮全绿的判定（为什么用树、只一条 ref、不看工作区，见 research/scripts/stage-must-run.sh 的文件头）；
# 只判选中的几行（GATE_MUTATION_ROW_SELECTION，驱动设的或 --item 换成的）时不问：那一问是整张表的，选中的几行按条记录复用。
if [[ -z "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
  source "$STAGE_REPOSITORY/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
  stage_run_or_skip "$STAGE_REPOSITORY/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")"
fi
# 前提读的是本阶段所在那一份仓的登记表：判别力样本的目录里没有登记表，前提照样按真表判。
python3 "$STAGE_REPOSITORY/research/scripts/admission.py" gate-preconditions "$STAGE_REPOSITORY" "$(basename "$0")" || exit 1
export GATE_MUTATION_TARGET_DIR="${GATE_MUTATION_TARGET_DIR:-${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-crates-mutation-target}"
# 带内存上限跑一条命令的包装在这份阶段所在的仓里（样本仓里没有 research/）
GATE_MUTATION_MEMORY_CAP_RUNNER="$STAGE_REPOSITORY/research/scripts/run-with-memory-cap.sh"
export GATE_MUTATION_MEMORY_CAP_RUNNER
MUTATION_ROWS_MODULE="$STAGE_REPOSITORY/research/scripts/crates-mutation-rows.py"
if [[ ! -f "$MUTATION_ROWS_MODULE" ]]; then
  echo "  ✗ 找不到 $MUTATION_ROWS_MODULE：这一道的判法、派活与按条记录都在它里面，一条都判不了"
  echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"
  exit 1
fi
# 双机分片（文件头「双机分片」）：驱动与配置判法的输出只进 stderr，stdout 只有下面 run_stage 那一趟的判定。
if [[ -z "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
  across_machines_skipped=""
  across_machines_configuration_exit=0
  if [[ "${SINGLEFS_GATE_FULL:-}" == 1 || "${GATE_MUTATION_START_OVER:-}" == 1 ]]; then
    across_machines_skipped="SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1：这一趟不复用按条记录，分出去判的行本机还要再跑一遍"
  else
    bash "$STAGE_REPOSITORY/research/scripts/layer0-shard-configuration-check.sh" "$ROOT" >/dev/null 2>&1 || across_machines_configuration_exit=$?
    if [[ ",${MUTATION_SHARD_RUN_BREAK:-}," == *,environment-opens,* && "${GATE_MUTATION_ACROSS_MACHINES:-}" == 1 ]]; then
      echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=environment-opens：GATE_MUTATION_ACROSS_MACHINES=1 照旧把双机打开，不看配置的开关" >&2
      across_machines_configuration_exit=0
    fi
    case "$across_machines_configuration_exit" in
      0) ;;
      3) across_machines_skipped="双机：关——本地多机配置没写 ENABLE_ACROSS_MACHINES=1；要开就在配置里写上（模板仓根 multi-host.env.example）" ;;
      *) across_machines_skipped="本地配置判不过；单跑 bash research/scripts/layer0-shard-configuration-check.sh 看原因" ;;
    esac
  fi
  if [[ -n "$across_machines_skipped" ]]; then
    echo "  … 双机分片：没分（$across_machines_skipped），整张表在本机判" >&2
  else
    if bash "$STAGE_REPOSITORY/research/scripts/mutation-shard-run.sh" "$ROOT" >&2; then shard_driver_exit=0; else shard_driver_exit=$?; fi
    echo "  … 双机分片：驱动退 $shard_driver_exit；下面在本机整张判一遍（命中按条记录的复用、缺的现跑）" >&2
    if [[ ",${MUTATION_SHARD_RUN_BREAK:-}," == *,no-final-pass,* ]]; then
      echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=no-final-pass：调完驱动就退，不再整张判" >&2
      exit "$shard_driver_exit"
    fi
  fi
fi
# 判法都在共用模块的 run_stage 里，这里 import 它、交出被判的仓根。
# 模块按名字登记进 sys.modules：进程池的工作进程靠这个名字找回要跑的函数。
# 本次未跑的那几条，模块逐条写进这一格自己的记录，跑完转进 $GATE_NOT_RUN_FILE。
not_run_record="$(mktemp)" || { echo "  ✗ 建不了本次未跑的记录文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
trap 'rm -f -- "${not_run_record:?}"' EXIT
GATE_MUTATION_NOT_RUN_RECORD="$not_run_record"
export GATE_MUTATION_NOT_RUN_RECORD
python3 - "$ROOT" "$MUTATION_ROWS_MODULE" <<'PY'
import importlib.util
import sys

specification = importlib.util.spec_from_file_location("crates_mutation_rows", sys.argv[2])
module = importlib.util.module_from_spec(specification)
sys.modules[specification.name] = module
specification.loader.exec_module(module)
sys.exit(module.run_stage(sys.argv[1]))
PY
module_exit=$?
unset GATE_MUTATION_NOT_RUN_RECORD
not_run_count="$(grep -c . "$not_run_record" 2>/dev/null)" || not_run_count=0
# 双机分片的驱动起的那两份（GATE_MUTATION_ROW_SELECTION 不是 --item 换成的）不转：分出去的行由本机整张那一趟再判、再报
if ((not_run_count > 0)) && [[ -n "${GATE_NOT_RUN_FILE:-}" && ( -z "${GATE_MUTATION_ROW_SELECTION:-}" || -n "$item_selection_file" ) ]]; then
  cat "$not_run_record" >> "$GATE_NOT_RUN_FILE"
fi
# --item 点名的几行：选中的全抓到时模块退 3（选行模式给驱动看的约定），人点名的全抓到就是这一格绿；
# 选中的里有本次未跑的不说都抓到，全是本次未跑的退 77。
if [[ -n "$item_selection_file" && "$module_exit" == 3 && "$not_run_count" -gt 0 ]]; then
  selected_count="$(sort -u "$item_selection_file" | grep -c .)"
  if ((not_run_count >= selected_count)); then
    echo "  · 点名的 $selected_count 条变异全是本次未跑（逐条见上），一条都没判到被改坏的那一处：这一格记本次未跑，不是通过"
    exit 77
  fi
  echo "  ✓ 点名的 $selected_count 条变异里抓到 $((selected_count - not_run_count)) 条、本次未跑 $not_run_count 条（逐条见上；只判了这几行，不是都抓到、也不是整道结论；全绿标记没写）"
  exit 0
fi
if [[ -n "$item_selection_file" && "$module_exit" == 3 ]]; then
  echo "  ✓ 点名的 ${#named_mutations[@]} 条变异都抓到了（只判了这几行，不是整道结论；全绿标记没写）"
  exit 0
fi
exit "$module_exit"
}

# 双机分片的驱动起的那两份不经格汇总，照驱动的约定退（文件头 GATE_MUTATION_ROW_SELECTION 那一条）
if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" && -z "$item_selection_file" ]]; then
  ( cell_crates_mutation_rows )
  exit $?
fi
stage_cells_run "$ROOT"
