# m3-batch25-code-r1 攻方腿报告（Opus，K2：N1、N2，另做 m3-batch24-code-r1 N1 单故障、N3 第二次抽样）（2026-10-06）

立场：攻方，造反例。只跑 harness 档，GPU 不跑（GPU 那一侧读 wgsl、用 CPU 算期望）。所有数都在冻结副本的拷贝上量（副本上的数，不算入库装置上的数）。
冻结副本 `/tmp/claude-1000/m3-batch25-code-r1/tree/` 的 7 份被判文件与 `research/prompts/m3-batch25-code-r1-snapshot/crates-sha256.txt` 逐个 `sha256sum -c` 为 OK（开工时现核）。

## 复跑

模型在 `research/prompts/m3-batch25-code-r1-opus-model/`。仓路径取 `SINGLEFS_REPO`；冻结副本只有 `crates/` 与 `Cargo.*`，包装脚本（`run-with-memory-cap.sh`、`capped.sh`）另从变量 `SINGLEFS_TOOLS=<仓根>` 取（`rerun.sh` 文件头写明）。编译 6G、跑 4G，线程经 `capped.sh`（`OPUS_THREADS`，默认 10；并行的批每件 2），用自己的编译目录（工作目录下 `target/`、`target-fix/`）；每段前台四分钟以内。

```
SINGLEFS_REPO=/tmp/claude-1000/m3-batch25-code-r1/tree SINGLEFS_TOOLS=<仓根> bash research/prompts/m3-batch25-code-r1-opus-model/rerun.sh <空工作目录> <段|all>
bash research/prompts/m3-batch25-code-r1-opus-model/rerun.sh compare <工作目录>
```

段：`setup build a b1 b2 c fix-setup fix-build fix-a fix-kbi`，之后 `summary`（`compare` 自己会先做 summary，不读 `SINGLEFS_REPO`，比之前两边各自排序）。交回前在空目录 `/tmp/claude-1000/m3-batch25-code-r1-opus/rc2` 里用仓里这份模型逐段从头跑过，`compare` 在 `/tmp` 下不带 `SINGLEFS_REPO` 跑，原样输出：`SUMMARY_MATCHES 717 行`。`fix-kbi` 那一段里测试二进制退 101 是预期的（F-N 改了 N5-b 那条用例钉的判定），结论看 summary 里 `[fix-kbi]` 两行。

| 文件 | sha256 |
|---|---|
| `expected-summary.txt` | `13d8b1ee2c2fb2a4c5eda69b0d7e97fa767e7341e0a22420b379f4f65dd52b35` |
| `fix-FN.diff` | `32f5feaed5479639be70763b83cf614ffbec3a7ab153cb21d75424b33b23c5bb` |
| `opus_b25_attack.rs` | `8ec29249cf46f2fb8fa6d2350fd8233a948792b1da4a9ec295de93892143f9ba` |
| `rerun.sh` | `0a812ca9f35ae39418ccda4d5faa952a3edcbca026c580ca1f68d3b29b2b92e7` |

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| K2 N1：走到末片而候选集没滤全（I-9.29 误红） | 没打中（一次抽样） | 41 形里门开的 26 形，候选集都按整张表滤；红 I-9.29 的两形是镜像上的实例表本身让被抛弃的根有效 | 一 |
| K2 N1：没走到末片而滤全了（无谓判不了） | **打中 H-a**（坏镜像） | 第 0 片已读到环里每个实例的行、只有第 1 片读不出，或单片表只有链指针记录不合法（N5-b 一族）：被抛弃的根已滤掉，门照样关；镜像上判得出，F-N 量过能分开 | 1.2 |
| K2 N1：同上，走读认「走过」的键 | **打中 H-b**（坏镜像） | 第 1 片指针的盘 0 位置条目指回第 0 片、盘 1 那条完好：core 读得出整张表，主走读读到第 1 片后按盘 0 那条认成走过、丢掉它的行，门关、多红 I-3.9、I-5.1 | 1.3 |
| K2 N2：CPU 与 GPU 判得不一样 | **打中 H-c**（坏镜像；GPU 侧推的） | 实例表链 257 片：CPU 走到末片、比、红 I-9.29；GPU 链循环 256 片到头、交接段第 30 个字写 0、不比、不置位 | 2.1 |
| K2 N2：别的支 | 没打中（读码，一次抽样） | 置位时机、合法性判法、提前出口、清零、交接段读写逐支相同 | 2.2 |
| K2：合法历史上用户动作放开扫 | 没打中（一次抽样） | 140 段、1 724 次判定（被抛弃版同号不同出生代、实例表两片都在内），I-9.29 全部成立，门关 0 次；546 次判定里门滤掉了一版会让 I-9.29 红的被抛弃版 | 三 |
| 第二次抽样：m3-batch24-code-r1 N1 单故障 | 第二次也没打中 → 记「没打中」 | 换一组类型段取值与身份三段、换四张起点，266 形全部只红目标 | 4.1 |
| 第二次抽样：m3-batch24-code-r1 N3 | 第二次也没打中 → 记「没打中」 | 1 724 次判定，缺省理由 0、专门理由说假话 0、九条红 0 | 4.2 |
| 改法 F-N | 只在我的模型上量过、被攻过零轮 | 修 H-a 那 6 形，H-b、H-c 碰不到；N5-b 那条 harness 用例要随之改 | 1.2、五 |

