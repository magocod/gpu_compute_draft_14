#![allow(non_camel_case_types, dead_code, non_snake_case)]

use crate::fmm::{hsakmt_fmm_init_process_apertures, hsakmt_open_drm_render_device};
use crate::globals::{check_kfd_open_and_panic, hsakmt_global_get};
use crate::hsakmttypes::HsakmtStatus::{
    HSAKMT_STATUS_ERROR, HSAKMT_STATUS_INVALID_NODE_UNIT, HSAKMT_STATUS_INVALID_PARAMETER,
    HSAKMT_STATUS_NOT_SUPPORTED, HSAKMT_STATUS_NO_MEMORY, HSAKMT_STATUS_SUCCESS,
};
use crate::hsakmttypes::HSA_HEAPTYPE::HSA_HEAPTYPE_FRAME_BUFFER_PUBLIC;
use crate::hsakmttypes::HSA_IOLINKTYPE::{
    HSA_IOLINKTYPE_PCIEXPRESS, HSA_IOLINKTYPE_UNDEFINED, HSA_IOLINK_TYPE_QPI_1_1,
};
use crate::hsakmttypes::{
    get_hsa_gfxip_table, hsa_gfxip_table, node_props_t, HsaCacheProperties, HsaIoLinkProperties,
    HsaMemoryProperties, HsaNodeProperties, HsaSystemProperties, HsakmtStatus, HSA_CPU_SIBLINGS,
    HSA_GET_GFX_VERSION_FULL, HSA_IOLINKTYPE, SGPR_SIZE_PER_CU,
};
use crate::queues::hsakmt_get_vgpr_size_per_cu;
use amdgpu_drm_sys::bindings::{
    amdgpu_device_deinitialize, amdgpu_device_handle, amdgpu_device_initialize,
    amdgpu_get_marketing_name, amdgpu_gpu_info, amdgpu_query_gpu_info, AMDGPU_IDS_FLAGS_FUSION,
};
use libc::{c_long, strlen, strtok, strtol, ENOENT, EPERM};
use std::ffi::{CStr, CString};
use std::fs;
use std::mem::MaybeUninit;
use std::path::Path;
use std::sync::Mutex;
use std::thread::available_parallelism;
use xf86drm_sys::bindings::{drmClose, drmOpenRender};

pub const KFD_SYSFS_PATH_GENERATION_ID: &str =
    "/sys/devices/virtual/kfd/kfd/topology/generation_id";

pub const KFD_SYSFS_PATH_SYSTEM_PROPERTIES: &str =
    "/sys/devices/virtual/kfd/kfd/topology/system_properties";
pub const KFD_SYSFS_PATH_NODES: &str = "/sys/devices/virtual/kfd/kfd/topology/nodes";

/* Supported System Vendors */
pub enum SUPPORTED_PROCESSOR_VENDORS {
    GENUINE_INTEL = 0,
    AUTHENTIC_AMD,
    IBM_POWER,
}

pub const PROCESSOR_VENDOR: SUPPORTED_PROCESSOR_VENDORS =
    SUPPORTED_PROCESSOR_VENDORS::AUTHENTIC_AMD;

// MOVE TO TYPES
// #[derive(Debug, PartialEq)]
// pub struct node_props_t {
//     node: HsaNodeProperties,
//     mem: HsaMemoryProperties,     /* node->NumBanks elements */
//     cache: HsaCacheProperties,
//     link: HsaIoLinkProperties,
// }

#[derive(Debug, PartialEq)]
pub struct HsaKmtTopologyGlobal {
    pub g_system: HsaSystemProperties,
    pub g_props: Vec<node_props_t>,
    /* This array caches sysfs based node IDs of CPU nodes + all supported GPU nodes.
     * It will be used to map user-node IDs to sysfs-node IDs.
     */
    pub map_user_to_sysfs_node_id: Vec<usize>,
    pub map_user_to_sysfs_node_id_size: usize,
    pub num_sysfs_nodes: usize,
}

impl Clone for HsaKmtTopologyGlobal {
    fn clone(&self) -> Self {
        Self {
            g_system: self.g_system,
            g_props: vec![],
            map_user_to_sysfs_node_id: self.map_user_to_sysfs_node_id.clone(),
            map_user_to_sysfs_node_id_size: self.map_user_to_sysfs_node_id_size,
            num_sysfs_nodes: self.num_sysfs_nodes,
        }
    }
}

static HSA_KMT_TOPOLOGY_GLOBAL: Mutex<HsaKmtTopologyGlobal> = Mutex::new(HsaKmtTopologyGlobal {
    g_system: HsaSystemProperties {
        NumNodes: 0,
        PlatformOem: 0,
        PlatformId: 0,
        PlatformRev: 0,
    },
    g_props: vec![],
    map_user_to_sysfs_node_id: vec![],
    map_user_to_sysfs_node_id_size: 0,
    num_sysfs_nodes: 0,
});

pub fn hsakmt_topology_global_get() -> HsaKmtTopologyGlobal {
    HSA_KMT_TOPOLOGY_GLOBAL.lock().unwrap().clone()
}

pub fn hsakmt_topology_global_map_user_to_sysfs_node_id_set(ids: Vec<usize>) {
    let mut g = HSA_KMT_TOPOLOGY_GLOBAL.lock().unwrap();

    g.map_user_to_sysfs_node_id_size = ids.len();
    g.map_user_to_sysfs_node_id = ids;
}

pub fn hsakmt_topology_global_num_sysfs_nodes_set(num_sysfs_nodes: usize) {
    HSA_KMT_TOPOLOGY_GLOBAL.lock().unwrap().num_sysfs_nodes = num_sysfs_nodes;
}

pub fn hsakmt_topology_global_g_system_set(g_system: HsaSystemProperties) {
    HSA_KMT_TOPOLOGY_GLOBAL.lock().unwrap().g_system = g_system;
}

pub fn hsakmt_topology_global_g_props_set(g_props: Vec<node_props_t>) {
    HSA_KMT_TOPOLOGY_GLOBAL.lock().unwrap().g_props = g_props;
}

pub fn hsakmt_topology_global_g_props_get_ref() {
    let g = HSA_KMT_TOPOLOGY_GLOBAL.lock().unwrap();
    println!("{:?}", g.g_props.len());

    // println!("g.g_props[0].cache.len(): {:?}", g.g_props[0].cache.len());
}

