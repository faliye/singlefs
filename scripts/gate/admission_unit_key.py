"""准入（入口 admission.py）的一块：单元键（Request、UnitKey、KeyContext、compute_unit_key）、工具链行与构建环境的生效值、清单与依赖版本（这两样不进单元键，
给装置 scripts/experiment-apparatus/device-code-key.py 与子命令 build-environment-lines 用），以及判定库 VerdictLibrary、组成清单的落盘与打包、复用哨兵的载入、按文件存的种类（library-files 子命令）。
入口 admission.py 把这一块的名字再导出。
"""

import collections
import copy
import glob
import gzip
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib

from admission_base import (
     VERDICT_LIBRARY_DIRECTORY_NAME, RED_VERDICT_REASON, CHANGED_UNITS_SHOWN_AT_MOST, DOMAIN_ENVIRONMENT_VARIABLES, DOMAIN_VARIABLE_LITERAL,
     EXIT_REGISTRATION_ERROR, PROBE_TIMEOUT_SECONDS, CARGO_CONFIGURATION_FILE_NAMES, TOOLCHAIN_FILE_NAMES, BUILD_ENVIRONMENT_EXACT_VARIABLES,
     BUILD_ENVIRONMENT_VARIABLE_FORMS, BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT, BUILD_CONFIGURATION_SECTIONS, BUILD_KEYS_LEFT_OUT,
     BUILD_ENVIRONMENT_ALL_SETTINGS_BREAK, CONFIGURATION_ANCESTORS_UNREAD_BREAK, BUILD_ENVIRONMENT_PROGRAM_VARIABLES,
     BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS, BUILD_ENVIRONMENT_COMPILER_VARIABLES, CONFIGURATION_PROGRAM_BUILD_KEYS, REPLAY_RESEARCH_DIRECTORY,
     RegistrationError, InputManifestError, read_registration_rows, function_units, UnitKeyError, break_is_set, say, crash_cases_of,
     git_common_directory, TEST_PLAN_SHARED,
)

JUDGE_ONLY_ENTRY = "-"   # 请求的入口写它：被判那棵树里没有程序入口，键只由判法与内容输入组成（文件头「判定请求」）


class Request:
    """一条判定的请求：入口与它的键要拼进来的东西（请求的写法见文件头「判定请求」）。tokens 原样存进库，下一趟 gate-reuse 拿它重算。"""

    def __init__(self, name, entry, tokens):
        self.name = name
        self.entry = entry
        self.tokens = list(tokens)
        self.root = "."
        self.target = None
        self.features = []
        self.no_default_features = False
        self.cfg = []
        self.profile = None
        self.judges = []
        self.judge_root = None
        self.lines = []
        self.files = []
        self.variables = {}
        # program-argument:<词>：程序入口（python 与 shell）带的参数，照次序（lib_function_units.py 文件头「程序入口带参数」）；None 是没给
        self.program_arguments = None
        for token in self.tokens:
            kind, _separator, value = token.partition(":")
            if token == "no-default-features":
                self.no_default_features = True
            elif kind == "root" and value:
                self.root = value
            elif kind == "target" and value:
                self.target = value
            elif kind == "feature" and "/" in value:
                self.features.append(value)
            elif kind == "cfg" and value:
                self.cfg.append(value)
            elif kind == "profile" and value:
                self.profile = value
            elif kind == "judge" and value:
                self.judges.append(value)
            elif kind == "judge-root" and value:
                self.judge_root = value
            elif kind == "line" and value:
                self.lines.append(value)
            elif kind == "file" and value:
                self.files.append(value)
            elif kind == "program-argument" and value:
                self.program_arguments = (self.program_arguments or []) + [value]
            elif kind == "variable" and "=" in value and domain_variable_form(value.partition("=")[0]) is not None:
                variable_name, _equals, setting = value.partition("=")
                self.variables[variable_name] = setting
            else:
                raise RegistrationError(f"判定请求 {name} 里认不出的一项「{token}」（写法见 admission.py 文件头「判定请求」）")
        if self.root.startswith("/") or ".." in self.root.split("/"):
            raise RegistrationError(f"判定请求 {name} 的 root:{self.root} 要写成仓根起的相对目录")

    def text(self):
        return "\t".join([self.name, self.entry, *self.tokens])


def parse_request_line(line):
    columns = line.rstrip("\n").split("\t")
    if len(columns) < 2 or not columns[0] or not columns[1]:
        raise RegistrationError(f"判定请求那一行要「<名字><制表符><入口>[<制表符><项>…]」：{line.strip()}")
    return Request(columns[0], columns[1], [column for column in columns[2:] if column])


class UnitKey:
    """一把单元键：key 是 64 位十六进制，refused 是拒算行（不空时 key 是 None）；composition 是组成清单（单元行与输入行，只当索引，作废时比它报出变了什么）。"""

    def __init__(self, key, composition, refusals):
        self.key = key
        self.composition = composition
        self.refusals = refusals


def entry_language(entry):
    path = entry.split("::", 1)[0]
    for suffix, language in ((".rs", "rust"), (".wgsl", "wgsl"), (".py", "python"), (".sh", "shell"), (".bash", "shell")):
        if path.endswith(suffix):
            return language
    return "rust" if "::" in entry and "/" not in path else None


def run_text(command, cwd, environment=None):
    """起一条只问版本的命令：交 (退出码, stdout 去掉两头空白)；起不来交 (127, "")。"""
    try:
        completed = subprocess.run(command, cwd=cwd, capture_output=True, text=True, errors="replace", env=environment, timeout=PROBE_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired):
        return 127, ""
    return completed.returncode, completed.stdout.strip()


def flattened_setting_lines(prefix, value, out):
    """TOML 解析出来的值摊成「<前缀>.<键>=<值>」：只取生效的值（注释、空白、写的次序都不进）。"""
    if isinstance(value, dict):
        for name in sorted(value):
            flattened_setting_lines(f"{prefix}.{name}" if prefix else name, value[name], out)
    else:
        out.append(f"{prefix}={json.dumps(value, ensure_ascii=False, sort_keys=True)}")


def merged_configuration(layers):
    """cargo 的配置层（从近到远）合成一份生效的：标量取最近的那一层，数组照 cargo 的做法远的在前、近的接在后面。"""
    merged = {}

    def merge(into, value):
        for name, item in value.items():
            if isinstance(item, dict) and isinstance(into.get(name), dict):
                merge(into[name], item)
            elif isinstance(item, list) and isinstance(into.get(name), list):
                into[name] = into[name] + item
            elif isinstance(item, dict):
                into[name] = copy.deepcopy(item)
            else:
                into[name] = item
    for layer in reversed(layers):
        merge(merged, layer)
    return merged


def build_environment_variable_left_out(variable):
    """构建环境那几行里不收的变量：CARGO_BUILD_JOBS 一律不收，BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT 里别的在 build-environment-all-settings 下收。"""
    if variable == "CARGO_BUILD_JOBS":
        return True
    if break_is_set(BUILD_ENVIRONMENT_ALL_SETTINGS_BREAK):
        return False
    return variable in BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT


def configuration_affecting_output(merged):
    """合成好的 cargo 配置里收的那几张表（BUILD_CONFIGURATION_SECTIONS）：build（去掉 BUILD_KEYS_LEFT_OUT）、target、env；
    [profile.<名>]、[alias]、[net]、[http]、[term] 这类不收。弄坏开关 build-environment-all-settings 下整份交回。"""
    if break_is_set(BUILD_ENVIRONMENT_ALL_SETTINGS_BREAK):
        return merged
    kept = {}
    for section in BUILD_CONFIGURATION_SECTIONS:
        value = merged.get(section)
        if section == "build" and isinstance(value, dict):
            value = {name: item for name, item in value.items() if name not in BUILD_KEYS_LEFT_OUT}
        if value:
            kept[section] = value
    return kept


# 弄坏开关 script-closures-process-wide 下脚本闭包的记法：(根, 入口, program-argument) → (闭包, 拒算说明)，整个进程一张
SCRIPT_CLOSURES_UNDER_BREAK = {}


