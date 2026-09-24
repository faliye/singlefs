"""目录 ↔ 清单表的双向比对（门禁 50、62、63、98 号共用）。

四道阶段判的是同一种形状：盘上有一批东西（规则文件、门禁阶段、agent 定义、kb 文件），
一张清单表登记它们（CLAUDE.md 的 @ 引用、stage-owners.tsv、agent-write-scope.tsv、「项目本地事实」表）。
两个方向都要判：盘上有、清单没登记的，读的人不知道它是什么；清单登记了、盘上没有的，
清单指向空处，让人以为那一格有人管着。只判一个方向的比对，另一个方向永远不红。

各阶段自己决定「盘上有什么」「清单登记了什么」「怎么算登记上了」，这里只管比对与读表：
  two_way(on_disk, registered, is_registered=None, exists=None) -> (unregistered, dangling)
  table_rows(path) -> [(行号, 去掉换行的整行, 按制表符切开的各格)]

阶段里的载入写法（文件名带连字符，不能直接 import）：
  spec = importlib.util.spec_from_file_location("lib_manifest", 路径)
  manifest = importlib.util.module_from_spec(spec); spec.loader.exec_module(manifest)
"""


def two_way(on_disk, registered, is_registered=None, exists=None):
    """返回 (unregistered, dangling)，两份都保持输入的次序（要排序由调用方先排好再传）。

    unregistered：on_disk 里清单没登记的；is_registered(item) 缺省是「item 在 registered 里」。
    dangling：registered 里指向盘上不存在的东西的；exists(entry) 缺省是「entry 在 on_disk 里」。
    两个判定各给各的：盘上那一侧与清单那一侧的集合可以不同
    （例：63 号只要求有 Write / Edit 的定义登记，却要求表里每个名字都有定义）。
    """
    on_disk = list(on_disk)
    registered = list(registered)
    registered_set = set(registered)
    on_disk_set = set(on_disk)
    if is_registered is None:
        is_registered = registered_set.__contains__
    if exists is None:
        exists = on_disk_set.__contains__
    unregistered = [item for item in on_disk if not is_registered(item)]
    dangling = [entry for entry in registered if not exists(entry)]
    return unregistered, dangling


def table_rows(path):
    """制表符分隔的清单表：空行与 # 开头的行是注释，不返回；其余每行给 (行号, 去掉换行的整行, 各格)。"""
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            rows.append((line_number, line, line.split("\t")))
    return rows
