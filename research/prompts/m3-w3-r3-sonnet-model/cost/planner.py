#!/usr/bin/env python3
"""记账树规划器的 Python 复刻（对着冻结树 crates/singlefs-core/src/code_two_tree.rs 的 plan_the_tree_after_this_publish 与它下面的
insert_below / delete_below / split_in_the_middle / insert_at_the_root / delete_at_the_root 逐步写）。
只算形状与「这次重写哪些节点」，不装字节。用 `probe_compare.py` 对着真规划器的探针输出（W3COST 行）逐行对账，容量压小的几档与产品容量都对。"""
import bisect

class Node:
    __slots__ = ("level", "keys", "children")   # 叶：keys；内部：children = [[分隔 key, 子节点], …]
    def __init__(self, level, keys=None, children=None):
        self.level = level; self.keys = keys; self.children = children
    @property
    def leaf(self): return self.keys is not None
    def count(self): return len(self.keys) if self.leaf else len(self.children)
    def smallest(self): return self.keys[0] if self.leaf else self.children[0][1].smallest()

class Tree:
    def __init__(self, leaf_cap, internal_cap):
        self.leaf_cap = leaf_cap; self.internal_cap = internal_cap
        self.root = None; self.touched = None; self.new_root_created = False

    def _touch(self, n): self.touched.add(id(n))

    def _split(self, n):
        if n.leaf:
            half = (len(n.keys) + 1) // 2
            right = Node(0, keys=n.keys[half:]); n.keys = n.keys[:half]
            sep = right.keys[0]
        else:
            half = (len(n.children) + 1) // 2
            right = Node(n.level, children=n.children[half:]); n.children = n.children[:half]
            sep = right.children[0][0]
        self._touch(right)
        return [sep, right]

    def _insert_below(self, n, key):
        self._touch(n)
        if n.leaf:
            i = bisect.bisect_left(n.keys, key)
            assert i == len(n.keys) or n.keys[i] != key
            n.keys.insert(i, key)
            return self._split(n) if len(n.keys) > self.leaf_cap else None
        idx = None
        for j in range(len(n.children) - 1, -1, -1):
            if n.children[j][0] <= key: idx = j; break
        if idx is None:
            n.children[0][0] = key; idx = 0
        right = self._insert_below(n.children[idx][1], key)
        if right is not None: n.children.insert(idx + 1, right)
        return self._split(n) if len(n.children) > self.internal_cap else None

    def insert(self, key):
        if self.root is None:
            self.root = Node(0, keys=[]); self._touch(self.root)
        right = self._insert_below(self.root, key)
        if right is None: return
        left = self.root
        new_root = Node(left.level + 1, children=[[left.smallest(), left], right]); self._touch(new_root)
        self.root = new_root

    def _delete_below(self, n, key):
        if n.leaf:
            i = bisect.bisect_left(n.keys, key)
            if i < len(n.keys) and n.keys[i] == key:
                del n.keys[i]; self._touch(n); return True
            return False
        idx = 0
        for j in range(len(n.children) - 1, -1, -1):
            if n.children[j][0] <= key: idx = j; break
        if not self._delete_below(n.children[idx][1], key): return False
        if n.children[idx][1].count() == 0: del n.children[idx]
        self._touch(n)
        return True

    def delete(self, key):
        assert self._delete_below(self.root, key), "删的 key 不在树里"
        while True:
            r = self.root
            if not r.leaf and len(r.children) == 0:
                self.root = Node(0, keys=[]); self._touch(self.root); return
            if not r.leaf and len(r.children) == 1:
                self.root = r.children[0][1]; continue
            return

    def stats(self):
        nodes = leaves = 0; stack = [self.root] if self.root else []
        while stack:
            n = stack.pop(); nodes += 1
            if n.leaf: leaves += 1
            else: stack.extend(c[1] for c in n.children)
        height = (self.root.level + 1) if self.root else 0
        return nodes, leaves, height

    def publish(self, deleted_sorted, inserted_sorted):
        """先删后插（都按 key 升序），交回这次重写的节点数。"""
        self.touched = set()
        if self.root is None and not inserted_sorted: return 0
        for k in deleted_sorted: self.delete(k)
        for k in inserted_sorted: self.insert(k)
        # 被摘掉的、没留在树里的节点不算：只数还在树里的重写节点
        alive = set(); stack = [self.root]
        while stack:
            n = stack.pop(); alive.add(id(n))
            if not n.leaf: stack.extend(c[1] for c in n.children)
        return len(self.touched & alive)