class KeyContext:
    """一趟里算单元键的共用状态：每个闭包根一份 Rust 工作区、工具链行与 Cargo.lock（这两样不进单元键：rust_toolchain_lines、dependency_lines
    给 scripts/experiment-apparatus/device-code-key.py 调）。"""

    def __init__(self, repository_root, environment=None):
        self.root = os.path.abspath(repository_root)
        self.environment = dict(os.environ if environment is None else environment)
        self.workspaces = {}
        self.toolchains = {}
        self.locks = {}
        self.registration_rows = None
        self.host = None
        self.script_indices = {}
        self.script_closures = {}
        self.judge_keys, self.judge_closures = {}, {}
        self.rust_closures = {}
        self.source_lines = {}
        self.unit_literals = {}

    def rust_closure(self, closure_root, workspace, unit, target_id, cfg, features, no_default_features):
        """同一个入口、同一组 cfg 与特性的 Rust 闭包一趟只收一次（test-plan 判编不编时收过的，算键时接着用）。"""
        cache_key = (closure_root, id(unit), target_id, tuple(cfg), tuple(features), no_default_features)
        if cache_key not in self.rust_closures:
            # 入口自己名字的定义那一处不当引用（lib_function_units.py 的 rust_closure_of「entry_definition_name_unresolved」）：同一目标里短名相同、
            #   不在同一模块的两条测试函数不互相收进闭包、不共用一把键。弄坏开关 entry-homonyms-pulled 下收
            self.rust_closures[cache_key] = function_units().rust_closure_of(
                workspace, unit, target_id, list(cfg), True, list(features), no_default_features,
                entry_definition_name_unresolved=not break_is_set("entry-homonyms-pulled"))
        return self.rust_closures[cache_key]

    # Python 与 shell 那一侧：同一棵树的脚本索引（解析过的模块、bash 规范化过的脚本）一趟只建一份，每个入口各收各的闭包
    def script_closure(self, closure_root, entry, program_arguments=None):
        """python / shell 入口的闭包；program_arguments 是程序入口带的参数（只写 <文件> 的入口才用；lib_function_units.py 文件头「程序入口带参数」）。
        同一个（根、入口、参数）在这个 KeyContext 里只收一次：驱动函数原文与行键各问一遍、几行共用的判法与登记入口都问到同一个，后面的拿前面收好的同一个对象（调用方只读它）。
        收不成的（入口不在、限定名不唯一）也记，再问照抛同一句。弄坏开关 script-closures-recollected：不记；script-closures-process-wide：记在进程里、不随 KeyContext。"""
        slot = (closure_root, entry, None if program_arguments is None else tuple(program_arguments))
        table = SCRIPT_CLOSURES_UNDER_BREAK if break_is_set("script-closures-process-wide") else self.script_closures
        if slot not in table or break_is_set("script-closures-recollected"):
            try:
                table[slot] = (self.collected_script_closure(closure_root, entry, program_arguments), None)
            except UnitKeyError as error:
                table[slot] = (None, str(error))
        closure, problem = table[slot]
        if closure is None:
            raise UnitKeyError(problem)
        return closure

    def collected_script_closure(self, closure_root, entry, program_arguments):
        """现收一个 python / shell 入口的闭包，不查也不记 script_closure 的记法；入口不在、限定名不唯一抛 UnitKeyError。"""
        library = function_units()
        path, _separator, qualified = entry.partition("::")
        under = [place for place in library.DEFAULT_SCRIPT_DIRECTORIES if os.path.isdir(os.path.join(closure_root, place))]
        excluded_files = not_judging_files_of_entry(library, path)
        dirname = os.path.dirname(path) or "."
        already_covered = any(dirname == place or dirname.startswith(place.rstrip("/") + "/") for place in under)
        if not already_covered:
            under = list(dict.fromkeys(under + [dirname]))
        index_key = (closure_root, tuple(under))
        try:
            base = self.script_indices.get(index_key)
            if base is None:
                base = self.script_indices[index_key] = library.ScriptClosure(closure_root, under)
            script = library.ScriptClosure(closure_root, under, excluded_files)
            script.python = base.python
            script.shell = base.shell
            language = entry_language(entry)
            if language == "python":
                module = script.python.module(path)
                if module is None:
                    raise UnitKeyError(f"入口文件不在：{path}")
                if qualified:
                    entries = [unit for unit in module.units if unit.name == qualified and not unit.data]
                    if len(entries) != 1:
                        raise UnitKeyError(f"{path} 里限定名 {qualified} 命中 {len(entries)} 个单元（要恰好一个）")
                else:
                    entries = module.program_entries()[:1]
                script.closure.refusals.update(module.refusals)
                closure = script.run(entries, None if qualified else program_arguments)
                if base.shell is None:
                    base.shell = script.shell
                return closure
            index = base.shell_index([path])
            script.shell = index
            shell_file = index.files.get(path)
            if shell_file is None:
                raise UnitKeyError(f"入口文件不在：{path}")
            script.closure.refusals.update(shell_file.refusals)
            if shell_file.script_body is None:
                return script.closure
            if qualified:
                entries = [unit for unit in shell_file.units if unit.name == qualified and not unit.data]
                if len(entries) != 1:
                    raise UnitKeyError(f"{path} 里限定名 {qualified} 命中 {len(entries)} 个单元（要恰好一个）")
            else:
                entries = [shell_file.script_body]
            # 弄坏开关 shell-program-arguments-dropped：shell 入口不交参数，整份收
            return script.run(entries, None if qualified or break_is_set("shell-program-arguments-dropped") else program_arguments)
        except (library.FunctionUnitsError, library.UsageError) as error:
            raise UnitKeyError(f"{entry}：{error}") from error

    def judge_key(self, judge, judge_root=None, entry_file=None):
        """判法入口的闭包键（只由闭包里的单元组成，解释器与外部程序的版本不进，文件头「单元键」）：(键或 None, 组成清单, 拒算行)；同一个判法、同一个入口文件一趟只算一次。
        judge_root 是判法所在的那棵树（门禁写它自己所在的仓：样本档被判的是样本，判法在门禁那一份仓里）；不给是被判的根。
        entry_file 是这一条判定的入口文件（相对根）：判法闭包里按行进键的登记表只进它查的那几行（lib_function_units.py 文件头「按行进键的登记表」）；
        不给照整份。"""
        root = os.path.normpath(os.path.join(self.root, judge_root)) if judge_root else self.root
        cache_key = (root, judge, entry_file)
        if cache_key not in self.judge_keys:
            library = function_units()
            closure_slot = (root, judge)
            try:
                if closure_slot not in self.judge_closures:
                    # 判法的闭包按判法收一次；按行进键的登记表随入口文件换（row_keyed_closure），键按入口文件各算
                    self.judge_closures[closure_slot] = self.script_closure(root, judge) if entry_language(judge) in ("python", "shell") else \
                        closure_of_entry(self, root, Request(judge, judge, []), judge)[0]
                closure = library.row_keyed_closure(self.judge_closures[closure_slot], root, entry_file)
                key, refusals = library.closure_key(closure)
                self.judge_keys[cache_key] = (key, closure_unit_lines(closure, prefix="judge "), [refusal.text() for refusal in refusals])
            except UnitKeyError as error:
                self.judge_keys[cache_key] = (None, [], [f"refused\tkey-uncomputable\t{judge}\t{error}"])
        return self.judge_keys[cache_key]

    # Rust 那一侧
    def workspace(self, closure_root):
        if closure_root not in self.workspaces:
            library = function_units()
            try:
                # 这里算的单元键都判产品行为：Rust 单元取产品行为视图，verus! 里的规格与证明不进键（lib_function_units.py 文件头「判定种类」）
                self.workspaces[closure_root] = library.RustWorkspace(closure_root, library.PRODUCT_BEHAVIOR)
            except library.FunctionUnitsError as error:
                raise UnitKeyError(f"Rust 工作区读不出（{error}）") from error
        return self.workspaces[closure_root]

    def host_triple(self, closure_root):
        if self.host is None:
            for line in self.rust_toolchain_lines(closure_root):
                if line.startswith("rustc -vV: host: "):
                    self.host = line[len("rustc -vV: host: "):]
            self.host = self.host or "x86_64-unknown-linux-gnu"
        return self.host

    def rust_toolchain_lines(self, closure_root):
        """工具链行：rustc -vV、cargo -V、构建环境的生效值（环境变量、各层 .cargo 配置合成之后的键、rust-toolchain、runner 与 wrapper 指的程序的内容摘要）。
        不进单元键（compute_unit_key 不读）：host_triple 从它取目标，scripts/experiment-apparatus/device-code-key.py 拿它算装置的代码键。"""
        if closure_root in self.toolchains:
            return self.toolchains[closure_root]
        lines = []
        code, output = run_text(["rustc", "-vV"], closure_root, self.environment)
        if code != 0 or not output:
            raise UnitKeyError(f"rustc -vV 退 {code}（工具链起不来，工具链行算不出）")
        lines += [f"rustc -vV: {line}" for line in output.split("\n") if line.strip()]
        code, output = run_text(["cargo", "-V"], closure_root, self.environment)
        if code != 0 or not output:
            raise UnitKeyError(f"cargo -V 退 {code}")
        lines.append(f"cargo -V: {output}")
        if not break_is_set("skip-build-environment"):
            try:
                lines += build_environment_setting_lines(closure_root, self.environment)
            except InputManifestError as error:
                raise UnitKeyError(f"构建环境读不出：{error}") from error
        self.toolchains[closure_root] = sorted(lines)
        return self.toolchains[closure_root]

    def lock(self, closure_root):
        if closure_root not in self.locks:
            path = os.path.join(closure_root, "Cargo.lock")
            try:
                with open(path, "rb") as handle:
                    self.locks[closure_root] = tomllib.load(handle).get("package") or []
            except (OSError, ValueError):
                self.locks[closure_root] = None
        return self.locks[closure_root]

    def dependency_lines(self, closure_root, workspace, closure, entry_target):
        """解析出的依赖版本行（不进单元键；scripts/experiment-apparatus/device-code-key.py 调）：闭包里有单元的工作区包，各自 Cargo.toml 写的外部依赖（入口是测试编译的另加入口那个包的 dev-dependencies），
        照 Cargo.lock 顺着传递依赖收（名字、版本、来源、checksum）；外部包自己的 Cargo.toml 在 cargo 缓存里找得到时，只在别的平台才要的依赖不收。
        Cargo.lock 不在交一行「Cargo.lock 不在」（不在也是一种取值）。"""
        packages = self.lock(closure_root)
        if packages is None:
            return ["Cargo.lock 不在或读不出"]
        crates, referenced = {}, set()
        for unit in closure.order:
            if getattr(unit, "language", None) != "rust" or getattr(unit, "data", False):
                continue
            crate = workspace.crate_of_path(unit.file)
            if crate is not None:
                crates[crate.directory] = crate
            referenced |= set(getattr(unit, "plain", None) or ())
            referenced |= {pair[0] for pair in (getattr(unit, "qualified", None) or ()) if isinstance(pair, tuple) and pair}
        entry_crate_directory = workspace.targets[entry_target].crate.directory if entry_target in workspace.targets else None
        entry_is_test = (entry_target is not None and entry_target.split(":")[1:2] in (["test"], ["bench"])) or any(line.startswith("test@") for line in closure.cfg_lines)
        by_name = collections.defaultdict(list)
        for package in packages:
            by_name[package.get("name")].append(package)
        wanted, lines, seen = [], set(), set()
        for directory, crate in sorted(crates.items()):
            declared = manifest_dependency_names(closure_root, directory, include_dev=entry_is_test and directory == entry_crate_directory,
                                                 host=self.host_triple(closure_root))
            # 闭包里有单元点了它的名字（use、路径、extern crate）的才收：同一个包里别的二进制用的外部包，改了不让这一条作废（弄坏开关 dependencies-by-crate 下照包收）
            allowed = {package for code_name, package in declared.items() if code_name in referenced or break_is_set("dependencies-by-crate")}
            own = [package for package in by_name.get(crate.package, ()) if not package.get("source")]
            if not own:
                continue
            for dependency in own[0].get("dependencies") or []:
                if dependency.split()[0] in allowed:
                    wanted.append(dependency)
        while wanted:
            dependency = wanted.pop()
            found = resolve_lock_dependency(by_name, dependency)
            if found is None:
                lines.add(f"Cargo.lock 里找不到 {dependency}")
                continue
            identity = (found.get("name"), found.get("version"), found.get("source"))
            if identity in seen:
                continue
            seen.add(identity)
            if not found.get("source"):
                continue   # 工作区里的包：它的单元在闭包里就由单元管，不在就没被用到
            lines.add(f"{found.get('name')} {found.get('version')} {found.get('source')} {found.get('checksum', '-')}")
            kept = registry_dependency_filter(self.environment, found, self.host_triple(closure_root))
            for next_dependency in found.get("dependencies") or []:
                if kept is None or next_dependency.split()[0] in kept:
                    wanted.append(next_dependency)
        return sorted(lines)

    def rows(self):
        if self.registration_rows is None:
            self.registration_rows = read_registration_rows(self.root)
        return self.registration_rows