## 一、N1（CPU 门）：最新根实例表的形状 × I-9.29（A 段）

### 1.1 起点与装置

起点 X 是一段合法历史，一个字节都不改（`opus_b25_attack.rs` 的 `pool_x`）：新池新建文件（实例 1）→ 再挂（实例 2）→ 建一个号（这一版记作 C）→ 崩溃恢复落到 C 之前那条根（`abandon_the_newest`，造法照 `common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`，换成内存稀疏盘）→ 实例 3 再建一个号。C 上号 2 的出生代是 6，现行线上号 2 的出生代是 9：同号不同出生代自然出现，不靠改字节。C（实例 2、txg 6）写回之后读得出，最新根（实例 3、txg 9）的实例表行是 (1, 3)、(2, 5)，按它 C 被滤掉。

每一形在 X 的镜像上只改最新根指着的实例表（第 0 片在原槽重封，整单元校验和经 `common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum` 补到根槽与 journal 新根段；第 1 片起写在槽 200 000 起的新槽上，两盘同槽），判一次 `check_pool_image`。I-9.29 一栏：「部分没判[门]」是说明以「最新根的实例表读不出」开头的那一支（门关）；「core读得出 / 读不出」是 core 的 `recovery::instance_table_of_root` 对择到的根读不读得出整张表。

冻结副本上的原样输出（`expected-summary.txt` 里 `[a]` 打头的 43 行，命令数过：`grep -c '^\[a\] RESULT A' expected-summary.txt` 为 43）：

```
[a] RESULT A pool-x abandoned=(i2,t6) records_on_C=[(1, 1, 3), (2, 1, 6)] records_on_current=[(1, 1, 3), (2, 1, 9)]
[a] RESULT A pool-x newest=(i3,t9) rows=(1,3)(2,5) ring=[(0, 0), (1, 1), (1, 2), (1, 3), (2, 4), (2, 5), (2, 6), (3, 7), (3, 8), (3, 9)]
[a] RESULT A A0 干净 I-9.29=成立 core读得出 violated=[]
[a] RESULT A A1 盘0那份坏 I-9.29=成立 core读得出 violated=["I-2.1", "I-4.8", "I-7.4"]
[a] RESULT A A1 盘1那份坏 I-9.29=成立 core读得出 violated=["I-2.1", "I-4.8", "I-7.4"]
[a] RESULT A A2 两份都坏 I-9.29=部分没判[门] core读不出 violated=["I-2.1", "I-3.1", "I-4.8", "I-7.2", "I-7.4"]
[a] RESULT A A3 载荷CRC翻一位只重封头 I-9.29=成立 core读不出 violated=["I-2.3"]
[a] RESULT A A4 链指针记录前插kind2 I-9.29=成立 core读不出 violated=[]
[a] RESULT A A4 链指针记录前插kind3 I-9.29=成立 core读不出 violated=[]
[a] RESULT A A4 链指针记录前插kind128 I-9.29=成立 core读不出 violated=[]
[a] RESULT A A4 链指针记录前插kind255 I-9.29=成立 core读不出 violated=[]
[a] RESULT A A4 第0条前插kind2 I-9.29=成立 core读不出 violated=[]
[a] RESULT A A5 有无下一片写2 I-9.29=部分没判[门] core读不出 violated=["I-3.8"]
[a] RESULT A A5 有无下一片写3 I-9.29=部分没判[门] core读不出 violated=["I-3.8"]
[a] RESULT A A5 有无下一片写255 I-9.29=部分没判[门] core读不出 violated=["I-3.8"]
[a] RESULT A A6 无下一片而指针不全零 I-9.29=部分没判[门] core读不出 violated=["I-3.8"]
[a] RESULT A A7 去掉链指针记录 I-9.29=部分没判[门] core读不出 violated=["I-3.8"]
[a] RESULT A A7 链指针记录之后多一行 I-9.29=部分没判[门] core读不出 violated=["I-3.8"]
[a] RESULT A A8 去掉实例2那一行 I-9.29=不成立 core读得出 violated=["I-3.1", "I-3.9", "I-9.29"]
[a] RESULT A A8 实例2那一行的txg改成被抛弃那一版的txg I-9.29=不成立 core读得出 violated=["I-3.1", "I-3.9", "I-9.29"]
[a] RESULT A A9 每一行重复一次 I-9.29=成立 core读得出 violated=["I-3.8"]
[a] RESULT A A10 行倒序 I-9.29=成立 core读得出 violated=[]
[a] RESULT A A11 每一行flags字节写1 I-9.29=成立 core读不出 violated=[]
[a] RESULT A A12 头里格式版本改成2 I-9.29=部分没判[门] core读不出 violated=["I-2.4", "I-3.1", "I-4.8", "I-7.2", "I-7.4"]
[a] RESULT A A13 容器号写1 I-9.29=部分没判[门] core读不出 violated=["I-1.1", "I-3.1", "I-4.8", "I-7.2", "I-7.4"]
[a] RESULT A A13 出生树写1 I-9.29=部分没判[门] core读不出 violated=["I-1.1", "I-3.1", "I-4.8", "I-7.2", "I-7.4", "I-1.12"]
[a] RESULT A A14 记录宽写89 I-9.29=部分没判[门] core读不出 violated=["I-1.7", "I-3.1"]
[a] RESULT A A14 声明长度多一条记录（尾巴补零） I-9.29=成立 core读不出 violated=[]
[a] RESULT A A14 声明长度多一条记录、尾巴写一行实例2txg0 I-9.29=成立 core读不出 violated=[]
[a] RESULT A P0 行全在第0片、第1片只有链指针记录 第1片完好 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.11"]
[a] RESULT A P0 行全在第0片、第1片只有链指针记录 第1片两份都坏 I-9.29=部分没判[门] core读不出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.2", "I-7.4"]
[a] RESULT A P0 行全在第0片、第1片只有链指针记录 第1片盘0那份坏 I-9.29=成立 core读得出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4"]
[a] RESULT A P2 被抛弃实例那一行在第1片 第1片完好 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.11"]
[a] RESULT A P2 被抛弃实例那一行在第1片 第1片两份都坏 I-9.29=部分没判[门] core读不出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.2", "I-7.4"]
[a] RESULT A P2 被抛弃实例那一行在第1片 第1片盘0那份坏 I-9.29=成立 core读得出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4"]
[a] RESULT A P6 行全在第1片 第1片完好 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.11"]
[a] RESULT A P6 行全在第1片 第1片两份都坏 I-9.29=部分没判[门] core读不出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.2", "I-7.4"]
[a] RESULT A P6 行全在第1片 第1片盘0那份坏 I-9.29=成立 core读得出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4"]
[a] RESULT A P4 第1片指针的盘0位置条目指回第0片（盘1那条对） I-9.29=部分没判[门] core读得出 violated=["I-2.1", "I-3.1", "I-3.9", "I-3.11", "I-4.8", "I-5.1", "I-7.4"]
[a] RESULT A P5 257片、去掉被抛弃实例那一行 I-9.29=不成立 core读得出 violated=["I-3.1", "I-3.9", "I-3.11", "I-9.29"]
[a] RESULT A P5 256片、去掉被抛弃实例那一行 I-9.29=不成立 core读得出 violated=["I-3.1", "I-3.9", "I-3.11", "I-9.29"]
[a] RESULT A P5 257片、行全在第0片 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.11"]
[a] RESULT A P5 1片、去掉被抛弃实例那一行 I-9.29=不成立 core读得出 violated=["I-3.1", "I-3.9", "I-9.29"]
```

