# m3-formal-code-r1 云端攻方（Opus）报告（2026-10-02）

<!-- doc-lint:not-numbers M1 M2 M3 M4 K1 K2 K3 K4 G1 C1 C2 C3 C4 C5 C6 C7 P0 P1 P2 P3 P4 P5 J1 J6 J6Qn S5 S6 V06 R3-10 -->

腿：云端攻方（Opus）。攻击面：正文第四节分工表「云端攻方」一行，K1–K4，重点 K1、K2。代码按冻结副本 `/tmp/claude-1000/m3-formal-code-r1/tree/` 拷出来的副本读、改、跑（清单 `research/prompts/m3-formal-code-r1-snapshot/crates-sha256.txt` 231 行在拷贝上 `sha256sum -c --quiet` 无输出、退出 0）。下文所有数都是副本上的数，不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 一句话 | 量过 / 推的 |
|---|---|---|---|
| K1-a 胶水与改前逐条相同（结局、读盘次数与次序、读不出与自证不过的槽怎么跳） | 没打中 | 改前写法照原文重写进原型，随机根环 5 万态、随机扫环 5 万态，结局与读盘序列逐条相同 | 量过 |
| K1-b 胶水摊择新键这一步谁罩（C650「改坏择根序」那一族的胶水一侧） | **打中** | 胶水把择新键两项摊反（实例代号为主）：harness 档 103 个测试二进制 742 条全绿、singlefs-core 库 141 条全绿；Verus 与合判推论看不见这一步 | 量过（checker 档没跑，推的） |
| K1-c Vec 收齐再建表的内存形状 | 量出形状 | 峰值 ≈ 盘数 × 改前（每条记录写到每块盘）；默认环是每盘 196 608 槽，不是 1024 槽 | 1024 / 8192 槽量过，196 608 槽是线性外推（推的） |
| K1-d 两个键完全相同的槽取哪个 | 半中（代码内自相矛盾，可达性没造出） | 写死在核（spec 与可执行判定）里；但 `newest_root_among` 文档写「与 choose_root 同一个择法」，实际 `max_by_key` 取后者 | `max_by_key` 取后者量过；两处在同一池上分叉没造出 |
| K2-a M3 不动点前件与 D13 已定项 1 M3 一行字面 | 打中（已知形状，独立复现） | 字面只有「在飞上限」一条前提；核要 I-8.12 才成立：只满足字面前提的 88 万态里 4 492 态不动点不成立，加上 I-8.12 的 91 万态 0 反例 | 量过 |
| K2-b 合判推论会不会空真 | 没打中 | 真调两个核、把交回值原样交给推论，Verus 57 verified 0 errors；还证得出那一处所选槽是 2 | 量过 |
| K2-c 必红表 99 行 2 → 3 的理由 | 理由站得住，但 M 钉的是求解上下文 | 在 10-3 之前的核上只加一个没人用的 spec fn，99 行就从 2 变 3；多出的那条就是在飞上限那条循环不变式 | 量过 |
| K2-d 必红表 125 行 2 → 3 的理由 | 站得住 | 多出的一条落在 `choose_root_slot` 的循环不变式上；合判推论在这一世界里不红 | 量过 |
| K3 变异行锚点 | 没打中；背景一句过时 | 这一轮相关的 48 行锚点都恰好命中一次；第 173、174、177 行在冻结副本上也恰好命中一次（背景说 0 命中，那是旧表） | 量过 |
| K4 装置 | 没攻 | 只读了一处：实现的扫环次序取 `device_identities()`，装置里「照 device_identities() 次序」那一栏代表实现 | 推的 |

## 复跑

工作目录与写法照下面。D 是草稿目录，M 是模型目录。

```bash
D=/tmp/claude-1000/m3-formal-code-r1-attack; M=/home/fy5090/code/singlefs/research/prompts/m3-formal-code-r1-opus-model
rsync -a --exclude target --exclude .git /tmp/claude-1000/m3-formal-code-r1/tree/ $D/tree/; cp -r /home/fy5090/code/singlefs/litmus $D/tree/litmus
cp $M/prototypes/attack_glue_differential.rs $M/prototypes/attack_root_choice_txg_beats_instance_through_the_glue.rs $D/tree/crates/singlefs-harness/tests/
mkdir -p $D/tree/crates/singlefs-replay-prefix/tests; cp $M/prototypes/attack_fixed_point_brute.rs $D/tree/crates/singlefs-replay-prefix/tests/
cd $D/tree
# K1-a（约 95 秒）与 K1-c（约 35 秒）
ATTACK_CASES=50000 CARGO_TARGET_DIR=$D/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh 5 \
  cargo test --offline -p singlefs-harness --test attack_glue_differential -- --test-threads=2 --nocapture glue_matches
CARGO_TARGET_DIR=$D/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 1500M bash research/scripts/capped.sh 5 \
  cargo test --offline -p singlefs-harness --test attack_glue_differential -- --test-threads=1 --nocapture --exact scan_journal_heap_peak_before_and_after_on_full_mirrored_rings
# K2-a（3 秒）
ATTACK_CASES=3000000 CARGO_TARGET_DIR=$D/target nice -n 19 bash research/scripts/run-with-memory-cap.sh auto bash research/scripts/capped.sh 5 \
  cargo test --offline --release -p singlefs-replay-prefix --test attack_fixed_point_brute -- --nocapture
# K1-b：打 G1 补丁，编整包 harness 测试，逐个二进制跑（脚本按预算分批，跑过的跳过），跑完退回补丁
patch -p1 < $M/probes/g1-glue-swap.patch
CARGO_TARGET_DIR=$D/target nice -n 19 bash research/scripts/capped.sh 5 cargo test --offline -p singlefs-harness --no-run > $D/build-g1.log 2>&1
nice -n 19 bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh 5 bash $M/tools/run-harness.sh $D/build-g1.log $D/g1 210   # 重复到不再出 budget-stop
CARGO_TARGET_DIR=$D/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 1500M bash research/scripts/capped.sh 5 cargo test --offline -p singlefs-core --lib
CARGO_TARGET_DIR=$D/target nice -n 19 bash research/scripts/run-with-memory-cap.sh auto bash research/scripts/capped.sh 5 \
  cargo test --offline -p singlefs-harness --test attack_root_choice_txg_beats_instance_through_the_glue   # G1 下红，退回补丁后绿
patch -R -p1 < $M/probes/g1-glue-swap.patch
# K2-b、K2-c、K2-d：Verus 单文件（verus1.sh 里经 run-with-memory-cap.sh 1500M、capped.sh 5，--no-cheating --rlimit 10）
mkdir -p $D/verus/base && cp -r crates/singlefs-replay-prefix $D/verus/base/
$M/tools/verus1.sh $D/verus/base/singlefs-replay-prefix $D/verus/base.out
python3 $M/tools/mut.py crates/singlefs-replay-prefix/verus-mutations.tsv 99 $D/verus/base/singlefs-replay-prefix $D/verus/r99 && $M/tools/verus1.sh $D/verus/r99 $D/verus/r99.out
#   改前的核：/tmp/claude-1000/m3-formal-10-3/base/lib.rs（sha256 b013f3b2…）放进 $D/verus/pre103/src/，表用同目录的 verus-mutations.tsv
#   无关改动：python3 $M/tools/edit2.py <目标> <源> <0..7>；见证：cd $D/verus && cp -r base/singlefs-replay-prefix witness1 && patch -p0 … < $M/probes/witness1.patch
# max_by_key 平局
rustc -O $M/prototypes/maxby_main.rs -o $D/maxby/maxby && bash research/scripts/run-with-memory-cap.sh auto $D/maxby/maxby
```

