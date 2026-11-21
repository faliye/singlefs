"""形式化证明判法自证（入口 verus-proof-check.py 的 --selftest）的一块：selftest。main 的 --selftest 分支里才 import。
"""

import collections
import concurrent.futures
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from heavy_threads import default_heavy_threads  # noqa: E402
from lib_verdict_library import FunctionUnitIndex, VerdictLibrary
from lib_verdict_library import key_of
import lib_verdict_library  # noqa: E402  library_root（库根）、locked_dependency_lines（闭包的依赖版本行算不出时退回整个 crate 的）
import lib_reuse_checks as reuse_checks  # noqa: E402
import lib_verus_trait_impl_lag as trait_impl_lag  # noqa: E402  放过表只缩不涨的判法与它的自证格（trait_impl_lag_growth、selftest_cells）

from verus_proof_check_base import (
     SCRIPT_DIRECTORY, REGISTRY, SCRIPT_KEY, EXIT_GREEN, EXIT_RED, EXIT_USAGE, DEFAULT_RLIMIT, OUTER_MEMORY_CAP_VARIABLE, TRUST_LIST, CFG_KIND,
     TRAIT_IMPLS_OUTSIDE_LAG, BREAK, FORMAL_BREAK, FORMAL_BREAK_ITEMS, ITEM_DONE_FILE_VARIABLE, ITEM_KEY_PREFIX, Red, report_red, one_line, Registry,
     read_registry, verus_directory, run_in_own_process_group, NOT_RUN_FILE_VARIABLE, ENTRY_SCRIPT, lag_baseline_library,
)
from verus_proof_check_listing import (
     region_spans, trusted_declarations, prepared, function_components, FORMAL_LIBRARY_DIRECTORY_NAME,
)
from verus_proof_check_proofs import (
     toolchain_digest, shared_components, requires_insertion_points, SLOT_DIRECTORY_NAME, claim_slot_directories, verify_in_parallel,
     package_source_fingerprint, tool_versions, tool_versions_text, default_threads, parse_options, run_check, list_items, selftest_break_caught,
     selftest_break_miss_detail,
)
from verus_proof_check_selftest_fixtures import (
     write_fake_tools, FAKE_VERUS, FAKE_CARGO, FAKE_CARGO_VERUS, SELFTEST_METHODS_EXTRA, SELFTEST_TRUST_EXTRA, SELFTEST_CONST_EXTRA,
     SELFTEST_IMPL_SIBLINGS_EXTRA, SELFTEST_IMPL_SIBLINGS_VACUITY_EXTRA, SELFTEST_ASSUME_TARGETS_EXTRA, SELFTEST_ASSUME_TARGETS_FORMAT,
     SELFTEST_CONST_PREFIX_EXTRA, SELFTEST_CONST_SECOND_IMPL_EXTRA, SELFTEST_CONST_TRAIT_DEFAULT_EXTRA, SELFTEST_CONST_FN_EXTRA,
     SELFTEST_ASSERT_BY_EXTRA, SELFTEST_PLAIN_BLOCK_REQUIRES_EXTRA, SELFTEST_ASSERT_WITHOUT_BY_EXTRA, SELFTEST_SPEC_CONST_EXTRA, SELFTEST_TRAIT_EXTRA,
     SELFTEST_TRAIT_OUTSIDE, SELFTEST_TRAIT_LAG_ROW, SELFTEST_TRAIT_OUTSIDE_SECOND, SELFTEST_TRAIT_INSIDE, SELFTEST_UNVERIFIED_OUTSIDE,
     SELFTEST_IMPL_EXTERNAL_BODY, SELFTEST_TRAIT_LITERAL_EXTRA, captured_stdout, counting_workspace_loads, selftest_kernel_with, SELFTEST_KERNEL,
     SELFTEST_LANDING_KERNEL, SELFTEST_SHARED_SPEC_KERNEL, SELFTEST_ITEMS_KERNEL,
)

