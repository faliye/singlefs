#!/usr/bin/env bash
# 门禁 doc-decisions 的格函数库：kb-shape、item-ref-status、status-redundancy、cross-decision-status、settled-item-self-open、settled-ref-says-open、stale-open-items、settled-same-file。
# 由 .claude/gate.d/doc-decisions.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ════════════════ 前十四格：决策正文、索引页与登记表 ════════════════

# 决策索引表「状态」列对各决策正文：只有格「kb 形状」用（第 5 段读成模块取 declared_open_count，第 7 段 python3 -c 跑 main），所以不另立库文件。
IFS= read -r -d '' INDEX_VS_BODY_SOURCE <<'PY' || true
"""决策索引表的「状态」列 vs 各决策正文（格「kb 形状」第 7 段用）。

索引表那一格写的是**分项计数**：`已定 6 项 / 未定 0 项`，没有分项的写
`无分项 · 整条已定`。它是各正文「### 已定项」/「### 未定项」两节的投影
（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条：同一个事实只许一处权威记录）。

⚠️ **本段自己去数正文，不经过 `.claude/scripts/gen-decision-items.py`**。
写回索引页的是那个生成器，格「决策分项清单与正文同步」拿它的输出与索引页逐字比对——
**审计与被审计用同一段代码时，那个比对什么也证明不了**（`.claude/rules/fs-design.md`
「记账是事务的副产品」那一格的同一条道理）。生成器自己数错时，那一格两边一起错，
本段才会红。

⚠️ 三态词（已定 / 半定 / 待定）**不在索引列里**，它的权威记录是正文首行
`## D<n> 简称 —— 状态`。所以本段只对没有分项的那种行核一次三态词——
那种行的索引格里确实写着它。

⚠️ 索引页 decisions.md 是中文，只对中文原文 `<号>-<名>.md`：译文 `<号>-<名>.en.md` / `.ja.md` 的分项小节叫
`### Settled Items` 这类别的语言，拿中文索引去对译文会数出「已定 0 项 / 未定 0 项」、每条决策多报两遍。
取哪几份走 lib-item-ref-status.py 的 same_language_files（路径由环境变量 LIB_ITEM_REF_STATUS 给）；译文的标题与分项归第 5 段判。
弄坏开关 KB_NUMBERED_FILE_BREAK=first-match：不分语言，译文一并拿来对中文索引。
"""
import re, sys


def load_item_ref_status():
    """lib-item-ref-status.py（按语言取文件的判法只在那一份）；读不到当场判红退出，不退回不分语言。"""
    import importlib.util, os
    library_path = os.environ.get("LIB_ITEM_REF_STATUS", "")
    try:
        spec = importlib.util.spec_from_file_location("item_ref_status", library_path)
        library = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(library)
    except Exception as error:
        print(f"  ✗ 读不到按语言取文件的判法 lib-item-ref-status.py（LIB_ITEM_REF_STATUS={library_path!r}：{error!r}），这一段没判")
        print("     → 怎么办：起这一段时把 LIB_ITEM_REF_STATUS 设成门禁目录里那份 lib-item-ref-status.py；取哪几份只有那一份判法，别在这里另抄。")
        sys.exit(1)
    return library

# 标题里声明的未定条数：阿拉伯数字或 1–99 的中文数字；第 5 段读同一份。只认一个字的中文数字会把「十一项未定」读成「一项未定」。
TITLE_OPEN_COUNT = re.compile(r'([0-9]+|[一二两三四五六七八九十]+)\s*[项条]未定')
CHINESE_DIGITS = {"一": 1, "二": 2, "两": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}


def parse_count_numeral(text):
    """阿拉伯数字或 1–99 的中文数字换成整数；认不出的写法（「一二」「十十」）返回 None。"""
    if text.isdigit():
        return int(text)
    tens_text, has_ten, ones_text = text.partition("十")
    if not has_ten:
        return CHINESE_DIGITS.get(text)
    if "十" in ones_text:
        return None
    tens = 1 if tens_text == "" else CHINESE_DIGITS.get(tens_text)
    ones = 0 if ones_text == "" else CHINESE_DIGITS.get(ones_text)
    if tens is None or ones is None:
        return None
    return tens * 10 + ones


def declared_open_count(title):
    """标题里声明的未定条数：(条数，认不出的写法为 None, 原文片段)；标题没声明返回 None。"""
    match = TITLE_OPEN_COUNT.search(title)
    if not match:
        return None
    return parse_count_numeral(match.group(1)), match.group(0)


CELL = re.compile(r'^已定\s*(\d+)\s*项\s*/\s*未定\s*(\d+)\s*项$')
NOITEM = re.compile(r'^无分项\s*·\s*整条(已定|半定|待定)$')
PROGRESS_MARK = re.compile(r'进度：\s*\*{0,2}(已实现|未实现|无对象)')


def count_progress(body):
    """数两节索引表里每条分项的进度标记，返回 (已实现, 未实现, 无对象, 缺标记)。行的取法与 count_items 同口径。"""
    done = todo = none = missing = 0
    for head in ("已定项", "未定项"):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, body, flags=re.M | re.S)
        seg = m.group(1) if m else ""
        cut = re.search(r'^#{4}\s', seg, flags=re.M)
        top = seg[:cut.start()] if cut else seg
        rows = re.findall(r'^\|\s*\d+\s*\|[^|]*\|.*$', top, flags=re.M) or re.findall(r'^\d+\.\s.*$', top, flags=re.M)
        for row in rows:
            marks = set(PROGRESS_MARK.findall(row))
            if len(marks) != 1:
                missing += 1
            elif "已实现" in marks:
                done += 1
            elif "未实现" in marks:
                todo += 1
            else:
                none += 1
    return done, todo, none, missing


def progress_cell_of(done, todo, none, has_items):
    if not has_items:
        return "无对象"
    return f"已实现 {done} 项 / 未实现 {todo} 项" + (f" / 无对象 {none} 项" if none else "")


def kind(s):
    for k in ("已定", "半定", "待定"):
        if s.lstrip("*").startswith(k):
            return k
    return "?"


def count_items(body):
    """数两节各自索引表里的顶格编号项，返回 (已定数, 未定数)。

    只看**第一个 `####` 之前**那一段：子标题之下是各分项各自的论证，
    里面另有编号列表与表格，那些不是分项。
    """
    got = []
    for head in ("已定项", "未定项"):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, body, flags=re.M | re.S)
        seg = m.group(1) if m else ""
        cut = re.search(r'^#{4}\s', seg, flags=re.M)
        top = seg[:cut.start()] if cut else seg
        # 表格行要有闭合的第二根竖线，与 gen-decision-items.py 的 harvest、lib-item-ref-status.py 同口径：
        # 口径不一时一行畸形表格一边数得到、一边数不到，两边都不报错。
        got.append(len(re.findall(r'^\|\s*(\d+)\s*\|[^|]*\|', top, flags=re.M))
                   or len(re.findall(r'^\d+\.\s', top, flags=re.M)))
    return got[0], got[1]


def main():
    index_path, *all_bodies = sys.argv[1:]
    # 中文索引只对中文原文（文档串第四条）；没对的译文按语言报数
    item_ref_status = load_item_ref_status()
    bodies = item_ref_status.same_language_files(all_bodies, index_path)
    not_compared = {}
    for p in all_bodies:
        if p not in bodies:
            not_compared.setdefault(item_ref_status.language_of(p), []).append(p)
    idx = open(index_path, encoding="utf-8").read()
    fail = 0
    seen = 0
    for p in bodies:
        body = open(p, encoding="utf-8").read()
        t = body.split("\n", 1)[0]
        # 破折号前的空格可选（D14 的标题就没有）；读不出首行判红，跳过会让少核一条在输出里看不出来。
        m = re.match(r'## (D\d+) .+?\s*——\s*(.+)$', t)
        if not m:
            print(f"  ✗ {p} 首行读不出 `## D<n> 简称 —— 状态`：{t[:40]}")
            print("     → 怎么办：把首行写成 `## D3 空间分配 —— 半定（…）`；")
            print("       读不出来就是这一条没核过，不许当成通过。")
            fail = 1
            continue
        num, bs = m.group(1), m.group(2)
        row = re.search(r'^\| %s（[^|]*） *\| ([^|]+)\|' % num, idx, flags=re.M)
        if not row:
            print(f"  ✗ {num} 在 decisions.md 索引里找不到对应行")
            print("     → 怎么办：给它补一行 `| D<n>（简称） | 状态 | 结论（简报） | 正文 | 进度 |`，")
            print("       状态列与进度列跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync 生成。")
            fail = 1
            continue
        seen += 1
        isx = row.group(1).strip()
        n_settled, n_open = count_items(body.split("\n## 历史版本")[0])

        # 进度列（第 5 列）：本段自己数正文里的进度标记，不经生成器。转义竖线 `\|` 不算分隔。
        line = re.search(r'^\|\s*%s（[^|]*）.*$' % num, idx, flags=re.M).group(0)
        cells = [c.strip() for c in re.split(r'(?<!\\)\|', line.strip().strip("|"))]
        if len(cells) < 5:
            print(f"  ✗ {num} 索引行只有 {len(cells)} 列，缺「进度」列（第 5 列）")
            print("     → 怎么办：索引表写成 `| 决策 | 状态 | 结论（简报） | 正文 | 进度 |`，")
            print("       进度列跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync 生成。")
            fail = 1
        else:
            done, todo, none, missing = count_progress(body.split("\n## 历史版本")[0])
            want_cell = progress_cell_of(done, todo, none, n_settled + n_open > 0)
            if cells[4] != want_cell:
                print(f"  ✗ {num} 进度列写「{cells[4]}」，而正文两节里数出「{want_cell}」"
                      + (f"（另有 {missing} 条分项缺进度标记或写了两个）" if missing else ""))
                print("     → 怎么办：权威记录是各分项索引行末尾的 **进度：已实现 / 未实现 / 无对象。** 标记；")
                print("       改完正文跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync")
                fail = 1

        mc, mn = CELL.match(isx), NOITEM.match(isx)
        if not mc and not mn:
            print(f"  ✗ {num} 状态列写着「{isx}」，不是分项计数的写法")
            print("     → 怎么办：状态列一律写 `已定 N 项 / 未定 M 项`（没有分项的写 `无分项 · 整条已定`）；")
            print("       跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync 从正文重新生成。")
            fail = 1
            continue
        if mn:
            if n_settled or n_open:
                print(f"  ✗ {num} 状态列写「无分项」，而正文里数出 {n_settled + n_open} 条分项")
                print("     → 怎么办：跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync")
                fail = 1
            elif mn.group(1) != kind(bs):
                print(f"  ✗ {num} 状态列写「整条{mn.group(1)}」，正文标题是「{kind(bs)}」")
                print("     → 怎么办：三态词的权威记录是正文首行；改完正文跑 --write 重新生成索引列。")
                fail = 1
            continue
        if (int(mc.group(1)), int(mc.group(2))) != (n_settled, n_open):
            print(f"  ✗ {num} 状态列写「已定 {mc.group(1)} 项 / 未定 {mc.group(2)} 项」，"
                  f"而正文两节里数出「已定 {n_settled} 项 / 未定 {n_open} 项」")
            print("     → 怎么办：权威记录是正文的「### 已定项」/「### 未定项」两节。")
            print("       改完正文跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync")
            fail = 1
            continue
        # 标题声明的未定条数也要与实数相符（标题同样会被检索到）；没声明按 0
        declared = declared_open_count(bs)
        if declared is not None and declared[0] is None:
            print(f"  ✗ {num} 正文标题写「{declared[1]}」，里面的数认不出来")
            print("     → 怎么办：条数写阿拉伯数字，或 1–99 的中文数字（「十一项未定」「两项未定」）。")
            fail = 1
        elif (declared_count := 0 if declared is None else declared[0]) != n_open:
            print(f"  ✗ {num} 正文标题说「{declared_count} 项未定」，而「### 未定项」一节里是 {n_open} 项")
            print("     → 怎么办：改正文标题里的那个数，或把分项挪到对的那一节去。")
            fail = 1

    if not seen:
        print("  ✗ 一条决策的索引行都没核到——本段等于没跑")
        print("     → 怎么办：确认 decisions.md 的索引行形如 `| D1（简称） | … |`，")
        print("       以及 decisions/*.md 首行形如 `## D1 简称 —— 已定`。")
        sys.exit(1)
    if not fail:
        not_compared_note = "；".join(f"{len(paths)} 份 .{language}.md" for language, paths in sorted(not_compared.items()))
        print(f"  ✓ 决策索引表的状态列与进度列与正文相符（{seen} 条决策，分项数与进度数是本段自己数的）"
              + (f"；没拿中文索引去对的译文：{not_compared_note}（标题与分项归第 5 段按语言判）" if not_compared else ""))
    sys.exit(1 if fail else 0)


