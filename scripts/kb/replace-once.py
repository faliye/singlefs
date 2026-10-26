#!/usr/bin/env python3
# admission: always 每一次调都是对此刻那份文件做一次定点替换，上一次的结论不替这一次作保
# run-condition: none 只读写被点名的那一份文件，除了 python3 之外没有环境要求
"""定点替换：旧串在文件里必须恰好命中一次，0 次或多次都拒绝、一个字节不写。

用法：
    replace-once.py 文件 旧串 新串        # 旧串、新串按字面给，不是正则；命中恰好一次才写
    replace-once.py --selftest            # 自证：0 次、2 次都拒绝，1 次写成功且写回读得出；换上的是新 inode

为什么要有它：几个会话共写一批 kb 文件时只许定点改、不许整份重写（`.claude/singlefs-ai-sop/rules/session-wrapup.md`），
而 `sed -i` 匹配不上不报错、匹配多处全改，都是静默失效（`rules/command-safety.md`「脚本改文件之后要回读确认」）。
这一段「count == 1 才 replace」的 python 在 2026-09-13 一天里手敲了二十多遍，按 session-wrapup 的判据（会不会被抄第二遍）收进这里。

写回不在原文件上就地写：同目录排他新建临时文件、fsync、照原权限位与属主、改名换上（`lib_atomic_replace.py`）。
正在按偏移边跑边读这份文件的进程（bash 跑脚本）读的仍是旧 inode 的旧内容，不会从改后文件的同一偏移接着读到别的东西。

退出码：0 写成功；2 参数或文件问题；3 旧串命中 0 次；4 旧串命中多于一次；5 写完回读与预期不符；
6 没换上（有多个硬链接、建不了临时文件、写或改名失败），原文件没动、临时文件已删；
7 目标是仓里的 .md，新串带钟点、时区词或时间戳（含运行目录名、日志文件名里连写的紧凑写法），一个字节没写；
8 目标在仓里（任何后缀），新串带会话号（ListAgents 列的会话名），一个字节没写。
命中 0 次时附上文件里最像旧串的那一段（行号、相似度）与第一处不同（旧串在那里是什么、文件里是什么），算法在 nearest_hint()。

写 .md 之前判新串（退出码 7）：目标是仓里的 .md 时，旧串恰好命中一次之后、写之前，用 write-guard.sh 与门禁 doc-text 同一份判据
（.claude/gate.d/lib-forbidden-notations.py 的钟点形态，lib_notation_check.py 调它）判新串；带钟点、时区词、时间戳的拒绝，逐处列出行号与形态与出路。
判新串不判旧串；别的后缀、仓外、排除前缀下的文件不判。拦不住的写法（shell 重定向、heredoc、python 的 open）靠提交时门禁 doc-text 的 clock-times 格兜底。

写仓里的文件之前判新串里的会话号（退出码 8）：旧串恰好命中一次之后、写之前，用 write-guard.sh 的第七道与门禁 doc-text 的 session-names 格同一份判据
（.claude/gate.d/lib-forbidden-notations.py 的会话号形态，lib_notation_check.py 的 session_name_refusal 调它，三份定点写入脚本共用）判新串；
带会话名的拒绝，逐处列出行号与出路（写成「别的会话」这类不带号的说法）。后缀不限，范围与写闸判 Write、Edit 一致：排除前缀、.sh .py .rs 里带
「# session-names:allow <理由>」的一行、仓外不判。拦不住的写法（shell 重定向、heredoc、python 的 open）靠提交时门禁 doc-text 的 session-names 格兜底。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证红的那几格（「」里是失败行的开头：自证逐条以 ✗ 起头列出判错的格，末尾一句「✗ --selftest 没通过：N 项」是汇总）：
  REPLACE_ONCE_BREAK=no-nearest  nearest_hint() 什么都不给（改前的说法）
      「✗ 命中 0 次时本该附上最像的一行」
  REPLACE_ONCE_BREAK=no-clock-guard  replace_once() 不判新串里的钟点（改前的写法）
      「✗ 写 .md：新串带紧凑时间戳本该退出码 7」、「✗ 写 .md：新串带紧凑时间戳被拒绝之后文件或目录变了」
      「✗ 写 .md：新串带日期、时区词、时分本该退出码 7」、「✗ 写 .md：新串带日期、时区词、时分被拒绝之后文件或目录变了」
      「✗ 写 .md：新串带光秃的时区词本该退出码 7」、「✗ 写 .md：新串带光秃的时区词被拒绝之后文件或目录变了」
      「✗ 写 .md：8 位数字后跟 T 但不是时间戳的新串本该放行」（前面那几格写进去之后旧串找不到了，连带红）
      「✗ 写 .md：假仓里没有共用库，本该退出码 7」（不判钟点就走到会话号那一道，退 8）
      「✗ 写 .md：入口本该退出码 7、打 ✗ 开头的说明且不写文件」
  REPLACE_ONCE_BREAK=no-session-guard  replace_once() 不判新串里的会话号（改前的写法）
      写之前判新串里的会话号，逐格以「✗ 写会话号：」起头（.md 与 .py 目标的新串带会话名、干跑、入口退出码 8、假仓里没有共用库与不写文件）
  LIB_FORBIDDEN_NOTATIONS_BREAK=<判据行键或形态键>  共用库的弄坏开关（写法在 lib-forbidden-notations.py 文件头），自证同样走它：
      clock-times（整个钟点形态不判）时写 .md 的拒绝几条全红；iso-compact-timestamp 时只有新串带紧凑时间戳的那几条红；
      compact-timestamp-loose 时「8 位数字后跟 T 但不是时间戳」的放行那一条红；
      session-names（整个会话号形态不判）时「写会话号：」的拒绝几条全红；LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=session-names 时带放行标记的那一条红，
      LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=session-names 时排除前缀下放行的那一条红
"""
import contextlib
import difflib
import io
import os
import sys
import tempfile

