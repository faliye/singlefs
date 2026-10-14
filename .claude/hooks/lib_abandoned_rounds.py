"""叫停轮登记表 research/prompts/abandoned-rounds.tsv 的一行怎么判：派发闸 ⑥ 与门禁 doc-process-records 的 abandoned-rounds 格共用这一份。

两处用这一份：
  .claude/hooks/runner-dispatch-guard.sh      ⑥ 开工快照：登记行判得收，那一轮才不算开着（写回员、实现员不再被它的快照挡）
  .claude/gate.d/doc-process-records.sh       abandoned-rounds 格：登记行判得不收，门禁红

同一张表只有一份判法，三样两边都判，「不收」与「红」是同一个结局：
  1. 形状：四列不空；第二列日期是 YYYY-MM-DD（shape_problem）。
  2. 同一轮重复：同一个轮名有两行合格的行就都不收（duplicated_rounds）。
  3. 出处（row_source_problem）：整行里有出处「records/<文件>.md 第 N 行〔<原话>〕」（或 records/<文件>.md:N〔<原话>〕），
     〔〕里抄那份文件里含这一轮名字的那一行的一段原话（照抄，不改字；不含〔〕、制表符与换行）；
     文件是 records/ 下现存的 .md（路径里没有 ..，解到真路径后仍在 records/ 之内）。拿〔〕里的原话在那份文件里找（locate_excerpt），按这三种取「那一行」，取到了才往下判：
       恰好命中一次：命中的那一行；行号只当提示，对不上而原话找得到照收（Judgement.moved 记下挪到第几行）；
       命中几处而行号提示正落在其中一处：提示的那一行（Judgement.hinted 记下）；
       原话找不到（照术语改名换了字、照 date-once 删了日期）而行号提示那一行还有这一轮的名字（前后不连着轮名字符），并且〔〕与提示那一行去掉
         .claude/kb/term-renames.md 登记的新旧名（旧名按「匹配」一列认整串、词边界、正则，新名按整串）与日期（YYYY-MM-DD），再去掉空白与逗号、
         最后去掉剩下的空括号「（）」「()」（date-once 删日期把只括着日期的括号连日期一起删）之后，〔〕逐字落在那一行里（excerpt_matches_hint_line）：提示的那一行（Judgement.hinted 记下），往下照判名字、别的轮名与「」原话。
     没带〔〕原话、原话找不到而提示那一行没有这一轮的名字（或没有那一行）、原话找不到而提示那一行去掉新旧名与日期之后对不上（〔〕写的是转述、改了别的字、瞎编），
     命中几处而提示不在其中，都不收。改名表不在或读不了时只去掉日期。
     那一行里有这一轮的名字（前后不连着轮名字符）、除这一轮外没有别的轮名（别的轮的叫停原话不是为这一轮说的），
     并有一句「」括起来的用户原话（去掉空白后至少 6 个字）。
     轮名的认法：前后不连着轮名字符（字母、数字、下划线、连字符）；这一轮的名字按全名找；别的轮名 = 仓里认得的轮名全集（recognised_round_names）∪ 形如 <名>-r<数字> 的写法，两样都拒：
     全集是 research/prompts/ 下的 _<轮>-body.md、<轮>-snapshot/、<轮>-main-verification.md，加上登记表第一列出现过的轮名；
     形如 <名>-r<数字> 的写法不看仓里有没有它的材料（没有材料的 other-r2 也算别的轮名）。名字既不在全集里、也不是 -r 形的（仓里没有任何它的材料，如 m3-close-b1）不当别的轮名。
     判决文件只认 <轮>-main-verification.md 这一种文件名（ROUND_MATERIAL_FORMS）。
     出处只认 records/ 下的文件：这一轮自己的材料、规则与定义文件、仓外路径都不认。那句话是不是用户为这一轮说的，钩子认不出，靠主 agent 核。

不要求出处的只有 LEGACY_SOURCELESS_ROUNDS 点名的几行（出处这条规矩立起来之前登记的，没有出处可补；名单写死在这一份里，只缩不涨，不按日期判）：
门禁 abandoned-rounds 格与派发闸 ⑥ 用同一张名单，名单外的行一律要出处。形状与重复两样不分名单。

接口：
  read_rows(项目根) → (行列表, 读不了的原因或 None)；表不在是 ([], None)；空行与 # 开头的行不算
  recognised_round_names(项目根, 行列表) → 仓里认得的轮名集合（别的轮名那一条用它）
  judge_table(项目根, 行列表, modes=None, only_names=None) → Judgement(shape, duplicates, sources, moved, hinted)
  round_problem(judgement, 轮名) → 这一轮的登记行没收的原因（一句），收了是 None
  命令行：python3 lib_abandoned_rounds.py <项目根>    逐轮打 收 / 不收 与原因，行号挪了的收行后面写挪到第几行（退 0 都收，1 有不收的，2 用法错；照行号提示取那一行的由门禁那一格的成功行报）

弄坏开关 LIB_ABANDONED_ROUNDS_BREAK=<项>（逗号连几个；认不出的值抛 ValueError）。打开后红在哪几格写在两个用它的文件头
（runner-dispatch-guard.sh 的自检格、doc-process-records.sh 的样本）：
  columns-unchecked        不核四列不空
  date-unchecked           不核日期是 YYYY-MM-DD
  duplicate-unchecked      同一轮有两行也不拒
  source-optional          出处一概不要求：没写出处也收
  source-required-always   名单里的旧行也要求出处（LEGACY_SOURCELESS_ROUNDS 不起作用）
  legacy-by-date           免出处退回按日期判：日期早于 LEGACY_DATE_BEFORE 的行不要求出处（名单不起作用）
  other-name-by-form       别的轮名的全集退回只认登记表里出现过的轮名（仓里的 _<轮>-body.md、<轮>-snapshot/、<轮>-main-verification.md 不算；<名>-r<数字> 形照认）
  other-form-ignored       别的轮名不认 <名>-r<数字> 形，只认仓里认得的全集（改前的判法：仓里没有材料的 other-r2 不当别的轮名）
  other-round-name-ok      出处那一行里有别的轮名也收
  line-number-read         退回按行号读：取出处写的第 N 行判，〔〕里的原话不要求也不找（改前的判法：出处那份文件前面插一行，登记就失效）
  line-hint-ignored        不看行号提示：原话命中几处、或找不到，一律不收（改前的判法：出处那份记录追加一行抄了原话的补记、照术语改名换字、删日期，登记就失效）
  hint-line-unmatched      原话找不到时不比〔〕与提示那一行，提示那一行有这一轮的名字就取它（改前的判法：〔〕写转述、行号写到同一轮另一句带名字与「」原话的话，照收）
  empty-parentheses-kept   比〔〕与提示那一行时，去掉日期、空白与逗号之后剩下的空括号「（）」「()」留着不去（改前的判法：出处那一行的日期写在括号里、照 date-once 连括号删掉，登记就失效）；
                           只在门禁 doc-process-records 的样本里有格，派发闸的自检没有
  source-any-path          出处不限 records/：research/、.claude/ 起头的也收，真路径只要在仓根之内
  source-escapes           不核出处路径里的 .. 与解到真路径后是否还在 records/ 之内
  line-unread              不找〔〕里的原话、不取那一行，整份文件里找名字与「」原话（别的轮名那一条跟着不看；〔〕原话不要求）
  name-substring           这一轮的名字按子串找，不看前后连着的轮名字符
  round-name-unchecked     那一行里不必有这一轮的名字
  quote-short-ok           「」里的原话一个字也收，不要求至少 6 个字
后六项在派发闸里也可以用 RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-<项> 打开（派发闸把它们并进 modes 传进来）。
"""
import os
import re
import sys

