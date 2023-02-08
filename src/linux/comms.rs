// SPDX-FileCopyrightText: Copyright (c) 2666680 Ontario Inc. All rights reserved.
// SPDX-License-Identifier: GPL-3.0
//! This handles the low level host -> guest communications.
//!
//! NOTE: ALL OF THESE FUNCTIONS HAVE POTENTIALLY DANGEROUS SIDE EFFECTS.
use std::os::raw::c_char;
use std::result::Result;
use std::ffi::{CStr, CString};

use crate::common::{GVMError, Command};

extern "C" {
    /// Initializes the communication layer, this has a side effect of opening a long
    /// lasting file descriptor.
    fn init_comms() -> i32;
    /// This reads a string from the buffer, this cannot surpass 1024 characters at the
    /// moment.
    fn read_comms() -> *const c_char;
    /// This writes the string into the host communications.
    fn write_comms(str : *const c_char) -> i32;
}

/// Initializes the host -> guest communication line.
pub fn init_communications() -> Result<(), GVMError> {
    if unsafe { init_comms() } == 1 {
        return Ok(());
    } else {
        return Err(GVMError::IOError);
    }
}

/// Reads a string from the host and passes it to the main program.
pub fn read_string() -> Result<String, GVMError> {
    let c_buf : *const c_char = unsafe { read_comms() };
    let c_str : &CStr = unsafe { CStr::from_ptr(c_buf) };
    let str_slice : &str = c_str.to_str().unwrap();
    let str_buf : String = str_slice.to_owned();
    Ok(str_buf)
}

/// Converts a `cmd` into a command and than passes it into the host.
pub fn write_command(cmd : Command) -> Result<(), GVMError> {
    let s : String = serde_json::to_string(&cmd).unwrap().to_owned();
    let cs = CString::new(s).expect("CString::new failed");
    if unsafe {
        write_comms(cs.as_ptr())
    } == 1 {
        return Ok(());
    } else {
        return Err(GVMError::IOError);
    }
}
