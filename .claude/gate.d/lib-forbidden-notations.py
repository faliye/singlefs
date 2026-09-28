#!/usr/bin/env python3
"""全仓禁用写法的登记：门禁 doc-text.sh（提交时全仓扫，格名就是形态的键）与 .claude/hooks/write-guard.sh（写入那一刻拒绝）共用这一份，不各抄一份。

FORMS 一项一种形态，加一种禁用写法就是往 FORMS 里加一项。每一项登记：
- key、name：形态键与名字。
- patterns：判据行 (键, 名字, 正则, 上下文规则)，一种形态可以有几行；一行文字命中其中任一条就算这一形态的一处违例，
  报最靠前的那一处（同一位置几行都中，报排在前面的那一行）。上下文规则是 None，或一个函数 (整行, 命中) → True 表示
  这一处按上下文不是违例、接着找这一行后面的命中。
- excluded_prefixes：(路径前缀, 理由)，相对仓根；这些文件里这一形态是被测输入或只能在上游改，不判。
- allow_suffixes：哪几种后缀的文件认逐行的放行标记（一行里写「# <形态键>:allow <理由>」，Rust 写「//」，理由至少 4 个字）；
  空的就是这一形态不认放行标记。
- clean：成功行「查了 N 份文本文件，」后面接的话。
- excluded_label：成功行「没扫的：」里这一形态排除掉的那几份怎么说（{count} 处填份数）。
- finding：判红汇总行「N 」后面接的话。
- detail_shows_pattern：判红明细行点不点名命中的判据行名字。
- snippet_to_match_end：明细里的片段往后取 10 个字，从命中的结尾算（True）还是从开头算（False）。
- remedy：判红的出路，一项一行；阶段在第一行前面加「→ 怎么办：」。

角标（prime-marks）：撇号类字符 U+2032、U+2033、U+2034、U+02B9、U+02BA 任一个出现就算，给变体起名见
.claude/rules/path-moves.md「变体起新名字，不用角标」；ASCII 单引号不在其内（机器分不开它与引号）。
这份源码里几个字符用码位拼，不出现字面：doc-text.sh 扫全仓，也扫这一份。只排除上游副本：门禁样本也扫，红样本的角标由 setup.sh 现写。

钟点（clock-times）：时间只写到日期，钟点、时区词、时间戳都不写。判据行认的是：
- 时:分 这一族：日期后面跟钟点（「日期 时区词 时:分」、diff 头与 stat 那种「日期 时:分:秒.毫秒 +0000」）、时区词后面跟钟点、
  钟点后面跟时区词、带 x 通配的「时:分x」、不带日期的 Z 钟点，与不挨日期时区的钟点（时:分、时:分:秒、带毫秒、区间、「月-日 时:分」）。
- ISO 时间戳：日期与钟点之间是 T 的（带不带日期、带不带时区偏移或 Z 都算），date 的原样输出也在内。
- 时区词：ZONE_WORDS 里那两个缩写，大写、前后不挨 ASCII 字母数字与下划线（标识符里的不算）。
- 中文的「N 点 / N 时」：挨在日期或时区词后面的（中间可有空格、「次日」；「时」后面跟间、代、刻、序、延、钟、效、长、候、机、区、态的
  是时间、时区这一类别的词，不算）；不挨日期、时区的，只认后面跟「前后 / 左右 / 许 / 半 / 钟」的「N 点」与后面跟「许」的「N 时」
  （前面是「第」的是序号，不算）。
- 时段词（PERIOD_WORDS）：挨在日期或时区词后面的（中间可隔空格、全角或半角左括号、全角逗号），与后面跟「N 点 / N 时」或时:分的。
  不挨日期、时区、数字的光秃时段词不判。
「东京时间」不认（仓里有用户原话引文用它，改不得）。
上下文规则放过长得像钟点、其实不是的冒号对（不挨日期时区的钟点那一行）：
- 紧挨在前面的是 = [ { , > " ' + - . 之一或「, 」：产物行的「名字=值」、切片、字典与逗号列表、「0->1:10」、引号里的代码字面、
  时区偏移、小数；前面是反引号而小时只有一位的，是反引号里的键（`3:10`）；
- 紧跟在后面的是 %、「.数字」或「-字母」：百分比、「分:秒.百分之一秒」（time -v 的格式）、门禁文件名（「20:54-layer0-replay.sh」这类）；
- 同一行里有取值出了钟点范围的冒号对（时大于 23、分大于 59、一边三位或分只有一位）或「文件:行号」（「.md:42」这类）的，
  这一行的冒号对按「门禁号:行号」「决策号:行号」这类编号读。
ISO 时间戳与「日期后面跟钟点」只放过紧跟在「=」后面的（产物行的字段值，「taken_jst=<日期>T<钟点>」这类）。
排除的是把钟点当被测输入或期望输出用的文件（hook 自检的假日志、门禁样本、以时区为被测算法的 c510 那一轮）与上游副本。
管不到的：同一行既有编号对又有真钟点的，真钟点跟着一起放过；引号、反引号里两位小时以外的代码字面按上面那几条放过。

弄坏开关（环境变量，逗号分隔，可以写几个键；每次现读，同一个进程里切换也认）：
- LIB_FORBIDDEN_NOTATIONS_BREAK=<判据行键或形态键>：那几条判据行（写形态键的，那一形态的全部判据行）不判。
  打开时 doc-text.sh 那一形态的红样本（prime-marks-red、clock-times-red）、write-guard.sh --selftest 里点名那一行的格必须判红，证判据真在管那一行。
- LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=<判据行键或形态键>：那几条判据行的上下文规则不生效，命中一律算违例。
  打开时 doc-text.sh 的 clock-times-green 样本里的冒号对、write-guard.sh --selftest 里靠上下文放行的格必须判红，证放行靠的是上下文规则。
- LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=<形态键>：那几种形态的放行标记不认。
  打开时 doc-text.sh 的 clock-times-green 样本与 write-guard.sh --selftest 里带放行标记的格必须判红。
- LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=<形态键>：那几种形态的 excluded_prefixes 不生效。
  打开时 doc-text.sh 那一形态的绿样本（prime-marks-green、clock-times-green）里落在排除前缀下的放行格必须判红，证排除前缀只放过登记的那几处。
"""
import os
import re

