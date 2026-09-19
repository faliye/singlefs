# sweep-acceptance-2026-09-18-v2 逐行判：judge-7（组 D13, D14, F1, F2, F3, F4, F5, F6, F7）

阶段：sweep-acceptance-v2-judge-7。改动范围：基准 b1c8cef~1，结束 00c9d4f（读上下文一律 `git show 00c9d4f:路径`，不读工作区）。
候选表：`research/prompts/sweep-acceptance-2026-09-18-v2-candidates.tsv`；事实表：同目录 `-v2-facts.tsv`。

判据（定义第 9 步）：把候选行里说到的那件事换成事实表的「新事实」，原句还是不是真话。
「事件句不改」用于：(a) 明确带日期、描述「那一次发生的事」或段首逐字标注「历史/逐字保留」的正文；
(b) 已经正确反映新事实（新值/新状态）、不构成过时的现行表述——这类严格意义上不是「历史事件句」，
但候选表给定的五类判定里没有「已正确、无需改」这一类，故按其「不随本次事实变化而改」的效果并入此类，
并在理由栏写明「已正确写…」以便与真正的历史叙述区分。
「不相干」用于：候选行命中检索词但讨论的是不同的量、不同的对象，或不同的、仍然独立开着的检查缺口。

本轮共判 240 行（D13 138、D14 1、F1 17、F2 5、F3 5、F4 8、F5 43、F6 9、F7 14）。
发现五处真实过时（均标「要改」，同一处过时命中多个组时只登记一次改法）：
1. `.claude/kb/decisions/06-快照实现模型.md:251` 仍写「第一个事务多 148 字节」，树表条目已于 2026-09-16 加宽到 200（D13 组）。
2. `.claude/kb/checks-owed.md:33`（C22）前置列仍写「层 0 的负载只有第一个事务，没有释放与重用」，
   而层 0 第二条流已在 F5/F7/F7b 落地后覆盖释放、回退与重用（F4、F7 两组均命中同一行）。
3. `.claude/kb/milestone/02-second-txn.md:222` 步 6「现状」段仍写「部分落地」「26 条不变量的 checker」，
   而同一提交 `00c9d4f` 里 `checks-owed.md`／`verification-build.md` 均已记为 29 条（F7 组）。
4. `.claude/kb/verification-build.md:155` 仍写「其余七条要的覆盖写、释放、多次挂载、回退还不在层 0 的负载里」，
   而覆盖写／释放／多次挂载／回退均已落地进层 0 固定脚本（F1、F5 两组均命中同一行）。
5. `.claude/kb/verification-build.md:237` 仍写「覆盖写事务…都还没有代码」，而覆盖写事务已随步 1/2 落地（F5 组）。
6. `.claude/kb/checks-owed.md:324`（C352）前置列仍写「里程碑「第二个事务」步 1 与步 2，2026-09-16 未开工」，
   而步 1/2 已落地；检查本身（D19 已定项 12 缺一行）仍欠，只是这句状态描述过期（F5 组）。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| D13 | .claude/kb/checks-owed.md:150 | 不相干 | C148 的「148」是检查编号本身（C148），条目宽讨论的是 67→121 这次更早的加宽，与树表条目 148→200 无关 |
