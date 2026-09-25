#!/usr/bin/env python3
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
"""
import os
import sys
import tempfile

from lib_atomic_replace import ReplaceRefused, problems_with_replacement_by_rename, replace_file_contents_by_rename


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
        return 3, f'旧串在 {path} 里命中 0 次，一个字节没写\n     → 先用 grep 找到那一行，把旧串按文件里的原样（含空格与标点）抄进来'
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
    print('  ✓ --selftest 通过：命中 0 次拒绝（3）、2 次拒绝（4）、空串拒绝（2），都一个字节没写；命中 1 次写成功并回读确认；'
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
    sys.exit(main(sys.argv[1:]))
