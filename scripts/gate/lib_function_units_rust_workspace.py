"""Rust 工作区：清单与目标的发现（discover_rust_crates、crate_targets）、cfg 求值、RustWorkspace（模块树、名字索引、闭包用的边）、文档测试入口与 Closure。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import collections
import glob
import itertools
import os
import posixpath
import re
import tomllib
from lib_function_units_lexer import IDENT, LITERAL, active_breaks
from lib_function_units_base import (
    FORMAL_VERIFICATION, FunctionUnitsError, JUDGMENT_KINDS, LINK_INSTRUCTION_NAMES, PRODUCT_BEHAVIOR, RUSTC_LINK_LIBRARY_INSTRUCTION, Refusal, Unit,
    UsageError, VERUS_GHOST_CODE_UNPARSED, files_under, link_instruction_pattern, snippet_of,
)
from lib_function_units_shell_index import shell_sourced_files
from lib_function_units_rust_file import (
    RUST_KEYWORDS, RUST_KINDS_DEFINING_NAMES, RUST_NAMED_KINDS, RUST_TYPE_KINDS, RUST_TYPE_OR_TRAIT_KINDS, RustFile, TEST_ATTRIBUTE_RAW,
    extern_block_attributes_name_link, use_path_leading_to, use_path_root,
)
from lib_function_units_rust_verus import product_view_agrees, verifier_attribute_token_ranges, verus_ghost_code_blanked
from lib_function_units_unit_graph import ProductViewUnavailable, product_view_dump


# ── Rust：包、目标、模块树、名字表 ───────────────────────────────────────────

KNOWN_CFG_NAMES = frozenset(("test", "debug_assertions", "unix", "windows", "doc", "doctest", "miri", "proc_macro", "overflow_checks",
                             "panic", "target_arch", "target_os", "target_family", "target_env", "target_endian", "target_pointer_width",
                             "target_vendor", "target_has_atomic", "target_abi", "target_feature", "feature", "relocation_model", "ub_checks",
                             "fmt_debug", "contract_checks"))
# x86_64-unknown-linux-gnu 那一行（与 crates/singlefs-unit-graph 的 TARGET_PLATFORMS 同一份，target_abi 是空串）加上恒开的 debug_assertions
DEFAULT_RUST_CFG = ("debug_assertions", "unix", 'target_os="linux"', 'target_family="unix"', 'target_arch="x86_64"',
                    'target_pointer_width="64"', 'target_endian="little"', 'target_env="gnu"', 'target_vendor="unknown"', 'target_abi=""', 'panic="unwind"',
                    'target_has_atomic="8"', 'target_has_atomic="16"', 'target_has_atomic="32"', 'target_has_atomic="64"',
                    'target_has_atomic="ptr"')
TARGET_PREFERENCE = ("lib", "bin", "test", "bench", "example", "build")
# 输出行限定名前缀取「头一个目标」时的目标次序（crates/singlefs-unit-graph 的 discover：lib、bin、test、example、bench、build）
RUST_TARGET_ORDER = {"lib": 0, "bin": 1, "test": 2, "example": 3, "bench": 4, "build": 5}


def load_manifest(root, path):
    try:
        with open(os.path.join(root, path), "rb") as handle:
            return tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise FunctionUnitsError(f"读不出 {path}：{error}") from error


class RustDependency:
    __slots__ = ("name", "directory", "features", "default_features")

    def __init__(self, name, directory, features, default_features):
        self.name, self.directory, self.features, self.default_features = name, directory, features, default_features


class RustCrate:
    def __init__(self, root, directory, manifest, workspace_dependencies):
        self.directory = directory
        package = manifest.get("package", {})
        self.package = package.get("name", posixpath.basename(directory) or "root")
        self.label = posixpath.basename(directory) if directory else self.package
        lib = manifest.get("lib", {})
        self.lib_name = lib.get("name", self.package.replace("-", "_"))
        self.proc_macro = bool(lib.get("proc-macro") or lib.get("proc_macro"))
        self.features = {name: list(items) for name, items in manifest.get("features", {}).items()}
        self.manifest = manifest
        self.outside_workspace = False   # discover_rust_crates 顺 path 依赖收进来的工作区外 crate 设真（文件头 Rust 闭包「工作区外的 path 依赖」）
        self.dependencies = {"normal": [], "dev": [], "build": []}
        tables = [manifest] + list(manifest.get("target", {}).values())
        for section, key in (("dependencies", "normal"), ("dev-dependencies", "dev"), ("build-dependencies", "build")):
            for table in tables:
                for name, specification in table.get(section, {}).items():
                    self.dependencies[key].append(self.dependency(root, name, specification, workspace_dependencies))
        self.check_cfg = set()
        lints = manifest.get("lints", {}).get("rust", {}).get("unexpected_cfgs", {})
        if isinstance(lints, dict):
            for entry in lints.get("check-cfg", []):
                found = re.match(r"^cfg\(([A-Za-z_][A-Za-z0-9_]*)", entry)
                if found:
                    self.check_cfg.add(found.group(1))

    def dependency(self, root, name, specification, workspace_dependencies):
        directory, features, default_features = None, [], True
        if isinstance(specification, dict):
            features = list(specification.get("features", []))
            default_features = specification.get("default-features", specification.get("default_features", True))
            if "path" in specification:
                directory = posixpath.normpath(posixpath.join(self.directory or ".", specification["path"]))
            elif specification.get("workspace"):
                shared = workspace_dependencies.get(name, {})
                if isinstance(shared, dict) and "path" in shared:
                    directory = posixpath.normpath(shared["path"])
                    features += list(shared.get("features", []))
        return RustDependency(name.replace("-", "_"), directory, features, default_features)


class RustTarget:
    def __init__(self, crate, kind, name, root_file):
        self.crate, self.kind, self.name, self.root_file = crate, kind, name, root_file
        self.id = f"{crate.label}:{kind}:{name}"
        self.files = {}       # 文件 → (模块路径, 那份文件的 cfg 条件链)
        self.declared_by = collections.defaultdict(list)    # 文件 → 这个目标里把它连进模块树的每一条 mod 声明单元（R-结构）
        self.refusals = []


def multiline_basic_strings(text):
    """TOML 原文里每一个多行基础串（三个双引号）定界符之间的原文；注释、单行串、多行字面串里的不算；结尾定界符前最多两个引号归内容。"""
    found, position, length = [], 0, len(text)
    while position < length:
        if text[position] == "#":
            position = text.find("\n", position)
            position = length if position < 0 else position
        elif text.startswith('"""', position):
            start, cursor = position + 3, position + 3
            while cursor < length and not text.startswith('"""', cursor):
                cursor += 2 if text[cursor] == "\\" else 1
            quotes = 0
            while text.startswith('"', cursor + quotes) and quotes < 5:
                quotes += 1
            found.append(text[start:cursor + quotes - 3])
            position = cursor + quotes
        elif text.startswith("'''", position):
            end = text.find("'''", position + 3)
            position = length if end < 0 else end + 3
        elif text[position] in "\"'":
            quote, cursor = text[position], position + 1
            while cursor < length and text[cursor] not in (quote, "\n"):
                cursor += 2 if quote == '"' and text[cursor] == "\\" else 1
            position = cursor + 1
        else:
            position += 1
    return found


def members_in_multiline_basic_strings_with_a_backslash(text, members):
    """members 里写成了含反斜杠的多行基础串的成员（原文有反斜杠、读出来的值在 members 里）。"""
    offending = []
    for content in multiline_basic_strings(text):
        if "\\" not in content:
            continue
        try:
            value = tomllib.loads('x = """' + content + '"""')["x"]
        except (tomllib.TOMLDecodeError, KeyError):
            continue
        if value in members:
            offending.append(value)
    return offending


