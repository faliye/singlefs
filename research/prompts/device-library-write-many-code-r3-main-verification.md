# device-library-write-many-code-r3：判决（2026-11-10）

正文 `research/prompts/_device-library-write-many-code-r3-body.md`，开工快照 `research/prompts/device-library-write-many-code-r3-snapshot/`。这一题的第三轮，也是最后一轮；前两轮判决 `research/prompts/device-library-write-many-code-r1-main-verification.md`、`research/prompts/device-library-write-many-code-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（sonnet） | `research/prompts/device-library-write-many-code-r3-sonnet-output.md` | r2 第三节四条改法都修到；第二节读数在日志里逐字找得到；「第二节的任务都派在第二台跑过」不成立 |
| 攻方（opus） | `research/prompts/device-library-write-many-code-r3-opus-output.md` | 前两轮站住的形态都站住；另报甲到己六处，都标零轮（前一条攻方腿死在网络中断里，落了 129 行；接着做的腿一字没改前 129 行、续写到 356 行） |

- `snapshot-tree.py device-library-write-many-code-r3`（任务 12401）：`✓ 快照 device-library-write-many-code-r3：清单点名 4 份，倒推出 4 份，sha256 对上 4 份`。
- `cite-check.py` 两份腿报告，`--root` 给倒推出的树：判出两处 ✗，都在正推腿报告里、都是测试代码里一句的行号：`device-library-write-many-code-r3-sonnet-output.md:51 引 lib.rs:709「回应认不出」：那一行不含这句；原文在这份文件第 15, 30, 134 行`、`device-library-write-many-code-r3-sonnet-output.md:135 引 lib.rs:993「这一步在测试里为什么成立」：那一行不含这句；这份文件里找不到这句`。两处都不撑它的判定（第一处的判定「两条路的话分得开」主 agent 现读 `lib.rs` 第 218–231 行坐实；第二处是 `expect` 措辞的旁注）。其余引文对上或没判，没判的多是只写 `lib.rs` 不带路径。
- 复跑（主 agent 的读数，量的）：
  - 任务 12399（`rounds.sh` 在快照那一版上）：`name=variant_summary variant=unmutated named_test=- named_failed=0 any_failed=0 of=5`；四个放锁变异各 `named_failed=5 any_failed=5 of=5`。
  - 任务 12317（第 7598 行按门禁点名）：`计数：抓到 1 条、没红 0 条、无效 0 条、锚点失效 0 行（共 1 条）`。
  - 任务 12400（攻方模型复跑，副本从主工作区现建，`name=frozen_library sha256=773f90a766cb5826a81ffe87d14345c56f49d1876b7a99100c20ea33d3b8bff8`）：
    - 未改：`in_memory_write_with_a_key_the_relay_refuses_exits_with_the_other_error_code=0/3`、`read_answered_failed_exits_with_the_code_in_the_reply=0/3`、`read_answered_found_with_contents_that_are_not_utf8_gives_none_and_takes_exactly_those_bytes=0/3`、`read_answered_found_touches_no_more_memory_than_the_contents_that_arrived=3/3`（红的这一条就是乙：`name=resident variant=unmodified round=1 name=resident_growth_before_contents announced=33554432 growth=17100800`）
    - 变异 `write-key-checked-in-the-relay-arm-only` 下内存单条写坏键那条 `3/3`；`read-answered-failed-taken-as-absent` 下读 failed 那条 `3/3`；`contents-not-utf8-decoded-lossily` 下非 UTF-8 那条 `3/3`
    - 变异 `write-lock-released-between-header-and-contents`：`a_write_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply=0/5`
    - 改法 `read-contents-without-filling-zeros-first`：`read_answered_found_touches_no_more_memory_than_the_contents_that_arrived=0/3`，三轮 `growth=1302528`、`growth=4235264`、`growth=2777088`

## 二、判决

| 格 | 判 | 依据 |
|---|---|---|
| r2 第三节四条改法 | 修到 | 正推逐条给 文件:行；12399、12317 在快照那一版上复核 |
| 攻击面 1「已定」项 | 前两轮站住的形态都站住 | 两条腿一致 |
| 甲：内存那一种的单条 `write` 坏键无测试 | 打中（测试缺口，行为对） | 12400：未改代码上那条测试绿、变异下 3/3 红 |
| 乙：`try_reserve_exact` 之后 `resize` 写零，回应报大长度时内容到之前就摸遍整段 | 打中 | 12400：报 33554432 字节，未改涨 17100800 字节，去掉写零涨 1302528–4235264 字节 |
| 丙：写的锁在请求头与内容之间放过，探锁测试罩不到 | 打中（测试缺口） | 12400：那个变异下写的探锁测试 0/5 |
| 丁：`read` 收 `failed` 无测试 | 打中（测试缺口，行为对） | 12400：未改绿、变异 3/3 红 |
| 戊：非 UTF-8 交 None 无测试 | 打中（测试缺口，行为对） | 12400：未改绿、变异 3/3 红 |
| 己：12232–12234 跑在别名之前那一版 | 关 | 12399 在快照那一版上四个放锁变异各 5/5、未改 0/5 |
| 攻击面 3 数字 | 读数都找得到；正文「都派在第二台跑过」写错 | 只有 12232、12235 在第二台 |
| 检查单八格 | 七格成立；「所有任务多机可用」读码成立，正文前提写错 | 同上一行 |

## 三、改法

第三轮之后不再开轮。甲到戊都是这一轮新冒出来的，被攻过零轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」写进交用户表。

- 甲、丁、戊：只补测试与变异行，行为不改；零轮，A/B 读数任务 12400（未改绿、变异红）。
- 乙：`read` 读内容不先写零（`read-contents-without-filling-zeros-first`）；零轮，A/B 读数任务 12400（未改 17100800 字节、改后 1302528–4235264 字节）。
- 丙：写的探锁测试补「请求头与内容之间」那一段；缺口是量过的（任务 12400 那个变异 0/5），补法零轮、还没量。

## 交用户的

| 项 | 零轮 | 建议 |
|---|---|---|
| 甲 内存单条 `write` 坏键测试 | 是 | 补测试与变异行，并进下一批实现员 |
| 乙 读内容前写零 | 是（A/B 任务 12400） | 改成不先写零，并进下一批 |
| 丙 写的锁在头与内容之间放过 | 是 | 补探锁造法，先量再并 |
| 丁 `read` 收 `failed` 测试 | 是 | 补测试与变异行 |
| 戊 非 UTF-8 交 None 测试 | 是 | 补测试与变异行 |

## 这一轮被判过的文件

- crates/singlefs-device-library/src/lib.rs：r2 判决之后的修补（`read` 核键、`found` 长度的可失败分配、四条探锁测试、文件头协议）；判了 r2 四条改法、三条攻击面与检查单八格，前两轮站住的形态都站住，另记五处零轮缺口交用户。

## 回看决策

不涉及决策：改动是实验装置读写判定库的口子，不碰 `.claude/kb/decisions/` 里任何一条的条款与依据。
