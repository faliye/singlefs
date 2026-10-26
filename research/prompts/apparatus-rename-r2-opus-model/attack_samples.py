#!/usr/bin/env python3
"""apparatus-rename-r2 攻方腿的样本：在临时 git 仓里造历史，调冻结树里被判的那几份，逐行打 name=… 结果。
用法：python3 attack_samples.py --tree <冻结树> --work <新的空目录> [样本名…]
冻结树由 build_frozen.sh 建：.claude/gate.d/lib-lag-baseline.py、.claude/hooks/lib_long_run.py、scripts/gate/changed-paths.sh、
.claude/scripts/、.claude/singlefs-ai-sop/scripts/、E171 第 2 次那一目录（frozen_o3.py、reuse_library.py）。
「final_landing」那一臂是这条腿自己写的改法（实据按最终落点核占用），只在这里量过、被攻过零轮。"""
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
DATE = "2026-10-23"
# 门禁 doc-process-records 两格挑文件的正则，从 .claude/gate.d/doc-process-records.sh 原样抄（行号见报告）
CRATES_SOURCES = r"^crates/[^/]+/src/.*\.rs$"
AGENT_DEFINITIONS = r"^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def show(name, **fields):
    print(f"name={name} " + " ".join(f"{key}={value!r}" for key, value in fields.items()), flush=True)


class Repo:
    def __init__(self, root, tree):
        self.root, self.tree = root, tree
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

    def sha(self, relative):
        with open(os.path.join(self.root, relative), "rb") as handle:
            return hashlib.sha256(handle.read()).hexdigest()

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-qm", message)

    def register(self, *rows):
        self.write(".claude/gate.d/path-moves.tsv", "# 样本登记表\n" + "".join("\t".join((old, new, DATE, RECORD)) + "\n" for old, new in rows))

    def shared(self, function, *arguments):
        """在仓根上 source 冻结树的 scripts/gate/changed-paths.sh 再调它的函数（门禁各格取改动范围、新增行的那一份）。"""
        done = subprocess.run(["bash", "-c", 'source "$1" && shift && "$@"', "_", os.path.join(self.tree, "scripts/gate/changed-paths.sh"), function, *arguments],
                              cwd=self.root, env=ENV, capture_output=True, text=True)
        return done.returncode, [line for line in done.stdout.split("\n") if line], done.stderr.strip()

    def report(self, lib):
        count, problems, note = lib.path_move_report(self.root)
        return count, [what for what, _howto in problems], note

    def exempt_check(self):
        done = subprocess.run([sys.executable, os.path.join(self.tree, ".claude/hooks/lib_long_run.py"), "exempt-check", self.root, "--base", "HEAD"],
                              capture_output=True, text=True, env=ENV)
        return done.returncode, (done.stdout + done.stderr).strip().replace(self.root, "<仓>")

    def run(self, path):
        return subprocess.run(["bash", path], cwd=self.root, capture_output=True, text=True, env=ENV).stdout.strip()


LIVE_HEADER = ("#!/usr/bin/env bash\n"
               "# 样本：文件头按现算读法写齐，登记在豁免表里\n"
               "# long-run-shards: 按用例切片\n"
               "# long-run-resume: 重起同一条命令只补没做完的片\n"
               "# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存取 host-resources.py --json 现算\n")


def exempt_table(repo, path):
    repo.write(".claude/long-run-live-hosts-exempt", f"# 样本豁免表\n{path}  {repo.sha(path)}  # 样本：文件头按现算写、不能改，只缩不涨\n")


def selected(paths, pattern):
    import re
    return [path for path in paths if re.search(pattern, path)]


