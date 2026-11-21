#!/usr/bin/env python3
"""no-task-numbers-governance-r3 攻方腿的探针：判据库 task-and-session-numbers 那一形态逐格判、喂两道钩子、扫现存真行。

用法：python3 cases.py <仓根> <运行目录>
  判据库、写闸、检出钩子都取 <仓根> 下的那一份（被判文件现读）；钩子只喂 stdin、不执行命令，什么都不往仓里写。
  每格打一行 name=case label=… want=… got=… agree=…，want 是这条腿推的或量的结局；末行之前打 name=case_summary cases=N disagreements=M。
  扫描行 name=scan_* 是现存真行的计数（没有 want，供判决引用）；逐行明细落 <运行目录>/scan-*.txt。
退出码：0 跑完且全部 agree；1 有不一致；2 用法错或读不到判据库。
"""
import importlib.util
import json
import os
import re
import subprocess
import sys

if len(sys.argv) != 3:
    print("用法：cases.py <仓根> <运行目录>", file=sys.stderr)
    sys.exit(2)
REPO, WORK = os.path.realpath(sys.argv[1]), os.path.realpath(sys.argv[2])
LIBRARY = os.path.join(REPO, ".claude", "gate.d", "lib-forbidden-notations.py")
FORM = "task-and-session-numbers"
RECORD = "records/2026-11-19-r3-opus-probe.md"
VERDICT = "research/prompts/x-main-verification.md"
LIVE = os.path.join(REPO, "research/prompts/no-task-numbers-governance-r3-opus-model/live-rerun")
try:
    specification = importlib.util.spec_from_file_location("lib_forbidden_notations", LIBRARY)
    notations = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(notations)
except Exception as error:
    print(f"name=library_unreadable error={error!r}")
    sys.exit(2)

results = []


def record(label, want, got, detail=""):
    agree = want == got
    results.append(agree)
    print(f"name=case label={label} want={want} got={got} agree={'yes' if agree else 'no'} detail={detail}")


def library_case(label, path, before, after, want):
    hits = notations.new_violation_hits(FORM, before, after, path)
    shown = ";".join(f"{number}:{name}:{matched}" for number, _, name, matched in hits)
    record(label, want, "refuse" if hits else "pass", shown)


def hook_case(label, hook, payload, want):
    environment = dict(os.environ, CLAUDE_PROJECT_DIR=REPO, AGENT_HOOK_DETECTIONS=os.path.join(WORK, "detections.jsonl"))
    done = subprocess.run(["bash", os.path.join(REPO, ".claude", "hooks", hook)], input=json.dumps(payload).encode(),
                          capture_output=True, env=environment, cwd=REPO, timeout=120)
    got = {0: "pass", 2: "refuse"}.get(done.returncode, f"exit{done.returncode}")
    first = (done.stderr.decode("utf-8", "replace").strip().splitlines() or [""])[0][:160]
    record(label, want, got, first.replace(" ", "_"))


# 关 1：新判法下照样写得进的近似写法（records/ 下，路径照判得最严的那一类）
library_case("records:mainagent加两位短号", RECORD, "", "mainagent(5b) 在修 `.claude/hooks/write-guard.sh`，subagent(c3) 交回\n", "pass")
library_case("records:撤回的agent加两位短号", RECORD, "", "agent(cc) 认领了；问 agent(4a) 回信\n", "pass")
library_case("records:括号里会话词在短号前", RECORD, "", "这一轮剩下的活交给里程碑三收尾那一轮（会话 4a，2026-11-19）\n", "pass")
library_case("records:括号短号对照", RECORD, "", "这一轮剩下的活交给里程碑三收尾那一轮（4a）\n", "refuse")
library_case("records:截止批次写成中文数字", RECORD, "", "截止批次为三。\n", "pass")
library_case("records:截止批次写成第几批对照", RECORD, "", "截止批次：第 3 批。\n", "refuse")
library_case("records:批后面跟中文数字", RECORD, "", "它的批三在修\n", "pass")
library_case("records:任务编号", RECORD, "", "任务编号 12345 退 0\n", "pass")
library_case("records:号任务", RECORD, "", "12345 号任务卡住\n", "pass")
library_case("records:做什么的会话", RECORD, "", "管任务服务的会话认领了；改检出钩子那个会话回信\n", "pass")
library_case("records:那一路的工具员", RECORD, "", "它那一路的工具员建的\n", "pass")
library_case("records:那一路后面跟会话动作词", RECORD, "", "里程碑四收口的那一路回信；GPU 垃圾回收那一路认领；task-server 那一路转达\n", "pass")
library_case("records:那一路会话对照", RECORD, "", "改检出钩子那一路会话回信\n", "refuse")
# 关 1：整份写前写后相抵按行文，已有违例的行带着别的号出围栏、离开批次列不拒
library_case("records:已有任务号的行出围栏", RECORD, "前文\n~~~\n第 6 批 交回（任务 12345）\n~~~\n后文\n",
             "前文\n第 6 批 交回（任务 12345）\n~~~\n后文\n", "pass")
library_case("records:出围栏对照", RECORD, "前文\n~~~\n第 6 批 交回\n~~~\n后文\n", "前文\n第 6 批 交回\n~~~\n后文\n", "refuse")
library_case("records:已有违例的行改表头离开批次列", RECORD, "| 批次 | 依赖 |\n|---|---|\n| 第 6 批 | 依赖第 5 批 |\n",
             "| 批 | 依赖 |\n|---|---|\n| 第 6 批 | 依赖第 5 批 |\n", "pass")
library_case("records:改表头对照", RECORD, "| 批次 | 依赖 |\n|---|---|\n| 第 6 批 | 工具员交回 |\n",
             "| 批 | 依赖 |\n|---|---|\n| 第 6 批 | 工具员交回 |\n", "refuse")
