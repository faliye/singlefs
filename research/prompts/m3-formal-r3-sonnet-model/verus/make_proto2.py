#!/usr/bin/env python3
"""原型 2：在原型 1 上修三处——锚点函数循环尾补证明、把循环头的 expected_next 存成幽灵量、函数尾补出口证明。"""
import pathlib, sys
src = pathlib.Path(sys.argv[1]).read_text()
out_dir = pathlib.Path(sys.argv[2]); (out_dir / "src").mkdir(parents=True, exist_ok=True)

def sub(old, new):
    global src
    assert src.count(old) == 1, (src.count(old), old[:80])
    src = src.replace(old, new)

# (1) 锚点函数：循环尾
sub("""            found = Some(record.counter);
        }
        index += 1;
    }
    // [原文 2985–2986]
    Ok(found)""",
"""            found = Some(record.counter);
        }
        proof {
            // 本条是不是「那一次发布带末条标志的」：与 spec 谓词同义
            assert(root_publish_last_flagged(records@, root_instance, root_checkpoint_txg, index as int)
                == (record.instance == root_instance && record.checkpoint_txg == root_checkpoint_txg
                && record.ends_its_publish));
        }
        index += 1;
    }
    // [原文 2985–2986]
    Ok(found)""")

# (2) 主循环头：存下 expected_next
sub("""    {
        let record = &records[index];
        // [原文 3078–3080] above 的筛选条件，同上面那一遍""",
"""    {
        let record = &records[index];
        let ghost expected_at_head = expected_next;
        // [原文 3078–3080] above 的筛选条件，同上面那一遍""")
sub("""            match expected_next {
                Some(expected_key) => {
                    assert(record.instance == expected_key.0 && record.counter == expected_key.1);
                },""",
"""            match expected_at_head {
                Some(expected_key) => {
                    assert(record.instance == expected_key.0 && record.counter == expected_key.1);
                },""")

# (3) 函数尾
sub("""    // [原文 3228]
    Ok(CoreOutcome { report, rebuilt, applied: Ghost(applied) })""",
"""    proof {
        let consumed = applied + as_ints(open@);
        assert(chain_from_the_head(root_checkpoint_txg, records@, root_own_record_counter, applied)) by {
            if applied.len() > 0 {
                assert(consumed[0] == applied[0]);
            }
            assert forall|position: int|
                0 <= position && position + 1 < applied.len() implies records@[(
            #[trigger] applied[position + 1])].counter == records@[applied[position]].counter + 1 by {
                assert(consumed[position + 1] == applied[position + 1]);
                assert(consumed[position] == applied[position]);
            }
        }
        assert(anchor_is(root_instance, root_checkpoint_txg, records@, root_own_record_counter));
    }
    // [原文 3228]
    Ok(CoreOutcome { report, rebuilt, applied: Ghost(applied) })""")
(out_dir / "src/lib.rs").write_text(src)
print("wrote", out_dir)