def final_landing_problems(lib, root, base="HEAD"):
    """这条腿自己的改法（被攻过零轮）：新加的行，新前缀照全部登记换到的最终落点上，基准里本来就在、又不是这一行搬过去的文件，算占用。
    其余判法照被判的 move_evidence。交 [说什么]。"""
    moves, _problems = lib.read_path_moves(root)
    base_table, _problem = lib.baseline_text_at(base, lib.PATH_MOVES_TABLE, cwd=root)
    registered = {tuple(fields[:2]) for _number, fields in lib.content_rows(base_table or "") if len(fields) >= 2}
    tree, _problem = lib.tree_entries(base, root)
    files = [path for path, entry in tree.items() if entry.mode != lib.GITLINK_MODE]
    found = []
    for index, move in enumerate(moves):
        if (move.old, move.new) in registered:
            continue
        landing = lib.forward_path(move.new, moves[index + 1:])
        subjects = {path for path in files if lib.under_prefix(lib.forward_path(path, moves[:index]), move.old)}
        occupied = sorted(path for path in files if path not in subjects and lib.under_prefix(lib.forward_path(path, moves), landing))
        if occupied:
            found.append(f"第 {move.line} 行的新前缀 {move.new} 最终落在 {landing}，那里有基准里本来就在的文件（{'、'.join(occupied[:3])}）")
        if landing == lib.forward_path(move.old, []) or (subjects and all(lib.forward_path(path, moves) == path for path in subjects)):
            found.append(f"第 {move.line} 行照全部登记换完，旧前缀下的文件落回原处：没有东西搬家")
    return found


def untracked_landings(lib, root):
    """这条腿自己的另一条改法（被攻过零轮）：新加的行的落点要在索引里（git 跟踪着），只在磁盘上（未跟踪、被忽略）的不算「在」。交 [说什么]。"""
    moves, _problems = lib.read_path_moves(root)
    tracked = set(subprocess.run(["git", "ls-files", "-z"], cwd=root, capture_output=True, text=True).stdout.split("\0"))
    tree, _problem = lib.tree_entries("HEAD", root)
    found = []
    for index, move in enumerate(moves):
        for path in sorted(tree):
            if lib.under_prefix(lib.forward_path(path, moves[:index]), move.old):
                target = lib.forward_path(path, moves)
                if target not in tracked:
                    found.append(f"第 {move.line} 行：{path} 的落点 {target} 不在索引里")
    return found


