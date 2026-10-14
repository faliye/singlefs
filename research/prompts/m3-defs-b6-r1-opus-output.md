# m3-defs-b6-r1 攻方腿（Opus）报告（2026-10-13）

攻击面：正文「一」表 V1、V3、V5、V6、V7、V8；第一轮分工（过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱）加「三、要答的」各格专问。
对象：零轮形态冻结副本 `~/.local/share/singlefs/work/m3-defs-b6-r1/repo/`（下称 b6）；零轮之前 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/`（下称 b3）。两份都只在草稿目录的拷贝上跑。

## 复跑

整条经内存包装（不起 cargo、不起重型测试、不跑门禁阶段；门禁格只抽格里的 python 原样跑）：

```
PRE_ZERO_FROZEN=$HOME/.local/share/singlefs/work/m3-defs-b3-r2/repo \
  bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b6-r1-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen $HOME/.local/share/singlefs/work/m3-defs-b6-r1/repo --work <工作根底下新的空目录>
```

`--work` 要在工作根（`~/.local/share/singlefs/work`）底下：V5 的「仓副本」样本靠它。末尾逐份打 `same` / `DIFF`（与 `out/` 比，路径换成 `<WORK>`、`<MODEL>`）。腿从仓里的模型目录最后复跑一次（`--work` 在草稿目录，上限 2G；8G 那一次在包装里排队 305 秒没起来、腿停了它）：21 份里 20 份 `same`，`v8-rev12.txt` 差的是这一次新加的两行 long-run-check，存进 `out/` 的就是这一次的输出。挂钟没计（前台 240 秒没跑完，四分钟以上），大半是拷两份冻结副本。
门禁格四条要主 agent 跑：`gate-samples/要主agent跑.txt`。

模型目录 `research/prompts/m3-defs-b6-r1-opus-model/` 每个文件的 sha256：

```
5db693256f0c454cd5db7db7edc12f6cca1df2e175f94e8aa9a3a2ec26024a89  ./feeds/v1/base-investigator.txt
30b2e3124cb94dd2fe4f6d832a1dd0eb7f8d6848b2510ca1dd39e8237df44625  ./feeds/v1/base-over2h.txt
00a3432836c343f63e58bde5985fae2a6e9add3977713ad8882c0d3a8dee533d  ./feeds/v1/base.txt
6920c0e108a0bff4dd2581a9ad58d93c0fdc5ec5421e30d81afc1ad79bf911d6  ./feeds/v1/variants-hosts.tsv
78034c61f796f7ef9eb88a3aba0cbd6b055fc8dd1265d3c8442f3fd5614ff355  ./feeds/v1/variants.tsv
0c92936ba1d7ee909877e746c68d29d4d0d8b1fbfdd2f53de630ced246d92af7  ./feeds/v3/sample/.claude/heavy-thread-default
3231f3efc1498c0997387135c149fabc31abc2723638cf78d3c1f57e5e235bd3  ./feeds/v3/sample/crates/demo/src/harness_wrapper.rs
f02b3a2a24d2d6be771c762bb36d3ffb31a0436f9f06246c2d4678bd0dc0fc64  ./feeds/v3/sample/crates/demo/src/lib.rs
fc9bd82d72ad7e1a9e60696dee085622d62c348b1757c28d59db6023c72001ee  ./feeds/v3/sample/crates/demo/src/project_wrapper.rs
4c5450ba8c8910f0a8017b89b31213d281ee5c3263be1ac055b6560a5a6341f0  ./feeds/v3/sample/crates/demo/src/tests.rs
78d1273421a56d2672895fbf75e83db7e7716597a2760c936d748a39ba9b0dd2  ./feeds/v3/sample/crates/demo/src/use_alias.rs
be9f49020cf18a169d9da06101abe8646cd79816db651a881cca9f286ff5a7a2  ./feeds/v3/sample/crates/demo/src/vars_iter.rs
fd713452409eba8d805a59edecaa9a4b8cbf4527d8ff1e2c21ac6200aed3bb16  ./feeds/v3/sample/scripts/experiments/alias_env.py
ad131d17d5c0d7235b67aeaf67153e2ea292bd3ccac0cde6fead8f29274d3d60  ./feeds/v3/sample/scripts/experiments/baseline_read.py
1c40b302488c91bdf83b38aae90cdd3ea9ebf221398a8185bd0f6a271a470b10  ./feeds/v3/sample/scripts/experiments/copied_env.py
0307030d3f491c74a9f51b8a7051d0b6c38d0ec1d831293257f5895496f921e5  ./feeds/v3/sample/scripts/experiments/wrap_helper.py
15b2f086fa9206e31521fd1f5cdfcbd5902f7829eb0cbcb0422446f965984b6e  ./feeds/v3/sample/scripts/hosts/and_default.sh
77314fde542c805dcdad14d16707944fdde4e8bfe8bc90e863a498b0d8004b73  ./feeds/v3/sample/scripts/hosts/capped.sh
daff66fe20eb64f1c0b46df188343237f5d1b7873f392680fa513615e0b8c2d9  ./feeds/v3/sample/scripts/hosts/heavy_threads.py
13a8b1fa23b4e143b5e134c85a53a4e7d4f9f6db35ef0b843a1b07f88ae84a9d  ./feeds/v3/sample/scripts/hosts/if_default.sh
1700a97d6657ea443ea7088278e0ba8480de3d25d1f3eb1415caf3865757c27e  ./feeds/v3/sample/scripts/hosts/or_default.sh
83584509c8107ec0c4004d7f87386eac6358cb796bdc028c6257e6eca2fcda65  ./feeds/v3/sample/scripts/hosts/printenv_read.sh
ca2a37c8add6e0441fbc4c46e1a63bdaa7e460f3b6caf934ce0cb095ab0c9db8  ./feeds/v5/s6-main-agent.json
812a68b1b92f2c023322a234657b7faae95cc7cf53993fd3f3181b62b489828e  ./feeds/v6-implementation-writer.txt
3c81b5c469a8f878019b5c14641ca3dba175ab9ced583c76459e98309f4fb9c7  ./feeds/v8/run_segment_one_revision_12_multi_host.sh
09bfd328e2870ec13bd48bc41d21b2551f45379ad674fd2727fec06e36657bf7  ./gate-samples/doc-process-records/abandoned-legacy-dispatch-date/records/2026-10-13-样本调度.md
ed03d68a1646471434ed3ef0a7620c9ebe64fb7fa027040e1a76ca1417711e38  ./gate-samples/doc-process-records/abandoned-legacy-dispatch-date/research/prompts/abandoned-rounds.tsv
7ba7fd9b2bba0af5b1d11855d0f293cbccfb059cc718130968d7325e3e4fbe82  ./gate-samples/doc-process-records/abandoned-legacy-dispatch-date/research/prompts/_old-topic-r1-body.md
0b6f0c7fdbcd16e6580496eb4dd864632a95fd69facb3068979dbee3b25e551d  ./gate-samples/doc-process-records/abandoned-legacy-dispatch-date/research/prompts/old-topic-r1-opus.md
c19ed58f6e6ca0ece29170993d1e5defb90fb17427ec08b39899f9cb731e17c2  ./gate-samples/doc-process-records/abandoned-legacy-dispatch-date/research/prompts/old-topic-r1-snapshot/frozen-copy-sha256.txt
3c46e4c1c565e94ad3773484ed8cf356be2fd09746227e2b9836efb6d73df30b  ./gate-samples/doc-text/pv-tar-mirror-red/expect
7dab0c219d3c0182005f809f212947f8672460788db80199e38621a142eae579  ./gate-samples/doc-text/pv-tar-mirror-red/.gate-cells
a0daed5b08813f2c3a70acbd198bd08b3e2f99c12bfe91ec18174e6b55908499  ./gate-samples/doc-text/pv-tar-mirror-red/multi-host.env
a619bc5ee4432588bc22a81b435f4d01b14e3f90361f87075dfb0169bf3ddc24  ./gate-samples/doc-text/pv-tar-mirror-red/.private-values-hostname
f780ee6484718842da3570e69f99ebb223c4bcbe6c914ab4bf093cd31c79eb04  ./gate-samples/doc-text/pv-tar-mirror-red/setup.sh
e8f63dffd09a3bbbf2d2fe55ee1726cfaa95f0050458d84094b304bdc77c7435  ./gate-samples/要主agent跑.txt
dc214deb47884fe4c7a31c0f7766d0e570a04edddafba4e107659172dd7d4f72  ./out/v1-b3-clerk.txt
e18fb57d101d6fe99cd781dc74636009ec5992f3f38bf476d27010ff1873d4f4  ./out/v1-b3-investigator.txt
417cf8d8074da9bb06adc33c7d036076e942f02b0ad1fae302284bb138158de4  ./out/v1-b6-clerk.txt
1f3d2ef4df2d84f6837fdff744b305ffaa9d24dbc0b89d85f85b802d13aa8cc2  ./out/v1-b6-investigator.txt
fac95b2cd27d62834938cb3ddf76137e5361ac45fa832cb5c295c891f8b136ac  ./out/v3-b3.txt
4721811d3defd87086cf908f7e40ce23116bd53c3602bb7858644da0bc9613c2  ./out/v3-b6.txt
9da71ae9dfaefcae3cc8c50a685239f60184e2a117803935ce9637e4107fa677  ./out/v5-chdir-b3.txt
dd6fd8feb4e8b56a40393b7734e912ce044332484b2758c1cf29ec05ad84c5f7  ./out/v5-chdir-b6.txt
09491fd07552aca12a2b6bdc3b86e4e5a598998103d3c62d9be40a1292de821b  ./out/v5-copy-b3.txt
214a941c2717185bcb2bed4069b022663c54bfab33e454ef37aee6c43abc5f4c  ./out/v5-copy-b6.txt
7efaa4fc65e988011630771779db120e00798d80748991473b3ed065deb6e953  ./out/v6-guard-b3.txt
5f5e44fa82c7b9bc956f56c53ac7b6e35fcd5e1dc1e7391e57c786779296177d  ./out/v6-guard-b6.txt
12e2b17e437e3a955b37a5d3fabfea3a53b65ee8ae5980af704c1c103979b3cf  ./out/v6-lib-b6.txt
5fbe9ea62e9fa634a97ab20514d15acae7ab09f853c49c4d4c562f4defbcf533  ./out/v7-b3.txt
897bb4ba833713f4ef169c6f64600d9ac657b263ac465eacb231349753923e54  ./out/v7-b6.txt
d298b851fc17b3b1197bfccdc8c9080618352c1f762689d952bb1c9c56e9f21b  ./out/v7-path-probe.txt
b4a24830796120e08aca87b4ad55793a8ca13e0a64eecea0c0712d70d57f9295  ./out/v8-hosts-b3.txt
310738e0c6c579be1a6a4d82df885d4bbb3bf92061e7e095955e351b5fb2be55  ./out/v8-hosts-b6.txt
be6718cfa32494a872aa02be9b8ad11a4722b84a6df1a7bbd11bd4de4dc5dd3b  ./out/v8-hosts-reasons-b3.txt
dc248ec5d5f7c25dc4e54bcf17f40d77733d78c204eadbbd57051e5de2e5009c  ./out/v8-hosts-reasons-b6.txt
6d8e924df89cd0460f3bfafa9f8467ef38a60308d3e16997f645c93302ef8d5d  ./out/v8-rev12.txt
db5f773af04dfb9d082939c0ec358ef8ba7365ba7d23452ee4d97345aae37ced  ./tools/pv_dir_scan.py
b1bbfc101f8754a780cdd5996b6e9de7698683e65d57170b964da9247cc90aec  ./tools/pv_path_probe.py
8c0622297d40557a0133b9ca9335c2d27cced3dcfbb338916c47faf7dd3059be  ./tools/rerun.sh
94646c94102cdfa4483a888399b582880fb7a18a17ab7571832b9f08398e2849  ./tools/run_htd_cell.py
341c53331a57ac0d4bef47400e351453022c60505aa30faa0b20d0a48d573859  ./tools/run_pv_cell.py
6d15aa2c8490c95579eb9db1a68cac55cfa9e67ff8d3a62dfec0b6a4e8ffa48a  ./tools/v1_feed.sh
67608231a8b526e650cbcaf1804c2746497d4eb06a18b6eafeb58b8be440813b  ./tools/v5_chdir_feed.sh
7acb3bc274fbbea07ce9e55ab2c915cf02317cca1b321910aafa11edc6ac754d  ./tools/v5_feed.sh
04df3e3ebac5f38cb5cf69c6c6574c31b2e5d8d4a18c08223aabb13900434902  ./tools/v7_big.sh
32d9e55bdd664371d48d2f45792e7dd133cb52e266338a76aa94fdddbd64322d  ./tools/v8_rev12.sh
```

## 各格判定一览

「量过」= 在冻结副本拷贝上真跑了钩子入口或抽出的判法函数，原样输出在模型目录 `out/`；「推的」= 按代码推，没跑。

| 格 | 判定 | 打中的那一步（一句） | 零轮之前（b3） | 量过 / 推的 |
|---|---|---|---|---|
| V1-甲 | 打中（钩子拒而定义没写，误伤） | ⑮ 括注写定义自己的原话「（全仓唯一的登记位）」「（双机时两台各取这一个数）」「（照登记值，第二台同）」都拒 | 放行（这一形是零轮新造的） | 量过 `out/v1-*-investigator.txt` A、R、L4 |
| V1-乙 | 打中（定义说拒而钩子不拒） | ⑮ 括注「（机器忙时按负载降）」「（忙闲看空闲核数再定）」「（小机器对折）」「（双倍）」「（第２台１６）」「（另台 sixteen）」都放 | 同样放（改法碰不到这几格） | 量过 D、D2、I2、I3、I4、E、H2 |
| V3 | 打中（零轮形态更糟） | 装置经包装函数读自己的线程变量（仓里现成的 `count_from_environment` 写法、harness 的 `process_environment_variable`、`ENV = os.environ` 别名、`std::env::vars()`、`use std::env::var as …`）与 shell 的 `[[ … ]] \|\| X=8`：九种写法零轮那一版一处都不点名 | 九种都点名 | 量过 `out/v3-*.txt`（判法函数）；门禁格要主 agent 跑 |
| V3-乙 | 打中（两版都漏，改法碰不到） | rust 文件头部 `#[cfg(test)]` 接 `mod tests;`（只是声明）之后整份不扫，后面读线程变量不点名 | 同样漏（b3 见 `#[cfg(test)]` 就停） | 量过 lib.rs 一格 |
| V5-甲 | 打中（改法碰不到） | 副本里只改入口脚本 import 的库（`lib_heavy_tests.py`、没登记的 `lib_shell_words.py`），入口脚本与主仓 sha256 相同，子 agent 在副本里跑入口的 `--selftest` 放行 | 同样放 | 量过 `out/v5-copy-*.txt` S1、S1b、S4 |
| V5-乙 | 打中（新矛盾：一处收一处拒） | 照拒绝说明「由主 agent 在集成副本里跑」，主 agent 在副本里跑改过的脚本的自证也被拒，出路只剩带 `SINGLEFS_HEAVY_TESTS=commit` 或弹窗 | 主 agent 放行 | 量过 S6、S7 |
| V5-丙 | 打中（定义说拒而钩子不拒） | `env -iC <checker 档目录> cargo test` 放行（`-C` 与别的短选项合写）；`env -S 'cargo test -p singlefs-checker-tier'` 放行 | 两样都放（后者另立账） | 量过 `out/v5-chdir-*.txt` C2、C8 |
| V6 | 打中（内部矛盾，⑥ 一侧不收） | 登记表表头写日期是「腿发出那天」，判法按「规矩立起来之前登记的」免出处：10-12 之前发腿、之后叫停的轮，门禁格不要出处判绿，⑥ 不收、照挡写回员与实现员 | ⑥ 同样不收；门禁格要主 agent 跑 | ⑥ 与判法函数量过 `out/v6-*.txt`；门禁格推的 |
| V7 | 打中（零轮形态更糟：值上了红行） | 私有值本身带 `/`（真配置 8 个词里 6 个带 `/`），路径命中时逐段打码打不到，红行把整条路径连值原样打出；从第二台 tar 一个目录过来就是这形 | 不比路径，退 0、不打 | 量过 `out/v7-*.txt`（判法函数，假值样本）；门禁格要主 agent 跑 |
| V8-甲 | 打中（零轮形态更糟） | E171 修订 12 入口拷修订 11、只把 `# long-run-hosts:` 改成登记值说法：long-run-check、exempt-check、heavy-thread-defaults 三道都过，正文照旧起按 available_threads 现算的冻结编排；原样拷（老实写现算）的反被拒 | 正相反：老实的放、改头的拒 | 量过 `out/v8-rev12.txt` 与 `detached-run.sh long-run-check` 退 0 / 退 5 |
| V8-乙 | 打中（两头：误伤与漏） | ⑭ 多机一行写定义原话「不看忙闲」、或「第二台在跑别的会话也照这个数」「两台核数不同也一样」都拒；写「登记值的一半」「按此刻余量少给」「（31）」都放 | 七种都放 | 量过 `out/v8-hosts-*.txt` M0–M7 |

