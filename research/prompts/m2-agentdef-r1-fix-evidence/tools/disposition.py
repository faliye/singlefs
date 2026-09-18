#!/usr/bin/env python3
"""m2-agentdef-r1 改法（规格第 1–4 条）的逐处处置：数据、处置表与核对。

    python3 disposition.py --table                 # 打印处置表的表格行（写进 m2-agentdef-r1-fix-disposition.md 的就是这份输出）
    python3 disposition.py --check <仓根>          # 核对，见下

核对三件事，任一件不成立退出码 1：
  一、把 definition-edits.json 里实际做过的 27 处替换倒着施加到仓里现在的 18 份定义上（新串换回旧串，各恰好命中一次），
      得到的内容的 sha256 与这一轮开工快照 research/prompts/m2-agentdef-r1-start-snapshot.sha256 逐份相同——
      说明 27 处替换就是这一次改动的全部，没有表外改动，也没有抄错的旧串；
  二、处置表每一行的「原句那一截」在倒推出的改前那一行里逐字命中，在现在那一行里不再出现（「加指令」只在原句后面加的那一行除外）；
  三、处置表每一行的「改成什么」在现在那一行里逐字命中。
"""
import hashlib, json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))

# (标签, 文件, 行号, 原句那一截, 实际做的那处替换, 类别, 改完那一处现在的样子, 原句是否留着)
ROWS = [
    ("规格 1", "implementation-writer.md", 27, "（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录），先跑一份不改动的副本", "iw1", "加指令（H1）",
     "（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本", False),
    ("规格 2", "three-way-verifier.md", 21, "没给快照的代码轮，停下要，不对主树核。", "tv1", "加指令（H2）",
     "没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时记「分不清：文件可能在腿交回之后被改过」（与第 6 步那一类一样单列），不记 ✗。", True),
    ("规格 3", "main-agent.md", 24, "盯住，不强制结束：", "ma3", "补回（A3）", "盯住，不能一直干等，也不强制结束：", False),
    ("M01", "crash-verifier.md", 24, "（54 号在本机负载下实测约 47 分钟，55、57、59 号各在一分钟内）", "cv3", "删说明", "结束后读输出。别的会话", False),
    ("M01 同一行", "crash-verifier.md", 24, "，后台起它的那条命令自己的 `$?` 只说明起没起来", "cv3", "前提改写成指令",
     "；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`）", False),
    ("M02", "kb-scribe.md", 26, "（agent-defs-r2 攻方模型：1/3 对 0/3）", "ks2", "删说明", "用它，不用逐条 Edit。", False),
    ("M03", "mutation-triage.md", 25, "（今天 4 张）", "mt1", "删说明", "表头写着「被测装置是 shell 探针」的，照表头", False),
    ("M04", "mutation-triage.md", 25, "：共用的会让开头几行链接上一轮最后一条变异，计划第十八节", "mt2", "删说明", "不用它默认那个跨轮共用的）。", False),
    ("M05", "experiment-runner.md", 26, "，免得读数被抢", "er1", "删说明", "`gate.sh` 也要等。", False),
    ("M06", "experiment-designer.md", 28, "，免得命中行把结论带进来", "ed1", "删说明", "--exclude-dir=prompts`。", False),
    ("M07", "experiment-designer.md", 29, "，防等价变异", "ed2", "删说明", "写明它在哪个取样点上改变输出。计时类", False),
    ("M08", "crash-verifier.md", 13, "，派你是为了在提交前的整轮门禁之前先拿到它们的读数", "cv1", "删说明", "里最重的那几道。", False),
    ("M09", "crash-verifier.md", 23, "（两份层 0 全量同时跑各要一个多钟头）", "cv2", "删说明", "不起 54 号，等它结束。", False),
    ("M10", "gate-triage.md", 24, "（两道门禁同时跑，重阶段互相拖、工作区指纹互相干扰）", "gt1", "删说明", "在跑就等它结束。", False),
    ("M11", "experiment-runner.md", 27, "（后面的门禁阶段都不查命名）", "er2", "前提改写成指令",
     "新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。", False),
    ("M12", "experiment-runner.md", 30, "（`replay.sh` 按这个相对路径找）", "er3", "前提改写成指令",
     "生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；", False),
    ("M13", "kb-scribe.md", 28, "（变异表锚点里的分项标签不跟着改，会腐化）", "ks4", "删说明", "加跑 33 号，并把", False),
    ("M14", "kb-scribe.md", 28, "（入库产物里印着旧标签的，复跑会对不上）", "ks4", "删说明", "逐个列给主 agent；预演", False),
    ("M15", "kb-scribe.md", 27, "（`--write` 会先往那些条目里写「（待补）」）", "ks3", "前提改写成指令",
     "就停在 `--write` 之前交回，不让 `--write` 往那些条目里写「（待补）」；", False),
    ("M16", "implementation-writer.md", 46, "（56 号门禁要的判决文件由主 agent 的那一轮产出）", "iw2", "删说明", "- 没走三方对抗；层 0", False),
    ("M17", "mutation-triage.md", 25, "；在仓根下跑会被报成「基线就是红的」", "mt1", "前提改写成指令",
     "（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）", False),
    ("M18", "kb-scribe.md", 26, "用它不用逐条 Edit：两阶段写在有一处不中时一个文件都不写，逐条 Edit 中途被打断或锚点被别人改掉会留下半套", "ks2", "删说明",
     "用它，不用逐条 Edit。", False),
    ("M19", "agent-common.md", 64, "：`/tmp` 下的草稿目录会话一重启就没了，那一跑等于白干", "ac3", "删说明", "再往下做。停机条款", False),
    ("M20", "agent-common.md", 43, "，那类读数被抢了 CPU 就不作数", "ac1", "删说明", "停下；只有别的", False),
    ("M21", "main-agent.md", 26, "临时派 general-purpose 干带长等待的活时，派发提示里写上这一条：它不读共用约束。", "ma4", "前提改写成指令",
     "临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上这一条。", False),
    ("M22", "kb-scribe.md", 19, "标题决定条目挂在哪条决策下、条目里裸写的分项归谁，所以也由主 agent 给；", "ks1", "删说明", "不由你概括；标题里「（其N）」那一段", False),
    ("M23", "experiment-designer.md", 34, "`experiment-runner` 照这些节名找东西，以后门禁也按它们查。", "ed3", "删说明", "写一句为什么不适用，不删节。", False),
    ("M24", "main-agent.md", 20, "发散是探索该有的样子，开口子不是问题，开出来没人收才是。", "ma2", "删说明", "一条都不许无声消失。", False),
    ("M25", "agent-common.md", 43, "这个仓里别的会话几乎一直在跑 cargo，见到就停等于开不了工。", "ac2", "删说明", "等锁等了多久。定义另有更严要求的照定义。", False),
    ("M26", "main-agent.md", 19, "延迟不是丢掉，", "ma1", "删说明", "逐条写去向。不做也要写明依据。", False),
    ("M27", "experiment-designer.md", 39, "（它只会整份写出口文件，出口写成登记本身会盖掉占号时写的文件头，而且退出码照样是 0）", "ed4", "前提改写成指令",
     "；出口不指向登记本身，追加完回读登记，占号时写的文件头在不在看文件，不看退出码 |", False),
]


