//! Memory reading library for Windows processes
//! Provides process enumeration, memory reading, and pointer chain traversal

use std::mem::zeroed;
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, MAX_PATH, BOOL};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, Process32FirstW, Process32NextW,
    MODULEENTRY32W, PROCESSENTRY32W, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    IsWow64Process, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};

/// Process information for display
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub exe_path: String,
    pub is_64bit: bool,
}

/// Module information
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleInfo {
    pub name: String,
    pub base_address: u64,
    pub size: u32,
}

/// Result of a memory traversal
#[derive(Clone, Debug)]
pub struct TraversalResult {
    pub offset_path: String,
    pub actual_address: u64,
    pub value_i8: i8,
    pub value_i16: i16,
    pub value_i32: i32,
    pub value_i64: i64,
    pub value_u8: u8,
    pub value_u16: u16,
    pub value_u32: u32,
    pub value_u64: u64,
    pub value_f32: f32,
    pub value_f64: f64,
    pub valid: bool,
}

impl Default for TraversalResult {
    fn default() -> Self {
        Self {
            offset_path: String::new(),
            actual_address: 0,
            value_i8: 0,
            value_i16: 0,
            value_i32: 0,
            value_i64: 0,
            value_u8: 0,
            value_u16: 0,
            value_u32: 0,
            value_u64: 0,
            value_f32: 0.0,
            value_f64: 0.0,
            valid: false,
        }
    }
}

/// Process handle wrapper for memory operations
pub struct ProcessHandle {
    handle: HANDLE,
    pub is_64bit: bool,
}

impl ProcessHandle {
    /// Open a process for reading memory
    pub fn open(pid: u32) -> Option<Self> {
        unsafe {
            let handle = OpenProcess(
                PROCESS_VM_READ | PROCESS_QUERY_LIMITED_INFORMATION,
                false,
                pid,
            )
            .ok()?;

            let mut is_wow64: BOOL = BOOL(0);
            let _ = IsWow64Process(handle, &mut is_wow64);
            let is_64bit = !is_wow64.as_bool();

            Some(Self { handle, is_64bit })
        }
    }

    /// Read memory at the specified address
    pub fn read_memory(&self, address: u64, buffer: &mut [u8]) -> bool {
        unsafe {
            let mut bytes_read = 0usize;
            ReadProcessMemory(
                self.handle,
                address as *const _,
                buffer.as_mut_ptr() as *mut _,
                buffer.len(),
                Some(&mut bytes_read),
            )
            .is_ok()
                && bytes_read == buffer.len()
        }
    }

    /// Read a pointer (4 bytes for 32-bit, 8 bytes for 64-bit)
    pub fn read_pointer(&self, address: u64) -> Option<u64> {
        if self.is_64bit {
            let mut buffer = [0u8; 8];
            if self.read_memory(address, &mut buffer) {
                Some(u64::from_le_bytes(buffer))
            } else {
                None
            }
        } else {
            let mut buffer = [0u8; 4];
            if self.read_memory(address, &mut buffer) {
                Some(u32::from_le_bytes(buffer) as u64)
            } else {
                None
            }
        }
    }

    /// Traverse a pointer chain and return the final address
    /// base_address: starting address
    /// offsets: list of offsets to follow
    /// Returns the final address after following all pointers
    pub fn traverse_pointer_chain(&self, base_address: u64, offsets: &[i64]) -> Option<u64> {
        if offsets.is_empty() {
            return Some(base_address);
        }

        let mut current_address = base_address;

        for (i, &offset) in offsets.iter().enumerate() {
            // For all but the last offset, read the pointer at current address
            if i < offsets.len() - 1 {
                current_address = self.read_pointer(current_address)?;
            }
            // Apply the offset
            current_address = (current_address as i64 + offset) as u64;
        }

        Some(current_address)
    }

