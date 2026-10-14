#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要 cargo：缺了就在单测那一格判红、复跑那一格由它调的 scripts/experiments/replay.sh 编实验二进制时判红，不交给 gate.sh 预判；此外只要跑门禁本身就要的 bash、git、python3
# gate-stage: research 工作区的构建与单测；入库的实验数今天还复现得出来吗
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: harness-model-differential-and-scenarios.sh 它也经内存包装跑 release 的 cargo test，但编的是仓根的 crates 工作区、判的是模型对拍那几段；这里编的是 research 工作区，两个工作区互相看不到对方，编译产物不共用，并进来反而要多编一份
# gate-similar: code-source-discipline.sh 它的实验源码纪律那一格静态判 research 的实验二进制与变异表（锚点、每个实验有变异表），不编译、不跑；这里真编、真跑单测与复跑，判的是编得过、跑得出同样的数；mutation-tables-compile 格枚举的表与它同一个来源，逐条施加替换文真编（只 cargo check），判的是每条替换文今天还编得过
# gate-similar: doc-experiments.sh 它判实验页与产物登记（索引行、产物写回、产物落在仓里），读的是 kb 与 research/results/ 的文字；这里把产物重跑一遍逐字节比，读的是实验二进制的输出
# gate-similar: code-tooling.sh 研究脚本的 --selftest 都在它的 research-script-selftests 格跑（不要 research 工作区先编好、经 light-selftest-guard.py 盯着）；这里只编 research 工作区、跑它的单测与复跑，一份研究脚本的自证都不跑
#
# gate-cell: research-unit-tests research 工作区编得过、单测全绿
# gate-cell: wallclock-asserts-registered 实验单测里按挂钟判的断言都有登记位（一行注释），没登记的判红
# gate-cell: experiment-replay 入库的纯模型实验重跑一遍，与 research/results/ 里的产物逐字节相同
# gate-cell: mutation-tables-compile 研究侧每张变异表逐条施加替换文只编译，一条编不过就红
#   research-unit-tests  每次都跑：cargo build --release 编整个 research 工作区，再 cargo test --release。
#                        上游「构建与单测」只编 crates/，kb 里实测结论背后的实验二进制只有这一格编。
#                        登记了的挂钟断言按名跳过（C465：门禁自己的并发让按挂钟判的断言误红）：源码里测试函数上带 // wallclock-assert: 登记位（一行注释，装置不加任何代码），
#                        判法 scripts/gate/wallclock-assert-lint.py --list 逐行打「<文件>TAB<函数名>」，这一格按登记名逐个给 cargo test 传 --skip <函数名>
#                        （cargo test --release -- --skip <名> --skip <名>…；登记 0 条时命令就是 cargo test --release）。不设环境变量、不加 --show-output。
#                        跑完从输出里找 ^test .*<函数名> ... 行：登记的名字出现了判红（登记了却跑了：--skip 没生效）；
#                        每个登记名逐条往 $GATE_NOT_RUN_FILE 报「挂钟断言 <函数名> 本次未跑」，成功句报「跳过挂钟断言 K 条（逐个列名）」，不记它们为绿、不判红。
#                        登记位写坏时判法 --list 退 1，这一格不数、不传 --skip，那一错归 wallclock-asserts-registered 格。
#                        要验这些断言：不带 --skip，在不并发的场合单独跑 cargo test。
#   wallclock-asserts-registered  python3 scripts/gate/wallclock-assert-lint.py：scripts/experiments/*.rs 的 #[test] 里按挂钟判的断言要带 // wallclock-assert: 登记位
#                        （或怕不着负载的 // wallclock-assert-load-safe:），缺了判红、点名文件:行号；带了而断言里没有挂钟量也判红；登记名是别的未登记测试名的子串
#                        （--skip 是子串匹配，会连那一条一起跳过）也判红。判法、登记位写法在判法脚本文件头；
#                        判法立之前就在仓里还没复核的在 .claude/gate.d/wallclock-assert-lag.tsv（只缩不涨），成功句另报条数，并往 $GATE_NOT_RUN_FILE 报一行。
#   experiment-replay    .claude/gate.d/stage-inputs.tsv 里本阶段那一行的路径相对上次整轮全绿那棵暂存树没变就退 77
#                        （为什么按树、按一条 ref、不看工作区，写在 scripts/gate/stage-must-run.sh 文件头）；
#                        再问 change-touches-crates.sh，前缀不含 research/prompts/（冻结证据改不了实验输出）。
#                        replay.sh 退 77（C350：整趟全是计时行的负载关，本次不判）不当红：这一格报「复跑整趟本次不判（负载高）」、往 $GATE_NOT_RUN_FILE 报一行、退 77；别的退出码照旧（非 0 判红）。
#                        派给 scripts/experiments/replay.sh 的是表里的 exact 行减掉 SLOW；SLOW 里的与表里每一行 timing 都逐个列成「没派」。
#                        对不上不一定是代码坏了，也可能是产物该更新，两种都要人判，所以判红。
#                        C8（门禁范围判不出来）还欠按布局算受影响集合，这里按前缀判只是它的粗粒度前身。
#                        要跑时 replay.sh 再按行复用（它文件头「按行复用」：每一行按装置源码、登记、留存产物与工具链算键，键相同、判过的那一行不跑）：
#                        这一格读它末尾那一行 REPLAY_TOTAL，报「复跑：现判 N 行、复用 M 行、已归档只跑不比 K 行」；没有那一行（老的或假的 replay.sh）报复用数取不出，
#                        不写 0。已归档只跑不比的行由 replay.sh 逐行往 $GATE_NOT_RUN_FILE 报。SINGLEFS_GATE_FULL=1 原样带下去（replay.sh 照它只记不读）。
#   mutation-tables-compile  研究侧变异表（scripts/mutation/tables/<名>.tsv）逐张、逐条施加替换文只编译：每张调
#                        scripts/mutation/mutate.sh --compile-only <bin> <名>.rs <表>（判法、cargo 跑什么、退出码都在它文件头「只编不跑」；
#                        每个 cargo 带的 RUSTFLAGS=…--cap-lints warn 在它文件头「deny 警告」，这里不另加）；
#                        有一条 ⏭（编不过：平常跑里就是一条无效变异）整格红，逐条点名表与条目、打第一行 error。
#                        枚举对象与 code-source-discipline 的 mutation-tables 格同一个来源：scripts/mutation/tables/ 下每张 .tsv 对 scripts/experiments/ 下同名的 .rs；
#                        bin 名经 mutate.sh --binary-of <名>.rs 取（与它核对 bin 的读法同一份）。没有同名 .rs 的表（shell 探针、Python 驱动、crates 那一侧的）
#                        取不出（bin、源文件、表）三元组，逐张点名、往 $GATE_NOT_RUN_FILE 报本次未跑、不算绿；一张都判不了退 77。
#                        整格的复用照 experiment-replay：.claude/gate.d/stage-inputs.tsv 里 checker-tier-research-build-and-replay.sh:mutation-tables-compile 那一行
#                        的路径相对上次整轮全绿那棵暂存树没变就退 77（scripts/gate/stage-must-run.sh）。
#                        要跑时再按表复用：键是表、源文件、它 include_str! / include_bytes! / #[path] 引的文件、scripts/experiments/ 的 Cargo.toml、Cargo.lock、lib.rs、
#                        mutate.sh 自身各自的 sha256 与工具链（rustc -vV、cargo -V），判法版本是这份阶段脚本与判定库两份的 sha256；
#                        存法与键的算法在判定库 scripts/crash/lib_verdict_library.py（只记绿，键相同就复用），库在
#                        ${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-mutation-tables-compile-verdicts/；SINGLEFS_GATE_FULL=1 时只记不读。成功那句报「现判 N 张、复用 M 张」。
#                        编译目录一个、跨轮留着：${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-mutation-tables-compile-target/，各表串行增量编。
#                        挂了 required-features 的 bin 由 mutate.sh 带上 --features；E162 的 rocksdb 要的 BINDGEN_EXTRA_CLANG_ARGS 没设时取
#                        scripts/crash/crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS（与 checker-tier-crash-point-replay 带特性那一趟同一处）。
# 格结构经 lib/stage-cells.sh。
#
# 读的环境变量：
#   GATE_RESEARCH_BUILD_MEMORY_MAX  cargo test 经 scripts/hosts/run-with-memory-cap.sh 跑的内存上限，没设取 8G；谁跑这一道外面都不再包一层。
#                                   包装退 245 或 250–254 不是单测的判定，单独判红、单独给出路。cargo build 不跑编出来的代码，不经包装。
#   GATE_REPLAY_FULL=1              复跑那一格连 SLOW 与 timing 行一起跑
#   SINGLEFS_GATE_FULL=1            由 stage-must-run.sh 读（不复用整格）；mutation-tables-compile 格另不读判定库（照样记）
#   GATE_CROSS_RUN_TMPDIR           mutation-tables-compile 格的判定库与编译目录放它下面（gate.sh 设它），没设取 TMPDIR
#
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh [项目根]                                三格都跑
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests [项目根]     只跑单测那一格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check wallclock-asserts-registered [项目根]  只跑挂钟断言登记位那一格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --list-items [项目根]                    逐行打 replay.sh 表里的实验号、exact 还是 timing、在不在 SLOW 里，不编、不跑
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check experiment-replay --item E142 [项目根]   只复跑点名的实验
#     --item 给几次取并集，timing 与 SLOW 也照跑；成功行写明只跑了点名的几个，不是整格结论；实验号不在表里退 2
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check mutation-tables-compile [项目根]  只跑变异表只编译那一格
#
# 样本：fixtures/checker-tier-research-build-and-replay.sh/ 下十一份，各放 .gate-cells 只点名它判的那一格：
#   research-unit-tests-red    research 工作区里有一条单测红着的小仓，判红
#   research-unit-tests-green  一条单测全绿、scripts/ 下一份脚本都没有，判绿，成功行报一个测试批次
#   green                      带假 replay.sh 的小仓，表里一行快的 exact、一行 SLOW 里的 exact、一行 SLOW 里只有 timing 的；
#                              快的那行派给 replay.sh，另两行逐个列进「没派」
#   red                        表一行都没有，判红
#   mutation-tables-compile-red    最小 research 工作区（一个 bin、warnings = "deny"），一张两条的表里一条替换文编不过（引了没有的名字），判红、点名表与那一条
#   mutation-tables-compile-green  同形的工作区，两条替换文都编得过，另有一张没有同名 .rs 的表（逐张列进本次未跑）；
#                              样本根放 .mutation-tables-compile-sample/run-twice：判定库放这一格自己的临时目录、读，接连判两趟，第二趟那一张复用
#   mutation-tables-compile-deny-green  同形的工作区（warnings = "deny"），一条替换文只让一个函数再没人调、留下死代码：deny 下编不过，
#                              mutate.sh 给每个 cargo 带的 --cap-lints warn 压成警告，判绿（两条都编得过）
#   wallclock-asserts-registered-red    scripts/experiments/ 下一份 .rs 里有一个按挂钟判的断言（.elapsed() 比大小）的测试，没登记位，判红、点名文件:行号
#   wallclock-asserts-registered-green  同形的一份 .rs：一个带 // wallclock-assert: 的测试、一个带 // wallclock-assert-load-safe: 的测试，判绿并报登记 1 条、load-safe 1 条
#   replay-load-skipped        带假 replay.sh（退 77）的小仓：复跑那一格认成本次未跑，整道退 77、输出里有「复跑整趟本次不判（负载高）」；改前这一格对 77 退 1
#   experiment-replay-reused   带假 replay.sh 的小仓，末行打一行 REPLAY_TOTAL（现判 1 行、复用 2 行、已归档只跑不比 1 行）：这一格照它报复用数，判绿
#   样本里判定库与编译目录都是这一格自己的临时目录，跑完删；只有 run-twice 那份读判定库。mutation-tables-compile 三份要真 cargo（cargo check 一个小 bin）；
#   wallclock-asserts-registered 两份只读样本里的 .rs，不起 cargo。
# 弄坏开关归共用库：STAGE_CELLS_BREAK=red-swallowed 让三份红样本整道退 0；=marker-ignored 让每份样本三格都跑，
#   research-unit-tests-green 里复跑那一格对着没有表的 replay.sh 判红。
# 这一道自己的弄坏开关 RESEARCH_BUILD_AND_REPLAY_BREAK=<项>（逐项写开关名、弄坏哪一处、打开后哪份样本红）：
#   invalid-rows-ignored  mutation-tables-compile 不看 mutate.sh --compile-only 打的 ⏭，退 1 的表也当绿（改前没有这一格的样子）
#       mutation-tables-compile-red（期望退 1、输出里有「[两条之二_引了没有的名字] 编不过」，实得退 0）
#   no-cap-lints  mutation-tables-compile 起 mutate.sh --compile-only 时，PATH 前面放一个垫片 cargo（这一格自己的临时目录里）：它先 env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS 再交给真 cargo，
#       mutate.sh 带的 --cap-lints warn 不管放在 RUSTFLAGS 还是 CARGO_ENCODED_RUSTFLAGS 里都到不了 rustc（改前的跑法）。
#       不靠设 CARGO_ENCODED_RUSTFLAGS：mutate.sh 看见它就把 cap 接进去，那一招顶不掉了。mutate.sh 自己的 MUTATE_BREAK 只在它的 --selftest 起的子进程里认，门禁里设了它 mutate.sh 退 2（要保留），所以样本上证红用这一个
#       mutation-tables-compile-deny-green（期望退 0，实得退 1、输出里有「[两条之二_留死代码_不再调用加倍] 编不过：error: function `double` is never used」）
#   skip-filter-dropped  research-unit-tests 起 cargo test 时不传 --skip：登记了的挂钟断言在门禁的负载下照跑，输出里出现它们的 test … ok / FAILED 行，这一格红
#       「登记了却跑了：--skip 没生效」。样本档里 research-unit-tests 两份样本的小仓没有登记位，登记数是 0、没有 --skip 可丢，看不出这一项，样本上证不了红；只靠真仓真跑：
#       SINGLEFS_HEAVY_TESTS=commit RESEARCH_BUILD_AND_REPLAY_BREAK=skip-filter-dropped bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests
#       必须退 1、输出里有「登记了却跑了」。wallclock-asserts-registered 格不起 cargo，不受影响。
#   replay-reuse-unreported  experiment-replay 不读 replay.sh 的 REPLAY_TOTAL 行，复用数不报（改前的样子）
#       experiment-replay-reused（输出里找不到「复跑：现判 1 行、复用 2 行、已归档只跑不比 1 行」）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
STAGE_REPOSITORY="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_NAME="$(basename "$0")"
source "$STAGE_DIR/lib/stage-cells.sh"

