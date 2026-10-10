import sys,shutil,subprocess,os,json
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch52-54-code-r1-sonnet')
MUT={
 'm1_left_first':('crates/singlefs-core/src/inode_tree.rs',
   "            if right_pair_fits {\n                self.absorb_the_right_neighbour(current_position);\n            } else if left_pair_fits {\n                self.absorb_the_right_neighbour(current_position - 1);\n                current_position -= 1;\n            } else {",
   "            if left_pair_fits {\n                self.absorb_the_right_neighbour(current_position - 1);\n                current_position -= 1;\n            } else if right_pair_fits {\n                self.absorb_the_right_neighbour(current_position);\n            } else {"),
 'm2_no_merge_after_borrow':('crates/singlefs-core/src/inode_tree.rs',
   "        borrower.is_changed_this_publish = true;\n        self.absorb_until_no_pair_fits(lender_position);\n    }",
   "        borrower.is_changed_this_publish = true;\n    }"),
 'm3_chain_stops_after_one':('crates/singlefs-core/src/inode_tree.rs',
   "            absorbed_at_least_once = true;\n        }\n    }",
   "            absorbed_at_least_once = true;\n            return true;\n        }\n    }"),
 'm4_empty_drop_no_merge':('crates/singlefs-core/src/inode_tree.rs',
   "        if has_new_neighbours {\n            tree.absorb_until_no_pair_fits(holding_position - 1);\n        }",
   "        if has_new_neighbours && holding_position == usize::MAX {\n            tree.absorb_until_no_pair_fits(holding_position - 1);\n        }"),
 'm5_no_left_absorb':('crates/singlefs-core/src/inode_tree.rs',
   "            let left_pair_fits =\n                current_position > 0 && self.pair_fits_in_one_container(current_position - 1);",
   "            let left_pair_fits = false;"),
 'm6_borrow_rounds_up':('crates/singlefs-core/src/inode_tree.rs',
   ".map_or(0, |difference| difference / 2);",
   ".map_or(0, |difference| (difference + 1) / 2);"),
 'm7_floor_116':('crates/singlefs-core/src/inode_tree.rs',
   'usize::try_from(INODE_LEAF_RECORDS.div_ceil(2)).expect("117")',
   'usize::try_from(INODE_LEAF_RECORDS.div_ceil(2) - 1).expect("117")'),
 'm8_floor_strict':('crates/singlefs-core/src/inode_tree.rs',
   "        if records_in_the_borrower >= records_below_which_a_leaf_borrows_from_its_right_neighbour()",
   "        if records_in_the_borrower > records_below_which_a_leaf_borrows_from_its_right_neighbour()"),
 'm9_separator_key_not_updated':('crates/singlefs-core/src/inode_tree.rs',
   '        lender.contents.separator_key_in_the_parent_entry = lender\n            .contents\n            .records\n            .first()\n            .map(|record| record.inode)\n            .expect("右邻借出的条数少于它的记录数：借完至少还剩一条");\n',
   ''),
}
name=sys.argv[1]
f,o,r=MUT[name]
p=os.path.join(W,'repo',f)
pr=os.path.join(W,'pristine',os.path.basename(f))
s=open(pr,encoding='utf-8').read()
assert s.count(o)==1,(name,s.count(o))
open(p,'w',encoding='utf-8').write(s.replace(o,r))
print('applied',name)
