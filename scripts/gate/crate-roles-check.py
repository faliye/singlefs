#!/usr/bin/env python3
# admission: always 判的是此刻树上 crates/ 下各包的样子与登记表，几秒跑完；加包、挪文件、改依赖都要现判
# run-condition: none 只读仓里的 Cargo.toml、源码目录与三张登记表，不起别的进程
"""crates/ 下每个包登记了角色、按角色放该放的东西（.claude/rules/directory-layout.md「crates/：各包放什么」）。

用法：
    crate-roles-check.py [根] [--lag-baseline <文件> | --lag-baseline-absent <原因> | --lag-baseline-unreadable <原因>]
    crate-roles-check.py --selftest     # CRATE_ROLES_CHECK_BREAK=<开关> 时必须红在点名的格

读的表（都在被判的根下；不在的当空表）：
  .claude/gate.d/crate-roles.tsv          登记表：crates/ 下一个包一行，四列制表符分隔：包名（Cargo.toml [package] 的 name）、角色（ROLES 四选一）、
                                          目录（相对根）、理由（去掉空白至少 REASON_MINIMUM 个字）；空行与 # 起头的行不算
  .claude/gate.d/crate-roles-lag.tsv      滞后表，只缩不涨：一行「<目标的源文件><TAB># 理由」，一个目标一行。目标是实验装置之外的包里没登放行表的二进制目标，
                                          或库为空、角色不是测试支撑的包的测试目标；源文件写相对根的路径。整包目录与别的目录不收
  .claude/gate.d/crate-roles-allowed.tsv  放行表：一行「<二进制目标的源文件><TAB># 理由」：实验装置之外的包里留在原处的二进制目标，
                                          验证包是门禁或崩溃入口自己起的工具，产品包是产品自己的可执行入口，理由写谁起它
判的包：crates/ 下任意深度的每一份 Cargo.toml（目录里有 CACHEDIR.TAG 的是 cargo 的编译目录，整棵不进）。数依赖者、找谁依赖实验装置包时另读根 Cargo.toml 的
  [workspace] members 与 exclude 点名的目录下的 Cargo.toml（这个仓里别的 cargo 工程依赖 crates/ 下的包也算依赖者）。依赖认 dependencies、dev-dependencies、
  build-dependencies 三张表，键名或 package = 给的真名都算；workspace = true 的照根 [workspace.dependencies] 那一项的 package = 认。
  数测试支撑包的依赖者只数无条件的依赖：target.<条件> 下的表与 optional = true 的项不算；找谁依赖实验装置包连这两种一起算。
二进制目标（照 cargo 的目标认法，不看名字）：[[bin]] 与 [[example]] 逐项（写了 path 取 path 指的文件，指到哪都算；没写取默认位置），
  autobins 不是 false 时另认 src/main.rs、src/bin/*.rs、src/bin/<目录>/main.rs，autoexamples 不是 false 时另认 examples/*.rs、examples/<目录>/main.rs；
  同一个源文件只算一个目标。测试目标：[[test]] 逐项，autotests 不是 false 时另认 tests/*.rs、tests/<目录>/main.rs。
库为空：库文件（[lib] path，没写取 src/lib.rs）不在；或去掉注释、属性（#[…]、#![…]）、use 与 extern crate（含 pub use 再导出）之后没有条目，
  mod 声明照 Rust 的模块规矩找到那个文件（#[path] 照认）、行内 mod { … } 逐层往里，里面同样没有条目的不算条目；mod 找不到文件的当有条目。

判法（红行以「✗ <格名>：」起头，格名如下）：
  registered            每个 crates/ 下 Cargo.toml 的包名在登记表里有一行；登记表里的包名在 crates/ 下都有
  duplicate             同一个包名登记了不止一行
  directory             登记的目录在，里面 Cargo.toml 的包名就是这一行的包名
  role                  行是四列、角色是 ROLES 之一、理由够长
  package-count         判到的包数为 0，或与 crates/ 下 Cargo.toml 的份数对不上（有一份读不出包名）
  product-binary        角色是产品的包有二进制目标，放行表、滞后表里都没有
  binary-unregistered   角色是验证或测试支撑的包有二进制目标，放行表、滞后表里都没有
  empty-library-tests   库为空而有测试目标、角色不是测试支撑，滞后表里没有的那几个测试目标
  support-dependents    角色是测试支撑的包，无条件的依赖者不到 SUPPORT_DEPENDENTS_MINIMUM 个（crates/ 下的包与根 Cargo.toml 点名的别的 cargo 工程都算）
  experiment-dependency 有包（或别的 cargo 工程）依赖角色是实验装置的包
  product-support-dependency  角色是产品的包经 [dependencies] 或 [build-dependencies] 依赖角色是测试支撑的包（target.<条件> 下的这两张表与 optional = true 的项都算：
                        产品构建可能并进来）；[dev-dependencies] 不算，测试一侧依赖测试支撑包照放
  lag-table             滞后表的行写不对、路径不在、是目录（整包行不收）、那一行已经用不着（不是没登放行的二进制目标，也不是空库包的测试目标）
  allowed-table         放行表的行写不对、条目不在、那一行用不着（不是实验装置之外的包里的二进制目标）
  lag-grown             滞后表比基准那一版多出路径；基准取不到（--lag-baseline-unreadable）
退出码：0 都对；1 有不对的（逐条列出）；2 用法错、弄坏开关设在自证之外；77 crates/ 不在。

基准：门禁 code-tooling 的 crate-roles 格经 .claude/gate.d/lib-lag-baseline.py 取基准那一版滞后表，有就交 --lag-baseline <文件>；
  基准里没有这张表或不在 git 仓里交 --lag-baseline-absent <原因>（只缩不涨这一条没比，成功句写明）；取不到交 --lag-baseline-unreadable <原因>（判红）。
  三个都不给按 --lag-baseline-absent「没给基准」。

管不到的：角色登得对不对；放行表里登的二进制是不是真是门禁工具或产品入口（理由靠人核）；[[bench]] 与 benches/ 下的目标；edition 2015 里显式写了目标就关掉
  自动发现的那一条（这里照开着认，多认不少认）；build.rs 引进来的源码；包里放的测试测的是不是这个包。

弄坏开关（只给自证用：--selftest 读它，平常跑设了退 2、点名只在自证里认）。一条一行「CRATE_ROLES_CHECK_BREAK=<项>  弄坏哪一处」，
下一行起写打开后自证红的格（失败行「✗ 自证 <格名>：」那一段）：
  CRATE_ROLES_CHECK_BREAK=registration-unchecked  包没登记、登记的包不在都不报
      「✗ 自证 registered：」「✗ 自证 registered-stale：」「✗ 自证 nested-package：」
  CRATE_ROLES_CHECK_BREAK=duplicate-accepted  同一个包名登记两行不报
      「✗ 自证 duplicate：」
  CRATE_ROLES_CHECK_BREAK=directory-unchecked  登记的目录不核
      「✗ 自证 directory：」
  CRATE_ROLES_CHECK_BREAK=role-unchecked  角色认不得、行写不对不报
      「✗ 自证 role：」
  CRATE_ROLES_CHECK_BREAK=package-count-unchecked  判到的包数为 0、与 Cargo.toml 份数对不上都不报
      「✗ 自证 package-count-unreadable：」「✗ 自证 package-count-zero：」
  CRATE_ROLES_CHECK_BREAK=nested-packages-unseen  只找 crates/*/Cargo.toml，嵌在子目录里的包看不见
      「✗ 自证 nested-package：」
  CRATE_ROLES_CHECK_BREAK=build-directories-entered  目录里有 CACHEDIR.TAG 的编译目录也往里找 Cargo.toml
      「✗ 自证 build-directory-skipped：」
  CRATE_ROLES_CHECK_BREAK=product-binary-accepted  产品包没登记的二进制目标不报
      「✗ 自证 product-binary：」「✗ 自证 product-binary-manifest：」「✗ 自证 product-examples：」「✗ 自证 product-main-rs：」
  CRATE_ROLES_CHECK_BREAK=unregistered-binary-accepted  验证包、测试支撑包没登记的二进制目标不报
      「✗ 自证 binary-unregistered：」「✗ 自证 verification-bin-outside-src-bin：」「✗ 自证 verification-hyphen-name：」「✗ 自证 support-binary：」
      「✗ 自证 tests-package-new-binary：」
  CRATE_ROLES_CHECK_BREAK=binary-targets-src-bin-only  二进制目标只认 src/bin/ 下的与 [[bin]]，不认 src/main.rs 与 examples/
      「✗ 自证 green：」「✗ 自证 product-examples：」「✗ 自证 product-main-rs：」「✗ 自证 product-entry-allowed：」「✗ 自证 build-directory-skipped：」
      「✗ 自证 library-with-module-items：」「✗ 自证 product-support-dev-dependency-allowed：」
      「✗ 自证 support-dependents-other-cargo-projects：」
  CRATE_ROLES_CHECK_BREAK=declared-binaries-ignored  [[bin]] 与 [[example]] 一项都不读，只认自动发现的
      「✗ 自证 product-binary-manifest：」「✗ 自证 verification-bin-outside-src-bin：」
  CRATE_ROLES_CHECK_BREAK=library-comments-counted  判库空不空时注释与属性也算条目
      「✗ 自证 empty-library-tests：」
  CRATE_ROLES_CHECK_BREAK=library-reexports-counted  判库空不空时 use、extern crate 与 mod 也算条目
      「✗ 自证 tests-package-one-line-library：」「✗ 自证 tests-package-reexport-module：」
  CRATE_ROLES_CHECK_BREAK=empty-library-tests-accepted  空库带测试目标不报
      「✗ 自证 empty-library-tests：」「✗ 自证 tests-package-one-line-library：」「✗ 自证 tests-package-reexport-module：」
      「✗ 自证 tests-package-new-test-target：」「✗ 自证 empty-library-file-absent：」
  CRATE_ROLES_CHECK_BREAK=support-dependents-unchecked  测试支撑包的依赖者不数
      「✗ 自证 support-dependents：」「✗ 自证 support-dependents-cfg：」「✗ 自证 support-dependents-optional：」
  CRATE_ROLES_CHECK_BREAK=conditional-dependents-counted  数测试支撑包的依赖者时 target.<条件> 下的与 optional = true 的也算
      「✗ 自证 support-dependents-cfg：」「✗ 自证 support-dependents-optional：」
  CRATE_ROLES_CHECK_BREAK=experiment-dependency-accepted  依赖实验装置包不报
      「✗ 自证 experiment-dependency：」「✗ 自证 experiment-dependency-cfg：」
  CRATE_ROLES_CHECK_BREAK=product-support-dependency-accepted  产品包经 [dependencies]、[build-dependencies] 依赖测试支撑包不报
      「✗ 自证 product-support-dependency：」「✗ 自证 product-support-build-dependency：」「✗ 自证 product-support-dependency-cfg：」
  CRATE_ROLES_CHECK_BREAK=stale-lag-accepted  滞后表用不着的行、指不到的路径、整包与目录行不报
      「✗ 自证 lag-table-stale：」「✗ 自证 lag-table-package-row：」
  CRATE_ROLES_CHECK_BREAK=package-lag-row-accepted  滞后表里包目录那一行照收，那个包整包不判
      「✗ 自证 lag-table-package-row：」
  CRATE_ROLES_CHECK_BREAK=allowed-unchecked  放行表的行不核
      「✗ 自证 allowed-table：」
  CRATE_ROLES_CHECK_BREAK=lag-growth-accepted  滞后表比基准多出的行、基准取不到都不报
      「✗ 自证 lag-grown：」「✗ 自证 lag-baseline-unreadable：」
  CRATE_ROLES_CHECK_BREAK=missing-library-file-not-empty  库文件不在当库不空
      「✗ 自证 empty-library-file-absent：」
  CRATE_ROLES_CHECK_BREAK=other-cargo-projects-uncounted  数测试支撑包的依赖者时只数 crates/ 下的包，根 Cargo.toml 点名的别的 cargo 工程不算
      「✗ 自证 green：」「✗ 自证 build-directory-skipped：」「✗ 自证 product-entry-allowed：」「✗ 自证 library-with-module-items：」
      「✗ 自证 support-dependents-optional：」「✗ 自证 support-dependents-other-cargo-projects：」
"""
import os
import re
import subprocess
import sys
import tempfile
import tomllib
from concurrent.futures import ThreadPoolExecutor

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "hosts"))
import heavy_threads  # noqa: E402

