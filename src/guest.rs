// SPDX-FileCopyrightText: Copyright (c) 2666680 Ontario Inc. All rights reserved.
// SPDX-License-Identifier: GPL-2.0
//! This is the crate for handling different VMs running with
//! the GVM/LibVF.IO stack. The structure is permissive to allow
//! individuals to use partially proprietary blobs in their internal
//! stacks, while at the same time providing them the capability to
//! have longer running applications through the use of GVM/LibVF.IO.
//!
//! This codebase only offers 1 example of a workable plugin for the
//! use of GVM. Future plugins (such as LIME) will be created and open
//! sourced as time goes on.
//!
//! To provide support for an operating system please create a directory
//! and provide the following 4 functions:
//!
//! 1. init_net - Initializes a networking NIC that has been passed into
//!               the system. The list of networking NIC information will
//!               contain virtualized MAC address, IP to assign, gateway
//!               with cidr.
//! 2. init_communications - Due to the nature of rust, it is better to
//!                          implement this function in C as it allows
//!                          for proper file descriptor control.
//! 3. read_string - Reads a string from the host -> guest vm communication channel.
//! 4. write_command - Writes a command to the host from inside the guest.
extern crate dlopen;
#[macro_use]
extern crate dlopen_derive;

mod common;

// Linux specific imports.
#[cfg(target_os = "linux")]
mod linux;

use std::{ffi::CStr, ffi::CString, path::Path};
use std::os::raw::c_char;
use dlopen::wrapper::{Container, WrapperApi};

// Common imports for gvm-guest
use crate::common::{GVMError, GVMCmd, Command, Network, PluginMsg};
use std::collections::HashMap;
use std::result::Result;

#[cfg(target_os = "linux")]
use crate::linux::networking::init_net;
#[cfg(target_os="linux")]
use crate::linux::comms::{init_communications, read_string, write_command};

/// This API is exposed by shared library files on the guest in question.
/// We use this api to expose additional, potentially proprietary guest specific
/// APIs.
#[derive(WrapperApi)]
pub struct PluginApi {
    /// Plugin initialization code, it creates a persistent state in the library.
    ///
    /// NOTE: The return MUST be statically allocated string as it will NOT be freed.
    start : unsafe extern "C" fn() -> *const c_char,
    /// Processes a command through the plugin API.
    ///
    /// NOTE: The return MUST be dynamically allocated string as it will be freed.
    cmd_process : unsafe extern "C" fn (msg : *const c_char) -> *const c_char,
    /// Shuts down the persistent state in the library.
    ///
    /// NOTE: The return MUST be statically allocated string as it will NOT be freed.
    stop : unsafe extern "C" fn() -> *const c_char
}

fn main() -> Result<(), GVMError> {
    let mut plugins : HashMap<String, Container<PluginApi>> = HashMap::new();

    init_communications()?;

    write_command(Command { cmd : GVMCmd::GetNetwork, resp : None, finished : None})?;
    loop {
        let nets_res : Result<Vec<Network>, serde_json::Error> = serde_json::from_str(&read_string()?);

        if nets_res.is_err() {
            continue;
        }

        let nets = nets_res.unwrap();
        init_net(&nets)?;
        println!("Initialized nets: {:#?}", nets);
        break;
    }

    loop {
        let command_res : Result<PluginMsg, serde_json::Error> =
            serde_json::from_str(&read_string()?);

        if command_res.is_err() {
            continue;
        }

        let command = command_res.unwrap();
        let mut fin = false;
        let mut resp : Option<String> = None;

        match command.cmd {
            GVMCmd::CreatePluginLinks => {
                if !plugins.contains_key(&command.plugin) {
                    let name = &command.plugin;
                    if !Path::new(name).exists() {
                        println!("Got error: {:?}", GVMError::PluginNotFound);
                        continue;
                    }
                    plugins.insert(
                        name.to_string(),
                        unsafe {
                            Container::load(&name)
                        }.expect("Could not open library or load symbols")
                    );
                    fin = true;
                } else {
                    println!("Plugin already loaded");
                }
            },
            GVMCmd::StartPlugin => {
                if plugins.contains_key(&command.plugin) {
                    let c_buf : *const c_char = unsafe { plugins[&command.plugin].start() };
                    if !c_buf.is_null() {
                        let c_str : &CStr = unsafe { CStr::from_ptr(c_buf) };
                        let str_slice : &str = c_str.to_str().unwrap();
                        let str_buf : String = str_slice.to_owned();
                        resp = Some(str_buf);
                    }
                    fin = true;
                } else {
                    println!("Plugin not loaded");
                }
            },
            GVMCmd::PluginCmd => {
                if plugins.contains_key(&command.plugin) {
                    if command.msg.is_some() {
                        let cstr = CString::new(command.msg.unwrap()).unwrap();
                        let c_buf : *const c_char = unsafe {
                            plugins[&command.plugin].cmd_process(cstr.as_ptr())
                        };
                        if !c_buf.is_null() {
                            let c_str : &CStr = unsafe { CStr::from_ptr(c_buf) };
                            let str_slice : &str = c_str.to_str().unwrap();
                            let str_buf : String = str_slice.to_owned();
                            resp = Some(str_buf);
                        }
                        fin = true;
                    }
                } else {
                    println!("Plugin not loaded");
                }
            },
            GVMCmd::StopPlugin => {
                if plugins.contains_key(&command.plugin) {
                    unsafe { plugins[&command.plugin].stop() };
                    let c_buf : *const c_char = unsafe { plugins[&command.plugin].stop() };
                    if !c_buf.is_null() {
                        let c_str : &CStr = unsafe { CStr::from_ptr(c_buf) };
                        let str_slice : &str = c_str.to_str().unwrap();
                        let str_buf : String = str_slice.to_owned();
                        resp = Some(str_buf);
                    }
                    fin = true;
                } else {
                    println!("Plugin not loaded");
                }
            },
            GVMCmd::ShutdownGuest => {
                break;
            },
            _ => println!("Unsupported plugin command: {:#?}", command)
        };
        write_command(Command { cmd : command.cmd, resp : resp, finished : Some(fin) })?;
    }

    Ok(())
}
