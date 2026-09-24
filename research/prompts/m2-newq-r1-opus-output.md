# m2-newq-r1 云端攻方（Opus）报告：N1、N2、N3

本腿接续一条撞了会话限额的同立场腿（交接摘要 `/tmp/claude-1000/handover/a89a2bee5683faebd.md`）：那条腿做的副本改动、第一版驱动与扫描（`out/sweep-len4.tsv`、`out/n2-*`、`out/ring-*`、`out/long-F4-*`、`out/long-F6-*`）原样沿用，本腿核过现场后接着补了故障 F8–F11、N1 候选丁、N2 读法乙、N3 的跨重挂历史，以及被 /dev/shm 写满污染的几组重跑。时刻均为 UTC（东京 = UTC+9）。

## 一、复跑

模型目录 `research/prompts/m2-newq-r1-opus-model/`（下称 `M`）。基线：主工作区 2026-09-24 01:22 UTC 的 `crates/`，打包在 `M/baseline-crates.tgz`（不含 `mutations.tsv`），逐文件 sha256 在 `M/baseline-crates.sha256`；工作区根的其余文件取同一时刻的主工作区（本腿草稿里的 `/tmp/claude-1000/m2-newq-opus/repo` 留着那一份）。

```
W=<草稿目录>; rsync -a --exclude target --exclude .git <那一刻的仓>/ $W/repo/
rm -rf $W/repo/crates && tar xzf $M/baseline-crates.tgz -C $W && mv $W/pristine-crates $W/repo/crates
# 第一、二段（最小释放核验、N1/N3 各臂、N2 读法甲）+ 第一版驱动
python3 $M/patch/apply.py $W/repo && python3 $M/patch/apply2.py $W/repo
cp $M/patch/m2_newq_attack.rs $W/repo/crates/singlefs-harness/tests/
# 第三段（写者错注入、N2 读法乙）+ 第三版驱动；第四段（N1 候选丁）+ 第四版驱动
rsync -a $W/repo/ $W/repo3/ && python3 $M/patch3/apply3.py $W/repo3 && cp $M/patch3/m2_newq_attack3.rs $W/repo3/crates/singlefs-harness/tests/
rsync -a $W/repo3/ $W/repo4/ && python3 $M/patch4/apply4.py $W/repo4 && cp $M/patch4/m2_newq_attack4.rs $W/repo4/crates/singlefs-harness/tests/
# 驱动 3、4 也可以由 make_driver3.py / make_driver4.py 从上一版现生成（在 M 下原地跑，输出同名文件）
for v in "" 3 4; do (cd $W/repo$v && CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR=$W/target$v nice -n 19 cargo test -p singlefs-harness --test m2_newq_attack$v --no-run); done
# 扫描（每条命令的 BIN 是上面对应 target 下 deps/ 里的 m2_newq_attack*-<hash>；镜像放内存盘，同一时刻只跑一路）
TMPDIR=/dev/shm/x NEWQ_OUT=out/sweep-len4.tsv NEWQ_MAXLEN=4 NEWQ_WORKERS=4 nice -n 19 $BIN --nocapture   # 上一条腿原命令（它的二进制没有 N2 甲那一臂）
TMPDIR=/dev/shm/x NEWQ_OUT=out/n2-<臂>.tsv NEWQ_ARM=<臂> NEWQ_MAXLEN=4 nice -n 19 $BIN2 --nocapture       # 两个臂：N1A-any-isoAll-mem、N2-same-verdict-rebuild-tolerates-data（原命令没留下，按产物形状反推）
TMPDIR=/dev/shm/x NEWQ_OUT=out/ring-<故障>-<臂>.tsv NEWQ_FAULT=<F4|F6> NEWQ_ARM=<臂> NEWQ_SUFFIX="$(cat $M/patch/suffixes-ringwrap.txt)" nice -n 19 $BIN2 --nocapture
bash $M/patch/run-long.sh $BIN2 out          # F4、F6 × 五臂 × 729 条；F3、F5 两组 every 的结果以 rerun 那份为准
bash $M/patch3/run3.sh $BIN3 out3            # F9、F10 × N3 各臂 × 两种尾巴；F8、F10 × N2 各臂；F9 × N1 各臂
bash $M/patch4/rerun-after-enospc.sh         # 重跑被污染的：F10 × 重挂现算两组（→ out3r）、run4.sh 全部（→ out4r）、F3/F5 every（→ outr）；脚本里的路径按 W 改
# 汇总
python3 $M/patch/summarize.py <tsv…>;  python3 $M/patch3/summarize3.py <tsv…>
```

报告里引的产物行，都是在 `M/out*/` 对应文件上用 `awk -F'\t' '$3=="<后缀>"' <文件> | cut -f<列>` 取出的（列号：1 故障、2 臂、3 后缀、4 各步、5 最终挂载、6 读回、7 checker 违例、8 两盘隔离数、9 核验隔离数、10 重挂现算隔离数、11 `free − 可发`、12 核验读次数、13 核验读字节、14 切换次数、15 被拒的读、16 核验事件；第三、四版多 17 复用步号、18 重建比出的不一致、19 被拒的写）。模型目录里 `out/` 放第一、二版驱动与 `outr` 重跑的产物，`out3/` 放第三版（含 `out3r` 重跑的 F10 × 重挂现算两份），`out4/` 放第四版（全部是 `out4r` 重跑）。

### 模型目录逐文件 sha256（`cd M && find . -type f | sort | xargs sha256sum`）

