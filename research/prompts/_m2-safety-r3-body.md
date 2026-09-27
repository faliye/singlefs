# 里程碑二收尾：准入把池卡住，安全设计轮第三轮正文（2026-09-26）

<!-- doc-lint:not-numbers S4 S4b S6 Z19 A1 A2 A3 A4 A1n A3ge A3gt A3d ND P1 P2 P3 P4 -->

## 一、这一轮要判什么

第三轮，也是最后一轮。第二轮判决 `research/prompts/m2-safety-r2-main-verification.md` 整份进材料：A1–A4 出局，ND 单独用出局；各条臂共用的病根是 C283（准入失败时不先推发布就报 ENOSPC）。用户 2026-09-25 定：alloc-basis 岔路 2「删，配需求改法」（ND+A1n，零轮），C283「带上，里程碑二做」（`records/2026-09-24-里程碑二收尾调度.md` 第三节「安全设计轮第二轮判完、用户两问」那一行）。回退改成挂着时向前发布、正常卸载抬 F 到现行那一版（B1）已经实现，这一轮在新代码上量。攻击面不重复前两轮（第一轮：S4 的归因、两条路径口径同不同；第二轮：A1 的重复计费、A3 的豁免被滥用、A4 放行之后卡住、近满删了再写、崩在删除中间、多文件碎片化）。

**臂，每条写成式子**（`R` = 副本数，第一版每个单元落池里每一块盘；`d` 是一块盘；槽数按盘计）：

