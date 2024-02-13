import { createRouter, createWebHashHistory } from "vue-router";
import MainViewVue from "../views/MainView.vue";
import HomePageVue from "../components/HomePage.vue";
const router = createRouter({
    history: createWebHashHistory(),
    routes: [
        {
            path: "/",
            name: "mainview",
            component: MainViewVue,
            children:[
                {
                    path:"",
                    name:"home",
                    component:HomePageVue
                }
            ]
        },
    ],
});

export default router;
