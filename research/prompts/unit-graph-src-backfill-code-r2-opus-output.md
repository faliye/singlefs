# unit-graph-src-backfill-code-r2：攻方腿报告（2026-11-19）

正文 `research/prompts/_unit-graph-src-backfill-code-r2-body.md`；第一轮判决 `research/prompts/unit-graph-src-backfill-code-r1-main-verification.md`。开工时 `sha256sum -c` 开工快照的 `crates-sha256.txt` 与 `governance-sha256.txt` 十七份全 OK，之后在草稿目录建的冻结副本里同一张表也是十七份全 OK（`rerun.sh` 开头再核一次）。没读禁读清单里的两处。

所有「打中」都是读码推出、写成夹具与探针的，没跑：子 agent 不交任务，命令在「要主 agent 跑的命令」一节，结局出来之前一律按「推的」算。

## 复跑

模型目录 `research/prompts/unit-graph-src-backfill-code-r2-opus-model/`：`fixtures/` 是新写的十二棵夹具加第一轮那八棵（`fixtures/round-one/`，与第一轮草稿目录里的逐字节相同，`sha256sum -c` 0 处不符），`tools/rerun.sh`（照 `scripts/agents/three-way-rerun-template.sh` 拷、只换了 `rerun_steps`），`tools/rerun-from-a-clean-work-directory.sh`（照范例），`tools/probes/` 是运行期探针源码、`libfakesync` 的 C 源码与增量探针的小包，`tools/frozen-sha256.txt` 是开工快照那十七份。

冻结副本：`/home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-attack/frozen`（`scripts/hosts/minimal-copy.py` 建，`--with scripts/gate/lib_function_units.py`，109M，在认领目录的 `.deliverables` 写成「交接」）。