# U+2032 PRIME、U+2033 DOUBLE PRIME、U+2034 TRIPLE PRIME、U+02B9 MODIFIER LETTER PRIME、U+02BA MODIFIER LETTER DOUBLE PRIME
PRIME_MARKS = ''.join(map(chr, (0x2032, 0x2033, 0x2034, 0x02B9, 0x02BA)))
MARKS_SHOWN = ' '.join(PRIME_MARKS)

DATE = r'20\d\d-\d\d-\d\d'
CLOCK = r'(?<![\d:])\d{1,2}:(?:\d\d|\dx|xx)(?::\d\d(?:\.\d+)?)?(?![\d:]|\.\d)'
CLOCK_RANGE = rf'{CLOCK}(?:\s*[–~\-]\s*(?:次日\s*)?{CLOCK})?'
ZONE_WORDS = ('JST', 'UTC')  # clock-times:allow 判据自己的时区词表
ZONE = '(?:' + '|'.join(ZONE_WORDS) + ')'
NEXT_DAY = r'(?:次日\s*)?'
# 「N 点 / N 时」：「时」后面跟这几个字的是时间、时代、时刻、时序、时延、时钟、时效、时长、时候、时机、时区、时态，不是钟点
HOUR = r'(?<![\d:])\d{1,2}\s*(?:点|时(?![间代刻序延钟效长候机区态]))'
PERIOD_WORDS = ('上午', '下午', '凌晨', '早上', '早晨', '清晨', '中午', '午后', '傍晚', '晚上', '夜里', '深夜', '半夜')
PERIOD = '(?:' + '|'.join(PERIOD_WORDS) + ')'
# ASCII 的字母数字与下划线（Python 的 \w 连汉字也算，钟点挨着汉字照样要判）
ASCII_WORD = 'A-Za-z0-9_'
HOUR_VALUE = r'(?:[01]?\d|2[0-3])'
MINUTE_VALUE = r'[0-5]\d'
BARE_CLOCK = rf'(?<![{ASCII_WORD}:]){HOUR_VALUE}:{MINUTE_VALUE}(?::{MINUTE_VALUE}(?:\.\d+)?)?(?![{ASCII_WORD}:])'
ISO_TIMESTAMP = rf'(?:{DATE}|(?<![{ASCII_WORD}\-]))T(?:[01]\d|2[0-3]):{MINUTE_VALUE}'
ZONE_WORD = rf'(?<![{ASCII_WORD}]){ZONE}(?![{ASCII_WORD}])'

# 上下文规则用的：紧挨在冒号对前面、说明它是代码或产物字段的字符
CODE_CHARACTERS_BEFORE = ('=', '[', '{', ',', '>', '"', "'", '+', '-', '.')
COLON_PAIR = re.compile(rf'(?<![{ASCII_WORD}:.])(\d{{1,3}}):(\d{{1,3}})(?![{ASCII_WORD}:])')
FILE_LINE_REFERENCE = re.compile(r'\.[A-Za-z]\w*:\d')


