# 删除写出哪些字节（样本）

**里程碑「样本」三·6 的登记表**（[03-third-txn.md](../milestone/03-third-txn.md)）。同一个字段的宽度与落点以 [layout/01-first-txn.md](01-first-txn.md) 为准。

## 一、形态

| 形态 | 第一次出现在哪次发布 | 写成什么 | 决策分项 | 里程碑那一步 |
|---|---|---|---|---|
| 意图 | 意图那次发布 | 被删的 inode 记录写成 nlink = 0 | D8（样本） 已定项 1 | 三·2、三·6 |
| 样本树根（码 2） | 意图那次发布 | 头的落点以 [layout/01-first-txn.md](01-first-txn.md)「inode 树根（码 2）」那一行为准 | D8（样本） 已定项 1 | 三·2、三·6 |
| 退役容器 | 有退役记录的那次发布 | 码 3 打包记录单元 | D8（样本） 已定项 1 | 三·5、三·6 |

## 四、段序列

删除类发布都是普通发布的形状（[layout/01-first-txn.md](01-first-txn.md) 段序列登记表「普通发布（新池新建文件）」那一行）。

| 发布 | 单元数（每盘） | 段串（两块盘） | 出处 |
|---|---|---|---|
| 意图 | 13 | `26+2+1+2` | `crates/sample/tests/unlink_sample.rs` 的 `intent_rewrites_the_sample_roots` |
| 最后一批 | 12 | `24+2+1+2` | `crates/sample/src/unlink.rs` 的 `last_batch_drops_the_record` |

## 历史版本

- 样本没有历史；这一节之后的错写不判：「一、」表里写「第 9 批」的行在这里也不扫。
