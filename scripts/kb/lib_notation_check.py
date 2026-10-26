"""replace-once.py、insert-row.py、replace-batch.py 写之前判新串：会话号（任何后缀的仓内文件）；三份另判写 .md 的新串里的钟点、时区词、时间戳
（含运行目录名、日志文件名里连写的紧凑写法）。

判据与 write-guard.sh 的第五道、门禁 doc-text 的 clock-times 格是同一份：`.claude/gate.d/lib-forbidden-notations.py`（形态键 clock-times），
这里没有另抄；排除前缀、上下文规则、.md 里不认放行标记都由那份库判，出路用那份库登记的 remedy。判的只是写进去的新串，旧串不判；
同一处命中、同一种形态，与 Edit 的 new_string 判得一样（write-guard.sh 判 Write、Edit、MultiEdit 的内容）。

为什么要有这一道：write-guard.sh 只挂 Write、Edit、MultiEdit；经这几个脚本写的不过它，
2026-10 有会话经定点替换往 records/ 的 .md 里写进了「日期 加 钟点 加 时区词」与光秃的时区词，到提交时门禁 doc-text 才判红；
会话号（ListAgents 列的会话名）同样：经 insert-row.py、replace-once.py 写进 records/ 的有十来处，写闸看不见。

会话号一判（session_name_refusal）：判据是同一份库里的 session-names 那一形态（write-guard.sh 的第七道、门禁 doc-text 的 session-names 格读的就是它），
排除前缀、「# session-names:allow <理由>」放行标记（.sh、.py、.rs 里）、仓外不判都由那份库管，这里没有另抄；与钟点不同的是后缀不限：写闸对 Write、Edit 的内容
不分后缀都判，这里照同一个范围。出路用那份库登记的 remedy（头一条就是「写成「别的会话」这类不带号的说法」）。

用法（给 scripts/kb/ 里写文件的脚本 import；脚本按路径跑时 sys.path[0] 就是这个目录）：
    from lib_notation_check import clock_refusal, session_name_refusal, session_guard_problems
    refusal = clock_refusal(路径, 新串, 新串第一行在改完之后文件里的行号)    # None 放行；否则是说明行的列表，第一行是摘要（不带「✗」）
    refusal = session_name_refusal(路径, 新串, 行号)                      # 同上，判会话号
    problems = session_guard_problems(临时目录, attempt, 拒绝时的退出码)    # 三份脚本的自证共用的会话号样本，见函数说明

范围：钟点——目标路径以 .md 结尾（不分大小写）、在仓里（以这份文件所在的仓为准）、不在那份库里钟点那一形态的排除前缀下才判；别的后缀与仓外不判。
会话号——目标路径在仓里、不在会话号那一形态的排除前缀下就判，不看后缀；仓外不判。
判据库取被判的那个仓里的那一份（root 下的 .claude/gate.d/lib-forbidden-notations.py）：仓范围、排除前缀、库三样同属一个仓；
平时 root 是这份文件所在的仓，所以等于「脚本自己所在的仓的库」。自证把临时目录当仓根时，用 copy_library_into 把真的库抄一份进去，
另造一个没有库的假仓核「读不到判据」那一支，两支各一格，拒绝的理由不会互相顶替。
读不到那份库（文件不在、语法错、弄坏开关写错）按拒绝处理，不静默放行。

没有自己的 --selftest：调它的三个脚本各自的 --selftest 走一遍拒绝与放行。
弄坏开关：这份库没有；判据的弄坏开关是 LIB_FORBIDDEN_NOTATIONS_BREAK 一族（写法在 lib-forbidden-notations.py 文件头），调用方各有「不判」的开关
（钟点 REPLACE_ONCE_BREAK / INSERT_ROW_BREAK / REPLACE_BATCH_BREAK=no-clock-guard，会话号三份脚本各一个 no-session-guard）。
"""
import importlib.util
import os
import shutil
import sys

FORM_KEY = 'clock-times'
SESSION_FORM_KEY = 'session-names'
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