def discover_rust_crates(root):
    manifest = load_manifest(root, "Cargo.toml") if os.path.isfile(os.path.join(root, "Cargo.toml")) else {}
    workspace = manifest.get("workspace")
    if workspace and "rust-workspace-multiline-backslash-read" not in active_breaks():
        # 多行基础串里有反斜杠的成员写法两份实现都报读不出（crates/singlefs-unit-graph 的 workspace.rs 的手写读法读不了）
        with open(os.path.join(root, "Cargo.toml"), encoding="utf-8", errors="replace") as handle:
            offending = members_in_multiline_basic_strings_with_a_backslash(handle.read(), workspace.get("members", []))
        if offending:
            raise FunctionUnitsError(f"读不出 Cargo.toml：[workspace] members 里的 {offending} 写成了含反斜杠的多行基础串（三个双引号），这一种写法不读，改成单行串")
    directories = []
    if workspace:
        excluded = {posixpath.normpath(path) for path in workspace.get("exclude", [])}
        # 仓根路径当字面（路径里的 [ ] * ? 不是通配）；成员写 "." 与根目录是同一个，收成 ""（根清单有 [package] 时它也是 ""）
        pattern_root = root if "rust-workspace-members-glob-unescaped" in active_breaks() else glob.escape(root)
        for pattern in workspace.get("members", []):
            for match in sorted(glob.glob(os.path.join(pattern_root, pattern))):
                relative = os.path.relpath(match, root).replace(os.sep, "/")
                if relative == "." and "rust-workspace-dot-member-unnormalized" not in active_breaks():
                    relative = ""
                if relative not in excluded and os.path.isfile(os.path.join(match, "Cargo.toml")):
                    directories.append(relative)
    elif os.path.isdir(os.path.join(root, "crates")):
        for name in sorted(os.listdir(os.path.join(root, "crates"))):
            if os.path.isfile(os.path.join(root, "crates", name, "Cargo.toml")):
                directories.append(f"crates/{name}")
    if "package" in manifest:
        directories.insert(0, "")
    shared = workspace.get("dependencies", {}) if workspace else {}
    crates, pending = [], list(dict.fromkeys(directories))
    known = set(pending)
    workspace_members = set(pending)
    follow_path_dependencies = "rust-excluded-path-dependencies-dropped" not in active_breaks()
    while pending:
        directory = pending.pop(0)
        crate_manifest = manifest if directory == "" else load_manifest(root, posixpath.join(directory, "Cargo.toml"))
        crate = RustCrate(root, directory, crate_manifest, shared)
        crate.outside_workspace = directory not in workspace_members
        crates.append(crate)
        # 工作区外的 path 依赖（根清单 exclude 里、或没列进 members、落在仓根底下的；文件头 Rust 闭包「工作区外的 path 依赖」）：顺依赖边收成一个 crate，
        # 调到它的入口照连；落在仓根外的不收。弄坏开关 rust-excluded-path-dependencies-dropped：不收
        for dependency in (item for section in ("normal", "dev", "build") for item in crate.dependencies[section]) if follow_path_dependencies else ():
            place = dependency.directory
            if (place is None or place in known or place in (".", "..") or place.startswith("../") or posixpath.isabs(place)
                    or not os.path.isfile(os.path.join(root, place, "Cargo.toml"))):
                continue
            known.add(place)
            pending.append(place)
    # 两个成员目录同名（crates/common 与 tools/common）时目标号用目录全路径：目标号是「标签:种类:名」，标签只取目录名会让两个包的同名测试目标撞成一个
    label_counts = collections.Counter(crate.label for crate in crates)
    if "rust-same-named-member-directories-merged" not in active_breaks():
        for crate in crates:
            if label_counts[crate.label] > 1:
                crate.label = crate.directory or "<root>"
    return crates


def crate_targets(root, crate):
    def place(*parts):
        return posixpath.normpath(posixpath.join(crate.directory or ".", *parts))

    def exists(path):
        return os.path.isfile(os.path.join(root, path))

    def listed(directory, single_files=True):
        found = []
        absolute = os.path.join(root, place(directory))
        if not os.path.isdir(absolute):
            return found
        for name in sorted(os.listdir(absolute)):
            if single_files and name.endswith(".rs") and os.path.isfile(os.path.join(absolute, name)):
                found.append((name[:-3], place(directory, name)))
            elif os.path.isfile(os.path.join(absolute, name, "main.rs")):
                found.append((name, place(directory, name, "main.rs")))
        return found

    targets, seen = [], set()

    def add(kind, name, path):
        if (kind, name) not in seen and exists(path):
            seen.add((kind, name))
            targets.append(RustTarget(crate, kind, name, path))

    manifest = crate.manifest
    lib = manifest.get("lib", {})
    if "package" in manifest or not crate.directory:
        add("lib", crate.lib_name, place(lib["path"]) if "path" in lib else place("src", "lib.rs"))
    for kind, section, directory in (("bin", "bin", "src/bin"), ("test", "test", "tests"), ("bench", "bench", "benches"), ("example", "example", "examples")):
        for entry in manifest.get(section, []):
            name = entry.get("name")
            path = place(entry["path"]) if "path" in entry else (place("src", "main.rs") if kind == "bin" and name == crate.package else place(directory, f"{name}.rs"))
            add(kind, name, path)
        if kind == "bin":
            add("bin", crate.package, place("src", "main.rs"))
        for name, path in listed(directory):
            add(kind, name, path)
    build = manifest.get("package", {}).get("build")
    if isinstance(build, str):
        add("build", crate.label, place(build))
    elif build is not False:
        add("build", crate.label, place("build.rs"))
    return targets


class CfgEvaluator:
    def __init__(self, names, pairs, features_by_crate, test_targets, known_extra):
        self.names, self.pairs, self.features_by_crate = names, pairs, features_by_crate
        self.test_targets, self.known_extra = test_targets, known_extra

    def value(self, tree, crate_directory, target_id):
        kind = tree[0]
        if kind == "all":
            return all(self.value(item, crate_directory, target_id) for item in tree[1])
        if kind == "any":
            return any(self.value(item, crate_directory, target_id) for item in tree[1])
        if kind == "not":
            return not self.value(tree[1][0], crate_directory, target_id)
        if kind == "name":
            name = tree[1]
            if name == "test":
                return target_id in self.test_targets
            if name in self.names:
                return True
            if name in KNOWN_CFG_NAMES or name in self.known_extra:
                return False
            raise ValueError(name)
        key, value = tree[1], tree[2]
        if key == "feature":
            return value in self.features_by_crate.get(crate_directory, ())
        if (key, value) in self.pairs:
            return True
        if key in KNOWN_CFG_NAMES or key in self.known_extra:
            return False
        raise ValueError(key)


def cfg_entries(entries):
    names, pairs = set(), set()
    for entry in entries:
        found = re.match(r'^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"(.*)"$', entry)
        if found:
            pairs.add((found.group(1), found.group(2)))
        elif re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", entry):
            names.add(entry)
        else:
            raise UsageError(f"认不出的 cfg：{entry}（写 <名> 或 <名>=\"<值>\"）")
    return names, pairs


def condition_mentions_test(tree):
    if tree is None:
        return False
    if tree[0] == "name":
        return tree[1] == "test"
    if tree[0] in ("all", "any"):
        return any(condition_mentions_test(item) for item in tree[1])
    return False


