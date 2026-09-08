Do not use any markdown emphasis (no bold, no italics) anywhere in your answer. Plain text and plain headings only. Answer in English.

You are auditing a filesystem design knowledge base written in Chinese. The verbatim clauses are quoted at the end of this message; the Chinese text is authoritative, my English gloss is only a reading aid.

BACKGROUND

The project is a copy-on-write filesystem, format design stage, no code yet. Decisions are numbered D1..D27, each with numbered sub-items that are either settled or open.

D22 has a settled principle called "settled-three": any structure that is overwritten in place must carry BOTH (a) a checksum over the whole unit, and (b) a generation number that is actually checked. The reasoning: a record that was never written at all leaves the previous lap's complete and valid old record at that offset; the checksum passes, but the record is stale. So the checksum answers "is this unit complete" and the generation number answers "is it of the current generation".

D22 then classifies the journal as belonging to settled-three, and adds a hard requirement onto D16 open item 3: the generation number must be in the format, and it must actually be checked by the replay path.

D23 is a separate decision that settled the journal format in 14 sub-items, all settled. Relevant ones:
- item 4: record header is 78 bytes, constrained to a single atomic unit.
- item 8: nothing in a record can tell "which timeline" it belongs to, so the record header carries a back-chain, 32 bits wide.
- item 9: instance-id 32 bits plus counter 48 bits, two fields, 10 bytes together (this pair is called jsn).
- item 10: prev_hash covers the entire header of the previous record, including jsn.
- item 11: back-chain hash is full 32-bit CRC32C.
- item 14: the replay lower bound is given by the chosen root: only records whose (instance-id, checkpoint_txg) is strictly greater than the root's are applied; the tail is only a scan-start optimisation. Plus an explicit exception for administrator rollback, and a truncation rule when the instance table has a rollback row.
- The record header field list includes jsn (10 bytes) and checkpoint_txg (8 bytes).

D16 settled item 7: publish durability order is units/nodes, barrier, journal record, barrier, root slot with FUA; on recovery, every named unit must be verified before the record is applied.

THE QUESTION

Has D22's hard requirement on D16 open item 3 already been discharged by D23's settled items? Concretely: is the journal's generation number (i) present in the on-disk format, and (ii) actually checked by the replay path?

My tentative reading is that it has been discharged: the instance-id and checkpoint_txg are both header fields, and D23 item 14 makes the pair the comparison key that gates whether a record is applied at all.

YOUR TASK, AND ONLY THIS TASK

Try to construct a concrete counterexample: a crash-and-recovery scenario in which a stale journal record (one left over at that ring offset from an earlier lap, or from an abandoned timeline) IS applied by the replay path, even though every clause quoted below holds. Give the exact sequence: what was written, where the crash fell, which root was chosen at recovery, what the stale record's instance-id and checkpoint_txg were, and why the lower-bound filter and the back-chain both fail to reject it.

If you can construct one, state it step by step. If after honest effort you cannot, say so plainly and name the single clause that blocks every attempt you tried.

Do not summarise the background. Do not restate my tentative reading. Go straight to the construction attempt.

APPENDIX: VERBATIM CLAUSES (Chinese, authoritative)

### [A1] D22（单元原子性怎么合成）「已定三」整节（正文行 37–41）

### 已定三：原地覆写的结构必须「整单元校验和 + 被实际检查的世代号」，两样缺一不可

只有校验和不够。一条**完全没落盘**的记录，读到的是同一位置上一圈遗留的
**完整合法旧记录**——校验和通过，而它是陈旧的。
⇒ 校验和答「这个单元完不完整」，世代号答「它是不是本代的」，**两个问题，两个字段。**

### [A2] D22 正文行 122–124（journal 归入已定三，并给 D16 未定项 3 加硬要求）


⇒ **journal 落在「已定三」那一类**：它需要整单元校验和 + 被实际检查的世代号。
⚠️ D16（发布语义） 未定项 3「journal 的格式」因此多了一条硬要求：**世代号必须在格式里，且必须被重放路径实际检查。**

### [A3] D23（journal 的角色与格式）已定项 4 / 8 / 9 / 10 / 11 / 13（各一行，整行抄）

4. **记录头能不能被约束在单个原子单元内（2026-08-29）：能。头 78 字节。** **状态：已定。**
8. **记录里没有任何东西能分辨「哪条时间线」：记录头带一条反向链，宽度 32 位。** **状态：已定。**
9. **序号与实例代号的位宽划分（2026-08-30，用户定案）：实例代号 32 位 + 计数器 48 位，两个字段共 10 字节。** **状态：已定。**
10. **`prev_hash` 覆盖哪些字段（2026-08-30，用户定案）：覆盖前一条记录的整个记录头，其中含 `jsn`。** **状态：已定。**
11. **反向链的 hash 算法选哪个（2026-08-31）：取完整 32 位 CRC32C。** **状态：已定。**
13. **载荷的完整性靠什么（2026-09-01，用户定案）：取候选乙——`header_csum` 仍只覆盖头，另加一个覆盖点名项数组的校验和。** **状态：已定。**

### [A4] D23 已定项 14（重放的下界，整行抄）