# 关 1：门禁 doc-text 新格对搬家、改名的文件基准取空串（cell 里 base_text 按新路径取）
old_record = "- 交回（任务 12345）\n- 第 6 批做完\n- 没有号的一行\n"
library_case("doc-text:改名后基准取空串", RECORD, "", old_record, "refuse")
library_case("doc-text:没改名对照", RECORD, old_record, old_record, "pass")
# 关 2：表头认列与 GFM 的切格不同：\\| 是转义的反斜杠加分隔符
library_case("records:反斜杠转义后的竖线", RECORD, "", "| 批次 | 依赖 |\n|---|---|\n| x\\\\| 第 6 批 |\n", "pass")
library_case("records:别的列批号对照", RECORD, "", "| 批次 | 依赖 |\n|---|---|\n| x | 第 6 批 |\n", "refuse")
# 关 2：A/B 读数：列表里的引用块、跨两行的两臂
library_case("verdict:列表里引用块的读数", VERDICT, "", "- > A/B 读数：任务 12345\n", "pass")
library_case("verdict:引用块的读数对照", VERDICT, "", "> A/B 读数：任务 12345\n", "refuse")
two_lines = "A/B 读数：任务 12345（有改法）\n任务 23456（没有改法）\n"
library_case("verdict:两臂分两行写", VERDICT, "", two_lines, "refuse")
library_case("dispatch:派发闸只判第一行", VERDICT, "", two_lines.split("\n")[0], "pass")

# 关 1：喂检出钩子 ⑰（主 agent 身份，不带 agent_type）：只判 stdin 里的命令，不执行
live = os.path.join(LIVE, "probe.md")
record_target = os.path.join(REPO, "records", "2026-11-19-r3-opus-probe.md")
for label, command, want in (
        ("detector:字面任务号对照", f"printf '%s\\n' '（任务 12345）退 0' >> {live}", "refuse"),
        ("detector:printf格式串拆开号", f"printf '（任务 %s）退 0\\n' 12345 >> {live}", "pass"),
        ("detector:赋过值的变量带号", f"n=12345; echo \"（任务 $n）退 0\" >> {live}", "pass"),
        ("detector:echo -e 转义空格", f"echo -e '批\\x20105 在修' >> {live}", "pass"),
        ("detector:printf格式串拆开号整份写新的records文件", f"printf '（任务 %s）退 0\\n' 12345 > {record_target}", "pass")):
    hook_case(label, "bash-command-detector.sh", {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": REPO}, want)

# 关 1：喂写闸（主 agent 身份）：Write 一份不在的 records/ 文件，只判不写
for label, content, want in (
        ("writeguard:字面任务号对照", "（任务 12345）退 0\n", "refuse"),
        ("writeguard:mainagent加两位短号", "mainagent(5b) 在修\n", "pass"),
        ("writeguard:截止批次写成中文数字", "截止批次为三。\n", "pass"),
        ("writeguard:括号里会话词在短号前", "交给里程碑三收尾那一轮（会话 4a，2026-11-19）\n", "pass")):
    hook_case(label, "write-guard.sh", {"tool_name": "Write", "tool_input": {"file_path": record_target, "content": content}, "cwd": REPO}, want)

print(f"name=case_summary cases={len(results)} disagreements={results.count(False)}")

# 关 3：现存真行扫描（记录、kb、规则、定义、技能与两份共用文件里判据库 session-descriptions 那一条的命中），逐行明细落运行目录
listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--", "records", ".claude/kb", ".claude/rules", ".claude/agents",
                         ".claude/skills", ".claude/agent-common.md", ".claude/main-agent.md"], cwd=REPO, capture_output=True, check=True)
paths = [path for path in listed.stdout.decode("utf-8").split("\0") if path and not notations.outside_judged_suffixes(FORM, path)]
description_name = next(name for key, name, _, _ in notations.FORMS_BY_KEY[FORM]["patterns"] if key == "session-descriptions")
VARIANT = re.compile(r"[^，。；、（(「 ]{2,14}(?:的)?(?:会话|那个会话)\s?(?:回信|来信|回话|答复|回复|认领|认了|认下|通知|转达|已答应|报|群发)")
GENERIC = re.compile(r"^(?:别的|本|主|这个|另一个|另一|其他|其它|各|两个|几个|每个|所有|全部|那个|同一个|新|旧|子|没有|发各)(?:的)?会话")
hit_lines, variant_count = 0, 0
with open(os.path.join(WORK, "scan-session-descriptions.txt"), "w", encoding="utf-8") as hits_out, \
        open(os.path.join(WORK, "scan-description-variants.txt"), "w", encoding="utf-8") as variants_out:
    for path in paths:
        text = open(os.path.join(REPO, path), encoding="utf-8", errors="replace").read()
        numbers = sorted({number for number, _, name, _ in notations.all_hits(FORM, text, path) if name == description_name})
        lines = text.split("\n")
        for number in numbers:
            hit_lines += 1
            line = lines[number - 1]
            start = max(line.find("那一路") - 20, 0)
            hits_out.write(f"{path}:{number}: {line[start:start + 50]}\n")
        for number, line in enumerate(lines, 1):
            for found in VARIANT.finditer(line):
                if "那一路" not in found.group(0) and not GENERIC.match(found.group(0)):
                    variant_count += 1
                    variants_out.write(f"{path}:{number}: {found.group(0)}\n")
print(f"name=scan_session_descriptions files={len(paths)} hit_lines={hit_lines}")
print(f"name=scan_description_variants_without_named_word occurrences={variant_count}")
sys.exit(0 if all(results) else 1)
