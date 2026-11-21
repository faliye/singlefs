#!/usr/bin/env python3
# admission: always 每次都拿此刻的快照目录现读清单、现生成骨架，上一次生成的骨架不替这一次作保
# run-condition: command bash
"""三方判决的骨架：照开工快照里的 sha256 清单预填「## 这一轮被判过的文件」一节，带判决要有的几节标题。

用法：
    verdict-skeleton.py <轮名> [--out <路径>] [--repository <仓根>] [--list <清单文件名> …]
    verdict-skeleton.py --selftest       # VERDICT_SKELETON_BREAK=<项> 时必须判红
    verdict-skeleton.py --help           # 打这一段用法（-h 同），退 0

读 <仓根>/research/prompts/<轮名>-snapshot/ 下的 sha256 清单：
    默认读目录里全部 *sha256*.txt，kb-sha256.txt 除外（它记的是引用用的 kb 文件，不是被判对象，门禁也不拿它当清单）；
    给了 --list 就只读点名的那几份（同一目录下的文件名，kb-sha256.txt 想读要点名）。
    清单是 sha256sum 的原样输出，每行「<64 位十六进制>  <路径>」（两个空格，二进制模式是一个空格接 *；路径开头的 ./ 不碍）。
    读不出一行路径、或有一行不是这个形状，就拒绝（退 2），不猜。
写骨架：默认写 <仓根>/research/prompts/<轮名>-main-verification.md，--out 另给路径；排他新建，文件已在就拒（退 2），不覆盖。
    骨架依次带这几节：「## 一、腿与核查」「## 二、判决」「## 三、改法」「## 交用户的」「## 这一轮被判过的文件」「## 回看决策」。
    「## 这一轮被判过的文件」一节按清单次序每个路径一行：「- <路径>：（待填：判了它什么）」，同一个路径在几份清单里只列一行。
    这一节的认法就是门禁 .claude/gate.d/doc-process-records.sh 的「点名一节」：标题逐字、二级标题，节到下一个一级或二级标题为止，
    节里写了否定词的行（门禁里 REVIEWED_SECTION_NEGATIONS 那几个词）不算点名；预填的行不带那些词。
    其余各节与回看决策表预填的「（待填）」是占位：填完删掉，门禁的回看决策格对没填的表行判红。
    待填提示不叫人把任务号写进判决：判决引的读数写命令与结局行的 type=、exit= 那一段；「## 三、改法」带一行行首的「A/B 读数：任务 <号>」，
    给零轮改法填号（派发闸 .claude/hooks/runner-dispatch-guard.sh 认这一行，判据库只在判决里放过它），没有零轮改法的删掉。
退出码：0 写好；1 --selftest 有格红；2 用法错、快照目录或清单不合、输出文件已在。

弄坏开关 VERDICT_SKELETON_BREAK 只在自证（--selftest）里认：平常跑设了它，不管值是什么，参数解析之前就退 2。几项可以逗号连写。
逐项写开关名、弄坏哪一处，下一行起写打开后自证判红的格（「」里是那条失败行开头的原文）：
  VERDICT_SKELETON_BREAK=reviewed-heading-typo  被判过的文件那一节的标题写错一个字（「文件」写成「文档」）
      「✗ 自检：门禁点名一节的判法应当认得出骨架里的每个路径」（同时「✗ 自检：骨架应当逐字带这几节标题」）
  VERDICT_SKELETON_BREAK=placeholder-negated  预填的占位里带「没判」，门禁整行不当点名认
      「✗ 自检：门禁点名一节的判法应当认得出骨架里的每个路径」
  VERDICT_SKELETON_BREAK=overwrite-existing  输出文件已在也照写，不排他新建
      「✗ 自检：输出文件已在应当退 2、原文件不动」
  VERDICT_SKELETON_BREAK=kb-manifest-read  默认也读 kb-sha256.txt
      「✗ 自检：默认不读 kb-sha256.txt」「✗ 自检：被判过的文件一节应当每个路径一行、同一路径只一行」「✗ 自检：快照目录里没有清单应当退 2」（只有 kb 清单的目录也照写了）
  VERDICT_SKELETON_BREAK=empty-snapshot-accepted  快照目录里没有可读的清单也照写一个空的被判过的文件一节
      「✗ 自检：快照目录里没有清单应当退 2」
  VERDICT_SKELETON_BREAK=readings-task-number-asked  「一、腿与核查」的待填提示叫人把判决引的读数写成任务号
      「✗ 自检：骨架的待填提示不要求把任务号写进判决」
  VERDICT_SKELETON_BREAK=zero-round-reading-line-dropped  「三、改法」里不带「A/B 读数：任务 <号>」那一行
      「✗ 自检：骨架里零轮改法那一条带一行派发闸认的 A/B 读数」
"""
import datetime
import importlib.util
import os
import re
import subprocess
import sys
import tempfile

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

