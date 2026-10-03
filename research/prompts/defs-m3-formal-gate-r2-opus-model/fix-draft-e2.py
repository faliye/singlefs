#!/usr/bin/env python3
"""E2 草稿改法 G10（只打在草稿拷贝里，被攻过零轮）：tool-versions 另打安装目录顶层全部普通文件的整体摘要（不只五样），
VERUS_Z3_PATH 分「未设」与「空串」，值里没有 / 的照 PATH 找到那一份再哈希（Verus 自己就是这样找的，e2-verus-toy.sh 的 V3）。
用法：python3 fix-draft-e2.py <拷贝根>（就地改拷贝里的 research/scripts/verus-proof-check.py；旧串要恰好命中一次）。"""
import os, sys

path = os.path.join(sys.argv[1], "research", "scripts", "verus-proof-check.py")
text = open(path, encoding="utf-8").read()
old = '''        z3_override = os.environ.get("VERUS_Z3_PATH", "")
        extra = "".join(f" {name}_sha256={file_sha256(os.path.join(directory, name))}" for name in TOOL_VERSION_FILES)
        extra += f" VERUS_Z3_PATH={z3_override + ':' + file_sha256(z3_override) if z3_override else '未设'}"'''
new = '''        z3_override = os.environ.get("VERUS_Z3_PATH")
        extra = "".join(f" {name}_sha256={file_sha256(os.path.join(directory, name))}" for name in TOOL_VERSION_FILES)
        every = hashlib.sha256()
        if os.path.isdir(directory):
            for name in sorted(os.listdir(directory)):
                full = os.path.join(directory, name)
                if os.path.isfile(full):
                    every.update(name.encode("utf-8", "surrogateescape") + b"\\0" + file_sha256(full).encode() + b"\\n")
        extra += f" install_top_level_sha256={every.hexdigest()}"
        if z3_override is None:
            extra += " VERUS_Z3_PATH=未设"
        elif z3_override == "":
            extra += " VERUS_Z3_PATH=空串"
        else:
            located = z3_override if "/" in z3_override else (shutil.which(z3_override) or "")
            extra += f" VERUS_Z3_PATH={z3_override}:{located or '找不到'}:{file_sha256(located) if located else '没有'}"'''
if text.count(old) != 1:
    sys.exit(f"旧串命中 {text.count(old)} 次")
temporary = path + ".draft"
open(temporary, "w", encoding="utf-8").write(text.replace(old, new))
os.replace(temporary, path)
print("打上 G10")
