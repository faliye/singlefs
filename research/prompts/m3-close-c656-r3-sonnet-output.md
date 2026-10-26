# m3-close-c656-r3 辩方腿报告（云端辩方，Sonnet，2026-10-20）

轮名 m3-close-c656-r3，最终轮。格 S3、S4 为主，兼 S1（`research/prompts/_m3-close-c656-r3-body.md` 第三、四节）。立场：复核前两轮判决 `research/prompts/m3-close-c656-r1-main-verification.md`、`research/prompts/m3-close-c656-r2-main-verification.md` 与用户定的合成形态（乙两趟写序 + 强制新 fsid）够不够得着、是不是同样打中替代方案；替被排掉的「拒」「认作已知形态」两支辩护；把 S3 的条文逐句写出。这一轮新冒出的形态写进「零轮」一节，不开下一轮。

数的标记：【量】= 本腿这一次在草稿目录的仓副本上跑出（原样在模型目录 `research/prompts/m3-close-c656-r3-sonnet-model/out/`）；【量·B】= 同，在第二份树 B 上；【r2量】= 前两轮腿产物里现读的数，本腿没重跑；【推】= 读码或推出，没跑。没标的是引文。

## 复跑命令与哈希

模型目录 `research/prompts/m3-close-c656-r3-sonnet-model/`：探针 `probe/`（一个 crate，加进仓副本工作区）、脚本 `tools/`、树 A 的参照输出 `out/`、树 B 的参照输出 `out-live/`。冻结副本 = 仓里 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 的一份拷贝：树 A 在 `~/.local/share/singlefs/work/m3-close-c656-r3-sonnet/frozen/`，树 B 在同目录 `frozen-live/`（交回前不删，复跑要它）。

```
# 树 A（参照 out/；甲七四段全量机器空闲时约 10 分钟，忙时翻几倍）
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=21600 bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-c656-r3-sonnet-model/tools/rerun.sh --repo . --frozen ~/.local/share/singlefs/work/m3-close-c656-r3-sonnet/frozen --work <新的草稿目录>
# 树 B（参照 out-live/，不带甲七四段）
RERUN_SKIP_UNITCLEAR=1 RERUN_OUT_DIR=research/prompts/m3-close-c656-r3-sonnet-model/out-live RUN_WITH_MEMORY_CAP_WAIT_SECONDS=21600 bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-c656-r3-sonnet-model/tools/rerun.sh --repo . --frozen ~/.local/share/singlefs/work/m3-close-c656-r3-sonnet/frozen-live --work <新的草稿目录>
```

可选环境变量：`RERUN_TARGET_DIR`（编译目录，默认 `<草稿目录>/target`，从头编要编 `librocksdb-sys`，内存紧时指到别处已编好的）、`RERUN_OUT_DIR`、`RERUN_RECORD=1`（不比，把这一趟记成参照）、`RERUN_SKIP_UNITCLEAR=1`。每项打一行 `name=same item=<项>`，不同打 `name=differs`；只比 `name=` 行、去掉 `name=done`（带挂钟）与 `name=example`。本腿整条跑过：树 A 的参照由一趟 `RERUN_RECORD=1` 记下，再经上面的命令（带 `RERUN_SKIP_UNITCLEAR=1`）复跑 28 项全 `same`、退 0；树 B 同法 30 项（含 2 项 `alpha-`）全 `same`、退 0；甲七四段在另一趟独立的跑里逐行相同（树 A，上一版脚本，`name=sig` 键逐段比）。`--selftest` 自证过（5 格）。线程 31、每次起探针经 4G 包装、整条经 8G 包装。

各文件 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里）：

