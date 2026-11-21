"""Rust 的闭包：从入口顺 R-目标、R-裸名、R-路径、R-方法、R-隐式、R-开放、R-结构、R-宏收闭包（rust_closure_of），WGSL 外名的着落（WgslNameSources）。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import posixpath
import re
from lib_function_units_lexer import IDENT, LITERAL, active_breaks
from lib_function_units_base import Refusal, UsageError, data_directory_unit, data_file_unit, snippet_of
from lib_function_units_shell_index import language_of_file
from lib_function_units_rust_file import (
    PRIMITIVE_TYPES, RUST_KEYWORDS, RUST_KINDS_DEFINING_NAMES, RUST_STRUCTURE_ONLY_KINDS, RUST_TYPE_KINDS, STANDARD_CRATE_PATH_ROOTS,
    TEST_ATTRIBUTE_RAW, bare_name_only_a_local_binding, cfg_atom_text, cfg_tree_leaves, rust_string_value,
)
from lib_function_units_rust_workspace import (
    CfgEvaluator, Closure, DEFAULT_RUST_CFG, cfg_entries, doctest_entry_group, rust_unit_is_test, rust_unit_is_test_code,
)
from lib_function_units_wgsl import literal_mentions, literal_prefixes, wgsl_closure, wgsl_file, wgsl_top_level_names_in


def rust_literal_texts(workspace, units):
    """旧口径（只给弄坏开关 wgsl-external-names-by-long-identifiers）：闭包里 Rust 单元的串字面量的值与长于 3 个字符的标识符，
    WGSL 外名是其中之一的子串、或以串里 <名>{ 的前缀起头就算有着落。"""
    texts = []
    for unit in units:
        if unit.language != "rust" or unit.first_token is None:
            continue
        rust_file = workspace.files[unit.file]
        for index in range(unit.first_token, unit.last_token):
            if rust_file.kinds[index] == LITERAL:
                value = rust_string_value(rust_file.texts[index])
                if value:
                    texts.append(value)
            elif rust_file.kinds[index] == IDENT and len(rust_file.texts[index]) > 3:
                texts.append(rust_file.texts[index])
    return texts


SHORTEST_WGSL_EXTERNAL_NAME_PREFIX_CHARACTERS = 2
# 按名字定义的名字能给 WGSL 外名着落的种类（crates/singlefs-unit-graph 的 defines_names_a_kernel_can_use）：常量、静态量与条目级宏调用
WGSL_NAME_DEFINING_KINDS = frozenset(("const", "static", "item_macro"))


def rust_declared_names(unit):
    """单元按名字定义的名字（crates/singlefs-unit-graph 的 declared_names）：条目级宏调用与 extern 块是它里面定义的那几个，
    assume_specification 是它点名路径末段的名字，别的是它自己的名字。"""
    if unit.kind in RUST_KINDS_DEFINING_NAMES:
        return unit.defines or set()
    return {unit.short}


def name_and_underscore_suffixes(name):
    """名字与它每个 _ 之后的尾段（GPU_FAILURE_POINT_NONE 交它自己、FAILURE_POINT_NONE、POINT_NONE、NONE）：外名以 _ 加它收尾的名字都算供它的名字。"""
    out = {name}
    for position, character in enumerate(name):
        if character == "_" and position + 1 < len(name):
            out.add(name[position + 1:])
    return out


def ascii_upper(text):
    """只把 ASCII 小写字母换成大写（与 Rust 的 to_ascii_uppercase 同；str.upper 会动非 ASCII 字符）。"""
    return "".join(character.upper() if "a" <= character <= "z" else character for character in text)


def rust_string_literal_words(token):
    """串字面量（原文以 " 或 r 起头）原文里 ASCII 字母、数字、_ 连成的每一段。"""
    return re.findall(r"[A-Za-z0-9_]+", token)


class WgslNameSources:
    """闭包里 Rust 单元给 WGSL 外名的着落（照 crates/singlefs-unit-graph 的 WgslExternalNameSources::supply）：
    ① 整名是某个 Rust 串字面量（原文以 " 或 r 起头）里的一个词（ASCII 字母、数字、_ 连成的一段）；
    ② 整名是闭包里常量、静态量、条目级宏调用按名字定义的名字，或这样一个名字以 _ 加整名收尾（GPU_FAILURE_POINT_NONE 供 FAILURE_POINT_NONE）；
    ③ 外名的首段连 _（按第一个 _ 切、连那个 _，T_HEADER 取 T_、FAILURE_POINT_NONE 取 FAILURE_）不短于 2 个字符、是某个 Rust 单元的串字面量原文
       或 macro_rules! 单元任一词法单元原文的子串，且其余那一段（不空，不分大小写）是闭包里某个串字面量的一个词，或闭包里某个单元定义或点名的名字。
    名字里没有 _ 的只认整名。suppliers(名字) 交（有没有着落, 给它着落的那几个单元）：WGSL 入口经 --rust-strings-entry 时把它们连同它们的闭包并进来。
    弄坏开关 wgsl-defined-name-suffixes-ignored 下不认 ②，wgsl-prefix-remainders-unchecked 下 ③ 不核其余那一段；
    wgsl-external-names-by-long-identifiers 下退回旧口径（rust_literal_texts：串的值与长于 3 个字符的标识符，不交单元）。"""

    def __init__(self, workspace, units):
        breaks = active_breaks()
        self.legacy = "wgsl-external-names-by-long-identifiers" in breaks
        self.check_remainders = "wgsl-prefix-remainders-unchecked" not in breaks
        self.words, self.texts, self.defined = {}, [], {}
        self.words_upper, self.names_upper = set(), set()
        if self.legacy:
            self.legacy_texts = rust_literal_texts(workspace, units)
            self.legacy_prefixes = literal_prefixes(self.legacy_texts)
            return
        for unit in units:
            if unit.language != "rust":
                continue
            declared = rust_declared_names(unit)
            if unit.kind in WGSL_NAME_DEFINING_KINDS and "wgsl-defined-name-suffixes-ignored" not in breaks:
                for name in declared:
                    for suffix in name_and_underscore_suffixes(name):
                        self.defined.setdefault(suffix, {})[id(unit)] = unit
            self.names_upper |= {ascii_upper(name) for name in declared}
            self.names_upper |= {ascii_upper(name) for name in unit.plain or () if name not in RUST_KEYWORDS}
            if unit.first_token is None:
                continue
            rust_file = workspace.files[unit.file]
            macro_body = unit.kind == "macro_rules"
            for index in range(unit.first_token, unit.last_token):
                token = rust_file.texts[index]
                literal = rust_file.kinds[index] == LITERAL and token[:1] in ('"', "r")
                if literal:
                    for word in rust_string_literal_words(token):
                        self.words.setdefault(word, {})[id(unit)] = unit
                if literal or macro_body:
                    self.texts.append((token, unit))
        self.words_upper = {ascii_upper(word) for word in self.words}

    def suppliers(self, name):
        """→ (有没有着落, 给它着落的单元列表)。"""
        if self.legacy:
            return literal_mentions(self.legacy_texts, self.legacy_prefixes, name), []
        if name in self.words or name in self.defined:
            found = dict(self.words.get(name, {}))
            found.update(self.defined.get(name, {}))
            return True, list(found.values())
        if "_" not in name:
            return False, []
        prefix = name[:name.index("_") + 1]
        remainder = ascii_upper(name[len(prefix):])
        if len(prefix) < SHORTEST_WGSL_EXTERNAL_NAME_PREFIX_CHARACTERS or not remainder:
            return False, []
        if self.check_remainders and remainder not in self.words_upper and remainder not in self.names_upper:
            return False, []
        found = {}
        for token, unit in self.texts:
            if prefix in token:
                found[id(unit)] = unit
        return bool(found), list(found.values())


def rust_entry_dependency_sections(target_kind, test_compiled):
    """入口目标顺自己 crate 的哪几段依赖解特性（crates/singlefs-unit-graph 的 dependency_sections_of）：构建脚本只顺 build 依赖；
    按 cfg(test) 编的 lib 与 bin、集成测试、example、bench、文档测试连 dev 依赖；不按 cfg(test) 编的 lib 与 bin 只顺普通依赖。"""
    if target_kind == "build":
        return ("build",)
    if test_compiled or target_kind in ("test", "bench", "example", "doctest"):
        return ("normal", "dev")
    return ("normal",)


def entry_names_only_defined(workspace, entry):
    """入口单元自己的名字在它的词法单元里只出现在定义那一处：交 {名字}，不然交空集（文件头 Rust 闭包「入口名字的定义那一处」）。"""
    if entry.first_token is None or entry.last_token is None or not entry.short:
        return frozenset()
    rust_file = workspace.files[entry.file]
    occurrences = sum(1 for index in range(entry.first_token, entry.last_token)
                      if rust_file.kind(index) == IDENT and rust_file.t(index) == entry.short)
    return frozenset((entry.short,)) if occurrences <= 1 else frozenset()


def rust_entry_is_the_main_of_a_program(workspace, target, roots, compiled_as_test):
    """这一趟的入口恰好是一个 bin 目标自己的 fn main（crates/singlefs-unit-graph 的 main_function_of：目标根文件里排头一个的顶层 fn main；一组入口里没有别的）、
    不按测试编：跑起来的进程从这个 main 起（文件头「程序起自己」）。弄坏开关 rust-own-program-entry-unchecked 下入口是什么都算。"""
    if "rust-own-program-entry-unchecked" in active_breaks():
        return True
    if target.kind != "bin" or compiled_as_test:
        return False
    main = next((unit for unit in workspace.files[target.root_file].units if unit.kind == "fn" and unit.short == "main" and unit.container is None and not unit.scope), None)
    return main is not None and len(roots) == 1 and roots[0] is main


def rust_closure_of(workspace, entry, target_id, cfg=None, no_default_cfg=False, features=(), no_default_features=False,
                    entry_definition_name_unresolved=False, stop_at=()):
    """从入口收 Rust 闭包（3.3 八条规则），连同入口闭包拼出的 WGSL 模块（3.5）。entry_definition_name_unresolved 见文件头 Rust 闭包「入口名字的定义那一处」，
    stop_at 见同一节「到这些单元就停」。"""
    breaks = active_breaks()
    closure = Closure()
    stopped_units = {id(unit) for source, unit in stop_at if source is None}
    stopped_edges = {(id(source), id(unit)) for source, unit in stop_at if source is not None}
    # 文档测试入口照 crates/singlefs-unit-graph 的 resolve_entry_units：所挂条目或模块的全部文档测试一起当入口、一起记进 entry 行（doctest_entry_group）；
    # 弄坏开关 rust-doctest-entry-alone：只取交进来的那一个（Rust 那一份记一组、这里记一个，两边的 entry 行与闭包对不上）
    roots = doctest_entry_group(workspace, entry) if entry.kind == "doctest" and "rust-doctest-entry-alone" not in breaks else [entry]
    closure.entry_units.extend(roots)
    entry_unresolved_names = entry_names_only_defined(workspace, entry) if entry_definition_name_unresolved else frozenset()
    target = workspace.targets[target_id]
    test_compiled = rust_unit_is_test(workspace, entry, target_id) if target.kind != "doctest" else False
    # 入口里是测试的那几个单元（连同包它的 impl / trait 头）：文档测试，或 rust_unit_is_test 认的（测试目标里的、带 #[test] 或 cfg(test) 的）。
    # entry_key_lines 照记它们的指纹，落在测试服务代码里也照记（文件头「测试服务代码不进键」：测试照进键，服务代码只记是哪一个）
    closure.test_code_entry_units = set()
    for root in roots:
        owner, visited = root, set()
        while owner is not None and id(owner) not in visited:
            visited.add(id(owner))
            if owner.kind == "doctest" or (target.kind != "doctest" and rust_unit_is_test(workspace, owner, target_id)):
                closure.test_code_entry_units.add(id(owner))
            owner = owner.container
    linked = set(workspace.visible(target_id, development=test_compiled))
    linked_crates = {workspace.targets[identifier].crate for identifier in linked}
    requested = []
    for item in features:
        package, _slash, feature = item.partition("/")
        requested.append((workspace.crate_by_package(package), feature))
    enabled = workspace.resolve_features(linked_crates, target.crate, requested, no_default_features,
                                         rust_entry_dependency_sections(target.kind, test_compiled))
    names, pairs = cfg_entries(([] if no_default_cfg else list(DEFAULT_RUST_CFG)) + list(cfg or ()))
    entry_is_the_main_of_a_program = rust_entry_is_the_main_of_a_program(workspace, target, roots, test_compiled or "test" in names)
    known_extra = set(names) | {key for key, _value in pairs}
    for crate in workspace.crates:
        known_extra |= crate.check_cfg
    test_targets = {target_id} if test_compiled else set()
    evaluator = CfgEvaluator(names, pairs, enabled, test_targets, known_extra)
    closure.cfg_lines = sorted(set(names) | {f'{key}="{value}"' for key, value in pairs}
                               | {f'feature="{feature}"@{crate.package}' for crate in linked_crates for feature in enabled[crate.directory]}
                               | {f"test@{identifier}" for identifier in test_targets})
    closure.entry_target_kind = target.kind
    closure.entry_cfg_atoms = sorted(set(names) | {f'{key}="{value}"' for key, value in pairs} | ({"test"} if test_compiled else set())
                                     | {f'feature="{feature}"' for feature in enabled[target.crate.directory]})
    # 键里的 cfg 成分：闭包里各单元真引到的原子，各带收它进来那个目标下的真值（record_cfg_atoms 逐个记）
    closure.cfg_atoms = set()
    for identifier in linked:
        if workspace.targets[identifier].kind == "doctest":
            continue
        for refusal in workspace.targets[identifier].refusals:
            closure.refusals.add(refusal)
        for path in workspace.targets[identifier].files:
            closure.refusals.update(workspace.files[path].refusals)
        crate = workspace.targets[identifier].crate
        if crate.proc_macro:
            closure.refusals.add(Refusal("proc-macro-crate", posixpath.join(crate.directory, "Cargo.toml"), 1, f"proc-macro = true（{crate.package}）"))
    no_open = "rust-open-entry-traits-ignored" in breaks
    # R-开放只管入口签名；闭包里单元的 dyn Trait 不开放全部 impl（值来自闭包里点名的具体类型，R-隐式已连上它的 trait impl）。
    # 弄坏开关 rust-dyn-traits-open-every-implementation 下闭包里单元的 dyn Trait 照旧开放这个 trait 的全部 impl
    open_interior_dyn = "rust-dyn-traits-open-every-implementation" in breaks
    no_constants = "rust-constants-dropped" in breaks
    no_macros = "rust-macros-dropped" in breaks
    no_trait_impls = "rust-trait-impl-edges-dropped" in breaks
    ignore_cfg = "rust-cfg-ignored" in breaks
    no_module_heads = "rust-module-heads-unlinked" in breaks
    no_declarations = "rust-module-declarations-unlinked" in breaks
    no_aliases = "rust-use-aliases-unresolved" in breaks
    trait_self_paths = "rust-trait-self-paths-ignored" not in breaks
    # blanket impl（impl<T: …> Tr for T）够得到的方法算边（python 与 crates/singlefs-unit-graph 两份都这样算）；
    # 弄坏开关 rust-blanket-impls-ignored 下 R-方法、R-路径不放行 blanket impl，trait 头进闭包也不带它的 blanket impl（Rust 现在的写法）
    blanket = "rust-blanket-impls-ignored" not in breaks
    # 文件头 Rust 闭包「条目级宏调用生成的条目」「#[global_allocator]」「外部类型上的 trait impl」：各一个弄坏开关去掉这些连边
    macro_types_named = "rust-macro-generated-types-unnamed" not in breaks
    allocators_linked = "rust-global-allocators-unlinked" not in breaks
    foreign_impls = workspace.foreign_trait_impls if "rust-foreign-self-trait-impls-unlinked" not in breaks else frozenset()
    # 文件头 Rust 闭包「工作区外的 path 依赖」：别的包按裸名连进来要经 use 或路径；同一个目标里只连所在模块看得见的那几份。各一个弄坏开关去掉这些限制
    outside_names_imported_only = "rust-outside-crate-names-linked-unimported" not in breaks
    outside_names_by_module = "rust-outside-crate-names-linked-across-modules" not in breaks
    # 文件头 Rust 闭包「R-裸名同一个目标里按模块筛」：crates/ 下的包同一个目标里同样只连所在模块看得见的那几份
    crates_names_by_module = "rust-crates-names-linked-across-modules" not in breaks
    local_bindings_shadow = "rust-let-shadowed-names-resolved" not in breaks
    doctest_structure = "rust-doctest-structure-linked" in breaks
    body_cfg_atoms = "rust-cfg-macro-atoms-ignored" not in breaks
    raw_wgsl = "wgsl-raw-text" in breaks
    pull_suppliers = not ({"wgsl-name-suppliers-not-pulled-in", "wgsl-external-names-by-long-identifiers"} & breaks)
    pull_wgsl_definers = "wgsl-other-file-definitions-unlinked" not in breaks
    suppliers_skip_test_code = "wgsl-name-suppliers-include-test-code" not in breaks
    modules_per_naming_unit = "wgsl-modules-per-naming-unit" in breaks
    structure_only = RUST_STRUCTURE_ONLY_KINDS - ({"use"} if "rust-use-names-resolved" in breaks else set())
    dropped = set()
    if no_constants:
        dropped |= {"const", "static"}
    if no_macros:
        dropped |= {"macro_rules", "item_macro"}
    named = set(PRIMITIVE_TYPES)
    methods_seen, dyn_seen, seen = set(), set(), set()
    wgsl_modules = set()
    wgsl_paths = set()           # 闭包里单元 include_str! / include_bytes! 读的 .wgsl：合成一个模块（design 3.5，与 crates/singlefs-unit-graph 的 wgsl_files 同）
    suppliers_tried = set()
    pending_children = []        # 起子进程的测试名是参数的：(单元, 函数名, 位置, 目标, current_exe 的词法单元下标)
    data_units = {}
    todo = []

    def record_cfg_atoms(unit, identifier):
        """这个单元在这个目标下真引到的 cfg 原子与真值进 closure.cfg_atoms（「<原子>\\t<true|false>」）：它自己连外层的条件链（#[test] 不算），
        与它词法单元里每一处 #[cfg(…)] / #![cfg(…)] / #[cfg_attr(条件, …)] 的条件与里面套的 cfg(…)、cfg!(…)。认不出、判不了真假的拒算 cfg-unrecognized。"""
        if ignore_cfg:
            return
        crate_directory = workspace.targets[identifier].crate.directory
        sources = [condition for condition in unit.conditions or () if condition[1] != TEST_ATTRIBUTE_RAW]
        if body_cfg_atoms:
            sources += list(unit.cfg_leaves or ())
        for tree, raw, line in sources:
            if tree is None:
                closure.refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
                continue
            for leaf in cfg_tree_leaves(tree):
                try:
                    holds = evaluator.value(leaf, crate_directory, identifier)
                except ValueError:
                    closure.refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
                    continue
                closure.cfg_atoms.add(f"{cfg_atom_text(leaf)}\t{'true' if holds else 'false'}")

    def push_exact(unit, identifier, via, rule):
        if (id(unit), identifier) not in seen:
            todo.append((unit, identifier, via, rule))

    def push(unit, identifier, via, rule):
        owners = workspace.unit_targets.get(id(unit), ())
        if identifier in owners:
            push_exact(unit, identifier, via, rule)
            return
        for owner in sorted(owners):
            if owner in linked:
                push_exact(unit, owner, via, rule)

    def push_impl(head, identifier, via, rule):
        push(head, identifier, via, rule)
        for member in workspace.members_of.get(id(head), ()):
            push(member, identifier, head, "member")

    def active(unit, identifier):
        if ignore_cfg:
            return True
        crate_directory = workspace.targets[identifier].crate.directory
        chain = list(workspace.targets[identifier].files.get(unit.file, ((), []))[1]) + list(unit.conditions or ())
        for tree, raw, line in chain:
            if tree is None:
                closure.refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
                continue
            try:
                if not evaluator.value(tree, crate_directory, identifier):
                    return False
            except ValueError:
                closure.refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
        return True

    def name_type(type_name, identifier, via):
        if type_name in named:
            return
        named.add(type_name)
        if not no_trait_impls:
            for head in workspace.implicit_impls.get(type_name, ()):
                push_impl(head, identifier, via, f"trait-impl-of {type_name}")
        for head in workspace.inherent_impls.get(type_name, ()):
            for member in workspace.members_of.get(id(head), ()):
                if member.short in methods_seen:
                    push(member, identifier, via, f"method-late {member.short}")

    def dyn_trait(trait_name, identifier, via):
        if trait_name in dyn_seen:
            return
        dyn_seen.add(trait_name)
        for head in workspace.impls_of_trait.get(trait_name, ()):
            push_impl(head, identifier, via, f"open {trait_name}")

    def entry_signature_traits(unit):
        if unit.first_token is None:
            return set()
        rust_file = workspace.files[unit.file]
        out, i = set(), unit.first_token
        while i < unit.last_token and rust_file.t(i) != "{":
            if rust_file.t(i) in ("impl", "dyn", ":", "+") and rust_file.kind(i + 1) == IDENT:
                out.add(rust_file.t(i + 1))
            i += 1
        return out

    def collect_shell_sources(path, reader, line):
        """读到的数据文件是 shell 脚本：它（传递地）source / . 进来的文件各收成一个 data-file 单元，折不出的拒算（文件头「Rust 闭包读到的 shell 脚本」）；
        每读一处这份脚本（reader 那个单元的第 line 行），被 source 的每一份在那一处记一条 data_sites（crates/singlefs-unit-graph 的 follow_shell_sources 同）。
        弄坏开关 shell-data-sources-unfollowed：一份都不收；shell-data-sources-unsited：照收、不记 data_sites。"""
        if "shell-data-sources-unfollowed" in breaks or language_of_file(workspace.root, path) != "shell":
            return
        sourced, refusals = workspace.shell_sourced(path)
        closure.refusals.update(refusals)
        for file in sourced:
            if file not in data_units:
                data_units[file] = data_file_unit(workspace.root, file)
            closure.add(data_units[file], data_units[path], "shell-source")
            if "shell-data-sources-unsited" not in breaks:
                closure.data_sites.add((reader.file, line, file, data_units[file].fingerprint, id(reader), "data"))

    def collect_data(unit, identifier):
        """这个单元读的数据文件进闭包（一份文件一个 data-file 单元），另按读它的那一处记一条 data_sites（Rust 那一份的数据单元写法，unit_graph_key 用）；
        OUT_DIR 与 CARGO_BIN_EXE_<名> 连到构建脚本与那个 bin 的 main。"""
        crate = workspace.targets[identifier].crate
        for kind, value, index in unit.data_references or ():
            if kind == "file":
                if value not in data_units:
                    data_units[value] = data_file_unit(workspace.root, value)
                line = workspace.files[unit.file].lines[index]
                collect_shell_sources(value, unit, line)   # 每一处读都记：被 source 的那几份的数据位置落在读这份脚本的那一处
                closure.add(data_units[value], unit, "data")
                closure.data_sites.add((unit.file, line, value, data_units[value].fingerprint, id(unit), "data"))
            elif kind == "current-exe-child":
                if value[0] == "parameter":
                    pending_children.append((unit, value[1], value[2], identifier, index))
                else:
                    name = folded_test_name(unit, value[1])
                    test = workspace.exact_test(name, identifier) if name is not None else None
                    if test is None:
                        refuse_child(unit, index)
                    elif "rust-current-exe-children-unlinked" not in breaks:
                        push_exact(test, identifier, unit, "current-exe-child")
            elif kind == "own-program-restart":
                # 路径原样交给 Command::new：起的是这个程序自己。入口恰好是 bin 的 main 的，子进程从同一个 main 进，走的代码都在闭包里，不连边、不拒算
                if not entry_is_the_main_of_a_program:
                    refuse_child(unit, index)
            elif kind == "directory":
                if value not in data_units:
                    data_units[value] = data_directory_unit(workspace.root, value)
                closure.add(data_units[value], unit, "data")
                closure.data_sites.add((unit.file, workspace.files[unit.file].lines[index], value, data_units[value].fingerprint, id(unit), "data-directory"))
            elif kind == "wgsl":
                if raw_wgsl:
                    if value not in data_units:
                        data_units[value] = data_file_unit(workspace.root, value)
                    closure.add(data_units[value], unit, "data")
                else:
                    wgsl_paths.add(value)
            elif kind == "out-directory":
                build = workspace.targets.get(f"{crate.label}:build:{crate.label}")
                mains = [candidate for candidate in workspace.files[build.root_file].units if candidate.kind == "fn" and candidate.short == "main"
                         and candidate.container is None] if build else []
                if not mains:
                    workspace.files[unit.file].refuse(unit, "include-argument", index)
                    closure.refusals.update(unit.refusals)
                for main in mains:
                    push_exact(main, build.id, unit, "out-directory")
            elif kind == "build-environment":
                # env!("X") 的 X 不是 cargo 给的：连到本 crate 构建脚本的 main（它经 cargo::rustc-env 交这个值）；没有构建脚本的不连
                # （值来自编译时的进程环境，不在仓里的代码里）。弄坏开关 rust-build-environment-unlinked：不连
                if "rust-build-environment-unlinked" in breaks:
                    continue
                build = workspace.targets.get(f"{crate.label}:build:{crate.label}")
                for main in ([candidate for candidate in workspace.files[build.root_file].units if candidate.kind == "fn" and candidate.short == "main"
                              and candidate.container is None] if build else []):
                    push_exact(main, build.id, unit, "build-environment")
            elif kind == "bin-executable":
                bin_target = workspace.targets.get(f"{crate.label}:bin:{value}")
                mains = [candidate for candidate in workspace.files[bin_target.root_file].units if candidate.kind == "fn" and candidate.short == "main"
                         and candidate.container is None] if bin_target else []
                if not mains:
                    closure.refusals.add(Refusal("bin-target-missing", unit.file, unit.first_line, f"CARGO_BIN_EXE_{value}"))
                for main in mains:
                    push_exact(main, bin_target.id, unit, "bin-executable")

    if not active(entry, target_id):
        raise UsageError(f"入口 {entry.file}::{entry.name} 在这一组 cfg 下不编（{'、'.join(closure.cfg_lines) or '空'}）：用 --feature <包名>/<特性> 或 --cfg 打开它")
    for root in roots:
        todo.append((root, target_id, None, "entry"))
        for trait_name in entry_signature_traits(root) if not no_open else ():
            dyn_trait(trait_name, target_id, root)
    for allocator in workspace.global_allocators if allocators_linked else ():
        # #[global_allocator] 的 static 管整个链接出来的程序：链接集里有它所在目标的，它进闭包（连同它的类型、R-隐式带上 GlobalAlloc 的 impl）
        for owner in sorted(workspace.unit_targets.get(id(allocator), ())):
            if owner in linked:
                push_exact(allocator, owner, None, "global-allocator")

    def drain():
        while todo:
            unit, identifier, via, rule = todo.pop()
            if (id(unit), identifier) in seen:
                continue
            if unit is not entry and (id(unit) in stopped_units or (via is not None and (id(via), id(unit)) in stopped_edges)):
                # 到这些单元就停（文件头 Rust 闭包那一节）：按边停的不记进 seen，别的单元连到它照连
                continue
            seen.add((id(unit), identifier))
            if unit.kind in dropped or not active(unit, identifier):
                continue
            if unit.ghost_only and unit is not entry:
                # 产品行为视图里只给 Verus 看的单元（文件头「判定种类」）：不进闭包、不往下连
                continue
            closure.add(unit, via, rule)
            closure.unit_targets.setdefault(id(unit), identifier)
            closure.refusals.update(unit.refusals)
            record_cfg_atoms(unit, identifier)
            if unit.kind == "extern_block" and (not unit.listed_c_library_calls or workspace.target_may_link_libraries_beyond_the_system_library(target_id, identifier)):
                closure.refusals.add(Refusal("extern-block", unit.file, unit.first_line, workspace.files[unit.file].row(unit.first_line)))
            alias_originals = []
            if unit.kind != "doctest" or doctest_structure:
                # R-结构：成员连头；单元连所在模块的模块头、把它所在文件连进这个目标模块树的 mod 声明（经它再连上层模块的头与声明）、
                # 所在模块里引进了它用到的名字的 use（glob 一律连）；use a::b as c 里用到 c 的，b 按 R-裸名再解。
                # 文档测试是另编的一个 crate，挂在哪个条目下只管它叫什么：不连所挂的条目、也不连那个模块的 use（crates/singlefs-unit-graph 的 follow_structure 同）
                if unit.container is not None:
                    push(unit.container, identifier, unit, "container")
                if unit.module_head is not None and not no_module_heads:
                    push(unit.module_head, identifier, unit, "module-head")
                if unit.kind != "doctest" and not no_module_heads and not no_declarations:
                    for declaring in workspace.targets[identifier].declared_by.get(unit.file, ()):
                        push_exact(declaring, identifier, unit, "mod-declaration")
                references = (unit.references or set()) - (entry_unresolved_names if unit is entry else frozenset())
                for use in workspace.uses_by_scope.get((unit.file, unit.scope), ()):
                    if use is not unit and (use.import_glob or (use.imports & references)):
                        push(use, identifier, unit, "use")
                        if not no_aliases:
                            alias_originals += [original for alias, original in use.aliases if alias in unit.plain]
            if unit.kind in structure_only:
                # use、mod 声明、模块头、extern crate 只是结构：读文件照收，里面的名字不往下解（crates/singlefs-unit-graph 的 resolves_its_own_names 同一口径）
                collect_data(unit, identifier)
                continue
            visible = workspace.visible(identifier, development=identifier in test_targets)
            unit_crate = workspace.targets[identifier].crate
            # macro_rules! 体里的裸名在调用处解，定义所在模块看不出它落在哪一份：一律全连
            resolved_at_call_site = unit.kind == "macro_rules"
            # 路径根名是这几个的指别的 crate（标准库与看得见的别的 crate 的库名），不点本目标里的哪一份（candidates_seen_from_module）
            other_crate_roots = frozenset(STANDARD_CRATE_PATH_ROOTS) | {
                workspace.targets[visible_id].crate.lib_name for visible_id in visible if workspace.targets[visible_id].crate is not unit_crate}
            for name in sorted(unit.plain) + alias_originals:
                if unit is entry and name in entry_unresolved_names:
                    continue
                # 只在这个名字真有同名函数要连时算一次（文件头 Rust 闭包「局部 let 与参数绑定遮住的名字」）
                only_a_local_binding = None
                for visible_id in visible:
                    table = workspace.by_name.get(visible_id)
                    if table is None:
                        continue
                    candidates = table.get(name, ())
                    visible_crate = workspace.targets[visible_id].crate
                    if candidates and visible_crate.outside_workspace and outside_names_imported_only and visible_crate is not unit_crate:
                        # 工作区外的 crate 按调用关系连（文件头 Rust 闭包「工作区外的 path 依赖」）：从别的包连进来，这个名字要在工作区里经 use 或路径
                        # 从它引进来过（验证包的 run 字段不连到 prune-server 的 command_line::run）
                        every_name, reaching = workspace.names_reaching_outside_crate(visible_crate)
                        if not every_name and name not in reaching:
                            continue
                    by_module = outside_names_by_module if visible_crate.outside_workspace else crates_names_by_module
                    if candidates and by_module and visible_id == identifier and name not in alias_originals and not resolved_at_call_site:
                        # 同一个目标里只连这个单元所在模块看得见的那几份（文件头 Rust 闭包「R-裸名同一个目标里按模块筛」：crates/ 下的包与工作区外的 crate 一样；
                        # 验证包里 CPU 判器的单元经 use 从格式库引进 SLOT_BYTES，不连到 gpu_unit_checks 里私有的同名常量）
                        candidates = workspace.candidates_seen_from_module(unit, identifier, name, candidates, other_crate_roots)
                    for candidate in candidates:
                        if candidate is unit:
                            continue
                        if local_bindings_shadow and candidate.kind == "fn" and unit.kind == "fn" and name not in alias_originals and unit.first_token is not None:
                            if only_a_local_binding is None:
                                rust_file = workspace.files[unit.file]
                                only_a_local_binding = bare_name_only_a_local_binding(rust_file.kinds, rust_file.texts, unit.first_token, unit.last_token, name)
                            if only_a_local_binding:
                                continue
                        if visible_id != identifier and not candidate.public:
                            continue
                        push_exact(candidate, visible_id, unit, f"name {name}")
            for name in unit.methods:
                methods_seen.add(name)
                for member in workspace.members_by_name.get(name, ()):
                    container = member.container
                    if container.kind == "trait" or (blanket and container.blanket) or (container.self_types & named) or id(container) in foreign_impls:
                        push(member, identifier, unit, f"method {name}")
            own = unit.container
            for qualifier, name in unit.qualified:
                for member in workspace.members_by_name.get(name, ()):
                    container = member.container
                    if container.kind == "trait" and container.short == qualifier:
                        push(member, identifier, unit, f"path {qualifier}::{name}")
                    elif (trait_self_paths and container.kind == "trait" and qualifier == "Self" and own is not None and own.kind == "trait"
                          and own.short == container.short):
                        # trait 成员里的 Self::x 落到这个 trait 自己的成员 x（crates/singlefs-unit-graph 的 follow_qualified_paths 同）
                        push(member, identifier, unit, f"path Self::{name}")
                    elif container.kind == "impl" and qualifier in container.self_types:
                        push(member, identifier, unit, f"path {qualifier}::{name}")
                    elif container.kind == "impl" and qualifier == "Self" and own is not None and (
                            container.self_types & own.self_types if trait_self_paths else container.self_types & (own.self_types or {own.short})):
                        push(member, identifier, unit, f"path Self::{name}")
                    elif container.kind == "impl" and container.is_trait_impl and container.trait_name == qualifier and (
                            (blanket and container.blanket) or container.self_types & named or id(container) in foreign_impls):
                        push(member, identifier, unit, f"path {qualifier}::{name}")
            if unit.kind in RUST_TYPE_KINDS:
                name_type(unit.short, identifier, unit)
            if unit.kind == "item_macro" and macro_types_named:
                # 条目级宏调用生成的类型算被点名：R-方法、R-隐式照连到写给它的 impl（文件头 Rust 闭包「条目级宏调用生成的条目」）
                for type_name in sorted(workspace.macro_declared_types.get(id(unit), ())):
                    name_type(type_name, identifier, unit)
            if unit.kind == "trait":
                for head in workspace.blanket_impls.get(unit.short, ()) if blanket else ():
                    push_impl(head, identifier, unit, f"blanket {unit.short}")
                if not no_trait_impls:
                    for head in workspace.implicit_impls.get(unit.short, ()):
                        push_impl(head, identifier, unit, f"trait-impl-of {unit.short}")
            for trait_name in (unit.dyn_traits or ()) if open_interior_dyn else ():
                dyn_trait(trait_name, identifier, unit)
            collect_data(unit, identifier)
            if modules_per_naming_unit and not raw_wgsl and unit.plain:
                # 弄坏开关 wgsl-modules-per-naming-unit：退回旧口径，按点名读 .wgsl 的常量的那个单元一组一组拼模块
                module_files = set()
                for name in unit.plain:
                    for visible_id in visible:
                        for candidate in workspace.by_name.get(visible_id, {}).get(name, ()):
                            if id(candidate) in workspace.wgsl_const_file and candidate.kind not in dropped:
                                module_files.add(workspace.wgsl_const_file[id(candidate)])
                if unit.kind == "const" and id(unit) in workspace.wgsl_const_file:
                    module_files.discard(workspace.wgsl_const_file[id(unit)])
                if module_files:
                    wgsl_modules.add(frozenset(module_files))

    def refuse_child(unit, index):
        rust_file = workspace.files[unit.file]
        line = rust_file.lines[index]
        closure.refusals.add(Refusal("current-exe", unit.file, line, rust_file.row(line)))

    def constant_string(context, name):
        """常量 name 的串值（crates/singlefs-unit-graph 的 constant_string_value）：先找 context 这个单元自己的词法单元里的 const <名>: … = "…";，
        再找同一个包、在这一次链接集里（同一包的别的 bin 不算）叫这个名字的常量单元（几个都得写成 = "…";、值相同）；找不到、写法不是串字面量或几个值不同回 None。
        弄坏开关 rust-current-exe-constants-same-file-only：第二步只找 context 所在的那一份文件。"""
        found = workspace.files[context.file].string_constant_in(context.first_token, context.last_token, name)
        if found is not None:
            return found
        context_crate = workspace.crate_of_path(context.file)
        values, searched = set(), set()
        for candidate_target in sorted(linked):
            if workspace.targets[candidate_target].kind == "doctest":
                continue
            for path in sorted(workspace.targets[candidate_target].files):
                if path in searched or workspace.crate_of_path(path) is not context_crate:
                    continue
                searched.add(path)
                if path != context.file and "rust-current-exe-constants-same-file-only" in breaks:
                    continue
                rust_file = workspace.files[path]
                for candidate in rust_file.units:
                    if candidate.kind == "const" and candidate.short == name:
                        value = rust_file.string_constant_in(candidate.first_token, candidate.last_token, name)
                        if value is None:
                            return None
                        values.add(value)
        return next(iter(values)) if len(values) == 1 else None

    def folded_test_name(context, pieces):
        """几段连成测试名：常量名在 context 这个单元里解（constant_string）；有一个解不出回 None。"""
        texts = []
        for piece_kind, text in pieces:
            if piece_kind == "literal":
                texts.append(text)
                continue
            value = constant_string(context, text)
            if value is None:
                return None
            texts.append(value)
        return "".join(texts)

    def child_tests_of(function_name, position, identifier):
        """名字是起子进程那个函数的参数：闭包里每一处调用它的那个实参各折一遍 → (对上的 #[test] 单元, 全都折得出又各对上恰好一个)；一处调用都没有也算不行。"""
        names, complete = set(), True
        for member in list(closure.order):
            if member.language != "rust" or member.first_token is None or member.kind == "doctest":
                continue
            found = workspace.files[member.file].call_site_test_names(member, function_name, position)
            if found is None:
                complete = False
                continue
            for pieces in found:
                name = folded_test_name(member, pieces)
                if name is None:
                    complete = False
                else:
                    names.add(name)
        tests = []
        for name in sorted(names):
            test = workspace.exact_test(name, identifier)
            if test is None:
                complete = False
            else:
                tests.append(test)
        return tests, complete and bool(names)

    def pull_in_child_process_tests():
        """参数写法的测试名（规则 current-exe-child）：闭包里每一处调用点交的实参对上的 #[test] 单元连进来；每趟 drain 之后、WGSL 供给之前跑。→ 有没有新推进去的。"""
        if "rust-current-exe-children-unlinked" in breaks:
            return False
        pushed = False
        for _unit, function_name, position, identifier, _index in pending_children:
            for test in child_tests_of(function_name, position, identifier)[0]:
                if (id(test), identifier) not in seen:
                    push_exact(test, identifier, _unit, "current-exe-child")
                    pushed = True
        return pushed

    def wgsl_module_groups():
        if raw_wgsl:
            return []
        if modules_per_naming_unit:
            return sorted(wgsl_modules, key=sorted)
        return [frozenset(wgsl_paths)] if wgsl_paths else []

    def pull_in_wgsl_name_suppliers():
        """着落做成边（crates/singlefs-unit-graph 的 pull_in_wgsl_name_suppliers）：闭包里还没有着落的 WGSL 外名，把链接集里、闭包外供它的单元
        （按名字定义了它或以 _ 加它收尾的常量、静态量、条目级宏调用；不是这一种的，串字面量里有它这个词的单元）连同它们的闭包并进来；
        每个只试一次。另补一条着落边：还没有着落的外名在链接集里别的单元 include_str! / include_bytes! 读进来的一份 .wgsl（还不在这个模块里）
        定义了（那份文件的顶层条目：fn、const、override、alias、var、struct），把读它的那个 Rust 单元连同它的闭包并进来，那份 .wgsl 随之进模块、
        名字在合起来的名字空间里解；拉不进的才拒算 wgsl-undefined-name。弄坏开关 wgsl-other-file-definitions-unlinked 下不补这一条。
        → 这一趟有没有新推进去的单元。"""
        groups = wgsl_module_groups()
        if not groups or not (pull_suppliers or pull_wgsl_definers):
            return False
        sources = WgslNameSources(workspace, closure.order)
        unsupplied = set()
        for module_files in groups:
            _members, refusals, _suppliers = wgsl_closure([wgsl_file(workspace.root, path) for path in sorted(module_files)], None, sources)
            unsupplied |= {refusal.snippet for refusal in refusals if refusal.form == "wgsl-undefined-name"}
        if not unsupplied:
            return False
        pushed = False
        for identifier in sorted(linked):
            if workspace.targets[identifier].kind == "doctest":
                continue
            for path in sorted(workspace.targets[identifier].files):
                rust_file = workspace.files[path]
                for candidate in rust_file.units:
                    if candidate.kind == "doctest" or id(candidate) in closure.members or id(candidate) in suppliers_tried:
                        continue
                    if pull_wgsl_definers and any(
                            kind == "wgsl" and value not in wgsl_paths and wgsl_top_level_names_in(workspace.root, value) & unsupplied
                            for kind, value, _index in candidate.data_references or ()):
                        suppliers_tried.add(id(candidate))
                        push(candidate, identifier, None, "wgsl-file-definer")
                        pushed = True
                        continue
                    if not pull_suppliers:
                        continue
                    if suppliers_skip_test_code and rust_unit_is_test_code(workspace, candidate, identifier):
                        continue
                    defines = candidate.kind in WGSL_NAME_DEFINING_KINDS and any(
                        name_and_underscore_suffixes(name) & unsupplied for name in rust_declared_names(candidate))
                    in_string = not defines and any(
                        rust_file.kinds[index] == LITERAL and rust_file.texts[index][:1] in ('"', "r")
                        and unsupplied & set(rust_string_literal_words(rust_file.texts[index]))
                        for index in range(candidate.first_token, candidate.last_token))
                    if defines or in_string:
                        suppliers_tried.add(id(candidate))
                        push(candidate, identifier, None, "wgsl-name-supplier")
                        pushed = True
        return pushed

    drain()
    while True:
        if pull_in_child_process_tests() or pull_in_wgsl_name_suppliers():
            drain()
            continue
        break
    for unit, function_name, position, identifier, index in pending_children:
        if not child_tests_of(function_name, position, identifier)[1]:
            refuse_child(unit, index)
    groups = wgsl_module_groups()
    if groups:
        sources = WgslNameSources(workspace, closure.order)
        for module_files in groups:
            wgsl_files = [wgsl_file(workspace.root, path) for path in sorted(module_files)]
            members, refusals, _suppliers = wgsl_closure(wgsl_files, None, sources)
            for member in members:
                closure.add(member, None, "wgsl-module")
            closure.refusals.update(refusals)
    return closure
