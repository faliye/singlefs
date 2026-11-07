#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
"""生成三方论证材料要带的那张小节清单——**含第一个标题之前的引言段**。

`.claude/rules/three-way-inference.md` 要求材料带一张小节清单，逐节标「抄 / 不抄 /
为什么不抄」，好让漏掉的那一节留下可见的空格。按 awk 抓 `^#{2,5} `
生成的清单里，**文档开头那段正文没有自己的行**：它住在 H2 标题之下、第一个 H3 之前，
在清单里只表现为那条 H2 标题行。而 H2 标题行看着就是个标题，标「不抄」几乎不需要理由,
一标下去，那段正文连同它一起无声消失。

⇒ 本脚本给那段正文**单独发一行**，标出它的行区间，让它必须被单独判一次抄不抄。
⚠️ 射程只有开头这一段：别的标题的正文仍然由它自己那一行代表。

**一级标题同样算标题**：只认二到六级的话，顶上是一级标题的文件——
`checks-owed.md`、`crates/singlefs-checker/invariants/README.md`、`layout/01-first-txn.md`——那个一级标题连同它下面、第一个
下级标题之前的正文**一行都不发**。`checks-owed.md` 的整张欠账表（第 2-243 行）就住在那里，
清单上根本看不见它，也就没有地方写「不抄，因为……」。
自证会红：`KB_SECTIONS_NO_H1=1` 让它只认二到六级，自检里那一行必须消失。

例（D6（快照实现模型） 未定项 2 的枢纽问题）：
`.claude/kb/decisions/05-快照-空间记账机制.md` 的第 1-14 行是整个模型的定义段，
逐字写着「销毁快照时把它的 deadlist 合并到下一个更新的那一侧……**不做全盘扫描**」。
它若无声消失，而枢纽问题正是「销毁快照是不是一次全盘遍历」，
材料就会把一个假前提发给三条腿。

用法：

    kb-sections.py 文件.md [文件.md ...]      # 打印清单，抄 / 不抄两列留空待填
    kb-sections.py --help                      # 打这一段用法（-h 同），退 0
    kb-sections.py --selftest

供别的脚本 import 的函数：heading_bounds(path) → ([(行号, 级别, 原文)], 总行数)，标题的认法与清单同一份（栅栏里的行不算）；quote-kb.py 用它另算一次 @标题 取法的起止行（C264）。

退出码：0 打出清单（或 --help）；2 没给文件、不认的选项、文件不存在或读不了（stderr 写哪一份、怎么办，不抛 traceback），
或平常跑设了下面四个弄坏开关里的任何一个（stderr 点名它只在自证里认，什么都不打）。

自证会红：`kb-sections.py --selftest` 造一份「标题之前有正文」的样本，断言清单里有那一行；一级标题、栅栏、heading_bounds、命令行各一格。
弄坏开关（只给 --selftest 证红用；四个变量同一道入口守卫 settle_break_switches()：平常跑设了任何一个就退 2，被别的脚本当模块读进来时
三个 NO_ 开关经 selftest_switch() 读，不在自证里一律不认，所以 quote-kb.py 平常跑时节边界不受它们左右）：
  KB_SECTIONS_BREAK=raw-arguments  main() 不先判 --help、不认的选项与读不了的文件
      命令行那一格（「selftest: 命令行 ['--help'] 应当退 0」）
  KB_SECTIONS_BREAK=old-switches-unguarded  三个 NO_ 开关不进入口守卫、平常跑也认
      开关守卫那一格（「selftest: 平常跑设了 KB_SECTIONS_NO_H1」）
  KB_SECTIONS_NO_PREAMBLE=1  sections() 不给开头那段正文单独发一行
      开头正文那一格（「selftest: 清单里没有开头正文那一行」）
  KB_SECTIONS_NO_H1=1  is_head() 只认二到六级标题
      一级标题那一格（「selftest: 一级标题之下的正文没有自己的行」）
  KB_SECTIONS_NO_FENCE=1  sections() 与 heading_bounds() 不跳代码栅栏
      栅栏那一格（「selftest: 代码栅栏里的 `## ` 行被当成标题发了一行」）
"""
import contextlib, io, os, re, subprocess, sys, tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

HEAD = re.compile(r'^(#{1,6}) .*$')
HEAD_NO_H1 = re.compile(r'^(#{2,6}) .*$')


