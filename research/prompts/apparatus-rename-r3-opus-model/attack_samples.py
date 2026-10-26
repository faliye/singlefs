#!/usr/bin/env python3
"""apparatus-rename-r3 攻方样本：在临时 git 仓里造历史，起冻结树里被判的那几份（lib-lag-baseline.py、changed-paths.sh、
lib.sh 的 diff_base、frozen_o3.py、reuse_library.py），逐行打 name=… 的结果。只起 python、git、bash，不起 cargo。

用法：python3 -I attack_samples.py --tree <冻结树> --work <新的空目录>
提交的作者、时刻写死，提交号逐次相同；输出里的临时路径换成 <work>。
"""
import argparse
import hashlib
import importlib.util
import os
import re
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
GIT_ENV = {"GIT_AUTHOR_NAME": "sample", "GIT_AUTHOR_EMAIL": "sample@example.invalid", "GIT_COMMITTER_NAME": "sample",
           "GIT_COMMITTER_EMAIL": "sample@example.invalid", "GIT_AUTHOR_DATE": "2026-01-01T00:00:00Z",
           "GIT_COMMITTER_DATE": "2026-01-01T00:00:00Z", "GIT_CONFIG_NOSYSTEM": "1", "HOME": "/nonexistent-home-for-samples"}
SOURCE = "records/2026-10-23-搬迁.md"
TREE = WORK = None


def environment(extra=None):
    env = {key: value for key, value in os.environ.items() if not key.endswith("_BREAK") and key not in ("GATE_BASE", "GATE_DIFF_BASE")}
    env.update(GIT_ENV)
    env.update(extra or {})
    return env


def run(arguments, cwd, extra=None, data=None):
    shown = subprocess.run(arguments, cwd=cwd, capture_output=True, text=True, env=environment(extra), input=data)
    return shown.returncode, (shown.stdout + shown.stderr).replace(WORK, "<work>")


def git(cwd, *arguments):
    code, out = run(["git", "-c", "init.defaultBranch=master", "-c", "core.quotepath=false", *arguments], cwd)
    if code != 0:
        raise SystemExit(f"git {' '.join(arguments)} 退 {code}：{out}")
    return out.strip()


def write(root, path, text, mode=None):
    full = os.path.join(root, path)
    os.makedirs(os.path.dirname(full) or ".", exist_ok=True)
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(text)
    if mode:
        os.chmod(full, mode)


def new_repo(name):
    root = os.path.join(WORK, name)
    os.makedirs(root)
    git(root, "init", "-q")
    write(root, SOURCE, "# 搬迁记录\n")
    return root


def commit(root, message):
    git(root, "add", "-A")
    git(root, "commit", "-q", "-m", message)
    return git(root, "rev-parse", "HEAD")


def row(old, new):
    return f"{old}\t{new}\t2026-10-23\t{SOURCE}\n"


def library():
    return os.path.join(TREE, ".claude", "gate.d", "lib-lag-baseline.py")


def gate_base(root, extra=None):
    """changed-paths.sh 的 gate_diff_base gate（门禁一侧的基准；GATE_BASE 给了就照它）。"""
    script = 'source "$1" && gate_diff_base gate'
    code, out = run(["bash", "-c", script, "_", os.path.join(TREE, "scripts", "gate", "changed-paths.sh")], root, extra)
    return out.strip()


def staged_base(root):
    """gate.sh --staged 交给里层的 GATE_BASE：上游 lib.sh 的 diff_base（在源仓上算）。"""
    script = 'source "$1" >/dev/null 2>&1; diff_base "$2"'
    code, out = run(["bash", "-c", script, "_", os.path.join(TREE, ".claude", "singlefs-ai-sop", "scripts", "lib.sh"), root], root)
    return out.strip() if code == 0 else f"<diff_base 退 {code}：{out.strip()[:120]}>"