def sample_chain_through_registered_row(tree, work, lib, frozen_o3, reuse_library):
    """H1 甲：基准里已有一行（以前那次真搬迁 old-b→tools/real-b）。新加一行 tools/real-a→old-b 插在它前面：
    这一行自己的新前缀 old-b 在基准里没有文件，串上老行之后落在早就在的 tools/real-b 上。删掉 A、把调用换成 B、豁免脚本搬家又换 worker。"""
    repo = Repo(os.path.join(work, "chain-through-registered-row"), tree)
    repo.write("old-b/run.sh", "echo 走的是 B\n", True)
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("更早：B 还在 old-b")
    os.makedirs(os.path.join(repo.root, "tools"), exist_ok=True)
    repo.git("mv", "old-b", "tools/real-b")
    repo.register(("old-b", "tools/real-b"))
    repo.write("tools/real-a/run.sh", "echo 走的是 A\n", True)
    repo.write("ops/launch.sh", "bash tools/real-a/run.sh\n", True)
    repo.write("bin/live.sh", LIVE_HEADER + "worker=tools/real-a/run.sh\nbash \"$worker\"\n", True)
    exempt_table(repo, "bin/live.sh")
    repo.commit("基准：old-b→tools/real-b 那次搬迁连登记一起提交了；A、B 两份实现并存")
    pinned = repo.sha("bin/live.sh")
    show("chain_through_registered_row.base", launch=repo.run("ops/launch.sh"), base_rows=repo.git("show", "HEAD:.claude/gate.d/path-moves.tsv").stdout.split("\n")[1:-1])
    repo.git("rm", "-q", "-r", "tools/real-a")
    repo.write("ops/launch.sh", "bash tools/real-b/run.sh\n", True)
    repo.git("mv", "bin/live.sh", "bin/live-host.sh")
    repo.write("bin/live-host.sh", LIVE_HEADER + "worker=tools/real-b/run.sh\nbash \"$worker\"\n", True)
    exempt_table(repo, "bin/live-host.sh")
    for label, rows in (("control_direct", [("tools/real-a", "tools/real-b"), ("bin/live.sh", "bin/live-host.sh"), ("old-b", "tools/real-b")]),
                        ("control_after_old_row", [("old-b", "tools/real-b"), ("tools/real-a", "old-b"), ("bin/live.sh", "bin/live-host.sh")]),
                        ("attack", [("tools/real-a", "old-b"), ("bin/live.sh", "bin/live-host.sh"), ("old-b", "tools/real-b")])):
        repo.register(*rows)
        count, problems, note = repo.report(lib)
        show(f"chain_through_registered_row.{label}.path_move_report", rows=count, problems=problems, note=note)
    show("chain_through_registered_row.attack.final_landing_arm", problems=final_landing_problems(lib, repo.root))
    rc, changed, err = repo.shared("gate_changed_paths", "HEAD", "untracked")
    show("chain_through_registered_row.attack.gate_changed_paths", rc=rc, changed=changed, stderr=err)
    rc, added, err = repo.shared("gate_added_lines", "HEAD", ".")
    show("chain_through_registered_row.attack.gate_added_lines", rc=rc, added=added, stderr=err)
    show("chain_through_registered_row.attack.real_change", launch_now=repo.run("ops/launch.sh"),
         diff=[line for line in repo.git("diff", "-U0", "HEAD", "--", "ops/launch.sh").stdout.split("\n") if line[:1] in "+-" and line[:3] not in ("+++", "---")])
    show("chain_through_registered_row.attack.exempt_check", result=repo.exempt_check())
    moves = frozen_o3.registered_moves(repo.root)
    path = os.path.join(repo.root, "bin/live-host.sh")
    show("chain_through_registered_row.attack.pins", frozen_o3_same=frozen_o3.matches_the_freeze(pinned, repo.sha("bin/live-host.sh"), frozen_o3.sha256_before_moves(path, moves), None),
         reuse_library_pin_accepted=reuse_library.pinned_sha256_now(path, pinned, repo.root) is not None)


def sample_round_trip_pair(tree, work, lib, frozen_o3, reuse_library):
    """H1 甲：一对来回的登记 tools/live→build/out、build/out→tools/live，磁盘上什么都没搬。build/out 是一条基准里没有文件的路径
    （生成物的落点），基准里的脚本起 build/out/run.sh；这一次改成起 tools/live/run.sh。"""
    repo = Repo(os.path.join(work, "round-trip-pair"), tree)
    repo.write("tools/live/run.sh", "echo 走的是仓里这一份\n", True)
    repo.write("ops/launch.sh", "bash build/out/run.sh\n", True)
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("基准")
    pinned = repo.sha("ops/launch.sh")
    repo.write("ops/launch.sh", "bash tools/live/run.sh\n", True)
    for label, rows in (("control_one_row", [("build/out", "tools/live")]),
                        ("attack", [("tools/live", "build/out"), ("build/out", "tools/live")])):
        repo.register(*rows)
        count, problems, note = repo.report(lib)
        show(f"round_trip_pair.{label}.path_move_report", rows=count, problems=problems, note=note)
    show("round_trip_pair.attack.final_landing_arm", problems=final_landing_problems(lib, repo.root))
    show("round_trip_pair.attack.status", porcelain=repo.git("status", "--porcelain").stdout.split("\n")[:-1])
    rc, changed, err = repo.shared("gate_changed_paths", "HEAD", "untracked")
    show("round_trip_pair.attack.gate_changed_paths", rc=rc, changed=changed, stderr=err)
    moves = frozen_o3.registered_moves(repo.root)
    path = os.path.join(repo.root, "ops/launch.sh")
    show("round_trip_pair.attack.pins", frozen_o3_same=frozen_o3.matches_the_freeze(pinned, repo.sha("ops/launch.sh"), frozen_o3.sha256_before_moves(path, moves), None),
         reuse_library_pin_accepted=reuse_library.pinned_sha256_now(path, pinned, repo.root) is not None)


