#!/usr/bin/env python3
"""攻方腿模型：照 Stop 钩子的 stdin 喂被判钩子 .claude/hooks/session-number-stop-check.sh，量正文第 4、5、6 问与第 7 问钩子一侧。

用法：python3 hook_probes.py <被判树> <运行目录>
  被判树：有 .claude/hooks/session-number-stop-check.sh 的那棵树（主工作区）。运行目录：已建好、可写。
每一例打一行：name=<例名> question=<问> rule_wants=<照规则该退几> exit=<钩子退出码> detections_added=<检出记录多了几行> verdict=<hit|ok|control-ok|control-broken|by-design>
  hit：钩子的结局与规则要的不同（第 4 问漏拦、第 5 问误拦、第 6 问没记检出、第 7 问注册形态让钩子不起作用）。
样本进程照 .claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」起：start_new_session=True，超时对它自己的组 killpg。
"""
import json
import os
import shutil
import signal
import subprocess
import sys

HOOK_RELATIVE = os.path.join(".claude", "hooks", "session-number-stop-check.sh")
TIMEOUT_SECONDS = 180


def run_sample(argv, payload_bytes, environment, cwd):
    """起一条样本进程（自己一个进程组），喂 stdin，交 (退出码, stdout, stderr)。超时对它自己的组先 TERM 再 KILL。"""
    process = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               env=environment, cwd=cwd, start_new_session=True)
    try:
        out, err = process.communicate(payload_bytes, timeout=TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            out, err = process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            out, err = process.communicate()
        return "timeout", out.decode("utf-8", "replace"), err.decode("utf-8", "replace")
    leftover = [entry for entry in os.listdir("/proc") if entry.isdigit() and group_of(entry) == process.pid]
    if leftover:
        os.killpg(process.pid, signal.SIGKILL)
        err += f"\n样本进程组 {process.pid} 留下了 {len(leftover)} 个进程，已停".encode("utf-8")
    return process.returncode, out.decode("utf-8", "replace"), err.decode("utf-8", "replace")


def group_of(pid_text):
    try:
        with open(f"/proc/{pid_text}/stat", encoding="utf-8", errors="replace") as handle:
            return int(handle.read().rsplit(")", 1)[1].split()[2])
    except (OSError, ValueError, IndexError):
        return None


class Probe:
    def __init__(self, tree, work):
        self.tree = os.path.abspath(tree)
        self.work = os.path.abspath(work)
        self.hook = os.path.join(self.tree, HOOK_RELATIVE)
        self.detections = os.path.join(self.work, "detections.jsonl")
        os.makedirs(os.path.join(self.work, "tmp"), exist_ok=True)
        self.lines = []

    def environment(self, **extra):
        environment = {key: value for key, value in os.environ.items()
                       if key not in ("SESSION_NUMBER_STOP_CHECK_BREAK", "SESSION_NUMBER_STOP_CHECK_SELFTEST_MARK")}
        environment.update(SESSION_NUMBER_STOP_CHECK_STATE_DIRECTORY=os.path.join(self.work, "state"),
                           AGENT_HOOK_DETECTIONS=self.detections, TMPDIR=os.path.join(self.work, "tmp"),
                           CLAUDE_PROJECT_DIR=self.tree)
        environment.update(extra)
        return environment

    def detection_count(self):
        try:
            with open(self.detections, encoding="utf-8") as handle:
                return sum(1 for line in handle if line.strip())
        except OSError:
            return 0

    def feed(self, payload, argv=None, environment=None, cwd=None):
        """payload 是 dict（转 JSON）或 bytes（原样）。交 (退出码, stderr, 检出记录多了几行)。"""
        raw = payload if isinstance(payload, bytes) else json.dumps(payload, ensure_ascii=False).encode("utf-8")
        before = self.detection_count()
        code, _, err = run_sample(argv or ["bash", self.hook], raw, environment or self.environment(), cwd or self.tree)
        return code, err, self.detection_count() - before

    def emit(self, name, question, rule_wants, code, added, verdict, note=""):
        line = f"name={name} question={question} rule_wants={rule_wants} exit={code} detections_added={added} verdict={verdict}"
        if note:
            line += f" note={note}"
        self.lines.append(line)
        print(line, flush=True)


def stop_input(session, reply, active=False):
    return {"hook_event_name": "Stop", "session_id": session, "stop_hook_active": active,
            "transcript_path": "", "last_assistant_message": reply}


# 第 4 问：写了 mainagent 加字母、用户照样看不出是谁的回复。规则要拦（退 2）；钩子退 0 记 hit。
# declared=True 的是钩子文件头「管不到的」已点名的写法，照记，verdict 后缀 -declared。
LEAK_REPLIES = (
    ("leak-capital-M", "MainAgent(A) 改了 scripts/x.py，做了一次换上。", False),
    ("leak-space-before-paren", "mainagent (A) 改了 scripts/x.py，做了一次换上。", False),
    ("leak-fullwidth-letter", "mainagent(Ａ) 改了 scripts/x.py，做了一次换上。", False),
    ("leak-fullwidth-parens-and-letter", "mainagent（Ａ）改了 scripts/x.py，做了一次换上。", False),
    ("leak-no-parens", "mainagent A 改了 scripts/x.py，做了一次换上。", False),
    ("leak-bold-letter", "mainagent(**A**) 改了 scripts/x.py，做了一次换上。", False),
    ("leak-markdown-escaped-parens", "mainagent\\(A\\) 改了 scripts/x.py，做了一次换上。", False),
    ("leak-zero-width-space", "main​agent(A) 改了 scripts/x.py，做了一次换上。", False),
    ("leak-chinese-main", "主 agent(A) 改了 scripts/x.py，做了一次换上。", False),
    ("leak-whole-sentence-in-corner-quotes", "「mainagent(A) 改了 scripts/x.py，做了一次换上。」", False),
    ("leak-whole-sentence-in-ascii-quotes", "\"mainagent(A) 改了 scripts/x.py，做了一次换上。\"", False),
    ("leak-whole-sentence-in-backticks", "`mainagent(A) 改了 scripts/x.py，做了一次换上。`", False),
    ("leak-stray-ascii-quote-same-line", "x.py 里那一行写的是 12\" 屏，mainagent(A) 改了它，输出 \"ok\"。", False),
    ("leak-lowercase-letter", "mainagent(a) 改了 scripts/x.py，做了一次换上。", True),
    ("leak-two-letters", "mainagent(AB) 改了 scripts/x.py，做了一次换上。", True),
    ("leak-soft-line-break", "这一处是 mainagent\n(A) 改的：scripts/x.py，做了一次换上。", True),
)

# 第 5 问：合规的回复（没有拿字母指别的会话，是在引用写法或贴命令）。规则不要拦（退 0）；钩子退 2 记 hit。
# 引的规则原文取自 .claude/rules/writing-discipline-by-review.md「怎么说」最后一条（运行时从被判树现读，读不到这一例记 control-broken）。
FALSE_BLOCK_REPLIES = (
    ("false-block-blockquote-rule", None),
    ("false-block-command-single-quotes", "跑了 grep -nF 'mainagent(A)' .claude/agent-common.md，命中 1 行，在「写」一节。"),
    ("false-block-double-backtick-code", "规则里的写法是 ``mainagent(A)``，括号里只放大写字母。"),
    ("false-block-four-backtick-fence", "日志原文：\n````\n```\nmainagent(A) 改了 x\n```\n````\n共 1 段。"),
    ("false-block-indented-code-block", "日志原文：\n\n    mainagent(A) 改了 x\n\n共 1 行。"),
    ("false-block-white-corner-quotes", "规则里的写法是『mainagent(A)』，括号里只放大写字母。"),
    ("false-block-curly-single-quotes", "规则里的写法是‘mainagent(A)’，括号里只放大写字母。"),
    ("false-block-path-outside-code", "那份草稿在 ~/.local/share/singlefs/work/notes/mainagent(A)-message.txt，是发给别的会话的原文。"),
    ("false-block-table-cell-rule-summary", "| 写给谁 | 指别的会话怎么写 |\n|---|---|\n| 会话之间的消息 | mainagent(A) |\n| 给用户的回复 | ListAgents 那一行的编号 |"),
)

CONTROLS = (
    ("control-plain-letter-blocks", "mainagent(A) 改了 scripts/x.py，做了一次换上。", 2),
    ("control-backticked-passes", "规则里的写法是 `mainagent(A)`。", 0),
    ("control-number-passes", "(5b) 改了 scripts/x.py，做了一次换上。", 0),
)


def rule_line(tree):
    path = os.path.join(tree, ".claude", "rules", "writing-discipline-by-review.md")
    try:
        with open(path, encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("- 给用户的回复与汇报里指别的会话"):
                    return line.rstrip("\n")
    except OSError:
        return None
    return None


def question_four_and_five(probe):
    for name, reply, want in CONTROLS:
        code, _, added = probe.feed(stop_input(f"probe-{name}", reply))
        probe.emit(name, "control", want, code, added, "control-ok" if code == want else "control-broken")
    for name, reply, declared in LEAK_REPLIES:
        code, _, added = probe.feed(stop_input(f"probe-{name}", reply))
        verdict = "hit" if code != 2 else "ok"
        probe.emit(name, 4, 2, code, added, verdict + ("-declared" if declared and verdict == "hit" else ""))
    quoted_rule = rule_line(probe.tree)
    for name, reply in FALSE_BLOCK_REPLIES:
        if reply is None:
            if quoted_rule is None:
                probe.emit(name, 5, 0, "-", 0, "control-broken", "被判树里找不到「怎么说」最后一条")
                continue
            reply = "规则原文：\n\n> " + quoted_rule[2:] + "\n\n所以给你的汇报里只写编号。"
        code, _, added = probe.feed(stop_input(f"probe-{name}", reply))
        probe.emit(name, 5, 0, code, added, "hit" if code == 2 else "ok")


def question_four_continuation(probe, hook=None, prefix=""):
    """续跑：同一会话里先被拦一次（指纹记下 mainagent(A)），改净；之后若干条用户决定的干净回合；
    最后一条是别的 Stop 钩子拦回来的续跑（stop_hook_active 为 true），这一回合头一次写 mainagent(<字母>)。
    用户决定的两样放开扫：中间干净回合数 0/1/3，中间有没有一回被拦的是 mainagent(B)；末一条的字母 A/B。
    规则要拦（退 2）。对照：新会话只喂末一条。"""
    final_reply = "已推。(5c) 与 mainagent({letter}) 都改了 y.py，各自换上了。"
    for clean_turns in (0, 1, 3):
        for middle_block in ("none", "B"):
            for letter in ("A", "B"):
                session = f"{prefix}seq-k{clean_turns}-mid{middle_block}-final{letter}"
                steps = [(False, "mainagent(A) 改了 scripts/x.py。"), (True, "(5b) 改了 scripts/x.py。")]
                for turn in range(clean_turns):
                    steps.append((False, f"回合 {turn + 1}：门禁绿了。"))
                if middle_block == "B":
                    steps += [(False, "mainagent(B) 改了 z.py。"), (True, "(5d) 改了 z.py。")]
                steps.append((True, final_reply.format(letter=letter)))
                codes = [probe.feed(stop_input(session, reply, active), argv=["bash", hook or probe.hook])[0] for active, reply in steps]
                code = codes[-1]
                verdict = "control-broken" if codes[0] != 2 else ("hit" if code != 2 else "ok")
                probe.emit(session, "4-continuation", 2, code, 0, verdict,
                           "prefix_exits=" + ",".join(str(value) for value in codes[:-1]))
    for letter in ("A", "B"):
        session = f"{prefix}seq-fresh-final{letter}"
        code, _, added = probe.feed(stop_input(session, final_reply.format(letter=letter), True), argv=["bash", hook or probe.hook])
        probe.emit(session, "4-continuation-control", 2, code, added, "control-ok" if code == 2 else "control-broken")
    # 设计如此的一种（文件头「不死循环」）：被拦之后续跑里同一份命中、换了整句话照样放行
    session = f"{prefix}seq-designed-repeat"
    first = probe.feed(stop_input(session, "mainagent(A) 改了 scripts/x.py。"), argv=["bash", hook or probe.hook])[0]
    code = probe.feed(stop_input(session, "mainagent(A) 另外还改了 z.py，与上一条说的不是同一件事。", True), argv=["bash", hook or probe.hook])[0]
    probe.emit(session, "4-continuation", 2, code, 0, "by-design" if code == 0 else "ok", f"prefix_exits={first}")


def question_six(probe):
    """判不了的路：照文件头要退 0、检出记录多一行。"""
    cases = (
        ("unjudged-empty-stdin", b""),
        ("unjudged-non-utf8", b"\xff\xfe{"),
        ("unjudged-message-not-string", {"hook_event_name": "Stop", "session_id": "u-nonstring", "last_assistant_message": 123}),
        ("unjudged-transcript-is-directory", {"hook_event_name": "Stop", "session_id": "u-dir", "transcript_path": probe.work}),
    )
    for name, payload in cases:
        code, _, added = probe.feed(payload)
        probe.emit(name, 6, 0, code, added, "ok" if code == 0 and added == 1 else "hit")
    transcript = os.path.join(probe.work, "transcript-without-assistant.jsonl")
    with open(transcript, "w", encoding="utf-8") as handle:
        handle.write(json.dumps({"type": "user", "message": {"role": "user", "content": "hi"}}) + "\n")
    code, _, added = probe.feed({"hook_event_name": "Stop", "session_id": "u-noassistant", "transcript_path": transcript})
    probe.emit("unjudged-transcript-without-assistant", 6, 0, code, added, "ok" if code == 0 and added == 1 else "hit")
    # 状态目录建不成（那条路径上是一份普通文件）：store_fingerprint 吞掉 OSError，命中照拦
    blocker = os.path.join(probe.work, "state-blocker")
    with open(blocker, "w", encoding="utf-8") as handle:
        handle.write("x\n")
    code, _, added = probe.feed(stop_input("u-state", "mainagent(A) 改了 x.py。"),
                                environment=probe.environment(SESSION_NUMBER_STOP_CHECK_STATE_DIRECTORY=os.path.join(blocker, "sub")))
    probe.emit("state-directory-unwritable", 6, 2, code, added, "ok" if code == 2 else "hit")
    # preflight 判不了：PATH 上没有 python3（只放 bash 与 dirname）。钩子在进 python 之前退出，Claude Code 照非 0 非 2 放行。
    fake_bin = os.path.join(probe.work, "path-without-python")
    os.makedirs(fake_bin, exist_ok=True)
    for tool in ("bash", "dirname"):
        found = shutil.which(tool)
        if found and not os.path.exists(os.path.join(fake_bin, tool)):
            os.symlink(found, os.path.join(fake_bin, tool))
    code, _, added = probe.feed(stop_input("u-nopython", "mainagent(A) 改了 x.py。"),
                                argv=[os.path.join(fake_bin, "bash"), probe.hook],
                                environment=probe.environment(PATH=fake_bin))
    probe.emit("unjudged-preflight-no-python", 6, 0, code, added,
               "ok" if code == 0 and added == 1 else "hit", "Claude Code 对非 0 非 2 的 Stop 钩子照常放行（推的）")


# 第 7 问钩子一侧：settings.json 里 Stop 那一条写成这些命令形态时，照 Claude Code 起钩子的样子（/bin/sh -c，工作目录是项目根，推的）
# 喂一条写了字母的与一条干净的；effective 指写了字母的退 2、干净的退 0。
COMMAND_FORMS = (
    ("form-registered", 'bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh'),
    ("form-selftest-argument", 'bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh --selftest'),
    ("form-or-true", 'bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh || true'),
    ("form-python3-interpreter", 'python3 "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh'),
    ("form-sh-interpreter", 'sh "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh'),
    ("form-other-file-same-name", 'bash "$CLAUDE_PROJECT_DIR"/.claude/gate.d/fixtures/stub/session-number-stop-check.sh'),
    ("form-cd-then-relative", 'cd "$CLAUDE_PROJECT_DIR" && bash .claude/hooks/session-number-stop-check.sh'),
    ("form-env-bash", 'env bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh'),
    ("form-assignment-prefix", 'LC_ALL=C.UTF-8 bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh'),
    ("form-bash-option", 'bash -e "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh'),
    ("form-direct-exec", '"$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh'),
)


def question_seven_hook_side(probe):
    for name, command in COMMAND_FORMS:
        letter = probe.feed(stop_input(f"{name}-letter", "mainagent(A) 改了 x.py。"), argv=["/bin/sh", "-c", command])
        clean = probe.feed(stop_input(f"{name}-clean", "(5b) 改了 x.py。"), argv=["/bin/sh", "-c", command])
        effective = letter[0] == 2 and clean[0] == 0
        probe.emit(name, "7-hook-side", "2/0", f"{letter[0]}/{clean[0]}", letter[2] + clean[2],
                   "effective" if effective else "not-effective")


LOOP_GUARD_ORIGINAL = 'repeated = hook_input.get("stop_hook_active") is True and transcript.stored_fingerprint(state_file) == fingerprint'
LOOP_GUARD_MUTANT = 'repeated = transcript.stored_fingerprint(state_file) == fingerprint'


CLEAN_PASS_ORIGINAL = """    if not hits or already_shown(courtesy, hook_input, hits):
        return 0, None"""
CLEAN_PASS_FIX = """    if not hits:
        forget_shown(courtesy, hook_input)
        return 0, None
    if already_shown(courtesy, hook_input, hits):
        return 0, None"""
FORGET_DEFINITION = '''def forget_shown(courtesy, hook_input):
    state_directory = os.environ.get(STATE_DIRECTORY_VARIABLE) or os.path.join(os.environ.get("TMPDIR") or "/tmp", "session-number-stop-check")
    state_file = courtesy.SESSION_TRANSCRIPT.shown_state_file(state_directory, hook_input, (("session_id", "no-session"),))
    try:
        os.remove(state_file)
    except OSError:
        pass

def blocked_message(hits):'''


def build_tree(tree, work, name, replacements):
    """在 <运行目录>/<name>/.claude/ 下建一份改过的钩子：hooks/ 放改后的钩子与 lib_work_root.py 的拷贝，
    scripts 与 singlefs-ai-sop 两个目录指回被判树（运行目录在工作根下、不在仓里）。每处替换在被判钩子里必须恰好命中一次。交改后钩子的路径。"""
    root = os.path.join(work, name, ".claude")
    hooks = os.path.join(root, "hooks")
    os.makedirs(hooks, exist_ok=True)
    with open(os.path.join(tree, HOOK_RELATIVE), encoding="utf-8") as handle:
        text = handle.read()
    for old, new in replacements:
        if text.count(old) != 1:
            raise SystemExit(f"{name}：替换点在被判钩子里命中 {text.count(old)} 次，不是 1 次：被判钩子变了，模型要改")
        text = text.replace(old, new)
    hook = os.path.join(hooks, "session-number-stop-check.sh")
    with open(hook, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.chmod(hook, 0o755)
    shutil.copy(os.path.join(tree, ".claude", "hooks", "lib_work_root.py"), os.path.join(hooks, "lib_work_root.py"))
    for linked in ("scripts", "singlefs-ai-sop"):
        link = os.path.join(root, linked)
        if not os.path.lexists(link):
            os.symlink(os.path.join(tree, ".claude", linked), link)
    return hook


def build_mutant(tree, work):
    """变异：去掉「只在续跑里」这个条件（任何一次收工，命中与上一次记下的指纹相同就放行）。
    改法（攻方腿自提，只在本模型上量）：回复里没有命中时删掉这一会话的指纹，「上一次拦下的」只活到下一次干净的收工。"""
    mutant = build_tree(tree, work, "mutant-loop-guard", ((LOOP_GUARD_ORIGINAL, LOOP_GUARD_MUTANT),))
    fix = build_tree(tree, work, "fix-forget-on-clean", ((CLEAN_PASS_ORIGINAL, CLEAN_PASS_FIX), ("def blocked_message(hits):", FORGET_DEFINITION)))
    return mutant + "\n" + fix


def mutant_discriminator(probe, mutant):
    """自证缺的那一格：同一会话两次平常收工（都不是续跑）写同一个字母，第二次照规则要拦。原钩子与变异各喂一遍。"""
    for label, hook in (("original", probe.hook), ("mutant", mutant)):
        session = f"cross-turn-{label}"
        first = probe.feed(stop_input(session, "mainagent(A) 改了 x.py。"), argv=["bash", hook])[0]
        second = probe.feed(stop_input(session, "mainagent(A) 又改了 y.py。"), argv=["bash", hook])[0]
        verdict = "control-broken" if first != 2 else ("ok" if second == 2 else "hit")
        probe.emit(f"loop-guard-cross-turn-{label}", "6-mutation", 2, second, 0, verdict, f"prefix_exits={first}")


def main(arguments):
    if len(arguments) == 3 and arguments[0] == "--build-mutant":
        print(build_mutant(arguments[1], arguments[2]))
        return 0
    if len(arguments) != 2:
        print("用法：hook_probes.py <被判树> <运行目录> | hook_probes.py --build-mutant <被判树> <运行目录>", file=sys.stderr)
        return 2
    probe = Probe(arguments[0], arguments[1])
    question_four_and_five(probe)
    question_four_continuation(probe)
    question_six(probe)
    question_seven_hook_side(probe)
    fix = os.path.join(probe.work, "fix-forget-on-clean", ".claude", "hooks", "session-number-stop-check.sh")
    if os.path.isfile(fix):
        question_four_continuation(probe, fix, "fix-")
    else:
        probe.emit("fix-seq", "4-continuation", 2, "-", 0, "control-broken", "改法钩子没建")
    mutant = os.path.join(probe.work, "mutant-loop-guard", ".claude", "hooks", "session-number-stop-check.sh")
    if os.path.isfile(mutant):
        mutant_discriminator(probe, mutant)
    else:
        probe.emit("loop-guard-cross-turn", "6-mutation", 2, "-", 0, "control-broken", "变异钩子没建")
    broken = [line for line in probe.lines if "verdict=control-broken" in line]
    hits = [line for line in probe.lines if "verdict=hit" in line]
    print(f"name=summary probes={len(probe.lines)} hits={len(hits)} control_broken={len(broken)}")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