`research/scripts/…` 指仓里的脚本（`/home/fy5090/code/singlefs/research/scripts/`）。草稿里实跑时所有命令另加了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 与 `RUN_WITH_MEMORY_CAP_WAIT_SECONDS`（整机内存排队，见「这条腿自己的限度」）。

## 模型目录每个文件的 sha256

`research/prompts/m3-formal-code-r1-opus-model/` 下 66 个文件（`find . -type f | sort | xargs sha256sum`，在模型目录里跑）：

```
45e6dc024153e7db32528bbba3cf25834cc23d042f8d915cae8bf2302ce11bcf  ./outputs/fix-g1.log
9aeb58bd2aea0e221200fb54be078494f12e0cd9c314f4061944575b73bc8678  ./outputs/fix-unmutated.log
0043fd97fd89654c6ce0933f267cfe56a0164dbd15505b058e1dfdc19f981455  ./outputs/fp-brute2.log
ccfca78b17fa6b69715025efaffb56d955a3877e1de5132ab9f21f23e4491f49  ./outputs/fp-brute.log
6ed7b1a15d27701cc4780cea0b4f4c339e17afa953b8b59cac556ce01b877df2  ./outputs/g1-batch1.txt
c002a5ac35629105e2e4cfbf296e50a108f8b7bd89752d824dfda8ab3d4ebacc  ./outputs/g1-batch2.txt
e20a51b1af845ed35832a3c58963f525dbbb833ca200e67fbdcd07639af82f0f  ./outputs/g1-batch3.txt
1e202250ca02a77bda5066be0828e42d71621e7edff0ef59c5fa032510288a34  ./outputs/g1-batch4.txt
de0cccc36d8cf901acd22ef8c92959a0dbdbfd37f52226561054416c292c1d01  ./outputs/g1-batch5.txt
8c1ce0811c60ee062c0b820a15c63578c176939c814659a00527c0625ef42270  ./outputs/g1-batch6.txt
7ed1b2b28e0c76838a9d65f5ff4d5770f81eeb83a0f701c7054221bfd8391bb2  ./outputs/g1-core-lib.log
0219cfe95241d5754f49cd3a2cb2ee4744611fc5cc087347b49c5b2d25b11567  ./outputs/g1/rc.txt
3feb125e7586de2643ab887d01c7aab38ab53689b105cbfab9f5037b2ecc51f9  ./outputs/g1/test-results.txt
3553a7d03df5e779e712d04e371caf55b80f4a687b1a5f328f262332ed532f30  ./outputs/maxby.out
7eb37dd02e912af008fc66065046998cd9a61d5970988d43bc358f1598a0cb17  ./outputs/run-50k.log
e351ad80e5d5d18bfa5992da60eff95ba422bff6ec937a035e9833b85746f79b  ./outputs/run-heap.log
13e60e8651acc62363b1f709eb93bdcc86e73c4aafb3a54687fe782daa59349f  ./outputs/verus/base.out
49444c6d37c9b2ba2142f90be2a3ba0019de7f3ca165fcd1dd804b32c4de1ce2  ./outputs/verus/pre103-r125.out
143ed20f420cfe744e853fbf9ddb67fc893495317a452eb8a1da58c1c5671abb  ./outputs/verus/pre103-r99-dummy-proof.out
4c73bc139e447595f210f5a8fa1c409919ae46e6b5c70a1f2ae63bbd7fde40ce  ./outputs/verus/pre103-r99-dummy-spec.out
d219fca13c3878eaec1029d0968a00ba0540fd861ede7bbca294aa4a01068b90  ./outputs/verus/pre103-r99.out
5784b3e0894cb214632976cc003c00017e0e82ac2d3b2d637505e0b6267d1577  ./outputs/verus/pre103-r99-p0.out
a8290942b37aa42a8b114c1e095f25aaa6235dc4c675b8990c6fe076f36bff76  ./outputs/verus/pre103-r99-p1.out
382898e8b0722f2ca3c7078f374f9d7c01b9955ae6b708e6c26328770e364bfc  ./outputs/verus/pre103-r99-p2.out
a814be7b36dad3b2fc605ceb8ad5729de1d50373d5baf8cc73fb6165b0d9037f  ./outputs/verus/pre103-r99-p3.out
536cc6eafac3f490f98b88b3164fa9d87af6f2b9471f759ef129582a1f890a92  ./outputs/verus/pre103-r99-p4.out
c771d614ae6608b987940f5823738f5ca585c9f80c0de8414bd6e6dc89009de1  ./outputs/verus/pre103-r99-p5.out
c771d614ae6608b987940f5823738f5ca585c9f80c0de8414bd6e6dc89009de1  ./outputs/verus/pre103-r99-p6.out
730a8a801eff63f0b5719249ea7963c124016896cb649363565d7a2a31eea614  ./outputs/verus/pre103-r99-p7.out
622755bd01bfe5c34debad86700ebcc79070c0a1d3d4238dab78dc06527c74dd  ./outputs/verus/pre103-r99-shift.out
6287bf1ad0164a4e8cd270e4664205ba407d749b8df2ae71e8e1c09b2a565476  ./outputs/verus/r125.out
30363811b0762dda51d2f8c045bd7a799fd899ee216c45ddffd54b6689bde09b  ./outputs/verus/r99-dummy-proof.out
ac9175c29fbe901647862c6cb1d438453b9fece7257d5b98fce53b7c6f3c9843  ./outputs/verus/r99-dummy-spec.out
ba6be6a683e7a87c30cf2a6999831b08372b26fb7e6910018dd413c8c6e66db4  ./outputs/verus/r99.out
c565e64b1f28daae904897e9d244549a255dd478becac215cad98f7f4b62d93a  ./outputs/verus/r99-p0.out
a2426a26ad594bdf149327c70050674a20ccc6aed8e2113418071f6fb25bb1ba  ./outputs/verus/r99-p1.out
da0398c6650ff6bf131ca846cef673e66871559e22a4b5fdc2b91dcd6562c5dc  ./outputs/verus/r99-p2.out
c565e64b1f28daae904897e9d244549a255dd478becac215cad98f7f4b62d93a  ./outputs/verus/r99-p3.out
ac9175c29fbe901647862c6cb1d438453b9fece7257d5b98fce53b7c6f3c9843  ./outputs/verus/r99-p4.out
ac9175c29fbe901647862c6cb1d438453b9fece7257d5b98fce53b7c6f3c9843  ./outputs/verus/r99-p5.out
3fc7b247b28b4905ddfdc78f1919b367322faadff2f36e5c904dfb60d327ec68  ./outputs/verus/r99-p6.out
8ee44eec72fd4c5f32ff4d97552ace3195f74fd775d505d568ba3d7124783aed  ./outputs/verus/r99-p7.out
107bf6859b57755967383c614720475979f784527d4b35937baad14b89656d86  ./outputs/verus/r99-rl20.out
107bf6859b57755967383c614720475979f784527d4b35937baad14b89656d86  ./outputs/verus/r99-rl5.out
b5c2d6293e39d81643ba9890c023b25e8b14a1fcc68b1153527599fdf56f7f66  ./outputs/verus/r99-shift.out
2fba1ac3364fa833e3ebd095cd5a1d8c2afae1233b0baed19808a1765fda801c  ./outputs/verus/r99-t1.out
107bf6859b57755967383c614720475979f784527d4b35937baad14b89656d86  ./outputs/verus/r99-t2.out
107bf6859b57755967383c614720475979f784527d4b35937baad14b89656d86  ./outputs/verus/r99-t5.out
bfcbbbea26d660b8b1841e3ed9d04d420e2c77142795c04d188ac83db599cedb  ./outputs/verus-summary.txt
737625ef7ec48519d5bf150173615ca0ed678e7df56080845b2c8edb0612a204  ./outputs/verus/witness1.out
0e3d0ed37ae6ddbf36078db1f416cfb18113dc2465f1a0451a686c157ae89901  ./outputs/verus/witness2.out
40b8519c0ba9502a9ba9cf2569f092fee6494492b469172e7fdae3e9657ffb62  ./outputs/verus/witness3.out
fe5464c3e3c279e3e842064e3a25d726dd4f5c4a811b8c60b8468921ba86cb75  ./outputs/verus/witness.out
250baa37e250e146075d02ae5f47c21976c1c1afca9a4c66280c1a2b2b63b2ab  ./probes/g1-glue-swap.patch
e3d61aa955871ba68d63ad972e541a5c027f13123a0ea13e1fd413be680de58c  ./probes/witness1.patch
1d774405838fab3ae745a42e006b358c6496a8b6ac677cb56ede2bef4d440853  ./probes/witness2.patch
748b7a83dac09e22b12c3aae95cdd5b95636c7098af3aa80a1e07c8b15bda857  ./probes/witness3.patch
866e34e0401667ce3096be2998c7ed10b13ef775cbdbc83384f17df1f5e43715  ./prototypes/attack_fixed_point_brute.rs
9a2d96ef80877cab07284029108459fac6c88dbfb47540098f25ff2d1be4ae9e  ./prototypes/attack_glue_differential.rs
9c66d797c11a63f8bb524a9b67b42fd22ba26974e6c84fdde881b0757eaaacba  ./prototypes/attack_root_choice_txg_beats_instance_through_the_glue.rs
4bef740c82e1e7641d0a6c402b4e4dcbfda99324044f14f4e09e512d6197b817  ./prototypes/maxby_main.rs
9579fa9d849a69a1f23e7b7d90dfafef79985e0081d624afa1ddc8f955cba28b  ./tools/edit2.py
e6ebff2eef403fb3585885cf13c49237391c63a1f1602ce24f8df3624e05e061  ./tools/edit.py
d902ea2801f1a8372d23fe7423521de60f3c1b0d373e4f9e4e489fa703f7ac9a  ./tools/mut.py
5e335283d95c1477ca80647a0f56f15d94b475093bf08e4a44074ba8d63cfb53  ./tools/run-harness.sh
9b2400dac5243b2f2e32d81fc5fd24edb8730b0da832c820970b499c570c8776  ./tools/verus1.sh
```

