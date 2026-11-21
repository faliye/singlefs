#!/usr/bin/env bash
# 门禁 doc-text 的格函数库：prime-marks、clock-times、session-names、date-once、trilingual-files、trilingual-structure、trilingual-changed-together、trilingual-glossary、trilingual-untranslated、trilingual-omission、private-values、account-quota、task-and-session-numbers。
# 由 .claude/gate.d/doc-text.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# scan_form <形态键>：按 lib-forbidden-notations.py 里那一形态扫全仓文本，退出码 0 没有违例、1 有违例（或读不到库）、77 一份文本文件都没扫到
scan_form() {
  private_value_sources
  python3 - "$LIBRARY" "$1" "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" "$text_notations_break" "$REPO_ROOT" <<'PY'
import errno, importlib.util, multiprocessing, os, subprocess, sys, types

library_path, form_key, private_values_source, configuration_path, hostname_path, broken, repository_root = sys.argv[1:8]
if broken == 'scan-prefilter-rejects-all':   # 弄坏：库里的逐行先筛一律说没有候选（LIB_FORBIDDEN_NOTATIONS_BREAK 的 candidate-prefilter-rejects-all），库在下面 fork 之前读环境
    os.environ['LIB_FORBIDDEN_NOTATIONS_BREAK'] = ','.join(filter(None, [os.environ.get('LIB_FORBIDDEN_NOTATIONS_BREAK', ''), 'candidate-prefilter-rejects-all']))
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

# 红行与「没扫的」清单里打的路径都经 private-values 那一份打码（路径里含私有值的那一段打成〔文件名含私有值〕，片段里的值打成〔私有值〕）；
# 没有 multi-host.env（别人的克隆、第二台）或读不了就没有词可打，原样打
private_values = types.ModuleType("lib_private_values")
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
masking_targets = []
if broken != "private-values-siblings-unmasked" and configuration_path:
    try:
        masking_targets = private_values.load_private_targets(configuration_path, hostname_path, broken)[0]
    except OSError:
        masking_targets = []


def shown_path(path):
    return private_values.shown_path(path, masking_targets, broken)


form = notations.FORMS_BY_KEY[form_key]


def scan_one(relative_path):
    """一份文件的判法，交 (状态, 内容)：excluded、irregular、binary，unreadable（内容是 errno），scanned（内容是 [(行号, 判据行名字, 片段)]）。
    各份文件互不依赖，下面按派出的次序收回；每行的判法是 notations.hits_per_line（与逐行 first_hit 逐项相同）。"""
    if notations.is_excluded(form_key, relative_path):
        return 'excluded', None
    if not os.path.isfile(relative_path):
        return 'irregular', None
    try:
        with open(relative_path, 'rb') as handle:
            content = handle.read()
    except OSError as error:
        if broken == 'private-values-unreadable-raises':   # 弄坏：读不了就让格崩
            raise
        return 'unreadable', error.errno
    if b'\0' in content[:8192]:
        return 'binary', None
    found = []
    for line_number, line, position, pattern_name, matched in notations.hits_per_line(form_key, content.decode('utf-8', 'replace'), relative_path):
        anchor = position + len(matched) if form['snippet_to_match_end'] else position
        start, end = max(0, position - 20), anchor + 10
        if broken != 'private-values-snippet-cut':   # 两头落在私有值中间的挪到值的边上，整值进片段再打码
            start, end = private_values.snippet_bounds(line, start, end, masking_targets)
        found.append((line_number, pattern_name, line[start:end].strip()))
    return 'scanned', found


# 扫文件的进程数：每台默认重活线程数（SINGLEFS_THREAD_CAP 设了照它，不然 .claude/heavy-thread-default）；取不到往标准错误打一行、一个进程扫
sys.path.insert(0, os.path.join(repository_root, 'scripts', 'hosts'))
try:
    import heavy_threads
    scan_processes = max(1, int(heavy_threads.default_heavy_threads()))
except (ImportError, ValueError) as error:   # HeavyThreadDefaultError 是 ValueError
    print(f'  ! 取不到每台默认重活线程数（{error}），一个进程扫', file=sys.stderr)
    scan_processes = 1
scan_processes = min(scan_processes, len(candidate_paths))
if scan_processes > 1:
    sys.stdout.flush()
    sys.stderr.flush()
    with multiprocessing.get_context('fork').Pool(processes=scan_processes) as pool:
        scan_results = pool.map(scan_one, candidate_paths, chunksize=1)
else:
    scan_results = [scan_one(relative_path) for relative_path in candidate_paths]
if len(scan_results) != len(candidate_paths):
    print(f'  ✗ 派出 {len(candidate_paths)} 份文件、收回 {len(scan_results)} 份结果，这一格没判完')
    print('     → 怎么办：再跑一次；还是对不上就查扫文件的子进程为什么丢了结果（内存不够被杀、磁盘满）。')
    sys.exit(1)
findings = []
scanned_paths = []
excluded_count = 0
binary_files, irregular_files, unreadable_files = [], [], []   # unreadable_files：(路径, errno)，读不了的（权限这一类）不让异常回溯把原路径打出来，路径经打码列进「没扫的」
for relative_path, (state, detail) in zip(candidate_paths, scan_results):
    if state == 'excluded':
        excluded_count += 1
    elif state == 'irregular':
        irregular_files.append(relative_path)
    elif state == 'unreadable':
        unreadable_files.append((relative_path, detail))
    elif state == 'binary':
        binary_files.append(relative_path)
    else:
        scanned_paths.append(relative_path)
        findings.extend((relative_path, line_number, pattern_name, snippet) for line_number, pattern_name, snippet in detail)

if findings:
    finding_files = sorted({path for path, _, _, _ in findings})
    print(f'  ✗ {len(findings)} {form["finding"]}，在 {len(finding_files)} 份文件里：')   # gate-lint:summary
    for path, line_number, pattern_name, snippet in findings[:200]:
        shown_pattern = f'  {pattern_name}' if form['detail_shows_pattern'] else ''
        print(f'      {shown_path(path)}:{line_number}{shown_pattern}  …{private_values.shown_text(snippet, masking_targets)}…')   # gate-lint:detail
    if len(findings) > 200:
        print(f'      （另有 {len(findings) - 200} 行没列出，每份文件的行数见下）')   # gate-lint:detail
        for path in finding_files:
            print(f'      {shown_path(path)}：{sum(1 for hit_path, _, _, _ in findings if hit_path == path)} 行')   # gate-lint:detail
    print('     → 怎么办：' + form['remedy'][0])
    for remedy_line in form['remedy'][1:]:
        print('               ' + remedy_line)
    sys.exit(1)
if not scanned_paths:
    print(f'  ! {form["name"]}这一形态本次无对象可判：没扫到一份文本文件（列出 {len(candidate_paths)} 个路径，'
          f'二进制 {len(binary_files)}、不是普通文件 {len(irregular_files)}、读不了 {len(unreadable_files)}、排除前缀下 {excluded_count}）')
    sys.exit(77)
print(f'  ✓ 查了 {len(scanned_paths)} 份文本文件，{form["clean"]}')
print('     没扫的：' + form['excluded_label'].format(count=excluded_count))
print(f'     二进制 {len(binary_files)} 份、列出了却不是普通文件 {len(irregular_files)} 份、读不了 {len(unreadable_files)} 份，这一形态没扫，逐个列在下面')
for path in binary_files:
    print(f'       二进制：{shown_path(path)}')
for path in irregular_files:
    print(f'       不是普通文件（工作区里删掉还没暂存的）：{shown_path(path)}')
for path, error_number in unreadable_files:
    print(f'       读不了（{errno.errorcode.get(error_number, error_number)}，补上读权限再跑才扫得到）：{shown_path(path)}')
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

# date-once 红行里的路径经 private-values 那一份打码（shown_lines_main 的 path）
date_once_shown_path() {
  printf '%s\n' "$1" | python3 -c 'import sys, types; library = types.ModuleType("lib_private_values"); exec(compile(sys.argv[1], "private-values-library", "exec"), library.__dict__); library.shown_lines_main(sys.argv[2:], sys.stdin, sys.stdout)' \
    "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" "$text_notations_break" path
}

# date_once_jobs：打 date-once 干跑的并行路数（每台默认重活线程数）。样本根里放 .sample-thread-cap-invalid 的，按 SINGLEFS_THREAD_CAP 设成了不是正整数的值取，造出取不出登记值的样子；
# 只在样本根里认（PRIVATE_IN_SAMPLE，与私有配置认样本同一种判法），真仓里放了它也照常取。
date_once_jobs() {
  if ((PRIVATE_IN_SAMPLE)) && [[ -f "$ROOT/.sample-thread-cap-invalid" ]]; then
    SINGLEFS_THREAD_CAP=not-a-number heavy_thread_default
  else
    heavy_thread_default
  fi
}

cell_date_once() {
  local base changed path count=0 red=0 unjudged=0 rc reason scratch summary dates merges shown excluded
  if [[ "$text_notations_break" == date-once-unjudged ]]; then
    echo "  ! 弄坏开关 date-once-unjudged：这一格不干跑，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  private_value_sources
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    base="$(gate_diff_base gate)"
    changed="$(text_changed_paths "$base")" || { echo "  ✗ 取不到这次改动碰了哪些路径（基准 ${base:-空}）"; echo "     → 怎么办：在 git 仓里跑；基准与路径的取法只在 scripts/gate/changed-paths.sh 里有一份，它报了什么先照它的输出处理。"; exit 1; }
  else
    changed="$(find records .claude/kb -name '*.md' 2>/dev/null)"
  fi
  # 不判的：两份变更史与 INDEX.md 连同它们的英语、日语译文。译文路径照 lib-trilingual.py 的 language_paths 取（<名>.en.md、<名>.ja.md，
  # .claude/kb/translation-paths.json 给译文另起了名的照它），不另抄一份认法；弄坏开关 date-once-history-translations-judged 只排除中文那三份。
  if ! excluded="$(python3 -B - "$TRILINGUAL" "$text_notations_break" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("lib_trilingual", sys.argv[1])
trilingual = importlib.util.module_from_spec(spec)
spec.loader.exec_module(trilingual)
for name in ("decisions-history", "experiments-history", "INDEX"):
    paths = trilingual.language_paths(".claude/kb", name)
    for language, _ in trilingual.LANGUAGES:
        if language == "zh" or sys.argv[2] != "date-once-history-translations-judged":
            print(paths[language])
PY
)"; then
    echo "  ✗ 取不到不判的那几份（两份变更史与 INDEX.md 连同英日译文）的路径：同目录 lib-trilingual.py 装不进来或 language_paths 出错（原因在上面几行）"
    echo "     → 怎么办：照上面的报错修 lib-trilingual.py 或 .claude/kb/translation-paths.json，再跑这一格；不退回只排除中文那三份。"
    exit 1
  fi
  # 并行路数取 heavy_thread_default（用这一道所在仓里那一份）；取不出就红，不退回按核数、不退回一路。弄坏开关 date-once-jobs-unregistered-passes 取不出时退回一路、照常判
  local jobs
  if ! source "$REPO_ROOT/scripts/hosts/heavy-threads.sh" || ! jobs="$(date_once_jobs)"; then
    if [[ "$text_notations_break" == date-once-jobs-unregistered-passes ]]; then
      jobs=1
    else
      echo "  ✗ 取不了每台默认重活线程数（原因在上一行），date-once 的干跑并行度定不了，一份都没判"
      echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
      exit 1
    fi
  fi
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/date-once.XXXXXX")"
  # 各份文件的干跑互不依赖：并行起（至多每台默认重活线程数那么多路同时在跑），输出与退出码各写各的文件，下面按原来的次序逐份收
  local index=0 selected_count
  local -a selected=() pids=()
  while IFS= read -r path; do
    [[ -n "$path" ]] || continue
    case "$path" in
      records/*.md) ;;
      .claude/kb/*.md) ;;
      *) continue ;;
    esac
    [[ $'\n'"$excluded"$'\n' == *$'\n'"$path"$'\n'* ]] && continue
    [[ -f "$path" ]] || continue
    selected+=("$path")
  done <<<"$changed"
  selected_count="${#selected[@]}"
  for ((index = 0; index < selected_count; index++)); do
    if ((index >= jobs)); then wait "${pids[index - jobs]}" || true; fi
    (
      rc=0
      DOC_CONSOLIDATE_LEDGER=off python3 "$DOC_CONSOLIDATE" migrate "${selected[index]}" --out "$scratch/$index.migrated" --force > "$scratch/$index.out" 2>&1 || rc=$?
      echo "$rc" > "$scratch/$index.rc"
    ) &
    pids+=("$!")
  done
  for ((index = 0; index < selected_count; index++)); do
    path="${selected[index]}"
    wait "${pids[index]}" || true
    count=$((count + 1))
    # 干跑的退出码要收：0 与 3（丢失核对找不到东西）才有那一行摘要；别的（5 认不出历史节、1 读不了那一份这类）是没判，不当干跑 0 处
    rc=1
    [[ -f "$scratch/$index.rc" ]] && rc="$(<"$scratch/$index.rc")"
    summary="$(head -1 "$scratch/$index.out" 2>/dev/null)"
    if ((rc != 0 && rc != 3)) && [[ "$text_notations_break" != date-once-exit-ignored ]]; then
      unjudged=$((unjudged + 1))
      shown="$(date_once_shown_path "$path")"
      reason="$(grep -m1 -E '✗|Error' "$scratch/$index.out" 2>/dev/null || true)"
      echo "  ✗ ${shown:-〔路径打码没打出来〕}：doc-consolidate.py 干跑退 $rc，这一份没判（${reason:0:200}）"   # gate-lint:detail
      continue
    fi
    dates="$(sed -n 's/.*同日日期删 \([0-9]*\).*/\1/p' <<<"$summary")"
    merges="$(sed -n 's/.*同日小节合并 \([0-9]*\).*/\1/p' <<<"$summary")"
    if [[ "${dates:-0}" != 0 || "${merges:-0}" != 0 ]]; then
      red=$((red + 1))
      shown="$(date_once_shown_path "$path")"
      echo "  ✗ ${shown:-〔路径打码没打出来〕}：正文里重复所属日期 ${dates:-0} 处、同一天的第二个小节 ${merges:-0} 个"   # gate-lint:detail
    fi
  done
  rm -rf "${scratch:?}"
  if ((unjudged)); then
    echo "  ✗ 这次改动碰到的 $count 份记录与 kb 页里 $unjudged 份干跑出错、没判（scripts/kb/doc-consolidate.py 退的不是 0 也不是 3）"   # gate-lint:summary
    echo "     → 怎么办：照上面每一份括号里的报错修：退 5 是认不出历史节（翻译术语表读不到，或没有中文列是「历史版本」的那一行，补那一行）；"
    echo "               别的退出码照 doc-consolidate.py 的报错修那一份文件或那份脚本，再跑这一格。出错的那几份不算判过。"
    ((red)) || exit 1
  fi
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

# 三语六格的判法都在 lib-trilingual.py；弄坏开关 trilingual-<格名>-unjudged 让这一格不判、记无对象可判（退 77）
trilingual_unjudged() {
  if [[ "$text_notations_break" == "trilingual-$1-unjudged" ]]; then
    echo "  ! 弄坏开关 trilingual-$1-unjudged：这一格不判，记本次无对象可判"
    exit 77
  fi
}

cell_trilingual_files() {
  trilingual_unjudged files
  python3 "$TRILINGUAL" files "$ROOT"
  exit $?
}

cell_trilingual_structure() {
  trilingual_unjudged structure
  python3 "$TRILINGUAL" structure "$ROOT"
  exit $?
}

cell_trilingual_changed_together() {
  local base changed
  trilingual_unjudged changed-together
  cd "$ROOT" || exit 1
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 不在 git 仓里，拿不到这次改动碰了哪些路径，这一格无对象可判"
    exit 77
  fi
  base="$(gate_diff_base gate)"
  if ! changed="$(text_changed_paths "$base")"; then
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 ${base:-空}）"   # gate-lint:summary
    echo "     → 怎么办：在 git 仓里跑；基准与路径的取法只在 scripts/gate/changed-paths.sh 里有一份，它报了什么先照它的输出处理。"
    exit 1
  fi
  python3 "$TRILINGUAL" changed-together "$ROOT" <<<"$changed"
  exit $?
}

cell_trilingual_glossary() {
  trilingual_unjudged glossary
  python3 "$TRILINGUAL" glossary "$ROOT"
  exit $?
}

# 判法在 lib-trilingual.py 的 untranslated；这一道的开关 trilingual-untranslated-<项>（-unjudged 之外）转成 LIB_TRILINGUAL_BREAK 同名一项交给它，
# 开关只在样本根上认（stage_cells_run 开头那一判已经拦过真仓）。
cell_trilingual_untranslated() {
  local library_break="${LIB_TRILINGUAL_BREAK:-}"
  trilingual_unjudged untranslated
  case "$text_notations_break" in
    trilingual-untranslated-quotes-kept|trilingual-untranslated-code-kept|trilingual-untranslated-japanese-unjudged|trilingual-untranslated-exempt-ignored|trilingual-untranslated-japanese-threshold-five|trilingual-untranslated-identical-ignored)
      library_break="$text_notations_break" ;;
  esac
  LIB_TRILINGUAL_BREAK="$library_break" python3 "$TRILINGUAL" untranslated "$ROOT"
  exit $?
}

