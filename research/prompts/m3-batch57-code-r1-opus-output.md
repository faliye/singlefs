# m3-batch57-code-r1 攻方腿（Opus）报告：删除线第 57 批代码轮第一轮（2026-10-10）

立场：云端攻方，攻 K1、K2、K3（K4 顺带），在冻结副本的拷贝上造反例。被判对象是冻结副本 `~/.local/share/singlefs/work/m3-batch57-code-r1/repo/`（拷到草稿目录后 `sha256sum -c research/prompts/m3-batch57-code-r1-snapshot/crates-sha256.txt` 8 行全 OK），下面的代码行号按冻结副本。前一轮判决 `research/prompts/m3-batch56-code-r1-main-verification.md`。没读禁读清单里的三样。H1–H4 修的那几形不当打中报；第二节列的已知欠账不当打中报。

## 复跑

模型目录 `research/prompts/m3-batch57-code-r1-opus-model/`。从空目录起（模型目录拷出仓用时设 `SINGLEFS_SCRIPTS=<仓根>/scripts`；`SINGLEFS_REPO` 缺省就是这一轮的冻结副本）：
```
SINGLEFS_SCRIPTS=$PWD/scripts OPUS_THREADS=10 OPUS_MEMORY_CAP=4G \
  bash research/prompts/m3-batch57-code-r1-opus-model/rerun.sh <空工作目录> all
```
`all` 依次跑 `rerun.sh <工作目录> list` 列的 41 个阶段与 compare：base-build、base-fast（被判的四份用例在副本上照绿）、base-slow（两份用例的四条慢档）、k1-random-b1…b4（旋钮 ≥ 2 的随机历史）、k4-middle、k3-probe、7 个 `mut-*`（我的 6 种改坏与第 57 批 H4 两行）、25 个 `row-*`（第 57 批变异表里不碰 `.wgsl` 的 25 行，照各行自己的 cargo 参数跑，看点名的测试红没红）。只拷冻结副本的 `Cargo.toml`、`Cargo.lock`、`crates/`；原型 crate `probe-crate/` 拷成副本工作区成员 `crates/opus-probe-b57`（测试目标名不带 layer0）；`probes/corrupt_on_disk_k3_append.rs` 接到副本里 `corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 末尾。base、mutw、mutr 三个副本各用自己的 `CARGO_TARGET_DIR`，同一时刻一条 cargo，每条经 `capped.sh` 与 `run-with-memory-cap.sh`。前台有超时的环境按阶段名分几次起、同一个工作目录接着跑。挂钟（本机负载 11–23 下）：编译约 1 分钟，base 与原型阶段约 12 分钟，变异与 25 行约 12 分钟。

每个文件的 sha256（在 `research/prompts/` 下跑 `find m3-batch57-code-r1-opus-model -type f | sort | xargs sha256sum`）：
```
caa8294fa130624eb09fbdb7a1c63d2cc8a0f2cb7a28cab71c6071010b3f27be  m3-batch57-code-r1-opus-model/compare.py
56f6e6304d02d413bb7d5d463ac5cdc58551266dc7269b467fc385815f39b913  m3-batch57-code-r1-opus-model/expected/base-build.txt
0933eb8221c056b20f7f193f0907dc011d8fbfbfe624ccb7ab6642989856e1dd  m3-batch57-code-r1-opus-model/expected/base-fast.txt
0835bdc08e13d82d6698b14e334c92351e50b65daab056e24582a9c70df0b6ed  m3-batch57-code-r1-opus-model/expected/base-slow.txt
6c3a8d972b8d976759d686adec766c79bad5181ca06c930d3a7efc1e85a05bc8  m3-batch57-code-r1-opus-model/expected/k1-random-b1.txt
55fff283727738e2cc93c3fdbe601d9889575dabc06d7452eba8df431748f5b7  m3-batch57-code-r1-opus-model/expected/k1-random-b2.txt
cbfbce2c3ff825abe07c967eb98ff17cfae91bfff027ee62c2f60ca584838223  m3-batch57-code-r1-opus-model/expected/k1-random-b3.txt
31feed0a4d77ae8ef1b98fdf0380f14a2f19ada4dce5398975ddc10d6cf73319  m3-batch57-code-r1-opus-model/expected/k1-random-b4.txt
009f7dd79457e77660f57a0d19df3a924baeaa120ca8e52ac33bec95689660ff  m3-batch57-code-r1-opus-model/expected/k3-probe.txt
8e0863cac7b79cc9d35fe3ab23c8660dbff2d1c3a0c80be2736f6d49b9f87fa6  m3-batch57-code-r1-opus-model/expected/k4-middle.txt
823b978c7522573c87aec51bb111ca8b0e2914354eb1f44ae3bcbbfcde1cff3b  m3-batch57-code-r1-opus-model/expected/mut-h1-first-only.txt
4bd3170bf9e9a6e9c0783b8d00a0dcd1b8607f65de540fe27f451926c79ec49c  m3-batch57-code-r1-opus-model/expected/mut-k2-image-path-skips-product.txt
136f16196dced9932c9005583b3836fb2af42d12e997b85b4ff90a617799315a  m3-batch57-code-r1-opus-model/expected/mut-k3-cold-walk-mapping-count-first.txt
40e235c6d7be50e8fa956ff43c847a56b75b91d97ac752b416a1f045a30027c1  m3-batch57-code-r1-opus-model/expected/mut-k3-count-skips-unregistered-tag.txt
40e235c6d7be50e8fa956ff43c847a56b75b91d97ac752b416a1f045a30027c1  m3-batch57-code-r1-opus-model/expected/mut-k3-tag-judged.txt
eaa6f80e61c734fb3d7feb1cadfc6d9008c758ad62cde0791905ae6fdff6fdac  m3-batch57-code-r1-opus-model/expected/mut-row3850.txt
badeef91ab88b3c7006e70cc742b591992e08630ae775f659459c8c9ed5e7da0  m3-batch57-code-r1-opus-model/expected/mut-row3851.txt
51cc581700c3afa03e92be8d210de7c966c551fb7eea9d42178ae5f7cf7d3183  m3-batch57-code-r1-opus-model/expected/row-3821.txt
53e2cedc7128e72cbf9139754153d6a3205c3ed9d62f45745d85493ebe80b453  m3-batch57-code-r1-opus-model/expected/row-3822.txt
c14519cfe66c849bf73e95c41e76f9b777d8dd2857d1a9eb8af09a84a039740a  m3-batch57-code-r1-opus-model/expected/row-3823.txt
97546e94ee0d5eed0f19325ffb703d76b4da4078c1abd5d8efae409c8a2a7156  m3-batch57-code-r1-opus-model/expected/row-3824.txt
decf7cf30cb40cbe5a46df2fa8567d6c86deedc060eed31ba9674d5fb19d2337  m3-batch57-code-r1-opus-model/expected/row-3825.txt
e7db59f4f722619a2c6d7fc60985b6c8d7f57cc2ffbbd34a2039aba7f81bee13  m3-batch57-code-r1-opus-model/expected/row-3826.txt
051d53becdcba56b2bbdde6dc8c089c69428a5c36f946c65b6c62081f02647e3  m3-batch57-code-r1-opus-model/expected/row-3827.txt
ed61e001d8720ed818f366f72e58cbab008db0591983ce541c3c8468de36e0c1  m3-batch57-code-r1-opus-model/expected/row-3828.txt
58e6de28608dfcf76221e9b850a27e776afdb65129ccf5e229b0225ba16f6432  m3-batch57-code-r1-opus-model/expected/row-3830.txt
a7dd4b77070427e34519efe7cda8b1cf4764a8a735ffa96d72d19086fdcd8764  m3-batch57-code-r1-opus-model/expected/row-3831.txt
1ca5682ec501b7d9f4a01fd753c65439d624ca108095801bd6256b4a426a5024  m3-batch57-code-r1-opus-model/expected/row-3832.txt
78054b8816136500a71b48fa8554171b9c49e72d568a32d962c38e9a213c96c0  m3-batch57-code-r1-opus-model/expected/row-3833.txt
2983ae0a7029446fa967800cb5d82d4ecdba21644a2d81d79fef7fa29a125eee  m3-batch57-code-r1-opus-model/expected/row-3834.txt
bcdcdb9019ed6ad3877dae289ef0b9620a65a8a073ec5a8a68589d1cc48b1e1d  m3-batch57-code-r1-opus-model/expected/row-3835.txt
ec5dd65db507fbda4fccdd3fe82a8fc45cfe044f1cc20bcc0682208822197ff6  m3-batch57-code-r1-opus-model/expected/row-3836.txt
692a90d99abe8c4eed65fe4f45221eb74277792356a71f27453dd2aa05b874ef  m3-batch57-code-r1-opus-model/expected/row-3837.txt
0f8ae42db425bff9d2b3db08fb4b53acfa7d89b8085b636bf1ef9c3d22ae4d4e  m3-batch57-code-r1-opus-model/expected/row-3839.txt
46da1e2cd42d178e4bdcf013ebb7b71dab9f4c9b47c9356e484fcd0ab565db58  m3-batch57-code-r1-opus-model/expected/row-3840.txt
a34d2c9dfca0048078bf4ef681180ea73a67c62fa21ca8429e842a1ea916ea7a  m3-batch57-code-r1-opus-model/expected/row-3841.txt
7be099d363b698c7a0fa0aa2f76da7e10e81334b36a193aed3b692847cdf59c8  m3-batch57-code-r1-opus-model/expected/row-3842.txt
6562da32b140638bcf7e42d4445f8ed2c6e7c30f82a42a9c7cbb0d0b534787a2  m3-batch57-code-r1-opus-model/expected/row-3843.txt
ba80df4895066a935c4ef52bdb6d6302024d5ba7d712b19941e9e32979a95d5b  m3-batch57-code-r1-opus-model/expected/row-3844.txt
71b268bf76c4f262b92c9defdebebc1611efa6199744ab0dd609faae3433a81d  m3-batch57-code-r1-opus-model/expected/row-3845.txt
b0363078b84cafc79651c598f27ec7322cbd41852cbcc95ebe1ce91cbd57f58e  m3-batch57-code-r1-opus-model/expected/row-3846.txt
7ef1b19020b286706a8f3f2448460a038ce69216de9327bf7ec5fca7d95c2af9  m3-batch57-code-r1-opus-model/expected/row-3848.txt
4388ea7853a7adb0426f7cc60ee88f1943d8e0c34dee06d11cfdc7b5e6a0470e  m3-batch57-code-r1-opus-model/mutants.py
5ea2639729ec8d9e9dc8dfc0127407cccbf37a8b3e7ba289ca868a07630ac360  m3-batch57-code-r1-opus-model/probe-crate/Cargo.toml
1ea49b9b068b7d8f754219fe631f68ad305479bed58c3ef7040aea59e9335ac6  m3-batch57-code-r1-opus-model/probe-crate/tests/common_b57/mod.rs
cc5ae307f54b185f82c5be5907daca157551f8a06ec10c7fa47b9a59cab65dd4  m3-batch57-code-r1-opus-model/probe-crate/tests/k1_random_knob_histories.rs
301f9f4a75a6e4be279e6dd73049c13d98268aa06b9d46626f554eaa24a359fc  m3-batch57-code-r1-opus-model/probe-crate/tests/k4_middle_short_container.rs
e0064a16313eeedcaedaf3f765c7a33724e281a0c114a4ae78e8e784919c4142  m3-batch57-code-r1-opus-model/probes/corrupt_on_disk_k3_append.rs
b2c764a3886b03ea2d9035dcc320cb38197b8c650504783be340fdb4c43534c6  m3-batch57-code-r1-opus-model/rerun.sh
124b85cca5ba95fcdd1fd4054a53e1924bce34906ed32f7fdfc0d8cd96f80003  m3-batch57-code-r1-opus-model/sort_lines.py
80cb8bfa6af556e31082d46e951d615b57e9fe70b17cb91ffe2b6c43ae9c9607  m3-batch57-code-r1-opus-model/summarize_random.py
```

## 各格判定一览

| 格 | 判定 | 依据（详见各节；「量过」指本腿副本上的原样输出，产物是模型目录 `expected/` 下的文件） |
|---|---|---|
| K1-1 旋钮下合法镜像误报（最右没满、根长高、回退照抄、换实例重挂、旋钮各异、只一棵树装旋钮） | 没打中（CPU 量过；GPU 读码） | 量过：旋钮 ≥ 2 的随机历史 40 个种子判 5700 次、0 次有违例，新判在按步累计 60 793 片「不是最右的容器」上判过（`k1-random-b*.txt`）；harness 慢档旋钮 2 那条照绿（`base-slow.txt`）。GPU 那一支与 CPU 逐句同判（读码），但 GPU 上没有一条旋钮 ≥ 2 的合法流（推的缺口，不当打中） |
| K1-2 旋钮下坏镜像漏报（早关、多一条、最右之外少一条），三棵树 | 没打中（CPU 量过；GPU 读码） | 单版本形状都红：中间那片少一条的三棵树都 Violated（`k4-middle.txt`）；两片早关与多一条由第 57 批用例与 25 行变异钉住（`row-*.txt`）。跨版本缩片是已知欠账（跨版本容器集包含），不算 |
| K1-3 CPU 与 GPU 判得不同 | 没打中（读码，GPU 没跑） | 新判在 `d_judge_containers_under_the_capacity_knob` 里与 CPU 两个判法同循环界、同比较；三棵树共用一处 |
| K2-1 产品支三判与首片 0 的改坏拦不拦 | 没打中（量过） | 第 57 批 H2、H3 的 18 行 CPU 变异各红自己点名的那条（`row-38*.txt`） |
| K2-2 单测与整镜像是不是同一段代码 | 没打中（量过） | 同一个函数；整镜像调用点上让没装旋钮的那一棵交空表（`k2-image-path-skips-product`）默认档 5 条红、慢档 2 条红（`mut-k2-image-path-skips-product.txt`） |
| K3-1 那张坏镜像是不是只由次序分得出、有没有别的判先拒 | 没打中（量过） | 正确次序下冷走读与重建都报分配记录；H4 两行变异各红自己那条（`mut-row3850.txt`、`mut-row3851.txt`）；只坏记账那一处报记账条目数、只坏分配记录报分配记录（`k3-probe.txt`） |
| K3-2 「排在两个数之前」的另一个数（映射条目数） | **打中（分辨批次）** | 冷走读把数映射条目挪到分配记录判之前，`corrupt_on_disk…` 39 条全绿（`mut-k3-cold-walk-mapping-count-first.txt`）；分配记录代 + 映射条目数的两处坏镜像上，正确次序报分配记录、这一改坏报映射条目数，可写挂载仍报分配记录——两条路成员分叉 |
| K3-3 标签 13 让用例钉住的是标签 | **打中（分辨批次）** | 13 在 D5 已定项 4 / 10 里是登记过的「清扫水位」，用例注释说「没有登记」；逐条判拒标签 > 12（`k3-tag-judged`）或数条目不算标签 > 12 的行（`k3-count-skips-unregistered-tag`）——次序都没动——H4 两条都红在造镜像那一句前提断言上；不碰标签的替代镜像（删一行记账）在这两种改法下照报分配记录、在 H4 两行变异下照样分得出（`k3-probe.txt`、`mut-k3-*.txt`、`mut-row385*.txt`） |
| K3-4 重建那一条钉的是哪一个次序 | 没打中（量过 + 推的） | 钉的是重建里「分配记录先于数记账条目」（`row 3851` 红可写挂载那条，冷走读那条不红）；重建里数映射条目要用记账树的节点数，排不到分配记录之前（推的） |
| K4 判别力 | **打中一处（分辨批次）**；其余 25 行都红 | 25 行 CPU 变异全红（`row-*.txt` 各行末 `named-test-red=yes`）；H1 新判「只判叶序第一片」（`h1-first-only`）在 checker 单测 32 条、坏镜像 253 条、容器树 8 条（含两条慢档随机历史）上全绿，中间那片少一条的三棵树上判 Holds（`mut-h1-first-only.txt`）。GPU 4 行没跑 |

## K1　旋钮支装满（甲 H1）

### 新判在写侧与判法两边的样子（读码，冻结副本）
- 条款：「3. **开放容器**：有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。」（`.claude/kb/decisions/18-块里携带什么信息.md:578`）。
- 写侧只有一条开新片的路，旋钮 K 下照样装满 K 条才开：「.is_some_and(|open| open.records.len() < records_per_container);」（`crates/singlefs-core/src/container_tree.rs:537`）；回退那一版三棵容器树都取现行那一版的（`crates/singlefs-core/src/mount.rs` 回退拼 `TransactionOutput` 那一段，`tombstone_containers: current.tombstone_containers.clone()` 等三行），崩溃恢复之后从恢复取的那一版接着追加。推不出旋钮下「不是最右的那片少于 K」的写法。
- CPU：谱系 / 退役「record_count == records_the_knob_allows;」（`crates/singlefs-checker/src/walk.rs:12059`），最右那片跳过「if is_the_rightmost_under_the_knob {」（`crates/singlefs-checker/src/walk.rs:12055`）；墓碑「record_count == knob_records;」（`crates/singlefs-checker/src/walk.rs:12158`），跳过「if is_the_rightmost_tombstone_container_under_the_knob {」（`crates/singlefs-checker/src/walk.rs:12154`）。
- GPU：「if (d_container_record_count(container_index) != knob_records) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3357`），循环界「for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3356`）——与 CPU 的「最右那片之外」同界；零片在函数头返回，与 CPU「装了旋钮至少一片」同；三棵树都经这一个函数。

### K1-1 合法镜像误报：没打中
旋钮 1 下「== 1」与「≤ 1」加「每片至少一条」同义，新判只在旋钮 ≥ 2 时有内容；第 57 批之前已有的 GPU 旋钮流全是旋钮 1（例「knobs_of_the_retirement_tree(1),」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13082`）），第 57 批 GPU 新加的旋钮 2 只用在坏流上（「knobs_of_the_tombstone_tree(2),」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13695`））。所以我只取旋钮 ≥ 2。

**随机历史**（`probe-crate/tests/k1_random_knob_histories.rs`，内存稀疏盘，mkfs 与第一个文件只做一次、每个种子从它 `clone`）：`all` 取法三棵树都装旋钮（墓碑 2–4、谱系 2–3、退役 2–3，三棵各异）；`one` 取法只一棵树装旋钮（2–3）、另两棵照产品容量、checker 那两棵也给 `None`；码 2 节点 产品 / 2 / 3（2、3 时根满长高）。先建 234–241 个空 inode，再走 120 或 150 步：22% 建 1–3 个，20% 从记录最少的叶里删一个，20% 删最大的活号，10% 随便删，20% 挂着回退到最近 12 版之一（含之前各实例的版），8% 干净地可写重挂（换实例，挂载带同一组旋钮）。用户动作全放开，只固定第一个文件那一前缀。每步之后带同一组旋钮交 `check_pool_image_given_the_container_capacity_knobs`。四批各 10 个种子、10 个线程；先跑 2 个种子 × 20 步估时（约 0.15 秒一步一线程），定每批步数，不设限时；挂钟 50 / 83 / 68 / 78 秒（第一次跑那一遍，`expected/` 里把挂钟字段滤掉了）。`python3 m3-batch57-code-r1-opus-model/summarize_random.py m3-batch57-code-r1-opus-model/expected` 原样：
```
合计：40 个种子，判 5700 次，有违例的 0 次，新判判到的不是最右的容器（按步累计）60793 片，重挂 471 次，回退 1126 次
| 种子 | 取法 | 旋钮 墓碑/谱系/退役/节点 | 重挂 | 回退 | 判次 | 有违例 | 新判判到的片（累计） | 末版片数 墓碑/谱系/退役 |
|---|---|---|---|---|---|---|---|---|
| 1 | all | 3/2/3/3 | 13 | 26 | 120 | 0 | 1973 | 19/13/2 |
| 2 | all | 4/3/2/2 | 10 | 25 | 120 | 0 | 1265 | 15/9/0 |
| 3 | all | 2/3/3/0 | 10 | 29 | 120 | 0 | 2044 | 26/10/1 |
| 4 | all | 3/3/2/2 | 12 | 19 | 120 | 0 | 1333 | 18/7/1 |
| 5 | all | 3/3/3/3 | 11 | 22 | 120 | 0 | 1418 | 19/8/0 |
| 6 | all | 4/2/2/3 | 13 | 25 | 120 | 0 | 1390 | 15/13/1 |
| 7 | all | 2/2/3/0 | 7 | 24 | 120 | 0 | 2273 | 32/12/1 |
| 8 | all | 4/3/2/2 | 6 | 23 | 120 | 0 | 1099 | 15/8/1 |
| 9 | all | 3/2/2/2 | 9 | 25 | 120 | 0 | 1842 | 20/13/2 |
| 10 | all | 4/3/2/2 | 11 | 23 | 120 | 0 | 1154 | 14/8/1 |
| 11 | all | 3/3/2/0 | 6 | 34 | 150 | 0 | 2737 | 24/12/3 |
| 12 | all | 2/2/3/0 | 15 | 30 | 150 | 0 | 3394 | 31/15/0 |
| 13 | all | 2/2/3/2 | 13 | 27 | 150 | 0 | 3724 | 38/14/3 |
| 14 | all | 2/2/2/3 | 10 | 23 | 150 | 0 | 3214 | 36/12/0 |
| 15 | all | 3/3/3/3 | 11 | 25 | 150 | 0 | 2158 | 23/9/1 |
| 16 | all | 3/2/2/3 | 9 | 25 | 150 | 0 | 2822 | 27/13/2 |
| 17 | all | 3/2/3/0 | 9 | 34 | 150 | 0 | 2824 | 25/17/2 |
| 18 | all | 2/2/2/2 | 12 | 37 | 150 | 0 | 4006 | 38/19/2 |
| 19 | all | 4/3/3/0 | 14 | 27 | 150 | 0 | 1927 | 20/9/0 |
| 20 | all | 4/3/2/0 | 8 | 27 | 150 | 0 | 1951 | 19/9/0 |
| 21 | one | 0/0/3/2 | 10 | 40 | 150 | 0 | 0 | 1/1/1 |
| 22 | one | 0/2/0/3 | 17 | 27 | 150 | 0 | 760 | 1/14/1 |
| 23 | one | 3/0/0/2 | 15 | 23 | 150 | 0 | 1837 | 28/1/1 |
| 24 | one | 3/0/0/0 | 12 | 29 | 150 | 0 | 1832 | 26/1/1 |
| 25 | one | 0/2/0/0 | 9 | 31 | 150 | 0 | 1155 | 1/16/1 |
| 26 | one | 0/0/3/2 | 19 | 26 | 150 | 0 | 314 | 1/1/4 |
| 27 | one | 0/0/2/2 | 10 | 33 | 150 | 0 | 186 | 1/1/3 |
| 28 | one | 0/2/0/2 | 13 | 34 | 150 | 0 | 1255 | 1/17/1 |
| 29 | one | 0/2/0/3 | 17 | 30 | 150 | 0 | 986 | 1/15/1 |
| 30 | one | 0/0/2/3 | 16 | 29 | 150 | 0 | 0 | 1/1/1 |
| 31 | one | 0/0/2/0 | 9 | 30 | 150 | 0 | 0 | 1/1/0 |
| 32 | one | 0/0/2/2 | 14 | 24 | 150 | 0 | 0 | 1/1/0 |
| 33 | one | 0/2/0/0 | 7 | 34 | 150 | 0 | 1366 | 1/17/0 |
| 34 | one | 0/3/0/3 | 13 | 26 | 150 | 0 | 618 | 1/9/0 |
| 35 | one | 0/0/2/0 | 12 | 35 | 150 | 0 | 0 | 1/1/0 |
| 36 | one | 0/0/3/3 | 16 | 26 | 150 | 0 | 0 | 1/1/1 |
| 37 | one | 2/0/0/3 | 11 | 34 | 150 | 0 | 2773 | 39/1/1 |
| 38 | one | 0/0/2/0 | 16 | 32 | 150 | 0 | 89 | 1/1/3 |
| 39 | one | 2/0/0/3 | 12 | 28 | 150 | 0 | 3074 | 41/1/1 |
| 40 | one | 0/0/3/2 | 14 | 25 | 150 | 0 | 0 | 1/1/0 |
```
「新判判到的片」是每步之后那一版里装了旋钮的树「不是最右的容器」片数的累计，`one` 取法里只装退役旋钮的种子退役记录少，有 5 个种子一片都没判到（表里 0 的那几行），这一类只剩 26、27、38 三个种子判到。harness 自己那条旋钮 2 的慢档随机历史（「fn a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step() {」（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:585`））在副本上照绿（`expected/base-slow.txt` 第一段 2 条过）。

