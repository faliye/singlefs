#!/usr/bin/env python3
# admission: always 判的是此刻树上的 Cargo.toml 与 .rs，几秒跑完；改了清单或挪了源文件就要现判
# run-condition: none 只读仓里的 Cargo.toml 与 Rust 源码，用 python3 自带的 tomllib，不起 cargo
"""判一棵目录树里每一份 .rs 都有着落：要么是某个 crate 的目标（库、bin、测试、例子、基准、构建脚本），
要么从某个目标经 #[path = "…"]、mod <名>; 或 include!("…") 引得到。两样都不是的，cargo 根本不编它，判红。

用法：
    crate-sources-reachable.py [目录…]     # 不给就判 scripts/experiments 与 crates（相对仓根）
    crate-sources-reachable.py --selftest  # CRATE_SOURCES_REACHABLE_BREAK=<开关> 时必须红在点名的格

判法：
  ① 目录里每一份 Cargo.toml（不进 target/、target-*/）读成一个 crate；只有 [workspace] 没有 [package] 的不算 crate。
  ② 一个 crate 的目标：[lib] 的 path（没写 path 时 src/lib.rs 在就算）；[[bin]]、[[test]]、[[example]]、[[bench]] 写了 path 的取 path，
     没写的照 cargo 的默认位置取；package.build 写了路径取它，没写、也没写 build = false 时 build.rs 在就算；
     autobins、autotests、autoexamples、autobenches 没写成 false 的，照 cargo 自动发现补上 src/main.rs、src/bin/*.rs、src/bin/*/main.rs、
     tests/*.rs、tests/*/main.rs、examples/…、benches/…。
  ③ 从目标出发逐份读：一行 #[path = "X"] 之后（中间可以隔几行属性）的 mod <名>; 引到「这份文件所在目录/X」；
     没有 #[path] 的 mod <名>; 在 crate 根与 mod.rs 里引到「所在目录/<名>.rs」或「所在目录/<名>/mod.rs」，在别的文件 a.rs 里引到「所在目录/a/<名>.rs」或「…/a/<名>/mod.rs」；
     include!("X") 引到「所在目录/X」，接着往下读；include_str!("X")、include_bytes!("X") 也算引到（装置当文本读、写进副本里编的载荷），只记着落、不往下读。
     整行以 // 起头的注释行不看。引到的文件在给的目录之外也照着读（跨 crate 的共用模块）。
  ④ 给的目录里（不进 target/、target-*/）每一份 .rs 不在引到的集合里，逐份列出、退 1。

退出码：0 每一份都有着落；1 有落空的（逐份列出）；2 用法错；77 给的目录里一份 .rs 都没有。

弄坏开关 CRATE_SOURCES_REACHABLE_BREAK=<开关>（只给 --selftest 用，--selftest 之外不认）：
  ignore-path-attributes   不认 #[path = "…"]：path-attribute 那一格该红
  ignore-mod-declarations  不认不带 #[path] 的 mod <名>;：mod-declaration 那一格该红
  ignore-automatic-targets 不认自动发现的目标：automatic-target 那一格该红
  non-mod-rs-like-root     非 mod.rs 文件里的 mod <名>; 也按所在目录找：nested-module 那一格该红
  ignore-text-includes     不认 include_str!、include_bytes!：text-include 那一格该红
  pass-orphans             落空的不报：orphan-reported 那一格该红
"""
import glob
import os
import re
import sys
import tempfile
import tomllib

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

DEFAULT_DIRECTORIES = ("scripts/experiments", "crates")
SKIPPED_DIRECTORY = re.compile(r"^(target|target-.*|\.git)$")
PATH_ATTRIBUTE = re.compile(r'^\s*#\[path\s*=\s*"([^"]+)"\]\s*$')
OTHER_ATTRIBUTE = re.compile(r"^\s*#\[[^\]]*\]\s*$")
MOD_DECLARATION = re.compile(r'^\s*(?:#\[path\s*=\s*"([^"]+)"\]\s*)?(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;')
INCLUDE_MACRO = re.compile(r'\binclude!\(\s*"([^"]+)"\s*\)')
INCLUDE_TEXT_MACRO = re.compile(r'\binclude_(?:str|bytes)!\(\s*"([^"]+)"\s*\)')
BREAKS = ("ignore-path-attributes", "ignore-mod-declarations", "ignore-automatic-targets", "non-mod-rs-like-root", "ignore-text-includes", "pass-orphans")
broken = set()


