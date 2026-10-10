# 根环实现第 10z 批第二十、二十一块：代码轮第一轮判决（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 R C F A B D E G H I K N -->

正文 `research/prompts/_m3-batch10z-tu-code-r1-body.md`。两腿报告：
- 攻方：`research/prompts/m3-batch10z-tu-code-r1-opus-output.md`，sha256 d8a2c3d9…6a6f，415 行；模型 `research/prompts/m3-batch10z-tu-code-r1-opus-model/`；
- 正推：`research/prompts/m3-batch10z-tu-code-r1-sonnet-output.md`，sha256 e0f1ad73…dee6，573 行；模型 `research/prompts/m3-batch10z-tu-code-r1-sonnet-model/`。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 crates | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree/` 对 `research/prompts/m3-batch10z-tu-code-r1-snapshot/crates-sha256.txt` `sha256sum -c --quiet` 全过 |
| 开工快照 kb | 六份里 D16、D28、checks-owed、E171 页 OK；D23、invariants 对不上，是删除线第 50 批在腿跑着时写的（原串新串在 `~/.local/share/singlefs/work/kb-writeback-batch50/report.md` 第三节，sha256 前 12 位 b54a8578a569，另两处 replace-once 记在调度记录）；两腿都没按行号引这两份（攻方 0 处 kb 行号引文，正推只引 `.claude/kb/decisions/28-挂载期承诺量.md:27`，D28 OK） |
| 引文 | `cite-check.py` `--root` 冻结副本加仓根：攻方 14 / 14、正推 6 / 6（正推另报 33 处按行号的文件:行它自己逐处核过） |
| 复跑攻方模型 | 主 agent 在新副本上整份跑 `rerun.sh`（`~/.local/share/singlefs/work/m3-batch10z-tu-code-r1-main-rerun/runs/`，rc=0）：去掉计时与峰值列后 81 份里 71 份 `^ATK` 行与腿交回的逐字相同，10 份只差路径前缀或汇总行多一个 `area=`、`rehearsal_kinds=` 标签，数逐个相同；`cost-4096.log`、`k4-v2-check.log` 两份腿是手跑的、不在 `rerun.sh` 里，没复跑。大盘代价（计时随机器负载浮动）：两块 2 TiB 盘 `rehearsal=adm 22.658s peak+2462.9MiB`、`push=Ok(22) 22.391s`、`overwrite=admitted 0.627s peak+1790.7MiB`（腿报 24.965 秒、2461.1 MiB、26.750 秒、0.687 秒）；16 GiB 0.060 秒、128 GiB 1.184 秒、512 GiB 6.243 秒、1 TiB 12.054 秒。复跑副本与编译目录已删 |
| 主 agent 现读 | 冻结副本 `crates/singlefs-checker-tier/src/crash_amplification.rs` 第 3581 行常量 `TESTS_MODULE_OPENING`、第 4666–4672 行 `code_lines_outside_the_tests_of` 用 `source.find(TESTS_MODULE_OPENING)` 取第一处，坐实攻方 K4-2 的切法 |
| 腿的数 | 都是腿的模型或副本上的数，判决只据它们定下一批与弹窗，不引进 kb |

被判的源文件：`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| 与用户定案甲、上一份判决第三节逐条 | 正推 14 条同义，两处补充：出路一「永久读不出的后果」只写在 R 有根一支（没根一支实测没有这个后果）；第二十一块「条目层调宏」把属性与常量初始值里的表达式宏也拒了 | 同义；第二处补充并进 K4 那一行 |
| K1-1 预演与真结局 | 攻方 3393 份报（上一轮 9 形 283 格、放开小动作 2057、实现员三形 33、两槽 C 245、被抛弃根 A 与 C 同在 209、更窄四档单元区 566）每份照出路二真走一遍：列不列推环 0 处对不上、N 0 处对不上、推完仍拒的预演拒绝与真再发逐字相同。正推：攻方 203 格列推环 20/20 推完放得下、不列的 183 格 182 格仍拒；随机 902 条 CMP 非空 0；它上一轮的推翻条件 0 例；1 格偏严（停因是一次准入发布数做满，推完第二次再试才放得下） | 没打中（两腿各一次，攻方带不推环的对照）；偏严那 1 格方向是少列出路，不把走不通的出路报给用户，记下 |
| K1-2 预演不落盘、读盘出错 | 攻方 360000 次对拍 0 差、底下的盘一字不变，一次性读错 4034 种与持续读不出 4034 种列不列推环 0 处分叉；正推：只持 `&dyn BlockDevice`、直接调预演 472 次会话与盘都没动、差分对拍 10 万请求 0 不同 | 没打中 |
| K1-3 预演代价 | 攻方：随每块盘槽数线性涨，乘推环次数；主 agent 复跑坐实（见核查）。正推：384 槽盘 20–66 ms，内存约 4.6 MB 加每单元约 74 KB | **打中（窄，代价）**。交用户弹窗，**用户 2026-10-09 定「改：预演延到要出路时才做」** |
| K2 措辞、冻结、无隔离 | 两腿：N 与真推之差逐盘相同，两种出路一说法与实际同义；冻结之外没找到预演过、真发第一次被拒的状态；C 有落点无隔离自然历史 0 例、两处各一条注入级用例钉住 | 没打中 |
| K3 判别力 | 正推：实现员 25 + 6 行逐行全红（loop-span1 那一行重新红），第二十一块 14 行在扫源码模型上全红；它上一轮 11 个活变异 10 个现在红、C762 那一个仍活（判决已定不补）；新造 19 个活 7 个（只进内存的盘读回拼接 4 个、对齐越界 2 个、`walkSkipped` 1 个）。攻方 11 个变异那份测试文件拦住 C、G、N，照绿 B、D（有行为后果：预演交回的拒绝换成错的原因 263/283 格，列不列推环不变）、F、H、K（没有用例走到），A、E、I 等价或这几形里等价；K5 对齐判改成恒放行那份测试照绿 | **打中（窄）**：补用例 |
| K4 扫源码 | 正推：被钉住的那一句挂 `#[cfg(any())]` GPU 与崩溃放量两条都判绿，rustc 小程序证实真丢叫停、真换掉被调函数，名单没列；崩溃放量按类型找层对 `+ Send`、`+ 'static`、`&mut`、`&'static`、夹注释、`Box<dyn …>` 的新层判绿；合法写法判红或 panic：属性与常量初始值里的表达式宏、`impl … + use<'a>`（panic）、GPU 分词不认 `cr#"…"#`（含 `"` 时 panic，与崩溃放量那份不一致）、`r#mod`、行尾注释或字符串里的 `let stop_request`。攻方：块注释里写测试模块头两行藏住一层（主 agent 现读坐实）、签名行尾注释带 `fn` 时层名取成注释里的字；`use wgpu;` 与函数体里块作用域的 `use core as std;` 判红 | **打中（窄）**：能拦的补到会拦，拦不了的写进名单，误判与 panic 改掉 |
| K5 抄的那一份对齐判 | 两腿：判法逐句相同，对拍 10 万请求 0 差；唯一差别是只进内存的盘 `write_zeroes_at` 先分配整段长度，今天的产品路径不触发 | 没打中；没有会红的用例并进 K3；抄的那一份改成共用（上一份调度记录已定） |

