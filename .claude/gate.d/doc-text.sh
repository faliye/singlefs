#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 文本写法：全仓不许出现登记过的禁用写法（撇号类角标、描述里的钟点与时区词、会话号）与登记过改名的术语旧名；改动碰到的记录与 kb 页日期只写在标题上
# gate-category: 文档类
# 不声明 gate-covers：.claude/gate-not-implemented.tsv 的两项判被测代码的行为，这一道只判文本写法。
# gate-similar: code-tooling.sh 它的 fixture-claims 格也扫全仓文件，但判的是文件名与样本目录里编出来的日期、阶段头部声称的样本在不在，不是文本里的写法
# gate-similar: code-source-discipline.sh 它的 vague-names 与 test-file-names 两格也判写法，但对象是 .rs 里声明的函数名、类型名与测试文件名，读的是空泛词表与按里程碑起名的正则，不扫全部文本
#
# gate-cell: prime-marks 撇号类角标
# gate-cell: clock-times 描述里的钟点、时区词与时间戳
# gate-cell: session-names 会话号（ListAgents 列的会话名）
# gate-cell: term-renames 登记过改名的术语不再出现旧名
# gate-cell: date-once 这次改动碰到的记录与 kb 页里，日期只在标题上：正文行不重复所属标题或文件名里的日期，同一天不开第二个同级小节
# 格结构经 lib/stage-cells.sh。
#
# ── prime-marks、clock-times、session-names ──
# 格名就是同目录 lib-forbidden-notations.py 里 FORMS 的形态键。git ls-files --cached --others --exclude-standard 列出的每份文本文件
# （不在 git 仓里时走目录树）逐行交那一形态判，违例报「文件:行」。判据行、上下文放行规则、放行标记、排除前缀、成功行与出路都在那份库里；
# 加一种禁用写法就是往 FORMS 加一项、再在格名表里加一格。.claude/hooks/write-guard.sh 写入时读同一份库。
# 射程是全仓，冻结证据（research/prompts/、records/、变更史、research/results/）也在内。
# 没扫的：二进制文件（前 8 KiB 里有 NUL）与列出了却不是普通文件的，逐个列名。一份文本文件都没扫到：退 77。
# 管不到的：ASCII 单引号当角标（与代码里的引号分不开）；改过标题的会话。
#
# ── term-renames ──
# 判法在 scripts/kb/sweep-term.py --check：按 .claude/kb/term-renames.md 查（「匹配」认整串、词边界、正则三种），
# 在 git 工作树里查 git 跟踪的与未跟踪而未忽略的文件、git 忽略的不查，不在 git 工作树里查整棵目录树；
# 排除表 .claude/term-rename-exempt 认整份豁免与按行豁免（<路径>:「<行内锚>」只豁免那一行），写法都在那份脚本文件头。
# 排除表指向不存在的路径、行内锚命中不是恰好一行、正则写不对，也判红。没有登记表：退 77。规则：.claude/rules/path-moves.md「改一个全仓术语：正文之外还有五处会红」。
# 管不到的：登记表里没登记的改名。
#
# ── date-once ──
# 这次改动碰到的（含未跟踪的）records/*.md 与 .claude/kb/ 下的 .md（两份变更史与 INDEX.md 不判）逐份交
# scripts/kb/doc-consolidate.py migrate 干跑，干跑改了什么就是违例；哪些日期不算，判法在那份脚本文件头。
# 基准取 scripts/gate/changed-paths.sh 的 gate 取法；不在 git 仓里整棵树都判；一份都没碰到：退 77。
# 规则：.claude/rules/changelog-format.md、kb-discipline 第 1 条。管不到的：别的日期该不该写、日期挪进标题之后小节切得对不对。
#
# 弄坏开关（打开后样本判错）：
#   GATE_TEXT_NOTATIONS_BREAK=date-once-unjudged      date-once 不干跑、记无对象可判：date-once-red 退 77
#   GATE_TEXT_NOTATIONS_BREAK=term-renames-unjudged   term-renames 不调 sweep-term、记无对象可判：term-renames-red 退 77
#   LIB_FORBIDDEN_NOTATIONS_BREAK=<判据行键或形态键>   关掉判据行：红样本里点名那一行的报不出来
#   LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=<判据行键或形态键>、LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=<形态键>、
#   LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=<形态键>  关掉上下文规则、不认放行标记、排除前缀失效：那一格的绿样本判红
#   （键写在 lib-forbidden-notations.py）
#   STAGE_CELLS_BREAK=red-swallowed 让每份红样本整道退 0；=marker-ignored 让每份样本几格都跑，「跑了 1 格」的 want 找不到
#
# 样本：fixtures/doc-text.sh/ 下十二份，每份样本根放 .gate-cells 只点名它判的那一格：
#   date-once-red       setup.sh 建 git 仓；这一轮往记录正文加一行带文件名日期的话、往实验页加第二个同日 `### ` 小节，逐份点名判红
#   date-once-green     同一基准；加的行只带别的日期、加的小节是另一天，判绿
#   prime-marks-red     一份带「K9」加一撇的 md、一份带「G5」加两撇的 rs，逐处点名「文件:行」判红
#   prime-marks-green   只有正常写法（变体起了新名字、ASCII 单引号），外加上游副本里带角标的 md（排除前缀下），判绿
#   clock-times-red     每条判据行各有一份带那种写法的 md 或 rs，外加写在 md 里与没写理由的放行标记，逐处点名「文件:行」与判据行判红
#   clock-times-green   只有日期与近似但该放行的写法，.sh、.py、.rs 里带放行标记的各一行，外加钟点排除前缀下的门禁样本、hook 自检与 c510 提示，判绿
#   session-names-red   标签式、当主语用的、上游三语仓的、长十六进制的会话号各一行，外加 rs 里没写理由的放行标记，逐处点名判红
#   session-names-green 近似但该放行的写法（crate 名、mktemp 模板与临时目录名、上游仓目录名、带字母后缀的标识符）与 .sh 里带放行标记的一行，
#                       外加会话号排除前缀下的门禁样本，判绿
#   term-renames-red    一份改名登记表与一份还用着旧名的 rs（旧名由几段拼出来，样本目录里不出现旧名的字面），判红
#   term-renames-green  同一张登记表，rs 里只有新名，判绿
#   term-renames-line-exempt-green  setup.sh 建 git 仓；登记表带一行「正则」，记录里只有树层、外层那一行与按行豁免的一句用户原话带旧名，
#                       git 忽略的 multi-host.env 里有旧名，判绿（正则去掉断言、扫了 git 忽略的文件都红；旧名同样由几段拼出来）
#   term-renames-line-exempt-red  同一现场，按行豁免的那份记录里另有一行没豁免的旧名：只报那一行（1 份文件、1 处），判红
#                       （按行豁免当成整份豁免就判绿，正则去掉断言、扫了 git 忽略的文件报的处数就不是 1）
#
#   bash .claude/gate.d/doc-text.sh [项目根]                                     五格都跑
#   bash .claude/gate.d/doc-text.sh --list                                       逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-text.sh --check <格名>[,<格名>…] [项目根]            只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
source "$STAGE_DIR/lib/stage-cells.sh"
LIBRARY="$STAGE_DIR/lib-forbidden-notations.py"
SWEEP_TERM="$REPO_ROOT/scripts/kb/sweep-term.py"
DOC_CONSOLIDATE="$REPO_ROOT/scripts/kb/doc-consolidate.py"
source "$REPO_ROOT/scripts/gate/changed-paths.sh"
text_notations_break="${GATE_TEXT_NOTATIONS_BREAK:-}"

