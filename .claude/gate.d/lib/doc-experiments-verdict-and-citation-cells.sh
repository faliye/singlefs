#!/usr/bin/env bash
# 门禁 doc-experiments 的格函数库：verdict-false-named、repro-command、experiment-orphans、quoted-result-lines、archive-past-rounds、multipath-registry、cited-paths-exist。
# 由 .claude/gate.d/doc-experiments.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ── 格 verdict-false-named：实验产物判决行里的 false 字段有没有被实验页点名
#
# 判据：scripts/experiment-apparatus/replay.sh 登记的入库产物（在 research/results/ 下的）里，以 `E7RESULT name=verdict ` 开头的判决行，
# 每个 `字段=false` 都要在对应实验号的实验页（.claude/kb/experiments/<号>-*.md）里被点名，否则判红。不开豁免。
# 射程只认这个字面形式：源码里有 name=verdict_candidate / _base / _unit 这几种变体，没落进入库产物，出现了这一格看不见。
#
# 点名：同一物理行里字段名紧跟它自己的 =false（中间可夹反引号与空白；同一行别的字段的 false 不算），且这一行在最新那条历史条目
# （「## 历史版本」之后第一个 ### 那一节）或正文（「## 历史版本」之前）里。只认字段名出现在正文不够：正文可能是旧一轮留下的。
# 正文里那一行自己标着「第 N 次跑」、而标的次数里没有最新历史条目标题里的那一次，是上一次跑留下的，不算点名。
# 登记的产物一份都不在 research/results/ 下、或一个 false 字段都没有，退 77。
#
# 样本：verdict-false-named-red 里三个没点名的 false（一个在正文里只写字段名、一个点名的那一行标着上一次跑、一个根本没提）；
# verdict-false-named-green 里一个在最新历史条目点名、一个实验页文件名补了零；verdict-false-named-zero 的登记表一行都读不到。
# 实验页取哪一份：lib-item-ref-status.py 的 numbered_files，产物不带语言，取中文原文。没有「## 历史版本」的页照判正文（最新历史条目当空的）。
# 样本：verdict-false-named-translation-green（译文排在前面、中文原文没有历史节而正文点了名，判绿）、verdict-false-named-no-history-red
# （中文原文没有历史节也没点名、英语译文点了名，判红）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
cell_verdict_false_named() {
  REPLAY=scripts/experiment-apparatus/replay.sh
  [[ -f "$REPLAY" ]] || { echo "  ! 找不到 $REPLAY，这一格无对象可判"; exit 77; }

  python3 - "$REPLAY" "$LIB_ITEM_REF_STATUS" "$LIB_LAG_BASELINE" <<'PY'
import glob
import importlib.util
import os
import re
import sys

replay_path = sys.argv[1]
item_ref_status_spec = importlib.util.spec_from_file_location('item_ref_status', sys.argv[2])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)
lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', sys.argv[3])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)

# 复跑行（实验号、产物）：lib-lag-baseline.py 的 replay_rows，与 results-cited、quoted-result-lines、replay-comparison-units 三格同一份读法
rows = [(row.experiment, row.product) for row in (lag_baseline.replay_rows(replay_path) or [])]

if not rows:
    print(f"  ✗ 从 {replay_path} 一行都没读到——登记表的格式变了？")
    print("     → 怎么办：这一格靠 '编号|二进制|参数|产物|exact/timing' 这个形状取行（取法在 .claude/gate.d/lib-lag-baseline.py 的 replay_rows，")
    print("               与门禁 research-build-and-replay.sh 的 experiment-replay 格同一个形状），改表格式要同时改这两处。")
    sys.exit(1)

RESULTS_DIR = 'research/results'
EXP_DIR = '.claude/kb/experiments'
VERDICT_PREFIX = 'E7RESULT name=verdict '
skip_named_check = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK') == 'named-check-skipped'

missing_files = []      # 登记了却在 research/results/ 下找不到文件（多半是已归档，见 research-build-and-replay.sh 复跑里「产物已归档」那一档）
checked_files = 0
verdict_lines = 0
false_fields = []       # (exp, product, lineno, field)

for exp, product in rows:
    path = os.path.join(RESULTS_DIR, product)
    if not os.path.isfile(path):
        missing_files.append((exp, product))
        continue
    checked_files += 1
    # 压成 .gz 入库的产物（超过单文件推送上限的）解压后读，判据不变；弄坏开关 gz-read-raw：不解压，按原字节读
    with (__import__('gzip').open(path, 'rt', encoding='utf-8', errors='replace')
          if path.endswith('.gz') and os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK') != 'gz-read-raw'
          else open(path, encoding='utf-8', errors='replace')) as handle:
        for lineno, raw in enumerate(handle, 1):
            stripped = raw.strip()
            if not stripped.startswith(VERDICT_PREFIX):
                continue
            verdict_lines += 1
            rest = stripped[len(VERDICT_PREFIX):]
            for token in rest.split():
                if '=' not in token:
                    continue
                field, value = token.split('=', 1)
                if value == 'false':
                    false_fields.append((exp, product, lineno, field))


def experiment_page(exp, product):
    # 实验页文件名有的补零（08-、09-），有的不补：两种都认（numbered_files）。产物不带语言，取中文原文；
    # 取排序后的第一个会取到 <号>-<名>.en.md，那份没有「## 历史版本」，split_page 当场崩
    matches = item_ref_status.numbered_files(EXP_DIR, int(exp[1:]), os.path.join(RESULTS_DIR, product))
    return matches[0] if matches else None


def split_page(text):
    """(正文, 最新那条历史条目, 它的标题行)：没有「## 历史版本」、或历史节里一个 ### 都没有的，后两样是空串。"""
    marker = '\n## 历史版本'
    idx = text.find(marker)
    body = text if idx < 0 else text[:idx]
    if idx < 0:
        # 这一支交回三样，调用方按三样拆包：只交两样，没有历史节的实验页（中文原文少了这一节、或取到了译文）会让整格崩在 ValueError 上
        return body, '', ''
    tail = text[idx + len(marker):]
    head_match = re.search(r'^### .*$', tail, re.M)
    if not head_match:
        return body, '', ''
    start = head_match.end()
    next_match = re.search(r'^#{2,3} ', tail[start:], re.M)
    end = start + next_match.start() if next_match else len(tail)
    return body, tail[start:end], head_match.group(0)


RUN_TAG = re.compile(r'第([一二三四五六七八九十百零〇两0-9]+)次跑')


def is_named(field, body, latest, latest_heading):
    """返回 (点没点名, 正文里那一行若是上一次跑留下的就交出它)。字段名要紧跟着它自己的 =false，同一行别的字段的 false 不算。"""
    if skip_named_check:
        return True, None
    pattern = re.compile(r'(?<![A-Za-z0-9_])' + re.escape(field) + r'`?\s*[=:：]\s*`?false(?![A-Za-z0-9_])')
    if any(pattern.search(line) for line in latest.split('\n')):
        return True, None
    latest_tags = set(RUN_TAG.findall(latest_heading))
    stale = None
    for line in body.split('\n'):
        if not pattern.search(line):
            continue
        tags = set(RUN_TAG.findall(line))
        if tags and latest_tags and not (tags & latest_tags):
            stale = stale or line.strip()
            continue
        return True, None
    return False, stale


unnamed = []   # (exp, product, lineno, field, reason)
named_count = 0
page_cache = {}
for exp, product, lineno, field in false_fields:
    if exp not in page_cache:
        page_cache[exp] = experiment_page(exp, product)
    page = page_cache[exp]
    if page is None:
        unnamed.append((exp, product, lineno, field,
                         f'找不到实验页 {EXP_DIR}/{exp[1:]}-*.md'))
        continue
    with open(page, encoding='utf-8') as handle:
        text = handle.read()
    body, latest, latest_heading = split_page(text)
    named, stale = is_named(field, body, latest, latest_heading)
    if named:
        named_count += 1
    elif stale:
        unnamed.append((exp, product, lineno, field,
                         f'{page} 正文里点到它的那一行标的是别的某次跑（「{stale[:40]}」），'
                         f'最新历史条目是「{latest_heading.lstrip("# ").strip()[:30]}」：那是上一次跑留下的，不算点名这一次的 false'))
    else:
        unnamed.append((exp, product, lineno, field,
                         f'{page} 里既不在最新历史条目、也不在正文里被点名'))

print(f'  · 登记了 {len(rows)} 行产物；{checked_files} 份文件在 {RESULTS_DIR} 下找到、'
      f'{len(missing_files)} 份登记了却找不到（多半已归档）')
if missing_files:
    print(f'     没查到文件的登记行（{len(missing_files)} 个）：')
    for exp, product in missing_files:
        print(f'       {exp}  {RESULTS_DIR}/{product}')  # gate-lint:detail
print(f'  · 查到的文件里共 {verdict_lines} 行判决（name=verdict）；{len(false_fields)} 个字段是 false，'
      f'其中 {named_count} 个已点名、{len(unnamed)} 个没点名')

if unnamed:
    print('  ✗ 这些判决字段是 false，对应实验页没有点名：')
    for exp, product, lineno, field, reason in unnamed:
        print(f'     {exp}  {RESULTS_DIR}/{product}:{lineno}  {field}=false  ({reason})')   # gate-lint:detail
    print('     → 怎么办：在实验页那一次跑的历史条目里写明这个字段为什么是 false——是判据如实判出')
    print('               的结论，还是装置或期望值的问题；后者照问题单重跑，把结论一起写回历史条目。')
    sys.exit(1)

if checked_files == 0:
    print(f'  ⊘ 本次无对象可判：登记的 {len(rows)} 行产物一份都不在 {RESULTS_DIR} 下（上面逐个列了）')
    sys.exit(77)
if not false_fields:
    print(f'  ⊘ 本次无对象可判：查到的 {checked_files} 份产物里 {verdict_lines} 行判决，没有一个字段是 false')
    sys.exit(77)
print(f'  ✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 {named_count} 个）')
PY
}