def fenced_lines(lines):
    """返回落在代码栅栏里的行号集合（1 起）。栅栏里的 `## ` 不是标题。"""
    inside, fence = set(), None
    for i, l in enumerate(lines, 1):
        fm = re.match(r' {0,3}(`{3,}|~{3,})', l)
        if fence is None and fm and not (fm.group(1)[0] == '`' and '`' in l[fm.end():]):
            fence = (fm.group(1)[0], len(fm.group(1))); inside.add(i); continue
        if fence is not None:
            inside.add(i)
            cm = re.fullmatch(r' {0,3}(`{3,}|~{3,})\s*', l)
            if cm and cm.group(1)[0] == fence[0] and len(cm.group(1)) >= fence[1]:
                fence = None
    return inside


SELFTEST_CONTEXT = []   # 这个进程在不在自证里，第一次问时算一次（往上数祖先要读 /proc，不逐行重算）


def selftest_switch(name, in_selftest=None):
    """三个 NO_ 弄坏开关的读法：变量设了、而且这一趟在自证里才认。
    在不在自证里：in_selftest 给了就照它；没给就看本进程的命令行以 --selftest 起头（quote-kb.py --selftest 把本文件当模块读进来时也算），
    或祖先里有自证（inside_selftest）。弄坏开关 KB_SECTIONS_BREAK=old-switches-unguarded 下三个 NO_ 开关设了就认。"""
    if not os.environ.get(name):
        return False
    if os.environ.get('KB_SECTIONS_BREAK') == 'old-switches-unguarded':
        return True
    if in_selftest is not None:
        return in_selftest
    if not SELFTEST_CONTEXT:
        SELFTEST_CONTEXT.append(sys.argv[1:2] == ['--selftest'] or inside_selftest())
    return SELFTEST_CONTEXT[0]


def is_head(i, l, fenced):
    head = HEAD_NO_H1 if selftest_switch('KB_SECTIONS_NO_H1') else HEAD
    return i not in fenced and head.match(l)


def sections(path):
    """返回 [(标签, 首行号)]；开头那段正文单独占一行，紧跟在它所属的标题之后。"""
    lines = io.open(path, encoding='utf-8').read().split('\n')
    fenced = set() if selftest_switch('KB_SECTIONS_NO_FENCE') else fenced_lines(lines)
    heads = [i for i, l in enumerate(lines, 1) if is_head(i, l, fenced)]
    out = []
    for i, l in enumerate(lines, 1):
        if not is_head(i, l, fenced):
            continue
        out.append((l.replace('|', '/'), i))
        if selftest_switch('KB_SECTIONS_NO_PREAMBLE') or i != heads[0]:
            continue
        nxt = next((h for h in heads if h > i), len(lines) + 1)
        body = lines[i:nxt - 1]
        if any(x.strip() for x in body):
            out.append((f'（{l.split()[1] if len(l.split()) > 1 else ""} 标题之下、'
                        f'第一个下级标题之前的正文：第 {i + 1}-{nxt - 1} 行）', i + 1))
    return out


def heading_bounds(path):
    """→ ([(行号, 级别, 原文)], 总行数)：每个标题一项，行号 1 起，级别是 `#` 的个数。
    标题的认法与 sections() 同一份（is_head、fenced_lines），所以栅栏里的 `#` 行不算；总行数是整份文本按换行切开的段数（文件以换行收尾时最后一段是空串，也算一行），与 quote-kb.py 的 read() 数法一致。
    给 quote-kb.py 另算一次 `@标题` 取法的起止行用（C264）：节从标题起，到下一个级别不更深的标题之前，没有就到文件末。"""
    lines = io.open(path, encoding="utf-8").read().split("\n")
    fenced = set() if selftest_switch("KB_SECTIONS_NO_FENCE") else fenced_lines(lines)
    rows = []
    for i, l in enumerate(lines, 1):
        found = is_head(i, l, fenced)
        if found:
            rows.append((i, len(found.group(1)), l))
    return rows, len(lines)


def render(paths):
    buf = []
    for p in paths:
        buf.append(f'### 小节清单：`{p}`（{os.path.basename(__file__)} 全量生成，未经任何过滤）')
        buf.append('')
        buf.append('| 小节 | 抄 / 不抄 | 理由 |')
        buf.append('|---|---|---|')
        for title, _ in sections(p):
            buf.append(f'| {title} | | |')
        buf.append('')
    return '\n'.join(buf)