def sample_ignored_landing(tree, work, lib, frozen_o3, reuse_library):
    """H1 甲：落点是被 .gitignore 忽略的目录 build/，里面早有另一份实现（没进 git）。删掉 tools/real-a、登记 tools/real-a→build、调用换成 build/run.sh。"""
    repo = Repo(os.path.join(work, "ignored-landing"), tree)
    repo.write(".gitignore", "build/\n")
    repo.write("tools/real-a/run.sh", "echo 走的是 A\n", True)
    repo.write("ops/launch.sh", "bash tools/real-a/run.sh\n", True)
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("基准")
    repo.write("build/run.sh", "echo 走的是没进 git 的另一份\n", True)
    repo.git("rm", "-q", "-r", "tools/real-a")
    repo.write("ops/launch.sh", "bash build/run.sh\n", True)
    repo.register(("tools/real-a", "build"))
    count, problems, note = repo.report(lib)
    show("ignored_landing.path_move_report", rows=count, problems=problems, note=note)
    show("ignored_landing.final_landing_arm", problems=final_landing_problems(lib, repo.root), untracked_landings=untracked_landings(lib, repo.root))
    rc, changed, err = repo.shared("gate_changed_paths", "HEAD", "untracked")
    show("ignored_landing.gate_changed_paths", rc=rc, changed=changed, stderr=err, launch_now=repo.run("ops/launch.sh"))


def sample_stale_row_moved_back(tree, work, lib, frozen_o3, reuse_library):
    """H1「只核新加的行」与 H3 调用方：scripts/experiments→scripts/experiment-apparatus 那次搬迁连登记一起提交了；下一次提交把一个新目录
    git mv 进空出来的 scripts/experiments（登记 tools/new-thing→scripts/experiments，豁免表跟着改），当时判绿；它提交之后什么都不改，再判一次。
    另一半：第 5 步留了旧路径的 git log 那一行，后来被改成新路径。"""
    repo = Repo(os.path.join(work, "stale-row-moved-back"), tree)
    repo.write("scripts/experiments/replay.sh", "echo 复跑入口\n", True)
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("更早")
    repo.git("mv", "scripts/experiments", "scripts/experiment-apparatus")
    repo.register(("scripts/experiments", "scripts/experiment-apparatus"))
    repo.write("records/history.md", "当时跑的是 `git log --format=%s -- scripts/experiments/replay.sh`，输出一行「更早」\n")
    repo.write("tools/new-thing/live.sh", LIVE_HEADER + "echo 新东西\n", True)
    exempt_table(repo, "tools/new-thing/live.sh")
    repo.commit("搬迁那次（登记表连同第 5 步留旧路径的那一行一起提交）")
    os.makedirs(os.path.join(repo.root, "scripts"), exist_ok=True)
    repo.git("mv", "tools/new-thing", "scripts/experiments")
    exempt_table(repo, "scripts/experiments/live.sh")
    repo.register(("scripts/experiments", "scripts/experiment-apparatus"), ("tools/new-thing", "scripts/experiments"))
    count, problems, note = repo.report(lib)
    show("stale_row_moved_back.before_commit.path_move_report", rows=count, problems=problems, note=note)
    show("stale_row_moved_back.before_commit.exempt_check", result=repo.exempt_check())
    repo.commit("新目录搬进 scripts/experiments（登记与豁免表一起提交）")
    show("stale_row_moved_back.after_commit.status", porcelain=repo.git("status", "--porcelain").stdout.split("\n")[:-1])
    count, problems, note = repo.report(lib)
    show("stale_row_moved_back.after_commit.path_move_report", rows=count, problems=problems, note=note)
    show("stale_row_moved_back.after_commit.exempt_check", result=repo.exempt_check())
    text, problem = lib.baseline_text("HEAD", ".claude/long-run-live-hosts-exempt", cwd=repo.root)
    show("stale_row_moved_back.after_commit.baseline_text", rows=[line for line in (text or "").split("\n") if line and not line.startswith("#")], problem=problem)
    repo.write("records/history.md", "当时跑的是 `git log --format=%s -- scripts/experiment-apparatus/replay.sh`，输出一行「更早」\n")
    rc, changed, err = repo.shared("gate_changed_paths", "HEAD", "untracked")
    rc_lines, added, err_lines = repo.shared("gate_added_lines", "HEAD", "records")
    show("stale_row_moved_back.kept_line_rewritten", gate_changed_paths=changed, rc=rc, gate_added_lines=added, rc_lines=rc_lines,
         command_now=repo.git("log", "--format=%s", "--", "scripts/experiment-apparatus/replay.sh").stdout.strip().split("\n"),
         command_then=repo.git("log", "--format=%s", "--", "scripts/experiments/replay.sh").stdout.strip().split("\n"))