class RustWorkspace:
    def __init__(self, root, judgment_kind=FORMAL_VERIFICATION, product_dump=None):
        if not os.path.isdir(root):
            raise FunctionUnitsError(f"根不在：{root}")
        if judgment_kind not in JUDGMENT_KINDS:
            raise UsageError(f"判定种类只认 {'、'.join(JUDGMENT_KINDS)}：{judgment_kind}")
        self.root = root
        self.breaks = active_breaks()
        # 产品行为视图（文件头「判定种类」）：product_dump 是调用方交的 unit-graph dump --judgment-kind product-behavior 的行（对拍格与自证用），
        # 不给就要用时起 unit-graph；product_view 是按文件分好的那一份（{文件: ([单元行], [拒算行])}），拿不到时是一句原因
        self.judgment_kind, self.product_dump, self.product_view = judgment_kind, product_dump, None
        self.crates = discover_rust_crates(root)
        self.crate_by_directory = {crate.directory: crate for crate in self.crates}
        self.files = {}
        self.targets = {}
        self.names_assigned = False
        self.links_beyond_system_library = {}   # (入口目标 id, 块所在的包目录) → target_may_link_libraries_beyond_the_system_library 的结果
        self.packages_adding_libraries = {}   # (包目录, 是不是入口所在的包, 是不是被当构建依赖, 入口目标 id 或 None) → package_adds_libraries 的结果
        self.shell_sources = {}   # 读到的 shell 脚本 → shell_sourced_files 的结果（shell_sourced）
        for crate in self.crates:
            for target in crate_targets(root, crate):
                self.targets[target.id] = target
                directory = posixpath.dirname(target.root_file)
                self.walk(target, target.root_file, [], [], directory)
        for path in sorted(self.files):
            self.assign_full_names(self.files[path])
        self.names_assigned = True
        self.index()
        for path in sorted(self.files):
            self.apply_product_view(self.files[path])

    def crates_reached_from(self, starts):
        """starts 里的包（名单块所在的包、入口所在的包）顺 normal、dev、build 三种依赖够到的全部仓内包（含 starts 自己；工作区外的 path 依赖也算）。
        registry、git 来的依赖看不到，package_adds_libraries 的外部依赖那一条另管。
        弄坏开关 rust-extern-dependency-dev-dependencies-unfollowed 下 starts 之外的包只跟 normal 与 build 依赖。"""
        found = {crate.directory: crate for crate in starts}
        todo = list(found.values())
        while todo:
            current = todo.pop()
            narrowed = "rust-extern-dependency-dev-dependencies-unfollowed" in self.breaks and current not in starts
            for section in ("normal", "build") if narrowed else ("normal", "dev", "build"):
                for dependency in current.dependencies[section]:
                    linked = self.crate_by_directory.get(dependency.directory)
                    if linked is not None and linked.directory not in found:
                        found[linked.directory] = linked
                        todo.append(linked)
        return list(found.values())

    def package_adds_libraries(self, package, own_target, used_as_build_dependency):
        """包 package（链进 own_target 的产物的那些包之一）可能多带 C 库之外的库（crates/singlefs-unit-graph 的 package_adds_libraries 同一个判法）：
        块带 link：它任一目标（库、测试、示例、构建脚本各目标）里有 extern 块的外属性带 link（属性里任何位置；不管进不进闭包：link 对整个产物生效）；
        构建脚本打指令：它的构建脚本目标里任一份文件的字面串写出 cargo:rustc-link-lib、cargo:rustc-link-arg、cargo:rustc-flags（含 cargo:: 写法；rustc-flags 不分 -l 与 -L；
        注释里提到、没有 cargo: 前缀的裸词不算，link_instruction_pattern）；
        库代码打指令：它被产物链进来的某个包当构建依赖（used_as_build_dependency）时，它的库目标里有同样的字面串（构建依赖的库代码替构建脚本打指令）；
        外部依赖：它有工作区外的依赖（普通、开发、构建任一种：registry、git 或仓外的 path 来的，cc、pkg-config 这一类替构建脚本编 C 代码、找库的，指令字面量在它们里面，看不到，宁可拒算）。
        弄坏开关 rust-extern-other-block-link-unchecked 下不看块带 link、rust-extern-link-of-other-targets-unchecked 下块带 link 只看 own_target 与别的包的库目标、
        rust-extern-build-script-link-unchecked 下不看构建脚本打指令、rust-extern-link-arguments-unchecked 下构建脚本打指令与库代码打指令只认 rustc-link-lib、
        rust-extern-two-colon-instructions-ignored 下只认 cargo: 一个冒号的写法、rust-extern-instruction-words-bare 下构建脚本与库代码里指令名裸词（注释里的、没有 cargo: 前缀的）也算、
        rust-extern-library-literals-of-any-package 下别的包（不是 own_target 所在的包）的库目标都看、不只看被当构建依赖的、
        rust-extern-external-dependencies-unchecked 下不看外部依赖；rust-extern-link-of-dependencies-unchecked 下 target_may_link_libraries_beyond_the_system_library 只看入口所在的包。"""
        is_own = package is own_target.crate
        key = (package.directory, is_own, used_as_build_dependency, own_target.id if is_own and "rust-extern-link-of-other-targets-unchecked" in self.breaks else None)
        cached = self.packages_adding_libraries.get(key)
        if cached is not None:
            return cached
        breaks = self.breaks
        package_targets = [target for target in self.targets.values() if target.crate is package]
        build_scripts = [target for target in package_targets if target.kind == "build"]
        if "rust-extern-link-of-other-targets-unchecked" in breaks:
            scanned_for_link = [own_target] if is_own else [target for target in package_targets if target.kind == "lib"]
        else:
            scanned_for_link = package_targets
        found = False
        if "rust-extern-other-block-link-unchecked" not in breaks:
            found = any(unit.kind == "extern_block" and extern_block_attributes_name_link(rust_file.kinds, rust_file.texts, unit.first_token, unit.last_token)
                        for target in scanned_for_link for rust_file in (self.files[path] for path in target.files) for unit in rust_file.units)
        if not found and "rust-extern-build-script-link-unchecked" not in breaks:
            found = any(self.file_prints_link_instruction(self.files[path]) for target in build_scripts for path in target.files)
        if not found and (used_as_build_dependency or ("rust-extern-library-literals-of-any-package" in breaks and not is_own)):
            found = any(self.file_prints_link_instruction(self.files[path]) for target in package_targets if target.kind == "lib" for path in target.files)
        if not found and "rust-extern-external-dependencies-unchecked" not in breaks:
            found = any(dependency.directory not in self.crate_by_directory for section in ("normal", "dev", "build") for dependency in package.dependencies[section])
        self.packages_adding_libraries[key] = found
        return found

    def file_prints_link_instruction(self, rust_file):
        """这份文件有字面串写出 cargo:rustc-link-lib、cargo:rustc-link-arg、cargo:rustc-flags 指令（含 cargo:: 写法）：字面串词法单元里含 link_instruction_pattern，
        注释里的不算；弄坏开关 rust-extern-instruction-words-bare 下改成原文里任何位置含指令名裸词。"""
        if "rust-extern-instruction-words-bare" in self.breaks:
            names = (RUSTC_LINK_LIBRARY_INSTRUCTION,) if "rust-extern-link-arguments-unchecked" in self.breaks else LINK_INSTRUCTION_NAMES
            text = "\n".join(rust_file.source_rows)
            return any(name in text for name in names)
        pattern = link_instruction_pattern(self.breaks)
        return "cargo:" in "\n".join(rust_file.source_rows) and any(kind == LITERAL and pattern.search(text) for kind, text in zip(rust_file.kinds, rust_file.texts))

    def target_may_link_libraries_beyond_the_system_library(self, target_id, block_target_id):
        """入口目标 target_id、名单里的块所在的目标 block_target_id 的产物链接时可能多出 C 库之外的库（crates/singlefs-unit-graph 的
        target_may_link_libraries_beyond_the_system_library 同一个判法）：从入口所在的包与块所在的包出发，顺依赖（普通、开发、构建三种，crates_reached_from）
        够到的全部仓内包里有一个包 package_adds_libraries。链接对最终产物做：名单里的块在哪个目标、哪个包里不论，同一个包的任一目标、依赖（含开发、构建依赖）带来的库都进同一次链接。
        被当构建依赖的包（有包的构建依赖指到它）单列出来交给 package_adds_libraries。"""
        own_target, block_target = self.targets[target_id], self.targets[block_target_id]
        key = (target_id, block_target.crate.directory)
        if key not in self.links_beyond_system_library:
            if "rust-extern-link-of-dependencies-unchecked" in self.breaks:
                packages = [own_target.crate]
            else:
                packages = self.crates_reached_from([own_target.crate, block_target.crate])
            build_dependencies = {dependency.directory for package in packages for dependency in package.dependencies["build"]}
            self.links_beyond_system_library[key] = any(self.package_adds_libraries(package, own_target, package.directory in build_dependencies) for package in packages)
        return self.links_beyond_system_library[key]

    def shell_sourced(self, path):
        """shell_sourced_files(根, path)，一个工作区里一份脚本只折一趟（几十个入口读同一份脚本）。"""
        if path not in self.shell_sources:
            self.shell_sources[path] = shell_sourced_files(self.root, path)
        return self.shell_sources[path]

    def product_view_lines(self):
        """Rust 那一份产品行为视图的 dump，按文件分好：{文件: ([单元行], [拒算行])}；拿不到交一句原因。一个工作区只取一次。"""
        if self.product_view is None:
            lines = self.product_dump
            if lines is None:
                try:
                    lines = product_view_dump(self.root)
                except ProductViewUnavailable as error:
                    self.product_view = str(error)
                    return self.product_view
            grouped = {}
            for line in lines:
                fields = line.split("\t")
                if len(fields) < 3 or fields[0] in ("data", "data-directory"):
                    continue
                place = grouped.setdefault(fields[2].rpartition(":")[0], ([], []))
                if fields[0] == "refused":
                    place[1].append(fields)
                elif len(fields) == 4:
                    place[0].append(line)
            self.product_view = grouped
        return self.product_view

    def product_view_file(self, rust_file, text, directory):
        """产品行为视图里有 verus! 块、或写着 #[verifier::…] / #[verifier(…)] 外属性的文件，载入时先照 Rust 那一份的剥法在这里剥一遍（verus_ghost_code_blanked）、
        重切：重切出来的单元与 Rust 那一份交的这份文件的单元逐个核得上（product_view_agrees）就用剥过的这一份，闭包的边、cfg、数据与文档测试都从剥过的词法单元取
        （与 unit-graph 剥完再切、再收闭包同一做法）；核不上、Rust 那一份解不开这份文件、拿不到 Rust 那一份的，照原样那一份交给 apply_product_view（文件头「判定种类」）。
        弄坏开关 product-view-edges-from-the-written-tokens 下一律照原样那一份；product-view-blanking-unverified 下剥过的那一份不核就用。"""
        breaks = self.breaks
        if (self.judgment_kind != PRODUCT_BEHAVIOR or not (rust_file.verus_spans or verifier_attribute_token_ranges(rust_file.kinds, rust_file.texts))
                or breaks & {"product-view-kept-formal", "product-view-strips-executable-code", "product-view-edges-from-the-written-tokens"}):
            return rust_file
        blanked_text = verus_ghost_code_blanked(text)
        if blanked_text is None or blanked_text == text:
            return rust_file
        blanked = RustFile(self.root, rust_file.path, blanked_text, directory)
        if "product-view-blanking-unverified" in breaks:
            return blanked
        view = self.product_view_lines()
        if isinstance(view, str):
            return rust_file
        theirs_lines, theirs_refusals = view.get(rust_file.path, ([], []))
        if any(fields[1] == VERUS_GHOST_CODE_UNPARSED for fields in theirs_refusals):
            return rust_file
        return blanked if product_view_agrees(blanked, theirs_lines) else rust_file

    def apply_product_view(self, rust_file):
        """产品行为视图（文件头「判定种类」）：有 verus! 块、或写着 #[verifier::…] / #[verifier(…)] 外属性的文件照 Rust 那一份换指纹、认只给 Verus 看的单元、
        记配不上的拒算。弄坏开关 product-view-kept-formal 下照原样（不剥）；product-view-strips-executable-code 下 verus! 块里的单元一律当只给 Verus 看的（剥到执行代码）。"""
        attribute_heads = [hash_index for hash_index, _close in verifier_attribute_token_ranges(rust_file.kinds, rust_file.texts)]
        if self.judgment_kind != PRODUCT_BEHAVIOR or not (rust_file.verus_spans or attribute_heads) or "product-view-kept-formal" in self.breaks:
            return
        if "product-view-strips-executable-code" in self.breaks:
            for unit in rust_file.units:
                if rust_file.inside_verus(unit):
                    unit.ghost_only = True
            return
        view = self.product_view_lines()
        block_line = rust_file.lines[rust_file.verus_spans[0][0] if rust_file.verus_spans else attribute_heads[0]]
        if isinstance(view, str):
            rust_file.refusals.append(Refusal("product-view-unavailable", rust_file.path, block_line, snippet_of(view)))
            return
        theirs_lines, theirs_refusals = view.get(rust_file.path, ([], []))
        for fields in theirs_refusals:
            if fields[1] == VERUS_GHOST_CODE_UNPARSED:
                at = fields[2].rpartition(":")[2]
                rust_file.refusals.append(Refusal(VERUS_GHOST_CODE_UNPARSED, rust_file.path, int(at) if at.isdigit() else block_line,
                                                  fields[3] if len(fields) > 3 else ""))
        theirs, ours = {}, {}
        for line in theirs_lines:
            placed, _tab, fingerprint = line.rpartition("\t")
            theirs.setdefault(placed, []).append(fingerprint)
        for unit in rust_file.units:
            ours.setdefault(unit.dump_text().rpartition("\t")[0], []).append(unit)
        for placed in sorted(set(theirs) | set(ours)):
            fingerprints, units = theirs.get(placed, []), ours.get(placed, [])
            if len(fingerprints) == len(units) and len(set(fingerprints)) <= 1:
                for unit, fingerprint in zip(units, fingerprints):
                    unit.fingerprint = fingerprint
                continue
            if not fingerprints and all(rust_file.inside_verus(unit) for unit in units):
                for unit in units:
                    unit.ghost_only = True
                continue
            line = units[0].first_line if units else block_line
            rust_file.refusals.append(Refusal("product-view-unmatched", rust_file.path, line,
                                              snippet_of(f"{placed}：unit-graph 交 {len(fingerprints)} 个、这一份切出 {len(units)} 个")))

    def file_prefix(self, path):
        """一份文件里单元限定名的前缀（照 crates/singlefs-unit-graph 的 module_prefix_of_file）：crate 名、这份文件所属的头一个目标的标记
        （目标次序照那一份：crate 按目录排，crate 里 lib、bin、test、example、bench、build，同种按名字与根文件排；lib 不写标记）、
        它在那个目标里的模块路径；哪个目标都不属于的写 <unlinked 路径>。"""
        crate = self.crate_of_path(path)
        prefix = [crate.lib_name] if crate is not None else []
        owners = [target for target in self.targets.values() if target.kind != "doctest" and path in target.files]
        if not owners:
            return prefix + [f"<unlinked {path}>"]
        first = min(owners, key=lambda target: (target.crate.directory, RUST_TARGET_ORDER.get(target.kind, len(RUST_TARGET_ORDER)),
                                                target.name, target.root_file))
        marker = {"bin": f"<bin {first.name}>", "test": f"<test {first.name}>", "example": f"<example {first.name}>",
                  "bench": f"<bench {first.name}>", "build": "<build>"}.get(first.kind)
        return prefix + ([marker] if marker else []) + list(first.files[path][0])

    def assign_full_names(self, rust_file):
        """单元的 full（输出行里的限定名）= 前缀 + 文件里的嵌套路径；文档测试挂条目的接在条目的 full 后面，挂模块的接在前缀加模块路径后面。"""
        prefix = self.file_prefix(rust_file.path)
        for unit in rust_file.units:
            if unit.kind != "doctest":
                unit.full = "::".join(prefix + [unit.name])
        for unit in rust_file.units:
            if unit.kind == "doctest":
                tail = unit.name.rsplit("::", 1)[-1]
                if unit.container is not None:
                    unit.full = f"{unit.container.full}::{tail}"
                else:
                    unit.full = "::".join(prefix + list(unit.doc_module or ()) + [tail])

    def crate_of_path(self, path):
        best = None
        for crate in self.crates:
            prefix = crate.directory + "/" if crate.directory else ""
            if path.startswith(prefix) and (best is None or len(crate.directory) > len(best.directory)):
                best = crate
        return best

    def file(self, path):
        if path not in self.files:
            crate = self.crate_of_path(path)
            with open(os.path.join(self.root, path), encoding="utf-8", errors="surrogateescape") as handle:
                text = handle.read()
            directory = crate.directory if crate else posixpath.dirname(path)
            self.files[path] = self.product_view_file(RustFile(self.root, path, text, directory), text, directory)
            if self.names_assigned:
                self.assign_full_names(self.files[path])
                self.apply_product_view(self.files[path])
        return self.files[path]

    def all_files(self):
        for crate in self.crates:
            for path in files_under(self.root, (".rs",), [crate.directory or "."]):
                if crate.directory == "" and any(path.startswith(other.directory + "/") for other in self.crates if other.directory):
                    continue
                self.file(path)
        return [self.files[path] for path in sorted(self.files)]

    def walk(self, target, path, module_path, chain, child_directory, declaring=None):
        if declaring is not None and declaring not in target.declared_by[path]:
            target.declared_by[path].append(declaring)
        if path in target.files:
            return
        rust_file = self.file(path)
        target.files[path] = (tuple(module_path), list(chain))
        for unit, name, path_attribute, inline in rust_file.mod_declarations:
            conditions = chain + unit.conditions
            if path_attribute is not None:
                candidate = posixpath.normpath(posixpath.join(posixpath.dirname(path), *inline, path_attribute))
                candidates = [candidate]
            else:
                base = posixpath.join(child_directory, *inline)
                candidates = [posixpath.normpath(posixpath.join(base, f"{name}.rs")), posixpath.normpath(posixpath.join(base, name, "mod.rs"))]
            found = next((candidate for candidate in candidates if os.path.isfile(os.path.join(self.root, candidate))), None)
            if found is None:
                target.refusals.append(Refusal("module-file-missing", path, unit.first_line, rust_file.row(unit.first_line)))
                continue
            if path_attribute is not None or found.endswith("/mod.rs"):
                next_directory = posixpath.dirname(found)
            else:
                next_directory = posixpath.normpath(posixpath.join(child_directory, *inline, name))
            self.walk(target, found, list(module_path) + list(inline) + [name], conditions, next_directory, unit)

    def index(self):
        self.unit_targets = collections.defaultdict(set)
        self.by_name = {}
        self.uses_by_scope = collections.defaultdict(list)
        self.item_macros_by_scope = collections.defaultdict(list)
        self.members_by_name = collections.defaultdict(list)
        self.members_of = collections.defaultdict(list)
        self.implicit_impls = collections.defaultdict(list)
        self.inherent_impls = collections.defaultdict(list)
        self.impls_of_trait = collections.defaultdict(list)
        self.blanket_impls = collections.defaultdict(list)
        self.wgsl_const_file = {}
        # 用到时才算（names_reaching_outside_crate、candidates_seen_from_module）
        self.outside_crate_reaching_names, self.outside_crate_module_candidates, self.outside_crate_type_names = {}, {}, {}
        self.index_macro_and_foreign_edges()
        indexed = set()
        # const _ / static _ 没有名字：R-裸名的着落里不收 _（let _ = …、_ => 里的 _ 不点它；条目级宏调用、extern 块定义的名字里的 _ 同样不收）；
        #   弄坏开关 rust-underscore-items-named 下照收
        underscore_named = "rust-underscore-items-named" in active_breaks()
        literal_names_linked = "rust-macro-rules-literal-names-unlinked" not in active_breaks()
        for target in self.targets.values():
            names = self.by_name.setdefault(target.id, collections.defaultdict(list))
            for path in target.files:
                for unit in self.files[path].units:
                    if unit.kind == "doctest":
                        continue
                    self.unit_targets[id(unit)].add(target.id)
                    if unit.container is None and unit.kind in RUST_NAMED_KINDS and (unit.short != "_" or underscore_named):
                        names[unit.short].append(unit)
                    if unit.kind in RUST_KINDS_DEFINING_NAMES:
                        for name in unit.defines:
                            if name != "_" or underscore_named:
                                names[name].append(unit)
                    if unit.kind == "item_macro" and literal_names_linked:
                        # 宏体里按字面写出的名字登到这次调用名下（文件头 Rust 闭包「条目级宏调用生成的条目」）；调用体里已有的不重登
                        for name in sorted(self.macro_rules_literal_names.get(unit.short.partition("!@")[0], frozenset()) - set(unit.defines or ())):
                            names[name].append(unit)
            for path in target.files:
                if path in indexed:
                    continue
                indexed.add(path)
                for unit in self.files[path].units:
                    if unit.kind == "doctest":
                        continue
                    if unit.kind == "use":
                        self.uses_by_scope[(unit.file, unit.scope)].append(unit)
                    if unit.kind == "item_macro" and unit.container is None:
                        self.item_macros_by_scope[(unit.file, unit.scope)].append(unit)
                    if unit.container is not None:
                        self.members_by_name[unit.short].append(unit)
                        self.members_of[id(unit.container)].append(unit)
                    if unit.kind == "impl":
                        if unit.is_trait_impl:
                            for key in unit.implicit_keys:
                                self.implicit_impls[key].append(unit)
                            for key in unit.open_keys or ():
                                self.impls_of_trait[key].append(unit)
                            if unit.trait_name and unit.blanket:
                                self.blanket_impls[unit.trait_name].append(unit)
                        else:
                            for key in unit.self_types:
                                self.inherent_impls[key].append(unit)
                    if unit.kind == "const":
                        wgsl = [reference[1] for reference in unit.data_references if reference[0] == "wgsl"]
                        if wgsl:
                            self.wgsl_const_file[id(unit)] = wgsl[0]

    def index_macro_and_foreign_edges(self):
        """文件头 Rust 闭包「条目级宏调用生成的条目」「#[global_allocator]」「外部类型上的 trait impl」那三条要的索引，扫进了目标的每份文件一趟：
          macro_declared_types   条目级宏调用（id）→ 调用体里跟在 enum / struct / union / type 后面的名字（宏生成的类型）
          macro_rules_literal_names  宏名 → 同名 macro_rules! 体里展开式顶层跟在 const / static / fn / struct / enum / union / type 后面、按字面写出的名字
                                 （不是 $ 起头的捕获）
          global_allocators      带 #[global_allocator] 的 static 单元
          foreign_trait_impls    自身类型是外部类型、trait 是本仓 trait 的 trait impl（id）：自身类型写成路径、头一段不是 crate / self / super / Self 与本仓
                                 crate 名的，或自身类型里的名字没有一个是本仓定义的类型（struct、enum、union、type 与宏生成的类型）"""
        self.macro_declared_types, self.macro_rules_literal_names = {}, collections.defaultdict(set)
        self.global_allocators, self.foreign_trait_impls = [], set()
        paths = sorted({path for target in self.targets.values() for path in target.files})
        type_names, trait_names, impls = set(), set(), []
        for path in paths:
            rust_file = self.files[path]
            for unit in rust_file.units:
                if unit.first_token is None or unit.kind == "doctest":
                    continue
                span = range(unit.first_token, unit.last_token)
                if unit.kind == "item_macro":
                    declared = frozenset(rust_file.t(index + 1) for index in span if rust_file.t(index) in ("enum", "struct", "union", "type")
                                         and rust_file.kind(index + 1) == IDENT and rust_file.t(index + 1) not in RUST_KEYWORDS)
                    self.macro_declared_types[id(unit)] = declared
                    type_names |= declared
                elif unit.kind == "macro_rules":
                    depth = 0
                    for index in span:
                        word = rust_file.t(index)
                        depth += (word in ("{", "(", "[")) - (word in ("}", ")", "]"))
                        # 只收展开结果顶层的条目（宏体的括号一层、展开式的括号一层）：impl $name { fn new … } 里的关联项按类型连，不按名字登
                        if depth != 2 or word not in ("const", "static", "fn", "struct", "enum", "union", "type"):
                            continue
                        name_index = index + 2 if word == "static" and rust_file.t(index + 1) == "mut" else index + 1
                        name = rust_file.t(name_index)
                        if rust_file.kind(name_index) == IDENT and name not in RUST_KEYWORDS and name != "_":
                            self.macro_rules_literal_names[unit.short].add(name)
                elif unit.kind == "static" and any(rust_file.t(index) == "global_allocator" and rust_file.t(index - 1) == "[" and rust_file.t(index - 2) == "#"
                                                   for index in span):
                    self.global_allocators.append(unit)
                elif unit.kind in RUST_TYPE_KINDS:
                    type_names.add(unit.short)
                elif unit.kind == "trait":
                    trait_names.add(unit.short)
                elif unit.kind == "impl" and unit.is_trait_impl and not unit.blanket:
                    impls.append(unit)
        local_roots = {"crate", "self", "super", "Self"} | {crate.lib_name for crate in self.crates}
        for unit in impls:
            qualified_outside = unit.self_path_root is not None and unit.self_path_root not in local_roots
            if unit.trait_name in trait_names and (qualified_outside or not unit.self_types & type_names):
                self.foreign_trait_impls.add(id(unit))

    def names_reaching_outside_crate(self, crate):
        """别的包按裸名连得到工作区外这个 crate 的哪些名字（文件头 Rust 闭包「工作区外的 path 依赖」）→ (全收, 名字集)。扫的是这个 crate 以外每个目标的文件：
        根名是这个 crate 的库名，use / extern crate 语句里出现根名的，它后面的名字都算引进来、也都当根名（引进来的可能是模块或别名）；收到不再变；
        语句里根名后面有 * 的全收。路径 根名::a::b 里根名后面的段都算；名字后面接 ! 与括号、又是这个 crate 里 macro_rules! 的名字的也算。
        引进来的名字按整个工作区收、不分模块：经别处 pub use 转出去再 glob 引进来的照样认得。一个 crate 只算一趟。"""
        cached = self.outside_crate_reaching_names.get(crate.directory)
        if cached is not None:
            return cached
        own_paths = {path for target in self.targets.values() if target.crate is crate for path in target.files}
        paths = sorted({path for target in self.targets.values() if target.crate is not crate for path in target.files} - own_paths)
        macro_names = {unit.short for path in own_paths for unit in self.files[path].units if unit.kind == "macro_rules"}
        statements = []
        for path in paths:
            rust_file = self.files[path]
            texts, kinds = rust_file.texts, rust_file.kinds
            for index in (i for i, text in enumerate(texts) if text in ("use", "crate") and kinds[i] == IDENT):
                if texts[index] == "crate" and rust_file.t(index - 1) != "extern":
                    continue
                end = index + 1
                while end < rust_file.count and texts[end] != ";":
                    end += 1
                statements.append((rust_file, index + 1, end))
        roots, every_name = {crate.lib_name}, False
        grown = True
        while grown:
            grown = False
            for rust_file, low, high in statements:
                start = next((i for i in range(low, high) if rust_file.kinds[i] == IDENT and rust_file.texts[i] in roots), None)
                if start is None:
                    continue
                for i in range(start + 1, high):
                    text = rust_file.texts[i]
                    if text == "*":
                        every_name = True
                    elif rust_file.kinds[i] == IDENT and text not in ("self", "super", "crate", "as") and text not in roots:
                        roots.add(text)
                        grown = True
        names = set(roots)
        for path in paths:
            rust_file = self.files[path]
            texts, kinds = rust_file.texts, rust_file.kinds
            if not (roots | macro_names) & set(texts):
                continue
            for index, text in enumerate(texts):
                if kinds[index] != IDENT:
                    continue
                if text in macro_names and rust_file.t(index + 1) == "!" and rust_file.t(index + 2) in ("(", "[", "{"):
                    names.add(text)
                if text not in roots:
                    continue
                following = index + 1
                while rust_file.t(following) == ":" and rust_file.t(following + 1) == ":" and rust_file.kind(following + 2) == IDENT:
                    names.add(texts[following + 2])
                    following += 3
        self.outside_crate_reaching_names[crate.directory] = (every_name, frozenset(names))
        return self.outside_crate_reaching_names[crate.directory]

    def module_of(self, unit, identifier):
        """单元在目标 identifier 里所在的模块路径：文件在目标模块树里的路径接文件里的行内模块路径。"""
        return tuple(self.targets[identifier].files[unit.file][0]) + tuple(unit.scope or ())

    def is_compiled_conditionally(self, unit, identifier):
        """单元带条件编译（crates/singlefs-unit-graph 的 is_conditionally_compiled 同一个判法）：它自己的条件链非空（外层行内模块、impl 头与它自己的 cfg、
        cfg_attr 里套的 cfg），或它所在文件在目标 identifier 里的连法里有一条带条件（连进来的每一条 mod 声明自己带条件、或声明它的文件又是这样）。"""
        return bool(unit.conditions) or self.file_is_linked_conditionally(identifier, unit.file, frozenset())

    def file_is_linked_conditionally(self, identifier, path, visited):
        target = self.targets[identifier]
        declarations = target.declared_by.get(path, ())
        if not declarations:
            return bool(target.files[path][1])
        return any(declaration.conditions or (declaration.file not in visited and self.file_is_linked_conditionally(identifier, declaration.file, visited | {path}))
                   for declaration in declarations)

    def candidates_seen_from_module(self, unit, identifier, name, candidates, other_crate_roots=frozenset()):
        """同一个目标里，单元按裸名 name 连得到 candidates 里的哪几份（文件头 Rust 闭包「R-裸名同一个目标里按模块筛」；crates/singlefs-unit-graph 的
        candidates_seen_from_the_module_of_the_unit 同一个判法）：与单元同一模块的；所在模块的 use 引进了 name（或是 glob）、use 路径里有候选所在模块的末段
        （花括号展开之后整条比，placed_by_the_use_path：crate、self、super 起头的从根模块、本模块、往上那一层算，别的看候选所在模块以这条路径收尾）的；单元里写成 前缀::name、前缀是候选所在模块末段（crate、self 照算；
        前缀是 super 时按整条路径里 super 的个数往上数层）的。use 路径或 …::name 整条路径的根名在 other_crate_roots 里的（std、core、alloc 与看得见的
        别的 crate 的库名）指的是别的 crate，跳过这一处。认不出落在哪一份的退回全连：use 或路径点到的模块里一份都没有（经 pub use 转出来的）、
        前缀不是类型也不是哪一份所在模块、单元里另有 use 语句（函数体里的 use）、name 后面接 !。
        前缀是类型或 trait（Self、Tr::x）的是关联项，R-路径另管，不按这一处连。哪一样都不沾的（字段名、局部变量）一份都不连。"""
        key = (id(unit), identifier, name, other_crate_roots)
        cached = self.outside_crate_module_candidates.get(key)
        if cached is not None:
            return cached
        own = self.module_of(unit, identifier)
        modules = {id(candidate): self.module_of(candidate, identifier) for candidate in candidates}
        seen = {id(candidate) for candidate in candidates if modules[id(candidate)] == own}
        every = False

        def placed(segments):
            """路径（不含 name 本身）点到的候选；一份都没点到交 None。路径里写了几个 super 就往上几层（超出根模块的按根模块算）。"""
            found = set()
            levels_up = segments.count("super")
            for candidate in candidates:
                module = modules[id(candidate)]
                if ((module and module[-1] in segments) or (not module and segments[:1] == ["crate"])
                        or (levels_up and module == own[:max(len(own) - levels_up, 0)]) or (segments[:1] == ["self"] and module == own)):
                    found.add(id(candidate))
            return found or None

        scope_uses = self.uses_by_scope.get((unit.file, unit.scope), ())
        conditions_checked = "rust-use-path-conditional-candidates-narrowed" not in self.breaks
        re_exports_checked = "rust-use-path-re-exporting-modules-narrowed" not in self.breaks
        first_segments_checked = "rust-use-path-first-segment-imported-narrowed" not in self.breaks
        same_use_checked = "rust-use-path-first-segment-same-use-excluded" not in self.breaks
        item_macros_checked = "rust-use-path-first-segment-item-macros-ignored" not in self.breaks
        scope_has_item_macros = bool(self.item_macros_by_scope.get((unit.file, unit.scope)))

        def is_imported_by_another_use(uses, except_use, imported_name):
            """作用域 uses 里除 except_use 之外有一条 use 按名字引进 imported_name（as 改出的名字也算；glob 不算）；except_use 是 None 时每一条都算。"""
            return any(other is not except_use and imported_name in (other.imports or ()) for other in uses)

        def placed_by_the_use_path(path, path_use):
            """use 的一支路径（不含名字本身）恰好点到的候选；一份都没点到交 None。恰好点到才算：点到的模块里自己定义的名字压过它 glob 转出来的；
            模块里没有这个名字的候选，名字可能是它经 pub use 转出来的，落在哪一份看不出，退回全连。点到的候选带条件编译、或候选所在模块另有 use 按名字引进它，
            名字可能是 cfg 互斥的另一支 pub use 转来的，退回全连；相对路径的首段在本作用域被 use 按名字引进（含 as 改名、含这一支所在的同一条 use 里别的一支），
            或本作用域有条目级宏调用（宏展开出的 use 看不见），首段真指的模块看不出，退回全连。path_use 是这一支所在的 use。"""
            leading_supers = len(path) - len(list(itertools.dropwhile(lambda segment: segment == "super", path)))
            exact = (tuple(path[1:]) if path[:1] == ["crate"] else own + tuple(path[1:]) if path[:1] == ["self"]
                     else own[:max(len(own) - leading_supers, 0)] + tuple(path[leading_supers:]) if leading_supers else None)
            if first_segments_checked and exact is None and path:
                if is_imported_by_another_use(scope_uses, path_use if not same_use_checked else None, path[0]):
                    return None
                if item_macros_checked and scope_has_item_macros:
                    return None
            found = {id(candidate) for candidate in candidates
                     if (modules[id(candidate)] == exact if exact is not None else bool(path) and modules[id(candidate)][-len(path):] == tuple(path))}
            for candidate in candidates:
                if id(candidate) not in found:
                    continue
                if conditions_checked and self.is_compiled_conditionally(candidate, identifier):
                    return None
                if re_exports_checked and is_imported_by_another_use(self.uses_by_scope.get((candidate.file, candidate.scope), ()), path_use, name):
                    return None
            return found or None

        rust_file = self.files[unit.file]
        for use in self.uses_by_scope.get((unit.file, unit.scope), ()):
            if use.first_token is None or not (use.import_glob or name in use.imports):
                continue
            if use_path_root(rust_file, use.first_token, use.last_token) in other_crate_roots:
                continue
            if "rust-use-paths-flattened" in self.breaks:
                hit = placed([rust_file.texts[index] for index in range(use.first_token, use.last_token)
                              if rust_file.kinds[index] == IDENT and rust_file.texts[index] not in ("use", "pub", "as", name)])
                every = every or hit is None
                seen |= hit or set()
                continue
            use_keyword = next((index for index in range(use.first_token, use.last_token)
                                if rust_file.kinds[index] == IDENT and rust_file.texts[index] == "use"), use.first_token)
            # 这条 use 里引进 name 的每一支（*、名字本身），花括号展开之后整条比；原名 as name 那一支 as 前面不是 段::，路径只到外层花括号，
            # 多半点不到、退回全连（不顺着原名点准：use 引进的是原名改的 name，原名所在模块里叫 name 的是另一份）
            leaves = [index for index in range(use_keyword + 1, use.last_token)
                      if rust_file.t(index) == "*" or (rust_file.kinds[index] == IDENT and rust_file.t(index) == name and rust_file.t(index + 1) != ":")]
            alias_followed_to_the_original = "rust-use-alias-leaf-followed-to-the-original" in self.breaks
            hits = [placed_by_the_use_path(use_path_leading_to(rust_file, use_keyword, index - 2 if alias_followed_to_the_original and rust_file.t(index - 1) == "as" else index), use)
                    for index in leaves]
            if not hits or None in hits:
                every = True
            else:
                seen |= set().union(*hits)
        if unit.first_token is not None and not every:
            if identifier not in self.outside_crate_type_names:
                self.outside_crate_type_names[identifier] = frozenset(
                    short for short, found in self.by_name.get(identifier, {}).items() if any(other.kind in RUST_TYPE_OR_TRAIT_KINDS for other in found))
            types = self.outside_crate_type_names[identifier]
            for index in range(unit.first_token, unit.last_token):
                text = rust_file.texts[index]
                if text == "use" and rust_file.kinds[index] == IDENT and index != unit.first_token:
                    every = True
                    break
                if text != name or rust_file.kinds[index] != IDENT:
                    continue
                if rust_file.t(index + 1) == "!" and rust_file.t(index + 2) in ("(", "[", "{"):
                    every = True
                    break
                if rust_file.t(index - 1) == ":" and rust_file.t(index - 2) == ":" and rust_file.kind(index - 3) == IDENT:
                    prefix = rust_file.t(index - 3)
                    # 整条路径 根::…::前缀::name，从前缀往前顺着「标识符 ::」收到根
                    path_before, position = [prefix], index - 3
                    while position >= 3 and rust_file.t(position - 1) == ":" and rust_file.t(position - 2) == ":" and rust_file.kind(position - 3) == IDENT:
                        position -= 3
                        path_before.append(rust_file.t(position))
                    path_before.reverse()
                    if path_before[0] in other_crate_roots:
                        continue
                    if prefix == "Self" or (prefix in types and prefix not in ("crate", "self", "super")):
                        continue
                    hit = placed(path_before if prefix == "super" else [prefix])
                    if hit is None:
                        every = True
                        break
                    seen |= hit
        found = list(candidates) if every else [candidate for candidate in candidates if id(candidate) in seen]
        self.outside_crate_module_candidates[key] = found
        return found

    def visible(self, target_id, development=False):
        target = self.targets[target_id]
        crate = target.crate
        out = [target_id]
        if target.kind == "doctest":
            lib = self.lib_target(crate)
            return [target_id] + (self.visible(lib.id, development=True) if lib else [])
        if target.kind == "build":
            direct = list(crate.dependencies["build"])
        else:
            direct = list(crate.dependencies["normal"])
            if development or target.kind in ("test", "bench", "example"):
                direct += crate.dependencies["dev"]
            lib = self.lib_target(crate)
            if target.kind != "lib" and lib is not None:
                out.append(lib.id)
        seen = set()
        todo = [dependency.directory for dependency in direct if dependency.directory in self.crate_by_directory]
        while todo:
            directory = todo.pop()
            if directory in seen:
                continue
            seen.add(directory)
            dependency_crate = self.crate_by_directory[directory]
            lib = self.lib_target(dependency_crate)
            if lib is not None and lib.id not in out:
                out.append(lib.id)
            todo += [dependency.directory for dependency in dependency_crate.dependencies["normal"] if dependency.directory in self.crate_by_directory]
        return out

    def lib_target(self, crate):
        return self.targets.get(f"{crate.label}:lib:{crate.lib_name}")

    def test_functions(self, target_id):
        """编译目标里带 #[test] 的 fn 单元：[(libtest 全名, 单元)]，全名 = 文件在目标里的模块路径加单元的限定名。"""
        cache = self.__dict__.setdefault("_test_functions", {})
        if target_id not in cache:
            found = []
            for path, (module_path, _chain) in sorted(self.targets[target_id].files.items()):
                for candidate in self.files[path].units:
                    if candidate.kind == "fn" and any(condition[1] == TEST_ATTRIBUTE_RAW for condition in candidate.conditions or ()):
                        found.append(("::".join(list(module_path) + [candidate.name]), candidate))
            cache[target_id] = found
        return cache[target_id]

    def exact_test(self, full_name, target_id):
        """同一编译目标里 libtest 全名恰好是 full_name 的 #[test] 单元；不是恰好一个回 None（拒算 current-exe）。"""
        hits = [candidate for name, candidate in self.test_functions(target_id) if name == full_name]
        return hits[0] if len(hits) == 1 else None

    def resolve_features(self, linked_crates, entry_crate, requested, no_default, entry_sections):
        """照 cargo 的特性统一（resolver 2，与 crates/singlefs-unit-graph 的 resolve_features 同一口径）：依赖边上请求的特性，
        入口那个 crate 只顺 entry_sections 那几段（rust_entry_dependency_sections），别的 crate 只顺普通依赖——它们的 dev 依赖不编，
        dev 依赖里才开的特性不进 bin 与库目标。弄坏开关 rust-dependency-features-from-every-section 下退回旧口径：每个 crate 每一段都顺。"""
        every_section = "rust-dependency-features-from-every-section" in active_breaks()
        enabled = {crate.directory: set() for crate in self.crates}

        def enable(crate, feature):
            have = enabled[crate.directory]
            if feature in have:
                return
            have.add(feature)
            for item in crate.features.get(feature, []):
                if item.startswith("dep:"):
                    continue
                if "/" in item:
                    dependency_name, sub_feature = item.split("/", 1)
                    dependency_name = dependency_name.rstrip("?").replace("-", "_")
                    for section in crate.dependencies.values():
                        for dependency in section:
                            if dependency.name == dependency_name and dependency.directory in self.crate_by_directory:
                                enable(self.crate_by_directory[dependency.directory], sub_feature)
                    continue
                enable(crate, item)

        if not no_default and "default" in entry_crate.features:
            enable(entry_crate, "default")
        for crate, feature in requested:
            enable(crate, feature)
        for crate in linked_crates:
            sections = tuple(crate.dependencies) if every_section else entry_sections if crate is entry_crate else ("normal",)
            for section in sections:
                for dependency in crate.dependencies[section]:
                    dependency_crate = self.crate_by_directory.get(dependency.directory)
                    if dependency_crate is None or dependency_crate not in linked_crates:
                        continue
                    if dependency.default_features and "default" in dependency_crate.features:
                        enable(dependency_crate, "default")
                    for feature in dependency.features:
                        enable(dependency_crate, feature)
        return enabled

    def crate_by_package(self, name):
        for crate in self.crates:
            if name in (crate.package, crate.lib_name, crate.label):
                return crate
        raise UsageError(f"没有叫 {name} 的包")

    def find_entry(self, path, qualified, target_option):
        """→ (入口单元, 目标号)。path 为 None 时 qualified 是 fn 路径 <crate>::<模块…>::<名>。"""
        if path is None:
            first, _sep, rest = qualified.partition("::")
            crate = self.crate_by_package(first)
            lib = self.lib_target(crate)
            if lib is None:
                raise UsageError(f"{first} 没有 lib 目标")
            hits = []
            for file_path, (module_path, _chain) in lib.files.items():
                for unit in self.files[file_path].units:
                    full = "::".join(list(module_path) + [unit.name])
                    if full == rest:
                        hits.append(unit)
            if len(hits) != 1:
                raise UsageError(f"fn 路径 {qualified} 命中 {len(hits)} 个单元（要恰好一个）")
            return hits[0], lib.id
        if not os.path.isfile(os.path.join(self.root, path)):
            raise UsageError(f"入口文件不在：{path}")
        rust_file = self.file(path)
        hits = [unit for unit in rust_file.units if unit.name == qualified]
        if len(hits) != 1:
            raise UsageError(f"{path} 里限定名 {qualified} 命中 {len(hits)} 个单元（要恰好一个；dump 看限定名）")
        unit = hits[0]
        owners = [target for target in self.targets.values() if path in target.files]
        if unit.kind == "doctest":
            crate = self.crate_of_path(path)
            target = RustTarget(crate, "doctest", unit.name, path)
            self.targets[target.id] = target
            self.unit_targets[id(unit)].add(target.id)
            return unit, target.id
        if target_option:
            if target_option not in self.targets or path not in self.targets[target_option].files:
                raise UsageError(f"目标 {target_option} 里没有 {path}")
            return unit, target_option
        if not owners:
            raise UsageError(f"{path} 不在任何编译目标的模块树里")
        owners.sort(key=lambda target: (TARGET_PREFERENCE.index(target.kind), target.id))
        return unit, owners[0].id


