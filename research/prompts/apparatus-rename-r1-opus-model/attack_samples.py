#!/usr/bin/env python3
"""apparatus-rename-r1 攻方腿的样本：在临时 git 仓里造历史，调被判的那几份（从 --tree 给的冻结树里载入），逐行打 name=… 结果。
用法：python3 attack_samples.py --tree <冻结树> --work <新的空目录> [样本名…]
冻结树里要有：.claude/gate.d/lib-lag-baseline.py、.claude/hooks/lib_long_run.py（与同目录的库）、scripts/gate/changed-paths.sh、
.claude/scripts/、.claude/singlefs-ai-sop/scripts/（changed-paths.sh 起的 moved-only-filter 要 preflight）、
scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/（frozen_o3.py、reuse_library.py 与它 import 的）。"""
import argparse
import hashlib
import importlib.util
import os
import subprocess
import sys

sys.dont_write_bytecode = True
ENV = dict(os.environ, GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@t", GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@t",
           GIT_AUTHOR_DATE="2026-10-01T00:00:00Z", GIT_COMMITTER_DATE="2026-10-01T00:00:00Z", LC_ALL="C.UTF-8")
for variable in ("LAG_BASELINE_BREAK", "LIB_LONG_RUN_BREAK", "FROZEN_O3_BREAK", "LIB_CHANGED_PATHS_BREAK", "GATE_BASE"):
    ENV.pop(variable, None)
    os.environ.pop(variable, None)