| 臂 | 可用(d) | 需求(d) | 判不过时 |
|---|---|---|---|
| T（今天） | 容量(d) − 已分配(d) − 不可回收(d) − defer 待释放(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − [待删 + 已承诺预留 + checkpoint 保留池] ÷ R（`crates/singlefs-core/src/admission.rs` 的 `available_on_each_device`） | Σ 这次发布重写的角色里 `space_budget_of_role` 为 `Demand` 的 `span_slots`（`admission.rs` 的 `demand_of_the_roles_on_each_device`） | 报拒（发布 `PublishError::SpaceAdmissionRefused`，挂载 `MountError::SpaceAdmissionRefusedBeforeAcquisition`） |
| P1（T + C283） | 同 T | 同 T | 先推一次空发布抬 F，F 取 D16（发布语义） 已定项 1「抬 F 的上限」那一行的准入上限（`.claude/kb/decisions/16-发布语义.md` 第 36 行），推完重判；一次准入最多 8 次发布（同文件第 41 行，B = 4 + 2 k_tol，k_tol = 2），做满仍不过才报拒；写行那次发布之前不推（同一行的括注） |
| P2（ND+A1n） | T 的式子去掉「− defer 待释放(d)」那一项 | Σ 这次发布重写的**全部**角色的 `span_slots`（不按 `space_budget_of_role` 过滤：普通分配加这次写出的固定点），不加换下的槽 | 报拒 |
| P3（ND+A1n + C283，用户定的组合） | 同 P2 | 同 P2 | 同 P1 |

P1、P2 是对照臂：P3 与 P1 比，量出 ND+A1n 在 C283 之上多换来什么；P3 与 P2 比，量出 C283 在 ND+A1n 之上多换来什么。挂载准入与发布准入用同一个式子（`admission_reading_before_a_publish`），四条臂都改两处。

| 格 | 问题 |
|---|---|
| P3 的可用性 | 在「正常卸载再挂」与「崩了再挂」两支上各量：S4（发布放行、下一次可写挂载被拒）、S4b（挂载放行、紧接着同样大小的覆盖写被拒）、D3（空间分配） 已定项 9 第 2 条「删掉 s 字节之后同样大小的写在有界步数内成功」要几步（按用户发布数与准入内推的空发布数分开报）、吸收态只读（重挂、写、删、回退全不成）有几格 |
| P3 的安全性 | 崩在准入推的那一串空发布中间、崩在删除类发布中间：挂载前后两份镜像 checker 违例数；准入推空发布抬 F 会不会把一个回退候选在它还被用户要的时候抬掉（F 的上限是不是够不着第 4 新的非空根） |
| P3 的多扣 | 根环转过之后「这次根写下时按谓词回收的那一笔」不记，按第二轮攻方的推算多扣 12–14 槽（推的）：量出来，看它会不会在小盘上让 P3 报拒而 P1 不拒 |
| 第二轮没造的历史 | 行数逼近 369 的整数倍；同一会话里挂着时回退过、又有根被抛弃（回退改形态之后按新形态造）——挂载路径的 `rows0` 与被抛弃根独占量两处结构性分叉在四条臂上会不会让结论相反 |
| 第二轮判决复核 | A3、A4 与 Sonnet 读法的 A1 只被打中过一轮：在有 C283 的前提下它们还出不出局；第二轮判 ND 单独用出局、判 C283 为共用病根，够不够得着它引的证据 |

**共用问句**：每条臂都答「这条合法历史走完之后，池还能不能写、删文件能不能腾出空间再写」，答案要量，不许推；不许用「拒绝」换「安全」而把池永远卡在只读，除非说得出用户手里还有哪一步能让它重新可写；`.claude/rules/fs-design.md`「释放空间这个操作本身不需要申请空间」与 D3（空间分配） 已定项 9 逐句核。

## 二、实现今天的样子（主 agent 的观测，2026-09-26 JST 13:xx；冻结副本 `/tmp/claude-1000/safety-r3-frozen/`，实一至实四乙之后、没提交，开工快照 `research/prompts/m2-safety-r3-snapshot/`）

- **准入**：`crates/singlefs-core/src/admission.rs`：`available_on_each_device`（`own_terms` 五项加池级三项），`admit_on_every_device` 逐盘合取，`demand_of_the_roles_on_each_device` 只算 `Demand` 那一类角色，`admission_reading_before_a_publish` 发布与可写挂载共用；拒绝类型 `AdmissionRefusedOnSomeDevices` 的注释写明「先推空发布抬 F 再判一次是调用方的事，这里不做」。发布路径在 `crates/singlefs-core/src/transaction.rs` 调它（`PublishError::SpaceAdmissionRefused`），挂载路径在 `crates/singlefs-core/src/mount.rs`（`MountError::SpaceAdmissionRefusedBeforeAcquisition`）。
- **C283 没实现**：`mount.rs` 的 `raise_rollback_floor` 注释写「今天只有测试入口，没有产品路径」，准入拒绝之后没有一处调它。抬 F 那一串（准入与卸载共用）是 `mount.rs` 的 `raise_the_floor_through`，入口 `RaiseFloorEntry::Admission` 判上限、`RaiseFloorEntry::Unmount` 不判。
- **正常卸载已实现**：`mount.rs` 的 `unmount`，推空发布把 F 抬到现行那一版的 txg，根带卸载记号；树表 0 条的一版一个字节都不写。
- **挂着时回退已实现**：`mount.rs` 的 `roll_back_by_a_forward_publish`，是一次普通发布，走发布准入。
- **还在查的**：故障注入大档在这一版代码上报了 25 条新发现，注入的全是设备说谎（写被吞），其中 3 条是对拍模型与实现在「抬 F 的上限」上对不上、2 条是冷启动读回对不上（实四乙报告 `research/prompts/impl-rbf-4b-tmp-evidence/report.md` 第九节第 8 条；实四丙在查原因）。P1、P3 用的就是准入抬 F 的上限：腿量到与这一格有关的反常，先对照这几条，写明是不是同一处。
- 第二轮攻方的模型在 `research/prompts/m2-safety-r2-opus-model/`、正推的在 `research/prompts/m2-safety-r2-sonnet-model/`，是在回退改形态之前的冻结副本上写的；这一轮要搬到新冻结副本上，搬的时候把挂载时回退那几步换成挂着时回退，并写明换了哪几步。

## 三、条款（材料员整段抄进附录）

- D28（挂载期承诺量） 已定项 1、3、4；D3（空间分配） 已定项 2、9；D16（发布语义） 已定项 1（准入、`df`、抬 F 的上限、抬 F 那一串、正常卸载）；
- D18（块里携带什么信息） 已定项 11（可写挂载的顺序、取号之前逐盘核）；
- 欠账 C283、C375、C546、C120、C482；岔路单 `research/prompts/alloc-basis-forks.md` 第 10 行；
- 第二轮判决整份。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | 第二轮判决复核 | 替只被打中过一轮的 A1（Sonnet 读法：换下的块算进可用）、A3、A4 辩：各自加上 C283（照 P1 的「判不过时」那一格）之后，在冻结副本的拷贝上两支（正常卸载再挂、崩了再挂）量 S4、S4b、删了再写的步数、吸收态，还出不出局；并指出第二轮判决里哪一句够不着它引的证据（ND 单独用出局、C283 是共用病根这两处必核） |
| 云端攻方（Opus） | P3 的可用性、P3 的安全性、P3 的多扣、第二轮没造的历史 | 自己实现 P1–P3 攻 P3：两支上量 S4、S4b、删了再写的步数、吸收态（P1、P2 当对照）；准入推空发布的那一串崩在中间、抬 F 抬掉用户还要的回退候选、准入里推的空发布自己取不到落点（C546 那一类）、一次准入 8 次发布做满仍不够的历史；量多扣；造第二轮没造的两类历史，量两处结构性分叉 |
| 本地攻方 | 算术 | 按事实表逐格算：给定一个小盘（事实表给容量、R、每次覆盖写的普通分配与固定点槽数、根环长度），在 T、P2 下各一步的可用(d) 与需求(d)；删掉 s 个槽之后在 P1、P3 下，准入里要推几次空发布才放行、推满 8 次还差多少。事实表每行写来源文件与行号，每张表不超过 6 行 |

辩方复核第二轮判决（前两轮的 Sonnet 都是正推腿）。两条攻方腿不重叠：Opus 攻 P3 的可用性、安全性与新历史，本地只算数。辩方与 Opus 各自实现一份 C283，不共用草稿目录。

## 五、交付

- 腿的报告写 `research/prompts/m2-safety-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 模型与用例放 `research/prompts/m2-safety-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`。
- 引 kb 写 kb 快照里那份文件自己的行号，行号去快照里现查，不从背景材料里数。
- 方案按 `crates/` 今天的实现来谈，也就是冻结副本。
- **内存与进程**：编译与跑一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>`，撞上限就记下、不许不套上限重跑；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；等后台任务就结束本轮等通知，不用 `true` 或 `sleep` 空转；交回之前后台不许留着跑的东西。
- 不跑名字带 `layer0` 的测试目标，不跑门禁 54、55、57、59 号。
