# sweep-acceptance-v3-judge-8：阶段同步逐行判（组 F8、F9、F12、G3、H4）

阶段名：sweep-acceptance-v3-judge-8。改动范围：基准提交 `b1c8cef~1`（`be22bde`），结束提交 `00c9d4f`。
候选表：`research/prompts/sweep-acceptance-2026-09-18-v2-candidates.tsv`；事实表：同目录 `-v2-facts.tsv`。
分给本轮的组：F8（81 行）、F9（31 行）、F12（5 行）、G3（16 行）、H4（125 行），合计 258 行，逐行判过，未整组放行、未抽样。

读上下文一律 `git show 00c9d4f:路径`，不读工作区文件。

## 各组事实（摘自事实表）

- F8：旧＝「invariants.md 里 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽） 写着「未实现、要第二个事务」」；新＝「步 6 部分落地：层 0 两条流的实况、必红八条七条已有，checker 新接 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽）」；检索词＝`I-7.4（近 K 代块未被复用）`。
- F9：旧＝「无 步 7（性能表出口）的产物」；新＝「步 7 跑完：E152（按里程碑对比六家文件系统的文件性能） 第三次正式跑只重跑 singlefs 一臂，5 轮一次过，判据全中」；检索词＝`E152（按里程碑对比六家文件系统的文件性能）`。
- F12：旧＝「并行线一、二每单元净荷预想是 32635（数据单元预留位头 133）」；新＝「提交 d2aeb7d 之后按新基准重审：并行线一、二每单元净荷从 32635 改成 32634（数据单元预留位头是 134 不是 133）」；检索词＝`32635`。
- G3：旧＝「verification-build.md 口径：三样东西是零代码，crates/ 不存在」；新＝「verification-build.md 按代码现状改写：crates/ 下四个 crate 已实现，第一个事务从 mkfs 到恢复跑通，层 0 崩溃点重放由门禁 54 号覆盖两条流」；检索词＝`crates&&(不存在|零代码|四个crate)`。
- H4：旧＝「singlefs-ai-sop 管 QEMU 与 herd7 相关记录」；新＝「singlefs-ai-sop 0.0.50 移交：不再管 QEMU 与 herd7，相关记录原样交到 .claude/handover/qemu-herd7/」；检索词＝`QEMU|herd7`。

## 判法摘要