cell_trilingual_omission() {
  trilingual_unjudged omission
  python3 "$TRILINGUAL" omission "$ROOT"
  exit $?
}

cell_private_values() {
  local base changed configuration_file changed_file hostname_file size_limit_file python_rc=0
  if [[ "$text_notations_break" == private-values-unjudged ]]; then
    echo "  ! 弄坏开关 private-values-unjudged：这一格不比对，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  private_value_sources
  configuration_file="$PRIVATE_CONFIGURATION_FILE" hostname_file="$PRIVATE_HOSTNAME_FILE" size_limit_file="$PRIVATE_SIZE_LIMIT_FILE"
  if [[ ! -f "$configuration_file" ]]; then
    echo "  ! 这台机器没有 multi-host.env（别人的克隆、第二台，或还没配多机），没有私有值可比，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 被判的根不在 git 仓里，划不出「这一次改动新加的行」，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  base="$(gate_diff_base gate)"
  changed="$(text_changed_paths "$base")" || { echo "  ✗ 取不到这一轮碰了哪些文件（基准 $base）"; echo "     → 怎么办：看 git 是不是可用、基准提交在不在；改动范围只经 scripts/gate/changed-paths.sh 取。"; exit 1; }
  changed_file="$(mktemp "${TMPDIR:-/tmp}/private-values.XXXXXX")" || { echo "  ✗ 建不了放改动文件清单的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  printf '%s\n' "$changed" > "$changed_file"
  python3 - "$base" "$configuration_file" "$hostname_file" "$changed_file" "$text_notations_break" "$size_limit_file" "$PRIVATE_VALUES_LIBRARY" "$text_diff_cache" <<'PY' || python_rc=$?
import errno, os, re, subprocess, sys, types

base, configuration_path, hostname_path, changed_path, broken, size_limit_path, private_values_source, diff_cache_path = sys.argv[1:9]
private_values = types.ModuleType("lib_private_values")      # 取词与路径打码的共用函数（doc-text.sh 的 PRIVATE_VALUES_LIBRARY；prime-marks / clock-times / session-names 三格打路径时用同一份）
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
HOSTNAME_KEY = private_values.HOSTNAME_KEY
FILE_NAME_MASK = private_values.FILE_NAME_MASK
HEAD_BYTES = 8192                           # 前 8 KiB 里有 NUL 的文件按二进制整份跳过
TRUNCATED_BYTES = 8 * 1024 * 1024           # 只有弄坏开关 private-values-big-file-truncated 才用：未跟踪文件只读前 8 MiB


def fail(what, howto):
    print(f"  ✗ {what}")
    print(f"     → 怎么办：{howto}")
    sys.exit(1)


UNTRACKED_SIZE_LIMIT = 1024 * 1024 * 1024      # 未跟踪文件整份逐行比，超过这个大小判红、不截断；样本根里的 .private-values-size-limit（字节数）顶替它，样本不必造出一个 GiB 级的文件
if size_limit_path:
    try:
        UNTRACKED_SIZE_LIMIT = int(open(size_limit_path, encoding="utf-8").read().strip())
    except (OSError, ValueError) as error:
        fail(f"读不了样本的大小上限 {size_limit_path}：{error}", "样本根里的 .private-values-size-limit 只写一个字节数。")


try:
    targets, short_words, configuration_word_count, hostname_note, MINIMUM_LENGTH, shared_words = private_values.load_private_targets(configuration_path, hostname_path, broken)
except OSError as error:
    fail(f"读不了 {configuration_path}：{error}", "看它的权限；这一格要读它里面每个键的值（只比对，不打印）。")
if not targets:
    print(f"  ! multi-host.env 里没有 {MINIMUM_LENGTH} 个字以上的词，本机主机名也不取，没有私有值可比，这一格本次无对象可判（这不是通过）")
    sys.exit(77)


def git(*arguments):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True)


untracked_listing = git("ls-files", "-z", "--others", "--exclude-standard")
if untracked_listing.returncode != 0:
    fail("列不出未跟踪的文件", "看 git 是不是可用；改动范围只经 scripts/gate/changed-paths.sh 取。")
untracked = {name for name in untracked_listing.stdout.decode("utf-8", "replace").split("\0") if name}
configuration_real = os.path.realpath(configuration_path)
changed_paths = [name for name in open(changed_path, encoding="utf-8").read().split("\n") if name]

def shown_path(path):
    """红行里的路径（私有值按整串在整条路径里找、命中的那一段打成 FILE_NAME_MASK，值不打印）：判法在共用函数 private_values.shown_path。"""
    return private_values.shown_path(path, targets, broken)


diff_segments = private_values.load_diff_segments(diff_cache_path, base, broken)   # 阶段预取的整份 diff 按文件切开；没预取到的文件 added_lines 退回逐份 git diff
whole_file_limit = TRUNCATED_BYTES if broken == "private-values-big-file-truncated" else None

findings = []             # (路径, 行号, 键名列表, 词)
name_findings = []        # (路径, 键名列表)：文件名本身含私有值
too_big = []              # (路径, 字节数)：未跟踪、超过 UNTRACKED_SIZE_LIMIT，没法整份比
link_findings = []        # (路径, 键名列表, 词)：符号链接的链接文字含私有值（git 进仓的是链接文字本身）
unreadable = []           # (路径, 错误)：读不了的文件（权限这一类），没法比对；路径经打码，不让异常回溯把原路径打出来
files_judged, lines_judged, links_judged, binary_files = 0, 0, 0, []
for path in changed_paths:
    if os.path.islink(path) and broken != "private-values-links-followed":
        # 符号链接比路径与链接文字，不跟进去：坏链接也比，好链接不读它指到的内容
        name_keys = sorted({key for key, word in targets if word in path}) if broken != "private-values-names-unjudged" else []
        if name_keys:
            name_findings.append((path, name_keys))
        try:
            link_text = os.readlink(path)
        except OSError as error:
            unreadable.append((path, error))
            continue
        links_judged += 1
        keys = sorted({key for key, word in targets if word in link_text})
        if keys:
            link_findings.append((path, keys, [word for key, word in targets if word in link_text]))
        continue
    if not os.path.isfile(path) or os.path.realpath(path) == configuration_real:
        continue
    name_keys = sorted({key for key, word in targets if word in path}) if broken != "private-values-names-unjudged" else []
    if name_keys:
        name_findings.append((path, name_keys))
    try:
        with open(path, "rb") as handle:
            head = handle.read(HEAD_BYTES)
        if b"\0" in head:
            binary_files.append(path)
            continue
        whole_file = path in untracked or broken == "private-values-whole-tree"
        size = os.path.getsize(path)
        if whole_file and size > UNTRACKED_SIZE_LIMIT and broken != "private-values-big-file-truncated":
            too_big.append((path, size))
            continue
        files_judged += 1
        for number, text in private_values.added_lines(path, whole_file, base, git, fail, shown_path, diff_segments, whole_file_limit):
            lines_judged += 1
            keys = sorted({key for key, word in targets if word in text})
            if keys:
                findings.append((path, number, keys, [word for key, word in targets if word in text]))
    except OSError as error:
        if broken == "private-values-unreadable-raises":   # 弄坏：读不了就让格崩，回溯里带原路径
            raise
        unreadable.append((path, error))

if findings:
    print(f"  ✗ 这次改动新加的 {len(findings)} 行里出现了本机私有值（multi-host.env 里 {MINIMUM_LENGTH} 个字以上的词，或本机主机名）；只列文件、行号与键名，不打印值：")   # gate-lint:summary
    for path, number, keys, words in findings:
        shown = f"（值：{'、'.join(words)}）" if broken == "private-values-print-values" else "（值不打印）"
        print(f"      {shown_path(path)}:{number}  键名：{'、'.join(keys)}{shown}")   # gate-lint:detail
    print("     → 怎么办：把命中的词换成「第二台」「本机」这类说法（要引整句的，整句里的值也换掉）；值只留在 multi-host.env，别写进记录、日志摘录、提示和代码。")
    print("               红行没打印值，对着行号和键名去 multi-host.env 找；路径里哪一段含私有值，那一段打成" + FILE_NAME_MASK + "；commit message 这一格看不到，提交说明里同样别写。")
if name_findings:
    print(f"  ✗ 这次改动里 {len(name_findings)} 个文件的名字含本机私有值（路径里含值的那一段打成{FILE_NAME_MASK}，值不打印）：")   # gate-lint:summary
    for path, keys in name_findings:
        print(f"      {shown_path(path)}  键名：{'、'.join(keys)}（值不打印）")   # gate-lint:detail
    print("     → 怎么办：文件名里的值换成「第二台」「本机」这类说法再改名（用 git status 看这次新加、改动的文件名，对着目录和键名找）；产物的文件名由脚本拼出来的，改脚本别拿私有值拼名字。")
if too_big:
    print(f"  ✗ 这次改动里 {len(too_big)} 个未跟踪的文件超过 {UNTRACKED_SIZE_LIMIT} 字节，这一格没法整份比对（截断会漏掉后面的行，不截断）：")   # gate-lint:summary
    for path, size in too_big:
        print(f"      {shown_path(path)}  {size} 字节")   # gate-lint:detail
    print(f"     → 怎么办：把它拆成每份不超过 {UNTRACKED_SIZE_LIMIT} 字节的几份，或去掉进度行这类重复内容后重新落盘；值不要进仓、不要进日志原样输出。")
if link_findings:
    print(f"  ✗ 这次改动里 {len(link_findings)} 个符号链接的链接文字含本机私有值（git 进仓的是链接文字本身，不是它指到的内容；值不打印）：")   # gate-lint:summary
    for path, keys, words in link_findings:
        shown = f"（值：{'、'.join(words)}）" if broken == "private-values-print-values" else "（值不打印）"
        print(f"      {shown_path(path)}  链接目标  键名：{'、'.join(keys)}{shown}")   # gate-lint:detail
    print("     → 怎么办：别把指到第二台路径、挂载点的链接放进仓里：要留的产物拷一份进来（内容里的值照样换成「第二台」「本机」这类说法），只是图找起来方便的链接删掉。")
if unreadable:
    print(f"  ✗ 这次改动里 {len(unreadable)} 个文件读不了（权限这一类），这一格没法比对（路径里含私有值的那一段打成{FILE_NAME_MASK}，值不打印）：")   # gate-lint:summary
    for path, error in unreadable:
        print(f"      {shown_path(path)}  {errno.errorcode.get(error.errno, error.errno)}（{error.strerror}）")   # gate-lint:detail
    print("     → 怎么办：给自己补上读权限（chmod u+r）再跑；从第二台 tar 回来的权限位不对的，解包时加 --no-same-permissions；确实不该留的删掉。")
if findings or name_findings or too_big or link_findings or unreadable:
    sys.exit(1)
if lines_judged == 0 and links_judged == 0:
    print(f"  ! 这次改动没有新加的行（改动范围 {len(changed_paths)} 个路径，二进制 {len(binary_files)} 份），这一格本次无对象可判（这不是通过）")
    sys.exit(77)
short_total = sum(short_words.values())
short_keys = "、".join(sorted(short_words)) or "无"
print(f"  ✓ 查了这次改动里 {files_judged} 份文本文件新加的 {lines_judged} 行：对着 multi-host.env 的 {configuration_word_count} 个长词（{MINIMUM_LENGTH} 个字以上）与{hostname_note}，没有一处命中")
if links_judged:
    print(f"     符号链接 {links_judged} 个：比了路径与链接文字（不跟进去），没有一处命中")
print(f"     没判的：不足 {MINIMUM_LENGTH} 个字的词 {short_total} 个（键 {short_keys}）、二进制 {len(binary_files)} 份；"
      f"共享根下的路径 {sum(shared_words.values())} 个（键 {'、'.join(sorted(shared_words)) or '无'}，NFS_SHARED_ROOTS 某一段本身或在它下面，两台同一绝对路径，不是私有值）")
PY
  rm -f "$changed_file"
  exit "$python_rc"
}

