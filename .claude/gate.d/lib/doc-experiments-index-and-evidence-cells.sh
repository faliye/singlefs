#!/usr/bin/env bash
# 门禁 doc-experiments 的格函数库：index-sync、results-cited、evidence-in-repo、decision-links。
# 由 .claude/gate.d/doc-experiments.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ── 格 index-sync：实验索引行与正文标题说的是不是同一件事
# 索引行与正文标题 `## E<n> 简称 —— 状态` 各写各的，就会一处改对、一处停在旧值，只检索到索引那一行的人拿到的是作废结论。
# 查四样：
#   1. 正文标题带「结论作废 / 结论整体作废」的，索引行里必须也出现「作废」
#   2. 正文标题的状态词是「未跑」「部分已跑」或「已跑（够判）」的，索引状态列必须跟着说
#   3. 索引结论列里的数，正文里要找得到
#   4. 两边一一对应：正文有而索引无、索引有而正文无都红
# 管不到：旧值以历史叙述的形态留在正文里时（「从 31.6M 抬到 31.8M」），第 3 项照样判它找得到。
# 样本：index-sync-red 里一行正文宣告作废而索引没提、一行索引结论列的数正文里找不到、一行正文「已跑（够判）」而索引没跟、
# 一行按子串会被「100」冒充找到的「10」；index-sync-green 里补零链接、结论整体作废两边都写了、够判两边都写了。
# 每个实验号的正文取哪一份：lib-item-ref-status.py 的 numbered_files，索引是中文，取中文原文（只有译文的号取那份译文，缺中文原文归 doc-text 的 trilingual-files）。
# 样本：index-sync-translation-green / -red（那个数在中文原文里 / 只在英语译文里）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
cell_index_sync() {
  local IDX=.claude/kb/experiments.md
  local EXP=.claude/kb/experiments
  [[ -f "$IDX" && -d "$EXP" ]] || { echo "  ! 找不到 $IDX 或 $EXP，这一格跳过"; exit 77; }

  python3 - "$IDX" "$EXP" "$LIB_ITEM_REF_STATUS" <<'PY'
import importlib.util, re, sys, glob, os

idx_path, exp_dir = sys.argv[1], sys.argv[2]
spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[3])
item_ref_status = importlib.util.module_from_spec(spec); spec.loader.exec_module(item_ref_status)

def unspace(s):
    for ch in (" ", " ", " "):   # 窄不换行空格、窄空格、不换行空格
        s = s.replace(ch, " ")
    return re.sub(r"(?<=\d)[ 　](?=\d\d\d)", "", s)

bodies = {}
# 每个实验号只取一份正文：索引是中文，取中文原文（numbered_files），不按 glob 不排序的次序逐份覆盖
# （那样留下哪一份，中文还是哪份译文，看目录项的次序）。只有译文、没有中文原文的号照旧算「有正文」（取那份译文），缺中文原文归 doc-text 的 trilingual-files
all_pages = sorted(glob.glob(os.path.join(exp_dir, "*.md")))
for n in sorted({int(os.path.basename(p).split("-")[0]) for p in all_pages}):
    found = (item_ref_status.numbered_files(exp_dir, n, idx_path)
             or [p for p in all_pages if re.match(r"0*%d-" % n, os.path.basename(p))])
    p = found[0]
    text = open(p, encoding="utf-8").read()
    bodies[n] = (os.path.basename(p), text.splitlines()[0], unspace(text))

rows = []
for line in open(idx_path, encoding="utf-8"):
    if not line.startswith("| E"):
        continue
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    m = re.match(r"E(\d+)", cells[0])
    if m and len(cells) >= 3:
        rows.append((int(m.group(1)), cells[1], cells[2], line))

def status_seg(title):
    seg = title.split("——", 1)[1] if "——" in title else title
    return seg.split("：", 1)[0]

bad = []
for n in sorted(set(bodies) - {r[0] for r in rows}):
    bad.append(f"E{n} 有正文（{bodies[n][0]}）却不在 experiments.md 的索引表里")
for n, st, concl, raw in rows:
    if n not in bodies:
        bad.append(f"E{n} 索引有行，{exp_dir}/ 下没有正文文件")
        continue
    fname, title, text = bodies[n]
    # 第 1 项只看状态列与结论列：简称里碰巧带「作废」二字不算索引说了这件事
    if re.search(r"结论(整体)?作废|结论已作废", title) and "作废" not in (st + concl):
        bad.append(f"E{n} 正文标题写着结论作废，索引行里没有「作废」二字（{fname}）")
    seg = status_seg(title)
    if seg.strip().startswith("未跑") and "未跑" not in st:
        bad.append(f"E{n} 正文状态是「未跑」，索引状态列写「{st}」")
    if "部分已跑" in seg and "部分" not in st:
        bad.append(f"E{n} 正文状态是「部分已跑」，索引状态列写「{st}」")
    if "够判" in seg and "够判" not in st:
        bad.append(f"E{n} 正文状态是「已跑（够判）」，索引状态列写「{st}」")
    c = re.sub(r"git show [0-9a-f]+:\S*", "", unspace(concl))
    for mm in re.finditer(r"\d[\d.]*", c):
        v = mm.group(0).rstrip(".")
        prev = c[max(0, mm.start() - 3):mm.start()]
        if re.search(r"[EDCIK\-]$", prev) or re.match(r"^20\d\d$", v) or len(v) <= 1:
            continue
        # 左边不许紧挨数字或小数点，整数右边不许紧挨数字：按子串比时「10 ms」会在「100 ms」里算找到。
        # 带小数的许正文多几位小数（索引写「10.6」、正文写「10.63」是索引按位截短，不算找不到）
        right_boundary = "" if "." in v else r"(?!\d)"
        if not re.search(r"(?<![\d.])" + re.escape(v) + right_boundary, text):
            bad.append(f"E{n} 索引结论列的「{v}」在 {fname} 正文里找不到")

if bad:
    print(f"  ✗ 实验索引与正文对不上 {len(bad)} 处：")   # gate-lint:summary
    for b in bad:
        print("      " + b)                                # gate-lint:detail
    print("  → 怎么办：以**正文标题**为准改索引行（编号的登记位是正文那行 `## E<n> 简称 —— 状态`，")
    print("    索引表只是导航）；若该改的是正文，就先改正文再回来改索引，并在")
    print("    .claude/kb/experiments-history.md 里按「改前 / 改后 / 依据」记一条。")
    print("    正文有而索引无的那一类：把它登记进 experiments.md 的索引表（编号、状态、一句话结论、正文链接）。")
    sys.exit(1)

if not rows and not bodies:
    print(f"  ! 本次无对象可判：{idx_path} 里一行 `| E<n>` 索引都没有，{exp_dir}/ 下也没有正文")
    sys.exit(77)
print(f"  ✓ 实验索引行与正文标题一致（索引 {len(rows)} 行、正文 {len(bodies)} 份）")
PY
}

