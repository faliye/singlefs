# tooling-broad-mutation-prune-r2 攻方腿（Opus）报告：V2、V3

日期 2026-10-01。被判对象：`research/prompts/_tooling-broad-mutation-prune-r2-body.md` 第三节第二版形态（P1–P5）。前几轮判决：`research/prompts/tooling-broad-mutation-prune-r1-main-verification.md`；第一轮攻过的 S1、S2、S4a、S4b、常量初始化式、死循环、第 556 条整段超时这里不再攻。
真 crates 上的数都是我副本上的（HEAD 导出，218 个文件逐个 sha256 与库目录这一版 manifest 相同），不是入库装置上的数；模型上的改法与判定只在我的模型上量过、被攻过零轮。库目录只读。

## 各格判定一览

| 格 | 形状 | 判定 | 在哪量的 | 节 |
|---|---|---|---|---|
| V2①-读法 | 「F2」没定义；被改文件算不算「T 执行到的源文件」正文没说 | 两种读法：甲（字面，被改文件照样按文件比）、乙（意图，只比被改函数与函数外部分） | 读正文 | 2.0 |
| V2①-同文件另一个函数 | 兄弟函数不再调被改函数（FA）；同名固有方法加进同一文件已有的 `impl` 块（FA2）；嵌套 fn 的外层（FF） | 打中读法乙：判「复用抓到」，重跑没红；读法甲不中 | 模型（量过） | 2.2 |
| V2①-文件外、无覆盖区域的项 | `pub use` 再导出换指向（FB）、类型别名（FC）、枚举判别值（FD） | 打中，甲、乙、丙三种读法都判「复用抓到」，重跑没红；今天 crates 里同形的零星 | 模型（量过）；crates 用 grep 数 | 2.3 |
| V2③-变异体走的路 | 变异把 T 拐进基线没走过的文件，第二版改那个文件（FH 抓到翻没红、FH2 没红翻抓到） | 打中，两版基线覆盖的并集接不住；真 crates 上按文件的前提 236 条里 0 条、按函数的前提 32 条 | 模型（量过）；crates 副本 236 条变异逐条插桩 | 3.1、3.2 |
| V2③-覆盖从哪里取、存多大 | 库目录只存 sha256，事后量不回旧版覆盖 | 只能判的当时存；留多久正文没写；一版名字表约 94 MB（推的） | 读库目录；副本量每条的大小 | 3.3 |
| V2②-限时的方向 | 超时 = 抓到 = 不红，又可复用 | 打中前提：负载或变异让会过的测试超限就记「抓到」并复用（误绿）；端到端的翻面没造 | 库目录产物 | 4.1 |
| V2②-倍数与基线的散 | 长测试 PASS 的耗时 / 基线：最大 30.59，最小 0.17 | 倍数 5、下限 60 秒时 1 446 次 PASS 里 216 次超限；一次都不超要倍数 ≥ 31 | 库目录产物 | 4.2 |
| V2②-墙钟与 CPU | 同一条测试挤在两个核上 | 墙钟 4.29 → 18.73 秒，CPU 3.09–5.56 秒不随挤涨 | 副本（量过） | 4.3 |
| V2④-常量闭包 | `use` 改名、通配、关联常量、泛型常量参数 | 没打中：按名字过近似收得到（推的）；漏的是常量之外的项（归 2.3） | 模型 | 五 |
| V3-函数边界 | 清单 5 091 条逐条切 | 4 918 条切对，170 条在常量初始化式（B2），嵌套 0 条 | 副本 | 6.1 |
| V3-被改那一份 | 最近 15 个提交 | 中位：按文件 3 073 条、按函数 1 308 条；函数外部分 15 个提交里 10 个变了 | 副本 | 6.2 |
| V3-连同关联 | 样本 235 条 × 15 个提交 | 读法甲与按文件逐个相等；读法乙只在 2 个提交上少 1 条；242 条「抓到」里 164 条 T 与变异同文件 | 副本（样本偏在 `allocator.rs`） | 6.3 |
| V3-覆盖遍 | 插桩倍数、每条导出 | 一条一进程 1.79 倍（harness）、多线程一进程最多 22.41 倍；一版第一段约 10 分钟（推的） | 副本（负载 23–29） | 6.4 |
| V3-改 1、5、20 个函数 | 照样本外推到全表 | 改 5 个函数：读法甲、乙一百多小时（推的），按第六节第三条交用户 | 推的 | 6.5 |

## 一、复跑命令与文件 sha256

模型（`research/prompts/tooling-broad-mutation-prune-r2-opus-model/`，说明在它的 `README.md`）：

```
bash research/scripts/run-with-memory-cap.sh 6G bash research/scripts/capped.sh 12 nice -n 19 python3 -B research/prompts/tooling-broad-mutation-prune-r2-opus-model/run.py <工作目录> <llvm-tools 1.98.0 的 bin 目录>
```

- llvm 工具：`curl -sSfO https://static.rust-lang.org/dist/2026-08-20/llvm-tools-1.98.0-x86_64-unknown-linux-gnu.tar.xz` 解到草稿目录（下载加解压 12 秒，236 MB），版本行「LLVM version 22.1.8-rust-1.98.0-stable」；没装 cargo-llvm-cov，没动 `~/.rustup`、`~/.cargo/bin`。
- 模型一趟 10–144 秒（机器负载不同）。
- `evidence/` 下的脚本在草稿目录里是不带前缀的名字（`v3-fnparse.py` 原名 `fnparse.py`，`v3-*.py` 互相按原名 import）；复跑时去掉前缀放进同一目录。各脚本的用法写在文件头，库目录与副本的路径按参数给。
- 真 crates 副本：`git archive HEAD crates Cargo.toml Cargo.lock litmus .cargo | tar -x -C <副本>`；施加变异用第一轮攻方的 `research/prompts/tooling-broad-mutation-prune-r1-opus-model/apply.py`。

