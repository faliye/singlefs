#!/usr/bin/env bash
# X5：private-values 路径打码的几种形状（假配置，值全是编的）。样本根拷自 gate-samples/doc-text/pv-tar-mirror-red，再造几份未跟踪的产物，
# after / before / pre0 三份门禁的 private-values 格（run_pv_cell.py 抽格里的 python 原样跑）各判一遍，数输出里假值出现几次、打码段几处。
# 用法：x5_shapes.sh <after 仓> <before 仓> <pre0 仓> <草稿目录（要不存在）> <模型目录>
set -u
after=$1; before=$2; pre0=$3; work=$4; model=$5
[[ ! -e "$work" ]] || { echo "  ✗ $work 已存在" >&2; exit 2; }
mkdir -p "$work"
cp -a "$model/gate-samples/doc-text/pv-tar-mirror-red" "$work/s"
(cd "$work/s" && bash setup.sh >/dev/null 2>&1; true)
rm -rf "$work/s/research/results/from-peer"
VALUE=/srv/sample-private/peer-tree   # 样本配置里 PEER_REPOSITORY_DIRECTORY 的假值
mk() { mkdir -p "$(dirname "$work/s/$1")"; printf 'name=x rows=1\n' > "$work/s/$1"; }
mk research/results/mirror/srv/sample-private/peer-tree/a.out                       # 1 值跨四级目录（tar 镜像）
mk research/results/mirror2/srv/sample-private/peer-tree-2/b.out                    # 2 值是目录名的前缀（peer-tree-2）
mk research/results/e1/srv/sample-private/peer-tree/inner/srv/sample-private/peer-tree/c.out   # 3 同一路径里值出现两次
mk research/results/e2-sample-peer-box-17.out                                       # 4 主机名替身作文件名的一部分
mk research/results/plain/ordinary.out                                              # 5 不含值（对照）
mk research/results/sample-private/ok.out                                           # 6 只含值的一半（/srv 之后的 sample-private 不是整串）
for v in after before pre0; do
  repo=$after; [[ $v == before ]] && repo=$before; [[ $v == pre0 ]] && repo=$pre0
  python3 -B "$model/tools/run_pv_cell.py" "$repo" "$work/s" > "$work/out-$v.txt" 2>&1; rc=$?
  leak=$(grep -c -F -e "$VALUE" -e "srv/sample-private" -e "sample-peer-box-17" "$work/out-$v.txt")
  masks=$(grep -o '〔文件名含私有值〕' "$work/out-$v.txt" | wc -l)
  echo "== $v rc=$rc 含假值的输出行=$leak 打码处=$masks"
  grep -E '^ +(research|〔)' "$work/out-$v.txt" | sed 's/  *键名.*//' | sed 's/^ */   /'
done