## K1　C3 胶水与核对不对得上

### K1-a　改前改后逐条比（没打中）

原型 `prototypes/attack_glue_differential.rs` 里把提交 fcc4d340 的两段写法照原文重写成 `choose_root_before`（逐槽 `read_root_ring_slot`、`(checkpoint_txg, 实例代号)` 严格大于才换）与 `scan_journal_before`（`entry(...).or_insert(record)`），读者外面包一层记下每次 `read` 的 (盘, 偏移, 长度)、指定的 (盘, 偏移) 交 None。两边用同一份镜像、同一组读不出的槽，比结局（`Option<RootRecord>` / 整张 `BTreeMap`）与读盘序列。

- 择根：只做过 mkfs 的池，根环 24 槽（3 区 × 8）。每槽 4/10 写一条随机键的根（txg 0..3、实例 0..2、树 ID 水位 0..2，所以常有键相同内容不同的两槽）、1/10 写校验和坏的根、1/10 写全 0、其余不动；每槽 1/8 读不出。
- 扫环：1–4 块盘（6 个设备号里随机挑、随机排 `device_identities()` 次序），环 16 槽；每槽 1/2 写一条随机键的记录（实例 0..2、计数器 1..6、提交标记与事务号随机，所以同键多份内容不同很常见），其中 1/10 弄坏一个字节；每槽 1/10 读不出；提示（`journal_record_offsets_hint`）一半给一半不给。