复跑命令（一条，仓根起）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/unit-graph-src-backfill-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-attack/frozen /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-attack-run /home/fy5090/code/singlefs/research/prompts/unit-graph-src-backfill-code-r2-opus-model
```

它做的事（读数各打一行 `name=…`，原样输出在 `<运行目录>/out/`）：核冻结副本十七份；在运行目录编 `unit-graph`（`cargo build -p singlefs-unit-graph --bin unit-graph`）；把每棵夹具分别喂 Rust 与 python 两份（`unit-graph closure`、`lib_function_units.py closure`，工作区那几棵另各 `dump` 一次逐行比）；两个运行期探针（`rustc` 把夹具原样编成程序跑，看运行时真调到哪一份）；链接探针（`libfakesync.so` 导出 `fdatasync`、交 4242，夹具各包 `cargo test --offline`，打 `fdatasync-result`）；增量探针（同一个小包五种设置下 `cargo build -v --offline`，读工作区包与依赖包 `cfg_if` 的 rustc 参数，再在同一环境下算 `unit-graph key --toolchain-of-the-build dev`）；观察者那一组的成员 A/B（冻结树拷两份、一份根清单删掉 `gpu-verification-server` 那一行，同四个入口各收一次闭包、逐行比，再 `why` 两次）。

估时（推的，没量）：编 `unit-graph` 与它的依赖几分钟；链接与增量探针十次小编译；A/B 两次整仓闭包加两次 `why`。合计十几分钟。内存：整仓闭包两次是大头，按 `unit-graph` 平常喂整仓的量；没量过。这一趟不走崩溃枚举，第 3b 步的状态数与内存稀疏盘不适用。

多机：链接与增量探针用 `--offline`，要 `cfg-if 1.0.4`、`cc 1.5.1` 与它的依赖在那台的 `CARGO_HOME` 缓存里（本机缓存里有，核过 `~/.cargo/registry/cache`）；第二台没有就那几行 `exit` 非 0，换本机重交。

各文件 sha256 在下一节。

## 模型目录各文件的 sha256

```
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/extern-link-from-another-target-of-the-final-binary/Cargo.lock
a3b854357a93c55acb115b06d5e5986a5c3e280afa7ddf1c6277bcb2a6d2f8bd  fixtures/extern-link-from-another-target-of-the-final-binary/crates/fexternclean/Cargo.toml
b81ce395b2d6f4067103882396bf3fc8974ff1075cdd42fe40522df47a0fcae0  fixtures/extern-link-from-another-target-of-the-final-binary/crates/fexternclean/src/lib.rs
8be4b8ccfc5b9b48e5521e647e555705d09a9e32848bae76f4f81556d71e4cc5  fixtures/extern-link-from-another-target-of-the-final-binary/crates/fexternclean/tests/clean.rs
a5c67c126ce0eeb4e0c68059fe62b9f4e7d66e69ef4636a5e3b20b088f35df26  fixtures/extern-link-from-another-target-of-the-final-binary/crates/fexterndep/Cargo.toml
f0ceddc212a155890624a4275952bdde94fa3415def8d63b71292bad570cfa4b  fixtures/extern-link-from-another-target-of-the-final-binary/crates/fexterndep/src/lib.rs
970fd2aa0213767f17285b047219c720952bac44160277a96b4f422bfe3cf7b9  fixtures/extern-link-from-another-target-of-the-final-binary/crates/fexterndep/tests/through_dependency.rs
e7ae71d850fc90f614e765d45ce76985708d7046d594e93094243a7432e8695d  fixtures/extern-link-from-another-target-of-the-final-binary/crates/fexternfinal/Cargo.toml
b81ce395b2d6f4067103882396bf3fc8974ff1075cdd42fe40522df47a0fcae0  fixtures/extern-link-from-another-target-of-the-final-binary/crates/fexternfinal/src/lib.rs
51c29991736be7e61f88de433d611fc810f31074ba63867d804368d8a7a4399b  fixtures/extern-link-from-another-target-of-the-final-binary/crates/fexternfinal/tests/linked.rs
254b3bc7d9b20abb59c0fa5914969e2a5368fe82c31f458409087d8a80ba1cd8  fixtures/extern-link-from-another-target-of-the-final-binary/crates/flinkdep/Cargo.toml
b75c9bcd50350fca9ee6c31f652855281fb4e2e557d26d7a44a37a17ff46013e  fixtures/extern-link-from-another-target-of-the-final-binary/crates/flinkdep/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/extern-link-through-a-build-dependency/Cargo.lock
fb79b20b242d49367f4b5538952893c0526c0715305eddd441a3993919276880  fixtures/extern-link-through-a-build-dependency/crates/fbuildhelper/Cargo.toml
502fb0546099175421f8e71d507d92b036cb363149d99c2be1651463b404d850  fixtures/extern-link-through-a-build-dependency/crates/fbuildhelper/src/lib.rs
64ee58fa4a822bdd209f3f55e5cd14fbe907f52fa1960987b861909e3722c2fd  fixtures/extern-link-through-a-build-dependency/crates/fbuildlink/build.rs
ead74f5b5cd8bd7d7c367b1222bbd69fd75f7f3342949e78a5f258cb29b910fa  fixtures/extern-link-through-a-build-dependency/crates/fbuildlink/Cargo.toml
b81ce395b2d6f4067103882396bf3fc8974ff1075cdd42fe40522df47a0fcae0  fixtures/extern-link-through-a-build-dependency/crates/fbuildlink/src/lib.rs
2b1c4c9d747b721d4113c47a809cd60c009023101ad91995c9741b7e2829a6ec  fixtures/extern-link-through-a-build-dependency/crates/fbuildlink/tests/through_build_script.rs
4d3361b56e550ef808287003c8657d6d9d6356e1438a08b512935541554e5111  fixtures/extern-link-through-a-build-dependency/crates/fcclink/build.rs
ed1f6f435a521c20830466443eace1d6b3bc47955f6e3b80600090d829def478  fixtures/extern-link-through-a-build-dependency/crates/fcclink/Cargo.toml
b81ce395b2d6f4067103882396bf3fc8974ff1075cdd42fe40522df47a0fcae0  fixtures/extern-link-through-a-build-dependency/crates/fcclink/src/lib.rs
3995590ec4d993dbf335583c4bb7dd786528fc08cb4dc6260e7899c83216403c  fixtures/extern-link-through-a-build-dependency/crates/fcclink/src/shim.c
33b7091ce29c363da4fd6e9e4f18da7b5c347b1d03ac12919026a691e1b45b13  fixtures/extern-link-through-a-build-dependency/crates/fcclink/tests/through_build_script.rs
735d463433cb42b53504a47d2726632ebbc8e3b8b79bca6a479c30d4f0932d21  fixtures/members-dot-unnamed-root/Cargo.toml
73c6865d7f68bf02b0aa7984dbce6fe11cc043ea20e34c5c32d1de739b066dd2  fixtures/members-dot-unnamed-root/src/lib.rs
ac480c8b1867002051596e749cd2798d03932da0fd7cf9adcf09db46c76d2a82  fixtures/members-dot-with-root-package/Cargo.toml
73c6865d7f68bf02b0aa7984dbce6fe11cc043ea20e34c5c32d1de739b066dd2  fixtures/members-dot-with-root-package/src/lib.rs
7d358b7db8ab4d10d9f5cdb88a07bf7e2aa5f1cf435a02fcb8345a84ef966403  fixtures/members-in-a-multi-line-string/Cargo.toml
13536643c491618b13eedcc9071822bfaf0f76ea9612a0668d861015f3900038  fixtures/members-in-a-multi-line-string/crates/alpha/Cargo.toml
a65fae83c2e096cb8e04b4a766ce9eeeff408f9535ada865556e20b329620adf  fixtures/members-in-a-multi-line-string/crates/alpha/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/members-sharing-a-directory-name/Cargo.lock
fbb750f36656fd506ceb07fdc6d9f77db1b9aebe82efaf3acb7313f3dce7241a  fixtures/members-sharing-a-directory-name/Cargo.toml
9f8ab20a633c2c68bc5fe5fdf3f1e49deccebd55f9358edcff9d5da151a9a0da  fixtures/members-sharing-a-directory-name/crates/common/Cargo.toml
49b02af435ab674d25a3b2ed8ad02ebe528e6b19af740cb9cc010b5ebc069e7b  fixtures/members-sharing-a-directory-name/crates/common/src/lib.rs
fff5ec893e72d4a5667ca01be40e0dd763956f76515833a110c408f86438702b  fixtures/members-sharing-a-directory-name/crates/common/tests/smoke.rs
b06db8d02ace18f3a6d23dc50a818fe6fc9eca207d35a00455d75464c548f76e  fixtures/members-sharing-a-directory-name/tools/common/Cargo.toml
4b5e613a00a204e03e5ecdbae75a50dc35f575f1520a68a6f939b33e1b97e1a0  fixtures/members-sharing-a-directory-name/tools/common/src/lib.rs
ab3adf8f6a52da812cf21ce506922b24fd3854a3cbbb2fa3d29266e538555269  fixtures/members-sharing-a-directory-name/tools/common/tests/smoke.rs
c63381e8eaa4d9394d149f1a99184fb0563eb940c2fcf6c8d715e18f9217c9ed  fixtures/members-under-a-root-with-glob-characters/Cargo.toml
13536643c491618b13eedcc9071822bfaf0f76ea9612a0668d861015f3900038  fixtures/members-under-a-root-with-glob-characters/crates/alpha/Cargo.toml
a65fae83c2e096cb8e04b4a766ce9eeeff408f9535ada865556e20b329620adf  fixtures/members-under-a-root-with-glob-characters/crates/alpha/src/lib.rs
6d0932b5295ee664669145298a53b3ab005fc0975913e1fbb322ee92a047d4e4  fixtures/nested-member-inside-another-member/Cargo.toml
f0c88b3bed62d6ac3dcd9958fa01e69694f70516ee9834999d9a9e8dff7eb21a  fixtures/nested-member-inside-another-member/crates/outer/Cargo.toml
678a2d2526d35fae3f07fcd2b92a87f3621eb07ba9c505d9b2c4bde671f978aa  fixtures/nested-member-inside-another-member/crates/outer/inner/Cargo.toml
31d307c4b0025eb56e3d1c2cb06fc8e2e02dc9e609e8cbc5f2788e8eaaf12690  fixtures/nested-member-inside-another-member/crates/outer/inner/src/lib.rs
114203ae1badb542f9507a99f93286d386c626e540ed8cf04692d922c7a93855  fixtures/nested-member-inside-another-member/crates/outer/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/round-one/extern-link-on-another-block/Cargo.lock
8af6b23d4ad13ad0a8e557b779671973d82e8aa702d6ae054708bf0fef72ecc8  fixtures/round-one/extern-link-on-another-block/crates/fexternother/Cargo.toml
f7d8e6aa212e681cef7881af59f81c644e692970637d5684a65eb1c7dfc5909e  fixtures/round-one/extern-link-on-another-block/crates/fexternother/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/round-one/extern-link-through-cfg-attr/Cargo.lock
dce13b795a234541bed0e9102b8f6f0b5be6953fc791b21ab03658cd812a1d53  fixtures/round-one/extern-link-through-cfg-attr/crates/fexternattr/Cargo.toml
28a6437102b678fed6eff155168b2007b4d0943b4e826a5a30c90152cec2ab3f  fixtures/round-one/extern-link-through-cfg-attr/crates/fexternattr/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/round-one/relaunch-shadowed-binding/Cargo.lock
0115c392d971acdfab7c062e45f5eae7930c5a8f64cc2f621d2ebc0c62202282  fixtures/round-one/relaunch-shadowed-binding/crates/frelaunch/Cargo.toml
f97bdabafc73f3e71c96ca8ea50c257372f515968a147f09ee4fe8e62b640b9a  fixtures/round-one/relaunch-shadowed-binding/crates/frelaunch/src/bin/worker.rs
3b51283e11b3c78a586dfdefb4571a610ae3466abf58e286ee999238c3eed6c6  fixtures/round-one/relaunch-shadowed-binding/crates/frelaunch/src/main.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/round-one/restart-with-argument-fallback/Cargo.lock
5cb79b1b6b3d47a0492dba37c5ac9e7fcb8ffe0806a20487ac1a1cd20d1a375c  fixtures/round-one/restart-with-argument-fallback/crates/ffallback/Cargo.toml
5a652833e8d694ed4f1d550ea34fd158205696dc8b5f912008dc01061e5c6fe2  fixtures/round-one/restart-with-argument-fallback/crates/ffallback/src/main.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/round-one/shadow-through-macro/Cargo.lock
d04c37265b594162b8ce2007a09abec1cf522b62cf3dc25da6822703537c8916  fixtures/round-one/shadow-through-macro/crates/fshadow/Cargo.toml
91cb5ce6e6d08e86b13decc57092c3d659e93c5bc53c98a48debc530150155ce  fixtures/round-one/shadow-through-macro/crates/fshadow/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/round-one/use-alias-leaf-divergence/Cargo.lock
f18055ce4a2adbea0de9495df0b00aa878bfed5094b26858535b1b300d65ad33  fixtures/round-one/use-alias-leaf-divergence/crates/fdiverge/Cargo.toml
463b04911b4d16c260a34f40ff5af0e281d600c65b2ed75ed4e5db2b285060d6  fixtures/round-one/use-alias-leaf-divergence/crates/fdiverge/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/round-one/use-cfg-swapped-re-export/Cargo.lock
daabef2f69bb6d63f0c16fad11210c756d6196796f7756f43ec0042d295f3333  fixtures/round-one/use-cfg-swapped-re-export/crates/fswap/Cargo.toml
d74c53053d59e04889c7e1cde6cd2dedc99e12f53ee16757f975ffdbcfdde0fb  fixtures/round-one/use-cfg-swapped-re-export/crates/fswap/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/round-one/use-relative-path-through-module-alias/Cargo.lock
e18fcf4a198612bdcc07ef86e3e2d29404320f182e669077584aeb9b0c8cf818  fixtures/round-one/use-relative-path-through-module-alias/crates/falias/Cargo.toml
69c943e3e0d7fcfb2125ff7688db00b44eb762510b6ed3a929100d17a97eaf8c  fixtures/round-one/use-relative-path-through-module-alias/crates/falias/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/shadow-through-a-macro-named-like-a-standard-one/Cargo.lock
46a5269512246f89ed54bd923ca6342c11a4ed1c289ba934016ff7305c981be5  fixtures/shadow-through-a-macro-named-like-a-standard-one/crates/fstdname/Cargo.toml
f41b65df14a41e3ef6a3898a79e9ed373ed9b510a41f2d2ff8b721b651b21632  fixtures/shadow-through-a-macro-named-like-a-standard-one/crates/fstdname/src/lib.rs
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixtures/use-relative-path-alias-in-the-same-use/Cargo.lock
89a8d9de0050c6d525dfd02bad75a572d7412e5ce64f353b06066b6238d97d96  fixtures/use-relative-path-alias-in-the-same-use/crates/fsameuse/Cargo.toml
6d3275bde83e4e70a616c2016962d6b12086936f40223acce75e9435125d13af  fixtures/use-relative-path-alias-in-the-same-use/crates/fsameuse/src/lib.rs
53d20622f40dc273b07166e710f6732d2a43d24cf23670ea160a9a250513735c  fixtures/workspace-inline-empty-table/Cargo.toml
13536643c491618b13eedcc9071822bfaf0f76ea9612a0668d861015f3900038  fixtures/workspace-inline-empty-table/crates/alpha/Cargo.toml
a65fae83c2e096cb8e04b4a766ce9eeeff408f9535ada865556e20b329620adf  fixtures/workspace-inline-empty-table/crates/alpha/src/lib.rs
7122ae73392771345479cb417ccf253cf7b98d9f2a04c1e5b0b995880957599e  tools/frozen-sha256.txt
3995590ec4d993dbf335583c4bb7dd786528fc08cb4dc6260e7899c83216403c  tools/probes/fakesync.c
e02a7b8b57a320689463a776e48d12c23702cf2e4753f0bf16b0754ed635b1dd  tools/probes/incremental-probe-crate/Cargo.toml.template
da5d39d4cb056419b09b144137623d907b29750cf7a819f323887a68223e162d  tools/probes/incremental-probe-crate/src/lib.rs
9ccc9ffb5d812954e9000d9b64fd8c813e7947df9682df68b2dc12c6073baadd  tools/probes/shadow-through-a-macro-named-like-a-standard-one.rs
bae096598d3e3fb168094978d07ab2c4ff9ff0e3d4010555c4197497017c2354  tools/probes/use-relative-path-alias-in-the-same-use.rs
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
c8944d1eb594cf5b764c721f9efa2cb543502cb721dcce424545181296556d5e  tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 第 1 条：第一轮点名的八形关没关 | 推的关了（八棵夹具原样复跑，待主 agent 跑） | 每一形都有对上的新测试与变异行；读码逐形对过改法 |
| 第 1 条：改法新开的少收 | 打中三形（推的，待跑） | 同一条 use 里改名再走相对路径、宏展开出的改名；工作区里与标准库宏同名的 `macro_rules!`；extern 判「同一个目标」，而链进最终产物的是测试目标、依赖包与构建依赖带来的库 |
| 第 1 条：改法新开的误拒 | 没打中新的；一处是设计钉着的 | `display()` 写在更深一层块里的 E16 形状照拒（测试 `bin_main_restarting_itself_through_a_binding_displayed_inside_a_deeper_block_is_refused` 钉着），只记 |
| 第 1 件 profile（改法第 1 条） | 打中（推的，增量探针待跑） | 生效 incremental 的链少了 cargo 配置 `build.incremental` 与它的环境变量 `CARGO_BUILD_INCREMENTAL`，`admission.py` 还把它写成「编出来的代码一样」并有自证格钉着；非路径依赖不增量，写明 `codegen-units = 256` 与不写在 dev 下同一行而依赖包切法不同 |
| 第 2 条：两份在第 1、4、5 件上 | 没打中两份之间的差 | 上面三形两份都漏（同一个判法、同一个洞）；`use … as` 那一支第一轮的不一致已对齐（复跑里第一轮那棵 `use-alias-leaf-divergence` 两边各喂一遍） |
| 第 2 条：工作区成员 | 打中（推的，待跑） | 内联空表、成员写在多行串里（TOML 开头换行要去掉）、`"."` 成员（python 收出 `""` 与 `"."` 两个 crate、缺名时叫 `.`）、绝对路径成员、仓根路径带 `[`、嵌套成员的文件 Rust 收两遍；今天的根清单一形都碰不到（核过 `Cargo.toml`） |
| 第 2 条：数据目录 | 没打中 | 两份都按「`<仓根>/crates/` 之下、不是它本身、盘上是目录」认，按整段比 |
| 第 3 条：判器组键为什么变 | 推不出（成员 A/B 与 `why` 写进复跑） | 验证包不依赖 `gpu-verification-server`（核过），能进它链接集的路读码排除了一半；变的只可能是多收，方向安全 |
| 第 4 条：第一轮第二节第 8 条的分支 | 补了（逐处对上） | 七处各有一行：表第 8、9、10、15、16、21、26 行 |
| 第 4 条：69 行红在不在名字说的断言上 | 抽 37 行读码：都红在点名测试的那条断言上，三处备注 | 第 60、62 行红在测试辅助的 `expect` 上；第 28 行的名字说「看全工作区的文件」，测试抓到的是另一种坏法；第 16 行测试分不出退回全连与点准 |
| 第 5 条：`TAIL_READ_SCOPES` | 13 行成立、2 行不合文件头的登法（推的实害无） | 相对 HEAD 加 15 行、删 3 行，不是 12 行；`crate-roles-check.py` 两行读被判那棵树的清单登成空元组 |
| 检查单：走不到的理由二次查实 | 成立 | 非测试段新加的没有 `todo!`、`unreachable!`、`assert!` |
| 检查单：类型与名字 | 成立（正文有一处说错） | 「算得出」与「codegen-units 是进键的设置…」两处 `expect` 在 `mod tests` 里，不在非测试段；四处非测试 `_ =>` 匹配的都是开放集合 |
| 检查单：不兼容旧代码与旧格式 | 成立 | |
| 检查单：不新建判定库 | 成立 | |
| 检查单：所有任务多机可用 | 不成立（推的，增量探针待跑） | 两台 `CARGO_BUILD_INCREMENTAL` 或 `CARGO_HOME` 下 `[build] incremental` 不同时键相同、工作区包切法不同 |
| 检查单：测试优先多线程 | 成立 | |
| 检查单：带 --item 的命令与项 | 推不出 | 两趟至多点了 43 行，69 行里至少 26 行没在这两趟；能不能都被 `--list-items` 列出要主 agent 跑 |
| 检查单：新崩溃流多罩了什么 | 成立 | 补丁里没有崩溃流与崩溃枚举用例 |
| 检查单：放的位置 | 成立 | 两处跨包 `include_str!` 构建脚本登在放行表与滞后表 |

