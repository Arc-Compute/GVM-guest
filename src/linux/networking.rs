// SPDX-FileCopyrightText: Copyright (c) 2666680 Ontario Inc. All rights reserved.
// SPDX-License-Identifier: GPL-3.0
//! This code is specific to the networking NIC section of the VM.
//!
//! The procedure for adding in a networking NIC is as follows:
//!
//! 1. Find corresponding networking device associated with the passed in MAC address.
//! 2. Determine if we are on a netplan or systemd backed system.
//! 3. Create backend specific configurations.
//! 4. Apply changes for backend specifically.
use crate::common::{GVMError, Network};
use std::fs;
use std::result::Result;
use std::path::Path;
use std::process::Command;
use uuid::Uuid;

/// This function iterates through the /sys/class/net devices and searches for the `mac`
/// inside the address field for the device. The name of the device is sent back to us once
/// we find a match.
fn find_mac(mac : &String) -> Result<String, GVMError> {
    let start_dir = "/sys/class/net/";
    let mut ret : Option<String> = None;

    for entry in fs::read_dir(start_dir).unwrap() {
        let entry = entry?;
        let path : &str = &entry.file_name().into_string().unwrap();

        let prev_contents =
            fs::read_to_string(start_dir.to_owned() + path + "/address")?;
        let contents = prev_contents.strip_suffix("\n").unwrap_or(&prev_contents);

        println!(
            "NIC: {}, MAC: {}",
            path,
            contents
        );

        if contents == mac {
            ret = Some(path.to_owned());
            break;
        }
    }

    if ret.is_none() {
        return Err(GVMError::NicNotFound);
    }

    Ok(ret.unwrap())
}

/// This function is to provide for us the incremental configuration for the
/// valid `net` device inside the GVM guest program.
fn netplan_networking(net : &Network) -> Result<String, GVMError> {
    let nic = find_mac(&net.mac)?;
    let gate_cidr : Vec<&str> = net.gateway.split('/').collect();

    let ret = "".to_owned() +
        "    " + &nic + ":\n" +
        "      dhcp4: false\n" +
        "      addresses:\n" +
        "        - " + &net.ip + "/" + gate_cidr[1] +"\n" +
        "      gateway4: " + gate_cidr[0] + "\n" +
        "      nameservers:\n" +
        "        addresses: [8.8.8.8]";

    Ok(ret)
}

/// This function configures the specific NIC network script inside
/// /etc/sysconfig/network-scripts to handle systemd networking control
/// correctly for a given `net`.
fn systemd_networking(net : &Network) -> Result<(), GVMError> {
    let nic = find_mac(&net.mac)?;
    let uuid = Uuid::new_v4();
    let file_name = "/etc/sysconfig/network-scripts/".to_owned() + "ifcfg-" + &nic;
    let gate_cidr : Vec<&str> = net.gateway.split('/').collect();
    let gateway = gate_cidr[0];
    let cidr = gate_cidr[1].parse::<u32>().unwrap();

    // Magic algorithm for CIDR calculation, don't touch now.
    let netmask_og : u32 = ((((1 as u64) << (32 as u64)) - 1) as u32) << (32 - cidr);
    let netmask_1 : u32 = netmask_og & 0x000000FF;
    let netmask_2 : u32 = (netmask_og & 0x0000FF00) >> 8;
    let netmask_3 : u32 = (netmask_og & 0x00FF0000) >> 16;
    let netmask_4 : u32 = (netmask_og & 0xFF000000) >> 24;
    let netmask = format!("{}.{}.{}.{}", netmask_4, netmask_3, netmask_2, netmask_1);

    println!("Using nic: {} -> {}", nic, uuid);

    let contents = "".to_owned() +
        "HWADDR=" + &net.mac + "\n" +
        "TYPE=Ethernet\n" +
        "BOOTPROTO=none\n" +
        "DEFROUTE=yes\n" +
        "NETMASK=" + &netmask + "\n" +
        "GATEWAY=" + &gateway + "\n" +
        "DNS1=8.8.8.8\n" +
        "DNS2=8.8.4.4\n" +
        "IPADDR=" + &net.ip + "\n" +
        "IPV4_FAILURE_FATAL=no\n" +
        "NAME=" + &nic + "\n" +
        "UUID=" + &uuid.to_string() + "\n" +
        "DEVICE=" + &nic + "\n" +
        "ONBOOT=yes\n" +
        "IPV6INIT=no";

    fs::write(file_name, contents).unwrap();

    Ok(())
}

/// This function is given a vector of network devices and initializes each of them either
/// using netplan or by using systemd.
pub fn init_net(nets: &Vec<Network>) -> Result<(), GVMError> {
    println!("Initializing network");

    let nets_len = nets.len();
    let netplan: bool = Path::new("/etc/netplan").is_dir();
    let file_name = "/etc/netplan/00-installer-config.yaml";
    let mut contents = "network:\n  ethernets:".to_owned();

    if netplan {
       println!("Using netplan");
    } else {
        println!("Using systemd networking");
    }

    for net in nets {
        println!("Adding {:#?}", net);
        if netplan {
            contents = contents + "\n" + &netplan_networking(net)?;
        } else {
            systemd_networking(net)?;
        }
    }

    if netplan && nets_len > 0{
        contents = contents + "\n" + "  version: 2\n";
        fs::write(file_name, contents).unwrap();
        Command::new("/bin/sudo").args(["netplan", "apply"]).output().unwrap();
    } else if nets_len > 0 {
        Command::new("/bin/sudo").args(["systemctl", "restart", "network"]).output().unwrap();
    }

    Ok(())
}
