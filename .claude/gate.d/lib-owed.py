#!/usr/bin/env python3
"""checks-owed.md 里哪些欠账开着、哪些已还清：门禁 67、92、96 号共用这一份，不各抄一份。

几道各写一份时切法分叉过：有的按子串「### 已还清」切，开着的那张表里有一行正文提到这几个字，
它之后开着的账就全被算成没开；认不出标题时有的判红，有的把整份文件——连历史版本节里的表格行——
都算成开着的账；还有的只切了一刀，把历史版本节里的表格行算进已还清。

分界：
- 开着 = 「已还清」那一行标题（整行，`^#+\\s*已还清\\s*$`）之前、表格首列是 C<编号> 的行；
- 已还清 = 那一行标题之后、下一个「## 历史版本」之前、首列是 C<编号> 的行；
- 「## 历史版本」之后的表格行两边都不算：那里的表是记事，不是登记。
认不出「已还清」标题时 paid_heading_found 为假，open_names 是全文的表格行——调用方要先看这个标记，
认不出就判红，不对着一张认不出的表判。
"""
import os
import re
from dataclasses import dataclass

PAID_HEADING = re.compile(r"^#+\s*已还清\s*$")
HISTORY_HEADING = re.compile(r"^##\s*历史版本\s*$")
OWED_ROW = re.compile(r"^\|\s*(C[0-9]+)\s*\|\s*([^|]*?)\s*\|")


@dataclass(frozen=True)
class OwedTable:
    file_found: bool
    paid_heading_found: bool
    open_names: dict
    paid_names: dict


def read_owed_table(path):
    """编号 → 简称（第二列），开着与已还清各一张；文件不在时两张都空、file_found 为假。"""
    if not os.path.isfile(path):
        return OwedTable(False, False, {}, {})
    open_names, paid_names = {}, {}
    section = "open"
    with open(path, encoding="utf-8") as owed_file:
        for line in owed_file:
            if section == "open" and PAID_HEADING.match(line):
                section = "paid"
                continue
            if section == "paid" and HISTORY_HEADING.match(line):
                section = "history"
                continue
            row = OWED_ROW.match(line)
            if not row:
                continue
            if section == "open":
                open_names[row.group(1)] = row.group(2)
            elif section == "paid":
                paid_names[row.group(1)] = row.group(2)
    return OwedTable(True, section != "open", open_names, paid_names)