def walk_files(directory, suffix):
    for current, subdirectories, files in os.walk(directory):
        subdirectories[:] = sorted(name for name in subdirectories if not SKIPPED_DIRECTORY.match(name))
        for name in sorted(files):
            if name.endswith(suffix):
                yield os.path.normpath(os.path.join(current, name))


def crate_targets(manifest):
    """一份 Cargo.toml 的全部目标文件（绝对路径）；只有 [workspace] 的交回空集。"""
    with open(manifest, "rb") as handle:
        data = tomllib.load(handle)
    package = data.get("package")
    if package is None:
        return set()
    root = os.path.dirname(manifest)
    automatic = "ignore-automatic-targets" not in broken
    targets = set()

    def add(relative):
        path = os.path.normpath(os.path.join(root, relative))
        if os.path.isfile(path):
            targets.add(path)

    library = data.get("lib")
    if library is not None and library.get("path"):
        add(library["path"])
    else:
        add("src/lib.rs")
    for kind, default_directory in (("bin", "src/bin"), ("test", "tests"), ("example", "examples"), ("bench", "benches")):
        for entry in data.get(kind, []):
            if entry.get("path"):
                add(entry["path"])
            elif kind == "bin" and entry.get("name") == package.get("name"):
                add("src/main.rs")
            else:
                add(f"{default_directory}/{entry.get('name', '')}.rs")
                add(f"{default_directory}/{entry.get('name', '')}/main.rs")
        if automatic and package.get(f"auto{kind}s", True) is not False:
            if kind == "bin":
                add("src/main.rs")
            for path in sorted(glob.glob(os.path.join(root, default_directory, "*.rs"))):
                targets.add(os.path.normpath(path))
            for path in sorted(glob.glob(os.path.join(root, default_directory, "*", "main.rs"))):
                targets.add(os.path.normpath(path))
    build = package.get("build")
    if isinstance(build, str):
        add(build)
    elif build is not False:
        add("build.rs")
    return targets


def references_of(path, is_crate_root):
    """一份 .rs 经 #[path]、mod、include! 引到的文件，与经 include_str!、include_bytes! 当文本引到的文件（都是绝对路径，不管在不在）。"""
    directory = os.path.dirname(path)
    stem = os.path.splitext(os.path.basename(path))[0]
    treat_as_root = is_crate_root or os.path.basename(path) == "mod.rs" or "non-mod-rs-like-root" in broken
    module_directory = directory if treat_as_root else os.path.join(directory, stem)
    found = []
    as_text = []
    pending_path = None
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if line.lstrip().startswith("//"):
                continue
            attribute = PATH_ATTRIBUTE.match(line)
            if attribute:
                pending_path = attribute.group(1)
                continue
            declaration = MOD_DECLARATION.match(line)
            if declaration:
                explicit = declaration.group(1) or pending_path
                if explicit is not None:
                    if "ignore-path-attributes" not in broken:
                        found.append(os.path.normpath(os.path.join(directory, explicit)))
                elif "ignore-mod-declarations" not in broken:
                    name = declaration.group(2)
                    found.append(os.path.normpath(os.path.join(module_directory, f"{name}.rs")))
                    found.append(os.path.normpath(os.path.join(module_directory, name, "mod.rs")))
                pending_path = None
                continue
            if not OTHER_ATTRIBUTE.match(line):
                pending_path = None
            for included in INCLUDE_MACRO.findall(line):
                found.append(os.path.normpath(os.path.join(directory, included)))
            if "ignore-text-includes" not in broken:
                for included in INCLUDE_TEXT_MACRO.findall(line):
                    as_text.append(os.path.normpath(os.path.join(directory, included)))
    return found, as_text


