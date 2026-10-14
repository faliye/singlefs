# R6：写者锁。第一轮正推腿的 wlcases（开工先认锁 1a–5b）、wlsub（嵌套目录）、dispatch16（派发闸 ⑯ a–f）在改前、改后两棵树上重喂；
# 再加：回执对不上时拒绝说明点没点名持锁者、有 Write/Edit 的定义个数（静态）。
# 用法：python3 -I r6_writer_lock.py <改前树> <改后树> <工作目录（要在 ~/.local/share/singlefs/work/ 底下，写闸才认草稿根）>
import importlib.util, json, os, shutil, subprocess, sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as c
trees, W = c.parse_trees(sys.argv)

def load_lib(tree, tag):
    spec = importlib.util.spec_from_file_location("lib_writer_lock_" + tag, f"{tree}/.claude/hooks/lib_writer_lock.py")
    sys.path.insert(0, f"{tree}/.claude/hooks")
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    sys.path.pop(0)
    return m

def receipt(agent, prompt, ts):
    return json.dumps({"timestamp": time.strftime("%Y-%m-%dT%H:%M:%S.000Z", time.gmtime(ts)),
                       "toolUseResult": {"status": "async_launched", "agentId": agent, "prompt": prompt}})

def write(tree, tr, agent, target):
    inp = {"tool_name": "Write", "tool_input": {"file_path": target, "content": "x\n"}, "agent_id": agent, "agent_type": "clerk",
           "session_id": "sess", "transcript_path": tr, "cwd": W}
    r = subprocess.run(["bash", f"{tree}/.claude/hooks/write-guard.sh"], input=json.dumps(inp), capture_output=True, text=True,
                       env=dict(os.environ, CLAUDE_PROJECT_DIR=tree))
    refused = "写者锁" in r.stderr and "不是" in r.stderr and "持有" in r.stderr
    return r.returncode, refused, r.stderr

PA, PB = "提示甲：写 D", "提示乙：别的活"
for label, tree in trees:
    L = load_lib(tree, label)
    D = f"{W}/wl-{label}"; tr = f"{W}/wl-{label}.jsonl"
    shutil.rmtree(D, ignore_errors=True); os.makedirs(D)
    now = time.time()
    def lock_with(prompt, at):
        for n in os.listdir(D):
            p = f"{D}/{n}"
            os.unlink(p) if os.path.isfile(p) else shutil.rmtree(p)
        lk = L.new_lock("sess", prompt, "clerk", tr, now=at); L.create_lock(D, lk); return lk
    def case(cid, agent, target=None, expect_holder=None):
        rc, refused, err = write(tree, tr, agent, target or f"{D}/report.md")
        claim = any(n.startswith(".writer-claim") for n in os.listdir(D))
        named = (expect_holder in err) if expect_holder else "-"
        print(f"name=R6 tree={label} case={cid} rc={rc} refused_by_lock={refused} claim_file={claim} names_holder={named}")
    # 1：正常，B 的回执对不上（别的提示）先写、A 后写
    lock_with(PA, now); open(tr, "w").write(receipt("agentA", PA, now+1) + "\n" + receipt("agentB", PB, now+2) + "\n")
    case("1a-B-wrong-prompt-first", "agentB", expect_holder="agentA"); case("1b-A-after", "agentA")
    # 2：重启后旧进程：回执在记录里、早于锁
    lock_with(PA, now+600); open(tr, "w").write(receipt("oldA", PA, now+1) + "\n" + receipt("newA", PA, now+601) + "\n")
    case("2a-old-process-receipt-before-lock", "oldA", expect_holder="newA"); case("2b-new-dispatch", "newA")
    # 3：旧进程回执不在记录里
    lock_with(PA, now+600); open(tr, "w").write(receipt("newA", PA, now+601) + "\n")
    case("3a-old-process-no-receipt-first", "oldA"); case("3b-new-dispatch-after", "newA", expect_holder="oldA")
    # 4：记录读不了
    os.unlink(tr); lock_with(PA, now+600)
    case("4a-transcript-unreadable-old-first", "oldA"); case("4b-transcript-unreadable-new-after", "newA", expect_holder="oldA")
    # 5：无回执的 B 写子目录
    lock_with(PA, now+600); open(tr, "w").write(receipt("agentA", PA, now+601) + "\n"); os.makedirs(f"{D}/sub", exist_ok=True)
    case("5a-B-no-receipt-sub", "agentB", f"{D}/sub/x.md")
    # 嵌套之前：只有父目录有锁（A 已认领），子目录还没另派给谁：A 写子目录放行（「父目录持锁者不写子目录」只有文字，钩子在子目录有自己的锁之后才拦）
    shutil.rmtree(D, ignore_errors=True); os.makedirs(f"{D}/sub"); open(tr, "w").write(receipt("agentA", "甲", now+1) + "\n")
    L.create_lock(D, L.new_lock("s", "甲", "clerk", tr, now=now))
    case("sub-before-E-lock-A-writes-parent", "agentA", f"{D}/a.md"); case("sub-before-E-lock-A-writes-sub", "agentA", f"{D}/sub/a.md")
    # 9：记录里没有任何回执对得上这把锁（只有别的提示的回执）：拒绝说明写不写锁登记的提示 sha256 前几位
    shutil.rmtree(D, ignore_errors=True); os.makedirs(D); lock_with(PA, now+600); open(tr, "w").write(receipt("agentB", PB, now+601) + "\n")
    rc9, refused9, err9 = write(tree, tr, "agentB", f"{D}/report.md")
    print(f"name=R6 tree={label} case=9-no-receipt-matches-lock rc={rc9} refused_by_lock={refused9} message_has_prompt_sha_prefix={'sha256 前 12 位' in err9} message_names_holder_agent={'子 agent agent' in err9}")
    # 嵌套（wlsub）：A 持父、E 持子
    shutil.rmtree(D, ignore_errors=True); os.makedirs(f"{D}/sub")
    open(tr, "w").write(receipt("agentA", "甲", now+1) + "\n" + receipt("agentE", "乙", now+2) + "\n")
    L.create_lock(D, L.new_lock("s", "甲", "clerk", tr, now=now)); L.create_lock(f"{D}/sub", L.new_lock("s", "乙", "clerk", tr, now=now))
    case("sub-A-writes-parent", "agentA", f"{D}/a.md"); case("sub-E-writes-own-sub", "agentE", f"{D}/sub/e.md")
    case("sub-A-writes-E-sub", "agentA", f"{D}/sub/a-in-sub.md"); case("sub-E-writes-parent", "agentE", f"{D}/e-in-parent.md")
    shutil.rmtree(D, ignore_errors=True)
    # 文件头编号：lib_writer_lock.py 第 4 行附近写 ⑮ 还是 ⑯
    head = open(f"{tree}/.claude/hooks/lib_writer_lock.py", encoding="utf-8").read().splitlines()[3]
    print(f"name=R6 tree={label} case=stale-number-in-lib-header line4_has_15={'⑮' in head} line4_has_16={'⑯' in head}")

