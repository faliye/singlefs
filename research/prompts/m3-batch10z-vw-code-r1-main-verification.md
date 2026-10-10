# 根环实现第 10z 批第二十二、二十三块：代码轮第一轮判决（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 R C F H M1 M2 M3 M5 M6 M7 M8 M9 M10 S14 S19 G14 G16 N01 N02 N03 N15 W8 W9 X6 X7 -->

正文 `research/prompts/_m3-batch10z-vw-code-r1-body.md`。两腿报告：
- 攻方：`research/prompts/m3-batch10z-vw-code-r1-opus-output.md`，sha256 a1c7152e…6b8，288 行；模型 `research/prompts/m3-batch10z-vw-code-r1-opus-model/`；
- 正推：`research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md`，sha256 73ba11fd…f134，535 行；模型 `research/prompts/m3-batch10z-vw-code-r1-sonnet-model/`。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 crates | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree/` 与主工作区对 `research/prompts/m3-batch10z-vw-code-r1-snapshot/crates-sha256.txt` `sha256sum -c --quiet` 都全过 |
| 开工快照 kb | 六份（D16、D28、D23、invariants、checks-owed、E171 页）全 OK，腿跑着时没人改 |
| 引文 | `cite-check.py` `--root` 冻结副本加仓根：攻方 6 / 6、正推 7 / 7 |
| 复跑攻方模型 | 主 agent 在新副本上整份跑 `rerun.sh`（`~/.local/share/singlefs/work/m3-batch10z-vw-code-r1-main-rerun/runs/`）：`rerun.sh` 退 1、停在变异表那一段（`read -r args test < <(… | tr '\n' ' ')` 没有换行收尾，read 退 1，`set -e` 下整份停），那一段与 K4 两段主 agent 用 `rest.sh`（同目录，照原样跑、只把取参数改成 `mapfile`）补跑。两边都有的 112 份日志，去掉计时与「filtered out」计数后 111 份逐字相同，1 份（`mut-M4-unlink-kept-on-the-next-inode-1-…`）腿那一份是换了参数手跑的（`ow=13`，`rerun.sh` 给的是 `ow=10,11`），复跑这一份里原样那一行与报告第六节引的 `ask=still … resend_unlink=pushed(1)->notwhat(ceiling)` 逐字相同；K1-3 两处打中逐字复现（`other.log`「same_pair=true pair=2.16 … same_text_as_own=false」；k1 主形 12 份日志里 MISMATCH 全在 `act=table-reversed`，别的动作 0 条）；变异表第 3575、3577、3583、3584 行四份 `test result` 行与腿逐字相同（5 / 5 / 3 / 3 条红）；K4 两段复跑的 13 + 44 行都在腿那两份里逐字找得到，腿那两份另多 7 + 12 行，是 `gen-scan-inputs.py` 不生成、腿手加的输入，没复跑。复跑副本与编译目录已删 |
| 主 agent 现读 | 冻结副本 `crates/singlefs-core/src/mounted_session.rs` 第 1254–1257 行要出路第一道只比 `root.instance` 与 `root.checkpoint_txg`，第 267 行文档括注写「或这一份报不是这个会话的」也归这一种拒；`ways_out_of_the_wide_protection`（第 1249 行起）不调 `is_the_device_table_of_the_mount`，预演里推环那一串核盘表用 `parameters_of_the_pool_agreeing_with_the_caller_inputs`（第 1325 行）；`publish_user_change` 第 1014 行先调 `is_the_device_table_of_the_mount`（`crates/singlefs-core/src/mount.rs` 第 950–958 行，`.eq` 逐项比，次序敏感）。坐实攻方 K1-3 两处的读码一半 |
| 腿的数 | 都是腿的模型或副本上的数，判决只据它们定下一批与弹窗，不引进 kb |

被判的源文件：`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/block_device.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| 与用户定案、上一份判决第三节逐条 | 正推 11 条同义，三处补充：「头两行恰好一处」的上界与「第 0 层」两道条件没有用例钉；`#[cfg]` 规则不管卡池的 `impl Drop` 与借卡入口；返回类型换写法的叫停类型不在名单里 | 同义；三处补充并进 K3、K4 那两行 |
| K1-1 延后之后的预演结局 | 攻方：九形 283、实现员三形 33、两槽 C 245、被抛弃根 A 与 C 209、窄四档 566，同会话同盘表的动作 1336 格 0 处对不上。正推：攻方 pqrs 203 格 20 / 182 / 1 与上一轮相同，tu 各族 3676 格 0 `agree=NO`；自己的探针列推环的 597 条全放得下、不列的 326 条全被拒 | 没打中（两腿各一次，攻方带不推环的对照） |
| K1-2 报里记的还剩什么 | 攻方：四种改动逐字是被拒那一刻剩下的那一份，M3、M5、M6、M7 被它自己的 kinds 用例抓到。正推：239 份报逐项记对；但入库用例只钉种类与顺序写的内容长度，记错字段的 8 种改坏（`newInodesCountPlus`、`unlinkInodeZero`、`overwriteContentHalf`、四种改动写入时间记 0、`asUserChangeSwapOverwrite`）照绿，前两种改变推完之后预演要发的改动 | 实现没打中；**判别力打中（窄）**：补用例，并进第二十四块 |
| K1-3 会话还在不在被拒那一版上 | 攻方：别的池同一个实例、推到同一个 txg（2,16），拿这个池的报去要出路，不拒、照那个池的状态预演，出路一文字用外来报的根环槽 2.3（`runs/other.log`「same_pair=true … same_text_as_own=false」）；交两块盘次序对调的盘表，预演 `adm`、列推环，真再发拒成 `DeviceTableOtherThanTheOneOfTheMount`（主形各格 `act=table-reversed` 全 MISMATCH；`table-swapped`、`table-one`、`table-stale` 预演里推环就拒、与真推同结局）。正推：同一会话里不动这两个数而动分配器的只有冻结，冻结着时要出路交 `PushRefused(Frozen)`、与真推同拒；没找到状态没变却被拒成 `SessionMoved` 的形 | **打中（窄，两处）**：判报身份只比两个数、文档说的「不是这个会话的报」没拦；要出路不核盘表次序。两处都要调用方交一份与被拒那一刻不一致的东西。交用户弹窗，**用户 2026-10-09 定「现在补第二十四块再攻一轮」** |
| K1-4 报那一步不碰盘、要出路只读不写 | 攻方：报 (探,读,改盘)=(0,0,0)，要出路盘不动，读错注入 4034 种结局不变、列不列推环 0 处分叉。正推：报那一步六项盘调用与分配器克隆数逐项 0；要出路读 666 到 868 次、写 0 次，会话、分配器、盘一字不变；报那一步多一笔 `content.to_vec()`，随改动内容线性涨（1 GiB 约 179 毫秒） | 没打中；拷内容那一笔记下，是「错误不借调用方的内容」的代价 |
| K2 对齐判共用 | 攻方 M10（设备大小改 `u64::MAX`）红在越界与差分。正推：`block_device.rs` 对 HEAD 只改可见性与文档 3 行，文件后端对拍 10 万请求 0 不同；入库用例只在物理块 512 的盘上，`checkRequestBlock512` 照绿 | 没打中；物理块 4096 没有入库用例钉，补用例并进第二十四块 |
| K3 判别力 | 攻方：15 个变异里 13 个被那份测试文件拦住，`walkSkipped` 照绿，造出了分开它的形（unlink 卡在走空、推环做得成、推完走空仍被拒：原样 `still`、`walkSkipped` 下 `adm`，`runs/k3walk-H-baseline-keep4.log` 对 `runs/mut-walkSkipped-1-…`）。正推：第二十二块 28 行全红（含第 12、14、20、21 行）；上一轮活的 7 个 6 个红，`walkSkipped` 与 C762 仍活；72 个变异 51 红、17 活、4 旧串对不上，17 个里等价或今天走不到的 6 个、C762 1 个、记进报里的字段没钉 8 个、物理块写死 1 个、`walkSkipped` 1 个；同样造出分开 `walkSkipped` 的形（k=4、n=1、没挡的槽留 29 到 31）；F、H 推完之后那一段在扫过的范围里分不开；第二十三块去掉「恰好一处」「第 0 层」两道条件（`W8`、`W9`、`X6`、`X7`）用例照绿 | **打中（窄）**：harness 那份测试文件补 `walkSkipped`、记进报里的字段、物理块 4096 三样，并进第二十四块；第二十三块那两道条件的用例并进第二十五块 |
| K4 扫源码 | 攻方：上一轮块注释藏头两行的口子已堵，新放行的写法都放对，上一轮两个误判改对，没找到新的遮盖写法。正推：两份分词在 18547 个真 Rust 文件上与 `proc_macro2` 层数 0 差、0 panic（阴性对照 32 个差、58 个 panic）；三种绕得过：宏吃掉 `#[cfg(test)]` 把测试模块包进括号（`S14`，rustc 小程序证实非测试构建里可调用）、卡池 `impl Drop` 与四个借卡入口前先放一份 `#[cfg(any())]` 干净副本（`G14`、`G16`，14 条用例全绿）、叫停类型的返回类型写成 `::core::primitive::bool`、`(bool)`、别名、类型宏（`N01`–`N03`、`N15`）；合法写法判红的：`#[cfg(any())] extern crate`、函数体里的 `mod`、通配、头两行换写法、CRLF；两份切口认头两行的口径不一致（卡池按词、崩溃放量按行） | **打中（窄，只在测试模块里）**：K4 这是第三次代码轮（第十八、二十一、二十三块各攻一次），照 `.claude/rules/three-way-inference.md`「第三轮之后停」不再为它开轮；三种绕法能拦的补到会拦（头两行前一个词限 `;`、`}`、`]` 或文件开头；`impl Drop` 与借卡入口要求恰好一处），拦不了的写进用例旁的名单，合法写法判红的写明故意拒；并进第二十五块，与第二十四块同时派、合成一批走下一轮代码轮，那一轮 K4 只核这几处补法拦没拦住点名的写法，不再开新的攻击面 |
| K5 E171 走的路径 | 攻方：`mount.rs` 与 `refusal_is_short_of_space` 逐字未改。正推：同上；不报宽护的形走同一条早退；两份检查器文件的改动全在 `mod tests` 里，E171 二进制不引这两块；发布路径上 4 个别的测试二进制绿 | 没打中 |

