#!/usr/bin/env python3
"""派发闸 --check-prompt 多副本矩阵：同一份提示喂给 after（第二轮冻结副本）、before（第一轮冻结副本）、pre0（零轮之前）三份，
每行打 label、各副本退出码、after 的首条拒绝原因。用例 = 第一轮正推腿的 feed.py 全部用例（原样搬来）+ 第二轮 X1/X6 新增用例。
用法：feed_matrix.py --root after=<路径> --root before=<路径> [--root pre0=<路径>] --work <草稿目录> [--only 子串] [--raw 文件]"""
import argparse, os, subprocess, sys

ap = argparse.ArgumentParser()
ap.add_argument("--root", action="append", required=True)
ap.add_argument("--work", required=True)
ap.add_argument("--only", default="")
ap.add_argument("--raw", default="")
args = ap.parse_args()
os.makedirs(args.work, exist_ok=True)
ROOTS = [tuple(item.split("=", 1)) for item in args.root]
WORK = args.work


def base(kind, thread="线程上限：31", wall="预计挂钟：2 小时内", extra=""):
    inputs = {
        "clerk": f"草稿目录：{WORK}/\n报告：{WORK}/r.md\n",
        "three-way-forward": f"禁读清单：无\n草稿目录：{WORK}/\n背景材料：x\n报告：{WORK}/r.md\n",
        "three-way-defense": f"禁读清单：无\n草稿目录：{WORK}/\n背景材料：x\n判决：x\n",
        "tooling-writer": f"草稿目录：{WORK}/\n报告：{WORK}/r.md\n出口：x\n要改的文件：x\n",
        "investigator": f"草稿目录：{WORK}/\n报告：{WORK}/r.md\n现象：x\n",
    }[kind]
    return "重型测试：不跑\n开工先读 `.claude/agent-common.md`\n" + inputs + (wall + "\n" if wall else "") + (thread + "\n" if thread else "") + extra

def run(root, kind, text, tag, env=None):
    path = os.path.join(args.work, f"prompt-{tag}.md")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", kind, "--root", root],
                          capture_output=True, text=True, cwd="/", timeout=120, env=env)
    err = (done.stdout + done.stderr).strip().split("\n")
    return done.returncode, err

CASES = []
def case(label, kind, **kw):
    CASES.append((label, kind, kw))

# ---- V1 ⑮ 括注
for note, label in [("（本机）", "括注 本机"), ("（登记值，不看忙闲）", "括注 不看忙闲"), ("（第二台十六）", "括注 中文数词 十六"), ("（两台各取登记值）", "括注 两台各"),
                    ("（一半）", "括注 一半"), ("（统一取登记值）", "括注 统一（统一不是数词）"), ("（唯一登记值）", "括注 唯一"), ("（十分稳定）", "括注 十分（副词）"),
                    ("（半角括号不认）", "括注 半角（说明词里有半）"), ("（各自取）", "括注 各自"), ("（不减半）", "括注 不减半"), ("（合计六十二）", "括注 合计"),
                    ("（三分之一）", "括注 分之"), ("（同上）", "括注 同上"), ("（每台默认）", "括注 每台默认"), ("（备用，随便写点什么说明）", "括注 任意说明"),
                    ("（万一内存不够）", "括注 万一"), ("（第1台）", "括注 阿拉伯数字"), ("(本机)", "半角括号"), ("（本机）（备注）", "两段括注"),
                    ("，本机", "整数后接逗号"), ("<登记值>", "尖括号 <登记值>"), ("<填 线程上限>", "尖括号 <填 …>（交 ⑬）"), ("<填线程上限>", "尖括号 <填线程上限>（无空格）"),
                    ("３１", "全角数字")]:
    case(f"V1 clerk 线程上限：{note if note[:1] in '<(（，３' else note}", "clerk", thread=f"线程上限：{note}" if note.startswith(("<", "３")) else (f"线程上限：31{note}"))
for kind in ["clerk", "three-way-forward", "three-way-defense", "tooling-writer", "investigator"]:
    case(f"V1 {kind} 没有线程上限行", kind, thread="")
    case(f"V1 {kind} 没有线程上限行、写已判", kind, thread="", extra="线程上限已判：这件活只读不编译\n")
    case(f"V1 {kind} 线程上限：30 没写已判", kind, thread="线程上限：30")
    case(f"V1 {kind} 线程上限：30 写已判", kind, thread="线程上限：30", extra="线程上限已判：用户给了参数\n")
    case(f"V1 {kind} 没有预计挂钟", kind, wall="")
    case(f"V1 {kind} 预计挂钟 超过 2 小时 不带三行", kind, wall="预计挂钟：超过 2 小时")
    case(f"V1 {kind} 预计挂钟 超过 2 小时 三行齐", kind, wall="预计挂钟：超过 2 小时",
         extra="分片：按用例切\n续跑：同一条命令重起只补没做完的片\n多机：台数取 layer0-shard-configuration-check.sh --emit-hosts，每台线程取 .claude/heavy-thread-default 的登记值\n")
