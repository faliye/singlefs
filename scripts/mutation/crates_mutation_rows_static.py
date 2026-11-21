"""crates-mutation-rows.py 拆出的一份。
变异表的静态核：拷贝范围的算法（path 依赖、--manifest-path、include 宏）、锚点失效、替换文里没绑定的标识符、转义字面量、release 补行与验证包补行，连同这几样在样本仓上的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import fnmatch
import importlib.util
import multiprocessing
import os
import re
import traceback
from crates_mutation_rows_base import (IGNORED_WHEN_COPYING_INTO_EACH_SHARD, INCLUDE_MACRO_BEFORE_LITERAL, LITERAL_OR_COMMENT_START,
    PATH_ATTRIBUTE_AFTER_LITERAL, PATH_ATTRIBUTE_BEFORE_LITERAL, REGISTERED_COPY_RANGE_REASON, RELEASE_ONLY_TESTS_PATH, RELEASE_REGISTRY_ROOT,
    REPOSITORY_COPY_PATH, ROW_PROBLEM_KINDS, STAGE_REPOSITORY, TABLE, TEST_NAME_FORM, _RELEASE_ONLY_TESTS_MODULE, _REPOSITORY_COPY_MODULE, admission_module, break_is,
    inserted_before_double_dash, merge_step_close, merge_step_open, merge_step_row_done, mutation_units, read_table, split_at_double_dash)
from crates_mutation_rows_support import (build_selftest_repository, selftest_write)


def repository_copy_module():
    """scripts/hosts/lib_repository_copy.py（拷贝范围的 copy_closure、path_dependency_entries 与副本里的环境），第一次用到时才读进来。"""
    if not _REPOSITORY_COPY_MODULE:
        specification = importlib.util.spec_from_file_location("lib_repository_copy_for_crates_mutation_rows", REPOSITORY_COPY_PATH)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _REPOSITORY_COPY_MODULE.append(module)
    return _REPOSITORY_COPY_MODULE[0]


def release_only_tests_module():
    """scripts/mutation/lib_release_only_tests.py（重测试目标只许 release 的判法），第一次用到时才读进来。"""
    if not _RELEASE_ONLY_TESTS_MODULE:
        specification = importlib.util.spec_from_file_location("lib_release_only_tests_for_crates_mutation_rows", RELEASE_ONLY_TESTS_PATH)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _RELEASE_ONLY_TESTS_MODULE.append(module)
    return _RELEASE_ONLY_TESTS_MODULE[0]


def judged_registry_root(root, environment):
    """static-check 读登记表（拷贝范围、重测试目标）的那一份仓根：被判的那棵树 root，不回落到本模块所在那一份仓（root 里没有那张表就是读不了）。
    弄坏开关 static-check-registry-from-script-repository：读本模块所在那一份仓的（STAGE_REPOSITORY），补丁新登记的行在被判的树里看不见。"""
    return STAGE_REPOSITORY if break_is(environment, "static-check-registry-from-script-repository") else root


def release_registered_targets(registry_root=None):
    """登记表里 crash-case: 行点到的测试目标（lib_release_only_tests.py 的 crash_case_targets）；registry_root 不给读 RELEASE_REGISTRY_ROOT 那一份（run_stage 一路），
    static-check 给被判的那棵树；读不了抛它的 RegistryUnreadable。"""
    return release_only_tests_module().crash_case_targets(RELEASE_REGISTRY_ROOT[0] if registry_root is None else registry_root)


def release_forced_row(row, registered, environment):
    """(行, 依据或 None)：命中重测试目标而 cargo 参数没带 release 的行换成补了 --release 的（文件头「重测试目标只许 release」），依据是补的理由；
    别的原样、依据交 None。Row.fields 不动（用时表与索引照它取名字）。弄坏开关 release-unforced：一律原样。"""
    if break_is(environment, "release-unforced"):
        return row, None
    arguments, requirement = release_only_tests_module().release_forced(row.arguments, registered=registered)
    if arguments == row.arguments:
        return row, None
    return row._replace(arguments=arguments), requirement.basis


def default_features_off_row(row, environment):
    """验证包只点名集成测试目标的行换成补了 --no-default-features 的（文件头「验证包的测试目标不编装置二进制」）：cargo 给集成测试编目标时把本包的二进制目标全编一遍，
    验证包的装置二进制（src/bin/ 下）挂着缺省开着的特性，关掉缺省特性就不编。命中 = cargo 参数（`--` 之前）-p 只点名验证包 singlefs-verification、点名了 --test <名>、
    没有别的选目标的选项（--lib、--bin、--bins、--tests、--all-targets 一类，选项表同 lib_release_only_tests.py 的 TARGET_OPTIONS）、没自己写 --no-default-features；
    别的一个字不改。补在第一个 `--` 之前（libtest 的参数不动）。Row.fields 不动（用时表与索引照它取名字）。弄坏开关 no-default-features-unadded：一律原样。"""
    if break_is(environment, "no-default-features-unadded"):
        return row
    library = release_only_tests_module()
    before, _after = library.argument_words(row.arguments)
    options = {word.split("=", 1)[0] for word in before}
    parsed = mutation_units.parse_cargo_arguments(before)
    other_targets = options & (library.TARGET_OPTIONS - {"--test"})
    if parsed.packages != ("singlefs-verification",) or "--test" not in options or other_targets or parsed.no_default_features:
        return row
    return row._replace(arguments=inserted_before_double_dash(row.arguments, ["--no-default-features"]))


def release_unwritten_rows(rows, environment, registry_root):
    """static-check 的 RELEASE（文件头 static-check 那一段）：命中重测试目标而第五段没写 release 的行 [(行号, 变异名, 依据)]，按表的次序；
    重测试目标读 registry_root（被判那棵树，judged_registry_root）里的登记表；登记表读不了抛 lib_release_only_tests.py 的 RegistryUnreadable。
    弄坏开关 release-unchecked：一行都不报。"""
    if break_is(environment, "release-unchecked"):
        return []
    module = release_only_tests_module()
    registered = release_registered_targets(registry_root)
    unwritten = []
    for row in rows:
        requirement = module.release_requirement(row.arguments, registered=registered)
        if requirement.release_missing:
            unwritten.append((row.line_number, row.name, requirement.basis))
    return unwritten


def copied_with_path_dependencies(root, copied, environment):
    """登记的拷贝范围 copied 再加被判那棵树 root 的共用拷贝范围（文件头「拷贝范围」）：lib_repository_copy.py 的 copy_closure（带测试代码扫，
    登记的那几处经 declared 声明，它们的 path 依赖照跟）。登记的在前、照原次序，闭包算出的接在后面（已落在登记那几处里的不重复交）。
    闭包算不成抛 ValueError（ClosureRefused 是它的子类）或 OSError。
    弄坏开关 path-dependencies-not-copied：不调 copy_closure，只交登记的那几处，副本里缺 prune-server/，cargo 解析工作区就失败。"""
    if break_is(environment, "path-dependencies-not-copied"):
        return list(copied)
    library = repository_copy_module()
    closure = library.copy_closure(os.path.realpath(root), [(item, REGISTERED_COPY_RANGE_REASON) for item in copied], test_inputs=True)
    return list(copied) + [item for item in closure.entries if not library.is_covered(item, copied)]


def is_copied_into_each_shard(path, copied):
    """一个仓根起的路径在不在每片的源码副本里：落在 copied 的某一项之内，而且那一项底下的每一段路径都不叫
    IGNORED_WHEN_COPYING_INTO_EACH_SHARD 里的名字。绝对路径、用 `..` 跳出去的、仓根本身都不在。"""
    normalized = os.path.normpath(path)
    if os.path.isabs(normalized):
        return False
    for item in copied:
        if normalized == item:
            return True
        if normalized.startswith(item + os.sep):
            below_item = normalized[len(item) + 1:].split(os.sep)
            return not any(fnmatch.fnmatch(part, pattern) for part in below_item for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD)
    return False


def rows_outside_copy(rows, copied):
    return [row for row in rows if not is_copied_into_each_shard(row.path, copied)]


def manifest_directory_of(root, arguments, environment):
    """一行 cargo 参数（`--` 之前）里 --manifest-path 指的那份 Cargo.toml 所在的目录，相对 <根>（文件头「--manifest-path」）：(目录, None)；
    没写 --manifest-path 交 ("", None)，是 <根> 那一份工作区。写坏了交 (None, 为什么)：写了不止一次、后面缺路径、是绝对路径
    （每片在自己的源码副本根起 cargo，绝对路径指不进副本）、文件名不是 Cargo.toml、落在 <根> 之外（经符号链接出去的同样算）、那份文件不在。
    相对路径照 cargo 在副本根起时的解法，相对 <根> 算。
    弄坏开关 manifest-path-ignored：一律交 ("", None)，拷贝范围不补那个包目录，单元键照 <根> 那一份工作区算。"""
    if break_is(environment, "manifest-path-ignored"):
        return "", None
    before = split_at_double_dash(arguments)[0]
    named = []
    for index, word in enumerate(before):
        if word == "--manifest-path":
            named.append(before[index + 1] if index + 1 < len(before) else "")
        elif word.startswith("--manifest-path="):
            named.append(word.partition("=")[2])
    if not named:
        return "", None
    if len(named) > 1:
        return None, f"--manifest-path 写了 {len(named)} 次，cargo 只认一份"
    path = named[0]
    if not path:
        return None, "--manifest-path 后面缺路径"
    if os.path.isabs(path):
        return None, f"--manifest-path {path} 是绝对路径（每片在自己的源码副本根起 cargo，要写相对仓根的路径）"
    normalized = os.path.normpath(path)
    if os.path.basename(normalized) != "Cargo.toml":
        return None, f"--manifest-path {path} 指的不是一份 Cargo.toml"
    tree = os.path.realpath(root)
    if os.path.commonpath((tree, os.path.realpath(os.path.join(root, normalized)))) != tree or normalized.startswith(os.pardir + os.sep):
        return None, f"--manifest-path {path} 不在仓内（落在被判那棵树 {root} 之外）"
    if not os.path.isfile(os.path.join(root, normalized)):
        return None, f"--manifest-path {path} 指的文件不在"
    directory = os.path.dirname(normalized)
    return ("" if directory in ("", os.curdir) else directory), None


def copied_with_manifest_directories(root, copied, rows, environment):
    """拷贝范围（已跟过 path 依赖的 copied）再补变异行 --manifest-path 指的包目录（文件头「拷贝范围」「--manifest-path」）：
    那个包目录的 Cargo.toml 已在范围里的不动（仓根 prune-server/ 这一类，验证包经 path 依赖已拷进来），不在的把那个目录补进来，
    连同它底下各份 Cargo.toml 经 path 依赖再指到的（跟法同 copied_with_path_dependencies）。
    --manifest-path 写坏了（manifest_directory_of 交的为什么：不在仓内、不是 Cargo.toml、文件不在……）抛 ValueError，带变异表的行号与变异名；
    补进来的包的 path 依赖指到树之外的同样抛 ValueError。弄坏开关 manifest-path-ignored 下 manifest_directory_of 一律交 <根>，一个都不补。"""
    result, added = list(copied), False
    for row in rows:
        directory, why = manifest_directory_of(root, row.arguments, environment)
        if why is not None:
            raise ValueError(f"{TABLE}:{row.line_number} {row.name}：{why}")
        if directory and not is_copied_into_each_shard(os.path.join(directory, "Cargo.toml"), result):
            result.append(directory)
            added = True
    if added and not break_is(environment, "path-dependencies-not-copied"):
        extra = repository_copy_module().path_dependency_entries(os.path.realpath(root), result)
        result += [item for item in sorted(extra) if item not in result]
    return result


def unit_keys_under_manifest(tree_root, directory):
    """--manifest-path 指的包（directory，相对 tree_root、不空）那一份单元键（文件头「--manifest-path」）：lib_mutation_units.py 的 UnitKeys 以包目录为根算，
    那个包自己是工作区根，被改坏的单元、点名的测试与闭包都照它的源码算；进出的路径换成相对 tree_root：
    问进来的路径去掉 <directory>/，交出去的单元（被改坏的、闭包成员、执行过的）的文件接上 <directory>/，库里的组成清单、牵连面与作废原因因此都写仓根起的路径。
    被改的文件不在那个包底下的交拒算那一句（单元键照那个包的源码算，包外的文件不在它的单元里）。KeyContext 与记覆盖的工作进程（record_coverage）各用一份。
    类写在函数里，不做模块级的类。"""
    unit_keys = mutation_units.UnitKeys(os.path.join(tree_root, directory))
    prefix = directory + os.sep

    def inside(path):
        return path[len(prefix):] if path.startswith(prefix) else None

    def outward(reference):
        # 数据文件与数据目录的单元拿路径当限定名（lib_mutation_units.py 的 mutated_units、lib_function_units.py 的数据单元）：限定名跟着文件一起换
        name = prefix + reference.name if reference.name == reference.file else reference.name
        return reference._replace(file=prefix + reference.file, name=name)

    def inward(reference):
        relative = inside(reference.file)
        return reference._replace(file=relative, name=relative if reference.name == reference.file else reference.name)

    class ManifestUnitKeys:
        root = tree_root   # row_view 照 <根> 起的路径打开被改的文件
        wrapped_unit_keys = unit_keys   # 包着的 UnitKeys（built_workspaces_of、released_unit_keys 要它的工作区）

        @property
        def workspace(self):
            return unit_keys.workspace

        def touches_stripped_text(self, path, text, start, end):
            relative = inside(path)
            return relative is not None and unit_keys.touches_stripped_text(relative, text, start, end)

        def source_bytes(self, path):
            # 包底下的取包着的那一份记着的（每份只读一遍）；包外的照 <根> 起的路径现读（mutated_units 对它拒算，row_view 照读）
            relative = inside(path)
            if relative is not None:
                return unit_keys.source_bytes(relative)
            with open(os.path.join(tree_root, path), "rb") as handle:
                return handle.read()

        def mutated_units(self, path, original):
            relative = inside(path)
            if relative is None:
                return None, f"被改坏的文件 {path} 不在 --manifest-path 指的包 {directory}{os.sep} 底下（这一行的单元键照那个包的源码算）"
            references, refusal = unit_keys.mutated_units(relative, original)
            return (None, refusal) if refusal is not None else ([outward(reference) for reference in references], None)

        def resolve_test(self, arguments, expected, view=None):
            return unit_keys.resolve_test(arguments, expected, view=view)

        def features_of(self, arguments, target_id):
            return unit_keys.features_of(arguments, target_id)

        def precompute_closures(self, requests, workers, labels=None):
            return unit_keys.precompute_closures(requests, workers, labels)

        def closure(self, unit, target_id, arguments=None):
            info = unit_keys.closure(unit, target_id, arguments)
            return info._replace(members=[outward(member) for member in info.members])

        def current_fingerprints(self, references):
            current, within = {}, []
            for reference in references:
                identity = (reference.kind, reference.file, reference.name)
                current[identity] = set()   # 不在那个包底下的文件照「文件不在」交空集合（lib_mutation_units.py 的 current_fingerprints）
                if inside(reference.file) is not None:
                    within.append((identity, inward(reference)))
            found = unit_keys.current_fingerprints([inner for _identity, inner in within])
            for identity, inner in within:
                current[identity] = found.get((inner.kind, inner.file, inner.name), set())
            return current

        def executed_units(self, export, work_root):
            executed, unplaced = unit_keys.executed_units(export, os.path.join(work_root, directory))
            return [outward(reference) for reference in executed], unplaced

    return ManifestUnitKeys()


def unit_keys_for_directory(tree_root, directory):
    """包目录（manifest_directory_of 交的，相对 tree_root）那一份单元键：空串是 tree_root 那一份工作区（lib_mutation_units.py 的 UnitKeys），
    别的是 unit_keys_under_manifest。"""
    return mutation_units.UnitKeys(tree_root) if directory == "" else unit_keys_under_manifest(tree_root, directory)


def stray_escapes(segment):
    """一段里 \\n 以外的反斜杠转义：本模块与 mutate.sh 只把 \\n 还原成换行，别的反斜杠按字面写进源码，那条变异编不过、等于没跑（C427）。"""
    return [match.group(0) for match in re.finditer(r"\\(.)", segment) if match.group(1) != "n"]


def source_text_outcome(root, path):
    """(原文, None) 或 (None, 读不成的异常)：照 utf-8 严格解码读 root 下的 path（anchor_problem 与 anchor_problems_by_line 共用这一种读法）。"""
    try:
        with open(os.path.join(root, path), encoding="utf-8") as handle:
            return handle.read(), None
    except (OSError, UnicodeDecodeError) as error:
        return None, error


def anchor_problem_of(row, outcome):
    """照 source_text_outcome 读到的 outcome 判这一行的锚点（文件头「锚点失效」）：对得上回 None，不然回原因。"""
    text, error = outcome
    if isinstance(error, FileNotFoundError):
        return f"文件 {row.path} 不存在"
    if error is not None:
        return f"文件 {row.path} 读不了（{type(error).__name__}）"
    count = text.count(row.old)
    return None if count == 1 else f"原文在 {row.path} 里命中 {count} 次，要恰好 1 次"


def anchor_problem(root, row):
    """这一行锚点失效的原因（文件头「锚点失效」）：原文在被判仓里「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了；对得上回 None。"""
    return anchor_problem_of(row, source_text_outcome(root, row.path))


def anchor_problems_by_line(root, rows):
    """{行号: anchor_problem 的结果}：每份文件只读一遍，几行改同一份文件共用那一次读；读到的原文只活到这一趟判完（不留在返回值里）。"""
    outcomes, problems = {}, {}
    for row in rows:
        if row.path not in outcomes:
            outcomes[row.path] = source_text_outcome(root, row.path)
        problems[row.line_number] = anchor_problem_of(row, outcomes[row.path])
    return problems


def anchor_broken_rows(root, rows, environment):
    """{行号: 为什么}：锚点失效的那几行（run_stage、plan_context 先把它们剔出去、单记一档）。环境里开了分步进度（merge_progress_variable）就把「核锚点」这一步逐行打进度。"""
    progress = merge_step_open(environment, "核锚点", len(rows))
    broken = {}
    for done, row in enumerate(rows, 1):
        why = anchor_problem(root, row)
        if why is not None:
            broken[row.line_number] = why
        merge_step_row_done(progress, done)
    merge_step_close(progress, f"锚点失效 {len(broken)} 行")
    return broken


def row_problems(root, rows, malformed, copied):
    """派活之前一个工作进程都不起就能判的那几样，crates-mutation-replay（prechecks）与门禁 code-source-discipline（static-check）都调这一份。回 {类别: [明细]}，类别照 ROW_PROBLEM_KINDS：
      MALFORMED (行号, 变异名, 为什么)   不成形（read_table 判的：六段，只有替换文可以空）
      ANCHOR    (行号, 变异名, 为什么)   原文在被判仓里「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了
      NAME      (行号, 变异名, 测试名)   必须红的测试名不是一个用例名（带了正则符号或空白）
      ROW       (行号, 变异名, 文件)     文件不在每片拷的范围里
      STRAY     (行号, 变异名, 段名, 转义) 原文或替换文里有 \\n 以外的反斜杠转义"""
    problems = {kind: [] for kind in ROW_PROBLEM_KINDS}
    problems["MALFORMED"] = list(malformed)
    anchors = anchor_problems_by_line(root, rows)
    for row in rows:
        why = anchors[row.line_number]
        if why is not None:
            problems["ANCHOR"].append((row.line_number, row.name, why))
        if not TEST_NAME_FORM.fullmatch(row.expected):
            problems["NAME"].append((row.line_number, row.name, row.expected))
        for segment_name, segment in (("原文", row.fields[2]), ("替换文", row.fields[3])):
            for escape in stray_escapes(segment):
                problems["STRAY"].append((row.line_number, row.name, segment_name, escape))
    problems["ROW"] = [(row.line_number, row.name, row.path) for row in rows_outside_copy(rows, copied)]
    return problems


def package_directory_of(root, relative_path):
    """被改文件所在的包：从它所在的目录往上找第一份 Cargo.toml，不越过 <根>、也不取 <根> 本身（仓根那一份是工作区）；找不到回 None。"""
    root_directory = os.path.abspath(root)
    directory = os.path.dirname(os.path.abspath(os.path.join(root, relative_path)))
    while directory != root_directory and directory.startswith(root_directory + os.sep):
        if os.path.isfile(os.path.join(directory, "Cargo.toml")):
            return directory
        directory = os.path.dirname(directory)
    return None


def rust_identifier_set(text):
    """一段 Rust 源码里作为标识符出现过的词（lib_function_units.py 的 rust_lex：注释、字符串里的不算；r#name 记成 name）。"""
    units = mutation_units.function_units()
    kinds, texts, _lines, _documents = units.rust_lex(text)
    return {word[2:] if word.startswith("r#") else word for kind, word in zip(kinds, texts) if kind == units.IDENT}


