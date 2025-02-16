pub const DEFAULT_INDEXER_PORT: u16 = 30000;
pub const DEFAULT_RPC_PORT: u16 = 30001;
pub const DEFAULT_FAUCET_PORT: u16 = 30002;
pub const DEFAULT_CONTROL_PORT: u16 = 30003;

/// Functions to define binding and client IPs
pub fn get_binding_ip(port: u16) -> String {
    format!("0.0.0.0:{port}")
}

pub fn get_client_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

pub fn get_indexer_binding_ip() -> String {
    get_binding_ip(DEFAULT_INDEXER_PORT)
}

pub fn get_rpc_binding_ip() -> String {
    get_binding_ip(DEFAULT_RPC_PORT)
}

pub fn get_faucet_binding_ip() -> String {
    get_binding_ip(DEFAULT_FAUCET_PORT)
}

pub fn get_control_binding_ip() -> String {
    get_binding_ip(DEFAULT_CONTROL_PORT)
}

pub fn get_indexer_client_url() -> String {
    get_client_url(DEFAULT_INDEXER_PORT)
}

pub fn get_rpc_client_url() -> String {
    get_client_url(DEFAULT_RPC_PORT)
}

pub fn get_faucet_url() -> String {
    get_client_url(DEFAULT_FAUCET_PORT)
}

pub fn get_control_url() -> String {
    get_client_url(DEFAULT_CONTROL_PORT)
}
