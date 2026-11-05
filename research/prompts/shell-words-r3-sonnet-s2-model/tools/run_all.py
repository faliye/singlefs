#!/usr/bin/env python3
"""跑一批样本：① 起真 bash 观测（串行，共用一棵树）② 按 policy.py + 下面几条写盘/循环规则算该拒不该拒 ③ 喂两份钩子（并行）④ 逐条写 out/<批>.jsonl。
用法：run_all.py --repo R --copy C --work D --corpus corpus_launch,corpus_forms --out out/x.jsonl [--only 前缀]"""
import argparse, concurrent.futures as cf, importlib, json, os, re, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import feed, oracle, policy
import corpus_base

REPO_DIRS = ("crates/", "scripts/", ".claude/", "research/", "records/", "litmus/", "briefs/")


def subst(text, repo, copy, work, uid=""):
    text = text.replace("{I}", uid)
    return text.replace("{R}", repo).replace("{C}", copy).replace("{D}", work).replace("{B}", os.path.join(HERE, "bin")).replace("{W}", os.path.join(work, "wr")).replace("{U}", os.path.join(work, "u", uid))


SED_I = re.compile(r"\bsed\s+(-[A-Za-z]*i|--in-place)|\bperl\s+-[A-Za-z]*i")
PY_WRITE = re.compile(r"open\(|write_text|write_bytes|os\.replace|os\.rename|shutil\.move")
GIT_INDEX = re.compile(r"\breset\b|\brestore\b")
W7_LISTED = re.compile(r">|\btee\b|\bcp\b|\bdd\b|\btruncate\b|open\(")   # ⑦ 点名的写法


def extra_reasons(sample, obs):
    """写盘、等待循环、git 索引这几类：期望来自真 bash 留下的痕迹，再按 agent-common 与检出钩子文件头点名的写法收窄。
    子 agent 里 three-way-forward 在写范围表里没有行（⑫ 不判）、也没有 Edit（⑧ 不判）；implementation-writer 的写范围是 crates/**、litmus/**、工作根。"""
    out, role, cmd = [], sample["role"], sample["cmd"]
    if not sample["bg"] and obs["timed_out"] and re.search(r"\b(until|while)\b", cmd) and "sleep" in cmd:
        out.append(("D2", "前台等待循环真的没停（真 bash 跑满超时）"))
    for ch in obs["changed"]:
        path = ch["path"]
        if role == "editor" and path.startswith(REPO_DIRS):
            if re.search(r"\bsed\b.*\s-[A-Za-z]*i|\bsed\s+--in-place", cmd) or (PY_WRITE.search(cmd) and "python" in cmd):
                out.append(("W8", f"有 Edit 的子 agent 在 Bash 里就地改了仓内文件：{path}"))
            if not path.startswith(("crates/", "litmus/")):
                out.append(("W12", f"子 agent 往写范围外的仓内文件落了改动：{path}"))
        if ch["code"] == "M" and ch.get("inode_changed") is False and not ch.get("append_only") and path.endswith((".sh", ".py")) and W7_LISTED.search(cmd):
            out.append(("W7", f"在同一个 inode 上改了已有脚本：{path}"))
    for p in obs["untracked_modified"]:
        if p.startswith("research/results/") and re.search(r">|\btee\b|\bcp\b|\bmv\b|\bdd\b|\btruncate\b|\binstall\b", cmd) and not obs_append_only(sample, obs, p):
            out.append(("W5", f"整份覆盖 research/results 下未跟踪产物：{p}"))
    if obs["index_cleared"] and GIT_INDEX.search(cmd):
        out.append(("G15", "暂存区非空时清了整个索引"))
    return out


