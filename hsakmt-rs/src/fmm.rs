#![allow(non_camel_case_types, dead_code, non_snake_case)]

use crate::fmm_globals::{hsakmt_fmm_global_get, DRM_FIRST_RENDER_NODE, DRM_LAST_RENDER_NODE};
use crate::hsakmttypes::HsakmtStatus;
use crate::hsakmttypes::HsakmtStatus::HSAKMT_STATUS_SUCCESS;
use amdgpu_drm_sys::bindings::{amdgpu_device, amdgpu_device_initialize};
use libc::{open, EACCES, EINVAL, ENOENT, EPERM, O_CLOEXEC, O_RDWR};
use std::ffi::CString;
use std::mem::MaybeUninit;

pub unsafe fn hsakmt_open_drm_render_device(minor: i32) -> i32 {
    let mut globals = hsakmt_fmm_global_get();

    if minor < DRM_FIRST_RENDER_NODE as i32 || minor > DRM_LAST_RENDER_NODE as i32 {
        println!(
            "DRM render minor {} out of range [{}, {}]\n",
            minor, DRM_FIRST_RENDER_NODE, DRM_LAST_RENDER_NODE
        );
        return -EINVAL;
    }

    let index = (minor - DRM_FIRST_RENDER_NODE as i32) as usize;

    /* If the render node was already opened, keep using the same FD */
    if globals.drm_render_fds[index] != 0 {
        return globals.drm_render_fds[index];
    }

    let path = format!("/dev/dri/renderD{}", minor);
    let path_cs = CString::new(path.as_str()).unwrap();

    // let fd = File::open(&path).unwrap();
    // println!("File fd {:?}", fd);

    let fd = open(path_cs.as_ptr(), O_RDWR | O_CLOEXEC);

    if fd < 0 {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap();

        if errno != ENOENT && errno != EPERM {
            println!("Failed to open {:?} {:?}", path, errno);
            if errno == EACCES {
                println!("Check user is in \"video\" group")
            }
        }
        return -errno;
    }

    globals.drm_render_fds[index] = fd;

    let mut device_handle: MaybeUninit<amdgpu_device> = MaybeUninit::uninit();
    let mut major_drm: MaybeUninit<u32> = MaybeUninit::zeroed();
    let mut minor_drm: MaybeUninit<u32> = MaybeUninit::zeroed();

    let ret = amdgpu_device_initialize(
        fd,
        major_drm.as_mut_ptr(),
        minor_drm.as_mut_ptr(),
        &mut device_handle.as_mut_ptr(),
    );
    if ret != 0 {
        panic!("amdgpu_device_initialize failed");
    }

    fd
}