def resolve_lock_dependency(by_name, dependency):
    """Cargo.lock 里一条依赖的写法（「名字」「名字 版本」「名字 版本 (来源)」）对到那一个包；对不上交 None。"""
    parts = dependency.split()
    candidates = by_name.get(parts[0], [])
    if len(parts) >= 2:
        candidates = [package for package in candidates if package.get("version") == parts[1]]
    if len(parts) >= 3:
        source = dependency.split("(", 1)[1].rstrip(")") if "(" in dependency else None
        candidates = [package for package in candidates if package.get("source") == source] or candidates
    return candidates[0] if len(candidates) == 1 else (candidates[0] if candidates and len(parts) >= 2 else None)


def host_cfg_value(expression, host):
    """target.'cfg(…)' 的谓词在本机目标上真不真：True / False / None（认不出：当真，多收一项依赖）。"""
    facts = {"target_os": "linux", "target_family": "unix", "target_arch": host.split("-")[0], "target_env": "gnu", "target_vendor": "unknown",
             "target_pointer_width": "64", "target_endian": "little"}
    names = {"unix": True, "windows": False, "test": False, "debug_assertions": None, "miri": False}
    tokens = re.findall(r'[A-Za-z_][A-Za-z0-9_]*|"[^"]*"|[(),=]', expression)
    position = [0]

    def peek():
        return tokens[position[0]] if position[0] < len(tokens) else None

    def take():
        position[0] += 1
        return tokens[position[0] - 1]

    def parse():
        word = take()
        if word in ("all", "any", "not") and peek() == "(":
            take()
            values = []
            while peek() not in (")", None):
                values.append(parse())
                if peek() == ",":
                    take()
            take()
            if word == "not":
                return None if not values or values[0] is None else not values[0]
            if word == "all":
                return False if False in values else (None if None in values else True)
            return True if True in values else (None if None in values else False)
        if peek() == "=":
            take()
            value = take().strip('"')
            if word not in facts:
                return None
            return facts[word] == value
        return names.get(word)
    try:
        return parse()
    except IndexError:
        return None


def target_section_applies(specification, host):
    if specification.startswith("cfg(") and specification.endswith(")"):
        return host_cfg_value(specification[4:-1], host) is not False
    return specification == host


def manifest_dependency_names(root, directory, include_dev, host):
    """工作区包 Cargo.toml 写的外部依赖：{代码里用的名字（键，- 换成 _）: 包名（package = 改名的取真名）}；本机目标不要的 target 段不收。"""
    sections = manifest_sections(os.path.join(root, directory or ".", "Cargo.toml")) or {}
    shared = ((manifest_sections(os.path.join(root, "Cargo.toml")) or {}).get("workspace") or {}).get("dependencies") or {}
    tables = [sections.get("dependencies") or {}, sections.get("build-dependencies") or {}]
    if include_dev:
        tables.append(sections.get("dev-dependencies") or {})
    for specification, table in (sections.get("target") or {}).items():
        if isinstance(table, dict) and target_section_applies(specification, host):
            tables += [table.get("dependencies") or {}, table.get("build-dependencies") or {}]
            if include_dev:
                tables.append(table.get("dev-dependencies") or {})
    names = {}
    for table in tables:
        for name, value in table.items():
            if isinstance(value, dict) and value.get("workspace") is True:
                value = dict(shared.get(name, {}) if isinstance(shared.get(name), dict) else {}, **{key: item for key, item in value.items() if key != "workspace"})
            if isinstance(value, dict) and "path" in value:
                continue
            names[name.replace("-", "_")] = value.get("package", name) if isinstance(value, dict) else name
    return names


def registry_dependency_filter(environment, package, host):
    """外部包自己的 Cargo.toml（cargo 缓存里 registry/src/*/<名>-<版本>/）：本机目标要的依赖名集合；找不到交 None（照 Cargo.lock 全收）。"""
    cargo_home = environment.get("CARGO_HOME") or os.path.join(environment.get("HOME") or os.path.expanduser("~"), ".cargo")
    matches = glob.glob(os.path.join(cargo_home, "registry", "src", "*", f"{package.get('name')}-{package.get('version')}", "Cargo.toml"))
    if not matches:
        return None
    sections = manifest_sections(matches[0])
    if sections is None:
        return None
    names = set()
    tables = [sections.get("dependencies") or {}, sections.get("build-dependencies") or {}]
    for specification, table in (sections.get("target") or {}).items():
        if isinstance(table, dict) and target_section_applies(specification, host):
            tables += [table.get("dependencies") or {}, table.get("build-dependencies") or {}]
    for table in tables:
        for name, value in table.items():
            names.add(value.get("package", name) if isinstance(value, dict) else name)
    return names


def rust_cfg_for_profile(closure_root, profile):
    """入口目标的生效 cfg：库的默认那一组，release 这一类不开 debug_assertions 的档去掉它（[profile.<名>] 写了 debug-assertions = true 的照留）。"""
    default = list(function_units().DEFAULT_RUST_CFG)
    if profile in (None, "dev", "test"):
        return default
    sections = manifest_sections(os.path.join(closure_root, "Cargo.toml")) or {}
    if ((sections.get("profile") or {}).get(profile) or {}).get("debug-assertions") is True:
        return default
    return [item for item in default if item != "debug_assertions"]


def closure_of_entry(context, closure_root, request, entry):
    """一个入口的闭包：交 (闭包, 目标号或 None, 工作区或 None)。入口找不到、cfg 下不编抛 UnitKeyError。"""
    library = function_units()
    language = entry_language(entry)
    if language is None:
        raise UnitKeyError(f"认不出入口 {entry} 是哪种语言（写 <文件>::<限定名>、<文件>，或 Rust 的 <crate>::<模块…>::<名>）")
    try:
        if language == "rust":
            workspace = context.workspace(closure_root)
            path, _separator, qualified = entry.partition("::")
            path, qualified = (None, entry) if "/" not in path and not path.endswith(".rs") else (path, qualified)
            unit, target_id = workspace.find_entry(path, qualified, request.target)
            cfg = rust_cfg_for_profile(closure_root, request.profile) + list(request.cfg)
            closure = context.rust_closure(closure_root, workspace, unit, target_id, cfg, request.features, request.no_default_features)
            return closure, target_id, workspace
        if language in ("python", "shell"):
            return context.script_closure(closure_root, entry, request.program_arguments), None, None
        closure = library.closure_of(closure_root, entry, library.Options())
        return closure, None, None
    except (library.FunctionUnitsError, library.UsageError) as error:
        raise UnitKeyError(f"{entry}：{error}") from error


# 组成清单里单元那一行按（前缀, 单元）共用一个串对象：test-plan 一趟几千条测试函数的闭包大半重叠，各条的组成清单只多存指针
UNIT_LINE_TEXTS = {}


def closure_unit_lines(closure, prefix=""):
    shared = not break_is_set("composition-lines-unshared")
    lines = []
    for unit in sorted(closure.order, key=lambda unit: unit.sort_key()):
        text = UNIT_LINE_TEXTS.get((prefix, unit)) if shared else None
        if text is None:
            text = prefix + unit.text()
            if shared:
                UNIT_LINE_TEXTS[(prefix, unit)] = text
        lines.append(text)
    return lines


def domain_variable_form(name):
    """名字在 DOMAIN_ENVIRONMENT_VARIABLES 里（或以登记的前缀起头、不以 THREADS 收尾）：交（登记的那一项，怎么进键）；不在交 None。"""
    for registered, form in DOMAIN_ENVIRONMENT_VARIABLES:
        if registered.endswith("_"):
            if name.startswith(registered) and name != registered and not name.endswith("THREADS"):
                return registered, form
        elif name == registered:
            return registered, form
    return None


def domain_variable_tokens(environment):
    """test-functions-request 打的 variable: 那几项（文件头「判定请求」）：environment 里设了的登记变量，照名字排好；取值里的制表符与换行照 \\t \\n 转写
    （请求一行一条、按制表符切）。弄坏开关 domain-environment-unkeyed 下一项都不打。"""
    if break_is_set("domain-environment-unkeyed"):
        return []
    escaped = {"\t": "\\t", "\n": "\\n"}
    return ["variable:" + name + "=" + "".join(escaped.get(character, character) for character in environment[name])
            for name in sorted(environment) if domain_variable_form(name) is not None]


def closure_literals(context, closure_root, closure):
    """闭包里各单元原文（文件里的起止行）里的大写字面串（"NAME" 或 'NAME'）：每个单元只读一次（KeyContext 里按（根，文件，起止行）存）；
    数据单元与读不了的文件不算。"""
    found = set()
    for unit in getattr(closure, "order", ()):
        if getattr(unit, "data", False) or not unit.file or not unit.first_line or not unit.last_line:
            continue
        cache_key = (closure_root, unit.file, unit.first_line, unit.last_line)
        literals = context.unit_literals.get(cache_key)
        if literals is None:
            path = os.path.join(closure_root, unit.file)
            if path not in context.source_lines:
                try:
                    with open(path, encoding="utf-8", errors="surrogateescape") as handle:
                        context.source_lines[path] = handle.read().split("\n")
                except OSError:
                    context.source_lines[path] = []
            text = "\n".join(context.source_lines[path][unit.first_line - 1:unit.last_line])
            literals = context.unit_literals[cache_key] = frozenset(DOMAIN_VARIABLE_LITERAL.findall(text))
        found |= literals
    return found


