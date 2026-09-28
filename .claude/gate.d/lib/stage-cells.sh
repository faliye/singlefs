#!/usr/bin/env bash
# admission: always 直接执行时跑这份库此刻的自证：库与用它的阶段随时在改，上一次的结论不替这一次作保
# run-condition: none 自证只在临时目录里造一道合成阶段来跑，除了 bash 与 coreutils 没有环境要求
# gate-stage: 门禁格共用库的自证
# gate-similar: code-tooling.sh 它自己写了一份 --check 解析与逐格汇总，并注明抽共用库归并组的收尾一件；这一份就是那个共用库，code-tooling 的判据不并进来，下一批改成 source 它
# gate-similar: doc-registries.sh 与 code-tooling 同形的逐格汇总外壳（按退出码分三堆、报本次未跑的格）；这一份把外壳抽出来，doc-registries 的格判的是欠账表，不并进来
# gate-similar: code-source-discipline.sh 同样各写一份 --check 解析与汇总外壳，格判的是格式常量与 kb 对账，不并进来；同样各写一份汇总外壳与弄坏开关，格判的是实验源码纪律，不并进来
# gate-similar: doc-experiments.sh 同样各写一份汇总外壳，格判的是实验页与产物，不并进来
# gate-similar: doc-decisions.sh 它是第一道改用这份库的阶段（样板），只留格的判法
#
# research/scripts/stage-run-or-skip.sh 也是被阶段 source 的库，但判的是一整道要不要跑，不管格。
#
# 门禁阶段共用的格结构：登记格、解析 --list / --check、样本只跑点名的格、每格在自己的子 shell 里跑、逐格汇总，
# 阶段 source 这一份，不各写一份。规则：.claude/rules/verification.md「门禁的结构」。
#
# 阶段里的写法（preflight 那一行之后）：
#   source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
#   stage_cell <格名> <函数> <判什么的一句> <判红时的出路>      每格一行，登记次序就是跑的次序
#   stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
#   …阶段自己解析剩下的参数（项目根、--write 这类），cd 到项目根…
#   stage_cells_run "$ROOT"                                       跑选中的格、汇总、按汇总退出，不返回
# 文件头的格名表每格一行 `# gate-cell: <格名> <判什么的一句>`，与 stage_cell 登记的逐格相同、次序相同
# （research/scripts/gate-structure-check.py 判）。格函数不带参数调，里面 exit 只结束那一格。
#
# stage_cells_parse：
#   --list                     逐行打「格名<制表符>判什么」到 stdout，退 0，不跑任何格
#   --check <格名>[,<格名>…]   只跑点名的格；--check=<…> 同义；给几次取并集；跑的次序按登记次序
#   不带 --check               跑全部登记的格
#   其余参数（项目根、--force、--write……）原样按次序放进 STAGE_CELLS_REST，交给阶段
#   格名写错、--check 后面缺格名：列出可用的格名，退 2
#   设 STAGE_CELLS_SELECTED（要跑的格，按登记次序）与 STAGE_CELLS_CHECK_GIVEN（给了 --check 是 1）
#
# 样本只跑点名的格：没给 --check、被判的根里有 .gate-cells（一行一个格名；空行与 # 开头的行不算）、
#   被判的根（pwd -P）不是这道阶段所在的仓（阶段文件往上两级，pwd -P）、也没设 GATE_STAGED_FROM 时，
#   只跑文件里点名的格。真仓与 gate.sh --staged 的临时 worktree 里这个文件一律不认
#   （与 code-tooling 的 research-script-selftests 格认 .selftest-coverage-only 同一种判法）。
#   文件里点了没登记的格、或一格都没点：退 2。
#
# stage_cells_run：每格在自己的子 shell 里跑，退出码写进临时目录，全部跑完按登记次序回读（数一遍，份数对不上判红）。
#   汇总逐格一行「<格名>：绿 / 红（退出 N）/ 本次未跑」。
#   有一格红（退出码既不是 0 也不是 77）⇒ 整道退 1，红格逐格接它登记的出路，末了给只重跑红格的 --check 命令；
#   跑到的格全退 77 ⇒ 整道退 77，不记通过；否则退 0。
#   退 77 的格逐格往 $GATE_NOT_RUN_FILE 报一行（这个变量不在时不写；.claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」）。
#
# 直接执行只跑自证（住在 .claude/gate.d/lib/ 下，gate.sh 只取门禁目录顶层的 *.sh，不把它当阶段跑；提交时由工具层自检的 runner 表跑一次 --selftest）：
#   bash .claude/gate.d/lib/stage-cells.sh [项目根] [--force] [--selftest]
#   跑自证：项目根下有 .claude/gate.d/lib/stage-cells.sh 就判那一份，没有（或没给根）就判这一份自己。
#   自证造一道三格（alpha、beta、gamma）的合成阶段，逐条判：--list 的输出与没跑格、--check 只跑点名的（一格与两格）、
#   格名写错与缺格名退 2、其余参数原样交给阶段、.gate-cells 在样本根生效而在阶段所在的仓与 --staged 下不生效、
#   .gate-cells 点了没登记的格退 2、汇总三种结局（有红退 1、全 77 退 77、绿与 77 混着退 0）与本次未跑的上报、
#   退出码 2 的格算红、一格红之后别的格照跑照判、红格接它自己的出路。
#   判的是这一份自己时，再逐个打开弄坏开关重跑一遍自证，每个开关都要让至少一条判错。
# 弄坏开关 STAGE_CELLS_BREAK=<项>，每一项让某几条自证判错：
#   list-empty          --list 什么都不打                          list 判错
#   check-ignored       --check 不起作用，照跑全部                  check-one、check-two、pass-through 判错
#   unknown-accepted    写错的格名悄悄丢掉、不退 2                  check-unknown、marker-unknown 判错
#   marker-everywhere   .gate-cells 在阶段所在的仓与 --staged 下也认  marker-own-repository、marker-staged 判错
#   marker-ignored      .gate-cells 一律不认                        marker-sample、marker-unknown 判错
#   red-swallowed       红格按绿记                                  red、odd-exit 判错
#   not-run-as-pass     全 77 退 0                                  all-not-run 判错
#   shared-shell        格不进子 shell，一格 exit 就带走整道        red、odd-exit、mixed 等判错
#   not-run-unreported  退 77 的格不往 $GATE_NOT_RUN_FILE 报        all-not-run、mixed 判错
#   generic-howto       红格不接它自己的出路，只给一句通用的        red 判错