三问的答：

- 过度外推与越位：V1-甲、V8-乙前半（词表按字拒，连定义自己的原话一起拒）；V5-乙（拒绝说明把活推给主 agent，钩子又不让主 agent 平常跑）。
- 内部矛盾与循环依赖：V5-乙（拒绝说明 ↔ 钩子）；V6（表头的日期 ↔ 判法的日期）；V8-甲（豁免表只缩不涨 ↔ 冻结层一个字节不改 ↔ 文件头必须写登记值，三条一起只剩「改头不改身」一条路过得去）；⑭ 与 ⑮ 两张词表不一（⑮ 拒「一半」⑭ 放，⑭ 拒「忙闲」⑮ 放）。
- 最脆弱的一条：⑭ / ⑮ 的词表（V1、V8-乙）——最自然的写法「登记值的一半」就过 ⑭，定义自己的一句「不看忙闲」就被 ⑭ 拒。后果最重的一条是 V7（门禁自己把私有值打进输出，输出照「贴原样输出」会被抄进仓）。

## V1　⑮ 括注的词表与九种类型

被判的字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`：「只写说明性的词；半角括号不认」；钩子的判法是一张按字拒的表：「THREAD_LIMIT_NOTE_NUMERALS = re.compile(r"[〇一二三四五六七八九十百千两半]|分之|合计|减|各")」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:991`）。

