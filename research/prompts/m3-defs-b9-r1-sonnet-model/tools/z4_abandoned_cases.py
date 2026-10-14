#!/usr/bin/env python3
"""Z4：叫停登记行的判法（派发闸 ⑥ 与门禁 abandoned-rounds 格共用的 lib_abandoned_rounds.py）。
python3 z4_cases.py <树> <临时根目录>；每例在 <临时根目录>/<例名>/ 造 records/f.md、research/prompts/ 的轮名桩与登记表，然后跑
  python3 -B <树>/.claude/hooks/lib_abandoned_rounds.py <那个根>   打 例名 / 退出码 / 输出首行"""
import os, shutil, subprocess, sys
tree, base = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
lib = os.path.join(tree, ".claude/hooks/lib_abandoned_rounds.py")
R = "zz-ab-r1"
REC = f"# 记录\n- 用户 {R} 叫停：「这一轮先不做了」\n其它\n"
def row(source, name=R, date="2026-10-13"):
    return f"{name}\t{date}\t用户叫停\t{source}\n"
# (例名, 记录正文, 登记行, 额外桩文件)
cases = [
 ("基线", REC, row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("冒号写法", REC, row(f"records/f.md:2〔{R} 叫停〕"), []),
 ("原话不含轮名而那一行含", REC, row("records/f.md 第 2 行〔叫停：「这一轮先不做了」〕"), []),
 ("原话只有一小段不含轮名", REC, row("records/f.md 第 2 行〔这一轮先不做了〕"), []),
 ("行号写错原话对", REC, row(f"records/f.md 第 99 行〔{R} 叫停〕"), []),
 ("行号写成第 0 行", REC, row(f"records/f.md 第 0 行〔{R} 叫停〕"), []),
 ("没有行号只有原话", REC, row(f"records/f.md〔{R} 叫停〕"), []),
 ("有行号没带原话", REC, row("records/f.md 第 2 行"), []),
 ("别的会话在前面插了两行", "新插一\n新插二\n" + REC, row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("别的会话在后面追加了逐字重复的一行", REC + f"- 用户 {R} 叫停：「这一轮先不做了」\n", row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("别的会话追加了一行只提到原话那几个字", REC + f"补记：{R} 叫停之后又有人问过\n", row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("别的会话追加的一行提到更长的轮名", REC + f"补记：{R}0 叫停之后又有人问过\n", row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("别的会话把叫停改成喊停", REC.replace("叫停", "喊停"), row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("别的会话把那一行折成两行", f"# 记录\n- 用户 {R}\n叫停：「这一轮先不做了」\n其它\n", row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("别的会话改了那一行里原话以外的字", REC.replace("先不做了", "先不做了（已确认）"), row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("别的会话改了那一行里原话以外的字且原话抄得长", REC, row(f"records/f.md 第 2 行〔- 用户 {R} 叫停：「这一轮先不做了」〕"), []),
 ("记录文件是回车换行", REC.replace("\n", "\r\n"), row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("原话里冒号半角全角不同", REC, row(f"records/f.md 第 2 行〔{R} 叫停:「这一轮先不做了」〕"), []),
 ("同一行里原话命中两次", f"# 记录\n- 用户 {R} 叫停 {R} 叫停：「这一轮先不做了」\n", row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("两处出处一坏一好", REC, row(f"records/f.md 第 99 行〔找不到的话〕 records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("那一行有大写的别的轮名", REC.replace("先不做了」", "先不做了」；Foo-r2 照常跑"), row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("那一行有小写的别的轮名", REC.replace("先不做了」", "先不做了」；foo-r2 照常跑"), row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("那一行有带下划线的别的轮名", REC.replace("先不做了」", "先不做了」；foo_bar-r2 照常跑"), row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("那一行有带点的别的轮名", REC.replace("先不做了」", "先不做了」；foo.bar-r2 照常跑"), row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("那一行有本轮名加一位数字", REC.replace("先不做了」", f"先不做了」；{R}0 照常跑"), row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("那一行有仓里以 -main-verification.md 认出的非 r 形名", REC.replace("先不做了」", "先不做了」；qq-close-b1 照常跑"), row(f"records/f.md 第 2 行〔{R} 叫停〕"),
  ["research/prompts/qq-close-b1-main-verification.md"]),
 ("那一行有仓里以 -body.md 认出的非 r 形名", REC.replace("先不做了」", "先不做了」；qq-close-b1 照常跑"), row(f"records/f.md 第 2 行〔{R} 叫停〕"),
  ["research/prompts/_qq-close-b1-body.md"]),
 ("那一行有仓里只有 -verdict.md 的非 r 形名", REC.replace("先不做了」", "先不做了」；qq-close-b1 照常跑"), row(f"records/f.md 第 2 行〔{R} 叫停〕"),
  ["research/prompts/qq-close-b1-verdict.md"]),
 ("那一行有仓里只有腿产出的非 r 形名", REC.replace("先不做了」", "先不做了」；qq-close-b1 照常跑"), row(f"records/f.md 第 2 行〔{R} 叫停〕"),
  ["research/prompts/qq-close-b1-opus-output.md"]),
 ("原话不到六个字", REC.replace("「这一轮先不做了」", "「先停」"), row(f"records/f.md 第 2 行〔{R} 叫停〕"), []),
 ("出处指到 research 下", REC, row(f"research/prompts/f.md 第 2 行〔{R} 叫停〕"), []),
 ("别的会话逐字重复了整行而原话已抄整行", REC + f"- 用户 {R} 叫停：「这一轮先不做了」\n", row(f"records/f.md 第 2 行〔- 用户 {R} 叫停：「这一轮先不做了」〕"), []),
 ("旧行名单里的名字可以再登记且没有出处", REC, "c196-r1\t2026-10-13\t随便写的理由\t随便写的去向\n", []),
 ("名单外的名字没有出处", REC, f"{R}\t2026-10-13\t随便写的理由\t随便写的去向\n", []),
]
os.makedirs(base, exist_ok=True)
for name, record, table_row, stubs in cases:
    root = os.path.join(base, name)
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(os.path.join(root, "records")); os.makedirs(os.path.join(root, "research/prompts"))
    with open(os.path.join(root, "records/f.md"), "w", newline="") as handle: handle.write(record)
    with open(os.path.join(root, "research/prompts/abandoned-rounds.tsv"), "w") as handle: handle.write(table_row)
    for stub in stubs:
        open(os.path.join(root, stub), "w").close()
    r = subprocess.run(["python3", "-B", lib, root], capture_output=True, text=True)
    first = (r.stdout.strip().splitlines() or [""])[0]
    print(f"{name}\t{r.returncode}\t{first[:230]}")