def doctest_entry_paths(library, doctest):
    """能点到这个文档测试的入口路径（不带 crate 名的段，元组），照 crates/singlefs-unit-graph 的 resolve_entry_units 文档测试那一支与 matches_entry_path 认的写法：
    挂模块的是那个模块在 lib 模块树里的路径；挂条目的是条目的模块路径接条目名，条目是 impl / trait 里的方法时是模块路径、行内模块、类型名、方法名，
    类型名取 impl 的各个 self 类型与头自己的名字。头一项是门禁 code-tooling 的 function-units-two-implementations-agree 格拼 unit-graph closure 时取的那一种
    （impl 取排序后头一个 self 类型）。所在文件不在 lib 目标模块树里的交空表（Rust 那一份只认 lib 里的文档测试）。"""
    placement = library.files.get(doctest.file)
    if placement is None:
        return []
    module_path = list(placement[0])
    owner = doctest.container
    if owner is None:
        return [tuple(module_path + list(doctest.doc_module or ()))]
    if owner.container is None:
        return [tuple(module_path + owner.name.split("::"))]
    head = owner.container
    named = sorted(head.self_types)[0] if head.kind == "impl" and head.self_types else head.short
    others = sorted((set(head.self_types or ()) | {head.short}) - {named})
    return [tuple(module_path + list(owner.scope or ()) + [type_name, owner.short]) for type_name in [named] + others]