- **F12**（5 行，全部命中在 `experiments/117-…md` 与 `experiments/140-…md`）：两份都是带日期的「已跑」实验页，表格与原始产物行是特定臂（`resv12`/`h133`，头 133）在那一次跑上的计算与实测记录，是「那一次发生的事」，不是「现在头宽是多少」的现状断言 —— 判**事件句不改**。
- **F8**（81 行）：检索词 `I-7.4（近 K 代块未被复用）` 是全仓通用的判据交叉引用写法，命中的绝大多数是别的 checks-owed / decisions / experiments 条目在讨论 I-7.4 的**语义**（K 代怎么定义、根环深度、回退候选集）而不是它**有没有 checker**；这些一律判**不相干**。真正描述「checker 现在接了 I-7.4/I-4.8」这件事的，只有 6 行：`invariants.md:12`、`milestone/02-second-txn.md:222`、`milestone/02-second-txn.md:357`、`verification-build.md:139`、`verification-build.md:153`、`CLAUDE.md:97`，均已是移交/落地当天写的现状记录、与新事实一致，判**事件句不改**。
- **F9**（31 行）：检索词是实验编号 `E152`，命中散在里程碑步骤描述、vm-harness.md 装置说明、CLAUDE.md 索引等；只有明确複述「第三/四次正式跑」「2026-09-17 跑完」这类已完成记录的行判**事件句不改**（`experiments.md:170`、`layout/02-second-txn.md:17`、`milestone/02-second-txn.md:245`、`milestone/02-second-txn.md:513`、`vm-harness.md:144/152/167/168`、`records/2026-09-16-…:635`），其余（并行线规划、装置技术细节、验收标准条目、通用示例）判**不相干**。
- **G3**（16 行）：检索词 `crates&&(不存在|零代码|四个crate)`；`checks-owed.md:93`、`:173` 是 `&&` 假阳性（「不存在」分别指「上界不存在」「R 内不存在这批落点」，与 crates/ 是否存在无关），`decisions/13-验证路线.md:62` 说的是 O1/O3 oracle 仍零代码（本身已经写对，不是 G3 要盯的那句话）——三行判**不相干**；其余 13 行都是移交/落地当天写的现状记录，与新事实一致，判**事件句不改**。
- **H4**（125 行）：检索词 `QEMU|herd7` 命中绝大多数是技术性提到这两个工具（实验装置、门禁阶段说明、历史调研记录），与「谁在管理这两样工具」无关，判**不相干**；真正记录移交这件事本身的只有 `handover/qemu-herd7/README.md` 的 12 行（1、3、14、15、16、19、21、22、23、24、26、32）与 `implementation-workflow.md` 的 2 行（4、32），判**事件句不改**。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F8 | `.claude/kb/checks-owed.md:87` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/checks-owed.md:119` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/checks-owed.md:197` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/checks-owed.md:263` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/checks-owed.md:264` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/checks-owed.md:292` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/checks-owed.md:351` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/checks-owed.md:385` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions.md:275` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/01-数据可移动性-反向索引.md:105` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/03-空间分配.md:180` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/16-发布语义.md:207` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/16-发布语义.md:209` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/16-发布语义.md:381` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/16-发布语义.md:413` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:25` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:98` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:101` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:183` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:194` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:198` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:214` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:227` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:258` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:316` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:385` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:452` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:454` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:716` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/22-单元原子性怎么合成.md:968` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/23-journal的角色与格式.md:603` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/23-journal的角色与格式.md:854` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/23-journal的角色与格式.md:1245` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/decisions/26-后台整理与放置回收.md:387` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments.md:171` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/123-K与回退深度的二选一各要付什么.md:8` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/123-K与回退深度的二选一各要付什么.md:17` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/123-K与回退深度的二选一各要付什么.md:23` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:6` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:72` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:74` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/153-账本形态与环上有洞的代价.md:108` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:311` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/33-两条钉块规则扣的不是同一批.md:24` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/33-两条钉块规则扣的不是同一批.md:59` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/33-两条钉块规则扣的不是同一批.md:67` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/33-两条钉块规则扣的不是同一批.md:96` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/41-根环槽几何2x4x256.md:14` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/41-根环槽几何2x4x256.md:81` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/41-根环槽几何2x4x256.md:85` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/47-根环失败域的损失.md:30` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/47-根环失败域的损失.md:32` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/47-根环失败域的损失.md:107` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/50-根环槽数的上下界.md:16` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/50-根环槽数的上下界.md:24` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/50-根环槽数的上下界.md:81` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/78-重放的起点.md:5` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/92-最坏块重用延迟需求.md:3` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/92-最坏块重用延迟需求.md:14` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/92-最坏块重用延迟需求.md:37` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/92-最坏块重用延迟需求.md:54` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/92-最坏块重用延迟需求.md:111` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/92-最坏块重用延迟需求.md:115` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/experiments/92-最坏块重用延迟需求.md:144` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/invariants.md:12` | 事件句不改 | 状态段已记2026-09-17步6加I-7.4与I-4.8，与新事实一致 |
| F8 | `.claude/kb/invariants.md:99` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/milestone/02-second-txn.md:40` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/milestone/02-second-txn.md:186` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/milestone/02-second-txn.md:210` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/milestone/02-second-txn.md:222` | 事件句不改 | 这正是步骤6现状段checker新接I-7.4/I-4.8那句的来源 |
| F8 | `.claude/kb/milestone/02-second-txn.md:229` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/milestone/02-second-txn.md:338` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/milestone/02-second-txn.md:357` | 事件句不改 | 收口表第40行记录checker已用I-7.4/I-4.8判红的实测 |
| F8 | `.claude/kb/prior-art.md:657` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/prior-art.md:693` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/tooling.md:304` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `.claude/kb/verification-build.md:139` | 事件句不改 | 已写明29条含I-7.4/I-4.8，是2026-09-14起接上的现状记录 |
| F8 | `.claude/kb/verification-build.md:153` | 事件句不改 | 必红用例表记录E150已验证I-7.4会红，是已完成的验证 |
| F8 | `CLAUDE.md:97` | 事件句不改 | 已写明checker判29条含I-7.4与I-4.8，与新事实一致 |
| F8 | `records/2026-09-13-总审核.md:161` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F8 | `records/2026-09-16-总审核回扫核查.md:29` | 不相干 | 这一行讨论I-7.4的语义或设计沿革，不是checker有没有接入I-7.4/I-4.8 |
| F9 | `.claude/agent-common.md:41` | 不相干 | 举例说明长任务类型，与步骤7有没有产物无关 |
| F9 | `.claude/kb/experiments.md:170` | 事件句不改 | 已经写明2026-09-17第三四次跑完，与新事实一致 |
| F9 | `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:179` | 不相干 | 说的是CoW文件系统写了什么没量，与步骤7有没有产物是两件事 |
| F9 | `.claude/kb/layout/02-second-txn.md:17` | 事件句不改 | 标题记录2026-09-17第三次跑的结果，是完成后的现状 |
| F9 | `.claude/kb/milestone/02-second-txn.md:24` | 不相干 | 只是说明步骤7的出口是性能表，不是断言有没有产物 |
| F9 | `.claude/kb/milestone/02-second-txn.md:245` | 事件句不改 | 这正是步骤7跑完那句现状的来源 |
| F9 | `.claude/kb/milestone/02-second-txn.md:247` | 不相干 | 设想实现描述怎么做，不是断言产物有没有 |
| F9 | `.claude/kb/milestone/02-second-txn.md:252` | 不相干 | 预想的细节是写在跑之前的规划说明，不主张现在没量 |
| F9 | `.claude/kb/milestone/02-second-txn.md:266` | 不相干 | 说的是增补1的现状，不是步骤7有没有产物 |
| F9 | `.claude/kb/milestone/02-second-txn.md:282` | 不相干 | 描述增补1第一件怎么实现，不主张步骤7没有产物 |
| F9 | `.claude/kb/milestone/02-second-txn.md:285` | 不相干 | 增补1候选改法的方法说明，与步骤7有没有产物无关 |
| F9 | `.claude/kb/milestone/02-second-txn.md:289` | 不相干 | 讨论已定项5的证据时点，不是步骤7有没有产物 |
| F9 | `.claude/kb/milestone/02-second-txn.md:293` | 不相干 | 增补1验收标准条目，不是步骤7的产物状态 |
| F9 | `.claude/kb/milestone/02-second-txn.md:296` | 不相干 | 增补1验收标准，提到重跑E152是未来动作，不是步骤7 |
| F9 | `.claude/kb/milestone/02-second-txn.md:429` | 不相干 | 并行线小标题提到E152缺的能力，不是步骤7产物状态 |
| F9 | `.claude/kb/milestone/02-second-txn.md:432` | 不相干 | 描述并行线怎么接入E152，不是步骤7产物状态 |
| F9 | `.claude/kb/milestone/02-second-txn.md:470` | 不相干 | 描述并行线二要加的能力缺口，不是步骤7产物状态 |
| F9 | `.claude/kb/milestone/02-second-txn.md:513` | 事件句不改 | 记录已经查清并修好的计时打包问题，是已发生的事 |
| F9 | `.claude/kb/milestone/02-second-txn.md:517` | 不相干 | 说明可比维度与报告口径，不是步骤7有没有产物 |
| F9 | `.claude/kb/vm-harness.md:48` | 不相干 | 举例说明取内核的方法，不是步骤7产物状态 |
| F9 | `.claude/kb/vm-harness.md:135` | 不相干 | 小标题说明E152加的两个开关，与步骤7产物无关 |
| F9 | `.claude/kb/vm-harness.md:139` | 不相干 | 描述VM_EXTRA_ROOT的用途，与步骤7产物无关 |
| F9 | `.claude/kb/vm-harness.md:144` | 事件句不改 | 记录2026-09-15第一次冒烟跑的实测结果 |
| F9 | `.claude/kb/vm-harness.md:152` | 事件句不改 | 记录2026-09-17第三四次正式跑发现的计时问题 |
| F9 | `.claude/kb/vm-harness.md:158` | 不相干 | 描述装置怎么读输出的设计做法，不是步骤7产物状态 |
| F9 | `.claude/kb/vm-harness.md:159` | 不相干 | 描述汇总时的自检做法，不是步骤7产物状态 |
| F9 | `.claude/kb/vm-harness.md:167` | 事件句不改 | 记录2026-09-17第三四次正式跑量到的具体读数 |
| F9 | `.claude/kb/vm-harness.md:168` | 事件句不改 | 引用第一次结果那一条，是已完成实验的结论 |
| F9 | `CLAUDE.md:82` | 不相干 | 只说明perf-by-milestone.md的数据来源，不是步骤7产物状态 |
| F9 | `records/2026-09-16-subagent拆分提案.md:154` | 不相干 | 把E152当重负载举例，不是步骤7产物状态 |
| F9 | `records/2026-09-16-subagent拆分提案.md:635` | 事件句不改 | 记录另一个会话2026-09-17改了E152装置这件事 |
| F12 | `.claude/kb/experiments/117-预留位算进去之后的头宽与几何.md:48` | 事件句不改 | 记录的是E117/E140在头133下算出的历史结果，不是现状断言 |
| F12 | `.claude/kb/experiments/140-单元含头逼出的跨单元页与读侧拷贝.md:7` | 事件句不改 | 记录的是E117/E140在头133下算出的历史结果，不是现状断言 |
| F12 | `.claude/kb/experiments/140-单元含头逼出的跨单元页与读侧拷贝.md:46` | 事件句不改 | 记录的是E117/E140在头133下算出的历史结果，不是现状断言 |
| F12 | `.claude/kb/experiments/140-单元含头逼出的跨单元页与读侧拷贝.md:47` | 事件句不改 | 记录的是E117/E140在头133下算出的历史结果，不是现状断言 |
| F12 | `.claude/kb/experiments/140-单元含头逼出的跨单元页与读侧拷贝.md:87` | 事件句不改 | 记录的是E117/E140在头133下算出的历史结果，不是现状断言 |
| G3 | `.claude/kb/checks-owed.md:35` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `.claude/kb/checks-owed.md:93` | 不相干 | &&命中的「不存在」说的是「上界不存在」，不是crates/不存在 |
| G3 | `.claude/kb/checks-owed.md:173` | 不相干 | &&命中的「不存在」说的是「R内不存在这批落点」，不是crates/不存在 |
| G3 | `.claude/kb/decisions/13-验证路线.md:62` | 不相干 | 说的是O1/O3仍零代码，是另一件已经写对的事，不是这句要改的话 |
| G3 | `.claude/kb/decisions/15-格式冻结政策.md:247` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `.claude/kb/invariants.md:170` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `.claude/kb/milestone/01-first-txn.md:19` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `.claude/kb/milestone/01-first-txn.md:43` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `.claude/kb/verification-build.md:4` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `.claude/kb/verification-build.md:219` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `.claude/rules/implementation-first.md:10` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `CLAUDE.md:6` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `records/2026-08-29-审计轮-外部引用复核与阻塞集.md:11` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `records/2026-09-03-验证三件套落地调研.md:13` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `records/2026-09-09-D15两格定案.md:27` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| G3 | `records/2026-09-09-D15两格定案.md:68` | 事件句不改 | 这是移交/落地当天写的现状记录，与crates/已实现四个crate的新事实一致 |
| H4 | `.claude/agents/crash-verifier.md:3` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/agents/crash-verifier.md:13` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/agents/crash-verifier.md:27` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/agents/implementation-writer.md:46` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/handover/qemu-herd7/README.md:1` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:3` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:14` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:15` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:16` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:19` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:21` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:22` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:23` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:24` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:26` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/handover/qemu-herd7/README.md:32` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/kb/INDEX.md:17` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/checks-owed.md:20` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/checks-owed.md:29` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/checks-owed.md:240` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/02-RAID条带策略.md:341` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/08-核心索引结构.md:304` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/09-加密.md:422` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/12-目标介质.md:160` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/12-目标介质.md:200` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/12-目标介质.md:277` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/13-验证路线.md:363` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/17-实现分层与第三方管道.md:122` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/17-实现分层与第三方管道.md:128` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/22-单元原子性怎么合成.md:881` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/decisions/22-单元原子性怎么合成.md:882` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments.md:22` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments.md:33` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/04-范围重建vs逐项修改的交叉点.md:1` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/04-范围重建vs逐项修改的交叉点.md:5` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/04-范围重建vs逐项修改的交叉点.md:12` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/06-加密算法选型.md:7` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/07-离线索引harness.md:159` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/07-离线索引harness.md:164` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/07-离线索引harness.md:167` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/07-离线索引harness.md:190` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/07-离线索引harness.md:208` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/07-离线索引harness.md:210` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/07-离线索引harness.md:229` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/07-离线索引harness.md:258` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/07-离线索引harness.md:283` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/12-攒批的顺序追加vs不攒批的随机页读改写.md:273` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/142-第一个事务的干跑.md:154` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/15-异构设备几何的模拟是否成立.md:3` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/15-异构设备几何的模拟是否成立.md:7` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/15-异构设备几何的模拟是否成立.md:8` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/15-异构设备几何的模拟是否成立.md:11` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/15-异构设备几何的模拟是否成立.md:39` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/15-异构设备几何的模拟是否成立.md:49` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:13` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/30-交叉点的写放大那一半.md:4` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/34-根环槽几何.md:175` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/43-扩展点字节上限.md:73` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:62` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/56-消息缓冲的收益vsε.md:330` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/60-加盘之后不搬数据的代价.md:110` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/61-反向链hash算法的均匀性.md:89` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/69-反向索引取权威态的增量维护代价.md:79` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/72-设备描述符表的挂载复算.md:78` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/experiments/72-设备描述符表的挂载复算.md:90` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/milestone/01-first-txn.md:217` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/milestone/02-second-txn.md:222` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/milestone/02-second-txn.md:237` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/milestone/02-second-txn.md:342` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/milestone/02-second-txn.md:451` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/milestone/02-second-txn.md:512` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/tooling.md:478` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/tooling.md:503` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/tooling.md:508` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/tooling.md:522` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/verification-build.md:4` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/verification-build.md:5` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/verification-build.md:15` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/verification-build.md:165` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/verification-build.md:169` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/verification-build.md:201` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/verification-build.md:202` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/kb/vm-harness.md:123` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/main-agent.md:38` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/rules/implementation-workflow.md:1` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/rules/implementation-workflow.md:4` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/rules/implementation-workflow.md:30` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/rules/implementation-workflow.md:32` | 事件句不改 | 这是2026-09-16移交文档自身的记录，描述移交做了什么，与新事实一致 |
| H4 | `.claude/rules/implementation-workflow.md:36` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/rules/implementation-workflow.md:37` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `.claude/skills/crash-test/SKILL.md:3` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `README.md:63` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `README.md:67` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `README.md:68` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `README.md:83` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `README.md:87` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `README.md:88` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `README.md:127` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-08-25-立项.md:59` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-08-25-立项.md:61` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-08-26-文献扫描与第一次实测.md:7` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-08-27-介质分支定案.md:13` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-08-27-介质分支定案.md:30` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-08-28-原子性与日志定案.md:13` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-08-29-加密与日志定案.md:13` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-08-29-审计轮-外部引用复核与阻塞集.md:13` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-08-30-三轮攻击轮.md:99` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-03-验证三件套落地调研.md:11` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-03-验证三件套落地调研.md:22` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-03-验证三件套落地调研.md:33` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-09-D22项6项9尝试定案.md:45` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-09-D22项6项9尝试定案.md:104` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-09-D22项6项9尝试定案.md:105` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-13-总审核.md:425` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-13-总审核.md:427` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-13-总审核.md:429` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-13-总审核.md:431` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-13-总审核.md:439` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-16-subagent拆分提案.md:99` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-16-subagent拆分提案.md:154` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-16-subagent拆分提案.md:311` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-16-subagent拆分提案.md:382` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-16-subagent拆分提案.md:409` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-16-subagent拆分提案.md:673` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |
| H4 | `records/2026-09-17-CLAUDE.md去冗余.md:19` | 不相干 | 提到QEMU/herd7说的是技术用途或历史记录，不是谁在管理这两样工具 |

## 没做什么

- 只判分给本轮的组（F8、F9、F12、G3、H4），候选表里其余组不归本轮，未看。
- 未改任何被搜到的文件，未改 `stale=` 标记；只判不改。
- 未读 `research/scripts/stale-candidates.py` 的常量区，只照文件头用法跑命令；未读 `research/prompts/` 下 `knowledge-rot-*`、`c382-*`、`sweep-acceptance-2026-09-18-*` 里除候选表、事实表之外的文件；未读 `records/2026-09-18-回扫员可靠性.md`。
- F8/H4 的检索词是全仓通用的判据/工具名交叉引用，命中里绝大多数与本轮事实无关（语义讨论、技术细节），已逐条读上下文核实，不是抽样判断。