if __name__ == "__main__":
    main()
PY

# 格「未定项被别处定了」的复核判据：只有那一格用（逐条 python3 -c 跑），所以不另立库文件。
IFS= read -r -d '' OPEN_ITEM_REVIEW_SOURCE <<'PY' || true
"""格「未定项被别处定了」的复核判据：一条未定项对它点名的决策 Dn，最近一次「复核」是什么时候。

复核 = 条目块历史里**最近一次让 Dn 的点名次数变多**的那次提交（条目诞生那次也算）；
工作区里条目块比 HEAD 多点了 Dn 一次，算刚复核过。改别的地方不算。

为什么不是「条目块最后一次改动」：那样任何一次改动都能把红消掉。例如：
D19（块指针的结构与宽度预算） 未定项 6 那一格因 D16 的状态变动被标红，往那一格补一段性能数（与 D16 无关），
红就没了——D16 那两句还成不成立是事后手核的，门禁没逼。

条目块：从分项首行起，到下一条分项（编号列表项或「| k |」表格行）或下一个标题之前。
表格行一行就是一条分项，不延到下一个标题，免得把后面几行分项都算进来。

用法：python3 -c "$OPEN_ITEM_REVIEW_SOURCE" <文件> <工作区里条目首行的行号> <Dn:状态变动时间>…
打印没复核过的那几个 Dn，一行一个。

条目块的历史取不到（git log -L 退非 0，或一个提交都没交出来）退 3、原因打到 stderr：
取不到历史不是「复核过了」，调用方（那一格）按「判据没跑成」判红。
"""
import re
import subprocess
import sys


class BlockHistoryUnavailable(Exception):
    """git log -L 没交出条目块的历史：对象缺了、行范围超出了文件、仓坏了。"""

ITEM_BOUNDARY = re.compile(r'^\s*\d+\.\s|^#{2,6} |^\|\s*\d+\s*\|')


def block_end(lines, start_index):
    for index in range(start_index + 1, len(lines)):
        if ITEM_BOUNDARY.match(lines[index]):
            return index - 1
    return min(start_index + 20, len(lines) - 1)


def mention_count(decision, text):
    return len(re.findall(r'(?<![A-Za-z0-9])' + decision + r'(?!\d)', text))


def head_start_index(head_lines, work_first_line):
    for index, line in enumerate(head_lines):
        if line == work_first_line:
            return index
    # 首行在工作区里改过：按同一个分项号在 HEAD 的「### 未定项」那一节里找
    prefix = re.match(r'^(\|\s*\d+\s*\||\s*\d+\.\s)', work_first_line)
    if not prefix:
        return None
    inside = False
    for index, line in enumerate(head_lines):
        if re.match(r'^### 未定项\s*$', line):
            inside = True
            continue
        if re.match(r'^#{2,4} ', line):
            inside = False
        if inside and line.startswith(prefix.group(1)):
            return index
    return None


def history_of_block(path, first_line_number, last_line_number):
    """条目块的历史（HEAD 里第 first..last 行，从 1 数）：[(提交时间, 提交前的内容, 提交后的内容)]，新的在前。"""
    # --no-color 写死：配了 color.ui = always 时行首多转义码，「@@」「+」「-」都认不出（绿样本 setup.sh 就这样配）。
    result = subprocess.run(['git', 'log', '--no-color', '-L', f'{first_line_number},{last_line_number}:{path}', '--format=%x00%ct'],
                            capture_output=True, text=True)
    if result.returncode != 0:
        # 失败时 stdout 可能是半截（-L 边走边打），不能当历史用
        reason = (result.stderr.strip().splitlines() or ['（git 没说原因）'])[-1]
        raise BlockHistoryUnavailable(f'git log -L {first_line_number},{last_line_number}:{path} 退出码 {result.returncode}：{reason}')
    log = result.stdout
    history = []
    for chunk in log.split('\x00')[1:]:
        lines = chunk.split('\n')
        try:
            committed = int(lines[0].strip())
        except ValueError:
            continue
        before, after, inside_hunk = [], [], False
        for line in lines[1:]:
            if line.startswith('@@'):
                inside_hunk = True
                continue
            if not inside_hunk:
                continue
            if line.startswith('-'):
                before.append(line[1:])
            elif line.startswith('+'):
                after.append(line[1:])
            elif line.startswith(' '):
                before.append(line[1:])
                after.append(line[1:])
        history.append((committed, '\n'.join(before), '\n'.join(after)))
    return history


def main():
    path, work_line_number = sys.argv[1], int(sys.argv[2])
    dependencies = [(entry.split(':')[0], int(entry.split(':')[1])) for entry in sys.argv[3:]]
    head = subprocess.run(['git', 'show', f'HEAD:{path}'], capture_output=True, text=True)
    if head.returncode != 0:
        return   # 文件还没进过 HEAD：比任何决策都新，无从陈旧
    head_lines = head.stdout.split('\n')
    with open(path, encoding='utf-8') as handle:
        work_lines = handle.read().split('\n')
    work_start = work_line_number - 1
    head_start = head_start_index(head_lines, work_lines[work_start])
    if head_start is None:
        return   # HEAD 里没有这条分项 = 新加的，无从陈旧
    head_end = block_end(head_lines, head_start)
    work_end = block_end(work_lines, work_start)
    head_block = '\n'.join(head_lines[head_start:head_end + 1])
    work_block = '\n'.join(work_lines[work_start:work_end + 1])
    try:
        history = history_of_block(path, head_start + 1, head_end + 1)
    except BlockHistoryUnavailable as error:
        print(f'  取不到条目块的历史，这一条没比：{error}', file=sys.stderr)
        sys.exit(3)
    if not history:
        # 条目块在 HEAD 里就一定有引入它的提交：退 0 却一个都没交出来是判不了，不是「复核过了」
        print(f'  git log -L 对 {path} 第 {head_start + 1}–{head_end + 1} 行一个提交都没交出来，这一条没比', file=sys.stderr)
        sys.exit(3)
    for decision, changed_at in dependencies:
        if mention_count(decision, work_block) > mention_count(decision, head_block):
            continue   # 工作区里刚补了一句点名它的复核
        reviewed_at = next((committed for committed, before, after in history
                            if mention_count(decision, after) > mention_count(decision, before)), 0)
        if changed_at > reviewed_at:
            print(decision)


if __name__ == '__main__':
    main()
PY

