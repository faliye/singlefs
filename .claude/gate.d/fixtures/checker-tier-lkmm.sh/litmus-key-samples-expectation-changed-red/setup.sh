#!/usr/bin/env bash
# 样本现场（情形 expectation-changed，S27）：同 litmus-key-samples-comment-only 的那一对 litmus、锚点、测试、假 herd7 与假内核树；
# 门禁头一趟现判、记下，跑 .lkmm-key-samples/change.sh 把 publish.litmus 的 singlefs-expect 从 Never 改成 Sometimes（这一行在注释里），
# 第二趟 herd7:publish.litmus 要现判（原因「期望 变了」），假 herd7 照 fake-observation 还判 Never，与新的声明不符、判红；对照那一条照复用。
# 弄坏开关 GATE_LKMM_BREAK=litmus-key-drops-every-comment 下这一条照复用、整格绿退 77，这份样本判错。
set -euo pipefail
mkdir -p crates/demo/src crates/demo/tests litmus .lkmm-fake-herd7/linux/tools/memory-model .lkmm-key-samples
printf '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n' > crates/demo/Cargo.toml
printf 'pub fn publish() {}\n\npub fn unrelated() {}\n' > crates/demo/src/lib.rs
printf '#[test]\nfn publish_litmus_is_read_by_name() {\n    let litmus_file_name = "publish.litmus";\n    assert!(litmus_file_name.ends_with(".litmus"));\n}\n' > crates/demo/tests/publish_order.rs
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
cat > .lkmm-key-samples/change.sh <<'CHANGE'
#!/usr/bin/env bash
# 第二趟之前：publish.litmus 的 singlefs-expect 从 Never 改成 Sometimes（S27；fake-observation 不动）
set -euo pipefail
sed -i 's/ \* singlefs-expect: Never/ * singlefs-expect: Sometimes/' litmus/publish.litmus
grep -q 'singlefs-expect: Sometimes' litmus/publish.litmus
CHANGE
