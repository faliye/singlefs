#!/usr/bin/env python3
"""L2 改法草稿 K3（只在攻方自己的副本上改、只在 l2-install.sh 上量过，被攻过零轮）：python3 fix-draft-l2.py <仓副本根>
tool-versions 另打 rust_verify 读的两个环境变量：VERUS_ROOT（未设 / 空串 / 值加那个目录顶层全部普通文件的整体摘要：设了它，宏库、vstd 从那里读）
与 VERUS_USE_RUSTUP（未设 / 原值：设成 0 时 verus 不经 rustup 起 rust_verify）。"""
import sys

path = sys.argv[1] + "/research/scripts/verus-proof-check.py"
text = open(path, encoding="utf-8").read()
old = '        z3_override = os.environ.get("VERUS_Z3_PATH")\n'
new = ('        for variable in ("VERUS_ROOT", "VERUS_USE_RUSTUP"):   # K3\n'
       '            value = os.environ.get(variable)\n'
       '            shown = "未设" if value is None else ("空串" if value == "" else value)\n'
       '            if variable == "VERUS_ROOT" and value and os.path.isdir(value):\n'
       '                every_root = hashlib.sha256()\n'
       '                for name in sorted(os.listdir(value), key=lambda entry: entry.encode("utf-8", "surrogateescape")):\n'
       '                    if os.path.isfile(os.path.join(value, name)):\n'
       '                        every_root.update(name.encode("utf-8", "surrogateescape") + b"\\0" + file_sha256(os.path.join(value, name)).encode() + b"\\n")\n'
       '                shown += ":" + every_root.hexdigest()\n'
       '            extra += f" {variable}={shown}"\n'
       + old)
if text.count(old) != 1:
    sys.exit(f"没恰好命中一次（{text.count(old)} 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))
print("改了 1 处")
