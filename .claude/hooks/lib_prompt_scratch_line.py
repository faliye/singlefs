"""派发提示里「本提示的草稿目录」那一行怎么认：只在这一份里写，六份文件都调它，不各写一份
（拆分提案第九十节 ㉘；判决 research/prompts/landing-audit-r4-main-verification.md K2-4、K3-2）。

六份文件用这一份：
  .claude/hooks/runner-dispatch-guard.sh     ⑯ 草稿目录写者锁建在哪个目录（scratch_directories），那一行在这一份的围栏认法（fenced_line_numbers）与宽的逐行认法下
                                             遮不遮不一致就拒（单看一行认不认得出路径用 scratch_line_values）；⑰ 报告行只判落在本提示草稿目录下的路径（scratch_directories）
  .claude/hooks/completion-cleanup-guard.sh  子 agent 交回、收工时判派发提示点名的那个草稿目录（scratch_line_values；写相对路径的按仓根解，用 repository_root）
  .claude/hooks/handback-guard.sh            ⑥ 写回员自己起草的 spec.json / spec.md 到哪个草稿目录下找（scratch_directories）
  scripts/agents/agent-watch.py              看门狗取子 agent 的 progress.md 只取派发提示点名的那个草稿目录（scratch_directories）
  scripts/agents/dispatch-prompt.py          骨架 --check 的认领行那一判：「草稿目录：」一行点名的目录要有同一目录的认领行（scratch_line_values）
  scripts/agents/agent-cost.py               账表每个子 agent 的草稿目录取派发提示里那一行点名的第一个（scratch_line_values）

认法（scratch_line_values；取骨架 python3 scripts/agents/dispatch-prompt.py <类型> 打的那一行）：
  一行开头是「草稿目录」（前面只许空格与制表符：列表记号、引用记号、粗体、编号起头的都不认），紧接全角或半角冒号（中间可隔空白），
  冒号后面（可隔空白、可带一个反引号）是路径；路径到空白、反引号、引号、中文标点、括号、尖括号为止，末尾的 . , ; : 去掉。
  冒号后面是 <填 …> 这类占位的（尖括号不进路径，取出来是空的）不认。
  围栏代码块照 CommonMark 认（里面的行不认）：起头一行前面至多三个空格（四个以上是缩进代码块，不算围栏），``` 或 ~~~ 至少三个；
  反引号围栏的起头行在语言名的位置不许再有反引号（那是行内代码，不开块）；收尾一行前面至多三个空格，同一种字符、不比起头短、后面只有空白（后面接语言名之类的字不收尾）；
  没收尾的围栏一直到文末，里面的行也不认。列表项与引用块里的围栏不单独认；派发闸 ⑭⑮ 的 without_code_blocks 只遮闭合了的围栏，没闭合的照判，这一份没闭合的遮到文末，两处在没闭合上不同。
  句中、列表里的说法一律不认：「，草稿目录 <别处>」「- 草稿目录 <别处>」是在转述别的子 agent 的目录，「草稿目录在 <路径>」不是骨架那一行。
  照骨架写的提示都有那一行；没有那一行的提示，五处各按自己的规矩办：派发闸 ⑰ 只判不带目录的裸文件名、⑯ 不建锁；
  完工清理闸只判 claim 认领的目录；交回闸 ⑥ 只看提示点名的规格文件与提示正文；看门狗报取不到 progress.md、不猜别处。
scratch_directories：scratch_line_values 取到的路径经同目录 lib_work_root.py 的 expand_written_path（~/、$HOME/、$SINGLEFS_WORK_ROOT/）
  展开成绝对路径（normpath），写相对路径的按仓根（这份文件往上三层，repository_root）解，不按调用时的当前目录；只留落在工作根或过渡期旧根（lib_work_root.py 的 scratch_roots）底下的，根本身不算；
  include_roots 为真时根本身也算（只给派发闸的弄坏开关 writer-lock-root-locked 用）。读不到 lib_work_root.py 时抛异常，调用方按读不到这份库办。

用法：
  python  把 .claude/hooks 放进 sys.path 再 import lib_prompt_scratch_line，或按文件路径用 importlib 导入（lib_work_root.py 按这份文件自己所在的目录找）
  python3 .claude/hooks/lib_prompt_scratch_line.py --selftest   # 自证；下面每个弄坏开关单独打开时必须判红

弄坏开关（平常跑设了退 2、点名只在自证里认；被 import 时照认，给调用方的自证判红用；几个用逗号连，不认识的项抛 ValueError）：
  LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere  「草稿目录」在一行里哪都认、冒号可有可无（第九批派发闸 ⑰ 的认法）
      「✗ 自检：转述里「，草稿目录 <别处>」不认」
      「✗ 自检：列表里「- 草稿目录 <别处>」不认」
      「✗ 自检：列表与粗体起头的那一行不认」
      「✗ 自检：「草稿目录在 <路径>」不是骨架那一行，不认」
  LIB_PROMPT_SCRATCH_LINE_BREAK=code-blocks-read  闭合了的围栏代码块里的行也认
      「✗ 自检：闭合了的围栏代码块里的那一行不认」
      「✗ 自检：没收尾的围栏一直到文末，里面的行不认」
      「✗ 自检：没收尾的围栏之前的那一行照认」
      「✗ 自检：收尾行后面接了字不算收尾，围栏一直到文末」
      「✗ 自检：收尾比起头短不算收尾，围栏一直到文末」
      「✗ 自检：收尾比起头长照收尾」
      「✗ 自检：波浪号围栏照认、收尾要同一种字符」
      「✗ 自检：缩进 3 格的围栏照认」
      「✗ 自检：缩进 4 格的收尾不算收尾，围栏一直到文末」
  LIB_PROMPT_SCRATCH_LINE_BREAK=roots-unfiltered  scratch_directories 不看落没落在工作根或过渡期旧根底下（根本身与根外都留）
      「✗ 自检：工作根本身不算草稿目录」
      「✗ 自检：工作根与旧根之外的目录不算」
      「✗ 自检：相对路径按仓根解，不按调用时的当前目录（当前目录在工作根）」
  LIB_PROMPT_SCRATCH_LINE_BREAK=fence-info-with-backtick  反引号围栏的起头行在语言名的位置有反引号也当围栏起头（改前的认法）
      「✗ 自检：反引号围栏的起头行在语言名的位置有反引号，那是行内代码，不开块」
      「✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致」（本机装了 markdown-it-py 才查）
  LIB_PROMPT_SCRATCH_LINE_BREAK=unclosed-fence-read  没收尾的围栏不遮到文末，里面的行照认（改前的认法）
      「✗ 自检：没收尾的围栏一直到文末，里面的行不认」
      「✗ 自检：收尾行后面接了字不算收尾，围栏一直到文末」
      「✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致」（本机装了 markdown-it-py 才查）
      「✗ 自检：没收尾的围栏之前的那一行照认」
      「✗ 自检：收尾比起头短不算收尾，围栏一直到文末」
      「✗ 自检：缩进 4 格的收尾不算收尾，围栏一直到文末」
  LIB_PROMPT_SCRATCH_LINE_BREAK=relative-by-cwd  写相对路径的按调用时的当前目录解，不按仓根（改前的认法）
      「✗ 自检：相对路径按仓根解，不按调用时的当前目录（当前目录在仓外）」
      「✗ 自检：相对路径按仓根解，不按调用时的当前目录（当前目录在工作根）」
"""
import importlib.util
import os
import re
import sys

