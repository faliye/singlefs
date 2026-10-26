#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 scripts/gate/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: 内存序（herd7 + litmus/，本工程自己的阶段）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: checker-tier-qemu-device-streams.sh 同是 checker-tier 类里要外部工具的那几道之一，但它起 QEMU 判真设备上收到的写；这里起 herd7 判 litmus 的内存序声明，工具、判定的键与失败方式都不同
# gate-similar: code-source-discipline.sh 它静态判源码的写法与登记（名字、格式常量、实验源码纪律）；这里的 litmus 也要绑到 crates 的代码锚点，但判的是 herd7 对每条 Never / Sometimes 的判定与声明符不符
#
# gate-cell: litmus-verdicts litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符
# 判法全在 .claude/scripts/lkmm.sh，这里只接进门禁、给出路；参数、每格子 shell、汇总与退出码经 lib/stage-cells.sh。
# 上游不再跑 LKMM，litmus/ 的这几样只有这一道在判。
#
# 复用：每份 litmus 两条判定（herd7:<项名> 与 binding:<项名>；项名是相对 litmus/ 的路径，顶层的就是文件名，子目录里的带目录），键写在 .claude/scripts/lkmm.sh 文件头「判定库」一段：
#   ① herd7 那一条的键是 litmus 去注释后的内容、singlefs-expect 的值、herd7 版本与可执行文件的 sha256、内核树 tools/memory-model 的 .cfg / .cat / .bell / .def 内容摘要
#      与内核树的版本；锚点指到的代码不进（herd7 不读代码）。键对得上的复用、不起 herd7，对不上的现判。
#   ② 绑定那一条的键另含锚点指到的函数单元指纹（scripts/gate/lib_function_units.py 的切法，锚点写「文件 :: 单元限定名」、恰好命中一个单元）。
#   判定库在被判那棵树的 git common-dir 的 singlefs-lkmm-verdicts/（GATE_LKMM_VERDICT_LIBRARY 盖过它），跨轮留着；SINGLEFS_GATE_FULL=1 只写不读，现判的照键写回同一份库。
#   没有整道标记，不问 stage-must-run.sh。前提（stage-inputs.tsv 本阶段那一行第三列 probe=：找得到 herd7）没齐判红，不退 77。
#
# 样本 fixtures/checker-tier-lkmm.sh/<名>/（stage-selftest.sh 跑，各放 .gate-cells 点名 litmus-verdicts；只供测试的开关：.claude/rules/fs-design.md 五条硬要求第 2 条）：
#   样本根放 .lkmm-static-only：不起真 herd7。没有 .lkmm-fake-herd7/ 的只跑不需要 herd7 的静态层，全过这一格退 77，绿样本的 expect 写 exit=77；
#   另有 .lkmm-fake-herd7/（一份打固定版本串、按 litmus 里 fake-observation 那一行交判定的假 herd7，与只有 tools/memory-model 几份模型文件的假内核树）的，
#     herd7 那一层经假 herd7 跑，全过同样退 77（没起过真 herd7）；
#   再有 .lkmm-key-samples/change.sh 的：判定库放这一趟的临时目录，接连跑两趟，头一趟现判、记下，跑完 change.sh 改样本，第二趟按键复用或现判，
#     expect 认第二趟每条的「· <项>：绿|红，现判（…）|复用（…）」那一行。几份 litmus-key-samples-*：
#     comment-only（只改 litmus 的普通注释，herd7 那一条复用）、expectation-changed-red（改 singlefs-expect，herd7 那一条现判、与假 herd7 的判定不符判红）、
#     anchor-body-changed（改锚点那个 fn 的函数体，herd7 那一条复用、绑定那一条现判、点名变了的单元）；
#   anchor-impl-method-green（锚点写 <impl 类型>::<方法>，静态层判绿）、anchor-duplicate-red（同一限定名在那份文件里两处，判红）；
#   litmus-same-name-subdirectories-red（litmus/a/ 与 litmus/b/ 各一对同名的 publish.litmus 与对照，测试只写出 litmus/a/publish.litmus，静态层：
#     项名按相对 litmus/ 的路径分开，binding:a/publish.litmus 绿、binding:b/publish.litmus 红，判红）；
#   binding-mention-boundary-red（一对 publish.litmus，唯一的测试只写出 mp-publish.litmus，静态层：写出项名那一查看边界，不算写出 publish.litmus，判红）；
#   litmus-count-subdirectories（litmus/ 顶层一对 publish.litmus、子目录 litmus/nested/ 一对 handoff.litmus，经假 herd7、没有 change.sh 只跑一趟、不给判定库：
#     份数连子目录一起数，成功句报「4 份 litmus 全过」，退 77）。
# 弄坏开关（litmus-count-top-level-only 由这一道读，其余 lkmm.sh 读；只给样本证红用；一条一行「GATE_LKMM_BREAK=<值>  弄坏哪一处」，下一行起写打开后判错的样本与怎样错，没写的样本照 expect 判对）：
#   GATE_LKMM_BREAK=litmus-key-drops-every-comment  herd7 那一条的键把注释整个去掉、连 singlefs-expect 的值也不留（S27：改了期望照样复用）
#     litmus-key-samples-expectation-changed-red：判绿退 77，expect 里的 exit=1 与「现判（期望 变了」找不到。
#   GATE_LKMM_BREAK=reuse-lookup-skipped  一律不读库（键相同照样现判）；lkmm.sh 的复用自证 scripts/gate/lkmm-selftest.sh 的 reuse-three-questions 那一格同样判错
#     litmus-key-samples-comment-only、litmus-key-samples-anchor-body-changed：第二趟每条都现判，复用哨兵报「复用失灵」、退 1，
#       expect 里的 exit=77、「复用（」那几句与「· 按条」那一句找不到（第二趟报「现判 3、复用 0」）；
#     litmus-key-samples-expectation-changed-red：照旧退 1，「· herd7:publish.litmus：红，现判（期望 变了」与「· herd7:publish-nofence.litmus：绿，复用（」找不到。
#   GATE_LKMM_BREAK=litmus-items-by-file-name  项名只取文件名（改前），两个子目录里的同名 litmus 共用一个项名
#     litmus-same-name-subdirectories-red：照旧退 1，两份都报成 binding:publish.litmus、都红，
#       「· binding:a/publish.litmus：绿」「写出 b/publish.litmus」「代码绑定检查未过：1 项」那几句找不到。
#   GATE_LKMM_BREAK=binding-mention-substring  写出项名那一查退回按子串找（改前）
#     binding-mention-boundary-red：mp-publish.litmus 也算写出 publish.litmus，判绿退 77，expect 里的 exit=1 与「binding:publish.litmus：红」那几句找不到。
#   GATE_LKMM_BREAK=litmus-count-top-level-only  成功句里的份数只数 litmus/ 顶层（改前 -maxdepth 1），子目录里的 litmus 不算
#     litmus-count-subdirectories：照旧退 77，成功句报成「2 份 litmus 全过」，expect 里「4 份 litmus 全过」那一句找不到。
# 复用三问与复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）：哨兵在 lkmm.sh 里每一趟真跑都抽（绑定、herd7 各一份），被现判、强制现判判定不同这一格红；
#   复用三问那一格在 scripts/gate/lkmm-selftest.sh（登记表 .claude/gate.d/verdict-library-scripts.tsv 那一行）。上次红的清单由 lib/stage-cells.sh 按格写。
#
#   bash .claude/gate.d/checker-tier-lkmm.sh [项目根]
#   bash .claude/gate.d/checker-tier-lkmm.sh --list                       逐行打格名与判什么，不起 herd7
#   bash .claude/gate.d/checker-tier-lkmm.sh --check litmus-verdicts      只跑这一格；格名写错退 2
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
ADMISSION_MODULE="$STAGE_REPOSITORY/scripts/gate/admission.py"
LKMM="$(cd "$(dirname "$0")/../scripts" && pwd)/lkmm.sh"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell litmus-verdicts cell_litmus_verdicts "litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符" \
  "改 litmus 还是改代码先想清楚（判定与声明不符时先核 singlefs-models 指的那段代码今天发的次序）；缺 herd7 跑 bash .claude/scripts/fetch-deps.sh --check"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check litmus-verdicts 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2