BREAK_VARIABLE = "CRATE_ROLES_CHECK_BREAK"
BREAKS = ("registration-unchecked", "duplicate-accepted", "directory-unchecked", "role-unchecked", "package-count-unchecked",
          "nested-packages-unseen", "build-directories-entered", "product-binary-accepted", "unregistered-binary-accepted",
          "binary-targets-src-bin-only", "declared-binaries-ignored", "library-comments-counted", "library-reexports-counted",
          "empty-library-tests-accepted", "support-dependents-unchecked", "conditional-dependents-counted", "experiment-dependency-accepted",
          "product-support-dependency-accepted", "stale-lag-accepted", "package-lag-row-accepted", "allowed-unchecked", "lag-growth-accepted",
          "missing-library-file-not-empty", "other-cargo-projects-uncounted")
ROLE_TABLE = ".claude/gate.d/crate-roles.tsv"
LAG_TABLE = ".claude/gate.d/crate-roles-lag.tsv"
ALLOWED_TABLE = ".claude/gate.d/crate-roles-allowed.tsv"
PRODUCT_ROLE = "产品"
VERIFICATION_ROLE = "验证"
SUPPORT_ROLE = "测试支撑"
EXPERIMENT_ROLE = "实验装置"
ROLES = (PRODUCT_ROLE, VERIFICATION_ROLE, SUPPORT_ROLE, EXPERIMENT_ROLE)
REASON_MINIMUM = 4
SUPPORT_DEPENDENTS_MINIMUM = 2
BUILD_DIRECTORY_MARK = "CACHEDIR.TAG"   # cargo 在它建的编译目录顶上放这一份
DEPENDENCY_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")
PRODUCT_BUILD_TABLES = ("dependencies", "build-dependencies")   # 产品构建会并进来的两张表：产品包在这里依赖测试支撑包判红（product-support-dependency）
TABLE_ROW = re.compile(r"^(?P<path>[^\t#]+?)\t+#\s*(?P<reason>.*?)\s*$")
VISIBILITY = r"(?:pub(?:\s*\([^)]*\))?\s+)?"
USE_ITEM = re.compile(r"\s*" + VISIBILITY + r"use\b[^;]*;", re.S)
EXTERN_CRATE_ITEM = re.compile(r"\s*" + VISIBILITY + r"extern\s+crate\b[^;]*;", re.S)
MODULE_DECLARATION = re.compile(r"\s*" + VISIBILITY + r"mod\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*;", re.S)
MODULE_BLOCK = re.compile(r"\s*" + VISIBILITY + r"mod\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*\{", re.S)
ATTRIBUTE_START = re.compile(r"\s*#(!?)\[")
PATH_ATTRIBUTE = re.compile(r'^\s*path\s*=\s*"([^"]*)"\s*$', re.S)
HOWTO = ("照 .claude/rules/directory-layout.md「crates/：各包放什么」改：新包在 .claude/gate.d/crate-roles.tsv 登一行角色；实验装置与测试二进制挪进实验装置包或被测包的 tests/，"
         "一个包的测试挪进它自己的 tests/，不为测试另开包；测试支撑包只剩一个无条件的依赖者的，并回那个包的 tests/common_<名>/mod.rs；没有包依赖实验装置包。"
         "门禁或崩溃入口起的工具、产品自己的可执行入口逐个登进 .claude/gate.d/crate-roles-allowed.tsv（理由写谁起它）；"
         "一时改不完的存量逐个目标登进 .claude/gate.d/crate-roles-lag.tsv（只缩不涨，整包行不收），改完删那一行")
broken = set()


def visible_length(text):
    return len("".join(text.split()))


def read_manifest(path):
    """Cargo.toml → dict；读不出（不在、不是合法 TOML）交 (None, 原因)。"""
    try:
        with open(path, "rb") as handle:
            return tomllib.load(handle), ""
    except (OSError, tomllib.TOMLDecodeError) as error:
        return None, f"{type(error).__name__}: {error}"


def workspace_renames(root_manifest):
    """根 Cargo.toml [workspace.dependencies] 里写了 package = 的键 → 真名。"""
    table = ((root_manifest or {}).get("workspace") or {}).get("dependencies") or {}
    return {key: value["package"] for key, value in table.items() if isinstance(value, dict) and isinstance(value.get("package"), str)}


