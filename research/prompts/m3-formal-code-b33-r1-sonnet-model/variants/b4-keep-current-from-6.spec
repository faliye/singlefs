crates/singlefs-harness/src/model.rs
                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                });
                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf) if leaf.position_from_the_leftmost() < 6)\n                });