def form_refusal(form_key, form_name, summary, show_shape, suffixes, path, new_text, first_line, root=None):
    """新串在共用库 form_key 那一形态下有命中就返回说明行的列表（第一行是摘要，其余行自带缩进与「→」），放行返回 None。
    suffixes 不是 None 时只判这些后缀的目标（不分大小写）；目标不在仓里（以 root 为准）不判；读不到库按拒绝处理。
    summary 里的 {count} 与 {relative} 换成命中数与目标相对仓根的路径；show_shape 为真时每处命中带判据行的名字。"""
    if not new_text or (suffixes is not None and not path.lower().endswith(suffixes)):
        return None
    root = os.path.realpath(root or repository_root())
    absolute = os.path.realpath(path)
    if os.path.commonpath([absolute, root]) != root:
        return None
    relative = os.path.relpath(absolute, root)
    library_path = library_file(root)
    try:
        library = load_library(root)
        if library.is_excluded(form_key, relative):
            return None
        hits = library.all_hits(form_key, new_text, relative)
        remedy = list(library.FORMS_BY_KEY[form_key]['remedy'])
    except Exception as error:   # 文件不在、语法错、弄坏开关写错都算读不到
        return [f'读不到{form_name}判据 {library_path}（{type(error).__name__}：{error}），写进 {relative} 的新串没法判，按拒绝处理，一个字节没写',
                '     → 怎么办：那一份是门禁 doc-text、write-guard.sh 与这个脚本共用的唯一定义，先 git status 看它是被删了还是被改坏了，'
                '用 Bash 恢复它（不在这里另抄一份）；恢复之前往仓里的写都会被拒']
    if not hits:
        return None
    lines = [summary.format(count=len(hits), relative=relative)]
    for number, _, name, matched in hits[:LISTED_HITS_LIMIT]:
        place = first_line + number - 1
        lines.append(f'       第 {place} 行（{name}）：「{matched}」' if show_shape else f'       第 {place} 行：「{matched}」')
    if len(hits) > LISTED_HITS_LIMIT:
        lines.append(f'       （另有 {len(hits) - LISTED_HITS_LIMIT} 处没列，改完上面这些再写一次会列出来）')
    lines.append('     → 怎么办：' + remedy[0])
    lines += ['     → ' + item for item in remedy[1:]]
    return lines


def clock_refusal(path, new_text, first_line, root=None):
    """新串带钟点、时区词或时间戳就返回说明行的列表（第一行是摘要，其余行自带缩进与「→」），放行返回 None。只判 .md 目标。
    first_line：新串第一行在改完之后的文件里是第几行（1 起），说明里的「第 n 行」按它折算。
    root：自检才传（把哪个目录当仓根，判据库取它下面的 .claude/gate.d/lib-forbidden-notations.py），平时取这份文件所在的仓。"""
    return form_refusal(FORM_KEY, '钟点', '新串在描述里写了钟点、时区词或时间戳，共 {count} 处，{relative} 一个字节没写：', True, JUDGED_SUFFIXES,
                        path, new_text, first_line, root)


def session_name_refusal(path, new_text, first_line, root=None):
    """新串带会话号（ListAgents 列的会话名）就返回说明行的列表（形状同 clock_refusal），放行返回 None。仓内任何后缀的目标都判，与写闸判 Write、Edit 的范围一致。
    first_line、root 的意思同 clock_refusal。"""
    return form_refusal(SESSION_FORM_KEY, '会话号', '新串写了会话号（ListAgents 列的会话名），共 {count} 处，{relative} 一个字节没写：', False, None,
                        path, new_text, first_line, root)


