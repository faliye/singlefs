# 根环实现第 10z 批第二十四、二十五块：代码轮第一轮判决（2026-10-10）

<!-- doc-lint:not-numbers K1 K2 K3 K4 R C H N1 N8 S2 S4 W1 W3 B2 B3 S14 S14b S14d S14e S19 G14 G14h G16 N01 N02 N03 N15 N16 W8 W9 X6 X7 -->

正文 `research/prompts/_m3-batch10z-xy-code-r1-body.md`。两腿报告：
- 攻方：`research/prompts/m3-batch10z-xy-code-r1-opus-output.md`，sha256 d5426d7b…bc54，329 行；模型 `research/prompts/m3-batch10z-xy-code-r1-opus-model/`（`SHA256SUMS` 221 行）；
- 正推：`research/prompts/m3-batch10z-xy-code-r1-sonnet-output.md`，sha256 d56936a9…e179，447 行；模型 `research/prompts/m3-batch10z-xy-code-r1-sonnet-model/`。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 crates | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-xy-code-r1/tree/` 对 `research/prompts/m3-batch10z-xy-code-r1-snapshot/crates-sha256.txt` `sha256sum -c --quiet` 全过。主工作区 69 份对不上，全是腿跑着时合入的删除线第 50 批与形式化线第 46 批的文件；被判的四份（`mounted_session.rs`、那份 harness 测试、`gpu_unit_checks.rs`、`crash_amplification.rs`）主工作区与快照逐个 OK，腿引的行没被碰 |
| 开工快照 kb | 四份（D16、D23、E171 页、checks-owed）全 OK |
| 引文 | `cite-check.py` 两份报告一起、`--root` 冻结副本加仓根、`--background` 背景材料：「✓ 核了 28 处引文，对上 28 处，没判 0 处」（`~/.local/share/singlefs/work/m3-batch10z-xy-code-r1-main-rerun/cite-check.log`） |
| 复跑攻方模型 | 模型目录 `SHA256SUMS` 全过；拷进 `~/.local/share/singlefs/work/m3-batch10z-xy-code-r1-main-rerun/` 整份跑 `rerun.sh`，退 0、289 份输出。比对腿的 `runs/`：上一轮那几族形 `agree=yes` 15912、`agree=NO` 0；`k1h.log`（H 形）、`k2order-baseline.log`、`k2order-N1.log` 判定行逐行相同；`k3-crash.log`、`k3-gpu.log` 逐字相同；N1 下那份 harness 测试文件「ok. 66 passed」、core 库单测「ok. 226 passed」；B2、B3 下 `c09`、`g09` 切口用例 GREEN；第 3624–3640 行 17 行 RED |
| 复跑正推的 S4 | 正推没有整份复跑脚本；判决只单引它自己量出的 S4，主 agent 在冻结副本的新拷贝上用它的 `scripts/mutworker.py` 施加 `rows-*.tsv` 里 S4 两行：「S4-…@core 绿」「S4-…@harness 绿」（`~/.local/share/singlefs/work/m3-batch10z-xy-code-r1-main-rerun-s4/done.txt`），与腿的 `logs/done-rows-E*.txt` 相同。S2 与攻方 N1 是同一个改坏，由攻方复跑坐实 |
| 腿的数 | 都是腿的模型或副本上的数，判决只据它们定下一批与弹窗，不引进 kb |

被判的源文件：`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| 与用户定案、上一份判决第三节逐条 | 正推逐条同义；被「撤掉身份，文档照实写」取代的会话身份与「别的池同一对数」用例不算差；故意拒清单缺两处（并进 K3） | 同义 |
| K1-1 次序核在上一轮那几族形上 | 攻方 1336 个卡住的格 × 21 / 11 种动作 17526 格，比得了的 15912 格 0 处对不上；`table-reversed`、`table-swapped`、`table-one` 各 1336 格拒成新成员、`ask_io=(0,0,0)`；`table-stale` 照常预演、推环那一套核拒、与真走同结局。正推盘表动作 8 种共 8016 格被拒成新成员、真走无一格放得下，探针 1055 条报 × 4 种坏盘表全拒、没碰盘 | 没打中（两腿各一次） |
| K1-1 H 形推完放得下 | 攻方 `atk_xy_c1.rs`：注入级（C 那一圈换成现行那一版之后第 k 次发布那一槽），k=4、iso ≥ 16、left=30 的 3 格：报里只剩走空，要出路预演 `adm`、出路二末句写「预演过：推完之后这次改动放得下」，照出路二推完再发同一个 unlink 拒成 `NoFileVersionToChange`（`runs/k1h.log`「!ADM_BUT_RESEND_REFUSED」）。不注入时（k=0，left 0–80）没造出 | **打中（窄，注入级）**：预演本身对，错的是出路二末句与第 1254、1259 行文档「再发同一个改动」在报里只剩走空时不成立。改法并进第二十六块 |
| K1-2 推环入口收次序对调的盘表 | 攻方 7 个 K1 形 177 格、正推 4 格：今天推环结局、按段写集合、推完盘面、会话、再发、再挂择根都与照挂载时盘表推的相同，只有写的先后不同；「推环入口也先比次序」那一臂 177 格在任何写之前拒、0 条写。条款没写（正推：D16 只第 37、90 行提推环，都没说比不比次序） | 不是打中；三个入口收不收同一份盘表口径不一，交用户定（第三节） |
| K1-3 `SessionMoved…` 照实写的那一段 | 两腿：逐句对得上（攻方 7 格、正推 756 对相同的 (实例代号, txg) 全不拦，预演与真走 756 / 756；出路二的槽与次数是报里的 266 / 266）。两处措辞两腿一致：例子「同一个池卸载或崩了之后再挂」撞不上（再挂取新实例，攻方 283 格 `moved[2.t->3.t]`、正推崩了再挂 32 / 32 格 `3.19` 对 `2.16`）；文档没写预演发的是报里拷下的那一份改动（攻方 7 格里 6 格别的会话自己发它直接放得下） | 没打中；两处措辞并进第二十六块 |
| K2 第二十四块 21 行与上一轮活下来的变异 | 两腿：21 行全红；`walkSkipped`、记进报里的字段 8 种、`checkRequestBlock512` 红。正推上一轮活的 17 个里 7 个仍活：6 个上一轮已分等价或今天走不到、1 个 C762 | 没打中 |
| K2 第二十四块照绿的改坏 | 攻方 N1 / 正推 S2（盘表那一道挪到版本那一道之前）：66 + 226 条全绿，而会话已动又交次序对调的盘表时交回从 `moved` 变成盘表那一种（攻方 `runs/k2order-*.log` 64 格）；正推 S4（`SessionMoved…` 里现行那一版的实例代号取成报里的）照绿，只有卸载再挂那一格分得开；攻方 N8、正推 W1、W3 推测等价 | **判别力打中（窄）**：N1 / S2、S4 补用例，并进第二十六块；N8、W1、W3 记等价（推的），不补 |
| K2 第二十五块 17 行 | 两腿：抄出来的扫源码小程序上 17 行全红，含 W8、W9、X6、X7 | 没打中；checker 档真用例上的证红归提交时整表复跑 |
| K2 第二十五块照绿的改坏 | 攻方 B2、B3（两份切口只数圆括号）照绿，此时方括号宏调用里、前面先放以分号收尾的条目的真测试模块切口照认（`c09`、`g09`，rustc 编得过，非测试构建里可调用）；原样下两份都红（正推 S14d、S14e 同样原样红） | **判别力打中（窄）**：两份切口新用例各加方括号那一形，并进第二十六块 |
| K3 点名写法与名单 | 两腿：S14、S14b、S19、G14、G16、N01、N02 全红，G15、G17 对照照旧，N03、N15、三处名单都在；括号那一道 0 误判 0 panic（正推 18547 个真文件层数 0 差、阴性对照会红；22 条合法位置放行、16 条非法全红） | 没打中 |
| K3 合法写法判红没写故意拒 | 攻方：卡池模块挂 `#[cfg(feature = …)]`（`g16`）、别处另有 `card_pool` 模块（`g17`）、卡池外另有 `fn lease_card`（`g19`）、借据丢掉时那一步挂 `#[cfg_attr]`（`g20`）。正推：`Fn() -> (bool, u8)`（`N16`）、唯一一份借据丢掉时那一步写成 `impl Drop for self::GpuCardLease`（`G14h`）。方向都是不过就红 | **打中（窄，文档）**：六种写进故意拒的写法、各写为什么，并进第二十六块 |
| K4 E171 走的路径 | 两腿：`mount.rs` 逐字节相同、`refusal_is_short_of_space` 函数体相同；测试之外的改动只有新成员与要出路那一道核；两份 checker 档文件测试模块之外逐字相同 | 没打中 |