读法：A0–A14 是单片表上的形状，P* 是两片及以上。门开（I-9.29 成立 / 不成立）的每一形里主走读都走到了末片；门关的每一形都是主走读在末片之前停下（读不出、头用不了、身份不对、链指针记录不合法）。CPU 的门与「主走读走没走到末片」逐形一致，没有一形是「走到末片而候选集没滤全」：A8 两形（去掉实例 2 那一行、或把它的 txg 改到 6）红 I-9.29，那是镜像上的实例表本身就说 C 有效，checker 照镜像判，不是门的错（`m3-batch22-code-r1` 正文第五节的误红要「从入口走得到的已发布状态」，这两张走不到）。

### 1.2 打中 H-a（无谓判不了，坏镜像上）：走不到末片、而被抛弃的根其实滤全了

**镜像**：P0「行全在第 0 片、第 1 片只有链指针记录」再把第 1 片两份都翻坏。主走读读完第 0 片（行 (1, 3)、(2, 5) 都收下了）、第 1 片读不出就停。根环里的实例只有 0、1、2、3：实例 3 是挂载根那个实例，按 I-3.8 不会有行；实例 0 是 mkfs，mkfs 不写行；实例 1、2 的行都已读到，按 I-3.8「行按实例代号唯一」第 1 片里不会再有它们的行。被抛弃的 C 已按 (2, 5) 滤掉，候选集与 P0 第 1 片完好时逐条相同——可 I-9.29 报「部分没判[门]」，P0 第 1 片完好时报「成立」。A5（有无下一片写 2 / 3 / 255）、A6（无下一片而指针不全零）、A7（去掉链指针记录）是同一族的单片形状：行全读到了、只是这一片的链指针记录自己不合法，同样报部分没判；A5 写 2 那一形就是第 23 批攻方的 N5-b。

