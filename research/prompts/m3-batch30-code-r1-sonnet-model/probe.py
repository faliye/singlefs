#!/usr/bin/env python3
"""在仓副本的 checker_known_bad_images.rs 里加两处 eprintln 探针（只打红了哪几条不变量，不改任何断言）。用法：probe.py <仓副本根>"""
import sys
root = sys.argv[1]
path = root + "/crates/singlefs-harness/tests/checker_known_bad_images.rs"
text = open(path, encoding="utf-8").read()
first_old = """        let verdicts = check_pool_image(&image);
        let violated = violated_invariant_set(&verdicts);
        assert!(
            violated.contains(subtree_invariant),"""
first_new = """        let verdicts = check_pool_image(&image);
        let violated = violated_invariant_set(&verdicts);
        eprintln!("PROBE-FW kind={kind} loc={location_index} only_violation={violated_by_the_violation_only:?} fw={violated:?}");
        assert!(
            violated.contains(subtree_invariant),"""
second_old = """        violated_on_each_device.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_device[0], violated_on_each_device[1],
        "同一张镜像只差码 2 那一份在哪块盘：红的同一批"
    );
}

/// 条目不改那一形"""
second_new = """        eprintln!("PROBE-HV kind={kind} loc={location_index} {entry_invariant} violated={:?}", violated_invariant_set(&verdicts));
        violated_on_each_device.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_device[0], violated_on_each_device[1],
        "同一张镜像只差码 2 那一份在哪块盘：红的同一批"
    );
}

/// 条目不改那一形"""
for old, new in ((first_old, first_new), (second_old, second_new)):
    assert text.count(old) == 1, "探针锚点要恰好命中一次"
    text = text.replace(old, new)
open(path, "w", encoding="utf-8").write(text)
print("probe ok")
