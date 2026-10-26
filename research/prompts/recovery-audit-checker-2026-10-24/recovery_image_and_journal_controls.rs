// 在 checker 的临时源码副本中追加；不改产品源码。路径由 verify_audit.py 注入。
#[cfg(test)]
mod audit_image_controls {
    use super::*;
    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
    use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite, WrittenContents};
    use singlefs_harness::segments::StepKind;
    #[path = "__COMMON_FIXTURE_PATH__"]
    mod common;
    struct Image<'a>(&'a MemoryPool);
    impl ImageReader for Image<'_> {
        fn devices(&self) -> Vec<u32> { original_checker::image::ImageReader::devices(self.0) }
        fn device_bytes(&self, d:u32) -> Option<u64> { original_checker::image::ImageReader::device_bytes(self.0,d) }
        fn read(&self,d:u32,o:u64,n:usize)->Option<Vec<u8>> { original_checker::image::ImageReader::read(self.0,d,o,n) }
        fn candidate_unit_slots(&self,d:u32)->Option<Vec<u64>> { original_checker::image::ImageReader::candidate_unit_slots(self.0,d) }
        fn candidate_journal_slots(&self,d:u32)->Option<Vec<u64>> { original_checker::image::ImageReader::candidate_journal_slots(self.0,d) }
    }
    fn control() -> bool { std::env::var_os("AUDIT_RECHECK_CONTROL").is_some() }
    #[test]
    fn audit_full_image_bad_reference_and_restore_controls() {
        let built=common::build_pool("audit-reference-controls");
        let original=built.memory_pool();
        let mut bad=original.clone();
        assert!(check_pool_image(&Image(&original)).iter().all(|(_,v)| !matches!(v,InvariantVerdict::Violated(_))));
        let geometry=chosen_system_configurations(&Image(&bad))[0].1.as_ref().unwrap().1;
        let mut changed=0;
        for (_,_,d,o) in root_slot_positions(&geometry) {
            let Some(mut bytes)=Image(&bad).read(d,o,512) else {continue};
            let Ok(root)=crate::check_root_slot(&bytes,&geometry.filesystem_identifier) else {continue};
            if root.checkpoint_txg!=1 {continue}
            let original_birth=read_u64(&bytes,78);
            let pointer=parse_node_pointer(&bytes[36..122]);
            let pointed=Image(&bad).read(pointer.locations[0].device,pointer.locations[0].slot*16384,16384).unwrap();
            let view=crate::index_node_view(&pointed).unwrap();
            assert_eq!(view.birth_txg,original_birth);
            assert_ne!(view.birth_txg,999);
            bytes[78..86].copy_from_slice(&999u64.to_le_bytes());
            bytes[138..170].fill(0);
            let crc=crc32_castagnoli_table(&bytes);
            bytes[138..142].copy_from_slice(&crc.to_le_bytes());
            assert!(crate::check_root_slot(&bytes,&geometry.filesystem_identifier).is_ok());
            bad.apply_writes(&[RetainedWrite{device:DeviceIdentity(d),offset:DeviceOffsetInBytes(o),kind:StepKind::RootRecordFua,is_force_unit_access:true,contents:WrittenContents::Bytes(bytes)}]);
            changed+=1;
            println!("AUDIT root_txg=1 original_pointer_birth={original_birth} actual_node_birth={} mutated_pointer_birth=999",view.birth_txg);
        }
        assert!(changed>0);
        common::copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut bad);
        let verdicts=check_pool_image(&Image(&bad));
        let red:Vec<_>=verdicts.iter().filter(|(_,v)|matches!(v,InvariantVerdict::Violated(_))).collect();
        let identity=verdicts.iter().find(|(i,_)|*i=="I-1.2").unwrap();
        assert_eq!(matches!(identity.1,InvariantVerdict::Violated(_)),control());
        if !control() {assert!(red.is_empty(),"{red:?}");}
        println!("AUDIT full_image control={} mutated_roots={changed} I-1.2={:?} all_red={red:?}",control(),identity.1);
        // 恢复输入后必须重新无违例；控制修改不能把合法镜像误判成红。
        let restored=check_pool_image(&Image(&original));
        assert!(restored.iter().all(|(_,v)|!matches!(v,InvariantVerdict::Violated(_))));
        println!("AUDIT restored_image all_red=0");
    }
    fn record(instance:u32,txg:u64,fsid:u64)->Vec<u8> {
        let mut bytes=vec![0;4096];
        bytes[..4].copy_from_slice(b"SFSJ");bytes[4]=1;bytes[7]=1;bytes[86]=1;bytes[87]=1;
        bytes[8..12].copy_from_slice(&4096u32.to_le_bytes());
        bytes[16..20].copy_from_slice(&instance.to_le_bytes());bytes[20]=1;
        bytes[26..34].copy_from_slice(&txg.to_le_bytes());bytes[472..480].copy_from_slice(&fsid.to_le_bytes());
        let crc=crc32_castagnoli_table(&bytes);bytes[46..50].copy_from_slice(&crc.to_le_bytes());
        assert!(crate::check_journal_record(&bytes,fsid).is_ok());
        assert!(singlefs_core::journal::JournalRecord::parse(&bytes,fsid).is_some());
        bytes
    }
    #[test]
    fn audit_journal_same_real_geometry_and_same_image() {
        let built=common::build_pool("audit-journal-controls");
        let base=built.memory_pool();
        let cfg=singlefs_core::recovery::choose_system_configuration(&base).unwrap();
        let geometry=chosen_system_configurations(&Image(&base))[0].1.as_ref().unwrap().1;
        let fsid=singlefs_core::unit::unit_filesystem_identifier(&cfg.immutable.filesystem_identifier);
        for different_instances in [false,true] {
            for reverse in [false,true] {
                let mut image=base.clone();
                let ring_start=geometry.journal_ring_start_slot*16384;
                let mut writes=Vec::new();
                for device in Image(&base).devices() {
                    writes.push(RetainedWrite{device:DeviceIdentity(device),offset:DeviceOffsetInBytes(ring_start),kind:StepKind::ZeroFill,is_force_unit_access:false,contents:WrittenContents::Zeros{length:geometry.journal_ring_bytes}});
                }
                let mut rows=[(1,10),(if different_instances {2}else{1},20)];
                if reverse {rows.reverse();}
                for (slot,(instance,txg)) in rows.iter().copied().enumerate() {
                    writes.push(RetainedWrite{device:DeviceIdentity(0),offset:DeviceOffsetInBytes(ring_start+slot as u64*4096),kind:StepKind::JournalRecord,is_force_unit_access:false,contents:WrittenContents::Bytes(record(instance,txg,fsid))});
                }
                image.apply_writes(&writes);
                let cpu=scanned_journal_records_of_device(&Image(&image),&geometry,0,fsid);
                let core=singlefs_core::recovery::scan_journal(&image,&cfg);
                if different_instances {
                    assert_eq!(cpu.len(),1);assert_eq!(core.len(),2);
                } else {
                    assert_eq!(core[&(InstanceGeneration(1),1)].checkpoint_txg.0,rows[0].1);
                    assert_eq!(cpu[&1].checkpoint_txg,rows[1].1);
                    assert_ne!(cpu[&1].checkpoint_txg,core[&(InstanceGeneration(1),1)].checkpoint_txg.0);
                }
                println!("AUDIT same_image journal different_instances={different_instances} reverse={reverse} checker={:?} core={:?}",cpu.iter().map(|(counter,r)|(r.instance,*counter,r.checkpoint_txg)).collect::<Vec<_>>(),core.iter().map(|((i,c),r)|(i.0,*c,r.checkpoint_txg.0)).collect::<Vec<_>>());
            }
        }
    }
}