| D13 | .claude/kb/checks-owed.md:159 | 不相干 | C157 讨论 67/121/134 这一段更早的口径分歧（预留 32 时代），不是 148→200 这次 |
| D13 | .claude/kb/checks-owed.md:164 | 不相干 | C162 讨论记账条目 30→34，不是树表条目 |
| D13 | .claude/kb/checks-owed.md:178 | 不相干 | C180 讨论设备身份比对，提到的条目宽是假设中的 dev_uuid 字段，不是树表 |
| D13 | .claude/kb/checks-owed.md:187 | 不相干 | C202 讨论打包巡回空转开销，与树表条目宽无关 |
| D13 | .claude/kb/checks-owed.md:231 | 不相干 | C249 讨论跨装置闸钉的档位（DRAM 档 67/111），「别的条目宽」泛指，非树表 148/200 |
| D13 | .claude/kb/checks-owed.md:243 | 不相干 | C261 讨论分配记录条目宽 30 vs 20，不是树表 |
| D13 | .claude/kb/checks-owed.md:248 | 不相干 | C266 讨论 livelist 位置条目 14 字节，不是树表 |
| D13 | .claude/kb/checks-owed.md:285 | 不相干 | C306 讨论码 2 头不自描述 key 宽，未涉及树表条目宽度具体值 |
| D13 | .claude/kb/checks-owed.md:286 | 不相干 | C307 讨论映射树 key 宽（码 1/2/3），不是树表 |
| D13 | .claude/kb/checks-owed.md:374 | 不相干 | C321 讨论中央映射条目定宽 55，不是树表 |
| D13 | .claude/kb/decisions.md:168 | 不相干 | 目录项只提“种类到条目宽的表”这个词，不含具体数值 |
| D13 | .claude/kb/decisions/03-空间分配.md:179 | 不相干 | 分配记录条目 18→20 字节，不是树表 |
| D13 | .claude/kb/decisions/03-空间分配.md:198 | 不相干 | 扩展点分配表的 ppm 与条目宽线性关系，不是树表 |
| D13 | .claude/kb/decisions/03-空间分配.md:426 | 不相干 | t5 分配记录条目宽 20，不是树表 |
| D13 | .claude/kb/decisions/05-快照-空间记账机制.md:517 | 事件句不改 | 已正确写“树表单元…条目宽 2026-09-16 起 200”，带日期，与新事实一致 |
| D13 | .claude/kb/decisions/06-快照实现模型.md:235 | 不相干 | 讨论 livelist 按 5 档条目宽（24–56）的点查深度，不是树表条目宽度本身 |
| D13 | .claude/kb/decisions/06-快照实现模型.md:251 | 要改 | “第一个事务多 148 字节（2026-09-14 起的树表条目宽，D8（核心索引结构） 已定项 8）” 改为 “第一个事务多 200 字节（2026-09-16 起的树表条目宽，D8（核心索引结构） 已定项 8）” |
| D13 | .claude/kb/decisions/06-快照实现模型.md:255 | 事件句不改 | 段首逐字标注“定案之前那一格…逐字保留”，是冻结的历史正文，不随新事实改 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:14 | 不相干 | 泛指“声明长度=条目数×条目宽”的通用原则，不含具体数值 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:79 | 不相干 | 码 2 节点头自描述的通用定义段，未钉树表条目具体值 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:354 | 不相干 | 讨论身份引用条目宽 93→117→120，是内部条目非树表 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:459 | 不相干 | 讨论 inode 树 117／记账树 105 的节点条目宽，不是树表 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:469 | 事件句不改 | 2026-09-06 预留 32 字节的历史论证，早于 148/200 这次 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:475 | 事件句不改 | 已正确写 200/81/6561，并完整列出 121→145→148→200 的日期轨迹 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:483 | 不相干 | 2026-09-06 按条目宽 175/171 等讨论 inode/记账/extent 节点条目，不是树表 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:511 | 不相干 | 码 2 头自描述通用定义，不含树表具体数值 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:512 | 不相干 | 条目区连续排列的通用原则，不含树表具体数值 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:519 | 不相干 | 通用解码规则，不含具体值 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:523 | 不相干 | 讨论中央映射条目 55/53 两宽，不是树表 |
| D13 | .claude/kb/decisions/08-核心索引结构.md:524 | 不相干 | 映射 key 定宽 27 的定案，不是树表 |
| D13 | .claude/kb/decisions/12-目标介质.md:155 | 不相干 | 设备表条目宽度（2/4/8 台三档），不是树表 |
| D13 | .claude/kb/decisions/12-目标介质.md:158 | 不相干 | 同上，设备表条目宽度未定 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:152 | 不相干 | 码 2 声明长度通用定义，不含树表具体值 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:617 | 不相干 | 码 2 头字段通用列举，不含具体树表宽度 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:625 | 不相干 | 码 2 头通用定义与 E145 代价，非树表宽度值 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:796 | 不相干 | 索引节点通用字段表，不含具体宽度值 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:938 | 不相干 | 码 2 声明长度定义的小节标题，不含具体数值 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:940 | 不相干 | 码 2 声明长度的通用定义，不含具体数值 |
| D13 | .claude/kb/decisions/18-块里携带什么信息.md:945 | 不相干 | 码 2 头偏移表，字段名“条目宽 2”是通用占位，不含具体数值 |
| D13 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:285 | 不相干 | 表头，属指针宽度预算表，与树表无关 |
| D13 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:425 | 不相干 | 讨论中央映射条目哨兵问题，不是树表 |
| D13 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:429 | 不相干 | t7 中央映射树条目宽 55，livelist/旁表 36/31，均非树表 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:368 | 不相干 | 讨论 512 字节槽装 6 棵树（超级块设备表相关），不是树表条目 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:513 | 事件句不改 | 已正确写 200 字节并带 2026-09-14/2026-09-16 完整日期轨迹 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:514 | 事件句不改 | 已正确写 ⌊16253/200⌋=81 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:515 | 事件句不改 | 完整日期轨迹：148→200（2026-09-16）、145→148（2026-09-14），逐字保留 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:516 | 事件句不改 | 历史记录，日期 2026-09-13，讨论 243→112 旧口径（67 字节条目），早于 148/200 |
| D13 | .claude/kb/decisions/22-单元原子性怎么合成.md:659 | 事件句不改 | 段首逐字标注“逐字保留”的历史正文 |
| D13 | .claude/kb/experiments.md:118 | 不相干 | E100 超级块设备表条目宽 24/40/64，不是树表 |
| D13 | .claude/kb/experiments.md:150 | 不相干 | E132 五档条目宽指 livelist 扫描档（24–56），不是树表条目宽度值 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:1 | 不相干 | 设备表条目宽度实验标题 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:18 | 不相干 | 设备表条目宽度实验正文，不是树表 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:25 | 不相干 | 设备表条目宽度扫描说明，不是树表 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:42 | 不相干 | 设备表阴性对照条目宽 24，不是树表 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:44 | 不相干 | 设备表条目宽度未定档位说明，不是树表 |
| D13 | .claude/kb/experiments/100-超级块的三段几何.md:74 | 不相干 | 设备表内联上限结论，24/40/64 档，不是树表 |
| D13 | .claude/kb/experiments/101-节点头留位的代价与表达力.md:22 | 不相干 | 三棵已定条目宽的树是节点头留位实验对象，非树表 |
| D13 | .claude/kb/experiments/101-节点头留位的代价与表达力.md:24 | 不相干 | 节点头留位实验的通用扇出公式，不含具体值 |
| D13 | .claude/kb/experiments/101-节点头留位的代价与表达力.md:51 | 不相干 | 三棵已定条目宽的树扇出结论，非树表 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:27 | 不相干 | 内部条目宽 81/54 是 extent 树内部指针条目，不是树表 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:32 | 不相干 | 内部条目宽 54/81 两档扇出公式，非树表 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:38 | 不相干 | 节点头留位实验的通用阳性对照，非树表 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:39 | 不相干 | 条目宽超过可用字节的通用阴性对照 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:58 | 不相干 | 条目宽 81 指内部条目宽档位，得到的 200/201 是扇出数，与树表 148/200 巧合同形不同义 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:59 | 不相干 | 同一份通用阴性对照的结果行，非树表 |
| D13 | .claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:61 | 不相干 | E73 自己的条目宽 54，不是树表 |
| D13 | .claude/kb/experiments/128-指针带出生树txg之后的点查代价.md:36 | 不相干 | 点查代价实验的表头，泛指条目宽 |
| D13 | .claude/kb/experiments/128-指针带出生树txg之后的点查代价.md:120 | 事件句不改 | 历史记录，121/137 两个分母之争，早于 148/200 |
| D13 | .claude/kb/experiments/128-指针带出生树txg之后的点查代价.md:145 | 不相干 | 方法论描述，不含具体数值 |
| D13 | .claude/kb/experiments/130-每头一份livelist过不过有界销毁.md:104 | 不相干 | 记账条目宽 30，不是树表 |
| D13 | .claude/kb/experiments/131-livelist的两条载体数值与代价.md:92 | 不相干 | livelist 内部扇出 bug 描述，不是树表 |
| D13 | .claude/kb/experiments/132-livelist载体按真实树数与内部扇出重算.md:37 | 事件句不改 | 已正确写“树表每层 81 棵（16253/200…2026-09-16 用户定案加宽后的 200）” |
| D13 | .claude/kb/experiments/132-livelist载体按真实树数与内部扇出重算.md:38 | 不相干 | livelist 条目宽扫描 24–56，不是树表 |
| D13 | .claude/kb/experiments/132-livelist载体按真实树数与内部扇出重算.md:59 | 不相干 | livelist 条目宽只能扫的结论，非树表 |
| D13 | .claude/kb/experiments/133-映射key各候选的格式代价.md:157 | 不相干 | 映射树内部扇出 bug，不是树表 |
| D13 | .claude/kb/experiments/136-映射key岔路的代价行.md:39 | 不相干 | 六种条目宽指 extent/inode/记账/映射等节点，不是树表 |
| D13 | .claude/kb/experiments/136-映射key岔路的代价行.md:107 | 不相干 | 六种条目宽扇出代价表行，非树表 |
| D13 | .claude/kb/experiments/137-映射key形态的性能差距.md:3 | 不相干 | 泛指格式量，不含具体树表数值 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:8 | 不相干 | 讨论 C321 映射树条目单值问题，不是树表 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:9 | 不相干 | 讨论 G20 映射条目哨兵删除，非树表 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:49 | 不相干 | 通用结构检查变异描述，不含具体数值 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:172 | 不相干 | 通用码 2 头字段定义 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:174 | 不相干 | 通用码 2 声明长度定义 |
| D13 | .claude/kb/experiments/142-第一个事务的干跑.md:179 | 不相干 | 中央映射条目 55，不是树表 |
| D13 | .claude/kb/experiments/145-码2自描述头与映射树key宽的代价.md:33 | 不相干 | 码 2 自描述头字段的通用定义，非树表 |
| D13 | .claude/kb/experiments/145-码2自描述头与映射树key宽的代价.md:65 | 不相干 | 五棵树通用扇出实验，不含树表具体数值 |
| D13 | .claude/kb/experiments/145-码2自描述头与映射树key宽的代价.md:73 | 不相干 | 自描述头代价结论，五棵树通用，非树表 |
| D13 | .claude/kb/experiments/146-livelist条目按映射key定身份之后的宽度与代价.md:43 | 不相干 | livelist 每次 FREE 预付，不是树表 |
| D13 | .claude/kb/experiments/146-livelist条目按映射key定身份之后的宽度与代价.md:44 | 事件句不改 | 已正确写“树表多一条 200 字节…每单元 81 条”并带出处 |
| D13 | .claude/kb/experiments/20-指针变宽的CPU缓存代价.md:56 | 不相干 | 表头，指针缓存实验，非树表 |
| D13 | .claude/kb/experiments/20-指针变宽的CPU缓存代价.md:89 | 不相干 | 指针缓存实验阳性对照配置，非树表 |
| D13 | .claude/kb/experiments/20-指针变宽的CPU缓存代价.md:119 | 不相干 | 指针缓存实验对照判据，非树表 |
| D13 | .claude/kb/experiments/43-扩展点字节上限.md:35 | 不相干 | 位置条目宽度，不是树表 |
| D13 | .claude/kb/experiments/72-设备描述符表的挂载复算.md:9 | 不相干 | 设备表条目宽度的绝对值断言，非树表 |
| D13 | .claude/kb/experiments/72-设备描述符表的挂载复算.md:54 | 不相干 | 设备表条目宽度三档结论，非树表 |
| D13 | .claude/kb/experiments/72-设备描述符表的挂载复算.md:55 | 不相干 | 设备表条目宽度未定、40 字节是假设 |
| D13 | .claude/kb/experiments/72-设备描述符表的挂载复算.md:82 | 不相干 | 设备表条目宽度与 dev_uuid 宽度是假设 |
| D13 | .claude/kb/experiments/73-节点key区间的扇出代价.md:8 | 不相干 | 通用节点扇出公式 |
| D13 | .claude/kb/experiments/73-节点key区间的扇出代价.md:99 | 不相干 | key 变宽讨论，不含树表具体数值 |
| D13 | .claude/kb/experiments/74-分配记录的条目数与整理开销.md:11 | 不相干 | 分配记录整理开销公式，不是树表 |
| D13 | .claude/kb/experiments/79-根记录的容量.md:14 | 不相干 | 根记录容量的位置条目宽度，不是树表 |
| D13 | .claude/kb/experiments/79-根记录的容量.md:26 | 不相干 | 超级块 512 字节槽装 6 棵树，不是树表条目宽度 |
| D13 | .claude/kb/experiments/79-根记录的容量.md:33 | 不相干 | 根记录容量档位 61/67，不是树表 |
| D13 | .claude/kb/experiments/79-根记录的容量.md:36 | 事件句不改 | 历史记录 2026-09-13，讨论 243→112 旧口径（67 字节条目），早于 148/200 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:35 | 不相干 | 记账/分配记录扇出的通用公式，非树表 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:42 | 不相干 | 条目宽单调性的通用描述，非树表 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:68 | 不相干 | 记账/分配记录实验的通用阳性对照 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:102 | 不相干 | fanout 计算的通用公式，非树表条目 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:254 | 不相干 | 通用 COW 放大比值说明 |
| D13 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:255 | 不相干 | 打散搬迁节点数与条目宽无关的说明 |
| D13 | .claude/kb/layout/01-first-txn.md:124 | 不相干 | 位置条目宽度 4/14，不是树表 |
| D13 | .claude/kb/layout/01-first-txn.md:245 | 不相干 | inode 树根条目宽 120，不是树表 |
| D13 | .claude/kb/layout/01-first-txn.md:271 | 不相干 | extent 树节点条目宽 112，不是树表 |
| D13 | .claude/kb/layout/01-first-txn.md:286 | 不相干 | 分配记录树条目宽 20，不是树表 |
| D13 | .claude/kb/layout/01-first-txn.md:289 | 不相干 | 记账树条目宽 34，不是树表 |
| D13 | .claude/kb/layout/01-first-txn.md:368 | 事件句不改 | “树表单元自己的头”行已正确写条目宽 200、声明长度 1400 |
| D13 | .claude/kb/milestone/01-first-txn.md:103 | 事件句不改 | 现状段已正确写“树表条目 200” |
| D13 | .claude/kb/milestone/01-first-txn.md:110 | 不相干 | 码 2 头字段通用定义，不含树表具体条目宽数值 |
| D13 | .claude/kb/milestone/01-first-txn.md:130 | 不相干 | D8 已定项 11 标题引用，不含具体数值 |
| D13 | .claude/kb/milestone/01-first-txn.md:155 | 不相干 | C321 标题引用，讨论映射树，不是树表 |
| D13 | .claude/kb/milestone/02-second-txn.md:48 | 事件句不改 | “设想实现”段按计划描述改动前的旧值 148/24，是这次改动本身的施工记录 |
| D13 | .claude/kb/milestone/02-second-txn.md:51 | 不相干 | 验收标准泛指“动条目宽与预留宽的两条变异仍被抓”，不含具体数值 |
| D13 | .claude/kb/milestone/02-second-txn.md:52 | 事件句不改 | 已正确写“按 200/81/1400 过” |
| D13 | .claude/kb/milestone/02-second-txn.md:60 | 事件句不改 | 2026-09-16 复核记录，已正确写常量 200 三份装置同步 |
| D13 | .claude/kb/milestone/02-second-txn.md:344 | 不相干 | 讨论 D6 已定项 3 的 66/44 头数与决策正文未跟上，是另一条独立待办（livelist 头数口径），不是这条 milestone 步 3/5 现状字段本身 |
| D13 | .claude/kb/milestone/02-second-txn.md:464 | 事件句不改 | 2026-09-17 用户指示引述，已正确写“树表条目有 200 字节的元数据” |
| D13 | .claude/kb/verification-build.md:41 | 不相干 | 分配记录条目宽度，不是树表 |
| D13 | .claude/kb/verification-build.md:218 | 不相干 | 分配记录条目宽度历史争议，不是树表 |
| D13 | records/2026-08-29-组合对攻轮.md:222 | 不相干 | 明文映射层条目宽度，不是树表 |
| D13 | records/2026-09-03-验证三件套落地调研.md:34 | 不相干 | 分配记录条目宽度，不是树表 |
| D13 | records/2026-09-10-D19项7第一轮.md:14 | 不相干 | E20 指针缓存实验档位，内部条目宽 93/109/81/97，不是树表 |
| D13 | records/2026-09-10-D19项7第一轮.md:65 | 事件句不改 | 历史记录，讨论 E128 口径 121/137，早于 148/200 |
| D13 | records/2026-09-13-总审核.md:80 | 事件句不改 | 总审核历史记录条目，dated 2026-09-13 |
| D13 | records/2026-09-13-总审核.md:270 | 事件句不改 | 总审核历史审核表行，dated 2026-09-13 |
| D13 | records/2026-09-13-总审核.md:284 | 事件句不改 | 总审核历史审核表行，dated 2026-09-13 |
| D13 | records/2026-09-13-总审核.md:335 | 事件句不改 | 总审核历史叙述，dated 2026-09-13 |
| D13 | records/2026-09-13-总审核.md:345 | 事件句不改 | 映射条目宽 55，历史记录 dated 2026-09-13 |
| D14 | .claude/kb/milestone/01-first-txn.md:43 | 不相干 | 内容是 make_filesystem 的 mkfs 现状描述，与文件从 milestone-first-txn.md 搬到 milestone/01-first-txn.md 这次纯路径搬迁本身无关 |
| F1 | .claude/kb/checks-owed.md:146 | 不相干 | C143 仍是各自独立的检查缺口，未因步 3 落地而销掉 |
| F1 | .claude/kb/checks-owed.md:292 | 不相干 | C314“检查仍欠”经 2026-09-17 逐句核过仍未销（milestone:332/verification-build:139 同日复核） |
| F1 | .claude/kb/checks-owed.md:301 | 不相干 | C330 是独立检查缺口，未因建档动作而变 |
| F1 | .claude/kb/checks-owed.md:302 | 不相干 | C331 仍是独立检查缺口，未因建档动作而变 |
| F1 | .claude/kb/checks-owed.md:303 | 不相干 | C332 仍是独立检查缺口，未因建档动作而变 |
| F1 | .claude/kb/checks-owed.md:304 | 不相干 | C333 仍是独立检查缺口，未因建档动作而变 |
| F1 | .claude/kb/checks-owed.md:305 | 不相干 | C334 仍是独立检查缺口，未因建档动作而变 |
| F1 | .claude/kb/checks-owed.md:306 | 不相干 | C335 仍是独立检查缺口，未因建档动作而变 |
| F1 | .claude/kb/checks-owed.md:310 | 不相干 | C339 仍是独立检查缺口，未因建档动作而变 |
| F1 | .claude/kb/checks-owed.md:313 | 不相干 | C342 仍是独立检查缺口，未因建档动作而变 |
| F1 | .claude/kb/checks-owed.md:341 | 不相干 | C366 仍是独立检查缺口，未因建档动作而变 |
| F1 | .claude/kb/decisions/16-发布语义.md:207 | 不相干 | C314“检查”那一半仍欠，与 F1 的建档事实（milestone 文档收题）无关 |
| F1 | .claude/kb/milestone/02-second-txn.md:14 | 事件句不改 | “逐字出处”表冻结的是建档当日（2026-09-16）CLAUDE.md 的原句，按文档口径本就不随后续更新改 |
| F1 | .claude/kb/milestone/02-second-txn.md:325 | 不相干 | C334 收口表条目，独立于 F1 的建档事实 |
| F1 | .claude/kb/milestone/02-second-txn.md:332 | 事件句不改 | 2026-09-17 逐句核过的 dated 收口表条目 |
| F1 | .claude/kb/verification-build.md:155 | 要改 | “其余七条要的覆盖写、释放、多次挂载、回退还不在层 0 的负载里”已过期：覆盖写（F5 步 1/2）、释放（F5 步 2）、多次挂载（F6 步 3）、回退（F7 步 4）均已落地并进层 0 固定脚本（到 E）。改为：“覆盖写、释放、多次挂载、回退均已进层 0 的负载（步 1/2/3/4 落地，固定脚本到 E）；其余各条要的验证覆盖是否补齐，见收口表”
| F1 | records/2026-09-13-总审核.md:494 | 事件句不改 | 总审核历史记录，dated 2026-09-13 |
| F2 | .claude/kb/experiments.md:170 | 事件句不改 | E152 按里程碑分别报告的历史数据，“第一个事务上只比得了三维”是对那个里程碑的准确历史陈述 |
| F2 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:4 | 事件句不改 | 2026-09-15 定向的历史记录 |
| F2 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:72 | 不相干 | 六家 4K 随机读放大倍数的基线比较，不是 singlefs 自己并行线二的进度断言 |
| F2 | .claude/kb/milestone/02-second-txn.md:248 | 不相干 | “预想能跑七维”是规划性描述，与 F2 新事实“现状都是未开工”一致，不构成过时 |
| F2 | .claude/kb/milestone/02-second-txn.md:458 | 不相干 | 只是并行线二小节标题，不含状态断言 |
| F3 | .claude/kb/checks-owed.md:309 | 事件句不改 | 已正确标注“2026-09-16 用户定案已把条目加宽到 200、预留 76”，其余是立账时原文的冻结引用 |
| F3 | .claude/kb/decisions/08-核心索引结构.md:474 | 事件句不改 | 已正确写 200/76 并带 2026-09-14→2026-09-16 完整日期轨迹 |
| F3 | .claude/kb/layout/01-first-txn.md:378 | 事件句不改 | format-const 的 stale= 标记本身即登记旧值供门禁盯防，是这次改动正确落地的产物，不是过时内容 |
| F3 | .claude/kb/milestone/02-second-txn.md:46 | 事件句不改 | “现状（2026-09-16）：已落地”一段正是 F3 这条事实所描述的落地记录本身 |
| F3 | .claude/kb/milestone/02-second-txn.md:48 | 事件句不改 | “设想实现”段是施工计划记录，按设计引用改动前的旧值 148/24 |
| F4 | .claude/kb/checks-owed.md:33 | 要改 | “层 0 的负载只有第一个事务，没有释放与重用” 改为 “层 0 第二条流（固定脚本到 E）已含释放（步 2）与重用（步 5 抬 F 复用），四样前置全部齐备”——F5/F7b 落地后层 0 固定脚本已远超第一个事务 |
| F4 | .claude/kb/layout/01-first-txn.md:397 | 事件句不改 | 已正确记录步 5（抬 F 空发布）2026-09-17 落地 |
| F4 | .claude/kb/milestone/02-second-txn.md:141 | 不相干 | 步 3（mount_writable）落地描述，不是 F4 的三方定案事实（影子账/C310/层 0 负载范围/字节登记位） |
| F4 | .claude/kb/milestone/02-second-txn.md:195 | 不相干 | 步 5（reclaim_released_up_to）落地描述，不是 F4 的三方定案事实 |
| F4 | .claude/kb/milestone/02-second-txn.md:332 | 事件句不改 | 2026-09-17 逐句核过的 dated 收口表条目 |
| F4 | .claude/kb/milestone/02-second-txn.md:358 | 不相干 | 验证装置代价追踪条目（2026-09-18），与 F4 三方定案无关 |
| F4 | .claude/kb/verification-build.md:139 | 不相干 | checker 条数已正确写 29（与 B16 事实一致），不是 F4 的三方定案范围 |
| F4 | CLAUDE.md:97 | 事件句不改 | 这正是替换旧句之后的现行正确表述，是 F4/F7 等多处改写的最终落点 |
| F6 | .claude/kb/checks-owed.md:146 | 不相干 | C143 仍是独立检查缺口，未因步 3 多次挂载（mount_writable）落地而解决，需要更细的故障注入 |
| F6 | .claude/kb/checks-owed.md:292 | 不相干 | C314 同上，2026-09-17 逐句核过仍未销 |
| F6 | .claude/kb/checks-owed.md:302 | 不相干 | C331 未因步 3 多次挂载落地而解决 |
| F6 | .claude/kb/checks-owed.md:303 | 不相干 | C332 未因步 3 多次挂载落地而解决 |
| F6 | .claude/kb/checks-owed.md:304 | 不相干 | C333 未因步 3 多次挂载落地而解决 |
| F6 | .claude/kb/checks-owed.md:305 | 不相干 | C334 未因步 3 多次挂载落地而解决 |
| F6 | .claude/kb/checks-owed.md:306 | 不相干 | C335 未因步 3 多次挂载落地而解决 |
| F6 | .claude/kb/checks-owed.md:310 | 不相干 | C339 未因步 3 多次挂载落地而解决 |
| F6 | records/2026-09-13-总审核.md:494 | 事件句不改 | 总审核历史记录，dated 2026-09-13 |
| F7 | .claude/kb/checks-owed.md:33 | 要改 | “层 0 的负载只有第一个事务，没有释放与重用” 改为 “层 0 第二条流（固定脚本到 E）已含释放、重用与回退（步 2/4/5 落地）”——F7 落地后回退（步 4）也已进层 0 固定脚本 |
| F7 | .claude/kb/checks-owed.md:129 | 不相干 | C124 是独立检查缺口（回退行与重放下界的分辨检查），未因 mount_rollback 代码落地而解决 |
| F7 | .claude/kb/checks-owed.md:200 | 不相干 | C199 讨论实例代号与 jsn 断号矛盾，不是 F7 的回退落地事实 |
| F7 | .claude/kb/checks-owed.md:207 | 不相干 | C230 讨论 zoned 设备 ZoneReset 之后态分不开，不是 F7 的回退落地事实 |
| F7 | .claude/kb/checks-owed.md:263 | 不相干 | C282 讨论环里最旧根定义，仍欠“会红的检查”，与 F7 的回退代码落地无关 |
| F7 | .claude/kb/layout/01-first-txn.md:397 | 事件句不改 | 已正确记录步 5（抬 F 空发布）2026-09-17 落地 |
| F7 | .claude/kb/layout/01-first-txn.md:402 | 事件句不改 | 第二条流段序列描述精确匹配 F7 新事实：到 D（33 段、791624 个状态） |
| F7 | .claude/kb/milestone/02-second-txn.md:64 | 事件句不改 | “现状（2026-09-17 步 5 落地之后）”精确匹配到 D（33 段、791624）与到 E 的段序列 |
| F7 | .claude/kb/milestone/02-second-txn.md:141 | 不相干 | 步 3（mount_writable）落地描述，不是 F7 的步 4 回退事实 |
| F7 | .claude/kb/milestone/02-second-txn.md:195 | 不相干 | 步 5（reclaim）落地描述，不是 F7 的步 4 回退事实 |
| F7 | .claude/kb/milestone/02-second-txn.md:222 | 要改 | 步 6“现状”段落写“部分落地”“26 条不变量的 checker”（清单 26 条）已过期：同一提交 00c9d4f 里 checks-owed.md 与 verification-build.md 均已记为 29 条（新增 I-3.8、I-7.4、I-4.8、I-3.9、I-9.14、I-5.4），此段落未跟上。改为：“现状（2026-09-17 步 6 落地）：…（exhaustive=true，每个状态两遍恢复 + 29 条不变量的 checker + 记录核对器）；…checker 新接了 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽）、I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变）、I-5.4（分配记录罩住的槽互不相交）…清单 29 条” |
| F7 | .claude/kb/milestone/02-second-txn.md:332 | 事件句不改 | 2026-09-17 逐句核过的 dated 收口表条目 |
| F7 | .claude/kb/milestone/02-second-txn.md:358 | 不相干 | 验证装置代价追踪条目（2026-09-18），与 F7 的回退落地事实无关 |
| F7 | CLAUDE.md:97 | 事件句不改 | 现行正确表述，与 F7 新事实（层 0 固定脚本到 E）一致 |
| F5 | .claude/kb/checks-owed.md:324 | 要改 | “里程碑「第二个事务」步 1 与步 2，2026-09-16 未开工”已过期：步 1/2（覆盖写、释放）已落地（publish_file_version、allocator::release）。改为：“里程碑「第二个事务」步 1 与步 2 已落地（2026-09-16/17）；门禁要求 D19（块指针的结构与宽度预算） 已定项 12 正文补的那一行「释放实验已跑，重判结论与依据」仍未见，检查仍欠” |
| F5 | .claude/kb/decisions/08-核心索引结构.md:536 | 不相干 | 讨论 write buffer 排空到叶的代价，覆盖写只是其中一种测试负载，不是 F5 的落地状态断言 |
| F5 | .claude/kb/decisions/09-加密.md:837 | 不相干 | D10 关于原地覆盖写与 nonce 重用的设计原则，与 F5 的落地状态无关 |
| F5 | .claude/kb/experiments/05-nonce唯一性×崩溃点重放.md:10 | 不相干 | 实验方法描述，覆盖写只是负载的一部分，不涉及 F5 落地状态 |
| F5 | .claude/kb/experiments/110-条带表在连续发布下的期望写放大.md:68 | 不相干 | RAID 条带实验的范围说明，与 F5 落地状态无关 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:124 | 事件句不改 | 2026-09-17 dated 实验记录，跑的正是已落地的覆盖写（second-transaction 模式），与新事实一致 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:160 | 事件句不改 | 2026-09-17 dated 实验记录，取发布 B（覆盖写）已落地的结果 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:191 | 事件句不改 | dated 实验观测（第三、四次跑），与新事实一致 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:222 | 事件句不改 | dated 实验对照表行，覆盖写已落地并已实测 |
| F5 | .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:245 | 不相干 | 前瞻性设计推论说明，不含落地状态断言 |
| F5 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:7 | 不相干 | E153 账本实验以覆盖写为既有构件描述测试历史，与 F5 落地状态断言无关 |
| F5 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:14 | 不相干 | 同上，World 驱动方法列举，不含落地状态断言 |
| F5 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:18 | 不相干 | 同上，段的发现意义说明 |
| F5 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:25 | 不相干 | 同上，G12 根因分析，不含落地状态断言 |
| F5 | .claude/kb/layout/01-first-txn.md:402 | 事件句不改 | 第二条流段序列描述，覆盖写×4 是已落地固定脚本的一部分，与新事实一致 |
| F5 | .claude/kb/milestone/02-second-txn.md:9 | 事件句不改 | “逐字出处”表冻结建档当日引用的 verification-build.md 原句 |
| F5 | .claude/kb/milestone/02-second-txn.md:14 | 事件句不改 | “逐字出处”表冻结建档当日 CLAUDE.md 原句 |
| F5 | .claude/kb/milestone/02-second-txn.md:28 | 事件句不改 | “固定脚本（预想）”是建档当日的规划段落，标明“预想”不是现状断言 |
| F5 | .claude/kb/milestone/02-second-txn.md:29 | 事件句不改 | 同上，规划段落续行 |
| F5 | .claude/kb/milestone/02-second-txn.md:64 | 事件句不改 | “现状（2026-09-17 步 5 落地之后）”精确匹配已落地状态 |
| F5 | .claude/kb/milestone/02-second-txn.md:72 | 不相干 | 步 0 状态数估算方法说明，覆盖写只是估算里的一个假设负载，不含落地状态断言 |
| F5 | .claude/kb/milestone/02-second-txn.md:88 | 不相干 | 仅为“步 1 覆盖写”小节标题，不含状态断言 |
| F5 | .claude/kb/milestone/02-second-txn.md:98 | 不相干 | 覆盖写机制的技术说明（整单元重写），不含落地状态断言 |
| F5 | .claude/kb/milestone/02-second-txn.md:117 | 事件句不改 | “现状（2026-09-16 开工，随步 1 一起落地）”正是 F5 新事实本身的落地记录 |
| F5 | .claude/kb/milestone/02-second-txn.md:195 | 不相干 | 步 5（reclaim）落地描述，不是 F5 的步 1/2 事实 |
| F5 | .claude/kb/milestone/02-second-txn.md:197 | 不相干 | 步 5“设想实现”段落的规划性描述，不含 F5 落地状态断言 |
| F5 | .claude/kb/milestone/02-second-txn.md:203 | 不相干 | 步 5 规划段落续行，不含 F5 落地状态断言 |
| F5 | .claude/kb/milestone/02-second-txn.md:245 | 事件句不改 | “现状（2026-09-17 跑完）”dated 记录 E152 重跑覆盖写场景的结果 |
| F5 | .claude/kb/milestone/02-second-txn.md:248 | 不相干 | 并行线规划段落，“主线步 1 的覆盖写”作为已落地构件被引用，与新事实一致，不构成过时 |
| F5 | .claude/kb/milestone/02-second-txn.md:266 | 不相干 | 增补 1 的现状记录，引用发布 B（已落地）的写调用数，不是 F5 落地状态断言本身 |
| F5 | .claude/kb/milestone/02-second-txn.md:283 | 不相干 | 增补 1 计数模型规划段落，不含 F5 落地状态断言 |
| F5 | .claude/kb/milestone/02-second-txn.md:311 | 不相干 | 增补 2 收口表条目，讨论 F 生效回落等其它议题，与 F5 落地状态无关 |
| F5 | .claude/kb/milestone/02-second-txn.md:348 | 不相干 | 收口表 20a 行，讨论取号准入缺口，覆盖写只是触发场景描述，不是 F5 落地状态断言 |
| F5 | .claude/kb/milestone/02-second-txn.md:350 | 不相干 | 收口表 20c 行，讨论聚簇段回落缺口，与 F5 落地状态无关 |
| F5 | .claude/kb/milestone/02-second-txn.md:356 | 不相干 | 收口表 39 行，讨论分配记录树上限精度缺口，覆盖写只是场景描述，与 F5 落地状态无关 |
| F5 | .claude/kb/prior-art.md:568 | 不相干 | bcachefs nocow/加密的外部先例引用，与 F5 落地状态无关 |
| F5 | .claude/kb/verification-build.md:155 | 要改 | 同 F1 组该行：“其余七条要的覆盖写、释放、多次挂载、回退还不在层 0 的负载里”已过期，覆盖写/释放/多次挂载/回退均已落地进层 0 固定脚本（到 E）。改法同 F1 组该行 |
| F5 | .claude/kb/verification-build.md:159 | 事件句不改 | 2026-09-14 起的设计论证段落（说明为何要把覆盖写、释放纳入层 0 固定负载），是导向 2026-09-13 C314 定案登记的历史推理，不是现状断言 |
| F5 | .claude/kb/verification-build.md:161 | 事件句不改 | 描述的是“崩溃点重放第一版”的计划构成，明确限定版本范围，不是当前全量负载的现状断言 |
| F5 | .claude/kb/verification-build.md:233 | 事件句不改 | “推荐的落地顺序”表格条目（目标描述），不含状态断言 |
| F5 | .claude/kb/verification-build.md:237 | 要改 | “覆盖写事务、第 2′ 步的 crash refinement、第 5 步的 RefFS 与模型对拍、第 6 步的 O3（独立规约执行器） 都还没有代码”已过期：覆盖写事务已落地。改为：“覆盖写事务已落地（里程碑「第二个事务」步 1/2）；第 2′ 步的 crash refinement、第 5 步的 RefFS 与模型对拍、第 6 步的 O3（独立规约执行器） 仍没有代码” |
| F5 | .claude/rules/implementation-workflow.md:44 | 事件句不改 | 2026-09-16 dated 的具体事件记录（用作规则说明的案例），不是现状断言 |
| F5 | CLAUDE.md:97 | 事件句不改 | 现行正确表述，与 F5 新事实（步 1/2 已落地）一致 |
## 没做什么

- 只判分给我的组（D13、D14、F1、F2、F3、F4、F5、F6、F7；派发提示原文与交回前必跑的 `--check-report --groups D13,D14,F1-F7` 一致，F1-F7 是含 F5 的范围——本报告草稿阶段一度漏抽 F5 的候选行，发现后已补判满），F7b 及其余各组不归本报告。
- 未改任何被判到的文件，未改 `stale=`；发现的六处「要改」（decisions/06:251、checks-owed.md:33/C22、milestone/02-second-txn.md:222、verification-build.md:155、verification-build.md:237、checks-owed.md:324/C352）只给改后的句子，交主 agent 或 kb-scribe 落地。
- 同一处过时命中多个组时（checks-owed.md:33 命中 F4/F7；verification-build.md:155 命中 F1/F5）只登记一条改法，不重复处理。
- 未反向核对「载体里有没有一处记着」（第 10 步「M 组」）：派发提示未把该子步骤分给本轮（只分了逐行判候选表这一段）；主 agent 若需要该反向核对，需另行派发。
- 未验证 C22（checks-owed.md:33）之外，checks-owed.md 里其余因层 0 负载扩大而可能连带过时的条目是否还有遗漏——只核了候选表命中的行，未做超出候选表范围的全表扫描。