四问一律按 `.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 答在各节末。这是代码轮，没有跑前条款给的改法，第四句答「无跑前条款；我自提的改法只在我的读法上推过、被攻过零轮」。

## 第 1 条：第一轮改法第 1 到 5 条合进来的样子

### 第一轮点名的八形（夹具第 2、3、5、6、10、11、12、13 条）

| 第一轮夹具 | 今天对上的测试（`crates/singlefs-unit-graph/tests/unit_graph_shapes_that_hide_the_callee_widen_the_closure_or_refuse_the_key.rs`） | 变异行（`mutation-rows-new.tsv` 行次） | 读码推的结局 |
|---|---|---|---|
| 第 2 条 `self::$f()` | `name_spliced_after_a_path_by_a_workspace_macro_links_the_function` | 5 | 连 `fshadow::helper_total` |
| 第 3 条 嵌套 `fn` | `name_in_a_block_moved_into_a_nested_function_by_a_workspace_macro_links_the_function` | 6 | 连 |
| 第 5 条 cfg 互斥的 `pub use` | `use_pointing_at_a_conditionally_compiled_candidate_links_the_re_exported_one_too`（夹具改成 glob 转出） | 11、12、13 | 连 `portable::checksum` |
| 第 6 条 相对路径首段是 `as` 改名 | `relative_use_path_starting_with_a_module_alias_links_the_aliased_module` | 14 | 连 `deep::real::helper_total` |
| 第 10 条 块里另一个同名绑定 | `binding_inside_a_block_with_the_use_after_the_block_is_refused_as_current_exe` | 17 | 拒算 `current-exe` |
| 第 11 条 `if … else { panic!() }` | `fallback_closure_ending_in_an_if_else_is_refused_as_current_exe` | 18 | 拒算 |
| 第 12 条 `cfg_attr` 里的 `link` | `link_inside_a_conditional_configuration_attribute_on_the_block_is_refused` | 22 | 拒算 `extern-function-interface` |
| 第 13 条 同一个 crate 另一个带 `#[link]` 的块 | `link_on_another_extern_block_of_the_target_refuses_the_listed_block` | 23、25 | 拒算 |

第一轮的八棵树原样复制进 `fixtures/round-one/`，复跑里两份各喂一遍（第 7 条 `use-alias-leaf-divergence` 也在，核两份对齐）。期望：第 2、3、5、6、7 条的闭包有点名的那一份；第 10、11 条拒算 `current-exe`；第 12、13 条拒算 `extern-function-interface`。有一条不是这样，第 1 条这一格改判「没关」。

### 打中一：同一条 use 里改名再走相对路径（还有宏展开出来的改名）