def package_identifier_set(package_directory, cache):
    """包目录底下每份 .rs（跳过 target）里出现过的标识符的并集；cache 是 {包目录: 集合}，同一趟 static-check 里每个包只读一遍。"""
    if package_directory not in cache:
        words = set()
        for current_directory, directory_names, file_names in os.walk(package_directory):
            directory_names[:] = sorted(name for name in directory_names if name != "target")
            for file_name in sorted(file_names):
                if file_name.endswith(".rs"):
                    with open(os.path.join(current_directory, file_name), encoding="utf-8", errors="replace") as handle:
                        words |= rust_identifier_set(handle.read())
        cache[package_directory] = words
    return cache[package_directory]


def rust_unbound_references(kinds, words, watched):
    """一串 rust_lex 词里，watched 中的名字哪几个有「引用」而没有一处「绑定」：按头一次引用的次序回出来（replacement_undefined_identifiers 判 UNDEFINED 用）。
    words 是去掉 r# 的词。认作绑定的：let 之后到 = ; 单个 : 之前、for 之后到 in 之前的模式，闭包两条 | 之间的参数，fn struct enum const static type mod trait union mut ref
    后面紧跟的名字，fn / impl / struct / enum / trait / type 之后尖括号里的泛型参数，后面接单个 : 的（参数名、字段名、类型标注前的绑定），后面接 @ 的，
    match 臂 => 之前模式里小写起头的名字（往后只隔着模式里会有的词就到 =>；越过深度 0 的逗号时到 => 要在外层，越过的右花括号之后只许接 => , ) ] } | if）。
    不算引用的：_ 起头的、#[…] / #![…] 属性里的、跟在 . 后面或 :: 前后的、后面接 ! 的。"""
    units = mutation_units.function_units()
    identifier, punctuation = units.IDENT, units.PUNCT
    count = len(words)
    padded = ["", ""] + words + ["", ""]   # rust_lex 的标点一个字符一个词：「::」是两个「:」，「=>」是「=」「>」；padded[index + 2] 是 words[index]
    single_binders = ("fn", "struct", "enum", "const", "static", "type", "mod", "trait", "union", "mut", "ref")
    pattern_words = ("ref", "mut", "if")
    pattern_punctuation = (",", "(", ")", "[", "]", "{", "}", "|", "@", "&", ".", "-")

    def is_identifier(index):
        return 0 <= index < count and kinds[index] == identifier

    def colon_alone(index):   # words[index] 是单个「:」，不是「::」里的
        return words[index] == ":" and padded[index + 1] != ":" and padded[index + 3] != ":"

    def arrow_at(index):
        return words[index] == "=" and padded[index + 3] == ">"

    def match_arm_binding(index):
        depth, crossed_comma, step = 0, False, index + 1
        while step < count and step < index + 400:
            word = words[step]
            if arrow_at(step):
                return depth < 0 or not crossed_comma
            if word == "if" and kinds[step] == identifier:
                return True
            if kinds[step] == identifier and word not in pattern_words and word in units.RUST_KEYWORDS:
                return False
            if kinds[step] == punctuation and word not in pattern_punctuation:
                return False
            if word in ("(", "[", "{"):
                depth += 1
            elif word in (")", "]", "}"):
                depth -= 1
                if word == "}" and depth < 0 and not (padded[step + 3] == "=" and padded[step + 4] == ">") \
                        and padded[step + 3] not in (",", ")", "]", "}", "|", "if"):
                    return False   # 出了外层的块（match 臂的块收尾），后面接的是下一臂
            elif word == ",":
                if depth < 0:
                    return False
                if depth == 0:
                    crossed_comma = True
            step += 1
        return False

    bound, references, pattern_until, attribute_depth, generic_depth = set(), [], None, 0, 0
    for index in range(count):
        word, kind = words[index], kinds[index]
        if attribute_depth:   # #[…] / #![…] 里：数方括号到收尾
            attribute_depth += {"[": 1, "]": -1}.get(word, 0) if kind == punctuation else 0
            continue
        if kind == punctuation and word == "[" and (padded[index + 1] == "#" or padded[index + 1] == "!" and padded[index] == "#"):
            attribute_depth = 1
            continue
        if generic_depth:   # fn / impl / struct … 之后尖括号里：< 或 , 后面紧跟的名字是泛型参数
            generic_depth += {"<": 1, ">": -1}.get(word, 0) if kind == punctuation else 0
            if kind == identifier and padded[index + 1] in ("<", ",") and word in watched:
                bound.add(word)
            continue
        if kind == punctuation and word == "<" and (is_identifier(index - 1) and padded[index] in ("fn", "struct", "enum", "trait", "type", "union")
                                                    or padded[index + 1] == "impl"):
            generic_depth = 1
            continue
        if pattern_until is not None:
            if word in pattern_until and (word != ":" or colon_alone(index)):
                pattern_until = None
            elif kind == identifier:
                bound.add(word)
            continue
        if kind == identifier and word == "let":
            pattern_until = ("=", ";", ":")
            continue
        if kind == identifier and word == "for" and padded[index + 3] != "<":
            pattern_until = ("in",)
            continue
        if kind == punctuation and word == "|" and padded[index + 1] in ("", "(", ",", "=", "move", "{", ";", ">", "return"):
            pattern_until = ("|",)
            continue
        if kind != identifier or word not in watched:
            continue
        before, after = padded[index + 1], padded[index + 3]
        if padded[index + 1] in single_binders or after == "@" or after == ":" and colon_alone(index + 1) \
                or word[:1].islower() and after not in ("(", "{", "!", ":", ".") and match_arm_binding(index):
            bound.add(word)
            continue
        if word.startswith("_") or before == "." or before == ":" and padded[index] == ":" or after in ("!", ":"):
            continue
        if word not in references:
            references.append(word)
    return [word for word in references if word not in bound]


