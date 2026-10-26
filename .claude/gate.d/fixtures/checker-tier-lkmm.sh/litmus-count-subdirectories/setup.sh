#!/usr/bin/env bash
# 样本现场（情形 litmus-count-subdirectories）：litmus/ 顶层一对 publish.litmus 与它去掉屏障的对照，子目录 litmus/nested/ 里一对 handoff.litmus 与对照
# （由顶层那一对把 publish 换成 handoff、data 换成 payload 得来），锚点各指 crates/demo/src/lib.rs 里的 publish、handoff，测试写出 publish.litmus 与 litmus/nested/handoff.litmus，
# 另写一处 handoff.litmus（项名只取文件名时也有 .rs 写出它）；假 herd7 与只有模型文件的假内核树。没有 .lkmm-key-samples/change.sh：只跑一趟、不给判定库。
# 门禁照 lkmm.sh 那一批递归数份数，成功句报「4 份 litmus 全过」；份数只数顶层时报成 2 份。
set -euo pipefail
mkdir -p crates/demo/src crates/demo/tests litmus/nested .lkmm-fake-herd7/linux/tools/memory-model
printf '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n' > crates/demo/Cargo.toml
printf 'pub fn publish() {}\n\npub fn handoff() {}\n' > crates/demo/src/lib.rs
cat > crates/demo/tests/publish_order.rs <<'RUST'
#[test]
fn litmus_files_are_read_by_path() {
    let top_level_litmus = "publish.litmus";
    let nested_litmus_path = "litmus/nested/handoff.litmus";
    let nested_litmus_file_name = "handoff.litmus";
    assert!(top_level_litmus.ends_with(".litmus"));
    assert!(nested_litmus_path.ends_with(nested_litmus_file_name));
}
RUST
cat > litmus/publish.litmus <<'LITMUS'
C publish

(*
 * 样本：写者先写数据、再立标志；读者先读标志、再读数据。
 *
 * singlefs-expect: Never
 * singlefs-models: crates/demo/src/lib.rs :: publish 写者
 * fake-observation: Never
 *)

{}

P0(int *data, int *flag)
{
	WRITE_ONCE(*data, 1);
	smp_wmb();
	WRITE_ONCE(*flag, 1);
}

P1(int *data, int *flag)
{
	int r0;
	int r1;

	r0 = READ_ONCE(*flag);
	smp_rmb();
	r1 = READ_ONCE(*data);
}

exists (1:r0=1 /\ 1:r1=0)
LITMUS
cat > litmus/publish-nofence.litmus <<'LITMUS'
C publish-nofence

(*
 * 样本：publish 的对照组，去掉写者一侧的屏障。
 *
 * singlefs-expect: Sometimes
 * fake-observation: Sometimes
 *)

{}

P0(int *data, int *flag)
{
	WRITE_ONCE(*data, 1);
	WRITE_ONCE(*flag, 1);
}

P1(int *data, int *flag)
{
	int r0;
	int r1;

	r0 = READ_ONCE(*flag);
	smp_rmb();
	r1 = READ_ONCE(*data);
}

exists (1:r0=1 /\ 1:r1=0)
LITMUS
sed -e 's/publish/handoff/g' -e 's/data/payload/g' litmus/publish.litmus > litmus/nested/handoff.litmus
sed -e 's/publish/handoff/g' -e 's/data/payload/g' litmus/publish-nofence.litmus > litmus/nested/handoff-nofence.litmus
grep -q 'singlefs-models: crates/demo/src/lib.rs :: handoff 写者' litmus/nested/handoff.litmus
grep -q '^C handoff-nofence$' litmus/nested/handoff-nofence.litmus
cat > .lkmm-fake-herd7/herd7 <<'HERD'
#!/usr/bin/env bash
# 样本用的假 herd7：-version 打固定版本串；-conf <cfg> <litmus> 照 litmus 里「fake-observation: <判定>」那一行交 Observation（只给 checker-tier-lkmm 的样本用）
if [[ "${1:-}" == -version ]]; then echo "7.57+fake, Rev: sample"; exit 0; fi
[[ -f linux-kernel.cfg ]] || { echo "假 herd7：没在 tools/memory-model 里跑" >&2; exit 2; }
litmus="${!#}"
name="$(sed -n '1s/^C[[:space:]]*//p' "$litmus")"
observation="$(sed -n 's/.*fake-observation:[[:space:]]*\([A-Za-z]*\).*/\1/p' "$litmus" | head -1)"
echo "Test $name Allowed"
echo "States 1"
echo "Observation $name ${observation:-Never} 0 1"
HERD
printf '"Linux-kernel memory consistency model (样本)"\nmodel linux-kernel.cat\nbell linux-kernel.bell\n' > .lkmm-fake-herd7/linux/tools/memory-model/linux-kernel.cfg
printf '"样本的 cat"\n' > .lkmm-fake-herd7/linux/tools/memory-model/linux-kernel.cat
printf '"样本的 bell"\n' > .lkmm-fake-herd7/linux/tools/memory-model/linux-kernel.bell