```
983781c79c9bc9cfabd085adc15f2cb417a1371aedf382c9f510a37cca35979a  out/final-distinct-today.log
1c06ca0c7e028ee877609d595493b8c08a2525a6467f2690561706c2d4dc7d5e  out/final-distinct-yi.log
a11e39c730ef0bbe0a9419d7c05cfd3b19a30ad8c8f5701f7fb7427a9307754c  out/final-distinct-yi-unitclear.log
78c2b9027053b60045521ae71597543c66f39591f6b218ba946ba3751002d432  out/final-edge8-today.log
8614a5c508138a71dabededb19adb32369ea0bc92426a6151a29ea4fb43a5711  out/final-edge8-yi.log
b63d568710e94f896f5e420f426ce7c1a1a7dd9558e654d125af1eccce3b84e7  out/final-edge8-yi-unitclear.log
8a26b892498ad668c999e0495d719b28943c532668bd598bb162490b2946cf6a  out/final-prefix-today.log
d55d71d213bb16f10db985c51c9ff80bb46e6c787adc613f0301324f67c6fead  out/final-prefix-yi.log
ea2b1c00ba2c5a87f10d1b095721297c616a61a6baf9fe7998b96702d0d9403c  out/final-prefix-yi-unitclear.log
37f7b2ec6d3654bf740a02e4f0fe25c4e4fa0d1e9b468dc7a8ce21630803099b  out/final-same-today.log
c75c689a60f7ed6f8ad480688b89709361de5a9325037e99d6847bb5562d836e  out/final-same-yi.log
0a0be8bbbcbca5d79b594352d97e76a60329b04fff3268346435c6df0f6038a3  out/final-same-yi-unitclear.log
2ff0c117055c3ead01fd9f6827662a7aebf032bfd7569935dd419e12cf4d77b1  out/guardcheck-distinct.log
7de765506aff93eebc18e7f737fcf7b5418b285cf0c965578b1fa2e64b2004ee  out/guardcheck-prefix.log
f6e654498cc38985f2a3a1cf5fa97ad5b52d5f049ab6dfa61d32343e5a161cd4  out/guardcheck-same.log
b235e1ab77d2153a99ac188aa033f9a7fe5abb04b7a0ea85d5feaa3b89c96f7c  out/lag-4.log
85aa347447859eee1d3ba590f489377c83ac14c80ba710172e5e1639281e234a  out/lag-acquire-4.log
8c7d7787b8369bfd16e1858d638aed6cec76989cd41c648429ebb5c37b6efc80  out/lifecycle-prefix-3-6.log
cf1601ad8556ebe9235a3f82656db3f4e6ff4f0780b9987c2ea8904cdfd59413  out/lifecycle-same-0-3.log
3040369216e6a0475b71d360a5aa0851c07197a5a60a57fd1f4cabee2d5e15f0  out/lifecycle-same-3-6.log
8718bca53b8604ff4e095f2ad69a06efc13d433af61b9636cc2f1cb78dbddd1b  out/lifecycle-use-distinct.log
c0d60c72a4eb85106974b1b80eb4fad3822615a03dcf31ebdb5fd3192d379ef0  out/lifecycle-use-prefix.log
e49725678152d8dcbebca94898210e20bfffc4c3532febd04eb45b2b17d77b83  out/lifecycle-use-same.log
d8c969cd42d977e42524f5607ccef450159f5995307e868061d19b31f9e03ca3  out-live/alpha-distinct-yi.log
3ecb3ac1c6a1cae4aecd326303879cadf21dc59b558595960467206d9d0bb097  out-live/alpha-same-yi.log
983781c79c9bc9cfabd085adc15f2cb417a1371aedf382c9f510a37cca35979a  out-live/final-distinct-today.log
1c06ca0c7e028ee877609d595493b8c08a2525a6467f2690561706c2d4dc7d5e  out-live/final-distinct-yi.log
a11e39c730ef0bbe0a9419d7c05cfd3b19a30ad8c8f5701f7fb7427a9307754c  out-live/final-distinct-yi-unitclear.log
78c2b9027053b60045521ae71597543c66f39591f6b218ba946ba3751002d432  out-live/final-edge8-today.log
8614a5c508138a71dabededb19adb32369ea0bc92426a6151a29ea4fb43a5711  out-live/final-edge8-yi.log
b63d568710e94f896f5e420f426ce7c1a1a7dd9558e654d125af1eccce3b84e7  out-live/final-edge8-yi-unitclear.log
8a26b892498ad668c999e0495d719b28943c532668bd598bb162490b2946cf6a  out-live/final-prefix-today.log
d55d71d213bb16f10db985c51c9ff80bb46e6c787adc613f0301324f67c6fead  out-live/final-prefix-yi.log
ea2b1c00ba2c5a87f10d1b095721297c616a61a6baf9fe7998b96702d0d9403c  out-live/final-prefix-yi-unitclear.log
37f7b2ec6d3654bf740a02e4f0fe25c4e4fa0d1e9b468dc7a8ce21630803099b  out-live/final-same-today.log
c75c689a60f7ed6f8ad480688b89709361de5a9325037e99d6847bb5562d836e  out-live/final-same-yi.log
0a0be8bbbcbca5d79b594352d97e76a60329b04fff3268346435c6df0f6038a3  out-live/final-same-yi-unitclear.log
2ff0c117055c3ead01fd9f6827662a7aebf032bfd7569935dd419e12cf4d77b1  out-live/guardcheck-distinct.log
7de765506aff93eebc18e7f737fcf7b5418b285cf0c965578b1fa2e64b2004ee  out-live/guardcheck-prefix.log
f6e654498cc38985f2a3a1cf5fa97ad5b52d5f049ab6dfa61d32343e5a161cd4  out-live/guardcheck-same.log
e2e5930b4182586b9904bb5512837614ab30b7de2842d2daaec59a6f86989b34  out-live/lag-4.log
aa34f179a6b2f4385e7e6108f448874b6623baa40446cfe9eed483abe3c25c39  out-live/lag-acquire-4.log
8c7d7787b8369bfd16e1858d638aed6cec76989cd41c648429ebb5c37b6efc80  out-live/lifecycle-prefix-3-6.log
cf1601ad8556ebe9235a3f82656db3f4e6ff4f0780b9987c2ea8904cdfd59413  out-live/lifecycle-same-0-3.log
3040369216e6a0475b71d360a5aa0851c07197a5a60a57fd1f4cabee2d5e15f0  out-live/lifecycle-same-3-6.log
8718bca53b8604ff4e095f2ad69a06efc13d433af61b9636cc2f1cb78dbddd1b  out-live/lifecycle-use-distinct.log
c0d60c72a4eb85106974b1b80eb4fad3822615a03dcf31ebdb5fd3192d379ef0  out-live/lifecycle-use-prefix.log
e49725678152d8dcbebca94898210e20bfffc4c3532febd04eb45b2b17d77b83  out-live/lifecycle-use-same.log
3ff70faed3931e9a9128499ac9c43d1ce2ca615331ad0059ef42d4718d2101cf  out-live/run-distinct-yi.log
b61d0f965f3cb982b453fb925d88ed879a5f8440cd32fb68df6e4aee8b650524  out-live/run-prefix-yi.log
971ce6646cf2a89b0d8de0c1850fe9204fcea154bca85a90446904f9edf37754  out-live/run-same-yi.log
3bea6756da0bfcb2922b73cc038cd1408a562a7a9a7e6c6d29d47f1b7779e1fe  out-live/streams-distinct.log
4dbb3502b2998b6812075a8818b562eabe423ba7b5b505d683b9e74a1ed72316  out-live/streams-same.log
5d804e0db8d985023b924f8ccd4e44e87587ab32d074639d29c4b28dc0816eab  out/run-distinct-yi.log
d8bf6f4d259c464d18c4d12c8070aef695f8657b05d66562eae31703f8ec3cca  out/run-prefix-yi.log
3e4b5553a64035c5c8195bdd20262d0a53de7ebbd2b3175f76afeba7d5361649  out/run-same-yi.log
0f728b358994c9a32012bef369dfc7323e232c3d58fdc8c5beacd70fe3ac365c  out/run-same-yi-unitclear-0-70000.log
c26e66b8d813de960f2a6e72e3b4ed886f2d1721f6600d33bdb5d771b939a380  out/run-same-yi-unitclear-140000-210000.log
9dfd5e7db14b4895d90b8dd09238d28b4b5a0092f527f3415089c4a04056d823  out/run-same-yi-unitclear-210000-280027.log
770b8a17c5186183848d4c9056592a5e8a13e1500dfeb6aa71cbdcde9495f52f  out/run-same-yi-unitclear-70000-140000.log
3bea6756da0bfcb2922b73cc038cd1408a562a7a9a7e6c6d29d47f1b7779e1fe  out/streams-distinct.log
4dbb3502b2998b6812075a8818b562eabe423ba7b5b505d683b9e74a1ed72316  out/streams-same.log
dfb19ae1a6386ba253b8ac5d872b89fa6d5f8dad9d5b8d16cd59ae0201e5ba7a  probe/Cargo.toml
de6272dd3aa25e8d3b643120cb5e6775a96a84bbde6f6b5a7eabc52dbfc5297a  probe/src/main.rs
7014ab4bda7b34dbf49985754085b40c2c259dc3432a41dc4a370def974ffcaa  tools/add_probe_member.py
47f3c9f58b6462e2c4ad7a6ef58a62fff3ab16b126d3b635d2aea87e93a39e2a  tools/items.txt
788d820d899d5fcd1cfcb1b6d8534517a5c60a52f6beded09bb42a0005815821  tools/patch_copy.py
84acbdca2f22dcd92a1f4f4c1d6aba64a75e1069dde98f0346e276ca6be52de6  tools/rerun.sh
18d1e030bb284b5e1b5dadea48360a6d0688da21d3e8770857f9bce64a46be60  tools/summarize.py
```

## 零、先说清楚的三件事