```
56a6cfb9d60ff3e29a4e482f72a5a2fa30f919ad5a589ddd4634dc0506d4fee2  evidence/crates-candidates.txt
45f11da574c24c9cb0234067911fcafcc3c4047eebffa483a228a91e411fd3f8  evidence/crates-mutant-path-all.txt
6f7a484a81b16c40b58ecc88e95350092a67e87e5d5e7b601111f9507a952302  evidence/crates-mutant_path_harness.py
8a382d459cb381c69a117e22f21e15952826d4f283aa2cbfb8bd3607de7a77a6  evidence/crates-mutant-path-harness.txt
41eadc81330b7791b653c60b8180acb1a6064fc695cb378cdd79cb26cf8116c7  evidence/crates-mutant_path.py
294f7620aca10b660183bea5197920b9b0bf86093346e92495f63d75bd84439c  evidence/crates-mutant-path-summary.txt
dee89f5b56f9ee84bbb4c4dbe3ea2ed8cfb606338e9176b8f075ac0be057c824  evidence/crates-summarize.py
7a02f805ff7b03b03367271123720204fa79c4d3f04dc0e91c0d9ed84c18dd1a  evidence/v2b-long_tests.py
5cffc962b2bc23f017a423ade646e20b418674444361095b5d5d300a3bbfa632  evidence/v2b-out-long.txt
06a377cb15cd6028e666fde6a2564b6a2349e76dc7727ab5168e2a0114c8865e  evidence/v2b-out-perlog.txt
57c2d40e6b1a3cb74edd7e2a00b8492271de877e6cb40994e8c31860cab3c53b  evidence/v2b-out-per-test-timeout.txt
646df9b27748db73e966334b5eb2304907da538cbed7a5c0c57b00550957a2e4  evidence/v2b-per_log_load.py
06619061a3a2e09a9711ab6807eb3184dc86a63d3dc1a72ce49dd468b5c9931c  evidence/v2b-per_test_timeout.py
7081c0747f3fe0a4ec442221babd73c63337fa4d3f460a4f460b30131cb6f668  evidence/v2b-wall_vs_cpu.py
4d955f1f4573b3474b1eaf9996c221c25937806465496ea5258512529ee18d0a  evidence/v2b-wall-vs-cpu.txt
2d8e502f4a0f87ea85e90fce860f953b7642f406493444e7e3793595156cd7a3  evidence/v3-executed-files-per-test.json
fa6d44437b15038d960f179271f992b56deede4b6e33ae9d385bbc5bba6efd2f  evidence/v3-fn_changes.py
113699f8405eaa34b62f212a59d7436169dd460cdddff0e9188a4179e61e144c  evidence/v3-fn-changes.txt
a9faffa238ed3860e4f6eb047ad21a22a732570010772dd8379bd3fe9e98748a  evidence/v3-fnparse.py
c239121dbfde602a15de57a4efc561f8d312c1480b383d565f82c11db2cf0520  evidence/v3-instr_cost.py
ae90677d0ec23bd7be1a26408abc70cd60f6e7c00429ae943af56ad7c1becdf2  evidence/v3-instr-cost.txt
74ba38db2c592afcd9a195dad1c99bb33e88e8ba1d55f724ee96151adda6d1fb  evidence/v3-outside_diff.py
be35316db9a8b154aad10675ec9eb859ba61b64754d90d511d9610cfdf5bcf0c  evidence/v3-parser_check.py
602936a52d1b0b9240665878a4acb20d0a72370f61ca358268b6c06f737deb49  evidence/v3-parser-check.txt
b6872d8f486e362ceeae5771e1677d302660f876f4b65bd52c54eb292f566646  evidence/v3-rerun_estimate.py
ada9d58927e9ebc102d78c900d51f6f5833f16f38ca50711ee4f8d3f136d9ef4  evidence/v3-rerun-estimate.txt
fa44471cdf983484a6adc77c80e323cd2f86e54ac3b7e9e8af34cdccbe031e90  README.md
318e7881770e2fb6e30e7c718889b6d5f471be57bcb505901b73fbe7ae6b3195  run-output.txt
2e2876694fd9f886a2d5567b11c1f3d79f483abca2271ad92e255d4ed2075529  run.py
e861b27dd4c8991e127ce8461d74ecb276717f1f19db932703c553419bae204a  scenarios.tsv
bf786f1fe33654a1d3ee3db8e493e465e648a73308debf351a37a51196c167d1  v2/C1-small.rs
f46dc15e183132d2178a73dbd586c146cac60afc1c50db4365665f82c8670f74  v2/C2-slots.rs
cf21e11e4ff40f53e0fe5284706d4c396293668add9872e226a302825b25eb71  v2/FA2-thing.rs
ce1e10e7d7b49b3bc664ea7ae65a4b8525bbfc6040c8df9ac66a3e876f953d3a  v2/FA-slots.rs
9e65e18b58c5b8ce1284c5ce7d3744919f6ea7616002da9a96b3f4d9921d5d51  v2/FB-codec.rs
d0e1393386cdf5a30af2ae0b96dcf88dd754cd43c15946a1dc612055356c463d  v2/FC-word.rs
e3e60e3c2941647e5c640cdfba899f4f42289178e82c692c41a48c5fd8f0aa19  v2/FD-kind.rs
bed1c543fa0a9f21512956a555f0f001ca55c68c4c2207dfb80d3f554e267747  v2/FF-nested.rs
71204efe966d1351a410bde4402b149139b2a9ef37b51597ddde48c8178a8010  v2/FH-big.rs
3420fce126e4647e38331217bb19bafe2203cdd762841fb94b0e1d38b10b2b7e  ws/core/Cargo.toml
bede3420525cd9f666a0e8c29eb701b318ad731ca52fdcb50f3d67e83e2faa95  ws/core/src/big.rs
a18f74eb251a670296ca60a87ccd2d7ea6e01069513cb04e75aba0267fccbe6b  ws/core/src/codec_lenient.rs
11de198d6bdfd56aa07061f7ccdb52a60eb7166fcc5cdd53e5258a7c069b6150  ws/core/src/codec.rs
c716f41a7c50a88de6137f027d56f9ead6b9001e6689bdec3bef54411abc8589  ws/core/src/codec_strict.rs
194f0edea040a1bf97d7237d5ceae776d5722e20143960c506ec0d41616aff41  ws/core/src/frame.rs
d92828cde203a383eb3633f4cb0177cd168d8f1008d3ee5008dce7888975c090  ws/core/src/kind.rs
c2d15af1545ed7c6d2a9087d8bda5c2ddafff76d62d3c5730511bd119c2a5384  ws/core/src/lib.rs
dc83cb728d9a7f608386ab4c53f4622a0238807600bb7c39f298344018ce8555  ws/core/src/nested.rs
79a571feedc1541f55b1c327dd2d0fdee22049141fa3c8dc72f40ee4a51201ba  ws/core/src/pack.rs
5a61c3c381102447d08b5b0861a421f105a2f5ddb902d4ea716851cdfa94c0f7  ws/core/src/route.rs
55d997a36721c5eb991b02a9dfe7401c914fdbfdde8c4d5c89119e644900bc0d  ws/core/src/slots.rs
a3f155f6dd733b38d0ffef3b72fedae2adc011e1f01c9ede4959b8425c6b0368  ws/core/src/small.rs
e3d541db4423818bbd5d30943c020e7fdf6fbd81587544326b353938242d54b6  ws/core/src/tag.rs
3a1383c09bb0abd835158f4f0601ba30bb7541f851c3ac01b02bd7b6e953da35  ws/core/src/thing.rs
05253f498dca1b600a08e5f8ab305e2562b61affb51aa2134bdaad7ae4a9ecda  ws/core/src/word.rs
cb8dc90e72def6b68c845a6027807a9419b740f7f16543fba82c65c5b8db2a47  ws/harness/Cargo.toml
bf298cd78ba0910e42f4e497767e6c724fc39736dcf30f49d60e55de4f638a21  ws/harness/tests/describe_slot.rs
d9cb1a780b841daee238c2c7389a45a07a06bf67d327e1f76439574c8f9668a4  ws/harness/tests/frame_roundtrip.rs
5ba4c4c4569c9472c6760a03c7166c0547ffca0fa63db8ac1ebbff6d1b37b348  ws/harness/tests/nested_outer.rs
09a1b50408ccf2990df7f7d0220b4f1e8be59bdff392e26eb4644f805575dc04  ws/harness/tests/pack_word.rs
a0320c7a5a77f94d87897c77121432bb53d267573e115a3846d1a49955f280e9  ws/harness/tests/route_small.rs
470bf6e539d34f057ac370234d1dac5185a01c4aba0e3d35589981d21753518e  ws/harness/tests/tag_kind.rs
c5add118532762e0c529005a7bf9ade451e27ae82b0a5f23595cf82a323b8473  ws/harness/tests/thing_describe.rs
```