喂法：`tools/v1_feed.sh <仓> <类型> feeds/v1/variants.tsv <输出>`，每格 = `feeds/v1/base*.txt` 加一行，经派发闸 `--check-prompt`（与 stdin 入口同一份 `judge_dispatch`）判。clerk 与 investigator 各跑一遍（b3 里 clerk 不在必写表，零轮前后比用 investigator）。原样输出 `out/v1-b6-investigator.txt`、`out/v1-b3-investigator.txt`，摘几行：

```
A	rc=2	✗ 派 investigator 的提示写了「线程上限：31（全仓唯一的登记位）」（括注里有数词或比例词「一」：括注只许说明性的词（「本机」「登记值」这类），第二台的数、减半、合计、各台各取多少都不写进括注）；这一行只认一个整数、后面至多接一段中文括注，整数要等于登记的每台默认重活线程数 31（.claude/heavy-thread-default）。
R	rc=2	✗ 派 investigator 的提示写了「线程上限：31（双机时两台各取这一个数）」（括注里有数词或比例词「两」：括注只许说明性的词（「本机」「登记值」这类），第二台的数、减半、合计、各台各取多少都不写进括注）；这一行只认一个整数、后面至多接一段中文括注，整数要等于登记的每台默认重活线程数 31（.claude/heavy-thread-default）。
D	rc=0	  ✓ 派发闸照这份提示放行（investigator，查了 8 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）
I3	rc=0	  ✓ 派发闸照这份提示放行（investigator，查了 8 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）
L4	rc=2	✗ 派 investigator 的提示写了「线程上限：31（照登记值，第二台同）」（括注里有数词或比例词「二」：括注只许说明性的词（「本机」「登记值」这类），第二台的数、减半、合计、各台各取多少都不写进括注）；这一行只认一个整数、后面至多接一段中文括注，整数要等于登记的每台默认重活线程数 31（.claude/heavy-thread-default）。
```

（b3 同一批：A、R、L4 都 rc=0；D、D2、E、H2、I2、I3、I4 两版都 rc=0。）

### V1-甲　定义自己的原话进括注被拒（量过）

历史：主 agent 派 tooling-writer，照 main-agent.md 第 53 行的说法给 N 加括注，抄同一行里的话「全仓唯一的登记位」或「双机时两台各取这一个数」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`：「双机时两台各取这一个数」），写成「线程上限：31（双机时两台各取这一个数）」。⑮ 拒（rc=2，原因「括注里有数词或比例词「两」」）。

四句：分辨臂——分：白名单（括注只许「本机」「登记值」「默认」这类词）不中，按字的黑名单中；看得到——看得到，括注原文就在提示里；分句——跑前条款「钩子拒而定义没写」：定义只说不写「中文数词」，「唯一」「两台各取这一个数」说的是同一个数，不是数词；改法——没有跑前改法；我提的 F1（白名单）在这一格上不中（推的）。

结局是多拒（fail-closed）：主 agent 改写括注就派得出去，不违反用户定案，代价是一次被拒。

### V1-乙　括注写「看负载」与不带数词字的比例，⑮ 放（量过）

历史：机器忙，主 agent 想让子 agent 别吃满，又记得「N 写登记值」，写成「线程上限：31（机器忙时按负载降）」或「线程上限：31（小机器对折）」。⑮ 放行（rc=0）。子 agent 照共用约束「N 照派发提示的「线程上限：N」」起命令，括注里那句话它看得到，按负载降就违反用户定案（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`：「机器忙时也固定这个数，不看忙闲」）。

