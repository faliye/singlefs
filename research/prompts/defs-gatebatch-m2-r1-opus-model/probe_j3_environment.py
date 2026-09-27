"""J3 探针：构建环境进指纹（build_environment_lines）之后还漏什么；主工作区与 worktree 算出的指纹什么时候不同。
复跑：python3 probe_j3_environment.py [--admission <admission.py>]
在 admission.py 自证那个小仓上算 crash-case-manifest --toolchain --build-environment；假 cargo、假 rustc 在 PATH 最前面。"""
import os
import shutil
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common  # noqa: E402

ADMISSION = probe_common.option("--admission", probe_common.DEFAULT_ADMISSION)
module = probe_common.load_admission(ADMISSION)
cells = probe_common.Cells("J3 构建环境")
OWN_ROW = "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0 exhaustive=LAYER0\t# 样本"
RUNNER_VARIABLE = "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER"

base = tempfile.mkdtemp(prefix="opus-j3-")
try:
    main = os.path.join(base, "side-a", "main")
    module.build_crash_case_repository(main, [OWN_ROW])
    module.write_text(os.path.join(main, ".gitignore"), "*.out\n")
    tools = os.path.join(base, "tools")
    for name, script in module.FAKE_TOOLCHAIN_SCRIPTS.items():
        module.write_executable(os.path.join(tools, name), script)
    environment = module.environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""),
                                                             "CARGO_HOME": os.path.join(tools, "cargo-home")})

    def fingerprint(root=main, changes=None):
        exit_code, output, messages = module.run_in_environment(
            [sys.executable, ADMISSION, "crash-case-manifest", root, "crash-case:own", os.path.join(base, "manifest.out"),
             "--toolchain", "--build-environment"], dict(environment, **(changes or {})))
        return output.split()[0] if exit_code == 0 and output.split() else f"退 {exit_code}：{output.strip()} {messages.strip()}"

    def executable(path, text):
        module.write_executable(path, "#!/usr/bin/env bash\n" + text)

    # E1 runner 只按路径进：同一个路径上的脚本换了内容（例：给测试二进制加 --skip、改环境变量），指纹不变
    runner = os.path.join(base, "tools-extra", "runner.sh")
    executable(runner, 'exec "$@"\n')
    first = fingerprint(changes={RUNNER_VARIABLE: runner})
    executable(runner, 'exec "$@" --skip every_crash_state\n')
    second = fingerprint(changes={RUNNER_VARIABLE: runner})
    cells.expect("ATTACK", f"E1 {RUNNER_VARIABLE} 指的脚本换了内容 ⇒ 指纹应当变", first != second, f"{first[:16]}… → {second[:16]}…")

    # E2 RUSTC_WRAPPER 同一个路径上换了内容
    wrapper = os.path.join(base, "tools-extra", "wrapper.sh")
    executable(wrapper, 'exec "$@"\n')
    first = fingerprint(changes={"RUSTC_WRAPPER": wrapper})
    executable(wrapper, 'exec "$@" -C overflow-checks=off\n')
    second = fingerprint(changes={"RUSTC_WRAPPER": wrapper})
    cells.expect("ATTACK", "E2 RUSTC_WRAPPER 指的脚本换了内容 ⇒ 指纹应当变", first != second, f"{first[:16]}… → {second[:16]}…")

    # E3 仓根 .cargo/config.toml 的 build.rustc 指一个编译器：那个编译器换了版本，配置文本没变
    alternative = os.path.join(base, "tools-extra", "rustc-alt")
    executable(alternative, '[[ "${1:-}" == -V ]] && echo "rustc 1.80.0 (aaaa 2024-07-01)"\nexit 0\n')
    module.write_text(os.path.join(main, ".cargo", "config.toml"), f'[build]\nrustc = "{alternative}"\n')
    first = fingerprint()
    executable(alternative, '[[ "${1:-}" == -V ]] && echo "rustc 1.99.0 (bbbb 2026-09-01)"\nexit 0\n')
    second = fingerprint()
    cells.expect("ATTACK", "E3 .cargo/config.toml 的 build.rustc 指的编译器换了版本（-V 变了）⇒ 指纹应当变", first != second,
                 f"{first[:16]}… → {second[:16]}…")
    os.remove(os.path.join(main, ".cargo", "config.toml"))

    # E4 对照：同一个编译器经 RUSTC 环境变量给，-V 变了指纹变（第三轮 U4 那一格）
    first = fingerprint(changes={"RUSTC": alternative})
    executable(alternative, '[[ "${1:-}" == -V ]] && echo "rustc 1.99.1 (cccc 2026-09-02)"\nexit 0\n')
    second = fingerprint(changes={"RUSTC": alternative})
    cells.expect("CONTROL", "E4 RUSTC 环境变量指的编译器 -V 变了 ⇒ 指纹变", first != second, f"{first[:16]}… → {second[:16]}…")

    # E5 对照：CARGO_BUILD_JOBS 故意不进（只定并行度，不改编出来的东西）
    cells.expect("CONTROL", "E5 CARGO_BUILD_JOBS=4 与 32 ⇒ 指纹相同",
                 fingerprint(changes={"CARGO_BUILD_JOBS": "4"}) == fingerprint(changes={"CARGO_BUILD_JOBS": "32"}), "")

    # E6–E8 主工作区与 worktree：内容与环境都相同时指纹相同；多一个未跟踪文件、或只有一边的上层目录里有 cargo 配置时不同（假红方向）
    module.run_quietly(["git", "-C", main, "add", "-A"], module.GIT_IDENTITY)
    module.run_quietly(["git", "-C", main, "commit", "-q", "-m", "base"], module.GIT_IDENTITY)
    worktree = os.path.join(base, "side-b", "tree")
    os.makedirs(os.path.dirname(worktree))
    module.run_quietly(["git", "-C", main, "worktree", "add", "-q", "--detach", worktree, "HEAD"], module.GIT_IDENTITY)
    cells.expect("CONTROL", "E6 干净的主工作区与 HEAD 的 worktree，环境相同 ⇒ 指纹相同", fingerprint(main) == fingerprint(worktree),
                 f"{fingerprint(main)[:16]}… / {fingerprint(worktree)[:16]}…")
    module.write_text(os.path.join(main, "crates/pkg/src/scratch_notes.rs"), "// 没暂存的草稿\n")
    cells.expect("INFO", "E7 主工作区 src/ 下多一个没暂存的未跟踪文件 ⇒ 两边指纹不同（主工作区的快档假红）",
                 fingerprint(main) != fingerprint(worktree), f"{fingerprint(main)[:16]}… / {fingerprint(worktree)[:16]}…")
    os.remove(os.path.join(main, "crates/pkg/src/scratch_notes.rs"))
    module.write_text(os.path.join(base, "side-a", ".cargo", "config.toml"), "[build]\nincremental = false\n")
    cells.expect("INFO", "E8 只有主工作区的上一层目录里有 .cargo/config.toml ⇒ 两边指纹不同（cargo 在两边读的配置也确实不同）",
                 fingerprint(main) != fingerprint(worktree), f"{fingerprint(main)[:16]}… / {fingerprint(worktree)[:16]}…")
finally:
    shutil.rmtree(base, ignore_errors=True)
sys.exit(cells.finish())
