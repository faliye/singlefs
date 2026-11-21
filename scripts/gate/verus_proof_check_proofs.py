"""形式化证明判法（入口 verus-proof-check.py）的一块：区域问题与 MIR 比对、判法单元 JUDGING_ENTRIES 与 judging_units、各格的判定（check_toolchain、check_proof、check_vacuity、check_must_red_worlds）、并行验证、工具版本、参数解析、逐 crate 的循环、list-items。
判法单元的入口在这一份里（judging_units 按 __file__ 取入口文件，check_proof 等与它同文件）。入口 verus-proof-check.py 把这一块的名字再导出。
"""

import collections
import concurrent.futures
import contextlib
import fcntl
import glob
import io
import hashlib
import json
import os
import queue
import re
import shutil
import sys
import tempfile
import threading
import time
import tomllib
from heavy_threads import default_heavy_threads  # noqa: E402
from lib_verdict_library import FunctionUnitIndex, UnitKey, VerdictLibrary, digest_text, digest_tree, judging_unit_key
import lib_reuse_checks as reuse_checks  # noqa: E402
from lib_function_units import rust_lex  # noqa: E402
import lib_verus_trait_impl_lag as trait_impl_lag  # noqa: E402  放过表只缩不涨的判法与它的自证格（trait_impl_lag_growth、selftest_cells）

from verus_proof_check_base import (
     REPOSITORY_ROOT, REGISTRY, EXIT_GREEN, EXIT_NOT_RUN, EXIT_RED, EXIT_USAGE, RLIMIT_EXHAUSTED, EXPECTED_PROOF_ERRORS, EXPECTATION_HOWTO,
     OUTER_MEMORY_CAP_VARIABLE, TOOL_VERSION_FILES, RUSTUP_DEFAULT_HOME, EXCLUDED_PACKAGE_FILES, TRUST_LIST, CFG_KIND, TRUST_ROW_HOWTO,
     TRAIT_IMPLS_OUTSIDE_LAG, TRAIT_IMPL_OUTSIDE_HOWTO, RLIMIT_ATTRIBUTE, FUNCTION_LINE, LANDING_NO_LOCATION, LANDING_OUTSIDE, EXPECTED_LANDING,
     LANDING_HOWTO, FUNCTION_HEADING, BREAK, FORMAL_BREAK, ITEM_TALLY_FILE_VARIABLE, ITEM_KEY_PREFIX, Red, NotRun, report_red, one_line, report_cell,
     strip_line_comment, Registry, crate_identifier, read_registry, kernel_files, verus_directory, read_version_json, crate_manifest, copy_crate,
     kernel_module, verus_arguments, forget_crate_fingerprints, verus_program, verus_program_arguments, crate_package_name, diagnostic_path,
     located_file, kernel_modules, run_verus, diagnostic_blocks, proof_error_messages, integer_counts, outcome_problem, outcome_judged, problem_howto,
     killed_signal, rustc_pin_problems, cheating_markers, rlimit_attributes, read_trust_list, lag_baseline_library, read_trait_impls_outside_lag,
     report_passed_not_run, trust_list_text, test_functions, innermost_unit, body_open,
)
from verus_proof_check_listing import (
     unit_path, region_spans, trusted_declarations, normalized_mir, mir_alias_text, mir_items, newest_mir, crate_registry, prepared, UsageError,
     unit_index, verus_ranges, impl_parts, unit_attribute_words, in_verified_region, verus_functions, DERIVED_VERIFIED_TRAITS, DERIVED_CLONE, EXEC_FN,
     PROOF_FN, SPEC_FN_WITH_DECREASES, ASSUME_SPECIFICATION, VALUED_CONST, PLAIN_PATH, ProofItem, function_mode, derived_traits, valued_const_entries,
     verus_listing, verus_function_matches, assume_specification_entries, owner_function, crate_build_text, dependency_component, function_components,
     entry_unit_component, proof_entry_unit, assume_specification_components, PLAN_STATES, plan_done, judged_how, plan_sentinel, sentinel_lookup,
     selected_items, record_item_done, report_item, cell_tally, finish_tally, merged_tally, proof_item, results_text, verification_clean,
     item_kinds_text, listed_names,
)

# ── 四格 ──

def check_toolchain(root, options):
    registry = read_registry(root)
    for view in registry.crates:
        kernel_files(root, view)
    problems = []
    directory = verus_directory(registry, options.get("verus_root"))
    version = read_version_json(directory)
    if version is None:
        problems.append((f"Verus 安装 {directory} 里没有读得出的 version.json", f"装登记的发布 {registry.tag}，或用 SINGLEFS_VERUS_ROOT 指到它的安装目录"))
    else:
        if version.get("version") != registry.tag:
            problems.append((f"Verus 安装的 version 是 {version.get('version')}，登记的是 {registry.tag}", f"换装 {registry.tag}；要升级先改 {REGISTRY} 并复核同源"))
        if version.get("commit") != registry.commit and "commit-unchecked" not in BREAK:
            problems.append((f"Verus 安装的 commit 是 {version.get('commit')}，登记的是 {registry.commit}", f"换装 {registry.tag}；要升级先改 {REGISTRY} 并复核同源"))
    zip_path = os.path.join(os.path.dirname(directory), "verus.zip")
    zip_note = "安装目录旁没有 verus.zip，没核它的 sha256"
    if os.path.isfile(zip_path):
        digest = hashlib.sha256(open(zip_path, "rb").read()).hexdigest()
        zip_note = f"verus.zip 的 sha256 与登记相同"
        if digest != registry.zip_sha256:
            problems.append((f"{zip_path} 的 sha256 是 {digest}，登记的是 {registry.zip_sha256}", "重新下载登记的发布包，或把改过的那份挪走"))
    lock_path = os.path.join(root, "Cargo.lock")
    locked = {}
    if not os.path.isfile(lock_path):
        problems.append(("仓根没有 Cargo.lock", "cargo generate-lockfile 之后照登记钉版本"))
    else:
        for package in tomllib.load(open(lock_path, "rb")).get("package", []):
            locked.setdefault(package.get("name"), []).append(package.get("version"))
    cargo_home = options.get("cargo_home") or os.environ.get("CARGO_HOME") or os.path.join(os.path.expanduser("~"), ".cargo")
    for package in registry.packages:
        name, version_text = package["name"], package["version"]
        if os.path.isfile(lock_path) and locked.get(name) != [version_text]:
            problems.append((f"Cargo.lock 里 {name} 的版本是 {locked.get(name) or '没有'}，登记的是 [{version_text}]", f"依赖写成 {name} = \"={version_text}\"（或经 vstd 带进来）再 cargo update -p {name} --precise {version_text}"))
        directories = sorted(glob.glob(os.path.join(cargo_home, "registry", "src", "*", f"{name}-{version_text}")))
        if not directories:
            problems.append((f"本机 cargo 缓存（{cargo_home}/registry/src/*/）里没有 {name}-{version_text}", "在仓根跑 cargo fetch --locked"))
        for package_directory in directories:
            vcs_path = os.path.join(package_directory, ".cargo_vcs_info.json")
            try:
                vcs = json.load(open(vcs_path, encoding="utf-8"))
            except (OSError, ValueError):
                vcs = {}
            if vcs.get("git", {}).get("sha1") != package["sha1"] or vcs.get("path_in_vcs") != package["path_in_vcs"]:
                problems.append((f"{package_directory} 的 .cargo_vcs_info.json 是 sha1={vcs.get('git', {}).get('sha1')} path_in_vcs={vcs.get('path_in_vcs')}，"
                                 f"登记的是 {package['sha1']} {package['path_in_vcs']}", "缓存被别的来源顶掉了：删掉这个目录再 cargo fetch --locked"))
            fingerprint, _count = package_source_fingerprint(package_directory)
            if fingerprint != package["fingerprint"] and "fingerprint-unchecked" not in BREAK:
                problems.append((f"{package_directory} 的源码内容指纹是 {fingerprint}，登记的是 {package['fingerprint']}",
                                 "缓存里的源码被改过或换了来源：删掉这个目录再 cargo fetch --locked；确要换包就先改登记并复核同源"))
    rustc_note = rustc_pin_problems(root, registry, version, options, problems)
    if problems:
        for message, howto in problems:
            report_red(message, howto)
        return EXIT_RED
    print(f"  ✓ Verus {registry.tag}（commit {registry.commit[:12]}…）与登记相同，{zip_note}；Cargo.lock 与本机缓存里 {len(registry.packages)} 个包的版本、出处与源码指纹都与登记相同；"
          f"{rustc_note}；按条：toolchain 一项，不复用（每趟现核）")
    return EXIT_GREEN