改法第 3 条只在「别的 use」引进首段时退回全连。Rust 那一份排掉的正是路径所在的这条 use：「Some(*other_use) != except_use」（`crates/singlefs-unit-graph/src/closure.rs:1833`），python 同一个判法：「return any(other is not except_use and imported_name in (other.imports or ()) for other in uses)」（`scripts/gate/lib_function_units.py:4724`）。

- 历史（夹具 `fixtures/use-relative-path-alias-in-the-same-use`，crate `fsameuse`，`deep::real::helper_total` 在第 3 到 5 行、`other::short::helper_total` 在第 11 到 13 行）：
  1. 模块 `through_one_use` 写 `use {crate::deep::real as short, short::helper_total};`，调 `helper_total()`。首段 `short` 由同一条 use 的另一支引进，被 `except_use` 排掉，`exact_module` 是 `None`，按末段 `short` 去配，点准到 `other::short`。
  2. 模块 `through_a_macro` 先 `bring_in_the_real_module_as_short!();`（宏体就是 `use crate::deep::real as short;`），再 `use short::helper_total;`。引进 `short` 的是条目级宏调用，不是 use 单元，`is_imported_by_another_use` 看不见它，同样点准到 `other::short`。
  3. 对照 `through_two_uses`：分两条 use 写，第一轮修过的那一形，退回全连。
- 运行时真调到的：三处都是 `deep::real::helper_total`（交 7）。探针 `tools/probes/use-relative-path-alias-in-the-same-use.rs` 是同一份源码加 `main`，复跑里 `rustc --edition 2021` 编了跑，打 `runtime through_one_use=7` 这样的行；编不过就撤回这一形（我对「同一条 use 里一支引另一支」能编过是印象里的，没核）。
- 期望：三个入口的闭包都有 `crates/fsameuse/src/lib.rs:3-5`。我推：`through_two_uses` 有；`through_one_use`、`through_a_macro` 两份实现都只有 `crates/fsameuse/src/lib.rs:11-13`、没有 `:3-5`（复跑行 `name=closure case=same-use-one-use …  found_lines=0`）。
- 违的：已定项 17「键过窄…同样算违反」（第一轮判决第二节第 3 条引的那一句）。

四问：分不分辨臂——只对「点准」与「退回全连」两种判法有分辨，与第一轮那一形同一类；系统看不看得到——看得到，同一条 use 的 `imports.names` 里就有 `short`，宏那一形要看条目级宏调用展开的 use，今天的索引没有；满足的分句——「键过窄」；改法——无跑前条款。

自提改法（只在我的读法上推过、被攻过零轮）：甲、首段检查不排除 `path_use` 自己的其他支（只排除引进 `name` 的那一支本身）；乙、相对路径首段只在它恰好是本模块里一个 `mod` 的名字、本作用域又没有任何条目级宏调用时才算点得准，别的一律退回全连。甲修第 1 步、不修第 2 步；乙两步都修，代价是多收。都是推的，待主 agent 跑。

### 打中二：工作区里与标准库宏同名的 `macro_rules!`

名单按宏名字面比：「.contains(&identifier_name(text_at(position - 2)));」（`crates/singlefs-unit-graph/src/closure.rs:1223`），python：「may_move_its_arguments = follows_a_macro_name and identifier_name(t(position - 2)) not in listed_macros」（`scripts/gate/lib_function_units.py:2544`）。宏名同字的工作区宏照算「就地求值」。

- 历史（夹具 `fixtures/shadow-through-a-macro-named-like-a-standard-one`，crate `fstdname`，`fn helper_total` 在第 14 到 16 行）：crate 根写 `macro_rules! format { ($function:ident) => { self::$function() }; }` 与 `macro_rules! vec { ($body:block) => {{ fn nested() -> u32 $body nested() }}; }`，入口 `let helper_total = 1u32; format!(helper_total) + helper_total` 与 `helper_total + vec!({ helper_total() })`。文本作用域里的 `macro_rules!` 盖过标准库预导入的同名宏（`no_std` 的 crate 自己定义 `println!` 是常见写法），这两个宏就是第一轮第 2、3 条那两种挪实参的宏，只换了名字。
- 运行时：两个入口都调到函数，交 8。探针 `tools/probes/shadow-through-a-macro-named-like-a-standard-one.rs`，复跑里编了跑。
- 期望：两个入口闭包都有 `crates/fstdname/src/lib.rs:14-16`；我推两份实现都没有（对照入口 `entry_without_a_binding` 有）。
- 违的：已定项 17 宏「不静默少收」那一句（第一轮判决第二节第 2 条）。

四问：分不分辨臂——分辨「按名字放行」与「按解析到标准库放行」；系统看得到——同一个 crate 里有同名 `macro_rules!` 单元，索引里查得到；分句——宏不静默少收；改法——无跑前条款。

自提改法（只在我的读法上推过、被攻过零轮）：名单里的名字只在看得见的目标里没有同名 `macro_rules!`（含 `#[macro_export]` 的、经 `use` 引进的宏）时才算就地求值。推的，待主 agent 跑。

### 打中三：extern 名单块只看「它编进的那个目标」，链接却按最终产物算

`target_may_link_libraries_beyond_the_system_library` 看两样：「&& workspace.membership(unit.file, target).is_some()」（`crates/singlefs-unit-graph/src/closure.rs:2359`），与这个 crate 的构建脚本目标里有没有一份文件字面含 `rustc-link-lib`：「.target_of_crate(crate_index, TargetKind::BuildScript, None)」（`crates/singlefs-unit-graph/src/closure.rs:2370`）、「&& file.source.contains(RUSTC_LINK_LIBRARY_INSTRUCTION)」（`crates/singlefs-unit-graph/src/closure.rs:2380`）。传进来的 `target` 是单元所在的目标：闭包按（单元，目标）对推进（`drain` 那一段），从测试目标连进库里的块时，`target` 是库目标。python 同一个判法：「for rust_file in (self.files[path] for path in target.files) for unit in rust_file.units)」（`scripts/gate/lib_function_units.py:4317`）、「for build_script in self.targets.values() if build_script.crate is target.crate and build_script.kind == "build"」（`scripts/gate/lib_function_units.py:4320`）。

链接是对最终产物（测试二进制、bin）做的：入口目标自己、它依赖的每个包、它们的构建脚本交出的库都进同一次链接。这几处带进来的库，两份都看不见：

| 夹具里的包 | 带进库的地方 | 名单块在哪 | 我推两份的判法 | 链接探针我推的读数 |
|---|---|---|---|---|
| `fexternfinal` | 集成测试 `tests/linked.rs` 里的 `#[link(name = "fakesync")] extern "C" {}` | 库 `src/lib.rs` | 不拒算 | 本地 `-l` 排在上游 rlib 前、`--as-needed` 下可能被丢，交 -1 或 4242 都可能（印象里的链接次序，没核） |
| `fexterndep` | 依赖包 `flinkdep` 的 `#[link(name = "fakesync")] extern "C" {}` | `fexterndep` 的库 | 不拒算 | 4242 |
| `fbuildlink` | 构建依赖 `fbuildhelper` 的函数打 `cargo:rustc-link-lib=fakesync`，`build.rs` 只调它 | `fbuildlink` 的库 | 不拒算 | 4242 |
| `fcclink` | 构建脚本用 `cc` crate 编 `src/shim.c`（定义 `fdatasync`、交 4242），字面量在 `cc` 里 | `fcclink` 的库 | 不拒算 | 4242 |
| `fexternclean`（对照） | 哪儿都没有 | 库 | 不拒算 | -1（C 库的 `fdatasync(-1)`） |

- 夹具在 `fixtures/extern-link-from-another-target-of-the-final-binary` 与 `fixtures/extern-link-through-a-build-dependency`。期望：前四个入口拒算 `extern-function-interface`，对照算得出键。
- 链接探针（复跑 `name=link_probe` 那几行）是判这一形的唯一本项目观测：交 4242 就是名单里的 `fdatasync` 在运行时调到了别的库，正是第 5 件放行的前提（「标准库的文件读写走同一层 C 库」）不成立的那一种；第一轮那一形（同一个 crate 另一个块）在这里换成同一次链接里的别的包。`cc` 这一形是 Rust 生态里给包加 C 代码的常见写法，构建脚本里没有那串字面量。
- 违的：已定项 17「键过窄」；用户定案第 40 条写的是「同一个目标里任一 extern 块带 link 或那个包的构建脚本里出现 rustc-link-lib 就不放行」（第一轮判决第三节第 4 条转述），实现照字面做了，打中的是定案的粒度。

四问：分不分辨臂——分辨「按目标」与「按最终产物（链接集）」两种粒度；系统看得到——闭包里有链接集（`build_link` 交的 `link.targets`），依赖包的 extern 块与构建脚本都在工作区里，`cc` 那一形看得到 `[build-dependencies]` 里有外部 crate；分句——键过窄；改法——无跑前条款。

