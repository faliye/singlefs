# m3-retire-r1 正推腿的小模型

纯 Python 小模型（`model.py`，不读 `crates/`、不依赖 `SINGLEFS_REPO`，只用标准库）。它不是对实现的核对，是正推腿对「退役记录怎么写、扫描重建怎么读、checker 判什么」几个候选形态的独立自证；
`implementation-first.md` 第 4、5 条：独立模型只用来验证，数不进 kb，要引先在入库装置上重做。

- 世界：单头；叶容器容量 4（`CAP`），退役容器容量 3（`RCAP`）；操作 = 建 inode、删一个 inode（合并政策 drop / absorb / borrow）、管理员回退、崩溃（老实例留下发布了单元没发布根、新实例 txg 重发或跳号）。
- 写法：退役集合 = 上一版叶身份集 - 这一版叶身份集（集合差），死亡 txg = 这次发布的 txg。
- 读者五种（`reader`）：D 不看退役树（今天的 `crates/`）；C 退役记录不看谱系；A 候选只过已发布谓词、记录按死亡 txg 滤谱系；B = A 加候选单元的诞生 txg 也要在父链上；E = B 但退役记录不先按已发布谓词滤版本、各版记录并起来。
- checker 候选判据（`check_invariants`）：a 记录合法、b 谱系上不重身份、c 谱系上的退役记录不指本版活叶、d 相邻版本之间消失的叶身份与死亡 txg = 本版 txg 的谱系上记录一一对应（d_missing 漏记、d_extra 多记）。
- 植入的写者错误五种：忘了写删空的、忘了写被吸收的、借时给右片写了退役记录、死亡 txg 少一、同一条写两次。
- 备选 α：回退那次发布给现行独有的叶身份写退役记录。

`rerun.sh`：`SINGLEFS_REPO=<仓根> bash rerun.sh <空目录> all`。`expected/summary.txt` 是第一次跑的产物，`compare` 排序后逐行比。
