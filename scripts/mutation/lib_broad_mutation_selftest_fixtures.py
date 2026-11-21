"""广谱变异驱动（scripts/mutation/broad-mutation.py）自证的夹具：假的一步与假的清单驱动的脚本正文、假源码树、各序号怎么答的计划表、期望的汇总。
只被驱动的自证在开头载入（run_selftest），没有命令行入口；驱动的运行期代码一处都不读这里的名字。

这些名字不写在驱动里：夹具只在自证里载入，驱动运行期读不到；自证的 selftest_fixture_reader_cells 核驱动里没有读它们的地方。
夹具里要与驱动一致的名字（结局名）写字面量，不经驱动的常量取：自证借此对着字面量核驱动的写法。
"""

SELFTEST_TRANSCRIBED_FAILURE_MARK = "〔转述〕"   # 失败行转述的驱动输出里，✗ 换成它（文件头开关表前那一段「转述」）
SELFTEST_DRIVER_FAILURE_LINES_SHOWN = 8          # 失败行末尾整行接上的、这一格起的驱动报的 ✗ 行：去掉数字之后相同的并成一种，至多这么多种，多的只报种数


SELFTEST_STEP_DRIVER = r'''#!/usr/bin/env python3
# 假的一步：照 plan.tsv（序号 <TAB> 步名 <TAB> 动作）办，记一笔调用（序号、步名、树里那份源码此刻是不是被改过）。
import os, sys, time
step, sequence, tree = sys.argv[1], sys.argv[2], sys.argv[3]
work = os.environ["BROAD_MUTATION_SELFTEST_WORK"]
with open(os.path.join(work, "multi-host-configuration.log"), "a") as log:   # 收到的多机配置（覆盖那一步也记）
    log.write(f"{sequence}\t{step}\t{os.environ.get('SINGLEFS_MULTI_HOST_CONFIG', '-')}\n")
if step.startswith("coverage-"):
    # 覆盖那一步：照 <工作目录>/<步名>.json 写这一趟的覆盖（没备就写 null：不按覆盖挑，跑全部）
    import json
    spec = json.load(open(os.environ["BROAD_MUTATION_STEP_SPEC"]))
    prepared = os.path.join(work, step + ".json")
    with open(spec["out"], "w") as out:
        out.write(open(prepared).read() if os.path.exists(prepared) else "null\n")
    with open(os.path.join(work, "coverage-known.log"), "a") as log:
        log.write(f"{step}\t{len(spec.get('known', []))}\n")
    sys.exit(0)
with open(os.path.join(work, "selections.log"), "a") as log:
    log.write(f"{sequence}\t{step}\t{os.environ.get('BROAD_MUTATION_STEP_SELECTION', '')}\n")
mutated = any("MUTATED_" in open(os.path.join(tree, path)).read()
              for path in ("crates/singlefs-core/src/lib.rs", "crates/singlefs-checker/src/lib.rs"))
with open(os.path.join(work, "calls.log"), "a") as log:
    log.write(f"{sequence}\t{step}\t{'mutated' if mutated else 'pristine'}\n")
action = "ok"
for line in open(os.path.join(work, "plan.tsv")):
    fields = line.rstrip("\n").split("\t")
    if fields[:2] == [sequence, step]:
        action = fields[2]
if step.startswith("gpu-"):
    # GPU 那一趟的假步（假 GPU）：记下收到的环境与命令（自证核命令的写法、BINDGEN 的取值），测试步的输出照 libtest 的写法打 Running 与 test result
    with open(os.path.join(work, "gpu-env.log"), "a") as log:
        log.write(f"{sequence}\t{step}\t{os.environ.get('BINDGEN_EXTRA_CLANG_ARGS', '-')}\t{os.environ.get('BROAD_MUTATION_STEP_COMMAND', '-')}\n")
    running = "     Running tests/gpu_pair.rs (target/release/deps/gpu_pair-0123456789abcdef)"
    if step == "gpu-test-verification":
        if action in ("ok", "gpu-pass"):
            print(running)
            print("test gpu_case ... ok")
            print("test result: ok. 3 passed; 0 failed")
            sys.exit(0)
        if action == "gpu-no-adapter":
            print(running)
            print("NOT_RUN_NO_GPU_ADAPTER test=gpu_case reason=没有找到 GPU 适配器（自证的假卡）")
            print("test result: ok. 1 passed; 0 failed")
            sys.exit(0)
        if action == "gpu-fail":
            print(running)
            print("test gpu_case ... FAILED")
            print("failures:")
            print("    gpu_case")
            print("test result: FAILED. 0 passed; 1 failed")
            sys.exit(101)
        if action == "gpu-fail-scrambled":
            # --nocapture 加 --test-threads 1：结果行被测试自己的输出打散，认名字靠收尾的 failures: 名单
            print(running)
            print("test gpu_case ... GPU 对拍：第 3 个状态两边不一致")
            print("FAILED")
            print("failures:")
            print("    gpu_case")
            print("test result: FAILED. 0 passed; 1 failed")
            sys.exit(101)
        if action == "gpu-zero":
            print(running)
            print("test result: ok. 0 passed; 0 failed")
            sys.exit(0)
if action == "fail-nextest":
    print("        FAIL [   0.010s] (3/9) singlefs-format::catching the_case_that_catches_it")
    sys.exit(100)
if action.startswith("fail-nextest:"):
    print("        FAIL [   0.010s] (3/9) " + action.split(":", 1)[1])
    sys.exit(100)
if action == "fail-cargo":
    # 头一条红的是 #[should_panic] 的用例：结果行在名字与「 ... 」之间多一段「 - should panic」，记下的名字不该带它；前面没有 Running 行，二进制认不出
    print("test tests::the_stage_two_case - should panic ... FAILED")
    print("test tests::another_stage_two_case ... FAILED")
    sys.exit(101)
if action == "fail-compile":
    print("error[E0277]: the trait bound `Fake: Default` is not satisfied")
    print("  --> crates/singlefs-core/src/lib.rs:2:5")
    print("error: could not compile `singlefs-core` (lib) due to 1 previous error")
    sys.exit(101)
if action == "hang":
    # 挂住，并且自己再分出一个子进程一起挂着（真链上一步底下还有 cargo 与测试进程）；两个的进程号都记下来
    os.fork()
    with open(os.path.join(work, "hang.pids"), "a") as pids:
        pids.write(f"{os.getpid()}\n")
    open(os.path.join(work, "hanging"), "w").close()
    deadline = time.time() + 60
    while time.time() < deadline and not os.path.exists(os.path.join(work, "release")):
        time.sleep(0.05)
    sys.exit(0)
if action == "leave-holder":
    # 漏下一个进程：起一个脱出这一步的 sleep，工作目录在这一趟的树里，进程号记进 <工作目录>/holder.pid，这一步照绿退出
    # （驱动删这一趟的运行目录之前核开着的文件那一格用：应当不删、报出它）
    import subprocess
    holder = subprocess.Popen(["sleep", "60"], cwd=tree, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    with open(os.path.join(work, "holder.pid"), "w") as out:
        out.write(f"{holder.pid}\n")
    sys.exit(0)
if action.startswith("sleep:"):
    # 慢的一步：先过一条用例（进度行要数到它），再睡那么多秒、绿着退出
    print("        PASS [   0.010s] (1/9) singlefs-format::catching the_case_that_passes", flush=True)
    time.sleep(float(action.split(":")[1]))
    sys.exit(0)
if action in ("cargo-build", "cargo-test"):
    # 假 cargo（每一趟自己的编译目录那一格）：照 cargo 的判法编两个包——fingerprint 不在、或有源文件比上次编出的新，才重编；
    # 测试那一步核编出来的是不是此刻树里的源码，不是就红（过时的产物）
    import hashlib
    target, stale = os.environ["CARGO_TARGET_DIR"], []
    for package in ("singlefs-core", "singlefs-checker"):
        root = os.path.join(tree, "crates", package)
        files = sorted(os.path.join(directory, name) for directory, _names, names in os.walk(root) for name in names)
        content = hashlib.sha256(b"".join(open(path, "rb").read() for path in files)).hexdigest()
        output = os.path.join(target, "debug", ".fingerprint", package + "-0123456789abcdef", "built-from")
        fresh = os.path.isfile(output) and all(os.stat(path).st_mtime_ns <= os.stat(output).st_mtime_ns for path in files)
        if action == "cargo-build" and not fresh:
            os.makedirs(os.path.dirname(output), exist_ok=True)
            with open(output, "w") as handle:
                handle.write(content)
            print(f"   Compiling {package}", flush=True)
            with open(os.path.join(work, "cargo.log"), "a") as log:
                log.write(f"{sequence}\tcompiled\t{package}\n")
        elif action == "cargo-test" and (not os.path.isfile(output) or open(output).read() != content):
            stale.append(package)
    if stale:
        print("        FAIL [   0.010s] (1/1) singlefs-format::catching built_from_stale_" + "_and_".join(stale))
        sys.exit(100)
    sys.exit(0)
if action.startswith("exit:"):
    sys.exit(int(action.split(":")[1]))
sys.exit(0)
'''