## 二、V2①：按函数之后，函数外的改动翻了函数内的结局

判法照正文第三节：P1 键里「被改函数的源码 sha256」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:30`「被改函数的源码 sha256：函数边界由工具解析（例如 `syn` 取 `fn` 项与它所在 `impl` 块的头），解析不出来的退回整个文件的 sha256。」），
外加「被改函数所在文件里函数外面的部分（`use`、类型定义、`impl` 块头、属性），按 F2 并进关联。」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:33`）；P2 抓到的那一条是「抓到的（被 T 抓到）：T 所在测试文件、T 执行到的源文件、常量闭包（P3）、宏与 derive 的定义文件都没变，才复用。」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:36`）。

### 2.0 正文自己有两种读法，它们给出的东西不一样

- 第 33 行的「F2」正文里没有定义（`grep -n 'F2' research/prompts/_tooling-broad-mutation-prune-r2-body.md` 只命中第 3 行的 not-numbers 表和第 33 行本身）。
- 被改文件自己算不算「T 执行到的源文件」，正文没说：
  - **读法甲（字面）**：算。T 抓到变异就一定执行了被改函数，被改文件一定在 T 的执行集合里；P2 按文件比，文件里任何一处变了就重判。按函数的那把键这时一条也省不下（第六节 6.2 数）。
  - **读法乙（意图）**：不算，被改文件只比「被改函数 + 函数外面的部分」。第 33 行特意把函数外面的部分列出来，读的是这一种。乙内、乙外是嵌套 fn 取最内层还是最外层（正文第 30 行「`fn` 项」没定）。
- 模型（`research/prompts/tooling-broad-mutation-prune-r2-opus-model/`）对每个场景三种读法都判，再在第二版上真跑。P3、B8 在模型里按名字过近似，比任何只凭语法解析名字的工具收得多（`run.py` 文件头）。

### 2.1 对照：判法会说「重判」（C1、C2）

- C1 改 T 执行到的另一个文件：`research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:5`「SCENARIO C1 reading=甲 mutated-function-and-outside-unchanged=True changed-associated=core/src/small.rs => rerun」，乙内、乙外同样 rerun（第 6、7 行）。
- C2 改被改函数自己：`research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:16`「SCENARIO C2 reading=乙内 mutated-function-and-outside-unchanged=False changed-associated=none => rerun」。

### 2.2 同一文件里的另一个函数变了（FA、FA2、FF：读法乙打中，读法甲不中）

- **FA（兄弟函数不再调被改函数）**：`slots.rs` 里 `slot_of` 是被改函数（`/` 换 `%`），同文件的 `describe` 调它；第二版只把 `describe` 改成自己算 `offset / 4096`，输出不变。
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:26`「SCENARIO FA reading=乙内 mutated-function-and-outside-unchanged=True changed-associated=none => reuse caught」（乙外第 27 行同）。
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:30`「SCENARIO FA v2-baseline=green(7) v2+M failing=none => rerun-verdict=missed」。
  - 许可它的每一步：被改函数一字没动（第 30 行那把键不变）；`describe` 是另一个 `fn` 项，不是第 33 行列的「`use`、类型定义、`impl` 块头、属性」；T 的测试文件没动；T 执行到的别的文件没动。
- **FA2（同名固有方法加进同一文件已有的 `impl` 块）**：被改的是 `impl Describe for Thing` 里的 `describe`；第二版在同一文件已有的 `impl Thing { … }` 里加一个同名固有方法，`impl` 块头一字不变。方法解析先选固有方法，T 不再走 trait 实现。
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:36`「SCENARIO FA2 reading=乙内 mutated-function-and-outside-unchanged=True changed-associated=none => reuse caught」；第 40 行「SCENARIO FA2 v2-baseline=green(7) v2+M failing=none => rerun-verdict=missed」。
  - 这是第一轮 S4a 搬进被改文件自己：第一轮靠「另一个文件进了第二版的覆盖」由并集接住；搬进被改文件之后，读法乙恰好把被改文件从按文件比里拿掉了。