cell_account_quota() {
  local base changed changed_file python_rc=0
  if [[ "$text_notations_break" == account-quota-unjudged ]]; then
    echo "  ! 弄坏开关 account-quota-unjudged：这一格不判，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  private_value_sources
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 被判的根不在 git 仓里，划不出「这一次改动新加的行」，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  base="$(gate_diff_base gate)"
  changed="$(text_changed_paths "$base")" || { echo "  ✗ 取不到这一轮碰了哪些文件（基准 ${base:-空}）"; echo "     → 怎么办：看 git 是不是可用、基准提交在不在；改动范围只经 scripts/gate/changed-paths.sh 取。"; exit 1; }
  changed_file="$(mktemp "${TMPDIR:-/tmp}/account-quota.XXXXXX")" || { echo "  ✗ 建不了放改动文件清单的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  printf '%s\n' "$changed" > "$changed_file"
  python3 - "$base" "$changed_file" "$text_notations_break" "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" "$text_diff_cache" <<'PY' || python_rc=$?
import errno, os, re, subprocess, sys, types

base, changed_path, broken, private_values_source, configuration_path, hostname_path, diff_cache_path = sys.argv[1:8]
private_values = types.ModuleType("lib_private_values")      # 路径与片段打码的共用函数（doc-text.sh 的 PRIVATE_VALUES_LIBRARY，与别的格同一份）
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
masking_targets = []
if configuration_path:
    try:
        masking_targets = private_values.load_private_targets(configuration_path, hostname_path, broken)[0]
    except OSError:
        masking_targets = []
HEAD_BYTES = 8192

# ── 词表（唯一一处）：逐条写明拦什么 ──
# 两层。
#   明说层（STRICT）：句子里写明了是哪个账号、哪一种限额（会话、周、Opus、用量、账号……），或是限额通知的原文，命中就红，不看同一行里还有什么词。
#   裸词层（BARE）：「撞限额」「额度恢复」这类没写是哪一种额度的说法；命中处前后 24 个字内有算力资源的词（RESOURCE_WORDS：显卡、显存、内存、线程、账上、磁盘、空间、quota……）的放过， account-quota:allow 词表与样本说明
#     那是显卡额度、内存额度、空间配额这类正常用法。只有「额度」「限额」两个词单独出现、没有上面这些动词与限定词的，不在词表里，不判。
# 键是红行里点名的判据名；改词表只改这一处。
SCOPE = r"(?:会话|周|每周|五小时|5\s?小时|Opus|Sonnet|Haiku|Claude|API|账号|帐号|订阅)"
ACCOUNT = r"(?<!欠)[账帐]号"       # 「欠账号」「欠账号 C820」是欠账编号，不是账号
STRICT = [
    ("账号切换·动词", "拦「切换账号」「换了账号」「换成别的账号」这类动作：切换、切到、换到、换成、轮换、更换、换（了）后六个字以内跟着账号。不拦「欠账号」",  # account-quota:allow 词表与出路说明
     r"(?:切换|切到|换到|换成|轮换|更换|换)(?:了)?[^，。；！？\n「」]{0,6}?" + ACCOUNT),
    ("账号切换·名词", "拦「账号切换」「账号额度」「另一个账号」「备用账号」「别的账号」这类名词写法",  # account-quota:allow 词表与出路说明
     ACCOUNT + r"(?:切换|轮换|额度|限额|用量|被限)|(?:另一个|别的|备用|第二个|不同的?)" + ACCOUNT),
    ("账号切换·英文", "拦 switched the account、account switch",  # account-quota:allow 词表与出路说明
     r"(?i:switch(?:ed|ing)?\s+(?:the\s+)?account|account\s+switch)"),
    ("限额·写明哪种", "拦「会话限额」「周限额」「每周额度」「Opus 周额度」「账号额度」「订阅限额」：会话、周、每周、五小时、Opus、Sonnet、Haiku、Claude、API、账号、订阅后面直接接限额或额度（中间可隔空白与「用量」）",  # account-quota:allow 词表与出路说明
     SCOPE + r"\s*(?:用量)?\s*(?:限额|额度)"),
    ("限额·窗口", "拦「限额窗口」「额度窗口」（含派发闸的放行行「限额窗口已判」被抄进记录）",  # account-quota:allow 词表与出路说明
     r"(?:限额|额度)窗口"),
    ("限额·会话周上限", "拦「会话上限」「周上限」",  # account-quota:allow 词表与出路说明
     r"(?:会话|周)上限"),
    ("限额·撞会话周限流", "拦「撞会话限流」「撞了周限流」：撞或碰、会话或周、限流",  # account-quota:allow 词表与出路说明
     r"(?:撞|碰)(?:到|上|了|过)*\s*" + SCOPE + r"\s*限流"),
    ("限额·通知原文", "拦限额通知的原文：You've hit your … limit、usage limit、session limit、weekly limit",  # account-quota:allow 词表与出路说明
     r"(?i:you(?:'|’)ve hit your|usage limit|session limit|weekly limit)"),  # account-quota:allow 词表与出路说明
    ("限额·通知重置时刻", "拦限额通知里 resets 后接月份缩写、日、十二小时制钟点和括号里的时区缩写（原样或解析它的正则：钟点写成 am 与 pm 的二选一、或 [ap]m，后面跟着括号）",  # account-quota:allow 词表本身
     r"(?i:resets\s+(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\.?\s+\d{1,2}(?:st|nd|rd|th)?,?\s+(?:at\s+)?\d{1,2}(?::\d{2})?\s?[ap]\.?m\.?\s*\([A-Za-z][A-Za-z0-9_/+\-]{1,}\)"
     r"|resets\b[^\n]{0,80}?(?:am\|pm|\[ap\]m)[^\n]{0,40}?\\?\()"),
]
if broken == "account-quota-notice-form-skipped":            # 弄坏：限额通知的重置时刻那一形态不判
    STRICT = [entry for entry in STRICT if entry[0] != "限额·通知重置时刻"]
BARE = [
    ("限额·撞", "拦「撞限额」「撞了额度」「碰到限流」「限额撞顶」「额度见底」：撞或碰（后面可接到、上、了、过）加用量、使用（可无）加限额、额度、限流；限额或额度后面接撞顶、到顶、见底、耗尽、用尽、耗光",  # account-quota:allow 词表与出路说明
     r"(?:撞|碰)(?:到|上|了|过)*(?:用量|使用)?(?:限额|额度|限流)|(?:限额|额度)(?:撞顶|到顶|见底|耗尽|用尽|耗光)"),
    ("限额·重置恢复", "拦「额度重置」「限额恢复」「额度刷新」「恢复了额度」「重置限额」：限额或额度后面接重置、刷新、恢复、解除、复位，或前面接重置、刷新、恢复（可带了）",  # account-quota:allow 词表与出路说明
     r"(?:限额|额度)(?:重置|刷新|恢复|解除|复位)|(?:重置|刷新|恢复)(?:了)?(?:限额|额度)"),
    ("限额·打断", "拦「被额度打断」「限额中断」「限额续做」「限额暂停」：被（用量）额度或限额后面接打断、卡住、停，或额度、限额后面接打断、暂停、中断、续做",  # account-quota:allow 词表与出路说明
     r"被(?:用量)?(?:额度|限额)(?:打断|卡住|停)|(?:额度|限额)(?:打断|暂停|中断|续做)"),
    ("限额·用完", "拦「额度用完」「限额用光」「用完了额度」「耗尽限额」",  # account-quota:allow 词表与出路说明
     r"(?:额度|限额)(?:用完|用光)|(?:用完|用光|耗尽)了?(?:限额|额度)"),
    ("用量限额·额度", "拦「用量限额」「用量额度」「使用限额」「使用额度」「用量上限」",  # account-quota:allow 词表与出路说明
     r"(?:用量|使用)(?:限额|额度)|用量上限"),  # account-quota:allow 词表与出路说明
]
# 命中处前后 RESOURCE_WINDOW 个字内有这些词，裸词层的命中放过（显卡额度、内存额度、空间配额、账上多少 G 这类算力与空间资源的正常用法）；
# 只看命中处附近、不看整行：一行里别处提到线程上限、草稿目录多少 G，不能把同一行里的模型限额放过去（对着仓里量过，整行放过的三行里有两行其实是模型限额）
RESOURCE_WINDOW = 24
RESOURCE_WORDS = re.compile(r"显卡|GPU|显存|VRAM|内存|memory|CPU|线程|账上|磁盘|空间|inode|文件系统|存储|quota|\d\s?(?:G|GiB|GB|MiB|MB)\b")
# 放行标记：行里写 account-quota:allow <理由>（理由至少 4 个字），整行放过；没写理由的不放过，另报。文件头「account-quota」一段写了什么时候用。
ALLOW_MARKER = re.compile(r"account-quota:allow(?P<reason>.*)")
COMPILED_STRICT = [(key, what, re.compile(pattern)) for key, what, pattern in STRICT]
COMPILED_BARE = [(key, what, re.compile(pattern)) for key, what, pattern in BARE]
if broken == "account-quota-strict-only":                    # 弄坏：裸词层不判
    COMPILED_BARE = []
if broken == "account-quota-resource-unexempt":              # 弄坏：裸词层不看资源词，显卡额度、内存额度这类也红
    RESOURCE_WORDS = re.compile(r"(?!)")


CODE_DIRECTORIES = (".claude/hooks/", "scripts/", ".claude/gate.d/", ".claude/rules/", ".claude/agents/", "task-server/", "prune-server/")
CODE_FILES = (".claude/main-agent.md", ".claude/agent-common.md")
SAMPLES_DIRECTORY = ".claude/gate.d/fixtures/"          # 门禁样本要造这些话才能证红，不判


def in_scope(path):
    """射程：records/ 与 .claude/kb/ 下的 .md，research/prompts/ 顶层的 *-main-verification.md（三方判决正文），
    加上代码与治理文件：CODE_DIRECTORIES 下的文本文件（不限后缀；样本目录除外）与 CODE_FILES 两份。
    prompts 里别的文件（_*-appendix.md、_*-background.md、_*-body.md、*-snapshot/ 下的副本）是从仓里抄来给云端腿读的，抄进来的旧话不算这一次写的，不判。"""
    if broken == "account-quota-records-only":                # 弄坏：只判 records/
        return path.startswith("records/") and path.endswith(".md")
    if broken != "account-quota-code-unscoped" and not path.startswith(SAMPLES_DIRECTORY) and (path in CODE_FILES or path.startswith(CODE_DIRECTORIES)):
        return True                                           # 弄坏 account-quota-code-unscoped：代码与治理文件不在射程里
    if not path.endswith(".md"):
        return False
    if path.startswith("records/") or path.startswith(".claude/kb/"):
        return True
    return path.startswith("research/prompts/") and "/" not in path[len("research/prompts/"):] and path.endswith("-main-verification.md")


def shown_path(path):
    return private_values.shown_path(path, masking_targets, broken)


def git(*arguments):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True)


def fail(what, howto):
    print(f"  ✗ {what}")
    print(f"     → 怎么办：{howto}")
    sys.exit(1)


untracked_listing = git("ls-files", "-z", "--others", "--exclude-standard")
if untracked_listing.returncode != 0:
    fail("列不出未跟踪的文件", "看 git 是不是可用；改动范围只经 scripts/gate/changed-paths.sh 取。")
untracked = {name for name in untracked_listing.stdout.decode("utf-8", "replace").split("\0") if name}
changed_paths = [name for name in open(changed_path, encoding="utf-8").read().split("\n") if name]


diff_segments = private_values.load_diff_segments(diff_cache_path, base, broken)   # 阶段预取的整份 diff 按文件切开；没预取到的文件 added_lines 退回逐份 git diff


def judge(line):
    """一行的判法：返回 (放行状况, 命中)。放行状况 allowed（带理由的标记）/ reasonless（标记没写理由）/ None；命中 (判据名, 命中位置, 命中文字) 或 None；
    资源词放过的裸词命中记成 exempt。"""
    if broken == "account-quota-words-skipped":                # 弄坏：整格不判（词表与放行标记都不看）
        return None, None
    marker = ALLOW_MARKER.search(line) if broken != "account-quota-marker-ignored" else None
    marker_state = None
    if marker:
        reason = re.sub(r"-->|^[\s:：，,]+|\s+$", "", marker.group("reason"))
        marker_state = "allowed" if (len(reason) >= 4 or broken == "account-quota-reasonless-allowed") else "reasonless"
    for key, _what, pattern in COMPILED_STRICT:
        found = pattern.search(line)
        if found:
            return marker_state, ("hit", key, found.start(), found.group(0))
    for key, _what, pattern in COMPILED_BARE:
        found = pattern.search(line)
        if found:
            if RESOURCE_WORDS.search(line[max(0, found.start() - RESOURCE_WINDOW): found.end() + RESOURCE_WINDOW]):
                return marker_state, ("exempt", key, found.start(), found.group(0))
            return marker_state, ("hit", key, found.start(), found.group(0))
    return marker_state, None


findings, reasonless, unreadable = [], [], []      # (路径, 行号, 判据名, 片段) / (路径, 行号, 片段) / (路径, 错误)
files_judged, lines_judged, allowed_lines, exempt_lines, binary_files = 0, 0, 0, 0, []
for path in changed_paths:
    if not in_scope(path) or os.path.islink(path) or not os.path.isfile(path):
        continue
    try:
        with open(path, "rb") as handle:
            head = handle.read(HEAD_BYTES)
        if b"\0" in head:
            binary_files.append(path)
            continue
        files_judged += 1
        whole_file = path in untracked or broken == "account-quota-whole-tree"
        for number, text in private_values.added_lines(path, whole_file, base, git, fail, shown_path, diff_segments):
            lines_judged += 1
            marker_state, hit = judge(text)
            if marker_state == "reasonless":
                reasonless.append((path, number, text.strip()[:60]))
                continue
            if hit is None:
                continue
            kind, key, position, matched = hit
            if kind == "exempt":
                exempt_lines += 1
                continue
            if marker_state == "allowed":
                allowed_lines += 1
                continue
            start, end = max(0, position - 15), position + len(matched) + 15
            start, end = private_values.snippet_bounds(text, start, end, masking_targets)
            findings.append((path, number, key, text[start:end].strip()))
    except OSError as error:
        unreadable.append((path, error))

if findings:
    print(f"  ✗ 这次改动新加的 {len(findings)} 行写了账号切换或用量限额，在 {len({path for path, _, _, _ in findings})} 份文件里（只判这次新加的行；旧行不报）：")  # account-quota:allow 词表与出路说明   # gate-lint:summary
    for path, number, key, snippet in findings[:200]:
        print(f"      {shown_path(path)}:{number}  {key}  …{private_values.shown_text(snippet, masking_targets)}…")   # gate-lint:detail
    if len(findings) > 200:
        print(f"      （另有 {len(findings) - 200} 行没列出）")   # gate-lint:detail
    print("     → 怎么办：删掉这句。子 agent 因此停下的，只写「中途停下没交回」，不写是撞了哪种额度、换没换账号、什么时候重置；")  # account-quota:allow 词表与出路说明
    print("               用户原话里带这些词、要原样留的，那一行末尾写「account-quota:allow <理由>」（理由至少 4 个字）；显卡额度、内存上限、空间配额这类不是这一条，命中处前后 24 个字内有显卡、内存、线程这类资源词的裸词命中本来就放过。")
if reasonless:
    print(f"  ✗ 这次改动新加的 {len(reasonless)} 行写了 account-quota:allow 却没写理由（理由至少 4 个字，写不出理由的就是该删的）：")   # gate-lint:summary
    for path, number, snippet in reasonless:
        print(f"      {shown_path(path)}:{number}  …{private_values.shown_text(snippet, masking_targets)}…")   # gate-lint:detail
    print("     → 怎么办：在 account-quota:allow 后面写为什么这一行要留（例如「用户原话」）；写不出来就删掉这一行里的限额写法。")
if unreadable:
    print(f"  ✗ 这次改动里 {len(unreadable)} 个文件读不了（权限这一类），这一格没法判：")   # gate-lint:summary
    for path, error in unreadable:
        print(f"      {shown_path(path)}  {errno.errorcode.get(error.errno, error.errno)}（{error.strerror}）")   # gate-lint:detail
    print("     → 怎么办：给自己补上读权限（chmod u+r）再跑；确实不该留的删掉。")
if findings or reasonless or unreadable:
    sys.exit(1)
if lines_judged == 0:
    print(f"  ! 这次改动没有新加到射程内（records/、.claude/kb/、research/prompts/*-main-verification.md，与钩子、脚本、门禁、规则、定义、task-server、prune-server）的行（改动范围 {len(changed_paths)} 个路径，二进制 {len(binary_files)} 份），这一格本次无对象可判（这不是通过）")
    sys.exit(77)
print(f"  ✓ 查了这次改动里 {files_judged} 份文本文件新加的 {lines_judged} 行（记录、kb 页、判决正文，与代码和治理文件）：没有账号切换、用量限额的写法")  # account-quota:allow 词表与出路说明
print(f"     放过的：带理由的放行标记 {allowed_lines} 行、命中处附近有显卡内存这类资源词的裸词命中 {exempt_lines} 行；没判的：二进制 {len(binary_files)} 份")
PY
  rm -f "$changed_file"
  exit "$python_rc"
}

