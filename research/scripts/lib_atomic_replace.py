"""把一份已有文件的内容整份换掉：同目录新建一份、改名盖上去，不在原文件上就地写。

为什么不就地写：`open(路径, 'w')` 截断重写改的是同一个 inode。正在按文件偏移边跑边读这份文件的进程
（bash 跑脚本就是这样读的，偏移在 /proc/<pid>/fdinfo/255 的 pos）下一次读会从改后内容的同一偏移接着读，
读到的是别的东西——后面的改动把前面正在跑的那一个弄坏了。改名盖上之后路径指向新 inode，
换上之前就打开了旧文件的进程读的仍是旧 inode 的旧内容。

用法（给 research/scripts/ 里要改已有文件的脚本 import；脚本按路径跑时 sys.path[0] 就是这个目录）：
    from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename
    notes = replace_file_contents_by_rename(路径, 新内容, check_before_rename=可选的回调)

做法：
  1. 路径是符号链接就解到它指向的文件，换的是那份文件，链接本身不动（与就地写的效果一致）；
  2. 不是普通文件、当前用户对它没有写权限、或有不止一个硬链接就拒绝：只读文件就地写本来写不进去，
     改名换上只看目录权限、会绕过只读；换 inode 之后别的硬链接名还指着旧内容，两个名字静默分家；
  3. 在同一目录排他新建临时文件（tempfile.mkstemp，O_CREAT | O_EXCL），写完新内容（UTF-8，与就地写同一种编码）；
  4. 照原文件的属主（能保就保：没有权限就在返回的提示行里写明）与权限位（含执行位）设到临时文件上，fsync；
  5. 调 check_before_rename（调用方的写前复核，例如读时记的 sha256 没变）：它抛异常就不换；
  6. os.replace 换上，再尽力 fsync 一次目录。
任何一步失败（OSError、check_before_rename 抛的异常、Ctrl-C）都先删临时文件，原文件一个字节没动；
OSError 换成 ReplaceRefused 抛出，别的异常原样抛出。

弄坏开关（只给自证判红用，生效时往 stderr 打一行）：ATOMIC_REPLACE_BREAK=
  inplace        走就地写（改前的写法）：inode 不变，在读的进程读到新内容
  nochmod        不照原文件的权限位：换上的文件是 mkstemp 给的 0600
  nofsync        不 fsync 临时文件
  nocleanup      失败时不删临时文件
  nofollow       不解符号链接：链接本身被换成普通文件
  allowhardlink  不拒绝有多个硬链接的文件
  allowreadonly  不拒绝当前用户只读的文件
不认识的值直接拒绝写，免得拼错的开关被当成「没开」。
"""
import contextlib
import glob
import io
import os
import stat
import subprocess
import sys
import tempfile

BREAK_ENVIRONMENT_VARIABLE = 'ATOMIC_REPLACE_BREAK'
KNOWN_BREAK_MODES = ('inplace', 'nochmod', 'nofsync', 'nocleanup', 'nofollow', 'allowhardlink', 'allowreadonly')
SELFTEST_READ_ONLY_MODE = 0o444
TEMPORARY_FILE_SUFFIX = '.replacing'
SELFTEST_ORIGINAL_MODE = 0o754
READER_TIMEOUT_IN_SECONDS = 30


class ReplaceRefused(Exception):
    """没换上：原文件一个字节没动，临时文件已删。str(异常) 的第二行是以「→」开头的下一步。"""


def active_break_mode():
    break_mode = os.environ.get(BREAK_ENVIRONMENT_VARIABLE, '')
    if break_mode == '':
        return ''
    if break_mode not in KNOWN_BREAK_MODES:
        raise ReplaceRefused(
            f'{BREAK_ENVIRONMENT_VARIABLE}={break_mode} 不认识，一个字节没写\n'
            f'     → 怎么办：弄坏开关只认 {"、".join(KNOWN_BREAK_MODES)}；平时不设这个变量')
    print(f'  ! {BREAK_ENVIRONMENT_VARIABLE}={break_mode} 生效：故意走坏的写法，只给自证判红用', file=sys.stderr)
    return break_mode


def remove_temporary_file_quietly(temporary_path):
    with contextlib.suppress(FileNotFoundError):
        os.unlink(temporary_path)


def fsync_directory_best_effort(directory):
    # 目录的 fsync 只让「改名」这件事落盘更稳；有的文件系统不支持对目录 fsync，失败不影响已经换上的内容
    try:
        directory_descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
    except OSError:
        return
    try:
        os.fsync(directory_descriptor)
    except OSError:
        pass
    finally:
        os.close(directory_descriptor)