同一批里 ⑭ 对多机一行另有一张现算词表（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_long_run.py:78`：「COMPUTE_PHRASES = re.compile(r"available_parallelism|nproc|除以|在跑|核数|忙闲|空闲|负载")」），⑮ 的括注不用它；反过来 ⑮ 拒的「一半」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:990`：「THREAD_LIMIT_FORMULA_MARKS = re.compile(r"÷|/|一半|除以")」）⑭ 不拒（V8-乙 M6）。两张表各管一半。

四句：分辨臂——分：白名单不中，黑名单（零轮形态）中；看得到——看得到；分句——「定义说拒（只写说明性的词）而钩子不拒」，后果按「照定义做却违反用户定案（固定不看忙闲）」那一句；改法——零轮改法（加数词字）在 D、D2、I2、I3、I4、E、H2 七格上照样中，「对这几格不起作用」。全角数字「第２台１６」中是因为判数字用 `[0-9]`，括注里的全角数字不算数字。

已攻过、这一轮不再交：第二台另起一行（`第二台线程上限：16`）与「线程上限已判：」理由写负载（上一题二轮攻方 A4、A5），两版都 rc=0（`out/v1-*` F、G、G2、W、W2），零轮形态没让它更糟。

### 专问：九种类型里哪种不带重活却被逼写

three-way-forward、three-way-defense 的多数轮只读源码、喂钩子（本轮正推腿就是），clerk 的一次性活多是数行、改表。三种都必写「线程上限：」（缺行 rc=2，`out/v1-b6-clerk.txt` X、Y），出路是照写登记值或写「线程上限已判：」放行行。钩子拒绝说明里放行行的理由包括「或确实不起重活」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:1079`），main-agent.md 第 53 行只列「用户给了参数、或内存装不下要用别的数的」：定义没写「不起重活」也能用放行行。代价是一行字，不算打中，记「文字没写到」。

## V3　heavy-thread-defaults 读自己线程变量那一路

被判的字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/code-tooling.sh:1872`：「引号里的名字只在读环境变量的写法里才算读」；shell 局部变量的认法「SHELL_ASSIGNMENT = re.compile(r"(?:^|[;&|])" + SHELL_ASSIGNMENT_BODY, re.M)」（同文件第 2004 行）；测试模块的认法「RUST_MODULE_ITEM = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?mod\b")」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/code-tooling.sh:2007`）。

喂法：`tools/run_htd_cell.py <门禁根> <被判树>` 从门禁根的 `code-tooling.sh` 抽出这一格的 python 与 `CODE_WITHOUT_TEXT_LIBRARY`，在被判树里原样跑（基准状态 none）。被判树是 `feeds/v3/sample`（拷了登记位与 capped.sh，另放 14 份装置，`find feeds/v3/sample -type f \( -name "*.py" -o -name "*.rs" -o -name "*.sh" \) ! -name capped.sh ! -name heavy_threads.py | wc -l` 得 14）。每份都读一个 capped.sh 不设、也没回退到登记值的线程变量；除 `tests.rs`（`lib.rs` 里 `#[cfg(test)] mod tests;` 声明的测试模块文件，两版都点名，算误报）外，照格的文件头都该点名；`baseline_read.py`、`if_default.sh` 是对照，两版都点名。原样输出 `out/v3-b6.txt`（零轮那一版的判法）：

```
     crates/demo/src/tests.rs:3  读自己的线程变量 SAMPLE_IN_TEST_FILE_THREADS：capped.sh 不设它，读它的那条语句里也没回退到 SINGLEFS_THREAD_CAP 或登记值  「let _ = std::env::var("SAMPLE_IN_TEST_FILE_THREADS");」
     scripts/experiments/baseline_read.py:3  读自己的线程变量 SAMPLE_BASELINE_THREADS：capped.sh 不设它，读它的那条语句里也没回退到 SINGLEFS_THREAD_CAP 或登记值  「THREADS = int(os.environ["SAMPLE_BASELINE_THREADS"])」
     scripts/hosts/if_default.sh:2  读自己的线程变量 SAMPLE_IF_FALLBACK_THREADS：capped.sh 不设它，读它的那条语句里也没回退到 SINGLEFS_THREAD_CAP 或登记值  「if [[ -z "${SAMPLE_IF_FALLBACK_THREADS:-}" ]]; then SAMPLE_IF_FALLBACK_THREADS=8; fi」
```

（另两行是样本里拷来的 capped.sh、heavy_threads.py 的 nproc / cpu_count，样本没有排除表；两版都有，不算。）`out/v3-b3.txt`（零轮之前的判法，同一棵树）多点名九份：

| 装置 | 写法 | b6 | b3 |
|---|---|---|---|
| `crates/demo/src/project_wrapper.rs` | 照仓里 `random_histories.rs` 的包装：`count_from_environment("SAMPLE_COUNT_WORKERS", 8)` | 不点名 | 点名 |
| `crates/demo/src/harness_wrapper.rs` | harness 现成的 `process_environment_variable("SAMPLE_HARNESS_WRAPPER_THREADS")` | 不点名 | 点名 |
| `scripts/experiments/wrap_helper.py` | 自写 `read_env("SAMPLE_WRAP_THREADS")`，函数体里 `os.environ[name]` | 不点名 | 点名 |
| `scripts/experiments/alias_env.py` | `ENV = os.environ` 后 `ENV["SAMPLE_ALIAS_THREADS"]` | 不点名 | 点名 |
| `scripts/experiments/copied_env.py` | `settings = os.environ.copy()` 后 `settings.get("SAMPLE_COPIED_THREADS", …)` | 不点名 | 点名 |
| `crates/demo/src/use_alias.rs` | `use std::env::var as read_variable;` 后 `read_variable("…THREADS")` | 不点名 | 点名 |
| `crates/demo/src/vars_iter.rs` | `std::env::vars().find(\|(key, _)\| key == "…THREADS")` | 不点名 | 点名 |
| `scripts/hosts/or_default.sh` | `[[ -n "${SAMPLE_OR_FALLBACK_JOBS:-}" ]] \|\| SAMPLE_OR_FALLBACK_JOBS=8` | 不点名 | 点名 |
| `scripts/hosts/and_default.sh` | `test -z "$X" && X=8` | 不点名 | 点名 |
| `crates/demo/src/lib.rs` | 文件头 `#[cfg(test)]` 接 `mod tests;`，后面 `std::env::var("…THREADS")` | 不点名 | 不点名 |
| `scripts/hosts/printenv_read.sh` | `$(printenv SAMPLE_PRINTENV_WORKERS)` | 不点名 | 不点名 |

