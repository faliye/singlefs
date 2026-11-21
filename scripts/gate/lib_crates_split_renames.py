"""测试改名表的读法，与它对变异表第 5、6 段和登记表的判法（crates-split-pure-move.py 的 remap-mutations、registries 两个子命令用）。

拆分把 #[cfg(test)] mod tests 里的用例挪进子模块，测试全名（库里的模块路径，例 transaction::tests::x）跟着变。
同一个全名可以在几个测试目标里各有一条（集成测试各目标是各自的二进制，库与二进制里的单测也是），改名只属其中一个目标时要分目标写。
改名表每行「旧全名<TAB>新全名」，空行与 # 起头的行跳过；--test-renames 给几次取并集，每个值两种写法：
  <改名表>                  不带目标：旧全名只落在一个测试目标里时照名字换，落在几个目标里时列出不改（见下）；
  <包>:<目标>=<改名表>      这份表只管那个包那个目标里的测试。目标三种：lib（库的单测）、test:<名>（集成测试，<名> 是 cargo test --test <名> 里那个名字）、
                            bin:<名>（二进制里的单测）。等号前面有 / 的当不带目标的路径；写成这种形状而包或目标不对的拒（退 2）。
同一个写法（同一个包与目标，或都不带目标）里同一个旧全名对到两个不同的新全名拒；不同目标里同一个旧全名可以对到不同的新全名。

变异表一行的目标从第 5 段 -- 之前认：-p <包>（或 --package <包>）定包，--lib、--test <名>、--bin <名> 定目标，包与目标各写且只写一个（重复写同一个不算多）；
没写、写了不止一个、值是空的或像选项的都认不出。登记表的行同样认，取整行里 -- 之前的那一段。
一行点名的测试全名：第 6 段，与第 5 段带 --exact 的筛选词；别的筛选词是子串，不算点名。

变异表第 5 段 -- 之后是 cargo test 交给测试程序的参数：不是选项的词是筛选词，不带 --exact 按子串筛，带 --exact 按整名相等筛；没有筛选词就是整个目标的测试都跑。
第 6 段是必须红的测试名（整名）。
  这一行用哪些改名：
    按目标写的表：包与目标都与这一行的目标相同的才用；这一行的目标认不出、而点名了表里的旧全名的，列出不改。
    不带目标的表：旧全名在变异表所有点名它的行里只落在一个目标（认不出目标的行各算一个自己的目标）的，用；落在几个目标的，点名它的每一行列出不改。
  换名：第 6 段、带 --exact 的筛选词，整名等于某个用得上的旧全名的换成它的新全名，只换整名相等的（子串相同不换）；不带 --exact 的筛选词一概不改。
  列出（只报不改）：
    不带 --exact 的筛选词，这一行用得上的（含落在几个目标而没换的不带目标的）旧全名拆前筛得到（筛选词是旧全名的子串）、拆后对应的新全名筛不到的；
    第 6 段换名之后，第 5 段的筛选在拆后筛不到它的；
    第 5 段 -- 之后有 --exact 以外的选项的（--skip 这类会改筛选的语义，第 5 段不判、不改；这一行没有用得上的旧全名时不列）。
  只改第 5、6 段里的测试名：词与词之间的空白、别的词、别的段一个字节不动。
登记表扫描：旧全名整串出现、前面不紧挨标识符字符或 ::、后面不紧挨标识符字符的行算点名；不带目标的表的旧全名都列；按目标写的表的旧全名，这一行的目标与表一致的列，
目标认不出的也列（注明目标认不出），目标认得出而不一致的不列。
"""
import re

import lib_crates_split_common as common

broken = common.broken
UsageProblem = common.UsageProblem