def doctest_entry_group(workspace, entry):
    """文档测试入口一起当入口的那一组文档测试单元（rust_closure_of 的入口与 Closure.entry_units）：照 crates/singlefs-unit-graph 的 resolve_entry_units，
    文档测试目标的入口是所挂条目或模块的路径，解到那个路径点到的条目与模块挂的全部文档测试；这里拿 entry 的头一条入口路径（doctest_entry_paths）
    在 lib 目标模块树的每份文件里找能被它点到的文档测试，按 Unit.sort_key 排。lib 目标没有、entry 不在里面的，只交 entry 自己。"""
    crate = workspace.crate_of_path(entry.file)
    library = workspace.lib_target(crate) if crate is not None else None
    paths = doctest_entry_paths(library, entry) if library is not None else []
    if not paths:
        return [entry]
    group = [unit for path in sorted(library.files) for unit in workspace.file(path).units
             if unit.kind == "doctest" and paths[0] in doctest_entry_paths(library, unit)]
    if not any(unit is entry for unit in group):
        return [entry]
    return sorted(group, key=Unit.sort_key)


def rust_unit_is_test(workspace, unit, target_id):
    if workspace.targets[target_id].kind in ("test", "bench"):
        return True
    return rust_unit_is_test_code(workspace, unit, target_id)