def dependency_names(manifest, renames, unconditional_only):
    """一份 Cargo.toml 依赖的真名集合：键名，或 package = 给的真名；workspace = true 的照根 [workspace.dependencies] 的改名。
    unconditional_only 时只数三张顶层表里不带 optional = true 的项（target.<条件> 下的不算；弄坏开关 conditional-dependents-counted 下照算）。"""
    conditional_counted = not unconditional_only or "conditional-dependents-counted" in broken
    tables = [manifest.get(name) for name in DEPENDENCY_TABLES]
    if conditional_counted:
        for condition in (manifest.get("target") or {}).values():
            if isinstance(condition, dict):
                tables += [condition.get(name) for name in DEPENDENCY_TABLES]
    names = set()
    for table in tables:
        for key, value in (table or {}).items():
            if isinstance(value, dict) and value.get("optional") is True and not conditional_counted:
                continue
            if isinstance(value, dict) and isinstance(value.get("package"), str):
                names.add(value["package"])
            elif isinstance(value, dict) and value.get("workspace") is True:
                names.add(renames.get(key, key))
            else:
                names.add(key)
    return names


def product_build_dependencies(manifest, renames):
    """一份 Cargo.toml 在 PRODUCT_BUILD_TABLES 里（顶层与 target.<条件> 下，optional 的照算）依赖的真名：[(表名, 真名)]，表名写成 [dependencies]、[target.<条件>.build-dependencies] 这样。"""
    tables = [(f"[{name}]", manifest.get(name)) for name in PRODUCT_BUILD_TABLES]
    for condition, specification in (manifest.get("target") or {}).items():
        if isinstance(specification, dict):
            tables += [(f"[target.{condition}.{name}]", specification.get(name)) for name in PRODUCT_BUILD_TABLES]
    found = []
    for label, table in tables:
        for key, value in (table or {}).items():
            if isinstance(value, dict) and isinstance(value.get("package"), str):
                found.append((label, value["package"]))
            elif isinstance(value, dict) and value.get("workspace") is True:
                found.append((label, renames.get(key, key)))
            else:
                found.append((label, key))
    return found


def without_comments(text):
    """Rust 源码去掉 // 行注释与 /* */ 块注释（块注释可嵌套）；字符串字面量里的 // 与 /* 不当注释。"""
    kept, index, length = [], 0, len(text)
    while index < length:
        if text.startswith("//", index):
            end = text.find("\n", index)
            index = length if end < 0 else end
        elif text.startswith("/*", index):
            depth, index = 1, index + 2
            while index < length and depth:
                if text.startswith("/*", index):
                    depth, index = depth + 1, index + 2
                elif text.startswith("*/", index):
                    depth, index = depth - 1, index + 2
                else:
                    index += 1
        elif text[index] == '"':
            end = index + 1
            while end < length and text[end] != '"':
                end += 2 if text[end] == "\\" else 1
            kept.append(text[index:end + 1])
            index = end + 1
        else:
            kept.append(text[index])
            index += 1
    return "".join(kept)


def closing_bracket(text, opening, open_character, close_character):
    """text[opening] 是 open_character：交与它配平的那个 close_character 的下标；配不平交 len(text)。"""
    depth = 0
    for position in range(opening, len(text)):
        if text[position] == open_character:
            depth += 1
        elif text[position] == close_character:
            depth -= 1
            if depth == 0:
                return position
    return len(text)


def module_file(directory, name, path_attribute, attribute_directory):
    """mod <name>; 落在哪个文件：写了 #[path] 的照 attribute_directory 起算，没写的找 <directory>/<name>.rs 与 <directory>/<name>/mod.rs；
    交 (文件, 它里面的 mod 从哪个目录起找)，找不到交 (None, None)。"""
    if path_attribute is not None:
        found = os.path.normpath(os.path.join(attribute_directory, path_attribute))
        return (found, os.path.dirname(found)) if os.path.isfile(found) else (None, None)
    flat = os.path.join(directory, name + ".rs")
    if os.path.isfile(flat):
        return flat, os.path.join(directory, name)
    nested = os.path.join(directory, name, "mod.rs")
    if os.path.isfile(nested):
        return nested, os.path.join(directory, name)
    return None, None


def module_has_items(text, child_directory, attribute_directory, seen):
    """一段去掉了注释的模块正文里有没有条目：属性、use、extern crate 跳过，mod 往里找（找不到文件的当有条目），别的都是条目。
    child_directory 是这一段里 mod <名>; 找文件的目录，attribute_directory 是这一段里 #[path] 起算的目录。
    弄坏开关 library-comments-counted 下属性算条目，library-reexports-counted 下 use、extern crate 与 mod 都算条目。"""
    position, pending_path, length = 0, None, len(text)
    while True:
        while position < length and text[position].isspace():
            position += 1
        if position >= length:
            return False
        attribute = ATTRIBUTE_START.match(text, position)
        if attribute:
            if "library-comments-counted" in broken:
                return True
            opening = attribute.end() - 1
            closing = closing_bracket(text, opening, "[", "]")
            found_path = PATH_ATTRIBUTE.match(text[opening + 1:closing])
            if not attribute.group(1) and found_path:
                pending_path = found_path.group(1)
            position = closing + 1
            continue
        plain = USE_ITEM.match(text, position) or EXTERN_CRATE_ITEM.match(text, position)
        if plain:
            if "library-reexports-counted" in broken:
                return True
            position, pending_path = plain.end(), None
            continue
        declared = MODULE_DECLARATION.match(text, position)
        if declared:
            if "library-reexports-counted" in broken:
                return True
            path, inner_directory = module_file(child_directory, declared.group(1), pending_path, attribute_directory)
            if path is None:
                return True
            if os.path.realpath(path) not in seen:
                seen.add(os.path.realpath(path))
                inner = without_comments(open(path, encoding="utf-8", errors="replace").read())
                if module_has_items(inner, inner_directory, os.path.dirname(path), seen):
                    return True
            position, pending_path = declared.end(), None
            continue
        block = MODULE_BLOCK.match(text, position)
        if block:
            if "library-reexports-counted" in broken:
                return True
            opening = block.end() - 1
            closing = closing_bracket(text, opening, "{", "}")
            inner_directory = os.path.join(child_directory, block.group(1))
            if module_has_items(text[opening + 1:closing], inner_directory, inner_directory, seen):
                return True
            position, pending_path = closing + 1, None
            continue
        return True


def library_is_empty(directory, manifest):
    """库文件（[lib] path，没写取 src/lib.rs）不在，或照 module_has_items 没有条目（文件头「库为空」）。
    弄坏开关 missing-library-file-not-empty 下库文件不在当库不空。"""
    library = ((manifest.get("lib") or {}).get("path")) or "src/lib.rs"
    path = os.path.join(directory, library)
    if not os.path.isfile(path):
        return "missing-library-file-not-empty" not in broken
    text = without_comments(open(path, encoding="utf-8", errors="replace").read())
    return not module_has_items(text, os.path.dirname(path), os.path.dirname(path), {os.path.realpath(path)})


def auto_targets(directory, folder):
    """<folder>/*.rs 与 <folder>/<目录>/main.rs：[(目标名, 相对包目录的源文件)]。"""
    base = os.path.join(directory, folder)
    found = []
    if os.path.isdir(base):
        for entry in sorted(os.listdir(base)):
            if entry.endswith(".rs") and os.path.isfile(os.path.join(base, entry)):
                found.append((entry[:-3], f"{folder}/{entry}"))
            elif os.path.isfile(os.path.join(base, entry, "main.rs")):
                found.append((entry, f"{folder}/{entry}/main.rs"))
    return found


def declared_source(directory, item, folder):
    """[[bin]] / [[example]] / [[test]] 的一项指的源文件（相对包目录）：写了 path 取 path，没写取 <folder>/<名>.rs，再看 <folder>/<名>/main.rs。"""
    if isinstance(item.get("path"), str):
        return item["path"]
    name = item.get("name") or ""
    if not os.path.isfile(os.path.join(directory, folder, name + ".rs")) and os.path.isfile(os.path.join(directory, folder, name, "main.rs")):
        return f"{folder}/{name}/main.rs"
    return f"{folder}/{name}.rs"