def domain_variable_lines(context, closure_root, closure, request):
    """单元键的 variable 行（文件头「单元键」）：[(键行, [组成清单行…])]。登记的变量只在闭包读它（单元原文里有那个名字的字面串，前缀登记的有以它起头的字面串）
    时进；取值取请求的 variable:（没有写 unset）。按文件登记的进内容 sha256，另记一行 file 组成（verdict-record 拿它判跑的过程中那份文件动没动）。"""
    literals = closure_literals(context, closure_root, closure)
    out = []
    for registered, form in DOMAIN_ENVIRONMENT_VARIABLES:
        extra = []
        if registered.endswith("_"):
            if not any(literal.startswith(registered) for literal in literals):
                continue
            settings = sorted((name, value) for name, value in request.variables.items() if domain_variable_form(name) == (registered, form))
            text = " ".join(f"{name}={value}" for name, value in settings) or "unset"
        else:
            if registered not in literals:
                continue
            value = request.variables.get(registered)
            if value is None:
                text = "unset"
            elif form == "file":
                path = value if os.path.isabs(value) else os.path.join(context.root, value)
                text = sha256_of_file(path) if os.path.isfile(path) else "不在"
                shown = os.path.relpath(path, context.root) if os.path.abspath(path).startswith(context.root + os.sep) else os.path.abspath(path)
                extra = [f"file\t{shown}\t{text}"]
            else:
                text = value
        out.append((f"variable\t{registered}\t{text}", [f"variable\t{registered}\t{text}"] + extra))
    return out


def not_judging_files_of_entry(library, path):
    """这一条闭包按角色排除哪几份（文件头「单元键」按角色排除）：入口文件 path 在 NOT_JUDGING_FILES 里的不排除，别的排除那两份。
    弄坏开关 not-judging-files-keyed 下一律不排除，not-judging-entries-excluded 下入口在那两份里的也排除。"""
    if break_is_set("not-judging-files-keyed"):
        return ()
    if path in library.NOT_JUDGING_FILES and not break_is_set("not-judging-entries-excluded"):
        return ()
    return library.NOT_JUDGING_FILES


def request_entry_file(request):
    """请求的入口文件（相对根）：<文件>::<限定名> 取文件，root: 给了的接在它后面；不是文件的入口（tests-of:、vm: 这一类）交 None。"""
    file = request.entry.split("::", 1)[0]
    if ":" in file:
        return None
    return os.path.normpath(os.path.join(request.root, file))


def compute_unit_key(context, request):
    """一条判定请求的单元键（键的组成见文件头「单元键」：只由代码单元与内容输入组成，工具链、解释器、外部程序与依赖的版本都不进）：交 UnitKey。
    算不出（库载不进、入口找不到、在这一组 cfg 下不编）的交拒算（key 是 None），不抛。"""
    kb_documents_in_key = break_is_set("experiment-key-with-kb-documents")
    closure_root = os.path.normpath(os.path.join(context.root, request.root))
    lines, composition, refusals = [], [], []
    judge_only = request.entry == JUDGE_ONLY_ENTRY
    closure = None
    try:
        library = function_units()
        if judge_only:
            # 入口写 -（文件头「判定请求」）：被判那棵树里没有程序入口，键只由判法与内容输入组成；一个判法都没有就没有可比的东西，拒算
            if not request.judges:
                raise UnitKeyError(f"入口写 {JUDGE_ONLY_ENTRY} 的请求至少要一个 judge:（键只由判法与内容输入组成）")
            lines.append(f"entry\t{JUDGE_ONLY_ENTRY}")
        else:
            closure, _target_id, _workspace = closure_of_entry(context, closure_root, request, request.entry)
            entry_key, entry_refusals = library.closure_key(closure)
            refusals += [refusal.text() for refusal in entry_refusals]
            lines.append(f"entry\t{entry_key}")
            composition += closure_unit_lines(closure)
            composition += [f"cfg\t{item}" for item in sorted(closure.cfg_lines)]
        for judge in request.judges:
            if judge_only and break_is_set("judge-only-entry-judges-unkeyed"):   # 弄坏开关：入口写 - 的请求判法不进键
                continue
            judge_key, judge_composition, judge_refusals = context.judge_key(judge, request.judge_root, None if judge_only else request_entry_file(request))
            refusals += judge_refusals
            lines.append(f"judge\t{judge_key}")
            present = set(composition)
            composition += [line for line in judge_composition if line not in present]
    except UnitKeyError as error:
        return UnitKey(None, composition, refusals + [f"refused\tkey-uncomputable\t{request.entry}\t{error}"])
    if request.program_arguments is not None and not break_is_set("program-arguments-unkeyed"):
        # 程序入口带的参数照次序进一行（文件头「单元键」program-arguments）：闭包相同、参数不同的两条判定不共用键（弄坏开关 program-arguments-unkeyed 下不进）
        lines.append("program-arguments\t" + json.dumps(request.program_arguments, ensure_ascii=False))
        composition.append("program-arguments\t" + json.dumps(request.program_arguments, ensure_ascii=False))
    for text in request.lines:
        lines.append(f"line\t{text}")
        composition.append(f"line\t{text}")
    for relative in request.files:
        path = os.path.join(context.root, relative)
        digest = sha256_of_file(path) if os.path.isfile(path) else "不在"
        lines.append(f"file\t{digest}")
        composition.append(f"file\t{relative}\t{digest}")
    if closure is not None and not break_is_set("domain-environment-unkeyed"):
        for key_line, composition_lines in domain_variable_lines(context, closure_root, closure, request):
            lines.append(key_line)
            composition += composition_lines
    if kb_documents_in_key:
        # 弄坏开关 experiment-key-with-kb-documents：.claude/kb/decisions/ 下的文件整份进键（kb 决策文档进指纹）
        for path in sorted(glob.glob(os.path.join(context.root, ".claude", "kb", "decisions", "*"))):
            if os.path.isfile(path):
                lines.append(f"file\t{sha256_of_file(path)}")
    if refusals:
        return UnitKey(None, composition, sorted(set(refusals)))
    return UnitKey(hashlib.sha256("".join(line + "\n" for line in sorted(lines)).encode("utf-8", "surrogateescape")).hexdigest(), composition, [])


def changed_composition(earlier, later, shown_at_most=CHANGED_UNITS_SHOWN_AT_MOST):
    """两份组成清单比出变了什么：单元按（种类、限定名）比指纹，别的行按原文比。交 (说明句的清单, 共几处)。"""
    def split(lines):
        units, others = {}, set()
        for line in lines:
            parts = line.split("\t")
            if len(parts) == 4 and parts[0] != "file" and ":" in parts[2]:
                units[(parts[0], parts[1], parts[2].split(":", 1)[0])] = parts[3]
            else:
                others.add(line)
        return units, others
    earlier_units, earlier_others = split(earlier)
    later_units, later_others = split(later)
    notes = []
    for identity in sorted(set(earlier_units) | set(later_units)):
        kind, name, file = identity
        if identity not in earlier_units:
            notes.append(f"多了 {kind} {file}::{name}")
        elif identity not in later_units:
            notes.append(f"少了 {kind} {file}::{name}")
        elif earlier_units[identity] != later_units[identity]:
            notes.append(f"变了 {kind} {file}::{name}")
    for line in sorted(later_others - earlier_others):
        notes.append(f"新的输入行 {line.replace(chr(9), ' ')[:160]}")
    for line in sorted(earlier_others - later_others):
        notes.append(f"没了的输入行 {line.replace(chr(9), ' ')[:160]}")
    return notes[:shown_at_most], len(notes)


# ── 判定库（每类判定一份，固定在 git common-dir 下；文件头「判定库」）────────────────────────────────────────

LATEST_COMPOSITIONS_KEPT = 3   # 一个名字最近算出的几把键留着组成清单（VerdictLibrary 文档串「组成清单的回收」）：跑的过程中键变了要拿开跑前那一把与此刻那一把比


