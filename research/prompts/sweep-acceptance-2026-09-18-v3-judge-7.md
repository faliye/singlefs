# sweep-acceptance-v3-judge-7：逐行判（组 D13, D14, F1-F7）

阶段：sweep-acceptance-v3-judge-7。基准 b1c8cef~1，结束提交 00c9d4f。候选表：
`research/prompts/sweep-acceptance-2026-09-18-v2-candidates.tsv`；事实表同目录 `-v2-facts.tsv`。
候选表分给本轮的组：D13（138 行）、D14（1 行）、F1（17 行）、F2（5 行）、F3（5 行）、F4（8 行）、
F5（43 行）、F6（9 行）、F7（14 行），合计 240 行，全部逐行判过，不抽样。

判据照 `.claude/agents/sweep.md` 第 9 步：把这一行里描述现状的分句换成新事实，在 00c9d4f 之后还是不是真话；
带日期的分句里，日期后面说的是当天发生的事才算事件句，说的是「到那天为止」的现状（如「2026-09-14 已有 X；
层 0 只有第一个事务」的后半句）当现状句判。上下文一律 `git show 00c9d4f:路径` 读，不读工作区。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| D13 | .claude/kb/checks-owed.md:150 | 事件句不改 | C148 立账原文是 67→121 那次历史定案的记录，检查形态列已按现行条目宽 200 写明重算要求 |
| D13 | .claude/kb/checks-owed.md:159 | 要人看 | C157「kb 这一半已收口：统一按 145 字节条目算成 112 棵/层」用的是 148 之前的中间值，而现行值已是 200/81，需要人核这条「已收口」的结论是否也要随之改写 |
| D13 | .claude/kb/checks-owed.md:164 | 不相干 | C162 说的是记账条目（30→34 字节），不是树表条目 |
| D13 | .claude/kb/checks-owed.md:178 | 不相干 | C180 说的是设备内禀身份字段宽度，不是树表条目 |
| D13 | .claude/kb/checks-owed.md:187 | 不相干 | C202 里的「条目宽」指中央映射条目，不是树表条目 |
| D13 | .claude/kb/checks-owed.md:231 | 不相干 | C249 说的是 DRAM 档点查模型的字段宽度，不是树表条目 |
| D13 | .claude/kb/checks-owed.md:243 | 不相干 | C261 说的是分配记录条目宽（30 与 20 之争），不是树表条目 |
| D13 | .claude/kb/checks-owed.md:248 | 不相干 | C266 说的是 livelist 条目宽，不是树表条目 |
| D13 | .claude/kb/checks-owed.md:285 | 不相干 | C306 说的是码 2 头对 key 宽/条目宽的一般自描述机制，不特指树表 |
| D13 | .claude/kb/checks-owed.md:286 | 不相干 | C307 说的是映射树两种 key 宽怎么装进定宽树，不是树表条目 |
| D13 | .claude/kb/checks-owed.md:374 | 不相干 | C321 说的是码 2 头条目宽字段与映射树条目按类两宽的冲突，不是树表条目 |
| D13 | .claude/kb/decisions.md:168 | 不相干 | decisions.md:168 是索引条目「已定」目录项标题，泛指码 2 节点条目宽的表，不特指树表数值 |
| D13 | .claude/kb/decisions/03-空间分配.md:179 | 不相干 | decisions/03:179 说的是分配记录条目（18→20 字节），不是树表条目 |
| D13 | .claude/kb/decisions/03-空间分配.md:198 | 不相干 | decisions/03:198 的 ppm/条目宽线性关系数据是分配记录相关表，不是树表条目 |
| D13 | .claude/kb/decisions/03-空间分配.md:426 | 不相干 | decisions/03:426 说的是 t5 分配记录条目（20 字节），不是树表条目 |
| D13 | .claude/kb/decisions/05-快照-空间记账机制.md:517 | 事件句不改 | decisions/05:517「树表单元 t8…条目宽 2026-09-16 起 200」已是现行值，且带日期是那次定案的记录 |
| D13 | .claude/kb/decisions/06-快照实现模型.md:235 | 不相干 | decisions/06:235 说的是 livelist 载体扇出，条目宽指 livelist 条目，不是树表条目 |
| D13 | .claude/kb/decisions/06-快照实现模型.md:251 | 要人看 | decisions/06:251「第一个事务多 148 字节（2026-09-14 起的树表条目宽）」是把已作废的 148 当成当前树表条目宽在算代价，现行值已是 200，这处派生代价需要人重算 |
| D13 | .claude/kb/decisions/06-快照实现模型.md:255 | 不相干 | decisions/06:255 说的是 livelist 条目里 14 字节位置提示，不是树表条目 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:14 | 不相干 | decisions/08:14 是「定宽条目数组…声明长度=条目数×条目宽」的通用原理陈述，不特指树表数值 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:79 | 不相干 | decisions/08:79 是码 2 节点通用载荷布局定案（已定项 11），条目宽是该节点类型的通用字段，不特指树表当前字节数 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:354 | 不相干 | decisions/08:354 说的是索引节点内部条目宽（93→117→120），不是树表条目 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:459 | 不相干 | decisions/08:459 的历史叙述里提到的 83/86 字节是根指针宽度、117/105 是 inode/记账节点条目宽，都不是树表条目总宽 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:469 | 事件句不改 | decisions/08:469「树表条目的最终布局（合计 200 字节…）」已是现行值，且带日期记的是 2026-09-14/16 两次定案 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:475 | 事件句不改 | decisions/08:475「条目宽 200 算…81 棵…都是用户定案里的数」已是现行值，且逐段列出了 121/134→145/112→148/109→200/81 的历史演进 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:483 | 不相干 | decisions/08:483 比较的是「树表条目自带长度」方案的 18 字节代价 vs 索引节点每条目自带长度的扇出代价，例证数字（inode 树 175→171 等）是别的树，不是树表当前总宽 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:511 | 不相干 | decisions/08:511 是码 2 节点头自描述的通用 schema 定案，不特指树表 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:512 | 不相干 | decisions/08:512 同上，通用 schema 原理 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:519 | 不相干 | decisions/08:519 同上，通用解码规则 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:523 | 不相干 | decisions/08:523 讨论的是 C321（映射树条目按类两宽）与码 2 单值条目宽字段的冲突，不是树表条目总宽 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:524 | 不相干 | decisions/08:524 说的是映射条目定宽为 55，不是树表条目 |
| D13 | .claude/kb/decisions/12-目标介质.md:155 | 不相干 | decisions/12:155 说的是设备描述符表条目宽度，不是树表条目 |
| D13 | .claude/kb/decisions/12-目标介质.md:158 | 不相干 | decisions/12:158 同上 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:152 | 不相干 | decisions/18:152 是码 2 声明长度语义的通用定案，不特指树表当前字节数 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:617 | 不相干 | decisions/18:617 是码 2 节点头字段清单的通用定案，不特指树表 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:625 | 不相干 | decisions/18:625 是码 2 完整偏移表定案，不特指树表 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:796 | 不相干 | decisions/18:796 是索引节点通用字段表，条目宽随树而变，不特指树表当前值 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:938 | 不相干 | decisions/18:938 是已定项 18 标题，通用码 2 声明长度定案 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:940 | 不相干 | decisions/18:940 同上，通用定案正文 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:945 | 不相干 | decisions/18:945 是码 2 通用偏移字段表，不特指树表 |
| D13 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:285 | 不相干 | decisions/19:285 是「条目宽」在 4KiB/16KiB 节点两档下的对照表表头，通用比较，不特指树表 |
| D13 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:425 | 不相干 | decisions/19:425 讨论的是码 2 条目宽字段单值与按类变长的冲突，通用原理，不特指树表当前值 |
| D13 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:429 | 不相干 | decisions/19:429 说的是中央映射树条目（55 字节）与 livelist/稀疏旁表条目宽度，不是树表条目 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:368 | 不相干 | decisions/22:368「512 字节槽带 jsn 水位只装得下 6 棵树」是另一个结构（记录头水位槽），条目宽是它自己的变量，不是树表条目 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:513 | 事件句不改 | decisions/22:513「条目宽取…200 字节」已是现行值 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:514 | 事件句不改 | decisions/22:514「⌊(16384−131)/200⌋=81」已是现行值计算 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:515 | 事件句不改 | decisions/22:515 逐日期列出 2026-09-14/16 两次改动（145→148→200，112→109→81），已是完整历史记录且落在现行值 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:516 | 事件句不改 | decisions/22:516 记的是 2026-09-13 那次 243→112 的定案与仍欠的 E79 重跑，属历史记录，已注明「仍欠」 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:659 | 不相干 | decisions/22:659 的历史讨论是超级块字段范围之争，逐字写明「树表不在其中」，与树表条目宽无关 |
| D13 | .claude/kb/experiments.md:118 | 不相干 | experiments.md:118 说的是超级块设备表条目宽度，不是树表条目 |
| D13 | .claude/kb/experiments.md:150 | 要人看 | experiments.md:150（E132）报的「44 头/66 头」等livelist数字基于旧的树表第二层门槛假设算出；milestone/02-second-txn.md:344 已自行标注这批数按 200 重跑的结果尚未同步进决策正文，需要人拿到重跑结果后一并更新 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:1 | 不相干 | experiments/100 全部 6 处（1/18/25/42/44/74）说的是超级块设备表条目宽度候选（24/40/64），不是树表条目 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:18 | 不相干 | experiments/100:18 同上 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:25 | 不相干 | experiments/100:25 同上 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:42 | 不相干 | experiments/100:42 同上 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:44 | 不相干 | experiments/100:44 同上 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:74 | 不相干 | experiments/100:74 同上 |
| D13 | .claude/kb/experiments/101-节点头留位的代价与表达力.md:22 | 不相干 | experiments/101 全部 3 处（22/24/51）说的是「三棵已定条目宽的树」（inode/extent/记账），不是树表条目 |
| D13 | .claude/kb/experiments/101-节点头留位的代价与表达力.md:24 | 不相干 | experiments/101:24 同上 |
| D13 | .claude/kb/experiments/101-节点头留位的代价与表达力.md:51 | 不相干 | experiments/101:51 同上 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:27 | 不相干 | experiments/117 全部 7 处说的是 extent 树内部条目宽（54/81 等点查代价实验参数），不是树表条目 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:32 | 不相干 | experiments/117:32 同上 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:38 | 不相干 | experiments/117:38 同上 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:39 | 不相干 | experiments/117:39 同上 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:58 | 不相干 | experiments/117:58 同上，其中 200/201 是扇出计数产物值，与树表条目宽 200 字节纯属数字巧合 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:59 | 不相干 | experiments/117:59 同上 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:61 | 不相干 | experiments/117:61 同上 |
| D13 | .claude/kb/experiments/128-指针带出生树txg之后的点查代价.md:36 | 不相干 | experiments/128:36 是实验里各臂条目宽对照表表头，说的是指针实验的候选宽度，不是树表条目 |
| D13 | .claude/kb/experiments/128-指针带出生树txg之后的点查代价.md:120 | 事件句不改 | experiments/128:120 是该实验明说测不了树表每层棵数、改报条目宽 121→137 这段历史注记，已自带 2026-09-13 的说明并指向 C157，属冻结的实验记录 |
| D13 | .claude/kb/experiments/128-指针带出生树txg之后的点查代价.md:145 | 不相干 | experiments/128:145 是通用装置方法论描述，非特指树表 |
| D13 | .claude/kb/experiments/130-每头一份livelist过不过有界销毁.md:104 | 不相干 | experiments/130:104 说的是记账条目宽（30 字节），不是树表条目 |
| D13 | .claude/kb/experiments/131-livelist的两条载体数值与代价.md:92 | 不相干 | experiments/131:92 说的是 livelist 内部扇出用的条目宽，不是树表条目 |
| D13 | .claude/kb/experiments/132-livelist载体按真实树数与内部扇出重算.md:37 | 事件句不改 | experiments/132:37「树表每层 81 棵…条目宽是…2026-09-16 用户定案加宽后的 200」已是现行值 |
| D13 | .claude/kb/experiments/132-livelist载体按真实树数与内部扇出重算.md:38 | 不相干 | experiments/132:38 说的是 livelist 条目宽扫描候选（24–56），不是树表条目 |
| D13 | .claude/kb/experiments/132-livelist载体按真实树数与内部扇出重算.md:59 | 不相干 | experiments/132:59 同上 |
| D13 | .claude/kb/experiments/133-映射key各候选的格式代价.md:157 | 不相干 | experiments/133:157 说的是映射内部扇出错用叶条目宽的 bug，不是树表条目 |
| D13 | .claude/kb/experiments/136-映射key岔路的代价行.md:39 | 不相干 | experiments/136:39 说的是六种索引节点条目宽的码 2 头 +4 测试，不是树表条目 |
| D13 | .claude/kb/experiments/136-映射key岔路的代价行.md:107 | 不相干 | experiments/136:107 同上 |
| D13 | .claude/kb/experiments/137-映射key形态的性能差距.md:3 | 不相干 | experiments/137:3 是映射 key 代价行的引用性提问，未提具体宽度数值，非树表条目声明 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:8 | 不相干 | experiments/142:8 说的是码 2 头条目宽字段（G20/C321），不是树表条目 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:9 | 不相干 | experiments/142:9 同上 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:49 | 不相干 | experiments/142:49 是变异测试 M62 对 parse_index_node 结构检查的说明，通用码 2 解析逻辑，不特指树表 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:172 | 不相干 | experiments/142:172 说的是码 2 头自描述定案（G13），通用 schema，不特指树表 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:174 | 不相干 | experiments/142:174 同上（G15） |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:179 | 不相干 | experiments/142:179 说的是中央映射条目定宽 55（G20），不是树表条目 |
| D13 | .claude/kb/experiments/145-码2自描述头与映射树key宽的代价.md:33 | 不相干 | experiments/145:33 是码 2 自描述头方案 B 的通用说明，不特指树表 |
| D13 | .claude/kb/experiments/145-码2自描述头与映射树key宽的代价.md:65 | 不相干 | experiments/145:65 是五棵树扇出对照表（不含树表），不特指树表条目宽 |
| D13 | .claude/kb/experiments/145-码2自描述头与映射树key宽的代价.md:73 | 不相干 | experiments/145:73 同上 |
| D13 | .claude/kb/experiments/146-livelist条目按映射key定身份之后的宽度与代价.md:43 | 不相干 | experiments/146:43 说的是 livelist 每次 FREE 的叶条目宽预付，不是树表条目 |
| D13 | .claude/kb/experiments/146-livelist条目按映射key定身份之后的宽度与代价.md:44 | 事件句不改 | experiments/146:44「树表多一条 200 字节…每单元 81 条」已是现行值，且带 C157 口径引用 |
| D13 | .claude/kb/experiments/20-指针变宽的CPU缓存代价.md:56 | 不相干 | experiments/20:56 是指针宽度 CPU 缓存实验的条目宽对照表表头（16/40/111），不是树表条目 |
| D13 | .claude/kb/experiments/20-指针变宽的CPU缓存代价.md:89 | 不相干 | experiments/20:89 同上 |
| D13 | .claude/kb/experiments/20-指针变宽的CPU缓存代价.md:119 | 不相干 | experiments/20:119 同上 |
| D13 | .claude/kb/experiments/43-扩展点字节上限.md:35 | 不相干 | experiments/43:35 说的是扩展点位置条目宽度，不是树表条目 |
| D13 | .claude/kb/experiments/72-设备描述符表的挂载复算.md:9 | 不相干 | experiments/72:9 说的是设备描述符表条目宽度，不是树表条目 |
| D13 | .claude/kb/experiments/72-设备描述符表的挂载复算.md:54 | 不相干 | experiments/72:54 同上 |
| D13 | .claude/kb/experiments/72-设备描述符表的挂载复算.md:55 | 不相干 | experiments/72:55 同上 |
| D13 | .claude/kb/experiments/72-设备描述符表的挂载复算.md:82 | 不相干 | experiments/72:82 同上 |
| D13 | .claude/kb/experiments/73-节点key区间的扇出代价.md:8 | 不相干 | experiments/73:8 说的是 extent 节点 key 区间扇出实验的条目宽变量，不是树表条目 |
| D13 | .claude/kb/experiments/73-节点key区间的扇出代价.md:99 | 不相干 | experiments/73:99 同上 |
| D13 | .claude/kb/experiments/74-分配记录的条目数与整理开销.md:11 | 不相干 | experiments/74:11 说的是分配记录整理开销公式里的条目宽度，不是树表条目 |
| D13 | .claude/kb/experiments/79-根记录的容量.md:14 | 不相干 | experiments/79:14 说的是根记录里位置条目宽度（14 字节），不是树表条目 |
| D13 | .claude/kb/experiments/79-根记录的容量.md:26 | 不相干 | experiments/79:26「512 槽带水位装 6 棵」是另一种平铺结构的条目宽变量，不是树表条目 |
| D13 | .claude/kb/experiments/79-根记录的容量.md:33 | 事件句不改 | experiments/79:33 与其后 2026-09-13 的作废批注同段，是对 67 字节旧口径的历史讨论，已被同一段落自行标注作废 |
| D13 | .claude/kb/experiments/79-根记录的容量.md:36 | 事件句不改 | experiments/79:36「这组 243…已作废，加一档重跑欠在 C157」是自行标注的历史注记，已指向未还的检查 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:35 | 不相干 | experiments/97:35 说的是分配/记账记录条目宽公式，不是树表条目 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:42 | 不相干 | experiments/97:42 同上，通用单调性原理 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:68 | 不相干 | experiments/97:68 同上，通用阳性对照原理 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:102 | 不相干 | experiments/97:102 同上，fanout() 公式说明 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:254 | 不相干 | experiments/97:254 同上，COW 放大倍数分母说明 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:255 | 不相干 | experiments/97:255 同上，打散搬迁节点数与条目宽无关的实测 |
| D13 | .claude/kb/layout/01-first-txn.md:124 | 不相干 | layout/01-first-txn.md:124 说的是位置条目宽度（4/14 字节），不是树表条目 |
| D13 | .claude/kb/layout/01-first-txn.md:245 | 不相干 | layout/01-first-txn.md:245 说的是 inode 树根节点条目宽字段（通用 schema），条目宽 2 字节是字段本身，不是树表条目宽的数值声明 |
| D13 | .claude/kb/layout/01-first-txn.md:271 | 不相干 | layout/01-first-txn.md:271 说的是 extent 树节点条目宽（112 字节），不是树表条目 |
| D13 | .claude/kb/layout/01-first-txn.md:286 | 不相干 | layout/01-first-txn.md:286 说的是分配记录树节点条目宽（20 字节），不是树表条目 |
| D13 | .claude/kb/layout/01-first-txn.md:289 | 不相干 | layout/01-first-txn.md:289 说的是记账树节点条目宽（34 字节），不是树表条目 |
| D13 | .claude/kb/layout/01-first-txn.md:368 | 事件句不改 | layout/01-first-txn.md:368「树表单元自己的头…条目宽 200、声明长度 1400」已是现行值 |
| D13 | .claude/kb/milestone/01-first-txn.md:103 | 事件句不改 | milestone/01-first-txn.md:103 现状段落「树表条目 200（树 ID 打头、头 ID、预留 76 非零拒收）」已是现行值，正是 D13 新事实要求落地的那一句 |
| D13 | .claude/kb/milestone/01-first-txn.md:110 | 不相干 | milestone/01-first-txn.md:110 说的是树表单元自己的码 2 头宽度（131 字节），是头部宽度不是条目总宽 |
| D13 | .claude/kb/milestone/01-first-txn.md:130 | 不相干 | milestone/01-first-txn.md:130 只说树表第 6/7 条注册槽位，未涉及条目字节宽度 |
| D13 | .claude/kb/milestone/01-first-txn.md:155 | 不相干 | milestone/01-first-txn.md:155 说的是映射条目相关定案，不是树表条目 |
| D13 | .claude/kb/milestone/02-second-txn.md:48 | 事件句不改 | milestone/02-second-txn.md:48 现状（2026-09-16）段落已按现行 200/76/81 写 |
| D13 | .claude/kb/milestone/02-second-txn.md:51 | 事件句不改 | milestone/02-second-txn.md:51 变异表验证描述，事件句 |
| D13 | .claude/kb/milestone/02-second-txn.md:52 | 事件句不改 | milestone/02-second-txn.md:52「按 200/81/1400 过」已是现行值 |
| D13 | .claude/kb/milestone/02-second-txn.md:60 | 事件句不改 | milestone/02-second-txn.md:60「2026-09-16 复核…常量 200 在三份装置里都是 200」已是现行值，事件句 |
| D13 | .claude/kb/milestone/02-second-txn.md:344 | 事件句不改 | milestone/02-second-txn.md:344 自行标注 D6 已定项 3 的「66/44 头」基于旧 121 宽算、200 宽重跑结果尚未同步，是准确的现存待办记录，不需要改这一行本身 |
| D13 | .claude/kb/milestone/02-second-txn.md:464 | 事件句不改 | milestone/02-second-txn.md:464「按树表条目宽 200 字节读…200 字节的元数据」已是现行值，2026-09-17 用户指示的记录 |
| D13 | .claude/kb/verification-build.md:41 | 不相干 | verification-build.md:41 说的是分配记录条目宽度与 key 编码定案标题，不是树表条目 |
| D13 | .claude/kb/verification-build.md:218 | 不相干 | verification-build.md:218 说的是分配记录相关的重复定案矛盾，不是树表条目 |
| D13 | records/2026-08-29-组合对攻轮.md:222 | 事件句不改 | records/2026-08-29-组合对攻轮.md:222 是带日期的对攻轮记录，说的是明文映射层条目宽度没进成本表，不是树表条目，且按惯例不回改 |
| D13 | records/2026-09-03-验证三件套落地调研.md:34 | 事件句不改 | records/2026-09-03-验证三件套落地调研.md:34 是带日期调研记录，讨论 D3 已定项重名问题，与树表条目宽无关，按惯例不回改 |
| D13 | records/2026-09-10-D19项7第一轮.md:14 | 事件句不改 | records/2026-09-10-D19项7第一轮.md:14 是带日期记录，讨论 E20 六档位与本工程真实条目宽（inode/extent 相关）不符，不是树表条目，按惯例不回改 |
| D13 | records/2026-09-10-D19项7第一轮.md:65 | 事件句不改 | records/2026-09-10-D19项7第一轮.md:65 是带日期记录，E128 处置说明（121/137），已冻结、按惯例不回改 |
| D13 | records/2026-09-13-总审核.md:80 | 事件句不改 | records/2026-09-13-总审核.md:80 是带日期总审核记录，讨论码 2 通用载荷布局待定项，非树表条目当前值声明，按惯例不回改 |
| D13 | records/2026-09-13-总审核.md:270 | 事件句不改 | records/2026-09-13-总审核.md:270 同上，码 2 载荷布局定案摘要，按惯例不回改 |
| D13 | records/2026-09-13-总审核.md:284 | 事件句不改 | records/2026-09-13-总审核.md:284 同上，码 2 声明长度定案摘要，按惯例不回改 |
| D13 | records/2026-09-13-总审核.md:335 | 事件句不改 | records/2026-09-13-总审核.md:335 同上，十二条审核腿方法论描述，「条目宽」出自「映射条目两宽 vs 码 2 单值条目宽」矛盾项，非树表条目，按惯例不回改 |
| D13 | records/2026-09-13-总审核.md:345 | 事件句不改 | records/2026-09-13-总审核.md:345 同上，Q1 映射条目宽定案摘要，非树表条目，按惯例不回改 |
| D14 | .claude/kb/milestone/01-first-txn.md:43 | 不相干 | 载体是 make_filesystem 现状描述，命中检索词 make_filesystem 是巧合，说的不是「只改路径」这件事 |
| F1 | .claude/kb/checks-owed.md:146 | 要改 | 同 F6 第 1 行 C143：多次挂载的录制流已落地，前置列要改 |
| F1 | .claude/kb/checks-owed.md:292 | 要改 | 同 F6 第 2 行 C314 |
| F1 | .claude/kb/checks-owed.md:301 | 要改 | C330 前置列「条款已写…检查仍欠：多次挂载的崩溃点重放」按字面仍是未落地。改后：「条款已写（同前）；检查仍欠：这条检查自己的故障注入（多次挂载的崩溃点重放已落地：mount_writable、层 0 固定脚本到 D）」 |
| F1 | .claude/kb/checks-owed.md:302 | 要改 | 同 F6 第 4 行 C331 |
| F1 | .claude/kb/checks-owed.md:303 | 要改 | 同 F6 第 5 行 C332 |
| F1 | .claude/kb/checks-owed.md:304 | 要改 | 同 F6 第 6 行 C333 |
| F1 | .claude/kb/checks-owed.md:305 | 要改 | 同 F6 第 7 行 C334 |
| F1 | .claude/kb/checks-owed.md:306 | 要改 | 同 F6 第 8 行 C335 |
| F1 | .claude/kb/checks-owed.md:310 | 不相干 | 同 F6 第 9 行 C339：命中的是检查形态里的方法论描述 |
| F1 | .claude/kb/checks-owed.md:313 | 不相干 | C342「检查形态」列是「多次挂载录制流：回退或挂载之后建的每棵树的 ID…」的方法论描述，不是不存在的断言；前置列无此提法 |
| F1 | .claude/kb/checks-owed.md:341 | 不相干 | C366「检查形态」列是「多次挂载的固定脚本里…」的方法论描述；前置列写「里程碑步 3」已落地，不是待补条件 |
| F1 | .claude/kb/decisions/16-发布语义.md:207 | 要改 | decisions/16-发布语义.md:207「检查那一半仍欠（崩溃点重放 harness 的多次挂载录制流）」是 F1 新事实（连同 F6/F7 落地）打假的原句。改后：「检查那一半仍欠这条检查自己的故障注入（崩溃点重放 harness 的多次挂载录制流已落地：mount_writable、层 0 固定脚本到 D），C314 因此仍在未还表」 |
| F1 | .claude/kb/milestone/02-second-txn.md:14 | 事件句不改 | milestone:14 是「题面」表逐字引用 2026-09-16 建档当天 CLAUDE.md 原句的出处引文，冻结的历史引用 |
| F1 | .claude/kb/milestone/02-second-txn.md:325 | 不相干 | milestone:325 是待派任务描述「并进多次挂载录制流的检查」，属方法论安排，不是覆盖度断言 |
| F1 | .claude/kb/milestone/02-second-txn.md:332 | 事件句不改 | milestone:332 与 F6/F7 同一行，2026-09-17 逐句核过那天的记录 |
| F1 | .claude/kb/verification-build.md:155 | 要改 | verification-build.md:155 带日期「2026-09-14 现状：…其余七条要的覆盖写、释放、多次挂载、回退还不在层 0 的负载里」，按规则「带日期的现状句」判，须更新。改后：「2026-09-14 现状（现已推进）：…其余七条要的覆盖写、释放、多次挂载、回退已进层 0 的负载（里程碑「第二个事务」步 1–4 落地，层 0 固定脚本到 D），仍欠这几条各自的故障注入用例」 |
| F1 | records/2026-09-13-总审核.md:494 | 事件句不改 | records 总审核第 494 行是 2026-09-13 审核当天的快照记录，按惯例不回改 |
| F2 | .claude/kb/experiments.md:170 | 不相干 | OpenZFS 4K 随机读放大 32 倍是六家基线对比数据，不是本仓并行四条线覆盖状态的声明 |
| F2 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:4 | 不相干 | 描述用户 2026-09-15 定的实验维度，不是并行四条线是否开工的现状声明 |
| F2 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:72 | 不相干 | 六家之间 4K 随机读放大对比结果，与并行四条线的落点无关 |
| F2 | .claude/kb/milestone/02-second-txn.md:248 | 事件句不改 | milestone/02-second-txn.md 步 7 设想实现的规划文字，写的是那一步要做什么，不是当前覆盖状态 |
| F2 | .claude/kb/milestone/02-second-txn.md:458 | 不相干 | 并行线二小标题本身就是新事实落地后新增的内容，不是待更新的旧说法 |
| F3 | .claude/kb/checks-owed.md:309 | 事件句不改 | C338 行记的是 2026-09-16 定案那次发生的事，检查形态列已按现行 200 字节写 |
| F3 | .claude/kb/decisions/08-核心索引结构.md:474 | 事件句不改 | D8 决策正文带日期写「今天的预留是 76 字节」，已是当前值，是那次定案的记录 |
| F3 | .claude/kb/layout/01-first-txn.md:378 | 不相干 | format-const 的 stale= 标记串，不是自然语言现状句，且写范围明令不改 stale= |
| F3 | .claude/kb/milestone/02-second-txn.md:46 | 事件句不改 | milestone 现状（2026-09-16）落地段落，写的是那次定案当天做完的事 |
| F3 | .claude/kb/milestone/02-second-txn.md:48 | 事件句不改 | 设想实现段落记的是那一步打算怎么做，属于已执行完的历史规划文字 |
| F4 | .claude/kb/checks-owed.md:33 | 要改 | C22 前置列「层 0 的负载只有第一个事务，没有释放与重用」现已不成立：里程碑「第二个事务」步 1/2（覆盖写、释放）已落地、层 0 固定脚本已到 D（33 段、791624 个状态）。改后：「事务层、分配器、根环、崩溃点重放 harness（四样 2026-09-14 已有：`crates/singlefs-core`、门禁 54 号；层 0 的负载已不止第一个事务：步 1/2 覆盖写与释放已落地，固定脚本到 D）」 |
| F4 | .claude/kb/layout/01-first-txn.md:397 | 事件句不改 | 抬 F 空发布行写的是里程碑步 5 2026-09-17 落地那次发生的事，装置钉住的数据未过时 |
| F4 | .claude/kb/milestone/02-second-txn.md:141 | 事件句不改 | mount_writable 现状（2026-09-16 落地）段落，记的是那次实现落地 |
| F4 | .claude/kb/milestone/02-second-txn.md:195 | 事件句不改 | 可再分配谓词现状（2026-09-17 落地）段落，记的是那次实现落地 |
| F4 | .claude/kb/milestone/02-second-txn.md:332 | 事件句不改 | 会红的检查欠表第 22 行写的是 2026-09-17 逐句核过那天的结论，事件句 |
| F4 | .claude/kb/milestone/02-second-txn.md:358 | 事件句不改 | 崩溃点重放代价那一行写的是 2026-09-18 单进程跑完那次发生的事 |
| F4 | .claude/kb/verification-build.md:139 | 事件句不改 | verification-build 里 2026-09-14 起三件都接上了，属于起算日之后持续成立且未被推翻的状态记录 |
| F4 | CLAUDE.md:97 | 不相干 | CLAUDE.md 当前正文已写「两条流：第一个事务，以及固定脚本到 E」，已是更新后的现状，不是旧说法 |
| F5 | .claude/kb/checks-owed.md:324 | 要改 | C352 前置列「里程碑步 1 与步 2，2026-09-16 未开工」现已落地。改后：「里程碑「第二个事务」步 1 与步 2 已落地（2026-09-16/17，`publish_file_version`、`allocator::release`）；D19 已定项 12 正文仍没有「释放实验已跑，重判结论与依据」那一行」 |
| F5 | .claude/kb/decisions/08-核心索引结构.md:536 | 不相干 | D8 决策代价分析里的「覆盖写负载」是排空写出量对比的实验维度名，不是覆盖写功能是否落地的声明 |
| F5 | .claude/kb/decisions/09-加密.md:837 | 不相干 | D10 决策正文的设计原则「原地覆盖写会直接造成 nonce 重用」，与步 1/2 是否落地无关 |
| F5 | .claude/kb/experiments/05-nonce唯一性×崩溃点重放.md:10 | 不相干 | E05 实验方法描述「随机写+覆盖写+截断负载」，是加密崩溃点重放的测试设计，非覆盖状态声明 |
| F5 | .claude/kb/experiments/110-条带表在连续发布下的期望写放大.md:68 | 不相干 | E110 实验范围声明「只覆盖写字节这一维」，与步 1/2 落地状态无关 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:124 | 事件句不改 | E152 描述 2026-09-17 用 second-transaction 模式实际跑出覆盖写场景的那次实验，事件句 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:160 | 事件句不改 | E152 描述第三、四次正式跑取的发布 B（覆盖写）数据，事件句 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:191 | 不相干 | E152 里「一次覆盖写碰到的七棵树」是树增长后写放大的设计推论，非落地状态声明 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:222 | 事件句不改 | E152 对比表行是已测得的覆盖写与新建写字节等数据，事件句 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:245 | 不相干 | E152「差距怎么缩」是待走三方论证的设计推论，非落地状态声明 |
| F5 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:7 | 不相干 | E153 问题描述里的「连续覆盖写」是这个计数模型实验驱动的历史序列名，非落地状态声明 |
| F5 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:14 | 事件句不改 | E153 描述 World 驱动跑批里 H1（连续覆盖写）等历史已实际跑过，事件句 |
| F5 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:18 | 事件句不改 | E153（d）段发现意义描述是本仓第一次真跑连续覆盖写超过根环容量那次发现，事件句 |
| F5 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:25 | 不相干 | E153 根因分析讨论 G12 谓词架构问题，与步 1/2 落地状态无关 |
| F5 | .claude/kb/layout/01-first-txn.md:402 | 事件句不改 | layout 第二条流段序列记的是 2026-09-17 做到发布 E 那次固定脚本内容，事件句 |
| F5 | .claude/kb/milestone/02-second-txn.md:9 | 事件句不改 | milestone 题面表引用 verification-build.md 逐字出处，冻结历史引用 |
| F5 | .claude/kb/milestone/02-second-txn.md:14 | 事件句不改 | 同 F1 第 13 行，milestone:14 题面表冻结历史引用 |
| F5 | .claude/kb/milestone/02-second-txn.md:28 | 事件句不改 | milestone:28 固定脚本（预想）是 2026-09-16 建档时写的规划模板，后续步骤按它落地，历史规划文字 |
| F5 | .claude/kb/milestone/02-second-txn.md:29 | 事件句不改 | milestone:29 同上，固定脚本（预想）续写 |
| F5 | .claude/kb/milestone/02-second-txn.md:64 | 事件句不改 | milestone:64 现状（步 5 落地之后）段落，记的是那次实现结果 |
| F5 | .claude/kb/milestone/02-second-txn.md:72 | 不相干 | milestone:72 状态数计数方法论，覆盖写段序列预想是纯算术推导，非落地状态声明 |
| F5 | .claude/kb/milestone/02-second-txn.md:88 | 不相干 | milestone:88 是「步 1 覆盖写」小节标题本身，不是待更新的现状句 |
| F5 | .claude/kb/milestone/02-second-txn.md:98 | 不相干 | milestone:98 是 extent 单元覆盖写的设计原理说明，与落地状态无关 |
| F5 | .claude/kb/milestone/02-second-txn.md:117 | 事件句不改 | milestone:117 现状（2026-09-16 开工，随步 1 一起落地）段落，记的正是 F5 新事实本身 |
| F5 | .claude/kb/milestone/02-second-txn.md:195 | 事件句不改 | milestone:195 现状（2026-09-17 落地）段落，记的是可再分配谓词落地 |
| F5 | .claude/kb/milestone/02-second-txn.md:197 | 事件句不改 | milestone:197 设想实现段落写的是步 2 之后要怎么做，历史规划文字 |
| F5 | .claude/kb/milestone/02-second-txn.md:203 | 不相干 | milestone:203 是抬 F 上限计算方法的设计说明，非落地状态声明 |
| F5 | .claude/kb/milestone/02-second-txn.md:245 | 事件句不改 | milestone:245 现状（2026-09-17 跑完）段落，记的是那次实验结果 |
| F5 | .claude/kb/milestone/02-second-txn.md:248 | 不相干 | 同 F2 第 4 行，milestone:248 是设想实现规划文字 |
| F5 | .claude/kb/milestone/02-second-txn.md:266 | 事件句不改 | milestone:266 现状（2026-09-17 立；同日开工）段落，记的是写字节记账工作交回 |
| F5 | .claude/kb/milestone/02-second-txn.md:283 | 不相干 | milestone:283 是计数模型跑前登记的设计说明，与步 1/2 落地状态无关 |
| F5 | .claude/kb/milestone/02-second-txn.md:311 | 事件句不改 | milestone:311 收口表第②行是 2026-09-18 三方攻方腿发现的记录，事件句 |
| F5 | .claude/kb/milestone/02-second-txn.md:348 | 事件句不改 | milestone:348 第 20a 行记的是 2026-09-17 已改那次修复，事件句 |
| F5 | .claude/kb/milestone/02-second-txn.md:350 | 事件句不改 | milestone:350 第 20c 行同上，2026-09-17 已改，事件句 |
| F5 | .claude/kb/milestone/02-second-txn.md:356 | 事件句不改 | milestone:356 第 39 行是 2026-09-18 用户定案的记录，事件句 |
| F5 | .claude/kb/prior-art.md:568 | 不相干 | prior-art.md 是 bcachefs 对照与 D9/D10 设计原则引文，与步 1/2 落地状态无关 |
| F5 | .claude/kb/verification-build.md:155 | 要改 | 同 F1 第 16 行 verification-build.md:155，带日期现状句需要更新 |
| F5 | .claude/kb/verification-build.md:159 | 事件句不改 | verification-build.md:159 是 2026-09-14 那段决定「层 0 固定负载要含第二个事务——覆盖写」的设计理由陈述，是决策记录不是覆盖度现状句 |
| F5 | .claude/kb/verification-build.md:161 | 不相干 | verification-build.md:161 的「所以层 0 的固定脚本是…」是同一段设计推导的中间结论，讨论的是脚本是否含回退（F7 范围），与覆盖写是否落地（F5）无关 |
| F5 | .claude/kb/verification-build.md:233 | 不相干 | verification-build.md:233 是「推荐的落地顺序」表里第 4 项目标条目，是路线图条目不是覆盖度现状句 |
| F5 | .claude/kb/verification-build.md:237 | 要改 | verification-build.md:237 带日期「2026-09-14 现状：…覆盖写事务…都还没有代码」，覆盖写已落地。改后：「2026-09-14 现状（现已推进）：里程碑「第一个事务」步 0–7 做完…；覆盖写事务已落地（里程碑「第二个事务」步 1/2，2026-09-16/17）；第 2′ 步的 crash refinement、第 5 步的 RefFS 与模型对拍、第 6 步的 O3 仍没有代码」 |
| F5 | .claude/rules/implementation-workflow.md:44 | 事件句不改 | implementation-workflow.md:44 是 2026-09-16 那次「发布 B 写完但没人拦」事件，作为门禁 56 号存在理由的案例引用 |
| F5 | CLAUDE.md:97 | 不相干 | 同 F4 第 8 行，CLAUDE.md 当前正文已是更新后的现状 |
| F6 | .claude/kb/checks-owed.md:146 | 要改 | C143 前置列「多次挂载的录制流（与 C331、C332 同一个前置）」按字面暗示这个 harness 还不存在，而 mount_writable 已落地、层 0 固定脚本已到 D。改后：「多次挂载的录制流已落地（mount_writable 步 3 落地，层 0 固定脚本到 D），仍欠这条检查自己的故障注入（与 C331、C332 同一个前置）」 |
| F6 | .claude/kb/checks-owed.md:292 | 要改 | C314 前置列逐字「检查仍欠：崩溃点重放 harness（多次挂载的录制流）」是 F6 新事实明确要打假的原句。改后：「检查仍欠：这条故障注入的具体跑（多次挂载的录制流已落地：mount_writable、层 0 固定脚本到 D）；查账集合 2026-09-16 用户定案取窄读法（只隔离被抛弃根独占的槽，D23 已定项 14）」 |
| F6 | .claude/kb/checks-owed.md:302 | 要改 | C331 前置列「修法…要另过三方；多次挂载的录制流」暗示 harness 未落地。改后：「修法（计数器从全环最大 + 1 起、超级块带 txg、或记录扫描水位）要另过三方；多次挂载的录制流已落地，仍欠这条检查自己的故障注入」 |
| F6 | .claude/kb/checks-owed.md:303 | 要改 | C332 前置列同型。改后：「择根要不要看实例表、回退要不要多一份持久见证，没有条款；多次挂载的录制流已落地，仍欠这条检查自己的故障注入」 |
| F6 | .claude/kb/checks-owed.md:304 | 要改 | C333 前置列同型。改后：「行回收的实现；多次挂载的录制流已落地，仍欠这条检查自己的故障注入」 |
| F6 | .claude/kb/checks-owed.md:305 | 要改 | C334 前置列同型。改后：「多次挂载的录制流已落地（与 C329、C330 同一个前置），仍欠这条检查自己的故障注入」 |
| F6 | .claude/kb/checks-owed.md:306 | 要改 | C335 前置列同型。改后：「根环槽持续读失败的处置没有条款（重定位，或按设备维处理）；多次挂载的录制流已落地，仍欠这条检查自己的故障注入」 |
| F6 | .claude/kb/checks-owed.md:310 | 不相干 | C339 命中的是「检查形态」列描述怎么用多次挂载录制流做故障注入的方法论，不是断言它不存在 |
| F6 | records/2026-09-13-总审核.md:494 | 事件句不改 | records 总审核第 494 行是 2026-09-13 那次审核当天「还没做」的记录，按惯例是带日期的审核快照，不回改 |
| F7 | .claude/kb/checks-owed.md:33 | 要改 | 与 F4 同一行 C22，改后同：「层 0 的负载已不止第一个事务：步 1/2 覆盖写与释放已落地，固定脚本到 D」 |
| F7 | .claude/kb/checks-owed.md:129 | 要改 | C124 前置列逐字「2026-09-14 已有，门禁 54 号；层 0 没有回退」，而步 4 管理员回退已落地、层 0 固定脚本已到 D。改后：「崩溃点重放（verification-build.md 的第三样）——只剩这条检查自己的故障注入要等它（2026-09-14 已有，门禁 54 号；层 0 已含回退：步 4 落地，固定脚本到 D）」 |
| F7 | .claude/kb/checks-owed.md:200 | 要改 | C199 前置列逐字「崩溃点重放 harness…（门禁 54 号，只有一次挂载）」，而层 0 已到 D（含多次挂载与回退）。改后：「I-8.3 未实现、崩溃点重放 harness 已接入（门禁 54 号，层 0 固定脚本到 D，含多次挂载与回退）；改法本身要 D23 定案」 |
| F7 | .claude/kb/checks-owed.md:207 | 不相干 | C230 里「回退」用的是「静默回退到旧根」的一般语义，「门禁 54 号」说的是只录纯 SSD 线，两处都与管理员回退功能是否落地无关 |
| F7 | .claude/kb/checks-owed.md:263 | 不相干 | C282「门禁 54 号」出现在「两样 2026-09-14 已有」里，是历史时点记录；「回退」出自 E139 的标题，非覆盖度声明 |
| F7 | .claude/kb/layout/01-first-txn.md:397 | 事件句不改 | 抬 F 空发布行是步 5 2026-09-17 落地那次发生的事 |
| F7 | .claude/kb/layout/01-first-txn.md:402 | 事件句不改 | 第二条流段序列是 2026-09-17 做到发布 E 那次固定脚本内容的记录，事件句 |
| F7 | .claude/kb/milestone/02-second-txn.md:64 | 事件句不改 | 步 5 落地之后现状段落，记的是那次实现结果 |
| F7 | .claude/kb/milestone/02-second-txn.md:141 | 不相干 | mount_writable 现状段落里「回退」指的是「树表 0 条时的回退」这个具体边界情形，「门禁 54 号」指的是另一件云端腿枚举没有接进门禁 54 号，两处都不是「层 0 没有回退」的断言 |
| F7 | .claude/kb/milestone/02-second-txn.md:195 | 事件句不改 | 可再分配谓词现状（2026-09-17 落地）段落，其中「回退」指 rollback_floor_ceiling 已落地实现，与 F7 新事实一致 |
| F7 | .claude/kb/milestone/02-second-txn.md:222 | 事件句不改 | 现状（2026-09-17 部分落地）整段已写「层 0 的负载是两条流…固定脚本到 E」，与新事实一致 |
| F7 | .claude/kb/milestone/02-second-txn.md:332 | 事件句不改 | 会红的检查欠表第 22 行是 2026-09-17 逐句核过那天的记录 |
| F7 | .claude/kb/milestone/02-second-txn.md:358 | 事件句不改 | 崩溃点重放代价行是 2026-09-18 单进程跑完那次发生的事 |
| F7 | CLAUDE.md:97 | 不相干 | CLAUDE.md 当前正文已是更新后的现状（两条流：第一个事务，以及固定脚本到 E，含回退），不是旧说法 |