E171 = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes"
RECORD = "records/move.md"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Repo:
    def __init__(self, root):
        self.root = root
        os.makedirs(root)
        self.git("init", "-q", "-b", "master")

    def git(self, *arguments, check=True):
        done = subprocess.run(["git", "-c", "core.quotepath=false", *arguments], cwd=self.root, env=ENV, capture_output=True, text=True)
        if check and done.returncode != 0:
            raise RuntimeError(f"git {arguments} 退 {done.returncode}：{done.stderr}")
        return done

    def write(self, relative, text, executable=False):
        full = os.path.join(self.root, relative)
        os.makedirs(os.path.dirname(full) or ".", exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(text)
        if executable:
            os.chmod(full, 0o755)

    def read(self, relative):
        with open(os.path.join(self.root, relative), encoding="utf-8") as handle:
            return handle.read()

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-qm", message)

    def register(self, *rows):
        self.write(".claude/gate.d/path-moves.tsv", "# 样本登记表\n" + "".join("\t".join(row) + "\n" for row in rows))

    def added_lines(self, base="HEAD"):
        """与 changed-paths.sh 的 gate_added_lines 同一种取法：--no-renames -U0 的 + 行，加未跟踪文件整份。"""
        diff = self.git("diff", "--no-renames", "--unified=0", "--no-color", "--src-prefix=a/", "--dst-prefix=b/", base, "--").stdout
        entries, path, header = [], None, False
        for line in diff.split("\n"):
            if line.startswith("diff --git "):
                header, path = True, None
            elif header and line.startswith("+++ "):
                target = line[4:]
                path = None if target == "/dev/null" else target[2:]
            elif line.startswith("@@ "):
                header = False
            elif not header and line.startswith("+") and path:
                entries.append((path, line[1:]))
        for untracked in self.git("ls-files", "--others", "--exclude-standard").stdout.split("\n"):
            if untracked:
                entries += [(untracked, text) for text in self.read(untracked).rstrip("\n").split("\n")]
        return entries

    def changed_paths(self, base="HEAD"):
        listed = set(self.git("diff", "--name-only", base, "--").stdout.split()) | set(self.git("diff", "--name-only", "--cached").stdout.split())
        listed |= set(self.git("ls-files", "--others", "--exclude-standard").stdout.split())
        return sorted(listed)


def show(name, **fields):
    print(f"name={name} " + " ".join(f"{key}={value!r}" for key, value in fields.items()))


LIVE_HEADER = ("#!/usr/bin/env bash\n"
               "# 样本：文件头按现算读法写齐，登记在豁免表里\n"
               "# long-run-shards: 按用例切片\n"
               "# long-run-resume: 重起同一条命令只补没做完的片\n"
               "# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存取 host-resources.py --json 现算\n")


def exempt_table(repo, path):
    digest = hashlib.sha256(open(os.path.join(repo.root, path), "rb").read()).hexdigest()
    repo.write(".claude/long-run-live-hosts-exempt", f"# 样本豁免表\n{path}  {digest}  # 样本：文件头按现算写、不能改，只缩不涨\n")


def sample_launder_parent_excuse(tree, work, lib):
    """G1：两个都还在的目录 A、B，一行 A→B 加一行「不在的前缀→A 的上一层」，path_move_problems 放行，A→B 的真改动被当成只差搬迁。"""
    repo = Repo(os.path.join(work, "launder-parent-excuse"))
    repo.write("tools/real-a/run.sh", "echo 走的是 A\n", True)
    repo.write("tools/real-b/run.sh", "echo 走的是 B\n", True)
    repo.write("ops/launch.sh", "bash tools/real-a/run.sh\n", True)
    repo.write("ops/live.sh", LIVE_HEADER + "worker=tools/real-a/run.sh\nbash \"$worker\"\n", True)
    exempt_table(repo, "ops/live.sh")
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("base")
    repo.write("ops/launch.sh", "bash tools/real-b/run.sh\n", True)
    repo.write("ops/live.sh", LIVE_HEADER + "worker=tools/real-b/run.sh\nbash \"$worker\"\n", True)
    exempt_table(repo, "ops/live.sh")
    ran = {path: subprocess.run(["bash", path], cwd=repo.root, capture_output=True, text=True).stdout.strip() for path in ("ops/launch.sh",)}
    repo.register(("tools/real-a", "tools/real-b", "2026-10-23", RECORD))
    count, problems = lib.path_move_problems(repo.root)
    show("launder_parent_excuse.control_one_row", rows=count, problems=[what for what, _howto in problems])
    repo.register(("tools/real-a", "tools/real-b", "2026-10-23", RECORD), ("not-on-disk", "tools", "2026-10-23", RECORD))
    count, problems = lib.path_move_problems(repo.root)
    show("launder_parent_excuse.path_move_problems", rows=count, problems=[what for what, _howto in problems])
    show("launder_parent_excuse.real_change", diff=[line for line in repo.git("diff", "-U0", "HEAD", "--", "ops/launch.sh").stdout.split("\n") if line[:1] in "+-" and line[:3] not in ("+++", "---")],
         launch_now=ran["ops/launch.sh"])
    changed = repo.changed_paths()
    moved, problem = lib.moved_only_paths("HEAD", changed, cwd=repo.root)
    show("launder_parent_excuse.moved_only_paths", changed=changed, moved_only=sorted(moved), problem=problem)
    kept, problem = lib.drop_moved_lines("HEAD", repo.added_lines(), cwd=repo.root)
    show("launder_parent_excuse.drop_moved_lines", kept=kept, problem=problem)
    done = subprocess.run([sys.executable, os.path.join(tree, ".claude/hooks/lib_long_run.py"), "exempt-check", repo.root, "--base", "HEAD"],
                          capture_output=True, text=True, env=ENV)
    show("launder_parent_excuse.exempt_check", rc=done.returncode, out=(done.stdout + done.stderr).strip().replace(repo.root, "<仓>"))


def sample_launder_deleted_directory(tree, work, lib):
    """G1：删掉目录 A，一行 A→B（B 是早就在的另一个目录），别处把 A 换成 B 的命令被当成只差搬迁；A 里的文件本身照算改动。"""
    repo = Repo(os.path.join(work, "launder-deleted-directory"))
    repo.write("tools/real-a/run.sh", "echo 走的是 A\n", True)
    repo.write("tools/real-b/run.sh", "echo 走的是 B，另一份实现\n", True)
    repo.write("ops/launch.sh", "bash tools/real-a/run.sh\n", True)
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("base")
    repo.git("rm", "-q", "-r", "tools/real-a")
    repo.write("ops/launch.sh", "bash tools/real-b/run.sh\n", True)
    repo.register(("tools/real-a", "tools/real-b", "2026-10-23", RECORD))
    count, problems = lib.path_move_problems(repo.root)
    show("launder_deleted_directory.path_move_problems", rows=count, problems=[what for what, _howto in problems])
    changed = repo.changed_paths()
    moved, problem = lib.moved_only_paths("HEAD", changed, cwd=repo.root)
    show("launder_deleted_directory.moved_only_paths", changed=changed, moved_only=sorted(moved), problem=problem)


def sample_history_references(tree, work, lib):
    """G1/G2：指历史的写法里只有「<7–40 位十六进制>:」后面的路径不换；标签、git log -- <路径> 这类换了也算只差搬迁，而换了以后命令的结果变了。"""
    repo = Repo(os.path.join(work, "history-references"))
    repo.write("scripts/experiments/run.sh", "echo 第一版\n", True)
    repo.write("scripts/experiments/README.md", "说明\n")
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("first")
    repo.git("tag", "v0")
    first = repo.git("rev-parse", "--short=8", "HEAD").stdout.strip()
    repo.write("notes-tag.md", "复现：`git show v0:scripts/experiments/run.sh` 打出「echo 第一版」\n")
    repo.write("notes-log.md", "复现：`git log --format=%s -- scripts/experiments/README.md` 打出 first\n")
    repo.write("notes-hex.md", f"复现：`git show {first}:scripts/experiments/run.sh` 打出「echo 第一版」\n")
    repo.commit("base")
    repo.git("mv", "scripts/experiments", "scripts/experiment-apparatus")
    for name in ("notes-tag.md", "notes-log.md", "notes-hex.md"):
        repo.write(name, repo.read(name).replace("scripts/experiments", "scripts/experiment-apparatus"))
    repo.register(("scripts/experiments", "scripts/experiment-apparatus", "2026-10-23", RECORD))
    count, problems = lib.path_move_problems(repo.root)
    changed = repo.changed_paths()
    moved, problem = lib.moved_only_paths("HEAD", changed, cwd=repo.root)
    show("history_references.moved_only_paths", registry_rows=count, registry_problems=problems, changed=changed, moved_only=sorted(moved), problem=problem)
    repo.commit("move")
    for label, command in (("tag_old", ["show", "v0:scripts/experiments/run.sh"]), ("tag_new", ["show", "v0:scripts/experiment-apparatus/run.sh"]),
                           ("log_old", ["log", "--format=%s", "--", "scripts/experiments/README.md"]),
                           ("log_new", ["log", "--format=%s", "--", "scripts/experiment-apparatus/README.md"])):
        done = repo.git(*command, check=False)
        show(f"history_references.after_commit.{label}", command="git " + " ".join(command), rc=done.returncode,
             out=(done.stdout.strip() or done.stderr.strip()).replace("\n", " | "))


def sample_reverse_partial(tree, work, lib, frozen_o3, reuse_library):
    """G2：钉值那一侧（frozen_o3、reuse_library）把此刻的内容倒着换回去再比；只换了一半路径的文件倒换之后也等于钉住的那一版，共用库正着换判它不同。"""
    repo = Repo(os.path.join(work, "reverse-partial"))
    pinned_text = "a=scripts/experiments/one.sh\nb=scripts/experiments/two.sh\n"
    repo.write("scripts/experiment-apparatus/pinned.py", pinned_text)
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("base")
    pinned = hashlib.sha256(pinned_text.encode()).hexdigest()
    repo.register(("scripts/experiments", "scripts/experiment-apparatus", "2026-10-23", RECORD))
    moves = frozen_o3.registered_moves(repo.root)
    for label, text in (("full", "a=scripts/experiment-apparatus/one.sh\nb=scripts/experiment-apparatus/two.sh\n"),
                        ("partial", "a=scripts/experiment-apparatus/one.sh\nb=scripts/experiments/two.sh\n")):
        repo.write("scripts/experiment-apparatus/pinned.py", text)
        path = os.path.join(repo.root, "scripts/experiment-apparatus/pinned.py")
        reverse = frozen_o3.sha256_before_moves(path, moves)
        accepted_by_freeze = frozen_o3.matches_the_freeze(pinned, hashlib.sha256(text.encode()).hexdigest(), reverse, None)
        accepted_by_pin = reuse_library.pinned_sha256_now(path, pinned, repo.root) is not None
        forward_equal = lib.rewrite_moved_bytes(pinned_text.encode(), lib.path_moves(repo.root)[0]) == text.encode()
        moved, problem = lib.moved_only_paths("HEAD", ["scripts/experiment-apparatus/pinned.py"], cwd=repo.root)
        show(f"reverse_partial.{label}", frozen_o3_same=accepted_by_freeze, reuse_library_pin_accepted=accepted_by_pin,
             lib_forward_equal=forward_equal, lib_moved_only=bool(moved), problem=problem)


def sample_chain_midway(tree, work, lib, frozen_o3, reuse_library):
    """G2：两行串起来的搬迁（tools/a→tools/b→tools/c），钉在两次搬迁之间（内容写 tools/b）；frozen_o3 只整串倒换，判不同；reuse_library 逐次倒换、共用库正着换都判同。"""
    repo = Repo(os.path.join(work, "chain-midway"))
    pinned_text = "x=tools/b/run.sh\n"
    repo.write("tools/c/run.sh", "echo c\n", True)
    repo.write("pinned.py", pinned_text)
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("base")
    repo.register(("tools/a", "tools/b", "2026-10-20", RECORD), ("tools/b", "tools/c", "2026-10-23", RECORD))
    count, problems = lib.path_move_problems(repo.root)
    now_text = "x=tools/c/run.sh\n"
    repo.write("pinned.py", now_text)
    path = os.path.join(repo.root, "pinned.py")
    pinned = hashlib.sha256(pinned_text.encode()).hexdigest()
    moves = frozen_o3.registered_moves(repo.root)
    reverse_text = moves.content_before_moves(now_text.encode()).decode()
    freeze_same = frozen_o3.matches_the_freeze(pinned, hashlib.sha256(now_text.encode()).hexdigest(), frozen_o3.sha256_before_moves(path, moves), None)
    pin_accepted = reuse_library.pinned_sha256_now(path, pinned, repo.root) is not None
    moved, problem = lib.moved_only_paths("HEAD", ["pinned.py"], cwd=repo.root)
    show("chain_midway", registry_rows=count, registry_problems=problems, frozen_o3_reverse=reverse_text, frozen_o3_same=freeze_same,
         reuse_library_pin_accepted=pin_accepted, lib_moved_only=bool(moved), problem=problem)


def sample_lines_relocated(tree, work, lib):
    """G1：drop_moved_lines 按次数抵、不按位置抵。原地改的页留着旧行、在别的节加一行换了路径的同文行，这一行不算新增；
    整份搬了家的页，行挪到别的节也一行都不算新增。"""
    repo = Repo(os.path.join(work, "lines-relocated"))
    repo.write("scripts/experiments/x.sh", "echo x\n", True)
    repo.write("kb/page.md", "## 正文\n结论甲\n## 历史\n- 旧说法见 scripts/experiments/x.sh\n")
    repo.write("docs/old/page.md", "## 正文\n结论乙\n## 历史\n- 旧说法见 scripts/experiments/x.sh\n")
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("base")
    repo.git("mv", "scripts/experiments", "scripts/experiment-apparatus")
    repo.git("mv", "docs/old", "docs/new")
    repo.write("kb/page.md", "## 正文\n结论甲\n- 旧说法见 scripts/experiment-apparatus/x.sh\n## 历史\n- 旧说法见 scripts/experiments/x.sh\n")
    repo.write("docs/new/page.md", "## 正文\n- 旧说法见 scripts/experiment-apparatus/x.sh\n## 历史\n结论乙\n")
    repo.register(("scripts/experiments", "scripts/experiment-apparatus", "2026-10-23", RECORD), ("docs/old", "docs/new", "2026-10-23", RECORD))
    count, problems = lib.path_move_problems(repo.root)
    changed = repo.changed_paths()
    moved, problem = lib.moved_only_paths("HEAD", changed, cwd=repo.root)
    added = repo.added_lines()
    kept, lines_problem = lib.drop_moved_lines("HEAD", added, cwd=repo.root)
    show("lines_relocated", registry_rows=count, registry_problems=problems, changed=changed, moved_only=sorted(moved), problem=problem)
    show("lines_relocated.added_lines", added=[entry for entry in added if entry[0].endswith(".md") and "records" not in entry[0]])
    show("lines_relocated.after_drop", kept=[entry for entry in kept if entry[0].endswith(".md") and "records" not in entry[0]], problem=lines_problem)


def gate_changed_paths(tree, repo):
    done = subprocess.run(["bash", "-c", 'source "$1" && gate_changed_paths HEAD untracked', "_", os.path.join(tree, "scripts/gate/changed-paths.sh")],
                          cwd=repo.root, capture_output=True, text=True, env=ENV)
    return done.returncode, [line for line in done.stdout.split("\n") if line], done.stderr.strip()


def sample_move_into_governed(tree, work, lib):
    """G3：一份文件经单文件登记搬进按路径管的地方（.claude/agents/、crates/<包>/src/），改动范围里看不见它，
    按路径挑文件的格（agent-def-adversarial-review、crates-adversarial-review 用的那两个正则）挑不出来。"""
    import re
    agents = re.compile(r"^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$")
    crates = re.compile(r"^crates/[^/]+/src/.*\.rs$")
    for registered in (False, True):
        repo = Repo(os.path.join(work, f"move-into-governed-{'registered' if registered else 'control'}"))
        repo.write("drafts/agents/reviewer.md", "---\nname: reviewer\n---\n# 新的子 agent 定义草稿\n")
        repo.write("research/proto/fast_path.rs", "pub fn fast_path() -> u32 { 7 }\n")
        repo.write("crates/core/src/lib.rs", "pub mod existing;\n")
        repo.write("crates/core/src/existing.rs", "pub fn existing() {}\n")
        repo.write(".claude/agents/old.md", "---\nname: old\n---\n")
        repo.write(RECORD, "# 样本记录\n")
        repo.commit("base")
        repo.git("mv", "drafts/agents/reviewer.md", ".claude/agents/reviewer.md")
        repo.git("mv", "research/proto/fast_path.rs", "crates/core/src/fast_path.rs")
        if registered:
            repo.register(("drafts/agents/reviewer.md", ".claude/agents/reviewer.md", "2026-10-23", RECORD),
                          ("research/proto/fast_path.rs", "crates/core/src/fast_path.rs", "2026-10-23", RECORD))
        count, problems = lib.path_move_problems(repo.root)
        rc, changed, stderr = gate_changed_paths(tree, repo)
        show(f"move_into_governed.{'registered' if registered else 'control'}", registry_rows=count, registry_problems=problems,
             gate_changed_paths_rc=rc, changed=changed, agent_definitions=[path for path in changed if agents.match(path)],
             crates_sources=[path for path in changed if crates.match(path)], stderr=stderr)


def sample_invariant_file_moved(tree, work, lib):
    """G3/G2：整份搬了家的文件不进「新增文件」（gate_changed_paths … A），而 invariant-anchors 那一格对「不是新增」的文件自己跑 git diff -U0 <基准> -- <路径>，
    新路径在基准里没有，整份都是 + 行。这里只打两样输出；那一格的判定是按它的代码推的。对照臂不登记搬迁。"""
    for registered in (False, True):
        repo = Repo(os.path.join(work, f"invariant-file-moved-{'registered' if registered else 'control'}"))
        repo.write("kb/old-invariants/table.md", "| I-1 | 旧行，没点名已定分项 |\n| I-2 | D1（样本） 已定项 1 |\n")
        repo.write(RECORD, "# 样本记录\n")
        repo.commit("base")
        repo.git("mv", "kb/old-invariants", "kb/invariants")
        if registered:
            repo.register(("kb/old-invariants", "kb/invariants", "2026-10-23", RECORD))
        done = subprocess.run(["bash", "-c", 'source "$1" && gate_changed_paths HEAD untracked A', "_", os.path.join(tree, "scripts/gate/changed-paths.sh")],
                              cwd=repo.root, capture_output=True, text=True, env=ENV)
        diff = repo.git("diff", "-U0", "HEAD", "--", "kb/invariants/table.md").stdout
        plus = [line[1:] for line in diff.split("\n") if line.startswith("+") and not line.startswith("+++")]
        show(f"invariant_file_moved.{'registered' if registered else 'control'}", added_files_rc=done.returncode,
             added_files=[line for line in done.stdout.split("\n") if line], own_diff_plus_lines=plus)


SAMPLES = ("launder_parent_excuse", "launder_deleted_directory", "history_references", "reverse_partial", "chain_midway",
           "lines_relocated", "move_into_governed", "invariant_file_moved")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tree", required=True)
    parser.add_argument("--work", required=True)
    parser.add_argument("samples", nargs="*")
    arguments = parser.parse_args()
    tree, work = os.path.abspath(arguments.tree), os.path.abspath(arguments.work)
    if os.path.exists(work) and os.listdir(work):
        sys.exit(f"--work {work} 不是空目录")
    os.makedirs(work, exist_ok=True)
    lib = load("lag_baseline_under_attack", os.path.join(tree, ".claude/gate.d/lib-lag-baseline.py"))
    sys.path.insert(0, os.path.join(tree, E171))
    import frozen_o3  # noqa: E402
    import reuse_library  # noqa: E402
    for name in arguments.samples or SAMPLES:
        function = globals()[f"sample_{name}"]
        extra = (frozen_o3, reuse_library) if name in ("reverse_partial", "chain_midway") else ()
        function(tree, work, lib, *extra)


if __name__ == "__main__":
    main()