```
b02c75af872057ad0355a2bac9051986e4572615c7645b7fec52d57ac487b55f  ./baseline-crates.sha256
c37f17c0ea63a222a85da752e9f8c5b79208580277ab46b404b293a0a2c068d9  ./baseline-crates.tgz
f9d6ee7e82c1e8060939b495ce7a6f1077296cead04ca3a6d9be2a3b67b7dcf1  ./out3/longsmoke-F10-N1A-any-isoAll-mem.tsv
62749af7c36df404e197696a095c808fd379360cae9a4f60d3809e8c99977011  ./out3/longsmoke-F10-N3-iso-persisted.tsv
c6470eadbbbce1c19b48a58af069d6caa4a27875b06f8e2de7726d3c4643387c  ./out3/longsmoke-F10-N3-iso-recompute.tsv
ed2864fde96adddce1a3ce717c288098f1108f83459f9fbdac77091b8f3c79ad  ./out3/longsmoke-F10-off(today).tsv
efe3f36a2f59233ad9ec9b20f2db36647a4da3ad718c857ff9b863e291a43ece  ./out3/longsmoke-F9-N1A-any-isoAll-mem.tsv
879eb7252d0fbb94b8951814bd199f5e76263d2e4294082eaf57a58d6cdd17d4  ./out3/longsmoke-F9-N3-iso-persisted.tsv
f290863f4bb933ea51dca06f62aedd71e66a8663e542623a37df45936eb43a00  ./out3/longsmoke-F9-N3-iso-recompute.tsv
0186564edb26565b97f62be0932d47e5580ef4ac13438a2803277aec38bed037  ./out3/longsmoke-F9-off(today).tsv
84118a36f14ba581d7238a37155c6bc93ca35db56f6f3a088d5480ce3a346516  ./out3/p4-F10-N1A-any-isoAll-mem.tsv
8c717cdb740e57ebd71d26b22c1fb8383ad49d6a75b0eaa5bc0221e11a8ef138  ./out3/p4-F10-N2-rebuild-checks-mapping-refuses.tsv
994d2876a7e5ede00a1cd4745f8616158f6df45cde088300bd38c11039f915bc  ./out3/p4-F10-N2-same-verdict-rebuild-tolerates-data.tsv
c8648c0be395512ea5943b695a34b07fae4e35eb7fa9191394f9706553d73a56  ./out3/p4-F10-off(today).tsv
2efd252db855b16ea5a786793f75729556a48189054732382c1fd55d14406f37  ./out3/p4-F8-N1A-any-isoAll-mem.tsv
1cf5a065ca747a117a65240b86cedae8b419806aa8407948605889ecc69584d5  ./out3/p4-F8-N2-rebuild-checks-mapping-refuses.tsv
b4dc768cff2f9a7231cea35e9d83c0391d502462fdd58627ea0247fdd1eb3e07  ./out3/p4-F8-N2-same-verdict-rebuild-tolerates-data.tsv
a78ed5255ea0868d94d2cbd82a795ab268dd9ed8364e350cef353ae2ef5aceab  ./out3/p4-F8-off(today).tsv
e9065e0a99fddecbf61f92155516ba5839728a9b1c01a5988456c245a7eb2fa7  ./out3/p4-F9-N1A-any-isoAll-mem.tsv
4a733de7e63f727c672033ea900663f2c3491a0897b8d0aee880ef4095db9569  ./out3/p4-F9-N1B-any.tsv
94762065e340b4fd002408befb55f0e95adda6a1272bee4cad988f91395965d6  ./out3/p4-F9-N1C-any.tsv
ac8bbbb7674ece207c61b65d94295b378cbf6d3eb112ae4b520d69e16794504a  ./out3/p4-F9-N2-same-verdict-rebuild-tolerates-data.tsv
2968f90fdf7e844119e2c54de57a04cf82baa7e4389f77bbf46a32f8af4d3676  ./out3/smoke-F10-N1A-any-isoAll-mem.tsv
c9c796024e558b087ef8fd91daad9377dc37ab644ea0e69030d9ac1b0436bdf9  ./out3/smoke-F10-N3-iso-persisted.tsv
b9cfd0743f24ef056f6b5de4105fdad25735c27b3ad7b449357f225b11850ffe  ./out3/smoke-F10-N3-iso-recompute.tsv
d1926a2ccd947d3283e15cb70d0da4fb2bc50157490965e0102b229b11faf933  ./out3/smoke-F10-off(today).tsv
734077af8556da741ce9d8c06367abcee44e31d7d8eaf76de130dd49553d56fa  ./out3/smoke-F8-N1A-any-isoAll-mem.tsv
07cd61ad46b2eee535b13190aa757a00eb20eccc027de8552577cc36f10786f3  ./out3/smoke-F8-N3-iso-persisted.tsv
cef5942b8b038edaaf74945b8b0db9aeab5e68d174237dfdd4ee2968d1b95303  ./out3/smoke-F8-N3-iso-recompute.tsv
a3daa01628c78d117b982b142e5f48ddf88f5467092662184a1bcce42d359ecf  ./out3/smoke-F8-off(today).tsv
f41cae7c12c76bca91457bb419d06efd40da3af8d207ec712196c06acc805524  ./out3/smoke-F9-N1A-any-isoAll-mem.tsv
620b60d829808c435a49f36f147a65f65ce7ba6f5da31e9b5c4554399b2aba3c  ./out3/smoke-F9-N3-iso-persisted.tsv
22e7202be4aedc82697c3ce3e8dfacb581c7350f308602129ab425b232c9408f  ./out3/smoke-F9-N3-iso-recompute.tsv
3979de90d6559e5e1346477b4d694ef5cb22d6201d46e7ccb0311f27689d8c31  ./out3/smoke-F9-off(today).tsv
8e2ee4b7f799fbeb48e6d1a8fe52f6113e14f5a839f3ff84b2bdcae85a26ab9c  ./out3/tailFO-F10-N1A-any-isoAll-mem.tsv
c106548ab25e4f37143a8bfe50eb78bfdc498570e671a034b78c580f3abf7fca  ./out3/tailFO-F10-N3-iso-persisted.tsv
e5d5e7b836a506ae908fd36454ea44df56c1c36c7a2b37d6d3ec3dabd55b6ef0  ./out3/tailFO-F10-N3-iso-recompute.tsv
c2422beff630c9db44a0cc5ba4804d20f781f313e629a82d9968591686911e29  ./out3/tailFO-F10-N3-keep-allocated.tsv
43a660ab29625c73d35eda740292ff56fc768241ad4054d956b7251e3bf7e7d7  ./out3/tailFO-F10-off(today).tsv
0ea9745895936d0178e6b1e8036a22678b4f639d8431f94a4acbf0727f94d36c  ./out3/tailFO-F9-N1A-any-isoAll-mem.tsv
675de1ebaeccd19a3ecba34ce192165897e1694ec817227b8ba6d37d68940f3f  ./out3/tailFO-F9-N3-iso-persisted.tsv
e320e7765bceb2876024dd6cf070ddefbe528b748becabed105189deda720d68  ./out3/tailFO-F9-N3-keep-allocated.tsv
1bc3ef54f34c5be39d9b0f69a22d36dabf4a747da20a44aa6d2a77abc4392ca5  ./out3/tailFO-F9-off(today).tsv
057c2d09dd7c59df0d48b426a79dbda5ceeb8b38b5d60b7075355ca275f6e82e  ./out3/tailM-F10-N1A-any-isoAll-mem.tsv
369f24dc7ca29db48a1277fe663b23ab5f9b451e195f7e775c806af8ab35d9fd  ./out3/tailM-F10-N3-iso-persisted.tsv
5dbc76b9e42c597d79ab6e4d680bd2d3af62ff57dd3ba84bfebccdc6834133fd  ./out3/tailM-F10-N3-iso-recompute.tsv
3db460a017ae91310e07aca49bb744f437d486d9fb67b9f31b97519b436ded6e  ./out3/tailM-F10-N3-keep-allocated.tsv
84474a89f6aa0ad89a468de104436ef745f46c88a7b3baeb07d204fc1ba10efb  ./out3/tailM-F10-off(today).tsv
5616d8a090ba110baaeee790875c6d2d0ecc07632530f767c7338539b8cae3a1  ./out3/tailM-F9-N1A-any-isoAll-mem.tsv
8bdab5ad54906a44e150ce9b6c228d79ab7464aa061902765317b45968a2fbee  ./out3/tailM-F9-N3-iso-persisted.tsv
2972d3e24303b4c4134facaf4f572ab4df32de7ecde83866fb24277d06a0c5bf  ./out3/tailM-F9-N3-keep-allocated.tsv
0bca40b402a365c255c7397dc3775cb0cb4e6f4e98395b60a0fb77c64aa4e570  ./out3/tailM-F9-off(today).tsv
05c65add9fdcb0a7e3e03c488544d7b22b9b59722fbca64312ea9cd67ae0e596  ./out4/p4-F11-N1A-any-isoAll-mem.tsv
d6c3ac0aca9e47acf57c88df25ca5e85c0ae95125708bf26f7518fd05d030141  ./out4/p4-F11-N1D-reread-then-mismatch-keep-allocated.tsv
aaddaa682697aab54910303a6620cdc0d5fb8cff460717c847e3b525a6da19c8  ./out4/p4-F11-N3-keep-allocated.tsv
19bcdeb403a998ef46fb2b9fcd2ae711f227da2ae035b9577107ef2159418d0a  ./out4/p4-F11-off(today).tsv
38044d873b8b586fc810d9d68a3f0ae16c280380d9e406b5d8e902c337a77738  ./out4/p4-F1--N1D-reread-then-mismatch-iso-mem.tsv
d16deb9181b22a8c1a9f61f2537aa0054622a53b20a001cdef3108d422a7cb6e  ./out4/p4-F1--N1D-reread-then-mismatch-keep-allocated.tsv
a67b8f25d6633fef8e4de1ca89b6ec0861b027cf95bf9cb457ef9a9fde264df5  ./out4/p4-F2--N1D-reread-then-mismatch-iso-mem.tsv
6ccf02d8a2c52ab2ff084ae51bc5c2a1c3a3fa0b4242582051fd7ecc3e08c132  ./out4/p4-F2--N1D-reread-then-mismatch-keep-allocated.tsv
7dbc0e7182aedd1df1b503143c1cecec93be7d4a91a3e31154cf1c7fc3490082  ./out4/p4-F3--N1D-reread-then-mismatch-iso-mem.tsv
6efd3c51c7b6e723b6c0f0f4a0a9a4a3087bcf77b6e841a255e3884e56cefd60  ./out4/p4-F3--N1D-reread-then-mismatch-keep-allocated.tsv
0a089aeb4c248caccb5800e20a910c13b449e9cfa4c6a819ed53bcbd9e2c981b  ./out4/p4-F9--N1D-reread-then-mismatch-iso-mem.tsv
978c763a840ed8633b8b6a5d92b79c16e7e3ed08bef12255b7cbce45327879a7  ./out4/p4-F9--N1D-reread-then-mismatch-keep-allocated.tsv
6dc993c2d56c278fc5d4878119fdec73e800587c13279e355a55d62f0e9acd0f  ./out4/tailFO-F9--N1D-reread-then-mismatch-keep-allocated.tsv
780ee0dcde9b67914d80192363cfff492bbb05898cb87f12b8a01f36c8925ad4  ./out4/tailM-F9--N1D-reread-then-mismatch-keep-allocated.tsv
fef6c08cdc3d38e6924924ecec1b1e42eab43d5abe7d35fd36217e86c4e42b45  ./out/long-F3-N1A-every-isoAll-mem.tsv
9624e0066801fb5482b06003486164e9ff468174540b8509e60b29b95a421f68  ./out/long-F3-N1A-every-isoBad-mem.tsv
d75831eda948d735659783993d17377a9e6c2bc860de566a8530e7166bcf3db4  ./out/long-F4-N1A-any-isoAll-mem.tsv
0998e0478c9b3bdfe92bb1b716c96770d1d5129995f907c35f5b2559f0c2693a  ./out/long-F4-N3-iso-persisted.tsv
34d16274aaee3aa47bf710e324712a60590a87d22bf48e98a2a8536139d87735  ./out/long-F4-N3-iso-recompute.tsv
d775c89d30aaa5b5e60debd084203572986d9ac2c8e5c87b8fd575131725518f  ./out/long-F4-N3-keep-allocated.tsv
57e0dfba3b89dea954ba752d2c4faf9147b9839846ed61c0f2d4b6dbd8bab34a  ./out/long-F4-off(today).tsv
ffa41d8d86ccd42495f1949c154164b50aca93647a9fc8c898651d28d1b6e405  ./out/long-F5-N1A-every-isoAll-mem.tsv
518195bc3b38b2f36002e47e84b80823a51ebd4b5050efc9662ea4a334ba434c  ./out/long-F5-N1A-every-isoBad-mem.tsv
57c8d53123336340dc84b5c2e883f06aef3e50b3d838eb1a4e5a4c66f1fb93eb  ./out/long-F6-N1A-any-isoAll-mem.tsv
9e33f2a0e2bd42790737a4bfc0898cb2cbfa3577ddc76e78dadc2df3b4e23ed5  ./out/long-F6-N3-iso-persisted.tsv
d3f0dc2364098ab7dddd26c9291bfa8c4f7ecf86263deddaf08d3b589e3be2f5  ./out/long-F6-N3-iso-recompute.tsv
f1ad26d7cbd0a30447e0b32edb02272f8d3118c79f0daca5fb8324ef9e92fdf3  ./out/long-F6-N3-keep-allocated.tsv
373984dba3d5cf3ed4a0ca2f31f31f2ae5c3c4366941e000d3864cadc1311b18  ./out/long-F6-off(today).tsv
19970f7b6d7d5d690d26253cf50ef4ab04d7a08d78434d81a0becd5887c48620  ./out/mmm-N1A-any-isoAll-mem.tsv
2ffd9cdd898e9e37edfa007bc3bdc6ee94eda644752b893afb5699a0b3be3d12  ./out/mmm-off(today).tsv
e7344d5c4fa39effcbf6e045624244f971e4fc5c87487e24aabff61b1dda045e  ./out/n2-N1A-any-isoAll-mem.tsv
c78f0b9e421891d74b73c6a917691d1993e2be42de0db30668b99e4003c97105  ./out/n2-N2-same-verdict-rebuild-tolerates-data.tsv
e696d7e4e3f39fe0d1bb4544bc9b8a2fb876d2c4ff905b86ae15a5dc2ccc5cfd  ./out/ring-F4-N1A-any-isoAll-mem.tsv
241eb85b9fc1c9ff86e07c630ef76be98ce0c07d0c65a505e0879471361f2a4e  ./out/ring-F4-N3-iso-persisted.tsv
5f8ee2e546fbcee4705c8a28c4ea16003c509c292563ca981a174a9aca92eb74  ./out/ring-F4-N3-iso-recompute.tsv
8b6bee410a27fcb9f1b7bd124bab37ab4f362dd73e3f16a2359314f5ca5540fe  ./out/ring-F4-N3-keep-allocated.tsv
849b1ebd4337ac7463f5f38d1f812802bac3dc83a0bb141d458fb1ef24f774e0  ./out/ring-F4-off(today).tsv
66ca4f689b395c361bf73e2e168db49eb5874432ce303c13be71096adc573204  ./out/ring-F6-N1A-any-isoAll-mem.tsv
7d02b8daa41bf83ebc595891233540b25ffe579449b6858e915528019f3b8169  ./out/ring-F6-N3-iso-persisted.tsv
e1086073d49048cbf049283160de50369d98ccd50a15e12d6f8a5bb325e19d19  ./out/ring-F6-N3-iso-recompute.tsv
49f78e81a23b8c9906ca1f92e87b52830515c5fe1f63059a23159421c9434d97  ./out/ring-F6-N3-keep-allocated.tsv
88a7fb5180d289ea559f021bc5087c009517cc18e9ec379401bbdbf0ee50fce1  ./out/ring-F6-off(today).tsv
fedb903c862764c60e58e724d093a43e95129efa1c16a2b81f6316a7303e0d70  ./out/sweep-len4.tsv
60736f63959ebd26b26aaee59d1b5c9b33f7876f41a4d11dc4dcc5a07e0a7e17  ./patch3/apply3.py
760aed57f98db51bbd9a07f7befa20e5e7e5a110dbf8a6fb43efe9aad652ab1a  ./patch3/m2_newq_attack3.rs
248b203dce747d713b707bdedbc34d43333bd563342237bf01536ea1132ab6b4  ./patch3/make_driver3.py
287ca3b5c5fca9b408e0692ebead6950364bb8d425b730acfb069abc10689586  ./patch3/run3.sh
5687e55677395a3096bcbae16c31cf2d85c8350fde94548ba6b67921505661db  ./patch3/suffixes-P4-FO15.txt
b50ce2858aa2397fe008ab7c6e3eb751ae1bdaf359d88a6ff396f5c6fa301a0f  ./patch3/suffixes-P4-M-FO15.txt
3722aca6f248cf60aa92154bebc1666bed4c5d0676a63812cff20f210b229297  ./patch3/suffixes-P4.txt
150d826333570c0de34589d152394729f7343d618f9059b8739c93440be7f4aa  ./patch3/summarize3.py
39cb3d2e6ab8c3f423b6aaf96f76e978d78b0ccc502a692decd327a7b3a9e01d  ./patch4/apply4.py
48b69c7f6d813d14ae3e5d4264a4a343595029dde4c2b10da75e677be1f5dcb8  ./patch4/m2_newq_attack4.rs
775ee1ac9222ca58387fc6931b6640690a3db785c5166513de8c851b99dd545f  ./patch4/make_driver4.py
0f3be2b9734f57c354f4d4956cc5994a2579ff48c3b523dcdb8707dbd278e686  ./patch4/rerun-after-enospc.sh
4972f2677c0abfb5d3f789c3af5471d670eb13a5e344da0079293d32364b6c3e  ./patch4/run4.sh
0dc8278b7e0f226670c0b771804c174a108fa2266e1006ac0b45c3377b9011eb  ./patch/apply2.py
9a3ab29976350a7317cb93c92932e42485f96f08c66294fca25f7d8e27676a7d  ./patch/apply.py
61802c6b5261019e66cd9ef69363f7577be864ba923a9143d11051b24acd0b35  ./patch/m2_newq_attack.rs
065b14b5730f5ed5b6e58501807002e099d8b4dd2596ca90b4edecbaa3b2c6fd  ./patch/release_check_model.rs
837f931c01c98e1490c2a3fb0902106e9df3d3cd49067506729f72e7c86ca1a7  ./patch/run-long.sh
416e7d817bfb3bdf5d8b689f1b90b77c1706f555993019055e3d62757f81118c  ./patch/suffixes-len7-O.txt
3861590bc2a76ce5d814b1332bccd42cbccf77a6c67d07f7ff3df4e33b7fc1ed  ./patch/suffixes-ringwrap.txt
15ea249e19641a976c27c7a05448dfd6df577b200d1f83e8d09e9e7912ff978f  ./patch/summarize.py
```