#[derive(Debug, PartialEq)]
pub struct KfdTopologyNodeProperties {
    cpu_cores_count: Option<usize>,
    simd_count: Option<usize>,
    mem_banks_count: Option<usize>,
    caches_count: Option<usize>,
    io_links_count: Option<usize>,
    p2p_links_count: Option<usize>,
    cpu_core_id_base: Option<usize>,
    simd_id_base: Option<usize>,
    capability: Option<usize>,
    debug_prop: Option<usize>,
    max_waves_per_simd: Option<usize>,
    lds_size_in_kb: Option<usize>,
    gds_size_in_kb: Option<usize>,
    wave_front_size: Option<usize>,
    array_count: Option<usize>,
    simd_arrays_per_engine: Option<usize>,
    cu_per_simd_array: Option<usize>,
    simd_per_cu: Option<usize>,
    max_slots_scratch_cu: Option<usize>,
    fw_version: Option<usize>,
    vendor_id: Option<usize>,
    device_id: Option<usize>,
    location_id: Option<usize>,
    domain: Option<usize>,
    max_engine_clk_fcompute: Option<usize>,
    max_engine_clk_ccompute: Option<usize>,
    local_mem_size: Option<usize>,
    drm_render_minor: Option<usize>,
    sdma_fw_version: Option<usize>,
    hive_id: Option<usize>,
    unique_id: Option<usize>,
    num_sdma_engines: Option<usize>,
    num_sdma_xgmi_engines: Option<usize>,
    num_gws: Option<usize>,
    num_sdma_queues_per_engine: Option<usize>,
    num_cp_queues: Option<usize>,
    num_xcc: Option<usize>,
    gfx_target_version: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub struct KfdTopologyNode {
    node_id: usize,
    gpu_id: usize,
    properties: KfdTopologyNodeProperties,
}

#[derive(Debug, PartialEq)]
pub struct SysDevicesVirtualKfd {
    platform_oem: u64,
    platform_id: u64,
    platform_rev: u64,
    nodes: Vec<KfdTopologyNode>,
}

/* Expects gfxv (full) in decimal */

pub fn HSA_GET_GFX_VERSION_MAJOR(gfxv: u16) -> u8 {
    (((gfxv) / 10000) % 100) as u8
}

pub fn HSA_GET_GFX_VERSION_MINOR(gfxv: u16) -> u8 {
    (((gfxv) / 100) % 100) as u8
}

pub fn HSA_GET_GFX_VERSION_STEP(gfxv: u16) -> u8 {
    (((gfxv) / 100) % 100) as u8
}

#[allow(clippy::manual_find)]
pub fn find_hsa_gfxip_device(device_id: u16, gfxv_major: u8) -> Option<hsa_gfxip_table> {
    if gfxv_major > 10 {
        return None;
    }

    let gfxip_lookup_table = get_hsa_gfxip_table();

    // let table_size = (std::mem::size_of_val(&gfxip_lookup_table)
    //     / std::mem::size_of::<hsa_gfxip_table>()) as u32;

    for dev in gfxip_lookup_table {
        if dev.device_id == device_id {
            return Some(dev);
        }
    }

    None
}

pub unsafe fn topology_get_node_props_from_drm(props: &mut HsaNodeProperties) -> i32 {
    let mut gpu_info = amdgpu_gpu_info {
        asic_id: 0,
        chip_rev: 0,
        chip_external_rev: 0,
        family_id: 0,
        ids_flags: 0,
        max_engine_clk: 0,
        max_memory_clk: 0,
        num_shader_engines: 0,
        num_shader_arrays_per_engine: 0,
        avail_quad_shader_pipes: 0,
        max_quad_shader_pipes: 0,
        cache_entries_per_quad_pipe: 0,
        num_hw_gfx_contexts: 0,
        rb_pipes: 0,
        enabled_rb_pipes_mask: 0,
        gpu_counter_freq: 0,
        backend_disable: [0; 4],
        mc_arb_ramcfg: 0,
        gb_addr_cfg: 0,
        gb_tile_mode: [0; 32],
        gb_macro_tile_mode: [0; 16],
        pa_sc_raster_cfg: [0; 4],
        pa_sc_raster_cfg1: [0; 4],
        cu_active_number: 0,
        cu_ao_mask: 0,
        cu_bitmap: [[0; 4]; 4],
        vram_type: 0,
        vram_bit_width: 0,
        ce_ram_size: 0,
        vce_harvest_config: 0,
        pci_rev_id: 0,
    };

    // const char *name;
    let mut ret = 0;

    let drm_fd = drmOpenRender(props.DrmRenderMinor);

    if drm_fd < 0 {
        return -1;
    }

    let mut device_handle: MaybeUninit<amdgpu_device_handle> = MaybeUninit::uninit();
    let mut major_version: MaybeUninit<u32> = MaybeUninit::zeroed();
    let mut minor_version: MaybeUninit<u32> = MaybeUninit::zeroed();

    if amdgpu_device_initialize(
        drm_fd,
        major_version.as_mut_ptr(),
        minor_version.as_mut_ptr(),
        device_handle.as_mut_ptr(),
    ) < 0
    {
        ret = -1;
        let d_h = device_handle.assume_init();
        amdgpu_device_deinitialize(d_h);
        drmClose(drm_fd);
    }

    let device_handle = device_handle.assume_init();

    let name = amdgpu_get_marketing_name(device_handle);
    if !name.is_null() {
        let _cs = CStr::from_ptr(name);
        // println!("MarketingName {:?}", cs.to_string_lossy().to_string());
        // props.MarketingName = cs
    }

    if amdgpu_query_gpu_info(device_handle, &mut gpu_info) != 0 {
        ret = -1;
        amdgpu_device_deinitialize(device_handle);
    }

    props.FamilyID = gpu_info.family_id;
    props.Integrated = !!(gpu_info.ids_flags & AMDGPU_IDS_FLAGS_FUSION as u64) as u8;

    ret
}

impl SysDevicesVirtualKfd {
    pub fn new() -> Self {
        let mut instance = Self {
            platform_oem: 0,
            platform_id: 0,
            platform_rev: 0,
            nodes: vec![],
        };

        let base_dir = Path::new(KFD_SYSFS_PATH_SYSTEM_PROPERTIES);
        let content = fs::read_to_string(base_dir).unwrap();
        let properties = content.split("\n").collect::<Vec<&str>>();

        for property in properties {
            let pair = property.split(" ").collect::<Vec<&str>>();

            if pair.len() != 2 {
                continue;
            }

            if pair[0] == "platform_oem" {
                instance.platform_oem = pair[1].trim().parse::<u64>().unwrap();
            } else if pair[0] == "platform_id" {
                instance.platform_id = pair[1].trim().parse::<u64>().unwrap();
            } else if pair[0] == "platform_rev" {
                instance.platform_rev = pair[1].trim().parse::<u64>().unwrap();
            }
        }

        instance
    }

    pub fn get_nodes(&self) -> &Vec<KfdTopologyNode> {
        &self.nodes
    }