def short(root, revision):
    """基准写成提交说明（「基准 B0」「搬迁 M」）；HEAD 照写 HEAD。"""
    if revision == "HEAD" or not re.fullmatch(r"[0-9a-f]{40}", revision):
        return revision
    return git(root, "log", "-1", "--format=%s", revision).replace(" ", "_")


def path_moves_check(root, base=None, extra=None):
    arguments = ["python3", library(), "path-moves-check", "."] + (["--base", base] if base else [])
    code, out = run(arguments, root, extra)
    lines = [line.strip() for line in out.splitlines() if line.strip()]
    return code, " | ".join(lines)


def changed_paths(root, extra=None):
    script = 'source "$1" && base="$(gate_diff_base gate)" && gate_changed_paths "$base" untracked'
    code, out = run(["bash", "-c", script, "_", os.path.join(TREE, "scripts", "gate", "changed-paths.sh")], root, extra)
    return code, sorted(line for line in out.splitlines() if line)


def added_lines(root, paths, extra=None):
    script = 'source "$1" && shift && base="$(gate_diff_base gate)" && gate_added_lines "$base" "$@"'
    code, out = run(["bash", "-c", script, "_", os.path.join(TREE, "scripts", "gate", "changed-paths.sh"), *paths], root, extra)
    return code, [line for line in out.splitlines() if line]


def show(root, base, path):
    return run(["python3", library(), "show", base, path], root)


def exempt_row_pattern():
    """code-tooling.sh 的 heavy-thread-defaults 格认排除表行的正则（从冻结树那一份里原样取，不抄第二份）。"""
    with open(os.path.join(TREE, ".claude", "gate.d", "code-tooling.sh"), encoding="utf-8") as handle:
        for line in handle:
            if line.startswith("EXEMPT_ROW = re.compile("):
                scope = {"re": re}
                exec(line, scope)
                return scope["EXEMPT_ROW"]
    raise SystemExit("code-tooling.sh 里找不到 EXEMPT_ROW")


def grown_keys(current_text, base_text):
    """照 heavy-thread-defaults 格的比法（code-tooling.sh「grown = …」那几行）：此刻的表里基准那一版没有的「路径:锚串」。"""
    pattern = exempt_row_pattern()
    base_keys = set()
    for line in base_text.split("\n"):
        found = pattern.fullmatch(line)
        if found and not line.lstrip().startswith("#"):
            base_keys.add((found.group("path"), found.group("anchor")))
    grown = []
    for line in current_text.split("\n"):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        found = pattern.fullmatch(line)
        if found and (found.group("path"), found.group("anchor")) not in base_keys:
            grown.append(f"{found.group('path')}:{found.group('anchor')}")
    return grown


def read(root, path):
    with open(os.path.join(root, path), encoding="utf-8") as handle:
        return handle.read()


def emit(name, **fields):
    print(f"name={name} " + " ".join(f"{key}={value}" for key, value in fields.items()))