## 二、各格判定一览

攻方立场：每个候选都假设它是错的，去造让它错的历史。「错」「不错」都是副本上真跑出来的；分辨臂的判定已按规则放开用户动作（前四步 O/M/F 全枚举，再接固定尾巴或再枚举六步）。

| 格 | 候选 | 造出的出错历史（副本、量过） | 同一段历史上不中的候选 | 结论（交主 agent 判） |
|---|---|---|---|---|
| N1 | 甲 当成对不上 | F1 瞬时读错：好槽被隔离、按对不上计数（58/120） | 乙、丙、丁 | 分得开，岔路 |
| N1 | 乙 发布失败交回 | F2 / F9 持续读错：文件永远改写不掉（58/120 条每次 O 都失败） | 甲、丁 | 分得开 |
| N1 | 丙 走 D23 失败表 | F2 / F9：每次挂载后第一次 O 就转只读（58/120） | 甲、丁 | 分得开 |
| N1 | 丁 重读一次再按甲（本腿提的，零轮） | 没造出（F1、F2、F3、F9 各 120 条） | — | 只在本腿模型上量过、被攻过零轮 |
| N1 | 两份镜像 every 读法 | F3 / F5：好镜像被一起隔离，或两盘隔离不对称（isoBad 49/120） | any | 分得开；不对称本身 729 条里没造出后果 |
| N2 | 各自判（今天） | F1 / F2 / F9 先重挂：一个数据单元读不出就整池挂不上（62/120；F2、F9 永远挂不上） | 读法甲 | 分得开 |
| N2 | 同判据·读法甲（重建容下读不出的数据单元） | 没造出（卡在第一版没有重写被照抄单元的路径） | — | 只罩数据单元；元数据单元没法同判 |
| N2 | 同判据·读法乙（重建核映射位置项、不一致拒挂） | F8 / F10 写者错落盘、先重挂：整池永远挂不上（F8 120/120、F10 40/120；今天 58、0） | 今天、读法甲 | 分得开；挂载那一刻分不出映射错还是提示过期 |
| N3 | 只在内存 | F10：重挂后活叶被覆写、整池挂不上（tailM 40/120）；F9：重挂后每次覆盖写都写到坏槽上失败（58/120） | 落盘、留在已分配（两段都不中）；现算（F10 不中、F9 中） | 分得开 |
| N3 | 重挂时现算 | F9 同上（58/120） | 落盘、留在已分配 | 分得开；它只罩「已释放却被所选根引用」 |
| N3 | 落盘 | 没造出（F9、F10 各 240 条 0 次）；代价是改格式 | — | 格式代价是推的 |
| N3 | 记录留在已分配（上一条腿提的，零轮） | 没造出挂载失败或坏写；代价：槽永久漏掉、checker I-3.1 报红 | — | 只在本腿模型上量过、被攻过零轮 |
| N3 准入 | 并进第九项 / 不进式子 | 并进：隔离了还没回收那一段扣两遍（每盘 2 槽，656/729 条）；不进：回收之后多报（每盘 2 槽，73/729 条） | 留在已分配两段都对 | 式子的输入差量过，式子本身的后果没跑（准入不在发布路径上） |
| 共用前提 | 硬规则 1 的「逻辑上照样释放」读成改写记录 | F6 / F8：下一次挂载撞 `ReleaseTargetAlreadyReleased`、永远挂不上（729/729）；F7：同一次发布释放两次、断言 panic（58/120，今天同样） | 只有「留在已分配」不中 | 不拿它判持久化三臂；F7 那处断言另立一笔账 |
| 共用前提 | 释放的结构判先于读盘核 | F11：映射指到没分配的槽，同一次挂载里每次 O 都失败（58/120，五臂同） | 无 | 另立一笔账 |

