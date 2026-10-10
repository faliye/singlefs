# m3-batch33r-code-r1 云端攻方（Opus）报告（2026-10-07）

攻击面：正文第三节 K2（R1 新旧判法分歧与「没有文件」组放过读出内容；R2 工作线程上的漏核与重核；R3 删除流里 oracle 放过而该红的状态）。
立场：假设 R 是错的，在冻结副本的拷贝上造历史、输入与操作序列去打穿它；只跑 harness 档与自己的原型，singlefs-checker-tier 的测试一个没跑。
冻结副本 `~/.local/share/singlefs/work/m3-batch33r-code-r1/tree/` 里被判的 4 份 sha256 与 `research/prompts/m3-batch33r-code-r1-snapshot/crates-sha256.txt` 逐行相同（开工时 `sha256sum` 现核）。

## 各格判定一览

| 格 | 判定 | 依据（详见各节） |
|---|---|---|
| R1 新旧判法分歧、旧的才对 | 没打中 | 新判法的违例集合是旧判法的子集，分歧只在「意图那一版起报没有文件」；H1/H2/H3 共 30 个根 × 4 种结局，新判法与由 `TransactionOutput` 现算的真值 0 格不同，旧判法 17 格不同、全是新的对。旧的才对要谱系分叉（同 txg 两实例），按代码推：那张表在正确实现上自己先红，不是静默放过（推的，没造） |
| R1「没有文件」组里的版本被读出内容判成立 | 没打中 | 读出内容落在「没有文件」组无条件红（crash.rs 1152 行），同一根两组在建表时停（1013 行）；逐根表里每个意图根喂读出内容全拒 |
| R2 工作线程上漏核、重核 | 没打中 | 33 态流 13 种跑法（1/4/6 工作线程、调用线程、三种跨线程续跑、panic 后再续、两片再并）计数逐项全等；续跑那一趟观察者恰被调 17 = 33 − 16 次；panic 后进度文件被删、从头重跑 |
| R3 删除流里 oracle 放过而该红 | 没打中 | 点名的两形（最后一批之后读出内容、意图之前报没有文件）都红；H2 意图之后两截全量 393259 态逐状态三方比，新判法与真值 0 个分开 |
| 观察（不是打中） | — | ① 计划哈希只拼「有没有观察者」，观察者函数不进哈希（R 之前就是，靠输入指纹）；② 冷走读 nlink = 0 就答没有文件，oracle 在删除那几版看不见 size 与单元，替它看的是 checker I-4.6 等（取样只取到 2 个删除态，都判了 holds） |

四格都是「没打中」，这是一次抽样：照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 要再抽一次才记「没打中」。

## 复跑

模型目录 `research/prompts/m3-batch33r-code-r1-opus-model/`（原型 crate、复跑脚本、比对脚本，与一次从空目录复跑的全部输出 `run/`）。复跑（仓根下；阶段可以分几条命令起，`all` 一次跑完，轻载时约 40 分钟）：

```
SINGLEFS_REPO=$PWD bash research/prompts/m3-batch33r-code-r1-opus-model/rerun.sh <空目录> all
python3 research/prompts/m3-batch33r-code-r1-opus-model/compare.py research/prompts/m3-batch33r-code-r1-opus-model/run <空目录>
```

`rerun.sh` 先拿 `research/prompts/m3-batch33r-code-r1-snapshot/crates-sha256.txt` 核冻结副本（`FROZEN_TREE`，默认 `~/.local/share/singlefs/work/m3-batch33r-code-r1/tree`）被判的 4 份，再拷进输出目录、把原型加成 workspace 成员、编 release（编译目录在输出目录里），每个阶段经 `run-with-memory-cap.sh`（编译 6G、跑 4G）与 `capped.sh 6`。交回前在空目录 `~/.local/share/singlefs/work/m3-batch33r-code-r1-opus/rerun1/` 从头跑通一次（分 18 条命令起，其中三条超过四分钟、被工具挪到后台跑完），`compare.py` 拿它与开发时的输出比（见文末「复跑核对」）；`run/` 就是那一次的输出。

模型目录里每个文件的 sha256（`cd research/prompts/m3-batch33r-code-r1-opus-model && find . -type f | sort | xargs sha256sum`，50 份）：

