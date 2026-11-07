#!/usr/bin/env bash
# admission: always 判的是此刻被判的那棵工作树上每条变异的单元键与广谱变异判定库（prune-server 里那一份），判定库不在 git 树里，上一次的结论不替这一次作保
# run-condition: command python3
# task-server-gpu-arguments: *   不指定范围的一趟先起驱动剪枝跑，驱动末尾补带 gpu 特性的那一趟，清场与分卡由服务在最外层办（task-server/task_server_lib.py 文件头「GPU」）；--check gate-verdict 只读库、不起驱动
# gate-stage: 广谱变异（提交门禁：此刻源码上每一条变异都有单元键相同的结论、没有一条没红）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: crates-mutation-replay.sh 它按 crates/mutations.tsv 手写的几十行逐条改坏 crates/、自己起 cargo、要点名的测试红；这里不自己起 cargo，起广谱变异驱动剪枝跑（cargo 归驱动）再读 cargo-mutants 全量清单在广谱变异判定库里的结论，两张表、两个判据、两个判定库都不同
# gate-similar: code-source-discipline.sh 它的 mutation-tables 格静态判变异表成形、锚点唯一；这里不读变异表，读驱动 --summary 的判定行与判定库
# gate-similar: formal-verification.sh 它起 Verus 判证明，每条判定按单元键在它自己的库里复用；这里只读广谱变异驱动的判定，并进去会让两件不相干的事共用一份判法
# gate-similar: crash-point-replay.sh 它逐条核崩溃枚举用例的崩溃状态块、缺块报本次未跑；这里有没结论的判红不报本次未跑（提交前要跑完才能提交）
#
# gate-cell: gate-verdict 先起驱动剪枝跑（按单元键沿用、沿用不了的现判，判到此刻清单的全部条；--check gate-verdict 指定范围时不起、只读库），再读驱动 --summary 的 BROAD_MUTATION_GATE 判绿：此刻源码上每一条变异都有单元键相同的结论（要 GPU 那一趟的条两趟合起来有结论）、没有一条没红、没有单元缺清单，判定行、汇总行、明细与退出码互相对得上；驱动没跑成判红；判定库服务不可用退 75
# 参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh。
#
# 判什么：此刻源码上每一条变异都有单元键相同的结论、没有一条没红（gate-verdict 格）；结论键是被变异的函数单元与它调到的单元各自的指纹、原文、替换文、
#   抓它的测试的单元闭包键（没红的是能碰到它的测试各自的闭包键）、工具链、cargo-mutants / nextest / llvm 版本、构建环境的生效值、解析出的依赖版本、判法单元指纹
#   （规则在 scripts/mutation/mutation-prune.py 文件头，判定库在 scripts/mutation/broad-mutation.py 文件头「判定库」「键」：prune-server 里那一份，族 broad-mutation）。
#   判定库里没有「一版」：不比暂存树、不比提交，没有 library-version-is-staged-tree 格，没有 --marker-check；也没有整道标记，不问 stage-must-run.sh，每趟照判。
#   记红、撤绿、下趟现判（驱动文件头「判定行」末段）：这一道判红的是没红（missed > 0）与没结论；没红的条在驱动的剪枝跑里不沿用，每趟现判（同一把键更早的抓到、无效撤掉，
#   单独一行没红、先抓到后判没红都一样；要 GPU 那一趟的条看合起来的状态）；抓到、无效照键沿用。这一道只读 --summary：状态本来就取最后一个定论，没红照没红数，不另设格，红的这一支照抄 MISSED 明细。
#   不指定范围（不带 --check）：这一道自己先起驱动 scripts/mutation/broad-mutation.py --source <被判的根>，不带选条参数（剪枝跑：每条按单元键沿用、沿用不了的现判，判到此刻清单的全部条，
#   末尾补 GPU 那一趟，没卡的机器上那几条是本次未跑），驱动退非 0（跑不成、基线红、锁被占、环境坏了……）这一格判红、不往下读 --summary；退 75（判定库服务不可用）这一格退 75。
#   驱动不设内存上限、不限时，整条门禁是统一任务服务的一个任务，撞上限、卡住由服务停；中断后重跑同一条命令就是续跑。
#   指定范围（--check gate-verdict）：不起驱动，只读 --summary，判库里此刻有没有全部条的结论；开关 driver-not-run 让不指定范围的一趟也不起驱动。
#   被判的那棵工作树就是第一个参数（主工作区，或 gate.sh --staged 的临时 worktree：内容是 HEAD 加暂存区）。
# gate-verdict：驱动 --summary --source <被判的根>（不给 --home：判定库只有服务里那一份，库目录取驱动的缺省）在此刻的源码上照清单按单元存的那张表复原清单、现算每条的单元键、查判定库，
#   打恰好一行 BROAD_MUTATION_SUMMARY、一行 BROAD_MUTATION_GATE；verdict=green、missed=0、without_conclusion=0、unlisted_units=0、concluded=total、total>0，退出码 0；
#   verdict=red 而退 8 是驱动判红，照抄它的 NO_CONCLUSION（带作废时变了的单元）/ MISSED / UNLISTED 明细；
#   判定行与退出码对不上（verdict 不是 green / red、写 red 而退 0）、计数自相矛盾（判绿而 missed、without_conclusion 或 unlisted_units 不是 0，
#   NO_CONCLUSION、MISSED、REFUSED、UNLISTED 明细行数与计数不同、清单 0 条）、汇总行与判定行各数对不上（两行的 total、refused、unlisted_units，
#   汇总行抓到加没红加无效对判定行的 concluded、汇总行只过了第一段加驱动错加没跑对 without_conclusion、汇总行六栏之和对它的 total），都判红。
#   拒算的条（refused=：单元键里有认不出的形态、或抓到它的测试认不出）照报条数、逐条 REFUSED，有键相同的现判结论就算有结论（驱动每趟都现判它们）。
#   GPU 那一趟（C489，驱动文件头「GPU 那一趟」）：CPU 那一趟没红、又有带 gpu 特性的测试够得着的条，
#   要 GPU 那一趟的结论才算有结论：驱动 --summary 的汇总行与判定行都带 gpu_needed（要这一趟的条数）、gpu_reused（库里键相同的 GPU 结论）、gpu_not_run（还没有 GPU 结论：没卡、
#   清场没过、没来得及跑，本次未跑不算没红也不算通过）、gpu_without_conclusion（GPU 趟驱动错）；后两种条在 without_conclusion 里、NO_CONCLUSION 明细的理由写 gpu-…。
#   这一格判：判定行要有这四个数（没有的是旧驱动，判红）；gpu_needed = gpu_reused + gpu_not_run + gpu_without_conclusion；后两个之和不超过 without_conclusion（判绿的驱动
#   不许把没有 GPU 结论的条当有结论数；GPU 趟没结论判红，红的那一支照抄 NO_CONCLUSION 明细）；汇总行的四个数与判定行的相同。
#   驱动不设内存上限、不限时（驱动文件头「内存与限时」）：整条驱动是统一任务服务的一个任务，撞上限、卡住由服务停，在途那一条不落行；
#   汇总行六栏：抓到、没红、无效、只过了第一段、驱动错、没跑。判定库里认不出的结局驱动 --summary 报对不上、退 1，这一格照退 1 判红。
#   判定库服务不可用（驱动 --summary 退 75：连不上、停了、忙到上限，scripts/prune-server/prune_server_client.py 文件头「退出码」）：这一格退 75，
#   共用库把整道记成「prune-server 不可用」（不是红，也不算通过）；驱动退 3（服务的配置不对、回应解不开）照「退了 0、8 之外的码」判红。
#
# 样本（fixtures/broad-mutation.sh/<格名>-<情形>/，各放 .gate-cells 点名一格；共享门禁的 stage-selftest.sh 跑）：
#   样本根放了标记 .broad-mutation-sample（只供样本的开关：被判的根不是这道阶段所在的仓、也没设 GATE_STAGED_FROM 时才认），里面一行 scenario=<族>:<情形>；
#   开格之前起 lib/broad-mutation-samples.py <驱动> <样本根> <情形> 造现场（两族的造法与各情形写在那份文件头）：summary 族写 .broad-sample/summary.sh，
#   格起它（打录好的输出、退录好的码）；units 族在样本根写一棵小工作区、用此刻驱动的函数把清单与判定记进判定库（库目录 .broad-sample/home 只放运行目录；
#   样本档在临时 prune-server 底下起，判定另占临时服务里一份命名空间，不碰两台共用的那一份），再改源码，格起真的驱动 --summary（--home .broad-sample/home、--toolchain-from .broad-sample/rustc-vv.txt、
#   --tools-from .broad-sample/tool-versions.txt）。units 族要连得上判定库服务（主 agent 照 scripts/prune-server/prune_server_client.py 文件头「命令行」
#   --with-temporary-service 那一条在临时服务底下起样本档；连不上的样本照造不成判红）。
#   gate-verdict-units-* 那几份是新键的样本（controls 路第 5 节、words 路 N27）：-green 照记的那一份源码判绿；-callee-changed-red 被调的 helper 改了函数体，
#   只有调它的那一条要现判（NO_CONCLUSION 带「变了：改了 helper」）、helper 自己那个单元没记清单（UNLISTED），别的条照样有结论；
#   -comment-only-green 只加注释，零条要现判；-unlisted-red 新加一个函数，那个单元没记清单；
#   -driver-error-red 一条只有 driver-error 行（没结论），驱动判红退 8。
#   gate-verdict-driver-* 两份（样本根放 .broad-sample/driver.sh 当驱动、summary.sh 当 --summary，标记里不写 scenario）：-driver-not-run-red 驱动退 1（没跑成）而 summary.sh 录的是全绿，
#   这一格判红、不往下读；-driver-run-green 驱动退 0、summary.sh 全绿，判绿、输出里有驱动那一趟的一句。没放 driver.sh 的样本（上面几族）不起驱动，只读 summary。
#   gate-verdict-* 录好输出的那几份：-green 全有结论；-missed-red 一条没红、退 8；-no-conclusion-red 三条没结论、退 8；-silent-driver-red 判定行写 green、退 0，而 missed=1；
#   -line-missing-red 只有汇总行、退 0；-red-exit-zero-red 判定行写 red、计数与明细一致、退 0；-detail-count-red 判定行 green、计数全零，却列了 2 行 NO_CONCLUSION；
#   -empty-list-red 清单 0 条、判定行 green；-unknown-verdict-red verdict=pending、退 8；-totals-mismatch-red 汇总行 total=5091、not_run=5085，判定行 total=6 写 green；
#   -refused-count-red 两行都写 refused=1、一行 REFUSED 都没列；-unlisted-lie-red 判定行写 green 而 unlisted_units=2；
#   -service-unavailable 驱动打客户端连不上的那两行、退 75：这一格退 75，整道退 75、汇总写 prune-server 不可用。
#   gate-verdict-gpu-* 那几份（GPU 那一趟）：summary 族录好的输出——-green 要 GPU 趟的 3 条都有库里键相同的结论；-no-conclusion-red 要 GPU 趟的 3 条里 2 条本次未跑（没卡）、退 8；
#   -driver-error-red 1 条 GPU 趟驱动错（没结论）、退 8；-silent-driver-red 判定行写 green、退 0，without_conclusion=0 而 gpu_not_run=2（把没有 GPU 结论的条当有结论数了）；
#   -counts-red 判定行 green、gpu_needed=3 却只有 1 条复用；-fields-missing-red 判定行没有 gpu_ 四个数（旧驱动）；-summary-mismatch-red 汇总行与判定行的 GPU 趟计数不同；
#   gate-verdict-units-gpu-* 两份（units 族，真驱动 --summary 在带 验证包的小工作区上）：-units-gpu-green CPU 那一趟没红的一条被带 gpu 特性的测试抓到（GPU 判定行是抓到，
#   测试键是此刻算的）；-units-gpu-no-conclusion-red 同一条 CPU 没红、GPU 趟还没有结论（NO_CONCLUSION 理由 gpu-not-run，gpu_not_run=1）。
# 弄坏开关（只给样本证红用，样本自检时设进环境；CHECKER_TIER_BROAD_MUTATION_BREAK 可以逗号分隔几项一起开，下面逐项写）。逐项写开关名、弄坏哪一处，
# 下一行起写打开后判错的样本（fixtures/broad-mutation.sh/ 下的目录名，括号里是判错的样子）：
#   CHECKER_TIER_BROAD_MUTATION_BREAK=summary-red-passes  驱动退 8、判定行写 red 也算绿，判定行与退出码对不上不判
#       gate-verdict-missed-red、gate-verdict-no-conclusion-red、gate-verdict-red-exit-zero-red、gate-verdict-units-callee-changed-red、gate-verdict-units-unlisted-red、
#       gate-verdict-gpu-no-conclusion-red、gate-verdict-gpu-driver-error-red、gate-verdict-units-gpu-no-conclusion-red、gate-verdict-units-driver-error-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=verdict-exit-unchecked  判定行写 red 而退 0 不判
#       gate-verdict-red-exit-zero-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=verdict-value-unchecked  verdict 不是 green / red 不判
#       gate-verdict-unknown-verdict-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=counts-unchecked  不核判定行的计数与明细
#       gate-verdict-silent-driver-red、gate-verdict-detail-count-red、gate-verdict-empty-list-red、gate-verdict-refused-count-red、gate-verdict-unlisted-lie-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=detail-count-unchecked  不核明细行数与计数
#       gate-verdict-detail-count-red、gate-verdict-refused-count-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=empty-list-passes  清单 0 条不判
#       gate-verdict-empty-list-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=unlisted-passes  判绿时不看 unlisted_units
#       gate-verdict-unlisted-lie-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=summary-gate-totals-unchecked  不核汇总行与判定行的各数
#       gate-verdict-totals-mismatch-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=gate-line-optional  没有判定行而退 0 算绿
#       gate-verdict-line-missing-red（照红，缺「0 行 BROAD_MUTATION_GATE，各该恰好一行」那条 want）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=gpu-fields-optional  判定行没有 gpu_ 四个数也认（旧驱动），连同不核汇总行与判定行的 GPU 趟计数
#       gate-verdict-gpu-fields-missing-red、gate-verdict-gpu-summary-mismatch-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=gpu-counts-unchecked  不核 gpu_needed 等于三项之和
#       gate-verdict-gpu-counts-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=gpu-pending-unchecked  不核 gpu_not_run 加 gpu_without_conclusion 不超过 without_conclusion
#       gate-verdict-gpu-silent-driver-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=gpu-totals-unchecked  不核汇总行的 GPU 趟计数与判定行的相同
#       gate-verdict-gpu-summary-mismatch-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=driver-not-run  不起驱动，只读库、不判到全部条
#       gate-verdict-driver-not-run-red（判绿）
#   CHECKER_TIER_BROAD_MUTATION_BREAK=summary-unavailable-as-red  驱动 --summary 退 75（判定库服务不可用）也照「退了 0、8 之外的码」判红、退 1
#       gate-verdict-service-unavailable（判红，退 1 不是期望的 75）
#   STAGE_CELLS_BREAK=red-swallowed  归共用库：每份红样本的格按绿记、整道退 0（汇总与 --check 归共用库）
#       gate-verdict-detail-count-red、gate-verdict-empty-list-red、gate-verdict-line-missing-red、gate-verdict-missed-red、gate-verdict-no-conclusion-red、
#       gate-verdict-red-exit-zero-red、gate-verdict-refused-count-red、gate-verdict-silent-driver-red、gate-verdict-totals-mismatch-red、gate-verdict-unknown-verdict-red、
#       gate-verdict-unlisted-lie-red、
#       gate-verdict-gpu-counts-red、gate-verdict-gpu-fields-missing-red、gate-verdict-gpu-no-conclusion-red、gate-verdict-gpu-silent-driver-red、
#       gate-verdict-gpu-summary-mismatch-red、gate-verdict-gpu-driver-error-red、
#       gate-verdict-units-callee-changed-red、gate-verdict-units-gpu-no-conclusion-red、gate-verdict-units-driver-error-red、
#       gate-verdict-units-unlisted-red、gate-verdict-driver-not-run-red（判绿）
#
#   bash .claude/gate.d/broad-mutation.sh [项目根]
#   bash .claude/gate.d/broad-mutation.sh --list              逐行打格名与判什么，不起驱动、不起 rustc
#   bash .claude/gate.d/broad-mutation.sh --check <格名>      只跑点名的格；格名写错退 2
#   bash .claude/gate.d/broad-mutation.sh --tool-versions     打 rustc -Vv、cargo nextest、cargo mutants 与 llvm-tools 的 llvm-profdata 版本（只问版本，不跑变异）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ broad-mutation.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
DRIVER="$STAGE_REPOSITORY/scripts/mutation/broad-mutation.py"
SAMPLE_BUILDER="$STAGE_REPOSITORY/.claude/gate.d/lib/broad-mutation-samples.py"
SAMPLE_MARKER=".broad-mutation-sample"
SAMPLE_DIRECTORY=".broad-sample"
RUN_DRIVER_HOWTO="重跑同一条命令（这一道自己先起驱动剪枝跑，中断后重跑就是续跑；没卡的机器上要 GPU 那一趟的条是本次未跑，到有卡的机器上重跑）"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell gate-verdict cell_gate_verdict "先起驱动剪枝跑（按单元键沿用、沿用不了的现判，判到此刻清单的全部条；--check gate-verdict 指定范围时不起、只读库），再读驱动 --summary 的 BROAD_MUTATION_GATE 判绿：此刻源码上每一条变异都有单元键相同的结论（要 GPU 那一趟的条两趟合起来有结论）、没有一条没红、没有单元缺清单，判定行、汇总行、明细与退出码互相对得上；驱动没跑成判红；判定库服务不可用退 75" \
  "驱动没跑成的：照上面驱动打的 ✗ 与 → 修（退 6 基线不绿或环境坏了、退 4 锁被占着、退 2 用法或环境不对）；没有结论的、有单元没记清单的：$RUN_DRIVER_HOWTO（整条经 task-server 交，内存上限与卡住由服务管，撞了上限服务放大重排）；没红的：加一条能杀它的测试（改 crates/ 归实现员）再让驱动判那一条，确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（只那几条不再判）；--summary 退 1（判定库或清单表对不上，含认不出的结局）照驱动打的 → 修"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
