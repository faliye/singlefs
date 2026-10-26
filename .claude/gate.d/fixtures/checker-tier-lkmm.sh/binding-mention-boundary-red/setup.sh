#!/usr/bin/env bash
# 样本现场（情形 binding-mention-boundary）：一对 publish.litmus 与它去掉屏障的对照，锚点恰好命中 fn publish，
# 唯一的测试只写出 mp-publish.litmus（别的 litmus 的名字，publish.litmus 只是它的尾巴）。写出项名那一查看边界：前面接着 -，不算写出 publish.litmus，判红。
# 改前按子串找（grep -F），mp-publish.litmus 里含 publish.litmus，整格判绿。
set -euo pipefail
mkdir -p crates/demo/src crates/demo/tests litmus
printf '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n' > crates/demo/Cargo.toml
printf 'pub fn publish() {}\n' > crates/demo/src/lib.rs
printf '#[test]\nfn another_litmus_is_read_by_name() {\n    let litmus_file_name = "mp-publish.litmus";\n    assert!(litmus_file_name.ends_with(".litmus"));\n}\n' > crates/demo/tests/publish_order.rs
cat > litmus/publish.litmus <<'LITMUS'
C publish

(*
 * 样本：写者先写数据、再立标志；读者先读标志、再读数据。
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