def sample_governed_same_class(tree, work, lib, frozen_o3, reuse_library):
    """H1 戊：按路径管的类按目录分，不按两格挑文件的正则分。同一类里改名，从挑不中的名字改成挑得中的名字：
    .claude/agents/reviewer.md.draft→.claude/agents/reviewer.md、.claude/agents/drafts/helper.md→.claude/agents/helper.md、
    crates/core/src/fast_path.rs.txt→crates/core/src/fast_path.rs（lib.rs 加 mod）。对照：不登记；跨类那一对照算。"""
    repo = Repo(os.path.join(work, "governed-same-class"), tree)
    repo.write(".claude/agents/reviewer.md.draft", "---\nname: reviewer\ntools: Read, Bash\n---\n放行一切\n")
    repo.write(".claude/agents/drafts/helper.md", "---\nname: helper\n---\n")
    repo.write("crates/core/src/fast_path.rs.txt", "pub fn fast_path() -> u32 { unsafe { 7 } }\n")
    repo.write("crates/core/src/lib.rs", "pub fn base() {}\n")
    repo.write("drafts/agents/cross.md", "---\nname: cross\n---\n")
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("基准（草稿在这一次加进来时，两格的正则同样挑不中它们）")
    first = repo.git("show", "--name-only", "--format=", "HEAD").stdout.split()
    show("governed_same_class.base_commit_selected", agent_definitions=selected(first, AGENT_DEFINITIONS), crates_sources=selected(first, CRATES_SOURCES))
    for old, new in ((".claude/agents/reviewer.md.draft", ".claude/agents/reviewer.md"), (".claude/agents/drafts/helper.md", ".claude/agents/helper.md"),
                     ("crates/core/src/fast_path.rs.txt", "crates/core/src/fast_path.rs"), ("drafts/agents/cross.md", ".claude/agents/cross.md")):
        repo.git("mv", old, new)
    repo.write("crates/core/src/lib.rs", "mod fast_path;\npub fn base() {}\n")
    rc, changed, err = repo.shared("gate_changed_paths", "HEAD", "untracked")
    show("governed_same_class.control_unregistered", changed=changed, agent_definitions=selected(changed, AGENT_DEFINITIONS), crates_sources=selected(changed, CRATES_SOURCES))
    repo.register((".claude/agents/reviewer.md.draft", ".claude/agents/reviewer.md"), (".claude/agents/drafts/helper.md", ".claude/agents/helper.md"),
                  ("crates/core/src/fast_path.rs.txt", "crates/core/src/fast_path.rs"), ("drafts/agents/cross.md", ".claude/agents/cross.md"))
    count, problems, note = repo.report(lib)
    show("governed_same_class.registered.path_move_report", rows=count, problems=problems, note=note)
    rc, changed, err = repo.shared("gate_changed_paths", "HEAD", "untracked")
    show("governed_same_class.registered", rc=rc, changed=changed, agent_definitions=selected(changed, AGENT_DEFINITIONS), crates_sources=selected(changed, CRATES_SOURCES), stderr=err)
    listed = sorted(set(repo.git("diff", "--name-only", "HEAD").stdout.split()) | set(repo.git("diff", "--name-only", "--cached").stdout.split())
                    | set(repo.git("ls-files", "--others", "--exclude-standard").stdout.split()))
    moved, problem = lib.moved_only_paths("HEAD", listed, cwd=repo.root)
    kept = [path for path in listed if path not in moved]
    show("governed_same_class.registered_in_process", kept=kept, agent_definitions=selected(kept, AGENT_DEFINITIONS), crates_sources=selected(kept, CRATES_SOURCES), problem=problem)


