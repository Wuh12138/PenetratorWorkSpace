import { defineStore } from "pinia";
import { reactive } from "vue";

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
      title: "远程端口",
      dataIndex: "remote_port",
      key: "remote_port",
    },
    {
      title: "状态",
      dataIndex: "status",
      key: "status",
    },
  ]);

  const data = reactive([
    {
      key: "1",
      name: "test",
      local_ip: "127.0.0.1",
      local_port: "8080",
      remote_ip: "47.92.175.152",
      remote_port: "80",
      status: true,
    },
    {
      key: "2",
      name: "test",
      local_ip: "127.0.0.1",
      local_port: "8080",
      remote_ip: "47.92.175.152",
      remote_port: "80",
      status: true,
    },
    {
      key: "3",
      name: "test",
      local_ip: "127.0.0.1",
      local_port: "8080",
      remote_ip: "47.92.175.152",
      remote_port: "80",
      status: false,
    },
    {
      key: "4",
      name: "test",
      local_ip: "127.0.0.1",
      local_port: "8080",
      remote_ip: "47.92.175.152",
      remote_port: "80",
      status: false,
    },
  ]);

  return { columns, data };
});