def keep_owner_where_permitted(temporary_descriptor, target_status, target_path):
    """返回提示行列表：属主保不住时写明原来是谁、换上之后是谁。"""
    temporary_status = os.fstat(temporary_descriptor)
    if (temporary_status.st_uid, temporary_status.st_gid) == (target_status.st_uid, target_status.st_gid):
        return []
    try:
        os.fchown(temporary_descriptor, target_status.st_uid, target_status.st_gid)
        return []
    except PermissionError:
        pass
    try:
        os.fchown(temporary_descriptor, -1, target_status.st_gid)
    except PermissionError:
        pass
    kept_status = os.fstat(temporary_descriptor)
    return [f'属主没能全保留：{target_path} 原来是 uid {target_status.st_uid} gid {target_status.st_gid}，'
            f'换上之后是 uid {kept_status.st_uid} gid {kept_status.st_gid}（没有改属主的权限）']


def replace_file_contents_by_rename(path, new_text, check_before_rename=None):
    """把 path 的内容换成 new_text。返回提示行列表（属主没能保留这类，通常为空）。

    拒绝或失败时抛 ReplaceRefused；check_before_rename 抛的异常原样抛出。两种情况原文件都没动、临时文件都已删。
    """
    break_mode = active_break_mode()
    target_path = path if break_mode == 'nofollow' else os.path.realpath(path)
    try:
        target_status = os.stat(target_path)
    except OSError as error:
        raise ReplaceRefused(
            f'读不到 {path} 的状态（{error}），一个字节没写\n'
            f'     → 怎么办：检查路径是否写对，文件是否已经被别的进程删掉或改名') from error
    if not stat.S_ISREG(target_status.st_mode):
        raise ReplaceRefused(
            f'{target_path} 不是普通文件，一个字节没写\n'
            f'     → 怎么办：这个工具只改普通文件；目录、设备、管道要换别的办法')
    if target_status.st_nlink > 1 and break_mode != 'allowhardlink':
        raise ReplaceRefused(
            f'{target_path} 有 {target_status.st_nlink} 个硬链接，改名换上之后别的名字还指着旧内容，一个字节没写\n'
            f'     → 怎么办：find <仓根> -samefile {target_path} 找出另外几个名字；不需要共用一份内容就先拆成各自独立的文件再改，'
            f'确实要共用就别用这个工具改')
    if not os.access(target_path, os.W_OK) and break_mode != 'allowreadonly':
        raise ReplaceRefused(
            f'{target_path} 对当前用户只读，一个字节没写\n'
            f'     → 怎么办：就地写本来就写不进只读文件，改名换上会绕过只读，所以同样拒绝；确实要改就先 chmod u+w 再跑')
    if break_mode == 'inplace':
        if check_before_rename is not None:
            check_before_rename()
        with open(target_path, 'w', encoding='utf-8') as handle:
            handle.write(new_text)
        return []
    directory = os.path.dirname(target_path) or '.'
    try:
        temporary_descriptor, temporary_path = tempfile.mkstemp(
            prefix='.' + os.path.basename(target_path) + '.', suffix=TEMPORARY_FILE_SUFFIX, dir=directory)
    except OSError as error:
        raise ReplaceRefused(
            f'在 {directory} 里建不了临时文件（{error}），一个字节没写\n'
            f'     → 怎么办：改名换上要对目录有写权限（就地写只要对文件有写权限）；查目录权限与剩余空间') from error
    notes = []
    try:
        with os.fdopen(temporary_descriptor, 'wb') as handle:
            handle.write(new_text.encode('utf-8'))
            handle.flush()
            notes = keep_owner_where_permitted(handle.fileno(), target_status, target_path)
            if break_mode != 'nochmod':
                os.fchmod(handle.fileno(), stat.S_IMODE(target_status.st_mode))
            if break_mode != 'nofsync':
                os.fsync(handle.fileno())
        if check_before_rename is not None:
            check_before_rename()
        os.replace(temporary_path, target_path)
    except BaseException as error:
        if break_mode != 'nocleanup':
            remove_temporary_file_quietly(temporary_path)
        if isinstance(error, OSError):
            raise ReplaceRefused(
                f'写临时文件或改名换上 {target_path} 时失败（{error}），原文件没动，临时文件已删\n'
                f'     → 怎么办：查目录的剩余空间与权限，再重跑一次') from error
        raise
    fsync_directory_best_effort(directory)
    return notes


# ── 自证用：给调用它的脚本的自证调，run_tool(路径) 走那个脚本自己的整条改法、返回它的退出码 ──

