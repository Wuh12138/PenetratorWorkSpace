import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue";
import 'ant-design-vue/dist/reset.css';
import router from "./router";
import { createPinia } from "pinia";

const app=createApp(App);
app.use(router);
app.use(createPinia());

app.mount("#app");