BREAK_VARIABLE = "LIB_PROMPT_SCRATCH_LINE_BREAK"
KNOWN_BREAK_ITEMS = ("label-anywhere", "code-blocks-read", "roots-unfiltered", "fence-info-with-backtick", "unclosed-fence-read", "relative-by-cwd")
SCRATCH_LABEL = "草稿目录"
PATH_CHARACTERS = r"[^\s`'\"，。；：、（）()「」『』“”,;*<>]+"
SCRATCH_LINE = re.compile(r"^[ \t]*" + SCRATCH_LABEL + r"[ \t]*[：:][ \t]*`?(?P<path>" + PATH_CHARACTERS + r")?", re.M)
# 弄坏开关 label-anywhere 下的认法（第九批派发闸 ⑰：前面不连着字就认，冒号可有可无，不限行首）
SCRATCH_ANYWHERE_BEFORE = re.compile(r"(?<![^\W_])" + SCRATCH_LABEL + r"[* \t]*[：:]?[* \t]*`?(?P<path>" + PATH_CHARACTERS + r")?")
FENCE_OPENING = re.compile(r"^ {0,3}(?P<fence>`{3,}|~{3,})(?P<info>.*)$")
TRAILING_PUNCTUATION = ".,;:"
WORK_ROOT_LIBRARY = []


