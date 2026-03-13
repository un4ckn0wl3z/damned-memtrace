//! Main application component

use dioxus::prelude::*;
use memlib::{
    get_process_modules, get_processes, parse_base_address, parse_offsets, ProcessHandle,
    ProcessInfo, TraversalResult, EntityResult, SavedMemoryScan, SavedEntityScan,
    save_memory_scan, load_memory_scan, save_entity_scan, load_entity_scan,
};
use std::sync::Arc;

use crate::styles::STYLES;


/// Main application component
#[component]
pub fn App() -> Element {
    // Process selection
    let mut processes = use_signal(Vec::<ProcessInfo>::new);
    let mut selected_pid = use_signal(|| 0u32);
    let mut process_filter = use_signal(String::new);

    // Input fields
    let mut base_address_input = use_signal(|| String::from("0"));
    let mut offsets_input = use_signal(String::new);
    let mut loop_count_input = use_signal(|| String::from("1000"));
    let mut step_size = use_signal(|| 4u32);
    let mut loop_offset_index = use_signal(|| -1i32); // -1 means last index

    // Display options
    let mut signed_display = use_signal(|| false);

    // Scan state
    let mut is_scanning = use_signal(|| false);
    let mut scan_progress = use_signal(|| 0.0f32);
    let mut results = use_signal(Vec::<TraversalResult>::new);
    let mut error_message = use_signal(String::new);
    let mut status_message = use_signal(|| String::from("Ready"));

    // Module list for selected process
    let mut modules = use_signal(Vec::new);

    // About popup state
    let mut about_popup = use_signal(|| false);

    // Real-time refresh state
    let mut auto_refresh = use_signal(|| false);
    let mut refresh_interval = use_signal(|| 500u64); // milliseconds

    // Edit state
    let mut editing_cell = use_signal(|| None::<(usize, String)>); // (row_index, column_type)
    let mut edit_value = use_signal(String::new);

    // Tab state
    let mut active_tab = use_signal(|| "memory".to_string());

    // Entity scanner state
    let mut entity_mode = use_signal(|| "array".to_string());
    let mut entity_base = use_signal(String::new);
    let mut entity_struct_size = use_signal(|| "0x100".to_string());
    let mut entity_max_count = use_signal(|| "100".to_string());
    let mut entity_value_offsets = use_signal(|| "0x0".to_string());
    let mut entity_data_type = use_signal(|| "i32".to_string());
    let _entity_next_offset = use_signal(|| "0x8".to_string());
    let mut entity_results = use_signal(Vec::<EntityResult>::new);
    let mut entity_scanning = use_signal(|| false);
    let mut entity_editing_cell = use_signal(|| None::<(usize, usize)>); // (entity_index, value_index)
    let mut entity_edit_value = use_signal(String::new);
    let mut entity_auto_refresh = use_signal(|| false);
    let mut entity_refresh_interval = use_signal(|| 500u64);

    // Refresh process list
    let refresh_processes = move |_| {
        let procs = get_processes();
        processes.set(procs);
        status_message.set("Process list refreshed".to_string());
    };

    // Load processes on mount
    use_effect(move || {
        let procs = get_processes();
        processes.set(procs);
    });

    // Load modules when process changes
    use_effect(move || {
        let pid = selected_pid();
        if pid > 0 {
            let mods = get_process_modules(pid);
            modules.set(mods);
        } else {
            modules.set(vec![]);
        }
    });

    // Auto-refresh timer
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(refresh_interval())).await;
            
            if !auto_refresh() || results().is_empty() || is_scanning() {
                continue;
            }
            
            let pid = selected_pid();
            if pid == 0 {
                continue;
            }
            
            // Refresh values for all results
            if let Some(handle) = ProcessHandle::open(pid) {
                let mut updated_results = results();
                for result in updated_results.iter_mut() {
                    let new_result = handle.read_all_types(result.actual_address);
                    if new_result.valid {
                        result.value_i8 = new_result.value_i8;
                        result.value_u8 = new_result.value_u8;
                        result.value_i16 = new_result.value_i16;
                        result.value_u16 = new_result.value_u16;
                        result.value_i32 = new_result.value_i32;
                        result.value_u32 = new_result.value_u32;
                        result.value_i64 = new_result.value_i64;
                        result.value_u64 = new_result.value_u64;
                        result.value_f32 = new_result.value_f32;
                        result.value_f64 = new_result.value_f64;
                    }
                }
                results.set(updated_results);
            }
        }
    });

    // Entity auto-refresh timer
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(entity_refresh_interval())).await;
            
            if !entity_auto_refresh() || entity_results().is_empty() || entity_scanning() {
                continue;
            }
            
            let pid = selected_pid();
            if pid == 0 {
                continue;
            }
            
            let data_type = entity_data_type();
            if let Some(handle) = ProcessHandle::open(pid) {
                let mut updated = entity_results();
                for entity in updated.iter_mut() {
                    for (off_name, val) in entity.values.iter_mut() {
                        let off = parse_base_address(off_name.trim_start_matches("0x").trim_start_matches("0X")).unwrap_or(0) as i64;
                        let va = (entity.address as i64 + off) as u64;
                        let new_val: Option<i64> = match data_type.as_str() {
                            "i8" => { let mut b = [0u8; 1]; if handle.read_memory(va, &mut b) { Some(i8::from_le_bytes(b) as i64) } else { None } }
                            "i16" => { let mut b = [0u8; 2]; if handle.read_memory(va, &mut b) { Some(i16::from_le_bytes(b) as i64) } else { None } }
                            "i32" => { let mut b = [0u8; 4]; if handle.read_memory(va, &mut b) { Some(i32::from_le_bytes(b) as i64) } else { None } }
                            "i64" => { let mut b = [0u8; 8]; if handle.read_memory(va, &mut b) { Some(i64::from_le_bytes(b)) } else { None } }
                            "f32" => { let mut b = [0u8; 4]; if handle.read_memory(va, &mut b) { Some(f32::from_le_bytes(b).to_bits() as i64) } else { None } }
                            "f64" => { let mut b = [0u8; 8]; if handle.read_memory(va, &mut b) { Some(f64::from_le_bytes(b).to_bits() as i64) } else { None } }
                            _ => None
                        };
                        if let Some(nv) = new_val {
                            *val = nv;
                        }
                    }
                }
                entity_results.set(updated);
            }
        }
    });

    // Filter processes
    let filtered_processes = use_memo(move || {
        let filter = process_filter().to_lowercase();
        if filter.is_empty() {
            processes()
        } else {
            processes()
                .into_iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&filter)
                        || p.pid.to_string().contains(&filter)
                })
                .collect()
        }
    });

    // Start scan
    let start_scan = move |_| {
        error_message.set(String::new());

        // Parse base address
        let base_addr = match parse_base_address(&base_address_input()) {
            Ok(addr) => addr,
            Err(e) => {
                error_message.set(format!("Invalid base address: {}", e));
                return;
            }
        };

        // Parse offsets
        let offsets = match parse_offsets(&offsets_input()) {
            Ok(offs) => offs,
            Err(e) => {
                error_message.set(format!("Invalid offsets: {}", e));
                return;
            }
        };

        // Parse loop count
        let loop_count: u32 = match loop_count_input().parse() {
            Ok(n) => n,
            Err(_) => {
                error_message.set("Invalid loop count".to_string());
                return;
            }
        };

        let pid = selected_pid();
        if pid == 0 {
            error_message.set("Please select a process".to_string());
            return;
        }

        let step = step_size();
        let _signed = signed_display();
        let offset_idx = loop_offset_index();

        // Start scanning in background
        is_scanning.set(true);
        scan_progress.set(0.0);
        results.set(vec![]);
        status_message.set("Scanning...".to_string());

        spawn(async move {
            let handle = match ProcessHandle::open(pid) {
                Some(h) => Arc::new(h),
                None => {
                    error_message.set("Failed to open process".to_string());
                    is_scanning.set(false);
                    status_message.set("Scan failed".to_string());
                    return;
                }
            };

            let mut scan_results = Vec::new();
            let total = loop_count as f32;

            for i in 0..loop_count {
                if i % 100 == 0 {
                    scan_progress.set((i as f32 / total) * 100.0);
                }

                // Calculate the current offset for this iteration
                let current_offset = (i as i64) * (step as i64);

                // Create modified offsets - replace the selected index with loop value
                let mut modified_offsets = offsets.clone();
                let target_idx = if offset_idx < 0 || offset_idx as usize >= modified_offsets.len() {
                    if modified_offsets.is_empty() { 0 } else { modified_offsets.len() - 1 }
                } else {
                    offset_idx as usize
                };

                if modified_offsets.is_empty() {
                    modified_offsets.push(current_offset);
                } else {
                    // Replace the offset at target index with the loop value
                    modified_offsets[target_idx] = current_offset;
                }

                // Build the offset path string showing the modified offsets
                let mut offset_path = format!("[[base]");
                for (idx, off) in modified_offsets.iter().enumerate() {
                    offset_path.push_str(&format!("+{:#X}]", off));
                    if idx < modified_offsets.len() - 1 {
                        // Not the last one, so there's another dereference
                    }
                }

                // Traverse the pointer chain
                if let Some(final_addr) = handle.traverse_pointer_chain(base_addr, &modified_offsets)
                {
                    let mut result = handle.read_all_types(final_addr);
                    result.offset_path = offset_path;
                    if result.valid {
                        scan_results.push(result);
                    }
                }
            }

            results.set(scan_results.clone());
            scan_progress.set(100.0);
            is_scanning.set(false);
            status_message.set(format!("Scan complete: {} results", scan_results.len()));
        });
    };

    // Stop scan
    let stop_scan = move |_| {
        is_scanning.set(false);
        status_message.set("Scan stopped".to_string());
    };

    // Copy to clipboard
    let copy_to_clipboard = |text: String| {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
    };

    rsx! {
        style { {STYLES} }
        div { class: "app-container",
            // Title bar
            div { class: "title-bar",
                h1 { "Damned Memory Traversal Tool" }
                div { class: "title-bar-buttons",
                    button {
                        class: "about-btn",
                        onclick: move |_: MouseEvent| {
                            about_popup.set(true);
                        },
                        "?"
                    }
                    button {
                        onclick: move |_: MouseEvent| {
                            let window = dioxus::desktop::window();
                            window.set_minimized(true);
                        },
                        "−"
                    }
                    button {
                        onclick: move |_: MouseEvent| {
                            let window = dioxus::desktop::window();
                            let is_max = window.is_maximized();
                            window.set_maximized(!is_max);
                        },
                        "□"
                    }
                    button {
                        class: "close",
                        onclick: move |_: MouseEvent| {
                            let window = dioxus::desktop::window();
                            window.close();
                        },
                        "×"
                    }
                }
            }

            // Tab bar
            div { class: "tab-bar",
                button {
                    class: if active_tab() == "memory" { "tab-btn active" } else { "tab-btn" },
                    onclick: move |_| active_tab.set("memory".to_string()),
                    "Memory Traversal"
                }
                button {
                    class: if active_tab() == "entity" { "tab-btn active" } else { "tab-btn" },
                    onclick: move |_| active_tab.set("entity".to_string()),
                    "Entity List"
                }
            }

            // Main content
            div { class: "main-content",
                // Left panel - Controls
                div { class: "left-panel",
                    // Process selection
                    div { class: "panel",
                        div { class: "panel-title", "Process Selection" }

                        div { class: "form-group",
                            label { "Filter" }
                            input {
                                r#type: "text",
                                placeholder: "Search process...",
                                value: "{process_filter}",
                                oninput: move |e| process_filter.set(e.value())
                            }
                        }

                        div { class: "form-group",
                            label { "Process" }
                            select {
                                value: "{selected_pid}",
                                onchange: move |e| {
                                    if let Ok(pid) = e.value().parse::<u32>() {
                                        selected_pid.set(pid);
                                    }
                                },
                                option { value: "0", "-- Select Process --" }
                                for proc in filtered_processes() {
                                    option {
                                        value: "{proc.pid}",
                                        {format!("[{}] {} ({})", proc.pid, proc.name, if proc.is_64bit { "x64" } else { "x86" })}
                                    }
                                }
                            }
                        }

                        button {
                            class: "btn btn-secondary btn-full",
                            onclick: refresh_processes,
                            "↻ Refresh"
                        }

                        // Module list
                        if !modules().is_empty() {
                            div { class: "divider" }
                            div { class: "panel-title", "Modules" }
                            div { class: "module-list",
                                for module in modules() {
                                    div {
                                        class: "module-item",
                                        onclick: {
                                            let addr = module.base_address;
                                            move |_| {
                                                base_address_input.set(format!("{:#X}", addr));
                                            }
                                        },
                                        span { class: "module-name", "{module.name}" }
                                        span { class: "module-addr", "{module.base_address:#X}" }
                                    }
                                }
                            }
                            p { class: "info-text", "Click module to set as base address" }
                        }
                    }

                    // Scan configuration (Memory tab only)
                    if active_tab() == "memory" {
                        div { class: "panel",
                            div { class: "panel-title", "Scan Configuration" }

                        div { class: "form-group",
                            label { "Base Address (hex)" }
                            input {
                                r#type: "text",
                                placeholder: "e.g. 1599610 or 0x1599610",
                                value: "{base_address_input}",
                                oninput: move |e| base_address_input.set(e.value())
                            }
                        }

                        div { class: "form-group",
                            label { "Offsets" }
                            input {
                                r#type: "text",
                                placeholder: "e.g. [[[base]+0x40]+0x20]+0x0",
                                value: "{offsets_input}",
                                oninput: move |e| offsets_input.set(e.value())
                            }
                            p { class: "info-text", "Formats: [[[base]+0x40]+0x20]+0x0 or 0x40+0x20+0x0" }
                        }

                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Loop Count" }
                                input {
                                    r#type: "text",
                                    value: "{loop_count_input}",
                                    oninput: move |e| loop_count_input.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Step Size" }
                                select {
                                    value: "{step_size}",
                                    onchange: move |e| {
                                        if let Ok(s) = e.value().parse::<u32>() {
                                            step_size.set(s);
                                        }
                                    },
                                    option { value: "1", "1 byte" }
                                    option { value: "2", "2 bytes" }
                                    option { value: "4", "4 bytes" }
                                    option { value: "8", "8 bytes" }
                                    option { value: "16", "16 bytes" }
                                }
                            }
                        }

                        div { class: "form-group",
                            label { "Traverse at Index" }
                            input {
                                r#type: "text",
                                placeholder: "-1 (last)",
                                value: "{loop_offset_index}",
                                oninput: move |e| {
                                    if let Ok(idx) = e.value().parse::<i32>() {
                                        loop_offset_index.set(idx);
                                    }
                                }
                            }
                            p { class: "info-text", "-1=last, 0=first, 1=second..." }
                        }

                        div { class: "checkbox-group",
                            input {
                                r#type: "checkbox",
                                id: "signed",
                                checked: signed_display(),
                                onchange: move |e| signed_display.set(e.checked())
                            }
                            label { r#for: "signed", "Show signed values" }
                        }

                        div { class: "divider" }

                        div { class: "checkbox-group",
                            input {
                                r#type: "checkbox",
                                id: "auto_refresh",
                                checked: auto_refresh(),
                                onchange: move |e| auto_refresh.set(e.checked())
                            }
                            label { r#for: "auto_refresh", "Auto-refresh values" }
                        }

                        if auto_refresh() {
                            div { class: "form-group",
                                label { "Refresh Interval (ms)" }
                                select {
                                    value: "{refresh_interval}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<u64>() {
                                            refresh_interval.set(v);
                                        }
                                    },
                                    option { value: "100", "100ms" }
                                    option { value: "250", "250ms" }
                                    option { value: "500", "500ms" }
                                    option { value: "1000", "1000ms" }
                                    option { value: "2000", "2000ms" }
                                }
                            }
                        }

                        div { class: "divider" }

                        if is_scanning() {
                            button {
                                class: "btn btn-danger btn-full",
                                onclick: stop_scan,
                                "⏹ Stop Scan"
                            }
                        } else {
                            button {
                                class: "btn btn-primary btn-full",
                                onclick: start_scan,
                                disabled: selected_pid() == 0,
                                "▶ Start Scan"
                            }
                        }

                        // Progress bar
                        if is_scanning() {
                            div { class: "progress-container",
                                div { class: "progress-bar",
                                    div {
                                        class: "progress-fill",
                                        style: "width: {scan_progress()}%"
                                    }
                                }
                                p { class: "progress-text", "{scan_progress():.1}%" }
                            }
                        }

                        // Error message
                        if !error_message().is_empty() {
                            p { class: "error-message", "{error_message}" }
                        }
                        }
                    }

                    // Entity List tab panel
                    if active_tab() == "entity" {
                        div { class: "panel",
                            div { class: "panel-title", "Entity List Scanner" }
                            div { class: "form-group",
                                label { "Mode" }
                                select {
                                    value: "{entity_mode}",
                                    onchange: move |e| entity_mode.set(e.value()),
                                    option { value: "array", "Array Scanner" }
                                    option { value: "pointer", "Pointer Table" }
                                    option { value: "linked", "Linked List" }
                                }
                            }
                            div { class: "form-group",
                                label { "Base Address" }
                                input {
                                    r#type: "text",
                                    placeholder: "0x7FF6B6D37140",
                                    value: "{entity_base}",
                                    oninput: move |e| entity_base.set(e.value())
                                }
                            }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "Struct Size / Next Offset" }
                                    input {
                                        r#type: "text",
                                        value: "{entity_struct_size}",
                                        oninput: move |e| entity_struct_size.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Max Count" }
                                    input {
                                        r#type: "text",
                                        value: "{entity_max_count}",
                                        oninput: move |e| entity_max_count.set(e.value())
                                    }
                                }
                            }
                            div { class: "form-group",
                                label { "Value Offsets (comma-sep)" }
                                input {
                                    r#type: "text",
                                    placeholder: "0x100,0x104,0x120",
                                    value: "{entity_value_offsets}",
                                    oninput: move |e| entity_value_offsets.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Data Type" }
                                select {
                                    value: "{entity_data_type}",
                                    onchange: move |e| entity_data_type.set(e.value()),
                                    option { value: "i8", "i8 (1 byte)" }
                                    option { value: "i16", "i16 (2 bytes)" }
                                    option { value: "i32", "i32 (4 bytes)" }
                                    option { value: "i64", "i64 (8 bytes)" }
                                    option { value: "f32", "f32 (float)" }
                                    option { value: "f64", "f64 (double)" }
                                }
                            }
                            div { class: "form-group checkbox-group",
                                input {
                                    r#type: "checkbox",
                                    id: "entity-auto-refresh",
                                    checked: entity_auto_refresh(),
                                    onchange: move |e| entity_auto_refresh.set(e.checked())
                                }
                                label { r#for: "entity-auto-refresh", "Auto-refresh values" }
                            }
                            div { class: "form-group",
                                label { "Refresh Interval (ms)" }
                                input {
                                    r#type: "number",
                                    value: "{entity_refresh_interval}",
                                    oninput: move |e| {
                                        if let Ok(v) = e.value().parse::<u64>() {
                                            entity_refresh_interval.set(v.max(100));
                                        }
                                    }
                                }
                            }
                            div { class: "divider" }
                            button {
                                class: "btn btn-primary btn-full",
                                disabled: selected_pid() == 0 || entity_scanning(),
                                onclick: move |_| {
                                    let pid = selected_pid();
                                    if pid == 0 { return; }
                                    let base = match parse_base_address(&entity_base()) {
                                        Ok(b) => b,
                                        Err(_) => { status_message.set("Invalid base".to_string()); return; }
                                    };
                                    let struct_size = parse_base_address(&entity_struct_size()).unwrap_or(0x100) as u64;
                                    let max_count = entity_max_count().parse::<usize>().unwrap_or(100);
                                    let mode = entity_mode();
                                    let value_offs: Vec<i64> = entity_value_offsets()
                                        .split(',')
                                        .filter_map(|s| parse_base_address(s.trim()).ok().map(|v| v as i64))
                                        .collect();
                                    let data_type = entity_data_type();
                                    entity_scanning.set(true);
                                    status_message.set("Scanning...".to_string());
                                    spawn(async move {
                                        let handle = match ProcessHandle::open(pid) {
                                            Some(h) => Arc::new(h),
                                            None => { entity_scanning.set(false); return; }
                                        };
                                        let mut res = Vec::new();
                                        for i in 0..max_count {
                                            let addr = if mode == "pointer" {
                                                let ptr_addr = base + (i as u64) * 8;
                                                let mut buf = [0u8; 8];
                                                if !handle.read_memory(ptr_addr, &mut buf) { continue; }
                                                let a = u64::from_le_bytes(buf);
                                                if a == 0 { continue; }
                                                a
                                            } else {
                                                base + (i as u64) * struct_size
                                            };
                                            let mut values = Vec::new();
                                            for off in &value_offs {
                                                let va = (addr as i64 + off) as u64;
                                                let val: Option<i64> = match data_type.as_str() {
                                                    "i8" => { let mut b = [0u8; 1]; if handle.read_memory(va, &mut b) { Some(i8::from_le_bytes(b) as i64) } else { None } }
                                                    "i16" => { let mut b = [0u8; 2]; if handle.read_memory(va, &mut b) { Some(i16::from_le_bytes(b) as i64) } else { None } }
                                                    "i32" => { let mut b = [0u8; 4]; if handle.read_memory(va, &mut b) { Some(i32::from_le_bytes(b) as i64) } else { None } }
                                                    "i64" => { let mut b = [0u8; 8]; if handle.read_memory(va, &mut b) { Some(i64::from_le_bytes(b)) } else { None } }
                                                    "f32" => { let mut b = [0u8; 4]; if handle.read_memory(va, &mut b) { Some(f32::from_le_bytes(b).to_bits() as i64) } else { None } }
                                                    "f64" => { let mut b = [0u8; 8]; if handle.read_memory(va, &mut b) { Some(f64::from_le_bytes(b).to_bits() as i64) } else { None } }
                                                    _ => { let mut b = [0u8; 4]; if handle.read_memory(va, &mut b) { Some(i32::from_le_bytes(b) as i64) } else { None } }
                                                };
                                                if let Some(v) = val {
                                                    values.push((format!("{:#X}", off), v));
                                                }
                                            }
                                            if !values.is_empty() {
                                                res.push(EntityResult { index: i, address: addr, values, valid: true });
                                            }
                                        }
                                        entity_results.set(res.clone());
                                        entity_scanning.set(false);
                                        status_message.set(format!("Found {} entities", res.len()));
                                    });
                                },
                                if entity_scanning() { "Scanning..." } else { "▶ Scan" }
                            }
                        }
                    }
                }

                // Right panel - Results
                div { class: "right-panel",
                    div { class: "panel results-panel",
                        div { class: "results-header",
                            div { class: "panel-title", "Results" }
                            div { class: "results-actions",
                                button {
                                    class: "btn btn-small",
                                    onclick: move |_| {
                                        let is_memory = active_tab() == "memory";
                                        if is_memory && results().is_empty() {
                                            status_message.set("No results to save".to_string());
                                            return;
                                        }
                                        if !is_memory && entity_results().is_empty() {
                                            status_message.set("No results to save".to_string());
                                            return;
                                        }
                                        let default_name = if is_memory { "memory_scan.json" } else { "entity_scan.json" };
                                        spawn(async move {
                                            let file = rfd::AsyncFileDialog::new()
                                                .add_filter("JSON", &["json"])
                                                .set_file_name(default_name)
                                                .save_file()
                                                .await;
                                            if let Some(f) = file {
                                                let path = f.path().to_string_lossy().to_string();
                                                if is_memory {
                                                    let scan = SavedMemoryScan {
                                                        base_address: base_address_input(),
                                                        offsets: offsets_input(),
                                                        loop_count: loop_count_input().parse().unwrap_or(1000),
                                                        step_size: step_size(),
                                                        results: results(),
                                                    };
                                                    match save_memory_scan(&path, &scan) {
                                                        Ok(_) => status_message.set(format!("Saved to {}", path)),
                                                        Err(e) => status_message.set(format!("Save failed: {}", e)),
                                                    }
                                                } else {
                                                    let scan = SavedEntityScan {
                                                        mode: entity_mode(),
                                                        base_address: entity_base(),
                                                        struct_size: entity_struct_size(),
                                                        max_count: entity_max_count().parse().unwrap_or(100),
                                                        value_offsets: entity_value_offsets(),
                                                        data_type: entity_data_type(),
                                                        results: entity_results(),
                                                    };
                                                    match save_entity_scan(&path, &scan) {
                                                        Ok(_) => status_message.set(format!("Saved to {}", path)),
                                                        Err(e) => status_message.set(format!("Save failed: {}", e)),
                                                    }
                                                }
                                            }
                                        });
                                    },
                                    "💾 Save"
                                }
                                button {
                                    class: "btn btn-small",
                                    onclick: move |_| {
                                        let is_memory = active_tab() == "memory";
                                        spawn(async move {
                                            let file = rfd::AsyncFileDialog::new()
                                                .add_filter("JSON", &["json"])
                                                .pick_file()
                                                .await;
                                            if let Some(f) = file {
                                                let path = f.path().to_string_lossy().to_string();
                                                if is_memory {
                                                    match load_memory_scan(&path) {
                                                        Ok(scan) => {
                                                            base_address_input.set(scan.base_address);
                                                            offsets_input.set(scan.offsets);
                                                            loop_count_input.set(scan.loop_count.to_string());
                                                            step_size.set(scan.step_size);
                                                            results.set(scan.results);
                                                            status_message.set(format!("Loaded {}", path));
                                                        }
                                                        Err(e) => status_message.set(format!("Load failed: {}", e)),
                                                    }
                                                } else {
                                                    match load_entity_scan(&path) {
                                                        Ok(scan) => {
                                                            entity_mode.set(scan.mode);
                                                            entity_base.set(scan.base_address);
                                                            entity_struct_size.set(scan.struct_size);
                                                            entity_max_count.set(scan.max_count.to_string());
                                                            entity_value_offsets.set(scan.value_offsets);
                                                            entity_data_type.set(scan.data_type);
                                                            entity_results.set(scan.results);
                                                            status_message.set(format!("Loaded {}", path));
                                                        }
                                                        Err(e) => status_message.set(format!("Load failed: {}", e)),
                                                    }
                                                }
                                            }
                                        });
                                    },
                                    "📂 Load"
                                }
                                div { class: "results-count",
                                    if active_tab() == "memory" { "{results().len()} entries" } else { "{entity_results().len()} entities" }
                                }
                            }
                        }

                        // Entity results table
                        if active_tab() == "entity" {
                            div { class: "results-table-container",
                                table { class: "results-table",
                                    thead {
                                        tr {
                                            th { "Index" }
                                            th { "Address" }
                                            for (off_name, _) in entity_results().first().map(|r| r.values.clone()).unwrap_or_default() {
                                                th { "{off_name}" }
                                            }
                                            th { "" }
                                        }
                                    }
                                    tbody {
                                        for (ent_idx, entity) in entity_results().into_iter().enumerate() {
                                            tr {
                                                td { class: "col-value", "{entity.index}" }
                                                td { class: "col-address", "{entity.address:#X}" }
                                                for (val_idx, (off_name, val)) in entity.values.iter().enumerate() {
                                                    {
                                                        let off = parse_base_address(off_name.trim_start_matches("0x").trim_start_matches("0X")).unwrap_or(0) as i64;
                                                        let val_addr = (entity.address as i64 + off) as u64;
                                                        let display_val = if entity_data_type() == "f32" {
                                                            format!("{:.3}", f32::from_bits(*val as u32))
                                                        } else if entity_data_type() == "f64" {
                                                            format!("{:.3}", f64::from_bits(*val as u64))
                                                        } else {
                                                            val.to_string()
                                                        };
                                                        rsx! {
                                                            td {
                                                                class: "col-value editable",
                                                                ondoubleclick: {
                                                                    let dv = display_val.clone();
                                                                    move |_| {
                                                                        entity_editing_cell.set(Some((ent_idx, val_idx)));
                                                                        entity_edit_value.set(dv.clone());
                                                                    }
                                                                },
                                                                if entity_editing_cell().map(|(ei, vi)| ei == ent_idx && vi == val_idx).unwrap_or(false) {
                                                                    input {
                                                                        class: "edit-input",
                                                                        r#type: "text",
                                                                        value: "{entity_edit_value}",
                                                                        autofocus: true,
                                                                        oninput: move |e| entity_edit_value.set(e.value()),
                                                                        onkeydown: {
                                                                            let dt = entity_data_type();
                                                                            move |e: KeyboardEvent| {
                                                                                if e.key() == Key::Enter {
                                                                                    let pid = selected_pid();
                                                                                    if let Some(handle) = ProcessHandle::open(pid) {
                                                                                        let written = match dt.as_str() {
                                                                                            "i8" => entity_edit_value().parse::<i8>().ok().map(|v| handle.write_i8(val_addr, v)).unwrap_or(false),
                                                                                            "i16" => entity_edit_value().parse::<i16>().ok().map(|v| handle.write_i16(val_addr, v)).unwrap_or(false),
                                                                                            "i32" => entity_edit_value().parse::<i32>().ok().map(|v| handle.write_i32(val_addr, v)).unwrap_or(false),
                                                                                            "i64" => entity_edit_value().parse::<i64>().ok().map(|v| handle.write_i64(val_addr, v)).unwrap_or(false),
                                                                                            "f32" => entity_edit_value().parse::<f32>().ok().map(|v| handle.write_f32(val_addr, v)).unwrap_or(false),
                                                                                            "f64" => entity_edit_value().parse::<f64>().ok().map(|v| handle.write_f64(val_addr, v)).unwrap_or(false),
                                                                                            _ => false
                                                                                        };
                                                                                        if written {
                                                                                            status_message.set(format!("Written to {:#X}", val_addr));
                                                                                        } else {
                                                                                            status_message.set("Write failed".to_string());
                                                                                        }
                                                                                    }
                                                                                    entity_editing_cell.set(None);
                                                                                } else if e.key() == Key::Escape {
                                                                                    entity_editing_cell.set(None);
                                                                                }
                                                                            }
                                                                        },
                                                                        onblur: move |_| entity_editing_cell.set(None)
                                                                    }
                                                                } else {
                                                                    "{display_val}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                td {
                                                    button {
                                                        class: "copy-btn",
                                                        onclick: {
                                                            let addr = entity.address;
                                                            move |_| {
                                                                copy_to_clipboard(format!("{:#X}", addr));
                                                            }
                                                        },
                                                        "Copy"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Memory results table
                        if active_tab() == "memory" {
                            div { class: "results-table-container",
                                table { class: "results-table",
                                    thead {
                                        tr {
                                            th { "Offset Path" }
                                            th { "Address" }
                                            th { "1-Byte" }
                                            th { "2-Byte" }
                                            th { "4-Byte" }
                                            th { "8-Byte" }
                                            th { "Float" }
                                            th { "Double" }
                                            th { "" }
                                        }
                                    }
                                    tbody {
                                        for (idx, result) in results().into_iter().enumerate() {
                                            tr {
                                                td { class: "col-offset", "{result.offset_path}" }
                                                td { class: "col-address", "{result.actual_address:#X}" }
                                            // 1-Byte column (editable)
                                            td {
                                                class: "col-value editable",
                                                ondoubleclick: {
                                                    let val = if signed_display() { result.value_i8.to_string() } else { result.value_u8.to_string() };
                                                    move |_| {
                                                        editing_cell.set(Some((idx, "i8".to_string())));
                                                        edit_value.set(val.clone());
                                                    }
                                                },
                                                if editing_cell().map(|(i, ref t)| i == idx && t == "i8").unwrap_or(false) {
                                                    input {
                                                        class: "edit-input",
                                                        r#type: "text",
                                                        value: "{edit_value}",
                                                        autofocus: true,
                                                        oninput: move |e| edit_value.set(e.value()),
                                                        onkeydown: {
                                                            let addr = result.actual_address;
                                                            move |e: KeyboardEvent| {
                                                                if e.key() == Key::Enter {
                                                                    if let Ok(val) = edit_value().parse::<i8>() {
                                                                        let pid = selected_pid();
                                                                        if let Some(handle) = ProcessHandle::open(pid) {
                                                                            if handle.write_i8(addr, val) {
                                                                                status_message.set(format!("Written {} to {:#X}", val, addr));
                                                                            } else {
                                                                                status_message.set("Write failed".to_string());
                                                                            }
                                                                        }
                                                                    }
                                                                    editing_cell.set(None);
                                                                } else if e.key() == Key::Escape {
                                                                    editing_cell.set(None);
                                                                }
                                                            }
                                                        },
                                                        onblur: move |_| editing_cell.set(None)
                                                    }
                                                } else {
                                                    {if signed_display() { result.value_i8.to_string() } else { result.value_u8.to_string() }}
                                                }
                                            }
                                            // 2-Byte column (editable)
                                            td {
                                                class: "col-value editable",
                                                ondoubleclick: {
                                                    let val = if signed_display() { result.value_i16.to_string() } else { result.value_u16.to_string() };
                                                    move |_| {
                                                        editing_cell.set(Some((idx, "i16".to_string())));
                                                        edit_value.set(val.clone());
                                                    }
                                                },
                                                if editing_cell().map(|(i, ref t)| i == idx && t == "i16").unwrap_or(false) {
                                                    input {
                                                        class: "edit-input",
                                                        r#type: "text",
                                                        value: "{edit_value}",
                                                        autofocus: true,
                                                        oninput: move |e| edit_value.set(e.value()),
                                                        onkeydown: {
                                                            let addr = result.actual_address;
                                                            move |e: KeyboardEvent| {
                                                                if e.key() == Key::Enter {
                                                                    if let Ok(val) = edit_value().parse::<i16>() {
                                                                        let pid = selected_pid();
                                                                        if let Some(handle) = ProcessHandle::open(pid) {
                                                                            if handle.write_i16(addr, val) {
                                                                                status_message.set(format!("Written {} to {:#X}", val, addr));
                                                                            } else {
                                                                                status_message.set("Write failed".to_string());
                                                                            }
                                                                        }
                                                                    }
                                                                    editing_cell.set(None);
                                                                } else if e.key() == Key::Escape {
                                                                    editing_cell.set(None);
                                                                }
                                                            }
                                                        },
                                                        onblur: move |_| editing_cell.set(None)
                                                    }
                                                } else {
                                                    {if signed_display() { result.value_i16.to_string() } else { result.value_u16.to_string() }}
                                                }
                                            }
                                            // 4-Byte column (editable)
                                            td {
                                                class: "col-value editable",
                                                ondoubleclick: {
                                                    let val = if signed_display() { result.value_i32.to_string() } else { result.value_u32.to_string() };
                                                    move |_| {
                                                        editing_cell.set(Some((idx, "i32".to_string())));
                                                        edit_value.set(val.clone());
                                                    }
                                                },
                                                if editing_cell().map(|(i, ref t)| i == idx && t == "i32").unwrap_or(false) {
                                                    input {
                                                        class: "edit-input",
                                                        r#type: "text",
                                                        value: "{edit_value}",
                                                        autofocus: true,
                                                        oninput: move |e| edit_value.set(e.value()),
                                                        onkeydown: {
                                                            let addr = result.actual_address;
                                                            move |e: KeyboardEvent| {
                                                                if e.key() == Key::Enter {
                                                                    if let Ok(val) = edit_value().parse::<i32>() {
                                                                        let pid = selected_pid();
                                                                        if let Some(handle) = ProcessHandle::open(pid) {
                                                                            if handle.write_i32(addr, val) {
                                                                                status_message.set(format!("Written {} to {:#X}", val, addr));
                                                                            } else {
                                                                                status_message.set("Write failed".to_string());
                                                                            }
                                                                        }
                                                                    }
                                                                    editing_cell.set(None);
                                                                } else if e.key() == Key::Escape {
                                                                    editing_cell.set(None);
                                                                }
                                                            }
                                                        },
                                                        onblur: move |_| editing_cell.set(None)
                                                    }
                                                } else {
                                                    {if signed_display() { result.value_i32.to_string() } else { result.value_u32.to_string() }}
                                                }
                                            }
                                            // 8-Byte column
                                            td {
                                                class: "col-value editable",
                                                ondoubleclick: {
                                                    let val = if signed_display() { result.value_i64.to_string() } else { result.value_u64.to_string() };
                                                    move |_| {
                                                        editing_cell.set(Some((idx, "i64".to_string())));
                                                        edit_value.set(val.clone());
                                                    }
                                                },
                                                if editing_cell().map(|(i, ref t)| i == idx && t == "i64").unwrap_or(false) {
                                                    input {
                                                        class: "edit-input",
                                                        r#type: "text",
                                                        value: "{edit_value}",
                                                        autofocus: true,
                                                        oninput: move |e| edit_value.set(e.value()),
                                                        onkeydown: {
                                                            let addr = result.actual_address;
                                                            move |e: KeyboardEvent| {
                                                                if e.key() == Key::Enter {
                                                                    if let Ok(val) = edit_value().parse::<i64>() {
                                                                        let pid = selected_pid();
                                                                        if let Some(handle) = ProcessHandle::open(pid) {
                                                                            if handle.write_i64(addr, val) {
                                                                                status_message.set(format!("Written {} to {:#X}", val, addr));
                                                                            } else {
                                                                                status_message.set("Write failed".to_string());
                                                                            }
                                                                        }
                                                                    }
                                                                    editing_cell.set(None);
                                                                } else if e.key() == Key::Escape {
                                                                    editing_cell.set(None);
                                                                }
                                                            }
                                                        },
                                                        onblur: move |_| editing_cell.set(None)
                                                    }
                                                } else {
                                                    {if signed_display() { result.value_i64.to_string() } else { result.value_u64.to_string() }}
                                                }
                                            }
                                            // Float column
                                            td {
                                                class: "col-float editable",
                                                ondoubleclick: {
                                                    let val = format!("{:.6}", result.value_f32);
                                                    move |_| {
                                                        editing_cell.set(Some((idx, "f32".to_string())));
                                                        edit_value.set(val.clone());
                                                    }
                                                },
                                                if editing_cell().map(|(i, ref t)| i == idx && t == "f32").unwrap_or(false) {
                                                    input {
                                                        class: "edit-input",
                                                        r#type: "text",
                                                        value: "{edit_value}",
                                                        autofocus: true,
                                                        oninput: move |e| edit_value.set(e.value()),
                                                        onkeydown: {
                                                            let addr = result.actual_address;
                                                            move |e: KeyboardEvent| {
                                                                if e.key() == Key::Enter {
                                                                    if let Ok(val) = edit_value().parse::<f32>() {
                                                                        let pid = selected_pid();
                                                                        if let Some(handle) = ProcessHandle::open(pid) {
                                                                            if handle.write_f32(addr, val) {
                                                                                status_message.set(format!("Written {} to {:#X}", val, addr));
                                                                            } else {
                                                                                status_message.set("Write failed".to_string());
                                                                            }
                                                                        }
                                                                    }
                                                                    editing_cell.set(None);
                                                                } else if e.key() == Key::Escape {
                                                                    editing_cell.set(None);
                                                                }
                                                            }
                                                        },
                                                        onblur: move |_| editing_cell.set(None)
                                                    }
                                                } else {
                                                    "{result.value_f32:.6}"
                                                }
                                            }
                                            // Double column
                                            td {
                                                class: "col-float editable",
                                                ondoubleclick: {
                                                    let val = format!("{:.6}", result.value_f64);
                                                    move |_| {
                                                        editing_cell.set(Some((idx, "f64".to_string())));
                                                        edit_value.set(val.clone());
                                                    }
                                                },
                                                if editing_cell().map(|(i, ref t)| i == idx && t == "f64").unwrap_or(false) {
                                                    input {
                                                        class: "edit-input",
                                                        r#type: "text",
                                                        value: "{edit_value}",
                                                        autofocus: true,
                                                        oninput: move |e| edit_value.set(e.value()),
                                                        onkeydown: {
                                                            let addr = result.actual_address;
                                                            move |e: KeyboardEvent| {
                                                                if e.key() == Key::Enter {
                                                                    if let Ok(val) = edit_value().parse::<f64>() {
                                                                        let pid = selected_pid();
                                                                        if let Some(handle) = ProcessHandle::open(pid) {
                                                                            if handle.write_f64(addr, val) {
                                                                                status_message.set(format!("Written {} to {:#X}", val, addr));
                                                                            } else {
                                                                                status_message.set("Write failed".to_string());
                                                                            }
                                                                        }
                                                                    }
                                                                    editing_cell.set(None);
                                                                } else if e.key() == Key::Escape {
                                                                    editing_cell.set(None);
                                                                }
                                                            }
                                                        },
                                                        onblur: move |_| editing_cell.set(None)
                                                    }
                                                } else {
                                                    "{result.value_f64:.6}"
                                                }
                                            }
                                            td {
                                                button {
                                                    class: "copy-btn",
                                                    onclick: {
                                                        let addr = result.actual_address;
                                                        move |_| {
                                                            copy_to_clipboard(format!("{:#X}", addr));
                                                        }
                                                    },
                                                    "Copy"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            }

            // Status bar
            div { class: "status-bar",
                span { "{status_message}" }
                span {
                    {if selected_pid() > 0 { format!("PID: {}", selected_pid()) } else { "No process selected".to_string() }}
                }
            }

            // About modal
            if about_popup() {
                div {
                    class: "about-modal-overlay",
                    onclick: move |_: MouseEvent| about_popup.set(false),

                    div {
                        class: "about-modal",
                        onclick: |e: MouseEvent| e.stop_propagation(),

                        div {
                            class: "about-modal-header",
                            h2 { "About Damned Memory Traversal Tool" }
                            button {
                                class: "about-modal-close",
                                onclick: move |_: MouseEvent| about_popup.set(false),
                                "✕"
                            }
                        }

                        div { class: "about-modal-content",
                            p { class: "about-description",
                                "A memory pointer traversal tool similar to Cheat Engine's pointer scanner. "
                                "Supports multi-level offset chains and automatic data type interpretation."
                            }

                            div { class: "about-divider" }

                            div { class: "about-info",
                                span { class: "about-label", "Developer:" }
                                span { class: "about-value", "un4ckn0wl3z" }
                            }

                            div { class: "about-info",
                                span { class: "about-label", "GitHub:" }
                                a {
                                    href: "https://github.com/un4ckn0wl3z",
                                    target: "_blank",
                                    class: "about-link",
                                    "github.com/un4ckn0wl3z"
                                }
                            }

                            div { class: "about-info",
                                span { class: "about-label", "Website:" }
                                a {
                                    href: "https://un4ckn0wl3z.dev/",
                                    target: "_blank",
                                    class: "about-link",
                                    "un4ckn0wl3z.dev"
                                }
                            }

                            div { class: "about-divider" }

                            div { class: "about-discord",
                                span { "Join our Discord community:" }
                                a {
                                    href: "https://discord.gg/ugYeeJRf5S",
                                    target: "_blank",
                                    class: "about-link discord-link",
                                    "discord.gg/ugYeeJRf5S"
                                }
                            }

                            div { class: "about-footer",
                                span { "Built with Rust • Dioxus • Windows API" }
                            }
                        }
                    }
                }
            }

            // Resize handles for borderless window
            div {
                class: "resize-handle resize-handle-n",
                onmousedown: move |_| {
                    let window = dioxus::desktop::window();
                    let _ = window.drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::North);
                }
            }
            div {
                class: "resize-handle resize-handle-s",
                onmousedown: move |_| {
                    let window = dioxus::desktop::window();
                    let _ = window.drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::South);
                }
            }
            div {
                class: "resize-handle resize-handle-e",
                onmousedown: move |_| {
                    let window = dioxus::desktop::window();
                    let _ = window.drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::East);
                }
            }
            div {
                class: "resize-handle resize-handle-w",
                onmousedown: move |_| {
                    let window = dioxus::desktop::window();
                    let _ = window.drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::West);
                }
            }
            div {
                class: "resize-handle resize-handle-nw",
                onmousedown: move |_| {
                    let window = dioxus::desktop::window();
                    let _ = window.drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::NorthWest);
                }
            }
            div {
                class: "resize-handle resize-handle-ne",
                onmousedown: move |_| {
                    let window = dioxus::desktop::window();
                    let _ = window.drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::NorthEast);
                }
            }
            div {
                class: "resize-handle resize-handle-sw",
                onmousedown: move |_| {
                    let window = dioxus::desktop::window();
                    let _ = window.drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::SouthWest);
                }
            }
            div {
                class: "resize-handle resize-handle-se",
                onmousedown: move |_| {
                    let window = dioxus::desktop::window();
                    let _ = window.drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::SouthEast);
                }
            }
        }
    }
}