def binary_targets(root, package_directory, manifest, package_name):
    """一个包的二进制目标：[(种类, 目标名, 源文件相对根的路径)]，同一个源文件只算一次（文件头「二进制目标」）。
    弄坏开关 declared-binaries-ignored 下 [[bin]]、[[example]] 不读；binary-targets-src-bin-only 下不认 src/main.rs 与 examples/。"""
    directory = os.path.join(root, package_directory)
    package = manifest.get("package") or {}
    found = {}

    def add(kind, name, source):
        found.setdefault(os.path.normpath(os.path.join(package_directory, source)), (kind, name))
    if "declared-binaries-ignored" not in broken:
        for item in manifest.get("bin") or []:
            if isinstance(item, dict):
                add("bin", item.get("name") or item.get("path"), declared_source(directory, item, "src/bin"))
        if "binary-targets-src-bin-only" not in broken:
            for item in manifest.get("example") or []:
                if isinstance(item, dict):
                    add("example", item.get("name") or item.get("path"), declared_source(directory, item, "examples"))
    if package.get("autobins") is not False:
        if "binary-targets-src-bin-only" not in broken and os.path.isfile(os.path.join(directory, "src", "main.rs")):
            add("bin", package_name, "src/main.rs")
        for name, source in auto_targets(directory, "src/bin"):
            add("bin", name, source)
    if package.get("autoexamples") is not False and "binary-targets-src-bin-only" not in broken:
        for name, source in auto_targets(directory, "examples"):
            add("example", name, source)
    return [(kind, name, path) for path, (kind, name) in sorted(found.items())]


def test_targets(root, package_directory, manifest):
    """一个包的测试目标的源文件（相对根的路径），排好序、不重复：[[test]] 逐项，autotests 不是 false 时另认 tests/*.rs 与 tests/<目录>/main.rs。"""
    directory = os.path.join(root, package_directory)
    sources = {declared_source(directory, item, "tests") for item in manifest.get("test") or [] if isinstance(item, dict)}
    if (manifest.get("package") or {}).get("autotests") is not False:
        sources |= {source for _name, source in auto_targets(directory, "tests")}
    return sorted({os.path.normpath(os.path.join(package_directory, source)) for source in sources})


def manifest_paths_under(root):
    """crates/ 下的 Cargo.toml（相对根）：任意深度，目录里有 BUILD_DIRECTORY_MARK 的整棵不进。
    弄坏开关 nested-packages-unseen 下只认 crates/*/Cargo.toml；build-directories-entered 下编译目录也进。"""
    crates = os.path.join(root, "crates")
    if "nested-packages-unseen" in broken:
        return sorted(os.path.join("crates", name, "Cargo.toml") for name in os.listdir(crates)
                      if os.path.isfile(os.path.join(crates, name, "Cargo.toml")))
    found = []
    for walked, directories, names in os.walk(crates):
        if BUILD_DIRECTORY_MARK in names and "build-directories-entered" not in broken:
            directories[:] = []
            continue
        directories.sort()
        if "Cargo.toml" in names and walked != crates:
            found.append(os.path.relpath(os.path.join(walked, "Cargo.toml"), root))
    return sorted(found)