1. **工作树在这一轮里动了，我量了两份。** 树 A = 开工时的 `crates/`（`.rs` 全部文件 sha256 再取 sha256：`3dc1ba5195ee541b85c7248f6ceeba3f82026f17a8ab6b382463688b9fc6ad8b`，与第二轮攻方冻结副本同值，即前两轮量的就是它）。树 B = 开工后另一个会话批 8（系统配置落地）的未提交改动进来之后的 `crates/`（同法取哈希：`9a136ba411d02a6a01fbac29769ab8bf43ddf6f91ac36043bebe334ca72ba63f`，快照时刻之后到交回前再比一次仍同值）。`diff -rq` 数出两树 48 项不同（9 项是只在 B 里有的新文件，含 `crates/singlefs-core/src/system_configuration_repair.rs` 与八个新用例）。`make_filesystem.rs` 两树逐字节相同。**两树在乙的崩溃态分类上一致，只在「可写挂载」与取号写上不同**，见第二节。
2. **入库 mkfs 一行没改。** 乙与甲七的写序不是改 mkfs 得到的，是把入库 `make_filesystem` 录下的 25 步流重排（乙 = 两盘各清 [0, 根环基址)、两道屏障、其余清零与四个单元写、两道屏障、根 FUA、系统配置槽、两道屏障；甲七 = 再在作废屏障之后加每盘一步单元区写零）。写的字节一个不变；仓副本里唯一的改动是崩溃点重放逐态展开函数的可见性 `pub(crate)` → `pub`（`tools/patch_copy.py`，对应 `crates/singlefs-checker-tier/src/crash.rs:1310` 「pub(crate) fn visit_crash_state_range_of_the_read_back_flow_with_slices(」那一行）。这比前两轮的「副本里改 mkfs」更接近入库代码，但**仍是副本**：入库 mkfs 今天不是乙的写序，数是在重排流上量的。
3. **「入库」在这里有三层意思，我分开写**：HEAD 提交（我没量）；工作树里已暂存的部分（树 A 的样子）；工作树里还没暂存的批 8 改动（树 B 比树 A 多的）。「`crates/` 今天的实现」我两份都量，哪一格依赖哪一份在表里标 A / B。

## 一、判定一览（判决条目 / 辩方判定 / 证据）

| # | 判决条目 | 辩方判定 | 证据 |
|---|---|---|---|
| 1 | 第一轮 V1 / 第二轮 U1「甲出局」（收窄到 fsid 相同） | **站住（收窄后）；在合成形态里它的出局理由已不存在**：强制新 fsid 之后「fsid 相同」那一格不会出现，甲此后是被用户口径「守乙那句」排掉，不是被打中排掉 | 引理与 sysconf-first 数 `research/prompts/m3-close-c656-r2-sonnet-output.md`【r2量，没重量】；本腿没碰甲 |
| 2 | 乙站住（两轮、九份历史没打中） | **站住，且在入库 mkfs 的重排流上复核对上**：段序列 `2+12+1+1+1+4`、27 步、21 写、闭式 4117、带撕裂 31 195；作废之前 8 态全读回最后确认的一版，作废之后 31 122 态整池拒、65 态读回第 0 代空池，择旧池 0 态、混合 0 态（同 fsid、前 8 字节同、前 8 字节异三份历史，A、B 两树） | 第二节表【量】【量·B】 |
| 3 | 乙附带①「一盘作废一盘没作废：旧池只读挂得上、可写挂载拒」 | **A 上站住（8 态里 4 态）；B 上够不着**：B 上这 8 态可写挂载全做成（批 8 的补写闸把作废了的那块盘补回去） | 第二节【量】【量·B】 |
| 4 | 乙附带②「择系统配置取第一块可择盘那一份」 | 站住（读码，A、B 同） | `crates/singlefs-core/src/recovery.rs:1539` 「None => chosen = Some(best_on_device),」【推】 |
| 5 | 第二轮 U3 条文 X | **大体站住，三处措辞对不上读数**：「照没跑过 mkfs 处置」「读照读」「可写挂载在取号那一步拒」；最后一处在 B + 不同 fsid（即合成形态）上不成立 | 第三节 T2、T4【量】【量·B】 |
| 6 | 第二轮 U4 C659 射程「前 8 字节相同」 | **站住，并再收窄两处**：边界精确在第 8 / 9 字节（只第 8 字节不同做完绿）；红随新池取号次数自愈（老池系统配置的实例代号为 K，新池第 K 次挂载之后转绿） | 第二节 S4-3、S4-5【量】 |
| 7 | 第二轮 U4 乙七（I-7.7 ① 收窄）出局 | 本腿没复核 | 没做 |
| 8 | 第二轮 U4 甲七（作废屏障之后清单元区） | 站住：重排流上 280 027 态，作废之前 8 态读回对、之后择旧池 0 态、混合 0 态、65 个新池态 checker 全绿 | 第二节 S4-4【量】 |
| 9 | 第二轮 U4 丙七「读旧池拒」被乙的作废掏空 | **按守卫读什么分**：只读系统配置的守卫被掏空（31 122 / 31 122 放行）；读单元头的守卫够不着（旧 fsid 重跑 31 122 / 31 122 拒）；但读单元头的守卫把同 fsid 原样重试也拒了 29 178 / 31 122（自撞） | 第四节【量】 |
| 10 | 用户定「强制新 fsid」修 I-7.7 与屏障那一格 | 站住（I-7.7 一格再量对上；屏障一格是第二轮攻方手搭，我没重量） | 第二节 S4-3【量】；`research/prompts/m3-close-c656-r2-opus-output.md`【r2量】 |
| 11 | 零轮「跨盘世代号差大于 1 就拒」 | **不要**：入库取号在合法崩溃序列下造出差 2–4 的池，恢复读回、只读挂载、可写挂载、池级 checker 全正常；强制新 fsid 之后它也没有要抓的态 | 第三节 T10【量】【量·B】 |
| 12 | 「不管」一句（屏障只到一块盘归 C6；两块盘来自两个池不在射程） | 站住；本腿没重量这两形 | 第三节 T8、T9 |
| 13 | 被排掉的「拒」与「认作已知形态」 | 两支都有站得住的部分，见第四节：「认作已知形态」是强制新 fsid 的残余、不是它的对手；「拒」出局的真正理由是自撞，不是被掏空 | 第四节 |

## 二、S4：前两轮站住的读数，在入库代码上核得了多少

方法一句话：把入库 `make_filesystem` 在「用过的盘」上录下的 25 步流，按乙 / 甲七重排，再用入库的 `writes_and_segments`、`crash_state_count_with_torn_in_place_overwrites`、崩溃点重放逐态展开、`choose_system_configuration`、`recover`、`mount_read_only`、`mount_writable`、`check_pool_image` 判。旧池 = 入库 mkfs + 取号 + 暖机 + 第一个文件 + 3 次覆盖写（最后确认的一版是根 (实例 1, txg 6)），新池 io_min 512。三份历史：旧 fsid 与新 fsid 相同（`same`）、前 8 字节相同后 8 字节不同（`prefix`）、前 8 字节不同（`distinct`）；另加只有第 8 个字节不同的 `edge8`。

### S4-1　段序列与态数（A、B 两树逐字相同）

`research/prompts/m3-close-c656-r3-sonnet-model/out/streams-same.log` 原样（`distinct` 一份只差历史名，`diff` 无差）：
```
name=stream h=same order=today operations=25 writes=21 segment_sizes=14+1+1+1+4 closed_form_two_state=16402 states_with_torn=280019 final_image_equals_today=true kinds=[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
name=stream h=same order=yi operations=27 writes=21 segment_sizes=2+12+1+1+1+4 closed_form_two_state=4117 states_with_torn=31195 final_image_equals_today=true kinds=[zero_fill×2,barrier×2]|[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
name=stream h=same order=yi-unitclear operations=29 writes=23 segment_sizes=2+14+1+1+1+4 closed_form_two_state=16405 states_with_torn=280027 final_image_equals_today=false kinds=[zero_fill×2,barrier×2]|[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
name=stream h=same order=yi-unitclear-early operations=29 writes=23 segment_sizes=4+12+1+1+1+4 closed_form_two_state=4129 states_with_torn=31267 final_image_equals_today=false kinds=[zero_fill×4,barrier×2]|[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
```

- 乙：`2+12+1+1+1+4`、27 步、21 写、闭式 4117、带撕裂 31 195【量】【量·B】。与第一轮正推、两次攻方抽样、第二轮攻方的数同。
- 今天：`14+1+1+1+4`、25 步、16 402、280 019【量】【量·B】。第一轮判决只对上这两个数；这一次在重排流的同一套入库函数上对上。
- 甲七（作废屏障之后清单元区）280 027；甲七错排（与作废同段）31 267【量】【量·B】，与第二轮攻方一致。
- **做完的盘面**：`final_image_equals_today=true`（乙）——乙与今天的 mkfs 做完，盘面逐字节相同；甲七两种（`false`）多出整个单元区的零。所以任何「mkfs 做完之后」的读数，乙与今天共用，不是乙独有的。

### S4-2　乙的 31 195 态怎么分（三份历史相同，A、B 相同）

`research/prompts/m3-close-c656-r3-sonnet-model/out/run-{same,prefix,distinct}-yi.log`，`tools/summarize.py` 汇总（只列 `same`，A 树）：
```
== run-same-yi.log 总态数 31195  
     all new_complete                             1
    seg0 old_readback_intact                      8
    seg1 refused                                  31103
    seg2 refused                                  1
    seg3 refused                                  1
    seg4 refused                                  1
    seg5 new_complete                             64
    seg5 refused                                  16
   混合(MIXTURE) 0；作废之后(非 seg0)择到旧池 0
   new_complete 可写挂载 Err:SystemConfigurationUnreadableAfterTheReadStage:SystemConfigurationUnreadableAfterTheReadStage: 40
   new_complete 可写挂载 Ok: 25
```

- 8 + 31 122 + 65 = 31 195【量】：作废之前（第 0 段）8 态全部读回最后确认的一版 (1, 6)；作废之后（第 1–5 段）31 122 态全部整池拒（`NoValidSystemConfiguration`），65 态读回第 0 代空池 `NoFile(i0t0)`；作废之后择到旧池 0 态、新旧混合 0 态。`same`、`prefix`、`distinct` 三份历史、A、B 两树的这几个数全相同【量】【量·B】；`diff` 汇总键（去掉可写挂载与系统配置世代号）A、B 逐键相等【量·B】。
- 65 个新池态里池级 checker：`same`、`prefix` 是 25 红（I-7.7）+ 40 绿；`distinct` 是 0 红【量】。红的 25 态 = 做完那 1 态 + 两盘各至少一份新槽的 24 态，绿的 40 态 = 一块盘一份新槽都没有（I-7.7 在缺盘时报不适用）。
- **可写挂载（A、B 在这里分开）**：A 树 65 个新池态里 40 态拒（`SystemConfigurationUnreadableAfterTheReadStage`）、25 态做成；B 树 `same`、`prefix` 同 A，**B 树 `distinct`（合成形态）65 态全做成**【量】【量·B】。第 0 段 8 态：A 树 4 态做成 4 态拒，B 树 8 态全做成。原因在 B 树的补写闸：`crates/singlefs-core/src/mount.rs:5845` 「有写序实例大于所选那一版实例的单元就不补写（闸四，零轮改法，用户 2026-10-20 采）」，旧池同 8 字节前缀的单元挡住补写（`same`、`prefix` 仍拒 40 态），前缀不同则不挡。

### S4-3　mkfs 做完之后池级 checker 的 I-7.7（乙与今天的做完盘面逐字节相同）

单元头里的 fsid 只有 8 字节：「单元头里的 fsid 是 8 字节：系统配置 fsid 的低 8 字节」（`crates/singlefs-core/src/unit.rs:42`）。`final` 模式 = 旧池之上 mkfs 全落之后的池级 checker 红项（`out/final-*.log` 的 `raw_red`；A、B 两树逐字相同【量】【量·B】）：

| 旧池与新池的 fsid | 今天的写序 | 乙 | 甲七 |
|---|---|---|---|
| 相同 | 红 I-7.7 | 红 I-7.7 | 绿 |
| 前 8 字节相同、后 8 字节不同 | 红 I-7.7 | 红 I-7.7 | 绿 |
| 前 8 字节不同 | 绿 | 绿 | 绿 |
| 只有第 8 个字节不同 | 绿 | 绿 | 绿 |

- 第二轮判决的「fsid 前 8 字节不同时乙做完 I-7.7 绿」在入库 mkfs 上对上【量】；而且它不是乙独有的：今天的写序做完同样绿，因为做完盘面相同。
- 边界精确在第 8 / 9 字节之间：`edge8`（只有下标 7 的字节不同）绿，`prefix`（下标 8–15 不同）红【量】。强制新 fsid 的条文要写前 8 字节，不写 16 字节。

### S4-4　甲七（作废屏障之后清单元区）的 280 027 态

四段（0–70 000、70 000–140 000、140 000–210 000、210 000–280 027），`PROBE_RERUN=0`（不放开整池拒之后再跑 mkfs），`tools/summarize.py --merge` 汇总（A 树；重跑一遍，四段的 `name=sig` 键逐行相同，两趟独立的跑）：

```
== run-same-yi-unitclear-0-70000.log+run-same-yi-unitclear-70000-140000.log+run-same-yi-unitclear-140000-210000.log+run-same-yi-unitclear-210000-280027.log 总态数 280027  
     all new_complete                             1
    seg0 old_readback_intact                      8
    seg1 refused                                  279935
    seg2 refused                                  1
    seg3 refused                                  1
    seg4 refused                                  1
    seg5 new_complete                             64
    seg5 refused                                  16
   混合(MIXTURE) 0；作废之后(非 seg0)择到旧池 0
   new_complete 可写挂载 Err:SystemConfigurationUnreadableAfterTheReadStage:SystemConfigurationUnreadableAfterTheReadStage: 40
   new_complete 可写挂载 Ok: 25
   整池拒后再跑 mkfs fsid=new 守卫 g_sysconf=allow g_units_fixed=refuse g_units_all=refuse: 279954
   整池拒后再跑 mkfs fsid=new 结局 rerun_not_tried: 279954
```

- 8 + 279 954 + 65 = 280 027【量】：作废之前 8 态读回最后确认的一版；作废之后择旧池 0 态、混合 0 态；65 个新池态池级 checker 全绿（乙不清单元区时同一份历史是 25 红）。与第二轮攻方一致。
- 只数态数的：甲七错排（单元区清零与 [0, 根环基址) 同段）31 267 态（S4-1）；它的分类（第二轮攻方：80 态里 32 态读回坏）本腿没重跑。

### S4-5　I-7.7 的红会自愈，读者一直读对（支持「认作已知形态」）

`lifecycle` 模式：旧池取过号（旧池系统配置的实例代号 K），同 fsid 重做 mkfs，之后新池每挂载一次（取号 +1）看一次池级 checker 与恢复读回。A、B 两树相同【量】【量·B】。K = 4 那一趟（`out/lifecycle-same-3-6.log`）原样：

```
after_remake sysconf_instance=0 recover=NoFile(i0t0) raw_red=["I-7.7"]
after_new_mount1 sysconf_instance=1 recover=NoFile(i1t2) raw_red=["I-7.7"]
after_new_mount2 sysconf_instance=2 recover=NoFile(i2t4) raw_red=["I-7.7"]
after_new_mount3 sysconf_instance=3 recover=NoFile(i3t7) raw_red=["I-7.7"]
after_new_mount4 sysconf_instance=4 recover=NoFile(i4t10) raw_red=[]
after_new_mount5 sysconf_instance=5 recover=NoFile(i5t13) raw_red=[]
after_new_mount6 sysconf_instance=6 recover=NoFile(i6t16) raw_red=[]
```

- K = 1（`lifecycle-same-0-3.log`）：做完红，新池第 1 次挂载之后绿；K = 4：做完红、第 1–3 次挂载仍红、第 4 次起绿。**红到新池系统配置的实例代号追上旧池单元里最大的实例代号为止**，这正是 I-7.7 ① 条文的字面（「实例代号 ≤ 各盘系统配置实例代号的最大者」，`crates/singlefs-checker/invariants/02-根环.md:14`）；红的全程恢复读回都是新池的 `NoFile(i{k}t…)`，可写挂载都做成。
- 用起来之后：旧池之上重做 mkfs，再照旧池的用法用到第 5 版（取号、暖机、第一个文件、3 次覆盖写），池级 checker 无红项，恢复读回的是新池最后一版的字节（不是旧池的任何一版），与全新空盘上同样用法的对照组逐项相同（`same`、`prefix`、`distinct` 三份历史，`out/lifecycle-use-*.log`）【量】【量·B】。
- 没量：旧池实例代号很大（上千次挂载）时红持续多久只是同一个线性关系（推的）；新池有了文件之后旧池的同前缀单元被扫描重建当成现行版本的可能（本腿里 `raw_red` 一直空，I-1.13 没红；但新池只写了一个文件，推不出更多）。

### S4-6　批 8 的离线出路（α'）在乙的崩溃态上能不能把旧池请回来（只在树 B 上有这个模块）

第二轮辩方列过的开放项（`research/prompts/m3-close-c656-r2-sonnet-output.md` 没做什么一节：乙作废后与在飞的 α' 离线出路的交互，推的）。树 B 里 `crates/singlefs-core/src/system_configuration_repair.rs` 已有，本腿在乙的 31 195 态上对每态各试两次：断言旧池最后确认的一版（旧 fsid）、断言新池第 0 代（新 fsid）。`out-live/alpha-{same,distinct}-yi.log` 汇总【量·B】：

| 态的类 | 态数 | 断言旧池那一版 | 断言新池第 0 代 |
|---|---|---|---|
| 第 0 段（作废未持久） | 8 | 8 态都写成（补写 0–3 个槽），之后恢复读回 (1, 6) 最后确认的一版 | `same`：拒（`TailOfTheAssertedVersionUndecided…`）；`distinct`：拒（`SystemConfigurationNotChosen`） |
| 第 1–4 段与第 5 段整池拒的态 | 31 122 | **全拒**（`SystemConfigurationNotChosen`：盘上没有一份自证的本池系统配置，拿不到几何） | 全拒（同） |
| 读回空池的态 | 65 | `same` 拒（`AssertedVersionReadableNeither…`）、`distinct` 拒（`SystemConfigurationNotChosen`，该池 fsid 是新的） | 全拒（`TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion`） |

- 结论【量·B】：作废持久之后，α' 请不回旧池（31 122 / 31 122 拒在第一步：`crates/singlefs-core/src/system_configuration_repair.rs:217` 「1. 按调用方的本池 fsid 择系统配置（拿几何）。」没有系统配置可择）；作废持久之前它能把旧池补回来，与 T2「旧池读回最后确认的一版」一致。乙的「作废之后不挂旧池」没有被 α' 绕开。
- 副产：半截的新池（那 40 态，一块盘一份新槽都没有）α' 也救不了（第 0 代没有末条 journal 记录，第一版不支持）；能救它的只有 B 树挂载里的补写闸，而闸四对同前缀旧单元不放行（S4-2）。

### S4-7　核不动的与入库里没有的

- **乙的写序本身不在 `crates/` 里**：`crates/singlefs-core/src/make_filesystem.rs:559` 「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」今天还在第一段里与根环三区、journal 环的清零和四个单元写同段。重排流与「mkfs 自己按乙的次序发」之间的等价是推的：同一批写（`writes=21`）、做完盘面逐字节相同（S4-1【量】），差别只在发出次序与两道多出来的屏障；每盘屏障按设备记、录制器会并掉紧接的重复屏障（`crates/singlefs-harness/src/lib.rs` 的 `this_device_already_barriered_in_the_trailing_run_of_barriers`）这一条我没单独验。
- **入库崩溃点重放没有任何一项枚举 mkfs 自己**：`.claude/gate.d/crash-point-replay-items.tsv` 里非注释行 32 条（`grep -vc` 数的），含 `mkfs` 的三条都写「mkfs 后…」，从 mkfs 之后的池起枚举。C656 的验收格（「在用过的盘上枚举 mkfs 第一道屏障之前的崩溃态」（`.claude/kb/checks-owed.md:429`））在入库装置里没有对应的项；本腿的探针不在任何门禁里。乙落地时要新增一条流与观察者，代价没估【推】。
- 没复跑：屏障只到一块盘（第二轮攻方 181 398 528 态，手搭 12 态）、两块盘来自两个池（第二轮攻方 31 168 态）、今天写序下 280 019 态的分类（第一轮的 248 832 等拆分）、甲七错排的分类。真设备上 `write_zeroes_at` 的实际语义、O_DIRECT 后端写零的耗时（第二轮攻方按 200 MB/s 估 4 TB 盘约 5.6 小时，拍的速率）也没量。

## 三、S3：条文逐句，对着读数

条文来源：乙的两趟写序与附带两样（`research/prompts/m3-close-c656-r2-main-verification.md` 第二节 U2 行）、宽条文 X（第二辩方草稿：`research/prompts/m3-close-c656-r2-sonnet-output.md:169`）、「不管」一句与「跨盘世代号差大于 1 就拒」（`research/prompts/_m3-close-c656-r3-body.md` 第三节 S3 的后两条）、用户定的强制新 fsid。每句：条文原话 / 它改或它依赖的 kb 与代码行 / 读数 / 对不上的地方 / 什么现象推翻。

### T1　写序

- 条文：「mkfs 先把每块盘的 [0, 根环基址) 整段写 0，两块盘各过一道屏障（作废），之后才写别的。」（`research/prompts/m3-close-c656-r2-sonnet-output.md:169`）
- 它改的 kb 句：`.claude/kb/decisions/22-单元原子性怎么合成.md:178` 「清零几段之间不加屏障」要改成 [0, 根环基址) 清零之后每块盘过一道屏障、其余清零与单元写之间不加屏障；同一行「写任何元数据之前，先把每块盘的 [0, 根环基址) 整段写 0」已经有，不动；「每块盘 5 步、两块盘共 10 步」不变（乙的清零仍是 10 步、21 写，S4-1【量】）。
- 同一行还把段序列交给另一处：`.claude/kb/layout/01-first-txn.md:410` 「25 次操作（21 次写，两道池屏障按设备记、每道两块盘各一步）、16402 个崩溃状态」要改成 27 步、`2+12+1+1+1+4`、4117，种类串跟着变；`.claude/kb/layout/01-first-txn.md:162` 「mkfs 写元数据之前把每块盘 [0, 根环基址) 整段写 0」要加屏障；`.claude/kb/decisions/22-单元原子性怎么合成.md:180` 与 `:191` 的「没有条款」「C656（mkfs 中途崩溃的语义没有条款）」随之收掉。三语言各一份（\*.en.md、\*.ja.md）。
- 代码里写死这组数的断言 7 处、3 个文件（grep 数的）：`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:367`、`:371`；`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:280`、`:285`、`:289`、`:294`；`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2679`；另 `crates/` 之外的样本与装置：`scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs`、`scripts/crash/check-segment-registry-product-sample.txt`、`.claude/gate.d/checker-tier-qemu-device-streams.sh` 及它的 fixtures。mkfs 自己的注释 `crates/singlefs-core/src/make_filesystem.rs:540`（「几段之间不另加屏障」）与 `:544`（「不为一个没有条款的语义加屏障」）也要改。
- 读数：段序列、步数、闭式、态数 S4-1【量】【量·B】。对不上的：无。
- 什么现象推翻：入库 mkfs 按这句实现之后，「fn recorded_stream_matches_the_registered_mkfs_segment_sequence() {」（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:267`）录到的不是 `2+12+1+1+1+4`，或做完的盘面与今天不逐字节相同。

### T2　作废持久之前

- 条文：「作废持久之前崩溃：下一次挂载照没跑过 mkfs 处置，旧池读回最后确认的一版，或整池拒。」（`research/prompts/m3-close-c656-r2-sonnet-output.md:169`）
- 读数：第 0 段 8 态全部读回 (1, 6) 最后确认的一版，整池拒 0 态（三份历史、A、B）【量】【量·B】。
- **对不上①**：「照没跑过 mkfs 处置」A 树上不成立。没跑过 mkfs 时可写挂载做成；A 树第 0 段 8 态里 4 态可写挂载拒（一盘已作废），B 树 8 态做成但那是补写闸把作废了的盘补回来之后的事。读回「最后确认的一版」才是量得到的；可写挂载归附带①与现行补写条款，不并进「照没跑过」。
- **对不上②**：「或整池拒」在两树、三份历史共 6 份的第 0 段里一态都没有；它只在两块盘来自两个池的起点出现（T9）。留着不会被数据推翻，但这样这一句对同一个池的情形推不翻，建议限定成「两块盘是同一个池时读回最后确认的一版」。
- 什么现象推翻：第 0 段任一态读回的字节不是最后确认的一版。

### T3　作废持久之后

- 条文：「作废持久之后、最后一道屏障之前崩溃：下一次挂载只有两种结局——整池拒（`NoValidSystemConfiguration`），或读回 mkfs 的第 0 代空池（根为实例 0、txg 0，没有文件）」（`research/prompts/m3-close-c656-r2-sonnet-output.md:169`）
- 读数：31 122 态整池拒，择系统配置的错全部是 `NoValidSystemConfiguration`；65 态读回 `NoFile(i0t0)`（三份历史、A、B）【量】【量·B】。没有第三种结局。
- **对不上**：判决与正文第三节的简写「作废之后整池拒」不能进条文：65 / 31 195 的态不是拒，是读回空池。条文写 X 的「整池拒，或读回第 0 代空池」。
- 什么现象推翻：作废持久之后任一态是第三种（读回旧池某一版、新旧混合、别的错名）。

### T4　空池里缺一盘的新系统配置

- 条文：「后一种里系统配置的冗余可以缺，缺一块盘的新槽时那块盘按本条射程现行那一句处置（可写挂载在取号那一步拒，读照读）」（`research/prompts/m3-close-c656-r2-sonnet-output.md:169`）；现行那一句：`.claude/kb/decisions/22-单元原子性怎么合成.md:180` 「某块盘一份自证过的系统配置都没有时，取号那一步也拒」。
- 读数：A 树：那 40 态可写挂载拒（`SystemConfigurationUnreadableAfterTheReadStage`）【量】。**B 树 + 不同的前 8 字节（即合成形态）：40 态可写挂载做成**【量·B】；B 树 + 前 8 字节相同：仍拒 40 态。
- **对不上①**：括号里的「拒」在合成形态 + 批 8 落地之后不成立。补写闸（闸四 `crates/singlefs-core/src/mount.rs:5845` 「有写序实例大于所选那一版实例的单元就不补写」）靠旧池同前缀的单元挡住补写；强制新 fsid 之后旧池单元的前缀不同，闸四不再挡，那块盘被补写、可写挂载做成。所以这一句要等批 8 的口径定了才能写，写成「按现行补写条款处置」，不要把「拒」写死。
- **对不上②**：「读照读」。空池没有文件，`mount_read_only` 对 mkfs 刚做完的池也报 `Open`：`crates/singlefs-core/src/mounted_read.rs:647` 「树表 0 条不是走读失败（这一版还没发布过文件版本），不退。」下面 `:648` 是 `Err(OpenPoolForReadFailure::TreeTableHasNoEntries) => {`；乙做完那一态也是 `read_only=Err:Open`【量】。只读挂载在空池上观测不到，「读」只能指恢复读回 `NoFile(i0t0)`；条文改写成「恢复读回第 0 代空池」。
- 不分辨臂：这 40 态属于最后四段（三条根、系统配置四槽），乙与今天的写序在这四段上逐段相同（S4-1 的种类串），所以它们在今天的 280 019 态里同样存在，不是乙带来的。
- 什么现象推翻：缺盘那些态里有一态恢复读回的不是 (0, 0)，或可写挂载做成而那块盘一份自证的槽也没补。

### T5　作废持久之后的两个「不得」

- 条文：「任何结局不得读回旧池的任何一版，也不得新系统配置与旧池的根并存。」（`research/prompts/m3-close-c656-r2-sonnet-output.md:169`）
- 读数：作废之后择旧池 0 态、混合 0 态（31 187 态 × 三份历史 × A、B）【量】【量·B】。
- **对不上**：句子没带前提「每块盘的屏障都到盘」。第二轮攻方手搭的「盘 1 不认 flush」态在 fsid 相同时就是新系统配置加旧根并存：`research/prompts/m3-close-c656-r2-opus-output.md:11` 「fsid 前 8 字节不同时变成整池拒」。这一形归 C6（`.claude/kb/checks-owed.md:22` 「块层重排 / FLUSH / FUA 的语义假设本身写错」），所以 T8 那句「不管」必须排在这句前面，或这句带「设备守屏障时」。
- 什么现象推翻：设备守屏障的前提下作废持久之后任一态择到旧池或混合。

### T6　附带①（一盘作废一盘没作废）

- 条文：旧池只读挂得上而可写挂载拒（`research/prompts/m3-close-c656-r2-main-verification.md:23` 的 U2 行：「附带两样写进条文」）。
- 读数：A 树第 0 段 8 态里 4 态（一盘作废的 2 个不撕裂态 + 2 个撕裂态）只读 `Ok(i1t6)`、可写挂载拒；另 4 态可写挂载做成【量】。**B 树 8 态可写挂载全做成**【量·B】（补写闸）。
- **对不上**：这一句的真假取决于批 8 落不落地，两树答案相反。批 8 的口径没定之前这一句不宜进条文；要写，写成「旧池读回最后确认的一版」（T2），把可写挂载的结局交给现行补写条款。
- 什么现象推翻：A 树上一盘作废的态可写挂载做成；或 B 树上这些态里有一个可写挂载拒。

### T7　附带②（择系统配置取第一块可择盘那一份）

- 条文：择系统配置取第一块可择盘那一份、不取全池最大世代号（`research/prompts/m3-close-c656-r2-opus-output.md:190` 的第 2 条）。
- 代码：`crates/singlefs-core/src/recovery.rs:1539` 「None => chosen = Some(best_on_device),」；后面的盘只比 fsid 与设备数：`crates/singlefs-core/src/recovery.rs:1545` 「return Err(RecoveryFailure::SystemConfigurationsDisagree);」。两树同【推】。
- 读数：第 0 段择中世代 7 或 8 的旧系统配置，读回都是最后确认的一版【量】【量·B】（B 树旧池世代是 9，同形）。
- 什么现象推翻：两盘世代号相差的态里，取第一块可择盘那一份读回的不是最后确认的一版。

### T8　「不管」一句之一：屏障只到一块盘归 C6

- 条文：屏障只到一块盘归 C6。C6 行 `.claude/kb/checks-owed.md:22` 「块层重排 / FLUSH / FUA 的语义假设本身写错」，字面归得上。
- 本腿没重量这一形（第二轮攻方手搭 12 态，181 398 528 态没跑）。提醒两点【推】：（a）乙新增的作废屏障要进门禁 checker-tier-qemu-device-streams 拿虚机录到的 mkfs 那一行比（`.claude/kb/layout/01-first-txn.md:410` 「门禁 checker-tier-qemu-device-streams 拿虚机录到的 mkfs 那一行与它比」，该行末格自己写了这一条），C6 的「仍欠」栏要带上它；（b）强制新 fsid 之后这一形在 fsid 相同的格子里已经是空集——手搭态变成整池拒（`research/prompts/m3-close-c656-r2-opus-output.md:11`），「不管」只在极少数前 8 字节碰撞时有对象。

### T9　「不管」一句之二：两块盘来自两个池的起点不在射程里

- 条文：两块盘来自两个池的起点不在射程里。第二轮量过：`research/prompts/m3-close-c656-r2-opus-output.md:10` 「作废之前 3 态择到盘 1 的池 Q，只读挂上 Q 的 (1, 4)，Q 最后确认的是 (1, 6)」。
- 本腿没重量（我的原型没有这种历史）。留一个后果：这一形里 T2 的「旧池读回最后确认的一版」的「旧池」没有定义（起点就整池拒，mkfs 的第一个写把拒变成挂上另一个池的更旧一版），T2 的限定语「两块盘是同一个池」要写进去。

### T10　「跨盘世代号差大于 1 就拒」（第二轮攻方提的，零轮）

- 条文：`research/prompts/m3-close-c656-r2-opus-output.md:190` 「我提的改法（**只在我的推理里、没实现没跑、被攻过零轮**）：跨盘择系统配置时两盘世代号差大于 1 就整池拒。」
- 它的出处是：「今天写序正常落盘时两盘世代号不可能差这么多……两盘差最多一次轮换」（同一行，读码）。本腿**量**了这个读码：`lag-acquire` 模式把取号这一步录成流，崩溃态 = 流里除「盘 1 的最后一次写」以外全落（它之前的屏障都生效，是 D13 已定项 4 的合法崩溃态：`.claude/kb/decisions/13-验证路线.md:138` 「当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上」，段内任意真子集可持久），连着崩 4 次、每次崩后恢复读回、只读挂载、可写挂载、池级 checker 全判一遍。A 树取号一轮（2 个写），B 树取号两轮（4 个写，每轮后一道屏障，`crates/singlefs-core/src/transaction.rs:1235` 「for _round in ACQUISITION_WRITE_ROUNDS {」）：

| 崩溃次数 | A 树 盘 0 / 盘 1 最大世代号（差） | B 树 盘 0 / 盘 1 最大世代号（差） | 恢复读回 / 只读 / 可写 / checker 红项（A、B 同） |
|---|---|---|---|
| 1 | g9 / g8（1） | g11 / g10（1） | FileRead(i1t6) / Ok(i1t6) / Ok(i1t6) / [] |
| 2 | g10 / g8（2） | g13 / g11（2） | FileRead(i1t6) / Ok(i1t6) / Ok(i1t6) / [] |
| 3 | g11 / g8（3） | g15 / g12（3） | FileRead(i1t6) / Ok(i1t6) / Ok(i1t6) / [] |
| 4 | g12 / g8（4） | g17 / g13（4） | FileRead(i1t6) / Ok(i1t6) / Ok(i1t6) / [] |

- 读法【量】【量·B】：两盘最大世代号的差随连续「盘 1 最后一写没落」的崩溃次数涨到 2、3、4，池每一步都读回最后确认的一版、两种挂载都做成、checker 零红。所以这条规则会把一个今天完全正常的池在第二次这样的崩溃之后整池拒。它防的那一形（第二轮攻方的屏障态：盘 0 世代 1、盘 1 世代 8）在合成形态里已经由前 8 字节不同的 fsid 变成整池拒（`research/prompts/m3-close-c656-r2-opus-output.md:11`），这条规则在合成形态里既多余又有误拒。**判：不要。**
- 限度：每次崩溃都是一个合法的单点崩溃态，连着四次这样的崩溃概率很低；盘 1 持续丢这类写（但屏障生效）是它的系统性来源。没量盘 1 读错、缺盘的变体【推】。
- 什么现象推翻：在 `lag-acquire` 的任一步，恢复读回不是最后确认的一版、或只读 / 可写挂载拒、或 checker 红；或有人给出这条规则要抓而别的东西抓不到的合法态。

### T11　强制新 fsid 的条文

- 用户口径 `research/prompts/_m3-close-c656-r3-body.md:11` 「mkfs 一律生成新 fsid、不许同 fsid 重用」。
- 条文要点一：比的是前 8 字节，不是 16 字节。S4-3【量】：只有第 8 个字节不同就绿，后 8 字节不同仍红；单元头只带前 8 字节（`crates/singlefs-core/src/unit.rs:42`）。「不许重用」mkfs 自己保证不了（它不读盘），能写的只有「前 8 字节取自随机源」；概率碰撞（16 字节随机，按前 8 字节一对 2⁻⁶⁴，第二轮攻方的数【r2量】）留给 T11 之后的「已知形态」（第四节）。用时间当源不行：同一秒内两次 mkfs（脚本、测试循环）前缀必同。
- 条文要点二：与现行文字冲突的地方：`crates/singlefs-core/src/make_filesystem.rs:66` 「同参数两次 mkfs 逐字节相同（里程碑步 1 验收）」，`crates/singlefs-core/src/make_filesystem.rs:72` 「pub filesystem_identifier: [u8; 16],」（调用方给），`crates/singlefs-core/src/make_filesystem.rs:533` 的「拿同一个 fsid 在用过的盘上重做 mkfs，旧池留在槽 1..7 里的根照样自证得过、解得开，」（注释前提），`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:118` 的 `same_parameters_twice_give_byte_identical_images`。
- 条文要点三（S1 的数，grep 数的，含注释之外的行）：`crates/` 里 `make_filesystem(` 的调用行 234 处、94 个文件；`filesystem_identifier:` 的初始化与字段行 179 处、65 个文件；`MakeFilesystemParameters` 出现在 74 个文件；`scripts/` 下 9 个文件用到；`base == prepared.base` 这类「两次独立造池的起点镜像逐字节相等」的断言 8 处；`crates/singlefs-harness/src/scenario.rs:57` 「filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,」 是装置共用的固定 fsid；`.claude/gate.d/crash-point-replay-items.tsv` 非注释行 32 条，起点镜像都是 mkfs 造的。随机 fsid 进 `make_filesystem` 本身，这些全部要改；保留一个带 fsid 的内层入口、只让对外入口生成，则「一律」就有了测试里的例外。这一岔（`一律` 是否允许内层入口）是用户的，不是本腿能判的。

### 三之末　辩方改写稿（零轮措辞，措辞待主 agent 定）

按上面逐句对读数的结果，把 X 与乙的条文合起来改成下面这样（不写态数；不写「读照读」「作废之后整池拒」；不写「跨盘世代号差大于 1 就拒」；括号里标出每一句依赖哪一条读数）：

> mkfs 先把每块盘的 [0, 根环基址) 整段写 0，两块盘各过一道屏障（作废），之后才写别的（T1）；fsid 由 mkfs 生成，前 8 字节取自随机源（T11）。
> 两块盘是同一个池时，作废持久之前崩溃：旧池读回最后确认的一版；可写挂载的结局按现行补写条款（T2、T6）。
> 作废持久之后、最后一道屏障之前崩溃：下一次挂载只有两种结局——整池拒（`NoValidSystemConfiguration`），或恢复读回 mkfs 的第 0 代空池（根为实例 0、txg 0，没有文件）；后一种里系统配置的冗余可以缺，缺新槽的那块盘的可写挂载按现行补写条款处置（T3、T4）。
> 设备守屏障时，作废持久之后任何结局不得读回旧池的任何一版，也不得新系统配置与旧池的根并存（T5）；屏障只到一块盘归 C6，两块盘来自两个池的起点不在射程里（T8、T9）。

## 四、S1（兼）与被排掉的两支：「拒」「认作已知形态」

### 4-1　三种落法写成对 `crates/` 的改动，各修不修得到、与乙的作废段怎么排

| 落法 | 对 `crates/` 的改动 | I-7.7 | 屏障只到一块盘 | 与乙的作废段 | 与「同参数逐字节相同」 |
|---|---|---|---|---|---|
| 生成 | mkfs 的 fsid 不再由调用方给（T11 要点三的改处数） | 前 8 字节不同 → 做完绿、整池拒后重跑（新 fsid）31 122 / 31 122 态绿【量】；前 8 字节撞（2⁻⁶⁴）仍红 | 变整池拒（`SystemConfigurationsDisagree`）【r2量】 | 无交互：作废段不读 fsid，两趟写序的段序列与 fsid 无关（S4-1，三份历史同） | 冲突，需保留带 fsid 的内层入口（T11） |
| 拒（读盘上已有 fsid，前 8 字节相同就拒） | mkfs 开头读盘；守卫读什么见 4-2 | 起点镜像上同前缀的 mkfs 被拒；作废之后的重跑看守卫读什么 | 起点同 fsid 已被拒则无此形【推】 | **要排在作废之前读**（作废之后读不到系统配置，守卫掏空） | 不冲突（fsid 仍是参数） |
| 别的落法：读单元区的单元头 fsid 前缀 | mkfs 开头扫单元区头（`unit_header_fsid_low`，探针里照 checker 的偏移表） | 作废之后重跑旧 fsid：31 122 / 31 122 拒 | 同「拒」 | 不要求排在作废之前（单元区作废不碰） | 不冲突 |

### 4-2　替「拒」辩护：它出局的真正理由是自撞，不是被掏空

正文 S1 转述的第二轮理由 `research/prompts/_m3-close-c656-r3-body.md:26` 「乙的作废之后再跑 mkfs 读不到旧系统配置，这条守卫被掏空」。守卫读什么，结论不同。`distinct` 历史（旧池 fsid 前 8 字节与新池不同），乙的 31 122 个整池拒态上，用户接着做两种事，三种守卫的放行 / 拒（`run-distinct-yi.log`【量】，`same`、`prefix` 两份与之同形，B 树相同【量·B】）：

| 守卫读什么 | 用旧池的 fsid 重跑（有害的重跑）拒 | 用新池的 fsid 原样重试（自然的重试）拒 |
|---|---|---|
| [0, 根环基址) 里的系统配置 | 0 / 31 122（被掏空） | 0 / 31 122 |
| 单元区里 mkfs 自己写的那 3 个槽的头 | 29 160 / 31 122 | 29 178 / 31 122 |
| 整个单元区的单元头 | **31 122 / 31 122** | **29 178 / 31 122** |

- 够不着的部分：读整个单元区的单元头的守卫在旧 fsid 重跑上一态不漏，所以「被作废掏空」不是「拒」这一支一般地出局的理由，只是读系统配置那一种守卫的。
- 站不住的部分：同一守卫把「用新 fsid 原样重试」也拒了 29 178 / 31 122（93.8%）：崩溃的那次 mkfs 自己写下的单元带着新 fsid 前缀，守卫分不出它们与旧池的单元。守卫要么放过自己的残骸（读不出区别，不可能），要么把最自然的恢复动作「再跑一次同样的命令」堵死、要用户显式覆盖。守卫的代价还有：读整个单元区的头，4 TB 盘每槽 16 KiB 约 2.4 亿次读【推】。
- 「拒」最强的一版不是比 fsid，是**盘上有任何自证过的本类系统配置就拒（不管 fsid），显式覆盖才放行**——C484 当年的路 ③：`research/prompts/m3-close-c656-r2-sonnet-output.md:94` 「③ mkfs 在一块已有有效池的盘上拒绝跑、除非显式覆盖」；用户选的是 ①，`.claude/kb/checks-owed.md:611` 「用户定的路 ① 连根环一起清」。这一版读的正是乙的作废要抹掉的东西，所以：乙的 31 195 态里它在 73 态上拒（作废之前 8 态 = 旧池，作废之后 65 态 = 已经读得回空池）、在 31 122 个整池拒态上放行，**不拦自己崩溃之后的重试，也不依赖 fsid**（三份历史同，由记下的 `choice=` 列推出【量】）。它保护的是另一件事：对一个还挂得上的池敲错 mkfs，乙让这件事一道屏障之后不可逆（S4-6：α' 也请不回来），用户事先没有任何提示。强制新 fsid 不改这一点（新 fsid 照样抹旧池）。这一版是否进条文是用户的取舍（要不要一个覆盖开关），本腿只报它不被乙掏空、不自撞。
- 什么现象推翻：整池拒态上这一版守卫拒了一态（它读的是「有没有自证过的系统配置」，整池拒的定义就是没有）；或乙的崩溃态里有一态「还挂得上旧池」却被它放行。

### 4-3　替「认作已知形态」辩护：它是强制新 fsid 的残余，不是它的对手

- 第二轮的「丁七」（认 I-7.7 在同 fsid 重做 mkfs 后报红，C659 射程收窄后记成已知形态）与「强制新 fsid」不是二选一：强制新 fsid 把 C659 缩到「前 8 字节碰撞」，丁七处理的是剩下的那一点。合成形态下该问的是这一点要不要花代码去消。
- 站得住的理由，都量过：（1）红会自愈，到新池系统配置的实例代号追上旧池单元里最大的实例代号为止（S4-5：K = 1 一次挂载，K = 4 四次挂载）；（2）红的全程读者读对：恢复读回新池的空池、可写挂载做成；用起来之后（5 个版本）checker 零红、读回新池最后一版的字节，与全新盘对照逐项相同（S4-5）；（3）消它的另一条路甲七要把崩溃态 31 195 → 280 027（9 倍）、每次 mkfs 写满整个单元区（S4-4、S4-1），而它只修一个 2⁻⁶⁴ 的形；（4）红的条件已写得很窄：前 8 字节相同（S4-3，边界到字节）。
- 站不住 / 要补的：（a）红的时长没有上界（K 是旧池的挂载次数），期间 I-7.7 ① 已经被旧单元顶红，这个池自己新出的真孤儿（C322 的取号后崩）再红一次看不出来——判据是「实例代号 ≤ 各盘系统配置实例代号的最大者」（`crates/singlefs-checker/invariants/02-根环.md:14`），红是整条不变式的一个布尔；这一条我是读判据推的，没量；（b）B 树的补写闸四同样被前 8 字节碰撞挡住（S4-2：40 态仍拒），所以碰撞后的新池「缺一盘新槽」要用户重跑 mkfs；（c）「认作已知形态」要落成一条会红的东西，不是一句话：C659 行（`.claude/kb/checks-owed.md:431`）的验收格要改成「前 8 字节碰撞的盘面 I-7.7 红且读者读对，红随取号自愈」，并造一张坏镜像钉住；否则它就是没有会红形式的结论。
- 什么现象推翻：新池有了文件、快照、回收之后，同前缀的旧单元被扫描重建或回收读到（本腿里新池只写了一个文件，I-1.13 一直绿，推不出更多）；或红的持续时间不随取号线性消失。

## 五、这一轮新冒出的形态（零轮，不再开一轮）

| # | 形态 | 来由 | 状态 |
|---|---|---|---|
| Z1 | 工作树在轮中动了：批 8（系统配置落地）的未提交改动让 T4、T6 的括号两树答案相反（A 拒 / B 做成） | 另一个会话的改动，树 B 的哈希写在第零节 | 量过两树；T4、T6 的括号要等批 8 的口径定 |
| Z2 | 强制新 fsid + 批 8 的补写闸：旧池单元前缀不同，闸四不再挡，「缺一盘新槽」的 40 态可写挂载做成并补写那块盘（`distinct` 65 / 65）；前 8 字节撞时仍拒 | S4-2【量·B】 | 零轮；与 `.claude/kb/decisions/22-单元原子性怎么合成.md:180` 「某块盘一份自证过的系统配置都没有时，取号那一步也拒」要对口径 |
| Z3 | 「盘上有任何自证过的系统配置就拒（不管 fsid），显式覆盖才放行」的 mkfs 守卫 | 4-2，由已记的 `choice=` 推出：乙的 31 195 态上 73 拒、31 122 放行 | 零轮；未实现、未在态上真跑守卫，只是 `choice` 列的推出【量·推】 |
| Z4 | 「认作已知形态」要带一张会红的坏镜像与改过的 C659 验收格 | 4-3（c） | 零轮措辞 |
| Z5 | 强制新 fsid 的源：前 8 字节要随机，时间当源同秒必撞；「一律」是否允许带 fsid 的内层入口（测试、E142 装置、崩溃点重放 32 项都靠固定 fsid 造起点镜像） | T11 | 用户的取舍 |
| Z6 | 「跨盘世代号差大于 1 就拒」会误拒合法崩溃之后的池（差 2–4） | T10【量】【量·B】 | 建议不要；若有人坚持，先给出它要抓而强制新 fsid 抓不到的合法态 |
| Z7 | 条文措辞：T2「照没跑过 mkfs 处置」「或整池拒」、T3「作废之后整池拒」的简写、T4「读照读」、T5 缺前提 | 第三节 | 零轮措辞，改写已在各条里给出 |

## 六、前两轮报出而没进任何一格的问题

| 问题 | 出处 | 本腿处置 |
|---|---|---|
| α' 离线出路与乙作废的交互 | `research/prompts/m3-close-c656-r2-sonnet-output.md` 没做什么一节、`research/prompts/m3-close-c656-r1-sonnet-output.md` 第 2.5 节 | 量·B：作废持久之后 31 122 / 31 122 请不回旧池（S4-6）。树 A 里没有这个模块 |
| 乙作废段 8 态上池级 checker 红不红 | 第一轮正推 没做什么 | 量：8 态 `raw_red=[]`，全绿（A、B）；「可写挂载做成的态做成之后再读回什么」没量 |
| 择到旧池世代 7 的态可写挂载做成之后再写，两盘世代号会不会错开 | 第一轮攻方限度 | 部分量：T10 的取号层面（差随崩溃次数涨）；「择到旧池后接着发布」没量 |
| 屏障只到一块盘全量、两块盘来自两个池的反向拼 | 第二轮攻方限度 | 没做 |
| 旧池处在发布半截、旧池几何不同而根环偏移有重合、journal 比根新 | 第二轮辩方没覆盖 | 没做 |
| 读故障、一块盘中途掉线 | 第二轮攻方（推的） | 没做 |
| 真设备上 `write_zeroes_at` 的映射、屏障对它是否生效、O_DIRECT 写零耗时 | 第一轮正推、第二轮攻方 | 没做（C6 范围） |
| 错误名 `SystemConfigurationUnreadableAfterTheReadStage` 分不出「mkfs 没做完」与「盘坏」 | 第二轮攻方（推的） | 没动；B 树 + 不同 fsid 时那 40 态不再报它（T4），问题缩到前 8 字节碰撞与 A 树 |
| 只读挂载在缺本池系统配置的盘时也拒、作废写墓碑 | 第二轮攻方（零轮改法） | 没碰；T4 的 `mount_read_only` 在空池上总报 `Open` 使「只读挂载」这一半在空池上观测不到 |
| 乙七判别子观测不到 | 第二轮判决 | 没复核；4-3（a）与它相关（同一个「同前缀单元与孤儿同形」） |
| 1 MiB 清零反着撕（槽 1 清了、槽 0 还在）不在枚举域 | 第一轮攻方第二次抽样限度 | 没量；第一轮正推用 16 种任意扇区子集盘面推过（`research/prompts/m3-close-c656-r1-sonnet-output.md`） |
| 入库没有枚举 mkfs 自己的崩溃点重放项 | 第一轮攻方 | S4-7：32 条里没有 |

## 七、没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；第一轮、第二轮判决条目里没碰的：乙七出局、屏障只到一块盘与两块盘来自两个池的重量、第一轮 V3 的改处数（第三节 T1 的 7 处是 grep 重数的，没逐份翻用例）、V4 的几何。
- 没跑重型测试、没调 checker 档任何测试目标、没起 `gate.sh` 与任何门禁阶段、没起 `--quick` 三条。探针 crate 依赖 `singlefs-checker-tier` 库只为用它的崩溃点重放计数与逐态展开函数；编译用 `cargo build`，跑编出来的二进制一律经 `run-with-memory-cap.sh`（探针 4G、整条 8G），线程 31。
- 没改主工作区的 `crates/`、kb、门禁表；副本里唯一的改动是那一处可见性。所有【量】都是仓副本上的数，不是入库装置上的数；按三方规则要引进 kb 须在入库装置上重做（第二节 S4-7 说明入库装置里没有枚举 mkfs 的崩溃点重放项，重做先要新增一条流与观察者）。
- 树 B 是另一个会话的未提交改动，本腿没动它；它此刻（交回前再比）仍是第零节写的哈希，但此后会不会变不归本腿。批 8 的口径没定，T4、T6 不宜定稿。
- 没复核第二轮的屏障只到一块盘与两块盘来自两个池（手搭 / 31 168 态）；没量读故障、一块盘中途掉线、旧池发布半截、journal 比根新（第六节）。
- 过程里的两处错，已改：（1）第一次建树 B 的探针时与树 A 共用一个编译目录，cargo 把树 B 的包当成「没变」沿用了树 A 的产物，量出的「树 B」数其实是树 A 的，发现后各用各的编译目录（拷贝同一份种子）、全部工作区包强制重编（脚本里 `touch` 全部 `.rs`），树 B 的数全部重量；（2）`lag` 模式一开始把「盘 1 在整次挂载里的写全丢」当崩溃态，那是屏障只到一块盘的故障、不是合法崩溃，改成只丢取号流里盘 1 的最后一写（`lag-acquire`）；两个模式的输出都留着（`lag-4.log`），后者是 T10 的依据。
- 前台命令被工具超时转后台过几次（排队等准入、全量枚举），没有用 `run_in_background`。

草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-c656-r3-sonnet/`：保留 `frozen/`（树 A 的冻结副本，35M）与 `frozen-live/`（树 B 的冻结副本，35M），主 agent 复跑要它们，不删；`battery.sh` 与几份批跑日志是草稿，留着。已删：两棵仓副本 `tree/`、`tree-live/`，三个编译目录 `target/`、`target-frozen/`、`target-live/`（各 6.3G），六个编好的探针二进制 `bin-*-probe`，各趟 `rerun-*` 的副本。复跑从头编要编 `librocksdb-sys`，内存紧时 8G 上限可能不够（第一轮攻方两次撞上限、10G 加 `-j 8` 编过）。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-close-c656-r3-sonnet-output.md --root . --background research/prompts/_m3-close-c656-r3-background.md`：

```
  ✓ 核了 38 处引文、15 处裸行号，对上 38 处，没判 15 处（其中裸行号没有引文可比 15 处；逐处列在上面）
```
