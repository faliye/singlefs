# m4-gpugc-precheck-b-a-code-r3：正推腿（Sonnet）报告

背景材料 `research/prompts/_m4-gpugc-precheck-b-a-code-r3-background.md`，改动全文 `research/prompts/_m4-gpugc-precheck-b-a-code-r3-diff.md`，冻结副本 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3/repo/`。
读法：「量的」是这一次我自己读文件、跑只读命令得到的；「推的」是读码推出来、没量过；「待主 agent 跑」是要起脚本才核得了的（第六节）。
冻结副本与主工作区里被判的三份产品文件、`mount.rs`、`transaction.rs`、`accept_raises_the_floor_for_space.rs` 此刻 sha256 前 16 位逐份相同（量的，`sha256sum` 两边比），所以下面引的 `crates/…:行号` 两边是同一行。

## 要主 agent 跑的命令（摘要，整条在第六节）

模型目录 `research/prompts/m4-gpugc-precheck-b-a-code-r3-sonnet-model/`，两份旧探针在今天这棵树上的重跑（含一处新增计数）。复跑入口 `tools/rerun-from-a-clean-work-directory.sh`。
模型目录每个文件的 sha256（量的，`sha256sum`）：

```
f45aed5da6126bd3b7e8f220e0c29bc2736d9a319f65d6dcb99a55a6204327f6  probe-fill-then-fragment/Cargo.toml
86928a59d589e645ac9401f453b1f687749ee7bf45e0018af88866dd51caa4de  probe-fill-then-fragment/core_probe_module.rs
81a8ae9934428a5a09ec1f59cc0f0567e49eb41aa7135ab584d9f956338959a3  probe-fill-then-fragment/src/device.rs
a911d3c722e8f067de5fc6a398d365bcbb1510a896f84a133a52022fc9f901c6  probe-fill-then-fragment/src/main.rs
e4125643126be91539e6a445988eb0719c53d82f89c8429260d2dc3ddeb7b4ac  probe-fill-then-fragment/src/pool.rs
001540ead70dff75773fb60d5e862f06207affd1de8727ea7da305b0303535b1  probe-fill-then-fragment/src/shapes.rs
f2b6e02be3598a75c598cbacc48402cd8741697093bafbfb796b53f3ee3dca0f  probe-fill-then-fragment/src/tally.rs
879caa27846b1df02d9b5d96b28db0deb03c1a47f99e518be4a7a0a754bd1bb5  probe-random-history/Cargo.toml
a70215ff93d16f77a5e39b14cfd3c40989adb56b12675a55bfde016b4cc2fef9  probe-random-history/core_probe_module.rs
81a8ae9934428a5a09ec1f59cc0f0567e49eb41aa7135ab584d9f956338959a3  probe-random-history/src/device.rs
0bebbb9358964da3486675d3ac61dc111ab92438035ca4cbded7c7ddcc227582  probe-random-history/src/main.rs
e4125643126be91539e6a445988eb0719c53d82f89c8429260d2dc3ddeb7b4ac  probe-random-history/src/pool.rs
294999a0ee99723e688cb77e674a1a24a881ba6d1b0017c07d6928b6020d4f48  probe-random-history/src/shapes.rs
c3485b63ff0310507a3f6123d2b45762b4e2489f4843f60586b70f259e8031f4  probe-random-history/src/tally.rs
f6b4858de60e136e4c81d99688d4897c4bd8d6e2f19efbbfa22e1b35453b6e48  reference/r2-fill-then-fragment.PROBE_RESULT.txt
ea6bee6e4d1040f55023f92c642074eb6072d09f93e39ebca579229f079753ee  reference/r2-random-history.PROBE_RESULT.txt
c86caf5cd7c2225376353f2cf7e6de0a40674b5a73ba993b1f61b4aacd9e962e  tools/compare_probe_result.py
05ddc176be772be661ac151064ae8642bbc94e4c2014b3f886aaf55b12b6b3f8  tools/patch_the_copy_fill_then_fragment.py
658123b2467e53c7d42e7d831eb8efed96757c336f264cf2568b84b7fd61fc7a  tools/patch_the_copy_random_history.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
68927314a2c38ed6a3bea82658efdf481a59782d987fc72caeebfc6c0398640e  tools/rerun.sh
```

模型是第二轮两份探针的拷贝加三处改动：两份 `main.rs` 的估时上限 40 分钟改 100 分钟；第二族探针（`probe-fill-then-fragment`）`shapes.rs` 加一行 `fired by=<判> oracle-series=<n>` 计数；第一族探针的 `patch_the_copy_random_history.py` 里老上界那一臂的 `unblocked_slot_pairs_outside_the_cluster_segments(…)` 后加 `.0`（今天它返回 `SlotPairCount`）。
语法核（量的）：`bash -n tools/rerun.sh` 过，三份 `.py` 的 `python3 -m py_compile` 过。**没编、没跑**：改过的探针在今天的树上能不能编过，待主 agent 跑。

## 一、T3：报出的数字能不能核（主攻）

被核的两份产物（量的，`sha256sum` 全长）：
- 第一族（随机历史，310 配置）`…-r2-main-check-sonnet-run/out/oracle.out`：`32c69feb2c70caaa5fae…`，157 行，前缀对上正文的 `32c69feb2c70caaa`。
- 第二族（先填满再打碎，155 配置）`…-r2-main-check-opus-run/out/oracle.out`：`30a9b0ce5b288e410f07…`，335 行，前缀对上正文的 `30a9b0ce5b288e41`。
下表「行」是产物文件里的行号（`grep -nF` 现取）。

### 1.1 正文第二节那几条（逐个核）

| 数 | 产物与行 | 原样 | 对不对 |
|---|---|---|---|
| 冻结副本三份 `sha256sum -c` OK | 副本里对 `m4-gpugc-precheck-b-a-code-r3-snapshot/crates-sha256.txt` | `crates/singlefs-core/src/allocator.rs: OK` 等三行 | 对（量的） |
| 证红 14814「跑了 3 条，抓到 3 条」 | 任务 14814 日志第 629 行 | `✓ 点名 3 条：现判 3 条、复用 0 条、拒算 0 条；跑了 3 条，抓到 3 条（其中复用 0 条），本次未跑 0 条，跑了的都抓到了` | 对；这 3 条里有重建用例那一条（第 625 行） |
| 证红 14846 | 任务 14846 日志第 274 行 | 同一句，3 条 | 对 |
| 变异复判 15050「抓到 5 条、没红 0 条、无效 0 条、锚点失效 0 行」 | 日志第 66 行 | `计数：抓到 5 条、没红 0 条、无效 0 条、锚点失效 0 行（共 5 条）` | 对。**正文没写**：第 68 行 `复用：现判 1 条、复用 4 条`，5 条里 4 条是按键复用上次的抓红、不是这一趟现判 |
| 合入后验证 15051「164 条；现判 164、复用 380、拒算 0」、clippy、fmt、build | 日志第 1233、1769、1300、2860 行 | `测试全绿（6 个测试二进制，共 164 条测试；跑了 6 项（2 个包）；按条复用：现判 164、复用 380、拒算 0）`、`clippy 过（…）`、`fmt 过`、`build 过` | 数对。**树对不上**：15051 拷的是拷快照那一刻的主工作区（日志第 57 行）；记录第 1049 行说另一个会话之后又打了一份只动 `allocator.rs` 测试模块与一处 `expect` 消息的补丁（任务号 15255 比 15051 大），冻结副本的 `allocator.rs` 是打完之后的 `eb822f09…`。产品逻辑同，但 `allocator::tests` 里（含新重建用例）在 `eb822f09…` 上没有绿的记录（推的，顺序按任务号推）。命令在第六节 |
| 崩溃点重放 13913「三项判器红数 0，不算门禁通过」 | 日志第 8、9、11 行与第 15–17 行 | 三行放量都是 `red=0`、`expand_up_to=4`、`verdict=red`；`truncate-to-zero-on-a-nearly-full-pool` 一行 `enumeration_red=1`；总行 `red=3` | **半对**：`red=0` 是放量那一格的判器红数；任务整体三项都 `verdict=red`、退 1，原因是缩域后有段落在展开之外（第 8 行 `unregistered_segments_beyond_expansion=…`）。「第 126 批那一版上跑的」：日志里运行目录名带 `20261111`，在第二轮（2026-11-12）之前（量的），哪一版代码日志里没写 |
| 第一族前 30 个字段与第一轮 13926 逐字段相同 | 第一轮 `…-r1-main-check-opus-run/out/oracle.out`（sha256 前缀 `82a71415e825c942`）对第二轮第一族第 152 行 | 我拿两行的 `key=value` 逐个比：前 30 项相同，第 31 项起第一轮是 `panics`、第二轮是新加的 `configs_fired_slot_pairs` | 对（量的；实际前 31 项相同） |

### 1.2 第二轮判决第一节抄的整行

| 判决里的数 | 产物行 | 原样 | 对不对 |
|---|---|---|---|
| 第 16 行 `oracle-series=0 825`、`=1 1082` | 第一族第 95、96 行 | `oracle COUNT fired by=slot-pairs oracle-series=0 825`、`… oracle-series=1 1082` | 对；825 加 1082 等于 `fired_by_slot_pairs=1907`（第 152 行），没有 `=2`、`=3` 档 |
| 第 17 行 `old-bound=` 三行 | 第 92、93、94 行 | `fires-by-conjunct-before-any-series 44`、`fires-by-slot-pairs 620`、`pushed-then-no-space-wide-protection-not 1243` | 对；三项相加 1907 |
| 第 18 行 `remount published … old-bound=` 两行 | 第 117、118 行 | `…fires-by-conjunct-before-any-series 2`、`…pushed-then-no-space-wide-protection-not 10` | 对；10 加 2 等于第 113 行 `oracle COUNT remount published by=slot-pairs 12` |
| 第 19 行 `count-below-floor=yes 62` | 第 87 行 | `oracle COUNT fired by=slot-pairs count-below-floor=yes 62`；第 86 行 `=no 1845` | 对；62 加 1845 等于 1907 |
| 第 20 行 `conjunct-first refused 413`、`forced=NoSpaceAfterRaisingTheFloor 413`、`conjunct_first_forced_published=0` | 第 51、45 行；第 152 行的字段 | 413、413、0 | 对 |
| 第 21 行 `empty-segments=lowered 6846`、`raised 24`、`same 24349` | 第 134、135、136 行 | 同 | 对；三项相加 31219，等于 `series_product=31219` |
| 第 21 行 `series_product_short_before=0`、`series_oracle_short_before=59 …_raised=0`、`series_forced_short_before=362 …_raised=0` | 第 152 行字段 | 同 | 对；产物里还有 `series arm=oracle short-before empty-segments=same 59`（第 132 行）、`series arm=forced short-before empty-segments=same 362`（第 126 行）：这 421 串**全是「不变」**，不只是「没推高」 |
| 第 22 行 `configs fired by=conjunct-before-any-series 2` | 第 32 行 | 同 | 对 |
| 第 25 行第二族整行 | 第二族第 330 行 | 我把判决第 25 行的反引号里那一整行存成文件，与第 330 行 `cmp`：**逐字节相同**（1130 字节） | 对（量的） |
| 第 14 行 sha256 前缀、自证 `bad=0`、310 与 155 个配置 | 第一族第 157 行、第二族第 335 行 | `name=self_checks bad=0`；`configurations=310`、`configurations=155` | 对 |

### 1.3 判决里算出来的三个数：算法与复算

**有效分母 1082（第一族）**
- 算法（读 `research/prompts/m4-gpugc-precheck-b-a-code-r2-sonnet-model/probe/src/shapes.rs`：`fired by=<判>` 在 :286–:287，`oracle-series` 在 :323–:327，`misrefused` 在 :346–:348）：槽对那一判判出的格，在拷贝上跑 O 那一臂（先算整个关掉、被拒就推），记它推成几串（封顶 3）；O 臂最后写进才记 `misrefused`。O 臂一串都没推成（`oracle-series=0`）的格，产品臂与 O 臂都是「没推就拒」，上界是大是小看不出来；有区分力的只有 O 臂推过串的格。
- 复算（量的，第一族第 95、96 行）：1907 − 825 = 1082，且 825 + 1082 = 1907；`misrefused_slot_pairs=0`（第 152 行）。**对。**
- 口径提醒（量的）：这 1907 格落在 `configs_fired_slot_pairs=212` 个配置里（第 152 行字段），不是 1907 个独立样本。

**1039（第二族）与判决把它叫「有效分母」**
- 第二族产物里没有 `oracle-series` 这一族计数：`grep -c 'oracle-series'` 第二族 `oracle.out` 得 0（量的）。1039 是 `fired_by_slot_pairs=1039` 的原数（第 330 行），没有按「O 臂推过串」扣。第一族同一口径扣掉了 43%（825÷1907，算的）。
- 第二族 O 臂推成的串共 `series arm=oracle 1188`（第 104 行），有推过串的格 `oracle pushed-more-series-than-the-product=1181`（第 330 行字段），分在槽对 1039、式子 312、合取 7 3643 三种判里，**产物拆不出 1039 里有几格推过**。所以「有效分母是 1082 与 1039」两个数口径不同：1082 是扣过的，1039 没扣，真有效数 ≤ 1039，也 ≤ 1181 减去另两种判里推过的格。判：**1039 只能叫原始分母**；有效分母要重跑才有（第六节命令，第二族探针加了一行计数）。
- 口径提醒（量的）：第二族 155 个配置只有 8 个互不相同的填满起点：`COUNT fills 8`（第 58 行），探针用 `seed % 起点数` 选起点（`research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model/probe/src/shapes.rs:537`）。1039 与 3643 都出自这 8 个起点上的随机步，「两份互不相同的取样」在配置这一层第二族只有 8 个起点。

**0/421**
- 算法：`short-before` 是一串抬 F 推之前某一块盘的全空段数少于 `N₀`（第一族探针 `shapes.rs:164`）；`raised` 是推之后任一块盘的全空段数变多（`shapes.rs:155`）。421 = O 臂的 `series_oracle_short_before=59` + 合取 7 自己先拒的 413 格强推一臂的 `series_forced_short_before=362`（第一族第 152 行字段）；两个 `…_short_before_raised` 都是 0。
- 复算（量的）：59 + 362 = 421，推高 0 + 0 = 0。**对。** 产物更强：`series arm=oracle short-before empty-segments=same 59`（第 132 行）、`series arm=forced short-before empty-segments=same 362`（第 126 行）——421 串全是「不变」；`series_oracle_crossed_the_floor=0`、`series_forced_crossed_the_floor=0`（第 152 行字段）。
- 口径提醒（量的）：① 421 是两个臂的串数相加；② 362 串出自 `configs_conjunct_first_refused=21` 个配置（第 152 行字段），59 串出自几个配置产物没拆，独立样本数远小于 421；③ 产品臂的 0（`series_product_short_before=0`）是结构的：这一条件下产品臂不推。判决已更正为引 0/421，这一点对。

**62 格**
- 算法：槽对那一判判出的格，产品臂发这次改动之前取一次每块盘的全空段数 `empty_before`，任一块少于 `N₀` 记 `count-below-floor=yes`（`shapes.rs:296`–`:298`）。
- 复算（量的）：62 + 1845 = 1907（第 87、86 行）。**对。**
- 「报的都是 `devices_short`」：按构造——`crates/singlefs-core/src/mounted_session.rs:6237` 先判盘上界、判出就带 `None` 交回，不再看合取 7（`judge_before_pushing_a_floor_raise_for_a_refusal` 在 :6228）。取数时刻与判的时刻是同一个数：这 1907 格 `versions-published=no` 全成立（第 105 行 `oracle COUNT fired by=slot-pairs versions-published=no 1907`），调用开头到被拒之间没有走空发成的版。

### 1.4 两批判别子探针：记录里的整行与产物逐行对

目录 `~/.local/share/singlefs/work/m4-gpugc-precheck-remount-outlet-ab/` 与 `…-ab-2/`，各有 `random-history-run`（第一族）与 `fill-then-fragment-run`（第二族）。四份 `oracle.out` 的 sha256 前 16 位：`494e8699ed25d23f`、`f8467e201e8626d1`（ab）、`b8c07c4599d7f0a9`、`2035afafa3c35606`（ab-2），与记录里的前缀逐个相同（量的）。四份自证行都是 `name=self_checks bad=0`。

| 记录里的话 | 产物行 | 对不对 |
|---|---|---|
| 第 1043 行：第二族槽对 1039 格、重挂写进 187，甲一、甲二 `true_published=187 true_not=852 false_published=0 false_not=0`，误报 82% | ab 第二族第 376、377 行；852÷1039 = 0.820（算的） | 对 |
| 第 1043 行：合取 7 3643 格写进 32，甲一、甲二、乙二恒真（`true_not=3611`），乙一恒假；式子 312 格写进 0 | 第 380–383 行；第 378、379 行 | 对 |
| 第 1043 行：合取 7 3339 格卸载被拒；槽对 174 格卸载被拒、95 格挂不上、454 格重挂后被合取 7 拒 | 第 116 行 3339；第 141、140、139 行 174、95、454 | 对 |
| 第 1047 行：第一族槽对 1907 格、重挂写进 12，甲一、甲二 `12/1895/0/0`；式子 11514 格写进 0；合取 7 4 格写进 2 | ab 第一族第 185、186 行；第 187、188 行；第 189–192 行 | 对 |
| 第 1047 行：重挂没写进的 1895 格里 1811、69、15 | 第 132、133、131 行；1811 + 69 + 15 = 1895（算的） | 对 |
| 第 1050 行：第一族丙一 `10/1835/2/60`、丁 `12/0/0/1895`；第二族丙一 `127/254/60/598`、丁 `187/0/0/852`；丙二与丙一逐行相同；合取 7 丙一恒假、丁第二族 `32/…/3611` | ab-2 第一族第 289–291 行；第二族第 490–492 行；第 522–524 行 | 对 |
| 第 1050 行：丁的挂钟中位数第一族 1391 槽 72.9、1455 槽 78.2、1647 槽 113.3、2415 槽 175.1 毫秒；第二族 1391 槽 48.8、1647 槽 71.1、2415 槽 93.6 毫秒 | ab 第一族第 338–341 行（`walk_median_ns=72885369`、`78213797`、`113251339`、`175058665`）；ab-2 第二族第 539、540、542 行（`48795116`、`71143813`、`93591674`） | 对 |
| 正文 T3 格写「记录第 1047、1050 行点名的任务 14992、14986、15495、15496」 | 14986 在记录第 1043 行（第 1047 行只写 14992，第 1050 行写 15495、15496）；记录第 1048 行自己写的是「记录第 1043、1047 行」 | **正文行号漏了 1043**，数对 |

### 1.5 探针在第二轮冻结副本上量的，搬到今天的树上哪几个还能引

今天的树与第二轮冻结副本三份被判文件逐份比（量的，`diff` 行数）：`mounted_session.rs` 35 行（全是文档与一处测试里的 `SlotPairCount(0)`）、`admission.rs` 38 行（类型与两处文档）、`allocator.rs` 1300 行。`allocator.rs` 的产品代码变了三样：用户数据落点按段取（`crates/singlefs-core/src/allocator.rs:1320` `lowest_user_data_slot_outside_the_region`，加了 `segments_with_an_unblocked_slot_pair`）、回落的名字与类型、`SlotPairCount` 换齐；其余约 580 行是新加的库单测。两个槽对计数函数（`:870`、`:893`）、`note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts`、`rebuild_from_records`（`:2670`）在 `diff` 里没有改动的块（量的：块范围对照）。两份探针的补丁锚点在今天的树里各命中恰好一次（量的，`anchor_count.py` 在草稿目录），所以能拼，但第一族补丁里老上界那一臂 `SlotPairCount(u64 + …)` 在今天编不过（`SlotPairCount` 只有 `Sum`，`allocator.rs:422`），我在模型里加了 `.0`。

| 读数 | 判 | 理由 |
|---|---|---|
| 乙误拒 0/1082、0/1039；甲误拒 0/3643 | 旧树上量的；读码上今天同样成立（推的） | O 臂写不写得进取决于落点；今天落点换了算法；`allocator.rs:5458`–`:5787` 一族用例（`user_data_slot_found_by_segment…`、`placement_by_segment_against_stepping…`、`segments_with_an_unblocked_slot_pair…`）把按段取法与逐候选取法对拍，我没跑它们。要在今天的树上重量才能写「量的」（第六节） |
| 0/421、62 格 | 旧树上量的；读码上今天同样成立（推的） | 数的是全空段数在推串前后的变化，推串的空发布走提交内生块落点，今天只改了名字与类型，没改取法 |
| 41544 步按记录重建 0 步不同 | 旧树上量的；今天同样成立的把握最大（推的） | 比的量与 `rebuild_from_records` 在 `diff` 里都没动；落点算法不在这条路上。但探针比的是（段外槽对数两种、全空段数）三样，不是新用例比的三样（见第二节第 4 条） |
| 老上界对照（620、1243、44；重挂 10、2） | **永远标旧树上量的** | 老上界算法已不在今天的树里，探针补丁里的拷贝在今天编不过，只能在改过的探针上重量 |
| 重挂写得进 12+2、187+32 与丙一、丙二、丁的四格表 | 旧树上量的 | 要走卸载、可写挂载、再发同一改动，三步都过落点；而且这是另立一题的判别子，不是乙、甲的读数 |
| 丁的挂钟中位数（毫秒） | 旧树上量的，最该重量 | 今天的落点函数文档（`crates/singlefs-core/src/allocator.rs` 第 1310 行起那段）写明代价不随单元区的槽数、也不随已占的槽数长，旧树上的逐候选取法随槽数涨，这组数多半高估今天的代价（推的） |
| 任务 14814、14846、15050、15051 | 副本、主工作区上量的 | 不是探针；15051 的树见 1.1 表 |

## 二、T1：「已定」项撑不撑得住，与第 131 批四条修补逐条对代码

### 2.0 乙、甲在今天这棵树上（正推；读码，没跑）

**乙与落得下那一判是不是同一把尺。** 两个数是两张各自增量维护的表，读法同一个减法：
- 落得下读 `unblocked_slot_pairs_outside_the_cluster_segments`（`crates/singlefs-core/src/allocator.rs:870`），由 `PlacementRoomOnOneDevice::of_every_device_of_the_allocator`（`crates/singlefs-core/src/admission.rs:995`）取；乙读 `slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`（`allocator.rs:893`），由 `mounted_session.rs:6068` 起的 `SlotsAFloorRaiseCouldRelease::of_the_device` 取。两边都是「总数减 `pairs_inside_the_cluster_segments`（`allocator.rs:579`）」，传的是同一个 `cluster_segments`。
- 别的会话改的按段取落点只新加了 `segments_with_an_unblocked_slot_pair`（`allocator.rs:485`），在 `set_blocking_bit` 里随 `unblocked_slot_pairs_per_segment` 在 0 与非 0 之间翻时加摘（`allocator.rs:775`–`:783`）；两个计数的读法与自己的维护没改：`set_blocking_bit` 里那一块只在原有的计数加减后面加了集合的插入与摘除，`note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts`（`allocator.rs:986`）没有改动的块（量的，`diff` 块范围对照）。落点函数另排除活意图 `R` 所在段（`allocator.rs:1320` 起），两个计数不排 `R`：落得下那一判可能多放行，方向是放行，不会让乙的上界显得小。
- **乙的上界不小于落得下放行的数**（推的，没有库单测把这两个数直接放在一起比）：罩位 `is_under_an_unreleased_record` 只在 `mark_allocated`（置 `Allocated` 在 `allocator.rs:1287`，置罩位在 `:1291`）与复活（`allocator.rs:1032`，`:1035`–`:1038` 断言槽已分配）里置；`Allocated` 只在 `mark_reclaimed`（`allocator.rs:1193`）里清，回收的是已释放、罩位已清的槽。所以「没挡的槽对」是「两槽都不罩在未释放记录下的槽对」的子集。现有的相关用例是乙对「真放开之后」的下界那条（`allocator.rs:4203`）。

**甲读的全空段数与合取 7 是不是同一个数。** 是：两处都读 `DeviceFreeMap::empty_segments`（`allocator.rs:1255`）、`N₀` 都取 `allocator.compaction().switches.empty_cluster_segments_floor.segments()`、都调 `admit_the_empty_cluster_segments_floor_on_every_device`（`admission.rs:947`；先算在 `mounted_session.rs:6211`，发布路径在 `crates/singlefs-core/src/transaction.rs:13472` 起的块里）。两处差别不是本轮改的：发布路径只在 `publish_has_an_ordinary_allocation`（`admission.rs:1657`）为真时判（`transaction.rs:13461` 起的 `match`），先算按改动类整类判（`mounted_session.rs:6158`），并要 `is_admission_judged`（`mounted_session.rs:6195`）。第二轮探针 `misrefused_conjunct_before_any_series=0` 量过这一差别，但在旧树上、8 个起点上（1.3）。

### 2.1 第 1 条：四处文档与代码逐句对

| 位置 | 文档说的 | 代码 | 对不对 |
|---|---|---|---|
| `crates/singlefs-core/src/mounted_session.rs:658`–`:660`（`UserChangeRefused` 那个成员） | 「抬 F 之前先算判出这一次不推，两形都在」（`crates/singlefs-core/src/mounted_session.rs:658`）；「合取 7 那一形是这次调用还一串没推、写之前那一刻全空聚簇段已少于 `N₀`」（`crates/singlefs-core/src/mounted_session.rs:659`）；「带 `N₀` 与少于它的盘」 | 先算在 `judge_before_pushing_a_floor_raise_for_a_refusal`（`mounted_session.rs:6228`）；合取 7 那一形要 `series_pushed_in_this_change > 0` 为假（`:6199`）、读每块盘的 `empty_segments`；带的是 `EmptyClusterSegmentsBelowTheFloorOnSomeDevices`（`admission.rs:935`，含 `floor_in_segments` 与 `short_devices`） | 对；**少一个前提**：被拒的得是合取 7 判的前台写入类（`mounted_session.rs:6158`），删除、截断、建快照照推 |
| `:687`–`:691`（`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 结构体） | 「槽对与式子那两判（`devices_short`）是推抬 F 放得出的全放出来也放不下：每块列出的盘上界都小于需求」（`crates/singlefs-core/src/mounted_session.rs:688`）；「合取 7 那一形（`empty_cluster_segments_below_the_floor`）是这次调用还一串没推、写之前那一刻全空聚簇段已少于 `N₀`，被拒的又是合取 7 判的前台写入，不推」（`crates/singlefs-core/src/mounted_session.rs:689`） | `device_short_even_with_everything_a_floor_raise_could_release`（`mounted_session.rs:6102`）只列上界小于需求的盘；提交内生块那一判交 `None` 算放得下（`mounted_session.rs:6097`） | 对，前提写全了（这一处是四处里唯一写了前台写入类的） |
| `:2939`–`:2940`（`publish_user_change` 文档） | 「合取 7 那一形是这次调用还一串没推、写之前那一刻全空聚簇段已少于 `N₀`」（`crates/singlefs-core/src/mounted_session.rs:2940`） | 同上 | 对；同样少前台写入类那个前提 |
| `:6832`–`:6833`（`AcceptRefused::RefusedAfterRaisingTheFloorForSpace`） | 「合取 7 那一形是这次调用还一串没推、写之前那一刻全空聚簇段已少于 `N₀`（头一次被拒就判到这里时 `floor_raises` 是空的）」（`crates/singlefs-core/src/mounted_session.rs:6833`） | 头一次被拒就判到这里时没推过串，`floor_raises` 空 | 对；同样少前台写入类 |
| 文件头 `mounted_session.rs:6`–`:7` | 「推串之前全空聚簇段已少于 `N₀`、被拒的又是前台写入时同样不推、同一个成员」（`crates/singlefs-core/src/mounted_session.rs:7`） | 同上 | 对，本轮没动，已分两形 |