class VerdictLibrary:
    """一种类一份，在 prune-server 结论表里（文件头「判定库」）：键 = unit-verdicts[@命名空间]/<种类目录名>/<相对路径>
    （scripts/prune-server/prune_server_client.py 文件头「键空间」）。keys/<键>.json 是一条判定（绿或红：record 写绿、record_red 写红，后写的盖掉先写的），
    items/<名字的 sha256>.json 是这个名字最近一次判绿的那一份，
    compositions/<键>.tsv.gz 是那把键的组成清单（按键存一份，算过键就写，只当索引：键对不上时拿它与此刻的比，报出变了的单元）。
    组成清单的回收：一个名字留着的组成清单是它此刻判绿的那一把（items/ 记的）与它最近算出的 LATEST_COMPOSITIONS_KEPT 把
    （latest-compositions/<名字的 sha256>.txt 记的，一行一把、旧的在前）；新写一份组成清单时挤出去的、判绿换键时换下来的交 drop_composition，
    四样都对才删：键是 64 位十六进制、不在最近算出的几把里、不是这个名字此刻判绿的那一把、keys/<键>.json 没记着别的名字。
    只删 compositions/ 那一份：keys/ 与 items/（判定本身）不碰，删掉组成清单的键照旧复用，下一次算到它时组成清单重写。
    两种情形会删掉别人还要比的那一份，后果都只是「键对不上」的原因里少「变了哪几个单元」那一句，红绿、复用还是现判、退出码不变：
    两个名字共用一把键（keys/<键>.json 只记最后记它的那个名字）时，最后记它的那个名字换键；核四样与删之间别的进程刚把这把键记成判绿。
    directory（<git common-dir>/singlefs-unit-verdicts/<种类>）只当这份库的名字（与临时服务底下的命名空间），本地不建、不读、不写；取不到 git common-dir、
    这棵树里没有客户端退 2（LibraryUnavailable）。服务连不上客户端退 75，调用方不当「库里没有」现判。判绿那一条的键文件与 items/ 那一份放同一个写帧。"""

    def __init__(self, root, kind):
        self.kind = kind
        common_directory = git_common_directory(root)
        if common_directory is None:
            if break_is_set("library-outside-git-tolerated"):   # 弄坏开关：取不到 common-dir 就没有库（store 是 None）
                self.directory = self.store = None
                return
            raise LibraryUnavailable(f"{os.path.abspath(root)} 不是 git 工作树（取不到 git common-dir），判定库没有名字，一条都没判",
                                     "在 git 工作树里跑；rsync 拷的仓副本先在副本根 git init（键里不带哪棵仓，副本与主仓读写同一份库）")
        self.directory = os.path.join(common_directory, VERDICT_LIBRARY_DIRECTORY_NAME, kind_directory_name(kind))
        self.store = self.service_store(self.directory, kind_directory_name(kind))

    @staticmethod
    def client():
        """scripts/prune-server/prune_server_client.py（判定库服务的客户端），照它的名字只载入一次；载入不连服务。
        拷进不带它的小树里跑的这一份交 None（service_store 与 file_library_store 照它退 2）。"""
        name = "singlefs_prune_server_client"
        if name not in sys.modules:
            path = os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "prune-server", "prune_server_client.py")
            if not os.path.isfile(path):
                return None
            specification = importlib.util.spec_from_file_location(name, path)
            module = importlib.util.module_from_spec(specification)
            sys.modules[name] = module
            specification.loader.exec_module(module)
        return sys.modules[name]

    @staticmethod
    def service_store(directory, kind_name):
        """这一种类在 prune-server 里的 LibraryStore；这棵树里没有客户端退 2（LibraryUnavailable）。"""
        client = VerdictLibrary.client()
        if client is None:
            raise LibraryUnavailable("这棵树里没有 scripts/prune-server/prune_server_client.py", "在带着 scripts/prune-server/ 的整棵仓里跑：判定库只经 prune-server 的客户端读写")
        return client.library_store(client.FAMILY_UNIT_VERDICTS, kind_name, directory)

    def load_stored_bytes(self, relative):
        """这份库里一份记录的原样字节（relative 是 keys/<键>.json 这类相对路径）；不在交 None，服务不可用客户端退 75。"""
        return self.store.read(relative)

    def load_stored_json(self, relative):
        data = self.load_stored_bytes(relative)
        if data is None:
            return None
        try:
            return json.loads(data.decode("utf-8"))
        except ValueError:
            return None

    def stored_keys(self):
        """库里有记录的键（排好序）：自证核「写进了哪一份库」用。"""
        names = [relative[len("keys/"):] for relative, _value in self.store.scan("keys/")]
        return sorted(name[:-len(".json")] for name in names if name.endswith(".json"))

    @staticmethod
    def path_of_key(key):
        return f"keys/{key}.json"

    @staticmethod
    def path_of_item(name):
        return f"items/{hashlib.sha256(name.encode('utf-8', 'surrogateescape')).hexdigest()}.json"

    @staticmethod
    def path_of_composition(key):
        return f"compositions/{key}.tsv.gz"

    def read(self, relative):
        return self.green_record(self.load_stored_json(relative))

    @staticmethod
    def green_record(record):
        # 只认判绿的记录：键相同而库里记着红的（或别的）不当判定，那一条下一趟照现判。弄坏开关 red-verdict-reused 下不看判定一律当判定读（文件头「弄坏开关」）
        return record if isinstance(record, dict) and (record.get("verdict") == "green" or break_is_set("red-verdict-reused")) else None

    @staticmethod
    def path_of_latest_composition(name):
        return f"latest-compositions/{hashlib.sha256(name.encode('utf-8', 'surrogateescape')).hexdigest()}.txt"

    def latest_composition_keys(self, name):
        """这个名字最近算出的几把键（旧的在前，至多 LATEST_COMPOSITIONS_KEPT 把；一行一把）。"""
        data = self.load_stored_bytes(self.path_of_latest_composition(name))
        return [line for line in data.decode("utf-8", "replace").split("\n") if line] if data else []

    def store_composition(self, unit_key, name=None):
        """组成清单按键存一份进 prune-server（已经在就不写）。给了 name（判定请求的名字）时照文件头「判定库」组成清单的回收：
        这一把（新写的与早就在的一样）挪到这个名字最近算出的几把键的末尾，挤出去的那几把交 drop_composition；已经在末尾的不写。"""
        if unit_key.key is None:
            return
        relative = self.path_of_composition(unit_key.key)
        if self.store.read(relative) is None:
            self.store.write_many([(relative, gzip.compress("".join(line + "\n" for line in composition_lines(unit_key.composition)).encode("utf-8", "surrogateescape"), mtime=0))])
        if name is None:
            return
        earlier = self.latest_composition_keys(name)
        if earlier and earlier[-1] == unit_key.key:
            return
        latest = [key for key in earlier if key != unit_key.key] + [unit_key.key]
        kept, pushed_out = latest[-LATEST_COMPOSITIONS_KEPT:], latest[:-LATEST_COMPOSITIONS_KEPT]
        self.store.write_many([(self.path_of_latest_composition(name), "".join(key + "\n" for key in kept))])
        for key in pushed_out:
            self.drop_composition(name, key, kept)

    def drop_composition(self, name, key, kept):
        """删 name 用过的一把键的组成清单，四样都对才删（文件头「判定库」组成清单的回收），只删 compositions/ 那一份，keys/ 与 items/ 不碰：
        键是 64 位十六进制；不在 kept（这个名字最近算出的几把）里；不是这个名字此刻判绿的那一把；keys/<键>.json 没记着别的名字。
        弄坏开关 superseded-composition-kept 下一把都不删。"""
        if break_is_set("superseded-composition-kept") or re.fullmatch(r"[0-9a-f]{64}", key) is None or key in kept:
            return
        green = self.load_stored_json(self.path_of_item(name))
        if isinstance(green, dict) and green.get("key") == key:
            return
        verdict = self.load_stored_json(self.path_of_key(key))
        if isinstance(verdict, dict) and verdict.get("item") != name:
            return
        self.store.delete_prefix(self.path_of_composition(key))

    def composition_of(self, key):
        data = self.load_stored_bytes(self.path_of_composition(key))
        if data is None:
            return None
        try:
            return gzip.decompress(data).decode("utf-8", "surrogateescape").rstrip("\n").split("\n")
        except (EOFError, gzip.BadGzipFile, OSError):
            return None

    def lookup(self, request, unit_key):
        """交 (决定, 原因)：决定是 reuse、judge、refused 之一。"""
        if unit_key.key is None:
            return "refused", "拒算：" + "；".join(line.split("\t", 1)[1].replace("\t", " ") for line in unit_key.refusals)
        if break_is_set("verdict-lookup-skipped"):   # 弄坏开关：不读库（复用失灵的样子），自证 reuse-three-questions 与 reuse-sentinel 那两格转红
            return "judge", "弄坏开关 verdict-lookup-skipped：不读库"
        record = self.read(self.path_of_key(unit_key.key))
        if record is not None:
            return "reuse", f"键相同的判定（{record.get('judged', '没记日期')}，{record.get('how', '没记跑法')}）"
        red = self.read_red(self.path_of_key(unit_key.key), unit_key.key)
        if red is not None:
            return "judge", f"{RED_VERDICT_REASON}（{red.get('judged', '没记日期')}，{red.get('how', '没记跑法')}）：键相同的判定是红，现判"
        earlier = self.read(self.path_of_item(request.name))
        if earlier is None:
            return "judge", "库里没有这一条"
        earlier_composition = self.composition_of(earlier.get("key", ""))
        if earlier_composition is None:
            return "judge", f"键对不上（上一次 {earlier.get('judged', '没记日期')} 判绿，那一份的组成清单不在库里）"
        notes, total = changed_composition(earlier_composition, composition_lines(unit_key.composition))
        return "judge", f"键对不上（上一次 {earlier.get('judged', '没记日期')} 判绿），变了 {total} 处：{'；'.join(notes) or '组成清单相同、键的写法变了'}"

    def read_red(self, relative, key):
        """键文件记着这把键判红（record_red 写的）就交那一份，不然交 None。"""
        record = self.load_stored_json(relative)
        return record if isinstance(record, dict) and record.get("verdict") == "red" and record.get("key") == key else None

    def record(self, request_text, name, key, how, summary=""):
        """判绿之后记一条：键、判定日期、跑法、请求原文（组成清单在 compositions/ 里按键另存）；键那一份与 items/ 那一份放一个写帧（键那一份盖掉这把键原有的红）。
        没记交原因，记了交空串；服务不可用客户端退 75。"""
        if not key:
            return "这一条拒算，不记"
        record = {"verdict": "green", "key": key, "item": name, "request": request_text, "how": how, "summary": summary, "judged": judged_date()}
        text = json.dumps(record, ensure_ascii=False, indent=1) + "\n"
        earlier = self.load_stored_json(self.path_of_item(name))
        self.store.write_many([(self.path_of_key(key), text), (self.path_of_item(name), text)])
        # 这个名字判绿的那把键换了：换下来的那一把交 drop_composition（文件头「判定库」组成清单的回收；还在最近算出的几把里的不删）
        earlier_key = earlier.get("key", "") if isinstance(earlier, dict) else ""
        if isinstance(earlier_key, str) and earlier_key and earlier_key != key:
            self.drop_composition(name, earlier_key, self.latest_composition_keys(name))
        return ""

    def record_red(self, request_text, name, key, how, summary=""):
        """判红之后记一条（文件头「判定库」记红、撤绿、下趟现判）：照同一把键写 verdict=red 的 keys/<键>.json，盖掉这把键原有的绿；items/ 那一份
        （这个名字最近判绿的那一份）不动。没记交原因，记了交空串。弄坏开关 red-verdict-not-recorded 下不写。"""
        if not key:
            return "这一条拒算，不记"
        if break_is_set("red-verdict-not-recorded"):
            return ""
        record = {"verdict": "red", "key": key, "item": name, "request": request_text, "how": how, "summary": summary, "judged": judged_date()}
        self.store.write_many([(self.path_of_key(key), json.dumps(record, ensure_ascii=False, indent=1) + "\n")])
        return ""

    def withdraw_green(self, name, key):
        """记红没成时撤掉这把键原有的绿（文件头「按条复用失灵」，plan-failure --red 调）：keys/<键>.json 记着绿才删（记着红的不动），
        items/ 那一份记的也是这把键就一起删；删完回读，还读得到这把键的绿就抛 RuntimeError。交撤掉的条数（0 或 1）。服务不可用客户端退 75。"""
        def green_of(relative):
            record = self.load_stored_json(relative)
            return isinstance(record, dict) and record.get("verdict") == "green" and record.get("key") == key
        withdrawn = 0
        if green_of(self.path_of_key(key)):
            self.store.delete_prefix(self.path_of_key(key))
            withdrawn = 1
        if green_of(self.path_of_item(name)):
            self.store.delete_prefix(self.path_of_item(name))
        if green_of(self.path_of_key(key)):
            raise RuntimeError("删完回读还是这把键的绿")
        return withdrawn

    def items(self):
        """库里每个名字最近一次判绿的那一份：[(名字, 记录)]。"""
        found = []
        for relative, data in self.store.scan("items/"):
            if not relative.endswith(".json"):
                continue
            try:
                record = self.green_record(json.loads(data.decode("utf-8")))
            except ValueError:
                continue
            if record is not None:
                found.append((record.get("item"), record))
        return found