SCRIPT = os.path.abspath(__file__)
DEFAULT_REPOSITORY = os.path.normpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', '..'))
GATE_PATH = '.claude/gate.d/doc-process-records.sh'

REVIEWED_HEADING = '## 这一轮被判过的文件'
HEADINGS = ['## 一、腿与核查', '## 二、判决', '## 三、改法', '## 交用户的', REVIEWED_HEADING, '## 回看决策']
KB_MANIFEST = 'kb-sha256.txt'
# 判决里的号只写零轮改法那一行行首的「A/B 读数：任务 <号>」（派发闸 .claude/hooks/runner-dispatch-guard.sh 的 ZERO_ROUND_READING_LINE 与 ZERO_ROUND_TASK_NUMBER 认它，
# 判据库 .claude/gate.d/lib-forbidden-notations.py 只在 research/prompts/*-main-verification.md 里放过它）；别处的读数写命令与结局行的 type=、exit=
READINGS_PROMPT = '（待填：cite-check 与 snapshot-tree 两条命令原样与它们判了什么；判决引的读数由主 agent 复跑，写那条命令与结局行的 type= 与 exit= 那一段，不写任务号）'
ZERO_ROUND_PROMPT = ('（待填：腿提的、判决里推的改法逐条标「量过」或「零轮」；标零轮的，写进产品代码之前先在探针里做 A/B，'
                     '读数写在那一条下面单起的一行「A/B 读数：任务 <号>」（行首起，<号> 换成那条 A/B 任务的号），别处不写任务号；没有零轮改法的删掉下面这一行）')
ZERO_ROUND_READING_PLACEHOLDER = 'A/B 读数：任务 <号>'
DISPATCH_GUARD_PATH = '.claude/hooks/runner-dispatch-guard.sh'
NOTATIONS_LIBRARY_PATH = '.claude/gate.d/lib-forbidden-notations.py'
NUMBERS_FORM = 'task-and-session-numbers'
SAMPLE_TASK_NUMBER = '12345'   # 自证里填进「<号>」的样本
MANIFEST_LINE = re.compile(r'^[0-9a-f]{64} [ *](?P<path>.+)$')
ROUND_NAME = re.compile(r'^[A-Za-z0-9][A-Za-z0-9._-]*$')

BREAK_VARIABLE = 'VERDICT_SKELETON_BREAK'
BREAK_ITEMS = ('reviewed-heading-typo', 'placeholder-negated', 'overwrite-existing', 'kb-manifest-read', 'empty-snapshot-accepted',
               'readings-task-number-asked', 'zero-round-reading-line-dropped')
SELFTEST_MARK_VARIABLE = 'VERDICT_SKELETON_SELFTEST_MARK'
SELFTEST_MARK_CONTENT = 'verdict-skeleton.py --selftest 起的子进程'


def broken_items():
    return {item for item in os.environ.get(BREAK_VARIABLE, '').split(',') if item}


class Refusal(Exception):
    """用法错或输入不合：message 写原因，way_out 写下一步。"""

    def __init__(self, message, way_out):
        super().__init__(message)
        self.way_out = way_out


