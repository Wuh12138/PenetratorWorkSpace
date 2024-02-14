import { invoke } from "@tauri-apps/api";

export interface TrConfig {
    name: string,
    password: string,
    port_to_pub: number,
    protocol: string,

    local_addr: string,
    local_port: number
    remote_host: string,
    remote_port: number
}

export async function start_a_map(config: TrConfig):Promise<number> {
    return await invoke<number>("start_a_map", {config:config}).catch((e) => {
        console.error(e);
        return -1;
    });
}
export async function stop_a_map(uid: number) {
    return await invoke("stop_a_map", {uid:uid}).catch((e) => {
        console.error(e);
    });
}

export async function update_config_list(newConfigList: TrConfig[]){
    return await invoke("update_config_list", {newConfigList:newConfigList}).catch((e) => {
        console.error(e);
    });
}

export async function get_config_list():Promise<TrConfig[]> {
    return await invoke<TrConfig[]>("get_config_list").catch((e) => {
        console.error(e);
        return [];
    });
}


export interface TrItemInfo {
    uid: number,
    local_addr: string,
    local_port: number,
    remote_host: string,
    remote_port: number,
    protocol: string,
}


export async function get_running_item():Promise<TrItemInfo[]> {
    return await invoke<TrItemInfo[]>("get_running_item").catch((e) => {
        console.error(e);
        return [];
    });
}