def broken(item):
    """弄坏开关 LIB_PROMPT_SCRATCH_LINE_BREAK 里有没有这一项；不认识的项抛 ValueError（拼错的开关不当成「没开」）。"""
    items = [value for value in os.environ.get(BREAK_VARIABLE, "").split(",") if value]
    unknown = [value for value in items if value not in KNOWN_BREAK_ITEMS]
    if unknown:
        raise ValueError(f"{BREAK_VARIABLE} 里的 {'、'.join(unknown)} 不认识，只认 {'、'.join(KNOWN_BREAK_ITEMS)}")
    return item in items


def work_root_library():
    """同目录的 lib_work_root.py（工作根与过渡期旧根只在那里定义），按这份文件所在的目录导入一次。"""
    if not WORK_ROOT_LIBRARY:
        path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "lib_work_root.py")
        specification = importlib.util.spec_from_file_location("lib_work_root_for_prompt_scratch_line", path)
        loaded = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(loaded)
        WORK_ROOT_LIBRARY.append(loaded)
    return WORK_ROOT_LIBRARY[0]


def repository_root():
    """仓根：这份文件在 <仓根>/.claude/hooks/ 下，往上三层。相对路径按它解，不按调用时的当前目录。"""
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def fenced_line_numbers(lines):
    """围栏代码块占的行号（含起止两行），照 CommonMark 认：起头一行前面至多三个空格，反引号围栏的语言名位置不许再有反引号；
    往后第一行同一种字符、不比起头短、前面至多三个空格、后面只有空白的是收尾；找不到收尾的一直到文末（弄坏开关 unclosed-fence-read 下不算代码块）。"""
    covered, index = set(), 0
    while index < len(lines):
        opening = FENCE_OPENING.match(lines[index].rstrip("\r"))
        if not opening or (opening.group("fence")[0] == "`" and "`" in opening.group("info") and not broken("fence-info-with-backtick")):
            index += 1
            continue
        fence = opening.group("fence")
        closing = re.compile(r"^ {0,3}" + re.escape(fence[0]) + "{" + str(len(fence)) + r",}[ \t]*$")
        end = next((later for later in range(index + 1, len(lines)) if closing.match(lines[later].rstrip("\r"))), None)
        if end is None:
            if broken("unclosed-fence-read"):
                index += 1
                continue
            end = len(lines) - 1
        covered.update(range(index, end + 1))
        index = end + 1
    return covered


def scratch_line_values(prompt):
    """提示里行首「草稿目录：<路径>」那几行的路径，照写的样子（没展开），按出现次序、去重。写法见文件头「认法」。"""
    lines = str(prompt or "").split("\n")
    skipped = set() if broken("code-blocks-read") else fenced_line_numbers(lines)
    pattern = SCRATCH_ANYWHERE_BEFORE if broken("label-anywhere") else SCRATCH_LINE
    values = []
    for number, line in enumerate(lines):
        if number in skipped:
            continue
        for match in pattern.finditer(line):
            value = (match.group("path") or "").rstrip(TRAILING_PUNCTUATION)
            if value and value not in values:
                values.append(value)
    return values


def scratch_directories(prompt, include_roots=False):
    """scratch_line_values 取到的路径展开成绝对路径，只留工作根或过渡期旧根底下的（根本身不算，include_roots 为真时也算），按次序、去重。"""
    work_roots = work_root_library()
    roots = [root.rstrip("/") for root in work_roots.scratch_roots()]
    found = []
    for value in scratch_line_values(prompt):
        written = work_roots.expand_written_path(value)
        if not os.path.isabs(written) and not broken("relative-by-cwd"):
            written = os.path.join(repository_root(), written)
        path = os.path.normpath(os.path.abspath(written))
        under = any(path.startswith(root + "/") for root in roots) or (include_roots and path in roots)
        if (under or broken("roots-unfiltered")) and path not in found:
            found.append(path)
    return found