def replacement_undefined_identifiers(root, row, cache):
    """static-check 的 UNDEFINED（文件头 static-check 那一段）：替换文施加之后新出现的标识符（施加之前整份被改文件里一处都没有的），
    在被改文件所在的包的 .rs 里也一处不出现、又不是替换文自己绑定的，按头一次引用的次序回出来；对得上回 []。
    先粗筛：替换文单独切词，去掉关键字、标准库预导入与基本类型的名字、_ 起头的、被改文件与它所在的包里出现过的；一个都不剩就回 []。
    剩下的再细判：把替换文施加进被改文件、整份切词（这样替换文起止落在字符串或注释中间也切得对），只看施加前没有的那几个名字，
    绑定与不算引用的写法照 rust_unbound_references。cache 是 {包目录: 标识符集合, ("file", 路径): 被改文件的标识符集合}。
    不判的：文件不是 .rs、锚点失效（ANCHOR 已报）。弄坏开关 undefinedidentifiersunchecked：一个都不报。"""
    if break_is(os.environ, "undefinedidentifiersunchecked") or not row.path.endswith(".rs") or anchor_problem(root, row) is not None:
        return []
    file_key = ("file", row.path)
    original_text = read_replaced_source(root, row.path)
    if file_key not in cache:
        cache[file_key] = rust_identifier_set(original_text)
    candidates = replacement_candidates(row, cache[file_key])
    package_directory = package_directory_of(root, row.path) if candidates else None
    if package_directory is not None:
        candidates -= package_identifier_set(package_directory, cache)
    if not candidates:
        return []
    return replacement_unbound(row, original_text, candidates)