自提改法（只在我的读法上推过、被攻过零轮）：判「多出库」时扫入口链接集里每个目标的 extern 块与每个包的构建脚本，另把「构建脚本或它的工作区依赖调了外部 crate」也当可能加库。推的，待主 agent 跑；这一改法在仓里会不会让现有名单块拒算（验证包、设备库的链接集大），没量。

### 改法新开的误拒（没打中新的）

- 起自己：用法要与 `let` 同一层花括号。仓里 E16 那一种形状（`display()` 写在起子进程失败时那个闭包的块里）照拒，测试 `bin_main_restarting_itself_through_a_binding_displayed_inside_a_deeper_block_is_refused` 把它钉成拒算，是有意多拒，只记。
- 我试过的别的绑定形（`for`、`match`、`if let`、不带花括号的闭包参数、宏里的 `let`）：同一层里出现的那个名字既不是 `Command::new(名)` 也不是 `名.display()`，整条照拒，不会误放行；`let` 之前的同名绑定不在 `is_bound_again` 的范围里，但用法检查按花括号层挡住了（读 `crates/singlefs-unit-graph/src/rust_items.rs` 的 `own_program_path_is_handed_to_command_unchanged`）。

### 第 1 件 profile：生效 incremental 的链少一层，非路径依赖不随它

改法第 1 条定的链是「根清单、`CARGO_PROFILE_<名>_INCREMENTAL`、`CARGO_INCREMENTAL` 依次盖」。cargo 另有一层：配置 `build.incremental`（环境变量形 `CARGO_BUILD_INCREMENTAL`）盖所有 profile、再被 `CARGO_INCREMENTAL` 盖（cargo 配置文档的那一条，我印象里的，本机没有 cargo 文档可核；增量探针是本项目的观测）。两份都不读它：

- Rust：文件头「//! 不认的：`.cargo/config.toml` 与 `--config` 里写的 profile、`[profile.<名>.package.<包>]` 与 `build-override`（今天工作区里一处都没有）。」（`crates/singlefs-unit-graph/src/build_toolchain.rs:7`），`build.incremental` 连这一句也没写，`CARGO_BUILD_INCREMENTAL` 不在 `environment_variables_the_profile_lines_read` 交的清单里，构建脚本不为它重跑。
- python：「"CARGO_BUILD_INCREMENTAL": "只定增量编译的缓存，编出来的代码一样"}」（`scripts/gate/admission.py:873`）、「BUILD_KEYS_LEFT_OUT = frozenset(("jobs", "target-dir", "incremental"))」（`scripts/gate/admission.py:876`），文件头另写「开取 256、关取 16，写了就照写（cargo 配置里写的 profile 与 [build] incremental 不读，同 Rust 那一份）。」（`scripts/gate/admission.py:27`）；自证格把它钉成键不变：「「单元键：构建环境只收改结局的设置：CARGO_BUILD_INCREMENTAL 与 [build] incremental 两种取值键相同、[alias] 加一条键不变，[profile.release] 改一项键变」」（`scripts/gate/admission.py:398`）。「编出来的代码一样」与第一轮判决第二节第 1 条（开不开增量，`codegen-units` 没写时的切法不同，探针 128 对 1）对不上。变异行键那一份同口径：「BUILD_KEYS_NOT_AFFECTING_OUTPUT = frozenset(("jobs", "target-dir", "incremental"))」（`scripts/mutation/lib_mutation_units.py:109`）。

另一处：cargo 只给工作区成员与 path 依赖开增量，registry 来的依赖一律不增量（同样是我印象里的 cargo 文档那一句）。键里那一行 `profile.codegen-units` 按工作区包的生效增量算；依赖包没写时按不增量的缺省切，写了就照写。所以 dev 下根清单写 `codegen-units = 256` 与什么都不写，键行都是 `profile.codegen-units: 256`，而依赖包一边是 256、一边是不增量的缺省。仓里 `Cargo.lock` 有 136 个 registry 包（`grep -c 'source = "registry'` 量的），测试二进制里都有它们的代码。

增量探针（复跑 `name=incremental` 五行）读的是 cargo 真交给 rustc 的参数与同一环境下 `unit-graph key` 的键。我推：

| 设置 | 工作区包 `-C incremental` | 依赖包 `cfg_if` `-C codegen-units` | 键 |
|---|---|---|---|
| default | 有 | 不写 | K1 |
| `CARGO_BUILD_INCREMENTAL=false` | 没有 | 不写 | K1（打中） |
| `.cargo/config.toml` 写 `[build] incremental = false` | 没有 | 不写 | K1（打中） |
| `CARGO_INCREMENTAL=0` | 没有 | 不写 | 不同于 K1（改法修了的那一形） |
| 根清单 `[profile.dev] codegen-units = 256` | 有 | 256 | K1（依赖包那一半打中） |

推翻条件：第二、三行工作区包仍带 `-C incremental=`，这一形撤回；第五行依赖包不带 `-C codegen-units=256`、或 default 那一行依赖包带 `-C incremental=`，依赖包那一半撤回。

四问：分不分辨臂——分辨「三层链」与「cargo 的完整链」；系统看得到——构建脚本拿得到 `CARGO_BUILD_INCREMENTAL`（环境变量），配置文件那一层要照 cargo 的合成读；分句——已定项 17「工具链（含 cargo 解析后的生效构建环境与外部工具版本）」；改法——无跑前条款。

自提改法（只在我的读法上推过、被攻过零轮）：甲、链里加 `build.incremental`（环境变量与配置文件两种），`admission.py` 与 `lib_mutation_units.py` 的 `incremental` 不再按「不改结局」去掉，那条自证格改成「两种取值键不同」；乙、`codegen-units` 写了与没写分开出行（没写时出「按增量缺省」的字样，不出数），依赖包那一半就分开了。推的，待主 agent 跑。

## 第 2 条：Rust 与 Python 两份

### 第 1、4、5 件

上一节三形与 profile 那一形，两份是同一个判法、同一个洞（各处引了两边的原文），不算两份之间的差。第一轮坐实的 `use … as` 改名那一支：今天两份都退回全连（Rust 第 29 行变异钉着，python 同一段 `alias_followed_to_the_original` 开关下才顺原名），复跑里第一轮那棵 `use-alias-leaf-divergence` 两份各喂一遍，期望两份都有 `crates/fdiverge/src/lib.rs:12-14`。入口判法（第一轮第二节第 4 条正推腿指的「比根文件第一个顶层 `main`」与「比入口本身」）这一轮没改，我没另攻。

### 工作区成员

python 那一段：「for match in sorted(glob.glob(os.path.join(root, pattern))):」（`scripts/gate/lib_function_units.py:4144`）、「relative = os.path.relpath(match, root).replace(os.sep, "/")」（`scripts/gate/lib_function_units.py:4145`）、「    if workspace:」（`scripts/gate/lib_function_units.py:4141`）、「        directories.insert(0, "")」（`scripts/gate/lib_function_units.py:4153`）、缺名：「self.package = package.get("name", posixpath.basename(directory) or "root")」（`scripts/gate/lib_function_units.py:4091`）。Rust 那一段：成员通配按「        .split('/')」（`crates/singlefs-unit-graph/src/workspace.rs:1606`）从仓根逐节往下，内联表「([], TomlValue::InlineTable(pairs)) => pairs」（`crates/singlefs-unit-graph/src/workspace.rs:1469`），仓根缺名「Some(_) | None => "root".to_string(),」（`crates/singlefs-unit-graph/src/workspace.rs:1684`）。