def session_guard_problems(workspace, attempt, refused_code, entry=None):
    """三份定点写入脚本（replace-once.py、insert-row.py、replace-batch.py）的自证共用的会话号样本：交问题列表，空表是全过。
    样本里的会话名在这里拆开拼，这份源码里不出现字面写法（门禁 doc-text 也扫它）。
    workspace：自证的临时目录，里面造一个假仓（抄一份真的共用库），另造一个没有库的假仓核「读不到判据」那一支。
    attempt(路径, 新串, 假仓根, 干跑) → (退出码, 说明文字)：把新串写在 `第一行 甲\\n第二行 乙\\n` 的第 2 行（替换第 2 行，或插在第 1 行之后），文字是脚本打出来的；
      没有干跑的脚本忽略第四个参数。refused_code：会话号被拒时这个脚本的退出码。
    entry(路径, 新串, 假仓根) → (退出码, 说明文字)：从脚本的入口（main）走一遍，说明文字要以「✗ 新串写了会话号」起头（可带缩进）；不给就不核入口。"""
    problems = []
    glue = ''.join
    session = glue(('singlefs-', 'e152'))
    original = '第一行 甲\n第二行 乙\n'
    root = os.path.join(workspace, 'session-repo')
    for directory in ('records', 'scripts', os.path.join('.claude', 'gate.d', 'fixtures')):
        os.makedirs(os.path.join(root, directory))
    copy_library_into(root)
    markdown = os.path.join(root, 'records', 'notes.md')
    program = os.path.join(root, 'scripts', 'probe.py')
    excluded = os.path.join(root, '.claude', 'gate.d', 'fixtures', 'case.md')
    outside = os.path.join(workspace, 'outside-session.md')

    def reset(target):
        with open(target, 'w', encoding='utf-8') as handle:
            handle.write(original)

    def content_of(target):
        with open(target, encoding='utf-8') as handle:
            return handle.read()

    def run(target, new_text, dry_run=False, base=root):
        said_code, said = attempt(target, new_text, base, dry_run)
        return said_code, said

    refused_text = glue(('已发消息，', session, ' 回话'))
    for label, target in (('.md 目标', markdown), ('.py 目标（不是 .md 也判）', program)):
        reset(target)
        names_before = sorted(os.listdir(os.path.dirname(target)))
        for dry_run in (False, True):
            code, said = run(target, refused_text, dry_run)
            if code != refused_code or '写了会话号' not in said or session not in said or '第 2 行' not in said or '别的会话' not in said \
                    or '→ 怎么办' not in said or '读不到' in said:
                problems.append(f'写会话号：{label}{"（干跑）" if dry_run else ""}的新串带会话名，本该退出码 {refused_code}、点名「{session}」与第 2 行与出路「别的会话」，'
                                f'实际 {code}：{said!r}（脚本的 no-session-guard 或 LIB_FORBIDDEN_NOTATIONS_BREAK=session-names 关掉这一道的话这一条本来就该红）')
            if content_of(target) != original or sorted(os.listdir(os.path.dirname(target))) != names_before:
                problems.append(f'写会话号：{label}被拒绝之后文件或目录变了（一个字节都不该写、不该留临时文件）')
    if entry is not None:
        reset(markdown)
        code, said = entry(markdown, refused_text, root)
        if code != refused_code or not said.lstrip().startswith('✗ 新串写了会话号') or content_of(markdown) != original:
            problems.append(f'写会话号：入口本该退出码 {refused_code}、打 ✗ 开头的说明且不写文件，实际 {code}：{said!r}')
    for label, target, new_text in (
            ('干净的新串写 .md', markdown, '第二行 丙'),
            ('干净的新串写 .py', program, 'VALUE = 1'),
            ('.py 里带放行标记的一行（# session-names:allow 加理由）', program, glue(('NAME = "', session, '"  # session-names:allow 自证输入')))):
        reset(target)
        code, said = run(target, new_text)
        if code != 0 or new_text not in content_of(target):
            problems.append(f'写会话号：{label}本该放行（退出码 0）并写进文件，实际 {code}：{said!r}'
                            '（LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=session-names 关掉放行标记的话带标记的那一条本来就该红）')
    reset(excluded)
    code, said = run(excluded, refused_text)
    if code != 0 or refused_text not in content_of(excluded):
        problems.append(f'写会话号：排除前缀（门禁样本目录）下的文件本该放行，实际 {code}：{said!r}'
                        '（LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=session-names 关掉排除前缀的话这一条本来就该红）')
    reset(outside)
    code, said = run(outside, refused_text)
    if code != 0:
        problems.append(f'写会话号：仓外的文件不归这一道判，本该放行（退出码 0），实际 {code}：{said!r}')
    # 读不到共用库按拒绝处理，不静默放行：假仓里没有库，新串干净也拒；目标是 .py，所以拒的是会话号这一道、不是钟点那一道
    bare = os.path.join(workspace, 'session-repo-without-library')
    os.makedirs(os.path.join(bare, 'scripts'))
    bare_program = os.path.join(bare, 'scripts', 'probe.py')
    reset(bare_program)
    for dry_run in (False, True):
        code, said = run(bare_program, 'VALUE = 1', dry_run, base=bare)
        if code != refused_code or '读不到会话号判据' not in said or 'lib-forbidden-notations.py' not in said or 'FileNotFoundError' not in said \
                or '→ 怎么办' not in said or content_of(bare_program) != original:
            problems.append(f'写会话号：假仓里没有共用库{"（干跑）" if dry_run else ""}，本该退出码 {refused_code}、说「读不到会话号判据」点名库文件与 FileNotFoundError 与出路且不写文件，'
                            f'实际 {code}：{said!r}')
    return problems
