# kb 写回规格第五批：起草报告（坏盘输入的 panic 面实审 A3a 收口）

写于 2026-09-27。规格文件：`/tmp/claude-1000/kb-batch5-drafter/spec.json`、`/tmp/claude-1000/kb-batch5-drafter/spec.md`（两份内容等价，`kb-spec-check.py` 都判过）。

## 结论

- 7 条大项，没超 8 条，出一份即可（没有 batch5a/b 之分）。
- `python3 research/scripts/kb-spec-check.py /tmp/claude-1000/kb-batch5-drafter/spec.json` 与同目录 `spec.md` 末行都是：
  `  ✓ 规格 7 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 7 条）`
- 7 条对应：D22（单元原子性怎么合成） 已定项 2、D23（journal 的角色与格式） 已定项 18/19、D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 10 各一条决策文件改动；`.claude/kb/checks-owed.md` 两条（C476 现状更新一条、追加 C581 + C582 一条）；`.claude/kb/decisions-history/2026-09.md` 一条（合并写入四个新条目：其十、其十一、其十二、其十三）。
- 欠账号：现查 `.claude/kb/checks-owed.md`（2026-09-27 磁盘状态）最大号是 C580，取 C581（extent 偏移非 0 没有判据）、C582（挂载盘容量下界没条款该取哪个）。文件当时有未提交改动（`git status --porcelain` 显示 M），取号按磁盘现状，不是仓库某个提交；kb-scribe 实际写入前应重新核一次最大号（`.claude/singlefs-ai-sop/rules/session-wrapup.md`「公共编号先到先得」）。
- 决策变更史编号：现查 `.claude/kb/decisions-history/2026-09.md` 今天（2026-09-27）已有条目最大是「其九」，取「其十」「其十一」「其十二」「其十三」给这四条，插在现有「其八」之上（同一批合并写入，锚点是「## 历史版本」紧跟着的「### 2026-09-27（其八）：……」整行）。

## 出口路径

按 kb-spec-drafter 定义「写范围」，我不写仓里任何文件，只写草稿目录与报告。`research/prompts/m2-kb-writeback-batch5-spec.md` 由主 agent 在核过草稿之后落盘（形态已照 `research/prompts/m2-kb-writeback-batch4a-spec.md`：每条「文件、旧串、新串、依据」，`spec.md` 就是这个形态，可以整份搬过去）。

## 7 条大项一览（文件、依据）

