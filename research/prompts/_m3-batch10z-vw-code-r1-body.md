# 根环实现第 10z 批第二十二、二十三块：代码轮第一轮正文（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 B C D F H K R N -->

## 一、题面

这一轮攻两块：

| 块 | 照什么改 | 落点 |
|---|---|---|
| 第二十二块（core） | 判决 `research/prompts/m3-batch10z-tu-code-r1-main-verification.md` 第二、三节；用户 2026-10-09 弹窗定「改：预演延到要出路时才做」：报 `StuckBehindTheWideProtection` 时不预演，调用方要出路时另调一个方法（传盘表）算；`DeviceKeepingWritesInMemory` 的对齐判改成调 `block_device.rs` 那一份（开成 `pub(crate)`），不留抄本；补用例拦住上一轮攻方 B、D、F、H、K 与 K5 对齐判恒放行、正推活下来的只进内存的盘读回拼接、对齐越界、`walkSkipped` | `crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/block_device.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` |
| 第二十三块（checker 档测试） | 同一份判决 K4 那一行：测试模块切法要求头两行在代码里恰好一处；`#[cfg(...)]` 挂在钉住那一句上判红；按类型找层认 `+ Send`、`+ 'static`、`&mut`、`&'static`、夹注释、`Box<dyn …>`；误判与 panic 逐个改 | `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` 的测试模块 |