cell_task_and_session_numbers() {
  local base changed changed_file python_rc=0
  if [[ "$text_notations_break" == task-and-session-numbers-unjudged ]]; then
    echo "  ! 弄坏开关 task-and-session-numbers-unjudged：这一格不判，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  private_value_sources
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 被判的根不在 git 仓里，划不出「这一次改动相对基准多出来的行」，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  base="$(gate_diff_base gate)"
  changed="$(text_changed_paths "$base")" || { echo "  ✗ 取不到这一轮碰了哪些文件（基准 ${base:-空}）"; echo "     → 怎么办：看 git 是不是可用、基准提交在不在；改动范围只经 scripts/gate/changed-paths.sh 取。"; exit 1; }
  changed_file="$(mktemp "${TMPDIR:-/tmp}/task-and-session-numbers.XXXXXX")" || { echo "  ✗ 建不了放改动文件清单的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  printf '%s\n' "$changed" > "$changed_file"
  python3 - "$LIBRARY" "$base" "$changed_file" "$text_notations_break" "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" <<'PY' || python_rc=$?
import errno, importlib.util, os, subprocess, sys, types

library_path, base, changed_path, broken, private_values_source, configuration_path, hostname_path = sys.argv[1:8]
sys.dont_write_bytecode = True
private_values = types.ModuleType("lib_private_values")      # 路径与片段打码的共用函数（doc-text.sh 的 PRIVATE_VALUES_LIBRARY，与别的格同一份）
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
masking_targets = []
if configuration_path:
    try:
        masking_targets = private_values.load_private_targets(configuration_path, hostname_path, broken)[0]
    except OSError:
        masking_targets = []
FORM = "task-and-session-numbers"
HEAD_BYTES = 8192
FROZEN_EVIDENCE_PREFIX = "research/prompts/"      # 冻结证据：三方材料与判决留着当时的原文，不判；弄坏开关 task-and-session-numbers-prompts-judged 下照判


def fail(what, howto):
    print(f"  ✗ {what}")
    print(f"     → 怎么办：{howto}")
    sys.exit(1)


try:
    specification = importlib.util.spec_from_file_location("lib_forbidden_notations", library_path)
    notations = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(notations)
except Exception as error:   # 文件不在、语法错都算读不到
    fail(f"读不到判据库 {library_path}（{type(error).__name__}：{error}），这一格没法判",
         "那一份是写闸、检出钩子 ⑰、scripts/kb/ 写入工具与这一道共用的唯一定义，先 git status 看它是被删了还是被改坏了，恢复之后重跑")


def shown_path(path):
    return private_values.shown_path(path, masking_targets, broken)


def git(*arguments):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True)


def renamed_sources():
    """这一次改动里搬家、改名的文件：{此刻的路径: 基准里的旧路径}，取 git diff -M 认出的改名对（基准对工作区，git mv 过的、改名之后又改过几行的都认）；
    没有基准、git 报错交空表。弄坏开关 task-and-session-numbers-renames-ignored 一律交空表（基准只按此刻的路径取）。"""
    if broken == "task-and-session-numbers-renames-ignored" or not base:
        return {}
    shown = git("diff", "-M", "--name-status", "-z", base, "--")
    if shown.returncode != 0:
        return {}
    fields, pairs, index = shown.stdout.decode("utf-8", "replace").split("\0"), {}, 0
    while index < len(fields) - 1:
        if fields[index].startswith("R") and index + 2 < len(fields):
            pairs[fields[index + 2]] = fields[index + 1]
            index += 3
        else:
            index += 2
    return pairs


renamed = renamed_sources()


def base_text(path):
    """这份文件在基准里的样子：搬家、改名过来的取基准里旧路径那一份（renamed），别的按此刻的路径取；基准里没有（新文件、基准是空树）交空串。
    弄坏开关 task-and-session-numbers-base-ignored 一律交空串。"""
    if broken == "task-and-session-numbers-base-ignored" or not base:
        return ""
    shown = git("cat-file", "-p", f"{base}:{renamed.get(path, path)}")
    return shown.stdout.decode("utf-8", "replace") if shown.returncode == 0 else ""


changed_paths = [name for name in open(changed_path, encoding="utf-8").read().split("\n") if name]
findings, unreadable, binary_files = [], [], []      # (路径, 行号, 判据行名字, 命中的串) / (路径, 错误) / 路径
files_judged, kept_lines = 0, 0
frozen_files = []
for path in changed_paths:
    if notations.outside_judged_suffixes(FORM, path) or notations.is_excluded(FORM, path) or os.path.islink(path) or not os.path.isfile(path):
        continue
    if path.startswith(FROZEN_EVIDENCE_PREFIX) and broken != "task-and-session-numbers-prompts-judged":
        frozen_files.append(path)
        continue
    try:
        with open(path, "rb") as handle:
            data = handle.read()
    except OSError as error:
        unreadable.append((path, error))
        continue
    if b"\0" in data[:HEAD_BYTES]:
        binary_files.append(path)
        continue
    files_judged += 1
    after = data.decode("utf-8", "replace")
    found = notations.new_violation_hits(FORM, base_text(path), after, path)
    findings += [(path, number, name, matched) for number, _, name, matched in found]
    kept_lines += len({number for number, _, _, _ in notations.every_hit(FORM, after, path)}) - len({number for number, _, _, _ in found})

if findings:
    print(f"  ✗ 这次改动相对基准多出 {len({(path, number) for path, number, _, _ in findings})} 行写了任务号、会话短号或批号，"
          f"在 {len({path for path, _, _, _ in findings})} 份文件里（新加的行与因围栏、表头改动而成了违例的行；基准里已有的违例行不报）：")   # gate-lint:summary
    for path, number, name, matched in findings[:200]:
        print(f"      {shown_path(path)}:{number}  {name}  「{private_values.shown_text(matched, masking_targets)}」")   # gate-lint:detail
    if len(findings) > 200:
        print(f"      （另有 {len(findings) - 200} 处没列出）")   # gate-lint:detail
    remedy = notations.FORMS_BY_KEY[FORM]["remedy"]
    print("     → 怎么办：" + remedy[0])
    for item in remedy[1:]:
        print("               " + item)
    print("               这一格补的是绕过写闸、检出钩子 ⑰ 与 scripts/kb/ 写入工具写进仓的那条路（cp、mv、命令的输出）：照上面改那几行，旧记录里基准已有的号不回改")
if unreadable:
    print(f"  ✗ 这次改动里 {len(unreadable)} 个文件读不了（权限这一类），这一格没法判：")   # gate-lint:summary
    for path, error in unreadable:
        print(f"      {shown_path(path)}  {errno.errorcode.get(error.errno, error.errno)}（{error.strerror}）")   # gate-lint:detail
    print("     → 怎么办：给自己补上读权限（chmod u+r）再跑；确实不该留的删掉。")
if findings or unreadable:
    sys.exit(1)
if files_judged == 0:
    print(f"  ! 这次改动没有碰到要判的 .md、.tsv、.txt（改动范围 {len(changed_paths)} 个路径，二进制 {len(binary_files)} 份，research/prompts/ 下的冻结证据 {len(frozen_files)} 份不判），这一格本次无对象可判（这不是通过）")
    sys.exit(77)
print(f"  ✓ 查了这次改动里 {files_judged} 份 .md、.tsv、.txt：相对基准没有多出写任务号、会话短号或批号的行")
print(f"     放过的：基准里已有的违例行 {kept_lines} 行不报；没判的：二进制 {len(binary_files)} 份、research/prompts/ 下的冻结证据 {len(frozen_files)} 份；"
      f"搬家、改名的 {len(renamed)} 份按基准里的旧路径取基准")
PY
  rm -f "$changed_file"
  exit "$python_rc"
}