SELFTEST_LIST_DRIVER = r'''#!/usr/bin/env python3
# 假的 cargo mutants --list --json --diff：把备好的那一份按包打出来。
import json, os, sys
with open(os.path.join(os.environ["BROAD_MUTATION_SELFTEST_WORK"], "mutants.json")) as handle:
    print(json.dumps([mutant for mutant in json.load(handle) if mutant["package"] == sys.argv[1]]))
'''


def selftest_package(name):
    return f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n'


SELFTEST_CORE = ("pub fn one() -> u32 {\n    1\n}\npub fn two() -> u32 {\n    2\n}\npub fn three() -> u32 {\n    3\n}\n"
                 "pub fn four() -> u32 {\n    4\n}\npub fn five() -> u32 {\n    5\n}\npub fn six() -> u32 {\n    6\n}\n")
SELFTEST_SOURCES = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/singlefs-core", "crates/singlefs-checker", "crates/singlefs-format"]\nresolver = "2"\n',
    "crates/singlefs-core/Cargo.toml": selftest_package("singlefs-core"),
    "crates/singlefs-core/src/lib.rs": SELFTEST_CORE,
    "crates/singlefs-checker/Cargo.toml": selftest_package("singlefs-checker"),
    "crates/singlefs-checker/src/lib.rs": "pub fn seven() -> u32 {\n    7\n}\npub fn eight() -> u32 {\n    8\n}\n",
    # 标了 #[ignore] 的用例 cargo 不跑、驱动不算（登记的崩溃枚举用例那几格另用 验证包）：lib 里模块中的一条、tests 里顶层的一条；包是 singlefs-format，不碰清单的两个包
    "crates/singlefs-format/Cargo.toml": selftest_package("singlefs-format"),
    "crates/singlefs-format/src/lib.rs": '#[cfg(test)]\nmod tests {\n    #[test]\n    fn in_lib_case() {}\n\n'
                                         '    #[test]\n    #[ignore = "要 NVIDIA 卡"]\n    fn on_the_card() {}\n}\n',
    "crates/singlefs-format/tests/catching_cases.rs": '#[test]\nfn catching_one() {}\n\n#[test]\n#[ignore = "要 NVIDIA 卡"]\nfn on_the_card_in_a_target() {}\n',
    # 抓到它的那条测试：nextest 名字「singlefs-format::catching the_case_that_catches_it」，静态认得出、测试键算得出
    "crates/singlefs-format/tests/catching.rs": "#[test]\nfn the_case_that_catches_it() {\n    assert_eq!(1 + 1, 2);\n}\n",
}
# 序号 → 假的一步怎么答（没写的步答 0）：各种结局，两段各有；第二段的两种测试步各答一次；退 104 是认不出的退出码（记驱动错，第 8 条在第二段，状态是只过了第一段）
SELFTEST_PLAN = [("1", "one-test", "fail-nextest"), ("2", "one-build", "fail-compile"), ("3", "one-test", "exit:104"),
                 ("5", "two-test-doc", "fail-cargo"), ("6", "one-test", "exit:104"), ("7", "two-test-verification", "fail-cargo"),
                 ("8", "two-test-verification", "exit:104")]
