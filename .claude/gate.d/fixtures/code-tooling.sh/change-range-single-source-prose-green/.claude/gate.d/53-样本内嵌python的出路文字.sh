#!/usr/bin/env bash
# 绿样本：内嵌 python 的出路文字点名逐行判的文件从哪条命令列出，那一行不跑 git；行尾带豁免标记与理由
python3 - <<'PY'
def fail(what, howto):
    raise SystemExit(f"{what}：{howto}")


fail("列不出跟踪的文件", "看 git 能不能用；逐行判的是 git ls-files 列出的每一份。")  # change-range:exempt 出路文字点名文件从哪条命令列出，这一行不跑 git
PY
