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
      >
        创建
      </a-button>

      <a-button
        :style="{
          width: '50%',
          height: '50%',
        }"
        @click="delete_callback"
      >
        删除
      </a-button>
    </a-flex>

    <a-flex class="col-two">
      <a-table
        :columns="columns"
        :data-source="data"
        :row-selection="{
          selectedRowKeys: state.selectedRowKeys,
          onChange: onSelectChange,
        }"
        :style="{
          width: '100%',
          height: '100%',
        }"
      >
        <template #bodyCell="{ column, record }">
          <span v-if="column.key === 'status'">
            <a-tag v-if="record[column.key]" color="green">运行中</a-tag>
            <a-tag v-else color="red">已停止</a-tag>
          </span>
          <span v-else>{{ record[column.key] }}</span>
        </template>
      </a-table>
    </a-flex>
  </a-flex>

  <a-modal v-model:open="form_open" title="创建配置">
    <a-form :model="formData" >
      <a-form-item
        label="名称"
        name="name"
        :rules="[{ required: true, message: '请输入名称' }]"
      >
        <a-input v-model:value="formData.name" placeholder="名称" />
      </a-form-item>
      <a-form-item
        label="密码"
        name="password"
        :rules="[{ required: true, message: '请输入密码' }]"
      >
        <a-input
          v-model:value="formData.password"
          placeholder="密码"
          type="password"
        />
      </a-form-item>
    </a-form>

    <a-form-item
      label="公网端口"
      name="port_to_pub"
      :rules="[{ required: true, message: '请输入端口' }]"
    >
      <a-input-number v-model:value="formData.port_to_pub" />
    </a-form-item>

    <a-form-item
      label="协议"
      name="protocol"
      :rules="[{ required: true, message: '请输入协议' }]"
    >
      <a-input v-model:value="formData.protocol" />
    </a-form-item>

    <a-form-item
      label="本地地址"
      name="local_addr"
      :rules="[{ required: true, message: '请输入本地地址' }]"
    >
      <a-input v-model:value="formData.local_addr" />
    </a-form-item>

    <a-form-item
      label="本地端口"
      name="local_port"
      :rules="[{ required: true, message: '请输入本地端口' }]"
    >
      <a-input-number v-model:value="formData.local_port" />
    </a-form-item>

    <a-form-item
      label="远程地址"
      name="remote_host"
      :rules="[{ required: true, message: '请输入远程地址' }]"
    >
      <a-input v-model:value="formData.remote_host" />
    </a-form-item>

    <a-form-item
      label="远程端口"
      name="remote_port"
      :rules="[{ required: true, message: '请输入远程端口' }]"
    >
      <a-input-number v-model:value="formData.remote_port" />
    </a-form-item>

    <a-button type="primary" @click="create_finish">创建</a-button>
  </a-modal>
</template>

<script setup lang="ts">
import { message } from "ant-design-vue";
import { reactive, ref } from "vue";
import { useStatusStore } from "../store/StatusList";
import { storeToRefs } from "pinia";
import { TrConfig } from "../command";

const store = useStatusStore();
const { columns, data } = storeToRefs(store);
const { updateDataStatus, initDate, add_config, start_map, stop_map,remove_config } = store;

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
  state.selectedRowKeys = [];
};

const stop_callback = () => {
  message.destroy();
  for (let i = 0; i < state.selectedRowKeys.length; i++) {
    const index = state.selectedRowKeys[i];
    stop_map(index);
  }
  message.info("服务已停止", 1);
  state.selectedRowKeys = [];
};

const formData: TrConfig = reactive({
  name: "",
  password: "",
  port_to_pub: 0,
  protocol: "",

  local_addr: "",
  local_port: 0,
  remote_host: "",
  remote_port: 0,
});

const form_open = ref(false);
const create_callback = async () => {
  message.destroy();
  form_open.value = true;
};
const create_finish=()=>{
  const config=Object.assign({},formData);
  add_config(config);
  form_open.value = false;
  formData.name = "";
  formData.password = "";
  formData.port_to_pub = 0;
  formData.protocol = "";
  formData.local_addr = "";
  formData.local_port = 0;
  formData.remote_host = "";
  formData.remote_port = 0;
  message.success("创建成功", 1);
}

const delete_callback=()=>{
  message.destroy();
  for (let i = 0; i < state.selectedRowKeys.length; i++) {
    const index = state.selectedRowKeys[i];
    stop_map(index);
    remove_config(index);
  }
  message.success("删除成功", 1);
  state.selectedRowKeys = [];
}

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