def replacement_candidates(row, file_words):
    """replacement_undefined_identifiers 的粗筛头一半：替换文单独切词，去掉关键字、标准库预导入与基本类型的名字、_ 起头的、被改文件里出现过的（file_words），回剩下的集合。"""
    units = mutation_units.function_units()
    predeclared = frozenset((
        "_", "self", "Self", "super", "crate", "std", "core", "alloc",
        "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
        "Some", "None", "Ok", "Err", "Option", "Result", "Vec", "String", "Box", "ToString", "ToOwned", "Clone", "Copy", "Default", "Drop",
        "Eq", "PartialEq", "Ord", "PartialOrd", "Iterator", "IntoIterator", "Extend", "FromIterator", "DoubleEndedIterator", "ExactSizeIterator",
        "Fn", "FnMut", "FnOnce", "From", "Into", "TryFrom", "TryInto", "AsRef", "AsMut", "Send", "Sync", "Sized", "Unpin", "drop"))
    return {word for word in rust_identifier_set(row.new)
            if word not in units.RUST_KEYWORDS and word not in predeclared and not word.startswith("_") and word not in file_words}


def replacement_unbound(row, original_text, candidates):
    """replacement_undefined_identifiers 的细判：把替换文施加进被改文件（original_text）、整份切词，candidates 里有引用没绑定的名字按头一次引用的次序回出来。"""
    units = mutation_units.function_units()
    kinds, texts, _lines, _documents = units.rust_lex(original_text.replace(row.old, row.new, 1))
    words = [word[2:] if word.startswith("r#") else word for word in texts]
    return rust_unbound_references(kinds, words, candidates)