    pub fn load_nodes(&mut self) {
        let base_dir = Path::new(KFD_SYSFS_PATH_NODES);

        if base_dir.is_dir() {
            for entry in fs::read_dir(base_dir).unwrap() {
                let node_entry_dir = entry.unwrap();
                // println!("{:?}", node_entry_dir);

                let node_id = node_entry_dir
                    .file_name()
                    .to_string_lossy()
                    .to_string()
                    .parse::<usize>()
                    .unwrap();

                let mut kfd_topology_node = KfdTopologyNode {
                    node_id,
                    gpu_id: 0,
                    properties: KfdTopologyNodeProperties {
                        cpu_cores_count: None,
                        simd_count: None,
                        mem_banks_count: None,
                        caches_count: None,
                        io_links_count: None,
                        p2p_links_count: None,
                        cpu_core_id_base: None,
                        simd_id_base: None,
                        capability: None,
                        debug_prop: None,
                        max_waves_per_simd: None,
                        lds_size_in_kb: None,
                        gds_size_in_kb: None,
                        wave_front_size: None,
                        array_count: None,
                        simd_arrays_per_engine: None,
                        cu_per_simd_array: None,
                        simd_per_cu: None,
                        max_slots_scratch_cu: None,
                        fw_version: None,
                        vendor_id: None,
                        device_id: None,
                        location_id: None,
                        domain: None,
                        max_engine_clk_fcompute: None,
                        max_engine_clk_ccompute: None,
                        local_mem_size: None,
                        drm_render_minor: None,
                        sdma_fw_version: None,
                        hive_id: None,
                        unique_id: None,
                        num_sdma_engines: None,
                        num_sdma_xgmi_engines: None,
                        num_gws: None,
                        num_sdma_queues_per_engine: None,
                        num_cp_queues: None,
                        num_xcc: None,
                        gfx_target_version: None,
                    },
                };

                if node_entry_dir.path().is_dir() {
                    for sub_entry in fs::read_dir(node_entry_dir.path()).unwrap() {
                        let node_entry = sub_entry.unwrap();

                        if node_entry.file_name() == "gpu_id" {
                            let gpu_id_str = fs::read_to_string(node_entry.path()).unwrap();
                            kfd_topology_node.gpu_id = gpu_id_str.trim().parse::<usize>().unwrap();
                        }

                        if node_entry.file_name() == "properties" {
                            let content = fs::read_to_string(node_entry.path()).unwrap();
                            let properties = content.split("\n").collect::<Vec<&str>>();

                            for property in properties {
                                let pair = property.split(" ").collect::<Vec<&str>>();

                                if pair.len() != 2 {
                                    continue;
                                }

                                if pair[0] == "cpu_cores_count" {
                                    kfd_topology_node.properties.cpu_cores_count =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "simd_count" {
                                    kfd_topology_node.properties.simd_count =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "mem_banks_count" {
                                    kfd_topology_node.properties.mem_banks_count =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "caches_count" {
                                    kfd_topology_node.properties.caches_count =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "io_links_count" {
                                    kfd_topology_node.properties.io_links_count =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "p2p_links_count" {
                                    kfd_topology_node.properties.p2p_links_count =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "cpu_core_id_base" {
                                    kfd_topology_node.properties.cpu_core_id_base =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "simd_id_base" {
                                    kfd_topology_node.properties.simd_id_base =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "capability" {
                                    kfd_topology_node.properties.capability =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "debug_prop" {
                                    kfd_topology_node.properties.debug_prop =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "max_waves_per_simd" {
                                    kfd_topology_node.properties.max_waves_per_simd =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "lds_size_in_kb" {
                                    kfd_topology_node.properties.lds_size_in_kb =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "gds_size_in_kb" {
                                    kfd_topology_node.properties.gds_size_in_kb =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "wave_front_size" {
                                    kfd_topology_node.properties.wave_front_size =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "array_count" {
                                    kfd_topology_node.properties.array_count =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "simd_arrays_per_engine" {
                                    kfd_topology_node.properties.simd_arrays_per_engine =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "cu_per_simd_array" {
                                    kfd_topology_node.properties.cu_per_simd_array =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "simd_per_cu" {
                                    kfd_topology_node.properties.simd_per_cu =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "max_slots_scratch_cu" {
                                    kfd_topology_node.properties.max_slots_scratch_cu =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "fw_version" {
                                    kfd_topology_node.properties.fw_version =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "vendor_id" {
                                    kfd_topology_node.properties.vendor_id =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "device_id" {
                                    kfd_topology_node.properties.device_id =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "location_id" {
                                    kfd_topology_node.properties.location_id =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "domain" {
                                    kfd_topology_node.properties.domain =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "max_engine_clk_fcompute" {
                                    kfd_topology_node.properties.max_engine_clk_fcompute =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "max_engine_clk_ccompute" {
                                    kfd_topology_node.properties.max_engine_clk_ccompute =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "local_mem_size" {
                                    kfd_topology_node.properties.local_mem_size =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "drm_render_minor" {
                                    kfd_topology_node.properties.drm_render_minor =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "sdma_fw_version" {
                                    kfd_topology_node.properties.sdma_fw_version =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "hive_id" {
                                    kfd_topology_node.properties.hive_id =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "unique_id" {
                                    kfd_topology_node.properties.unique_id =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "num_sdma_engines" {
                                    kfd_topology_node.properties.num_sdma_engines =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "num_sdma_xgmi_engines" {
                                    kfd_topology_node.properties.num_sdma_xgmi_engines =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "num_gws" {
                                    kfd_topology_node.properties.num_gws =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "num_sdma_queues_per_engine" {
                                    kfd_topology_node.properties.num_sdma_queues_per_engine =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "num_cp_queues" {
                                    kfd_topology_node.properties.num_cp_queues =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "num_xcc" {
                                    kfd_topology_node.properties.num_xcc =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                } else if pair[0] == "gfx_target_version" {
                                    kfd_topology_node.properties.gfx_target_version =
                                        Some(pair[1].trim().parse::<usize>().unwrap());
                                }
                            }
                        }
                    }
                }

                self.nodes.push(kfd_topology_node);
            }
        }
    }

    /* Check if the @sysfs_node_id is supported. This function will be passed with sysfs node id.
     * This function can not use topology_* help functions, because those functions are
     * using user node id.
     * A sysfs node is not supported
     *	- if corresponding drm render node is not available.
     *	- if node information is not accessible (EPERM)
     */
    pub fn topology_sysfs_check_node_supported(&self, sysfs_node_id: usize) -> bool {
        let node = self
            .nodes
            .iter()
            .find(|x| x.node_id == sysfs_node_id)
            .unwrap();

        /* Retrieve the GPU ID */
        if node.gpu_id == 0 {
            return true;
        }

        /* Retrieve the node properties */

        /* Open DRM Render device */
        let ret_value = unsafe {
            hsakmt_open_drm_render_device(node.properties.drm_render_minor.unwrap() as i32)
        };

        if ret_value > 0 {
            return true;
        } else if ret_value != -ENOENT && ret_value != -EPERM {
            // ret = HSAKMT_STATUS_ERROR;
        }

        false
    }

    #[allow(clippy::identity_op)]
    pub unsafe fn topology_sysfs_get_node_props(
        &self,
        node_id: u32,
        props: &mut HsaNodeProperties,
        p2p_links: &mut bool,
        num_p2pLinks: &mut u32,
    ) -> HsakmtStatus {
        let node = self
            .nodes
            .iter()
            .find(|x| x.node_id == node_id as usize)
            .unwrap();

        let mut simd_arrays_count = 0;
        let mut gfxv = 0;

        /* Retrieve the GPU ID */
        props.KFDGpuID = node.gpu_id as u32;

        /* Retrieve the node properties */

        if let Some(v) = node.properties.cpu_cores_count {
            props.NumCPUCores = v as u32;
        }

        if let Some(v) = node.properties.simd_count {
            props.NumFComputeCores = v as u32;
        }

        if let Some(v) = node.properties.mem_banks_count {
            props.NumMemoryBanks = v as u32;
        }

        if let Some(v) = node.properties.caches_count {
            props.NumCaches = v as u32;
        }

        if let Some(v) = node.properties.io_links_count {
            props.NumIOLinks = v as u32;
        }

        if let Some(v) = node.properties.p2p_links_count {
            props.NumIOLinks += v as u32;

            *num_p2pLinks = v as u32;
            *p2p_links = true;
        }

        if let Some(v) = node.properties.cpu_core_id_base {
            props.CComputeIdLo = v as u32;
        }

        if let Some(v) = node.properties.simd_id_base {
            props.FComputeIdLo = v as u32;
        }

        if let Some(v) = node.properties.capability {
            props.Capability.Value = v as u32;
        }

        if let Some(v) = node.properties.debug_prop {
            props.DebugProperties.Value = v as u64;
        }

        if let Some(v) = node.properties.max_waves_per_simd {
            props.MaxWavesPerSIMD = v as u32;
        }

        if let Some(v) = node.properties.lds_size_in_kb {
            props.LDSSizeInKB = v as u32;
        }

        if let Some(v) = node.properties.gds_size_in_kb {
            props.GDSSizeInKB = v as u32;
        }

        if let Some(v) = node.properties.wave_front_size {
            props.WaveFrontSize = v as u32;
        }

        if let Some(v) = node.properties.array_count {
            simd_arrays_count = v as u32;
        }

        if let Some(v) = node.properties.simd_arrays_per_engine {
            props.NumArrays = v as u32;
        }

        if let Some(v) = node.properties.cu_per_simd_array {
            props.NumCUPerArray = v as u32;
        }

        if let Some(v) = node.properties.simd_per_cu {
            props.NumSIMDPerCU = v as u32;
        }

        if let Some(v) = node.properties.max_slots_scratch_cu {
            props.MaxSlotsScratchCU = v as u32;
        }

        if let Some(v) = node.properties.fw_version {
            props.EngineId.Value = (v as u32) & 0x3ff;
        }

        if let Some(v) = node.properties.vendor_id {
            props.VendorId = v as u16;
        }

        if let Some(v) = node.properties.device_id {
            props.DeviceId = v as u16;
        }

        if let Some(v) = node.properties.device_id {
            props.LocationId = v as u32;
        }

        if let Some(v) = node.properties.domain {
            props.Domain = v as u32;
        }

        if let Some(v) = node.properties.max_engine_clk_fcompute {
            props.MaxEngineClockMhzFCompute = v as u32;
        }

        if let Some(v) = node.properties.max_engine_clk_ccompute {
            props.MaxEngineClockMhzCCompute = v as u32;
        }

        if let Some(v) = node.properties.local_mem_size {
            props.LocalMemSize = v as u64;
        }

        if let Some(v) = node.properties.drm_render_minor {
            props.DrmRenderMinor = v as i32;
        }

        if let Some(v) = node.properties.sdma_fw_version {
            props.uCodeEngineVersions.Value = v as u32;
        }

        if let Some(v) = node.properties.hive_id {
            props.HiveID = v as u64;
        }

        if let Some(v) = node.properties.unique_id {
            props.UniqueID = v as u64;
        }

        if let Some(v) = node.properties.num_sdma_engines {
            props.NumSdmaEngines = v as u32;
        }

        if let Some(v) = node.properties.num_sdma_xgmi_engines {
            props.NumSdmaXgmiEngines = v as u32;
        }

        if let Some(v) = node.properties.num_gws {
            props.NumGws = v as u8;
        }

        if let Some(v) = node.properties.num_sdma_queues_per_engine {
            props.NumSdmaQueuesPerEngine = v as u8;
        }

        if let Some(v) = node.properties.num_cp_queues {
            props.NumCpQueues = v as u8;
        }

        if let Some(v) = node.properties.num_xcc {
            props.NumXcc = v as u32;
        }

        if let Some(v) = node.properties.gfx_target_version {
            gfxv = v as u32;
        }

        let global = hsakmt_global_get();

        if !global.hsakmt_is_svm_api_supported {
            props.Capability.ui32.SVMAPISupported = 0;
        }

        /* Bail out early, if a CPU node */
        if props.NumFComputeCores == 0 {
            return HSAKMT_STATUS_SUCCESS;
        }

        if props.NumArrays != 0 {
            props.NumShaderBanks = simd_arrays_count / props.NumArrays;
        }

        let gfxv_major = HSA_GET_GFX_VERSION_MAJOR(gfxv as u16);
        let gfxv_minor = HSA_GET_GFX_VERSION_MINOR(gfxv as u16);
        let gfxv_stepping = HSA_GET_GFX_VERSION_STEP(gfxv as u16);

        let hsa_gfxip = find_hsa_gfxip_device(props.DeviceId, gfxv_major);

        if hsa_gfxip.is_some() || gfxv > 0 {
            // snprintf(per_node_override, sizeof(per_node_override), "HSA_OVERRIDE_GFX_VERSION_%d", node_id);
            // if ((envvar = getenv(per_node_override)) || (envvar = getenv("HSA_OVERRIDE_GFX_VERSION"))) {
            //     /* HSA_OVERRIDE_GFX_VERSION=major.minor.stepping */
            //     if ((sscanf(envvar, "%u.%u.%u%c",
            //                 &major, &minor, &step, &dummy) != 3) ||
            //         (major > 63 || minor > 255 || step > 255)) {
            //         pr_err("HSA_OVERRIDE_GFX_VERSION %s is invalid\n",
            //                envvar);
            //         ret = HSAKMT_STATUS_ERROR;
            //         goto out;
            //     }
            //     props->OverrideEngineId.ui32.Major = major & 0x3f;
            //     props->OverrideEngineId.ui32.Minor = minor & 0xff;
            //     props->OverrideEngineId.ui32.Stepping = step & 0xff;
            // }

            if hsa_gfxip.is_some() {
                let hsa_gfxip_table = hsa_gfxip.unwrap();

                props.EngineId.ui32.Major = (hsa_gfxip_table.major & 0x3f) as u32;
                props.EngineId.ui32.Minor = (hsa_gfxip_table.minor & 0xff) as u32;
                props.EngineId.ui32.Stepping = (hsa_gfxip_table.stepping & 0xff) as u32;
            } else {
                props.EngineId.ui32.Major = (gfxv_major & 0x3f) as u32;
                props.EngineId.ui32.Minor = (gfxv_minor & 0xff) as u32;
                props.EngineId.ui32.Stepping = (gfxv_stepping & 0xff) as u32;
            }

            /* Set the CAL name of the node. If DID-based hsa_gfxip lookup was
             * successful, use that name. Otherwise, set to GFX<GFX_VERSION>.
             */
            // if (hsa_gfxip && hsa_gfxip->amd_name)
            // strncpy((char *)props->AMDName, hsa_gfxip->amd_name,
            //         sizeof(props->AMDName)-1);
            // else
            // snprintf((char *)props->AMDName, sizeof(props->AMDName)-1, "GFX%06x",
            //          HSA_GET_GFX_VERSION_FULL(props->EngineId.ui32));

            /* Is dGPU Node, not APU
             * Retrieve the marketing name of the node.
             */
            if topology_get_node_props_from_drm(props) != 0 {
                println!(
                    "failed to get marketing name for device ID {}",
                    props.DeviceId
                );
            }

            /* Get VGPR/SGPR size in byte per CU */
            props.SGPRSizePerCU = SGPR_SIZE_PER_CU as u32;
            props.VGPRSizePerCU =
                hsakmt_get_vgpr_size_per_cu(HSA_GET_GFX_VERSION_FULL(&props.EngineId.ui32));
        } else if props.DeviceId == 0 {
            /* still return success */
            println!("device ID {} is not supported in libhsakmt", props.DeviceId);
        }

        // if (props->NumFComputeCores)
        // assert(props->EngineId.ui32.Major && "HSA_OVERRIDE_GFX_VERSION may be needed");
        //
        // if props.NumFComputeCores > 0 {
        //     assert_eq!(props.EngineId.ui32.Major, 0, "HSA_OVERRIDE_GFX_VERSION may be needed");
        // }

        /* On Older kernels, num_xcc may not be present in system properties.
         * Set it to 1 if system properties do not report num_xcc.
         */
        if props.NumXcc == 0 {
            props.NumXcc = 1;
        }

        HSAKMT_STATUS_SUCCESS
    }

    pub unsafe fn topology_sysfs_get_mem_props(
        &self,
        node_id: u32,
        mem_id: u32,
        props: &mut HsaMemoryProperties,
    ) -> HsakmtStatus {
        let node = self
            .nodes
            .iter()
            .find(|x| x.node_id == node_id as usize)
            .unwrap();

        let mem_banks_path = format!(
            "{}/{}/mem_banks/{}/properties",
            KFD_SYSFS_PATH_NODES, node.node_id, mem_id
        );

        let content = fs::read_to_string(mem_banks_path).unwrap();

        let lines = content.split("\n").collect::<Vec<&str>>();

        for line in lines {
            let pair = line.split(" ").collect::<Vec<&str>>();

            if pair.len() != 2 {
                continue;
            }

            if pair[0] == "heap_type" {
                let v = pair[1].trim().parse::<usize>().unwrap();
                props.HeapType = v.try_into().unwrap();
            } else if pair[0] == "size_in_bytes" {
                props.prop.SizeInBytes = pair[1].trim().parse::<u64>().unwrap();
            } else if pair[0] == "flags" {
                props.Flags.MemoryProperty = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "width" {
                props.Width = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "mem_clk_max" {
                props.MemoryClockMax = pair[1].trim().parse::<u32>().unwrap();
            }
        }

        HSAKMT_STATUS_SUCCESS
    }

    pub unsafe fn topology_sysfs_get_cache_props(
        &self,
        node_id: u32,
        cache_id: u32,
        props: &mut HsaCacheProperties,
    ) -> HsakmtStatus {
        let node = self
            .nodes
            .iter()
            .find(|x| x.node_id == node_id as usize)
            .unwrap();

        let caches_path = format!(
            "{}/{}/caches/{}/properties",
            KFD_SYSFS_PATH_NODES, node.node_id, cache_id
        );
        let content = fs::read_to_string(caches_path).unwrap();

        let lines = content.split("\n").collect::<Vec<&str>>();

        for line in lines {
            let pair = line.split(" ").collect::<Vec<&str>>();

            if pair.len() != 2 {
                continue;
            }

            if pair[0] == "processor_id_low" {
                props.ProcessorIdLow = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "level" {
                props.CacheLevel = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "size" {
                props.CacheSize = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "cache_line_size" {
                props.CacheLineSize = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "cache_lines_per_tag" {
                props.CacheLinesPerTag = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "association" {
                props.CacheAssociativity = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "latency" {
                props.CacheLatency = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "type" {
                props.CacheType.Value = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "sibling_map" {
                let values = pair[1].trim().split(",").collect::<Vec<&str>>();

                for (i, value) in values.iter().enumerate() {
                    props.SiblingMap[i] = value.parse::<u32>().unwrap();
                }
            }
        }

        HSAKMT_STATUS_SUCCESS
    }

    /* For a give Node @node_id the function gets @iolink_id information i.e. parses sysfs the following sysfs entry
     * ./nodes/@node_id/io_links/@iolink_id/properties. @node_id has to be valid accessible node.
     *
     * If node_to specified by the @iolink_id is not accessible the function returns HSAKMT_STATUS_NOT_SUPPORTED.
     * If node_to is accessible, then node_to is mapped from sysfs_node to user_node and returns HSAKMT_STATUS_SUCCESS.
     */
    pub unsafe fn topology_sysfs_get_iolink_props(
        &self,
        node_id: u32,
        iolink_id: u32,
        props: &mut HsaIoLinkProperties,
        p2pLink: bool,
    ) -> HsakmtStatus {
        let node = self
            .nodes
            .iter()
            .find(|x| x.node_id == node_id as usize)
            .unwrap();

        let sys_node_id = node.node_id;

        let link_path = if p2pLink {
            format!(
                "{}/{}/p2p_links/{}/properties",
                KFD_SYSFS_PATH_NODES, sys_node_id, iolink_id
            )
        } else {
            format!(
                "{}/{}/io_links/{}/properties",
                KFD_SYSFS_PATH_NODES, sys_node_id, iolink_id
            )
        };

        let content = fs::read_to_string(link_path);

        // FIXME topology_sysfs_get_iolink_props
        if content.is_err() {
            return HSAKMT_STATUS_NOT_SUPPORTED;
        }

        let content = content.unwrap();

        let lines = content.split("\n").collect::<Vec<&str>>();

        for line in lines {
            let pair = line.split(" ").collect::<Vec<&str>>();

            if pair.len() != 2 {
                continue;
            }

            if pair[0] == "type" {
                let v = pair[1].trim().parse::<usize>().unwrap();
                props.IoLinkType = v.try_into().unwrap();
            } else if pair[0] == "version_major" {
                props.VersionMajor = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "version_minor" {
                props.VersionMinor = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "node_from" {
                let v = pair[1].trim().parse::<usize>().unwrap();

                if sys_node_id != v {
                    return HSAKMT_STATUS_INVALID_NODE_UNIT;
                }

                props.NodeFrom = node_id;
            } else if pair[0] == "node_to" {
                let v = pair[1].trim().parse::<usize>().unwrap();

                let is_node_supported = self.topology_sysfs_check_node_supported(v);
                if !is_node_supported {
                    return HSAKMT_STATUS_NOT_SUPPORTED;
                }

                props.NodeTo = node_id;
            } else if pair[0] == "weight" {
                props.Weight = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "min_latency" {
                props.MinimumLatency = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "max_latency" {
                props.MaximumLatency = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "min_bandwidth" {
                props.MinimumBandwidth = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "max_bandwidth" {
                props.MaximumBandwidth = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "recommended_transfer_size" {
                props.RecTransferSize = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "recommended_sdma_engine_id_mask" {
                props.RecSdmaEngIdMask = pair[1].trim().parse::<u32>().unwrap();
            } else if pair[0] == "flags" {
                props.Flags.LinkProperty = pair[1].trim().parse::<u32>().unwrap();
            }
        }

        HSAKMT_STATUS_SUCCESS
    }
}

impl Default for SysDevicesVirtualKfd {
    fn default() -> Self {
        Self::new()
    }
}

/// ...
///
///
/// # Safety
///
/// TODO safety function explain
pub unsafe fn hsakmt_topology_sysfs_get_system_props(
    props: &mut HsaSystemProperties,
) -> HsakmtStatus {
    let mut kfd = SysDevicesVirtualKfd::new();
    kfd.load_nodes();

    props.PlatformOem = kfd.platform_oem as u32;
    props.PlatformId = kfd.platform_id as u32;
    props.PlatformRev = kfd.platform_rev as u32;

    /*
     * Discover the number of sysfs nodes:
     * Assuming that inside nodes folder there are only folders
     * which represent the node numbers
     */
    let num_sysfs_nodes = kfd.get_nodes().len();

    let mut ids = vec![];

    for i in 0..num_sysfs_nodes {
        let is_node_supported = kfd.topology_sysfs_check_node_supported(i);
        if is_node_supported {
            ids.push(i);
        }
    }

    props.NumNodes = ids.len() as u32;

    hsakmt_topology_global_map_user_to_sysfs_node_id_set(ids);
    hsakmt_topology_global_num_sysfs_nodes_set(num_sysfs_nodes);

    HSAKMT_STATUS_SUCCESS
}

/* information from /proc/cpuinfo */
#[derive(Debug, Clone)]
pub struct proc_cpuinfo {
    proc_num: u32,      /* processor */
    apicid: u32,        /* apicid */
    model_name: String, /* model name */
}

/* CPU cache table for all CPUs on the system. Each entry has the relative CPU
 * info and caches connected to that CPU.
 */
#[derive(Debug)]
pub struct cpu_cacheinfo_t {
    len: u32,                            /* length of the table = number of online procs */
    proc_num: i32,                       /* this cpu's processor number */
    num_caches: u32,                     /* number of caches reported by this cpu */
    cache_prop: Vec<HsaCacheProperties>, /* a list of cache properties */
}

pub unsafe fn topology_parse_cpuinfo() -> (Vec<proc_cpuinfo>, usize) {
    let proc_cpuinfo_path = "/proc/cpuinfo";

    // let num_procs = get_nprocs();
    let num_procs = available_parallelism().unwrap().get();
    let mut cpu_info = vec![
        proc_cpuinfo {
            proc_num: 0,
            apicid: 0,
            model_name: "".to_string(),
        };
        num_procs
    ];

    let content = fs::read_to_string(proc_cpuinfo_path).unwrap();
    let lines = content.split("\n").collect::<Vec<&str>>();

    let mut cpu_index: i32 = -1;

    for line in lines {
        let pair = line.split(":").collect::<Vec<&str>>();

        if pair.len() != 2 {
            continue;
        }

        if pair[0].trim() == "processor" {
            cpu_index += 1;
            cpu_info[cpu_index as usize].proc_num = cpu_index as u32;
            continue;
        }

        // model name
        if pair[0].trim() == "model name" {
            cpu_info[cpu_index as usize].model_name = pair[1].trim().to_string();
            continue;
        }

        // apicid
        if pair[0].trim() == "apicid" {
            cpu_info[cpu_index as usize].apicid = pair[1].trim().parse::<u32>().unwrap();
            continue;
        }
    }

    (cpu_info, num_procs)
}

pub unsafe fn topology_search_processor_vendor() {
    todo!()
}

pub unsafe fn topology_sysfs_get_generation(gen_start: &mut u32) -> HsakmtStatus {
    let content = fs::read_to_string(KFD_SYSFS_PATH_GENERATION_ID).unwrap();
    *gen_start = content.trim().parse::<u32>().unwrap();

    HSAKMT_STATUS_SUCCESS
}

pub fn num_subdirs(path: &str, text: &str) -> usize {
    let mut count = 0;

    for entry in fs::read_dir(path).unwrap() {
        let node_entry_dir = entry.unwrap();
        let file_name = node_entry_dir.file_name();

        if file_name.to_string_lossy().to_string().contains(text) {
            count += 1;
        }
    }

    count
}

/* cpumap_to_cpu_ci - translate shared_cpu_map string + cpuinfo->apicid into
 *		      SiblingMap in cache
 *	@shared_cpu_map [IN ] shared_cpu_map string
 *	@cpuinfo [IN ] cpuinfo to get apicid
 *	@this_cache [OUT] CPU cache to fill in SiblingMap
 */
#[allow(unused_assignments, clippy::strlen_on_c_strings)]
pub unsafe fn cpumap_to_cpu_ci(
    shared_cpu_map: &str,
    cpuinfo: &[proc_cpuinfo],
    this_cache: &mut HsaCacheProperties,
) {
    let shared_cpu_map = CString::new(shared_cpu_map.trim()).unwrap();

    /* shared_cpu_map is shown as ...X3,X2,X1 Each X is a hex without 0x
     * and it's up to 8 characters(32 bits). For the first 32 CPUs(actually
     * procs), it's presented in X1. The next 32 is in X2, and so on.
     */
    let mut num_hexs = (strlen(shared_cpu_map.as_ptr()) + 8) / 9; /* 8 characters + "," */

    let t_cs = CString::new(",").unwrap();
    let mut ch_ptr = strtok(shared_cpu_map.into_raw(), t_cs.as_ptr());

    let mut mask = 0;
    let mut proc = 0;
    let mut apicid = 0;

    // FIXME cpumap_to_cpu_ci SiblingMap
    while num_hexs > 0 {
        mask = strtol(ch_ptr, std::ptr::null_mut(), 16); /* each X */

        for bit in 0..32 {
            let v = !((1 << bit as c_long) & mask);

            if v < 0 {
                continue;
            }

            proc = num_hexs * 32 + bit;

            println!("v long {} {}", v, v > 0);
            println!("v proc {}", proc);

            apicid = cpuinfo[proc].apicid;

            if apicid >= HSA_CPU_SIBLINGS as u32 {
                println!("SiblingMap buffer %d is too small {}", HSA_CPU_SIBLINGS);
                continue;
            }

            this_cache.SiblingMap[apicid as usize] = 1;
        }

        ch_ptr = strtok(std::ptr::null_mut(), t_cs.as_ptr());

        num_hexs -= 1;
    }

    // while (num_hexs -= 1 > 0) {
    // 	mask = strtol(ch_ptr, NULL, 16); /* each X */
    // 	for (bit = 0; bit < 32; bit++) {
    // 		if (!((1 << bit) & mask))
    // 			continue;
    // 		proc = num_hexs * 32 + bit;
    // 		apicid = cpuinfo[proc].apicid;
    // 		if (apicid >= HSA_CPU_SIBLINGS) {
    // 			pr_warn("SiblingMap buffer %d is too small\n",
    // 				HSA_CPU_SIBLINGS);
    // 			continue;
    // 		}
    // 		this_cache->SiblingMap[apicid] = 1;
    // 	}
    // 	ch_ptr = strtok(NULL, ",");
    // }
}

/* get_cpu_cache_info - get specified CPU's cache information from sysfs
 *     @prefix [IN] sysfs path for target cpu cache,
 *                  /sys/devices/system/node/nodeX/cpuY/cache
 *     @cpuinfo [IN] /proc/cpuinfo data to get apicid
 *     @cpu_ci: CPU specified. This parameter is an input and also an output.
 *             [IN] cpu_ci->num_caches: number of index dirs
 *             [OUT] cpu_ci->cache_info: to store cache info collected
 *             [OUT] cpu_ci->num_caches: reduces when shared with other cpu(s)
 * Return: number of cache reported from this cpu
 */
pub unsafe fn get_cpu_cache_info(
    prefix: &str,
    cpuinfo: &[proc_cpuinfo],
    cpu_ci: &mut cpu_cacheinfo_t,
) -> i32 {
    // bool is_power9 = false;
    //
    // if (processor_vendor == IBM_POWER) {
    // 	if (strcmp(cpuinfo[0].model_name, "POWER9") == 0) {
    // 		is_power9 = true;
    // 	}
    // }

    let num_idx = cpu_ci.num_caches;

    for idx in 0..num_idx {
        let mut this_cache = HsaCacheProperties::default();

        /* If this cache is shared by multiple CPUs, we only need
         * to list it in the first CPU.
         */
        // if (is_power9) {
        //     // POWER9 has SMT4
        //     if (cpu_ci->proc_num & 0x3) {
        //         /* proc is not 0,4,8,etc.  Skip and reduce the cache count. */
        //         --cpu_ci->num_caches;
        //         continue;
        //     }
        // } else

        let shared_cpu_list_path = format!("{}/index{}/shared_cpu_list", prefix, idx);
        let content = fs::read_to_string(shared_cpu_list_path).unwrap();
        // println!("{}", content.trim());

        /* shared_cpu_list is shown as n1,n2... or n1-n2,n3-n4...
         * For both cases, this cache is listed to proc n1 only.
         */
        let elements = content.split(",").collect::<Vec<&str>>();

        let mut n = -1;

        if elements.len() == 2 {
            n = elements[0].trim().parse::<i32>().unwrap();
        } else if elements.len() == 1 {
            let elements_v2 = content.split("-").collect::<Vec<&str>>();
            n = elements_v2[0].trim().parse::<i32>().unwrap();
        }

        if cpu_ci.proc_num != n {
            /* proc is not n1. Skip and reduce the cache count. */
            cpu_ci.num_caches -= 1;
            continue;
        }

        this_cache.ProcessorIdLow = cpuinfo[cpu_ci.proc_num as usize].apicid;

        /* CacheLevel */
        let cache_level_path = format!("{}/index{}/level", prefix, idx);
        let content = fs::read_to_string(cache_level_path).unwrap();
        this_cache.CacheLevel = content.trim().parse::<u32>().unwrap();

        /* CacheType */
        let cache_type_path = format!("{}/index{}/type", prefix, idx);
        let content = fs::read_to_string(cache_type_path).unwrap();

        if content.trim() == "Data" {
            this_cache.CacheType.ui32.Data = 1;
        }

        if content.trim() == "Instruction" {
            this_cache.CacheType.ui32.Instruction = 1;
        }

        if content.trim() == "Unified" {
            this_cache.CacheType.ui32.Data = 1;
            this_cache.CacheType.ui32.Instruction = 1;
        }

        this_cache.CacheType.ui32.CPU = 1;

        /* CacheSize */
        let path = format!("{}/index{}/size", prefix, idx);
        let content = fs::read_to_string(path).unwrap();

        // FIXME cache size
        // If it does not end with K, this code will fail
        let mut content = content.trim().to_string();
        content.pop();
        this_cache.CacheSize = content.parse::<u32>().unwrap();

        /* CacheLineSize */
        let path = format!("{}/index{}/coherency_line_size", prefix, idx);
        let content = fs::read_to_string(path).unwrap();
        this_cache.CacheLineSize = content.trim().parse::<u32>().unwrap();

        /* CacheAssociativity */
        let path = format!("{}/index{}/ways_of_associativity", prefix, idx);
        let content = fs::read_to_string(path).unwrap();
        this_cache.CacheAssociativity = content.trim().parse::<u32>().unwrap();

        /* CacheLinesPerTag */
        let path = format!("{}/index{}/physical_line_partition", prefix, idx);
        let content = fs::read_to_string(path).unwrap();
        this_cache.CacheLinesPerTag = content.trim().parse::<u32>().unwrap();

        /* CacheSiblings */
        let path = format!("{}/index{}/shared_cpu_map", prefix, idx);
        let content = fs::read_to_string(path).unwrap();
        cpumap_to_cpu_ci(&content, cpuinfo, &mut this_cache);

        cpu_ci.cache_prop.push(this_cache);
    }

    cpu_ci.num_caches as i32
}

/* topology_create_temp_cpu_cache_list - Create a temporary cpu-cache list to
 *		store cpu cache information. This list will be used to copy
 *		HsaCacheProperties in the CPU node. Two buffers are allocated
 *		inside this function: cpu_ci list and cache_prop under each
 *		cpu_ci. Must call topology_destroy_temp_cpu_cache_list to free
 *		the memory after the information is copied.
 *	@node [IN] CPU node number
 *	@cpuinfo [IN] /proc/cpuinfo data
 *	@temp_cpu_ci_list [OUT] cpu-cache-info list with data filled
 * Return: total number of caches under this CPU node
 */
pub unsafe fn topology_create_temp_cpu_cache_list(
    node: i32,
    cpuinfo: &[proc_cpuinfo],
) -> (i32, Vec<cpu_cacheinfo_t>) {
    /* Get max path size from /sys/devices/system/node/node%d/%s/cache
     * below, which will max out according to the largest filename,
     * which can be present twice in the string above. 29 is for the prefix
     * and the +6 is for the cache suffix
     */
    let mut temp_cpu_ci_list: Vec<cpu_cacheinfo_t> = vec![];

    let mut cache_cnt = 0;

    /* Get info from /sys/devices/system/node/nodeX/cpuY/cache */
    let node_real = node;
    // if (processor_vendor == IBM_POWER) {
    // 	if (!strcmp(cpuinfo[0].model_name, "POWER9")) {
    // 		node_real = node * 8;
    // 	}
    // }

    let node_dir = format!("/sys/devices/system/node/node{}", node_real);

    /* Other than cpuY folders, this dir also has cpulist and cpumap */
    let _max_cpus = num_subdirs(&node_dir, "cpu");
    // 	if (max_cpus <= 0) {
    // 		/* If CONFIG_NUMA is not enabled in the kernel,
    // 		 * /sys/devices/system/node doesn't exist.
    // 		 */
    // 		if (node) { /* CPU node must be 0 or something is wrong */
    // 			pr_err("Fail to get cpu* dirs under %s.", node_dir);
    // 			goto exit;
    // 		}
    // 		/* Fall back to use /sys/devices/system/cpu */
    // 		snprintf(node_dir, MAXPATHSIZE, "/sys/devices/system/cpu");
    // 		max_cpus = num_subdirs(node_dir, "cpu");
    // 		if (max_cpus <= 0) {
    // 			pr_err("Fail to get cpu* dirs under %s\n", node_dir);
    // 			goto exit;
    // 		}
    // 	}

    for entry in fs::read_dir(&node_dir).unwrap() {
        let node_entry_dir = entry.unwrap();
        let file_name = node_entry_dir.file_name();

        /* ignore files like cpulist */
        let file_name = file_name.to_string_lossy().to_string();

        if file_name.contains("cpu") && node_entry_dir.path().is_dir() {
            let mut this_cpu = cpu_cacheinfo_t {
                len: 0,
                proc_num: 0,
                num_caches: 0,
                cache_prop: vec![],
            };

            // remove cpu text
            let mut proc_num = file_name.clone();
            proc_num.remove(0);
            proc_num.remove(0);
            proc_num.remove(0);

            // println!("file_name: {:?}", file_name);
            this_cpu.proc_num = proc_num.parse::<i32>().unwrap();

            let cache_path = format!("{}/{}/cache", node_dir, file_name);

            this_cpu.num_caches = num_subdirs(&cache_path, "index") as u32;

            cache_cnt += get_cpu_cache_info(&cache_path, cpuinfo, &mut this_cpu);

            temp_cpu_ci_list.push(this_cpu);
        }
    }

    (cache_cnt, temp_cpu_ci_list)
}

/* topology_get_cpu_cache_props - Read CPU cache information from sysfs
 *	@node [IN] CPU node number
 *	@cpuinfo [IN] /proc/cpuinfo data
 *	@tbl [OUT] the node table to fill up
 * Return: HSAKMT_STATUS_SUCCESS in success or error number in failure
 */
pub unsafe fn topology_get_cpu_cache_props(
    node: i32,
    cpuinfo: &[proc_cpuinfo],
    tbl: &mut node_props_t,
) -> HsakmtStatus {
    let (num_caches, cpu_ci_list) = topology_create_temp_cpu_cache_list(node, cpuinfo);

    tbl.node.NumCaches = num_caches as u32;

    // if (!tbl->node.NumCaches) {
    // 	/* For "Intel Meteor lake Mobile", the cache info is not in sysfs,
    // 	 * That means /sys/devices/system/node/node%d/%s/cache is not exist.
    // 	 * here AMD will not black this issue.
    // 	 */
    // 	pr_debug("CPU cache info is not available for node %d \n", node);
    // 	goto exit;
    // }

    /* Now fill in the information to cache properties. */

    // println!("tbl {}", tbl.node.NumCaches);
    // println!("cache {}", cpu_ci_list.iter().map(|x| x.cache_prop.len()).sum::<usize>());

    for cpu_ci in cpu_ci_list {
        for this_cache in cpu_ci.cache_prop {
            tbl.cache.push(this_cache);
        }
    }

    HSAKMT_STATUS_SUCCESS
}

/* Find the CPU that this GPU (gpu_node) directly connects to */
pub fn gpu_get_direct_link_cpu(gpu_node: u32, node_props: &mut [node_props_t]) -> i32 {
    let props = &node_props[gpu_node as usize].link;

    if !node_props[gpu_node as usize].node.KFDGpuID > 0
        || props.is_empty()
        || node_props[gpu_node as usize].node.NumIOLinks == 0
    {
        return -1;
    }

    for prop in props {
        /* >20 is GPU->CPU->GPU */
        if prop.IoLinkType == HSA_IOLINKTYPE_PCIEXPRESS && prop.Weight <= 20 {
            return prop.NodeTo as i32;
        }
    }

    -1
}

/* Get node1->node2 IO link information. This should be a direct link that has
 * been created in the kernel.
 */
pub fn get_direct_iolink_info(
    node1: u32,
    node2: u32,
    node_props: &[node_props_t],
    weight: &mut u32,
    hsa_type: Option<&mut HSA_IOLINKTYPE>,
) -> HsakmtStatus {
    let props = &node_props[node1 as usize].link;

    if props.is_empty() {
        return HSAKMT_STATUS_INVALID_NODE_UNIT;
    }

    for prop in props {
        if prop.NodeTo == node2 {
            if *weight > 0 {
                *weight = prop.Weight;
            }

            if let Some(v) = hsa_type {
                *v = prop.IoLinkType
            }

            return HSAKMT_STATUS_SUCCESS;
        }
    }

    HSAKMT_STATUS_INVALID_PARAMETER
}

pub unsafe fn get_indirect_iolink_info(
    node1: u32,
    node2: u32,
    node_props: &mut [node_props_t],
    weight: &mut u32,
    hsa_type: &mut HSA_IOLINKTYPE,
) -> HsakmtStatus {
    let mut dir_cpu1 = -1;
    let mut dir_cpu2 = -1;

    let mut weight1: u32 = 0;
    let mut weight2: u32 = 0;
    let mut weight3: u32 = 0;

    let mut i = 0;

    *weight = 0;
    *hsa_type = HSA_IOLINKTYPE_UNDEFINED;

    if node1 == node2 {
        return HSAKMT_STATUS_INVALID_PARAMETER;
    }

    /* CPU->CPU is not an indirect link */
    // !node_props[node1 as usize].node.KFDGpuID && !node_props[node2 as usize].node.KFDGpuID

    if node_props[node1 as usize].node.KFDGpuID == 0
        && node_props[node2 as usize].node.KFDGpuID == 0
    {
        return HSAKMT_STATUS_INVALID_NODE_UNIT;
    }

    if (node_props[node1 as usize].node.HiveID > 0)
        && (node_props[node2 as usize].node.HiveID > 0)
        && node_props[node1 as usize].node.HiveID == node_props[node2 as usize].node.HiveID
    {
        return HSAKMT_STATUS_INVALID_PARAMETER;
    }

    if node_props[node1 as usize].node.KFDGpuID > 0 {
        dir_cpu1 = gpu_get_direct_link_cpu(node1, node_props);
    }
    if node_props[node2 as usize].node.KFDGpuID > 0 {
        dir_cpu2 = gpu_get_direct_link_cpu(node2, node_props);
    }

    if dir_cpu1 < 0 && dir_cpu2 < 0 {
        return HSAKMT_STATUS_ERROR;
    }

    /* if the node2(dst) is GPU , it need to be large bar for host access*/
    if node_props[node2 as usize].node.KFDGpuID > 0 {
        for node_mem in &node_props[node2 as usize].mem {
            if node_mem.HeapType == HSA_HEAPTYPE_FRAME_BUFFER_PUBLIC {
                break;
            }

            i += 1;
        }

        if i >= node_props[node2 as usize].node.NumMemoryBanks {
            return HSAKMT_STATUS_ERROR;
        }
    }

    #[allow(unused_assignments)]
    let mut ret = HSAKMT_STATUS_SUCCESS;

    /* Possible topology:
     *   GPU --(weight1) -- CPU -- (weight2) -- GPU
     *   GPU --(weight1) -- CPU -- (weight2) -- CPU -- (weight3) -- GPU
     *   GPU --(weight1) -- CPU -- (weight2) -- CPU
     *   CPU -- (weight2) -- CPU -- (weight3) -- GPU
     */
    if dir_cpu1 >= 0 {
        /* GPU->CPU ... */
        if dir_cpu2 >= 0 {
            if dir_cpu1 == dir_cpu2
            /* GPU->CPU->GPU*/
            {
                ret =
                    get_direct_iolink_info(node1, dir_cpu1 as u32, node_props, &mut weight1, None);
                if ret != HSAKMT_STATUS_SUCCESS {
                    return ret;
                }

                ret = get_direct_iolink_info(
                    dir_cpu1 as u32,
                    node2,
                    node_props,
                    &mut weight2,
                    Some(hsa_type),
                );
            } else
            /* GPU->CPU->CPU->GPU*/
            {
                ret =
                    get_direct_iolink_info(node1, dir_cpu1 as u32, node_props, &mut weight1, None);
                if ret != HSAKMT_STATUS_SUCCESS {
                    return ret;
                }

                ret = get_direct_iolink_info(
                    dir_cpu1 as u32,
                    dir_cpu2 as u32,
                    node_props,
                    &mut weight2,
                    Some(hsa_type),
                );

                if ret != HSAKMT_STATUS_SUCCESS {
                    return ret;
                }
                /* On QPI interconnection, GPUs can't access
                 * each other if they are attached to different
                 * CPU sockets. CPU<->CPU weight larger than 20
                 * means the two CPUs are in different sockets.
                 */
                if *hsa_type == HSA_IOLINK_TYPE_QPI_1_1 && weight2 > 20 {
                    return HSAKMT_STATUS_NOT_SUPPORTED;
                }
                ret =
                    get_direct_iolink_info(dir_cpu2 as u32, node2, node_props, &mut weight3, None);
            }
        } else
        /* GPU->CPU->CPU */
        {
            ret = get_direct_iolink_info(node1, dir_cpu1 as u32, node_props, &mut weight1, None);
            if ret != HSAKMT_STATUS_SUCCESS {
                return ret;
            }
            ret = get_direct_iolink_info(
                dir_cpu1 as u32,
                node2,
                node_props,
                &mut weight2,
                Some(hsa_type),
            );
        }
    } else {
        /* CPU->CPU->GPU */
        ret = get_direct_iolink_info(
            node1,
            dir_cpu2 as u32,
            node_props,
            &mut weight2,
            Some(hsa_type),
        );
        if ret != HSAKMT_STATUS_SUCCESS {
            return ret;
        }

        ret = get_direct_iolink_info(dir_cpu2 as u32, node2, node_props, &mut weight3, None);
    }

    if ret != HSAKMT_STATUS_SUCCESS {
        return ret;
    }

    *weight = weight1 + weight2 + weight3;

    ret
}

/* topology_get_free_io_link_slot_for_node - For the given node_id, find the
 * next available free slot to add an io_link
 */
pub fn topology_get_free_io_link_slot_for_node<'a>(
    node_id: u32,
    sys_props: &HsaSystemProperties,
    node_props: &'a mut [node_props_t],
) -> Option<&'a mut HsaIoLinkProperties> {
    if node_id >= sys_props.NumNodes {
        println!("Invalid node [{}]", node_id);
        return None;
    }

    let props = &mut node_props[node_id as usize].link;
    if props.is_empty() {
        println!("No io_link reported for Node [{}]", node_id);
        return None;
    }

    if node_props[node_id as usize].node.NumIOLinks >= sys_props.NumNodes - 1 {
        println!("No more space for io_link for Node [{}]", node_id);
        return None;
    }

    let index = node_props[node_id as usize].node.NumIOLinks;
    let v = &mut props[index as usize];

    Some(v)
}

/* topology_add_io_link_for_node - If a free slot is available,
 * add io_link for the given Node.
 * TODO: Add other members of HsaIoLinkProperties
 */
pub fn topology_add_io_link_for_node(
    node_from: u32,
    sys_props: &HsaSystemProperties,
    node_props: &mut [node_props_t],
    IoLinkType: HSA_IOLINKTYPE,
    node_to: u32,
    Weight: u32,
) -> HsakmtStatus {
    let props = topology_get_free_io_link_slot_for_node(node_from, sys_props, node_props);
    if props.is_none() {
        return HSAKMT_STATUS_NO_MEMORY;
    }

    let props = props.unwrap();

    props.IoLinkType = IoLinkType;
    props.NodeFrom = node_from;
    props.NodeTo = node_to;
    props.Weight = Weight;

    node_props[node_from as usize].node.NumIOLinks += 1;

    HSAKMT_STATUS_SUCCESS
}

pub unsafe fn topology_create_indirect_gpu_links(
    sys_props: &HsaSystemProperties,
    node_props: &mut [node_props_t],
) {
    let mut weight = 0;
    let mut hsa_type = HSA_IOLINKTYPE_UNDEFINED;

    for i in 0..sys_props.NumNodes {
        for j in 0..sys_props.NumNodes {
            get_indirect_iolink_info(i, j, node_props, &mut weight, &mut hsa_type);

            if weight == 0 {
                // pass
            } else {
                let ret =
                    topology_add_io_link_for_node(i, sys_props, node_props, hsa_type, j, weight);
                if ret != HSAKMT_STATUS_SUCCESS {
                    println!("Fail to add IO link {} -> {}", i, j);
                }
            }

            get_indirect_iolink_info(j, i, node_props, &mut weight, &mut hsa_type);

            if weight == 0 {
                continue;
            } else {
                let ret =
                    topology_add_io_link_for_node(j, sys_props, node_props, hsa_type, i, weight);
                if ret != HSAKMT_STATUS_SUCCESS {
                    println!("Fail to add IO link {} -> {}", j, i);
                }
            }
        }
    }
}

pub unsafe fn topology_take_snapshot() -> HsakmtStatus {
    let mut gen_start: u32 = 0;
    let mut gen_end: u32 = 0;

    let mut sys_props: HsaSystemProperties = HsaSystemProperties::default();
    let mut temp_props: Vec<node_props_t> = Vec::new();

    let _num_ioLinks: u32 = 0;
    let mut p2p_links = false;
    let mut num_p2pLinks: u32 = 0;

    let (cpu_info, _num_procs) = topology_parse_cpuinfo();

    let ret = topology_sysfs_get_generation(&mut gen_start);
    if ret != HSAKMT_STATUS_SUCCESS {
        return ret;
    }

    let ret = hsakmt_topology_sysfs_get_system_props(&mut sys_props);
    if ret != HSAKMT_STATUS_SUCCESS {
        return ret;
    }

    let mut sys_devices_kfd = SysDevicesVirtualKfd::new();
    sys_devices_kfd.load_nodes();

    // FIXME temp_props

    if sys_props.NumNodes > 0 {
        for i in 0..sys_props.NumNodes as usize {
            temp_props.push(node_props_t::new());

            let ret = sys_devices_kfd.topology_sysfs_get_node_props(
                i as u32,
                &mut temp_props[i].node,
                &mut p2p_links,
                &mut num_p2pLinks,
            );

            if ret != HSAKMT_STATUS_SUCCESS {
                return ret;
            }

            // if temp_props[i].node.NumCPUCores != 0 {
            //     topology_get_cpu_model_name(&temp_props[i].node, cpuinfo, num_procs);
            // }

            if temp_props[i].node.NumMemoryBanks != 0 {
                for mem_id in 0..temp_props[i].node.NumMemoryBanks {
                    let mut hsa_mem_props = HsaMemoryProperties::default();

                    let ret = sys_devices_kfd.topology_sysfs_get_mem_props(
                        i as u32,
                        mem_id,
                        &mut hsa_mem_props,
                    );

                    if ret != HSAKMT_STATUS_SUCCESS {
                        return ret;
                    }

                    temp_props[i].mem.push(hsa_mem_props);
                }
            }

            if temp_props[i].node.NumCaches > 0 {
                for cache_id in 0..temp_props[i].node.NumCaches {
                    let mut hsa_cache_props = HsaCacheProperties::default();

                    let ret = sys_devices_kfd.topology_sysfs_get_cache_props(
                        i as u32,
                        cache_id,
                        &mut hsa_cache_props,
                    );

                    if ret != HSAKMT_STATUS_SUCCESS {
                        return ret;
                    }

                    temp_props[i].cache.push(hsa_cache_props);
                }
            } else if temp_props[i].node.KFDGpuID == 0 {
                /* a CPU node */
                let ret = topology_get_cpu_cache_props(i as i32, &cpu_info, &mut temp_props[i]);

                if ret != HSAKMT_STATUS_SUCCESS {
                    return ret;
                }
            }

            let num_ioLinks = temp_props[i].node.NumIOLinks - num_p2pLinks;
            let mut link_id = 0;

            if num_ioLinks > 0 {
                let mut sys_link_id = 0;
                /* Parse all the sysfs specified io links. Skip the ones where the
                 * remote node (node_to) is not accessible
                 */
                while sys_link_id < num_ioLinks && link_id < sys_props.NumNodes - 1 {
                    let mut temp_link = HsaIoLinkProperties::default();

                    let ret = sys_devices_kfd.topology_sysfs_get_iolink_props(
                        i as u32,
                        sys_link_id,
                        &mut temp_link,
                        false,
                    );

                    if ret == HSAKMT_STATUS_NOT_SUPPORTED {
                        continue;
                    } else if ret != HSAKMT_STATUS_SUCCESS {
                        return ret;
                    }

                    link_id += 1;
                    sys_link_id += 1;

                    temp_props[i].link.push(temp_link);
                }

                /* sysfs specifies all the io links. Limit the number to valid ones */
                temp_props[i].node.NumIOLinks = link_id;
            }

            if num_p2pLinks > 0 {
                let mut sys_link_id = 0;

                /* Parse all the sysfs specified p2p links.
                 */
                while sys_link_id < num_p2pLinks && link_id < sys_props.NumNodes - 1 {
                    let mut temp_link = HsaIoLinkProperties::default();

                    let ret = sys_devices_kfd.topology_sysfs_get_iolink_props(
                        i as u32,
                        sys_link_id,
                        &mut temp_link,
                        true,
                    );
                    if ret == HSAKMT_STATUS_NOT_SUPPORTED {
                        continue;
                    } else if ret != HSAKMT_STATUS_SUCCESS {
                        return ret;
                    }

                    link_id += 1;
                    sys_link_id += 1;

                    temp_props[i].link.push(temp_link);
                }

                temp_props[i].node.NumIOLinks = link_id;
            }
        }
    }

    if !p2p_links {
        /* All direct IO links are created in the kernel. Here we need to
         * connect GPU<->GPU or GPU<->CPU indirect IO links.
         */
        topology_create_indirect_gpu_links(&sys_props, &mut temp_props);
    }

    let ret = topology_sysfs_get_generation(&mut gen_end);
    if ret != HSAKMT_STATUS_SUCCESS {
        return ret;
    }

    if gen_start != gen_end {
        panic!("gen start != gen end");
    }

    hsakmt_topology_global_g_system_set(sys_props);
    hsakmt_topology_global_g_props_set(temp_props);

    HSAKMT_STATUS_SUCCESS
}

/* Drop the Snapshot of the HSA topology information. Assume lock is held. */
pub fn topology_drop_snapshot() {
    // ...
}

pub unsafe fn hsaKmtAcquireSystemProperties(
    system_properties: &mut HsaSystemProperties,
) -> HsakmtStatus {
    check_kfd_open_and_panic();

    let global = hsakmt_topology_global_get();

    if global.g_system != HsaSystemProperties::default() {
        system_properties.NumNodes = global.g_system.NumNodes;
        system_properties.PlatformOem = global.g_system.PlatformOem;
        system_properties.PlatformId = global.g_system.PlatformId;
        system_properties.PlatformRev = global.g_system.PlatformRev;

        return HSAKMT_STATUS_SUCCESS;
    }

    let ret = topology_take_snapshot();
    if ret != HSAKMT_STATUS_SUCCESS {
        return ret;
    }

    let global = hsakmt_topology_global_get();

    let err = hsakmt_fmm_init_process_apertures(global.g_system.NumNodes);
    if err != HSAKMT_STATUS_SUCCESS {
        topology_drop_snapshot();
        return err;
    }

    // err = hsakmt_init_process_doorbells(g_system->NumNodes);
    // if (err != HSAKMT_STATUS_SUCCESS)
    // goto init_doorbells_failed;

    *system_properties = global.g_system;

    HSAKMT_STATUS_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sysfs_nodes() {
        let mut sys_devices_virtual_kfd = SysDevicesVirtualKfd::new();
        sys_devices_virtual_kfd.load_nodes();

        println!("{:#?}", sys_devices_virtual_kfd);
        // TODO assert
    }

    #[test]
    fn test_topology_parse_cpuinfo() {
        let (cpu_info, c) = unsafe { topology_parse_cpuinfo() };

        println!("{:#?}", cpu_info);
        println!("{:#?}", c);
        // TODO assert
    }

    #[test]
    fn test_num_subdirs() {
        let p = "/sys/devices/system/node/node0";
        let count = num_subdirs(p, "cpu");

        println!("{:#?}", count);
        // TODO assert
    }
}