EXACT_OPTION = "--exact"
ARGUMENT_SEPARATOR = " -- "
RENAME_FIELD_COUNT = 2
NAME_CHARACTERS = r"A-Za-z0-9_"
NAME_PATTERN = re.compile(r"[A-Za-z0-9_-]+")
LIBRARY_TARGET = "lib"
NAMED_TARGET_KINDS = ("test", "bin")
PACKAGE_FLAGS = ("-p", "--package")
TARGET_FLAGS = {"--lib": LIBRARY_TARGET, "--test": "test", "--bin": "bin"}
FORMAT_HOWTO = "改名表每行写「旧全名<TAB>新全名」两段，同一个旧全名只对到一个新全名；--test-renames 给几份取并集"
SPEC_HOWTO = "--test-renames 写成 <改名表> 或 <包>:<目标>=<改名表>；目标写 lib、test:<名>（cargo test --test <名> 里那个名字）或 bin:<名>"
UNRECOGNIZED_TARGET = "目标认不出"


class RenameTable:
    """--test-renames 读出来的表：loose 是不带目标的 {旧全名: 新全名}，scoped 是 {(包, 目标): {旧全名: 新全名}}。"""

    def __init__(self):
        self.loose = {}
        self.scoped = {}

    def entries_for(self, scope):
        return self.loose if scope is None else self.scoped.setdefault(scope, {})

    def __len__(self):
        return len(self.loose) + sum(len(entries) for entries in self.scoped.values())

    def olds(self):
        names = set(self.loose)
        for entries in self.scoped.values():
            names.update(entries)
        return sorted(names)


def target_label(target):
    return f"{target[0]}:{target[1]}"


def valid_target(target):
    if target == LIBRARY_TARGET:
        return True
    kind, colon, name = target.partition(":")
    return bool(colon) and kind in NAMED_TARGET_KINDS and NAME_PATTERN.fullmatch(name) is not None


def split_spec(spec):
    """--test-renames 的一个值 → (None 或 (包, 目标), 改名表路径)。等号前面没有冒号或有 / 的当不带目标的路径。"""
    head, equals, path = spec.partition("=")
    if not equals or ":" not in head or "/" in head:
        return None, spec
    package, _colon, target = head.partition(":")
    if NAME_PATTERN.fullmatch(package) and valid_target(target) and path:
        return (package, target), path
    if "malformed-target-accepted" in broken:
        return None, spec
    raise UsageProblem(f"--test-renames {spec} 的「<包>:<目标>」不对：包写包名，目标写 lib、test:<名> 或 bin:<名>，等号后面写改名表的路径", SPEC_HOWTO)


def load_renames(specs):
    """--test-renames 给的几份表取并集 → RenameTable。一行不是两段、有空段，同一个写法里同一个旧全名对到两个新全名，或目标写得不对，抛 UsageProblem。"""
    table = RenameTable()
    claimed = {}
    for spec in (specs[-1:] if "renames-union-last-only" in broken else specs):
        scope, path = split_spec(spec)
        entries = table.entries_for(scope)
        for number, line in enumerate(common.read_text(path).split("\n"), start=1):
            if not line or line.startswith("#"):
                continue
            fields = line.split("\t")
            if len(fields) != RENAME_FIELD_COUNT or not all(fields):
                if "malformed-rename-line-skipped" in broken:
                    continue
                raise UsageProblem(f"{path}:{number} 不是「旧全名<TAB>新全名」两段：{line}", FORMAT_HOWTO)
            old, new = fields
            key = old if "renames-conflict-across-targets" in broken else (scope, old)
            if claimed.get(key, new) != new and "conflicting-renames-accepted" not in broken:
                raise UsageProblem(f"{path}:{number} 旧全名 {old} 已对到 {claimed[key]}，这一行又对到 {new}", FORMAT_HOWTO)
            claimed[key] = new
            entries[old] = new
    return table