def rust_unit_is_test_code(workspace, unit, target_id):
    """单元自己带 #[test]、或它（连同外层的 impl、行内模块与把它所在文件连进来的 mod 声明）带 cfg(test)：只在按测试编时才有的代码。
    不含整个目标都是测试的目标种类（tests/ 下的集成测试里没有 #[test] 的单元不算）。"""
    chain = workspace.targets[target_id].files.get(unit.file, ((), []))[1]
    probe = unit
    while probe is not None:
        if any(condition_mentions_test(condition[0]) for condition in probe.conditions):
            return True
        probe = probe.container
    return any(condition_mentions_test(condition[0]) for condition in chain)


class Closure:
    def __init__(self):
        self.members = {}
        self.order = []
        self.refusals = set()
        self.cfg_lines = []
        self.programs = {}
        self.via = {}
        # Rust 闭包另记（unit_graph_key 照 crates/singlefs-unit-graph 的 reuse_key 算键用）：入口目标的种类、入口目标的生效 cfg 集（无值名字写名字，
        # 键值写 键="值"）、读数据文件或数据目录的每一处（文件、行、数据文件或目录、摘要、读它的单元、种类 data 或 data-directory）、每个单元收进来时所在的目标
        self.entry_target_kind, self.entry_cfg_atoms = None, []
        # 键里的 cfg 成分（「<原子>\t<true|false>」的集合，rust_closure_of 填）；None 是没记（key_cfg_items 退回整套生效 cfg）
        self.cfg_atoms = None
        self.data_sites = set()
        self.unit_targets = {}
        # 从哪几个入口收的（键里的 entry 行，entry_key_lines）：rust_closure_of、ScriptClosure.run 与 WGSL 点名的入口各记；别处自己拼的闭包不记，不出那几行
        self.entry_units = []
        # ScriptClosure.run 每收完一轮时闭包的单元数（explain_lines 打 round 那几行）
        self.round_sizes = []
        # 按第一列选行的表脚本的每一次起法 [(脚本, 词或 None, 起它的单元或 None, whole|rows|no-table)]（ScriptClosure.table_launched 记，explain_lines 打 table-launch 那几行）
        self.table_launches = []
        # 工具链询问（文件头「版本询问」）：{(文件, 行)}，只记、不进键（弄坏开关 python-toolchain-queries-keyed 下每一处另收一个工具链单元）
        self.toolchain_queries = set()
        # 按角色排除的文件（文件头「按角色排除的文件」）：它们的单元（连同以它们为内容的数据单元）不进闭包
        self.excluded_files = frozenset()

    def add(self, unit, via, rule):
        if id(unit) in self.members or unit.file in self.excluded_files:
            return False
        self.members[id(unit)] = unit
        self.order.append(unit)
        self.via[id(unit)] = (via, rule)
        return True

    def units(self):
        return sorted(self.order, key=Unit.sort_key)

    def refusal_list(self):
        return sorted(self.refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))