- **FF（嵌套 fn）**：被改的是 `outer` 里嵌着的 `inner`；第二版只改 `outer` 的函数体，不再调 `inner`。
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:76`「SCENARIO FF reading=乙内 mutated-function-and-outside-unchanged=True changed-associated=none => reuse caught」；第 77 行乙外 rerun；第 80 行重跑 missed。
  - 今天的清单里碰不到：5 091 条里没有一条落在嵌套 fn 里（第六节 6.1 的 parser 核对，`nested` 计数为 0）。
- 读法甲在三格上都是 rerun（第 25、35、75 行），代价见第六节：甲下按函数省 0 条。

### 2.3 文件外、而且没有覆盖区域的项（FB、FC、FD：所有读法都打中）

- **FB（再导出换指向）**：`codec.rs` 只有一行 `pub use crate::codec_strict::check;`，第二版换成 `codec_lenient::check`。被改函数在 `frame.rs`，两版都没动。
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:45`「SCENARIO FB reading=甲 mutated-function-and-outside-unchanged=True changed-associated=none => reuse caught」；第 50 行「SCENARIO FB v2-baseline=green(7) v2+M failing=none => rerun-verdict=missed」。
  - `codec.rs` 没有可执行的代码，两版的覆盖都不列它；它不是常量、不是宏、没有 derive，P3、B8 不收；它也不是被改文件，第 33 行管不到。两版覆盖的并集里有 `codec_lenient.rs`，但那份文件没变。
- **FC（类型别名）**：`word.rs` 只有 `pub type Word = u16;`，第二版改成 `u8`；被改函数 `pack` 返回 `Word`，`<<` 换 `>>` 在 `u8` 下截断后看不出来。第 55 行甲「reuse caught」，第 60 行重跑 missed。
- **FD（枚举判别值）**：`kind.rs` 只有一个带判别值、不带 derive 的枚举，第二版把 `Data = 1` 改成 `Data = 2`；被改函数 `tag` 里 `*` 换 `+`，`2 * 2` 与 `2 + 2` 相等。第 65 行甲「reuse caught」，第 70 行重跑 missed。
- 三格在乙内、乙外下同样 reuse（第 46–47、56–57、66–67 行）。
- 今天的 crates 里碰得到的有多少（2026-10-01 工作树）：类型别名 3 处，都与用它的代码在同一个文件（`grep -rn '^\s*pub type \|^\s*type .* = '` 在 core、checker、format 的 src）；再导出 1 处（`crates/singlefs-core/src/mount.rs:85`「pub use crate::instance_table::{InstanceRow, InstanceTableRecords};」，`mount.rs` 本身有代码、会进覆盖）；带显式判别值的枚举 0 处；`size_of::<` 0 处。今天碰到的少，第二版是按通用规则写的，规则本身漏这一类。

## 三、V2③：两版覆盖的并集——变异体自己走的路不在任何一版的覆盖里

正文：「覆盖取「上一次判它那一版」与这一版的并集（B4）。」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:35`）。两版的覆盖都是在没改过的树上量的。

### 3.1 变异把 T 拐进基线没走过的文件（FH、FH2：所有读法都打中，模型量过）

- **FH（抓到翻成没红）**：`route` 里 `value > 100` 换成 `value < 100`。没改过的树上 T（`route(5)`）只走 `small.rs`；变异体走 `big.rs`。没有一条测试在没改过的树上走 `big.rs`，所以第二版改 `big.rs` 不动基线。
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:82`「SCENARIO FH T=harness/tests/route_small.rs::small_values_take_the_small_path T-on-v1+M-executed=core/src/big.rs,core/src/route.rs,harness/tests/route_small.rs not-in-v1-or-v2-baseline-coverage-of-T=core/src/big.rs」
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:85`「SCENARIO FH reading=甲 mutated-function-and-outside-unchanged=True changed-associated=none => reuse caught」；第 90 行「SCENARIO FH v2-baseline=green(7) v2+M failing=none => rerun-verdict=missed」。
- **FH2（没红翻成抓到）**：同一条变异，第一版 `big.rs` 与 `small.rs` 结果相同（变异没红），第二版改 `big.rs` 让它不同。
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/run-output.txt:94`「SCENARIO FH2 reading=甲 mutated-function-and-outside-unchanged=True changed-associated=none => reuse missed」；第 99 行「SCENARIO FH2 v2-baseline=green(7) v2+M failing=['harness/tests/route_small.rs::small_values_take_the_small_path'] => rerun-verdict=caught」。
  - 没红的复用照 P2「没红的：覆盖到被改函数的每条测试及其执行到的文件都没变，并集里没有新的测试覆盖到它，才复用。」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:37`）；按 P5 这条会一直红，而且按 P5「等价变异写测试留档后从清单排除」会被当成等价变异排掉——第二版上它其实是被抓到的那一种。
- 并集修不了这一格：两版的并集都是「没改过的树」上的覆盖。第一轮 S4a、S4b 是「两版没改过的树走的路不同」，这里是「变异体走的路与没改过的树不同」，是另一件事。
- 只凭模型的判法，读法乙内、乙外同样 reuse（第 86–87、95–96 行）。

### 3.2 真 crates 上这一格的前提今天有多常见（副本量过，一次取样）

- 取法：这一版判成「抓到」的 242 条里，T 是 core 库测试的 186 条全取；T 是 harness 测试的 55 条里去掉 T 单条超过 30 秒的 5 条，取 50 条（另 1 条的 T 是 checker 的库测试，没取）。每条在副本上插桩，T 在没改过的树上与在变异体上各跑一次，比执行到的文件与函数（脚本 `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/crates-mutant_path.py` 与同目录 `crates-mutant_path_harness.py`）。
- 结果（汇总行是 `evidence/crates-summarize.py` 的原样输出）：
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/crates-mutant-path-summary.txt:2`「rows=186 baseline-pass=186 mutant-fails-T=186 mutant-executes-a-file-outside-T-baseline-files=0 mutant-executes-a-function-outside-T-baseline-functions=11」
  - `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/crates-mutant-path-summary.txt:3`「rows=50 baseline-pass=50 mutant-fails-T=50 mutant-executes-a-file-outside-T-baseline-files=0 mutant-executes-a-function-outside-T-baseline-functions=21」
  - core 库测试 186 条：变异体走进 T 基线没走过的**文件** 0 条，走进基线没走过的**函数** 11 条（分支类变异 59 条里 7 条）。
  - harness 测试 50 条：走进新文件 0 条，走进新函数 21 条（分支类 23 条里 10 条）；第 224 条变异体多走了 11 个文件里的 235 个函数，而那 11 个文件 T 在基线上本来就走到了。