def region_problems(root, registry, kernels, options):
    """被验区域的几样，交 [(卡在哪, 下一步)]；options["region_note"] 记一句给成功句。
    ① axiom（axiom fn、broadcast axiom）在核里一律判红；② 被验区域里每一处可信声明（TRUST_KINDS）与 cfg 原子都要在信任清单里恰好登记一行：
       种类、文件、点名的条目相同，规格原文的 sha256 与现算的相同，差分测试名在 crates/ 下标了 #[test]；清单里这个 crate 的行在核里找不到那一处的判红（标记集合与清单相等）；
       vstd 恰好一行，版本与内容指纹等于登记表 package vstd 那一行；③ 被验区域里声明的类型与 trait，在 crate 的 src 里 verus! 外面（属性写法也没标）有 impl <Trait> for <类型>
       的判红（cfg(test) 下的不判：不进产品构建；把所在文件连进来的 mod 声明带 cfg(test) 的整份文件同样不判），登记在放过表 TRAIT_IMPLS_OUTSIDE_LAG 里的（所在文件、trait 名、实现类型三样对上）放过、成功句逐个点名
       「契约没证、靠信任」；表里这个 crate 的行在源码里已挪进 verus! 或找不到的判红（impl 还在外面、trait 与类型此刻都不在被验区域里的不红，成功句点名），
       trait 写的全路径与被验的那个不同判红，比 gate 基准那一版涨了判红（lib_verus_trait_impl_lag.py 的 trait_impl_lag_growth）。
    弄坏开关（FORMAL_VERIFICATION_BREAK）：axiom-accepted 不判 ①；trust-list-unchecked 不判 ② 的条目行；trust-digest-unchecked 不比 sha256；trust-test-unchecked 不核测试名；
    trust-vstd-unchecked 不核 vstd 行；cfg-atom-accepted 不认 cfg 原子；cfg-macro-unchecked 不认宏里的 cfg；trait-impl-outside-accepted 不判 ③（放过表也不读）。
    放过表那一半的弄坏开关是 VERUS_PROOF_CHECK_BREAK 的 trait-impl-lag-matched-by-trait-only、trait-impl-lag-stale-unchecked、trait-impl-lag-growth-unchecked、
    trait-impl-lag-passed-unnamed、trait-impl-lag-family-unrecognized、trait-impl-lag-family-main-excluded、trait-impl-lag-family-wide、
    trait-impl-lag-family-sum-unchecked、trait-impl-lag-family-python-split-unrecognized、trait-impl-lag-family-hook-split-unrecognized、
    trait-impl-lag-family-shell-split-unrecognized、trait-impl-lag-family-directory-entry-unrecognized（文件头逐项写）。"""
    problems = []
    manifest = crate_manifest(root, registry)
    crate = crate_identifier(registry, manifest)
    workspace = unit_index(root, options).workspace()
    crate_files = sorted(os.path.relpath(path, root).replace(os.sep, "/")
                         for path in glob.glob(os.path.join(root, registry.crate_directory, "src", "**", "*.rs"), recursive=True))
    try:
        parsed = {relative: workspace.file(relative) for relative in crate_files}
    except (OSError, ValueError) as error:
        raise Red(f"切不出 {registry.crate_directory} 的源码（scripts/gate/lib_function_units.py）：{error}",
                  "先让 python3 scripts/gate/lib_function_units.py dump <根> --language rust 在这棵树上跑通")
    cfg_macros = set()
    for rust_file in parsed.values():
        for unit in rust_file.units:
            if unit.kind == "macro_rules" and unit.first_token is not None and "cfg" in rust_file.texts[unit.first_token:unit.last_token + 1][2:]:
                cfg_macros.add(unit.short)
    kernel_relatives = [os.path.relpath(kernel, root).replace(os.sep, "/") for kernel in kernels]
    declarations, axioms = [], []
    verified_types, verified_traits, verified_trait_paths = set(), set(), {}
    for relative in kernel_relatives:
        rust_file = parsed.get(relative) or workspace.file(relative)
        module = kernel_module(registry, manifest, relative)
        spans = region_spans(rust_file)
        for index, word in enumerate(rust_file.texts):
            if word == "axiom" and rust_file.kinds[index] == "ident":
                axioms.append(f"{relative}:{rust_file.lines[index]}（{unit_path(crate, module, innermost_unit(rust_file, index))}）")
        declarations += trusted_declarations(rust_file, relative, crate, module, spans, cfg_macros)
        ranges = verus_ranges(rust_file)
        for unit in rust_file.units:
            if unit.container is None and unit.kind in ("struct", "enum", "union", "type", "trait") and in_verified_region(rust_file, unit, ranges):
                (verified_traits if unit.kind == "trait" else verified_types).add(unit.short)
                if unit.kind == "trait":
                    verified_trait_paths.setdefault(unit.short, set()).add(unit_path(crate, module, unit))
    if "axiom-accepted" not in FORMAL_BREAK:
        for where in axioms:
            problems.append((f"核里有 axiom：{where}（axiom fn 与 broadcast axiom 不验、直接当真）",
                             "axiom 一律不许：把它写成 proof fn 证出来；确是可信的外部事实，写成 uninterp spec fn 或 external_body 的 exec fn，逐条登记进信任清单 " + TRUST_LIST))
    cfg_declarations = [entry for entry in declarations if entry["kind"] == CFG_KIND]
    if "cfg-atom-accepted" in FORMAL_BREAK:
        declarations = [entry for entry in declarations if entry["kind"] != CFG_KIND]
        cfg_declarations = []
    listed = read_trust_list(root)
    checked_rows = 0
    if "trust-list-unchecked" not in FORMAL_BREAK or cfg_declarations:
        if listed is None:
            problems.append((f"信任清单 {TRUST_LIST} 不在（被验区域里要登记的 {len(declarations)} 处：{listed_names(entry['kind'] + ' ' + entry['point'] for entry in declarations)}）",
                             f"建这张表：头一行 vstd、版本、内容指纹、理由（版本与指纹照 {REGISTRY} 的 package vstd 行），再逐处登记；{TRUST_ROW_HOWTO}"))
        else:
            vstd_rows, rows, written_problems = listed
            problems += [(problem, f"照 {TRUST_LIST} 文件头的格式改") for problem in written_problems]
            vstd_package = next((package for package in registry.packages if package["name"] == "vstd"), None)
            if "trust-vstd-unchecked" not in FORMAL_BREAK:
                if len(vstd_rows) != 1:
                    problems.append((f"信任清单里 vstd 行有 {len(vstd_rows)} 行，要恰好一行（vstd 按版本整体登记）", f"照 {REGISTRY} 的 package vstd 行写一行 vstd、版本、内容指纹、理由"))
                elif vstd_package is None or (vstd_rows[0][0], vstd_rows[0][1]) != (vstd_package["version"], vstd_package["fingerprint"]):
                    problems.append((f"信任清单第 {vstd_rows[0][2]} 行的 vstd {vstd_rows[0][0]}（指纹 {vstd_rows[0][1][:12]}…）与登记表 package vstd 行"
                                     f"（{vstd_package['version'] + '，指纹 ' + vstd_package['fingerprint'][:12] + '…' if vstd_package else '没有这一行'}）不同",
                                     "换了 vstd 就照新的 package vstd 行一起改这一行（信任的是那一版 vstd 的全部规格），并复核同源"))
            mine = [row for row in rows if row["file"].startswith(registry.crate_directory + "/")]
            if registry.all_crate_directories and registry.crate_directory == registry.all_crate_directories[0]:
                for row in rows:
                    if not any(row["file"].startswith(directory + "/") for directory in registry.all_crate_directories):
                        problems.append((f"信任清单第 {row['number']} 行的文件 {row['file']} 不在登记的哪个 crate 里", "删掉这一行，或照新路径改"))
            tests = test_functions(root) if "trust-test-unchecked" not in FORMAL_BREAK else set()
            used = set()
            for entry in declarations:
                if entry["kind"] != CFG_KIND and "trust-list-unchecked" in FORMAL_BREAK:
                    continue
                matched = [row for row in mine if (row["kind"], row["file"], row["point"]) == (entry["kind"], entry["file"], entry["point"])]
                wanted_row = f"{entry['kind']}\\t{entry['file']}\\t{entry['point']}\\t{entry['digest']}\\t<差分测试名>\\t<理由>"
                if not matched:
                    problems.append((f"{entry['file']}:{entry['line']} 的 {entry['kind']} {entry['point']} 没登记进信任清单 {TRUST_LIST}（这一处的规格原文 sha256 {entry['digest']}）",
                                     f"{TRUST_ROW_HOWTO}；这一处那一行：{wanted_row}；cfg 原子能不用就不用（被验区域里一律判红，确要的才登记）"))
                    continue
                if len(matched) > 1:
                    problems.append((f"信任清单第 {'、'.join(str(row['number']) for row in matched)} 行登记的是同一处 {entry['kind']} {entry['point']}", "留一行"))
                row = matched[0]
                used.add(row["number"])
                checked_rows += 1
                if row["digest"] != entry["digest"] and "trust-digest-unchecked" not in FORMAL_BREAK:
                    problems.append((f"信任清单第 {row['number']} 行 {entry['kind']} {entry['point']} 登记的规格原文 sha256 是 {row['digest'][:12]}…，现算的是 {entry['digest']}"
                                     f"（{entry['file']}:{entry['line']} 的规格改过）",
                                     "规格改了就先让那条差分测试对着新规格跑过，再照现算的 sha256 改这一行；不是有意改的就照 git 找回原来的规格"))
                if "trust-test-unchecked" not in FORMAL_BREAK and row["test"].split("::")[-1] not in tests:
                    problems.append((f"信任清单第 {row['number']} 行 {entry['kind']} {entry['point']} 点名的差分测试 {row['test']} 在 crates/ 下找不到标了 #[test] 的函数",
                                     "先写那条差分测试（把这份规格与真实现在一批输入上逐值比，规格写错时它要红），再照它的函数名改这一行"))
            if "trust-list-unchecked" not in FORMAL_BREAK:
                for row in mine:
                    if row["number"] not in used and (row["kind"] != CFG_KIND or "cfg-atom-accepted" not in FORMAL_BREAK):
                        problems.append((f"信任清单第 {row['number']} 行登记了 {row['kind']} {row['point']}（{row['file']}），核的被验区域里没有这一处",
                                         "那一处删了或改了名：删掉这一行，或照新的点名改（信任清单与被验区域里的标记一一对应）"))
    trait_impls, passed, outside_note = [], [], "verus! 外面的 trait impl 0 处"
    if "trait-impl-outside-accepted" not in FORMAL_BREAK:
        lag_rows, lag_problems = read_trait_impls_outside_lag(root)
        problems += [(problem, f"照 {TRAIT_IMPLS_OUTSIDE_LAG} 文件头的格式改") for problem in lag_problems]
        mine_lag = [row for row in lag_rows if row["file"].startswith(registry.crate_directory + "/")]
        if registry.all_crate_directories and registry.crate_directory == registry.all_crate_directories[0]:
            for row in lag_rows:
                if not any(row["file"].startswith(directory + "/") for directory in registry.all_crate_directories):
                    problems.append((f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {row['number']} 行的文件 {row['file']} 不在登记的哪个 crate 里", "删掉这一行，或照新路径改"))
        inside_impls, unverified_impls, used_rows = set(), set(), set()
        for relative, rust_file in parsed.items():
            ranges = verus_ranges(rust_file)
            for unit in rust_file.units:
                if unit.kind != "impl" or unit.first_token is None:
                    continue
                trait, target = impl_parts(unit.name[unit.name.rfind("<impl"):] if "<impl" in unit.name else unit.name)
                if trait is None or any(re.search(r"\btest\b", str(condition[1] if isinstance(condition, tuple) else condition)) for condition in unit.conditions) or trait_impl_lag.in_test_only_module_file(workspace, relative, unit, BREAK):
                    continue
                if in_verified_region(rust_file, unit, ranges):
                    inside_impls.add((trait, target, relative))
                    continue
                if not (trait in verified_traits or target in verified_types):
                    unverified_impls.add((trait, target, relative))
                    continue
                where = f"{relative}:{unit.first_line} {unit.name}"
                # 登记行按（所在文件、trait 名、实现类型）认；弄坏开关 trait-impl-lag-matched-by-trait-only 下不看实现类型（同一个 trait 的别的 impl 也放过）
                row = next((row for row in mine_lag if row["file"] == relative and row["trait"].split("::")[-1] == trait
                            and (row["type"] == target or "trait-impl-lag-matched-by-trait-only" in BREAK)), None)
                if row is None:
                    trait_impls.append(where)
                    continue
                used_rows.add(row["number"])
                if trait in verified_trait_paths and row["trait"] not in verified_trait_paths[trait]:
                    problems.append((f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {row['number']} 行写的 trait 是 {row['trait']}，被验的那个是 {'、'.join(sorted(verified_trait_paths[trait]))}（{where}）",
                                     "照被验区域里那个 trait 的全路径（<crate>::<模块>::<trait>）改这一行"))
                    continue
                passed.append((row, where))
        for where in trait_impls:
            problems.append((f"被验的类型或 trait 在 verus! 外面有 trait impl：{where}（verus! 外面的 impl Verus 不看，trait 的 ensures 在这里没人核）", TRAIT_IMPL_OUTSIDE_HOWTO))
        unneeded = []
        for row in mine_lag:
            if row["number"] in used_rows:
                continue
            key = (row["trait"].split("::")[-1], row["type"], row["file"])
            if key in unverified_impls:
                unneeded.append(row)   # impl 还在外面，它的 trait 与类型此刻都不在被验区域里（那一簇的补丁还没打上）：没放过什么，成功句点名
                continue
            if "trait-impl-lag-stale-unchecked" in BREAK:
                continue
            if key in inside_impls:
                problems.append((f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {row['number']} 行登记放过的 {row['trait']} for {row['type']}（{row['file']}）已经挪进 verus! 里了",
                                 "删掉这一行（表只缩不涨，挪进去的那一行就该删）"))
            else:
                problems.append((f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {row['number']} 行登记放过的 {row['trait']} for {row['type']}（{row['file']}）在那份文件里找不到"
                                 "（verus! 外面与里面都没有这个 impl）",
                                 "impl 删了就删掉这一行；改了名、挪了文件的，照现在的写法改这一行（不许借这一行放过别的 impl）"))
        growth_note = ""
        if mine_lag:
            growth_problems, growth_note = trait_impl_lag.trait_impl_lag_growth(root, mine_lag, lag_baseline_library(), TRAIT_IMPLS_OUTSIDE_LAG, BREAK)
            problems += growth_problems
        options["trait_impls_passed"] = passed   # check_proof 判完往 $GATE_NOT_RUN_FILE 逐个报本次未跑（report_passed_not_run）
        if passed:
            named = ("、".join(f"{row['trait']} for {row['type']}（{where.split(' ', 1)[0]}，契约没证、靠信任，归「{row['cluster']}」挪进 verus!）" for row, where in passed)
                     if "trait-impl-lag-passed-unnamed" not in BREAK else f"{len(passed)} 处")
            outside_note = f"verus! 外面的 trait impl {len(passed)} 处，都登记在 {TRAIT_IMPLS_OUTSIDE_LAG} 里放过、这一格不算绿到底：{named}"
        if unneeded:
            outside_note += (f"；放过表里 {len(unneeded)} 行的 trait 与类型此刻都不在被验区域里、没放过什么："
                             + "、".join(f"第 {row['number']} 行 {row['trait']} for {row['type']}" for row in unneeded))
        if mine_lag:
            outside_note += f"（放过表：{growth_note}）"
    cfg_count = sum(1 for entry in declarations if entry["kind"] == CFG_KIND)
    options["region_note"] = (f"被验区域：可信声明与 cfg 原子 {len(declarations)} 处都登记在信任清单里（其中 cfg 原子 {cfg_count} 处）、核对了 {checked_rows} 行，"
                              f"axiom 0 处，被验类型与 trait {len(verified_types) + len(verified_traits)} 个、{outside_note}，vstd 按版本整体登记一行")
    return problems


def compare_mir(root, registry, manifest, executable, options):
    """产品构建（cargo rustc --emit=mir，产品的 rustc；路径不裁短：RUSTC_BOOTSTRAP=1 -Z trim-diagnostic-paths=false，与 Verus 印的同一种写法）与
    Verus 构建（直接起的 verus --no-verify --compile -- --emit=mir；cargo 那一种 cargo verus build -- --no-verify -- --emit=mir）的 MIR 逐条目比：产品构建的每个条目在 Verus 构建里都有、
    整段逐字相同；Verus 构建多出来的（spec fn、proof fn 擦出来的空壳）不比。比之前两边照 MIR_PATH_ALIASES 规整（normalized_mir），成功句与红句报各改写几处。
    交 ([(卡在哪, 下一步)], 一句给成功句)。弄坏开关 mir-aliased-items-skipped 下改写过的条目整个不比（真分叉也放过）。
    两边各在这一份副本里编，编译目录各一份，比完删。弄坏开关 FORMAL_VERIFICATION_BREAK=mir-compare-skipped 下不比。"""
    if "mir-compare-skipped" in FORMAL_BREAK:
        return [], "MIR 比对没跑（弄坏开关 mir-compare-skipped）"
    identifier = crate_identifier(registry, manifest)
    package = crate_package_name(registry)
    howto = ("两边编出的函数体不同：多半是 cfg 原子或宏里的 cfg 让 Verus 看的与产品编的不是同一份（verus_keep_ghost、feature、debug_assertions），"
             "把分叉去掉；确要的分叉连同它登记的那一行一起重想（信任清单只管规格，不替两边的执行代码作保）")
    with tempfile.TemporaryDirectory(prefix="verus-proof-check-mir-") as work:
        copy = copy_crate(root, registry, work)
        if not registry.cargo_mode:
            # 直接起的那一种：产品构建在 crate 副本里单独成一个工作区编（MIR 里 impl 的位置写 src/…，与 verus 在 crate 副本里起印的相同）
            with open(os.path.join(copy, "Cargo.toml"), "a", encoding="utf-8") as handle:
                handle.write("\n# verus-proof-check.py 的 MIR 比对：副本自成一个工作区\n[workspace]\n")
            if os.path.isfile(os.path.join(root, "Cargo.lock")):
                shutil.copy2(os.path.join(root, "Cargo.lock"), os.path.join(copy, "Cargo.lock"))
        product_target, verus_target = os.path.join(work, "product-target"), os.path.join(work, "verus-target")
        # 产品的 cargo 照 PATH 找（自证把假 cargo 放在 PATH 最前面）
        product = run_verus("cargo", ["rustc", "-p", package, "--lib", "--offline", "--", "--emit=mir", "-Z", "trim-diagnostic-paths=false"], copy,
                            extra_environment={"CARGO_TARGET_DIR": product_target, "RUSTC_BOOTSTRAP": "1"})
        if registry.cargo_mode:
            built = run_verus(verus_program(registry, executable),
                              verus_program_arguments(registry, verus_arguments(registry, manifest, options["threads"]) + ["--no-verify", "--", "--emit=mir"], build=True),
                              copy, extra_environment={"CARGO_TARGET_DIR": verus_target})
            verus_mir = newest_mir(verus_target, identifier)
        else:
            verus_mir = os.path.join(work, "verus.mir")
            built = run_verus(verus_program(registry, executable),
                              verus_program_arguments(registry, verus_arguments(registry, manifest, options["threads"]) + ["--no-verify", "--compile", "--", "--emit=mir", "-o", verus_mir]),
                              copy)
        problems = []
        verus_label = "Verus 构建（cargo verus build -- --no-verify -- --emit=mir）" if registry.cargo_mode else "Verus 构建（verus --no-verify --compile -- --emit=mir）"
        for label, outcome in (("产品构建（cargo rustc --emit=mir）", product), (verus_label, built)):
            if killed_signal(outcome) is not None or outcome["exit"] != 0:
                tail = " | ".join(line for line in outcome.get("diagnostics", "").strip().split("\n")[-6:] if line.strip())
                problems.append((f"MIR 比对：{label} 没编成（退 {outcome['exit']}）：{outcome_problem(outcome, options) or tail[-400:]}",
                                 problem_howto(outcome, "照报错修；产品构建编不过先让 cargo build 过，Verus 构建编不过先看完整 crate 证明那一步")))
        if problems:
            return problems, ""
        product_mir = newest_mir(product_target, identifier)
        if product_mir is None or verus_mir is None or not os.path.isfile(verus_mir):
            return [(f"MIR 比对：找不到 MIR 文件（产品构建 {product_mir or '没有'}，Verus 构建 {verus_mir or '没有'}）",
                     "cargo 或 Verus 换了 --emit=mir 的落点：照新落点改 scripts/gate/verus-proof-check.py 的 newest_mir")], ""
        product_text = open(product_mir, encoding="utf-8", errors="replace").read()
        verus_text = open(verus_mir, encoding="utf-8", errors="replace").read()
    # 同一个函数两边印法不同的名字先照规整表写成一种（MIR_PATH_ALIASES），函数体别处一个字都不碰
    product_normalized, product_rewrites = normalized_mir(product_text)
    verus_normalized, verus_rewrites = normalized_mir(verus_text)
    product_items, verus_items = mir_items(product_normalized), mir_items(verus_normalized)
    aliases = mir_alias_text(product_rewrites, verus_rewrites)
    missing = [name for name in product_items if name not in verus_items]
    different = [name for name in product_items if name in verus_items and product_items[name] != verus_items[name]]
    if "mir-aliased-items-skipped" in BREAK:
        # 弄坏开关：改写过的条目整个当相同、不比（同一个条目里另有真分叉也放过）
        raw_items = mir_items(product_text)
        different = [name for name in different if raw_items.get(name) == product_items[name]]
    if not product_items:
        return [("MIR 比对：产品构建的 MIR 里一个条目都切不出来", "看 mir_items 认的条目头（fn / const / static / promoted）还对不对得上 rustc 的印法")], ""
    if missing or different:
        return [(f"MIR 比对：产品构建 {len(product_items)} 个条目里，{len(different)} 个与 Verus 构建的函数体不同（{listed_names(different, 10)}），"
                 f"{len(missing)} 个在 Verus 构建里没有（{listed_names(missing, 10)}）；{aliases}",
                 howto + "；不同行只差一个名字的路径（std:: 对 core:: 这类），先核两边是不是同一个函数（两份 rlib 引用的符号相同），"
                 "是就往 scripts/gate/verus-proof-check.py 的 MIR_PATH_ALIASES 加一行，不按前缀整片放过")], ""
    return [], (f"产品构建与 {'cargo verus build' if registry.cargo_mode else 'verus --compile'} 构建的 MIR 逐条目比：产品的 {len(product_items)} 个条目都在、整段逐字相同"
                f"（Verus 构建另有 {len(verus_items) - len(product_items)} 个擦出来的条目，不比；{aliases}）")


# ── 按条判定：键与复用（判定库 lib_verdict_library.py；判据在文件头「逐项点名与按条复用」一段）──

# 判法单元：三格各自判结局的那个函数（连同它调到的；判定库与 lib_function_units.py 两份不进），取这份脚本所在的仓
JUDGING_ENTRIES = {"proof": "check_proof", "vacuity": "check_vacuity", "must-red-worlds": "check_must_red_worlds"}
_JUDGING_UNITS = {}


def judging_units(cell):
    """判法单元指纹：这一格判结局的函数（JUDGING_ENTRIES）在这份脚本里的单元闭包键。自证给 options["judging_units"] 时用那一个（造「判法单元指纹变了」）；
    弄坏开关 reuse-key-without-judging-units 下一律交空串（判法变了照旧复用）。"""
    if "reuse-key-without-judging-units" in BREAK:
        return ""
    if cell not in _JUDGING_UNITS:
        try:
            _JUDGING_UNITS[cell] = judging_unit_key(REPOSITORY_ROOT, [f"{os.path.relpath(os.path.realpath(__file__), REPOSITORY_ROOT)}::{JUDGING_ENTRIES[cell]}"])
        except ValueError as error:
            _JUDGING_UNITS[cell] = UnitKey(None, (), (f"refused\tjudging-units\t{os.path.basename(__file__)}:1\t{error}",))
    return _JUDGING_UNITS[cell]


def proof_items(root, registry, kernels, functions, options):
    """证明这一格的项：有证明义务的单元各一项，按核与行号的次序交 [ProofItem]，与没成项的 [(名, 核的相对路径)]。
    成项的四类：proof fn、exec fn（各一条，exec fn 里每个 while / loop / for 另一条：Verus 把循环体单独验）、带 decreases 的 spec fn（终止义务一条）、
    verus! 里 derive(Clone) 的类型（Verus 验它生成的 <类型>::clone，一条）。spec fn 不带 decreases 的没有证明义务（真 Verus 逐函数验报 verified 0）：
    不成项，随用到它的项进那一项的单元闭包键；trait 里没有函数体的声明与 external_body 的 fn 也不成项。项名 proof:<全路径>（<crate>::<模块>::<名>），
    交给 --verify-function 的 <名> 是 Verus 可点名函数表里恰好那一个（自由 fn 的名字、<类型>::<方法>、<trait>::<方法>、<类型>::clone），
    Verus 有恰好同名的就取它，不会点到别人。另两类不是 fn、Verus 整 crate 验时照样各算条数：verus! 里带值的 const（valued_const_entries）与
    assume_specification（assume_specification_entries），各一项。弄坏开关：spec-fn-as-item 下 spec fn 一律成项，spec-decreases-not-item 下带 decreases 的 spec fn
    不成项，derived-clone-not-item 下 derive(Clone) 不成项，valued-const-not-item、assume-specification-not-item 下那两类不成项。"""
    workspace = unit_index(root, options).workspace()
    manifest = crate_manifest(root, registry)
    crate = crate_identifier(registry, manifest)
    by_kernel = {}
    for function in functions:
        by_kernel.setdefault(function.relative, []).append(function)
    items, left_out = [], []
    for kernel in kernels:
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        rust_file = workspace.file(relative)
        module = kernel_module(registry, manifest, relative)
        ranges = verus_ranges(rust_file)
        entries = []
        for function in by_kernel.get(relative, []):
            name, unit = function.name, function.unit
            mode = function_mode(rust_file, unit)
            body = rust_file.texts[unit.first_token:unit.last_token]
            # trait 里只有签名与契约、没有函数体的声明：Verus 不验它（义务落在各 impl 上）。按切出的单元认（body_open：单元以 ; 收尾），
            # ensures 里的结构体字面量不算函数体；弄坏开关 trait-declaration-by-brace 下退回「单元里一个 { 都没有」
            declared_only = ("{" not in body) if "trait-declaration-by-brace" in BREAK else body_open(rust_file, unit) is None
            if unit.container is not None and unit.container.kind == "trait" and declared_only:
                left_out.append((function.path, relative))
                continue
            if "external_body" in unit_attribute_words(rust_file, unit):
                left_out.append((function.path, relative))   # external_body：函数体不验，规格照信任清单登记（region_problems），不成项
                continue
            if mode == "proof":
                kind, estimate = PROOF_FN, 1
            elif mode == "exec":
                kind, estimate = EXEC_FN, 1 + sum(1 for text in body if text in ("while", "loop", "for"))
            elif "decreases" in body and "spec-decreases-not-item" not in BREAK:
                kind, estimate = SPEC_FN_WITH_DECREASES, 1
            elif "spec-fn-as-item" in BREAK:
                kind, estimate = "spec fn", 0
            else:
                left_out.append((function.path, relative))
                continue
            entries.append((unit.first_token, ProofItem(proof_item(function.path), name, relative, unit, kind, f"{relative}::{unit.name}", estimate)))
        if "derived-clone-not-item" not in BREAK:
            for unit in rust_file.units:
                if unit.kind not in ("struct", "enum", "union") or unit.first_token is None or unit.container is not None \
                        or not any(low < unit.first_token < high for low, high in ranges):
                    continue
                for trait in DERIVED_VERIFIED_TRAITS:
                    derived = derived_traits(rust_file, unit)
                    # 弄坏开关 derived-clone-copy-only：只认同时 derive(Copy) 的（把 Verus 那条「clone is not a copy」的警告当成没有义务）
                    if trait in derived and ("derived-clone-copy-only" not in BREAK or "Copy" in derived):
                        verus_name = f"{unit.short}::{trait.lower()}"
                        full = "::".join(part for part in (crate, module, verus_name) if part)
                        entries.append((unit.first_token, ProofItem(proof_item(full), verus_name, relative, unit, DERIVED_CLONE, f"{relative}::{unit.short}", 1)))
        entries += valued_const_entries(rust_file, relative, crate, module, ranges)
        entries += assume_specification_entries(rust_file, relative, crate, module)
        point_name_problems(relative, [item for _position, item in entries], verus_listing(rust_file, relative, ranges))
        items += [item for _position, item in sorted(entries, key=lambda pair: pair[0])]
    return items, left_out


def point_name_problems(relative, items, listing):
    """一份核（一个模块）里每一项交给 --verify-function 的名字，照 Verus 的点法（verus_function_matches）在这个模块的可点名函数表（verus_listing）里
    要恰好命中一项：命中几项 Verus 就一起验几项（带 *）或报 more than one match（不带 *），逐项点不准，判红并点名命中的那几项；一项都没命中是表与点名对不上
    （这份判法的毛病），也判红。fn 之间的同名在 verus_functions 里先判了。弄坏开关 point-names-by-containment 下用 point_names_by_containment 的判法。"""
    if "point-names-by-containment" in BREAK:
        point_names_by_containment(relative, items)
        return
    names = [name for name, _where in listing]
    for item in items:
        hits = verus_function_matches(item.verus_name, names)
        if len(hits) == 1:
            continue
        if not hits:
            raise Red(f"{relative} 的 {item.item} 交给 --verify-function 的 {item.verus_name} 在这个模块的 Verus 可点名函数表里一项都点不中"
                      f"（表里 {len(names)} 项：{listed_names(names)}）",
                      "这是 verus-proof-check.py 的 verus_listing 与点名（verus_point_name、valued_const_point_name、assume_specification_entries）对不上："
                      "用真 Verus 在这个模块上点一个不在的名字，照它列的可点名函数表改这两处")
        raise Red(f"{relative} 的 {item.item} 交给 --verify-function 的 {item.verus_name} 在这个模块的 Verus 可点名函数表里命中 {len(hits)} 项："
                  f"{'、'.join(listing[index][1] for index in hits)}：Verus 会把它们一起验（或报 more than one match），逐项点不准",
                  "给撞上的 const 或函数改名，让这个模块里只有它一个以 ::<名> 收尾（关联 const）或恰好叫这个名字，同步改必红世界表里引它的行")


def point_names_by_containment(relative, items):
    """弄坏开关 point-names-by-containment 下的判法：只看关联 const 与 std 方法的 assume_specification 两类，只在成项的几项之间比名字是不是别人的一段，
    不照 Verus 的点法、不看表里不成项的函数与 const。"""
    names = collections.Counter(item.verus_name for item in items)
    for item in items:
        if item.kind not in (VALUED_CONST, ASSUME_SPECIFICATION):
            continue
        if names[item.verus_name] > 1:
            raise Red(f"{relative} 的被验区域里有 {names[item.verus_name]} 项交给 --verify-function 的名字都是 {item.verus_name}（{item.item} 是其中一项）",
                      "给其中一个改名，同步改必红世界表里引它的行")
        exact = item.kind == ASSUME_SPECIFICATION and PLAIN_PATH.fullmatch(item.verus_name) and "::" in item.verus_name \
            or item.kind == VALUED_CONST and item.unit.container is None
        clashing = [other.verus_name for other in items if other is not item and item.verus_name in other.verus_name]
        if not exact and clashing:
            raise Red(f"{relative} 的 {item.item} 交给 --verify-function 的 {item.verus_name} 是 {listed_names(clashing)} 的一段",
                      "给 const 或那几个函数改名，让它的名字不是别的函数名的一段")


def toolchain_digest(root, options):
    """「工具链」那一样的摘要（tool_versions_text 的 for_key 那一行取摘要；要哈希 Verus 安装目录与 rustup 工具链 lib/ 下约 500 MB 的文件）。
    options["toolchain_digests"] 给了（字典：run_per_crate 与 list-items --key-mismatches 在一趟里逐 crate、逐格的几次调用共用一份）就按（根，Verus 安装目录）存取，
    同一趟同一个安装目录只算一次；没给每次现算。弄坏开关 toolchain-digest-shared-across-installs：共用的那一份只按根存，换了 Verus 安装目录取到的还是上一个目录的摘要。"""
    cache = options.get("toolchain_digests")
    slot = (root,) if "toolchain-digest-shared-across-installs" in BREAK else (root, options.get("verus_root"))
    if cache is not None and slot in cache:
        return cache[slot]
    digest = digest_text(tool_versions_text(root, options.get("verus_root"), for_key="toolchain-key-with-paths" not in BREAK))
    if cache is not None:
        cache[slot] = digest
    return digest


def shared_components(root, registry, options):
    """Verus 每一项都带的几样：工具链（tool-versions 那一行去掉路径的那一份，tool_versions_text 的 for_key：Verus、它自带的 vstd、z3、rustup 工具链的版本与内容摘要）、
    rlimit（登记的数，改变结论）、crate 起法（crate_build_text：cargo 还是直接起、edition、包名）。
    verus-pins.tsv 整份不进（登记表的注释、verified 的登记数与别的行不改任何一项的结论）。
    弄坏开关 toolchain-key-with-paths：工具链那一样照 tool-versions 打的整行（带 RUSTUP_HOME、rustup、VERUS_ROOT、VERUS_Z3_PATH 的路径）；
    reuse-key-without-crate-build：crate 起法不进。算一次存进 options；没给判定库时不算（交空表，lookup 也不看它）。"""
    if not options.get("verdict_library"):
        return []
    if options.get("shared_components") is None:
        components = []
        if "reuse-key-without-toolchain" not in BREAK:
            components.append(("工具链", toolchain_digest(root, options)))
        components.append(("rlimit", digest_text(str(registry.rlimit))))
        if "reuse-key-without-crate-build" not in BREAK:
            components.append(("crate 起法", digest_text(crate_build_text(root, registry))))
        # 信任清单（vstd 行与这个 crate 的条目行）：登记的规格就是证明的前提，改一行这个 crate 的每一项都现判
        if "reuse-key-without-trust-list" not in FORMAL_BREAK:
            components.append(("信任清单", digest_text(trust_list_text(root, registry))))
        options["shared_components"] = components
    return options["shared_components"]


def verdict_library(options, kind, cell):
    version = judging_units(cell) if "reuse-key-without-judging-units" in BREAK else (options.get("judging_units") or judging_units(cell))
    store = VerdictLibrary(options.get("verdict_library"), kind, version)
    store.plan = collections.Counter() if options.get("plan") is not None else None
    store.plan_items = [] if options.get("plan") is not None else None   # [(项, 状态)]：只数键那一趟逐项的键状态（ITEM_KEY 行的原料）
    return store


def check_proof(root, options):
    """证明逐项：有证明义务的单元各一项（proof_items：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 的 clone；spec fn 不带 decreases 的不成项），
    各验一次（--verify-root 或 --verify-only-module <核所在的模块>，再 --verify-function <名>；kernel_module）。每项判据：出结局、没有 vir 错、errors 0（verification_clean：--verify-function 下不看 success 与
    is-verifying-entire-crate）、verified 至少 1（0 是 Verus 在点的函数上一条义务都没验：点错了函数，或它不该成项）、跑的 Verus 的 commit 等于登记。
    整格跑时各项 verified 之和（复用的取记下的数；按 --verify-function 点的名字计，同名的只算一次）要等于登记的数（verus-pins.tsv 的 crate 行，整 crate 验的数），
    不等判红并点名：verified 与静态估的义务条数不同的项、没成项的 verus! fn。--item proof 或不给 --item 是整格；--item proof:<名> 只验点名的，不比之和。"""
    registry, kernels, manifest, executable = prepared(root, options)
    modules = kernel_modules(root, registry, manifest, kernels)
    # 被验区域的另几样：信任清单逐条登记、axiom、cfg 原子、被验类型的 trait impl 在 verus! 外面。
    # 先于切函数判：按 cfg 分叉的两份同名 fn 在切函数那一步就红（逐项点不准），红句要先说 cfg 原子
    region = region_problems(root, registry, kernels, options)
    try:
        functions = verus_functions(root, registry, kernels, options)
        items, left_out = proof_items(root, registry, kernels, functions, options)
    except Red as red:
        if not region:
            raise
        report_cell(region + [(red.message, red.howto)])
        return EXIT_RED
    if not items:
        raise Red(f"{len(kernels)} 份核的 verus! 里一个有证明义务的单元都没有（verus! 里 {len(functions)} 个 fn，{len(left_out)} 个是不带 decreases 的 spec fn）：证明这一格没有项",
                  "核里的证明写在 verus! { … } 里的 proof fn 与 exec fn 上；核挪了地方就改 .claude/gate.d/verus-pins.tsv 的 kernel 行")
    available = [entry.item for entry in items]
    named = options.get("items") or []
    wanted = [item for item in named if item != "proof"]
    # 点名里有整格 proof：点名的 proof:<名> 加在整格之上，整格照跑（各项之和、完整 crate 证明、MIR 比对），点名的只核项名在不在。
    # 弄坏开关 proof-item-narrows：proof 与 proof:<名> 一起点名时只验点名的那几个（--item proof --item proof:X 丢了整格）
    whole_cell_named = "proof" in named and "proof-item-narrows" not in BREAK
    whole = not wanted or whole_cell_named or "item-filter-ignored" in BREAK
    chosen = selected_items(dict(options, items=wanted), ["proof", *available], "verus-proof") if wanted else available
    if whole_cell_named:
        chosen = available
    selected = [entry for entry in items if entry.item in chosen]
    store, tally = verdict_library(options, "proof-function", "proof"), cell_tally(options)
    shared = shared_components(root, registry, options)
    problems = []
    if "cheating-unchecked" not in BREAK:
        for marker in cheating_markers(root, kernels):
            problems.append((f"核里有不许的标记 {marker}", "被验区域里不许 admit、assume(…)、axiom、verifier::external：把它换成真证明，证不出就照实报，不绕过；"
                             f"std 规格与设备、校验和这类可信声明写成 assume_specification / external_body / uninterp spec fn，逐条登记进信任清单 {TRUST_LIST}"))
        if "rlimit-attribute-unchecked" not in BREAK:
            for marker in rlimit_attributes(root, kernels):
                problems.append((f"核里有不许的标记 {marker}（函数上的 rlimit 属性盖过命令行的 --rlimit {registry.rlimit}，这个函数不在登记的额度上验）",
                                 f"删掉这个属性；证明确要更多额度，照 {REGISTRY} rlimit 行上面的注释在新数上把核与必红世界表整张重验一遍，再调高登记的数"))
    problems += region
    counts, pending = {}, []
    keyed = [(entry, shared + (assume_specification_components(root, registry, options, entry) if entry.kind == ASSUME_SPECIFICATION
                               else function_components(root, registry, options, [entry.entry], "被验函数单元闭包"))
              + entry_unit_component(proof_entry_unit(root, options, entry), "被验入口单元") if shared else []) for entry in selected]
    sentinel = plan_sentinel(options, store, "proof-function", "verus-proof", [(entry.item, components) for entry, components in keyed])
    for entry, components in keyed:
        reused, reason = sentinel_lookup(store, sentinel, entry.item, components)
        verified = ((store.record_of(entry.item) or {}).get("details") or {}).get("verified") if reused else None
        if reused and not str(verified).isdigit():
            reused, reason = False, "记录里没有 verified 数"
        tally.count(reused, reason)
        if reused:
            counts[entry.item] = int(verified)
            report_item(entry.item, "绿", f"复用（{reason}）")
            sentinel.observe(entry.item, reuse_checks.REUSED, "绿")
        else:
            pending.append((entry, components, reason))
    if options.get("plan") is not None:
        return plan_done(options, "verus-proof", store)

    def judge_one(index, outcome):
        entry = pending[index][0]
        item = entry.item
        problem = outcome_problem(outcome, options)
        if problem:
            return [(f"{item}：{problem}", problem_howto(outcome, "看上面 Verus 的报错"))], "", None
        results = outcome["json"]["verification-results"]
        commit = outcome["json"].get("verus", {}).get("commit")
        clean = verification_clean(results, True)
        if not clean and "nonzero-errors-accepted" in BREAK and not results.get("encountered-vir-error") and integer_counts(results):
            clean = True   # 弄坏开关：errors 不是 0 也当验过（只要不是 vir 错）
        if not clean:
            blocks = diagnostic_blocks(outcome["diagnostics"])
            tail = " | ".join(line for line in outcome.get("stderr", "").strip().split("\n")[-6:] if line.strip())
            listed = "；".join(f"{path}:{line} {message}" for message, path, line in blocks[:8]) or tail[-400:] or "stderr 里没有报错"
            return [(f"{item}：Verus 没验过（{entry.kind} {entry.verus_name}）：{results_text(results)}：{listed}",
                     "证明或实现有一处对不上：照报错的位置修；反例是真的就先修实现")], "", None
        if not integer_counts(results):
            return [(f"{item}：Verus 的 JSON 里没有整数的 verified 与 errors（读到 verified {results.get('verified')!r}），各项之和比不了",
                     "Verus 换了 JSON 的写法：照新写法改 scripts/gate/verus-proof-check.py 的 integer_counts")], "", None
        if commit != registry.commit:
            return [(f"{item}：跑的 Verus 报的 commit 是 {commit}，登记的是 {registry.commit}", "SINGLEFS_VERUS_ROOT 指到登记的那份安装")], "", None
        if results["verified"] < 1 and "zero-verified-accepted" not in BREAK and entry.kind != ASSUME_SPECIFICATION:   # 指向函数的 assume_specification 验出 0 条
            return [(f"{item}：Verus 在 --verify-function {entry.verus_name} 上一条义务都没验（{results_text(results)}；它是 {entry.kind}，静态估 {entry.estimate} 条）",
                     "点到的不是要验的那个函数，或它本没有证明义务：看 scripts/gate/verus-proof-check.py 的 proof_items 怎么认它这一类"
                     "（不带 decreases 的 spec fn 不成项），名字与别的函数撞了就改名")], "", None
        counts[entry.item] = results["verified"]
        return [], f"{results['verified']} verified、0 errors", {"verified": str(results["verified"])}

    judging = ItemJudging(store, judge_one, "verus-proof",
                          f"{'cargo verus focus' if registry.cargo_mode else 'verus'} --verify-root|--verify-only-module <核所在的模块> --verify-function <名> "
                          f"--rlimit {registry.rlimit}（{registry.tag}）",
                          sentinel=sentinel)
    started = time.monotonic()
    verify_in_parallel(root, registry, manifest, executable, options,
                       [(entry.item, {}, entry.verus_name, modules[entry.relative]) for entry, _components, _reason in pending],
                       judged=lambda index, outcome, seconds: judging(index, outcome, seconds, pending[index][0].item, pending[index][1],
                                                                      judged_how(store, pending[index][0].item, pending[index][2], sentinel)))
    problems += judging.finish(len(pending))
    problems += sentinel.finish()
    total = sum(counts.values())
    breakdown = item_kinds_text(selected)
    if whole and not problems and total != registry.verified_count and "count-unchecked" not in BREAK:
        off = [f"{entry.item} 验出 {counts[entry.item]}、静态估 {entry.estimate}" for entry in selected
               if entry.item in counts and counts[entry.item] != entry.estimate]
        problems.append((f"各项 verified 之和 {total} 与登记的 {registry.verified_count} 不同（{'少' if total < registry.verified_count else '多'} "
                         f"{abs(registry.verified_count - total)} 条；成项 {len(selected)} 个：{breakdown}，静态估之和 {sum(entry.estimate for entry in selected)}）："
                         f"verified 与静态估不同的项：{listed_names(off)}；没成项的 verus! fn（不带 decreases 的 spec fn，静态估 0 条）{len(left_out)} 个："
                         f"{listed_names(name for name, _relative in left_out)}",
                         "少了是整 crate 验出的义务有一部分不落在成项的单元上：在副本上整 crate 验一次（不带 --verify-function），再逐个 --verify-function 没成项的那几个，"
                         "验出非 0 的那一类补进 scripts/gate/verus-proof-check.py 的 proof_items；多了先看 verified 与静态估不同的那几项；"
                         f"确是加减了证明就照整 crate 验出的数改 {REGISTRY} 的 crate 行，并在改动说明里写加减了哪几个"))
    # 逐函数的计数相同并不表示义务集合完整；完整验证也检查未选中的引理。
    whole_started = time.monotonic()   # 整格这一项（proof）自己的活：各项之和、完整 crate 验证、MIR 比对，都不按条入库，每趟现判
    if whole and not problems:
        outcome = verify_in_parallel(root, registry, manifest, executable, options,
                                     [("完整 crate 证明", {})])[0]
        problem = outcome_problem(outcome, options)
        if problem or not verification_clean((outcome.get("json") or {}).get("verification-results", {}), False) or (outcome.get("json") or {}).get("verification-results", {}).get("is-verifying-entire-crate") is not True:
            problems.append((f"完整 crate 验证失败：{problem or outcome['diagnostics'][-3000:]}",
                             "逐项通过不能替代完整验证：检查漏登记、宏生成或未验证的引理"))
    # 产品构建与 verus --compile 构建的 MIR 逐函数比：整格跑、前面都没红时比一次，不按条入库（同完整 crate 证明）
    mir_note = "MIR 比对没跑（--item 点名、或前面已经红了）"
    if whole and not problems:
        mir_problems, mir_note = compare_mir(root, registry, manifest, executable, options)
        problems += mir_problems
    if whole:
        record_item_done("proof", time.monotonic() - whole_started, 1, 0, 1 if problems else 0)
    finish_tally(tally, "verus-proof", options)
    not_run_note = report_passed_not_run(options, registry)
    if problems:
        report_cell(problems)
        return EXIT_RED
    scope = (f"--item 点名的 {len(selected)} 项（不比之和）" if not whole
             else f"成项 {len(selected)} 个（{breakdown}），各项 verified 之和 {total}、0 errors（登记 {registry.verified_count}）；"
                  f"不成项的 spec fn {len(left_out)} 个（没有证明义务，随用到它们的项进键）")
    print(f"  ✓ Verus 逐项验 {registry.crate_directory}{'（cargo verus focus）' if registry.cargo_mode else ''}：{scope}，--rlimit {registry.rlimit}，"
          f"{len(kernels)} 份核里 0 处不许的标记，{options.get('region_note', '被验区域没扫')}{not_run_note}；{mir_note}；"
          f"空真那一格{assert_by_skipped_text(assert_by_requires_sites(root, kernels))}；"
          f"按条{tally.text()}（{time.monotonic() - started:.1f} 秒）")
    return EXIT_GREEN


def assert_by_requires_lines(text):
    """属于 assert(…) by (…) 的 requires 所在的行（从 1 数）：这种 requires 是那一处要先证出来的前提，不是外层函数的前提，不是空真插入点
    （外层函数入口那一处照旧插 assert(false)）。按词法单元认：requires 前面紧接 assert(…) by (…)（两组配对的圆括号，中间是 by，头一组前面是 assert），
    往后在同一层先碰到 ; 收尾（配对的圆括号、方括号整段跳过；先碰到 { 或 } 的是函数体或别的块，不算）。没有函数体的 fn 声明、trait 里无体的方法、
    挂在普通块或不带 by 的 assert 上的 requires 都不是这个样子，照旧走 requires_insertion_points 的找函数体那一支。
    弄坏开关 assert-by-requires-as-insertion-point 下一处都不认（当插入点、找不到函数体判红）；
    assert-by-requires-by-semicolon-line 下照「往下先碰到一行只有 ; 」认（不看前面接的是不是 assert(…) by (…)：普通块与不带 by 的 assert 上的 requires 也放过）。"""
    if "assert-by-requires-as-insertion-point" in BREAK:
        return []
    if "assert-by-requires-by-semicolon-line" in BREAK:
        lines, found = text.split("\n"), []
        for index, line in enumerate(lines):
            stripped = strip_line_comment(line).strip()
            if stripped != "requires" and not stripped.startswith("requires "):
                continue
            indent = len(line) - len(line.lstrip())
            for later in range(index + 1, len(lines)):
                code = strip_line_comment(lines[later])
                if FUNCTION_LINE.match(code) or (code.strip().startswith("{") and len(code) - len(code.lstrip()) <= indent):
                    break
                if code.strip() == ";":
                    found.append(index + 1)
                    break
        return found
    _kinds, words, token_lines, _docs = rust_lex(text)
    opening_of, closing_of, stack = {}, {}, []
    for position, word in enumerate(words):
        if word in ("(", "[", "{"):
            stack.append(position)
        elif word in (")", "]", "}") and stack and words[stack[-1]] == {")": "(", "]": "[", "}": "{"}[word]:
            opening = stack.pop()
            opening_of[position], closing_of[opening] = opening, position
    found = []
    for position, word in enumerate(words):
        if word != "requires" or position < 1 or words[position - 1] != ")":
            continue
        by_opening = opening_of.get(position - 1)
        if by_opening is None or by_opening < 2 or words[by_opening - 1] != "by" or words[by_opening - 2] != ")":
            continue
        assert_opening = opening_of.get(by_opening - 2)
        if assert_opening is None or assert_opening < 1 or words[assert_opening - 1] != "assert":
            continue
        index = position + 1
        while index < len(words) and words[index] not in (";", "{", "}"):
            index = closing_of[index] + 1 if words[index] in ("(", "[") and index in closing_of else index + 1
        if index < len(words) and words[index] == ";":
            found.append(token_lines[position])
    return found


def assert_by_requires_sites(root, kernels):
    """各份核里属于 assert(…) by (…) 的 requires（assert_by_requires_lines），交 [<相对仓根的核>:<行>]：列项与证明、空真两格的成功句报跳过了哪几处。"""
    sites = []
    for kernel in kernels:
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        sites += [f"{relative}:{line}" for line in assert_by_requires_lines(open(kernel, encoding="utf-8").read())]
    return sites


def assert_by_skipped_text(sites):
    """「跳过 assert-by requires N 处：<文件>:<行>…」（逐处列，0 处写「没有」）。"""
    return f"跳过 assert-by requires {len(sites)} 处（属于 assert(…) by (…)，在那一处先证出来，不是外层函数的前提）：{listed_names(sites, limit=max(1, len(sites)))}"


def requires_insertion_points(text, relative=None):
    """每一处 requires 的函数体头：[(requires 行号, { 所在行的下标, 说明)]；认不出函数体的抛 Red（relative 给了就写进红句）。
    属于 assert(…) by (…) 的 requires（assert_by_requires_lines）不是插入点，跳过、不判独立契约行。"""
    lines = text.split("\n")
    where = f"{relative} " if relative else ""
    skipped = set(assert_by_requires_lines(text))
    kinds, words, token_lines, _docs = rust_lex(text)
    for position, word in enumerate(words):
        if word != "requires" or (position > 0 and words[position - 1] in (".", "::")):
            continue
        line = token_lines[position]
        if line in skipped:
            continue
        prefix = strip_line_comment(lines[line - 1]).strip()
        if not (prefix == "requires" or prefix.startswith("requires ")):
            raise Red(f"{where}第 {line} 行的 requires 不是独立契约行：拒绝漏检",
                      "将 requires 单独起行，函数体的 { 单独起行后再做空真筛查")
    points = []
    for index, line in enumerate(lines):
        stripped = strip_line_comment(line).strip()
        if stripped != "requires" and not stripped.startswith("requires "):
            continue
        if index + 1 in skipped:
            continue
        indent = len(line) - len(line.lstrip())
        brace = None
        for later in range(index + 1, len(lines)):
            code = strip_line_comment(lines[later])
            if FUNCTION_LINE.match(code):
                break
            if code.strip().startswith("{") and len(code) - len(code.lstrip()) <= indent:
                brace = later
                break
        if brace is None:
            raise Red(f"{where}第 {index + 1} 行的 requires 找不到函数体的 {{（没有函数体，或 {{ 没单独起行；也不是 assert(…) by (…) 接 requires … ; 的样子）",
                      "把函数体的 { 单独起一行（verusfmt 的写法）；是 assert 的前提就写成 assert(…) by (…) 接 requires … ;（判法 assert_by_requires_lines）；"
                      "确是没有函数体的声明，把判法补进 scripts/gate/verus-proof-check.py 的 requires_insertion_points")
        owner = next((FUNCTION_LINE.match(strip_line_comment(lines[back])).group(5) for back in range(index, -1, -1)
                      if FUNCTION_LINE.match(strip_line_comment(lines[back]))), "?")
        inline = lines[brace].split("{", 1)[1].strip()
        label = f"{owner} 里第 {index + 1} 行起的闭包" if inline else f"fn {owner}"
        points.append((index + 1, brace, label))
    return points


def parallel_locations(location_count, options):
    """几处同时验、每处几条线程：交 (处数, 每处的线程数)。处数是 --jobs，0 取 min(处数, max(1, N // 4))；每处 N // 处数条、至少 1；N 是 --threads。
    不按内存收：外层内存上限（OUTER_MEMORY_CAP_VARIABLE）不看，整条命令的内存额度归 task-server（文件头「通用选项」--jobs 一段）。
    弄坏开关 outer-memory-narrows-jobs 下：设了外层上限就收到 1 处。"""
    jobs = options["jobs"] or min(location_count, max(1, options["threads"] // 4))
    if "outer-memory-narrows-jobs" in BREAK and os.environ.get(OUTER_MEMORY_CAP_VARIABLE):
        jobs = 1
    return jobs, max(1, options["threads"] // jobs)


SLOT_DIRECTORY_NAME = "verus-proof-check-targets"


def claim_slot_directories(count):
    """cargo 那一种几处同时验各占的 count 个编译目录：交 ([目录路径], 放掉它们的函数)。
    环境里有绝对路径的 CARGO_TARGET_DIR（任务服务或整轮门禁给的这一趟的编译目录）时，建在它底下 verus-proof-check-targets/target-<编号>：
    一趟里先后几次调用（证明、空真、必红世界三格各一个进程，各 crate，整 crate 证明）共用同一批目录，vstd 与别的依赖只编一次。
    每个编号由 verus-proof-check-targets/target-<编号>.lock 上的一把 flock 占着（进程退出时自动放），同一时刻别的进程占着的编号跳过、取下一个；
    放掉只放锁、不删目录（编译目录归给它的人删）。没有这个变量、不是绝对路径或这个文件系统不认 flock 时，在临时目录里建、放掉时整个删。
    弄坏开关 slot-directory-per-call：一律每次新建临时目录，一趟里几次调用各编各的依赖。"""
    base = os.environ.get("CARGO_TARGET_DIR") or ""
    if "slot-directory-per-call" not in BREAK and os.path.isabs(base):
        parent = os.path.join(base, SLOT_DIRECTORY_NAME)
        claimed, handles, number = [], [], 0
        try:
            os.makedirs(parent, exist_ok=True)
            while len(claimed) < count:
                handle = open(os.path.join(parent, f"target-{number}.lock"), "a")
                try:
                    fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
                except BlockingIOError:
                    handle.close()
                else:
                    handles.append(handle)
                    claimed.append(os.path.join(parent, f"target-{number}"))
                number += 1
        except OSError:
            for handle in handles:
                handle.close()
            claimed = []
        else:
            def release():
                for handle in handles:
                    handle.close()
            return claimed, release
    temporary = tempfile.mkdtemp(prefix="verus-proof-check-targets-")
    return [os.path.join(temporary, f"target-{slot}") for slot in range(count)], lambda: shutil.rmtree(temporary, ignore_errors=True)


def verify_in_parallel(root, registry, manifest, executable, options, edits, judged=None):
    """edits：[(说明, {相对仓根的文件: 新内容}[, 只验的函数名, 它所在的模块])]；每一处一份副本、一次 Verus（给了函数名就只验它：模块是 None 点 --verify-root，
    不是 None 点 --verify-only-module <模块>，再 --verify-function，verus_arguments）。交回与 edits 同序的 outcome。
    judged(下标, outcome, 秒数) 给了就在那一处出结局时当场调（按条判定在这里判完就落盘，打断之后判过的不丢）。
    几处同时、每处几条线程照 parallel_locations。"""
    if not edits:
        return []
    jobs, threads = parallel_locations(len(edits), options)
    # cargo 那一种：几处同时验各占一个编译目录（同一个编译目录 cargo 一次只让一处用），vstd 与依赖在一个编译目录里只编一次
    # （有 CARGO_TARGET_DIR 时一趟里先后几次调用共用同一批目录，claim_slot_directories）；
    # 副本各在新目录里（copy_crate），被验的 crate 每处都现编，不会拿到上一处的结果
    slots = queue.Queue()
    slot_paths, release_slots = claim_slot_directories(jobs) if registry.cargo_mode else ([None] * jobs, lambda: None)
    for slot in slot_paths:
        slots.put(slot)

    def one(index, edit):
        started = time.monotonic()
        slot = slots.get()
        try:
            with tempfile.TemporaryDirectory(prefix="verus-proof-check-") as work:
                crate_copy = copy_crate(root, registry, work)
                for relative, content in edit[1].items():
                    with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                        handle.write(content)
                arguments = verus_program_arguments(registry, verus_arguments(registry, manifest, threads, edit[2] if len(edit) > 2 else None,
                                                                              edit[3] if len(edit) > 3 else None))
                if slot:
                    forget_crate_fingerprints(slot, crate_package_name(registry))
                outcome = run_verus(verus_program(registry, executable), arguments, crate_copy,
                                    extra_environment={"CARGO_TARGET_DIR": slot} if slot else None)
        finally:
            slots.put(slot)
        if judged is not None:
            judged(index, outcome, time.monotonic() - started)
        return outcome

    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
            futures = [pool.submit(one, index, edit) for index, edit in enumerate(edits)]
            return [future.result() for future in futures]
    finally:
        release_slots()


class ItemJudging:
    """一格里逐项判：每项出结局时（verify_in_parallel 的 judged）判出这一项的问题，判绿就当场记进判定库；问题按项的次序交回。
    弄坏开关 reuse-records-whole-cell-only 下不当场记，等整格判完、一处问题都没有才一起记（被打断或有一项红，判过的全不留）；
    reuse-records-red 下红的也记成绿。"""

    def __init__(self, store, judge_one, cell, run="", sentinel=None):
        self.store, self.judge_one, self.cell, self.run, self.sentinel = store, judge_one, cell, run, sentinel
        self.lock = threading.Lock()
        self.created = time.monotonic()
        self.problems, self.lines, self.summaries, self.pending = {}, {}, {}, []

    def __call__(self, index, outcome, seconds, item, components, how):
        """how 是这一项那一行的「现判（原因…）」整句（judged_how）；judge_one 交 (问题, 一句判定[, 随判定存的数])。"""
        judged = self.judge_one(index, outcome)
        problems, summary = judged[0], judged[1]
        details = judged[2] if len(judged) > 2 else None
        failed_record = None
        if not problems or "reuse-records-red" in BREAK:
            if "reuse-records-whole-cell-only" in BREAK:
                with self.lock:
                    self.pending.append((item, components, summary, details))
            else:
                failed_record = self.store.record(item, components, summary, run=self.run, details=details)
        elif outcome_judged(outcome) and "red-keeps-green" not in BREAK:
            # 记红、撤绿（文件头「逐项点名与按条复用」）：判红的照同一把键记一条红（共用库的 record_red），盖掉这把键原有的绿，
            # 下一趟这一项现判；没判成的（被信号停了）不碰库。弄坏开关 red-keeps-green：判红不记（原有的绿照复用）
            failed_record = self.store.record_red(item, components, one_line(problems[0][0])[:400], run=self.run, details=details)
        with self.lock:
            self.problems[index] = problems
            self.summaries[index] = summary
            self.lines[index] = (item, "红" if problems else "绿", how, seconds, failed_record)

    def finish(self, count):
        """按项的次序打每项那一行、交回全部问题；whole-cell-only 下整格没问题才把攒着的一起记。"""
        problems = []
        for index in range(count):
            if index in self.lines:
                item, verdict, how, seconds, failed_record = self.lines[index]
                if "item-done-whole-cell-seconds" in BREAK:   # 弄坏开关：每项都记整格从起到收尾的挂钟，不记这一项自己的
                    seconds = time.monotonic() - self.created
                report_item(item, verdict, how, seconds)
                if self.sentinel is not None:
                    self.sentinel.observe(item, reuse_checks.FORCED if self.sentinel.forced(item) else reuse_checks.JUDGED, verdict)
                if failed_record:
                    print(f"  ! {failed_record}：下一趟这一项照判")
            problems += self.problems.get(index, [])
        if not problems:
            for item, components, summary, details in self.pending:
                self.store.record(item, components, summary, run=self.run, details=details)
        return problems


def with_assert_false(line):
    head, tail = line.split("{", 1)
    return head + "{ assert(false);" + tail


def vacuity_points(root, registry, kernels, functions, options):
    """空真的每一处：[(项名, 说明, {相对仓根的文件: 插了断言的内容}, (相对 crate 的文件, 插的那一行, requires 那一行), 所属函数名, 函数内第几处)]
    与每份核全部插进去的那一份（弄坏开关用）。项名 vacuity:<所属函数名>#<函数内第几处 requires>，只当索引：这一处的身份是所属函数单元与第几处，
    不带路径与行号（空行让 requires 挪一行，项名与键都不变）。functions 是 verus_functions 交的；requires 不在其中任何一个 fn 里判红。
    工作区索引取 options 里已建的那一份（unit_index），不另建。"""
    points, all_at_once = [], []
    workspace = unit_index(root, options).workspace()
    for kernel in kernels:
        text = open(kernel, encoding="utf-8").read()
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        rust_file = workspace.file(relative)
        lines = text.split("\n")
        every_point = list(lines)
        ordinals = {}
        for requires_line, brace, label in requires_insertion_points(text, relative):
            owner = owner_function(functions, relative, requires_line)
            if owner is None:
                raise Red(f"{relative} 第 {requires_line} 行的 requires 不在 verus! 里的任何 fn 里：认不出它属于哪个函数",
                          "requires 只写在 verus! 里的 fn 与它里面的闭包上；确是新写法，把判法补进 scripts/gate/verus-proof-check.py 的 vacuity_points")
            if "external_body" in unit_attribute_words(rust_file, owner.unit):
                continue
            ordinals[owner.path] = ordinals.get(owner.path, 0) + 1
            changed = list(lines)
            later = brace + 1
            last_header = None
            while later < len(lines):
                code = strip_line_comment(lines[later]).strip()
                if not code:
                    later += 1
                    continue
                if re.match(r"^(?:hide|reveal|reveal_with_fuel)\b", code):
                    last_header = later
                    later += 1
                    continue
                break
            if last_header is not None:
                insert_line = last_header
                indent = " " * (len(lines[insert_line]) - len(lines[insert_line].lstrip()))
                changed[insert_line] = changed[insert_line] + f"\n{indent}assert(false);"
                every_point[insert_line] = every_point[insert_line] + f"\n{indent}assert(false);"
                points.append((f"vacuity:{owner.path}#{ordinals[owner.path]}", f"{relative} {label}", {relative: "\n".join(changed)},
                               (diagnostic_path(registry, relative), insert_line + 2, requires_line), owner, ordinals[owner.path]))
            else:
                changed[brace] = with_assert_false(changed[brace])
                every_point[brace] = with_assert_false(every_point[brace])
                points.append((f"vacuity:{owner.path}#{ordinals[owner.path]}", f"{relative} {label}", {relative: "\n".join(changed)},
                               (diagnostic_path(registry, relative), brace + 1, requires_line), owner, ordinals[owner.path]))
        all_at_once.append((relative, {relative: "\n".join(every_point)}))
    return points, all_at_once


def vacuity_item_problems(label, expected, outcome, options):
    """空真一处的问题（(卡在哪, 下一步)），交回 (问题, 一句判定)。"""
    path, line, requires_line = expected
    problem = outcome_problem(outcome, options)
    if problem:
        return [(f"{label}（第 {requires_line} 行的 requires）：{problem}",
                 problem_howto(outcome, "插断言的副本判不出来：看报错，修好再跑"))], ""
    results = outcome["json"]["verification-results"]
    if verification_clean(results, True):
        if "vacuity-accepts-verified" not in BREAK:
            return [(f"{label}：函数体头插 assert(false) 照样验得过——第 {requires_line} 行起的前置条件不可满足（空真），这个函数的 ensures 什么也没证",
                     "改前置条件让它可满足（常见：两条互相矛盾、范围写反）；改完这一格插的 assert(false) 要在那一行报 assertion failed")], ""
        return [], "验得过（弄坏开关 vacuity-accepts-verified）"
    blocks = diagnostic_blocks(outcome["diagnostics"])
    if not any("assertion failed" in message and path == block_path and line == block_line for message, block_path, block_line in blocks):
        if "vacuity-all-at-once" in BREAK and "闭包" in label:
            return [], ""
        listed = "；".join(f"{block_path}:{block_line} {message}" for message, block_path, block_line in blocks[:6])
        return [(f"{label}：插的 assert(false) 在 {path}:{line}，Verus 报的错不在那一行（{listed or '没有配上位置的报错'}）",
                 "插入点认错了或副本编不过：看 requires_insertion_points 认的 {{ 是不是函数体的那一个")], ""
    return [], f"{path}:{line} 报 assertion failed"


def check_vacuity(root, options):
    """空真每处 requires 一项：插了 assert(false) 的副本上只验它所属的那一个函数（--verify-root 或 --verify-only-module <核所在的模块>，再 --verify-function；
    kernel_module），键是所属函数的单元闭包、所属函数单元（entry_unit_component）与函数内第几处。"""
    registry, kernels, manifest, executable = prepared(root, options)
    modules = kernel_modules(root, registry, manifest, kernels)
    functions = verus_functions(root, registry, kernels, options)
    points, all_at_once = vacuity_points(root, registry, kernels, functions, options)
    if not points:
        raise NotRun(f"{len(kernels)} 份核里一处 requires 都没有：没有前置条件，空真这一格无对象可判")
    chosen = selected_items(options, [point[0] for point in points], "verus-vacuity")
    points = [point for point in points if point[0] in chosen]
    store, tally = verdict_library(options, "vacuity-requires", "vacuity"), cell_tally(options)
    shared = shared_components(root, registry, options)
    pending, keyed = [], []
    for item, label, change, expected, owner, ordinal in points:
        components = []
        if shared:
            relative = label.split(" ", 1)[0]
            components = shared + function_components(root, registry, options, [f"{relative}::{owner.unit.name}"], "所属函数单元闭包")
            components += entry_unit_component(owner.unit, "所属函数单元")
            if "reuse-key-without-row" not in BREAK:
                components.append(("函数内第几处", digest_text(str(ordinal))))
            if "vacuity-key-by-line-number" in BREAK:
                components.append(("这一处", digest_text(f"{relative}:{expected[2]}")))
        keyed.append(components)
    sentinel = plan_sentinel(options, store, "vacuity-requires", "verus-vacuity", [(point[0], components) for point, components in zip(points, keyed)])
    for (item, label, change, expected, owner, ordinal), components in zip(points, keyed):
        reused, reason = sentinel_lookup(store, sentinel, item, components)
        tally.count(reused, reason)
        if reused:
            report_item(item, "绿", f"复用（{reason}）")
            sentinel.observe(item, reuse_checks.REUSED, "绿")
        else:
            pending.append((item, label, change, expected, components, reason, owner))
    if options.get("plan") is not None:
        return plan_done(options, "verus-vacuity", store)
    if "vacuity-all-at-once" in BREAK:
        # 弄坏：一份核的全部插入点插进同一份副本只跑一遍、验整个 crate。外层函数入口的 assert(false) 一失败，里面闭包那一处就验得过、不单报
        together = dict(zip((relative for relative, _change in all_at_once),
                            verify_in_parallel(root, registry, manifest, executable, options, all_at_once)))
        problems = []
        for item, label, _change, expected, _components, reason, _owner in pending:
            item_problems, _summary = vacuity_item_problems(label, expected, together[label.split(" ", 1)[0]], options)
            report_item(item, "红" if item_problems else "绿", f"现判（{reason}）")
            problems += item_problems
    else:
        judging = ItemJudging(store, lambda index, outcome: vacuity_item_problems(pending[index][1], pending[index][3], outcome, options), "verus-vacuity",
                              f"{'cargo verus focus' if registry.cargo_mode else 'verus'} --verify-root|--verify-only-module <核所在的模块> --verify-function <所属函数> --rlimit {registry.rlimit}"
                              f"（{registry.tag}），函数体头插 assert(false)",
                              sentinel=sentinel)
        verify_in_parallel(root, registry, manifest, executable, options,
                           [(label, change, owner.name, modules[label.split(" ", 1)[0]]) for _item, label, change, _expected, _components, _reason, owner in pending],
                           judged=lambda index, outcome, seconds: judging(index, outcome, seconds, pending[index][0], pending[index][4],
                                                                          judged_how(store, pending[index][0], pending[index][5], sentinel)))
        problems = judging.finish(len(pending))
        problems += sentinel.finish()
    finish_tally(tally, "verus-vacuity", options)
    if problems:
        report_cell(problems)
        return EXIT_RED
    named = f"（--item 点名的 {len(points)} 处）" if options.get("items") else ""
    print(f"  ✓ 空真：{len(kernels)} 份核里 {len(points)} 处 requires{named}，每一处在函数体头插 assert(false)、只验它所属的函数，都在插的那一行报 assertion failed（未检出入口空真；这不是前置条件可满足性的证明）；"
          f"{assert_by_skipped_text(assert_by_requires_sites(root, kernels))}；按条{tally.text()}")
    return EXIT_GREEN


def must_red_rows(root, registry):
    """必红世界表的行：六段制表符分隔——变异名、文件、原文、替换文、proof_errors=M（条数，只报）、landing=<落点>（判的是它）。
    不是六段的行（或变异名、文件、原文、第五段、第六段有空的）一律判红，报行号、要六段。弄坏开关：row-segments-unchecked 不核段数。"""
    path = os.path.join(root, registry.must_red_table)
    if not os.path.isfile(path):
        raise Red(f"必红世界表 {registry.must_red_table} 不在", "核进了仓就要带它（六段制表符分隔：变异名、文件、原文、替换文、proof_errors=<证明错条数>、landing=<落点>）；路径改了就改登记表的 must-red-worlds 行")
    rows, problems = [], []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        required = (0, 1, 2, 4)
        if "row-segments-unchecked" in BREAK:   # 弄坏开关：段数不核，缺的段当空串往下走
            fields += [""] * (6 - len(fields))
        else:
            required += (5,)
        if len(fields) != 6 or not all(fields[index] for index in required):
            problems.append(f"第 {number} 行不是六段（或变异名、文件、原文、第五段 proof_errors=、第六段 landing= 有空的）")
            continue
        rows.append({"number": number, "name": fields[0], "file": fields[1], "original": fields[2].replace("\\n", "\n"),
                     "replacement": fields[3].replace("\\n", "\n"), "expected": fields[4], "landing_text": fields[5]})
    if problems:
        raise Red(f"{registry.must_red_table} 写坏了：" + "；".join(problems), "要六段制表符分隔：变异名、文件、原文、替换文、proof_errors=<证明错条数>、landing=<落点>；照表头的格式改")
    if not rows:
        raise Red(f"{registry.must_red_table} 一行都没有：核没有一个必须报红的世界", "每一层证明至少一条改回去 Verus 就红的变异（show-me-test.md「设计论证同样照办」）")
    return rows


def must_red_item(row):
    return f"must-red:{row['name']}"


def function_keys(root, options, functions):
    """被验区域里每个 fn 的单元闭包键（{全路径: UnitKey}），一格里算一次存进 options。"""
    if options.get("function_keys") is None:
        index = unit_index(root, options)
        keys = {}
        try:
            for function in functions:
                keys[function.path] = index.key([f"{function.relative}::{function.unit.name}"])
        except ValueError as error:
            raise Red(f"算不出核里函数的单元闭包：{error}", "先让 python3 scripts/gate/lib_function_units.py closure <根> <入口> 在这棵树上跑通")
        options["function_keys"] = keys
    return options["function_keys"]


def must_red_components(root, registry, options, row, text, functions):
    """必红世界一行的键：这一行改坏的单元（原文所在的那几个单元）与 verus! 里闭包用到它们的每个 fn（改坏之后可能报错的就是这几个），
    这些入口并起来的单元闭包键；加原文、替换文与第五段（变异名与文件路径不进：改名、挪表不重判）。没给判定库交空表。
    弄坏开关：must-red-key-with-name-and-path 下变异名与文件进「这一行」；reuse-key-without-row 下「这一行」不进；reuse-key-whole-crate 下单元那一样换回核 crate 整个目录。"""
    if not options.get("verdict_library"):
        return []
    shared = shared_components(root, registry, options)
    pieces = [row["original"], row["replacement"], ",".join(sorted(parse_landing(row["landing_text"]) or ()))]
    if "must-red-key-with-name-and-path" in BREAK:
        pieces = [row["name"], row["file"]] + pieces
    row_component = [] if "reuse-key-without-row" in BREAK else [("这一行", digest_text("\t".join(pieces)))]
    if "reuse-key-whole-crate" in BREAK:
        return shared + [("核 crate", digest_tree(os.path.join(root, registry.crate_directory)))] + row_component
    index = unit_index(root, options)
    position = text.find(row["original"])
    if position == -1:
        first_line = 0
        last_line = 0
    else:
        first_line = text.count("\n", 0, position) + 1
        last_line = first_line + row["original"].count("\n")
    try:
        rust_file = index.workspace().file(row["file"])
    except (OSError, ValueError) as error:
        raise Red(f"第 {row['number']} 行「{row['name']}」的文件 {row['file']} 切不出单元：{error}", "先让 python3 scripts/gate/lib_function_units.py dump <根> --language rust 在这棵树上跑通")
    mutated = [unit for unit in rust_file.units if unit.first_line <= last_line and unit.last_line >= first_line and not unit.data]
    identities = {(unit.kind, unit.name, unit.file) for unit in mutated}
    keys = function_keys(root, options, functions)
    affected = []
    for function in functions:
        members = {(line.split("\t")[0], line.split("\t")[1], line.split("\t")[2].rsplit(":", 1)[0]) for line in keys[function.path].units}
        if identities & members:
            affected.append(f"{function.relative}::{function.unit.name}")
    entries = affected + [f"{row['file']}::{unit.name}" for unit in mutated if f"{row['file']}::{unit.name}" not in affected]
    if not entries:
        return shared + [("受影响的函数单元", digest_text("原文不在任何单元里"))] + row_component
    try:
        units = index.key(entries)
    except ValueError as error:
        raise Red(f"第 {row['number']} 行「{row['name']}」算不出受影响单元的闭包：{error}", "原文所在的单元名字不唯一时让原文只落在一个 fn 里")
    return shared + [("受影响的函数单元", units)] + dependency_component(root, registry, options, units) + row_component


def enclosing_function(rows, line):
    """核文本的行列表 rows 里第 line 行落在哪个 fn 里：往上找最近一行 fn 的开头（FUNCTION_HEADING），交它的名字；往上一行都没有交 None。"""
    for index in range(min(line, len(rows)) - 1, -1, -1):
        found = FUNCTION_HEADING.match(rows[index])
        if found:
            return found.group(1)
    return None


def parse_landing(text):
    """第六段 landing=<落点>[,<落点>…] 读成集合；不是这个形态交 None。"""
    form = EXPECTED_LANDING.fullmatch(text or "")
    return set(form.group(1).split(",")) if form else None


def proof_error_landing(root, registry, files, diagnostics):
    """改坏的副本上 Verus 的证明错（除去 rlimit 用尽）各落在哪个 fn 里，交排好序、去重的落点列表：每条证明错取它 --> 位置的那一行，
    在改坏之后的文件文本里（files 是 {相对仓根的文件: 改坏之后的内容}，别的文件读 root 里的）往上找最近一行 fn 的开头；位置在 crate 之外、文件读不出、
    往上没有 fn 的记（函数外），输出里有证明错没带位置的记（无位置）。落点不数条数：同一个 fn 里多报一条循环不变式，落点还是那一个（C654）。
    弄坏开关 landing-read-from-unmutated-text：读改坏之前的文件（改坏的行数不一样，位置对不上）。"""
    texts = {}

    def rows_of(relative):
        if relative not in texts:
            if relative in files and "landing-read-from-unmutated-text" not in BREAK:
                texts[relative] = files[relative].split("\n")
            else:
                path = os.path.join(root, relative)
                texts[relative] = open(path, encoding="utf-8").read().split("\n") if os.path.isfile(path) else None
        return texts[relative]

    located = [(path, line) for message, path, line in diagnostic_blocks(diagnostics) if RLIMIT_EXHAUSTED not in message]
    found = set()
    for path, line in located:
        rows = None if os.path.isabs(path) else rows_of(located_file(registry, path))
        name = enclosing_function(rows, line) if rows is not None else None
        found.add(name or LANDING_OUTSIDE)
    if len(proof_error_messages(diagnostics)) > len(located):
        found.add(LANDING_NO_LOCATION)
    return sorted(found)


def must_red_row_problems(row, outcome, options, registry, root=None, files=None):
    """必红世界一行改坏之后的结局判出的问题，交回 (问题, 一句判定)；一句判定里带「rlimit 用尽夹证明错」时成功句把它数进去。
    判的是落点（proof_error_landing 与第六段 landing= 相等），条数只报：与第五段 proof_errors= 不同的在一句判定里写「条数只报」，成功句数进去。
    弄坏开关：landing-unchecked 不比落点；landing-by-count 改回按条数判（条数不同判红、不比落点）。"""
    problem = outcome_problem(outcome, options)
    if problem:
        return [(f"第 {row['number']} 行「{row['name']}」：{problem}",
                 problem_howto(outcome, "必红世界要红在证明上：改坏之后编不过的变异不算，换一处只改判定的改法"))], ""
    results = outcome["json"]["verification-results"]
    if not integer_counts(results):
        return [(f"第 {row['number']} 行「{row['name']}」：Verus 的 JSON 里没有整数的 verified 与 errors（读到 verified {results.get('verified')!r}、"
                 f"errors {results.get('errors')!r}、vir 错 {results.get('encountered-vir-error')}），判不出这一行红没红",
                 "vir 错是改坏的副本没走到验证（类型或模式错）：换一处只改判定的改法；没有 vir 错就是 Verus 换了 JSON 的写法："
                 "照新写法改 scripts/gate/verus-proof-check.py 的 integer_counts")], ""
    rlimit_exhausted = RLIMIT_EXHAUSTED in outcome["diagnostics"]
    proof_errors = proof_error_messages(outcome["diagnostics"])
    counted = proof_error_messages(outcome["diagnostics"], count_rlimit="rlimit-counted-in-proof-errors" in BREAK)
    if rlimit_exhausted and (not proof_errors or "rlimit-any-line-voids" in BREAK) and "rlimit-counts-as-red" not in BREAK:
        listed = "；".join(f"{path}:{line} {message}" for message, path, line in diagnostic_blocks(outcome["diagnostics"])[:6])
        return [(f"第 {row['number']} 行「{row['name']}」：Verus 输出里有 {RLIMIT_EXHAUSTED}，除它、aborting due to 与 cargo 的 could not compile 之外没有别的报错（--rlimit {registry.rlimit}；"
                 f"verified {results.get('verified')}、errors {results.get('errors')}：{listed}），这一行的红不作数",
                 f"这一行红在算力额度上，不是证明失败：改替换文或调高登记的 rlimit（{REGISTRY} 的 rlimit 行）")], ""
    landing = proof_error_landing(root, registry, files or {}, outcome["diagnostics"]) if root is not None else []
    registered_count = row.get("proof_errors")
    summary = f"落点 {'、'.join(landing) or '无'}；证明错 {len(counted)} 条" + ("，rlimit 用尽夹证明错" if rlimit_exhausted else "")
    if registered_count is not None and len(counted) != registered_count:
        summary += f"（条数与登记的 proof_errors={registered_count} 不同，条数只报）"
    if results.get("errors", 0) < 1 and "mutation-accepts-green" not in BREAK:
        return [(f"第 {row['number']} 行「{row['name']}」改坏之后 Verus 照样验过（verified {results.get('verified')}、errors 0）：证明没盯住这一处",
                 "补证明或收紧 spec，让这一处改坏就验不过；确是等价变异就从表里删掉并写明为什么")], summary
    registered = parse_landing(row["landing_text"])
    listed = "；".join(f"{path}:{line} {message}" for message, path, line in diagnostic_blocks(outcome["diagnostics"])[:6])
    if "landing-by-count" in BREAK:   # 弄坏开关：改回按条数判，不比落点
        if registered_count is not None and len(counted) != registered_count and "mutation-accepts-green" not in BREAK:
            return [(f"第 {row['number']} 行「{row['name']}」红了，但证明错 {len(counted)} 条，表里写的是 proof_errors={registered_count}", "按条数判（弄坏开关）")], summary
        return [], summary
    if registered is not None and set(landing) != registered and "landing-unchecked" not in BREAK and "mutation-accepts-green" not in BREAK:
        return [(f"第 {row['number']} 行「{row['name']}」红了，但证明错落在 {'、'.join(landing) or '无'}，表里写的落点是 {'、'.join(sorted(registered))}"
                 f"（--rlimit {registry.rlimit}；verified {results.get('verified')}、errors {results.get('errors')}；证明错 {len(counted)} 条，报的是：{listed}）",
                 "先看红的是不是这条变异要盯的那一处：落到别的函数里，多半是证明或 spec 改过、红已经不是这条变异的本意（改证明或换改法）；"
                 "确是本意（函数改了名、证明挪了位置），照这一次的落点改第六段 landing=。条数跳了不用管，只报不判")], summary
    return [], summary


def prepare_must_red_row(root, registry, row):
    """必红世界一行起 Verus 之前的几样：文件在登记的 crate 里、第五段是 proof_errors=<正整数>、第六段是 landing=<落点>、替换文里没有 rlimit 属性、
    原文在文件里恰好命中一次。交 (问题或 None, 改坏之前的文本, {文件: 改坏之后的内容})；问题是 (卡在哪, 下一步)，有问题的这一行不起 Verus。
    row["proof_errors"] 在这里读出（第五段不是规范形态时 None）。
    弄坏开关：expectation-form-unchecked、landing-form-unchecked 不核第五段、第六段的形态；mutation-rlimit-attribute-accepted、anchor-unchecked 见文件头。"""
    target = os.path.join(root, row["file"])
    if not row["file"].startswith(registry.crate_directory + "/") or not os.path.isfile(target):
        return (f"第 {row['number']} 行「{row['name']}」的文件 {row['file']} 不在登记的 crate 里或不存在", "变异只许改被验的那个 crate 里的文件"), None, None
    form = EXPECTED_PROOF_ERRORS.fullmatch(row["expected"])
    row["proof_errors"] = int(form.group(1)) if form else None
    if form is None and "expectation-form-unchecked" not in BREAK:
        return (f"第 {row['number']} 行「{row['name']}」的第五段「{row['expected']}」不是 proof_errors=<证明错条数>（没起 Verus）", EXPECTATION_HOWTO), None, None
    if parse_landing(row["landing_text"]) is None and "landing-form-unchecked" not in BREAK:
        return (f"第 {row['number']} 行「{row['name']}」的第六段「{row['landing_text']}」不是 landing=<落点>[,<落点>…]（没起 Verus）", LANDING_HOWTO), None, None
    attributes = [match.group(0) for match in (RLIMIT_ATTRIBUTE.search(strip_line_comment(line)) for line in row["replacement"].split("\n")) if match]
    if attributes and "mutation-rlimit-attribute-accepted" not in BREAK:
        return (f"第 {row['number']} 行「{row['name']}」的替换文里有 {attributes[0]}：函数上的 rlimit 属性盖过命令行的 --rlimit {registry.rlimit}，改坏的副本不在登记的额度上验",
                "删掉替换文里的 rlimit 属性；这一行要更多额度才红得出来，就换一处改法（必红世界要在登记的 rlimit 上红）"), None, None
    text = open(target, encoding="utf-8").read()
    hits = text.count(row["original"])
    if hits != 1 and "anchor-unchecked" not in BREAK:
        return (f"第 {row['number']} 行「{row['name']}」的原文在 {row['file']} 里命中 {hits} 次，要恰好命中一次（锚点腐化）",
                "照今天的代码改这一行的原文，让它只命中要改坏的那一处"), None, None
    return None, text, {row["file"]: text.replace(row["original"], row["replacement"], 1)}


def negative_coverage_gaps(root, registry, kernels, functions, rows):
    """静态覆盖：每个带契约的函数、每个循环不变式块都要有唯一锚点落入。
    只判负例的登记完整性；变异是否真的失败仍由 must-red-worlds 验证。
    """
    gaps = []
    for kernel in kernels:
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        source = open(kernel, encoding="utf-8").read()
        spans = []
        for row in rows:
            if row["file"] != relative or source.count(row["original"]) != 1:
                continue
            start = source.index(row["original"])
            first = source[:start].count("\n") + 1
            spans.append((first, first + row["original"].count("\n")))
        def covered(first, last):
            return any(low <= last and first <= high for low, high in spans)
        lines = source.splitlines()
        for name, path, unit, _full in functions:
            if path != relative:
                continue
            text = "\n".join(lines[unit.first_line - 1:unit.last_line])
            _kinds, words, _lines, _docs = rust_lex(text)
            if any(word == "ensures" and (index == 0 or words[index - 1] != ".") for index, word in enumerate(words)) and not covered(unit.first_line, unit.last_line):
                gaps.append(f"{relative}:{unit.first_line} 函数 {name}")
        _kinds, words, token_lines, _docs = rust_lex(source)
        for index, word in enumerate(words):
            if word != "invariant":
                continue
            first = token_lines[index]
            last = next((token_lines[later] for later in range(index + 1, len(words))
                         if words[later] == "decreases"), first)
            if not covered(first, last):
                gaps.append(f"{relative}:{first} invariant 块")
    return gaps


def check_negative_coverage(root, options):
    registry = crate_registry(root, options)
    kernels = kernel_files(root, registry)
    functions = verus_functions(root, registry, kernels, options)
    rows = must_red_rows(root, registry)
    gaps = negative_coverage_gaps(root, registry, kernels, functions, rows)
    if gaps:
        raise Red("必红覆盖缺口：" + "；".join(gaps),
                  "给点名的函数或 invariant 块补唯一锚点的负例，再用真实 Verus 验证落点")
    print("  ✓ 必红覆盖：每个带契约的函数与 invariant 块都有唯一锚点负例")
    return EXIT_GREEN


def check_must_red_worlds(root, options):
    registry, kernels, manifest, executable = prepared(root, options)
    functions = verus_functions(root, registry, kernels, options)
    rows = must_red_rows(root, registry)
    chosen = selected_items(options, [must_red_item(row) for row in rows], "verus-must-red-worlds")
    rows = [row for row in rows if must_red_item(row) in chosen]
    store, tally = verdict_library(options, "must-red-world-row", "must-red-worlds"), cell_tally(options)
    problems, edits, judged, texts = [], [], [], {}
    rlimit_with_proof_error = 0
    for row in rows:
        problem, text, files = prepare_must_red_row(root, registry, row)
        if problem:
            problems.append(problem)
            continue
        edits.append((row["name"], files))
        judged.append(row)
        texts[row["number"]] = text
    # 原文、第五段、rlimit 属性没过的行没起 Verus，按项各打一行红
    passed = {row["number"] for row in judged}
    for row in rows:
        if row["number"] not in passed:
            report_item(must_red_item(row), "红", "现判（没起 Verus：这一行本身不对）")
            tally.count(False, "这一行本身不对")
    pending = []
    keyed = [must_red_components(root, registry, options, row, texts[row["number"]], functions) for row in judged]
    sentinel = plan_sentinel(options, store, "must-red-world-row", "verus-must-red-worlds",
                             [(must_red_item(row), components) for row, components in zip(judged, keyed)])
    for row, edit, components in zip(judged, edits, keyed):
        item = must_red_item(row)
        reused, reason = sentinel_lookup(store, sentinel, item, components)
        tally.count(reused, reason)
        if reused:
            report_item(item, "绿", f"复用（{reason}）")
            sentinel.observe(item, reuse_checks.REUSED, "绿")
        else:
            pending.append((row, edit, item, components, reason))
    if options.get("plan") is not None:
        return plan_done(options, "verus-must-red-worlds", store)
    judging = ItemJudging(store, lambda index, outcome: must_red_row_problems(pending[index][0], outcome, options, registry, root, pending[index][1][1]), "verus-must-red-worlds",
                          f"{'cargo verus focus' if registry.cargo_mode else 'verus'} --rlimit {registry.rlimit}（{registry.tag}），整个 crate、原文换成替换文的副本", sentinel=sentinel)
    verify_in_parallel(root, registry, manifest, executable, options, [edit for _row, edit, _item, _components, _reason in pending],
                       judged=lambda index, outcome, seconds: judging(index, outcome, seconds, pending[index][2], pending[index][3],
                                                                      judged_how(store, pending[index][2], pending[index][4], sentinel)))
    problems += judging.finish(len(pending))
    problems += sentinel.finish()
    rlimit_with_proof_error = sum(1 for summary in judging.summaries.values() if "rlimit 用尽夹证明错" in summary)
    count_drifted = [pending[index][0]["name"] for index, summary in sorted(judging.summaries.items()) if "条数只报" in summary]
    finish_tally(tally, "verus-must-red-worlds", options)
    if problems:
        report_cell(problems)
        return EXIT_RED
    named = f"（--item 点名的 {len(rows)} 行）" if options.get("items") else ""
    drift = (f"；其中 {len(count_drifted)} 行证明错条数与第五段 proof_errors= 不同，只报不判（{'、'.join(count_drifted[:5])}{'等' if len(count_drifted) > 5 else ''}）"
             if count_drifted else "")
    print(f"  ✓ 必红世界：{registry.must_red_table} 的 {len(rows)} 行{named}每行改坏之后 Verus 都报错，证明错落点与第六段 landing= 相同；原文都恰好命中一次；"
          f"没有一行只红在 rlimit 用尽上（--rlimit {registry.rlimit}；{rlimit_with_proof_error} 行输出里夹着 rlimit 用尽、另有证明错，红计入，rlimit 用尽不计入证明错条数）{drift}；"
          f"按条{tally.text()}")
    return EXIT_GREEN


# ── 指纹与工具版本 ──

def package_source_fingerprint(directory):
    entries = []
    for current, subdirectories, names in os.walk(directory):
        subdirectories.sort()
        for name in names:
            full = os.path.join(current, name)
            relative = os.path.relpath(full, directory)
            if relative in EXCLUDED_PACKAGE_FILES:
                continue
            content = ("symlink:" + os.readlink(full)).encode() if os.path.islink(full) else open(full, "rb").read()
            entries.append((relative.encode("utf-8", "surrogateescape"), hashlib.sha256(content).hexdigest()))
    entries.sort()
    text = b"".join(digest.encode() + b"  " + relative + b"\n" for relative, digest in entries)
    return hashlib.sha256(text).hexdigest(), len(entries)


def streamed_sha256(path):
    """文件内容的 sha256，按 1 MiB 一块读（工具链里的 libLLVM 约 200 MB，不整份读进内存）。"""
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def rustup_toolchain_fields(toolchain_text, for_key=False):
    """tool-versions 里 rustup 那几段（前面带空格）：
    RUSTUP_HOME=<未设|空串|原值>:<解析到的真实路径>（没设或空串时照 rustup 取 ~/.rustup）；
    rustup_toolchain_lib_sha256=<摘要>:<文件数>，工具链目录 <RUSTUP_HOME>/toolchains/<version.json 的 toolchain 第一个词> 下
      lib/ 顶层全部普通文件与 lib/rustlib/*/lib/ 下全部普通文件（逐层往下），逐个「相对工具链目录的路径 NUL 它的 sha256 换行」按路径字节序接起来再取 sha256；
      version.json 里没有工具链名写「不知道工具链名」，目录不在写「没有」；
    rustup=<PATH 里找到的那一份>:<它的真实路径>:<sha256>（launcher 照 PATH 找 rustup），找不到写「找不到」。
    for_key（按条判定的「工具链」那一样用）：不写 RUSTUP_HOME 的原值与真实路径、不写 rustup 在 PATH 里的位置，rustup 只写 rustup_sha256=<程序的 sha256>：
    同内容的工具链换个目录、PATH 里同一份 rustup 换个位置，判定不变，键也不变；工具链目录下的文件与 rustup 程序换了一个字节照样变。"""
    value = os.environ.get("RUSTUP_HOME")
    shown = "未设" if value is None else "空串" if value == "" else value
    home = os.path.realpath(os.path.abspath(value) if value else RUSTUP_DEFAULT_HOME)
    fields = "" if for_key else f" RUSTUP_HOME={shown}:{home}"
    name = toolchain_text.split(" ")[0]
    if not name:
        fields += " rustup_toolchain_lib_sha256=不知道工具链名"
    else:
        toolchain = os.path.join(home, "toolchains", name)
        if not os.path.isdir(toolchain):
            fields += " rustup_toolchain_lib_sha256=没有"
        else:
            relatives = []
            library = os.path.join(toolchain, "lib")
            if os.path.isdir(library):
                relatives += [os.path.join("lib", entry) for entry in os.listdir(library) if os.path.isfile(os.path.join(library, entry))]
            for target_library in glob.glob(os.path.join(library, "rustlib", "*", "lib")):
                for current, _subdirectories, names in os.walk(target_library):
                    relatives += [os.path.relpath(os.path.join(current, entry), toolchain) for entry in names if os.path.isfile(os.path.join(current, entry))]
            relatives.sort(key=lambda relative: relative.encode("utf-8", "surrogateescape"))
            every = hashlib.sha256()
            for relative in relatives:
                every.update(relative.encode("utf-8", "surrogateescape") + b"\0" + streamed_sha256(os.path.join(toolchain, relative)).encode() + b"\n")
            fields += f" rustup_toolchain_lib_sha256={every.hexdigest()}:{len(relatives)}"
    located = shutil.which("rustup")
    if for_key:
        fields += f" rustup_sha256={streamed_sha256(located)}" if located else " rustup=找不到"
    else:
        fields += f" rustup={located}:{os.path.realpath(located)}:{streamed_sha256(located)}" if located else " rustup=找不到"
    return fields


def tool_versions(root, override):
    try:
        print(tool_versions_text(root, override))
    except Red as red:   # 安装目录取不到（数据根没配）：判红说出路，不抛回溯
        report_red(red.message, red.howto)
        return EXIT_RED
    return EXIT_GREEN


def tool_versions_text(root, override, for_key=False):
    """tool-versions 打的那一行（不带换行）。for_key 交按条判定的「工具链」那一样（键是它的摘要）：同一行去掉不改判定的环境差异——
    RUSTUP_HOME 与 rustup 的位置（rustup_toolchain_fields 的 for_key）、VERUS_ROOT 与 VERUS_Z3_PATH 的原值与解析到的路径、没装时的安装目录，
    只留版本串与内容摘要（Verus 安装、工具链、z3、VERUS_ROOT 目录的内容换了照样变）。"""
    try:
        registry = read_registry(root)
    except Red:
        registry = Registry()
        registry.install_relative = "singlefs/formal-tools/verus"
    directory = verus_directory(registry, override)
    version = read_version_json(directory)

    def file_sha256(path):
        return streamed_sha256(path) if os.path.isfile(path) else "没有"

    def top_level_sha256(path):
        """目录顶层全部普通文件：逐个「文件名 NUL 它的 sha256 换行」按文件名字节序接起来再取 sha256；目录不在时是空串的 sha256。"""
        every = hashlib.sha256()
        if os.path.isdir(path):
            for name in sorted(os.listdir(path), key=lambda entry: entry.encode("utf-8", "surrogateescape")):
                full = os.path.join(path, name)
                if os.path.isfile(full):
                    every.update(name.encode("utf-8", "surrogateescape") + b"\0" + file_sha256(full).encode() + b"\n")
        return every.hexdigest()

    digest = file_sha256(os.path.join(directory, "verus"))
    extra = ""
    if "tool-versions-launcher-only" not in BREAK:
        extra = "".join(f" {name}_sha256={file_sha256(os.path.join(directory, name))}" for name in TOOL_VERSION_FILES)
        if "tool-versions-five-files-only" not in BREAK:
            # 安装目录顶层全部普通文件：Verus 还读五样之外的（宏库、libverus_builtin.rlib），换了任何一份这一行都变
            extra += f" install_top_level_sha256={top_level_sha256(directory)}"
        if "tool-versions-verus-environment-ignored" not in BREAK:
            # rust_verify 读的 VERUS_ROOT（宏库、vstd 从那里读）与 launcher 读的 VERUS_USE_RUSTUP（0 时不经 rustup 起 rust_verify）：没设与空串分开
            for variable in ("VERUS_ROOT", "VERUS_USE_RUSTUP"):
                value = os.environ.get(variable)
                shown = "未设" if value is None else "空串" if value == "" else "设了" if for_key and variable == "VERUS_ROOT" else value
                if variable == "VERUS_ROOT" and value and os.path.isdir(value):
                    shown += f":top_level_sha256={top_level_sha256(value)}"
                extra += f" {variable}={shown}"
        if "tool-versions-rustup-ignored" not in BREAK:
            # launcher 经 rustup 起 rust_verify：那条工具链里 rust_verify 读的库、RUSTUP_HOME、rustup 程序本身，换了任何一样这一行都变
            extra += rustup_toolchain_fields((version or {}).get("toolchain") or "", for_key)
        z3_override = os.environ.get("VERUS_Z3_PATH")
        if z3_override is None or (z3_override == "" and "tool-versions-empty-z3-path-as-unset" in BREAK):
            extra += " VERUS_Z3_PATH=未设"
        elif z3_override == "":
            extra += " VERUS_Z3_PATH=空串"   # Verus 不把空串当没设：照它去起 z3、起不来
        else:
            bare = "/" not in z3_override and "tool-versions-bare-z3-from-current-directory" not in BREAK
            located = (shutil.which(z3_override) or "") if bare else z3_override
            if for_key:
                extra += f" VERUS_Z3_PATH=设了:{file_sha256(located) if located else '没有'}"
            else:
                extra += f" VERUS_Z3_PATH={z3_override}:{located or '找不到'}:{file_sha256(located) if located else '没有'}"
    if version is None:
        return f"verus 没装（{'安装目录' if for_key else directory} 里没有 version.json） verus_sha256={digest}{extra}"
    return f"verus version={version.get('version')} commit={version.get('commit')} toolchain={(version.get('toolchain') or '').split(' ')[0]} verus_sha256={digest}{extra}"


# ── 命令行 ──

def default_threads(environment=os.environ):
    """--threads 的缺省：SINGLEFS_THREAD_CAP 设了取它，否则登记的重活默认并行度（.claude/heavy-thread-default）。"""
    if "threads-default-cpu-count" in BREAK:   # 弄坏开关：--threads 的缺省取核数
        return int(environment.get("SINGLEFS_THREAD_CAP") or os.cpu_count() or 1)
    return default_heavy_threads(environment)


def parse_options(arguments):
    options = {"verus_root": None, "cargo_home": None,
               "threads": default_threads(),
               "jobs": int(os.environ.get("GATE_VERUS_JOBS") or 0),
               "items": [], "verdict_library": None, "cargo": None, "rustc_version_file": None}
    positional = []
    flags = {"--verus-root": "verus_root", "--cargo-home": "cargo_home", "--threads": "threads",
             "--jobs": "jobs", "--rustc-version-file": "rustc_version_file"}
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--item":
            if position + 1 >= len(arguments) or not arguments[position + 1]:
                raise ValueError("--item 后面缺项名（可点名的项：verus-proof-check.py list-items <根>）")
            options["items"].append(arguments[position + 1])
            position += 2
        elif argument == "--key-mismatches":
            options["key_mismatches"] = True
            position += 1
        elif argument in flags:
            if position + 1 >= len(arguments):
                raise ValueError(f"{argument} 后面缺值")
            value = arguments[position + 1]
            options[flags[argument]] = int(value) if flags[argument] in ("threads", "jobs") else value
            position += 2
        elif argument.startswith("--"):
            raise ValueError(f"认不出的选项 {argument}")
        else:
            positional.append(argument)
            position += 1
    if options["threads"] < 1 or options["jobs"] < 0:
        raise ValueError("--threads 要正整数，--jobs 不许负")
    return positional, options


CHECKS = {"coverage": check_negative_coverage, "toolchain": check_toolchain, "proof": check_proof, "vacuity": check_vacuity, "must-red-worlds": check_must_red_worlds}


# 逐个 crate 判的几格（登记了几个 crate 时 run_per_crate 一个一个判，--item 照各 crate 列出的项分过去）；toolchain 各 crate 共用一份
PER_CRATE_CHECKS = {"coverage": None, "proof": "verus-proof", "vacuity": "verus-vacuity", "must-red-worlds": "verus-must-red-worlds"}


def run_check(name, root, options):
    options = dict(options, shared_components=None, unit_index=None, function_keys=None)
    try:
        if name == "toolchain" and options.get("items"):
            raise UsageError("verus-toolchain-pinned 格只有一项（toolchain），不收 --item", ["toolchain"])
        if name in PER_CRATE_CHECKS and options.get("crate_view") is None:
            registry = read_registry(os.path.abspath(root))
            if len(registry.crates) > 1:
                return run_per_crate(name, os.path.abspath(root), options, registry)
        return CHECKS[name](os.path.abspath(root), options)
    except UsageError as usage:
        report_red(usage.message, "照下面列的项名写（verus-proof-check.py list-items <根> 也列），给几次 --item 取并集")
        for item in usage.available:
            print(f"       {item}")  # gate-lint:detail
        return EXIT_USAGE
    except Red as red:
        report_red(red.message, red.howto)
        return EXIT_RED
    except NotRun as not_run:
        print(f"  ! {not_run}")
        return EXIT_NOT_RUN


def crate_item_lines(root, registry, options):
    """一个 crate 的可点名的项（只读登记表、核与必红世界表，不起 Verus）：交 (项的那几行 [(项, 格, 说明)], 成项的证明 [ProofItem], 不成项的 [(全路径, 核)], 核在不在)。"""
    try:
        kernels = kernel_files(root, registry)
    except NotRun:
        kernels = []
    functions = verus_functions(root, registry, kernels, options) if kernels else []
    items, left_out = proof_items(root, registry, kernels, functions, options) if kernels else ([], [])
    lines = [(entry.item, "verus-proof", f"{entry.kind} {entry.relative}:{entry.unit.first_line}") for entry in items]
    points, _all_at_once = vacuity_points(root, registry, kernels, functions, options)
    lines += [(point[0], "verus-vacuity", point[1]) for point in points]
    if kernels:
        lines += [(must_red_item(row), "verus-must-red-worlds", f"第 {row['number']} 行") for row in must_red_rows(root, registry)]
    return lines, items, left_out, bool(kernels)


def run_per_crate(name, root, options, registry):
    """登记了几个 crate 时，一格逐个 crate 判（crate 行可以多条、各有各的登记数）：--item 照各 crate 列出的项（crate_item_lines）分过去，
    点名的项一个 crate 都没有的退 2；一个 crate 一项都没点到的这一趟不判它。各 crate 的按条数合成一行报、往门禁的 FORMAL_ITEM_TALLY_FILE 追加一行。
    退出码：有用法错退 2，有红退 1，全是本次未跑退 77，否则 0。"""
    wanted = list(options.get("items") or [])
    owners = {}
    if wanted:
        for view in registry.crates:
            lines, _items, _left_out, _wired = crate_item_lines(root, view, {"unit_index": unit_index(root, options)})
            for item, cell, _note in lines:
                if PER_CRATE_CHECKS.get(name) not in (None, cell):
                    continue
                if item in owners and owners[item] != view.crate_directory:
                    raise Red(f"两个 crate（{owners[item]}、{view.crate_directory}）都有项 {item}", "必红世界表里的变异名在几张表之间也不许重名：给其中一行改名")
                owners[item] = view.crate_directory
        unknown = [item for item in wanted if item not in owners and not (name == "proof" and item == "proof")]
        if unknown and "item-filter-ignored" not in BREAK:
            raise UsageError(f"{PER_CRATE_CHECKS.get(name) or name} 格里没有点名的项：{'、'.join(unknown)}", (["proof"] if name == "proof" else []) + sorted(owners))
    # 各 crate 共用：单元索引（按根建，下面 crate_options 复制 options 时带着）与工具链摘要（按根与 Verus 安装目录，同一份字典）
    unit_index(root, options)
    toolchain_digests = {}
    exits, tallies = [], []
    for view in registry.crates:
        mine = [item for item in wanted if owners.get(item) == view.crate_directory or item == "proof"]
        if wanted and not mine and "item-filter-ignored" not in BREAK:
            print(f"  · {view.crate_directory}：--item 没点到这个 crate 的项，这一格这一趟不判它")
            continue
        print(f"  ── {view.crate_directory}{'（cargo verus focus）' if view.cargo_mode else ''}")
        crate_options = dict(options, crate_view=view, items=mine, crate_tallies=tallies, shared_components=None, function_keys=None, toolchain_digests=toolchain_digests)
        try:
            exits.append(CHECKS[name](root, crate_options))
        except UsageError:
            raise
        except Red as red:
            report_red(red.message, red.howto)
            exits.append(EXIT_RED)
        except NotRun as not_run:
            print(f"  ! {not_run}")
            exits.append(EXIT_NOT_RUN)
    if tallies and PER_CRATE_CHECKS.get(name):
        total = merged_tally(tallies)
        print(f"  · 按条：{total.text()}")  # gate-lint:detail
        total.append_to(os.environ.get(ITEM_TALLY_FILE_VARIABLE), PER_CRATE_CHECKS[name])
    if EXIT_USAGE in exits:
        return EXIT_USAGE
    if EXIT_RED in exits:
        return EXIT_RED
    if exits and all(code == EXIT_NOT_RUN for code in exits):
        return EXIT_NOT_RUN
    print(f"  ✓ {PER_CRATE_CHECKS.get(name) or name}：登记的 {len(registry.crates)} 个 crate 判了 {len(exits)} 个（{'、'.join(view.crate_directory for view in registry.crates)}），都没红")
    return EXIT_GREEN


def list_items(root, verdict_library=None, verus_root=None, items=None):
    """逐行打「项<TAB>格[<TAB>说明]」：toolchain、proof（整格；说明里报成项几个、各哪一类，不成项的 spec fn 逐个点名）各一项，证明每个成项的单元一项
    proof:<全路径>（说明是哪一类与它在核里的位置），空真每处 requires 一项 vacuity:<所属函数的全路径>#<第几处>，必红世界每行一项（表不是六段的红在这里，同必红世界那一格）。登记了几个 crate 的，逐个 crate 列（全路径的头一段是 crate 名，各 crate 的项不会重名）。
    只读登记表、核与表（核经 lib_function_units.py 切单元），不起 Verus、不编。给了 verdict_library（命令行的 --key-mismatches 取库放在哪那一份）另往 stderr 报每格键对不上的条数
    （report_key_mismatches）。"""
    root = os.path.abspath(root)
    lines = ["toolchain\tverus-toolchain-pinned", "proof\tverus-proof\t整格（有证明义务的单元各一项）"]
    if not os.path.isfile(os.path.join(root, REGISTRY)):
        # 登记表不在（只摆了红例两格的样本根）：只列两项固定的；真要判时那几格自己报登记表不在、判红
        print("\n".join(lines))
        return EXIT_GREEN
    try:
        registry = read_registry(root)
        options = {"unit_index": None}
        all_items, all_left_out, wired, seen, assert_by_sites = [], [], False, {}, []
        for view in registry.crates:
            # 这一个 crate 的成项单元叫 crate_units，不与参数 items（--item 点名的项名，后面交给 report_key_mismatches）同名
            crate_lines, crate_units, left_out, crate_wired = crate_item_lines(root, view, options)
            if crate_wired:
                assert_by_sites += assert_by_requires_sites(root, kernel_files(root, view))
            for item, _cell, _note in crate_lines:
                if item in seen:
                    raise Red(f"两个 crate（{seen[item]}、{view.crate_directory}）都有项 {item}", "必红世界表里的变异名在几张表之间也不许重名：给其中一行改名")
                seen[item] = view.crate_directory
            lines += ["\t".join(line) for line in crate_lines]
            all_items += crate_units
            all_left_out += left_out
            wired = wired or crate_wired
        if wired:
            lines[1] = (f"proof\tverus-proof\t整格（{len(registry.crates)} 个 crate 成项 {len(all_items)} 个：{item_kinds_text(all_items)}" if len(registry.crates) > 1
                        else f"proof\tverus-proof\t整格（成项 {len(all_items)} 个：{item_kinds_text(all_items)}") + \
                (f"；不成项的 spec fn {len(all_left_out)} 个：{listed_names((name for name, _relative in all_left_out), limit=len(all_left_out))}）") + \
                f"；空真{assert_by_skipped_text(assert_by_sites)}"
    except Red as red:
        report_red(f"列不出可点名的项：{red.message}", red.howto)
        return EXIT_RED
    for line in lines:
        print(line)
    if verdict_library and wired:
        report_key_mismatches(root, verdict_library, verus_root, items, options["unit_index"])
    return EXIT_GREEN


def report_key_mismatches(root, verdict_library, verus_root, items=None, index=None):
    """list-items 给了 --key-mismatches：证明、空真、必红世界三格逐项照现在的键比判定库，每格往 stderr 打一行各几条（键对不上、无记录、键相同、拒算），
    末了一行合计（底稿第二节第 9 条 ⑦：各道 --list-items 报每项键对不上的判定条数）。只读库、不写库，不起 Verus（要装着：键里的工具链那一样读它的安装）；
    项名逐行的 stdout 不变。必红世界里原文、第五段、rlimit 属性没过的行不起 Verus、不进库，不数。
    三格、各 crate 的这几次调用共用：单元索引（index，list-items 列项时建的那一份；没给另建一份，只按根建、与 crate 无关）、工具链摘要（按根与 Verus 安装目录）；
    共用组成（工具链、rlimit、crate 起法、信任清单）按 crate 共用，三格同一个 crate 的相同、不同 crate 的不同（crate 起法与信任清单各带各的）。
    弄坏开关 key-mismatches-share-across-crates：共用组成不分 crate、第一个 crate 算的交给后面的 crate。"""
    plan, plan_items, unreadable = [], [], []
    _positional, defaults = parse_options([])
    registry = read_registry(root)
    views = registry.crates if len(registry.crates) > 1 else [None]
    shared_index = index if index is not None else FunctionUnitIndex(root)
    toolchain_digests, components_of_crate = {}, {}
    # 给了 items（--item，可几个）只比点名的那几项：每格只收属于它的项（proof 与 proof:*、vacuity:*、must-red:*），这一格没有点名项的整格不比；
    #   proof 这一项（整格，不按条入库）自己没有键，单独点名它时 proof 格不比。成本随点名项数走。弄坏开关 key-items-ignored 下照旧比整表
    prefixes = {"proof": ("proof:",), "vacuity": ("vacuity:",), "must-red-worlds": ("must-red:",)}
    for cell in ("proof", "vacuity", "must-red-worlds"):
        named = [item for item in (items or []) if item.startswith(prefixes[cell])] if "key-items-ignored" not in BREAK else []
        if items and "key-items-ignored" not in BREAK and not named:
            continue
        for view in views:
            crate_slot = None if view is None or "key-mismatches-share-across-crates" in BREAK else view.crate_directory
            cell_options = dict(defaults, items=named, verdict_library=verdict_library, verus_root=verus_root, plan=plan, plan_items=plan_items,
                                shared_components=components_of_crate.get(crate_slot), unit_index=shared_index, function_keys=None,
                                toolchain_digests=toolchain_digests, crate_view=view)
            buffer = io.StringIO()
            try:
                with contextlib.redirect_stdout(buffer):
                    CHECKS[cell](root, cell_options)
            except (Red, NotRun, UsageError) as error:
                unreadable.append(f"{cell}{'（' + view.crate_directory + '）' if view is not None else ''}：{getattr(error, 'message', str(error))}")
            if cell_options["shared_components"] is not None:
                components_of_crate[crate_slot] = cell_options["shared_components"]
    totals = collections.Counter()
    merged = collections.OrderedDict()
    for cell, counts in plan:
        merged.setdefault(cell, collections.Counter()).update(counts)
    for cell, counts in merged.items():
        totals.update(counts)
        print(f"  · {cell}：" + "、".join(f"{state} {counts[state]} 条" for state in PLAN_STATES), file=sys.stderr)  # gate-lint:detail
    for line in unreadable:
        print(f"  ! 这一格的键数不出：{line}", file=sys.stderr)
    print(f"  · 键对不上共 {totals['键对不上']} 条（{len(merged)} 格比了 {sum(totals.values())} 项，判定库 {verdict_library}）", file=sys.stderr)
    # 逐项的键状态，一项一行（门禁 --list-items --with-keys 转成 ITEM_PLAN；没比出状态的项——必红世界里原文、第五段、rlimit 属性没过的行——不打）。
    # 弄坏开关 item-key-lines-skipped 下不打
    if "item-key-lines-skipped" not in BREAK:
        for item, state in plan_items:
            print(f"{ITEM_KEY_PREFIX} item={item} state={state}", file=sys.stderr)


def selftest_break_caught(returncode, output):
    """故障注入必须被具体自证格检出；崩溃、信号或用法错不算证明会红。"""
    return returncode == EXIT_RED and re.search(r"(?m)^\s*✗ .+ 这一格判错[：:]", output) is not None


def selftest_break_miss_detail(returncode, output):
    """开关轮里没检出的那一趟为什么没检出（selftest_break_caught 交 False 之后调）：被信号停了、崩了（输出里有 Python 回溯）、
    退 0（每格都判对）、退 1 却没有一格报「这一格判错」、别的退出码；后面接输出的末一个非空行（崩了的那就是回溯末行的异常）。"""
    lines = [line.strip() for line in output.splitlines() if line.strip()]
    last = lines[-1][-200:] if lines else "（没有输出）"
    if returncode < 0:
        why = f"被信号 {-returncode} 停了"
    elif "Traceback (most recent call last):" in output:
        why = f"退 {returncode}、崩了（输出里有 Python 回溯）"
    elif returncode == EXIT_GREEN:
        why = "退 0、每格都判对（开关没让任何一格红）"
    elif returncode == EXIT_RED:
        why = "退 1 却没有一格报「这一格判错」"
    else:
        why = f"退 {returncode}"
    return f"{why}；输出末行：{last}"