### 本腿提的改法各修哪一格（全部只在本腿副本上量过、被攻过零轮）

| 改法 | F1（假隔离） | F2 / F9 发布（乙、丙卡死） | F9 跨重挂（坏槽复用） | F10 跨重挂（活叶覆写） | F6 / F8（撞已释放） | F7（panic） | F11 | 准入读数 |
|---|---|---|---|---|---|---|---|---|
| 丁（重读一次再按甲） | 修（量过 0/120） | 修（量过 0 条发布失败） | 不修（单用丁 + 只在内存，推的：同甲） | 不碰 | 不碰 | 不碰 | 不修（量过 58/120） | 不碰 |
| 记录留在已分配 | 不修（单用会把好槽永久漏掉，推的） | 同甲 | 修（量过 tailM、tailFO 0 次坏写） | 修（量过 0/240） | 修（量过 0/729） | 修（量过 0/120，但多漏叶容器那 2 槽，推的） | 不修（量过 58/120） | 修（量过：隔离位 0、`free − 可发` 0） |
| 丁 + 记录留在已分配 | 修（量过 0/120） | 修（量过） | 修（量过 tailM、tailFO 0 次坏写） | 修（推的：F10 没有读失败，丁不起作用，同上一行） | 修（推的，同上一行） | 修（推的） | 不修（量过 58/120） | 修（推的） |

## 三、N1：释放前读盘核时读本身失败

### 候选（副本里的定义，`patch/release_check_model.rs` 的 `OnReadFailure`、`MirrorRule`、`IsolationScope`）

| 候选 | 定义 |
|---|---|
| 甲（N1A） | 读失败当成对不上：隔离、发布照成、计数 |
| 乙（N1B） | 发布失败交回（副本里的错误成员 `ReleaseCheckReadFailed`），状态一个不动 |
| 丙（N1C） | 交 D23 已定项 14 失败表：探针写（本装置写不失败）→ 对失败落点只读复核一次 → 读得出按切换（副本按同一挂载里重做这次发布模拟，N_switch = 3）、读不出转只读到下次挂载 |
| 丁（N1D，本腿提的，零轮） | 一份镜像读失败先对同一落点再读一次（只读），读得出按读出的字节判，仍读不出按甲处置 |
| 镜像合成的两读法 | any：任一份读得出且对得上即过（与 `recovery::read_unit_via_locations` 同一条）；every：每份都要读得出且对得上 |

### 历史与读数（全部是副本上的数，前缀 = 挂载写行 M、覆盖写 O；故障打在上一版数据单元那个槽上；用户后缀 O/M/F 长度 1–4 全枚举 120 条）

| 故障 | 甲 | 乙 | 丙 | 丁 |
|---|---|---|---|---|
| F1 读错一次（两份各一次） | **错**：一个好槽被隔离、按「对不上」计数（58/120 条的 `isolated_by_check>0`） | 不错：那次发布失败交回，下一次 O 照成 | 不错：切换一次后照成（58/120 `switched`） | 不错：重读读得出、对得上，0/120 隔离，发布照成 |
| F2 潜在扇区错（两份、直到被重写） | 不错：发布照成、坏槽隔离；以 O 开头的 40 条后缀里没有一步报错 | **错**：每一次 O 都读同一个坏槽、每次都失败，文件永远改写不掉（58/120 条全程 `ReleaseCheckReadFailed`） | **错**：第一次 O 就转只读，重挂之后第一次 O 再转只读 | 不错：同甲，58/120 隔离、发布照成、0 条发布失败 |
| F3 只有盘 0 那份读错 | any：过、不隔离；every：隔离（isoAll 两盘各 2 槽、isoBad 只隔盘 0，49/120 条两盘隔离数不等） | any：过；every：90/120 发布失败 | any：过；every：90/120 转只读 | 不错（any）：0/120 隔离 |
| F9 坏介质（读写都失败、永远） | 不错（发布照成）；跨重挂那一半归 N3 | **错**：同 F2，120/120 最终挂不上 | **错**：同 F2，58 条转只读、120/120 最终挂不上 | 不错（发布照成）；与「记录留在已分配」合用时 tailM、tailFO 各 120 条 0 次写到坏槽 |

引产物（`out/sweep-len4.tsv`、`out3/p4-F9-*.tsv`，整行太长，按列截取；截取命令见第一节）：

