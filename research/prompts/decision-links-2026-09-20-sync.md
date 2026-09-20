<!-- knowledge-sync -->
# decision-links-2026-09-20 阶段同步

触发文件：.claude/rules/format-evolution.md、.claude/rules/fs-design.md、.claude/gate.d/lib-item-ref-status.py、.claude/gate.d/lib-history-brief.py、.claude/gate.d/75-decision-experiment-links.sh

这一阶段做成的事：C23、C32、C33、C134、C136 五笔账按无密钥 scrub 这一侧改写（编号留着、不销账，简称跟着题面改，全仓 25 处引用一次改齐），D9（加密） 已定项 7 的欠段改挂已定项 5；第三批决策瘦身十五条（D1、D2、D3、D5、D6、D7、D10、D12、D15、D17、D20、D21、D24、D25、D26），正文 247 479 → 183 608 字、日期 678 → 58，把 73 节「是现行规则却不是编号分项」的小节立成编号分项；新立 C411（分配记录的跨度段与单元类别不符没检查）、C412（整理的挂钟代价没量过）、C413（碎片度与全空段统计没有不变量）、C414（挂载时不比对 io_min 与记录的槽距） 四笔账。

`.claude/rules/format-evolution.md` 这一轮改了一处：「让格式永久化的那个动作」一节里那句实测陈述的指向改成 D15（格式冻结政策） 已定项 7，并去掉组件计数（那句说的是 2026-09-09 那天实测到什么，那天是两个组件，2026-09-11 才有第三个，把数改成「三个」会让它说假话）。`.claude/rules/fs-design.md` 改了两处指向：「记账是事务的副产品」一节指 D5（快照 / 空间记账机制） 的那一行改成已定项 4 的统计量清单第 8 行（原指向本来就错，指的是索引表第 8 行），「一个事务层，所有结构共用」一节指 D17（实现分层与第三方管道） 的那一半加上已定项 6。两处的逐字引文都没动。

## 搜索