READER_SOURCE = r'''
import os, sys
path, head_length = sys.argv[1], int(sys.argv[2])
descriptor = os.open(path, os.O_RDONLY)
head = b''
while len(head) < head_length:
    piece = os.read(descriptor, head_length - len(head))
    if not piece:
        break
    head += piece
sys.stdout.write('ready\n')
sys.stdout.flush()
sys.stdin.readline()
rest = b''
while True:
    piece = os.read(descriptor, 65536)
    if not piece:
        break
    rest += piece
os.close(descriptor)
sys.stdout.write(rest.hex() + '\n')
sys.stdout.flush()
'''


def leftover_temporary_files(directory):
    return sorted(glob.glob(os.path.join(directory, '.*' + TEMPORARY_FILE_SUFFIX)))


def first_differing_byte_offset(original_bytes, expected_bytes):
    for offset, (original_byte, expected_byte) in enumerate(zip(original_bytes, expected_bytes)):
        if original_byte != expected_byte:
            return offset
    return min(len(original_bytes), len(expected_bytes))


def run_tool_quietly(run_tool, path):
    with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
        return run_tool(path)


@contextlib.contextmanager
def patched_os_function(name, replacement):
    original = getattr(os, name)
    setattr(os, name, replacement)
    try:
        yield original
    finally:
        setattr(os, name, original)