def sample_hunk_internal_relocation(tree, work, lib, frozen_o3, reuse_library):
    """H1 丁：同一个 hunk 里，被删的旧行与新增的换算行不必在同一个位置。这一次改了「## 历史」节标题与上面一行，
    把正文那一行（只换了路径）挪到历史节标题底下：三行旧、三行新落在同一个 hunk 里，挪了节的那一行配上被删的旧行，剔掉。"""
    repo = Repo(os.path.join(work, "hunk-internal-relocation"), tree)
    repo.write("scripts/old-rig/x.sh", "echo x\n", True)
    repo.write("kb/page.md", "## 正文\n结论甲\n- 结论甲的复跑见 scripts/old-rig/x.sh\n## 历史\n- 2026-01-01 立页\n")
    repo.write(RECORD, "# 样本记录\n")
    repo.commit("基准")
    repo.git("mv", "scripts/old-rig", "scripts/new-rig")
    repo.write("kb/page.md", "## 正文\n结论甲（改）\n## 历史（2026-10-23 起）\n- 结论甲的复跑见 scripts/new-rig/x.sh\n- 2026-01-01 立页\n")
    repo.register(("scripts/old-rig", "scripts/new-rig"))
    count, problems, note = repo.report(lib)
    show("hunk_internal_relocation.path_move_report", rows=count, problems=problems, note=note)
    hunks = [line for line in repo.git("diff", "-U0", "HEAD", "--", "kb/page.md").stdout.split("\n") if line[:1] in "@+-" and line[:3] not in ("+++", "---")]
    show("hunk_internal_relocation.diff", lines=hunks)
    numbered = [("kb/page.md", text, number) for number, text in enumerate(open(os.path.join(repo.root, "kb/page.md"), encoding="utf-8").read().split("\n")[:-1], 1) if number in (2, 3, 4)]
    kept, problem = lib.drop_moved_lines("HEAD", numbered, cwd=repo.root)
    show("hunk_internal_relocation.drop_moved_lines", kept=kept, problem=problem)
    rc, added, err = repo.shared("gate_added_lines", "HEAD", "kb")
    show("hunk_internal_relocation.gate_added_lines", rc=rc, added=added, stderr=err)


