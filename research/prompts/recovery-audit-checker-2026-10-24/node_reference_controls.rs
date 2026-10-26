
#[cfg(test)]
mod audit_repro {
    use super::*;
    struct Reader(Vec<u8>);
    impl ImageReader for Reader {
        fn devices(&self)->Vec<u32>{vec![0,1]}
        fn device_bytes(&self,_:u32)->Option<u64>{Some(1<<30)}
        fn read(&self,_:u32,offset:u64,len:usize)->Option<Vec<u8>>{
            (offset==16384 && len==16384).then(||self.0.clone())
        }
        fn candidate_unit_slots(&self,_:u32)->Option<Vec<u64>>{Some(vec![1])}
        fn candidate_journal_slots(&self,_:u32)->Option<Vec<u64>>{Some(vec![])}
    }
    fn fixture()->(Reader,Vec<u8>){
        let mut unit=vec![0u8;16384];
        unit[..4].copy_from_slice(b"SFSU"); unit[4]=1; unit[6]=2;
        unit[42..50].copy_from_slice(&7u64.to_le_bytes()); unit[51]=8;
        unit[100..102].copy_from_slice(&8u16.to_le_bytes()); // entry width; zero entries
        let crc=crc32_castagnoli_table(&unit[102..]);
        unit[92..96].copy_from_slice(&crc.to_le_bytes());
        let crc=crc32_castagnoli_table(&unit[..102]);
        unit[10..14].copy_from_slice(&crc.to_le_bytes());
        assert!(crate::index_node_view(&unit).is_ok(), "{:?}", crate::index_node_view(&unit));
        let mut pointer=vec![0u8;86];
        for (device,at) in [(0u32,50),(1u32,64)]{
            pointer[at..at+4].copy_from_slice(&device.to_le_bytes());
            pointer[at+4..at+10].copy_from_slice(&1u64.to_le_bytes()[..6]);
            pointer[at+10..at+14].copy_from_slice(&crc32_castagnoli_table(&unit).to_le_bytes());
        }
        (Reader(unit),pointer)
    }
    fn read(walk:&mut Walk<'_>,p:&[u8],tree:u64)->IndexNodeReading{
        walk.read_index_node(p,tree,crate::KEY_SCHEMA_INODE,"audit",KeyRangeReading::SubtreeCoverageOfTheInodeTreeJudgedByTheCaller,EntryFieldTableRegistration::Registered,PositionAuthority::ThePointerItself)
    }
    #[test]
    fn audit_cached_node_skips_new_pointer_birth_identity(){
        let (r,mut p)=fixture();
        let mut w=Walk::starting_with(&r,Judgements::default(),0,0,10);
        assert!(matches!(read(&mut w,&p,7),IndexNodeReading::WalkedFirstByThisVersion{..}));
        assert_eq!(w.judgements.violation_count("I-1.2"),0);
        w.begin_walking_a_version();
        p[42..50].copy_from_slice(&9u64.to_le_bytes());
        assert!(matches!(read(&mut w,&p,7),IndexNodeReading::WalkedByAnEarlierVersion{..}));
        assert_eq!(w.judgements.violation_count("I-1.2"), if std::env::var_os("AUDIT_RECHECK_CONTROL").is_some(){1}else{0});
        let mut fresh=Walk::starting_with(&r,Judgements::default(),0,0,10);
        read(&mut fresh,&p,7);
        assert_eq!(fresh.judgements.violation_count("I-1.2"),1,"control detects mismatch");
        println!("AUDIT I-1.2 cached={} fresh=1",w.judgements.violation_count("I-1.2"));
    }
    #[test]
    fn audit_cached_node_skips_expected_tree(){
        let (r,p)=fixture();
        let mut w=Walk::starting_with(&r,Judgements::default(),0,0,10);
        read(&mut w,&p,7); w.begin_walking_a_version(); read(&mut w,&p,8);
        assert_eq!(w.judgements.violation_count("I-1.3"), if std::env::var_os("AUDIT_RECHECK_CONTROL").is_some(){1}else{0});
        let mut fresh=Walk::starting_with(&r,Judgements::default(),0,0,10);
        read(&mut fresh,&p,8);
        assert_eq!(fresh.judgements.violation_count("I-1.3"),1);
        println!("AUDIT I-1.3 cached={} fresh=1",w.judgements.violation_count("I-1.3"));
    }
}
