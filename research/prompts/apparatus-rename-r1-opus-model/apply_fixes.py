#!/usr/bin/env python3
"""把攻方腿提的三个改法打进一棵冻结树的副本（只在副本上改，原树不动）。用法：python3 apply_fixes.py <树>
改法一：path_move_problems 的「旧前缀还在磁盘上」只豁免「后面的登记又搬进这个前缀」，去掉「后面某一行的新前缀是旧前缀的上一层」那一支。
改法三：lib_long_run 的 moved_script_unchanged 只在这一行的脚本自己搬了家（基准里取到的是搬迁之前的路径）时认搬迁。
改法四：frozen_o3.RegisteredMoves.content_before_moves 与 reuse_library.pinned_sha256_now 倒换之前，要求此刻的内容正着换一遍不动（不留登记过的旧前缀）。"""
import os
import sys

tree = sys.argv[1]
D = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/"


def edit(relative, old, new):
    path = os.path.join(tree, relative)
    text = open(path, encoding="utf-8").read()
    assert text.count(old) == 1, (relative, text.count(old))
    with open(path + ".tmp", "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))
    os.replace(path + ".tmp", path)
    print(f"name=apply_fix file={relative}")


edit(".claude/gate.d/lib-lag-baseline.py",
     'moved_back = any(following.new == move.old or following.new.startswith(move.old + "/") or move.old.startswith(following.new + "/") for following in later)',
     'moved_back = any(following.new == move.old or following.new.startswith(move.old + "/") for following in later)')
edit(".claude/hooks/lib_long_run.py",
     '    if hashlib.sha256(data).hexdigest() not in base_sha256s:',
     '    if taken == path:\n        return f"{path} 自己没搬家（基准里同一路径就有），不按搬迁认"\n    if hashlib.sha256(data).hexdigest() not in base_sha256s:')
edit(D + "frozen_o3.py",
     '        backwards = [move._replace(old=move.new, new=move.old) for move in reversed(self.moves)]\n        return self.library.rewrite_moved_bytes(data, backwards)',
     '        if self.library.rewrite_moved_bytes(data, list(self.moves)) != data:\n            return data\n        backwards = [move._replace(old=move.new, new=move.old) for move in reversed(self.moves)]\n        return self.library.rewrite_moved_bytes(data, backwards)')
edit(D + "reuse_library.py",
     '        for count in range(1, len(moves.moves) + 1):',
     '        if moves.library.rewrite_moved_bytes(data, list(moves.moves)) != data:\n            return None\n        for count in range(1, len(moves.moves) + 1):')