# 派发闸 ⑯（dispatch16 的 a–f）
for label, tree in trees:
    D = f"{W}/d16-{label}"; shutil.rmtree(D, ignore_errors=True)
    tr = f"{W}/d16-{label}.jsonl"; open(tr, "w").close()
    def p(line, extra=""):
        return f"# 派发：clerk\n\n重型测试：不跑\n开工先读 `.claude/agent-common.md`\n{line}\n{extra}报告：{W}/r.md\n"
    def dispatch(cid, text):
        inp = {"tool_name": "Agent", "tool_input": {"subagent_type": "clerk", "prompt": text}, "session_id": "s16", "transcript_path": tr}
        r = subprocess.run(["bash", f"{tree}/.claude/hooks/runner-dispatch-guard.sh"], input=json.dumps(inp), capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=tree))
        refused = "写者锁" in r.stderr and "✗" in r.stderr
        lock = os.path.isfile(f"{D}/.writer-lock")
        print(f"name=R6 tree={label} case=d16-{cid} rc={r.returncode} lock_refusal={refused} lock_present={lock}")
    dispatch("a-first", p(f"草稿目录：{D}")); dispatch("b-same-dir-again", p(f"草稿目录：{D}", "另一件活\n"))
    dispatch("c-with-valve", p(f"草稿目录：{D}", "写者锁已判：旧写者是重启留下的\n"))
    dispatch("d-inline-not-line-start", f"# 派发：clerk\n\n重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿放在 草稿目录：{D}/x 里\n报告：{W}/r.md\n")
    dispatch("e-sub-dir", p(f"草稿目录：{D}/sub")); dispatch("f-backtick", p(f"草稿目录：`{D}`"))
    shutil.rmtree(D, ignore_errors=True)

# 静态：tools 里有 Write / Edit 的定义个数；共用约束「开工先认锁」那一条是否写了「tools 里有 Write / Edit 的」
for label, tree in trees:
    agents = sorted(os.listdir(f"{tree}/.claude/agents"))
    with_edit = bad = 0
    for a in agents:
        t = [l for l in open(f"{tree}/.claude/agents/{a}", encoding="utf-8") if l.startswith("tools:")]
        if t and ("Write" in t[0] or "Edit" in t[0]): with_edit += 1
        else: bad += 1
    text = open(f"{tree}/.claude/agent-common.md", encoding="utf-8").read()
    print(f"name=R6 tree={label} case=static-agents total={len(agents)} with_write_or_edit={with_edit} bash_only={bad} "
          f"common_scopes_lock_to_write_edit={'tools 里有 Write / Edit 的，开工先认锁' in text} common_says_child_dir_rule={'子目录另派给别的子 agent 之后，父目录的持锁者不写那个子目录' in text}")
