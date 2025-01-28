use crate::fmm::hsakmt_open_drm_render_device;
use crate::types::HsakmtStatus::HSAKMT_STATUS_SUCCESS;
use crate::types::{HsaSystemProperties, HsakmtStatus};
use libc::{ENOENT, EPERM};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub const KFD_SYSFS_PATH_SYSTEM_PROPERTIES: &str =
    "/sys/devices/virtual/kfd/kfd/topology/system_properties";
pub const KFD_SYSFS_PATH_NODES: &str = "/sys/devices/virtual/kfd/kfd/topology/nodes";

#[derive(Debug, PartialEq, Clone)]
pub struct HsaKmtTopologyGlobal {
    pub map_user_to_sysfs_node_id: Vec<usize>,
    pub map_user_to_sysfs_node_id_size: usize,
}

static HSA_KMT_TOPOLOGY_GLOBAL: Mutex<HsaKmtTopologyGlobal> = Mutex::new(HsaKmtTopologyGlobal {
    map_user_to_sysfs_node_id: vec![],
    map_user_to_sysfs_node_id_size: 0,
});

pub fn hsakmt_topology_global_get() -> HsaKmtTopologyGlobal {
    HSA_KMT_TOPOLOGY_GLOBAL.lock().unwrap().clone()
}

pub fn hsakmt_topology_global_map_user_to_sysfs_node_id_set(ids: Vec<usize>) {
    let mut g = HSA_KMT_TOPOLOGY_GLOBAL.lock().unwrap();
    g.map_user_to_sysfs_node_id = ids;
    g.map_user_to_sysfs_node_id_size = g.map_user_to_sysfs_node_id.len();
}

#[derive(Debug, PartialEq)]
pub struct KfdTopologyNodeProperties {
    drm_render_minor: usize,
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
                        drm_render_minor: 0,
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

                                if pair[0] == "drm_render_minor" {
                                    kfd_topology_node.properties.drm_render_minor =
                                        pair[1].trim().parse::<usize>().unwrap();
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
        let ret_value =
            unsafe { hsakmt_open_drm_render_device(node.properties.drm_render_minor as i32) };

        if ret_value > 0 {
            return true;
        } else if ret_value != -ENOENT && ret_value != -EPERM {
            // ret = HSAKMT_STATUS_ERROR;
        }

        false
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
}