def manifest_paths(repository, round_name, listed):
    """→ (快照目录, 读过的清单文件名, 去重后按清单次序排的路径)。"""
    directory = os.path.join(repository, 'research', 'prompts', f'{round_name}-snapshot')
    if not os.path.isdir(directory):
        raise Refusal(f'找不到这一轮的开工快照目录 {directory}',
                      '判决骨架照开工快照的 sha256 清单预填被判过的文件；派腿之前先 sha256sum <判决要点名的文件…> > research/prompts/<轮>-snapshot/crates-sha256.txt'
                      '（定义与规则这类写 judged-sha256.txt），再来生成骨架')
    if listed:
        names = list(listed)
        missing = [name for name in names if not os.path.isfile(os.path.join(directory, name))]
        if missing:
            raise Refusal(f'--list 点名的清单不在 {directory} 里：{"、".join(missing)}', f'核对文件名，ls {directory}')
    else:
        names = sorted(name for name in os.listdir(directory)
                       if 'sha256' in name and name.endswith('.txt') and os.path.isfile(os.path.join(directory, name))
                       and (name != KB_MANIFEST or 'kb-manifest-read' in broken_items()))
    if not names and 'empty-snapshot-accepted' not in broken_items():
        raise Refusal(f'{directory} 里没有可读的 sha256 清单（kb-sha256.txt 不算）',
                      '先 sha256sum <判决要点名的文件…> > <快照目录>/crates-sha256.txt（定义与规则这类写 judged-sha256.txt）；清单名不带 sha256 的用 --list 点名')
    paths = []
    for name in names:
        with open(os.path.join(directory, name), encoding='utf-8') as handle:
            for number, line in enumerate(handle.read().split('\n'), start=1):
                if not line.strip():
                    continue
                match = MANIFEST_LINE.match(line.rstrip('\r'))
                if match is None:
                    raise Refusal(f'{directory}/{name}:{number} 不是 sha256sum 的格式（64 位十六进制、两个空格、路径）：{line[:80]}',
                                  '清单用 sha256sum 原样写出，不手改')
                path = match.group('path')
                path = path[2:] if path.startswith('./') else path
                if path not in paths:
                    paths.append(path)
    if not paths and 'empty-snapshot-accepted' not in broken_items():
        raise Refusal(f'{directory} 里读过的清单（{"、".join(names)}）一行路径都没有', '清单是空的：重记开工快照')
    return directory, names, paths


def skeleton_text(round_name, paths):
    placeholder = '（待填：没判的路径删掉这一行）' if 'placeholder-negated' in broken_items() else '（待填：判了它什么）'
    reviewed_heading = REVIEWED_HEADING.replace('文件', '文档') if 'reviewed-heading-typo' in broken_items() else REVIEWED_HEADING
    today = datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).date().isoformat()
    lines = [
        f'# {round_name}：判决（{today}）',
        '',
        f'正文 `research/prompts/_{round_name}-body.md`，开工快照 `research/prompts/{round_name}-snapshot/`。（待填：这是这一题的第几轮，前几轮的判决）',
        '',
        HEADINGS[0], '',
        '| 腿 | 报告 | 结论（腿的原话，当线索） |', '|---|---|---|', '| （待填） | （待填） | （待填） |', '',
        READINGS_PROMPT.replace('，不写任务号）', '，写任务号）') if 'readings-task-number-asked' in broken_items() else READINGS_PROMPT, '',
        HEADINGS[1], '', '（待填）', '',
        HEADINGS[2], '',
        ZERO_ROUND_PROMPT,
        *([] if 'zero-round-reading-line-dropped' in broken_items() else [ZERO_ROUND_READING_PLACEHOLDER]), '',
        HEADINGS[3], '', '（待填）', '',
        reviewed_heading, '',
        *[f'- {path}：{placeholder}' for path in paths], '',
        HEADINGS[5], '',
        '| 决策分项 | 关系 | 回看 |', '|---|---|---|', '| （待填） | （待填） | （待填） |', '',
    ]
    return '\n'.join(lines)


