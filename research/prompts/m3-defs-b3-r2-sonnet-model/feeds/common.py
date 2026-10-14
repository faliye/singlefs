# 共用：建前后两棵树、喂派发闸 / 重型闸 / 写闸，打 name=… 行。只读冻结副本，不碰主工作区。
import json, os, re, shutil, subprocess, sys, tempfile

COPY_ITEMS = [".claude", "scripts", "records", "crates", ".cargo", "CLAUDE.md", "Cargo.toml", "Cargo.lock"]

def build_tree(frozen, dest):
    """把冻结副本的 .claude / scripts / records / crates 拷成一棵可喂钩子的树（不拷 multi-host.env、research/ 大目录、target、__pycache__）。"""
    os.makedirs(os.path.join(dest, "research", "prompts"), exist_ok=True)
    for item in COPY_ITEMS:
        src = os.path.join(frozen, item)
        if os.path.isdir(src):
            subprocess.run(["rsync", "-a", "--exclude", "__pycache__", "--exclude", "target", src, dest + "/"], check=True)
        elif os.path.isfile(src):
            shutil.copy2(src, dest)
    table = os.path.join(frozen, "research", "prompts", "abandoned-rounds.tsv")
    if os.path.isfile(table):
        shutil.copy2(table, os.path.join(dest, "research", "prompts"))
    return dest

KITCHEN = {
    "implementation-writer": "要动的 crates 文件：crates/x/src/a.rs\n条款：x\n",
    "mutation-triage": "变异表：scripts/mutation/tables/x.tsv\n",
    "experiment-runner": "这一段回答的岔路：岔路单第 1 行\n自证在哪跑：不跑\n",
    "experiment-designer": "被测条款：x\n岔路单：x\n",
    "investigator": "现象：x\n",
    "three-way-attack": "禁读清单：x\n背景材料：x\n攻击面：x\n",
    "tooling-writer": "出口：x\n要改的文件：x\n",
    "clerk": "",
}

def prompt(typ, lines="", scratch="~/.local/share/singlefs/work/r2s-x", wall="预计挂钟：2 小时内\n"):
    """一份「除了被测那几行之外都齐」的派发提示。lines 是被测的行（线程上限、预计挂钟、多机……）。"""
    return (f"# 派发：{typ}\n\n重型测试：不跑\n开工先读 `.claude/agent-common.md`\n批次：records/2026-10-12-里程碑三工具与治理批.md#批4第二轮三方\n"
            f"草稿目录：{scratch}\n报告：{scratch}/report.md\n条款：x\n" + KITCHEN.get(typ, "") + (wall if "预计挂钟" not in lines else "") + lines)

def tags(text):
    out = []
    for line in text.splitlines():
        if not line.startswith("✗"):
            continue
        if "线程上限" in line: out.append("15")
        elif "预计挂钟" in line or "长活" in line or "分片" in line or "多机" in line: out.append("14")
        elif "开工快照" in line: out.append("6")
        elif "处不成立" in line: continue
        else: out.append("other:" + line[:40])
    return sorted(set(out))

def check_prompt(tree, typ, text, env=None, workdir=None):
    """--check-prompt 喂一份提示：返回 (退出码, 拒绝标签集合, stderr+stdout 合并文本)。"""
    d = workdir or tempfile.mkdtemp(prefix="r2s-")
    path = os.path.join(d, "p.md")
    with open(path, "w") as f: f.write(text)
    e = dict(os.environ); e.pop("SINGLEFS_THREAD_CAP", None)
    if env: e.update(env)
    r = subprocess.run(["bash", f"{tree}/.claude/hooks/runner-dispatch-guard.sh", "--check-prompt", path, "--type", typ, "--root", tree],
                       capture_output=True, text=True, env=e)
    out = r.stdout + r.stderr
    return r.returncode, tags(out), out

def stdin_hook(tree, hook, payload, env=None):
    e = dict(os.environ); e["CLAUDE_PROJECT_DIR"] = tree
    e.pop("SINGLEFS_THREAD_CAP", None)
    if env: e.update(env)
    r = subprocess.run(["bash", f"{tree}/.claude/hooks/{hook}"], input=json.dumps(payload), capture_output=True, text=True, env=e)
    return r.returncode, r.stdout + r.stderr

def row(case_id, case, rc, extra=""):
    print(f"name={case_id} case={case} rc={rc}{(' ' + extra) if extra else ''}")

def both(case_id, case, fn, trees):
    """对 (before, after) 两棵树各跑 fn(tree) -> (rc, detail)，打一行 name=… before=… after=…。"""
    cells = []
    for label, tree in trees:
        rc, detail = fn(tree)
        cells.append(f"{label}_rc={rc} {label}_detail={detail}")
    print(f"name={case_id} case={case} " + " ".join(cells))

def parse_trees(argv):
    """argv: <before树> <after树> <工作目录>"""
    before, after, work = argv[1], argv[2], argv[3]
    os.makedirs(work, exist_ok=True)
    return [("before", before), ("after", after)], work