```
452ca721cb72b3e5a2f801703bd7c98430ca02d2cfe9c5a1814d224d20ba58f1  ./compare.py
627aff9fcae6bc06ea24d57668a1cbcd44d9bc7508c2588145881dab0a6e1b65  ./compare-run-vs-development.txt
7a87bc3571cb2965603bab5f73911e1c5e917146d7d6cf29afeeb3d409872a83  ./probe/Cargo.toml
91e9745515228026c5032f9f903a409fb330182abc09fb5c10e609761e00fee1  ./probe/src/main.rs
cb68e99777c5e8009b59f6b1ff08d95787dce71894e6b85c6e847e3c4a2475b6  ./rerun.sh
21e19b600062043bc90d7cd5978330d70474973b0077a3db339fe3d0f3c93522  ./run/oracle-instance-row-merge.out
8bf584ff299ece66b0e4252ce2a8b44226a4595043876141a81840d31adc4424  ./run/oracle-instance-row-s0.out
0800a68af3365dd390316770e795f013ec5e261e2da097e33e1197b1977ef746  ./run/oracle-instance-row-s10.out
7fea91de038670d2b5216593c40770cd957d97c861ee4b26ea92283043b8d758  ./run/oracle-instance-row-s11.out
b97555d9d06ddec0fe096038e4fdcb8adbf957318407b259b65966a865f1c67c  ./run/oracle-instance-row-s12.out
76853d858b4e7feb893236785b058ce27ea232dfdbb6438f2555f6736cbed474  ./run/oracle-instance-row-s13.out
d1c998dff6eed0f72f82cf5a3759fa6fde0b1c48f8fcdf7aab360a25acfa7057  ./run/oracle-instance-row-s14.out
27c29d1066283a1b166045a617b027025051288b91daf3591dfb2b2a85df8f03  ./run/oracle-instance-row-s15.out
5972f08ec8ab49645bdec57ef20e3b0f254961151d42cedd8fe5b72a77e680ae  ./run/oracle-instance-row-s16.out
a322375f1d911f1e66e5b3e35bc61810f008a9f548673e1323fc46c47417394b  ./run/oracle-instance-row-s17.out
25e91b6e5ef34dcabf009bd2bf908127565b2dc40b9300eb8ff721b5666b43e7  ./run/oracle-instance-row-s18.out
211aba87440853b1de2f0833dd17a4ed75c67ab44198484f7010f7c270eb5f39  ./run/oracle-instance-row-s19.out
97e0f8b5e3ea279304ce9786996a94ac587e07cd6d1b3dab9e20db179051f05c  ./run/oracle-instance-row-s1.out
4554c7b2ca4f6d90205339d646c76b21ce6d78438e1e386bad7e994f966fbfe1  ./run/oracle-instance-row-s20.out
35a9414234cf6af1875340147106559a90a12014a0f50bc91092c5a2a1416115  ./run/oracle-instance-row-s21.out
5d90b5400c00a6b792888f36fecb1563f9a909ac8dacf396b0defffd92b0a3ca  ./run/oracle-instance-row-s22.out
6f14c51273636731f19ec6cfe2a76c03e4ef51d3d56174126e1aefa5c811ffe5  ./run/oracle-instance-row-s23.out
1ddd350950a9abdde4309e7b2af000a4435b7b1f834bd35a9c1d7ca15a586600  ./run/oracle-instance-row-s2.out
2662511a802ec2d5a93ab687e4c792cb442a223f2546f6b460aa959c1839a8fc  ./run/oracle-instance-row-s3.out
e349f3b2348cb5bd6c281e4beeaaa27e55475648df8c6b2e655cba2203946d82  ./run/oracle-instance-row-s4.out
203e44c6b9864dcd8fe3042d8685f319393a8245c47c59cf8d263d19dcf91ee1  ./run/oracle-instance-row-s5.out
f4ae4dfe12b7e6657e0865816b885d70ee192f8224b9e42be95d729c7297c0ab  ./run/oracle-instance-row-s6.out
8dd4df429a9262e36850d5238213d225becf0a279a9ef6988acac46f4ba0ade1  ./run/oracle-instance-row-s7.out
738b8c59cda4ced2715108593a2592a0bff01f15511cb684b44d15967b8c6247  ./run/oracle-instance-row-s8.out
6b3fe3e955cd0f7e456868b3a65eb6a6f2e956e4fbbe5faa9d48fc9417a301f7  ./run/oracle-instance-row-s9.out
5c3398d4f054b7afa65e0b3249d55e70330440254746c2c0b457399eb33aa2d8  ./run/oracle-warm-ups-merge.out
27405f78119521d397715e46a5a4f66e42aafd84ff3aa36d5bd136bce7d06b9a  ./run/oracle-warm-ups-s0.out
0c31d8a93d25eb4d48132d285b631318f3a60987d62c1d75f1db69340e3dd7c7  ./run/oracle-warm-ups-s10.out
425f2acd4760d52015fc39e8afa181d536e5b40b1631000dc76c8e788ce086e8  ./run/oracle-warm-ups-s11.out
8cd4a4e058eb928adae38daf5ea2956e8098332c9ec529b33674b8f3fa960ea5  ./run/oracle-warm-ups-s12.out
54aacdfb1a38e1cdf9ebea02cd2cd48f118a2432b264f42a947c2827a50f335b  ./run/oracle-warm-ups-s13.out
b9de9e407fb2a74409171a847d4fff0192c0ff9b41e64e154999b57554327d83  ./run/oracle-warm-ups-s14.out
8e1e25ce2e7df037da32a28927352b408c414f19d7dc66f164e7b006781cc22e  ./run/oracle-warm-ups-s15.out
90475e127ef82052fb3c80cd1a3f208f29558a2abfd7d0e8cc48debcaee8395d  ./run/oracle-warm-ups-s1.out
1389aa166979e61566429252949d7892bc4f07e0c2c8edd6e4aafe380f288aff  ./run/oracle-warm-ups-s2.out
58d876836e1977d06cbee3aeff0a9690a0931b156ac064bd40f1f4a17f555268  ./run/oracle-warm-ups-s3.out
c80ac63ca7adc57151b77c975d9aaf70530c543aa8ee2d6afb24005e34170653  ./run/oracle-warm-ups-s4.out
0a5ea3c71cbdc6b35da936189d044e12345ccec80af3a542f1459368c5954a95  ./run/oracle-warm-ups-s5.out
1493e94b7848dca782fcb1c035354aeceeb4422a6aaf15341fcd4f0858832cfb  ./run/oracle-warm-ups-s6.out
9469b96a52437a99ad719719acfe20a99b1f09e33fcbbeec97fa9bf5a43686a3  ./run/oracle-warm-ups-s7.out
c9ea1a1f445ff619424d0c226be5cec4d6f78089745f314b8f5ec2337e287210  ./run/oracle-warm-ups-s8.out
182ce995478bba49a77ff39d64db680ea5b44fc7bb19995d19888cf8331df9a1  ./run/oracle-warm-ups-s9.out
c7e84326c54481c680a5f5b3d797a741f22996915508d45e0d1221fa5e34dfe8  ./run/roots.out
5bd4838ac0148ef193d5ec91c0260eac1dbfc33382af3d424ed73cac56c317b5  ./run/threads.out
b1f8bcb4e1d8d046900aaad7cbeffb59477f3e1b5f52c099c20a96b3725442d8  ./run/verdicts-h1.out
```