stage_cell prime-marks cell_prime_marks "撇号类角标" \
  "给变体起一个新名字（那一族里下一个没用过的号，或一个短的描述性名字），撞号先查，全仓一次改完：.claude/rules/path-moves.md「变体起新名字，不用角标」"
stage_cell clock-times cell_clock_times "描述里的钟点、时区词与时间戳" \
  "时间只写到日期，钟点、时区词、时间戳连同「前后」一起删；代码真要用这些字面的那一行写「# clock-times:allow <理由>」，整份是被测输入的登记进 lib-forbidden-notations.py 的 excluded_prefixes"
stage_cell session-names cell_session_names "会话号（ListAgents 列的会话名）" \
  "会话号不写进仓：标签式的「（会话 <号>，<日期>）」只留日期，当主语用的写成「别的会话」这类不带号的说法；代码真要用的那一行写「# session-names:allow <理由>」"
stage_cell term-renames cell_term_renames "登记过改名的术语不再出现旧名" \
  "python3 scripts/kb/sweep-term.py --apply 一次换完（跑不出来的产物一个字节都不许动，连同引它的正文留旧名）；确实该留旧名的登记进 .claude/term-rename-exempt 并写明为什么"
stage_cell date-once cell_date_once "这次改动碰到的记录与 kb 页里，日期只在标题上：正文行不重复所属标题或文件名里的日期，同一天不开第二个同级小节" \
  "python3 scripts/kb/doc-consolidate.py migrate <那一份> --out <临时文件> 看它删了哪些、并了哪些，照着改；红行列的是每份删了几个日期、并了几个小节"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$REPO_ROOT}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级。"; exit 2; }