## 三、用户定案与下一批（第 10z 批第二十四、二十五块）

用户 2026-10-09 弹窗原话「现在补第二十四块再攻一轮」（弹窗里另两条：记欠账现在拍树；只补盘表次序那一处再攻一轮）。弹窗里交代了：两处都要调用方交一份与被拒那一刻不一致的东西，同会话同盘表时攻方 1336 格一格不出；改 `mounted_session.rs` 会让 E171 装置的 sha 变（推的，没量过）；改法被攻过零轮。

- 第二十四块（core 与那份 harness 测试）：
  - 要出路先核盘表：`ways_out_of_the_wide_protection` 预演之前照 `publish_user_change` 那样调 `is_the_device_table_of_the_mount`，不符就拒、不预演，一个盘都不读；
  - 报带会话身份：`StuckBehindTheWideProtection` 与会话各记一个这次挂载的身份，要出路第一道连它一起比，不是这个会话的报拒、不预演；第 267 行文档照实写；
  - 补用例：别的池同一对数的报、次序对调的盘表各一条；`walkSkipped`（注入级：unlink 卡在走空、推环做得成、比预演 `ChangeStillRefusedAfterThePush` 的拒绝与推完真再发同一个 unlink 的拒绝 `Debug` 逐字）；报里记的字段逐项钉（正推那 8 种改坏要红）；只进内存的盘在物理块 4096 的盘上走一遍。
- 第二十五块（checker 档测试，与第二十四块同时派、文件不相交）：第二十三块「恰好一处」「第 0 层」两道条件补用例钉住（`W8`、`W9`、`X6`、`X7` 要红）；三种绕法能拦的补到会拦、拦不了的写进名单；合法写法判红的写明故意拒；两份切口认头两行的口径对齐。只改两份文件的 `mod tests`。
- 两块合入后合成一批走一轮代码轮；那一轮判完没打中再回形式化线「全部合完」。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-09 不受影响：第 37 行写回等第二十四块那一轮判完，带上要出路先核盘表与会话身份 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-09 不受影响：第九项与这两块无关 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-09 不受影响：推环入口先判冻结两腿没打中 |
