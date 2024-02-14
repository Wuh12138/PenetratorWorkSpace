import { defineStore } from "pinia";
import {reactive} from "vue";
import { TrConfig,get_config_list,update_config_list,start_a_map,stop_a_map,get_running_item,TrItemInfo } from "../command";

export interface TableDateFormat {
  key: number;
  uid: number;
  name: string;
  local_ip: string;
  local_port: number;
  remote_ip: string;
  remote_port: number;
  port_to_pub: number;
  status: boolean;
}

export const useStatusStore = defineStore("status", () => {
  const columns = reactive([
    {
      title: "名称",
      dataIndex: "name",
      key: "name",
    },
    {
      title: "本地ip",
      dataIndex: "local_ip",
      key: "local_ip",
    },
    {
      title: "本地端口",
      dataIndex: "local_port",
      key: "local_port",
    },
    {
      title: "远程ip",
      dataIndex: "remote_ip",
      key: "remote_ip",
    },
    {
      title: "公网端口",
      dataIndex: "port_to_pub",
      key: "port_to_pub",
    },
    {
      title: "状态",
      dataIndex: "status",
      key: "status",
    },
  ]);

  let config_list:TrConfig[] = reactive([]);
  const uid_map_key = reactive(new Map<number, number>());
  
  const data = reactive<TableDateFormat[]>([]);


  function confit_to_data(config:TrConfig):TableDateFormat {
    return {
      key: data.length,
      uid: -1,
      name: config.name,
      local_ip: config.local_addr,
      local_port: config.local_port,
      remote_ip: config.remote_host,
      remote_port: config.remote_port,
      port_to_pub: config.port_to_pub,
      status: false,
    };
  }

  async function initDate() {
    config_list = await get_config_list();
    data.length = 0;
    for (const config of config_list) {
      data.push(confit_to_data(config));
    }
  }
  
  let is_query_running = false;
  async function updateDataStatus() {
    if (is_query_running) {
      return;
    }
    is_query_running = true;
    const running_item:TrItemInfo[] = await get_running_item();
    is_query_running = false;
    for (const item of data) {
      item.status = false;
    }

    for (const item of running_item) {
      const key = uid_map_key.get(item.uid);
      if (key !== undefined) {
        data[key].status = true;
      }
    }
  }

  async function add_config(config:TrConfig) {
    config_list.push(config);
    data.push(confit_to_data(config));
    await update_config_list(config_list);

  }

  async function start_map(key:number) {
    let old_uid = data[key].uid;
    if (old_uid !== -1) {
      return ;
    }

    const uid = await  start_a_map(config_list[key]);
    if (uid !== -1) {
      data[key].uid = uid;
      uid_map_key.set(uid, key);
    }
  }
  async function stop_map(key:number) {
    const uid = data[key].uid;
    if (uid !== -1) {
      await stop_a_map(uid);
      data[key].uid = -1;
      uid_map_key.delete(uid);
    }
  }


  return { columns, data, updateDataStatus,initDate,add_config,start_map,stop_map};
});