```
F2-latent-eio-both	N1B-any	OOOO	O:err:ReleaseCheckReadFailed { unit: Data, slot: SlotNumber(50182) } O:err:ReleaseCheckReadFailed { unit: Data, slot: SlotNumber(50182) } O:err:ReleaseCheckReadFailed { unit: Data, slot: SlotNumber(50182) } O:err:ReleaseCheckReadFailed { unit: Data, slot: SlotNumber(50182) }	mount-err:Recovery(UnitUnreadable { slot: SlotNumber(50182) })
F2-latent-eio-both	N1C-any	OOOO	O:failure-table:read-only-until-next-mount O:refused(not-writable) O:refused(not-writable) O:refused(not-writable)	mount-err:Recovery(UnitUnreadable { slot: SlotNumber(50182) })
F2-latent-eio-both	N1A-any-isoAll-mem	OOOO	O:ok O:ok O:ok O:ok	ok
F1-transient-eio-both	N1A-any-isoAll-mem	O	O:ok	ok	same	-	[2, 2]	4	0	[0, 0]	7	163840	0	2	t1@50182:released-and-isolated
F3-latent-eio-dev0	N1A-every-isoBad-mem	O	O:ok	[2, 0]	2	12	262144	t1@50182:released-and-isolated
```

（最后两行里的 `[2, 2]` / `[2, 0]` 是两块盘各自的隔离槽数。F2、F9 两行「最终挂不上」是 N2 那一格的今天的重建判据造成的，见第四节；乙、丙在那两段历史上的错不靠它：挂得上时它们照样每次 O 都失败或转只读。）

### 打中之后的四句

| 打中 | 分辨臂 | 被判的系统当时看不看得到判别子 | 满足的是哪一句 | 各候选在这几格上还中不中 |
|---|---|---|---|---|
| 甲 × F1（好槽被当成对不上） | 分辨（乙、丙、丁不中） | 看得到：再读一次就分得开，丙的只读复核、丁的重读就是这一步；甲自己不读第二次 | 硬规则 1（第 99 行）的隔离分句只说对不上；F1 不是对不上，甲把读失败并进这一分句，计数报出的对不上是假的 | 乙、丙、丁不中 |
| 乙 × F2 / F9（文件永远改写不掉） | 分辨（甲、丁不中） | 看得到：设备报的就是读错 | 硬规则 1（第 99 行）的发布照成分句只说对不上时；读失败时照不照成正是这一格要定的 | 甲、丁不中；丙同样中（转只读） |
| 丙 × F2 / F9（每次挂载后第一次 O 就只读） | 分辨 | 看得到：复核读不出 | D23 第 371 行失败表的只读复核读不出那一支（整行见第七节）——那张表是给发布失败定的，丙把它搬到释放核验的读失败上，而这里的坏槽是要被丢掉的旧版本，不是这次要写的落点 | 甲、丁不中；乙同样中 |
| every × F3（好镜像被隔离、或两盘隔离不对称） | 分辨 any / every | 看得到：另一份读得出且对得上 | 没有条款说两份怎么合成；any 与 `recovery::read_unit_via_locations` 同一条 | any 不中 |

### 代价（量过，`check_reads`、`check_read_bytes` 两列）

| 候选 | 多读 | 多写 | 改不改盘上格式 |
|---|---|---|---|
| 甲 / 乙 / 丙，any | 每次覆盖写发布 6 次、131072 字节（换下的 6 个进映射单元各读一份；只在一份坏时多读另一份）：`OO` 与 `O` 两行之差 13 − 7 = 6 次、294912 − 163840 = 131072 字节 | 0 | 不改 |
| 同上，every | 每次 12 次、262144 字节（两份都读） | 0 | 不改 |
| 丙另加 | 失败时一次探针写 + 一次只读复核 | 探针写一次（落在固定落点） | 不改 |
| 丁另加 | 只在读失败时每份多读一次（量过：F1 那一行 7 次读里 1 次是重读） | 0 | 不改 |

## 四、N2：`rebuild_version` 与释放判定用不用同一条判据核位置项

### 今天的两条判据（主工作区 2026-09-24 04:30 UTC 现查；副本基线早于这一刻，见第八节）

- 重建读数据单元走父指针里的位置项（提示），任一单元读不出整次挂载失败：`crates/singlefs-core/src/recovery.rs` 第 1038 行 `        data_unit_bytes_in_file_order.push(read_unit_via_locations(`（副本基线里是单个数据单元的同一种读法）
- 重建读映射条目只取 key、位置项丢掉：同一文件第 1127 行 `        let (key, _locations) =`
- 释放经映射条目的位置项找槽：`crates/singlefs-core/src/transaction.rs` 第 1493 行 `pub fn placements_to_release_via_mapping(`

### 候选

| 候选 | 定义 |
|---|---|
| 各自判（今天） | 如上；副本里释放那一边装甲臂的读盘核 |
| 同判据·读法甲（上一条腿做的臂 `N2-same-verdict-rebuild-tolerates-data`，`patch/apply2.py`） | 重建时数据单元读不出，按释放核验的处置走：不拒挂载，内存里拿全零占位，交释放核验去隔离。只罩数据单元：下一版要读字节的元数据单元（extent 根、叶容器）没法照这条走 |
| 同判据·读法乙（本腿做的臂 `N2-rebuild-checks-mapping-refuses`，`patch3/apply3.py`） | 重建时把每条映射条目的位置项与父指针里的位置项逐条比（不另读盘），不一致就拒绝挂载（副本借用 `UnitUnreadable` 这个错误成员报，槽号是映射条目指的那个） |

### 历史与读数（副本）

| 历史 | 各自判（今天） | 读法甲 | 读法乙 |
|---|---|---|---|
| F1 读错一次 × 后缀以 M 开头（先重挂再写） | **错**：62/120 条那次挂载失败（瞬时读错也让整池挂不上；重挂一次又好了，最终 4 条挂不上） | 不错：0/120 | 同今天（不碰读失败） |
| F2 潜在扇区错 × 先重挂 | **错**：62/120 条挂不上、永远挂不上（坏的是现行文件的数据单元，唯一的出路是覆盖写它，而挂不上就写不了） | 不错：0/120，之后覆盖写照成、坏槽隔离 | 同今天 |
| F9 坏介质 × 先重挂 | **错**：62/120 | 不错：0/120 挂载失败 | 同今天 |
| F8 写者错落盘（数据单元那条映射条目指到现行实例表）× 先重挂 | 不错：挂载照成，挂载那次写行发布按父指针重装映射节点、错条目被换掉（`M:ok O:ok`） | 同今天 | **错**：挂载被拒、永远被拒（盘上那一版的映射一直是错的，没有发布能换掉它）：最终挂不上 120/120（今天 58/120） |
| F10 写者错落盘（指到照抄着的右边那片叶）× 先重挂 | 不错：同上，0/120 最终挂不上 | 同今天 | **错**：40/120（今天 0/120） |

引产物（`out/n2-*.tsv` 的汇总，`patch/summarize.py` 跑出；`out3/p4-F8-*.tsv` 后缀 `MO` 那几行按列截取）：

```
F2-latent-eio-both              N1A-any-isoAll-mem                      120   62             62               0
F2-latent-eio-both              N2-same-verdict-rebuild-tolerates-data  120   0              0                0
N2-rebuild-checks-mapping-refuses	M:mount-err:Recovery(UnitUnreadable { slot: SlotNumber(50304) }) O:refused(not-writable)	mount-err:Recovery(UnitUnreadable { slot: SlotNumber(50304) })
off(today)	M:ok O:ok	ok
```

（前两行四个数依次是跑数、步骤里出过挂载失败的条数、最终挂载失败的条数、读回与最后一次写不同的条数。）

**读法甲造不出出错的历史，卡在这里**：全零占位只住内存。第一版里写数据单元字节的只有覆盖写，它写的是新内容；挂载写行、抬 F 的发布都只照抄指针，不重写被照抄单元的字节。F1、F2、F4、F9 四种故障 × 120 条后缀里，凡有 O 的读回都与最后一次写相同；F4 的 30 条「读回不同」全是后缀里没有一次 O 的（内容本来就烂了，今天同样读不回）。F9 × 读法甲那 58 条「别的单元落在故障槽上」是驱动的误报：检测按原主人的字节 CRC 认主人，重建换成全零占位之后 CRC 变了；那 58 条的 `writes_refused` 全是 0（没有一次写落到坏槽上）。要让读法甲出错，得有一条「从内存里的字节重写被照抄单元」的路径（例如以后的搬迁、整理），第一版没有。

### 打中之后的四句