def write_skeleton(repository, round_name, out, listed):
    if not ROUND_NAME.match(round_name) or round_name.endswith('-main-verification'):
        raise Refusal(f'轮名 {round_name!r} 不合：只许字母、数字、点、下划线、连字符，首字符是字母或数字，不带 -main-verification 后缀', '轮名就是 research/prompts/<轮>-snapshot/ 里的那个 <轮>')
    directory, names, paths = manifest_paths(repository, round_name, listed)
    target = out or os.path.join(repository, 'research', 'prompts', f'{round_name}-main-verification.md')
    mode = 'w' if 'overwrite-existing' in broken_items() else 'x'
    try:
        with open(target, mode, encoding='utf-8') as handle:
            handle.write(skeleton_text(round_name, paths))
    except FileExistsError:
        raise Refusal(f'{target} 已经在了，不覆盖', '那一份是已有的判决：接着写它；要重起一份换 --out 另给路径') from None
    except OSError as error:
        raise Refusal(f'写不了 {target}：{error.strerror}', '核对目录在不在、有没有写权限；--out 的父目录要先有') from None
    print(f'verdict-skeleton：写好 {target}（读了 {directory} 里的 {"、".join(names)}，预填 {len(paths)} 个被判过的路径）')
    return 0


def gate_judge(repository, verdict):
    """用门禁自己的两个函数（review_section_text、review_drop_negated）和它的标题、否定词常量判一份判决，→ 点名一节认下的那段文字。
    门禁文件里抽不出这几样就抛 Refusal：自证判不了，不当绿。"""
    gate_file = os.path.join(repository, GATE_PATH)
    try:
        gate = open(gate_file, encoding='utf-8').read()
    except OSError as error:
        raise Refusal(f'读不了门禁 {gate_file}：{error.strerror}', '自证拿门禁的点名判法核骨架，门禁文件要在')
    pieces = []
    for variable in ('REVIEWED_SECTION_HEADING', 'REVIEWED_SECTION_NEGATIONS'):
        match = re.search(rf'^{variable}=.*$', gate, re.M)
        if match is None:
            raise Refusal(f'{gate_file} 里找不到 {variable}= 那一行', '门禁改了写法：这份自证跟着改')
        pieces.append(match.group(0))
    for function in ('review_section_text', 'review_drop_negated'):
        match = re.search(rf'^{function}\(\) \{{\n.*?^\}}$', gate, re.M | re.S)
        if match is None:
            raise Refusal(f'{gate_file} 里找不到函数 {function}', '门禁改了写法：这份自证跟着改')
        pieces.append(match.group(0))
    pieces.append('review_section_text "$1" | review_drop_negated')
    with tempfile.TemporaryDirectory() as directory:
        script = os.path.join(directory, 'judge.sh')
        with open(script, 'w', encoding='utf-8') as handle:
            handle.write('\n'.join(pieces) + '\n')
        done = subprocess.run(['bash', script, verdict], capture_output=True, text=True)
    if done.returncode != 0:
        raise Refusal(f'门禁的点名判法跑不起来（退 {done.returncode}）：{done.stderr[-200:]}', '门禁改了写法：这份自证跟着改')
    return done.stdout


def dispatch_guard_reading_patterns(repository):
    """派发闸认零轮读数那一行的三条（LINE_START、ZERO_ROUND_READING_LINE、ZERO_ROUND_TASK_NUMBER）从它的源码原样抽出来执行 → (读数行, 任务号) 两条正则。
    抽不出就抛 Refusal：自证判不了，不当绿。"""
    guard_file = os.path.join(repository, DISPATCH_GUARD_PATH)
    try:
        guard = open(guard_file, encoding='utf-8').read()
    except OSError as error:
        raise Refusal(f'读不了派发闸 {guard_file}：{error.strerror}', '自证拿派发闸的认法核骨架里的 A/B 读数一行，派发闸文件要在')
    namespace = {'re': re}
    for name in ('LINE_START', 'ZERO_ROUND_READING_LINE', 'ZERO_ROUND_TASK_NUMBER'):
        match = re.search(rf'^{name} = .*$', guard, re.M)
        if match is None:
            raise Refusal(f'{guard_file} 里找不到 {name} = 那一行', '派发闸改了写法：这份自证跟着改')
        exec(match.group(0), namespace)
    return namespace['ZERO_ROUND_READING_LINE'], namespace['ZERO_ROUND_TASK_NUMBER']


