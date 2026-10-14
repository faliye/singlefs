# R7：叫停轮。第一轮：正推 D6-1（r-real / r-self / r-unrel / r-nobracket / r-short / r-open）与攻方 F1–F3 的 halted 文件写法，
# 改前树照原样写 halted 文件喂；改后树把同一段历史换成 abandoned-rounds.tsv 一行（出处照原来指的那份文件）再喂，另喂一遍原样的 halted 文件（看「已撤」）。
# 度量：派 kb-scribe 点名 .claude/kb/<轮>.md，输出里「开工快照」拒绝的有无（rc=2 且带 6 标签 = 被快照拦；rc=0 = 放行）。
# 用法：python3 -I r7_abandoned.py <改前树> <改后树> <工作目录>
import hashlib, os, shutil, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as c
trees, W = c.parse_trees(sys.argv)
REAL_SRC = "records/2026-10-01-删除与并发事务.md"
REAL_QUOTE = "「增加流程限制  测试代码不需要打， 改了直接测。」"
rule_line = 0
for label, tree in trees:
    sandbox = f"{W}/r7-{label}"
    shutil.rmtree(sandbox, ignore_errors=True)
    os.makedirs(sandbox)
    for item in (".claude", "scripts", "records", "research"):
        if os.path.exists(f"{tree}/{item}"):
            subprocess.run(["rsync", "-a", "--exclude", "__pycache__", f"{tree}/{item}", sandbox + "/"], check=True)
    os.makedirs(f"{sandbox}/research/prompts", exist_ok=True)
    # 自己的样本 records 与「仓外」文件
    outside = f"{W}/r7-outside-{label}"; os.makedirs(outside, exist_ok=True)
    note_text = "r-outside-r1 随便一句十个字以上的话「随便一句十个字以上的话」\n"
    open(f"{outside}/outside-note.md", "w").write(note_text)
    open(f"{W}/outside-note.md", "w").write(note_text)   # research/../../outside-note.md 从样本根的 research/ 往上两层落在这里（样本根的上一层）
    open(f"{sandbox}/records/r2s-sample.md", "w").write(
        "用户 2026-10-11 说：「别再往下开这一轮了」，轮 r-good-r1 到此为止\n"                 # 1  合格：名字、原话
        "与任何一轮无关的用户话「这是一句和任何一轮都无关的话」\n"                                # 2  没有轮名
        "用户让 r-other-r1 停：「别的轮的叫停原话也有六个字」\n"                                  # 3  别的轮的原话
        "r-short-r1 的叫停：「增加流程」\n"                                                      # 4  不到 6 个字
        "r-nobracket-r1 的叫停，没有引原话\n"                                                    # 5  没有「」
        "用户同时让 r-two-a-r1 与 r-two-b-r1 停：「两轮一起先停下来吧」\n"                          # 6  同一行两个轮名
        "r-unrel-quote-r1 在这一行只被提到，另一句话「这句话是用户说的别的事情」\n"                     # 7  名字在、原话是别的事
        "r-baddate-r1 的叫停：「日期写坏的登记行」\n"                                            # 8
        "r-dup-r1 的叫停：「同一轮登记两行的样本」\n"                                            # 9
    )
    os.symlink(f"{outside}/outside-note.md", f"{sandbox}/records/r2s-link.md")
    main_line = [i for i, l in enumerate(open(f"{sandbox}/.claude/main-agent.md", encoding="utf-8"), 1) if "本机常有别的会话在同一个仓里干活" in l][0]
    unrel_quote = "本机常有别的会话在同一个仓里干活"
    # (轮名, 出处文本（tsv/halted 里「出处」那一段）, 说明)
    cases = [
        ("m3-batch63-code-r1", f"{REAL_SRC} 第 1595 行{REAL_QUOTE}", "D6-1-real-user-quote"),
        ("r-self-r1", "research/prompts/r-self-r1-halted.md 第 1 行「这是一句自己写的六个字以上的话」", "F1-self-cite"),
        ("r-unrel-r1", f".claude/main-agent.md 第 {main_line} 行「{unrel_quote}」", "D6-1-unrelated-rule-line"),
        ("r-ownbody-r1", "research/prompts/_r-ownbody-r1-body.md 第 1 行「第一轮腿还在跑」", "F1-cite-own-body"),
        ("r-rule-r1", ".claude/rules/three-way-inference.md 第 121 行「用户叫停、不写判决的轮」", "F1-cite-the-rule"),
        ("r-outside-r1", "research/../../outside-note.md 第 1 行「随便一句十个字以上的话」", "F2-outside-repo-via-research"),
        ("r-dotdot-r1", "records/../research/prompts/_r-dotdot-r1-body.md 第 1 行「路径里带点点的六个字以上」", "X-records-dotdot-inside-repo"),
        ("r-symlink-r1", "records/r2s-link.md 第 1 行「随便一句十个字以上的话」", "X-records-symlink-to-outside"),
        ("r-good-r1", "records/r2s-sample.md 第 1 行「别再往下开这一轮了」", "control-good-source"),
        ("r-nobracket-r1", "records/r2s-sample.md 第 5 行", "D6-2-no-bracket-quote"),
        ("r-short-r1", "records/r2s-sample.md 第 4 行「增加流程」", "D6-2-short-quote"),
        ("r-borrow-r1", "records/r2s-sample.md 第 3 行「别的轮的叫停原话也有六个字」", "X-other-rounds-quote"),
        ("r-noname-r1", "records/r2s-sample.md 第 2 行「这是一句和任何一轮都无关的话」", "X-line-without-round-name"),
        ("r-two-a-r1", "records/r2s-sample.md 第 6 行「两轮一起先停下来吧」", "X-two-round-names-one-line-a"),
        ("r-two-b-r1", "records/r2s-sample.md 第 6 行「两轮一起先停下来吧」", "X-two-round-names-one-line-b"),
        ("r-unrel-quote-r1", "records/r2s-sample.md 第 7 行「这句话是用户说的别的事情」", "X-name-present-quote-about-something-else"),
        ("r-baddate-r1", "records/r2s-sample.md 第 8 行「日期写坏的登记行」", "X-gate-red-row-bad-date"),
        ("r-dup-r1", "records/r2s-sample.md 第 9 行「同一轮登记两行的样本」", "X-gate-red-row-duplicated"),
        ("r-open-r1", None, "D6-1-open-no-registration"),
    ]
    for name, source, cid in cases:
        pdir = f"{sandbox}/research/prompts"
        open(f"{pdir}/_{name}-body.md", "w").write(f"# {name} 正文\n" if name != "r-ownbody-r1" else "第一轮腿还在跑\n")
        digest = hashlib.sha256(name.encode()).hexdigest()
        os.makedirs(f"{pdir}/{name}-snapshot", exist_ok=True)
        open(f"{pdir}/{name}-snapshot/kb-sha256.txt", "w").write(f"{digest}  .claude/kb/{name}.md\n")
    open(f"{sandbox}/research/prompts/_r-dotdot-r1-body.md", "w").write("r-dotdot-r1 正文「路径里带点点的六个字以上」\n")
    # 登记 / halted 文件按树的形态写
    rows = []
    for name, source, cid in cases:
        if source is None: continue
        if label == "before":
            open(f"{sandbox}/research/prompts/{name}-halted.md", "w").write(f"出处：{source}\n" if name != "r-self-r1" else f"出处：{source}\n")
        else:
            date = "昨天" if name == "r-baddate-r1" else "2026-10-12"
            rows.append(f"{name}\t{date}\t用户叫停\t不写判决；出处 {source}")
            if name == "r-dup-r1": rows.append(f"{name}\t2026-10-12\t用户叫停（第二行）\t不写判决；出处 records/r2s-sample.md 第 2 行「这是一句和任何一轮都无关的话」")
    if label == "after":
        tsv = f"{sandbox}/research/prompts/abandoned-rounds.tsv"
        base = [l for l in open(tsv, encoding="utf-8").read().splitlines() if not l.startswith("m3-batch63-code-r1\t")]
        real = [l for l in open(f"{tree}/research/prompts/abandoned-rounds.tsv", encoding="utf-8").read().splitlines() if l.startswith("m3-batch63-code-r1\t")]
        open(tsv, "w").write("\n".join(base + real + [r for r in rows if not r.startswith("m3-batch63-code-r1\t")]) + "\n")
    # 喂：派 kb-scribe 点名 .claude/kb/<轮>.md
    for name, source, cid in cases:
        text = (f"# 派发：kb-scribe\n\n重型测试：不跑\n开工先读 `.claude/agent-common.md`\n规格检查已判：自检样本不带规格\n"
                f"规格：改 `.claude/kb/{name}.md` 里一行。\n判决：x\n草稿目录：~/.local/share/singlefs/work/r2s-x\n报告：~/.local/share/singlefs/work/r2s-x/report.md\n")
        rc, tg, out = c.check_prompt(sandbox, "kb-scribe", text, workdir=W)
        blocked = "6" in tg
        notes = [l for l in out.splitlines() if l.startswith("! research/prompts") and name in l]
        reason = (notes[0].split("：", 1)[-1][:90].replace(" ", "_") if notes else "-")
        print(f"name=R7 tree={label} case={cid} round={name} blocked_by_snapshot={blocked} rc={rc} note={reason}")
    # 第一轮另立的一笔账：「快照冲突已判：<轮名> <理由>」理由非空就放行（改前改后都在）
    name = "r-open-r1"
    for cid, reason in (("X-valve-snapshot-conflict-any-reason", "随便"), ("X-valve-snapshot-conflict-placeholder", "<理由>")):
        text = (f"# 派发：kb-scribe\n\n重型测试：不跑\n开工先读 `.claude/agent-common.md`\n规格检查已判：自检样本不带规格\n"
                f"规格：改 `.claude/kb/{name}.md` 里一行。\n判决：x\n快照冲突已判：{name} {reason}\n草稿目录：~/.local/share/singlefs/work/r2s-x\n报告：~/.local/share/singlefs/work/r2s-x/report.md\n")
        rc, tg, out = c.check_prompt(sandbox, "kb-scribe", text, workdir=W)
        print(f"name=R7 tree={label} case={cid} round={name} blocked_by_snapshot={'6' in tg} rc={rc}")
    # 「已撤」：改后树上喂原样的 halted 文件（合格的那一份，对应 D6-1 的 r-real）
    if label == "after":
        name = "r-halted-only-r1"
        pdir = f"{sandbox}/research/prompts"
        open(f"{pdir}/_{name}-body.md", "w").write("x\n"); os.makedirs(f"{pdir}/{name}-snapshot", exist_ok=True)
        open(f"{pdir}/{name}-snapshot/kb-sha256.txt", "w").write(f"{hashlib.sha256(name.encode()).hexdigest()}  .claude/kb/{name}.md\n")
        open(f"{pdir}/{name}-halted.md", "w").write(f"出处：{REAL_SRC} 第 1595 行{REAL_QUOTE}\n")
        text = (f"# 派发：kb-scribe\n\n重型测试：不跑\n开工先读 `.claude/agent-common.md`\n规格检查已判：自检样本不带规格\n规格：改 `.claude/kb/{name}.md` 里一行。\n判决：x\n草稿目录：~/.local/share/singlefs/work/r2s-x\n报告：~/.local/share/singlefs/work/r2s-x/report.md\n")
        rc, tg, out = c.check_prompt(sandbox, "kb-scribe", text, workdir=W)
        notes = [l for l in out.splitlines() if "不再认这一支" in l]
        print(f"name=R7 tree=after case=F3-halted-file-only-after-withdrawal round={name} blocked_by_snapshot={'6' in tg} rc={rc} halted_note_present={bool(notes)}")