from lib_atomic_replace import ReplaceRefused, problems_with_replacement_by_rename, replace_file_contents_by_rename
from lib_notation_check import clock_refusal, copy_library_into, session_guard_problems, session_name_refusal
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402


def shown_piece(piece):
    """说明里给人看的一小段：空的、只有空白的写成 repr，别的原样（超长截断）。"""
    if piece.strip() == '':
        return repr(piece)
    return piece if len(piece) <= 40 else piece[:40] + '…'


def nearest_hint(content, old_text):
    """旧串命中 0 次时，文件里最像它的那一段：→ [说明行]（已带缩进），文件是空的返回 []。
    相似度：旧串有几行，就拿同样几行的窗口逐行滑过整份文件；每一窗算旧串里有几个字按次序出现在这一窗里（difflib.SequenceMatcher
    的 matching blocks 之和），占旧串的比例最高的一窗算最像（同分取 ratio 高的、再同取靠前的），旧串只是长行里的一截时也认得出。
    再按它的 opcodes 找出第一处不同（跳过这一窗在旧串两头之外多出来的字），写出旧串在那里是什么、文件里是什么。
    replace-batch.py 命中 0 次时调的也是这一份。"""
    if os.environ.get('REPLACE_ONCE_BREAK') == 'no-nearest':   # 弄坏开关：改前的说法，不附最像的一行
        return []
    target = old_text.strip('\n')
    if not target:
        return []
    lines = content.split('\n')
    width = target.count('\n') + 1
    best = None
    for index in range(max(1, len(lines) - width + 1)):
        window = '\n'.join(lines[index:index + width])
        matcher = difflib.SequenceMatcher(None, target, window, autojunk=False)
        # quick_ratio 给的是按字数（不管次序）能对上的上界：上界都比不过已有的最好那一窗就不细算
        if best and matcher.quick_ratio() * (len(target) + len(window)) / 2 / len(target) < best[0]:
            continue
        covered = sum(block.size for block in matcher.get_matching_blocks()) / len(target)
        score = (covered, matcher.ratio())
        if best is None or score > best[:2]:
            best = (covered, score[1], index, window, matcher)
    if best is None or best[0] == 0:
        return []
    covered, _ratio, index, window, matcher = best
    span = f'第 {index + 1} 行' if width == 1 else f'第 {index + 1}-{index + width} 行'
    shown = window if len(window) <= 200 else window[:200] + '…'
    hint = [f'     文件里最像的一段在{span}（旧串的 {covered:.0%} 按次序出现在这一段里）：{shown}']
    for tag, old_start, old_end, file_start, file_end in matcher.get_opcodes():
        if tag == 'equal' or (tag == 'insert' and old_start in (0, len(target))):
            continue
        if tag == 'delete':
            difference = f'旧串多了「{shown_piece(target[old_start:old_end])}」（文件里这里没有）'
        elif tag == 'insert':
            difference = f'文件里这里多了「{shown_piece(window[file_start:file_end])}」（旧串里没有）'
        else:
            difference = f'旧串是「{shown_piece(target[old_start:old_end])}」，文件里是「{shown_piece(window[file_start:file_end])}」'
        hint.append(f'     第一处不同在旧串第 {old_start + 1} 个字起：{difference}')
        break
    return hint