declare -ga STAGE_CELLS_NAMES=()
declare -gA STAGE_CELLS_FUNCTION=() STAGE_CELLS_TITLE=() STAGE_CELLS_HOWTO=()
declare -ga STAGE_CELLS_SELECTED=() STAGE_CELLS_REST=()
STAGE_CELLS_CHECK_GIVEN=0
STAGE_CELLS_SELECTION_SOURCE="全部登记的格"
STAGE_CELLS_STAGE_FILE="${BASH_SOURCE[1]:-$0}"
STAGE_CELLS_STAGE_NAME="${STAGE_CELLS_STAGE_FILE##*/}"
STAGE_CELLS_STAGE_REPOSITORY=""
if [[ "$STAGE_CELLS_STAGE_FILE" == */* ]]; then
  STAGE_CELLS_STAGE_REPOSITORY="$(cd "${STAGE_CELLS_STAGE_FILE%/*}/../.." 2>/dev/null && pwd -P)"
else
  STAGE_CELLS_STAGE_REPOSITORY="$(cd ../.. 2>/dev/null && pwd -P)"
fi

stage_cells_available() {
  local name
  for name in ${STAGE_CELLS_NAMES[@]+"${STAGE_CELLS_NAMES[@]}"}; do
    printf '       %s  %s\n' "$name" "${STAGE_CELLS_TITLE[$name]}"
  done
}

# stage_cell <格名> <函数> <判什么的一句> <判红时的出路>
stage_cell() {
  if (($# != 4)) || [[ -z "$3" || -z "$4" ]]; then
    printf '  ✗ %s：stage_cell 要四个参数（格名、函数、判什么的一句、判红时的出路），收到 %s 个：%s\n' "$STAGE_CELLS_STAGE_NAME" "$#" "$*"
    printf '     → 怎么办：照 .claude/gate.d/lib/stage-cells.sh 文件头的写法补齐；判什么与出路都不许空，红格的出路就从这里取。\n'
    exit 1
  fi
  if [[ ! "$1" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
    printf '  ✗ %s：格名「%s」只许小写字母、数字与连字符\n' "$STAGE_CELLS_STAGE_NAME" "$1"
    printf '     → 怎么办：改成说出判什么的短名字（shape、status-sync 这类），--check 与 .gate-cells 都按这个名字认。\n'
    exit 1
  fi
  if [[ -n "${STAGE_CELLS_FUNCTION[$1]+set}" ]]; then
    printf '  ✗ %s：格名 %s 登记了两次\n' "$STAGE_CELLS_STAGE_NAME" "$1"
    printf '     → 怎么办：一个格名只登记一行；两格判的不是同一件事就起两个名字。\n'
    exit 1
  fi
  STAGE_CELLS_NAMES+=("$1")
  STAGE_CELLS_FUNCTION[$1]="$2"
  STAGE_CELLS_TITLE[$1]="$3"
  STAGE_CELLS_HOWTO[$1]="$4"
}

# stage_cells_pick <来源说明> <格名…>：把点名的格按登记次序放进 STAGE_CELLS_SELECTED；有没登记的就退 2
stage_cells_pick() {
  local source_description="$1" name requested unknown=()
  shift
  declare -A wanted=()
  for requested in "$@"; do
    [[ -z "$requested" ]] && continue
    if [[ -z "${STAGE_CELLS_FUNCTION[$requested]+set}" ]]; then
      unknown+=("$requested")
    else
      wanted[$requested]=1
    fi
  done
  if ((${#unknown[@]})) && [[ "${STAGE_CELLS_BREAK:-}" != unknown-accepted ]]; then
    printf '  ✗ %s：%s里有没登记的格名：%s\n' "$STAGE_CELLS_STAGE_NAME" "$source_description" "${unknown[*]}"
    printf '     → 怎么办：格名照下面这张表写（也可以 bash .claude/gate.d/%s --list 看），几格用逗号隔开：\n' "$STAGE_CELLS_STAGE_NAME"
    stage_cells_available
    exit 2
  fi
  if ((${#wanted[@]} == 0)); then
    printf '  ✗ %s：%s一个格名都没点\n' "$STAGE_CELLS_STAGE_NAME" "$source_description"
    printf '     → 怎么办：写成 --check <格名>[,<格名>…]（样本根的 .gate-cells 一行一个格名）；可用的格名：\n'
    stage_cells_available
    exit 2
  fi
  STAGE_CELLS_SELECTED=()
  for name in "${STAGE_CELLS_NAMES[@]}"; do
    if [[ -n "${wanted[$name]+set}" ]]; then STAGE_CELLS_SELECTED+=("$name"); fi
  done
}

stage_cells_parse() {
  local list_requested=0 part
  local requested=() parts=()
  STAGE_CELLS_REST=()
  STAGE_CELLS_CHECK_GIVEN=0
  if ((${#STAGE_CELLS_NAMES[@]} == 0)); then
    printf '  ✗ %s：一格都没登记就调了 stage_cells_parse\n' "$STAGE_CELLS_STAGE_NAME"
    printf '     → 怎么办：先逐格 stage_cell <格名> <函数> <判什么> <出路>，再 stage_cells_parse "$@"。\n'
    exit 1
  fi
  while (($#)); do
    case "$1" in
      --list) list_requested=1; shift ;;
      --check|--check=*)
        if [[ "$1" == --check ]]; then
          if (($# < 2)) || [[ "$2" == -* || -z "$2" ]]; then
            printf '  ✗ %s：--check 后面缺格名\n' "$STAGE_CELLS_STAGE_NAME"
            printf '     → 怎么办：写成 --check <格名>[,<格名>…]；不给 --check 就全跑。可用的格名：\n'
            stage_cells_available
            exit 2
          fi
          part="$2"; shift 2
        else
          part="${1#--check=}"; shift
        fi
        IFS=, read -r -a parts <<<"$part"
        requested+=(${parts[@]+"${parts[@]}"})
        STAGE_CELLS_CHECK_GIVEN=1 ;;
      *) STAGE_CELLS_REST+=("$1"); shift ;;
    esac
  done
  if ((list_requested)); then
    if [[ "${STAGE_CELLS_BREAK:-}" != list-empty ]]; then
      for part in "${STAGE_CELLS_NAMES[@]}"; do printf '%s\t%s\n' "$part" "${STAGE_CELLS_TITLE[$part]}"; done
    fi
    exit 0
  fi
  if ((STAGE_CELLS_CHECK_GIVEN)); then
    stage_cells_pick "--check " ${requested[@]+"${requested[@]}"}
    STAGE_CELLS_SELECTION_SOURCE="--check 点名"
    if [[ "${STAGE_CELLS_BREAK:-}" == check-ignored ]]; then STAGE_CELLS_SELECTED=("${STAGE_CELLS_NAMES[@]}"); fi
  else
    STAGE_CELLS_SELECTED=("${STAGE_CELLS_NAMES[@]}")
  fi
}

# 样本标记：被判的根是样本（不是这道阶段所在的仓、不在 --staged 里）时，只跑 .gate-cells 点名的格。
# 读的是当前目录（stage_cells_run 之前阶段已经 cd 到项目根），参数只用来写进输出。
stage_cells_apply_sample_marker() {
  local root="$1" root_physical line honored=0
  local marked=()
  ((STAGE_CELLS_CHECK_GIVEN)) && return 0
  [[ -f .gate-cells ]] || return 0
  root_physical="$(pwd -P)"
  if [[ -n "$root_physical" && "$root_physical" != "$STAGE_CELLS_STAGE_REPOSITORY" && -z "${GATE_STAGED_FROM:-}" ]]; then honored=1; fi
  case "${STAGE_CELLS_BREAK:-}" in
    marker-everywhere) honored=1 ;;
    marker-ignored) honored=0 ;;
  esac
  if ((honored == 0)); then
    printf '  · %s 里有 .gate-cells，但这里是阶段所在的仓或 --staged 的临时树，不认它，照跑%s\n' "$root" "$STAGE_CELLS_SELECTION_SOURCE"
    return 0
  fi
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%%#*}"; line="${line//[[:space:]]/}"
    [[ -n "$line" ]] && marked+=("$line")
  done < .gate-cells
  stage_cells_pick "样本根的 .gate-cells " ${marked[@]+"${marked[@]}"}
  STAGE_CELLS_SELECTION_SOURCE="样本根的 .gate-cells 点名"
  printf '  · 样本只跑 .gate-cells 点名的格：%s（%s 是样本，不是这道阶段所在的仓）\n' "${STAGE_CELLS_SELECTED[*]}" "$root"
}

# stage_cells_run <项目根>：调用前阶段已经 cd 到项目根。跑完按汇总 exit，不返回。
stage_cells_run() {
  local root="${1:-.}" scratch name function_name cell_exit index=0 collected=0
  local green=() red=() not_run=()
  local -A verdict_of=()
  stage_cells_apply_sample_marker "$root"
  scratch="$(mktemp -d)" || { printf '  ✗ %s：建不了放逐格退出码的临时目录\n' "$STAGE_CELLS_STAGE_NAME"; printf '     → 怎么办：看 $TMPDIR 满没满、有没有写权限，修好再跑；这一道一格都没跑。\n'; exit 1; }
  for name in "${STAGE_CELLS_SELECTED[@]}"; do
    function_name="${STAGE_CELLS_FUNCTION[$name]}"
    printf '── %s：%s\n' "$name" "${STAGE_CELLS_TITLE[$name]}"
    if ! declare -F "$function_name" >/dev/null; then
      printf '  ✗ 格 %s 登记的函数 %s 没有定义\n' "$name" "$function_name"
      printf '     → 怎么办：stage_cell 第二个参数写实现这一格的函数名，函数要在 stage_cells_run 之前定义。\n'
      cell_exit=1
    elif [[ "${STAGE_CELLS_BREAK:-}" == shared-shell ]]; then
      "$function_name"; cell_exit=$?
    elif ( "$function_name" ); then
      cell_exit=0
    else
      cell_exit=$?
    fi
    printf '%s\n' "$cell_exit" > "$scratch/$index.exit"
    index=$((index + 1))
  done
  index=0
  for name in "${STAGE_CELLS_SELECTED[@]}"; do
    if [[ ! -f "$scratch/$index.exit" ]]; then
      cell_exit="缺"
    else
      cell_exit="$(<"$scratch/$index.exit")"
      collected=$((collected + 1))
    fi
    index=$((index + 1))
    if [[ "$cell_exit" == 77 ]]; then
      not_run+=("$name"); verdict_of[$name]="本次未跑"
    elif [[ "$cell_exit" == 0 || "${STAGE_CELLS_BREAK:-}" == red-swallowed ]]; then
      green+=("$name"); verdict_of[$name]="绿"
    else
      red+=("$name"); verdict_of[$name]="红（退出 $cell_exit）"
    fi
  done
  rm -rf "${scratch:?}"

  printf '── 汇总：%s 跑了 %s 格（%s）\n' "$STAGE_CELLS_STAGE_NAME" "${#STAGE_CELLS_SELECTED[@]}" "$STAGE_CELLS_SELECTION_SOURCE"
  for name in "${STAGE_CELLS_SELECTED[@]}"; do
    printf '  %s：%s\n' "$name" "${verdict_of[$name]}"
  done

  if [[ -n "${GATE_NOT_RUN_FILE:-}" && "${STAGE_CELLS_BREAK:-}" != not-run-unreported ]]; then
    for name in ${not_run[@]+"${not_run[@]}"}; do
      printf '%s 的 %s 格本次无对象可判（%s），原因见这一道输出里那一格的末行\n' "$STAGE_CELLS_STAGE_NAME" "$name" "${STAGE_CELLS_TITLE[$name]}" >> "$GATE_NOT_RUN_FILE"
    done
  fi

  if ((collected != ${#STAGE_CELLS_SELECTED[@]})); then
    printf '  ✗ %s：派出去 %s 格，只收回 %s 份退出码\n' "$STAGE_CELLS_STAGE_NAME" "${#STAGE_CELLS_SELECTED[@]}" "$collected"
    printf '     → 怎么办：临时目录在跑的过程中被删了或写不进去；看 $TMPDIR 再重跑，收不全的一轮不算判过。\n'
    exit 1
  fi
  if ((${#red[@]})); then
    local rerun=""
    for name in "${red[@]}"; do rerun+="${rerun:+,}$name"; done
    printf '  ✗ %s：%s 格里 %s 格红（%s）\n' "$STAGE_CELLS_STAGE_NAME" "${#STAGE_CELLS_SELECTED[@]}" "${#red[@]}" "${red[*]}"
    if [[ "${STAGE_CELLS_BREAK:-}" == generic-howto ]]; then
      printf '     → 照上面各格的输出改。\n'
    else
      for name in "${red[@]}"; do
        printf '     → %s：%s\n' "$name" "${STAGE_CELLS_HOWTO[$name]}"
      done
    fi
    printf '     → 改完只重跑红的格：bash .claude/gate.d/%s --check %s\n' "$STAGE_CELLS_STAGE_NAME" "$rerun"
    exit 1
  fi
  if ((${#green[@]} == 0)) && [[ "${STAGE_CELLS_BREAK:-}" != not-run-as-pass ]]; then
    printf '  ! %s：跑到的 %s 格都本次无对象可判，这一道本次未跑（不记通过）\n' "$STAGE_CELLS_STAGE_NAME" "${#not_run[@]}"
    exit 77
  fi
  local not_run_list="无"
  if ((${#not_run[@]})); then not_run_list="${not_run[*]}"; fi
  printf '  ✓ %s：%s 格里 %s 格绿（%s），%s 格本次未跑（%s）\n' "$STAGE_CELLS_STAGE_NAME" "${#STAGE_CELLS_SELECTED[@]}" "${#green[@]}" "${green[*]}" "${#not_run[@]}" "$not_run_list"
  exit 0
}

# ── 自证 ──
# stage_cells_selftest <要判的库> [nested]：在临时目录里造一道三格的合成阶段 source 那一份库，逐条判；
# 判的是这一份自己、又不是 nested 时，再逐个打开弄坏开关重跑，每个开关都要让至少一条判错。
stage_cells_selftest() {
  local library_under_test="$1" nested="${2:-}" scratch stage_file sample_root own_root
  local cases=0 wrong=0 label
  scratch="$(mktemp -d)" || { printf '  ✗ 自证建不了临时目录\n'; printf '     → 怎么办：看 $TMPDIR 满没满、有没有写权限，修好再跑。\n'; return 1; }
  own_root="$scratch/stage-repository"
  sample_root="$scratch/sample-root"
  stage_file="$own_root/.claude/gate.d/50-sample-cells.sh"
  mkdir -p "$own_root/.claude/gate.d" "$sample_root"
  {
    printf '#!/usr/bin/env bash\nset -uo pipefail\n'
    printf 'source %q\n' "$library_under_test"
    # shellcheck disable=SC2016
    printf '%s\n' \
      'cell_alpha() { printf "alpha\n" >> "$SAMPLE_TRACE"; echo "  alpha 格判完"; exit "${OUTCOME_ALPHA:-0}"; }' \
      'cell_beta()  { printf "beta\n"  >> "$SAMPLE_TRACE"; echo "  beta 格判完";  exit "${OUTCOME_BETA:-0}"; }' \
      'cell_gamma() { printf "gamma\n" >> "$SAMPLE_TRACE"; echo "  gamma 格判完"; exit "${OUTCOME_GAMMA:-0}"; }' \
      'stage_cell alpha cell_alpha "判甲" "甲的出路：改甲"' \
      'stage_cell beta cell_beta "判乙" "乙的出路：改乙"' \
      'stage_cell gamma cell_gamma "判丙" "丙的出路：改丙"' \
      'stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}' \
      'printf "%s\n" "$@" > "$SAMPLE_REST"' \
      'ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"' \
      'cd "$ROOT" || exit 2' \
      'stage_cells_run "$ROOT"'
  } > "$stage_file"

  # sample_case <标签> <想要的退出码> <想要的 trace（空格隔开，- 表示一格都没跑）> <环境…> -- <参数…>
  # 输出落在 $scratch/out，其余检查由调用方接着判
  local got_exit got_trace
  sample_case() {
    local case_label="$1" want_exit="$2" want_trace="$3"
    shift 3
    local environment=()
    while (($#)) && [[ "$1" != -- ]]; do environment+=("$1"); shift; done
    shift
    : > "$scratch/trace"; : > "$scratch/rest"; : > "$scratch/not-run"
    got_exit=0
    env -u GATE_NOT_RUN_FILE -u GATE_STAGED_FROM SAMPLE_TRACE="$scratch/trace" SAMPLE_REST="$scratch/rest" \
      ${environment[@]+"${environment[@]}"} bash "$stage_file" "$@" > "$scratch/out" 2>&1 || got_exit=$?
    got_trace="$(tr '\n' ' ' < "$scratch/trace")"; got_trace="${got_trace% }"; [[ -n "$got_trace" ]] || got_trace="-"
    cases=$((cases + 1))
    label="$case_label"
    if [[ "$got_exit" != "$want_exit" ]]; then
      case_wrong "期望退出 $want_exit，实测 $got_exit"; return 1
    fi
    if [[ "$want_trace" != "*" && "$got_trace" != "$want_trace" ]]; then
      case_wrong "期望跑了「$want_trace」，实测跑了「$got_trace」"; return 1
    fi
    return 0
  }
  local -A wrong_labels=()
  case_wrong() {
    wrong_labels[$label]=1
    wrong="${#wrong_labels[@]}"
    printf '    %s 这一条判错：%s\n' "$label" "$1"   # gate-lint:detail
    sed 's/^/      | /' "$scratch/out" | head -n 30   # gate-lint:detail
  }
  output_has() { grep -qF -- "$1" "$scratch/out" || case_wrong "输出里找不到「$1」"; }

  if sample_case list 0 - -- --list; then
    if [[ "$(cat "$scratch/out")" != "$(printf 'alpha\t判甲\nbeta\t判乙\ngamma\t判丙')" ]]; then case_wrong "--list 的输出不是逐行「格名<制表符>判什么」"; fi
  fi
  sample_case check-one 0 "beta" -- --check beta "$sample_root"
  sample_case check-two 0 "alpha gamma" -- --check gamma,alpha "$sample_root"
  if sample_case check-unknown 2 - -- --check beta,nope "$sample_root"; then
    output_has "nope"; output_has "gamma  判丙"
  fi
  sample_case check-missing 2 - -- --check
  if sample_case pass-through 0 "alpha" -- "$sample_root" --write --check alpha --force; then
    if [[ "$(cat "$scratch/rest")" != "$(printf '%s\n--write\n--force' "$sample_root")" ]]; then case_wrong "交给阶段的其余参数不是「根 --write --force」原样：$(tr '\n' ' ' < "$scratch/rest")"; fi
  fi
  printf 'beta\n' > "$sample_root/.gate-cells"
  if sample_case marker-sample 0 "beta" OUTCOME_ALPHA=1 OUTCOME_GAMMA=1 -- "$sample_root"; then
    output_has "样本只跑 .gate-cells 点名的格：beta"
  fi
  sample_case marker-staged 0 "alpha beta gamma" GATE_STAGED_FROM=/nonexistent-source-repository -- "$sample_root"
  printf 'nope\nbeta\n' > "$sample_root/.gate-cells"
  sample_case marker-unknown 2 - -- "$sample_root"
  rm -f "$sample_root/.gate-cells"
  printf '# 样本标记\nbeta\n' > "$own_root/.gate-cells"
  sample_case marker-own-repository 0 "alpha beta gamma" -- "$own_root"
  rm -f "$own_root/.gate-cells"
  if sample_case red 1 "alpha beta gamma" OUTCOME_ALPHA=1 OUTCOME_GAMMA=77 -- "$sample_root"; then
    output_has "alpha：红（退出 1）"; output_has "beta：绿"; output_has "gamma：本次未跑"
    output_has "→ alpha：甲的出路：改甲"; output_has "--check alpha"
  fi
  sample_case odd-exit 1 "alpha beta gamma" OUTCOME_ALPHA=2 -- "$sample_root"
  if sample_case all-not-run 77 "alpha beta gamma" OUTCOME_ALPHA=77 OUTCOME_BETA=77 OUTCOME_GAMMA=77 GATE_NOT_RUN_FILE="$scratch/not-run" -- "$sample_root"; then
    if [[ "$(grep -c '格本次无对象可判' "$scratch/not-run")" != 3 ]]; then case_wrong "\$GATE_NOT_RUN_FILE 里该有 3 行本次未跑，实有 $(grep -c '格本次无对象可判' "$scratch/not-run")"; fi
  fi
  if sample_case mixed 0 "alpha beta gamma" OUTCOME_BETA=77 GATE_NOT_RUN_FILE="$scratch/not-run" -- "$sample_root"; then
    output_has "beta：本次未跑"
    if ! grep -q '的 beta 格本次无对象可判' "$scratch/not-run" || [[ "$(wc -l < "$scratch/not-run")" != 1 ]]; then case_wrong "\$GATE_NOT_RUN_FILE 里该只有 beta 那一行"; fi
  fi

  local breaks_proven=0 breaks_missed=() break_name
  if ((wrong == 0)) && [[ -z "$nested" && -z "${STAGE_CELLS_BREAK:-}" ]]; then
    for break_name in list-empty check-ignored unknown-accepted marker-everywhere marker-ignored red-swallowed not-run-as-pass shared-shell not-run-unreported generic-howto; do
      if STAGE_CELLS_BREAK="$break_name" bash "$library_under_test" --selftest-nested >/dev/null 2>&1; then
        breaks_missed+=("$break_name")
      else
        breaks_proven=$((breaks_proven + 1))
      fi
    done
  fi
  rm -rf "${scratch:?}"
  if ((wrong)); then
    printf '  ✗ 格共用库 %s 的自证 %s 条里 %s 条判错（上面逐条列着）\n' "$library_under_test" "$cases" "$wrong"
    printf '     → 怎么办：按判错那几条的标签找 stage_cells_parse / stage_cells_apply_sample_marker / stage_cells_run 里对应的一段修；判据写在这份库的文件头。\n'
    return 1
  fi
  if ((${#breaks_missed[@]})); then
    printf '  ✗ 弄坏开关打开之后自证照样全对：%s\n' "${breaks_missed[*]}"
    printf '     → 怎么办：那几个开关管的那一段没有自证盯着；补一条会被它弄错的自证，或者开关已经失效就修开关。\n'
    return 1
  fi
  if [[ -n "$nested" || -n "${STAGE_CELLS_BREAK:-}" ]]; then
    printf '  ✓ 格共用库的自证 %s 条都判对（没重跑弄坏开关）\n' "$cases"
  else
    printf '  ✓ 格共用库的自证 %s 条都判对；弄坏开关 %s 个，每个都让至少一条判错\n' "$cases" "$breaks_proven"
  fi
  return 0
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -uo pipefail
  source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
  preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
  root_argument=""; nested_run=""
  for argument in "$@"; do
    case "$argument" in
      --selftest|--force) ;;
      --selftest-nested) nested_run=nested ;;
      -*)
        printf '  ✗ 认不出的参数 %s\n' "$argument"
        printf '     → 怎么办：这一份是被阶段 source 的库，直接执行只跑自证：bash .claude/gate.d/lib/stage-cells.sh [项目根] [--selftest]\n'
        exit 2 ;;
      *) root_argument="$argument" ;;
    esac
  done
  ROOT="${root_argument:-}"
  library_under_test="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/${BASH_SOURCE[0]##*/}"
  if [[ -n "$ROOT" && -f "$ROOT/.claude/gate.d/lib/stage-cells.sh" ]]; then
    library_under_test="$(cd "$ROOT/.claude/gate.d/lib" && pwd -P)/stage-cells.sh"
  fi
  if stage_cells_selftest "$library_under_test" "$nested_run"; then exit 0; else exit 1; fi
fi