stage_cell research-unit-tests cell_research_unit_tests "research 工作区编得过、单测全绿" \
  "先修好编不过或红着的单测（kb 里的实测结论全靠它们背书）；改完只重跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests"
stage_cell wallclock-asserts-registered cell_wallclock_asserts_registered "实验单测里按挂钟判的断言都有登记位（一行注释），没登记的判红" \
  "按判红点名的文件:行号，在 #[test] 与 fn 之间写一行 // wallclock-assert: <量的是什么>（只写注释，装置代码不动）；负载再高也不会红的写 // wallclock-assert-load-safe: <为什么>；登记名撞上别的测试名的，改名或把那一条也登记；改完只重跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check wallclock-asserts-registered"
stage_cell experiment-replay cell_experiment_replay "入库的纯模型实验重跑一遍，与 research/results/ 里的产物逐字节相同" \
  "对不上先判是代码坏了还是产物该更新（两种都要人判）；只重跑红的那几个实验：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check experiment-replay --item <实验号>"
stage_cell mutation-tables-compile cell_mutation_tables_compile "研究侧每张变异表逐条施加替换文只编译，一条编不过就红" \
  "改那一行替换文（编得过、仍改被测行为），或装置改回能编；表归实验执行员改。单张核：在 scripts/experiments/ 里 bash ../mutation/mutate.sh --compile-only <bin> <源文件> <表>；改完只重跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check mutation-tables-compile"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