def selftest_red(problem):
    """自证判错的那一格：打一行以 ✗ 起头的失败行与出路，交回退出码 1。"""
    print(f'  ✗ selftest: {problem}')
    print('     → 怎么办：看那一格测的是哪一支（开头正文、一级标题、栅栏、heading_bounds），对着 sections() / heading_bounds() 修；'
          'KB_SECTIONS_NO_PREAMBLE、KB_SECTIONS_NO_H1、KB_SECTIONS_NO_FENCE 设着的话对应那一格本来就该红')
    return 1


def selftest():
    with tempfile.TemporaryDirectory() as d:
        f = os.path.join(d, 'sample.md')
        io.open(f, 'w', encoding='utf-8').write(
            '## D99 样本 —— 已定\n\n这一段在第一个三级标题之前，是承重的定义段。\n\n### 甲节\n\n正文。\n')
        rows = [t for t, _ in sections(f)]
        has_pre = any('之前的正文' in r for r in rows)
        if not has_pre:
            return selftest_red('清单里没有开头正文那一行 —— 正是本脚本要防的形态')
        h = os.path.join(d, 'h1.md')
        io.open(h, 'w', encoding='utf-8').write(
            '# 样本清单\n\n| C1 | 一整张表住在一级标题之下 |\n\n### 已还清\n\n正文。\n')
        hrows = [t for t, _ in sections(h)]
        has_h1 = any(r.startswith('（样本清单 标题之下') for r in hrows)
        if not has_h1:
            return selftest_red('一级标题之下的正文没有自己的行 —— 整张表会从清单上消失')
        g = os.path.join(d, 'fenced.md')
        io.open(g, 'w', encoding='utf-8').write(
            '## D98 样本 —— 已定\n\n### 甲节\n\n```bash\n## 这一行在栅栏里，不是标题\n```\n\n### 乙节\n')
        frows = [t for t, _ in sections(g)]
        if any('栅栏里' in r for r in frows):
            return selftest_red('代码栅栏里的 `## ` 行被当成标题发了一行')
        b = os.path.join(d, 'bounds.md')
        io.open(b, 'w', encoding='utf-8').write(
            '## 甲\n\n```bash\n# 像标题的注释\n```\n\n### 甲一\n\n正文\n\n## 乙\n\n末行\n')
        brows, btotal = heading_bounds(b)
        if brows != [(1, 2, '## 甲'), (7, 3, '### 甲一'), (11, 2, '## 乙')] or btotal != 14:
            return selftest_red(f'heading_bounds 给出的标题表不对（栅栏里的 # 行不算、级别是 # 的个数、总行数按换行切开数）——{brows} 共 {btotal} 行')
        # 命令行：--help / -h 打文件头的用法退 0；不认的选项退 2；读不了的文件（目录）报文件名与出路、不抛 traceback
        cli_cases = [(['--help'], 0, '用法：'), (['-h'], 0, '用法：'), (['--bogus'], 2, '不认的选项：--bogus'),
                     ([d], 2, f'读不了 {d}'), ([os.path.join(d, 'nothere.md')], 2, '这些文件不存在')]
        for arguments, want_code, want_text in cli_cases:
            child = subprocess.run([sys.executable, os.path.abspath(__file__), *arguments], capture_output=True, text=True)
            said = child.stdout + child.stderr
            if child.returncode != want_code or want_text not in said or 'Traceback' in said or (want_code and '→ 怎么办' not in said):
                print(f'  ✗ selftest: 命令行 {arguments} 应当退 {want_code}、说「{want_text}」（拒绝时带「→ 怎么办」、不抛 traceback），'
                      f'实际退 {child.returncode}：{said[-300:]}')
                print('     → 怎么办：看 main() 开头那几道判；KB_SECTIONS_BREAK=raw-arguments 设着的话这里本来就该红')
                return 1
        # 三个 NO_ 开关与 KB_SECTIONS_BREAK 同一道入口守卫：平常跑设了就退 2 并点名；被别的脚本当模块读进来、不在自证里时读法也不认。
        # 子进程的祖先里有这一趟自证、守卫会放行，所以这一格在本进程里问守卫与读法，在不在自证里由参数给死
        saved_no_h1 = os.environ.get('KB_SECTIONS_NO_H1')
        os.environ['KB_SECTIONS_NO_H1'] = '1'
        guard_said = io.StringIO()
        try:
            with contextlib.redirect_stderr(guard_said):
                refused = settle_break_switches(['文件.md'], in_selftest=False)
            honored = selftest_switch('KB_SECTIONS_NO_H1', in_selftest=False)
        finally:
            if saved_no_h1 is None:
                del os.environ['KB_SECTIONS_NO_H1']
            else:
                os.environ['KB_SECTIONS_NO_H1'] = saved_no_h1
        if refused != 2 or 'KB_SECTIONS_NO_H1' not in guard_said.getvalue() or '只在自证' not in guard_said.getvalue() or honored:
            print(f'  ✗ selftest: 平常跑设了 KB_SECTIONS_NO_H1，入口守卫应当退 2 并点名它只在自证里认、当模块读时读法也不认，'
                  f'实际守卫交回 {refused}、读法{"认了" if honored else "没认"}：{guard_said.getvalue()[-300:]}')
            print('     → 怎么办：看 BREAK_VARIABLES 与 selftest_switch()；KB_SECTIONS_BREAK=old-switches-unguarded 设着的话这里本来就该红')
            return 1
        print(f'selftest: 通过（{len(rows)} 行，开头那段正文单独占一行；一级标题下的正文单独占一行；栅栏里的 `## ` 不算标题；'
              f'命令行 {len(cli_cases)} 种：--help、-h 退 0，不认的选项、读不了的、不存在的退 2 带出路；'
              f'平常跑设了 NO_ 开关入口退 2、当模块读也不认）')
        return 0


