# E171 跑前登记：三oracle合谋测试

写于 2026-10-03，装置写之前。


## 一、问题

英文名：three_oracle_collusion

<!-- doc-lint:not-numbers O1 O2 O3 Q1 Q2 Q3 Q4 Y1 Y3 P1 P2 P3 P4 P5 P8 P9 N1 N2 N3 N4 N5 S1 S2 S3 S4 S5 S6 S7 S8 Z1 Z2 Z3 Z4 F1 F2 F3 F4 F5 F6 F7 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 X11 X12 X13 X14 X15 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 V1 V2 W1 K4 -->

主 agent 给的问题（派发提示原文逐字）：

> 要回答的问题：照问题单四行的候选定义，一种读法。

> 要写死的（登记里交代，给执行员照做）：三条 oracle 各自「对同一批状态出结论」的入口在哪（O1 的崩溃判官与对拍、O2 的 check_pool_image、O3 读导出装置写的中性格式）；坏实现表从哪来（可以复用 crates/mutations.tsv 里改实现的那些行，挑法写死，或另立一张，二选一写依据）；坏镜像表从哪来（E170 第三段的坏状态 P1–P9 与盲测族 research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states.out 已有的那一批可以复用，与 crates/singlefs-harness/tests/checker_known_bad_images.rs 的关系写清）；O3 的执行器这一批会照 C651 判决（research/prompts/m3-c651-r1-main-verification.md，用户定逐档）改一处、严格跑法会重跑，登记写明用哪一版执行器（冻结哪一份 sha256）、O3 判「规约欠一句」的态在三条比较里怎么算；每族的阳性对照与阴性对照（正常实现、正常镜像必须三条都放行）；哪些是重型（cargo 跑 checker 档、变异表）归提交时、哪些执行员能跑。

问题单 `research/prompts/three-oracle-collusion-questions.md`（sha256 `31d831eba7ae5240e031725721278611b369d8fa9c71c4bfcd71a27a7b392b2a`）第 7–12 行逐字抄（`sed -n '7,12p'` 原样）：

| # | 问题 | 候选（各自的定义） | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| Q1 | 合谋-坏实现：一张坏实现表（每行把实现改坏一处，形态照 `crates/mutations.tsv`）在同一批历史与崩溃状态上，有没有一行三条 oracle 全放行 | **没有共同盲区**：每一行至少一条 oracle 判红。**有**：至少一行三条全放行，逐行列出 | 一行三条全放行 | 表里每一行在登记的那一批状态上三条都跑完 | 开着 |
| Q2 | 合谋-坏镜像：手造的坏镜像（每张改坏一处盘上字节）喂 O2、O3，有没有一张两条全放行 | **没有**：每一张至少一条判红。**有**：至少一张两条全放行 | 一张两条全放行 | 坏镜像表每一张两条都跑完 | 开着 |
| Q3 | 合谋-关一条：拿 Q1 里已知被抓的坏实现，逐条关掉一条 oracle 的判定，剩下的还红不红 | **都还红**：关掉任一条之后，每一行仍至少一条判红。**有一行只靠一条**：关掉某一条之后那一行全放行，逐行列出是哪一条撑着 | 关掉一条后一行全放行 | Q1 每一行三种关法都跑完 | 开着 |
| Q4 | 这道测试一次跑多久、怎么进门禁 | **能进提交门禁**：整表一次在提交门禁可接受的挂钟里跑完（门槛照 `.claude/rules/verification.md` checker 档快档的口径）。**只能进全量**：超出 | 整表挂钟 | Q1–Q3 跑完、挂钟量过 | 开着 |

问题单出处行里的岔路单那一行（Y3）与它的代价表在第二节整段抄。问题单四行就是本登记的岔路单：第六节每个量写明它对应哪一行、取什么值让那一行翻面。

### 读法写死

1. **三条 oracle 与各自的入口**（「对同一批状态出结论」那一句的落点；行号在第三节）：
   - **O1** 分两样，分列报、合成一列判：**O1-对拍**——每次 API 调用之后拿模型的答案与实现的结局比，入口 `IdealModel::answer_*` 加 `IdealModel::judge_and_advance`（与 `crates/singlefs-harness/src/history.rs` 的 `judge_by_model` 同一种比法，观测由 `crates/singlefs-harness/src/model_comparison.rs` 的胶水造，不另写比法）；**O1-崩溃判官**——每个崩溃状态上，实现只读恢复（看 journal）读回的那一版交给 `crash_recovery_disagreement`，模型那一侧给每一步之后并起来的 `committed_versions`，「盘上已持久的最新根槽」由 `newest_persisted_root` 从截断的写流数出（与 `crates/singlefs-checker-tier/src/crash_injection.rs` 调它的那几行同一种交法）。O1 在一行上算红 = 两样里任一样判出一处对不上。
   - **O2**：池级 checker `check_pool_image` 判这个崩溃状态的崩溃镜像（恢复之前那一份，与层 0 的 `judge_crash_image` 判的是同一份），任一条不变量 `Violated` 算红；`NotApplicable` 不算红。
   - **O3**：E170（O3规约执行器第一版） 的 Python 执行器读本实验装置按 E170 登记 5.3 写出的中性格式，一态出两个判定：严格跑法（「在 / 不在 / 规约欠一句」）与暂代跑法（「在 / 不在」）。执行器是哪一版见第 8 条。
   - 记录核对器（`check_records`）与层 0 照手写版本表判的 oracle（`crates/singlefs-checker-tier/src/crash.rs` 的 `classified_oracle_violation_for_versions`）**不是三条之一**：D13（验证路线） 已定项 7「故意不给它编号」，层 0 那份 oracle 的版本表由调用方手写、不是 `model.rs`（`model.rs` 第 2246 行上方注释原文「与层 0 的 `crash::oracle_violation_for_versions` 分开写」）。两样照样算、另列（附带），不进 Q1–Q3 的判定；「三条全放行而它们红」的行单列出来交主 agent。
   - D13 已定项 6 表的 O1 行今天写着「无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3）」：崩溃判官判的正是崩溃之后的允许集合。本登记照派发提示把崩溃判官算进 O1；另用同一张矩阵把「O1 只取对拍」那一种重算一遍（附带，第六节 Q1-丙），两种结论都交主 agent。
2. **同一批历史与崩溃状态**：历史取 E170 那两条层 0 流——新池新建文件（流一，流名 `new_pool_file_creation`）与固定脚本到 E 再正常卸载（流二，`fixed_script_through_e_then_unmount`），驱动照 E170 导出装置的同名驱动。崩溃状态取 E170 登记 5.4 的批 1 取样规则（每段八个点、全持久态、每段至多 4 个带撕裂的态）作用在**这一行自己录下的写流**上：坏实现改了写流，状态的字节随之变，「同一批」指同一段历史、同一条取样规则，不指同一份字节。批 2（E170 5.4 的段内居中点）是几何取样点（第八节）。
   - 为什么取这两条流、不取崩溃注入的随机历史：O3 只在这两条流上跑过、导出格式只为它们写过；崩溃注入的历史里有冷启动、崩溃恢复抛弃根这类 O3 从没见过的调用，O3 在那里报「不在」分不清是执行器没写到还是坏实现被抓（推的，没量）。
3. **一行的红**：一行（一个坏实现）在 oracle X 上算红，当且仅当 X 在这一行两条流的批 1 里至少判红一处（O1-对拍按调用算，其余按状态算）。批 2 另判一遍，只进第八节。
4. **触达**：一行算触达，当且仅当下面三样里至少一样与基线（不改实现那一趟）不同：两条流任一条的 `ops.tsv`、`ops-payload.bin`、`calls.tsv`、`base-sectors.bin` 的 sha256；任一次调用的结局（成功 / 哪一种错误）；写流全同时，批 1 任一态上看 journal 那一遍恢复的报告（`effective_root`、`outcome` 的种类与读回内容的 sha256、施加的记录条数）。三样都相同的行叫**未触达**：它在这一批上与正常实现逐字节同行为，哪条 oracle 都不该红，也不进 Q1 的分母。只在不看 journal 那一遍恢复上不同的行另记「只在不看 journal 那一遍不同」，不进分母（那一遍是层 0 的诊断读法，不是实现交给用户的恢复）。
5. **不进分母的另几类**（逐行列出、照数报，不删）：替换之后编不过（`.claude/rules/mutation-sampling.md` 第八类，按 5.2 的补位规则换行）；锚点在源码里不唯一（第七类，同样补位）；表里按名字找不到这一行（同样补位）；**实现自崩**——驱动一次调用或某一态的看 journal 恢复里 panic（panic 不是悄无声息的错，不考 oracle 的独立性）；单行超过 1800 秒（`.claude/gate.d/checker-tier-crates-mutation-replay.sh` 第 46 行同一个默认限时）记超时。一次调用返回错误而模型说该成功，不算自崩，是 O1-对拍的红。
6. **O1 够不到的地方**：流二最后一次调用 `unmount`（卸载时抬 F 那两次发布）模型没有对应的答（`model.rs` 里没有 `answer_unmount` 一族），这次调用不对拍，任一写落在这次调用里的状态 O1-崩溃判官记「不适用」；一次对拍判出不同之后模型不再往前走，此后的调用与写落在此后的状态同样记「不适用」。不适用算不红，个数照报。除这两种来源之外出现不适用，走停机 S5。
7. **「规约欠一句」怎么算**：严格跑法判「规约欠一句」的态算 **O3 弃权，不红**——它没有判这个态不在允许集合里。Q1、Q3 各按两种 O3 判一遍、各占一行：**甲**取 O3 严格跑法（欠一句 = 不红），**乙**取 O3 暂代跑法（E170 登记 5.9 写的进门禁那一种跑法就是暂代跑法）。乙红而甲不红的行标「只靠暂代读法撑着」。执行器对一态抛出「error」（`executor_main.py` 第 45 行那一种）算不红、单列；执行器报底镜像恢复不出（`executor_core.py` 第 74 行 `BaseImageNotRecoverable`，它自己的说明是「这条流不判」）时，这一行这条流 O3 两种跑法都记弃权、不红，行上标「O3 底镜像恢复不出」。
8. **O3 用哪一版**：E170 照 C651 判决（用户定逐档：V1 逐档、V2 世代号最大同号取偏移小）改过执行器、严格跑法重跑时冻结的那一版——E170 那一次产物里记的冻结逐文件 sha256（文件集合照 E170 登记 5.6：执行器目录里的源文件，不含 `comparator*` 与 `mutation_driver*`）。执行员在装置写之前把那几行 sha256 连同产物路径抄进第十二节修订（只抄冻结的 sha256 行，不读那份产物的结论行），开跑前与最后一次跑 O3 之后各 `sha256sum -c` 一次。今天（C651 改之前）的逐文件 sha256 记在第三节，**不用它**：一行一个版本，不混。那一版还没有冻结行时 O3 那几列不跑（停机 S7）。
9. **关一条（Q3）**：三条 oracle 的判定互不吃对方的输出（各自只读镜像、写流或实现的恢复报告），所以「关掉 X 之后剩下的红不红」由每行三列的红 / 不红直接算：关掉 X = 把 X 那一列当不红。这条读法靠一次实测撑着：头三行被抓的坏实现，在装置上真的逐条关掉一条 oracle 重跑，其余两列要与不关时逐字节相同（第六节 Q3-核，失败条款 F5）。
10. **Q4 的门槛**：`.claude/rules/verification.md`「checker 档自己分快档与全量」与 D13 已定项 15 都没给快档一个挂钟数（`grep` 结果在第十三节）。**Q4-甲**——照 E170 的取法，取 D13 已定项 15 唯一的时长数「debug 下单条跑到 60 秒及以上的是重」，整表一次 ≤ 60 秒算能进（问题单 Q4 写「门槛照 checker 档快档的口径」；主 agent 2026-10-02 在装置写之前删掉了取每行 1800 秒的那一种读法）。
11. **坏镜像那一族只到两条**：O1 不碰盘（D13 已定项 6 表 O1 行「不碰盘、不解析磁盘格式」），Q2 只判 O2、O3；「正常镜像三条都放行」在这一族读作 O2、O3 都放行。实现的恢复在坏镜像上读回什么交给 O1-崩溃判官也算得出，附带，够判后不跑。

## 二、被测条款与它引的定义

用 `research/scripts/quote-kb.py` 整段抄（出口 `/tmp/claude-1000/collusion-designer/quotes.md`，回读逐字节一致），七段：D13（验证路线） 已定项索引表第 6 行、已定项 6 全节、已定项 7 全节（记录核对器不是三条之一的依据）、已定项 15 全节（Q4 的档）、`checks-owed.md` 的 C407 一行、岔路单 Y3 一行、岔路单第二节 Y3 代价表。E170 登记 5.3（导出格式）、5.5（坏状态表）、5.6（两种跑法与冻结）是本实验复用的定义，不是 kb，不抄，引路径与 sha256（第十三节）。

**出处 `.claude/kb/decisions/13-验证路线.md:14-14`（整段抄，未转述）**

```markdown
| 6 | **三个互相独立的 oracle** | 内存模型对拍、独立解析器 + checker、独立规约执行器三个 oracle 各判一样；独立性要靠「三 oracle 合谋测试」验证，不许自称 **状态：已定。** |
```

**出处 `.claude/kb/decisions/13-验证路线.md:130-155`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义** |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
本工程功能正确性的全部故事就是这三个 oracle。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 今天的形态是 `crates/singlefs-harness/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 harness-model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 的第一版是只照 kb 文字写的 Python 执行器 `research/e7-index-bench/src/bin/e170_o3_spec_executor/`（E170（O3规约执行器第一版））：严格跑法撞「规约欠一句」判不出，暂代跑法与实现在 972 个取样态上一致（两边全放行，不当它判得对的证据），阳性对照照字面没有判别力（26 行判「在」4 行，原因在跑前登记的读法与那一行的期望之间），盲测 40 行全判「不在」，还没接进崩溃点重放；RefFS（已定项 3）的第一版在 `crates/singlefs-reference-filesystem`，调用点没接；「三 oracle 合谋测试」没有（`grep -rn 合谋 crates` 零命中）。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够进快档；严格跑法因规约欠一句判不出、阳性对照照字面判别力不足，这两样它撑不住。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性要靠「三 oracle 合谋测试」观测，那个测试还没有。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C446（未实现的不变量没有前置登记，也没有到期）；C447（独立规约执行器与 RefFS 都零代码）；「三 oracle 合谋测试」没有 C 编号（C407（三 oracle 合谋测试没有代码） 记着它零代码，那一格本身仍没有编号）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:156-174`（整段抄，未转述）**

```markdown
#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像

**定案**：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`——持久集合是枚举器给出的这个崩溃状态里哪些写落了盘（层 0 的段内整写子集）；只看盘上字节分不开「写没落、而原地内容恰好相同」与「写落了」，判复用与缺席要拿持久集合判；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-checker-tier/src/crash.rs`，接在层 0 的每个崩溃状态上（门禁 checker-tier-layer0-replay）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 层 0 不覆盖可写挂载：层 0 在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在层 0 的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-checker-tier/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

**依据**：
- 无实验：**依据是类型，不是成本**：「逐字节比对」是二元关系 `R(镜像₁, 镜像₂)`，**没有任何投入能把一元谓词变成二元关系**；I-4.5（已停用·需两镜像） 因同一条门槛退役。
- 「崩溃后镜像」是两份：随 D16（发布语义） 已定项 4 收窄问法那一轮三方论证（材料 `_d16-items134-background.md`）由用户定案，记在变更史。
- 持久集合那一样入参：用户定案，原话在变更史；判别子观测不到的实据在 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第二节按扇区豁免那一行。
- 层 0 不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」），记在变更史；崩溃注入那一侧的实现在 `research/prompts/m2-rev-b3b-implementer-report.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:354-366`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码按「什么时候跑」分两档，各住一个包，与池级 checker 一共三个名字：**harness 档**是 `crates/singlefs-harness`，单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），改了代码随时跑；包内再分轻重：debug 下单条跑到 60 秒及以上的是重，标 `#[ignore]`，单条用时看平常跑的时候量到的、不另外单独量，要跑随时跑，一律经内存包装；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。**checker 档**是 `crates/singlefs-checker-tier`，崩溃态枚举引擎与记录核对器、断点续跑与双机分片、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例，连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档里的崩溃点重放（门禁 checker-tier-layer0-replay）分两档，checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 不分：**快档**是 `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（装置二进制的内联单测不在快档里）（门禁 checker-tier-layer0-replay 在整轮门禁与提交时都跑它），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（checker-tier-layer0-replay `--full`），提交时不默认跑，由用户要求时跑，跑法按已定项 9 分层；GPU 只接校验和那一截（E163（GPU多卡算单元校验和））、要不要接归 D24（后台重活能不能卸给 GPU）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被两档调用。依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ④ 条判，dev-dependencies 也算）；两档都要的一小段代码宁可各留一份。崩溃枚举用例一律写在 `crates/singlefs-checker-tier/tests/` 里（`research/scripts/crash-case-check.py` 判），重型测试闸按包判：跑到 `singlefs-checker-tier` 的测试就是 checker 档。