def obs_append_only(sample, obs, path):
    """追加（>>、tee -a）不拦：命令文本里只有追加写法、没有整份覆盖写法"""
    cmd = sample["cmd"]
    cmd = re.sub(r"\d*>\s*/dev/null|\d*>&\d", "", cmd)
    appends = len(re.findall(r">>|tee\s+-a", cmd))
    overwrites = len(re.findall(r"(?<![>&])>(?![>&])|&>(?!>)|>\||\btee\b(?!\s+-a)|\bcp\b|\bmv\b|\bdd\b|\btruncate\b|\binstall\b", cmd))
    return appends > 0 and overwrites == 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo"); ap.add_argument("--copy"); ap.add_argument("--work"); ap.add_argument("--corpus"); ap.add_argument("--out")
    ap.add_argument("--only", default=""); ap.add_argument("--stride", type=int, default=1); ap.add_argument("--ids", default=""); ap.add_argument("--shard", default="0/1"); ap.add_argument("--slice", default="0:100000"); ap.add_argument("--jobs", type=int, default=1)
    a = ap.parse_args()
    for name in a.corpus.split(","):
        importlib.import_module(name)
    samples = [s for s in corpus_base.SAMPLES if s["id"].startswith(a.only)]
    samples = samples[::a.stride]
    if a.ids:
        wanted = set(open(a.ids).read().split())
        samples = [s for s in samples if s["id"] in wanted]
    lo, hi = map(int, a.slice.split(":")); samples = samples[lo:hi]
    i, n = map(int, a.shard.split("/")); samples = samples[i::n]
    logdir = os.path.join(a.work, "oraclelog"); os.makedirs(logdir, exist_ok=True)
    observed, fed = [], []
    import subprocess, shutil
    for s in samples:
        uid = s["id"]
        udir = os.path.join(a.work, "u", uid)
        shutil.rmtree(udir, ignore_errors=True); os.makedirs(udir)
        cmd = subst(s["cmd"], a.repo, a.copy, a.work, uid)
        cwd = a.repo if s["cwd"] == "R" else (a.copy if s["cwd"] == "C" else subst(s["cwd"], a.repo, a.copy, a.work, uid))
        oracle.reset_repo(a.repo)
        if s["setup"]:
            subprocess.run(["bash", "-c", subst(s["setup"], a.repo, a.copy, a.work, uid)], cwd=a.repo, capture_output=True, timeout=30,
                           env={**os.environ, "ORACLE_LOG": "/dev/null"})
            if s["stage"]:
                oracle.git(a.repo, "add", "-A")
        fed.append(feed.feed(a.repo, a.work, cmd, role=s["role"], background=s["bg"], cwd=cwd))
        obs = oracle.run_sample(a.repo, cmd, cwd=cwd, setup="", log_dir=logdir, sample_id=uid, stage=s["stage"], reset_first=False, timeout=s.get("timeout", 30))
        observed.append((s, cmd, cwd, obs))
        shutil.rmtree(udir, ignore_errors=True)
        # 仓副本只留 Cargo.toml 与 crates/ 下的清单：样本往里写的（new.rs、被 sed -i 改过的 Cargo.toml）清掉
        for name in os.listdir(a.copy):
            if name not in ("Cargo.toml", "crates"):
                p = os.path.join(a.copy, name)
                shutil.rmtree(p, ignore_errors=True) if os.path.isdir(p) else os.unlink(p)
        shutil.copy(os.path.join(a.repo, "Cargo.toml"), os.path.join(a.copy, "Cargo.toml"))
    with open(a.out, "w") as fo:
        for (s, cmd, cwd, obs), f in zip(observed, fed):
            if s["want"] is None:
                want = policy.expected_rejections(s["role"], s["bg"], obs, a.repo, a.copy) + extra_reasons(s, obs)
                basis = "观测+policy"
            else:
                want = [tuple(w) for w in s["want"]]
                basis = "手标"
            rec = {"id": s["id"], "fam": s["fam"], "role": s["role"], "bg": s["bg"], "cmd": cmd, "cwd": cwd, "basis": basis,
                   "note": s["note"], "want_reject": bool(want), "want": want, "hook": feed.verdict(f),
                   "hook_detail": {k: {"rc": v["rc"], "first": v["stderr"].strip().split("\n")[0][:200]} for k, v in f.items()},
                   "obs": {"events": [(e["name"], e["args"][:8], e["env"].get("HEAVY"), e["env"].get("MEMCAP")) for e in obs["events"]],
                           "rc": obs["rc"], "timed_out": obs["timed_out"], "orphans": obs["orphans"], "changed": obs["changed"],
                           "stderr": obs["stderr"][-120:]}}
            fo.write(json.dumps(rec, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
