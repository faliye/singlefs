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
6 没换上（有多个硬链接、建不了临时文件、写或改名失败），原文件没动、临时文件已删。
命中 0 次时附上文件里最像旧串的那一段（行号、相似度）与第一处不同（旧串在那里是什么、文件里是什么），算法在 nearest_hint()。

弄坏开关（只给 --selftest 证红用）：
  REPLACE_ONCE_BREAK=no-nearest  nearest_hint() 什么都不给（改前的说法）
      命中 0 次附最像的一行（「✗ --selftest 没通过：」下面「命中 0 次时本该附上最像的一行」那一条）
"""
import difflib
import os
import sys
import tempfile

from lib_atomic_replace import ReplaceRefused, problems_with_replacement_by_rename, replace_file_contents_by_rename
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


def replace_once(path, old_text, new_text):
    """返回 (退出码, 说明)。只有退出码 0 时文件被改过。"""
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
    # 写回的方式：inode 要换、在读的进程读旧内容、权限位与符号链接保住、硬链接拒绝、失败不留临时文件（判据在 lib_atomic_replace.py）
    running_script = '#!/usr/bin/env bash\n' + 'echo 前面这几行正在被 bash 边跑边读\n' * 20 + 'run_step 甲\nexit $?\n'
    problems.extend(problems_with_replacement_by_rename(
        lambda path: replace_once(path, 'run_step 甲', 'run_step 戊')[0],
        running_script, running_script.replace('run_step 甲', 'run_step 戊', 1)))
    if os.environ.get('REPLACE_ONCE_CORRUPT') == '1':
        # 测试缝：强制走「多次命中也写」那条旧毛病，自检必须判红
        problems.append('REPLACE_ONCE_CORRUPT=1：强制走旧毛病（多次命中也写）')
    if problems:
        print('  ✗ --selftest 没通过：')
        for problem in problems:
            print(f'     {problem}')
        print('     → 怎么办：看 replace_once 里对应分支的退出码与是否写文件；方括号开头的几条看 lib_atomic_replace.py 里同名那一格')
        return 1
    print('  ✓ --selftest 通过：命中 0 次拒绝（3）并附上最像的一行与第一处不同、2 次拒绝（4）、空串拒绝（2），都一个字节没写；命中 1 次写成功并回读确认；'
          '换上的是新 inode、权限位不变、改之前打开文件的读者接着读到旧内容、fsync 在改名之前、'
          '经符号链接改的是它指向的文件、有两个硬链接或当前用户只读都拒绝、fsync 或改名失败都不动原文件也不留临时文件')
    return 0


def main(arguments):
    if arguments == ['--selftest']:
        return selftest()
    if len(arguments) != 3:
        print(__doc__)
        return 2
    path, old_text, new_text = arguments
    code, message = replace_once(path, old_text, new_text)
    print(('  ✓ ' if code == 0 else '  ✗ ') + message)
    # → 怎么办：message 本身已经带着下一步（见 replace_once 各分支的返回值），这里不重复打印
    return code


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