def unreachable_sources(directories):
    """交回 (判了几份, 目标几份, 引到几份, 落空的清单)。"""
    sources = sorted({path for directory in directories for path in walk_files(directory, ".rs")})
    roots = set()
    for directory in directories:
        for manifest in walk_files(directory, "Cargo.toml"):
            roots |= crate_targets(manifest)
    reached = set(roots)
    queue = sorted(roots)
    while queue:
        path = queue.pop()
        compiled, as_text = references_of(path, path in roots)
        for reference in compiled:
            if reference not in reached and os.path.isfile(reference):
                reached.add(reference)
                queue.append(reference)
        reached.update(reference for reference in as_text if os.path.isfile(reference))
    orphans = [] if "pass-orphans" in broken else [path for path in sources if path not in reached]
    return len(sources), len([path for path in sources if path in roots]), len([path for path in sources if path in reached and path not in roots]), orphans


def judge(directories, base):
    total, as_targets, as_references, orphans = unreachable_sources(directories)
    if total == 0:
        print(f"  ! 本次无对象可判：{'、'.join(os.path.relpath(d, base) for d in directories)} 下一份 .rs 都没有")
        return 77
    if orphans:
        for path in orphans:
            print(f"  ✗ {os.path.relpath(path, base)}：不是任何 crate 的目标，也没有别的源文件经 #[path]、mod 或 include! 引它，cargo 不编它")  # gate-lint:detail
        print(f"  ✗ 判了 {total} 份 .rs，{len(orphans)} 份落空（逐份列在上面）")
        print("     → 怎么办：要编的在它那个 crate 的 Cargo.toml 里登记成 [[bin]]（或 [lib]、[[test]]），或从某个目标用 #[path]、mod 引它；不要了就删掉")
        return 1
    print(f"  ✓ 判了 {total} 份 .rs：{as_targets} 份是 crate 的目标、{as_references} 份经 #[path]、mod 或 include! 引到，没有落空的")
    return 0


def run_selftest():
    cells = []

    def write(root, relative, text):
        path = os.path.join(root, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)

    def build_tree(root, with_orphan):
        write(root, "flat/Cargo.toml", '[package]\nname = "flat"\nversion = "0.1.0"\n\n[lib]\npath = "lib.rs"\n\n[[bin]]\nname = "first"\npath = "first.rs"\n')
        write(root, "flat/lib.rs", "mod shared;\nmod plain;\n")
        write(root, "flat/shared/mod.rs", "pub fn helper() {}\n")
        write(root, "flat/first.rs", '#[cfg(test)]\n#[path = "helper_by_path.rs"]\nmod helper_by_path;\nconst PAYLOAD: &str = include_str!("first/payload.rs");\nfn main() {}\n')
        write(root, "flat/helper_by_path.rs", "\n")
        write(root, "flat/first/payload.rs", "mod never_followed;\n")
        write(root, "flat/plain.rs", "mod inner;\n")
        write(root, "flat/plain/inner.rs", 'include!("included_piece.rs");\n')
        write(root, "flat/plain/included_piece.rs", "\n")
        write(root, "classic/Cargo.toml", '[package]\nname = "classic"\nversion = "0.1.0"\n')
        write(root, "classic/src/bin/automatic.rs", "// mod commented_out;\nfn main() {}\n")
        write(root, "classic/target/release/build/ignored.rs", "\n")
        if with_orphan:
            write(root, "flat/orphan.rs", "fn main() {}\n")

    with tempfile.TemporaryDirectory() as work:
        green, red = os.path.join(work, "green"), os.path.join(work, "red")
        build_tree(green, False)
        build_tree(red, True)
        total, _, _, orphans = unreachable_sources([green])
        cells.append(("all-reachable", total == 9 and orphans == [], f"绿样本 9 份 .rs 都该有着落，实判 {total} 份、落空 {orphans}"))
        _, _, _, orphans = unreachable_sources([red])
        cells.append(("orphan-reported", [os.path.basename(path) for path in orphans] == ["orphan.rs"], f"红样本该只报 orphan.rs，实报 {orphans}"))
        _, _, _, orphans = unreachable_sources([green])
        names = [os.path.relpath(path, green) for path in orphans]
        cells.append(("path-attribute", "flat/helper_by_path.rs" not in names, f"#[path] 引到的 helper_by_path.rs 被报成落空：{names}"))
        cells.append(("mod-declaration", "flat/shared/mod.rs" not in names, f"mod shared; 引到的 shared/mod.rs 被报成落空：{names}"))
        cells.append(("automatic-target", "classic/src/bin/automatic.rs" not in names, f"自动发现的 src/bin/automatic.rs 被报成落空：{names}"))
        cells.append(("nested-module", "flat/plain/inner.rs" not in names, f"plain.rs 里 mod inner; 引到的 plain/inner.rs 被报成落空：{names}"))
        cells.append(("text-include", "flat/first/payload.rs" not in names, f"include_str! 当文本读的 first/payload.rs 被报成落空：{names}"))
    failed = [cell for cell in cells if not cell[1]]
    for name, passed, message in cells:
        if not passed:
            print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    if failed:
        print(f"  ✗ --selftest：{len(cells)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查判法；弄坏开关打开时红在点名的格是对的")
        return 1
    print(f"  ✓ --selftest：{len(cells)} 格都对（全有着落、落空的报出来、#[path]、mod、自动发现的目标、非 mod.rs 里的子模块、当文本读的载荷）")
    return 0