# scan_form <形态键>：按 lib-forbidden-notations.py 里那一形态扫全仓文本，退出码 0 没有违例、1 有违例（或读不到库）、77 一份文本文件都没扫到
scan_form() {
  python3 - "$LIBRARY" "$1" <<'PY'
import importlib.util, os, subprocess, sys

library_path, form_key = sys.argv[1], sys.argv[2]
try:
    spec = importlib.util.spec_from_file_location('lib_forbidden_notations', library_path)
    notations = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(notations)
except OSError as error:
    print(f'  ✗ 读不到禁用写法的登记 {library_path}：{error}')
    print('     → 怎么办：形态只在那一份里登记（.claude/hooks/write-guard.sh 读同一份），从 git 恢复它，别在阶段里再抄一份。')
    sys.exit(1)
if form_key not in notations.FORMS_BY_KEY:
    print(f'  ✗ 格名 {form_key} 在 lib-forbidden-notations.py 的 FORMS 里没有对应的形态')
    print('     → 怎么办：格名表与 FORMS 的键要逐个相同；在 FORMS 里加了或改了形态，就把阶段文件头的格名表与 stage_cell 登记一起改。')
    sys.exit(1)

listing = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'],
                         capture_output=True)
if listing.returncode == 0:
    candidate_paths = sorted({name for name in listing.stdout.decode('utf-8', 'replace').split('\0') if name})
else:
    candidate_paths = sorted(os.path.relpath(os.path.join(directory, name), '.')
                             for directory, subdirectories, names in os.walk('.')
                             if not subdirectories.__setitem__(slice(None), [entry for entry in subdirectories if entry != '.git'])
                             for name in names)

form = notations.FORMS_BY_KEY[form_key]
findings = []
scanned_paths = []
excluded_count = 0
binary_files, irregular_files = [], []
for relative_path in candidate_paths:
    if notations.is_excluded(form_key, relative_path):
        excluded_count += 1
        continue
    if not os.path.isfile(relative_path):
        irregular_files.append(relative_path)
        continue
    with open(relative_path, 'rb') as handle:
        content = handle.read()
    if b'\0' in content[:8192]:
        binary_files.append(relative_path)
        continue
    scanned_paths.append(relative_path)
    for line_number, line in enumerate(content.decode('utf-8', 'replace').split('\n'), 1):
        found = notations.first_hit(form_key, line, relative_path)
        if found:
            position, pattern_name, matched = found
            anchor = position + len(matched) if form['snippet_to_match_end'] else position
            findings.append((relative_path, line_number, pattern_name, line[max(0, position - 20):anchor + 10].strip()))

if findings:
    finding_files = sorted({path for path, _, _, _ in findings})
    print(f'  ✗ {len(findings)} {form["finding"]}，在 {len(finding_files)} 份文件里：')   # gate-lint:summary
    for path, line_number, pattern_name, snippet in findings[:200]:
        shown_pattern = f'  {pattern_name}' if form['detail_shows_pattern'] else ''
        print(f'      {path}:{line_number}{shown_pattern}  …{snippet}…')   # gate-lint:detail
    if len(findings) > 200:
        print(f'      （另有 {len(findings) - 200} 行没列出，每份文件的行数见下）')   # gate-lint:detail
        for path in finding_files:
            print(f'      {path}：{sum(1 for hit_path, _, _, _ in findings if hit_path == path)} 行')   # gate-lint:detail
    print('     → 怎么办：' + form['remedy'][0])
    for remedy_line in form['remedy'][1:]:
        print('               ' + remedy_line)
    sys.exit(1)
if not scanned_paths:
    print(f'  ! {form["name"]}这一形态本次无对象可判：没扫到一份文本文件（列出 {len(candidate_paths)} 个路径，'
          f'二进制 {len(binary_files)}、不是普通文件 {len(irregular_files)}、排除前缀下 {excluded_count}）')
    sys.exit(77)
print(f'  ✓ 查了 {len(scanned_paths)} 份文本文件，{form["clean"]}')
print('     没扫的：' + form['excluded_label'].format(count=excluded_count))
print(f'     二进制 {len(binary_files)} 份、列出了却不是普通文件 {len(irregular_files)} 份，这一形态没扫，逐个列在下面')
for path in binary_files:
    print(f'       二进制：{path}')
for path in irregular_files:
    print(f'       不是普通文件（工作区里删掉还没暂存的）：{path}')
