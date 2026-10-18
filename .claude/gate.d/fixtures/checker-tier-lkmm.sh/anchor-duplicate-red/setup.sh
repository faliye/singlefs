#!/usr/bin/env bash
# 样本现场（情形 anchor-duplicate）：锚点的限定名 publish 在那份 .rs 里有两个单元（cfg 两支各一个），锚点判不准是哪一个，判红。
set -euo pipefail
mkdir -p crates/demo/src crates/demo/tests litmus
printf '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n' > crates/demo/Cargo.toml
printf '#[cfg(unix)]\npub fn publish() {}\n\n#[cfg(not(unix))]\npub fn publish() {}\n' > crates/demo/src/lib.rs
printf '#[test]\nfn publish_litmus_is_read_by_name() {\n    let litmus_file_name = "publish.litmus";\n    assert!(litmus_file_name.ends_with(".litmus"));\n}\n' > crates/demo/tests/publish_order.rs
cat > litmus/publish.litmus <<'LITMUS'
C publish

(*
 * 样本：锚点的限定名在那份文件里有两处。
 *
 * singlefs-expect: Never
 * singlefs-models: crates/demo/src/lib.rs :: publish 写者
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