def selftest():
    # 判定库那几格写进 prune-server：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），
    # 不是就一格都不跑，不写真服务
    client = lib_verdict_library.prune_server_client()
    service_problem = client.selftest_service_problem() if client is not None else "这棵树里没有 scripts/prune-server/prune_server_client.py"
    if service_problem:
        print(f"  ✗ verus-proof-check 自证没跑：{service_problem}")
        print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY if client is not None else '在带着 scripts/prune-server/ 的仓里跑'}")
        return 2
    os.environ.pop(NOT_RUN_FILE_VARIABLE, None)   # 同上：放过项报本次未跑那一格自己建一份，别的格不往门禁的汇总里写
    # task-server 套的内存包装给的外层上限不进自证：jobs-not-narrowed-by-outer-memory 那一格自己设（开关 outer-memory-narrows-jobs 下别的格不跟着收）
    os.environ.pop(OUTER_MEMORY_CAP_VARIABLE, None)
    # 服务或整轮门禁给的编译目录不进自证：cargo 那一种几处同时验的编译目录在 slot-directories-shared-across-calls 那一格自己设
    os.environ.pop("CARGO_TARGET_DIR", None)
    # 复用哨兵：别的格比的是现判与复用的条数，键相同的全当哨兵、不抽强制现判（哨兵数比项数多）；reuse-three-questions 与 reuse-sentinel 两格自己设
    os.environ[reuse_checks.SENTINEL_COUNT_VARIABLE] = "1000"
    cases, wrong = 0, []
    with tempfile.TemporaryDirectory(prefix="verus-proof-check-selftest-") as work:
        commit, other_commit = "a" * 40, "b" * 40
        install = os.path.join(work, "tools", "verus-x86-linux")
        os.makedirs(install)
        for tool, text in (("verus", FAKE_VERUS), ("cargo", FAKE_CARGO), ("cargo-verus", FAKE_CARGO_VERUS)):
            with open(os.path.join(install, tool), "w") as handle:
                handle.write(text)
            os.chmod(os.path.join(install, tool), 0o755)
        rustc_version_file = os.path.join(work, "rustc-version.txt")
        open(rustc_version_file, "w").write(f"rustc 1.0.0 (dddddddd 2026-01-01)\nbinary: rustc\ncommit-hash: {'d' * 40}\nrelease: 1.0.0\n")
        zip_path = os.path.join(work, "tools", "verus.zip")
        open(zip_path, "wb").write(b"zip")
        zip_digest = hashlib.sha256(b"zip").hexdigest()
        cargo_home = os.path.join(work, "cargo-home")
        package_directory = os.path.join(cargo_home, "registry", "src", "index-x", "vstd-0.0.1")
        os.makedirs(os.path.join(package_directory, "src"))
        open(os.path.join(package_directory, "src", "lib.rs"), "w").write("// vstd\n")
        open(os.path.join(package_directory, "Cargo.toml"), "w").write("[package]\n")
        open(os.path.join(package_directory, ".cargo-ok"), "w").write('{"v":1}')
        json.dump({"git": {"sha1": "c" * 40}, "path_in_vcs": "source/vstd"}, open(os.path.join(package_directory, ".cargo_vcs_info.json"), "w"))
        fingerprint, _count = package_source_fingerprint(package_directory)

        def make_root(name, kernel=SELFTEST_KERNEL, count=2, must_red=None, version_commit=commit, lock_version="0.0.1", with_kernel=True, wired=True, extra_dependency="",
                      rlimit=7, trust_rows=(), trust_list=True, rustc_row=f"rustc\tstable\t1.0.0\t{'d' * 40}\n", crate_mode=""):
            root = os.path.join(work, name)
            os.makedirs(os.path.join(root, ".claude", "gate.d"))
            os.makedirs(os.path.join(root, "crates", "core", "src"))
            os.makedirs(os.path.join(root, "crates", "core", "tests"))
            json.dump({"verus": {"version": "0.9.9.tag", "commit": version_commit, "toolchain": "stable"}}, open(os.path.join(install, "version.json"), "w"))
            open(os.path.join(root, REGISTRY), "w").write(
                "# 自证用\n"
                f"release\t0.9.9.tag\t{commit}\t{zip_digest}\tunused\n"
                f"package\tvstd\t0.0.1\t{'c' * 40}\tsource/vstd\t{fingerprint}\n"
                "crate\tcrates/core\t%d%s\n" % (count, crate_mode) +
                "kernel\tcrates/core/src/kernel.rs\n"
                "must-red-worlds\tcrates/core/verus-must-red-worlds.tsv\n" + (f"rlimit\t{rlimit}\n" if rlimit is not None else "") + rustc_row)
            if trust_list:
                open(os.path.join(root, TRUST_LIST), "w").write(f"# 自证用\nvstd\t0.0.1\t{fingerprint}\t自证用：vstd 整体登记\n" + "".join(row + "\n" for row in trust_rows))
            open(os.path.join(root, "crates", "core", "tests", "differential.rs"), "w").write(
                "#[test]\nfn selftest_differential() {}\n\n#[test]\n#[ignore]\nfn selftest_differential_other() {}\n")
            open(os.path.join(root, "Cargo.lock"), "w").write(f'version = 4\n\n[[package]]\nname = "vstd"\nversion = "{lock_version}"\n')
            open(os.path.join(root, "crates", "core", "Cargo.toml"), "w").write(
                '[package]\nname = "core-sample"\nedition = "2021"\n\n[dependencies]\n' + ('vstd = "=0.0.1"\n' if wired else "") + extra_dependency)
            open(os.path.join(root, "crates", "core", "src", "lib.rs"), "w").write("pub mod kernel;\n" if with_kernel else "")
            if with_kernel:
                open(os.path.join(root, "crates", "core", "src", "kernel.rs"), "w").write(kernel)
            rows = must_red if must_red is not None else ["收紧改松\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // SELFTEST_BROKEN\tproof_errors=1\tlanding=clamp_to_limit"]
            open(os.path.join(root, "crates", "core", "verus-must-red-worlds.tsv"), "w").write("# 自证用\n" + "".join(row + "\n" for row in rows))
            return root

        options = {"verus_root": install, "cargo_home": cargo_home, "threads": 2, "jobs": 2,
                   "rustc_version_file": rustc_version_file}
        fresh_libraries = []

        def fresh_library():
            """一份新的空库（库目录在这一趟的临时目录底下，只当名字：临时服务底下另占一个命名空间）：只看判法不看复用的那几格各用一份，不串。"""
            fresh_libraries.append(os.path.join(work, f"fresh-library-{len(fresh_libraries)}"))
            return fresh_libraries[-1]
        # MIR 比对的产品构建照 PATH 找 cargo：假 cargo 放在最前面（只在这一趟自证的进程里）
        os.environ["PATH"] = install + os.pathsep + os.environ.get("PATH", "")

        def expect(label, check, root, want_exit, want_text):
            nonlocal cases
            cases += 1
            import io
            import contextlib
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                got = run_check(check, root, dict(options, verdict_library=fresh_library()))
            output = buffer.getvalue()
            want_texts = want_text if isinstance(want_text, tuple) else (want_text,)
            if got != want_exit or any(text not in output for text in want_texts):
                wrong.append(label)
                print(one_line(f"  ✗ {label} 这一格判错：期望退 {want_exit}、输出含「{'」「'.join(want_texts)}」，实测退 {got}：{output.strip()[-300:]}"))  # gate-lint:detail
            return output

        def expect_absent(label, output, unwanted):
            """一格已经跑出来的输出里不许有 unwanted（输出是 expect 交回的那一份，不再起一遍 Verus）。"""
            nonlocal cases
            cases += 1
            if unwanted in output:
                wrong.append(label)
                print(one_line(f"  ✗ {label} 这一格判错：期望输出里没有「{unwanted}」，实测有：{output.strip()[-300:]}"))  # gate-lint:detail

        omitted_root = make_root("unregistered-proof-module")
        open(os.path.join(omitted_root, "crates/core/src/omitted.rs"), "w").write(
            "use vstd::prelude::*;\nverus! { pub proof fn bad() ensures false, {} }\n")
        open(os.path.join(omitted_root, "crates/core/src/lib.rs"), "a").write("pub mod omitted;\n")
        expect("unregistered-proof-module", "proof", omitted_root, 1, "未登记的 Verus 核")
        inline_requires = SELFTEST_KERNEL.replace("    requires\n        limit > 0,", "    requires limit > 0,")
        inline_requires = inline_requires.replace("-> (result: u64)\n    requires limit > 0,", "-> (result: u64) requires limit > 0,")
        expect("inline-requires-refused", "vacuity", make_root("inline-requires", kernel=inline_requires), 1, "拒绝漏检")
        coverage_root = make_root("negative-coverage")
        expect("negative-coverage-missing", "coverage", coverage_root, 1, "函数 keep_small")
        with open(os.path.join(coverage_root, "crates/core/verus-must-red-worlds.tsv"), "a") as handle:
            handle.write("keep-small-contract\tcrates/core/src/kernel.rs\tresult <= 10,\tresult < 10,\tproof_errors=1\tlanding=keep_small\n")
        expect("negative-coverage-complete", "coverage", coverage_root, 0, "每个带契约的函数")
        green = make_root("green")
        expect("toolchain-green", "toolchain", green, 0, "个包的版本、出处与源码指纹都与登记相同")
        expect("proof-green", "proof", green, 0, "成项 2 个（exec fn 2），各项 verified 之和 2、0 errors（登记 2）；不成项的 spec fn 0 个")
        expect("vacuity-green", "vacuity", green, 0, "2 处 requires")
        expect("must-red-green", "must-red-worlds", green, 0, "1 行每行改坏之后")
        expect("toolchain-commit", "toolchain", make_root("commit", version_commit=other_commit), 1, "Verus 安装的 commit 是")
        expect("toolchain-lock", "toolchain", make_root("lock", lock_version="0.0.2"), 1, "Cargo.lock 里 vstd 的版本")
        json.dump({"verus": {"version": "0.9.9.tag", "commit": commit, "toolchain": "stable"}}, open(os.path.join(install, "version.json"), "w"))
        tampered = make_root("tampered")
        open(os.path.join(package_directory, "src", "lib.rs"), "a").write("// 改过\n")
        expect("toolchain-fingerprint", "toolchain", tampered, 1, "源码内容指纹是")
        open(os.path.join(package_directory, "src", "lib.rs"), "w").write("// vstd\n")
        expect("kernel-not-wired", "proof", make_root("absent", with_kernel=False, wired=False), 77, "还没进仓")
        expect("kernel-deleted", "proof", make_root("deleted", with_kernel=False, wired=True), 1, "而 crate 已经接了 Verus")
        expect("proof-broken", "proof", make_root("broken", kernel=SELFTEST_KERNEL.replace("result <= limit,", "result <= limit, // SELFTEST_BROKEN")), 1, "Verus 没验过")
        # 计数之和不等判红：两个函数各验出 1 条，登记的是 3（开关 count-unchecked 下判错）
        expect("proof-count", "proof", make_root("count", count=3), 1, "各项 verified 之和 2 与登记的 3 不同")
        expect("proof-cheating", "proof", make_root("cheating", kernel=SELFTEST_KERNEL.replace("if value > limit { limit }", "assume(false);\n    if value > limit { limit }")), 1, "不许的标记")
        # Verus 被信号停了：输出不算判定、出路是查谁停了它，不写修证明（开关 killed-as-proof-red 下这一格与 proof-killed-no-proof-howto 判错）
        killed = expect("proof-killed-direct", "proof", make_root("killeddirect", kernel=SELFTEST_KERNEL + "// SELFTEST_KILLED\n"), 1,
                        ("Verus 被信号停了（退 -9，信号 9）", "查是谁在那一刻停了它"))
        cases += 1
        if "照 Verus 报错的位置修" in killed or "看上面 Verus 的报错" in killed:
            wrong.append("proof-killed-no-proof-howto")
            print(one_line(f"  ✗ proof-killed-no-proof-howto 这一格判错：被信号停了的那一条不该给修证明的出路：{killed.strip()[-300:]}"))  # gate-lint:detail
        vacuous = SELFTEST_KERNEL.replace("pub fn clamp_to_limit(", "pub fn clamp_to_limit( /* SELFTEST_VACUOUS */")
        expect("vacuity-vacuous", "vacuity", make_root("vacuous", kernel=vacuous), 1, "空真")
        expect("vacuity-closure-found", "vacuity", green, 0, "2 处 requires")
        closure_vacuous = SELFTEST_KERNEL.replace("pub fn keep_small(value: u64) -> (result: u64)\n",
                                                  "pub fn keep_small(value: u64) -> (result: u64)\n    requires\n        value < 100,\n").replace(
            "            candidate <= 10,", "            candidate <= 10, // SELFTEST_CLOSURE_VACUOUS")
        expect("vacuity-closure-vacuous", "vacuity", make_root("closurevacuous", kernel=closure_vacuous), 1, "keep_small 里第 20 行起的闭包：函数体头插 assert(false) 照样验得过")
        expect("crate-extra-dependency", "proof", make_root("extradependency", extra_dependency='singlefs-format = { path = "../format" }\n'), 1, "除 vstd 之外还依赖 singlefs-format")

        # ── 被验区域与多 crate（文件头「被验区域与多 crate 的弄坏开关」）──
        def expect_true(label, passed, detail):
            nonlocal cases
            cases += 1
            if not passed:
                wrong.append(label)
                print(one_line(f"  ✗ {label} 这一格判错：{detail}"))  # gate-lint:detail

        # 按全路径逐项：不同模块里同名的各是各的项；impl 与 trait 里的方法成项（<类型>::<方法>），trait 里没有函数体的声明不成项（开关 methods-refused 下判错）
        two_modules = make_root("same-name-in-two-modules", count=4)
        with open(os.path.join(two_modules, REGISTRY), "a") as handle:
            handle.write("kernel\tcrates/core/src/other.rs\n")
        open(os.path.join(two_modules, "crates/core/src/other.rs"), "w").write(SELFTEST_KERNEL)
        open(os.path.join(two_modules, "crates/core/src/lib.rs"), "a").write("pub mod other;\n")
        expect("same-name-in-two-modules", "proof", two_modules, 0,
               ("proof:core_sample::kernel::clamp_to_limit：绿", "proof:core_sample::other::clamp_to_limit：绿", "成项 4 个（exec fn 4）"))
        expect("proof-method-items", "proof", make_root("methoditems", kernel=selftest_kernel_with(SELFTEST_METHODS_EXTRA), count=4), 0,
               ("proof:core_sample::kernel::Region::contains：绿", "proof:core_sample::kernel::Region::read_slot：绿", "成项 4 个（exec fn 4）"))
        same_point = selftest_kernel_with(SELFTEST_METHODS_EXTRA + "\nimpl Region {\n    pub fn read_slot(&self, slot: u64) -> (r: u64)\n    {\n        4\n    }\n}\n")
        expect("proof-same-point-name", "proof", make_root("samepoint", kernel=same_point, count=5), 1, "交给 --verify-function 的名字都是 Region::read_slot")
        # 属性写法（#[verus_verify]）标的文件也是核：没登记判红（开关 attribute-mode-unrecognized 下判错）
        attribute_root = make_root("unregistered-attribute-kernel")
        open(os.path.join(attribute_root, "crates/core/src/attributed.rs"), "w").write("#[verus_verify]\npub fn attributed(x: u64) -> u64 { x }\n")
        open(os.path.join(attribute_root, "crates/core/src/lib.rs"), "a").write("pub mod attributed;\n")
        expect("unregistered-attribute-kernel", "proof", attribute_root, 1, "未登记的 Verus 核：crates/core/src/attributed.rs（属性写法")
        # 信任清单：没登记判红，红句给现算的 sha256；登记齐了判绿；sha256 不对、差分测试不在、登记了却没有那一处、vstd 行对不上、表不在，各判红
        trust_kernel = selftest_kernel_with(SELFTEST_TRUST_EXTRA)
        unregistered = expect("trust-unregistered", "proof", make_root("trustunregistered", kernel=trust_kernel), 1,
                              ("assume_specification <[T]>::binary_search 没登记进信任清单", "external_body core_sample::kernel::read_device 没登记进信任清单",
                               "uninterp core_sample::kernel::crc 没登记进信任清单"))
        digests = dict(re.findall(r"的 (?:assume_specification|external_body|uninterp) (\S+) 没登记进信任清单 \S+（这一处的规格原文 sha256 ([0-9a-f]{64})）", unregistered))
        expect_true("trust-digests-reported", len(digests) == 3, f"没登记的三处期望各报一个现算的 sha256，读到 {digests}")

        def trust_rows(test="selftest_differential", wrong_digest=None, extra=()):
            rows = []
            for kind, point in (("assume_specification", "<[T]>::binary_search"), ("external_body", "core_sample::kernel::read_device"), ("uninterp", "core_sample::kernel::crc")):
                digest = digests.get(point, "0" * 64)
                rows.append(f"{kind}\tcrates/core/src/kernel.rs\t{point}\t{wrong_digest if wrong_digest and kind == 'external_body' else digest}\t{test}\t自证用：登记的可信声明")
            return rows + list(extra)

        trust_green = expect("trust-registered-green", "proof", make_root("trustregistered", kernel=trust_kernel, trust_rows=trust_rows()), 0,
                             ("成项 3 个（exec fn 2、assume_specification 1）", "可信声明与 cfg 原子 3 处都登记在信任清单里"))
        expect("trust-digest", "proof", make_root("trustdigest", kernel=trust_kernel, trust_rows=trust_rows(wrong_digest="e" * 64)), 1,
               "external_body core_sample::kernel::read_device 登记的规格原文 sha256 是 eeeeeeeeeeee…")
        expect("trust-test-missing", "proof", make_root("trusttest", kernel=trust_kernel, trust_rows=trust_rows(test="no_such_test")), 1,
               "点名的差分测试 no_such_test 在 crates/ 下找不到标了 #[test] 的函数")
        stale = "external_body\tcrates/core/src/kernel.rs\tcore_sample::kernel::gone\t" + "f" * 64 + "\tselftest_differential\t自证用：删了的那一处"
        expect("trust-stale-row", "proof", make_root("truststale", kernel=trust_kernel, trust_rows=trust_rows(extra=[stale])), 1,
               "登记了 external_body core_sample::kernel::gone（crates/core/src/kernel.rs），核的被验区域里没有这一处")
        vstd_root = make_root("trustvstd")
        edit_text = open(os.path.join(vstd_root, TRUST_LIST)).read().replace("vstd\t0.0.1\t", "vstd\t0.0.2\t")
        open(os.path.join(vstd_root, TRUST_LIST), "w").write(edit_text)
        expect("trust-vstd", "proof", vstd_root, 1, "的 vstd 0.0.2")
        expect("trust-list-missing", "proof", make_root("trustmissing", trust_list=False), 1, "信任清单 .claude/gate.d/verus-trust-list.tsv 不在")
        # 不带 --no-cheating：登记了的 assume_specification 与 external_body 照验（开关 no-cheating-kept 下假 Verus 照真 Verus 拒，trust-registered-green 判错）
        expect_true("no-cheating-dropped", "--no-cheating" not in trust_green and "成项 3 个" in trust_green, f"登记齐了期望判绿：{one_line(trust_green.strip()[-300:])}")
        # axiom 一律判红（开关 axiom-accepted 下判错）
        axiom_kernel = selftest_kernel_with("\npub axiom fn lemma_false()\n    ensures\n        false;\n")
        expect("axiom-red", "proof", make_root("axiom", kernel=axiom_kernel, count=3), 1, "核里有 axiom：crates/core/src/kernel.rs:")
        # cfg 原子：cfg!(…)、#[cfg(…)]、宏定义里有 cfg 的宏在被验区域里的调用，没登记判红；登记了判绿（开关 cfg-atom-accepted、cfg-macro-unchecked 下判错）
        body = "if value > limit { limit } else { value }"
        cfg_kernel = SELFTEST_KERNEL.replace(body, "if cfg!(debug_assertions) { " + body + " } else { " + body + " }")
        cfg_output = expect("cfg-atom-red", "proof", make_root("cfgatom", kernel=cfg_kernel), 1, "cfg core_sample::kernel::clamp_to_limit#cfg!(debug_assertions) 没登记进信任清单")
        cfg_digest = re.search(r"#cfg!\(debug_assertions\) 没登记进信任清单 \S+（这一处的规格原文 sha256 ([0-9a-f]{64})）", cfg_output)
        cfg_row = f"cfg\tcrates/core/src/kernel.rs\tcore_sample::kernel::clamp_to_limit#cfg!(debug_assertions)\t{cfg_digest.group(1) if cfg_digest else '0' * 64}\tselftest_differential\t自证用：登记的 cfg 原子"
        expect("cfg-atom-registered", "proof", make_root("cfgregistered", kernel=cfg_kernel, trust_rows=[cfg_row]), 0, "其中 cfg 原子 1 处")
        macro_kernel = SELFTEST_KERNEL.replace("verus! {\n", "macro_rules! plus_span {\n    ($e:expr) => { if cfg!(verus_keep_ghost) { $e } else { $e } };\n}\nverus! {\n", 1).replace(body, f"plus_span!({body})")
        expect("cfg-macro-red", "proof", make_root("cfgmacro", kernel=macro_kernel), 1, "cfg core_sample::kernel::clamp_to_limit#plus_span! 没登记进信任清单")
        fork_kernel = SELFTEST_KERNEL.replace("pub fn keep_small(", "#[cfg(feature = \"wide\")]\npub fn keep_small(", 1) + \
            "verus! {\n#[cfg(not(feature = \"wide\"))]\npub fn keep_small(value: u64) -> (result: u64)\n    ensures\n        result <= 10,\n{\n    value % 10\n}\n} // verus!\n"
        expect("cfg-feature-fork", "proof", make_root("cfgfork", kernel=fork_kernel), 1,
               ("keep_small#cfg(feature=\"wide\") 没登记进信任清单", "交给 --verify-function 的名字都是 keep_small"))
        # 被验的 trait 在 verus! 外面有 impl 判红，cfg(test) 下的不判（开关 trait-impl-outside-accepted 下判错）
        trait_output = expect("trait-impl-outside", "proof", make_root("traitoutside", kernel=selftest_kernel_with(SELFTEST_TRAIT_EXTRA, SELFTEST_TRAIT_OUTSIDE)), 1,
                              "verus! 外面有 trait impl：crates/core/src/kernel.rs:")
        expect_absent("trait-impl-test-module-not-judged", trait_output, "Mock")

        # 登记式放过（放过表 .claude/gate.d/verus-trait-impls-outside-lag.tsv）：登记了的 verus! 外面的 impl 放过，成功句逐个点名「契约没证、靠信任」、这一格不算绿到底
        # （开关 trait-impl-lag-passed-unnamed 下判错）
        def lag_root(name, kernel, rows, count=2):
            root = make_root(name, kernel=kernel, count=count)
            open(os.path.join(root, TRAIT_IMPLS_OUTSIDE_LAG), "w").write("# 自证用\n" + "".join(row + "\n" for row in rows))
            return root

        # 另登记一行 trait 与类型都不在被验区域里的 impl（那一簇的补丁还没打上的样子）：不红、没放过什么，成功句点名
        lag_kernel = selftest_kernel_with(SELFTEST_TRAIT_EXTRA, SELFTEST_TRAIT_OUTSIDE)
        plain_row = "core_sample::kernel::Plain\tOther\tcrates/core/src/kernel.rs\t会话丢弃与实例表清扫\t自证用：它的 trait 还不在被验区域里"
        expect("trait-impl-lag-registered-green", "proof", lag_root("traitlag", lag_kernel + SELFTEST_UNVERIFIED_OUTSIDE, [SELFTEST_TRAIT_LAG_ROW, plain_row]), 0,
               (f"都登记在 {TRAIT_IMPLS_OUTSIDE_LAG} 里放过、这一格不算绿到底", "core_sample::kernel::Device for BadDevice（crates/core/src/kernel.rs:",
                "契约没证、靠信任，归「会话丢弃与实例表清扫」挪进 verus!",
                "放过表里 1 行的 trait 与类型此刻都不在被验区域里、没放过什么：第 3 行 core_sample::kernel::Plain for Other",
                "放过表：被判的根不在 git 仓里，没与 gate 基准比"))
        # 放过的每一处另往 $GATE_NOT_RUN_FILE 报一行本次未跑（只写在成功句里不算报了；开关 trait-impl-lag-not-run-unreported 下判错）；变量不在时不写
        not_run_file = os.path.join(work, "trait-impl-lag-not-run.txt")
        open(not_run_file, "w").close()
        lag_not_run_root = lag_root("traitlagnotrun", lag_kernel + SELFTEST_UNVERIFIED_OUTSIDE, [SELFTEST_TRAIT_LAG_ROW, plain_row])
        os.environ[NOT_RUN_FILE_VARIABLE] = not_run_file
        try:
            not_run_exit, not_run_output = captured_stdout(run_check, "proof", lag_not_run_root, dict(options, verdict_library=fresh_library()))
        finally:
            os.environ.pop(NOT_RUN_FILE_VARIABLE, None)
        not_run_lines = [line for line in open(not_run_file, encoding="utf-8").read().split("\n") if line]
        expect_true("trait-impl-lag-not-run-reported",
                    not_run_exit == 0 and len(not_run_lines) == 1 and "verus-proof 格（crates/core）：verus! 外面的 core_sample::kernel::Device for BadDevice（crates/core/src/kernel.rs:" in not_run_lines[0]
                    and "契约本次未验" in not_run_lines[0] and "放过的 1 处逐个往 $GATE_NOT_RUN_FILE 报了本次未跑" in not_run_output.getvalue(),
                    f"期望退 0、$GATE_NOT_RUN_FILE 里恰好一行点名 BadDevice、成功句写报了，实测退 {not_run_exit}、文件里 {not_run_lines}：{one_line(not_run_output.getvalue().strip()[-300:])}")
        # 同一个 trait 在外面另有一个没登记的 impl：照旧判红（开关 trait-impl-lag-matched-by-trait-only 下它也被那一行放过、判错）
        expect("trait-impl-lag-unregistered-red", "proof", lag_root("traitlagextra", lag_kernel + SELFTEST_TRAIT_OUTSIDE_SECOND, [SELFTEST_TRAIT_LAG_ROW]), 1,
               ("verus! 外面有 trait impl：crates/core/src/kernel.rs:", "<impl Device for WorseDevice>"))
        # 登记了的已挪进 verus!、登记了的在那份文件里找不到：都判红（开关 trait-impl-lag-stale-unchecked 下判错）
        gone_row = "core_sample::kernel::Device\tGoneDevice\tcrates/core/src/kernel.rs\t会话丢弃与实例表清扫\t自证用：这个 impl 已经删了"
        expect("trait-impl-lag-stale-rows-red", "proof",
               lag_root("traitlagstale", selftest_kernel_with(SELFTEST_TRAIT_EXTRA + SELFTEST_TRAIT_INSIDE), [SELFTEST_TRAIT_LAG_ROW, gone_row], count=3), 1,
               ("第 2 行登记放过的 core_sample::kernel::Device for BadDevice（crates/core/src/kernel.rs）已经挪进 verus! 里了",
                "第 3 行登记放过的 core_sample::kernel::Device for GoneDevice（crates/core/src/kernel.rs）在那份文件里找不到"))
        # 只缩不涨与拆分族那几格（lib_verus_trait_impl_lag.py 的 selftest_cells）
        trait_impl_lag.selftest_cells(expect, make_root, lag_root, selftest_kernel_with, TRAIT_IMPLS_OUTSIDE_LAG, lag_kernel,
                                      SELFTEST_TRAIT_EXTRA, SELFTEST_TRAIT_OUTSIDE, SELFTEST_TRAIT_OUTSIDE_SECOND, SELFTEST_TRAIT_LAG_ROW,
                                      expect_true, lag_baseline_library(), BREAK)

        # external_body 的规格原文是签名到函数体的 { 之前：impl 里唯一（也是最后）的 fn 取得到签名、不是空串，改签名 sha256 就变、只改函数体不变；
        # 后面还接着别的条目的 fn 只改函数体也不变（开关 body-open-past-unit-end 下判错：impl 里最后一个 fn 取成空串，别的取成整个 fn）
        def external_body_digests(name, kernel):
            root = make_root(name, kernel=kernel)
            relative = "crates/core/src/kernel.rs"
            rust_file = FunctionUnitIndex(root).workspace().file(relative)
            # 按条目名的最后一段认（Shown::show 记 show）：点名的全路径怎么拼不归这一格
            return {entry["point"].split("::")[-1]: entry["digest"] for entry in trusted_declarations(rust_file, relative, "core_sample", "kernel", region_spans(rust_file), set())
                    if entry["kind"] == "external_body"}

        plain_digests = external_body_digests("externalbodyplain", selftest_kernel_with(SELFTEST_TRUST_EXTRA + SELFTEST_IMPL_EXTERNAL_BODY))
        signature_digests = external_body_digests("externalbodysignature", selftest_kernel_with(
            SELFTEST_TRUST_EXTRA.replace("        r == x,", "        r >= x,") + SELFTEST_IMPL_EXTERNAL_BODY.replace("shown == x,", "shown >= x,")))
        body_digests = external_body_digests("externalbodybody", selftest_kernel_with(
            SELFTEST_TRUST_EXTRA.replace("{\n    x\n}", "{\n    x + 0\n}") + SELFTEST_IMPL_EXTERNAL_BODY.replace("        x\n", "        x + 0\n")))
        last_point, other_point, empty_digest = "show", "read_device", hashlib.sha256(b"").hexdigest()
        expect_true("trust-digest-signature-only",
                    all(point in digests for point in (last_point, other_point) for digests in (plain_digests, signature_digests, body_digests))
                    and plain_digests[last_point] != empty_digest
                    and all(plain_digests[point] != signature_digests[point] and plain_digests[point] == body_digests[point] for point in (last_point, other_point)),
                    f"期望 impl 里最后一个 fn 的 sha256 不是空串的、改签名就变、只改函数体不变，实测 原样 {plain_digests}、改签名 {signature_digests}、改函数体 {body_digests}"
                    f"（空串的 sha256 是 {empty_digest[:12]}…）")

        # uninterp、external_type_specification 与带 external_body 的类型规格取整个条目，照半开区间取：后面接着的条目换了头一个词（pub fn 换成 fn），
        # 三处的 sha256 都不变；改条目自己就变（开关 whole-entry-past-unit-end 下判错：连下一个条目的头一个词一起取）
        def whole_entry_digests(name, extra):
            root = make_root(name, kernel=selftest_kernel_with(extra))
            relative = "crates/core/src/kernel.rs"
            rust_file = FunctionUnitIndex(root).workspace().file(relative)
            return {(entry["kind"], entry["point"].split("::")[-1]): entry["digest"]
                    for entry in trusted_declarations(rust_file, relative, "core_sample", "kernel", region_spans(rust_file), set())
                    if entry["kind"] in ("uninterp", "external_type_specification") or entry["point"].endswith("OfDuration")}

        type_specification = ("\n#[verifier::external_type_specification]\n#[verifier::external_body]\n"
                              "pub struct ExternalTypeSpecificationOfDuration(std::time::Duration);\n")
        following = "\npub fn after_the_entry(x: u64) -> (r: u64)\n    ensures\n        r == x,\n{\n    x\n}\n"
        between = "pub fn between(x: u64) -> (r: u64)\n    ensures\n        r == x,\n{\n    x\n}\n"
        entries_then_public = whole_entry_digests("wholeentrypublic", type_specification + between + SELFTEST_TRUST_EXTRA + following)
        entries_then_private = whole_entry_digests("wholeentryprivate", type_specification + between.replace("pub fn", "fn", 1) + SELFTEST_TRUST_EXTRA
                                                   + following.replace("\npub fn", "\nfn", 1))
        entries_changed = whole_entry_digests("wholeentrychanged", type_specification.replace("Duration);", "Duration, );") + between
                                              + SELFTEST_TRUST_EXTRA.replace("crc(x: u64) -> u64;", "crc(x: u64) -> u32;") + following)
        whole_keys = [("uninterp", "crc"), ("external_type_specification", "ExternalTypeSpecificationOfDuration"), ("external_body", "ExternalTypeSpecificationOfDuration")]
        expect_true("trust-digest-whole-entry-half-open",
                    all(key in digests for key in whole_keys for digests in (entries_then_public, entries_then_private, entries_changed))
                    and all(entries_then_public[key] == entries_then_private[key] and entries_then_public[key] != entries_changed[key] for key in whole_keys),
                    f"期望三处整个条目的 sha256 不随下一个条目的头一个词变、改条目自己就变，实测 后接 pub {entries_then_public}、后接 fn {entries_then_private}、"
                    f"改条目 {entries_changed}")
        # cfg 原子的规格原文是它所在的条目整个，与 uninterp 同一种半开区间：clamp_to_limit 里一处 cfg!(debug_assertions)，后面接着的 keep_small 换了头一个词
        # （pub fn 换成 fn）sha256 不变，改 clamp_to_limit 自己就变（开关 cfg-entry-past-unit-end 下判错：连 keep_small 的头一个词一起取；whole-entry-past-unit-end 下同样判错）
        def cfg_entry_digests(name, kernel):
            root = make_root(name, kernel=kernel)
            relative = "crates/core/src/kernel.rs"
            rust_file = FunctionUnitIndex(root).workspace().file(relative)
            return [entry["digest"] for entry in trusted_declarations(rust_file, relative, "core_sample", "kernel", region_spans(rust_file), set())
                    if entry["kind"] == CFG_KIND]

        cfg_then_public = cfg_entry_digests("cfgentrypublic", cfg_kernel)
        cfg_then_private = cfg_entry_digests("cfgentryprivate", cfg_kernel.replace("pub fn keep_small(", "fn keep_small(", 1))
        cfg_changed = cfg_entry_digests("cfgentrychanged", cfg_kernel.replace("limit > 0,", "limit > 1,", 1))
        expect_true("trust-digest-cfg-entry-half-open",
                    len(cfg_then_public) == len(cfg_then_private) == len(cfg_changed) == 1
                    and cfg_then_public == cfg_then_private and cfg_then_public != cfg_changed,
                    f"期望 cfg 原子所在条目的 sha256 不随下一个条目的头一个词变、改条目自己就变，实测 后接 pub {cfg_then_public}、后接 fn {cfg_then_private}、"
                    f"改条目 {cfg_changed}")

        # trait 里没有函数体的声明、ensures 里有结构体字面量：按切出的单元认（以 ; 收尾），不成项（开关 trait-declaration-by-brace 下 Store::put 成项、之和 4 与登记的 3 不同）
        expect("proof-trait-declaration-struct-literal", "proof", make_root("traitliteral", kernel=selftest_kernel_with(SELFTEST_TRAIT_LITERAL_EXTRA), count=3), 0,
               ("proof:core_sample::kernel::Shelf::put：绿", "成项 3 个（exec fn 3）"))
        # MIR：产品构建与 Verus 构建的函数体不同判红（开关 mir-compare-skipped 下判错）；判绿时成功句报比了几个条目
        expect("mir-divergence", "proof", make_root("mirdivergence", kernel=SELFTEST_KERNEL.replace(body, body + " // SELFTEST_MIR_VERUS_ONLY")), 1,
               "MIR 比对：产品构建 2 个条目里，1 个与 Verus 构建的函数体不同（fn kernel::clamp_to_limit）")
        expect("mir-green", "proof", green, 0, "产品构建与 verus --compile 构建的 MIR 逐条目比：产品的 2 个条目都在、整段逐字相同")
        # MIR 规整表（MIR_PATH_ALIASES）：产品构建印 std::rt::panic_fmt、Verus 构建印 core::panicking::panic_fmt 的同一个函数，规整之后判绿、报两边各改写几处
        # （开关 mir-panic-path-unnormalized 下判错）；同一个条目里另有一处真分叉照报（开关 mir-aliased-items-skipped 下判错）；
        # 表外的同类名字照红（开关 mir-alias-by-prefix 下判错）；前面接 ::、后面接标识符字符的不改（开关 mir-alias-without-boundary 下判错）
        rt_call = " // SELFTEST_MIR_RT_PATH _6 = core::panicking::panic_fmt(move _7) -> bb18;"
        aliased_kernel = SELFTEST_KERNEL.replace(body, body + rt_call).replace(
            "if value > 10 { pick(10) } else { pick(value) }", "if value > 10 { pick(10) } else { pick(value) } // SELFTEST_MIR_RT_PATH _3 = core::panicking::panic_fmt::<u64>(move _4);")
        expect("mir-panic-path-normalized-green", "proof", make_root("mirpanicpath", kernel=aliased_kernel), 0,
               ("MIR 逐条目比：产品的 2 个条目都在、整段逐字相同",
                "比之前照规整表改写了产品构建 2 处、Verus 构建 0 处（std::rt::panic_fmt → core::panicking::panic_fmt，同一个函数的两种印法）"))
        diverged_kernel = SELFTEST_KERNEL.replace(body, body + rt_call).replace("        limit > 0,\n", "        limit > 0, // SELFTEST_MIR_VERUS_ONLY\n", 1)
        expect("mir-panic-path-divergence-red", "proof", make_root("mirpanicdivergence", kernel=diverged_kernel), 1,
               ("MIR 比对：产品构建 2 个条目里，1 个与 Verus 构建的函数体不同（fn kernel::clamp_to_limit）", "比之前照规整表改写了产品构建 1 处、Verus 构建 0 处"))
        unlisted_kernel = SELFTEST_KERNEL.replace(body, body + " // SELFTEST_MIR_RT_PATH _3 = core::panicking::panic_display::<u64>(move _4) -> bb2;")
        expect("mir-unlisted-panic-path-red", "proof", make_root("mirunlisted", kernel=unlisted_kernel), 1,
               ("1 个与 Verus 构建的函数体不同（fn kernel::clamp_to_limit）", "比之前照规整表改写了产品构建 0 处、Verus 构建 0 处"))
        boundary_kernel = SELFTEST_KERNEL.replace(body, body + " // SELFTEST_MIR_RT_PATH _6 = wrapper::core::panicking::panic_fmt(move _7);").replace(
            "        result <= limit,\n", "        result <= limit, // SELFTEST_MIR_RT_PATH _8 = core::panicking::panic_fmt_with(move _9);\n", 1)
        expect("mir-panic-path-boundary-red", "proof", make_root("mirboundary", kernel=boundary_kernel), 1,
               ("1 个与 Verus 构建的函数体不同（fn kernel::clamp_to_limit）", "比之前照规整表改写了产品构建 0 处、Verus 构建 0 处"))
        # Verus 与产品的 rustc 版本差进工具链钉表（开关 rustc-pin-unchecked 下判错）
        expect("rustc-pin-product", "toolchain", make_root("rustcproduct", rustc_row=f"rustc\tstable\t1.0.1\t{'d' * 40}\n"), 1, "产品的 rustc 是 1.0.0")
        expect("rustc-pin-verus", "toolchain", make_root("rustcverus", rustc_row=f"rustc\tnightly\t1.0.0\t{'d' * 40}\n"), 1, "Verus 用的工具链是 stable")
        expect("rustc-pin-missing", "toolchain", make_root("rustcmissing", rustc_row=""), 1, "没有 rustc 行")
        # cargo 那一种（crate 行第四段 cargo）：经 cargo verus focus 验、路径依赖连同 crate 一起拷，报错位置相对副本根（开关 cargo-mode-ignored 下判错）
        cargo_root = make_root("cargomode", crate_mode="\tcargo", extra_dependency='singlefs-format = { path = "../format" }\n')
        os.makedirs(os.path.join(cargo_root, "crates", "format", "src"))
        open(os.path.join(cargo_root, "crates", "format", "Cargo.toml"), "w").write('[package]\nname = "singlefs-format"\nedition = "2021"\n')
        open(os.path.join(cargo_root, "crates", "format", "src", "lib.rs"), "w").write("pub const LIMIT: u64 = 10;\n")
        cargo_arguments = os.path.join(work, "cargo-verus-arguments.txt")
        open(cargo_arguments, "w").close()
        os.environ["SELFTEST_CARGO_VERUS_ARGUMENTS"] = cargo_arguments
        try:
            expect("cargo-mode-proof", "proof", cargo_root, 0, ("Verus 逐项验 crates/core（cargo verus focus）", "cargo verus build 构建的 MIR 逐条目比"))
            expect("cargo-mode-vacuity", "vacuity", cargo_root, 0, "2 处 requires")
            expect("cargo-mode-must-red", "must-red-worlds", cargo_root, 0, "证明错落点与第六段 landing= 相同")
        finally:
            os.environ.pop("SELFTEST_CARGO_VERUS_ARGUMENTS", None)
        # 几处同时验共用一个编译目录（一处一个 job、两项轮流用它）：每处起之前删被验的 crate 的指纹，第二项照样出判定行（开关 cargo-fingerprint-kept 下第二项红）
        shared_target_exit, shared_target_output = captured_stdout(run_check, "proof", cargo_root, dict(options, jobs=1, verdict_library=fresh_library()))
        expect_true("cargo-mode-shared-build-directory", shared_target_exit == 0 and "proof:core_sample::kernel::keep_small：绿" in shared_target_output.getvalue(),
                    f"一个编译目录轮流验两项期望都判绿，实测退 {shared_target_exit}：{one_line(shared_target_output.getvalue().strip()[-300:])}")
        cargo_runs = [json.loads(line) for line in open(cargo_arguments, encoding="utf-8").read().split("\n") if line]
        expect_true("cargo-mode-command", bool(cargo_runs) and all(run[:6] in (["verus", "focus", "-p", "core-sample", "--offline", "--"],
                                                                                  ["verus", "build", "-p", "core-sample", "--offline", "--"]) for run in cargo_runs)
                    and any(run[1] == "build" for run in cargo_runs) and not any("--no-cheating" in run for run in cargo_runs),
                    f"cargo-verus 期望照 verus focus|build -p core-sample --offline -- … 起、不带 --no-cheating，实测 {cargo_runs[:3]}")
        # 几处同时验的编译目录（claim_slot_directories）：环境里有绝对路径的 CARGO_TARGET_DIR（服务给这一趟的编译目录）时建在它底下，先后几次调用共用同一批、放掉只放锁不删，
        # 一次还占着的编号下一次跳过；没有这个变量时在临时目录里建、放掉整个删（开关 slot-directory-per-call 下一律每次新建临时目录，这一格判错）；
        # 再经 run_check 在 CARGO_TARGET_DIR 下接连判两趟，两趟都判绿
        service_target = os.path.join(work, "service-given-target")
        os.makedirs(service_target)
        os.environ["CARGO_TARGET_DIR"] = service_target
        try:
            held_paths, release_held = claim_slot_directories(2)
            skipped_paths, release_skipped = claim_slot_directories(2)
            release_held()
            reclaimed_paths, release_reclaimed = claim_slot_directories(2)
            release_reclaimed()
            release_skipped()
            service_exit_first, service_output_first = captured_stdout(run_check, "proof", cargo_root, dict(options, jobs=1, verdict_library=fresh_library()))
            service_exit_second, service_output_second = captured_stdout(run_check, "proof", cargo_root, dict(options, jobs=1, verdict_library=fresh_library()))
        finally:
            os.environ.pop("CARGO_TARGET_DIR", None)
        slot_parent = os.path.join(service_target, SLOT_DIRECTORY_NAME)
        plain_paths, release_plain = claim_slot_directories(1)
        plain_parent = os.path.dirname(plain_paths[0])
        plain_made = os.path.isdir(plain_parent) and not plain_parent.startswith(service_target)
        release_plain()
        expect_true("slot-directories-shared-across-calls",
                    held_paths == [os.path.join(slot_parent, "target-0"), os.path.join(slot_parent, "target-1")]
                    and skipped_paths == [os.path.join(slot_parent, "target-2"), os.path.join(slot_parent, "target-3")]
                    and reclaimed_paths == held_paths and os.path.isfile(os.path.join(slot_parent, "target-0.lock"))
                    and service_exit_first == 0 and service_exit_second == 0
                    and "proof:core_sample::kernel::keep_small：绿" in service_output_first.getvalue() and "proof:core_sample::kernel::keep_small：绿" in service_output_second.getvalue()
                    and plain_made and not os.path.exists(plain_parent),
                    f"CARGO_TARGET_DIR 下第一次占 {held_paths}、还占着时第二次占 {skipped_paths}、放掉后再占 {reclaimed_paths}；两趟 run_check 退 {service_exit_first}、{service_exit_second}；"
                    f"没有变量时在临时目录 {plain_parent}（建了 {plain_made}，放掉后还在 {os.path.exists(plain_parent)}）")
        # 几个 crate（crate 行可以多条）：逐个 crate 判、各比各的登记数，--item 分到它所属的 crate（开关 single-crate-registry 下判错）
        multi_root = make_root("multicrate")
        os.makedirs(os.path.join(multi_root, "crates", "second", "src"))
        open(os.path.join(multi_root, "crates", "second", "Cargo.toml"), "w").write('[package]\nname = "second-sample"\nedition = "2021"\n\n[dependencies]\nvstd = "=0.0.1"\n')
        open(os.path.join(multi_root, "crates", "second", "src", "lib.rs"), "w").write(
            "use vstd::prelude::*;\nverus! {\npub fn double_it(value: u64) -> (result: u64)\n    requires\n        value < 100,\n    ensures\n        result == value + value,\n"
            "{\n    value + value\n}\n} // verus!\n")
        open(os.path.join(multi_root, "crates", "second", "verus-must-red-worlds.tsv"), "w").write(
            "# 自证用\n第二个 crate\tcrates/second/src/lib.rs\t    value + value\t    value + value // SELFTEST_BROKEN\tproof_errors=1\tlanding=double_it\n")
        with open(os.path.join(multi_root, REGISTRY), "a") as handle:
            handle.write("crate\tcrates/second\t1\nkernel\tcrates/second/src/lib.rs\nmust-red-worlds\tcrates/second/verus-must-red-worlds.tsv\n")
        expect("multi-crate-proof", "proof", multi_root, 0, ("proof:second_sample::double_it：绿", "proof:core_sample::kernel::clamp_to_limit：绿", "登记的 2 个 crate 判了 2 个"))
        multi_listed, multi_listing = captured_stdout(list_items, multi_root)
        expect_true("multi-crate-list-items", multi_listed == 0 and "proof:second_sample::double_it\tverus-proof" in multi_listing.getvalue()
                    and "vacuity:second_sample::double_it#1\tverus-vacuity" in multi_listing.getvalue() and "2 个 crate 成项 3 个" in multi_listing.getvalue(),
                    f"期望列出两个 crate 的项：{one_line(multi_listing.getvalue().strip()[-400:])}")
        multi_options = dict(options, items=["proof:second_sample::double_it"], verdict_library=fresh_library())
        multi_item_exit, multi_item_buffer = captured_stdout(run_check, "proof", multi_root, multi_options)
        expect_true("multi-crate-item-routed", multi_item_exit == 0 and "crates/core：--item 没点到这个 crate 的项" in multi_item_buffer.getvalue()
                    and "proof:second_sample::double_it：绿" in multi_item_buffer.getvalue(),
                    f"--item 点第二个 crate 的项期望只判它，实测退 {multi_item_exit}：{one_line(multi_item_buffer.getvalue().strip()[-300:])}")
        # 带值的 const 与 assume_specification 各成一项（core 的 DataUnitIndexInFile::FIRST 与两条格式常量规格的样子）：假 Verus 整 crate 验出
        # 2（exec fn）+ 2（两个带值的 const）+ 1（指向常量、写 returns 的那一条）+ 0（指向 std 方法的那一条）= 5，各项之和要等于它
        # （开关 valued-const-not-item 下少 2 条、assume-specification-not-item 下少 1 条，判红）；指向 std 方法的那一项验出 0 条照判绿
        const_kernel = selftest_kernel_with(SELFTEST_CONST_EXTRA)
        _harvest_exit, harvested = captured_stdout(run_check, "proof", make_root("constharvest", kernel=const_kernel, count=5), dict(options, verdict_library=fresh_library()))
        const_digests = dict(re.findall(r"的 assume_specification (\S+) 没登记进信任清单 \S+（这一处的规格原文 sha256 ([0-9a-f]{64})）", harvested.getvalue()))
        const_rows = [f"assume_specification\tcrates/core/src/kernel.rs\t{point}\t{const_digests.get(point, '0' * 64)}\tselftest_differential\t自证用：登记的可信声明"
                      for point in ("selftest_format::SLOT_WIDTH", "<[T]>::binary_search")]
        expect("proof-valued-const-and-assume-specification-items", "proof", make_root("constitems", kernel=const_kernel, count=5, trust_rows=const_rows), 0,
               ("成项 6 个（exec fn 2、带值的 const 2、assume_specification 2），各项 verified 之和 5、0 errors（登记 5）",
                "proof:core_sample::kernel::Width::NARROWEST：绿", "proof:core_sample::kernel::WIDEST_SAMPLE：绿",
                "proof:core_sample::kernel::assume_specification[selftest_format::SLOT_WIDTH]：绿", "proof:core_sample::kernel::assume_specification[<[T]>::binary_search]：绿"))
        # 关联 const 与自由 const 名字一前一后互为前缀（core 的 JournalRecordOrdinalWithinPublish::FIRST 与 FIRST_JOURNAL_COUNTER 同在 journal 的样子）：
        # 关联 const 点 *::FIRST、自由 const 点 FIRST_COUNTER，各恰好命中一项，列项退 0、逐项验各 1 条、之和 6 等于登记（开关 associated-const-bare-name 下
        # 关联 const 点 FIRST，表里 FIRST_COUNTER 与 impl&%?::FIRST 都包含它，列项与证明都判红）
        prefix_root = make_root("constprefix", kernel=selftest_kernel_with(SELFTEST_CONST_PREFIX_EXTRA), count=6)
        prefix_listed, prefix_listing = captured_stdout(list_items, prefix_root)
        expect_true("list-items-associated-const-prefix", prefix_listed == 0
                    and all(f"proof:core_sample::kernel::{name}\tverus-proof\t带值的 const " in prefix_listing.getvalue()
                            for name in ("Ordinal::FIRST", "FIRST_COUNTER", "LAST", "Ordinal::LAST_SLOT")),
                    f"期望列项退 0、四个 const 各成一项，实测退 {prefix_listed}：{one_line(prefix_listing.getvalue().strip()[-400:])}")
        prefix_arguments = os.path.join(work, "constprefix-arguments.jsonl")
        os.environ["SELFTEST_VERUS_ARGUMENTS"] = prefix_arguments
        try:
            expect("proof-associated-const-prefix-green", "proof", prefix_root, 0,
                   ("成项 6 个（exec fn 2、带值的 const 4），各项 verified 之和 6、0 errors（登记 6）",
                    "proof:core_sample::kernel::Ordinal::FIRST：绿", "proof:core_sample::kernel::FIRST_COUNTER：绿"))
        finally:
            os.environ.pop("SELFTEST_VERUS_ARGUMENTS", None)
        prefix_points = [arguments[arguments.index("--verify-function") + 1] for arguments in
                         (json.loads(line) for line in (open(prefix_arguments).read().split("\n") if os.path.isfile(prefix_arguments) else []) if line)
                         if "--verify-function" in arguments]
        expect_true("proof-associated-const-point-names", sorted(point for point in prefix_points if "FIRST" in point or "LAST" in point)
                    == ["*::FIRST", "*::LAST_SLOT", "FIRST_COUNTER", "LAST"],
                    f"期望关联 const 点 *::<名>、自由 const 点 <名>，实测交给 --verify-function 的 {prefix_points}")
        # 撞名当场判红并点名撞的几项（起 Verus 之前）：trait 里带默认值的 Limited::LIMIT 与 impl 里覆盖它的 LIMIT，*::LIMIT 两个都以 ::LIMIT 收尾
        # （开关 point-names-by-containment 下的判法只比成项的名字谁是谁的一段，放过它）；同一个模块两个 impl 各有一个 FIRST
        expect("proof-point-name-trait-default-collision-red", "proof",
               make_root("consttraitdefault", kernel=selftest_kernel_with(SELFTEST_CONST_TRAIT_DEFAULT_EXTRA), count=4), 1,
               ("proof:core_sample::kernel::Ordinal::LIMIT 交给 --verify-function 的 *::LIMIT 在这个模块的 Verus 可点名函数表里命中 2 项",
                "const Limited::LIMIT（crates/core/src/kernel.rs:", "const Ordinal::LIMIT（crates/core/src/kernel.rs:"))
        expect("proof-point-name-two-impls-collision-red", "proof",
               make_root("consttwoimpls", kernel=selftest_kernel_with(SELFTEST_CONST_PREFIX_EXTRA + SELFTEST_CONST_SECOND_IMPL_EXTRA), count=7), 1,
               ("交给 --verify-function 的 *::FIRST 在这个模块的 Verus 可点名函数表里命中 2 项",
                "const Ordinal::FIRST（crates/core/src/kernel.rs:", "const OtherOrdinal::FIRST（crates/core/src/kernel.rs:"))
        # 空真插入点的说明认 const fn（core 的 SlotNumber::to_device_offset）：列项里 const fn 那一处写「fn <它自己>」（开关 const-fn-heading-unrecognized 下写成上一个 fn）
        const_fn_listed, const_fn_listing = captured_stdout(list_items, make_root("constfn", kernel=selftest_kernel_with(SELFTEST_CONST_FN_EXTRA), count=3))
        expect_true("vacuity-const-fn-label", const_fn_listed == 0
                    and "vacuity:core_sample::kernel::to_slot_offset#1\tverus-vacuity\tcrates/core/src/kernel.rs fn to_slot_offset\n" in const_fn_listing.getvalue(),
                    f"期望 const fn 那一处的说明是 fn to_slot_offset，实测退 {const_fn_listed}：{one_line(const_fn_listing.getvalue().strip()[-400:])}")
        # 属于 assert(…) by (…) 的 requires 不是空真插入点（core 的 by (nonlinear_arith) 接 requires … ;）：列项退 0、halve_within 只有入口那一处，
        # 列项的 proof 那一行与空真、证明两格的成功句报「跳过 assert-by requires 1 处」并点名那一行（开关 assert-by-requires-as-insertion-point 下三格判错）；
        # requires 挂在普通块与不带 by 的 assert 上、下面一行只有 ; 的照旧判红、红句带文件名（开关 assert-by-requires-by-semicolon-line 下这两格判错）
        assert_by_kernel = selftest_kernel_with(SELFTEST_ASSERT_BY_EXTRA)
        assert_by_rows = assert_by_kernel.split("\n")
        assert_by_line = next((number for number, text in enumerate(assert_by_rows, 1) if text.strip() == "requires" and "by (nonlinear_arith)" in assert_by_rows[number - 2]), None)
        assert_by_skipped = f"跳过 assert-by requires 1 处（属于 assert(…) by (…)，在那一处先证出来，不是外层函数的前提）：crates/core/src/kernel.rs:{assert_by_line}"
        assert_by_root = make_root("assertby", kernel=assert_by_kernel, count=3)
        assert_by_listed, assert_by_listing = captured_stdout(list_items, assert_by_root)
        expect_true("list-items-assert-by-requires-skipped", assert_by_listed == 0 and f"；空真{assert_by_skipped}\n" in assert_by_listing.getvalue()
                    and [row.split("\t")[0] for row in assert_by_listing.getvalue().split("\n") if row.startswith("vacuity:core_sample::kernel::halve_within#")]
                    == ["vacuity:core_sample::kernel::halve_within#1"],
                    f"期望列项退 0、proof 那一行报「{assert_by_skipped}」、halve_within 只有 #1，实测退 {assert_by_listed}：{one_line(assert_by_listing.getvalue().strip()[-500:])}")
        expect("vacuity-assert-by-requires-skipped", "vacuity", assert_by_root, 0, ("1 份核里 3 处 requires，", assert_by_skipped))
        expect("proof-assert-by-requires-skip-reported", "proof", assert_by_root, 0, ("成项 3 个（exec fn 3）", f"空真那一格{assert_by_skipped}"))
        for stray_label, stray_extra in (("list-items-requires-on-plain-block-red", SELFTEST_PLAIN_BLOCK_REQUIRES_EXTRA),
                                         ("list-items-requires-on-assert-without-by-red", SELFTEST_ASSERT_WITHOUT_BY_EXTRA)):
            stray_kernel = selftest_kernel_with(stray_extra)
            stray_line = max(number for number, text in enumerate(stray_kernel.split("\n"), 1) if text.strip() == "requires")
            stray_exit, stray_listing = captured_stdout(list_items, make_root(stray_label, kernel=stray_kernel, count=3))
            expect_true(stray_label, stray_exit == 1 and f"crates/core/src/kernel.rs 第 {stray_line} 行的 requires 找不到函数体的 {{" in stray_listing.getvalue(),
                        f"期望列项退 1、红句点名 crates/core/src/kernel.rs 第 {stray_line} 行，实测退 {stray_exit}：{one_line(stray_listing.getvalue().strip()[-400:])}")
        # spec const 与别的带值的 const 一样成项（真 Verus 整 crate 验时各算 1 条）：impl 里的点 *::SPEC_WIDTH、自由的点 SPEC_HEIGHT，之和 4 等于登记
        # （开关 spec-const-not-item 下两个 spec const 不成项、之和 2 与登记的 4 不同）
        expect("proof-spec-const-items", "proof", make_root("specconst", kernel=selftest_kernel_with(SELFTEST_SPEC_CONST_EXTRA), count=4), 0,
               ("成项 4 个（exec fn 2、带值的 const 2），各项 verified 之和 4、0 errors（登记 4）",
                "proof:core_sample::kernel::Window::SPEC_WIDTH：绿", "proof:core_sample::kernel::SPEC_HEIGHT：绿"))
        # 几处同时验不按内存收（内存额度归 task-server）：外层内存上限 1G、--threads 8、两处同时验，每处起的 Verus 都带 --num-threads 4
        # （开关 outer-memory-narrows-jobs 下收到 1 处、每处 8 条，判错）
        jobs_arguments = os.path.join(work, "jobs-verus-arguments.jsonl")
        saved_outer = os.environ.get(OUTER_MEMORY_CAP_VARIABLE)
        os.environ.update({OUTER_MEMORY_CAP_VARIABLE: "1G", "SELFTEST_VERUS_ARGUMENTS": jobs_arguments})
        try:
            jobs_registry, _jobs_kernels, jobs_manifest, jobs_executable = prepared(green, options)
            jobs_outcomes = verify_in_parallel(green, jobs_registry, jobs_manifest, jobs_executable, dict(options, jobs=0, threads=8),
                                               [("整 crate 甲", {}), ("整 crate 乙", {})])
        finally:
            os.environ.pop("SELFTEST_VERUS_ARGUMENTS", None)
            if saved_outer is None:
                os.environ.pop(OUTER_MEMORY_CAP_VARIABLE, None)
            else:
                os.environ[OUTER_MEMORY_CAP_VARIABLE] = saved_outer
        jobs_threads = [arguments[arguments.index("--num-threads") + 1] if "--num-threads" in arguments else None
                        for arguments in (json.loads(line) for line in open(jobs_arguments, encoding="utf-8").read().split("\n") if line.strip())] \
            if os.path.isfile(jobs_arguments) else []
        expect_true("jobs-not-narrowed-by-outer-memory", len(jobs_outcomes) == 2 and jobs_threads == ["4", "4"],
                    f"外层上限 1G、--threads 8、两处：期望起两次 Verus、各带 --num-threads 4，实测收回 {len(jobs_outcomes)} 处、--num-threads {jobs_threads}")
        # Verus 退了先停它的进程组里留下的：直接起的 bash 在后台起一个拿着 stdout 与 stderr 的 sleep 20 就退，判法几秒内交回、sleep 也不在了
        # （开关 group-leftovers-kept 下不停，判法等到 sleep 自己退才交回，判错；留下的 sleep 按记下的进程号停掉）
        child_file = os.path.join(work, "leftover-child-pid")
        group_started = time.monotonic()
        group_process, group_stdout, _group_stderr = run_in_own_process_group(
            ["bash", "-c", 'sleep 20 & echo "$!" > "$1"; echo left-a-child', "leftover-group", child_file], cwd=work)
        group_seconds = time.monotonic() - group_started
        child_pid = int(open(child_file, encoding="utf-8").read().strip() or 0) if os.path.isfile(child_file) else 0
        child_alive = False
        for _attempt in range(20):
            try:
                with open(f"/proc/{child_pid}/stat", encoding="utf-8") as handle:
                    child_alive = child_pid > 0 and handle.read().rsplit(")", 1)[1].split()[0] != "Z"
            except (OSError, IndexError):
                child_alive = False
            if not child_alive:
                break
            time.sleep(0.1)
        if child_alive:
            os.kill(child_pid, signal.SIGKILL)   # 这一格自己起的那一个 sleep，按记下的进程号停
        expect_true("group-leftovers-stopped", group_process.returncode == 0 and "left-a-child" in group_stdout
                    and child_pid > 0 and not child_alive and group_seconds < 10,
                    f"期望 bash 退 0、10 秒内交回、后台的 sleep（进程号 {child_pid}）也停了，"
                    f"实测退 {group_process.returncode}、用了 {group_seconds:.1f} 秒、sleep 还在 {child_alive}")
        # 每个 Verus 进程的内存上限没有了：--memory-cap 当认不出的选项，退 2（用法错）
        try:
            parse_options([green, "--memory-cap", "x"])
            memory_option = "收下了"
        except ValueError as error:
            memory_option = str(error)
        expect_true("memory-cap-option-gone", memory_option == "认不出的选项 --memory-cap", f"期望「认不出的选项 --memory-cap」，实测：{memory_option}")
        expect("must-red-survives", "must-red-worlds", make_root("survives", must_red=["改注释\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // 只是注释\tproof_errors=1\tlanding=clamp_to_limit"]), 1, "照样验过")
        expect("must-red-anchor", "must-red-worlds", make_root("anchor", must_red=["不在的原文\tcrates/core/src/kernel.rs\tnot present anywhere\tx\tproof_errors=1\tlanding=clamp_to_limit"]), 1, "恰好命中一次")
        # 必红世界按落点判、条数只报（C654）：证明错条数与第五段 proof_errors= 不同而落点没变，判绿并在成功句里数进「只报不判」（开关 landing-by-count 下判错）；
        # 同一个 fn 里多报两条（条数 3，登记 1）也是；落点换了函数而条数没变判红（开关 landing-unchecked、landing-by-count 下判错）
        broken_line = "if value > limit { limit } else { value } // SELFTEST_BROKEN"
        clamp_row = "\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
        expect("must-red-proof-error-count-drift", "must-red-worlds",
               make_root("proofcount", must_red=["条数不对" + clamp_row + broken_line + "\tproof_errors=2\tlanding=clamp_to_limit"]),
               0, ("证明错落点与第六段 landing= 相同", "1 行证明错条数与第五段 proof_errors= 不同，只报不判（条数不对）"))
        expect("must-red-landing-same-function-more-errors", "must-red-worlds",
               make_root("moreerrors", must_red=["多报两条" + clamp_row + broken_line + "\\n    // SELFTEST_BROKEN\\n    // SELFTEST_BROKEN\tproof_errors=1\tlanding=clamp_to_limit"]),
               0, ("证明错落点与第六段 landing= 相同", "1 行证明错条数与第五段 proof_errors= 不同，只报不判（多报两条）"))
        expect("must-red-landing-moved", "must-red-worlds",
               make_root("landingmoved", must_red=["落到别的函数" + "\tcrates/core/src/kernel.rs\tif value > 10 { pick(10) } else { pick(value) }\t"
                                                   "if value > 10 { pick(10) } else { pick(value) } // SELFTEST_BROKEN\tproof_errors=1\tlanding=clamp_to_limit"]),
               1, "证明错落在 keep_small，表里写的落点是 clamp_to_limit")
        # 落点按改坏之后的文本找（替换文比原文多几行，后面的行都挪了位置）：改坏的是 clamp_to_limit，原来就带标记的 tiny 在它后面，落点是 tiny；
        # 读改坏之前的文本找就落到后面的 keep_small（开关 landing-read-from-unmutated-text 下判错）
        shifted = "if value > limit { limit } else { value }" + "\\n" * 9
        expect("must-red-landing-line-shift", "must-red-worlds",
               make_root("landingshift", kernel=SELFTEST_LANDING_KERNEL, must_red=["往后挪九行" + clamp_row + shifted + "\tproof_errors=1\tlanding=tiny"]),
               0, "证明错落点与第六段 landing= 相同")
        # 两个函数各报一条：落点是两个名字的集合，登记的顺序随意
        expect("must-red-landing-two-functions", "must-red-worlds",
               make_root("twofunctions", kernel=SELFTEST_LANDING_KERNEL, must_red=["两处" + clamp_row + broken_line + "\tproof_errors=2\tlanding=tiny,clamp_to_limit"]),
               0, "证明错落点与第六段 landing= 相同")
        expect("must-red-landing-two-functions-one-registered", "must-red-worlds",
               make_root("twoonereg", kernel=SELFTEST_LANDING_KERNEL, must_red=["两处只登记一处" + clamp_row + broken_line + "\tproof_errors=2\tlanding=clamp_to_limit"]),
               1, "证明错落在 clamp_to_limit、tiny，表里写的落点是 clamp_to_limit")
        # 证明错的位置不在任何 fn 里（核的第一行）：落点记（函数外）
        expect("must-red-landing-outside-function", "must-red-worlds",
               make_root("outsidefunction", must_red=["函数外\tcrates/core/src/kernel.rs\tuse vstd::prelude::*;\tuse vstd::prelude::*; // SELFTEST_BROKEN\tproof_errors=1\tlanding=（函数外）"]),
               0, "证明错落点与第六段 landing= 相同")
        # 不是六段的行：判红，报行号、要六段（开关 row-segments-unchecked 下判错）；第六段不是 landing=<落点>：判红（开关 landing-form-unchecked 下判错）
        expect("must-red-row-segments", "must-red-worlds", make_root("rowsegments", must_red=["少一段" + clamp_row + broken_line + "\tproof_errors=1"]),
               1, ("第 2 行不是六段", "要六段制表符分隔"))
        expect("must-red-landing-form", "must-red-worlds", make_root("landingform", must_red=["落点写法不对" + clamp_row + broken_line + "\tproof_errors=1\tclamp_to_limit"]),
               1, ("的第六段「clamp_to_limit」不是 landing=<落点>", "必红世界表第六段一律写 landing="))
        expect("must-red-empty", "must-red-worlds", make_root("empty", must_red=[]), 1, "一行都没有")
        # rlimit：登记的数写上 Verus 命令行（假 Verus 见到的 --rlimit 小于标记要的数就报 rlimit 用尽）；没有 rlimit 行取 DEFAULT_RLIMIT；
        # 必红世界里只因 rlimit 用尽才报错的那一行不作数、判红（rlimit 用尽不计入证明错条数）
        needs_seven = SELFTEST_KERNEL.replace("result <= limit,", "result <= limit, // SELFTEST_NEEDS_RLIMIT_7")
        expect("proof-rlimit-registered", "proof", make_root("rlimitregistered", kernel=needs_seven, rlimit=7), 0, "--rlimit 7")
        expect("proof-rlimit-too-low", "proof", make_root("rlimitlow", kernel=needs_seven, rlimit=3), 1, "Resource limit (rlimit) exceeded")
        expect("proof-rlimit-default", "proof", make_root("rlimitdefault", kernel=SELFTEST_KERNEL.replace("result <= limit,", f"result <= limit, // SELFTEST_NEEDS_RLIMIT_{DEFAULT_RLIMIT}"),
                                                          rlimit=None), 0, f"--rlimit {DEFAULT_RLIMIT}")
        expect("registry-rlimit-zero", "proof", make_root("rlimitzero", rlimit=0), 1, "rlimit 要是正整数")
        expect("must-red-rlimit", "must-red-worlds", make_root("mustredrlimit", must_red=["只撞 rlimit\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
                                                                                        "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_99\tproof_errors=1\tlanding=clamp_to_limit"]),
               1, "这一行红在算力额度上，不是证明失败")
        # 输出里夹着 rlimit 用尽、另有一条真的证明错：红计入（只剩 rlimit 用尽的才不作数）
        # rlimit 用尽不计入证明错条数：证明错 1 条对得上登记的 proof_errors=1，成功句里没有「条数不同、只报不判」那一段
        # （开关 rlimit-counted-in-proof-errors 下条数成 2，成功句多出那一段：下面的 no-count-drift 一格判错）
        count_drift_text = "证明错条数与第五段 proof_errors= 不同"
        with_proof_error_output = expect("must-red-rlimit-with-proof-error", "must-red-worlds", make_root("mustredrlimitproof", must_red=[
            "撞 rlimit 又有证明错\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
            "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_99 SELFTEST_BROKEN\tproof_errors=1\tlanding=clamp_to_limit"]),
               0, "1 行输出里夹着 rlimit 用尽、另有证明错，红计入")
        expect_absent("must-red-rlimit-with-proof-error-no-count-drift", with_proof_error_output, count_drift_text)
        # 同一行在 rlimit 10 上输出一条 rlimit 用尽加一条证明错（Verus 把 rlimit 用尽也计进 errors，errors 2），
        # 在 rlimit 20 上只输出同一条证明错（errors 1）；第五段 proof_errors=1 只数证明错，两份都判绿，成功句里都没有「条数不同」那一段
        # （开关 rlimit-counted-in-proof-errors 下 rlimit 10 那份条数成 2、成功句多出那一段：no-count-drift 一格判错）
        j3_row = ("rlimit 用尽另记\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
                  "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_15\\n    // SELFTEST_BROKEN\t")
        rlimit_10_output = expect("must-red-rlimit-10-counts-proof-errors-only", "must-red-worlds",
                                  make_root("j3rlimit10", must_red=[j3_row + "proof_errors=1\tlanding=clamp_to_limit"], rlimit=10),
                                  0, ("1 行输出里夹着 rlimit 用尽、另有证明错，红计入", "证明错落点与第六段 landing= 相同"))
        expect_absent("must-red-rlimit-10-no-count-drift", rlimit_10_output, count_drift_text)
        expect("must-red-rlimit-20-counts-proof-errors-only", "must-red-worlds", make_root("j3rlimit20", must_red=[j3_row + "proof_errors=1\tlanding=clamp_to_limit"], rlimit=20),
               0, ("0 行输出里夹着 rlimit 用尽", "证明错落点与第六段 landing= 相同"))
        # 第五段不是 proof_errors=<正整数>（写成 0）的行判红、出路写约定（开关 expectation-form-unchecked 下判错）
        expect("must-red-expectation-form", "must-red-worlds", make_root("expectationform", must_red=[j3_row + "proof_errors=0\tlanding=clamp_to_limit"], rlimit=10),
               1, ("的第五段「proof_errors=0」不是 proof_errors=<证明错条数>", "必红世界表第五段一律写 proof_errors=M"))
        # 函数上的 rlimit 属性盖过命令行的 --rlimit：核里有它，proof 判红；必红世界的替换文里有它，那一行判红
        expect("proof-rlimit-attribute", "proof", make_root("rlimitattribute", kernel=SELFTEST_KERNEL.replace(
            "pub fn clamp_to_limit(", "#[verifier::rlimit(20)]\npub fn clamp_to_limit(")), 1, "verifier::rlimit（函数上的 rlimit 属性盖过命令行的 --rlimit 7")
        expect("must-red-rlimit-attribute", "must-red-worlds", make_root("mustredrlimitattribute", must_red=[
            "加 rlimit 属性\tcrates/core/src/kernel.rs\tpub fn clamp_to_limit(\t#[verifier::rlimit(20)] // SELFTEST_BROKEN\\npub fn clamp_to_limit(\tproof_errors=1\tlanding=clamp_to_limit"]),
               1, "的替换文里有 verifier::rlimit")
        # JSON 里缺整数的 verified 与 errors（vir 错没走到验证、或 Verus 换了 JSON 的写法）：那一行判红
        expect("must-red-counts-missing", "must-red-worlds", make_root("mustrednocounts", must_red=[
            "JSON 缺数\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
            "if value > limit { limit } else { value } // SELFTEST_BROKEN SELFTEST_NO_COUNTS\tproof_errors=1\tlanding=clamp_to_limit"]),
               1, "Verus 的 JSON 里没有整数的 verified 与 errors")
        # ── 逐项点名与按条复用（文件头「逐项点名与按条复用」）──
        import io
        import contextlib
        item_line = re.compile(r"^  · (\S.*?)：(绿|红)，(强制现判|现判|复用)", re.M)

        def run_items(check, root, environment=None, **changes):
            """跑一格交回 (退出码, 输出, {项: (绿/红, 现判/复用)})；changes 盖进 options（items、verdict_library、judging_units、reuse），environment 临时设进 os.environ。
            changes 里没给 verdict_library 的给一份新的空库（fresh_library）：判定库没有不读不写的形态。"""
            changes = dict({"verdict_library": fresh_library()}, **changes)
            saved_environment = {name: os.environ.get(name) for name in (environment or {})}
            os.environ.update(environment or {})
            buffer = io.StringIO()
            try:
                with contextlib.redirect_stdout(buffer):
                    got = run_check(check, root, dict(options, **changes))
            finally:
                for name, value in saved_environment.items():
                    if value is None:
                        os.environ.pop(name, None)
                    else:
                        os.environ[name] = value
            output = buffer.getvalue()
            return got, output, {match.group(1): (match.group(2), match.group(3)) for match in item_line.finditer(output)}

        def judge_cell(label, passed, detail):
            nonlocal cases
            cases += 1
            if not passed:
                wrong.append(label)
                print(one_line(f"  ✗ {label} 这一格判错：{detail}"))  # gate-lint:detail

        # ── 复用的几处修补（文件头「逐项点名与按条复用」：整格不丢、键的组成、记红撤绿、库位置）──
        # 整格不丢：proof 与 proof:<名> 一起点名照整格判（开关 proof-item-narrows 下只验点名的那一个、不比之和，这一格判错）
        core_root = make_root("proof-whole-kept")
        got, output, verdicts = run_items("proof", core_root, items=["proof", "proof:core_sample::kernel::clamp_to_limit"])
        judge_cell("proof-whole-kept", got == 0 and len(verdicts) == 2 and "各项 verified 之和 2" in output and "不比之和" not in output,
                   f"proof 与 proof:core_sample::kernel::clamp_to_limit 一起点名期望照整格判两项、比之和（退 0），实测退 {got}、判了 {list(verdicts)}：{output.strip()[-300:]}")

        # 键的组成：crate 起法（edition、包名、cargo 还是直接起）、闭包真点名的外部 crate 在 Cargo.lock 里的版本进键；RUSTUP_HOME、PATH 里 rustup 的位置不进键
        def item_key(root, entry):
            registry = read_registry(root)
            cell_options = dict(options, verdict_library=os.path.join(work, "key-only-library"), shared_components=None, unit_index=None)
            components = shared_components(root, registry, cell_options) + function_components(root, registry, cell_options, [entry], "被验函数单元闭包")
            return key_of(components, "自证的判法单元"), components

        clamp_entry, vstd_entry = "crates/core/src/kernel.rs::clamp_to_limit", "crates/core/src/kernel.rs::through_vstd"
        build_roots = {}
        for name, mode in (("key-edition-2021", ""), ("key-edition-2024", ""), ("key-cargo-mode", "\tcargo")):
            build_roots[name] = make_root(name, crate_mode=mode)
        manifest_path = os.path.join(build_roots["key-edition-2024"], "crates", "core", "Cargo.toml")
        with open(manifest_path, encoding="utf-8") as handle:
            manifest_text = handle.read()
        with open(manifest_path, "w", encoding="utf-8") as handle:
            handle.write(manifest_text.replace('edition = "2021"', 'edition = "2024"'))
        build_keys = {name: item_key(root, clamp_entry)[0] for name, root in build_roots.items()}
        judge_cell("reuse-key-crate-build", len(set(build_keys.values())) == 3,
                   f"edition 2021、edition 2024、cargo 起法三种期望三把不同的键，实测 {build_keys}")
        vstd_kernel = selftest_kernel_with("\npub fn through_vstd(value: u64) -> (result: u64)\n    ensures\n        result == value,\n{\n    vstd::prelude::identity_for_selftest(value)\n}\n")
        lock_text = 'version = 4\n\n[[package]]\nname = "core-sample"\nversion = "0.1.0"\ndependencies = [\n "vstd",\n]\n\n[[package]]\nname = "vstd"\nversion = "{0}"\nsource = "registry+https://example.invalid/selftest"\n'
        dependency_keys = {}
        for version in ("0.0.1", "0.0.2"):
            dependency_root = make_root(f"key-lock-vstd-{version}", kernel=vstd_kernel)
            with open(os.path.join(dependency_root, "Cargo.lock"), "w", encoding="utf-8") as handle:
                handle.write(lock_text.format(version))
            dependency_keys[version] = (item_key(dependency_root, vstd_entry)[0], item_key(dependency_root, clamp_entry)[0])
        judge_cell("reuse-key-closure-dependencies", dependency_keys["0.0.1"][0] != dependency_keys["0.0.2"][0],
                   f"Cargo.lock 里 vstd 0.0.1 → 0.0.2：点名 vstd:: 的 through_vstd 期望键变（另报闭包不点名它的 clamp_to_limit 变没变），实测 {dependency_keys}")
        saved_rustup_home, saved_search_path = os.environ.get("RUSTUP_HOME"), os.environ.get("PATH", "")
        toolchain_digests, printed_lines = [], []
        try:
            for variant in ("a", "b"):
                home = os.path.join(work, f"key-rustup-home-{variant}")
                library = os.path.join(home, "toolchains", "stable", "lib")
                os.makedirs(library, exist_ok=True)
                with open(os.path.join(library, "librustc_driver-selftest.so"), "wb") as handle:
                    handle.write(b"selftest driver, same content in both homes")
                program_directory = os.path.join(work, f"key-path-rustup-{variant}")
                os.makedirs(program_directory, exist_ok=True)
                with open(os.path.join(program_directory, "rustup"), "w", encoding="utf-8") as handle:
                    handle.write("#!/bin/sh\n# 自证用：两份内容相同、位置不同的 rustup\n")
                os.chmod(os.path.join(program_directory, "rustup"), 0o755)
                os.environ["RUSTUP_HOME"], os.environ["PATH"] = home, program_directory + os.pathsep + saved_search_path
                registry = read_registry(green)
                cell_options = dict(options, verdict_library=os.path.join(work, "key-only-library"), shared_components=None)
                toolchain_digests.append(dict(shared_components(green, registry, cell_options)).get("工具链"))
                printed_lines.append(tool_versions_text(green, install))
        finally:
            os.environ["PATH"] = saved_search_path
            if saved_rustup_home is None:
                os.environ.pop("RUSTUP_HOME", None)
            else:
                os.environ["RUSTUP_HOME"] = saved_rustup_home
        judge_cell("reuse-key-toolchain-without-paths", toolchain_digests[0] == toolchain_digests[1] and printed_lines[0] != printed_lines[1],
                   f"同内容的工具链与 rustup 换个位置：键里工具链那一样期望不变、tool-versions 打的行期望照旧不同，实测键 {toolchain_digests}，打的行相同 {printed_lines[0] == printed_lines[1]}")

        # 记红撤绿：先判绿入库；同一把键（只在环境里点名一处证明错，不改源码）走复用哨兵强制现判那一路判一趟红（哨兵 1 条、种子 5：keep_small 当哨兵，clamp_to_limit 被抽成强制现判）；
        # 下一趟那一项现判，别的复用（开关 red-keeps-green 下照复用，这一格判错）
        withdraw_root = make_root("red-withdraws-green")
        withdraw_store = os.path.join(work, "verdict-library-red-withdraws-green")
        first_got, _output, _verdicts = run_items("proof", withdraw_root, verdict_library=withdraw_store)
        red_got, _output, red_verdicts = run_items("proof", withdraw_root, {"SELFTEST_FAIL_MARK": "if value > limit { limit }", reuse_checks.SENTINEL_COUNT_VARIABLE: "1",
                                                                           reuse_checks.SENTINEL_SEED_VARIABLE: "5"},
                                                   verdict_library=withdraw_store)
        got, output, verdicts = run_items("proof", withdraw_root, verdict_library=withdraw_store)
        judge_cell("reuse-red-withdraws-green", first_got == 0 and red_got == 1 and red_verdicts.get("proof:core_sample::kernel::clamp_to_limit", ("", ""))[0] == "红"
                   and verdicts.get("proof:core_sample::kernel::clamp_to_limit", ("", ""))[1] == "现判" and verdicts.get("proof:core_sample::kernel::keep_small", ("", ""))[1] == "复用" and got == 0,
                   f"头一趟退 {first_got}、判红那一趟退 {red_got}（{red_verdicts}），下一趟期望 clamp_to_limit 现判、keep_small 复用，实测退 {got}、{verdicts}")

        # 库位置只由被判那棵仓定（文件头「库放在哪」）：git 工作树的 library-directory 打 <common-dir>/singlefs-formal-unit-verdicts；不是 git 工作树的退 2、
        # proof 一项都不判（共用库 library_directory 退 2，它的弄坏开关 library-directory-outside-git-tolerated 由共用库自证证红）
        library_main = os.path.join(work, "library-main")
        os.makedirs(library_main)
        subprocess.run(["git", "init", "-q", library_main], check=True)
        library_copy = make_root("library-copy")
        located = subprocess.run([sys.executable, ENTRY_SCRIPT, "library-directory", library_main], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        outside = subprocess.run([sys.executable, ENTRY_SCRIPT, "proof", library_copy], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        expected_library = os.path.join(library_main, ".git", FORMAL_LIBRARY_DIRECTORY_NAME)
        judge_cell("library-directory-from-judged-tree", located.returncode == EXIT_GREEN and os.path.realpath(located.stdout.strip()) == os.path.realpath(expected_library)
                   and outside.returncode == EXIT_USAGE and "不是 git 工作树" in outside.stderr,
                   f"git 工作树期望打 {expected_library}，实测退 {located.returncode}「{located.stdout.strip()}」；不是 git 工作树的 proof 期望退 {EXIT_USAGE}、"
                   f"stderr 点名，实测退 {outside.returncode}：{(outside.stdout + outside.stderr)[-300:]}")

        green_row = "改坏{0}\tcrates/core/src/kernel.rs\tif value > limit {{ limit }} else {{ value }}\tif value > limit {{ limit }} else {{ value }} // SELFTEST_BROKEN {0}\tproof_errors=1\tlanding=clamp_to_limit"
        survives_row = "改注释{0}\tcrates/core/src/kernel.rs\tif value > limit {{ limit }} else {{ value }}\tif value > limit {{ limit }} else {{ value }} // 只是注释 {0}\tproof_errors=1\tlanding=clamp_to_limit"
        # 三行必红世界改的是同一处原文：原文要恰好命中一次，三行各自改一份副本
        mixed_root = make_root("itemsmixed", must_red=[green_row.format("A"), survives_row.format("B"), green_row.format("C")])
        vacuity_lines = [requires_line for requires_line, _brace, _label in requires_insertion_points(SELFTEST_KERNEL)]
        listed_items = ["toolchain", "proof", "proof:core_sample::kernel::clamp_to_limit", "proof:core_sample::kernel::keep_small", "vacuity:core_sample::kernel::clamp_to_limit#1", "vacuity:core_sample::kernel::keep_small#1",
                        "must-red:改坏A", "must-red:改注释B", "must-red:改坏C"]
        judge_cell("vacuity-points-in-kernel", len(vacuity_lines) == 2, f"自证核里该有 2 处 requires，认出 {vacuity_lines}")
        trace_path = os.path.join(work, "verus-trace.txt")
        open(trace_path, "w").close()
        os.environ["SELFTEST_VERUS_TRACE"] = trace_path
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            listed_exit = list_items(mixed_root)
        os.environ.pop("SELFTEST_VERUS_TRACE", None)
        listed = [line.split("\t")[0] for line in buffer.getvalue().strip().split("\n")]
        judge_cell("list-items", listed_exit == 0 and listed == listed_items, f"期望退 0、逐行 {listed_items}，实测退 {listed_exit}、{listed}")
        judge_cell("list-items-no-verus", open(trace_path).read() == "", f"list-items 起了 Verus：{open(trace_path).read()!r}")
        # 读的一头先关了（接 | head 的样子）：被 SIGPIPE 安静停掉，stderr 一个字都没有
        read_end, write_end = os.pipe()
        os.close(read_end)
        closed_pipe = subprocess.run([sys.executable, ENTRY_SCRIPT, "list-items", mixed_root], stdout=write_end, stderr=subprocess.PIPE, text=True)
        os.close(write_end)
        judge_cell("list-items-closed-pipe", closed_pipe.returncode == -signal.SIGPIPE and closed_pipe.stderr == "",
                   f"期望被 SIGPIPE 停（{-signal.SIGPIPE}）、stderr 空，实测 {closed_pipe.returncode}：{closed_pipe.stderr.strip()[-200:]!r}")
        got, output, verdicts = run_items("must-red-worlds", mixed_root, items=["must-red:改注释B"])
        judge_cell("item-select-one", got == 1 and list(verdicts) == ["must-red:改注释B"], f"--item must-red:改注释B 期望只判这一行（退 1），实测退 {got}、判了 {list(verdicts)}")
        got, output, _verdicts = run_items("must-red-worlds", mixed_root, items=["must-red:没有这一行"])
        judge_cell("item-unknown", got == EXIT_USAGE and "must-red:改坏A" in output, f"点名没有的项期望退 2、列出可点名的项，实测退 {got}：{output.strip()[-300:]}")
        got, output, verdicts = run_items("proof", mixed_root, items=["proof:core_sample::kernel::clamp_to_limit"])
        judge_cell("item-proof-one", got == 0 and list(verdicts) == ["proof:core_sample::kernel::clamp_to_limit"] and "不比之和" in output,
                   f"--item proof:core_sample::kernel::clamp_to_limit 期望只验这一个函数、不比之和（退 0），实测退 {got}、判了 {list(verdicts)}：{output.strip()[-200:]}")
        got, output, _verdicts = run_items("proof", mixed_root, items=["proof:没有这个函数"])
        judge_cell("item-proof-unknown", got == EXIT_USAGE and "proof:core_sample::kernel::keep_small" in output, f"点名没有的函数期望退 2、列出可点名的项，实测退 {got}")
        # 点名单跑一项与它在整格里跑的判定相同（必红世界一绿一红一绿；空真里外层可满足、闭包不可满足）
        _got, _output, whole = run_items("must-red-worlds", mixed_root)
        alone = {item: run_items("must-red-worlds", mixed_root, items=[item])[2].get(item) for item in whole}
        judge_cell("item-alone-equals-whole-must-red", len(whole) == 3 and alone == whole and whole["must-red:改注释B"][0] == "红",
                   f"整格 {whole}，逐项单跑 {alone}")
        closure_root = make_root("itemsclosure", kernel=closure_vacuous)
        _got, _output, whole = run_items("vacuity", closure_root)
        alone = {item: run_items("vacuity", closure_root, items=[item])[2].get(item) for item in whole}
        judge_cell("item-alone-equals-whole-vacuity", len(whole) == 3 and alone == whole and sorted(verdict for verdict, _how in whole.values()) == ["红", "绿", "绿"],
                   f"整格 {whole}，逐项单跑 {alone}")
        # 按条复用：头一趟全现判、记下；第二趟全复用；关了复用全现判；核 crate、工具链变了全现判，原因分栏点名
        store_directory = os.path.join(work, "verdict-library")
        reuse_root = make_root("reuse", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        got, output, _verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory)
        judge_cell("reuse-first-run-records", got == 0 and "现判 3、复用 0（现判的原因：无记录 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        got, output, verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory)
        judge_cell("reuse-second-run-reuses", got == 0 and "现判 0、复用 3" in output and all(how == "复用" for _verdict, how in verdicts.values()),
                   f"退 {got}：{output.strip()[-300:]}")
        for check, count in (("proof", 2), ("vacuity", 2)):
            run_items(check, reuse_root, verdict_library=store_directory)
            got, output, _verdicts = run_items(check, reuse_root, verdict_library=store_directory)
            judge_cell(f"reuse-{check}", got == 0 and f"现判 0、复用 {count}" in output, f"第二趟期望全复用，实测退 {got}：{output.strip()[-300:]}")
        # 判法单元指纹是另一列：输入一样都没变、只换判法单元指纹，整格全现判、原因点名它；再照真的跑一趟记回去，后面几格比的是真判法单元指纹
        got, output, _verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory, judging_units="自证造的另一份判法单元")
        judge_cell("reuse-judging-units-changed", got == 0 and "现判 3、复用 0（现判的原因：判法单元指纹 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        run_items("must-red-worlds", reuse_root, verdict_library=store_directory)
        # N38：被验函数体里只加一行注释，三格一项都不现判（开关 reuse-key-whole-crate 下判错：核 crate 整个目录的原文变了）
        kernel_path = os.path.join(reuse_root, "crates", "core", "src", "kernel.rs")

        def edit_kernel(path, old, new):
            with open(path, encoding="utf-8") as handle:
                kernel_text = handle.read()
            assert kernel_text.count(old) == 1, old
            with open(path + ".new", "w", encoding="utf-8") as handle:
                handle.write(kernel_text.replace(old, new))
            os.replace(path + ".new", path)

        edit_kernel(kernel_path, "    if value > limit { limit } else { value }\n", "    // 函数体里只加一行注释\n    if value > limit { limit } else { value }\n")
        after = {check: run_items(check, reuse_root, verdict_library=store_directory) for check in ("proof", "vacuity", "must-red-worlds")}
        judge_cell("reuse-comment-in-function-body", all(got == 0 for got, _output, _verdicts in after.values())
                   and "现判 0、复用 2" in after["proof"][1] and "现判 0、复用 2" in after["vacuity"][1] and "现判 0、复用 3" in after["must-red-worlds"][1],
                   "期望三格全复用，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-200:]}" for check, (got, output, _verdicts) in after.items()))
        # N39：requires 上面加一个空行（行号挪一行），空真两处都复用（开关 vacuity-key-by-line-number 下判错）
        edit_kernel(kernel_path, "    requires\n        limit > 0,", "\n    requires\n        limit > 0,")
        got, output, _verdicts = run_items("vacuity", reuse_root, verdict_library=store_directory)
        judge_cell("reuse-vacuity-line-moved", got == 0 and "现判 0、复用 2" in output, f"期望空真两处都复用，实测退 {got}：{output.strip()[-300:]}")
        # 改一个函数只重验那一项：clamp_to_limit 的 ensures 改一个字，证明与空真只现判它那一项、keep_small 的复用；必红世界三行改的都在它里面，全现判
        # （开关 reuse-key-without-units 下全复用、reuse-key-whole-crate 下全现判，这一格都判错）
        edit_kernel(kernel_path, "        result <= limit,\n", "        result <= limit + 0,\n")
        after = {check: run_items(check, reuse_root, verdict_library=store_directory) for check in ("proof", "vacuity", "must-red-worlds")}
        judge_cell("reuse-proof-function-only", all(got == 0 for got, _output, _verdicts in after.values())
                   and after["proof"][2].get("proof:core_sample::kernel::clamp_to_limit", ("", ""))[1] == "现判" and after["proof"][2].get("proof:core_sample::kernel::keep_small", ("", ""))[1] == "复用"
                   and after["vacuity"][2].get("vacuity:core_sample::kernel::clamp_to_limit#1", ("", ""))[1] == "现判" and after["vacuity"][2].get("vacuity:core_sample::kernel::keep_small#1", ("", ""))[1] == "复用"
                   and "现判 3、复用 0" in after["must-red-worlds"][1] and "改了 fn clamp_to_limit" in after["proof"][1],
                   "期望只现判 clamp_to_limit 那几项、点名它变了，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-300:]}" for check, (got, output, _verdicts) in after.items()))
        with open(os.path.join(install, "libextra.so"), "w") as handle:
            handle.write("安装目录里多一份库\n")
        try:
            got, output, _verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory)
        finally:
            os.remove(os.path.join(install, "libextra.so"))
        judge_cell("reuse-toolchain-changed", got == 0 and "现判 3、复用 0（现判的原因：工具链 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        # 信任清单进键：这个 crate 的一行换一条差分测试，证明两项都现判、原因写「信任清单 变了」（开关 reuse-key-without-trust-list 下照复用，判错）
        trust_store = os.path.join(work, "verdict-library-trust")
        # 用登记了 cfg 原子的核：样本根里指不到工作区条目的 assume_specification 拒算（那一项照判、不复用），比不出复用
        trust_reuse_root = make_root("reusetrust", kernel=cfg_kernel, trust_rows=[cfg_row])
        first_got, first_output, _verdicts = run_items("proof", trust_reuse_root, verdict_library=trust_store)
        trust_path = os.path.join(trust_reuse_root, TRUST_LIST)
        trust_text = open(trust_path).read()
        open(trust_path, "w").write(trust_text.replace("\tselftest_differential\t", "\tselftest_differential_other\t", 1))
        got, output, _verdicts = run_items("proof", trust_reuse_root, verdict_library=trust_store)
        judge_cell("reuse-trust-list-changed", first_got == 0 and got == 0 and "现判 2、复用 0（现判的原因：信任清单 变了 2）" in output,
                   f"头一趟退 {first_got}，改了信任清单一行之后期望两项都现判，实测退 {got}：{output.strip()[-300:]}")
        # 必红世界一行只改变异名：键里不带名字与文件，照复用（开关 must-red-key-with-name-and-path 下判错）
        renamed_store = os.path.join(work, "verdict-library-renamed")
        renamed_root = make_root("reuserenamed", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        run_items("must-red-worlds", renamed_root, verdict_library=renamed_store)
        table_of_renamed = os.path.join(renamed_root, "crates", "core", "verus-must-red-worlds.tsv")
        edit_kernel(table_of_renamed, "改坏B\t", "改坏乙\t")
        got, output, verdicts = run_items("must-red-worlds", renamed_root, verdict_library=renamed_store)
        judge_cell("reuse-must-red-renamed", got == 0 and "现判 0、复用 3" in output and verdicts.get("must-red:改坏乙", ("", ""))[1] == "复用",
                   f"期望改名那一行照复用，实测退 {got}：{output.strip()[-300:]}")
        # 改一个 spec fn：用到它的两个函数都重验，没用它的那个复用；spec fn 自己不带 decreases、不成项（开关 reuse-key-function-only 下判错：只看函数自己的指纹）
        spec_store = os.path.join(work, "verdict-library-spec")
        spec_root = make_root("reusespec", kernel=SELFTEST_SHARED_SPEC_KERNEL, count=3)
        first_spec = run_items("proof", spec_root, verdict_library=spec_store)
        edit_kernel(os.path.join(spec_root, "crates", "core", "src", "kernel.rs"), "{\n    10\n}", "{\n    11\n}")
        got, output, verdicts = run_items("proof", spec_root, verdict_library=spec_store)
        judge_cell("reuse-spec-function-all-users", first_spec[0] == 0 and got == 0
                   and {item: how for item, (_verdict, how) in verdicts.items()}
                   == {"proof:core_sample::kernel::clamp_to_limit": "复用", "proof:core_sample::kernel::keep_small": "现判", "proof:core_sample::kernel::keep_tiny": "现判"}
                   and "改了 fn small_bound" in output,
                   f"第一趟退 {first_spec[0]}，期望用到 small_bound 的两项现判、点名它变了，clamp_to_limit 复用，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # 计数之和用复用的记录：判绿记下之后把登记数改成 3，两项都复用、之和 2 照比、判红（开关 count-unchecked 下判错）
        sum_store = os.path.join(work, "verdict-library-sum")
        sum_root = make_root("reusesum")
        run_items("proof", sum_root, verdict_library=sum_store)
        edit_kernel(os.path.join(sum_root, REGISTRY), "crate\tcrates/core\t2\n", "crate\tcrates/core\t3\n")
        got, output, verdicts = run_items("proof", sum_root, verdict_library=sum_store)
        judge_cell("proof-sum-from-reused-records", got == 1 and "现判 0、复用 2" in output and "各项 verified 之和 2 与登记的 3 不同" in output,
                   f"期望两项复用、之和 2 与登记的 3 不同判红，实测退 {got}：{output.strip()[-300:]}")
        # ── 证明成哪几项、每项怎么判（check_proof、proof_items、verification_clean；假 Verus 出真 Verus 那样的 JSON）──
        # 假 Verus 自己的形态：--verify-function 下没有 success 键、is-verifying-entire-crate false，整 crate 下两样都有（照真 Verus 的探针）
        items_root = make_root("proofitems", kernel=SELFTEST_ITEMS_KERNEL, count=5)
        crate_of_items = os.path.join(items_root, "crates", "core")
        shapes = {}
        for label, extra in (("按函数", ["--verify-only-module", "kernel", "--verify-function", "count_up"]), ("整 crate", [])):
            ran = subprocess.run([os.path.join(install, "verus"), "--no-cheating", "--rlimit", "7", "--output-json", *extra, "src/lib.rs"],
                                 cwd=crate_of_items, capture_output=True, text=True)
            shapes[label] = json.loads(ran.stdout)["verification-results"] if ran.stdout.strip().startswith("{") else {"读不出": ran.stdout[-200:]}
        judge_cell("fake-verus-real-shape",
                   shapes["按函数"] == {"encountered-error": False, "encountered-vir-error": False, "verified": 2, "errors": 0, "is-verifying-entire-crate": False}
                   and shapes["整 crate"].get("success") is True and shapes["整 crate"].get("is-verifying-entire-crate") is True and shapes["整 crate"].get("verified") == 5,
                   f"假 Verus 的输出不像真 Verus：{shapes}")
        # 假 Verus 点模块的形态照真 Verus（文件头「只验一个函数」一段）：点根找不到子模块 kernel 里的函数，--verify-module 配 --verify-function 退 255、stdout 空
        module_shapes = {}
        for label, extra in (("点根", ["--verify-root"]), ("点 --verify-module", ["--verify-module", "kernel"]), ("点没有的模块", ["--verify-only-module", "nosuch"])):
            ran = subprocess.run([os.path.join(install, "verus"), "--no-cheating", "--rlimit", "7", "--output-json", *extra, "--verify-function", "count_up", "src/lib.rs"],
                                 cwd=crate_of_items, capture_output=True, text=True)
            module_shapes[label] = (ran.returncode, ran.stdout.strip()[:80], ran.stderr.strip().split("\n")[0] if ran.stderr.strip() else "")
        judge_cell("fake-verus-module-shape",
                   module_shapes["点根"][0] == 1 and module_shapes["点根"][2] == "error: could not find function count_up specified by --verify-function"
                   and module_shapes["点 --verify-module"] == (255, "", "Error: When using --verify-function, use --verify-only-module instead of --verify-module")
                   and module_shapes["点没有的模块"][2] == "error: could not find module nosuch specified by --verify-module or --verify-only-module",
                   f"假 Verus 点模块不像真 Verus：{module_shapes}")
        # 核按所在模块点（kernel_module）：根 src/lib.rs 点 --verify-root，src/kernel.rs、src/nested/mod.rs、src/nested/deep.rs 点 --verify-only-module
        # kernel、nested、nested::deep。四份核都进登记表，证明与空真两格跑完，逐次核假 Verus 收到的命令行（开关 submodule-verify-root 下子模块的项一律
        # --verify-root、点不中，这一格判错）
        def lemma_kernel(name, head=""):
            return (head + "use vstd::prelude::*;\nverus! {\n"
                    f"pub proof fn {name}(value: u64)\n    requires\n        value > 0,\n    ensures\n        value >= 1,\n{{\n}}\n}} // verus!\n")

        modules_root = make_root("modules", count=5)
        modules_source = os.path.join(modules_root, "crates", "core", "src")
        os.makedirs(os.path.join(modules_source, "nested"))
        for relative, text in (("lib.rs", lemma_kernel("root_lemma", "pub mod kernel;\npub mod nested;\n")), ("nested/mod.rs", lemma_kernel("nested_lemma", "pub mod deep;\n")),
                               ("nested/deep.rs", lemma_kernel("deep_lemma"))):
            with open(os.path.join(modules_source, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        edit_kernel(os.path.join(modules_root, REGISTRY), "kernel\tcrates/core/src/kernel.rs\n",
                    "".join(f"kernel\tcrates/core/src/{relative}\n" for relative in ("lib.rs", "kernel.rs", "nested/mod.rs", "nested/deep.rs")))
        arguments_path = os.path.join(work, "verus-arguments.txt")
        open(arguments_path, "w").close()
        os.environ["SELFTEST_VERUS_ARGUMENTS"] = arguments_path
        try:
            proof_exit, proof_output, _verdicts = run_items("proof", modules_root)
            vacuity_exit, vacuity_output, _verdicts = run_items("vacuity", modules_root)
        finally:
            os.environ.pop("SELFTEST_VERUS_ARGUMENTS", None)
        want_scopes = {"root_lemma": ["--verify-root"], "clamp_to_limit": ["--verify-only-module", "kernel"], "keep_small": ["--verify-only-module", "kernel"],
                       "nested_lemma": ["--verify-only-module", "nested"], "deep_lemma": ["--verify-only-module", "nested::deep"]}
        seen_scopes = collections.defaultdict(list)
        for line in open(arguments_path, encoding="utf-8").read().split("\n"):
            if not line:
                continue
            given = json.loads(line)
            function = given[given.index("--verify-function") + 1] if "--verify-function" in given else "（整个 crate）"
            seen_scopes[function].append([word for index, word in enumerate(given)
                                          if word in ("--verify-root", "--verify-only-module", "--verify-module")
                                          or (index and given[index - 1] in ("--verify-only-module", "--verify-module"))])
        # 每个函数两次：证明一项、空真一处（keep_small 那一处在闭包上，所属的还是 keep_small）
        judge_cell("module-per-kernel",
                   proof_exit == 0 and "各项 verified 之和 5、0 errors（登记 5）" in proof_output and vacuity_exit == 0 and "5 处 requires" in vacuity_output
                   and dict(seen_scopes) == {**{name: [scope, scope] for name, scope in want_scopes.items()}, "（整个 crate）": [[]]},
                   f"期望证明与空真都退 0、每个函数两次各点 {want_scopes}，实测证明退 {proof_exit}、空真退 {vacuity_exit}、点的是 {dict(seen_scopes)}："
                   f"{proof_output.strip()[-200:]} | {vacuity_output.strip()[-200:]}")
        # 核不在入口所在目录下：认不出模块，开跑之前判红、出路点 kernel_module
        outside_root = make_root("moduleoutside")
        os.makedirs(os.path.join(outside_root, "crates", "core", "proofs"))
        os.replace(os.path.join(outside_root, "crates", "core", "src", "kernel.rs"), os.path.join(outside_root, "crates", "core", "proofs", "kernel.rs"))
        with open(os.path.join(outside_root, "crates", "core", "src", "lib.rs"), "w", encoding="utf-8") as handle:
            handle.write('#[path = "../proofs/kernel.rs"]\npub mod kernel;\n')
        edit_kernel(os.path.join(outside_root, REGISTRY), "kernel\tcrates/core/src/kernel.rs\n", "kernel\tcrates/core/proofs/kernel.rs\n")
        got, output, _verdicts = run_items("proof", outside_root)
        judge_cell("module-unrecognized-red", got == 1 and "核 crates/core/proofs/kernel.rs 认不出是 crates/core 的哪个模块" in output and "kernel_module" in output,
                   f"期望认不出模块判红、出路点 kernel_module，实测退 {got}：{output.strip()[-300:]}")
        got, output, verdicts = run_items("proof", items_root)
        want_items = ["proof:core_sample::kernel::Pair::clone", "proof:core_sample::kernel::sum_to", "proof:core_sample::kernel::lemma_within_itself", "proof:core_sample::kernel::count_up"]
        # spec fn 不带 decreases 的不成项（开关 spec-fn-as-item 下 within 成项、验出 0 条判红，这一格判错）
        judge_cell("proof-spec-fn-not-item", got == 0 and list(verdicts) == want_items and "不成项的 spec fn 1 个" in output,
                   f"期望退 0、逐项 {want_items}、报不成项的 spec fn 1 个，实测退 {got}、{list(verdicts)}：{output.strip()[-300:]}")
        # proof fn 按函数验出 verified 1（JSON 里没有 success 键）判绿（开关 require-success-key-per-function 下判错）
        judge_cell("proof-fn-verified-one-green", verdicts.get("proof:core_sample::kernel::lemma_within_itself") == ("绿", "现判") and "proof:core_sample::kernel::lemma_within_itself：绿" in output,
                   f"期望 proof:core_sample::kernel::lemma_within_itself 现判判绿，实测 {verdicts}：{output.strip()[-300:]}")
        # 按函数验时 is-verifying-entire-crate 报 false 不判红（开关 require-entire-crate-per-function 下判错）
        judge_cell("proof-entire-crate-false-not-red", got == 0 and all(verdict == "绿" for verdict, _how in verdicts.values()) and "is-verifying-entire-crate" not in output,
                   f"期望逐项都绿，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # 带 decreases 的 spec fn 有终止义务，成项（开关 spec-decreases-not-item 下判错）
        judge_cell("proof-spec-decreases-item", verdicts.get("proof:core_sample::kernel::sum_to", ("",))[0] == "绿" and "带 decreases 的 spec fn 1" in output,
                   f"期望 proof:core_sample::kernel::sum_to 成项判绿，实测 {verdicts}：{output.strip()[-300:]}")
        # verus! 里 derive(Clone) 的类型：Verus 验它生成的 <类型>::clone，成项（开关 derived-clone-not-item 下判错）
        judge_cell("proof-derived-clone-item", verdicts.get("proof:core_sample::kernel::Pair::clone", ("",))[0] == "绿" and "derive(Clone) 生成的 clone 1" in output,
                   f"期望 proof:core_sample::kernel::Pair::clone 成项判绿，实测 {verdicts}：{output.strip()[-300:]}")
        # 不同时 derive(Copy)、带 Vec 字段的 derive(Clone) 也成项（真 Verus 报 clone is not a copy、不给规格，函数体照验 1 条；开关 derived-clone-copy-only 下判错：之和 5 与登记的 6 不同）
        holder_kernel = SELFTEST_ITEMS_KERNEL.replace("pub open spec fn within(", "#[derive(Clone, Debug, PartialEq, Eq)]\npub struct Holder {\n    pub bytes: Vec<u8>,\n}\n\npub open spec fn within(", 1)
        holder_exit, holder_output, holder_verdicts = run_items("proof", make_root("proofitemsholder", kernel=holder_kernel, count=6))
        judge_cell("proof-derived-clone-not-copy-item", holder_exit == 0 and holder_verdicts.get("proof:core_sample::kernel::Holder::clone", ("",))[0] == "绿"
                   and "derive(Clone) 生成的 clone 2" in holder_output and "各项 verified 之和 6、0 errors（登记 6）" in holder_output,
                   f"期望 proof:core_sample::kernel::Holder::clone 成项判绿、之和 6，实测退 {holder_exit}、{holder_verdicts}：{holder_output.strip()[-300:]}")
        # 各项之和等于登记的数判绿（真仓的 56 那一格的小样：1 + 1 + 1 + 2 = 5）
        judge_cell("proof-sum-equals-registered-green", got == 0 and "各项 verified 之和 5、0 errors（登记 5）" in output,
                   f"期望之和 5 等于登记的 5、判绿，实测退 {got}：{output.strip()[-300:]}")
        # 之和比登记的少一条判红，点名没成项的 fn 与验出数和静态估不同的项（开关 count-unchecked 下判错）
        short_root = make_root("proofitemsshort", kernel=SELFTEST_ITEMS_KERNEL, count=6)
        got, output, _verdicts = run_items("proof", short_root)
        judge_cell("proof-sum-short-red-named", got == 1 and "各项 verified 之和 5 与登记的 6 不同（少 1 条" in output
                   and "没成项的 verus! fn（不带 decreases 的 spec fn，静态估 0 条）1 个：core_sample::kernel::within" in output and "verified 与静态估不同的项：没有" in output,
                   f"期望之和 5 与登记的 6 不同判红、点名 within，实测退 {got}：{output.strip()[-400:]}")
        # 选中的函数一条义务都没验（verified 0）判红（开关 zero-verified-accepted 下判错）；--item 只点它，不比之和
        zero_root = make_root("proofzero", kernel=SELFTEST_ITEMS_KERNEL.replace("        within(value, value),\n{\n", "        within(value, value),\n{\n    // SELFTEST_ZERO_VERIFIED\n"), count=5)
        got, output, verdicts = run_items("proof", zero_root, items=["proof:core_sample::kernel::lemma_within_itself"])
        judge_cell("proof-zero-verified-red", got == 1 and "Verus 在 --verify-function lemma_within_itself 上一条义务都没验（verified 0" in output,
                   f"期望 verified 0 判红，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # errors 1 判红，红句照原样报 JSON 的几样（开关 nonzero-errors-accepted 下判错）；--item 只点它，不比之和
        broken_root = make_root("prooferrorsone", kernel=SELFTEST_KERNEL.replace("result <= limit,", "result <= limit, // SELFTEST_BROKEN"))
        got, output, verdicts = run_items("proof", broken_root, items=["proof:core_sample::kernel::clamp_to_limit"])
        judge_cell("proof-errors-one-red", got == 1 and "Verus 没验过（exec fn clamp_to_limit）：verified 0、errors 1、encountered-error true、vir 错 false" in output
                   and "postcondition not satisfied" in output,
                   f"期望 errors 1 判红、红句带 JSON 的几样与报错位置，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # list-items 给了判定库：每格报键对不上几条（底稿第二节第 9 条 ⑦）；只读库、不写库、不起 Verus。三格记过之后改 clamp_to_limit 的 ensures：
        # 证明 1 项、空真 1 处、必红世界 3 行（都改在它里面）键对不上，共 5 条（开关 key-mismatch-by-item-only 下全报键相同，判错）
        listing_store = os.path.join(work, "verdict-library-listing")
        listing_root = make_root("listingkeys", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        for check in ("proof", "vacuity", "must-red-worlds"):
            run_items(check, listing_root, verdict_library=listing_store)

        def listing_entries():
            # 三格那三份库在临时服务里的全部条目（库目录只当名字，本地没有文件）：前后一样就是没写库
            return [(kind, entry) for kind in ("proof-function", "vacuity-requires", "must-red-world-row")
                    for entry in VerdictLibrary(listing_store, kind, "").store.scan("")]

        def listing_counts():
            snapshot = listing_entries()
            open(trace_path, "w").close()
            os.environ["SELFTEST_VERUS_TRACE"] = trace_path
            out, err = io.StringIO(), io.StringIO()
            try:
                with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                    listed_exit = list_items(listing_root, listing_store, install)
            finally:
                os.environ.pop("SELFTEST_VERUS_TRACE", None)
            after = listing_entries()
            return listed_exit, err.getvalue(), bool(snapshot) and snapshot == after and open(trace_path).read() == ""

        before_exit, before_text, before_untouched = listing_counts()
        edit_kernel(os.path.join(listing_root, "crates", "core", "src", "kernel.rs"), "        result <= limit,\n", "        result <= limit + 0,\n")
        after_exit, after_text, after_untouched = listing_counts()
        judge_cell("list-items-key-mismatch-count",
                   before_exit == 0 and after_exit == 0 and before_untouched and after_untouched
                   and "verus-proof：键对不上 0 条、无记录 0 条、键相同 2 条、拒算 0 条" in before_text and "键对不上共 0 条" in before_text
                   and "verus-proof：键对不上 1 条、无记录 0 条、键相同 1 条" in after_text and "verus-vacuity：键对不上 1 条、无记录 0 条、键相同 1 条" in after_text
                   and "verus-must-red-worlds：键对不上 3 条、无记录 0 条、键相同 0 条" in after_text and "键对不上共 5 条（3 格比了 7 项" in after_text,
                   f"改之前退 {before_exit}（库与 Verus 没动 {before_untouched}）：{before_text.strip()!r}；改之后退 {after_exit}（没动 {after_untouched}）：{after_text.strip()!r}")
        # list-items --key-mismatches 逐项往 stderr 打 ITEM_KEY 行（门禁 --list-items --with-keys 转成 ITEM_PLAN 的原料）：改 clamp_to_limit 的 ensures 之前 7 项都键相同；
        # 改之后它的证明、空真与三行必红世界 5 项键对不上，keep_small 的证明与空真 2 项键相同（开关 item-key-lines-skipped 下一行都不打，这一格判错）
        def item_key_states(text):
            return dict(match.groups() for match in re.finditer(rf"^{ITEM_KEY_PREFIX} item=(.+) state=(\S+)$", text, re.M))
        before_states, after_states = item_key_states(before_text), item_key_states(after_text)
        judge_cell("list-items-item-key-lines",
                   len(before_states) == 7 and set(before_states.values()) == {"键相同"} and len(after_states) == 7
                   and sorted(after_states.values()).count("键对不上") == 5 and sorted(after_states.values()).count("键相同") == 2
                   and after_states.get("proof:core_sample::kernel::clamp_to_limit") == "键对不上" and after_states.get("proof:core_sample::kernel::keep_small") == "键相同"
                   and all(after_states.get(f"must-red:改坏{name}") == "键对不上" for name in "ABC"),
                   f"改之前 {before_states}，改之后 {after_states}")
        # list-items --key-mismatches 带 --item：只比点名的项，成本随点名项数走：点名 keep_small 的证明一项，只打这一项的 ITEM_KEY、只比了 1 项（"1 格比了 1 项"），
        # 空真与必红世界两格不比；点名两项（一项证明、一项必红世界）比 2 项（开关 key-items-ignored 下照旧比整表 7 项，这一格判错）
        def item_key_listing(named):
            keyed_error = io.StringIO()
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(keyed_error):
                keyed_exit = list_items(listing_root, listing_store, install, named)
            return keyed_exit, keyed_error.getvalue()
        one_exit, one_text = item_key_listing(["proof:core_sample::kernel::keep_small"])
        two_exit, two_text = item_key_listing(["proof:core_sample::kernel::keep_small", "must-red:改坏A"])
        judge_cell("list-items-item-key-with-item",
                   one_exit == 0 and two_exit == 0 and item_key_states(one_text) == {"proof:core_sample::kernel::keep_small": "键相同"}
                   and "（1 格比了 1 项" in one_text
                   and item_key_states(two_text) == {"proof:core_sample::kernel::keep_small": "键相同", "must-red:改坏A": "键对不上"} and "（2 格比了 2 项" in two_text,
                   f"点名一项 {item_key_states(one_text)}：{one_text.strip()[-200:]!r}；点名两项 {item_key_states(two_text)}：{two_text.strip()[-200:]!r}")
        # 逐项挂钟与条数（ITEM_DONE 的原料，环境里的 FORMAL_ITEM_DONE_FILE）：clamp_to_limit 的函数体里一行 SELFTEST_SLEEP_10 让假 Verus 在它身上睡 1 秒，keep_small 没有；
        # 两项同时验（jobs 2），每项记的是自己的挂钟（clamp_to_limit ≥ 0.9、keep_small < 0.6），不是整格的（开关 item-done-whole-cell-seconds 下 keep_small 也 ≥ 0.9，这一格判错）；
        # 整格这一项 proof 另记一行（现判 1、复用 0）；第二趟两项全复用（0 秒、现判 0、复用 1）；有一项判红的那一趟它与 proof 的退出码是 1
        # （开关 item-done-unrecorded 下一行都不记，这一格判错）
        done_root = make_root("item-done")
        edit_kernel(os.path.join(done_root, "crates", "core", "src", "kernel.rs"), "    if value > limit { limit } else { value }\n",
                    "    if value > limit { limit } else { value } // SELFTEST_SLEEP_10\n")
        done_file = os.path.join(work, "item-done.tsv")
        done_store = os.path.join(work, "verdict-library-item-done")

        def item_done_rows(verdict_library, environment=None):
            open(done_file, "w").close()
            run_items("proof", done_root, dict({ITEM_DONE_FILE_VARIABLE: done_file}, **(environment or {})), verdict_library=verdict_library, jobs=2)
            rows = [line.split("\t") for line in open(done_file, encoding="utf-8").read().split("\n") if line]
            return {row[0]: row[1:] for row in rows}, len(rows)
        clamp_item, small_item = "proof:core_sample::kernel::clamp_to_limit", "proof:core_sample::kernel::keep_small"
        first_rows, first_count = item_done_rows(done_store)
        second_rows, second_count = item_done_rows(done_store)
        red_rows, red_count = item_done_rows(fresh_library(), {"SELFTEST_FAIL_MARK": "if value > limit { limit }"})
        judge_cell("item-done-lines",
                   first_count == 3 and set(first_rows) == {clamp_item, small_item, "proof"}
                   and float(first_rows[clamp_item][0]) >= 0.9 and float(first_rows[small_item][0]) < 0.6
                   and all(first_rows[name][1:] == ["1", "0", "0"] for name in first_rows)
                   and second_count == 3 and second_rows[clamp_item] == ["0.0", "0", "1", "0"] and second_rows[small_item] == ["0.0", "0", "1", "0"]
                   and second_rows["proof"][1:] == ["1", "0", "0"]
                   and red_count == 3 and red_rows[clamp_item][3] == "1" and red_rows[small_item][3] == "0" and red_rows["proof"][3] == "1",
                   f"第一趟 {first_rows}，第二趟（全复用）{second_rows}，一项判红那一趟 {red_rows}")
        # 一趟里共用的几样不重建、不串（run_per_crate、list-items --key-mismatches 逐 crate、逐格调用同一批函数）：
        #   单元索引按根建、各 crate 各格共用：list-items 列两个 crate 的项并比三格的键、两个 crate 逐个判的 run_check，各只载入一次 Rust 工作区
        #   （开关 unit-index-not-shared 下每处各建一份，载入多于一次，这一格判错）；
        #   两个 crate 的证明记过之后列项比键，共用组成（工具链、rlimit、crate 起法、信任清单）按 crate 共用：每个 crate 的证明项都是键相同、下一趟 run_check 逐项复用
        #   （开关 key-mismatches-share-across-crates 下后面的 crate 拿到第一个 crate 的共用组成，crate 起法里的包名不同，second_sample::double_it 键对不上）
        multi_store = os.path.join(work, "verdict-library-multi-crate-keys")
        run_items("proof", multi_root, verdict_library=multi_store)
        multi_proof_items = ("proof:core_sample::kernel::clamp_to_limit", "proof:core_sample::kernel::keep_small", "proof:second_sample::double_it")

        def multi_key_listing():
            listing_error = io.StringIO()
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(listing_error):
                listed_exit = list_items(multi_root, multi_store, install)
            return listed_exit, listing_error.getvalue()
        (multi_listed_exit, multi_listed_text), multi_listed_loads = counting_workspace_loads(multi_key_listing)
        multi_states = item_key_states(multi_listed_text)
        (_multi_exit, _multi_output, multi_verdicts), multi_checked_loads = counting_workspace_loads(lambda: run_items("proof", multi_root, verdict_library=multi_store))
        judge_cell("key-mismatches-two-crates-shared",
                   multi_listed_exit == 0 and all(multi_states.get(item) == "键相同" for item in multi_proof_items) and multi_listed_loads == 1
                   and all(multi_verdicts.get(item) == ("绿", "复用") for item in multi_proof_items) and multi_checked_loads == 1,
                   f"两个 crate 记过证明之后列项比键：退 {multi_listed_exit}，证明项的键状态 {[multi_states.get(item) for item in multi_proof_items]}，Rust 工作区载入 {multi_listed_loads} 次；"
                   f"同一份库再判一趟：{[multi_verdicts.get(item) for item in multi_proof_items]}，载入 {multi_checked_loads} 次（期望各 1 次、键相同、复用）")
        # 工具链摘要一趟里共用（toolchain_digest）：同一份缓存下同一个 Verus 安装目录只算一次、取到的与现算的相同，换一个安装目录取到那个目录的摘要
        # （开关 toolchain-digest-shared-across-installs 下共用的那一份只按根存，换了安装目录取到的还是第一个目录的摘要，这一格判错）
        other_install = os.path.join(work, "tools-other", "verus-x86-linux")
        shutil.copytree(install, other_install)
        open(os.path.join(other_install, "z3"), "w").write("#!/bin/sh\n# 另一份 z3\n")

        def toolchain_digest_of(directory, cache):
            return toolchain_digest(green, dict(options, verus_root=directory, toolchain_digests=cache))
        digest_cache = {}
        fresh_digests = (toolchain_digest_of(install, None), toolchain_digest_of(other_install, None))
        cached_digests = (toolchain_digest_of(install, digest_cache), toolchain_digest_of(other_install, digest_cache), toolchain_digest_of(install, digest_cache))
        judge_cell("toolchain-digest-shared",
                   fresh_digests[0] != fresh_digests[1] and cached_digests == (fresh_digests[0], fresh_digests[1], fresh_digests[0]) and len(digest_cache) == 2,
                   f"两个 Verus 安装目录现算的摘要期望不同、共用缓存下取到的与现算的相同、缓存里两项：现算 {fresh_digests}，共用 {cached_digests}，缓存 {len(digest_cache)} 项")
        # 门禁样本档的假工具（write-fake-tools）：写出可执行的 verus、cargo、cargo-verus，version.json 的 commit 抄登记表 release 行（开关 fake-tools-commit-unpinned 下写成全 0，这一格判错）
        fake_tools_directory = os.path.join(work, "fake-tools-written")
        with contextlib.redirect_stdout(io.StringIO()):
            fake_tools_exit = write_fake_tools(fake_tools_directory, make_root("fake-tools-root"))
        fake_version = json.load(open(os.path.join(fake_tools_directory, "version.json")))["verus"] if fake_tools_exit == 0 else {}
        judge_cell("fake-tools-written",
                   fake_tools_exit == 0 and fake_version.get("commit") == commit
                   and all(os.access(os.path.join(fake_tools_directory, tool), os.X_OK) for tool in ("verus", "cargo", "cargo-verus")),
                   f"退 {fake_tools_exit}，version.json {fake_version}（期望 commit {commit}）")
        # 每一行的键里有这一行自己：只判过 A，B 不能拿 A 的判定
        own_store = os.path.join(work, "verdict-library-own")
        own_root = make_root("reuseown", must_red=[green_row.format("A"), survives_row.format("B")])
        run_items("must-red-worlds", own_root, verdict_library=own_store, items=["must-red:改坏A"])
        got, output, verdicts = run_items("must-red-worlds", own_root, verdict_library=own_store)
        judge_cell("reuse-row-own-key", got == 1 and verdicts.get("must-red:改注释B") == ("红", "现判") and "现判 1、复用 1" in output,
                   f"期望 A 复用、B 现判判红，实测退 {got}、{verdicts}")
        # 红的不记：同一份输入第二趟照判、照红
        red_store = os.path.join(work, "verdict-library-red")
        run_items("must-red-worlds", own_root, verdict_library=red_store)
        got, output, verdicts = run_items("must-red-worlds", own_root, verdict_library=red_store)
        judge_cell("reuse-red-not-recorded", got == 1 and verdicts.get("must-red:改注释B") == ("红", "现判"), f"第二趟期望 B 照判照红，实测退 {got}、{verdicts}")
        # 一项判完当场落盘：第一趟 C 那一处被外面 kill -9（不在键里的环境），A、B 已记；第二趟同一条命令只判 C
        resume_store = os.path.join(work, "verdict-library-resume")
        resume_root = make_root("resume", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C SELFTEST_KILLME")])
        first, _output, _verdicts = run_items("must-red-worlds", resume_root, environment={"SELFTEST_KILL_MARK": "SELFTEST_KILLME"}, verdict_library=resume_store)
        got, output, verdicts = run_items("must-red-worlds", resume_root, verdict_library=resume_store)
        judge_cell("reuse-resume-after-kill", first == 1 and got == 0 and "现判 1、复用 2" in output, f"第一趟退 {first}，第二趟退 {got}：{output.strip()[-300:]}")
        # 必红世界表不进共用键：改表里一行的替换文只现判那一行，证明与空真全复用；表头加一行注释一项都不现判；核 crate 里别的文件改了三格全现判
        table_store = os.path.join(work, "verdict-library-table")
        table_root = make_root("reusetable", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        table_cells = ("proof", "vacuity", "must-red-worlds")
        for check in table_cells:
            run_items(check, table_root, verdict_library=table_store)
        table_path = os.path.join(table_root, "crates", "core", "verus-must-red-worlds.tsv")
        with open(table_path, encoding="utf-8") as handle:
            table_text = handle.read()
        with open(table_path, "w", encoding="utf-8") as handle:
            handle.write(table_text.replace("// SELFTEST_BROKEN B", "// SELFTEST_BROKEN B 换了替换文"))
        after = {check: run_items(check, table_root, verdict_library=table_store) for check in table_cells}
        judge_cell("reuse-must-red-table-row-only",
                   all(got == 0 for got, _output, _verdicts in after.values())
                   and "现判 1、复用 2（现判的原因：这一行 变了 1）" in after["must-red-worlds"][1] and after["must-red-worlds"][2].get("must-red:改坏B") == ("绿", "现判")
                   and "现判 0、复用 2" in after["proof"][1] and "现判 0、复用 2" in after["vacuity"][1],
                   "期望必红世界只现判 B、证明与空真全复用，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-200:]}" for check, (got, output, _verdicts) in after.items()))
        with open(table_path, encoding="utf-8") as handle:
            table_text = handle.read()
        with open(table_path, "w", encoding="utf-8") as handle:
            handle.write("# 表头另加的一行注释\n\n" + table_text)
        after = {check: run_items(check, table_root, verdict_library=table_store) for check in table_cells}
        judge_cell("reuse-must-red-table-comment-only",
                   all(got == 0 for got, _output, _verdicts in after.values())
                   and "现判 0、复用 3" in after["must-red-worlds"][1] and "现判 0、复用 2" in after["proof"][1] and "现判 0、复用 2" in after["vacuity"][1],
                   "期望三格全复用，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-200:]}" for check, (got, output, _verdicts) in after.items()))
        with open(os.path.join(table_root, "crates", "core", "src", "lib.rs"), "a", encoding="utf-8") as handle:
            handle.write("\n")
        after = {check: run_items(check, table_root, verdict_library=table_store) for check in table_cells}
        # 核 crate 里别的文件加一个空行：三格一项都不现判（若按核 crate 整个目录取原文则全现判；开关 reuse-key-whole-crate 下判错）
        judge_cell("reuse-crate-other-file-no-cells",
                   all(got == 0 for got, _output, _verdicts in after.values())
                   and "现判 0、复用 3" in after["must-red-worlds"][1] and "现判 0、复用 2" in after["proof"][1] and "现判 0、复用 2" in after["vacuity"][1],
                   "期望三格全复用，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-200:]}" for check, (got, output, _verdicts) in after.items()))
        # 复用三问（格名 reuse-three-questions，判法与规范输出在 scripts/gate/lib_reuse_checks.py 文件头「一」）：三格（证明、空真、必红世界）的项一起算，
        # 头一趟（空库）、同一份输入第二趟、被验函数体里只加一行注释（闭包外：注释不进单元指纹）、clamp_to_limit 的 ensures 改一个字（闭包内）各跑一趟；
        # affected 是闭包含 clamp_to_limit 的五项（证明与空真各一，必红世界三行改的都在它里面），keep_small 那两项要复用。开着复用哨兵（种子 5、哨兵 1 条），
        # 抽中强制现判的那一条按复用算。已有同义的格在这一格里点名、不另留：①＝reuse-first-run-records、reuse-second-run-reuses；
        # ②＝reuse-comment-in-function-body、reuse-crate-other-file-no-cells；③＝reuse-proof-function-only、reuse-spec-function-all-users
        # （开关 reuse-key-whole-crate 下 ② 现判、reuse-key-without-units 下 ③ 少现判、reuse-lookup-skipped 下 ① 全现判，这一格都判错）
        three_store = os.path.join(work, "verdict-library-three-questions")
        three_root = make_root("reusethree", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        three_kernel = os.path.join(three_root, "crates", "core", "src", "kernel.rs")
        sentinel_environment = {reuse_checks.SENTINEL_COUNT_VARIABLE: "1", reuse_checks.SENTINEL_SEED_VARIABLE: "5"}

        def three_round():
            states, exits = {}, []
            for check in ("proof", "vacuity", "must-red-worlds"):
                got, output, _verdicts = run_items(check, three_root, environment=sentinel_environment, verdict_library=three_store)
                exits.append(got)
                states.update(reuse_checks.states_from_output(output))
            return states, exits

        rounds = [three_round()]
        rounds.append(three_round())
        edit_kernel(three_kernel, "    if value > limit { limit } else { value }\n", "    // 函数体里只加一行注释\n    if value > limit { limit } else { value }\n")
        rounds.append(three_round())
        edit_kernel(three_kernel, "        result <= limit,\n", "        result <= limit + 0,\n")
        rounds.append(three_round())
        three_affected = ["proof:core_sample::kernel::clamp_to_limit", "vacuity:core_sample::kernel::clamp_to_limit#1", "must-red:改坏A", "must-red:改坏B", "must-red:改坏C"]
        three_passed, three_problems = reuse_checks.report_three_questions(SCRIPT_KEY, *[states for states, _exits in rounds], three_affected,
                                                                           sentinels=3)
        three_exits = [got for _states, exits in rounds for got in exits]
        judge_cell(reuse_checks.CELL_NAME, three_passed and all(got == 0 for got in three_exits),
                   f"四趟各格退 {three_exits}；{'；'.join(three_problems)}")
        # 复用哨兵（reuse-sentinel，lib_reuse_checks.py 文件头「二」）：上面四趟之后库里三行必红世界都有判绿记录；同一个种子照常跑判绿、抽中的那一条报「强制现判」，
        # 开着弄坏开关 reuse-lookup-skipped（一律不读库）跑，哨兵要判红「复用失灵」（共用库的开关 REUSE_CHECKS_BREAK=rejudged-sentinel-passes 下这一格判错）
        got_green, output_green, _verdicts = run_items("must-red-worlds", three_root, environment=sentinel_environment, verdict_library=three_store)
        skipped_added = "reuse-lookup-skipped" not in BREAK
        BREAK.add("reuse-lookup-skipped")
        try:
            got_red, output_red, _verdicts = run_items("must-red-worlds", three_root, environment=sentinel_environment, verdict_library=three_store)
        finally:
            if skipped_added:
                BREAK.discard("reuse-lookup-skipped")
        judge_cell(reuse_checks.SENTINEL_CELL_NAME, got_green == 0 and "复用哨兵（verus-must-red-worlds）：种子 5" in output_green
                   and "，强制现判（复用哨兵抽中" in output_green and got_red == 1 and "复用失灵：" in output_red,
                   f"照常那一趟退 {got_green}：{output_green.strip()[-300:]}；不读库那一趟退 {got_red}：{output_red.strip()[-300:]}")
        # 一个 trait impl 里两个方法的单元闭包一模一样（样本先核这一点：两个入口的闭包键相同），键里另带「被验入口单元」才各是各的：
        # 一趟一处（jobs=1，width 后判、后写）记下，第二趟两项都复用、各取自己那一份，各项之和 5 等于登记（开关 reuse-key-without-entry-unit 下两项共用一把键，
        # fill 复用到 width 那一份的 1 条，之和 4 判红，这一格判错）
        sibling_store = os.path.join(work, "verdict-library-entry-unit")
        sibling_root = make_root("reuseentryunit", kernel=selftest_kernel_with(SELFTEST_IMPL_SIBLINGS_EXTRA), count=5)
        sibling_index = FunctionUnitIndex(sibling_root)
        sibling_closures = {name: sibling_index.key([f"crates/core/src/kernel.rs::<impl SampleStore for SampleDevice>::{name}"]).key for name in ("fill", "width")}
        sibling_first = run_items("proof", sibling_root, verdict_library=sibling_store, jobs=1)
        sibling_second = run_items("proof", sibling_root, verdict_library=sibling_store, jobs=1)
        judge_cell("reuse-proof-entry-unit-own-key", None not in sibling_closures.values() and len(set(sibling_closures.values())) == 1
                   and sibling_first[0] == 0 and "现判 4、复用 0" in sibling_first[1]
                   and sibling_second[0] == 0 and "现判 0、复用 4" in sibling_second[1] and "各项 verified 之和 5" in sibling_second[1],
                   f"两个方法的闭包键 {sibling_closures}（期望相同）；头一趟退 {sibling_first[0]}：{sibling_first[1].strip()[-200:]}；"
                   f"第二趟期望全复用、之和 5，实测退 {sibling_second[0]}：{sibling_second[1].strip()[-300:]}")
        # 空真同理：两个方法各一处 requires，头一个空真（判红、记红）、后一个判绿；第二趟空真那一处照样现判、判红，另一处复用
        # （开关 reuse-key-without-entry-unit 下两处共用一把键，后写的绿盖掉红，空真那一处复用成绿，这一格判错）
        latch_store = os.path.join(work, "verdict-library-owner-unit")
        latch_root = make_root("reuseownerunit", kernel=selftest_kernel_with(SELFTEST_IMPL_SIBLINGS_VACUITY_EXTRA), count=4)
        latch_first = run_items("vacuity", latch_root, verdict_library=latch_store, jobs=1)
        latch_second = run_items("vacuity", latch_root, verdict_library=latch_store, jobs=1)
        judge_cell("reuse-vacuity-owner-unit-own-key", latch_first[0] == 1 and latch_second[0] == 1
                   and latch_second[2].get("vacuity:core_sample::kernel::SampleLatch::open_gate#1") == ("红", "现判")
                   and latch_second[2].get("vacuity:core_sample::kernel::SampleLatch::close_gate#1") == ("绿", "复用"),
                   f"头一趟退 {latch_first[0]}（期望 1）；第二趟期望空真那一处现判判红、另一处复用，实测退 {latch_second[0]}、{latch_second[2]}：{latch_second[1].strip()[-300:]}")
        # assume_specification 进键、入库、复用（复用三问的形态，判法同 reuse-three-questions，只是不打规范输出那一行）：cargo 那一种 crate 连同路径依赖
        # singlefs-format，两条 assume_specification 各指它的一个常量。头一趟（空库）、同一份输入第二趟（现判 0）、改一个没人指的常量（现判 0）、
        # 改 SLOT_WIDTH 的值（现判的恰好是指它的那一项）各跑一趟
        # （开关 assume-specification-refused 下一律拒算、第二趟照样现判，assume-specification-target-unkeyed 下改 SLOT_WIDTH 照样复用，这一格都判错）
        assume_store = os.path.join(work, "verdict-library-assume-specification")
        assume_kernel = selftest_kernel_with(SELFTEST_ASSUME_TARGETS_EXTRA)

        def assume_root(name, trust_rows=()):
            root = make_root(name, kernel=assume_kernel, count=4, crate_mode="\tcargo", extra_dependency='singlefs-format = { path = "../format" }\n', trust_rows=trust_rows)
            os.makedirs(os.path.join(root, "crates", "format", "src"))
            open(os.path.join(root, "crates", "format", "Cargo.toml"), "w").write('[package]\nname = "singlefs-format"\nedition = "2021"\n')
            open(os.path.join(root, "crates", "format", "src", "lib.rs"), "w").write(SELFTEST_ASSUME_TARGETS_FORMAT)
            return root

        _assume_harvest_exit, assume_harvested = captured_stdout(run_check, "proof", assume_root("assumeharvest"), dict(options, verdict_library=fresh_library()))
        assume_digests = dict(re.findall(r"的 assume_specification (\S+) 没登记进信任清单 \S+（这一处的规格原文 sha256 ([0-9a-f]{64})）", assume_harvested.getvalue()))
        assume_three_root = assume_root("assumethree", trust_rows=[
            f"assume_specification\tcrates/core/src/kernel.rs\t{point}\t{assume_digests.get(point, '0' * 64)}\tselftest_differential\t自证用：登记的可信声明"
            for point in ("singlefs_format::SLOT_WIDTH", "singlefs_format::RING_START")])
        format_path = os.path.join(assume_three_root, "crates", "format", "src", "lib.rs")

        def assume_round():
            got, output, _verdicts = run_items("proof", assume_three_root, verdict_library=assume_store)
            return reuse_checks.states_from_output(output), got, output

        assume_rounds = [assume_round(), assume_round()]
        edit_kernel(format_path, "UNUSED_SAMPLE: u64 = 3;", "UNUSED_SAMPLE: u64 = 4;")
        assume_rounds.append(assume_round())
        edit_kernel(format_path, "SLOT_WIDTH: u64 = 8;", "SLOT_WIDTH: u64 = 9;")
        assume_rounds.append(assume_round())
        assume_affected = ["proof:core_sample::kernel::assume_specification[singlefs_format::SLOT_WIDTH]"]
        assume_passed, _assume_line, assume_problems = reuse_checks.three_questions(SCRIPT_KEY, *[states for states, _got, _output in assume_rounds], assume_affected)
        judge_cell("reuse-assume-specification-three-questions", assume_passed and len(assume_rounds[0][0]) == 4 and all(got == 0 for _states, got, _output in assume_rounds),
                   f"四趟各退 {[got for _states, got, _output in assume_rounds]}、头一趟的项 {assume_rounds[0][0]}；{'；'.join(assume_problems)}；"
                   f"末一趟：{assume_rounds[-1][2].strip()[-300:]}")
        # 指到工作区外（std 的方法）的那一条照旧拒算，拒算行点名是哪一处（开关 assume-specification-refused 下另一条也拒算，上一格判错）
        assume_outside = run_items("proof", make_root("assumeoutside", kernel=const_kernel, count=5, trust_rows=const_rows), verdict_library=os.path.join(work, "verdict-library-assume-outside"))
        judge_cell("reuse-assume-specification-outside-refused", assume_outside[0] == 0
                   and "assume_specification[<[T]>::binary_search]：绿，现判（拒算（被验函数单元闭包）：被验函数单元闭包：refused\tassume-specification-target\tcrates/core/src/kernel.rs:" in assume_outside[1],
                   f"指 std 方法的那一条期望拒算、拒算行点名 assume-specification-target，实测退 {assume_outside[0]}：{assume_outside[1].strip()[-400:]}")
        # 剪枝自证：小输入上开着复用与关掉复用，逐项判定相同；输入没变（全复用）与改了一行（只现判那一行、B 转红）各比一次
        prune_store = os.path.join(work, "verdict-library-prune")
        prune_root = make_root("prune", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        for check in ("proof", "vacuity", "must-red-worlds"):
            run_items(check, prune_root, verdict_library=prune_store)
        on = {check: run_items(check, prune_root, verdict_library=prune_store)[2] for check in ("proof", "vacuity", "must-red-worlds")}
        off = {check: run_items(check, prune_root)[2] for check in ("proof", "vacuity", "must-red-worlds")}
        judge_cell("prune-unchanged-input", {check: {item: verdict for item, (verdict, _how) in items.items()} for check, items in on.items()}
                   == {check: {item: verdict for item, (verdict, _how) in items.items()} for check, items in off.items()} and len(on["must-red-worlds"]) == 3,
                   f"开着复用 {on}，关掉 {off}")
        table_path = os.path.join(prune_root, "crates", "core", "verus-must-red-worlds.tsv")
        with open(table_path, encoding="utf-8") as handle:
            table_text = handle.read()
        with open(table_path + ".new", "w", encoding="utf-8") as handle:
            handle.write(table_text.replace("// SELFTEST_BROKEN B", "// 只是注释 B"))
        os.replace(table_path + ".new", table_path)
        on = run_items("must-red-worlds", prune_root, verdict_library=prune_store)[2]
        off = run_items("must-red-worlds", prune_root)[2]
        judge_cell("prune-after-change", {item: verdict for item, (verdict, _how) in on.items()} == {item: verdict for item, (verdict, _how) in off.items()}
                   and off.get("must-red:改坏B", ("?",))[0] == "红", f"开着复用 {on}，关掉 {off}")
        # tool-versions 换了 z3、设了 VERUS_Z3_PATH 打的那一行都要变（开关 tool-versions-launcher-only 下这一格判错）
        import io
        import contextlib

        def tool_versions_line():
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                tool_versions(green, install)
            return buffer.getvalue()

        cases += 1
        saved_z3_path = os.environ.pop("VERUS_Z3_PATH", None)
        try:
            without_z3 = tool_versions_line()
            with open(os.path.join(install, "z3"), "w", encoding="utf-8") as handle:
                handle.write("# 自证用的假 z3\n")
            with_z3 = tool_versions_line()
            os.environ["VERUS_Z3_PATH"] = zip_path
            with_override = tool_versions_line()
        finally:
            os.environ.pop("VERUS_Z3_PATH", None)
            if saved_z3_path is not None:
                os.environ["VERUS_Z3_PATH"] = saved_z3_path
        if len({without_z3, with_z3, with_override}) != 3:
            wrong.append("tool-versions-z3")
            print(one_line(f"  ✗ tool-versions-z3 这一格判错：换 z3、设 VERUS_Z3_PATH 之后打的行应当各不相同，实测：{without_z3.strip()} | {with_z3.strip()} | {with_override.strip()}"))  # gate-lint:detail
        # 安装目录顶层多一份五样之外的文件（宏库）、VERUS_Z3_PATH 没设与设成空串、裸名 z3 而 PATH 里换一份：打的行都要变
        # （开关 tool-versions-five-files-only、tool-versions-empty-z3-path-as-unset、tool-versions-bare-z3-from-current-directory 下各自那一格判错）
        saved_path = os.environ.get("PATH", "")
        saved_z3_path = os.environ.pop("VERUS_Z3_PATH", None)
        z3_lines = {}
        try:
            before_macros = tool_versions_line()
            with open(os.path.join(install, "libverus_builtin_macros.so"), "wb") as handle:
                handle.write(b"selftest macros")
            z3_lines["install-top-level"] = (before_macros, tool_versions_line())
            unset_line = tool_versions_line()
            os.environ["VERUS_Z3_PATH"] = ""
            z3_lines["z3-path-empty"] = (unset_line, tool_versions_line())
            bare_lines = []
            for variant in ("a", "b"):
                directory = os.path.join(work, f"path-z3-{variant}")
                os.makedirs(directory)
                with open(os.path.join(directory, "z3"), "w", encoding="utf-8") as handle:
                    handle.write(f"#!/bin/sh\n# 自证用的 PATH 里的 z3（{variant}）\n")
                os.chmod(os.path.join(directory, "z3"), 0o755)
                os.environ["VERUS_Z3_PATH"] = "z3"
                os.environ["PATH"] = directory + os.pathsep + saved_path
                bare_lines.append(tool_versions_line())
            z3_lines["z3-bare-name"] = tuple(bare_lines)
        finally:
            os.environ["PATH"] = saved_path
            os.environ.pop("VERUS_Z3_PATH", None)
            if saved_z3_path is not None:
                os.environ["VERUS_Z3_PATH"] = saved_z3_path
        for name, (first, second) in z3_lines.items():
            cases += 1
            if first == second:
                wrong.append(f"tool-versions-{name}")
                print(one_line(f"  ✗ tool-versions-{name} 这一格判错：前后两种环境打的行应当不同，实测都是：{first.strip()}"))  # gate-lint:detail
        # VERUS_ROOT 与 VERUS_USE_RUSTUP：没设、空串、换值、VERUS_ROOT 指的目录换了一份文件，打的行两两不同（开关 tool-versions-verus-environment-ignored 下判错）
        cases += 1
        saved_environment = {variable: os.environ.pop(variable, None) for variable in ("VERUS_ROOT", "VERUS_USE_RUSTUP")}
        verus_root_copy = os.path.join(work, "verus-root-copy")
        os.makedirs(verus_root_copy)
        with open(os.path.join(verus_root_copy, "libverus_builtin_macros.so"), "wb") as handle:
            handle.write(b"selftest macros copy")
        environment_lines = {}
        try:
            environment_lines["两个都没设"] = tool_versions_line()
            os.environ["VERUS_ROOT"] = ""
            environment_lines["VERUS_ROOT 空串"] = tool_versions_line()
            os.environ["VERUS_ROOT"] = verus_root_copy
            environment_lines["VERUS_ROOT 指到一份拷贝"] = tool_versions_line()
            with open(os.path.join(verus_root_copy, "libverus_builtin_macros.so"), "wb") as handle:
                handle.write(b"selftest macros copy, truncated")
            environment_lines["VERUS_ROOT 指的拷贝换了宏库"] = tool_versions_line()
            os.environ.pop("VERUS_ROOT")
            os.environ["VERUS_USE_RUSTUP"] = ""
            environment_lines["VERUS_USE_RUSTUP 空串"] = tool_versions_line()
            os.environ["VERUS_USE_RUSTUP"] = "0"
            environment_lines["VERUS_USE_RUSTUP=0"] = tool_versions_line()
        finally:
            for variable, value in saved_environment.items():
                os.environ.pop(variable, None)
                if value is not None:
                    os.environ[variable] = value
        if len(set(environment_lines.values())) != len(environment_lines):
            wrong.append("tool-versions-verus-environment")
            print(one_line(f"  ✗ tool-versions-verus-environment 这一格判错：{len(environment_lines)} 种环境打的行应当两两不同，实测只有 {len(set(environment_lines.values()))} 种："  # gate-lint:detail
                  + " | ".join(f"{label}：{line.strip()[-120:]}" for label, line in environment_lines.items())))
        # rustup 工具链（C632）：假的 RUSTUP_HOME 里 toolchains/<version.json 的 toolchain> 的 lib/ 与 lib/rustlib/<三元组>/lib/ 各换一个文件、
        # RUSTUP_HOME 指到另一份同内容的拷贝、PATH 里换一份 rustup，打的行两两不同（开关 tool-versions-rustup-ignored 下判错）
        cases += 1
        saved_rustup_home = os.environ.pop("RUSTUP_HOME", None)
        rustup_homes = [os.path.join(work, f"rustup-home-{variant}") for variant in ("a", "b")]
        driver_relative = os.path.join("toolchains", "stable", "lib", "librustc_driver-selftest.so")
        core_relative = os.path.join("toolchains", "stable", "lib", "rustlib", "x86_64-unknown-linux-gnu", "lib", "libcore-selftest.rmeta")
        for home in rustup_homes:
            for relative, content in ((driver_relative, b"selftest driver\x00"), (core_relative, b"selftest core rmeta")):
                os.makedirs(os.path.dirname(os.path.join(home, relative)), exist_ok=True)
                with open(os.path.join(home, relative), "wb") as handle:
                    handle.write(content)
        rustup_lines = {}
        try:
            for variant in ("a", "b"):
                directory = os.path.join(work, f"path-rustup-{variant}")
                os.makedirs(directory)
                with open(os.path.join(directory, "rustup"), "w", encoding="utf-8") as handle:
                    handle.write(f"#!/bin/sh\n# 自证用的 PATH 里的 rustup（{variant}）\n")
                os.chmod(os.path.join(directory, "rustup"), 0o755)
            os.environ["PATH"] = os.path.join(work, "path-rustup-a") + os.pathsep + saved_path
            os.environ["RUSTUP_HOME"] = rustup_homes[0]
            rustup_lines["拷贝 a"] = tool_versions_line()
            with open(os.path.join(rustup_homes[0], driver_relative), "r+b") as handle:
                handle.seek(-1, 2)
                handle.write(b"\xff")
            rustup_lines["拷贝 a 的 librustc_driver 末尾 1 字节改了"] = tool_versions_line()
            with open(os.path.join(rustup_homes[0], core_relative), "ab") as handle:
                handle.write(b" changed")
            rustup_lines["拷贝 a 的 libcore rmeta 也改了"] = tool_versions_line()
            os.environ["RUSTUP_HOME"] = rustup_homes[1]
            rustup_lines["RUSTUP_HOME 指到同内容的拷贝 b"] = tool_versions_line()
            os.environ["PATH"] = os.path.join(work, "path-rustup-b") + os.pathsep + saved_path
            rustup_lines["PATH 里换一份 rustup"] = tool_versions_line()
        finally:
            os.environ["PATH"] = saved_path
            os.environ.pop("RUSTUP_HOME", None)
            if saved_rustup_home is not None:
                os.environ["RUSTUP_HOME"] = saved_rustup_home
        if len(set(rustup_lines.values())) != len(rustup_lines):
            wrong.append("tool-versions-rustup-toolchain")
            print(one_line(f"  ✗ tool-versions-rustup-toolchain 这一格判错：{len(rustup_lines)} 种 rustup 工具链与 rustup 打的行应当两两不同，实测只有 {len(set(rustup_lines.values()))} 种："  # gate-lint:detail
                  + " | ".join(f"{label}：{line.strip()[-160:]}" for label, line in rustup_lines.items())))

    # 默认安装目录不经家目录：不设 --verus-root 与 SINGLEFS_VERUS_ROOT 时落在共享数据根下的 formal-tools（HOME 与 XDG_DATA_HOME 指到别处、SINGLEFS_DATA_ROOT 指到一份假数据根）；
    # --verus-root 盖过它；第五列是绝对路径就是它自己；数据根取不到、第五列写法不认都判红并给出路，不退回家目录（开关 default-install-by-home 下这一格判错）
    cases += 1
    fake_data_root = os.path.join(work, "default-install-data-root")
    saved_install_environment = {name: os.environ.get(name) for name in ("HOME", "XDG_DATA_HOME", "SINGLEFS_VERUS_ROOT", "SINGLEFS_DATA_ROOT", "SINGLEFS_MULTI_HOST_CONFIG")}
    install_registry, odd_install_registry, absolute_install_registry = Registry(), Registry(), Registry()
    install_registry.install_relative = "singlefs/formal-tools/verus-0.9.9/verus-x86-linux"
    odd_install_registry.install_relative = "elsewhere/formal-tools/verus"
    absolute_install_registry.install_relative = os.path.join(work, "stand-in-install")
    try:
        os.environ["HOME"] = os.path.join(work, "default-install-other-home")
        os.environ["XDG_DATA_HOME"] = os.path.join(work, "default-install-xdg")
        os.environ.pop("SINGLEFS_VERUS_ROOT", None)
        os.environ["SINGLEFS_DATA_ROOT"] = fake_data_root
        landed_install = verus_directory(install_registry, None)
        explicit_install = verus_directory(install_registry, os.path.join(work, "explicit-install"))
        absolute_install = verus_directory(absolute_install_registry, None)
        os.environ.pop("SINGLEFS_DATA_ROOT")
        os.environ["SINGLEFS_MULTI_HOST_CONFIG"] = os.path.join(work, "no-such-multi-host.env")
        refusals = {}
        for label, registry_under_test in (("数据根取不到", install_registry), ("第五列不以 singlefs/formal-tools/ 起头", odd_install_registry)):
            try:
                refusals[label] = f"没拒，交 {verus_directory(registry_under_test, None)}"
            except Red as red:
                refusals[label] = "拒" if red.howto else "拒但没给出路"
    finally:
        for name, value in saved_install_environment.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
    if (landed_install != os.path.join(fake_data_root, "formal-tools", "verus-0.9.9", "verus-x86-linux")
            or explicit_install != os.path.join(work, "explicit-install") or absolute_install != absolute_install_registry.install_relative
            or set(refusals.values()) != {"拒"}):
        wrong.append("default-install-not-by-home")
        print(one_line(f"  ✗ default-install-not-by-home 这一格判错：HOME 与 XDG_DATA_HOME 指到别处、SINGLEFS_DATA_ROOT 指到 {fake_data_root} 时，默认安装目录应当是它下面的 "  # gate-lint:detail
                       f"formal-tools/verus-0.9.9/verus-x86-linux，实测 {landed_install}；--verus-root 应当盖过它，实测 {explicit_install}；第五列是绝对路径应当就是它自己，实测 {absolute_install}；"
                       f"取不到数据根与第五列写法不认应当判红，实测 {refusals}"))

    # --threads 的缺省：没设 SINGLEFS_THREAD_CAP 取登记文件第一条非注释行的数，设了取它
    with open(os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "heavy-thread-default"), encoding="utf-8") as handle:
        registered_default = next((int(line.split()[0]) for line in handle if line.strip() and not line.lstrip().startswith("#")), None)
    cases += 1
    unset_threads, capped_threads = default_threads({}), default_threads({"SINGLEFS_THREAD_CAP": "7"})
    if registered_default is None or unset_threads != registered_default or capped_threads != 7:
        wrong.append("threads-default-registered")
        print(one_line(f"  ✗ threads-default-registered 这一格判错：登记 {registered_default}，没设上限得 {unset_threads}，上限 7 得 {capped_threads}"))  # gate-lint:detail

    # 开关轮没检出的那一趟要说出为什么：信号、崩了（回溯末行）、退 0、退 1 没有判错格、别的退出码，各带输出末行
    cases += 1
    miss_samples = {
        "crashed": (1, "  ✓ 前面的格\nTraceback (most recent call last):\n  File \"x.py\", line 1, in <module>\nValueError: substring not found\n"),
        "signalled": (-9, ""),
        "all-green": (0, "  ✓ verus-proof-check 自证 3 格都判对\n"),
        "red-without-cell": (1, "  ✗ 别的一句\n     → 怎么办：x\n"),
        "usage": (2, "  ✗ 用法\n"),
    }
    miss_wants = {"crashed": ("退 1、崩了", "输出末行：ValueError: substring not found"), "signalled": ("被信号 9 停了", "输出末行：（没有输出）"),
                  "all-green": ("退 0、每格都判对",), "red-without-cell": ("退 1 却没有一格报",), "usage": ("退 2；",)}
    miss_details = {name: selftest_break_miss_detail(*sample) for name, sample in miss_samples.items()}
    miss_wrong = {name: miss_details[name] for name, wants in miss_wants.items() if any(want not in miss_details[name] for want in wants)}
    if miss_wrong:
        wrong.append("break-miss-detail")
        print(one_line(f"  ✗ break-miss-detail 这一格判错：开关轮没检出的那一趟要说出为什么，这几样说得不对：{miss_wrong}"))  # gate-lint:detail

    breaks_missed, breaks_proven = [], 0
    if not wrong and not BREAK and not reuse_checks.BREAK and not FORMAL_BREAK:
        items = ("commit-unchecked", "fingerprint-unchecked", "absent-kernel-not-run", "cheating-unchecked", "count-unchecked", 
                     "vacuity-accepts-verified", "vacuity-all-at-once", "mutation-accepts-green", "anchor-unchecked", "rlimit-counts-as-red", "rlimit-unpinned",
                     "rlimit-any-line-voids", "rlimit-attribute-unchecked", "mutation-rlimit-attribute-accepted",
                     "expectation-form-unchecked", "row-segments-unchecked", "landing-unchecked", "landing-by-count", "landing-read-from-unmutated-text",
                     "landing-form-unchecked", "rlimit-counted-in-proof-errors",
                     "tool-versions-launcher-only", "tool-versions-five-files-only", "tool-versions-empty-z3-path-as-unset",
                     "tool-versions-bare-z3-from-current-directory", "tool-versions-verus-environment-ignored", "killed-as-proof-red",
                     "tool-versions-rustup-ignored", "default-install-by-home",
                     "item-filter-ignored", "reuse-key-without-units", "reuse-key-whole-crate", "reuse-key-function-only",
                     "vacuity-key-by-line-number", "must-red-key-with-name-and-path", "reuse-key-without-judging-units",
                     "reuse-key-without-toolchain", "reuse-key-without-row",
                     "reuse-ignores-key",
                     "reuse-records-red", "reuse-records-whole-cell-only", "threads-default-cpu-count", "list-items-pipe-unhandled",
                     "spec-fn-as-item", "spec-decreases-not-item", "derived-clone-not-item", "valued-const-not-item", "assume-specification-not-item",
                     "could-not-compile-counted", "const-fn-heading-unrecognized", "require-success-key-per-function",
                     "require-entire-crate-per-function", "zero-verified-accepted", "nonzero-errors-accepted", "submodule-verify-root", "key-mismatch-by-item-only",
                     "reuse-lookup-skipped", "body-open-past-unit-end", "trait-declaration-by-brace", "trait-impl-lag-matched-by-trait-only",
                     "trait-impl-lag-stale-unchecked", "trait-impl-lag-passed-unnamed", "REUSE_CHECKS_BREAK=rejudged-sentinel-passes",
                     "mir-panic-path-unnormalized", "mir-aliased-items-skipped", "mir-alias-by-prefix", "mir-alias-without-boundary", "derived-clone-copy-only",
                     "whole-entry-past-unit-end", "trait-impl-lag-not-run-unreported", "cfg-entry-past-unit-end", "associated-const-bare-name",
                     "point-names-by-containment", "assert-by-requires-as-insertion-point", "assert-by-requires-by-semicolon-line", "spec-const-not-item",
                     *trait_impl_lag.BREAK_SWITCHES,
                     "outer-memory-narrows-jobs", "group-leftovers-kept",
                     "proof-item-narrows", "red-keeps-green", "reuse-key-without-crate-build", "reuse-key-without-closure-dependencies",
                     "toolchain-key-with-paths",
                     "item-key-lines-skipped", "key-items-ignored", "unit-index-not-shared", "key-mismatches-share-across-crates",
                     "toolchain-digest-shared-across-installs", "slot-directory-per-call", "item-done-unrecorded", "item-done-whole-cell-seconds", "fake-tools-commit-unpinned",
                     "reuse-key-without-entry-unit", "assume-specification-refused", "assume-specification-target-unkeyed",
                     *(f"FORMAL_VERIFICATION_BREAK={item}" for item in FORMAL_BREAK_ITEMS))
        # 各开关的那一趟互不依赖、各用各的临时目录：几趟同时起（每台默认重活线程数趟，按开关条数夹），按开关的次序收退出码、数份数；
        # 写成「变量=项」的是共用库 lib_reuse_checks.py 的开关（复用哨兵被现判不红：reuse-sentinel 那一格判错）
        def run_break(item):
            variable, _equals, value = item.rpartition("=") if "=" in item else ("VERUS_PROOF_CHECK_BREAK", "=", item)
            completed = subprocess.run([sys.executable, ENTRY_SCRIPT, "--selftest"], capture_output=True, text=True,
                                       env=dict(os.environ, **{variable: value}))
            output = completed.stdout + completed.stderr
            caught = selftest_break_caught(completed.returncode, output)
            return item, caught, "" if caught else selftest_break_miss_detail(completed.returncode, output)

        with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(len(items), int(default_heavy_threads(os.environ))))) as pool:
            outcomes = list(pool.map(run_break, items))
        if len(outcomes) != len(items):
            breaks_missed.append(f"开关轮收回 {len(outcomes)} 趟、派出 {len(items)} 趟")
        for item, caught, detail in outcomes:
            if not caught:
                breaks_missed.append(item)
                print(f"    · 开关 {item} 没检出：{detail}")
            else:
                breaks_proven += 1
    if wrong:
        report_red(f"verus-proof-check 自证 {cases} 格里 {len(wrong)} 格判错（逐格见上面以 ✗ 起头、带「这一格判错」的几行）",  # gate-lint:summary
                   "按判错那几格的标签找对应的 check_* 函数修；判据写在文件头")
        return EXIT_RED
    if breaks_missed:
        report_red(f"弄坏开关没有被具体自证格检出：{' '.join(breaks_missed)}", "看上面每个开关「没检出」那一行：崩了的照回溯末行修那一趟撞上的代码，被信号停了的查是谁停的；退 0 或退 1 却没有判错格的，补自证或修开关")
        return EXIT_RED
    suffix = f"；弄坏开关 {breaks_proven} 个，每个都让至少一格判错" if not (BREAK or FORMAL_BREAK or reuse_checks.BREAK) else "（开着弄坏开关，没重跑开关）"
    print(f"  ✓ verus-proof-check 自证 {cases} 格都判对{suffix}")
    return EXIT_GREEN