def main(args):
    """命令行入口：→ 退出码。清单打到 stdout，拒绝打到 stderr。"""
    if not args:
        print(__doc__); return 2
    if args[0] == '--selftest':
        return selftest()
    guarded = os.environ.get('KB_SECTIONS_BREAK') != 'raw-arguments'   # 弄坏开关：--help、不认的选项、读不了的文件都不先判
    if guarded and args[0] in ('--help', '-h'):
        print(__doc__); return 0
    options = [p for p in args if p.startswith('-')]
    if guarded and options:
        print(f'kb-sections: 不认的选项：{" ".join(options)}', file=sys.stderr)
        print('  → 怎么办：只认 --help（-h）与 --selftest；要列小节就只给文件路径：kb-sections.py 文件.md [文件.md ...]', file=sys.stderr)
        return 2
    missing = [p for p in args if not os.path.exists(p)]
    if missing:
        print('kb-sections: 这些文件不存在：' + ' '.join(missing), file=sys.stderr)
        print('  → 怎么办：核对路径（相对路径相对当前工作目录，kb 文件一般写成 .claude/kb/…），或先 ls 一下那个目录。', file=sys.stderr)
        return 2
    try:
        text = render(args)
    except (OSError, UnicodeDecodeError) as error:
        if not guarded:
            raise
        print(f'kb-sections: 读不了 {getattr(error, "filename", None) or "其中一份文件"}（{error.__class__.__name__}：{error}）', file=sys.stderr)
        print('  → 怎么办：给的要是 UTF-8 的 markdown 文件（不是目录），核对路径再跑', file=sys.stderr)
        return 2
    print(text)
    return 0


# 入口守卫管的变量：KB_SECTIONS_BREAK 与三个 NO_ 开关（文件头「弄坏开关」）；弄坏开关 KB_SECTIONS_BREAK=old-switches-unguarded 下只管第一个
BREAK_VARIABLES = ("KB_SECTIONS_BREAK", "KB_SECTIONS_NO_H1", "KB_SECTIONS_NO_FENCE", "KB_SECTIONS_NO_PREAMBLE")


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments, in_selftest=None):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest；in_selftest 给了就照它，自证在本进程里问这一道时用）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    guarded = BREAK_VARIABLES[:1] if os.environ.get("KB_SECTIONS_BREAK") == "old-switches-unguarded" else BREAK_VARIABLES
    setting = [name for name in guarded if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or (inside_selftest() if in_selftest is None else in_selftest):
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/kb/kb-sections.py --selftest", file=sys.stderr)
    return 2


if __name__ == '__main__':
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
