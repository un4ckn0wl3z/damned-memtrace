//! CSS styles for the application

pub const STYLES: &str = r#"
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: "Segoe UI", "Microsoft YaHei", sans-serif;
    font-size: 13px;
    background-color: #1e1e1e;
    color: #d4d4d4;
    overflow: hidden;
}

.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 8px;
    gap: 8px;
}

.title-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #2d2d2d;
    padding: 6px 12px;
    border-radius: 4px;
    -webkit-app-region: drag;
}

.title-bar h1 {
    font-size: 14px;
    font-weight: 600;
    color: #4fc3f7;
}

.title-bar-buttons {
    display: flex;
    gap: 8px;
    -webkit-app-region: no-drag;
}

.title-bar-buttons button {
    width: 28px;
    height: 28px;
    border: none;
    background: #3d3d3d;
    color: #d4d4d4;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
}

.title-bar-buttons button:hover {
    background: #4d4d4d;
}

.title-bar-buttons button.close:hover {
    background: #e81123;
    color: white;
}

.tab-bar {
    display: flex;
    gap: 4px;
    background: #252525;
    padding: 4px;
    border-radius: 4px;
}

.tab-btn {
    padding: 8px 16px;
    border: none;
    background: transparent;
    color: #888;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s;
}

.tab-btn:hover {
    background: #3d3d3d;
    color: #d4d4d4;
}

.tab-btn.active {
    background: #4fc3f7;
    color: #1e1e1e;
    font-weight: 600;
}

.main-content {
    display: flex;
    flex: 1;
    gap: 8px;
    min-height: 0;
}

.left-panel {
    width: 280px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex-shrink: 0;
}

.right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
}

.panel {
    background: #252526;
    border: 1px solid #3d3d3d;
    border-radius: 4px;
    padding: 10px;
}

.panel-title {
    font-size: 12px;
    font-weight: 600;
    color: #4fc3f7;
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.form-group {
    margin-bottom: 10px;
}

.form-group label {
    display: block;
    font-size: 12px;
    color: #9e9e9e;
    margin-bottom: 4px;
}

.form-group input,
.form-group select {
    width: 100%;
    padding: 6px 8px;
    background: #1e1e1e;
    border: 1px solid #3d3d3d;
    border-radius: 3px;
    color: #d4d4d4;
    font-size: 12px;
    font-family: "Consolas", monospace;
}

.form-group input:focus,
.form-group select:focus {
    outline: none;
    border-color: #4fc3f7;
}

.form-group input::placeholder {
    color: #6e6e6e;
}

.form-row {
    display: flex;
    gap: 8px;
}

.form-row .form-group {
    flex: 1;
}

.btn {
    padding: 8px 16px;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    transition: background 0.2s;
}

.btn-primary {
    background: #0e639c;
    color: white;
}

.btn-primary:hover {
    background: #1177bb;
}

.btn-primary:disabled {
    background: #3d3d3d;
    color: #6e6e6e;
    cursor: not-allowed;
}

.btn-danger {
    background: #c42b1c;
    color: white;
}

.btn-danger:hover {
    background: #e81123;
}

.btn-secondary {
    background: #3d3d3d;
    color: #d4d4d4;
}

.btn-secondary:hover {
    background: #4d4d4d;
}

.btn-full {
    width: 100%;
}

.btn-small {
    padding: 4px 8px;
    font-size: 11px;
}

.results-actions {
    display: flex;
    gap: 8px;
    align-items: center;
}

.type-select {
    padding: 2px 4px;
    background: #1e1e1e;
    border: 1px solid #3d3d3d;
    border-radius: 3px;
    color: #4fc3f7;
    font-size: 11px;
    font-family: "Consolas", monospace;
    cursor: pointer;
}

.col-type {
    width: 60px;
}

.col-export {
    width: 50px;
    text-align: center;
}

.col-export input[type="checkbox"] {
    width: 16px;
    height: 16px;
    cursor: pointer;
}

.progress-container {
    margin-top: 8px;
}

.progress-bar {
    height: 4px;
    background: #3d3d3d;
    border-radius: 2px;
    overflow: hidden;
}

.progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #4fc3f7, #29b6f6);
    transition: width 0.3s;
}

.progress-text {
    font-size: 11px;
    color: #9e9e9e;
    margin-top: 4px;
    text-align: center;
}

.results-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
}

.results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
}

.results-count {
    font-size: 11px;
    color: #9e9e9e;
}

.results-table-container {
    flex: 1;
    overflow: auto;
    border: 1px solid #3d3d3d;
    border-radius: 3px;
}

.results-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
    font-family: "Consolas", monospace;
}

.results-table th {
    position: sticky;
    top: 0;
    background: #2d2d2d;
    padding: 6px 8px;
    text-align: left;
    font-weight: 600;
    color: #4fc3f7;
    border-bottom: 1px solid #3d3d3d;
    white-space: nowrap;
}

.results-table td {
    padding: 4px 8px;
    border-bottom: 1px solid #2d2d2d;
    white-space: nowrap;
}

.results-table tr:hover td {
    background: #2a2d2e;
}

.results-table tr.selected td {
    background: #094771;
}

.results-table .col-offset {
    color: #ce9178;
}

.results-table .col-address {
    color: #4ec9b0;
}

.results-table .col-value {
    color: #b5cea8;
}

.results-table .col-float {
    color: #dcdcaa;
}

