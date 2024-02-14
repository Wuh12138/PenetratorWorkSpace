<template>
  <a-flex class="outer-flex">
    <a-flex
      class="col-one"
      :vertical="false"
      justify="space-around"
      wrap="wrap"
    >
      <a-button
        :style="{
          width: '50%',
          height: '50%',
        }"
        @click="start_callback"
      >
        启动
      </a-button>
      <a-button
        :style="{
          width: '50%',
          height: '50%',
        }"
        @click="stop_callback"
      >
        停止
      </a-button>

      <a-button
        :style="{
          width: '50%',
          height: '50%',
        }"
        @click="create_callback"
        :disabled="crb_disable"
      >
        创建
      </a-button>

      <a-button
        :style="{
          width: '50%',
          height: '50%',
        }"
      >
        删除
      </a-button>
    </a-flex>

    <a-flex class="col-two">
      <a-table 
      :columns="columns" 
      :data-source="data"
      :row-selection="{ selectedRowKeys: state.selectedRowKeys, onChange: onSelectChange }"
      :style="{
        width: '100%',
        height: '100%',
      }"
      >
        <template #bodyCell="{column,record}">
          <span v-if="column.key === 'status'">
            <a-tag v-if="record[column.key]" color="green">运行中</a-tag>
            <a-tag v-else color="red">已停止</a-tag>
          </span>
          <span v-else>{{ record[column.key] }}</span>
      
      </template>
      </a-table>
    </a-flex>
  </a-flex>
</template>

<script setup lang="ts">
import { message } from "ant-design-vue";
import { reactive, ref } from "vue";
import { useStatusStore } from "../store/StatusList";
import { storeToRefs } from "pinia";
import { TrConfig } from "../command";


const store=useStatusStore();
const { columns, data}=storeToRefs(store);
const {updateDataStatus,initDate,add_config,start_map,stop_map}=store;

// initDate();
initDate();
setInterval(updateDataStatus, 300);


const state = reactive({
  selectedRowKeys: [],
});

const onSelectChange = (selectedRowKeys: []) => {
  state.selectedRowKeys = selectedRowKeys;
  console.log("selectedRowKeys changed: ", selectedRowKeys);
};

const start_callback = () => {
  message.destroy();
  for (let i = 0; i < state.selectedRowKeys.length; i++) {
    const index = state.selectedRowKeys[i];
    start_map(index);
  }
  message.success("启动成功", 1);
};

const stop_callback = () => {
  message.destroy();
  for (let i = 0; i < state.selectedRowKeys.length; i++) {
    const index = state.selectedRowKeys[i];
    stop_map(index);
  }
  message.info("服务已停止", 1);
};


const crb_disable=ref(false);
const create_callback =async () => {
  message.destroy();
  const test_config:TrConfig = {
    name: "test",
    password: "test",
    port_to_pub: 12345,
    protocol: "tcp",

    local_addr: "127.0.0.1",
    local_port: 5173,
    remote_host:"47.92.175.152",
    remote_port: 21212,
  };

  crb_disable.value = true;
  add_config(test_config);
  message.success("创建成功", 1);
};

</script>

<style scoped>
.col-one {
  height: 100%;
  width: 40%;
}
.class-two {
  height: 100%;
  width: 60%;
}
.outer-flex {
  height: 100%;
  width: 100%;
}
</style>
