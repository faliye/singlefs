# 已结束会话留下的、没过代码轮的改动：代码轮第一轮正文（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 M1 M2 M3 Z1 Z2 Z4 L1 B0 -->

## 一、题面

三块改动写进了 `crates/`，没走过代码轮（或在最终轮之后又改过）；写它们的会话都已结束，由删除与并发这条线收尾（`records/2026-10-01-删除与并发事务.md` 2026-10-03 盘点条目）。这一轮判：代码做的是不是条款或用户定案说的，哪一格会错。原定的 m2-five-decisions-code-r1（材料 `research/prompts/_m2-five-decisions-code-r1-*.md`）判的是 2026-09-28 的冻结副本，快照对今天的工作区 135 份对不上，那一轮退役，M1 并进这一轮按今天的代码判。

改动全文与定位在 `research/prompts/_m3-leftover-code-r1-diff.md`，冻结副本 `/tmp/claude-1000/m3-leftover-code-r1/tree/`：

| # | 改动 | 主要落点（冻结副本行号） | 压着的条款或定案 |
|---|---|---|---|
| M1 | 五条定案的实现：① C331（择根倒挂压过已确认的写） 取甲：可写挂载的见证读里一个系统配置槽读不出或自证不过就判不出、按真；取号那一刻某块盘缺一槽就在第一个取号写之前拒；② C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算；③ 被抛弃根带的 F 算进 F_生效（实现与 checker 各一份）；④ I-3.1 的遍历并上 F_生效 之下的 defer 槽（收口表第 43 行丁-defer） | `crates/singlefs-core/src/mount.rs` 第 440、1312 行；`transaction.rs` 第 681 行；`recovery.rs` 第 1832 行；`crates/singlefs-checker/src/walk.rs` 第 4616 行；四份新用例与改钉的旧用例（diff 文件第一节点名） | D23（journal 的角色与格式） 已定项 14（N-配置续、影子账那两句）；D16（发布语义） 已定项 1「生效」那一段；I-3.1（已分配统计对得上）；判决 `research/prompts/unreadable-at-mount-r2-main-verification.md` L2、L3，`research/prompts/abandoned-floor-r1-main-verification.md` M2；用户 2026-09-28 定案（`records/2026-09-28-收口表对齐与三笔欠账复查.md`「用户定案」两节） |
| M2 | 写零折叠块内乱序：宿主检查折叠写零从「按序、首尾相接」改成按 `write_zeroes_in_chunks` 的切法分块，块内不论次序、块间按序；长度 0 不折 | `crates/singlefs-checker-tier/src/device_log.rs`（diff 文件第二节） | 调查报告 `/tmp/claude-1000/gate-comment-distill/investigate-qemu-zero-fill/report.md`；改法报告 `/tmp/claude-1000/gate-comment-distill/fix-zero-fill-fold/report.md` |
| M3 | 对接上游第三轮判决之后的改动：解析去前后空白与几个入口写 0 判红；坏盘输入种子数写 0 挪回入口；窄一断言（基镜像照 journal 恢复落到的根就是它里面最新的根，不是就拒）；按登记用例的白名单表；抬 F 搭建拆进 `common_admission_pool` | diff 文件第三节；`crash_amplification.rs` 第 447–470 行；`crash_identity.rs` 第 890、1584–1650 行；`crates/singlefs-harness/tests/common_admission_pool/mod.rs` | 判决 `research/prompts/test-script-align-code-r3-main-verification.md`；用户 2026-10-02 定案：第 1 条取「窄一」、第 3 条取「按登记用例白名单」（`records/2026-10-01-对接上游测试脚本规范.md` 第 679 行） |

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 冻结副本取自 2026-10-03 主工作区（删除批已合入），被判 32 份文件的 sha256 在 `research/prompts/m3-leftover-code-r1-snapshot/crates-sha256.txt`。
- 删除批合入后的副本上 harness 档 117 个测试二进制全绿（`/tmp/claude-1000/m3-unlink-body/r8-main/`）；checker 档没跑。对接上游收尾那一趟 checker 档快档红在 GPU 判器 5 条与 `crash_enumeration_*` 3 条（`records/2026-10-01-对接上游测试脚本规范.md` 第 683 行，记为不归那一批、HEAD 上同样红）。
- C600（吞写之后可写挂载被拒而模型说该成，`.claude/kb/checks-owed.md` 第 463 行）：种子 7463871032432355222 吞掉起点段一次系统配置槽写之后，模型 MountWritable 该成，实现以 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)` 拒；那个拒法是 M1 ① 那一批进的。条款没写这种拒算不算允许的结局。
- 删除批的代码轮 m3-unlink-code-r1 同时在跑，判的是别的文件里的删除路径；两轮的格不重叠。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　五条定案：条款对实现（M1）
照 `research/prompts/_m2-five-decisions-code-r1-body.md` 第三节 Z1 那一格的问法，改成对今天冻结副本里的代码问：五条定案各自的条款原文与代码对不对得上，九月二十八日之后别的批改这些函数时有没有把定案的行为改掉；`SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration` 那一支经可写挂载走不走得到。

### K2　五条定案：哪一格会错（M1）
照同一份正文第三节 Z2 ①–⑤ 与 Z4 ①–④ 的攻法，在今天的冻结副本的拷贝上攻；那一轮没派过腿，这些角度都没被攻过。

### K3　C600（M1 ① 的一个结局）
设备吞掉一次系统配置槽写之后，可写挂载以「见证读里有更新的发布、选中的那一版读不出」拒，是不是 D23（journal 的角色与格式） 已定项 14 允许的结局；两条出路（模型认这种拒，或改挂载的判法）各自在哪些合法历史上给出什么结局、各会不会把别的形压成永远只读或漏判倒挂。只交两边的结局与依据，不替用户选。

### K4　写零折叠（M2）
分块的切法与 `write_zeroes_in_chunks` 逐字一致吗；「块内不论次序」会不会把一段真缺了中间一块的写零折成整段（乱序之外还丢了一块）；按序到达的一条跨块仍折那条旧行为与新规则会不会在某种到达次序上互相打架；长度 0 不折之外还有没有不终止的输入。

### K5　对接上游第三轮之后（M3）
窄一断言是不是用户定的那一句（只断言落到的就是基镜像里最新的根），会不会误拒今天登记的流；白名单表与「逐个引入者判」能不能被一个经库模块间接引入、没登记的用例绕过；去空白与写 0 判红在四个入口上是不是同一个口径；抬 F 搭建拆分之后，九个使用者拿到的池几何与拆分前逐项相同吗（实现员抄了 384 槽单元区与 6 MiB journal 环两个数）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3、K4、K5 | 逐条从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行；K3 两条出路各推一遍 |
| 云端攻方（Opus） | K2、K3、K4、K5，重点 K2 | 在冻结副本的拷贝上造反例，只跑 harness 档；K3 造出两条出路各自在哪段合法历史上给错结局 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；第二轮攻改法落地与这一轮站住的。
- K3 不判对错，判决把两条出路的结局与依据并排交用户。
- 腿提的新改法算零轮。

交付：腿的报告写 `research/prompts/m3-leftover-code-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-leftover-code-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