## R1　新旧判法分歧与「没有文件」组放过读出内容

### 分歧只落在哪一格（按代码推，再在三段历史上逐根量）

新判法只在两处与旧判法不同，都是「报没有文件」：

- 落到的根在「没有文件」组里：「        (RecoveryOutcome::NoFile { .. }, Layer0VersionAtTheRoot::WithoutAFile) => None,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1145`）；旧判法把这个根当成不在表里，任一更旧版有文件就红。
- 落到的根不在表里，比它旧的最新一项是「没有文件」：「            (None, Some(_) | None) => None,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1070`）与「                    .then_some(with_a_file)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1067`）。

读出内容的三格新旧相同（读出内容落在不在表里或「没有文件」的根上一律红：「            Layer0VersionAtTheRoot::NotInTheTable | Layer0VersionAtTheRoot::WithoutAFile,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1152`））；新判法的违例集合是旧判法的子集，「或反过来」那一向（新的红、旧的放）不存在。
所以「旧的才对」只能是：某个根报没有文件、它比「没有文件」组里最新的那一版新、而它其实是从一个带文件的更旧版本长出来的（谱系分叉）。

### 逐根逐结局量（模型 `roots` 模式，不枚举）

三段删除历史（`singlefs_harness::scenario::run_deletion_history` 的 H1 意图与每批同进程、H2 意图后正常卸载再可写挂载续做、H3 旁边多一个 size 0 inode；两盘，内存稀疏盘）各录一遍，每个根的真值从各次发布交回的 `TransactionOutput`（第一个文件记录在不在、nlink 是不是 0）与写进去的内容现算，不经恢复代码。
版本表照 R3 那条流的写法建：带文件的进带文件组，只有意图那一版进「没有文件」组，其余（各批、卸载两次抬 F、挂载写行与暖机、挂载续做各批）不进表。每个根喂四种结局（没有文件、读出第一个文件 3000 字节、读出三单元内容、读出别的），新判法、旧判法各与真值比接受集：

```
name=alternative_summary stream=h1-2dev roots=8 judged_pairs=32 differing_pairs=0 old_differing_pairs=4
name=alternative_summary stream=h2-2dev roots=13 judged_pairs=52 differing_pairs=0 old_differing_pairs=9
name=alternative_summary stream=h3-2dev roots=9 judged_pairs=36 differing_pairs=0 old_differing_pairs=4
```