sys.dont_write_bytecode = True

BREAK_VARIABLE = "LIB_ABANDONED_ROUNDS_BREAK"
KNOWN_BREAK_MODES = ("columns-unchecked", "date-unchecked", "duplicate-unchecked", "source-optional", "source-required-always", "legacy-by-date", "other-name-by-form",
                     "other-form-ignored", "other-round-name-ok", "line-number-read", "line-hint-ignored", "source-any-path", "source-escapes", "line-unread", "name-substring",
                     "round-name-unchecked", "quote-short-ok", "hint-line-unmatched", "empty-parentheses-kept")
TABLE_PATH = os.path.join("research", "prompts", "abandoned-rounds.tsv")   # 叫停的轮只在这一处登记
COLUMNS = 4                                                                # 轮名、日期、为什么撂下、去向（制表符分隔，四列不空）
DATE_FORM = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")
# 出处这条规矩（用户 2026-10-12 那一批把 ⑥ 改读这张表）立起来之前登记、没有出处可补的旧行：只这几个轮名不要求出处，只缩不涨（新行一律要出处，不按日期判）
LEGACY_SOURCELESS_ROUNDS = ("c196-r1", "d16-item1-r2", "c364-r3", "smalldata-ttime-r1")
LEGACY_DATE_BEFORE = "2026-10-12"                                          # 只有弄坏开关 legacy-by-date 下还在用：改前按日期免出处的界线
# 登记行里的出处：records/ 下的 .md 加行号（「第 N 行」或 :N），后面紧跟〔〕括的一段原话（照抄那一行里含这一轮名字的一段）。
# 路径前面不连着路径字符，research/../records/x.md 这类拼出来的不认；路径里的 .. 留给后面核
SOURCE_CHARACTERS = r"[^\s`'\"，。；：、（）()「」『』“”,;*<>]*?"
SOURCE_LINE = r"(?:[ \t]*第[ \t]*([0-9]+)[ \t]*行|:([0-9]+)(?![0-9]))"
SOURCE_EXCERPT = r"(?:[ \t]*〔([^〔〕\t\n]+)〕)?"                             # 第 4 组：〔〕里的原话；没带是 None（不收，弄坏开关 line-number-read、line-unread 下不要求）
SOURCE = re.compile(r"(?<![A-Za-z0-9_\-./])(records/" + SOURCE_CHARACTERS + r"\.md)" + SOURCE_LINE + SOURCE_EXCERPT)
SOURCE_ANY_PATH = re.compile(r"(?<![A-Za-z0-9_\-./])((?:records|research|\.claude)/" + SOURCE_CHARACTERS + r"\.md)" + SOURCE_LINE + SOURCE_EXCERPT)   # 只有弄坏开关用
QUOTE = re.compile(r"「([^「」\n]+)」")
QUOTE_MINIMUM_CHARACTERS = 6
ROUND_NAME_EDGE_CHARACTERS = r"A-Za-z0-9_\-"
ROUND_NAME_FORM = r"[a-z0-9][a-z0-9-]*-r[0-9]+"                            # 别的轮名的 <名>-r<数字> 形：不看仓里有没有它的材料，都当别的轮名（弄坏开关 other-form-ignored 下不认）
PROMPTS_PATH = os.path.join("research", "prompts")
ROUND_MATERIAL_FORMS = (re.compile(r"^_(.+)-body\.md$"), re.compile(r"^(.+)-main-verification\.md$"), re.compile(r"^(.+)-snapshot$"))   # research/prompts/ 下认出轮名的三种文件名