list_items=0; root_argument=""; named_experiments=()
while (($#)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then
        echo "  ✗ --item 后面缺实验号"
        echo "     → 怎么办：写成 --item <实验号>（例 --item E142），几个实验就给几次；可点的号用 --list-items 看。"
        exit 2
      fi
      named_experiments+=("$2"); shift 2 ;;
    --item=*) named_experiments+=("${1#--item=}"); shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--list-items、--item <实验号> 与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" || exit 2
R=scripts/experiments
REPLAY_SCRIPT="scripts/experiments/replay.sh"

# replay.sh 表里的行：编号|二进制|参数|产物|exact 或 timing。门禁靠这个形状取行，改表格式要同时改这里。
replay_rows_of_kind() { sed -n "s/^\(E[0-9]*\)|[^|]*|[^|]*|[^|]*|$1\$/\1/p" "$REPLAY_SCRIPT" 2>/dev/null; }
SLOW=(E9 E17 E20 E21)

if ((list_items)); then
  if [[ ! -f "$REPLAY_SCRIPT" ]]; then
    echo "  ✗ 没有 $REPLAY_SCRIPT：列不出可点名的实验"
    echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"
    exit 1
  fi
  listed=0
  while IFS= read -r experiment; do
    [[ -n "$experiment" ]] || continue
    kinds="$(sed -n "s/^${experiment}|[^|]*|[^|]*|[^|]*|\(exact\|timing\)\$/\1/p" "$REPLAY_SCRIPT" | sort -u | paste -sd, -)"
    slow_note=""
    for s in "${SLOW[@]}"; do [[ "$s" == "$experiment" ]] && slow_note="（慢，默认不跑）"; done
    [[ "$kinds" == timing ]] && slow_note="（只有 timing 行，默认不跑）"
    printf '%s\t%s%s\n' "$experiment" "$kinds" "$slow_note"
    listed=$((listed + 1))
  done < <({ replay_rows_of_kind exact; replay_rows_of_kind timing; } | awk '!seen[$0]++')
  ((listed)) || { echo "  ✗ 从 $REPLAY_SCRIPT 里一行都没读到：表的格式变了？"; echo "     → 怎么办：这里靠 '编号|二进制|参数|产物|exact 或 timing' 这个形状取行，改表格式要同时改这里。"; exit 1; }
  exit 0
fi

if ((${#named_experiments[@]})); then
  mapfile -t known_experiments < <({ replay_rows_of_kind exact; replay_rows_of_kind timing; } | awk '!seen[$0]++')
  unknown_experiments=()
  for experiment in "${named_experiments[@]}"; do
    found=0
    for known in ${known_experiments[@]+"${known_experiments[@]}"}; do [[ "$known" == "$experiment" ]] && found=1; done
    ((found)) || unknown_experiments+=("$experiment")
  done
  if ((${#unknown_experiments[@]})); then
    echo "  ✗ --item 点名的实验不在 $REPLAY_SCRIPT 的表里：${unknown_experiments[*]}"
    echo "     → 怎么办：实验号照 bash .claude/gate.d/${STAGE_NAME} --list-items 列的写（表里有 ${#known_experiments[@]} 个）。"
    exit 2
  fi
fi

# ── research-unit-tests ──
cell_research_unit_tests() {
  local out rc b n lint listing listing_rc name joined
  local registered_wallclock=() skipped_wallclock=() ran_wallclock=() skip_arguments=()
  local MEMORY_CAP_RUNNER RESEARCH_BUILD_MEMORY_MAX
  [[ -f "$R/Cargo.toml" ]] || { echo "  ! 没有 $R/Cargo.toml，这一格无对象可判"; exit 77; }
  command -v cargo >/dev/null || {
    echo "  ✗ 没有 cargo，装不了就没法验 research 的证据"
    echo "     → 怎么办：装 Rust 工具链（curl https://sh.rustup.rs -sSf | sh），"
    echo "               或把已装的 cargo 放进 PATH。跳过这一步等于 research/ 的数字没人验过。"
    exit 1; }
  MEMORY_CAP_RUNNER="$STAGE_REPOSITORY/scripts/hosts/run-with-memory-cap.sh"
  RESEARCH_BUILD_MEMORY_MAX="${GATE_RESEARCH_BUILD_MEMORY_MAX:-8G}"
  if [[ ! "$RESEARCH_BUILD_MEMORY_MAX" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    echo "  ✗ GATE_RESEARCH_BUILD_MEMORY_MAX 写成了「$RESEARCH_BUILD_MEMORY_MAX」，内存包装不认（或认成字节数）：这一格没跑"
    echo "     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G"
    exit 1
  fi
  # 先编整个 research 工作区的 release 产物：复跑那一格的 replay.sh 编的就是这一份，跑到那一格时 cargo 判它新鲜、不再编
  # cargo 的编译与测试线程不吃 cargo 自己的缺省（核数）：经 capped.sh default 起，取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）
  out="$(cd "$R" && bash "$STAGE_REPOSITORY/scripts/hosts/capped.sh" default cargo build --release 2>&1)"
  rc=$?
  if (( rc != 0 )); then
    grep -E '^error' <<<"$out" | head -8 | sed 's/^/     /'
    echo "  ✗ research 编不过（cargo build --release 退出码 $rc）"
    echo "     → 怎么办：先修好——kb 里的实测结论全靠这些实验二进制背书，编不过就等于那些数字今天没有来源。"
    exit 1
  fi
  # C465：登记了哪几条按挂钟判的断言以判法的 --list（一行「<文件>TAB<函数名>」）为准；登记位写坏了（判法退 1）时数不出，不在这里查，那一错归 wallclock-asserts-registered 格
  lint="$STAGE_REPOSITORY/scripts/gate/wallclock-assert-lint.py"
  if [[ -f "$lint" ]] && command -v python3 >/dev/null; then
    listing="$(python3 "$lint" --list . 2>/dev/null)"; listing_rc=$?
    case "$listing_rc" in
      0) mapfile -t registered_wallclock < <(cut -f2 <<<"$listing" | sed '/^$/d' | sort -u) ;;
      1|77) ;;
      *) echo "  ✗ python3 scripts/gate/wallclock-assert-lint.py --list 退 $listing_rc（0 列出来了、1 登记位有红、77 无对象）：数不出登记了哪几条按挂钟判的断言"
         echo "     → 怎么办：单独跑 python3 scripts/gate/wallclock-assert-lint.py --list . 看它的输出，修好再重跑这一格。"
         exit 1 ;;
    esac
  fi
  # 登记了的测试按名 --skip；弄坏开关 skip-filter-dropped（文件头）：不传，它们照跑
  if ((${#registered_wallclock[@]} > 0)) && ! research_build_broken skip-filter-dropped; then
    skip_arguments=(--)
    for name in "${registered_wallclock[@]}"; do skip_arguments+=(--skip "$name"); done
  fi
  out="$(cd "$R" && RUN_WITH_MEMORY_CAP_KEY="gate checker-tier-research-build-and-replay: cargo test --release (research)" \
    bash "$MEMORY_CAP_RUNNER" "$RESEARCH_BUILD_MEMORY_MAX" bash "$STAGE_REPOSITORY/scripts/hosts/capped.sh" default cargo test --release ${skip_arguments[@]+"${skip_arguments[@]}"} 2>&1)"
  rc=$?
  if (( rc == 245 || (rc >= 250 && rc <= 254) )); then
    tail -8 <<<"$out" | sed 's/^/     /'
    echo "  ✗ research 的构建与单测没判出结果：内存包装 scripts/hosts/run-with-memory-cap.sh 退 $rc（上限 $RESEARCH_BUILD_MEMORY_MAX），这一次的输出不算判定"
    echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一格要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；"
    echo "               251、252、254 先跑 bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑。"
    echo "               253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑。245 是同一段时间内核报了显存不够：nvidia-smi 看谁在用卡，等它跑完再重跑这一格。"
    exit 1
  fi
  if (( rc != 0 )); then
    echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
    grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
    echo "     → 怎么办：先修好——kb 里的实测结论全靠它们背书，编不过就等于那些数字今天没有来源。"
    exit 1
  fi
  # 条数只用一个 awk 数（「test result: ok. N passed; …」的第 4 段）：串几道工具的话哪一环失败，条数就静默变成「?」
  b="$(grep -c '^test result: ok' <<<"$out")"
  n="$(awk '/^test result: ok\./ { passed += $4 } END { print passed + 0 }' <<<"$out")"
  [[ "$n" =~ ^[0-9]+$ ]] || n="没数出来（awk 输出「$n」）"
  # 一个测试批次都没有：不记通过，退 77
  if (( b == 0 )); then
    echo "  ! 本次无对象可判：cargo test 退 0，但输出里一个「test result: ok」批次都没有，research 的单测没判"
    exit 77
  fi
  # 登记了的按挂钟判的断言：--skip 之后输出里不该再有它们的 test … ok / FAILED 行（被 --skip 滤掉的测试不打任何一行）
  for name in ${registered_wallclock[@]+"${registered_wallclock[@]}"}; do
    if grep -Eq -- "^test (.*::)?${name} \\.\\.\\. " <<<"$out"; then ran_wallclock+=("$name"); else skipped_wallclock+=("$name"); fi
  done
  if ((${#ran_wallclock[@]} > 0)); then
    echo "  ✗ 登记了的挂钟断言 ${#ran_wallclock[@]} 条（${ran_wallclock[*]}）在 cargo test 的输出里以 test … 行出现：登记了却跑了，--skip 没生效，它们在门禁的负载下照跑了"
    echo "     → 怎么办：看起 cargo test 的命令里有没有 -- --skip <函数名>（判法 python3 scripts/gate/wallclock-assert-lint.py --list . 列登记名）、"
    echo "               登记名是不是被别的写法拼错；--skip 是子串匹配，函数名要和登记的一字不差。"
    exit 1
  fi
  joined=""
  for name in ${skipped_wallclock[@]+"${skipped_wallclock[@]}"}; do
    echo "  ! 本次未跑：挂钟断言 $name 本次未跑（cargo test --skip：门禁自己的并发会让它误红，要验在不并发的场合单独跑 cargo test）"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 research-unit-tests 格：挂钟断言 $name 本次未跑" >> "$GATE_NOT_RUN_FILE"; fi
    joined+="${joined:+、}$name"
  done
  if ((${#skipped_wallclock[@]})); then
    echo "  ✓ research 构建与单测全绿：$b 个测试批次、共 $n 个单测；跳过挂钟断言 ${#skipped_wallclock[@]} 条（$joined），它们本次未跑，不记为绿"
  else
    echo "  ✓ research 构建与单测全绿：$b 个测试批次、共 $n 个单测"
  fi
}

# ── wallclock-asserts-registered ──
cell_wallclock_asserts_registered() {
  local lint="$STAGE_REPOSITORY/scripts/gate/wallclock-assert-lint.py" out rc lagging
  command -v python3 >/dev/null || { echo "  ✗ 没有 python3，跑不了挂钟断言登记位的判法"; echo "     → 怎么办：装 python3 或把它放进 PATH（门禁本身也要它）。"; exit 1; }
  [[ -f "$lint" ]] || { echo "  ✗ 没有 $lint：这一格的判法在它里面"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一格按红记。"; exit 1; }
  out="$(python3 "$lint" . 2>&1)"; rc=$?
  printf '%s\n' "$out"
  case "$rc" in
    0)
      lagging="$(grep -o '另有 [0-9]* 条在滞后表' <<<"$out" | grep -o '[0-9]*')"
      if [[ -n "$lagging" && -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 wallclock-asserts-registered 格：滞后表里 $lagging 条挂钟断言还没复核登记" >> "$GATE_NOT_RUN_FILE"; fi
      exit 0 ;;
    1|77) exit "$rc" ;;
    *)
      echo "  ✗ python3 scripts/gate/wallclock-assert-lint.py 退 $rc（0 全对、1 有红、2 用法错、77 无对象）：判不出结果"
      echo "     → 怎么办：照上面它的输出查；单独跑 python3 scripts/gate/wallclock-assert-lint.py . 看。"
      exit 1 ;;
  esac
}

# ── experiment-replay ──
# replay.sh 退 0 往下走；退 77（整趟全是本次不判的计时行：负载过高，它自己的汇总里有逐行说明）认成本次未跑，这一格退 77；别的退出码判红
run_replay() {
  local replay_rc replay_log replay_total judged reused archived_run_only
  replay_log="$(mktemp)"
  bash "$REPLAY_SCRIPT" "$@" | tee "$replay_log"
  replay_rc="${PIPESTATUS[0]}"
  # 按行复用（replay.sh 文件头「按行复用」）：照它末尾那一行 REPLAY_TOTAL 报现判、复用与已归档只跑不比的行数；没有那一行报取不出，不写 0
  replay_total="$(grep '^REPLAY_TOTAL ' "$replay_log" | tail -n 1)"
  rm -f -- "${replay_log:?}"
  if [[ -n "$replay_total" ]] && ! research_build_broken replay-reuse-unreported; then
    judged="$(sed -n 's/.* judged=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    reused="$(sed -n 's/.* reused=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    archived_run_only="$(sed -n 's/.* archived_run_only=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    echo "  · 复跑：现判 ${judged:-取不出} 行、复用 ${reused:-取不出} 行、已归档只跑不比 ${archived_run_only:-取不出} 行（replay.sh 按行复用：键相同、判过的那一行不跑，见它文件头「按行复用」）"
  elif ! research_build_broken replay-reuse-unreported; then
    echo "  ! 复跑：复用数取不出（replay.sh 没打 REPLAY_TOTAL 行），这一趟现判几行、复用几行这一格报不出"
  fi
  case "$replay_rc" in
    0) return 0 ;;
    77)
      echo "  ! 复跑整趟本次不判（负载高）：replay.sh 退 77，判的全是本次不判的行，一行都没比；不算通过也不算红"
      if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 experiment-replay 格：复跑整趟本次不判（负载高）" >> "$GATE_NOT_RUN_FILE"; fi
      exit 77 ;;
    *) exit 1 ;;
  esac
}

cell_experiment_replay() {
  local e s skip dup ALL_EXACT=() ALL_TIMING=() FAST=() SKIPPED=()
  if ((${#named_experiments[@]})); then
    # 点名的实验：不问整道的复用判定（那一问是整张表的），replay.sh 按实验自己的准入照旧判「输入没变」
    run_replay "${named_experiments[@]}"
    echo "  ✓ 只复跑了点名的 ${#named_experiments[@]} 个实验：${named_experiments[*]}（其中准入判「输入没变」而没跑的，见上面 replay.sh 汇总里那一行）；这不是整格结论"
    exit 0
  fi
  source "$STAGE_REPOSITORY/scripts/gate/stage-run-or-skip.sh" || { echo "  ✗ source 不进 scripts/gate/stage-run-or-skip.sh，判不出这一格要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一格按红记。"; exit 1; }
  stage_run_or_skip "$STAGE_REPOSITORY/scripts/gate/stage-must-run.sh" "复用上一次整轮全绿的判定" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一格读哪几条路径见 .claude/gate.d/stage-inputs.tsv 里 ${STAGE_NAME} 那一行。" -- "$ROOT" "$STAGE_NAME"
  stage_run_or_skip "$STAGE_REPOSITORY/scripts/gate/change-touches-crates.sh" "这次改动没碰它判的东西" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；判据与前缀见 scripts/gate/change-touches-crates.sh。" \
    -- "$ROOT" crates/ scripts/experiments/ scripts/ research/results/
  # 跑哪些、没跑哪些都从 replay.sh 的表里现算，不写死清单：写死的清单漏掉新加的实验，「没跑」那句就成了假话
  # （.claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」）。
  mapfile -t ALL_EXACT < <(replay_rows_of_kind exact)
  if ((${#ALL_EXACT[@]} == 0)); then
    echo "  ✗ 从 scripts/experiments/replay.sh 里一行都没读到——表的格式变了？"
    echo "    → 本阶段靠 '编号|二进制|参数|产物|exact' 这个形状取行，改表格式要同时改这里。"
    exit 1
  fi
  mapfile -t ALL_TIMING < <(replay_rows_of_kind timing)
  for e in "${ALL_EXACT[@]}"; do
    skip=0
    for s in "${SLOW[@]}"; do [[ "$e" == "$s" ]] && skip=1; done
    if ((skip)); then SKIPPED+=("$e（慢）"); else FAST+=("$e"); fi
  done
  # timing 行默认一行都不跑：逐个进「没跑」清单。去重只比已经进了清单的那几行（同一个编号既有 exact 行又有 timing 行时），
  # 不比 SLOW：SLOW 里的编号若只有 timing 行，拿 SLOW 去重会让它既不跑、也不进清单。
  for e in ${ALL_TIMING[@]+"${ALL_TIMING[@]}"}; do
    dup=0
    for s in ${SKIPPED[@]+"${SKIPPED[@]}"}; do [[ "$s" == "$e（慢）" || "$s" == "$e（计时）" ]] && dup=1; done
    ((dup)) || SKIPPED+=("$e（计时）")
  done

  if [[ "${GATE_REPLAY_FULL:-0}" == 1 ]]; then
    run_replay
    exit 0
  fi

  run_replay "${FAST[@]}"
  # 派给 replay.sh 的行里，登记了准入（.claude/gate.d/stage-inputs.tsv）而输入自留存产物以来没变的不跑，
  # replay.sh 汇总行报「输入没变没跑 N」并逐个列名；这里不说成「跑了」。
  echo "  ! 本次派给 replay.sh ${#FAST[@]} 个（表里的 exact 行减掉慢的；其中准入判「输入没变」而没跑的，见上面 replay.sh 汇总里那一行）；没派 ${#SKIPPED[@]} 个：${SKIPPED[*]}"
  echo "    标（慢）的是 SLOW 清单（${SLOW[*]}）里判定为 exact 的行；标（计时）的是表里判定为 timing 的行，默认档一行都不跑。"
  echo "    跑： GATE_REPLAY_FULL=1 bash .claude/scripts/gate.sh，或逐个点名：bash .claude/gate.d/${STAGE_NAME} --check experiment-replay --item <实验号>"
  echo "    ⚠️ 它们的结论靠 replay.sh 里的区间断言钉住，不跑就等于那几条断言本轮没验。"
}

# ── mutation-tables-compile ──
research_build_broken() { [[ ",${RESEARCH_BUILD_AND_REPLAY_BREAK:-}," == *",$1,"* ]]; }

# 判定库的读写（文件头 mutation-tables-compile 那一段「按表复用」，存法在 scripts/crash/lib_verdict_library.py）。$1 lookup 或 record，其余经环境变量交：
#   COMPILE_LIBRARY（空串不读不写）、COMPILE_LIBRARY_READ（1 读，别的只记不读）、COMPILE_TABLE、COMPILE_SOURCE（都相对项目根）、COMPILE_MUTATE、
#   COMPILE_TOOLCHAIN（rustc -vV 与 cargo -V 的原文）、COMPILE_JUDGE_FILES（判法版本算哪几份文件，空格分隔）、COMPILE_SUMMARY（record 记的那一句）。
# lookup 打一行「reused<TAB>原因」或「judge<TAB>原因」；record 写不成打一行原因（不红：后果只是下一趟这一张照判）
mutation_compile_library() {
  COMPILE_ACTION="$1" COMPILE_VERDICT_MODULE_DIRECTORY="$STAGE_REPOSITORY/scripts/crash" python3 - <<'PY'
import os
import re
import sys
sys.dont_write_bytecode = True
sys.path.insert(0, os.environ["COMPILE_VERDICT_MODULE_DIRECTORY"])
from lib_verdict_library import VerdictLibrary, digest_file, digest_text

table = os.environ["COMPILE_TABLE"]
source = os.environ["COMPILE_SOURCE"]
workspace = os.path.dirname(source)


def referenced_files_digest():
    """源文件里 include_str! / include_bytes! / #[path] 引的文件（相对源文件所在目录）逐个「名字 NUL 摘要」接起来的摘要；不在的也是一种取值。"""
    try:
        with open(source, encoding="utf-8", errors="surrogateescape") as handle:
            text = handle.read()
    except OSError:
        return digest_text("读不了源文件")
    names = set(re.findall(r'(?:include_str|include_bytes)!\(\s*"([^"]+)"\s*\)', text))
    names |= set(re.findall(r'#\[path\s*=\s*"([^"]+)"\s*\]', text))
    return digest_text("".join(f"{name}\0{digest_file(os.path.join(workspace, name))}\n" for name in sorted(names)))


components = [
    ("表", digest_file(table)),
    ("源文件", digest_file(source)),
    ("源文件引的文件", referenced_files_digest()),
    ("Cargo.toml", digest_file(os.path.join(workspace, "Cargo.toml"))),
    ("Cargo.lock", digest_file(os.path.join(workspace, "Cargo.lock"))),
    ("lib.rs", digest_file(os.path.join(workspace, "lib.rs"))),
    ("mutate.sh", digest_file(os.environ["COMPILE_MUTATE"])),
    ("工具链", digest_text(os.environ["COMPILE_TOOLCHAIN"])),
]
judge_version = digest_text("".join(f"{os.path.basename(path)}\0{digest_file(path)}\n" for path in os.environ["COMPILE_JUDGE_FILES"].split()))
library = VerdictLibrary(os.environ["COMPILE_LIBRARY"] or None, "mutation-tables-compile", judge_version,
                         reuse=os.environ["COMPILE_LIBRARY_READ"] == "1")
item = os.path.basename(table)
if os.environ["COMPILE_ACTION"] == "lookup":
    reused, reason = library.lookup(item, components)
    print(("reused" if reused else "judge") + "\t" + reason)
else:
    problem = library.record(item, components, os.environ["COMPILE_SUMMARY"])
    if problem:
        print(problem)
PY
}

cell_mutation_tables_compile() {
  local tables_directory=scripts/mutation/tables mutate_script="$STAGE_REPOSITORY/scripts/mutation/mutate.sh"
  local sample_mode=0 rounds=1 round library="" library_read=1 target_directory scratch toolchain_text cleanup_paths=""
  local table stem binary lookup reason rc log rows line record_problem reused_count judged_count compiled_rows
  local -a not_run=() red_lines=() red_tables=() invalid_lines=() broken_environment=()
  [[ -d "$R" && -d "$tables_directory" ]] || { echo "  ! 没有 $R 或 $tables_directory，这一格无对象可判"; exit 77; }
  if ! command -v cargo >/dev/null; then
    echo "  ✗ 没有 cargo：变异表的替换文编不编得过判不了"
    echo "     → 怎么办：装 Rust 工具链或把已装的 cargo 放进 PATH，再只重跑这一格：bash .claude/gate.d/${STAGE_NAME} --check mutation-tables-compile"
    exit 1
  fi
  if [[ ! -f "$mutate_script" ]]; then
    echo "  ✗ 没有 $mutate_script：这一格的判法（--compile-only、--binary-of）在它里面"
    echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一格按红记。"
    exit 1
  fi
  # 样本（被判的根不是这道阶段所在的仓、也不是 --staged）：判定库与编译目录都用这一格自己的临时目录，跑完删；只有 run-twice 那份读判定库、接连判两趟
  if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then sample_mode=1; fi
  if ((sample_mode == 0)); then
    source "$STAGE_REPOSITORY/scripts/gate/stage-run-or-skip.sh" || { echo "  ✗ source 不进 scripts/gate/stage-run-or-skip.sh，判不出这一格要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一格按红记。"; exit 1; }
    stage_run_or_skip "$STAGE_REPOSITORY/scripts/gate/stage-must-run.sh" "复用上一次整轮全绿的判定" \
      "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一格读哪几条路径见 .claude/gate.d/stage-inputs.tsv 里 ${STAGE_NAME}:mutation-tables-compile 那一行。" \
      -- "$ROOT" "${STAGE_NAME}:mutation-tables-compile"
    library="${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-mutation-tables-compile-verdicts"
    [[ "${SINGLEFS_GATE_FULL:-}" == 1 ]] && library_read=0
    target_directory="${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-mutation-tables-compile-target"
  else
    target_directory="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
    cleanup_paths="$(printf '%q' "$target_directory")"
    if [[ -f .mutation-tables-compile-sample/run-twice ]]; then
      library="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -rf "${target_directory:?}"; exit 1; }
      cleanup_paths+=" $(printf '%q' "$library")"
      rounds=2
    fi
  fi
  scratch="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  cleanup_paths+=" $(printf '%q' "$scratch")"
  # shellcheck disable=SC2064  # 路径在这里就展开（各自 %q 过）：退出时 local 变量已经不在
  trap "rm -rf -- $cleanup_paths" EXIT
  # 弄坏开关 no-cap-lints（文件头）：PATH 前面放垫片 cargo，剥掉 RUSTFLAGS 与 CARGO_ENCODED_RUSTFLAGS 再交给真 cargo，mutate.sh 带的 cap 一条都到不了 rustc
  if research_build_broken no-cap-lints; then
    local real_cargo
    real_cargo="$(command -v cargo)"
    mkdir "$scratch/no-cap-bin" && printf '#!/usr/bin/env bash\nexec env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS %q "$@"\n' "$real_cargo" >"$scratch/no-cap-bin/cargo" \
      && chmod +x "$scratch/no-cap-bin/cargo" || { echo "  ✗ 弄坏开关 no-cap-lints 建不了垫片 cargo"; exit 1; }
    broken_environment=("PATH=$scratch/no-cap-bin:$PATH")
  fi
  toolchain_text="$(rustc -vV 2>&1; cargo -V 2>&1)"
  # E162 的 rocksdb 要 BINDGEN_EXTRA_CLANG_ARGS（mutate.sh 文件头 required-features 那一段：由调用方设）；没设就取放量共用库里那一份
  if [[ -z "${BINDGEN_EXTRA_CLANG_ARGS:-}" && -f "$STAGE_REPOSITORY/scripts/crash/crash-amplification-lib.sh" ]]; then
    source "$STAGE_REPOSITORY/scripts/crash/crash-amplification-lib.sh"
    BINDGEN_EXTRA_CLANG_ARGS="${CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS:-}"
    export BINDGEN_EXTRA_CLANG_ARGS
  fi
  for ((round = 1; round <= rounds; round++)); do
    ((rounds > 1)) && echo "  ── 第 $round 趟（样本 .mutation-tables-compile-sample/run-twice：判定库 $library，第二趟应当整张复用）"
    not_run=(); red_lines=(); red_tables=(); reused_count=0; judged_count=0; compiled_rows=0
    for table in "$tables_directory"/*.tsv; do
      [[ -f "$table" ]] || continue
      stem="$(basename "$table" .tsv)"
      if [[ ! -f "$R/$stem.rs" ]]; then
        not_run+=("$table（$R/ 下没有同名的 $stem.rs，取不出 bin、源文件、表三元组）")
        continue
      fi
      if ! binary="$(cd "$R" && bash "$mutate_script" --binary-of "$stem.rs" 2>&1)"; then
        not_run+=("$table（mutate.sh --binary-of $stem.rs 没给出 bin 名：$binary）")
        continue
      fi
      lookup="$(COMPILE_LIBRARY="$library" COMPILE_LIBRARY_READ="$library_read" COMPILE_TABLE="$table" COMPILE_SOURCE="$R/$stem.rs" \
        COMPILE_MUTATE="$mutate_script" COMPILE_TOOLCHAIN="$toolchain_text" COMPILE_JUDGE_FILES="$STAGE_DIR/$STAGE_NAME $STAGE_REPOSITORY/scripts/crash/lib_verdict_library.py" \
        mutation_compile_library lookup)"
      reason="${lookup#*$'\t'}"
      if [[ "${lookup%%$'\t'*}" == reused ]]; then
        reused_count=$((reused_count + 1))
        echo "  · $table：复用（$reason）"
        continue
      fi
      log="$scratch/$stem.log"
      if (cd "$R" && env ${broken_environment[@]+"${broken_environment[@]}"} MUTATE_TARGET_DIR="$target_directory" bash "$mutate_script" --compile-only "$binary" "$stem.rs" "../mutation/tables/$stem.tsv") >"$log" 2>&1; then rc=0; else rc=$?; fi
      judged_count=$((judged_count + 1))
      research_build_broken invalid-rows-ignored && ((rc == 1)) && rc=0   # 弄坏开关：⏭ 不看，退 1 的表也当绿
      case "$rc" in
        0)
          rows="$(grep -c '^✅ ' "$log")"
          compiled_rows=$((compiled_rows + rows))
          record_problem="$(COMPILE_LIBRARY="$library" COMPILE_LIBRARY_READ="$library_read" COMPILE_TABLE="$table" COMPILE_SOURCE="$R/$stem.rs" \
            COMPILE_MUTATE="$mutate_script" COMPILE_TOOLCHAIN="$toolchain_text" COMPILE_JUDGE_FILES="$STAGE_DIR/$STAGE_NAME $STAGE_REPOSITORY/scripts/crash/lib_verdict_library.py" \
            COMPILE_SUMMARY="$rows 条全编得过（$binary）" mutation_compile_library record)"
          echo "  · $table：$rows 条全编得过，现判（$reason）"
          [[ -n "$record_problem" ]] && echo "    ! 没记进判定库：$record_problem（下一趟这一张照判）"
          ;;
        1)
          mapfile -t invalid_lines < <(grep '^⏭ ' "$log")
          if ((${#invalid_lines[@]})); then
            for line in "${invalid_lines[@]}"; do red_lines+=("$table：${line#⏭  }"); done
            echo "  · $table：${#invalid_lines[@]} 条编不过，现判（$reason）"
          else
            red_tables+=("$table：mutate.sh --compile-only 退 1 而一行 ⏭ 都没有；输出末尾：$(tail -n 3 "$log" | tr '\n' ' ')")
          fi
          ;;
        *)
          red_tables+=("$table：mutate.sh --compile-only $binary $stem.rs 退 $rc（2 基线编不过或用法错、3 锚点没命中、5 跑的时候源文件被改、6 bin 与源文件对不上、8 收回的判定对不上）；输出末尾：$(tail -n 4 "$log" | tr '\n' ' ')")
          ;;
      esac
    done
    if ((${#red_lines[@]} + ${#red_tables[@]})); then
      for line in ${red_lines[@]+"${red_lines[@]}"}; do
        echo "  ✗ $line"   # gate-lint:detail
      done
      for line in ${red_tables[@]+"${red_tables[@]}"}; do
        echo "  ✗ $line"   # gate-lint:detail
      done
      echo "  ✗ 研究侧变异表有 ${#red_lines[@]} 条替换文编不过、${#red_tables[@]} 张表没判完（现判 $judged_count 张、复用 $reused_count 张）"
      echo "     → 怎么办：编不过的改那一行替换文（编得过、仍改被测行为），或装置改回能编；没判完的照括号里 mutate.sh 的退出码办。"
      echo "               单张核：(cd scripts/experiments && bash ../mutation/mutate.sh --compile-only <bin> <源文件> ../mutation/tables/<表>)"
      exit 1
    fi
  done
  for line in ${not_run[@]+"${not_run[@]}"}; do
    echo "  ! 本次未跑：$line"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 mutation-tables-compile 格没判 $line" >> "$GATE_NOT_RUN_FILE"; fi
  done
  if ((judged_count + reused_count == 0)); then
    echo "  ! 本次无对象可判：$tables_directory 下没有一张表对得上 $R/ 下同名的 .rs（没跑 ${#not_run[@]} 张，上面逐张列名）"
    exit 77
  fi
  echo "  ✓ 研究侧变异表逐条只编译全编得过：判了 $((judged_count + reused_count)) 张表，现判 $judged_count 张、复用 $reused_count 张（现判的共 $compiled_rows 条）；没跑 ${#not_run[@]} 张（上面「本次未跑」逐张列名）"
}

stage_cells_run "$ROOT"
