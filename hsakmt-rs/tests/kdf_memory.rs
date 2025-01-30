use hsakmt_rs::test_kfd_utils::kfd_base_component::KFDBaseComponentTest;
use hsakmt_rs::topology::{hsakmt_topology_global_g_props_get_ref, hsakmt_topology_global_get};

#[test]
fn test_a() {
    let mut kfd_base = KFDBaseComponentTest::new();

    unsafe {
        kfd_base.set_up();
    }

    println!("{:#?}", kfd_base);
    let topology_global = hsakmt_topology_global_get();
    println!("{:#?}", topology_global);

    hsakmt_topology_global_g_props_get_ref();
}
