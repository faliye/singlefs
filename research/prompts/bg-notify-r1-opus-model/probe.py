#!/usr/bin/env python3
"""bg-notify-r1 攻方腿的模型：每条形态同时问两件事。

  hook 记不记：把 shapes.py 里 hook 那一列喂给仓里真的 `.claude/hooks/bash-command-detector.sh`
               （标准输入 JSON，AGENT_HOOK_DETECTIONS 指到临时文件），看第三种检出有没有落进文件。
  脱不脱钩：    把 exec 那一列放进模拟 harness 的外层 shell 真跑——
               `bash -c 'set -m; eval "$命令" < /dev/null && pwd -P >| cwd.out'`，新会话起（harness 的外层 shell
               是会话首进程，$- 里有 m，标准输出是文件；本腿 2026-09-18 在真 harness 里现查），
               量外层 shell 退出的时刻（= harness 发完成通知的时刻）与每件活写 done 标记的时刻。
               有一件活在外层 shell 退出 0.5 秒之后才完 ⇒ 脱钩。

  用法：python3 probe.py <仓根> <草稿目录>        # 打一张表，退出码 0
        python3 probe.py <仓根> <草稿目录> --only X01,F02
"""
import json, os, re, subprocess, sys, tempfile, time
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from shapes import SHAPES

THIRD = "run_in_background 里又自己放后台"
WORK = '#!/usr/bin/env bash\nsleep "$2"\ndate +%s.%N > "$(dirname "$0")/done.$1"\n'


def hook_verdict(hook_path, text, background):
    with tempfile.TemporaryDirectory() as work:
        detections = os.path.join(work, "detections.jsonl")
        environment = {k: v for k, v in os.environ.items() if not k.startswith("BASH_COMMAND_DETECTOR_")}
        environment["AGENT_HOOK_DETECTIONS"] = detections
        payload = json.dumps({"tool_name": "Bash", "session_id": "bg-notify-r1-opus-model",
                              "tool_input": {"command": text, "run_in_background": background}})
        completed = subprocess.run(["bash", hook_path], input=payload, text=True, capture_output=True, env=environment)
        findings = []
        if os.path.exists(detections):
            for line in open(detections, encoding="utf-8"):
                findings += json.loads(line)["findings"]
        return completed.returncode, findings


def expand(template, directory, shape_id):
    text = template.replace("%%DIR%%", directory).replace("%%ID%%", shape_id)
    return re.sub(r"%%([a-z]+):([0-9]+)%%", lambda m: f"bash {directory}/work.sh {m.group(1)} {m.group(2)}", text)


def job_names(template):
    names = re.findall(r"%%([a-z]+):[0-9]+%%", template) + re.findall(r"work\.sh'?,? '?([a-z]+)'?,? '?[0-9]", template)
    if "xargs" in template:
        names += ["a", "b", "c"]
    if "for stage in a b c" in template:
        names += ["a", "b", "c"]
    return sorted(set(names))


def exec_verdict(draft, shape):
    if shape["exec"] is None:
        return None
    directory = tempfile.mkdtemp(prefix=f"probe-{shape['id']}-", dir=draft)
    with open(os.path.join(directory, "work.sh"), "w") as handle:
        handle.write(WORK)
    command = expand(shape["exec"], directory, shape["id"])
    names = job_names(shape["exec"])
    wrapper = 'set -m; eval "$BGN_COMMAND" < /dev/null && pwd -P >| cwd.out'
    started = time.time()
    with open(os.path.join(directory, "task.output"), "w") as output:
        process = subprocess.Popen(["bash", "-c", wrapper], cwd=directory, stdin=subprocess.DEVNULL, stdout=output,
                                   stderr=subprocess.STDOUT, env=dict(os.environ, BGN_COMMAND=command), start_new_session=True)
        process.wait()
    shell_exit = time.time() - started
    deadline = started + 20
    while time.time() < deadline and not all(os.path.exists(os.path.join(directory, f"done.{name}")) for name in names):
        time.sleep(0.1)
    done = {}
    for name in names:
        path = os.path.join(directory, f"done.{name}")
        done[name] = round(float(open(path).read()) - started, 1) if os.path.exists(path) else None
    if "tmux" in shape["exec"]:
        subprocess.run(["tmux", "-L", f"bgnr1-{shape['id']}", "kill-server"], capture_output=True)
    outlived = [name for name, at in done.items() if at is None or at > shell_exit + 0.5]
    return dict(shell_exit=round(shell_exit, 1), done=done, outlived=outlived, exit_code=process.returncode)


def main():
    repo, draft = sys.argv[1], sys.argv[2]
    only = set(sys.argv[4].split(",")) if len(sys.argv) > 4 and sys.argv[3] == "--only" else None
    hook_path = os.path.join(repo, ".claude/hooks/bash-command-detector.sh")
    shapes = [shape for shape in SHAPES if only is None or shape["id"] in only]
    with ThreadPoolExecutor(max_workers=len(shapes)) as pool:
        executions = list(pool.map(lambda shape: exec_verdict(draft, shape), shapes))
    print("| 编号 | run_in_background | hook 第三种记了没有 | hook 全部检出 | 外层 shell 退出（秒） | 各件活完成（秒） | 外层退出之后才完的活 | 判定 |")
    print("|---|---|---|---|---|---|---|---|")
    for shape, execution in zip(shapes, executions):
        exit_code, findings = hook_verdict(hook_path, shape["hook"], shape["bg"])
        third = any(finding.startswith(THIRD) for finding in findings)
        short = "；".join(finding.split("（")[0] for finding in findings) or "无"
        if execution is None:
            verdict = ("hook 记" if third else "hook 不记") + "；没真跑"
            print(f"| {shape['id']} | {shape['bg']} | {'记' if third else '不记'} | {short} | — | — | — | {verdict} |")
            continue
        detached = bool(execution["outlived"])
        verdict = {(True, True): "记且脱钩（对）", (False, False): "不记不脱钩（对）",
                   (False, True): "漏检", (True, False): "误检"}[(third, detached)]
        done = ", ".join(f"{name}={at}" for name, at in execution["done"].items())
        print(f"| {shape['id']} | {shape['bg']} | {'记' if third else '不记'} | {short} | {execution['shell_exit']} | {done} | "
              f"{','.join(execution['outlived']) or '无'} | {verdict} |")
    return 0


if __name__ == "__main__":
    sys.exit(main())
