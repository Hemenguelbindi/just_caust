use serde::Deserialize;
use std::fs;
use std::process::Command;

#[derive(Deserialize, Debug)]
struct VMConfig {
    virtual_machine: VirtualMachine,
}

#[derive(Deserialize, Debug)]
struct VirtualMachine {
    name: String,
    os_type: String,
    memory: u32,
    cpus: u32,
    disk_size: u32,
    iso_path: Option<String>,
}

fn load_config(path: &str) -> Result<VMConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read the config file: {}", e))?;
    let config: VMConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse TOML: {}", e))?;
    Ok(config)
}

fn create_vm(config: &VirtualMachine) -> Result<(), String> {
    // Create the VM
    Command::new("VBoxManage")
        .args(&["createvm", "--name", &config.name, "--ostype", &config.os_type, "--register"])
        .output()
        .map_err(|e| format!("Failed to create VM: {}", e))?;

    // Configure the VM
    Command::new("VBoxManage")
        .args(&[
            "modifyvm",
            &config.name,
            "--memory",
            &config.memory.to_string(),
            "--cpus",
            &config.cpus.to_string(),
            "--nic1",
            "nat",
        ])
        .output()
        .map_err(|e| format!("Failed to configure VM: {}", e))?;

    // Create and attach disk
    let disk_path = format!("~/VirtualBox VMs/{}/{}.vdi", &config.name, &config.name);
    Command::new("VBoxManage")
        .args(&["createhd", "--filename", &disk_path, "--size", &config.disk_size.to_string()])
        .output()
        .map_err(|e| format!("Failed to create disk: {}", e))?;
    Command::new("VBoxManage")
        .args(&["storagectl", &config.name, "--name", "SATA Controller", "--add", "sata", "--controller", "IntelAhci"])
        .output()
        .map_err(|e| format!("Failed to add storage controller: {}", e))?;
    Command::new("VBoxManage")
        .args(&[
            "storageattach",
            &config.name,
            "--storagectl",
            "SATA Controller",
            "--port",
            "0",
            "--device",
            "0",
            "--type",
            "hdd",
            "--medium",
            &disk_path,
        ])
        .output()
        .map_err(|e| format!("Failed to attach disk: {}", e))?;

    // Attach ISO if specified
    if let Some(iso_path) = &config.iso_path {
        Command::new("VBoxManage")
            .args(&["storagectl", &config.name, "--name", "IDE Controller", "--add", "ide"])
            .output()
            .map_err(|e| format!("Failed to add IDE controller: {}", e))?;
        Command::new("VBoxManage")
            .args(&[
                "storageattach",
                &config.name,
                "--storagectl",
                "IDE Controller",
                "--port",
                "0",
                "--device",
                "0",
                "--type",
                "dvddrive",
                "--medium",
                iso_path,
            ])
            .output()
            .map_err(|e| format!("Failed to attach ISO: {}", e))?;
    }

    println!("VM '{}' created successfully!", config.name);
    Ok(())
}

fn main() {
    let config_path = "vm_config.toml";

    // Load the configuration
    let config = match load_config(config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    // Create the VM
    if let Err(err) = create_vm(&config.virtual_machine) {
        eprintln!("Error: {}", err);
    }
}