| 夹具 | 两份读出来的（推的） | 复跑看哪一行 | 真实仓碰不碰得到 |
|---|---|---|---|
| `workspace-inline-empty-table`：`workspace = {}` 加 `crates/alpha` | Rust：有 workspace、`members` 空，一个成员都没有；python：`{}` 为假，退回 `crates/`，收 `alpha` | `name=dump fixture=workspace-inline-empty-table … identical=no`；`case=inline-empty` 两份 `found_lines` 一个 0 一个 1 | 碰不到：根清单写的是 `[workspace]` 表头与键（核过 `Cargo.toml` 第 4 到 20 行） |
| `members-in-a-multi-line-string`：`members = ["""` 换行 `crates/alpha"""]` | python 的 `tomllib` 去掉开头那个换行（本机 `python3 -c 'import tomllib…'` 读出 `{'workspace': {'members': ['crates/alpha']}}`，量的）；Rust 的多行基础串照原样收（读 `crates/singlefs-unit-graph/src/manifest.rs` 的 `read_basic_string`），首节是换行加 `crates`，收不到 | 同上，`case=multi-line-string` | 碰不到 |
| `members-dot-unnamed-root`：只有 `[workspace] members = ["."]`、没有 `[package]` | Rust：仓根是成员，名 `root`；python：`relpath` 交 `.`，crate 目录是 `.`、名 `basename(".")` 即 `.` | `case=dot-unnamed`：入口 `root::root_value` Rust 找得到、python 找不到 | 碰不到；cargo 也不认这种根清单（虚清单把自己列成成员，印象里会报错） |
| `members-dot-with-root-package`：`members = ["."]` 加 `[package] name = "rootpkg"` | Rust：`.` 规范化成仓根，与 `[package]` 那一个去重，一个 crate；python：`"."` 与 `""` 两个字串都进 `directories`、`dict.fromkeys` 去不掉，两个 crate 同名 `rootpkg` | `case=dot-named`：两份结局（找没找到、找到几个）| 碰不到；cargo 认这种写法（多写一个 `.` 无害，印象里的） |
| `members-with-an-absolute-path`（复跑现生成，成员写运行目录下的绝对路径） | python：`os.path.join` 遇绝对路径取它本身，收进仓根外的 crate；Rust：按 `/` 切、空节丢掉，接在仓根后面，收不到 | `case=absolute-member` | 碰不到 |
| `root[with]brackets`（复跑把一棵普通夹具拷到名字带 `[`、`]` 的目录） | python 把仓根路径也交给 `glob`，`[with]` 当字符类，一个成员都配不上；Rust 从仓根起、只把成员那一截当通配 | `case=brackets` 与它的 dump | 碰不到：仓根是固定路径；副本与运行目录的名字没见过带 `[` 的（推的） |
| `nested-member-inside-another-member`：`crates/outer` 与 `crates/outer/inner` 都是成员 | Rust 只给仓根那个 crate 去掉底下别的 crate 的文件（`crates/singlefs-unit-graph/src/workspace.rs:1795` 那一句只判仓根），`outer` 照收 `inner/src/lib.rs`，同一份文件在 dump 里两遍；python 按最长前缀把文件归给 `inner`、只一遍 | `name=nested_member_file_lines rust=… python=…`（我推 2 对 1） | 碰不到：今天的十三份 `Cargo.toml` 没有套着的（`find … -name Cargo.toml` 核过）。同一个洞在第一轮之前就有（顺 path 依赖收进来的嵌套包），这次只是成员也能走到 |
| `members-sharing-a-directory-name`：`crates/common`（包 `commonlib`）与 `tools/common`（包 `commontool`）各有 `tests/smoke.rs` | python 的目标 id 是「目录名:种类:名」，两个 `common:test:smoke` 撞成一个；Rust 按入口首段的 crate 名选 | `case=same-name-tool`、`case=same-name-library` 两份各找自己那一份 `src/lib.rs:1-3` | 碰不到：今天成员的目录名各不相同 |

`"."` 成员缺名那一处（正文「推的，没量」那一条）：真差，但只在仓根清单没有 `[package]` 时才差（有 `[package]` 时两份都取 `root` 或写的名字，差的是 python 多一个同名 crate）。内联空表那一处：真差，方向是 Rust 少收（一个成员都没有、闭包空）、python 多收。

### 数据目录

python：「return path.startswith("crates/") and not path.startswith("../") and os.path.isdir(os.path.join(self.root, path))」（`scripts/gate/lib_function_units.py:3870`）；Rust 在调 `directory_fold` 之前就判了「是文件」「不是目录」，再按整段比 `<仓根>/crates`。`crates-extra/…` 这类前缀相同的名字两份都不收（python 比的串带着 `/`，Rust 按路径段比）；仓根外、`crates/` 本身两份都拒。没找到两份不同的输入。

## 第 3 条：`gpu-verification-server` 进成员之后 `observer_conclusion_composition` 的组键为什么变

核过的：验证包的 `Cargo.toml` 不依赖 `gpu-verification-server`（反过来是服务依赖验证包、`prune-server` 与核心层，`gpu-verification-server/Cargo.toml` 的 `[dependencies]`）；`crates/*/Cargo.toml` 没有一份写 path 依赖指到它（`grep -rn 'gpu-verification-server' crates/*/Cargo.toml` 零命中）。所以补丁之前它根本不在工作区里（不在 `crates/` 下、也没人顺 path 依赖收它），补丁之后它是成员；验证包 `lib` 目标的链接集（`build_link` 从入口目标顺依赖收）里没有它。

链接集管不到、又会因为多了一个成员而变的路，读码找到三条（都是推的，哪条是真的我没定）：

1. trait 头进闭包时按名字找 impl：「.blanket_implementations_by_trait_name」与「.implementations_by_self_type_name」两处按名字查全工作区的索引（`crates/singlefs-unit-graph/src/closure.rs` 的 `visit_unit_for_target`，trait 头那两段），我没读到按链接集筛。服务里写给自己的类型的 impl 有十几个（`grep -rn '^\s*impl\b' gpu-verification-server/src` 列的），名字撞上验证包闭包里的某个 trait 头或类型名就会被带进来。
2. 工作区外的 crate「按调用关系连」：`names_reaching_crates_outside_the_workspace` 对全部 crate 的 use 与路径收「连得到 `prune-server` 的名字」。服务经 `use prune_server::…` 引进的名字里，`GPU_CLAIM_FAMILY_PREFIX`、`ClaimReleaseOutcome`、`gpu_claim_key`、`install_termination_handlers`、`is_interrupted`、`ReconnectPolicy` 在 `crates/` 的七个包源码里一处都没有（`grep -rlw` 量的），路径段 `interruption`、`gpu_judging_rows`、`server`、`configuration` 算不算「引进来过」我没读到；验证包闭包里同名的裸名（字段、局部变量不算）因此新连到 `prune-server` 的同名条目。
3. 外部类型上的本仓 trait impl（`foreign_type_trait_implementations`）放行成员时看不看链接集，我没读到。

分辨这三条的是复跑里的成员 A/B：同一份冻结树，根清单删不删 `gpu-verification-server` 那一行，同四个入口（`judging_units_entries.rs` 里那一组登记的）各收一次闭包、逐行比，`name=members_ab_only_with_the_server line=…` 列出只在有服务那一臂里的行；再 `why --unit gpu-verification-server/` 与 `--unit prune-server/` 列它们是经哪条边头一回进来的。多出来的行在 `gpu-verification-server/` 下是第 1 或第 3 条，在 `prune-server/` 下是第 2 条；两臂逐行相同就说明变的不在单元上（那时看键行，再查依赖行与工具链行），我这三条都撤回。

该不该变：验证包的代码与链接集都没变，它运行时调不到服务里的代码，照「键只随闭包里的单元变」，理想是不变；变了是多收，方向安全（多一次判定失效，不会少判）。第 2 条若坐实，变是那条规则照字面做出来的（「在工作区里」引进过就算），要收窄得改规则本身，不是这次补丁的毛病。别的 92 组没变：上面任一条都只碰闭包里恰好有那几个名字的组，与「只有一组变」相容，但不证明别的组不该变；要逐组定，在 A/B 里把 `judging_units_entries.rs` 的每一组都收一遍（没写进复跑，组多、整仓闭包每组一次，代价我没估）。

## 第 4 条：没有变异行的判定分支与 69 行新变异

### 第一轮第二节第 8 条点名的分支

| 第一轮点名（第一轮快照的行号） | 今天对上的行（`mutation-rows-new.tsv` 行次与名字） |
|---|---|
| `closure.rs:1190-1191` 宏调用标记往里传 | 第 8 行「宏调用标记不往里层的括号传」；新加的「会挪走实参」标记往里传另有第 6 行 |
| `:1659` 改名原名不看遮法 | 第 9 行「改名的原名也看局部绑定」 |
| `:1236` 区间 `..名` | 第 10 行「区间的两个点也当字段的点」 |
| `:1865` 每一支都点到才算 | 第 15 行「一支点不到也只收点到的那几支」 |
| `:1861` 一支叶子都找不到时 | 第 16 行「一支叶子都找不到时一份都不连」 |
| 起自己「至少起一次」 | 第 21 行「路径只显示、没交给 Command::new 也算起自己」 |
| extern「`link_name` 之外的属性就拒」 | 第 26 行「声明前别的属性当 link_name 读」 |

