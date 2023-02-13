// SPDX-FileCopyrightText: Copyright (c) 2666680 Ontario Inc. All rights reserved.
// SPDX-License-Identifier: GPL-2.0
//! This is the commonly used types inside the GVM guest program.
//!
//! The host can only send 2 types of messages into the GVM guest program,
//! the first is a [Network] vector, and the second is a [PluginMsg].
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;

/// GVM specific errors that can be run into in the program.
#[derive(Debug)]
pub enum GVMError {
    /// IO Error related to file/host device control.
    IOError,
    /// NIC was requested by the host but not found in the guest.
    NicNotFound,
    /// The plugin is not found.
    PluginNotFound,
    /// Plugin was already loaded.
    PluginLoaded,
    /// Plugin command was not supported by GVM Guest.
    PluginCommandNotSupported,
}

impl fmt::Display for GVMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GVMError::IOError => write!(f, "IOError"),
            GVMError::NicNotFound => write!(f, "NicNotFound"),
            GVMError::PluginNotFound => write!(f, "PluginNotFound"),
            GVMError::PluginLoaded => write!(f, "PluginLoaded"),
            GVMError::PluginCommandNotSupported => write!(f, "PluginCommandNotSupported"),
        }
    }
}

impl From<io::Error> for GVMError {
    fn from(_: io::Error) -> GVMError {
        GVMError::IOError
    }
}

/// Possible commands available inside the GVM Guest.
#[derive(Serialize, Deserialize, Debug)]
pub enum GVMCmd {
    /// Gets a list of networking macs/ips/and gateways.
    GetNetwork,
    /// Issues the start command for the plugin in question.
    StartPlugin,
    /// Creates a plugin link this command needs to be sent before a start plugin command.
    CreatePluginLinks,
    /// Command that should be forwarded to the plugin.
    PluginCmd,
    /// Stops the plugin from running.
    StopPlugin,
    /// Shuts down the guest program, eventually this will also shut down the system.
    ShutdownGuest,
}

/// Command to be sent from guest to the host.
#[derive(Serialize, Debug)]
pub struct Command {
    /// The command we are working with.
    pub cmd: GVMCmd,
    /// This is the response for the different commands sent in/out of the guest.
    pub resp: Option<String>,
    /// This should be None when we initiate the command from the guest, and a success
    /// or failure otherwise.
    pub finished: Option<bool>,
}

/// Networking structure to add to the system.
#[derive(Deserialize, Debug)]
pub struct Network {
    /// MAC address of the NIC passed into the guest.
    pub mac: String,
    /// IP address to assign to the NIC.
    pub ip: String,
    /// Gateway in the form of gateway-ip/cidr
    pub gateway: String,
}

/// Control of GVM guest utility message.
#[derive(Deserialize, Debug)]
pub struct PluginMsg {
    /// Command to run on the plugin system.
    pub cmd: GVMCmd,
    /// Plugin name to execute on, it is recommended to use absolute path name.
    pub plugin: String,
    /// Message field is ONLY allowed during [GVMCmd::PluginCmd] commands.
    pub msg: Option<String>,
}
