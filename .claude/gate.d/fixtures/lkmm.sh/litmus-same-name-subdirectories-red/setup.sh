#!/usr/bin/env bash
# 样本现场（情形 litmus-same-name-subdirectories）：litmus/a/ 与 litmus/b/ 各有一对同名的 publish.litmus 与它去掉屏障的对照（b/ 的变量名换成 payload，内容与 a/ 不同），
# 测试只写出 litmus/a/publish.litmus。项名是相对 litmus/ 的路径：binding:a/publish.litmus 绿，binding:b/publish.litmus 红（没有 .rs 写出 b/publish.litmus）。
# 改前项名只取文件名、写出文件名按子串找：两份都叫 binding:publish.litmus，a/ 的那一处提及把 b/ 的也算绑上了，整格判绿。
set -euo pipefail
git init -q .   # 判定库的名字取被判那棵树的 git common-dir：不是 git 工作树的根 .claude/scripts/lkmm.sh 退 2、一条都不判
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
# 假 herd7 与只有模型文件的假内核树（门禁样本只走假 herd7；litmus 里没写 fake-observation 的当 Never）
mkdir -p .lkmm-fake-herd7/linux/tools/memory-model
cat > .lkmm-fake-herd7/herd7 <<'HERD'
#!/usr/bin/env bash
# 样本用的假 herd7：-version 打固定版本串；-conf <cfg> <litmus> 照 litmus 里「fake-observation: <判定>」那一行交 Observation（只给 门禁 lkmm 的样本用）
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