- 读法：关联按文件比的话，今天这 236 条里没有一条满足 FH 的前提；变异体大多在原来走过的文件里早早失败。一旦关联细到函数（让按函数真省下东西的那条路，见 6.3），前提就在 32/236 条上成立。
- 「没红」的那一种（FH2）在真 crates 上没量：这一版第一段没红的 15 条日志没留（库目录 `logs/` 里没有它们的 one-test 日志）。

### 3.3 「上一次判它那一版」的覆盖从哪里取、存多大

- 从哪里取：只能在判它的当时量、当时存。库目录这一版只存每个文件的 sha256（`manifest.tsv` 218 行，没有文件内容），版本按整棵树认、可以是没提交过的工作树，事后造不回那棵树，也就量不回它的覆盖。
- 留多久：一条结论被复用多少版，它「上一次判它那一版」的覆盖就要留多少版；被改文件从来不动、T 也不动的那批变异，会一直指着最早那一版。库目录需要一条「还有谁引用这一版的覆盖」的回收规则，正文没写。
- 存多大（副本量的，`evidence/v3-instr-cost.txt`，量的时候机器负载 23–29）：每条测试一份「执行到的文件 + 函数名」表，中位数 core 库测试 1 578 字节，harness 8 个目标各自的中位在 17 410 到 230 462 字节；llvm-cov 的 export JSON 每条 250 万到 520 万字节；profraw 每条 9.9 万到 35.9 万字节。第一段 776 条测试按 harness 8 个目标中位数的中位（约 12 万字节）算，一版约 94 MB 的名字表（推的；换成函数编号的位图会小得多，也是推的）。
- 读法丙（攻方提的改法，被攻过零轮）要的是另一份覆盖：变异体自己那一趟的覆盖。它得在判每条变异的那一趟里插桩量，每条变异都存一份。

## 四、V2②：单条测试限时——把「慢」记成「抓到」，再被复用

正文：「单条测试限时 = 基线里那条测试的耗时乘倍数再加下限，超了算那条测试没过，即抓到（B1、用户定）。整段总时限去掉。」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:52`）。

### 4.1 方向翻了：第一轮是误红，第二版是误绿，而且会被复用

- 第一轮的病（第 556 条）：负载把「抓到」记成「超时」，按 P5 是红——误红，挡提交但不放过东西。
- 第二版：超时 = 抓到 = 不红。负载或变异让一条本来会过的测试跑过了限时，就记「抓到」——误绿，放过一条活着的变异。
- 这条「抓到」按 P2 可以复用：T 是超时的那条测试，T 的文件与 T 执行到的文件不变就一直复用。一个文件都没改，换个空闲时候重跑结论翻成「没红」。这就是「第二版判可复用、重跑结论翻面」，翻的方向是放过。

### 4.2 基线一条测试的耗时在负载下怎么散（库目录产物，一次取样）

只读库目录这一版的 nextest 日志（脚本 `evidence/v2b-per_test_timeout.py`、`evidence/v2b-long_tests.py`、`evidence/v2b-per_log_load.py`，原样输出同目录 `evidence/v2b-*.txt`）：
- 第一段基线里耗时 ≥ 10 秒的测试 191 条；在 55 份结局是「抓到」的变异日志里，这些测试 PASS 了 1 446 次。PASS 的耗时 / 基线耗时：中位 1.89，p90 10.51，p99 22.59，最大 30.59，最小 0.17（`evidence/v2b-out-long.txt` 第 3 行）。
- 最大的那一次：`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/baseline.one-test.log:608`「PASS [  10.370s] (602/776)」，同一条在第 420 条变异的日志里 `/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/420.one-test.log:977`「PASS [ 317.194s] (617/776)」。
- 一份日志里所有长测试一起变慢：第 466、468 条（相隔 9 分钟跑完）长测试耗时比的中位是 11.77 与 11.07；也有一起变快的：6 份日志的中位在 0.25–0.31（`evidence/v2b-out-perlog.txt` 第 4 行与第 46–51 行）。后者说明基线那一趟本身就比后来慢 3–4 倍——基线是在负载下量的（推的：每份里都是 7 条长测试，在 6 条不相干的变异下一起快 3–4 倍，变异本身让它们变快说不通）。
- 分不开的：这几份日志里的变慢有多少是负载、多少是变异让测试变慢（第 420、466、468 条都在 `allocator.rs`，可能让很多测试一起变慢）。对 B1 两者结局一样：超了就记「抓到」，而超不超取决于当时的负载。
- 照 B1 套这批 PASS（`evidence/v2b-out-long.txt`）：倍数 2、下限 10 秒，1 446 次里 647 次超；倍数 3、下限 30 秒，430 次超；倍数 5、下限 60 秒，216 次超，落在 10 份日志里。这批日志里每份都另有一条 FAIL，所以这一版的结论不变；同样的事落在一条没红的变异上，它就被记成「抓到」。
- 要让这一版自己的数据里一次都不超，倍数要 ≥ 31（最大比 30.59，取样上沿，没量到边界）；那时第一段最长的那条测试（基线 147.7 秒）挂住要等 4 580 秒以上才记「抓到」。

### 4.3 墙钟随负载涨，CPU 时间不怎么涨（副本量过，一次取样）

同一条 harness 测试（基线 3.95 秒的 `seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish`），用 `taskset` 挤在同两个核上并发跑（`evidence/v2b-wall_vs_cpu.py`）：
- `evidence/v2b-wall-vs-cpu.txt` 第 2 行「concurrent=1 cores=30,31 wall-min=4.29s wall-max=4.29s cpu-min=4.28s cpu-max=4.28s exit-codes=[0]」
- `evidence/v2b-wall-vs-cpu.txt` 第 7 行「concurrent=12 cores=30,31 wall-min=12.48s wall-max=18.73s cpu-min=3.09s cpu-max=3.20s exit-codes=[0]」
- 没有变异、一字没改，单是挤在一起墙钟就涨 4.4 倍（4.29 → 18.73 秒）；CPU 时间在 3.09–5.56 秒之间（单跑三次就散到 4.28–5.56，第 2–4 行），不随挤的程度涨。
- 读法：B1 用墙钟，倍数要盖住负载的散；改用每条测试的 CPU 时间限，散小得多。CPU 限时管不到不耗 CPU 的挂（等锁、睡眠），那一种还要一个墙钟的兜底（推的，没造等锁的变异）。

### 4.4 「倍数取多少」两头都输

- 倍数小：负载一高就把会过的测试记成「抓到」并复用（4.2）。
- 倍数大：真挂住的变异每条要等倍数 × 基线；这一版 7 条超时里，第 102 条是确定的死循环（第一轮 3.3），挂在哪条测试上就要付那条测试基线的 31 倍以上。
- 基线本身只量一次，在负载下量的会偏大 3–4 倍（4.2 第三条），偏大的基线让真挂住的更晚才停。

## 五、V2④：常量闭包只凭语法能不能找全

正文：「由工具从被改文件与测试执行到的文件出发，沿常量、`static`、const fn 的引用逐跳展开到不动点，结果并进关联文件（B3）。」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:40`）