- 旧句「推了也写不进」在 `crates/` 下 `grep -rnF` 零命中（量的，冻结副本）。
- 三处文档（:658、:2939、:6832）把「被拒的是合取 7 判的前台写入」漏在外面，读的人会以为任何被空间拒的改动在全空段少于 `N₀` 时都先算判出；其实只有 `ForegroundWriteTheConjunctJudges` 那一类（`mounted_session.rs:6158`）。字面不假，是漏了前提；归「没去向」第 T2-6 条。
- :660 的「头一次被拒就判到这里时它们是空的，盘上逐字节不变」本轮没改。调用开头先走空发成几步时这句字面不成立，C1003 已登（T2-3）。

### 2.2 第 2 条：`SlotPairCount` 换齐之后 `admission.rs` 里的比较

- 返回值：`allocator.rs:870` 起的函数现在交回 `SlotPairCount`（`allocator.rs:873` 的 `) -> SlotPairCount {`）；字段 `crates/singlefs-core/src/admission.rs:980`：「pub slot_pairs_for_data_units: SlotPairCount,」。
- 比较（`admission.rs:1077`）：「if room_on_the_device.slot_pairs_for_data_units < SlotPairCount(units.data_units) {」。左边是这块盘段外没挡的槽对数，右边是这次要落的数据单元个数；一个数据单元落一对（`allocator.rs:417`–`:418` 的类型文档），所以右边包成 `SlotPairCount` 是对的。`#[derive(…PartialOrd, Ord)]`（`allocator.rs:419`）单字段，次序与原 `u64` 比较相同，`<` 语义没变。
- 换成槽数那一处（`admission.rs:1079`–`:1081`）：`.slot_pairs_for_data_units`、`.0`、`.checked_mul(slots_of_a_data_unit)`——槽对数乘每个数据单元的槽数，得槽数，再交 `bytes_of_slots`（`admission.rs:1070` 的闭包）；乘数与原来同一个 `slots_of_a_data_unit`。乙那一侧同样的换算在 `mounted_session.rs:6094`（`slots.slot_pairs_once_everything_is_released.0 * slots_of_a_data_unit`，字段本来就是 `SlotPairCount`，`:6060`）。
- 核心包里读这两个名字的地方就这几处（量的，`grep -rn` 冻结副本）；E180、E181 与验证包测试都在读的地方加 `.0` 拆回 `u64`，所以这些文件里槽对数还以裸 `u64` 活着（检查单「类型与名字」）。

### 2.3 第 3 条：走空那条真池用例补的三句断言

用例 `a_walk_step_refused_for_space_while_empty_cluster_segments_are_below_the_floor_is_pushed_for`（`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1559`）。三句新断言的消息在 :1610、:1616、:1621：不再推的原因是 `FloorRaiseStop::FloorRaiseRefusedForSpace`；`floor_raises` 与 `refusals_that_pushed_the_floor_raises` 空；`versions_published` 空。池是 `mark_every_free_slot_allocated`（`:659`）把单元区空槽全占满之后造的：走空的头一步被拒、抬 F 那一串自己也放不下，三句都与这个池一致（读码）。三句本身没有各自的变异行（推的）：这条用例已有的一行变异是「走空被拒的一步交成合取 7 判的前台写入」，14846 第 272 行记「抓到」，红点是用例原有的断言（量的）；新三句防的是「推串之前先算判出」之外别的偏差（多推了串、多发了版），没有弄坏它们的行，没被单独证红。

### 2.4 第 4 条：重建之后比计数的用例钉的是不是重建走的那条路

- 用例 `slot_pairs_once_everything_is_released_are_the_same_after_rebuilding_the_allocator_from_its_records`（`crates/singlefs-core/src/allocator.rs:4228`）在 600 步历史（`allocator.rs:4103` 的 `for step_number in 1..=600 {`）的每一步之后调 `PoolAllocator::rebuild_from_records`（`:4231`），两块盘都比。
- **是同一条路**：产品重开分配器就调同一个函数，`crates/singlefs-core/src/mount.rs:3248`（「PoolAllocator::rebuild_from_records(device_maps, output.allocation_records.clone()),」）与 `mount.rs:3762`；函数体对每条记录先 `mark_allocated`、已释放的再 `mark_released`（`allocator.rs:2683` 的 `if record.is_released {`）。唯一差别：用例建空闲图用 `DeviceFreeMap::new`（`allocator.rs:4235`，默认环起点），产品用 `with_unit_area_start`（`mount.rs:3242`）；起点只改位图下标原点，计数公式不依赖它（推的）。
- 判别力：变异行把 `if record.is_released {` 改成「`&& span == 1`」时，重建出来跨度 2 的已释放落点不进 defer；14814 第 625 行记它「抓到」（量的）。
- **比的三样与判决写的「三个槽对计数」不是同一组**（量的，对着第二轮探针 `rebuild_from_records` 那段 `research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model/probe/src/shapes.rs:350` 起）：探针比（不排聚簇段的乙数、排这次挂载聚簇段的乙数、`empty_segments()`）；用例比（逐槽罩位 `is_under_an_unreleased_record`、每段乙数、总数）。乙那两个数由用例的后两样推出；**甲读的 `empty_segments()` 重建之后与重建之前相同，今天的树上没有一条用例比它**（`allocator.rs` 里 `empty_segments()` 的断言所在函数：3817 `unit_area_of_a_4_gib_image_has_211968_slots_in_3312_segments`、3866 `new_pool_file_creation_placements_follow_the_byte_table`、4266 与 4286 在 `accounting_deltas_are_maintained_incrementally` 里（4286 按行号推的）、4304 起 `segment_left_with_only_deferred_slots_counts_as_empty_by_the_records_while_its_slots_stay_occupied`，对象都是 `new` 或一路增量维护的分配器，量的 `grep -n` 加读上下文）。这一格只有旧树上的探针量过（41544 步 0 步不同）。
- 用例名没有缩写；写了「不并行」的理由（`allocator.rs:4226`：「不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。」），与同文件别的历史用例同一句。

## 三、T2：未定项清单的完整性（每一形写成「有去向 / 没去向」）

| # | 形 | 判 | 去向或该登哪里（证据） |
|---|---|---|---|
| T2-1 | C1002（先算对槽对那一判挡得少）、C1003（被宽护卡住那条出路没量过） | 有去向 | `.claude/kb/checks-owed.md:606`、`:607` 两行；去向 `.claude/kb/milestone/05-fifth-txn.md` 十一·4（第 185 行的欠一栏列着 C1002、C1003，量的）。两行的「前置」一栏还写着乙、甲「零轮」「分配器按段增量维护…（改法乙，零轮）」，代码已落地，三轮判完由写回员改 |
| T2-2 | 第一轮判决「交用户的」：乙、甲误拒只有一份取样，登欠账 | 去向有，**依据要补一句口径** | 第二轮判决「交用户的」第二条说那条欠账可以关；`checks-owed.md` 里没有对应的 C 号行（`grep` 先算只命中 C1002，量的），所以没有行要关。但「没打中」靠的两份取样，第一族有效数 1082 落在 212 个配置里，第二族 1039 没按 O 臂推过串扣、只有 8 个起点（本报告 1.3）。**该登**：第二轮判决第三节第 5 条要写回 D16 已定项 10 射程的口径句里，带上「第二族 155 个配置 8 个起点；1039 是原始分母」；重跑（第六节）若扣完不到 10 格，按跑前写死的读法（分母不到 10 不判）退回「没撑住」 |
| T2-3 | 卸载重挂出路：用户定「做成调用方显式发起的查询，排进里程碑五」 | 有去向（记录），kb 待写回 | 记录第 1051 行；去向是 C1003 那一行已写的「十一·4」。`checks-owed.md` 里还没有「做成单独的查询」这句（`grep -n '做成单独的查询\|显式发起'` 零命中，量的），随乙、甲写回补。C1003 的「开头走空」那半句要改的文档，行里写「第 645 行」，今天那段文档在 `mounted_session.rs:658`–`:660`（量的），**行号过期**，写回时一并改 |
| T2-4 | D16 已定项 10 射程：「不罩改动发成之后另发的搬迁批」（`.claude/kb/decisions/16-发布语义.md:276`） | 乙、甲之后仍成立 | 先算只在 `push_one_floor_raise_for_a_refusal_for_space`（`mounted_session.rs:6298`）里走，它的调用点是 `:3842`（`publish_the_change_raising_the_floor_for_space`）、`:6422`（走空）、`:7262`（`accept`）；搬迁批在 `:2490` 直接调 `push_one_floor_raise_within_the_admission_budget`，不经它（量的，`grep -n`） |
| T2-5 | 同一句里「也不判提交内生块那一判（照推）」 | 仍成立 | 提交内生块那一判交 `None` 算放得下：「JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,」（`crates/singlefs-core/src/mounted_session.rs:6097`）；乙的上界只对槽对与式子两判算 |
| T2-6 | 射程没写的两处推：**挂载续做**（`crates/singlefs-core/src/mount.rs:8463`）与**挂载准入**（`mount.rs:9007`）也直接调 `push_one_floor_raise_within_the_admission_budget`，不经先算；另外落点类被拒（`PublishError::PlacementRefused`）先算「交空的、照旧推」（`mounted_session.rs:6117`，量的） | **没去向** | 行为上无害（推的）：挂载续做是删除类发布，式子那一判不判，先算对它们本来判不出；挂载准入的需求是空（`no_demand_on_any_device_of`）。但射程只点了三处与搬迁批，读的人会漏这两处与落点类。**该登**：D16 已定项 10 射程那句「先算罩 `accept`、`publish_user_change` 与走空三处」后补一句「挂载续做与挂载准入的推不经先算；落点类被拒照推」，同一批写回做 |
| T2-7 | 三处文档与射程漏「被拒的是合取 7 判的前台写入类」这个前提（2.1 表） | **没去向** | 只有结构体文档（:689）写了。**该登**：并进 C1003「射程与文档补…」那一条的同一次改文档，或写回员在射程里补一句：删除、截断、建快照不先算、照推（依据 `mounted_session.rs:6158` 的 `of_the_user_change`） |
| T2-8 | 先算在**只读挂载**里 | 不适用，条款没写 | 先算在会话里；会话在可写挂载之后才有（`mounted_session.rs:8` 起的文件头），只读挂载没有发布入口，先算无从判（推的）。无需条款；射程可不提 |
| T2-9 | 先算在**冻结**之后 | 有行为、有条款、乙那一侧没有单独用例 | 冻结的发布被拒是 `PublishFrozenAfterAWriteFailureIsNotResentYet`，在「不是空间不够」那一组里（`mounted_session.rs:2141`），甲不判（用例 `a_refusal_not_short_of_space_is_not_judged_against_the_empty_cluster_segments_floor`，`:8808`），乙的盘上界只对 `SpaceAdmissionRefused` 算（`:6117` 那句）；`accept` 冻结后在任何读写之前拒是已定项 10 定案原文（`16-发布语义.md:274`）。乙对非空间类拒绝没有专门用例（推的，没全搜） |
| T2-10 | 先算在**准入关着**（只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`，`admission.rs:1149`） | 有用例 | 甲：`is_admission_judged`（`mounted_session.rs:6195`），用例 `with_the_space_admission_switched_off_the_empty_cluster_segments_floor_is_not_judged_before_pushing`（`:8830`）；准入关着时没有 `SpaceAdmissionRefused`，乙的盘上界为空。产品路径没有这个开关，条款不必写 |
| T2-11 | 甲的 `N₀` 取自整理开关：整理**关着、停着**或**换了开关**时甲判不判 | 判，且与合取 7 同步，无独立问题 | `N₀` 是 `EmptyClusterSegmentsFloor`（`crates/singlefs-core/src/compaction_plan.rs:490` 起的枚举，第一版取值在 `:498`、是 1，只有测试开关 `ForcedForTests` 能改），不是「整理开不开」的开关；先算与发布路径的合取 7 读同一个量（2.0），整理停着不改它、换成测试档两处一起变。整理停着时空间拒绝报不带「后台停着」：记录第 1052 行（第 134 批事实表）量到，用户第 1053 行定「写进报警与观测、拒绝报不加字段」，去向 D26 已定项 13 第 4 条与 C1024（第 1054 行）。有去向 |
| T2-12 | 第一轮判决那一行「每次发布克隆分配器多一张每槽一字节的位图」判不违规、接着写「登欠账量代价」（`research/prompts/m4-gpugc-precheck-b-a-code-r1-main-verification.md:34`） | **没去向** | `checks-owed.md` 里找不到这一行（`grep -nE '克隆'` 再接 `grep -E '分配器|空闲图|位图'` 零命中；`grep -E '64 MiB|每槽一字节'` 只命中无关的 C310；量的）。今天的树上同一条路又多了 `segments_with_an_unblocked_slot_pair`（`allocator.rs:485`，按段号的 `BTreeSet`）。**该登**：`checks-owed.md` 新一行，「怎么拦」写在 1 TiB、每槽 16 KiB 盘上量一次发布克隆分配器的挂钟与字节，去向里程碑五 |

## 四、检查单八格

| 格 | 判 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 改动全文的加号行共 120 行（`grep -cE '^\+[^+]'`，量的）。含 `assert`、`expect(`、`unreachable!`、`panic!`、`unwrap(` 的加号行 4 行：改动全文第 165 行（重建用例里的 `assert_eq!`）与第 381、389、395 行（走空用例的三句 `assert!`），全在测试里。含 ` - ` 的加号行 1 行：第 95 行，是 `allocator.rs:875`–`:881` 里原有的减法换了缩进、套进 `SlotPairCount(…)`；被减数是总数、减数是 `cluster_segments` 各段之和，各段互不相同，减数不会大于总数（推的；增量维护的数与按扫描值数出来的数对拍的助手在 `allocator.rs:3915`）。没有新的 `unreachable!`、没有新的未定条款的错误成员 |
| 类型与名字 | **不成立**（核心包成立，验证包里不成立） | 核心包里读 `unblocked_slot_pairs_outside_the_cluster_segments` 与 `slot_pairs_for_data_units` 的地方全是 `SlotPairCount`（`allocator.rs:870`、`admission.rs:980`、`:1077`、`mounted_session.rs:6060`）。验证包的 E180、E181 与测试在读的地方加 `.0` 拆回 `u64`，下面这些以裸整数表示槽对数（量的，`grep -n`）：`crates/singlefs-verification/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs` 的 `data_unit_pairs_outside_the_clusters_of -> u64`（:1354）、`data_unit_pairs_outside_the_clusters -> u64`（:1360）、`pairs_outside_the_clusters: u64`（:1367）、`aligned_free_pairs: Vec<u64>`（:2217）、`needed_pairs: u64`（:2221）、`remaining_pairs_outside_the_clusters: Vec<u64>`（:2522）、`-> Vec<u64>`（:2567）；`e180_admission_reading_versus_writable_bytes.rs` 的 `half_of_the_pairs_outside_the_cluster_segments -> u64`（:2692）；`accept_raises_the_floor_for_space.rs` 的 `slot_pairs_left`（:1032）、`bytes_of_slot_pairs(pairs: u64)`（:1082）、`-> u64` 的 `slot_pairs_counting_one_pair_for_every_slot…`（:1091）、`slot_pairs_left: u64`（:1322）。新用例名 `slot_pairs_once_everything_is_released_are_the_same_after_rebuilding_the_allocator_from_its_records` 没有缩写 |
| 不兼容旧代码与旧格式 | 成立 | 加号行里 `legacy`、`旧版`、`兼容`、`migrat`、`迁移`、`Version` 零命中（量的）；改动全文没碰盘上字节与版本号 |
| 不新建判定库 | 成立 | 加号行里 `RocksDB`、`sqlite`、`.tsv`、`verdict`、`判定库`、`prune` 零命中（量的）；变异表不是判定库 |
| 所有任务多机可用 | 成立（改动）；模型也守 | 加号行里 `/home`、`/tmp`、`hostname`、`env::var`、`std::fs`、`File::`、`TMPDIR` 零命中（量的）；新用例在内存里造 `DeviceFreeMap::new`（`allocator.rs:4235`）。我的模型里路径只从参数与 `$TMPDIR` 取 |
| 测试优先多线程 | 成立 | 重建用例是普通 `#[test]`，没有串行属性（加号行里 `thread`、`serial`、`Mutex` 零命中）；每一步读上一步的分配器状态，一段 600 步历史本身排不成并行，理由写在 `allocator.rs:4226` |
| 带 --item 的命令与项 | 成立 | 删掉的 4 个旧名（`…/m4-gpugc-precheck-b-a-r2-fixes-writer/patch/mutations-delete.txt`）在 `crates/mutations.tsv` 里按第一列逐字比对 0 行；追加的 5 个新名各恰好 1 行（量的，`awk -F'\t'`）。注意第一个旧名是第一个新名去掉「（槽对数类型）」，用 `grep -F` 子串查会误命中 1 行，要按整格比。15050：抓到 5 条、锚点失效 0 行，现判 1 条、复用 4 条（1.1） |
| 新崩溃流多罩了什么 | 不要新崩溃流（推的） | 先算「只读分配器（内存），一个盘都不读、一个写都不发」（`mounted_session.rs:6227`），判出就在写之前拒，没有崩溃点可加。推过串或走空发成过版之后才判出的形，已落盘的写走已登记的崩溃流（13913 点名的 `floor-raise-pushed-by-the-session` 等，`.claude/gate.d/crash-point-replay-items.tsv:18`）。没量过：缩域那一趟三项都 `verdict=red`，不缩域的留给提交时 |

## 五、什么现象会推翻这些结论

| 结论 | 什么会推翻它 |
|---|---|
| 1082 对、1039 只是原始分母 | 第六节第 1 条重跑的第二族 `oracle.out` 里 `fired by=slot-pairs oracle-series=0` 为 0（那就是 1039 都推过串，两个数口径才同） |
| 421 全「不变」、0 推高 | 重跑出现 `series arm=oracle short-before empty-segments=raised` 或 `series arm=forced short-before empty-segments=raised` 大于 0 |
| 今天的树上两族读数与第二轮同（我标的「推的」） | `compare_probe_result` 打出 `differing` 大于 0，尤其 `misrefused_*`、`fired_by_*`、`remount_published_*`、`series_*_short_before_raised` 里任何一个字段不同 |
| 乙的上界不小于落得下放行的数（2.0） | 造出一步让 `unblocked_slot_pairs_outside_the_cluster_segments` 大于 `slot_pairs_without_an_unreleased_record_outside_the_cluster_segments` 的历史（只可能是罩位置位而 `Allocated` 位未置，即 `mark_reclaimed` 回收了一条未释放记录的槽） |
| 重建用例钉的是重建走的那条路 | `mount.rs:3248`、`:3762` 之外出现另一处从记录重建分配器的入口，或 `rebuild_from_records` 里在 `mark_allocated`/`mark_released` 之外多了一步改罩位或全空段数 |
| 甲的 `empty_segments()` 重建之后与重建之前相同 | 新写一条与 `allocator.rs:4228` 同形、比 `empty_segments()` 的用例，红了 |
| 没去向的 T2-2、T2-6、T2-7、T2-12 | `checks-owed.md`、D16 已定项 10 射程或 05 里程碑文件里找到对应的行或句，我的 `grep` 漏了（命令都写在表里） |
| 15051 的树与冻结副本 `allocator.rs` 不同 | 第六节第 2 条在冻结副本同一份 `allocator.rs` 上全绿，且主 agent 查到 15051 拷快照那一刻 `allocator.rs` 已是 `eb822f09…` |

## 六、要主 agent 跑的命令

两条，各自单起；主 agent 经 `task-server/task submit` 交，我没交、没起 cargo。

1. 两份旧探针在今天的树（冻结副本）上重跑，并逐字段与第二轮读数比；第二族加了 `oracle-series` 计数。
   - 命令（工作目录仓根）：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3-sonnet-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r3-sonnet-model`
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；日志里 `name=exit_code family=random-history oracle=0` 与 `name=exit_code family=fill-then-fragment oracle=0`；两份 `name=self_checks bad=0`（在运行目录 `out/<族>/oracle.out` 末行）；`name=compare_probe_result fields_old=… differing=<N>` 两行，N 为 0 算今天的树复现第二轮全部字段，非 0 的逐行是 `name=compare_probe_result_differs field=…`；第二族 `oracle.out` 里 `grep 'fired by=slot-pairs oracle-series=' ` 的 0、1、2、3 档拆出 1039 的有效分母。
   - 内存与时间（量的，第二轮同样两份探针的任务）：第一族 14740 第 4 次尝试内存峰值 8.99 GiB、用时 1009 秒，前三次撞 4、6、9 GiB 上限；第二族 14746 内存峰值 8.21 GiB、用时 820 秒。这一条两族依次跑、各编一次，估时 35–45 分钟（推的）；探针自带估时上限我放到 100 分钟（两份 `main.rs`）。探针每个种子往标准错误打一行 `PROGRESS`，脚本每族起编、起跑各打一行。
   - 编译没验过：我没编改过的探针（第一族补丁加了 `.0`、第二族加了一行计数）。编不过时退 1、日志末 80 行在 `out/<族>/build.log`，由主 agent 派同立场腿接着改。
   - 跑完删：运行目录 `…-r3-sonnet-run` 与服务给的编译目录（`task clean`）；脚本自己把本地临时拷贝删了。
2. 新重建用例与同族的库单测，在今天树上的 `allocator.rs`（`eb822f09…`）上跑一遍（15051 在它之前的树上跑的，1.1）：
   - 命令（工作目录仓根）：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的测试目标 --lib 里 allocator::tests 的槽对四族名字; 范围=1 个测试目标; 为什么=合入后验证 15051 拷的是另一个会话最后一份补丁之前的树，allocator.rs 此刻是 eb822f09，新的重建用例与按段取落点对拍的几条在这一版上没有绿的记录' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-core --lib -- allocator::tests::slot_pairs_once_everything_is_released allocator::tests::user_data_slot_found_by_segment allocator::tests::placement_by_segment_against_stepping allocator::tests::segments_with_an_unblocked_slot_pair`
   - 看哪几行：`TASK_SERVER_RESULT` `type=succeeded`、`exit=0`；日志里 `test result: ok.` 那一行的 `passed` 数不小于 11，`0 failed`；其中要有 `slot_pairs_once_everything_is_released_are_the_same_after_rebuilding_the_allocator_from_its_records … ok`。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T3 正文第二节各数 | 一致（带三处补充） | 14814、14846、15050、15051 的计数行都对；15050 的 5 条里 4 条是复用；13913 红数 0 是放量那一格，任务整体三项 `verdict=red`；15051 的树早于最后一份补丁 |
| T3 判决第一节抄的整行 | 一致 | 逐行对上两份 `oracle.out`；第二族整行与判决第 25 行逐字节相同 |
| T3 有效分母 1082 | 一致 | 1907 − 825 = 1082，算法对 |
| T3 有效分母 1039 | **冲突** | 见下面并排；1039 是原始分母，第二族产物没有 O 臂推过串的拆分，且 155 个配置只有 8 个起点 |
| T3 0/421 | 一致 | 59 + 362 = 421，推高 0 + 0；产物更强（全「不变」）；但 362 串出自 21 个配置 |
| T3 62 格 | 一致 | 62 + 1845 = 1907；取数与判的时刻同一个数 |
| T3 两批判别子探针整行 | 一致 | 记录第 1043、1047、1050 行逐行对上四份产物；正文行号漏了 1043 |
| T3 搬到今天的树 | 没有一个是今天树上量的 | 乙甲误拒、0/421、62、41544 读码推今天同样成立；老上界对照、重挂 12+2、187+32、丁的毫秒数永远标旧树上量的 |
| T1 乙与落得下同一把尺 | 一致（推的） | 乙的集合包含落得下的；按段取落点只加了段号集合 |
| T1 甲与合取 7 同一个数 | 一致 | 同一个 `empty_segments`、同一个 `N₀`、同一个函数 |
| T1 第 1 条四处文档 | 一致，**漏前提** | 三处没写「被拒的是合取 7 判的前台写入」（T2-7） |
| T1 第 2 条 `SlotPairCount` | 一致 | `<` 两边都是槽对数，换成槽数那一处乘数没变 |
| T1 第 3 条三句断言 | 一致 | 与池一致；新三句没被单独证红 |
| T1 第 4 条重建用例 | 一致（同一条路），**两处补充** | 走 `rebuild_from_records`；比的三样与探针不同，甲读的 `empty_segments()` 重建后没有用例比 |
| T2 十二形 | 有去向 6（T2-1、2、3、4、5、11）、不适用或有用例 3（T2-8、9、10）、没去向 3 加一（T2-6、7、12；T2-2 要补口径） | 该登哪里见第三节 |
| 检查单 走不到的理由 | 成立 | 加号行里断言全在测试，唯一减法是原有的 |
| 检查单 类型与名字 | 不成立 | 核心包成立；E180、E181 与测试里 11 处槽对数以裸 `u64` |
| 检查单 不兼容旧代码与旧格式 | 成立 | 加号行零命中 |
| 检查单 不新建判定库 | 成立 | 加号行零命中 |
| 检查单 所有任务多机可用 | 成立 | 加号行零命中本机路径与环境 |
| 检查单 测试优先多线程 | 成立 | 普通 `#[test]`，不并行的理由写在 `allocator.rs:4226` |
| 检查单 带 --item 的命令与项 | 成立 | 4 个旧名 0 残留、5 个新名各 1 行、抓到 5 条（4 条复用） |
| 检查单 新崩溃流多罩了什么 | 不要新崩溃流（推的） | 先算一个写都不发，推出去的串走已登记的崩溃流 |

「有效分母 1039」冲突，两边整行并排：

| 判决那一边 | 产物那一边 |
|---|---|
| 「有效分母是 1082 与 1039，不是 19351」（`research/prompts/m4-gpugc-precheck-b-a-code-r2-main-verification.md:34`）；同一行写「第二份互不相同的一族 1039 格、写进 0（量的）」 | 第一族第 95、96 行：`oracle COUNT fired by=slot-pairs oracle-series=0 825`、`oracle COUNT fired by=slot-pairs oracle-series=1 1082`；第二族第 330 行字段 `fired_by_slot_pairs=1039 … misrefused_slot_pairs=0`，第二族整份 `grep -c 'oracle-series'` 得 0（量的） |

## 八、没做什么

- 不判别：T1、T2 的攻击面（找今天的树上乙、甲站不住的形）归攻方腿；我只写读码推出的「应该是什么」与逐格比对，不替主 agent 采纳或出判决。
- 没往统一任务服务交任务、没起 cargo；改过的两份探针没编没跑；判决要引的数（尤其今天树上的读数、第二族 1039 的有效分母）都没复跑，命令在第六节。
- 库单测（包括新重建用例与按段取落点对拍的一族）没跑；2.0 的包含关系、甲与合取 7 的差别、T2-6 的两处推「行为上无害」都是读码推的。
- 没读禁读清单里的文件与目录；没读 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3-opus/` 下任何东西。
- 没查：第二族探针 8 个起点各自贡献多少格（产物没拆）；15051 拷快照那一刻 `allocator.rs` 的实际哈希（日志没写，运行目录已删）；`accept` 在冻结之后乙那一侧有没有单独用例（只搜了 `mounted_session.rs` 与 `accept_raises_the_floor_for_space.rs` 的用例名）。
- 草稿与脚本在 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3-sonnet/`（`anchor_count.py` 锚点计数、`alloc-r2-r3.diff` 第二轮与今天 `allocator.rs` 的差、几份任务日志副本），没放进仓；原因：只是我核数时的中间件，结论与行号都已写进本报告。