- `grep -rn '让渡区' .claude/kb/decisions/ .claude/kb/invariants.md .claude/rules/ | grep -v decisions-history | wc -l` → 5（D9（加密） 已定项 7 / 9 的撤回记录各一、D9（加密） 射程一、D2（RAID 条带策略） 一处不相关、D3（空间分配） 分配准入合取式第 3 条）
- `grep -rn '无分项' .claude/kb/ records/ --include='*.md' | grep -v history | wc -l` → 3（`decisions.md` 的格式说明、C237（码 1 / 码 2 的指针里没有出生 (树, txg) 的位置） 里指字段表的「宽度无分项」、`layout/01-first-txn.md` 一句事件陈述）
- `grep -rn 'D2（RAID 条带策略）[^|]\{0,30\}硬要求\|D2 硬要求' .claude/kb/ research/ --include='*.md' | grep -v 'research/prompts\|history' | wc -l` → 7（E126（超级块槽宽四条候选的代价） 三处、`checks-owed.md` 三处、D22（单元原子性怎么合成） 一处）
- `grep -rn '六个未定项' .claude/kb/ records/ --include='*.md' | grep -v history | wc -l` → 1（E96（混合架构的一致性） 第 3 行）
- `grep -rni 'scrub\|无密钥\|keyless' crates/*/src/ | wc -l` → 0
- `grep -rn '无密钥挂载' .claude/kb/ .claude/rules/ crates/ | wc -l` → 6（全部在已作废的欠账行与决策变更史里，现行 kb 正文零命中）
- `grep -c '碎片度\|全空段' .claude/kb/invariants.md` → 0；`grep -c 'minimum_input_output_bytes' crates/singlefs-core/src/mount.rs` → 0

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/rules/format-evolution.md:59 | 实测（2026-09-09）：`.claude/kb/decisions/15-格式冻结政策.md` 的两个独立冻结组件里， | 改了：指向改成 D15（格式冻结政策） 已定项 7，并去掉组件计数——那句说的是那一天实测到什么，把「两个」改成「三个」会让它说假话 |
| .claude/rules/fs-design.md:32 | 权威记录在 `.claude/kb/decisions/05-快照-空间记账机制.md` D5（快照 / 空间记账机制） 已定项索引表第 8 行 | 改了：改成已定项 4 的统计量清单第 8 行，原指向指的是索引表第 8 行、说的是另一件事 |
| .claude/rules/fs-design.md:117 | 见 [decisions.md](../kb/decisions.md) D17 与 D13。 | 改了：指 D17（实现分层与第三方管道） 的那一半加上已定项 6，指 D13（验证路线） 的那一半没动 |
| .claude/kb/checks-owed.md:89 | D2（RAID 条带策略）硬要求「**不发出小于 `io_min` 的写**」 | 改了：改成 D2（RAID 条带策略） 已定项 16，引文一个字没动 |
| .claude/kb/checks-owed.md:196 | **D2（RAID 条带策略）「写的粒度」已定硬要求 1 逐字是「不发出小 | 改了：改成 D2（RAID 条带策略） 已定项 16 |
| .claude/kb/decisions/22-单元原子性怎么合成.md:117 | D2（RAID 条带策略）「写的粒度」两条硬要求都写进去了 | 改了：改成 D2（RAID 条带策略） 已定项 16 与已定项 17 |
| .claude/kb/decisions/22-单元原子性怎么合成.md:66 | D2（RAID 条带策略） 已定项 7 逐字「**用户定案：归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来。**」 | 改了：只引规则句，「用户定案：」这个来历前缀随瘦身进了变更史 |
| .claude/kb/checks-owed.md:199 | ⚠️ 先量误判率：仓里仍有「不编号的独立定案小节」，都在还没瘦身的决策里——D2（RAID 条带策略）…、D6（快照实现模型）…、D10（随机写 / 碎片化的真答案）…都是 | 改了：这类小节 2026-09-20 现查在决策正文里已经清零，要量误判率得另找语料 |
| .claude/kb/experiments/96-混合架构的一致性.md:3 | **问题**：D26（后台整理与放置回收）「六个未定项的论证材料」推荐的位置权威形态是 | 改了：改成不带引号的陈述；并给这一页补了整张「### 影响的决策」表（门禁 75 号判「正文改了而表一行都没回看」） |
| .claude/kb/decisions/03-空间分配.md:136 | ⚠️ D9（加密） 已定项 7 随已定项 5 改取「只做 scrub」而撤回（让渡区 A 无对象）⇒ 这个合取恒真，条款原样保留 | 不改：分配准入合取式第 3 条自己已经写明今天没有对象、是知情保留；要不要从表里撤掉是设计改动，回扫的账在 C167（明文映射层的引用全仓回扫没做） |
| .claude/kb/experiments/126-系统配置槽宽四条候选的代价.md:22 | \| **乙** \| `= max(physical_block_size, io_min)`（探测值） \| D2（RAID 条带策略） 已定硬要求 1 | 不改：跑前写死的臂定义，改它等于事后改判据（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」） |
| .claude/kb/experiments/129-小于iomin的写在真设备上怎么出事.md:5 | 它答两件事……逐字引 D2（RAID 条带策略） 旧小节名 | 不改：跑前登记的复述；页内三处结论段的指向已由做 D2（RAID 条带策略） 的那一组改成已定项 16 |
| .claude/kb/decisions.md:19 | 没有分项的决策那一格写 `无分项 · 整条已定` | 不改：这是索引页的格式说明，今天 D8（核心索引结构）、D16（发布语义）、D19（块指针的结构与宽度预算） 三条还没瘦身，规则仍有对象 |
| .claude/kb/layout/01-first-txn.md:480 | 表里那四处「无分项」当天各自立成了未定项 | 不改：说的是那一次发生的事 |
| .claude/gate.d/75-decision-experiment-links.sh:236 | 「**状态：已定。**」是 20 号要的规范标记（检索端出来的是这一行），不算在一句话的字数里 | 改了：量字数前也剥掉「改第一个事务的字节：否（日期，依据：…）」——31 号要它写在未定项登记行里而且必须带日期，与 75 号的「不带日期、不超过 100 字」直接相撞，补完字节判决四行一起红。判别力：把剥这一段的那行去掉，四处当场红回来 |
| .claude/rules/format-evolution.md:30 | 索引表每行一句话定案，不抄分项正文，不带日期，不超过 100 字 | 补了：未定项登记行里的字节判决同样是规范标记、不算字数，那个日期也不算「带日期」 |
| .claude/gate.d/lib-item-ref-status.py:58 | （新增） | 补了：门禁 22 号跳过代码围栏——变更史的「改前」原行、产物行、源码片段都是逐字照抄的东西，按今天的状态判它们等于要求证据跟着现状改。判别力：绿样本里加一段围栏、里面放一处状态写反的引用，仍判绿；把围栏标记去掉，同一行当场判红 |
| .claude/gate.d/lib-history-brief.py:100 | def cell(text): return text.replace(…) | 改了：搬快查行时把每个以 ../ 开头的链接目标去掉一层——源在 decisions-history/、汇总在 kb/，原样搬过来指到 .claude/prior-art.md，门禁 23 号判红 6 处 |
| .claude/kb/decisions-history/2026-09.md:3904 | [decisions/18-块里携带什么信息.md](18-块里携带什么信息.md) | 改了：相对路径补一层 ../decisions/，三处（第二批写下的，不是这一轮带进来的） |
| .claude/kb/decisions/02-RAID条带策略.md:39 | （新增） | 补了：未定项 21 的「改第一个事务的字节：否（2026-09-20，依据：第一版两盘镜像按 parity = 1 实现，今天没有第二种取值写得出不同的字节）」，门禁 31 号要它写在登记行里 |
| .claude/kb/decisions/26-后台整理与放置回收.md:200 | （新增） | 补了：未定项 8 / 9 / 10 各自的「改第一个事务的字节：否（2026-09-20，依据：…）」，原来只写在表下那一段里、门禁 31 号定位不到 |
| .claude/kb/experiments/10-老化结构抗老化vs事后整理.md:5 | 两条路是替代还是叠加，决定 D3（空间分配） 第 2 条的分量 | 改了：指向改成已定项 14。同一页第 14 行「判据」块里同样的旧叫法没动——那是跑前写死的判据 |
| .claude/kb/experiments/56-消息缓冲的收益vsε.md:24 | D11（索引节点要不要留消息缓冲区） 前置 4 要的「节点大小目标线」已经有值 | 不改：D11（索引节点要不要留消息缓冲区） 瘦身后那张前置表没了，但这句住在「### 为什么现在跑」里，说的是建这个实验那天 D11 的结构；同页第 292、390 行与 E7（离线索引 harness） 第 57、289 行同理，都在带日期的小节里 |
| .claude/kb/experiments/57-明文侧字段集的权威落点.md:87 | decisions/09-加密.md:173、decisions/18-块里携带什么信息.md:205 与 :218、decisions/21-权威态与派生态的分界.md:6-8 | 不改：四处行号今天都指不到那句话，但它们是 2026-08-31 一次逐字核的事件记录，不是现状句。全仓写 kb 行号的只有这一页 |
| .claude/kb/experiments/104-扫描重建的现行版本判定.md:88 | 判据 1：全规则臂在 24 个种子 × 十八个世界……全部 0 | 不改：现查三个数全对——产物 23 类世界 × 12 臂 × 3 种子 = 828 行 cell，全规则臂全 0 的恰好 18 类；24 与 12 出自单测 `full_rule_is_exact_everywhere` / `scan_count_matches_generator`。`records/` 里先前记的那条「要回产物数」判错了，已改 |
| .claude/kb/checks-owed.md:122 | \| C119 \| 两个冻结组件没有 spec 文件 \| | 改了：简称改成「冻结组件没有 spec 文件」，不带计数——2026-09-11 D16（发布语义） 已定项 3 立了第三个组件（journal 记录格式），简称一直没跟。连带改 10 处引用，代码围栏里逐字照抄的 3 处不动 |
| .claude/kb/decisions/22-单元原子性怎么合成.md:105 | D16（发布语义）新规则 2「checkpoint C 中产生的一切释放……」 | 改了：D16（发布语义） 立项之后「新规则 1 / 2」与「连带定死的三条」第 1 条分别成了已定项 14 / 15 / 11，全仓 30 处指向一次改齐，引文一个字没动 |
| .claude/kb/checks-owed.md:373 | （新增） | 补了：C416（去重窗口被 T_dirty 界住那条骑手没人查） 到 C419（F 生效之后靠什么保持不回落） 四笔，都是子 agent 报「全仓没有 C 编号」、主 agent 逐条现查原文坐实之后立的 |
| .claude/kb/experiments/89-嵌套区间标号与删除前沿.md:52 | 恰好是 D5（快照 / 空间记账机制）「与上一个快照比 txg」那个常数形态 | 不改：D5（快照 / 空间记账机制） 从来没有过这句话（HEAD 版也是 0 次），但这两处落在跑前写死的判据 3 里，属冻结证据 |
| .claude/rules-rationale/CLAUDE.md:1 | （整个目录） | 不改：另一个会话 2026-09-20 04:15 新建的未跟踪目录，doc-lint 报它有「## 历史版本」节，不在这一轮的射程里 |
| crates/singlefs-core/src/block_device.rs:35 | 源码注释里按 D2（RAID 条带策略） 旧小节名写「写的粒度」 | 不改：改源码要连留存产物一起改才复跑逐字一致，留给下一轮实现改动时一起做 |