    /// Read all data types at the specified address
    pub fn read_all_types(&self, address: u64) -> TraversalResult {
        let mut result = TraversalResult {
            actual_address: address,
            ..Default::default()
        };

        // Read 8 bytes to cover all types
        let mut buffer = [0u8; 8];
        if !self.read_memory(address, &mut buffer) {
            return result;
        }

        result.valid = true;
        result.value_i8 = buffer[0] as i8;
        result.value_u8 = buffer[0];
        result.value_i16 = i16::from_le_bytes([buffer[0], buffer[1]]);
        result.value_u16 = u16::from_le_bytes([buffer[0], buffer[1]]);
        result.value_i32 = i32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        result.value_u32 = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        result.value_i64 = i64::from_le_bytes(buffer);
        result.value_u64 = u64::from_le_bytes(buffer);
        result.value_f32 = f32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        result.value_f64 = f64::from_le_bytes(buffer);

        result
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

/// Get list of running processes
pub fn get_processes() -> Vec<ProcessInfo> {
    let mut processes = Vec::new();

    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(handle) => handle,
            Err(_) => return processes,
        };

        let mut entry: PROCESSENTRY32W = zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szExeFile[..entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len())],
                );

                let (exe_path, is_64bit) = get_process_details(entry.th32ProcessID);

                processes.push(ProcessInfo {
                    pid: entry.th32ProcessID,
                    name,
                    exe_path,
                    is_64bit,
                });

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }

    // Sort by name
    processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    processes
}

/// Get process details (exe path and architecture)
fn get_process_details(pid: u32) -> (String, bool) {
    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return (String::new(), true),
        };

        // Get architecture
        let mut is_wow64: BOOL = BOOL(0);
        let _ = IsWow64Process(handle, &mut is_wow64);
        let is_64bit = !is_wow64.as_bool();

        // Get executable path
        let mut path_buf = [0u16; MAX_PATH as usize];
        let mut size = path_buf.len() as u32;
        let exe_path = if QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(path_buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
        {
            String::from_utf16_lossy(&path_buf[..size as usize])
        } else {
            String::new()
        };

        let _ = CloseHandle(handle);
        (exe_path, is_64bit)
    }
}

/// Get modules for a process
pub fn get_process_modules(pid: u32) -> Vec<ModuleInfo> {
    let mut modules = Vec::new();

    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)
        {
            Ok(handle) => handle,
            Err(_) => return modules,
        };

        let mut entry: MODULEENTRY32W = zeroed();
        entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;

        if Module32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szModule[..entry
                        .szModule
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szModule.len())],
                );

                modules.push(ModuleInfo {
                    name,
                    base_address: entry.modBaseAddr as u64,
                    size: entry.modBaseSize,
                });

                if Module32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }

    modules
}

/// Parse offset string like "0x3C4+0xC8+0x10" or "0x3C4,0xC8,0x10" into a vector of offsets
pub fn parse_offsets(input: &str) -> Result<Vec<i64>, String> {
    if input.trim().is_empty() {
        return Ok(vec![]);
    }

    let separators = ['+', ',', ' '];
    let parts: Vec<&str> = input
        .split(|c| separators.contains(&c))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut offsets = Vec::new();
    for part in parts {
        let offset = parse_hex_or_dec(part)?;
        offsets.push(offset);
    }

    Ok(offsets)
}

/// Parse a hex (0x...) or decimal number, supporting negative values
pub fn parse_hex_or_dec(input: &str) -> Result<i64, String> {
    let input = input.trim();
    let (negative, input) = if input.starts_with('-') {
        (true, &input[1..])
    } else {
        (false, input)
    };

    let value = if input.starts_with("0x") || input.starts_with("0X") {
        i64::from_str_radix(&input[2..], 16).map_err(|e| format!("Invalid hex: {}", e))?
    } else {
        input
            .parse::<i64>()
            .map_err(|e| format!("Invalid number: {}", e))?
    };

    Ok(if negative { -value } else { value })
}

/// Parse base address (hex string like "1599610" or "0x1599610")
pub fn parse_base_address(input: &str) -> Result<u64, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty address".to_string());
    }

    if input.starts_with("0x") || input.starts_with("0X") {
        u64::from_str_radix(&input[2..], 16).map_err(|e| format!("Invalid hex address: {}", e))
    } else {
        // Try hex first, then decimal
        u64::from_str_radix(input, 16)
            .or_else(|_| input.parse::<u64>())
            .map_err(|e| format!("Invalid address: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_offsets() {
        assert_eq!(parse_offsets("0x3C4+0xC8+0x10").unwrap(), vec![0x3C4, 0xC8, 0x10]);
        assert_eq!(parse_offsets("0x3C4,0xC8,0x10").unwrap(), vec![0x3C4, 0xC8, 0x10]);
        assert_eq!(parse_offsets("100+200").unwrap(), vec![100, 200]);
        assert_eq!(parse_offsets("-0x10+0x20").unwrap(), vec![-0x10, 0x20]);
    }

    #[test]
    fn test_parse_base_address() {
        assert_eq!(parse_base_address("1599610").unwrap(), 0x1599610);
        assert_eq!(parse_base_address("0x1599610").unwrap(), 0x1599610);
    }
}
