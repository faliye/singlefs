# 回收门槛取一圈窗口还站不站得住：第一轮正文（2026-10-04）

<!-- doc-lint:not-numbers K1 W1 W2 W3 G2S G2SW -->

## 一、题面

D16（发布语义） 已定项 1「可再分配」一行把回收门槛定成 max(F_生效, 一圈窗口里最旧有效根)（`.claude/kb/decisions/16-发布语义.md:33`，窗口的定义在同一份文件第 35 行）。窗口是 `research/prompts/m3-ring-r2-main-verification.md` 第二节第 4–6 行、`research/prompts/m3-ring-r3-main-verification.md` 第二节第 3–5 行两轮三方加的，那两轮的依据是三方副本上的量，没有入库装置（同一份文件第 73 行「无实验……E139 重跑欠着」）。

E139（按盘回退下界的收严形态） 第 1 次重跑把窗口加进入库装置（登记 `research/prompts/e139-r1-prereg.md`，产物 `research/results/e139-tightened-floor-2026-10-04.out`，实验页 `.claude/kb/experiments/139-按盘回退下界的收严形态.md`「### 第 1 次重跑」一节）。主 agent 现数产物的 `name=w_activation` 行：臂 G2SW（门槛取窗口）60 行里 46 行 `fakes` 大于 0、3 行 `unrecoverable` 大于 0（skip_then_raise_r2 / r3、S = 4、丢盘 0）；门槛换回「环里最旧有效根」的对照臂 G2SW_ringgate 60 行全 0。装置里窗口之外的有效根只由一种情形造出：根槽写失败之后 checkpoint_txg 推进一格再发（`.claude/kb/decisions/23-journal的角色与格式.md:399`，目标形态）；今天的实现照同一份文件第 402 行逐字节原样重发、不推进 txg。执行员推「今天的实现因此不留这种洞」，没量过；而窗口当初要罩的 K1（取号与写行跳过的 txg 留下的洞）就是今天的实现会留的洞（`research/prompts/m3-ring-r2-main-verification.md` 第二节第 1、4 行）。

用户 2026-10-04 弹窗定：现在开三方重判回收门槛。

## 二、实现今天的样子（主 agent 的观测，2026-10-04）

- 分配器的回收门槛：`crates/singlefs-core/src/allocator.rs:1041` 的 `oldest_valid_root_in_the_window`、`:1710` 的 `reclaim_released_up_to`；挂载重建分配器时 `crates/singlefs-core/src/mount.rs:1347` 的 `reclaim_floor`（单测 `:6584` `reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root`）；抬 F 那一遍 `mount.rs:2866` 的 `oldest_valid_root_in_the_window_for_raising_the_floor`。
- 根槽写失败的处置：`crates/singlefs-core/src/transaction.rs:1250` 的 `resend_the_frozen_publish`，冻结之后原样重发（同一个 txg）。
- 池级 checker 同取窗口的两处：I-3.1（已分配统计对得上） 丁-defer 门槛与候选过滤，`crates/singlefs-checker/src/walk.rs` 里现查（写回丁核过候选过滤 `outside_the_window` 在第 8097 行一带）。
- 删除与并发那条线第 16、17 批在改 `mount.rs` 的 `rebuilt_allocator` 当场回收（m = min(F_生效, 每块盘最新有效根)；m 取不到时只放到窗口里最旧有效根，用户选甲，`research/prompts/m3-batch16-code-r1-main-verification.md`）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### W1　今天的实现会不会留下「窗口之外仍有效的根」
在今天的 `crates/` 上（原样重发、不推进 txg），哪些合法历史或读坏根槽的镜像会让根环里有一条窗口之外、自证合法、按实例表仍有效的根：K1（取号、写行跳过的 txg）、连崩两次写行、挂载那一刻读不出的槽、被抛弃的根、两块盘根环不齐。有的话，那一形上 E139 r1 的假候选与不可恢复会不会在实现上出现（候选集里的根引用了已被复用的单元，回退到它读出坏数据）。

### W2　E139 r1 的 46 / 3 是不是装置造出来的
装置对 D16 表逐条比有 5 处丙（实验页「### 第 1 次重跑」一节的对照表）。逐处判：哪一处丙会单独造出 G2SW 的假候选或不可恢复（装置的窗口锚、窗口长度、「有效」「非空」的判法与条款不同的那几处）；照条款改掉那几处之后，46 / 3 还在不在。

### W3　回收门槛取什么
至少三个候选，各写明它在 W1、W2 那几形上判成什么，以及它会不会重新打开窗口当初要关的那一形（`research/prompts/m3-ring-r2-main-verification.md` 第二节第 4–6 行：K1 后洞上候选集里的根引用已复用的单元、只改分配器时 checker 的 I-3.1 误红）：
- 今天的窗口照留；
- 退回「环里最旧有效根」（E139 的 G2SW_ringgate）；
- 窗口加一项修正（例：窗口长度随跳过的 txg 数加长，或锚取别的量），腿自己提的照写。
每个候选连同 D16 已定项 1 同取窗口的另外五处（同一份文件第 35 行末句）要不要一起改。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | W1、W2、W3 | 从已定条款、E139 r1 的产物与登记、今天的实现逐句推出每一格该怎么写，推不出的写明 |
| 云端攻方（Opus） | W1、W3 | 在冻结副本的拷贝上造今天实现会留下窗口外有效根的历史，以及每个候选的假候选、不可恢复、checker 误红；能写原型的就写（只跑 harness 档） |

## 五、跑前写死的判据

- 一个候选在一段合法历史上（今天的实现，或 D23 第 399 行的目标形态）让回退候选集里出现引用了已复用单元的根（假候选）、或回退之后读不出原来的数据（不可恢复）⇒ 那个候选出局。
- 一个候选重新打开窗口当初要关的那一形（K1 后洞上候选集里的根引用已复用单元，或只改一处时 checker 的 I-3.1 在合法盘面上误红）⇒ 那个候选出局。
- W1 两条腿都造不出、且只抽了一次样 ⇒ 写「一次抽样没造出」，不写「今天造不出」。
- 候选都出局的那一格 ⇒ 判决写「推不出」，交用户；剩下不止一个 ⇒ 写岔路单，每个候选带代价（多放还是少放多少槽、改几处），交用户。
- 腿提的新形态算零轮。

交付：腿的报告写 `research/prompts/m3-reclaim-window-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-reclaim-window-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
