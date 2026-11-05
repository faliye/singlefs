#!/usr/bin/env python3
"""起真 bash 的判法：每条样本放进自己的会话与进程组（start_new_session），超时与结束都杀整组；
还按环境变量里的标记（ORACLE_MARK）扫 /proc，把 setsid -f 之类逃出会话的孙进程也找出来、杀掉。
桩（tools/stublib.sh、build_tree.sh 造的）把"谁被起了、带什么参数、什么前缀"写进 ORACLE_LOG。
"""
import json, os, signal, subprocess, sys, time, uuid

HERE = os.path.dirname(os.path.abspath(__file__))
BIN = os.path.join(HERE, "bin")


def marked_processes(mark):
    """环境里带 ORACLE_MARK=<mark> 的、还活着（不是僵尸）的进程：[(pid, cmdline)]"""
    found = []
    needle = ("ORACLE_MARK=" + mark).encode()
    me = os.getpid()
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == me:
            continue
        try:
            with open(f"/proc/{entry}/environ", "rb") as f:
                if needle not in f.read().split(b"\0"):
                    continue
            with open(f"/proc/{entry}/stat") as f:
                state = f.read().rsplit(")", 1)[1].split()[0]
            if state == "Z":
                continue
            with open(f"/proc/{entry}/cmdline", "rb") as f:
                cmd = f.read().replace(b"\0", b" ").decode("utf-8", "replace").strip()
            found.append((int(entry), cmd))
        except (OSError, IndexError):
            continue
    return found


def kill_marked(mark):
    for _ in range(5):
        procs = marked_processes(mark)
        if not procs:
            return
        for pid, _cmd in procs:
            try:
                os.kill(pid, signal.SIGKILL)
            except OSError:
                pass
        time.sleep(0.05)


def parse_log(path):
    events = []
    if not os.path.exists(path):
        return events
    for line in open(path, encoding="utf-8", errors="replace"):
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 9:
            continue
        name, cwd, pid = parts[0], parts[1], parts[2]
        env = dict(p.split("=", 1) for p in parts[3:8] if "=" in p)
        args = parts[8].split("\x1f")[:-1] if parts[8] else []
        events.append({"name": name, "cwd": cwd, "pid": pid, "env": env, "args": args})
    return events


def git(repo, *args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True)


def reset_repo(repo):
    git(repo, "reset", "-q")
    git(repo, "checkout", "-q", "--", ".")
    git(repo, "clean", "-fdq")


def inode_map(repo, keep_dir=None):
    """记每个已跟踪文件的 inode；给了 keep_dir 就给脚本类文件建硬链接钉住原 inode（释放后的 inode 号会被复用，光比号会漏判）"""
    result = {}
    for rel in git(repo, "ls-files").stdout.splitlines():
        if rel.startswith(".claude/hooks/"):
            continue
        try:
            result[rel] = os.stat(os.path.join(repo, rel)).st_ino
            if keep_dir and rel.endswith((".sh", ".py")):
                os.makedirs(keep_dir, exist_ok=True)
                os.link(os.path.join(repo, rel), os.path.join(keep_dir, rel.replace("/", "__")))
        except OSError:
            pass
    return result


def untracked_hashes(repo):
    import hashlib
    result = {}
    for rel in git(repo, "ls-files", "--others", "--exclude-standard").stdout.splitlines():
        try:
            result[rel] = hashlib.sha256(open(os.path.join(repo, rel), "rb").read()).hexdigest()
        except OSError:
            pass
    return result


def run_sample(repo, command, cwd=None, setup="", log_dir="/tmp", sample_id="x", timeout=6.0, stage=True, reset_first=True):
    """起真 bash -c <command>；返回观测。setup 是在同一棵树里先跑的准备（不经钩子、不记入观测）。"""
    if reset_first:
        reset_repo(repo)
    mark = uuid.uuid4().hex
    log = os.path.join(log_dir, f"{sample_id}.log")
    if os.path.exists(log):
        os.unlink(log)
    env = {"PATH": BIN + ":/usr/local/bin:/usr/bin:/bin", "HOME": os.environ.get("ORACLE_HOME", os.environ["HOME"]), "LANG": "C.UTF-8",
           "TMPDIR": "/tmp", "PYTHONDONTWRITEBYTECODE": "1", "CLAUDE_CODE_EXECPATH": BIN + "/claude", "ORACLE_LOG": log, "ORACLE_MARK": mark, "USER": os.environ.get("USER", "x")}
    if setup:
        subprocess.run(["bash", "-c", setup], cwd=repo, env={**env, "ORACLE_LOG": "/dev/null"}, capture_output=True, timeout=30)
        if stage:
            git(repo, "add", "-A")       # 准备出来的文件当作"已有的"（暂存区里的内容算已有，不算本条命令的改动）
    untracked_before = untracked_hashes(repo)
    keep_dir = os.path.join(log_dir, "keep-" + sample_id)
    before_inodes = inode_map(repo, keep_dir)
    staged_before = git(repo, "diff", "--cached", "--name-only").stdout.split()
    marker_time = time.time()
    out_path, err_path = log + ".out", log + ".err"
    with open(out_path, "wb") as fo, open(err_path, "wb") as fe:
        proc = subprocess.Popen(["bash", "-c", command], cwd=cwd or repo, env=env, stdin=subprocess.DEVNULL,
                                stdout=fo, stderr=fe, start_new_session=True)
        timed_out = False
        try:
            proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            try:
                os.killpg(proc.pid, signal.SIGKILL)
            except OSError:
                pass
            proc.wait()
    out, err = open(out_path, "rb").read(), open(err_path, "rb").read()
    time.sleep(0.15)
    orphans = [cmd for _pid, cmd in marked_processes(mark)]
    kill_marked(mark)
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except OSError:
        pass
    changed = []
    status = git(repo, "status", "--porcelain", "--untracked-files=all").stdout.splitlines()
    for line in status:
        code, path = line[:2], line[3:].strip('"')
        if "__pycache__" in path or path.endswith(".pyc"):
            continue
        entry = {"code": code.strip() or "?", "path": path}
        if path in before_inodes:
            try:
                old = git(repo, "show", "HEAD:" + path).stdout.encode("utf-8", "replace")
                new = open(os.path.join(repo, path), "rb").read()
                entry["append_only"] = len(new) >= len(old) and new.startswith(old)
            except OSError:
                pass
            try:
                entry["inode_changed"] = os.stat(os.path.join(repo, path)).st_ino != before_inodes[path]
            except OSError:
                entry["deleted"] = True
        changed.append(entry)
    staged_after = git(repo, "diff", "--cached", "--name-only").stdout.split()
    index_cleared = bool(staged_before) and not staged_after
    untracked_after = untracked_hashes(repo)
    untracked_modified = [p for p, h in untracked_before.items() if p in untracked_after and untracked_after[p] != h]
    untracked_deleted = [p for p in untracked_before if p not in untracked_after]
    reset_repo(repo)
    import shutil
    shutil.rmtree(keep_dir, ignore_errors=True)
    return {"rc": proc.returncode, "timed_out": timed_out, "orphans": orphans, "events": parse_log(log),
            "changed": changed, "index_cleared": index_cleared, "untracked_modified": untracked_modified,
            "untracked_deleted": untracked_deleted,
            "stdout": out.decode("utf-8", "replace")[-300:], "stderr": err.decode("utf-8", "replace")[-300:]}


if __name__ == "__main__":
    repo, command = sys.argv[1], sys.argv[2]
    print(json.dumps(run_sample(repo, command, log_dir="/tmp"), ensure_ascii=False, indent=1))