def replace_once(path, old_text, new_text, notation_root=None):
    """返回 (退出码, 说明)。只有退出码 0 时文件被改过。notation_root 只给自证传（判新串时把哪个目录当仓根）。"""
    if old_text == '':
        return 2, '旧串是空串，拒绝：空串在每个位置都命中\n     → 把旧串换成文件里真实存在、唯一定位那处的一段文字'
    if not os.path.isfile(path):
        return 2, f'找不到文件 {path}\n     → 检查路径是否写对（相对路径相对当前工作目录），文件是否已经被别的进程删掉或改名'
    with open(path, encoding='utf-8') as handle:
        content = handle.read()
    occurrences = content.count(old_text)
    if occurrences == 0:
        hint = ''.join(f'\n{line}' for line in nearest_hint(content, old_text))
        return 3, (f'旧串在 {path} 里命中 0 次，一个字节没写{hint}\n'
                   '     → 先用 grep 找到那一行，把旧串按文件里的原样（含空格与标点）抄进来')
    if occurrences > 1:
        return 4, f'旧串在 {path} 里命中 {occurrences} 次，一个字节没写\n     → 把旧串往前后多带几个字，直到只命中一次'
    if os.environ.get('REPLACE_ONCE_BREAK') != 'no-clock-guard':   # 弄坏开关 no-clock-guard：不判新串里的钟点
        refusal = clock_refusal(path, new_text, content[:content.index(old_text)].count('\n') + 1, notation_root)
        if refusal:
            return 7, '\n'.join(refusal)
    if os.environ.get('REPLACE_ONCE_BREAK') != 'no-session-guard':   # 弄坏开关 no-session-guard：不判新串里的会话号
        refusal = session_name_refusal(path, new_text, content[:content.index(old_text)].count('\n') + 1, notation_root)
        if refusal:
            return 8, '\n'.join(refusal)
    replaced = content.replace(old_text, new_text, 1)
    try:
        notes = replace_file_contents_by_rename(path, replaced)
    except ReplaceRefused as refused:
        return 6, str(refused)
    with open(path, encoding='utf-8') as handle:
        written = handle.read()
    if written != replaced or written.count(new_text) < 1:
        return 5, f'写完回读 {path} 与预期不一致\n     → 检查文件是不是被别的进程同时改了'
    return 0, f'{path}：命中 1 次，已替换（改名换上新 inode）并回读确认' + ''.join(f'\n  ! {note}' for note in notes)


def selftest():
    with tempfile.TemporaryDirectory(prefix='replace-once-selftest-') as workspace:
        return selftest_in(workspace)