**每一步许可它的那一句**：
- 门只看主走读走没走到末片：`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker/src/walk.rs:10579`：「let abandoned_roots_filtering = walk.abandoned_roots_filtering;」；记成滤全只在末片那一支：`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker/src/walk.rs:2929`：「ChainRecordView::LastPage => {」。
- 实例 0 不写行：`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-core/src/mount.rs:3971`：「/// 中间实例 (i, 0, 0)；实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。纯算，取号之前就列得出来。」
- 行唯一、低于挂载根：`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker/src/walk.rs:2954`：「/// I-3.8（实例表行唯一且低于挂载根）：整张表（链上各片的行接起来）里 `kind` 0 行按实例代号唯一、每行的实例代号 < 挂载根（最新根）的实例；」
- 第 25 批自己的 N5-b 用例钉的是部分没判，理由写的是镜像上分不出（`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:4123`，这一行带直角引号，原样放进代码块）：

```
/// 走不到末片：I-9.29 报部分没判，不报成立（这一片其实就是整张表，可镜像上分不出「只有一片」与「后面还有片」）。
```

- 被比的条款，`.claude/kb/invariants.md:316` 整行（I-9.29）：

```
| I-9.29 | 同号只许一个出生代 | 回退候选集里各版合起来（含由记录施加出来的那几版）同一个 inode 号只许对应一个对象出生代：每一版的 inode 记录的出生代与那一版谱系上墓碑的出生代都算；最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）。依据 D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」；判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T10、T15 两格与第四节用户定案（T15 定「写」）。 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
```

**四句**：
1. 分不分辨臂：这一轮没有臂。它分辨的是「门 = 走到末片」与「门 = 滤全了」两种读法：P0 第 1 片坏这一形上前者报部分没判、后者报成立。
2. 被判的系统当时看不看得到判别它的东西：看得到。判别的东西是「根环与 journal 环里出现的每个实例（挂载根那个与 0 除外）都在已读到的行里各有恰好一行」，三样都在 `check_pool_image` 手里（`roots`、`journal_records_by_device`、`instance_table_rows`）。N5-b 用例那句「镜像上分不出」说的是分不出后面还有没有片，对 I-9.29 要的「滤没滤全」不必分出它。下面 F-N 就是只用这三样的门（量过）。要保留的一处保留：没读到的片里若有违反 I-3.8 唯一性的重复行（同一实例第二行、T 更大），由记录施加出来的那几版会多出来——这一种在镜像上确实看不到，F-N 是按 I-3.8 成立来推的。
3. 满足判据字面的哪一句：这一轮正文第三节 K2 第一条「没走到末片而滤全了（I-9.29 无谓地判不了）」。按 `m3-batch22-code-r1` 正文第五节它不是误红、漏判、漏红里的任何一种（镜像不违反 I-9.29，报的是部分没判不是成立）；按 I-9.29 字面「被崩溃恢复抛弃的根滤不掉时记判不了」，这里 C 滤掉了，条件不成立。
4. 跑前条款的改法在这几格上还中不中：第 23 批判决第三节第 1 条的改法（门看主走读走没走到末片）就是今天的实现，这几格照中；那一条自己写明「N5-b、N5-d 仍报部分没判」，第 23 批判决第二节 K1 N5 那一行也写了「偏保守的方向…采」（下面代码块原样）。所以 H-a 是已知、已采的保守方向里的一形，新的是：它在镜像上判得出，有一个只用镜像上东西的门。

`research/prompts/m3-batch23-code-r1-main-verification.md:24`（带直角引号，原样放进代码块）：

```
| K1 N5 读法 | 一致，有读法差：实现只要实例表读不出就报部分没判，即使环里没有被抛弃的根 | — | **采**：偏保守的方向，与条款「读不出、滤不掉时记判不了」相容（读不出时滤没滤掉认不出来） |
```

**改法 F-N（只在我的模型上量过、被攻过零轮）**：门没开时再看一眼「根环每条根、journal 环每条记录的实例，除挂载根那个与 0，都在已读到的行里、且行唯一」，成立就当滤全（diff 在模型目录 `fix-FN.diff`，29 行，只改 `walk.rs`）。F-N 副本上的原样输出（`[fix-a]` 那几行，与 `[a]` 不同的只有这 6 行）：

```
[fix-a] RESULT A A5 有无下一片写2 I-9.29=成立 core读不出 violated=["I-3.8"]
[fix-a] RESULT A A5 有无下一片写3 I-9.29=成立 core读不出 violated=["I-3.8"]
[fix-a] RESULT A A5 有无下一片写255 I-9.29=成立 core读不出 violated=["I-3.8"]
[fix-a] RESULT A A6 无下一片而指针不全零 I-9.29=成立 core读不出 violated=["I-3.8"]
[fix-a] RESULT A A7 去掉链指针记录 I-9.29=成立 core读不出 violated=["I-3.8"]
[fix-a] RESULT A P0 行全在第0片、第1片只有链指针记录 第1片两份都坏 I-9.29=成立 core读不出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.2", "I-7.4"]
```

该留部分没判的照留：P2（实例 2 那一行在第 1 片）第 1 片坏、A2（第 0 片两份都坏）、P4、A7「链指针记录之后多一行」（行重复，唯一性不成立）在 F-N 副本上仍是「部分没判[门]」。F-N 副本上 `checker_known_bad_images` 整个二进制：`test result: FAILED. 123 passed; 1 failed;`，红的那一条是 `a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged`——正是 N5-b 那条，它钉的就是 F-N 要改的判定；采 F-N 要连它的断言与「判别力」那段注释一起改，变异表第 25 批第 3 行（把链指针记录不合法当成走到末片）的判别力也要重看。GPU 那一侧要同改（推的，没写）。

### 1.3 打中 H-b（无谓判不了，单处坏的位置条目上；门本身没错，错在走读认「走过」的键）

**镜像**：P2（实例 2 那一行在第 1 片）之后，只改第 0 片链指针记录里那条指针的盘 0 位置条目，把槽号写成第 0 片自己的槽（校验和仍是第 1 片的）；盘 1 那条位置条目没动、指着完好的第 1 片。core 读得出整张表（「core读得出」），主走读读第 1 片时盘 0 那份对不上（I-2.1 红一次），盘 1 那份对得上、单元已经拿到手——随后按「盘 0 那条位置条目」认这一片走没走过，认成第 0 片、当成走过就停：第 1 片的行没收下、没走到末片，I-9.29 报部分没判，另多红 I-3.9、I-5.1（P2 第 1 片盘 0 那份坏的对照里没有这两条）。同一处单份坏，坏在「内容」上（P2 第 1 片盘 0 那份坏）报成立，坏在「位置条目指错」上报部分没判。

**每一步许可它的那一句**：
- 读的时候两条位置条目都读、交回第一份对得上的：`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker/src/image.rs:1490`：「/// 按位置条目读一个被引用的单元：每条位置条目都读、都判 I-2.1（校验和与内容匹配），返回第一份对得上的。」
- 认走过只看第 0 条位置条目：`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker/src/walk.rs:2883`：「let page_placement = (pointer.locations[0].device, pointer.locations[0].slot);」，`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker/src/walk.rs:2884`：「if !self.visited_units.insert(page_placement) {」，命中就并进前一版的落点表、`break`（`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker/src/walk.rs:2885`：「self.merge_placements_below_a_unit_walked_by_an_earlier_version(page_placement);」）。
- GPU 同一处同法：`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1601`：「if (!w_visit(pointer.device0, pointer.slot0)) {」。

**四句**：
1. 分不分辨臂：没有臂；它分辨「按读到的那一份认走过」与「按第 0 条位置条目认走过」。
2. 看不看得到：看得到，`read_referenced_unit` 已经知道是哪一份对上的。
3. 字面的哪一句：同 H-a，正文第三节 K2 第一条「没走到末片而滤全了」的近亲——这里候选集确实没滤全（第 1 片的行丢了），门如实报部分没判，所以门没错；无谓的是走读丢掉了一片已经读到手的行。core 照常挂得上（读得出整张表）。
4. 跑前改法还中不中：第 23 批第三节第 1 条的改法与 F-N 都碰不到它（F-N 副本上这一形仍是「部分没判[门]」，量过）。碰得到的是改走读：按交回的那一份所在的位置认走过（推的，没写、没量；`visited_units` 别处也用这个键，改它要连 I-3.5 / I-5.1 那几处一起看）。

只在第 1 片以后的指针上造得出，而且要指回本链已走过的片（新根的走读里实例表是第一样走的，走过的只有本链前几片）；指向一个没走过的槽时走读照走、走到末片。比 H-a 更窄，同属坏镜像。

## 二、N2（GPU 门）：读 wgsl 逐支对 CPU，用 CPU 算期望（GPU 不跑）

| 支 | CPU（`walk.rs`） | GPU（`crash_judge_checker.wgsl` / `_scan.wgsl`） | 同不同 |
|---|---|---|---|
| 记「走到末片」的时机 | 末片那一支、只在挂载根那一遍 | 行那一半（`judge_rows`，只有最新根那一遍为真）里 `chain == 0u` 时置位 | 同 |
| 链指针记录合不合法 | `chain_record_view`：kind 1、宽 88、有无下一片 0 / 1、指针 86 字节全零与否配得上 | `w_chain_record_kind` 同四条 | 同 |
| 提前出口 | 读不出、走过、容器不合法、身份不对、链指针记录不合法 | 同五处，次序同 | 同 |
| 清零与交接 | 每次 `check_pool_image` 新建 `Walk` | 择根之后、走最新根之前清掉；收尾写交接段第 30 个字，扫描内核读它；第 30 个字两个内核别处都不写（交接段只在 `walk_and_hand_off` 收尾那一处写） | 同 |
| 链的片数 | 不设上限（走过的单元不再走） | 循环上限 256 片 | **不同** |

### 2.1 打中 H-c（CPU 与 GPU 判得不一样，坏镜像上）：实例表链长过 256 片

**镜像**：X 上把最新根的实例表改造成 257 片，第 0 片放行、第 1–256 片只有链指针记录，去掉实例 2 那一行（被抛弃的 C 于是进候选集，比就红）。CPU 的原样输出（`[a]` 那几行里的 P5）：

```
[a] RESULT A P5 257片、去掉被抛弃实例那一行 I-9.29=不成立 core读得出 violated=["I-3.1", "I-3.9", "I-3.11", "I-9.29"]
[a] RESULT A P5 256片、去掉被抛弃实例那一行 I-9.29=不成立 core读得出 violated=["I-3.1", "I-3.9", "I-3.11", "I-9.29"]
[a] RESULT A P5 257片、行全在第0片 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.11"]
[a] RESULT A P5 1片、去掉被抛弃实例那一行 I-9.29=不成立 core读得出 violated=["I-3.1", "I-3.9", "I-9.29"]
```

CPU 在 256 片与 257 片上都走到末片、都比、都红 I-9.29。GPU 那一侧（推的，GPU 不跑）：走读内核的链循环
`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1591`：「for (var page = 0u; page < 256u; page = page + 1u) {」
走完第 255 片时那一片的链指针记录说「有下一片」（`chain == 1u`），循环条件到头退出，置位那一句
`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1667`：「w_newest_instance_table_chain_reached_its_last_page = true;」
一次都没走到，交接段第 30 个字写 0，扫描内核
`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3098`：「d_object_births_are_compared = sw(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE) == 1u;」
得假、整张表不比：GPU 不置 I-9.29 那一位，CPU 红。256 片那一形 GPU 第 255 片就是末片、置位、比，与 CPU 同红。这张镜像上 GPU 也不会转成判不了：行只有 1 条，碰不到
`/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:364`：「const C_MAX_INSTANCE_ROWS: u32 = 1024u;」
走过的单元 257 片加别的，碰不到 `/tmp/claude-1000/m3-batch25-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:312`：「const C_MAX_VISITED: u32 = 2048u;」。第 256 片之后 GPU 也不登记那一片的引用，I-3.1 一族在这张镜像上 CPU 与 GPU 也可能不一样（推的）。

**四句**：
1. 分不分辨臂：没有臂；分辨 CPU 与 GPU。
2. 看不看得到：看得到，第 256 片的链指针记录就在 GPU 手里，只是循环不再读。
3. 字面的哪一句：这一轮正文第五节「GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中」；也是正文第三节 K2 第二条「CPU 与 GPU 判得不一样的一形」。
4. 改法：第 25 批 N2 的改动让 GPU 门「与候选集的滤法用同一遍走读」，这一形上两者确实同一遍，只是那一遍在 256 片截断——改法碰不到它。碰得到的（推的，没写）：循环退出时 `chain == 1u` 记判不了（`undecidable`，交 CPU 判），或把上限抬到与 CPU 同一个界（CPU 的界是盘上装得下的单元数）。

可达性：合法写者每片装满 369 行（`crates/singlefs-core/src/instance_table_sweep.rs` 的 `PLACEHOLDER_UNTIL_C690` 注释），257 片要 9 万多行，早过 GPU 的 1 024 行上限（GPU 转判不了、交 CPU），所以合法流的崩溃状态上碰不到；只在对拍自己造的坏流上显出来。

### 2.2 别的支：没打中（读码，一次抽样）

GPU 门的置位、清零、交接逐支与 CPU 相同（上表）。上一行之外再看了三处：交接段第 30 个字之前的区域 `C_GROUPS` 到 `C_SCRATCH_END_SCAN` 恰好止于交接段起点、写有上限，不会盖到它；交接段写完之后 I-4.1 那一遍（`WALK_PHASE_REPLAYED_VERSION`）走读用上下文 1、`walk_is_newest` 为假，不碰这个变量；扫描内核只在交接段有效位为 1 时才读它。都没有一支与 CPU 不同。

## 三、合法历史上用户动作放开扫（B 段）：没打中（一次抽样）

**造法**（`opus_b25_attack.rs` 的 `b_one_birth_gate_on_legal_histories_with_the_user_steps_open`）：四个起点，每一步从九种用户动作里随机抽：建 1 / 2 个号、对一个活号发意图、删一批、走空、回退到前 1–12 新的一版（被抛弃的那几版也在可抽的版里，该被拒）、再挂、崩溃恢复落到前一条根、先建一个号再崩溃恢复抛弃它（重发同号）、连着再挂 5 次。被拒的换回动作之前那一份；做成的判一次，看 I-9.29 与门、删除线九条的理由。固定的只有起点；动作、回退目标、崩溃恢复落在哪一步都放开。起点：S0 新池新建文件、建 2、3 号；SX 即 A 段的 X（被抛弃的 C 与现行线同号不同出生代）；SK 是 S0 装删除开关（墓碑 / 谱系一片 1 条、节点 2 条，判定走旋钮入口）；SR 是 S0 之后实例表清扫交给后台（`HandedToTheBackgroundSweepWhileMounted`，没有调度器，行只增不删）、连着再挂 370 次，最新根的实例表长到两片（第 0 片 369 行、第 1 片在后）。

旗：I-9.29 不成立或部分没判；九条落缺省理由、九条红、I-9.24 报不适用；九条「不适用」那句话按现行那一版的真值核（核法照抄 m3-batch24-code-r1 攻方原型）。另数「暴露」：判的那一刻根环里还有一版被崩溃恢复抛弃的、它上面有号与现行那一版同号不同出生代——门要是放过它、不滤，I-9.29 就红。

| 批 | 起点 | 种子 | 段 × 步 | 判定次数 | 暴露 | 实例表两片 | 旗 |
|---|---|---|---|---|---|---|---|
| b1 | S0 | 101–130 | 30 × 12 | 286 | 43 | 0 | 无 |
| b1 | SX | 201–230 | 30 × 12 | 274 | 194 | 0 | 无 |
| b1 | SK | 301–330 | 30 × 12 | 297 | 37 | 0 | 无 |
| b1 | SR | 401–404 | 4 × 8 | 24 | 3 | 24 | 无 |
| b2 | S0 | 131–144 | 14 × 25 | 277 | 67 | 0 | 无 |
| b2 | SX | 231–244 | 14 × 25 | 274 | 150 | 0 | 无 |
| b2 | SK | 331–344 | 14 × 25 | 269 | 51 | 0 | 无 |
| b2 | SR | 405–408 | 4 × 8 | 23 | 1 | 23 | 无 |

合计（命令数出来：`grep -E 'RESULT B tally \S+ checks=' expected-summary.txt` 各行相加）：判定 1 724 次（含 8 次起点），I-9.29「成立」1 724 次、别的判定 0 次；暴露 546 次，即 546 次判定里门开着、按实例表滤掉了一版会让 I-9.29 红的被抛弃版；两片实例表上判 47 次；旗 0 条（`grep -cE 'RESULT B tally \S+ flag '` 为 0）；140 段历史每段 `flags=[]`。动作做成 / 被拒：建 309 / 0、意图 251 / 0、删一批 29 / 152、走空 11 / 118、回退 307 / 131、再挂（含连挂 5 次）332 / 0、崩溃恢复抛弃 224 / 24、建号再抛弃 253 / 28。

九条之外红的只有 I-3.1，221 次判定，全在做过崩溃恢复抛弃的段里；与 m3-batch22-code-r1 判决「附带」那一行、m3-batch24-code-r1 判决「附带：I-3.1 在崩溃恢复落到前一条根的历史里红 14 次」同族，已转根环线，不归这一轮。

读码补一句为什么合法流上门永远是开的（推的）：门关只有主走读在最新根实例表的末片之前停下一种来由；合法发布先写齐单元、过屏障、再 FUA 根槽，所以崩溃状态里落了盘的最新根，它的实例表两份都在、都对得上；实例表清扫只删环里已经没有根的实例的行（`crates/singlefs-core/src/instance_table_sweep.rs` 文件头第 1–9 行），环里有一槽读不出或全零时那一轮一行都不删。B 段 1 724 次里门关 0 次与此相符。

**崩溃状态**：一个都没跑。门关要最新根的实例表走不到末片，合法流的崩溃状态里碰不到（上一段，推的）；另外一次发布在层 0 的枚举域里就有约 2²⁰ 个状态（m3-batch24-code-r1 攻方报告 3.3 节数过 1 048 583，那是当时的数，只作参考），过约 10⁶，照派发规则不跑。B 段因此只判每一步做完之后的镜像。

估时与分批：探针 S0、SX 各 2 段 × 8 步，判一次约 0.12 秒；SR 起点 370 次再挂一次 146 秒。每批四件并行（每件单线程测试、`capped.sh 2`），按探针估一件 1–3 分钟；实测 b1、b2 两批墙钟各 3 分 16–21 秒（SR 那件 196 秒最长）。第一次在工作目录 w1 跑 c 段时有两件经内存包装退 252（别的会话占着 24 GiB，命令一行都没跑），那一次的输出不算，`rerun.sh` 加了等待（`RUN_WITH_MEMORY_CAP_WAIT_SECONDS` 默认 150）之后原样参数重跑。

## 四、上一轮「没打中（一次抽样）」的第二次抽样

### 4.1 m3-batch24-code-r1 N1 单故障：第二次也没打中

换了取样，不换判法（目标那一条要红、别的一条不许红）。类型段取第一次没取过的值 3、4、6、8、9、10、11、12、255、256、32768，另加条目身份三段（出生树、容器号、容器出生代）各单独加一；起点换成第一次没用过的四张：墓碑树一片 1 条删 4 个号（T2b，层级 2）、一片 2 条删 5 个号（T2k2，层级 2，第二层有一个节点只有一条条目）、谱系树回退 4 次（L2b，层级 2）、inode 树 400 个号（I0b，层级 1、两片叶）。树的走读与条目改写照抄第一次的原型。

冻结副本上的原样输出（`expected-summary.txt` 里 `[c-…]` 的 pool 与 tally 行）：

```
[c-T2b] RESULT C pool T2b kind=10 clean_violated=[]
[c-T2b] RESULT C tally T2b ok 84
[c-T2k2] RESULT C pool T2k2 kind=10 clean_violated=[]
[c-T2k2] RESULT C tally T2k2 ok 70
[c-L2b] RESULT C pool L2b kind=11 clean_violated=[]
[c-L2b] RESULT C tally L2b ok 84
[c-I0b] RESULT C pool I0b kind=2 clean_violated=[]
[c-I0b] RESULT C tally I0b ok 28
```

266 形全部只红目标（`grep -c 'RESULT C shape HIT'` 为 0）。第一次 265 形、这一次 266 形，取样不重叠，两次都没打中，按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`：「至少再抽一次」，两次都没打中，记「没打中」。限度：两次都只造到层级 2–3、都是单故障；第一次打中的 H1（两故障、陈旧副本在盘 0）不在「单故障」这一格里，这一次没再造。

### 4.2 m3-batch24-code-r1 N3（合法历史上新九条的理由）：第二次也没打中

B 段同一批判定顺带核了 N3（第一次的判法：缺省理由、九条红、I-9.24 报不适用、「不适用」那句话按现行那一版的真值核）。起点（多了 SX 与两片实例表的 SR）、动作（多了「建号再抛弃」与「连挂 5 次」）、种子都与第一次不重叠。1 724 次判定，旗 0 条。九条各判定的次数（命令数出来）：

| 不变量 | 成立 | 不适用（专门理由） |
|---|---|---|
| I-9.17 | 1 057 | 667 R-no-lineage-entry |
| I-9.18 | 946 | 778 R-no-queue |
| I-9.19 | 946 | 778 R-no-tombstone-naming-record |
| I-9.20 | 1 012 | 712 R-no-tombstone |
| I-9.21 | 0 | 1 158 R-no-non-rightmost（产品入口）、566 R-knob-product-not-judged（旋钮入口） |
| I-9.22 | 362 | 1 158 R-old-entry（产品入口）、204 R-no-container-under-knob |
| I-9.23 | 1 012 | 712 R-no-tombstone-entry |
| I-9.24 | 1 724 | 0 |
| I-9.29 | 1 724 | 0 |

缺省理由 0 次、专门理由说假话 0 次。两次都没打中，记「没打中」。限度同第一次：产品入口下 I-9.21 一次都没评估过（要 816 条墓碑才有非最右的容器），崩溃状态没跑。

## 五、几个改法各修哪一格（都只在我的模型上量过或只是推的，被攻过零轮）

| 格 | 冻结副本 | F-N（门没开时按实例覆盖再判一次） | 改走读认「走过」的键（按交回的那一份） | GPU 链循环到头仍有下一片记判不了 |
|---|---|---|---|---|
| H-a：P0 第 1 片两份都坏 | 部分没判[门] | 成立（量过，`[fix-a]` 那一行） | 碰不到（推的） | 碰不到 |
| H-a：A5 / A6 / A7（N5-b 一族） | 部分没判[门] | 成立（量过）；N5-b 那条 harness 用例随之红（量过：`123 passed; 1 failed`） | 碰不到 | 碰不到 |
| H-a 的对照：P2 第 1 片坏、A2、A7 行重复 | 部分没判[门] | 仍部分没判（量过） | — | — |
| H-b：P4 盘 0 位置条目指回第 0 片 | 部分没判[门]，多红 I-3.9、I-5.1 | 仍部分没判（量过） | 走到末片、行收全，应转成立、不多红（推的，没写没跑） | 碰不到 |
| H-c：257 片 | CPU 不成立；GPU 不置位（推的） | 不碰 | 不碰 | GPU 转判不了、交 CPU，与 CPU 不再相悖（推的，没写） |
| B 段 1 724 次合法判定 | 全部成立 | 不改（门在合法流上从不关，F-N 那一支走不到；推的，没在 F-N 副本上重跑 B 段） | 推的：不改 | 不改 |
| C 段 266 形 | 全部只红目标 | 没在 F-N 副本上重跑（F-N 只改 I-9.29 的门，推的：不改） | — | — |

F-N 的代价与要交主 agent 判的：它把「门 = 主走读走到末片」放宽成「门 = 走到末片，或环里出现的实例的行都已读到且唯一」，依据是 I-3.8 的唯一性与「实例 0 不写行」两条；没读到的片里若有违反唯一性的重复行，由记录施加出来的版本可能多出来，F-N 在那种镜像上会比不该比的（推的，没造）。第 23 批判决已经把「读不出就报判不了」当偏保守的方向采了；F-N 是另一种取舍，不是对那一条的纠错。

## 各格判定一览之外：没打中的形状

| 形状 | 取样范围 |
|---|---|
| 走到末片而候选集没滤全（CPU） | A 段 41 形（43 行减去起点那 2 行）里门开的 26 形：单片表上插各种种类的记录、行重复、倒序、flags 非 0、声明长度带尾巴、载荷 CRC、单份坏；两片表行在哪一片、第 1 片完好或单份坏；256 / 257 片。每一形门开时候选集都按整张表滤（A8 两形是镜像上的表本身让 C 有效） |
| 没走到末片而滤全了（CPU） | 打中 H-a、H-b；另外门关的形状（A2、A12、A13、A14 宽 89、P2 / P6 第 1 片两份坏）都是该关的：有要滤的实例的行没读到 |
| CPU 与 GPU 判得不一样 | 打中 H-c；置位时机、合法性判法、提前出口、清零、交接段第 30 个字的读写与覆盖逐支对过（第二节表与 2.2） |
| 合法历史上 I-9.29 误红或无谓判不了 | B 段 140 段、1 724 次判定；四个起点（含被抛弃版与现行线同号不同出生代、实例表两片），九种用户动作全部放开 |
| m3-batch24-code-r1 N1 单故障、N3 | 第二次抽样，见第四节 |

## 这条腿自己的限度

- 一次抽样。H-a、H-b、H-c 是线索，要主 agent 现查；「没打中」的几格（合法历史、CPU 走到末片而没滤全）照规则还要再抽一次才记。
- 崩溃状态一个都没跑（第三节末段的理由是推的：合法流上门不会关）。
- GPU 不跑，H-c 的 GPU 期望是读码推的；F-N 的 GPU 那一半没写。
- H-a、H-b、H-c 都只在坏镜像上造得出，合法流上碰不到；它们的分量交主 agent 定。
- SR 起点只有 8 段 × 8 步（每件要先挂 370 次、146 秒），两片实例表上的合法判定只有 47 次；三片及以上的合法表没造（要 738 次以上的再挂）。
- 被抛弃的根只用「系统配置没见证、根槽与点名单元一时读不出」这一种造法（照 `common` 的那一个）；第 22 / 24 批攻方原型的动作表我照抄了一部分，它造不出的历史（文件很大、多头）这里也没有。
- 判定口径里「暴露」只看现行那一版的号，与被抛弃版逐号比；候选集里别的版同号不同出生代没单数。

## 没做什么

- 不判正推那几格（K1、K3）；不替主 agent 采纳；F-N 只在我的副本上量过、被攻过零轮，没写进 `crates/`；副本上的数不算入库装置上的数。
- 没跑 checker 档、GPU、变异表、门禁；`checker_known_bad_images` 只在 F-N 副本上跑过一次（harness 档）。
- 改走读认「走过」的键、GPU 链循环到头记判不了：都只是推的，没写、没跑。
- 草稿目录 `/tmp/claude-1000/m3-batch25-code-r1-opus/` 里的仓副本、编译目录交回前删掉（逐个列在交回里）。