# ════ 格「kb 形状」════
# 同一件事在 kb 里只许一种写法（.claude/singlefs-ai-sop/rules/kb-discipline.md）；位置、自指、时间指代归上游 doc-lint.sh。
# 样本：kb-shape-red 每一段至少犯一次；第 7 段的两种红互斥，放的是「状态列的分项计数与正文不符」。
grid_kb_shape() (
  KB=.claude/kb
  fail=0
  bad() { printf '  ✗ %s\n' "$*"; fail=1; }
  ok()  { printf '  ✓ %s\n' "$*"; }
  howto() { printf '     → %s\n' "$*"; }

  # 文件装进数组，不靠 $(find …) 的分词：带空格的文件名会被切成两个不存在的路径
  kb_md_files=()
  while IFS= read -r -d '' found_file; do kb_md_files+=("$found_file"); done \
    < <(find "$KB" -name '*.md' -print0 2>/dev/null)
  shopt -s nullglob
  rule_md_files=(.claude/rules/*.md)
  kb_top_md_files=("$KB"/*.md "$KB"/decisions/*.md)
  shopt -u nullglob
  kb_and_rule_md_files=(${kb_md_files[@]+"${kb_md_files[@]}"} ${rule_md_files[@]+"${rule_md_files[@]}"})
  grep_error_file="$(mktemp)"
  trap 'rm -f "${grep_error_file:?}"' EXIT
  # 用法：grep_files <段名> <grep 参数…> -- <文件…>；命中写进 $grep_hits。
  # 退 0 都读到了；退 1 没有要扫的文件；退 2 有文件读不了（读到的照样交命中）。非 0 都已判红，调用方只在退 0 时报绿。
  grep_files() {
    local stage_label="$1" grep_exit_code=0; shift
    local -a grep_options=()
    while [[ $# -gt 0 && "$1" != "--" ]]; do grep_options+=("$1"); shift; done
    shift
    grep_hits=""
    if [[ $# -eq 0 ]]; then
      bad "$stage_label：一份要扫的文件都没有，这一段没判"
      howto "确认门禁是在仓库根上跑的、$KB 目录在；扫到 0 份不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
      return 1
    fi
    grep_hits="$(grep -Hn "${grep_options[@]}" -- "$@" 2>"$grep_error_file")" || grep_exit_code=$?
    if [[ "$grep_exit_code" -ge 2 ]]; then
      bad "$stage_label：有文件读不了（grep 退 $grep_exit_code），那几份没判："; sed 's/^/     /' "$grep_error_file"
      howto "按上面 grep 的原话修：悬空的符号链接删掉或指回真文件，没有读权限的补上读权限；读不了的那几份不能当成没命中。"
      return 2
    fi
    return 0
  }

  echo "── 1. 同一概念只许一个名字 ──"
  grep_status=0
  grep_files "「未答项 / 已答项」" -e '未答项' -e '已答项' -- ${kb_and_rule_md_files[@]+"${kb_and_rule_md_files[@]}"} || grep_status=$?
  if [[ -n "$grep_hits" ]]; then
    bad "kb 里出现「未答项 / 已答项」"; printf '%s\n' "$grep_hits" | sed 's/^/     /'
    howto "统一写「未定项 / 已定项」。records/ 是当时的会话记录，不在本检查范围。"
  elif [[ "$grep_status" -eq 0 ]]; then ok "未定项 / 已定项 用词统一（扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则）"; fi

  grep_status=0
  grep_files "「### 未定」/「### 已定」小节标题" -e '^### 未定$' -e '^### 已定$' -- ${kb_top_md_files[@]+"${kb_top_md_files[@]}"} || grep_status=$?
  if [[ -n "$grep_hits" ]]; then
    bad "小节标题写作「### 未定」/「### 已定」"; printf '%s\n' "$grep_hits" | sed 's/^/     /'
    howto "统一写「### 未定项」/「### 已定项」——按标题检索时两种写法只能命中一种。"
  elif [[ "$grep_status" -eq 0 ]]; then ok "已定项 / 未定项 小节标题统一（扫 $KB 顶层与 decisions/ 下 ${#kb_top_md_files[@]} 份）"; fi

  echo
  echo "── 3. 文件内自指链接 ──"
  # 历史类文件（*-history.md、decisions-history/ 下）整份不判：里面是当时的原文，分不出哪一句换了会成假话；
  # 它们里的自链靠人看（.claude/rules/path-moves.md「改一个全仓术语：正文之外还有五处会红」）。
  self_link_report=$(python3 - "$KB" <<'PY'
import os, re, sys, glob
kb = sys.argv[1]
scanned = 0
for path in sorted(glob.glob(os.path.join(kb, "**", "*.md"), recursive=True)):
    if os.path.basename(path).endswith("-history.md") or "/decisions-history/" in path:
        continue
    scanned += 1
    own = os.path.normpath(path)
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            for link in re.finditer(r'\[[^\]\n]*\]\(([^)\s]+)\)', line):
                target = link.group(1).split("#", 1)[0]
                if not target or "://" in target:
                    continue
                if os.path.normpath(os.path.join(os.path.dirname(path), target)) == own:
                    print(f"HIT {path}:{line_number}: {link.group(0)}")
print(f"SCANNED {scanned}")
PY
  )
  self_link_scanned=$(sed -n 's/^SCANNED //p' <<<"$self_link_report")
  hit=$(sed -n 's/^HIT //p' <<<"$self_link_report")
  if [[ -z "$self_link_scanned" ]]; then
    bad "扫自指链接的 python 没跑完（没报出扫了几份），这一段等于没判"
    howto "单独跑这一段看它报什么错（多半是某份文件不是 UTF-8）；没跑完不许当成没有自指链接。"
  elif [[ "$self_link_scanned" -eq 0 ]]; then
    bad "一份 kb 文件都没扫到（$KB 下的 .md，历史类文件除外）——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的、$KB 目录在；扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md）。"
  elif [[ -n "$hit" ]]; then
    bad "kb 文件里有链接指回它自己：$(grep -c . <<<"$hit") 处（扫 $self_link_scanned 份）"; printf '%s\n' "$hit" | sed 's/^/     /'
    howto "同一文件内直接写编号带简称（例：D8（核心索引结构））或小节标题，不要链回本文件。"
  else ok "没有文件内自指链接（扫 $self_link_scanned 份 kb 文件，历史类文件除外）"; fi

  echo
  echo "── 4. 上游规则的路径写法 ──"
  # 裸 singlefs-ai-sop/rules/… 从仓库根解析不到，副本在 .claude/ 下。行首的裸路径也算（前面没有字符可配）。
  grep_status=0
  grep_files "上游规则的裸路径" -E -e '(^|[^/.])singlefs-ai-sop/rules/' -- ${kb_and_rule_md_files[@]+"${kb_and_rule_md_files[@]}"} || grep_status=$?
  hit=""
  [[ -n "$grep_hits" ]] && hit="$(grep -v '\.claude/singlefs-ai-sop/rules/' <<<"$grep_hits")"
  if [[ -n "$hit" ]]; then
    bad "上游规则写成了裸路径"; printf '%s\n' "$hit" | sed 's/^/     /'
    howto "统一写 .claude/singlefs-ai-sop/rules/<文件>.md ——裸路径从仓库根打不开。"
  elif [[ "$grep_status" -eq 0 ]]; then ok "上游规则路径统一（扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则）"; fi

  echo
  echo "── 5. 决策标题：编号连续、带状态；分项两节严格分开、未定条数与列表相符 ──"
  BREAK_SWITCH="$BREAK_SWITCH" INDEX_VS_BODY_SOURCE="$INDEX_VS_BODY_SOURCE" LIB_ITEM_REF_STATUS="$LIB_ITEM_REF_STATUS" python3 - $KB/decisions/*.md <<'PY'
import os, re, sys, types
# 标题里未定条数的读法与第 7 段同一份（INDEX_VS_BODY_SOURCE 的 declared_open_count）
lib_index_vs_body = types.ModuleType("lib_index_vs_body")
exec(compile(os.environ["INDEX_VS_BODY_SOURCE"], "INDEX_VS_BODY_SOURCE", "exec"), lib_index_vs_body.__dict__)
# 标题带状态、分项两节、未定条数这几条只判中文原文（lib-item-ref-status.py 的 originals_only）：状态词、小节标题与「状态：」标记都是中文写法，
# 译文的首行写成 `— Settled`、`—— 確定` 这类各自的语言，照中文的判据判就整份报红。译文只按语言判编号连号（下面 files_by_language 那一段）。
# 弄坏开关 DOC_DECISIONS_BREAK=read-translations 连译文一起判。
judged_paths = lib_index_vs_body.load_item_ref_status().originals_only(
    sys.argv[1:], os.environ.get("BREAK_SWITCH") == "read-translations")
translations_numbering_only = len(sys.argv[1:]) - len(judged_paths)
body = ""
for p in judged_paths:
    body += open(p, encoding="utf-8").read().split("\n## 历史版本", 1)[0] + "\n"
fail = 0
index_rows_checked = 0
def bad(m, how=""):
    """每一条拒绝都要给下一步——`.claude/singlefs-ai-sop/scripts/gate-lint.sh` 管的是
    上游脚本里的 `bad "..."`，够不着这里的 python，所以这条纪律在本文件里只能自己守。"""
    global fail
    print(f"  ✗ {m}")
    if how:
        print(f"     → {how}")
    fail = 1

# 编号连续：每个文件一个标题、按文件名次序读入，所以直接比。按语言各判各的（中文原文一组、每种译文一组）：
# 三语三份的号并成一个列表是 [1, 1, 1, 2, …]，把译文当原文多数了两遍；分组走 lib-item-ref-status.py 的 files_by_language
# （弄坏开关 KB_NUMBERED_FILE_BREAK=first-match 下并成一组）。
nums = [int(m.group(1)) for m in re.finditer(r'^## D(\d+) ', body, flags=re.M)]
numbering_groups = []
for language, paths in sorted(lib_index_vs_body.load_item_ref_status().files_by_language(sys.argv[1:]).items()):
    group_body = "".join(open(p, encoding="utf-8").read().split("\n## 历史版本", 1)[0] + "\n" for p in paths)
    group_nums = [int(m.group(1)) for m in re.finditer(r'^## D(\d+) ', group_body, flags=re.M)]
    group_label = "中文原文" if language == "" else f"译文 .{language}.md"
    numbering_groups.append(f"{group_label} {len(group_nums)} 条")
    if group_nums and group_nums != list(range(1, len(group_nums) + 1)):
        bad(f"决策编号不连续或不从 D1 起（{group_label}）：{group_nums}",
             "决策编号必须从 D1 起连号；废弃一条也要留占位并在变更史里写明。译文与中文原文同号，缺号先补那一份译文（.claude/skills/trilingual/SKILL.md）。")
if not nums:
    bad("一个 `## D<n>` 标题都没读到——这一段又变成恒绿了，先修检查再谈别的",
        "确认 decisions/*.md 首行是 `## D<n> 简称 —— 状态`；读不到就是这段检查在空转。")

starts = [m.start() for m in re.finditer(r'^## D\d+ ', body, flags=re.M)]
starts.append(len(body))
decision_count = len(starts) - 1
for a, b in zip(starts, starts[1:]):
    blk = body[a:b]
    title = blk.split("\n", 1)[0]
    if not re.search(r'——\s*(已定|半定|待定)', title):
        bad(f"标题缺状态（已定/半定/待定）：{title[:50]}",
            "首行写成 `## D<n> 简称 —— 已定/半定/待定（…）`，索引行要与它一致。")
    # 只看每节第一个 `####` 之前的索引：之后是各分项的论证，里面另有编号列表
    def index_of(head):
        sec = re.search(r'^### %s$(.*?)(?=^#{1,3} |\Z)' % head, blk, flags=re.M | re.S)
        if not sec:
            return None
        t = sec.group(1)
        cut = re.search(r'^#{4}\s', t, flags=re.M)
        return t[:cut.start()] if cut else t
    # ①② 每条分项索引行要带规范标记「状态：已定 / 未定」且与所在节一致：单条取出时出来的是这一行，不是小节标题。
    #    不靠「取行内第一个状态词」猜：合法分项常先提别处的状态。加粗与不加粗都认。
    #    判不了：标记写对而后半句自相矛盾，欠账 C51（分项引用只验状态不验身份）。
    for head in ("已定项", "未定项"):
        seg = index_of(head)
        if not seg:
            continue
        for row in re.findall(r'^(?:\|\s*\d+\s*\||\d+\.\s).*$', seg, flags=re.M):
            index_rows_checked += 1
            marks = re.findall(r'状态：\s*\*{0,2}(已定|未定)', row)
            if not marks:
                bad(f"{title.split()[1]} {head}里有一条分项没写「状态：已定/未定」这个规范标记，"
                    f"状态只挂在小节位置上：{row[:56]}",
                    "在这一行末尾补上 **状态：已定。** 或 **状态：未定。**——检索端出来的是这一行，不是它上面的小节标题。")
            elif len(set(marks)) > 1:
                bad(f"{title.split()[1]} {head}里有一条分项写了两个互相冲突的状态标记："
                    f"{row[:56]}",
                    "一行只许有一个「状态：」标记；要提别处的状态就写成「D<n>（简称） 已定项 k」。")
            elif marks[0] + "项" != head:
                bad(f"{title.split()[1]} {head}里有一条分项的状态标记写着「{marks[0]}」："
                    f"{row[:56]}",
                    "要么把它挪到对应的小节去（编号不变），要么改正状态标记；两节的编号是同一套。")
            # 进度标记与状态标记并排：已实现 / 未实现 / 无对象 三选一，缺了与写了两个不同的值都红（判法在 decisions.md「进度」一段）。
            progress_marks = sorted(set(re.findall(r'进度：\s*\*{0,2}(已实现|未实现|无对象)', row)))
            if not progress_marks:
                bad(f"{title.split()[1]} {head}里有一条分项没写「进度：已实现 / 未实现 / 无对象」这个规范标记："
                    f"{row[:56]}",
                    "在这一行的 **状态：…。** 之后补上 **进度：已实现。**、**进度：未实现。** 或 **进度：无对象。**；判法在 decisions.md「进度」一段。")
            elif len(progress_marks) > 1:
                bad(f"{title.split()[1]} {head}里有一条分项写了两个不同的进度标记（{'、'.join(progress_marks)}）："
                    f"{row[:56]}",
                    "一行只许一个「进度：」标记，删掉多余的那个。")
    # ③ 已定项一侧的正文不许自陈本项没定；措辞只认「说本项自己没定」那几句，不误伤「别处仍未定」。
    SELF_UNDECIDED = r'不是定案|落成未定项|退回未定|本项未定|本项仍未定|本项还没定|该项未定|仍未定案|尚未定案'
    regions = []
    m0 = re.search(r'^### 已定项\s*$(.*?)(?=^### |\Z)', blk, flags=re.M | re.S)
    if m0:
        regions.append(m0.group(1))
    for hm in re.finditer(r'^(#{3,4}) 已定项 \d+[^\n]*$', blk, flags=re.M):
        rest = blk[hm.end():]
        nx = re.search(r'^#{1,%d} ' % len(hm.group(1)), rest, flags=re.M)
        regions.append(rest[:nx.start()] if nx else rest)
    for reg in regions:
        for hit in re.finditer(SELF_UNDECIDED, reg):
            a = max(0, hit.start() - 34)
            bad(f"{title.split()[1]} 已定项一侧的正文自陈还没定："
                f"…{reg[a:hit.end() + 10]}…",
                "正文只写现状：定了就删掉这句陈旧的证据等级；真没定就把这条分项挪回未定项小节。")
    # ④ 两节共用一套编号，合起来要唯一且从 1 连到 n：重号让一个号指两件事，断号像有分项被删
    nums = []
    for head in ("已定项", "未定项"):
        seg = index_of(head)
        if not seg:
            continue
        nums += [int(x) for x in (re.findall(r'^\|\s*(\d+)\s*\|', seg, flags=re.M)
                                  or re.findall(r'^(\d+)\.\s', seg, flags=re.M))]
    if nums:
        dup = sorted({n for n in nums if nums.count(n) > 1})
        if dup:
            bad(f"{title.split()[1]} 分项编号重号：{dup}（一套编号分住两节，重号等于一个号指两件事）",
                "给后加的那条换一个没用过的号，并同步改全仓引用；改完跑 .claude/gate.d/doc-decisions.sh，看格「分项引用状态」")
        elif sorted(nums) != list(range(1, len(nums) + 1)):
            bad(f"{title.split()[1]} 分项编号不是从 1 连到 {len(nums)}：{sorted(nums)}",
                "断号说明有分项被删了却没交代。要么补回那一条，要么在变更史里写明它去哪了。")
    # ⑤ 标题声明的未定条数与「### 未定项」小节的分项数相符
    declared = lib_index_vs_body.declared_open_count(title)
    if declared is None:
        continue
    want, declared_phrase = declared
    if want is None:
        bad(f"{title.split()[1]} 标题写「{declared_phrase}」，里面的数认不出来",
            "条数写阿拉伯数字，或 1–99 的中文数字（「十一项未定」「两项未定」）。")
        continue
    t = index_of("未定项")
    if t is None:
        bad(f"标题声明了「{declared_phrase}」却没有「### 未定项」小节：{title[:40]}",
                "补一个「### 未定项」小节，或把标题里的未定条数改成 0 并去掉那句。")
        continue
    got = len(re.findall(r'^\|\s*\d+\s*\|', t, flags=re.M)) or \
          len(re.findall(r'^\d+\.\s', t, flags=re.M))
    if not got:
        first = [l for l in t.strip().split("\n") if l.strip()]
        got = 1 if first else 0
        if not got:
            bad(f"「### 未定项」小节里既没有编号条目也没有表格：{title[:40]}",
                "分项要写成带编号的表格行或编号列表，生成器与门禁都按这两种形状抽。")
            continue
    if got != want:
        bad(f"{title.split()[1]} 标题写「{declared_phrase}」，「### 未定项」小节里实际 {got} 项",
                "改正文标题里的条数，并同步 decisions.md 索引行；改完跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync")
if not fail:
    translations_note = f"；译文 {translations_numbering_only} 份只按语言判编号连号" if translations_numbering_only else ""
    print(f"  ✓ 决策标题与未定项列表相符，且两节没有互相串味（{decision_count} 条决策、核 {index_rows_checked} 行分项索引{translations_note}）"
          f"；编号连号按语言各判各的：{'、'.join(numbering_groups)}")
sys.exit(1 if fail else 0)
PY
  [[ $? -ne 0 ]] && fail=1

  echo
  echo "── 7. 决策索引表的状态列（分项计数）vs 正文实际分项数 ──"
  # 本段自己数正文、不经生成器（理由在 INDEX_VS_BODY_SOURCE 的文档串）；中文索引只对中文原文，取法在 lib-item-ref-status.py 的 same_language_files
  if LIB_ITEM_REF_STATUS="$LIB_ITEM_REF_STATUS" python3 -c "$INDEX_VS_BODY_SOURCE" "$KB/decisions.md" $KB/decisions/*.md; then :; else fail=1; fi

  echo
  if [[ $fail -eq 0 ]]; then echo "  ✓ kb 形状检查通过（kb 文件 ${#kb_md_files[@]} 份、规则 ${#rule_md_files[@]} 份）"; else echo "  ✗ kb 形状检查未通过"; fi   # gate-lint:summary
  exit $fail
)

# ════ 格「分项引用状态」════
# 每一处「D<n>（简称） 已定项 k / 未定项 k」都在断言那条分项的状态，权威是正文两节的索引表；扫 kb / records / research / crates 全域，含 .rs 注释。
# 判据、出路与成功行在 lib-item-ref-status.py 的 main（格「冻结层归属登记表」与 scripts/kb/relabel-item.py 也用这份库）。
# 变更史 `## D<n>（简称）` 节里裸写的分项归这一节的决策；decisions/ 下没有同名中文原文的译文单独报一类红、不解任何引用（判法同在 lib-item-ref-status.py）。
# 样本：item-ref-status-red 判红；item-ref-status-history-section-red / -green、item-ref-status-orphan-translation-red / -green 判节标题与孤立译文。
grid_item_ref_status() (
  python3 "$LIB_ITEM_REF_STATUS"
)

# ════ 格「状态别说两遍」════
# 分项按状态分住两节，节本身就是状态；再写一遍会各自漂移（.claude/singlefs-ai-sop/rules/kb-discipline.md 第 4 条）。判三样：
#   1. 小节标题不写成「已定项 N —— 已定…」（编号后带括注的也算）
#   2. 同一条目不同时写「—— 已定 / 未定」与「状态：已定 / 未定」，两处值同不同都判
#   3. 条目的「状态：」与所在节一致
#   2、3 只看每节第一个 `####` 之前的索引，与格「kb 形状」第 5 段同一条边界；只认显式的「状态：X」与破折号形态，句中的「D18 已定项 3 已定」是引用。
#   一份都没扫到或一条索引行都没核到退 77。决策正文之外也扫实验正文与 records/，防同样的写法在那里复发。
# 样本：status-redundancy-red 三样各犯一次判红；decision-documents-green 里论证带「3. …状态：未定」的决策判绿。
grid_status_redundancy() (
  DEC=.claude/kb/decisions
  [[ -d "$DEC" ]] || { echo "  ! 找不到 $DEC，这一格跳过"; exit 77; }

  python3 - "$DEC" .claude/kb/experiments records <<'PY'
import re, sys, glob, os
bad = []
files = []
rows_checked = 0
for d in sys.argv[1:]:
    if os.path.isdir(d):
        files += sorted(glob.glob(os.path.join(d, '**', '*.md'), recursive=True))

for f in files:
    body = open(f, encoding='utf-8').read().split('\n## 历史版本')[0]
    name = os.path.basename(f)

    # 1. 标题里重复
    for i, line in enumerate(body.split('\n'), 1):
        # 编号后面许跟一段括注（「已定项 3（2026-09-01）—— 已定」，history-kept: 标题里日期的写法举例），否则带日期的标题整个漏过去
        if re.match(r'^#{3,4} (已定项|未定项) \d+ *(?:[（(][^）)]*[）)])? *—— *\*{0,2}(已定|未定)', line):
            bad.append((name, i, '标题重复', line.strip()[:60]))

    # 2 / 3. 条目层：节到下一个一至三级标题，再在第一个 `####` 处收口（与 index_of、count_items 同一条边界）
    for sec_name, want in (('已定项', '已定'), ('未定项', '未定')):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % sec_name, body, re.M | re.S)
        if not m:
            continue
        section_text = m.group(1)
        first_subheading = re.search(r'^#{4}\s', section_text, re.M)
        index_text = section_text[:first_subheading.start()] if first_subheading else section_text
        base = body[:m.start(1)].count('\n') + 1
        for off, line in enumerate(index_text.split('\n')):
            if not re.match(r'^(?:\d+\.|\|\s*\d+\s*\|)', line):
                continue
            rows_checked += 1
            ln = base + off
            dash = re.search(r'——\s*\*{0,2}(已定|未定)', line)
            stat = re.search(r'状态：\s*\*{0,2}(已定|未定)', line)
            if dash and stat:
                bad.append((name, ln, '条目说两遍', line.strip()[:60]))
            if stat and stat.group(1) != want:
                bad.append((name, ln, '挂错节', line.strip()[:60]))

if bad:
    print('  ✗ 状态被说了两遍，或分项挂错了节：')
    for n, ln, why, txt in bad[:20]:
        print(f'     {n}:{ln}  {why} —— {txt}')
    if len(bad) > 20:
        print(f'     …… 另有 {len(bad)-20} 处')
    print('     → 小节标题写成「### 已定项 N（日期）：结论」，别再写「—— 已定」；')
    print('     → 条目二选一：留「状态：已定」这一处机器可读的标注，把「—— 已定」去掉，')
    print('       日期与结论一个字都不要丢；')
    print('     → 状态与所在节不一致的，把条目搬到对的那一节，别就地改状态词。')
    sys.exit(1)

if not files or rows_checked == 0:
    print(f'  ! 本次无对象可判：扫了 {len(files)} 份、核到 {rows_checked} 条分项索引行（决策正文、实验正文、records/ 下的 .md 里没有分项索引）')
    sys.exit(77)
print(f'  ✓ 状态只说一遍，且分项都在对的节里（扫 {len(files)} 份、核 {rows_checked} 条分项索引行）')
PY
)

# ════ 格「说未定其实已定」════
# kb 正文里「D<n>（简称）」后面写的「未定」，与那条决策状态行比：状态不是「待定」（含「待定（…）」）判红。
# 与格「未定项被别处定了」不是一条：那一格只扫未定项小节、按时间比；这一格扫全部正文、按文本比，写在已定项论证、不变量清单、实验正文里的它才抓得到。
# 三种写法都判（C61）：
#   紧贴：「D<n>（简称） 未定」「D<n> 取值仍然未定」，旧判法，一字没动。
#   隔着名词短语：决策号之后、同一句（到「。；！？」或行尾）里出现「未定」，中间不夹别的编号（D<n>、已定项 / 未定项 k、C<n>、E<n>、I-x.y）：
#     「D2（RAID 条带策略） 的条带宽度可变粒度仍然未定」。
#   引文：落在「」里的过时写法（紧贴的也算）要求引文关闭之后的同一句或下一句有一句现状或过时标注（词表见 current_re：已定、现已、改写、没了……，
#     不含引文里的字），没有判红；「」跨行、跨段（引用块）照段内累计，引文是段末时下一句取下一个非空行。
# 射程（隔名词短语这一支）：只加「同一句内、中间不夹别的编号」一条而不加别的滤，在真仓 kb 全部正文上命中 71 处，真缺陷 1 处、其余 70 处不是缺陷：
#   计数与标题 59 处（「已定 8 项 / 未定 0 项」「半定（一项未定）」「其中未定 0」）、说成未定或当时未定 4 处、实验变更史 3 处、表格竖线 1 处、不是谓语 1 处（「相对次序未定义」）、
#   「」引文旁边已带现状的 2 处；真缺陷是 experiments/43 一处引 D21 的旧正文而旁边没有现状（登进挂账表）。下面的滤照这 70 处定：
#   不判的写法——计数（「已定 8 项 / 未定 0 项」「一项未定」「未定 0」）；表格竖线、「——」破折号、斜杠、「其中」之后（结构而不是陈述）；
#   「说成 / 写成 / 记成 / 当成 / 判成 / 读成 / 标成 / 称成」之后（在描述一处旧的错写）；「当时 / 曾 / 原来 / 此前 / 早先 / 旧」之后（历史陈述；history-kept: 门禁认的历史陈述词表）；
#   「未定项 / 未定分项 / 未定义 / 未定的 / 未定过」不是谓语：只认「未定」（可带「案」）之后紧跟标点、空白、括号、引号、星号、省略号、「但 / 仍」或句末的写法；
#   实验变更史（experiments-history.md，整份写的是当时的状态，与格「引用写已定紧跟说没定」同一份名单）里新加的两种写法（隔名词短语、引文）不判，紧贴的照旧判。
#   代价：D<n> 半定时，「D2 的某分项未定」可能是实话（那个分项真的开着），这一支照旧判红，出路是写成「D<n>（简称） 未定项 k」；
#   中间夹了别的编号的、「若 / 如果」假设句里的过时写法判不出，靠人看。
# 存量：此刻真仓上判红的一处登在 .claude/gate.d/decision-evidence-lag.tsv（格名 cross-decision-status），写法与只缩不涨的规矩在那张表的文件头。
# 样本：cross-decision-status-red 里「D180 未定」而 D180 已定判红；cross-decision-status-noun-phrase-red 里隔着名词短语的判红；
#   cross-decision-status-quoted-red 里没有现状的引文判红；cross-decision-status-relaxed-green 里带现状的引文、计数、说成未定、当时未定、夹别的编号的判绿；
#   decision-documents-green 里被引的那条标着待定（含「待定（两项未定）」）判绿。
# 弄坏开关 DOC_DECISIONS_BREAK=cross-decision-status-tight-only（只认紧贴）/ -quote-skipped（引文一概放过）/ -quote-current-ignored（引文一概判红）/
#   -count-form-judged（计数也判）/ -history-phrases-judged（说成未定、当时未定也判）/ -other-number-ignored（中间夹了别的编号也判）。
grid_cross_decision_status() (
  KB="$PWD/.claude/kb"
  [[ -d "$KB/decisions" ]] || { echo "  ! 没有 $KB/decisions，这一格无对象可判"; exit 77; }

  load_diff_base

  DECISION_EVIDENCE_SOURCE="$DECISION_EVIDENCE_SOURCE" LIB_LAG_BASELINE="$LIB_LAG_BASELINE" python3 - "$KB" "$BREAK_SWITCH" "$diff_base" <<'PY'
import os, re, sys, pathlib, types

lib = types.ModuleType("decision_evidence")
exec(compile(os.environ["DECISION_EVIDENCE_SOURCE"], "DECISION_EVIDENCE_SOURCE", "exec"), lib.__dict__)
kb = pathlib.Path(sys.argv[1])
diff_base = sys.argv[3]

# 1) 每条决策的状态取正文标题行 `## D<n> 简称 —— 状态`：破折号前的空格可选（D14 没有）；解析不了的标题判红，不当成跳过
status, name, unparsed = {}, {}, []
title_re = re.compile(r'^##\s+(D\d+)\s*(.+?)\s*——\s*(.+?)\s*$')
for f in sorted((kb / "decisions").glob("*.md")):
    if re.search(r'\.[a-z]{2}\.md$', f.name):
        continue
    lines = f.read_text(encoding="utf-8").splitlines()
    hit = next((m for m in (title_re.match(l) for l in lines) if m), None)
    if hit:
        status[hit.group(1)], name[hit.group(1)] = hit.group(3), hit.group(2)
    elif any(re.match(r'^##\s+D\d+', l) for l in lines):
        unparsed.append(f.name)
if unparsed:
    for fn in unparsed:
        print(f"  ✗ {fn} 的决策标题解析不了，这一格没有扫过它")
    print("     → 标题要写成 `## D<n> 简称 —— 状态`。解析不了就等于没扫，不许当成通过。")
    sys.exit(1)
if not status:
    print("  ! 没有决策正文，这一格无对象可判"); sys.exit(77)

# 2) 三种写法。紧贴：D<n>（简称） 之后只许空白、逗号、括号、「取值」「状态」与「仍 / 仍然 / 尚 / 都 / 均」，然后是「未定」（不是「未定项」）
ref_re = re.compile(r'D\d+(?:（[^）]*）)?')
tight_re = re.compile(r'^[\s，,、（(]*(?:取值|状态)?[\s：:]*(?:仍然|仍|尚|都|均)?未定(?!项)')
# 隔着名词短语：同一句里决策号之后的「未定」。「未定」要是谓语（可带「案」，后面紧跟标点、空白、括号、引号、星号、省略号、「但 / 仍」或句末）
open_re = re.compile(r'未定案?(?=$|[\s，。；：、）)」』*—…（(`_\\但仍])')
number_class = '0-9一二三四五六七八九十两'
count_before_re = re.compile(rf'[{number_class}]\s*[项条个]$')            # 「一项未定」「三条未定」
count_after_re = re.compile(rf'\s*[{number_class}]')                       # 「未定 0 项」「未定 3」
other_number_re = re.compile(r'D\d+|(?:已定|未定)项\s*\d+|(?<![A-Za-z0-9])(?:[CE]\d+|I-\d+(?:\.\d+)?)')
structure_re = re.compile(r'\||——|/|其中')
described_re = re.compile(r'(?:说|写|记|当|算|判|读|标|称)成\s*$')
past_re = re.compile(r'当时|曾|原来|此前|早先|旧')
# 引文旁边的现状：引文关闭之后的同一句或下一句有这些词之一（不含引文里的字）
current_re = re.compile(r'已定|已经定|现已|现在|现状|如今|后来|改写|改成|已改|作废|过时|陈旧|没了|不再成立|已不成立|当时|曾')
sentence_end = '。；！？'
break_switch = sys.argv[2]
tight_only = break_switch == "cross-decision-status-tight-only"
quote_skipped = break_switch == "cross-decision-status-quote-skipped"
quote_current_ignored = break_switch == "cross-decision-status-quote-current-ignored"
count_form_judged = break_switch == "cross-decision-status-count-form-judged"
history_phrases_judged = break_switch == "cross-decision-status-history-phrases-judged"
other_number_ignored = break_switch == "cross-decision-status-other-number-ignored"


def relaxed_hit(sentence, tallies):
    """隔着名词短语的写法：返回命中的「未定」在 sentence 里的位置，没有返回 None；被滤掉的写法记进 tallies。"""
    for open_match in open_re.finditer(sentence):
        between, after = sentence[:open_match.start()], sentence[open_match.end():]
        if not count_form_judged and (count_before_re.search(between.rstrip()) or count_after_re.match(after)):
            tallies["计数写法"] += 1
            continue
        if not other_number_ignored and other_number_re.search(between):
            tallies["中间夹了别的编号"] += 1
            return None
        if structure_re.search(between):
            tallies["表格竖线、破折号、斜杠、「其中」之后"] += 1
            return None
        if not history_phrases_judged and (described_re.search(between) or past_re.search(between)):
            tallies["说成未定或当时未定"] += 1
            return None
        return open_match
    return None


def current_beside(paragraph_lines, start_line, start_column, depth_here):
    """引文旁边有没有现状：从命中的位置往后取，引文还开着就先跳到它的右引号，再取同一句剩下的和下一句（句子按「。；！？」和换行切）。"""
    text = paragraph_lines[start_line][start_column:]
    following = start_line + 1
    while following < len(paragraph_lines) and len(text) < 2000:
        text += "\n" + paragraph_lines[following]
        following += 1
    if depth_here > 0:
        close = text.find("」")
        if close < 0:
            return False
        text = text[close + 1:]
    text = re.sub(r"「[^」]*」", "", text)
    segments = [s for s in re.split(rf"[{sentence_end}\n]", text) if s.strip()]
    return any(current_re.search(s) for s in segments[:2])


bad, in_hist = [], False
scanned_files, history_files, history_section_lines = 0, [], 0
tallies = {"计数写法": 0, "中间夹了别的编号": 0, "表格竖线、破折号、斜杠、「其中」之后": 0, "说成未定或当时未定": 0}
tight_hits = relaxed_hits = quoted_with_current = as_of_then_skipped = 0
for f in sorted(kb.rglob("*.md")):
    if re.search(r'\.[a-z]{2}\.md$', f.name):
        continue
    rel = f.relative_to(kb.parent.parent)
    # 决策变更史整份是历史，写的是当时的状态
    if f.name == "decisions-history.md" or f.parent.name == "decisions-history":
        history_files.append(str(rel))
        continue
    scanned_files += 1
    in_hist = False
    file_lines = f.read_text(encoding="utf-8").splitlines()
    depth = 0           # 「」的嵌套深度，空行归零（引用块跨行的引文照段内累计）
    paragraph_start = 0
    for i, line in enumerate(file_lines, 1):
        if not line.strip():
            depth, paragraph_start = 0, i
            continue
        depth_start = depth
        depth = max(0, depth + line.count('「') - line.count('」'))
        # 只认整行的「## 历史版本」：前缀匹配会让「### 历史版本对照」把后半份文件整段跳过
        if re.match(r'^##\s*历史版本\s*$', line):
            in_hist = True          # 文末历史节按定义写的是旧状态，不判
        if in_hist:
            history_section_lines += 1
            continue
        for m in ref_re.finditer(line):
            d = re.match(r'D\d+', m.group(0)).group(0)
            if d not in status:
                continue
            # 标题可以写成「待定（两项未定）」：逐字比「待定」会把合法的「D<n> 未定」判红
            if re.match(r'\*{0,2}待定', status[d]):
                continue
            rest = line[m.end():]
            kind, stale_column = None, None
            tm = tight_re.match(rest)
            if tm:
                kind, stale_column = "紧贴", m.end() + tm.end()
            elif not tight_only:
                cut = min([rest.find(c) for c in sentence_end if c in rest] or [len(rest)])
                hit = relaxed_hit(rest[:cut], tallies)
                if hit:
                    kind, stale_column = "隔着名词短语", m.end() + hit.end()
            if kind is None:
                continue
            quoted = max(0, depth_start + line[:stale_column].count('「') - line[:stale_column].count('」')) > 0
            # 实验变更史整份写的是当时的状态（与格「引用写已定紧跟说没定」同一份名单）：紧贴、不在引文里的写法照判，隔名词短语、引文两种写法不判
            if f.name == "experiments-history.md" and (quoted or kind != "紧贴"):
                as_of_then_skipped += 1
                continue
            if quoted:
                # 「」引文里的是被当作历史陈述引用的原话：旁边（同一句或下一句）有一句现状才放过
                if quote_skipped or (not quote_current_ignored and current_beside(file_lines[paragraph_start:], i - 1 - paragraph_start, stale_column, 1)):
                    quoted_with_current += 1
                    continue
                kind += "，在「」引文里而旁边（同一句或下一句）没有现状"
            elif kind == "紧贴":
                tight_hits += 1
            else:
                relaxed_hits += 1
            bad.append((str(rel), i, d, status[d], kind, line.strip()))
            break

# 3) 存量挂账（.claude/gate.d/decision-evidence-lag.tsv 里格名是 cross-decision-status 的行）：一行恰好罩住此刻判红的一处，罩不住判红，表相对基准涨了判红
rows, ledger_problems, ledger_present = lib.read_ledger()
if break_switch == 'evidence-ledger-ignored':
    rows, ledger_problems = [], []
red_entries = [(rel, "-", ln, text, (rel, ln, d, st, kind, text)) for rel, ln, d, st, kind, text in bad]
still_red, ledgered, stale = lib.apply_ledger("cross-decision-status", red_entries, rows)
bad = [entry[4] for entry in still_red]
grown, base_problem = ([], None) if break_switch == "evidence-ledger-growth-unjudged" else lib.ledger_growth("cross-decision-status", rows, diff_base)
table_problems = [] if break_switch == "evidence-ledger-unjudged" else ledger_problems + stale
table_problems += [f"{lib.LEDGER}:{row[0]} 相对基准那一版新增了一行（{row[2]} {row[3]}）：这张表只缩不涨" for row in grown]
table_problems += [base_problem] if base_problem else []
filtered_note = ("隔着名词短语的写法滤掉的：" + "；".join(f"{key} {count} 处" for key, count in tallies.items())
                 + f"；「」引文里旁边带现状放过的 {quoted_with_current} 处；实验变更史里新判法不判的 {as_of_then_skipped} 处；被挂账表罩住的 {ledgered} 处")

if bad or table_problems:
    for rel, ln, d, st, kind, text in bad:
        print(f"  ✗ {rel}:{ln} 说 {d} 未定（{kind}），而 {d}（{name[d]}）的状态行是「{st}」")  # gate-lint:detail
        print(f"     原文：{text[:110]}")
    if bad:
        print("     → 怎么办：读那条决策现在定了什么，把这句改写成它的现状；")
        print("               若指的是它下面某个还开着的分项，写成「D<n>（简称） 未定项 k」；")
        print("               在「」里的旧写法要原样留着，就在引文之后的同一句或下一句写一句现状（「现已定……」「已改写成……」）。")
    if table_problems:
        print(f"  ✗ 挂账表 {lib.LEDGER} 有 {len(table_problems)} 处不对")  # gate-lint:summary
        for problem in table_problems:
            print(f"     {problem}")  # gate-lint:detail
        print("     → 怎么办：这张表只收此刻判红、当场补不上的存量：罩不住的行删掉（对象改好了就是它该退场），格式写坏的照表头改；新判红的句子改写成现状，别往表里加行。")
        print("               读不出、取不到基准那一版的，按括号里 git 的报错修好仓库状态（基准要存在、对象库没坏）再跑：取不到基准不是「基准里没有这张表」。")
    print(f"     {filtered_note}")
    sys.exit(1)
print(f"  ✓ 没有把已定的决策说成未定（扫 {scanned_files} 份 kb 文件，比对 {len(status)} 条决策的状态行）")
print(f"     没查的：决策变更史 {len(history_files)} 份（整份是历史）；各文件「## 历史版本」一节里 {history_section_lines} 行")
print(f"     {filtered_note}")
for history_file in history_files:
    print(f"       变更史：{history_file}")
PY
)

# ════ 格「已定分项自称未定」════
# 一条决策的已定项区段（「### 已定项」索引与各「#### 已定项 N」论证）里出现「X 仍未定 / 还没定 / 尚未定」，看 X：
#   a. 自指（取值 / 本项 / 该项 / 本分项 / 本条；打头的「的」去掉再比）：标着已定却说自己没定，判红；
#   b. 同一决策已定项区段里另一条带编号、写着「已定」的行里有 X（兄弟分项已经定了它）：判红。
#   同一行逐处看，判出一处为止。没有一份决策有已定项区段退 77。
# 格「kb 形状」「分项引用状态」只比状态标记，看不见正文；这一格抓已定之后没清理的推导过程。
# 不判「本笔仍然未定」这类子问题：那是 kb-discipline 第 3 条要的显式记录不知道，所以只认自指与兄弟分项，宁可漏不可假红。
# 样本：settled-item-self-open-red 里 a 支「取值仍未定」、b 支「校验算法选哪个仍未定」各一条判红；decision-documents-green 里无害的「本分项取区间上端 5」判绿。
grid_settled_item_self_open() (
  DEC="$PWD/.claude/kb/decisions"
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }

  python3 - "$DEC" <<'PY'
import re, sys, pathlib

dec = pathlib.Path(sys.argv[1])
SELF = ("取值", "本项", "该项", "本分项", "本条")
OPEN = re.compile(r'(仍然未定|仍未定|尚未定(?!案)|还没定|均未定)')
# X：断言之前那一小段名词短语。停在标点 / markdown 记号 / 空白上。
PHRASE = re.compile(r'([^\s。，、；：（）()「」|*`＊>#⚠️⇒—]{2,12})$')

bad = []
files_with_settled_region, settled_region_lines = 0, 0
decision_files = sorted(dec.glob("*.md"))
if not decision_files:
    print(f"  ! {dec} 下没有决策正文，这一格无对象可判")
    sys.exit(77)
for f in decision_files:
    lines = f.read_text(encoding="utf-8").splitlines()

    # 已定项小节 = `### 已定项` 索引表 + 各 `#### 已定项 N` 论证
    inside, region, settled_lines = False, [], []
    for i, line in enumerate(lines, 1):
        if re.match(r'^###\s+已定项\s*$', line) or re.match(r'^####\s+已定项', line):
            inside = True
            settled_lines.append(line)
            continue
        if re.match(r'^###\s+未定项\s*$', line) or re.match(r'^####\s+未定项', line) or re.match(r'^##\s', line):
            inside = False
        if inside:
            region.append((i, line))
            settled_lines.append(line)

    if not region:
        continue
    files_with_settled_region += 1
    settled_region_lines += len(region)
    settled_text = "\n".join(settled_lines)

    for i, line in region:
        for m in OPEN.finditer(line):
            head = line[:m.start()].rstrip()
            pm = PHRASE.search(head)
            if not pm:
                continue
            x = pm.group(1).strip('*` ')
            if len(x) < 2:
                continue
            # 「`T_time` 的取值仍未定」取到的是「的取值」：去掉打头的「的」再比自指词
            x_core = x[1:] if x.startswith('的') else x
            why = None
            if x_core in SELF:
                why = "自指——这条分项标着已定，正文却说它自己没定"
            else:
                # 同一文件里另一条已定分项的标题含 X？（排除本行自己）
                for other in settled_text.splitlines():
                    if other.strip() == line.strip():
                        continue
                    if re.match(r'^\s*(\|\s*)?\d+[.\s|]', other) and x in other and '已定' in other:
                        why = f"「{x}」在同一条决策的另一条**已定**分项里已经定了"
                        break
            # 一行只报一处；这一处没判出问题就接着看同一行后面的断言（「子问题仍未定，取值仍未定」）
            if why:
                bad.append((f.name, i, x, why, line.strip()[:110]))
                break

if bad:
    for fn, ln, x, why, text in bad:
        print(f"  ✗ {fn}:{ln} 已定项正文里断言「{x}」未定：{why}")
        print(f"     原文：{text}")
    print("     → 怎么办：这是定案之后没清理的推导过程。把这句改写成定案后的现状；")
    print("               若那个子问题真的还开着，把它升成一条独立的未定项，别留在已定项正文里。")
    sys.exit(1)
if files_with_settled_region == 0:
    print(f"  ! 本次无对象可判：{len(decision_files)} 份决策正文里没有一份有已定项区段（### 已定项 / #### 已定项 N）")
    sys.exit(77)
print(f"  ✓ 已定项的正文没有把已经定了的东西说成未定（扫 {len(decision_files)} 条决策，"
      f"其中 {files_with_settled_region} 条有已定项区段、共 {settled_region_lines} 行）")
PY
)

# ════ 格「引用写已定紧跟说没定」════
# 引用写着「已定项 k」、归属到的那条也已定，后面紧跟「未定 / 没定 / 定不下 / 待定 / 空着」（中间只许一段括注、标点与「今天 / 仍 / 还」这类词）：判红。
# 抓的是改完标签、句子还在说没定的那一型：格「分项引用状态」只比标签，「说未定其实已定」只认紧贴的「未定」，「已定分项自称未定」只扫已定项小节里的自指。
# 归属用 lib-item-ref-status.py（与格「分项引用状态」同一份）。两份变更史与 records/ 写的是当时的状态，不扫，成功行逐个列出。
# 样本：settled-ref-says-open-red 判红；decision-documents-green 里 records/ 下同一种句子不扫，判绿。
grid_settled_ref_says_open() (
  python3 - "$LIB_ITEM_REF_STATUS" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location('item_ref_status', sys.argv[1])
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
heads, unreadable = lib.decision_heads(strict=False)
item_map, names = lib.load_map(heads)
self_map = lib.self_decisions(heads)
def written_as_of_then(f):
    return (f.endswith(('decisions-history.md', 'experiments-history.md')) or '/decisions-history/' in f
            or f.startswith('records/'))
candidates = lib.scanned_files()
files = [f for f in candidates if not written_as_of_then(f)]
skipped = [f for f in candidates if written_as_of_then(f)]
bad = []; seen = 0
for path in files:
    for ln, line, m, want, owner, missing, _ in lib.references(path, item_map, self_map):
        if missing or owner is None or want != '已' or item_map[owner][int(m.group(2))] != '已':
            continue
        seen += 1
        if lib.says_open_after(line, m.end()):
            snippet = line[max(0, m.start() - 20):m.end() + 30].strip()
            bad.append(f"{path}:{ln} 「{owner}（{names[owner]}） {m.group(0)}」后面紧跟着说它没定：…{snippet}…")
if unreadable:
    lib.report_unreadable(unreadable)
if bad:
    print(f"  ✗ 引用写着已定项、紧跟着却说它没定 {len(bad)} 处")
    for b in bad[:40]: print("    ", b)
    print("     → 那条分项已经定了：把句子改成它定下来之后的说法（定成了什么、这一处按它重算了没有），别只改标签。")
if bad or unreadable:
    sys.exit(1)
if seen == 0:
    print(f"  ! 扫了 {len(files)} 个文件，没有一处归属到已定分项的「已定项」引用，这一格无对象可判")
    sys.exit(77)
print(f"  ✓ 扫了 {len(files)} 个文件、{seen} 处已定项引用，没有一处紧跟着说它没定")
print(f"    没扫 {len(skipped)} 个（两份变更史与 records/ 写的是当时的状态）" + ('：' + '、'.join(skipped) if skipped else ''))
PY
)

# ════ 格「未定项被别处定了」════
# 还开着的未定项的条目块点名了别的决策 Dn，而 Dn 最近一次新增「—— 已定」小节标题晚于这条未定项对 Dn 的最近一次复核：判红，要求复核。
# 按时间比、不按文本依赖图：陈旧的未定项多半没写「前置是某某」这种标记。
# git 取不到时间或条目块历史判红，不当「从没定过 / 复核过了」。没有一条未定项点名定过东西的别的决策退 77。
# 判不了：决策文档拆分之前的历史（逐行历史在拆分处断了），欠账 C36（未定项检查的第三个盲区）。
# 只读中文原文：译文（<名>.<两位语言>.md）与中文原文同一个决策号，没有「—— 已定」标题；连译文一起记每条决策的「—— 已定」小节时间，后读到的译文会把中文原文那一条盖成 0，点名它的未定项就一条不比。
# 通配次序按字节排（LC_COLLATE=C），不随 LANG 变：红行的先后在哪台机器上都一样。
# 样本：stale-open-items-red 另造缺旧版本对象的仓：一份决策的 git log -G、一份条目块的 git log -L 读不到，两处都要报；
#   stale-open-items-translation-red 是 D96 的中文原文新增了已定小节、D96 的译文排在它后面，D95 点名 D96 的未定项判红。
# 弄坏开关 DOC_DECISIONS_BREAK=stale-open-items-translations-read：两遍都连译文一起读，stale-open-items-translation-red 那条未定项不比（退 77）。
grid_stale_open_items() (
  DEC=.claude/kb/decisions
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }
  git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，这一格跳过"; exit 77; }

  export LC_COLLATE=C
  decision_files=()
  for f in "$DEC"/*.md; do
    if [[ "$BREAK_SWITCH" != stale-open-items-translations-read && "$f" =~ \.[a-z]{2}\.md$ ]]; then continue; fi
    decision_files+=("$f")
  done

  # 每个决策状态行最后一次变动的提交时间；取不到（git 失败）记 -1，点名它的未定项逐条报「没比过」
  declare -A st_time
  time_unavailable=()
  for f in "${decision_files[@]}"; do
    n=$(grep -m1 -oE '^## (D[0-9]+)' "$f" | awk '{print $2}') || continue
    [[ -n "$n" ]] || continue
    # 「定了新东西」取这个文件最近一次新增「—— 已定」小节标题的提交：决策首行几乎不变；-G 加行首锚定只认标题行，正文里的「已定」二字会造成假阳性。
    # git 的退出码先判再取第一行：「git log … | head -1」只交 head 的退出码，git 失败会被读成「从没定过」
    if ! t_out="$(git log -1 --format=%ct -G'^#{2,4} .*—— 已定' -- "$f")"; then
      st_time[$n]=-1
      time_unavailable+=("$n（$(basename "$f")）")
      continue
    fi
    t="$(head -n 1 <<<"$t_out")"
    # 从没加过「—— 已定」标题记 0、不触发复核；不退化成文件修改时间（那样每次补内容都把引用它的未定项全报一遍）
    [[ -n "$t" ]] || t=0
    st_time[$n]=$t
  done

  flagged=0
  if ((${#time_unavailable[@]})); then
    flagged=1
    echo "  ✗ 这些决策的「—— 已定」小节最近一次变动时间取不到（git log 失败），点名它们的未定项这一轮没法比：${time_unavailable[*]}"
    echo "     → 怎么办：按上面 git 的报错修好仓库（git fsck 看缺了哪些对象，从远端或备份找回）再跑；取不到时间不是「这条决策从没定过」，不是通过。"
  fi
  open_items=0     # 扫到的未定项条数
  judged_items=0   # 其中点名了「定过东西的别的决策」、真拿去比过复核时间的条数
  for f in "${decision_files[@]}"; do
    self=$(grep -m1 -oE '^## (D[0-9]+)' "$f" | awk '{print $2}')
    while IFS=: read -r ln text; do
      [[ -n "$ln" ]] || continue
      open_items=$((open_items + 1))
      # 复核时间按被点名的决策逐条算，判据在 OPEN_ITEM_REVIEW_SOURCE：条目块的任意改动不算复核（否则什么改动都能消红），
      # 工作区行号先映射回 HEAD 再喂 git log -L（-L 按历史里的文件解释行号，上方插过行会错位成假红）。
      # 复核写一句点名 Dn 的「（YYYY-MM-DD 复核 Dn：……）」就满足。
      deps=()
      # 点名按整个条目块认（首行加续行，到下一条分项或标题之前、至多 20 行），与 OPEN_ITEM_REVIEW_SOURCE 的 block_end 同口径：续行里的点名也算
      block_text="$(awk -v s="$ln" 'NR == s { print; next }
          NR > s && NR <= s + 20 { if (/^[[:space:]]*[0-9]+\. / || /^##+ / || /^\|[[:space:]]*[0-9]+[[:space:]]*\|/) exit; print }' "$f")"
      # D 编号前面要是非字母数字：「RAID5」里的「D5」不是在点名 D5
      for d in $(grep -oE '(^|[^A-Za-z0-9])D[0-9]+' <<<"$block_text" | grep -oE 'D[0-9]+' | sort -u); do
        [[ "$d" == "$self" ]] && continue
        dt=${st_time[$d]:-0}
        if (( dt < 0 )); then
          echo "  ✗ $(basename "$f"):$ln 的未定项点名了 $d，而 $d 的变动时间取不到，这一条对 $d 没比过"   # gate-lint:detail
          continue
        fi
        (( dt > 0 )) && deps+=("$d:$dt")
      done
      (( ${#deps[@]} > 0 )) || continue
      judged_items=$((judged_items + 1))
      # 复核判据的输出先落到变量、判过退出码再读：接进 `< <(…)` 时它崩了只是少打几行，这一条被读成「复核过了」
      review_out="$(python3 -c "$OPEN_ITEM_REVIEW_SOURCE" "$f" "$ln" "${deps[@]}")" || {
        review_rc=$?
        echo "  ✗ $(basename "$f"):$ln 的复核判据没跑成（复核判据 OPEN_ITEM_REVIEW_SOURCE 退出码 $review_rc），这一条没比过"
        echo "     → 按上面的报错修 .claude/gate.d/lib/doc-decisions-status-cells.sh 里的 OPEN_ITEM_REVIEW_SOURCE 或这一份决策正文；判据没跑成不是通过。"
        exit 1
      }
      while IFS= read -r d; do
        [[ -n "$d" ]] || continue
        echo "  ✗ $(basename "$f"):$ln 的未定项点名了 $d，而 $d 的状态行在它之后变过"
        echo "     ⇒ 复核这一项是不是已经被 $d 定掉了；复核完在这一条里写一句点名 $d 的复核记录。原文：${text:0:60}"
        flagged=1
      done <<<"$review_out"
    # 未定项按所在小节取，不按行内关键字滤：列表式条目行内没有「未定」，未定项也常提到别处的「已定」
    done < <(awk -F: '
        # 只认光秃秃的「### 未定项」（「### 未定项 3 —— 已定」也以它开头）；在第一个 #### 处收口，之后是论证里的编号列表
        /^### 未定项[[:space:]]*$/  { inside=1; next }
        /^### /              { inside=0 }
        /^#### /             { inside=0 }
        /^## /               { inside=0 }
        inside && /^[[:space:]]*[0-9]+\. |^\| [0-9]+ \|/ { print NR":"$0 }
      ' "$f")
  done

  if ((flagged)); then
    echo "     → 怎么办：逐条复核；已被别处定掉的就改写成「已定，权威记录在 XX」，"
    echo "               仍然开着的就把点名改成不构成依赖的写法。"
    exit 1
  fi
  if ((judged_items == 0)); then
    echo "  ! 查了 $open_items 条未定项，没有一条点名定过东西的别的决策，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 没有未定项被别处的更新甩在后面（查了 $open_items 条未定项，其中 $judged_items 条点名了定过东西的别的决策、逐条比过复核时间）"
)

# ════ 格「定了新东西没回看同文件未定项」════
# 这一轮在一份决策里新增了「已定」小节标题（首行登记标题的 `—— 已定`，或分项的 `已定项 N：…` / `已定项 N（…）`），而同文件还开着的未定项（「### 未定项」与「### 还要回答的」两节）一条都没碰、
# 索引页 decisions.md 还悬着的「## 待议」节一个字没动：判红，要一次回头看。
# 与格「未定项被别处定了」是两把尺：那一格跨文件看历史，这一格同文件看这一轮的 diff。按历史判会在每个有已定小节又有未定项的决策上恒红。
# 「碰没碰」按条目块判（表格行一行一条，列表项到下一条分项或标题之前，与 OPEN_ITEM_REVIEW_SOURCE 同口径）；待议按那一节判（`## 待议` 到下一个 `## `），索引页别处动过不算。
# 新增了已定小节的几份里一条未定项都没有、索引页也没有待议节时退 77。
# 样本：settled-same-file-red 另放一份两行表格的决策（只改了第 2 行，第 1 行要报）与一节没动的待议（同一次只改了索引表那一行）。
# settled-same-file-item-heading-red / -green 新增的是现行写法 `#### 已定项 N：名字`：红的那份只有未定项没动，绿的那份未定项与待议节都补了复核注。
# 改过的几份决策的新增行一次取齐（行首带路径）、按路径分回各文件，新增了已定小节的只算那一份自己：settled-same-file-two-files-green 里 D982 新增了已定小节、D983 只改了无关的一句话，D983 不算。
# 弄坏开关 DOC_DECISIONS_BREAK=settled-same-file-added-lines-pooled：把几份的新增行并成一堆看，D983 也算新增了已定小节，它没动的未定项判红。
grid_settled_same_file() (
  DEC=.claude/kb/decisions
  IDX=.claude/kb/decisions.md
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }
  git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，这一格跳过"; exit 77; }

  # 改动范围取共用脚本 changed-paths.sh，不另算一份（code-tooling 的 change-range-single-source 格判）；名单带未跟踪文件
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  BASE="$(gate_diff_base gate)"

  # 名单先落到变量、判过退出码再读：git 失败时名单是空的，会被读成「本次没有新增已定小节」
  all_changed="$(gate_changed_paths "$BASE" untracked)" || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $BASE），这次改了哪些决策文件没取到"
    echo "     → 按上面 git 的报错修好仓库状态（基准 $BASE 要存在；仓里还没有提交就先提交一次）再跑；git 失败时这一格什么都没比，不是通过。"
    exit 1
  }
  changed_names="$(grep "^\.claude/kb/decisions/" <<<"$all_changed" || true)"

  # 只认新增行里的小节标题：`## D1 简称 —— 已定` / `### 未定项 3 —— 已定` / `#### 已定项 12：名字` / `#### 已定项 12（名字）`；未跟踪的文件整份算新增
  # 决策文件首行的登记标题写作 `## D<n> 简称 —— 已定`，新分项的标题写作 `#### 已定项 N：名字`（format-evolution.md「决策正文只写现状」），两种都认
  # 开关 settled-item-heading-unrecognised 退回不认「已定项 N」（样本自检用）
  settled_heading_pattern='^#{2,4} .*—— 已定|^#{2,4} 已定项[[:space:]]*[0-9]+[[:space:]]*(：|:|（|\()'
  if [[ "$BREAK_SWITCH" == settled-item-heading-unrecognised ]]; then
    settled_heading_pattern='^#{2,4} .*—— 已定'
  fi
  changed_name_list=()
  while IFS= read -r f; do
    [[ -n "$f" ]] && changed_name_list+=("$f")
  done <<<"$changed_names"
  settled_files=()
  if ((${#changed_name_list[@]})); then
    # 改过的决策文件的新增行一次取齐（行首是路径，剔搬迁行按路径各算各的，合起来取与逐份取结果相同），再按行首的路径分回各文件：第 k 行的路径与正文是同一行的两半。
    # 不接 `| grep -q`：pipefail 下前段吃 SIGPIPE，命中会被读成没命中
    added_out="$(gate_added_lines "$BASE" "${changed_name_list[@]}")" || {
      echo "  ✗ 取不到这次改动的决策文件新增了哪些行（基准 $BASE）"
      echo "     → 按上面 git 的报错修好仓库状态再跑；取不到新增行时这一格什么都没比，不是通过。"
      exit 1
    }
    added_paths="$(cut -f1 <<<"$added_out")"
    added_texts="$(cut -f2- <<<"$added_out")"
    heading_line_numbers="$(grep -nE "$settled_heading_pattern" <<<"$added_texts" | cut -d: -f1)" || true
    declare -A has_settled_heading=()
    while IFS= read -r heading_path; do
      [[ -n "$heading_path" ]] && has_settled_heading["$heading_path"]=1
    done < <(awk 'NR == FNR { hit[$1] = 1; next } FNR in hit { print }' <(printf '%s\n' "$heading_line_numbers") <(printf '%s\n' "$added_paths"))
    # 弄坏开关 settled-same-file-added-lines-pooled：几份改过的决策的新增行并成一堆看，有一份新增了已定小节就当每一份都新增了
    if [[ "$BREAK_SWITCH" == settled-same-file-added-lines-pooled ]] && ((${#has_settled_heading[@]})); then
      for f in "${changed_name_list[@]}"; do has_settled_heading["$f"]=1; done
    fi
    for f in "${changed_name_list[@]}"; do
      [[ -n "${has_settled_heading[$f]:-}" ]] && settled_files+=("$f")
    done
  fi

  if ((${#settled_files[@]} == 0)); then
    echo "  ! 本次 diff 没有新增「已定」小节，这一格无对象可判"
    exit 77
  fi

  # touched_lines_of <文件>：这一轮碰过的行号（新文件侧）一行一个，未跟踪的整份都算碰过。
  # ls-files --error-unmatch 退 0 跟踪、1 未跟踪，别的码返回 2 由调用方判红：当成未跟踪会整份算碰过、一条都不报。
  touched_lines_of() {
    local file="$1" tracked_rc=0
    git -c core.quotepath=false ls-files --error-unmatch -- "$file" >/dev/null 2>&1 || tracked_rc=$?
    case "$tracked_rc" in
      0) git diff --no-color --no-ext-diff -U0 "$BASE" -- "$file" \
           | awk 'match($0,/^@@ .* \+([0-9]+)(,([0-9]+))? @@/,m){s=m[1]; n=(m[3]==""?1:m[3]); for(i=0;i<n;i++) print s+i}' ;;
      1) seq 1 "$(wc -l < "$file")" ;;
      *)
        echo "  判不了 $file 跟踪没有：git 退出码 $tracked_rc" >&2
        return 2 ;;
    esac
  }

  flagged=0
  open_checked=0      # 查过的未定项条数（新增了已定小节的那几份里）
  pending_checked=0   # 查过的索引页「待议」节数

  # ── ① 同文件里还开着、而本次一个都没碰的未定项 ──────────────────
  for f in "${settled_files[@]}"; do
    touched="$(touched_lines_of "$f")" || {
      echo "  ✗ 判不了 $f 有没有被 git 跟踪（上面是 git 的退出码），这一份的未定项这一轮没比"
      echo "     → 怎么办：按 git 的报错修好仓库状态（索引坏了就 git status 看、必要时重建索引）再跑；判不了不是「未跟踪、整份都碰过」。"
      exit 1
    }
    while IFS=: read -r ln text; do
      [[ -n "$ln" ]] || continue
      open_checked=$((open_checked + 1))
      end=$(awk -v s="$ln" 'NR>s && (/^[[:space:]]*[0-9]+\. /||/^\|[[:space:]]*[0-9]+[[:space:]]*\|/||/^##+ /){print NR-1; exit}' "$f")
      [[ -n "$end" ]] || end=$((ln+20))
      hit=0
      for t in $touched; do (( t>=ln && t<=end )) && { hit=1; break; }; done
      (( hit )) && continue
      echo "  ✗ $(basename "$f"):$ln 本次新增了「已定」小节，而这条未定项一个字都没动"   # gate-lint:detail
      echo "     ⇒ 复核它是不是被这次定案顺带答掉了。原文：${text:0:60}"
      flagged=1
    done < <(awk '
        /^### 未定项[[:space:]]*$/    { inside=1; next }
        /^### 还要回答的[[:space:]]*$/ { inside=1; next }
        /^### /                      { inside=0 }
        /^#### /                     { inside=0 }
        /^## /                       { inside=0 }
        inside && /^[[:space:]]*[0-9]+\. |^\| [0-9]+ \|/ { print NR":"$0 }
      ' "$f")
  done

  # ── ② 索引页里还悬着、而本次一个都没碰的「待议」节 ──────────────
  if [[ -f "$IDX" ]]; then
    idx_touched_lines=""
    if grep -qxF "$IDX" <<<"$all_changed"; then
      idx_touched_lines="$(touched_lines_of "$IDX")" || {
        echo "  ✗ 判不了 $IDX 有没有被 git 跟踪（上面是 git 的退出码），索引页的待议节这一轮没比"
        echo "     → 怎么办：按 git 的报错修好仓库状态再跑；判不了不是「整份都碰过」。"
        exit 1
      }
    fi
    while IFS=: read -r ln text; do
      [[ -n "$ln" ]] || continue
      grep -qE '已回收|已收摊|已并入' <<<"$text" && continue
      pending_checked=$((pending_checked + 1))
      section_end=$(awk -v s="$ln" 'NR>s && /^## /{print NR-1; exit}' "$IDX")
      [[ -n "$section_end" ]] || section_end=$(wc -l < "$IDX")
      section_touched=0
      for t in $idx_touched_lines; do (( t>=ln && t<=section_end )) && { section_touched=1; break; }; done
      (( section_touched )) && continue
      echo "  ✗ $(basename "$IDX"):$ln 本次有决策定案，而这一节「待议」一个字都没动"
      echo "     ⇒ 复核它是不是被这次定案实质回答/否决了。原文：${text:0:60}"
      flagged=1
    done < <(grep -nE '^## 待议' "$IDX")
  fi

  if ((flagged)); then
    echo "     → 怎么办：被顺带定掉的就改写成「已定，权威记录在 XX」或直接收摊；"
    echo "               仍然开着的就在条目里补一行「YYYY-MM-DD 复核过，仍然开着，因为 …」。"
    echo "               后一种做法本身就是这条检查要的东西——它要的是一次回头看，不是一次沉默。"
    exit 1
  fi
  if ((open_checked == 0 && pending_checked == 0)); then
    echo "  ! 本次无对象可判：新增了已定小节的 ${#settled_files[@]} 份决策里没有一条未定项，索引页也没有悬着的待议节，什么都没查"
    exit 77
  fi
  echo "  ✓ 本次定案之后，同文件的未定项与索引页的待议节都被回头看过（新增已定小节的决策 ${#settled_files[@]} 份，查了 $open_checked 条未定项、$pending_checked 节待议）"
)
