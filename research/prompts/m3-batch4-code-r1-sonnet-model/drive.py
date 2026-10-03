#!/usr/bin/env python3
import sys,subprocess,os
W='/tmp/claude-1000/m3-batch4-code-r1-sonnet/'
WALK='crates/singlefs-checker/src/walk.rs'; IMG='crates/singlefs-checker/src/image.rs'
P={}
def p(label,file,orig,repl): P[label]=(file,orig,repl)
p('A1_generic_ptr_site',WALK,"        judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, what);\n","")
p('A2_inode_child_site',WALK,'''            judge_pointer_extent_offset_is_zero(
                &mut self.judgements,
                &child,
                "inode 内部条目的子指针",
            );
''',"")
p('A3_extent_data_site',WALK,'''        judge_pointer_extent_offset_is_zero(
            &mut self.judgements,
            &pointer,
            "extent 记录的数据指针",
        );
''',"")
p('A4_instance_chain_site',WALK,"            judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, &pointer_what);\n","")
p('B1_i112_chain_site',WALK,"            self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, &what);\n","")
p('B2_i112_data_site',WALK,'        self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, "数据单元");\n',"")
p('C1_i925_inode_half',WALK,"            previous_snapshot_txg <= version_txg\n        } else {","            true\n        } else {")
p('C2_i925_noninode_half',WALK,"            previous_snapshot_txg == 0\n        };","            true\n        };")
p('D1_i926_lower_bound',WALK,"object_birth <= change_count && change_count <= container_birth_txg,","change_count <= container_birth_txg,")
p('D2_i926_upper_bound',WALK,"object_birth <= change_count && change_count <= container_birth_txg,","object_birth <= change_count,")
p('E1_i811_birth_tree',WALK,"unit[6] == named.unit_class && identity_matches && birth_tree_matches;","unit[6] == named.unit_class && identity_matches;")
p('E2_i811_class',WALK,"unit[6] == named.unit_class && identity_matches && birth_tree_matches;","identity_matches && birth_tree_matches;")
p('E3_i811_instance',WALK,"                                && identity.write_order.instance() == named_instance\n","")
p('E4_i811_tail',WALK,"                                && identity.tail.birth_identity_tail_number() == named_tail\n","")
p('E5_i811_birth_txg',WALK,"identity.write_order.birth_txg() == named.birth_txg\n                                && identity.write_order.instance() == named_instance","true\n                                && identity.write_order.instance() == named_instance")
p('F1_i810_tree_table',WALK,"let same = segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER]\n                    == root_bytes[ROOT_RECORD_TREE_TABLE_POINTER]\n                    &&","let same = true\n                    &&")
p('F2_i810_mapping_root',WALK,"                    && segment[NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER]\n                        == root_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER]\n","")
p('F3_i810_watermark',WALK,"                    && segment[NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK]\n                        == root_bytes[ROOT_RECORD_TREE_IDENTIFIER_WATERMARK]\n","")
p('G1_i49_second_site',WALK,'''            if let Some(birth_txg) = self.birth_txg_of_walked_units.get(&unit).copied() {
                self.judge_unit_birth_against_an_older_candidate_root(
                    birth_txg,
                    &format!("别的版本先走过的盘 {} 槽 {} 的单元", unit.0, unit.1),
                );
            }
''',"")
p('H1_i717_region',IMG,"expected_region.is_none_or(|expected| expected == *region) && expected_slot == *slot;","expected_slot == *slot;")
p('H2_i717_slot',IMG,"expected_region.is_none_or(|expected| expected == *region) && expected_slot == *slot;","expected_region.is_none_or(|expected| expected == *region);")
p('I1_i812_strict',WALK,"record.checkpoint_txg >= previous_txg,","record.checkpoint_txg > previous_txg,")
p('I2_i81_count_strict',WALK,"record_count <= declared_in_flight_record_limit,","record_count < declared_in_flight_record_limit,")
p('J1_i75_min_width',IMG,"        .max()\n        .map(|bytes| crate::DecisionWidth::DeclaredOnly { bytes })","        .min()\n        .map(|bytes| crate::DecisionWidth::DeclaredOnly { bytes })")
p('K1_i85_unchoosable_not_counted',IMG,"        | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => true,\n        SystemConfigurationSlotReading::NotSelfVerified => false,","        => true,\n        SystemConfigurationSlotReading::SelfVerifiedButNotChoosable\n        | SystemConfigurationSlotReading::NotSelfVerified => false,")
p('L1_i41_limit',WALK,"        if records_taken_before_this_one >= in_flight_record_limit {\n            break;\n        }\n","        let _ = records_taken_before_this_one;\n")
p('L2_i41_follow',WALK,"                || readable_record_of_the_same_publish_follows(\n                    records_the_replay_reads,\n                    counter,\n                    record,\n                ))","                || false)")
p('L3_i41_commit',WALK,"            && (record.commit_marker != CommitMarker::Present\n                ||","            && (false\n                ||")
p('L4_i41_open_publish',WALK,"        if !fits_the_open_publish {\n            break;\n        }\n","")
p('L5_i41_prefix_entry',WALK,"            || !record_can_enter_a_replay_prefix(records_the_replay_reads, counter, record)\n","")
p('L6_i41_chain_counter',WALK,"            Some(expected) => counter == expected,","            Some(_expected) => true,")
p('M1_i716_diff',IMG,"both_written_by_mkfs || slot_zero_generation.abs_diff(slot_one_generation) == 1,","true,")
p('M2_i716_parity',IMG,"generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE == slot_index,","true,")
T={'ptr':['--test','checker_known_bad_images','--','each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant'],
   'rel':['--test','checker_known_bad_images','--','each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite'],
   'ring':['--test','checker_known_bad_images','--','each_ring_geometry_and_root_slot_alignment_bad_image_reddens_only_its_own_invariant'],
   'full':['--test','checker_known_bad_images'],
   'wm':['--test','writable_mount_of_a_formatted_pool'],
   'i41':['--test','checker_known_bad_images','--','replay'],
   'sc':['--test','checker_known_bad_images','--','system_configuration'],
}
tkey=sys.argv[1]
for label in sys.argv[2:]:
    f,o,r=P[label]
    open(W+'probes/'+label+'.orig','w').write(o); open(W+'probes/'+label+'.repl','w').write(r)
    # keep trailing newline semantics: use .keepnl suffix by passing exact files
    import shutil
    shutil.copyfile(W+'probes/'+label+'.orig',W+'probes/'+label+'.orig.keepnl'); shutil.copyfile(W+'probes/'+label+'.repl',W+'probes/'+label+'.repl.keepnl')
    cmd=['python3',W+'probe.py',label+'__'+tkey,f,W+'probes/'+label+'.orig.keepnl',W+'probes/'+label+'.repl.keepnl']+T[tkey]
    print(subprocess.run(cmd,capture_output=True,text=True).stdout.strip(),flush=True)
