#!/usr/bin/env python3
"""K1 路径 2、3 用：对一份 lib_function_units.py 做「切单元的逻辑」一处改动，或对 crates 树做一处源码改动。
用法：
  k1_edits.py lfu <名> <lib_function_units.py 路径>   原地换成改动后的一份（先写同目录临时文件再 rename，不在原 inode 上改）
  k1_edits.py tree <crates 树根> <文件> <fn 名>        在 <文件> 里 <fn 名> 的定义上面加一行 #[inline(never)]
每处改动的旧串必须在文件里恰好出现一次，不然退 2。
"""
import os
import sys

EDITS = {
    # 只动单元的起始行（首个属性所在行 → 属性之后的首个词所在行）；指纹取的词法单元不变
    "E1-first-line-after-attributes": (
        'unit = Unit("rust", kind, "::".join(context + [name]), self.path, self.lines[start], self.lines[min(end, self.count) - 1],',
        'unit = Unit("rust", kind, "::".join(context + [name]), self.path, self.lines[max(self.current_header, start)], self.lines[min(end, self.count) - 1],'),
    # 单元限定名里「模块名与函数名」的连接符 :: → .
    "E2-qualified-name-separator": (
        'unit = Unit("rust", kind, "::".join(context + [name]), self.path, self.lines[start], self.lines[min(end, self.count) - 1],',
        'unit = Unit("rust", kind, ".".join(context + [name]), self.path, self.lines[start], self.lines[min(end, self.count) - 1],'),
    # 等价重写：scan 里「下一项从哪里起」换一种写法，输出不变
    "E3-equivalent-rewrite": (
        "            end = self.item(start, keyword, high, context, module_path, container, item_conditions, public, attributes, verus)\n            i = max(end, i + 1)\n",
        "            end = self.item(start, keyword, high, context, module_path, container, item_conditions, public, attributes, verus)\n            i = end if end > i else i + 1\n"),
    # 指纹不再连前面的外属性（指纹取的词法单元从属性之后起）
    "E4-fingerprint-after-attributes": (
        '        pieces = self.texts[start:end]\n',
        '        pieces = self.texts[max(self.current_header, start):end]\n'),
    # 指纹不再连 #[inline…] 这种外属性（当它不改语义）；别的属性照连
    "E5-fingerprint-ignores-inline-attribute": (
        '        pieces = self.texts[start:end]\n',
        '        pieces = self.texts[start:end]\n'
        '        cleaned, skip_until = [], -1\n'
        '        for offset, piece in enumerate(pieces):\n'
        '            index = start + offset\n'
        '            if index <= skip_until:\n'
        '                continue\n'
        '            if piece == "#" and self.t(index + 1) == "[" and self.t(index + 2) == "inline" and (index + 1) in self.pairs:\n'
        '                skip_until = self.pairs[index + 1]\n'
        '                continue\n'
        '            cleaned.append(piece)\n'
        '        pieces = cleaned\n'),
}


def replace_once(path, old, new):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    if text.count(old) != 1:
        print(f"旧串在 {path} 里出现 {text.count(old)} 次，要恰好 1 次：{old[:60]!r}", file=sys.stderr)
        sys.exit(2)
    temporary = path + ".partial"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new, 1))
    os.chmod(temporary, os.stat(path).st_mode)
    os.replace(temporary, path)


def main(arguments):
    if len(arguments) == 3 and arguments[0] == "lfu" and arguments[1] in EDITS:
        replace_once(arguments[2], *EDITS[arguments[1]])
        print(f"edit={arguments[1]} file={arguments[2]}")
        return 0
    if len(arguments) == 4 and arguments[0] == "tree":
        root, relative, name = arguments[1:]
        path = os.path.join(root, relative)
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        hits = [index for index, line in enumerate(lines) if line.lstrip().startswith(("fn ", "pub fn ", "pub(crate) fn ")) and f"fn {name}(" in line]
        if len(hits) != 1 or lines[hits[0] - 1].lstrip().startswith("#["):
            print(f"{relative} 里 fn {name} 对上 {len(hits)} 处，或它上面已有属性", file=sys.stderr)
            return 2
        indent = lines[hits[0]][:len(lines[hits[0]]) - len(lines[hits[0]].lstrip())]
        lines.insert(hits[0], indent + "#[inline(never)]")
        temporary = path + ".partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write("\n".join(lines))
        os.replace(temporary, path)
        print(f"edit=add-attribute file={relative} fn={name} line={hits[0] + 1}")
        return 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