历史：实现员给 harness 新装置加并行，照仓里现成的写法（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/crates/singlefs-harness/tests/random_histories.rs:1959`：「fn count_from_environment(name: &str, default: u64)」）写 `count_from_environment("SINGLEFS_NEW_DEVICE_WORKERS", 8)`，capped.sh 的清单里没有它。`capped.sh 31` 到不了这个装置，它没设时取 8；heavy-thread-defaults 判绿。b3 那一版判红（引号里的名字一律算读）。shell 一侧 `|| X=8` 是「没设就给默认」的常见写法，零轮把 `||` 之后的赋值认成局部变量，读环境的那一处跟着不算。

真树上的对照（量过）：b6 判法比 `CODE_TOOLING_BREAK=thread-default-quoted-name-any-use` 少认 14 处（全是变量名清单与比较，没有真读），比 `thread-default-shell-inline-assignment-unseen` 少认 1 处（`scripts/gate/merge-and-verify.sh` 的 THREADS，是该少认的）。所以今天真树上没有漏，漏的是以后照现成写法写的新装置。

四句：分辨臂——分：零轮形态中，零轮之前不中（九格）；看得到——看得到：名字就在调用处的引号里，判法只看同一条语句里有没有 os.environ 一类的字；分句——跑前条款「打中也打中零轮之前」不成立（零轮之前不中），按「站不住」那一句；`#[cfg(test)] mod tests;` 与 printenv 两格两版都中，另立账；改法——零轮改法（引号名字限读环境写法、`;&&||` 后赋值算局部）正是这九格的病根；我提的 F3（推的）：引号名字回到一律算读、只把「同一条语句是比较或清单」的排掉；`||`、`&&` 之后的赋值只在右边不读同名变量且同一份文件没有 `${X:-}` / `$X` 读它时才算局部；`#[cfg(test)]` 后接以 `;` 收尾的 `mod x;` 不跳过。

专问（真树里别的读法）：仓里已有的读法是 harness 的 `process_environment_variable` / `number_from_the_variable_value_panicking_when_unreadable`（调用处多半传常量，常量那一路照认）与 `random_histories.rs` 的 `count_from_environment` 一类按名字包一层的函数；配置文件读线程数格的文件头自己写了判不了；argparse 的 `default=os.environ.get(...)` 同一条语句里有 environ，照认（没造出漏）。

## V6　叫停轮登记行：⑥ 与门禁 abandoned-rounds 格

被判的字面：判法免出处的那一刀按日期切（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_abandoned_rounds.py:55`：「SOURCE_REQUIRED_FROM = "2026-10-12"」），文件头说切的是登记日（同文件第 18 行：「出处这条规矩立起来之前登记的，没有出处可补」）；登记表自己的表头说第二列是腿发出那天（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/research/prompts/abandoned-rounds.tsv:2`：「日期（腿发出那天，YYYY-MM-DD）」）；规则只写格式（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/three-way-inference.md@多轮：一次打穿不算数，三轮里多数打穿才算`：「日期写 YYYY-MM-DD，同一轮只登记一行」），没写哪一天、也没写门禁对旧行免出处。

历史：某轮 2026-10-09 发腿，2026-10-13 用户叫停。主 agent 照表头把日期写成 2026-10-09，出处没写（或写了一个指不准的行号）。门禁格传 `legacy_sourceless_ok`、日期早于 2026-10-12，不看出处，判绿；⑥ 不传，不收，这一轮照旧算开着，写回员、实现员要改它快照里的文件就被拒。样本 `gate-samples/doc-process-records/abandoned-legacy-dispatch-date/`。判法函数两种读法（`out/v6-lib-b6.txt` 原样）：

```
old-topic-r1	不收：第 7 行没有出处：要写 records/<文件>.md 第 N 行（research/、.claude/ 下的文件、这一轮自己的材料、仓外路径都不认）
rc=1
old-topic-r1	收
rc=0
```

派发闸在同一棵样本树上（`out/v6-guard-b6.txt`，b3 同样拒）：

```
✗ 派 implementation-writer 要改的文件落在还没判完的三方轮 old-topic-r1 的开工快照里：
```

四句：分辨臂——分：「免出处按登记日」的读法与「表头写发腿日」不能同时成立，改表头或改判法的切法都不中（推的）；看得到——判法看得到第二列，看不到那是哪一天的日期（两种情形在登记行上逐字相同，判别子观测不到），要改的是判据而不是判法；分句——「一处收一处拒」：门禁格收、⑥ 拒；结局 fail-closed（⑥ 照挡，主 agent 补出处就收），不违反用户定案；改法——零轮改法（W06 免旧行出处）正是这一格的来处。

专问（10-12 之前免出处的行，一收一拒还剩哪些形状）：只剩日期这一刀——同一行在门禁格（`legacy_sourceless_ok=True`）与 ⑥（不传）上，形状与同一轮重复两样同判，出处一样只差「日期早于 10-12」：① 真旧行（今天表里第 7–10 行四行）门禁绿、⑥ 不收，这四轮在冻结副本里都没有 `-snapshot` 目录（逐个 `[ -d research/prompts/<轮>-snapshot ]` 查过），不算开着，今天不挡人；② 新登记而日期写发腿日早于 10-12 的（上面的历史）；③ 日期格式对而值不像日子的（`0000-00-00` 过形状、早于切点，门禁免出处）。⑥ 只判开着的轮（`only_names`）、门禁判全表，同一轮的判定一样。轮名不以 `-r<N>` 收尾的：⑥ 认作开着的轮、门禁格认不出它发过腿（登记了就红），表头写轮名「以 -r<N> 结尾」，按表头不可达，不算。

## V7　private-values：整份读与路径比对

被判的字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/doc-text.sh:51`：「路径里含私有值的那一段（文件名或目录名）打成」〔文件名含私有值〕；比对是整条路径（同文件第 471 行：「name_keys = sorted({key for key, word in targets if word in path})」），打码是逐段（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/doc-text.sh:440`：「return "/".join(FILE_NAME_MASK if any(word in part for _key, word in targets) else part for part in path.split("/"))」）。

私有值本身带 `/`（主机上的目录）时，命中的那一段跨了几段目录，没有哪一段单独含它，逐段打码一段都不打，红行把整条路径原样打出。真配置里有多少这种值（量过，只打个数，`tools/pv_path_probe.py`，原样 `out/v7-path-probe.txt`）：

```
取到的词 8 个（含本机主机名）；带 / 的 6 个
树里 14352 个文件（除 multi-host.env），相对路径命中的 0 个；按键名：{}；命中跨了 / 而逐段打码打不到的 0 个
```

历史：执行员把第二台上一个产物目录拉回本机，`ssh 第二台 tar czf - <第二台仓路径>/research/results/e999 | tar xzf - -C research/results/from-peer`，tar 去掉开头的 `/`，本地路径成了 `research/results/from-peer/<第二台仓路径去掉开头 />/research/results/e999/…`，这条路径含 `/<第二台仓路径>`。门禁这一格判红，红行打出整条路径。执行员照共用约束「贴原样输出」把红行贴进报告——值进了仓（共用约束里换值那一条要它先认出那是私有值，而这一格自己写着「值不打印」）。样本（假值，拷自冻结副本门禁样本的假配置）`gate-samples/doc-text/pv-tar-mirror-red/`，判法函数（`tools/run_pv_cell.py` 抽格里的 python 原样跑）原样输出 `out/v7-b6.txt`：

```
  ✗ 这次改动里 1 个文件的名字含本机私有值（路径里含值的那一段打成〔文件名含私有值〕，值不打印）：
      research/results/from-peer/srv/sample-private/peer-tree/research/results/e999.out  键名：PEER_REPOSITORY_DIRECTORY（值不打印）