def cell(text):
    return text.replace("|", "\\|")


def table():
    print("| 标签 | 位置 | 原句那一截（整抄） | 改成什么 | 类别 |")
    print("|---|---|---|---|---|")
    for tag, name, line, fragment, _edit, category, result, kept in ROWS:
        if category == "删说明":
            change = f"删掉；那一处现在是「{result}」"
        elif kept:
            change = f"原句留着，后面加「{result}」"
        else:
            change = f"改成「{result}」"
        print(f"| {tag} | `.claude/agents/{name}:{line}` | 「{cell(fragment)}」 | {cell(change)} | {category} |")


def check(root):
    edits = json.load(open(os.path.join(HERE, "definition-edits.json"), encoding="utf-8"))
    snapshot = {}
    for entry in open(os.path.join(root, "research/prompts/m2-agentdef-r1-start-snapshot.sha256"), encoding="utf-8"):
        digest, path = entry.split(None, 1)
        snapshot[path.strip()] = digest
    problems = []
    current, before = {}, {}
    agents = sorted(p for p in snapshot if p.startswith(".claude/agents/"))
    for path in agents:
        current[path] = open(os.path.join(root, path), encoding="utf-8").read()
        text = current[path]
        for edit in reversed([e for e in edits if e["file"] == path]):
            hits = text.count(edit["new"])
            if hits != 1:
                problems.append(f"{path}：替换 {edit['id']} 的新串命中 {hits} 次，倒推不了")
                continue
            text = text.replace(edit["new"], edit["old"])
        before[path] = text
        digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
        verdict = "同" if digest == snapshot[path] else "不同"
        touched = sum(1 for e in edits if e["file"] == path)
        print(f"一、{path}：倒推 {touched} 处替换之后 sha256 与开工快照{verdict}")
        if digest != snapshot[path]:
            problems.append(f"{path}：倒推后的 sha256 与开工快照不同")
    for tag, name, line, fragment, edit, category, result, kept in ROWS:
        path = f".claude/agents/{name}"
        before_line = before[path].split("\n")[line - 1]
        current_line = current[path].split("\n")[line - 1]
        in_before = fragment in before_line
        gone = kept or fragment not in current_line
        result_in_current = result in current_line
        edit_ids = [e["id"] for e in edits]
        ok = in_before and gone and result_in_current and edit in edit_ids
        print(f"二三、{tag}（{path}:{line}，替换 {edit}）：原句在改前那一行{'命中' if in_before else '没命中'}；"
              f"{'原句留着' if kept else ('现在那一行里不再有原句' if gone else '现在那一行里还有原句')}；"
              f"改成的串在现在那一行{'命中' if result_in_current else '没命中'}")
        if not ok:
            problems.append(f"{tag}：核对不过")
    used = {row[4] for row in ROWS}
    unused = [e["id"] for e in edits if e["id"] not in used]
    print(f"处置表 {len(ROWS)} 行，覆盖实际替换 {len(used)} / {len(edits)} 处；没被任何一行引到的替换：{unused or '无'}")
    if unused:
        problems.append(f"有替换没进处置表：{unused}")
    if problems:
        for problem in problems:
            print(f"  ✗ {problem}")
        return 1
    print("  ✓ 全部核对通过")
    return 0


if __name__ == "__main__":
    if sys.argv[1:2] == ["--table"]:
        table()
    elif sys.argv[1:2] == ["--check"] and len(sys.argv) == 3:
        sys.exit(check(sys.argv[2]))
    else:
        print(__doc__)
        sys.exit(2)