## 三、用户定案与下一批（第 10z 批第二十二、二十三块）

用户 2026-10-09 弹窗原话「改：预演延到要出路时才做」：报 `StuckBehindTheWideProtection` 时不预演；调用方要出路时另调一个方法算（传盘表），只要错误码、不看出路的调用方不付这一笔。弹窗里交代了：不改（记欠账、现在拍树）、记住上一次预演（只省重试）两条的代价；改法被攻过零轮。形式化线回：改 `mounted_session.rs` 要在回「全部合完」之前改完、合完、判完。

- 第二十二块（core）：预演延后；`DeviceKeepingWritesInMemory` 的对齐判改成调 `crates/singlefs-core/src/block_device.rs` 那一份（开成 `pub(crate)`），不留抄本；补用例拦住攻方 B、D（比预演交回的拒绝与推环之后真再发的拒绝）、F、H、K（注入级）、K5 对齐判恒放行与正推活下来的 7 个（只进内存的盘读回拼接、对齐越界、`walkSkipped`）。落在 `mounted_session.rs`、`block_device.rs` 与那份 harness 测试。
- 第二十三块（checker 档测试）：测试模块切法改成要求头两行在源文件里恰好一处（或先认注释与字面量再切）；`#[cfg(...)]` 挂在钉住那一句上判红；按类型找层认 `+ Send`、`+ 'static`、`&mut`、`&'static`、夹注释、`Box<dyn …>` 或写进名单；误判与 panic 逐个改（`use wgpu;`、块作用域 `use core as std;`、属性与常量初始值里的表达式宏、`impl … + use<'a>`、`cr#"…"#`、`r#mod`、行尾注释或字符串里的 `let stop_request`、签名行尾注释带 `fn`）。落在 `gpu_unit_checks.rs`、`crash_amplification.rs` 的测试模块。
- 两块合入后合成一批再走一轮代码轮；那一轮判完没打中再回形式化线「全部合完」。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-09 不受影响：第 37 行写回时带上「出路按预演定、预演在调用方要出路时才做」 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-09 不受影响：第九项与这两块无关 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-09 不受影响：推环入口先判冻结两腿没打中 |