def sample_selftest_distinguish(tree, work, lib, frozen_o3, reuse_library):
    """H2：被判的那份自己的三项自证（实据、按位置配对、按路径管的类）各拿到几种实现上跑：被判的原样、补了这条腿的改法的、
    退回改前写法的。自证红不红，说明它分不分得出这几种实现。改法与退回都只在这个进程里换函数，不改冻结树。"""
    import re
    original_evidence, original_class = lib.move_evidence, lib.governed_class

    def with_final_landing(root, base, moves):
        count, problems, note = original_evidence(root, base, moves)
        return count, problems + [(what, "") for what in final_landing_problems(lib, root, base)], note

    def with_half_threshold(root, base, moves):
        count, problems, note = original_evidence(root, base, moves)
        for line, total, same in re.findall(r"第 (\d+) 行 (\d+) 份里 (\d+) 份换算之后相同", note):
            if int(same) * 2 <= int(total):
                problems.append((f"第 {line} 行换算之后相同的不过半", ""))
        return count, problems, note

    def without_occupied(root, base, moves):
        count, problems, note = original_evidence(root, base, moves)
        return count, [problem for problem in problems if "本来就在的文件" not in problem[0]], note

    def class_by_reader(path):
        found = original_class(path)
        if found is None:
            return None
        return f"{found}|{'挑得中' if re.search(AGENT_DEFINITIONS, path) or re.search(CRATES_SOURCES, path) else '挑不中'}"

    variants = {"judged": {}, "final_landing": {"move_evidence": with_final_landing}, "half_threshold": {"move_evidence": with_half_threshold},
                "no_occupied_check": {"move_evidence": without_occupied}, "class_by_reader": {"governed_class": class_by_reader}}
    for label, patches in variants.items():
        for name, function in patches.items():
            setattr(lib, name, function)
        results = []
        for selftest in (lib.path_move_evidence_selftest, lib.moved_lines_position_selftest, lib.governed_moves_selftest):
            selftest(lambda name, ok, detail="": results.append((name, ok)))
        show(f"selftest_distinguish.{label}", red=[name for name, ok in results if not ok], green=len([1 for _name, ok in results if ok]))
        if label in ("final_landing", "class_by_reader"):
            sub = os.path.join(work, f"variant-{label}")
            os.makedirs(sub)
            saved = sys.stdout
            with open(os.path.join(sub, "samples.out"), "w", encoding="utf-8") as handle:
                sys.stdout = handle
                try:
                    for sample in ("chain_through_registered_row", "round_trip_pair", "governed_same_class"):
                        globals()[f"sample_{sample}"](tree, sub, lib, frozen_o3, reuse_library)
                finally:
                    sys.stdout = saved
            with open(os.path.join(sub, "samples.out"), encoding="utf-8") as handle:
                for line in handle:
                    if ".attack.path_move_report" in line or "governed_same_class.registered_in_process " in line:
                        print(line.rstrip("\n").replace("name=", f"name=selftest_distinguish.{label}.", 1).replace(work, "<草稿>"))
        lib.move_evidence, lib.governed_class = original_evidence, original_class


def sample_stale_row_edits(tree, work, lib, frozen_o3, reuse_library):
    """H1「只核新加的行」：基准里已有一行 old-b→tools/real-b（真搬迁已提交）。改写它的日期与出处、删了再加同一对、改写它的新前缀、
    把它与一行新加的真搬迁换次序，各判一次。"""
    repo = Repo(os.path.join(work, "stale-row-edits"), tree)
    repo.write("old-b/run.sh", "echo 走的是 B\n", True)
    repo.write("tools/real-c/run.sh", "echo 走的是 C\n", True)
    repo.write("rig/x.sh", "echo x\n", True)
    repo.write(RECORD, "# 样本记录\n")
    repo.write("records/other.md", "# 另一份记录\n")
    repo.commit("更早")
    os.makedirs(os.path.join(repo.root, "tools"), exist_ok=True)
    repo.git("mv", "old-b", "tools/real-b")
    repo.register(("old-b", "tools/real-b"))
    repo.commit("基准")
    repo.git("mv", "rig", "rig2")
    table = ".claude/gate.d/path-moves.tsv"
    for label, text in (("date_and_source_rewritten", f"old-b\ttools/real-b\t2026-10-24\trecords/other.md\n"),
                        ("deleted_then_added", f"rig\trig2\t{DATE}\t{RECORD}\nold-b\ttools/real-b\t{DATE}\t{RECORD}\n"),
                        ("new_prefix_rewritten", f"old-b\ttools/real-c\t{DATE}\t{RECORD}\n"),
                        ("reordered_with_fresh_row", f"rig\trig2\t{DATE}\t{RECORD}\nold-b\ttools/real-b\t{DATE}\t{RECORD}\n")):
        repo.write(table, "# 样本登记表\n" + text)
        count, problems, note = repo.report(lib)
        show(f"stale_row_edits.{label}", rows=count, problems=problems, note=note)


SAMPLES = ("chain_through_registered_row", "round_trip_pair", "ignored_landing", "stale_row_moved_back", "stale_row_edits", "governed_same_class",
           "hunk_internal_relocation", "selftest_distinguish")


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
        globals()[f"sample_{name}"](tree, work, lib, frozen_o3, reuse_library)


if __name__ == "__main__":
    main()