条款与用户原话：
- `.claude/kb/decisions/16-发布语义.md` 已定项 1 第 29 行（回退候选集保留最近 4 个可退到的不同状态）、第 36 行「抬 F 的上限」那一行、第 37 行「根槽这一次读坏」那一行；`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第 27 行第九项；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（冻结那一道）；`.claude/kb/invariants.md` 第 58 行 I-7.4（近 K 代块未被复用）；欠账 `.claude/kb/checks-owed.md` C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）、C762（抬 F 给根环表没记着的被抛弃根补的隔离一直不放）。
- 用户 2026-10-08 弹窗原话「显式推环命令加带出路的错」：不自动推环；推环只在用户显式调用时、调用即确认回退候选集掉到少于 4 个不同状态。用户 2026-10-09 弹窗原话「甲：预演推环与准入，推不通就不列」，之后同日「改：预演延到要出路时才做」。

改动全文：`research/prompts/_m3-batch10z-vw-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-v/patch/report.md`（第二十二块）、`~/.local/share/singlefs/work/m3-r1/batch10z-w/patch/report.md`（第二十三块）。上一轮两腿报告：`research/prompts/m3-batch10z-tu-code-r1-opus-output.md`、`research/prompts/m3-batch10z-tu-code-r1-sonnet-output.md`；上一轮攻方模型 `research/prompts/m3-batch10z-tu-code-r1-opus-model/`（`rerun.sh` 是整份复跑的起法，`mutants.py` 是 B、D、F、H、K 那几个变异的原文）。

## 二、实现今天的样子（主 agent 的观测，2026-10-09）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree/`，取在两块都合入之后；被判文件 sha256 在 `research/prompts/m3-batch10z-vw-code-r1-snapshot/crates-sha256.txt`，与合入目录 `~/.local/share/singlefs/work/m3-r1/merge-10z-vw/after.sha256` 逐个相同。
- 第二十二块关键位置（冻结副本 `crates/singlefs-core/src/mounted_session.rs`）：第 215 行 `StuckBehindTheWideProtection`（多记私有的被拒那一版实例代号与 txg、这次改动还剩什么的一份拷贝；字段 `pushing_the_root_ring` 与 `ways_out()` 删掉）、第 256 行 `WaysOutOfTheWideProtection`、第 266 行 `WaysOutOfTheWideProtectionRefused`（成员 `SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`）、第 280 行私有 `ways_out_after`（出路文字逐字未改）、第 325 行 `DeviceKeepingWritesInMemory`、第 346 行 `check_request`（调 `crates/singlefs-core/src/block_device.rs` 第 315 行开成 `pub(crate)` 的 `check_aligned_and_in_range`）、第 481、488 行 `RemainderOfTheUserChangeKeptByTheRefusal` / `UserChangeKeptByTheRefusal`、第 612 行预演 `rehearse_the_push_then_the_remainder_of_the_change`（本体没改）、第 753 行 `wide_protection_at_the_ceiling`（只收分配器、被拒那一版、还剩什么、停因，不读盘不拷分配器）、第 804 行 `RootRingPushRefused`、第 1215 行推环入口、第 1249 行新方法 `ways_out_of_the_wide_protection(&self, devices: &[(DeviceIdentity, Device)], stuck: &StuckBehindTheWideProtection)`、第 1453 行 `push_one_floor_raise_for_a_refusal_for_space`（删了只为预演收的 `instance` 参数）、第 1586 行新 `#[cfg(test)] mod tests`（只进内存的盘的单元测试与两万请求差分对拍）。`mount.rs` 没动，`refusal_is_short_of_space` 函数体逐字未改（实现员 `diff` 无输出）。
- 第二十三块：改动都在两份文件的 `#[cfg(test)] mod tests` 里；新用例 8 条、改了的老用例与函数、名单改动在实现员报告「结论」一节，行号是冻结副本的行号。
- 合后验证（`~/.local/share/singlefs/work/m3-r1/merge-10z-vw/summary.txt`）：merge-and-verify rc=0（编译、harness 档全部测试二进制、fmt、clippy）；mutation-tables 格 rc=0；static-check `CHECKED 3597 322 10`；e158 `95 passed`；e168、E171 装置编过，E171 sha 前 12 位 ab35105f861d；`raise_floor_takes_…` `--include-ignored` 59 passed、另两份慢档 23 / 16 passed、第九项相关 10 份全绿；checker 档快档 61 份 0 失败；带 gpu 的 lib 与 12 份挂 gpu 的测试全过、NOT_RUN 0；合后副本上 prove-red：checker 档 17 行、core 4 行、harness 档整行替换 5 行全抓到。实现员自己证过第二十二块追加 23 行里的 19 行（第 12、14、20、21 行没单证）、第二十三块 16 行只在抽出来的草稿小程序上证过（合后 prove-red 那 17 行含这 16 行）。
- 实现员自定、被攻过零轮的：
  - 第二十二块：报里带这次改动内容的一份拷贝（错误不借调用方的内容）；新方法收 `&self`、盘表只读借，盘表照推环入口那一套在预演里核，不另判；「会话已不在被拒那一版上」按实例代号与 txg 判，冻结着的发布不动 txg、照常预演；「报错那一步不预演」的读数取底下那块盘被探物理块大小的次数；F、H 两条从报的 `{:?}` 读私有字段；
  - 第二十三块：卡池 `use wgpu;` 类（来路就是它自己）不拒；块里的 `use` 只在那一对花括号里有钉住那两段的签名时才拒；条目层宏看前一个词（`;`、`}`、`]` 或模块开头）；后面紧跟 `<` 的 `use` 不当一条 use；原始标识符拼成关键字的连着 `r#` 出词；崩溃放量另照 `Fn() -> bool` 这个形（含 `FnMut`、`FnOnce`、`fn() -> bool`）找层；叫停参数名前面挂属性判红；钉住那一行、短层函数体上的行尾注释不再判红；卡池 `!=` 那条老用例头上加一句分词断言（不加则第 3551 行变异成等价）。