def problems_with_clock_guard(workspace):
    """写 .md 之前判新串：钟点、时区词、时间戳。样本串在这里拆开拼，这份源码里不出现字面写法（门禁 doc-text 也扫它）。
    仓根用自证的临时目录顶替（notation_root）：假仓里抄一份真的共用库（copy_library_into）；另造一个没有库的假仓核「读不到判据」那一支。"""
    problems = []
    glue = ''.join
    colon = ':'
    zone = glue(('J', 'ST'))
    stamp = glue(('20261014', 'T153000Z'))
    root = os.path.join(workspace, 'repo')
    os.makedirs(os.path.join(root, 'records'))
    copy_library_into(root)
    notes = os.path.join(root, 'records', 'notes.md')
    original = '第一行 甲\n第二行 乙\n'

    def reset(path=notes):
        with open(path, 'w', encoding='utf-8') as handle:
            handle.write(original)

    def content_of(path=notes):
        with open(path, encoding='utf-8') as handle:
            return handle.read()

    reset()
    refused = [
        ('新串带紧凑时间戳', glue(('运行目录 ', stamp)), '（ISO 紧凑时间戳）：「' + stamp + '」'),
        ('新串带日期、时区词、时分', glue(('用户 2026-10-14 ', zone, ' 15', colon, '30 定')), '（日期后面跟钟点）'),
        ('新串带光秃的时区词', glue(('本机时钟是 ', zone)), '（时区词）：「' + zone + '」'),
    ]
    for label, new_text, wanted in refused:
        code, message = replace_once(notes, '第二行 乙', new_text, notation_root=root)
        if code != 7 or wanted not in message or '第 2 行' not in message or '→ 怎么办' not in message or '读不到钟点判据' in message:
            problems.append(f'写 .md：{label}本该退出码 7、点名形态「{wanted}」与第 2 行与出路，实际 {code}：{message!r}'
                            '（REPLACE_ONCE_BREAK=no-clock-guard 或 LIB_FORBIDDEN_NOTATIONS_BREAK 关掉那一形态的话这一条本来就该红）')
        if content_of() != original or os.listdir(os.path.dirname(notes)) != ['notes.md']:
            problems.append(f'写 .md：{label}被拒绝之后文件或目录变了（一个字节都不该写、不该留临时文件）')
    code, message = replace_once(notes, '第二行 乙', glue(('编号 99999999', 'T1234')), notation_root=root)
    if code != 0 or content_of() != '第一行 甲\n编号 99999999' + 'T1234\n':
        problems.append(f'写 .md：8 位数字后跟 T 但不是时间戳的新串本该放行（退出码 0），实际 {code}：{message!r}'
                        '（LIB_FORBIDDEN_NOTATIONS_BREAK=compact-timestamp-loose 设着的话这一条本来就该红）')
    reset()
    code, message = replace_once(notes, '第二行 乙', '第二行 用户 2026-10-14 定', notation_root=root)
    if code != 0 or content_of() != '第一行 甲\n第二行 用户 2026-10-14 定\n':
        problems.append(f'写 .md：只写日期的新串本该放行（退出码 0），实际 {code}：{message!r}')
    # 旧串里带钟点、新串干净（把钟点删掉的改动）放行：判的是新串
    dirty = os.path.join(root, 'records', 'dirty.md')
    with open(dirty, 'w', encoding='utf-8') as handle:
        handle.write(glue(('用户 2026-10-14 ', zone, ' 15', colon, '30 定\n')))
    code, message = replace_once(dirty, glue(('2026-10-14 ', zone, ' 15', colon, '30 定')), '2026-10-14 定', notation_root=root)
    if code != 0 or content_of(dirty) != '用户 2026-10-14 定\n':
        problems.append(f'写 .md：旧串带钟点、新串只写日期（删掉钟点的改动）本该放行，实际 {code}：{message!r}')
    # 范围：别的后缀与仓外不判（带紧凑时间戳的新串照写）
    for label, path in (('.txt 目标', os.path.join(root, 'records', 'notes.txt')), ('仓外的 .md', os.path.join(workspace, 'outside.md'))):
        reset(path)
        code, message = replace_once(path, '第二行 乙', glue(('运行目录 ', stamp)), notation_root=root)
        if code != 0:
            problems.append(f'写 .md：{label}不归这一道判，本该放行（退出码 0），实际 {code}：{message!r}')
    # 读不到共用库按拒绝处理，不静默放行：假仓里没有库，新串干净也拒，理由是读不到判据（不是钟点）
    bare = os.path.join(workspace, 'repo-without-library')
    os.makedirs(os.path.join(bare, 'records'))
    bare_notes = os.path.join(bare, 'records', 'notes.md')
    reset(bare_notes)
    code, message = replace_once(bare_notes, '第二行 乙', '第二行 丙', notation_root=bare)
    if code != 7 or not message.startswith('读不到钟点判据 ') or 'lib-forbidden-notations.py' not in message \
            or 'FileNotFoundError' not in message or '→ 怎么办' not in message or content_of(bare_notes) != original:
        problems.append(f'写 .md：假仓里没有共用库，本该退出码 7、说「读不到钟点判据」点名库文件与 FileNotFoundError 与出路且不写文件，实际 {code}：{message!r}')
    # 入口：退出码与输出
    reset()
    captured = io.StringIO()
    with contextlib.redirect_stdout(captured):
        code = main([notes, '第二行 乙', glue(('运行目录 ', stamp))], notation_root=root)
    if code != 7 or not captured.getvalue().startswith('  ✗ 新串在描述里写了钟点、时区词或时间戳，共 1 处') or content_of() != original:
        problems.append(f'写 .md：入口本该退出码 7、打 ✗ 开头的说明且不写文件，实际 {code}：{captured.getvalue()!r}')
    return problems


def problems_with_session_guard(workspace):
    """写之前判新串里的会话号：样本与判法在 lib_notation_check.py 的 session_guard_problems（三份脚本共用），这里只接上 replace_once 与入口。"""
    def attempt(path, new_text, root, dry_run):
        return replace_once(path, '第二行 乙', new_text, notation_root=root)

    def entry(path, new_text, root):
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            code = main([path, '第二行 乙', new_text], notation_root=root)
        return code, captured.getvalue()
    return session_guard_problems(workspace, attempt, 8, entry)