# ── 格 results-cited：实验产物有没有写回
# 三道，判完再一起退出；三道各报各的对象数，都没有对象时退 77：
#   ① research/results/ 下（含子目录）每个普通文件（不只 *.out）要在实验索引或正文里被点名，顶层按文件名、子目录按相对路径；
#      不算产物的几类照名字跳过、成功行列名。
#   ② 标着已跑的实验要点名产物，或写明「原始输出未留存」。
#   ③ 这次改动新增或改写的点名，树里或版本库历史里要找得到。
# 样本：results-cited-red 里一份没被点名的产物、一份名字像逐轮中间件而登记了却没点名的产物、一个标着已跑却没点产物的实验、
# 一页未跟踪的新实验页点名了树里与历史里都没有的产物；results-cited-green 里点名齐、一份没登记的逐轮中间件列进没判的清单。
#
# 产物压成 <名>.out.gz 入库（超过单文件推送上限的）之后：第一道，树里只有 <名>.out.gz 时正文点名 <名>.out 或 <名>.out.gz 都算点名了它；
# 第三道，点名的 <名>.out（点 .out.gz 的同样取到 <名>.out）在树里、HEAD 里、历史里只以 <名>.out.gz 存在也算找得到（product_stored_forms 列两种存放）。
# 样本：results-cited-gz-green（产物只以 .out.gz 存在，一页点名 .out、一页点名 .out.gz，一份只在历史里有 .out.gz）、results-cited-gz-red（点名的 .out 树里与历史里 .out 与 .out.gz 都没有，另一份 .out.gz 没人点名）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=gz-citation-unmatched：第一道要正文逐字点名 .out.gz、第三道只查原样的名字，两份绿样本判错。
product_stored_forms() {
  printf '%s\n' "$1"
  [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == gz-citation-unmatched ]] || printf '%s\n' "$1.gz"
}
cell_results_cited() {
  # 索引与各实验正文都要扫：产物可能被任一实验正文点名。
  EXP_DIR=.claude/kb/experiments
  EXP_ALL="$(mktemp)"; UNCITED="$(mktemp)"; NAMED="$(mktemp)"
  # 中间文件全交给 trap 删：判红那一支直接 exit，逐个手删会漏。
  trap 'rm -f "$EXP_ALL" "$UNCITED" "$NAMED"' EXIT
  cat .claude/kb/experiments.md "$EXP_DIR"/*.md > "$EXP_ALL" 2>/dev/null
  EXP="$EXP_ALL"
  RES=research/results
  [[ -s "$EXP" ]] || { echo "  ! 找不到实验正文，这一格跳过"; exit 77; }
  [[ -d "$RES" ]] || { echo "  ! 没有 $RES 目录，这一格无对象可判"; exit 77; }

  # 三道都判完再退出，一次把问题说全；判的工具自己没跑成（awk 出错）才当场退出。
  failed=0
  missing=()
  # replay.sh 登记的入库产物（复跑行经 lib-lag-baseline.py 的 replay-rows 取，与 verdict-false-named、quoted-result-lines、replay-comparison-units 三格同一份读法，
  # 行形状与 research-build-and-replay.sh 的 experiment-replay 格相同；这里取第 4 列的文件名）：名字像逐轮中间件（*.r<一位数>.out）的，登记了照样要被点名；
  # 读不了 replay.sh 时这种名字的产物全会被当成中间件跳过，所以有这种产物就判红。
  REPLAY_TABLE=scripts/experiment-apparatus/replay.sh
  replay_table_readable=1
  REGISTERED=""
  if [[ -r "$REPLAY_TABLE" && -f "$REPLAY_TABLE" ]]; then
    REGISTERED="$(python3 "$LIB_LAG_BASELINE" replay-rows "$REPLAY_TABLE" | cut -f4 | sed 's#.*/##')" || replay_table_readable=0
  else
    replay_table_readable=0
  fi
  judged_products=0
  not_products=()
  round_named_skipped=()
  while IFS= read -r f; do
    relative="${f#"$RES"/}"
    b="${f##*/}"
    # 三类不算产物：本地腿的问答、逐轮复跑的中间件（replay.sh 登记了的除外）、确认目录
    case "$relative" in
      *local*|*.round*|confirm*) not_products+=("$relative"); continue ;;
      *.r[0-9].out)
        case "$(printf "\n%s\n" "$REGISTERED")" in
          *"$(printf "\n%s\n" "$b")"*) : ;;
          *) not_products+=("$relative"); round_named_skipped+=("$relative"); continue ;;
        esac ;;
    esac
    judged_products=$((judged_products + 1))
    # 压成 <名>.out.gz 入库的产物：正文点名 <名>.out 或 <名>.out.gz 都算点名了它（点名的名字取产物压缩之前的那个，grep -F 找子串，所以点名 .out.gz 的同样命中）
    cited_name="$relative"
    if [[ "$relative" == *.out.gz && "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" != gz-citation-unmatched ]]; then cited_name="${relative%.gz}"; fi
    grep -qF -- "$cited_name" "$EXP" || missing+=("$relative")
  done < <(find "$RES" -type f | sort)

  if ((${#missing[@]})); then
    echo "  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核："
    for m in "${missing[@]}"; do echo "     $RES/$m"; done
    echo "     → 怎么办：把结论写进 experiments.md 对应实验的正文，"
    echo "               并在口径段点名这份原始输出；确实是废弃产物就删掉它。"
    failed=1
  elif ((judged_products == 0)); then
    echo "  ! 第一道本次无对象可判：$RES 下没有要点名的产物（不算产物的 ${#not_products[@]} 份除外）"
  else
    echo "  ✓ $RES 下的实验产物全部被 experiments.md 点名（查了 ${judged_products} 份）"
  fi
  if ((replay_table_readable == 0 && ${#round_named_skipped[@]})); then
    echo "  ✗ 读不到 $REPLAY_TABLE：它登记的入库产物认不出来，${#round_named_skipped[@]} 份名字像逐轮中间件的产物全被当成中间件跳过了"
    echo "     → 怎么办：replay.sh 搬了家或改了名，就把这里的 REPLAY_TABLE 一起改（.claude/rules/path-moves.md）；读权限丢了就补上。"
    failed=1
  fi
  if ((${#not_products[@]})); then
    echo "     第一道没判的 ${#not_products[@]} 份（本地腿的问答、没登记的逐轮中间件、确认目录）："
    printf '       %s\n' "${not_products[@]}"
  fi

  # ── 第二道：已跑的实验要么点名产物，要么写明产物没留 ──
  awk '
    /^## E[0-9]+ /{
      if (cur != "" && done && !cited && !excused) print cur
      cur=$2; done=($0 ~ /已跑|已测/); cited=0; excused=0
      if (done) ran_count++
      next
    }
    # 目录名后面要跟着一个文件名才算点名：只写「产物放在 research/results/ 下」的那一节什么都没点
    /research\/results\/[A-Za-z0-9_.-]/{ cited=1 }
    # 归档之后引用只剩文件名，所以裸文件名也算点名；它点的那份由第三道去树里与 git 历史里找。
    # 前面要有词边界：少了它，`-stage1.out` 里的 "stag|e1.out" 会被咬成一个产物名
    match($0, /(^|[^a-zA-Z0-9_-])e[0-9]+[a-zA-Z0-9._-]*\.out/) {
      tok=substr($0, RSTART, RLENGTH); sub(/^[^a-zA-Z0-9_-]/, "", tok)
      named[tok]=cur; cited=1
    }
    /原始输出未留存|输出未留存/{ excused=1 }
    END{ if (cur != "" && done && !cited && !excused) print cur
         for (n in named) print "NAMED" "\t" n "\t" named[n] > "/dev/stderr"
         print "RAN_COUNT" "\t" ran_count + 0 > "/dev/stderr" }
  ' "$EXP" > "$UNCITED" 2>"$NAMED" || {
    echo "  ✗ 判「已跑实验有没有点名产物」的 awk 没跑成（退出码 $?），这一格没判："
    sed 's/^/     /' "$NAMED"   # gate-lint:detail
    echo "     → 怎么办：看上面 awk 的报错修这一段；它没跑完时未点名的实验一个都不会被列出来，不许当成通过。"
    exit 1
  }
  if [[ -s "$UNCITED" ]]; then
    echo "  ✗ 这些实验标着已跑，却既没点名原始输出、也没说明产物为什么没留："
    sed 's/^/     /' "$UNCITED"
    echo "     → 怎么办：存一份产物并在口径段点名；确实留不下（要虚机/真设备）"
    echo "               就写明「原始输出未留存」以及为什么，别让读的人以为能翻到。"
    failed=1
  fi
  rm -f "$UNCITED"
  ran_count="$(sed -n 's/^RAN_COUNT\t//p' "$NAMED")"

  # ── 第三道：点名的产物，树里没有就必须在版本库历史里找得到 ──
  # 只判这次改动新增或改写的点名行（射程与 quoted-result-lines 格同一条）：早先的点名是历史；别的会话还没提交产物的实验页也不由这一次判。
  # 只看 .claude/kb/experiments.md 与 .claude/kb/experiments/ 下的新增行，未跟踪的实验页整份算新增；不在 git 仓里或取不到新增行就判全部。
  # 少了这一道，第二道就退化成「正文里写个像文件名的串」：从没存在过的文件名与归档进历史的产物在正文里长得一样。
  if ((! changed_paths_library_loaded)); then
    echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
    echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
    exit 1
  fi
  BASE="" touched="" scope_note="全部的行（不在 git 仓里，判全部）"
  if ((in_git_work_tree)); then
    BASE="$shared_base"
    if ((added_lines_ok)); then
      touched="$(awk -F'\t' '$1 == ".claude/kb/experiments.md" || $1 == ".claude/kb/experiments" || index($1, ".claude/kb/experiments/") == 1' "$SHARED_ADDED_LINES" \
                 | cut -f2- | grep -oE "e[0-9]+[a-zA-Z0-9._-]*\.out" | sort -u)"
      scope_note="这次改动（基准 $BASE）新增或改写的行"
    else
      BASE="" scope_note="全部的行（取不到基准 $shared_base 起的新增行，判全部）"
    fi
  fi
  missing=0; checked=0; skipped=0
  declare -A skipped_names_by_owner=()
  # $touched 是换行分隔的名字，前面补一个换行（只补一次），点名的名字前面也补一个换行，用 case 比整段，不用管道接 grep -q（pipefail 下退出码会被后一段盖掉）。
  touched_padded=$'\n'"$touched"
  while read -r _ name owner; do   # 字段是 NAMED、文件名、实验号，都不含空格，默认分隔够用
    [[ -n "$name" ]] || continue
    if [[ -n "$BASE" ]]; then
      case "$touched_padded" in
        *$'\n'"$name"*) : ;;
        *) skipped=$((skipped+1))
           skipped_names_by_owner["${owner:-（不在任何实验节里）}"]+=" $name"
           continue ;;
      esac
    fi
    checked=$((checked+1))
    # 树里、HEAD 里、历史里各查一遍，原样的 <名> 与压成 .gz 入库的 <名>.gz 两种存放都算在
    found_stored=0
    while IFS= read -r stored; do
      [[ -f "$RES/$stored" ]] && { found_stored=1; break; }
      # 正在归档的：HEAD 里还在、工作区里删了（这一批照归档规则删上一轮的产物，删除还没提交，下面的 git log 还找不到它）
      git -c core.quotepath=false cat-file -e "HEAD:$RES/$stored" 2>/dev/null && { found_stored=1; break; }
      if [[ -n "$(git -c core.quotepath=false log --all --diff-filter=D --format=%h --name-only -- "*$stored" 2>/dev/null | head -1)" ]]; then
        found_stored=1; break
      fi
    done < <(product_stored_forms "$name")
    ((found_stored)) && continue
    # 实验页里有一行同时写着这个产物名与「原始输出未留存 / 原件未留存」：如实交代过没留下，不算指空
    naming_lines="$(grep -rhF -- "$name" .claude/kb/experiments.md .claude/kb/experiments/ 2>/dev/null || true)"
    if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" != unretained-note-ignored ]] && grep -qE "原始输出未留存|原件未留存" <<<"$naming_lines"; then
      continue
    fi
    echo "     $owner 点名 $name —— 树里没有，git 历史里也没有"   # gate-lint:detail
    missing=$((missing+1))
  done < <(grep '^NAMED' "$NAMED" | sort -u)
  rm -f "$NAMED"
  if [[ $missing -gt 0 ]]; then
    echo "  ✗ $missing 份被点名的产物既不在 $RES 下、也不在版本库历史里"   # gate-lint:summary
    echo "     → 怎么办：产物归档了就该能从历史取回（git log --all --diff-filter=D --name-only 找删它的提交，"
    echo "               再 git show <提交>^:<路径> 读回来）；取不回来说明它从没存在过，"
    echo "               把那一节改成「原始输出未留存」并写明为什么，别让读的人以为翻得到。"
    failed=1
  elif ((checked == 0)); then
    echo "  ! 第三道本次无对象可判：$scope_note里没有点名产物的"
  else
    echo "  ✓ $scope_note点名的 $checked 份产物在树里或版本库历史里都找得到"
  fi
  if ((skipped)); then
    echo "     第三道没判的 $skipped 份：不在这次改动新增或改写的行里的点名，按实验列："
    while IFS= read -r skipped_owner; do
      echo "       $skipped_owner：${skipped_names_by_owner[$skipped_owner]# }"
    done < <(printf '%s\n' "${!skipped_names_by_owner[@]}" | sort -V)
  fi
  ((failed)) && exit 1
  if ((judged_products == 0 && ${ran_count:-0} == 0 && checked == 0)); then
    echo "  ! 本次无对象可判：没有要点名的产物、没有标着已跑的实验、这次改动里也没有点名产物的行"
    exit 77
  fi
  if ((${ran_count:-0} == 0)); then
    echo "  ! 第二道本次无对象可判：没有标着已跑的实验"
  else
    echo "  ✓ 已跑的 $((ran_count)) 个实验都点了名或写明了产物未留存"
  fi
}

# ── 格 evidence-in-repo：跑出来的产物与引来的依据都要落在仓里，不许只留在 /tmp
#
# 判据一「实验跑了没留存」：改动范围里还在盘上的装置源码（scripts/experiment-apparatus/e<号>_*.rs、crates/*/src/bin/e<号>_*.rs、crates/*/src/bin/e<号>_*/ 底下的 .rs）
#   或变异表（scripts/mutation/tables/e<号>_*.tsv），research/results/ 下（含子目录）要有一份这个实验号的产物（文件名 e<号> 后面跟非数字或到头），且 mtime 不早于源码。
#   登记了准入的实验（.claude/gate.d/stage-inputs.tsv 有 E<号> 那一行）先按产物头记的实验键判（admission.py 的 stored_product_row_key_status：
#   实验键 = 复跑表那一行的行键去掉期望产物，算法在 admission.py 文件头「复跑表每一行的行键」）：对得上就绿、对不上就红，mtime 不作数；
#   一份带实验键的产物都没有（或今天的实验键拒算）才按 mtime 判。mtime 会被 git checkout / stash pop 刷新，实验键不会。
#   只改了 // 注释的装置源码不判、成功行列名：按 path-moves.md 改注释里的路径碰不到产物。
#   按要求才跑的实验（scripts/experiment-apparatus/on-request-experiments.tsv 登记的）不要求新产物：装置比产物新时，实验页这一轮新加的行里要写
#   「没有正式跑」或「未正式跑」；页里早先那句说的是上一次改装置，不算。
#   复跑表（scripts/experiment-apparatus/replay.sh）里这个实验的行全是 exact、点名的产物在 research/results/ 里在盘的（准入的实验键没判出对不上、不是按要求才跑的），
#   不要求新产物、也不比修改时刻：装置改了而复跑出来逐字节相同时产物不重写、修改时刻不动，输出变了由 research-build-and-replay.sh 的逐字节复跑判红；成功行列「按复跑表判」。
#   复跑表里这个实验有 timing 行的（哪怕另有 exact 行）：装置源码与复跑表的行之间没有机读的对应，这一次的改动可能落在只出 timing 产物的那一段，
#   所以整个实验不走复跑表这一路，比修改时刻的对象是每一条 timing 行点名的那一份产物（按 research/results/ 里的文件名找，只存着 .gz 的也算）：
#   每一份都要在盘、都不比源码旧（拿其中最旧的那一份比，红句也报最旧的那一份）；exact 行点名的产物、重跑另存的别的 e<号> 文件都不替 timing 行的产物作保。
#   复跑表行键归实验号：E<号> 后面可以跟一段 R<号>（E16R5、E155R2 这种重跑行），都归 E<号>；别的写法的行键不归任何实验号。
#   没有这个实验的行、只有 exact 行而点名的产物不在盘的，比这个实验号在盘最新的那一份产物。
# 判据二「证据指向仓外」：.claude/kb/ 下全部 .md 与 research/prompts/ 下这一轮新写（新增或未跟踪）的 .md 里，
#   依据词（citation_form）后面接 /tmp/ 路径就红。
#   两半射程不同：kb 正文只写现状，旧文件照样该改；research/prompts/ 是原样保存的证据（evidence-discipline.md），只判还没冻的新写那一批。
#
# 链接 worktree 里（git-dir 落在 .git/worktrees/ 下；gate-staged.sh 与 --staged 的临时 worktree 就是）mtime 是检出时写的：
# 判据一里只能比 mtime 的那几份（产物在、实验键没判出对不上）不判，逐份列成本次未跑并报进 $GATE_NOT_RUN_FILE；别的照判，
# 所以提交路径上判据二照样判得到。全是要比 mtime 的、判据二又没有 .md 时退 77。
#
# 管不到：产物对不对得上源码（归 research-build-and-replay.sh 的逐字节复跑，它不看变异表的重跑日志）；
#   replay.sh 的登记行指没指到最新那份；句子是说做法还是引依据只按形状认（「草稿放 /tmp/…」不算，「报告原件在 /tmp/…，已转存」照样红）；
#   已提交的 prompts 文件后来加进的 /tmp 依据；装置经 #[path] 引进来的共用模块（住在 crates/*/tests/ 这类判据一那几种路径形态之外的 .rs）改了，
#   这一格认不出它属于哪个实验、不判；行键不是 E<号> 或 E<号>R<号> 的复跑行（E203Y1 这类变体）不归任何实验号，它点名的 timing 产物不进比较。
#
# 样本：evidence-in-repo-red 放一个产物比源码旧的装置、一个一份产物都没有的变异表、kb 里一处「依据：/tmp/…」、
# 一份新写的 prompts 里「报告 `/tmp/…`」、一个登记了准入而产物头记的实验键对不上今天的装置（E910，按 mtime 会判绿；样本里放了一份只有它一个 [[bin]] 的
# scripts/experiment-apparatus/Cargo.toml，实验键算得出），必须判红；
# evidence-in-repo-green 放产物比源码新的装置、不在改动范围里的旧装置、kb 里三种说做法的 /tmp 写法、改过但不是新写的 prompts 里的「依据：/tmp/…」、
# 登记了准入而产物头没记实验键、产物比源码新的装置（E909，按 mtime 判绿）、复跑表行是 exact 而点名的产物在盘、产物比装置旧的装置（E912，按复跑表判绿），必须判绿并报对数。
# evidence-in-repo-red 另放复跑表行是 timing 的装置（E913）与 exact 而点名的产物不在盘的装置（E914），产物比装置旧，都必须判红。产物头记的实验键与今天的相同就判绿（mtime 不作数）
# 那一路样本算不出（实验键要准入模块现算，setup.sh 不认得它在哪），由 python3 scripts/gate/admission.py --selftest 的「给 doc-experiments 的四种状态」那几格判。
# evidence-in-repo-mixed-replay-red 放一个实验（E915）在复跑表里既有 exact 行又有 timing 行、两份产物都比装置旧：exact 行点名的产物在盘也不按复跑表放过，照旧按修改时刻判红。
# 有 timing 行的实验按 timing 行点名的产物比：evidence-in-repo-exact-only-rerun-red（E917 只多放一份更新的 exact 产物，timing 产物还是旧的）、
# evidence-in-repo-unrelated-file-red（E918 只多放一份 e918 起头的笔记）、evidence-in-repo-timing-rows-red（E919 两条 timing 行、一份产物比装置新、另一份不在盘；
# E920 两条 timing 行、一份比装置新、另一份比装置旧），都判红；evidence-in-repo-timing-rows-green（E925 两条 timing 行的产物都比装置新，exact 行点名的产物旧）判绿。
# crates/*/src/bin/ 下的装置：evidence-in-repo-crates-bin-red（E921 单文件装置一份产物都没有，E922 目录形装置的产物比它旧）、
# evidence-in-repo-crates-bin-green（E923 单文件装置的产物比它新，E924 只改了 // 注释，名字不是 e<号>_ 起头的二进制不在对象里）。
# 行键带重跑后缀：evidence-in-repo-suffix-timing-row-red（E926 一条 exact 行、一条行键写成 E926R2 的 timing 行；装置改了，timing 产物旧，
# exact 产物与一份 e926 起头的笔记比装置新），判红、红句点名 timing 产物。
# timing 产物只存着 .gz：evidence-in-repo-timing-gz-green（E927 两条 timing 行，一份存成 .out、一份只存着 .out.gz，都比装置新）判绿；
# evidence-in-repo-timing-gz-red（E928 一条 timing 行，产物只存着 .out.gz、比装置旧）判红、红句点名那份 .out.gz 是最旧的。
# 链接 worktree 的一对（setup.sh 在样本根里建主仓 .origin-repository/，把它的一个链接 worktree 搬到样本根）：
# evidence-in-repo-worktree-red 放 kb 与新写的 prompts 里各一处 /tmp 依据、一个产物比源码旧的装置（E920），必须判红、点名两处、把 E920 列成本次未跑；
# evidence-in-repo-worktree-green 放一个产物比源码旧的装置（E921）与只说做法的 /tmp 写法，必须判绿、把 E921 列成本次未跑。
cell_evidence_in_repo() {
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，这一格跳过"; exit 77; }
  local timestamps_from_checkout=0
  if [[ "$(git rev-parse --git-dir 2>/dev/null)" == */worktrees/* ]]; then
    timestamps_from_checkout=1
    if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == worktree-skips-whole-cell ]]; then
      echo "  ! 弄坏开关 worktree-skips-whole-cell：$ROOT 是链接 worktree，整格退 77，判据二也不判"
      exit 77
    fi
    if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == worktree-trusts-timestamps ]]; then
      echo "  ! 弄坏开关 worktree-trusts-timestamps：$ROOT 是链接 worktree，照信检出时写的修改时刻"
      timestamps_from_checkout=0
    fi
  fi
  if ((! changed_paths_library_loaded)); then
    echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"
    echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"
    exit 1
  fi
  base="$shared_base"
  if ((! changed_paths_ok || ! added_paths_ok)); then
    print_change_range_error
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    exit 1
  fi
  # 按要求才跑的实验要读实验页这一轮新加的行
  if ((! added_lines_ok)); then
    print_change_range_error
    echo "  ✗ 取不到实验页这一轮新加了哪些行（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到时按要求才跑的实验判不了，不是通过。"
    exit 1
  fi
  # 几份清单经文件与进程替换当文件传：当成命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - "$base" "$SHARED_CHANGED_PATHS" "$SHARED_ADDED_PATHS" "$(dirname "$LIB_CHANGED_PATHS")" \
    <(awk -F'\t' '$1 == ".claude/kb/experiments" || index($1, ".claude/kb/experiments/") == 1' "$SHARED_ADDED_LINES") "$timestamps_from_checkout" <<'PY'
import os, re, subprocess, sys
from datetime import datetime
from zoneinfo import ZoneInfo

base = sys.argv[1]
# argv[6] 为 1：在链接 worktree 里，修改时刻是检出时写的
timestamps_from_checkout = sys.argv[6] == "1"
# 实验键的算法只有 scripts/gate/admission.py 一份（门禁与实验共用），不在这里另算。
sys.path.insert(0, sys.argv[4])
try:
    import admission
except Exception as error:
    print(f"  ✗ 读不到准入模块 {sys.argv[4]}/admission.py：{error}")
    print("     → 怎么办：恢复 scripts/gate/admission.py（判「产物跟不跟得上输入」的实验键只有那一份算法），再跑这一格。")
    sys.exit(1)

def read_list(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return sorted({line for line in handle.read().split("\n") if line})

changed_files = read_list(sys.argv[2])
added_files = set(read_list(sys.argv[3]))

results_dir = "research/results"
kb_dir = ".claude/kb"
prompts_dir = "research/prompts"

crates_binary_forms = (
    re.compile(r"^crates/[^/]+/src/bin/e([0-9]+)_[^/]*\.rs$"),
    re.compile(r"^crates/[^/]+/src/bin/e([0-9]+)_[^/]+/.+\.rs$"),
)
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=crates-binaries-unrecognized：不认 crates/*/src/bin/ 下的装置。
source_forms = (
    (re.compile(r"^scripts/experiment-apparatus/e([0-9]+)_[^/]*\.rs$"), "装置源码"),
    *(() if os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") == "crates-binaries-unrecognized"
      else tuple((form, "装置源码") for form in crates_binary_forms)),
    (re.compile(r"^scripts/mutation/tables/e([0-9]+)_[^/]*\.tsv$"), "变异表"),
)
product_form = re.compile(r"^e([0-9]+)(?![0-9])")
on_request_list = "scripts/experiment-apparatus/on-request-experiments.tsv"
on_request_numbers = set()
if os.path.isfile(on_request_list):
    with open(on_request_list, encoding="utf-8") as handle:
        for line in handle:
            cells = line.split("#", 1)[0].split("\t")
            if cells[0].strip():
                on_request_numbers.add(cells[0].strip().lstrip("Ee"))
not_run_note = re.compile(r"没有正式跑|未正式跑")
# 实验页这一轮新加的行（gate_added_lines 的「路径<TAB>行文」）
added_experiment_page_lines = []
with open(sys.argv[5], encoding="utf-8", errors="replace") as handle:
    for added in handle.read().split("\n"):
        if "\t" in added:
            added_experiment_page_lines.append(tuple(added.split("\t", 1)))

def experiment_page_says_not_run(number):
    """按要求才跑的实验，这一轮有没有在实验页新写一句「装置改过、之后没有正式跑」。
    只认这一轮新加的行：页里早先那句说的是上一次改装置（门禁说明里提到这个词的那种句子也在其中），不给这一次作证。"""
    page_prefix = os.path.join(kb_dir, "experiments", f"{number}-")
    return any(page_path.startswith(page_prefix) and not_run_note.search(page_line)
               for page_path, page_line in added_experiment_page_lines)

# ── 判据一：改动范围里的实验装置与变异表，要有一份不比它旧的产物 ───────────
changed_sources = []
for path in changed_files:
    for form, kind in source_forms:
        match = form.match(path)
        if match and os.path.isfile(path):
            changed_sources.append((path, match.group(1), kind))
            break

newest_product_by_number = {}
product_path_by_file_name = {}
if os.path.isdir(results_dir):
    for directory, _subdirectories, file_names in os.walk(results_dir):
        for file_name in file_names:
            match = product_form.match(file_name)
            if not match:
                continue
            product_path = os.path.join(directory, file_name)
            try:
                modified_at = os.path.getmtime(product_path)
            except OSError:
                continue
            number = match.group(1)
            product_path_by_file_name.setdefault(file_name, product_path)
            previous = newest_product_by_number.get(number)
            if previous is None or modified_at > previous[1]:
                newest_product_by_number[number] = (product_path, modified_at)

# 复跑表（scripts/experiment-apparatus/replay.sh）里比对方式是 exact 的行：实验号 → 那一行点名的、在 research/results/ 里在盘的产物。
# 这样的实验装置改了而复跑出来逐字节相同，产物不重写、修改时刻不动；逐字节比对归 research-build-and-replay.sh，输出变了它判红，所以不要求新产物。
# 这个实验在复跑表里另有 timing 行的，整个实验不进这张表（timing 行不进逐字节复跑，装置改在只出 timing 产物的那一段时没人比），
# 它的 timing 行点名的产物名另记在 replay_timing_names_by_number：判据一比修改时刻的对象是这几份。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=replay-exact-by-experiment：有 timing 行的实验只要有一条 exact 行也进这张表。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=replay-exact-ignored：不认 exact 行（timing 行照读）。
replay_exact_product_by_number = {}
replay_timing_names_by_number = {}
# 复跑表行键归哪个实验号：E<号> 后面可以跟一段 R<号>（E16R5、E155R2 是同一个实验的重跑行）；别的写法（E203Y1 这类变体）不归任何实验号。
REPLAY_KEY_NUMBER = re.compile(r"^E([0-9]+)(?:R[0-9]+)?$")


def replay_key_number(row_key):
    """复跑表一行的行键归到的实验号（不带 E）；认不出给 None。
    弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=replay-key-suffix-unattributed：行键去掉 E 整个当号，带重跑后缀的行归不到实验号。"""
    if os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") == "replay-key-suffix-unattributed":
        return row_key[1:]
    attributed = REPLAY_KEY_NUMBER.match(row_key)
    return attributed.group(1) if attributed else None


if os.path.isfile(admission.REPLAY_SCRIPT):
    with open(admission.REPLAY_SCRIPT, encoding="utf-8", errors="replace") as handle:
        for replay_line in handle.read().split("\n"):
            replay_row = admission.REPLAY_ROW_FORM.match(replay_line)
            row_number = replay_key_number(replay_row.group(1)) if replay_row else None
            if row_number is None:
                continue
            if replay_row.group(5) == "timing":
                replay_timing_names_by_number.setdefault(row_number, []).append(replay_row.group(4))
            if (replay_row.group(5) == "exact" and replay_row.group(4) in product_path_by_file_name
                    and os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") != "replay-exact-ignored"):
                replay_exact_product_by_number.setdefault(row_number, product_path_by_file_name[replay_row.group(4)])
    if os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") != "replay-exact-by-experiment":
        for timing_number in replay_timing_names_by_number:
            replay_exact_product_by_number.pop(timing_number, None)

TOKYO = ZoneInfo("Asia/Tokyo")

def moment(seconds):
    """修改时刻按东京的日期报（只报日子，不报钟点）：比新旧用的是秒，这里只是给人看的那一截。"""
    return datetime.fromtimestamp(seconds, TOKYO).strftime("%Y-%m-%d")

def changes_only_comments(path):
    """装置源码与基准比，增删的每一行都是 // 注释或空行：这种改动（多半是按 path-moves.md 改注释里的路径）碰不到产物。"""
    if path in added_files:
        return False
    diff = subprocess.run(["git", "-c", "core.quotepath=false", "diff", "-U0", base, "--", path],
                          capture_output=True, text=True, errors="replace")
    if diff.returncode != 0:
        return False
    changed_lines = [line[1:] for line in diff.stdout.split("\n")
                     if line[:1] in "+-" and not line.startswith(("+++", "---"))]
    return bool(changed_lines) and all(line.strip() == "" or line.lstrip().startswith("//") for line in changed_lines)

unstored_runs = []
comment_only_sources = []
on_request_not_run = []
on_request_missing_note = []
fingerprint_matched = []
fingerprint_absent = []
fingerprint_errors = []
replay_exact_covered = []
timestamp_unjudged = []
for path, number, kind in changed_sources:
    if kind == "装置源码" and changes_only_comments(path):
        comment_only_sources.append(path)
        continue
    source_modified_at = os.path.getmtime(path)
    # 比修改时刻的产物（compared_product）：有 timing 行的实验是每一条 timing 行点名的产物里最旧的那一份（有点名的产物不在盘就是没有，名字记进 timing_missing）；
    # 别的实验是这个实验号在盘最新的那一份。
    # 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=timing-rows-ignored：有 timing 行的实验也比这个实验号在盘最新的那一份；
    # timing-newest-row-only：只比 timing 行点名的产物里最新的那一份，不在盘的名字不算；
    # timing-gz-unrecognized：timing 行点名的产物只认原名，只存着 <名>.gz 的当不在盘。
    timing_missing = []
    timing_names = [] if os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") == "timing-rows-ignored" else replay_timing_names_by_number.get(number, [])
    if timing_names:
        timing_present = []
        for timing_name in timing_names:
            timing_path = product_path_by_file_name.get(timing_name)
            if timing_path is None and os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") != "timing-gz-unrecognized":
                timing_path = product_path_by_file_name.get(timing_name + ".gz")
            try:
                timing_present.append((timing_path, os.path.getmtime(timing_path)))
            except (OSError, TypeError):
                timing_missing.append(timing_name)
        if os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") == "timing-newest-row-only":
            timing_missing = []
            timing_present = [max(timing_present, key=lambda item: item[1])] if timing_present else []
        compared_product = min(timing_present, key=lambda item: item[1]) if timing_present and not timing_missing else None
    else:
        compared_product = newest_product_by_number.get(number)
    try:
        fingerprint_status, fingerprint_detail = admission.stored_product_row_key_status(".", f"E{number}")
    except (admission.InputManifestError, admission.RegistrationError) as error:
        fingerprint_errors.append(f"{path}（{kind}）：E{number} 登记了准入，却算不出实验键：{error}")
        continue
    if fingerprint_status == "matched":
        fingerprint_matched.append(f"E{number}（{path}；{fingerprint_detail}）")
        continue
    if fingerprint_status == "unfingerprinted":
        fingerprint_absent.append(f"E{number}")
    fingerprint_stale = fingerprint_status == "stale"
    if (number not in on_request_numbers and not fingerprint_stale and compared_product is not None and number in replay_exact_product_by_number
            and (timestamps_from_checkout or compared_product[1] < source_modified_at)):
        replay_exact_covered.append(f"E{number}（{path}；复跑表 exact 行点名的 {replay_exact_product_by_number[number]}）")
        continue
    if timestamps_from_checkout and not fingerprint_stale and compared_product is not None:
        # 产物在、实验键没判出对不上，剩下只能比修改时刻，而修改时刻是检出时写的：这一份本次未跑
        timestamp_unjudged.append(f"{path}（{kind}，E{number}）")
        continue
    if number in on_request_numbers and (fingerprint_stale or compared_product is None or compared_product[1] < source_modified_at):
        if experiment_page_says_not_run(number):
            on_request_not_run.append(f"E{number}（{path}）")
        else:
            on_request_missing_note.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：E{number} 按要求才跑，实验页没写明装置改过之后没有正式跑"
                                           "（只认这一轮新加进实验页的那一句）")
        continue
    if fingerprint_stale:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：E{number} 今天的实验键与产物头记的对不上，{fingerprint_detail}")
        continue
    if compared_product is None and timing_missing:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：复跑表里 E{number} 的 timing 行点名的产物 {'、'.join(timing_missing)} 在 {results_dir} 下不在盘")
    elif compared_product is None:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：{results_dir} 下一份 E{number} 的产物都没有")
    elif compared_product[1] < source_modified_at and timing_names:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：复跑表里 E{number} 的 timing 行点名的产物里最旧的 {compared_product[0]} "
                             f"还是 {moment(compared_product[1])} 那份（每一份都要不比装置旧）")
    elif compared_product[1] < source_modified_at:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：最新的 E{number} 产物 {compared_product[0]} 还是 {moment(compared_product[1])} 那份")

# ── 判据二：kb 与这一轮新写的提示里，不许把 /tmp 路径当依据引用 ────────────
citation_form = re.compile(
    r"(依据|证据|出处|产物|报告|原件|结论|存档|留档|留存|记录|落点|详见|参见|见)"
    r"[ \t]{0,2}(?:是|在|为|：|:|=|→)?[ \t]{0,2}[`「]?"
    r"(/tmp/[^\s`\"'()（）「」，。；：、|]*)"
)
tmp_form = re.compile(r"/tmp/[^\s`\"'()（）「」，。；：、|]*")

scanned_files = []
skipped_prompt_files = 0
if os.path.isdir(kb_dir):
    for directory, _subdirectories, file_names in os.walk(kb_dir):
        for file_name in sorted(file_names):
            if file_name.endswith(".md"):
                scanned_files.append(os.path.join(directory, file_name))
if os.path.isdir(prompts_dir):
    for directory, _subdirectories, file_names in os.walk(prompts_dir):
        for file_name in sorted(file_names):
            if not file_name.endswith(".md"):
                continue
            path = os.path.join(directory, file_name)
            if path in added_files:
                scanned_files.append(path)
            else:
                skipped_prompt_files += 1

# 两份变更史里与基准版逐字相同的行是早先写下的历史条目（丢失核对要它们原样留着），按「这次改动」判，不判它们
HISTORY_CARRIERS = {os.path.join(kb_dir, "decisions-history.md"), os.path.join(kb_dir, "experiments-history.md")}
outside_citations = []
tmp_mentions = 0
history_lines_kept = 0
for path in sorted(set(scanned_files)):
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        continue
    # 两条正则（tmp_form、citation_form）都要文本里有 /tmp/：没有它的文件对 tmp_mentions 与引用的贡献都是 0，整份跳过，不逐行跑正则、也不取变更史的基准版。
    # 弄坏开关 tmp-prefilter-skips-all：预筛一律说没有 /tmp/，所有文件都跳过。
    if "/tmp/" not in text or os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") == "tmp-prefilter-skips-all":
        continue
    lines = text.split("\n")
    lines_at_base = set()
    if path in HISTORY_CARRIERS and base and os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") != "history-lines-judged":
        shown = subprocess.run(["git", "-c", "core.quotepath=false", "show", f"{base}:{os.path.relpath(path)}"], capture_output=True, text=True)
        if shown.returncode == 0:
            lines_at_base = set(shown.stdout.split("\n"))
    for line_number, line in enumerate(lines, 1):
        tmp_mentions += len(tmp_form.findall(line))
        if line in lines_at_base and citation_form.search(line):
            history_lines_kept += 1
            continue
        for match in citation_form.finditer(line):
            excerpt = line[max(0, match.start() - 24):match.end()].strip()
            outside_citations.append(f"{path}:{line_number}：…{excerpt}")

if not changed_sources and not scanned_files:
    print(f"  ! 这次改动没碰实验装置与变异表（基准 {base}），{kb_dir} 与 {prompts_dir} 下也没有要扫的 .md，这一格无对象可判")
    sys.exit(77)
if changed_sources and len(timestamp_unjudged) == len(changed_sources) and not scanned_files:
    print(f"  ! 链接 worktree 里这一轮改过的 {len(changed_sources)} 份实验装置与变异表都要比修改时刻（检出时写的，判不了），"
          f"{kb_dir} 与 {prompts_dir} 下也没有要扫的 .md，这一格无对象可判：{'、'.join(timestamp_unjudged)}")
    sys.exit(77)

failed = False
if unstored_runs:
    failed = True
    print("  ✗ 这些实验装置与变异表这一轮改了，research/results/ 里却没有一份不比它旧的产物——这一跑多半只留在 /tmp 的草稿里：")  # gate-lint:summary
    for entry in unstored_runs:
        print(f"     {entry}")  # gate-lint:detail
    print(f"     → 怎么办：跑一次把产物存进 {results_dir}/，并在 scripts/experiment-apparatus/replay.sh 把这个实验的登记行指到它。")
    print("               这一跑已经跑过、产物还在 /tmp 的草稿目录里，就现在拷进来（会话一重启那个目录就没了）；")
    print("               只改了 // 注释的装置这一格不判；改了字符串里的路径，照 .claude/rules/path-moves.md 第 4 条连同留存产物一起改，复跑仍逐字相同。")
    print("               复跑表里有 timing 行的实验，每一条 timing 行点名的产物都要重跑存回（只重跑 exact 那一条、另存别的 e<号> 文件不算）；crates/*/src/bin/ 下的装置同样判。")
    print("               登记了准入的实验（.claude/gate.d/stage-inputs.tsv 有 E<号> 那一行）按产物头记的实验键判：实验键对不上就是产物跟不上今天的输入，")
    print("               重跑一次（装置入口放行并把新的实验键打在产物头），修改时刻新旧不作数。")
if fingerprint_errors:
    failed = True
    print("  ✗ 这些实验登记了准入，却算不出实验键，判不了产物跟不跟得上：")  # gate-lint:summary
    for entry in fingerprint_errors:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：单跑 python3 scripts/gate/admission.py experiment . E<号> 看它报什么；多半是 .claude/gate.d/stage-inputs.tsv 那一行的路径写错，")
    print("               或 cargo -V / rustc -V 跑不出来（bash .claude/scripts/env.sh 报缺什么）。")
if on_request_missing_note:
    failed = True
    print("  ✗ 这些按要求才跑的实验装置改了、没有新产物，实验页也没写明之后没有正式跑：")  # gate-lint:summary
    for entry in on_request_missing_note:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：不用跑（scripts/experiment-apparatus/on-request-experiments.tsv 里的实验只在用户说跑时才跑）；")
    print("               这一轮在实验页新写一句「装置某日改过（改了什么），之后没有正式跑」，要数的时候等用户说跑；页里早先那句说的是上一次改装置，不算。")
if outside_citations:
    failed = True
    print("  ✗ 这些地方把 /tmp 下的东西当依据引用了——那是会话自己的草稿目录，一重启就没了，三个月后没人核得动：")  # gate-lint:summary
    for entry in outside_citations:
        print(f"     {entry}")  # gate-lint:detail
    print(f"     → 怎么办：把 /tmp 下那份东西拷进仓里该在的位置（报告、提示与判决进 {prompts_dir}/，跑出来的数进 {results_dir}/），")
    print("               引用改成指那一份：在引用它的文件里把 /tmp 路径逐处换成仓内路径。")
    print("               原件已经没了，就写明它没了、把还核得动的那部分落进仓里；只是在说做法（草稿放在哪），把句子里的依据词去掉。")
if timestamp_unjudged:
    print(f"    本次未跑 {len(timestamp_unjudged)} 份要比修改时刻的（链接 worktree 里全部文件的时间戳都是检出那一刻写的，分不出装置与产物谁新；"
          f"判据二与判据一里不看时间戳的照判了）：{'、'.join(timestamp_unjudged)}")
    print("    不带 --staged 在工作区里跑 bash .claude/gate.d/doc-experiments.sh --check evidence-in-repo 才判得了这几份。")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    if not_run_file:
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"doc-experiments.sh 的 evidence-in-repo 格有 {len(timestamp_unjudged)} 份装置与变异表本次未跑"
                         f"（链接 worktree 里修改时刻是检出时写的）：{'、'.join(timestamp_unjudged)}\n")
if failed:
    sys.exit(1)

print(f"  ✓ 产物与依据都落在仓里（判了 {len(changed_sources) - len(timestamp_unjudged)} 份这一轮改过的实验装置与变异表、{len(set(scanned_files))} 份 .md，扫到 {tmp_mentions} 处 /tmp 路径、没有一处是引依据；基准 {base}）")
if not changed_sources:
    print("    没判：这次改动没碰 scripts/experiment-apparatus/ 与 scripts/mutation/tables/，判据一这一半没有对象")
print(f"    没判 {skipped_prompt_files} 份 {prompts_dir} 下这一轮没新写的 .md（登记在 .claude/doc-lint-exclude 的原样保存证据，改不得）")
if comment_only_sources:
    print(f"    没判 {len(comment_only_sources)} 份只改了 // 注释的装置源码（碰不到产物，不要求重跑）：{'、'.join(comment_only_sources)}")
if on_request_not_run and os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") != "on-request-unreported":
    print(f"    没要求重跑 {len(on_request_not_run)} 份按要求才跑的实验（装置比产物新，实验页写明了没有正式跑）：{'、'.join(on_request_not_run)}")
if fingerprint_matched:
    print(f"    按实验键判 {len(fingerprint_matched)} 份（登记了准入，产物头记的实验键与今天的相同，修改时刻不作数）：{'、'.join(fingerprint_matched)}")
if replay_exact_covered:
    print(f"    按复跑表判 {len(replay_exact_covered)} 份（复跑表里这个实验的行是 exact、点名的产物在盘；装置比产物新也不要求重写产物，输出变了归 research-build-and-replay.sh 的逐字节复跑判红）：{'、'.join(replay_exact_covered)}")
if fingerprint_absent:
    print(f"    按修改时刻判 {len(fingerprint_absent)} 份登记了准入、research/results/ 里却还没有一份带实验键产物的（或今天的实验键拒算）：{'、'.join(fingerprint_absent)}")
PY
}

# ── 格 decision-links：决策与实验双向登记，实验或结论变了之后回看过决策
#
# 规则在 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。判据：
#   ① 实验页（待回填清单之外的）有「### 影响的决策」一节，表头 | 决策分项 | 关系 | 回看 |；
#      实验正文（历史版本与这一节之外）提到的每条决策（`D<号>（`）都要有一行。
#   ② 表里每一行：决策真实存在，写了分项的那条分项也存在；关系是 支撑 / 推翻 / 备料 / 不影响 之一；
#      回看格以日期开头，后跟「改了」或「不受影响：理由」。
#   ③ 双向：关系是支撑或推翻的那条分项，它的「**依据**」段要引回这个实验；反过来，
#      分项「**依据**」段引的每个实验，那个实验页里要有这一行、关系是支撑或推翻。待回填清单里的决策不判这一条。
#   ④ 时效（全量）：实验最近一次变动的日期——页内历史节带日期的 ### 标题、experiments-history.md 里标题
#      （标题没点名实验时看条目正文）点名它的条目——晚于表里某一行的回看日期，判红。
#   ⑤ 按这次改动（这一道开头取的那一份）：
#      实验页正文（历史版本与影响的决策两节之外）改了、或 research/results/e<号>… 的产物新增或改写了（删掉的不算：归档删上一轮的产物不是结论变了），
#      这次改动要在那张表里改过至少一行；新加的三方判决 research/prompts/*-main-verification.md
#      要有「## 回看决策」一节（同样的表，或一行「不涉及决策：理由」）；这次改动里新写成「改了」的行，
#      它指的决策文件要在这次改动里。这次改动把实验的标题状态改成「已跑」（状态段恰好是这两个字）时，
#      决策正文（历史版本之前）引了它的每条决策（`E<号>（`，粗体、反引号、空格、半角括号的写法也认）都要回看：
#      这次改动在那份决策里新增或改写的行点到了它，或者表里那条决策的一行改过。搬了家的页按实验号找基准那一版的标题。
#   ⑥ 待回填清单 .claude/decision-links-pending：一行一个 E<号> 或 D<号>，# 后写理由；指到的要存在；
#      只减不增——比基准那一版多出来的行判红；基准里还没有这张清单时整张都算新加，git 取不到基准那一版判红
#      （取法 lib-lag-baseline.py 的 baseline_rows，红行句式是它的 BaselineProblem.sentence；样本 decision-links-pending-base-unreadable-red）。
#   认不出标题的实验页（首个二级标题不是「## E<号> 」）与决策文件（不是「## D<号> 」）判红，不静默跳过；
#   ⑤ 取某一页的新增行时 git diff 失败，判红，不当成那一页没改。
#   ⑦ 瘦身形态（待回填清单之外的决策）：首行 `## D<号> 简称 —— 状态` 后面不带括注；分项标题不带日期；
#      每个已定项有「**定案**：」「**射程**：」「**依据**：」「**欠**：」四块（history-kept: 块标签是被判的 kb 页里的原文）；索引表里写结论的那一格不带日期、不超过 100 字
#      （行末「**状态：…**」「**进度：…**」两个规范标记与未定项登记行那两句判定不算字数、不判日期）。
#   ⑧ 有实验才有决策：已定项的依据段至少引一个实验，或写「无实验：理由」（理由至少八个字）；
#      成功行报出写「无实验」的已定项有几个。「无实验：」到行尾是理由，理由里点名的实验（`E<号>（`）不算这条分项引的依据，
#      不进③的双向检查：不要求那个实验页回指这条分项；「无实验：」之前、或依据段别的行里引的实验照算。
#   ⑨ 实验必须对应决策：待回填清单之外的实验页，表里至少一行关系是支撑 / 推翻 / 备料；
#      标题里写着「结论……作废」或「结论……退役」的实验不判（它们留着是为了记下那条路不通）。
#
# 管不到的：回看写的理由对不对、关系判得对不对（支撑写成不影响也过得了③以外的判据），靠人与抽查；
# 实验正文只用编号不用「D<号>（」形态提到的决策，①看不见。
#
# 只删与所属标题相同的行内日期、只把同一天的第二个小节并进第一个的改动，不算「正文改了」（判法借 scripts/kb/doc-consolidate.py，
# 两版整理后的正文按行排序逐字相同就是），样本 decision-links-formatting-green 判绿、成功行报「不算正文改了 1 份」。
# 只改了顶上第三行语言切换行的改动也不算「正文改了」：切换行是三语导航（写法照 .claude/skills/trilingual/SKILL.md「改一组的做法」第 3 条的三种，
# 链接目标是 <名>.md / .en.md / .ja.md），不是实验的结论、数与产物。两版各自去掉这一行之后，历史版本之前、影响的决策一节之外的正文逐字相同就是；
# 第三行认不出这三种形态的照旧算正文。样本 decision-links-language-switch-green（只改切换行的链接目标）判绿、成功行报「只改语言切换行的改动 1 份」；
# decision-links-language-switch-red（切换行与正文一行都改了）照旧判红在「没回看」。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=language-switch-counted 退回把切换行算正文，decision-links-language-switch-green 判红在「没回看」。
# 旧版第三行没有切换行、新版在第三行新插了切换行（加紧跟的一个空行）的，与改了一样不算正文：去掉切换行时连同紧跟它的那一个空行一起去掉，
# 旧版没有切换行时什么都不去，两版这样处理之后逐字相同才算，计进同一个「只改语言切换行的改动」。样本 decision-links-switch-inserted-green
# （旧版没有切换行，新版插进切换行加一个空行）判绿、成功行报「只改语言切换行的改动 1 份」；decision-links-switch-inserted-red（插进切换行，正文一行也改了）
# 照旧判红在「没回看」。弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=inserted-switch-counted 退回只去掉切换行、留着紧跟的空行，
# decision-links-switch-inserted-green 判红在「没回看」。
# 两版正文（切换行、日期整理这两种判法）比之前，基准那一版先照此刻生效的搬迁登记换算（baseline_page_text，调 lib-lag-baseline.py 的 baseline_text：
# 只按路径前缀换、提交引用后面的不换、页自己搬了家按搬迁之前的路径取；边界规则归那份库，这里不另写）：页里一行因登记的搬迁换了路径、
# 同时顶上切换行也改了的，只差这两样，不算正文改了（只差搬迁的整份由 changed-paths.sh 剔出改动范围，到不了这里）。
# 基准里没有这一页、或 git 取不到 / 登记表写坏了：前者按正文改了判，后者另报「取不到基准」一条。
# 样本 decision-links-moved-and-switch-green（一行登记的搬迁，含 `cd scripts/experiments &&` 这种后面不跟斜杠的写法，加切换行）判绿、成功行报「只改语言切换行的改动 1 份」；
# decision-links-moved-and-switch-red（同样两样再加正文一处真改动）照旧判红在「没回看」。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=moves-not-rewritten 去掉这一步换算，decision-links-moved-and-switch-green 判红在「没回看」。
# 样本：decision-links-red 在一个临时仓里把上面每一条各犯一次（want 逐条点名）；decision-links-green 放瘦身的与没瘦身的决策、
# 待回填清单、结论作废的实验、这次改动改了正文与产物又回看了表、新判决带「## 回看决策」、搬了家的实验页，必须判绿并报对数。
# decision-links-no-experiment-green 放一条依据写「无实验：」、理由里点名一个实验（只有备料关系、没有回指这条分项）的已定项，必须判绿；
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=no-experiment-counted 退回把理由里的实验也记成引用，这份样本判红（不对称）。
# decision-links-progress-mark-green 的索引行去掉「**状态：已定。**」「**进度：已实现。**」之后 95 字、带上是 104 字，必须判绿；
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=progress-mark-counted 下把进度标记算进字数，这份样本判红。
# decision-links-progress-mark-red 的索引行去掉两个标记之后 101 字，照样判红在那一行（现在 101 字）。
# decision-links-deleted-product-green 的这次改动只删了一份已提交的产物（research/results/e1-sample.out），实验页与决策表一行没动，必须判绿、
# 报「这次改动触发回看 0 处」；弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=deleted-products-counted 下删除照旧算产物变了，判红在「没回看」。
# 决策正文与实验页只读中文原文（lib-item-ref-status.py 的 originals_only）：译文的首行、分项小节、依据段与「影响的决策」表头是各自的语言。
# decision-links-translation-green 是 decision-links-no-experiment-green 加上 D1、E5 的英语与日语译文（首行写成各自的语言，日语那两份没有 `## D1 ` / `## E5 ` 的空格），
# 必须判绿、报数同那一份；弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=read-translations 连译文一起读，判红在「认不出标题」「没有表」。
# decision-links-pending-base-unreadable-red 的现场与 decision-links-green 相同，setup.sh 末尾删掉基准那一版待回填清单的对象：判红在「读不出基准 HEAD 里的
# .claude/decision-links-pending（git show 退 128」；弄坏开关 LAG_BASELINE_BREAK=unreadable-as-absent 下当成基准里没有这张清单，红在「新加了」，那一条 want 找不到。
cell_decision_links() {
  if [[ ! -d .claude/kb/experiments || ! -d .claude/kb/decisions ]]; then
    echo "  ✗ 找不到 .claude/kb/experiments 或 .claude/kb/decisions"
    echo "  → 怎么办：kb 挪了位置就同步改这一格里的路径。"
    exit 1
  fi
  base="HEAD"
  in_git=0
  changed_list=/dev/null
  if ((in_git_work_tree && head_born)); then
    in_git=1
    if ((! changed_paths_library_loaded)); then
      echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
      echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
      exit 1
    fi
    base="$shared_base"
    if ((! changed_paths_ok)); then
      print_change_range_error
      echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
      echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
      exit 1
    fi
    changed_list="$SHARED_CHANGED_PATHS"
  fi
  # 只删同日日期、只并同日小节的改动不算「正文改了」：判法借 scripts/kb/doc-consolidate.py（门禁 doc-text 的 date-once 格用同一份）；
  # 只改了顶上语言切换行的也不算，判法在下面的 language_switch_only_change
  DOC_CONSOLIDATE="$(dirname "$LIB_CHANGED_PATHS")/../kb/doc-consolidate.py"
  export DOC_CONSOLIDATE
  LIB_ITEM_REF_STATUS="$LIB_ITEM_REF_STATUS" LIB_LAG_BASELINE="$LIB_LAG_BASELINE" python3 - "$ROOT" "$base" "$in_git" "$changed_list" <<'PY'
import glob, importlib.util, os, re, subprocess, sys

root, base, in_git = sys.argv[1], sys.argv[2], sys.argv[3] == '1'
item_ref_status_spec = importlib.util.spec_from_file_location('item_ref_status', os.environ['LIB_ITEM_REF_STATUS'])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)
# 待回填清单与基准那一版比（⑥）：取基准那一版的三种结局经 lib-lag-baseline.py 的 baseline_rows，不在这里另取
lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', os.environ['LIB_LAG_BASELINE'])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
# 决策正文与实验页只读中文原文（lib-item-ref-status.py 的 originals_only）：译文的首行、分项小节、依据段与「影响的决策」表头都是各自的语言，
# 读进来就报「认不出标题」「没有表」，读得出的还靠文件名排序让中文那份最后覆盖。弄坏开关 read-translations 连译文一起读
read_translations = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'read-translations'
changed = {line for line in open(sys.argv[4], encoding='utf-8').read().split('\n') if line}
kb = os.path.join(root, '.claude', 'kb')
RELATIONS = ('支撑', '推翻', '备料', '不影响')
BASIS_RELATIONS = ('支撑', '推翻')
if os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'no-experiment-counted':
    print('  ! 弄坏开关 no-experiment-counted：依据段里「无实验：」之后的 E<号>（ 也记成引用')
E_REF = re.compile(r'(?<![A-Za-z0-9])E(\d+)（')
D_REF = re.compile(r'(?<![A-Za-z0-9])D(\d+)（')
# 「决策正文引了它」按 doc-lint 认引用的写法认：粗体、反引号、编号与括号之间有空格、半角括号都算
MENTION_REF = re.compile(r'(?<![A-Za-z0-9._-])[*`]*E(\d+)[*`]*\s*[（(]')
TABLE_HEAD = re.compile(r'^\|\s*决策分项\s*\|\s*关系\s*\|\s*回看\s*\|\s*$')
REVIEW = re.compile(r'^(20\d\d-\d\d-\d\d)\s*(改了|不受影响：\s*\S.{3,})')
DATE_HEAD = re.compile(r'^### (20\d\d-\d\d-\d\d)')


def read(path):
    return open(path, encoding='utf-8').read()


def history_start(lines):
    for index, line in enumerate(lines):
        if re.match(r'^## 历史版本\s*$', line):
            return index
    return len(lines)


def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], cwd=root, capture_output=True, text=True)


def split_cells(line):
    return [cell.strip() for cell in re.sub(r'\\\|', '\x00', line.strip()).strip('|').split('|')]


def parse_decision_ref(cell):
    """「D28（挂载期承诺量） 已定项 2」→ (28, ('已定项', 2))；只写决策 → (28, None)；认不出 → None。"""
    head = re.match(r'^\**D(\d+)（', cell)
    if not head:
        return None
    item = re.search(r'(已定项|未定项)\s*(\d+)\s*\**$', cell)
    return int(head.group(1)), ((item.group(1), int(item.group(2))) if item else None)


def section_range(lines, heading_pattern, stop_level):
    """标题行号与它那一节的结束行号（下一个级别 ≤ stop_level 的标题之前），找不到返回 None。"""
    for index, line in enumerate(lines):
        if re.match(heading_pattern, line):
            for end in range(index + 1, len(lines)):
                level = re.match(r'^(#{1,6}) ', lines[end])
                if level and len(level.group(1)) <= stop_level:
                    return index, end
            return index, len(lines)
    return None


def table_rows(lines, start, end):
    """一节里那张影响的决策表的行：[(行号, 单元格)]；没有表头返回 None。"""
    rows, seen_head = [], False
    for index in range(start + 1, end):
        line = lines[index]
        if TABLE_HEAD.match(line.strip()):
            seen_head = True
            continue
        if seen_head and re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', line.strip()):
            continue
        if seen_head and line.strip().startswith('|'):
            rows.append((index, split_cells(line)))
        elif seen_head:
            break
    return rows if seen_head else None


problems = []   # (类别, 说明)


def bad(kind, text):
    if (kind, text) not in problems:   # 同一份文件被几处调到时（git 失败那一类）只报一次
        problems.append((kind, text))


# ── 决策：分项与各分项的依据段 ──
decisions = {}
for path in item_ref_status.originals_only(sorted(glob.glob(os.path.join(kb, 'decisions', '*.md'))), read_translations):
    lines = read(path).split('\n')
    body = lines[:history_start(lines)]
    head = next((re.match(r'^## D(\d+) ', line) for line in body if re.match(r'^## D(\d+) ', line)), None)
    if not head:
        # 认不出标题的决策文件不许静默跳过：跳过就不计数、不列名，这份决策的分项与依据一条都没判
        bad('认不出标题', f'{os.path.relpath(path, root)}：「## 历史版本」之前没有一行写成「## D<号> 简称 —— 状态」，这份决策一条都没判')
        continue
    number = int(head.group(1))
    items, basis, shapes, index_cells = set(), {}, [], []
    for index, line in enumerate(body):
        item = re.match(r'^(#{3,4}) (已定项|未定项) (\d+)', line)
        if not item:
            continue
        key = (item.group(2), int(item.group(3)))
        items.add(key)
        cited, in_basis, labels, basis_text = set(), False, set(), ''
        for inner in body[index + 1:]:
            level = re.match(r'^(#{1,6}) ', inner)
            if level and len(level.group(1)) <= len(item.group(1)):
                break
            label = re.match(r'^\*\*(定案|射程|依据|欠)\*\*：', inner)
            if label:
                labels.add(label.group(1))
            if inner.startswith('**依据**'):
                in_basis = True
            elif re.match(r'^\*\*[^*]{1,12}\*\*：', inner) or level:
                in_basis = False
            if in_basis:
                # 「无实验：」到行尾是不引实验的理由，理由里点名的实验不是这条分项引的依据，不进 cited（C535）
                scanned = inner
                cut = inner.find('无实验：')
                if cut >= 0 and os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') != 'no-experiment-counted':
                    scanned = inner[:cut]
                cited |= {int(e) for e in E_REF.findall(scanned)}
                basis_text += inner + '\n'
        basis[key] = cited
        shapes.append((key, line, labels, cited, basis_text))
    for section in ('已定项', '未定项'):
        found = section_range(body, r'^### %s\s*$' % section, 3)
        if found:
            for line in body[found[0] + 1:found[1]]:
                if line.startswith('#### '):
                    break
                row = re.match(r'^\|\s*(\d+)\s*\|', line)
                if row:
                    items.add((section, int(row.group(1))))
                    index_cells.append((section, int(row.group(1)), split_cells(line)[-1]))
                # 没瘦身的决策有的用编号列表列分项（`9. **名字**…`），doc-decisions.sh 数分项时也认这种写法
                listed = re.match(r'^(\d+)\.\s', line)
                if listed:
                    items.add((section, int(listed.group(1))))
    title = next(line for line in body if re.match(r'^## D(\d+) ', line))
    decisions[number] = {'path': os.path.relpath(path, root), 'items': items, 'basis': basis,
                         'shapes': shapes, 'index_cells': index_cells, 'title': title,
                         'mentions': {int(e) for e in MENTION_REF.findall('\n'.join(body))}}

# ── 待回填清单 ──
pending_path = os.path.join(root, '.claude', 'decision-links-pending')
pending_rel = '.claude/decision-links-pending'
pending = set()
pending_lines = read(pending_path).split('\n') if os.path.exists(pending_path) else []
for number, line in enumerate(pending_lines, 1):
    if not line.strip() or line.lstrip().startswith('#'):
        continue
    entry = re.match(r'^(E|D)(\d+)\s+#\s*(\S.{3,})$', line.strip())
    if not entry:
        bad('待回填', f'{pending_rel}:{number}：写成「E<号>  # 理由」或「D<号>  # 理由」，理由至少四个字——「{line.strip()[:30]}」')
        continue
    pending.add((entry.group(1), int(entry.group(2))))
if in_git and os.path.exists(pending_path):
    # 「基准里还没有这张清单」与「git 取不到基准」要分开：前者整张都算新加，后者判不了只减不增，两种都不许静默放过（lib-lag-baseline.py 的三种结局）
    previous_rows, baseline_problem = lag_baseline.baseline_rows(base, pending_rel, cwd=root)
    before, base_note = None, ''
    if baseline_problem is not None:
        bad('待回填', f'{baseline_problem.sentence()}，「只减不增」这一条没判')
    elif previous_rows is None:
        before, base_note = set(), f'（基准 {base} 里还没有这张清单，这次改动新建了它，整张都算新加）'
    else:
        before = set()
        for _number, fields in previous_rows:
            entry = re.match(r'^(E|D)(\d+)\s', '\t'.join(fields).strip())
            if entry:
                before.add((entry.group(1), int(entry.group(2))))
    if before is not None:
        for kind, number in sorted(pending - before):
            bad('待回填', f'{pending_rel} 新加了 {kind}{number}：清单只减不增，新写的实验页与决策当场登记，不进清单{base_note}')

# ── 瘦身形态与「有实验才有决策」（待回填清单之外的决策） ──
DATE = re.compile(r'20\d\d-\d\d-\d\d')
slim_items = no_experiment_items = 0
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    where = info['path']
    # 半定 / 待定的决策，doc-decisions.sh 要标题里写明未定几项（`—— 半定（一项未定）`），只许这一种括注
    if not re.match(r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二两三四五六七八九十]+\s*[项条]未定）)?\s*$', info['title']):
        bad('瘦身形态', f'{where}：首行写成「## D{decision} 简称 —— 状态」，状态后面除了半定 / 待定要写的「（N 项未定）」不带别的括注——「{info["title"][:40]}」')
    for (kind, item_number), heading, labels, cited, basis_text in info['shapes']:
        if DATE.search(heading):
            bad('瘦身形态', f'{where}：D{decision} {kind} {item_number} 的标题带日期，日期与来历进变更史')
        if kind != '已定项':
            continue
        slim_items += 1
        for label in ('定案', '射程', '依据', '欠'):
            if label not in labels:
                bad('瘦身形态', f'{where}：D{decision} 已定项 {item_number} 缺「**{label}**：」一块')
        if '依据' in labels and not cited:
            if re.search(r'无实验：\s*\S.{7,}', basis_text):
                no_experiment_items += 1
            else:
                bad('没有实验', f'{where}：D{decision} 已定项 {item_number} 的依据一个实验都没引——有实验才有决策；实在没有可量的，写「无实验：理由」')
    for kind, item_number, cell in info['index_cells']:
        # 写结论的那一格里这几样规范标记不算字数、不判日期：「**状态：…**」是 doc-decisions.sh 要的检索标记，「**进度：…**」是行末三选一的进度标记
        # （.claude/rules/format-evolution.md「决策正文只写现状」）；「改新池新建文件的字节：…」「动不动格式：…」是 doc-decisions.sh 的
        # blocking-verdict 格要写在登记行里、必须带日期的两句。弄坏开关 progress-mark-counted 下「**进度：…**」照算字数
        without_marks = cell
        for mark in ('改新池新建文件的字节', '动不动格式'):
            without_marks = re.sub(mark + r'：[^（(]*[（(][^）)]*[）)]', '', without_marks)
        counted_marks = r'\*\*状态：[^*]*\*\*' if os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'progress-mark-counted' else r'\*\*(状态|进度)：[^*]*\*\*'
        plain = re.sub(r'\*\*|`', '', re.sub(counted_marks, '', without_marks)).strip()
        if DATE.search(plain) or len(plain) > 100:
            bad('瘦身形态', f'{where}：D{decision} {kind}索引表第 {item_number} 行的定案格要一句话、不带日期、不超过 100 字（现在 {len(plain)} 字）')

# ── 实验最近一次变动的日期：页内历史节 + experiments-history.md ──
central_dates = {}
central_path = os.path.join(kb, 'experiments-history.md')
if os.path.exists(central_path):
    lines = read(central_path).split('\n')
    start = history_start(lines)
    current_date, heading_refs, body_refs = None, set(), set()

    def flush():
        refs = heading_refs or body_refs
        for experiment in refs:
            if current_date and current_date > central_dates.get(experiment, ''):
                central_dates[experiment] = current_date

    for line in lines[start:]:
        dated = DATE_HEAD.match(line)
        if dated or line.startswith('### ') or line.startswith('## '):
            flush()
            current_date = dated.group(1) if dated else None
            heading_refs = {int(e) for e in E_REF.findall(line)} if dated else set()
            body_refs = set()
            continue
        body_refs |= {int(e) for e in E_REF.findall(line)}
    flush()

# ── 实验页 ──
experiments, rows_total = {}, 0
for path in item_ref_status.originals_only(sorted(glob.glob(os.path.join(kb, 'experiments', '*.md'))), read_translations):
    rel = os.path.relpath(path, root)
    lines = read(path).split('\n')
    head_line = next((line for line in lines if re.match(r'^## E(\d+) ', line)), None)
    if not head_line:
        # 认不出标题的实验页不许静默跳过：跳过就不计数、不列名，它的影响的决策表一行都没判
        bad('认不出标题', f'{rel}：没有一行写成「## E<号> 简称 —— 状态」（编号后面要跟空格），这一页一条都没判')
        continue
    number = int(re.match(r'^## E(\d+) ', head_line).group(1))
    # 只认「结论……作废 / 退役」：标题里别处出现这两个词（「上界 A 作废」「容器退役记录」）的实验结论照样成立，照判
    voided = re.search(r'结论[^，。：]{0,3}(作废|退役)', head_line) is not None
    hist = history_start(lines)
    section = section_range(lines[:hist], r'^### 影响的决策\s*$', 3)
    rows = table_rows(lines, section[0], section[1]) if section else None
    outside = [line for index, line in enumerate(lines[:hist]) if not (section and section[0] <= index < section[1])]
    mentioned = {int(d) for d in D_REF.findall('\n'.join(outside))}
    local_dates = [m.group(1) for m in (DATE_HEAD.match(line) for line in lines[hist:]) if m]
    latest = max(local_dates + ([central_dates[number]] if number in central_dates else []), default='')
    is_pending = ('E', number) in pending
    experiments[number] = {'rel': rel, 'lines': lines, 'hist': hist, 'section': section, 'rows': rows or []}
    if rows is None and not is_pending:
        bad('没有表', f'{rel}：E{number} 没有「### 影响的决策」表（表头 | 决策分项 | 关系 | 回看 |）')
        continue
    covered = set()
    for index, cells in rows or []:
        rows_total += 1
        where = f'{rel}:{index + 1}'
        if len(cells) != 3:
            bad('行形状', f'{where}：影响的决策表一行要三格，这一行 {len(cells)} 格')
            continue
        ref = parse_decision_ref(cells[0])
        if not ref:
            bad('行形状', f'{where}：第一格要写「D<号>（简称）」或「D<号>（简称） 已定项 k」——「{cells[0][:30]}」')
            continue
        decision, item = ref
        covered.add(decision)
        if decision not in decisions:
            bad('指不到', f'{where}：D{decision} 在 decisions/ 下没有正文')
            continue
        if item and item not in decisions[decision]['items']:
            bad('指不到', f'{where}：D{decision} 没有{item[0]} {item[1]}')
        relation = cells[1].strip('* ')
        if relation not in RELATIONS:
            bad('行形状', f'{where}：关系写「{relation[:12]}」，只认 支撑 / 推翻 / 备料 / 不影响')
            continue
        if relation in BASIS_RELATIONS and item is None and decisions[decision]['items']:
            bad('行形状', f'{where}：关系是{relation}就要写到分项（D{decision} 有分项）')
        review = REVIEW.match(cells[2].strip('* '))
        if not review:
            bad('行形状', f'{where}：回看格写「YYYY-MM-DD 改了」或「YYYY-MM-DD 不受影响：理由」——「{cells[2][:30]}」')
            continue
        if latest and review.group(1) < latest:
            bad('回看过期', f'{where}：E{number} 最近一次变动在 {latest}，对 D{decision} 的回看停在 {review.group(1)}')
        if relation in BASIS_RELATIONS and item and ('D', decision) not in pending \
                and number not in decisions[decision]['basis'].get(item, set()):
            bad('不对称', f'{where}：E{number} 说它{relation} D{decision} {item[0]} {item[1]}，而那条分项的「**依据**」段没引 E{number}')
    if not is_pending and not voided and rows is not None \
            and not any(len(cells) == 3 and cells[1].strip('* ') in ('支撑', '推翻', '备料') for _, cells in rows):
        bad('没对应决策', f'{rel}：E{number} 的影响的决策表里没有一行是支撑 / 推翻 / 备料——实验必须对应决策（结论作废或退役的实验在标题里写明，就不判这一条）')
    if not is_pending:
        for decision in sorted(mentioned - covered):
            bad('没登记', f'{rel}：E{number} 正文提到 D{decision}，影响的决策表里没有它那一行')

# ── 决策这一侧：依据引的实验，实验页里要有支撑或推翻那一行 ──
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    for item, cited in sorted(info['basis'].items()):
        for experiment in sorted(cited):
            page = experiments.get(experiment)
            if not page:
                bad('指不到', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，kb 里没有这个实验页")
                continue
            ok = any(parse_decision_ref(cells[0]) == (decision, item) and cells[1].strip('* ') in BASIS_RELATIONS
                     for _, cells in page['rows'] if len(cells) == 3)
            if not ok:
                bad('不对称', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，{page['rel']} 的影响的决策表里没有「支撑 / 推翻」这一行")
for kind, number in sorted(pending):
    exists = number in (experiments if kind == 'E' else decisions)
    if not exists:
        bad('待回填', f'{pending_rel} 里的 {kind}{number} 在 kb 里没有正文')

# ── 按这次改动 ──
triggered = 0
formatting_only = 0
language_switch_only = 0
became_ran_checked = 0


def added_lines(rel):
    """这次改动里 rel 新加的行：[(新文件里的行号, 内容)]；未跟踪的文件整份都算。"""
    if git('ls-files', '--error-unmatch', rel).returncode != 0:
        return [(index, line) for index, line in enumerate(read(os.path.join(root, rel)).split('\n'))]
    shown = git('diff', '--no-renames', '--no-color', '--no-ext-diff', '-U0', base, '--', rel)
    if shown.returncode != 0:
        # git 失败时取不到新增行：当成「一行没改」就会把 ⑤ 整条放过去
        bad('取不到改动', f'{rel}：git diff 相对基准 {base} 失败（退 {shown.returncode}：{shown.stderr.strip()[:80]}），这一页新增了哪些行判不了，⑤ 对它没判')
        return []
    diff = shown.stdout
    result, new_line, in_header = [], 0, True
    for line in diff.split('\n'):
        hunk = re.match(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@', line)
        if hunk:
            in_header = False
            new_line = int(hunk.group(1)) - 1
            count = int(hunk.group(2)) if hunk.group(2) is not None else 1
            if count == 0:
                result.append((new_line, None))   # 纯删除：记它在新文件里的位置
            continue
        # 「+++ 」只在第一个 @@ 之前是文件头：正文里以「++ 」开头的一行新增在 diff 里也长成「+++ 」
        if line.startswith('+') and not in_header:
            result.append((new_line, line[1:]))
            new_line += 1
    return result


def ran_status(title_line):
    """标题的状态段（「——」之后、括注之前）恰好是「已跑」：部分已跑、已跑但作废都不算。"""
    status = re.match(r'^## E\d+ .*?——\s*(.*)$', title_line or '')
    return bool(status) and re.sub(r'（.*$', '', status.group(1)).strip() == '已跑'


def base_title(rel, number=None):
    """基准那一版的标题行；基准里没有这一页返回 None。"""
    shown = git('show', f'{base}:{rel}')
    # 基准里这个路径没有页（这一批搬了家）：按实验号在基准那棵树里找它原来那页的标题，搬家不算「改成已跑」
    if shown.returncode != 0 and number is not None:
        found = git('grep', '-h', '-m1', '-E', f'^## E{number} ', base, '--', '.claude/kb/experiments/')
        hit = next((l[l.find('## E'):] for l in found.stdout.split('\n') if '## E' in l), None)
        return hit if found.returncode == 0 else None
    if shown.returncode != 0:
        return None
    return next((line for line in shown.stdout.split('\n') if re.match(r'^## E\d+ ', line)), None)


def baseline_page_text(rel):
    """基准那一版实验页的全文，照此刻生效的搬迁登记把旧前缀换成新前缀（lib-lag-baseline.py 的 baseline_text：只按路径前缀换，
    提交引用后面的不换，页自己搬了家就按搬迁之前的路径取）；基准里没有这一页交 None，git 取不到或登记表写坏了报一条并交 None（按正文改了判）。
    弄坏开关 moves-not-rewritten 不换算（git show 原样读）。"""
    if os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'moves-not-rewritten':
        shown = git('show', f'{base}:{rel}')
        return shown.stdout if shown.returncode == 0 else None
    text, problem = lag_baseline.baseline_text(base, rel, cwd=root)
    if problem is not None:
        bad('取不到基准', f'{problem.sentence()}，{rel} 是不是只差语言切换行或日期整理判不了，按正文改了判')
    return text


_consolidate = None


def consolidate_module():
    """scripts/kb/doc-consolidate.py：读不到就当没有这一层判法（改动照样算正文改了）。"""
    global _consolidate
    if _consolidate is None:
        path = os.environ.get('DOC_CONSOLIDATE', '')
        if not os.path.isfile(path):
            _consolidate = False
        else:
            spec = importlib.util.spec_from_file_location('doc_consolidate', path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            _consolidate = module
    return _consolidate


def normalized_body_lines(text):
    """整理之后的正文行（去空白与列表、标题、表格的符号）按行排序：两版一样就是只删了同日日期、只并了同日小节。"""
    module = consolidate_module()
    body, _hist = module.split_history(text.split('\n'))
    migrated, _report = module.migrate_document('\n'.join(body), '', drop_history=False)
    return sorted(stripped for stripped in (re.sub(r'[\s#*|>\-]', '', line) for line in migrated.split('\n')) if stripped)


def formatting_only_change(rel):
    """这次对实验页的改动是不是只删了与所属标题相同的日期、只把同一天的第二个小节并进第一个。"""
    if not consolidate_module():
        return False
    before = baseline_page_text(rel)
    if before is None:
        return False
    try:
        return normalized_body_lines(before) == normalized_body_lines(read(os.path.join(root, rel)))
    except Exception:  # noqa: BLE001 —— 判法自己出错时按「正文改了」判，不放过
        return False


# 顶上第三行的语言切换行：.claude/skills/trilingual/SKILL.md「改一组的做法」第 3 条的三种写法（中文、英语、日语），链接目标照它们的后缀认
_SWITCH_TARGET = r'\]\([^()\s]+%s\)'
LANGUAGE_SWITCH = re.compile(
    r'\*\*中文\*\* · \[English' + _SWITCH_TARGET % r'\.en\.md' + r' · \[日本語' + _SWITCH_TARGET % r'\.ja\.md'
    + r'|\[中文' + _SWITCH_TARGET % r'\.md' + r' · \*\*English\*\* · \[日本語' + _SWITCH_TARGET % r'\.ja\.md'
    + r'|\[中文' + _SWITCH_TARGET % r'\.md' + r' · \[English' + _SWITCH_TARGET % r'\.en\.md' + r' · \*\*日本語\*\*')


def body_without_language_switch(lines):
    """历史版本之前、影响的决策一节之外的正文行；顶上第三行是认得出的语言切换行时先去掉它，连同紧跟它的那一个空行
    （旧版没有切换行、新版在第三行插进切换行加一个空行的，两版这样去掉之后逐字相同）；没有切换行的什么都不去，认不出的照旧算正文。
    弄坏开关 language-switch-counted 不去掉（切换行算正文）；inserted-switch-counted 只去掉切换行、留着紧跟的空行
    （只认两版第三行都是切换行、只是改了）。"""
    breaking = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '')
    if breaking != 'language-switch-counted' and len(lines) > 2 and LANGUAGE_SWITCH.fullmatch(lines[2].rstrip()):
        trailing_blank = breaking != 'inserted-switch-counted' and len(lines) > 3 and not lines[3].strip()
        lines = lines[:2] + lines[4 if trailing_blank else 3:]
    hist = history_start(lines)
    section = section_range(lines[:hist], r'^### 影响的决策\s*$', 3)
    return [line for index, line in enumerate(lines[:hist]) if not (section and section[0] <= index < section[1])]


def language_switch_only_change(rel):
    """这次对实验页的改动在正文里是不是只改了、或只新插了语言切换行（两版各自去掉它与紧跟的空行之后正文逐字相同）；基准里取不到这一页的按正文改了判。"""
    before = baseline_page_text(rel)
    if before is None:
        return False
    return body_without_language_switch(before.split('\n')) == body_without_language_switch(read(os.path.join(root, rel)).split('\n'))


def check_changed_rows(rel, row_lines):
    for text in row_lines:
        cells = split_cells(text)
        if len(cells) != 3:
            continue
        ref = parse_decision_ref(cells[0])
        review = REVIEW.match(cells[2].strip('* '))
        if ref and review and review.group(2) == '改了' and ref[0] in decisions \
                and decisions[ref[0]]['path'] not in changed:
            bad('改了没改', f"{rel}：这次写了「改了」D{ref[0]}，而 {decisions[ref[0]]['path']} 不在这次改动里")


if in_git:
    # 「产物变了」只认这次改动里新增或改写的产物：删掉的（树上已经不在）不算——归档按规则删上一轮的产物，
    # 不是实验的结论变了，不要求回看决策表（research/ 整个可删，.claude/rules/directory-layout.md）。
    # 判法：改动范围里的路径在被判的树上还在才算；改名的旧路径不在、新路径在，照新增算。
    count_deleted_products = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'deleted-products-counted'
    result_hits = {}
    for rel in sorted(changed):
        match = re.match(r'^research/results/e(\d+)[^0-9]', rel)
        if match and (count_deleted_products or os.path.lexists(os.path.join(root, rel))):
            result_hits.setdefault(int(match.group(1)), rel)
    for number, page in sorted(experiments.items()):
        rel = page['rel']
        body_changed = False
        added = added_lines(rel) if rel in changed and os.path.exists(os.path.join(root, rel)) else []
        section = page['section']
        for line_number, _ in added:
            inside_table = section and section[0] <= line_number < section[1]
            if line_number < page['hist'] and not inside_table:
                body_changed = True
        if body_changed and language_switch_only_change(rel):
            body_changed = False
            language_switch_only += 1
        if body_changed and formatting_only_change(rel):
            body_changed = False
            formatting_only += 1
        table_touched = [text for line_number, text in added
                         if text is not None and section and section[0] <= line_number < section[1]
                         and text.strip().startswith('|') and not TABLE_HEAD.match(text.strip())
                         and not re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', text.strip())]
        check_changed_rows(rel, table_touched)
        reasons = []
        if body_changed:
            reasons.append('正文改了')
        if number in result_hits:
            reasons.append(f'产物 {result_hits[number]} 变了')
        if reasons:
            triggered += 1
            if not table_touched:
                bad('没回看', f"{rel}：这次改动里 E{number} {'、'.join(reasons)}，影响的决策表一行都没回看")
        # ⑤ 的第二半：这一批把实验改成已跑，正文里引了它的每条决策都要回看——决策文件在这一批里，
        # 或者影响表里那条决策的一行在这一批里改过。只看「表里任一行被改过」放不过「决策在欠账块里等它、表里却没它」那一种。
        title_line = next((line for line in page['lines'] if re.match(r'^## E\d+ ', line)), None)
        if body_changed and ran_status(title_line) and not ran_status(base_title(rel, number)):
            reviewed = {ref[0] for ref in (parse_decision_ref(split_cells(text)[0]) for text in table_touched) if ref}
            for decision, info in sorted(decisions.items()):
                if number not in info['mentions'] or ('D', decision) in pending:
                    continue
                became_ran_checked += 1
                # 决策文件在这一批里还不够：要这一批在那份决策里新增或改写的行点到了这个实验号（改一处别的不算回看）
                touched_mention = info['path'] in changed and any(
                    text and re.search(r'(?<![A-Za-z0-9])E%d(?![0-9])' % number, text) for _, text in added_lines(info['path']))
                if not touched_mention and decision not in reviewed:
                    bad('没回看引用它的决策', f"{rel}：这次改动把 E{number} 改成已跑，{info['path']} 的正文引了它，"
                        f"而这次改动在那份决策里没有一行点到它、影响的决策表里 D{decision} 那一行也没回看")
    for rel in sorted(changed):
        if not re.match(r'^research/prompts/[^/]+-main-verification\.md$', rel) or not os.path.exists(os.path.join(root, rel)):
            continue
        if git('cat-file', '-e', f'{base}:{rel}').returncode == 0:
            continue
        triggered += 1
        lines = read(os.path.join(root, rel)).split('\n')
        section = section_range(lines, r'^## 回看决策\s*$', 2)
        if not section:
            bad('没回看', f'{rel}：新写的三方判决没有「## 回看决策」一节')
            continue
        rows = table_rows(lines, section[0], section[1])
        waived = any(re.match(r'^不涉及决策：\s*\S.{3,}', line.strip()) for line in lines[section[0] + 1:section[1]])
        if not rows and not waived:
            bad('没回看', f'{rel}：「## 回看决策」里既没有表行，也没有一行「不涉及决策：理由」')
            continue
        for index, cells in rows or []:
            ref = parse_decision_ref(cells[0]) if len(cells) == 3 else None
            if not ref or not REVIEW.match(cells[2].strip('* ')):
                bad('行形状', f'{rel}:{index + 1}：回看决策表一行写成 | D<号>（简称） [已定项 k] | 关系 | YYYY-MM-DD 改了 / 不受影响：理由 |')
        check_changed_rows(rel, [lines[index] for index, _ in rows or []])

pending_experiments = sum(1 for kind, _ in pending if kind == 'E')
pending_decisions = sum(1 for kind, _ in pending if kind == 'D')
summary = (f'实验页 {len(experiments)} 个（待回填 {pending_experiments}）、决策 {len(decisions)} 条（待回填 {pending_decisions}）、'
           f'已瘦身决策的已定项 {slim_items} 个（其中写「无实验」的 {no_experiment_items} 个）、'
           f'表行 {rows_total} 行、这次改动触发回看 {triggered} 处、改成已跑而引了它的决策 {became_ran_checked} 条；'
           f'只删同日日期、只并同日小节的改动 {formatting_only} 份不算正文改了；只改语言切换行的改动 {language_switch_only} 份不算正文改了')
if problems:
    kinds = {}
    for kind, _ in problems:
        kinds[kind] = kinds.get(kind, 0) + 1
    print(f"  ✗ 决策与实验的登记有 {len(problems)} 处问题（{'、'.join(f'{k} {v}' for k, v in kinds.items())}）；{summary}：")  # gate-lint:summary
    for kind, text in problems:
        print(f'      [{kind}] {text}')  # gate-lint:detail
    print('  → 怎么办：格式与判据见 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。')
    print('            实验出了新结论、换了产物，就逐行回看它影响的决策，回看格写当天日期与「改了」或「不受影响：理由」；')
    print('            支撑 / 推翻的那条分项在「**依据**」段引回这个实验；待回填清单只能删行，新实验页当场写全这张表。')
    sys.exit(1)
print(f'  ✓ 决策与实验双向登记对得上、回看不过期（查了实验页 {len(experiments)} 个、决策 {len(decisions)} 条；{summary}）')
PY
}