**射程**：管 `crates/` 下代码与测试住哪个包、什么时候跑、闸按什么认、三个包怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants.md](../invariants.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑、住哪个包是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；三方 `verification-split-r1`（判决 `research/prompts/verification-split-r1-main-verification.md`）攻的是拆出 `singlefs-checker-tier` 之前的那一版，拆包之后的改动被攻过零轮。
- 用户定案（2026-10-01）：全量不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/checks-owed.md:331-331`（整段抄，未转述）**

```markdown
| C407 | 三 oracle 合谋测试没有代码 | **D13（验证路线） 已定项 6 逐字要求「独立性要被验证，不许自称」：设计一个「三 oracle 合谋测试」，拿一个精心构造的坏实现（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。** 现查 `grep -rn 合谋 crates .claude/gate.d` 零命中，D13（验证路线） 已定项 6 的欠块逐字写着这个测试「没有 C 编号」⇒ 三条 oracle 的独立性至今只是声明，而 `CLAUDE.md` 把「没有 oracle」列为本工程第一条特殊性，功能正确性的全部故事就押在这三条上 | 合谋测试做成门禁阶段：一份已知坏镜像（或坏实现）喂给三条 oracle，至少一条必须红；三条全绿即判红并打印哪三条都放行了。判别力自证：同一个阶段对正常镜像必须转绿（阴性对照）；把其中一条 oracle 的判定关掉，必须仍有别的腿红，不红就说明共同盲区正在那里 | 三条 oracle 要都有代码：O2（独立解析器 + checker） 在 `crates/singlefs-checker`、O1（内存模型对拍（HashMap 理想文件系统）） 在 `crates/singlefs-harness/src/model.rs`（门禁 harness-model-differential-and-scenarios），O3（独立规约执行器） 零代码（C447（独立规约执行器与 RefFS 都零代码） 记着） | 2026-09-19 第二批 D13（验证路线） 瘦身时子 agent 指出，主 agent 现查 D13（验证路线） 已定项 6 正文与欠块、`grep -rn 合谋 crates .claude/gate.d` 零命中坐实 |
```

**出处 `research/prompts/m3-formal-7-forks.md:17-17`（整段抄，未转述）**

```markdown
| Y3 | 三 oracle 合谋测试怎么接 | 三个 oracle 取 O1（`crates/singlefs-harness/src/model.rs` 对拍）、O2（`crates/singlefs-checker`）、O3（Y1 选定那一份）。**合谋-坏实现**：登记一张坏实现表（形态照 `crates/mutations.tsv`），每行把实现改坏一处，在同一批历史与崩溃状态上三条 oracle 都跑，三条全放行即红。**合谋-坏镜像**：手造坏镜像喂 O2、O3，全放行即红（O1 不读盘，这一族只到两条）。**合谋-关一条**：拿已知被抓的坏实现，逐条关掉一条 oracle 的判定，剩下的仍要红（C407 判别力自证那一句） | 坏实现表里出现一行三条全放行（共同盲区）；或某一族根本到不了三条 | 坏实现表头几行在三条 oracle 上各跑一遍的挂钟；O3 有代码 | 开着（依赖 Y1） |
```

**出处 `research/prompts/m3-formal-7-forks.md:44-51`（整段抄，未转述）**

```markdown
### Y3 三 oracle 合谋测试

| 候选 | 另写多少行 | 动哪些文件 | 新工具或新语言 | 一次跑多久 | 罩住 | 罩不住 |
|---|---|---|---|---|---|---|
| 合谋-坏实现 | 推：门禁一道加驱动 300–800 行（参照 `checker-tier-formal-verification.sh` 147 行加判法 `research/scripts/verus-proof-check.py` 1 434 行、变异跑道 `research/scripts/mutate.sh` 1 173 行），坏实现表每行一条 | 新坏实现表、新门禁一道、`stage-inputs.tsv`；三条 oracle 各要一个「对同一批状态出结论」的入口 | 不要（O3 的语言照 Y1） | 推：每行一次冷编译加三条 oracle 各跑一遍；参照 E165 第二段 26 个变体 `--jobs 4 --threads 4` 整段 32 分钟（E165 页「复跑」一节，别的会话量的），折每变体约 5 分钟 | 三条一起放过的坏实现 | 表里没登记的坏法；D13 已定项 6 举的「nonce 永远为 0」今天造不出：加密关着，I-6.1–I-6.9 状态列都是「未实现」，checker 本来就判 nonce 全 0（`crates/singlefs-checker/src/image.rs` 第 696–699 行） |
| 合谋-坏镜像 | 推：100–300 行；参照坏镜像用例 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 5 442 行（O2 一家的） | 同上 | 不要 | 推：秒级 | 坏镜像上 O2、O3 的共同盲区 | O1（不读盘） |
| 合谋-关一条 | 推：在坏实现那一道上加开关，50–150 行 | 同合谋-坏实现 | 不要 | 推：合谋-坏实现的三倍 | C407 判别力自证那一句 | 同合谋-坏实现 |

```

## 三、实现今天的样子

行号是 2026-10-03 现查的（`grep -n`，命令在第十三节）；别的会话正在改这几份文件（`crates/singlefs-checker/src/walk.rs` 的 `check_pool_image` 在 E170 登记里是第 5737 行，今天是第 6487 行），执行员开工时再查一遍。

- **合谋测试零代码**：`grep -rln '合谋\|collusion' crates .claude/gate.d --include=*.rs --include=*.sh --include=*.tsv` 只命中 `.claude/gate.d/checker-independence-and-sync.sh` 第 40 行一句注释（「归 D13（验证路线） 已定项 6 的合谋测试」）。
- **O1**：`crates/singlefs-harness/src/model.rs`——`IdealModel::after_make_filesystem` 第 731 行、`acquire_and_warm_up_in_the_make_filesystem_process` 第 778 行、`close_session` 第 797 行、`committed_versions` 第 811 行、`answer_publish_first_file` 第 1107 行、`answer_publish_overwrite` 第 1149 行、`answer_mount_writable` 第 1252 行、`answer_rollback_while_mounted` 第 1363 行、`answer_raise_rollback_floor` 第 1544 行、`judge_and_advance` 第 1694 行、`crash_recovery_disagreement` 第 2246 行（四条判据：走读不许失败、择到的根要是模型提交过的某一版、不许比盘上已持久的最新根槽旧、读回内容与那一版逐字节相同）。没有卸载的答（`grep -n 'unmount' crates/singlefs-harness/src/model.rs` 零命中）。对拍的比法：`crates/singlefs-harness/src/history.rs` 第 911 行 `judge_by_model`；崩溃判官的交法：`crates/singlefs-checker-tier/src/crash_injection.rs` 第 994 行（`newest_persisted_root` → `model_root_key`）与第 1018 行（`crash_recovery_disagreement(&committed_versions, newest_persisted, &read_back)`），`newest_persisted_root` 在 `crates/singlefs-harness/src/memory_pool.rs` 第 812 行，`observed_read_back_after_a_crash` 在 `crates/singlefs-harness/src/model_comparison.rs` 第 940 行。
- **O2**：`crates/singlefs-checker/src/walk.rs` 第 6487 行 `check_pool_image(reader: &dyn ImageReader)`，交回每条不变量的 `InvariantVerdict`（`Holds` / `Violated` / `NotApplicable`）。层 0 的 `crates/singlefs-checker-tier/src/crash.rs` 第 1222 行 `judge_crash_image` 在崩溃镜像上调它（同一函数里还有两种 journal 政策的恢复、手写版本表的 oracle 第 971 行、记录核对器第 51 行 `check_records`）；第 1267 行 `judge_layer0_state_range` 按层 0 序号造 `CrashImage` 再判。实现的恢复入口 `crates/singlefs-core/src/recovery.rs` 第 4330 行 `recover`。
- **O3**：`research/e7-index-bench/src/bin/e170_o3_spec_executor/`。驱动 `executor_main.py` 第 213 行 `main`（`--dumps --streams --batches --workers --output`），一态同时出严格与暂代两个判定（`executor_core.py` 第 105 行 `strict = VERDICT_SPEC_GAP if gaps.identifiers else provisional`），冻结范围不含 `comparator*` 与 `mutation_driver*`（`executor_main.py` 第 29 行）；判坏状态由 `comparator_q3_bad_states.py` 的「判的那一半」在原样建好的镜像上叠 `bad-patches.bin` 的补丁再调 `executor_core.judge_state`（文件头第 1–9 行）。欠一句表 `spec-gaps.tsv`（`grep -c .` 21 行，含表头）。
- **今天（C651 改之前）执行器目录的逐文件 sha256**，只作对照、不用（第一节读法 8）：

```
7837e04a4967870ee02091943a97bd453c19a30e59d4448ce794866c7cbc97e2  comparator_mutation_checks.py
6a95449c49515a0edc07fb3c58aa496d6a255d4cf0deac94cda76b85cabc4b5c  comparator_q2_implementation_verdicts.py
8d3e68c0405ff3674d804a193fbaa16d7a20efbdc52819dbb878795ee69d0cab  comparator_q3_bad_states.py
65074f9566cad5faae4a868e397aa7bf1ebbfbb8ede52b88b57b6860a7c9a784  comparator_result_lines.py
7a671b85d8346f2aee45982a888e2111e98495f5570ecfc57a348ee7595d30dd  crash_dump.py
e391dbe8377014258f1978005062d4cb14891b8565ca420a6a5b731ad4387a5f  crash_image.py
34b9f571bc7d0e9dc32dce56b166536a6c26581b9f4744e7c3a14b0ef69bea6d  crc32c_castagnoli.py
2500ccde5cfa5e1f4dbf7ceb9476b00da7c857a58af2d9e1dcac9bd207eb76d9  executor_core.py
de96a31db1b90c0dab3f7ab318f5d0998e9c096069e0debfee3e3830be08c603  executor_main.py
9e0b1bd1fc73a9d7917470f38c66d17026ef150dc32a07d7d250e271ddceb744  journal_ring.py
1b24867516160e22cf1c36ce1d12fed209135acac99605577527cff8e038ce02  mutation_driver.py
5fd06bb04719b5b6999c504bdd91aacd6ff687d583e7bf0e6f2a749e0be824aa  on_disk_layout.py
ff2f52d4f93910a88dbab37bfeafaecc9d7b58fe0e7c139923e6de771bbff9da  replay_prefix.py
0daf6a77a81266abbd43c4a7e01f0c8b5de66d5116fdfa2480a973a3e21bd908  root_ring.py
b3b1b6926fc6e1d1719f6b732c2c74b6af9a9f15ce3469c9316b664fb8bc171c  spec_gaps.py
ffd6abf288af89dfa25b4ccd1b69714f71511e5a5e606f787943406fe7551f69  synthetic_pool_for_tests.py
5763d601370cecf9ac783ac9e522c84d95c8d3eca4e769035f9f5bec4e1818b6  system_configuration.py
88d5a2350311f5b2ea120db04fa4ec9541b01ba5db497e972ae0ee30950c6055  test_executor_on_synthetic_pool.py
a743c37809782d1775d815d0df6f3bd1ad48c4807e0b5a0ce201b0deacd51ec3  version_table.py
994d480338d43c27abeae0a137b9723c6dfcf025ef7e335e50c760796bed3b22  spec-gaps.tsv
```

- **导出装置**：`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`（3 742 行，sha256 `8c5262935b45ebf119fd3f0866f6451685864b4e50fb25520d7f58942e63fb64`）。两种模式（文件头第 15–16 行）：`dump --output-directory <目录> [--compare-with <目录>]` 与 `bad-states --dump-directory <目录> --seed-hex <16 个十六进制字符>`；流一驱动第 253 行、流二驱动第 387 行（`calls.tsv` 的调用名 `acquire`、`warm_up`、`publish`、`mount_writable`、`roll_back`、`raise_floor`、`unmount`）、每次调用记返回点第 210 行 `record_call`、取样第 696 行、导出一条流第 1269 行。驱动里的 API 调用一律 `.expect(…)`：坏实现一次调用出错，这个装置当场 panic。决定几何与工作量的本地常量第 78–140 行（流一 41 次写、3 次根槽写，流二 477 次写、19 次根槽写，两条流的段长表与全量 / 快档状态数），`main` 开头与 crates 回比，不等退 3。
- **坏实现的来源**：`crates/mutations.tsv`（1 967 行，sha256 `40c75af965ea7e63d9bd32356dc3d290bb99616f5a9aa1ddd1bd703aa28da30b`，`git status` 显示 `MM`：别的会话正在加行）。每行六段：变异名、文件、原文、替换文、cargo test 参数、必须红的测试名；原文在文件里必须恰好命中一次。文件列在 `crates/singlefs-core/src/` 下或等于 `crates/singlefs-replay-prefix/src/lib.rs` 的行 844 行，分在 24 个文件里。
- **已知坏镜像用例**：`crates/singlefs-harness/tests/checker_known_bad_images.rs`（sha256 `e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820`）。第 747 行 `known_bad_images(clean)` 交回一串（目标不变量名，改坏的闭包），第 2815 行用的干净镜像是 `build_pool("known-bad").memory_pool()`（`crates/singlefs-harness/tests/common/mod.rs` 第 677 行 `build_pool`，就是新池新建文件那条流走完的池），第 2814 行起的用例断言干净镜像上每条不变量不红、每一条改坏都让它的目标不变量 `Violated`。它是 O2 自己的回归语料：每一份都是照 O2 的判据造来让 O2 红的。其余几族（`known_bad_images_after_the_overwrite` 第 1319 行起等）各自在别的历史上的干净镜像上改，不在本实验的两条流上。
- **RefFS**：`crates/singlefs-reference-filesystem/src/lib.rs` 在，调用点由第 12 批 12-2 在接；本实验不用它（派发提示「不依赖 RefFS 接调用点」）。

## 四、跑之前已经存在的数

设计时读到或自己算出的，照实列：

1. **被测条款里已经写着 E170 的结论**：第二节抄的 D13 已定项 6「实现今天的样子」一句（这一段是第 12 批写回刚改的，我开工时读的那一版还写着「O3 零代码」，抄条款时才读到新的一版）：「严格跑法撞「规约欠一句」判不出，暂代跑法与实现在 972 个取样态上一致（两边全放行，不当它判得对的证据），阳性对照照字面没有判别力（26 行判「在」4 行，原因在跑前登记的读法与那一行的期望之间），盲测 40 行全判「不在」」，依据行「一态够进快档」。影响：① O3 严格跑法在 E170 那一版上可能一态都判不出——第一节读法 7 把 Q1、Q3 拆成甲（严格）乙（暂代）两行，这个拆法是读到这句之前、看到 `spec-gaps.tsv` 第 2 行 G01（三处 magic 在 kb 里没有取值）时定的，读到之后没改；失败条款 F7 是读到之后补的，它只让「O3 严格一行都判不出」时 Q1-甲改写成「某一族到不了三条」，不改任何门槛。② P 行 26 行里 O3 判「在」4 行：Q2 在 P 行上很可能出现「O3 放行」的行，Q2 翻不翻面取决于 O2 在那几行上红不红；Q2 的门槛（一张两条全放行即翻面）是问题单的定义，不从这个数来。坏镜像表另加了照 O2 判据造的那一族（5.5），这一族是读 `checker_known_bad_images.rs` 时定的，在读到这句之前。③ 那是 C651 改之前那一版的数；本实验用改之后的那一版（读法 8），数可能变。
2. **岔路单第二节代价表**（全文读了，Y3 一节整段抄在第二节）：E165 第二段 26 个变体 `--jobs 4 --threads 4` 整段 32 分钟、折每变体约 5 分钟（别的会话量的）；纯 Python CRC-32C 4 KiB 一块 0.299–0.320 毫秒（量的）；Python 每态 1–10 毫秒、层 0 D 全域单线程 14–144 小时（推的）；Y5、Y6 各行的行数与挂钟。影响：Q4 的两种门槛不从这些数来；按「每行一次冷编译、几分钟」推，Q4-甲（整表 ≤ 60 秒）几乎一定不过（推的，没量），所以第五节把整表交给主 agent 跑。
3. **E170 登记**（读了第一节、第三至十二节；结果与结论在实验页与产物里，没读）：第四节列的提交标题（崩溃放量七个分项 26.16 亿态零红、展开 28 全枚举 26.15 亿态零红到 E 全域 1 662 648 564 态、固定脚本到 D 全域 51773558 态零违例）；导出装置本地常量（流一 41 次写、3 次根槽写、全量 16 777 260 态、快档 46 态；流二 477 次写、19 次根槽写、全量 1 662 648 564 态、快档 278 态；坏状态 26 + 10 + 40 行）。影响：预期正常实现在这两条流的层 0 状态上 O2 不红——阴性对照的门槛（0）是问题单的定义（「正常实现、正常镜像必须三条都放行」），不从这些数来；坏状态行数进第七节 A9 的锚点（出自 E170 登记 5.5 的表，用命令数过）。
4. **C651 判决**（全文读了）：C 组 12 态、S 组 240 态各读法的择对 / 拒数，W1 mkfs 清零，V1 逐档、V2 世代号最大同号取偏移小，O3 执行器暂代读法「收到第一个就停、不比世代号」。影响：只用来定 O3 用哪一版（读法 8）；两条流的 io_min 都是 512（E170 导出装置第 83 行），C651 那一处读法只在「全池没有可用的槽 0」的态上起作用，本实验的取样里有没有这种态没数。
5. **批次记录第 7 批起各节**（读了第 12 批全文与第 7 批开头；第 7–11 批的表格只看了标题行）：11-11 补丁「改动的测试 5 passed」、W1 让 mkfs 在第一道 barrier 之前每盘多一步写零（23 → 25 步）。影响：E170 导出装置第 78–140 行的本地常量可能已与今天的 crates 对不上（W1 改的是 mkfs），停机 S1、S6 为此而设；不影响任何门槛。
6. **执行器的欠一句表** `spec-gaps.tsv`：读了表头与前两行（G01 三处 magic 在 kb 里只写「常量」、G02 系统配置字段表），`grep -c .` 21 行。影响：见第 1 条。
7. **自己算的**：`crates/mutations.tsv`（sha256 `40c75af9…`）里改实现的行 844 行、24 个文件；步长 8 的挑法 117 行、步长 4 的 220 行、两张重叠 56 行、步长 4 独有 164 行、每个文件取第一行 24 行；`known_bad_images(clean)` 41 条（38 条字面名、3 条常量名）。这几个数进第七节当独立算出的锚点（A4–A8），不进任何门槛。门槛「触达行不少于 30」来自「30 行全被抓时，每行全放行的概率 95% 上界约 3/30 = 10%」（三法则，第六节），不来自这些数。
8. **快档没有挂钟数**：`.claude/rules/verification.md`「checker 档自己分快档与全量」整节、D13 已定项 15 全节读了，唯一的时长数是 harness 档「debug 下单条跑到 60 秒及以上的是重」；`.claude/gate.d/checker-tier-crates-mutation-replay.sh` 第 46–48 行每条变异默认限时 1800 秒（依据「16 个工作进程并发时一条超过 6–8 分钟」）。影响：Q4 两种读法的门槛就是这两个数（第一节读法 10）。

## 五、臂、阳性对照、真实基线

### 5.1 臂

| 行 | 一边 | 另一边（写成支持它的人认的样子） |
|---|---|---|
| Q1 | **没有共同盲区**：触达的每一行至少一条 oracle 判红 | **有**：至少一行三条全放行。支持它的人会要求：表不是照三条 oracle 的长处挑的（5.2 的挑法是按文件机械取样，不看哪条变异会被谁抓）；「全放行」按最弱的 O3 算（甲行把欠一句当不红）；不许事后把一行全放行的坏实现说成「其实没坏」从分母里删掉——一行算不算真坏交主 agent 判，本实验照数列出 |
| Q2 | **没有**：每张坏镜像 O2、O3 至少一条判红 | **有**：至少一张两条全放行。支持它的人会要求：表里不能只有照 O3 判据造的（E170 的 P 行与盲测族），也要有照 O2 判据造的（已知坏镜像那一族），两边各自的长处都考到；P 行期望「不在」是 E170 设计员照条款推的、被攻过零轮，两条全放行的 P 行要同时报「期望可能推错」 |
| Q3 | **都还红**：被抓的每一行，关掉任一条之后仍至少一条红 | **有一行只靠一条**：关掉某一条之后全放行。支持它的人会要求：关法是真的关掉判定、不是从矩阵里假设关掉——所以读法 9 要一次实测撑着（Q3-核）；甲乙两种 O3 各判一遍 |
| Q4 | **能进提交门禁**（门槛照文件头删剩的那一种读法） | **只能进全量**。支持它的人会要求：挂钟在被测进程里计（装置用 `Instant`、执行器用 `perf_counter_ns`、驱动在子进程起止两点计，不按输出行到达的时刻切），冷编译与跑分开报，单线道与多线道各报 |

**真实基线**：不改实现的那一趟（下称基线）——同两条流、同取样规则，三条 oracle 各判一遍。它同时是 Q1、Q3 的阴性对照：基线上三条 oracle 一处都不许红（问题单原文「正常实现、正常镜像必须三条都放行」）。

### 5.2 坏实现表：复用 `crates/mutations.tsv`，挑法写死

**二选一取复用**，依据：① 表里每一行都是「把实现改坏一处」、形态就是问题单 Q1 写的「形态照 `crates/mutations.tsv`」；② 锚点由门禁 code-source-discipline 的 mutation-tables 格逐条核唯一命中，不会悄悄腐化；③ 另立一张要由我挑坏法，而我读过三条 oracle 的代码，挑出来的坏法会带着我对三条盲区的猜测——机械取样不带。代价：D13 已定项 6 举的「精心构造」那一种（例：nonce 永远为 0）不在表里；那一例今天造不出（岔路单第二节 Y3 代价表「罩不住」列：加密关着），本实验不补。

**挑法**（脚本 `/tmp/claude-1000/collusion-designer/select_bad_implementations.py`，sha256 `4032a0e06b614a66c67df5fe5eae82e15f3399177655ee4230e50aee4b7785b4`，执行员照抄进 `research/e7-index-bench/src/bin/e171_three_oracle_collusion/select_bad_implementations.py`）：取 `crates/mutations.tsv` 里文件列以 `crates/singlefs-core/src/` 起头、或等于 `crates/singlefs-replay-prefix/src/lib.rs` 的行（注释行与不是六段的行不算）；`singlefs-format` 不取——它是 checker 与实现共用的常量（D13 已定项 5），改它同时改 O2，不是「坏实现」；harness、checker、checker-tier 的行改的是验证一侧，不取。按文件名字典序分组，组内保持表里的先后；一组 n 行取 m = ⌈n/8⌉ 行，组内下标 ⌊i·n/m⌋（i = 0..m−1，0 起）。在 sha256 `40c75af9…` 那一版上得 117 行（逐行在 5.2 末尾的表里，**以这张表为准**，执行员开工时表若已变，照名字取行，不重挑）。

- **第一刀**：每个文件 i = 0 那一行，24 行（表里「刀」列标 1）。
- **补位**：一行编不过、锚点不唯一、或按名字找不到，换同一文件组内下标 j + 1（到组尾回到 0）、不在本表里的那一行；同一行至多补三次，补不上照实报。补位换上的行在产物里带「补位自 <原名>」。
- **扩表**（失败条款 F4、F6 触发时）：步长 4 的挑法多出的 164 行（与步长 8 重叠 56 行不重跑），按同一张脚本的输出顺序跑。扩表与补位都不重挑已跑过的行。
- **行文变了**：开工时某一行的原文或替换文与 sha256 `40c75af9…` 那一版不同（别的会话修了锚点），照今天的行跑，产物里记两份行文的 sha256。
- **触发顺序**：先第一刀 24 行，再其余 93 行。

### 5.3 装置写在哪

- **入库装置**（这条岔路问的是 `crates/` 今天的 O1、O2 与实现本身在同一批状态上一起放不放过坏实现；另写一份独立模型答不了）：`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`，只读、只驱动、只观测，不改 core、checker、harness、format、replay-prefix。
- **照抄 E170 导出装置的两条流驱动、取样与 5.3 导出写出代码**（照抄，不 `#[path]` 引那个 bin 文件），改三处：① 驱动里的 API 调用不 `.expect`，结局（成功 / 错误成员）记进 `calls.tsv` 的 `ok` / `err` 列与本装置的 `call-outcomes.tsv`，出错之后这条流不再往下驱动；调用与每一态的恢复包在 `std::panic::catch_unwind` 里，panic 记「实现自崩」；② 跟着每次调用推模型（下表），记 O1-对拍；③ 每个取样态算 O1-崩溃判官、O2、附带两样，写 `oracle-verdicts.tsv`。导出目录的九个文件与 E170 登记 5.3 逐列相同，执行器的 `crash_dump.load_stream_dump` 读得进就是验收。
- **调用与模型的对应**（对不上的调用记不适用，见读法 6；参数怎么交照 `history.rs` 里同一个操作的写法）：

| `calls.tsv` 调用名 | 模型 | 对拍 |
|---|---|---|
| 流一的 `acquire`、`warm_up` | 两次都返回之后调一次 `acquire_and_warm_up_in_the_make_filesystem_process` | 不比（这一步模型不交答） |
| `publish`（第一个文件） | `answer_publish_first_file` → `judge_and_advance` | 比 |
| `publish`（覆盖写） | `answer_publish_overwrite` → `judge_and_advance` | 比 |
| `mount_writable` | `close_session` 后 `answer_mount_writable` → `judge_and_advance` | 比 |
| `roll_back` | `answer_rollback_while_mounted` → `judge_and_advance` | 比 |
| `raise_floor` | `answer_raise_rollback_floor` → `judge_and_advance` | 比 |
| `unmount` | 无 | 不适用，模型停在这里 |

  模型的起点：照 `history.rs` 第 1059 行起模型的同一个构造（`IdealModel::after_make_filesystem_on_a_journal_ring_of`，几何取这条流 mkfs 的参数）；每推一步之后把 `committed_versions` 并进一张表（崩溃注入的做法）。
- **要读 crates 常量的量**（逐个写成本地常量、`main` 开头与 crates 的同名公开常量或现算值回比，不直接引；回比不过走停机 S1）：盘数、盘字节数、物理块与 io_min、固定 fsid 与写时刻、两份文件内容的生成式、流二的段长表与写数与根槽写数、两条流的全量与快档状态数——与 E170 导出装置第 78–140 行同一批。**只在基线那一趟回比**：坏实现改了写流，这些数在坏实现那一趟本来就会变，不回比、照数记进产物。
- **模式**：`baseline --output-directory <目录>`（基线：回比常量、导出、判）；`row --output-directory <目录>`（坏实现那一趟：同上但不回比常量）；`known-bad-images --dump-directory <基线导出目录>`（5.5 的已知坏镜像族）；`--positive-control <O1-对拍|O1-崩溃判官|O2>`（5.6）；`--disable-oracle <O1|O2>`（Q3-核）。
- **研究一侧**：`research/e7-index-bench/src/bin/e171_three_oracle_collusion/`（Python，目录里不放 `main.rs`；先例 E170 执行器目录）：挑法脚本、逐行驱动（拷仓、改坏、编、跑装置、跑执行器、收判定）、比对器（矩阵、Q1–Q3、触达、轨迹）、比对器的单测（`python3 -m unittest`）。变异表 `research/mutations/e171_three_oracle_collusion.tsv`（比对器与驱动那一侧）；装置一侧的变异行追加在 `crates/mutations.tsv` 末尾。产物 `research/results/e171-three-oracle-collusion-<日期>-s<段>-<名>.out`；导出目录 `research/target/e171-dumps/<基线或行名>/<流名>/`，坏实现那几趟的导出判完即删（只留 `manifest.tsv` 与判定表），基线的留着。
- **执行器不改**：本实验只调冻结的那一版；判坏状态照 `comparator_q3_bad_states.py` 的「判的那一半」叠补丁（拷进本实验目录、改名，不改原文件；拷来的那份记 sha256）。

### 5.4 O3 怎么跑

- 版本照第一节读法 8；开跑前 `sha256sum -c`（冻结行抄在第十二节）不过走作废 Z2。
- 每一趟（基线、每一行）：`python3 executor_main.py --dumps <这一趟的导出目录> --batches 1,2 --workers <不超过线程上限> --output <新文件>`，在冻结的执行器目录里跑，经 `capped.sh` 与 `run-with-memory-cap.sh`。每态两个判定（严格、暂代）照读法 7 记进矩阵；`digest=mismatch` 任一态走作废 Z4。
- 坏镜像（5.5）：本实验目录里那份拷来的「判的那一半」叠补丁判，不打开期望与实现判定文件，判完再交比对器。
- 盲测族的种子照 E170 登记 5.5：这一版冻结的执行器文件（按路径排序）逐个喂进 sha256，取摘要前 16 个十六进制字符。

### 5.5 坏镜像表（Q2）

每张坏镜像 = 两条流上一个合法崩溃状态（底态）打字节补丁；O2 判补丁之后的镜像，O3 照 5.4 判（A(c) 按底态算，补丁不算写，E170 登记第一节读法 3）。三族同权：

| 族 | 从哪来 | 行数 | 造给谁的 |
|---|---|---|---|
| P 行 | E170 登记 5.5 的 P1、P2、P3、P4、P5、P8、P9（那张表没有 P6、P7：`grep -n 'P6\|P7' research/prompts/e170-preregistration.md` 零命中），底态 F、Mk（P9 是 Mk⁺），两条流 | 26 | 照 O3 的判据（kb 条款）造 |
| 盲测族 | E170 登记 5.5 的盲测族，种子取本实验用的那一版（5.4），两条流各 20 行 | 40 | 照 O3 的判据造、种子在执行器冻结之后才算得出 |
| 已知坏镜像族 | `checker_known_bad_images.rs` 第 747 行 `known_bad_images(clean)` 的每一条，施加在流一的全持久态 F 上；补丁 = 施加前后逐 512 字节扇区的差 | 41 | 照 O2 的判据造 |

- **P 行与盲测族的字节由 E170 导出装置造**：`e170_o3_spec_executor_export bad-states --dump-directory <基线导出目录的一份拷贝> --seed-hex <种子>`；它写的 `bad-impl-verdicts.tsv` 里池级 checker 那一列就是 O2 在这几行上的判定（同一个 `check_pool_image`），比对器从那里取 O2，与本装置自己在同一行上调 `check_pool_image` 的结果逐行比（停机 S3 同形）。E170 第三段已经造过的那一批（`research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states.out` 记的那一次）**不直接拿来**：盲测族的种子随执行器版本变，P 行由同一份装置在同一份基线导出上确定性重造。旧种子那一批 40 行若还在 `research/target/e170-dumps/`，当回归族另跑一遍（附带，够判后不跑；它对改过的执行器已经不是盲测）。
- **已知坏镜像族与 `checker_known_bad_images.rs` 的关系**：用例里的干净镜像就是 `build_pool` 走完的池，等于流一的全持久态 F（停机 S4 核摘要相等）；用例断言每一条都让 O2 的目标不变量红，本实验在中性格式这条管子上重放同一批改坏，同时第一次让 O3 判它们。条目由装置 `#[path]` 引或照抄（执行员定）；照抄的逐条与原函数比条目名与条数。那个文件里其余几族（发布 B 之后、回退之后、抬 F 之后……）的干净镜像不在两条流的任何一个状态上，不取。
- **E170 的 N 行**（N1–N5，条款容忍的改坏，期望「在」）不进 Q2 的表：它们按条款不算坏镜像。O2、O3 在 N 行上的判定照报，附带，够判后不跑。
- **几何取样点**：P 行自带两个底态（F、Mk），逐行比两个底态上的判定（第八节）；已知坏镜像族只能在流一 F 上（用例的闭包写死了流一那份池的单元槽号，例 `DATA_UNIT` = 50180），没有方向相反的取样点，第八节照写。

### 5.6 阳性对照与阴性对照（每条 oracle、每一族都跑）

| 族 | 阴性（必须全放行） | 阳性（必须红） |
|---|---|---|
| 坏实现（Q1、Q3） | 基线两条流批 1、批 2 每一态、每一次调用：O1-对拍、O1-崩溃判官、O2、O3 甲乙一处都不红；**空变异**——拿表里第一行、把替换文换成原文本身，走完整条逐行驱动（拷仓、编、跑、执行器、比对），必须判未触达、四列全不红 | 装置的 `--positive-control`：**O1-对拍**——流二第一次 `publish` 的观测里那一版的文件（`ObservedRoot` 的 `file: ObservedFile::Decoded`，`model.rs` 第 441 行起）内容末字节翻一位再交 `judge_and_advance`；**O1-崩溃判官**——批 1 里第一个「实现读回了文件」的态，读回内容末字节翻一位再交 `crash_recovery_disagreement`；**O2**——流一 F 态叠已知坏镜像族第一条再交 `check_pool_image`；**O3**——流一 F 态叠 P1 补丁交执行器（期望出自条款，A2）。四样各必须红；在基线那一趟与空变异那一趟各跑一遍（证明逐行驱动这条管子把红一路带到矩阵）。比对器：手造矩阵里放一行四列全不红的触达行，Q1 必须报「有」并点名它 |
| 坏镜像（Q2） | 两条流的四种底态 F、Mk、Mk⁺、Mk⁻ 不打补丁（8 张）：O2、O3 甲乙都不红 | P1（O3，期望出自条款）；已知坏镜像族第一条（O2，期望出自用例的断言）；比对器手造一行两列全不红，Q2 必须报「有」 |
| 关一条（Q3） | 手造矩阵里一行 O1、O2 都红：关掉任一条仍红 | 手造矩阵里一行只 O2 红：关掉 O2 必须报「只靠 O2」；Q3-核（读法 9）用真装置关 |

### 5.7 谁跑什么：执行员能跑的、交主 agent 的、归提交时的

| 跑什么 | 谁 | 依据 |
|---|---|---|
| `cargo build --release -p singlefs-checker-tier --bin e171_three_oracle_collusion`、fmt、clippy | 执行员（前台超过 240 秒的写进交回、由主 agent 跑） | 只编不跑代码，不是重型 |
| 装置的 `baseline`（两次，作废 Z1）、`--positive-control` 四样、`known-bad-images`；E170 导出装置的 `dump`（停机 S2 对照）与 `bad-states`；执行器在基线与坏镜像上；比对器与它的单测；`research/mutations/e171_three_oracle_collusion.tsv` 经 `research/scripts/mutate.sh` | 执行员，一律经 `run-with-memory-cap.sh` 与 `capped.sh` | 跑实验装置二进制与研究一侧的变异表，E170 执行员同样跑过 |
| 逐行驱动试跑：空变异一行加第一刀里的前两行（量单行挂钟，Q4） | 执行员 | 三行，不是整表 |
| 第一刀 24 行、其余 93 行、扩表的行（逐行驱动整表） | **主 agent**，后台起；每行拷仓、改坏、编、跑；`.claude/hooks/heavy-test-guard.sh` 若把它判成重型，主 agent 照 `.claude/skills/crash-test/commit-stages.md` 在提交时跑或问用户（`SINGLEFS_HEAVY_TESTS=user-request`） | 形态就是一张变异表整表跑：每行一次 checker 档冷编译，派发提示「重型（cargo 跑 checker 档、变异表）归提交时」 |
| 装置的内联单测（`cargo test -p singlefs-checker-tier --bin e171_three_oracle_collusion`）、追加进 `crates/mutations.tsv` 的装置变异行的复跑、`research/scripts/replay.sh` 里本实验那几行 | **重型，归提交时**（主 agent 照 `commit-stages.md`） | checker 档包的测试二进制与 crates 变异整表、全部实验复跑都在重型清单里（`.claude/agent-common.md`「不做」一节） |

线程上限与内存上限从派发提示取；逐行驱动的并行线道数、每条线道里 cargo 与装置与执行器的线程数加起来不超过线程上限。

### 5.8 实现量与分段

实现量（推的，没量）：装置 1 500–2 200 行 Rust（照抄 E170 导出装置的驱动、取样、导出约 900，推模型与 O1 两样约 300，已知坏镜像族约 250 或 `#[path]` 引，对照与关一条的开关约 150，常量回比与停机约 150）；逐行驱动约 300 行 Python，比对器约 400 行加单测约 300 行。枚举量：2 条流 × 2 批 ×（117 行 + 基线 + 空变异）；坏镜像 107 行 + 8 张阴性底态（另 10 行 N 行附带）；Q3-核 3 行 × 3 种关法。不另枚举崩溃支线与几何格。

分段（每段交回之后主 agent 对着问题单逐行判续不续，一行开着的都不剩就停，后面的段不跑）：

| 段 | 谁做 | 做什么 | 补问题单哪一行 | 补完够不够判 |
|---|---|---|---|---|
| 第一段 | 执行员（读 `crates/`） | 写装置、挑法脚本、逐行驱动、比对器与单测；基线两次、停机 S1–S6 与 S8、阴性与阳性对照（5.6 全表）；Q2 三族全跑；逐行驱动试跑三行（空变异 + 第一刀前两行） | Q2；Q4 的单行挂钟 | Q2 够判；Q1、Q3 不够；Q4 单行数有了，整表数没有 |
| 第二段 | 主 agent 起整表（第一刀 24 行，再其余 93 行），执行员接着比对 | 117 行逐行跑完、比对、轨迹与批 2；Q3-核三行 | Q1、Q3、Q4 | 够判；F4 或 F6 触发时不够，进第三段 |
| 第三段（只在 F4 或 F6 触发时） | 同第二段 | 扩表 164 行 | Q1、Q3 | 够判，或照 F4 记「不够判」 |

第一段与第二段共用装置与导出目录，第二段排在第一段之后。第一刀 24 行跑完先交一次：那时 Q1 若已出现一行三条全放行（甲或乙），Q1 的「有」已够判，主 agent 判其余 93 行还跑不跑（Q1「没有」那一边与 Q3、Q4 还要它们）。

### 5.9 坏实现表（117 行，以这张表为准）

`python3 /tmp/claude-1000/collusion-designer/select_bad_implementations.py crates/mutations.tsv 8` 在 `crates/mutations.tsv` sha256 `40c75af9…` 那一版上的输出；「刀」1 是第一刀（每个文件 i = 0 那一行），2 是其余；「行文 sha256」是那一行整行（六段连制表符）的 sha256 前 12 位，开工时对不上照 5.2「行文变了」那一条办。表格本身的 sha256 `638f8a80bc4f3698f4a8e84504e0d8ccb1032fdd649ee3a6bc10df9ceb274bc7`（生成在草稿目录 `table-117.md`）。

| # | 刀 | 文件 | 组内下标 | 变异名 | 行文 sha256 前 12 位 |
|---|---|---|---|---|---|
| 1 | 1 | `core/admission.rs` | 0 | C318 实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒 0（影子账隔离的单元又不进不等式） | `15642eb5e39f` |
| 2 | 2 | `core/admission.rs` | 7 | D28 已定项 3 实现：实例表链的片数向下取整（370 行算成一片） | `e5eaaceedacc` |
| 3 | 2 | `core/admission.rs` | 15 | 实五 ND（D28 已定项 1：可用不再扣 defer 待释放）：defer 里的槽扣两次（已分配里一次、另加一次） | `803a81a7028c` |
| 4 | 2 | `core/admission.rs` | 23 | 实审 A4b：中央映射树叶层一次都不按会切算 | `b240c9e0d23f` |
| 5 | 2 | `core/admission.rs` | 31 | 实审 A4d：K1 漏掉罩整个 key 空间的根 | `d78e96210028` |
| 6 | 1 | `core/allocation_record_tree.rs` | 0 | K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它的叶，按位置寻址的核判出） | `4e994f9ef97b` |
| 7 | 2 | `core/allocation_record_tree.rs` | 7 | K1 单测：分配记录树按槽号找叶时叶宽算成两倍（叶的边界不在 812 的整数倍上） | `7c045a579033` |
| 8 | 1 | `core/allocator.rs` | 0 | 空闲不随分配减（I-5.2 要红） | `16ec71e44229` |
| 9 | 2 | `core/allocator.rs` | 7 | 步 5：提交内生块的 bump 游标不绕开开段之后才隔离或扣住的槽 | `a836d4b989c8` |
| 10 | 2 | `core/allocator.rs` | 14 | 增补 2：提交内生块段耗尽不回落（抬 F 的空发布在唯一全空段被扣住时又报 NoSpaceFor） | `a1a9ffe21eb8` |
| 11 | 2 | `core/allocator.rs` | 22 | 增补 3 第 1 件（代码三方第一轮第 3 条）：随机历史快档判出「复用改写已回收记录时不改分配代」（N2） | `217469aeb5c7` |
| 12 | 2 | `core/allocator.rs` | 29 | 步 0 开关一（复用窗口置 0）：回收下界退回 0（不再无视 F），释放的落点一个都不回收 | `a6433e2374f5` |
| 13 | 2 | `core/allocator.rs` | 36 | C518（一次挂载之内环转过一圈之后不回收）：挂载内不回收时，一次挂载里转过根环的历史又停在已知红第 0 条（C518 那一处） | `c6eeb65822b3` |
| 14 | 2 | `core/allocator.rs` | 44 | 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2j）：分配器提交内生块那一处把「每块盘上都没有」报成「小盘写满」；抬 F 的空发布拿不到固定点那条写死用例判出（接替原第 166 行：单元区墙取样点换到 256 槽小盘之后只走得到用户数据那一处） | `d3a4311edd30` |
| 15 | 2 | `core/allocator.rs` | 51 | 实审 A2c 第 15 条：用户数据落点不从这张图的单元区起点起扫 | `6d1ee440a604` |
| 16 | 1 | `core/block_device.rs` | 0 | 增补 3 第 3 件（用户 2026-09-20 定案第 9 条）：镜像文件不排他创建，撞上同名接着用旧镜像 | `6eb72f44fd68` |
| 17 | 2 | `core/block_device.rs` | 7 | 文件后端 ENOSYS 不算不支持打洞（内核没有 fallocate 时照常报错、不退回真写零） | `59c465df406c` |
| 18 | 1 | `core/code_two_tree.rs` | 0 | 树分裂 核心规则：叶切在末尾（左半留 n − 1 条）而不是从中间切 | `d98101386f1e` |
| 19 | 2 | `core/code_two_tree.rs` | 7 | 树分裂 上一版的叶一律不算照抄（没碰到的叶也整片重写） | `91999f5da1d7` |
| 20 | 2 | `core/code_two_tree.rs` | 14 | 树分裂 读树时不核内部节点的子树覆盖区间 | `c6f8577750a1` |
| 21 | 1 | `core/extent_tree.rs` | 0 | K2：extent 树下段根的层级取「罩的单元数大于单元数」的最低一层（144 个单元装满一片叶时白长一层） | `afcbcbf908fe` |
| 22 | 2 | `core/extent_tree.rs` | 4 | K2 单测：extent 树上段根的层级取「罩的 inode 数不小于最大 inode 号」的最低一层（inode 143 还算在第 0 片叶里） | `c9a6ce82c798` |
| 23 | 1 | `core/inode_tree.rs` | 0 | 并行线三 C116：分裂出来的右半容器号不是触发分裂那条新记录的 inode 号 | `e090f27ea3b7` |
| 24 | 1 | `core/instance_table.rs` | 0 | 实例表第二片：读者收下位置指针没清零的「无下一片」 | `fec37df3f268` |
| 25 | 2 | `core/instance_table.rs` | 5 | 实例表第二片：片数向下取整（370 行算一片） | `cf125f04dd5b` |
| 26 | 1 | `core/journal.rs` | 0 | 并行线一：记录读者不判点名项区越没越过 4096（并行线一验收第 4 条第三个变异：点名项超过 67 仍塞进一条记录 ⇒ 记录解析拒收） | `eef360900a56` |
| 27 | 2 | `core/journal.rs` | 5 | 实二十：记录头读者把末条标志位 0 读反（带标志的读成不是末条） | `fd0f4a108c59` |
| 28 | 1 | `core/make_filesystem.rs` | 0 | 新池新建文件 步 1：mkfs 不核根环区域归属是不是第一版写死的 0 / 1 / 0 | `350167260488` |
| 29 | 2 | `core/make_filesystem.rs` | 7 | 实审 A2a 第 16 条：根槽装不下根记录不在开头拒（走到 to_slot 的断言） | `933c0542110e` |
| 30 | 2 | `core/make_filesystem.rs` | 14 | 实审 A2b 环短于三条记录：恰好三条也拒（边界） | `2ff413741862` |
| 31 | 2 | `core/make_filesystem.rs` | 21 | A3b 善后一：mkfs 判单元落不落在盘内只算到实例表（树表那一槽在盘外照往下写） | `66bec7ff9af0` |
| 32 | 1 | `core/mount.rs` | 0 | 步 3：不写行（实例表照抄） | `74b18c84938e` |
| 33 | 2 | `core/mount.rs` | 7 | 步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效 | `b093430c0295` |
| 34 | 2 | `core/mount.rs` | 15 | 步 5：抬 F 生效之后不放开扣住的槽 | `742d1ec48e41` |
| 35 | 2 | `core/mount.rs` | 23 | 步 5：算抬 F 上限时有效根的树表读不出，按「树表里没有这两棵树」猜而不是拒绝 | `71b7c1644cea` |
| 36 | 2 | `core/mount.rs` | 31 | 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4a）：根环转过之后回收门槛取「环里最旧有效根 + 1」；逼近分配记录墙那一段照跑 checker 判出 | `71ec2b156c2d` |
| 37 | 2 | `core/mount.rs` | 39 | 增补 2 第 21 行（C366，挂载层）：写行那次发布的 jsn 取 txg（所选根自己那条记录读不出时写行 txg 5、jsn 应是 4） | `6d454f491473` |
| 38 | 2 | `core/mount.rs` | 47 | 增补 2 收口表第 58 行：可写挂载在暖机里失败时已落盘的账只交写行、漏掉前几次暖机 | `1e9b2866ef90` |
| 39 | 2 | `core/mount.rs` | 54 | 实二五 C546（抬 F 被拒时扣住的槽不退回）：第一次空发布在任何写之前被拒时分配器不换回抬 F 之前那一份 | `3b00e31a95f8` |
| 40 | 2 | `core/mount.rs` | 62 | 实二六（代码三方 m2-final-code-r3 判决第四节第 1 条，C546 第二次起那一半）：抬 F 那一串真发时第二次落盘途中报错、交回的已落盘份数成了 0（接替第 495 行：它点名的用例随「落点被拒」那一格挪进预演而删掉） | `eb25fefa9e8d` |
| 41 | 2 | `core/mount.rs` | 70 | 实二七（fsync 失败掉盘的核查，调度记录 2026-09-24 第三节；D2 已定项 13 挂载准入、D18 已定项 11 可写挂载的顺序与前提）：树表 0 条那一版的分配记录树节点在单元清单里记成树表 | `22f8061467ce` |
| 42 | 2 | `core/mount.rs` | 78 | 实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：卸载把 F 抬到现行那一版带的 F（原地不动）而不是它的 txg | `b14a841b61f6` |
| 43 | 2 | `core/mount.rs` | 86 | 实三（D23 已定项 14「这一版的失败处置」、C558）：回退入口不先判冻结着的发布 | `030219028afe` |
| 44 | 2 | `core/mount.rs` | 94 | 实五 C283：一次准入最多 8 次发布改成 10 次（推满的判据松了，推满仍不够那一格多推一串） | `fcef7e71e97b` |
| 45 | 2 | `core/mount.rs` | 102 | 实五 445：挂载那一刻就读不出的槽也重读、仍坏就拒（挡住了条款说当没有根的那一格） | `023c98c68fc5` |
| 46 | 2 | `core/mount.rs` | 109 | 实审A1b Q1：抬 F 在预演里被落点拒不算空间不够（挂载写行之后推的那一串取不到落点就整次失败） | `a05aa4e911f1` |
| 47 | 2 | `core/mount.rs` | 117 | 实审A1b Q2：一次准入里再推一串不核调用方的参数（预算用完那一格照样交回） | `d5671fae0462` |
| 48 | 2 | `core/mount.rs` | 125 | 实审 A2c Q5：抬 F 报错时不交回之前推成的那几串 | `c5ae1fe2af6c` |
| 49 | 2 | `core/mount.rs` | 133 | C554 乙：c_见证 取系统配置槽 tail 的最小值（只剩较旧那一槽的见证） | `9098a4bc2454` |
| 50 | 2 | `core/mount.rs` | 141 | Z3-A 乙：挂着之后收盘表的入口不核盘「可见」（管理员回退收到空盘） | `774a735747c1` |
| 51 | 2 | `core/mount.rs` | 149 | C554 乙 模型跟进：可写挂载只写上一个实例那一行、不写中间实例行（烧掉的号那一形护；崩溃恢复写 (2, 0, 0) 那一形乙之后拒可写、checker 那份改走事务层） | `328de0db43f0` |
| 52 | 2 | `core/mount.rs` | 156 | 合入后验证一：C554 乙关掉，崩溃恢复抛弃根又造得出被抛弃的根（落到实例 2） | `293b6256b09e` |
| 53 | 2 | `core/mount.rs` | 164 | 代码三方第二轮 Y2-b：挂着之后收盘表的入口不比本盘设备号（管理员回退） | `5754e74ef393` |
| 54 | 2 | `core/mount.rs` | 172 | C585 改法 C：落后支去掉根槽那一半、回到落后且缺单元才拒（正常卸载放进发布途中取下的、那次发布的根住的那块盘） | `5f2074cd8e77` |
| 55 | 2 | `core/mount.rs` | 180 | m3-code-r1 H2：落后支缺根那一半只查盘表里 region 0 那块盘的根槽（根住盘 1 的那一版缺根放过去） | `23a169293706` |
| 56 | 2 | `core/mount.rs` | 188 | m3-code-r2 J3：正常卸载、准入抬 F、管理员回退上缺根那一半把根槽里自证得过的根都当成现行那一版的（准入抬 F，根环转过一圈、txg 27） | `67a23814fa6b` |
| 57 | 2 | `core/mount.rs` | 196 | m3-code-r3 L1：落后支缺根那一半只在第一次读不出时才重读（准入抬 F：根槽里是现行那一版的落后盘，第一次读回的字节坏一位、自证不过就被拒成缺根） | `90de0b6375c3` |
| 58 | 1 | `core/mounted_read.rs` | 0 | 并行线二：位置提示过期转查中央映射时多跳计数不加（D19 已定项 5 硬规则 3 的观测点等于没有） | `b34d2e5309a7` |
| 59 | 2 | `core/mounted_read.rs` | 7 | C483 ②：挂载态打开时 inode 树根只按位置提示读（不经映射回退） | `91313790af6c` |
| 60 | 2 | `core/mounted_read.rs` | 14 | 实审 A2a 第 21 条：只读挂载把没有 extent 条目的 inode 当写过内容 | `77d23604908d` |
| 61 | 1 | `core/mounted_session.rs` | 0 | 实五 P3pl（D16 已定项 1「准入」那一行：准入放行而落点取不到也推）：会话只在准入拒时推抬 F，落点被拒原样交回 | `7f1fa64b586f` |
| 62 | 2 | `core/mounted_session.rs` | 5 | Z3-A 乙：会话发布之前不核盘「可见」（别的池的盘） | `25a4624c4cee` |
| 63 | 1 | `core/pointer.rs` | 0 | A3a 第 29 条：指针头部 MAC / nonce 非 0 不判 | `2e1832bc7f31` |
| 64 | 1 | `core/records.rs` | 0 | extent 叶记录的读者不判条目宽（按 112 的固定偏移切 24 字节的条目） | `fc06e850313e` |
| 65 | 2 | `core/records.rs` | 8 | C480：blocks 向下取整（⌊size ÷ 512⌋，新池新建文件那 3000 字节写成 5） | `a1fe0db4a783` |
| 66 | 1 | `core/recovery.rs` | 0 | 逐盘核退回只核总数 | `97362055b3fd` |
| 67 | 2 | `core/recovery.rs` | 7 | 增补 3 第 4 件：失败时点名的换成最后一块两槽皆废的盘，不是设备次序里第一块 | `c995d8b8cd7d` |
| 68 | 2 | `core/recovery.rs` | 15 | 普查 R7：分配记录的几何判不判跨度（被抛弃根那棵账里的坏记录又喂进 isolate 的跨度断言） | `9c67167f26e5` |
| 69 | 2 | `core/recovery.rs` | 22 | 实例表第二片：读者读到第 0 片就停，按实例表判被抛弃看不到第二片上的行 | `0e81a0a45326` |
| 70 | 2 | `core/recovery.rs` | 30 | 实二十：层 0 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去） | `bff9fa2c9892` |
| 71 | 2 | `core/recovery.rs` | 38 | 实审 A2a 第 25 条：链值不等的记录照样进重放前缀 | `9f8c87579ea2` |
| 72 | 2 | `core/recovery.rs` | 45 | 实审 A2b 第 27 条：分配记录读者不经共用判定 | `8483ca537f47` |
| 73 | 2 | `core/recovery.rs` | 53 | 实审 A2b 第 37 条：逐档试一槽可择的都没有时丢掉 incompat 位图不认识的那一槽 | `d4ee352257eb` |
| 74 | 2 | `core/recovery.rs` | 61 | A3a 第 33 条：重建读分配记录树退回只核位置 | `adba948d07c4` |
| 75 | 2 | `core/recovery.rs` | 68 | A3a 第 38 条：physical_block_size 不判上界（根槽盖到下一个槽） | `7f4b1929847f` |
| 76 | 2 | `core/recovery.rs` | 76 | 实审 A3d：重建不判分配记录每个落点每盘各一条（两盘的账不对称：可写挂载改红在下一道分配记录代的判定上） | `a8aae8398925` |
| 77 | 2 | `core/recovery.rs` | 84 | 实审 A3b Q6：算生效 F 时最新根的实例表重读仍读不出改回「不按表滤」（算 F 那一段本身） | `0d8d4580e16d` |
| 78 | 2 | `core/recovery.rs` | 91 | I-9.16 树表排序：核心层不判 ② 发号次序（重建放过互换种类的树表） | `24120af2ebcf` |
| 79 | 2 | `core/recovery.rs` | 99 | 代码三方第二轮之后规格第 3 条（D16 第 37 行全句）：知道住着根的槽自证不过不重读（管理员回退，step_four 那一形） | `2af541a68dfa` |
| 80 | 2 | `core/recovery.rs` | 107 | 胶水把核报的「所选根那次发布带末条标志的多于一条」当不变量破坏 panic（G6 改回去） | `943b0d3d240a` |
| 81 | 2 | `core/recovery.rs` | 114 | recover 的 Consult 一臂调 replay_journal 的 verify_named_units 从「policy 是 Consult」改成 false（形式化题面第三轮第四节第 9 条，R3-9 之三）：验的那一臂也不验点名单元 | `b595e5bb6750` |
| 82 | 2 | `core/recovery.rs` | 122 | 扫环胶水不照核交回的位置取、读到的每一份都插（同一 (实例代号, 计数器) 后读到的那份覆盖先读到的，R3-10 的胶水一侧） | `75d4fcd44182` |
| 83 | 1 | `core/root_record.rs` | 0 | 代码三方 m2-wave3-code-r1 第四节第 4 条：根记录里分配记录树根指针的偏移错一位（写者 342 写到 343，记录总长 457 不变；全零指针的往返单测看不出） | `1efecfd03bd0` |
| 84 | 1 | `core/root_ring.rs` | 0 | 根环一个区域按 S − 1 个槽算：最后一个根槽不清，旧池留在那一槽的根活下来 | `c5a3c85e3f7e` |
| 85 | 1 | `core/system_configuration.rs` | 0 | 系统配置四类：运行配置那一档的字节预算从 36 改成 35（D22 已定项 26 的 389 / 4 / 36 / 60 里动一个数） | `d41988152086` |
| 86 | 2 | `core/system_configuration.rs` | 7 | 实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：写系统配置槽时 F 那 8 字节写成 0、不写字段里的值 | `99ebf8e26724` |
| 87 | 2 | `core/system_configuration.rs` | 14 | 实审 C11b 在飞上限两份各算：core 那份把 F 多算 1（core 与装置在环长上对不上） | `35cd0e2d6867` |
| 88 | 1 | `core/transaction.rs` | 0 | 映射查到 key 就算（落点指错不核记录） | `79533897aa3e` |
| 89 | 2 | `core/transaction.rs` | 7 | 步 3：第一个文件版本的 txg 写死 3，不从它要建在上面的那一版接着算（2026-09-23 用户定案：`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 那条流） | `95f9b7ed315a` |
| 90 | 2 | `core/transaction.rs` | 15 | 增补 2 第 11 行：新池新建文件的改动计数写回 1（字段表定义是最后一次改动所在发布的 checkpoint_txg = 3） | `434000734c8e` |
| 91 | 2 | `core/transaction.rs` | 23 | 并行线三：建 N 个 inode 之后记账里的水位不加 N（I-9.6 要在盘上判红） | `4b3240a0b7bb` |
| 92 | 2 | `core/transaction.rs` | 31 | 步 1 验收第 4 条：inode 记录的写入时间留成 A 的（覆盖写照抄上一版的写入时间） | `1075b816d5f9` |
| 93 | 2 | `core/transaction.rs` | 39 | 增补 2 第 8 行（C495）：拿掉第一个文件版本那道核验（上一条记录解不出或 txg 对不上都照写） | `a68291ac2b0b` |
| 94 | 2 | `core/transaction.rs` | 46 | 并行线一：C491：共享的提交内生块挪到一次发布的第一条记录里点名（末条只点名自己那个数据单元） | `ec67c0d6b5b3` |
| 95 | 2 | `core/transaction.rs` | 54 | C518（一次挂载之内环转过一圈之后不回收）：第一个文件版本之前调用方直发的零单元根不补记进根环表（表判出跳号、停下） | `124b60604eba` |
| 96 | 2 | `core/transaction.rs` | 62 | 实二十：多条记录的发布每条都带末条标志 | `496325f31dd8` |
| 97 | 2 | `core/transaction.rs` | 70 | C394 N1：「先重读一次」写成重读两次（条款只重读一次，不多次重试） | `7e53fc7c5274` |
| 98 | 2 | `core/transaction.rs` | 78 | 实例表第二片写路径：树表 0 条的一版上写行时旧链只释放第 0 片 | `8b8073ee79cd` |
| 99 | 2 | `core/transaction.rs` | 85 | 步 1 变异：defer 行写 0（树分裂换锚点：记账行按行种类取值） | `673592fce861` |
| 100 | 2 | `core/transaction.rs` | 93 | 实二二三 d（D19 已定项 5，用户 2026-09-25 定）：映射条目两条位置项指同一块盘时不拒（另一块盘那一份没核就释放） | `814d605e606e` |
| 101 | 2 | `core/transaction.rs` | 101 | 实二五 D19 已定项 5（核出对不上先重读一次）：读得出而对不上不重读、当场隔离 | `0315bf1e8062` |
| 102 | 2 | `core/transaction.rs` | 109 | 实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：抬 F 先写系统配置那一步不带新 F、照带盘上旧的生效值 | `3a6d16eb5b48` |
| 103 | 2 | `core/transaction.rs` | 117 | 实审 A2c 第 34 条：第一个文件版本换下 mkfs 那几片时不逐盘核（在册就交给释放） | `fb49f9b7a760` |
| 104 | 2 | `core/transaction.rs` | 124 | 实审 A2c 第 26 条：树表 0 条的一版上写行数记录条数只数一个角色 | `64ebb5e5a1c5` |
| 105 | 2 | `core/transaction.rs` | 132 | 实审 B3c-3（树表 0 条的一版上写行那一版比实例表与分配代）：写行那次发布把这一版的分配记录报成零单元发布那一臂（不交回） | `978d4529bfed` |
| 106 | 2 | `core/transaction.rs` | 140 | 实审 A3c Q1：屏障之后取见证值那一遍读不出的盘不拒、照写取号写 | `ef6cef922c52` |
| 107 | 2 | `core/transaction.rs` | 148 | C331 甲 V4（用户 2026-09-28 定）：取号那一刻缺一槽照样取号，回到「一份都没有才拒」 | `d056fe99f816` |
| 108 | 1 | `core/unit.rs` | 0 | 增补 3 第 1 件（代码三方第一轮第 4 条）：随机历史快档判出「checker 判绿的镜像上冷启动读回报错」（B6：补齐字节从最后一个载荷字节算起） | `b2f18ffee9f3` |
| 109 | 2 | `core/unit.rs` | 6 | A3a 第 29 条：码 2 读者不判 29 字节预留位 | `abebad6cd099` |
| 110 | 1 | `core/write_accounting.rs` | 0 | 增补 1：树表单元的写记到中央映射树名下（合计不变，按种类钉的字节表红） | `19ffd5418e65` |
| 111 | 1 | `core/write_request_split.rs` | 0 | 实审 A2a 第 21 条：没写过的 inode 长度 0 也按除法给一个单元 | `33bd7451555d` |
| 112 | 1 | `replay-prefix/lib.rs` | 0 | 步 3：前缀跨实例边界（把别的实例的记录也接上） | `7d3a144c6d52` |
| 113 | 2 | `replay-prefix/lib.rs` | 7 | P6 后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮 K4-b） | `7d7442a05ba2` |
| 114 | 2 | `replay-prefix/lib.rs` | 14 | 实二二三 b（C539，D23 已定项 4 读者规则）：锚点读得出时下一次发布的首条序号不是 1 照接 | `be0e12abf325` |
| 115 | 2 | `replay-prefix/lib.rs` | 21 | 取前缀核锚点筛选不看 txg（攻方 K19）：别的发布的末条标志让恢复停在报错上 | `7dcbfa7fb61d` |
| 116 | 2 | `replay-prefix/lib.rs` | 28 | 取前缀核读盘形状（代码轮第三轮 J1、用户定案 U1）：锚点读不出那一支不判序号（H5-2 删回去），链首序号不是 1 时先调反向链回调、多重读一次上一条记录 | `c36c7f312668` |
| 117 | 2 | `replay-prefix/lib.rs` | 35 | 取前缀核读盘形状（C648 攻方 S03，可写挂载、上一条记录 0 号盘那一份全零那一格）：反向链回调每个候选调两次，可写挂载时读缓存不收的零份每个候选在块层重读两次 | `274ff0c7f6c7` |

## 六、报哪些量与各自的判据

Q1–Q3 问的本身是计数（「有没有一行全放行」），翻面的门槛就是问题单的候选定义；照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「门槛不能是臂定义的同义反复」，每一行另配一格不由候选定义推出的门槛（触达行数、O3 覆盖、阴性为 0），由它决定「没有」那一边够不够判。每个量各占一行，合取留给人做。

| 量 | 怎么算 | 门槛与判定 | 对应问题单哪一行、取什么值翻面 |
|---|---|---|---|
| Q1-分母 | 117 行（含补位）里触达、且不在读法 5 那几类里的行数 D；另逐类报：未触达、只在不看 journal 那一遍不同、编不过、锚点不唯一、找不到、实现自崩、超时 | D ≥ 30 时 Q1 的「没有」够判；D < 30 走 F4（扩表），扩完仍 < 30 记「不够判」。30 的来处：30 行全被抓时，「一行全放行」的概率 95% 上界约 3/30 = 10%（三法则），不由臂的定义推出 | Q1：D < 30 时 Q1 停在「开着」 |
| Q1-甲 | D 行里 O1（对拍或崩溃判官任一红）、O2、O3 严格（「不在」才算红，欠一句不红）三列都不红的行数，逐行列出（名字、三列各自红的态数与适用态数、写流从第几条 op 起与基线不同） | ≥ 1 判「有」；0 且 D ≥ 30 判「没有」 | Q1：≥ 1 行即翻向「有」 |
| Q1-乙 | 同 Q1-甲，O3 取暂代跑法；乙红甲不红的行标「只靠暂代读法撑着」 | 同上 | Q1：同上 |
| Q1-O3覆盖 | 每行两条流批 1 上 O3 严格跑法给出「在 / 不在」的态数（不含欠一句、error、底镜像恢复不出）；报为 0 的行数占 D 的比例 | 比例 ≥ 90% 走 F7：Q1-甲改写成「O3 严格跑法到不了这一族」，不报「没有」 | Q1 与岔路单 Y3 翻面观测「某一族根本到不了三条」 |
| Q1-丙（附带） | 同 Q1-乙，O1 只取对拍（读法 1 末条） | 同上，结论只交主 agent，不替代甲乙 | Q1 |
| Q1-丁（附带） | 三条全放行（甲或乙）而记录核对器或层 0 手写版本表的 oracle 红的行 | 照数列出 | Q1：不翻面，交主 agent 判「回头补 oracle」时用 |
| Q1-各列 | 每条（O1、O1-对拍、O1-崩溃判官、O2、O3 甲、O3 乙）单独判红的行数；O1 不适用的态数（读法 6）逐流报 | 描述量，不判；O1 不适用出现在 `unmount` 与「对拍判出不同之后」以外的来源，走停机 S5 | Q3 的输入 |
| Q2-全放行 | 107 张坏镜像里 O2 不红且 O3 严格不红的张数（甲），O2 不红且 O3 暂代不红的张数（乙）；分族报、逐张列出（族、行名、底态、O2 红的不变量、O3 两个判定） | 甲、乙各自 ≥ 1 判「有」，0 判「没有」；P 行里全放行的同时走 F3 | Q2：≥ 1 张即翻向「有」 |
| Q2-阴性 | 8 张不打补丁的底态上 O2、O3 甲乙判红的处数 | 必须 0，不是 0 走 F2 | Q2 的前提 |
| Q2-族覆盖 | 每族里 O2 单独红、O3 单独红（甲、乙）、两条都红的张数 | 描述量；已知坏镜像族里 O2 不红的条目走停机 S4 | Q2 |
| Q3-甲 | Q1-甲里至少一条红的行，逐行数红的条数；只有一条红的行，按那一条（O1 / O2 / O3）分组列出 | 任一组 ≥ 1 判「有一行只靠一条」；全为 0 判「都还红」 | Q3：≥ 1 行即翻面 |
| Q3-乙 | 同 Q3-甲，O3 取暂代 | 同上 | Q3 |
| Q3-核 | 第一刀里按表序头三行至少两条红的行（不够三行时取至少一条红的头三行），装置 `--disable-oracle O1`、`--disable-oracle O2` 与驱动不跑执行器各一次，其余列与不关时逐字节比 | 9 次都逐字节相同，读法 9 成立；任一次不同走 F5 | Q3 的读法是否成立 |
| Q4-单行 | 每行：拷仓与改坏、编（冷 / 增量照实记）、装置两条流（装置进程里 `Instant` 分 O1、O2、恢复、导出各计）、执行器（`perf_counter_ns`，严格与暂代同一趟）、比对；最小、中位、最大 | 描述量 | Q4 |
| Q4-整表 | 整表在 L 条并行线道下的挂钟（驱动进程里 `time.monotonic_ns` 计第一行开始到最后一行比完），与单线道各行之和 | 描述量 | Q4 |
| Q4-甲 | 整表挂钟 | ≤ 60 秒判「能进提交门禁」，否则「只能进全量」 | Q4 |
| Q4-关一条 | Q3-核那 9 次的挂钟与不关时之比 | 描述量（岔路单代价表推「合谋-坏实现的三倍」，本读法下关一条不重跑整表） | Q4 |
| N 行（附带，够判后不跑） | E170 的 N1–N5 两条流 10 行上 O2、O3 甲乙的判定 | 照数报 | 不对应任何一行 |
| 旧种子盲测族（附带，够判后不跑） | 5.5 那一条 | 照数报 | 不对应任何一行 |
| 坏镜像上的 O1（附带，够判后不跑） | 读法 11 | 照数报 | 不对应任何一行 |

**写结论时**：Q1、Q2 判「没有」只能写成「D 行触达、每行至少一条红」「107 张每张至少一条红」，不写「三条 oracle 独立」「没有共同盲区」——表里没登记的坏法照样可能三条一起放过（岔路单第二节 Y3 代价表「罩不住」列原文）。

## 七、钉绝对值的断言

每一条互比（「三列都不红」「关掉之后其余列相同」）旁边配一条绝对值：要是三条 oracle 或比对器一起错了，下面这几条照样会红。

**出自被测条款或问题单本身**（不符时走 F3「条款可能错」或 F2，不作废）：

| # | 断言 | 出处 |
|---|---|---|
| A1 | 三条 oracle 各一列，坏镜像那一族只有 O2、O3 两列（O1 列在 Q2 上为空，比对器若算出 O1 列就是接错了） | D13 已定项 6 表（第二节）：O1「不碰盘、不解析磁盘格式」；岔路单 Y3 一行「O1 不读盘，这一族只到两条」 |
| A2 | P 行 26 张每一张 O3 期望「不在」 | E170 登记 5.5 照 D22、D23、D13 条款推的期望（被攻过零轮）；不符走 F3 |
| A3 | 基线（两条流批 1、批 2）与 8 张不打补丁的底态上，O1-对拍、O1-崩溃判官、O2、O3 甲乙判红的处数都是 0 | 问题单原文「正常实现、正常镜像必须三条都放行」（派发提示）；不符走 F2 |

**独立算出、用命令核过的**（不符就作废那一格；命令与原样输出在第十三节）：

| # | 断言 | 怎么算的 |
|---|---|---|
| A4 | 坏实现表 117 行；按文件：admission 5、allocation_record_tree 2、allocator 8、block_device 2、code_two_tree 3、extent_tree 2、inode_tree 1、instance_table 2、journal 2、make_filesystem 4、mount 26、mounted_read 3、mounted_session 2、pointer 1、records 2、recovery 17、root_record 1、root_ring 1、system_configuration 3、transaction 20、unit 2、write_accounting 1、write_request_split 1（以上在 `crates/singlefs-core/src/`）、replay-prefix 6 | 挑法脚本在 sha256 `40c75af9…` 上的输出（`wc -l` 117、`cut -f1 | uniq -c`） |
| A5 | 第一刀 24 行，一个文件一行 | 同一脚本 `--first-slice`（`wc -l` 24） |
| A6 | 步长 4 的表 220 行，与步长 8 重叠 56 行，步长 4 独有 164 行 | `comm` 两张名字表（56 / 164 / 61） |
| A7 | `crates/mutations.tsv` 里改实现的行 844 行、24 个文件 | 同一筛法 `wc -l` |
| A8 | `known_bad_images(clean)` 41 条（第 747–1294 行里 `Box::new(` 41 处；字面名 38 条、常量名 3 条） | `grep -c`，文件 sha256 `e8543908…`；装置数出的条数与它不等就停下查（照抄漏了，或文件变了） |
| A9 | P 行 26 = 6 种 × 2 个底态 × 2 条流 + P9 1 种 × 1 个底态 × 2 条流；盲测族 40 = 20 × 2；N 行 10 = 4 种 × F × 2 + N5 × Mk⁻ × 2 | E170 登记 5.5 表 `grep -E '^\| P[0-9]'` 7 种、`'^\| N[0-9]'` 5 种与各自的底态列 |
| A10 | 坏镜像表 107 张 = 26 + 40 + 41；阴性底态 8 张 = 4 种底态 × 2 条流 | A8、A9 相加 |
| A11 | Q3-核 9 次 = 3 行 × 3 种关法；阳性对照装置一侧 4 样 × 2 趟（基线、空变异）= 8 次，每次红 | 5.6 |

装置自己再算的两个数与比对器各算一遍、互比（比对器用 Python 从 `calls.tsv` 与 `states.tsv` 现算，不读装置的计数）：基线上 O1-崩溃判官适用的态数（落地向量里有 `unmount` 那次调用的写持久或撕裂的态不适用）；批 1 每条流的态数（照 E170 登记 5.4 的取样规则从段表现算）。两份不等走停机 S5。

## 八、轨迹与几何敏感性

**轨迹**。被谓词消费的量是「这一行在这条 oracle 上红不红」（Q1–Q3 的谓词与将来门禁「三条全绿即判红」的谓词都吃它），它由逐态的红累出来。每行、每条 oracle（O1-对拍按调用、其余按态）、每条流报：

- 按段号（实现的切段）排开的每段红态数；**峰值**（红态最多的那一段与它的数）、**为正段数**（至少一个红态的段数）、**期末值**（最后一段的红态数）；第一处红在第几段、第几个调用之后。
- 只在一段里红、且那一段只红一个态的格标「单点红」：这一行在这条 oracle 上的「红」靠一个取样点撑着，第八节下一条的批 2 若不红，就是不稳定。
- 正文写这一行「被 X 抓」时引峰值与为正段数，不写「X 恒红」「X 从来不放」这类说轨迹的词，除非为正段数等于这条流有写的段数。

**几何敏感性**（`.claude/rules/mutation-sampling.md` 第六类）：

| 判决 | 主取样点 | 方向相反的取样点 | 怎么报 |
|---|---|---|---|
| Q1、Q3 每行每列的红 / 不红 | 批 1：每段两端点、六个等分点、全持久态、带撕裂的态（偏向段界与撕裂） | 批 2：每段七个居中点（E170 登记 5.4），不取段界、不取全持久态、不专挑撕裂 | 逐行逐列比两批；翻面的格记「不稳定」、不翻记两点；Q1-甲、Q1-乙、Q3 在批 2 上各重判一遍，与批 1 不同就整行判「不稳定」 |
| Q2 每张的红 / 不红 | P 行底态 F（全持久） | P 行同一种补丁底态 Mk（居中调用边界，之后的写都没持久） | 逐种补丁比两底态；已知坏镜像族只能在流一 F 上（读法见 5.5），写「只取一点，没有方向相反的取样点」 |
| Q4 | 线程上限 N 条线道 | 1 条线道 | 两点各报整表与单行 |

判别力自证：比对器单测里把 Q1 的覆盖门槛（30）挪到实测分母 D 与 D + 1 之间，「没有」那一格必须由够判转为不够判；把「单点红」的判法挪成「两个态才算红」，单点红的格必须由红转不红。

## 九、变异

装置一侧（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`）的行追加在 `crates/mutations.tsv` 末尾，必须红的测试是装置的内联单测（重型，提交时复跑）；比对器与驱动一侧的行进 `research/mutations/e171_three_oracle_collusion.tsv`，执行员经 `research/scripts/mutate.sh` 跑。每条写明它在哪个取样点上改变输出：

| # | 改哪 | 改成什么 | 在哪个取样点上改变输出（会红的那一格） |
|---|---|---|---|
| X1 | 比对器 Q1 的全放行谓词 | 只看 O1、O2，漏掉 O3 | 手造矩阵一行 O1 不红、O2 不红、O3 甲红：原式判「没全放行」，变异判「全放行」 |
| X2 | 比对器读 O3 严格判定 | 欠一句当红 | 手造矩阵一行 O1、O2 不红、O3 严格欠一句：原式在 Q1-甲判「全放行」，变异判「没全放行」 |
| X3 | 比对器读 O3 两列 | 严格与暂代两列对调 | 手造判定文件一态严格「欠一句」、暂代「不在」：原式 Q1-甲不红 Q1-乙红，变异反过来 |
| X4 | 装置 O1-崩溃判官 | `crash_recovery_disagreement` 的结果丢掉（恒为没有不同） | 阳性对照 O1-崩溃判官（读回内容末字节翻一位）：原式红，变异不红 |
| X5 | 装置 O1-对拍 | `judge_and_advance` 的不同丢掉 | 阳性对照 O1-对拍：原式红，变异不红 |
| X6 | 装置 O2 | `Violated` 当 `Holds` | 阳性对照 O2（流一 F 叠已知坏镜像第一条）：原式红，变异不红 |
| X7 | 比对器触达判法 | 只比写流 sha256，不比调用结局与恢复报告 | 手造一对：写流全同、一次调用结局不同：原式触达，变异未触达 |
| X8 | 比对器触达判法 | 恒判触达 | 空变异那一行：原式未触达，变异触达 |
| X9 | 挑法脚本 | m = ⌊n/8⌋（且至少 1）代替 ⌈n/8⌉ | 行数不是 8 的倍数的文件（例 mount.rs 204 行：⌈204/8⌉ = 26，⌊204/8⌋ = 25）：A4 的 117 变少 |
| X10 | 补位规则 | 不跳过已在表里的行 | 单测里一组 n = 3、表里已取下标 0 与 1、下标 0 编不过：原式补到下标 2，变异补到下标 1（重复） |
| X11 | 比对器 Q3 | 「关掉 X」不把 X 列当不红 | 手造矩阵一行只 O2 红：原式关 O2 后报「只靠 O2」，变异报「都还红」 |
| X12 | 比对器 Q1 覆盖门槛 | `D >= 30` 写成 `D > 30` | 手造分母 D = 30、零全放行：原式「没有」够判，变异不够判 |
| X13 | 装置取样 | 批 1 漏掉全持久态（T − 1） | 停机 S2：`states.tsv` 与 E170 导出装置同一条流的不再逐字节相同 |
| X14 | 装置 O1 不适用的判法 | 流二每一态都记不适用 | 第七节末尾的互比：装置报的 O1-崩溃判官适用态数与比对器现算的不等 |
| X15 | 已知坏镜像族的补丁提取 | 只取盘 0 的扇区差 | 改两块盘的条目（例 I-4.8 那一条对每块盘都写）：补丁之后 O2 判红的不变量与在 `MemoryPool` 上直接改坏的不等（停机 S4） |

变异全抓才算这套测量有判别力；没红的照 `.claude/rules/mutation-sampling.md`「没红的三类」分，不直接记盲区或等价。

## 十、失败条款

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F1 | 某一条 oracle 的阳性对照（5.6）不红：那条 oracle 在这条管子上接错了，Q1–Q3 里它那一列整轮作废，不记成「这条 oracle 盲」 | 阳性对照那一行报这一条 oracle 红的处数为 0（基线或空变异任一趟） |
| F2 | 阴性对照（基线、空变异、8 张底态）上任一条 oracle 判红：不是合谋结果，是 oracle 误报或装置与实现对不上——走停机，两边都查（`.claude/rules/implementation-first.md` 第 4 条），既不作废也不当结果 | Q2-阴性或基线那一趟任一列红的处数 > 0（O3 甲以「不在」计，欠一句不计） |
| F3 | P 行（期望出自条款，A2）上 O3 判「在」：三种可能并列交主 agent——执行器照条款读错、E170 设计员推期望推错、条款本身欠一句或写错；这一张在 Q2 里照「O3 放行」算，不因期望可能错而从表里删 | P 行任一张 O3 甲或乙判「在」 |
| F4 | 触达行太少，Q1 的「没有」不够判：进第三段扩表（5.2），扩完仍不够记「不够判」，不记「没有」 | Q1-分母 D < 30 |
| F5 | 「关一条」的矩阵读法不成立：三条 oracle 在装置里互相吃了输出，Q3-甲、Q3-乙作废，改成每个被抓的行三种关法都真跑一遍（Q4-关一条照新跑法重量） | Q3-核 9 次里任一次其余列与不关时不逐字节相同 |
| F6 | 坏实现表考不到 oracle：不进分母的几类（编不过、锚点不唯一、找不到、实现自崩、超时）合计超过所跑行数的一半，表的形态不对——进第三段扩表，主 agent 判要不要改挑法（改挑法是新实验） | 读法 5 那几类合计 > 所跑行数 / 2 |
| F7 | O3 严格跑法到不了这一族：Q1-甲、Q3-甲不报「没有 / 都还红」，改报「O3 严格跑法在 D 行里 ≥ 90% 一态都没判（欠一句或弃权）」，交岔路单 Y3 翻面观测「某一族根本到不了三条」；Q1-乙、Q3-乙照常判 | Q1-O3覆盖 那一行的比例 ≥ 90% |

## 十一、作废条款与停机条款

**作废**（那一次的输出不算结果；修好之后重跑，不改判据）：

| # | 条款 | 触发观测 |
|---|---|---|
| Z1 | 基线不确定 | 基线连跑两次，导出目录九个文件（`impl-verdicts.tsv` 去掉耗时那一列，照 E170 第十二节修订 1 的 V2b）、`oracle-verdicts.tsv`、`call-outcomes.tsv` 任一不逐字节相同 |
| Z2 | O3 不是登记的那一版 | 冻结行 `sha256sum -c` 在开跑前或最后一次跑 O3 之后有一行不 OK：这期间跑的 O3 列全部作废 |
| Z3 | 内存包装自己的结局 | `run-with-memory-cap.sh` 退 250–254：那一行（或那一趟）的输出不算，命令与退出码照记，重跑一次；同一行两次都撞，记进读法 5 的「超时」一类并单列 |
| Z4 | 执行器建的镜像与导出的不是同一个状态 | 执行器任一态 `digest=mismatch` |

**停机**（装置与 `crates/` 对不上，或拿来复用的东西与登记的不是同一个：两边都查，既不作废也不当结果，交主 agent）：

| # | 条款 | 触发观测 |
|---|---|---|
| S1 | 本地常量与 crates 对不上（只在基线那一趟查） | `main` 开头回比任一项不等 |
| S2 | 本装置照抄的两条流与 E170 导出装置的不是同一份 | 同一棵树上 E170 导出装置 `dump` 写出的 `base-sectors.bin`、`ops.tsv`、`ops-payload.bin`、`calls.tsv`、`torn.tsv`、`torn-payload.bin`、`states.tsv`、`quick-tier.tsv` 与本装置基线的任一个 sha256 不等，或 `impl-verdicts.tsv` 去掉耗时列之后不逐字节相同 |
| S3 | O2 两处取法不一致 | 同一态上本装置直接调 `check_pool_image` 判红的不变量集合与 `judge_crash_image(...).pool_checker` 的不等；或坏镜像上本装置的与 E170 导出装置 `bad-impl-verdicts.tsv` 池级 checker 那一列的不等 |
| S4 | 已知坏镜像族不在登记的那一份干净镜像上 | 流一全持久态的镜像摘要（E170 登记 5.3 的算法）与 `build_pool(...).memory_pool()` 的不等；或某一条在补丁之后 O2 不红它的目标不变量；或补丁之后与直接改坏的 `MemoryPool` 上 O2 判红的集合不等；或装置数出的条数不是 41（A8，文件 sha256 也变了的先重数再比） |
| S5 | 推模型那一层与实现对不上 | 基线上 O1 不适用的来源不止 `unmount` 与「对拍判出不同之后」两种；或第七节末尾两份互比的数不等 |
| S6 | E170 导出装置在今天的树上跑不起来 | 它退 3（它自己的 S1：本地常量与 crates 不等，W1 改了 mkfs 之后可能如此）：S2 与 P 行、盲测族那一半跑不了，交主 agent 定是先修 E170 的装置还是本实验另造 |
| S7 | O3 那一版还没有 | E170 照 C651 改过、严格跑法重跑之后没有冻结行可抄：O3 那几列不跑，O1、O2 照跑，这一段交回 |
| S8 | O1 的交法与崩溃注入不是同一种 | 装置里 O1-崩溃判官调的不是 `crash_recovery_disagreement`，或「最新已持久根」不是经 `newest_persisted_root` → `model_root_key` 取的，或读回不是经 `observed_read_back_after_a_crash` 造的（执行员在交回里列装置调这三处的行号） |

**够判停机**：每段交回附问题单四行的表（已够判 / 还差什么 / 剩下的量能不能让它翻面）。四行都够判就停，没跑的量逐条标「够判后未跑」；第一刀之后 Q1 已出现全放行（甲或乙）时，其余 93 行跑不跑由主 agent 对着 Q1「没有」那一边、Q3、Q4 还差什么判。附带量一律够判后不跑。

## 十二、修订

（留空；装置写之后、产物之前由执行员写，只许收严或补臂。第一节读法 8 要的冻结 sha256 行抄在这里。）

### 修订 1：第一段与登记字面不同的四处（第二段开工时补记）

依据：第二段派发提示那一句——「上一段与登记字面不同的四处（Python 变异驱动、阳性对照每条流翻一次、P/N 行取第一套表、E170 导出装置用暂存版本）：主 agent 认，登记第十二节补一条修订写明时点在 O3 列产物之前」。
四处都是第一段做的，那时没有任何一份 O3 列产物；补记写在第二段开工时，下面每一处的做法在第二段的 O3 列产物之前就已经定了。判据、门槛、变异的去留与报告方式都没动。

| # | 登记字面 | 实际做法 | 依据的读数（第一段产物，都在 `research/results/` 下，前缀 `e171-three-oracle-collusion-2026-10-03-s1-`） | 对判据的影响 |
|---|---|---|---|---|
| 1 | 第九节：Python 一侧的变异行经 `research/scripts/mutate.sh` 跑 | 经本实验目录的 `mutation_driver.py` 跑：`mutate.sh` 只起 cargo test，起不了 Python 的单测；形态照 E170 的变异驱动（锚点核唯一、`py_compile` 判无效、单测判抓到、收尾核已还原） | `comparator-mutations.out`：9 条抓到 9、无效 0、没红 0、超时 0 | 没有；变异表的行不变，结局符号与 `mutation-sampling.md` 一致 |
| 2 | 5.6：O1-崩溃判官的阳性对照「批 1 里第一个『实现读回了文件』的态」翻一位 | 每条流各翻一次（两处） | `positive-control-o1-crash-judge.out` 判决行 `o1_crash_judge_red_states=2` | 收严：红的处数更多，对照要的「红」不变 |
| 3 | 5.5：P 行、N 行由 E170 导出装置 `bad-states` 造（盲测族要执行器冻结之后的种子） | P、N 行取 E170 导出装置第一套表（`bad-states`，不是 `bad-states-second-run`）；第一段用占位种子 0000000000000000 造，盲测族那 40 行第一段没用 | `e170-bad-states-placeholder-seed.out`；第二段用冻结的种子重造（`…-s2-bad-states-export.out`），盲测族 40 行用上了 | 没有；P 行 26、N 行 10 的行数与 A9 一致（第二段 `…-s2-comparator.out` 的 `name=a9_a10_a8` 行） |
| 4 | 第三节：导出装置是 `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` 那一份（sha256 8c526293…） | S2 的参照与 P、N 行用 `git show :<那份>` 的暂存版本在草稿目录编的（开工后另一条线改了工作区那份）；工作区版本的 `bad-states` 输出与它逐字节相同 | `e170-pinned-dump.out`、`e170-bad-states-placeholder-seed.out`；基线与 E170 导出装置同一棵树上八个文件逐字节相同（`comparator.out` 里 `name=s2_summary files_compared=18 files_differing=0`） | 没有；S2 的比对对象就是登记钉的那一份 |

### 修订 2：O3 用哪一版（第一节读法 8 要抄的冻结行）

冻结行出自 E170 第 1 次重跑第一段的产物 `research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out` 的第 645–660 行（派发提示点名的行号；只抄这十六行，没读那份产物的结论行；产物的 sha256 `981d29999b66eb2f04dd49ba4ce44acba3b45076f205aa3c50deca9a9055c106`），文件集合是执行器目录 `research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/` 里不含 `comparator*` 与 `mutation_driver*` 的十五个文件，种子是 `85441ae491419411`：

```
name=freeze file=crash_dump.py sha256=7a671b85d8346f2aee45982a888e2111e98495f5570ecfc57a348ee7595d30dd
name=freeze file=crash_image.py sha256=e391dbe8377014258f1978005062d4cb14891b8565ca420a6a5b731ad4387a5f
name=freeze file=crc32c_castagnoli.py sha256=34b9f571bc7d0e9dc32dce56b166536a6c26581b9f4744e7c3a14b0ef69bea6d
name=freeze file=executor_core.py sha256=7bdbc912ee84b4419f6b9bd0d1f5eee870641b118d335072301f3cf9ecc52ec6
name=freeze file=executor_main.py sha256=acdae4aa3f1a6a3fde3dcaa19bddd0ed667d2ccf0855d5040bc416cf7c310a67
name=freeze file=journal_ring.py sha256=9f78d7ed299d51998d8e67d1b6c28c006e8ea778f6d18cb6bb9332afee640679
name=freeze file=on_disk_layout.py sha256=86e1a92cab056ca3dd908be2d25cd2f4a25c9ce7e7c896e4704d58716ec02d51
name=freeze file=replay_prefix.py sha256=ff2f52d4f93910a88dbab37bfeafaecc9d7b58fe0e7c139923e6de771bbff9da
name=freeze file=root_ring.py sha256=6ac949d183997789419d1aeeb448d00bc341b26216ce7c22b6691879a7a2f89e
name=freeze file=spec-gaps.tsv sha256=32ec144a900d09baedc88869f4e392a003866d566e41f1a4df611a67f0ace4f7
name=freeze file=spec_gaps.py sha256=b71a245a49438a24e53f4a3b469ad965c4d9661bcf2d7fe87ac8b90a18c82e8c
name=freeze file=synthetic_pool_for_tests.py sha256=69368e07c746dade040618b7277b27d7a36f557d2fcac86ccf4048a1c8e40f63
name=freeze file=system_configuration.py sha256=a07a280f3c96d8be8cedeaf9f92bddb8a5d7101b50e7e494357c2fdcdb16b394
name=freeze file=test_executor_on_synthetic_pool.py sha256=8b745ac92da2db667f96fb7d756a25faa3a05c9f23d7619dc74a3105ddb435f3
name=freeze file=version_table.py sha256=d407a8c1a53f58aa5a85a6af4eff215844b838d0a48871aaee4b567446a52ba4
name=freeze_seed sha256_prefix16=85441ae491419411
```

核对的做法：O3 列的每一趟（逐行驱动、坏镜像判、比对器）开头与收尾各把这十六行与执行器目录现算的 sha256 与种子比一次（`research/e7-index-bench/src/bin/e171_three_oracle_collusion/frozen_o3.py` 的 `freeze_differences`，等价于 `sha256sum -c` 再加种子与「冻结范围里没有多出文件」两样），结果写进产物的 `name=freeze_check` 行；每份执行器产物自己也带冻结行，比对器逐份与上面这十六行比。
本节抄写时第二段已经把 O3 列的几份产物跑出来了：补记写在它们落盘之后，没有赶在它们之前；这几份产物各自带的开头与收尾两次核对都是 0 处不同，所以 Z2 没有触发。

### 修订 3：装置缺陷修复（取号、暖机出错记成 O1-对拍的红；S5 在逐行那一趟也判）

依据：第三段派发提示的原句——「先修装置缺陷（代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第二节 K4-c 一行、第三节第 4 条；攻方报告 `research/prompts/m3-formal-code-r3-opus-output.md` 第三节）」「登记本来就这么要：第 41 行读法 5 末句……第 42 行读法 6……装置没落地，照登记改装置，不改登记（登记补一条修订写明是装置缺陷、时点在整表产物之前）」。
时点：2026-10-03，装置改完、下面「依据的读数」那几份验证产物跑完之后写，在整表（117 行逐行驱动）的任何产物之前。验证产物只用来查这一处修得对不对，不进 Q1–Q4 的任何数。第一至二节的判据、门槛、失败条款、停机条款一个字没改。

| # | 登记字面 | 修之前的装置 | 现在的装置 | 对判据的影响 |
|---|---|---|---|---|
| 1 | 读法 5 末句：「一次调用返回错误而模型说该成功，不算自崩，是 O1-对拍的红」 | 取号、暖机交回 `Err` 时 `step_acquire_and_warm_up` 直接返回 false、不问模型，那次调用的 O1-对拍一栏留在 `not_modelled`；一行让取号或暖机出错的坏实现三列都可以不红 | 两次调用交回 `Err`（不是 panic）、O1 开着：记成 `disagrees:模型说该成、实现拒了`（两条流不带故障，模型对取号、暖机只有「往前走」一种走法），模型停在这次调用。panic 仍走自崩；关掉 O1（Q3-核）时不记，那一栏写 `not_modelled` | 没动判据。这一类行 O1 从「没红」变「红」（读法 3：O1-对拍按调用算） |
| 2 | 读法 6 与停机 S5：「除这两种来源之外出现不适用，走停机 S5」 | S5 只在基线那一趟判 | 逐行那一趟也判（`s5_checks_of_a_bad_implementation_row`，由 `s5_checks_to_judge` 选）：① 返回了错误的调用（卸载除外：模型没有卸载的答）的 O1-对拍都有红，没有一次留在 `not_modelled` 或「停下之后没比」；② O1-崩溃判官的不适用只许来自模型停在卸载、或停在对拍判出不同的那次调用，来源不合规时数出不合规的态数。两样都回比 0，不是 0 就是停机、装置退 3。阳性对照那一趟照判；关掉 O1 与驱动没走完整（panic）的流不判。基线那一趟的 S5 一个字没动 | 没动判据。停机的行在驱动里 `device_exit=3`，比对器归 `device_stop` 一类、不进分母、单列 |
| 3 | 第七节末尾「装置与比对器各算一遍」、A6、A7 | 比对器没有这两样的回查；A6、A7 数 `crates/mutations.tsv` 里全部改实现的行 | 比对器另算一遍：`name=row_s5` 一行（两个数各按流报）与判决行两个计数 `rows_with_an_errored_call_the_comparison_did_not_judge`、`rows_with_o1_not_applicable_from_another_source`（期望 0）。A6、A7 不数名字以 `E171 装置` 起头的行（装置自己的变异行是测量工具、不是被测的坏实现；为证红而改核心的那一行在 core 下，数进去挑法对 `transaction.rs` 一组的下标整体挪动、与 5.9 的表对不上），另报被去掉的行数 `name=a7_device_rows` | 没动判据；117 行照 5.9 的名字取行，不受影响 |

我读登记读法 5、6 定下的两处细节（登记没写，交主 agent 认）：
- 卸载交回错误不算「没问模型」：模型没有卸载的答，读法 6 写明这次调用不对拍。这是一个留着的盲点：让卸载出错的坏实现，O1-对拍仍是 `not_modelled`、之后的态记不适用（停在卸载是合规来源）。整表里有没有这样的行：`grep -h -E '^E7RESULT name=call .* label=unmount .* status=err' <各行的 device.out>`。
- 模型认可的拒绝（`agrees` 的错误调用）不记红；其后若出现不适用的态，来源既不是卸载也不是对拍不同，走 S5。

依据的读数（命令都在 `research/results/` 下跑）：
- 基线上取号、暖机不出错，所以基线与第一、二段的产物不重出：`grep -h -E '^E7RESULT name=call .*label=(acquire|warm_up) ' e171-three-oracle-collusion-2026-10-03-s1-{baseline-run1,baseline-run2,disable-o1,disable-o2,positive-control-o1-comparison,positive-control-o1-crash-judge,positive-control-o2,row-0-device,row-1-device,row-6-device}.out | sed 's/.*label=\([a-z_]*\) .*status=\([^ ]*\) .*/\1 \2/' | sort | uniq -c` 输出 `20 acquire ok` 与 `20 warm_up ok`（十份产物 × 两条流）；第二段试跑的九行与 `rows-s1` 三行的装置输出同一命令数出 `24 acquire ok`、`24 warm_up ok`（在 gitignored 的 `research/target/e171-dumps/` 下）。新代码只在错误那一支与逐行趟多出的 S5 行上起作用。
- 实核（不只靠推）：新装置的基线（`e171-three-oracle-collusion-2026-10-03-s3-baseline-after-the-fix.out`）的 26 个中性文件与 `baseline-run1` 导出逐字节相同，装置标准输出的 E7RESULT 130 行与 `baseline-run1.stdout` 逐行相同；新装置的空变异行（`…-s3-row-0-device.out`）的 26 个文件与修之前第二段试跑的 `row-0` 导出逐字节相同，装置标准输出除新增的 4 条 S5 回比行与 `name=done` 外 87 行逐行相同。核对脚本的原样输出：`baseline files_compared 26 files_differing 0`、`baseline E7RESULT lines old 130 new 130 identical True`、`empty_mutation_row files_compared 26 files_differing 0`、`empty_mutation_row E7RESULT lines old 87 new_without_s5_and_done 87 new_s5_lines 4 identical True s5_all_hold True`。
- K4-c 攻方造法一行（`crates/mutations.tsv` 末尾第一条 `E171 装置 取号逐盘核…`：`crates/singlefs-core/src/transaction.rs` 取号逐盘核 `<` 写成 `<=`，不带故障的盘上每次取号都拒），经逐行驱动跑一行（`…-s3-k4c-attack-row-device.out`、`…-s3-k4c-check-driver.out`）：`E7RESULT name=call stream=new_pool_file_creation call=0 label=acquire first_operation=0 return_point=0 status=err:DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness o1_comparison=disagrees:模型说该成、实现拒了`；流二同样；判决行 `E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 streams_not_exported=2 o1_comparison_red_calls=2 o1_crash_judge_red_states=0 o2_red_states=0`；驱动 `device_exit=0`。修之前同一行是 `o1_comparison=not_modelled`、`o1_comparison_red_calls=0`（攻方报告第三节第 2 小节读码推的，没在修之前的装置上跑过）。

变异与单测（只补不删）：装置一侧在 `crates/mutations.tsv` 末尾追加 8 行（名字都以 `E171 装置` 起头：1 行核心造法、3 行改装置的记红、4 行改逐行 S5 与它的接线），装置内联单测 24 条变 31 条（`grep -c '#\[test\]'`）；Python 一侧补 X33–X43 共 11 条（表 37 条），单测 76 条变 84 条。这些不是第九节的字面（第九节到 X15）。装置内联单测与装置一侧变异是 checker 档，这一段没跑，只编过（`cargo test --no-run`）。

### 修订 4：收回修订 3 第 2 行 ① 多收的一半；Q3-核关 O1 的比较去掉模型停下的那一行

依据：第五段派发提示的原句——「装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` 第 4964–4969 行逐行 S5 过滤只留 `NotModelled`，`SkippedAfterTheModelStopped` 不算（登记读法 6 原文：对拍判出不同之后的调用不再对拍是合规来源）」「登记补修订 4：修订 3 第 2 行 ① 的「或「停下之后没比」」越过了读法 6 原文，收回到读法 6；照实记一次——第 108 行在修订 3 的字面下停过机，时点在整表产物之后」「Q3-核关 O1 那一趟，`stream-status.tsv` 的 `model_stopped_at_call` 是 O1 自己的状态，算进「关掉的那一列」，`baseline_checks.py` 第 215–236 行的遮罩补上它；登记修订 4 写明，并说清这一格只去掉被关那条 oracle 自己的状态、别的列照比」。
时点：2026-10-03，**在整表产物之后**——117 行逐行驱动（输出根 `research/target/e171-dumps/rows-s3/`）与第 8、32 行的 Q3-核六趟（`rows-s3-q3core-O1`、`-O2`、`-no-executor`）都已跑完，第四段执行员用它们出过一份只放草稿目录、不入库的比对器预览；在 s3 比对器产物、第 108 行重跑与第 108 行 Q3-核三趟之前。这两处都不是收严，是主 agent 定的（第一节读法 6、第六节 Q3-核、第十节 F5 的原文一个字没改）。

| # | 改了什么 | 改之前照字面发生过什么（照实记） | 改之后 | 对判据的影响 |
|---|---|---|---|---|
| 1 | 修订 3 第 2 行 ① 的「没有一次留在 `not_modelled` 或「停下之后没比」」收回成「没有一次留在 `not_modelled`」：读法 6「一次对拍判出不同之后模型不再往前走」与 S5「O1 不适用的来源不止 `unmount` 与「对拍判出不同之后」两种」都把对拍不同之后的调用算合规来源，修订 3 那一半比它们多收 | 第 108 行（`core/unit.rs` 组内下标 0，B6 补齐字节）在修订 3 的字面下停过机：流二第 2 次调用 `publish` 对拍 `disagrees:这一版的文件内容`、模型停下，第 4 次调用 `mount_writable` 交回 `err:Recovery`、记 `skipped_after_the_model_stopped`；装置标准输出第 43 行 `E7RESULT name=s1 stream=fixed_script_through_e_then_unmount check=fixed_script_through_e_then_unmount_o1_every_errored_call_but_the_unmount_is_judged_by_the_comparison local=0 crates=1 holds=false`，判决行 `s1_failed_checks=1`，驱动 `device_exit=3`，比对器归 `device_stop`、不进分母。旧目录改名留在 `research/target/e171-dumps/rows-s3-row-108-before-the-s5-fix/` | 装置逐行 S5 ① 与比对器 `errored_calls_left_unjudged` 只数 `not_modelled` 的错误调用；第 108 行修好后重跑一次，进 `rows-s3` 同一个根 | 读法 6 与 S5 的原文不动；117 行里只有第 108 行有「对拍不同之后的错误调用」（命令与输出在第五段执行员报告），别的 116 行的 S5 读数不变 |
| 2 | Q3-核关 O1 那一趟与不关那一趟的比较，`stream-status.tsv` 去掉 `model_stopped_at_call` 那一行再比（模型停在第几次调用是 O1 自己的状态：关掉 O1 模型就不停）；这一行之外的四行（`ending`、`judged_root_index`、`writes`、`calls`）与别的文件、别的列照比；关 O2 时 `stream-status.tsv` 整份照比 | 草稿预览里第 32 行关 O1 那一趟报 1 处不同：`rows-s3/row-32/dump/fixed_script_through_e_then_unmount/stream-status.tsv` 第 2 行是 `model_stopped_at_call`（制表符）`4`，关 O1 那一趟是 `14`，其余逐字节相同；判决行 `q3_core_runs_with_differences=1`，照 F5 的字面这一趟算「不同」 | `baseline_checks.py` 的 `differences_after_closing_one_oracle` 关 O1 时用 `STREAM_STATUS_KEYS_OF_O1` 去掉那一行 | F5 的比较范围少了一行（放宽，主 agent 定）；被去掉的只有被关那条 oracle 自己写的那一行，它不是 O2、O3 或实现的输出 |

跟着改的工具（不动判据）：比对器 `--driver-output` 可以给几份、每行耗时带出自哪一份驱动输出（整表分两次起、Q3-核各一次）；比对器 config 行写 `segment=s3`。变异与单测：装置一侧在 `crates/mutations.tsv` 末尾追加 1 行（`E171 装置 逐行 S5 又把模型因对拍不同停下之后的错误调用数成「没比」…`），装置内联单测 `the_row_s5_counts_…` 加一格（第 108 行那一形两样都 0），只编过（`cargo test --no-run`）；Python 一侧 X34（「回查不数停下之后没比的错误调用」：它改成的样子就是现在的原式）换成 X44（反方向），补 X45–X47（遮罩三条），表 40 条；单测 84 条变 86 条。

### 修订 5：s3 整表线道 1 有 45 行带着第 112 行的变异跑，整表作废、全部 117 行重跑；逐行驱动的改法

依据：第六段派发提示（主 agent 判第五段执行员报告第七节四问）的原句——「修驱动：`research/e7-index-bench/src/bin/e171_three_oracle_collusion/row_driver.py` 还原改过的文件用 `copy2`（第 257 行）、同步用 `rsync -a`（第 238 行），都把修改时间带回快照那一刻，cargo 按修改时间判新旧，上一行改过的 crate 不重编。改成每一行开跑前，凡是上一行改过、这一行还原的文件都把修改时间置成现在（或照你判更稳的办法：这一行编之前核工作根里每个被改过的文件内容与快照逐字节相同且修改时间晚于上一次编译产物），并加一道核对：编完之后在产物里能认出这一行自己的变异、认不出上一行的（取法你定，写依据）」「整表全部 117 行重跑（不只 43 行）……用修过的驱动、新的工作根与新的输出根（例 `research/target/e171-dumps/rows-s4`）从头跑；Q3-核三趟同样重跑」「用今天的树跑」「登记补修订 5：写明 s3 整表线道 1 有 45 行被带着跑、整表作废重跑、驱动怎么改，时点在 s4 产物之前；s3 的产物留档、不删」。
时点：2026-10-03，在 s3 整表（输出根 `research/target/e171-dumps/rows-s3/`）、s3 比对器产物之后，在 s4 整表（`rows-s4`）、它的 Q3-核三趟与 s4 比对器产物之前。这一条不是收严、也不是补臂，是测量管子的修复与整表重跑，由主 agent 定；第一至十一节的判据、门槛、臂、失败条款、停机条款一个字没改。

| 项 | 内容 |
|---|---|
| 发生了什么（照实记） | s3 整表两条线道，每条线道跨行复用一个编译目录（第一行冷编、之后增量）。线道 1 第 23 个执行的是第 112 行（`crates/singlefs-replay-prefix/src/lib.rs`），此后线道 1 上到第 114 行为止的 45 行，编出来的装置都带着第 112 行的变异：`singlefs-core` 依赖 `singlefs-replay-prefix`，core 的一行只让 core 与它下游重编，replay-prefix 的编译产物留着上一行的变异。反方向不发生：replay-prefix 的一行让依赖它的 core 跟着重编。45 行：2,4,9,11,13,15,19,22,27,30,33,34,36,38,40,42,44,46,48,50,52,54,56,59,60,65,68,70,72,74,76,78,80,82,87,90,92,94,96,98,100,102,104,106,109（第五段执行员查出；其中 33 行与基线的不同只有第 112 行那一形，10 行自己的变异也留下了不同，33、59 两行编不过）。第 8、32 行的 Q3-核六趟是在整表的工作根上接着编的，同样可能带着上一行。整表分两次驱动进程跑（第一刀、其余），第二次接着用第一次的编译目录，进程之间没有任何记录 |
| 机理的实测 | cargo 1.98.0，一个三包的小工作区（`rp` ← `core_probe` ← `app`，与 replay-prefix ← core ← checker-tier 同形；第六段执行员草稿，不入库），`--message-format=json-render-diagnostics` 读 `compiler-artifact` 的 `fresh`，四步输出原样：`step=1_clean exit=0 fresh_rp=False fresh_core=False binary_says=value=11`；`step=2_row_mutates_rp exit=0 fresh_rp=False fresh_core=False binary_says=value=12`；`step=3_next_row_mutates_core_after_copy2_restore exit=0 fresh_rp=True fresh_core=False binary_says=value=22`（干净应是 21：`rp` 用 `copy2` 还原之后没重编，带着上一步的变异）；`step=4_restored_files_touched_now exit=0 fresh_rp=False fresh_core=False binary_says=value=11` |
| 作废什么 | s3 整表 117 行与 s3 的 Q3-核各趟，不再当 Q1、Q3、Q4 的读数；Q1–Q3 与 Q4 的整表挂钟一律照 s4 整表重判、重量。留档不删：`research/target/e171-dumps/rows-s3*`、`rows-s5-clean-check`，`research/results/` 下的 `e171-three-oracle-collusion-2026-10-03-s3-comparator.out`、`…-s3-rows-timing.out`、`…-s3-clean-check-comparator.out` |
| 驱动怎么改 | ① 每条线道一份 `build-record.tsv`：上一次 cargo 编完的时刻（墙上时间，纳秒），与自那以后这条线道上改过、还没有一次编过并核过「重编了」的文件；改坏之前先把这一行要改的文件记进去，编过并核过之后只留这一行的，编不过或没核过就全留着。② 每行同步之后、改坏之前，把记录里的文件修改时间置成现在。③ 编之前核：记录里每个文件与快照逐字节相同、修改时间晚于上一次编完。④ 编时加 `--message-format=json-render-diagnostics`；编完核：记录里的文件与这一行改的文件，所在的每个包 cargo 都报了库目标的 `compiler-artifact` 且 `fresh=false`。③④ 任一不过：这一行不跑装置，驱动不再领新行，退 5。⑤ 开跑前：工作根里已有编译目录而没有记录（旧驱动建的），或快照摘要与上一次在这个工作根里的不同，一行都不跑，退 2。⑥ `--snapshot-from`：整表与 Q3-核各趟从同一份冻住的树拍快照。⑦ 编过的行在 `row-result.tsv` 里带 `packages_checked_rebuilt`；比对器判决行加 `rows_compiled_without_the_rebuild_check`（编过而没带这个键的行数，期望 0） |
| 「认得出这一行的变异、认不出上一行的」怎么取、依据 | 不从装置二进制里认：坏实现表的变异多是改算术与比较，二进制里没有可认的字面。取成两道核对合起来：③ 保证编的那一刻，这一行改的文件之外，上一行改过的文件都是快照的内容；④ 保证这些文件所在的包这一次 cargo 从当时的源文件重编了（`fresh` 是 cargo 对每个编译单元报的「这次没编、用的是编译目录里已有的产物」，机理实测第 3 步它就是 `true`）。两样都过，编出的装置里上一行改过的包只能是快照的内容，这一行改的包带着这一行的变异 |
| 新驱动的实测（真 cargo、真装置） | 冻住今天的树（快照摘要 `73c9487c41a862ad495c16155773e4f8b58e920545762edf2d44c6fd80627dc5`），同一工作根先跑第 112 行、再起一次驱动跑第 2 行（都不跑执行器）：第 2 行 `packages_checked_rebuilt=crates/singlefs-core,crates/singlefs-replay-prefix`，cargo 报 `singlefs_replay_prefix` 的 `fresh=false`；第 2 行对 `baseline-run1` 未触达（`write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false touched=false`，与第五段干净复核那一次相同；s3 带着跑那一次是触达）。这两行只验管子，不进 Q1–Q4 |
| 单测与变异 | 驱动单测 6 条（`test_o3_columns.py` 的 `DriverRebuildTests`：用真的 `rsync -a` 与 `copy2`，假 cargo 照「源文件修改时间晚于上一次编这个包的起点、或依赖的包重编了才重编」判）；比对器单测 1 条；研究一侧变异表补 X48（不置修改时间）、X49（编前核对不看修改时间）、X50（编后不核 `fresh`）、X51（判决行不数没核过的行），四条各有必须红的单测 |
| 树换成今天的 | 与 s3 整表那一版（快照摘要 `149640557b61cdd78cb3f81ecde5505abc8731f70316351fbfb3045488f428ee`）逐文件比，不同的 10 个文件：E171 装置（修订 4 的逐行 S5 过滤与单测一格）、`crates/mutations.tsv`（改与加的行都不在 `crates/singlefs-core/src/` 与 replay-prefix 下，117 行的挑法不变）、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crash_judge_checker.wgsl` 与新加的一条 GPU 测试、`crates/singlefs-harness/src/` 的 `fault_injection.rs`、`mount_transparency.rs`、`reference_filesystem_translation.rs` 与两条 harness 测试。装置非测试代码用到的模块（harness 的 `model`、`model_comparison`、`memory_pool`、`segments`、`sha256`、`hexadecimal`、`numeric_environment_variable`、`lib`，checker-tier 的 `crash`、`crash_facts`、`layer0_progress`、`lib`）按非测试代码里 `crate::` 引用的闭包读码，一个也不引那几个改过的文件（harness 的 `lib.rs` 只用 `pub mod` 把它们编进同一个库，`model_comparison.rs` 唯一一处 `use crate::history` 在 `#[cfg(test)]` 里）；实现（core、replay-prefix、format）与 O2（`singlefs-checker`）没有不同。所以这几处不改 O1、O2、实现在 117 行上的行为（读码判的）；整表开跑前驱动脚本另核冻住的树与读码时的那一份逐文件相同、空变异一行对 `baseline-run1` 未触达，两样不过就不跑 |

## 十三、读过的文件与跑过的命令

**读过的文件**（行号区间；grep 命中行也算读过）：

- `.claude/agent-common.md` 全文；`.claude/singlefs-ai-sop/rules/test-discipline.md` 1–122；`.claude/rules/three-way-inference.md` 1–174；`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 1–176；`.claude/rules/mutation-sampling.md` 1–96；`.claude/rules/implementation-first.md` 1–20。
- `research/prompts/three-oracle-collusion-questions.md` 1–12；`research/prompts/m3-formal-7-forks.md` 1–83；`research/prompts/m3-c651-r1-main-verification.md` 1–83。
- `records/2026-10-01-里程碑三形式化接入.md`：`grep -n '^#'`；387–420 前后约 2 KB（工具截断的预览）、521–536（每行截到 3000 字符）。
- `.claude/kb/decisions/13-验证路线.md`：开工时的一版 1–24、109–151、329–341（那一版已定项 6 在第 109 行、已定项 15 在第 329 行）；第 12 批写回之后的一版 `grep -n '^#'`、14、130–174、354–366（第二节整抄的那几段）。
- `.claude/kb/checks-owed.md`：`grep -n '^| C407'` 第 331 行。
- `.claude/rules/verification.md`：`grep -n '^#'`、`grep -n '快档\|挂钟\|分钟\|秒'` 命中行（第 17、22、25–27、30、34、44、98、110 行）、30–40。
- `.claude/gate.d/checker-tier-crates-mutation-replay.sh` 第 45–48 行、`.claude/gate.d/checker-tier-formal-verification.sh` 第 24、36、48、114 行（`grep -i 'timeout\|预算\|budget\|分钟\|minutes'` 命中）；`.claude/gate.d/checker-tier-layer0-replay.sh` 同一 grep 零命中；`.claude/gate.d/checker-independence-and-sync.sh` 第 40 行。
- `research/scripts/claim-experiment.sh` 1–60；`research/scripts/quote-kb.py` 1–40；`research/scripts/replace-once.py` 1–25。
- `research/prompts/e170-preregistration.md`：`grep -n '^#'`；1–35、378–550、682–690；`grep -n 'P6\|P7'`（零命中）；477–515 里 `^\| P[0-9]`、`^\| N[0-9]` 的行；第 512、533 行（grep 命中）。没读第 551–681 行（第六至十一节）。
- `crates/singlefs-checker-tier/src/crash.rs`：`grep -n` 命中行（第 3、51、108、413–493、744、760、931–971、1061、1099、1132–1267、1417–1458、4613–4700）；925–1060、1130–1300。
- `crates/singlefs-harness/src/model.rs`：`grep -n` 命中行（第 1–17、33–1694 的公开项、2246）；425–460、537–556、1694–1700、2200–2330；`grep -c 'unmount'`、`grep -ci 'unmount\|卸载'` 都是 0。
- `crates/singlefs-harness/src/model_comparison.rs` 1–40 与公开函数的 `grep -n`；`crates/singlefs-harness/src/history.rs` 376–420、905–920、1055–1065 与 `grep -n` 命中行（第 446、843、916、1059、2230、3493、3958、4390、4407）。
- `crates/singlefs-checker-tier/src/crash_injection.rs` 1–60（每行截到 400 字符）、公开项的 `grep -n`、990–1030。
- `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` 1–60、函数与常量的 `grep -n`、第 387–517 行里 `record_call` 与字符串的命中行、第 15、16、210、253、387、696、1269、1857、1865、1936、3712–3739 行。
- `crates/mutations.tsv` 1–5 与 `awk` 计数；`crates/singlefs-harness/tests/checker_known_bad_images.rs` 1–30、100–135、747–760、780–800、2814–2860、函数名的 `grep -n`、747–1294 的几条 `grep`。
- `research/e7-index-bench/src/bin/e170_o3_spec_executor/`：`ls -la`；`executor_main.py` 1–40 与 `grep -n` 命中行（第 23、29、45、48、54、102–110、213–220、239、268）；`executor_core.py` 70–80 与第 69、82、105、124、134 行；`comparator_q3_bad_states.py` 1–25；`spec_gaps.py` 第 4、5、60 行；`spec-gaps.tsv` 1–3 行与 `grep -c .`。
- `crates/singlefs-checker/src/walk.rs` 第 6487 行；`crates/singlefs-core/src/recovery.rs` 第 4330 行；`crates/singlefs-harness/src/memory_pool.rs` 第 812 行；`crates/singlefs-harness/tests/common/mod.rs` 第 677 行（都是 `grep -n` 命中行）。
- `research/results/`：只 `ls | grep e170` 看了文件名，没打开任何一份。`.claude/kb/experiments/` 与 `.claude/kb/experiments.md` 没读。

**跑过的命令**（原样；输出是 2026-10-03 这一次真跑出来的）：

```
$ bash research/scripts/claim-experiment.sh --next
E171
$ bash research/scripts/claim-experiment.sh E171 三oracle合谋测试
  ✓ 占住 E171：/home/fy5090/code/singlefs/research/prompts/e171-preregistration.md
$ python3 research/scripts/quote-kb.py /tmp/claude-1000/collusion-designer/quotes.md '.claude/kb/decisions/13-验证路线.md~^\| 6 \| \*\*三个互相独立的 oracle' '.claude/kb/decisions/13-验证路线.md@#### 已定项 6：三个互相独立的 oracle' '.claude/kb/decisions/13-验证路线.md@#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像' '.claude/kb/decisions/13-验证路线.md@#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑' '.claude/kb/checks-owed.md~^\| C407 \|' 'research/prompts/m3-formal-7-forks.md~^\| Y3 \|' 'research/prompts/m3-formal-7-forks.md@### Y3 三 oracle 合谋测试'
  ✓ 7 段整抄进 /tmp/claude-1000/collusion-designer/quotes.md，回读逐字节一致
# 第二节追加之后：tail -n +<追加前行数+1> 登记 | cmp - quotes.md → appended-quotes-identical
# 5.9 追加之后：tail -n 119 登记 | cmp - table-117.md → table-identical
$ sha256sum research/e7-index-bench/src/bin/e170_o3_spec_executor/*.py research/e7-index-bench/src/bin/e170_o3_spec_executor/spec-gaps.tsv
# 输出照抄在第三节代码块（20 行）
$ sha256sum crates/mutations.tsv crates/singlefs-harness/tests/checker_known_bad_images.rs crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs research/prompts/e170-preregistration.md research/prompts/three-oracle-collusion-questions.md research/prompts/m3-formal-7-forks.md research/prompts/m3-c651-r1-main-verification.md
40c75af965ea7e63d9bd32356dc3d290bb99616f5a9aa1ddd1bd703aa28da30b  crates/mutations.tsv
e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820  crates/singlefs-harness/tests/checker_known_bad_images.rs
8c5262935b45ebf119fd3f0866f6451685864b4e50fb25520d7f58942e63fb64  crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
89abbb37206e706786743958b2516ca734420ffd3fb99ad8311bddb895e10bb7  research/prompts/e170-preregistration.md
31d831eba7ae5240e031725721278611b369d8fa9c71c4bfcd71a27a7b392b2a  research/prompts/three-oracle-collusion-questions.md
9e027185cf535868e544336176e8ba9bced1f50f0de1d6abcadd10356e4dffca  research/prompts/m3-formal-7-forks.md
da11737f371c91f83d28ca4961e6b6145587ca4fae3dd941201fa979dacbb18b  research/prompts/m3-c651-r1-main-verification.md
$ sha256sum /tmp/claude-1000/collusion-designer/select_bad_implementations.py
4032a0e06b614a66c67df5fe5eae82e15f3399177655ee4230e50aee4b7785b4  /tmp/claude-1000/collusion-designer/select_bad_implementations.py
$ python3 /tmp/claude-1000/collusion-designer/select_bad_implementations.py crates/mutations.tsv 8 | wc -l
117
$ python3 /tmp/claude-1000/collusion-designer/select_bad_implementations.py crates/mutations.tsv 8 --first-slice | wc -l
24
$ python3 /tmp/claude-1000/collusion-designer/select_bad_implementations.py crates/mutations.tsv 4 | wc -l
220
$ comm -12 k4.names k8.names | wc -l; comm -23 k4.names k8.names | wc -l; comm -13 k4.names k8.names | wc -l   # k4.names / k8.names = 两张表第 4 列 sort
56
164
61
$ awk -F'\t' '$0 !~ /^#/ && NF==6 && ($2 ~ /^crates\/singlefs-core\/src\// || $2=="crates/singlefs-replay-prefix/src/lib.rs")' crates/mutations.tsv | wc -l
844
$ awk 'NR>=747 && NR<=1294' crates/singlefs-harness/tests/checker_known_bad_images.rs | grep -cE 'Box::new\('
41
$ awk 'NR>=747 && NR<=1294' crates/singlefs-harness/tests/checker_known_bad_images.rs | grep -cE '^            "I-[0-9]+\.[0-9]+",?$'
38
$ grep -rln '合谋\|collusion' crates .claude/gate.d --include=*.rs --include=*.sh --include=*.tsv
.claude/gate.d/checker-independence-and-sync.sh
$ grep -c 'unmount' crates/singlefs-harness/src/model.rs
0
$ grep -n 'P6\|P7' research/prompts/e170-preregistration.md | wc -l
0
$ grep -rn -i 'timeout\|预算\|budget\|分钟\|minutes' .claude/gate.d/checker-tier-layer0-replay.sh | wc -l
0
```
