"""正推腿样本的共用件：载入被判的读法库、造临时 git 仓、起命令行。只读 --repo 里的被判文件，写只写 --work。"""
import importlib.util
import os
import subprocess
import sys

GIT_ENV = dict(os.environ, GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1", GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@t.invalid",
               GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@t.invalid", LAG_BASELINE_BREAK="", LIB_LONG_RUN_BREAK="")
MOVES_ROW = "{old}\t{new}\t2026-10-23\trecords/2026-10-23-sample.md\n"


def load_library(repo):
    """载入 repo 里的 .claude/gate.d/lib-lag-baseline.py（被判的读法库本身，不改一个字节）。"""
    path = os.path.join(repo, ".claude", "gate.d", "lib-lag-baseline.py")
    specification = importlib.util.spec_from_file_location("lib_lag_baseline_under_test", path)
    module = importlib.util.module_from_spec(specification)
    sys.dont_write_bytecode = True
    specification.loader.exec_module(module)
    return module, path


def git(root, *arguments, check=True):
    done = subprocess.run(["git", "-c", "core.quotepath=false", *arguments], cwd=root, env=GIT_ENV, capture_output=True, text=True)
    if check and done.returncode != 0:
        raise RuntimeError(f"git {' '.join(arguments)} 退 {done.returncode}：{done.stderr.strip()}")
    return done


def write(root, relative, text):
    full = os.path.join(root, relative)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(text)


def make_repository(root, files):
    """root 里建 git 仓、写 files（{相对路径: 正文}）、提交成基准 HEAD。"""
    os.makedirs(root)
    git(root, "init", "-q", "-b", "master", ".")
    for relative, text in files.items():
        write(root, relative, text)
    git(root, "add", "-A")
    git(root, "commit", "-qm", "base")


def register_move(root, old, new):
    write(root, ".claude/gate.d/path-moves.tsv", MOVES_ROW.format(old=old, new=new))


def run_library(library_path, root, *arguments, data=None):
    done = subprocess.run([sys.executable, "-I", library_path, *arguments], cwd=root, env=GIT_ENV, input=data, capture_output=True, text=True)
    return done.returncode, done.stdout, done.stderr


def say(name, **fields):
    """一行产物：name=<名> key=value …（值里的换行写成 ⏎）。"""
    rendered = " ".join(f"{key}={str(value).replace(chr(10), '⏎')}" for key, value in fields.items())
    print(f"name={name} {rendered}".rstrip())