## 反向问一遍（第 10 步）

对 D13/D14/F1–F7 各条「新事实」做成的事，抽查它在对应载体里是否记着（不搜 `research/prompts/` 与 `briefs/`）：

- D13（树表条目 148→200）：`milestone/01-first-txn.md:103` 已写 200，已记（本表该行判「事件句不改」）。
- F2（并行四条线一节）：`milestone/02-second-txn.md:429` 标题「与主线并行的四条线」已在，已记。
- F3（开工前树表条目 148→200 一节）：`milestone/02-second-txn.md:46` 现状段已记，已记。
- F5（门禁 59 号、`crates/mutations.tsv`）：`grep -c` 在 `milestone/02-second-txn.md` 里分别命中 8 次、13 次，已记。
- F6（mount_writable 步 3 落地）：`milestone/02-second-txn.md:141` 已记。
- F7（mount_rollback 步 4 落地、层 0 到 D）：`milestone/02-second-txn.md:64/222`、`CLAUDE.md:97` 已记。
- F4（三方第一轮定案：影子账窄读法）：`decisions/23-journal的角色与格式.md` 命中「影子账」2 次，已记。

抽查范围内没有发现「做成的事在该记它的载体里一处都没记着」的情形，因此本报告「逐行判定」表未新增 M 组行。
这只是抽查，不是对 43 条事实表全表逐条做反向核对（那超出分给本组的 D13/D14/F1–F7 范围）。

## 没做什么

- 只判分给本组的候选行（D13、D14、F1–F7，共 240 行），F8 及更高编号、其余分组不归本轮。
- 反向检查（第 10 步）只抽查了七件事的落点，没有逐条重新走一遍事实表第 7 步的全部 H 编号来源核对。
- 对标「要人看」的 4 行（D13 里 checks-owed:159、decisions/06:251、experiments.md:150 各一，
  以及它们牵出的 milestone:344 已自行标注但未重算的 E131/E132 数字）没有去重算树表新旧口径下的
  具体数值，需要人核对 D6 已定项 3「66 头/44 头」按 200 字节条目宽重跑后的准确数字。
- 没有改动任何被搜到的文件、没有改 `stale=`。
