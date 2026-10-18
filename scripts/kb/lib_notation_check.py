"""replace-once.py 与 insert-row.py 写 .md 之前判新串：钟点、时区词、时间戳（含运行目录名、日志文件名里连写的紧凑写法）。

判据与 write-guard.sh 的第五道、门禁 doc-text 的 clock-times 格是同一份：`.claude/gate.d/lib-forbidden-notations.py`（形态键 clock-times），
这里没有另抄；排除前缀、上下文规则、.md 里不认放行标记都由那份库判，出路用那份库登记的 remedy。判的只是写进去的新串，旧串不判；
同一处命中、同一种形态，与 Edit 的 new_string 判得一样（write-guard.sh 判 Write、Edit、MultiEdit 的内容）。

为什么要有这一道：write-guard.sh 只挂 Write、Edit、MultiEdit；经这两个脚本写 .md 的不过它，
2026-10 有会话经定点替换往 records/ 的 .md 里写进了「日期 加 钟点 加 时区词」与光秃的时区词，到提交时门禁 doc-text 才判红。

用法（给 scripts/kb/ 里写 .md 的脚本 import；脚本按路径跑时 sys.path[0] 就是这个目录）：
    from lib_notation_check import clock_refusal
    refusal = clock_refusal(路径, 新串, 新串第一行在改完之后文件里的行号)    # None 放行；否则是说明行的列表，第一行是摘要（不带「✗」）

范围：目标路径以 .md 结尾（不分大小写）、在仓里（以这份文件所在的仓为准）、不在那份库里钟点那一形态的排除前缀下才判；别的后缀与仓外不判。
判据库取被判的那个仓里的那一份（root 下的 .claude/gate.d/lib-forbidden-notations.py）：仓范围、排除前缀、库三样同属一个仓；
平时 root 是这份文件所在的仓，所以等于「脚本自己所在的仓的库」。自证把临时目录当仓根时，用 copy_library_into 把真的库抄一份进去，
另造一个没有库的假仓核「读不到判据」那一支，两支各一格，拒绝的理由不会互相顶替。
读不到那份库（文件不在、语法错、弄坏开关写错）按拒绝处理，不静默放行。

没有自己的 --selftest：调它的两个脚本各自的 --selftest 走一遍拒绝与放行。
弄坏开关：这份库没有；判据的弄坏开关是 LIB_FORBIDDEN_NOTATIONS_BREAK 一族（写法在 lib-forbidden-notations.py 文件头），两个调用方各有「不判」的开关。
"""
import importlib.util
import os
import shutil
import sys

FORM_KEY = 'clock-times'
JUDGED_SUFFIXES = ('.md',)
LISTED_HITS_LIMIT = 20


def repository_root():
    """这份文件所在的仓的根：scripts/kb/ 往上两层。"""
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))


def library_file(root=None):
    return os.path.join(root or repository_root(), '.claude', 'gate.d', 'lib-forbidden-notations.py')


def load_library(root):
    """加载仓根 root 下的共用库 .claude/gate.d/lib-forbidden-notations.py（路径与 library_file 同一个）；不留 __pycache__。读不到就抛异常，由调用方按拒绝处理。"""
    spec = importlib.util.spec_from_file_location('lib_forbidden_notations', os.path.join(root, '.claude', 'gate.d', 'lib-forbidden-notations.py'))
    library = importlib.util.module_from_spec(spec)
    before = sys.dont_write_bytecode
    sys.dont_write_bytecode = True
    try:
        spec.loader.exec_module(library)
    finally:
        sys.dont_write_bytecode = before
    return library


def copy_library_into(root):
    """自证用：把这份文件所在的仓里真的判据库抄进假仓 root 的同一相对位置，返回抄到的路径。"""
    destination = library_file(root)
    os.makedirs(os.path.dirname(destination), exist_ok=True)
    shutil.copyfile(library_file(repository_root()), destination)
    return destination


def clock_refusal(path, new_text, first_line, root=None):
    """新串带钟点、时区词或时间戳就返回说明行的列表（第一行是摘要，其余行自带缩进与「→」），放行返回 None。
    first_line：新串第一行在改完之后的文件里是第几行（1 起），说明里的「第 n 行」按它折算。
    root：自检才传（把哪个目录当仓根，判据库取它下面的 .claude/gate.d/lib-forbidden-notations.py），平时取这份文件所在的仓。"""
    if not new_text or not path.lower().endswith(JUDGED_SUFFIXES):
        return None
    root = os.path.realpath(root or repository_root())
    absolute = os.path.realpath(path)
    if os.path.commonpath([absolute, root]) != root:
        return None
    relative = os.path.relpath(absolute, root)
    library_path = library_file(root)
    try:
        library = load_library(root)
        if library.is_excluded(FORM_KEY, relative):
            return None
        hits = library.all_hits(FORM_KEY, new_text, relative)
        remedy = list(library.FORMS_BY_KEY[FORM_KEY]['remedy'])
    except Exception as error:   # 文件不在、语法错、弄坏开关写错都算读不到
        return [f'读不到钟点判据 {library_path}（{type(error).__name__}：{error}），写进 {relative} 的新串没法判，按拒绝处理，一个字节没写',
                '     → 怎么办：那一份是门禁 doc-text、write-guard.sh 与这个脚本共用的唯一定义，先 git status 看它是被删了还是被改坏了，'
                '用 Bash 恢复它（不在这里另抄一份）；恢复之前往仓里 .md 的写都会被拒']
    if not hits:
        return None
    lines = [f'新串在描述里写了钟点、时区词或时间戳，共 {len(hits)} 处，{relative} 一个字节没写：']
    lines += [f'       第 {first_line + number - 1} 行（{name}）：「{matched}」' for number, _, name, matched in hits[:LISTED_HITS_LIMIT]]
    if len(hits) > LISTED_HITS_LIMIT:
        lines.append(f'       （另有 {len(hits) - LISTED_HITS_LIMIT} 处没列，改完上面这些再写一次会列出来）')
    lines.append('     → 怎么办：' + remedy[0])
    lines += ['     → ' + item for item in remedy[1:]]
    return lines