def kind_directory_name(kind):
    return re.sub(r"[^A-Za-z0-9._:-]", "-", kind)


JUDGED_DATE = []


def judged_date():
    if not JUDGED_DATE:
        environment = dict(os.environ, TZ="Asia/Tokyo")
        code, output = run_text(["date", "+%F"], "/", environment)
        JUDGED_DATE.append(output if code == 0 and output else time.strftime("%Y-%m-%d"))
    return JUDGED_DATE[0]


def write_atomically(path, content):
    """content 是 str 或 bytes；同目录建临时文件、写完 fsync、改名换上。写不成交原因，写成交空串。"""
    directory = os.path.dirname(path)
    try:
        os.makedirs(directory, exist_ok=True)
        handle_number, temporary = tempfile.mkstemp(dir=directory, prefix="." + os.path.basename(path) + ".partial.")
        try:
            with os.fdopen(handle_number, "wb") as handle:
                handle.write(content if isinstance(content, bytes) else content.encode("utf-8", "surrogateescape"))
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temporary, path)
        except OSError:
            os.unlink(temporary)
            raise
    except OSError as error:
        return f"写不了 {path}：{error}"
    return ""


class LibraryUnavailable(SystemExit):
    """判定库没处放（文件头「判定库」）：被判的根不是 git 工作树、这棵树里没有客户端。退 2（EXIT_REGISTRATION_ERROR）；抛之前在标准输出打 ✗ 与出路。
    是 SystemExit：调用方的 except Exception 拦不住，进程照 2 退。"""

    def __init__(self, message, remedy):
        print(f"  ✗ 判定库没处放：{message}")
        print(f"     → 怎么办：{remedy}")
        super().__init__(EXIT_REGISTRATION_ERROR)
        self.message = message

    def __str__(self):
        return self.message


def composition_file_names(composition):
    """组成清单里点到的文件：单元行的文件、数据行的文件。"""
    files = set()
    for line in composition:
        parts = line.split("\t")
        if len(parts) == 4 and ":" in parts[2]:
            files.add(parts[2].split(":", 1)[0])
        elif len(parts) == 3 and parts[0] == "file":
            files.add(parts[1])
    return files


def composition_packed(composition, line_ids):
    """组成清单压成一小段字节（文件头 test-plan「内存」）：每行记行表里的下标，行表里没有的记行表长度、原文另存；下标按与上一行的差存（单元行按文件与行号排好，差多半很小）；
    连同清单点到的文件名单（composition_file_names）pickle 之后 zlib。"""
    import array
    import pickle
    import zlib
    literal_mark = len(line_ids)
    differences, literals, previous = array.array("I"), [], 0
    for line in composition:
        position = line_ids.get(line, literal_mark)
        if position == literal_mark:
            literals.append(line)
        differences.append((position - previous) & 0xFFFFFFFF)
        previous = position
    return zlib.compress(pickle.dumps((differences.tobytes(), literals, sorted(composition_file_names(composition))), protocol=pickle.HIGHEST_PROTOCOL), 1)


def composition_unpacked(packed, line_table):
    """composition_packed 反过来：→ (组成清单, 文件名单)。弄坏开关 test-plan-packing-literals-moved：行表外的那几行挪到最前面。"""
    import array
    import pickle
    import zlib
    encoded, literals, files = pickle.loads(zlib.decompress(packed))
    differences = array.array("I")
    differences.frombytes(encoded)
    literal_mark, position, lines, remaining_literals = len(line_table), 0, [], iter(literals)
    for difference in differences:
        position = (position + difference) & 0xFFFFFFFF
        lines.append(next(remaining_literals) if position == literal_mark else line_table[position])
    if break_is_set("test-plan-packing-literals-moved"):
        lines = literals + [line for line in lines if line not in literals]
    return lines, files


def composition_lines(composition):
    """UnitKey 的组成清单的行：test-plan 的子进程交回的是压过的字节，别处就是行的列表。"""
    return composition_unpacked(composition, TEST_PLAN_SHARED["line_table"])[0] if isinstance(composition, bytes) else composition


def composition_files(root, unit_key):
    """组成清单里点到的文件（单元行的文件、数据行的文件）各自此刻的 sha256：verdict-record 拿它判「跑的过程中这一条的输入动没动」，动了才重算键。
    只当这一趟的捷径，不进键。test-plan 那一趟（TEST_PLAN_SHARED 里有 file_digests）同一份文件一趟只算一次：几千条测试函数点到同一批文件。
    弄坏开关 test-plan-file-digests-reread：每条各算一遍。"""
    composition = unit_key.composition
    files = composition_unpacked(composition, TEST_PLAN_SHARED["line_table"])[1] if isinstance(composition, bytes) else composition_file_names(composition)
    read_digests = TEST_PLAN_SHARED.get("file_digests")
    if read_digests is None:
        read_digests = {}
    elif break_is_set("test-plan-file-digests-reread"):
        read_digests.clear()
    digests = {}
    for relative in [*sorted(files), "Cargo.toml", "Cargo.lock"]:
        path = os.path.join(root, relative)
        if path not in read_digests:
            read_digests[path] = sha256_of_file(path) if os.path.isfile(path) else "不在"
            TEST_PLAN_SHARED["file_digest_reads"] = TEST_PLAN_SHARED.get("file_digest_reads", 0) + 1
        digests[relative] = read_digests[path]
    return digests


REUSE_CHECKS_LIBRARY = "lib_reuse_checks.py"
SENTINEL_STATE_SUFFIX = ".reuse-sentinel.json"
UNIT_VERDICT_SAMPLE_DIRECTORY = ".unit-verdict-sample"
_REUSE_CHECKS = []


def reuse_checks_module():
    """同目录的 lib_reuse_checks.py（复用哨兵的判法，文件头「复用哨兵」）；拷进不带它的小树里跑的这一份交 None。"""
    if not _REUSE_CHECKS:
        path = os.path.join(os.path.dirname(os.path.realpath(__file__)), REUSE_CHECKS_LIBRARY)
        module = None
        if os.path.isfile(path):
            specification = importlib.util.spec_from_file_location("lib_reuse_checks_for_admission", path)
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
        _REUSE_CHECKS.append(module)
    return _REUSE_CHECKS[0]


def sentinel_candidates(library, keyed):
    """复用哨兵的候选：此刻键算得出、库里 keys/<键>.json 记着判绿、记的键与此刻的相同的请求 → {名字: "绿"}。
    与复用判法（VerdictLibrary.lookup）分开查：直接读 prune-server 里那一份键文件的原样，不经 lookup 与 read。"""
    candidates = {}
    for request, unit_key in keyed:
        if not unit_key.key:
            continue
        record = library.load_stored_json(f"keys/{unit_key.key}.json")
        if isinstance(record, dict) and record.get("verdict") == "green" and record.get("key") == unit_key.key:
            candidates[request.name] = "绿"
    return candidates


def plan_line(root, request, unit_key, decision):
    return json.dumps({"request": request.text(), "key": unit_key.key, "decision": decision,
                       "files": composition_files(root, unit_key) if unit_key.key else {}}, ensure_ascii=False)


def sha256_of_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def read_configuration_file(path):
    try:
        with open(path, "rb") as handle:
            return handle.read()
    except OSError as error:
        raise InputManifestError(f"读不了构建环境里的 {path}：{error}") from error

def resolved_program(program, base_directory, environment):
    """cargo 配置或环境变量里写的程序：带斜杠的按 base_directory 解开相对路径，不带斜杠的在 environment 的 PATH 里找；找不到交 None。"""
    if not program:
        return None
    if "/" in program:
        return program if os.path.isabs(program) else os.path.join(base_directory, program)
    return shutil.which(program, path=environment.get("PATH"))


def runner_argument_directories(root):
    """runner / wrapper 参数里的相对路径另从哪几个目录解：登记的每条崩溃枚举用例的包目录（cargo 起 runner 时当前目录是包目录）。
    交 [(仓根起的包目录, 绝对路径)]，按包目录排好；登记表读不了、写错的交空表（登记行本身另有自查）。弄坏开关
    runner-arguments-from-configuration-directory-only 下交空表（只从 .cargo 那一层 / 仓根解）。"""
    if break_is_set("runner-arguments-from-configuration-directory-only"):
        return []
    try:
        cases = crash_cases_of(read_registration_rows(root))
    except (RegistrationError, OSError):
        return []
    packages = workspace_package_directories(root)
    relatives = sorted({packages[case.package] for case in cases if case.package in packages})
    return [(relative, os.path.join(root, relative)) for relative in relatives]