| 打中 | 分辨臂 | 当时看不看得到判别子 | 满足的是哪一句 | 各候选在这几格上还中不中 |
|---|---|---|---|---|
| 今天 × 先重挂（F1/F2/F9） | 分辨（读法甲不中） | 看得到：读失败是设备报的 | 没有条款要求重建在数据单元读不出时拒绝挂载；D19 第 169 行（自举豁免那一段）只写提示读不出时经映射回退，第一版提示与映射指同一个槽，回退读的是同一个坏槽 | 读法乙不改读失败，照样中 |
| 读法乙 × 先重挂（F8/F10） | 分辨（今天、读法甲不中） | **看不到哪一边错**：挂载那一刻只知道映射条目与父指针不一致，分不出是映射写错（F8/F10）还是提示过期（以后有搬迁时的正常情形）；拒绝挂载把「映射写错」从「下一次发布自己修好」变成「整池挂不上」 | D19 已定项 5（第 87 行标题）把中央映射定为唯一入口：读法乙按它把映射当权威，结果比今天坏 | 今天、读法甲不中 |

### 代价

| 候选 | 多读 | 多写 | 改不改盘上格式 |
|---|---|---|---|
| 读法甲 | 0（量过：`check_reads` 与各自判同） | 0 | 不改 |
| 读法乙（只比父指针） | 0 次读（比的是已在内存里的两份位置项），每次挂载多一趟映射条目 × 父指针的比较 | 0 | 不改 |
| 读法乙（改成读盘核映射条目指的槽） | 推的：每次挂载每个进映射的单元多读 1–2 份（第一版每版 6 个，any 读法 6 次） | 0 | 不改 |

## 五、N3：核出对不上而隔离的槽跨不跨重挂、进不进准入式子

### 候选

| 候选 | 副本里的臂 | 定义 |
|---|---|---|
| 只在内存 | `N1A-any-isoAll-mem` | 隔离位置在分配器里（`isolate_abandoned` 同一张位图），重挂即丢 |
| 落盘 | `N3-iso-persisted` | 驱动把核出来隔离的槽记成一个「盘上集合」，每次挂载之后原样施回（**只模拟语义，没真写盘**，格式代价是推的） |
| 重挂时现算 | `N3-iso-recompute`（`patch/apply.py` 第 4 段，改 `mount.rs`） | 挂载时把「分配记录已释放、却被所选根那一版的单元引用着」的槽隔离 |
| 记录留在已分配（上一条腿提的臂，零轮） | `N3-keep-allocated` | 核出对不上时映射条目照样随新一版去掉，分配记录不改写成已释放；不置隔离位。「物理槽不还回空闲池」由盘上那条已分配的记录担保，跨重挂不用任何新状态 |
| 对照 | `off(today)` | 不核 |

### 历史与读数（副本；用户动作放开：前四步 O/M/F 全枚举 120 条，再接固定尾巴）

尾巴 tailM = `M` + `(FO)×15`（强制一次重挂，再抬 F、写）；tailFO = `(FO)×15`（不强制重挂）。

| 历史 | 今天 | 只在内存 | 重挂现算 | 落盘 | 记录留在已分配 |
|---|---|---|---|---|---|
| F10 写者错：数据单元那条映射条目指到照抄着的右边那片叶（池里先建 240 个 inode 分出两片叶），tailM | **40/120 活叶被覆写、整池挂不上** | **40/120** | 0/120 | 0/120 | 0/120（I-3.1 红 40 条，见下） |
| 同上，tailFO | **25/120** | **25/120** | 0/120 | 0/120 | 0/120 |
| F9 坏介质（读写都失败、永远），tailM：之后每次覆盖写都写到坏槽上失败 | **58/120** | **58/120** | **58/120** | 0/120 | 0/120 |
| 同上，tailFO | **62/120** | **32/120** | **32/120** | 0/120 | 0/120 |
| F4 两份都烂（读得出、校验和不对），首步 O 其后六步全枚举 729 条 | 0/729 挂不上 | 0/729 | 0/729 | 0/729 | 0/729 |

F10 只有 40 条中招，是因为另外 80 条的首个非空操作是 M 或抬 F：那一次发布按父指针重装映射节点，错条目在被拿去释放之前就被换掉了（第四节 F8 那一行同一件事）。F9 的 tailM 另外 62 条是「先重挂」的挂载失败（第四节今天的重建判据），五个臂一样，不计在上表。

引产物（`out3/tailM-F10-*.tsv`、`out3/tailM-F9-*.tsv`，同一条后缀、按列截取：臂、后缀、最终挂载、读回、checker 违例、两盘隔离数、核验隔离数、`free − 可发`、核验事件、复用步号、重建比出的不一致、被拒的写次数）：

```
F10-writer-bug-disk-carried-leaf	N1A-any-isoAll-mem	OMFMFOFOFOFOFOFOFOFOFOFOFOFOFOFOFO	mount-err:Recovery(UnitUnreadable { slot: SlotNumber(50326) })	fail:Failed { root: Some((InstanceGeneration(4), CheckpointTxg(62))), failu	I-2.1,I-3.9,I-4.8,I-7.2,I-7.4	[0, 0]	4	0	[0, 0]	t1@50326:released-and-isolated	[21]	0	0
F10-writer-bug-disk-carried-leaf	N3-iso-persisted	OMFMFOFOFOFOFOFOFOFOFOFOFOFOFOFOFO	ok	same	I-3.9	[2, 2]	4	0	[2, 2]	t1@50326:released-and-isolated	[]	0	0
F10-writer-bug-disk-carried-leaf	N3-keep-allocated	OMFMFOFOFOFOFOFOFOFOFOFOFOFOFOFOFO	ok	same	I-3.1	[0, 0]	0	0	[0, 0]	t1@50326:record-kept-allocated	[]	0	0
F9-bad-media-rw-forever	N1A-any-isoAll-mem	OFOMFOFOFOFOFOFOFOFOFOFOFOFOFOFOFO	ok	same	-	[0, 0]	4	0	[0, 0]	t1@50182:released-and-isolated	[]	0	12
F9-bad-media-rw-forever	N3-iso-persisted	OFOMFOFOFOFOFOFOFOFOFOFOFOFOFOFOFO	ok	same	-	[2, 2]	4	0	[2, 2]	t1@50182:released-and-isolated	[]	0	0
```

F9 那一条在只在内存的臂上，重挂、抬 F 之后从第 12 步起每次 O 都是 `O:err:BlockDevice(InputOutput(Custom { kind: Other, error: "injected write e`（第 1 行末列：被拒的写 12 次）。

### 不分辨这三个持久化臂的两处（共用前提，不拿它们判 N3）

| 历史 | 今天 | 只在内存 | 重挂现算 | 落盘 | 记录留在已分配 |
|---|---|---|---|---|---|
| F6 映射条目（内存里）指到现行实例表，首步 O 其后六步全枚举 729 条：下一次挂载写行时释放旧实例表撞 `ReleaseTargetAlreadyReleased`，整池永远挂不上 | 729/729 | 729/729 | 729/729 | 729/729 | 0/729 |
| F7 映射条目指到同一次发布也要换下的叶容器，前四步 120 条：同一次发布里同一个落点释放两次，分配器断言 panic | 58/120 panic | 58/120 | 58/120 | 58/120 | 0/120 |
| F11 写者错落盘：数据单元那条映射条目指到一个没分配的槽，前四步 120 条：`placements_to_release_via_mapping` 报 `ReleaseTargetNotAllocated`，同一次挂载里每次 O 都失败，重挂（写行发布重装映射）才好 | 58/120 | 58/120 | 没跑（推的：同） | 没跑（推的：同） | 58/120 |

F6、F7 两处的病根都在硬规则 1（第 99 行）逻辑上照样释放那一分句被读成：把映射条目指的那个槽的分配记录改写成已释放：映射指错时，被改写的是别的单元的记录，隔离位跨不跨重挂都救不回来。F7 另有今天代码的一处缺口：`placements_to_release_via_mapping` 逐盘核「在册、未释放、跨度对得上」，不核同一次发布里两个角色查出同一个落点，于是走到 `crates/singlefs-core/src/allocator.rs` 第 858 行 `                "同一个落点释放了两次：这块盘的记录是不是已释放由 transaction::placements_to_release_via_mapping 逐盘判过"` 那条断言——那句话里「逐盘判过」对同一次发布内的重复不成立。这一处今天就在（`off(today)` 同样 58/120），与 N1–N3 选哪个候选无关，另立一笔账。