原样输出：

- 「name=choose-root-differential cases=50000 same=50000 cases_with_a_key_written_twice_with_different_watermarks=43066」（`research/prompts/m3-formal-code-r1-opus-model/outputs/run-50k.log:5`）
- 「name=scan-journal-differential cases=50000 same=50000 keys_with_two_or_more_different_self_verified_copies=212921」（`research/prompts/m3-formal-code-r1-opus-model/outputs/run-50k.log:8`）

推翻条件：改前写法在别的输入形状上（根环槽数、区域归属表不同的几何、读者在同一偏移上两次交回不同字节）与改后不同。这一次没扫几何，读者是确定的。

### K1-b　胶水摊择新键这一步没有罩（打中）

被判对象里许可这一步的两句：

- 胶水把根摊成择新键：「checkpoint_txg: PrefixKernelCheckpointTxg(root.checkpoint_txg.0),」（`crates/singlefs-core/src/recovery.rs:1278`）与「instance: PrefixKernelInstanceGeneration(root.instance.0),」（`crates/singlefs-core/src/recovery.rs:1279`）。核只看它拿到的两项，择新序在核里证（`choose_root_slot` 的 ensures）；这两项是不是根的那两项，Verus 看不见。
- 合判推论交代不在 Verus 里的只有调重放那一步：「（recovery.rs 的 recover、mount.rs 与 mounted_read.rs 的读阶段）不在 Verus 里。」（`crates/singlefs-replay-prefix/src/lib.rs:2980`）；择根核那一节写的是「读盘、自证校验和、fsid 由胶水（recovery.rs 的 choose_root）做，不在 Verus 里。」（`crates/singlefs-replay-prefix/src/lib.rs:950`），都没提摊键。

变异 G1（`probes/g1-glue-swap.patch`）：摊键时两项对调，`checkpoint_txg` 取 `u64::from(root.instance.0)`、`instance` 取 `u32::try_from(root.checkpoint_txg.0).unwrap_or(u32::MAX)`。效果就是「实例代号为主、txg 破平局」，正是 C650 点名要红的那一族：「改坏择根序（同 `checkpoint_txg` 平局取低、实例代号为主这类）这一族要红」（`.claude/kb/checks-owed.md:519`）。核那一侧的同一族在 `crates/mutations.tsv` 有行，「择根核以实例代号为主、txg 只破平局（C650 改坏择根序：实例代号为主）」（`crates/mutations.tsv:1826`），点名的是 `-p singlefs-replay-prefix --lib` 的单测，那条单测直接调核、不经胶水；胶水那一侧只有「择根胶水摊择新键时不带实例代号」（`crates/mutations.tsv:1827`）一行。

G1 下的结局（副本上，原样）：

- harness 档整包 104 个测试二进制，除我的原型之外 103 个：`binaries=103 passed=742 failed=0 ignored=64`（`outputs/g1/test-results.txt` 去掉原型那一行之后用 awk 求和；逐个二进制的退出码在 `outputs/g1/rc.txt`）。原型那一个红：「attack_glue_differential-2466d8aadc031d7c rc=101」（`research/prompts/m3-formal-code-r1-opus-model/outputs/g1/rc.txt:16`）。
- `cargo test -p singlefs-core --lib`：「test result: ok. 141 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s」（`research/prompts/m3-formal-code-r1-opus-model/outputs/g1-core-lib.log:151`）。
- 专为这一格写的用例（前一个空槽写实例 1、txg = mkfs + 2，后一个空槽写实例 2、txg = mkfs + 1，经 `choose_root` 择）：不打补丁绿「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s」（`research/prompts/m3-formal-code-r1-opus-model/outputs/fix-unmutated.log:10`），打 G1 红「assertion `left == right` failed: txg 为主：txg 大的实例 1 那条赢过实例代号高、txg 小的实例 2 那条」（`research/prompts/m3-formal-code-r1-opus-model/outputs/fix-g1.log:15`），择到的是 `instance: InstanceGeneration(2), checkpoint_txg: CheckpointTxg(1)` 那条。