看到的一处覆盖缺口（推的，不当打中）：旋钮 ≥ 2 的合法镜像只有那条慢档用例与 GPU 那边零条——默认档不跑 `#[ignore]`（「#[ignore = "harness 慢档用例：60 步随机历史、每步一次池级 checker，debug 下约两分钟"]」（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:584`）），新判在合法镜像上误报的改坏要到提交时的慢档才红；GPU 旋钮 ≥ 2 合法流一条都没有，GPU 那一支的误报只靠读码。

### K1-2 坏镜像漏报：没打中
单版本里「不是最右的那片少于旋钮值」的三种排布都红：两片早关（第 57 批用例，25 行变异里 H1 那 6 行都红）、多一条（≤ 那一判，旧用例）、三片里中间那片少一条（`probe-crate/tests/k4_middle_short_container.rs`：写的时候中途把旋钮从 2 换成 1，写出 [2, 1, 1]，带旋钮 2 判）。`expected/k4-middle.txt` 三行的违例原样（截取每行 `violations=` 那一段）：
```
tree=lineage I-9.22=NotApplicable I-9.37=Violated violations=["I-9.37: 根（实例 1、txg 9）：谱系树（树 65546）里不是最右的容器 2 装了 1 条，不是旋钮值 2（满了才关）"]
tree=retirement I-9.22=NotApplicable I-9.37=Violated violations=["I-9.37: 根（实例 1、txg 16）：退役树（树 65548）里不是最右的容器 2 装了 1 条，不是旋钮值 2（满了才关）"]
tree=tombstone I-9.22=Violated I-9.37=NotApplicable violations=["I-9.22: 根（实例 1、txg 12）：墓碑树（树 65545）里不是最右的容器 2 装了 1 条，不是旋钮值 2（满了才关）"]
```
跨版本「后一版比前一版少片 / 少条」那一形单版本判不出，是第二节点名的已知欠账（跨版本容器集包含），不算。