都补了。第一轮正推腿指的第 79 行（索引越界 panic）那一类：今天 profile 那几行不以「单元图」起头，不在这 69 行里，在 `crates/mutations.tsv` 以「构建档键」起头的十一行（第 8251 到 8261 行，`awk -F'\t' '$2 ~ /build_toolchain.rs/'` 列的），我没逐行判。

### 69 行抽查（读码推的，没跑变异）

抽了 40 行：第 2、5 到 30 行（除第 27 行）、第 46 到 49 行、第 51 到 54 行、第 57、60、62、63、64、68 行。每行把替换文套进源码，顺着点名的测试走到它的断言：

- 都红在点名测试自己的那条断言上，断言说的就是名字说的那一形：局部绑定那几行（第 2、5 到 10、30 行与第 46 行）各改坏一处，对应入口的闭包有没有 `fshadow::helper_total`（第 9 行是 `fshadow::aliased::helper_total`）翻过来；use 路径那几行（第 11 到 16、29 行）点准与退回全连翻过来；起自己那几行（第 17 到 21、47、48 行）拒算 `current-exe` 与算得出键翻过来；extern 那几行（第 22 到 26 行）拒算 `extern-function-interface` 与算得出键翻过来；工作区成员那几行（第 49、51 到 54、57、63、64、68 行）`crates_by_package_name` 那条 `assert_eq!` 的集合变了。
- 三处备注：
  1. 第 60、62 行（「有 workspace 而一个成员都展开不出时报读不了」「没有 workspace 而一个成员都收不到时报读不了」）把 `load_workspace` 改成交错误，红在测试辅助 `workspace_of` 取工作区那一步的 panic 上，不在两条 `assert_eq!` 上；测试要说的正是「不报错」，语义对得上，只是红的位置不是断言。
  2. 第 28 行名字说「构建脚本那一支看全工作区的文件」，替换是 `&&` 换成 `||`：点名测试 `listed_calls_without_any_link_are_still_keyed` 红，是因为构建脚本自己那份文件（不含字面量）也被算成「加了库」；名字说的那一形（工作区别处有一份文件写了 `rustc-link-lib`）夹具里没有，单靠这一形改坏的变体它测不出。
  3. 第 16 行的测试只断言闭包里有 `counting::total`：点准与退回全连两种都满足，它红是因为改坏之后一份都不连；名字说的就是「一份都不连」，对得上，但它测不出「点准」这一种坏法（那一种由第 15 行的测试管）。

## 第 5 条：`TAIL_READ_SCOPES` 新登的行

正文说补 12 行。`git diff HEAD -- scripts/gate/lib_function_units.py` 里这张表加了 15 行、删了 3 行（删的是 `task-server` 下三处：`package_directories`、`prove_red_uses_gpu`、`mutation_gate_card_need`；换上的那份 Python 补丁 `python-scripts.patch` 不碰这张表），净多 12 行。15 行逐行读了被登的那一处：

| 登的那一处 | 前缀 | 判 |
|---|---|---|
| `apply-writer-patch.py` `plan_with_three_way` 读 `result.md`、`mutations-append.tsv`、`mutations-replacements.tsv`、`mutations-delete.txt` 四行，`snapshot_touched_paths` 读 `os.path.join(patch_directory, name)` 一行 | 空元组 | 成立：`patch_directory` 是命令行给的补丁目录（`plan_with_three_way(patch_directory, root)`，`name` 取 `MUTATION_FILES`） |
| `merge-and-verify.sh` `mutation_rows` 读 `os.path.join(directory, file_name)` | 空元组 | 成立：同名函数有两个（第 1970 行与第 3468 行），登的表达式只在前一个里；`directory` 取 `select_targets` 的 `patch_directories`（`sys.argv[6:]`） |
| `replace-batch.py` `selftest` 读 `first.md` | 空元组 | 成立：`tempfile.TemporaryDirectory()` |
| `session-checks.py` 两行读 `records.tsv` | 空元组 | 成立：`run_selftest_cases`、`run_hook_cases` 里的 `scratch` 都是临时目录；平常跑读的 `SESSION_CHECKS_RECORD` 那一处是另一条表达式，没登 |
| `expansion_table.py` `expansion_rows` 读 `mutations.tsv` | 空元组 | 成立：写进临时目录的是这一次交进来的表文本 |
| `mutation-shard-run.sh` 内联正文读 `f'{directory}/mutations.tsv'` | 空元组 | 成立：自证那一格自己建、前一段刚写出的 |
| `e165-replay-mutation-kill-bounds.sh` `source_hashes` 读 `os.path.join(setup.root, path)` | `("crates",)` | 成立，不是「调用方输入」那一类：`path` 只取 `SOURCE_FILES` 两份 `crates/` 下的源码，前缀写的是真走得到的地方 |
| `crate-roles-check.py` `cargo_projects` 读 `os.path.join(root, 'Cargo.toml')` | 空元组 | 不合文件头的登法 |
| `crate-roles-check.py` `judge` 读 `os.path.join(root, directory, 'Cargo.toml')` | 空元组 | 不合文件头的登法 |

最后两行：文件头写「读调用方给的根上的根级文件（Cargo.toml、Cargo.lock）登 "."；只有读到的值已由调用方解析之后写进键的（依赖行、工具链行、cfg、crate 起法）才登空元组，」（`scripts/gate/lib_function_units.py:261`）。这两行自己的理由写「平常跑（门禁 code-tooling 的 crate-roles 格）是 $ROOT 给的被判那棵树，都由调用方给，不是判法自己的数据；」（`scripts/gate/lib_function_units.py:14361`）：读的正是被判那棵树的根清单与 `members` 点名的成员清单，读到的值也没有被谁解析进键，照文件头该登 `"."` 与成员目录的前缀（同表 `admission.py` 的 `manifest_dependency_names` 就是这么登的）。实害（推的）：`crate-roles-check.py` 与起它的 `code-tooling.sh` 都是 `admission: always`，平常跑不按键复用；它的自证只读临时树，登空元组对自证那把键反倒是准的。所以是登法不一致、没有放出一次该判没判的；哪天有按键复用它平常那一跑的入口，根清单改了键不变。

没有登进去而该拒算的：上面十五处读的都不是 `records/`、`research/` 下的东西。

## 检查单九格

1. **走不到的理由二次查实：成立。** `changes.patch` 第 1 到 1729 行与第 3008 到 3038 行（非测试文件）加的行里，`todo!`、`unreachable!`、`assert!` 一族只出现在 `build_toolchain.rs` 的 `mod tests`（`crates/singlefs-unit-graph/src/build_toolchain.rs:313` 起）与宏名单的字串里；`panic!` 只在名单与文件头注释里。
2. **类型与名字：成立；正文有一处说错。** 非测试段新加的 `expect` 只有一处：「.expect("内置根 profile 的默认值里有 incremental，上面只往里盖、不删");」（`crates/singlefs-unit-graph/src/build_toolchain.rs:243`），它说出了依赖的不变量（两个根 profile 的默认表都有 `incremental`、之后只 `insert`），我核了两张默认表与三处只往里盖的循环，成立。正文列的「算得出」「codegen-units 是进键的设置，每份生效值都有这一行」两处在第 355、365、368 行，都在第 313 行起的 `mod tests` 里，不是非测试段。非测试 `_ =>` 四处（`workspace.rs` 收根清单键值的三处、括号表达式读区间的一处）匹配的是键路径切片、字段名字串与字符，都是开放集合；`TomlValue` 的两处匹配逐个列了成员、没用 `_`。新名字我没找到说不出它是什么的；`target_may_link_libraries_beyond_the_system_library` 名字说的「目标」正是第 1 条打中三的粒度，名实相符、判法窄。
3. **不兼容旧代码与旧格式：成立。** 补丁加行里没有「旧」「legacy」「兼容」一类（`grep -nE` 只命中测试夹具里的函数名 `old_total` 与一句注释「键还是旧的」）；`NoCrates` 删了。
4. **不新建判定库：成立。** 加行里没有建目录、开文件写、`.tsv`、sqlite、RocksDB。
5. **所有任务多机可用：不成立（推的，增量探针待跑）。** 改法第 1 条修了 `CARGO_INCREMENTAL` 与 `CARGO_PROFILE_<名>_INCREMENTAL` 两台不同的那一形；两台 `CARGO_BUILD_INCREMENTAL` 不同、或各自 `CARGO_HOME` 下 `config.toml` 的 `[build] incremental` 不同时，键相同、工作区包的切法不同（第 1 条 profile 那一节）。任务服务自己不设这几个变量（`grep -rn 'CARGO_INCREMENTAL\|CARGO_BUILD_INCREMENTAL' task-server/` 零命中），差异只会来自两台的进程环境与 `CARGO_HOME`。本机 `~/.cargo/config.toml` 不在（`ls` 核过），第二台我看不到。
6. **测试优先多线程：成立。** 新测试是毫秒级小夹具，不走崩溃枚举，切片并行省不出东西。
7. **带 --item 的命令与项：推不出。** 两趟点了 21 行与 22 行，就算不重叠也只有 43 行，69 行里至少 26 行不在这两趟里；profile 那十一行（「构建档键」起头）不在 69 行里，跑没跑我不知道。能不能都被 `--list-items` 列出，命令在下一节第 4 条。
8. **新崩溃流多罩了什么：成立。** 补丁里没有 `crash-case:` 登记，也没有调 `enumerate_crash_states` 一族的测试。
9. **放的位置：成立。** 新文件都在 `crates/singlefs-unit-graph/{src,tests}/`；`build_toolchain.rs` 测试里两处 `include_str!` 读别的包的 `build.rs`，登在放行表 `.claude/gate.d/crate-cross-references-allowed.tsv` 第 5 行与滞后表 `.claude/gate.d/crate-cross-references-lag.tsv` 第 4 行，照 `.claude/rules/directory-layout.md@crates/：各包放什么` 跨包引源码那一条与「门禁管哪一半」表里的登法（转述）。

