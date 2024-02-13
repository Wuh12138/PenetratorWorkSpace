import { invoke } from "@tauri-apps/api";

export interface TrConfig {
    name: String,
    password: String,
    port_to_pub: Number,
    protocol: String,

    local_addr: String,
    local_port: Number
    remote_host: String,
    remote_port: Number
}

export async function create_config(config: TrConfig) {
    await invoke("new_config", {config: config}).catch((e) => {
        console.error(e);
    });
}