def row_target(arguments):
    """cargo 参数（变异表第 5 段，或登记表的一行）-- 之前点到的 (包, 目标)；目标写成 lib、test:<名>、bin:<名>；认不出 → None。"""
    words = arguments.partition(ARGUMENT_SEPARATOR)[0].split()
    packages, targets = [], []
    for index, word in enumerate(words):
        flag, equals, inline = word.partition("=")
        if flag in TARGET_FLAGS and flag != "--lib" and "test-bin-flags-unparsed" in broken:
            continue
        if flag not in PACKAGE_FLAGS and flag not in TARGET_FLAGS:
            continue
        if flag == "--lib":
            targets.append(LIBRARY_TARGET)
            continue
        value = inline if equals else (words[index + 1] if index + 1 < len(words) else "")
        if not value or value.startswith("-"):
            return None
        if flag in PACKAGE_FLAGS:
            packages.append(value)
        else:
            targets.append(f"{TARGET_FLAGS[flag]}:{value}")
    if "several-targets-first-taken" in broken:
        packages, targets = packages[:1], targets[:1]
    if len(set(packages)) != 1 or len(set(targets)) != 1:
        return None
    return packages[0], targets[0]


def named_tests(arguments, test_name):
    """变异表一行点名的测试全名：第 6 段，与第 5 段带 --exact 的筛选词。"""
    names = {test_name}
    _head, separator, tail = arguments.partition(ARGUMENT_SEPARATOR)
    words = tail.split() if separator else []
    if EXACT_OPTION in words:
        names.update(word for word in words if not word.startswith("-"))
    return names


def scope_applies(scope, target):
    """按目标写的表（scope 是 (包, 目标)）管不管这一行的目标（认不出是 None）→ "yes"、"no"，目标认不出而该列出的是 "unknown"。"""
    if target is None:
        if "unrecognized-target-renamed" in broken:
            return "yes"
        return "no" if "unrecognized-target-not-listed" in broken else "unknown"
    same_package = scope[0] == target[0] or "package-ignored" in broken
    same_target = scope[1] == target[1] or "target-ignored" in broken
    return "yes" if same_package and same_target else "no"


def landing_places(rows, table):
    """变异表各行（变异名、第 5 段、第 6 段）→ {不带目标的表里的旧全名: {点名它的各行落在哪些测试目标}}；认不出目标的行各算一个自己的目标。"""
    places = {}
    for name, arguments, test_name in rows:
        target = row_target(arguments)
        place = target_label(target) if target else f"{name}（{UNRECOGNIZED_TARGET}）"
        for tested in named_tests(arguments, test_name):
            if tested in table.loose:
                places.setdefault(tested, set()).add(place)
    return places


def renames_for_row(table, arguments, test_name, places):
    """变异表一行（第 5、6 段）该用哪些改名 → (换名用的 {旧全名: 新全名}, 算「筛选词拆后筛不到」用的 {旧全名: 新全名}, 列出的说明表)。
    places 是 landing_places 的结果。"""
    target = row_target(arguments)
    named = named_tests(arguments, test_name)
    applied, undecided, notes = {}, set(), []
    for scope, entries in table.scoped.items():
        verdict = scope_applies(scope, target)
        if verdict == "yes":
            applied.update(entries)
        elif verdict == "unknown":
            undecided.update(old for old in entries if old in named)
    if undecided:
        notes.append(f"{UNRECOGNIZED_TARGET}（第 5 段没有唯一的 -p 加 --lib、--test <名> 或 --bin <名>），而按目标写的改名点到了它的 {'、'.join(sorted(undecided))}，没换")
    listing = dict(applied)
    for old, new in table.loose.items():
        if old in applied:
            continue
        landed = places.get(old, set())
        if len(landed) > 1 and "loose-ambiguity-ignored" not in broken:
            listing[old] = new
            if old in named:
                notes.append(f"旧全名 {old} 在变异表里落在 {len(landed)} 个测试目标（{'、'.join(sorted(landed))}），不带目标的改名表分不清改哪个，没换")
            continue
        applied[old] = new
        listing[old] = new
    return applied, listing, notes


def renamed(name, renames):
    """整名等于某个旧全名的换成新全名，别的原样。"""
    if "renames-by-substring" in broken:
        for old, new in renames.items():
            name = name.replace(old, new)
        return name
    return renames.get(name, name)


def selects(words, exact, name):
    """筛选词 words 筛不筛得到测试 name；没有筛选词就是全选。"""
    if not words:
        return True
    return any(word == name for word in words) if exact else any(word in name for word in words)


