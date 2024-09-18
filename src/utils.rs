use std::env;
use std::process;
pub fn check_bn() -> bool {
    let bn_value = env::var("BN").unwrap_or_else(|_| "0".to_string());
    bn_value == "1"
}

pub fn boot_node_address() -> String {
    if !check_bn() {
        match env::var("BNAD") {
            Ok(boot_node_address) => boot_node_address,
            Err(_) => "unset".to_string(),
        }
    } else {
        "BN is not set".to_string()
    }
}

pub fn get_own_address() -> String {
    let output = process::Command::new("hostname")
        .arg("-i")
        .output()
        .expect("failed to execute hostname command");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}
