// This file contains the definitions and implementations related to OBD (On-Board Diagnostics) communication.
// It includes functions for sending and receiving OBD requests and responses.

#[cfg(target_os = "linux")]
pub mod request;
pub mod response;