pub unsafe fn hsakmt_fmm_init_process_apertures(_NumNodes: u32) -> HsakmtStatus {
    // uint32_t i;
    // int32_t gpu_mem_id = 0;
    // let process_apertures = kfd_process_device_apertures {
    //
    // };
    // uint32_t num_of_sysfs_nodes;
    // HSAKMT_STATUS ret = HSAKMT_STATUS_SUCCESS;
    // char *disableCache, *pagedUserptr, *checkUserptr, *guardPagesStr, *reserveSvm;
    // char *maxVaAlignStr;
    // unsigned int guardPages = 1;
    // uint64_t svm_base = 0, svm_limit = 0;
    // uint32_t svm_alignment = 0;

    // 	/* If HSA_DISABLE_CACHE is set to a non-0 value, disable caching */
    // 	disableCache = getenv("HSA_DISABLE_CACHE");
    // 	svm.disable_cache = (disableCache && strcmp(disableCache, "0"));
    //
    // 	/* If HSA_USERPTR_FOR_PAGED_MEM is not set or set to a non-0
    // 	 * value, enable userptr for all paged memory allocations
    // 	 */
    // 	pagedUserptr = getenv("HSA_USERPTR_FOR_PAGED_MEM");
    // 	svm.userptr_for_paged_mem = (!pagedUserptr || strcmp(pagedUserptr, "0"));
    //
    // 	/* If HSA_CHECK_USERPTR is set to a non-0 value, check all userptrs
    // 	 * when they are registered
    // 	 */
    // 	checkUserptr = getenv("HSA_CHECK_USERPTR");
    // 	svm.check_userptr = (checkUserptr && strcmp(checkUserptr, "0"));
    //
    // 	/* If HSA_RESERVE_SVM is set to a non-0 value,
    // 	 * enable packet capture and replay mode.
    // 	 */
    // 	reserveSvm = getenv("HSA_RESERVE_SVM");
    // 	svm.reserve_svm = (reserveSvm && strcmp(reserveSvm, "0"));
    //
    // 	/* Specify number of guard pages for SVM apertures, default is 1 */
    // 	guardPagesStr = getenv("HSA_SVM_GUARD_PAGES");
    // 	if (!guardPagesStr || sscanf(guardPagesStr, "%u", &guardPages) != 1)
    // 		guardPages = 1;
    //
    // 	/* Sets the max VA alignment order size during mapping. By default the order
    // 	 * size is set to 9(2MB)
    // 	 */
    // 	maxVaAlignStr = getenv("HSA_MAX_VA_ALIGN");
    // 	if (!maxVaAlignStr || sscanf(maxVaAlignStr, "%u", &svm.alignment_order) != 1)
    // 		svm.alignment_order = 9;
    //
    // 	gpu_mem_count = 0;
    // 	g_first_gpu_mem = NULL;
    //
    // 	/* Trade off - NumNodes includes GPU nodes + CPU Node. So in
    // 	 * systems with CPU node, slightly more memory is allocated than
    // 	 * necessary
    // 	 */
    // 	gpu_mem = (gpu_mem_t *)calloc(NumNodes, sizeof(gpu_mem_t));
    // 	if (!gpu_mem)
    // 		return HSAKMT_STATUS_NO_MEMORY;
    //
    // 	/* Initialize gpu_mem[] from sysfs topology. Rest of the members are
    // 	 * set to 0 by calloc. This is necessary because this function
    // 	 * gets called before hsaKmtAcquireSystemProperties() is called.
    // 	 */
    //
    // 	hsakmt_is_dgpu = false;
    //
    // 	for (i = 0; i < NumNodes; i++) {
    // 		HsaNodeProperties props;
    //
    // 		ret = hsakmt_topology_get_node_props(i, &props);
    // 		if (ret != HSAKMT_STATUS_SUCCESS)
    // 			goto gpu_mem_init_failed;
    //
    // 		hsakmt_topology_setup_is_dgpu_param(&props);
    //
    // 		/* Skip non-GPU nodes */
    // 		if (props.KFDGpuID) {
    // 			int fd = hsakmt_open_drm_render_device(props.DrmRenderMinor);
    // 			if (fd <= 0) {
    // 				ret = HSAKMT_STATUS_ERROR;
    // 				goto gpu_mem_init_failed;
    // 			}
    //
    // 			gpu_mem[gpu_mem_count].drm_render_minor = props.DrmRenderMinor;
    // 			gpu_mem[gpu_mem_count].usable_peer_id_array =
    // 				calloc(NumNodes, sizeof(uint32_t));
    // 			if (!gpu_mem[gpu_mem_count].usable_peer_id_array) {
    // 				ret = HSAKMT_STATUS_NO_MEMORY;
    // 				goto gpu_mem_init_failed;
    // 			}
    // 			gpu_mem[gpu_mem_count].usable_peer_id_array[0] = props.KFDGpuID;
    // 			gpu_mem[gpu_mem_count].usable_peer_id_num = 1;
    //
    // 			gpu_mem[gpu_mem_count].EngineId.ui32.Major = props.EngineId.ui32.Major;
    // 			gpu_mem[gpu_mem_count].EngineId.ui32.Minor = props.EngineId.ui32.Minor;
    // 			gpu_mem[gpu_mem_count].EngineId.ui32.Stepping = props.EngineId.ui32.Stepping;
    //
    // 			gpu_mem[gpu_mem_count].drm_render_fd = fd;
    // 			gpu_mem[gpu_mem_count].gpu_id = props.KFDGpuID;
    // 			gpu_mem[gpu_mem_count].local_mem_size = props.LocalMemSize;
    // 			gpu_mem[gpu_mem_count].device_id = props.DeviceId;
    // 			gpu_mem[gpu_mem_count].node_id = i;
    // 			hsakmt_is_svm_api_supported &= props.Capability.ui32.SVMAPISupported;
    //
    // 			gpu_mem[gpu_mem_count].scratch_physical.align = PAGE_SIZE;
    // 			gpu_mem[gpu_mem_count].scratch_physical.ops = &reserved_aperture_ops;
    // 			pthread_mutex_init(&gpu_mem[gpu_mem_count].scratch_physical.fmm_mutex, NULL);
    //
    // 			gpu_mem[gpu_mem_count].gpuvm_aperture.align =
    // 				get_vm_alignment(props.DeviceId);
    // 			gpu_mem[gpu_mem_count].gpuvm_aperture.guard_pages = guardPages;
    // 			gpu_mem[gpu_mem_count].gpuvm_aperture.ops = &reserved_aperture_ops;
    // 			pthread_mutex_init(&gpu_mem[gpu_mem_count].gpuvm_aperture.fmm_mutex, NULL);
    //
    // 			if (!g_first_gpu_mem)
    // 				g_first_gpu_mem = &gpu_mem[gpu_mem_count];
    //
    // 			gpu_mem_count++;
    // 		}
    // 	}
    //
    // 	/* The ioctl will also return Number of Nodes if
    // 	 * args.kfd_process_device_apertures_ptr is set to NULL. This is not
    // 	 * required since Number of nodes is already known. Kernel will fill in
    // 	 * the apertures in kfd_process_device_apertures_ptr
    // 	 */
    // 	num_of_sysfs_nodes = hsakmt_get_num_sysfs_nodes();
    // 	if (num_of_sysfs_nodes < gpu_mem_count) {
    // 		ret = HSAKMT_STATUS_ERROR;
    // 		goto sysfs_parse_failed;
    // 	}
    //
    // 	process_apertures = calloc(num_of_sysfs_nodes, sizeof(struct kfd_process_device_apertures));
    // 	if (!process_apertures) {
    // 		ret = HSAKMT_STATUS_NO_MEMORY;
    // 		goto sysfs_parse_failed;
    // 	}
    //
    // 	/* GPU Resource management can disable some of the GPU nodes.
    // 	 * The Kernel driver could be not aware of this.
    // 	 * Get from Kernel driver information of all the nodes and then filter it.
    // 	 */
    // 	ret = get_process_apertures(process_apertures, &num_of_sysfs_nodes);
    // 	if (ret != HSAKMT_STATUS_SUCCESS)
    // 		goto get_aperture_ioctl_failed;
    //
    // 	all_gpu_id_array_size = 0;
    // 	all_gpu_id_array = NULL;
    // 	if (num_of_sysfs_nodes > 0) {
    // 		all_gpu_id_array = malloc(sizeof(uint32_t) * gpu_mem_count);
    // 		if (!all_gpu_id_array) {
    // 			ret = HSAKMT_STATUS_NO_MEMORY;
    // 			goto get_aperture_ioctl_failed;
    // 		}
    // 	}
    //
    // 	for (i = 0 ; i < num_of_sysfs_nodes ; i++) {
    // 		HsaNodeProperties nodeProps;
    // 		HsaIoLinkProperties linkProps[NumNodes];
    // 		uint32_t nodeId;
    // 		uint32_t j;
    //
    // 		/* Map Kernel process device data node i <--> gpu_mem_id which
    // 		 * indexes into gpu_mem[] based on gpu_id
    // 		 */
    // 		gpu_mem_id = gpu_mem_find_by_gpu_id(process_apertures[i].gpu_id);
    // 		if (gpu_mem_id < 0)
    // 			continue;
    //
    // 		if (all_gpu_id_array_size == gpu_mem_count) {
    // 			ret = HSAKMT_STATUS_ERROR;
    // 			goto aperture_init_failed;
    // 		}
    // 		all_gpu_id_array[all_gpu_id_array_size++] = process_apertures[i].gpu_id;
    //
    // 		/* Add this GPU to the usable_peer_id_arrays of all GPUs that
    // 		 * this GPU has an IO link to. This GPU can map memory
    // 		 * allocated on those GPUs.
    // 		 */
    // 		nodeId = gpu_mem[gpu_mem_id].node_id;
    // 		ret = hsakmt_topology_get_node_props(nodeId, &nodeProps);
    // 		if (ret != HSAKMT_STATUS_SUCCESS)
    // 			goto aperture_init_failed;
    // 		assert(nodeProps.NumIOLinks <= NumNodes);
    // 		ret = hsakmt_topology_get_iolink_props(nodeId, nodeProps.NumIOLinks,
    // 						linkProps);
    // 		if (ret != HSAKMT_STATUS_SUCCESS)
    // 			goto aperture_init_failed;
    // 		for (j = 0; j < nodeProps.NumIOLinks; j++) {
    // 			int32_t to_gpu_mem_id =
    // 				gpu_mem_find_by_node_id(linkProps[j].NodeTo);
    // 			uint32_t peer;
    //
    // 			if (to_gpu_mem_id < 0)
    // 				continue;
    //
    // 			assert(gpu_mem[to_gpu_mem_id].usable_peer_id_num < NumNodes);
    // 			peer = gpu_mem[to_gpu_mem_id].usable_peer_id_num++;
    // 			gpu_mem[to_gpu_mem_id].usable_peer_id_array[peer] =
    // 				gpu_mem[gpu_mem_id].gpu_id;
    // 		}
    //
    // 		gpu_mem[gpu_mem_id].lds_aperture.base =
    // 			PORT_UINT64_TO_VPTR(process_apertures[i].lds_base);
    // 		gpu_mem[gpu_mem_id].lds_aperture.limit =
    // 			PORT_UINT64_TO_VPTR(process_apertures[i].lds_limit);
    //
    // 		gpu_mem[gpu_mem_id].scratch_aperture.base =
    // 			PORT_UINT64_TO_VPTR(process_apertures[i].scratch_base);
    // 		gpu_mem[gpu_mem_id].scratch_aperture.limit =
    // 			PORT_UINT64_TO_VPTR(process_apertures[i].scratch_limit);
    //
    // 		if (IS_CANONICAL_ADDR(process_apertures[i].gpuvm_limit)) {
    // 			uint64_t vm_alignment = get_vm_alignment(
    // 				gpu_mem[gpu_mem_id].device_id);
    //
    // 			/* Set proper alignment for scratch backing aperture */
    // 			gpu_mem[gpu_mem_id].scratch_physical.align = vm_alignment;
    //
    // 			/* Non-canonical per-ASIC GPUVM aperture does
    // 			 * not exist on dGPUs in GPUVM64 address mode
    // 			 */
    // 			gpu_mem[gpu_mem_id].gpuvm_aperture.base = NULL;
    // 			gpu_mem[gpu_mem_id].gpuvm_aperture.limit = NULL;
    //
    // 			/* Update SVM aperture limits and alignment */
    // 			if (process_apertures[i].gpuvm_base > svm_base)
    // 				svm_base = process_apertures[i].gpuvm_base;
    // 			if (process_apertures[i].gpuvm_limit < svm_limit ||
    // 			    svm_limit == 0)
    // 				svm_limit = process_apertures[i].gpuvm_limit;
    // 			if (vm_alignment > svm_alignment)
    // 				svm_alignment = vm_alignment;
    // 		} else {
    // 			gpu_mem[gpu_mem_id].gpuvm_aperture.base =
    // 				PORT_UINT64_TO_VPTR(process_apertures[i].gpuvm_base);
    // 			gpu_mem[gpu_mem_id].gpuvm_aperture.limit =
    // 				PORT_UINT64_TO_VPTR(process_apertures[i].gpuvm_limit);
    // 			/* Reserve space at the start of the
    // 			 * aperture. After subtracting the base, we
    // 			 * don't want valid pointers to become NULL.
    // 			 */
    // 			aperture_allocate_area(
    // 				&gpu_mem[gpu_mem_id].gpuvm_aperture,
    // 				NULL,
    // 				gpu_mem[gpu_mem_id].gpuvm_aperture.align);
    // 		}
    //
    // 		/* Acquire the VM from the DRM render node for KFD use */
    // 		ret = acquire_vm(gpu_mem[gpu_mem_id].gpu_id,
    // 				 gpu_mem[gpu_mem_id].drm_render_fd);
    // 		if (ret != HSAKMT_STATUS_SUCCESS)
    // 			goto aperture_init_failed;
    // 	}
    // 	all_gpu_id_array_size *= sizeof(uint32_t);
    //
    // 	if (svm_limit) {
    // 		/* At least one GPU uses GPUVM in canonical address
    // 		 * space. Set up SVM apertures shared by all such GPUs
    // 		 */
    // 		ret = init_svm_apertures(svm_base, svm_limit, svm_alignment,
    // 					 guardPages);
    // 		if (ret != HSAKMT_STATUS_SUCCESS)
    // 			goto init_svm_failed;
    //
    // 		for (i = 0 ; i < num_of_sysfs_nodes ; i++) {
    // 			uintptr_t alt_base;
    // 			uint64_t alt_size;
    // 			int err;
    //
    // 			if (!IS_CANONICAL_ADDR(process_apertures[i].gpuvm_limit))
    // 				continue;
    //
    // 			/* Set memory policy to match the SVM apertures */
    // 			alt_base = (uintptr_t)svm.dgpu_alt_aperture->base;
    // 			alt_size = VOID_PTRS_SUB(svm.dgpu_alt_aperture->limit,
    // 				svm.dgpu_alt_aperture->base) + 1;
    // 			err = fmm_set_memory_policy(process_apertures[i].gpu_id,
    // 						    svm.disable_cache ?
    // 						    KFD_IOC_CACHE_POLICY_COHERENT :
    // 						    KFD_IOC_CACHE_POLICY_NONCOHERENT,
    // 						    KFD_IOC_CACHE_POLICY_COHERENT,
    // 						    alt_base, alt_size);
    // 			if (err) {
    // 				pr_err("Failed to set mem policy for GPU [0x%x]\n",
    // 				       process_apertures[i].gpu_id);
    // 				ret = HSAKMT_STATUS_ERROR;
    // 				goto set_memory_policy_failed;
    // 			}
    // 		}
    // 	}
    //
    // 	cpuvm_aperture.align = PAGE_SIZE;
    // 	cpuvm_aperture.limit = (void *)0x7FFFFFFFFFFF; /* 2^47 - 1 */
    //
    // 	fmm_init_rbtree();
    //
    // 	if (!init_mem_handle_aperture(PAGE_SIZE, guardPages))
    // 		pr_err("Failed to init mem_handle_aperture\n");
    //
    // 	for (gpu_mem_id = 0; (uint32_t)gpu_mem_id < gpu_mem_count; gpu_mem_id++) {
    // 		if (!hsakmt_topology_is_svm_needed(gpu_mem[gpu_mem_id].EngineId))
    // 			continue;
    // 		gpu_mem[gpu_mem_id].mmio_aperture.base = map_mmio(
    // 				gpu_mem[gpu_mem_id].node_id,
    // 				gpu_mem[gpu_mem_id].gpu_id,
    // 				hsakmt_kfd_fd);
    // 		if (gpu_mem[gpu_mem_id].mmio_aperture.base)
    // 			gpu_mem[gpu_mem_id].mmio_aperture.limit = (void *)
    // 			((char *)gpu_mem[gpu_mem_id].mmio_aperture.base +
    // 			 PAGE_SIZE - 1);
    // 		else
    // 			pr_err("Failed to map remapped mmio page on gpu_mem %d\n",
    // 					gpu_mem_id);
    // 	}
    //
    // 	free(process_apertures);
    // 	return ret;
    //
    // aperture_init_failed:
    // init_svm_failed:
    // set_memory_policy_failed:
    // 	free(all_gpu_id_array);
    // 	all_gpu_id_array = NULL;
    // get_aperture_ioctl_failed:
    // 	free(process_apertures);
    // sysfs_parse_failed:
    // gpu_mem_init_failed:
    // 	hsakmt_fmm_destroy_process_apertures();
    // 	return ret;

    HSAKMT_STATUS_SUCCESS
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_globals() {
//         let mut drm_render_fds = HSA_KMT_FMM_GLOBAL.lock().unwrap();
//         println!("{:?}", drm_render_fds);
//         drm_render_fds.drm_render_fd_set(0, 32);
//         drm_render_fds.amdgpu_handle_set(0, amdgpu_device { _unused: [] });
//         println!("{:?}", drm_render_fds);
//
//         // TODO assert
//     }
// }