def rewrite_test_names(arguments, test_name, renames, listing=None):
    """变异表一行的第 5 段（arguments）与第 6 段（test_name）照这一行用得上的改名（renames：{旧全名: 新全名}）换名 → (新的第 5 段, 新的第 6 段, 列出的说明表)。
    listing 是算「筛选词拆后筛不到」用的改名，缺省同 renames。"""
    listing = renames if listing is None else listing
    new_name = test_name if "sixth-column-not-renamed" in broken else renamed(test_name, renames)
    head, separator, tail = arguments.partition(ARGUMENT_SEPARATOR)
    if not separator:
        return arguments, new_name, []
    pieces = re.split(r"(\s+)", tail)
    words = [(index, piece) for index, piece in enumerate(pieces) if index % 2 == 0 and piece]
    options = [word for _index, word in words if word.startswith("-")]
    unknown = [option for option in options if option != EXACT_OPTION]
    if unknown and (renames or listing) and "unknown-option-ignored" not in broken:
        return arguments, new_name, [f"第 5 段 -- 之后有不认得的选项 {unknown[0]}，筛选词不能判"]
    exact = EXACT_OPTION in options
    filters = [(index, word) for index, word in words if not word.startswith("-")]
    notes = []
    if not exact and "lost-filter-not-listed" not in broken:
        for _index, word in filters:
            lost = [new for old, new in listing.items() if word in old and (word not in new or "kept-filter-listed" in broken)]
            if lost:
                notes.append(f"筛选词 {word} 拆前筛得到、拆后筛不到 {len(lost)} 个新全名：{'、'.join(lost)}")
    if (exact and "exact-filter-not-renamed" not in broken) or (not exact and "substring-filter-renamed" in broken):
        for index, word in filters:
            pieces[index] = renamed(word, renames)
    if new_name != test_name and "renamed-test-selection-unchecked" not in broken:
        chosen = [pieces[index] for index, _word in filters]
        if not selects(chosen, exact, new_name):
            notes.append(f"第 6 段换名后是 {new_name}，第 5 段的筛选词 {' '.join(chosen)} 筛不到它")
    rebuilt = " ".join(piece for piece in pieces if piece.strip()) if "columns-reserialized" in broken else "".join(pieces)
    return head + separator + rebuilt, new_name, notes


def old_name_pattern(table):
    """登记表一行里点名某个旧全名的写法：整串出现，前面不紧挨标识符字符或 ::，后面不紧挨标识符字符。"""
    alternatives = "|".join(re.escape(old) for old in table.olds())
    if "registry-test-name-boundary-ignored" in broken:
        return re.compile(alternatives)
    return re.compile(rf"(?<![{NAME_CHARACTERS}])(?<!::)(?:{alternatives})(?![{NAME_CHARACTERS}])")


def registry_renames_of(old, table, target):
    """登记表一行（目标 target，认不出是 None）里点名的旧全名 old 该列出的改名写法表（「旧全名 改名为 新全名」，按目标写的带目标）。"""
    pieces = []
    if old in table.loose:
        pieces.append(f"{old} 改名为 {table.loose[old]}")
    for scope, entries in table.scoped.items():
        if old not in entries:
            continue
        verdict = scope_applies(scope, target)
        if verdict == "yes":
            pieces.append(f"{old} 改名为 {entries[old]}（{target_label(scope)}）")
        elif verdict == "unknown":
            pieces.append(f"{old} 改名为 {entries[old]}（{target_label(scope)}；这一行的{UNRECOGNIZED_TARGET}，按名字列出）")
    return pieces


def registry_name_hit(pattern, table, relative, number, line):
    """登记表一行点名了改名表的旧全名 → 要列出的一条（文件:行、整行、旧全名改成什么）；没点名，或点名的都是别的目标里的 → None。"""
    if "registry-test-names-not-scanned" in broken:
        return None
    olds = sorted(set(pattern.findall(line)))
    if not olds:
        return None
    target = row_target(line)
    pieces = [piece for old in olds for piece in registry_renames_of(old, table, target)]
    if not pieces:
        return None
    return f"{relative}:{number}\t{line}\t← " + "；".join(pieces)
