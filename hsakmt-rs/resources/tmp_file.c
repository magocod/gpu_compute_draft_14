HSAKMT_STATUS topology_take_snapshot(void) {
	uint32_t gen_start, gen_end, i, mem_id, cache_id;
	HsaSystemProperties sys_props;
	node_props_t *temp_props = 0;
	HSAKMT_STATUS ret = HSAKMT_STATUS_SUCCESS;
	struct proc_cpuinfo *cpuinfo;
	const uint32_t num_procs = get_nprocs();
	uint32_t num_ioLinks;
	bool p2p_links = false;
	uint32_t num_p2pLinks = 0;

	cpuinfo = calloc(num_procs, sizeof(struct proc_cpuinfo));
	if (!cpuinfo) {
		pr_err("Fail to allocate memory for CPU info\n");
		return HSAKMT_STATUS_NO_MEMORY;
	}
	topology_parse_cpuinfo(cpuinfo, num_procs);

retry:
	ret = topology_sysfs_get_generation(&gen_start);
	if (ret != HSAKMT_STATUS_SUCCESS)
		goto err;
	ret = hsakmt_topology_sysfs_get_system_props(&sys_props);
	if (ret != HSAKMT_STATUS_SUCCESS)
		goto err;
	if (sys_props.NumNodes > 0) {
		temp_props = calloc(sys_props.NumNodes * sizeof(node_props_t), 1);
		if (!temp_props) {
			ret = HSAKMT_STATUS_NO_MEMORY;
			goto err;
		}
		for (i = 0; i < sys_props.NumNodes; i++) {
			ret = topology_sysfs_get_node_props(i,
					&temp_props[i].node,
					&p2p_links, &num_p2pLinks);
			if (ret != HSAKMT_STATUS_SUCCESS) {
				free_properties(temp_props, i);
				goto err;
			}

			if (temp_props[i].node.NumCPUCores)
				topology_get_cpu_model_name(&temp_props[i].node,
							cpuinfo, num_procs);

			if (temp_props[i].node.NumMemoryBanks) {
				temp_props[i].mem = calloc(temp_props[i].node.NumMemoryBanks * sizeof(HsaMemoryProperties), 1);
				if (!temp_props[i].mem) {
					ret = HSAKMT_STATUS_NO_MEMORY;
					free_properties(temp_props, i + 1);
					goto err;
				}
				for (mem_id = 0; mem_id < temp_props[i].node.NumMemoryBanks; mem_id++) {
					ret = topology_sysfs_get_mem_props(i, mem_id, &temp_props[i].mem[mem_id]);
					if (ret != HSAKMT_STATUS_SUCCESS) {
						free_properties(temp_props, i + 1);
						goto err;
					}
				}
			}

			if (temp_props[i].node.NumCaches) {
				temp_props[i].cache = calloc(temp_props[i].node.NumCaches * sizeof(HsaCacheProperties), 1);
				if (!temp_props[i].cache) {
					ret = HSAKMT_STATUS_NO_MEMORY;
					free_properties(temp_props, i + 1);
					goto err;
				}
				for (cache_id = 0; cache_id < temp_props[i].node.NumCaches; cache_id++) {
					ret = topology_sysfs_get_cache_props(i, cache_id, &temp_props[i].cache[cache_id]);
					if (ret != HSAKMT_STATUS_SUCCESS) {
						free_properties(temp_props, i + 1);
						goto err;
					}
				}
			} else if (!temp_props[i].node.KFDGpuID) { /* a CPU node */
				ret = topology_get_cpu_cache_props(
						i, cpuinfo, &temp_props[i]);
				if (ret != HSAKMT_STATUS_SUCCESS) {
					free_properties(temp_props, i + 1);
					goto err;
				}
			}

			/* To simplify, allocate maximum needed memory for io_links for each node. This
			 * removes the need for realloc when indirect and QPI links are added later
			 */
			temp_props[i].link = calloc(sys_props.NumNodes - 1, sizeof(HsaIoLinkProperties));
			if (!temp_props[i].link) {
				ret = HSAKMT_STATUS_NO_MEMORY;
				free_properties(temp_props, i + 1);
				goto err;
			}
			num_ioLinks = temp_props[i].node.NumIOLinks - num_p2pLinks;
			uint32_t link_id = 0;

			if (num_ioLinks) {
				uint32_t sys_link_id = 0;

				/* Parse all the sysfs specified io links. Skip the ones where the
				 * remote node (node_to) is not accessible
				 */
				while (sys_link_id < num_ioLinks &&
					link_id < sys_props.NumNodes - 1) {
					ret = topology_sysfs_get_iolink_props(i, sys_link_id++,
								&temp_props[i].link[link_id], false);
					if (ret == HSAKMT_STATUS_NOT_SUPPORTED) {
						continue;
					} else if (ret != HSAKMT_STATUS_SUCCESS) {
						free_properties(temp_props, i + 1);
						goto err;
					}
					link_id++;
				}
				/* sysfs specifies all the io links. Limit the number to valid ones */
				temp_props[i].node.NumIOLinks = link_id;
			}

			if (num_p2pLinks) {
				uint32_t sys_link_id = 0;

				/* Parse all the sysfs specified p2p links.
				 */
				while (sys_link_id < num_p2pLinks &&
					link_id < sys_props.NumNodes - 1) {
					ret = topology_sysfs_get_iolink_props(i, sys_link_id++,
								&temp_props[i].link[link_id], true);
					if (ret == HSAKMT_STATUS_NOT_SUPPORTED) {
						continue;
					} else if (ret != HSAKMT_STATUS_SUCCESS) {
						free_properties(temp_props, i + 1);
						goto err;
					}
					link_id++;
				}
				temp_props[i].node.NumIOLinks = link_id;
			}
		}
	}

	if (!p2p_links) {
		/* All direct IO links are created in the kernel. Here we need to
		 * connect GPU<->GPU or GPU<->CPU indirect IO links.
		 */
		topology_create_indirect_gpu_links(&sys_props, temp_props);
	}

	ret = topology_sysfs_get_generation(&gen_end);
	if (ret != HSAKMT_STATUS_SUCCESS) {
		free_properties(temp_props, sys_props.NumNodes);
		goto err;
	}

	if (gen_start != gen_end) {
		free_properties(temp_props, sys_props.NumNodes);
		temp_props = 0;
		goto retry;
	}

	if (!g_system) {
		g_system = malloc(sizeof(HsaSystemProperties));
		if (!g_system) {
			free_properties(temp_props, sys_props.NumNodes);
			ret = HSAKMT_STATUS_NO_MEMORY;
			goto err;
		}
	}

	*g_system = sys_props;
	if (g_props)
		free(g_props);
	g_props = temp_props;
err:
	free(cpuinfo);
	return ret;
}