- 主 agent 已收、报出来不算打中的：要出路时会话已不在被拒那一版上，拒成 `SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`、不读盘（条款没写按哪一刻算）；上一轮攻方 F、H 推完之后那一段与正推 `walkSkipped` 实现员造不出（报告「停下交主 agent 的」第 1 条列了三种试过的造法），只钉到报里记着的「还剩什么」。
- 已知没做、不在这一轮被判范围的：C762、C554；只进内存的盘 `write_zeroes_at` 先分配整段长度（发布路径不写零）；正推上一轮 K4-2 另提的新层参数前有函数指针类型时层名读成空串、`#[cfg(any())] extern crate` 照拒。
- 用户已定、报出来不算打中的：tri 的判法本身与它在「不知道」那一支上多护的那一截；显式推环会让回退候选集掉到少于 4 个不同状态；不选乙（推完不抬 F，推环在长写上仍可能不够，只是不再列给用户）；预演在要出路时才做、每要一次整串推一遍环的代价（上一轮判决核查那一行的数）；两进程同读一份空闲的竞态；G2 乙。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　第二十二块 延后的预演

1. 延后之后的结局：要出路时交回的预演结局与出路，和上一版（报错那一刻预演）在同一形上是否逐格相同、与照出路二真推环再发同一个改动的真结局是否相同？上一轮攻方那几族形（`m3-batch10z-tu-code-r1-opus-model/` 里的 9 形 283 格、放开小动作、实现员三形、两槽 C、被抛弃根 A 与 C 同在、更窄四档单元区）在新接口上逐格对一遍。
2. 报里记的「还剩什么」：四种 `UserChange` 与走空、走空再发几支，记进报里的是不是被拒那一刻真剩下的那一份？有没有一形记错（内容、写入时间、inode、偏移）让预演预演的不是被拒的那个改动？
3. 「会话还在被拒那一版上」的判：只比实例代号与 txg，有没有一种状态变化（分配器、隔离、C 那一圈、冻结、读坏的根槽、卸载重挂换会话）不动这两个数而让预演用的已不是被拒那一刻的状态？反过来，有没有该能预演而被拒成 `SessionMoved…` 的形？
4. 报错那一步：签名改后，报错路径还碰不碰盘、拷不拷分配器？`&self` 加只读借的盘表，新方法能不能把会话、分配器或盘弄动（读盘出错、预演里抬 F 被拒、取不到落点）？

### K2　第二十二块 对齐判共用

`DeviceKeepingWritesInMemory::check_request` 调 `block_device.rs` 那一份之后，读、写、写零三条请求与文件后端判得是否相同（设备大小取底下那块盘的 `size_in_bytes()`）？两个文件后端经 `pub(crate)` 改动有没有行为变化？

### K3　判别力

第二十二块追加 23 行、整行替换 5 行，第二十三块追加 16 行：能不能拦住改回去？没单证的第 12、14、20、21 行点名的用例真红不红？注入级用例（预演里取不到落点那一形、两形「卡在走空里」）在自然历史里存不存在、钉的是不是条款说的那件事？F、H 推完之后那一段与 `walkSkipped`，能不能造出一形把它们分开（推环只要一两次就写到 R 那一槽、同时走空被拒，或换盘几何）？还有哪几种改坏的写法照绿？

### K4　第二十三块 扫源码

1. 新的放行（`use wgpu;` 类、块作用域 `use`、条目层宏看前一个词、`use<`、`r#` 关键字、注释与字面量换空格之后找层与数 `let`）里有没有一种写法真能遮住钉住路径或绕过叫停而判绿？
2. 新的切法（头两行在代码里恰好一处）、`#[cfg]` 挂钉住那一句判红、`Fn() -> bool` 形找层，还有哪几种写法骗得过、哪几种合法产品写法会被判红或 panic？两份分词（卡池与崩溃放量）还有没有认法不一致的地方？

### K5　E171 走的路径

形式化线要在「全部合完」之后拍树跑 E171（三 oracle 合谋测试）：这两块改动有没有改变挂载路径（`crates/singlefs-core/src/mount.rs` 第 33 行引 `mounted_session::refusal_is_short_of_space`）或发布路径上任何不报宽护的形的行为？有没有一处测试之外的改动不在第二十二块题面里？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 本机内存 60G、几条线同时在跑：每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 12G 的上限，同时起的件数乘上限不超过 12G；编出来的测试二进制也经它跑，不直接执行。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