# ── 格 repro-command：跑过的实验有没有留下复跑命令
#
# 判据：实验页点名了 research/results/ 里的产物，就要写出怎么把它跑出来，三种写法认其一：
#   一，一条 `cargo run --release --bin X`，或者点名复跑入口 scripts/experiment-apparatus/replay.sh / scripts/devices/vm-bench.sh；
#   二，起 scripts/experiment-apparatus/ 下 python3 或 bash 脚本的复跑行（纯 Python 的装置没有 cargo run）。认三种行：
#      `… -- python3|bash [-选项…] [<占位>/]scripts/experiment-apparatus/…/X.py|X.sh`（脚本写全路径，含 E200（修订6近似树重跑）页那行 `<仓根>/scripts/…`）、
#      `… --cwd scripts/experiment-apparatus/<装置目录> -- python3|bash [-选项…] X.py|X.sh`（脚本相对 --cwd）、
#      `… --cwd scripts/experiment-apparatus/<装置目录> -- python3 [-选项…] -m <模块> …`（只点名装置目录，跑单测这类）。
#      页里这样的行点名的脚本与装置目录都要在仓里存在，至少一行点名的是存在的脚本文件（只有 -m 行不够）；有一个不存在判红，红行点名页与路径。
#      提到 mutate.sh 这类别的脚本、只写路径不写 python3 / bash 起它的，不算复跑行。
#   三，产物逐字节复跑不了的页登记进 .claude/gate.d/repro-command-unreproducible.tsv（一行一页：实验页文件名、制表符、原因），登记了的页不判；
#      登记表在页上抄的产物行仍归 quoted-result-lines 判，这里只放行「没写复跑命令」。登记行指到 kb/experiments/ 下不存在的页、没写原因判红。登记表不在当没有登记。
# 写明「原始输出未留存」的页不判、成功行列名。读不了的页判红（grep 出错与「没点产物」不是一回事）；一页 .md 都没有、或没有一页点产物，退 77。
# 成功行报点了产物的页数、未留存的页、靠装置复跑行过的页、登记表登了几页与其中靠登记放行的页。
# 样本：repro-command-red 里一页点了产物却只提到变异脚本；repro-command-green 里一页写了 cargo run、一页写明原始输出未留存。
# repro-command-apparatus-green：页一用 E200（修订6近似树重跑）页那几种行（--cwd 加 -m 单测、--cwd 加脚本名、带占位的全路径 bash 脚本、全路径 python3），页二用裸起 `python3 -B <全路径>`，脚本都在；
# repro-command-apparatus-red：页一的复跑行一条点名的脚本在、两条不在，页二点名的装置目录不在（红行点名页与不存在的路径，不报「没写复跑命令」）。
# repro-command-unreproducible-green：一页没写复跑命令但登记了、一页写了 cargo run；
# repro-command-unreproducible-page-missing-red：登记行指到不存在的实验页；repro-command-unreproducible-reason-empty-red：登记行没写原因。
# 弄坏开关：apparatus-lines-unrecognized、apparatus-script-existence-unchecked、unreproducible-registry-ignored、
# unreproducible-page-existence-unchecked、unreproducible-reason-unchecked（逐条说明和判错的样本在文件头开关表）。
cell_repro_command() {
  EXP_DIR=.claude/kb/experiments
  UNREPRODUCIBLE=.claude/gate.d/repro-command-unreproducible.tsv
  [[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，这一格跳过"; exit 77; }
  PRODUCT_RE='research/results/[A-Za-z0-9._-]+\.out(\.gz)?'
  # 弄坏开关 gz-name-not-product：.out.gz 不算产物名，只认以 .out 收尾的名字（点名 .gz 产物的页不再算点了产物）
  if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == gz-name-not-product ]]; then PRODUCT_RE='research/results/[A-Za-z0-9._-]+\.out([^.A-Za-z0-9_-]|$)'; fi
  REPRO_RE='cargo run --release --bin |scripts/(experiment-apparatus/replay|devices/vm-bench)\.sh'
  # 装置复跑行的三种写法（判据二）；检查顺序：全路径、--cwd 加脚本名、--cwd 加 -m
  SP='[[:space:]]'
  APPARATUS_DIRECTORY_RE='scripts/experiment-apparatus/[A-Za-z0-9._/-]+'
  LAUNCH_PATH_RE="(python3|bash)(${SP}+-[A-Za-z]+)*${SP}+(<[^>]*>/)?(${APPARATUS_DIRECTORY_RE}\\.(py|sh))(${SP}|\$)"
  LAUNCH_CWD_FILE_RE="--cwd${SP}+(${APPARATUS_DIRECTORY_RE})${SP}+--${SP}+(python3|bash)(${SP}+-[A-Za-z]+)*${SP}+([A-Za-z0-9._/-]+\\.(py|sh))(${SP}|\$)"
  LAUNCH_CWD_MODULE_RE="--cwd${SP}+(${APPARATUS_DIRECTORY_RE})${SP}+--${SP}+python3${SP}+(-[A-Za-z]+${SP}+)*-m${SP}"

  # 登记表：一行一页，页文件名在 $EXP_DIR 下要在、原因要写
  declare -A registered=()
  registry_rows=0; registry_errors=()
  if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" != unreproducible-registry-ignored && -e "$UNREPRODUCIBLE" ]]; then
    if [[ ! -r "$UNREPRODUCIBLE" ]]; then
      echo "  ✗ 登记表 $UNREPRODUCIBLE 读不了，登记了谁都没判"
      echo "  → 怎么办：修好权限再跑；读不了的登记表不能当成没有登记。"
      exit 1
    fi
    registry_line=0
    while IFS=$'\t' read -r registry_page registry_reason _ || [[ -n "$registry_page" ]]; do
      registry_line=$((registry_line + 1))
      [[ "$registry_page" =~ ^${SP}*$ || "$registry_page" == \#* ]] && continue
      registry_rows=$((registry_rows + 1))
      if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" != unreproducible-page-existence-unchecked && ! -e "$EXP_DIR/$registry_page" ]]; then
        registry_errors+=("第 ${registry_line} 行：${registry_page} 不是 $EXP_DIR 下的实验页")
      fi
      if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" != unreproducible-reason-unchecked && "${registry_reason:-}" =~ ^${SP}*$ ]]; then
        registry_errors+=("第 ${registry_line} 行：${registry_page} 没写原因")
      fi
      registered["$registry_page"]=1
    done < "$UNREPRODUCIBLE"
  fi
  if ((${#registry_errors[@]})); then
    echo "  ✗ 登记表 $UNREPRODUCIBLE 有 ${#registry_errors[@]} 处写得不对："
    printf '      %s\n' "${registry_errors[@]}"
    echo "  → 怎么办：一行一页，<实验页文件名><TAB><产物逐字节复跑不了的原因>（写法在登记表文件头）；页改了名或删了，改这一行或删这一行。"
    exit 1
  fi

  # 逐页的三种判据（点了产物、写明原始输出未留存、写了复跑入口）：普通文件、读得了、名字里没有换行的页，各种判据对这些页一次 grep -l 取齐、按页名查命中；
  # 别的页（读不了、不是普通文件）与批量 grep 没跑成（退 2 以上）时，这一趟所有页退回逐页 grep（退 2 的记成读不了的页）。
  # 弄坏开关 repro-command-batch-unattributed：命中不分页，写明未留存或写了复跑入口的页只要有一页，就当每一页都有。
  declare -A batch_covered=() batch_product_hit=() batch_exempt_hit=() batch_repro_hit=()
  batch_pages=()
  for f in "$EXP_DIR"/*.md; do
    [[ -f "$f" && -r "$f" && "$f" != *$'\n'* ]] && batch_pages+=("$f")
  done
  if ((${#batch_pages[@]})); then
    batch_ok=1
    product_hits="$(grep -lE "$PRODUCT_RE" -- "${batch_pages[@]}")"; (($? <= 1)) || batch_ok=0
    exempt_hits="$(grep -lF '原始输出未留存' -- "${batch_pages[@]}")"; (($? <= 1)) || batch_ok=0
    repro_hits="$(grep -lE "$REPRO_RE" -- "${batch_pages[@]}")"; (($? <= 1)) || batch_ok=0
    if ((batch_ok)); then
      for f in "${batch_pages[@]}"; do batch_covered["$f"]=1; done
      while IFS= read -r hit; do [[ -n "$hit" ]] && batch_product_hit["$hit"]=1; done <<<"$product_hits"
      while IFS= read -r hit; do [[ -n "$hit" ]] && batch_exempt_hit["$hit"]=1; done <<<"$exempt_hits"
      while IFS= read -r hit; do [[ -n "$hit" ]] && batch_repro_hit["$hit"]=1; done <<<"$repro_hits"
      if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == repro-command-batch-unattributed ]]; then
        if ((${#batch_exempt_hit[@]})); then for f in "${batch_pages[@]}"; do batch_exempt_hit["$f"]=1; done; fi
        if ((${#batch_repro_hit[@]})); then for f in "${batch_pages[@]}"; do batch_repro_hit["$f"]=1; done; fi
      fi
    fi
  fi

  bad=(); unreadable=(); exempt=(); registry_exempt=(); apparatus_ok=(); apparatus_missing=(); pointing=0
  for f in "$EXP_DIR"/*.md; do
    # 一页 .md 都没有时通配原样留下，grep 找不到它会退 2、被当成「读不了的页」：那是无对象可判，不是读不了
    [[ -e "$f" || -L "$f" ]] || continue
    page_name="${f##*/}"
    if [[ -n "${batch_covered[$f]:-}" ]]; then
      if [[ -n "${batch_product_hit[$f]:-}" ]]; then rc=0; else rc=1; fi
    else
      grep -qE "$PRODUCT_RE" "$f"; rc=$?
    fi
    if ((rc == 2)); then unreadable+=("$page_name"); continue; fi
    ((rc == 0)) || continue
    pointing=$((pointing + 1))
    if [[ -n "${batch_covered[$f]:-}" ]]; then
      [[ -n "${batch_exempt_hit[$f]:-}" ]] && page_exempt=1 || page_exempt=0
      [[ -n "${batch_repro_hit[$f]:-}" ]] && page_repro=1 || page_repro=0
    else
      grep -qF '原始输出未留存' "$f" && page_exempt=1 || page_exempt=0
      grep -qE "$REPRO_RE" "$f" && page_repro=1 || page_repro=0
    fi
    if ((page_exempt)); then exempt+=("$page_name"); continue; fi
    ((page_repro)) && continue
    if [[ -n "${registered[$page_name]:-}" ]]; then registry_exempt+=("$page_name"); continue; fi
    # 起 scripts/experiment-apparatus/ 下脚本的复跑行：点名的脚本文件、装置目录都要在
    found_files=0; missing_paths=()
    if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" != apparatus-lines-unrecognized ]]; then
      while IFS= read -r line; do
        target=""; target_kind=file
        if [[ "$line" =~ $LAUNCH_PATH_RE ]]; then target="${BASH_REMATCH[4]}"
        elif [[ "$line" =~ $LAUNCH_CWD_FILE_RE ]]; then target="${BASH_REMATCH[1]%/}/${BASH_REMATCH[4]}"
        elif [[ "$line" =~ $LAUNCH_CWD_MODULE_RE ]]; then target="${BASH_REMATCH[1]%/}"; target_kind=directory
        else continue; fi
        if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == apparatus-script-existence-unchecked ]]; then
          present=1
        elif [[ "$target_kind" == file && -f "$target" ]] || [[ "$target_kind" == directory && -d "$target" ]]; then
          present=1
        else
          present=0
        fi
        if ((present == 0)); then missing_paths+=("$target")
        elif [[ "$target_kind" == file ]]; then found_files=$((found_files + 1))
        fi
      done < <(grep -hE 'scripts/experiment-apparatus/' "$f")
    fi
    if ((${#missing_paths[@]})); then apparatus_missing+=("${page_name}：${missing_paths[*]}"); continue; fi
    if ((found_files > 0)); then apparatus_ok+=("$page_name"); continue; fi
    bad+=("$page_name")
  done

  if ((${#unreadable[@]})); then
    echo "  ✗ 这些实验页读不了，点没点产物、写没写复跑命令都没判："
    printf '      %s\n' "${unreadable[@]}"
    echo "  → 怎么办：按 grep 的报错修好（权限、编码），再跑；读不了的页不能当成没点产物放过去。"
    exit 1
  fi

  if ((${#apparatus_missing[@]} || ${#bad[@]})); then
    if ((${#apparatus_missing[@]})); then
      echo "  ✗ 这些实验页的复跑行点名了仓里不存在的脚本或装置目录，照着跑不起来："
      printf '      %s\n' "${apparatus_missing[@]}"
      echo "  → 怎么办：脚本或装置目录改了名、搬了或删了，把复跑行改成现存的路径（搬迁登记在 .claude/gate.d/path-moves.tsv）；"
      echo "    脚本确实没了、产物也复跑不了的，照下面登记表那条处理。"
    fi
    if ((${#bad[@]})); then
      echo "  ✗ 这些实验点了产物却没写复跑命令，读的人只能靠反推："
      printf '      %s\n' "${bad[@]}"
      echo "  → 怎么办：在正文的口径一节补一句，格式与别处一致——"
      echo "    代码 \`scripts/experiment-apparatus/eNN_xxx.rs\`（\`cargo run --release --bin eNN-xxx\`），"
      echo "    原始输出 \`research/results/eNN-xxx-YYYY-MM-DD.out\`。"
      echo "    纯 Python 或 bash 的装置，写起它的整条命令：\`task-server/task submit -- python3 scripts/experiment-apparatus/eNN_xxx/xxx.py …\`，"
      echo "    或带装置目录的 \`task-server/task submit --cwd scripts/experiment-apparatus/eNN_xxx -- python3 xxx.py …\`；点名的脚本要在仓里。"
      echo "    多次运行拼起来的产物，要把那个循环也写出来（种子、参数各扫了哪些值）。"
      echo "    要真设备 / 虚机因而没留产物的，正文写明「原始输出未留存」，这一格就不管它。"
      echo "    产物逐字节复跑不了的（配置行带输出目录、计数随回收变这类），在 $UNREPRODUCIBLE 加一行：<实验页文件名><TAB><原因>。"
      echo "    复跑入口只认 cargo run --release --bin、scripts/experiment-apparatus/replay.sh、scripts/devices/vm-bench.sh，以及起 scripts/experiment-apparatus/ 下 python3 或 bash 脚本的复跑行；提到 mutate.sh 这类别的脚本不算。"
    fi
    exit 1
  fi
  if ((pointing == 0)); then
    echo "  ⊘ 本次无对象可判：$EXP_DIR 下没有一页点到 research/results/ 里的产物"
    exit 77
  fi
  echo "  ✓ 点了产物的实验都写了复跑命令（点了产物的 ${pointing} 页里判了 $((pointing - ${#exempt[@]} - ${#registry_exempt[@]})) 页）"
  echo "    写明「原始输出未留存」不判的 ${#exempt[@]} 页：${exempt[*]:-（没有）}"
  echo "    靠起 scripts/experiment-apparatus/ 下脚本的复跑行放行的 ${#apparatus_ok[@]} 页：${apparatus_ok[*]:-（没有）}"
  echo "    登记表 $UNREPRODUCIBLE 登了 ${registry_rows} 页，其中靠登记放行的 ${#registry_exempt[@]} 页：${registry_exempt[*]:-（没有）}"
}

# ── 格 experiment-orphans：目录登记列的各处出现过的实验号在 kb 里有没有正文
#
# 判据：目录登记 .claude/gate.d/experiment-number-directories.tsv 列的每一处（research/、scripts/、入库装置的两处 crates/singlefs-verification/src/bin/ 与
# 产品 GPU 测试包的 crates/singlefs-background-gpu-tests/src/bin/、kb 实验页与索引行；名字形态写在登记文件头，以登记为准）出现过的实验号，kb/experiments/ 里都要有对应编号的正文：
# 跑过的东西没入库，会以「我们量过」的形式活在对话里，谁也复核不了。results-cited 格按名字排除的（*local* 这类）也在这里判到。
# 「有正文」只认 kb/experiments/ 这一层名字是「<号>-<简称>.md」的普通文件（号可带前导 0，译文 .en.md / .ja.md 也算），且有一个非空白字符：
# 同号的目录、别的后缀（.bin 这类）、符号链接、空的或只有空白的 .md 都不算。正文写得全不全不归这一格（各节的判法在别的格）。
# 读法只有一份（scripts/kb/lib_experiment_number_directories.py），scripts/kb/claim-experiment.sh 的查号读同一份登记：实验号住进新地方，登记里加一行，两处跟着认。
# 射程逐处报：登记了而不在的列进成功行；登记不在、写坏，或哪个目录读不了，判红——射程静默缩小与判过了在输出里长得一样。
# 样本：experiment-orphans-red 里 scripts/experiment-apparatus/ 与 crates/singlefs-verification/src/bin/ 下各一个 kb 里没有正文的实验号；
# experiment-orphans-green 里的实验号都有正文；experiment-orphans-registry-red 的登记末尾加了一行 notes，notes/ 下的号与只登了索引行的号都判成孤儿。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=orphans-registry-ignored：不读登记、用写死的四个目录，experiment-orphans-registry-red 判错。
# experiment-orphans-page-directory-red / -page-binary-red / -empty-page-red：E42 有产物，kb/experiments/ 里同号的只有目录 42-placeholder/、只有 42.bin、只有只含换行的 42-空白.md，
# 三份都判 E42 没有正文。弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=orphans-page-shape-ignored：「有正文」只看条目名的数字前缀，这三份判错。
# 三方腿的模型目录（research/prompts/ 直接底下名字以 -model 结尾的目录）读法里不往里走：腿在里面造的样本世界不是实验（登记文件头写着这一条）。
# experiment-orphans-model-directory-green 里模型目录中的假 e990 样本不算、只剩 kb 有正文的 E99；experiment-orphans-model-directory-sibling-red 里模型目录外同名形态的照算。
# 这一条读法的弄坏开关归读法那份库（scripts/kb/lib_experiment_number_directories.py 的自证，开关名与红的格写在它文件头），不在这一道设：库的守卫平常跑设了就拒，样本根里设上只会让整格报「没扫全」。
cell_experiment_orphans() {
  EXP_DIR=.claude/kb/experiments
  REGISTRY=.claude/gate.d/experiment-number-directories.tsv
  [[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，这一格跳过"; exit 77; }

  if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == orphans-page-shape-ignored ]]; then   # 弄坏开关：只看条目名的数字前缀
    have=$(ls "$EXP_DIR" | grep -oE '^[0-9]+' | sed 's/^0*//' | sort -un)
  else
    # 有正文的页：名字是「<号>-<简称>.md」的普通文件（不是符号链接），里面有一个非空白字符。这些页一次 grep -l 取出有非空白字符的（读不了的页不在输出里，
    # 与逐页 grep -q 退 2 当「没有正文」同义），按页名查；名字里带换行的页逐页 grep -q。
    # 弄坏开关 orphans-batch-any-body：只要有一页有非空白字符，就当每一页都有正文。
    body_candidates=(); body_listed=(); declare -A body_number=() nonblank_page=()
    for page in "$EXP_DIR"/*; do
      [[ "${page##*/}" =~ ^([0-9]+)-.*\.md$ && -f "$page" && ! -L "$page" ]] || continue
      body_candidates+=("$page"); body_number["$page"]="${BASH_REMATCH[1]}"
      [[ "$page" == *$'\n'* ]] || body_listed+=("$page")
    done
    if ((${#body_listed[@]})); then
      nonblank_pages="$(grep -l '[^[:space:]]' -- "${body_listed[@]}")" || true
      while IFS= read -r hit; do [[ -n "$hit" ]] && nonblank_page["$hit"]=1; done <<<"$nonblank_pages"
    fi
    have=$(for page in "${body_candidates[@]}"; do
             if [[ "$page" == *$'\n'* ]]; then
               grep -q '[^[:space:]]' -- "$page" || continue
             elif [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == orphans-batch-any-body ]]; then
               ((${#nonblank_page[@]})) || continue
             else
               [[ -n "${nonblank_page[$page]:-}" ]] || continue
             fi
             echo "$((10#${body_number[$page]}))"
           done | sort -un)
  fi
  listing_file="$(mktemp)"; errors_file="$(mktemp)"
  trap 'rm -f "$listing_file" "$errors_file"' EXIT
  registry_source="$REGISTRY"; registry_text=""; registry_label="目录登记 $REGISTRY"
  if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == orphans-registry-ignored ]]; then
    registry_source="-"; registry_label="改前写死的四个目录"
    registry_text=$'research\te-number-names\t# 改前写死的那一组\nscripts/experiment-apparatus\te-number-names\t# 改前写死的那一组\nscripts/mutation/tables\te-number-names\t# 改前写死的那一组\ncrates/singlefs-verification/src/bin\te-number-names\t# 改前写死的那一组'
  elif [[ ! -f "$REGISTRY" ]]; then
    echo "  ✗ 目录登记 $REGISTRY 不在：不知道实验号住在哪几处，这一格什么都没扫"
    echo "  → 怎么办：从 git 找回这份登记（git log --diff-filter=D --name-only -- $REGISTRY）；写法在它自己的文件头。"
    exit 1
  fi
  if ! python3 "$LIB_EXPERIMENT_NUMBER_DIRECTORIES" numbers "$registry_source" . <<<"$registry_text" >"$listing_file" 2>"$errors_file"; then
    echo "  ✗ 照 $registry_label 扫实验号没扫全："
    sed 's/^/      /' "$errors_file"
    echo "  → 怎么办：按上面的报错改登记（写法在 $REGISTRY 文件头）或修好读不了的目录，再跑；扫不全的目录会让孤儿实验号静默漏掉。"
    exit 1
  fi
  mapfile -t scope < <(awk -F'\t' '$1 == "entry" { print $2 }' "$listing_file")
  mapfile -t absent < <(awk -F'\t' '$1 == "absent" { print $2 }' "$listing_file")
  want=$(awk -F'\t' '$1 == "number" { print $2 }' "$listing_file" | sort -un)

  orphan=()
  declare -A have_number=()
  for n in $have; do have_number["$n"]=1; done
  for n in $want; do [[ -n "${have_number[$n]:-}" ]] || orphan+=("$n"); done

  if ((${#orphan[@]})); then
    echo "  ✗ $registry_label 列的各处有这些实验号的东西，kb/experiments/ 里却没有正文："
    for n in "${orphan[@]}"; do
      printf '      %-6s' "E$n"
      awk -F'\t' -v number="$n" '$1 == "number" && $2 == number { print $3 }' "$listing_file" | head -5 | tr '\n' ' '
      echo
    done
    echo "  → 怎么办：给它建正文（测什么 / 判据 / 失败条款 / 口径 / 复跑命令），并登记进 experiments.md 的索引表；"
    echo "    若那轮的结论不打算入库，就把 research 下那些文件删掉——留着等于留一份没人能复核的证据。"
    exit 1
  fi
  if [[ -z "$want" ]]; then
    echo "  ⊘ 本次无对象可判：$registry_label 列的 ${scope[*]} 下没有一处出现过实验号（不在的：${absent[*]:-（没有）}）"
    exit 77
  fi
  echo "  ✓ $registry_label 列的各处出现过的实验号在 kb 里都有正文（$(wc -w <<<"$want") 个）"
  echo "    扫了 ${#scope[@]} 处：${scope[*]}（target/ 不扫）；不在的 ${#absent[@]} 处：${absent[*]:-（没有）}"
}

# ── 格 quoted-result-lines：kb 与 research 正文里整行抄的产物行，产物里逐字找得到
#
# 判据：kb 正文（*-history.md 与 decisions-history/ 不算）里去掉首尾空白后以 `E7RESULT ` 开头的行，
# 必须在 research/results/*.out 的某一行里逐字出现；「## 历史版本」之后的行只判这次改动新增的（射程 3）。
# 找不到 = 抄的时候改了数，或者产物重跑之后正文没跟。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）：
#   1. 只认整行。句中反引号里的片段、按字段转述成一句话的数，一概不判。
#   2. 不按实验号配对：正文允许引别的实验的产物（E152 引 E142 的 write_list）。代价是两个实验恰好吐出同一行时，
#      引错了实验认不出来。**这次改动新增或改写的行只对照 `scripts/experiment-apparatus/replay.sh` 登记着的产物**
#      （树里还留着的旧一轮产物不算：登记的已经是新一轮，正文却新抄了旧一轮的行，就是「重跑之后正文没跟」）；
#      登记的产物已归档出了树的，去版本库历史里取它最后那一版。退回全量判时照旧对照全部产物加全部归档。
#   3. 历史节里的旧行不判：旧产物被删掉之后，历史节里留着当时抄的那一行是对的。历史节里**这次改动新增的行**照判（C438），
#      新增与否按行号认（该页相对基准的 git diff -U0，未跟踪的页整份算新增），不按文字认；退回全量判时历史节整节不判（分不出新旧）。
#      历史节里新增的行对照的产物与正文新增的行一样，只是登记着的那些；git diff 取不到时判红。
#   4. 只判这次改动新增或改写的行：正文里早先抄下的行是历史，那时的产物按归档规则删掉了，再判就是判一个不存在的对照；
#      要推翻早先的结论按 `.claude/rules/three-way-inference.md` 重跑。不在 git 仓里、或取不到新增行时退回全量判（宁可多判，不可漏判）。
# 样本：quoted-result-lines-red 里一页未跟踪的新实验页整行抄了两行在登记着的产物里找不到的产物行（一行只在没登记的旧一轮产物里有）；
# quoted-result-lines-green 里正文抄的那一行在产物里逐字找得到、历史节里的旧行不判。
# quoted-result-lines-history-added-red 是已跟踪的页：基准里历史节已有一行错引文，这次改动在历史节里又新增一行错引文，必须红并只点名新增的那一行；
# quoted-result-lines-history-old-green 同一页这次改动只在正文新抄一行对的，历史节里早先的错行不判（成功行的「没判的」里列着它）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=history-added-lines-skipped 退回整节排除，history-added-red 判绿，样本判错。
# 压成 .out.gz 入库的产物（树里的、归档删出树之后只在历史里的）同一处解压再比（product_text）；quoted-result-lines-archived-gz-green / -red 是历史里只有 .out.gz 的一对，
# 开关 archived-gz-not-searched（历史里只认 .out 收尾的路径）与 gz-read-raw（不解压）各让它们判错。
# 登记了搬迁（.claude/gate.d/path-moves.tsv）时，历史节里只因搬迁才变了的行不算新增（new_line_numbers 末尾经 lib-lag-baseline.py 的 drop_moved_lines 剔，
#   与 changed-paths.sh 的 gate_added_lines 给正文剔的同一种；未跟踪的页整份也逐行过一遍）：quoted-result-lines-moved-history-green 历史节里那一行只换了路径，不判、判绿；
#   quoted-result-lines-moved-unregistered-red（没登记）、quoted-result-lines-moved-and-edited-red（登记了又多写一段）照判、判红在那一行。
#   弄坏开关 LAG_BASELINE_BREAK=moved-only-kept 下 quoted-result-lines-moved-history-green 判红。
# 进不了 replay.sh 复跑表的产物（分片跑出、一趟数小时这类 replay.sh 一条任务里整份跑装不下的跑法）登记在 .claude/gate.d/quoted-product-unreplayable.tsv
#   （三列：产物名、怎么跑出的、为什么进不了复跑表）：这次改动新增或改写的行除了对照 replay.sh 登记着的产物，也在这里登记的产物里逐字找
#   （树里没有的去历史里取）；没登记的产物照旧只认 replay.sh。一列空着、产物名在树里与历史里都找不到判红；登记表不在当没有登记。
#   红行里「只在没在 replay.sh 登记的产物里有」那一句按该实验在 replay.sh 登记着什么分别说明：登记着别的产物，那几份可能是旧一轮的；
#   两种出路（回去抄登记着的那份、把这一轮的产物登记进这张表）都点名。
#   样本（放了什么写在样本的 setup.sh 与页里）：quoted-result-lines-unreplayable-green（登记了，整行抄的行在登记的产物里逐字找得到，复跑表里那份的行也照认）、
#   -mistyped-red（登记了但抄错一字）、-unlisted-red（产物在树里、复跑表与登记表都没有它）、-product-missing-red（登记行指到树里与历史里都没有的产物）、
#   -reason-empty-red（登记行没写为什么进不了复跑表）。
#   弄坏开关 unreplayable-registry-ignored（不读登记表）、unreplayable-product-existence-unchecked（不查登记的产物在不在）、
#   unreplayable-reason-unchecked（不查三列写没写全）；各自判错的样本在文件头开关表。
cell_quoted_result_lines() {
  [[ -d .claude/kb ]] || { echo "  ! 找不到 .claude/kb，这一格无对象可判"; exit 77; }

  # 改动范围用这一道开头取的那一份（新增行里未跟踪的文件整份算新增）。
  if ((! changed_paths_library_loaded)); then
    echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
    echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
    exit 1
  fi
  SCOPE_BASE=""
  if ((in_git_work_tree && added_lines_ok)); then
    SCOPE_BASE="$shared_base"
  fi
  ADDED_LINES="$SHARED_ADDED_LINES"
  export SCOPE_BASE ADDED_LINES

  python3 - "$LIB_LAG_BASELINE" "$LIB_ITEM_REF_STATUS" <<'PY'
import glob, importlib.util, os, subprocess, sys

lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', sys.argv[1])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)

item_ref_status_spec = importlib.util.spec_from_file_location('item_ref_status', sys.argv[2])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)

BREAK = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '')
read_translations = BREAK == 'read-translations'

# 射程含 research/ 下的正文（scripts/experiment-apparatus/perf-by-milestone.md 也整行抄产物）；冻结证据目录不判：
# research/prompts/ 按 evidence-discipline 不许事后改，research/results/ 是被比对的一方。
import re
kb_raw = sorted(f for f in glob.glob('.claude/kb/**/*.md', recursive=True)
                 + glob.glob('research/**/*.md', recursive=True)
            if not f.endswith('-history.md') and '/decisions-history/' not in f
            and not f.startswith('research/prompts/') and not f.startswith('research/results/'))
kb = item_ref_status.originals_only(kb_raw, read_translations)
quoted = []
history_quoted = []   # 「## 历史版本」之后整行抄的产物行：只有这次改动新增的那几行判（C438）
for path in kb:
    text = open(path, encoding='utf-8').read()
    in_history = False
    for number, line in enumerate(text.split('\n'), 1):
        if number > 1 and (line.startswith('## 历史版本') or line.startswith('## Historical Versions') or line.startswith('## 履歴バージョン')):
            in_history = True
        stripped = line.strip()
        if stripped.startswith('E7RESULT '):
            (history_quoted if in_history else quoted).append((path, number, stripped))
# 弄坏开关 history-added-lines-skipped：退回整节排除，历史节里新增的行也不判
if os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'history-added-lines-skipped':
    print('  ! 弄坏开关 history-added-lines-skipped：「## 历史版本」整节不判，历史节里这次新增的行也跳过')
    history_quoted = []


def new_line_numbers(path, base):
    """path 里这次改动（相对 base）新增的行的行号集合；未跟踪的文件整份算新增，返回 None。git 取不到就红，不当成一行没改。
    登记了搬迁时只因搬迁才变了的行不算（without_moved_lines）；未跟踪的整份也逐行过一遍，交行号集合。"""
    if subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-files', '--error-unmatch', '--', path], capture_output=True).returncode != 0:
        moves, problems = lag_baseline.path_moves()
        if not moves and not problems:
            return None
        with open(path, encoding='utf-8', errors='surrogateescape') as handle:
            line_count = len(handle.read().split('\n'))
        return without_moved_lines(path, base, set(range(1, line_count + 1)))
    shown = subprocess.run(['git', '-c', 'core.quotepath=false', 'diff', '--no-renames', '--no-color', '--no-ext-diff', '-U0', base, '--', path],
                           capture_output=True, text=True)
    if shown.returncode != 0:
        print(f'  ✗ {path}：git diff 相对基准 {base} 失败（退 {shown.returncode}：{shown.stderr.strip()[:80]}），这一页历史节里新增了哪些行判不了，不是没改')
        print('     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑。')
        sys.exit(1)
    numbers = set()
    for diff_line in shown.stdout.split('\n'):
        hunk = re.match(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@', diff_line)
        if hunk:
            numbers |= set(range(int(hunk.group(1)), int(hunk.group(1)) + (int(hunk.group(2)) if hunk.group(2) is not None else 1)))
    return without_moved_lines(path, base, numbers)


def without_moved_lines(path, base, numbers):
    """只因登记的搬迁才变了的行不算新增（lib-lag-baseline.py 的 drop_moved_lines，与 changed-paths.sh 的 gate_added_lines 剔得一样）。
    搬迁登记表没有内容行时原样交回；登记表写坏了照没有登记时交回、打一行提示；git 取不到判红。"""
    moves, problems = lag_baseline.path_moves()
    if not numbers or (not moves and not problems):
        return numbers
    lines = open(path, encoding='utf-8', errors='surrogateescape').read().split('\n')
    entries = [(path, lines[number - 1], number) for number in sorted(numbers) if 0 < number <= len(lines)]
    kept, problem = lag_baseline.drop_moved_lines(base, entries)
    if isinstance(problem, lag_baseline.PathMovesProblem):
        print(f'  ! {problem.sentence()}；{path} 历史节的新增行没剔只差搬迁的')
        return numbers
    if problem is not None:
        print(f'  ✗ {path}：{problem.sentence()}，这一页历史节里哪些行只因搬迁才变了判不了')
        print('     → 怎么办：按括号里 git 的报错修好仓库状态（基准要存在、对象库没缺）再跑。')
        sys.exit(1)
    return {number for _path, _text, number in kept} | {number for number in numbers if not 0 < number <= len(lines)}


# 只判这次改动新增或改写的行；SCOPE_BASE 为空（不在 git 仓里或取不到新增行）就退回全量判（全量里历史节整节不判）。
scope = '全量'
all_quoted = list(quoted)
if os.environ.get('SCOPE_BASE', '').strip():
    added = set()
    with open(os.environ['ADDED_LINES'], encoding='utf-8', errors='replace') as added_handle:
        for record in added_handle.read().split('\n'):
            added_path, separator, added_text = record.partition('\t')
            if separator and added_text.strip().startswith('E7RESULT '):
                added.add((added_path, added_text.strip()))
    quoted = [(path, number, stripped) for path, number, stripped in quoted
              if (path, stripped) in added]
    # 历史节：按行号认这次新增的行（同一句话早先留在历史节里、正文又新抄一遍，不能按文字冒充新增）
    for history_path in sorted({path for path, _, _ in history_quoted}):
        new_numbers = new_line_numbers(history_path, os.environ['SCOPE_BASE'].strip())
        for item in (entry for entry in history_quoted if entry[0] == history_path):
            all_quoted.append(item)
            if new_numbers is None or item[1] in new_numbers:
                quoted.append(item)
    scope = '这次改动新增或改写的'

if not quoted:
    print('  ! %s kb 正文与历史节里没有整行抄的 E7RESULT 行，这一格无对象可判' % scope)
    sys.exit(77)

# 登记的产物：replay.sh 登记表每一行的第 4 列（lib-lag-baseline.py 的 replay_rows，与 results-cited、verdict-false-named、replay-comparison-units 三格同一份读法，
# 形状与 experiment-replay 格相同）
REPLAY = 'scripts/experiment-apparatus/replay.sh'
registered = None
registered_experiments = set()
registered_by_experiment = {}   # 实验号（大写）-> replay.sh 为它登记着的产物名
if os.path.isfile(REPLAY):
    replay_rows = lag_baseline.replay_rows(REPLAY) or []
    registered = {row.product for row in replay_rows}
    registered_experiments = {row.experiment.upper() for row in replay_rows}
    for row in replay_rows:
        registered_by_experiment.setdefault(row.experiment.upper(), set()).add(row.product)
if scope != '全量' and not registered:
    print(f'  ✗ 读不到 {REPLAY} 的登记表（文件不在，或一行 编号|二进制|参数|产物|exact/timing 都没读到），分不出哪些产物是这一轮登记的')
    print('     → 怎么办：登记表是新抄的产物行唯一的对照；它挪了地方或改了格式，就同步改这一格（与 verdict-false-named 格、门禁 research-build-and-replay.sh 的 experiment-replay 格同一份形状）。')
    sys.exit(1)

BREAK = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '')


def product_text(name, raw_bytes):
    """产物字节转成文本：压成 .out.gz 入库的（超过单文件推送上限的）先解压，判据不变；树里的与历史里取出来的共用这一处。
    弄坏开关 gz-read-raw：不解压，按原字节读。解压不开的（不是 gzip）当场红，不当成一行都没有。"""
    if name.endswith('.gz') and BREAK != 'gz-read-raw':
        try:
            raw_bytes = __import__('gzip').decompress(raw_bytes)
        except (OSError, EOFError) as problem:
            print(f'  ✗ {name} 以 .gz 结尾却解压不开（{problem}），里面有哪些行判不了')
            print('     → 怎么办：产物压缩要用 gzip -n -9 <文件>（别的压缩格式不认）；文件坏了就从历史取回或重跑。')
            sys.exit(1)
    return raw_bytes.decode('utf-8', errors='ignore')


RESULT_PREFIX = 'E7RESULT '


def result_lines(content):
    """产物正文里去掉首尾空白、去掉 \\r 之后以 E7RESULT 起头的行的集合。比对的另一边（整行抄的引文）一定以 E7RESULT 起头，
    不以它起头的产物行永远命中不了，所以只收这一类（产物逐份可达上百 MB，逐行建集合与字典是这一格的大头）。
    正文里没有 \\r 时先按「含 E7RESULT 加一个空格」筛一遍再去空白：去空白只去两端的空白、去 \\r 才可能让一行变成以它起头，没有 \\r 时筛掉的行去空白之后也不以它起头；
    正文里有 \\r 的逐行照去空白、去 \\r 之后再筛。
    弄坏开关 result-lines-raw-prefix：不去空白就筛前缀（缩进的产物行收不进来）；result-lines-prefilter-ignores-carriage-return：有 \\r 也走先筛一遍那一支（\\r 隔在 E7RESULT 与空格之间的行收不进来）。"""
    if BREAK == 'result-lines-raw-prefix':
        return {line.strip().replace('\r', '') for line in content.split('\n') if line.startswith(RESULT_PREFIX)}
    if '\r' in content and BREAK != 'result-lines-prefilter-ignores-carriage-return':
        stripped_lines = (line.strip().replace('\r', '') for line in content.split('\n'))
        return {text for text in stripped_lines if text.startswith(RESULT_PREFIX)}
    if RESULT_PREFIX not in content:
        return set()
    candidates = (line.strip().replace('\r', '') for line in content.split('\n') if RESULT_PREFIX in line)
    return {text for text in candidates if text.startswith(RESULT_PREFIX)}


# 进不了 replay.sh 复跑表的产物登记表（一行一份产物：产物名、怎么跑出的、为什么进不了复跑表）：这里登记的产物里也逐字找整行抄的行。
# 表不在当没有登记；弄坏开关 unreplayable-registry-ignored 不读它。
UNREPLAYABLE = '.claude/gate.d/quoted-product-unreplayable.tsv'
unreplayable = {}   # 产物名 -> (怎么跑出的, 为什么进不了复跑表, 登记表里的行号)
unreplayable_errors = []   # (行号, 说明)
if BREAK != 'unreplayable-registry-ignored' and os.path.exists(UNREPLAYABLE):
    try:
        with open(UNREPLAYABLE, encoding='utf-8') as registry_handle:
            registry_text = registry_handle.read()
    except OSError as problem:
        print(f'  ✗ 登记表 {UNREPLAYABLE} 读不了（{problem}），登记了的产物谁都没判')
        print('     → 怎么办：修好权限再跑；读不了的登记表不能当成没有登记。')
        sys.exit(1)
    for registry_line, registry_row in enumerate(registry_text.split('\n'), 1):
        if not registry_row.strip() or registry_row.startswith('#'):
            continue
        registry_columns = [column.strip() for column in registry_row.split('\t')] + ['', '']
        registry_name, registry_how, registry_why = registry_columns[0], registry_columns[1], registry_columns[2]
        blank = [label for label, value in (('怎么跑出的', registry_how), ('为什么进不了复跑表', registry_why)) if not value]
        if blank and BREAK != 'unreplayable-reason-unchecked':
            unreplayable_errors.append((registry_line, f'第 {registry_line} 行：{registry_name} 没写{"、".join(blank)}'))
        unreplayable[registry_name] = (registry_how, registry_why, registry_line)

products = sorted(glob.glob('research/results/*.out') + glob.glob('research/results/*.out.gz'))
product_lines = set()
registered_lines = set()
line_sources = {}
for product in products:
    base_prod = os.path.basename(product)
    is_registered = (registered is not None and base_prod in registered) or base_prod in unreplayable
    m_exp = re.match(r'^e(\d+)', base_prod)
    if m_exp and ('E' + m_exp.group(1)) not in registered_experiments:
        is_registered = True
    with open(product, 'rb') as handle:
        product_content = product_text(product, handle.read())
    for text in result_lines(product_content):
        product_lines.add(text)
        line_sources.setdefault(text, set()).add(base_prod)
        if is_registered:
            registered_lines.add(text)

# 树里找不到就去版本库历史里找：产物按归档规则删进了 git，历史引文当时逐字是对的；少了这一段，这道检查会退化成「抓归档过的产物」。
# 引文与归档那一刻的产物字节不符照样红。
archived_count = 0
archived_found_names = set()   # 在版本库历史里取到字节的产物文件名（登记表里的产物树里没有时，靠它认「历史里有」）
# 归档的产物可以是 .out，也可以是压成 .out.gz 入库的（取出来经 product_text 解压，与树里的同一处）；弄坏开关 archived-gz-not-searched：只认 .out 收尾的路径，历史里只有 .out.gz 的算找不到
ARCHIVED_SUFFIXES = ('.out',) if BREAK == 'archived-gz-not-searched' else ('.out', '.out.gz')


blob_reader = None


def read_blob(revision_path):
    """git 里 <提交>:<路径> 那一份的字节；取不出（提交不在、路径不在）返回 None，与 git show 同义。
    归档的产物逐份取，常驻一个 git cat-file --batch 进程按名字一份份问，省掉每份一个 git show 子进程；
    读不出标准的三行应答、取到的不是 blob、进程起不来或中途断了，这一份退回 git show 取。
    弄坏开关 archived-blob-batch-unread：批量那一路取到的字节读成空（取得到的份数不变、内容全空）。"""
    global blob_reader
    try:
        if blob_reader is None:
            blob_reader = subprocess.Popen(['git', 'cat-file', '--batch'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        blob_reader.stdin.write(revision_path.encode('utf-8') + b'\n')
        blob_reader.stdin.flush()
        header = blob_reader.stdout.readline()
        if header.endswith(b' missing\n'):
            return None
        fields = header.split()
        if len(fields) == 3 and fields[1] == b'blob':
            size = int(fields[2])
            content = blob_reader.stdout.read(size)
            blob_reader.stdout.read(1)
            if len(content) == size:
                return b'' if BREAK == 'archived-blob-batch-unread' else content
    except (OSError, ValueError):
        pass
    # 应答认不出、进程断了：这个进程与下一份的应答可能已错位，丢掉它、这一份走 git show
    if blob_reader is not None:
        blob_reader.kill()
        blob_reader.wait()
        blob_reader = None
    shown = subprocess.run(['git', 'show', revision_path], capture_output=True)
    return shown.stdout if shown.returncode == 0 else None


def blob_lines(entry, revision_path):
    """归档产物的字节（read_blob），按文件名解压（若压过）后逐行去首尾空白、只收 E7RESULT 行（result_lines）；取不出返回 None。"""
    blob = read_blob(revision_path)
    if blob is None:
        return None
    return result_lines(product_text(entry, blob))


deleted_products_log = None


def archived_product_lines(only_names=None):
    """归档进版本库的产物的行；only_names 给了就只取这些文件名的（登记着、却已出了树的那几份）。"""
    global archived_count, deleted_products_log
    lines = set()
    if deleted_products_log is None:
        # 删除清单整趟只取一次（这个函数一趟最多调两次，两次问的是同一份历史）
        deleted_products_log = subprocess.run(['git', '-c', 'core.quotepath=false', 'log', '--all', '--diff-filter=D', '--format=%H', '--name-only',
                                               '--', 'research/results'], capture_output=True, text=True).stdout
    log = deleted_products_log
    commit = None
    for entry in log.split('\n'):
        entry = entry.strip()
        if not entry:
            continue
        if len(entry) == 40 and all(character in '0123456789abcdef' for character in entry):
            commit = entry
            continue
        if commit and entry.endswith(ARCHIVED_SUFFIXES) and (only_names is None or os.path.basename(entry) in only_names):
            archived = blob_lines(entry, '%s^:%s' % (commit, entry))
            if archived is not None:
                archived_count += 1
                archived_found_names.add(os.path.basename(entry))
                lines |= archived
    # 正在归档的：HEAD 里还在、工作区里已经删了的产物（这一批照归档规则删上一轮的产物，删除还没提交，
    # git log --diff-filter=D 还找不到它）。它们的字节就在 HEAD 里，与已提交的归档是同一件事。
    in_head = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-tree', '-r', '--name-only', 'HEAD', '--', 'research/results'],
                             capture_output=True, text=True)
    for entry in in_head.stdout.split('\n') if in_head.returncode == 0 else []:
        entry = entry.strip()
        if entry.endswith(ARCHIVED_SUFFIXES) and not os.path.exists(entry) and (only_names is None or os.path.basename(entry) in only_names):
            archived = blob_lines(entry, 'HEAD:%s' % entry)
            if archived is not None:
                archived_count += 1
                archived_found_names.add(os.path.basename(entry))
                lines |= archived
    return lines

# 登记表里的产物：树里有的，行已经在上面的循环里并进登记着的产物的行；树里没有的去历史里取，都找不到判红（改了名或删了）
if unreplayable:
    off_tree = set(unreplayable) - {os.path.basename(product) for product in products}
    if off_tree:
        registered_lines |= archived_product_lines(only_names=off_tree)
        if BREAK != 'unreplayable-product-existence-unchecked':
            for name in sorted(off_tree - archived_found_names):
                line_in_registry = unreplayable[name][2]
                unreplayable_errors.append((line_in_registry, f'第 {line_in_registry} 行：{name} 在 research/results/ 与版本库历史里都找不到'))
if unreplayable_errors:
    print(f'  ✗ 登记表 {UNREPLAYABLE} 有 {len(unreplayable_errors)} 处写得不对：')
    for _line, text in sorted(unreplayable_errors):
        print(f'      {text}')  # gate-lint:detail
    print('     → 怎么办：一行一份产物，<产物文件名><TAB><怎么跑出的><TAB><为什么进不了复跑表>（写法在登记表文件头）；')
    print('               产物改了名或删了，改这一行或删这一行；写不出怎么跑出的、为什么进不了复跑表的，说明它能进 replay.sh 复跑表，别登记在这里。')
    sys.exit(1)

if scope == '全量':
    compared_lines = product_lines
else:
    compared_lines = registered_lines
missing = [(path, number, stripped) for path, number, stripped in quoted if stripped not in compared_lines]
if missing:
    if scope == '全量':
        compared_lines |= archived_product_lines()
    else:
        on_disk = {os.path.basename(product) for product in products}
        compared_lines |= archived_product_lines(only_names=registered - on_disk)
    missing = [item for item in missing if item[2] not in compared_lines]
if missing:
    compared_what = (f'research/results/ 的 {len(products)} 份产物' if scope == '全量'
                     else f'{REPLAY} 登记着的 {len(registered)} 份产物'
                     + (f'与 {UNREPLAYABLE} 登记的 {len(unreplayable)} 份产物' if unreplayable else ''))
    print(f'  ✗ {scope} kb 正文与历史节里有 {len(missing)} 行整行抄的产物行，在 {compared_what}里一份都找不到：')
    for path, number, stripped in missing:
        print(f'     {path}:{number}  {stripped[:160]}')  # gate-lint:detail
        unregistered = sorted(line_sources.get(stripped, ()))
        if unregistered:
            siblings = set()
            for unregistered_name in unregistered:
                unregistered_experiment = re.match(r'^e(\d+)', unregistered_name)
                if unregistered_experiment:
                    siblings |= registered_by_experiment.get('E' + unregistered_experiment.group(1), set())
            sibling_text = (f'；replay.sh 为这个实验登记着 {"、".join(sorted(siblings)[:3])}，正文该抄的若是登记着的那份，回去抄它的行'
                            if siblings else '')
            print(f'       只在没在 replay.sh 登记的产物里有（{"、".join(unregistered[:3])}）：它们没进 replay.sh 的登记，也没进 {UNREPLAYABLE}{sibling_text}')  # gate-lint:detail
    print('     → 怎么办：打开这个实验复跑命令登记的那份产物，把对应的行原样抄回来，别手改数；')
    print('               正文里照这几行写的结论句一起回对。产物确实没留存，就删掉这几行并写明「原始输出未留存」。')
    print(f'               正文抄的就是这一轮跑出的产物、只是 replay.sh 起不了它的跑法（分片跑这类）：在 {UNREPLAYABLE} 登一行（产物名、怎么跑出的、为什么进不了复跑表），整行抄的行就在它里面找。')
    sys.exit(1)
compared_count = len(products) if scope == '全量' else len(registered)
print(f'  ✓ {scope} kb 正文与历史节里整行抄的产物行都在产物里逐字找得到（判了 {len(quoted)} 行，对照 {compared_count} 份'
      + ('产物）' if scope == '全量' else '登记着的产物）'))
if unreplayable:
    print(f'     另认 {UNREPLAYABLE} 登记的 {len(unreplayable)} 份进不了复跑表的产物（{"、".join(sorted(unreplayable)[:3])}）')
judged = {(path, number) for path, number, _ in quoted}
skipped = {}
for path, number, _ in all_quoted:
    if (path, number) not in judged:
        skipped.setdefault(path, []).append(number)
skipped_count = sum(len(numbers) for numbers in skipped.values())
print(f'     没判的 {skipped_count} 行（早先抄下的，历史参考，产物按「每次提交删上一次的实验记录」删掉了；要推翻它们按三方重跑）'
      + (''.join(f'\n       {path}:{",".join(str(n) for n in numbers)}' for path, numbers in sorted(skipped.items())) if skipped else '：（没有）'))
PY
}

# ── 格 archive-past-rounds：上一轮及更早的实验记录已经归档进版本库，工作区里不留
#
# 规则：.claude/rules/evidence-discipline.md「原样保存的证据不许事后改」之下那三段；判法与保留哪几类在 scripts/kb/archive-past-rounds.py。
# 基准是这一道开头取的 gate_diff_base gate（与 doc-process-records.sh 的 knowledge-sync 格同一个），经 GATE_BASE 交给脚本，出路里打出的基准就是这个提交。
# 判别力：fixtures/doc-experiments.sh/archive-past-rounds-red 是一个留着上一轮产物的小仓，必须判红；
# archive-past-rounds-green 只有本轮的，必须判绿。
cell_archive_past_rounds() {
  [[ -d "$ROOT/research/prompts" || -d "$ROOT/research/results" ]] || { echo "  ! 没有 research/prompts 与 research/results，这一格无对象可判"; exit 77; }
  # 不在 git 仓里分不出哪些是上一轮提交进来的（脚本的 git ls-files 会带着调用栈退出）：这一格无对象可判，不记通过
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，分不出哪些记录是上一轮提交进来的，这一格无对象可判"; exit 77; }
  if ((changed_paths_library_loaded)); then
    if GATE_BASE="$shared_base" python3 "$ARCHIVE_SCRIPT" --check "$ROOT"; then
      exit 0
    fi
  elif python3 "$ARCHIVE_SCRIPT" --check "$ROOT"; then
    exit 0
  fi
  echo "  ✗ 上一轮及更早的实验记录没收干净（上面 scripts/kb/archive-past-rounds.py 逐类列了红的是哪几类：工作区里还留着的上一轮记录、登记表点名而已经不在的路径、这一趟要删而被平铺登记表点名的产物）"
  echo "     → 怎么办：还留着的上一轮记录，照上面那一行打出来的基准跑 --apply 删掉（不带基准它会按 HEAD 算、把这一次提交要带的记录也删掉），"
  echo "               它同时把别处指向这些文件的引用改成只留文件名、不留路径；删完跑共享门禁的「链接指向」阶段确认链接都还到得了。"
  echo "               登记表指空、平铺登记表点名的那几行，同一次提交里删掉或改掉那几行登记（上面每一类自己的出路写了怎么改）。"
  exit 1
}

# ── 格 multipath-registry：多条路径互证的实验，路径与共用项要登记且指得到
#
# 判据与形态在 `.claude/rules/format-evolution.md`「实验页的「路径与结论登记」」，这一格判四条：
#   ① 登记表的形状：`### 路径与结论登记` 之下恰好一张表，表头逐字
#      `| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |`，每行五格；表头下面要有分隔行，
#      数据行按「不是分隔行」认，不按位置跳过前两行（少了分隔行时第一条数据行照判）；
#   ② 源码落点指得到：写成 `路径:行号` 或 `路径:行号 函数名` 的，那个文件要在仓里现存；只活在正文里的写「无实现：理由」，理由不许空；
#   ③ 共用项那一格不许空，也不许只写「无」（去掉括号、标点与「共用项」这类字眼之后只剩「无」「没有」「N/A」「none」这一类的都算）；
#   ④ 还没写登记节的实验页登记在 `.claude/gate.d/multipath-registry-lag.tsv`，这张表每一行：
#      两列（页面路径、普查判定的依据）用制表符分隔、都不许空；路径形如
#      `.claude/kb/experiments/<编号>-….md` 且文件在仓里现存；那一页确实还没有登记节——补上了就判过期，删行；
#      **只缩不涨**：与基准提交比，基准里这张表没有的实验编号不许出现。按编号比、不按整条路径比：
#      页面改名、登记行抄错了题目改正，都不算新加了一份。基准里还没有滞后表时不比；git 取不到基准那一版判红（取法 lib-lag-baseline.py 的 baseline_rows）。
#   没有一页写了登记节、滞后表也一行都没有，退 77。
#
# 射程只到已经写了这一节的实验页。「该写而没写」要先认出实验在做多条路径互证，按关键词扫正文认不出来，
# 那一半欠在 C48（多条校验路径共用同一个前提）。
#
# 判别力：fixtures/doc-experiments.sh/{multipath-registry-red,multipath-registry-green}/ 各一份实验页，
# red 那份四行里一行共用项写「无」、一行写「N/A」、一行源码落点指不到文件，另一份页的表缺分隔行、第一条数据行共用项写「无」，
# 另带一张滞后表：一行指向不存在的页、
# 一行指向已经写了登记节的页、一行少了依据那一列、一行的实验编号基准里没有，必须判红；
# green 的滞后表只有一行、指向一份还没写登记节的页、与基准相同，必须判绿。
cell_multipath_registry() {
  EXPERIMENTS=.claude/kb/experiments
  LAG=.claude/gate.d/multipath-registry-lag.tsv
  [[ -d "$EXPERIMENTS" ]] || { echo "  ! 没有 $EXPERIMENTS，这一格无对象可判"; exit 77; }
  base=""
  if ((in_git_work_tree)); then
    if ((! changed_paths_library_loaded)); then
      echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
      echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
      exit 1
    fi
    base="$shared_base"
  fi

  python3 - "$EXPERIMENTS" "$LAG" "$base" "$LIB_LAG_BASELINE" <<'PY'
import importlib.util, os, re, sys, glob

experiments, lag_path, base = sys.argv[1], sys.argv[2], sys.argv[3]
lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', sys.argv[4])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
HEADING = '### 路径与结论登记'
WANTED = '| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |'
LAG_PAGE = re.compile(r'\.claude/kb/experiments/(?P<number>[0-9]+)-[^/]+\.md')

def has_registration(text):
    return HEADING in text

SAYS_NOTHING = ('', '无', '没有', '没', 'na', 'none', 'nil', '不适用')

def says_nothing(shared):
    """共用项那一格去掉括号、标点、空白与「共用项 / 共用 / 共享」这类字眼之后，只剩「无」这一类：等于没写。"""
    core = re.sub(r'[\s（）()\[\]【】「」`*。．.，,、;；:：/\\\-—–_]+', '', shared).lower()
    core = re.sub(r'共用项|共用|共享', '', core)
    return core in SAYS_NOTHING

problems, checked_pages, checked_rows = [], 0, 0
for path in sorted(glob.glob(os.path.join(experiments, '*.md'))):
    text = open(path, encoding='utf-8').read()
    if not has_registration(text):
        continue
    checked_pages += 1
    section = text.split(HEADING, 1)[1]
    cut = re.search(r'^#{2,4}\s', section, re.M)
    if cut:
        section = section[:cut.start()]
    rows = [line for line in section.split('\n') if line.startswith('|')]
    if not rows:
        problems.append(f"{path}：「{HEADING}」之下一张表都没有")
        continue
    if rows[0].strip() != WANTED:
        problems.append(f"{path}：表头不是 {WANTED}，实际是 {rows[0].strip()[:60]}")
        continue
    if len(rows) < 2 or not re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', rows[1].strip()):
        problems.append(f"{path}：表头下面没有分隔行（|---|---|…|），表格渲染不出来；下面的数据行照判")
    for row in rows[1:]:
        if re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', row.strip()):
            continue
        cells = [cell.strip() for cell in row.strip().strip('|').split('|')]
        if len(cells) != 5:
            problems.append(f"{path}：这一行不是五格：{row.strip()[:60]}")
            continue
        checked_rows += 1
        number, name, _how, where, shared = cells
        for hit in re.findall(r'`([^`:]+?):(\d+)(?:\s[^`]*)?`', where):
            if not os.path.exists(hit[0]):
                problems.append(f"{path}：第 {number} 行「{name}」的源码落点指不到文件：{hit[0]}")
        if not re.search(r'`[^`:]+?:\d+(?:\s[^`]*)?`', where) and not where.startswith('无实现：'):
            problems.append(f"{path}：第 {number} 行「{name}」的源码落点既不是 路径:行号，也不是「无实现：理由」：{where[:40]}")
        if where.startswith('无实现：') and not where[len('无实现：'):].strip():
            problems.append(f"{path}：第 {number} 行「{name}」写了「无实现：」而理由是空的")
        if says_nothing(shared):
            problems.append(f"{path}：第 {number} 行「{name}」的共用项那一格是空的或只写了「{shared}」")

# ④ 滞后表：形状、路径、过期、只缩不涨
lag_problems, lagging, lag_pages_by_number = [], [], {}
if os.path.isfile(lag_path):
    for line_number, fields in lag_baseline.content_rows(open(lag_path, encoding='utf-8').read()):
        where = f"{os.path.basename(lag_path)} 第 {line_number} 行"
        if len(fields) != 2 or not all(field.strip() for field in fields):
            lag_problems.append(f"{where}：不是两列（页面路径、普查判定的依据）或有一列是空的：{chr(9).join(fields)[:80]}")
            continue
        page = fields[0].strip()
        page_shape = LAG_PAGE.fullmatch(page)
        if not page_shape:
            lag_problems.append(f"{where}：路径不是 .claude/kb/experiments/<编号>-….md 的形状：{page}")
            continue
        lag_pages_by_number[int(page_shape.group('number'))] = page
        if not os.path.isfile(page):
            lag_problems.append(f"{where}：指向的实验页不存在：{page}")
            continue
        if has_registration(open(page, encoding='utf-8').read()):
            lag_problems.append(f"{where}：{page} 已经写了「{HEADING}」，这一行过期了")
            continue
        lagging.append(page)

baseline_numbers = None
if base:
    # 「基准里还没有这张表」与「git 取不到基准那一版」分开（lib-lag-baseline.py 的 baseline_rows）：后者判红，不当成没有基线静默不比
    baseline, baseline_problem = lag_baseline.baseline_rows(base, lag_path)
    if baseline_problem is not None:
        lag_problems.append(f"{baseline_problem.sentence()}，只缩不涨没比")
    elif baseline is not None:
        baseline_numbers = set()
        for _line_number, fields in baseline:
            page_shape = LAG_PAGE.fullmatch(fields[0].strip())
            if page_shape:
                baseline_numbers.add(int(page_shape.group('number')))
grown = sorted(set(lag_pages_by_number) - baseline_numbers) if baseline_numbers is not None else []

if problems:
    print(f"  ✗ {len(problems)} 处登记不合形态：")                       # gate-lint:summary
    for entry in problems:
        print(f"     {entry}")                                          # gate-lint:detail
    print("     → 怎么办：形态见 .claude/rules/format-evolution.md「实验页的「路径与结论登记」」。")
    print("               表头逐字 | # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |，每行五格；")
    print("               源码落点写 `路径:行号 函数名`，只活在正文里的写「无实现：理由」；")
    print("               共用项写这条路径读了哪些常量、哪些规则、哪份规范，一个都不许省——")
    print("               多条路径逐格相等不构成证据，当它们共用同一个错误前提时。")
if lag_problems:
    print(f"  ✗ 滞后表 {lag_path} 里 {len(lag_problems)} 行对不上仓里的实验页：")  # gate-lint:summary
    for entry in lag_problems:
        print(f"     {entry}")                                          # gate-lint:detail
    print("     → 怎么办：路径抄错了就照 .claude/kb/experiments/ 里现存的文件名改；那一页已经补了登记节，就删掉这一行——")
    print("               留着的滞后行会让人以为那一页还欠着，而指向不存在的页等于那一格没人管。")
if grown:
    print(f"  ✗ 滞后表涨了：这些实验页的编号，基准 {base} 的滞后表里没有：")  # gate-lint:summary
    for number in grown:
        print(f"     {lag_pages_by_number[number]}")                     # gate-lint:detail
    print("     → 怎么办：这张表只缩不涨。新写的实验页要做多条路径互证，就当场写「### 路径与结论登记」那一节，")
    print("               不许加一行滞后把它按下去——加得了一行，这一格对新写的实验页就什么都拦不住了。")
if problems or lag_problems or grown:
    sys.exit(1)

if checked_pages == 0 and not lagging and not lag_pages_by_number:
    print(f"  ⊘ 本次无对象可判：没有一页实验写了「{HEADING}」，{os.path.basename(lag_path)} 里也一行都没有")
    sys.exit(77)
compared = f"与基准 {base} 比没涨" if baseline_numbers is not None else "基准里还没有这张表（或不是 git 仓），只缩不涨这一条没比"
print(f"  ✓ 登记了路径与共用项的实验页判过了（{checked_pages} 份、{checked_rows} 条路径）；"
      f"还没写登记节的 {len(lagging)} 份在 {os.path.basename(lag_path)} 里、每一份都现存且确实还没写，{compared}；逐份补齐后删行")
PY
}

# ── 格 cited-paths-exist：kb 点名的实验产物、变异表、实验源码，路径要在树里或版本库历史里
#
# 这是 C43 反向那一半的存量判：results-cited 格第三道只判「这次改动新增或改写的点名行」、只判 research/results/ 下的产物，
# 存量的点名行与变异表、实验源码两类它都不看。分工：results-cited 第三道按这次改动、认裸文件名、带登记在 replay.sh 的那一道判法；
# 这一格不看改动范围，每次判 kb 全部正文，只判存在性。
#
# 扫哪些文件：.claude/kb 与 crates/singlefs-checker/invariants 下全部 .md（含历史节、含子目录），
# 再加 .claude/kb-directories 登记的别的目录；目录不在就跳过，一份 .md 都扫不到退 77。
# 认哪三类路径（整路径形态，反引号、括号、链接里都认；`../../research/results/…` 这种相对链接按仓内路径算）：
#   research/results/<名>.out（名里可带子目录）、scripts/mutation/tables/<名>.tsv、scripts/experiment-apparatus/<名>.rs。
# 光写文件名不带目录的不认（那是 results-cited 第二道的事）；带通配符、占位符（`e17-*.out`、`<名>.out`）或写成区间（`c113-r1..r6-local.out`）的不是一个具体路径，不认。
# 每个路径去重后判一次：工作区里在、或 git ls-files 里在 → 树里；不在 → git log --all --diff-filter=A 找得到加入过 → 只在历史里；都没有 → 红。
# 不在 git 仓里没有历史可查，树里没有的一律判红。
#
# 判别力：fixtures/doc-experiments.sh/cited-paths-exist-red 一页点名一个树里与历史里都没有的产物 e9-missing.out、
# 一张不存在的变异表、一个只在写法上带区间的名字（不该被认）和一个存在的实验源码，必须判红且只点名那两处；
# cited-paths-exist-green 点名三类各一个在树里的、另一个产物先提交后删掉（只在历史里），必须判绿并报「只在历史里 1 个」。
# 产物压成 <名>.out.gz 入库之后，点名 <名>.out 而树里、历史里只有 <名>.out.gz 的也算找得到：存放名由 results-cited 格的 product_stored_forms 列
# （export -f 递给 python，不抄第二份）；变异表与实验源码只认原样。cited-paths-exist-gz-green 绿、cited-paths-exist-gz-red 红；
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=cited-path-gz-unmatched：只查原样的名字，gz-green 判错。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=history-lookup-skipped：历史里找得到也算缺，green 判红；
# existence-unchecked：一律当在，red 判绿。
cell_cited_paths_exist() {
  export -f product_stored_forms   # python 里查存放名要调它（results-cited 格同一份）
  python3 - "$ROOT" <<'PY'
import os, re, subprocess, sys

root = sys.argv[1]
break_switch = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '')
os.chdir(root)

NAME = r'[A-Za-z0-9_+-]+(?:\.[A-Za-z0-9_+-]+)*'
KINDS = [
    ('research/results 下的产物', r'research/results/(?:' + NAME + r'/)*' + NAME + r'\.out'),
    ('变异表', r'scripts/mutation/tables/(?:' + NAME + r'/)*' + NAME + r'\.tsv'),
    ('实验源码', r'scripts/experiment-apparatus/(?:' + NAME + r'/)*' + NAME + r'\.rs'),
]
cited_path = re.compile(r'(?<![A-Za-z0-9_-])(' + '|'.join(f'(?P<k{i}>{pattern})' for i, (_label, pattern) in enumerate(KINDS))
                        + r')(?![A-Za-z0-9_+-]|\.[A-Za-z0-9])')

directories = ['.claude/kb', 'crates/singlefs-checker/invariants']
if os.path.isfile('.claude/kb-directories'):
    with open('.claude/kb-directories', encoding='utf-8') as registry:
        for line in registry:
            entry = line.split('#', 1)[0].strip()
            if entry and entry not in directories:
                directories.append(entry)

pages = []
for directory in directories:
    for walked, _names, files in os.walk(directory):
        pages.extend(os.path.join(walked, name) for name in files if name.endswith('.md'))
pages = sorted(set(pages))
if not pages:
    print(f"  ! 没有可扫的 .md（{', '.join(directories)} 下一份都没有），这一格无对象可判")
    sys.exit(77)

places = {}      # 路径 -> [(页, 行号)]
kind_of = {}
# 哪一行写明了原件没留存（出路里给的那种写法），那一行点名的路径就是如实交代过的：kb 里别处再点名它，也不算指空
unretained_declared = set()
if break_switch != 'unretained-note-ignored':
    for page in pages:
        with open(page, encoding='utf-8', errors='replace') as handle:
            for line in handle:
                if '原件未留存' in line or '原始输出未留存' in line:
                    unretained_declared.update(found.group(1) for found in cited_path.finditer(line))
unretained_noted = 0
for page in pages:
    with open(page, encoding='utf-8', errors='replace') as handle:
        for number, line in enumerate(handle, 1):
            for found in cited_path.finditer(line):
                path = found.group(1)
                if path in unretained_declared:
                    unretained_noted += 1
                    continue
                places.setdefault(path, []).append((page, number))
                kind_of[path] = next(i for i in range(len(KINDS)) if found.group(f'k{i}'))

in_git = subprocess.run(['git', 'rev-parse', '--is-inside-work-tree'], capture_output=True, text=True).returncode == 0
tracked = set()
if in_git:
    listed = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-files', '-z'], capture_output=True)
    if listed.returncode != 0:
        print(f"  ✗ git ls-files 退 {listed.returncode}（{listed.stderr.decode('utf-8', 'replace').strip()[:80]}），树里有没有没判成")   # gate-lint:summary
        print("     → 怎么办：先让 git 在这个仓里能正常跑（索引锁、仓损坏），再重跑这一格；不许把没判成当成都在。")   # gate-lint:detail
        sys.exit(1)
    tracked = set(listed.stdout.decode('utf-8', 'replace').split('\0'))

# 产物类路径的存放名一次问齐（一个 bash 进程逐个调 product_stored_forms，每个路径一段：「@@ <退出码> <路径>」接着它打的每个名字）。
STORED_FORMS_SCRIPT = r'''
while IFS= read -r stored_path; do
  stored_forms="$(product_stored_forms "$stored_path")"; stored_status=$?
  printf '@@ %s %s\n' "$stored_status" "$stored_path"
  printf '%s\n' "$stored_forms"
done
'''
batched_forms = {}   # 路径 -> [product_stored_forms 的退出码, 它打的名字]
batched_stderr = ''
if break_switch != 'cited-path-gz-unmatched':
    product_paths = [path for path in sorted(places) if kind_of[path] == 0]
    if product_paths:
        batched = subprocess.run(['bash', '-c', STORED_FORMS_SCRIPT, '_'], input=''.join(path + '\n' for path in product_paths), capture_output=True, text=True)
        batched_stderr = batched.stderr.strip()[:80]
        current_path = None
        for batched_line in batched.stdout.split('\n'):
            if batched_line.startswith('@@ '):
                _marker, status_text, current_path = batched_line.split(' ', 2)
                batched_forms[current_path] = [int(status_text), []]
            elif batched_line and current_path is not None:
                batched_forms[current_path][1].append(batched_line)


def stored_forms(path):
    """点名的路径在树里、历史里可能的存放名：.out 产物压成 .out.gz 入库之后只剩后者。
    判法是 results-cited 格的 product_stored_forms（同一份，bash 函数经 export -f 递过来，上面一个进程问齐所有产物类路径），这里不抄第二份；其余两类路径只认原样。"""
    if kind_of[path] != 0 or break_switch == 'cited-path-gz-unmatched':
        return [path]
    status, forms = batched_forms.get(path, [127, []])
    if status != 0 or path not in forms:
        print(f"  ✗ product_stored_forms 查 {path} 的存放名退 {status}（{batched_stderr}），这一个没判成")   # gate-lint:summary
        print("     → 怎么办：它是 doc-experiments.sh 里 results-cited 格用的同一个函数，看这一份脚本有没有被改坏；不许把没判成当成在。")   # gate-lint:detail
        sys.exit(1)
    return forms


in_tree, only_history, absent = [], [], []
for path in sorted(places):
    forms = stored_forms(path)
    if break_switch != 'existence-unchecked' and not any(os.path.isfile(form) or form in tracked for form in forms):
        found_in_history = False
        if in_git and break_switch != 'history-lookup-skipped':
            for form in forms:
                logged = subprocess.run(['git', '-c', 'core.quotepath=false', 'log', '--all', '--diff-filter=A', '-n1', '--format=%H', '--', form],
                                        capture_output=True, text=True)
                if logged.returncode != 0:
                    print(f"  ✗ git log 查 {form} 的历史退 {logged.returncode}（{logged.stderr.strip()[:80]}），这一个没判成")   # gate-lint:summary
                    print("     → 怎么办：先让 git 在这个仓里能正常跑，再重跑这一格；不许把没判成当成历史里有。")   # gate-lint:detail
                    sys.exit(1)
                if logged.stdout.strip():
                    found_in_history = True
                    break
        (only_history if found_in_history else absent).append(path)
    else:
        in_tree.append(path)

if absent:
    reference_count = sum(len(places[path]) for path in absent)
    print(f"  ✗ kb 点名的 {len(absent)} 个路径（{reference_count} 处引用）树里没有、版本库历史里也没有：")   # gate-lint:summary
    for path in absent:
        shown = '、'.join(f'{page}:{number}' for page, number in places[path][:3])
        more = f" 等 {len(places[path])} 处" if len(places[path]) > 3 else ''
        print(f"     {shown}{more} 点名 {path}（{KINDS[kind_of[path]][0]}）")   # gate-lint:detail
    print("     → 怎么办：补回那份文件；或把那一行改成写明「原件未留存、从哪次提交取」；或改成现存的路径")
    print("               （移过位置的在 git log --all --diff-filter=A --name-only -- '*<文件名>' 里找新路径）。")
    print("               只写文件名不写目录的不在这一格的射程里，改成不带目录的写法不算修好——取不回来的就如实写原件未留存。")
    sys.exit(1)

by_kind = [sum(1 for path in places if kind_of[path] == i) for i in range(len(KINDS))]
reference_total = sum(len(spots) for spots in places.values())
if not places:
    print(f"  ! 本次无对象可判：扫了 {len(pages)} 份 .md，一个整路径形态的产物、变异表、实验源码路径都没有")
    sys.exit(77)
kinds_text = '、'.join(f'{KINDS[i][0]} {by_kind[i]}' for i in range(len(KINDS)))
print(f"  ✓ {len(pages)} 份 .md 点名的 {len(places)} 个路径（{reference_total} 处引用；{kinds_text}）都找得到：树里 {len(in_tree)} 个、只在历史里 {len(only_history)} 个")
if only_history:
    print("     只在历史里的（树里没有，git log --all --diff-filter=A 找得到它加入过）：")
    for path in only_history:
        print(f"       {path}")
if unretained_noted:
    print(f"     写明了原件未留存的 {len(unretained_declared)} 个路径，点名 {unretained_noted} 处不判")
PY
}