def numbered_pairs_line(line):
    """这一行的冒号对按编号读：有取值出了钟点范围的冒号对，或有「文件:行号」。"""
    if FILE_LINE_REFERENCE.search(line):
        return True
    return any(int(hour) > 23 or int(minute) > 59 or len(hour) > 2 or len(minute) != 2 for hour, minute in COLON_PAIR.findall(line))


def bare_clock_is_not_clock(line, match):
    """不挨日期时区的钟点那一行的上下文规则：True 是这一处按上下文不是钟点。"""
    before, after = line[:match.start()], line[match.end():]
    if before.endswith(CODE_CHARACTERS_BEFORE) or before.endswith(', '):
        return True
    if before.endswith('`') and len(match.group(0).split(':')[0]) == 1:
        return True
    if after.startswith('%') or re.match(r'\.\d|-[A-Za-z]', after):
        return True
    return numbered_pairs_line(line)


def field_value(line, match):
    """紧跟在「=」后面的是产物行的字段值（名字=值）。"""
    return line[:match.start()].endswith('=')


UPSTREAM_COPY = '.claude/singlefs-ai-sop/'
C510_REASON = 'c510 那一轮论证的就是按哪个时区取日期，时区词与钟点是被测算法本身'

FORMS = (
    {
        'key': 'prime-marks',
        'name': '角标',
        'patterns': (
            ('prime-marks', '撇号类角标', re.compile('[' + re.escape(PRIME_MARKS) + ']'), None),
        ),
        'excluded_prefixes': (
            (UPSTREAM_COPY, '上游副本，只能在上游改'),
        ),
        'allow_suffixes': (),
        'clean': f'没有角标写法（{MARKS_SHOWN}）',
        'excluded_label': '上游副本 ' + UPSTREAM_COPY + ' 下 {count} 份（只能在上游改）',
        'finding': f'行用了角标写法（{MARKS_SHOWN}）',
        'detail_shows_pattern': False,
        'snippet_to_match_end': False,
        'remedy': (
            '给这个变体起一个新名字——在它那一族里取下一个没用过的号，或起一个短的描述性名字，全仓一次改完；',
            '撞号先查（新名字在它那一族的全部文件里零命中）。做法见 .claude/rules/path-moves.md「变体起新名字，不用角标」。',
        ),
    },
    {
        'key': 'clock-times',
        'name': '钟点',
        'patterns': (
            ('date-clock', '日期后面跟钟点', re.compile(rf'{DATE} ?(?:{ZONE} ?)?{CLOCK_RANGE}'), field_value),
            ('zone-clock', '时区后面跟钟点', re.compile(rf'{ZONE} ?{NEXT_DAY}{CLOCK_RANGE}'), None),
            ('clock-zone', '钟点后面跟时区', re.compile(rf'{CLOCK_RANGE} ?{ZONE}'), None),
            ('x-wildcard', '带 x 通配的钟点', re.compile(r'(?<![\w:])(?:\d{1,2}|\dx):(?:\dx|xx)'), None),
            ('bare-z', '不带日期的 Z 钟点', re.compile(r'(?<![\w:`])\d{1,2}:\d\d(?::\d\d)?Z(?!\w)'), None),
            ('date-zone-hour', '日期或时区后面跟「N 点 / N 时」', re.compile(rf'(?:{DATE}|{ZONE})\s*(?:{ZONE}\s*)?{NEXT_DAY}{HOUR}'), None),
            ('date-zone-period', '日期或时区后面跟时段词', re.compile(rf'(?:{DATE}(?:\s*{ZONE})?|{ZONE})[\s（(，]*{NEXT_DAY}{PERIOD}'), None),
            ('period-hour', '时段词后面跟钟点', re.compile(rf'{PERIOD}\s*(?:{HOUR}|{CLOCK})'), None),
            ('hour-suffix', '「N 点前后」这类钟点', re.compile(r'(?<![第\d])(?<!第 )\d{1,2}\s*(?:点(?:前后|左右|许|半|钟)|时许)'), None),
            ('bare-clock', '不挨日期时区的钟点', re.compile(BARE_CLOCK), bare_clock_is_not_clock),
            ('iso-timestamp', 'ISO 时间戳', re.compile(ISO_TIMESTAMP), field_value),
            ('zone-word', '时区词', re.compile(ZONE_WORD), None),
        ),
        'excluded_prefixes': (
            ('.claude/hooks/session-start.sh', '自检用的假 journalctl 行与它打出来的期望输出，钟点是被测输入'),
            ('.claude/gate.d/fixtures/', '门禁样本：造现场的 touch -d、GIT_COMMITTER_DATE 与门禁打出来的期望输出'),
            (UPSTREAM_COPY, '上游副本，只能在上游改'),
            ('research/prompts/_c510-date-gate-r1-appendix.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-attack-output.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-background.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-body.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-checklist.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-forward-output.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-local-attack-output.md', C510_REASON),
            ('research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack.md', C510_REASON),
            ('research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-output-s1.md', C510_REASON),
            ('research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-output-s2.md', C510_REASON),
            ('research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-translation-audit.md', C510_REASON),
            ('research/prompts/c510-date-gate-r1-verifier-output.md', C510_REASON),
        ),
        'allow_suffixes': ('.sh', '.py', '.rs'),
        'clean': '描述里没有钟点、时区词与时间戳',
        'excluded_label': '钟点的排除前缀下 {count} 份（钟点是被测输入，或上游副本）',
        'finding': '行在描述里写了钟点、时区词或时间戳',
        'detail_shows_pattern': True,
        'snippet_to_match_end': True,
        'remedy': (
            '时间只写到日期，把钟点连同挨着它的时区词、「前后」一起删掉（例：「用户 <日期> <时区词> <时:分> 定」写成「用户 <日期> 定」）；',
            '「N 点 / N 时」与时段词同样删（例：「<日期> <时区词> <N> 点前后要开」「交回于 <日期> <上午 / 下午>」「<时段词> <N> 点」'
            '都写成「<日期> 要开」「交回于 <日期>」这样只到日期）；',
            '光秃的时区词（lib-forbidden-notations.py 的 ZONE_WORDS）也删；ISO 时间戳、diff 头与 stat 的机器时间、date 的原样输出不贴进仓，只写日期；',
            '命中的冒号对其实不是钟点（产物字段、切片、编号对），改那份库里「不挨日期时区的钟点」的上下文规则，并在 doc-text.sh 的 clock-times-green 样本补一行；',
            '.sh、.py、.rs 里代码真要用这些字面（TZ 环境变量、解析外部文本的正则、造时间戳的自检输入），在那一行写'
            '「# clock-times:allow <理由>」（Rust 写「//」），理由不许省；',
            '钟点确实是被测输入的整份文件，登记进 .claude/gate.d/lib-forbidden-notations.py 里钟点那一形态的 excluded_prefixes 并写明理由。',
        ),
    },
)