def program_contents(words, base_directory, environment, argument_directories=()):
    """首词是程序、其后是参数的一个值：交回那个程序的内容（进指纹），找不到的交「找不到」一句（值本身另有一行进指纹）；
    首词之后指到现存文件的参数（绝对路径，或从 base_directory 起的相对路径）接着按内容进：`bash tools/r.sh` 这一类，真正定行为的脚本在参数里；
    相对路径另从 argument_directories（runner_argument_directories：登记用例的包目录，cargo 在那里起 runner）逐个解，解得到的都进，
    名字里写从哪个包目录解的（仓根起，不写绝对路径）。那个脚本再 source、再读的文件不进（文件头「管不到的」）。"""
    program = words[0] if words else ""
    path = resolved_program(program, base_directory, environment)
    contents = read_configuration_file(path) if path and os.path.isfile(path) else f"找不到：{program}".encode("utf-8", "surrogateescape")
    if break_is_set("runner-program-only"):
        return contents
    for argument in words[1:]:
        candidate = argument if os.path.isabs(argument) else os.path.join(base_directory, argument)
        if os.path.isfile(candidate):
            contents += b"\n<" + argument.encode("utf-8", "surrogateescape") + b">\n" + read_configuration_file(candidate)
        if os.path.isabs(argument):
            continue
        for relative_directory, directory in argument_directories:
            candidate = os.path.join(directory, argument)
            if os.path.isfile(candidate):
                contents += (b"\n<" + argument.encode("utf-8", "surrogateescape") + "，从包目录 ".encode("utf-8")
                             + relative_directory.encode("utf-8", "surrogateescape") + " 解>\n".encode("utf-8") + read_configuration_file(candidate))
    return contents


def compiler_version(program, root, environment, described_as):
    """编译器的 `-V`（在仓根跑）；起不来、退非 0、一个字不打都抛 InputManifestError。"""
    try:
        completed = subprocess.run([program, "-V"], cwd=root, capture_output=True, env=dict(environment))
    except OSError as error:
        raise InputManifestError(f"{described_as} 起不来：{error}") from error
    if completed.returncode != 0 or not completed.stdout.strip():
        raise InputManifestError(f"{described_as} -V 退 {completed.returncode}、打了「{completed.stdout.decode('utf-8', 'replace').strip()}」")
    return completed.stdout


def configured_program_lines(label, content, base_directory, root, environment, argument_directories=()):
    """一份 cargo 配置文件里指程序的键（build.rustc 取 -V；build.rustc-wrapper、build.rustc-workspace-wrapper、target.<…>.runner 按内容）：
    [(名字, 内容)]。base_directory 是 .cargo 所在的那一层（相对路径从它起）。读不成 TOML 的不另进（原文已经进了，cargo 也读不了它）。"""
    try:
        settings = tomllib.loads(content.decode("utf-8"))
    except (UnicodeDecodeError, ValueError):
        return []
    lines = []
    build = settings.get("build") if isinstance(settings.get("build"), dict) else {}
    compiler = build.get("rustc")
    if isinstance(compiler, str):
        lines.append((f"<构建环境：{label} 里 build.rustc 的 -V>",
                      compiler_version(resolved_program(compiler, base_directory, environment) or compiler, root, environment,
                                       f"{label} 里的 build.rustc = {compiler}")))
    for key in CONFIGURATION_PROGRAM_BUILD_KEYS:
        if isinstance(build.get(key), str):
            lines.append((f"<构建环境：{label} 里 build.{key} 指的程序>", program_contents([build[key]], base_directory, environment, argument_directories)))
    targets = settings.get("target") if isinstance(settings.get("target"), dict) else {}
    for target_name in sorted(targets):
        runner = targets[target_name].get("runner") if isinstance(targets[target_name], dict) else None
        words = runner.split() if isinstance(runner, str) else [str(word) for word in runner] if isinstance(runner, list) else []
        if words:
            lines.append((f"<构建环境：{label} 里 target.{target_name}.runner 指的程序>", program_contents(words, base_directory, environment,
                                                                                                      argument_directories)))
    return lines


def manifest_sections(path):
    try:
        with open(path, "rb") as handle:
            return tomllib.load(handle)
    except (OSError, ValueError):  # TOMLDecodeError 与编码错都是 ValueError
        return None


def workspace_package_directories(root):
    """仓根 Cargo.toml 的工作区成员（members 按 glob 展开；仓根自己是包也算）：包名 → 仓根起的目录。"""
    root_sections = manifest_sections(os.path.join(root, "Cargo.toml")) or {}
    directories = []
    if "package" in root_sections:
        directories.append(".")
    for pattern in (root_sections.get("workspace") or {}).get("members") or []:
        for directory in sorted(glob.glob(os.path.join(root, pattern))):
            directories.append(os.path.relpath(directory, root))
    packages = {}
    for directory in directories:
        name = ((manifest_sections(os.path.join(root, directory, "Cargo.toml")) or {}).get("package") or {}).get("name")
        if name:
            packages[name] = os.path.normpath(directory)
    return packages


# ── 构建环境的生效值（rust_toolchain_lines 那一截；不进单元键）────────────────────────────────────

def configuration_layers(closure_root, environment):
    """cargo 读的配置层，从近到远：闭包根（被判那棵树的工作区根）往上每一层的 .cargo/config(.toml)，最后 CARGO_HOME 的（最低；同时是某一层的只算一次）。
    读不成 TOML 的交一行原样的「读不出」。弄坏开关 cargo-configuration-ancestors-unread 下只读闭包根自己那一层与 CARGO_HOME 的。"""
    cargo_home = environment.get("CARGO_HOME") or os.path.join(environment.get("HOME") or os.path.expanduser("~"), ".cargo")
    cargo_home_files = [os.path.join(cargo_home, name) for name in CARGO_CONFIGURATION_FILE_NAMES]
    cargo_home_real = {os.path.realpath(path) for path in cargo_home_files}
    found = []
    directory = os.path.realpath(closure_root)
    while True:
        for name in CARGO_CONFIGURATION_FILE_NAMES:
            path = os.path.join(directory, ".cargo", name)
            if os.path.isfile(path) and os.path.realpath(path) not in cargo_home_real:
                found.append((path, directory))
        parent = os.path.dirname(directory)
        if parent == directory or break_is_set(CONFIGURATION_ANCESTORS_UNREAD_BREAK):
            break
        directory = parent
    for path in cargo_home_files:
        if os.path.isfile(path):
            found.append((path, os.path.dirname(cargo_home)))
    return found


def build_environment_lines(root, environment=None):
    """构建环境的生效值，形状 [(名字, 内容字节)]（子命令 build-environment-lines 打它，scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py 的 build_environment_lines 起程序取）：每一行生效值各一项。算不出抛 InputManifestError。"""
    try:
        return [(f"<构建环境：{line}>", line.encode("utf-8", "surrogateescape")) for line in build_environment_setting_lines(root, environment)]
    except UnitKeyError as error:
        raise InputManifestError(str(error)) from error


def command_build_environment_lines(arguments):
    """build-environment-lines <根>：build_environment_lines 的每一项打一行「名字<制表符>内容的 sha256」（字节写，不看终端编码）；算不出退 2。"""
    if len(arguments) != 1:
        say("  ✗ 用法：admission.py build-environment-lines <根>")
        say("     → 怎么办：只给一个参数，被判的树的根")
        return EXIT_REGISTRATION_ERROR
    try:
        lines = build_environment_lines(os.path.abspath(arguments[0]))
    except InputManifestError as error:
        say(f"  ✗ 构建环境算不出：{error}")
        say("     → 怎么办：照上面那句改 cargo 配置、rust-toolchain 或点名的环境变量，再来一遍")
        return EXIT_REGISTRATION_ERROR
    for name, content in lines:
        sys.stdout.buffer.write(f"{name}\t{hashlib.sha256(content).hexdigest()}\n".encode("utf-8", "surrogateescape"))
    sys.stdout.buffer.flush()
    return 0


def build_environment_setting_lines(closure_root, environment=None):
    """构建环境的生效值（rust_toolchain_lines 的那一截，不进单元键）：[行]。不写绝对路径：同一批内容在主工作区与临时 worktree 里算出同一份。"""
    environment = os.environ if environment is None else environment
    lines, layers, unreadable = [], [], []
    argument_directories = runner_argument_directories(closure_root)
    for path, base_directory in configuration_layers(closure_root, environment):
        try:
            with open(path, "rb") as handle:
                content = handle.read()
            settings = tomllib.loads(content.decode("utf-8"))
        except (OSError, UnicodeDecodeError, ValueError):
            unreadable.append(hashlib.sha256(read_configuration_file(path) if os.path.isfile(path) else b"").hexdigest())
            continue
        layers.append(settings)
        for label, program_bytes in configured_program_lines("cargo 配置", content, base_directory, closure_root, environment, argument_directories):
            lines.append(f"build-environment program {label}: {hashlib.sha256(program_bytes).hexdigest()}")
    flattened = []
    flattened_setting_lines("", configuration_affecting_output(merged_configuration(layers)), flattened)
    lines += [f"build-environment config {line}" for line in flattened]
    lines += [f"build-environment config 读不出的一层 {digest}" for digest in unreadable]
    directory = os.path.realpath(closure_root)
    while True:
        for name in TOOLCHAIN_FILE_NAMES:
            path = os.path.join(directory, name)
            if os.path.isfile(path):
                try:
                    with open(path, "rb") as handle:
                        content = handle.read()
                    parsed = tomllib.loads(content.decode("utf-8")) if name.endswith(".toml") else {"channel": content.decode("utf-8").strip()}
                except (OSError, UnicodeDecodeError, ValueError):
                    parsed = {"读不出": hashlib.sha256(read_configuration_file(path)).hexdigest()}
                toolchain_settings = []
                flattened_setting_lines(name, parsed, toolchain_settings)
                lines += [f"build-environment toolchain-file {line}" for line in toolchain_settings]
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    for variable in sorted(environment):
        if build_environment_variable_left_out(variable):
            continue
        if variable in BUILD_ENVIRONMENT_EXACT_VARIABLES or any(form.match(variable) for form in BUILD_ENVIRONMENT_VARIABLE_FORMS):
            lines.append(f"build-environment env {variable}={environment[variable]}")
    for variable in sorted(environment):
        if variable in BUILD_ENVIRONMENT_PROGRAM_VARIABLES or any(form.match(variable) for form in BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS):
            contents = program_contents(environment[variable].split(), closure_root, environment, argument_directories)
            lines.append(f"build-environment program env {variable}: {hashlib.sha256(contents).hexdigest()}")
    for variable in BUILD_ENVIRONMENT_COMPILER_VARIABLES:
        if variable in environment:
            try:
                version = compiler_version(environment[variable], closure_root, environment, f"{variable}={environment[variable]}")
            except InputManifestError as error:
                raise UnitKeyError(str(error)) from error
            lines.append(f"build-environment compiler {variable}: {version.decode('utf-8', 'replace').strip()}")
    return lines


