#!/usr/bin/env python3
"""攻方变异（m3-batch10z-vw-code-r1，opus 腿）：mutants.py <仓副本根> <变异名> 改那份副本；mutants.py x --list 列名字。
原文在被改文件里恰好命中一次才改，否则退 3。只改副本，不碰冻结副本与仓。"""
import pathlib
import sys

MS = "crates/singlefs-core/src/mounted_session.rs"
BD = "crates/singlefs-core/src/block_device.rs"

MUTANTS = {
    # 上一轮正推活下来的那一个（原文照 research/prompts/m3-batch10z-tu-code-r1-sonnet-model/scripts/mut3.py 的 walkSkipped）
    "walkSkipped": (MS, "        RemainderOfTheUserChange::WalkTheQueue => {\n            let mut version_to_walk",
                    "        RemainderOfTheUserChange::WalkTheQueue if true => Ok(()),\n        RemainderOfTheUserChange::WalkTheQueue => {\n            let mut version_to_walk"),
    # 攻方上一轮 F、H（与变异表第二十二块第 8、9 行同一原文）
    "F-walk-before-keeps-only-the-walk": (MS, "                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),\n                published_in_this_change,",
                    "                RemainderOfTheUserChange::WalkTheQueue,\n                published_in_this_change,"),
    "H-after-the-intent-keeps-the-change": (MS, "                        RemainderOfTheUserChange::WalkTheQueue,\n                        published_in_this_change,",
                    "                        RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),\n                        published_in_this_change,"),
    # 这一轮新代码
    "M1-version-check-only-txg": (MS, "            version_of_trees.root.instance == stuck.instance_of_the_version_refused_on\n                && version_of_trees",
                    "            true\n                && version_of_trees"),
    "M2-version-check-only-instance": (MS, "                && version_of_trees.root.checkpoint_txg\n                    == stuck.checkpoint_txg_of_the_version_refused_on",
                    "                && true"),
    "M3-overwrite-kept-one-second-later": (MS, "            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content.to_vec(),\n                write_time_seconds: file.write_time_seconds,",
                    "            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content.to_vec(),\n                write_time_seconds: file.write_time_seconds + 1,"),
    "M4-unlink-kept-on-the-next-inode": (MS, "            } => UserChangeKeptByTheRefusal::Unlink {\n                inode,",
                    "            } => UserChangeKeptByTheRefusal::Unlink {\n                inode: InodeNumber(inode.0 + 1),"),
    "M5-new-inodes-kept-one-more": (MS, "            } => UserChangeKeptByTheRefusal::NewInodes {\n                count,",
                    "            } => UserChangeKeptByTheRefusal::NewInodes {\n                count: count + 1,"),
    "M6-sequential-kept-one-second-later": (MS, "            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {\n                content: file.content.to_vec(),\n                write_time_seconds: file.write_time_seconds,",
                    "            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {\n                content: file.content.to_vec(),\n                write_time_seconds: file.write_time_seconds + 1,"),
    "M7-overwrite-content-kept-empty": (MS, "            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content.to_vec(),",
                    "            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: Vec::new(),"),
    "M8-version-without-file-trees-counts-as-moved": (MS, "            PoolVersion::WithFile(version_of_trees)\n            | PoolVersion::WithTreesAndNoFile(version_of_trees)\n                if is_the_version_refused_on(version_of_trees) =>",
                    "            PoolVersion::WithFile(version_of_trees)\n                if is_the_version_refused_on(version_of_trees) =>"),
    "M9-instance-kept-off-by-one": (MS, "            instance_of_the_version_refused_on: current.root.instance,",
                    "            instance_of_the_version_refused_on: InstanceGeneration(current.root.instance.0 + 1),"),
    "M10-overlay-device-size-unbounded": (MS, "            self.physical_block_size,\n            self.disk.size_in_bytes(),",
                    "            self.physical_block_size,\n            u64::MAX,"),
}


def main():
    if len(sys.argv) >= 3 and sys.argv[2] == "--list":
        print("\n".join(MUTANTS))
        return
    root = pathlib.Path(sys.argv[1])
    name = sys.argv[2]
    rel, old, new = MUTANTS[name]
    path = root / rel
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        print(f"{name}: 原文命中 {count} 次（要恰好一次）", file=sys.stderr)
        sys.exit(3)
    path.write_text(text.replace(old, new))
    print(f"{name}: 改了 {rel}")


if __name__ == "__main__":
    main()