- `use` 改名、通配引入、关联常量：只凭语法按**名字**过近似就收得到——起点文件里出现过的标识符，工作区里凡有同名的 const / static / const fn 就并进定义文件。`use X as Y` 的那一行里 `X` 本身就是一个标识符，通配引入的名字在用的地方出现，关联常量 `T::N` 的 `N` 也出现。模型 `run.py` 就是这么收的。代价是收多（同名的常量全进来），不是收漏（推的，没在 crates 上数收进来多少）。
- 泛型常量参数 `f::<{ N }>()`：`N` 出现在调用处，调用处在测试文件或执行到的文件里，起点覆盖得到（推的）。
- 收漏的是「不是常量、却像常量一样在编译期定死结局」的项，跟语法好不好无关，是 P3 的范围只列了常量、`static`、const fn：
  - 类型别名（FC）、枚举判别值（FD）、再导出（FB）：模型里按名字过近似的闭包也一样漏（第二节 2.3，`run-output.txt` 第 45、55、65 行的读法甲与第 48、58、68 行的读法丙都是「reuse caught」）。
  - `size_of::<T>()` 一类读类型布局的 const fn：被叫的是标准库的 const fn，结局取决于 `T` 的字段，字段所在文件不是常量定义文件（推的；今天 crates 里 `size_of::<` 0 处）。
- 把闭包的范围扩到「任何同名的类型别名、struct、enum、union、trait 与 `pub use` 再导出」（读法丁，攻方提的改法，被攻过零轮）：FB、FC、FD 三格都改判重判（`run-output.txt` 第 49、59、69 行）。它在真 crates 上会把关联集扩到多大没量：core 的类型名几乎每个文件都在用，猜是整包（推的）。

## 六、V3：代价

### 6.1 函数边界在今天的清单上切得开（副本量过）

`research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/v3-parser-check.txt:1`「mutations 5091 {'innermost-name-matches': 4918, 'no-enclosing-fn': 170, 'innermost-name-differs': 3}」
- 用 `run.py` 里那段只凭语法的切法（去注释与字符串、配括号），给清单每条变异找包住它的最内层 fn，与变异名里的函数名比：4 918 条对上；170 条不在任何 fn 里（常量初始化式，B2 一律跑）；3 条对不上是变异名带 `PoolWriter<'_, Device>`、我的名字正则切错，不是切函数错。嵌套 fn 一条都没有（输出里没有 nested 一项），宏 0 处（第一轮）。
- 所以「函数边界解析的边角」今天不是代价的来源；代价来自下面两件。

### 6.2 「被改的那一份」按函数省多少：只看被改文件，最近 15 个提交（副本量过）

`evidence/v3-fn-changes.txt`（`fn_changes.py`：被改函数变了、或那份文件「函数外面的部分」变了就重判；清单是库目录这一版的 5 091 条）：
- `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/v3-fn-changes.txt:9`「8186d5b5 1 0 17 2 2」：改 1 个函数，按文件 17 条、按函数 2 条。
- `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/v3-fn-changes.txt:11`「495bede6 5 0 3452 203 203」：改 5 个函数，按文件 3 452 条、按函数 203 条。
- `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/v3-fn-changes.txt:4`「973e1f58 16 0 1793 105 105」：改 16 个函数，按文件 1 793 条、按函数 105 条。
- `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/v3-fn-changes.txt:2`「dbcc0d2f 42 9 3006 3006 495」：42 个函数、9 个文件的函数外部分变了，按函数一条没省（不算函数外部分是 495 条）。
- 15 个提交的中位：按文件 3 073 条（60.4%），按函数 1 308 条（25.7%），不算函数外部分 495 条（9.7%）。函数外部分在 15 个提交里有 10 个变了，变的多半是 `use` 列表与类型、枚举定义（`evidence/v3-outside_diff.py` 跑 0a47816d、dbcc0d2f、bc57af7a 三个看过，没落文件）。

### 6.3 加上关联文件之后：样本里按函数几乎不省（副本量过，样本有偏）

`evidence/v3-rerun-estimate.txt`：这一版判成「抓到」的 242 条里，T 的执行文件量到的 235 条（186 条 T 是 core 库测试），拿同样 15 个提交当下一版，数四种键各重判几条：
- `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/v3-rerun-estimate.txt:12`「495bede6 39 27 | 197 197 196 74」：按文件 197、读法甲 197、读法乙 196、乙宽 74。
- `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/v3-rerun-estimate.txt:5`「973e1f58 146 1915 | 106 106 106 91」
- `research/prompts/tooling-broad-mutation-prune-r2-opus-model/evidence/v3-rerun-estimate.txt:10`「8186d5b5 51 158 | 47 47 47 47」：这个提交只改了 1 个函数，却有 47 条重判——它全仓改名，core 与 checker 的 src 里按记号比只有 1 个函数变了，其余是注释，按文件比的 sha256 照样变。
- 15 个提交里读法甲与按文件逐个相等；读法乙只在 2 个提交上少 1 条；乙宽（T 的测试文件就是被改文件时只比 T 那个测试函数）中位 91，按文件中位 197。
- 两个原因：
  - 读法甲：T 执行到的文件含被改文件（2.0）。
  - 读法乙：单元测试的「T 所在测试文件」就是被改文件。这一版 242 条「抓到」里 164 条的 T 与变异在同一个文件（按 T 的模块路径与变异文件比），P2 按文件比测试文件，被改文件里任何一处变了照样重判。
- 样本有偏：这一版只判了清单前 303 条，集中在 `address.rs`、`admission.rs`、`allocation_record_tree.rs`、`allocator.rs`，凡动到 `allocator.rs` 的提交都把样本重判掉大半。

