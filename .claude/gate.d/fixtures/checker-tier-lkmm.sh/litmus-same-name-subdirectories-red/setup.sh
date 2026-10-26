#!/usr/bin/env bash
# 样本现场（情形 litmus-same-name-subdirectories）：litmus/a/ 与 litmus/b/ 各有一对同名的 publish.litmus 与它去掉屏障的对照（b/ 的变量名换成 payload，内容与 a/ 不同），
# 测试只写出 litmus/a/publish.litmus。项名是相对 litmus/ 的路径：binding:a/publish.litmus 绿，binding:b/publish.litmus 红（没有 .rs 写出 b/publish.litmus）。
# 改前项名只取文件名、写出文件名按子串找：两份都叫 binding:publish.litmus，a/ 的那一处提及把 b/ 的也算绑上了，整格判绿。
set -euo pipefail
mkdir -p crates/demo/src crates/demo/tests litmus/a litmus/b
printf '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n' > crates/demo/Cargo.toml
printf 'pub fn publish() {}\n' > crates/demo/src/lib.rs
printf '#[test]\nfn publish_litmus_is_read_by_path() {\n    let litmus_path = "litmus/a/publish.litmus";\n    assert!(litmus_path.ends_with(".litmus"));\n}\n' > crates/demo/tests/publish_order.rs
cat > litmus/a/publish.litmus <<'LITMUS'
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
cat > litmus/a/publish-nofence.litmus <<'LITMUS'
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
sed 's/data/payload/g' litmus/a/publish.litmus > litmus/b/publish.litmus
sed 's/data/payload/g' litmus/a/publish-nofence.litmus > litmus/b/publish-nofence.litmus