def read_table(path, columns):
    """登记表、滞后表、放行表 → ([(行号, 字段…)], [(行号, 一句为什么写不对)])。columns 是 4（登记表）或 1（另两张：路径加 # 理由）。"""
    rows, problems = [], []
    if not os.path.isfile(path):
        return rows, problems
    for number, line in enumerate(open(path, encoding="utf-8").read().split("\n"), start=1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        if columns == 4:
            fields = line.split("\t")
            if len(fields) != 4 or any(not field.strip() for field in fields):
                problems.append((number, f"不是四列（包名、角色、目录、理由，制表符分隔）：{line.strip()[:160]}"))
                continue
            name, role, directory, reason = (field.strip() for field in fields)
            if visible_length(reason) < REASON_MINIMUM:
                problems.append((number, f"理由「{reason}」不到 {REASON_MINIMUM} 个字"))
                continue
            rows.append((number, name, role, directory.rstrip("/")))
        else:
            found = TABLE_ROW.match(line)
            if not found or visible_length(found.group("reason")) < REASON_MINIMUM:
                problems.append((number, f"要写成「<路径><TAB># 理由>」、理由至少 {REASON_MINIMUM} 个字：{line.strip()[:160]}"))
                continue
            rows.append((number, os.path.normpath(found.group("path").strip())))
    return rows, problems


def cargo_projects(root):
    """根 Cargo.toml [workspace] members 与 exclude 点名的、crates/ 之外的目录（有 Cargo.toml 的）；交 (目录列表, 根清单)。"""
    root_manifest, _ = read_manifest(os.path.join(root, "Cargo.toml"))
    workspace = (root_manifest or {}).get("workspace") or {}
    directories = []
    for entry in list(workspace.get("members") or []) + list(workspace.get("exclude") or []):
        if isinstance(entry, str) and not entry.startswith("crates/") and os.path.isfile(os.path.join(root, entry, "Cargo.toml")):
            directories.append(entry.rstrip("/"))
    return sorted(set(directories)), root_manifest


def judge(root, baseline):
    """→ (退出码, [(格, 一句)], 成功句)。baseline 是 ("table", 文件) / ("absent", 原因) / ("unreadable", 原因)。"""
    crates = os.path.join(root, "crates")
    if not os.path.isdir(crates):
        return 77, [], f"{root}/crates 不在，本次无对象可判"
    problems = []
    manifest_paths = manifest_paths_under(root)
    other_projects, root_manifest = cargo_projects(root)
    renames = workspace_renames(root_manifest)
    packages = {}   # 包名 → [{directory, manifest}]
    unreadable = []
    for manifest_path in manifest_paths:
        manifest, why = read_manifest(os.path.join(root, manifest_path))
        name = ((manifest or {}).get("package") or {}).get("name")
        if not isinstance(name, str):
            unreadable.append(f"{manifest_path}（{why or '没有 [package] name'}）")
            continue
        packages.setdefault(name, []).append({"directory": os.path.dirname(manifest_path), "manifest": manifest})
    judged = sum(len(entries) for entries in packages.values())
    if "package-count-unchecked" not in broken:
        if judged == 0:
            problems.append(("package-count", f"crates/ 下判到 0 个包（Cargo.toml {len(manifest_paths)} 份）：扫到 0 个而报绿，与判过了一模一样"))
        elif judged != len(manifest_paths):
            problems.append(("package-count", f"判到 {judged} 个包，crates/ 下 Cargo.toml 有 {len(manifest_paths)} 份；读不出包名的：{'、'.join(unreadable)}"))

    role_rows, role_problems = read_table(os.path.join(root, ROLE_TABLE), 4)
    if "role-unchecked" not in broken:
        problems += [("role", f"{ROLE_TABLE} 第 {number} 行{why}") for number, why in role_problems]
    rows_by_name = {}
    for number, name, role, directory in role_rows:
        rows_by_name.setdefault(name, []).append((number, role, directory))
    roles = {}
    for name, rows in sorted(rows_by_name.items()):
        if len(rows) > 1 and "duplicate-accepted" not in broken:
            problems.append(("duplicate", f"{ROLE_TABLE} 第 {'、'.join(str(row[0]) for row in rows)} 行都登记 {name}"))
        number, role, directory = rows[0]
        if role not in ROLES:
            if "role-unchecked" not in broken:
                problems.append(("role", f"{ROLE_TABLE} 第 {number} 行 {name} 的角色「{role}」不是 {'、'.join(ROLES)} 之一"))
        else:
            roles[name] = role
        if name not in packages:
            if "registration-unchecked" not in broken:
                problems.append(("registered", f"{ROLE_TABLE} 第 {number} 行登记的 {name} 在 crates/ 下的 Cargo.toml 里没有（包删了或改名了，删或改这一行）"))
            continue
        if "directory-unchecked" not in broken:
            actual = [entry["directory"] for entry in packages[name]]
            if directory not in actual:
                problems.append(("directory", f"{ROLE_TABLE} 第 {number} 行 {name} 登记的目录 {directory} 不对：它的 Cargo.toml 在 {'、'.join(actual)}"
                                 + ("" if os.path.isdir(os.path.join(root, directory)) else f"（{directory} 不在）")))
    if "registration-unchecked" not in broken:
        for name in sorted(set(packages) - set(rows_by_name)):
            problems.append(("registered", f"{packages[name][0]['directory']} 的包 {name} 在 {ROLE_TABLE} 里没有一行"))

    # 依赖关系：crates/ 下各包加根 Cargo.toml 点名的别的 cargo 工程；数测试支撑包的依赖者与找谁依赖实验装置包都数这两样
    crate_manifests = [(entry["directory"], name, entry["manifest"]) for name, entries in packages.items() for entry in entries]
    project_manifests = []
    for directory in other_projects:
        manifest, _ = read_manifest(os.path.join(root, directory, "Cargo.toml"))
        if manifest is not None:
            project_manifests.append((directory, ((manifest.get("package") or {}).get("name")) or directory, manifest))
    depending_manifests = crate_manifests + project_manifests
    # 弄坏开关 other-cargo-projects-uncounted：数依赖者时只数 crates/ 下的包，别的 cargo 工程不算
    dependent_manifests = crate_manifests if "other-cargo-projects-uncounted" in broken else depending_manifests
    dependents = {}   # 被依赖的包名 → [无条件依赖它的清单]
    for directory, name, manifest in dependent_manifests:
        for dependency in dependency_names(manifest, renames, unconditional_only=True):
            if dependency != name:
                dependents.setdefault(dependency, []).append(f"{directory}/Cargo.toml")

    allowed_rows, allowed_problems = read_table(os.path.join(root, ALLOWED_TABLE), 1)
    lag_rows, lag_problems = read_table(os.path.join(root, LAG_TABLE), 1)
    allowed_paths = {path for _number, path in allowed_rows}
    lag_paths = {path for _number, path in lag_rows}
    package_directories = {entry["directory"] for entries in packages.values() for entry in entries}
    skipped_packages = lag_paths & package_directories if "package-lag-row-accepted" in broken else set()

    binary_sources = set()        # 实验装置之外的包里的二进制目标
    registrable = set()           # 滞后表收的目标：没登放行表的二进制目标，与空库、不是测试支撑的包的测试目标
    binary_lagged = test_lagged = 0
    for name, entries in sorted(packages.items()):
        role = roles.get(name)
        for entry in entries:
            directory, manifest = entry["directory"], entry["manifest"]
            if role is None or directory in skipped_packages:
                continue
            if role != EXPERIMENT_ROLE:
                unregistered = []
                for kind, target, path in binary_targets(root, directory, manifest, name):
                    binary_sources.add(path)
                    if path in allowed_paths:
                        continue
                    registrable.add(path)
                    if path in lag_paths:
                        binary_lagged += 1
                    else:
                        unregistered.append(f"{path}（{kind} {target}）")
                cell, switch = ("product-binary", "product-binary-accepted") if role == PRODUCT_ROLE else ("binary-unregistered", "unregistered-binary-accepted")
                if unregistered and switch not in broken:
                    problems.append((cell, f"{directory}（{role}）有 {len(unregistered)} 个二进制目标在放行表、滞后表里都没有：{'、'.join(unregistered)}"))
            targets = test_targets(root, directory, manifest)
            if role != SUPPORT_ROLE and targets and library_is_empty(os.path.join(root, directory), manifest):
                registrable.update(targets)
                unlagged = [path for path in targets if path not in lag_paths]
                test_lagged += len(targets) - len(unlagged)
                if unlagged and "empty-library-tests-accepted" not in broken:
                    problems.append(("empty-library-tests", f"{directory}（{role}）库是空的、却有 {len(unlagged)} 个测试目标不在滞后表里（{'、'.join(unlagged[:4])}"
                                     f"{'……' if len(unlagged) > 4 else ''}）：为测试另开的包，测试挪进被测的那个包的 tests/"))
            if role == SUPPORT_ROLE and "support-dependents-unchecked" not in broken:
                users = sorted(set(dependents.get(name, [])))
                if len(users) < SUPPORT_DEPENDENTS_MINIMUM:
                    problems.append(("support-dependents", f"{directory}（测试支撑）只有 {len(users)} 个无条件的依赖者（{'、'.join(users) or '没有'}），"
                                     f"至少 {SUPPORT_DEPENDENTS_MINIMUM} 个才单开包"))

    experiment_packages = {name for name, role in roles.items() if role == EXPERIMENT_ROLE}
    if "experiment-dependency-accepted" not in broken:
        for directory, name, manifest in depending_manifests:
            for dependency in sorted(dependency_names(manifest, renames, unconditional_only=False) & experiment_packages - {name}):
                problems.append(("experiment-dependency", f"{directory}/Cargo.toml 依赖实验装置包 {dependency}：实验装置包不给别的包依赖"))
    support_packages = {name for name, role in roles.items() if role == SUPPORT_ROLE}
    if "product-support-dependency-accepted" not in broken:
        for name, entries in sorted(packages.items()):
            if roles.get(name) != PRODUCT_ROLE:
                continue
            for entry in entries:
                for table, dependency in product_build_dependencies(entry["manifest"], renames):
                    if dependency in support_packages and dependency != name:
                        problems.append(("product-support-dependency", f"{entry['directory']}/Cargo.toml 的 {table} 依赖测试支撑包 {dependency}："
                                         "测试支撑包只给测试与装置用，产品包只在 [dev-dependencies] 里依赖它"))

    if "stale-lag-accepted" not in broken:
        problems += [("lag-table", f"{LAG_TABLE} 第 {number} 行{why}") for number, why in lag_problems]
        for number, path in lag_rows:
            if path in skipped_packages:
                continue
            if not os.path.lexists(os.path.join(root, path)):
                problems.append(("lag-table", f"{LAG_TABLE} 第 {number} 行的 {path} 不在，删掉这一行"))
            elif os.path.isdir(os.path.join(root, path)):
                problems.append(("lag-table", f"{LAG_TABLE} 第 {number} 行的 {path} 是目录：滞后表一个目标一行（目标的源文件），整包与目录行不收，改成逐个目标的行"))
            elif path not in registrable:
                problems.append(("lag-table", f"{LAG_TABLE} 第 {number} 行的 {path} 不是没登放行表的二进制目标，也不是空库包的测试目标，用不着，删掉这一行"))
    if "allowed-unchecked" not in broken:
        problems += [("allowed-table", f"{ALLOWED_TABLE} 第 {number} 行{why}") for number, why in allowed_problems]
        for number, path in allowed_rows:
            if not os.path.lexists(os.path.join(root, path)):
                problems.append(("allowed-table", f"{ALLOWED_TABLE} 第 {number} 行的 {path} 不在，删掉这一行"))
            elif path not in binary_sources:
                problems.append(("allowed-table", f"{ALLOWED_TABLE} 第 {number} 行的 {path} 不是实验装置之外的包里的二进制目标，用不着，删掉这一行"))

    kind, detail = baseline
    if kind == "table":
        base_paths = {path for _number, path in read_table(detail, 1)[0]}
        grown = [(number, path) for number, path in lag_rows if path not in base_paths]
        if grown and "lag-growth-accepted" not in broken:
            problems.append(("lag-grown", f"{LAG_TABLE} 比基准那一版多出 {len(grown)} 行（只缩不涨）：" + "、".join(f"第 {number} 行 {path}" for number, path in grown)))
        growth_note = f"滞后表与基准那一版比没涨（基准 {len(base_paths)} 行）"
    elif kind == "unreadable":
        if "lag-growth-accepted" not in broken:
            problems.append(("lag-grown", f"取不到基准那一版的 {LAG_TABLE}（{detail}），只缩不涨没法比"))
        growth_note = "基准取不到"
    else:
        growth_note = f"只缩不涨这一条没比（{detail}）"

    if problems:
        return 1, problems, ""
    by_role = {role: sum(1 for name in packages if roles.get(name) == role) for role in ROLES}
    return 0, [], (f"判了 crates/ 下 {judged} 个包（{'、'.join(f'{role} {count}' for role, count in by_role.items())}）：都登记了角色、目录对得上；"
                   f"实验装置之外的二进制目标 {len(binary_sources)} 个都登记了（放行 {len(binary_sources) - binary_lagged}、滞后 {binary_lagged}），"
                   f"库为空而有测试目标的只有测试支撑包与滞后表里的测试目标（滞后 {test_lagged}），"
                   f"测试支撑包都有至少 {SUPPORT_DEPENDENTS_MINIMUM} 个无条件的依赖者（数了 {len(depending_manifests)} 份清单），没有包依赖实验装置包，"
                   "产品包的 [dependencies]、[build-dependencies] 里没有测试支撑包；"
                   f"滞后表 {len(lag_rows)} 行，{growth_note}")


def report(root, baseline):
    code, problems, summary = judge(root, baseline)
    if code == 77:
        print(f"  ! {summary}")
        return 77
    if problems:
        for cell, message in problems:
            print(f"  ✗ {cell}：{message}")  # gate-lint:detail
        print(f"  ✗ crates/ 下各包的角色与放置有 {len(problems)} 处不合 .claude/rules/directory-layout.md「crates/：各包放什么」（逐条列在上面）")
        print(f"     → 怎么办：{HOWTO}")
        return 1
    print(f"  ✓ {summary}")
    return 0


# ── 自证 ──
def write_tree(root, files):
    for path, content in files.items():
        full = os.path.join(root, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(content)


def manifest_text(name, dependencies="", extra=""):
    return f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n{dependencies}\n{extra}'


EMPTY_LIBRARY_WITH_COMMENTS = "//! 只放测试的包\n/* 块注释 /* 嵌套 */ 还在注释里 */\n#![deny(\n    unsafe_code\n)]\n"


def green_files():
    """绿样本：产品两个、验证两个（一个空库测试包的测试目标与二进制逐个滞后）、测试支撑一个（一个包加一个别的 cargo 工程无条件依赖它）、实验装置一个。
    验证包的二进制目标：两个登放行表、一个登滞后表、一个 example 登放行表。"""
    return {
        "Cargo.toml": '[workspace]\nmembers = ["crates/p-format", "crates/p-gpu"]\nexclude = ["tools/apparatus"]\n',
        "crates/p-format/Cargo.toml": manifest_text("p-format"),
        "crates/p-format/src/lib.rs": "pub const UNIT_BYTES: u32 = 4096;\n",
        "crates/p-gpu/Cargo.toml": manifest_text("p-gpu", 'p-format = { path = "../p-format" }\n'),
        "crates/p-gpu/src/lib.rs": "pub fn kernel() -> u32 { 1 }\n",
        "crates/p-gpu/tests/kernel_agrees.rs": "#[test]\nfn kernel_agrees() {}\n",
        "crates/p-verify/Cargo.toml": manifest_text("p-verify", 'p-support = { path = "../p-support" }\nformat = { path = "../p-format", package = "p-format" }\n'),
        "crates/p-verify/src/lib.rs": "pub fn judge() -> bool { true }\n",
        "crates/p-verify/src/bin/e7_gate_tool.rs": "fn main() {}\n",
        "crates/p-verify/src/bin/e8_moving_out.rs": "fn main() {}\n",
        "crates/p-verify/src/bin/library_reader.rs": "fn main() {}\n",
        "crates/p-verify/examples/shrink_seed.rs": "fn main() {}\n",
        "crates/p-support/Cargo.toml": manifest_text("p-support"),
        "crates/p-support/src/lib.rs": "pub fn helper() {}\n",
        "crates/p-experiments/Cargo.toml": manifest_text("p-experiments", 'p-gpu = { path = "../p-gpu" }\np-verify = { path = "../p-verify" }\n'),
        "crates/p-experiments/src/bin/e9_measure.rs": "fn main() {}\n",
        "crates/p-legacy-tests/Cargo.toml": manifest_text("p-legacy-tests", 'p-gpu = { path = "../p-gpu" }\n'),
        "crates/p-legacy-tests/src/lib.rs": "\n",
        "crates/p-legacy-tests/src/bin/e3_old.rs": "fn main() {}\n",
        "crates/p-legacy-tests/tests/one.rs": "#[test]\nfn one() {}\n",
        "crates/p-legacy-tests/tests/two.rs": "#[test]\nfn two() {}\n",
        "crates/mutations.tsv": "# 不是包\n",
        "tools/apparatus/Cargo.toml": manifest_text("apparatus", "", '[dev-dependencies]\nsupport = { path = "../../crates/p-support", package = "p-support" }\n'),
        ROLE_TABLE: ("# 登记表\n"
                     "p-format\t产品\tcrates/p-format\t格式常量，样本里的产品\n"
                     "p-gpu\t产品\tcrates/p-gpu\t产品 GPU 路径，测试在包内\n"
                     "p-verify\t验证\tcrates/p-verify\t验证包，门禁起的工具在 src/bin/\n"
                     "p-support\t测试支撑\tcrates/p-support\t两处依赖的测试支撑\n"
                     "p-experiments\t实验装置\tcrates/p-experiments\t实验包，只放装置二进制\n"
                     "p-legacy-tests\t验证\tcrates/p-legacy-tests\t拆包进行中的测试包\n"),
        LAG_TABLE: ("crates/p-legacy-tests/tests/one.rs\t# 拆包进行中，搬回被测包时删这一行\n"
                    "crates/p-legacy-tests/tests/two.rs\t# 拆包进行中，搬回被测包时删这一行\n"
                    "crates/p-legacy-tests/src/bin/e3_old.rs\t# 要搬进实验包的装置\n"
                    "crates/p-verify/src/bin/e8_moving_out.rs\t# 要搬进实验包的装置\n"),
        ALLOWED_TABLE: ("crates/p-verify/src/bin/e7_gate_tool.rs\t# 门禁起它，留在验证包\n"
                        "crates/p-verify/src/bin/library_reader.rs\t# 崩溃入口起它读判定库\n"
                        "crates/p-verify/examples/shrink_seed.rs\t# 随机历史判红打的复现行起它\n"),
    }


GREEN_SUMMARY = ("判了 crates/ 下 6 个包（产品 2、验证 2、测试支撑 1、实验装置 1）：都登记了角色、目录对得上；实验装置之外的二进制目标 5 个都登记了（放行 3、滞后 2），"
                 "库为空而有测试目标的只有测试支撑包与滞后表里的测试目标（滞后 2），测试支撑包都有至少 2 个无条件的依赖者（数了 7 份清单）")


def without(files, *paths):
    return {path: content for path, content in files.items() if path not in paths}


def replaced(files, path, old, new):
    changed = dict(files)
    changed[path] = changed[path].replace(old, new)
    return changed


def with_tests_package(green, library, extra_files=None):
    """绿样本加一个库为 library、带两个测试目标、没登滞后表的验证包 p-x-tests。"""
    return dict(green, **{
        "crates/p-x-tests/Cargo.toml": manifest_text("p-x-tests", 'p-gpu = { path = "../p-gpu" }\n'),
        "crates/p-x-tests/src/lib.rs": library,
        "crates/p-x-tests/tests/alpha.rs": "#[test]\nfn alpha() {}\n",
        "crates/p-x-tests/tests/beta.rs": "#[test]\nfn beta() {}\n",
        ROLE_TABLE: green[ROLE_TABLE] + "p-x-tests\t验证\tcrates/p-x-tests\t为产品 GPU 包的测试另开的包\n"}, **(extra_files or {}))


def selftest_cases():
    """(格名, 文件, 基准, [(该报的格, 那一句里该有的一段)], 绿样本成功句里该有的一段)；该报的为空是绿样本，要一处都不报、成功句里有那一段。"""
    green = green_files()
    lag_rows_base = green[LAG_TABLE].replace("crates/p-verify/src/bin/e8_moving_out.rs\t# 要搬进实验包的装置\n", "")
    absent = ("absent", "自证：基准里没有这张表")
    return [
        ("green", green, absent, [], GREEN_SUMMARY),
        ("registered", dict(green, **{"crates/p-new/Cargo.toml": manifest_text("p-new"), "crates/p-new/src/lib.rs": "pub fn new() {}\n"}), absent,
         [("registered", "包 p-new 在")], ""),
        ("registered-stale", replaced(green, ROLE_TABLE, "p-format\t产品", "p-gone\t产品\tcrates/p-gone\t删掉了的包\np-format\t产品"), absent,
         [("registered", "登记的 p-gone 在 crates/ 下的 Cargo.toml 里没有")], ""),
        ("nested-package", dict(green, **{"crates/p-gpu/gpu-tests/Cargo.toml": manifest_text("p-gpu-tests"),
                                          "crates/p-gpu/gpu-tests/src/lib.rs": "pub fn shared() {}\n"}), absent,
         [("registered", "crates/p-gpu/gpu-tests 的包 p-gpu-tests 在")], ""),
        ("build-directory-skipped", dict(green, **{"crates/p-gpu/target/CACHEDIR.TAG": "Signature: 8a477f597d28d172789f06886806bc55\n",
                                                   "crates/p-gpu/target/package/p-gpu-0.1.0/Cargo.toml": manifest_text("p-gpu")}), absent, [], GREEN_SUMMARY),
        ("duplicate", replaced(green, ROLE_TABLE, "p-gpu\t产品\tcrates/p-gpu\t产品 GPU 路径，测试在包内\n",
                               "p-gpu\t产品\tcrates/p-gpu\t产品 GPU 路径，测试在包内\np-gpu\t产品\tcrates/p-gpu\t又登记一次\n"), absent,
         [("duplicate", "都登记 p-gpu")], ""),
        ("directory", replaced(green, ROLE_TABLE, "p-gpu\t产品\tcrates/p-gpu", "p-gpu\t产品\tcrates/background-gpu"), absent,
         [("directory", "登记的目录 crates/background-gpu 不对")], ""),
        ("role", replaced(green, ROLE_TABLE, "p-gpu\t产品\tcrates/p-gpu\t产品 GPU 路径，测试在包内\n",
                          "p-gpu\t工具\tcrates/p-gpu\t产品 GPU 路径，测试在包内\np-bad\t产品\n"), absent,
         [("role", "角色「工具」不是"), ("role", "不是四列")], ""),
        ("package-count-unreadable", dict(green, **{"crates/p-broken/Cargo.toml": "[package\nname = \n"}), absent,
         [("package-count", "读不出包名的：crates/p-broken/Cargo.toml")], ""),
        ("package-count-zero", {"crates/mutations.tsv": "# 不是包\n", ROLE_TABLE: "# 空表\n"}, absent, [("package-count", "判到 0 个包")], ""),
        ("product-binary", dict(green, **{"crates/p-gpu/src/bin/e1_x.rs": "fn main() {}\n"}), absent,
         [("product-binary", "crates/p-gpu（产品）有 1 个二进制目标在放行表、滞后表里都没有：crates/p-gpu/src/bin/e1_x.rs（bin e1_x）")], ""),
        ("product-binary-manifest", replaced(green, "crates/p-format/Cargo.toml", "[dependencies]", '[[bin]]\nname = "dump"\npath = "tools/dump.rs"\n\n[dependencies]'),
         absent, [("product-binary", "crates/p-format/tools/dump.rs（bin dump）")], ""),
        ("product-examples", dict(green, **{"crates/p-gpu/examples/e202_gpu_cpu_gc_comparison.rs": "fn main() {}\n"}), absent,
         [("product-binary", "crates/p-gpu/examples/e202_gpu_cpu_gc_comparison.rs（example e202_gpu_cpu_gc_comparison）")], ""),
        ("product-main-rs", dict(green, **{"crates/p-gpu/src/main.rs": "fn main() {}\n"}), absent,
         [("product-binary", "crates/p-gpu/src/main.rs（bin p-gpu）")], ""),
        ("product-entry-allowed", dict(green, **{"crates/p-gpu/src/main.rs": "fn main() {}\n",
                                                 ALLOWED_TABLE: green[ALLOWED_TABLE] + "crates/p-gpu/src/main.rs\t# 产品自己的挂载入口\n"}), absent, [],
         "实验装置之外的二进制目标 6 个都登记了（放行 4、滞后 2）"),
        ("binary-unregistered", dict(green, **{"crates/p-verify/src/bin/region_bytes.rs": "fn main() {}\n"}), absent,
         [("binary-unregistered", "crates/p-verify/src/bin/region_bytes.rs（bin region_bytes）")], ""),
        ("verification-bin-outside-src-bin", replaced(dict(green, **{"crates/p-verify/src/apparatus/e202_gpu_cpu_gc_comparison.rs": "fn main() {}\n"}),
                                                      "crates/p-verify/Cargo.toml", "[dependencies]",
                                                      '[[bin]]\nname = "e202_gpu_cpu_gc_comparison"\npath = "src/apparatus/e202_gpu_cpu_gc_comparison.rs"\n\n[dependencies]'),
         absent, [("binary-unregistered", "crates/p-verify/src/apparatus/e202_gpu_cpu_gc_comparison.rs（bin e202_gpu_cpu_gc_comparison）")], ""),
        ("verification-hyphen-name", dict(green, **{"crates/p-verify/src/bin/e202-gpu-cpu-gc-comparison.rs": "fn main() {}\n"}), absent,
         [("binary-unregistered", "crates/p-verify/src/bin/e202-gpu-cpu-gc-comparison.rs（bin e202-gpu-cpu-gc-comparison）")], ""),
        ("support-binary", dict(green, **{"crates/p-support/src/bin/support_tool.rs": "fn main() {}\n"}), absent,
         [("binary-unregistered", "crates/p-support（测试支撑）有 1 个二进制目标")], ""),
        ("tests-package-new-binary", dict(green, **{"crates/p-legacy-tests/src/bin/e203_gc_new_apparatus.rs": "fn main() {}\n"}), absent,
         [("binary-unregistered", "crates/p-legacy-tests/src/bin/e203_gc_new_apparatus.rs（bin e203_gc_new_apparatus）")], ""),
        ("empty-library-tests", with_tests_package(green, EMPTY_LIBRARY_WITH_COMMENTS), absent,
         [("empty-library-tests", "crates/p-x-tests（验证）库是空的、却有 2 个测试目标不在滞后表里（crates/p-x-tests/tests/alpha.rs、crates/p-x-tests/tests/beta.rs）")], ""),
        ("tests-package-one-line-library", with_tests_package(green, "pub use p_gpu;\n"), absent,
         [("empty-library-tests", "crates/p-x-tests（验证）库是空的")], ""),
        ("tests-package-reexport-module", with_tests_package(green, "pub mod reexports {\n    pub use p_gpu::kernel;\n}\nmod shared;\n",
                                                             {"crates/p-x-tests/src/shared.rs": "pub(crate) use p_gpu::kernel as shared_kernel;\n"}), absent,
         [("empty-library-tests", "crates/p-x-tests（验证）库是空的")], ""),
        ("library-with-module-items", with_tests_package(green, "pub mod judge;\n", {
            "crates/p-x-tests/src/judge.rs": "pub fn judge() -> bool { true }\n",
            LAG_TABLE: green[LAG_TABLE]}), absent, [], GREEN_SUMMARY.replace("6 个包（产品 2、验证 2", "7 个包（产品 2、验证 3").replace("数了 7 份清单", "数了 8 份清单")),
        ("empty-library-file-absent", without(with_tests_package(green, "", {
            "crates/p-x-tests/src/main.rs": "fn main() {}\n",
            ALLOWED_TABLE: green[ALLOWED_TABLE] + "crates/p-x-tests/src/main.rs\t# 门禁起它，留在验证包\n"}), "crates/p-x-tests/src/lib.rs"), absent,
         [("empty-library-tests", "crates/p-x-tests（验证）库是空的、却有 2 个测试目标不在滞后表里")], ""),
        ("tests-package-new-test-target", dict(green, **{"crates/p-legacy-tests/tests/three.rs": "#[test]\nfn three() {}\n"}), absent,
         [("empty-library-tests", "crates/p-legacy-tests（验证）库是空的、却有 1 个测试目标不在滞后表里（crates/p-legacy-tests/tests/three.rs）")], ""),
        ("support-dependents", replaced(green, "tools/apparatus/Cargo.toml", 'support = { path = "../../crates/p-support", package = "p-support" }', ""), absent,
         [("support-dependents", "crates/p-support（测试支撑）只有 1 个无条件的依赖者（crates/p-verify/Cargo.toml）")], ""),
        ("support-dependents-cfg", replaced(green, "tools/apparatus/Cargo.toml", "[dev-dependencies]", "[target.'cfg(any())'.dev-dependencies]"), absent,
         [("support-dependents", "crates/p-support（测试支撑）只有 1 个无条件的依赖者（crates/p-verify/Cargo.toml）")], ""),
        ("support-dependents-optional", replaced(green, "crates/p-verify/Cargo.toml", 'p-support = { path = "../p-support" }', 'p-support = { path = "../p-support", optional = true }'),
         absent, [("support-dependents", "crates/p-support（测试支撑）只有 1 个无条件的依赖者（tools/apparatus/Cargo.toml）")], ""),
        ("support-dependents-other-cargo-projects",
         dict(replaced(replaced(green, "crates/p-verify/Cargo.toml", 'p-support = { path = "../p-support" }\n', ""),
                       "Cargo.toml", 'exclude = ["tools/apparatus"]', 'exclude = ["tools/apparatus", "tools/replay"]'),
              **{"tools/replay/Cargo.toml": manifest_text("replay", 'p-support = { path = "../../crates/p-support" }\n')}),
         absent, [], "测试支撑包都有至少 2 个无条件的依赖者（数了 8 份清单）"),
        ("experiment-dependency", replaced(green, "crates/p-gpu/Cargo.toml", "[dependencies]", '[dev-dependencies]\nmeasure = { path = "../p-experiments", package = "p-experiments" }\n\n[dependencies]'),
         absent, [("experiment-dependency", "crates/p-gpu/Cargo.toml 依赖实验装置包 p-experiments")], ""),
        ("experiment-dependency-cfg", replaced(green, "crates/p-gpu/Cargo.toml", "[dependencies]",
                                               "[target.'cfg(any())'.dev-dependencies]\nmeasure = { path = \"../p-experiments\", package = \"p-experiments\" }\n\n[dependencies]"),
         absent, [("experiment-dependency", "crates/p-gpu/Cargo.toml 依赖实验装置包 p-experiments")], ""),
        ("product-support-dependency", replaced(green, "crates/p-gpu/Cargo.toml", "[dependencies]\n", '[dependencies]\np-support = { path = "../p-support" }\n'), absent,
         [("product-support-dependency", "crates/p-gpu/Cargo.toml 的 [dependencies] 依赖测试支撑包 p-support")], ""),
        ("product-support-build-dependency", replaced(green, "crates/p-gpu/Cargo.toml", "[dependencies]",
                                                      '[build-dependencies]\nsupport = { path = "../p-support", package = "p-support", optional = true }\n\n[dependencies]'),
         absent, [("product-support-dependency", "crates/p-gpu/Cargo.toml 的 [build-dependencies] 依赖测试支撑包 p-support")], ""),
        ("product-support-dependency-cfg", replaced(green, "crates/p-format/Cargo.toml", "[dependencies]",
                                                    "[target.'cfg(unix)'.dependencies]\np-support = { path = \"../p-support\" }\n\n[dependencies]"),
         absent, [("product-support-dependency", "crates/p-format/Cargo.toml 的 [target.cfg(unix).dependencies] 依赖测试支撑包 p-support")], ""),
        ("product-support-dev-dependency-allowed", replaced(green, "crates/p-gpu/Cargo.toml", "[dependencies]",
                                                            '[dev-dependencies]\np-support = { path = "../p-support" }\n\n[dependencies]'),
         absent, [], "产品包的 [dependencies]、[build-dependencies] 里没有测试支撑包"),
        ("lag-table-stale", dict(green, **{LAG_TABLE: green[LAG_TABLE] + "crates/p-format/src/lib.rs\t# 本来就不犯的文件\ncrates/p-gone/src/bin/e1_gone.rs\t# 删掉了的装置\n"}), absent,
         [("lag-table", "crates/p-format/src/lib.rs 不是没登放行表的二进制目标"), ("lag-table", "crates/p-gone/src/bin/e1_gone.rs 不在")], ""),
        ("lag-table-package-row", dict(green, **{LAG_TABLE: "crates/p-legacy-tests\t# 拆包进行中，拆完删这一行\ncrates/p-verify/src/bin/e8_moving_out.rs\t# 要搬进实验包的装置\n"}), absent,
         [("lag-table", "crates/p-legacy-tests 是目录")], ""),
        ("allowed-table", dict(green, **{ALLOWED_TABLE: green[ALLOWED_TABLE] + "crates/p-verify/src/lib.rs\t# 不是二进制目标\n"
                                                                            "crates/p-verify/src/bin/e11_gone.rs\t# 删掉了的工具\n"}), absent,
         [("allowed-table", "crates/p-verify/src/lib.rs 不是实验装置之外的包里的二进制目标"), ("allowed-table", "e11_gone.rs 不在")], ""),
        ("lag-grown", green, ("table", lag_rows_base), [("lag-grown", "多出 1 行（只缩不涨）：第 4 行 crates/p-verify/src/bin/e8_moving_out.rs")], ""),
        ("lag-baseline-unreadable", green, ("unreadable", "自证：基准那一版对象库里读不出"), [("lag-grown", "取不到基准那一版")], ""),
    ]


def run_case(case):
    """一格在自己的临时目录里建树、判；交 (格名, 对不对, 一句)。基准是 ("table", 文本) 的先写进临时文件。"""
    name, files, baseline, wanted, summary_part = case
    with tempfile.TemporaryDirectory(prefix="crate-roles-check-selftest-") as work:
        tree = os.path.join(work, "tree")
        write_tree(tree, files)
        if baseline[0] == "table":
            base_file = os.path.join(work, "base-lag.tsv")
            with open(base_file, "w", encoding="utf-8") as handle:
                handle.write(baseline[1])
            baseline = ("table", base_file)
        code, problems, summary = judge(tree, baseline)
    if not wanted:
        ok = code == 0 and not problems and summary_part in summary
        return name, ok, f"绿样本该一处都不报、成功句里有「{summary_part}」，实际退 {code}、报 {problems}、成功句「{summary}」"
    missing = [(cell, text) for cell, text in wanted if not any(cell == found and text in message for found, message in problems)]
    return name, code == 1 and not missing, f"该在 {'、'.join(cell for cell, _ in wanted)} 报出含 {[text for _, text in missing]} 的句子，实际退 {code}、报 {problems}"


def run_selftest():
    try:
        jobs = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 取不了每台默认重活线程数：{error}")
        print(f"     → 怎么办：{error.howto}")
        return 2
    cases = selftest_cases()
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        results = list(pool.map(run_case, cases))
    script = os.path.realpath(__file__)
    environment = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
    outside = subprocess.run([sys.executable, script, REPOSITORY_ROOT], capture_output=True, text=True,
                             env=dict(environment, **{BREAK_VARIABLE: "role-unchecked"}))
    results.append(("break-outside-selftest", outside.returncode == 2 and BREAK_VARIABLE in outside.stderr and "自证" in outside.stderr,
                     f"平常跑设了弄坏开关该退 2、stderr 点名它只在自证里认，实际退 {outside.returncode}、stderr「{outside.stderr.strip()[:200]}」"))
    usage = subprocess.run([sys.executable, script, "--no-such-option"], capture_output=True, text=True, env=environment)
    results.append(("usage", usage.returncode == 2, f"认不得的参数该退 2，实际退 {usage.returncode}"))
    failed = [result for result in results if not result[1]]
    for name, _passed, message in failed:
        print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    if failed:
        print(f"  ✗ --selftest：{len(results)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查判法；设了弄坏开关时红在文件头点名的格是对的")
        return 1
    print(f"  ✓ --selftest：{len(results)} 格都对（并行 {jobs} 路；绿样本与产品入口登了放行、编译目录不进、库里有条目的几种不报，没登记、登记的包不在、嵌在子目录里的包、"
          "登两行、目录不对、角色认不得、包数为 0 与读不出、产品包与验证包、测试支撑包没登记的二进制目标（src/bin/、src/main.rs、examples/、[[bin]] 指到别处、连字符名）、"
          "空库带没登滞后表的测试目标（只有注释、只有再导出、再导出放在模块里、库文件不在）、测试支撑无条件的依赖者不够（条件依赖与可选依赖不算；"
          "crates/ 之外根 Cargo.toml 点名的 cargo 工程算依赖者）、依赖实验装置包（条件依赖也算）、"
          "产品包经 [dependencies]、[build-dependencies] 依赖测试支撑包（条件依赖与可选依赖也算，[dev-dependencies] 照放）、"
          "滞后表用不着的行与整包行、放行表用不着的行、滞后表涨了与基准取不到都报；平常跑设了开关退 2、认不得的参数退 2）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        broken.update(item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item in BREAKS)
        return run_selftest()
    root, baseline, rest = None, ("absent", "没给基准"), list(arguments)
    while rest:
        argument = rest.pop(0)
        if argument in ("--lag-baseline", "--lag-baseline-absent", "--lag-baseline-unreadable") and rest:
            baseline = ({"--lag-baseline": "table", "--lag-baseline-absent": "absent", "--lag-baseline-unreadable": "unreadable"}[argument], rest.pop(0))
        elif not argument.startswith("-") and root is None:
            root = argument
        else:
            print(f"  ✗ 认不得的参数：{argument}")
            print("     → 怎么办：用法 crate-roles-check.py [根] [--lag-baseline <文件> | --lag-baseline-absent <原因> | --lag-baseline-unreadable <原因>]，或 --selftest")
            return 2
    root = os.path.realpath(root) if root else REPOSITORY_ROOT
    if not os.path.isdir(root):
        print(f"  ✗ 根不在：{root}")
        print("     → 怎么办：给仓根，或不给参数判这份脚本所在的仓")
        return 2
    if baseline[0] == "table" and not os.path.isfile(baseline[1]):
        print(f"  ✗ --lag-baseline 给的文件不在：{baseline[1]}")
        print("     → 怎么办：给基准那一版滞后表落成的文件，基准里没有这张表的改给 --lag-baseline-absent <原因>")
        return 2
    return report(root, baseline)


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在自证里认：命令行不是单独一个 --selftest 而设了 CRATE_ROLES_CHECK_BREAK，参数解析之前就拒，交回退出码 2；放行交回 None。"""
    if not os.environ.get(BREAK_VARIABLE) or arguments == ["--selftest"]:
        return None
    print(f"  ✗ {BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没判", file=sys.stderr)
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> python3 scripts/gate/crate-roles-check.py --selftest（开关写在文件头）",
          file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = refuse_break_outside_selftest(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
