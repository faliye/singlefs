#!/usr/bin/env python3
"""第二轮模型自己的判别力自证：每条检查在一个已知该红的配置上红、在它的改法上不红。任何一条对不上退出码 1。"""
from model2 import DEFAULT, build, search, Geometry
from run2 import GM, LM, LM_OPENS, GX, LX, LX_OPENS, LM2


def found(geo, layout, opens, cfg, alphabet, depth, kinds):
    return search(geo, cfg, build(geo, layout, opens=opens), set(alphabet), [1], depth, 3000000, set(kinds))[0]


def selftest():
    failures = []

    def expect(label, red, green, kind):
        if kind not in red:
            failures.append(f"{kind} 应当在「{label}」的红臂上红")
        if kind in green:
            failures.append(f"{kind} 不应在「{label}」的改法上红")

    base = DEFAULT._replace(R="A", S="C", E="rec", SP="AC", move="unit")
    rb = ["SIGN", "BATCH", "PUBLISH", "ROLLBACK"]
    k3 = ("c_lost", "rp_ahead", "never", "stuck", "r3", "resurrect")
    expect("意图随超级块写 + 回退（R_old 里意图活着、超级块上已删）",
           found(GM, LM, LM_OPENS, base._replace(place="SB", P="B"), rb, 6, k3),
           found(GM, LM, LM_OPENS, base._replace(place="ROOT", P="B"), rb, 6, k3), "c_lost")
    expect("P甲 + 意图随超级块写 + 回退：读回的续做点在所选根那份之后",
           found(GM, LM, LM_OPENS, base._replace(place="SB", P="A"), rb, 5, k3),
           found(GM, LM, LM_OPENS, base._replace(place="ROOT", P="A"), rb, 5, k3), "rp_ahead")
    expect("只在 journal 里、每次发布重记，回退到 journal 已覆盖的根",
           found(GM, LM, LM_OPENS, base._replace(place="JEACH", P="B"), rb, 6, k3),
           found(GM._replace(journal_keep=10), LM, LM_OPENS, base._replace(place="JEACH", P="B"), rb, 6, k3), "c_lost")
    crash = ["SIGN", "BATCH", "PUBLISH", "CRASH"]
    expect("只在 journal 里、只在变了时记，普通崩溃",
           found(GM, LM, LM_OPENS, base._replace(place="JCHG", P="B"), crash, 6, k3),
           found(GM, LM, LM_OPENS, base._replace(place="JEACH", P="B"), crash, 6, k3), "c_lost")
    att = ["SIGN", "BATCH", "WRITE"]
    red = found(GX, LX, LX_OPENS, base._replace(R="B", P="A", exclusion="published", move="copy"), att, 3, k3)
    expect("P甲 + 排除读已发布意图（多盘、R乙）：目的地落回续做点之前",
           red, found(GX, LX, LX_OPENS, base._replace(R="B", P="A", exclusion="published", move="copy", wrap=True),
                      att, 3, k3), "stuck")
    expect("P甲 + 排除读已发布意图：改成读在飞意图",
           red, found(GX, LX, LX_OPENS, base._replace(R="B", P="A", exclusion="inflight", move="copy"), att, 3, k3),
           "stuck")
    att_c = att + ["PUBLISH", "CRASH"]
    expect("P甲 卡住之后崩溃，续做在界内删不掉意图",
           found(GX, LX, LX_OPENS, base._replace(R="B", P="A", exclusion="published", move="copy"), att_c, 5, k3),
           found(GX, LX, LX_OPENS, base._replace(R="B", P="B", exclusion="published", move="copy"), att_c, 5, k3),
           "never")
    s_kinds = ("a", "b", "b_fn", "c")
    w4 = ["SIGN", "BATCH"]
    expect("多盘 R甲：全池之和把 dev2 上没产出的段算进来",
           found(GX, LX, LX_OPENS, DEFAULT._replace(R="A", S="B", E="rec", move="copy"), w4, 3, s_kinds),
           found(GX, LX, LX_OPENS, DEFAULT._replace(R="A", S="A", E="rec", move="copy"), w4, 3, s_kinds), "a")
    expect("多盘 R甲 + S甲：R 在 dev2 上签发时就空，谓词判不出真产出",
           found(GX, LX, LX_OPENS, DEFAULT._replace(R="A", S="A", E="rec", move="copy"), w4, 3, s_kinds),
           found(GX, LX, LX_OPENS, DEFAULT._replace(R="B", S="A", E="rec", move="copy"), w4, 3, s_kinds), "b_fn")
    fg = ["SIGN", "BATCH", "DELETE", "PUBLISH"]
    expect("两盘 S甲 + 谓词真就删意图 + 前台释放",
           found(GM, LM, LM_OPENS, DEFAULT._replace(S="A", SP="SD", E="rec", move="unit"), fg, 5, s_kinds),
           found(GM, LM, LM_OPENS, DEFAULT._replace(S="A", SP="AC", E="rec", move="unit"), fg, 5, s_kinds), "c")
    for failure in failures:
        print(f"C364R2SELFTEST FAIL {failure}")
    print(f"C364R2SELFTEST {'ok' if not failures else 'red'} checks=20 failures={len(failures)}")
    return not failures


if __name__ == "__main__":
    import sys
    sys.exit(0 if selftest() else 1)