FORMS_BY_KEY = {form['key']: form for form in FORMS}


def environment_keys(variable):
    return {key.strip() for key in os.environ.get(variable, '').split(',') if key.strip()}


def is_excluded(form_key, relative_path):
    """这份文件（相对仓根）在不在这一形态的排除前缀下；弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS 点名这一形态时一律不排除。"""
    if form_key in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS'):
        return False
    return any(relative_path.startswith(prefix) for prefix, _ in FORMS_BY_KEY[form_key]['excluded_prefixes'])


def allowed_by_marker(form_key, line, relative_path):
    """这一行带这一形态的放行标记（后缀在 allow_suffixes 里、标记后面写了至少 4 个字的理由）；LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW 点名这一形态时一律不认。"""
    form = FORMS_BY_KEY[form_key]
    if not form['allow_suffixes'] or not relative_path.endswith(form['allow_suffixes']):
        return False
    if form_key in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW'):
        return False
    return re.search(rf'(?:#|//)\s*{re.escape(form_key)}:allow[ \t]+\S.{{3,}}', line) is not None


def first_hit(form_key, text, relative_path=''):
    """这一形态在 text 里最靠前的一处：(位置, 判据行名字, 命中的串)；没有返回 None。
    relative_path 是 text 所在文件相对仓根的路径，用来认放行标记；不给就不认。"""
    skipped = environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK')
    if form_key in skipped:
        return None
    context_off = environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT')
    patterns = [(index, key, name, pattern, None if key in context_off or form_key in context_off else not_violation)
                for index, (key, name, pattern, not_violation) in enumerate(FORMS_BY_KEY[form_key]['patterns']) if key not in skipped]
    offset = 0
    for line in text.split('\n'):
        if not allowed_by_marker(form_key, line, relative_path):
            found = []
            for index, key, name, pattern, not_violation in patterns:
                match = next((candidate for candidate in pattern.finditer(line)
                              if not_violation is None or not not_violation(line, candidate)), None)
                if match:
                    found.append((match.start(), index, name, match.group(0)))
            if found:
                position, _, name, matched = min(found)
                return offset + position, name, matched
        offset += len(line) + 1
    return None