F11 那一行五个臂一样：映射指到的槽没有分配记录，结构判在读盘核之前就把这次发布拒了，硬规则 1 的发布照成在这里够不着——读盘核核出对不上（没分配的槽读出来是全零）也没用。它同样是共用前提，不拿它判 N3。

### 进不进准入式子

驱动每条跑完记两个数：`isolated_per_device`（两盘各自隔离位的槽数）与 `free_minus_allocatable`（分配器的空闲计数减去 `is_free` 为真的槽数）。`DeviceFreeMap::isolate` 不动空闲计数（`crates/singlefs-core/src/allocator.rs` 第 320 行 `    /// 隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数，只让分配器绕开它（用户数据落点与开放段都不落在它上面）。`），所以后者正是「已隔离且已回收」的槽数，前者减后者是「已隔离但还在已分配 / defer 里」的槽数。F4 首步 O 其后六步全枚举 729 条：

| 臂 | 隔离 [2,2]、已回收 [2,2] | 隔离 [2,2]、未回收 [0,0] | 隔离 [0,0] |
|---|---|---|---|
| 落点 | 73 | 656 | 0 |
| 只在内存 / 重挂现算 | 22 | 42 | 665（重挂之后隔离丢了） |
| 记录留在已分配 | 0 | 0 | 729（不置隔离位，槽一直算在已分配里） |

据此三个口径各错在哪一段（数是量过的，式子后果是推的——准入式子今天不在发布路径上，见下）：

| 口径 | 隔离了、还没回收（656 条那一段） | 隔离了、已回收（73 条那一段） |
|---|---|---|
| 并进第九项（共用隔离位，主工作区 `crates/singlefs-core/src/admission.rs` 第 316 行 `                abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),` 读的就是这张位图的计数：核验隔离若也走 `isolate`，就被它自动算进去） | **少报**：这几块同时在「已分配」「defer 待释放」里又在第九项里，扣了两遍（每盘 2 槽） | 对 |
| 不进式子（另开一张位图、不计入任何一项） | 对 | **多报**：空闲计数把它们算回空闲、分配器却发不出去（每盘 2 槽），与 C318 同形 |
| 记录留在已分配 | 对 | 对（它从不回收） |

另一处冲突：D28 第 27 行第九项的取值句同时要求它等于只被被抛弃根引用的槽数、又等于分配器实际隔离的集合（整行见第七节）；核验隔离走同一张位图时，后一个等式成立、前一个不成立。

**卡在哪**：准入读数与合取已在主工作区（`admission.rs`，本腿副本拍在它之前，没有这份文件），它的文件头自己写着发布路径与可写挂载都还不调它；本腿因此没法造一条「假性 ENOSPC / 假放行」的准入历史，只量到了式子的输入差多少。

### 打中之后的四句

| 打中 | 分辨臂 | 当时看不看得到判别子 | 满足的是哪一句 | 各候选在这几格上还中不中 |
|---|---|---|---|---|
| 只在内存 × F10 | 分辨（现算、落盘、留在已分配不中） | 挂载时看得到：所选根那一版引用着一条已释放的记录（现算臂用的就是它） | 硬规则 1（第 99 行）物理槽不还回空闲池那一分句：重挂之后它回去了，活叶被覆写 | 今天同样中；现算、落盘、留在已分配不中 |
| 只在内存、重挂现算 × F9 | 分辨（落盘、留在已分配不中） | **挂载时看不到**：释放发布之后，盘上没有任何东西记着那个槽核出过对不上（记录是「已释放」，映射条目已去掉），现算臂无从算起 | 同上一分句 | 只有带盘上痕迹的两个候选（落盘、留在已分配）不中 |
| 并进第九项 / 不进式子 × 准入读数 | 分辨口径 | 看得到（隔离位与回收位都在内存里） | D28 第九项取值句（整行见第七节） | 记录留在已分配两段都对 |

### 代价

| 候选 | 多写几个字节 | 多读几次 | 改不改盘上格式 |
|---|---|---|---|
| 只在内存 | 0 | 0 | 不改 |
| 落盘（推的） | 借 D3 已定项 7 跨度段里空着的一位当「隔离」标志：每条记录 0 字节，隔离与释放同一次发布改写同一条记录、0 次多写；另立一张隔离表则每个槽一条（key 10 字节起） | 0 | **改**：记录多一个态，而 D23 第 373 行写着分配记录条目只有两个态、盘上没法表达第三个（整行见第七节）；落 D15 第 2 层 |
| 重挂时现算 | 0 | 0（记录与所选根那一版的单元挂载时都已在内存里；多一趟记录 × 单元的比较，推的） | 不改；只罩「已释放却被所选根引用」这一种，F9 量过不罩 |
| 记录留在已分配 | 0 | 0 | 不改；代价是那个槽永远不回来（F10 的旧数据槽、F9 的坏槽，每次 2 槽 × 2 盘），checker 的 I-3.1 把它当账不平报红（F10 tailM 40/40、F9 tailFO 62/62 条），要么 checker 认一类「核验留下的」，要么留着红；F7 那一格它把叶容器自己那一次合法释放也一起滤掉（同一个落点），多漏 2 槽（按 `patch/apply.py` 第 2 段的过滤推的） |

## 六、没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 读法甲（重建容下读不出的数据单元）写出坏字节 | F1、F2、F4、F9 × 前四步 120 条 | 0 条：第一版没有从内存字节重写被照抄单元的路径（第四节「卡在这里」） |
| 只在内存的隔离在 F4（两份都烂）上出错 | 首步 O 其后六步全枚举 729 条 + 三条长后缀（30 步纯 O、第 15 步抬 F、`OOOOOOFMOOOOFO`） | 0 条挂不上、0 条读回不同：烂槽被复用只是被覆盖，没有东西丢 |
| 只在内存的臂在同一次挂载里（不重挂）被复用 | F10 tailFO 里后缀前四步不含 M 的那些条 | 今天与只在内存同为 25/120，差别全在含 M 的条里；同一次挂载里分配器往前走，没回头发过那个槽 |
| any 读法下 F5（只有盘 0 那份烂） | 前四步 120 条 × 甲乙丙 | 0 条出错：读到盘 1 那份就过，槽照常释放、盘 0 那份被覆盖 |
| 两盘隔离不对称（isoBad）让后续分配出错 | F3、F5 × every × isoBad，首步 O 其后六步 729 条（`out/long-F3-*`、`out/long-F5-*`） | 各 64/729 条两盘隔离数不等，0 条挂载失败、0 条发布失败、0 条读回不同 |
| 准入式子上的假性 ENOSPC / 假放行 | 没跑 | 卡在准入不在发布路径上（第五节） |

## 七、引文（整行，行号现查）

`.claude/kb/decisions/19-块指针的结构与宽度预算.md` 第 99 行：

```
1. **释放一律经映射，不经提示**；经映射核到那条映射条目之后，释放之前还要按它位置项里带的单元校验和读盘核一次。**核出对不上时隔离那个槽，发布照成**：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。
```

`.claude/kb/decisions/23-journal的角色与格式.md` 第 371 行：

```
**失败的处置按失败的性质分两支**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 失败表）。**判别子是当下判得出的量，不是次数**：失败发生时先对**目标设备的固定落点做一次探针写**——写不进去 ⇒ 持续失败，走**转只读到下次挂载**；写得进去，再**对这次失败的那个落点做一次只读复核**：读得出 ⇒ 瞬时失败，走**实例切换**；读不出 ⇒ 落点级的持续坏，同样走**转只读到下次挂载**。⚠️ 复核这一步**只读不写**：写探针进失败落点会抹掉那里的权威数据（8 个失败点里 4 格的落点是根槽或系统配置槽），而固定落点这一步不许改——`D2（RAID 条带策略）` **已定项 13** 逐字「与 `D23（journal 的角色与格式）` 已定项 14 的两支判别子同一个动作」靠它成立（主 agent 现查：那句话在 `.claude/kb/decisions/02-RAID条带策略.md:225`，归属已定项 13「降级期间只读」，不是已定项 15）。**失败点是屏障时没有落点可复核**，那几格只有固定落点探针写这一步。可写设备数掉到 w 的下限以下、切换自己的预留拿不到，同样直接走只读支；连续切换次数越过 **N_switch = 3**（具名可调参数）也转只读，它是兜底的兜底，真正的判别子是探针写加落点只读复核。**收敛靠的是切换的第一步就是一次会失败的写**：切换要先取号，取号要写独占打开成功的每一份系统配置、全或无 ⇒「写不出去」这类故障最多让切换走一步（探针写先拦，侥幸过了全或无也立刻判失败）。
```