def forked_slice_by_pool(arguments):
    """弄坏开关 forked-parts-by-pool 下进程池的一片：(task, 这一片的几份) → 逐份 task 的结果。"""
    task, items = arguments
    return [task(item) for item in items]


def forked_slice_child(task, items, sender):
    """forked_parts 的一片（在 fork 出来的子进程里跑）：逐份 task 的结果经管道交回 ("ok", 结果)；出错交 ("error", 回溯)。"""
    try:
        sender.send(("ok", [task(item) for item in items]))
    except BaseException:
        sender.send(("error", traceback.format_exc()))
    finally:
        sender.close()


def forked_parts(task, items, workers):
    """items 切成 min(workers, 份数) 片连续的，每片在 fork 出来的一个子进程里逐份算 task，交 [[一片里逐份的结果], …]，片的次序同切的次序。
    用 fork 的 Process 加管道，不用进程池：进程池把 task 当「模块名.函数名」pickle 过去，本模块被按文件路径装成别的模块名（scripts/mutation/prove-red.sh 的
    crates_mutation_rows_for_prove_red 这一类，sys.modules 里没有）时 pickle 不了；fork 出来的子进程直接拿父进程里的 task 与 items，只有结果经管道 pickle。
    子进程出错或没交回结果就退了，抛 RuntimeError 带回溯。调用方先判本进程不是守护进程（守护进程不许起子进程）。
    弄坏开关 forked-parts-by-pool：换回进程池（按名字 pickle task），按文件路径装成别的模块名时抛 PicklingError。"""
    context = multiprocessing.get_context("fork")
    count = min(workers, len(items))
    bounds = [len(items) * index // count for index in range(count + 1)]
    if break_is(os.environ, "forked-parts-by-pool"):   # 弄坏开关：换回进程池，task 按「模块名.函数名」pickle
        with context.Pool(processes=count) as pool:
            return pool.map(forked_slice_by_pool, [(task, items[bounds[index]:bounds[index + 1]]) for index in range(count)])
    started, parts, failures = [], [], []
    try:
        for index in range(count):
            receiver, sender = context.Pipe(duplex=False)
            process = context.Process(target=forked_slice_child, args=(task, items[bounds[index]:bounds[index + 1]], sender))
            process.start()
            sender.close()
            started.append((process, receiver))
        for process, receiver in started:
            try:
                status, value = receiver.recv()
            except EOFError:
                status, value = "error", "没交回结果就退了"
            if status == "ok":
                parts.append(value)
            else:
                failures.append(f"子进程 {process.pid}：{value}")
    finally:
        for process, receiver in started:
            receiver.close()
            process.join()
    if failures:
        raise RuntimeError(f"fork 出来的 {count} 个子进程里有 {len(failures)} 个出错：\n" + "\n".join(failures))
    return parts


def static_check_slices(task, items, workers, environment):
    """[task(item) for item in items]，次序同 items（文件头 static-check「并行」）。workers ≤ 1、不到 lib_mutation_units.py 的 PARALLEL_MINIMUM 份、
    或本进程是守护进程（进程池的工作进程，不许再起子进程）时在本进程里逐份算；不然切成 min(workers, 份数) 片连续的，fork 出来的子进程各算一片（forked_parts），
    按片的次序接起来。只有结果经管道交回，要 pickle 得动；task 与 items 不 pickle（本模块被按文件路径装成别的模块名时照样能用）。
    并回的份数与派出的对不上、子进程出错抛 RuntimeError。弄坏开关 static-check-slices-misordered：并回时片的次序倒过来。"""
    if workers <= 1 or len(items) < mutation_units.PARALLEL_MINIMUM or multiprocessing.current_process().daemon:
        return [task(item) for item in items]
    parts = forked_parts(task, items, workers)
    if break_is(environment, "static-check-slices-misordered"):
        parts.reverse()
    merged = [result for part in parts for result in part]
    if len(merged) != len(items):
        raise RuntimeError(f"static-check 切片并行派出 {len(items)} 份、收回 {len(merged)} 份")
    return merged


def read_replaced_source(root, path):
    """replacement_undefined_identifiers 读被改文件的那一种读法（utf-8，解不了的字节换掉）。"""
    with open(os.path.join(root, path), encoding="utf-8", errors="replace") as handle:
        return handle.read()


def identifier_set_task(arguments):
    """replacement_undefined_identifiers_of_rows 的一份标识符集合（并行时在 fork 出来的子进程里算）：("file", 根, 路径) 是那份文件的，("package", 根, 包目录) 是那个包的。"""
    kind, root, place = arguments
    if kind == "file":
        return rust_identifier_set(read_replaced_source(root, place))
    return package_identifier_set(place, {})


def replacement_unbound_task(arguments):
    """replacement_undefined_identifiers_of_rows 的一行细判（并行时在 fork 出来的子进程里算）：(根, 行, 粗筛剩下的名字)。"""
    root, row, candidates = arguments
    return replacement_unbound(row, read_replaced_source(root, row.path), candidates)


def replacement_undefined_identifiers_of_rows(root, rows, workers, environment):
    """rows（都是 .rs、锚点对得上的行）逐行的 replacement_undefined_identifiers，次序同 rows（文件头 static-check「并行」）。
    workers ≤ 1 时逐行调 replacement_undefined_identifiers（共用一份 cache）。不然分三步，同一趟里共用的只算一次：
      ① 被改文件与它所在的包的标识符集合，每份只切一次词，按份切片并行（static_check_slices）；
      ② 逐行粗筛（replacement_candidates，再减包的集合），在本进程里做；
      ③ 粗筛剩下名字的行，把替换文施加进被改文件整份切词细判（replacement_unbound），按行切片并行。
    包的集合对 rows 里每个被改文件所在的包都先算（逐行调时只在粗筛剩下名字时才算）：集合是同一个，判出的行相同。"""
    if break_is(os.environ, "undefinedidentifiersunchecked"):
        return [[] for _row in rows]
    if workers <= 1:
        cache = {}
        return [replacement_undefined_identifiers(root, row, cache) for row in rows]
    packages = {row.path: package_directory_of(root, row.path) for row in rows}
    places = [("file", root, path) for path in dict.fromkeys(row.path for row in rows)]
    places += [("package", root, directory) for directory in dict.fromkeys(packages.values()) if directory is not None]
    words = dict(zip([(kind, place) for kind, _root, place in places], static_check_slices(identifier_set_task, places, workers, environment)))
    results, pending = [[] for _row in rows], []
    for index, row in enumerate(rows):
        candidates = replacement_candidates(row, words[("file", row.path)])
        if candidates and packages[row.path] is not None:
            candidates -= words[("package", packages[row.path])]
        if candidates:
            pending.append((index, (root, row, candidates)))
    for (index, _task), missing in zip(pending, static_check_slices(replacement_unbound_task, [task for _index, task in pending], workers, environment)):
        results[index] = missing
    return results


def string_literals(text):
    """源码里每一处字符串字面量（"…"、r#"…"#、b"…"）的 (起, 止)，注释里的不算；字面量的止点照 admission.py 的 end_of_literal。"""
    end_of_literal = admission_module().end_of_literal
    position = 0
    while True:
        found = LITERAL_OR_COMMENT_START.search(text, position)
        if found is None:
            return
        start, token = found.start(), found.group(0)
        if token == "//":
            line_end = text.find("\n", start)
            if line_end < 0:
                return
            position = line_end
            continue
        if token == "/*":
            depth, cursor = 0, start
            while cursor < len(text):
                if text.startswith("/*", cursor):
                    depth, cursor = depth + 1, cursor + 2
                elif text.startswith("*/", cursor):
                    depth, cursor = depth - 1, cursor + 2
                    if depth == 0:
                        break
                else:
                    cursor += 1
            position = cursor
            continue
        if token[0] in "br" and start > 0 and (text[start - 1].isalnum() or text[start - 1] == "_"):
            position = start + 1   # 标识符的末一个字母，不是字面量的前缀
            continue
        end = end_of_literal(text, start)
        if end is None:
            position = start + 1
            continue
        if token != "'":
            yield start, end
        position = end


def literal_value(literal):
    """字面量里引号之间的原文：r#"…"#、b"…" 都去掉前缀与引号；转义不还原（路径里用不到）。"""
    raw = re.match(r'^b?r(#*)"(.*)"\1$', literal, re.S)
    if raw:
        return raw.group(2)
    plain = re.match(r'^b?"(.*)"$', literal, re.S)
    return plain.group(1) if plain else literal


def escaping_literals_of_file(arguments):
    """escaping_literals 的一份 .rs（并行时在 fork 出来的子进程里算）：(根, crate 名, 文件的路径, 拷贝范围, include 宏的实参按不按 crate 根解析)
    → 这份文件里逃出 crates/ 的字面量，次序照在文件里的先后。include_str!(…)、include_bytes!(…) 紧跟着的字面量照源文件所在目录解析，别的照 crates/<crate>/ 解析；
    第五项为真（弄坏开关 include-literals-from-crate-root）时 include 宏的实参也照 crates/<crate>/ 解析。"""
    root, crate_name, path, copied, include_literals_from_crate_root = arguments
    relative = os.path.relpath(path, root)
    found = []
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    for start, end in string_literals(text):
        literal = text[start:end]
        value = literal_value(literal)
        if not value.startswith("../"):
            continue
        before = text[max(0, start - 200):start]
        if PATH_ATTRIBUTE_BEFORE_LITERAL.search(before) and PATH_ATTRIBUTE_AFTER_LITERAL.match(text, end):
            continue
        if INCLUDE_MACRO_BEFORE_LITERAL.search(before) and not include_literals_from_crate_root:
            base = os.path.dirname(relative)
        else:
            base = os.path.join("crates", crate_name)
        resolved = os.path.normpath(os.path.join(base, value))
        if resolved == "crates" or resolved.startswith("crates" + os.sep):
            continue
        found.append((relative, text.count("\n", 0, start) + 1, literal, resolved,
                      is_copied_into_each_shard(resolved, copied)))
    return found


def escaping_literals(root, copied, workers, environment):
    """crates/<crate>/ 下每份 .rs（跳过 target）里以 "../ 起头的字符串字面量，解析之后逃出 crates/ 的（include_str!(…)、include_bytes!(…) 的实参照源文件所在目录解析，
    别的照 crates/<crate>/ 解析）：返回 ([(文件, 行号, 字面量, 解析到的路径, 在不在拷贝范围里)], 扫了几份 .rs)。#[path = …] 里的不算。
    先按目录次序列出要扫的文件，再按份切片并行扫（static_check_slices，workers 份），并回的次序同列出的次序。
    弄坏开关 include-literals-from-crate-root：include 宏的实参也照 crates/<crate>/ 解析。"""
    crates_directory = os.path.join(root, "crates")
    if not os.path.isdir(crates_directory):
        return [], 0
    include_literals_from_crate_root = break_is(environment, "include-literals-from-crate-root")
    files = []
    for crate_name in sorted(os.listdir(crates_directory)):
        crate_root = os.path.join(crates_directory, crate_name)
        if not os.path.isdir(crate_root):
            continue
        for current_directory, directory_names, file_names in os.walk(crate_root):
            directory_names[:] = sorted(name for name in directory_names
                                        if not any(fnmatch.fnmatch(name, pattern) for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD))
            files += [(root, crate_name, os.path.join(current_directory, file_name), copied, include_literals_from_crate_root)
                      for file_name in sorted(file_names) if file_name.endswith(".rs")]
    found = [item for part in static_check_slices(escaping_literals_of_file, files, workers, environment) for item in part]
    return found, len(files)


# ── 行键、判定库 ────────────────────────────────────────────────────────────

def read_bytes(path):
    with open(path, "rb") as handle:
        return handle.read()


def selftest_static_check_parallel_repository(root):
    """static-check 切片并行那几格的样本仓（selftest_static_check_parallel_cell 的说明）：可达性样本 crate 加 8 份 extra_<n>.rs，变异表 18 行。"""
    extras = range(1, 9)
    extra_tests = "".join(f"    #[test]\n    fn extra_bump_{number}() {{ assert_eq!(super::extra_{number}::bump_{number}(1), {number + 1}); }}\n" for number in extras)
    source = "".join(f"pub mod extra_{number};\n" for number in extras) + SELFTEST_REACHABILITY_SOURCE.replace(
        "#[cfg(test)]\nmod tests {\n", "#[cfg(test)]\nmod tests {\n" + extra_tests, 1)
    rows = list(SELFTEST_REACHABILITY_ROWS) + [
        f"extra-ghost-{number}\tcrates/tiny/src/extra_{number}.rs\t{{ value + {number} }}\t{{ value + {number} + ghost_{number} }}\t"
        f"-p tiny --lib -- extra_bump_{number}\textra_bump_{number}" for number in extras] + [
        "anchor-missing\tcrates/tiny/src/lib.rs\t{ value * 99 }\t{ value * 98 }\t-p tiny --lib -- plain_sum\tplain_sum",
        "file-missing\tcrates/tiny/src/absent.rs\t{ value }\t{ value + 1 }\t-p tiny --lib -- plain_sum\tplain_sum"]
    build_selftest_repository(root, rows, source=source)
    for number in extras:
        selftest_write(os.path.join(root, "crates", "tiny", "src", f"extra_{number}.rs"),
                       f"pub fn bump_{number}(value: u32) -> u32 {{ value + {number} }}\n\npub const OUTSIDE_{number}: &str = \"../../outside-{number}/data\";\n")


def selftest_static_check_parallel_compare(selftest, root, single_code, single_output, parallel_code, parallel_output):
    """static-check 切片并行那几格的两格对拍（selftest_static_check_parallel_cell 的说明）：单线程与并行两趟逐行相同；ANCHOR、UNDEFINED、IDENTIFIERS_CHECKED 与逐行调 anchor_problem、replacement_undefined_identifiers 拼出的相同。"""
    single_lines, parallel_lines = single_output.splitlines(), parallel_output.splitlines()
    first_difference = next((f"第 {index + 1} 行：单线程「{left}」、并行「{right}」" for index, (left, right) in enumerate(zip(single_lines, parallel_lines)) if left != right),
                            f"行数 {len(single_lines)} 与 {len(parallel_lines)}")
    selftest.check("static-check 切片并行与单线程逐行相同（GATE_MUTATION_KEY_WORKERS=3 与 =1，同一份 18 行的样本仓，退出码相同）",
                   parallel_code == single_code and parallel_lines == single_lines, f"退 {single_code} 与 {parallel_code}；{first_difference}")
    table_rows, _malformed = read_table(os.path.join(root, TABLE))
    cache, judged, expected = {}, 0, []
    expected += [f"ANCHOR\t{row.line_number}\t{row.name}\t{anchor_problem(root, row)}" for row in table_rows if anchor_problem(root, row) is not None]
    for row in table_rows:
        if not row.path.endswith(".rs") or anchor_problem(root, row) is not None:
            continue
        judged += 1
        missing = replacement_undefined_identifiers(root, row, cache)
        if missing:
            expected.append(f"UNDEFINED\t{row.line_number}\t{row.name}\t{row.path}\t{','.join(missing)}")
    expected.append(f"IDENTIFIERS_CHECKED\t{judged}\t{len(table_rows) - judged}")
    got = [line for line in parallel_lines if line.startswith(("ANCHOR\t", "UNDEFINED\t", "IDENTIFIERS_CHECKED\t"))]
    undefined = [line for line in got if line.startswith("UNDEFINED\t")]
    literals = [line for line in parallel_lines if line.startswith("LITERAL\t")]
    selftest.check("static-check 切片并行的 ANCHOR、UNDEFINED、IDENTIFIERS_CHECKED 与逐行调 anchor_problem、replacement_undefined_identifiers 拼出的逐行相同"
                   f"（锚点失效 2 行、UNDEFINED 8 行不少于 {mutation_units.PARALLEL_MINIMUM}、逃出 crates/ 的字面量 8 处，不是空跑）",
                   got == expected and len(undefined) == 8 and len(undefined) >= mutation_units.PARALLEL_MINIMUM and len(literals) == 8
                   and sum(line.startswith("ANCHOR\t") for line in got) == 2,
                   f"并行 {got}；逐行 {expected}；LITERAL {len(literals)} 行")


def selftest_include_macro_literal_cell(selftest, work):
    """拷贝范围 ② 的解析（文件头「拷贝范围」）：crates/demo-tests/src/bin/probe.rs 里 include_str!(…)、include_bytes!(…) 的实参照源文件所在目录解析，
    别的字面量照 crates/demo-tests/ 解析。三格：include 宏的实参落在 crates/ 之内的不报、指到拷贝范围里 litmus/ 的判在范围里；
    不在 include 宏里的同一个字面量逃出 crates/；include_str! 的实参照源文件所在目录解析之后真逃出 crates/ 的照报。
    弄坏开关 include-literals-from-crate-root（include 宏的实参也照 crate 根解析）下头一格判错。"""
    scope_root = os.path.join(work, "include-macro-scope")
    selftest_write(os.path.join(scope_root, "crates", "demo-tests", "src", "bin", "probe.rs"),
                   'const IN_MACRO: &str = include_str!("../../../other-package/Cargo.toml");\n'
                   'const SPLIT: &str = include_str!(\n    "../../../other-package/README.md"\n);\n'
                   'const BYTES: &[u8] = include_bytes!("../../../other-package/data.bin");\n'
                   'const LITMUS: &str = include_str!("../../../../litmus/a.litmus");\n'
                   'const PLAIN: &str = "../../../other-package/Cargo.toml";\n'
                   'const ESCAPING: &str = include_str!("../../../../fixture-data/answer.txt");\n')
    literals, scanned = escaping_literals(scope_root, ["crates", "Cargo.toml", "Cargo.lock", "litmus"], 1, os.environ)
    entries = [(line, resolved, inside) for _file, line, _literal, resolved, inside in literals]
    selftest.check('include 宏的实参照源文件所在目录解析：src/bin/ 下 include_str!("../../../other-package/Cargo.toml")、跨行写的 include_str!(…) 与 include_bytes!(…) '
                   "落在 crates/ 之内不报，include_str! 指到拷贝范围里的 litmus/ 判在范围里",
                   scanned == 1 and not any(line in (1, 3, 5) for line, _resolved, _inside in entries) and (6, "litmus/a.litmus", True) in entries,
                   f"扫了 {scanned} 份；报的（行号、解析到的路径、在不在范围）{entries}")
    selftest.check('不在 include 宏里的同一个字面量照所在 crate 的根解析："../../../other-package/Cargo.toml" 解析到 ../other-package/Cargo.toml、逃出 crates/、判不在范围',
                   (7, "../other-package/Cargo.toml", False) in entries, f"报的 {entries}")
    selftest.check('include_str! 的实参照源文件所在目录解析之后真逃出 crates/ 的照报：src/bin/ 下 include_str!("../../../../fixture-data/answer.txt") 判不在范围',
                   any(line == 8 and not inside for line, _resolved, inside in entries), f"报的 {entries}")


def selftest_run_overrides_outside_service_cell(selftest, work):
    """只给自证换的运行记录、多机配置两个变量只在临时 prune-server 底下认（文件头「判定库」）：自证的基础环境里两个都设着，
    只把临时服务的标记设成空串（演平常跑） ⇒ run_stage 退 1、两个都点名、假 cargo 一次 test 都没起。弄坏开关 run-overrides-honored-outside-service 下这一格判错。"""
    client = mutation_units.Library.client()
    repository = os.path.join(work, "run-overrides-outside-service")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    exit_code, stdout, stderr, invocations = selftest.run(repository, **{client.TEMPORARY_SERVICE_VARIABLE: ""})
    printed = stdout + stderr   # 拒绝句经 print_rejection → say 打在标准输出上
    named = all(f"{name}（「" in printed for name in ("GATE_MUTATION_RUN_RECORDS_DIRECTORY", "GATE_MUTATION_MULTI_HOST_CONFIG"))
    selftest.check("不在临时 prune-server 底下设了只给自证换的运行记录、多机配置两个变量：退 1、两个都点名、一条都不跑",
                   exit_code == 1 and "不在临时 prune-server 底下设了" in printed and named and not invocations,
                   f"退 {exit_code}、假 cargo 起了 {len(invocations)} 次；{printed[-500:]}")
