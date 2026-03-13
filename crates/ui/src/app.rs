//! Main application component

use dioxus::prelude::*;
use memlib::{
    get_process_modules, get_processes, parse_base_address, parse_offsets, ProcessHandle,
    ProcessInfo, TraversalResult,
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

                // Build the offset path string
                let mut offset_path = format!("[{:#X}", base_addr);
                for off in &offsets {
                    offset_path.push_str(&format!("+{:#X}", off));
                }
                offset_path.push_str(&format!("]+{:#X}", current_offset));

                // Create modified offsets with the loop offset added to the last one
                let mut modified_offsets = offsets.clone();
                if modified_offsets.is_empty() {
                    modified_offsets.push(current_offset);
                } else {
                    let last_idx = modified_offsets.len() - 1;
                    modified_offsets[last_idx] += current_offset;
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
                        class: "close",
                        onclick: move |_: MouseEvent| {
                            let window = dioxus::desktop::window();
                            window.close();
                        },
                        "×"
                    }
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

                    // Scan configuration
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

                // Right panel - Results
                div { class: "right-panel",
                    div { class: "panel results-panel",
                        div { class: "results-header",
                            div { class: "panel-title", "Results" }
                            div { class: "results-count", "{results().len()} entries" }
                        }

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
        }
    }
}
