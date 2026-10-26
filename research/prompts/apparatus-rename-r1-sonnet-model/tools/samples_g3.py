"""G3 改动范围剔除的副作用：scripts/gate/changed-paths.sh 的 gate_changed_paths / gate_added_lines 在登记了搬迁的仓里，哪些文件与行被剔、哪些留下。
用法：python3 samples_g3.py --repo <仓根> --work <新目录>。一个临时 git 仓，十几种文件；产物一行 name=… 。"""
import argparse
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import GIT_ENV, git, make_repository, register_move, say, write  # noqa: E402

OLD, NEW = "scripts/experiments", "scripts/experiment-apparatus"
BASE = {
    f"{OLD}/run.sh": f"bash {OLD}/run.sh\n", f"{OLD}/edited.sh": f"bash {OLD}/edited.sh\nline two\n", f"{OLD}/deleted.sh": "gone\n",
    f"{OLD}/mode.sh": f"echo {OLD}\n", f"{OLD}/copied.sh": f"echo {OLD}/copied.sh\n",
    ".claude/agents/triage.md": f"- 在 `{OLD}/` 下跑\n", "crates/x/src/lib.rs": f'let p = "{OLD}/run.sh"; // 见 {OLD}\n',
    ".claude/hooks/guard.sh": f'exec bash {OLD}/run.sh\n', "docs/note.md": f"见 {OLD}/run.sh\n", "records/2026-10-23-sample.md": "# 样本\n",
}


def gate_call(repo, root, function, *arguments):
    script = f'source "{repo}/scripts/gate/changed-paths.sh" && {function} "$@"'
    done = subprocess.run(["bash", "-c", script, "bash", *arguments], cwd=root, env=GIT_ENV, capture_output=True, text=True)
    return done.returncode, [line for line in done.stdout.split("\n") if line], done.stderr.strip()[:200]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    repo, work = os.path.abspath(arguments.repo), os.path.abspath(arguments.work)
    root = os.path.join(work, "c1-change-range")
    os.makedirs(work, exist_ok=True)
    make_repository(root, BASE)
    git(root, "mv", OLD, NEW)
    for name in ("run.sh", "mode.sh"):
        write(root, f"{NEW}/{name}", BASE[f"{OLD}/{name}"].replace(OLD, NEW))
    write(root, f"{NEW}/edited.sh", f"bash {NEW}/edited.sh\nline two changed\n")        # 搬家又改了别的
    git(root, "rm", "-q", "-f", f"{NEW}/deleted.sh")                                      # 真删掉
    os.chmod(os.path.join(root, NEW, "mode.sh"), 0o755)                                   # 搬家又改了模式
    write(root, f"{NEW}/copied.sh", BASE[f"{OLD}/copied.sh"].replace(OLD, NEW))
    write(root, f"{NEW}/brand-new.sh", f"echo {NEW}/brand-new.sh\n")                      # 基准里没有对应的新文件
    for relative in (".claude/agents/triage.md", "crates/x/src/lib.rs", ".claude/hooks/guard.sh", "docs/note.md"):
        write(root, relative, BASE[relative].replace(OLD, NEW))
    register_move(root, OLD, NEW)
    git(root, "add", "-A")
    write(root, f"{NEW}/run.sh", BASE[f"{OLD}/run.sh"].replace(OLD, NEW) + "extra unstaged line\n")   # 暂存的只差搬迁，工作区又多了一行
    write(root, f"{NEW}/untracked-copy.sh", BASE[f"{OLD}/run.sh"].replace(OLD, NEW))                  # 未跟踪的整份拷贝，内容 = 基准 run.sh 换算后
    code, kept, error = gate_call(repo, root, "gate_changed_paths", "HEAD", "untracked")
    all_paths = sorted(set(BASE) | {f"{NEW}/{n}" for n in ("run.sh", "edited.sh", "deleted.sh", "mode.sh", "copied.sh", "brand-new.sh", "untracked-copy.sh")} | {".claude/gate.d/path-moves.tsv"})
    say("c1-gate-changed-paths", exit=code, kept=kept, error=error)
    for relative in (".claude/agents/triage.md", "crates/x/src/lib.rs", ".claude/hooks/guard.sh", "docs/note.md", f"{NEW}/copied.sh", f"{NEW}/mode.sh",
                     f"{NEW}/edited.sh", f"{NEW}/deleted.sh", f"{NEW}/brand-new.sh", f"{NEW}/run.sh", f"{NEW}/untracked-copy.sh", ".claude/gate.d/path-moves.tsv"):
        say("c1-verdict", path=relative, seen_as_changed=relative in kept)
    code, kept_added, error = gate_call(repo, root, "gate_changed_paths", "HEAD", "untracked", "A")
    say("c1-gate-changed-paths-added-only", exit=code, kept=kept_added)
    code, lines, error = gate_call(repo, root, "gate_added_lines", "HEAD", ".")
    say("c1-gate-added-lines", exit=code, lines=[line[:90] for line in lines])
    copy_root = os.path.join(work, "c2-copy-old-still-there")
    make_repository(copy_root, {f"{OLD}/a.sh": f"echo {OLD}/a.sh\n", "records/2026-10-23-sample.md": "# 样本\n"})
    write(copy_root, f"{NEW}/a.sh", f"echo {NEW}/a.sh\n")
    register_move(copy_root, OLD, NEW)
    code, kept_copy, error = gate_call(repo, copy_root, "gate_changed_paths", "HEAD", "untracked")
    check = subprocess.run([sys.executable, "-I", os.path.join(repo, ".claude/gate.d/lib-lag-baseline.py"), "path-moves-check", copy_root], cwd=copy_root, capture_output=True, text=True)
    say("c2-copy-old-still-there", change_range=kept_copy, registry_cell_exit=check.returncode, registry_cell_says=check.stdout.strip().split("\n")[0][:160])


if __name__ == "__main__":
    main()