def notations_library(repository):
    """判据库 .claude/gate.d/lib-forbidden-notations.py（不留 __pycache__）；读不到抛 Refusal。"""
    library_file = os.path.join(repository, NOTATIONS_LIBRARY_PATH)
    try:
        specification = importlib.util.spec_from_file_location('lib_forbidden_notations', library_file)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
    except (OSError, SyntaxError, AttributeError) as error:
        raise Refusal(f'读不了判据库 {library_file}：{error}', '判据库是写闸与检出钩子共用的那一份，先 git status 看它在不在、是不是被改坏了')
    return library


def selftest():
    os.environ[SELFTEST_MARK_VARIABLE] = SELFTEST_MARK_CONTENT   # 自证起的子进程（环境原样继承）靠它认：弄坏开关只在自证里认
    failures = []

    def red(summary, detail):
        failures.append(summary)
        print(f'  ✗ 自检：{summary}')
        print(f'    {detail}')

    def run(arguments):
        done = subprocess.run([sys.executable, SCRIPT, *arguments], capture_output=True, text=True)
        return done.returncode, done.stdout + done.stderr

    sample_paths = ['crates/demo/src/mounted_session.rs', '.claude/agents/中文名.md', 'scripts/agents/kept.py']
    kb_path = 'kb/should-not-appear.md'
    digest = '0' * 63 + '1'
    with tempfile.TemporaryDirectory() as root:
        prompts = os.path.join(root, 'research', 'prompts')
        snapshot = os.path.join(prompts, 'demo-r1-snapshot')
        os.makedirs(snapshot)
        with open(os.path.join(snapshot, 'crates-sha256.txt'), 'w', encoding='utf-8') as handle:
            handle.write(f'{digest}  {sample_paths[0]}\n{digest} *./{sample_paths[1]}\n')
        with open(os.path.join(snapshot, 'agents-sha256.txt'), 'w', encoding='utf-8') as handle:
            handle.write(f'{digest}  {sample_paths[2]}\n{digest}  {sample_paths[0]}\n')
        with open(os.path.join(snapshot, KB_MANIFEST), 'w', encoding='utf-8') as handle:
            handle.write(f'{digest}  {kb_path}\n')
        default_out = os.path.join(prompts, 'demo-r1-main-verification.md')
        code, said = run(['demo-r1', '--repository', root])
        if code != 0 or not os.path.isfile(default_out):
            red('不给 --out 应当写到 research/prompts/<轮>-main-verification.md、退 0', f'实际退 {code}：{said[-300:]}')
            text = ''
        else:
            text = open(default_out, encoding='utf-8').read()
            # 门禁自己的点名判法认得出每个路径（路径逐个在节里字面出现）
            try:
                scope = gate_judge(DEFAULT_REPOSITORY, default_out)
            except Refusal as refusal:
                red('门禁的点名判法要拿得到', f'{refusal}；{refusal.way_out}')
                scope = None
            if scope is not None:
                unseen = [path for path in sample_paths if path not in scope]
                if unseen:
                    red('门禁点名一节的判法应当认得出骨架里的每个路径', f'认不出：{"、".join(unseen)}；门禁认下的那段文字共 {len(scope.splitlines())} 行')
            lines = text.split('\n')
            positions = [lines.index(heading) if heading in lines else -1 for heading in HEADINGS]
            if -1 in positions or positions != sorted(positions) or any(lines.count(heading) != 1 for heading in HEADINGS):
                red('骨架应当逐字带这几节标题、各一次、按次序', f'标题行号 {positions}（-1 是没有这一行）')
            if kb_path in text:
                red('默认不读 kb-sha256.txt', f'骨架里出现了 {kb_path}')
            listed_lines = [line for line in lines if line.startswith('- ') and '：（待填：' in line]
            if len(listed_lines) != len(sample_paths):
                red('被判过的文件一节应当每个路径一行、同一路径只一行', f'读到 {len(listed_lines)} 行：{listed_lines}')
            # 判决里的号只写零轮改法那一行：待填提示不叫人写任务号；「三、改法」带一行派发闸认的「A/B 读数：任务 <号>」，填上号之后判据库在判决里放过、在记录里照拒
            asked = [line for line in lines if '写任务号' in line.replace('不写任务号', '')]
            if asked:
                red('骨架的待填提示不要求把任务号写进判决', f'这几行叫人写任务号：{asked}')
            try:
                reading_line, task_number = dispatch_guard_reading_patterns(DEFAULT_REPOSITORY)
                library = notations_library(DEFAULT_REPOSITORY)
            except Refusal as refusal:
                red('派发闸的读数认法与判据库要拿得到', f'{refusal}；{refusal.way_out}')
                reading_line = None
            if reading_line is not None:
                section = lines[positions[2] + 1:positions[3]] if -1 not in positions[2:4] else []
                filled = [line.replace('<号>', SAMPLE_TASK_NUMBER) for line in section if '<号>' in line and reading_line.match(line.replace('<号>', SAMPLE_TASK_NUMBER))]
                filled = [line for line in filled if task_number.search(reading_line.match(line).group('value'))]
                if not filled:
                    red('骨架里零轮改法那一条带一行派发闸认的 A/B 读数', f'「三、改法」一节里没有一行照派发闸 {DISPATCH_GUARD_PATH} 的 ZERO_ROUND_READING_LINE 与 ZERO_ROUND_TASK_NUMBER 认得出：{section}')
                verdict_hits = [hit for line in filled for hit in library.all_hits(NUMBERS_FORM, line, 'research/prompts/demo-r1-main-verification.md')]
                record_hits = [hit for line in filled for hit in library.all_hits(NUMBERS_FORM, line, 'records/demo.md')]
                if filled and (verdict_hits or not record_hits):
                    red('判据库应当在判决里放过骨架那一行填上号之后的样子、在记录里照拒', f'判决里命中 {verdict_hits}，记录里命中 {record_hits}（填好的行：{filled}）')
        # 排他新建
        if os.path.isfile(default_out):
            before = open(default_out, encoding='utf-8').read()
            handwritten = before + '\n手写的一行\n'
            with open(default_out, 'w', encoding='utf-8') as handle:
                handle.write(handwritten)
            code, said = run(['demo-r1', '--repository', root])
            untouched = open(default_out, encoding='utf-8').read() == handwritten
            if code != 2 or not untouched or '→ 怎么办' not in said:
                red('输出文件已在应当退 2、原文件不动', f'退 {code}，原文件{"未动" if untouched else "被改"}：{said[-200:]}')
        # --out 另给路径；--list 只读点名的清单
        other = os.path.join(root, 'other.md')
        code, said = run(['demo-r1', '--repository', root, '--out', other, '--list', 'agents-sha256.txt'])
        other_text = open(other, encoding='utf-8').read() if os.path.isfile(other) else ''
        if code != 0 or sample_paths[1] in other_text or sample_paths[2] not in other_text:
            red('--out 与 --list 应当各管各的：写到 --out、只读点名的清单', f'退 {code}：{said[-200:]}')
        # 没有快照目录、只有 kb 清单、清单行不合、轮名不合
        code, said = run(['nowhere-r1', '--repository', root])
        if code != 2 or os.path.join(prompts, 'nowhere-r1-snapshot') not in said or '→ 怎么办' not in said:
            red('没有快照目录应当退 2、说出该有的路径', f'退 {code}：{said[-300:]}')
        only_kb = os.path.join(prompts, 'onlykb-r1-snapshot')
        os.makedirs(only_kb)
        with open(os.path.join(only_kb, KB_MANIFEST), 'w', encoding='utf-8') as handle:
            handle.write(f'{digest}  {kb_path}\n')
        code, said = run(['onlykb-r1', '--repository', root])
        if code != 2 or os.path.isfile(os.path.join(prompts, 'onlykb-r1-main-verification.md')):
            red('快照目录里没有清单应当退 2', f'退 {code}（只有 kb-sha256.txt 不算清单）：{said[-300:]}')
        bad = os.path.join(prompts, 'bad-r1-snapshot')
        os.makedirs(bad)
        with open(os.path.join(bad, 'crates-sha256.txt'), 'w', encoding='utf-8') as handle:
            handle.write(f'{digest}  fine.rs\nnot a manifest line\n')
        code, said = run(['bad-r1', '--repository', root])
        if code != 2 or 'crates-sha256.txt:2' not in said:
            red('清单里有不是 sha256sum 格式的行应当退 2、点名文件与行号', f'退 {code}：{said[-300:]}')
        code, said = run(['../escape', '--repository', root])
        if code != 2:
            red('轮名带路径应当退 2', f'退 {code}：{said[-200:]}')
        # 命令行
        for arguments, want_code, want_text in (([ '--help'], 0, '用法：'), (['-h'], 0, '用法：'), (['--bogus'], 2, '不认的选项'), ([], 2, '缺轮名')):
            code, said = run(arguments)
            if code != want_code or want_text not in said or 'Traceback' in said:
                red(f'命令行 {arguments} 应当退 {want_code}、说「{want_text}」、不抛 traceback', f'退 {code}：{said[-200:]}')
    if failures:
        print(f'  ✗ 自检：{len(failures)} 格红')
        print('     → 怎么办：看上面每一行「✗ 自检：<格名>」和它下面缩进的明细，按那一格改 verdict-skeleton.py 的骨架或判法，再跑一遍 --selftest')
        return 1
    print(f'  ✓ 自检通过：骨架逐字带 {len(HEADINGS)} 节标题，门禁的点名判法认得出每个预填路径（含中文文件名、./ 与 * 写法、多份清单去重），'
          'kb 清单默认不读，已有文件不覆盖，没有快照与坏清单与坏轮名都退 2；待填提示不叫人写任务号，「三、改法」带一行派发闸认的「A/B 读数：任务 <号>」，'
          '填上号之后判据库在判决里放过、在记录里照拒')
    return 0