SELFTEST_EXPECTED = {"1": ("caught", "one"), "2": ("unviable", "one"), "3": ("driver-error", "one"), "4": ("missed", "two"),
                     "5": ("caught", "two"), "6": ("driver-error", "one"), "7": ("caught", "two"), "8": ("driver-error", "two")}
# 第 5 条（文档测试那一步）与第 7 条（验证包那一步）都抓到了、但 libtest 的日志里没有 Running 行，抓到它的测试认不出，两条都算拒算
SELFTEST_EXPECTED_SUMMARY = "total=8 caught=3 missed=1 unviable=1 stage_one_only=1 driver_error=2 not_run=0 refused=2 unlisted_units=0"
SELFTEST_STAGE_TWO_CASE = "tests::the_stage_two_case"   # fail-cargo 那一步（假 cargo test）头一条红的用例名
SELFTEST_STAGE_TWO_STEPS = ["two-test-doc", "two-build-verification", "two-test-verification"]
SELFTEST_BUILD_STEPS = ("one-build", "two-build-verification", "two-build-debug")   # 编译步：基线键拒算、每趟照跑
SELFTEST_NEXTEST_CASE = "singlefs-format::catching the_case_that_catches_it"
SELFTEST_CATCHING_CASE = "singlefs-format::catching_cases catching_one"
# 按条记基线那一格（selftest_baseline_entry_cells）另加的源码：第一段测试步一个测试二进制三条、一条起自己这个可执行文件且不点名测试（Command::new(current_exe())，单元闭包拒算）；
# 验证包的库里两条（一条拒算）、另一个测试目标一条
SELFTEST_ENTRY_BASELINE_FILE = "crates/singlefs-format/tests/per_entry.rs"
SELFTEST_ENTRY_BASELINE_SOURCES = {
    "Cargo.toml": SELFTEST_SOURCES["Cargo.toml"].replace('"crates/singlefs-format"]', '"crates/singlefs-format", "crates/singlefs-verification"]'),
    SELFTEST_ENTRY_BASELINE_FILE: "#[test]\nfn kept_first() {\n    assert_eq!(2 + 2, 4);\n}\n\n#[test]\nfn kept_second() {\n    assert_eq!(3 + 3, 6);\n}\n\n"
                                  "#[test]\nfn refused_third() {\n    let _output = std::process::Command::new(std::env::current_exe().unwrap()).output();\n}\n",
    "crates/singlefs-verification/Cargo.toml": selftest_package("singlefs-verification"),
    "crates/singlefs-verification/src/lib.rs": "pub fn nine() -> u32 {\n    9\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn kept_in_the_library() {\n"
                                               "        assert_eq!(super::nine(), 9);\n    }\n\n    #[test]\n    fn refused_in_the_library() {\n"
                                               "        let _output = std::process::Command::new(std::env::current_exe().unwrap()).output();\n    }\n}\n",
    "crates/singlefs-verification/tests/quick.rs": "#[test]\nfn kept_in_a_test_target() {\n    assert_eq!(1 + 1, 2);\n}\n",
}
# GPU 那一趟的格用的工作树（SELFTEST_SOURCES 加一个 验证包）：库里没有测试；tests/gpu_pair.rs 的测试体里有 #[cfg(feature = "gpu")] 的语句（挂 gpu 特性、不带特性也编），
# 里面一条测试经 judge() 够得着 singlefs-core 的 one、two、four、five（序号 1、2、4、5 的变异）；tests/plain_case.rs 不挂特性，只够得着 three（序号 3，CPU 那一趟的测试碰得到、
# 带 gpu 的这一趟帮不上）；six 与 singlefs-checker 的 seven、eight（序号 6、7、8）没有任何测试够得着。
# 不让 gpu_pair.rs 整个文件挂 #![cfg(feature = "gpu")]：这一格测的是测试体里有挂 gpu 的语句的写法；整个文件挂的那种（不带特性时不编，entry-cfg-inactive）
# CPU 那一趟的覆盖者键（mutation-prune.py 的 coverers_key）跳过这一种，由 mutation-prune.py 自己的自证管，这一格不去测。
SELFTEST_GPU_FILES = {
    "Cargo.toml": SELFTEST_SOURCES["Cargo.toml"].replace('"crates/singlefs-format"]', '"crates/singlefs-format", "crates/singlefs-verification"]'),
    "crates/singlefs-verification/Cargo.toml": selftest_package("singlefs-verification") + '\n[features]\ngpu = []\n\n[dependencies]\nsinglefs-core = { path = "../singlefs-core" }\n',
    "crates/singlefs-verification/src/lib.rs": "pub fn judge() -> u32 {\n    singlefs_core::one() + singlefs_core::two() + singlefs_core::four() + singlefs_core::five()\n}\n",
    "crates/singlefs-verification/tests/gpu_pair.rs": '#[test]\nfn gpu_case() {\n    #[cfg(feature = "gpu")]\n    let extra = 1;\n    #[cfg(not(feature = "gpu"))]\n    let extra = 0;\n'
                                                      '    assert!(singlefs_verification::judge() + extra > 0);\n}\n',
    "crates/singlefs-verification/tests/plain_case.rs": "#[test]\nfn plain_case() {\n    assert!(singlefs_core::three() > 0);\n}\n",
}
SELFTEST_GPU_CASE = "singlefs-verification::gpu_pair gpu_case"      # 抓到它的测试，nextest 的写法（二进制 id 加用例名）


FRESH_BUILD_COMMIT_TIME_NS = 1_790_000_000 * 10 ** 9    # 自证里的「源码的 mtime」：早于自证里编出的每一份产物
