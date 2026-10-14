#!/usr/bin/env python3
"""T3-A：同一句「多机」写法，派发闸 ⑭（登记值读法）与脚本文件头的判法（现算读法：detached-run.sh long-run-check、host-dispatch.py ⓪）判法相反。
用法：python3 -I t3_long_run.py <树> <工作目录> <标签>；树是冻结副本 .claude 与 scripts 的拷贝。打印 name=… 行，路径换成 <WORK>、<TREE>。"""
import os, subprocess, sys

tree, work, label = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(work, exist_ok=True)
SHARDS = "按登记的取样点切片，片数取多机配置的台数"
RESUME = "同一条命令重起，按产物里已写完的片号跳过"
HOSTS = {
    # r3 定义 experiment-runner.md 1d 与骨架 dispatch-prompt.py 的写法：台数取多机配置、每台线程取登记值、不现算
    "registered": "台数从 scripts/crash/layer0-shard-configuration-check.sh --emit-hosts 读、每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量",
    # r1 定义 1d 的写法：每台的线程与内存读 host-resources.py --json 现算
    "live": "台数读 scripts/crash/layer0-shard-configuration-check.sh --emit-hosts，每台的线程与内存读 scripts/hosts/host-resources.py --json 现算，不写常量",
    # 照登记值做、文件头只在否定句里提一下 host-resources.py
    "registered-negated": "台数从 scripts/crash/layer0-shard-configuration-check.sh --emit-hosts 读、每台线程取 .claude/heavy-thread-default 的登记值，不调 host-resources.py 现算",
}

def clean(text):
    return text.replace(work, "<WORK>").replace(tree, "<TREE>").strip().splitlines()

def first_cross(text):
    for line in clean(text):
        if "✗" in line:
            return line.strip()[:260]
    return "-"

sys.path.insert(0, os.path.join(tree, ".claude", "hooks"))
import lib_long_run  # noqa: E402

for name, hosts in HOSTS.items():
    script = os.path.join(work, f"long-{name}.sh")
    with open(script, "w", encoding="utf-8") as handle:
        handle.write("#!/usr/bin/env bash\n"
                     f"# long-run-shards: {SHARDS}\n# long-run-resume: {RESUME}\n# long-run-hosts: {hosts}\n"
                     "set -euo pipefail\necho run\n")
    # ① 脚本文件头：detached-run.sh long-run-check 调的同一个入口（lib_long_run.py script），host-dispatch.py ⓪ 调的同一个 judge
    done = subprocess.run([sys.executable, "-I", os.path.join(tree, ".claude", "hooks", "lib_long_run.py"), "script", work, "--", "bash", script],
                          capture_output=True, text=True, env=dict(os.environ, LIB_LONG_RUN_BREAK=""))
    print(f"name=header tree={label} hosts={name} rc={done.returncode} first={first_cross(done.stderr) if done.returncode else clean(done.stdout)[0][:120]}")
    problems = lib_long_run.judge(lib_long_run.script_header_declaration(script), lib_long_run.HEADER_SPELLING)
    print(f"name=host-dispatch-header tree={label} hosts={name} problems={len(problems)} verdict={'single' if problems else 'multi'}")
    # ② detached-run.sh long-run-check 本身（在树里起）
    checked = subprocess.run(["bash", os.path.join(tree, "scripts", "hosts", "detached-run.sh"), "long-run-check", "--", "bash", script],
                             capture_output=True, text=True, cwd=tree)
    print(f"name=long-run-check tree={label} hosts={name} rc={checked.returncode} first={first_cross(checked.stderr) if checked.returncode else '-'}")
    # ③ 派发闸 ⑭：同一句写进派 experiment-runner 的提示「多机：」一行
    prompt = os.path.join(work, f"prompt-{name}.md")
    with open(prompt, "w", encoding="utf-8") as handle:
        handle.write("# 派发：experiment-runner\n\n重型测试：不跑\n线程上限：31\n"
                     f"草稿目录：{work}/runner-{name}/\n报告：{work}/runner-{name}/report.md\n"
                     "这一段回答的岔路：岔路单第 1 行\n预计挂钟：超过 2 小时，按上一次同规模的用时推\n"
                     f"分片：{SHARDS}\n续跑：{RESUME}\n多机：{hosts}\n\n## 关的是哪几条\n\n照跑前登记跑产物。\n")
    guarded = subprocess.run(["bash", os.path.join(tree, ".claude", "hooks", "runner-dispatch-guard.sh"), "--check-prompt", prompt,
                              "--type", "experiment-runner", "--root", tree], capture_output=True, text=True,
                             env=dict(os.environ, CLAUDE_PROJECT_DIR=tree, AGENT_HOOK_DETECTIONS=os.path.join(work, "detections.jsonl")))
    long_lines = [line for line in clean(guarded.stderr + guarded.stdout) if "长活" in line or "多机" in line and "✗" in line]
    print(f"name=dispatch-14 tree={label} hosts={name} rc={guarded.returncode} long_run_refusals={sum('✗' in l for l in long_lines)} first={long_lines[0][:200] if long_lines else '-'}")