```

（`/srv/sample-private/peer-tree` 就是样本配置里 PEER_REPOSITORY_DIRECTORY 的假值。）b3 不比路径：`out/v7-b3.txt` 退 0、判绿。

四句：分辨臂——分：零轮之前不打（不比路径），零轮形态打；按「整条路径命中就把整条路径打成〔文件名含私有值〕」或「把命中的那一段子串换掉」都不中（推的，F7）；看得到——看得到，词与路径都在手里；分句——「照规则做、结果错」：违反「私有信息不进仓」那一条用户定案；改法——零轮改法本身是来处。

专问（整份读的内存与时长，量过，假配置 5 个词）：200 MiB 的短行日志加 300 MiB 的单行文件一起判（`tools/v7_big.sh`），最后一次复跑输出「wall_seconds_rounded_to_one=2 maxrss_kB=636640」（挂钟与常驻每次不同，不进 `out/` 比；先前两次 2.72 秒 / 635900 kB、取整 1 秒 / 636420 kB）。单行文件整行读进内存再解码，常驻随最长一行涨：到 1 GiB 上限的单行约 2 GB（推的，按 300 MiB 单行量出的比例外推；内容不是纯 ASCII 时 Python 字符串变宽，更大，没量）。时长不成问题。前 8 KiB 里有 NUL 就整份跳过，文件头已写明，不另算。

## V5　重型测试闸：仓副本里的自证与 env 换目录

被判的字面：共用约束（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:81`）：「改过的脚本的自证不在副本里起」，同一条「换了目录的，都按换到的目录判」；钩子比的只是被起的那一份（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_heavy_tests.py:1421`：「main_file = os.path.realpath(os.path.join(SELFTEST_REGISTRY_ROOT, key))」），拒绝说明把出路写成主 agent（同文件第 1429 行：「由主 agent 在集成副本里跑」）。

喂法：`tools/v5_feed.sh <被判仓> <工作根底下的草稿目录>` 在草稿目录造三份「仓副本」（只拷 `.claude/hooks`）：wc-lib（`lib_heavy_tests.py` 末尾加一段起 checker 档测试的代码，入口 `heavy-test-guard.sh` 不动）、wc-words（只改没登记的 `lib_shell_words.py`）、wc-body（改登记了的 `write-guard.sh` 本身），把 Bash 工具的 hook JSON 喂给被判仓 `heavy-test-guard.sh` 的入口。原样输出 `out/v5-copy-b6.txt`：

```
S1-lib-imported-by-unchanged-guard	rc=0	
S1b-same-as-implementation-writer	rc=0	
S2-changed-lib-run-directly	rc=2	✗ 提交时才跑的检查被拒：<WORK>/v5-b6/wc-lib/.claude/hooks/lib_heavy_tests.py --selftest（解到的不是登记表登记的那一份 <WORK>/b6/repo/.claude/hooks/lib_heavy_tests.py：别处路径尾部相同的脚本、路径写在变量里或当前目录认不出的不按登记表放行；跑本仓那一份，写它在本仓的字面路径；在工作根底下的仓副本里跑它的自证，要主仓登记成 light、副本登记表里这一行与主仓逐字相同、副本里那份脚本与主仓那一份 sha256 相同，这一份不放的原因：副本里的脚本 <WORK>/v5-b6/wc-lib/.claude/hooks/lib_heavy_tests.py 与主仓那一份 <WORK>/b6/repo/.claude/hooks/lib_heavy_tests.py 内容不同（sha256 2969a6277d70 对 8b2d2045d690）：闸不读副本里的脚本正文，改过的脚本的自证不放，把脚本改动与要核的命令写进交回报告，由主 agent 在集成副本里跑）（提交时才跑的检查）：tooling-writer 不跑「提交时才跑的检查」
S4-unregistered-lib-imported	rc=0	
S5-changed-body-subagent	rc=2	✗ 提交时才跑的检查被拒：<WORK>/v5-b6/wc-body/.claude/hooks/write-guard.sh --selftest（解到的不是登记表登记的那一份 <WORK>/b6/repo/.claude/hooks/write-guard.sh：别处路径尾部相同的脚本、路径写在变量里或当前目录认不出的不按登记表放行；跑本仓那一份，写它在本仓的字面路径；在工作根底下的仓副本里跑它的自证，要主仓登记成 light、副本登记表里这一行与主仓逐字相同、副本里那份脚本与主仓那一份 sha256 相同，这一份不放的原因：副本里的脚本 <WORK>/v5-b6/wc-body/.claude/hooks/write-guard.sh 与主仓那一份 <WORK>/b6/repo/.claude/hooks/write-guard.sh 内容不同（sha256 3ecf5b6dd33b 对 874afb35fd12）：闸不读副本里的脚本正文，改过的脚本的自证不放，把脚本改动与要核的命令写进交回报告，由主 agent 在集成副本里跑）（提交时才跑的检查）：tooling-writer 不跑「提交时才跑的检查」
S6-changed-body-main-agent	rc=2	→ 照抄（提交流程里跑；用户要求时把 commit 换成 user-request，提交之外要跑先弹窗问用户）：
S7-changed-lib-run-directly-main-agent	rc=2	→ 照抄（提交流程里跑；用户要求时把 commit 换成 user-request，提交之外要跑先弹窗问用户）：
```

`out/v5-copy-b3.txt` 七格全是 rc=0。

### V5-甲　改了 import 的库，入口脚本不动，副本自证照放（量过）

历史：tooling-writer 在仓副本里改 `lib_heavy_tests.py`（或 `lib_shell_words.py`），照共用约束「副本里那份脚本与主工作区那一份 sha256 相同」理解成「我没改 heavy-test-guard.sh」，在副本里起 `bash <副本>/.claude/hooks/heavy-test-guard.sh --selftest`。闸放行（S1、S1b、S4 rc=0），自证 import 的是副本里改过的库——改过的代码在副本里跑了，跟用户定的「仓副本自证收回放宽」（H4 乙）要挡的是同一件事：闸不读正文，改过的代码里有什么它不知道。

四句：分辨臂——不分：零轮前后都放（零轮改法在这一格上不起作用）；看得到——看得到：副本里同目录的库就在磁盘上，比一下 sha256 就知道；分句——「定义说拒（改过的脚本的自证不在副本里起）而钩子不拒」，「脚本」包不包括它 import 的库，定义没写到；改法——零轮改法碰不到这一格，交出去时写「对 S1、S4 不起作用」；F4（推的）。

### V5-乙　照拒绝说明交给主 agent，主 agent 也被拒（量过）

历史：tooling-writer 被拒（S5），照拒绝说明把命令写进报告「要主 agent 跑的命令」一节。主 agent 照「由主 agent 在集成副本里跑」在集成副本里起同一条命令（不带前缀，这是轻型自证，不是提交）：闸拒（S6、S7 rc=2），出路是「提交流程里跑；用户要求时把 commit 换成 user-request，提交之外要跑先弹窗问用户」。b3 主 agent 这两格放行。

四句：分辨臂——分：零轮之前不中；看得到——看得到；分句——「一处收一处拒」：拒绝说明（与共用约束第 81 行）把活交给主 agent，同一个闸不让主 agent 平常跑，只能借提交前缀或弹窗——一条轻型自证被升成了提交时才跑的检查；改法——F5（推的）。

### V5-丙　env 前缀的两种写法（量过）

钩子只认 env 的 `-C` / `--chdir` 单独成词（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_shell_words.py:477`：「if argument in ("-C", "--chdir") and position + 1 < len(words):」，`-C` 贴目录写也认）。原样输出 `out/v5-chdir-b6.txt`（implementation-writer，当前目录 `crates/singlefs-harness`）：