# ── J1-A：前缀接在 / 后面也换——别的目录下的同名段、单段前缀当词，都被当成「只差搬迁」 ──
def nested_same_name():
    """登记 docs → manual（真搬了 docs/）。同一个改动窗口里，另把没登记的 tools/docs/run.sh 换成 tools/manual/run.sh 的另一份实现，
    排除表那一行、调用方 ci.sh 那一行跟着改；kb/note.md 散文里的「docs」改成「manual」。对照臂：同一段历史不放登记表那一行。"""
    for arm in ("registered", "control_without_row"):
        root = new_repo(f"nested-{arm}")
        write(root, "docs/guide.md", "# 用法\n见 docs/howto.md\n")
        write(root, "docs/howto.md", "# 怎么做\n")
        write(root, "tools/docs/run.sh", '#!/usr/bin/env bash\necho "本机核数 $(nproc)"\n', 0o755)
        write(root, ".claude/heavy-thread-exempt", "# 排除表\ntools/docs/run.sh:$(nproc)  # 只打核数，不是重活\n")
        write(root, "ci.sh", "#!/usr/bin/env bash\nbash tools/docs/run.sh\n", 0o755)
        write(root, "kb/note.md", "# 记\n复跑先读 docs 再起\n")
        base = commit(root, "基准")
        git(root, "mv", "docs", "manual")
        write(root, ".claude/gate.d/path-moves.tsv", row("docs", "manual") if arm == "registered" else "# 空表\n")
        git(root, "rm", "-q", "tools/docs/run.sh")
        write(root, "tools/manual/run.sh", '#!/usr/bin/env bash\nexec make -j"$(nproc)" all\n', 0o755)
        write(root, ".claude/heavy-thread-exempt", "# 排除表\ntools/manual/run.sh:$(nproc)  # 只打核数，不是重活\n")
        write(root, "ci.sh", "#!/usr/bin/env bash\nbash tools/manual/run.sh\n", 0o755)
        write(root, "kb/note.md", "# 记\n复跑先读 manual 再起\n")
        write(root, "manual/guide.md", "# 用法\n见 manual/howto.md\n")
        git(root, "add", "-A")
        gb = gate_base(root)
        code, report = path_moves_check(root)
        emit(f"nested_same_name.{arm}.path_moves_check", rc=code, base=short(root, gb), out=f"「{report}」")
        code, shown = show(root, gb, ".claude/heavy-thread-exempt")
        emit(f"nested_same_name.{arm}.base_table_shown", rc=code, rows=[line for line in shown.splitlines() if line and not line.startswith("#")])
        emit(f"nested_same_name.{arm}.exempt_table_grown", grown=grown_keys(read(root, ".claude/heavy-thread-exempt"), shown))
        code, paths = changed_paths(root)
        emit(f"nested_same_name.{arm}.change_range", rc=code, paths=paths)
        code, lines = added_lines(root, ["ci.sh", "kb/note.md", ".claude/heavy-thread-exempt"])
        emit(f"nested_same_name.{arm}.added_lines", rc=code, lines=lines)
        emit(f"nested_same_name.{arm}.implementation_swapped", base_run_sh=git(root, "rev-parse", f"{base}:tools/docs/run.sh")[:9],
             now_run_sh=git(root, "hash-object", "tools/manual/run.sh")[:9], tools_manual_registered=False)