新判法与真值在 30 个根 × 4 种结局上一格不差；旧判法差的 17 格全是「意图那一版或之后的根报没有文件，旧判法拒、真值收」，H2 的 9 格（实例 1 的意图与两次抬 F、实例 2 的写行、两次暖机、续做三批）：

```
name=alternative_old_differs stream=h2-2dev root=(1, 5) label=unlink_intent outcome=no_file old_accepts=false truth_accepts=true
name=alternative_old_differs stream=h2-2dev root=(2, 8) label=mount_instance_row outcome=no_file old_accepts=false truth_accepts=true
name=alternative_old_differs stream=h2-2dev root=(2, 13) label=mount_continuation_unlink_batch outcome=no_file old_accepts=false truth_accepts=true
```

（上面摘了 9 行里的 3 行，全 9 行在模型目录 `run/roots.out`。）H2 的根跨了实例（实例 1 第 7 代之后是实例 2 第 8 代），按 (txg, 实例) 字典序取「比它旧的最新一项」在这条历史上照样落到意图那一版。

### 谱系分叉那一形：推的，没造出来

要让旧判法对、新判法错，得有一个不在表里的根 R、比它旧的最新一项是「没有文件」的 W，而 R 实际从比 W 更旧的带文件版本 V 长出来。同 txg 两实例是唯一能让 (txg, 实例) 序与谱系序分开的来路：「两条同 txg 不同实例的版本（设备失而复得、或回退实例与被抛弃实例同时在环里会造出来）。」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:4925`）按代码推它在这里放不过去：

1. R 从 V 长出来、V 带文件，R 下面也带文件；正确的实现在 R 上读出内容，而 R 不在表里时读出内容一律红（上面 1152 行）——这一版表跑正确实现就红，写表的人必须把 R 列进带文件组，列进去之后 R 按自己那一项判、报没有文件照红。
2. 所以新判法放过 R 上的「没有文件」只在「R 带文件、却没进表」时发生，而那张表在正确实现上自己先红，不是静默放过。
3. 回退走向前发布（`roll_back_by_a_forward_publish`），回退出来的根 txg 更大、内容是回退目标的，同样落进第 1 条。

这一形我没在内存盘上造（被抛弃根那套搭建 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 在 harness 测试的 `tests/common` 里、用文件镜像，没有删除那几步），记「推的、没量」。

### 「没有文件」组里的版本被读出内容判成立：代码上走不到

一个根同时在两组里，`Layer0VersionTable::new` 当场停：「                "实例 {} 第 {} 代既写成有文件、又写成文件从这一版起没有了",」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1013`）；字段私有、只有 `new` 与 `with_a_file_only` 两个构造口，后者「没有文件」组为空。读出内容落在「没有文件」组的根上只走 1152 行那一支。上面逐根表里每个意图根喂「读出三单元内容」，新判法全拒。

## R2　工作线程上的漏核与重核

### 按代码推的三处