def selftest_in(workspace):
    sample = os.path.join(workspace, 'sample.md')
    with open(sample, 'w', encoding='utf-8') as handle:
        handle.write('第一行 甲\n第二行 乙\n第三行 乙\n')
    problems = []
    code, _ = replace_once(sample, '丙', '丁')
    if code != 3:
        problems.append(f'命中 0 次本该退出码 3，实际 {code}')
    code, message = replace_once(sample, '第二行 丙', '丁')
    if code != 3 or '最像的一段在第 2 行' not in message or '旧串是「丙」，文件里是「乙」' not in message:
        problems.append(f'命中 0 次时本该附上最像的一行（第 2 行）与第一处不同（「丙」对「乙」），实际退 {code}：{message!r}'
                        '（REPLACE_ONCE_BREAK=no-nearest 设着的话这一条本来就该红）')
    code, _ = replace_once(sample, '乙', '丁')
    if code != 4:
        problems.append(f'命中 2 次本该退出码 4，实际 {code}')
    with open(sample, encoding='utf-8') as handle:
        if handle.read() != '第一行 甲\n第二行 乙\n第三行 乙\n':
            problems.append('被拒绝的替换改了文件')
    code, _ = replace_once(sample, '甲', '戊')
    if code != 0:
        problems.append(f'命中 1 次本该退出码 0，实际 {code}')
    with open(sample, encoding='utf-8') as handle:
        if handle.read() != '第一行 戊\n第二行 乙\n第三行 乙\n':
            problems.append('命中 1 次的替换没写对')
    code, _ = replace_once(sample, '', '戊')
    if code != 2:
        problems.append(f'空旧串本该退出码 2，实际 {code}')
    problems.extend(problems_with_clock_guard(workspace))
    problems.extend(problems_with_session_guard(workspace))
    # 写回的方式：inode 要换、在读的进程读旧内容、权限位与符号链接保住、硬链接拒绝、失败不留临时文件（判据在 lib_atomic_replace.py）
    running_script = '#!/usr/bin/env bash\n' + 'echo 前面这几行正在被 bash 边跑边读\n' * 20 + 'run_step 甲\nexit $?\n'
    problems.extend(problems_with_replacement_by_rename(
        lambda path: replace_once(path, 'run_step 甲', 'run_step 戊')[0],
        running_script, running_script.replace('run_step 甲', 'run_step 戊', 1)))
    if os.environ.get('REPLACE_ONCE_CORRUPT') == '1':
        # 测试缝：强制走「多次命中也写」那条旧毛病，自检必须判红
        problems.append('REPLACE_ONCE_CORRUPT=1：强制走旧毛病（多次命中也写）')
    if problems:
        for problem in problems:
            print(f'  ✗ {problem}')  # gate-lint:detail
        print(f'  ✗ --selftest 没通过：{len(problems)} 项（上面逐条列着）')
        print('     → 怎么办：看 replace_once 里对应分支的退出码与是否写文件；方括号开头的几条看 lib_atomic_replace.py 里同名那一格')
        return 1
    print('  ✓ --selftest 通过：命中 0 次拒绝（3）并附上最像的一行与第一处不同、2 次拒绝（4）、空串拒绝（2），都一个字节没写；命中 1 次写成功并回读确认；'
          '写仓里的 .md 之前判新串：带紧凑时间戳、带日期时区词时分、带光秃时区词的拒绝（7）并点名形态、行号与出路且不写文件，'
          '8 位数字后跟 T 但不是时间戳、只写日期、旧串带钟点而新串干净、.txt 目标与仓外的 .md 放行，假仓里没有共用库时读不到判据按拒绝办（退出码 7，理由不是钟点），入口退出码 7；'
          '写仓里任何后缀的文件之前判新串里的会话号：.md 与 .py 目标带会话名的拒绝（8）并点名行号与出路「别的会话」且不写文件，干净的新串、.py 里带放行标记的一行、排除前缀下的文件、仓外的文件放行，'
          '假仓里没有共用库时读不到会话号判据按拒绝办（退出码 8），入口退出码 8；'
          '换上的是新 inode、权限位不变、改之前打开文件的读者接着读到旧内容、fsync 在改名之前、'
          '经符号链接改的是它指向的文件、有两个硬链接或当前用户只读都拒绝、fsync 或改名失败都不动原文件也不留临时文件')
    return 0


def main(arguments, notation_root=None):
    if arguments == ['--selftest']:
        return selftest()
    if len(arguments) != 3:
        print(__doc__)
        return 2
    path, old_text, new_text = arguments
    code, message = replace_once(path, old_text, new_text, notation_root=notation_root)
    print(('  ✓ ' if code == 0 else '  ✗ ') + message)
    # → 怎么办：message 本身已经带着下一步（见 replace_once 各分支的返回值），这里不重复打印
    return code


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