def fence_lines_by_reference(samples):
    """拿 markdown-it-py 的 commonmark 预设（派发闸 without_code_blocks 用的那一份）认同一批样本的围栏行，与 fenced_line_numbers 比：
    [(样本名, 参照认的行号, 这一份认的行号)] 只交不一样的；本机没装 markdown-it-py 交 None（这一格不查）。"""
    try:
        import markdown_it
        parser = markdown_it.MarkdownIt("commonmark")
    except Exception:
        return None
    different = []
    for label, (text, _want) in samples.items():
        lines = text.split("\n")
        reference = set()
        for token in parser.parse(text):
            if token.type == "fence" and token.map:
                reference.update(range(token.map[0], token.map[1]))
        mine = {number for number in fenced_line_numbers(lines) if number < len(lines) - 1}
        if reference != mine:
            different.append((label, sorted(reference), sorted(mine)))
    return different


def selftest():
    work_roots = work_root_library()
    root, legacy = work_roots.configured_work_root(), work_roots.legacy_root()
    home = os.path.expanduser("~")
    written_root = "~" + root[len(home):] if root.startswith(home + "/") else root
    mine, other = f"{root}/b10-selftest-mine", f"{root}/b10-selftest-previous"
    fence_samples = {
        "反引号围栏的起头行在语言名的位置有反引号，那是行内代码，不开块": (f"``` a`b\n草稿目录：{mine}\n```\n", [mine]),
        "没收尾的围栏一直到文末，里面的行不认": (f"```\n草稿目录：{mine}\n", []),
        "没收尾的围栏之前的那一行照认": (f"草稿目录：{mine}\n```\n草稿目录：{other}\n", [mine]),
        "收尾行后面接了字不算收尾，围栏一直到文末": (f"```\n草稿目录：{other}\n``` 说明\n草稿目录：{other}\n", []),
        "收尾比起头短不算收尾，围栏一直到文末": (f"````\n草稿目录：{other}\n```\n草稿目录：{other}\n", []),
        "收尾比起头长照收尾": (f"```\n草稿目录：{other}\n`````\n草稿目录：{mine}\n", [mine]),
        "波浪号围栏照认、收尾要同一种字符": (f"~~~\n草稿目录：{other}\n```\n草稿目录：{other}\n~~~\n草稿目录：{mine}\n", [mine]),
        "缩进 4 格的围栏起头不算围栏": (f"    ```\n草稿目录：{mine}\n    ```\n", [mine]),
        "缩进 3 格的围栏照认": (f"   ```\n草稿目录：{other}\n   ```\n草稿目录：{mine}\n", [mine]),
        "缩进 4 格的收尾不算收尾，围栏一直到文末": (f"```\n草稿目录：{other}\n    ```\n草稿目录：{other}\n", []),
    }
    relative_to_root = os.path.relpath(mine, repository_root())
    probe_under_root = os.path.normpath(os.path.join(repository_root(), "b12-relative-probe"))
    probe_expected = [probe_under_root] if any(probe_under_root.startswith(known.rstrip("/") + "/") for known in work_roots.scratch_roots()) else []
    here = os.getcwd()
    try:
        os.chdir("/")
        relative_from_outside = scratch_directories(f"草稿目录：{relative_to_root}\n")
        os.chdir(root)
        relative_from_work_root = (scratch_directories("草稿目录：b12-relative-probe\n"), scratch_directories(f"草稿目录：{relative_to_root}\n"))
    finally:
        os.chdir(here)
    cells = [
        ("骨架那一行（全角冒号）认", scratch_line_values(f"# 派发\n\n草稿目录：{mine}\n报告：{mine}/result.md\n"), [mine]),
        ("半角冒号、冒号前后的空白也认", scratch_line_values(f"草稿目录 : {mine}/\n"), [f"{mine}/"]),
        ("行首的空格与制表符之后照认", scratch_line_values(f"  \t草稿目录：{mine}\n"), [mine]),
        ("反引号包着的路径认、后面的括注截在括号前", scratch_line_values(f"草稿目录：`{mine}`（自己建，排他）\n"), [mine]),
        ("路径末尾的句号与逗号去掉", scratch_line_values(f"草稿目录：{mine}.\n"), [mine]),
        ("<填 …> 占位不认", scratch_line_values("草稿目录：<填 草稿目录>\n"), []),
        ("转述里「，草稿目录 <别处>」不认",
         scratch_line_values(f"草稿目录：{mine}\n前一段交回在先，草稿目录 {other}/ 里有它的探针，报告 {other}/report.md 是那一段写的。\n"), [mine]),
        ("列表里「- 草稿目录 <别处>」不认", scratch_line_values(f"草稿目录：{mine}\n要读的：\n- 草稿目录 {other}/ 下的 logs/\n"), [mine]),
        ("列表与粗体起头的那一行不认", scratch_line_values(f"- 草稿目录：{other}\n**草稿目录**：{other}\n> 草稿目录：{other}\n"), []),
        ("「草稿目录在 <路径>」不是骨架那一行，不认", scratch_line_values(f"草稿目录在 {mine}（自己建）\n"), []),
        ("闭合了的围栏代码块里的那一行不认", scratch_line_values(f"草稿目录：{mine}\n示例：\n```\n草稿目录：{other}\n```\n"), [mine]),
        *[(label, scratch_line_values(text), want) for label, (text, want) in fence_samples.items()],
        ("相对路径按仓根解，不按调用时的当前目录（当前目录在仓外）", relative_from_outside, [mine]),
        ("相对路径按仓根解，不按调用时的当前目录（当前目录在工作根）", relative_from_work_root, (probe_expected, [mine])),
        ("几行都认、按次序去重", scratch_line_values(f"草稿目录：{mine}\n草稿目录：{other}\n草稿目录：{mine}\n"), [mine, other]),
        ("~ 写法展开成绝对路径", scratch_directories(f"草稿目录：{written_root}/b10-selftest-mine/\n"), [mine]),
        ("过渡期旧根底下的认", scratch_directories(f"草稿目录：{legacy}/b10-selftest-legacy\n"), [f"{legacy}/b10-selftest-legacy"]),
        ("工作根本身不算草稿目录", scratch_directories(f"草稿目录：{root}/\n"), []),
        ("工作根本身在 include_roots 下算", scratch_directories(f"草稿目录：{root}/\n", include_roots=True), [root]),
        ("工作根与旧根之外的目录不算", scratch_directories("草稿目录：/var/b10-selftest-elsewhere\n"), []),
    ]
    failures = [(label, want, got) for label, got, want in cells if got != want]
    reference = fence_lines_by_reference(fence_samples)
    if reference is not None:
        failures += [(f"围栏行与 markdown-it-py 的 commonmark 预设认的一致（{label}）", want, got) for label, want, got in reference]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 scratch_line_values() 与 scratch_directories()（认法写在文件头「认法」一段）；LIB_PROMPT_SCRATCH_LINE_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_prompt_scratch_line 自检通过（查了 {len(cells) + (0 if reference is None else len(fence_samples))} 格）：只认行首「草稿目录：」一行（全角半角冒号、反引号、末尾标点都照认），"
          "转述里的、列表与粗体起头的、「草稿目录在」、占位、围栏代码块（照 CommonMark，没收尾的到文末）里的都不认；展开 ~ 之后写相对路径的按仓根解，"
          "只留工作根与过渡期旧根底下的，根本身不算" + ("" if reference is not None else "；本机没装 markdown-it-py，与它比围栏行的那几格没查"))
    return 0


def main(arguments):
    value = os.environ.get(BREAK_VARIABLE, "")
    if value and arguments != ["--selftest"]:
        print(f"✗ lib_prompt_scratch_line.py：{BREAK_VARIABLE}={value} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒", file=sys.stderr)
        print(f"→ 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 python3 {os.path.abspath(__file__)} --selftest", file=sys.stderr)
        return 2
    if arguments == ["--selftest"]:
        return selftest()
    print("用法：lib_prompt_scratch_line.py --selftest（这一份是共用库，平常由钩子 import，不单独跑）", file=sys.stderr)
    print("→ 怎么办：要核认法就跑 --selftest；认法写在文件头「认法」一段", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.exit(main(sys.argv[1:]))