1. `.claude/kb/decisions/22-单元原子性怎么合成.md`：已定项 2 射程补一句读者判根槽宽与固定结构槽距的界（槽距 ∈ [4096, 1 MiB − 4096]，根槽宽 ∈ [457, 槽距]）。数值现查 `crates/singlefs-core/src/recovery.rs`（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES` = 4096、`largest_fixed_structure_slot_spacing_the_format_allows` = `region_start(0) − 4096`）与 `crates/singlefs-format/src/lib.rs`（`ROOT_RING_BASE_SLOT * SLOT_BYTES` = 1 MiB）；依据：`research/prompts/m2-rev-a3a-implementer-report.md` 第四节「条款要补的原句」第 1 条。
2. `.claude/kb/decisions/23-journal的角色与格式.md`：已定项 19 射程补一句读者判环长的界（在飞上限 ≥ 1，环末端 ≤ 768 MiB）。768 MiB 现查 `crates/singlefs-format/src/lib.rs` 的 `UNIT_AREA_START_SLOT`（50176）、`JOURNAL_RING_START_SLOT`（1024）、`SLOT_BYTES`（16384）：(50176 − 1024) × 16384 = 805306368 字节 = 768 MiB。依据：同报告第四节第 2 条。
3. `.claude/kb/decisions/19-块指针的结构与宽度预算.md`：已定项 3 射程补 MAC / nonce 恒 0、非 0 判损坏；extent 偏移那 2 字节记进新欠账 C581。依据：同报告第四节第 3 条。
4. `.claude/kb/decisions/09-加密.md`：已定项 10 射程「读路径跳过它们，池级 checker 不判它们」改成现状（core 读路径今天判单元头 29 字节、系统配置加密类型、两处指针头部；池级 checker 仍不判，写「checker 那一半实现在做」）。依据：同报告第四节第 4 点、「三、checker 那一侧」。
5. `.claude/kb/checks-owed.md`：C476 那一行按 A3a 实际改动更新现状——判全接上 rebuild_version 使分配器族 R6–R9 接上真实路径、缺口表 pointer.rs 升序断言已按 I-2.5 判；列出 A3a 清单外仍开着的（②条目宽度族 8 处、R11–R13、缺口表另三处、清单外还没换上的指针读者）；补一处判全带来的新发现（`transaction.rs:6339` 树表条目树 ID 次序不判，水位以下换序仍可 panic，拦不拦交主 agent）。依据：同报告第三节 C476 行、第七节、第十一节。
6. `.claude/kb/checks-owed.md`：追加 C581（extent 偏移非 0 没有判据）、C582（挂载盘容量下界没条款该取哪个，写明今天实现取「完整槽数 ≥ 单元区起始槽号」）。依据：同报告第四节第 4、5 点。
7. `.claude/kb/decisions-history/2026-09.md`：一次插入四条（其十 D22、其十一 D23、其十二 D19、其十三 D9），各带「### 」标题、两行快查、改前/改后/依据三条。依据：同报告第四节「条款要补的原句」全五条。

## 规格自检细节

- 每条旧串在目标文件里 `grep -cF`/等价的 python `count()` 核过恰好 1 次（脚本内断言，全部通过，见 `build_spec.py`）。
- 新串里的编号一律「编号（简称）」：D22（单元原子性怎么合成）、D23（journal 的角色与格式）、D19（块指针的结构与宽度预算）、D9（加密）、I-2.4（头校验和覆盖范围）、I-2.5（位置条目按设备身份升序）、I-7.8（根记录树 ID 水位不低于全池最大树 ID）。R6–R9、R11–R13、R2、R7、R10 这几个记号已被 `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md` 的 `doc-lint:not-numbers` 摘出去，不算编号引用，与 `checks-owed.md` 现文里 C476 那一行已有的用法一致。
- 踩过一次坑：C476 更新草稿里 `InvariantViolated { invariant: "I-2.5" }` 那句 Rust 代码引文，字面上让 "I-2.5" 裸出现（前面跟的是引号不是括号），`kb-spec-check.py` 判红一次；改写成不重复裸拼这个记号（说「`invariant` 字段填这条不变量的名字」），保留同一处已经带名字的 I-2.5（位置条目按设备身份升序） 引用，改完判绿。
- 没有改动任何决策的「**依据**」段、没有增删任何 E<数字> 引用，所以没有配套的实验页「### 影响的决策」表改动（`kb-spec-check.py` 的第 ④ 项判据本身没有触发，不是漏做）。

## 要主 agent 判的点

- 无。A3a 报告「四、做了判断、交主 agent 核的几处」七点主 agent 已按先例定（都认），本批只是把这七点落成 kb 正文与欠账表，没有遇到两种读法互相矛盾、也没有定案句与 kb 现文冲突的情形。
- 惟一要留意的是取号时点：`checks-owed.md` 与 `decisions-history/2026-09.md` 当时都带别的会话未提交的改动（`git status --porcelain` 分别显示 M），本规格的 C581/C582 与「其十～其十三」是按 2026-09-27 起草时刻的磁盘最大号取的；kb-scribe 实际写入前按 `session-wrapup.md`「公共编号先到先得」重新核一次两处的当前最大号，号被别的批占用了就顺移，不是本规格内容有误。

## 没做什么

- 没写任何仓内文件（`.claude/kb/**`、`research/**` 一个字节都没改），只写了 `/tmp/claude-1000/kb-batch5-drafter/` 下的 `spec.json`、`spec.md`、`build_spec.py`、这份报告。
- 没跑 `kb-scribe` 的门禁阶段（21、30、49、75 号等），那是书记员写完之后的活。
- 没有另做推论：D22/D23/D19/D9 四处新句子的数值（457、4096、1 MiB、768 MiB 等）都现查了对应源码常量，没有凭印象补；D9 的「checker 那一半实现在做」按主 agent 派发提示原句抄录，没有自己判断实现进度。
- 没有清理 `/tmp/claude-1000/kb-batch5-drafter/` 下的任何文件——这些正是要交主 agent 核用的草稿产物，不是本轮自建又该删的编译目录或仓副本。