.status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #007acc;
    padding: 4px 12px;
    border-radius: 0 0 4px 4px;
    font-size: 11px;
    color: white;
}

.checkbox-group {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
}

.checkbox-group input[type="checkbox"] {
    width: 14px;
    height: 14px;
    cursor: pointer;
}

.checkbox-group label {
    font-size: 12px;
    color: #d4d4d4;
    cursor: pointer;
}

.radio-group {
    display: flex;
    gap: 12px;
    margin-bottom: 8px;
}

.radio-group label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: #d4d4d4;
    cursor: pointer;
}

.radio-group input[type="radio"] {
    width: 14px;
    height: 14px;
    cursor: pointer;
}

.error-message {
    color: #f48771;
    font-size: 11px;
    margin-top: 4px;
}

.info-text {
    font-size: 11px;
    color: #6e6e6e;
    margin-top: 4px;
}

.module-list {
    max-height: 120px;
    overflow-y: auto;
    border: 1px solid #3d3d3d;
    border-radius: 3px;
    margin-top: 4px;
}

.module-item {
    padding: 4px 8px;
    font-size: 11px;
    font-family: "Consolas", monospace;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
}

.module-item:hover {
    background: #2a2d2e;
}

.module-item .module-name {
    color: #4ec9b0;
}

.module-item .module-addr {
    color: #ce9178;
}

.copy-btn {
    padding: 2px 6px;
    font-size: 10px;
    background: #3d3d3d;
    border: none;
    border-radius: 2px;
    color: #d4d4d4;
    cursor: pointer;
}

.copy-btn:hover {
    background: #4d4d4d;
}

.divider {
    height: 1px;
    background: #3d3d3d;
    margin: 8px 0;
}

.about-btn {
    background: #0e639c !important;
}

.about-btn:hover {
    background: #1177bb !important;
}

.about-modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.about-modal {
    background: #252526;
    border: 1px solid #4fc3f7;
    border-radius: 8px;
    width: 400px;
    max-width: 90%;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.about-modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #3d3d3d;
}

.about-modal-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: #4fc3f7;
    margin: 0;
}

.about-modal-close {
    width: 28px;
    height: 28px;
    border: none;
    background: #3d3d3d;
    color: #d4d4d4;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
}

.about-modal-close:hover {
    background: #e81123;
    color: white;
}

.about-modal-content {
    padding: 20px;
}

.about-description {
    color: #b0b0b0;
    font-size: 13px;
    line-height: 1.5;
    margin-bottom: 16px;
}

.about-divider {
    height: 1px;
    background: #3d3d3d;
    margin: 16px 0;
}

.about-info {
    display: flex;
    align-items: center;
    margin-bottom: 10px;
}

.about-label {
    color: #9e9e9e;
    font-size: 12px;
    width: 80px;
}

.about-value {
    color: #4ec9b0;
    font-size: 13px;
    font-weight: 500;
}

.about-link {
    color: #4fc3f7;
    text-decoration: none;
    font-size: 13px;
}

.about-link:hover {
    text-decoration: underline;
    color: #81d4fa;
}

.about-discord {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: #2d2d2d;
    border-radius: 6px;
    text-align: center;
}

.about-discord span {
    color: #9e9e9e;
    font-size: 12px;
}

.discord-link {
    color: #7289da !important;
    font-weight: 500;
}

.discord-link:hover {
    color: #99aab5 !important;
}

.about-footer {
    margin-top: 16px;
    text-align: center;
}

.about-footer span {
    color: #6e6e6e;
    font-size: 11px;
}

.editable {
    cursor: pointer;
    transition: background-color 0.15s;
}

.editable:hover {
    background-color: #3d3d3d;
}

.edit-input {
    width: 100%;
    padding: 2px 4px;
    border: 1px solid #4fc3f7;
    border-radius: 2px;
    background: #1e1e1e;
    color: #4fc3f7;
    font-family: "Consolas", monospace;
    font-size: 11px;
    outline: none;
}

.edit-input:focus {
    border-color: #81d4fa;
    box-shadow: 0 0 4px rgba(79, 195, 247, 0.3);
}

/* Resize handles for borderless window */
.resize-handle {
    position: fixed;
    z-index: 9999;
}

.resize-handle-n {
    top: 0;
    left: 8px;
    right: 8px;
    height: 4px;
    cursor: n-resize;
}

.resize-handle-s {
    bottom: 0;
    left: 8px;
    right: 8px;
    height: 4px;
    cursor: s-resize;
}

.resize-handle-e {
    top: 8px;
    right: 0;
    bottom: 8px;
    width: 4px;
    cursor: e-resize;
}

.resize-handle-w {
    top: 8px;
    left: 0;
    bottom: 8px;
    width: 4px;
    cursor: w-resize;
}

.resize-handle-nw {
    top: 0;
    left: 0;
    width: 8px;
    height: 8px;
    cursor: nw-resize;
}

.resize-handle-ne {
    top: 0;
    right: 0;
    width: 8px;
    height: 8px;
    cursor: ne-resize;
}

.resize-handle-sw {
    bottom: 0;
    left: 0;
    width: 8px;
    height: 8px;
    cursor: sw-resize;
}

.resize-handle-se {
    bottom: 0;
    right: 0;
    width: 8px;
    height: 8px;
    cursor: se-resize;
}
"#;
