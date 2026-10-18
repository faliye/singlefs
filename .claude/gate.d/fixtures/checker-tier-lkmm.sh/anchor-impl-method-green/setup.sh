#!/usr/bin/env bash
# 样本现场（情形 anchor-impl-method）：锚点写成「文件 :: <impl 类型>::<方法>」，那份 .rs 里恰好一个这个限定名的单元，静态层判绿（herd7 那一层不跑）。
set -euo pipefail
mkdir -p crates/demo/src crates/demo/tests litmus
printf '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n' > crates/demo/Cargo.toml
printf 'pub struct Pool;\n\nimpl Pool {\n    pub fn publish(&self) {}\n}\n\npub fn publish() {}\n' > crates/demo/src/lib.rs
printf '#[test]\nfn publish_litmus_is_read_by_name() {\n    let litmus_file_name = "publish.litmus";\n    assert!(litmus_file_name.ends_with(".litmus"));\n}\n' > crates/demo/tests/publish_order.rs
cat > litmus/publish.litmus <<'LITMUS'
C publish

(*
 * 样本：写者是 Pool 的 publish 方法（自由 fn publish 同名，按 impl 限定名点的是方法那一个）。
 *
 * singlefs-expect: Never
 * singlefs-models: crates/demo/src/lib.rs :: <impl Pool>::publish 写者
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