def breaks():
    modes = {mode for mode in os.environ.get(BREAK_VARIABLE, "").split(",") if mode}
    unknown = modes - set(KNOWN_BREAK_MODES)
    if unknown:
        raise ValueError(f"{BREAK_VARIABLE} 认不出 {'、'.join(sorted(unknown))}（认得的：{'、'.join(KNOWN_BREAK_MODES)}）")
    return modes


class Row:
    """登记表的一行：行号、整行、按制表符切的各列；name 是第一列去空白（空的是没有名字）。"""
    def __init__(self, number, line):
        self.number, self.line = number, line
        self.fields = line.split("\t")
        self.name = self.fields[0].strip()
        self.date = self.fields[1] if len(self.fields) > 1 else ""


class Judgement:
    """shape / sources：行号 → 没收的原因（一句）；duplicates：轮名 → 合格的行的行号列表（只列有两行及以上的）；rows：所有行；
    moved：行号 → (出处文件, 写的第 N 行, 原话现在在第 M 行)，只记收了而行号对不上的（行号只当提示，不算不收）；
    hinted：行号 → (出处文件, 写的第 N 行, 为什么照提示取)，只记收了而那一行是照行号提示取的（原话命中几处、或找不到）。"""
    def __init__(self, rows, shape, duplicates, sources, moved=None, hinted=None):
        self.rows, self.shape, self.duplicates, self.sources = rows, shape, duplicates, sources
        self.moved = moved or {}
        self.hinted = hinted or {}