### 6.4 覆盖遍要多久（副本量过，机器负载 23–29）

`evidence/v3-instr-cost.txt`（`instr_cost.py`；core 库测试 132 条，harness 8 个测试目标 77 条）：
- 一条测试一个进程（nextest 的跑法），插桩 / 不插桩：core 0.31 / 0.31 秒；harness 8 个目标合计 396.8 / 221.3 秒，约 1.79 倍，各目标 1.06–2.91 倍。
- 一个进程里多线程跑全部测试，插桩之后慢到 0.96–22.41 倍（corrupt_on_disk 那个目标 12.87 → 288.37 秒）：覆盖遍只能一条测试一个进程。
- 每条测试的 `llvm-profdata merge` 加 `llvm-cov export`：core 每条约 0.06 秒，harness 每条 0.1–0.25 秒。
- 推到第一段（基线 278.5 秒，776 条，`baseline.tsv` 第 3 行）：插桩跑约 1.8 × 278.5 ≈ 500 秒，逐条导出约 776 × 0.15 ≈ 120 秒，再加一次插桩编译；一版约 10 分钟（推的）。第二段（慢档 936 秒、checker 档 release）插桩的倍数没量。

### 6.5 一次提交改 1、5、20 个函数

| 改动 | 例 | 被改那一份：按文件 / 按函数（全表 5 091 条） | 样本 235 条：按文件 / 甲 / 乙 / 乙宽 | 单驱动耗时（推的） |
|---|---|---|---|---|
| 1 个函数 | 8186d5b5 | 17 / 2 | 47 / 47 / 47 / 47 | 只算被改那一份：2 × 124.8 秒 ≈ 4 分钟；关联文件照样按文件比，样本里 20% 重判，换到全表约千条量级、三十几小时（推的） |
| 5 个函数 | 495bede6 | 3 452 / 203 | 197 / 197 / 196 / 74 | 被改那一份 203 条 ≈ 7 小时；样本按乙 83% 重判，全表照此约 147 小时 |
| 16 个函数 | 973e1f58 | 1 793 / 105 | 106 / 106 / 106 / 91 | 被改那一份 105 条 ≈ 3.6 小时；样本按乙 45% 重判，全表照此约 80 小时 |

- 124.8 秒是这一版 303 条（只第一段）平均的编译加测试耗时（`results.tsv` 第 7、8 列）；第一段没红的 15 条平均 648.7 秒、超时 7 条平均 1 510.6 秒，清单后面的文件更贵还是更便宜不知道。
- 编译次数：每条重判一次编译（这一版 242 条「抓到」的编译加测试中位 19.1 秒）；覆盖遍每版多一次插桩编译。按函数不减少每条的编译，只减少条数。
- 照正文第六节第三条「V3 估出一次提交改 5 个函数也要几十小时 ⇒ 记下，交用户看值不值。」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:83`）：照样本外推，读法甲、乙下改 5 个函数是一百多小时；只有乙宽（再细到测试函数）样本里降到三成多。外推是推的，样本偏在 `allocator.rs`。

## 七、打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:126`「打中之后先问四句」）

判据：「V2 造出一条「第二版判可复用、重跑结论翻面」的反例 ⇒ 打中，改规则，并把这个反例做进 P4 的自证样本。」（`research/prompts/_tooling-broad-mutation-prune-r2-body.md:81`）

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足判据字面的哪一句 | 改法在这几格上 |
|---|---|---|---|---|
| FA、FA2、FF 同文件另一个函数 | 分辨：读法乙中、读法甲不中；用户定的「按函数」只在读法乙下才省东西，所以打中的正是按函数省下的那一块 | 看得到：两版被改文件的全文都在；T 在两版都执行了那个兄弟函数（函数级覆盖看得到） | 第 81 行那一句，反例在模型上 | 读法甲修（量过）但按函数省 0（6.3 量过）；关联细到「T 执行到的函数」修（推的），可它又撞上 FH 的函数级前提（3.2） |
| FB、FC、FD 文件外无覆盖区域的项 | 不分辨甲、乙，也打中第一轮按文件的设计；分辨的是 P3 的范围：只收常量的闭包中，收全部项的闭包（读法丁）不中 | 静态看得到（再导出、别名、判别值都在源码里）；覆盖看不到（这些文件没有可执行的代码） | 第 81 行那一句，模型上；今天的 crates 里同形的只有零星几处（2.3 末条） | 读法丙不修（量过：第 48、58、68 行）；读法丁修（量过：第 49、59、69 行），它在真 crates 上的关联集多大没量 |
| FH、FH2 变异体走进基线没走过的文件 | 不分辨甲、乙，也打中按文件；分辨的是覆盖取哪一趟：基线并集中，「变异体那一趟 ∪ 新版基线」（读法丙）不中 | 只量没改过的树时看不到（判别子观测不到）：变异体走到哪里，要在判它的那一趟插桩才看得到 | 第 81 行那一句，模型上；真 crates 上按文件的前提 236 条里 0 条，按函数的前提 32 条 | 读法丙修（量过：第 88、97 行）；代价是判每条变异都插桩（6.4 的 1.8 倍，推的） |
| B1 限时把慢记成抓到 | 不分辨复用的各种键：凡是「超时 = 抓到、抓到可复用」的组合都中；分辨的是墙钟限时与 CPU 限时 | 只有墙钟时看不到「是负载还是变异」；同一趟里 CPU 时间看得到一部分（4.3） | 第 81 行那一句的前提（PASS 超过 B1 的限时）在库目录产物里出现（倍数 5、下限 60 秒时 216 次）；端到端的翻面（一条没红的变异被记成抓到、空闲重跑翻回）没造出 | 正文没给改法；第八节的改法都是推的 |
| 代价：按函数省不下 | 不是打中，是 V3 那一格 | — | 第 83 行那一句：照样本外推改 5 个函数一百多小时 | 乙宽把样本里的重判降到三成多，代价是 FA 型的洞扩到测试函数（推的） |

## 八、几个改法各修哪一格（都只在我的模型上量过或没量，被攻过零轮）