## 三、用户定案与下一批（第 10z 批第二十六块）

用户 2026-10-10 弹窗原话「也先比次序 (Recommended)」（问推环入口要不要也先比盘表次序；另一项是维持今天不比、文档照实写、口径不一记欠账）。弹窗里交代了：今天的做法量过无害（177 格、4 格），另一臂 177 格在任何写之前拒，D16 没写，改法被攻过零轮。

第二十六块按文件分两件、同时派：

- 甲（core 与那份 harness 测试）：
  - 推环入口先照 `publish_user_change` 那样调 `is_the_device_table_of_the_mount`，不符在任何读写之前拒，交回与要出路同一种盘表成员（带两份盘表）；补用例（次序对调、多一块、少一块各一条：拒、读写探三计数 0、盘不动、换回挂载时盘表照常推）与变异；
  - H 形：报里只剩走空时出路二末句改成「意图已发成、推完之后剩下的走空在下一次改动开头走、放得下，不再发这次改动」，第 1254、1259 行「再发这次改动」限定为报里还剩这次改动时；补注入级用例（攻方 k=4、iso=16、left=30 那一形：要出路列推环，推完再发同一个 unlink 被拒，推完之后发别的改动时开头的走空做成）；
  - 补用例：会话已动又交次序对调的盘表，要出路拒成 `SessionMoved…`（N1 / S2 要红）；卸载再挂之后拿旧报要出路，`SessionMoved…` 里现行那一版的实例代号是新实例的（S4 要红）；
  - 第 273、274 行两处措辞：删去「同一个池卸载或崩了之后再挂」那个例子；补「预演发的是报里拷下的那一份改动」。
- 乙（checker 档）：
  - 两份切口新用例各加一形方括号宏调用（B2、B3 要红）；
  - 六种合法写法（`g16`、`g17`、`g19`、`g20`、`N16`、`G14h`）写进故意拒的写法、各写为什么；
  - `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs` 内联测试 `publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros` 的钉值补上 `retirement_tree_node_write_calls=0 retirement_tree_node_written_bytes=0`（删除线合后快档那一条红）。
- 两件合入后与形式化线第 46 批（E165 装置内联测试与变异行第 3639–3640 行，已在主工作区）合成一批走一轮代码轮。E171 乙类常量换新值照 E171 第 8 次修订 8.2 等全部各线合完再做，不进第二十六块。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-10 不受影响：第 37 行写回等第二十六块那一轮判完，带上要出路与推环入口先核盘表次序、报不带会话身份 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-10 不受影响：推环入口先判冻结，两腿没打中 |