def read_rows(project_root):
    path = os.path.join(project_root, TABLE_PATH)
    try:
        with open(path, encoding="utf-8") as table:
            text = table.read()
    except FileNotFoundError:
        return [], None
    except (OSError, UnicodeDecodeError) as error:
        return [], error
    rows = []
    for number, line in enumerate(text.split("\n"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        rows.append(Row(number, line))
    return rows, None


def shape_problem(row, modes):
    """四列不空、日期是 YYYY-MM-DD。合返回 None，不合返回一句原因。"""
    if (len(row.fields) != COLUMNS or any(not field.strip() for field in row.fields)) and "columns-unchecked" not in modes:
        return f"第 {row.number} 行不是四列、或有空列：{row.line}"
    if not DATE_FORM.fullmatch(row.date.strip()) and "date-unchecked" not in modes:
        return f"第 {row.number} 行日期不是 YYYY-MM-DD：{row.date}"
    return None


def duplicated_rounds(rows, shape, modes):
    """同一个轮名有两行及以上合格的行 → {轮名: [行号…]}（按轮名排序）。"""
    if "duplicate-unchecked" in modes:
        return {}
    numbers = {}
    for row in rows:
        if row.number not in shape and row.name:
            numbers.setdefault(row.name, []).append(row.number)
    return {name: found for name, found in sorted(numbers.items()) if len(found) > 1}


def recognised_round_names(project_root, rows):
    """仓里认得的轮名全集：research/prompts/ 下 _<轮>-body.md、<轮>-snapshot/、<轮>-main-verification.md 的 <轮>，加上登记表第一列的轮名。目录不在、读不了的只剩登记表的。"""
    names = {row.name for row in rows if row.name}
    try:
        entries = os.listdir(os.path.join(project_root, PROMPTS_PATH))
    except OSError:
        return names
    for entry in entries:
        for form in ROUND_MATERIAL_FORMS:
            found = form.match(entry)
            if found:
                names.add(found.group(1))
                break
    return names


def other_round_names(text, round_name, known_names, modes=frozenset()):
    """text 里除 round_name 之外的轮名，前后不连着轮名字符：known_names（recognised_round_names 交的全集）里的名字，加上形如 <名>-r<数字> 的写法（不看仓里有没有它的材料）。
    弄坏开关 other-form-ignored 下不认 <名>-r<数字> 形（known_names 之外的 -r 形不算）；other-name-by-form 下 known_names 只有登记表里出现过的轮名（judge_table 传的）。"""
    left, right = r"(?<![" + ROUND_NAME_EDGE_CHARACTERS + r"])", r"(?![" + ROUND_NAME_EDGE_CHARACTERS + r"])"
    alternatives = ([] if "other-form-ignored" in modes else [ROUND_NAME_FORM]) + [re.escape(name) for name in sorted(set(known_names), key=lambda name: (-len(name), name)) if name]
    if not alternatives:
        return []
    found = []
    for match in re.finditer(left + "(?:" + "|".join(alternatives) + ")" + right, text):
        if match.group(0) != round_name and match.group(0) not in found:
            found.append(match.group(0))
    return found


def locate_excerpt(text, excerpt):
    """excerpt 在 text 里每一处命中所在的行号（从 1 起；重叠的命中也各算一处），按出现的次序。"""
    found, start = [], 0
    while True:
        position = text.find(excerpt, start)
        if position < 0:
            return found
        found.append(text.count("\n", 0, position) + 1)
        start = position + 1


RENAMES_PATH = os.path.join(".claude", "kb", "term-renames.md")   # 术语改名对照表：表上方一行 RENAMES_TABLE_MARK，四列「旧、新、匹配、是什么」（scripts/kb/sweep-term.py 按同一张表换）
RENAMES_TABLE_MARK = "<!-- term-renames:table -->"
RENAMES_CELL_SEPARATOR = re.compile(r"(?<!\\)\|")
DATE_PATTERN = r"[0-9]{4}-[0-9]{2}-[0-9]{2}"
UNCOMPARED_CHARACTERS = re.compile(r"[\s，,]+")   # 删日期时它两边的空白与逗号跟着删（scripts/kb/history-consolidate.py 的 strip_same_date），两边都去掉再比
EMPTY_PARENTHESES = re.compile(r"（）|\(\)")       # 只括着日期的括号，删日期时连括号一起删（strip_same_date 的「（<日期>）」）：去掉日期、空白与逗号之后剩下的空括号两边都去掉再比


def rename_patterns(project_root):
    """改名表里每一行的旧名（按「匹配」一列：整串、词边界、正则）与新名（整串），合成一个去掉它们的正则；表不在、读不了、没有标记的只去掉日期。"""
    try:
        with open(os.path.join(project_root, RENAMES_PATH), encoding="utf-8") as renames:
            text = renames.read()
    except (OSError, UnicodeDecodeError):
        text = ""
    literals, patterns = set(), []
    if RENAMES_TABLE_MARK in text:
        for line in text[text.index(RENAMES_TABLE_MARK):].split("\n"):
            if not line.startswith("|"):
                continue
            cells = [cell.strip().strip("`").replace("\\|", "|") for cell in RENAMES_CELL_SEPARATOR.split(line.strip())[1:-1]]
            if len(cells) < 3 or cells[0] in ("旧", "") or set(cells[0]) <= set("-: "):
                continue
            old, new, how = cells[0], cells[1], cells[2]
            if new:
                literals.add(new)
            if how == "正则":
                try:
                    re.compile(old)
                except re.error:
                    continue
                patterns.append(old)
            elif how == "词边界":
                patterns.append((r"\b" if not old.startswith("_") else "") + re.escape(old) + (r"\b" if not old.endswith("_") else ""))
            elif old:
                literals.add(old)
    alternatives = patterns + [re.escape(literal) for literal in sorted(literals, key=lambda literal: (-len(literal), literal))] + [DATE_PATTERN]
    return re.compile("|".join(f"(?:{alternative})" for alternative in alternatives))


def compared_form(text, removed, modes=frozenset()):
    """去掉改名表的新旧名与日期（removed，rename_patterns 交的），再去掉空白与逗号、最后去掉剩下的空括号「（）」「()」之后的样子；弄坏开关 empty-parentheses-kept 下空括号留着。"""
    stripped = UNCOMPARED_CHARACTERS.sub("", removed.sub("", text))
    return stripped if "empty-parentheses-kept" in modes else EMPTY_PARENTHESES.sub("", stripped)


def excerpt_matches_hint_line(project_root, excerpt, hint_line, modes=frozenset()):
    """原话找不到时：〔〕与行号提示那一行，去掉改名表的新旧名与日期（连同剩下的空括号）之后，〔〕逐字落在那一行里（只经过术语改名、删日期的那一行照收，转述与改了别的字的不收）。"""
    removed = rename_patterns(project_root)
    wanted = compared_form(excerpt, removed, modes)
    return bool(wanted) and wanted in compared_form(hint_line, removed, modes)


def name_in(text, round_name):
    """text 里有没有这一轮的名字（前后不连着轮名字符）。"""
    return re.search(r"(?<![" + ROUND_NAME_EDGE_CHARACTERS + r"])" + re.escape(round_name) + r"(?![" + ROUND_NAME_EDGE_CHARACTERS + r"])", text) is not None


def source_problem(project_root, round_name, source_path, line_number, excerpt, known_names, modes, bound_to_records=True):
    """一处出处收不收得了这一轮 → (原因, 原话命中的行号, 照提示取的原因)：收得了原因是 None；命中的行号与写的第 N 行不同时，调用方记进 Judgement.moved；
    那一行是照行号提示取的（原话命中几处而提示落在其中一处、或原话找不到而提示那一行有这一轮的名字、去掉改名表的新旧名与日期之后〔〕落在那一行里），第三项写为什么，调用方记进 Judgement.hinted。
    默认拿〔〕里的原话 excerpt 在那份文件里找，按文件头那三种取那一行；弄坏开关 line-number-read 下退回取第 N 行，line-unread 下整份文件当那一行，
    line-hint-ignored 下不看行号提示（命中几处、找不到都不收），hint-line-unmatched 下找不到时不比〔〕与提示那一行。"""
    if ".." in source_path.split("/") and "source-escapes" not in modes:
        return f"{source_path} 的路径里有 ..（出处只认 records/ 下现存文件的原路径）", None, None
    real_path = os.path.realpath(os.path.join(project_root, source_path))
    bound = os.path.realpath(os.path.join(project_root, "records") if bound_to_records else project_root)
    if "source-escapes" not in modes and real_path != bound and not real_path.startswith(bound + os.sep):
        return f"{source_path} 解到 {real_path}，不在 {bound} 之内（符号链接或 .. 指到了外面）", None, None
    try:
        with open(real_path, encoding="utf-8") as cited_file:
            whole_text = cited_file.read()
    except (OSError, UnicodeDecodeError):
        return f"{source_path} 在仓里不存在或读不了", None, None
    cited_lines = whole_text.split("\n")
    located, hint_reason = None, None
    if "line-unread" in modes:
        cited_text = whole_text
        where = f"{source_path} 第 {line_number} 行"
    elif "line-number-read" in modes:
        if not 1 <= line_number <= len(cited_lines):
            return f"{source_path} 没有第 {line_number} 行（共 {len(cited_lines)} 行）", None, None
        cited_text = cited_lines[line_number - 1]
        where = f"{source_path} 第 {line_number} 行"
    else:
        if excerpt is None:
            return (f"{source_path} 第 {line_number} 行后面没带〔〕括的原话（写成 {source_path} 第 {line_number} 行〔<那一行里含这一轮名字 {round_name} 的一段原话，逐字照抄>〕；"
                    "判的时候拿原话在那份文件里找，行号只当提示）"), None, None
        hits = locate_excerpt(whole_text, excerpt)
        hint_usable = "line-hint-ignored" not in modes and 1 <= line_number <= len(cited_lines)
        if len(hits) > 1 and hint_usable and line_number in hits:
            hits, hint_reason = [line_number], f"原话命中 {len(hits)} 处，取行号提示落着的那一处"
        hint_matches = None   # 原话找不到、行号提示那一行有这一轮的名字时，去掉改名表的新旧名与日期之后〔〕在不在那一行里（excerpt_matches_hint_line）
        if not hits and hint_usable and name_in(cited_lines[line_number - 1], round_name):
            hint_matches = "hint-line-unmatched" in modes or excerpt_matches_hint_line(project_root, excerpt, cited_lines[line_number - 1], modes)
            if hint_matches:
                hits, hint_reason = [line_number], "原话找不到，行号提示那一行去掉改名表的新旧名与日期之后与原话逐字相同，取那一行"
        if not hits:
            hint_note = (f"行号提示的第 {line_number} 行有这一轮的名字，但去掉改名表（{RENAMES_PATH}）的新旧名与日期、空白与逗号之后，〔〕里的字不在那一行里"
                         "（删日期剩下的空括号也去掉再比；〔〕要逐字照抄那一行，不写转述；那一行只经过术语改名、删日期才对不上的照收）" if hint_matches is False
                         else f"行号提示的第 {line_number} 行里也没有这一轮的名字 {round_name}" if 1 <= line_number <= len(cited_lines)
                         else f"行号提示的第 {line_number} 行不在那份文件里（共 {len(cited_lines)} 行）")
            return f"{source_path} 里找不到〔{excerpt}〕这段原话（被删或改了字；〔〕里要与那份文件逐字相同），{hint_note}", None, None
        if len(hits) > 1:
            return (f"{source_path} 里〔{excerpt}〕这段原话命中 {len(hits)} 次（第 {'、'.join(map(str, hits[:6]))} 行），行号提示的第 {line_number} 行不在其中："
                    "把行号改成含这一轮名字的那一处，或多抄几个字直到只命中那一行"), None, None
        located = hits[0]
        cited_text = cited_lines[located - 1]
        where = f"{source_path} 第 {located} 行" + (f"（出处写的是第 {line_number} 行，原话现在在第 {located} 行）" if located != line_number else "")
    name_edges = re.compile(r"(?<![" + ROUND_NAME_EDGE_CHARACTERS + r"])" + re.escape(round_name) + r"(?![" + ROUND_NAME_EDGE_CHARACTERS + r"])")
    if "name-substring" in modes:
        name_edges = re.compile(re.escape(round_name))
    if not name_edges.search(cited_text) and "round-name-unchecked" not in modes:
        return f"{where}里没有这一轮的名字 {round_name}（那不是为这一轮说的叫停原话）", None, None
    others = [] if ("other-round-name-ok" in modes or "line-unread" in modes) else other_round_names(cited_text, round_name, known_names, modes)   # 整份文件里找时「那一行」没有意义，别的轮名不看
    if others:
        return (f"{where}里除这一轮外还有别的轮名 {'、'.join(others[:4])}"
                "（出处那一行只许写这一轮的名字；同一行里别的轮的叫停原话不是为这一轮说的，另找只写这一轮的那一行）"), None, None
    minimum = 1 if "quote-short-ok" in modes else QUOTE_MINIMUM_CHARACTERS
    if not any(len(re.sub(r"\s+", "", quote)) >= minimum for quote in QUOTE.findall(cited_text)):
        return f"{where}里没有「」括起来的用户原话（去掉空白后至少 {QUOTE_MINIMUM_CHARACTERS} 个字）", None, None
    return None, located, hint_reason


def source_required(row, modes):
    """这一行要不要出处：名单（LEGACY_SOURCELESS_ROUNDS）里的旧行不要，别的一律要；弄坏开关 legacy-by-date 下退回按日期免。"""
    if "source-optional" in modes:
        return False
    if "legacy-by-date" in modes:
        return not row.date.strip() < LEGACY_DATE_BEFORE
    if row.name in LEGACY_SOURCELESS_ROUNDS and "source-required-always" not in modes:
        return False
    return True


def row_source_problem(project_root, row, known_names, modes):
    """一行的出处收不收得了这一行的轮 → (原因, 挪动, 照提示)：整行里有一处出处收得了就收（原因是 None；挪动是 (出处文件, 写的第 N 行, 原话现在在第 M 行)，行号对得上是 None；
    照提示是 (出处文件, 写的第 N 行, 为什么照提示取)，那一行不是照行号提示取的是 None），否则原因是把各处没收的原因接起来的一句。"""
    any_path = "source-any-path" in modes
    sources = list((SOURCE_ANY_PATH if any_path else SOURCE).finditer(row.line))
    if not sources:
        return (f"第 {row.number} 行没有出处：要写 records/<文件>.md 第 N 行〔<那一行里含这一轮名字的一段原话>〕"
                "（research/、.claude/ 下的文件、这一轮自己的材料、仓外路径都不认）"), None, None
    reasons = []
    for source in sources:
        written = int(source.group(2) or source.group(3))
        problem, located, hint_reason = source_problem(project_root, row.name, source.group(1), written, source.group(4), known_names, modes, bound_to_records=not any_path)
        if problem is None:
            return (None, ((source.group(1), written, located) if located is not None and located != written else None),
                    ((source.group(1), written, hint_reason) if hint_reason is not None else None))
        reasons.append(f"第 {row.number} 行的出处 {problem}")
    return "；".join(dict.fromkeys(reasons)), None, None


def judge_table(project_root, rows, modes=None, only_names=None):
    """整张表判一遍。only_names 给了就只判这些轮名的行（出处要读文件，派发闸只判开着的轮）；别的行的轮名仍算仓里认得的轮名。"""
    modes = breaks() if modes is None else set(modes)
    known_names = [row.name for row in rows if row.name] if "other-name-by-form" in modes else sorted(recognised_round_names(project_root, rows))
    shape, sources, moved, hinted, judged = {}, {}, {}, {}, []
    for row in rows:
        if only_names is not None and row.name not in only_names:
            continue
        judged.append(row)
        problem = shape_problem(row, modes)
        if problem is not None:
            shape[row.number] = problem
    duplicates = duplicated_rounds(judged, shape, modes)
    for row in judged:
        if row.number in shape or not source_required(row, modes):
            continue
        problem, movement, hint = row_source_problem(project_root, row, known_names, modes)
        if problem is not None:
            sources[row.number] = problem
            continue
        if movement is not None:
            moved[row.number] = movement
        if hint is not None:
            hinted[row.number] = hint
    return Judgement(rows, shape, duplicates, sources, moved, hinted)


def round_problem(judgement, round_name):
    """这一轮的登记行没收的原因（形状、重复、出处依次接起来的一句）；收了返回 None。这一轮没有登记行也返回 None（没登记是调用方判的事）。"""
    own = [row for row in judgement.rows if row.name == round_name]
    reasons = [judgement.shape[row.number] for row in own if row.number in judgement.shape]
    if round_name in judgement.duplicates:
        reasons.append(f"登记了不止一行（第 {'、'.join(map(str, judgement.duplicates[round_name]))} 行）：合成一行，理由与去向写在同一行里")
    reasons += [judgement.sources[row.number] for row in own if row.number in judgement.sources]
    return "；".join(reasons) if reasons else None


def main(argv):
    arguments = list(argv)
    if len(arguments) != 1:
        print("  ✗ 用法：lib_abandoned_rounds.py <项目根>", file=sys.stderr)
        print("     → 怎么办：项目根是有 research/prompts/abandoned-rounds.tsv 的目录；不要求出处的只有 LEGACY_SOURCELESS_ROUNDS 点名的旧行，门禁格与派发闸 ⑥ 同一套", file=sys.stderr)
        return 2
    try:
        judgement_modes = breaks()
    except ValueError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE}，或只写认得的项", file=sys.stderr)
        return 2
    rows, error = read_rows(arguments[0])
    if error is not None:
        print(f"  ✗ 读不了 {TABLE_PATH}：{error}", file=sys.stderr)
        print(f"     → 怎么办：看 {TABLE_PATH} 在不在、有没有读权限；表的写法在这份文件头", file=sys.stderr)
        return 2
    judgement = judge_table(arguments[0], rows, judgement_modes)
    failed = False
    for name in dict.fromkeys(row.name for row in rows if row.name):
        problem = round_problem(judgement, name)
        failed = failed or problem is not None
        moves = [judgement.moved[row.number] for row in rows if row.name == name and row.number in judgement.moved]
        note = "".join(f"（出处 {path} 写的第 {written} 行，原话现在在第 {located} 行）" for path, written, located in moves)
        print(f"{name}\t{'收' + note if problem is None else '不收：' + problem}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
