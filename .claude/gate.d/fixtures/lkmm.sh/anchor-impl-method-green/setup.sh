#!/usr/bin/env bash
# 样本现场（情形 anchor-impl-method）：锚点写成「文件 :: <impl 类型>::<方法>」，那份 .rs 里恰好一个这个限定名的单元，静态层判绿（herd7 那一层不跑）。
set -euo pipefail
git init -q .   # 判定库的名字取被判那棵树的 git common-dir：不是 git 工作树的根 .claude/scripts/lkmm.sh 退 2、一条都不判
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
