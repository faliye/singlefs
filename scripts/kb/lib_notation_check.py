"""replace-once.py、insert-row.py、replace-batch.py 写之前判新串：会话号（任何后缀的仓内文件）；三份另判写 .md、.json、.txt 的新串里的钟点、时区词、时间戳
（含运行目录名、日志文件名里连写的紧凑写法）。

判据与 write-guard.sh 的第五道、门禁 doc-text 的 clock-times 格是同一份：`.claude/gate.d/lib-forbidden-notations.py`（形态键 clock-times），
这里没有另抄；排除前缀、上下文规则、.md 里不认放行标记都由那份库判，出路用那份库登记的 remedy。判的只是写进去的新串，旧串不判；
同一处命中、同一种形态，与 Edit 的 new_string 判得一样（write-guard.sh 判 Write、Edit、MultiEdit 的内容）。

为什么要有这一道：write-guard.sh 只挂 Write、Edit、MultiEdit；经这几个脚本写的不过它，
2026-10 有会话经定点替换往 records/ 的 .md 里写进了「日期 加 钟点 加 时区词」与光秃的时区词，到提交时门禁 doc-text 才判红；
会话号（ListAgents 列的会话名）同样：经 insert-row.py、replace-once.py 写进 records/ 的有十来处，写闸看不见。

会话号一判（session_name_refusal）：判据是同一份库里的 session-names 那一形态（write-guard.sh 的第七道、门禁 doc-text 的 session-names 格读的就是它），
排除前缀、「# session-names:allow <理由>」放行标记（.sh、.py、.rs 里）、仓外不判都由那份库管，这里没有另抄；与钟点不同的是后缀不限：写闸对 Write、Edit 的内容
不分后缀都判，这里照同一个范围。出路用那份库登记的 remedy（头一条是「全名与两位短号都不写，别的会话写成 mainagent(A)、mainagent(B) 这类按出场次序的叫法」）。
新串里没有会话号的，同一个函数接着判批次代号（同一份库的 batch-codes 形态，write-guard.sh 第七道里「写了批次代号」那一判读的也是它），
再没有的接着判任务号、会话短号与批号（同一份库的 task-and-session-numbers 形态，write-guard.sh 第七道里「写了任务号、会话短号或批号」那一判读的也是它）：
这一形态判的不是新串，是写前、写后两份整文（库的 new_violation_hits：两份各判一遍，写后每一行配上写前的一行，违例按判据行名字与命中的串只拿配上的那一行去抵，抵不掉的才拒；
文件里已有的旧号不挡这一次写，只改了那一行别处的也不挡；同一字面的旧号离开写前它所在的那一行、这一次写让后面已有的行出了围栏、离开了「批次」那一列的照拒），
调用方把这一次写之前的整份与写之后的整份交进来；
两种形态库都按后缀只判 .md、.tsv、.txt，排除前缀、上下文规则、围栏与表头都在库里；拒绝的退出码与会话号同一个（三份脚本不另分），出路用那一形态的 remedy，
调用方另接的收尾一句按拒绝的是哪一判取（closing_remedy）。
几样都有时按会话号、批次代号、任务号与批号的次序报第一样，改完再写一次报下一样。三份脚本的 no-session-guard 开关关掉这个函数，几样一起不判。
append-record.py、ledger-append-section.py 调的也是这个函数。

用法（给 scripts/kb/ 里写文件的脚本 import；脚本按路径跑时 sys.path[0] 就是这个目录）：
    from lib_notation_check import clock_refusal, closing_remedy, session_name_refusal, session_guard_problems
    refusal = clock_refusal(路径, 新串, 新串第一行在改完之后文件里的行号)    # None 放行；否则是说明行的列表，第一行是摘要（不带「✗」）
    refusal = session_name_refusal(路径, 新串, 行号, before_text=写前整份, after_text=写后整份)   # 同上，判会话号，再判批次代号，再判任务号、会话短号与批号
    refusal = numbers_refusal(路径, 写前整份, 写后整份)                    # 只判任务号、会话短号与批号（session_name_refusal 两份都交 None 时调用方另调它）
    closing_remedy(refusal)                                              # 收尾那一句出路
    problems = session_guard_problems(临时目录, attempt, 拒绝时的退出码)    # 三份脚本的自证共用的会话号样本，见函数说明

范围：钟点——目标路径以 JUDGED_SUFFIXES 那几种（.md、.json、.txt，不分大小写）结尾、在仓里（以这份文件所在的仓为准）、不在那份库里钟点那一形态的排除前缀下才判；别的后缀与仓外不判。
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
BATCH_FORM_KEY = 'batch-codes'
NUMBERS_FORM_KEY = 'task-and-session-numbers'
JUDGED_SUFFIXES = ('.md', '.json', '.txt')
LISTED_HITS_LIMIT = 20
NUMBERS_SUMMARY = '新串写了任务号、会话短号或批号（写前、写后整份各判一遍，每一行只拿它配上的写前那一行上的同一种去抵，抵不掉的），共 {count} 处，{relative} 一个字节没写：'
# 调用方在 session_name_refusal 的说明行之后各接一句收尾的出路：按拒绝说明第一行是哪一判取（closing_remedy）
CLOSING_REMEDIES = (
    ('新串写了会话号', '会话号换成按出场次序的叫法（别的会话写 mainagent(A)、mainagent(B)，子 agent 写 subagent(A)、subagent(B)），事情与日期照留'),
    ('新串写了批次代号', '批次代号换成那一批做了什么事'),
    ('新串写了任务号', '任务号换成交给服务的那条命令（不带号）或结局行的 type= 与 exit= 那一段，引日志只抄那一行的内容、不抄日志路径；'
                    '批号换成那一批做了什么、改了哪几个文件；会话短号与「<做什么>那一路的会话」这类描述换成按出场次序的叫法'
                    '（别的会话写 mainagent(A)、mainagent(B)，子 agent 写 subagent(A)、subagent(B)），写清它改了哪个文件、做了什么'),
    ('读不到', '先照上面那一句把判据库恢复了'),
)


def closing_remedy(refusal):
    """session_name_refusal 交的说明行（第一行是摘要）对应的收尾出路：按摘要的开头取 CLOSING_REMEDIES 里那一句；都对不上的交通用的一句。"""
    head = refusal[0] if refusal else ''
    return next((remedy for prefix, remedy in CLOSING_REMEDIES if head.startswith(prefix)), '照上面每一条的出路改')


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


def form_refusal(form_key, form_name, summary, show_shape, suffixes, path, new_text, first_line, root=None, before_text=None):
    """新串在共用库 form_key 那一形态下有命中就返回说明行的列表（第一行是摘要，其余行自带缩进与「→」），放行返回 None。
    suffixes 不是 None 时只判这些后缀的目标（不分大小写）；目标不在仓里（以 root 为准）不判；读不到库按拒绝处理。
    summary 里的 {count} 与 {relative} 换成命中数与目标相对仓根的路径；show_shape 为真时每处命中带判据行的名字。
    before_text 不是 None 时 new_text 是写之后的整份文件：交库的 new_violation_hits 判写前、写后两份整文（每一行只拿它配上的写前那一行上的同一种去抵，抵不掉的才算），行号就是写后文件的行号，first_line 不用。"""
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
        if before_text is None:
            hits = library.all_hits(form_key, new_text, relative)
        else:
            hits = library.new_violation_hits(form_key, before_text, new_text, relative)
            first_line = 1
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
    """新串带钟点、时区词或时间戳就返回说明行的列表（第一行是摘要，其余行自带缩进与「→」），放行返回 None。只判 JUDGED_SUFFIXES 那几种后缀的目标。
    first_line：新串第一行在改完之后的文件里是第几行（1 起），说明里的「第 n 行」按它折算。
    root：自检才传（把哪个目录当仓根，判据库取它下面的 .claude/gate.d/lib-forbidden-notations.py），平时取这份文件所在的仓。"""
    return form_refusal(FORM_KEY, '钟点', '新串在描述里写了钟点、时区词或时间戳，共 {count} 处，{relative} 一个字节没写：', True, JUDGED_SUFFIXES,
                        path, new_text, first_line, root)


def session_name_refusal(path, new_text, first_line, root=None, *, before_text, after_text):
    """新串带会话号（ListAgents 列的会话名）就返回说明行的列表（形状同 clock_refusal），放行返回 None。仓内任何后缀的目标都判，与写闸判 Write、Edit 的范围一致。
    没有会话号的接着判新串里的批次代号（库里 batch-codes 那一形态），再没有的接着判任务号、会话短号与批号（库里 task-and-session-numbers 那一形态）：
    这一形态判的是写前、写后两份整文（before_text 是这一次写之前的整份文件，不在是空串；after_text 是写之后的整份），每一行只拿它配上的写前那一行上的同一种去抵、抵不掉的才拒，行号是写后文件的行号；
    后两种库按后缀只判 .md、.tsv、.txt。读不到库时交会话号那一判的拒绝，后两种不再判。first_line、root 的意思同 clock_refusal。
    before_text、after_text 都交 None 的，任务号那一形态这里不判，由调用方按文件另调 numbers_refusal（一次写改同一份文件几处的 replace-batch.py）。"""
    for form_key, form_name, summary, show_shape in (
            (SESSION_FORM_KEY, '会话号', '新串写了会话号（ListAgents 列的会话名），共 {count} 处，{relative} 一个字节没写：', False),
            (BATCH_FORM_KEY, '批次代号', '新串写了批次代号，共 {count} 处，{relative} 一个字节没写：', True)):
        new_text_refusal = form_refusal(form_key, form_name, summary, show_shape, None, path, new_text, first_line, root)
        if new_text_refusal is not None:
            return new_text_refusal
    if before_text is None and after_text is None:
        return None
    return numbers_refusal(path, before_text, after_text, root)


def numbers_refusal(path, before_text, after_text, root=None):
    """写前（before_text，文件原来不在是空串）、写后（after_text）两份整文的任务号、会话短号与批号（库里 task-and-session-numbers 那一形态的 new_violation_hits）：
    每一行只拿它配上的写前那一行上的同一种（判据与命中的串）去抵，抵不掉的有命中就返回说明行的列表（形状同 clock_refusal，行号是写后文件的行号），放行返回 None。读不到库按拒绝处理。"""
    return form_refusal(NUMBERS_FORM_KEY, '任务号与批号', NUMBERS_SUMMARY, True, None, path, after_text, 1, root, before_text)


def session_guard_problems(workspace, attempt, refused_code, entry=None):
    """三份定点写入脚本（replace-once.py、insert-row.py、replace-batch.py）的自证共用的会话号样本：交问题列表，空表是全过。
    样本里的会话名在这里拆开拼，这份源码里不出现字面写法（门禁 doc-text 也扫它）。
    workspace：自证的临时目录，里面造一个假仓（抄一份真的共用库），另造一个没有库的假仓核「读不到判据」那一支。
    attempt(路径, 新串, 假仓根, 干跑) → (退出码, 说明文字)：把新串写在 `第一行 甲\\n第二行 乙\\n` 的第 2 行（替换「第二行 乙」那一行，或插在「第一行」起头的那一行之后），
      文字是脚本打出来的；写前写后那几格的文件头上多一行开着的围栏，或尾巴上多几行（批号、旧任务号），或「第二行 乙」那一行本身带旧任务号、在围栏里、是表头，
      脚本要把写前、写后的整份交给 session_name_refusal 的 before_text、after_text。
      没有干跑的脚本忽略第四个参数。refused_code：会话号被拒时这个脚本的退出码。
    entry(路径, 新串, 假仓根) → (退出码, 说明文字)：从脚本的入口（main）走一遍，说明文字要以「✗ 新串写了会话号」起头（可带缩进）；不给就不核入口。
    旧号挪到另一句、另一行与只改同一行别处那三格不经 attempt：写前、写后两份整文直接交 numbers_refusal，三份脚本判得一样。"""
    problems = []
    glue = ''.join
    session = glue(('singlefs-', 'e152'))
    original = '第一行 甲\n第二行 乙\n'
    root = os.path.join(workspace, 'session-repo')
    for directory in ('records', 'scripts', os.path.join('research', 'prompts'), os.path.join('.claude', 'gate.d', 'fixtures')):
        os.makedirs(os.path.join(root, directory))
    copy_library_into(root)
    markdown = os.path.join(root, 'records', 'notes.md')
    verdict = os.path.join(root, 'research', 'prompts', 'demo-r1-main-verification.md')
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
    # 批次代号与会话号同一个函数判：.md 目标的新串带代号拒（与会话号同一个退出码），.py 目标不判（库里批次代号只判 .md、.tsv、.txt）
    for label, target, new_text, refused in (
            ('.md 目标的新串带数字加字母的代号', markdown, '24c-1 交回', True),
            ('.md 目标的新串带天干编号的批号', markdown, '批 10 甲交回', True),
            ('.py 目标的新串带代号（批次代号只判 .md、.tsv、.txt）', program, 'VALUE = 1  # 24c-1 交回', False)):
        reset(target)
        code, said = run(target, new_text)
        if refused and (code != refused_code or '写了批次代号' not in said or '→ 怎么办' not in said or content_of(target) != original):
            problems.append(f'写会话号：{label}，本该退出码 {refused_code}、说「写了批次代号」带出路且不写文件，实际 {code}：{said!r}'
                            '（脚本的 no-session-guard 或 LIB_FORBIDDEN_NOTATIONS_BREAK=batch-codes 关掉这一判的话这一条本来就该红）')
        if not refused and (code != 0 or new_text not in content_of(target)):
            problems.append(f'写会话号：{label}，本该放行（退出码 0）并写进文件，实际 {code}：{said!r}'
                            '（LIB_FORBIDDEN_NOTATIONS_BREAK=judged-suffixes-ignored 让批次代号什么后缀都判的话这一条本来就该红）')
    # 任务号、会话短号与批号与会话号同一个函数判（排在批次代号之后）：.md 目标的新串带三条判据行之一拒（记录里的「A/B 读数：」一行与反引号里的任务号也拒），
    # 记录里表头是「批次」那一列的批号、判决里行首的「A/B 读数：」一行与 .py 目标放行
    for label, target, new_text, row in (
            ('.md 目标的新串带任务号', markdown, '调度那一条跑完（任务 21036）', 'task-numbers'),
            ('.md 目标（记录）里带「A/B 读数：」的一行', markdown, 'A/B 读数：任务 11964', 'task-numbers'),
            ('.md 目标的新串带反引号里的任务号', markdown, '结局行 `任务 21036 退 0` 原样', 'task-numbers'),
            ('.md 目标的新串带括号里的会话短号', markdown, '这件事 (5b) 知道', 'session-short-codes'),
            ('.md 目标的新串带别的会话的批号', markdown, '它的批 105 在修', 'batch-numbers'),
            ('.md 目标的新串拿描述指别的会话（那一路会话）', markdown, '检出钩子换上前问过改 ㉒ 那一路会话，换完把新 sha256 发给它', 'session-descriptions')):
        reset(target)
        code, said = run(target, new_text)
        if code != refused_code or '写了任务号、会话短号或批号' not in said or '第 2 行' not in said or '→ 怎么办' not in said or content_of(target) != original:
            problems.append(f'写会话号：{label}，本该退出码 {refused_code}、说「写了任务号、会话短号或批号」点名第 2 行带出路且不写文件，实际 {code}：{said!r}'
                            f'（脚本的 no-session-guard、LIB_FORBIDDEN_NOTATIONS_BREAK=task-and-session-numbers 或 ={row} 关掉这一判的话这一条本来就该红）')
    # 写前、写后整份各判一遍，写后每一行上的违例拿它配上的写前那一行上的同一种（判据行名字与命中的串）去抵：(说明, 写前的文件, 新串, 该不该拒, 打开时这一条该红的开关)。
    # 新串都不带号的五条：插入点前开着的围栏被新串收掉、后面那一行出了围栏；新串只写一行围栏、让后面已有的批号出了围栏；文件里已有的旧号不挡这一次写；
    # 带号那一行只改别处（替换的两份改的是带号那一行本身，insert-row.py 插在它前面）；围栏里带批号的那一行被新串挪到围栏外；新串改了表头、已有的批号离开「批次」那一列。
    # 新串带号的两条：同一行新加第二个任务号（替换的两份加在带旧号那一行里）；整份多出一行带同一个旧号（只报一处、点名新写的第 2 行，check_lines 里的两段都要在）
    numbers_line = '第二行 乙（任务 21036）'
    table_before = '第一行 甲\n| 第二行 乙 | 批次 |\n|---|---|\n| 交回 | 批 6 |\n'
    for label, before, new_text, refused, switches, check_lines in (
            ('插入点前围栏开着、新串以收尾围栏起头', '```\n' + original, '```\n\n正文：这件事 (5b) 知道', True,
             '脚本没把写前、写后的整份交给 session_name_refusal', None),
            ('新串只写一行围栏、让后面已有的批号出了围栏', original + '```\n那一路第 7 批交回\n```\n', '```', True,
             'LIB_FORBIDDEN_NOTATIONS_BREAK=after-lines-judged-alone 或 before-context-ignored 或脚本没把写前、写后的整份交给 session_name_refusal', None),
            ('写前文件里已有的旧号不挡这一次写', original + '旧行（任务 21036）\n', '第二行 丙', False, 'LIB_FORBIDDEN_NOTATIONS_BREAK=violations-not-netted', None),
            ('行里已有任务号、同一行只改路径', '第一行 甲\n' + numbers_line + '\n', 'crates/new/src/lib.rs', False,
             'LIB_FORBIDDEN_NOTATIONS_BREAK=violations-netted-by-line-text 或 violations-not-netted', None),
            ('行里已有任务号、同一行新加第二个任务号', '第一行 甲\n' + numbers_line + '\n', '第二行 丙（任务 21037）', True,
             'LIB_FORBIDDEN_NOTATIONS_BREAK=violations-netted-by-line-number', None),
            ('围栏里已有批号的那一行挪到围栏外', '第一行 甲\n```\n第二行 乙 那一路第 7 批交回\n```\n', '```\n第二行 乙', True,
             'LIB_FORBIDDEN_NOTATIONS_BREAK=before-context-ignored 或 after-lines-judged-alone', None),
            ('改了表头、已有的批号离开批次那一列', table_before, '| 第二行 丙 | 内容', True,
             'LIB_FORBIDDEN_NOTATIONS_BREAK=before-context-ignored 或 after-lines-judged-alone', None),
            ('整份多出一行带同一个旧号', original + '旧行（任务 21036）\n', '第二行 丙（任务 21036）', True,
             'LIB_FORBIDDEN_NOTATIONS_BREAK=violations-netted-uncounted 或 violations-not-netted', ('共 1 处', '第 2 行（'))):
        with open(markdown, 'w', encoding='utf-8') as handle:
            handle.write(before)
        code, said = run(markdown, new_text)
        if refused and (code != refused_code or '写了任务号、会话短号或批号' not in said or content_of(markdown) != before):
            problems.append(f'写会话号：{label}，本该退出码 {refused_code}、说「写了任务号、会话短号或批号」且不写文件，实际 {code}：{said!r}（{switches}的话这一条本来就该红）')
        elif refused and check_lines is not None and not all(piece in said for piece in check_lines):
            problems.append(f'写会话号：{label}，拒绝说明本该只点名新写的那一行（说「{check_lines[0]}」、点名「{check_lines[1]}」：原样留着的那一行先拿去抵），'
                            f'实际：{said!r}（{switches}的话这一条本来就该红）')
        if not refused and (code != 0 or new_text not in content_of(markdown)):
            problems.append(f'写会话号：{label}，本该放行（退出码 0）并写进文件，实际 {code}：{said!r}（{switches} 设着的话这一条本来就该红）')
    # 相抵按行配：同一字面的旧号离开写前它所在的那一行照拒，只改同一行别处照放行。insert-row.py 只插行、挪不走已有的号，这三格不经 attempt，
    # 写前、写后两份整文直接交 numbers_refusal（三份脚本判任务号与批号都走它），三份自证判得一样：(说明, 写前, 写后, 该不该拒, 打开时这一条该红的开关, 拒绝说明里要有的两段)。
    # 挪到另一句：号从头一句挪进下一行那一句，两行都改了、行数没变；挪到另一行：号从头一行挪进末尾新加的一行；
    # 只改同一行别处：带号那一行改了路径，前面另加了一行（两行连成一块改动，要按去掉违例之后的字配回原来那一行）
    for label, before, after, refused, switches, check_lines in (
            ('旧号挪到另一句照拒', '甲句（任务 21036）。\n乙句说另一件事。\n', '甲句。\n乙句说另一件事（任务 21036）。\n', True,
             'LIB_FORBIDDEN_NOTATIONS_BREAK=violations-netted-across-lines 或 violations-netted-uncounted', ('共 1 处', '第 2 行（')),
            ('旧号挪到另一行照拒', '第一行 甲（任务 21036）\n第二行 乙\n第三行 丙\n', '第一行 甲\n第二行 乙\n第三行 丙\n第四行 丁（任务 21036）\n', True,
             'LIB_FORBIDDEN_NOTATIONS_BREAK=violations-netted-across-lines 或 violations-netted-uncounted', ('共 1 处', '第 4 行（')),
            ('只改同一行别处照放行', '第一行 甲\n第二行 乙改了 a/b.rs（任务 21036）。\n', '第一行 甲\n新加的一行说明。\n第二行 乙改了 a/c/main.rs（任务 21036）。\n', False,
             'LIB_FORBIDDEN_NOTATIONS_BREAK=line-pairing-positional、violations-netted-by-line-text、violations-netted-by-line-number 或 violations-not-netted', None)):
        refusal = numbers_refusal(markdown, before, after, root)
        said = '\n'.join(refusal or ())
        if refused and (refusal is None or '写了任务号、会话短号或批号' not in said or not all(piece in said for piece in check_lines)):
            problems.append(f'写会话号：{label}，写前写后两份整文交 numbers_refusal 本该拒、说「写了任务号、会话短号或批号」「{check_lines[0]}」并点名「{check_lines[1]}」，'
                            f'实际：{said!r}（{switches}的话这一条本来就该红）')
        if not refused and refusal is not None:
            problems.append(f'写会话号：{label}，写前写后两份整文交 numbers_refusal 本该放行，实际：{said!r}（{switches}设着的话这一条本来就该红）')
    for label, target, new_text in (
            ('.md 目标（记录）里表头是批次的那一列的批号', markdown, '| 批次 | 内容 |\n|---|---|\n| 批 6 | 交回 |'),
            ('判决里行首的「A/B 读数：」一行', verdict, 'A/B 读数：任务 11964'),
            ('.py 目标的新串带任务号（这一形态只判 .md、.tsv、.txt）', program, 'VALUE = 1  # 任务 21036 跑完')):
        reset(target)
        code, said = run(target, new_text)
        if code != 0 or new_text not in content_of(target):
            problems.append(f'写会话号：{label}，本该放行（退出码 0）并写进文件，实际 {code}：{said!r}'
                            '（LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=task-and-session-numbers 或 judged-suffixes-ignored 设着的话这一条本来就该红）')
    # 按出场次序的 mainagent(A)、subagent(A) 与不跟会话词的「一路」不算拿描述指别的会话（LIB_FORBIDDEN_NOTATIONS_BREAK=session-descriptions-loose 设着的话这一条本来就该红）
    lettered = 'mainagent(A) 在改判据库，subagent(B) 交回；这一路、那一路径、一路绿、按块那一路都照写'
    reset(markdown)
    code, said = run(markdown, lettered)
    if code != 0 or lettered not in content_of(markdown):
        problems.append(f'写会话号：.md 目标的新串用按出场次序的叫法与不跟会话词的一路，本该放行（退出码 0）并写进文件，实际 {code}：{said!r}'
                        '（LIB_FORBIDDEN_NOTATIONS_BREAK=session-descriptions-loose 设着的话这一条本来就该红）')
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