# ── 复跑表每一行的行键（replay_row_request；文件头「复跑表每一行的行键」）──────────

def research_binary_sources(root):
    """scripts/experiment-apparatus/Cargo.toml 的 [[bin]]：{名字: scripts/experiment-apparatus 起的源文件}；清单不在或读不出给空表。"""
    sections = manifest_sections(os.path.join(root, REPLAY_RESEARCH_DIRECTORY, "Cargo.toml")) or {}
    return {entry["name"]: os.path.normpath(entry["path"])
            for entry in sections.get("bin") or [] if isinstance(entry, dict) and isinstance(entry.get("name"), str) and isinstance(entry.get("path"), str)}


def research_package_name(root):
    """scripts/experiment-apparatus/Cargo.toml 的 [package] name（研究工作区的包）；清单不在或读不出交 None。"""
    package = (manifest_sections(os.path.join(root, REPLAY_RESEARCH_DIRECTORY, "Cargo.toml")) or {}).get("package")
    return package.get("name") if isinstance(package, dict) and isinstance(package.get("name"), str) else None


def workspace_binary_entry(root, package, binary):
    """仓根工作区里包 package 的 bin binary 的 main 入口（仓根起）：[[bin]] 写了 path 的取它，没写的取 src/bin/<名>.rs 或 src/bin/<名>/main.rs；找不到交 None。"""
    packages = workspace_package_directories(root)
    if package not in packages:
        return None
    directory = packages[package]
    sections = manifest_sections(os.path.join(root, directory, "Cargo.toml")) or {}
    for entry in sections.get("bin") or []:
        if isinstance(entry, dict) and entry.get("name") == binary and isinstance(entry.get("path"), str):
            return os.path.normpath(os.path.join(directory, entry["path"])) + "::main"
    for candidate in (os.path.join(directory, "src", "bin", binary + ".rs"), os.path.join(directory, "src", "bin", binary, "main.rs")):
        if os.path.isfile(os.path.join(root, candidate)):
            return os.path.normpath(candidate) + "::main"
    return None


def test_plan_memoized_readers():
    """这一趟里 Cargo.toml、cargo 缓存里的清单与数据文件单元按路径只读一次（文件头 test-plan「内存」）：换掉模块里这几个名字，答案与现读的相同，同一趟里输入不变。
    弄坏开关 test-plan-file-readers-unmemoized：不换。"""
    global manifest_sections, registry_dependency_filter
    if break_is_set("test-plan-file-readers-unmemoized"):
        return
    library = function_units()

    def remembered(read, key_of):
        answers = {}

        def answer(*arguments):
            key = key_of(*arguments)
            if key in answers:
                TEST_PLAN_SHARED["reader_memo_hits"] = TEST_PLAN_SHARED.get("reader_memo_hits", 0) + 1
            else:
                answers[key] = read(*arguments)
            return answers[key]
        return answer

    manifest_sections = remembered(manifest_sections, lambda path: path)
    registry_dependency_filter = remembered(registry_dependency_filter, lambda _environment, package, host: (package.get("name"), package.get("version"), host))
    # 算键库拆成几份，真调数据单元的那几份（Rust 闭包、脚本闭包、dump）各自 from lib_function_units_base import 了自己模块里的名字：
    # 库本身（按路径载入、不在 sys.modules 里）与进程里凡是绑着这两个函数对象的模块名字都换成同一份记着答案的（只换绑的正是原函数的，别处同名的不动）
    for name in ("data_file_unit", "data_directory_unit"):
        original = getattr(library, name)
        replacement = remembered(original, lambda root, path: (root, path))
        for module in [library] + [module for module in list(sys.modules.values()) if module is not library]:
            namespace = getattr(module, "__dict__", None)
            if namespace is not None and namespace.get(name) is original:
                setattr(module, name, replacement)


# ── library-files（文件头「library-files」）──────────

def file_library_store(library_name):
    """库名 <git common-dir>/singlefs-unit-verdicts/<种类> → (这个种类在服务里的 LibraryStore, "")；库名写错或这棵树里没有客户端交 (None, 为什么)。不连服务。"""
    name = os.path.abspath(library_name)
    kind = os.path.basename(name)
    if os.path.basename(os.path.dirname(name)) != VERDICT_LIBRARY_DIRECTORY_NAME or not kind or kind_directory_name(kind) != kind:
        return None, f"库名要是 <git common-dir>/{VERDICT_LIBRARY_DIRECTORY_NAME}/<种类>（种类只用字母、数字与 ._:-），给的是 {library_name}"
    client = VerdictLibrary.client()
    if client is None:
        return None, "这棵树里没有 scripts/prune-server/prune_server_client.py"
    return client.library_store(client.FAMILY_UNIT_VERDICTS, kind, name), ""


def hidden_relative(relative):
    """相对路径里有空段、..、以 . 起头的段（原子写的临时文件）：不搬。"""
    return any(part.startswith(".") or not part for part in relative.split("/"))


def transit_files(directory):
    """中转目录里要并回服务的文件：[(相对路径, 绝对路径)]，按相对路径排；以 . 起头的路径段不算。"""
    found = []
    for current, subdirectories, names in os.walk(directory):
        subdirectories[:] = sorted(name for name in subdirectories if not name.startswith("."))
        for name in names:
            path = os.path.join(current, name)
            relative = os.path.relpath(path, directory).replace(os.sep, "/")
            if not hidden_relative(relative) and os.path.isfile(path):
                found.append((relative, path))
    return sorted(found)


def library_files_fetch(store, directory, prefixes):
    """服务里这个种类（给了前缀只取那几个前缀底下的）逐条写进中转目录：交写了几条。服务连不上客户端退 75
    （弄坏开关 library-files-unreachable-as-absent 下照「库里没有」交 0）。"""
    client = VerdictLibrary.client()
    entries = []
    try:
        for prefix in prefixes or [""]:
            entries += store.scan(prefix)
    except client.PruneServerFailure:
        if not break_is_set("library-files-unreachable-as-absent"):
            raise
        entries = []
    written = 0
    for relative, data in entries:
        if hidden_relative(relative):
            continue
        path = os.path.join(directory, *relative.split("/"))
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as handle:
            handle.write(data)
        written += 1
    return written


def library_files_store(store, directory, relatives):
    """中转目录里的文件（给了相对路径只写那几条，照给的次序）写进服务，逐字节相同的跳过：交 (写了, 跳过)。
    照给的次序分批写：调用方把「这一条在就算整条记录在」的那一份（replay-row 的 verdict）放在最后，它落库时前面的都已落库。"""
    if relatives:
        files = [(relative, os.path.join(directory, *relative.split("/"))) for relative in relatives]
    else:
        files = transit_files(directory)
    pairs = []
    for relative, path in files:
        with open(path, "rb") as handle:
            pairs.append((relative, handle.read()))
    present = store.read_many([relative for relative, _data in pairs]) if pairs else []
    writes = [(relative, data) for (relative, data), stored in zip(pairs, present) if stored != data]
    if writes:
        store.write_many(writes)
    return len(writes), len(pairs) - len(writes)


def command_library_files(arguments):
    if len(arguments) < 3 or arguments[0] not in ("fetch", "store", "has"):
        print("  ✗ 用法：admission.py library-files fetch <库名> <中转目录> [<相对前缀>…] | store <库名> <中转目录> [<相对路径>…] | has <库名> <相对路径>…")
        print("     → 怎么办：看文件头「library-files」")
        return EXIT_REGISTRATION_ERROR
    operation, library_name, rest = arguments[0], arguments[1], arguments[2:]
    store, problem = file_library_store(library_name)
    if store is None:
        print(f"  ✗ library-files {operation}：{problem}")
        print("     → 怎么办：看文件头「library-files」")
        return EXIT_REGISTRATION_ERROR
    relatives = rest if operation == "has" else rest[1:]
    bad = [relative for relative in relatives if relative.startswith("/") or (relative and hidden_relative(relative.rstrip("/")))]
    if bad:
        print(f"  ✗ library-files {operation}：相对路径写错（空段、以 . 起头的段或绝对路径）：{'、'.join(bad[:5])}")
        print("     → 怎么办：相对路径照服务里的写法写，例 keys/<键>/verdict")
        return EXIT_REGISTRATION_ERROR
    if operation == "has":
        for relative, data in zip(relatives, store.read_many(relatives)):
            if data is not None:
                print(relative)
        return 0
    directory = rest[0]
    if operation == "store":
        missing = [relative for relative in relatives if not os.path.isfile(os.path.join(directory, *relative.split("/")))]
        if not os.path.isdir(directory) or missing:
            print(f"  ✗ library-files store：中转目录 {directory} {'不在' if not os.path.isdir(directory) else '里没有 ' + '、'.join(missing[:5])}")
            print("     → 怎么办：给这一趟写好记录的那个目录，只点名里面写好了的文件")
            return EXIT_REGISTRATION_ERROR
        written, skipped = library_files_store(store, directory, relatives)
        print(f"LIBRARY_FILES_STORE written={written} skipped={skipped}")
        return 0
    os.makedirs(directory, exist_ok=True)
    print(f"LIBRARY_FILES_FETCH entries={library_files_fetch(store, directory, relatives)}")
    return 0