- 工作线程那一支先评状态、再就地调观察者、再记一次：「                tally.observed_states += 1;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2623`）。计数记在这一片自己的 `tally` 里，片评完才发回调用线程；调用线程那一支只给「这一趟跑的片」调观察者（「                            if let Some(observe) = observe_state.as_mut() {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:3570`）），读回的片不再看。两支都在观察者看完之后才写进度文件（「                                file.append_finished_slice(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:3585`））。
- 并完之后核「看过的 = 这一趟负责的」：「    if has_observer {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:3664`）。读回的片的观察者计数来自进度文件（片行带 `observed_states` 与 `observer_counts`），读回时逐片核「有观察者就等于片的状态数」。
- 计划哈希只拼「有没有观察者」：「    message.push(u8::from(has_observer));」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2839`），观察者在哪个线程、观察者是哪个函数都不进哈希。前一半是有意的（文档注释写了两种看法计数相同）；后一半意味着同一条流换了观察者函数而输入指纹不变时，读回的片沿用旧观察者的结论。输入指纹由跑的人从环境变量 `SINGLEFS_LAYER0_INPUT_FINGERPRINT` 给（门禁怎么算它我没查）；这一条 R 之前就是这样，不是 R 带进来的。

### 量：同一段流 13 种跑法逐项比

流：H1 截到第一个文件那次发布之前（取号与两次暖机，7 段、33 个状态，每段展开；全量状态数在预算里，删除那几次发布每次的单元段都在 2^20 以上，见「这条腿自己的限度」）。每片 1 个状态（33 片），观察者每看一个状态记 `observed` 加 1、`digest_sum` 加这个状态持久集合的 FNV 摘要模 1000003（漏一个、重一个和就变，与次序无关），另用进程内原子计数数这一趟观察者真被调了几次。参照是调用线程、1 个工作线程那一趟；`equals_reference` 是 `Layer0Tally` 整体 `==`（状态数、违例、checker 逐条、发现表、观察者计数）。原样输出（去掉 `stream=` 一栏）：

```
name=threads_case case=calling_thread_1 states=33 observed_states=33 invocations_in_this_run=33 observed=33 digest_sum=15049070 equals_reference=true
name=threads_case case=calling_thread_4 states=33 observed_states=33 invocations_in_this_run=33 observed=33 digest_sum=15049070 equals_reference=true
name=threads_case case=worker_threads_1 states=33 observed_states=33 invocations_in_this_run=33 observed=33 digest_sum=15049070 equals_reference=true
name=threads_case case=worker_threads_4 states=33 observed_states=33 invocations_in_this_run=33 observed=33 digest_sum=15049070 equals_reference=true
name=threads_case case=worker_threads_6 states=33 observed_states=33 invocations_in_this_run=33 observed=33 digest_sum=15049070 equals_reference=true
name=threads_resume case=resume_worker4_then_worker6 kept_slice_lines=16 all_slice_lines=33
name=threads_case case=resume_worker4_then_worker6_resumed states=33 observed_states=33 invocations_in_this_run=17 observed=33 digest_sum=15049070 equals_reference=true
name=threads_resume case=resume_calling4_then_worker6 kept_slice_lines=16 all_slice_lines=33
name=threads_case case=resume_calling4_then_worker6_resumed states=33 observed_states=33 invocations_in_this_run=17 observed=33 digest_sum=15049070 equals_reference=true
name=threads_resume case=resume_worker4_then_calling6 kept_slice_lines=16 all_slice_lines=33
name=threads_case case=resume_worker4_then_calling6_resumed states=33 observed_states=33 invocations_in_this_run=17 observed=33 digest_sum=15049070 equals_reference=true
name=threads_panic panicked=true files_left=0 lines_left=[]
name=threads_case case=panic_then_resume states=33 observed_states=33 invocations_in_this_run=33 observed=33 digest_sum=15049070 equals_reference=true
name=threads_case case=two_shards_merged states=33 observed_states=33 invocations_in_this_run=33 observed=33 digest_sum=15049070 equals_reference=true
```

（三种续跑各自的 `_first` 一行同为 33/33/33、`equals_reference=true`，在模型目录 `run/threads.out`。）

- 续跑：进度文件截掉后一半片行（留 16 片），换线程数、换观察者所在的线程再续，这一趟观察者恰被调 17 次（= 33 − 16），合起来计数逐项与参照相同：读回的片没有重看，没读回的片没有漏看；跨「调用线程 ↔ 工作线程」续跑同样相同。
- 观察者在工作线程上故意 panic（第 16 次观察）：整趟红，进度文件被删（`files_left=0`，删文件的守卫是「    let _delete_the_progress_file_if_this_run_goes_red =」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:3443`）），再起一趟从头跑 33 次。没有「panic 之前并进去的片带着没看完的计数留下」这一形。
- 两片分片（第 0 片观察者在工作线程、第 1 片在调用线程）再并：两片合计调了 33 次，并出来与参照相同。

### 没打中，以及这组量罩不住的

工作线程那一支只调观察者一次、调用线程那一支不再看，结构上没有重核；读回、分片、panic 三条路上量到的调用次数与状态数逐个对上。罩不住的：33 个状态、每片 1 个，并发度低；观察者是轻的计数函数，不是 R3 那条流的可写挂载（挂载里有没有跨线程共享的可变状态这组量看不到；我读到的挂载入口参数都由调用方显式传，`deletion_history_switches()` 每次新建，没找到进程级的可变全局，见「这条腿自己的限度」）。

## R3　删除流里 oracle 放过而该红的状态

### 正文点名的两形：都红

oracle 只读三样：结局、实际走的根、盘上最新根槽（表无关的「恢复到旧态」那一支 R 没动）。所以「哪个崩溃状态能让 oracle 放过」等于「哪个（根，结局）对被放过」，R1 那张逐根表就是全集：三段历史 30 个根 × 4 种结局，新判法与真值逐格相同。按格说：

- 「最后一批之后读出了内容」：最后一批的根不在表里，读出内容走 1152 行那一支、红；H1 的最后一批 `(1, 8)`、H2 的续做最后一批 `(2, 13)` 喂「读出三单元内容」，新判法拒（与真值同，逐根表里不差）。
- 「意图之前就报没有文件」：意图之前带文件的根都在带文件组，报没有文件走「这一版有文件却报没有」那一支、红；比第一个文件更早的根（两次暖机）真值本来就没有文件，报没有文件放过是对的。

### 在删除之后的真崩溃状态上逐状态比（全量，每段展开）

删除那几次发布每次的单元段 20–26 个写，任何一次单独全展开就在 2^20 以上，整段 H1 是 1179648159 个状态（`run/roots.out` 里 `layer0_states=`），按定义 3b 的上限跑不了（见「这条腿自己的限度」）。我只取 H2 意图之后、状态数放得进上限的两截，前面的写全部并进基镜像（全持久），截里每段展开，枚举走 R 的新入口、观察者放在工作线程上（同时也是 R2 那条路的一次真跑），每个状态的看 journal 那一遍恢复按三种判法各判一次（新判法、旧判法、真值）：

1. 挂载的两次暖机（实例 2 第 9、10 代；比它旧的最新一项跨实例落到实例 1 的意图）：131095 个状态，分 16 片跑、再并。并出来的原样：

```
name=oracle_tally stream=h2-2dev-after-the-mount_instance_row-up-to-the-continuation states=131095 observed_states=131095 violations=0 ignored_violations=0 failed_states=0 no_file_states=131095 file_read_states=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violated=0
name=oracle_count stream=h2-2dev-after-the-mount_instance_row-up-to-the-continuation counter=consulted_landed_mount_instance_row states=65536
name=oracle_count stream=h2-2dev-after-the-mount_instance_row-up-to-the-continuation counter=consulted_landed_mount_warm_up states=65559
name=oracle_count stream=h2-2dev-after-the-mount_instance_row-up-to-the-continuation counter=consulted_new_none_old_no_file_where_an_older_root_has_one_truth_none states=131095
```

每个状态都报没有文件、新判法放、真值放、旧判法判「更旧的根下面有文件」红；新判法与真值分开的状态 0 个（各片 `name=disagreement` 行数都是 0）。

2. 挂载写行那一次发布（实例从 1 换到 2；卸载两次抬 F 并进基镜像）：262164 个状态，分 24 片跑、再并：

```
name=oracle_tally stream=h2-2dev-after-the-unmount_floor_raise-up-to-the-continuation-up-to-the-first-mount_warm_up states=262164 observed_states=262164 violations=0 ignored_violations=0 failed_states=0 no_file_states=262164 file_read_states=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violated=0
name=oracle_count stream=h2-2dev-after-the-unmount_floor_raise-up-to-the-continuation-up-to-the-first-mount_warm_up counter=consulted_landed_mount_instance_row states=9
name=oracle_count stream=h2-2dev-after-the-unmount_floor_raise-up-to-the-continuation-up-to-the-first-mount_warm_up counter=consulted_landed_unmount_floor_raise states=262155
name=oracle_count stream=h2-2dev-after-the-unmount_floor_raise-up-to-the-continuation-up-to-the-first-mount_warm_up counter=consulted_new_none_old_no_file_where_an_older_root_has_one_truth_none states=262164
```

两截合起来 393259 个状态、40 份分片输出，`name=disagreement` 行数全是 0。落到实例 2 写行那一版的只有 9 个状态（写行的根落了之后那几段），其余落在实例 1 最后一次抬 F 上。

### 删除那几版上 oracle 看不见什么、谁替它看

冷走读只找第一个文件那条记录、nlink = 0 就答没有文件，不往下读 extent 与数据单元：「    if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-core/src/recovery.rs:5077`）。所以意图到最后一批之间，「size 没按批改」「释放错了单元」这类错在 oracle 眼里一律是「没有文件」、放过。这是 D8 已定项 16 第 4 条要的读法（「4. **读者与写入口**：`InodeRecord` 的 nlink 由 `parse` 读、`to_bytes` 写结构里的值；现造记录的入口拒「nlink = 0」与「没有这条记录」；从意图提交起只读挂载、冷走读、模型读回答「没有文件」；写入口拒 nlink = 0 的 inode（`research/prompts/m3-delete-r2-main-verification.md` 第二节第 1 行、第三节 V6）。」（`.claude/kb/decisions/08-核心索引结构.md:468`）），不是 R 的毛病；替它看的是池级 checker 的 I-4.6（「/// I-4.6（中间态其余不变量成立） 取乙：每条 nlink = 0 的记录，extent 树里它的数据单元数与 size 对得上——size 为 0 时 0 或 1 个，」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker/src/image.rs:251`））、I-4.7、I-9.5、I-9.19。层 0 的计数把「部分没判」「不适用」记成不红（「            InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1635`）），R3 那条用例只核违例数是 0、不核这几条真判过，我就取样看它们在删除那几版上是不是真判：H1 整段状态序号上均匀取 100 个再加全持久那一个，逐个判（`verdicts` 模式，原样摘落到意图与某一批的两格）：

```
name=verdict stream=h1-2dev landed=unlink_batch invariant=I-4.6 verdict=holds states=1
name=verdict stream=h1-2dev landed=unlink_intent invariant=I-4.6 verdict=holds states=1
name=verdict stream=h1-2dev landed=unlink_intent invariant=I-9.5 verdict=holds states=1
```

落到删除那几版的只取到 2 个状态（均匀取样大多落在大单元段里、走回之前的版本），四条删除类不变量都是 `holds`、没有 `not_applicable` / `partially_judged`；落到第一个文件、顺序写那几版时 I-4.6、I-4.7、I-9.19 是 `not_applicable`（那几版没有 nlink = 0 的记录，照不变量自己的写法）。2 个状态只够说「删除那几版上它们在判」，不够说「每个删除态都判」。

### R3 那条用例的逐状态核

`observe_the_deletion_cut_in_the_middle` 按「单元落了至少一个、记录一份都没落、根没落」认中间态，意图读回 C、各批答没有文件；记录落了一份而根没落时走 journal 落到这一版的根，oracle 照这一版判（意图起没有文件）、mount 观察者照这一版数续做步数（`continuation_steps_from_this_version`）。我没找到 oracle 与这两个观察者都放过、而 D8 已定项 16 第 4、6 条要红的崩溃状态。可写挂载那一层（续做步数、挂完是不是「有树、文件集合为空」）只在 checker 档用例里核，我没跑。

## 没打中的形状

| 攻击面 | 试过的形状 | 取样范围 | 结果 |
|---|---|---|---|
| R1 | 新旧判法逐根逐结局比，版本表照 R3 的写法（只有意图进「没有文件」组） | H1、H2、H3 三段历史共 30 个根 × 4 种结局（没有文件、读出 3000 字节、读出三单元、读出别的） | 新判法与真值 0 格不同；旧判法 17 格不同，全是「意图起报没有文件」，新的对 |
| R1 | 「没有文件」组的根上读出内容 | 同上，每个意图根喂两种内容；代码两处（1013、1152 行） | 全拒；同根两组在建表时停 |
| R1 | 谱系分叉（同 txg 两实例、回退）让旧判法对 | 按代码推，没造（见 R1 节） | 推的：表在正确实现上先红，不是静默放过 |
| R1 | 跨实例取「比它旧的最新一项」 | H2 实例 1 → 2，逐根 + 两截全量 393259 个状态 | 与真值相同 |
| R2 | 1/4/6 个工作线程、调用线程 1/4 个，计数逐项比 | 33 个状态、每片 1 个 | 全等 |
| R2 | 续跑：截掉一半片行，换线程数、换观察者所在线程 | 3 种组合 | 观察者恰调 17 次（= 33 − 16），合起来全等 |
| R2 | 工作线程上的观察者 panic，再续 | 第 16 次观察 panic | 进度文件被删、从头重跑，全等 |
| R2 | 两片分片（两片观察者各在一种线程上）再并 | 2 片 | 合计 33 次，全等 |
| R3 | 「最后一批之后读出内容」「意图之前报没有文件」与其余每个（根，结局）对 | 同 R1 第一行 | 该红的都红 |
| R3 | 意图之后的真崩溃状态逐状态三方比 | H2 挂载写行一次（262164 态）、两次暖机（131095 态），每段展开 | 0 个状态新判法与真值分开 |
| R3 | 删除那几版上 checker 的删除类不变量是不是真判 | H1 均匀取 101 个状态，落到删除那几版 2 个 | 都是 holds |

用户决定的那几步怎么放开的：三段历史（H1 意图与每批同进程、H2 意图后正常卸载再挂载续做、H3 旁边多建一个 size 0 inode、意图时间晚 60 秒）就是 `run_deletion_history` 给的三种用户动作次序；R3 那条流自己的前缀（多记录流 A、B、C）我没录，H1 的前缀是新池新建文件 + 顺序写成三单元，删除段与 R3 那条流同形（意图、两批、最后一批，批大小 1）。回退之后先做什么、删完之后再建 size 0 inode、空发布这几种后缀我只按代码推（R1 节第 1–3 条），没造历史。

## 这条腿自己的限度

- **删除那几次发布本身没有全量枚举。** mkfs 只认两盘（冻结副本 `crates/singlefs-core/src/make_filesystem.rs` 第 477 行的断言消息「第一版跑 2 块盘」，一盘的几何录不出来），两盘时意图、各批的单元段 20–26 个写，任何一次单独展开就在 2^20 以上，整段 H1 是 1179648159 个状态，超过定义 3b 一次约 10^6 的上限。崩溃状态不缩，所以我改缩历史：只枚举 H2 意图之后两截（写行 262164 态、两次暖机 131095 态），前面全持久并进基镜像。意图、各批、最后一批「崩在中间」的状态一个没跑；它们落到的根都在逐根表里，oracle 那一侧由逐根表罩住，系统那一侧（恢复是不是真读出该读的）没量。
- **缩历史的经过与估时。** 先取 H2 意图之后到挂载续做之前整截（524360 态）：第 0/24 片 21856 态挂钟 138 秒，估全量 24 × 138 秒约 55 分钟，超过 40 分钟；缩到卸载之后（393258 态）第 0/16 片 24592 态 204 秒，估约 54 分钟；再缩到挂载写行一次（262164 态）第 0/12 片 21856 态 233 秒（那时机器满载，约 30 个核被别的会话占着），估约 47 分钟；改取两次暖机（131095 态）先单线程均匀取 60 个状态量每态 0.0319 秒、估 6 线程 697 秒，16 片实跑每片 44–116 秒；之后机器轻了，写行那一截 24 片实跑每片 45–155 秒、合计 1333 秒（约 22 分钟）。上面三次没跑完的首片（共 68304 态，外加一次跑到 3600 态时我停掉的）不算证据；其中两份的输出留在草稿目录（见「草稿与仓外产物」）。
- **观察者里砍掉了不看 journal 那一遍的三方比。** 每个状态再恢复一遍要多三成挂钟（估时超过 40 分钟那一步砍的）；不看 journal 那一遍只由枚举自己按 R 的版本表判（两截 `ignored_violations=0`），没与旧判法、真值逐状态比。逐根表对两遍都成立（它不看 journal 策略）。
- **R2 的量只有 33 个状态。** 再大的流在预算里放不进删除发布；观察者是轻的计数函数，没在工作线程上跑可写挂载。
- **checker 删除类不变量只取到 2 个删除态。**
- **跑了多少状态（定义 3b 的总账）：** 全量跑完并入证据的 393259 态 × 2 遍（开发时一遍、`rerun.sh` 从空目录一遍）= 786518 态；没跑完的首片 68304 + 约 3600 态；R2 的 33 态流各种跑法合计约 1000 态；`verdicts` / `estimate` 两种取样模式共约 580 个单状态判定。合计不到 9 × 10^5，在约 10^7 之内；每一次全量（各截）的闭式状态数都先打印、都不超过约 10^6（最大 524360）。
- 模型只在我的副本上编过、跑过；副本上的数不进 kb。
- 跑法上的两处出格：开发与复跑里各有几条前台命令超过 240 秒，被工具自动挪到后台跑完（不是我用 run_in_background 起的；其中一次我没等到它结束就又起了同一片，见「复跑核对」）；原型 `main.rs` 第一段新建时一次写了 159 行，超过每次 150 行。

## 没做什么

- singlefs-checker-tier 的测试一个没跑（含 R3 那条用例的快档与全量、R4 变异），只编了我自己的原型 crate、只跑它。
- K1、K3 不归这条腿；R4（变异行与锚点）没看。
- 谱系分叉（同 txg 两实例、被抛弃根与删除叠在一起）、回退之后再删、删完之后再建 size 0 inode 这几种用户动作后缀只按代码推，没造历史。
- 可写挂载那一层（R3 用例的 `observe_the_mount_continuation`）没在我的原型里复刻。
- 没改法可交：四个攻击面都没打中。

## 复跑核对

`rerun.sh` 在空目录 `~/.local/share/singlefs/work/m3-batch33r-code-r1-opus/rerun1/` 从 `build` 起分阶段跑完（每条命令按四分钟以内排，三条超了、被工具挪到后台跑完；其中 instance-row 第 20 片第一次被我同时起了两份、写同一份进度文件，删掉那一片的进度文件与账本后单独重跑，`LAYER0_RESUME … restored_slices=0/16386`），再拿 `compare.py` 与开发时的输出（按 `rerun.sh` 的文件名改名拷到草稿目录 `dev-renamed/`）比，原样末行与开头一行：

```
✓ oracle-instance-row-merge.out：30 行相同
compared=45 differing=0
```

全文在模型目录 `compare-run-vs-development.txt`（45 行 ✓）。`run/` 是这一次复跑的输出原样（不含 `build.out`）。

## 草稿与仓外产物

- 草稿目录 `~/.local/share/singlefs/work/m3-batch33r-code-r1-opus/`：开发时各次跑的 `.out`（含两份没跑完的首片 `oracle-h2w-s0.out`、`oracle-h2m-s0.out`；写行那一截分 12 片时的首片与暖机那一截分 10 片时被我停掉的那一片，输出被后来的同名文件盖掉了，耗时只记在「这条腿自己的限度」）、`dev-renamed/`、`progress.md`、`rerun1/` 里的分片账本与进度文件。冻结副本的拷贝（`tree/`、`rerun1/tree/`）与编译目录（`target/`、`rerun1/target/`）交回前删了，复跑用 `rerun.sh` 从冻结副本重建。
- 没有要入 `research/results/` 的产物：这一轮的输出都是腿的模型输出，随模型目录 `run/` 入仓。