`.claude/kb/decisions/23-journal的角色与格式.md` 第 373 行：

```
- 切换要用的块在挂载准入时预留：**它是内存里的一个空间量**，每次挂载重算、崩溃即丢、**不落盘**——分配记录条目只有「已分配」与「已释放 + 释放代」两个态（D3（空间分配） 已定项 7），盘上没法表达第三个态，而落盘表达它本身就要写单元、回到「要写才能写」。预留量按 **(N_switch + 1) × 一次切换的最坏量**（一次挂载允许 N_switch 次切换，且实例表链每切一次长一行、写行要 COW 重写整条链，第 k 次比第 1 次贵；多的一份给这次挂载写行那次发布的元数据）；**这个量的口径与式子的权威正文在 D28（挂载期承诺量） 已定项 3**：一次切换的最坏量 = 实例表链重写（副本数 2 × 32768 × 片数，按设备算）+ 暖机空发布（至多 R 次 × 现算 c_max），**固定点重做与号 > W 的数据单元重写不另要空间**——切换是挂载内一次恢复，重建态里失败那次的分配不存在，重做走 checkpoint 的保留池；引这一段时引 D28（挂载期承诺量） 已定项 3。
```

`.claude/kb/decisions/28-挂载期承诺量.md` 第 27 行：

```
- **被抛弃根独占量**（第九项）：取值 = 只被被抛弃根引用的槽数 = 被抛弃根引用的槽 − 回退候选集里的根引用的槽（下界 0），与分配器实际隔离的集合是同一个数；各条根引用的槽从它的分配记录树读、不加盘上字段；每次挂载与每次抬 F 时算出、只住内存、按设备算，被抛弃的根被轮转覆写时清零；由回退路径维护。
```

`.claude/kb/decisions/03-空间分配.md` 第 123 行：

```
| 跨度段（含已释放标志位） | **2（初值）** | value（已定项 11） | 一条记录记一个单元，跨度段说它覆盖几个槽。宽度按 D19（块指针的结构与宽度预算） 已定项 3 的「定宽留位」政策取，**理由是政策不是用量**——第一版恒为 1 或 2；**最高位借作已释放标志**（0 = 仍分配、1 = 已释放），不占表达跨度值的位 |
```

## 八、这条腿自己的限度

- 全部数都在副本上量：基线是主工作区 2026-09-24 01:22 UTC 的 `crates/`（未提交态，文件与 sha256 在模型目录 `baseline-crates.tgz`、`baseline-crates.sha256`）。之后主工作区又变了（新增 `admission.rs`，`recovery.rs`、`mount.rs` 等有改动），本腿没在新代码上重跑。副本上的数按规则不进 kb，要引须在入库装置上重做。
- 「核 + 隔离」是本腿（及上一条腿）自己加的最小实现（`patch/release_check_model.rs` + `patch/apply.py`），不是那个实现员的正式版；隔离走的是影子账同一张位图（`isolate_abandoned`）。正式版若另开位图，第五节「并进第九项」那一行的数就不适用。
- 「落盘」臂只在驱动里模拟（挂载之后把记下的集合施回），没真写盘、没真改格式；它的格式代价是推的。丙的实例切换按「同一挂载里重做这次发布」模拟，第一版没有实现切换。
- F6、F7 的写者错只改内存里上一版的映射节点字节（重挂即消失）；F8、F10、F11 是在发布装映射节点时注入、照常封好落盘。两种都是「写者自己算错映射条目」这一类，不是介质故障。
- F10 与 F9 的「中招条数」依赖尾巴长度（15 组 FO）：F10 首次复用落在后缀第 15–31 步（从 1 数，`reuse_steps` 列从 0 数），F9 的第一次坏写都在重挂、抬 F 之后；尾巴更短会少、更长可能更多。这些是在固定尾巴上数到的条数，不是上界；分辨臂的结论（哪一臂 0 条）不依赖它。
- 池是单文件 + （F10）240 个 inode 两片叶；多单元写、树分裂到多层、多盘（> 2）、真设备都没碰。debug 构建、tmpfs 上的内存盘。
- 读错注入在块设备包装层（`Flaky`），挂载后的读回检查走内存镜像、绕过注入：F1–F3、F9 的「读回相同」只说明盘上字节没被写坏，不说明挂载态读得出。
- 旁见，不在射程：在今天代码的副本上，`O` 之后连续 7 次挂载，checker 报 I-3.1（`out/mmm-off(today).tsv` 第 4 行 `OMMMMMM`；六次挂载那条不红），与臂无关（关着核验一样红）。主工作区之后改过，没在新代码上核。

## 九、没做什么

- 没判正推、辩方那几格（N4–N7 不碰）；没替主 agent 采纳任何候选。
- 没跑准入式子本身的假性 ENOSPC / 假放行（准入不在发布路径上，副本早于 `admission.rs`）。
- 落盘臂没真写盘；「并进第九项」「另开位图」两个口径只量了输入差。
- 没跑多轮：这是第一轮，攻过零轮的丁、「记录留在已分配」两个改法都没被攻过。
- 本腿没有登记给自己的门禁阶段要跑（没查 `stage-owners.tsv` 以外的阶段；写的只有报告、模型目录与草稿目录）。
- 过程事故：03:27 UTC 前后 /dev/shm 被三路并发扫描写满（内存盘镜像没清掉），当时在跑的几组（F10 × 重挂现算两组、N1 丁与 F11 全部、F3/F5 every 三组）结果被 `No space left on device` 污染；已删掉死进程留下的镜像、单路重跑（`patch4/rerun-after-enospc.sh`），报告只用重跑的结果，污染的文件没拷进模型目录（草稿目录里还在：`/tmp/claude-1000/m2-newq-opus/out3/tail*-F10-N3-iso-recompute.tsv`、`out4/`、`out/long-F3-*`、`out/long-F5-N1A-every-isoBad-mem.tsv`）。
- 跑时 `ps` 看到别的会话有 3–4 个 `cargo test`，没有性能测量进程；本腿全部加 `nice -n 19`、`CARGO_BUILD_JOBS=4`。
- 原始产物（tsv）放在模型目录 `out/`、`out3/`、`out4/`，没进 `research/results/`（写范围只给了报告、模型目录与草稿目录），要不要挪由主 agent 定。

## 十、主工作区在本腿期间落了正式的「核 + 隔离」（只读对照，没在它上面跑）

2026-09-24 04:30 UTC 现查，主工作区已有正式实现（本腿副本基线早于它）。按代码读，它落在本腿的哪几臂上：

| 维度 | 正式实现（读出来的） | 对应本腿的臂 | 本腿在那一臂上量到的 |
|---|---|---|---|
| 读失败 | 交回一个点名「条款没定」的错误成员：`crates/singlefs-core/src/transaction.rs` 第 1849 行 `    ReleaseChecksumReadFailedWhoseHandlingIsUndecided {` | 乙 | F2、F9 上文件永远改写不掉（第三节） |
| 隔离哪几份 | 只隔离核出对不上的那一份所在的盘：`crates/singlefs-core/src/allocator.rs` 第 1123 行 `            .expect("核出对不上的那一份是从这块盘上读出来的：读得到就说明池里有这块盘，而分配器按池里的盘建");` | every-isoBad 的形状（按份） | 729 条里两盘不对称 64 条、0 条出错（第六节） |
| 隔离位 | 另开一张位图：同一文件第 347 行 `    pub fn quarantine_after_release_checksum_mismatch(&mut self, slot: SlotNumber, span: u64) {` | 「不进式子」那一口径 | 回收之后空闲计数多报（第五节准入表） |
| 跨不跨重挂 | `mount.rs`、`admission.rs` 里 `quarantin` 命中各 0 次（`grep -c quarantin`）：只在内存、不进准入 | 只在内存 | F9、F10 重挂之后出错（第五节） |

以上是读代码的对应，**没在正式实现上重跑**；它的空闲计数在回收时怎么动、F10 那条历史在它上面是不是同样中，都要在入库装置上重做才能说。