def switch_wheel():
    """逐个打开文件头登记的弄坏开关，各自要红在点名的格。"""
    import subprocess
    expected = {"ignore-path-attributes": "path-attribute", "ignore-mod-declarations": "mod-declaration",
                "ignore-automatic-targets": "automatic-target", "non-mod-rs-like-root": "nested-module", "ignore-text-includes": "text-include", "pass-orphans": "orphan-reported"}
    failures = 0
    for switch, cell in expected.items():
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest-inner"], capture_output=True, text=True,
                                   env=dict(os.environ, CRATE_SOURCES_REACHABLE_BREAK=switch))
        red_cells = re.findall(r"✗ 自证 ([a-z-]+)：", completed.stdout)
        if completed.returncode == 1 and cell in red_cells:
            print(f"  · 开关轮：{switch} 红在「{'、'.join(red_cells)}」，含点名的 {cell}")
        else:
            failures += 1
            print(f"  ✗ 开关轮：{switch} 打开后退 {completed.returncode}、红在「{'、'.join(red_cells) or '（没有）'}」，点名的是 {cell}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 开关轮 {len(expected)} 个开关里 {failures} 个没红在点名的格")
        print("     → 怎么办：改那一处判法，或照实改文件头的开关表")
        return 1
    print(f"  ✓ 开关轮：{len(expected)} 个开关逐个打开，各红在点名的格")
    return 0


def main(arguments):
    if arguments[:1] in (["--selftest"], ["--selftest-inner"]):
        if arguments[0] == "--selftest-inner":
            broken.update(item for item in os.environ.get("CRATE_SOURCES_REACHABLE_BREAK", "").split(",") if item in BREAKS)
            return run_selftest()
        status = run_selftest()
        return status or switch_wheel()
    if any(argument.startswith("-") for argument in arguments):
        print(f"  ✗ 认不得的参数：{' '.join(arguments)}")
        print("     → 怎么办：用法 crate-sources-reachable.py [目录…] 或 --selftest")
        return 2
    base = os.getcwd()
    directories = [os.path.realpath(argument) for argument in arguments] or [os.path.join(REPOSITORY_ROOT, name) for name in DEFAULT_DIRECTORIES]
    missing = [directory for directory in directories if not os.path.isdir(directory)]
    if missing:
        print(f"  ✗ 目录不在：{'、'.join(missing)}")
        print("     → 怎么办：给存在的目录，或不给参数判 scripts/experiments 与 crates")
        return 2
    return judge(directories, base)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