14. **重放的下界（2026-09-02，用户定案）：由所选根给出——只施加 `(实例代号, checkpoint_txg)` 严格大于根的记录；tail 只是扫描起点的优化。** C113（扫描重建时多版单元的现行版本判定无输入） 定案（2026-09-05）加一条显式例外——管理员回退：从回退候选集（根环里按实例表判仍有效的根）选 R_old，不施加它之后的任何记录、取新实例代号、写回退行，第一个新根的 checkpoint_txg = 根环里全部根记录 txg 的最大值 + 1，回退与第一个新根同一次发布、之前没有持久效果；回退深度 ≤ 根环深度。同一次定案把事务按单元切分 ⇒ 崩溃原子性的单位是一个单元、不是一次写请求（仓里没有任何决策承诺过写请求级原子性，这是一次对外语义的收缩）。**失败的处置按失败的性质分两支**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 失败表，2026-09-05）。**判别子是当下判得出的量，不是次数**：失败发生时先对**目标设备的固定落点做一次探针写**——写得进去 ⇒ 瞬时失败，走**实例切换**；写不进去 ⇒ 持续失败，走**转只读到下次挂载**。可写设备数掉到 w 的下限以下、切换自己的预留拿不到，同样直接走只读支；连续切换次数越过 **N_switch = 3**（具名可调参数）也转只读，它是兜底的兜底，真正的判别子是探针写。**收敛靠的是切换的第一步就是一次会失败的写**：切换要先取号，取号要写独占打开成功的每一份超级块、全或无 ⇒「写不出去」这类故障最多让切换走一步（探针写先拦，侥幸过了全或无也立刻判失败）。此前写的「只读支不需要分配也不写单元所以必然收敛」说的是只读支自己，不是切换支怎么终止（第八轮反推腿 2.1）。切换要用的块在挂载准入时预留：**它是内存里的一个空间量**，每次挂载重算、崩溃即丢、**不落盘**——分配记录条目只有「已分配」与「已释放 + 释放代」两个态（D3（空间分配） 已定项 7），盘上没法表达第三个态，而落盘表达它本身就要写单元、回到「要写才能写」。预留量按 **N_switch × 一次切换的最坏量**（一次挂载允许 N_switch 次切换，且实例表链每切一次长一行、写行要 COW 重写整条链，第 k 次比第 1 次贵）；「一次固定点重做的最坏量」今天没有可算的口径，立成欠账。**预留挡的是分配失败那一路，不挡写失败那一路**——写失败由探针写与取号全或无挡（第八轮反推腿 4.1–4.3）。**「分配失败」按它发生在哪里拆三支**（此前把三种事搅成一格，撞了 `.claude/rules/fs-design.md` 那句「释放空间这个操作本身不需要申请空间」：池一满就整卷只读，而释放空间要写盘，重挂又被预留合取挡住，成了不可逆）——前台事务的用户数据分配失败 ⇒ **向调用者返回 ENOSPC**，checkpoint 照常发布、不动挂载状态；后台、可续做、非决策路径（重平衡、墓碑回收、scrub、整理）的分配失败 ⇒ **暂停那个活并记进意图**，不动挂载状态（`.claude/rules/fs-design.md` 第三格「无戒律」、D2（RAID 条带策略） 已定项 4 c「前台不停」）；**切换自己的预留拿不到** ⇒ 才转只读。根槽写失败重发时 checkpoint_txg 推进一格再发（根环区域 = `txg mod R`，D22（单元原子性怎么合成） 已定项 2，不推进就是反复重写同一个槽、把 R 个失败域用成一个）；⚠️ 由此「同一个根槽连续失败」不再是一条判据——槽位随 txg 轮转，N_switch ≤ R 时它永远够不着，留着它读起来像一道闸、实际是空的。**实例切换** = 挂载内做一次恢复：取新实例代号、写行 (旧实例, 最后发布的 txg, 最后施加的事务号)、重发在飞 checkpoint，号 ≤ W 的事务照旧、号 > W 的按新写序重做或向还没返回的调用者报错，固定点单元全部按新实例重写；不转只读、不等下次挂载，代价 = 一条行 + 整条实例表链重写 + 号 > W 的数据单元重写 + 一次固定点重做，频率未测。**重放下界遇回退行截断**：所选根的实例在实例表里有回退行时，该实例的记录只施加到回退行的 W 为止——否则一次落在 R_old 上的普通恢复会把被回退抛弃的那段时间线整段重放回来，管理员的回退被静默撤销，盘上没有任何痕迹（C124（回退行与重放下界没有会红的检查））。 **状态：已定。**

### [A5] D23 正文里记录头的字段与宽度（整行抄）

点名项数 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + 头部校验和 32），

### [A6] D16（发布语义）已定项 7 与未定项 3（整行抄）

7. **发布的持久顺序：单元/节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）；恢复重放施加前必须逐项验证点名单元。** **状态：已定。**（2026-09-02 用户定案）
3. **journal 的格式**——它现在是格式的一部分了，要进 D15（格式冻结政策） 的冻结分层 改第一个事务的字节：否（2026-09-01，依据：问的是 journal 在 D15（格式冻结政策） 的冻结分层里记第几层，而 journal 的字节已由 D23（journal 的角色与格式） 十一条分项定完）。 ⚠️ 复核（2026-09-02）：D23（journal 的角色与格式）当日新增已定项 14（重放下界），改的是恢复算法不是记录字节，冻结分层这一问不受影响，照旧未定。 **状态：未定。**