# ── J1-B：生效窗口跟着 gate 基准走，基准不等于「上一次提交」 ──
def window_after_commit():
    """基准 B0 推到上游；M 提交 git mv old new 与登记（不推）。之后几样合规的改动在三种基准下各判一次：
    upstream（changed-paths.sh 的 gate_diff_base：有上游取 merge-base）、no_upstream（批 6 模拟的那种：没有上游，基准是 HEAD）、
    staged_after_push（M 推上去之后，gate.sh --staged 交给里层的 GATE_BASE 是 lib.sh diff_base 的 HEAD~1）。"""
    remote = os.path.join(WORK, "window-remote.git")
    git(WORK, "init", "-q", "--bare", remote)
    seed = new_repo("window-seed")
    for name in ("a", "b", "c"):
        write(seed, f"old/{name}.sh", f"#!/usr/bin/env bash\necho {name}\n", 0o755)
    write(seed, "ops/launch.sh", "#!/usr/bin/env bash\nbash old/a.sh\n", 0o755)
    commit(seed, "基准 B0")
    git(seed, "remote", "add", "origin", remote)
    git(seed, "push", "-q", "origin", "master")
    git(seed, "branch", "-q", "--set-upstream-to=origin/master")
    git(seed, "mv", "old", "new")
    write(seed, ".claude/gate.d/path-moves.tsv", row("old", "new"))
    write(seed, "ops/launch.sh", "#!/usr/bin/env bash\nbash new/a.sh\n", 0o755)
    commit(seed, "搬迁 M")
    cases = {
        "clean_after_move_commit": lambda root: None,
        "delete_a_moved_file": lambda root: (git(root, "rm", "-q", "new/c.sh")),
        "second_step_as_the_rule_says": lambda root: (write(root, ".claude/gate.d/path-moves.tsv", row("old", "new") + row("new", "newer")),
                                                       git(root, "mv", "new", "newer"),
                                                       write(root, "ops/launch.sh", "#!/usr/bin/env bash\nbash newer/a.sh\n", 0o755),
                                                       git(root, "add", "-A")),
        "reuse_the_old_name": lambda root: (write(root, "old/fresh.sh", "#!/usr/bin/env bash\necho fresh\n", 0o755), git(root, "add", "-A")),
    }
    for case, change in cases.items():
        for arm in ("upstream", "no_upstream", "staged_after_push"):
            root = os.path.join(WORK, f"window-{case}-{arm}")
            git(WORK, "clone", "-q", seed, root)
            git(root, "reset", "-q", "--hard", git(seed, "rev-parse", "HEAD"))
            if arm == "upstream":
                git(root, "remote", "set-url", "origin", remote)
                git(root, "fetch", "-q", "origin")
                git(root, "branch", "-q", "--set-upstream-to=origin/master")
            elif arm == "no_upstream":
                git(root, "remote", "remove", "origin")
            else:
                pushed = os.path.join(WORK, f"window-{case}-pushed.git")
                git(WORK, "clone", "-q", "--bare", remote, pushed)
                git(root, "remote", "set-url", "origin", pushed)
                git(root, "push", "-q", "origin", "master")
                git(root, "fetch", "-q", "origin")
                git(root, "branch", "-q", "--set-upstream-to=origin/master")
            change(root)
            extra = None
            if arm == "staged_after_push":
                extra = {"GATE_BASE": staged_base(root)}
            gb = gate_base(root, extra)
            code, report = path_moves_check(root, extra=extra)
            emit(f"window_after_commit.{case}.{arm}.path_moves_check", rc=code, base=short(root, gb), out=f"「{report}」")
            code, paths = changed_paths(root, extra)
            emit(f"window_after_commit.{case}.{arm}.change_range", rc=code, paths=paths)
            if arm == "staged_after_push":
                code, report = path_moves_check(root)
                emit(f"window_after_commit.{case}.{arm}.same_tree_without_staged_base", rc=code, base=short(root, gate_base(root)), out=f"「{report}」")


# ── J1-C：从来不生效的登记行门禁不核，钉值那一侧照用它倒换 ──
def frozen_o3_module():
    path = os.path.join(TREE, "scripts", "experiment-apparatus", "e171_three_oracle_collusion_after_the_oracle_fixes", "frozen_o3.py")
    specification = importlib.util.spec_from_file_location("frozen_o3_for_samples", path)
    module = importlib.util.module_from_spec(specification)
    sys.modules["frozen_o3_for_samples"] = module
    specification.loader.exec_module(module)
    return module


def reuse_library_pin(path, pinned, root):
    """reuse_library.pinned_sha256_now（E171 修订 11 载入钉住的编排器时核 sha256 的那一处）；载不进交说明。"""
    directory = os.path.join(TREE, "scripts", "experiment-apparatus", "e171_three_oracle_collusion_after_the_oracle_fixes")
    sys.path.insert(0, directory)
    try:
        specification = importlib.util.spec_from_file_location("reuse_library_for_samples", os.path.join(directory, "reuse_library.py"))
        module = importlib.util.module_from_spec(specification)
        sys.modules["reuse_library_for_samples"] = module
        specification.loader.exec_module(module)
        return module.pinned_sha256_now(path, pinned, root) is not None
    except Exception as error:   # 载不进就照实报，不当成认
        return f"载不进：{type(error).__name__}"
    finally:
        sys.path.remove(directory)