### K1-3 CPU 与 GPU：没打中（读码，GPU 没跑）
见第一小节：同界、同比较、同一个空树出口；`d_container_record_count` 读的是容器头解出来的记录数，CPU 数的是解出来的记录条数，两边对同一张合法容器相等（读码）。GPU 一棵树多于 256 片时整态判不了、交 CPU，是旧上限，不是这一批的差别。

## K2　产品支三判与首片 0（甲 H2、H3）：没打中

### 单测与整镜像走的是不是同一段代码
是同一个函数。整镜像那条路在一版里调三次：谱系「judge_lineage_or_retirement_container_capacity_of_a_version(」（`crates/singlefs-checker/src/walk.rs:11403`）、墓碑「judge_tombstone_container_capacity_of_a_version(」（`crates/singlefs-checker/src/walk.rs:11496`）、退役（`crates/singlefs-checker/src/walk.rs:11650`）；第 57 批的单测直接调同名的两个函数（`crates/singlefs-checker/src/walk.rs:14042`、`:14064`）。三判、首片、产品容量那一句都在函数里面，单测改得到的整镜像也改得到。调用点本身（产品支交不交表）用变异量：`k2-image-path-skips-product` 让没装旋钮的那一棵交空表（判法零片返回、等于不判），`expected/mut-k2-image-path-skips-product.txt` 里红的原样：
```
command args=-p singlefs-checker --lib rc=0
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
command args=-p singlefs-harness --test checker_known_bad_images rc=101
    the_old_tombstone_knob_entry_judges_the_lineage_tree_by_its_product_capacity
    the_old_tombstone_knob_entry_judges_the_retirement_tree_by_its_product_capacity
    two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant
    two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant
test result: FAILED. 249 passed; 4 failed; 2 ignored; 0 measured; 0 filtered out; 
command args=-p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full rc=101
    a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant
test result: FAILED. 5 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; 
command args=-p singlefs-harness --test checker_known_bad_images -- --ignored rc=101
    nine_hundred_sixty_retirements_at_product_capacity_fill_every_retirement_container_but_the_rightmost_and_redden_nothing
    one_thousand_three_hundred_sixty_rollbacks_at_product_capacity_fill_every_lineage_container_but_the_rightmost_and_redden_nothing
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 253 filtered out; 
```
默认档就有 5 条红（旋钮写出、按产品入口判的那几条，判的是同一个调用点上的产品容量那一句），慢档两条产品容量用例也红。单测绿、整镜像那条路没判的改坏，我造的这一种拦得住。

### 改坏拦不拦（量过）
第 57 批变异表里不碰 `.wgsl` 的 25 行，照各行自己的 cargo 参数在副本上逐行跑（`row-*` 阶段，各行末一行是我加的「点名的那条红没红」）。`grep -h named-test-red expected/row-*.txt` 原样：
```
row 3825 named-test-red=yes
row 3826 named-test-red=yes
row 3822 named-test-red=yes
row 3830 named-test-red=yes
row 3839 named-test-red=yes
row 3843 named-test-red=yes
row 3824 named-test-red=yes
row 3833 named-test-red=yes
row 3827 named-test-red=yes
row 3832 named-test-red=yes
row 3835 named-test-red=yes
row 3831 named-test-red=yes
row 3823 named-test-red=yes
row 3840 named-test-red=yes
row 3834 named-test-red=yes
row 3821 named-test-red=yes
row 3828 named-test-red=yes
row 3837 named-test-red=yes
row 3846 named-test-red=yes
row 3841 named-test-red=yes
row 3845 named-test-red=yes
row 3844 named-test-red=yes
row 3836 named-test-red=yes
row 3848 named-test-red=yes
row 3842 named-test-red=yes
```
其中 H2 产品支三判 12 行（墓碑、谱系、退役各三句加「相邻要求加二」的误报形）、H3 首片 ≤ 1 六行（三棵树产品支单测与旋钮支坏镜像各一）都红。谱系与退役共用一个判法，表里「谱系树那一形」「退役树那一形」两行改的是同一处原文、跑的是两条不同的单测，两条都红说明两棵树的单测都走到那一句。

四句不适用（没打中）。

## K3　冷走读次序（乙 H4）

### 造了哪几张镜像（`probes/corrupt_on_disk_k3_append.rs`，接在副本那份用例文件末尾、用它自己的造法函数）
都从 H4 用例同一个池起（第一个文件之后再建一个空 inode，txg 4，记账叶根 16 条含一级台阶行）：① 只删中央映射树根 key 最大的那一条（映射条目数对不上）；② 只把罩树表单元那个槽的两条分配记录代改成 5（H4 用例的 ② 那一处）；③ ① + ②；⑤ 只删记账叶根 key 最小的那一行（记账条目数 14 ≠ 15，标签与代都不碰）；⑥ ② + ⑤；④ H4 用例那张原样（台阶行标签 13 + ②），单放一条测试。每张交只读恢复（冷走读）与可写挂载（重建）。正确次序下 `expected/k3-probe.txt` 原样：
```
OPUS_K3 image=accounting-row-dropped recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数" } }
OPUS_K3 image=accounting-row-dropped writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数" })
OPUS_K3 image=allocation-generation-after-the-root recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" } }
OPUS_K3 image=allocation-generation-after-the-root writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
OPUS_K3 image=allocation-generation-and-accounting-row-dropped recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" } }
OPUS_K3 image=allocation-generation-and-accounting-row-dropped writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
OPUS_K3 image=allocation-generation-and-mapping-count recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" } }
OPUS_K3 image=allocation-generation-and-mapping-count writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
OPUS_K3 image=batch57-h4-image recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" } }
OPUS_K3 image=batch57-h4-image writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
OPUS_K3 image=mapping-count-off recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数" } }
OPUS_K3 image=mapping-count-off writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数" })
```

### K3-1 那张坏镜像是不是只由次序分得出：没打中
只坏记账那一处报记账条目数、只坏分配记录报分配记录，两处一起时两条路都报分配记录——H4 那张在正确次序下不是先被别的判拒的。H4 两行变异各红自己点名的那条（`expected/mut-row3850.txt`、`expected/mut-row3851.txt` 的失败名）：
```
command args=-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking rc=101
    allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_writable_mount
test result: FAILED. 38 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; 
command args=-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking rc=101
    allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_recovery
test result: FAILED. 38 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; 
```

### K3-2 「排在两个数之前」只钉了一个数：打中
被判的说法：冷走读那一段注释「之后才数记账条目与映射条目——先判得了自己那一条的，再数」（`crates/singlefs-core/src/recovery.rs:5394`），用例那一节的标题「// ─── 分配记录逐条判代与跨度先于数记账条目与数映射条目 ───」（`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1690`）。两条用例的名字与造法只碰记账条目数（`…refused_before_the_accounting_entry_count_by_the_recovery`，`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1905`）。

历史与输入：冷走读里把「mapping_entry_count_is_one_per_mapped_unit(」（`crates/singlefs-core/src/recovery.rs:5407`）那一整段挪到「allocation_record_generations_and_spans_are_judged(allocation_records, root)?;」（`crates/singlefs-core/src/recovery.rs:5395`）之前（`mutants.py` 的 `k3-cold-walk-mapping-count-first`；数映射条目要的几样——各树节点数与 `deletion_tree_nodes`——在那之前都已读好，挪得动、编得过）。`expected/mut-k3-cold-walk-mapping-count-first.txt` 原样：
```
command args=-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking rc=0
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
OPUS_K3 image=allocation-generation-and-mapping-count recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数" } }
OPUS_K3 image=allocation-generation-and-mapping-count writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
```
用例文件 39 条（37 条原有加我的 2 条原型）全绿；③ 那张在正确次序下冷走读报分配记录（上一小节），在这一改坏下报映射条目数，而可写挂载（重建，没动）仍报分配记录：只读恢复与可写挂载对同一张盘报不同的成员——G1 / H4 要防的正是这种分叉。

四句：
1. 分不分辨批次：分辨——「排在两个数之前」是第 57 批 H4 自己的验收，它的用例只造了记账条目数那一个。
2. 判别它的东西冷走读当时看得到吗：看得到——分配记录与映射条目数在那一刻都已读进来（上面那一改坏编得过就是证据）；次序由一张两处坏的镜像观测得到（③）。
3. 满足的分句：K3「那张坏镜像是不是真只由次序分得出」——它只分得出分配记录与记账条目数的次序，分不出与映射条目数的次序；也落在 K4「还有哪几种改坏照绿」。
4. 改法在这格上中不中：H4 的用例在这一格上不中（量过：这一改坏下那两条照绿）。我提的改法：把 ③ 那张做成第三条用例，断言冷走读报分配记录——量过：正确次序报分配记录、这一改坏报映射条目数（只在我的模型上量过、被攻过零轮）。重建那一边见 K3-4。

### K3-3 标签 13 让用例钉住了标签：打中
用例的说法：「/// 台阶行的统计量标签换成的值：`singlefs_core::records` 登记的统计量标签到 inode 号水位 12 为止，这是之后的一个、没有登记。」（`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1692`）。代码里的常量确实只到 12（「pub const STATISTIC_INODE_WATERMARK: u16 = 12;」（`crates/singlefs-core/src/records.rs:29`）），但 kb 的登记表到 14：「统计量标签的登记表就是已定项 4 那张表——码 = 行号（1 到 14；第 8 / 9 项退役不复用；0 保留为无效）」（`.claude/kb/decisions/05-快照-空间记账机制.md:244`），第 13 行是「13 | 清扫水位（截至哪个 txg 的释放已由整轮清扫确认不可读）」（`.claude/kb/decisions/05-快照-空间记账机制.md:130`）。所以那一行不是「没登记的标签」，是登记过、代码还没实现的那个统计量。

H4 那张能分辨次序，要两条与次序无关的现状同时成立：逐条判收下标签 13，数条目又把它算进 3 + 6 × 盘数。造法里那一句前提断言把这两条一起钉住（「"前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目数"」（`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1879`））。两种不动次序的改法（`k3-tag-judged`：逐条判也拒标签 > 12；`k3-count-skips-unregistered-tag`：数条目不算标签 > 12 的行）下，`expected/mut-k3-tag-judged.txt` 与 `expected/mut-k3-count-skips-unregistered-tag.txt` 红的原样（两份相同）：
```
thread 'allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_recovery' panicked at crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1873:5:
thread 'allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_writable_mount' panicked at crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1873:5:
thread 'opus_k3_batch57_h4_image' panicked at crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1873:5:
    allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_recovery
    allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_writable_mount
    opus_k3_batch57_h4_image
test result: FAILED. 36 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; 
```
两条 H4 用例都红在造镜像那一句前提上（第 1873 行是那一句 `assert_eq!` 的起头），不是红在它们要钉的次序上。不碰标签的 ⑥ 那张在这两种改法下照报分配记录，在 H4 两行变异下照样分得出（各份原样）：
```
[mut-k3-tag-judged]
OPUS_K3 image=allocation-generation-and-accounting-row-dropped recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" } }
OPUS_K3 image=allocation-generation-and-accounting-row-dropped writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
[mut-k3-count-skips-unregistered-tag]
OPUS_K3 image=allocation-generation-and-accounting-row-dropped recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" } }
OPUS_K3 image=allocation-generation-and-accounting-row-dropped writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
[mut-row3850]
OPUS_K3 image=allocation-generation-and-accounting-row-dropped recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数" } }
OPUS_K3 image=allocation-generation-and-accounting-row-dropped writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
[mut-row3851]
OPUS_K3 image=allocation-generation-and-accounting-row-dropped recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" } }
OPUS_K3 image=allocation-generation-and-accounting-row-dropped writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数" })
```

四句：
1. 分不分辨批次：分辨——第 57 批乙自定、主 agent 接受、被攻过零轮的造法（正文第二节「乙用没登记的统计量标签 13 造「条目数对不上」」）。
2. 判别它的东西当时看得到吗：读者看得到标签；毛病不在读者，在用例的分辨力建在两条与次序无关的现状上。会动这两条现状的已在 kb 里：13 号统计量要实现（D5 已定项 4 第 13 行）、C71 要求「运行期的形态是挂载时遇到不认识的统计量标签必须按 compat_ro 拒绝可写挂载」（`.claude/kb/checks-owed.md:69`）。
3. 满足的分句：K3「用没登记的标签 13 造数不对，会不会让用例钉住的是标签而不是次序」——会，量过两种。
4. 改法在这格上中不中：第 57 批 H4 的造法就是这一格的对象，照中。我提的改法：造「记账条目数对不上」改用删一行（⑥ 那张，不碰标签与代）——量过：上面两种改法下照报分配记录、H4 两行变异下各分得出（只在我的模型上量过、被攻过零轮）。

### K3-4 重建那一条钉的是哪一个次序：没打中
钉的是重建里「分配记录逐条判」（「allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;」（`crates/singlefs-core/src/recovery.rs:2894`））先于数记账条目：`row 3851` 只红可写挂载那条（上面原样），③ / ⑥ 在 `row 3850` 下可写挂载仍报分配记录、冷走读改报——两条路互不连带，两条用例各钉自己那条路。重建里数映射条目在「let mapping_tree = central_mapping_root.into_tree()?;」（`crates/singlefs-core/src/recovery.rs:2943`）之后、要用记账树的节点数，记账树在分配记录判之后才读，挪不到分配记录之前（推的，没造这一改坏）。

## K4　判别力（顺带）

### 第 57 批新加的变异行：不碰 `.wgsl` 的 25 行都红
见 K2 那一节的原样（`named-test-red=yes` 25 行）。H4 两行在 K3-1。GPU 4 行没跑（checker 档）；读码：H1 那 3 行（墓碑、谱系、退役）改的是同一个文件里同一处原文、替换文也相同，只是点名的测试不同——三条测试各自都红，才说明三棵树都走到 `d_judge_containers_under_the_capacity_knob` 里那一句。

### H1 只判叶序第一片：照绿（打中）
改坏（`mutants.py` 的 `h1-first-only`）：两个判法里跳过最右那片的那一句各加「或不是第一片」——即只判叶序第一片装满，第二片起不判。`expected/mut-h1-first-only.txt` 原样：
```
command args=-p singlefs-checker --lib rc=0
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
command args=-p singlefs-harness --test checker_known_bad_images rc=0
test result: ok. 253 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; 
command args=-p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full -- --include-ignored rc=0
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
command args=-p opus-probe-b57 --test k4_middle_short_container rc=0
OPUS_K4M tree=lineage I-9.22=NotApplicable I-9.37=Holds violations=[]
OPUS_K4M tree=retirement I-9.22=NotApplicable I-9.37=Holds violations=[]
OPUS_K4M tree=tombstone I-9.22=Holds I-9.37=NotApplicable violations=[]
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
```
checker 单测 32 条、坏镜像 253 条、容器树 8 条（含旋钮 1 与旋钮 2 两条慢档随机历史）全绿；中间那片少一条的三张（`k4-middle`）在正确的判法下三棵都 Violated（K1-2 原样），在这一改坏下三棵都 Holds。原因：第 57 批 H1 的会红用例都是「两片各 1 条、按旋钮 2 判」，两片时「不是最右的」只有第一片；写侧早关那一行（`row 3848`）让每片都少一条，第一片也少，同样分不出只判第一片（推的：没叠两份改动跑）。

四句：
1. 分不分辨批次：分辨——H1 是第 57 批的改法，它的会红用例都是两片形。
2. 判别它的东西 checker 当时看得到吗：看得到（每片记录数与叶序位置）。
3. 满足的分句：K4「还有哪几种改坏照绿」。
4. 改法在这格上中不中：第 57 批 H1 的验收（攻方那一形三棵树各红、写侧早关下随机历史红、合法旋钮历史不红）在这一格上都不中（量过：上面全绿）。我提的改法：把 `k4_middle_short_container` 那一形（写 [2, 1, 1]、按旋钮 2 判）做成三棵树各一条坏镜像用例——量过：正确判法三棵 Violated、这一改坏三棵 Holds（只在我的模型上量过、被攻过零轮）。GPU 那一份同形的缺口（只判下标 0 的改坏在 GPU 两片对拍上照绿）是推的，GPU 没跑。

### 几个改法各修哪一格

| 格 | 改法 | 正确代码上 | 对应改坏下 |
|---|---|---|---|
| K3-2 映射条目数那一个次序 | 第三条用例：③ 那张（分配记录代 + 删映射一条），断言冷走读报分配记录 | 报分配记录（量过，`k3-probe.txt`） | `k3-cold-walk-mapping-count-first` 下报映射条目数（量过） |
| K3-3 标签 13 | H4 那张的「记账条目数对不上」改用删一行（⑥） | 报分配记录（量过） | `row 3850` 冷走读报记账条目数、`row 3851` 可写挂载报记账条目数（量过）；`k3-tag-judged`、`k3-count-skips-unregistered-tag` 下照报分配记录、不红（量过） |
| K4 H1 只判第一片 | 三棵树各一条 [2, 1, 1] 按旋钮 2 判的坏镜像 | 三棵 Violated（量过，`k4-middle.txt`） | `h1-first-only` 下三棵 Holds（量过） |
| K4 GPU 同形 | GPU 对拍补一条三片、中间少一条的流 | 推的 | 推的 |

GPU H1 那 3 行原文与替换文相同的核法（冻结副本的变异表）：`grep -c` 那一整段「文件 TAB 原文 TAB 替换文」，原样输出：
```
3
```

## 没打中的形状

| 格 | 试过的形状 | 取样 |
|---|---|---|
| K1-1 误报 | 旋钮 ≥ 2 的随机历史：四类删、挂着回退（含换实例之后回退到旧实例的版）、换实例重挂、根长高（节点 2 / 3）、三棵树旋钮各异、只一棵树装旋钮 | 40 个种子 × 120 / 150 步，判 5700 次，新判按步累计判到 60 793 片；harness 旋钮 2 慢档 1 条 |
| K1-2 漏报 | 两片早关、多一条、三片中间少一条，三棵树 | 第 57 批用例 + 25 行变异；`k4-middle` 三张 |
| K1-3 | 循环界、比较、空树出口、三棵树共用 | 读码，GPU 没跑 |
| K2 | 单测与整镜像同函数；整镜像调用点交空表 | 默认档 5 条红、慢档 2 条红；25 行变异全红 |
| K3-1 | 只坏一处的三张、H4 那张、H4 两行变异 | 冷走读与可写挂载各判一次 |
| K3-4 | 重建次序 | `row 3851`；映射条目数那一边读码 |

## 这条腿自己的限度
- 崩溃恢复下的旋钮 ≥ 2 那一形没造：上一轮攻方按那一轮的代码数过，一次发布的写落在同一段（20–30 个写），最小窗口全量约 10⁶ 态、估时以小时计，按定义「一段历史自己就超过约 10⁶ 的，那段不跑」；这一轮原型跑过的层 0 状态 0 个（离 10⁶ / 10⁷ 的上限都远）。这一句的数是上一轮的，没在这一轮的代码上重数。
- K1-1 只在 CPU 上量；GPU 旋钮 ≥ 2 合法流一条都没有，GPU 那一支的误报只靠读码。`one` 取法里只装退役旋钮的 10 个种子里 7 个新判一片都没判到（退役记录要叶删空才有），末版退役树多于一片的只有 `all` 取法 7 个种子与 `one` 取法 3 个种子（26、27、38）。
- 随机历史与 K4 三张都在内存稀疏盘上自己建的池（`probe-crate/tests/common_b57/mod.rs`，由上一轮攻方模型的 `common_b56` 改名而来）；K3 的五张借用 harness 那份用例文件自己的造法函数（拷进副本、接在末尾）。
- 改坏与改法都只在副本上量；副本上的数不算入库装置上的数。我提的三个改法（K3-2 第三条用例、K3-3 换造法、K4 三片坏镜像）只在我的模型上量过、被攻过零轮，没写进任何被判文件。
- 读码推的几句：K1-3 全部、K3-4 重建里映射条目数挪不到分配记录之前、K4 写侧早关那一行分不出只判第一片、GPU 同形缺口。
- 编译与测试经 `run-with-memory-cap.sh`，上限 4G（派发给的上限 10G；第一次用 10G 时包装在 slice 里排队，钩子报本机 slice 只放得下约 5 GiB，改用 4G），线程 10。开工时 `ps` 看到别的会话在跑 `cargo test --offline --release -p singlefs-checker-tier --features verdict-store,gpu --lib --tests --no-fail-fast -- --test-threads 31`（经 `gpu-quiesce-run.sh`），没有性能测量在跑。
- 有三条前台命令越过 240 秒的超时，被工具改放到后台（第一次 base 用例全跑、K1 四批合起来那一次、交回前复跑的第 10 段）；我没有起轮询，等完成通知之后读的产物。之后把每次前台调用拆短。

## 没做什么
- 不判正推那几格，不替主 agent 采纳；没读禁读清单里的三样（另一条腿的报告与模型、主 agent 的核对稿）。
- 没跑 checker 档（GPU 对拍、dispatch 钉值单测、坏盘战役）、门禁阶段、变异整表；第 57 批变异表的 GPU 4 行没跑。
- 没造：崩溃恢复下的旋钮 ≥ 2 一形；GPU 上三片中间少一条的流；重建里把数映射条目挪前的改坏（推的挪不动）。
- 没改冻结副本与主工作区的任何被判文件；写的只有这份报告与模型目录。

## 交回前的复跑
模型目录定稿之后（开头那张 sha256 表），在空目录 `~/.local/share/singlefs/work/m3-batch57-code-r1-opus/run3/` 里从头跑全部阶段（`SINGLEFS_SCRIPTS=/home/fy5090/code/singlefs/scripts OPUS_THREADS=10 OPUS_MEMORY_CAP=4G`，`SINGLEFS_REPO` 取缺省的冻结副本；前台按阶段名分 12 次起，其中第 10 次越过 240 秒被改放到后台、等完成通知后接着起），compare 的原样输出：
```
一致 base-build.txt
一致 base-fast.txt
一致 base-slow.txt
一致 k1-random-b1.txt
一致 k1-random-b2.txt
一致 k1-random-b3.txt
一致 k1-random-b4.txt
一致 k3-probe.txt
一致 k4-middle.txt
一致 mut-h1-first-only.txt
一致 mut-k2-image-path-skips-product.txt
一致 mut-k3-cold-walk-mapping-count-first.txt
一致 mut-k3-count-skips-unregistered-tag.txt
一致 mut-k3-tag-judged.txt
一致 mut-row3850.txt
一致 mut-row3851.txt
一致 row-3821.txt
一致 row-3822.txt
一致 row-3823.txt
一致 row-3824.txt
一致 row-3825.txt
一致 row-3826.txt
一致 row-3827.txt
一致 row-3828.txt
一致 row-3830.txt
一致 row-3831.txt
一致 row-3832.txt
一致 row-3833.txt
一致 row-3834.txt
一致 row-3835.txt
一致 row-3836.txt
一致 row-3837.txt
一致 row-3839.txt
一致 row-3840.txt
一致 row-3841.txt
一致 row-3842.txt
一致 row-3843.txt
一致 row-3844.txt
一致 row-3845.txt
一致 row-3846.txt
一致 row-3848.txt
compare：41 份，不一致 0 份
```

副本与编译目录交回前删了（base、run1、run2、run3 各自的仓副本与 target）；留下的只有草稿目录里的 `run3/out/`（这一次复跑的 41 份输出与 compare）、`notes/`（探索时的日志）与 `model/`（模型目录定稿前的草稿，与仓里那份逐字节相同）。