PY
}

cell_prime_marks() {
  scan_form prime-marks
}

cell_clock_times() {
  scan_form clock-times
}

cell_session_names() {
  scan_form session-names
}

cell_term_renames() {
  if [[ "$text_notations_break" == term-renames-unjudged ]]; then
    echo "  ! 弄坏开关 term-renames-unjudged：这一格不调 sweep-term，记本次无对象可判"
    exit 77
  fi
  if [[ ! -f "$ROOT/.claude/kb/term-renames.md" ]]; then
    echo "  ! 没有 .claude/kb/term-renames.md，这一格本次无对象可判"
    exit 77
  fi
  if python3 "$SWEEP_TERM" --check "$ROOT"; then
    exit 0
  fi
  echo "  ✗ 登记过的术语改名，仓里还有地方用着旧名（上面逐份列出份数与处数），或排除表指向不存在的路径"
  echo "     → 怎么办：python3 scripts/kb/sweep-term.py --apply 一次换完。"
  echo "               产物分两类，别一把梭：**跑得出来的**（装置源码在树里、replay.sh 判 exact）跟着源码一起换，"
  echo "               换完跑 cargo test --workspace 与 bash scripts/experiments/replay.sh，同一份源码要仍然吐出逐字节相同的产物——"
  echo "               那是重新生成，不是改证据；**跑不出来的**（已归档、要虚机或真设备、别人的产物）一个字节都不许动，"
  echo "               连同引它的正文整段留旧名（evidence-discipline「原样保存的证据不许事后改」）。"
  echo "               确实该留旧名的（别家术语、冻结证据目录、历史类文件里换了就成假话的那一句、引文块、对照表自己），"
  echo "               登记进 .claude/term-rename-exempt 并写明为什么；历史类文件里只有一句该留的写 <路径>:「<那一行里的一段原文>」只豁免那一行，"
  echo "               整份都该留的逐文件登记，不整个目录豁免；指向不存在路径、行内锚命中不是恰好一行的那一条改掉或删掉。"
  exit 1
}

cell_date_once() {
  local base changed path count=0 red=0 scratch summary dates merges
  if [[ "$text_notations_break" == date-once-unjudged ]]; then
    echo "  ! 弄坏开关 date-once-unjudged：这一格不干跑，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    base="$(gate_diff_base gate)"
    changed="$(gate_changed_paths "$base" untracked)"
  else
    changed="$(find records .claude/kb -name '*.md' 2>/dev/null)"
  fi
  scratch="$(mktemp "${TMPDIR:-/tmp}/date-once.XXXXXX")"
  while IFS= read -r path; do
    [[ -n "$path" ]] || continue
    case "$path" in
      records/*.md) ;;
      .claude/kb/*.md) ;;
      *) continue ;;
    esac
    case "$path" in
      .claude/kb/decisions-history.md|.claude/kb/experiments-history.md|.claude/kb/INDEX.md) continue ;;
    esac
    [[ -f "$path" ]] || continue
    count=$((count + 1))
    summary="$(python3 "$DOC_CONSOLIDATE" migrate "$path" --out "$scratch" --force 2>&1 | head -1)"
    dates="$(sed -n 's/.*同日日期删 \([0-9]*\).*/\1/p' <<<"$summary")"
    merges="$(sed -n 's/.*同日小节合并 \([0-9]*\).*/\1/p' <<<"$summary")"
    if [[ "${dates:-0}" != 0 || "${merges:-0}" != 0 ]]; then
      red=$((red + 1))
      echo "  ✗ $path：正文里重复所属日期 ${dates:-0} 处、同一天的第二个小节 ${merges:-0} 个"   # gate-lint:detail
    fi
  done <<<"$changed"
  rm -f "$scratch"
  if ((red)); then
    echo "  ✗ 这次改动碰到的 $count 份记录与 kb 页里 $red 份日期没有只写在标题上"   # gate-lint:summary
    echo "     → 怎么办：python3 scripts/kb/doc-consolidate.py migrate <那一份> --out <临时文件>，看它删了哪些日期、并了哪些小节，照着改回仓里；"
    echo "               规则：.claude/rules/changelog-format.md「只有决策与实验有历史」与 kb-discipline 第 1 条。"
    exit 1
  fi
  if ((count == 0)); then
    echo "  ! 这次改动没碰到 records/ 与 .claude/kb/ 下的 .md，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 日期只在标题上：这次改动碰到的 $count 份记录与 kb 页干跑一份都没改"
}

stage_cells_run "$ROOT"
