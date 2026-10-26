"""辩方腿的样本公共件：载入冻结副本里的 lib-lag-baseline.py、造临时 git 仓、独立数相同的份数（不调库里的换算）。"""
import importlib.util
import os
import subprocess
import sys

sys.dont_write_bytecode = True


def load_library(frozen):
    path = os.path.join(frozen, ".claude", "gate.d", "lib-lag-baseline.py")
    spec = importlib.util.spec_from_file_location("lag_baseline_frozen", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Repo:
    def __init__(self, root, fresh=True):
        self.root = root
        os.makedirs(root, exist_ok=True)
        if fresh:
            self.git("init", "-q")
            self.git("config", "user.email", "d@example.invalid")
            self.git("config", "user.name", "defense")

    def git(self, *arguments, check=True):
        return subprocess.run(["git", "-C", self.root, "-c", "core.quotepath=false", *arguments], check=check, capture_output=True, text=True)

    def write(self, relative, text, executable=False):
        full = os.path.join(self.root, relative)
        os.makedirs(os.path.dirname(full) or self.root, exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(text)
        if executable:
            os.chmod(full, 0o755)

    def read(self, relative):
        with open(os.path.join(self.root, relative), "rb") as handle:
            return handle.read()

    def remove_tree(self, relative):
        subprocess.run(["rm", "-rf", os.path.join(self.root, relative)], check=True)

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-qm", message)

    def register(self, *rows):
        self.write(".claude/gate.d/path-moves.tsv", "# 搬迁登记（样本）\n" + "".join("\t".join(row) + "\n" for row in rows))

    def copy_to(self, target):
        subprocess.run(["cp", "-a", self.root, target], check=True)
        return Repo(target, fresh=False)

    def staged_paths(self):
        self.git("add", "-A")
        out = self.git("diff", "--cached", "--name-only", "--no-renames", "HEAD").stdout
        return [line for line in out.split("\n") if line]


SOURCE = "records/2026-10-23-搬迁.md"


def row(old, new):
    return (old, new, "2026-10-23", SOURCE)


def plain_same_count(repo, old, new, files):
    """独立数「换算之后相同」：基准里 old 下每份，把正文里的 old 整串换成 new 之后与工作区 new 下同名那份逐字节比（不调库里的换算与边界规则）。"""
    same = 0
    for relative in files:
        target = new + relative[len(old):]
        base = repo.git("show", f"HEAD:{relative}", check=False)
        if base.returncode != 0:
            continue
        try:
            current = repo.read(target)
        except OSError:
            continue
        if current == base.stdout.replace(old, new).encode("utf-8"):
            same += 1
    return same