def refuse(refusal):
    print(f'✗ {refusal}\n  → 怎么办：{refusal.way_out}', file=sys.stderr)
    return 2


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在自证里认：命令行不是单独一个 --selftest、环境里也没有自证留的标记而开关非空，参数解析之前就拒（退 2）；放行交回 None。"""
    value = os.environ.get(BREAK_VARIABLE)
    if not value or arguments == ['--selftest'] or os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT:
        return None
    print(f'✗ {BREAK_VARIABLE}={value} 是弄坏开关，只在自证（verdict-skeleton.py --selftest）里认，平常跑设了就拒，这一趟什么都没做', file=sys.stderr)
    print(f'  → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 scripts/agents/verdict-skeleton.py --selftest', file=sys.stderr)
    return 2


def main():
    arguments = sys.argv[1:]
    refused = refuse_break_outside_selftest(arguments)
    if refused is not None:
        return refused
    unknown = sorted(broken_items() - set(BREAK_ITEMS))
    if unknown:
        print(f'✗ {BREAK_VARIABLE} 里有不认的项：{"、".join(unknown)}\n  → 怎么办：只认 {"、".join(BREAK_ITEMS)}', file=sys.stderr)
        return 2
    if arguments == ['--selftest']:
        return selftest()
    usage = 'verdict-skeleton.py <轮名> [--out <路径>] [--repository <仓根>] [--list <清单文件名> …]'
    if not arguments:
        return refuse(Refusal('缺轮名', usage))
    if arguments[0] in ('--help', '-h'):
        print(__doc__)
        return 0
    round_name = None
    out = None
    repository = DEFAULT_REPOSITORY
    listed = []
    index = 0
    while index < len(arguments):
        flag = arguments[index]
        if flag in ('--out', '--repository', '--list'):
            if index + 1 >= len(arguments):
                return refuse(Refusal(f'{flag} 后面没给值', usage))
            value = arguments[index + 1]
            index += 2
            if flag == '--out':
                out = value
            elif flag == '--repository':
                repository = value
            else:
                listed.append(value)
        elif flag.startswith('-'):
            return refuse(Refusal(f'不认的选项：{flag}', f'只认 --out、--repository、--list（{usage}）'))
        elif round_name is None:
            round_name = flag
            index += 1
        else:
            return refuse(Refusal(f'多给了参数 {flag}（轮名已经是 {round_name}）', usage))
    if round_name is None:
        return refuse(Refusal('缺轮名', usage))
    try:
        return write_skeleton(repository, round_name, out, listed)
    except Refusal as refusal:
        return refuse(refusal)


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main())