def pin_never_effective_row():
    """基准里有两份实现 impl/a、impl/b，钉住的 rig/orchestrator.py 起 impl/a。改动：编排器改起 impl/b，登记表加一行 impl/a → impl/b
    （impl/b 在基准里早就有文件，这一行从来不生效：门禁不按它换算，登记表也不核它）。对照臂：不加那一行。"""
    frozen_o3 = frozen_o3_module()
    for arm in ("never_effective_row", "control_without_row"):
        root = new_repo(f"pin-{arm}")
        write(root, "impl/a/run.py", "print('实现 A：按旧判据判')\n")
        write(root, "impl/b/run.py", "print('实现 B：判据放宽了')\n")
        write(root, "rig/orchestrator.py", 'RUNNER = "impl/a/run.py"\n')
        commit(root, "基准")
        pinned = hashlib.sha256(read(root, "rig/orchestrator.py").encode()).hexdigest()
        write(root, "rig/orchestrator.py", 'RUNNER = "impl/b/run.py"\n')
        write(root, ".claude/gate.d/path-moves.tsv", row("impl/a", "impl/b") if arm == "never_effective_row" else "# 空表\n")
        git(root, "add", "-A")
        code, report = path_moves_check(root)
        emit(f"pin_never_effective_row.{arm}.path_moves_check", rc=code, out=f"「{report}」")
        code, paths = changed_paths(root)
        emit(f"pin_never_effective_row.{arm}.gate_change_range", rc=code, orchestrator_counted="rig/orchestrator.py" in paths)
        moves = frozen_o3.registered_moves(root)
        with open(os.path.join(root, "rig/orchestrator.py"), "rb") as handle:
            data = handle.read()
        versions = moves.contents_before_moves(data)
        emit(f"pin_never_effective_row.{arm}.frozen_o3", rows=[(move.old, move.new) for move in moves.moves],
             matches_pin=any(hashlib.sha256(version).hexdigest() == pinned for version in versions),
             runner_now=data.decode().strip())
        emit(f"pin_never_effective_row.{arm}.reuse_library_pinned_sha256_now", accepted=reuse_library_pin(os.path.join(root, "rig/orchestrator.py"), pinned, root))


# ── J2：按路径管的类只列了五样，别的按路径起作用的地方整份搬进去照剔 ──
def governed_class_gaps():
    """单个文件登记、文件名不变，搬进三处：.claude/gate.d/（gate.sh 把那里的每个 *.sh 当阶段跑）、
    .claude/singlefs-ai-sop/rules/（定义要求整份读的规则目录）、.claude/rules/（列在按路径管的类里，对照）。"""
    root = new_repo("governed-gaps")
    write(root, "notes/check.sh", "#!/usr/bin/env bash\necho 检查\n", 0o755)
    write(root, "drafts/shared.md", "# 草稿\n不许跑重型测试\n")
    write(root, "drafts/local.md", "# 草稿\n不许跑重型测试\n")
    commit(root, "基准")
    moves = [("notes/check.sh", ".claude/gate.d/check.sh"), ("drafts/shared.md", ".claude/singlefs-ai-sop/rules/shared.md"),
             ("drafts/local.md", ".claude/rules/local.md")]
    for old, new in moves:
        os.makedirs(os.path.join(root, os.path.dirname(new)), exist_ok=True)
        git(root, "mv", old, new)
    write(root, ".claude/gate.d/path-moves.tsv", "".join(row(old, new) for old, new in moves))
    git(root, "add", "-A")
    code, report = path_moves_check(root)
    emit("governed_class_gaps.path_moves_check", rc=code, out=f"「{report}」")
    code, paths = changed_paths(root)
    emit("governed_class_gaps.change_range", rc=code, paths=paths)


def main():
    global TREE, WORK
    parser = argparse.ArgumentParser()
    parser.add_argument("--tree", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    TREE = os.path.abspath(arguments.tree)
    WORK = os.path.abspath(arguments.work)
    if os.path.exists(WORK) and os.listdir(WORK):
        raise SystemExit(f"{WORK} 不是空目录")
    os.makedirs(WORK, exist_ok=True)
    nested_same_name()
    window_after_commit()
    pin_never_effective_row()
    governed_class_gaps()
    shutil.rmtree(WORK)
    print("name=done")


if __name__ == "__main__":
    main()
