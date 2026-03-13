//! Memory reading library for Windows processes
//! Provides process enumeration, memory reading, and pointer chain traversal

use serde::{Deserialize, Serialize};
use std::mem::zeroed;
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, MAX_PATH, BOOL};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, Process32FirstW, Process32NextW,
    MODULEENTRY32W, PROCESSENTRY32W, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::Threading::{
    IsWow64Process, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION,
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

/// Result of entity list scan
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EntityResult {
    pub index: usize,
    pub address: u64,
    pub values: Vec<(String, i64)>,
    pub valid: bool,
}

/// Result of a memory traversal
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraversalResult {
    pub label: String,
    pub offset_path: String,
    pub selected_type: String,
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

/// Saved scan configuration and results for Memory Traversal
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedMemoryScan {
    pub base_address: String,
    pub offsets: String,
    pub loop_count: u32,
    pub step_size: u32,
    pub field_names: String,
    pub results: Vec<TraversalResult>,
}

/// Saved scan configuration and results for Entity List
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedEntityScan {
    pub mode: String,
    pub base_address: String,
    pub struct_size: String,
    pub max_count: usize,
    pub value_offsets: String,
    pub data_type: String,
    pub results: Vec<EntityResult>,
}

impl Default for TraversalResult {
    fn default() -> Self {
        Self {
            label: String::new(),
            offset_path: String::new(),
            selected_type: "i32".to_string(),
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
                PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_LIMITED_INFORMATION,
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

    /// Write memory at the specified address
    pub fn write_memory(&self, address: u64, buffer: &[u8]) -> bool {
        unsafe {
            let mut bytes_written = 0usize;
            WriteProcessMemory(
                self.handle,
                address as *const _,
                buffer.as_ptr() as *const _,
                buffer.len(),
                Some(&mut bytes_written),
            )
            .is_ok()
                && bytes_written == buffer.len()
        }
    }

    /// Write an i8 value
    pub fn write_i8(&self, address: u64, value: i8) -> bool {
        self.write_memory(address, &value.to_le_bytes())
    }

    /// Write an i16 value
    pub fn write_i16(&self, address: u64, value: i16) -> bool {
        self.write_memory(address, &value.to_le_bytes())
    }

    /// Write an i32 value
    pub fn write_i32(&self, address: u64, value: i32) -> bool {
        self.write_memory(address, &value.to_le_bytes())
    }

    /// Write an i64 value
    pub fn write_i64(&self, address: u64, value: i64) -> bool {
        self.write_memory(address, &value.to_le_bytes())
    }

    /// Write a f32 value
    pub fn write_f32(&self, address: u64, value: f32) -> bool {
        self.write_memory(address, &value.to_le_bytes())
    }

    /// Write a f64 value
    pub fn write_f64(&self, address: u64, value: f64) -> bool {
        self.write_memory(address, &value.to_le_bytes())
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
    /// base_address: starting address (address containing a pointer)
    /// offsets: list of offsets to follow
    /// 
    /// For chain [[[base]+0x0]+0x40]+0x20]+0x0 with base=&g_pGameWorld:
    /// 1. Read pointer at base → GameWorld addr
    /// 2. Add 0x0 → GameWorld+0x0, read pointer → still GameWorld (0x0 offset)
    /// 3. Add 0x40 → GameWorld+0x40, read pointer → Player addr
    /// 4. Add 0x20 → Player+0x20, read pointer → Stats addr
    /// 5. Add 0x0 → Stats+0x0 = health address (no dereference)
    /// 
    /// Number of dereferences = number of offsets (last one doesn't dereference)
    pub fn traverse_pointer_chain(&self, base_address: u64, offsets: &[i64]) -> Option<u64> {
        if offsets.is_empty() {
            // No offsets, just read pointer at base and return that address
            return self.read_pointer(base_address);
        }

        let mut current_address = base_address;

        for (_i, &offset) in offsets.iter().enumerate() {
            // Read pointer at current address
            current_address = self.read_pointer(current_address)?;
            
            // Apply the offset
            current_address = (current_address as i64 + offset) as u64;
            
            // Don't dereference after the last offset - that's our target address
            // The dereference happens at the START of each iteration
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

/// Parse offset string in multiple formats:
/// - Simple: "0x3C4+0xC8+0x10" or "0x3C4,0xC8,0x10"
/// - Bracket (x64dbg style): "[[[base]+0x40]+0x20]+0x0" where [xxx] means pointer dereference
pub fn parse_offsets(input: &str) -> Result<Vec<i64>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(vec![]);
    }

    // Check if it's bracket notation (contains '[')
    if input.contains('[') {
        return parse_bracket_notation(input);
    }

    // Simple format: offsets separated by + , or space
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

/// Parse x64dbg-style bracket notation: [[[base]+0x40]+0x20]+0x0
/// Each [...] indicates reading a pointer at that address
/// Returns the list of offsets to apply
fn parse_bracket_notation(input: &str) -> Result<Vec<i64>, String> {
    let mut offsets = Vec::new();
    let mut current = input.trim();
    
    // Remove leading brackets and "base" keyword if present
    // Format: [[[base]+0x40]+0x20]+0x0
    // We extract offsets: 0x40, 0x20, 0x0
    
    // Count and strip leading brackets
    while current.starts_with('[') {
        current = &current[1..];
    }
    
    // Remove "base" or any initial text before first ]
    if let Some(pos) = current.find(']') {
        // Check if there's an offset before the bracket
        let before_bracket = &current[..pos];
        // Skip "base" keyword or empty content
        if !before_bracket.eq_ignore_ascii_case("base") && !before_bracket.is_empty() {
            // It might be an offset like "0x0" in [[0x0]+...]
            if let Ok(off) = parse_hex_or_dec(before_bracket) {
                offsets.push(off);
            }
        }
        current = &current[pos..];
    }
    
    // Now parse the rest: ]+0x40]+0x20]+0x0
    // Split by ']' and extract offsets after '+'
    for part in current.split(']') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        
        // Part should be like "+0x40" or "+0x20"
        let offset_str = part.trim_start_matches('+').trim();
        if offset_str.is_empty() {
            continue;
        }
        
        match parse_hex_or_dec(offset_str) {
            Ok(off) => offsets.push(off),
            Err(_) => {
                // Skip non-numeric parts (like trailing text)
                continue;
            }
        }
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

// Re-export serde_json for UI crate
pub use serde_json;

/// Save memory scan results to JSON file
pub fn save_memory_scan(path: &str, scan: &SavedMemoryScan) -> Result<(), String> {
    let json = serde_json::to_string_pretty(scan).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

/// Load memory scan results from JSON file
pub fn load_memory_scan(path: &str) -> Result<SavedMemoryScan, String> {
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

/// Save entity scan results to JSON file
pub fn save_entity_scan(path: &str, scan: &SavedEntityScan) -> Result<(), String> {
    let json = serde_json::to_string_pretty(scan).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

/// Load entity scan results from JSON file
pub fn load_entity_scan(path: &str) -> Result<SavedEntityScan, String> {
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

/// Get C++ type name from selected_type
fn cpp_type_name(t: &str) -> &'static str {
    match t {
        "i8" => "int8_t",
        "i16" => "int16_t",
        "i32" => "int32_t",
        "i64" => "int64_t",
        "u8" => "uint8_t",
        "u16" => "uint16_t",
        "u32" => "uint32_t",
        "u64" => "uint64_t",
        "f32" => "float",
        "f64" => "double",
        _ => "int32_t",
    }
}

/// Get type size in bytes
fn type_size(t: &str) -> u64 {
    match t {
        "i8" | "u8" => 1,
        "i16" | "u16" => 2,
        "i32" | "u32" | "f32" => 4,
        "i64" | "u64" | "f64" => 8,
        _ => 4,
    }
}

/// Sanitize label for C++ variable name
fn sanitize_name(label: &str, offset: u64) -> String {
    if label.is_empty() {
        return format!("field_{:#X}", offset);
    }
    let sanitized: String = label
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if sanitized.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
        format!("_{}", sanitized)
    } else {
        sanitized
    }
}

/// Export results to C++ header file
pub fn export_to_cpp(results: &[TraversalResult], class_name: &str, base_address: &str, offsets: &str) -> String {
    let mut output = String::new();
    
    // Parse base address to extract the offset portion (lower bits that would be module-relative)
    let base_addr = parse_base_address(base_address).unwrap_or(0);
    // Assume module base is aligned to 0x10000, extract the offset
    let base_offset = base_addr & 0xFFFF; // Lower 16 bits as module offset
    
    // Header guard and includes
    output.push_str(&format!("// Generated by Damned Memory Traversal Tool\n"));
    output.push_str(&format!("// Base: {} Offsets: {}\n", base_address, offsets));
    output.push_str(&format!("// Base offset from module: {:#X}\n\n", base_offset));
    output.push_str("#pragma once\n");
    output.push_str("#include <cstdint>\n");
    output.push_str("#include <vector>\n\n");
    
    // Define base offset constant
    output.push_str(&format!("constexpr uintptr_t BASE_OFFSET = {:#X};\n\n", base_offset));
    
    // Generate struct
    output.push_str(&format!("// Struct layout (use with direct memory access)\n"));
    output.push_str(&format!("struct {} {{\n", class_name));
    
    let mut last_offset: u64 = 0;
    let mut pad_count = 0;
    
    for result in results {
        // Extract final offset from offset_path (last number after last +)
        let final_offset = extract_final_offset(&result.offset_path);
        
        // Add padding if needed
        if final_offset > last_offset {
            let pad_size = final_offset - last_offset;
            output.push_str(&format!("    char _pad{}[{:#X}];\n", pad_count, pad_size));
            pad_count += 1;
        }
        
        let var_name = sanitize_name(&result.label, final_offset);
        let cpp_type = cpp_type_name(&result.selected_type);
        output.push_str(&format!("    {} {}; // {:#X}\n", cpp_type, var_name, final_offset));
        
        last_offset = final_offset + type_size(&result.selected_type);
    }
    
    output.push_str("};\n\n");
    
    // Generate pointer chain helper class
    output.push_str("// Pointer chain helper (use with ReadProcessMemory/WriteProcessMemory)\n");
    output.push_str(&format!("class {}Reader {{\n", class_name));
    output.push_str("    uintptr_t moduleBase;\n");
    output.push_str("    \n");
    output.push_str("    template<typename T>\n");
    output.push_str("    T Read(uintptr_t addr) {\n");
    output.push_str("        T val{};\n");
    output.push_str("        // ReadProcessMemory(hProcess, (LPCVOID)addr, &val, sizeof(T), nullptr);\n");
    output.push_str("        return val;\n");
    output.push_str("    }\n");
    output.push_str("    \n");
    output.push_str("    template<typename T>\n");
    output.push_str("    void Write(uintptr_t addr, T val) {\n");
    output.push_str("        // WriteProcessMemory(hProcess, (LPVOID)addr, &val, sizeof(T), nullptr);\n");
    output.push_str("    }\n");
    output.push_str("    \n");
    output.push_str("    uintptr_t FollowChain(uintptr_t base, std::vector<int64_t> offsets) {\n");
    output.push_str("        uintptr_t addr = base;\n");
    output.push_str("        for (size_t i = 0; i < offsets.size() - 1; i++) {\n");
    output.push_str("            addr = Read<uintptr_t>(addr + offsets[i]);\n");
    output.push_str("            if (addr == 0) return 0;\n");
    output.push_str("        }\n");
    output.push_str("        return addr + offsets.back();\n");
    output.push_str("    }\n");
    output.push_str("    \n");
    output.push_str("public:\n");
    output.push_str(&format!("    {}Reader(uintptr_t base) : moduleBase(base) {{}}\n", class_name));
    output.push_str("    \n");
    
    // Generate getter/setter for each field
    for result in results {
        let final_offset = extract_final_offset(&result.offset_path);
        let var_name = sanitize_name(&result.label, final_offset);
        let cpp_type = cpp_type_name(&result.selected_type);
        let offsets_vec = extract_offsets_from_path(&result.offset_path);
        
        // Getter - use moduleBase + BASE_OFFSET
        output.push_str(&format!("    {} get{}() {{\n", cpp_type, capitalize(&var_name)));
        output.push_str(&format!("        return Read<{}>(FollowChain(moduleBase + BASE_OFFSET, {{{}}}));\n", 
            cpp_type, offsets_vec));
        output.push_str("    }\n");
        
        // Setter - use moduleBase + BASE_OFFSET
        output.push_str(&format!("    void set{}({} val) {{\n", capitalize(&var_name), cpp_type));
        output.push_str(&format!("        Write<{}>(FollowChain(moduleBase + BASE_OFFSET, {{{}}}), val);\n", 
            cpp_type, offsets_vec));
        output.push_str("    }\n");
        output.push_str("    \n");
    }
    
    output.push_str("};\n");
    
    output
}

/// Extract final offset from offset_path like "[[base]+0x40]+0x20]+0x100"
fn extract_final_offset(path: &str) -> u64 {
    // Find the last +0x... pattern
    let parts: Vec<&str> = path.split('+').collect();
    if let Some(last) = parts.last() {
        let cleaned = last.trim().trim_end_matches(']');
        parse_base_address(cleaned).unwrap_or(0)
    } else {
        0
    }
}

/// Extract all offsets from path as comma-separated string
fn extract_offsets_from_path(path: &str) -> String {
    let mut offsets = Vec::new();
    for part in path.split('+') {
        let cleaned = part.trim().trim_start_matches('[').trim_end_matches(']');
        if cleaned.eq_ignore_ascii_case("base") || cleaned.is_empty() {
            continue;
        }
        if let Ok(off) = parse_base_address(cleaned) {
            offsets.push(format!("{:#X}", off));
        }
    }
    offsets.join(", ")
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