为什么现有用例都不红（推的，按读代码）：合法历史里新实例第一次发布取 max(根环, 记录) + 1，所以实例代号高的根 txg 也高，两种序在合法历史的每一对根上同号；只做过 mkfs 再手写根的那条用例 `root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot` 用的是同 txg 平局，平局时实例代号为主与 txg 为主取同一条。两种序只在「实例代号更高而 txg 更低」的根同在时分叉，这种态只有造出来或带故障才有。checker 档（层 0、崩溃放量 CPU 判器）在合法崩溃状态上同理看不出（推的，没跑）。

打中之后的四句：

1. 分不分辨臂：这一轮的「臂」是「C650 的 Verus 一半罩住改坏择根序」这一句对不对。G1 在核、合判推论与必红表全部原样的情况下改坏了产品的择根序，分辨的正是这一句。
2. 系统当时看不看得到判别它的东西：`choose_root` 读到了两条根的全部字段，看得到；判别它的状态（实例代号更高而 txg 更低）在合法历史里不出现（推的）。
3. 满足的是判据字面的哪一个分句：C650 判据栏「改坏择根序（…实例代号为主这类）这一族要红」那一分句，胶水一侧。核一侧的同一族是红的（第 1826 行），不归这一格。
4. 跑前条款给的改法在这一格上中不中：正文第五节只给「打中 ⇒ 改法进下一批代码」，没给具体改法；10-3 报告与推论注释给的罩法（层 0 与崩溃放量 CPU 判器）在这一格上推的不中（同第 2 句），没量。

### K1-c　Vec 收齐再建表的内存形状（量出形状）

先改正背景里的一个数：默认环是「pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;」（`crates/singlefs-format/src/lib.rs:220`），805306368 ÷ 4096 = 196 608 槽一块盘（`python3 -c` 算的）；1024 是环的起点槽号「pub const JOURNAL_RING_START_SLOT: u64 = 1024;」（`crates/singlefs-format/src/lib.rs:216`），不是环长。

