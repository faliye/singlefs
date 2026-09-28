#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 research/scripts/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判（用户 2026-09-26 定：项目更严）
# gate-stage: crates 变异表复跑（每条改坏一处、点名的测试必须红）
# gate-category: checker-tier 类
# gate-similar: code-source-discipline.sh 它的实验与变异源码纪律那一格经同一份判法（research/scripts/crates-mutation-rows.py static-check）静态判变异表成形与拷贝范围，不改坏、不起 cargo；这里真改坏每一处、真跑点名的测试，是重型，两边共用判法模块、不共用阶段
# gate-similar: checker-tier-research-build-and-replay.sh 它的复跑那一格也按输入复用、也逐条跑编出来的代码，但跑的是 research 的实验二进制、比的是产物；这里改坏的是 crates 的源码、判的是点名的测试红不红
#
# 一格（--list 打格名表，--check crates-mutation-rows 只跑它，格名写错退 2）：
# gate-cell: crates-mutation-rows crates/mutations.tsv 每条改坏一处、点名的测试都红，基线在没改坏的副本上绿
# 参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh（写法与判据在它的文件头）；判法、全绿标记与复用判定照改用共用库之前那一版。
# 逐项可点名（.claude/rules/verification.md「门禁的结构」：重型的按变异行点名）：
#   --list-items           逐行打变异表里每一条的变异名、行号与改哪个文件，不起 cargo
#   --item <变异名>         只判点名的几行（给几次取并集；名字原样写，名字里的逗号不拆；不在表里退 2）：换成行号写进这一趟自己的选行文件，
#                          走共用模块的选行模式（与双机分片的驱动同一个入口：不问整道的复用判定、不调双机驱动、不写全绿标记，照常写按条记录）；
#                          选中的全抓到这一格判绿（模块退 3，这里换成 0，成功行写明只判了这几行）。
# 双机分片的驱动起的那两份（环境里带 GATE_MUTATION_ROW_SELECTION）不经格汇总，照驱动的约定退 3 / 1（见下面「只判一部分」）。
#
# 判据：`crates/mutations.tsv` 里每一条变异（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名）成形——六段，
# 替换文可以空（删掉原文：改坏时原文换成空串），其余五段一段都不许空；
# 原文在文件里恰好命中一次，必须红的测试名是一个用例名，原文与替换文里只有 \n 这一种反斜杠转义，
# 文件落在每片拷的范围之内（就是 stage-inputs.tsv 本阶段那一行登记的路径）：这几样派活之前判，判法是共用模块的 row_problems，
# 门禁 code-source-discipline 经 static-check 静态判的是同一份（2026-09-28 之前这里要六段一段都不空、code-source-discipline 放过空段，删行变异在 code-source-discipline 绿、到这里判红）；
# 点名的测试在没改坏的副本上恰好对上一个用例、结果是 ok（基线，派变异之前按批跑）；
# 把仓拷到临时目录、改坏那一处、跑点名的测试，那条测试必须判红；跑完还原再下一条。
# 一条锚点腐化、一条指到拷贝范围之外、一条基线不绿、一条没红，整道红。
# 判法、派活、按条记录与全绿标记都在 research/scripts/crates-mutation-rows.py（门禁 code-source-discipline 静态判拷贝范围用的是同一份），这里只接它：
#   它的文件头写全了拷贝范围、底座指纹、行键、记录放哪、几时复用、弄坏开关；这一份写为什么与几样上限怎么定。
#
# 为什么：show-me-test.md 要「存进仓的变异清单，交给门禁反复复跑」——commit message 里的叙述只被读一次。
# 2026-09-16 发布 B 三方第二轮攻方腿把「空闲独立维护」那处改法整个撤回，全仓零判红零警告：
# 改法本身没有任何东西守着。这张表让每一处三方打中之后的改法都留一条「改回去它就红」的变异，撤回时这里先响。
#
# 分片并发跑（command-safety.md「一个脚本里的检测项，能并行就并行」）：每条变异都要起 cargo 编译 + 跑测试，
# 彼此不依赖，而串行时一条约 8 秒，整张表的挂钟按条数乘上去（条数现数：grep -cvE '^(#|$)' crates/mutations.tsv）。并发的障碍是同一篇里写着的「共用一份可写状态」——
# 所有变异改同一份源码副本、共用一个 CARGO_TARGET_DIR（cargo 的文件锁会把它们重新串行化）。
# 所以每个分片各给一份源码副本与一个编译目录。同一个工作进程连着改同一个 crate，增量编译的命中率才稳：派活按 crate 归堆（久的先派那几条除外，见下）。
#
# 工作进程数取下面两项的最小值、至少 1（crates-mutation-rows.py 的 worker_count_for）；实际开几个、各项给几个、卡在哪一项，打在 stderr（stdout 要与进程数无关，见下一段）：
#   ① 核数的一半、至多 16；设了 GATE_MUTATION_WORKERS 就用它顶掉 ①（设成 1 就是原来的串行跑法）；
#   ② 这一趟要现跑的条数（没有作数的抓到记录的行）。
#   内存不在这里收：每条经 research/scripts/run-with-memory-cap.sh 跑，它起跑之前先判整机放不放得下（slice 已占的加这一条要的，不超过 slice 的总上限才起），
#   放不下的在它那里排队，几条合起来撞顶也只在 slice 里杀（records/2026-09-16-subagent拆分提案.md 第四十节第 30 行）。按可用内存收进程数只看得见开跑那一刻，
#   看不见别的重活同时来抢，治标不治本（用户 2026-09-25 定）。这一道自己在包装之外的进程（Python 父进程与工作进程、包装的 bash）不在 slice 里，
#   从包装的余量里出：2026-09-25 量进程池工作进程每个约 12 MiB、父进程约 15 MiB。
# 活是**动态领的**（谁先跑完谁再领下一条），不按条数预先切片：`singlefs-harness` 那几条点名的是几十秒的重测试，
# 等分之下拿到它们的那一片会独自拖住整道（实测：7 片跑完、最后 1 片又跑了 4 分钟）。
# ⚠️ **输出与进程数、复用了多少都无关**：判定行按变异表的行号排序再打印到 stdout，GATE_MUTATION_WORKERS=1 与 =8 的 stdout 要逐字相同，
# 这一条是并发化之后重新证明它还红得出来的那一半（command-safety.md「改成并行之后要重新证明它红得出来」）；复用的行打的判定与现跑的逐字相同。
# 进度、复用几条、前移几条实时打到 stderr，不进 stdout，不参与那条比对。
#
# 编译产物放 ${GATE_MUTATION_TARGET_DIR:-${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-crates-mutation-target}（跨轮复用，第一次要整编一遍）；
# 分片各自的编译目录是它加 -w<片号>，各自冷编译一次（不从它拷种子，见 prepare_shard）。放 GATE_CROSS_RUN_TMPDIR 不放 TMPDIR：
# 门禁每轮给阶段一个私有 TMPDIR，跑完剩下的判红并删掉（command-safety.md「测试镜像一律放临时目录」），放进去每轮都要从头编 16 份。
#
# 每条变异（与每一批基线）一个它自己的 TMPDIR（建在这一轮源码副本的总目录底下），传给那一条的 cargo 与测试进程；那一条结束时不论退出码是几
# （含限时、撞顶被杀）都整个删掉，删之前数下里面有几个文件、占盘多少，合计报在「临时目录」那一行，被杀的那几条另报在各自的判定里。
# 被杀的测试 Drop 守卫不跑，它建的镜像留在 TMPDIR 里：只靠收尾时删，一轮下来能堆几百 G（records/2026-09-16-subagent拆分提案.md 第四十节第 42 行）。
# 全部判完再把总目录底下列一遍，还剩哪一条的目录就整道判红。弄坏开关 GATE_MUTATION_BREAK=keepmutationtmp（只给证红用）拿掉删除那一步。
#
# 每条变异的 cargo test 放进内存上限里跑（research/scripts/run-with-memory-cap.sh：systemd 的临时 scope，MemoryMax=<上限>、MemorySwapMax=0），
# 撞上限只杀这一条的进程，不把整机拖进 OOM（2026-09-25 一条无界分配的变异两次把整机拖进 OOM，records/2026-09-16-subagent拆分提案.md 第四十节第 28 行）。
#   上限从三处取，先到先用：变异表里单起一行「# 每条变异的内存上限：<上限>」（判别力样本用它把上限压到 512M）、GATE_MUTATION_MEMORY_MAX、默认 4G。
#   默认 4G 的依据：本机 60 GiB 内存，本地模型服务连同会话常驻约 12 GiB（2026-09-25 `free -g` 的 used 列是 12），
#   同时可能有 3 件重活 ⇒ 每件 16 GiB；这一道同时开好几个工作进程，4G 给正常的编译与测试留余量。2026-09-25 量过编译那一半：
#   HEAD 的四个 crate 全部测试目标在 4G 上限、2 个并行编译下从零编得过，匿名内存峰值 0.60 GiB（每 0.2 秒取一次样）；点名的测试跑起来要多少没量，
#   包装的峰值表（research/scripts/memory-peaks.tsv）跑过一遍之后按条记着。换机器要重算（这些数只在本机成立）。
#   几条同时跑合起来放不放得下由包装排队判、slice 的总上限兜底，这里不按内存收进程数。
#   撞了这一条自己的上限（包装退 250）记「内存撞顶」：这条破坏让被测代码无界分配，点名的测试没来得及红，不算抓到也不算没红，单列一栏、整道判红。
#   被总上限挤掉（包装退 254：整个 slice 满了、内核在 slice 里挑了这一条杀）记「被总上限挤掉」，排队等满包装的等待上限还放不下（包装退 252）记「排不上没跑」：
#   两栏都不是这条变异的结论，单列、整道判红，出路是重跑。
#   带上限的 scope 或 slice 的总上限起不来、设不上（没有用户级 systemd、D-Bus 连不上）就整道判红，一条都不跑，不退回无上限去跑。
#
# 每条变异的 cargo test 限时（交给包装：RUN_WITH_MEMORY_CAP_TIME_LIMIT，给 scope 设 RuntimeMaxSec，从起跑算、不算排队的时间；限时里含这一条要做的重编译）：
#   秒数从三处取，先到先用：变异表里单起一行「# 每条变异的超时秒数：<秒>」（判别力样本用它压到 20 秒）、GATE_MUTATION_TIMEOUT、默认 1800。
#   到点 systemd 给 scope 里每个进程（cargo 与测试进程）发 TERM，TIMEOUT_KILL_GRACE_SECONDS 秒还不退再发 KILL，包装按 scope 的 Result=timeout 退 253。
#   不在包装外面套 timeout：排队的时间会算进限时里，排得久的变异会被误判成超时。
#   超过限时记「超时」：这条破坏让被测代码不终止了，或者点名的测试在并发下就要跑这么久。不算抓到、不算没红，与「内存撞顶」「没红」「无效」分开单列一栏、各自计数，整道判红。
#   默认 1800 秒的依据：点名随机历史测试的变异在 16 个工作进程并发时，一条卡在那个测试文件上超过 6–8 分钟（没记到跑完的挂钟），串行单次 1.5–3 分钟
#   （.claude/kb/checks-owed.md 的 C457（被测试自己的线程池与外层进程池叠加超订阅），2026-09-21 在副本上量的）；四个 crate 的全部测试目标在 4G 上限、
#   2 个并行编译下冷编译 23–25 秒（2026-09-25 量）。1800 秒约是 8 分钟的 4 倍，给没量到的尾巴留余地；代价是一条真挂死的变异让一个工作进程多占 30 分钟。
#   这些数只在本机成立，换机器要重算。
#
# 基线（2026-09-28 加）：点名的测试在没改坏的副本上本来就红（读不到副本里没有的文件、本来就坏），改坏之后照样红，原先记「抓到」、其实什么都没证明
#   （litmus/ 登记进拷贝范围之前，点名 crates/singlefs-harness/tests/publish_order_matches_litmus.rs 那一类读 CARGO_MANIFEST_DIR/../../litmus 的测试的变异行就是这样）。
#   所以派变异之前按批在没改坏的副本上先跑一遍点名的测试：批的键与过滤词怎么取见 crates-mutation-rows.py 文件头「基线」；同样经内存包装、同样的上限与限时。
#   不是恰好一个 ok 的行记「基线不绿」：不算抓到、不算没红、它的变异不跑，整道判红。「计数：」那一行的格式不动（mutation-triage 照栏读），基线另起一行「基线：…」报。
# 按条记录与续跑（2026-09-28 加）：每条判完立刻写一条按条记录（被判那棵树的 git common-dir 下 singlefs-mutation-rows/<底座指纹>/），下一趟只跑没有作数记录的行：
#   往变异表追加一行只跑那一行，跑到一半被杀下一趟接着跑剩下的；登记输入里别的任何一份变了（被测源码、Cargo 清单、litmus/、阶段脚本、共用模块、工具链）整张重跑。
#   只有「抓到」与基线 ok 复用；SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 一律不复用。复用几条、现跑几条只写 stderr 与全绿标记。
#   判绿之后照旧写这一道这批输入的全绿标记（admission.py stage-marker-write 那一格，整轮门禁与下一趟按它复用整道），写不成只报不红。
# 派活次序：上一次用时 ≥ GATE_MUTATION_LONG_ROW_SECONDS（默认 300）秒或上一次超时的行先派、按用时从长到短，其余照按 crate 归堆的次序；
#   用时表按「这一行六段原样的哈希」查、跨底座指纹沿用，用时分排队与跑两段记；stderr 头一行报前移了几条。一批基线判完，它 ok 的行随即进变异的队。
# 只判一部分（双机分片的驱动用）：GATE_MUTATION_ROW_SELECTION=<文件>（一行一个变异表行号）只判这些行（它们所在的基线批照跑），照常写按条记录，
#   不写全绿标记、不问复用整道的判定（stage-must-run.sh 那一问是整张表的），末行写明「只判了选中的 N 行，不是整道结论」；选中的全抓到退 3
#   （门禁汇总记失败，不会被当成通过），有红退 1。驱动起的这一种不经格汇总，整道照这个约定退。另一台写的记录用 python3 research/scripts/crates-mutation-rows.py import 导进本机，逐条核底座指纹与行键。
# 双机分片：只在本地多机配置写了 ENABLE_ACROSS_MACHINES=1 时开（用户 2026-09-28 定「双机和gpu都需要参数开启」「如果不设置参数也不会主动开启」；
#   开关只在判法 research/scripts/layer0-shard-configuration-check.sh 里定义：退 0 双机开，退 3 双机关，退 1 判不过），不再认环境变量 GATE_MUTATION_ACROSS_MACHINES。
#   整道跑（没设 GATE_MUTATION_ROW_SELECTION）、没设 SINGLEFS_GATE_FULL=1 与 GATE_MUTATION_START_OVER=1
#   （这两个开关下最后那一趟不复用，分出去的白跑）、本地配置（与层 0 分片同一份）判法退 0 时，
#   先调 research/scripts/mutation-shard-run.sh 把没有作数记录的行分到两台各判一份、第二台的记录核过导进本机（步骤与弄坏开关在它文件头），
#   驱动的输出进 stderr；驱动退出之后（退几都一样）这里照常整张判一遍：命中记录的复用、缺的现跑，stdout 与单机逐字相同，全绿标记照旧由本机写。
#   没分的在 stderr 报一行「双机分片：没分（为什么）」，判法退 3 时写「双机：关」。选行模式下（驱动起的那两份、--item）不调驱动。
#   弄坏开关 MUTATION_SHARD_RUN_BREAK=no-final-pass（驱动的自证用）：调完驱动就按驱动的退出码退，不再整张判。
#   弄坏开关 MUTATION_SHARD_RUN_BREAK=environment-opens（驱动的自证用）：GATE_MUTATION_ACROSS_MACHINES=1 照旧把双机打开、不看配置的开关
#   （驱动自证「配置没开双机、环境变量写 1：checker-tier-crates-mutation-replay 报双机：关、不调驱动」那一格判错）。
#
# 判别力：fixtures/checker-tier-crates-mutation-replay.sh/red（red、green 两份各放 .gate-cells 点名 crates-mutation-rows）的变异表把上限压到 512M、限时压到 20 秒：加一条把 1 MiB 的缓冲改成 1536 MiB 的变异，必须报「内存撞顶」；
#   加一条把步长 1 改成 0、循环永远走不到上界的变异，必须报「超时」；计数行里没红、无效、内存撞顶、超时各数各的。
#   那一条点名的测试先往 TMPDIR 写一个 4 KiB 的文件再走：被限时杀掉之后，它的判定里必须报「临时目录已整个删掉（删前留有 1 个文件」，
#   0 个文件说明 TMPDIR 没传到测试进程；「临时目录」那一行必须报 4 条都删掉了（基线不绿的两行不跑变异）。
#   点名的测试读 fixture-data/answer.txt（样本仓里有、不在拷贝范围里）的那一行，与点名只写名字、两个模块各有一个同名用例的那一行，都必须报「基线不绿」
#   （基线那一行报 4 行 ok、2 行不绿）；GATE_MUTATION_BREAK=nobaseline 跳过基线，读副本外文件的那一行误判成抓到，这份样本必须判错。
#   GATE_MUTATION_BREAK=keepmutationtmp 拿掉删除那一步，red 与 green 两份样本都必须判错。
# green 同样压到 512M，留着一行撤掉了的「# 给整机留的内存余量：1T」：工作进程数那一行必须报开 5 个（五条、核数的一半多于 5）、卡在「这一趟要现跑的条数」，
#   不许再按内存收进程数；五条正常变异必须照旧红在点名的测试上，其中一条改的是 litmus/sample.litmus（litmus/ 登记进拷贝范围之前这一条让整道在派活前判红），
#   一条替换文空着、删掉 add_twice 里第二次相加（删行变异：派活之前判成形、改坏时原文换成空串）；
#   基线那一行报 5 行都 ok，「临时目录」那一行必须报 5 条都删掉了，stderr 头一行报复用 0 条。
#   GATE_MUTATION_BREAK=emptyreplacementrefused 让替换文空着也判不成形，这份样本必须判错。
#   两份样本的 setup.sh 在样本目录里 git init（算底座指纹要 git 列文件，按条记录写在那棵树的 git common-dir 里）。
# 复用、续跑、只改一行只跑那一行、改源码全部重跑、久的先派、选行、导入这几样要连跑好几趟，在 research/scripts/crates-mutation-rows.py --selftest 里
#   用假 cargo 跑（门禁 code-tooling 替它复跑），弄坏开关 basewithoutsource、timingsignored、importunchecked、nobaseline、emptyreplacementrefused 下各有一格必须判错。
#
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh [项目根]                                整张表
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --list                                  逐行打格名与判什么，不起 cargo
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --list-items [项目根]                    逐行打变异表里可点名的变异名与行号
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --item <变异名> [--item <变异名>…] [项目根]   只判点名的几行
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