root_argument=""
tool_versions_requested=0
while (($#)); do
  case "$1" in
    --tool-versions) tool_versions_requested=1; shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--tool-versions 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不了项目根 $ROOT"; echo "     → 怎么办：第一个参数给被判的仓根（不给就判这道阶段所在的仓）。"; exit 2; }
ROOT="$(pwd)"

if ((tool_versions_requested)); then
  if command -v rustc >/dev/null 2>&1; then
    rustc -Vv 2>&1 || echo "rustc -Vv 退出码非 0"
    # llvm-tools：与驱动不给 --llvm-bin 时同一处（rustc 的 sysroot 下 lib/rustlib/<host>/bin）
    llvm_sysroot="$(rustc --print sysroot 2>/dev/null)"
    llvm_host="$(rustc -Vv 2>/dev/null | sed -n 's/^host: //p')"
    if [[ -x "$llvm_sysroot/lib/rustlib/$llvm_host/bin/llvm-profdata" ]]; then
      "$llvm_sysroot/lib/rustlib/$llvm_host/bin/llvm-profdata" --version 2>&1 | head -n 3
    else
      echo "llvm-profdata：$llvm_sysroot/lib/rustlib/$llvm_host/bin 下没有（rustup component add llvm-tools）"
    fi
  else
    echo "rustc：PATH 上没有"
  fi
  if command -v cargo >/dev/null 2>&1; then
    cargo nextest --version 2>&1 | head -n 1
    cargo mutants --version 2>&1 | head -n 1
  else
    echo "cargo：PATH 上没有"
  fi
  exit 0
fi

sample_mode=0
if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" && -f "$SAMPLE_MARKER" ]]; then sample_mode=1; fi
# 判定库只有 prune-server 里那一份（驱动文件头「判定库」）：不给 --home，库目录取驱动的缺省；样本根下面换样本自己的库目录（只放运行目录，判定的命名空间由临时 prune-server 给）
summary_command=(python3 "$DRIVER" --summary --source "$ROOT")
# 不指定范围的一趟先起的驱动：不带选条参数的剪枝跑（驱动文件头第一条），判到此刻清单的全部条；样本里换成样本自己的 driver.sh，没放的不起
driver_command=(python3 "$DRIVER" --source "$ROOT")
if ((sample_mode)); then
  driver_command=()
  if [[ -f "$SAMPLE_DIRECTORY/driver.sh" ]]; then driver_command=(bash "$ROOT/$SAMPLE_DIRECTORY/driver.sh"); fi
  LIBRARY_HOME="$ROOT/$SAMPLE_DIRECTORY/home"
  scenario="$(sed -n 's/^scenario=//p' "$SAMPLE_MARKER" | head -n 1)"
  if [[ -n "$scenario" ]] && ! env -u BROAD_MUTATION_BREAK python3 "$SAMPLE_BUILDER" "$DRIVER" "$ROOT" "$scenario"; then
    echo "  ✗ 样本现场没造成：lib/broad-mutation-samples.py 情形 $scenario 退非 0"
    echo "     → 怎么办：看上面的报错；units 族载入 $DRIVER 调 Library、Judgement、canonical_items、record_list_cache，驱动里这几样改了名或参数就照着改那份造法；"
    echo "       报 prune-server 客户端连不上的，照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条在临时服务底下起样本档。"
    exit 1
  fi
  if [[ -f "$SAMPLE_DIRECTORY/summary.sh" ]]; then
    summary_command=(bash "$ROOT/$SAMPLE_DIRECTORY/summary.sh")
  else
    summary_command=(python3 "$DRIVER" --summary --home "$LIBRARY_HOME" --source "$ROOT"
                     --toolchain-from "$ROOT/$SAMPLE_DIRECTORY/rustc-vv.txt" --tools-from "$ROOT/$SAMPLE_DIRECTORY/tool-versions.txt")
  fi
fi

# ── 格 gate-verdict ──
cell_gate_verdict() {
local output driver_output driver_exit
output="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
# 不指定范围：先起驱动剪枝跑，判到全部条再读 --summary。--check 指定范围、弄坏开关 driver-not-run、样本没放 driver.sh 时不起
if ((STAGE_CELLS_CHECK_GIVEN)); then
  echo "  · 指定了 --check：不起驱动，只读判定库里此刻有没有全部条的结论（不判到全部条）"
elif [[ ",${CHECKER_TIER_BROAD_MUTATION_BREAK:-}," != *,driver-not-run,* ]] && ((${#driver_command[@]})); then
  driver_output="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${output:?}"; exit 1; }
  echo "  · 先起驱动剪枝跑（按单元键沿用、沿用不了的现判，判到此刻清单的全部条）：${driver_command[*]}"
  if env -u BROAD_MUTATION_BREAK "${driver_command[@]}" > "$driver_output" 2>&1; then driver_exit=0; else driver_exit=$?; fi
  if ((driver_exit == 75)); then
    echo "  ✗ 驱动退 75：prune-server 不可用（判定库服务连不上、停了或忙到上限），这一格没判；它的输出末几行："
    tail -n 12 "$driver_output" | sed 's/^/       | /'
    echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；服务回来之后重跑同一条命令。"
    rm -f "${output:?}" "${driver_output:?}"
    exit 75
  fi
  if ((driver_exit != 0)); then
    echo "  ✗ 驱动剪枝跑退 $driver_exit（不是 0）：没判到此刻清单的全部条，不往下读 --summary；它的输出末几行："
    tail -n 12 "$driver_output" | sed 's/^/       | /'
    echo "     → 怎么办：照上面驱动打的 ✗ 与 → 修（退 6 基线不绿或疑似环境坏了、退 4 库目录的锁被占着、退 2 用法或环境不对、退 9 运行目录没删掉、退 143 被停）；修好之后$RUN_DRIVER_HOWTO。"
    rm -f "${output:?}" "${driver_output:?}"
    exit 1
  fi
  echo "  · 驱动剪枝跑退 0：$(grep -E '^  ✓ ' "$driver_output" | tail -n 1 | sed 's/^  //')"
  rm -f "${driver_output:?}"
fi
if env -u BROAD_MUTATION_BREAK "${summary_command[@]}" > "$output" 2>&1; then summary_exit=0; else summary_exit=$?; fi
env -u BROAD_MUTATION_BREAK python3 - "$output" "$summary_exit" "${CHECKER_TIER_BROAD_MUTATION_BREAK:-}" "$RUN_DRIVER_HOWTO" <<'PY'
import sys
sys.dont_write_bytecode = True
output_path, exit_text, broken_text, howto_run = sys.argv[1:5]
broken = set(filter(None, broken_text.split(",")))
summary_exit = int(exit_text)
with open(output_path, encoding="utf-8", errors="replace") as handle:
    lines = handle.read().splitlines()
def fields_of(prefix):
    found = [line for line in lines if line.startswith(prefix + " ")]
    return [dict(token.split("=", 1) for token in line.split()[1:] if "=" in token) for line in found]
def tail():
    return "\n".join("       | " + line for line in lines[-12:])
def detail(prefix):
    return [line for line in lines if line.startswith(prefix + "\t")]
summaries, gates = fields_of("BROAD_MUTATION_SUMMARY"), fields_of("BROAD_MUTATION_GATE")
no_conclusion, missed_lines, refused_lines, unlisted_lines = detail("NO_CONCLUSION"), detail("MISSED"), detail("REFUSED"), detail("UNLISTED")
if summary_exit == 75 and "summary-unavailable-as-red" not in broken:
    print("  ✗ 驱动 --summary 退 75：prune-server 不可用（判定库服务连不上、停了或忙到上限），这一格没判；它的输出末几行：")
    print(tail())
    print("     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；服务回来之后只重跑这一格（不改走本地文件，判定库没有那条路）。")
    sys.exit(75)
if summary_exit not in (0, 8):
    print(f"  ✗ 驱动 --summary 退 {summary_exit}（不是判绿的 0、也不是判红的 8）：判定库或清单表对不上、或驱动没跑成；它的输出末几行：")
    print(tail())
    print("     → 怎么办：照上面驱动打的 → 办（退 1 是判定库或清单表对不上，退 2 是用法或库目录不对，退 3 是判定库服务的配置不对或回应解不开）；修好再跑这一格。")
    sys.exit(1)
if len(summaries) != 1 or (len(gates) != 1 and not ("gate-line-optional" in broken and not gates and summary_exit == 0)):
    print(f"  ✗ 驱动 --summary 打了 {len(summaries)} 行 BROAD_MUTATION_SUMMARY、{len(gates)} 行 BROAD_MUTATION_GATE，各该恰好一行：判不了提交门禁；输出末几行：")
    print(tail())
    print("     → 怎么办：没有判定行是驱动还是旧的那一份：scripts/mutation/broad-mutation.py 从 git 取今天的那一份；行数多了是输出被拼进了别的东西。")
    sys.exit(1)
summary = summaries[0]
gate = gates[0] if gates else {"verdict": "green", "total": summary.get("total", "0"), "concluded": summary.get("total", "0"), "missed": "0",
                                "without_conclusion": "0", "refused": summary.get("refused", "0"), "unlisted_units": summary.get("unlisted_units", "0")}
def number(key, source=gate):
    try:
        return int(source.get(key, ""))
    except ValueError:
        return None
total, concluded, missed, without = number("total"), number("concluded"), number("missed"), number("without_conclusion")
refused, unlisted = number("refused"), number("unlisted_units")
if None in (total, concluded, missed, without, refused, unlisted):
    print(f"  ✗ 判定行的计数读不出：{gate}")
    print("     → 怎么办：驱动打的判定行坏了（要 total、concluded、missed、without_conclusion、refused、unlisted_units 各一个整数），scripts/mutation/broad-mutation.py 从 git 取今天的那一份再跑。")
    sys.exit(1)
# GPU 那一趟（驱动文件头「GPU 那一趟」）：要这一趟的条数 = 库里键相同的结论 + 本次未跑（还没有结论）+ 没结论（驱动错）；后两种在 without_conclusion 里
gpu_keys = ("gpu_needed", "gpu_reused", "gpu_not_run", "gpu_without_conclusion")
gpu = {key: number(key) for key in gpu_keys}
if None in gpu.values():
    if "gpu-fields-optional" not in broken:
        print(f"  ✗ 判定行没有 GPU 那一趟的计数（要 {'、'.join(gpu_keys)} 各一个整数）：{gate}")
        print("     → 怎么办：这是驱动的毛病（旧的那一份不报 GPU 趟，判不了「要 GPU 那一趟的条都有结论」）：scripts/mutation/broad-mutation.py 从 git 取今天的那一份再跑；GPU 趟的写法在驱动文件头「GPU 那一趟」。")
        sys.exit(1)
    gpu = dict.fromkeys(gpu_keys, 0)
verdict = gate.get("verdict")
if verdict == "red" and summary_exit == 8 and "summary-red-passes" not in broken:
    print(f"  ✗ 广谱变异提交门禁判红：此刻源码上 {total} 条里 {without} 条沿用不了也没现判、{missed} 条没红、{unlisted} 个单元没记清单"
          f"；沿用不了的里要 GPU 那一趟而没有结论的 {gpu['gpu_not_run'] + gpu['gpu_without_conclusion']} 条（本次未跑 {gpu['gpu_not_run']}、没结论 {gpu['gpu_without_conclusion']}）")  # gate-lint:summary
    for line in (missed_lines + no_conclusion + unlisted_lines)[:30]:
        print("     " + line.replace("\t", " "))  # gate-lint:detail
    if len(missed_lines) + len(no_conclusion) + len(unlisted_lines) > 30:
        print(f"     ……另有 {len(missed_lines) + len(no_conclusion) + len(unlisted_lines) - 30} 行，全文在驱动 --summary 的输出里")  # gate-lint:detail
    print(f"     → 怎么办：没有结论的、有单元没记清单的：{howto_run}（整条经 task-server 交，内存上限与卡住由服务管，撞了上限服务放大重排；理由写 gpu-… 的是 CPU 那一趟没红又要 GPU 那一趟的条：在有卡的机器上重跑同一条命令，"
          "GPU 趟接在最后自己清场再跑，没卡的机器上它们是本次未跑，不算没红也不算通过）；"
          "没红的：加一条能杀它的测试（改 crates/ 归实现员）再让驱动判那一条，确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（只那几条不再判）。")
    sys.exit(1)
expected_exit = 0 if verdict == "green" else 8
inconsistent = []
if verdict not in ("green", "red") and "verdict-value-unchecked" not in broken:
    inconsistent.append(f"verdict={verdict}（只认 green / red）")
elif summary_exit != expected_exit and "summary-red-passes" not in broken and "verdict-exit-unchecked" not in broken:
    inconsistent.append(f"判定行写 verdict={verdict}，驱动却退 {summary_exit}")
if "counts-unchecked" not in broken:
    if verdict == "green" and (missed or without or concluded != total):
        inconsistent.append(f"判定行写 green，计数却是 total={total} concluded={concluded} missed={missed} without_conclusion={without}")
    if verdict == "green" and unlisted and "unlisted-passes" not in broken:
        inconsistent.append(f"判定行写 green，却有 unlisted_units={unlisted} 个单元没记清单（清单不全，判不了「每一条都有结论」）")
    if concluded + without != total:
        inconsistent.append(f"concluded={concluded} 加 without_conclusion={without} 不等于 total={total}")
    if "detail-count-unchecked" not in broken and (len(no_conclusion) != without or len(missed_lines) != missed or number("missed", summary) != missed
                                                  or len(refused_lines) != refused or len(unlisted_lines) != unlisted):
        inconsistent.append(f"明细与计数不同：NO_CONCLUSION {len(no_conclusion)} 行、MISSED {len(missed_lines)} 行、REFUSED {len(refused_lines)} 行、UNLISTED {len(unlisted_lines)} 行，"
                            f"判定行 without_conclusion={without}、missed={missed}、refused={refused}、unlisted_units={unlisted}，汇总行 missed={summary.get('missed')}")
    if total <= 0 and "empty-list-passes" not in broken:
        inconsistent.append("此刻清单 0 条：没有可判的变异，不是通过")
# GPU 那一趟的计数（各自的开关，不跟着上面的 counts-unchecked / summary-gate-totals-unchecked 走）
if "gpu-counts-unchecked" not in broken and gpu["gpu_needed"] != gpu["gpu_reused"] + gpu["gpu_not_run"] + gpu["gpu_without_conclusion"]:
    inconsistent.append(f"gpu_needed={gpu['gpu_needed']} 不等于 gpu_reused={gpu['gpu_reused']} 加 gpu_not_run={gpu['gpu_not_run']} 加 gpu_without_conclusion={gpu['gpu_without_conclusion']}")
if "gpu-pending-unchecked" not in broken and gpu["gpu_not_run"] + gpu["gpu_without_conclusion"] > without:
    inconsistent.append(f"gpu_not_run 加 gpu_without_conclusion 是 {gpu['gpu_not_run'] + gpu['gpu_without_conclusion']}，比 without_conclusion={without} 还多"
                        "（要 GPU 那一趟而没有结论的条在没结论里：判绿的驱动把它们当有结论数了，本次未跑不是通过）")
gpu_in_summary = {key: number(key, summary) for key in gpu_keys}
if "gpu-totals-unchecked" not in broken and "gpu-fields-optional" not in broken and gpu_in_summary != gpu:
    inconsistent.append(f"汇总行的 GPU 趟计数 {gpu_in_summary} 与判定行的 {gpu} 不同")
if "summary-gate-totals-unchecked" not in broken:
    columns = ("total", "caught", "missed", "unviable", "stage_one_only", "driver_error", "not_run", "refused", "unlisted_units")
    counts = {key: number(key, summary) for key in columns}
    if None in counts.values():
        inconsistent.append(f"汇总行的计数读不出（要 {'、'.join(columns)} 各一个整数）：{summary}")
    else:
        decided = counts["caught"] + counts["missed"] + counts["unviable"]
        undecided = counts["stage_one_only"] + counts["driver_error"] + counts["not_run"]
        if counts["total"] != total:
            inconsistent.append(f"汇总行 total={counts['total']}，判定行 total={total}")
        if decided != concluded:
            inconsistent.append(f"汇总行抓到 {counts['caught']} 加没红 {counts['missed']} 加无效 {counts['unviable']} 是 {decided}，判定行 concluded={concluded}")
        if undecided != without:
            inconsistent.append(f"汇总行只过了第一段、驱动错、没跑加起来是 {undecided}，判定行 without_conclusion={without}")
        if decided + undecided != counts["total"]:
            inconsistent.append(f"汇总行六栏加起来 {decided + undecided}，它的 total={counts['total']}")
        if counts["refused"] != refused or counts["unlisted_units"] != unlisted:
            inconsistent.append(f"汇总行 refused={counts['refused']}、unlisted_units={counts['unlisted_units']}，判定行 refused={refused}、unlisted_units={unlisted}")
if inconsistent:
    print("  ✗ 驱动 --summary 的判定行、汇总行、计数、明细与退出码对不上（驱动的提交门禁判法失灵）：")  # gate-lint:summary
    for entry in inconsistent:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：这是驱动的毛病，不是此刻源码的结论：交工具实现员查 scripts/mutation/broad-mutation.py 的 summarise（它的 --selftest 有 summary-gate-silent 那一格）；修好前不提交。")
    sys.exit(1)
print(f"  ✓ 广谱变异提交门禁判绿：此刻源码上 {total} 条每一条都有单元键相同的结论，没红 0 条，没记清单的单元 0 个，拒算 {refused} 条（有键相同的现判结论）；"
      f"要 GPU 那一趟的 {gpu['gpu_needed']} 条都有库里键相同的结论（复用 {gpu['gpu_reused']} 条）")
PY
python_exit=$?
rm -f "${output:?}"
exit "$python_exit"
}

if ( stage_cells_run "$ROOT" ); then stage_exit=0; else stage_exit=$?; fi
exit "$stage_exit"
