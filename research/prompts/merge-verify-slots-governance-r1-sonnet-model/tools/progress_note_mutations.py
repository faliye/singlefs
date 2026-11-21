#!/usr/bin/env python3
"""把 progress_note 读进度文件的各支与模块名逐个弄坏，量 WatchTests.test_progress_deleted_before_read_is_reported 红不红。
用法：progress_note_mutations.py <副本根> [--dry-run]
  <副本根> 是 minimal-copy.py 建的、带 scripts/agents 的仓副本；本脚本只改副本里的 agent_watch_alerts.py、agent-watch.py，每个弄坏项跑完照原字节换回，收尾核哈希。
  --dry-run：只照每项把改动施加到内存里核「旧串恰好命中」并打出改动，不起测试。
每一项一行：项名<TAB>预期<TAB>结局（OK=Ran 1 test 过、FAIL=断言红、ERROR=用例报错）。预期与结局不同的行末加 DIFFERENT。退出码：0 全部与预期相同；1 有不同；2 用法错或旧串没命中。"""
import hashlib, os, signal, subprocess, sys

ALERTS = "scripts/agents/agent_watch_alerts.py"
WATCH = "scripts/agents/agent-watch.py"
MESSAGE = "；progress.md 在检查期间已消失或不可读"

def replace_nth(text, old, new, nth, expect_count):
    if text.count(old) != expect_count:
        raise SystemExit(f"旧串命中 {text.count(old)} 次，要 {expect_count} 次：{old!r}")
    position = -1
    for _ in range(nth):
        position = text.index(old, position + 1)
    return text[:position] + new + text[position + len(old):]

def mutations():
    """(项名, 预期, 环境变量, {文件: 改法函数}, 要改名的文件对或 None)"""
    first_message = lambda t: replace_nth(t, f'return "{MESSAGE}"', 'return "；X"', 1, 2)
    second_message = lambda t: replace_nth(t, f'return "{MESSAGE}"', 'return "；X"', 2, 2)
    return [
        ("基线：不弄坏", "OK", {}, {}, None),
        ("M1 moment 为 None 那一支的句子改掉", "FAIL", {}, {ALERTS: first_message}, None),
        ("M2 except OSError 那一支的句子改掉", "OK", {}, {ALERTS: second_message}, None),
        ("M3 去掉 if moment is None 那一支", "ERROR", {}, {ALERTS: lambda t: replace_nth(t, "        if moment is None:\n", "        if False:\n", 1, 1)}, None),
        ("M4 readable_mtime 读不到时交 0.0 不交 None", "OK", {}, {ALERTS: lambda t: replace_nth(t, "    except OSError:\n        return None\n", "    except OSError:\n        return 0.0\n", 1, 1)}, None),
        ("M5 except OSError 改成 except KeyError", "OK", {}, {ALERTS: lambda t: replace_nth(t, "    except OSError:\n        return \"" + MESSAGE + "\"\n    return (f", "    except KeyError:\n        return \"" + MESSAGE + "\"\n    return (f", 1, 1)}, None),
        ("M6 末几行取头不取尾（平常路径）", "OK", {}, {ALERTS: lambda t: replace_nth(t, "[-PROGRESS_TAIL_LINES:]", "[:PROGRESS_TAIL_LINES]", 1, 1)}, None),
        ("M7 path 不取 progress_file_of", "FAIL或ERROR", {}, {ALERTS: lambda t: replace_nth(t, "    path = progress_file_of(transcript)\n    if path is None and BROKEN", "    path = None\n    if path is None and BROKEN", 1, 1)}, None),
        ("M8 弄坏开关 AGENT_WATCH_BREAK=progressnote", "FAIL", {"AGENT_WATCH_BREAK": "progressnote"}, {}, None),
        ("N1 模块改名（agent-watch.py 载入的名字不再是 agent_watch_alerts）", "ERROR", {},
         {WATCH: lambda t: t.replace("agent_watch_alerts", "agent_watch_alert_handlers")}, (ALERTS, "scripts/agents/agent_watch_alert_handlers.py")),
    ]

def run_test(root, environment):
    command = [sys.executable, "-B", "-m", "unittest", "discover", "-s", "scripts/agents/tests", "-p", "test_agents.py", "-k", "test_progress_deleted_before_read_is_reported"]
    env = {key: value for key, value in os.environ.items() if not key.endswith("_BREAK")}
    env.update(environment)
    process = subprocess.Popen(command, cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
    try:
        output, _ = process.communicate(timeout=300)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        output, _ = process.communicate()
        return "TIMEOUT", output
    if process.returncode == 0 and "Ran 1 test" in output:
        return "OK", output
    if "errors=" in output:
        return "ERROR", output
    if "failures=" in output:
        return "FAIL", output
    return f"OTHER(rc={process.returncode})", output

def main():
    if len(sys.argv) < 2 or sys.argv[1].startswith("--"):
        raise SystemExit(2)
    root = os.path.realpath(sys.argv[1])
    dry = "--dry-run" in sys.argv[2:]
    paths = [ALERTS, WATCH]
    originals = {}
    for path in paths:
        with open(os.path.join(root, path), "rb") as handle:
            originals[path] = handle.read()
    digests = {path: hashlib.sha256(data).hexdigest() for path, data in originals.items()}
    different = 0
    for name, expected, environment, edits, rename in mutations():
        try:
            if dry:
                for path, edit in edits.items():
                    changed = edit(originals[path].decode("utf-8"))
                    if changed == originals[path].decode("utf-8"):
                        raise SystemExit(f"{name}: 改动没改变文本")
                print(f"{name}\t{expected}\t（dry-run：旧串命中核过）")
                continue
            for path, edit in edits.items():
                with open(os.path.join(root, path + ".new"), "w", encoding="utf-8") as handle:
                    handle.write(edit(originals[path].decode("utf-8")))
                os.replace(os.path.join(root, path + ".new"), os.path.join(root, path))
            if rename:
                os.replace(os.path.join(root, rename[0]), os.path.join(root, rename[1]))
            outcome, output = run_test(root, environment)
        finally:
            if not dry:
                if rename and os.path.exists(os.path.join(root, rename[1])):
                    os.replace(os.path.join(root, rename[1]), os.path.join(root, rename[0]))
                for path in paths:
                    with open(os.path.join(root, path + ".new"), "wb") as handle:
                        handle.write(originals[path])
                    os.replace(os.path.join(root, path + ".new"), os.path.join(root, path))
        if dry:
            continue
        accepted = expected.split("或")
        flag = "" if outcome in accepted else "\tDIFFERENT"
        different += bool(flag)
        print(f"{name}\t预期 {expected}\t结局 {outcome}{flag}")
        for line in output.splitlines()[-4:]:
            print(f"    | {line[:200]}")
    for path in paths:
        with open(os.path.join(root, path), "rb") as handle:
            if hashlib.sha256(handle.read()).hexdigest() != digests[path]:
                print(f"✗ {path} 没照原字节换回")
                raise SystemExit(1)
    print("收尾：两份文件的哈希与开跑时相同")
    raise SystemExit(1 if different else 0)

main()
