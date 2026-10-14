#!/usr/bin/env python3
"""V6：叫停轮登记行的三条判法（lib_abandoned_rounds.py）与派发闸 ⑥ 的端到端。
每个用例在草稿目录里造一棵小树（records/log.md、research/prompts/abandoned-rounds.tsv、一个开着的三方轮的开工快照），
然后 (1) 用库判一遍：默认读法（⑥ 用）与 --legacy-ok 读法（门禁格用）；(2) 起派发闸 --check-prompt 派 implementation-writer 点名快照里的文件，看 ⑥ 拒不拒。
用法：v6.py --frozen <冻结副本根> --work <草稿目录>"""
import argparse, os, shutil, subprocess, sys

ap = argparse.ArgumentParser()
ap.add_argument("--frozen", required=True)
ap.add_argument("--work", required=True)
ap.add_argument("--only", default="")
args = ap.parse_args()

LIB = os.path.join(args.frozen, ".claude/hooks/lib_abandoned_rounds.py")
HOOK = os.path.join(args.frozen, ".claude/hooks/runner-dispatch-guard.sh")
ROUND = "mtest-r1"

def build(tree, rows, records):
    if os.path.exists(tree):
        shutil.rmtree(tree)
    os.makedirs(os.path.join(tree, "records"))
    os.makedirs(os.path.join(tree, "research/prompts", f"{ROUND}-snapshot"))
    shutil.copytree(os.path.join(args.frozen, ".claude/agents"), os.path.join(tree, ".claude/agents"))
    with open(os.path.join(tree, "records/log.md"), "w", encoding="utf-8") as handle:
        handle.write(records)
    with open(os.path.join(tree, f"research/prompts/_{ROUND}-body.md"), "w", encoding="utf-8") as handle:
        handle.write("# body\n")
    with open(os.path.join(tree, f"research/prompts/{ROUND}-snapshot/frozen-sha256.txt"), "w", encoding="utf-8") as handle:
        handle.write("0123456789abcdef" * 4 + "  crates/foo/src/lib.rs\n")
    with open(os.path.join(tree, "research/prompts/abandoned-rounds.tsv"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(rows) + "\n")

def lib_verdict(tree, legacy):
    done = subprocess.run(["python3", "-I", LIB, tree] + (["--legacy-ok"] if legacy else []), capture_output=True, text=True, cwd="/")
    lines = [line for line in done.stdout.split("\n") if line.startswith(ROUND + "\t")]
    return (lines[0].split("\t", 1)[1] if lines else "(无此行)")

def guard_verdict(tree):
    prompt = os.path.join(tree, "prompt.md")
    with open(prompt, "w", encoding="utf-8") as handle:
        handle.write("重型测试：不跑\n草稿目录：~/.local/share/singlefs/work/m3-defs-b6-r1-sonnet/x/\n报告：~/.local/share/singlefs/work/m3-defs-b6-r1-sonnet/x/r.md\n"
                     "条款：x\n要动的 crates 文件：crates/foo/src/lib.rs\n出口：x\n预计挂钟：2 小时内\n线程上限：31\n")
    done = subprocess.run(["bash", HOOK, "--check-prompt", prompt, "--type", "implementation-writer", "--root", tree], capture_output=True, text=True, cwd="/")
    text = done.stdout + done.stderr
    return "⑥ 拒（轮仍算开着）" if "开工快照里" in text else ("⑥ 放行" if done.returncode in (0, 2) else f"退出 {done.returncode}")

def row(name, date, reason, destination="并进别处"):
    return "\t".join([name, date, reason, destination])

CASES = []
def case(label, rows, records):
    CASES.append((label, rows, records))

SRC = "records/log.md 第 1 行"
Q = "「先放一放这一轮吧」"
case("基线：新日期、出处合格", [row(ROUND, "2026-10-13", f"用户叫停，出处 {SRC}")], f"{ROUND} {Q}\n")
case("日期 2026-10-11 没写出处（门禁免、⑥ 要）", [row(ROUND, "2026-10-11", "用户叫停，没出处")], f"{ROUND} {Q}\n")
case("日期 2026-10-12 没写出处（边界日，两边都要）", [row(ROUND, "2026-10-12", "用户叫停，没出处")], f"{ROUND} {Q}\n")
case("日期 2000-01-01 没写出处（拿旧日期绕门禁）", [row(ROUND, "2000-01-01", "用户叫停，没出处")], f"{ROUND} {Q}\n")
case("日期 2026-13-45（格式对、日历上不存在）", [row(ROUND, "2026-13-45", f"用户叫停 {SRC}")], f"{ROUND} {Q}\n")
case("日期写 2026/10/13", [row(ROUND, "2026/10/13", f"用户叫停 {SRC}")], f"{ROUND} {Q}\n")
case("出处那一行另有 <名>-r<数字> 形的轮名", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 other-r2 {Q}\n")
case("出处那一行另有不以 -r<数字> 结尾、也没登记的别的轮名 m3-close-b1", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 m3-close-b1 {Q}\n")
case("出处那一行另有登记表里出现过的别的轮名（无 -r 形）", [row(ROUND, "2026-10-13", f"出处 {SRC}"), row("oldround", "2026-09-01", "旧行没出处")], f"{ROUND} 与 oldround {Q}\n")
case("出处那一行另有大写的别的轮名 OTHER-R2", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 OTHER-R2 {Q}\n")
case("出处那一行另有别的轮名，下一行才有这一轮的原话（登记指第 1 行）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 other-r2\n{Q}\n")
case("轮名后面紧跟中文字符", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND}这一轮 {Q}\n")
case("轮名前后连着连字符（是别的轮名的一部分）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"x-{ROUND}-y {Q}\n")
case("原话 5 个字", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 「先放一放」\n")
case("原话 6 个字（去空白后）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 「先 放 一 放 吧 吧」\n")
case("原话里嵌套引号 「他说「先放一放」好吧」", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 「他说「先放一放」好吧」\n")
case("出处写 :1", [row(ROUND, "2026-10-13", "出处 records/log.md:1")], f"{ROUND} {Q}\n")
case("出处写 第1行（无空格）", [row(ROUND, "2026-10-13", "出处 records/log.md 第1行")], f"{ROUND} {Q}\n")
case("出处行号越界", [row(ROUND, "2026-10-13", "出处 records/log.md 第 9 行")], f"{ROUND} {Q}\n")
case("出处路径带 ..", [row(ROUND, "2026-10-13", "出处 records/../records/log.md 第 1 行")], f"{ROUND} {Q}\n")
case("两处出处：一处错一处对", [row(ROUND, "2026-10-13", "出处 records/log.md 第 9 行；又见 records/log.md 第 1 行")], f"{ROUND} {Q}\n")
case("出处写在第三列理由里、第四列去向空着不算（四列有空）", [row(ROUND, "2026-10-13", f"出处 {SRC}", "")], f"{ROUND} {Q}\n")
case("开轮行：records 里那一行是开轮并引了一句规则里的话", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"开 {ROUND} 第一轮，规则写「用户叫停的轮要登记」\n")
case("同一轮两行（都合格）", [row(ROUND, "2026-10-13", f"出处 {SRC}"), row(ROUND, "2026-10-13", f"出处 {SRC} 重复")], f"{ROUND} {Q}\n")
case("同一轮两行（一行日期写坏）", [row(ROUND, "2026-10-13", f"出处 {SRC}"), row(ROUND, "bad", f"出处 {SRC} 重复")], f"{ROUND} {Q}\n")
case("轮名第一列带首尾空格", [row(" " + ROUND + " ", "2026-10-13", f"出处 {SRC}")], f"{ROUND} {Q}\n")
case("出处指到 research/ 下这一轮自己的材料", [row(ROUND, "2026-10-13", "出处 research/prompts/_mtest-r1-body.md 第 1 行")], f"{ROUND} {Q}\n")

for label, rows, records in CASES:
    if args.only not in label:
        continue
    tree = os.path.join(args.work, "v6-tree")
    build(tree, rows, records)
    print(f"{label}\n    库默认读法（⑥ 用）：{lib_verdict(tree, False)}\n    库 --legacy-ok 读法（门禁格用）：{lib_verdict(tree, True)}\n    派发闸 --check-prompt：{guard_verdict(tree)}")
