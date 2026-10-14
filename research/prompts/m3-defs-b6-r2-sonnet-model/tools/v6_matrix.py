#!/usr/bin/env python3
"""X4：叫停轮登记行的判法（lib_abandoned_rounds.py）与派发闸 ⑥ 端到端，after / before 两份冻结副本各判一遍。
每例造一棵小树（records/log.md、research/prompts/abandoned-rounds.tsv、一个开着的三方轮 mtest-r1 的开工快照与正文，可再加别的轮的材料文件），
库两种读法：默认（⑥ 的读法）、--legacy-ok（第一轮门禁格的读法；第二轮的库已无这个开关，after 只判默认读法）。派发闸：--check-prompt 派 implementation-writer 点名快照里的文件，⑥ 拒不拒。
用法：v6_matrix.py --root after=<仓> --root before=<仓> --work <草稿目录> [--only 子串]"""
import argparse, os, shutil, subprocess, sys

ap = argparse.ArgumentParser()
ap.add_argument("--root", action="append", required=True)
ap.add_argument("--work", required=True)
ap.add_argument("--only", default="")
args = ap.parse_args()
ROOTS = [tuple(item.split("=", 1)) for item in args.root]
os.makedirs(args.work, exist_ok=True)
ROUND = "mtest-r1"
SRC = "records/log.md 第 1 行"
Q = "「先放一放这一轮吧」"

def build(tree, rows, records, extras, agents_from):
    if os.path.exists(tree):
        shutil.rmtree(tree)
    os.makedirs(os.path.join(tree, "records"))
    os.makedirs(os.path.join(tree, "research/prompts", f"{ROUND}-snapshot"))
    shutil.copytree(os.path.join(agents_from, ".claude/agents"), os.path.join(tree, ".claude/agents"))
    with open(os.path.join(tree, "records/log.md"), "w", encoding="utf-8") as handle:
        handle.write(records)
    with open(os.path.join(tree, f"research/prompts/_{ROUND}-body.md"), "w", encoding="utf-8") as handle:
        handle.write("# body\n")
    with open(os.path.join(tree, f"research/prompts/{ROUND}-snapshot/frozen-sha256.txt"), "w", encoding="utf-8") as handle:
        handle.write("0123456789abcdef" * 4 + "  crates/foo/src/lib.rs\n")
    with open(os.path.join(tree, "research/prompts/abandoned-rounds.tsv"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(rows) + "\n")
    for relative in extras:
        path = os.path.join(tree, "research/prompts", relative)
        if relative.endswith("/"):
            os.makedirs(path, exist_ok=True)
        else:
            os.makedirs(os.path.dirname(path), exist_ok=True)
            open(path, "w", encoding="utf-8").write("x\n")

def lib_verdict(root, tree, legacy):
    argv = ["python3", "-B", "-I", os.path.join(root, ".claude/hooks/lib_abandoned_rounds.py"), tree] + (["--legacy-ok"] if legacy else [])
    done = subprocess.run(argv, capture_output=True, text=True, cwd="/")
    if done.returncode == 2 and not done.stdout:
        return "(无此读法)"
    lines = [line.replace(args.work, "<WORK>") for line in done.stdout.split("\n") if "\t" in line and not line.startswith(" ")]
    return " ; ".join(f"{line.split(chr(9), 1)[0]}→{line.split(chr(9), 1)[1][:70]}" for line in lines) or "(无此行)"

def guard_verdict(root, tree):
    prompt = os.path.join(tree, "prompt.md")
    with open(prompt, "w", encoding="utf-8") as handle:
        handle.write("重型测试：不跑\n草稿目录：~/.local/share/singlefs/work/m3-defs-b6-r2-sonnet/x/\n报告：~/.local/share/singlefs/work/m3-defs-b6-r2-sonnet/x/r.md\n"
                     "条款：x\n要动的 crates 文件：crates/foo/src/lib.rs\n出口：x\n预计挂钟：2 小时内\n线程上限：31\n")
    done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", prompt, "--type", "implementation-writer", "--root", tree],
                          capture_output=True, text=True, cwd="/")
    text = done.stdout + done.stderr
    return "⑥拒" if "开工快照里" in text else ("⑥放行" if done.returncode in (0, 2) else f"退出{done.returncode}")

def row(name, date, reason, destination="并进别处"):
    return "\t".join([name, date, reason, destination])

