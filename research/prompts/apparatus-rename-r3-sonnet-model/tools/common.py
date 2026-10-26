"""辩方腿第三轮的样本公共件：载入冻结树里的 lib-lag-baseline.py、造临时 git 仓（可带上游）、把冻结树里的几份拷进样本仓。
样本脚本都不调库里的换算当「对照」：需要独立数的地方自己用 str.replace 或自己写的正则。"""
import importlib.util
import os
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
SOURCE = "records/2026-10-23-搬迁.md"
TABLE = ".claude/gate.d/path-moves.tsv"
CARRIED = (".claude/gate.d/lib-lag-baseline.py", ".claude/scripts/project_preflight.py", "scripts/gate/changed-paths.sh")


def load_library(frozen, name="lag_baseline_frozen", relative=".claude/gate.d/lib-lag-baseline.py"):
    path = os.path.join(frozen, relative)
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def row(old, new):
    return (old, new, "2026-10-23", SOURCE)


class Repo:
    def __init__(self, root, frozen=None, upstream=False):
        self.root = root
        os.makedirs(root, exist_ok=True)
        self.env = {**os.environ, "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_CONFIG_NOSYSTEM": "1", "GIT_AUTHOR_NAME": "d", "GIT_AUTHOR_EMAIL": "d@example.invalid",
                    "GIT_COMMITTER_NAME": "d", "GIT_COMMITTER_EMAIL": "d@example.invalid"}
        self.env.pop("GATE_BASE", None)
        self.git("init", "-q", "-b", "master")
        self.remote = None
        if frozen:
            for relative in CARRIED:
                self.write(relative, open(os.path.join(frozen, relative), encoding="utf-8").read())
                os.chmod(os.path.join(root, relative), 0o755)
        if upstream:
            self.remote = root + "-origin.git"
            subprocess.run(["git", "init", "-q", "--bare", "-b", "master", self.remote], check=True, env=self.env, capture_output=True)
            self.git("remote", "add", "origin", self.remote)

    def git(self, *arguments, check=True, stdin=None):
        return subprocess.run(["git", "-C", self.root, "-c", "core.quotepath=false", *arguments], check=check, capture_output=True, text=True, env=self.env, input=stdin)

    def write(self, relative, text):
        full = os.path.join(self.root, relative)
        os.makedirs(os.path.dirname(full) or self.root, exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(text)

    def read(self, relative):
        with open(os.path.join(self.root, relative), encoding="utf-8") as handle:
            return handle.read()

    def rm(self, relative):
        subprocess.run(["rm", "-rf", os.path.join(self.root, relative)], check=True)

    def register(self, *rows):
        self.write(TABLE, "# 搬迁登记（样本）\n" + "".join("\t".join(item) + "\n" for item in rows))
        self.write(SOURCE, "# 搬迁\n")

    def commit(self, message, push=False):
        self.git("add", "-A")
        self.git("commit", "-qm", message)
        if push:
            self.git("push", "-q", "-u", "origin", "master")
        return self.git("rev-parse", "HEAD").stdout.strip()

    def stage_all(self):
        self.git("add", "-A")

    def base(self, lag):
        base, problem = lag.gate_base_of(self.root)
        return base, problem

    def changed(self, base):
        """基准到工作区、HEAD 到暂存区、未跟踪的并集（与 gate_changed_paths 同一个范围，不剔搬迁）。"""
        names = set()
        names.update(self.git("diff", "--name-only", base).stdout.split("\n"))
        names.update(self.git("diff", "--name-only", "--cached").stdout.split("\n"))
        names.update(self.git("ls-files", "--others", "--exclude-standard").stdout.split("\n"))
        return sorted(name for name in names if name)


def short(items):
    return "[" + ", ".join(items) + "]"