## 要主 agent 跑的命令

1. `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/unit-graph-src-backfill-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-attack/frozen /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-attack-run /home/fy5090/code/singlefs/research/prompts/unit-graph-src-backfill-code-r2-opus-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（在新 shell 里起）。
   - 看：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 `name=frozen_check files_ok=17 verdict=ok`、`name=build_unit_graph exit=0`、`name=rerun_done process_groups_left_behind=0`。判各条的行：
     - 第一轮八形：`name=closure case=r1-…` 那几行。`r1-macro-path`、`r1-macro-nested`、`r1-cfg-swap`、`r1-module-alias`、`r1-alias-leaf` 两份都 `found_lines=1`；`r1-relaunch`、`r1-fallback` 两份 `refused_forms` 有 `current-exe`；`r1-extern-attribute`、`r1-extern-other-block` 两份 `refused_forms` 有 `extern-function-interface`。不是这样的，第 1 条第一格改判「没关」。
     - 打中一：`case=same-use-two-uses` 两份 `found_lines=1`（对照）；`case=same-use-one-use`、`case=same-use-macro` 两份 `found_lines=0` 是打中，`=1` 就撤回那一形；`name=runtime probe=use-relative-path-alias-in-the-same-use` 三行都是 `=7`，`name=runtime_compile … exit=` 非 0 就撤回那一形。
     - 打中二：`case=std-name-control` 两份 `found_lines=1`（对照）；`case=std-name-format`、`case=std-name-vec` 两份 `found_lines=0` 是打中；`name=runtime probe=shadow-through-a-macro-named-like-a-standard-one` 两行都是 `=8`。
     - 打中三：`case=extern-control` 两份 `refused_lines=0`（对照）；`case=extern-test-target`、`extern-dependency`、`extern-build-helper`、`extern-cc-crate` 两份 `refused_lines=0` 是打中；`name=link_probe` 五行：`fexternclean … fdatasync-result -1`，其余四行里是 4242 的那几形坐实「运行时调到了别的库」，是 -1 或 `exit` 非 0 的那一形只剩「链接时多了库」这一半（看 `out/link-<包>.txt`）。
     - profile：`name=incremental` 五行，照第 1 条 profile 那一节的表与推翻条件判；`build_exit` 非 0 的那一行作废（多半是那台缓存里没有 `cfg-if 1.0.4`，换本机重交）。
     - 工作区成员：`name=dump fixture=…` 八行的 `identical=` 与 `name=closure case=dot-named|dot-unnamed|inline-empty|multi-line-string|brackets|absolute-member|nested|same-name-tool|same-name-library` 两份的 `exit=`、`found_lines=`；`name=nested_member_file_lines`。两份一样的那一形撤回。
     - 第 3 条：`name=members_ab arm=…` 两行、`name=members_ab_difference`、`name=members_ab_only_with_the_server line=…`、`name=members_ab_why …`，再看 `out/members-ab-why-*.txt`。
   - 跑完照共用约束删运行目录 `/home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-attack-run`（里面有编译目录与两份整仓树）与冻结副本。
2. `task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items /home/fy5090/code/singlefs`
   - 在哪起：仓根。
   - 看：`exit=0`；日志里以「单元图」起头的变异名逐行出现，数一遍是不是 69 行（`grep -c` 数），锚点失效那一段有没有点到这 69 行里的。检查单第 7 格照它改判。

## 没打中的形状

- 第 2 件宏名单：名单里的 16 个宏我逐个想了实参会不会被挪走（`write!` 的第一个实参、`panic!` 两种版次、格式串捕获），都是就地求值；嵌套在名单宏里的别的宏、别的宏里套名单宏，标记往里传对了（第 6、8 行变异也钉着）。打中的只有「同名的工作区宏」这一形。
- 第 3 件：`self::`、`crate::`、`super::` 起头的路径点到改名的那一份时，那个模块里不可能同时有同名的 `mod`（rustc 报名字冲突），点到的模块不存在就退回全连；`extern crate … as` 改出的首段指的是别的 crate，跨目标的候选本来就照连（不按模块筛）；glob 引进的首段被同名的显式条目盖住。都没打出少收。
- 第 4 件：起自己的「同一层花括号」我试了 `for`、`match`、`if let`、无花括号的闭包参数、宏调用里的 `let`、`let` 之前的同名绑定、`Command` 是自己定义的类型；前几种整条照拒；最后一种是「起别的程序本来就不判」那个既有缺口（第一轮判决第二节第 4 条）。`expression_ending_at_diverges` 只认紧跟 `|` 与 `=>` 的块：`|_| -> ! { … }` 这种带返回类型的闭包会被多拒，方向安全。
- 第 5 件：`#[link]` 写成块里的内属性 `#![link(…)]`：块里出现 `fn` 之外的东西就整块拒算，不会放行。
- 数据目录：前缀同名的兄弟目录、`..` 走出去再走回来、仓根自己那个包读自己的目录，两份同判。
- 工作区成员的通配：`*`、`?`、`[…]`、`[!…]`、反着写的区间、没闭上的 `[`、`**`、带尾 `/` 的成员、`.`/`..` 段、点起头的名字，两份同判（读 Rust 的 `component_pattern_matches` 与 python 3.12 `glob`/`fnmatch` 的那几条规则，python 那一半是我印象里的标准库行为，没逐条量）。`exclude` 两份都按整条路径相等比，cargo 本身按前缀比（印象里的），两份一起与 cargo 不同，不算两份之间的差。

## 这条腿自己的限度

- 打中的六形（同一条 use、宏展开的改名、同名宏、extern 粒度、`build.incremental`、非路径依赖）与工作区成员七形，全是读码推的，没喂过一次；复跑出结局之前不当结论。
- 三处 cargo 与 rustc 的外部事实是我印象里的、本机没有文档可核：`build.incremental` 与 `CARGO_BUILD_INCREMENTAL` 盖 profile；registry 依赖不增量；`use {a as b, b::c}` 这种一条 use 里互相引的写法能编过。每一处都配了本项目的探针（增量探针、运行期探针），探针读数与我说的不同就照那一节写的推翻条件撤回。链接次序（本地 `-l` 排在上游 rlib 前、`--as-needed`）也是印象里的，所以测试目标那一形我没押 4242。
- 第 3 条我只找到候选的机制，没定是哪一条；A/B 只收了那一组的四个入口，别的 92 组该不该变没量。
- 69 行变异读了 40 行，没读的是第 1、3、4、27、31 到 45、50、55、56、58、59、61、65 到 67、69 行（第 31 到 45 行与第 30 行同一个写法，各删名单里的一项）；变异一行都没跑。
- 正文说的「`TAIL_READ_SCOPES` 补 12 行」我按 `git diff HEAD` 读成加 15 删 3；那 3 行删的是谁、为什么删，不在这一轮被判的范围里，我没追。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没有副本上量出的数，模型里的数都待主 agent 在冻结副本上复跑。
- 没往统一任务服务交任务；判决要引的数都在「要主 agent 跑的命令」第 1 条里，没复跑。
- 没改 `crates/` 与治理文件，没建仓副本以外的编译目录。草稿目录里建的冻结副本 `/home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-attack/frozen`（109M，`du -sh` 量的）留给主 agent 当 `--frozen` 用，在 `.deliverables` 写成「交接」；运行目录由复跑自己建，跑完由主 agent 删。