CASES = []
def case(label, rows, records, extras=()):
    CASES.append((label, rows, records, tuple(extras)))

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
# ---- 第二轮新增
case("X4 别的轮名 m3-close-b1，树里没有它的材料（第一轮攻击树原样）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 m3-close-b1 {Q}\n")
case("X4 别的轮名 m3-close-b1，树里有 _m3-close-b1-body.md", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 m3-close-b1 {Q}\n", ["_m3-close-b1-body.md"])
case("X4 别的轮名 m3-close-b1，树里只有 m3-close-b1-snapshot/", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 m3-close-b1 {Q}\n", ["m3-close-b1-snapshot/"])
case("X4 别的轮名 m3-close-b1，树里只有 m3-close-b1-main-verification.md", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 m3-close-b1 {Q}\n", ["m3-close-b1-main-verification.md"])
case("X4 别的轮名 m3-close-b1，树里只有 m3-close-b1-opus-output.md（腿报告，不在三种材料里）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 m3-close-b1 {Q}\n", ["m3-close-b1-opus-output.md"])
case("X4 别的轮名 other-r2，树里没有它的材料（R1 判过：形状认得，不收）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 other-r2 {Q}\n")
case("X4 别的轮名 other-r2，树里有 _other-r2-body.md", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 other-r2 {Q}\n", ["_other-r2-body.md"])
case("X4 别的轮名大写 OTHER-R2，树里有 _other-r2-body.md（大小写）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 OTHER-R2 {Q}\n", ["_other-r2-body.md"])
case("X4 别的轮名 Other-r2 混合大小写，树里有 _other-r2-body.md", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 Other-r2 {Q}\n", ["_other-r2-body.md"])
case("X4 别的轮名是已归档的旧轮（材料被删），登记表也没有", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 old-archived-r3 {Q}\n")
case("X4 别的轮名写成材料文件名 _other-r2-body.md（带下划线前缀）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 _other-r2-body.md {Q}\n", ["_other-r2-body.md"])
case("X4 别的轮名在路径里 research/prompts/other-r2-opus-output.md，树里有 _other-r2-body.md", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 见 research/prompts/other-r2-opus-output.md {Q}\n", ["_other-r2-body.md"])
case("X4 别的轮名是本轮名的前缀扩展 mtest-r10（树里有它的 body）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} 与 mtest-r10 {Q}\n", ["_mtest-r10-body.md"])
case("X4 别的轮名是本轮名的子串 test-r1（树里有 _test-r1-body.md）", [row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} {Q}\n", ["_test-r1-body.md"])
case("X4 名单内旧行 c196-r1 日期 2026-10-11 无出处，且是另一轮 mtest 的 c196-r1 登记行", [row("c196-r1", "2026-10-11", "旧行没出处"), row(ROUND, "2026-10-13", f"出处 {SRC}")], f"{ROUND} {Q}\n")
case("X4 新叫停轮日期 2026-10-11（腿发出日）无出处（R1 的 A 形）", [row(ROUND, "2026-10-11", "用户叫停，没出处")], f"{ROUND} {Q}\n")
case("X4 新叫停轮日期 2000-01-01 无出处（拿旧日期绕）", [row(ROUND, "2000-01-01", "用户叫停，没出处")], f"{ROUND} {Q}\n")
case("X4 轮名改成名单里的 c196-r1 并写新日期无出处（占名单名）", [row("c196-r1", "2026-10-13", "用户叫停，没出处")], "c196-r1 " + Q + "\n")
case("X4 名单名带空白 ' c196-r1 '", [row(" c196-r1 ", "2026-10-13", "用户叫停，没出处")], "c196-r1 " + Q + "\n")
case("X4 名单名大小写 C196-R1", [row("C196-R1", "2026-10-13", "用户叫停，没出处")], "C196-R1 " + Q + "\n")

for label, rows, records, extras in CASES:
    if args.only not in label:
        continue
    out = []
    for name, root in ROOTS:
        tree = os.path.join(args.work, f"v6tree-{name}")
        build(tree, rows, records, extras, root)
        lib = lib_verdict(root, tree, False)
        legacy = lib_verdict(root, tree, True) if name != "after" else "-"
        out.append(f"{name}: 库={lib} | 门禁读法={legacy} | 派发闸={guard_verdict(root, tree)}")
    print(label + "\n    " + "\n    ".join(out))