ROOT="$(pwd)"

# ── 格 litmus-verdicts ──
cell_litmus_verdicts() {
[[ -f "$LKMM" ]] || { echo "  ✗ 找不到 $LKMM"; echo "     → 怎么办：它随仓走（.claude/scripts/lkmm.sh），被删了就从 git 找回来。"; exit 1; }
[[ -d "$ROOT/litmus" ]] || { echo "  ! $ROOT 下没有 litmus/，本阶段跳过"; exit 77; }
# 样本根（不是这道阶段所在的仓、也没设 GATE_STAGED_FROM）才认 .lkmm-static-only 与它旁边的假 herd7（文件头「样本」一段）
sample_root=0
[[ -f "$ROOT/.lkmm-static-only" ]] && sample_root=1
static_only=(); fake_tools=0; rounds=1; library_options=(); temporary_library=""; fake_tools_directory=""
# 样本建的两个临时目录（这一趟的判定库、假 herd7 的替身）：这一格的子 shell 退出时删（哪条路退都删）
cleanup_lkmm_temporaries() {
  [[ -n "$temporary_library" ]] && rm -rf "${temporary_library:?}"
  [[ -n "$fake_tools_directory" ]] && rm -rf "${fake_tools_directory:?}"
  return 0
}
trap cleanup_lkmm_temporaries EXIT
if ((sample_root)); then
  if [[ -f "$ROOT/.lkmm-fake-herd7/herd7" && "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then
    fake_tools=1
    # 样本里的文件不带执行位：PATH 上放一个经 bash 起它的替身（临时目录，跑完删）
    fake_tools_directory="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
    printf '#!/usr/bin/env bash\nexec bash %q "$@"\n' "$ROOT/.lkmm-fake-herd7/herd7" > "$fake_tools_directory/herd7"
    chmod +x "$fake_tools_directory/herd7"
    export PATH="$fake_tools_directory:$PATH" SINGLEFS_KERNEL_TREE="$ROOT/.lkmm-fake-herd7/linux" SINGLEFS_NO_FETCH=1
    if [[ -f "$ROOT/.lkmm-key-samples/change.sh" ]]; then
      temporary_library="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
      library_options=(--verdict-library "$temporary_library")
      rounds=2
    fi
  else
    static_only=(--static-only)
  fi
else
  python3 "$ADMISSION_MODULE" gate-preconditions "$STAGE_REPOSITORY" "$(basename "$0")" || exit 1
  # 判定库：被判那棵树的 git common-dir 里（跨轮留着，几棵 worktree 共用），GATE_LKMM_VERDICT_LIBRARY 盖过它；SINGLEFS_GATE_FULL=1 只写不读
  verdict_library="${GATE_LKMM_VERDICT_LIBRARY:-}"
  if [[ -z "$verdict_library" ]] && common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    verdict_library="$common_directory/singlefs-lkmm-verdicts"
  fi
  if [[ -n "$verdict_library" ]]; then
    library_options=(--verdict-library "$verdict_library")
    [[ "${SINGLEFS_GATE_FULL:-}" == 1 ]] && library_options+=(--no-reuse)
  else
    echo "  ! 取不到 $ROOT 的 git common-dir，判定库没处放：这一趟每一条都现判、不记"
  fi
fi
rc=0
for ((round = 1; round <= rounds; round++)); do
  if ((round == 2)); then
    echo "── 第二趟（样本 .lkmm-key-samples/change.sh 改过之后；判定库 $temporary_library 里是头一趟记下的）"
    if ! (cd "$ROOT" && bash .lkmm-key-samples/change.sh); then
      echo "  ✗ 样本的 .lkmm-key-samples/change.sh 没跑成"
      echo "     → 怎么办：照它的报错改样本；它只许改样本根里的 litmus 与代码。"
      exit 1
    fi
  elif ((rounds == 2)); then
    echo "── 头一趟（样本 .lkmm-key-samples：判定库 $temporary_library，现判之后记下）"
  fi
  bash "$LKMM" "$ROOT" ${static_only[@]+"${static_only[@]}"} ${library_options[@]+"${library_options[@]}"}
  rc=$?
  if ((round < rounds && rc != 0)); then
    echo "  ✗ 样本头一趟就没过（lkmm.sh 退 $rc），第二趟比不出复用"
    echo "     → 怎么办：先让样本不带 change.sh 的那一份判绿，再放 .lkmm-key-samples/change.sh。"
    exit 1
  fi
done
if [[ ${#static_only[@]} -gt 0 ]]; then
  # lkmm.sh 静态档全过退 3：herd7 那一层没判，记本次未跑（77）；共用库把 0、77 之外的码都算红。别的码原样交出（1 是红）
  if [[ "$rc" == 3 ]]; then
    echo "  ! 静态档（.lkmm-static-only）：herd7 那一层没判，这一格本次未跑"
    exit 77
  fi
  exit "$rc"
fi
if [[ "$rc" -ne 0 ]]; then
  echo "  ✗ 内存序判红（细节在上面 lkmm.sh 的输出里）"
  echo "     → 怎么办：改 litmus 还是改代码先想清楚：判定与声明不符时先核 singlefs-models 指的那段代码今天发的次序，"
  echo "               再决定是 litmus 没跟上代码、还是代码把屏障漏了；缺 herd7 就跑 bash .claude/scripts/fetch-deps.sh --check。"
  exit 1
fi
# 份数与 lkmm.sh 判的那一批同一种找法：litmus/ 下递归找 .litmus（子目录里的也数），按 NUL 数
# 弄坏开关 GATE_LKMM_BREAK=litmus-count-top-level-only 退回改前：只数 litmus/ 顶层（文件头「弄坏开关」一段）
if [[ ",${GATE_LKMM_BREAK:-}," == *,litmus-count-top-level-only,* ]]; then
  litmus_file_count="$(find "$ROOT/litmus" -maxdepth 1 -name '*.litmus' -type f | wc -l)"
else
  litmus_file_count="$(find "$ROOT/litmus" -name '*.litmus' -printf '%P\0' | tr -cd '\0' | wc -c)"
fi
if ((fake_tools)); then
  echo "  ! 假 herd7（样本 .lkmm-fake-herd7）：$litmus_file_count 份 litmus 全过，没起过真 herd7，这一格本次未跑"
  exit 77
fi
echo "  ✓ 内存序：litmus/ 下 $litmus_file_count 份 litmus 里每条 Never 有对照组、绑到代码，herd7 判定与声明相符（逐条的判定与复用在上面 lkmm.sh 的输出里）"
}

stage_cells_run "$ROOT"