```
C1-env-C	rc=2	✗ 重型测试被拒：cargo test 跑到 singlefs-checker-tier 包的测试（checker 档，默认只在提交时跑；按包）（checker 档）：implementation-writer 不跑「checker 档」
C2-env-iC-combined	rc=0	
C6-env-C-var	rc=0	
C8-env-S-split	rc=0	
```

C2：`env -iC ../singlefs-checker-tier cargo test`，`-i` 与 `-C` 合写，放行；共用约束第 81 行说 `env -C` 换的目录照判。C8：`env -S 'cargo test -p singlefs-checker-tier'`，`-S`（把一串按空白切开当命令）后面整串当一个词，闸认不出 cargo，放行——不靠换目录，直接点名 checker 档包。两格 b3 同样放（`out/v5-chdir-b3.txt`）。C8 不是零轮形态的格，打中两版，另立一笔账。

四句：分辨臂——C2 不分（零轮只加了单独成词的 `-C`）；看得到——看得到，命令原文就在；分句——C2「定义说拒而钩子不拒」，C8 落在「越出重型测试那一条的命令」的拒绝范围；改法——F6（推的）。

专问（「主仓那一份」在合入前后各指哪一份）：指钩子所在仓，也就是挂钩子的主工作区，比的是工作区里此刻那一份（含别的会话没提交的改动），不是提交。合入之前：副本里改过的脚本与主工作区不同，拒；别的会话正在主工作区里改同一份脚本时，所有副本（哪怕没改）都不同，一起拒（fail-closed，推的）。合入之后：副本里那一份与主工作区相同，放——这时跑的就是合入的那一版。副本里脚本 import 的别的文件改了：照放（V5-甲，量过）。

## V8　长活文件头的登记值读法与 ⑭ 的词表

### V8-甲　E171 修订 12：改头不改身三道全过，老实拷的被拒（量过）

被判的字面：豁免表只放修订 6–11 的六份入口，并写「固定每台 31 线程归 E171 下一次修订」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/long-run-live-hosts-exempt:7`），「新脚本不往这里加」（同文件第 5 行）；现算发生在编排里，编排那几行在 heavy-thread-defaults 的排除表里（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/heavy-thread-exempt:58`：「min(found.available_threads, HOST_THREADS_CEILING)  # 按 sha256 钉住的冻结层」）。

历史：E171 下一次修订，实验员照修订 11 的做法拷一份入口 `run_segment_one_revision_12_multi_host.sh`，正文照旧起修订 11 的单元编排（编排按 available_threads 现算每台槽数与线程）。① 原样拷（文件头老实写「host-resources.py --json 现查现算」）：不在豁免表里，按登记值读，`detached-run.sh long-run-check` 退 5；要放进豁免表就是表涨了，门禁格红、要问用户。② 只把 `# long-run-hosts:` 一行改成「台数从 --emit-hosts 读，每台线程取登记值 .claude/heavy-thread-default（经 capped.sh default）」（`feeds/v8/run_segment_one_revision_12_multi_host.sh`，与修订 11 只差这一行与标题里的修订号）：三道全过，跑起来照旧按忙闲现算。原样输出 `out/v8-rev12.txt`：

```
b6 lib_long_run script run_segment_one_revision_12_multi_host.sh rc=0
b6 lib_long_run script run_segment_one_revision_12_honest_multi_host.sh rc=1
b3 lib_long_run script run_segment_one_revision_12_multi_host.sh rc=1
b3 lib_long_run script run_segment_one_revision_12_honest_multi_host.sh rc=0
  ✓ .claude/long-run-live-hosts-exempt 6 行：每行的脚本都在，文件头都得靠现算读法才齐（按登记值读不齐）；与基准那一版比没涨
exempt-check rc=0
  ✓ 每台默认重活线程数只经登记位取：登记值 31；扫了 834 份（scripts 457、.claude/gate.d 24、.claude/hooks 17、crates 335、.claude/cargo-command-prefix 1），可疑写法 94 处全在排除表里（表 66 行，每行都命中）；装置自己读的线程变量 58 个，其中 capped.sh 不设的 28 个都回退到上限或登记值、或被判行在排除表里（capped.sh 清单 9 个变量）；基准里还没有这张表（或不在 git 仓里），只缩不涨这一条没比
heavy-thread-defaults rc=0
b6 detached-run long-run-check run_segment_one_revision_12_multi_host.sh rc=0
b6 detached-run long-run-check run_segment_one_revision_12_honest_multi_host.sh rc=5
```

最后两行是 `detached-run.sh long-run-check`（主 agent 起长活前、子 agent 交回前跑的那一道）：② 退 0，① 退 5。

四句：分辨臂——分：b3（文件头默认按现算读）上 ① 放、② 拒，b6 正相反；看得到——判别子观测不到：文件头判法只看注释，跑起来按什么取线程在正文与编排里，两份入口的文件头之外逐字相同；分句——「照规则做、结果错」：照「文件头按登记值写、豁免表不涨、冻结层一个字节不改」三条做，结果违反用户定案「固定 31、不看忙闲」；改法——零轮改法（Q1 文件头改登记值读法）是让 ② 能过、① 被拒的那一步；F9（推的）。

专问（豁免表与 E171 下一次修订的衔接）：三条规矩合起来，下一次修订只有三条路：改冻结编排（破 sha256 钉）、豁免表加一行（门禁红、交用户）、改头不改身（全绿，结果错）。最省事的那条正是错的。

### V8-乙　⑭ 现算词表：定义原话被拒，「一半」放行（量过）

被判的字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_long_run.py:78`（见 V1-乙），否定句里写也拒。喂法同 V1，基础提示 `feeds/v1/base-over2h.txt`（预计挂钟超过 2 小时、分片续跑齐），只换多机一行；原样输出 `out/v8-hosts-b6.txt`：

```
M0	rc=0	  ✓ 派发闸照这份提示放行（investigator，查了 11 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）
M1	rc=2	✗ 派 investigator 的提示写了预计挂钟超过 2 小时，长活声明不齐：
M2	rc=2	✗ 派 investigator 的提示写了预计挂钟超过 2 小时，长活声明不齐：
M3	rc=2	✗ 派 investigator 的提示写了预计挂钟超过 2 小时，长活声明不齐：
M4	rc=0	  ✓ 派发闸照这份提示放行（investigator，查了 11 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）
M5	rc=0	  ✓ 派发闸照这份提示放行（investigator，查了 11 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）
M6	rc=0	  ✓ 派发闸照这份提示放行（investigator，查了 11 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）
M7	rc=0	  ✓ 派发闸照这份提示放行（investigator，查了 11 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）
```

拒的那几格的原因行与 M6（`out/v8-hosts-reasons-b6.txt` 原样）：

```
M1	  多机： 一行写了按核数、忙闲、负载现算的说法（忙闲）：每台线程取登记值 .claude/heavy-thread-default，不除以此刻在跑的重活数、不按 nproc 与核数取、不看忙闲与负载；这一行里不要出现这些词（否定句也不行）
M2	  多机： 一行写了按核数、忙闲、负载现算的说法（在跑）：每台线程取登记值 .claude/heavy-thread-default，不除以此刻在跑的重活数、不按 nproc 与核数取、不看忙闲与负载；这一行里不要出现这些词（否定句也不行）
M3	  多机： 一行写了按核数、忙闲、负载现算的说法（核数）：每台线程取登记值 .claude/heavy-thread-default，不除以此刻在跑的重活数、不按 nproc 与核数取、不看忙闲与负载；这一行里不要出现这些词（否定句也不行）
M6	  ✓ 派发闸照这份提示放行（investigator，查了 11 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）
```

M1 的「不看忙闲」是 main-agent.md 第 53 行的原话（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`：「机器忙时也固定这个数，不看忙闲」）；M3 正是专问里说的「核数」落在台数说明里。M6「每台线程取登记值 .claude/heavy-thread-default 的一半」放行，而「一半」是 ⑮ 自己拒的比例词。`out/v8-hosts-b3.txt` 八格全 rc=0。