def problems_with_reader_and_inode(run_tool, original_text, expected_text, workspace):
    """一个读者进程在改之前打开文件、读到第一处不同的字节之前，改完再接着读：它必须读到旧内容；inode 必须变、权限位不变。"""
    problems = []
    original_bytes = original_text.encode('utf-8')
    head_length = first_differing_byte_offset(original_bytes, expected_text.encode('utf-8'))
    if head_length == 0:
        return ['自证样本写得不对：改前改后第 0 个字节就不同，读者读不出「接着读」的那一段']
    path = os.path.join(workspace, 'running.sh')
    with open(path, 'wb') as handle:
        handle.write(original_bytes)
    os.chmod(path, SELFTEST_ORIGINAL_MODE)
    inode_before = os.stat(path).st_ino
    reader = subprocess.Popen([sys.executable, '-c', READER_SOURCE, path, str(head_length)],
                              stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    ready_line = reader.stdout.readline()
    if ready_line != 'ready\n':
        reader.communicate(timeout=READER_TIMEOUT_IN_SECONDS)
        return [f'读者进程没报 ready（读到 {ready_line!r}），这一格没判']
    call_order = []
    original_fsync, original_replace = os.fsync, os.replace

    def recording_fsync(descriptor):
        call_order.append('fsync')
        return original_fsync(descriptor)

    def recording_replace(source, destination):
        call_order.append('replace')
        return original_replace(source, destination)

    with patched_os_function('fsync', recording_fsync), patched_os_function('replace', recording_replace):
        exit_code = run_tool_quietly(run_tool, path)
    try:
        reader_output, _ = reader.communicate('go\n', timeout=READER_TIMEOUT_IN_SECONDS)
    except subprocess.TimeoutExpired:
        reader.kill()
        reader.communicate()
        return problems + [f'读者进程 {READER_TIMEOUT_IN_SECONDS} 秒没读完，这一格没判']
    if exit_code != 0:
        problems.append(f'正常改一处本该退出码 0，实际 {exit_code}')
    with open(path, 'rb') as handle:
        if handle.read() != expected_text.encode('utf-8'):
            problems.append('换上之后文件内容不是预期的新内容')
    status_after = os.stat(path)
    if status_after.st_ino == inode_before:
        problems.append(f'换上之后 inode 没变（{inode_before}）：还是在原文件上就地写，正在读它的进程会读到新内容')
    if stat.S_IMODE(status_after.st_mode) != SELFTEST_ORIGINAL_MODE:
        problems.append(f'权限位没保住：原来 {SELFTEST_ORIGINAL_MODE:o}，换上之后 {stat.S_IMODE(status_after.st_mode):o}')
    reader_rest = bytes.fromhex(reader_output.strip())
    if reader_rest != original_bytes[head_length:]:
        problems.append(f'改之前打开文件的读者接着读，读到的不是旧内容（从第 {head_length} 字节起读到新内容）：'
                        '正在跑的脚本会从改后文件的同一偏移接着读')
    if 'replace' not in call_order or 'fsync' not in call_order[:call_order.index('replace')]:
        problems.append(f'换上之前没 fsync 临时文件（调用次序 {call_order}）')
    leftovers = leftover_temporary_files(workspace)
    if leftovers:
        problems.append(f'改成功之后还留着临时文件 {leftovers}')
    return problems


def problems_with_symbolic_link(run_tool, original_text, expected_text, workspace):
    real_path = os.path.join(workspace, 'real.md')
    link_path = os.path.join(workspace, 'link.md')
    with open(real_path, 'w', encoding='utf-8') as handle:
        handle.write(original_text)
    os.symlink('real.md', link_path)
    exit_code = run_tool_quietly(run_tool, link_path)
    problems = []
    if exit_code != 0:
        problems.append(f'经符号链接改本该退出码 0，实际 {exit_code}')
    if not os.path.islink(link_path) or os.readlink(link_path) != 'real.md':
        problems.append('经符号链接改之后，链接本身被换掉了（就地写改的是它指向的文件，链接不动）')
    with open(real_path, encoding='utf-8') as handle:
        if handle.read() != expected_text:
            problems.append('经符号链接改之后，它指向的文件内容不是预期的新内容')
    return problems


def problems_with_hard_link(run_tool, original_text, workspace):
    shared_path = os.path.join(workspace, 'shared.md')
    other_name_path = os.path.join(workspace, 'other-name.md')
    with open(shared_path, 'w', encoding='utf-8') as handle:
        handle.write(original_text)
    os.link(shared_path, other_name_path)
    exit_code = run_tool_quietly(run_tool, shared_path)
    problems = []
    if exit_code == 0:
        problems.append('有两个硬链接的文件本该拒绝，实际照改了：两个名字从此指向两份不同的内容')
    for name in (shared_path, other_name_path):
        with open(name, encoding='utf-8') as handle:
            if handle.read() != original_text:
                problems.append(f'拒绝之后 {os.path.basename(name)} 的内容变了')
    if os.stat(shared_path).st_ino != os.stat(other_name_path).st_ino:
        problems.append('拒绝之后两个硬链接不再指向同一个 inode')
    return problems


def problems_with_read_only_file(run_tool, original_text, workspace):
    if os.geteuid() == 0:
        return ['以 root 跑：root 对只读文件照样判可写，这一格判不了，换普通用户跑自证']
    path = os.path.join(workspace, 'read-only.md')
    with open(path, 'w', encoding='utf-8') as handle:
        handle.write(original_text)
    os.chmod(path, SELFTEST_READ_ONLY_MODE)
    inode_before = os.stat(path).st_ino
    exit_code = run_tool_quietly(run_tool, path)
    problems = []
    if exit_code == 0:
        problems.append('当前用户只读的文件本该拒绝，实际照改了：改名换上绕过了只读')
    with open(path, encoding='utf-8') as handle:
        if handle.read() != original_text:
            problems.append('拒绝之后只读文件的内容变了')
    status_after = os.stat(path)
    if status_after.st_ino != inode_before or stat.S_IMODE(status_after.st_mode) != SELFTEST_READ_ONLY_MODE:
        problems.append('拒绝之后只读文件被换掉了，或权限位变了')
    return problems


def problems_with_injected_failure(run_tool, original_text, workspace, failing_function_name):
    path = os.path.join(workspace, f'fails-at-{failing_function_name}.md')
    with open(path, 'w', encoding='utf-8') as handle:
        handle.write(original_text)
    inode_before = os.stat(path).st_ino

    def failing(*arguments):
        raise OSError(5, f'自证注入的 {failing_function_name} 失败')

    with patched_os_function(failing_function_name, failing):
        exit_code = run_tool_quietly(run_tool, path)
    problems = []
    if exit_code == 0:
        problems.append(f'{failing_function_name} 失败时本该退出码非 0，实际 0')
    with open(path, encoding='utf-8') as handle:
        if handle.read() != original_text:
            problems.append(f'{failing_function_name} 失败之后原文件内容变了')
    if os.stat(path).st_ino != inode_before:
        problems.append(f'{failing_function_name} 失败之后原文件被换掉了')
    leftovers = leftover_temporary_files(workspace)
    if leftovers:
        problems.append(f'{failing_function_name} 失败之后还留着临时文件 {[os.path.basename(name) for name in leftovers]}')
    return problems


def problems_with_replacement_by_rename(run_tool, original_text, expected_text):
    """调用方自证用：返回问题列表，空列表就是全过。每一格各用一个新目录，互不干扰。"""
    problems = []
    for label, check in (
            ('读者与 inode', lambda workspace: problems_with_reader_and_inode(run_tool, original_text, expected_text, workspace)),
            ('符号链接', lambda workspace: problems_with_symbolic_link(run_tool, original_text, expected_text, workspace)),
            ('硬链接', lambda workspace: problems_with_hard_link(run_tool, original_text, workspace)),
            ('只读文件', lambda workspace: problems_with_read_only_file(run_tool, original_text, workspace)),
            ('fsync 失败', lambda workspace: problems_with_injected_failure(run_tool, original_text, workspace, 'fsync')),
            ('改名失败', lambda workspace: problems_with_injected_failure(run_tool, original_text, workspace, 'replace'))):
        with tempfile.TemporaryDirectory(prefix='atomic-replace-selftest-') as workspace:
            problems.extend(f'[{label}] {problem}' for problem in check(workspace))
    return problems