# --item：点名的变异名换成变异表的行号，写进这一趟自己的选行文件，交给共用模块的选行模式（与双机分片的驱动同一个入口）
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

# ── crates-mutation-rows：整段判法照改用共用库之前那一版，只包进一个格函数 ──
cell_crates_mutation_rows() {
# 先问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
# 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
# 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
# 这一道原先没有任何范围判定，每趟跑满：2026-09-22 实测，一批 27 个路径里 crates/ 零个，它照跑不误。
# 只判选中的几行（GATE_MUTATION_ROW_SELECTION，驱动设的或 --item 换成的）时不问：那一问是整张表的，选中的几行按条记录复用。
if [[ -z "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
  source "$STAGE_REPOSITORY/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
  stage_run_or_skip "$STAGE_REPOSITORY/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")"
fi
# 前提（cargo）登记在 .claude/gate.d/stage-inputs.tsv 本阶段那一行第三列，经门禁与实验共用的准入模块判，没齐判红（不退 77）。
# 读的是本阶段所在那一份仓的登记表：判别力样本的目录里没有登记表，前提照样按真表判。
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
# 开不开只听本地多机配置：判法退 0 双机开、退 3 双机关（配置没写 ENABLE_ACROSS_MACHINES=1）、退 1 判不过；不认环境变量开。
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
# 拷贝范围、锚点、基线、派活、按条记录、判定的打印与全绿标记都在共用模块的 run_stage 里；这里 import 它、交出被判的仓根。
# 模块按名字登记进 sys.modules：进程池的工作进程靠这个名字找回要跑的函数。
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
# --item 点名的几行：选中的全抓到时模块退 3（选行模式的约定，给驱动看的），这里是人点名要跑的那几行，全抓到就是这一格绿；
# 模块末行已经写明「只判了选中的 N 行，不是整道结论」，全绿标记也不写
if [[ -n "$item_selection_file" && "$module_exit" == 3 ]]; then
  echo "  ✓ 点名的 ${#named_mutations[@]} 条变异都抓到了（只判了这几行，不是整道结论；全绿标记没写）"
  exit 0
fi
exit "$module_exit"
}

# 双机分片的驱动起的那两份（环境里带 GATE_MUTATION_ROW_SELECTION，不是 --item 换成的）照驱动的约定退：选中的全抓到退 3、有红退 1，
# 不经格汇总（汇总把 0、77 之外的码一律并成 1，驱动就分不出「全抓到」与「有红」）
if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" && -z "$item_selection_file" ]]; then
  ( cell_crates_mutation_rows )
  exit $?
fi
stage_cells_run "$ROOT"