| 改法 | 修哪几格 | 量过 / 推的 |
|---|---|---|
| 读法甲：被改文件照样按文件进关联 | FA、FA2、FF | 量过（`run-output.txt` 第 25、35、75 行 rerun）；代价是按函数一条不省（6.3 量过，样本） |
| 关联细到函数：T 在两版执行到的函数，加被改函数用到的函数外项 | FA、FA2、FF；让按函数真省下东西 | 推的；单独上它，FH 的前提在真 crates 上 32/236 条成立（3.2 量过），要与读法丙一起上 |
| 读法丙：覆盖取变异体那一趟（抓到的取 T，没红的取在变异体上执行到被改文件的每条测试）∪ 新版基线 | FH、FH2；按文件比时也修 FA、FA2、FF | 量过（模型第 28、38、78、88、97 行）；每条变异都要插桩跑，倍数 1.06–2.91（6.4 量过，harness 8 个目标） |
| 读法丁：名字闭包扩到类型别名、struct、enum、union、trait 与 `pub use` 再导出 | FB、FC、FD | 量过（模型第 49、59、69 行）；真 crates 上关联集多大没量，猜近整包（推的） |
| 单条限时改用 CPU 时间，另加一个宽的墙钟兜底 | 4.2、4.3 的负载那一半 | 墙钟与 CPU 的散量过（副本一条测试）；规则本身推的 |
| 超时得来的「抓到」不复用，记之前在空闲时单跑那一条测试复核 | 4.1 的复用那一半 | 推的 |
| 基线耗时取几趟里的最小值 | 4.2 基线偏大 3–4 倍那一条 | 推的 |
| 乙宽：T 的测试文件就是被改文件时，只比 T 那个测试函数 | 6.3 的代价 | 量过（样本中位 91 对 197）；同时把 FA 型的洞开到测试函数（推的） |
| P4 自证样本补 FA、FA2、FF、FB、FC、FD、FH、FH2 | 让以后的剪枝规则在这八格上会红 | 模型 `scenarios.tsv` 里现成；没接进任何门禁 |

## 九、没打中的形状（各只取样一次，不支撑「没有」：`.claude/rules/three-way-inference.md:159`「**一次不算**」）

| 形状 | 试了什么、取样多大 | 为什么没打中 |
|---|---|---|
| 函数边界的边角：宏里的 `fn`、嵌套 fn、闭包、`impl Trait` 返回、trait 默认方法 | 模型 FF（嵌套）；清单 5 091 条逐条切（6.1） | 嵌套在模型上只打中读法乙内；今天的清单里嵌套 0 条、宏 0 处；闭包与 `impl Trait` 的变异落在包住它的 fn 里，切法对上；默认方法没单独造 |
| `#[cfg]` 换掉别的文件里的代码 | 读规则，没造 | 加进或拿掉的是代码时，它在某一版的覆盖里，并集接得住（推的）；拿掉或换的是别名、常量这类时同 FC |
| `impl` 块上的属性 | 同文件的属于第 33 行的「属性」 | 别的文件里的 `impl` 带 `#[cfg]` 同上一行 |
| 类型字段的次序或默认值 | 读规则 | 带 derive 的由 B8 收；不带 derive 时字段次序只经 `size_of` 一类影响结局，今天 crates 里 0 处（推的） |
| 常量闭包的名字解析（`use` 改名、通配、关联常量、泛型常量参数） | 模型的按名字过近似 | 按名字过近似收得到（推的，没在 crates 上数） |
| 第二段（慢档、checker 档）的覆盖与倍数 | 没量 | checker 档属重型，不跑 |

## 十、这条腿自己的限度

- 翻面（FA、FA2、FF、FB、FC、FD、FH、FH2）都只在模型上造出、真跑；真 crates 上只量了 FH 的前提（236 条），没在 crates 上造「基线绿、只改一处非关联的地方、结论翻面」的第二版：要写新代码，而改 crates 的代码会动别的测试。
- 每个场景的第二版只取了一种改法（`v2/` 下那一份），没把「第二版怎么改」放开扫。各读法「不中」只对那一种改法成立，不说明那条读法在这一格上一般不中。
- 模型里的 P3、B8、读法乙的切函数都是我写的工具（按名字过近似、只凭语法），不是正文要的那个工具；它比只凭语法解析名字的工具收得多，打中的格对更窄的工具同样成立，没打中的格不一定。
- 4.2 的散分不开负载与变异；4.3 只量了一条测试、一种挤法；6.4 量的时候机器负载 23–29（别的会话在跑 cargo test 与 checker 档），插桩倍数是那时候的。
- 6.3、6.5 的样本是这一版判过的 235 条，集中在清单前几个文件；外推到全表是推的。
- 两批作废的跑：core 库测试那批第一次把基线覆盖量在了变异体的二进制上（cargo 把测试二进制写回同一路径），第二次拷回原件后 mtime 比编出来的旧、cargo 当成没变；两次都作废，改成先量完全部基线、并在干净的编译目录里重跑，作废的输出留在草稿目录 `crates/` 下没进仓。`instr_cost.py` 第一次在 checker_known_bad_images 那个目标撞了 6G 上限（包装退 250），那一次输出不算；第二次去掉那个目标重跑。
- 一切都只抽了一次样，没有 s2。

## 十一、收尾

- 删了（草稿目录 `/tmp/claude-1000/tooling-broad-mutation-prune-r2-opus/` 下）：`repo` 11M 与 `repo2` 11M（HEAD 导出的副本）、`target-cov` 170M、`target-cov-harness` 1.9G、`target-plain` 566M、`v3/work` 3.5G（编译目录与 profraw）、`model-work` 3.7G（模型各版的树与编译目录）、`tools` 236M（llvm-tools 1.98.0）。草稿目录里留着脚本、日志与报告草稿，不到 1M。
- 运行时看到的负载：开工时有两条别的会话的 `cargo test`（16、31 线程）与一条 checker 档装置在跑，负载 7；后来升到 23–29。内存包装两次因为同一会话已有别的跑（主会话派出去的别的腿）要求设并行理由，设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`；一次因整机内存排不上，把上限从 8G 降到 4G、1G（峰值表里实测峰值 527M）。没有等 cargo 的锁。
- 没做：没跑 checker 档、没起广谱驱动与 cargo-mutants；没量第二段的覆盖；没在真 crates 上造翻面；没判辩方那几格；没替主 agent 采纳任何改法。
- `python3 research/scripts/cite-check.py research/prompts/tooling-broad-mutation-prune-r2-opus-output.md` 末行：「✓ 核了 33 处引文，对上 33 处，没判 0 处（逐处列在上面）」（加这一行之前跑的）。
