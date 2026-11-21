# device-library-write-many-code-r2：判定库口子成批写，代码轮第二轮正文（2026-11-10）

## 一、题面

第一轮判决 `research/prompts/device-library-write-many-code-r1-main-verification.md` 打中五处（文件头「一次提交」过度外推；坏键之后转交把剩下的字节当新请求读、大批两头互等；内存那一种不核键；`expect` 没写凭什么不中毒；`reply != "written"` 没有测试喂过 `failed`），第三节交了五条改法。这一轮攻修补本身，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。不许重复第一轮已攻过的角度（第一轮报告：`research/prompts/device-library-write-many-code-r1-sonnet-output.md`、`research/prompts/device-library-write-many-code-r1-opus-output.md`）。

改动全文：`research/prompts/_device-library-write-many-code-r2-diff.md`，第一节是 `lib.rs` 从第一轮开工快照那一版到此刻，第二节是转交脚本对 HEAD（上下文）。开工快照：`research/prompts/device-library-write-many-code-r2-snapshot/crates-sha256.txt`。

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 被判的文件：`crates/singlefs-device-library/src/lib.rs`（快照 sha256 `d9e7b711…`）。实现员报告：`~/.local/share/singlefs/work/device-library-write-many-r1-fix/result.md`。
- 修补做了什么（照实现员报告，主 agent 读过 diff）：
  - `write`、`write_many` 在拿锁、按后端分支之前先核相对键（`give_up_unless_the_relay_accepts_the_key`，`lib.rs` 第 98 行起），转交与内存两种都核，一批里有一条不合整批不发；退出码常量 `RELAY_BROKEN_EXIT_CODE` 改名 `OTHER_ERROR_EXIT_CODE`（第 29 行）。`read` 不核键。
  - 文件头写明一帧（8192 条或 16 MiB）以内一次提交、超过切几帧各自提交、整批 `written` 在全部帧确认之后才回。
  - 两把锁的 `expect` 各用一条常量写理由。
  - 为让「持锁期间不会 panic」成真，另改两处：`found` 报的字节数超过 `isize::MAX` 当认不出退 3；`give_up` 写标准错误写不进也照退（原来 `eprintln!` 会 panic）。
  - 收进攻方的两条 `*_answered_failed_*` 测试（再起一次本测试二进制当子进程），新多线程测试 `a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply`。实现员自己写明：这条多线程测试在「锁在请求与回应之间放过」变异下不是严格必红（std `Mutex` 放锁不交给等的一方），抓到的机会是推的、没量过。
- 转交一侧（上下文）：`scripts/experiment-apparatus/device-code-key.py` 认不出一条请求后抛 `RefusedRequest`，`serve` 回完 `failed` 就停读这根管道；文件头同样改成一帧一次提交。
- 主 agent 的读数（量的）：
  - 本包测试任务 12078 退 0；clippy 任务 12079 退 0；变异行静态核任务 12080 退 0。
  - 证红任务 12083：`✓ 点名 9 条：现判 2 条、复用 0 条、拒算 7 条；跑了 9 条，抓到 9 条`（「读的锁在请求与回应之间放过」那一行这一次抓到，只跑了一次）。
  - 下游：E158 装置单测任务 12081 `158 passed`；E174、E178 任务 12085 `30 passed`、`54 passed`。
  - 转交自证任务 12073 `device-code-key 自证：36 格，红 0 格`；弄坏开关 `relay-reads-on-after-refusal` 任务 12074 只红新加三格。
- 压着的条款与第一轮相同：`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」；`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」里「一块判完才落盘，写入是原子的」。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 修补本身：核键放在拿锁之前，键规则与转交 `RELATIVE_KEY_PATTERN` 是两份实现，两边会不会对同一个键判得不一样（空串、`..`、开头的 `/`、非 ASCII 字母数字）；`read` 不核键留下了什么。
2. 修补本身：`found` 长度过大、`give_up` 标准错误写不进这两处新改法，是不是引进了别的 panic 或吞掉了本该报的错。
3. 产生结论的方法：起子进程的七条测试（拒算）判的是不是子进程真走到的路；多线程测试在锁变异下抓到的机会，「一次抓到」能不能当证据（`.claude/rules/mutation-sampling.md` 第三类「取样点不敏感」）。
4. 从未被看过的地方：`give_up` 一族在测试进程里退整个进程，与 libtest 的多线程并行跑测试怎么相处；`read` 的回应解析（`found <字节数>` 之后的内容读法）在修补后有没有变。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告「走不到的理由」写无：产品代码没有新加 `todo!`、`unreachable!`、`assert!` 与条款没定的错误成员。`count_write_many_calls` 在 `Backend::Relay` 上的 `panic!` 第一轮已判走不到（只在本包与 E158 的测试模块里调）。腿逐处追调用点判。

### 检查单：类型与名字

新加：`give_up_unless_the_relay_accepts_the_key`，常量 `OTHER_ERROR_EXIT_CODE`（由 `RELAY_BROKEN_EXIT_CODE` 改名），两把锁 `expect` 理由的常量，新测试九条（名单在 `crates/mutations.tsv` 末尾九行的测试列）。腿判名字、`expect` 理由写的不变量对不对、有没有 `_ =>`。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；腿现跑 `grep -nE 'legacy|old|migrat|兼容' research/prompts/_device-library-write-many-code-r2-diff.md` 核。

### 检查单：不新建判定库

本轮改动没有这一类：只加键核对、错误处理与测试；内存那一种照旧只在单测、测完即丢。

### 检查单：所有任务多机可用

起子进程的测试用 `std::env::current_exe()` 再起本测试二进制，不读主机名与地址。腿核子进程读不读本机独有状态。

### 检查单：测试优先多线程

新多线程测试本身就是并发测试；其余新测试毫秒级、不切片。腿判那条多线程测试的抓法够不够（攻击面 3）。

### 检查单：带 --item 的命令与项

`crates/mutations.tsv` 末尾新加九行（名字以「判定库口子」起头），点名证红任务 12083 九条全抓到；门禁 crates-mutation-replay 的 `--item` 命令写法与第一轮任务 12058 相同（`--item '<变异名>'` 逐个点名，末尾带仓根）。腿核新项能被 `--list-items` 列出（第一轮任务 12056 的列法）。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新的崩溃点重放流或崩溃枚举用例。

## 五、格

1. 第一轮判决第三节五条，每条的改法是不是修到了它点名的那一格（逐条判「修到」「没修到」「修过头」）。
2. 第三节四条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。

### 小节清单：`.claude/kb/experiments/158-择根与修复四岔路.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E158 择根与修复四岔路 —— 第 4 次跑已交（岔路 1、2 用户已定，3 记欠） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E158 标题之下、第一个下级标题之前的正文：第 2-24 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 2026-11-09：岔路单第 3 行换成如实账本重出，岔路 1 的 q1 三行删除 | 抄 | 默认全抄（脚本生成） |
| ### 2026-09-27：第 4 次跑（重跑登记 `e158-r4-prereg.md`） | 抄 | 默认全抄（脚本生成） |
| ### 2026-09-26：第 2 次跑第一段（重跑登记 `e158-r2-prereg.md`） | 抄 | 默认全抄（脚本生成） |
| ### 这一段做了什么 | 抄 | 默认全抄（脚本生成） |
| ### 2026-09-24：第一段重跑（产物结构性对不上，原因已诊断） | 抄 | 默认全抄（脚本生成） |
| ### 结果整行抄自产物（产物已移出工作区的行写成一行指针） | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 2026-09-25（续，session s10）：op1 第三种变体（岔路单第 1 行）；`replay.sh` 全量重出；发现另一条并行线正在改的 `crates/` 影响岔路单第 3 行的前提 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-09-23 | 不抄 | 历史节的子节 |
| ### 2026-09-24 | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s3，接续一个撞了会话限额而中断的执行员） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s3 第二段，主 agent 消息「接着做，这一段还没交出岔路单要的数，不停」） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s4） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s5，派发要求先做完 PC2 阳性对照才往下） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s6） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s7） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s8） | 不抄 | 历史节的子节 |
| ### 2026-09-26（第 2 次跑第一段） | 不抄 | 历史节的子节 |
| ### 2026-09-27（第 3 次跑第一段与第 4 次跑） | 不抄 | 历史节的子节 |
| ### 2026-11-03 | 不抄 | 历史节的子节 |
| ### 2026-11-09 | 不抄 | 历史节的子节 |

### 小节清单：`.claude/kb/experiments/174-借据丢掉时等待与显存回还.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E174 借据丢掉时等待与显存回还 —— 两段跑完、够判 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E174 标题之下、第一个下级标题之前的正文：第 2-25 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 做了什么 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 产物清单 | 抄 | 默认全抄（脚本生成） |
| ### 读数：不起卡的几份（模型卡数据，不是 G2-乙 的读数） | 抄 | 默认全抄（脚本生成） |
| ### 读数：第一段（真卡，设备上只有这一笔借据的活） | 抄 | 默认全抄（脚本生成） |
| ### 读数：第二段（真卡，同一设备几条线程共用） | 抄 | 默认全抄（脚本生成） |
| ### 判决行的点名 | 抄 | 默认全抄（脚本生成） |
| ### 单测与变异 | 抄 | 默认全抄（脚本生成） |
| ### 岔路表 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-10-08：两段真卡跑完、判完 | 不抄 | 历史节的子节 |
| ### 2026-10-08：建页，装置写好、没上卡跑 | 不抄 | 历史节的子节 |

### 小节清单：`.claude/kb/experiments/178-叶容器与记录死亡的计数.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E178 叶容器与记录死亡的计数 —— 已跑（四段） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E178 标题之下、第一个下级标题之前的正文：第 2-32 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 做了什么 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 判决（整行抄自 `research/results/e178-leaf-and-record-death-counts-segment-one.out`） | 抄 | 默认全抄（脚本生成） |
| ### 读数：每次借的多写与退役记录（Q-X4-1、Q-X4-2、Q-X4-3） | 抄 | 默认全抄（脚本生成） |
| ### 读数：n1 在现有与新补坏镜像上判得出、判不出的（Q-X4-4 至 Q-X4-7） | 抄 | 默认全抄（脚本生成） |
| ### 锚点、停机对拍与阳性对照（整行抄自产物第 2–9 行、第 1520–1527 行） | 抄 | 默认全抄（脚本生成） |
| ### 读数：第二段（底座 B-on、X2、X3 的漏活那一半） | 抄 | 默认全抄（脚本生成） |
| ### 读数：第三段（X3 的崩溃那一半） | 抄 | 默认全抄（脚本生成） |
| ### 读数：第四段（X1 的两个新底座 B-stub、B-tsz） | 抄 | 默认全抄（脚本生成） |
| ### 单测与变异 | 抄 | 默认全抄（脚本生成） |
| ### 岔路表（`m3-leaf-death-forks.md` 的 X1、X2、X3、X4 四行） | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 路径与结论登记 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-10-08：建页，第一段已跑 | 不抄 | 历史节的子节 |
| ### 2026-10-08：第四段的读数 | 不抄 | 历史节的子节 |
| ### 2026-10-08：第二、三段的读数，第四段的装置 | 不抄 | 历史节的子节 |

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1-24`（整段抄，未转述）**

```markdown
## E158 择根与修复四岔路 —— 第 4 次跑已交（岔路 1、2 用户已定，3 记欠）

**中文** · [English](158-four-forks-in-root-selection-and-repair.en.md) · [日本語](158-ルート選択と修復の四つの分岐-第4回実行を提出済み-分岐1・2はユーザーが決定、3は未完.ja.md)

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 V1 V2 V3 V4 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 H1 H2 H3 PC1 PC2 PC3 S16 -->
本文里 S1–S5、V1–V4、F1–F10、H1/H2/H3、Q1-1/Q1-2/Q1-3/Q1-4、Q2-1/Q2-2/Q2-3、Q3-1/Q3-2、Q4-1/Q4-2、PC1/PC2/PC3/PC-Nw、N_w
这些编号与代号都是 `e158-preregistration.md` 跑前登记自己的判据 / 历史族 / 阳性对照代号，登记位在那份文件里，不是 kb 的 D / E / C / I 编号。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| G0 | 主几何点 |
| P331 | 打中判据 |
| P332 | 违例判据 |

G0（主几何点）、P331（打中判据）、P332（违例判据） 是本页自己的本地标签（跑前登记 5.1/5.0 定义：G0（主几何点） = (2, S=8, 环 3 MiB) 这个主要
取样几何点；P331（打中判据） 是岔路 2「打中」的判定过程；P332（违例判据） 是岔路 3「违例」的判定
过程），登记位就在这张表，不在跑前登记文件里另登记第二份。

<!-- doc-lint:not-numbers K0 K1 K2 K3 K4 S0 S6 F11 F12 -->

<!-- doc-lint:not-numbers Q0 Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 L0 L1 L2 L3 L4 L5 L6 F13 F14 F15 F16 F17 F18 F19 S7 S8 S9 V5 C0 M3 PC-22 PC-554 -->

```

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:25-102`（整段抄，未转述）**

````markdown
### 2026-11-09：岔路单第 3 行换成如实账本重出，岔路 1 的 q1 三行删除

这一节是 E158（择根与修复四岔路） 岔路单第 3 行（第一段 `all` 里的 H3、Q3-2、Q2-3、PC3，第二段 Q3-1 的四个几何点）的现状，下面各节里这几项的读数与产物行（2026-09-23 至 2026-09-25 的各份）由它取代；岔路 1 的 `q1-g0`、`q1-s16`、`q1-s4` 三行，用户 2026-11-09 定案删除并标作废。岔路 1、2 的承重读数在第 4 次跑里，由 `r4-compare` 那一行承重（现判 `pass`）；第 2 次跑与第 3 次跑的读数不承重（「2026-09-27：第 4 次跑」一节已写明）。同日另删 7 行，见下面的 ⑥。

**更正**：「2026-09-26（第 2 次跑第一段）」历史条写过「岔路单第 3 行不再量」，与现状不符。岔路单第 3 行这五行（`all` 与 q3-1 四档）一直在 `scripts/experiment-apparatus/replay.sh` 复跑表里，这一次按新账本重出，问的是新判据 `content_differs_from_its_effective_root`。作废的只有 C332（回退实例两个根都读不出时回退被撤销） 正文钉的那句「2 个故障就能撤销回退」（回退改成向前发布之后产品没有这个动作，2026-09-25 session s10 一节已记测不出）与 q1 三行。

**做了什么**：装置账本由「截断时间线」改成如实账本，只改 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs`，产品代码 `crates/singlefs-core` 没动；旧账本那条路与两个注入开关（`E158_ROLLBACK_LEDGER`、`E158_INJECT`）删了。五份新产物：`e158-root-choice-repair-2026-11-09-segment1-rerun.out`（85 行，其中 73 行是 `E7RESULT`，另 12 行是 `N1_DONE`）、`e158-root-choice-repair-2026-11-09-q3-1-g0.out`（66 行）、`e158-root-choice-repair-2026-11-09-q3-1-s16.out`（66 行）、`e158-root-choice-repair-2026-11-09-q3-1-small-ring.out`（67 行）、`e158-root-choice-repair-2026-11-09-q3-1-s4.out`（66 行），每份末行 `E7RESULT name=done emitted=` 的数等于该份 `E7RESULT` 行的行数；`replay.sh` 的 E158 五行改指它们，2026-09-25 的各份留在 git 历史里。

**① 11907 条假违例的原因与账本修法**：中间一版装置（回退按「截断时间线」记账，任务 10904 的输出里 `pairs=70470 pairs_with_any_violation=11907`，原始输出未留存于仓内）在 G0（主几何点） 上报 weight=0 的 104 条、weight=1 继承的 1510 条、weight=2 继承的 10293 条违例，合计 11907 条。全是装置账本造成的，产品没读错：`mount_rollback` 先做一次可写挂载，真写下一行与暖机的几条根（内容是回退之前的），旧账本把这几条根记成回退目标的内容，又把时间线截到目标；之后再回退到这几条根之一，产品读回它们的真内容（对的），账上那一段已被截掉，V2 判「不在时间线上」。另一处错：旧账本不记回退那条根，下一步被 S5 拒绝（G0 上 M 被拒 76 次、R 被拒 303 次），枚举域被悄悄剪小（G0 节点 679，如实是 1371）。只按账本规则写的算术模型逐数对上了 G0 的 15 个数，含 104 / 1510 / 10293。修法是如实账本：回退是一次向前发布（D23（journal 的角色与格式） 同一份文件里「管理员回退是挂着时的一次向前发布」），时间线整条留着；写行与暖机的根按回退之前最新根的内容记，回退那条根按目标的内容记并追加；`abandoned` 恒空；目标不在时间线上、或所选根不是时间线尾的挂载，按 S5 拒绝，计入 `e158_ledger_diagnostics_refused`（G0 上 402 次）。

**② 判据预言记输一次**：旧判据 `faulted_cold_content_differs_from_its_root`（读回内容与所选根账上内容比）的预言是 0，G0 上量到 10351、S4 上量到 93530。不是内容读丢，是重放走到了时间线尾：所选根不是尾（`faulted_cold_chosen_root_is_not_the_tip=19610`），journal 重放把内容带回尾。本次产物里的直接读数：152027 次带故障冷读读失败 0、读回内容不是时间线尾 0（带回退与不带回退各 0）；不看 journal 的那一臂 `ignore_policy_content_is_not_the_tip` 是 10351，恰好等于旧判据的 10351 条（关掉重放，这 10351 条读回的就不是尾）。这条预言记输一次。新判据 `content_differs_from_its_effective_root` 拿读回内容与 journal 施加之后实际走的那条根（`effective_root`）的账上内容比：G0 与 S4 都是 0；同组的 `effective_root_off_the_timeline`、`differs_rows_with_nothing_applied`、`ignore_policy_content_differs_from_chosen_root`、`no_fault_read_is_not_the_tip`、`no_fault_read_failed` 也都是 0。这六个量任一非 0，装置打 `name=stop` 退 1，四档都没打。旧判据的数留着当读数，不判红。

**③ 四档 Q3-1 读数**（`pairs_with_any_violation` 四档都是 0，`cold_recover_with_fault_failed` 四档都是 0）：

| 几何点 | nodes_visited / pairs（本次） | nodes_visited / pairs（2026-09-25 产物） | mount_writable_with_fault_failed（本次 / 2026-09-25） | faulted_cold_content_differs_from_its_root（本次） |
|---|---|---|---|---|
| G0（主几何点） | 1371 / 153398 | 1822 / 170719 | 115 / 0 | 10351 |
| S16 | 1371 / 153398 | 1822 / 170719 | 115 / 0 | 10351 |
| 小环 | 1371 / 153398 | 1822 / 170719 | 115 / 0 | 10351 |
| S4 | 15956 / 1258176 | 14252 / 1116897 | 1236 / 239 | 93530 |

`mount_writable_with_fault_failed` 的 115 与 1236 各等于同一份产物里 `effective_root_is_not_the_tip` 的数，为什么相等没单查。S4 的 `cold_recover_with_fault_failed` 旧值是 239，本次 0。小环那一点 `small_ring_search` 找到的环长是 1048576（2026-09-25 产物是 49152），这一档与 G0、S16 的各项计数逐项相同，环长这一维上三点没分开；环长为什么变没单查。

`research/results/e158-root-choice-repair-2026-11-09-q3-1-g0.out` 第 57、63、64、65 行：

```text
E7RESULT name=fault_set_violation_summary geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 nodes_visited=1371 pairs=153398 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=115 skipped_by_the_reduction_rule=132417
E7RESULT name=e158_effective_root_summary reads_checked=152027 effective_root_is_not_the_tip=115 effective_root_off_the_timeline=0 content_differs_from_its_effective_root=0 differs_from_chosen_root_rows=10351 differs_rows_with_nothing_applied=0 ignore_policy_reads_checked=19610 ignore_policy_read_failed=0 ignore_policy_chosen_root_changed=0 ignore_policy_content_differs_from_chosen_root=0 ignore_policy_content_is_not_the_tip=10351
E7RESULT name=e158_ledger_diagnostics_summary no_fault_reads_checked=1371 no_fault_reads_just_after_a_rollback=1100 no_fault_read_failed=0 no_fault_read_is_not_the_tip=0 faulted_cold_reads_checked=152027 faulted_cold_read_failed=0 faulted_cold_chosen_root_is_not_the_tip=19610 faulted_cold_chosen_root_off_the_timeline=0 faulted_cold_content_differs_from_its_root=10351 faulted_cold_content_is_not_the_tip_with_rollback=0 faulted_cold_content_is_not_the_tip_without_rollback=0 mount_then_reads_checked=20866 mount_then_content_is_not_the_tip=0 after_fault_removed_reads_checked=20981 after_fault_removed_content_is_not_the_tip_with_rollback=0 after_fault_removed_content_is_not_the_tip_without_rollback=0
E7RESULT name=e158_ledger_diagnostics_refused step_and_error="R mount_rollback: \"挂着回退：TargetNotACandidate" count=402
```

`research/results/e158-root-choice-repair-2026-11-09-q3-1-s4.out` 第 57、63 行：

```text
E7RESULT name=fault_set_violation_summary geometry=S4(S=4,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=4 nodes_visited=15956 pairs=1258176 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=1236 skipped_by_the_reduction_rule=1050960
E7RESULT name=e158_effective_root_summary reads_checked=1242220 effective_root_is_not_the_tip=1236 effective_root_off_the_timeline=0 content_differs_from_its_effective_root=0 differs_from_chosen_root_rows=93530 differs_rows_with_nothing_applied=0 ignore_policy_reads_checked=191260 ignore_policy_read_failed=0 ignore_policy_chosen_root_changed=0 ignore_policy_content_differs_from_chosen_root=0 ignore_policy_content_is_not_the_tip=93530
```

**第一段 `all` 与旧产物比（空域）**：账上没有被抛弃的根（`abandoned` 恒空），H3 的 N_w 变 0，PC3 判 `not_applicable`，Q3-2、Q2-3 的算术行按 N_w=0 打。这几行是空域：0 是「没有可数的对象」，不是「打不中」，也不是「候选 2 的见证窄到放得下」。它们仍在 `replay.sh` 的 exact 比对里，复跑逐字节一致，但一致不代表问题还有域。2026-09-24 页上「Q3-2 的 N_w = 4」「PC3 坐实……」两段（「结果整行抄自产物」一节末两段）与那一节抄的 `h3_family_summary` 四行、`peak_live_rollback_events_across_the_family` 行不再承重。

| 项（G0 / S16 / 小环三点同值，S4 另列） | 2026-09-25 产物 | 本次 |
|---|---|---|
| `h3_family_summary` 的 histories（G0 / S4） | 1822 / 14252 | 1371 / 15956 |
| `h3_family_summary` 的 error_count（G0 / S4） | 855 / 10629 | 402 / 3053 |
| `h3_family_summary` 的 peak_events_live（G0 / S4） | 3 / 4 | 0 / 0 |
| `h3_family_summary` 的 peak_roots_live（G0 / S4） | 10 / 10 | 0 / 0 |
| `peak_live_rollback_events_across_the_family` 的 value | 4 | 0 |
| Q3-2 `表1+16*peak` 的 width_bytes / margin_within_512 / 判定 | 65 / -34 / past_512 | 1 / 30 / fits |
| Q3-2 `表1+12*peak` 的 width_bytes / margin_within_512 / 判定 | 49 / -18 / past_512 | 1 / 30 / fits |
| Q2-3 `表1+16*peak` 的 margin_within_512_with_plus8 / 判定 | -42 / past_512_together | 22 / fits |
| Q2-3 `表1+12*peak` 的 margin_within_512_with_plus8 / 判定 | -26 / past_512_together | 22 / fits |
| `pc3_overall_verdict` | fail（`pc3` 一行在） | not_applicable（`pc3` 一行不再打） |

`research/results/e158-root-choice-repair-2026-11-09-segment1-rerun.out` 第 57、75、84 行：

```text
E7RESULT name=h3_family_summary geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 histories=1371 peak_events_live=0 peak_roots_live=0 steps_with_positive_events_live=0 leaf_count=1237 leaves_with_positive_events_live=0 peak_events_live_at_a_leaf=0 error_count=402
E7RESULT name=peak_live_rollback_events_across_the_family value=0
E7RESULT name=pc3_overall_verdict verdict=not_applicable reason="GEOMETRY_PRIMARY 全族里没有找到 events_live≥1 的节点"
```

五份产物里 `local_constant_check` 的系统配置字段表合计与根记录宽，本地值与 `crates/` 值同步从 481 变 489、从 457 变 466，来自 `crates/` 在 2026-09-25 之后的字段变化，与账本无关。五份产物的判决行：没有 `name=verdict` 行，没有 `verdict=fail`；带 `=false` 的只有 `section_seven_one_anchor` 与 `section_seven_three_row` 的 `checked_in_this_segment=false`（`verdict=deferred`，登记预期内，旧产物里同样）。

**④ q1 三行删除并标作废的依据**：如实账本之后回退是向前发布，不再产生被抛弃的根，H1 家族的 Φ1 故障域（被抛弃根的账）是空的。任务 10974（`q1-g0` 在新账本上跑的一趟，输出未入库）量到：`q1_h1_family_size` 的 nodes 由 2026-09-25 产物的 36 变 27，`rejected_rollback_attempts` 由 0 变 9；`q1_1a_trigger_summary` 的 pairs 由 3942 变 0（trigger_count 由 798 变 0）；`q1_1a_raise_floor_trigger_summary` 的 pairs 是 0；`pc1_a_and_b` 的 `verdict=not_found`。留着这三行重出，只会得到一份空域的 exact 比对，所以删除：`replay.sh` 里 `driver_e158_q1_g0`、`driver_e158_q1_s16`、`driver_e158_q1_s4` 三行与三个驱动函数删了（`grep -c driver_e158_q1 scripts/experiment-apparatus/replay.sh` 为 0）。session s3 至 s10 各节里第一次跑的 Q1-1 至 Q1-4、PC1-a/b、op1 三种变体与 S16、S4 两点 `N_trig=798` 的读数作废，不再承重。另算一个「目标之后那一段」的域（实现员报告的乙走法）没做。

**⑤ `.claude/gate.d/replay-comparison-units.tsv`**：现查没有 E158 的行（`grep -c 'E158\|e158'` 为 0），不需要改；这一次没有新写重跑登记。

**⑥ 同日再删 7 行（用户 2026-11-09 定「删 7 行并补回枚举」）**：`replay.sh` 复跑表里删 `driver_e158_q2_2a_g0`、`driver_e158_q2_1_pc2`、`driver_e158_q2_1_g0_session_s5`、`driver_e158_q2_1_hc1`、`driver_e158_q2_1_hc1_lower_bound`、`driver_e158_r2_all`、`driver_e158_r3_seg1_today` 七行与只有它们用的驱动函数（`grep -c` 这七个驱动名在 `replay.sh` 里为 0）；留 `driver_e158`、四行 q3-1、`driver_e158_r3_seg1_compare`、`driver_e158_r4_compare`。两个 compare 驱动只按路径读 `research/results/` 下的臂产物（r3 读的 `…-2026-09-27-r3-seg1-today.out` 是被删那一行的旧产物），不调被删的驱动；旧产物一个没删、没挪，留在 git 历史里。
- 删除依据（量的，来自把这七行在今天的 `crates/` 上现跑一遍的判定输出，输出未入库）：差异来自 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙合入，乙把「最新根读不出且系统配置见证过更新的版本」的挂载拒成 K3，这七行的「今天」基线写在乙之前。`r2-all`：`r2_q1_0` 的 87 次 `K0` 全变 `K3`，造出被抛弃根 56 → 0，PC1-a、PC1-c 由 `pass` 变 `not_constructible`，PC1-b 不再打。`r3-seg1` 今天：`verdict=fail` 由 2 变 11（PC-O 与八条 PC-N0），`q0_hidden_root_abandoned_cells` 由 32、8 变 0、0。q2 五行里，`q2-1-hc1` 的 `hit=true` 变成 `error="faulted mount_writable: NewerStateStillUnreadableAfterOneReread(PublishWitnessed…)"`（拒因整行在产物里，量的）；`q2-1-g0` session s5 与 hc1 下界的 `k_min=12` 变 `none_found`，按同一历史同一拒因推（推的，没打成员名）。`q2-1-pc2` 只差常量回显 481 → 489、457 → 466，结果行没变。`q2-2a-g0` 只差代价数：分配记录树节点写 14 → 10 次、229376 → 163840 字节、总写 33 → 29；是哪次改动造成的没追到。
- 页面现状由 `r4-compare` 承重（岔路单 `c554-fix-forks.md` 三行的读数在第 4 次跑），这七行回答的是乙合入之前的基线，已不回答现状的问题；留着只会把「乙拦住了」冻成 exact，`crates/` 每变一次就红。
- **更正上一段写进页里的 q1 依据**：q1 三行删除时用的「r2-all、r3 罩着真实路径的 PC1」不成立，r2-all 现跑造出 0 条被抛弃根，r3 的 `q0_hidden_root_abandoned_cells` 也是 0。真正罩着「被抛弃根的账读不出，从映射现算」这条路径的是写死历史的用例，8 条：`crates/singlefs-verification/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`（14 条用例）、`unreadable_abandoned_root_slot.rs`（3 条）、`abandoned_root_whose_account_is_unreadable_passes_…`、`raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`reuse_after_raising_the_floor.rs`、`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs`，以及 checker 侧两个测试文件（`checker_narrow_invariants_and_abandoned_roots.rs`、`crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）；最近读数全绿，有变异行（`crates/mutations.tsv`）。这些用例的造法把系统配置见证藏掉，所以乙挡不住它们。④ 的其余内容（任务 10974 的读数）不变，仍站得住。
- **枚举缺口**：这条路径没有任何枚举型覆盖（多条历史、多个故障组合）；缺口形成的时点是 C554 乙合入（r2 的 Q1-0 曾有 56 个节点，现在 0），不是这次删除。随机历史那一半记在 C599（随机历史造不出被抛弃的根）；补回枚举由下一次 E158 重跑登记补，还没登记、没跑。

**复跑**：`bash scripts/experiment-apparatus/replay.sh E158`，复跑表里 E158 现有 7 行（`driver_e158`、四行 q3-1、`r3_seg1_compare`、`r4_compare`）。

**这一节答不了的**：新判据只判带故障且读成了的冷读；省法省掉的故障集合（G0 132417 个、S4 1050960 个）没有 (b)(c) 读；只在这四个几何点，故障只点名根槽、journal 环完好；重放有没有跨实例边界这里分不出来；回退到以前被剪掉的根现在才第一次走到，产品拒绝的步列在 `e158_ledger_diagnostics_refused`（G0 402、S4 3053），其余读错了严格判据会红，没红。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:103-437`（整段抄，未转述）**

````markdown
### 2026-09-27：第 4 次跑（重跑登记 `e158-r4-prereg.md`）

「2026-09-27：第 4 次跑」这一节是 E158（择根与修复四岔路） 的现状，回答岔路单 `c554-fix-forks.md`（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 怎么修）的三行。「2026-09-27：第 3 次跑第一段」一节只当参考；「2026-09-26：第 2 次跑第一段」一节与它之后的各节是更早几次跑的记录，读数不带进第 4 次跑。Q0–Q9、H1d–H1g、H-随、H-随全、实七-甲、实七-乙、实八、L0–L6、PC-N-环、PC-N-盖、PC-O、PC-N0、PC-多拒、PC-多读、PC-22、PC-554、PC-随、F1–F19、S0–S9、V1–V5、N_真、N-配置、N-配置续、N-槽 是第 4 次跑登记（与它照用的第 3 次跑登记 `e158-r3-prereg.md`）自己的代号，登记位在那两份文件里。

**岔路表**（岔路单三行；「产物行」里的行号都指五段合并的 `research/results/e158-root-choice-repair-2026-09-27-r4-compare.out`）：

| # | 岔路 | 状态 | 够判依据 | 产物行 |
|---|---|---|---|---|
| 1 | 挂载时有一条根（或它引用的账）暂时读不出、而它可能比所选根新，可写挂载怎么办 | 用户已定乙，R = 1，判「有一条更新的根读不出」用系统配置记的位置（-配置 一族） | **丢写**：今天、甲-槽、乙-槽、乙-窄读-槽 在 H1d、H1e、H1f、H1g、H-随、实七-甲、实七-乙、实八 上都 > 0（今天 H1d 304 / 1680、H-随 43 / 43），按岔路单出局；-配置 四臂（甲、乙、乙-窄读、丙）在 H1g 上各 16 / 32；-配置续 四臂在 H1f、H1g 与第二段实七-甲、实七-乙、实八 上都是 0，在 H-随 上照登记字面 6 / 37，这 6 格全是 ③c 尾巴（不算 ③c 2 格，Q1 只算环内且不算 ③c 1 格）；H1d、H1e 按登记第六节开头对 -配置 各臂不算判据。**多拒**（L 族，G0（主几何点））：甲、乙、乙-窄读 的 -配置 与 -配置续 各 22（L3 16、L5 6，L5 那 6 格里 3 格是罩住第 22 条那一格的代价），丙 两臂各 16，-槽 三臂各 19；第二段五族每臂 0。**多读**（同成，L 族 G0（主几何点））：乙-配置 每格 +3 或 +4（629 格合计 2472），乙-配置续 每格 +7 或 +8（4988）。**第 22 条那一格**：今天没罩住 3 / 72；候选各臂没罩住 0（甲 三臂的 Q6 按 V1 ④ 作废）。**判据用的那一样今天有没有**：S0 在快照上逐条核过，系统配置槽里记着每次发布写下的 `journal_tail`，但今天的可写挂载只判「落后」、不拿它判「领先」，取号那一写把 tail 写成 0（登记修订第 3 条）；要加的是 N-配置 那一判与取号续写见证值 | 丢写 1546–1554、1567–1575；多拒 634、638；多读见「多读」一段的命令；Q6 1564、1585 |
| 2 | 乙-配置 的判据只认系统配置槽里见证的 tail，没见证的最新根怎么办 | 用户已定乙-配置续；乙-槽 与轮换后加屏障不走 | 两臂只在 H1g 上分开：乙-配置 16 / 32（全在 a = 2：连着两次取号都写 tail = 0，装置解出的 N-配置 为假，被测挂载 K0、回滚），乙-配置续 0 / 32（a = 2 上 N-配置 为真、K3）；H1f、第二段四族（seg2b）两臂逐族相同，H-随 同是 6 / 37；多拒同是 22；同成多读乙-配置续每格比乙-配置多 4 次（L 族 4988 对 2472，同 629 格）；-槽 那一支（乙-槽）丢写全落在读回全零那一种造法上（H1d 288 / 2560） | 1575、1596；1569、1590；634、754 |
| 3 | 乙的读缓存要不要收窄（乙 / 乙-窄读） | 记欠（第四批 kb 规格 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙 Q4 那一笔） | 两臂逐格判定**不**全相同：乙-窄读 在读回全零、b = records 的 T(1) 与 O(1 只) 格上拒（PC-N-盖 那三句 [臂] 打不中，乙-窄读 两条 -配置 臂的 Q1–Q3 按 V1 ④ 作废，主 agent 认定这是窄读与乙的差别、照量），拦下栏 H1d 多出 `O=4,T=4`、H1f-cuts 多出 `O=8,T=8`；第二段、第三段的丢写与 L 族多拒两臂逐族相同；同成多读 L 族两臂相同（2472 / 4988），H1d 上乙-配置 +50136（32 格）、乙-窄读-配置 +942（24 格）；**读缓存峰值没量**：第 4 次跑的登记里没有这个量，不为它加臂 | 617、856（H1d 拦下）；624、863（H1f-cuts 拦下）；1775、1782（作废） |

「出局」「选哪个」由主 agent 按岔路单判，第 4 次跑只报数。

**装置与口径**：

- 入库装置 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 的 `r4-*` 模式（`r4-seg1` 至 `r4-seg5`、`r4-bases`、`r4-compare`），确定性（同一个二进制跑两遍逐字节一致只说明没有隐藏状态，第四段执行员拿今天那一臂重跑一次 `cmp` 过）。装置自己记账、自己解码（根槽、系统配置槽、记录经 `singlefs_checker`），与 `crates/` 交回的逐项对拍（S4，每族 `s4_mismatches=0`）。
- 底座：开跑条件 C0 那份快照（对 `crates` 做一次 `cp -a`，底座认定的四个 `singlefs-core` 文件 sha256 逐个对上；登记修订第 1 条）。**今天那一臂 = 快照原样**；十八条臂各自一份副本，从那一次 `cp -a` 派生，改法是 `scripts/mutation/tables/e158_arms.tsv` 里以 `r4 ` 打头的 18 行。
- 十八条臂：今天；甲-配置、甲-配置续、甲-槽；乙-配置、乙-配置续、乙-槽；乙-窄读-配置、乙-窄读-配置续、乙-窄读-槽；丙-配置、丙-配置续（主族十二条）；乙 三臂与乙-窄读 三臂各一条 R = 3（`-three-rounds`，只跑第五段）。乙-窄读 是 5.5 主 agent 认定第 1 项另立的对照臂：只重读根环槽、系统配置槽与点名单元，把 journal 环范围内读回全零的落点当空槽收进缓存（登记修订第 5 条）。
- 五段：第一段 H1d、H1e 与 PC-N-环、PC-N-盖、PC-O、PC-丢写、PC-N0、PC-多读；第二段实七-甲、实七-乙、实八、H-随（30 段）、H-随全（68 段，只进多拒、多读）与 PC-随；第三段 H1f、H1g；第四段 L0–L6 与 PC-多拒、PC-22、PC-554；第五段几何敏感性（G_S4、G_S16、G_默认、G_小环、k = 2、R = 3）。前四段除第二段外都在 G0（主几何点）上，第二段的历史由 harness 的生成器定几何（4 GiB 盘）。
- 装置 bin 的三版：第一、三、四、五段用 sha256 `37a8a312cf9c2d3bf6bf1a835530e78fa6d9f3a4b36607d3ecb8fd5e5cf1445b`（装置执行员报告 `e158-r4-device-runner-report.md` 第一节），产物第一行（`E7INPUT name=crates_snapshot` 那一行）的指纹是 `sha256=bb5ca9bef0b09eb957906317adf7d2b47567d901ef341e85c14b6fe23a37cf4b`；第二段装置另写之后 bin 是 `12b82f491453e4f598f8262632dc949967508a8a1cae745a1b4db078499fb148`，把 C0 清单里装置那一行换成它之后整份清单的 sha256 是 `ad9f544263a5a61bf0bed305ea6e98701042e63ff2ad5e96f2b66552f360f9dc`（登记修订第 29 条），第二段两次产物与五段合并的 compare 第一行都写这个；seg2b 的 bin 又修了一处（崩溃分支挂载之后的读回改用分支自己的账），源码 sha256 `e71224fc02fde638e92f2548c19c71c2f7032b65ec7056a1bc014bebbac8dba8`，seg2b 产物第一行仍是 `ad9f5442…`（**没把修过的那一版算进指纹**，seg2b 执行员报告 `e158-r4-seg2b-runner-report.md` 第二节）。五段合并的 compare 用 seg2b 今天那一臂的二进制（release 二进制 sha256 `ac25b3fe0c58c26c52100d8e5b1c6851fafc8e3b47a53914df7ad204235dd5d2`）跑。
- 各段报告：`e158-r4-seg1-runner-report.md`、`e158-r4-seg2-runner-report.md`、`e158-r4-seg2b-runner-report.md`、`e158-r4-seg3-runner-report.md`、`e158-r4-seg4-runner-report.md`、`e158-r4-seg5-runner-report.md`；装置 `e158-r4-device-runner-report.md`、`e158-r4-seg2-device-runner-report.md`；撞键调查 `e158-r4-seg2-ledger-investigation-report.md`。
- 跨段的量（多拒、多读、阳性对照的跨臂句子、V1 作废范围）**以五段合并的 `research/results/e158-root-choice-repair-2026-09-27-r4-compare.out` 为准**（第四段之后主 agent 认定第 4 项）；它的 `r4_loss`、`r4_q4`、`r4_q4_after_divergence`、`r4_q5`、`r4_q6` 各行与五份分段 compare（第二段取 seg2b 的 `…-compare-r2.out`）排序后逐行相同，只有 `r4_void` 与跨臂句子多了跨段的那一部分。命令与原样输出：

```text
$ R=research/results/e158-root-choice-repair-2026-09-27-r4; for n in r4_loss r4_q4 r4_q4_after_divergence r4_q5 r4_q6; do m=$(grep "name=$n " $R-compare.out | sort | sha256sum | cut -c1-16); s=$(cat $R-seg1-compare.out $R-seg2b-compare-r2.out $R-seg3-compare.out $R-seg4-compare.out $R-seg5-compare.out | grep "name=$n " | sort | sha256sum | cut -c1-16); echo "$n merged=$(grep -c "name=$n " $R-compare.out) segments=$(cat $R-seg1-compare.out $R-seg2b-compare-r2.out $R-seg3-compare.out $R-seg4-compare.out $R-seg5-compare.out | grep -c "name=$n ") same=$([ $m = $s ] && echo yes || echo no)"; done
r4_loss merged=228 segments=228 same=yes
r4_q4 merged=385 segments=385 same=yes
r4_q4_after_divergence merged=40 segments=40 same=yes
r4_q5 merged=571 segments=571 same=yes
r4_q6 merged=36 segments=36 same=yes
```

**产物**（`research/results/` 下 86 份，文件名 `e158-root-choice-repair-2026-09-27-r4-<段>-<臂>.out`，每段另有 `…-<段>-compare.out`；第二段两次都留：`seg2` 13 份、`seg2b` 14 份，seg2b 的 compare 两份）：

```text
$ cd research/results; for s in seg1 seg2 seg2b seg3 seg4 seg5; do n=$(ls e158-root-choice-repair-2026-09-27-r4-$s-*.out | wc -l); d=$(for f in e158-root-choice-repair-2026-09-27-r4-$s-*.out; do tail -1 $f; done | grep -c '^E7RESULT name=done'); st=$(cat e158-root-choice-repair-2026-09-27-r4-$s-*.out | grep -c 'name=stop'); fp=$(for f in e158-root-choice-repair-2026-09-27-r4-$s-*.out; do head -1 $f | grep -o 'sha256=[0-9a-f]\{8\}\|sha256=unset'; done | sort | uniq -c | tr '\n' ' '); echo "$s files=$n done_last=$d stop_lines=$st first_line=$fp"; done
seg1 files=13 done_last=13 stop_lines=0 first_line=     13 sha256=bb5ca9be 
seg2 files=13 done_last=13 stop_lines=0 first_line=     13 sha256=ad9f5442 
seg2b files=14 done_last=14 stop_lines=0 first_line=     13 sha256=ad9f5442       1 sha256=unset 
seg3 files=13 done_last=13 stop_lines=0 first_line=     13 sha256=bb5ca9be 
seg4 files=13 done_last=13 stop_lines=0 first_line=     13 sha256=bb5ca9be 
seg5 files=19 done_last=19 stop_lines=0 first_line=     19 sha256=bb5ca9be 
```

- 第一段：`e158-root-choice-repair-2026-09-27-r4-seg1-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-slot.out`。
- 第二段第一次（装置撞键，只当对照）：`e158-root-choice-repair-2026-09-27-r4-seg2-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-slot.out`。
- 第二段重跑（以它为准）：`e158-root-choice-repair-2026-09-27-r4-seg2b-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-compare-r2.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-slot.out`。
- 第三段：`e158-root-choice-repair-2026-09-27-r4-seg3-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-slot.out`。
- 第四段：`e158-root-choice-repair-2026-09-27-r4-seg4-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-slot.out`。
- 第五段：`e158-root-choice-repair-2026-09-27-r4-seg5-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg-carry-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg-carry-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-slot-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-slot-three-rounds.out`。
- 五段合并：`e158-root-choice-repair-2026-09-27-r4-compare.out`（1832 行，末行 `E7RESULT name=done emitted=1831`；`replay.sh` 登记的是它）。

**丢写（翻面观测第一样）**：丢写格 = Q1 覆盖格 ∪ Q2 回滚格 ∪ Q3 读回丢写格（含 ③c 尾巴）按格去重，照登记字面算（逐格字段 `lost_write_cell` 不含 Q2，写页以族摘要与 compare 的 `r4_loss` 为准，第五段之后主 agent 认定第 2 项）。每臂每族「丢写格 / 格数」，G0（主几何点）与第二段的 harness 几何：

```text
$ grep 'name=r4_loss ' research/results/e158-root-choice-repair-2026-09-27-r4-compare.out | grep -E 'geometry=(GEOMETRY_PRIMARY|harness-4gib)' | grep -Ev 'family=(h1d-uncut|h1e-k2|h1d-uncut-k2|h1e-r3|h1d-uncut-r3)' | awk '{for(i=1;i<=NF;i++){split($i,kv,"=");v[kv[1]]=substr($i,length(kv[1])+2)}; t[v["arm"]" "v["family"]]=v["lost_write_cells"]"/"v["cells"]} END{for(k in t) print k, t[k]}' | sort | awk '{a[$1]=a[$1]" "$2"="$3} END{for(k in a) print k a[k]}' | sort
bing-cfg-carry eighth-crash=0/2 h1d=0/912 h1e=0/64 h1f-cuts=0/2080 h1f-tail=0/128 h1g=0/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
bing-cfg eighth-crash=0/2 h1d=0/912 h1e=0/64 h1f-cuts=0/2080 h1f-tail=0/128 h1g=16/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
jia-cfg-carry eighth-crash=0/2 h1d=0/48 h1e=0/64 h1f-cuts=0/96 h1f-tail=0/128 h1g=0/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
jia-cfg eighth-crash=0/2 h1d=0/48 h1e=0/64 h1f-cuts=0/96 h1f-tail=0/128 h1g=16/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
jia-slot eighth-crash=3/6 h1d=152/864 h1e=32/64 h1f-cuts=48/2080 h1f-tail=64/128 h1g=16/32 h-random=43/43 seventh-crash=1/5 seventh-fault=10/10
today eighth-crash=3/6 h1d=304/1680 h1e=64/64 h1f-cuts=96/4064 h1f-tail=128/128 h1g=32/32 h-random=43/43 seventh-crash=1/5 seventh-fault=10/10
yi-cfg-carry eighth-crash=0/2 h1d=0/1792 h1e=0/128 h1f-cuts=0/4096 h1f-tail=0/256 h1g=0/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
yi-cfg eighth-crash=0/2 h1d=0/1792 h1e=0/128 h1f-cuts=0/4096 h1f-tail=0/256 h1g=16/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
yi-narrow-cfg-carry eighth-crash=0/2 h1d=0/1360 h1e=0/128 h1f-cuts=0/3104 h1f-tail=0/256 h1g=0/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
yi-narrow-cfg eighth-crash=0/2 h1d=0/1360 h1e=0/128 h1f-cuts=0/3104 h1f-tail=0/256 h1g=16/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
yi-narrow-slot eighth-crash=3/6 h1d=288/2560 h1e=64/128 h1f-cuts=64/6080 h1f-tail=128/256 h1g=16/32 h-random=43/43 seventh-crash=1/5 seventh-fault=10/10
yi-slot eighth-crash=3/6 h1d=288/2560 h1e=64/128 h1f-cuts=64/6080 h1f-tail=128/256 h1g=16/32 h-random=43/43 seventh-crash=1/5 seventh-fault=10/10
```

- 族名对应：`h-random` = H-随，`seventh-fault` = 实七-甲，`seventh-crash` = 实七-乙，`eighth-crash` = 实八；第二段的数取自 seg2b（五段合并的 compare 喂的就是 seg2b）。格数随臂不同：H1d 的断点格按被测挂载的写数立，乙 各臂多一档时长；-配置 各臂在实七-甲 上 3 个注入点 `fault_not_reached`、在实八 上 4 个崩溃状态 `not_constructible=segment_missing`（它们在崩溃恢复那一步就拒了，写序列与今天那一臂不同），所以实八只有主路径 2 格，今天那一臂是 6 格（登记修订第 19 条的记法，不立格）。
- 哪些族对哪些臂算判据（登记第六节开头）：H1d、H1e 对甲-配置、甲-配置续、乙-配置、乙-配置续、丙-配置、丙-配置续 **不算**（N-配置 在那两族的每一格按构造判真，0 由臂的定义推得出；乙-窄读 两条 -配置 臂同一个理由，而且它们的 Q1–Q3 已按 V1 ④ 作废）；这几条臂的丢写判据落在 H1f、H1g、实七-甲、实七-乙、实八、H-随 上。今天与 -槽 各臂全族都算。
- 乙-配置、乙-配置续 在 H1g、H-随、实七-乙 上的原样行（五段合并的 compare 第 1575、1596、1569、1590、1571、1592 行）：

```text
E7RESULT name=r4_loss arm=yi-cfg family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=16 q1_overwrite_cells=16 q2_rollback_cells=16 q3_lost_cells=0 q3c_lost_cells=8 q3_last_confirmed_cells=32
E7RESULT name=r4_loss arm=yi-cfg-carry family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=32
E7RESULT name=r4_loss arm=yi-cfg family=h-random geometry=harness-4gib cells=37 lost_write_cells=6 q1_overwrite_cells=2 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=6 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=yi-cfg-carry family=h-random geometry=harness-4gib cells=37 lost_write_cells=6 q1_overwrite_cells=2 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=6 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=yi-cfg family=seventh-crash geometry=harness-4gib cells=6 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=6
E7RESULT name=r4_loss arm=yi-cfg-carry family=seventh-crash geometry=harness-4gib cells=6 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=6
```

- H1g 按 a 分开：-配置 四臂在 a = 1 上 16 格都拒（K3，不丢写），在 a = 2 上 16 格都 K0、回滚、丢写（装置解出的 N-配置 为假：两次取号都写 tail = 0，系统配置两槽的见证都被盖成 0）；-配置续 四臂 a = 1、a = 2 都 K3（取号续写了见证）。数自第三段臂产物的 `name=r4_cell family=h1g` 行，按臂数 `a`、`a_class`、`device_n_cfg`、`q2_rollback`、`lost_write_cell`（第三段报告 `e158-r4-seg3-runner-report.md` 第四节贴了命令与原样输出，例 `…-r4-seg3-yi-cfg.out` 那一臂 `16 a=1 a_class=K3 device_n_cfg=true …`、`16 a=2 a_class=K0 device_n_cfg=false q2_rollback=true lost_write_cell=true`）。

**H-随 那几格：照登记字面算、并列两栏**（第二段之后主 agent 认定第 3 项）。登记字面的丢写格含 ③c 尾巴（关闭 C → 可写挂载 → 冷启动读回 → 落到被藏那条根之前的旧根时的读回）；另两栏出自各臂产物的 `r4_random_summary`（登记修订第 23 条，只报数、不改判据）：`lost_write_cells_without_q3c`（不算 ③c）与 `lost_write_cells_with_q1_in_ring_only_and_without_q3c`（Q1 只算「还在根环里」那一栏且不算 ③c）：

```text
$ cd research/results; for f in e158-root-choice-repair-2026-09-27-r4-seg2b-*.out; do a=${f#e158-root-choice-repair-2026-09-27-r4-seg2b-}; a=${a%.out}; case $a in compare*) continue;; esac; grep -n 'name=r4_random_summary family=h-random ' $f | sed -E "s/^([0-9]+):.* cells=([0-9]+) lost_write_cells_q1_q2_q3=([0-9]+) .*lost_write_cells_without_q3c=([0-9]+) lost_write_cells_with_q1_in_ring_only_and_without_q3c=([0-9]+) .*q3c_lost_cells=([0-9]+) .*/$a line=\1 cells=\2 literal=\3 without_q3c=\4 in_ring_only_without_q3c=\5 q3c=\6/"; done
bing-cfg-carry line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
bing-cfg line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
jia-cfg-carry line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
jia-cfg line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
jia-slot line=1209 cells=43 literal=43 without_q3c=39 in_ring_only_without_q3c=39 q3c=40
today line=1241 cells=43 literal=43 without_q3c=39 in_ring_only_without_q3c=39 q3c=40
yi-cfg-carry line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
yi-cfg line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
yi-narrow-cfg-carry line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
yi-narrow-cfg line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
yi-narrow-slot line=1209 cells=43 literal=43 without_q3c=39 in_ring_only_without_q3c=39 q3c=40
yi-slot line=1209 cells=43 literal=43 without_q3c=39 in_ring_only_without_q3c=39 q3c=40
```

| 臂 | H-随 丢写格（登记字面） | 不算 ③c | Q1 只算环内且不算 ③c |
|---|---|---|---|
| -配置、-配置续 八臂（甲、乙、乙-窄读、丙） | 6 / 37 | 2 / 37 | 1 / 37 |
| 今天、甲-槽、乙-槽、乙-窄读-槽 | 43 / 43 | 39 / 43 | 39 / 43 |

-配置 各臂那 6 格的 ③c 尾巴读回 5 格 `failed:NoValidRoot`、1 格 `failed:UnitUnreadable`，Q2 回滚与 Q3 读回丢写都是 0；不算 ③c 仍丢写的 2 格（`…-r4-seg2b-yi-cfg.out` 里种子基 + 23 的 `main:crash_recovery@3` 与种子基 + 79 的 `main:crash_recovery@6` 两格）都是 Q1 覆盖：前一格那一版在历史末尾已不在根环里、第一次被改在第 30 步，那时它之后已有 11 笔确认；后一格那一版还在根环里、第一次被改在第 25 步，之后已有 5 笔确认（`q1_first_change=order25:later_confirmations5:in_ringtrue`）。那一版早已被后来的确认取代时，Q1 与 ③c 也可能为正（登记修订第 23 条，推的，没量过）；这几格算不算丢写交主 agent，第 4 次跑照字面报。

**第二段两次：seg2 与 seg2b**。第一次（seg2）-配置 八臂在实七-乙 上记了 3 / 6 丢写（`crash0`–`crash2` 三个崩溃状态格，读回 `failed:content_differs_from_the_ledger`），今天那一臂实八 `crash1:crash@5` 的读回判定也受影响。调查报告 `e158-r4-seg2-ledger-investigation-report.md` 第一节查明是装置记账撞键：崩溃分支挂载之后的读回拿主路径的账按根的键 `(实例, txg)` 查，两条时间线在同一个键上各有一版；调查员改用分支自己的账之后 -配置 实七-乙 3 / 6 → 0 / 6，故意记错一次 4 格回红（第二节）。按装置 bug 修（分支的读回用分支自己的账），原判据一个字不改，12 条臂整段重跑成 seg2b。**以 seg2b 为准**。两次的差别只在实七-乙、实八两族，H-随、H-随全、实七-甲三族逐字节相同：

```text
$ cd research/results; P=e158-root-choice-repair-2026-09-27-r4; for a in today jia-cfg jia-cfg-carry jia-slot yi-cfg yi-cfg-carry yi-slot yi-narrow-cfg yi-narrow-cfg-carry yi-narrow-slot bing-cfg bing-cfg-carry; do r="$a"; for f in h-random h-random-all seventh-fault seventh-crash eighth-crash; do if cmp -s <(grep "family=$f " $P-seg2-$a.out) <(grep "family=$f " $P-seg2b-$a.out); then r="$r $f:same"; else r="$r $f:differs"; fi; done; echo "$r"; done
today h-random:same h-random-all:same seventh-fault:same seventh-crash:same eighth-crash:differs
jia-cfg h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
jia-cfg-carry h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
jia-slot h-random:same h-random-all:same seventh-fault:same seventh-crash:same eighth-crash:differs
yi-cfg h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
yi-cfg-carry h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
yi-slot h-random:same h-random-all:same seventh-fault:same seventh-crash:same eighth-crash:differs
yi-narrow-cfg h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
yi-narrow-cfg-carry h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
yi-narrow-slot h-random:same h-random-all:same seventh-fault:same seventh-crash:same eighth-crash:differs
bing-cfg h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
bing-cfg-carry h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
```

今天与 -槽 三臂实八那几行变了、丢写格数没变（3 / 6）。seg2b 的 compare 有两份：`e158-root-choice-repair-2026-09-27-r4-seg2b-compare.out` 跑的时候漏设了快照指纹的环境变量，首行是 `sha256=unset`；另存的 `e158-root-choice-repair-2026-09-27-r4-seg2b-compare-r2.out` 首行是 `ad9f5442…`，两份 `diff` 只差第 1 行，**以 `…-compare-r2.out` 为准**，两份都留着。

**多拒（Q4，翻面观测第二样）**：合法状态（N_真 为假）上臂拒而今天同一格不拒。L 族在 G0（主几何点）上 663 格全是 N_真 为假；乙-配置 与 乙-配置续 的 L 族各行（compare 第 627–641、747–761 行里 G0（主几何点）那几行；L3、L5 两族各另有两个几何点的行）只有 L3、L5 不为 0：

```text
E7RESULT name=r4_q4 arm=yi-cfg family=l3@GEOMETRY_PRIMARY(S=8,ring=3MiB) compared_tested_mounts=24 extra_refusals=16 intercepted=""
E7RESULT name=r4_q4 arm=yi-cfg family=l5@GEOMETRY_PRIMARY(S=8,ring=3MiB) compared_tested_mounts=72 extra_refusals=6 intercepted=""
E7RESULT name=r4_q4 arm=yi-cfg-carry family=l3@GEOMETRY_PRIMARY(S=8,ring=3MiB) compared_tested_mounts=24 extra_refusals=16 intercepted=""
E7RESULT name=r4_q4 arm=yi-cfg-carry family=l5@GEOMETRY_PRIMARY(S=8,ring=3MiB) compared_tested_mounts=72 extra_refusals=6 intercepted=""
```

| 臂 | L 族 G0（主几何点）多拒合计 | 其中 L5（第 22 条那一格） | L5 里罩住第 22 条的代价 | 第二段五族（含 H-随全 68 段） |
|---|---|---|---|---|
| 甲-配置、甲-配置续、乙-配置、乙-配置续、乙-窄读-配置、乙-窄读-配置续 | 22（L3 16） | 6 | 3 | 0 |
| 甲-槽、乙-槽、乙-窄读-槽 | 19（L2 8、L6 4、PC-554 那一格 1） | 6 | 3 | 0 |
| 丙-配置、丙-配置续 | 16（L3 16） | 0 | 0 | 0 |

- l5 那 6 格：甲、乙、乙-窄读 各臂在每份起始镜像的 m = 2、O(2 起) 上拒（`R3NewestInstanceTableUnreadableForTheShadowLedger`），一半在 b = records（今天 `Ok`、隔离 0 = 孪生镜像的 0，今天那一格罩住了），一半在 b = unit_first_copy（今天 `Ok`、隔离 0 < 孪生 28，正是今天没罩住的那 3 格）。后 3 格是「为罩第 22 条多拒」，照第四段之后主 agent 认定第 2 项在「选哪个」里单列、不并进多拒总数；照 Q4 字面它们仍算多拒，合计那一栏含它们。丙 在这 6 格上 `Ok`、隔离 = 孪生（用第一次读出的那张表）。出处：第四段报告 `e158-r4-seg4-runner-report.md` 第五节，产物 `…-r4-seg4-<臂>.out` 的 `name=r4_l_cell family=l5` 行。
- 第二段：`r4_q4` 55 行没有一行 `extra_refusals` 大于 0，走岔之后的 `r4_q4_after_divergence` 40 行没有一行 `refusals_with_truth_newer_false` 大于 0（`grep -c 'name=r4_q4_after_divergence .*refusals_with_truth_newer_false=[1-9]' research/results/e158-root-choice-repair-2026-09-27-r4-compare.out` 输出 `0`）。H-随全 上 -配置 各臂拦下 18 次（N_真 为真时拒），例（compare 第 611、612 行）：

```text
E7RESULT name=r4_q4 arm=yi-cfg family=h-random-all@harness-4gib compared_tested_mounts=293 extra_refusals=0 intercepted="W=18"
E7RESULT name=r4_q4 arm=yi-cfg family=h-random@harness-4gib compared_tested_mounts=64 extra_refusals=0 intercepted="W=30"
```

- 同一种 N 里甲与乙（含窄读）的多拒逐族相等，多拒这一样分不出甲、乙。

**多读（Q5-同成，翻面观测第三样）**：臂与今天那一臂在同一格上都交回 `Ok`、起始镜像逐字节相同的被测挂载，设备一层读调用次数臂减今天。按族归三组汇总（L 族与 PC-554 = 合法状态；随机历史 = 第二段五族；H1 = G0（主几何点）上 H1d–H1g 那几族里两边都挂上的格，乙 的重读在那里起作用）：

```text
$ grep 'name=r4_q5 .*column=both_mounted' research/results/e158-root-choice-repair-2026-09-27-r4-compare.out | grep -E '@(GEOMETRY_PRIMARY|harness-4gib)' | awk '{for(i=1;i<=NF;i++){n=index($i,"=");if(n)v[substr($i,1,n-1)]=substr($i,n+1)}; split(v["family"],fg,"@"); f=fg[1]; g=(f ~ /^(l[0-9]|l1|pc554)/)?"L":(f ~ /^h1/)?"H1":"rand"; k=v["arm"]" "g; c[k]+=v["cells"]; t[k]+=v["total_extra_read_calls"]; if(!(k in mn)||v["minimum_extra_read_calls"]+0<mn[k])mn[k]=v["minimum_extra_read_calls"]+0; if(!(k in mx)||v["maximum_extra_read_calls"]+0>mx[k])mx[k]=v["maximum_extra_read_calls"]+0} END{for(k in c) print k, "cells="c[k], "total="t[k], "min="mn[k], "max="mx[k]}' | sort
bing-cfg-carry H1 cells=62 total=96568 min=1544 max=1614
bing-cfg-carry L cells=635 total=-10831 min=-18 max=-17
bing-cfg-carry rand cells=313 total=-5321 min=-17 max=-17
bing-cfg H1 cells=70 total=96152 min=-21 max=1610
bing-cfg L cells=635 total=-13371 min=-22 max=-21
bing-cfg rand cells=313 total=-6573 min=-21 max=-21
jia-cfg-carry L cells=629 total=5032 min=8 max=8
jia-cfg-carry rand cells=313 total=2504 min=8 max=8
jia-cfg H1 cells=8 total=32 min=4 max=4
jia-cfg L cells=629 total=2516 min=4 max=4
jia-cfg rand cells=313 total=1252 min=4 max=4
jia-slot H1 cells=132 total=0 min=0 max=0
jia-slot L cells=632 total=0 min=0 max=0
jia-slot rand cells=532 total=0 min=0 max=0
yi-cfg-carry H1 cells=124 total=193722 min=1474 max=1643
yi-cfg-carry L cells=629 total=4988 min=7 max=8
yi-cfg-carry rand cells=313 total=2503 min=7 max=8
yi-cfg H1 cells=132 total=193254 min=3 max=1639
yi-cfg L cells=629 total=2472 min=3 max=4
yi-cfg rand cells=313 total=1251 min=3 max=4
yi-narrow-cfg-carry H1 cells=100 total=4120 min=-40 max=120
yi-narrow-cfg-carry L cells=629 total=4988 min=7 max=8
yi-narrow-cfg-carry rand cells=313 total=2503 min=7 max=8
yi-narrow-cfg H1 cells=108 total=3748 min=-44 max=116
yi-narrow-cfg L cells=629 total=2472 min=3 max=4
yi-narrow-cfg rand cells=313 total=1251 min=3 max=4
yi-narrow-slot H1 cells=194 total=1657 min=-52 max=108
yi-narrow-slot L cells=632 total=-40 min=-1 max=0
yi-narrow-slot rand cells=532 total=-73 min=-1 max=0
yi-slot H1 cells=194 total=96053 min=-1 max=1631
yi-slot L cells=632 total=-40 min=-1 max=0
yi-slot rand cells=532 total=-73 min=-1 max=0
```

- 合法状态上：乙-配置 每格 +3 或 +4、乙-配置续 +7 或 +8，甲 两臂 +4 / +8，丙 两臂比今天**少**读（−21 / −22、−17 / −18，每格都比今天少）。格数只在同格数的臂之间可比（-配置 各臂 629、丙 635、-槽 632，拒的格不同）。
- **甲-槽、乙-槽、乙-窄读-槽 的 Q5-同成 按 V1 ④ 作废**（PC-多读 (ii) 登记预言 +3S = 24、量到 0，读代码推的 +0 在装置之前就交了主 agent，认定第 1 项定为走 F18、作废）；表里 -槽 三臂那几行只当明细。
- H1d 上乙 的重读（N_真 为真、故障在 R 轮之内撤掉、重读之后挂上）的代价，整行（compare 第 657、896 行）：

```text
E7RESULT name=r4_q5 arm=yi-cfg column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 positive_cells=32 total_extra_read_calls=50136 total_extra_read_bytes=207872000 minimum_extra_read_calls=1488 maximum_extra_read_calls=1639
E7RESULT name=r4_q5 arm=yi-narrow-cfg column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=24 positive_cells=23 total_extra_read_calls=942 total_extra_read_bytes=5744640 minimum_extra_read_calls=-38 maximum_extra_read_calls=111
```

**第 22 条那一格（Q6）**：L5 每一格 `Ok` 且隔离总数 < 孪生镜像上不注入时的 ⇒ 没罩住。今天那一臂没罩住 3 / 72（n1 = 1、2、3，b = unit_first_copy，m = 2，O(2 起)），候选各臂 0：

```text
E7RESULT name=r4_q6 arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=3 verdict=not_covered
E7RESULT name=r4_q6 arm=yi-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=yi-cfg-carry geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
```

Q6 对候选各臂不算判据（「罩住」由臂的定义推得出，登记第六节开头），判别力在今天那一臂：PC-22 的 [今] 句 `pass`。L5 这个构造分不出乙与甲：乙 三臂在 L5 上没有一格是重读之后读出、挂上的（O(m 只) 只打坏两份合计的第 m 次读，另一份读得出，走不到第 22 条那一形；O(m 起) 一直读坏，重读也读坏）。

**阳性对照与 V1 作废范围（以五段合并的 compare 为准）**：跨臂句子按对照、标签、判定数：

```text
$ grep 'name=r4_cross_arm_sentence' research/results/e158-root-choice-repair-2026-09-27-r4-compare.out | awk '{c="";v="";t="";for(i=1;i<=NF;i++){if($i~/^control=/)c=substr($i,9); if($i~/^verdict=/)v=substr($i,9); if($i~/^tag=/)t=substr($i,5)}; print c, t, v}' | sort | uniq -c
      3 PC-22 arm fail
     12 PC-22 arm not_constructible
     17 PC-22 arm pass
      1 PC-22 today pass
     24 PC-extra-reads-ii arm fail
     64 PC-extra-reads-ii arm pass
     10 PC-extra-reads-iii arm fail
     38 PC-extra-reads-iii arm pass
     20 PC-extra-reads-iii measure pass
      6 PC-extra-refusal arm fail
     11 PC-extra-refusal arm pass
     11 PC-extra-refusal measure pass
      6 PC-extra-refusal measure vacuous
     17 PC-extra-refusal today pass
     88 PC-N0 arm pass
      6 PC-N-cover arm pass
      6 PC-N-ring arm pass
      9 PC-random arm pass
```

各臂自己产物里的对照（PC-N-环、PC-N-盖、PC-O、PC-丢写、PC-N0 的逐臂句子、PC-554、PC-随）只有两行 `verdict=fail`：`…-r4-seg1-yi-narrow-cfg.out` 与 `…-r4-seg1-yi-narrow-cfg-carry.out` 各自 `control=PC-N-cover form=reads_zeros duration=t1` 那一行（PC-N-盖，读回全零、时长 T(1) 那一格三句 [臂] 打不中，`missed_routes=F16,F16,F16`）。`r4_void` 126 行里 `voided=true` 26 行，原样：

```text
E7RESULT name=r4_void arm=jia-cfg quantity=Q6 voided=true basis=PC-22:refused_only(arm)
E7RESULT name=r4_void arm=jia-cfg-carry quantity=Q6 voided=true basis=PC-22:refused_only(arm)
E7RESULT name=r4_void arm=jia-slot quantity=Q5-both-mounted voided=true basis=PC-extra-reads-ii:point_value[n1=0 c=C](arm),PC-extra-reads-ii:point_value[n1=0 c=U](arm),PC-extra-reads-ii:point_value[n1=1 c=C](arm),PC-extra-reads-ii:point_value[n1=1 c=U](arm),PC-extra-reads-ii:point_value[n1=2 c=C](arm),PC-extra-reads-ii:point_value[n1=2 c=U](arm),PC-extra-reads-ii:point_value[n1=3 c=C](arm),PC-extra-reads-ii:point_value[n1=3 c=U](arm)
E7RESULT name=r4_void arm=jia-slot quantity=Q6 voided=true basis=PC-22:refused_only(arm)
E7RESULT name=r4_void arm=yi-cfg quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm),PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=reads_zeros](arm)
E7RESULT name=r4_void arm=yi-cfg-carry quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm),PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=reads_zeros](arm)
E7RESULT name=r4_void arm=yi-cfg-carry-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-cfg-carry-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-cfg-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-cfg-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-narrow-cfg quantity=Q1-Q3 voided=true basis=PC-N-cover:destination_chosen_or_applied[form=reads_zeros duration=T1](arm),PC-N-cover:ok_after_one_round_chosen_hidden_no_rollback[form=reads_zeros duration=T1](arm),PC-N-cover:row_publish_txg_is_hidden_txg_plus_one[form=reads_zeros duration=T1](arm)
E7RESULT name=r4_void arm=yi-narrow-cfg quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm),PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=reads_zeros](arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-carry quantity=Q1-Q3 voided=true basis=PC-N-cover:destination_chosen_or_applied[form=reads_zeros duration=T1](arm),PC-N-cover:ok_after_one_round_chosen_hidden_no_rollback[form=reads_zeros duration=T1](arm),PC-N-cover:row_publish_txg_is_hidden_txg_plus_one[form=reads_zeros duration=T1](arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-carry quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm),PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=reads_zeros](arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-carry-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-carry-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-narrow-slot quantity=Q5-both-mounted voided=true basis=PC-extra-reads-ii:point_value[n1=0 c=C](arm),PC-extra-reads-ii:point_value[n1=0 c=U](arm),PC-extra-reads-ii:point_value[n1=1 c=C](arm),PC-extra-reads-ii:point_value[n1=1 c=U](arm),PC-extra-reads-ii:point_value[n1=2 c=C](arm),PC-extra-reads-ii:point_value[n1=2 c=U](arm),PC-extra-reads-ii:point_value[n1=3 c=C](arm),PC-extra-reads-ii:point_value[n1=3 c=U](arm)
E7RESULT name=r4_void arm=yi-narrow-slot quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm)
E7RESULT name=r4_void arm=yi-narrow-slot-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-narrow-slot-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-slot quantity=Q5-both-mounted voided=true basis=PC-extra-reads-ii:point_value[n1=0 c=C](arm),PC-extra-reads-ii:point_value[n1=0 c=U](arm),PC-extra-reads-ii:point_value[n1=1 c=C](arm),PC-extra-reads-ii:point_value[n1=1 c=U](arm),PC-extra-reads-ii:point_value[n1=2 c=C](arm),PC-extra-reads-ii:point_value[n1=2 c=U](arm),PC-extra-reads-ii:point_value[n1=3 c=C](arm),PC-extra-reads-ii:point_value[n1=3 c=U](arm)
E7RESULT name=r4_void arm=yi-slot quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm)
E7RESULT name=r4_void arm=yi-slot-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-slot-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
```

| 作废的量 | 臂 | 依据的句子（标签） | 怎么处置 |
|---|---|---|---|
| Q1–Q3 | 乙-窄读-配置、乙-窄读-配置续 | PC-N-盖 读回全零、时长 T(1) 那一格三句（[臂]）：窄读把读回全零的记录当 journal 空槽收进缓存，E 那次发布的末条计数器找不到，按 N 为真拒 | 第一段之后主 agent 认定第 2 项：记一次输，这两臂在第 4 次跑里 Q1–Q3 不可判，不改窄读的收法、不另开一次跑（用户已定乙，窄读是对照臂） |
| Q5-同成 | 甲-槽、乙-槽、乙-窄读-槽 | PC-多读 (ii) 点值 +3S = 24、量到 0（[臂]） | 装置之前主 agent 认定第 1 项：走 F18，不改臂行 |
| Q5-拒（附带量） | 六条乙臂（配置、配置续、槽 × 乙、乙-窄读；-槽 两臂只在 `read_fails` 上） | PC-多读 (iii)「乙-X 第一遍的读 = 甲-X 整次」，PC-N-环 上差 1（1573 对 1574、1569 对 1570） | 第一段之后主 agent 认定第 3 项：记一次输；成因（读缓存去重）是执行员推的、没量证，不采信、不查；「选哪个」用同成栏，不受影响 |
| Q6 | 甲-配置、甲-配置续、甲-槽 | PC-22「甲在那个 m 上拒」的 O(m 只) 那一半（`refused_only`，[臂]）：O(2 只) 只打坏一份的一次读、另一份读得出，甲照定义照常挂载、隔离 28 = 孪生 | 第四段之后主 agent 认定第 1 项：记一次输，是登记里这一句的构造写错，不改臂、不改构造 |
| Q4、Q6 | 六条 R = 3 臂 | PC-多拒 `arm_refuses`（[臂]）`fail`、PC-22 两句（[臂]）`not_constructible` | **五段合并之后才出现，各段 compare 里没有**：R = 3 臂只跑第五段的 `h1d-uncut-r3`、`h1e-r3` 两族，没有 L 族；合并之后今天那一臂有了 L 族的格，compare 在这六臂上找不到 L3 那一格，把「没有格」记成没拒、判 `fail`（`r4-compare` 源码里 `arm_cell` 为空时 `refused=false`，读代码核过），PC-22 那两句则记 `not_constructible`。这六臂没有一行 `r4_q4` / `r4_q6`，作废不碰任何数；是 compare 的写法把「构造不出」记成了「打不中」，交主 agent |

- V1 ③「装置缺陷（全部臂一起）」没有出现（compare 里没有 `r4_device_defect` 行）。今天那一臂的 [今] 句全 `pass`（PC-多拒 17 句、PC-22 1 句；各分段 compare 里因为前提在别的段而记 `not_constructible` 的那 12 句，合并之后都有了格、都 `pass`），V1 ⑤ 没有触发任何 F 条款。
- 7.3 与跨臂核对（compare 原样）：

```text
E7RESULT name=r4_section_seven_three row=4 arm=today cells=1896 nonzero_cells=0 verdict=pass
E7RESULT name=r4_section_seven_three row=2 witness=device_n_cfg jia_arm=jia-cfg cells=1856 mismatches=0 verdict=pass
E7RESULT name=r4_section_seven_three row=2 witness=device_n_slot jia_arm=jia-slot cells=1880 mismatches=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=bing-cfg cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=bing-cfg-carry cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=jia-cfg cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=jia-cfg-carry cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=jia-slot cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=today cells=110 differences=0 verdict=pass
E7RESULT name=r4_f11 cells=1552 cells_differing_outside_o1_and_cell_22=0 verdict=inference_holds
```

`r4_base_identity` 18 行全 `verdict=pass`、`differing=0`；十二条主族臂各 `prefixes=60`，六条 R = 3 臂 `prefixes=0`（没有跨臂比的前缀，这六行是空判）。

**F / S / V 条款，各段**（数取自各段臂产物的 `r4_family_summary`、`r4_stop_check`、`r4_failure_clause` 等行与五段合并的 compare；逐行原样在各段报告里）：

| 段 | 条款 | 结果 |
|---|---|---|
| 开跑前 | S0 | 不停：快照上逐条重核七件（择根那一遍仍丢掉读不出的根、可写挂载仍只判「落后」、取号仍写 tail = 0、发布仍在根槽 FUA 之后轮换系统配置、`rebuilt_allocator` 仍再择一次根、重放仍在第一份点名单元读不出时停、影子账仍只计数），`first_txg_of_new_instance` 与 `target_for_publish` 两件也没变（登记修订第 3 条，行号在那里） |
| 各段 | V3、V4、S1–S4 | 开跑检查（本地常量回比、臂探针 `r4_arm_code_check`、7.2 锚点）每段每臂全 `pass`；各族 `s4_mismatches=0` |
| 第一段 | F4、F6、F8、F13、F15、F16、F17 | 都没触发：今天那一臂 H1d `f4_failures=0 f6_checked=48 f6_findings=0 f13_checked=48 f13_failures=0 f15_checked=8 f15_findings=0`（`…-r4-seg1-today.out` 的 `r4_family_summary family=h1d` 行）；今天那一臂「环在且被抛弃」∪「根槽被盖」H1d 48、H1e 64（F8 的前件不成立）；PC-N-环、PC-N-盖 今天的 [今] 句全 `pass`；乙-配置、乙-配置续 在 H1d 不断格与 H1e m = 1 格的 T(1)、O(1 只) 上都 K0 且所选根 = 被藏那条根 |
| 第二段 | S5a、S5b、S7、S9、F19 | S9 的种子与写死历史 6 项全 `pass`；装置执行器与 harness 执行器逐步对拍五族 3070 步 `mismatches=0`（S7、S9 不停）；S5：实七-甲、实八两边都复现（`verdict=reproduced`），**实七-乙 harness 与装置两边都复现不出**（`…-r4-seg2b-today.out` 里 `verdict=S5b_not_reproduced_on_the_base` 那一行，S5b：不停、全部臂照跑、标「底座上复现不出」；那一段今天那一臂主路径第 4 步那一格仍是「环在且被抛弃」、丢写 1 格，其余 4 格是崩溃状态格、都读回最后确认那一版）；F19 没触发（第 1239 行 `segments_without_an_abandoned_red=0`）；PC-随 12 臂 36 句与 compare 9 句全 `pass` |
| 第三段 | F7、F16 | F7 没触发：H1g(a = 2) 上甲-配置 16 / 16 交回 K0（推的 ② 成立）；F16 在 H1f 上没触发：乙-配置、乙-配置续 不断的 T(1)、O(1 只) 格全部 `Ok`、所选根就是被藏那条根（各 32 格）；H1g 的取号崩溃核对 72 次全 `holds=true` |
| 第四段 | F1、F2、F5、F14、F11 | PC-554 12 臂全 `pass`（今天那一臂 `a_abandoned_roots_unreadable=0 a_isolated=0`、被抛弃根独占 24 个槽一个没隔离，F1 没触发）；PC-22 今天那一句 `pass`（F2 没触发）；F5（L6 8 格）、F14（每臂 4 次卸载，12 臂 48 行）全 `pass`；F11 见 compare 的 `r4_f11` 行 |
| 第五段 | 8.2 几何敏感性 | 判决格 210 格（丢写 120、多拒 66、Q6 24）逐格比 G0（主几何点）全部「不翻」。k = 2 与 R = 3 两点推动了机制（k = 2 上「根槽被盖」变 0；R = 3 上时长 T(2)、T(3) 从拒变挂上），12 格两点值不同、把门槛挪到两值之间都翻，判别力自证过；**G_S4、G_S16、G_默认、G_小环 四点没把机制推动**（前缀最多 6 次发布，根环与记录都没绕环，H1e@G_小环 64 格 `device_undecidable=false`），这四点上的「不翻」是两点同值，不算判据（第五段之后主 agent 认定第 1 项）。出处：第五段报告 `e158-r4-seg5-runner-report.md` 第四、五节 |

**判决行**：各段臂产物与各 compare 都不打 `name=verdict` 行（`grep -c 'name=verdict'` 在 86 份上都是 0），判定写在各行的 `verdict=` 字段里。五段合并的 compare 按取值数：`pass` 381、`covered` 33、`fail` 43、`not_constructible` 12、`vacuous` 6、`not_covered` 3、`inference_holds` 1；`fail` 43 = PC-多读 (ii) 24 + PC-多读 (iii) 10 + PC-22 甲 3 + R = 3 臂 PC-多拒 6，`not_constructible` 12 与 `vacuous` 6 都在六条 R = 3 臂上，`not_covered` 3 是今天那一臂在 G0（主几何点）、S16、S4 上的 Q6；`r4_missed_arm_and_today_sentences count=61` = 这 55 句加乙-窄读 两臂 PC-N-盖 各 3 句。各段臂产物里只有乙-窄读 两臂第一段第 71 行两个 `verdict=fail`；第二段今天那一臂的 `S5b_not_reproduced_on_the_base` 是登记预先写好的分支。

**它答不了的（第 4 次跑）**：

- **读缓存峰值没量**（岔路单第 3 行的够判条件要它）：第 4 次跑的登记里没有这个量，不为它加臂；乙 与乙-窄读 只报了每次挂载多读的读调用次数。第 3 行记欠，落在第四批 kb 规格里 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙 Q4 那一笔。
- **H-随 那 6 格怎么读**：照登记字面全算丢写，全是 ③c 尾巴，其中 2 格同时有 Q1 覆盖、那一版早已被后来的确认取代；这几格该不该算丢写，第 4 次跑只并列两栏，不判。
- **根环大小、环长两条轴上的敏感性没量到**：G_S4、G_S16、G_默认、G_小环 上的前缀最多 6 次发布，根槽按 (txg mod 3, (txg div 3) mod S) 落、txg div 3 ≤ 2，S 取 4、8、16 落点不变，环最小 12 个记录槽也写不满（推的，没量过）。要量得另立更长前缀（让根槽在 S = 4 上回绕、让记录在小环上回绕）的一族，登记外，记作下一次跑的候选。
- **第 22 条那一格上乙与甲分不开**：L5 的两种时长都走不到「第二次读实例表失败、重读读得出」，乙 的重读在那一格上没有被测到。「两份各读坏一次、重读读得出」那种时长这一次不补（补格是事后建模；它只分甲与乙，而用户已定乙），要分就登记进下一次跑。
- **PC-多读 (ii) 丙 那一句的 p**（实例表沿链读的页数）取的是 `crates/` 的沿链读（`instance_table_chain_of_root(E).page_pointers.len()`），装置与 checker 都没有公开的实例表沿链读，这个数不是装置自己解出的。
- **PC-多读 (iii) 差 1 的成因**（乙 第一遍比甲 整次少 1 次读）没量证：执行员推的「读缓存去重」不采信、不查；作废的只是附带的 Q5-拒。
- -配置 各臂在实七-甲 上 3 个注入点 `fault_not_reached`、在实八 上 4 个崩溃状态 `segment_missing`，这几格没立；实八「根还在环里、单元被复用」那一格在 -配置 各臂上只量到主路径。实七-乙 那个签名（崩溃状态读回报 `MappingStillUnreadable`）在底座上两边都复现不出（S5b）。
- 管理员回退的另一种读法（回退交回 `Ok` 也算一笔确认）只报数、不进判据（登记修订第 22 条）：-配置 各臂 H-随 两种读法都是 6 格（`lost_write_cells_rollback_aware=6`），今天那一臂两种读法都是 43 格、Q3 读回丢写 6 对 5。
- **量的是臂副本，不是合进 `crates/` 的实现**：工作树的 `crates/` 在快照之后已合进 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续 的实现（`crates/singlefs-core/src/mount.rs` 里 `PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion` 那一支），它与第 4 次跑的乙-配置续 臂（`scripts/mutation/tables/e158_arms.tsv` 的 `r4` 行）是不是同一个行为，没有对拍。
- 第二段的历史由 harness 的生成器定几何（4 GiB 盘），不做几何敏感性；H1d 除 k = 2 那一点外只藏最新 1 条根。
- seg2b 的产物首行指纹 `ad9f5442…` 没把 seg2b 修过的装置（`e71224fc…`）算进去，第二段两次的首行指纹相同，靠文件名与装置 sha256 分。

**单测与变异**：装置单测 95 条（`grep -c '#\[test\]' crates/singlefs-verification/src/bin/e158_root_choice_repair.rs` 输出 `95`：装置那一趟 82 条、第二段装置加 12 条、seg2b 加 1 条撞键的单测）。第 4 次跑的装置变异在 `crates/mutations.tsv` 里 24 行（`grep -c 'root_choice_repair 第 4 次跑' crates/mutations.tsv` 输出 `24`），归门禁 crates-mutation-replay，三个数待提交时 crates-mutation-replay；各趟执行员在草稿副本上逐条证红：装置那一趟 11 条抓到 11 / 无效 0 / 没红 0、第二段 11 条 11 / 0 / 0、seg2b 在主工作区副本加补丁上 12 行 12 / 0 / 0（含撞键那两条）。臂表变异 `scripts/mutation/tables/e158_r4_arm_mutations.tsv` 8 行：装置那一趟 8 / 0 / 0，第三段另跑了 M3 的臂那一半 1 / 0 / 0（H1g a = 1 上甲-配置 由 K3 变 K0、32 格丢写）。

**复跑**：`bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路） 里的 `driver_e158_r4_compare` 那一行复跑五段合并的 compare（只读 `research/results/` 下 seg1、seg2b、seg3、seg4、seg5 五组臂产物，不碰 `crates/`，产物头的 `E7INPUT` 行比对前删掉）。各段的臂产物不登记：它们要在 C0 快照上套 `scripts/mutation/tables/e158_arms.tsv` 的 `r4` 行编出来的副本上跑，工作树的 `crates/` 已不是今天那一臂。在副本里复跑一条臂的写法（`<段>` 取 `r4-seg1`…`r4-seg5`，第二段写 `r4-seg2`）：`task-server/task submit --cwd <副本根> -- env SINGLEFS_E158_ARM=<臂> SINGLEFS_E158_SNAPSHOT_SHA256=<指纹> SINGLEFS_E158_TODAY_BASES=<今天那一臂先跑 r4-bases 写出的目录> cargo run -q --release -p singlefs-verification --bin e158_root_choice_repair -- <段>`；第二段另要 `SINGLEFS_E158_INVESTIGATOR_REPORT=research/prompts/m2-investigate-gate74-reds-report.md` 与（今天之外的臂）`SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT=<今天那一臂的第二段产物>`，今天那一臂先跑；跨臂的量再跑 `r4-compare <各段产物前缀…>`。

**第 3 次跑第一段（重跑登记 `e158-r3-prereg.md`；只当参考）**

第 3 次跑只跑了第一段（H1d、H1e 与阳性对照，G0（主几何点），九条臂：今天、甲-配置、甲-配置续、甲-槽、乙-配置、乙-配置续、乙-槽、丙-配置、丙-配置续），第二段没跑；第 4 次跑的登记把它原本要跑的全部并进第 4 次跑，第一段量过的也在另一版底座（第 4 次跑的 C0 快照）上重跑（第 4 次跑登记文件头）。第 3 次跑第一段的读数因此只当参考、不当依据。它交回的两处阳性对照没过（执行员报告 `e158-r3-runner-seg1-report.md` 第一节）：PC-N 今天那一臂 `verdict=fail`（b = records 时新实例第一次发布的 txg 等于被藏那条根的 txg，写行那次发布盖掉被藏根的槽），按那次登记的 V1 字面会作废全部臂的 Q1–Q3；PC-多读 乙 三臂 `verdict=fail`。第 4 次跑把 PC-N 拆成 PC-N-环 与 PC-N-盖、把 V1 按句子标签分路，出处是第 4 次跑登记 5.8 与 11.1。

产物（快照指纹 `74912842…a41a`，写在每份第一行）：

- 九条臂与跨臂比：`e158-root-choice-repair-2026-09-27-r3-seg1-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-compare.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-today.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-yi-slot.out`。
- `r2-all` 在那份快照上重出的今天那一臂：`e158-root-choice-repair-2026-09-27-r2-all-today.out`（1055 行，末行 `E7RESULT name=done emitted=1054`，`replay.sh` 的 `driver_e158_r2_all` 那一行改指它）。它与 `…-2026-09-26-r2-all-today.out` 对不上：`r2_h1d_cell` 800 → 832 行、`r2_seventh_batch_crash_finding` 1 → 0 行（第 3 次跑执行员报告第二节）。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:438-531`（整段抄，未转述）**

````markdown
### 2026-09-26：第 2 次跑第一段（重跑登记 `e158-r2-prereg.md`）

「2026-09-26：第 2 次跑第一段（重跑登记 `e158-r2-prereg.md`）」这一节是第 2 次跑第一段的记录（现状在「2026-09-27：第 4 次跑」一节）。它之后、「影响的决策」之前的各节都是第一次跑（跑前登记 `e158-preregistration.md`）的记录：那次的「被抛弃根」全靠挂载时回退造，今天的代码里没有那条路（D23（journal 的角色与格式） 已定项 14「管理员回退是挂着时的一次向前发布」），那几节的读数不带进第 2 次跑。H1c、H1d、Q1-0、K0–K4、op1 的三种入口调用、S0–S6、F1–F12 是重跑登记自己的代号，登记位在那份文件里。

**做了什么**：入库装置 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 加了第 2 次跑那一节（模式 `r2-all` 等；第一次跑的各模式与 `replay.sh` 里它们那几行原样留着），在八份副本上各跑一次 `r2-all`：今天那一臂（工作树）、A1（岔路单第 1 行 (a)）、甲-txg、乙F-留环、乙-留环、乙-只配置、丁-留环、丁-只配置（岔路单第 2 行六臂）。七份臂副本都从同一次快照（`git rev-parse HEAD` = 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，快照时 `crates/` 与工作树逐文件 sha256 相同）`cp -a` 出来，按 `scripts/mutation/tables/e158_arms.tsv` 逐行套用（每行旧串在副本里恰好命中一次，S2），臂改动之外的 `crates/` 与快照逐文件相同（S4），七份都编得过（S3）。岔路单第 1 行的 (b) 原形、(b) 含链、(c) 不在 `crates/` 里实现（重跑登记第三节末句），从今天那一臂的计数推出（重跑登记 12.1 第 2 条）。开跑前的停机条款 S0、S1 按登记第一节、第三节的行号逐项重读 `crates/singlefs-core`（`first_txg_of_new_instance` 在 `mount.rs:599`、`isolate_slots_referenced_only_by_abandoned_roots` 在 `:684`、账读不出只计数在 `:714`、`MountSpaceAdmission` 在 `:370`、写行之后推在 `:2744`、`unmount` 在 `:1349`），与登记写的一致、行号没漂。登记修订 7 条写在重跑登记 12.1，都在产物之前。

**前提 1–3（问题单第一节）三条都成立**：

| 前提 | 答 | 依据 |
|---|---|---|
| 前提 1（被抛弃根今天只由崩溃恢复造出） | 成立 | Q1-0：H1c 抛弃步（故障一律读错）111 次尝试、87 次挂载做成、56 次造出至少一条被抛弃根，最少 2 个故障（最新根的根槽 1 + 断链发布第一个点名单元的第一份 1）；产物行在「结果整行抄自产物」一节。b = unit（两份都坏）那 25 次第二份一次都没被读到（重放验点名单元是全部份都要过，第一份读错就停），照 V2 作废；只坏第一份就断得了链（重跑登记 12.1 第 1 条） |
| 前提 2（C331（择根倒挂压过已确认的写） 今天仍打得中、乙要重定义） | 成立 | `first_txg_of_new_instance`（`mount.rs`）仍只取可读根与自证记录的 max + 1、不读系统配置的 F；今天那一臂 87 次写行发布的 txg 与装置按这次读得出的根与记录算的 max + 1 逐次相等（`first_txg_rule_of_today_mismatches=0`）。乙族照重跑登记 5.3 分成加 8 字节的乙 / 丁四臂与零字节的乙F-留环；乙 / 丁四臂的副本在 [489, 497) 写出刚落盘那条根的 txg（关闭 U 之后解出 7 = 最新根的 txg，产物行在「结果整行抄自产物」一节） |
| 前提 3（准入改了形态，「被拒」重定义） | 成立 | `MountSpaceAdmission` 四个成员、写行之后推都照旧；装置按 K0–K4 分类，K3 / K4 按这次调用设备一层的写次数分（单测钉住）。G0（主几何点） 的 4 GiB 盘上这一段见到的挂载全是 K0、K3 或注入写失败那几次的 K4，K1、K2 一次没出现——G_小盘这一段没跑，「K1 / K2 走不走得到」还没量 |

```text
E7RESULT name=r2_q1_0_summary arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) attempted=111 not_applicable=24 void_attempts=25 mounted=87 producing_at_least_one_abandoned_root=56 abandoned_roots_total=202 fewest_faults_that_produced_one=2 first_txg_rule_of_today_mismatches=0 row_publish_txg_at_or_below_the_floor=0
E7RESULT name=r2_decoded_offsets arm=today verdict=pass tail_offset=469 instance_offset=477 floor_offset=481 registered_tail=469 registered_floor=481
E7RESULT name=r2_unmount_check arm=today context="q1-0 n1=1" verdict=pass layout_verdict=pass txg_before_unmount=4 floor_in_newest_system_configuration=[4, 4] unmount_publishes=3 expected_publishes=3 last_nonzero_byte=[481, 481] bytes_489_to_497_as_txg=[0, 0] newest_root_txg=7
```

乙-留环那一份（`e158-root-choice-repair-2026-09-26-r2-all-yi-keep-ring.out`，不在 `replay.sh` 登记里，下同）同一格 `last_nonzero_byte=[489, 489] bytes_489_to_497_as_txg=[7, 7] newest_root_txg=7`：[489, 497) 解出的正是刚落盘那条根的 txg。

Q1-0 在另外几臂上（同一套历史；各臂 `name=r2_q1_0_summary` 那一行的几个字段）：

| 臂（产物） | `producing_at_least_one_abandoned_root` | `abandoned_roots_total` | `fewest_faults_that_produced_one` | `first_txg_rule_of_today_mismatches` | `row_publish_txg_at_or_below_the_floor` |
|---|---|---|---|---|---|
| A1 | 56 | 202 | 2 | 0 | 0 |
| 甲-txg（`…-r2-all-jia-txg.out`） | 0 | 0 | none | 81 | 33 |
| 乙F-留环（`…-r2-all-yif-keep-ring.out`） | 24 | 65 | 4 | 81 | 0 |
| 乙-留环、乙-只配置、丁-留环、丁-只配置（各自那一份） | 62 | 208 | 2 | 6 | 0 |

甲-txg 一条被抛弃根都造不出来：新实例第一次发布的 txg 不看记录，恰好落回被藏那几条根的 txg，写进同一个根槽，把它们盖掉了（例 `n1=1 c1=C k=2 b=records`：所选根 (1, 2)、写行发布 txg = 3）；乙F-留环造出来的 24 次全在关闭 U 的历史上（F 把第一次发布的 txg 顶到被藏的根之上）。这几行是岔路单第 2 行的旁证，不是登记给第 2 行的量。

**H1d（崩溃恢复把最新的根当成读不出、抛弃它、取号，这次可写挂载在写行根落盘之前断；故障撤掉、再挂载）**：操作化见重跑登记 12.1 第 2 条（n1 ∈ {0, 1, 2, 3}、关闭 C、藏最新 1 条根、读错与读回清零两种造法、断链 b ∈ {records, unit, unit_first_copy}，断法取「不断 / 崩在第 j 个写之前 / 第 j 个写报错 / 写行根之前的每一次屏障报错」，j 取 0 到写行根那个写）。每臂 800 格（b = unit 那 8 个基准照 V2 作废、不进这 800 格）。三格：

| 臂 | ① 恢复落到的根引用的单元被覆盖 | ② 再挂载 | ③ 读回的是最后一次确认的那一版 | 与今天那一臂逐格比三格 |
|---|---|---|---|---|
| 今天（= 岔路单第 2 行的丙） | 128 / 800 | K0 680、K3 120（`MountError::Recovery`，冷启动读回 `MappingStillUnreadable`） | 664 / 800（另 120 读失败、12 读回别的版本、4 读回没有文件） | — |
| A1 | 128 / 800 | K0 678、K3 122 | 664 / 800 | 差 2 格：那 2 格的再挂载被拒（`AbandonedRootLedgerUnreadable`），三格里只有 ② 变 |
| (c)（两个指称） | 同今天 | 同今天 | 同今天 | 相同（(c) = 今天的实现加一笔落点未定的记录，不改择根与拒不拒） |
| (b) 原形、(b) 含链（各两个指称） | 同今天，除那 2 格 | 同今天，除那 2 格 | 同今天，除那 2 格 | 那 2 格再挂载时被抛弃根的账读不出计数 = 1，(b) 在那 2 格上重建不重建得出来这一段没算 |
| 甲-txg、乙F-留环、乙-留环、乙-只配置、丁-留环、丁-只配置 | 128 / 800 | K0 680、K3 120 | 664 / 800 | 800 格三格逐格相同 |

① 的 128 格全部落在 n1 = 0（被藏的最新根就是首个文件；站在它前一条根——写行之后的暖机根、树表 0 条——看，首个文件那次发布开的段是全空的，写行那次发布在自己的根落盘之前就把那段重新发出去）；n1 = 1、2、3 的 636 格一格都没盖、再挂载全是 K0、读回全是最后确认那一版。「挂载做完」那 16 格（n1 各 4 格）三格是：① 恢复落到新实例自己写的根、② K0、③ 读回的不是最后确认那一版（n1 ≥ 1 读回前一版的内容 12 格，n1 = 0 读回没有文件 4 格）——被当成读不出的那一版在每一臂上都丢了。按断法分（今天那一臂）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-26-r2-all-today.out 第 869 行（name=r2_h1d_summary）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-26-r2-all-today.out 第 870 行（name=r2_h1d_summary_by_cut）
E7RESULT name=r2_h1d_summary_by_cut arm=today cut=crash cases=344 cell1_overwritten_cases=56 cell2_remount_classes={"K0": 292, "K3": 52} cell3_read_back_after_remount={"failed:MappingStillUnreadable": 52, "last_confirmed": 292}
E7RESULT name=r2_h1d_summary_by_cut arm=today cut=none cases=16 cell1_overwritten_cases=0 cell2_remount_classes={"K0": 16} cell3_read_back_after_remount={"no_file": 4, "other_content": 12}
E7RESULT name=r2_h1d_summary_by_cut arm=today cut=write_fails cases=344 cell1_overwritten_cases=56 cell2_remount_classes={"K0": 292, "K3": 52} cell3_read_back_after_remount={"failed:MappingStillUnreadable": 52, "last_confirmed": 292}
E7RESULT name=r2_h1d_fork_one_derived arm=today fork_one_arm=(b)原形 referent=tree_table cases_where_it_differs_from_this_copy=2 cells=same_as_this_copy_except_those
```

a1 那一份（`…-r2-all-a1.out`）的 `name=r2_h1d_summary` 行与今天那一臂的 `name=r2_h1d_summary` 行只差 `cell2_remount_classes={"K0": 678, "K3": 122}` 与 `trigger_positive_cases=0`；岔路单第 2 行六臂那六份的 `name=r2_h1d_summary` 行 `cases`、`cell1_overwritten_cases`、`cell2_remount_classes`、`cell3_read_back_after_remount` 四栏与今天那一行逐字相同。逐格比三格用的命令（每臂一份三格表，与今天那一臂在快照上的那一份 diff；输出原样）：`cells() { grep "name=r2_h1d_cell " "$1" | sed -E 's/.* (n1=[0-9]+) tip=\([0-9, ]+\) (form=[a-z_]+) (break=[a-z_]+) (cut=[a-z_]+) (point=[a-z0-9]+) .*(cell1_overwritten=[a-z]+) .*(cell2_remount_class=K[0-9]) remount_error=([A-Za-z]+) .*(cell3_read_back_after_remount=[^ ]+) .*/\1 \2 \3 \4 \5 \6 \7 \8 \9/'; }`，结果 `a1 three_cells_differing=2`、`jia-txg three_cells_differing=0`、`yif-keep-ring three_cells_differing=0`、`yi-keep-ring three_cells_differing=0`、`yi-config-only three_cells_differing=0`、`ding-keep-ring three_cells_differing=0`、`ding-config-only three_cells_differing=0`。

**实七两段复现历史**（重跑登记 12.1 第 5 条）：今天那一臂两段都复现——故障注入大档种子基 + 110 在第 1 步（崩溃恢复抛弃根）注 `barrier_fails` 之后重开走读失败 `MappingStillUnreadable { slot: SlotNumber(50240) }`，崩溃注入快档种子基 + 16 第 3 步的崩溃状态同一形：

```text
E7RESULT name=r2_seventh_batch_fault_summary arm=today seed=7463871032432355223 drawn=6 new_findings=2 findings_reopening_to_mapping_still_unreadable=2 known_red_hits=0
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-26-r2-all-today.out 第 1022 行（name=r2_seventh_batch_crash_summary）
```

（快照上的今天那一臂 `…-r2-all-today-snapshot.out` 第二行是 `new_findings=2`、多一条崩溃状态上的 `claimed_state_missing_unit`：快照之后另一条线改了 harness 的记录核对器，工作树上不再报它。两份的其余各行逐字节相同。）乙-留环、乙-只配置两份与快照上的今天那一臂这一段逐行相同，丁两臂只有两处抽到的读序号不同、新发现相同；A1、甲-txg、乙F-留环三臂的历史一开头就走岔了（harness 的模型按今天的规则答第一次发布的 txg，这几臂的挂载与它对不上，抽到的注入点与崩溃点都不再落在那一步），这两段历史在这三臂上没罩住，这三臂的 H1d 只能看「H1d」那张逐臂三格表（① 覆盖、② 再挂载、③ 读回最后确认那一版）。

**PC1-a / PC1-b / PC1-c**（第一段只跑今天那一臂、(c) 与 A1 的 (a) 那一句）：今天那一臂三条都过，A1 的 PC1-a 在任何写之前拒：

```text
E7RESULT name=r2_positive_control arm=today control=PC1-a node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=2 unintercepted=[] class=K0 error=none writes=71 abandoned_roots_unreadable=1 expected_count_by_the_device=1 isolated_slots=0 unit=allocation_record_tree_root shared=false expectation=ok_count_equals_the_device_count verdict=pass
E7RESULT name=r2_positive_control arm=today control=PC1-b node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=2 unintercepted=[] class=K0 error=none writes=71 abandoned_roots_unreadable=1 expected_count_by_the_device=1 isolated_slots=0 unit=tree_table shared=false expectation=ok_count_equals_the_device_count verdict=pass
E7RESULT name=r2_positive_control arm=today control=PC1-c node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=1 unintercepted=[] class=K0 error=none writes=71 abandoned_roots_unreadable=0 expected_count_by_the_device=0 isolated_slots=0 exclusive_placements=24 isolated_slots_without_the_fault=28 expectation=ok_count_zero_isolated_zero verdict=pass
```

A1 那一份（`…-r2-all-a1.out`）的 PC1-a 行：`class=K3 error=AbandonedRootLedgerUnreadable writes=0 … expectation=AbandonedRootLedgerUnreadable_and_zero_writes verdict=pass`，同一段历史、同一个落点（`n1=1 c1=C k=1 b=unit_first_copy`、被抛弃根 (1, 4)、分配记录树节点两份），在任何写之前拒。

PC1-c 那一行就是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 钉的行为：被抛弃根自己的根槽读不出时计数 0、隔离 0（不注入时这次挂载隔离 28 个槽）。甲-txg 那一份两行 `verdict=fail`（`H1c 里找不到…的历史`）：甲-txg 下抛弃步一条被抛弃根都造不出来（见 Q1-0），PC1 不是岔路单第 2 行各臂的阳性对照，那两行只是照实报「这一臂上 PC1 摆不出来」。

**岔路单第 1、2 行这一段量到哪**：

| 岔路单行 | 这一段量了的 | 还差（登记的量） |
|---|---|---|
| 第 1 行（被抛弃根的账读不出时「修复」指什么） | 前提 1 的装置读数 Q1-0；PC1-a / b / c（今天、(c)、A1 的 PC1-a）；H1d 在 A1 与 (b)(c) 两个指称上的三格（A1 实跑，(b)(c) 由计数推） | Q1-1 至 Q1-5（G0（主几何点） 全族、op1 的三种入口调用、瞬时 / 持续）；(b) 两个指称的 PC 与 H1d 那 2 格上 (b) 重建不重建得出；G_S4、G_S16、G_小盘的敏感性格 |
| 第 2 行（C331（择根倒挂压过已确认的写） 怎么修） | 前提 2；六臂副本编成、臂表重写；H1d 在六臂上的三格；Q1-0 在六臂上的读数（旁证：甲-txg 与乙F-留环（关闭 C 时）让新实例的第一次发布落进被藏根的 txg、盖掉根环里那几条根） | Q2-1（H2、H2c）、PC2 与它的开关（5.3，这一段臂表里没加）、Q2-2a、Q2-2b 的解码核对之外的部分、第八节行 2 的敏感性格 |

**它答不了的（第 2 次跑第一段）**：H1d 的崩溃只取前缀那一种状态（崩溃点重放的段内子集没有枚举）；H1d 只在 G0（主几何点） 一个几何上跑、只藏最新 1 条根、只取关闭 C；岔路单第 2 行各臂的定义都只改新实例第一次发布的 txg，不改择根、也不改「要不要拒」，H1d 的三格在这六臂上相同是这几臂定义的直接后果——用户要改的那一形（崩溃恢复把暂时读不出的最新根当成被抛弃），重跑登记里没有一条臂是「系统配置说有更新的根时拒绝可写挂载」这种形状，这张表答不了那种修法；(b) 在 H1d 那 2 格上的结局没算；实七两段历史在 A1、甲-txg、乙F-留环上没罩住；第一节处理表里的删与改（shim、H3、Q3-1、N_w、PC3、PC-Nw、权重上限 12 与它们的单测、变异行、`replay.sh` 里指向它们的行）这一段没做。

**单测与变异**：装置单测 57 条（`grep -c '#\[test\]' crates/singlefs-verification/src/bin/e158_root_choice_repair.rs`），开工时红的 `mount_writable_trajectory_distinguishes_persistent_from_transient_faults` 照第一节处理表改了取被抛弃根的地方、转绿；`mount_writable_then_raise_floor_reaches_the_injected_fault` 改打分配记录树节点、断言 `RaisedFloor::abandoned_roots_unreadable > 0`。`crates/mutations.tsv` 末尾新加第 2 次跑那一节的 17 行（归门禁 crates-mutation-replay），在草稿副本上逐条改坏、跑点名的单测：17 条全红（其中「根槽写之前的屏障把紧挨在它之后的也算进去」第一遍没红，是取样点不敏感——那两种写法只在「前 0 个写」这一格上不同，补了这一格之后红），沿用的第 551、659 行在新单测上也都红。

**复跑**：（2026-11-09 该行已删，见「2026-11-09」一节 ⑥；下面是当时的复跑说明）`bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路） 里的 `driver_e158_r2_all` 那一行（工作树、今天那一臂：`cargo run -q --release -p singlefs-verification --bin e158_root_choice_repair -- r2-all`，2026-09-27 单独复跑这一行逐字节一致；同一编号下第一次跑的各行这一次复跑全都对不上，差在第一次跑之后 `crates/` 的字段表 481 → 489 与挂载时回退删除，不是这一段改的）。⚠️ 另一条线改了 `crates/singlefs-core/src/mount.rs`、`transaction.rs`（产物出来之后），之后在工作树上重跑 `r2-all` 已与登记的产物对不上：H1d 多出 32 格（写行根之前的屏障每个基准从 6 次变 8 次），`cell1_overwritten_cases=128`、K3 120 不变，读回最后确认那一版从 664 变 696；Q1-0、PC1 那几行不变。登记的产物照旧指改动之前那一版（与各臂副本同一份 `crates/`），要不要按新代码重出交主 agent。另外七臂：对快照 `cp -a` 一份整树，按 `scripts/mutation/tables/e158_arms.tsv` 里说明列以那一臂的臂名起头（几条臂同一行时用「、」连写）的行逐行替换（旧串恰好命中一次），再以 `task-server/task submit --cwd <副本根> -- env SINGLEFS_E158_ARM=<臂名> SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES=<今天、A1、甲-txg、乙F-留环 取 489；乙 / 丁四臂取 497> cargo run -q --release -p singlefs-verification --bin e158_root_choice_repair -- r2-all` 编并跑。

**产物**：`e158-root-choice-repair-2026-09-26-r2-all-today.out`（工作树，`replay.sh` 登记的那一份）、`e158-root-choice-repair-2026-09-26-r2-all-today-snapshot.out`（快照上的今天那一臂，与各臂比用它）、`e158-root-choice-repair-2026-09-26-r2-all-a1.out`、`e158-root-choice-repair-2026-09-26-r2-all-jia-txg.out`、`e158-root-choice-repair-2026-09-26-r2-all-yif-keep-ring.out`、`e158-root-choice-repair-2026-09-26-r2-all-yi-keep-ring.out`、`e158-root-choice-repair-2026-09-26-r2-all-yi-config-only.out`、`e158-root-choice-repair-2026-09-26-r2-all-ding-keep-ring.out`、`e158-root-choice-repair-2026-09-26-r2-all-ding-config-only.out`。每份末行是 `E7RESULT name=done emitted=N`，N 等于这份里 `E7RESULT` 行数。

**问题**：跑前登记 `e158-preregistration.md`——为里程碑「覆盖写、释放、回退与复用」收口表第 ③、④、7、16 行等「择根与修复」的四条岔路（C393（被抛弃根的账读不出时修复没有条款）、C331（择根倒挂压过已确认的写）的修法、C332（回退实例两个根都读不出时回退被撤销）、C334（切换的所选根没有会红的检查））配代价数。装置放在**入库装置**上：`crates/singlefs-verification` 新加只读 bin `e158_root_choice_repair`，驱动真实 `mount_writable` / `mount_rollback` / `recover`，不改 `crates/singlefs-core`、`crates/singlefs-checker` 的生产代码。第一段（2026-09-23）做跑前登记「5.8」第一段：停机条款 S1–S5 逐项对拍、5.7 常量回比、第七节全部锚点、H3（岔路 3 的历史族）不注入全跑出 N_w 与轨迹、Q3-2、Q2-3 的算术、PC-Nw、PC3——对应岔路单第 3 行（候选 2 那一半）与第 2 行的 ③。**这一次（2026-09-24）**先把第一段在今天的 `crates/` 上重跑、逐格诊断与入库产物的差异（`crates/` 在此期间落地了 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 定案，拿掉了「回退到无文件那一版」的拒绝），再做跑前登记「5.8」第二段：Q3-1（岔路单第 3 行候选 3 那一半：H3 × Φ3 的 P332（违例判据） 违例比例，今天那一臂）与第八节行 3 的敏感性格。岔路单第 1、2 行（第四、三段）与第 4 行（第五段，前置未满足）这一次没有做，见「它答不了的」。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:532-544`（整段抄，未转述）**

```markdown
### 这一段做了什么

1. **S1（开跑前重读第三节表里每个函数）**：现查 `choose_root` 比较键 `(checkpoint_txg, instance)`（不读实例表，在 `crates/singlefs-core/src/recovery.rs` 里）、`first_txg_of_new_instance` 取 `max(可读根最大 txg, 环里自证记录最大 txg)+1`、`mount_writable` 的 `next_counter` 取环里最大计数器 + 1（jsn，= 甲-jsn = 今天）、`mount_rollback` 的 `next_counter` 同取环里最大计数器 + 1、`isolate_slots_referenced_only_by_abandoned_roots` 读不出账时只 `unreadable += 1; continue`（不隔离、不拒绝挂载，在 `mount.rs` 里）、`SystemRuntimeQuantities` 恰 3 个字段（`slot_generation`、`journal_tail`、`journal_instance`）。全部与第三节写的相符，S1 不触发。S2/S3（臂副本编译）本段不适用——只跑「今天」这一臂，不建 `e158_arms.tsv` 副本。**S4**：`git rev-parse HEAD` = `3b60f098e97dc4c4f3ed9c6355422b607db1c34c`、`git status --porcelain crates/` 74 行、`sha256sum crates/singlefs-core/src/*.rs crates/singlefs-format/src/*.rs | sha256sum` = `c23b2008600d66067f63373474059f0aa62af1f9ab227a4fac20e8b95e704fc8`，开跑前与交回前两次相同（主工作区在这一段稳定）。**S5**：装置用 `singlefs_checker::image`（另一套独立解析代码）解根环，与 `singlefs_core::recovery::readable_roots` 逐条比对（`name=s5_independent_decode_cross_check verdict=pass readable_root_count=6`），另有单测 `bootstrap_timeline_matches_independent_decode` 做同一件事的最小版本。
2. **5.7 常量回比**：18 个本地常量（根环区域数、每区槽数上下界与 mkfs 默认、journal 环长/记录宽/起点槽、落点粒度、系统配置字段表合计/槽宽/每盘槽数、根记录宽、实例表行宽与一片记录数、系统配置整槽校验和偏移、三个区域住哪块盘）逐条与 `crates::` 同名常量比对，18 条全 `pass`（`name=local_constant_check` × 18）。
3. **第七节全部锚点**：7.2 的 11 条独立算术 + 3 个几何点的根槽总数与 Φ3 故障集合个数 + 3 个环长的记录数/在飞上限，共 20 条全 `pass`；7.1 里能在本段核的 1 条（D22（单元原子性怎么合成） 已定项 9「481/3615/512 没跨」）`pass`，另外 4 条（C332（回退实例两个根都读不出时回退被撤销）「2 个故障」、C393（被抛弃根的账读不出时修复没有条款）「只计数」、D23（journal 的角色与格式） 已定项 14「N_switch=3」、C334（切换的所选根没有会红的检查） 判别力自证）与 7.3 的 4 行都标 `deferred`，留给对应段。
4. **H3 全枚举（不注入）**：四个几何点——`GEOMETRY_PRIMARY`（S=8，环 3 MiB）、`S16`（S=16，环 3 MiB）、`小环`（S=8，环 49152 字节，8.2 的搜法从 48 KiB 起找到的第一个跑得通的值）、`S4`（S=4，环 3 MiB，另跑 L=4）——每个几何点在 overwrites_before_sigma∈{1,2,3} 上穷举 σ∈{W,M,R(j)}^L（R(j) 取那一刻装置自己解出的每一条可读根），逐节点算「还有可读被抛弃根的回退事件数」（events_live）。`GEOMETRY_PRIMARY`/`S16`/`小环` 三个几何点历史数、峰值、正步数逐字段相同（856 段历史，峰值 3），`S4` 因根环容量小（12 槽）在 L=4 下出现峰值 4（7264 段历史）。四个几何点合计 N_w = max(3,3,3,4) = **4**。剪枝分支只有两类（`RollbackTargetNotACandidate`、`RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion`），逐条现查确认是候选集排除与「树表 0 条时环里还有带文件版本的根」两种合法拒绝，不是装置的错。
5. **Q3-2/Q2-3 算术**：单条见证 16/12 字节都放得下（512 以内余量 15/19）；表形态 `1+16*N_w`（65 字节）与 `1+12*N_w`（49 字节）都推过 512（余量 −34/−18），与候选 2 见证同放系统配置槽（沿用 N_w）时余量 −42/−26，同放也推过 512。四种宽度在 4096 以内都放得下。
6. **PC-Nw**：单测 `pc_nw_counts_two_live_rollback_events_from_a_hand_written_ledger`（手写两次回退、两段被抛弃根都读得出的假账，不碰 `crates/`）数出 2；对照单测 `pc_nw_does_not_count_a_rollback_event_whose_abandoned_roots_are_all_unreadable` 数出 0。
7. **PC3**：H3 全族里按固定次序第一个满足 events_live≥1 的历史是 `[Overwrite, Overwrite, Overwrite, RollbackTo(1, 3)]`（`GEOMETRY_PRIMARY`）——三次覆盖写之后回退到根 (1,3)，被抛弃的 (1,4)(1,5)(1,6) 仍可读。F = 回退之后新实例 2 自己的 2 条根槽（把「回退实例」操作化成刚做完 R(j) 建立起来的新实例，按第四节已有用例「回退实例 3 的四条根……四条都读不出时恢复退到被抛弃的根 (2, 8)」的形态；⚠️ **这个操作化没有走 5.6，交回报告点名，请主 agent 核对与 C332（回退实例两个根都读不出时回退被撤销） 攻方腿原文是不是同一个意思**）：检查 (a) 带故障的冷启动 `recover`（走 `PoolReaderWithUnreadableRootRingSlots`）与 (b) 带故障的 `mount_writable`（走 `FaultInjectingBlockDevice` + `SharedFaultPlan`）都选中 (1,6)，与期望「读得出的被抛弃根里 (txg,实例) 最大的那一条」一致，两处都 `pass`。
8. **小环取法的代理探针**：8.2 原文的校准点是「H2 最长模板」（n1=6、mount_writable、n2=2）整段跑得通；H2 不在这一段实现，用 `small_ring_probe` 复刻同样的操作序列（不判是哪一类错，只判有没有错）代替，48 KiB 第一次就通过。⚠️ 这个简化交回报告点名，主 agent 核这个代理够不够格。
9. **变异**：3 条，全部手工验证红→绿→复原绿（本地常量每区槽数 8→9、系统配置槽余量算术 4096-481→4096-480、活回退事件计数只数最近一次）。三行在 `crates/mutations.tsv` 末尾（「E158（择根与修复四岔路） root_choice_repair」开头的三行），归门禁 crates-mutation-replay 跑；门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次。
10. 命名纪律：`naming-lint.sh` 对新文件全部通过（0 处违规；改了 `n1`→`overwrites_before_sigma`、`G0`（主几何点）/`G_S16`/`G_S4`→`GEOMETRY_PRIMARY`/`GEOMETRY_LARGER_ROOT_RING`/`GEOMETRY_SMALLER_ROOT_RING` 等约 15 处单字母/字母加数字的标识符）。

```

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:545-1253`（整段抄，未转述）**

````markdown
### 2026-09-24：第一段重跑（产物结构性对不上，原因已诊断）

第一段自 2026-09-23 之后没有重跑过；这一次重跑 `bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路），走的是
`driver_e158`（参数 `-- all`），与入库产物 `e158-root-choice-repair-2026-09-23-segment1.out` 比对：
**对不上，50 行不同**。根因是
`crates/singlefs-core/src/mount.rs` 在此期间落地了 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 定案，
拿掉了 `MountError::RollbackToVersionWithoutFileUnsupported` 与 `InstanceRowsOnVersionWithoutFileUnsupported`
两个拒绝分支（`format_time_allocator` 文档注释里 2026-09-23 追加的一段原话：「这一道今天的依据……它**不再**拦
任何一条正当历史」），H3 的 `R(j)` 分支从此能真的回退到树表 0 条的根（如 `(0,0)`），穷举树因此变大：

| 差异 | 旧（2026-09-23） | 新 | 原因 |
|---|---|---|---|
| `h3_family_summary` 与同一段里另一行结果（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 里打 `N1_DONE` 的那一行，GEOMETRY_PRIMARY/S16/小环） | histories=856，peak_roots_live=7 | histories=1822，peak_roots_live=10 | 回退到无文件版本的根现在能走到，树变大，派生量（`leaf_count`/`error_count` 等）跟着变 |
| `h3_family_summary`（S4） | histories=7264 | histories=14252 | 与 GEOMETRY_PRIMARY/S16/小环 那一行同一个原因 |
| `h3_family_pruned_branch_category` | `RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion`（`crates::MountError` 的一个已删除变体） | 「现行版本没有文件」（**装置自己** `apply_overwrite` 的守卫，不是 `crates/` 的拒绝） | 拒绝从 `crates/` 移走后，装置在「回退到无文件版本之后再做一次覆盖写」时自己的守卫接手报错，错误分类逻辑按字符串首词抓取字面对上了 |
| `pc3` 的 `path` | `RollbackTo(1, 3)` | `RollbackTo(0, 0)` | PC3 取「按固定次序第一个满足 events_live≥1 的历史」，`(0,0)` 现在合法且排在更前；**核心断言不变**：`expected_root=(1,6)`，`recover`/`mount_writable` 两处仍 `verdict=pass` |

**S1 现查**（`choose_root` 比较键、`first_txg_of_new_instance` 公式、`next_counter`、`isolate_slots_referenced_only_by_abandoned_roots`
只计数不拒绝）全部与第三节写的相符，S1 不触发；今天新增的 `tree_identifier_watermark_of_the_ring` 管的是
D8（核心索引结构） 已定项 8 ② 的树 ID 水位，不是 C331（择根倒挂压过已确认的写） 的择新逻辑，不影响「今天=丙=甲-jsn」这句话。

**第一段承重的量不受影响**：Q3-2 的 N_w=4、witness 宽度 margin（四种宽度的 fits/past_512/past_512_together 判定）、
PC3 的核心断言在新产物里逐字节相同（`path` 字段除外）。**岔路单第 3 行候选 2 那一半、第 2 行 ③ 在今天的代码上
依然成立，不需要重跑。** 新产物另存 `e158-root-choice-repair-2026-09-24-segment1-rerun.out`，
旧产物原样留着，**两份都点名，承重哪一份交主 agent 定**（我的判断是新产物结构性正确反映了今天的 `crates/`，
但按定义不许自己下这个结论）。

**第二段 Q3-1（岔路单第 3 行候选 3 那一半）**

新增内容全部在 `e158_root_choice_repair.rs` 这一份 bin 里（`crates/singlefs-core`、`crates/singlefs-checker`
一行未动）：`SimNode` 加一个只增不删的 `content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>>`
（P332（违例判据） 的 V2 唯一需要的新状态，记录每一条根曾经对应的「内容号」），P332（违例判据） 判定（V1=所选根落在被抛弃集合里，
V2=读回的内容号不属于当前时间线上任何一个根）在 (a) 带故障冷启动 recover、(b) 带故障 `mount_writable` 之后
再带同一故障冷启动、(c) 撤故障之后冷启动三处各判一次，`|F|≤2` 的 Φ3 全枚举（含空集），5.1「H3 的省法」
（(a) 所选根与 F=∅ 相同就跳过 (b)(c)）按跑前登记原样实现。新增 7 条单测（12 条全绿，逐条改坏证明会红——
`content_is_off_the_current_timeline` 取反、`is_abandoned` 换成 `.all()`、`combinations_of_size` 的 0 判成 1，
三处改坏后按预期各自只让对应测试红，还原后复跑全绿）；3 条变异追加进 `crates/mutations.tsv` 末尾（只追加，
`git diff` 核过没碰别人的行），门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次。`naming-lint.sh` 全部通过（约 45 处重命名：
`v1`/`v2`→`chosen_root_abandoned`/`content_off_timeline`，检查点 `a`/`b`/`c`→
`cold_recover_with_fault`/`mount_then_recover_with_fault`/`recover_after_fault_removed` 等）。

**开跑之前先做了代价标定**：G0（主几何点） 满深度第一次探测在 100 秒里一行输出都没有；标定发现是 (a) 每个故障集合都
重建一次只读镜像，改成整节点建一次、56 个故障集合共用（不改判定逻辑，只读镜像可以安全复用）之后，
G0（主几何点） 全量从预估「层一单独跑不完」降到 real 2m58s。**这处优化是产物之前发现并修的**，按
`evidence-discipline.md`「产物之前修订」记进跑前登记第十二节，原判据、原臂原样保留。

**结果**（G0（主几何点）/S16/小环三点逐字节相同，整行抄自产物（产物已移出工作区的行写成一行指针））：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-g0.out 第 57 行（name=fault_set_violation_summary）
E7RESULT name=fault_set_violation_by_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) weight=0 pairs=1822 violations=0
E7RESULT name=fault_set_violation_by_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) weight=1 pairs=23752 violations=0
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-g0.out 第 60 行（name=fault_set_violation_by_weight）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-g0.out 第 61 行（name=fault_set_violation_by_rollback_presence）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-g0.out 第 62 行（name=fault_set_violation_by_checkpoint_and_predicate）
```

**读法**：N_pairs=170719（1822 个 H3 节点 × 每节点 `|F|∈{0,1,2}` 全部子集）；N_viol=213，比例约 **0.1248%**。
**违例只出现在 `|F|=2` 且历史含至少一次回退**（`|F|∈{0,1}` 与「不含回退」三类各 0 违例）——与 C332（回退实例两个根都读不出时回退被撤销） 正文
「2 个故障」的门槛方向一致（未触发 F2：最小 `|F|` 不是 1、3、4，也不是「≤4 都找不到」）。213 个违例对里
(a) 全部 213 个都 V1，其中 175 个同时 V1+V2；(b)(c) 的 V1 都是 0（`mount_writable` 会把「选中被抛弃的根」
自我修正）但 V2 都是 175（读回的内容仍对不上当前时间线任何一个根，(a)(b)(c) 三处 V2 判定完全一致）。
**这 175 对具体是什么形态、是不是都落在「回退到无文件版本」这个新出现的分支上，这一段没有逐条拆开看**，
是没做完的部分（第「它答不了的」一节点名）。

**第八节敏感性（行 3）**：G_S16、G_小环两点与 G0（主几何点） 逐字节相同；G_S4（`sigma_length_limit=4`，比其余三点
多穷举一层，nodes_visited=14252，7.8 倍于 G0（主几何点）的 nodes_visited）判定方向不翻面——比例、breakdown 整行抄自产物（产物已移出工作区的行写成一行指针）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-s4.out 第 57 行（name=fault_set_violation_summary）
E7RESULT name=fault_set_violation_by_weight geometry=S4(S=4,ring=3MiB) weight=0 pairs=14252 violations=0
E7RESULT name=fault_set_violation_by_weight geometry=S4(S=4,ring=3MiB) weight=1 pairs=170243 violations=0
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-s4.out 第 60 行（name=fault_set_violation_by_weight）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-s4.out 第 61 行（name=fault_set_violation_by_rollback_presence）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-s4.out 第 62 行（name=fault_set_violation_by_checkpoint_and_predicate）
```

比例 1355/1116897 ≈ 0.1213%（G0（主几何点） 是 0.1248%，同一数量级），**同样只在 `|F|=2` 且含回退时出现**，
(a)(b)(c) 三处的 V1/V2 分布形状与 G0（主几何点） 逐比例相同（(a) 全 V1、其中约 86% 同时 V1+V2；(b)(c) 的 V1 全 0、
V2 与 (a) 的 V2 计数相同）——**四个方向相反的取样点里三个（S16、小环、S4）都实测过，判定不翻面**；
第四个（G_默认，768 MiB 环）因 `scan_journal` 按环全部记录槽逐一读、字节数是 3 MiB 环的 256 倍，
按 G0（主几何点） 的 170719 对、real 2m58s 线性外推，**这一段剩下的时间连一次全量都跑不完**，只给出机制论证
（判定只依赖「点名的槽今天存的是哪条根、它的内容号」，不依赖环总槽数——这句话已经在 S=8→S=16、
3 MiB→49152 字节两个方向上被验证），**没有独立产物**，交主 agent 定要不要另开一段跑一个只重放
已知 1355+213 个违例对应 (节点,故障集合) 的缩小版验证。

**（续，session s3）：岔路单第 1 行的 A1 副本建出来了，H1 装置本身没写**

这一段主要做了「另外两件」（`driver_e158` 改指今日承重产物、5 个驱动在主工作区新一批未提交改动之后复核字节一致，
已写进上面「结果整行抄自产物」与「复跑」两节）；在此之外，为岔路单第 1 行（C393（被抛弃根的账读不出时修复没有条款））建了候选 (a) 的 A1 副本：

- 草稿目录 `/tmp/claude-1000/e158-s3/arms/a1/`（只含 `Cargo.toml`/`Cargo.lock`/`.cargo`/`crates`，不含
  `target`），`SNAPSHOT.txt` 记快照时刻、`git rev-parse HEAD`、`git status --porcelain crates/` 行数（92，
  确认 `crates/` 此刻仍在被另一条线 `impl-m2-mountfix` 改动）、`crates/*/src/**/*.rs` 的 sha256 汇总
  （S4 停机条款要求的记录）。
- `crates/singlefs-core/src/mount.rs`：新增 `MountError::AbandonedRootLedgerUnreadable { count: u64 }`，
  在 `mount_writable`/`mount_rollback` 的 `RebuiltAllocator` 解构之后、`establish_instance`（取号，第一次写）
  之前，`abandoned_roots_unreadable > 0` 就提前返回这个错误——按跑前登记 5.2 (a) 逐字实现。
- 收尾两处穷尽 `match` 要跟着补一支才编得过：`crates/singlefs-verification/src/history.rs` 的
  `mount_error_member`、`crates/singlefs-verification/src/model_comparison.rs` 的 `refusal_reason_of_mount_error`
  与 `reported_ceiling_of_mount_error`（新变体归「模型没有理由」的默认档，不改判定逻辑）。
- 四处改动记进 `scripts/mutation/tables/e158_arms.tsv`（新建，四列同 `crates/mutations.tsv`：说明、文件、旧串、
  新串，逐条用 Edit 工具应用、Edit 本身保证旧串在改动那一刻的文件里恰好命中一次）。
- 编译核过：`cargo build --release -p singlefs-verification --bin e158_root_choice_repair`（在 A1 副本目录下）
  `Finished` 无错误无告警。
- **侦察结论**：`crates/singlefs-core::recovery` 里 `tree_table_has_no_entries`、`allocation_records_under_root`
  （都是 `pub fn`）与 `RootRecord.tree_table: NodePointer`、`NodePointer.locations: [LocationEntry; 2]`、
  `LocationEntry{device, slot, ..}`、`SlotNumber::to_device_offset()`（都是 `pub`）合起来就是 `mount.rs` 里
  `isolate_slots_referenced_only_by_abandoned_roots` 判定 `abandoned_roots_unreadable` 时走的同一条路径——
  这条路径**恰好对应跑前登记 5.2 的两个指称**：`tree_table_has_no_entries` 返回 `Err` ⇒ 树表指称的读失败；
  返回 `Ok(false)` 之后 `allocation_records_under_root` 内部再失败 ⇒ 分配记录树指称的读失败（`Ok(true)`，
  即树表 0 条那一版，走 `root.allocation_record_tree_root.locations` 直接判）。

**（续，session s3 第二段）：H1 家族 + Φ1 故障注入 + PC1-a/PC1-b 写出来了，Q1-1 至 Q1-4 有真数了**

写了 H1 全族枚举（`h1_family`：n1∈{1,2,3}、j 取每一条可读根、m∈{0,1,2}，回退到无文件版本之后 `apply_overwrite`
按今天的守卫拒绝、m 停在那一步）、Φ1 故障注入（`attempt_op1`：按 (设备, 偏移) 精确注入，落点由
`locate_allocation_record_tree` 现查，5.2 (a)/(c) 用同一个函数、跨臂只用 `error_member_of_debug` 的字符串
首词识别 A1 副本才有的 `AbandonedRootLedgerUnreadable`，不直接 `match` 枚举名——这份装置源码两边共用一份，
`match` 会让编不过「今天」那份没有这个变体的 `crates/`）、PC1-a/PC1-b（`run_pc1_a_and_b`：按固定次序找第一条
「分配记录树节点不共享」的历史）。**修了一个真 bug**（产物之前发现，见跑前登记「十二、修订」）：最初
`root_record_of` 拿 `RootView::record_bytes`（截到 457 字节）喂给 `RootRecord::parse_slot`，校验和按整槽宽
（512）算的、截短之后对不上，`parse_slot` 恒 `None`——H1 全族「被抛弃可读根」一个都取不到；改成按
`checker_image::root_slot_positions` 自己重新按整槽宽读一遍。新增 4 条单测（16 条全绿，`fault_targets_for`/
`error_member_of_debug` 两条逐条改坏验证会红、还原后 `diff` 逐字节一致），3 条变异（`crates/mutations.tsv`
追加），命名与格式纪律（`naming-lint.sh`、`cargo fmt`、`cargo clippy -D warnings` 加共用约束那批 lint）全过。

**候选 (c)=今天** G0（主几何点） 上跑出的数（产物 `e158-root-choice-repair-2026-09-24-q1-g0-today.out`，
两次独立跑逐字节相同）：

```text
E7RESULT name=q1_h1_family_size geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) nodes=36 rejected_rollback_attempts=0 unresolved_units=0
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-today.out 第 56 行（name=q1_1a_trigger_summary）
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=both_copies pairs=657 trigger=399
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk0_only pairs=657 trigger=0
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk1_only pairs=657 trigger=0
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=both_copies pairs=657 trigger=399
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk0_only pairs=657 trigger=0
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk1_only pairs=657 trigger=0
E7RESULT name=q1_1c_error_member geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) member=Recovery count=42
E7RESULT name=q1_1c_error_member geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) member=RollbackTargetNotACandidate count=1296
E7RESULT name=q1_2_allocation_record_tree geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=can_recompute count=141
E7RESULT name=q1_2_allocation_record_tree geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=cannot_recompute count=57
E7RESULT name=q1_2_allocation_released_records geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) zero=27 nonzero=114
E7RESULT name=q1_2_tree_table_copy_elsewhere geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) has_copy=9 no_copy=57
E7RESULT name=q1_3_persistence_cost_values geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) values={32768}
```

**候选 (a)=A1 副本**（产物 `e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out`，
**副本上的数**，只在 `/tmp/claude-1000/e158-s3/arms/a1/` 编出来、`replay.sh` 不覆盖它，见「复跑」）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out 第 48 行（name=q1_1a_trigger_summary）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out 第 55 行（name=q1_1c_error_member）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out 第 63 行（name=q1_4_a_rejected_device_bytes）
```

**读法**：

- **Q1-1a（N_trig）**：`abandoned_roots_unreadable > 0` 只在「两份都读失败」（`both_copies`，2 个故障）时
  触发（399+399=798 对），只坏一份（`disk0_only`/`disk1_only`，1 个故障）**从不**触发——RAID 镜像的另一份
  顶住了；`empty_baseline`（不注入任何故障，36 个节点各跑一次）**0 次**自然触发，H1 在 n1≤3、m≤2 的规模下
  没有绕环巧合。**N_trig=798>0**：行 1 翻面观测前半成立的方向（(a) 与 (c) 的被拒比例不同）在这批数据上是
  真的——今天那一臂全部 798 个 (Ok, count>0) 对，A1 副本在**同样的 798 对**上全部改判
  `AbandonedRootLedgerUnreadable`（error_member 计数 798 与 (c) 的 trigger_count 798 逐一对应），
  其余 3144 对两臂结局相同（Ok 或同一种候选集拒绝）。
- **Q1-1b（三种被拒比例）**：(c)=今天 N_err=0（这批构造出的 3942 对里没有一个真的入口报错到「今天算法本身
  的缺口」——`Recovery`/`RollbackTargetNotACandidate` 都是候选集本身排除、不是账读不出）；被拒比例
  = 798/3942 ≈ **20.2%**。(a)=A1：被拒比例同样 798/3942（这批数据下 (a) 与 (c) 的「触发即拒绝」严格对应，
  没有观测到 (a) 拒绝了 (c) 没触发的对，也没有 (c) 触发了 (a) 没拒绝的对）。(b) 分配记录树那一档见 Q1-2。
- **Q1-1c（今天报错数分类）**：`Recovery`（42）、`RollbackTargetNotACandidate`（1296）——**F1 不触发**：
  798 个「被抛弃根的单元两份都读失败」的对里，(c) 一次都没有报 `Err`，全部 `Ok(count>0)`，与 C393（被抛弃根的账读不出时修复没有条款） 正文
  「今天只计数、不拒绝挂载」逐条相符（这批数据坐实这句话，不是推断）。
- **Q1-4（(a) 被拒那次写几个字节）**：**798/798 = 100%** 的 `AbandonedRootLedgerUnreadable` 拒绝，设备一层
  写字节都是 **0**（`zero_byte_rejections=798 nonzero_byte_rejections=0`）——「拒绝在任何写之前」这句在
  A1 副本上全量成立，不只是 PC1-a 那一个例子。
- **Q1-2（候选 (b) 分配记录树指称）**：`can_recompute=141`、`cannot_recompute=57`（只统计 op1=mount_writable
  那一档，与 Q1-2 定义一致）——**这些成功大多来自单盘故障**（`disk0_only`/`disk1_only`：走树能用另一份镜像，
  与 Q1-1a 的「1 个故障从不触发」是同一个机制的另一面）；已释放记录条数 `zero=27 nonzero=114`——大多数
  被测的分配记录树节点里混着已释放记录，**只有 27/168 能「按字节原样写回」**（有已释放记录的那 114 个，
  按字节原样重写会把已释放记录也当现行数据写回，候选 (b) 在这些情形下的「就地重建」定义本身要处理已释放
  记录，不只是原样抄一份）。
- **Q1-2（候选 (b) 树表指称）**：`has_copy=9 no_copy=57`——多数被抛弃根的树表节点在孪生镜像别处**没有**
  字节相同的副本（COW 每次发布都重写树表，节点内容随之变化），「有副本」只在少数情形（约 14%）成立；
  「crates 有路」（调 `rebuild_version`/`replay_journal`）**这一段没有测**，`no_copy` 不能读成「无来源」，
  只能读成「只测过的这一支没有」。
- **Q1-3（(b) 的持久化开销）**：两个指称在这批数据里量出的取值集合都是 **{32768}**（一个 16384 字节节点
  × 2 份镜像）——**跑前登记「两个指称的取值集合不同 ⇒ 成立」这一格翻面条件在这个粒度的测法下不成立**：
  树表节点与分配记录树节点在这批被抛弃根上都恰好是单节点（崩溃点重放），开销算式给出同一个数，不是两个指称的
  持久化开销真的相等——粒度更细（比如分配记录树多层、树表跨页）时会不同，这批 H1 历史都没有走到那种规模。
- **PC1-a**：`overwrites_before_rollback=1 rollback_target=(0,0) overwrites_after_rollback=0 abandoned_root=(1,3)`，
  `roots_through_unit=1`。(c)=今天 `Ok(count=1)`；
  (a)=A1 `AbandonedRootLedgerUnreadable{count:1}`、`device_write_bytes=0`——**两条断言都通过**。
  (b) 分配记录树：`outcome=cannot_recompute`——**跑前登记 PC1-a 原文「(b) 分配记录树：装置走树必须成功」
  在这个操作化下没有复现**：候选 (b) 的「就地重建」在这一段被操作化成「用同一组注入目标重新调
  `allocation_records_under_root`」，而这个函数内部本来就要重读**同一个**被注入的节点——两份都读失败时
  它当然也读不出，逻辑上不可能在这个操作化下成功。**这不是这一段算错了 Q1-2 的数**（Q1-2 的
  `can_recompute`/`cannot_recompute` 分类如实反映了这个操作化的行为），**是候选 (b) 支持者原意很可能指
  另一种算法**——从树表出发走它引用的每一棵内容树、按单元指针反推占用集合，完全不重读被注入的那个节点
  本身（跑前登记 5.2 候选 (b) 原文「把它能走到的每棵树的每个单元指针走一遍」）——那种算法要新写一整套
  内容树遍历（inode / 中央映射树 / extent 指针），这一段没有实现。按 `evidence-discipline.md`
  「阳性对照没有过 ⇒ 作废」（跑前登记第十一节 V1），**PC1-a 只按 (a)(c) 两条通过计**，(b) 那一句的
  操作化对不对交主 agent 判断。
- **PC1-b**：同一段历史，改打树表指称：(c) `Ok(count=1)`、(a) `AbandonedRootLedgerUnreadable`——与 PC1-a
  一致；候选 (b) 分配记录树在树表被注入时会不会失败（附带检查）：`walk_fails=true`（这个被抛弃根的树表
  非 0 条，分配记录树指针本身住在树表节点里，树表读不出连指针都拿不到，分配记录树自然也走不通——这是
  跑前登记原文没有单独问的一个附带观测，供参考）；候选 (b) 树表「有副本」：`has_copy_elsewhere=false`。

**（续，session s4）：岔路单第 2 行 ①②——H2 主族穷举下界 + Q2-2a 有真数了**

写了 Q2-1 的新装置代码（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 新增约 550 行）：
Φ2 证据项枚举（`evidence_items_at`：根环里每个存着自证根的槽，权重 1；journal 环里每一条至少一份
自证的记录，权重 = 自证的份数）、权重从小到大的穷举下界搜索（`candidate_fault_sets_of_weight` +
`search_minimum_weight_that_triggers_rootback`，根槽权重 1、记录权重 2，逐权重把两边组合叉乘）、
H2 主族（`run_rootback_tolerance_family`：mkfs → 可写挂载 → 首个文件 → n1 次覆盖写 → 关闭 →
op2=`mount_writable`（受注入）→ 成功则 n2 次覆盖写 → 关闭 → 判 P331（打中判据） ）、P331（打中判据） 两处判定
（`judge_recovery_outcome` + `evaluate_rootback_predicate`：冷启动 recover 与再做一次不注入的
`mount_writable` 之后冷启动 recover）。另写了 Q2-2a（`run_fixed_publication_byte_script`）：跑前登记
5.8 表的固定脚本（mkfs → 可写挂载 → 首个文件 → 3 次覆盖写 → 关闭 → 可写挂载 → 2 次覆盖写 → 关闭 →
回退到首个文件那条根 → 1 次覆盖写），逐次发布读 `TransactionOutput.writes`/
`VersionWithoutFilePublishOutput.writes`（`WritesByStructureKind`）按结构种类报字节。

**过程中发现并修了一个真 bug（产物之前）**：P331（打中判据） 检查②最初直接套用 `judge_recovery_outcome` 判读回的
所选根「在不在时间线上」，而检查②本身先做了一次**不注入**的 `mount_writable`（推进了状态、新实例的
写行/暖机已经落盘），它的读回所选根天然是这个新实例的根，不在装置只记到 op2 探针为止的
`node.timeline` 里——套用「倒挂」半句会把这次正常前进的 mount_writable 误判成打中，导致 G0（主几何点） 上任何一个
模板在权重 0（零故障）时就报「打中」。改成检查②只比内容号，不再重套「倒挂」判定（跑前登记 5.0 原文
「而它之后冷启动读回的不是当前末端的内容号，记读不回」本来就只问内容，不问根的时间线归属）。这不改变
任何臂或判据——判的还是 P331（打中判据） 定义的那两处，只是修好了检查②本身的实现。

候选 (c)=今天=丙=甲-jsn 与候选甲-txg 两臂都跑了同一份搜索（Φ2 只含根槽 + 记录两类，不含系统配置槽：
这两条臂的 `first_txg_of_new_instance` 都不读系统配置，跳过这一类证据不改变这两条臂的搜索结果）：

```text
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=0 overwrites_after_the_rootback_probe=1 k_min=none_found_within_full_evidence_space maximum_weight_available=10 subsets_tried=128 rootback_probe_failures=0 overwrite_failures=0
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=1 overwrites_after_the_rootback_probe=1 k_min=none_found_within_full_evidence_space maximum_weight_available=13 subsets_tried=512 rootback_probe_failures=0 overwrite_failures=0
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=2 overwrites_after_the_rootback_probe=1 k_min=none_found_within_full_evidence_space maximum_weight_available=16 subsets_tried=2048 rootback_probe_failures=0 overwrite_failures=0
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=3 overwrites_after_the_rootback_probe=1 k_min=none_found_within_full_evidence_space maximum_weight_available=19 subsets_tried=8192 rootback_probe_failures=0 overwrite_failures=0
```

（`research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-today.out`；甲-txg 臂的同一份产物
`e158-root-choice-repair-2026-09-24-q2-1-g0-jia-txg-arm.out` 与它 `diff` 逐字节相同。）

**读法**：n1∈{0,1,2,3}、n2=1、op2=`mount_writable` 这个缩小范围内，**穷举已经走到了这个模板 Φ2
的全部权重**（`subsets_tried` 精确等于该权重下枚举出的全部子集数，没有被 `SUBSET_ENUMERATION_CAP`
截断），**一次都没有打中**——哪怕把根槽 + 记录能藏的全部藏起来（n1=3 时权重上限 19），今天那一臂与
甲-txg 都不会把新实例的写行发布落在一个会被之后择新覆盖的旧位置上。**这与判决 K2 表「今天推的是……丙
12 个」的方向不一致**：19 ≥ 12，而权重 19 已经穷尽了 n1=3 这个模板的全部证据空间还是没打中，说明「丙
12」这个数（或它所在的具体构造）在**这个缩小的评测范围**里复现不出来——按跑前登记 F4「岔路单里推的数
……与跑出来的不同」的触发观测（「今天那一臂（=丙）在崩溃 0 档的区间不含 12」），这一格判据**触发**了，
但按 F4 的处置步骤，触发之后要先让主 agent 对照攻方腿原文核实这里的 n1≤3、op2=`mount_writable`、
Φ2 缺系统配置槽是不是与判决 K2 推「12」时用的同一个构造——**不是同一个构造几乎可以肯定**（K2 的原始
推导细节这一段没有重新读一遍），所以更准确的说法是「这一段缩小范围的搜索没有复现『12』，不能判定
『12』本身错了」，交主 agent 核对判决 K2 原文之后再定是记「推的数不成立」还是「臂不同，推的数无可比」。
两条臂在这个缩小范围内**逐字节相同**：甲-txg（只看根环 txg）与丙（根环 txg 与记录 txg 取 max）在
n1≤3 时从未出现「记录读不出让择新结果不同」的一格——也符合直觉：记录的 txg 不会超过它所属发布的根的
txg，n1 小、根环没有绕环巧合时，两种口径能看到的「最大 txg」应当一致，需要绕环或记录被择新单独依赖
的那种情形（n1 更大、或跨过环容量）才会分化，这一段的 n1≤3 还没有走到那种规模。

**Q2-2a**（固定脚本，`e158-root-choice-repair-2026-09-24-q2-2a-g0-today.out`，
124 行）：`TransactionOutput.writes`/`VersionWithoutFilePublishOutput.writes` 的口径已经是「一次写
调用 = 交给一块盘的一次写，镜像的两份各算一次」（`write_accounting.rs` 顶部注释），与跑前登记「每块盘
收到的写的长度之和」是同一个数，不必另建按 (设备, 偏移) 分类的字节计数器。逐次发布（写行、暖机、
覆盖写、回退写行）按 12 种 `WrittenStructureKind` 分别报字节；甲-txg 臂的同一份产物
（`e158-root-choice-repair-2026-09-24-q2-2a-g0-jia-txg-arm.out`）与今天逐字节
**完全相同**——甲-txg 只改变 `first_txg_of_new_instance` 选中**哪个 txg 值**，不改变任何一次发布要
写哪些结构、写多少字节，这个结果与该臂定义本身直接推得出，是同义反复式的确认（跑前登记 Q2-2b 那一格
「只报数、不作判据」的理由同理适用于这里）：**Q2-2a 在甲-txg 与丙之间的 Δ = 0**（每一种结构、每一种
发布种类都是 0），不支持「行 2 翻面观测后半（每次发布开销不同）」在这一对臂上成立。

**验证**：新增 6 条单测（`candidate_fault_sets_of_weight_mixes_root_slots_and_journal_records_by_weight`、
`candidate_fault_sets_of_weight_at_zero_is_only_the_empty_set`、`judge_recovery_outcome_flags_a_root_off_the_timeline_as_rootback_before_checking_content`、
`judge_recovery_outcome_flags_wrong_content_as_unreadable`、
`judge_recovery_outcome_passes_when_root_on_timeline_and_content_matches`、
`judge_recovery_outcome_passes_on_no_file_when_no_file_is_expected`，22 条全绿）；3 条变异
（`crates/mutations.tsv` 累计 12 条 E158（择根与修复四岔路） 行，逐条手工改坏验证会红、还原后 `diff` 逐字节一致、
门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次）；`cargo fmt`/`clippy -D warnings`/`naming-lint.sh` 对这份 bin
清零。甲-txg 臂：草稿目录 `/tmp/claude-1000/e158-s4/arms/jia-txg/`（`cp -a` 整棵工作树的
`Cargo.toml`/`Cargo.lock`/`.cargo`/`crates`，`SNAPSHOT.txt` 记快照时刻、`git rev-parse HEAD`、
`git status --porcelain crates/` 行数、`crates/*/src/**/*.rs` 的 sha256 汇总），改动记进
`scripts/mutation/tables/e158_arms.tsv`（只改 `crates/singlefs-core/src/mount.rs` 的
`first_txg_of_new_instance` 一处：只取根环 txg，`records` 参数改名 `_records` 保留不读）。

**（续，session s5）：PC2 阳性对照——查出装置故障装配 bug，修好之后甲-txg 的 k_min 与判决 K2 一致**

**派发要求「先做完才往下」的一件**：复现判决 K2（`m2-rootchoice-repair-r1-main-verification.md` 第
47 行）已经判过的两个具体构造——甲「计数器从全环最大 + 1 起」4 个瞬时根槽读失败打中；乙「系统配置带
txg」0 个注入故障、1 个崩溃点打中——不靠穷举搜索，作为「装置看得见判决说存在的打中」的独立验证。

**臂对齐**：K2 表「甲」逐字对应本登记「甲-txg」（`highest_root_txg`，D16（发布语义） 已定项 6 读法），**不是**
5.6 项 2 登记的「甲-jsn」（今天的 `next_counter`）——现查 `mount.rs`：`next_counter` 只喂给下一条
journal 记录的 jsn 分配，`first_txg_of_new_instance`（选根 txg 的唯一入口）从未读它，两个量完全不
相交。详细依据见跑前登记「十二、修订」session s5 条目第 1 条。

**第一次直接测「甲-txg，4 个瞬时根槽读失败」，hit=false**——查出根因：`FaultInjectingBlockDevice` 一
块盘只嵌套一层、一个 `SharedFaultPlan` 一次只装一条计划，`attempt_rootback_probe_and_advance`
（H2 全家族的穷举搜索都靠它装故障）原来用 `.find()` 只挑每块盘上第一个匹配的故障目标去装，只有 2 块
盘时「至少两个目标落在同一块盘」按鸽笼原理在权重 ≥ 2 就可能撞上——**这条 bug 影响 H2 全家族至今用
这套机制跑出的每一个「0 命中」**，包括 session s3/s4 入库的 `q2-1-g0-today.out` 与
`q2-1-g0-jia-txg-arm.out`。修法：新写 `MultiOffsetReadFailingBlockDevice`（本地包装，一次记住一块盘
上全部该装的精确偏移，不改 `fault_injection.rs` 共用基础设施），`attempt_rootback_probe_and_advance`
与新写的 `attempt_a_faulted_mount_writable` 都改用它。

**修好之后，不需要新机制，既有的穷举搜索直接找到 k_min=4**（与专用构造 `attempt_a_faulted_mount_
writable_and_append` 互证；这条专用构造最初是因为怀疑 `effective_root` 截断本身测不出 H-A 才写的，修好根因
一之后证明这层担心不成立——`effective_root` 的位置本来就在截断前的时间线末尾，截断是空操作，见跑前
登记「十二、修订」session s5 条目第 3 条）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-pc2-root-ring-txg-only.out 第 46 行（name=q2_1_pc2_root_ring_txg_only）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-root-ring-txg-only-session-s5.out 第 49 行（name=q2_1_minimum_weight）
```
（`e158-root-choice-repair-2026-09-24-pc2-root-ring-txg-only.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-root-ring-txg-only-session-s5.out`）

**乙-只配置候选新建了副本、复现 K2「0 个故障、1 个崩溃点」**：新增系统配置字段 `published_txg`
（481→489），`first_txg_of_new_instance` 只信这个字段（`crates/` 现存字段表比对因此要按臂现算，
`local_system_configuration_bytes()` 读环境变量、没设时取 481）。崩溃点用 `acquire_instance`（公开
函数，取号本身只做「写系统配置槽 + 一道屏障」两步，中途无其他写）单独调用后立即收工来还原，不需要
崩溃点重放的崩溃点枚举器：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-pc2-configuration-published-txg.out 第 47 行（name=q2_1_pc2_configuration_published_txg）
```
（`e158-root-choice-repair-2026-09-24-pc2-configuration-published-txg.out`，跑在
副本上，环境变量 `SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES=489`）

四组负对照（今天/甲-txg 副本上乙-候选 hit=false；今天副本上甲-候选 hit=false）与两组正对照
（`q2_1_pc2_root_ring_txg_only` hit=true、`q2_1_pc2_configuration_published_txg` hit=true，两条
`E7RESULT` 原样已引在这一节前两段），逐条落在三份产物里：
`research/results/e158-root-choice-repair-2026-09-24-pc2-today.out`（今天，两条候选都
`hit=false`）、`e158-root-choice-repair-2026-09-24-pc2-root-ring-txg-only.out`、
`e158-root-choice-repair-2026-09-24-pc2-configuration-published-txg.out`。

**H2 主族按登记 5.1 补齐 n1∈{0,...,6}**（此前只到 3）：今天/丙在整个范围内仍是穷举完整个可枚举证据
空间都没打中（n1≤3）或撞上 `SUBSET_ENUMERATION_CAP=20000` 提前撞顶（n1∈{4,5,6}，`k_min=capped`）；
**这不构成「丙挡得住」的确认**——判决 K2 给丙的具体历史 H-C1 需要 12 个特定故障（4 根槽 + 4 条记录
各在两块盘上都读不出），这一段没有像甲、乙那样为它单独写一个直接构造，`capped` 只说明穷举没走到那
么远。甲-txg 在 n1=3/4/5/6 各自给出 `k_min=4/4/5/4`（一次 `k_min=5`），与判决 K2「甲 4 个」一致
（`research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-today-session-s5.out`、
`...-root-ring-txg-only-session-s5.out`）。

**验证**：新增 6 条单测（`parse_local_system_configuration_bytes_*` 三条、
`multi_offset_read_failing_block_device_*` 两条、`root_ring_slot_targets_for_finds_exactly_the_
named_roots`，28 条全绿）；3 条变异（`crates/mutations.tsv` 累计 15 条 E158（择根与修复四岔路） 行，逐条手工改坏验证会
红、还原后 `diff` 逐字节一致、门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次）；`cargo fmt`/`clippy -D
warnings`/`naming-lint.sh` 清零；`replay.sh E158`（择根与修复四岔路） 开工与交回前各跑一次。乙-只配置臂：草稿目录
`/tmp/claude-1000/e158-s5/arms/configuration-published-txg/`（同 s4 的整棵工作树 `cp -a` 手法），
改动记进 `scripts/mutation/tables/e158_arms.tsv`（15 行，跨 `singlefs-format/src/lib.rs`、
`singlefs-core/src/{system_configuration,transaction,make_filesystem,mount}.rs`）。

**没做的**（详见「它答不了的」）：乙-留环/丁-留环/丁-只配置三份副本；H2-R、H2c 两个变体；n2=2；
Φ2 系统配置槽证据类（对已跑的两臂不改变结果）；H-C1（丙）的直接构造；Q2-2b。

**（续，session s6）：H-C1 直接构造坐实丙恰好需要 12 个故障；第一行几何敏感性；第二行 ③ 算式**

**开工先做的四处 clippy（session s4/s5 遗留，不是本段新写的代码）**：`items after a test module`
（`mod tests` 挪到文件末尾，纯粹的代码块重排——写脚本切出前后两段互换顺序，用 sha256 核对「把重排
撤销之后与改前的文件逐字节相同」，核对通过：`7a2a46dde4bceb4745dac5bfc7a7245de96708496cc0f67c
91e2440eb8d88df7`，与 session s5 报告记录的改前哈希一致）；`judge_recovery_outcome`/
`evaluate_rootback_predicate` 两处通配臂改成穷举 `RecoveryOutcome` 三个具名变体；
`run_fixed_publication_byte_script` 两处 `shadow_unrelated` 改名（`mounted`→`rollback_mount_
output`、`allocator`→`rollback_allocator`）。四处改完 `cargo clippy --all-targets --all-features`
加共用约束那批 lint 对这份 bin 清零；22 条既有单测保持全绿。

**主工作区在本段期间被合并进「里程碑二收尾」提交系列**（开工 `HEAD`=`3b60f098e97dc4c4f3ed9c6
355422b607db1c34c`、`git status --porcelain crates/` 95 行；中途变成 `HEAD`=`e980a219f1834c638
cd2fae18b25a60525a6bd52`，`crates/*/src/**/*.rs` 净改动 5139 行新增/1185 行删除）。S1 逐一现查本
登记依赖的每个函数与常量（`first_txg_of_new_instance`、两种挂载的 `next_counter`、`choose_root`
比较键、`isolate_slots_referenced_only_by_abandoned_roots` 只计数不拒绝、`SystemRuntimeQuantities`
三字段、五个格式常量）**全部与第三节写的相符，S1 不触发**——这批改动是别处的新增（如
`crates/singlefs-core/src/admission.rs`），不影响 E158（择根与修复四岔路） 已测的任何观测面；装置在新 `HEAD` 上编译、
既有单测全绿。

**H-C1 直接构造（岔路单第 2 行 ①，丙的具体历史，判决 K2「丙……要 12 个」）**：新写
`run_direct_construction_for_the_record_scan_watermark_candidate`（与甲-txg 同一段历史 n1=4，
精确点名顶上 4 条根槽 + `evidence_items_at` 里计数器最大的 4 条记录、两份镜像的 (设备, 偏移) 全拦，
总权重 4×1+4×2=12）与负对照（只挡 4 条根槽，不挡记录）：

```text
E7RESULT name=q2_1_h_c1_record_scan_watermark geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) candidate=record_scan_watermark overwrites_before_the_rootback_probe=4 overwrites_after_the_rootback_probe=1 topmost_four={(1, 4), (1, 5), (1, 6), (1, 7)} record_labels=["record(instance=1,counter=7)", "record(instance=1,counter=6)", "record(instance=1,counter=5)", "record(instance=1,counter=4)"] root_target_count=4 record_target_count=8 fault_target_count=12 fault_weight=12 expected_by_k2_table_fault_count=12 hit=true expected_by_k2_table_hit=true first_check="cold_recover: 读不回 outcome=FileRead { root: (InstanceGeneration(1), CheckpointTxg(7)), content: [150, 30, 161, 41, 172, 52, 183, 63, 194, 74, 205, 85, 216, 96, 227, 107, 238, 118, 249, 129, 9, 140, 20, 151, 31, 162, 42, 173, 53, 184, 64, 195, 75, 206, 86, 217, 97, 228, 108, 239, 119, 250, 130, 10, 141, 21, 152, 32, 163, 43, 174, 54, 185, 65, 196, 76, 207, 87, 218, 98, 229, 109, 240, 120, 0, 131, 11, 142, 22, 153, 33, 164, 44, 175, 55, 186, 66, 197, 77, 208, 88, 219, 99, 230, 110, 241, 121, 1, 132, 12, 143, 23, 154, 34, 165, 45] } expected=Some([96, 227, 107, 238, 118, 249, 129, 9, 140, 20, 151, 31, 162, 42, 173, 53, 184, 64, 195, 75, 206, 86, 217, 97, 228, 108, 239, 119, 250, 130, 10, 141, 21, 152, 32, 163, 43, 174, 54, 185, 65, 196, 76, 207, 87, 218, 98, 229, 109, 240, 120, 0, 131, 11, 142, 22, 153, 33, 164, 44, 175, 55, 186, 66, 197, 77, 208, 88, 219, 99, 230, 110, 241, 121, 1, 132, 12, 143, 23, 154, 34, 165, 45, 176, 56, 187, 67, 198, 78, 209, 89, 220, 100, 231, 111, 242])" second_check="unfaulted_mount_writable_then_recover: 读不回 outcome=FileRead { root: (InstanceGeneration(3), CheckpointTxg(10)), content: [150, 30, 161, 41, 172, 52, 183, 63, 194, 74, 205, 85, 216, 96, 227, 107, 238, 118, 249, 129, 9, 140, 20, 151, 31, 162, 42, 173, 53, 184, 64, 195, 75, 206, 86, 217, 97, 228, 108, 239, 119, 250, 130, 10, 141, 21, 152, 32, 163, 43, 174, 54, 185, 65, 196, 76, 207, 87, 218, 98, 229, 109, 240, 120, 0, 131, 11, 142, 22, 153, 33, 164, 44, 175, 55, 186, 66, 197, 77, 208, 88, 219, 99, 230, 110, 241, 121, 1, 132, 12, 143, 23, 154, 34, 165, 45] } expected=Some([96, 227, 107, 238, 118, 249, 129, 9, 140, 20, 151, 31, 162, 42, 173, 53, 184, 64, 195, 75, 206, 86, 217, 97, 228, 108, 239, 119, 250, 130, 10, 141, 21, 152, 32, 163, 43, 174, 54, 185, 65, 196, 76, 207, 87, 218, 98, 229, 109, 240, 120, 0, 131, 11, 142, 22, 153, 33, 164, 44, 175, 55, 186, 66, 197, 77, 208, 88, 219, 99, 230, 110, 241, 121, 1, 132, 12, 143, 23, 154, 34, 165, 45, 176, 56, 187, 67, 198, 78, 209, 89, 220, 100, 231, 111, 242])"
E7RESULT name=q2_1_h_c1_negative_control_root_slots_only geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) candidate=record_scan_watermark overwrites_before_the_rootback_probe=4 overwrites_after_the_rootback_probe=1 topmost_four={(1, 4), (1, 5), (1, 6), (1, 7)} fault_target_count=4 expected_by_k2_table_fault_count=4 hit=false expected_hit=false first_check="ok" second_check="ok"
```

（`research/results/e158-root-choice-repair-2026-09-24-q2-1-hc1.out`；session s8 门禁 doc-experiments 的 quoted-result-lines 格发现
上一行曾被摘成 `content=[...]`，现按产物整行补全，不再摘录）**直接构造 `hit=true`（4 条根槽 + 4 条
记录都拦下，权重 12），负对照 `hit=false`（只拦根槽挡不住丙）**——与 K2「丙……要 12 个」一致。

**下界探针坐实「恰好 12」，不只是上界**：新写 `run_record_scan_watermark_lower_bound_probe`，复用
既有的 `search_minimum_weight_that_triggers_rootback`，只把它已有的 `subset_enumeration_cap` 参数
从 `SUBSET_ENUMERATION_CAP`=20000（session s5 在这个 n1 上撞了顶）单独调高到 60000（**session s9
把 `subset_enumeration_cap` 参数改名为 `weight_ceiling`、并把「数够就停」的截断机制换成按权重分级
截断，详见 `.claude/kb/experiments/158-择根与修复四岔路.md`「2026-09-24（续，session s9）」一节；
`q2_1_h_c1_lower_bound_probe` 这一行结果是按新字段名重跑之后的产物，`k_min`/`subsets_tried` 等结
论数字与当年逐字相同，只是多了 `full_space_subset_count`）：

```text
E7RESULT name=q2_1_h_c1_lower_bound_probe geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=4 weight_ceiling=None k_min=12 hit_count_at_that_weight=1 first_hit_combination=["root(region=0,slot=2)", "root(region=1,slot=1)", "root(region=1,slot=2)", "root(region=2,slot=1)", "record(instance=1,counter=4)", "record(instance=1,counter=5)", "record(instance=1,counter=6)", "record(instance=1,counter=7)"] subsets_tried=22558 full_space_subset_count=32768 rootback_probe_failures=29 overwrite_failures=153
```

（`research/results/e158-root-choice-repair-2026-09-24-q2-1-hc1-lower-bound.out`）`subsets_tried=
22558 < 60000`——**没有撞顶**，权重 0 到 11 的全部组合穷举完都没有命中，权重 12 上第一个尝试的组合
（4 根槽 + 4 条记录，与直接构造用的是不同的具体根槽、同样是计数器 4/5/6/7 那 4 条记录）就命中。
按 `.claude/rules/mutation-sampling.md`「构造上界给出的数，穷举在它之下全部权重都没打中才写成
『恰好 k』」，**丙恰好需要 12 个故障，这是穷举下界给出的严格证据，不是撞顶推出的上界**。

⚠️ **判决 K2 原表把丙这一格同时标「攻不动（结构性：每次发布必写记录、记录先于根 ⇒ 水位单调不减）」
与「要 12 个」，这两句字面有张力**：「攻不动」读作「不可能被打中」，「要 12 个」给出一个具体的打中数。这一段两条
独立路径（手写构造、独立穷举）都在权重 12 上给出 `hit=true`——**丙确实可以被打中，且恰好需要 12**，
与后一句吻合、与前一句的字面读法（「不可能」）不吻合。这句话该怎么读（也许「攻不动」原意专指「只靠
压根环这一种简单攻法」）交主 agent 核对攻方腿原文判断，本段不越权下结论。**这也更新了 session s4
的一句旧判断**：session s4 曾报告「今天那一臂（=丙）在崩溃 0 档的区间不含 12」、按 F4 判「触发」
（见「结果整行抄自产物」573 行前后）；那时的搜索范围是 n1≤3（穷尽到 19 都没打中），本段在
n1=4 上用直接构造 + 60000 上限的穷举证明「不含 12」是搜索范围不够大，不是丙真的挡得住——**F4 在丙
这一格上，按这一段的读数应改判「不触发」（与推的数一致）**，具体归类交主 agent 定。

**第八节几何敏感性（岔路单第 1 行判决格 = Q1-1a 的 N_trig）**：新增 `q1-s16`、`q1-s4` 两个模式，
复用既有的 `run_ledger_fault_family`（本就是几何参数化的函数，未改代码）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-s16.out 第 56 行（name=q1_1a_trigger_summary）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-s4.out 第 56 行（name=q1_1a_trigger_summary）
```

（`e158-root-choice-repair-2026-09-24-q1-s16.out`、
`e158-root-choice-repair-2026-09-24-q1-s4.out`；两点与 G0（主几何点）
的 `pairs=3942 trigger_count=798` 逐字节相同，`q1_1a_by_aspect_severity`/`q1_1c_error_member`
各分项计数也一致）**判定不翻面**。判别力自证：两点同值，门槛无处可挪，按 8.2 原文记「两点同值，
自证不适用」。

**岔路单第 2 行 ③ 的字段表宽度算式（甲/丙、乙/丁两组，对候选 2 的两种宽度与两种表形态）**：在
`run_constants_and_anchors` 里补 8 条算术断言（候选 2 表形态 `1+16*N_w`/`1+12*N_w`，N_w=4 取自
Q3-2 已量出的最大值，算出 65/49；甲、丙都不改字段表，与候选 2 同放按「单放」公式算：单条 16/12
余量 15/19（放得下），表形态 65/49 余量 −34/−18（推过 512）；乙、丁都把字段表改成 489，按「与乙
同放」公式算：表形态 65/49 余量 −42/−26，推过 512）：

```text
E7RESULT name=section_seven_two_arithmetic item="候选2表形态 1+16*N_w(N_w=4)" verdict=pass computed=65 registered=65
E7RESULT name=section_seven_two_arithmetic item="候选2表形态 1+12*N_w(N_w=4)" verdict=pass computed=49 registered=49
E7RESULT name=section_seven_two_arithmetic item="甲/丙同放候选2单条16 512-481-16" verdict=pass computed=15 registered=15
E7RESULT name=section_seven_two_arithmetic item="甲/丙同放候选2单条12 512-481-12" verdict=pass computed=19 registered=19
E7RESULT name=section_seven_two_arithmetic item="甲/丙同放候选2表形态65 512-481-65" verdict=pass computed=-34 registered=-34
E7RESULT name=section_seven_two_arithmetic item="甲/丙同放候选2表形态49 512-481-49" verdict=pass computed=-18 registered=-18
E7RESULT name=section_seven_two_arithmetic item="乙/丁同放候选2表形态65 512-489-65" verdict=pass computed=-42 registered=-42
E7RESULT name=section_seven_two_arithmetic item="乙/丁同放候选2表形态49 512-489-49" verdict=pass computed=-26 registered=-26
```

（同一份 `research/results/e158-root-choice-repair-2026-09-24-q2-1-hc1.out` 开头的常量回比段；
这 8 条与其余 local_constant_check/section_seven_two_arithmetic 一起在每次运行开跑时都会打印）
**这只是算术，不需要新建 5.3 表里没有的臂**：甲、丙已经「与今天同一份代码」，乙、丁的字段表改动
就是 5.3 表原文写的那 8 字节，数值早已由 Q2-2b/Q3-2 各自算出，这里只是把「与哪个修法同放」的四种
组合摆全。

**验证**：单测仍是 28 条全绿（本段没有新增单测——新增的两个 PC2 风格驱动函数与算术检查复用的都是
已被单测/变异覆盖过的既有函数：`root_ring_slot_targets_for`、`evidence_items_at`、`evaluate_
rootback_predicate`、`search_minimum_weight_that_triggers_rootback`，与既有的
`run_positive_control_for_the_root_ring_txg_only_candidate` 等驱动函数同一惯例——它们的正确性
靠自己的 E7RESULT 输出核对，不是靠专门的 `#[test]`）；1 条变异追加进 `crates/mutations.tsv`
（改候选 2 表形态算术的 `N_w` 从 4 改成 3，手工验证：改坏 → `constants_and_anchors_all_pass` 红
→ 还原 → 复绿），累计 16 条 E158（择根与修复四岔路） 变异行，门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次；`cargo fmt`/
`clippy -D warnings`（含共用约束那批 lint）/`naming-lint.sh` 对这份 bin 清零（`run_h_c1_lower_
bound_probe` 最初撞了单字母 `h` 违规，改名 `run_record_scan_watermark_lower_bound_probe` 后清零）。
新增 `driver_e158_q1_s16`、`driver_e158_q1_s4`、`driver_e158_q2_1_hc1`、
`driver_e158_q2_1_hc1_lower_bound` 登记进 `replay.sh`。

**没做的**（详见「它答不了的」）：乙-留环、丁-留环、丁-只配置三份副本的 ①②（这三个子臂的系统
不配置层面工程量在 session s5 建乙-只配置时已验证过，这一段的时间全给了 H-C1）；H2-R、H2c 两个
变体；n2=2；Φ2 系统配置槽证据类；Q2-2b。**岔路单第 2 行仍不算够判**——丙的 ① 现在是全实验最强的
一格证据，但乙、丁的多数子臂一格数据都没有。

**（续，session s7）：岔路单第 2 行三份新副本（乙-留环/丁-留环/丁-只配置）①②；岔路单第 1 行候选 (b) 两个子分支**

**开工重核（S1）**：`git rev-parse HEAD`=`e980a219f1834c638cd2fae18b25a60525a6bd52`、`git status
--porcelain crates/` 只剩 E158（择根与修复四岔路） 自己的 bin 与 `mutations.tsv`（2 行），与 session s6 收尾时一致；本
登记依赖的函数与常量再次现查全部相符，S1 不触发。

**岔路单第 2 行三份新副本**：`scripts/mutation/tables/e158_arms.tsv` 追加 47 行（乙-留环 15 行、丁-留环
与丁-只配置各 16 行——比乙族多一个 `published_txg_to_carry_over` 方法 + 取号那两处调用点改成续传）。
三份副本 `cp -a` 自同一次快照（`git_head`/`git_status_crates_lines`/`crates_sha256_of_sha256s` 与
开工时一致），各自 `cargo build --release` 编译通过，跑 `q2-1-g0`（Φ2 穷举下界，n1∈{0,...,6}）、
`q2-2a-g0`（固定脚本设备一层写字节）、`q2-1-pc2`（K2 给甲/乙的具体构造）三个模式（均加
`SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES=489`）。**结果整行抄自产物（产物已移出工作区的行写成一行指针）**：

```
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-yi-ring-retained.out 第 54 行（name=q2_1_minimum_weight）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-yi-ring-retained.out 第 58 行（name=q2_1_minimum_weight）
```
（`e158-root-choice-repair-2026-09-24-q2-1-g0-yi-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-ding-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-ding-only-configuration.out` 三份
逐字节相同；n1∈{1,2,3,5,6} 各档同一种形状，只贴 n1=0 与 n1=4 两档示例。）

```
E7RESULT name=q2_1_pc2_configuration_published_txg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) candidate=configuration_published_txg overwrites_before_the_crash=2 acquired_instance_before_the_crash=2 injected_faults=0 crash_points=1 hit=false expected_by_k2_table=true first_check="ok" second_check="ok"
```
（`e158-root-choice-repair-2026-09-24-pc2-yi-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-pc2-ding-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-pc2-ding-only-configuration.out` 三份都是
这一行；乙-只配置在同一个构造上是 `hit=true`，见 session s5。）

```
$ diff <(grep 'q2_2a_publish_writes' e158-root-choice-repair-2026-09-24-q2-2a-g0-today.out) <(grep 'q2_2a_publish_writes' e158-root-choice-repair-2026-09-24-q2-2a-g0-yi-ring-retained.out)
（无输出，rc=0；e158-root-choice-repair-2026-09-24-q2-2a-g0-ding-ring-retained.out、
e158-root-choice-repair-2026-09-24-q2-2a-g0-ding-only-configuration.out 两份同样 rc=0）
```

**读法**：①打中最少几个故障——三份副本在 Φ2（根槽+记录两类故障）穷举下 n1≤3 全空、n1∈{4,5,6} 撞顶
未穷尽；K2 给乙的具体构造（崩在取号之后、写行发布之前）在三份上都不打中——乙-留环因为公式仍读根环
（`max(ring, published_txg)`，崩溃没压低根环给出的值），丁两族因为取号续传（不写 0）。本段没有找到
任何打中这三份副本的构造，① 报告为「本段测过的两类攻法（Φ2 穷举、K2 给乙的构造）都没打中」，不是一
个具体的 k_min 数，也不能推广成「挡得住全部攻法」（Φ2 缺系统配置槽这一类证据）。②每次发布多写几字
节——三份副本与今天逐字节相同（Δ=0，设备一层的写按整槽计，加 8 字节字段不改变槽本身的写字节数）。

**岔路单第 1 行候选 (b) 两个子分支**：新写 `nearest_older_readable_root_on_same_instance`
（同一实例上最近的可读祖先）与 `tree_table_crates_has_a_path`（从祖先出发调 `replay_journal`，
`verify_named_units` 两种都跑，比对交回的 `tree_table` 指针）实现「crates 有路」；新写
`allocation_record_tree_reachable_placements_via_central_mapping`（走中央映射树、不重读被注入的
分配记录树节点）实现「从内容树反推占用集合」的一种操作化。**结果整行抄自产物（产物已移出工作区的行写成一行指针）**：

```
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-today.out 第 69 行（name=q1_2_tree_table_crates_has_a_path）
E7RESULT name=q1_2_tree_table_crates_has_a_path geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) verify_named_units=false outcome=no_older_readable_root_on_timeline count=9
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-today.out 第 71 行（name=q1_2_tree_table_crates_has_a_path）
E7RESULT name=q1_2_tree_table_crates_has_a_path geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) verify_named_units=true outcome=did_not_reach_target count=189
E7RESULT name=q1_2_tree_table_crates_has_a_path geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) verify_named_units=true outcome=no_older_readable_root_on_timeline count=9
E7RESULT name=q1_2_allocation_record_tree_from_content_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=central_mapping_unreadable count=27
E7RESULT name=q1_2_allocation_record_tree_from_content_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=unequal count=171
```
（`e158-root-choice-repair-2026-09-24-q1-g0-today.out`，覆盖了旧产物、纯增量，
旧的 76 行原样保留，只多了这 7 行诊断与末行 `emitted` 计数 76→83。）

**读法**：树表指称「crates 有路」——按今天挂载真用的口径（`verify_named_units=true`）198 个触发对
里零格给出匹配指针（189 格没推进到目标、9 格没有更旧的可读祖先），说明 `replay_journal` 在真实验证
口径下没有独立于那次被挡住的读的路径（验证点名单元那一步会去碰同一个物理位置）；放宽到不验证
（`verify_named_units=false`）时 81/189（约 43%）能给出匹配指针——journal 记录自带的新根段字段本
身携带了重建根字段所需的信息，只是「验证」这一步会去碰物理位置。这两个数字合起来：候选 (b) 树表指
称「crates 有路」这句话，在「按今天真实验证口径」下不成立；在「只信记录字段、放弃当场验证」这个更
弱的读法下，对将近一半的触发对成立。哪个读法才是候选 (b) 支持者的原意，交主 agent 判。分配记录树指
称「从内容树反推占用集合」——只走中央映射树的操作化，171 个可比对的对里**全部 `unequal`**（27 个
`central_mapping_unreadable` 是树表 0 条那一版 `mapping_root` 恒空指针，正当拿不到，不是读错）。
这不是候选 (b) 这条思路本身不成立，是「只走中央映射树」这一种操作化本身不完整——中央映射树只记「当
前逻辑地址映射到的数据落点」，分配记录树的「未释放」集合还含各棵树自己的结构节点（树表、8 棵子树各
自的根与内部/叶节点、accounting 树），要让这种操作化真正给出匹配的占用集合，还要另外走这些结构树。

两个新函数各配一条单测（`nearest_older_readable_root_on_same_instance_picks_the_closest_
ancestor_on_the_same_instance`、`allocation_record_tree_reachable_placements_via_central_
mapping_reflects_written_content_and_errors_on_an_empty_pointer`），逐条手工改坏验证会红、还原后
复绿；单测总数 28→30。变异表 `crates/mutations.tsv` 追加 2 行（对应这两条新单测），门禁 code-source-discipline 的 mutation-tables 格核过
原文各命中源码一次（536 条）。

**不改判据、不改臂定义**：`scripts/mutation/tables/e158_arms.tsv` 新增的 47 行是 5.3 表已写死的臂定义的
字面实现；新写的三个函数是跑前登记 5.2 表「② crates 有路」与「(b) 分配记录树：…从账里取的那一份」这
两句此前只有文字描述、现在实现成真跑得出结果的代码，定义原文一个字没有改。第五、六、七节的判据、
5.1–5.4 的臂定义、P331（打中判据）/P332（违例判据） 的定义同样一个字没有改。

**没做**：op1 第三种变体（先不注入 `mount_writable` 再 `raise_rollback_floor`）与持续故障（op1 及其
后两次挂载都带着）本段仍未实现；H2-R、H2c、n2=2、Φ2 系统配置槽证据类本段也仍未做，见「它答不了的」。

**（续，session s8）：岔路单第 2 行 Φ2 系统配置槽这一类证据；岔路单第 1 行候选 (b) 分配记录树指称「走全」**

**开工重核（S1）+ 实做才发现的 S4 变体**：`git rev-parse HEAD` 仍是 `e980a219f1834c638cd2fae18b25a60
525a6bd52`；`git status --porcelain crates/` 从 session s7 收尾时的 2 行涨到 31 行——另一条并行改动
线正在改 `mount.rs`/`recovery.rs`/`checker/walk.rs`。逐一现查本登记依赖的每个函数名，命中数全部 0，
S1 不触发。但把新写的装置源码拷进 session s7 已建好的三份旧副本重新编译时**编不过**：`replay_journal`
的返回类型在两个快照之间变了（旧：裸元组；今天：`Result<(...), _>`），session s7 自己新写的
`tree_table_crates_has_a_path` 调它的 `.expect(...)` 因此报错——这不是第三节任何一条（`replay_
journal` 不在依赖清单里，是 session s7 自己引入的依赖），照实记；处置：今天重新 `cp -a` 一份基准
快照（`git_status_crates_lines=31`、`crates_sha256_of_sha256s=fac44f8b508802d112dbdd4f1b826a54379
c87776d2ce43570f03ff78248798a`），从它派生三份新的乙-留环/丁-留环/丁-只配置副本（`e158_arms.tsv`
已有的 47 行原样重新套用，行数与 s7 一致：15/16/16），s7 的旧副本留作对照、不删除。详见跑前登记
「十二、修订」session s8 条目。

**Φ2 系统配置槽这一类证据**：`evidence_items_at` 新增 `include_system_configuration_slots` 开关；
打开时枚举两块盘各自 2 个自证过的系统配置槽（权重 1）。H-C2 直接构造（机制假说：`choose_system_
configuration` 按 `device_identities()` 顺序取第一块有效盘的值直接当 `chosen`，后面的盘只核对
`filesystem_identifier`/`device_count`——单测 `faulting_the_newer_device_zero_system_
configuration_slot_regresses_the_chosen_generation` 不依赖任何臂改法证明了这个假说对「今天」的
`choose_system_configuration` 成立）只把设备 0 上世代号更高的槽读失败（权重 1），加两个负对照。
**结果整行抄自产物（产物已移出工作区的行写成一行指针）**：

```
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-hc2-yi-ring-retained.out 第 54 行（name=q2_1_h_c2_system_configuration_evidence）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-hc2-yi-ring-retained.out 第 55 行（name=q2_1_h_c2_negative_control_older_slot）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-hc2-yi-ring-retained.out 第 56 行（name=q2_1_h_c2_negative_control_other_device）
```
（`e158-root-choice-repair-2026-09-24-hc2-yi-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-hc2-ding-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-hc2-ding-only-configuration.out` 三份逐字节
相同）——**三份副本上都 `hit=false`**：单个系统配置槽故障
（哪怕是精确针对机制假说构造的）不足以打中任何一条，与三条修法各自的设计初衷（乙-留环保留根环回
落、丁族取号续传）一致。

**Q2-1 加 Φ2 系统配置槽的穷举（n1 拆两段跑，见修订条目②）**：三份副本结果逐字节相同——n1=0、1 穷举
完整个证据空间都没打中（不是撞顶）；n1=2...6 全部撞 `SUBSET_ENUMERATION_CAP=20000` 上限、没找到
命中。

⚠️ **这一段原本在这里逐行抄的 `E7RESULT` 产物行，session s9 用新的 `weight_ceiling` 机制重跑
`q2-1-g0-configuration-evidence` 时把
`e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-yi-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-ding-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-ding-only-configuration.out`
三份文件**直接覆盖**了，没有先按「对不上就按今天的日期另存一份」的规矩另存旧产物——这是 session s9
的一处操作失误，如实记在这里：本段引用的这几行 `capped` 产物的原始字节今天已经找不到了，只有这段
文字里留着的数字（`subsets_tried=2048/8192/20000/20000/20000/20000`，n1=0..6 对应
`maximum_weight_available=14/17/…`）还能证明当时测过什么、测到了哪一步。**这几行的结论仍然成
立**（session s9 的新机制在这三份副本上重跑出的结果与这里描述的方向一致：n1=0、1 完整穷举不打中，
n1≥2 此前因为撞旧的计数上限没有搜完——新机制补齐了完整搜索，见「它答不了的」岔路 2 那一条的完整
读法与新产物），只是这一处不再贴可复核的原始行。**读法**：H-C2 与穷举是两条独立代码路径（H-C2 用
`attempt_a_faulted_mount_writable_and_append`，
穷举用 `attempt_rootback_probe_and_advance`），两条路径在这三份副本上给出一致的负结果——本段测过的
两类攻法（单点构造 + 权重递增穷举，穷举在 n1≤1 是完整的、n1≥2 只到 20000 子集为止）都没有找到能
打中乙-留环/丁-留环/丁-只配置的系统配置槽相关故障集合。**没有可报的 k（没有上界，因为没有一个能打
中的构造）**，只能报「测过的攻法都没打中」，不能推广成「挡得住全部攻法」。

**候选 (b) 分配记录树指称「走全」**：新增 `allocation_record_tree_reachable_placements_via_all_
structural_trees`，在只走中央映射树的基础上并入树表自己的节点、`TREE_KIND_INODE`/`_EXTENT`/
`_ACCOUNTING` 三种子树各自的根节点、中央映射树自己的节点（`TREE_KIND_ALLOCATION` 不并入，那是要
验证的账本身；`_LIVELIST`/`_SPARSE_SIDE_TABLE`/`_DEADLIST` 三种跑前登记原文没点名，没有并入）。
**结果整行抄自产物**：

```
E7RESULT name=q1_2_allocation_record_tree_from_content_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=central_mapping_unreadable count=27
E7RESULT name=q1_2_allocation_record_tree_from_content_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=unequal count=171
E7RESULT name=q1_2_allocation_record_tree_from_all_structural_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=structural_trees_unreadable count=27
E7RESULT name=q1_2_allocation_record_tree_from_all_structural_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=subset_of_pristine count=171
```
（`e158-root-choice-repair-2026-09-24-q1-g0-today.out`；S16、S4 两个几何点同样的
171/27 分布，`e158-root-choice-repair-2026-09-24-q1-s16.out`、`...-q1-s4.out`）。
**读法**：走全版本没有让任何一格从「不等」变成「相等」，但把 171 格从「不等，方向不明」精确到了
「`subset_of_pristine`，走全版本给出的集合仍然是真实未释放集合的真子集」——单测证明走全版本是只走
中央映射版本的严格超集（新增了树表与映射树自己的节点），所以这不是走全反而漏了东西，是**走全之后
仍然不够全**：真实的未释放集合里还有走全版本没覆盖到的落点（候选是 `TREE_KIND_LIVELIST`/`_SPARSE_
SIDE_TABLE`/`_DEADLIST`，或者是别的机制，本段没有再往下查）。这不是候选 (b) 这条思路整体不成立，
是这种操作化目前已知的范围（树表 + inode + extent + accounting + 映射树自己）仍不足以覆盖真实账的
全部内容。

**新增单测 5 条**（`system_configuration_slot_positions_uses_slot_spacing_for_the_second_slot_on_
each_device`、`evidence_items_at_includes_system_configuration_slots_only_when_asked`、
`candidate_fault_sets_of_weight_mixes_root_and_configuration_slots_by_weight`、`faulting_the_
newer_device_zero_system_configuration_slot_regresses_the_chosen_generation`、`allocation_
record_tree_reachable_placements_via_all_structural_trees_is_a_superset_of_the_central_mapping_
only_version`），逐条手工改坏验证会红、还原后复绿；单测总数 30→35。变异表 `crates/mutations.tsv`
追加 5 行（3 条给 Φ2、2 条给「走全」），逐条手工验证会红；改名 `candidate_fault_sets_of_weight` 的
第一个参数导致 session s5 写的一条既有变异锚点漂移，已同步改新变量名。

**不改判据、不改臂定义**：`evidence_items_at`/`run_rootback_tolerance_family`/`search_minimum_
weight_that_triggers_rootback` 新增的开关与参数，既有调用点都传关闭/默认值，逐字节复现之前的行为
（`q1-g0`/`q1-s16`/`q1-s4` 三份产物按纯增量覆盖，旧内容原样保留，各多 2 行诊断）；第五、六、七节的
判据、5.1–5.4 的臂定义、P331（打中判据）/P332（违例判据） 的定义一个字没有改。

**没做**：op1 第三种变体与持续故障仍未实现；H2-R、H2c、n2=2、Q2-2b 仍未做；「走全」仍不是穷举（还差
`_LIVELIST`/`_SPARSE_SIDE_TABLE`/`_DEADLIST` 三种、以及为什么仍是 `subset_of_pristine` 的根因）。

**（续，session s9）：171 格差集的内容与根因；op1 起三步挂载的持续/瞬时轨迹；n1=2…6 撤掉穷举的计数上限；实十九多层树防护**

**171 格差集 = 实例表自己的节点，不是 LIVELIST/SPARSE_SIDE_TABLE/DEADLIST**：新增 `classify_diff_
pair`，对每个 `subset_of_pristine` 格算 `pristine − 走全` 差集、逐个落点分类。**结果整行抄自产物**
（`e158-root-choice-repair-2026-09-24-q1-g0-today.out`）：

```text
E7RESULT name=q1_2_subset_diff_pair geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) device=0 slot=50176 label=instance_table_self_node_not_walked_by_the_all_structural_trees_arm occurrences=171
E7RESULT name=q1_2_subset_diff_pair geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) device=1 slot=50176 label=instance_table_self_node_not_walked_by_the_all_structural_trees_arm occurrences=171
E7RESULT name=q1_2_subset_diff_distinct_pairs geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) distinct_pairs=2
```

`(0, 50176)`/`(1, 50176)` 就是 `UNIT_AREA_START_SLOT`=`INSTANCE_TABLE_SLOT`——`RootRecord.
instance_table` 是与 `tree_table`/`mapping_root` 并列的第三条指针，走全版本没读过它。核过
`transaction.rs` 的 `publish_admitted`：`TREE_KIND_LIVELIST`/`_SPARSE_SIDE_TABLE`/`_DEADLIST` 三
条树表条目的根**恒为** `NodePointer::empty_root()`，是全仓每次发布都这样、不是这批历史的特例——差
集与这三种树无关，候选 (b) 走全版本真正的缺口是实例表。

**op1 起三步挂载的持续/瞬时轨迹**（8.1 要
求）：新写 `mount_writable_trajectory`，复用 PC1-a 已选中的落点：

```text
E7RESULT name=q1_1a_op1_trajectory geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) mode=persistent trajectory=[Some(1), Some(1), Some(1)]
E7RESULT name=q1_1a_op1_trajectory geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) mode=transient trajectory=[Some(1), Some(0), Some(0)]
```

与登记原文逐字吻合。**n1=2…6 撤掉了穷举的计数上限**：`search_minimum_weight_that_triggers_
rootback` 的 `subset_enumeration_cap`（会在权重档中途停手）换成 `weight_ceiling`（只在权重档边界
停，恒报 `full_space_subset_count` 与 `highest_weight_examined`）。今天/丙臂
`q2-1-g0`（`research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-today-session-s5.out`）：

```text
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=4 overwrites_after_the_rootback_probe=1 k_min=12 hit_count_at_that_weight=1 first_hit_combination=["root(region=0,slot=2)", "root(region=1,slot=1)", "root(region=1,slot=2)", "root(region=2,slot=1)", "record(instance=1,counter=4)", "record(instance=1,counter=5)", "record(instance=1,counter=6)", "record(instance=1,counter=7)"] subsets_tried=22558 full_space_subset_count=32768 rootback_probe_failures=29 overwrite_failures=153
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=5 overwrites_after_the_rootback_probe=1 k_min=not_found_up_to_weight_ceiling highest_weight_examined=12 maximum_weight_available=25 full_space_subset_count=131072 subsets_tried=65536 rootback_probe_failures=9 overwrite_failures=120
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=6 overwrites_after_the_rootback_probe=1 k_min=12 hit_count_at_that_weight=1 first_hit_combination=["root(region=0,slot=2)", "root(region=0,slot=3)", "root(region=1,slot=2)", "root(region=2,slot=2)", "record(instance=1,counter=6)", "record(instance=1,counter=7)", "record(instance=1,counter=8)", "record(instance=1,counter=9)"] subsets_tried=173716 full_space_subset_count=524288 rootback_probe_failures=10 overwrite_failures=57
```

**读法**：n1=4、6 此前撞旧计数上限报 `capped`，现在主搜索本身（不靠 H-C1 那种额外直接构造）就在
两处独立给出 `k_min=12`，与 session s6 的 H-C1 直接构造互证；n1=5 完整空间 131072，只搜完一半
（65536，因为按权重分档的组合数在这个空间上关于权重 12 对称）就停在权重 12，如实报「未打中」，不
是「挡得住」——`stopped_by_weight_ceiling` 字段与 `full_space_subset_count`/`highest_weight_
examined` 一起报，不再是一个不说明含义的 `capped`。**乙-留环/丁-留环/丁-只配置三份副本本段中途按
主 agent 要求重跑完，不留成欠账，读法见「它答不了的」岔路 2 那一条**。

**实十九多层树防护**：
`allocation_record_tree_reachable_placements_via_central_mapping` 加了 `level != 0` 检查——手工
构造 `level=1` 的假节点证明改动前会**静默解出一个假的落点集合**（不是报错）；实测这批历史
中央映射树根 `entries.len()` 恒为 6、`level` 恒为 0，远低于 294 的单叶容量，暂时不会撞上，但检查
留在代码里。新增单测 4 条（`classify_diff_pair_identifies_the_instance_table_self_node`、
`mount_writable_trajectory_distinguishes_persistent_from_transient_faults`、`search_minimum_
weight_that_triggers_rootback_honors_an_explicit_weight_ceiling`、`allocation_record_tree_
reachable_placements_via_central_mapping_rejects_a_multi_level_root`），单测 35→39；变异表追加 4
行、修了 1 行既有锚点（门禁 code-source-discipline 的 mutation-tables 格红——`classify_diff_pair` 插入位置挤开了原有一条变异的锚点，已改到
函数体内更窄的一段，重新核对命中一次）。**完整读法见跑前登记「十二、修订」session s9 条目**（8 条，逐条附手工
验证依据）。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1254-1280`（整段抄，未转述）**

````markdown
### 结果整行抄自产物（产物已移出工作区的行写成一行指针）

**2026-09-24 主 agent 定案**：`driver_e158`（第一段）承重产物改指
`e158-root-choice-repair-2026-09-24-segment1-rerun.out`（82 行，
`E7RESULT name=done emitted=70`）——这是 `crates/` 落地 C512（树表 0 条的一版上被换下的实例表记在哪没有条款）
定案（拿掉「回退到无文件那一版」的拒绝）**之后**重跑的产物；`scripts/experiment-apparatus/replay.sh` 的
`driver_e158` 行已改指到它。旧产物 `e158-root-choice-repair-2026-09-23-segment1.out`
原样留着，对应 **C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 落地之前**的代码，不再是这一行比对的对象，50 行差异的逐格原因见「2026-09-24：第一段重跑」一节
一节与 `/tmp/claude-1000/e158-s2/report.md` 第 1 节。

```text
E7RESULT name=s5_independent_decode_cross_check verdict=pass readable_root_count=6 readable_roots={(0, 0), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5)}
E7RESULT name=h3_family_summary geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 histories=1822 peak_events_live=3 peak_roots_live=10 steps_with_positive_events_live=1702 leaf_count=1605 leaves_with_positive_events_live=1524 peak_events_live_at_a_leaf=3 error_count=855
E7RESULT name=h3_family_summary geometry=S16(S=16,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 histories=1822 peak_events_live=3 peak_roots_live=10 steps_with_positive_events_live=1702 leaf_count=1605 leaves_with_positive_events_live=1524 peak_events_live_at_a_leaf=3 error_count=855
E7RESULT name=h3_family_summary geometry=小环 overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 histories=1822 peak_events_live=3 peak_roots_live=10 steps_with_positive_events_live=1702 leaf_count=1605 leaves_with_positive_events_live=1524 peak_events_live_at_a_leaf=3 error_count=855
E7RESULT name=h3_family_summary geometry=S4(S=4,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=4 histories=14252 peak_events_live=4 peak_roots_live=10 steps_with_positive_events_live=13857 leaf_count=12430 leaves_with_positive_events_live=12155 peak_events_live_at_a_leaf=4 error_count=10629
E7RESULT name=peak_live_rollback_events_across_the_family value=4
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-segment1-rerun.out 第 88 行（name=pc3）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-segment1-rerun.out 第 89 行（name=pc3_overall_verdict）
```

（`grep -c '^E7RESULT' e158-root-choice-repair-2026-09-24-segment1-rerun.out` → `70`；`grep -c 'verdict=pass'` → `42`、`grep -c 'verdict=fail'` → `0`；分类计数 `grep -oE 'name=[a-z0-9_]+' e158-root-choice-repair-2026-09-24-segment1-rerun.out | sort | uniq -c` 见「复跑」。`pc3` 的 `path` 从旧产物的 `RollbackTo(1, 3)` 变成 `RollbackTo(0, 0)`——PC3 取「按固定次序第一个满足 events_live≥1 的历史」，`(0,0)` 在 2026-09-24 重跑时合法且排在更前，核心断言 `expected_root=(1, 6)` 与两处 `verdict=pass` 不变。）

**岔路单第 2 行 ③（乙与候选 2 抢不抢槽空间）**：Q2-3 在与乙同放的四种宽度里，单条 16/12 都放得下（余量 7/11），两种表形态都同放推过 512（余量 −42/−26）——判定落在候选 2 取哪种宽度形态上，不落在单条上，与跑前登记的预判一致。

**岔路单第 3 行（候选 2 那一半）**：Q3-2 的 N_w = 4（来自 S4/L=4，比 `GEOMETRY_PRIMARY`/S16/小环 三个几何点的 3 更大），候选 2 的表形态见证（1+16*N_w=65、1+12*N_w=49）都推过 512（放得进 4096），单条见证不推过 512；PC3 用已有用例的同一种形态（回退之后再让新实例自己的根全部读不出）坐实了「读得出的被抛弃根里 (txg,实例) 最大的那一条」这个操作化的判定式与实测（recover / mount_writable）逐字一致。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1281-1328`（整段抄，未转述）**

````markdown
### 复跑

```
bash scripts/experiment-apparatus/replay.sh E158
```

第一段（`driver_e158`，`-- all`）**2026-09-24 主 agent 定案改指今日承重产物**（见「结果整行抄自产物」与
「第一段重跑」两节：与 2026-09-23 旧产物比 50 行不同，逐格原因已诊断；改指之后 `driver_e158` 与今日承重产物
`e158-root-choice-repair-2026-09-24-segment1-rerun.out` 字节一致），旧产物原样留着、不再是比对对象；
Q3-1 四个几何点各占一行（`driver_e158_q3_1_g0` / `_s16` / `_small_ring` / `_s4`，走同一个二进制的
`q3-1-*` 模式），其中 G0（主几何点） 已现查两次独立跑逐字节相同（确定性核对，`diff` 无输出）。

**2026-09-24 复核**：
主工作区在段-2 之后又打进一批 `crates/` 未提交改动（`git status --porcelain crates/` 87 个文件，覆盖
`mount.rs`/`recovery.rs`/`transaction.rs`/`allocator.rs`/`instance_table.rs`/`root_record.rs`/
`root_ring.rs`/`system_configuration.rs`/`write_accounting.rs`/`singlefs-checker` 的 `image.rs`/`walk.rs`
等），重跑 `bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路） 一次：**5 个驱动全部「字节一致」**（`driver_e158` 与
四个 `q3_1_*`），说明这批改动没有改变 E158（择根与修复四岔路） 已测的任一条观测面；日志 `/tmp/claude-1000/e158-s3/replay-e158-2.log`。

**session s3 第二段**又新增 `driver_e158_q1_g0`（岔路单第 1 行，候选 (c)=今天，只跑 G0（主几何点） ）并登记进
`replay.sh`；重跑 `bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路）：**6 个驱动全部「字节一致」**（日志
`/tmp/claude-1000/e158-s3/replay-e158-3.log`）。候选 (a)=A1 副本的同一份数**不登记进 `replay.sh`**
（产物 `e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out`，副本上的数；复跑要先按
`scripts/mutation/tables/e158_arms.tsv` 在草稿目录重建 A1 副本、把
`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 原样拷进去、`cargo build --release`、
跑 `-- q1-g0`）。单测：`cargo test -p singlefs-verification --bin e158_root_choice_repair` → `16 passed; 0 failed`
（12 条到 session s3 第一段为止 + 4 条 session s3 第二段新增：`fault_targets_for_both_copies_hits_both_devices`、
`fault_severity_fault_count_matches_the_prereg_table`、`error_member_of_debug_extracts_the_variant_name`、
`location_key_ignores_the_checksum_field`）。变异：9 条手工验证（改坏必红、还原后 `diff` 逐字节一致、
基线重新全绿），登记在 `crates/mutations.tsv` 里 E158（择根与修复四岔路） 的九行（每行变异名都以「root_choice_repair」这个 bin 名起头，前面缀着这个实验的编号），归门禁 crates-mutation-replay 跑；
门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次（2026-09-24 `cargo fmt` 把 `is_abandoned` 的实现拆成三行后，对应那一条的
原文跟着改成三行、用 `\n` 分隔，重新核过命中一次）。运行记录（挂钟、git 状态、sha256）在
`e158-root-choice-repair-2026-09-23-segment1-run-record.out`（只覆盖 2026-09-23 第一段，
不参与 `replay.sh` 的逐字节比对）。

**session s4** 又新增 `driver_e158_q2_1_g0`（岔路单第 2 行 ①，候选 (c)=今天，只跑 G0（主几何点） 、n1∈{0,1,2,3}、
n2=1）与 `driver_e158_q2_2a_g0`（岔路单第 2 行 ②，固定脚本）并登记进 `replay.sh`；重跑
`bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路）：8 个驱动（含新增两个）全部「字节一致」（日志
`/tmp/claude-1000/e158-s4/replay-final.log`）。单测：`cargo test -p singlefs-verification --bin
e158_root_choice_repair` → `22 passed; 0 failed`（新增 6 条，见「session s4」一节）。变异：3 条手工
验证（改坏必红、还原后 `diff` 逐字节一致、基线重新全绿），登记在 `crates/mutations.tsv` 里 E158（择根与修复四岔路）
的十二行（累计，每行变异名都以「root_choice_repair」这个 bin 名起头，前面缀着这个实验的编号），归门禁 crates-mutation-replay 跑；门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次（489 条）。
甲-txg 臂的同一份数**不登记进 `replay.sh`**（产物 `e158-root-choice-repair-2026-09-24-q2-1-g0-jia-txg-arm.out`
与 `e158-root-choice-repair-2026-09-24-q2-2a-g0-jia-txg-arm.out`，副本上的数；复跑要先按
`scripts/mutation/tables/e158_arms.tsv` 甲-txg 那一行在草稿目录重建副本、把
`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 原样拷进去、`cargo build --release`、
分别跑 `-- q2-1-g0`、`-- q2-2a-g0`）。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1329-1410`（整段抄，未转述）**

```markdown
### 它答不了的

- **岔路 1（C393（被抛弃根的账读不出时修复没有条款）） Q1-1 至 Q1-4 有数了，171 格差集的内容与根因
  session s9 已经查清，但 op1 第三种变体与几何敏感性仍不全**：见「2026-09-24（续，session s3 第二
  段）」「2026-09-24（续，session s6）」「2026-09-24（续，session s9）」三节的完整读法。几何敏感性
  session s6 已补两个方向相反的点（S16、S4，与 G0（主几何点） 逐字节相同，判定不翻面），小环、`G_默认` 两点仍未
  跑。session s7/s8 补了候选 (b) 的三种操作化（「crates 有路」、只走中央映射树、「走全」），171 格
  从「`unequal`，方向不明」变成「`subset_of_pristine`，方向已知但仍不够全」。**session s9 把这 171
  格的差集内容与根因查清了**：差集只有 2 个不同的落点（`(0, 50176)`、`(1, 50176)`，各出现 171
  次），就是 `RootRecord::instance_table` 自己的物理节点（`crates/singlefs-format` 的
  `UNIT_AREA_START_SLOT`=50176，与 `INSTANCE_TABLE_SLOT` 是同一个槽）——它是与 `tree_table`/
  `mapping_root` 并列的第三条独立指针，走全版本从来没有读过它；`TREE_KIND_LIVELIST`/
  `_SPARSE_SIDE_TABLE`/`_DEADLIST` 三种树在 2026-09-24 的 `crates/` **全仓任何一次发布**都恒是
  `NodePointer::empty_root()`（现查 `transaction.rs` 的 `publish_admitted`），不是这批历史的特例，
  也不是候选 (b) 走全版本漏了它们——差集与这三种树无关。**session s9 还把 8.1 要的持续/瞬时故障轨
  迹补上了**：`mount_writable_trajectory` 在 PC1-a 已选中的同一个落点上给出 persistent=
  `[Some(1), Some(1), Some(1)]`、transient=`[Some(1), Some(0), Some(0)]`，与登记原文逐字吻合。
  **还差**：op1 第三种变体（先不注入 `mount_writable` 再 `raise_rollback_floor`，见跑前登记「十二、
  修订」session s9 条目第 5 条——`raise_rollback_floor` 的签名理论上接得进 `SimNode.session`，但要
  正确处理它内部推的若干次空发布各自产生的中间根，工程量本段没做完）；小环、`G_默认` 两个几何点
  （S16/S4 已补，dispatch 要求的「至少一个方向相反的几何点」已满足）。这些缺口逐条列在跑前登记
  「十二、修订」2026-09-24（session s3、s7、s8、s9）条目，不是随手漏掉。
- **岔路 2（C331（择根倒挂压过已确认的写） ①②③）session s9 用新机制把乙-留环/丁-留环/丁-只配置三
  份副本补齐到 n1=0..6，这一个切面够判了，但完整的「六臂×两档×两种」矩阵仍不算够判**：见「session
  s6」「session s7」「session s9」三节的完整读法。甲-txg 的 k_min 稳定=4，丙的 k_min 恰好=12（session
  s6 穷举下界坐实、session s9 的 `q2-1-g0` 主搜索本身在 n1=4 与 n1=6 上又各自独立给出 12），都与判
  决 K2 一致；③（四条修法与候选 2 抢不抢槽空间）session s6 已给出算式。**session s9 撤掉了旧的计
  数上限（`SUBSET_ENUMERATION_CAP`，权重档中途停手、不报完整空间多大），换成按权重档边界停的
  `weight_ceiling` 机制，恒报完整空间大小与穷举到的权重**：乙-留环、丁-留环、丁-只配置三份副本从
  实十九落地之后的新快照重建，跑完整个 n1=0..6：

  | n1 | 完整空间 | 结局 | subsets_tried |
  |---|---|---|---|
  | 0 | 2048 | 完整穷举，none_found | 2048 |
  | 1 | 8192 | 完整穷举，none_found | 8192 |
  | 2 | 32768 | 完整穷举，none_found（旧上限下报 capped，这次是真穷举完了） | 32768 |
  | 3 | 131072 | 穷举到权重 12，not_found_up_to_weight_ceiling | 82598 |
  | 4 | 524288 | 穷举到权重 12，not_found_up_to_weight_ceiling | 229720 |
  | 5 | 2097152 | 穷举到权重 12，not_found_up_to_weight_ceiling | 582829 |
  | 6 | 8388608 | 穷举到权重 12，not_found_up_to_weight_ceiling | 1367161 |

  三份副本逐字节相同
  （`e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-yi-ring-retained.out`、
  `e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-ding-ring-retained.out`、
  `e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-ding-only-configuration.out`）。
  **读法**：n1=0..2
  是完整穷举（`subsets_tried` 恰等于完整空间），n1=3..6 都在权重 12 那一档边界上停手，完整空间与
  穷举到的权重如实报出，没有静默截断。跑前登记 5.8 分段表第 777 行「Q2-1 全族里各模板区间…的最小
  者」要求的是**全族**（n1=0 到 6），这次是全族的结果：**乙-留环/丁-留环/丁-只配置在权重≤12、覆
  盖 H2 全族的范围内，一次都没有被今天/丙那个「4 根槽 + 4 条记录」权重 12 的具体构造打中**——它们
  的 k_min（如果存在）这次确立为 **> 12**，与甲-txg 的区间 `[4,4]`、丙的区间 `[12,12]` 都不重叠。
  按第 761 行「行 2 够判，除非各臂的 k_min 区间两两重叠」，**这一个切面（今天/甲-txg/丙 与
  乙-留环/丁-留环/丁-只配置，崩溃档 0，任一种打中）这次够判**——K2 给乙的那个「0 故障+1 崩溃点」
  具体构造（session s5/s7）在乙-留环上仍 `hit=false`（保留根环读取挡住了这个具体构造），在丁-留环、
  丁-只配置上也仍 `hit=false`（取号续传挡住了）；H-C2 直接构造（session s8，精确针对
  `choose_system_configuration` 的设备优先级机制假说）与两负对照在三份副本上仍 `hit=false`。
  **仍不是完整的「六臂×两档×两种」矩阵，标「够判后未跑」，不是本段能补的欠账**：H2-R
  （op2=`mount_rollback`）、H2c（崩溃档 1）两个变体；n2=2（n1 已补到 {0,...,6}）；Q2-2b（乙族字段
  表宽度解码核对，三份副本都只跑了固定脚本按结构种类报字节，没有单独做字段表宽度解码核对）。
- **岔路 4（C334（切换的所选根没有会红的检查）与 Q4-1/Q4-2）** 前置未满足，按登记第五段不开，不是失败也不是
  作废。⚠️ **session s3（2026-09-24 续）更正**：`grep -rn -i 'switch' crates/singlefs-core/src/` **不再是
  0 命中**——新文件 `crates/singlefs-core/src/admission.rs` 有 19 处（`INSTANCE_SWITCHES_ALLOWED_PER_MOUNT`、
  `instance_switch_reserve_on_one_device`），逐条读过确认是 D28（挂载期承诺量） 已定项 3 的**空间预留算术**，
  不是实例切换本身（没有取新号、写行、重发在飞 checkpoint 的代码）；`grep -rn -i 'probe'` 22 处全在
  `block_device.rs` 的 `probe_physical_block_size`（探测物理块大小），与 D23（journal 的角色与格式） 已定项 14 的「固定落点探针写」
  无关。两项前置条件逐条读过后仍都不满足，P4 判定不变，但「0 命中」这句表述本身已经不成立——
  `impl-m2-mountfix` 已经在为实例切换打地基（先落空间预留），下一次核 P4 时留意 `admission.rs`。
- **Q3-1 的 175 个「V1 在 (a) 消失、V2 在 (a)(b)(c) 三处持续为真」的对，具体形态没有拆开看**：不知道它们是不是
  都落在「回退到无文件版本」这个新出现的分支上、内容号具体对应哪个历史节点——这一段只报了汇总计数，没有
  逐条摘出来读；`e158-root-choice-repair-2026-09-24-q3-1-g0.out` 里有全部原始数据，
  逐条拆解留给下一段或三方判决用。
- **G_默认（768 MiB 环）没有独立跑 Q3-1**：`scan_journal` 按环全部记录槽逐一读，768 MiB 环是 3 MiB 环的 256 倍，
  按 G0（主几何点） 的 real 2m58s 线性外推跑不完；只给出机制论证（判定不依赖环总槽数，已在 S=8→S=16、3 MiB→49152 字节
  两个方向验证过），**不是独立观测**。
- **H3 的省法自己没有单独验证过会不会漏判**：5.1 原文要求省法有自检抽样（「每 100 个抽 1 个全做」），Q3-1
  的实现里省法条件是「(a) 所选根与 F=∅ 相同」，逻辑上（b)(c) 在 F=∅ 时必然非违例，但**没有做 5.1 原文那条
  独立抽检**去证明「省掉的那些的确都不会是违例」——这一段直接论证了省法的正确性（见「新增了什么」一节），
  但没有按登记原文再加一层抽样自检，留给下一段判是否需要补。
- **小环取法的代理探针是简化版**：见 2026-09-23 记录，未变。
- **门禁 doc-experiments 的 results-cited 格与当时编号 85、86 的两道阶段的完整核对**：本次修改实验页与索引行、修了 `cargo fmt`/clippy，但没有重新跑一遍
  这三个阶段确认变绿（归属 `gate-triage`）。

```

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1411-1493`（整段抄，未转述）**

````markdown
### 2026-09-25（续，session s10）：op1 第三种变体（岔路单第 1 行）；`replay.sh` 全量重出；发现另一条并行线正在改的 `crates/` 影响岔路单第 3 行的前提

**op1 第三种变体做完了**：新增 `FaultedOperationKind::MountWritableThenRaiseFloorTo`（先不注入地
做一次 `mount_writable` 只为量出这一步的 F 与上限，受注入的一步是 `raise_rollback_floor` 本身），
按跑前登记 Q1-1c 原文「抬 F 那一处单独一行」不并入既有的 `pairs`/`trigger_count`。**结果整行抄自
产物**（`research/results/e158-root-choice-repair-2026-09-25-q1-g0-today.out`）：

```text
E7RESULT name=q1_1a_raise_floor_trigger_summary geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) pairs=96 trigger_count=32 history_nodes_without_room=26
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=both_copies pairs=16 trigger=16
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk0_only pairs=16 trigger=0
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk1_only pairs=16 trigger=0
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=both_copies pairs=16 trigger=16
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk0_only pairs=16 trigger=0
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk1_only pairs=16 trigger=0
```

**读法**：两份都读失败（`both_copies`）时两个指称各 16/16 全触发，只坏一份（`disk0_only`/
`disk1_only`）时 0/16——与既有 `mount_writable`/`mount_rollback` 那两种 op1 在同一批构造上的模式
一致（另一份仍读得出账）；`history_nodes_without_room=26` 是这个历史节点在这一步没有 `[F+1, 上限]`
余量可抬（不是漏测）。G0（主几何点） 与 S16 两点逐字节相同
（`research/results/e158-root-choice-repair-2026-09-25-q1-s16.out`），S4 不同
（`pairs=132 trigger_count=44 history_nodes_without_room=25`，
`research/results/e158-root-choice-repair-2026-09-25-q1-s4.out`）——几何变了、H1 家族的节点与候选
根都会变，不要求与 G0（主几何点） 相同。单测 39→43（4 条新增：3 条 `floor_targets_between` 边界 + 1 条端到端
「不注入必须成功、注入必须与不注入不同」），变异 30→33（3 条），`cargo test --release -p
singlefs-verification --bin e158_root_choice_repair` 43 passed，`bash .claude/gate.d/code-source-discipline.sh --check mutation-tables` 复跑绿。**没做「把 raise_rollback_floor 里若干次空发布各自的中间根追加进
node.timeline」**——现查发现这一步不需要：`RaisedFloor::abandoned_roots_unreadable` 是从盘上现读
实例表 + 现行根的候选集算出来的，不依赖装置自己的内存时间线（与 `MountRollbackTo` 分支同理，那个
分支交回的中间根同样从不追加进时间线），详见跑前登记「十二、修订」session s10 条目第 2、3 条。

**`replay.sh` 全量重出，发现另一条并行线正在改的 `crates/` 影响岔路单第 3 行的前提**：
`bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路）（14 行）：字节一致 4／对不上 10／跑不了 0。1 行
（`driver_e158_q2_1_g0`）是 session s4 遗留、driver 自己的注释早就说明不再当依据引用，本来就该对
不上；其余 9 行按今天日期另存新产物：
`research/results/e158-root-choice-repair-2026-09-25-segment1-rerun.out`、
`research/results/e158-root-choice-repair-2026-09-25-q3-1-g0.out`、
`research/results/e158-root-choice-repair-2026-09-25-q3-1-s16.out`、
`research/results/e158-root-choice-repair-2026-09-25-q3-1-small-ring.out`、
`research/results/e158-root-choice-repair-2026-09-25-q3-1-s4.out`、
`research/results/e158-root-choice-repair-2026-09-25-q1-g0-today.out`、
`research/results/e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out`、
`research/results/e158-root-choice-repair-2026-09-25-q1-s16.out`、
`research/results/e158-root-choice-repair-2026-09-25-q1-s4.out`；
`…-09-24-…` 原样留着，`replay.sh` 登记表已改指向新文件。逐份现查差在哪（不是只看行数），除了本段新增的 `q1_1a_raise_floor_*` 行之外，全部差异都指
向同一批文件：主 agent 知会另一条并行线（「实二六」）此刻正在改
`crates/singlefs-core/src/mount.rs`、`allocator.rs`、`allocation_record_tree.rs`（抬 F、挂载读、
分配记录树根层，见「重出产物之前」与「交回前」两份 crates/ 快照
`e158-root-choice-repair-2026-09-25-s10-crates-sha256.out`、
`e158-root-choice-repair-2026-09-25-s10-crates-sha256-final.out`）：

- `driver_e158` 的 `pc3` 一行从 `recover_verdict=pass recover_root=Some((1, 6))` 变成
  `recover_verdict=fail recover_root=Some((0, 0))`——与 session s9 报告的同一处翻转方向一致。
- **`q3_1_g0`/`q3_1_s16`/`q3_1_small_ring`（岔路单第 3 行 Q3-1）**：`pairs_with_any_violation`
  从 213 变成 0；**`q3_1_s4`** 除了同一项从 1355 变成 0，`cold_recover_with_fault_failed`/
  `mount_writable_with_fault_failed` 还从 0 变成 239——同一批构造里，239 个从「成功但违例」变成
  「调用直接报错」，失效形态本身变了。**⚠️ 这一条可能动到岔路单第 3 行「候选 3 = 今天」
  这个前提与 F2 的触发条件**：C332（回退实例两个根都读不出时回退被撤销） 正文钉的「2 个故障就能撤
  销回退」在今天这份 `crates/`（含另一条并行线还没提交的改动）上现在测不出来了。是要重新核对 F2、
  还是这一刻的 `crates/` 状态本就不该当「候选 3 = 今天」的代表（等那条并行线落定提交后再复核），
  交主 agent 判断，本段不越权下结论、不改判据或臂定义。
- `q1_2_tree_table_crates_has_a_path`（`verify_named_units=false`）从「`did_not_reach_
  target=108`+`reached_target_with_matching_pointer=81`」变成「`did_not_reach_target=189`」（原
  来能不碰物理拷贝复算出指针的 81 格现在全部走不到）；`pc1_a` 的 `device_write_bytes` 从 157184
  变 353792；`q2_2a_g0` 的 `AllocationRecordTreeNode` 每次发布 `write_calls`/`written_bytes` 全
  面上涨（如 `write_calls=2`→`10`）——三处都与 session s9 报告的方向一致，是既有漂移的重复坐实。
- **没有做第二次确认性复跑**：新存的 9 份文件是这一次真实运行的原始输出，逐字节复跑要再等两个
  `q2-1-g0` 全穷举跑一遍（约 1～2.5 小时），而 crates/ 这一刻仍可能继续变，投入产出比低，续不续跑
  交主 agent 定。

**doc-lint 第 497 行附近的位置指代——现查没有找到**：`bash .claude/scripts/doc-lint.sh` 全仓跑一遍
对本页给出 0 处违规；第 498 行「详见 `.claude/kb/experiments/158-择根与修复四岔路.md`「2026-09-24
（续，session s9）」一节」指向的标题在第 759 行确实存在，是写明文件名与小节标题的有效引用。派发点
名的问题这一刻的正文里没有找到，大概率是主 agent 派发时看到的是 session s9 那两处自指还没修的中间
状态（s9 交回前已改成写死文件名与小节标题、复跑确认过 0 处），不强行改一处现查看不出毛病的文字。

**它答不了的（更新）**：岔路单第 1 行——op1 第三种变体已做完，仍差小环、`G_默认` 两个几何点；岔路
单第 2 行——H2-R、H2c、n2=2、Q2-2b 仍是「够判后未跑」；本段现查到的 `crates/` 波动（`mount.rs`/
`allocator.rs`/`allocation_record_tree.rs`，另一条并行线还没提交）让岔路单第 3 行「候选 3 = 今天」
这个前提本身可能已经不成立，交主 agent 判断续不续核；`replay.sh` 没有做第二次确认性复跑——理由见
「（续，session s10）：op1 第三种变体（岔路单第 1 行）；`replay.sh` 全量重出；发现另一条
并行线正在改的 `crates/` 影响岔路单第 3 行的前提」这一节「`replay.sh` 全量重出……」一段最后一条。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1494-1503`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 6 | 备料 | 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |
| D22（单元原子性怎么合成） 已定项 9 | 备料 | 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |
| D8（核心索引结构） 已定项 8 | 备料 | 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |
| D28（挂载期承诺量） 已定项 3 | 备料 | 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:1-25`（整段抄，未转述）**

````markdown
## E174 借据丢掉时等待与显存回还 —— 两段跑完、够判

**中文** · [English](174-wait-and-gpu-memory-return-on-lost-iou.en.md) · [日本語](174-借用証書喪失時の待機とGPUメモリ返却.ja.md)

<!-- doc-lint:not-numbers G1 G2 S1 S2 Q1 Q1b Q2a Q2b Q2c Q3 Q3b Q3c K1 K2 K3 K4 K5 K6 K7 K8 K9 B1 B2 B3 B4 B5 B6 B7 F1 F2 F3 F4 F5 F6 F7 F8 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 P1 P2 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 M15 M16 M17 M18 M19 M20 M21 M22 M23 M24 M25 M26 M27 M28 -->

```text
本页的本地臂名与记号写小写（j1、j2 ……）：跑前登记、装置、产物与命令里写作对应的大写，与全局登记的编号同形；kb 正文里改成小写只为与那些编号分开，读产物时按大写对。
```

跑前登记 `research/prompts/e174-preregistration.md`（装置与单测写完、真卡产物一次没跑之前的 sha256 `b2d5cd0c56552ec1ad322d5d8cca22f1fcb705617cb876d3726635058edc0d92`；执行员在第十二节「修订」写了 1 条：补一条臂「在飞·中」、钉死几处登记没写死的读法，原判据一字没动）；岔路单 `m3-batch10z-a-code-r1-forks.md` 的 G2 一行里"乙"的两个数：借据丢掉时先等设备把这笔借据提交的活做完再销账（`device.poll` 用等待那一种），等一次要多久（G2-乙-等待）；等完、缓冲丢掉之后，下一次 `nvidia-smi -q -d MEMORY,PIDS` 读到这个进程占着的是否已经不含那笔缓冲，含多少、隔多久才不含（G2-乙-读数）。问法里"再丢掉缓冲"按登记的读法：缓冲对象在借据活着时先丢，借据丢掉时再 poll，poll 返回那一刻算销账。

装置是独立手写的，不是入库装置：`scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs`（bin 名 `e174-lease-drop-wait-memory-return`，特性 `e174-gpu`，依赖沿用 E163（GPU多卡算单元校验和） 那份钉死的 `wgpu =30.0.1`、`pollster =1.0.1`），不依赖 `crates/`、不与实现共用代码。设备怎么起（Vulkan、`required_limits: adapter.limits()`、`MemoryHints::Performance`）、读数口径（`nvidia-smi -q -d MEMORY,PIDS` 按进程号把 `Used GPU Memory` 加起来）、`PollType::Poll` 的写法都抄自跑时 `crates/` 的取法，开跑前用登记第十一节那 7 条 `grep -c` 回比过：输出依次 2、1、1、1、1、1、1，与登记写的值逐条相同；驱动版本现读是 595.84（`nvidia-smi --query-gpu=index,name,pci.bus_id,memory.total,driver_version --format=csv` 两张卡都报 595.84，Total 32607 MiB 与 16311 MiB）。

**现状**：两段真卡产物都跑完、`judge` 判过，G2-乙 的两个数在两段上都量到了，登记第十一节的够判条件满足（状态「两段跑完、够判」）。

- **第一段**（设备上只有这一笔借据的活，不含在飞·轻）：88 格、446 个子进程，430 个完整、16 个失败；16 个失败全在 5060 Ti 的 8 GiB、不等待臂，wgpu 报 Out of Memory，那两格（在飞·中、在飞·重各一格）作废，别的 86 格、168 栏都凑满 5 个有效轮。
- **第二段**（同一设备两条线程共用，不含在飞·轻）：12 格、60 个子进程全完整，24 栏。
- **在飞·轻**两段都没跑，记「够判后未跑」：登记第九节 K9 在登记自己的数下必红（读一次显存的时长大于那一档活的 0.8 倍），修订 1 补的「在飞·中」一条臂在每个用在飞·轻的组里顶了它。
- **G2-乙-读数**：等待臂（第二段还有 `Wait` 带自己最后一次提交号的那一条臂）在两段里每一栏（第一段的阳性对照「扣住不丢」除外，它按设计含），t0 之后的第一份 `nvidia-smi -q -d MEMORY,PIDS` 读数都不含那笔缓冲，窗口末也不留；不等待臂：第一段里活还在飞、S 不小于 512 MiB 时第一份读数含整笔、2 秒窗口里一直不还，第二段 S = 2048 MiB 的 4 栏第一份读数含整笔、0.7 到 1.15 秒后还回去。读数与计数命令在「读数：第一段」「读数：第二段」两节。
- **G2-乙-等待**：已做完档的等待 27 微秒以内；活在飞的两档，等待比剩下的活本身多出来的部分从负几百微秒到 213.6 毫秒不等；同一设备另有一条线程一直在提交 400 毫秒的活时，等一次要 0.3 到 0.69 秒。数在两节读数里。
- 用户已照两段的数在岔路单 `m3-batch10z-a-code-r1-forks.md` G2 一行定「改用乙」，实现在第 10z 批第六块，不在这一页。

不起卡的部分（装置、单测 19 个、变异表 28 条全抓、自检、模型卡上的父子流水线）的读数留在「读数：不起卡的几份」「单测与变异」两节。

````

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:26-33`（整段抄，未转述）**

```markdown
### 做了什么

- **一笔借据的步骤**（登记第五节）写成对着 `CardHost` 这个 trait 的一份代码，有两份实现：真卡（`WgpuCardHost`）与模型卡（`SimulatedCardHost`，虚拟时钟，读一份报告记 50 毫秒，小于 256 MiB 的缓冲共用一块 256 MiB 的块、块用过就留着，更大的单独拿，释放要等缓冲被丢掉、提交做完、之后有一次 poll）。登记第九节的 M1–M7 都改在对着 trait 的那份代码上，所以不起卡也有单测可红。模型卡只复现步骤的算术，不是 wgpu 与驱动的行为，不拿来当 G2-乙 的读数。
- **子命令**：`run`（父进程，按清单一个接一个起子进程，等它退出后整份读、不在转发输出的循环里打时间戳；续跑、补轮、原文压成 `.raw.gz`、扣住不丢组出现 F1 就停）、`child`（一个装置进程）、`judge`（读产物判，确定性）、`checks`（不起卡的自检，确定性）、`plan`（清单各阶段的格数与子进程数，确定性）、`simulate`（在模型卡上跑某一格几轮，试 `judge` 用）；`run --simulated` 让每个子进程跑在模型卡上，`--limit-jobs N` 冒烟，`--skip-light` 先不跑在飞·轻（这两样是缩域，产物 config 行写出实际值；不给就是整段）。
- **清单**（`e174-lease-drop-wait-memory-return-plan-first-2026-10-08.out`、`…-plan-second-2026-10-08.out`）：第一段 128 格、首轮 640 个子进程，第二段 24 格、首轮 120 个子进程；"扣住不丢"最先，在飞·轻最后。
- **修订 1**：登记第九节 K9 要 e < 0.8 W，而 e 至少含 U_peak 那一次 -q 读与 B5 另读的一次 compute-apps（各约 0.05 秒），在飞·轻 W = 50 毫秒的 0.8 倍只有 40 毫秒，K9 在登记自己的数下必红。在飞·轻照登记保留、K9 照登记判，另补一条臂「在飞·中」（空转目标 250 毫秒）给每个用在飞·轻的组；已经作废的借据不再读窗口。其余是读法钉死：一轮有效的定义、补轮从 6 起至多 3 次、次序、B5 字面读法、F1 停机、判别力自证怎么取新 τ。
- **判法**：`judge --input 产物` 对每格按"第 1 笔"与"第 2、3 笔"两栏、取轮号最小的 5 个有效轮，逐轮列出 Q1（等多久）、Q2a / Q2b（t0 之后第一份是否含）、Q2c（隔多久不含）、Q3（窗口末留着）、Q3b（按块多拿）、Q3c（wgpu 分配器），再给一行 `name=verdict` 汇总 F1–F8 与几何敏感性；汇总里"…_ok"类字段写成通过的样子，`false` 就是那条没过（F2、F3、F4 打中是登记认的正当结果，不是装置有错）。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:34-59`（整段抄，未转述）**

````markdown
### 复跑

```
# 仓根；装置在 scripts/experiment-apparatus/（不依赖 crates/）。真卡上的命令也整条经 task-server 交，卡、清场、内存额度、线程由服务定
cd /home/fy5090/code/singlefs/scripts/experiment-apparatus
# 不起卡：单测 19 条、变异表 28 条、自检、清单
../../task-server/task submit -- cargo test --release -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return
../../task-server/task submit -- bash ../mutation/mutate.sh e174-lease-drop-wait-memory-return e174_lease_drop_wait_memory_return.rs ../mutation/tables/e174_lease_drop_wait_memory_return.tsv
../../task-server/task submit -- cargo run --release -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- checks
../../task-server/task submit -- cargo run --release -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- plan --segment first
# 上卡：先冒烟两个子进程，再第一段（不含在飞·轻），再在飞·轻，再第二段；同一条命令重起就是续跑；一律经 scripts/experiment-apparatus/device-code-key.py run 在仓根起
cd ../..
task-server/task submit -- python3 scripts/experiment-apparatus/device-code-key.py run . --binary e174-lease-drop-wait-memory-return --feature e174-gpu --entry e7_index_bench::run_child_on_the_card --entry e7_index_bench::ChildJob::child_arguments -- run --segment first --skip-light --limit-jobs 2 --output research/results/e174-lease-drop-wait-memory-return-segment-one-<日期>.out
# 判（确定性，读产物）
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- judge --input research/results/e174-lease-drop-wait-memory-return-segment-one-<日期>.out --scope first
```

两段真卡产物是 `run --segment first --skip-light` 与 `run --segment second --skip-light` 写的（产物第 1 行 `config` 的 `skip_light=true` 为证），命令形态是冒烟那条命令去掉 `--limit-jobs`（`gpu-quiesce-run.sh -- … run --segment first --skip-light --output …`，第二段换 `--segment second`）。真卡产物含计时与显存读数，两次跑不同：复跑表的 `driver_e174_run_segment_one`、`driver_e174_run_segment_two` 两行经 `device-code-key.py run` 不带 `--skip-light`、`--limit-jobs` 整段重跑两段，按 `timing` 比结构；逐字节比的是它们的 `judge` 输出（确定性，读产物）。判的命令，仓根下跑：

```
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- judge --input research/results/e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out --scope first
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return -- judge --input research/results/e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out --scope second
```

`scripts/experiment-apparatus/replay.sh` 登记六行，整段重跑的两行 E174（借据丢掉时等待与显存回还） `|@driver_e174_run_segment_one`、E174（借据丢掉时等待与显存回还） `|@driver_e174_run_segment_two`（`timing`）之外是这四行：E174（借据丢掉时等待与显存回还） `|@driver_e174_checks`、E174（借据丢掉时等待与显存回还） `|@driver_e174_simulated_judge`（不起卡、不含计时字段的两份）与 E174（借据丢掉时等待与显存回还） `|@driver_e174_segment_one_judge`、E174（借据丢掉时等待与显存回还） `|@driver_e174_segment_two_judge`（真卡两段产物的 `judge` 输出，产物 `…-segment-one-judge-2026-10-08.out`、`…-segment-two-judge-2026-10-08.out`）。replay.sh 整份目前一开跑就在准入表检查上退 2（`bash scripts/experiment-apparatus/replay.sh --admission-only` 报「E142（新池新建文件的干跑） 登记了准入，而这张复跑表里它有 2 行」），与这四行无关，所以这四行没经 replay.sh 复跑，是手工跑同一条命令，输出与登记的产物 `cmp` 逐字节相同：两段的 judge 命令都退 0、stderr 为空，`cmp` 无输出。

````

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:60-68`（整段抄，未转述）**

```markdown
### 产物清单

`research/results/` 下 E174（借据丢掉时等待与显存回还） 的全部产物：
- 自检与清单：`e174-lease-drop-wait-memory-return-checks-2026-10-08.out`、`e174-lease-drop-wait-memory-return-plan-first-2026-10-08.out`、`e174-lease-drop-wait-memory-return-plan-second-2026-10-08.out`；
- 单测与变异：`e174-lease-drop-wait-memory-return-unit-tests-2026-10-08-r2.out`、`e174-lease-drop-wait-memory-return-mutate-2026-10-08-r2.log` 是现行的；`e174-lease-drop-wait-memory-return-unit-tests-2026-10-08.out`、`e174-lease-drop-wait-memory-return-mutate-2026-10-08.log` 是被 r2 取代的第一次；
- 模型卡流水线：`e174-lease-drop-wait-memory-return-simulated-second-2026-10-08.out`、原文 `e174-lease-drop-wait-memory-return-simulated-second-2026-10-08.out.raw.gz`、判的输出 `e174-lease-drop-wait-memory-return-simulated-second-judge-2026-10-08.out`；
- 真卡第一段：`e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out`、原文 `e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out.raw.gz`、判的输出 `e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out`；
- 真卡第二段：`e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out`、原文 `e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out.raw.gz`、判的输出 `e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out`。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:69-75`（整段抄，未转述）**

```markdown
### 读数：不起卡的几份（模型卡数据，不是 G2-乙 的读数）

- **自检**（`research/results/e174-lease-drop-wait-memory-return-checks-2026-10-08.out`，33 行，31 条自检）：第 32 行 `E7RESULT name=verdict scope=checks self_checks_total=31 self_checks_failed=0 void_conditions=0`。检查本身分得出差别：第 16–18 行三条负对照（模型卡被注入别的进程号、compute-apps 比 -q 多 3 MiB、标定不在带内）各自打出对应的作废理由。第 12 行 `E7RESULT name=self_check id=simulated_light_trips_k9_at_the_registered_numbers passed=true`：模型卡上在飞·轻按登记的数 K9 必红，复现的是修订 1 的算术。
- **清单**：`…-plan-first-2026-10-08.out` 里 `E7RESULT name=verdict scope=plan segment=first cells_total=128 first_pass_children_total=640 void_conditions=0`；`…-plan-second-2026-10-08.out` 里 `E7RESULT name=verdict scope=plan segment=second cells_total=24 first_pass_children_total=120 void_conditions=0`。
- **父子流水线走通**（`run --simulated --segment second --skip-light`，子进程都在模型卡上）：`…-simulated-second-2026-10-08.out`（243 行）第 242 行 `E7RESULT name=verdict scope=run segment=second cells_total=12 cells_incomplete=0 children_total=60 children_complete=60 children_failed=0 children_void=0 stopped_by=none`，原文压缩包 `…-simulated-second-2026-10-08.out.raw.gz`（`zcat` 读得出，每个子进程一段）。同一条命令用 `--limit-jobs 7`、`--limit-jobs 20` 分三次续跑，与一次跑完的产物和原文 `cmp` 逐字节相同（草稿读数，续跑文件不入库）。
- **判法在模型卡产物上**：`…-simulated-second-judge-2026-10-08.out`（26 行）第 25 行 `E7RESULT name=verdict scope=second columns_judged=24 cells_void=0 hold_control_holds=not_run wait_arm_next_reading_absent_everywhere=false wait_arm_end_of_window_not_kept_everywhere=false wait_always_returns=true poll_arm_also_absent_where_wait_arm_absent=false in_flight_cells_void_for_work_not_in_flight=0 long_window_clear_within_window=not_run geometry_same_as_main_grid=not_run done_wait_expensive_candidate_columns=0 discrimination_real_data_flipped_columns=12 discrimination_synthetic_flips=true`。这几个 `false` 来自模型卡自己的规则（小于 256 MiB 的借据让进程多占一块 256 MiB 的块、块用过就留着），模型卡上的数不是 wgpu 与驱动的数；`not_run` 是这份产物里没有对应的组。判别力自证：真数据里 τ 挪到含着的读数的中位数时 12 栏翻面，另用模型卡产物自证 `discrimination_synthetic_flips=true`。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:76-274`（整段抄，未转述）**

````markdown
### 读数：第一段（真卡，设备上只有这一笔借据的活）

产物 `research/results/e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out`（1699 行，原文压缩包是同名加 `.raw.gz`），判的输出 `research/results/e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out`（188 行）。命令都在 `research/results/` 下跑，先设四个变量（`S2`、`j2` 是第二段两份）：

```
cd /home/fy5090/code/singlefs/research/results
S1=e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out
J1=e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out
S2=e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out
J2=e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out
```

产物第 1 行与第 1698 行（`grep -n 'name=config' $S1`、`grep -n 'name=verdict' $S1`，原样带行号；真卡产物含计时与显存读数，再跑要上卡，不登记进 `scripts/experiment-apparatus/replay.sh`，登记的是它的判的输出）：

```
1:E7RESULT name=config segment=first seed=174 driver_version=595.84 skip_light=true cells=88 rounds_required=5 refills_at_most=3 simulated=false
1698:E7RESULT name=verdict scope=run segment=first cells_total=88 cells_incomplete=2 children_total=446 children_complete=430 children_failed=16 children_void=0 stopped_by=none
```

判的输出第 187 行（`grep -n 'name=verdict' $j1`）：

```
E7RESULT name=verdict scope=first columns_judged=168 cells_void=2 hold_control_holds=true wait_arm_next_reading_absent_everywhere=true wait_arm_end_of_window_not_kept_everywhere=true wait_always_returns=true poll_arm_also_absent_where_wait_arm_absent=false in_flight_cells_void_for_work_not_in_flight=0 long_window_clear_within_window=true geometry_same_as_main_grid=false done_wait_expensive_candidate_columns=0 discrimination_real_data_flipped_columns=40 discrimination_synthetic_flips=true
```

**栏数**（组 × 臂；每格分「第 1 笔」「第 2、3 笔」两栏，长窗一格一栏）与每栏的轮数：

```
grep 'name=cell_verdict' $J1 | awk '{print $3,$8}' | sort | uniq -c
      8 group=coexisting_long_lease arm=poll
      8 group=coexisting_long_lease arm=wait
      8 group=hold_control arm=poll
      8 group=hold_control arm=wait
      4 group=long_window arm=wait
     56 group=main arm=poll
     60 group=main arm=wait
      8 group=pass_by_pass arm=poll
      8 group=pass_by_pass arm=wait
grep -h -o 'rounds_used=[0-9]*' $J1 $J2 | sort | uniq -c
    192 rounds_used=5
```

主格子不等待臂 56 栏而不是 60：5060 Ti 上 8 GiB 的两格作废（最后一条命令的 192 是两段 168 + 24 合计）。

**阳性对照「扣住不丢」**（登记 F1）：缓冲对象在 t0 之后还活着，读数必须含它。两条臂、两张卡、512 MiB 与 2 GiB、在飞·中，16 栏 R_next 都含整笔，判的输出里 `hold_control_holds=true`：

```
grep 'name=cell_verdict' $J1 | grep ' group=hold_control ' | grep -o ' q2_next_reading=[a-z_]*' | sort | uniq -c
     16  q2_next_reading=present
```

**G2-乙-读数，等待臂**（登记 Q2a、Q2c、Q3）：除扣住不丢之外的 80 栏等待臂（主格子 60、已做完逐趟 8、并存的长借据 8、长窗 4），t0 之后的第一份读数都不含，隔 0 毫秒就不含，窗口末（2 秒；长窗 30 秒）也不留：

```
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | grep ' arm=wait ' | grep -o ' q2_next_reading=[a-z_]*\| q2c_time_to_clear=[a-z_]*\| q3_end_of_window=[a-z_]*' | sort | uniq -c
     80  q2c_time_to_clear=next_reading_already_absent
     80  q2_next_reading=absent
     80  q3_end_of_window=absent
```

**不等待臂**（登记 Q2b，跑时 `crates/` 的做法）：活已经做完的档（已做完、逐趟）下一次读数不含；活还在飞（中、重）的档，S 不小于 512 MiB 时下一次读数含整笔，窗口里一直不还（`not_absent_within_window`）；S = 1、64 MiB 不含。8 GiB 在 5060 Ti 上没有读数（「作废的两格」一段）：

```
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | grep ' arm=poll ' | grep -o ' q2_next_reading=[a-z_]*\| q2c_time_to_clear=[a-z_]*' | paste - - | sort | uniq -c
     48  q2_next_reading=absent	 q2c_time_to_clear=next_reading_already_absent
     24  q2_next_reading=present	 q2c_time_to_clear=not_absent_within_window
grep 'name=cell_verdict' $J1 | grep ' group=main ' | grep ' arm=poll ' | grep -v ' liveness=done ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["card"],d["size_mib"],d["liveness"],d["q2_next_reading"]}' | sort | uniq -c
      2 rtx5060ti 1 heavy absent
      2 rtx5060ti 1 medium absent
      2 rtx5060ti 2048 heavy present
      2 rtx5060ti 2048 medium present
      2 rtx5060ti 512 heavy present
      2 rtx5060ti 512 medium present
      2 rtx5060ti 64 heavy absent
      2 rtx5060ti 64 medium absent
      2 rtx5090 1 heavy absent
      2 rtx5090 1 medium absent
      2 rtx5090 2048 heavy present
      2 rtx5090 2048 medium present
      2 rtx5090 512 heavy present
      2 rtx5090 512 medium present
      2 rtx5090 64 heavy absent
      2 rtx5090 64 medium absent
      2 rtx5090 8192 heavy present
      2 rtx5090 8192 medium present
```

第一条命令的 24 个「含」是主格子 20 栏（5090 的 512、2048、8192 MiB 与 5060 Ti 的 512、2048 MiB，各在飞·中、在飞·重两档，每格两栏）加并存的长借据 64 MiB 的 4 栏；并存的长借据那一组写在同名的一段。

**G2-乙-等待**（登记 Q1、Q1b）：每栏 5 轮的 T_wait 与 `judged`（已做完档就是 T_wait；在飞档是 T_wait 超出剩下的活的部分 E，负数是等待比剩下的活短）按组、档、臂取全部轮的最小与最大（微秒）。已做完档的等待最长 27 微秒，在飞·中 175–392 毫秒、在飞·重 325–571 毫秒；不等待臂都在 24 微秒以内：

```
awk '/name=cell_verdict/ && !/ group=hold_control /{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} n=split(d["q1_wait_microseconds"],v,";"); m=split(d["q1_judged_microseconds"],w,";"); k=d["group"]" "d["liveness"]" "d["arm"]; for(j=1;j<=n;j++){x=v[j]+0; if(!(k in lo)||x<lo[k])lo[k]=x; if(!(k in hi)||x>hi[k])hi[k]=x} for(j=1;j<=m;j++){y=w[j]+0; if(!(k in jlo)||y<jlo[k])jlo[k]=y; if(!(k in jhi)||y>jhi[k])jhi[k]=y}} END{for(k in lo)print k, "wait_us", lo[k], hi[k], "judged_us", jlo[k], jhi[k]}' $J1 | sort
coexisting_long_lease medium poll wait_us 1 12 judged_us 1 12
coexisting_long_lease medium wait wait_us 175786 254725 judged_us -175 17137
long_window medium wait wait_us 161848 248056 judged_us -124 33292
main done poll wait_us 1 22 judged_us 1 22
main done wait wait_us 1 27 judged_us 1 27
main heavy poll wait_us 2 24 judged_us 2 24
main heavy wait wait_us 325083 571300 judged_us -196 213567
main medium poll wait_us 1 20 judged_us 1 20
main medium wait wait_us 175290 392336 judged_us -185 165674
pass_by_pass done poll wait_us 2 11 judged_us 2 11
pass_by_pass done wait wait_us 2 13 judged_us 2 13
```

每栏的判定（登记 Q1 的门槛：已做完档 5 轮都 ≤ 5 毫秒记「零头」；在飞档 5 轮都 ≤ 20 毫秒记「只等活本身」，都 > 20 毫秒记「等得比活长」，否则「不稳定」）。已做完档 28 栏全是「零头」，`done_wait_expensive_candidate_columns=0`（没有「乙的代价翻面候选」）；在飞档 34 栏「只等活本身」、13 栏「等得比活长」、5 栏「不稳定」，没有一栏「等不回来」（判的输出里 `wait_always_returns=true`）：

```
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | grep ' arm=wait ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["liveness"],d["q1_wait"]}' | sort | uniq -c
     28 done zero_overhead
      6 heavy longer_than_the_work
     12 heavy only_the_work_itself
      2 heavy unstable
      7 medium longer_than_the_work
     22 medium only_the_work_itself
      3 medium unstable
grep -h -c 'q1_wait_never_returned=true' $J1 $J2
0
0
```

在飞档里不是「只等活本身」的 18 栏列在这里：5090 的 8192 MiB、5060 Ti 的 2048 与 8192 MiB 是「等得比活长」，别的是 5 轮不都在 20 毫秒同一侧的「不稳定」：

```
grep 'name=cell_verdict' $J1 | grep ' arm=wait ' | grep -v ' liveness=done ' | grep -v ' group=hold_control ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} if(d["q1_wait"]!="only_the_work_itself") print d["group"],d["card"],d["size_mib"],d["liveness"],d["column"],d["q1_wait"]}' | sort
long_window rtx5060ti 2048 medium first longer_than_the_work
long_window rtx5090 2048 medium first unstable
main rtx5060ti 2048 heavy first longer_than_the_work
main rtx5060ti 2048 heavy later longer_than_the_work
main rtx5060ti 2048 medium first longer_than_the_work
main rtx5060ti 2048 medium later longer_than_the_work
main rtx5060ti 512 heavy later unstable
main rtx5060ti 512 medium later unstable
main rtx5060ti 8192 heavy first longer_than_the_work
main rtx5060ti 8192 heavy later longer_than_the_work
main rtx5060ti 8192 medium first longer_than_the_work
main rtx5060ti 8192 medium later longer_than_the_work
main rtx5090 2048 heavy later unstable
main rtx5090 2048 medium later unstable
main rtx5090 8192 heavy first longer_than_the_work
main rtx5090 8192 heavy later longer_than_the_work
main rtx5090 8192 medium first longer_than_the_work
main rtx5090 8192 medium later longer_than_the_work
```

**Q3、Q3b、Q3c**（窗口末留着、按块多拿、wgpu 分配器）：等待臂窗口末都不留（Q3 80 栏 `absent`，见 Q2a 那条命令的第三行）；wgpu 分配器在等待臂 80 栏都「不留块」。不等待臂有 24 栏「wgpu 留着块」，这 24 栏都是第一份读数含的那 24 栏。并存的长借据 64 MiB 的两条臂各 4 栏 Q3b 判「按块多拿」（缓冲活着时进程多占的比这笔多），等待臂等完之后仍不含：

```
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["arm"],"q3b="d["q3b_block_overtake"],"q3c="d["q3c_wgpu_blocks"]}' | sort | uniq -c
     48 poll q3b=absent q3c=wgpu_does_not_keep_blocks
     20 poll q3b=absent q3c=wgpu_keeps_blocks
      4 poll q3b=present q3c=wgpu_keeps_blocks
     76 wait q3b=absent q3c=wgpu_does_not_keep_blocks
      4 wait q3b=present q3c=wgpu_does_not_keep_blocks
grep 'name=cell_verdict' $J1 | grep -v ' group=hold_control ' | grep ' arm=poll ' | grep ' q2_next_reading=present ' | grep -c 'q3c_wgpu_blocks=wgpu_keeps_blocks'
24
```

**并存的长借据**（登记第八节几何，F8）：子进程里另有一张 64 MiB 缓冲活到子进程末尾，借据 S 取 1、64 MiB，在飞·中。判的输出逐对比主格子同一格，等待臂 8 对都相同，不等待臂的 S = 64 MiB 四对不同——并存的长借据下第一份读数含 256 MiB（一整块），主格子里不含。`geometry_same_as_main_grid=false` 来自这四对：

```
grep 'name=geometry_pair' $J1 | grep -c 'same=true'
12
grep 'name=geometry_pair' $J1 | grep ' arm=wait ' | grep -c 'same=true'
8
grep -n 'name=geometry_pair' $J1 | grep 'same=false'
177:E7RESULT name=geometry_pair group=coexisting_long_lease card=rtx5090 size_mib=64 liveness=medium method=one_build arm=poll column=first q2_main_grid=absent q2_coexisting=present q3_main_grid=absent q3_coexisting=present same=false
178:E7RESULT name=geometry_pair group=coexisting_long_lease card=rtx5090 size_mib=64 liveness=medium method=one_build arm=poll column=later q2_main_grid=absent q2_coexisting=present q3_main_grid=absent q3_coexisting=present same=false
185:E7RESULT name=geometry_pair group=coexisting_long_lease card=rtx5060ti size_mib=64 liveness=medium method=one_build arm=poll column=first q2_main_grid=absent q2_coexisting=present q3_main_grid=absent q3_coexisting=present same=false
186:E7RESULT name=geometry_pair group=coexisting_long_lease card=rtx5060ti size_mib=64 liveness=medium method=one_build arm=poll column=later q2_main_grid=absent q2_coexisting=present q3_main_grid=absent q3_coexisting=present same=false
```

登记 F8 写的是「Q2a、Q3 在那几格记『不稳定』，乙在这一形上不够判」，触发观测是「第八节逐格对，主格子在飞·轻与『并存的长借据』组判定不同」；第八节同时写「被判的是『等待臂读数不含』与『留不留着』两件事」。按字面逐格对，F8 在不等待臂的四对上打中（比的是在飞·中，不是在飞·轻）；只看被判的等待臂，八对都相同，F8 没打中。登记没写不等待臂上的差别算不算打中，两种读法并列在这里，哪种承重交主 agent。

**作废的两格**（登记 V8：同一格有 2 轮子进程报 Out of Memory 或非 0 退出，那一格作废并交回原文）：5060 Ti 上 S = 8 GiB 的不等待臂，在飞·中与在飞·重各一格，轮 1 至 8（首轮 5 个加补轮 3 个）的子进程都以 wgpu 的 Out of Memory 退出；同一张卡同一个 S 的等待臂 6 栏（三档 × 两栏）正常，不等待臂的已做完档 2 栏也正常。16 个 `child_failed` 行的原文（stderr 末尾）在产物里；第几笔借据报的错没有留下，因为失败的子进程没有 `lease` 行，原文压缩包里也没有它们的段：

```
grep -c 'name=child_failed' $S1
16
grep 'name=child_failed' $S1 | grep -c 'card=rtx5060ti size_mib=8192 .* arm=poll'
16
grep 'name=child_failed' $S1 | grep -c 'Out_of_Memory'
16
grep -c 'name=cell_void' $J1
2
grep 'name=cell_verdict' $J1 | grep ' card=rtx5060ti size_mib=8192 ' | awk '{print $6,$8}' | sort | uniq -c
      2 liveness=done arm=poll
      2 liveness=done arm=wait
      2 liveness=heavy arm=wait
      2 liveness=medium arm=wait
grep 'name=lease ' $S1 | grep -c 'card=rtx5060ti size_mib=8192 liveness=\(medium\|heavy\) .*arm=poll'
0
zcat $S1.raw.gz | grep -c '^######## .*card=rtx5060ti size_mib=8192 liveness=\(medium\|heavy\) .*arm=poll'
0
```

**判别力自证**（登记第八节）：把 τ 换成含着的读数的中位数，真数据上 40 栏判定翻面（`discrimination_real_data_flipped_columns=40`），模型卡产物另证 `discrimination_synthetic_flips=true`（两项都在判的输出第 187 行）。

````

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:275-340`（整段抄，未转述）**

````markdown
### 读数：第二段（真卡，同一设备几条线程共用）

产物 `research/results/e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out`（243 行，原文压缩包是同名加 `.raw.gz`），判的输出 `research/results/e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out`（26 行）。一个子进程两条线程：线程 B 在同一个设备上一次接一次提交 400 毫秒的活，线程 A 做一笔借据（S = 64 MiB、2 GiB，在飞·中，空转目标 250 毫秒，一次建）；臂三条：`Wait { submission_index: None }`（下记 wait）、`Wait { submission_index: Some(线程 A 这笔借据最后一次提交) }`（wait_own）、`Poll`（poll）。两张卡，每格 5 轮。变量同第一段那一节。

产物第 1 行与第 242 行（`grep -n 'name=config' $S2`、`grep -n 'name=verdict' $S2`，原样带行号；不登记进 `scripts/experiment-apparatus/replay.sh`，理由同第一段）：

```
1:E7RESULT name=config segment=second seed=174 driver_version=595.84 skip_light=true cells=12 rounds_required=5 refills_at_most=3 simulated=false
242:E7RESULT name=verdict scope=run segment=second cells_total=12 cells_incomplete=0 children_total=60 children_complete=60 children_failed=0 children_void=0 stopped_by=none
```

判的输出第 25 行（`grep -n 'name=verdict' $j2`）：

```
E7RESULT name=verdict scope=second columns_judged=24 cells_void=0 hold_control_holds=not_run wait_arm_next_reading_absent_everywhere=true wait_arm_end_of_window_not_kept_everywhere=true wait_always_returns=true poll_arm_also_absent_where_wait_arm_absent=false in_flight_cells_void_for_work_not_in_flight=0 long_window_clear_within_window=not_run geometry_same_as_main_grid=not_run done_wait_expensive_candidate_columns=0 discrimination_real_data_flipped_columns=4 discrimination_synthetic_flips=true
```

**G2-乙-读数**（Q2a、Q2c、Q3、Q3c）：wait 与 wait_own 各 8 栏，t0 之后第一份读数都不含、隔 0 毫秒就不含、窗口末不留、wgpu 不留块；poll 在 S = 2048 MiB 的 4 栏第一份读数含整笔 2048 MiB，之后在窗口（2 秒）里还回去，T_clear 的 20 个值（两张卡 × 两栏 × 5 轮）在 0.71 到 1.15 秒之间（判定名 `absent_after_wait`：每轮都有限，不是都为 0）；S = 64 MiB 的 4 栏不含。第一段的不等待臂在同样是 S = 2048 MiB、活在飞的格子里 2 秒窗口里一直不还（`not_absent_within_window`），第二段的 poll 会还。两段的差别是第二段有线程 B 一直在设备上提交活，读数还回去与它有没有关系，这些数分不出来，页上不下结论：

```
grep 'name=cell_verdict' $J2 | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["arm"],d["q1_wait"],"q2="d["q2_next_reading"],"q2c="d["q2c_time_to_clear"],"q3="d["q3_end_of_window"],"q3c="d["q3c_wgpu_blocks"]}' | sort | uniq -c
      4 poll does_not_wait q2=absent q2c=next_reading_already_absent q3=absent q3c=wgpu_does_not_keep_blocks
      4 poll does_not_wait q2=present q2c=absent_after_wait q3=absent q3c=wgpu_does_not_keep_blocks
      8 wait longer_than_the_work q2=absent q2c=next_reading_already_absent q3=absent q3c=wgpu_does_not_keep_blocks
      8 wait_own longer_than_the_work q2=absent q2c=next_reading_already_absent q3=absent q3c=wgpu_does_not_keep_blocks
grep 'name=cell_verdict' $J2 | grep ' arm=poll ' | awk '{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} print d["card"],d["size_mib"],d["column"],"q2="d["q2_next_reading"],d["q2_remainders_mebibytes"]}' | sort
rtx5060ti 2048 first q2=present 2048;2048;2048;2048;2048
rtx5060ti 2048 later q2=present 2048;2048;2048;2048;2048
rtx5060ti 64 first q2=absent 0;0;0;0;0
rtx5060ti 64 later q2=absent 0;0;0;0;0
rtx5090 2048 first q2=present 2048;2048;2048;2048;2048
rtx5090 2048 later q2=present 2048;2048;2048;2048;2048
rtx5090 64 first q2=absent 0;0;0;0;0
rtx5090 64 later q2=absent 0;0;0;0;0
grep 'name=cell_verdict' $J2 | grep ' arm=poll ' | grep ' size_mib=2048 ' | grep -o ' q2c_values_microseconds=[0-9;]*'
 q2c_values_microseconds=803316;707802;743818;732683;719862
 q2c_values_microseconds=1025177;903407;897814;1016805;793476
 q2c_values_microseconds=797972;860259;821002;1026236;851788
 q2c_values_microseconds=932552;854468;1015238;1146156;986349
```

**G2-乙-等待**（Q1、Q1b）：按臂、栏取全部轮的 T_wait（`wait_us`）与超出剩下的活的部分（`judged_us`）的最小与最大（微秒）。wait 与 wait_own 的区间大幅重叠（wait 306–691 毫秒、wait_own 301–639 毫秒），两条臂的 16 栏都判「等得比活长」；poll 在 19 微秒以内。第 2、3 笔那一栏（`later`）的下界比第 1 笔高（wait 418 对 306 毫秒，wait_own 393 对 301 毫秒）：

```
awk '/name=cell_verdict/{for(i=1;i<=NF;i++){split($i,a,"=");d[a[1]]=a[2]} n=split(d["q1_wait_microseconds"],v,";"); m=split(d["q1_judged_microseconds"],w,";"); k=d["arm"]" "d["column"]; for(j=1;j<=n;j++){x=v[j]+0; if(!(k in lo)||x<lo[k])lo[k]=x; if(!(k in hi)||x>hi[k])hi[k]=x} for(j=1;j<=m;j++){y=w[j]+0; if(!(k in jlo)||y<jlo[k])jlo[k]=y; if(!(k in jhi)||y>jhi[k])jhi[k]=y}} END{for(k in lo)print k, "wait_us", lo[k], hi[k], "judged_us", jlo[k], jhi[k]}' $J2 | sort
poll first wait_us 2 13 judged_us 2 13
poll later wait_us 6 19 judged_us 6 19
wait first wait_us 306115 682744 judged_us 122617 508382
wait later wait_us 418499 691320 judged_us 225340 497541
wait_own first wait_us 300820 532027 judged_us 100692 317773
wait_own later wait_us 393284 638682 judged_us 207968 417178
```

wait_own 的区间整体略低于 wait（first 栏上界 532 毫秒对 683 毫秒），但两条臂的 5 轮值没有按轮配对比过，页上不下「Some 比 None 省」的结论。产物里 `child_done` 60 个、`child_failed` 0 个、`lease` 行 180 条：

```
grep -c 'name=child_done' $S2
60
grep -c 'name=child_failed' $S2
0
grep -c 'name=lease ' $S2
180
```

**判别力自证**：真数据上 4 栏判定翻面，模型卡产物另证翻面（判的输出第 25 行 `discrimination_real_data_flipped_columns=4`、`discrimination_synthetic_flips=true`）。

````

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:341-356`（整段抄，未转述）**

```markdown
### 判决行的点名

四份产物的判决行（`grep -n 'name=verdict' $S1 $j1 $S2 $j2`）逐个点名没过的字段；字段名表示「通过」的取值（`true`、`zero`、0）不点名：

| 产物与行 | 字段 | 取值 | 为什么，是不是登记预期内的 |
|---|---|---|---|
| `$S1` 第 1698 行 | `cells_incomplete` | 2 | 5060 Ti、8 GiB、不等待臂的两格（在飞·中、在飞·重）补满 3 次仍 0 个有效轮，原因是 16 个子进程 Out of Memory（登记 V8，同一格 2 轮这样的就作废并交回原文）。V8 预期内，Out of Memory 本身登记没有预料到是哪一格 |
| `$S1` 第 1698 行 | `children_failed` | 16 | 5060 Ti、8 GiB、不等待臂那两格的 16 个子进程 Out of Memory，16 个 `child_failed` 行 |
| `$j1` 第 187 行 | `cells_void` | 2 | 5060 Ti、8 GiB、不等待臂那两格，判的输出里是 `name=cell_void` 两行 |
| `$j1` 第 187 行 | `poll_arm_also_absent_where_wait_arm_absent` | false | 登记 F5（不等待臂在每个等待臂判「不含」的格上也判「不含」）的反面：不等待臂在 24 栏含着，F5 没打中，登记预期内 |
| `$j1` 第 187 行 | `geometry_same_as_main_grid` | false | 登记 F8，不等待臂四对不同、等待臂八对相同，「并存的长借据」一段两种读法并列，哪种承重交主 agent |
| `$j2` 第 25 行 | `poll_arm_also_absent_where_wait_arm_absent` | false | 同 F5 的反面，poll 在 S = 2048 MiB 的 4 栏含着，登记预期内 |
| `$j2` 第 25 行 | `hold_control_holds`、`long_window_clear_within_window`、`geometry_same_as_main_grid` | not_run | 第二段只有 `shared_device` 一个组，没有扣住不丢、长窗、并存的长借据三个组，登记第五节第二段只写了共用设备一形，预期内 |

别的判决字段：`$S2` 第 242 行 `cells_incomplete=0 children_failed=0 children_void=0`；`$j1` 与 `$j2` 的 `wait_arm_next_reading_absent_everywhere=true`、`wait_arm_end_of_window_not_kept_everywhere=true`、`wait_always_returns=true`、`in_flight_cells_void_for_work_not_in_flight=0`、`done_wait_expensive_candidate_columns=0`、`discrimination_synthetic_flips=true`；`$j1` 的 `hold_control_holds=true`、`long_window_clear_within_window=true`。`discrimination_real_data_flipped_columns` 是 40（第一段）与 4（第二段），是判别力自证的栏数、不是「没过」。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:357-364`（整段抄，未转述）**

```markdown
### 单测与变异

续跑键（装置代码键）的样本与复跑（2026-10-15）：`e174-lease-drop-wait-memory-return-resume-key-samples-2026-10-15.out` 判决行 `key_samples=8 key_samples_holding=8 f_k1_missed=0 f_k2_extra=0 unit_test_samples=2`；同日整表变异 `e174-lease-drop-wait-memory-return-mutate-2026-10-15.log` 收尾「计数：内存撞顶 0 条（上限 6G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条、现判 34 条、复用 0 条、拒算 0 条」「已还原，基线仍全绿」。

单测 19 个：`grep -c '^test .* \.\.\. ok$' e174-lease-drop-wait-memory-return-unit-tests-2026-10-08-r2.out` 输出 19，同一份文件里 `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`。变异表 `scripts/mutation/tables/e174_lease_drop_wait_memory_return.tsv` 28 行（`grep -c '' scripts/mutation/tables/e174_lease_drop_wait_memory_return.tsv` 输出 28），`mutate.sh` 输出 `e174-lease-drop-wait-memory-return-mutate-2026-10-08-r2.log`：`grep -c '^✅' …` 输出 28，`grep -c '^⏭' …` 与 `grep -c '^❌' …` 都输出 0；第 30、31 行 `计数：内存撞顶 0 条（上限 6G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条` 与 `已还原，基线仍全绿`。单测与变异各有一份更早的输出（`…-unit-tests-2026-10-08.out`、`…-mutate-2026-10-08.log`，同样 19 个、28 条全抓），对的是之后又改了 `run` 续跑语义（已有 `name=done` 行但要补轮或换一组格时接着跑并追加新的 verdict 与 done）之前的源码，留着，正文引的数以 r2 为准。

变异表的行（页内编号）：M1–M7 是登记第九节的七条（M1 等待臂的 `Wait` 换成 `Poll`、M2 拆缓冲张数向上取整换成向下取整、M3 把 BAR1 下的 `Used` 当成进程占着的、M4 建缓冲时每张取一半、M5 扣住不丢组在第 6 步照样丢缓冲、M6 t0 记在 poll 开始之前、M7 在飞档第 4 步也等完）；M8–M28 是执行员补的二十一条，改判定一侧的门槛与比较（M8–M13、M25）、读数解析与分配器比较（M15–M17）、外层计时与 K7、K9、扣住不丢组的窗口比较（M18、M22–M24）、续跑与清单（M14、M19–M21、M26–M28）。登记第九节说"判定脚本那几处门槛的变异由第八节的判别力自证代替"，这里另补了变异，不算替代。M5 登记写的取样点（"等待臂某一格 R_next 已经 ≤ τ 的 S"）在模型卡上就有（等待臂 2048 MiB 的 R_next 为 0），变异被抓，真卡上有没有这样的格要看产物。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:365-377`（整段抄，未转述）**

```markdown
### 岔路表

登记第一节的岔路单只有 G2 一行，拆成乙的两个数；第二段把每个数再按「设备上只有这一笔借据的活」（第一段）与「同一设备几条线程共用」（第二段）分成两形。每一行的依据是「读数：第一段」「读数：第二段」两节里对应的命令输出。

| 岔路行 | 状态 | 依据 | 还差什么，剩下的量能不能让它翻面 |
|---|---|---|---|
| G2-乙-等待，设备上只有这一笔借据的活 | 已够判 | 第一段 Q1 的 `awk` 区间命令与 `q1_wait` 计数命令：已做完档 28 栏「零头」（最长 27 微秒），在飞档 34 栏「只等活本身」、13 栏「等得比活长」（超出最多 213.6 毫秒）、5 栏「不稳定」，0 栏「等不回来」 | 无。剩下的量只有在飞·轻，改变不了这一行 |
| G2-乙-读数，设备上只有这一笔借据的活 | 已够判 | 第一段 Q2a、Q2c、Q3 的计数命令：等待臂 80 栏全部「不含」、T_clear 为 0、窗口末不留；阳性对照 16 栏全含（F1 没打中）；不等待臂 24 栏含 | 不等待臂在 5060 Ti、8 GiB 没有读数（两格作废），不影响等待臂这一半；并存的长借据一组 F8 的两种读法交主 agent。剩下的量翻不了面 |
| G2-乙-等待，同一设备几条线程共用 | 已够判 | 第二段 Q1 的 `awk` 区间命令：wait 306–691 毫秒、wait_own 301–639 毫秒，16 栏「等得比活长」，0 栏等不回来 | 无 |
| G2-乙-读数，同一设备几条线程共用 | 已够判 | 第二段 `uniq -c` 计数命令：wait 与 wait_own 各 8 栏「不含」、T_clear 为 0、窗口末不留；poll 的 2048 MiB 4 栏含、0.7 到 1.15 秒后还 | 无 |
| 在飞·轻那一档（上一段岔路表里还差） | 够判后未跑 | 登记第九节 K9 在登记自己的数下必红（模型卡自检 `simulated_light_trips_k9_at_the_registered_numbers`，`research/results/e174-lease-drop-wait-memory-return-checks-2026-10-08.out` 里 `id=simulated_light_trips_k9_at_the_registered_numbers` 那一行）；修订 1 补的在飞·中在两段里 `in_flight_cells_void_for_work_not_in_flight=0`，没有一格因「活没在飞」作废 | 登记 F6 预计在飞·轻凑不够有效轮；G2-乙 的四行已经够判，剩下的量翻不了面，续不续由主 agent 定 |
| G2 甲、丙 | 不在这一次实验里 | 登记第一节：「甲、丙两格不在这一次实验里」 | 甲要真跑产品路径数少借次数，丙要在模型上跑补法并被攻一轮，都不是这个装置能量的 |

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:378-388`（整段抄，未转述）**

```markdown
### 它答不了的

- 两段只在登记的取样点上：S 五档（第二段只有 64 MiB 与 2 GiB）、在飞·中与在飞·重（第二段只有在飞·中）、借据连做 3 笔、窗口 2 秒（长窗 30 秒）。长窗那 4 栏只在 64 MiB、2 GiB 与在飞·中上做，没有更大的 S。
- 5060 Ti 上 8 GiB 的不等待臂两格作废：没有这一格的第一份读数，不等待臂在 8 GiB 上是否含、是否还，只有 5090 的 8192 MiB 那 4 栏有读数（含，`not_absent_within_window`）。Out of Memory 发生在哪一笔借据没有记下来，页上不下原因的结论。
- 第二段的线程 B 只提交 400 毫秒的活、不建缓冲；线程 B 的活更短、更长、或也建缓冲时的等待时长与读数没量。wait 与 wait_own 的 5 轮值没有按轮配对比过，页上只报了区间。
- 不等待臂在两段里表现不同（第一段 2048 MiB 的在飞格 2 秒里一直不还，第二段 0.7 到 1.15 秒后还回去），差别来自什么没有量。
- 没量的旋钮照登记：`MemoryHints` 只取 `Performance`，后端只取 Vulkan，驱动只取 595.84，一个进程里借据只连做 3 笔。
- 登记第四节自己推出的两条（gpu-allocator 的块大小、"被还在飞的提交用着的缓冲要等下一次 maintain"）是假设，装置没有按它们写任何门槛。
- B5（`-q` 与 `--query-compute-apps` 逐 MiB 相等）在两段里没有让任何子进程作废（两份 `run` 的 verdict 都是 `children_void=0`），本机驱动 595.84 把 Vulkan 进程列进了 compute-apps。
- 什么现象会推翻页上的读数：同一装置、同一驱动重跑时等待臂任一栏第一份读数含着缓冲，或不等待臂在 S 不小于 512 MiB、活在飞的格子里第一份读数不含；换驱动版本、换 `MemoryHints`、换后端，这些读数都不保证。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:389-394`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-23 不受影响：只改了产物名的写法，结论、数与产物不变；此前 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-18 不受影响：单测与变异一节补点名 10-15 续跑键样本与整表变异的产物，结论、数不变 |

```

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:1-32`（整段抄，未转述）**

````markdown
## E178 叶容器与记录死亡的计数 —— 已跑（四段）

**中文** · [English](178-counts-of-leaf-container-and-record-deaths.en.md) · [日本語](178-葉コンテナとレコードの死亡数.ja.md)

<!-- doc-lint:not-numbers X1 X2 X3 X4 H51 F1 F3 F8 F10 F11 F12 K1 K2 K3 K4 K5 K6 S1 S2 S3 S4 S5 S6 V1 V2 V3 V4 V5 V6 V7 V8 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 M15 M16 M17 M18 M19 M20 M21 M22 FL0 FL1 FL2 FL3 FL4 FL5 FL6 FL7 FL8 FL9 FL10 B1 B2 BN1 BN2 K-2 P-1 -->

```text
本页的本地臂名与记号写小写（a1、a2、a3、a4 ……）：跑前登记、装置、产物与命令里写作对应的大写，与全局登记的编号同形；kb 正文里改成小写只为与那些编号分开，读产物时按大写对。
```

跑前登记 `research/prompts/e178-preregistration.md`（第十二节「修订 1」「修订 3」由执行员在单测跑完、产物之前写；「修订 2」是实验设计员补的两条底座臂，作第四段）。回答的是岔路单 `m3-leaf-death-forks.md` 的四行：X1（容器身份消失的死亡在盘上怎么留：退役记录 B-on、零条目版本留在树里 B-stub、盘上零新增 B-tsz；X2–X4 都以它为前提）、X2（文件变短那种死亡在盘上怎么留：读法 Z 与区间墓碑 T）、X3（unlink 的整对象墓碑从哪一次发布起生效：意图那次 E-I 与最后一批 E-L）、X4（I-9.4（容器号不超最小 key） 判据与借怎么配：n1 抬分隔 key、n2 借时换身份）。登记分四段跑：第一段 X4，第二段 X2 与 X3 的漏活那一半，第三段 X3 的崩溃那一半，第四段 X1 的两个新候选。

装置是独立手写计数模型 `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs`，不与 `crates/` 共用代码、不依赖 `crates/`。四段各有一个入口（`segment-one` 至 `segment-four`），四段的产物都跑了（见「复跑」那张表）。

**现状**：

- 第一段跑完：H51（照 E177（逐树不写墓碑的复活数） 装置逐段重放的 51 段）与 F10（多对象带数据、借与吸收，28 段）在 n1 / n2 × 叶容器容量 {233, 16} × 合并门槛 θ {0, ⌊容量 ÷ 4⌋, ⌊容量 ÷ 2⌋} 下只跑写路径、不扫描，加登记 5.6 的坏镜像全表（44 形 × 两种键布局）。岔路单 X4 那一行够判。
- 每次借 n2 比 n1 多写码 3 一个、码 2 一个（退役容器与退役树根），多一条退役记录，四个 θ > 0 的格都如此；n2 多写占一段历史全部写单元的比例按格是 129–33003 ppm（容量 16、θ = 8 最大）。θ = 0 时两条写流逐发布相同。
- I-9.4（容器号不超最小 key） 的第三句换成「左最大 key < 右分隔 key」之后（n1 的写法），在 44 形里有 8 形「2026-10-08 的 I-9.4（容器号不超最小 key） 红而 n1 的绿」：K-2、S 族错号 2、9、10 配正确分隔 key 的三形、BN1 的 s = 25、30、31 与 s = 30 配右号 10；并上 I-9.12（分隔 key 落在孩子区间之外） 之后还剩 7 形（BN1-s31 被 I-9.12（分隔 key 落在孩子区间之外） 判红）。8 形里 BN1-s30 是登记写的 n1 借之后的正确形态，其余 7 形是号给错或分隔 key 抬错的坏镜像。两种键布局逐形同判。失败条款 FL8（n1 漏判）打中。
- 作废条款一条都没打中，停机对拍 S1–S3、S6 都对上。F10 在三个 θ > 0 的格里有一部分段一次借都没有（V6 只认这些段对 X4 不算结果，族不作废），见「失败、作废、停机条款逐条」。
- 修订 1：F11（回退）上登记写的 PC-B2 量不出 B-off-lin 的容器 m，补了一个直接构造的 PC-B2-direct，第九节 M3 的敏感取样点改由它承担；主 agent 定 F11 上的原 PC-B2 不当结果，Q-B2 的敏感性由 PC-B2-direct 承担。
- 第二段跑完（H51、F10、F11 × 12 格 = 996 项，20 个读者每次发布之后扫一次）：主格（B-on × Z / T × E-I / E-L-w / E-L-r × W ∈ {∞, 23}）在 48 个判定格（写路径格 × 批）上，底座 B-on 的容器复活与回退之后的容器误判死亡都是 0；Z 与 T 的复活、误判死亡（不在途中）、E-L 两臂下途中变短误判都是 0；E-L-w、E-L-r 的 unlink 途中漏活（inode、extent）都是 0，E-I 的途中漏活照定义是每次途中扫描 1 条记录与剩下的全部单元。T 每次多写的发布多写码 3、码 2 各一个，多写发布占比按族是 0（F1）到 766891 ppm（F8、E-L 两臂）。四个旋钮（W、θ、容量、批）26 行稳定性比对一处都没翻面。
- 第二段的作废与失败条款逐条见「失败、作废、停机条款逐条」：PC-B1 在 F11 的三段 n2（借缺口 θ = 4、θ = 8，被挪过的对象变短缺口 θ = 8）上不成立，按 V2 字面作废那三段（成因已由单测钉住：缺口里生、又在缺口里消失的身份，期末任何读者都读不到，PC-B1 的期望数没扣它）；PC-M 照登记字面（每个读者 extent m + 1、抹容器 inode m + 记录数、容器 m + 1）在 12 段数据单元、612 段叶容器上不成立（W = ∞ 下被抹那一版之下还有读得出的旧版），装置写的判法比登记松（≤），交主 agent 定；V8 照登记字面打中 E-I 读者在 F8、F3 途中扫描读出空的那几次；F11 有 4 段回退过远、24 段缺口事件没出现。这几样拿掉之后 48 个判定格的判定一格不变（全是 0，拿掉的段不会让 0 变正）。
- 第三段跑完（F12 的 12 个前缀、5448 个崩溃点）：崩溃点上 E-L-w、E-L-r 的漏活（inode m、extent m）都是 0，E-I 照定义漏 1 条记录与剩下的单元；读法 o2（重建视图里有记录、而意图已发布的对象）E-I 每个崩溃点 0，E-L-w、E-L-r 每个崩溃点恰好 1，FL6 照字面打中（这一项是 E-L 两臂定义的直接推论，见「失败、作废、停机条款逐条」）；恢复之后续做那一段三个名字空间的复活峰值都是 0；W 两档逐行同判。
- 第四段跑完（登记修订 2、修订 3；997 项：H51、F10、F11 × 12 格，加 PC-B2-direct 构造 1 项）：B-stub（零条目版本留在树里，只配 n1，问法一 n2-none）与 B-tsz（盘上零新增，供出规则 P-C，只配 E-I）的复活与误判死亡在全部判定格上都是 0，与 B-on 同格并排一格不差。B-stub 的 12 个读者 × 12 个 n1 判定格（Q-Bs-1、Q-Bs-2，含 F11 回退之后那一档）、B-tsz 的 4 个读者 × 24 个判定格（Q-Bt-1 的 inode r 与 extent r 差、Q-Bt-2）都是 0；B-tsz 的容器 r 按格是 1–166（定义的推论：死身份最后一版照读，a12 钉住，与 B-off-ret 逐次相等）。
- 两条新底座的阳性对照都成立：PC-Bs2（回退构造上 B-stub 的容器 m、inode m 是 0，关掉谱系那一项的 B-stub-lin-off 是 1、1）、PC-Bt3（B-tsz 的 inode m 0，墓碑不按谱系滤时 1）、PC-Bs1 962 项、PC-Bt1 1724 项、PC-Bt2-mid 224 项（其中 220 项手算值 > 0）两路一致、期望成立。原 PC-Bt2（期末那次扫描）224 项手算值都是 0，照修订 3 写明的那样量不出。
- 换了底座之后 X2、X3 的判定不变：Q-Bs-4（B-stub）384 行、Q-Bt-4（B-tsz）192 行，Z、T 的五个子问题在新底座与 B-on 下的判定一行都没变，FL-Bx 的名单是空的；B-stub 下 E-L-w、E-L-r 的 unlink 途中漏活（Q-Bs-3）是 0，E-I 照定义漏 1 条记录与剩下的单元（峰值 1、300），B-tsz 同（Q-Bt-3）。
- B-stub 的代价（Q-Bs-5、Q-Bs-6）：期末桩数最多 56（F1、F10，容量 16），叶片总数峰值最多 66（F3，容量 16），期末桩占比最高 1000000 ppm（F8，唯一那片叶删空变成桩）；有零条目版本、撞 I-9.13（类型 2 容器不空） 的版本占全部发布 1056–670710 ppm（按族、容量、θ、批）；叶片总数没有一版超过根装得下的 135 片。B-stub 写路径每次删空、吸收写多少单元，登记没登这个量，产物里没有。
- 作废与失败条款：配对（V-Bs1）、锚点（a10–a12）、读到的桩数（V-Bs3）、B-tsz 与 B-off-ret（V-Bt1）都没打中；V-Bt2：第四段 B-on、B-off-ret 的 43176 行轨迹与第二段同读者的行逐行相同。FL-Bs1、FL-Bt1、FL-B3、FL-Bs2、FL-Bx 都没打中。照字面打中、与第二段同样形态的有 V8（E-I 读者在 F3、F8 的 unlink 途中读出空，新底座 2448 条轨迹）与 PC-M 的登记字面（1128 项），都只碰只报数的格，见「失败、作废、停机条款逐条」。

确定性模型：每个工作项跑两遍逐项比，换线程数产物不变，说明的是没有隐藏状态，不是统计上稳定；证据强度靠钉绝对值的锚点、阳性对照、停机对拍与变异表。改过的装置复跑第一段与旧产物逐字节相同；第四段的 B-on、B-off-ret 轨迹与第二段产物逐行相同（V-Bt2）；第四段另抽 20 项（`--shard 0/50 --no-reuse`，都是 n1 的项）在单独的项目录里重算，与拼产物用的项文件逐字节相同。`replay.sh` 的第四段那一行没复跑（见「它答不了的」）。

````

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:33-44`（整段抄，未转述）**

```markdown
### 做了什么

- **写路径**：新池新建文件（txg 3，mkfs 那个进程的实例 1）、可写挂载的写行与暖机（实例 2），之后经会话的三种改动（写文件 = 对象内容整份换成 d 个新单元、建空 inode、unlink = 走空队列 + 意图 + 全部批）。叶容器新号只落最右叶、满了末尾分裂（右半号 = 触发那条记录的号、出生代 = 本次 txg）；删一条之后照 M(θ) 合并：删空不落盘、右边各片左移全部重写；|C| < θ 时有右邻就吸收（|C| + |R| ≤ 容量）或借 ⌈(|R| − |C|) ÷ 2⌉ 条，C 是最右一片就让左邻吸收。借时 n1 保留右片身份、分隔 key 抬到右片新的最小 key；n2 给右片换身份（容器号 = 借后最小 key、出生代 = 本次 txg），旧身份写退役记录。身份消失（删空、被吸收、n2 借）那次发布追加一条 34 字节退役记录，退役树一容器 959 条、有新记录那次重写开放容器与根。五条墓碑写流（Z / T × E-I / E-L-w / E-L-r 配出来的）各一棵墓碑树，第一段只数它们写的单元。
- **第一段的量**：每个工作项（一段历史 × 容量 × θ）n1、n2 各跑一遍写路径，逐发布配对：借的那次发布 n2 − n1 的码 3、码 2 与退役记录条数，别的发布要逐单元相同；每一版判 I-9.13（类型 2 容器不空）、2026-10-08 的 I-9.4（容器号不超最小 key）、n1 的 I-9.4（容器号不超最小 key）、I-9.12（分隔 key 落在孩子区间之外），这四个判定是第十一节 V7 的模型自查；借、吸收、删空次数与「被挪过的对象之后变短 / 被 unlink」两个次数（V6）。
- **5.6 全表**：K（`crates/` 现有坏镜像六形，逐形抄测试）、S（分裂时号给错，22 形）、BN1（n1 借之后，9 形）、BN2（n2 借之后，4 形）、DV（方向错，3 形），每形判2026-10-08 的 I-9.4（容器号不超最小 key）、n1 的 I-9.4（容器号不超最小 key）、I-9.12（分隔 key 落在孩子区间之外） 与路由（每个 key 取最后一个分隔 key ≤ 它的条目，落到的是不是装它的那一片）；第二种键布局把 key 乘 100、夹在两个 key 之间的号与分隔 key 取中点。
- **停机对拍**（登记第十一节，只读 `crates/` 源码与测试断言，在装置里写成同样的判定逐项比，没编译、没跑 `crates/`）：S1 末尾分裂（`crates/singlefs-core/src/inode_tree.rs:181-283`）、S2 删一条（`:286-318`）、S3 unlink 的步（`crates/singlefs-core/src/unlink.rs:118-130`）、S6 K-0 至 K-5 的判定（`crates/singlefs-checker/src/walk.rs:2768-2789`、`:3067`、`:3079`、`:3245-3266` 与 `crates/singlefs-verification/tests/checker_narrow_invariants_and_abandoned_roots.rs:936`、`:947`、`:962` 那几条断言）。S4、S5 也写好了，归第二段报。
- 装置在登记没写死的几处取了一个读法，写在装置文件头第 1–9 条（容器名字空间只数类型 2；X2 的判活用施行整对象墓碑之前的树；T 的区间规则只认区间起 > 0 的记录；回退那次发布不重写墓碑树、退役树的开放容器；每次用户改动先走空队列；F11「续按升序删」= 删号最小的活对象；F12 再建 5 个空 inode 是一次发布、P-m 前缀不做覆盖写规则；Q-X4-1 的「全部写单元」口径；植入的「写序最大」）。第一段只用到第 8 条。
- 挂钟：第一段 474 个工作项在 6 个线程上连编译约 29 秒（`time` 的 real 一行，没进产物）。
- **第二段**：每个工作项写路径跑两遍（第一遍定阳性对照的扫描、两遍逐次扫描比），第二遍每次发布之后 20 个读者各读一次（登记 5.4：候选 → 择新 → 底座 → 重建 inode 树与整对象墓碑 → 数据单元 → 与真值比），主路径按组索引择新，期末那次与全部阳性对照另用整扫的参照路径比。阳性对照 PC-B1、PC-B2（F11）、PC-B2-direct、PC-Z、PC-T、PC-G0、PC-EL、PC-ELr、PC-M。判定行按「写路径格 × 批」把 H51、F10、F11 的轨迹并起来判（`segment_two_verdict_lines`），回退之后那一档只并 F11。主 agent 起跑，`E178_THREADS=5`、内存包装 8G，约 21 分钟（草稿目录两个文件的修改时刻相减，没进产物）。
- **第三段**：F12 的前缀只模拟一次，意图那一版与每个不是最后一批的批上各分一支崩溃：下一步的单元全写上盘、根没写，崩溃那一刻 20 个读者各读一次（两条择新路径比），再换实例代号恢复、续做到队列空、建 5 个空 inode，每次发布之后扫一次。阳性对照 PC-O（植入、拿掉墓碑两项）与 PC-M。主 agent 起跑，`E178_THREADS=5`、8G，约 5 分钟。
- **第四段**：每个工作项（H51、F10、F11 × 12 格，加 PC-B2-direct 构造 1 项，共 997 项）原写路径跑两遍、18 个读者（B-on 12 个、B-tsz 4 个、B-off-ret 2 个）；n1 的格另跑 B-stub 写路径两遍、12 个读者，与原写路径逐版配对（V-Bs1），配对成立才算「差」（新臂减同一 (X2, X3, W) 的 B-on）。B-stub 写路径：身份消失（删空、被吸收）那次原位写一个零条目版本、之后一直留在叶序里不再重写，右边各片不左移，不写退役记录；桩不收新号、不当合并的邻片（装置文件头读法 10）。B-tsz 读者不丢任何版本，同号几条记录按改动计数供出（P-C，平局按写序），整对象墓碑照 E-I。每项算完原子落进项目录，同一条命令重起只补没落盘的项。主 agent 经 `detached-run.sh` 起执行员写的起法脚本：本机 14 线程跑 `--shard 0/2`（现判 499），第二台那一片被内存包装拒了（退 252，要 50G 而 slice 总上限 40.2 GiB，命令没跑），本机补算另一半（现判 498、复用 499），997 项齐了拼产物（现判 0、复用 997）；起跑日志 `~/.local/state/singlefs/detached-runs/e178-segment-four/log`。项文件的修改时刻从第一个到最后一个约 15 分钟（`ls -l` 相减，没进产物）。

```

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:45-101`（整段抄，未转述）**

````markdown
### 复跑

```
# 仓根；模型在 scripts/experiment-apparatus/（不依赖 crates/），线程数读 E178_THREADS（默认 6），只影响挂钟、不影响输出
cd /home/fy5090/code/singlefs/scripts/experiment-apparatus
../../task-server/task submit -- env E178_THREADS=6 cargo run -q --release --bin e178-leaf-and-record-death-counts -- segment-one
../../task-server/task submit -- env E178_THREADS=5 cargo run -q --release --bin e178-leaf-and-record-death-counts -- segment-two
../../task-server/task submit -- env E178_THREADS=5 cargo run -q --release --bin e178-leaf-and-record-death-counts -- segment-three
cd ../..
# 第四段：工作项读写 prune-server 里的那一份判定库（项键相同的项读回不重算），一律经 scripts/experiment-apparatus/device-code-key.py run 起、由它给 --code-key-file；补算缺的项、997 项齐了拼产物；要两台分判由统一任务服务分片
task-server/task submit -- env E178_THREADS=6 python3 scripts/experiment-apparatus/device-code-key.py run . --binary e178-leaf-and-record-death-counts --entry e7_index_bench::segment_four_item_lines -- segment-four
# 登记的复跑（scripts/experiment-apparatus/replay.sh 四行：segment-one、segment-two、segment-three 与 @driver_e178_segment_four，后者经 device-code-key.py run 起 segment-four，库里键相同的项复用、不同的现判）
bash scripts/experiment-apparatus/replay.sh E178
# 单测 44 个与变异表 22 条
cd scripts/experiment-apparatus
../../task-server/task submit -- cargo test --release --bin e178-leaf-and-record-death-counts
../../task-server/task submit -- bash ../mutation/mutate.sh e178-leaf-and-record-death-counts e178_leaf_and_record_death_counts.rs ../mutation/tables/e178_leaf_and_record_death_counts.tsv
```

第四段的整套起法（本机与第二台各跑一片、取回项文件、补算、数满 997 项再拼）写在执行员草稿目录的 `run-segment-four.sh`（`detached-run.sh long-run-check` 判过长活声明齐了），由主 agent 起；产物 `research/results/e178-leaf-and-record-death-counts-segment-four.out`。

| 文件 | 是什么 |
|---|---|
| `research/results/e178-leaf-and-record-death-counts-segment-one.out` | 第一段：config、常量锚点四行、停机对拍四行、5.6 全表 88 行、948 行写路径、474 行借的配对、断言总计与锚点、两行阳性对照、98 行判定（1626 行，sha256 `9644a139c0c01402a18918f232e6fbafcfc6caa96d68578441e66ccc30f3a8ea`） |
| `e178-leaf-and-record-death-counts-unit-tests.out` | 第一段时的单测输出（32 个，含编译的行），装置改过之后由 `…-unit-tests-2026-10-08.out` 取代 |
| `e178-leaf-and-record-death-counts-mutations.out` | 第一段时 `mutate.sh` 整张表（15 条）的输出（stderr 头一行在前），由 `…-mutations-2026-10-08.out` 取代 |
| `e178-leaf-and-record-death-counts-unit-tests-2026-10-08.out` | 加了第四段之后的单测输出（44 个，含编译的行；sha256 `cc3519224f26f975e9a36f37492f7bb87f88fb2710e599754a017b038e280368`） |
| `e178-leaf-and-record-death-counts-mutations-2026-10-08.out` | 加了第四段之后 `mutate.sh` 整张表（22 条）的 stdout（sha256 `60c198835f92e6bb0314c5aa16e67803ff90055070edc2d0d2ccf415e9e1b536`） |
| `research/results/e178-leaf-and-record-death-counts-segment-two.out` | 第二段（73284 行，末行 `E7RESULT name=done emitted=73284`，sha256 `e73d4aa979f0f683ae7a46f35009926631365cd8be1f3f04640c308a5e052bab`） |
| `research/results/e178-leaf-and-record-death-counts-segment-three.out` | 第三段（4335 行，末行 `E7RESULT name=done emitted=4335`，sha256 `30565bfcba92fce5ba3be3a6bae3385eea82a89da97968caa7c284bbf7978fc3`） |
| `research/results/e178-leaf-and-record-death-counts-segment-four.out` | 第四段（140179 行，末行 `E7RESULT name=done emitted=140179`，sha256 `17c292f968a235134376444fc9aa20833bf55c68245e7c43c92b004dc954668e`；装置源文件 sha256 `93193bd5929e7867b7d18ea5c8b0d6af9e32ceffed5c866f073ba8e5c0d1dfa6`，项文件第一行的键里同一个值） |

产物里各类行的条数（`awk '{print $2}' research/results/e178-leaf-and-record-death-counts-segment-one.out | sort | uniq -c`）：

```
      6 name=anchor
      4 name=assertion
    474 name=borrow_cost
      1 name=config
      1 name=done
      2 name=positive_control
     88 name=shape
      4 name=stop_check
     98 name=verdict
    948 name=write_path
```

948 = 79 段 × 12 个写路径格。按族、N、容量、θ 分组数，每组的行数等于那一族的段数（F1 14、F10 28、F8 32、F3 5），12 个格一个不缺：

```
grep 'name=write_path' research/results/e178-leaf-and-record-death-counts-segment-one.out | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]} print v["family"], v["borrow_arm"], v["capacity"], v["theta"]}' | sort | uniq -c | awk '{print $1}' | sort | uniq -c
     12 14
     12 28
     12 32
     12 5
```

````

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:102-120`（整段抄，未转述）**

````markdown
### 判决（整行抄自 `research/results/e178-leaf-and-record-death-counts-segment-one.out`）

Q-X4-1、Q-X4-2（第 1528–1569 行里按格的两行；θ = 0 的两格借的次数是 0、两条写流逐发布相同）：

```
E7RESULT name=verdict question=Q-X4-1 capacity=233 theta=0 histories=79 borrow_publishes=0 extra_code3_per_borrow_count=0 extra_code3_per_borrow_smallest=none extra_code3_per_borrow_largest=none extra_code3_per_borrow_total=0 extra_code2_per_borrow_count=0 extra_code2_per_borrow_smallest=none extra_code2_per_borrow_largest=none extra_code2_per_borrow_total=0 n2_extra_share_ppm_over_histories_with_borrows_count=0 n2_extra_share_ppm_over_histories_with_borrows_smallest=none n2_extra_share_ppm_over_histories_with_borrows_largest=none n2_extra_share_ppm_over_histories_with_borrows_total=0 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=233 theta=0 n2_retirements_per_borrow_count=0 n2_retirements_per_borrow_smallest=none n2_retirements_per_borrow_largest=none n2_retirements_per_borrow_total=0 n1_retirements_per_borrow_count=0 n1_retirements_per_borrow_smallest=none n1_retirements_per_borrow_largest=none n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=233 theta=58 histories=79 borrow_publishes=12 extra_code3_per_borrow_count=12 extra_code3_per_borrow_smallest=1 extra_code3_per_borrow_largest=1 extra_code3_per_borrow_total=12 extra_code2_per_borrow_count=12 extra_code2_per_borrow_smallest=1 extra_code2_per_borrow_largest=1 extra_code2_per_borrow_total=12 n2_extra_share_ppm_over_histories_with_borrows_count=6 n2_extra_share_ppm_over_histories_with_borrows_smallest=147 n2_extra_share_ppm_over_histories_with_borrows_largest=1100 n2_extra_share_ppm_over_histories_with_borrows_total=2805 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=233 theta=58 n2_retirements_per_borrow_count=12 n2_retirements_per_borrow_smallest=1 n2_retirements_per_borrow_largest=1 n2_retirements_per_borrow_total=12 n1_retirements_per_borrow_count=12 n1_retirements_per_borrow_smallest=0 n1_retirements_per_borrow_largest=0 n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=233 theta=116 histories=79 borrow_publishes=102 extra_code3_per_borrow_count=102 extra_code3_per_borrow_smallest=1 extra_code3_per_borrow_largest=1 extra_code3_per_borrow_total=102 extra_code2_per_borrow_count=102 extra_code2_per_borrow_smallest=1 extra_code2_per_borrow_largest=1 extra_code2_per_borrow_total=102 n2_extra_share_ppm_over_histories_with_borrows_count=21 n2_extra_share_ppm_over_histories_with_borrows_smallest=146 n2_extra_share_ppm_over_histories_with_borrows_largest=5464 n2_extra_share_ppm_over_histories_with_borrows_total=23828 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=233 theta=116 n2_retirements_per_borrow_count=102 n2_retirements_per_borrow_smallest=1 n2_retirements_per_borrow_largest=1 n2_retirements_per_borrow_total=102 n1_retirements_per_borrow_count=102 n1_retirements_per_borrow_smallest=0 n1_retirements_per_borrow_largest=0 n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=16 theta=0 histories=79 borrow_publishes=0 extra_code3_per_borrow_count=0 extra_code3_per_borrow_smallest=none extra_code3_per_borrow_largest=none extra_code3_per_borrow_total=0 extra_code2_per_borrow_count=0 extra_code2_per_borrow_smallest=none extra_code2_per_borrow_largest=none extra_code2_per_borrow_total=0 n2_extra_share_ppm_over_histories_with_borrows_count=0 n2_extra_share_ppm_over_histories_with_borrows_smallest=none n2_extra_share_ppm_over_histories_with_borrows_largest=none n2_extra_share_ppm_over_histories_with_borrows_total=0 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=16 theta=0 n2_retirements_per_borrow_count=0 n2_retirements_per_borrow_smallest=none n2_retirements_per_borrow_largest=none n2_retirements_per_borrow_total=0 n1_retirements_per_borrow_count=0 n1_retirements_per_borrow_smallest=none n1_retirements_per_borrow_largest=none n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=16 theta=4 histories=79 borrow_publishes=222 extra_code3_per_borrow_count=222 extra_code3_per_borrow_smallest=1 extra_code3_per_borrow_largest=1 extra_code3_per_borrow_total=222 extra_code2_per_borrow_count=222 extra_code2_per_borrow_smallest=1 extra_code2_per_borrow_largest=1 extra_code2_per_borrow_total=222 n2_extra_share_ppm_over_histories_with_borrows_count=12 n2_extra_share_ppm_over_histories_with_borrows_smallest=137 n2_extra_share_ppm_over_histories_with_borrows_largest=16448 n2_extra_share_ppm_over_histories_with_borrows_total=64634 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=16 theta=4 n2_retirements_per_borrow_count=222 n2_retirements_per_borrow_smallest=1 n2_retirements_per_borrow_largest=1 n2_retirements_per_borrow_total=222 n1_retirements_per_borrow_count=222 n1_retirements_per_borrow_smallest=0 n1_retirements_per_borrow_largest=0 n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=16 theta=8 histories=79 borrow_publishes=1505 extra_code3_per_borrow_count=1505 extra_code3_per_borrow_smallest=1 extra_code3_per_borrow_largest=1 extra_code3_per_borrow_total=1505 extra_code2_per_borrow_count=1505 extra_code2_per_borrow_smallest=1 extra_code2_per_borrow_largest=1 extra_code2_per_borrow_total=1505 n2_extra_share_ppm_over_histories_with_borrows_count=45 n2_extra_share_ppm_over_histories_with_borrows_smallest=129 n2_extra_share_ppm_over_histories_with_borrows_largest=33003 n2_extra_share_ppm_over_histories_with_borrows_total=466033 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=16 theta=8 n2_retirements_per_borrow_count=1505 n2_retirements_per_borrow_smallest=1 n2_retirements_per_borrow_largest=1 n2_retirements_per_borrow_total=1505 n1_retirements_per_borrow_count=1505 n1_retirements_per_borrow_smallest=0 n1_retirements_per_borrow_largest=0 n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
```

````

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:121-161`（整段抄，未转述）**

````markdown
### 读数：每次借的多写与退役记录（Q-X4-1、Q-X4-2、Q-X4-3）

| 容量 | θ | 借的发布数（79 段合计） | 每次借 n2 − n1 码 3 / 码 2 | 每次借退役记录 n2 / n1 | n2 多写占全部写单元（只数有借的段，ppm） |
|---|---|---|---|---|---|
| 233 | 0 | 0 | — | — | — |
| 233 | 58 | 12 | 1 / 1（12 次全是） | 1 / 0 | 147–1100（6 段） |
| 233 | 116 | 102 | 1 / 1（102 次全是） | 1 / 0 | 146–5464（21 段） |
| 16 | 0 | 0 | — | — | — |
| 16 | 4 | 222 | 1 / 1（222 次全是） | 1 / 0 | 137–16448（12 段） |
| 16 | 8 | 1505 | 1 / 1（1505 次全是） | 1 / 0 | 129–33003（45 段） |

这张表的数取自判决那一节抄的十二行。借之外的发布 n2 与 n1 逐单元相同（`differences_outside_borrows=0`），两臂的借、吸收、删空次数逐段相同（Q-X4-3 每一行 `merge_counts_differ_between_n1_n2=0`）。「每次借多写 1 + 1、退役记录 1 条」是臂定义的推论（登记第四节第 5 条），由第七节 a6 的一借小历史钉绝对值（锚点那一节）；它不是门槛，这几格只报数。比例的分母照读法 8：数据单元 + 叶容器 + inode 树根 + 退役容器 + 退役树根 + 2026-10-08 那条墓碑写流的墓碑容器与墓碑树根。

Q-X4-3 里 F10 那六行（整行抄）：

```
E7RESULT name=verdict question=Q-X4-3 capacity=233 theta=0 family=F10 segments=28 borrows=0 absorbs=0 drops=56 moved_then_shortened=0 moved_then_unlinked=0 publishes=68181 n2_retirement_records=56 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=0 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=233 theta=58 family=F10 segments=28 borrows=8 absorbs=56 drops=0 moved_then_shortened=718 moved_then_unlinked=4428 publishes=68112 n2_retirement_records=64 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=24 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=233 theta=116 family=F10 segments=28 borrows=68 absorbs=56 drops=0 moved_then_shortened=990 moved_then_unlinked=5712 publishes=68099 n2_retirement_records=124 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=14 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=16 theta=0 family=F10 segments=28 borrows=0 absorbs=0 drops=1036 moved_then_shortened=0 moved_then_unlinked=0 publishes=68181 n2_retirement_records=1036 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=0 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=16 theta=4 family=F10 segments=28 borrows=148 absorbs=1036 drops=0 moved_then_shortened=894 moved_then_unlinked=5904 publishes=68185 n2_retirement_records=1184 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=20 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=16 theta=8 family=F10 segments=28 borrows=1000 absorbs=1036 drops=0 moved_then_shortened=1170 moved_then_unlinked=8026 publishes=68189 n2_retirement_records=2036 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=0 judgement=reported
```

V7（模型写路径自查，整行抄；n1 的版本按2026-10-08 的 I-9.4（容器号不超最小 key） 红是定义的推论，只报版本数）：

```
E7RESULT name=verdict question=V7 capacity=233 theta=0 borrow_arm=N1 versions=88184 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=0 borrow_arm=N2 versions=88184 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=58 borrow_arm=N1 versions=88115 i913_red=0 today_i94_red=2956 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=58 borrow_arm=N2 versions=88115 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=116 borrow_arm=N1 versions=88102 i913_red=0 today_i94_red=9190 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=116 borrow_arm=N2 versions=88102 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=0 borrow_arm=N1 versions=88184 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=0 borrow_arm=N2 versions=88184 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=4 borrow_arm=N1 versions=88188 i913_red=0 today_i94_red=5870 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=4 borrow_arm=N2 versions=88188 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=8 borrow_arm=N1 versions=88192 i913_red=0 today_i94_red=36890 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=8 borrow_arm=N2 versions=88192 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
```

````

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:162-197`（整段抄，未转述）**

````markdown
### 读数：n1 在现有与新补坏镜像上判得出、判不出的（Q-X4-4 至 Q-X4-7）

全表 88 行在产物第 10–97 行（`grep 'name=shape ' research/results/e178-leaf-and-record-death-counts-segment-one.out`），每形一行写三个谓词的红绿与路由。汇总（整行抄，密排布局 `dense` 与稀排布局 `sparse` 各三行，再加 G-K 一行）：

```
E7RESULT name=verdict question=Q-X4-4 layout=dense family=all shapes=44 today_red_n1_green=8 names=K-2,S-m2-separator_twenty,S-m9-separator_twenty,S-m10-separator_twenty,BN1-s25,BN1-s30,BN1-s31,BN1-s30-m10 n1_misses_shapes=true
E7RESULT name=verdict question=Q-X4-5 layout=dense family=all shapes=44 today_or_i912_red_n1_and_i912_green=7 names=K-2,S-m2-separator_twenty,S-m9-separator_twenty,S-m10-separator_twenty,BN1-s25,BN1-s30,BN1-s30-m10 i912_does_not_cover_the_misses=true
E7RESULT name=verdict question=Q-X4-6 layout=dense family=all misses_with_wrong_routing=1 names=BN1-s31 n1_misses_hurt_lookup=true
E7RESULT name=verdict question=Q-X4-4 layout=sparse family=all shapes=44 today_red_n1_green=8 names=K-2,S-m2-separator_twenty,S-m9-separator_twenty,S-m10-separator_twenty,BN1-s25,BN1-s30,BN1-s31,BN1-s30-m10 n1_misses_shapes=true
E7RESULT name=verdict question=Q-X4-5 layout=sparse family=all shapes=44 today_or_i912_red_n1_and_i912_green=7 names=K-2,S-m2-separator_twenty,S-m9-separator_twenty,S-m10-separator_twenty,BN1-s25,BN1-s30,BN1-s30-m10 i912_does_not_cover_the_misses=true
E7RESULT name=verdict question=Q-X4-6 layout=sparse family=all misses_with_wrong_routing=1 names=BN1-s31 n1_misses_hurt_lookup=true
E7RESULT name=verdict question=G-K shapes=44 verdicts_differ_between_layouts=0 names=none stable=true
```

「2026-10-08 的 I-9.4（容器号不超最小 key） 红而 n1 的 I-9.4（容器号不超最小 key） 绿」那 8 形与三个合法对照（整行抄，密排布局）：

```
E7RESULT name=shape family=K shape=K-2 layout=dense leaves=1:1/10,8:20/30 separators=1,15 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=S shape=S-m2-separator_twenty layout=dense leaves=1:1/10,2:20/30 separators=1,20 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=S shape=S-m9-separator_twenty layout=dense leaves=1:1/10,9:20/30 separators=1,20 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=S shape=S-m10-separator_twenty layout=dense leaves=1:1/10,10:20/30 separators=1,20 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=S shape=S-m20-separator_twenty layout=dense leaves=1:1/10,20:20/30 separators=1,20 today_i94=green n1_i94=green i912=green routing=right
E7RESULT name=shape family=BN1 shape=BN1-s25 layout=dense leaves=1:1/10/20,20:30 separators=1,25 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=BN1 shape=BN1-s30 layout=dense leaves=1:1/10/20,20:30 separators=1,30 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=BN1 shape=BN1-s31 layout=dense leaves=1:1/10/20,20:30 separators=1,31 today_i94=red n1_i94=green i912=red routing=wrong
E7RESULT name=shape family=BN1 shape=BN1-s30-m10 layout=dense leaves=1:1/10/20,10:30 separators=1,30 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=BN2 shape=BN2-m30 layout=dense leaves=1:1/10/20,30:30 separators=1,30 today_i94=green n1_i94=green i912=green routing=right
```

- K-2（右号 8、分隔 key 15）：跑的那天（2026-10-08）的 I-9.4（容器号不超最小 key） 只有第三句说话（`crates/singlefs-verification/tests/checker_narrow_invariants_and_abandoned_roots.rs` 里「第二片的容器号 8」那条用例的注释），n1 把第三句换成比分隔 key 之后绿。登记第四节第 4 条推的那一格（「n1 之后这一形不红」）对上了；它对上不算独立印证（登记原话）。
- S 族错号 2、9、10 配正确的分隔 key 20：分裂时号给错、分隔 key 对，n1 的四句与 I-9.12（分隔 key 落在孩子区间之外） 都绿，路由也对。同一个错号配「分隔 key = 号」（E178（叶容器与记录死亡的计数） 跑的时候 `crates/singlefs-core/src/inode_tree.rs` 里分隔 key 直接取容器号的写法；现行写法是 `separator_key_in_the_parent_entry` 字段，函数 `separator_key`）时，n1 的 I-9.4（容器号不超最小 key） 与 I-9.12（分隔 key 落在孩子区间之外） 都判红、路由错（产物里 `shape=S-m2-separator_equals_number` 等三行）。
- BN1-s25（分隔 key 抬了不到）、BN1-s30-m10（右号 10）：n1 与 I-9.12（分隔 key 落在孩子区间之外） 都绿、路由对。BN1-s31（抬过头）n1 绿、I-9.12（分隔 key 落在孩子区间之外） 红、路由错，是 Q-X4-6 里唯一一形。
- BN1-s30 是登记 5.6 写的 n1 借之后的正确形态（「30（正确）」），2026-10-08 的 I-9.4（容器号不超最小 key） 红是 n1 定义的推论，它在 Q-X4-4、Q-X4-5 的名单里是照判据字面数进去的；不算它，Q-X4-4 还有 7 形、Q-X4-5 还有 6 形，FL8 照样打中。
- BN2 四形、DV 三形没有一形「2026-10-08 的 I-9.4（容器号不超最小 key） 红而 n1 绿」；K-1、K-3、K-4、K-5 在 2026-10-08 的 I-9.4（容器号不超最小 key） 与 n1 下一起红。
- 按族分的判决行（产物第 1578–1606 行）里写 false 的是这几族没打中的那一格：BN2、DV 两族 `n1_misses_shapes=false`（Q-X4-4）、`i912_does_not_cover_the_misses=false`（Q-X4-5）、`n1_misses_hurt_lookup=false`（Q-X4-6）；K、S 两族在两种布局上 `n1_misses_hurt_lookup=false`（Q-X4-6）。BN2、DV 两族 n1 没有漏判的形；K、S 两族有漏判的形，但漏判不伤路由。

````

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:198-218`（整段抄，未转述）**

````markdown
### 锚点、停机对拍与阳性对照（整行抄自产物第 2–9 行、第 1520–1527 行）

```
E7RESULT name=anchor id=K1 inode_leaf_records=233 expected=233 holds=true
E7RESULT name=anchor id=K2 tombstone_container_records=815 expected=815 holds=true
E7RESULT name=anchor id=K3 root_node_leaf_containers=135 expected=135 holds=true
E7RESULT name=anchor id=A2 retirement_record_bytes=34 retirement_container_records=959 expected=34,959 holds=true
E7RESULT name=stop_check id=S1 holds=true detail=capacity_16=true,capacity_233=true
E7RESULT name=stop_check id=S2 holds=true detail=partial=Some((UnitsWritten { data_units: 0, inode_leaf_containers: 1, inode_roots: 1, retirement_containers: 0, retirement_roots: 0, retirement_records: 0, tombstone_containers: [0, 1, 0, 0, 1], tombstone_roots: [0, 1, 0, 0, 1] }, NoMerge)) emptied=Some((UnitsWritten { data_units: 0, inode_leaf_containers: 1, inode_roots: 1, retirement_containers: 1, retirement_roots: 1, retirement_records: 1, tombstone_containers: [0, 1, 0, 0, 1], tombstone_roots: [0, 1, 0, 0, 1] }, EmptiedAndDropped)) leaves_after_emptying=[1, 33] last_record=Some((UnitsWritten { data_units: 0, inode_leaf_containers: 0, inode_roots: 1, retirement_containers: 1, retirement_roots: 1, retirement_records: 1, tombstone_containers: [0, 1, 0, 0, 1], tombstone_roots: [0, 1, 0, 0, 1] }, EmptiedAndDropped))
E7RESULT name=stop_check id=S3 holds=true detail=steps_hold=true size_at_the_intent=Some(9790200) after_the_first_batch=Some(InodeRecord { inode: 1, object_birth: 3, nlink: 0, size_bytes: 5090904 })
E7RESULT name=stop_check id=S6 holds=true detail=K-0=true,K-1=true,K-2=true,K-3=true,K-4=true,K-5=true
E7RESULT name=assertion assertion=A1_history_publishes judged=612 violated=0
E7RESULT name=assertion assertion=A1_history_refused_steps judged=612 violated=0
E7RESULT name=assertion assertion=A8_retirements_equal_identity_disappearances judged=184 violated=0
E7RESULT name=assertion assertion=K6_split_right_half_numbered_after_the_record_left_half_unchanged judged=12654 violated=0
E7RESULT name=anchor id=A5 zero_threshold_streams_identical=true holds=true
E7RESULT name=anchor id=A6 borrows=1 other_merges=0 merge=BorrowedFromTheRightNeighbour { records_moved: 5 } sizes=[12, 11] right_smallest=Some(22) n2_right_identity_number=Some(22) n1_right_separator=Some(22) n1_right_number=Some(17) extra_code3=1 extra_code2=1 extra_retirements=1 holds=true
E7RESULT name=positive_control id=PC-N zero_borrow_histories_identical=true single_borrow_history_holds=true holds=true
E7RESULT name=positive_control id=PC-K today_i94_reddens_k1_to_k5=true k0_all_green=true n1_i94_reddens_bn1_s20=true holds=true
```

````

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:219-317`（整段抄，未转述）**

````markdown
### 读数：第二段（底座 B-on、X2、X3 的漏活那一半）

产物里各类行的条数（`awk '{print $2}' research/results/e178-leaf-and-record-death-counts-segment-two.out | sort | uniq -c`）：

```
    196 name=anchor
     12 name=assertion
      1 name=config
      1 name=done
   2988 name=extra_writes
   5437 name=positive_control
    996 name=simulation
     26 name=stability
      2 name=stop_check
  61680 name=trajectory
   1945 name=verdict
```

996 = 83 段 × 12 格；61680 = 948 项 × 20 个读者 × 3 类扫描 + F11 的 48 项 × 20 × 5 类。判决行按问题与状态数（`grep 'name=verdict' … | awk` 取 `question=` 与 `status=` / `judgement=`、`fl1_cell_excluded=`）：

```
     48 question=Q-B1 status=zero
     24 question=Q-B2 status=not_run
     24 question=Q-B2 status=zero
    144 question=Q-X2-1 status=zero fl1_cell_excluded=false
    144 question=Q-X2-2 status=zero fl1_cell_excluded=false
    144 question=Q-X2-3 status=zero fl1_cell_excluded=false
    144 question=Q-X2-4 status=zero fl1_cell_excluded=false
    192 question=Q-X2-5 status=zero fl1_cell_excluded=false
     36 question=Q-X2-6 judgement=reported
    180 question=Q-X2-7 judgement=reported
     96 question=Q-X3-1 status=reported fl1_cell_excluded=false
    192 question=Q-X3-1 status=zero fl1_cell_excluded=false
     96 question=Q-X3-2 status=reported fl1_cell_excluded=false
    192 question=Q-X3-2 status=zero fl1_cell_excluded=false
    288 question=Q-X3-3 status=incidental
      1 question=summary
```

Q-B2 的 24 行 `not_run` 是批 = 1 的判定格（F11 只有批 144）。`reported` 的 96 + 96 行都是 E-I 臂（途中漏活是 E-I 定义的推论，只报数、钉 a7）。

整行抄（同一份产物；容量 16、θ = 8、n1、批 1 的四个 X2 主格，容量 233、θ = 0 的途中漏活，与 θ = 8 的多写）：

```
E7RESULT name=verdict question=Q-X2-1 borrow_arm=N1 capacity=16 theta=8 batch=1 base=B-on shrink=Z effect=E-I window=inf extent_r_peak=0 extent_r_peak_at=0 extent_r_positive_scans=0 extent_r_last=0 inode_r_peak=0 inode_r_peak_at=0 inode_r_positive_scans=0 inode_r_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X2-3 borrow_arm=N1 capacity=16 theta=8 batch=1 base=B-on shrink=Z effect=E-I window=inf extent_m_outside_unlink_peak=0 extent_m_outside_unlink_peak_at=0 extent_m_outside_unlink_positive_scans=0 extent_m_outside_unlink_last=0 inode_m_outside_unlink_peak=0 inode_m_outside_unlink_peak_at=0 inode_m_outside_unlink_positive_scans=0 inode_m_outside_unlink_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X2-2 borrow_arm=N1 capacity=16 theta=8 batch=1 base=B-on shrink=T effect=E-I window=inf extent_r_peak=0 extent_r_peak_at=0 extent_r_positive_scans=0 extent_r_last=0 inode_r_peak=0 inode_r_peak_at=0 inode_r_positive_scans=0 inode_r_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X2-4 borrow_arm=N1 capacity=16 theta=8 batch=1 base=B-on shrink=T effect=E-I window=inf extent_m_outside_unlink_peak=0 extent_m_outside_unlink_peak_at=0 extent_m_outside_unlink_positive_scans=0 extent_m_outside_unlink_last=0 inode_m_outside_unlink_peak=0 inode_m_outside_unlink_peak_at=0 inode_m_outside_unlink_positive_scans=0 inode_m_outside_unlink_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X3-1 borrow_arm=N1 capacity=233 theta=0 batch=1 base=B-on shrink=Z effect=E-I window=inf inside_unlink_peak=1 inside_unlink_peak_at=5 inside_unlink_positive_scans=24441 inside_unlink_last=1 status=reported fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X3-2 borrow_arm=N1 capacity=233 theta=0 batch=1 base=B-on shrink=Z effect=E-I window=inf inside_unlink_peak=300 inside_unlink_peak_at=5 inside_unlink_positive_scans=24441 inside_unlink_last=1 status=reported fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X3-1 borrow_arm=N1 capacity=233 theta=0 batch=1 base=B-on shrink=Z effect=E-L-w window=inf inside_unlink_peak=0 inside_unlink_peak_at=0 inside_unlink_positive_scans=0 inside_unlink_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X3-1 borrow_arm=N1 capacity=233 theta=0 batch=1 base=B-on shrink=Z effect=E-L-r window=inf inside_unlink_peak=0 inside_unlink_peak_at=0 inside_unlink_positive_scans=0 inside_unlink_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X2-6 borrow_arm=N1 capacity=16 theta=8 effect=E-I extra_code3_per_multi_write_publish_count=1753 extra_code3_per_multi_write_publish_smallest=1 extra_code3_per_multi_write_publish_largest=1 extra_code3_per_multi_write_publish_total=1753 extra_code2_per_multi_write_publish_count=1753 extra_code2_per_multi_write_publish_smallest=1 extra_code2_per_multi_write_publish_largest=1 extra_code2_per_multi_write_publish_total=1753 not_one_and_one=0 judgement=reported
E7RESULT name=verdict question=Q-X2-7 borrow_arm=N1 capacity=16 theta=8 family=F10 effect=E-I multi_write_publishes=1652 publishes=68189 share_ppm=24226 judgement=reported
E7RESULT name=verdict question=Q-X2-7 borrow_arm=N1 capacity=16 theta=8 family=F10 effect=E-L-w multi_write_publishes=16809 publishes=68189 share_ppm=246506 judgement=reported
E7RESULT name=verdict question=Q-B1 borrow_arm=N2 capacity=16 theta=8 batch=144 window=inf container_r_peak=0 container_r_peak_at=0 container_r_positive_scans=0 container_r_last=0 status=zero
E7RESULT name=verdict question=Q-B2 borrow_arm=N2 capacity=16 theta=8 batch=144 window=inf rollback_histories=true container_m_peak=0 container_m_peak_at=0 container_m_positive_scans=0 container_m_last=0 inode_m_peak=0 inode_m_peak_at=0 inode_m_positive_scans=0 inode_m_last=0 status=zero
E7RESULT name=verdict question=summary v2_positive_controls_paths_agree=true positive_controls_closed_form_holds=true v3_last_scan_paths_agree=true v4_two_runs_identical=true v5_anchors_hold=false v6_f10_cells_without_coverage=116 f11_rollbacks_too_far=4 f11_events_missing=24
```

Q-X2-6 的 36 行（12 格 × 3 条 X3 臂）每一行最小、最大都是 1、`not_one_and_one=0`。Q-X2-7 按族与 X3 臂把 12 个格的占比并起来（ppm；F11 的批是 144、对象至多 5 个单元，没有不是最后一批的批，E-L 两臂与 E-I 同值）：

Q-B2 那一列 `rollback_histories`（产物第二段，`grep -c 'rollback_histories=false'` 与 `=true` 各 24 行）：写 false 的格没有带回退的历史，装置不把 Q-B2 记进判定（`scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs` 里只在 `rollback_trajectories` 有值时 `judged("Q-B2"…)`），那几行 container_m 各量都是 0。

| 族 | E-I | E-L-w | E-L-r |
|---|---|---|---|
| F1 | 0 | 0 | 0 |
| F8 | 10135 | 766891 | 766891 |
| F3 | 27218 | 27218 | 27218 |
| F10 | 24226–24258 | 245510–246506 | 245510–246506 |
| F11 | 18448–30647 | 18448–30647 | 18448–30647 |

E-I 下 Q-X3-1 的峰值 96 行都是 1，Q-X3-2 的峰值 96 行都是 300（`grep 'question=Q-X3-[12] ' … | grep 'effect=E-I' | grep -oE 'inside_unlink_peak=[0-9]+' | sort | uniq -c` 输出 `96 inside_unlink_peak=1`、`96 inside_unlink_peak=300`）。

阳性对照按编号数（`grep 'name=positive_control' … | awk` 取 `id=`、`holds=`、`paths_agree=`、`closed_form_holds=`）：

```
      3 id=PC-B1 holds=false
    957 id=PC-B1 holds=true
     48 id=PC-B2
      1 id=PC-B2-direct holds=true paths_agree=true
    996 id=PC-EL paths_agree=true closed_form_holds=true
    996 id=PC-ELr paths_agree=true closed_form_holds=true
    144 id=PC-G0 holds=true
    576 id=PC-M-data paths_agree=true closed_form_holds=true
    612 id=PC-M-leaf paths_agree=true closed_form_holds=true
    552 id=PC-T paths_agree=true closed_form_holds=true
    552 id=PC-Z paths_agree=true closed_form_holds=true
```

PC-B1 不成立的三行与 PC-B2-direct（整行抄，第 68389、68613、71299、8 行）：

```
E7RESULT name=positive_control id=PC-B1 family=F11 n=300 gap_event=borrow batch=144 borrow_arm=N2 capacity=16 theta=4 disappeared_identities=4 final_container_r_inf=3 disappearance_scans_without_r_23=0 holds=false
E7RESULT name=positive_control id=PC-B1 family=F11 n=300 gap_event=borrow batch=144 borrow_arm=N2 capacity=16 theta=8 disappeared_identities=6 final_container_r_inf=4 disappearance_scans_without_r_23=0 holds=false
E7RESULT name=positive_control id=PC-B1 family=F11 n=300 gap_event=moved_object_shortened batch=144 borrow_arm=N2 capacity=16 theta=8 disappeared_identities=5 final_container_r_inf=4 disappearance_scans_without_r_23=0 holds=false
E7RESULT name=positive_control id=PC-B2-direct rolled_back=true b_on_container_m=0 b_off_lin_container_m=1 b_off_ret_container_m=0 expected=0,1,0 paths_agree=true holds=true
```

````

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:318-361`（整段抄，未转述）**

````markdown
### 读数：第三段（X3 的崩溃那一半）

产物里各类行的条数（`awk '{print $2}' research/results/e178-leaf-and-record-death-counts-segment-three.out | sort | uniq -c`）：

```
      1 name=config
     12 name=crash_prefix
   1440 name=crash_summary
      1 name=done
    288 name=positive_control
   2593 name=verdict
```

12 个前缀都找到了（`grep -c 'found=true'` 输出 12）：P-1 四个（容量 233、16 × n1、n2，θ = 0）、P-m 八个（容量 233 的 θ 58、116 与容量 16 的 θ 4、8 × n1、n2；P-m 要有借或吸收，θ = 0 不跑）。每个前缀 454 个崩溃点（u = 3、145、300 × 批 1、144：3 + 145 + 300 + 1 + 2 + 3），共 5448 个（B-on × Z × E-I × ∞ 那 72 行 `crash_points=` 相加）。判决行按问题、读者臂与状态数：

```
    288 question=Q-X3-4 base=B-on effect=E-I status=reported
    288 question=Q-X3-4 base=B-on effect=E-L-r status=zero
    288 question=Q-X3-4 base=B-on effect=E-L-w status=zero
    288 question=Q-X3-5 base=B-on effect=E-I status=incidental
    288 question=Q-X3-5 base=B-on effect=E-L-r status=incidental
    288 question=Q-X3-5 base=B-on effect=E-L-w status=incidental
    288 question=Q-X3-O2 base=B-on effect=E-I status=zero
    288 question=Q-X3-O2 base=B-on effect=E-L-r status=fl6_last_batch_effect_shows_extra_objects
    288 question=Q-X3-O2 base=B-on effect=E-L-w status=fl6_last_batch_effect_shows_extra_objects
      1 question=summary
```

Q-X3-O2 的 o2 按臂数（`o2_smallest`、`o2_largest` 与「每个崩溃点都为正」）：E-I 288 行都是 0 / 0；E-L-w、E-L-r 各 288 行都是 1 / 1、每个崩溃点都为正。Q-X3-5 的 864 行三个复活峰值都是 0。E-I 的 Q-X3-4：inode m 最小最大都是 1，extent m 最大 3、145、300 各 96 行（= u）。W 两档逐行比，1296 对一对都没翻面（草稿脚本按除 W 之外的字段配对数出来）。

整行抄（同一份产物第 4082、4083、4094、4095、4106、4107、4334 行）：

```
E7RESULT name=verdict question=Q-X3-O2 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-I window=inf o2_count=3 o2_smallest=0 o2_largest=0 o2_total=0 positive_crash_points=0 status=zero
E7RESULT name=verdict question=Q-X3-4 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-I window=inf inode_m_count=3 inode_m_smallest=1 inode_m_largest=1 inode_m_total=3 extent_m_count=3 extent_m_smallest=12 extent_m_largest=300 extent_m_total=468 status=reported
E7RESULT name=verdict question=Q-X3-O2 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-L-w window=inf o2_count=3 o2_smallest=1 o2_largest=1 o2_total=3 positive_crash_points=3 status=fl6_last_batch_effect_shows_extra_objects
E7RESULT name=verdict question=Q-X3-4 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-L-w window=inf inode_m_count=3 inode_m_smallest=0 inode_m_largest=0 inode_m_total=0 extent_m_count=3 extent_m_smallest=0 extent_m_largest=0 extent_m_total=0 status=zero
E7RESULT name=verdict question=Q-X3-O2 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-L-r window=inf o2_count=3 o2_smallest=1 o2_largest=1 o2_total=3 positive_crash_points=3 status=fl6_last_batch_effect_shows_extra_objects
E7RESULT name=verdict question=Q-X3-4 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-L-r window=inf inode_m_count=3 inode_m_smallest=0 inode_m_largest=0 inode_m_total=0 extent_m_count=3 extent_m_smallest=0 extent_m_largest=0 extent_m_total=0 status=zero
E7RESULT name=verdict question=summary v2_positive_controls_paths_agree=true positive_controls_closed_form_holds=true v3_paths_agree=true prefixes_missing=0
```

阳性对照 PC-O-plant、PC-O-remove、PC-M-data、PC-M-leaf 各 72 行（12 前缀 × 6 个 (u, b)），每行 `paths_agree=` 与 `closed_form_holds=` 都等于那一行的 `crash_points=`。

````

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:362-571`（整段抄，未转述）**

````markdown
### 读数：第四段（X1 的两个新底座 B-stub、B-tsz）

产物里各类行的条数（`awk '{print $2}' research/results/e178-leaf-and-record-death-counts-segment-four.out | sort | uniq -c`）：

```
      4 name=anchor
     17 name=assertion
      1 name=config
  30840 name=difference
      1 name=done
   9080 name=item_assertion
   9754 name=positive_control
     10 name=positive_control_tally
   9960 name=reader_extras
   1008 name=side_by_side
   1494 name=simulation
     98 name=stability
    498 name=stub_leaves
  74016 name=trajectory
   3398 name=verdict
```

1494 = 996 项原写路径 + 498 项 B-stub 写路径（只有 n1 的 6 个写路径格），按族数（`grep 'name=simulation' … | awk` 取 `family=` 与 `path=`）F1 168 / 84、F3 60 / 30、F8 384 / 192、F10 336 / 168、F11 48 / 24，每一行 `two_runs_identical=true last_scan_paths_agree=true`。74016 = 原写路径 18 个读者 × (948 × 3 + 48 × 5) 类扫描 + B-stub 写路径 12 个读者 × (474 × 3 + 24 × 5)；按底座 B-on 37008、B-off-ret 6168、B-tsz 12336、B-stub 18504。30840 = B-stub 减 B-on 18504 + B-tsz 减 B-on 12336。每族都造出了被测形状：每个族 × 容量 × θ × 批的格里 B-stub 都写了桩（3–518 个，Q-Bs-5），B-tsz 的容器 r 峰值在每一格都 ≥ 1（死身份确实出现过），PC-Bs1、PC-Bt1 在每一族 × W 两档上都取到了扫描（汇总行 `pc_bs1_taken`、`pc_bt1_taken`）。

判决行按问题、量与状态数（`grep 'name=verdict question=Q-B[st]-[1-4] ' … | awk` 取 `question=`、`quantity=`、`tier=`、`status=` / `judgement=`、`sub_question=`、`differs=`，再按问题并起来）：

| 问题 | 行数 | 状态 |
|---|---|---|
| Q-Bs-1 B-stub 复活（容器 r、inode r、extent r 比 B-on 多） | 432 | 都是 `zero` |
| Q-Bs-2 B-stub 误判死亡（容器 m、inode m、extent m 比 B-on 多；全部与回退之后两档） | 864 | 648 `zero`；216 `not_run`（批 = 1 的格回退之后那一档，F11 只有批 144） |
| Q-Bs-3 B-stub 下 unlink 途中漏活 | 288 | E-L-w、E-L-r 192 行 `zero`；E-I 96 行 `reported`（定义的推论） |
| Q-Bs-4 B-stub 下的 X2（Q-X2-1 至 Q-X2-5） | 384 | 都是 `differs=false`，新底座与 B-on 都是 `zero` |
| Q-Bt-1 B-tsz 复活 | 288 | inode r、extent r 比 B-on 多的 192 行 `zero`；容器 r 96 行 `reported`（定义的推论） |
| Q-Bt-2 B-tsz 误判死亡 | 672 | 432 `zero`；144 `not_run`（同 Q-Bs-2）；两种供出规则选出不同记录的条数 96 行，全是 `total=0` |
| Q-Bt-3 B-tsz 的 unlink 途中漏活 | 192 | 都是 `reported`（只配 E-I，定义的推论） |
| Q-Bt-4 B-tsz 下的 X2（Q-X2-1 至 Q-X2-4） | 192 | 都是 `differs=false`，新底座与 B-on 都是 `zero` |
| Q-Bs-5、Q-Bs-6 | 各 42 | `reported` |

E-I 下 Q-Bs-3 与 Q-Bt-3 的途中峰值：inode m 都是 1、extent m 都是 300（`grep -E 'question=Q-B[st]-3 ' … | grep 'effect=E-I'` 数出 Q-Bs-3 48 + 48 行、Q-Bt-3 96 + 96 行），与第二段 B-on 的 E-I 同值。B-tsz 的容器 r 峰值按格 1–166，同格 B-on 0（`b_on_peak=0`）。

整行抄（同一份产物；容量 16、θ = 8、n1、批 144、W = ∞、Z × E-I 那一格的两个新底座，同格 E-L-w 的途中漏活，n2 同格 B-tsz 的容器 r，与汇总、FL-Bx、两个构造）：

```
E7RESULT name=verdict question=Q-Bs-1 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=container_r container_r_peak=0 container_r_peak_at=0 container_r_positive_scans=0 container_r_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bs-1 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=inode_r inode_r_peak=0 inode_r_peak_at=0 inode_r_positive_scans=0 inode_r_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bs-2 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I tier=after_rollback quantity=container_m container_m_peak=0 container_m_peak_at=0 container_m_positive_scans=0 container_m_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bs-2 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I tier=after_rollback quantity=inode_m inode_m_peak=0 inode_m_peak_at=0 inode_m_positive_scans=0 inode_m_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bs-3 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-L-w quantity=inode_m inode_m_inside_unlink_peak=0 inode_m_inside_unlink_peak_at=0 inode_m_inside_unlink_positive_scans=0 inode_m_inside_unlink_last=0 b_on_status=zero status=zero
E7RESULT name=verdict question=Q-Bs-3 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-L-w quantity=extent_m extent_m_inside_unlink_peak=0 extent_m_inside_unlink_peak_at=0 extent_m_inside_unlink_positive_scans=0 extent_m_inside_unlink_last=0 b_on_status=zero status=zero
E7RESULT name=verdict question=Q-Bs-4 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I sub_question=Q-X2-1 new_base_status=zero b_on_status=zero differs=false
E7RESULT name=verdict question=Q-Bt-1 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=container_r container_r_peak=56 container_r_peak_at=1799 container_r_positive_scans=25219 container_r_last=56 b_on_peak=0 status=reported
E7RESULT name=verdict question=Q-Bt-1 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=inode_r inode_r_peak=0 inode_r_peak_at=0 inode_r_positive_scans=0 inode_r_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bt-2 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I tier=after_rollback quantity=inode_m inode_m_peak=0 inode_m_peak_at=0 inode_m_positive_scans=0 inode_m_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bt-2 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=provenance_differences total=0 judgement=reported
E7RESULT name=verdict question=Q-Bt-4 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I sub_question=Q-X2-1 new_base_status=zero b_on_status=zero differs=false
E7RESULT name=verdict question=Q-Bt-1 base=B-tsz borrow_arm=N2 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=container_r container_r_peak=166 container_r_peak_at=1799 container_r_positive_scans=26876 container_r_last=166 b_on_peak=0 status=reported
E7RESULT name=verdict question=FL-Bx x2_cells_whose_judgement_changes_with_the_base=0 cells=none
E7RESULT name=verdict question=summary v2_paths_disagree=0 v2_closed_form_fails=0 v3_last_scan_paths_disagree=0 v4_runs_not_identical=0 v5_anchors_hold=true v6_f10_cells_without_coverage=116 v8_new_base_trajectories_with_empty_reads=2448 v_bs1_pairing_fails=0 v_bs3_segments_with_stub_read_mismatches=0 does_not_fit_segments=0 f11_rollbacks_too_far=4 f11_events_missing=24 pc_bt2_segments_with_a_positive_hand_count=0 pc_bt2_mid_segments_with_a_positive_hand_count=220 pc_bs1_taken=F1/W23,F1/Winf,F10/W23,F10/Winf,F11/W23,F11/Winf,F3/W23,F3/Winf,F8/W23,F8/Winf pc_bt1_taken=F1/W23,F1/Winf,F10/W23,F10/Winf,F11/W23,F11/Winf,F3/W23,F3/Winf,F8/W23,F8/Winf construction=PC-Bs2=true,PC-Bt3=true
E7RESULT name=positive_control id=PC-Bs2 rolled_back=true b_stub_container_m=0 b_stub_inode_m=0 b_stub_extent_m=0 b_stub_lin_off_container_m=1 b_stub_lin_off_inode_m=1 b_stub_lin_off_extent_m=0 expected=0,0,1,1,0 paths_agree=true holds=true
E7RESULT name=positive_control id=PC-Bt3 rolled_back=true b_tsz_inode_m=0 b_tsz_container_m=0 b_tsz_extent_m=0 b_tsz_inode_m_ignoring_lineage=1 expected=0,1 paths_agree=true holds=true
```

这几行在产物第 136748、136749、136754、136755、136783、136784、136759、136830、136831、136837、136843、136841、138568、138960、140178、135646、135647 行。

B-stub 的叶片数、桩占比与撞 I-9.13（类型 2 容器不空） 的版本（Q-Bs-5、Q-Bs-6 那 84 行按族与容量并起来，θ 与批各档取最大或区间；命令与原样输出）：

```
grep -E 'question=Q-Bs-[56] ' research/results/e178-leaf-and-record-death-counts-segment-four.out | awk '{for(i=3;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]}; k=v["family"]" "v["capacity"]; if(v["question"]=="Q-Bs-5"){sw[k]=sw[k]","v["stubs_written"]; if(v["stubs_last_largest"]>st[k])st[k]=v["stubs_last_largest"]; if(v["leaves_peak_largest"]>lp[k])lp[k]=v["leaves_peak_largest"]; if(v["stub_share_last_ppm_largest"]>sh[k])sh[k]=v["stub_share_last_ppm_largest"]; ov[k]+=v["versions_over_the_root"]} else {p=v["share_ppm"]+0; if(!(k in mn)||p<mn[k])mn[k]=p; if(p>mx[k])mx[k]=p}} END{for(k in st) print k, "stubs_written="substr(sw[k],2), "stubs_last_max="st[k], "leaves_peak_max="lp[k], "stub_share_last_ppm_max="sh[k], "versions_over_the_root="ov[k], "i913_false_share_ppm="mn[k]"-"mx[k]}' | sort
F10 16 stubs_written=518,518,518,518,518,518 stubs_last_max=56 leaves_peak_max=57 stub_share_last_ppm_max=982456 versions_over_the_root=0 i913_false_share_ppm=264180-526791
F10 233 stubs_written=28,28,28,28,28,28 stubs_last_max=3 leaves_peak_max=4 stub_share_last_ppm_max=750000 versions_over_the_root=0 i913_false_share_ppm=56343-462809
F11 16 stubs_written=57,26,25 stubs_last_max=18 leaves_peak_max=19 stub_share_last_ppm_max=947368 versions_over_the_root=0 i913_false_share_ppm=340586-543564
F11 233 stubs_written=4,5,5 stubs_last_max=1 leaves_peak_max=2 stub_share_last_ppm_max=500000 versions_over_the_root=0 i913_false_share_ppm=1056-286353
F1 16 stubs_written=518,518,518 stubs_last_max=56 leaves_peak_max=57 stub_share_last_ppm_max=982456 versions_over_the_root=0 i913_false_share_ppm=386173-670710
F1 233 stubs_written=28,28,28 stubs_last_max=3 leaves_peak_max=4 stub_share_last_ppm_max=750000 versions_over_the_root=0 i913_false_share_ppm=82675-594982
F3 16 stubs_written=3,3,4 stubs_last_max=2 leaves_peak_max=66 stub_share_last_ppm_max=35087 versions_over_the_root=0 i913_false_share_ppm=366358-499727
F3 233 stubs_written=3,3,3 stubs_last_max=2 leaves_peak_max=6 stub_share_last_ppm_max=333333 versions_over_the_root=0 i913_false_share_ppm=366358-366358
F8 16 stubs_written=16,16,16,16,16,16 stubs_last_max=1 leaves_peak_max=1 stub_share_last_ppm_max=1000000 versions_over_the_root=0 i913_false_share_ppm=15473-106666
F8 233 stubs_written=16,16,16,16,16,16 stubs_last_max=1 leaves_peak_max=1 stub_share_last_ppm_max=1000000 versions_over_the_root=0 i913_false_share_ppm=15473-106666
```

`stubs_written` 是各段合计、按 θ（与批）各一个数；`i913_false_share_ppm` 是有零条目版本的版本数占全部发布的比例（Q-Bs-6 的 `share_ppm`，分母同第一节读法 3）。θ 越大撞 I-9.13（类型 2 容器不空） 的版本越多（F1 容量 16：θ 0、4、8 是 386173、615828、670710 ppm，第一个桩最早在第 71、49、41 次发布）；推的成因是 θ > 0 时被吸收的片也落桩，第一个桩来得更早，没另量。a10、a11 两条锚点都对上（`assertion=A10_stubs_written_equal_final_stubs_equal_a8 judged=28 violated=0`、`assertion=A11_first_stub_publish_and_versions_with_a_stub judged=4 violated=0`）。叶片总数从来没超过根装得下的 135 片（42 行 `versions_over_the_root=0`），所以「桩让根长高」在这些历史里没出现，模型也不算根长高的写。

三个底座并排（`name=side_by_side` 1008 行 = 84 个族 × 写路径格 × 批 × Z / T × 三条 X3 臂 × W 两档；每格写「复活 容器/inode/extent : 误判死亡 容器/inode/extent」）：

```
grep 'name=side_by_side' research/results/e178-leaf-and-record-death-counts-segment-four.out | awk '{for(i=1;i<=NF;i++){if($i ~ /^b_on=/)a=$i; if($i ~ /^b_stub=/)b=$i; if($i ~ /^b_tsz=/)c=$i}; sub(/^b_tsz=[0-9]+\//,"b_tsz=<container_r>/",c); print a, b, c}' | sort | uniq -c
    168 b_on=0/0/0:0/0/0 b_stub=0/0/0:0/0/0 b_tsz=<container_r>/0/0:0/0/0
    336 b_on=0/0/0:0/0/0 b_stub=0/0/0:0/0/0 b_tsz=n/a
    168 b_on=0/0/0:0/0/0 b_stub=n/a b_tsz=<container_r>/0/0:0/0/0
    336 b_on=0/0/0:0/0/0 b_stub=n/a b_tsz=n/a
```

`n/a` 是那条臂不配的格：B-stub 不配 n2（问法一 n2-none），B-tsz 不配 E-L-w、E-L-r。两行整行抄（第 139585、139729 行）：

```
E7RESULT name=side_by_side family=F10 borrow_arm=N1 capacity=16 theta=8 batch=1 window=inf shrink=Z effect=E-I resurrected_c_i_e_and_misjudged_c_i_e b_on=0/0/0:0/0/0 b_stub=0/0/0:0/0/0 b_tsz=56/0/0:0/0/0
E7RESULT name=side_by_side family=F10 borrow_arm=N2 capacity=16 theta=8 batch=1 window=inf shrink=Z effect=E-I resurrected_c_i_e_and_misjudged_c_i_e b_on=0/0/0:0/0/0 b_stub=n/a b_tsz=166/0/0:0/0/0
```

两条执行员另做的比对（登记没写成判据，只当旁证）：B-stub 的 18504 行轨迹把 `base=` 换掉之后与同格 n1 的 B-on 18504 行逐行相同（容器、inode、extent 的 r 与 m 在每个范围上都一样）；B-tsz × Z × E-I 的 6168 行与 B-off-ret × Z × E-I 的 6168 行换掉 `base=` 之后逐行相同。后一条就是登记修订 2 之四第 1 条推的「P-W 下两者是同一个读者」，P-C 与 P-W 在这些历史上选出的记录一条不差（`provenance_differences total=0` 96 行），所以它也成立。B-tsz × Z 那几格的答案因此在第四段跑之前就已经在第二段 B-off-ret × Z × E-I 的产物里，第二、三段那一位执行员读过那份产物，那几格不算盲判（登记修订 2 之四第 1 条要求照实写）。

V-Bt2（第四段的 B-on、B-off-ret 与第二段同格同读者逐行比）：第二段每格多 4 个 g0 读者（B-on × g0 × E-I / E-L-w × W 两档，第二段阳性对照用，修订 2 之五第四段的读者表里没有它们），去掉之后逐行相同：

```
grep '^E7RESULT name=trajectory' research/results/e178-leaf-and-record-death-counts-segment-four.out | grep -E 'base=(B-on|B-off-ret) ' | sort > four.txt
grep '^E7RESULT name=trajectory' research/results/e178-leaf-and-record-death-counts-segment-two.out | grep -E 'base=(B-on|B-off-ret) ' | sort > two.txt
grep -v ' shrink=G0 ' two.txt > two-without-g0.txt; wc -l four.txt two.txt two-without-g0.txt; cmp four.txt two-without-g0.txt && echo identical
   43176 four.txt
   55512 two.txt
   43176 two-without-g0.txt
identical
```

阳性对照计数与断言总计（整行抄，第 140168–140177 行与第 140151–140167 行里的第四段那几条）：

```
E7RESULT name=positive_control_tally id=PC-Bs1 count=962 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-Bt1 count=1724 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-Bt2 count=224 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-Bt2-mid count=224 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-EL count=1494 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-ELr count=1494 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-M-data count=864 paths_disagree=0 closed_form_fails=0 registered_literal_fails=18
E7RESULT name=positive_control_tally id=PC-M-leaf count=1110 paths_disagree=0 closed_form_fails=0 registered_literal_fails=1110
E7RESULT name=positive_control_tally id=PC-T count=828 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-Z count=828 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=assertion path=segment_four assertion=A12_tombstones_only_container_r_equals_without_retirement judged=4374408 violated=0
E7RESULT name=assertion path=segment_four assertion=A12_tombstones_only_final_container_r_equals_a8 judged=112 violated=0
E7RESULT name=assertion path=segment_four assertion=V-Bs1_pairing_holds judged=498 violated=0
E7RESULT name=assertion path=segment_four assertion=V-Bs3_stubs_read_equal_the_current_stubs judged=498 violated=0
E7RESULT name=assertion path=segment_four assertion=V2_positive_control_closed_form_holds judged=9752 violated=0
E7RESULT name=assertion path=segment_four assertion=V2_positive_control_paths_agree judged=9752 violated=0
```

`registered_literal_fails` 只对 PC-M 两项算（装置 `segment_four_positive_control_line` 只给 PC-M 判登记字面），别的对照那一栏的 0 是没判、不是照字面成立；PC-T 的判法与第二段相同，比登记松。PC-M 照登记字面仍然不成立（数据单元 18 项、叶容器 1110 项全部），装置的 ≤ 判法成立，与第二段同一个成因。PC-Bt2-mid 224 项的手算值（`grep 'id=PC-Bt2-mid ' … | grep -oE 'hand_extent_r_z=[0-9]+' | sort | uniq -c`）：0 有 4 项、1 有 114 项、2 有 36 项、3 有 32 项、4 有 38 项。稳定性比对 98 行（Q-Bs-1 至 Q-Bs-4、Q-Bt-1、Q-Bt-2、Q-Bt-4 在 W、θ、容量、批四个旋钮上）都是 `stable=true`、翻面数都是 0。

| 条款 | 打中没有 | 依据 |
|---|---|---|
| FL0 条款可能错 | 没打中：K1、K2、K3 与 a2 四行 `holds=true`，K6 在 12654 次分裂上 `violated=0` | 产物第 2–5 行、第 1523 行 |
| FL8 n1 漏判 | 打中：Q-X4-4 两种布局都是 8 形，名单原样在判决那一节；K-2 判成绿，登记第四节第 4 条推的那一格没错 | 产物第 1596、1620 行 |
| FL9 n2 的「I-9.4（容器号不超最小 key） 原样」不成立 | 没打中：n2 在六个格上2026-10-08 的 I-9.4（容器号不超最小 key） 与 I-9.12（分隔 key 落在孩子区间之外） 红的版本数都是 0 | V7 那十二行 |
| FL10 n1 改过的 I-9.4（容器号不超最小 key） 在合法镜像上不绿 | 没打中：n1 在六个格上 n1 的 I-9.4（容器号不超最小 key） 与 I-9.12（分隔 key 落在孩子区间之外） 红的版本数都是 0 | V7 那十二行 |
| FL1 底座输一格 | 没打中：Q-B1 48 行、Q-B2 有回退历史的 24 行都是 `status=zero`，判决行 `fl1_cell_excluded=false` 1344 行 | 第二段判决行 |
| FL2 Z 输一格、FL3 T 输一格、FL4 一起输 | 没打中：Q-X2-1 至 Q-X2-4 各 144 行都是 0 | 第二段判决行 |
| FL5「最后一批生效」也漏活 | 没打中：E-L-w、E-L-r 的 Q-X3-1、Q-X3-2 各 192 行是 0（第二段）；Q-X3-4 各 288 行是 0（第三段） | 第二、三段判决行 |
| FL6「最后一批生效」崩溃后多出对象 | 照字面打中：Q-X3-O2 在 E-L-w、E-L-r 下每个崩溃点都是 1（576 行）。记 E-L-w、E-L-r 各输一项。这一项是两条臂定义的直接推论：E-L 两臂在 unlink 途中照定义让 N 的记录留在重建视图里（登记 5.3「unlink 途中 o 的记录还在树里」），崩溃点又都在意图到最后一批之间，o2 数的正是这一条；门槛与臂定义同义反复（`test-discipline.md`「判据不写成合取，门槛不能是臂定义的同义反复」），这一格算不算判据交主 agent | 第三段判决行 |
| FL7 E-I 崩溃后多出对象 | 没打中：E-I 的 Q-X3-O2 288 行都是 0 | 第三段判决行 |
| V1 历史复现对不上 | 没打中：a1 两条断言各判 612 次（51 段 × 6 格 × 2 臂）、违反 0；F3 五段的发布数与被拒步数与 E177（逐树不写墓碑的复活数） 产物 `geometry=wide` 那条 `name=simulation` 行的两段相同 | 产物第 1520–1521 行；装置常量 `E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS` |
| V4 不确定 | 没打中：474 个工作项两遍逐项相同（`grep -c 'two_runs_identical=true'` 在 `name=borrow_cost` 行上数出 474）、n1 与 n2 的用户改动序列逐段相同；`E178_THREADS=6` 与 `=3` 两份整产物 sha256 都是 `9644a139…a8ea` | 产物第 1625 行 |
| V5 独立锚点对不上 | 没打中：a2、a5、a6、a8 都对上（a8 判 184 次、违反 0） | 产物第 5、1522、1524、1525 行 |
| V6 路径没跑到 | 部分打中：F10 在 θ > 0 的四个格里，容量 233、θ = 58 有 24 段，θ = 116 有 14 段，容量 16、θ = 4 有 20 段一次借都没有（被挪过的对象变短、被 unlink 的次数在每一段都 > 0）；容量 16、θ = 8 的 28 段都跑到。这些段对 X4 不算结果；每个格都有跑到的段，族不作废。没跑到的段借的次数是 0，不往 Q-X4-1、Q-X4-2 里加观测 | Q-X4-3 F10 那六行的 `v6_segments_without_coverage` |
| V7 模型写路径自查 | 没打中：要全绿的格都是 0；I-9.13（类型 2 容器不空） 在 12 行里都是 0 | V7 那十二行 |
| V1 | 没打中：a1 两条各判 612 次、违反 0 | 第二段 `name=assertion` 行 |
| V2（第二段）PC-B1 | 打中三段：F11 的借缺口（容量 16、θ = 4、n2）、借缺口（θ = 8、n2）、被挪过的对象变短缺口（θ = 8、n2），`holds=false`，消失过的身份 4、6、5 个而 B-off-ret 期末容器 r 3、4、4。照字面作废那三段。成因（单测 `identities_born_and_retired_inside_the_abandoned_gap_are_what_pc_b1_overcounts` 复算出同样的 (4, 3)、(6, 4)、(5, 4) 并钉住）：差的那几个身份每一版的诞生 txg 都不在期末父链上（n2 借在缺口里给右片换的新身份、又在缺口里消失），任何读者都读不到；PC-B1 的期望数的是「消失过的身份」，没扣这一类。V5 汇总那一格（`v5_anchors_hold=false`）就是这三次违反（`assertion=PC-B1_without_retirement_resurrects_every_disappeared_identity judged=960 violated=3`），别的 11 条断言违反都是 0 | 第二段第 68389、68613、71299、73281、73283 行 |
| V2（第二段）PC-M | 照登记字面不成立：登记写「每个读者 extent m + 1；抹容器那一次 inode m + 那片的记录数、容器 m + 1」，产物里抹数据单元的 576 项有 12 项全部读者 + 0（被抹的那个单元那一组还有别的版本，`single_version=false`），抹叶容器的 612 项没有一项全部读者都是 + 记录数（W = ∞ 的读者都回落到那片读得出的上一版，132 项里 W = 23 的 10 个读者是 + 记录数）。装置的判法写成 ≤（`erasure_positive_controls`），比登记松，所以 `closed_form_holds=true`。按字面算 V2 打中，作不作废交主 agent | 第二段 `id=PC-M-` 行；命令在报告 |
| V2（第二段）PC-T | 装置只判 Z、g0 的增量是 0，T 的增量没有按登记写的个数去比（参照路径与主路径相同，`paths_agree=true`）；552 项里 308 项 T 读者增量全为正、220 项部分为正、24 项全是 0。比登记松，交主 agent | 第二段 `id=PC-T ` 行 |
| V2（第二段）其余 | PC-Z、PC-EL、PC-ELr 两路一致、期望成立；PC-G0 144 项（= a3）成立；PC-B2-direct 成立；F11 上原 PC-B2 主 agent 定不当结果 | 阳性对照计数 |
| V3 两路计数不一致 | 没打中：第二段 `v3_last_scan_paths_agree=true`，第三段 `v3_paths_agree=true` | 两段汇总行 |
| V4 不确定 | 没打中（第二段）：996 项两遍逐次扫描相同（`grep -c 'two_runs_identical=true last_scan_paths_agree=true'` 在 `name=simulation` 行上数出 996）；换线程数没比（第二、三段只跑了一次） | 第二段汇总行 |
| V5 | a3、a4、a7、a8、a9、K4、K6 都是违反 0；汇总行的 `v5_anchors_hold=false` 只来自 PC-B1 那三次（V2 那一行） | 第二段第 73271–73282 行 |
| V6 | F10 在 θ > 0 的 224 项里 116 项借、被挪过之后变短、被挪过之后被 unlink 三个次数有一个是 0（`v6_f10_cells_without_coverage=116`）；8 个 (N, 容量, θ) 格每格都有跑到的段（容量 16、θ = 4 各 8 段，θ = 8 各 28 段；容量 233、θ = 58 各 4 段，θ = 116 各 14 段；n1、n2 同），族不作废。F11 有 24 项缺口事件没出现（`f11_events_missing=24`，多是 θ = 0 没有借与吸收），那几项对 PC-B2 不算结果 | 第二段 `name=simulation family=F10` 行；命令在报告 |
| F11 回退过远 | 4 段（被挪过的对象变短缺口，容量 233、θ = 58 与容量 16、θ = 8，n1、n2）回退距离 21，照登记字面记「回退过远」、这几段不算结果 | 第二段 `id=PC-B2` 行的 `too_far=true` |
| V8 读出空 | 照字面打中：E-I 读者（B-on 的 Z、T、g0，B-off-ret、B-off-lin）在 F8 的全部 384 项、F3 的 60 项里 24 项上有途中扫描读出的 inode 记录集合为空而真值不空（E-I 照定义丢掉唯一那条记录），那几次扫描照登记作废、不当 m = |T| 记。E-L 两臂一次都没有。这几次都在 unlink 途中，只碰 E-I 的 Q-X3-1、Q-X3-2（只报数、不判），不碰 X2 与 E-L 的判定 | 第二段 `name=trajectory` 行的 `read_out_empty_scans`；命令在报告 |
| 第三段的作废条款 | V2：PC-O 两项、PC-M 两项在 5448 个崩溃点上两路一致、装置写的期望成立（PC-M 同样是 ≤ 判法）；V3 没打中；前缀全找到 | 第三段汇总行 |
| 停机 S1、S2、S3、S6 | 都对上 | 产物第 6–9 行 |
| 停机 S4、S5 | 都对上：`E7RESULT name=stop_check id=S4 holds=true detail=record=Some(InodeRecord { inode: 1, object_birth: 3, nlink: 0, size_bytes: 97902 }) tombstones=[TombstoneRecord { object: 1, object_birth: 3, range_start_bytes: 0, death_txg: 7 }] intent_txg=7`、`E7RESULT name=stop_check id=S5 holds=true detail=container_sizes=[815, 1] container_numbers=[1, 2] intent_publishes=816` | 第二段第 6、7 行 |
| 停机（第四段） | 不新增对拍项；为第四段改过装置之后 S1–S6 的单测照跑、都过（单测输出） | `e178-leaf-and-record-death-counts-unit-tests-2026-10-08.out` |
| FL-Bs1 B-stub 输一格 | 没打中：Q-Bs-1 的 432 行、Q-Bs-2 判了的 648 行都是 `zero`；PC-Bs2 构造上 B-stub 自己的容器 m、inode m、extent m 都是 0 | 第四段判决行；第 135646 行 |
| FL-Bt1 B-tsz 输一格 | 没打中：Q-Bt-1 的 inode r 与 extent r 差 192 行、Q-Bt-2 判了的 432 行都是 `zero`；PC-Bt3 构造上 B-tsz 自己的 inode m、容器 m、extent m 都是 0 | 第四段判决行；第 135647 行 |
| FL-B3 三个底座同格一起输 | 没打中：三个底座一格都没输 | 并排 1008 行 |
| FL-Bs2 B-stub 下「最后一批生效」也漏活 | 没打中：Q-Bs-3 的 E-L-w、E-L-r 192 行都是 `zero`；与 FL5 在 B-on 下没打中并排 | 第四段判决行 |
| FL-Bx X2 的判定随底座变 | 没打中：Q-Bs-4 384 行、Q-Bt-4 192 行都是 `differs=false`，名单空（`x2_cells_whose_judgement_changes_with_the_base=0 cells=none`） | 第 138960 行 |
| V-Bs1 配对前提 | 没打中：498 项配对断言违反 0 | 第 140160 行 |
| V-Bs2 B-stub 锚点 | 没打中：a10 判 28 次、a11 判 4 次，违反 0 | 第 140156、140157 行 |
| V-Bs3 读到的桩数 | 没打中：498 项违反 0，汇总 `v_bs3_segments_with_stub_read_mismatches=0` | 第 140161、140178 行 |
| V-Bt1 B-tsz 与 B-off-ret | 没打中：a12 逐次判 4374408 次、期末判 112 次，违反 0；问法二取的是 P-C，P-W 那一句不判，但两种规则选出不同记录 0 条，轨迹也逐行相同（「读数：第四段」） | 第 140158、140159 行 |
| V-Bt2 重出的 B-on 与第二段不同 | 没打中：43176 行逐行相同（第二段多出的 12336 行是第四段不跑的 g0 读者） | 「读数：第四段」那条命令 |
| V2 | 两路一致与装置的期望全部成立（9752 次、违反 0）；PC-Bs1、PC-Bt1、PC-Bt2-mid 成立；原 PC-Bt2 的「F10 θ > 0 至少一段 > 0」照修订 3 写明的量不出（224 项手算值都是 0，`pc_bt2_segments_with_a_positive_hand_count=0`），交主 agent 定还算不算结果；PC-M 照登记字面不成立 1128 项（同第二段），PC-T 的判法同第二段比登记松，都交主 agent | 第 140168–140178 行 |
| V3、V4（第四段） | 没打中：`v3_last_scan_paths_disagree=0`、`v4_runs_not_identical=0`，1494 条写路径两遍逐次相同；本机分两片算、补算、拼，与线程数、片数无关由抽 20 项重算逐字节相同旁证 | 第 140178 行；`name=simulation` 行 |
| V6 | 与第二段相同：F10 θ > 0 有 116 项没跑全三个次数（`v6_f10_cells_without_coverage=116`），每个格都有跑到的段；F11 缺口事件没出现 24 项、回退过远 4 段 | 第 140178 行 |
| V8 | 照字面打中，形态同第二段：新底座有 2448 条轨迹（B-stub 816、B-tsz 1632）出现读出空的扫描，全是 E-I 读者、全在 F3、F8；这几条轨迹在「不在途中」那一档的 inode m 峰值都是 0（读出空而真值不空必然 m ≥ 1），所以读出空全在 unlink 途中，只碰只报数的 Q-Bs-3、Q-Bt-3 的 E-I 那几格 | 第 140178 行；命令在执行员报告 |

V6 那几段一次借都没有的分法（n2 那一臂，F10、θ > 0；`b0` 是借 0 次，`s+ u+` 是变短、被 unlink 都 > 0；同一个数在两个批大小上各一行）：

```
grep 'name=write_path family=F10' research/results/e178-leaf-and-record-death-counts-segment-one.out | grep 'borrow_arm=N2' | grep -v 'theta=0 ' | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]} print v["capacity"], v["theta"], v["n"], v["batch"], (v["borrows"]>0?"b+":"b0"), (v["moved_then_shortened"]>0?"s+":"s0"), (v["moved_then_unlinked"]>0?"u+":"u0")}' | sort | uniq -c | sort -k2,3n
      1 16 4 300 144 b+ s+ u+
      1 16 4 300 1 b+ s+ u+
      3 16 4 900 144 b+ s+ u+
      3 16 4 900 1 b+ s+ u+
      4 16 4 900 144 b0 s+ u+
      4 16 4 900 1 b0 s+ u+
      6 16 4 300 144 b0 s+ u+
      6 16 4 300 1 b0 s+ u+
      7 16 8 300 144 b+ s+ u+
      7 16 8 300 1 b+ s+ u+
      7 16 8 900 144 b+ s+ u+
      7 16 8 900 1 b+ s+ u+
      2 233 58 900 144 b+ s+ u+
      2 233 58 900 1 b+ s+ u+
      5 233 58 900 144 b0 s+ u+
      5 233 58 900 1 b0 s+ u+
      7 233 116 300 144 b0 s+ u+
      7 233 116 300 1 b0 s+ u+
      7 233 116 900 144 b+ s+ u+
      7 233 116 900 1 b+ s+ u+
      7 233 58 300 144 b0 s+ u+
      7 233 58 300 1 b0 s+ u+
```

````

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:572-585`（整段抄，未转述）**

```markdown
### 单测与变异

续跑键（装置代码键）的样本与证红（2026-10-15）：`e178-leaf-and-record-death-counts-resume-key-samples-2026-10-15.out` 判决行 `key_samples=8 key_samples_holding=8 f_k1_missed=0 f_k2_extra=0 unit_test_samples=1`；项键两条变异 M23（项键漏参数）、M24（项键带序号）的证红在 `e178-leaf-and-record-death-counts-resume-key-prove-red-2026-10-15.out`，两条都退 101，还原后 `restored exit=0 passed=54 failed=0`。

加了第四段之后：单测 44 个（`grep -c '^test .* \.\.\. ok$' e178-leaf-and-record-death-counts-unit-tests-2026-10-08.out` 输出 44；同一份文件第 51 行 `test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.48s`）。新加的 12 个钉：SHA-256 的两个标准值；a10、a11（H51 F1 n = 300 升序、容量 16、θ = 0：桩 18、期末叶片 19、第一个桩第 71 次发布、有桩的版本 537；n = 900 容量 233 先偶后奇：桩 3、叶片 4）；V-Bs1（F10 两段、F3、F11 吸收缺口上两条写路径逐版配对，且真有桩、真有吸收）；PC-Bs2、PC-Bt3 的构造；B-stub 期末容器 r 0、读到的桩 18 且逐次等于现行桩数（V-Bs3）；B-tsz 期末容器 r 18（= B-off-ret，a12）、inode r 0，PC-Bt1 两档 W 成立；F11 吸收缺口里 B-stub 容器 r 恒 0；PC-Bt2 期末手算 0、PC-Bt2-mid 手算 > 0 而 P-C 下那次扫描 extent r 0；第四段一个工作项的配对、阳性对照与断言；续跑（关复用与开复用拼出的产物逐行相同，分片只算轮到的项）；参数；第二段 PC-B1 那三段的成因。

变异表 `scripts/mutation/tables/e178_leaf_and_record_death_counts.tsv` 加到 22 条（修订 2 之十的 M16–M22 七条；22 条锚点各恰好命中一次）：抓到 22、无效 0、没红 0；`💥`、`⚠️`、`⏱`、`🧱` 各 0；收尾「计数：内存撞顶 0 条（上限 16G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条」「已还原，基线仍全绿」（`e178-leaf-and-record-death-counts-mutations-2026-10-08.out` 里「计数：」与「已还原，基线仍全绿」两行）。M18 在登记写的取样点（期末那次扫描）上不敏感，由 PC-Bt2-mid 那次扫描抓到（修订 3）；M15 补的那一格（PC-Bt3 构造上 B-tsz 的 inode m + 1）由构造那条单测抓到。

第一段时：单测 32 个（`grep -c '^test .* \.\.\. ok$' e178-leaf-and-record-death-counts-unit-tests.out` 输出 32；同一份文件第 39 行 `test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.43s`）。钉绝对值的有：K1、K2、K3、a2 与净荷容量；5.6 各族的形数；K-2、BN1-s20、BN1-s30 的判定；稀排布局的中点；a1（F1 n = 300 发布数 607、F8 一段 13、F3 五段与 E177（逐树不写墓碑的复活数） 产物相同）；a5（θ = 0 两条写流相同）；a6（一借小历史）；a8（F1 n = 300、容量 16 退役 18 条）；a3（F8 n_u = 3 删前覆盖写，g0 的 extent r = 2、Z 与 T 是 0）；a7（F8 n_u = 300、b = 144，E-I 途中 extent m 300、156、12）；停机对拍 S1–S6；环窗 23 与 24 的边界；父链跳过回退抛弃的那一段；判定门槛挪动由绿转红；轨迹的峰值、为正次数、期末值。对拍类：按组索引择新与整扫择新在三段历史末尾相同；第二段一个工作项（F10 n = 300）上 PC-Z、PC-T、PC-EL、PC-ELr、PC-M 两路对得上、登记的期望成立；第三段 P-1、u = 3、b = 1 的三个崩溃点两路对得上、PC-O 两项成立。

变异表 `scripts/mutation/tables/e178_leaf_and_record_death_counts.tsv` 15 条（登记第九节 M1–M15；锚点各恰好命中一次）：抓到 15、无效 0、没红 0；`💥`、`⚠️`、`⏱`、`🧱` 各 0；收尾第 18、19 行「计数：内存撞顶 0 条（上限 6G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条」「已还原，基线仍全绿」（`e178-leaf-and-record-death-counts-mutations.out`）。M3 在登记写的 F11 取样点上不敏感（修订 1），由直接构造那条单测抓到；M7 只由 F10 合并历史那条单测抓到；M10 由 F12 P-1、u = 145、b = 1 的崩溃支那条单测抓到。

22 条全抓只说明写了断言的那几处被变异测过：判定行的组装（`segment_one_cell_verdict_lines`、`shape_verdict_lines`、`segment_two_verdict_lines`、`segment_three_verdict_lines`、第四段的 `new_base_verdict_lines`、`side_by_side_lines`、`stub_leaf_verdict_lines`、`segment_four_summary_lines`）、稳定性比对（`stability_lines`）与项文件的读回、聚合（`aggregate_segment_four`）没有变异动它们。

```

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:586-594`（整段抄，未转述）**

```markdown
### 岔路表（`m3-leaf-death-forks.md` 的 X1、X2、X3、X4 四行）

| 岔路单的那一行 | 已够判 / 还差什么 | 剩下的量能不能让它翻面 | 产物里算出它的命令 |
|---|---|---|---|
| X4：I-9.4（容器号不超最小 key） 判据与借怎么配（n1 / n2） | 已够判。翻面观测两样都量到：n2 每次借多写码 3 一个、码 2 一个、退役记录一条（四个 θ > 0 的格，借 12、102、222、1505 次，每次都是；占全部写单元 129–33003 ppm）；n1 漏判的坏镜像形态 8 形（并上 I-9.12（分隔 key 落在孩子区间之外） 之后 7 形，其中 BN1-s30 是 n1 的正确形态），分裂时号给错那一类（K-2、S 族三形）在 n1 的 I-9.4（容器号不超最小 key） 与 I-9.12（分隔 key 落在孩子区间之外） 下都绿。够判条件两样都齐：多写量在带借的历史上量到（F10 每个 θ > 0 的格都有借过的段，V6 只拿掉没借的段）；44 形 × 两种键布局逐形列全、两种布局逐形同判 | 不能。第二、三、四段不量 X4 的量（第四段的 B-stub 只配 n1，是问法一 n2-none 定的：选 B-stub 时 X4 只剩 n1，这是两行之间的耦合，不是量出来的翻面）；第二段 Q-B1、Q-B2 若在某一格打中 FL1，那一格的 X4 数按登记不拿来判，但每次借 1 + 1、退役 1 条与 5.6 全表不随读者、也不随格变 | `grep -E 'question=(Q-X4-1\|Q-X4-2\|Q-X4-3\|Q-X4-4\|Q-X4-5\|Q-X4-6\|Q-X4-7\|G-K) ' research/results/e178-leaf-and-record-death-counts-segment-one.out`；形态逐行 `grep 'name=shape '` |
| X2：文件变短那种死亡在盘上怎么留（Z / T） | 在 B-on 底座下已够判：H51 的 51 段、F10 的借与吸收（每个 θ > 0 的格都有借、被挪过又变短、被挪过又被 unlink 的段）、F11 上，Z 与 T 的复活（Q-X2-1、Q-X2-2）与不在途中的误判死亡（Q-X2-3、Q-X2-4）在 48 个判定格 × W 两档上都是 0，E-L 两臂下途中变短误判（Q-X2-5）也都是 0，四个旋钮不翻面；区间墓碑每次多写码 3、码 2 各 1（Q-X2-6），多写发布占比按族 0–766891 ppm（Q-X2-7）。两个候选在复活、误判死亡上都不输，差别只剩 T 的写代价。前提：V2 字面打中的 PC-B1 三段、PC-M（判法比登记松）与 V8 那几次途中扫描交主 agent 认；拿掉它们判定不变。换底座之后也够判：第四段 Q-Bs-4（B-stub，n1 的 12 个判定格 × Z / T × 三条 X3 臂 × W）384 行、Q-Bt-4（B-tsz，24 格 × Z / T × E-I × W）192 行都是 `differs=false`，FL-Bx 名单空，X2 的判定与这三个底座无关 | 不能：登记的量都跑完了。没覆盖的组合只有 B-stub × n2（问法一定的不配）与 B-tsz × E-L（臂定义不配），不是没跑的量 | `grep -E 'question=Q-X2-[1-7] ' research/results/e178-leaf-and-record-death-counts-segment-two.out`；`grep 'name=stability question=Q-X2' …`；`grep -E 'question=(Q-Bs-4\|Q-Bt-4\|FL-Bx) ' research/results/e178-leaf-and-record-death-counts-segment-four.out` |
| X3：整对象墓碑从哪一次发布起生效（E-I / E-L） | 已够判（B-on 底座下）：漏活那一半 E-L-w、E-L-r 的途中 inode m、extent m 在 48 格 × W 都是 0，E-I 照定义每次途中扫描漏 1 条记录与剩下的全部单元（峰值 1、300）；崩溃那一半 5448 个崩溃点上 E-L 两臂的漏活都是 0，o2 在 E-I 下都是 0、E-L 两臂下每个崩溃点都是 1（FL6 照字面打中，是臂定义的推论），恢复之后的复活峰值都是 0。漏活那一半换底座之后不变：B-stub 下 E-L-w、E-L-r 的途中漏活（Q-Bs-3）192 行都是 0，E-I 照定义峰值 1、300，FL-Bs2 没打中。B-tsz 只配 E-I（臂定义），选 B-tsz 就等于 X3 取 E-I，这是两行之间的耦合，不是量出来的翻面 | 漏活那一半不能。崩溃那一半在两条新底座下没跑（登记修订 2 之五写明第四段不跑 F12），B-stub 下 E-L 两臂崩溃点的漏活与 o2 没有数；登记只在主 agent 判它能让 X1 翻面时另续 | `grep -E 'question=Q-X3-[123] ' …segment-two.out`；`grep -E 'question=Q-X3-(o2\|4\|5) ' research/results/e178-leaf-and-record-death-counts-segment-three.out`；`grep -E 'question=Q-B[st]-3 ' research/results/e178-leaf-and-record-death-counts-segment-four.out` |
| X1：容器身份消失的死亡在盘上怎么留（底座：退役记录 B-on / 零条目版本 B-stub / 盘上零新增 B-tsz） | 正确性那一侧已够判：三个底座在 H51、F10（借与吸收）、F11（回退）同一组历史上，复活与误判死亡都是 0（B-on：Q-B1、Q-B2；B-stub：Q-Bs-1、Q-Bs-2 两档 1080 行；B-tsz：Q-Bt-1 的 inode r、extent r 差与 Q-Bt-2 共 624 行），B-tsz 的容器 r 按格 1–166 是定义的推论；unlink 途中漏活 E-L 两臂在 B-on、B-stub 下都是 0，E-I（B-tsz 只配它）照定义漏；阳性对照 PC-B2-direct、PC-Bs2、PC-Bt3 三个回退构造都成立。B-stub 的代价量到：叶片总数峰值最多 66、期末桩数最多 56、期末桩占比最高 1000000 ppm、撞 I-9.13（类型 2 容器不空） 的版本占发布 1056–670710 ppm，没有一版超过 135 片。还差：翻面观测里「每次删空 / 合并多写的单元」三个底座并排的数——登记修订 2 没登 B-stub 写路径的写单元，产物里没有（B-on 的退役写在第一段 `name=write_path` 行里，B-tsz 对底座零新增）；两条新底座下的崩溃那一半（F12）没跑 | 能：三个底座的复活、误判死亡一样（都是 0），这一行的取舍只剩代价那一侧，B-stub 每次删空、吸收写几个单元（原位写一个桩、右边各片不左移重写）与 B-on 的退役写谁多，没量，量出来可能让它翻面；崩溃那一半同理没数 | `grep -E 'question=(Q-Bs-[1-6]\|Q-Bt-[1-4]\|FL-Bx) ' research/results/e178-leaf-and-record-death-counts-segment-four.out`；`grep 'name=side_by_side' …segment-four.out`；`grep -E 'question=Q-B[12] ' …segment-two.out` |

```

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:595-610`（整段抄，未转述）**

```markdown
### 它答不了的

- 第一段只跑写路径、不扫描；X2、X3 与底座 B-on 的数在第二、三段，B-stub、B-tsz 的数在第四段。
- B-stub 写路径每次删空、吸收写几个单元没量（登记修订 2 没登这个量，产物里只有桩的个数），三个底座的写代价没法并排；X1 的翻面观测里这一项还缺。B-stub 只配 n1、B-tsz 只配 E-I，B-stub × n2 与 B-tsz × E-L 没有数；两条新底座下的崩溃那一半（F12）没跑。
- 三个底座复活、误判死亡都是 0 是这个模型里的数：B-stub 的读数与 B-on 逐行相同、B-tsz × Z 与 B-off-ret 逐行相同，都是旁证同一个结论——在读者照登记 5.4 择新、旋钮 W 只取 ∞ 与 23 的模型里，三种写死亡的办法让读者读出的东西一样；模型里没有分配器、没有 extent 树、没有并发、没有码 2 根长高（B-stub 的叶片总数最多 66 片，没碰 135）。
- 第四段的 `replay.sh` 那一行（`segment-four --no-reuse`）没复跑：本机 6 线程推约 35 分钟（推的：拼产物那一趟 997 项在 14 线程上约 15 分钟），执行员前台跑不完，交主 agent；第四段的确定性只由每项两遍逐次相同、抽 20 项在另一个项目录重算逐字节相同、V-Bt2 三样看。
- 「Z、T、E-L 两臂在 48 格上都是 0」是这个模型里的数：读者择新照登记 5.4、旋钮 W 只取 ∞ 与 23，模型里没有分配器、没有 extent 树、没有并发；同一个 inode 号在这些历史里只有一个对象出生代（号不在崩溃恢复之后重发），Z 的「对象出生代 = b」那一项没有取样点（登记第九节末段）。
- FL6 打中的那一格（E-L 两臂崩溃点上 o2 = 1）是臂定义的推论，不是从模型里量出来的新事实；E-I 的 o2 = 0 同理。
- 第二段的 PC-M、PC-T 判法比登记写的松（见作废条款那一行），那两项对照只证明了「两路一致」与「Z、g0 不动」，没证明登记写的确切增量。
- 坏镜像是登记 5.6 的 44 个枚举点加一种稀排布局，不是定义域；「n1 漏判 8 形」只说这张表上的数。
- 每次借多写 1 + 1、退役 1 条是 n2 臂定义的推论，模型照定义写出来就是这个数；它证明的是模型照定义写了（a6 钉住），不证明 n2 在 `crates/` 里落地之后的代价。比例的分母是读法 8 的口径，别的口径（含 extent 树、分配记录、记账）没算。
- 借、吸收、n1、退役记录这一页的装置跑的时候在 `crates/` 里都没有，`crates/singlefs-core/src/inode_tree.rs` 已有吸收与借（n1：分隔 key 取右片这一版的最小 key），`crates/singlefs-core/src/retirement_tree.rs` 已有退役记录，I-9.4（容器号不超最小 key） 第三句已删（n2 没选，不落地）；当时与 `crates/` 对拍的只有 θ = 0 的写路径（S1–S3）与当时的 I-9.4（容器号不超最小 key）、I-9.12（分隔 key 落在孩子区间之外） 判法（S6），都是读源码与测试断言之后在装置里写成同样的判定，没编译、没跑 `crates/`，对拍没重做。
- 叶容器容量 16 是模型自定的只供测试的旋钮；θ 的两档是登记给的点，借的多少随删序变（V6 那几段一次借都没有）。
- 修订 1 记的那件事（F11 上 PC-B2 量不出 B-off-lin 的容器 m）在第二段全部 F11 上都照样：48 项 `b_off_lin_container_m_peak_after_rollback=0`，主 agent 定不当结果。
- 第四段的挂钟没进产物：项文件的修改时刻从第一个到最后一个约 15 分钟（本机 14 线程，第二台那一片没跑成、由本机补算）。

```

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:611-627`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 写路径模拟 | 一次发布一次 `prepare`：照登记 5.1 的规则改叶容器、文件、墓碑树、退役树，数这次写的单元 | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:1725 prepare`、`:1941 adopt` | 格式常量（233、815、959、135、32634）；合并策略 M(θ)；批大小 |
| 2 | 闭式发布数（a1） | F1 = 3 + ⌈(n − 1) ÷ 50⌉ + 2(n − 1)；F8 = 3 + (k + 1) + [删前覆盖写] + 1 + max(1, ⌈意图时单元数 ÷ b⌉)；F3 抄 E177（逐树不写墓碑的复活数） 产物那两段 | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:3094 run_history`（第 3107、3126、3157 行的判定）、`:2972 E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS` | 批大小 144、一步建 50 个；F3 用同一个 splitmix64 |
| 3 | 一借小历史的手算（a6） | 登记第十三节 `anchors2.py` 的输出（挪 5 条、左 12 右 11、右片最小 22）抄成断言 | 无实现：手算在登记里，装置只有断言 `:3489 single_borrow_anchor_holds` | 容量 16、θ = 8、借的条数式子 |
| 4 | 按组索引择新（主路径） | 每组的版本清单从新往旧找第一个候选 | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:2404 select_by_the_group_index` | 候选谓词 `:2335 is_candidate` 经 `:2351 enters_the_selection`；读者判定 `:2470 judge_reader` |
| 5 | 整扫择新（参照路径） | 盘上每个单元整扫一遍，每组按 (写序, 单元序号) 取最大 | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:2426 select_by_a_full_scan` | 候选谓词 `:2335 is_candidate` 经 `:2351 enters_the_selection`；读者判定 `:2470 judge_reader` |
| 6 | 桩的闭式（a11） | 第一个桩 = 3 + ⌈(n − 1) ÷ 50⌉ + 2(min(n, 2 × 容量) − 1)，有桩的版本 = 发布总数 − 它 + 1（登记 13.3 `anchors3.py`） | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:5873 first_stub_publish_and_versions_with_a_stub` | 一步建 50 个；第二片最先删空的读法 |
| 7 | B-stub 写路径与原写路径（V-Bs1） | 两条写路径各跑一遍，逐版比 txg、活叶摘要、数据单元摘要、合并动作与扫描 | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:5883 pairing_holds` | 写路径模拟（路径 1 的同一段 `prepare`，只是死亡写法不同） |
| 8 | PC-Bs1、PC-Bt2 的手算 | 抹桩读回的那一版里逐条记录数（PC-Bs1 inode r）；两条供出规则逐单元比（PC-Bt2 extent r） | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:5506 hand_count_of_records_a_stub_erasure_brings_back`、`:5609 hand_count_of_units_the_oldest_copies_bring_back` | 参照路径的择新（路径 5）；墓碑树的现行那一版 |

行号是第四段装置（源文件 sha256 `93193bd5929e7867b7d18ea5c8b0d6af9e32ceffed5c866f073ba8e5c0d1dfa6`，第四段产物就是它跑的）的；第一至三段产物是改之前那一版（`e0512d3f…1897`）跑的，改过之后第一段逐字节复跑相同。

跨路径的比对断言：路径 1 对路径 2 钉在 `run_history` 的 a1 判定（第二段 612 次、违反 0）；路径 1 对路径 3 钉在 `single_borrow_anchor_holds`（第一段产物第 1525 行 `holds=true`）；路径 4 对路径 5 钉在 `:4367 group_index_equals_the_full_scan` 与 `:4135 positive_control`（第二段 `v3_last_scan_paths_agree=true`、阳性对照 `paths_agree` 全是 true；第三段 `v3_paths_agree=true`）；路径 6 对路径 1 钉在 a11 的判定（单测里过；第四段产物 `assertion=A11_first_stub_publish_and_versions_with_a_stub judged=4 violated=0`）；路径 7 钉在 V-Bs1（第四段 498 项、违反 0）；路径 8 对读者钉在 PC-Bs1、PC-Bt2 的 `closed_form_holds`。共用项问句：路径 1 与 2 共用批大小与建 inode 的步长，两者一起错时 a1 照样对上，挡它的是 S3 对 `crates/` 的对拍；路径 4 与 5 共用 `is_candidate` 与 `judge_reader`，候选谓词或读者判定写错时两路一起错，挡它的是 PC-Z、PC-EL、PC-ELr、PC-O 写在登记里的期望与 M1–M22 那几条单测；路径 7 的两条写路径共用 `prepare` 里除死亡写法之外的全部，`prepare` 写错时两条一起错、配对照样成立，挡它的是 S1–S3 与 a1；路径 8 用参照路径的择新，择新写错时手算与读者一起错，挡它的是 V3 的两路比对。

```

**出处 `.claude/kb/experiments/178-叶容器与记录死亡的计数.md:628-638`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 6 | 支撑 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：归档上一轮记录只把指向已删文件的路径改成只留文件名，结论与产物不变；此前 2026-10-18 不受影响：单测与变异一节补点名 10-15 续跑键样本与项键证红的产物，路径与结论登记六行补上文件路径，结论、数不变 |
| D8（核心索引结构） 已定项 16 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：归档上一轮记录只把指向已删文件的路径改成只留文件名，结论与产物不变；此前 2026-10-18 不受影响：单测与变异一节补点名 10-15 续跑键样本与项键证红的产物，路径与结论登记六行补上文件路径，结论、数不变 |
| D18（块里携带什么信息） 已定项 11 | 支撑 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：归档上一轮记录只把指向已删文件的路径改成只留文件名，结论与产物不变；此前 2026-10-18 不受影响：单测与变异一节补点名 10-15 续跑键样本与项键证红的产物，路径与结论登记六行补上文件路径，结论、数不变 |
| D18（块里携带什么信息） 已定项 24 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：归档上一轮记录只把指向已删文件的路径改成只留文件名，结论与产物不变；此前 2026-10-18 不受影响：单测与变异一节补点名 10-15 续跑键样本与项键证红的产物，路径与结论登记六行补上文件路径，结论、数不变 |
| D18（块里携带什么信息） 已定项 10 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：归档上一轮记录只把指向已删文件的路径改成只留文件名，结论与产物不变；此前 2026-10-18 不受影响：单测与变异一节补点名 10-15 续跑键样本与项键证红的产物，路径与结论登记六行补上文件路径，结论、数不变 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：归档上一轮记录只把指向已删文件的路径改成只留文件名，结论与产物不变；此前 2026-10-18 不受影响：单测与变异一节补点名 10-15 续跑键样本与项键证红的产物，路径与结论登记六行补上文件路径，结论、数不变 |

```