四句：分辨臂——M1–M3 分（b3 放、b6 拒），M6、M7 不分（两版都放）；看得到——看得到；分句——M1–M3「钩子拒而定义没写」（定义只说「不现算」，没说这几个词连否定句也不许出现，main-agent.md 与共用约束的 ⑭ 条都没写词表）；M6「定义说拒（每台线程取登记的每台默认重活线程数）而钩子不拒」；改法——F2（推的）。

## 我提的改法（只在我的模型上量过、被攻过零轮）

每一格都是「推的」：按代码推，没实现、没跑。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| F1 ⑮ 括注改白名单（只许「本机」「登记值」「默认」「不起重活」这类词，别的一律拒），或括注整个去掉 | V1-甲（不再按字误伤）、V1-乙（「按负载降」「对折」「双倍」「第２台１６」都不在白名单里） | 推的 |
| F2 ⑭ 的多机一行与 ⑮ 的括注、放行行理由共用一张现算词表，词表加「一半」「÷」「/」「余量」「此刻」，去掉否定句也拒那一条（或把定义原话「不看忙闲」「不现算」列成准写的固定句） | V8-乙（M1–M3 不再误伤，M6、M7 拒）、V1-乙 | 推的 |
| F3 heavy-thread-defaults：引号里的线程变量名回到一律算读，只排掉同一条语句是 `in` 比较或纯清单的；`\|\|`、`&&` 之后的赋值，同一份文件里另有 `$X` / `${X:-}` 读它的不算局部；`#[cfg(test)]` 后接以 `;` 收尾的 `mod x;` 不跳过 | V3 九格、V3-乙 lib.rs | 推的 |
| F4 仓副本自证另比「入口脚本加它同目录 import 的 lib_*.py」的 sha256（或比整个 `.claude/hooks/` 目录的清单） | V5-甲 S1、S4 | 推的 |
| F5 拒绝说明不再写「由主 agent 在集成副本里跑」，改写主 agent 的真出路（带 `SINGLEFS_HEAVY_TESTS=user-request` 先弹窗，或等合入主工作区后跑主仓那一份）；或钩子对主 agent 放行工作根底下副本里登记成 light 的单独 `--selftest` | V5-乙 | 推的 |
| F6 lib_shell_words 的 env 前缀认合写短选项（`-iC`）与 `-S` / `--split-string`（把串按 shell 切开当命令） | V5-丙 C2、C8 | 推的 |
| F7 private-values 的红行：路径里命中的子串整段换成〔文件名含私有值〕（按子串替换，不按 `/` 分段） | V7 | 推的 |
| F8 登记表第二列改名「登记日」并改表头；或判法改按一列单独的「登记日」切 | V6 | 推的 |
| F9 E171 修订 12 之前，长活文件头判法同时看被起脚本调到的编排是否在 heavy-thread-exempt 的冻结层行里（命中就按现算读并要求进豁免表、交用户） | V8-甲 | 推的 |

## 没打中的形状

- ⑮：`1. 线程上限：16`、表格 `| 线程上限 | 16 |`（缺行拒，rc=2）；`线程上限：31。`、`31 线程`（拒）；`031`（放，等于 31，不算错）。上标、全角主数上一题已试过。
- ⑮ 九种类型（量过）：`grep -c 'cargo\|capped\|编译\|原型\|run-with-memory-cap'` 在 clerk.md、three-way-defense.md 各 1，three-way-forward.md 0，sweep、kb-scribe、prior-art、three-way-materials、experiment-designer 也是 0，没漏类型。钩子注释「后三种：定义许编译、跑编译出来的代码（clerk.md、three-way-forward.md、three-way-defense.md）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:994`）对前向腿那一份对不上：它被逼写线程上限，定义里没有起重活的步骤（文字没写到，代价一行字，不算打中）。
- V3：python `parser.add_argument(..., default=os.environ.get("X_THREADS"))` 同一条语句里有 environ，照认；`if …; then X=8; fi` 不算局部（`then` 挡住），照认。
- V5 换目录：`pushd`、`( cd …; … )`、`env -u RUSTFLAGS -C 目录`、`env -C 目录 sh -c '…'`、`--chdir=目录`、`cargo -Zunstable-options -C 目录 test` 都拒（`out/v5-chdir-b6.txt`）；目录写在变量里的（C6）钩子文件头已写罩不住，不另算。
- V5 自证：副本里改了登记了的入口本身（S2、S5）拒，零轮改法在这两格上有效。
- V6：出处那一行里出现别的轮名、轮名只出现在路径里（`…/m3-x-r1-opus-output.md`）、`_` 前缀的材料名——判法前后边界都对，没造出误收。同一句用户原话叫停两轮时两轮都不收，主 agent 分两行写记录即可，不算打中。
- V7：路径误报——真配置的 8 个词在冻结副本拷贝 14352 个文件的相对路径上命中 0 个（`out/v7-path-probe.txt`）；内容一侧逐行读，500 MiB 一到三秒，没打中。
- V8：`多机：…（31）` 放行，算写死常量但数等于登记值，只记一笔（M4）；M0（main-agent 第 55 行原话）放行。

## 这条腿自己的限度

- 门禁格（heavy-thread-defaults、long-run-header-exempt、private-values、abandoned-rounds）我只抽格里的 python 在小树上跑，没起 `--check`；四条交主 agent（`gate-samples/要主agent跑.txt`）。抽出来跑与 `--check` 的差别：改动范围用 git 的未跟踪加已改动代替 `changed-paths.sh`，基准状态取 none（只缩不涨那一条没比）。
- 只在副本上量：钩子与判法都在 `~/.local/share/singlefs/work/m3-defs-b6-r1-opus/` 下的拷贝里跑，数不进 kb。
- V1-乙、V8-乙的「子 agent 会照括注降线程」是推的：没有派真 agent。
- V7 的 1 GiB 单行内存是按 300 MiB 外推的；非 ASCII 内容没量。
- V6 门禁一侧（绿）是读代码推的，没跑格。
- 没读主工作区里被判的文件与禁读清单里的任何一份；读了上一题三轮的腿报告（只为去重）。
- 我造的仓副本（`b6/`、`b3/`、`rr2/`、`rr3/`、`b6-rev12/`、`v5-*`、`pv-*`）交回前删掉，见交回说明。