# ---- V8 ⑭ 多机一行
OVER = "预计挂钟：超过 2 小时"
S2 = "分片：按用例切\n续跑：同一条命令重起只补没做完的片\n"
for text, label in [("台数取 layer0-shard-configuration-check.sh --emit-hosts，每台线程取 .claude/heavy-thread-default 的登记值", "多机 标准写法"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default 的登记值，不按核数现算", "多机 标准写法 + 不按核数现算（否定）"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default 的登记值，核数不进来", "多机 标准写法 + 核数"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，不看忙闲", "多机 不看忙闲"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，机器空闲时也不变", "多机 空闲"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，负载高也不降", "多机 负载"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，别的会话在跑也不降", "多机 在跑"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，不除以台数", "多机 除以"),
                    ("台数取多机配置，每台线程取登记的每台默认重活线程数", "多机 照 main-agent.md 字面写（不点名路径）"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，不写 available_parallelism", "多机 available_parallelism（否定）"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，不调 host-resources", "多机 host-resources（否定）"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，两台各 31 线程", "多机 写死 31 线程"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，包含全部卡", "多机 包含全部卡（无词表词）"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，用 nproc 之外的办法", "多机 nproc")]:
    case("V8 clerk " + label, "clerk", wall=OVER, extra=S2 + "多机：" + text + "\n")
case("V8 clerk 分不了 点名多机", "clerk", wall=OVER, extra="分片：按用例切\n续跑：同一条命令重起只补没做完的片\n分不了：多机没法分，本机单台跑完就好了\n")

# ---- V1 补：尖括号在括注里、前导零、加粗行首、句中
for tail, label in [("31（<登记值>）", "括注里的尖括号 <登记值>"), ("31（<按机器忙闲定>）", "括注里的尖括号 <按机器忙闲定>"), ("031", "前导零 031"), ("31.0", "31.0"),
                    ("31 （本机）", "整数与括注之间一个空格"), ("31　（本机）", "整数与括注之间全角空格"), ("+31", "+31"), ("三十一", "三十一"), ("31（本机）。", "行尾句号")]:
    case(f"V1 clerk 线程上限行写 {label}", "clerk", thread=f"线程上限：{tail}")
case("V1 clerk 加粗行首 - **线程上限**：31", "clerk", thread="- **线程上限**：31")
case("V1 clerk 线程上限写在句中", "clerk", thread="这件活的线程上限：31 写在句中")
case("V1 clerk 两行线程上限，一行对一行错", "clerk", thread="线程上限：31", extra="线程上限：16\n")

# ---- V2：线程上限已判 的理由不看内容
for reason, label in [("机器忙，负载高，让出 CPU", "理由 机器忙负载高（文字说只看到负载忙不算）"), ("用户在消息里明说让出 CPU", "理由 用户明说"), ("<理由>", "理由 占位 <理由>"),
                      ("无", "理由 无"), ("x", "理由 一个字 x"), ("随便", "理由 随便")]:
    case(f"V2 clerk N=16 + 线程上限已判：{label}", "clerk", thread="线程上限：16", extra=f"线程上限已判：{reason}\n")
case("V2 clerk N=16 + 已判行写在句中", "clerk", thread="线程上限：16", extra="我已经线程上限已判：用户明说让出 CPU\n")
case("V2 clerk 括注里写已判理由 31（线程上限已判：机器忙）", "clerk", thread="线程上限：31（线程上限已判：机器忙）")

# ---- V1/V8 交集：⑮ 括注不套 ⑭ 的现算词表
for note, label in [("（按核数取）", "括注 按核数取"), ("（忙时降低）", "括注 忙时降低"), ("（负载高就少起）", "括注 负载高就少起"), ("（nproc）", "括注 nproc"), ("（host-resources 现算）", "括注 host-resources 现算")]:
    case(f"V8x clerk ⑮ {label}", "clerk", thread=f"线程上限：31{note}")

for tail, label in [("÷2", "31÷2"), ("/2", "31/2"), (" 一半", "31 一半"), ("，即 15", "31，即 15"), ("（本机），第二台 16", "31（本机），第二台 16"), ("（÷）", "括注里 ÷"), ("（/）", "括注里 /")]:
    case(f"V1b clerk 线程上限行写 {label}", "clerk", thread=f"线程上限：31{tail}")

for note, label in [("（本机，与第二台同值）", "括注 与第二台同值"), ("（与另一台同值）", "括注 与另一台同值"), ("（两台同值）", "括注 两台同值"), ("（本机，不看忙闲）", "括注 本机不看忙闲"), ("（登记值）", "括注 登记值")]:
    case(f"V1n clerk ⑮ {label}", "clerk", thread=f"线程上限：31{note}")
ENV_CASES = [("V1e SINGLEFS_THREAD_CAP=16 时 clerk 线程上限：16", "线程上限：16"), ("V1e SINGLEFS_THREAD_CAP=16 时 clerk 线程上限：31", "线程上限：31")]

# ======== 第二轮新增用例（X1：只认整数、说明另起一行；X6：多机规范句） ========
CANON = "台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量"
S2 = "分片：按用例切\n续跑：同一条命令重起只补没做完的片\n"
# X1：线程上限说明那一行
for label, thread, extra in [
        ("X1 线程上限 31 + 另起一行 线程上限说明（正当说明）", "线程上限：31", "线程上限说明：本机，登记值，不看忙闲\n"),
        ("X1 线程上限 31 + 线程上限说明里写数字", "线程上限：31", "线程上限说明：双机时第二台也是 31\n"),
        ("X1 线程上限 31 + 线程上限说明里写 ÷ 与一半", "线程上限：31", "线程上限说明：不除以台数，不取一半 31/2\n"),
        ("X1 线程上限说明在前、线程上限在后", "线程上限说明：本机\n线程上限：31", ""),
        ("X1 只有线程上限说明、没有线程上限行", "", "线程上限说明：本机\n"),
        ("X1 只有线程上限说明里写 16、没有线程上限行", "", "线程上限说明：16\n"),
        ("X1 线程上限 16 + 线程上限说明", "线程上限：16", "线程上限说明：用户给了参数\n"),
        ("X1 线程上限 16 + 已判 + 线程上限说明", "线程上限：16", "线程上限已判：用户给了参数\n线程上限说明：本机\n"),
        ("X1 线程上限说明里套一行 线程上限：16（行首空格）", "线程上限：31", "线程上限说明：见下\n 线程上限：16\n"),
        ("X1 线程上限（说明）：31 全角括号在标签里", "线程上限（说明）：31", ""),
        ("X1 线程上限 31 半角冒号", "线程上限:31", ""),
        ("X1 线程上限： 31 冒号后空格", "线程上限： 31", ""),
        ("X1 线程上限：31 行尾空格", "线程上限：31 ", ""),
        ("X1 线程上限：31 行尾制表符", "线程上限：31\t", ""),
        ("X1 线程上限：31 行尾句号", "线程上限：31。", ""),
        ("X1 线程上限：31 行尾 半角分号", "线程上限：31;", ""),
        ("X1 线程上限：31 后接 行内 线程上限说明", "线程上限：31 线程上限说明：本机", ""),
        ("X1 线程上限：31（本机） 括注（R1 判过：放，R2 应拒）", "线程上限：31（本机）", ""),
        ("X1 线程上限：31(本机) 半角括注", "线程上限：31(本机)", ""),
        ("X1 线程上限：31 [本机] 方括号", "线程上限：31 [本机]", ""),
        ("X1 线程上限：31 <本机> 尖括号", "线程上限：31 <本机>", ""),
        ("X1 线程上限：31 # 本机 井号注释", "线程上限：31 # 本机", ""),
        ("X1 线程上限：31 -- 本机", "线程上限：31 -- 本机", ""),
        ("X1 两行线程上限：31 与 31", "线程上限：31", "线程上限：31\n"),
        ("X1 线程上限：31 加粗标签 **线程上限**：31", "**线程上限**：31", ""),
        ("X1 线程上限：31 列表项 - 线程上限：31", "- 线程上限：31", ""),
        ("X1 线程上限：31 行首空格", "  线程上限：31", ""),
        ("X1 线程上限：31 引用 > 线程上限：31", "> 线程上限：31", ""),
        ("X1 线程上限：031", "线程上限：031", ""),
        ("X1 线程上限：+31", "线程上限：+31", ""),
        ("X1 线程上限：31.0", "线程上限：31.0", ""),
        ("X1 线程上限：3１ 混全角", "线程上限：3１", ""),
        ("X1 线程上限已判 顶缺行 理由=这件活只读", "", "线程上限已判：这件活只读\n"),
        ("X1 线程上限已判 理由空", "", "线程上限已判：\n"),
        ("X1 线程上限已判 理由=无", "", "线程上限已判：无\n"),
        ("X1 线程上限已判 理由=<理由>", "", "线程上限已判：<理由>\n"),
        ("X1 线程上限已判 理由=没有", "", "线程上限已判：没有\n"),
        ("X1 线程上限已判 写在括注里 线程上限：31（线程上限已判：忙）", "线程上限：31（线程上限已判：忙）", ""),
        ("X1 线程上限已判 理由里写 机器忙", "", "线程上限已判：机器忙，降一半\n"),
        ("X1 线程上限说明 当成已判的替身（说明行写 已判）", "", "线程上限说明：线程上限已判，这件活只读\n"),
]:
    for kind in ["clerk", "three-way-forward", "three-way-defense", "investigator"]:
        if kind != "clerk" and not label.startswith(("X1 线程上限 31 + 另起一行", "X1 只有线程上限说明、", "X1 线程上限已判 顶缺行", "X1 线程上限：31（本机） 括注")):
            continue
        case(f"{label} [{kind}]", kind, thread=thread, extra=extra)
# X6：多机一行
for text, label in [(CANON, "规范句原样"), (CANON + "；双机时各取一份", "规范句 + ；说明（无数字）"), (CANON + "；每台 31 线程", "规范句 + ；说明含数字"),
                    (CANON + "；每台取十六", "规范句 + ；说明含中文数词（只拦数字）"), (CANON + "；取登记值的一半", "规范句 + ；说明写一半"),
                    (CANON + "；按此刻余量少给", "规范句 + ；说明写按余量少给"), (CANON + "；按核数现算", "规范句 + ；说明写按核数现算"),
                    (CANON + "；不看忙闲", "规范句 + ；不看忙闲（定义原话）"), (CANON + "，不看忙闲", "规范句 + ，不看忙闲（逗号）"), (CANON + "（本机）", "规范句 + 括注"),
                    (CANON + "；", "规范句 + 空说明"), (CANON + "；两台各取一份，机器忙时降", "规范句 + ；机器忙时降"),
                    (CANON + "；每台线程取１６", "规范句 + ；全角数字"), (CANON + "；每台线程取 sixteen", "规范句 + ；英文数词"),
                    (CANON + "；available_parallelism", "规范句 + ；available_parallelism"), (CANON + "；host-resources", "规范句 + ；host-resources 现算"),
                    ("台数取 layer0-shard-configuration-check.sh --emit-hosts，每台线程取 .claude/heavy-thread-default 的登记值", "R1 标准写法（改前能过）"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，不看忙闲", "R1 的 不看忙闲（改前拒）"),
                    ("台数取多机配置，每台线程取登记的每台默认重活线程数", "照 main-agent 字面（改前拒）"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default 的一半", "R1 M6 一半（改前放）"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default（31）", "R1 M4 括注 31（改前放）"),
                    ("台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default 的登记值，按此刻余量少给", "R1 M7 余量少给（改前放）"),
                    (CANON.replace("，", ","), "规范句 半角逗号"), (CANON + " ", "规范句 + 行尾空格"), (" " + CANON, "规范句 前导空格"),
                    ("「" + CANON + "」", "规范句 带直角引号"), ("`" + CANON + "`", "规范句 带反引号"), (CANON.replace("  ", " ").replace(" 读", "读"), "规范句 少一个空格"),
                    ("<填 超过 2 小时才写：逐字写这一句「" + CANON + "」>", "骨架占位原样（未填）")]:
    case("X6 clerk 多机 " + label, "clerk", wall="预计挂钟：超过 2 小时", extra=S2 + "多机：" + text + "\n")
case("X6 clerk 分不了 点名多机", "clerk", wall="预计挂钟：超过 2 小时", extra=S2.split("\n")[0] + "\n续跑：同一条命令重起只补没做完的片\n分不了：多机没法分，本机单台跑完就好了\n")
case("X6 experiment-designer 多机 规范句原样", "clerk", wall="预计挂钟：超过 2 小时", extra=S2 + "多机：" + CANON + "\n")

def short(text):
    shown = []
    lines = text.strip().split("\n")
    for index, line in enumerate(lines):
        if line.strip().startswith("✗"):
            shown.append(line.strip()[:140])
            break
    return shown[0] if shown else "放行"

import re
for label, kind, kw in CASES:
    if args.only not in label:
        continue
    text = base(kind, **kw)
    tag = "".join(ch if ch.isalnum() else "_" for ch in label)[:50] + str(abs(hash(label)) % 100000)
    results = []
    for name, root in ROOTS:
        code, err = run(root, kind, text, f"{tag}-{name}")
        results.append((name, code, err))
        if args.raw:
            with open(args.raw, "a", encoding="utf-8") as raw:
                raw.write(f"=== {label}\n--- {name} 退出={code}\n" + "\n".join(err) + "\n")
    print(label + "\t" + "\t".join(f"{name}={code}" for name, code, _ in results) + "\t" + short("\n".join(results[0][2])))