每条记录写到池里每一块盘：「CommitStep::WriteJournalRecordToEveryDevice { counter, record } => {」（`crates/singlefs-core/src/transaction.rs:290`）。所以环满时读到的份数 = 盘数 × 不同键数，改前的 `BTreeMap` 只留不同键，改后的 Vec 先装下每一份：「let mut scanned_records_in_read_order: Vec<JournalRecord> = Vec::new();」（`crates/singlefs-core/src/recovery.rs:3031`）。

量法：测试二进制里装计数分配器，读者现编字节（环满、每块盘每槽都是计数器 = 槽号 + 1 的那条），量 `scan_journal_before` 与 `scan_journal` 各自调用期间的堆峰值。`size_of::<JournalRecord>()` 200、`size_of::<NamedUnit>()` 64。原样（每条记录 67 个点名项，一条记录装得下的最多）：

- 「name=heap named=67 ring_slots=8192 devices=1 records_kept=8192 before_peak_bytes=38484828 after_peak_bytes=40581488 ratio=1.05」（`research/prompts/m3-formal-code-r1-opus-model/outputs/run-heap.log:17`）
- 「name=heap named=67 ring_slots=8192 devices=2 records_kept=8192 before_peak_bytes=38489120 after_peak_bytes=77478256 ratio=2.01」（`research/prompts/m3-formal-code-r1-opus-model/outputs/run-heap.log:18`）
- 「name=heap named=67 ring_slots=8192 devices=4 records_kept=8192 before_peak_bytes=38489128 after_peak_bytes=151271792 ratio=3.93」（`research/prompts/m3-formal-code-r1-opus-model/outputs/run-heap.log:19`）

1024 槽与 8192 槽两档的峰值比是 8.0（同一份输出第 14、17 行），按线性外推到默认环 196 608 槽（推的，没在这个尺寸上量）：改前约 923.6 MB；改后两块盘约 1 859.5 MB、四块盘约 3 630.5 MB。不带点名项时改前约 80.6 MB、改后四块盘约 258.3 MB。改前那 0.9 GB 本身也是 `scan_journal` 把整环留在内存里的形状，不是这一轮带进来的；这一轮带进来的是乘上盘数。

### K1-d　两个键完全相同的槽取哪个（代码内自相矛盾，可达性没造出）

写死在哪：核的 spec 写明「（条款没写键相同取哪一个，这一句照 recovery.rs 原来「严格大于才换」的行为写）。」——引号套引号，cite-check 认不出，行号是 `crates/singlefs-replay-prefix/src/lib.rs:975`（`grep -nF` 现查）；可执行判定严格大于才换。改谁会红：必红表第 149、157 行（Verus），`crates/mutations.tsv` 第 1822 行（库单测）。胶水只能靠倒过槽序改它。

矛盾：同一个文件里另一处择新文档说与 `choose_root` 同一个择法，「读出来的根里 (checkpoint_txg, 实例代号) 最大的那一条（与 [`choose_root`] 同一个择法）；一条都没有时 `None`。」（`crates/singlefs-core/src/recovery.rs:1796`），实现是「.max_by_key(|root| (root.checkpoint_txg, root.instance))」（`crates/singlefs-core/src/recovery.rs:1801`）。`max_by_key` 键相同时交回后一个：原型 `prototypes/maxby_main.rs` 跑出「max_by_key_picks=slot2」（`research/prompts/m3-formal-code-r1-opus-model/outputs/maxby.out:2`）。`newest_root_among` 给回退下界那一路（`effective_rollback_floor` 一族）取「最新那条根的实例表」；可写挂载另一路取的是「最新那条根（`choose_root` 现择）指着的实例表」（`crates/singlefs-core/src/mount.rs:1225`）。键完全相同、实例表指针不同的两条根同在时，两路认的「最新那条根」不是同一条。

可达性：没造出。合法历史里每次发布的 (txg, 实例代号) 不重；失败处置「原样重发」写出的是同一份内容。所以只在造出来的态或带校验和碰撞的坏盘上分叉（推的）。不分辨 10-3 改动的对错（10-3 照原 `choose_root` 行为写，没错），分辨的是「条款没写、代码里两处各写一种」。

## K2　C2、C3 的证明在证什么

### K2-a　M3 不动点的前件与条款字面不是同一句（打中；形状已知，这里在核上独立复现）

条款 M3 一行给不动点的前提只有一条：「前提是所选根之上的记录不超在飞上限」（`.claude/kb/decisions/13-验证路线.md:33`）。核的 spec 把三样写成前件：「// 前提照第二轮判决第 6 行辩方那一句：I-8.12；同一个读者；所选根之上的记录条数不超在飞上限。」（`crates/singlefs-replay-prefix/src/lib.rs:841`），而 I-8.12 核不判：「// 核不判它：记录级的第二半与不动点把它写成前提（核拿到的记录表满足它）。」（`crates/singlefs-replay-prefix/src/lib.rs:684`）。「同一个读者」在条款里是设定（同一个读者再重放一次），对得上；多出来的是 I-8.12。

多出来的这一条是不是性质本身要的（不只是证明要的）：原型 `prototypes/attack_fixed_point_brute.rs` 用编进产品的 `replay_core`（稳定版 rustc 编出来的那一份，不是 spec）在随机小表上照条款字面跑两次：所选根 (实例 1 或 2, txg 3)，两个实例各 7 个计数器里随机留，txg 2..6、末条标志、提交标记、序号随机，在飞上限 1..6，两个回调按下标确定地交答案（同一个读者），一半的表把每个实例的 txg 按计数器排成不减（I-8.12 成立）；第一次 Ok 且所选根之上的候选不超上限才算条款的前提成立；第二次以落点为所选根、同一张表、同一对回调、同一个开关、同一个上限再调。原样：

- 「name=fixed-point-brute cases=3000000 first_ok=2631573 premise_cases=1790678 premise_cases_without_i812=881122 violated_with_i812=0 violated_without_i812=4492」（`research/prompts/m3-formal-code-r1-opus-model/outputs/fp-brute2.log:10`）
- 见证之一（第一趟，表全随机）：「name=fixed-point-witness i812=false root=(1,3) limit=5 first_applied=1 landed=(1,4)」（`research/prompts/m3-formal-code-r1-opus-model/outputs/fp-brute.log:7`）。同实例记录 (1,2) txg 4 末条、(1,3) txg 6、(1,4) txg 4 末条：第一次施加计数器 2 那次发布落到 txg 4；第二次以 (1, txg 4) 为所选根，那次发布里带末条标志的有计数器 2 与 4 两条，报「锚点认哪一条条款没写」的错，不是「一条没施加、根不变」。

所以：只满足条款字面前提的 881 122 态里 4 492 态不动点不成立，全部出在 I-8.12 不成立的表上；I-8.12 成立的 909 556 态 0 反例（与 Verus 的证明相合）。

条款的域是 E165 的合法历史一族（M1–M4 的前提与域那一段）。I-8.12 在那一族上成不成立我没量；按写侧推，同一实例里后写的记录 txg 不小（推的）。另有两处与背景材料不同的现状：

- I-8.12 已经登进 `invariants.md`，状态未实现：「同一块盘 journal 环里自证过的记录，同一实例代号的记录按 jsn 计数器从小到大，`checkpoint_txg` 不减。」（`.claude/kb/invariants.md:101`）。背景材料第二节 C2 一行说「还没登进」，是旧的。
- 登进去的那条按「同一块盘」判，核要的是 `scan_journal` 先到先得、跨盘拼出来的那张表上成立。两者之间的桥（每块盘各自不减 ⇒ 拼出来的表不减）要「同一 (实例代号, 计数器) 只对应一份内容」才推得出；E165 的 R 域恰是不带这一前提的合法历史（「M2 字面为假，前提承重」，`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:172`）。在 R 域那种重发形状上桥断不断，我按写侧次序推的是不断（同一键的两份 txg 都小于下一个计数器的 txg），没量。

同形的见证 E165 第 2 次重跑对抗一族已经报过（三个 J6Qn 红例都违反 I-8.12，`research/prompts/e165-r3-prereg.md` 第 435 行把 Q3 按 I-8.12 拆两行）；这一格不新，新的只是直接在核上、按条款字面前提量出的比例。

四句：分辨的是「条款 M3 一行的字面」与「核证的那一句」；系统看得到判别它的东西（I-8.12 只看表就判得出，核选择不判）；满足的是 M3 一行「不动点（…；前提是所选根之上的记录不超在飞上限）」那一分句；跑前条款没给改法。可选的两种改法见文末改法表。

### K2-b　合判推论不空真（没打中）

10-3 的空真探针是在推论体头插 `assert(false)`，证不出就算前提不矛盾。我加一处更强的见证：在核文件里加一个可执行函数，真调 `choose_root_slot`（三个槽：txg 3 实例 1、空、txg 3 实例 2）与 `replay_core`（空记录表、回调带 ensures），把两者交回的东西原样交给 `corollary_recovery_replays_from_the_newest_self_verified_root`。原样：

- 「witness1.out proof_errors=0 verification results:: 57 verified, 0 errors」（`research/prompts/m3-formal-code-r1-opus-model/outputs/verus-summary.txt:33`）：推论的 requires 在一处可达的调用点上被两个核的 ensures 推出来了。
- 「witness2.out proof_errors=0 verification results:: 57 verified, 0 errors」（`research/prompts/m3-formal-code-r1-opus-model/outputs/verus-summary.txt:34`）：同一处还证得出所选槽是 2（平局实例代号高者赢），择根 spec 在具体输入上把选择钉死了。
- 「witness3.out proof_errors=1 verification results:: 56 verified, 1 errors」（`research/prompts/m3-formal-code-r1-opus-model/outputs/verus-summary.txt:35`）：不写 ensures 的闭包，「同一个读者」既证不出成立也不默认为假（报错的是 `assert(!the_reader_answers_each_position_one_way(bc))`）；写了 ensures 的那一个证得出成立。产品胶水里的两个回调在 Verus 之外、没有 ensures，M3 的两条对它们是有条件的，这是 C649 那一笔，不是空真。

推论新推出来的只有「落点不早于任一自证过的根」这一项（其余六项是 requires 原样转写）；它说的是 `records` 这一张任意的表，与 `first_copy_of_each_record_key` 的 ensures 没有接上，扫环核不在合判里（推论注释也只写了择根与取前缀）。

### K2-c　必红表 99 行（V06）2 → 3：理由站得住，M 钉的是求解上下文

被判的那一行：「改判定·在飞上限不看（已定项 18；攻方 V06）」（`crates/singlefs-replay-prefix/verus-mutations.tsv:99`），第五段 `proof_errors=3`。副本上量了四组（Verus 0.2026.09.20，`--no-cheating --rlimit 10`，`verus1.sh`）：

| 核 | 改动 | M | 出处（`outputs/verus-summary.txt` 或同目录 `.out`） |
|---|---|---|---|
| 今天的核 | 只有 99 行那一处 | 3 | 第 18 行 |
| 今天的核 | 线程 1、2、5；rlimit 5、20 | 都是 3 | `r99-t1.out`、`r99-t2.out`、`r99-t5.out`、`r99-rl5.out`、`r99-rl20.out` |
| 今天的核 | 另加 8 种没人用的定义之一（7 个 spec fn、1 个 struct）、加 proof fn、文件头加 10 行注释 | 都是 3 | `r99-p0.out` … `r99-p7.out`、`r99-dummy-*.out`、`r99-shift.out` |
| 10-3 之前的核（`/tmp/claude-1000/m3-formal-10-3/base/lib.rs`） | 只有 99 行那一处 | 2 | 第 5 行 |
| 10-3 之前的核 | 另加一个没人用的 spec fn（7 种写法每一种） | 3 | 第 6 行（`pre103-r99-p0.out`），`p1`–`p6` 与 `dummy-spec` 同 |
| 10-3 之前的核 | 另加一个没人用的 struct、或 proof fn、或 10 行注释 | 2 | 第 13 行（`p7`）、第 3 行（`dummy-proof`）、`pre103-r99-shift.out` |

多出来的那一条是哪一条：今天的核上三条报错都在 `replay_core` 里，落在源码第 3720、3847、3899 行；第 3720 行是「admitted.len() <= in_flight_limit,」（`crates/singlefs-replay-prefix/src/lib.rs:3720`），也就是在飞上限那条循环不变式本身。10-3 之前的核上只报后两条（那份文件的第 3514、3566 行），加一个没人用的 spec fn 之后第三条（那份文件的第 3387 行，同一条循环不变式）出现。

结论：10-3 报告的推测「新加的 spec 进了 SMT 上下文，求解器多报了一处」量得出来——任何一个新 spec fn 都能让它从 2 变 3，与择根、扫环无关。3 比 2 更贴题（多出的正是被改坏的那条不变式）。代价是这一行的 M 钉的是「文件里有几个 spec 定义」这类与变异无关的东西；今天的核上 8 种无关改动都没再动它，下一次结构性的改动会不会再动没量。门禁 must-red-proof-error-count 按 M 逐条比，这一行再漂时红的是门禁，不是证明（推的）。

### K2-d　必红表 125 行（S5 严格小于）2 → 3：站得住

被判的那一行：「M3·spec·S5 版本序：实例代号破平局改成严格小于」（`crates/singlefs-replay-prefix/verus-mutations.tsv:125`）。今天的核上 3 条（`verus-summary.txt` 第 15 行），落在源码第 2707 行（`lemma_replay_core_result_lands_no_earlier_than_the_chosen_root` 的 postcondition）、第 3600 行（`replay_core` 的 postcondition）、第 3986 行（`choose_root_slot` 的循环不变式）；10-3 之前的核 2 条（第 2 行），没有择根核那一条。理由「择根核共用 S5 序」对上。

顺带：这一世界里合判推论不报错。严格序下所选槽对自己不满足「不新于」，推论的 requires 在有所选槽时不可满足，推论在这个世界里空真（推的，没在这个世界里插 `assert(false)`）。空真格只在不改坏的核上判，所以不归门禁；只说明推论的「不空真」靠择新序自反，核里没有一条冒烟引理钉自反。

## K3　C1、C6 的变异行与用例

- 锚点：`crates/mutations.tsv` 里名字带 R2-4、R3-9、R3-1、R3-10、C648、H08、H09、择根、扫环、E165 r2、XE1–XE3、E170 的 48 行，原文（`\n` 当换行）在各自文件里逐行数命中：`rows_checked 48 not_exactly_once 0`（草稿里的 python 一段，冻结副本上跑）。
- 背景说第 173、174、177 行「改前就 0 命中」。冻结副本上这三行都恰好命中一次（同一段 python：`line=173 hits=1`、`line=174 hits=1`、`line=177 hits=1`）；10-3 报告里的 0 命中是对 10-3 快照里那一版表算的，那三行的原文后来换过（10-3 快照与冻结副本里这三行第三段不同，`diff` 现查）。不归这一轮，只更正现状。
- C648 那两种形状（S03、S07 在健康镜像上读缓存命中不红、补写全零的用例）没攻。

## K4　C4、C5 装置

没攻。只读了一处：实现 `scan_journal` 按 `reader.device_identities()` 的次序逐盘读（`crates/singlefs-core/src/recovery.rs` 第 3032 行，`grep -nF` 现查），所以 E165 第四段反盘序里「照 `device_identities()` 次序」那一栏代表实现；「盘号升序」那一栏代表的是另一种先到先得。产品里这个次序是挂载时交进来的盘表次序（`mount.rs` 的 `device_identities_in_table_order`），那一处会不会拒换序的盘表，没查到底。

## 改法表（全部只在我的模型上量过或推的，被攻过零轮）

| 改法 | 修哪一格 | 在打中的那几格上 | 量过 / 推的 |
|---|---|---|---|
| A　加 harness 用例经 `choose_root` 钉「txg 为主」：前一个槽 txg 高实例低、后一个槽 txg 低实例高（`prototypes/attack_root_choice_txg_beats_instance_through_the_glue.rs`），`crates/mutations.tsv` 加 G1 一行点名它 | K1-b | 不打补丁绿、打 G1 红（`outputs/fix-unmutated.log`、`outputs/fix-g1.log`）；同一条用例对第 1826 行（核一侧实例代号为主）也该红 | 量过（G1 那一格）；第 1826 行那一格推的 |
| B　推论上方注释与 D13 已定项 1 M1 那一格「核之外」那句，把「摊择新键」列进不在 Verus 里的一截 | K1-b 的说法 | 不改行为，只让不罩的那一截有字 | 推的 |
| C　扫环胶水第一遍只收 (键, 盘, 偏移)，问过核之后只重读留下的那几份 | K1-c | 峰值回到约「不同键数 × 每份」；代价是留下的每份多读一次盘，读盘次数用例（C648 一族）要跟着改 | 推的 |
| D　扫环核改成逐条问（「这个键见过没」），胶水里边读边建表 | K1-c | 峰值同改前；先到先得只证得到一步，证不到整趟（10-3 报告第七节已写） | 推的 |
| E　`newest_root_among` 改成经 `choose_root_slot` 择，或把文档改成「键相同取后一个」 | K1-d | 两路在键完全相同时认同一条根，或至少文档说真话 | 推的 |
| F　D13 已定项 1 M3 一行的不动点前提补「I-8.12（按 `scan_journal` 交出的表判）」 | K2-a | 条款字面与核证的句子一致；4 492 个反例全部落在补上的前提之外 | 推的（反例分布量过） |
| G　核在同实例 txg 下降处断链，不动点不再要 I-8.12 | K2-a | 改了取前缀的行为，要重过 M2、M4 与必红表全套 | 推的 |
| H　99 行不改；表上注明这一行的 M 随文件里的 spec 定义个数漂过（7-2 一次 3 → 2、10-3 一次 2 → 3） | K2-c | 只让下次漂的时候有出处 | 推的 |

## 没打中的形状

- 胶水与改前不同的输入：随机根环 5 万态（24 槽、键 txg 0..3 × 实例 0..2、坏校验和、全 0、1/8 读不出）、随机扫环 5 万态（1–4 块盘随机次序、16 槽、键实例 0..2 × 计数器 1..6、内容随机、1/10 坏字节、1/10 读不出、有无提示各半）：结局与读盘序列 0 处不同。没扫几何（区域归属表、每区槽数、根槽宽）。
- `choose_root` 新加的 `.expect` 那一条 panic 路：核的 ensures 担保它到不了；5 万态里没触发，没再造。
- 合判推论空真：可达调用点上的见证 0 errors；`assert(false)` 那一种 10-3 已探过，我没重跑。
- 必红表 99 行在今天的核上换线程（1、2、5）、换 rlimit（5、20）、加 9 种无关改动：M 都是 3。
- 键完全相同、内容不同的两条根在合法历史里同在：没造出来（按写侧推不可达）。
- 没试的：改择新序时 spec（`version_is_not_later_than`）与核可执行判定一起改成同一个错序，会不会整份照样验过（spec 是可信件，这一种只能由单测与用例罩，没量）；胶水先滤掉 None 再交核、下标对不上的那种写法（推的：现有用例会红，没跑）。

## 这条腿自己的限度

- 全部数都在冻结副本的拷贝上量；checker 档（层 0、崩溃放量、E165、E170、变异整表、门禁）一样没跑。G1 下 harness 档默认忽略的 64 条没跑（没带 `--include-ignored`）。
- 原型没建崩溃历史、没调层 0 枚举函数，`closed_form_state_count` 累计 0 个状态；用的是 harness 的 `MemoryPool` 与自写的读者，不是 `SparseBlockDevice`。
- 扫环内存只在 1024 与 8192 槽两档上量，默认环的数是线性外推；计数分配器量的是调用期间堆峰值，不含栈与读者自己的临时字节。
- 不动点暴力取样只覆盖 2 个实例 × 7 个计数器、txg 2..6、在飞上限 1..6；没覆盖 txg 顶格那一支。
- Verus 计数只在这一台机器、这一个发布上量；`--num-threads` 与 rlimit 只扫了上表那几个点。
- 内存包装没跑成的几次（命令一行都没跑，都按包装给的出路重跑过）：扫环峰值第一次 252（整机排不上，`4G`）；不动点暴力第一次 251（上限超过峰值表 2 倍，改 `auto`）；G1 第 4 批第一次 252（同会话已有一条在跑，加并行理由）；`singlefs-core --lib` 第一次 252（整机排不上，改 `1500M`）；G1 下那条针对用例第一次 251（改 `auto`）；`maxby` 两次（252 排不上、251 超峰值）。
- K3、K4 只做了上面写的那几处。

## 没做什么

- 没判正推腿那几格，没替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑 `verus-proof-check.py`、门禁与任何重型测试；Verus 只在自己的拷贝上单文件起。
- 没改仓里任何文件；只写了这份报告与模型目录。
