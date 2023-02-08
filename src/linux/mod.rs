//! This is the linux specific component of GVM guest programs.
//!
//! 1. init_net - Implemented inside the networking module, and supports both systemd and
//!               netplan backed networking stacks.
//! 2. init_communications - This is implemented inside the comms module, and uses a mutable
//!                          C module.
//! 3. read_string, write_command - These are implemented inside the comms module and uses
//!                                 a mutable C module.
pub mod comms;
pub mod networking;
