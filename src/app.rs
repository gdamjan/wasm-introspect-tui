use crate::wasm::inspector::WasmInfo;
use crate::wasm::runtime::WasmRuntime;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Imports,
    Exports,
    Memory,
    Disassembly,
}

impl Tab {
    pub fn next(self) -> Self {
        match self {
            Tab::Imports => Tab::Exports,
            Tab::Exports => Tab::Memory,
            Tab::Memory => Tab::Disassembly,
            Tab::Disassembly => Tab::Imports,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Tab::Imports => Tab::Disassembly,
            Tab::Exports => Tab::Imports,
            Tab::Memory => Tab::Exports,
            Tab::Disassembly => Tab::Memory,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Tab::Imports => "Imports",
            Tab::Exports => "Exports",
            Tab::Memory => "Memory",
            Tab::Disassembly => "WAT",
        }
    }

    pub const ALL: [Tab; 4] = [Tab::Imports, Tab::Exports, Tab::Memory, Tab::Disassembly];
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Invoke,
}

pub struct App {
    pub wasm_info: WasmInfo,
    pub runtime: Option<WasmRuntime>,
    pub active_tab: Tab,
    pub should_quit: bool,

    // List selection state
    pub import_selected: usize,
    pub export_selected: usize,

    // Memory viewer state
    pub memory_offset: usize,
    pub memory_page_size: usize,

    // Disassembly state
    pub wat_lines: Vec<String>,
    pub wat_scroll: usize,

    // Invoke dialog state
    pub input_mode: InputMode,
    pub invoke_export_idx: Option<usize>,
    pub invoke_args: Vec<String>,
    pub invoke_cursor: usize,
    pub invoke_result: Option<String>,
    pub invoke_error: Option<String>,

    pub status_message: Option<String>,
    pub file_path: String,
}

impl App {
    pub fn new(wasm_info: WasmInfo, runtime: Option<WasmRuntime>, file_path: String, wat_lines: Vec<String>) -> Self {
        App {
            wasm_info,
            runtime,
            active_tab: Tab::Imports,
            should_quit: false,
            import_selected: 0,
            export_selected: 0,
            memory_offset: 0,
            memory_page_size: 256,
            wat_lines,
            wat_scroll: 0,
            input_mode: InputMode::Normal,
            invoke_export_idx: None,
            invoke_args: Vec::new(),
            invoke_cursor: 0,
            invoke_result: None,
            invoke_error: None,
            status_message: None,
            file_path,
        }
    }

    pub fn max_imports(&self) -> usize {
        if self.wasm_info.is_component {
            self.wasm_info.component_imports.len()
        } else {
            self.wasm_info.imports.len()
        }
    }

    pub fn max_exports(&self) -> usize {
        if self.wasm_info.is_component {
            self.wasm_info.component_exports.len()
        } else {
            self.wasm_info.exports.len()
        }
    }

    pub fn open_invoke_dialog(&mut self) {
        if self.wasm_info.is_component {
            self.status_message = Some("Function invocation not supported for component model".into());
            return;
        }
        if self.wasm_info.exports.is_empty() {
            self.status_message = Some("No exports available".into());
            return;
        }
        let idx = self.export_selected;
        if idx >= self.wasm_info.exports.len() {
            self.status_message = Some("No export selected".into());
            return;
        }
        let export = &self.wasm_info.exports[idx];
        if !matches!(export.kind, crate::wasm::inspector::ExternKind::Func) {
            self.status_message = Some(format!("'{}' is a {}, not a function", export.name, export.kind));
            return;
        }
        let param_count = export.signature.as_ref().map(|s| s.params.len()).unwrap_or(0);
        self.invoke_export_idx = Some(idx);
        self.invoke_args = vec![String::new(); param_count];
        self.invoke_cursor = 0;
        self.invoke_result = None;
        self.invoke_error = None;
        self.input_mode = InputMode::Invoke;
    }

    pub fn execute_invoke(&mut self) {
        let idx = match self.invoke_export_idx {
            Some(i) => i,
            None => return,
        };
        let export = self.wasm_info.exports[idx].clone();
        if let Some(runtime) = &mut self.runtime {
            match runtime.call_function(&export, &self.invoke_args) {
                Ok(results) => {
                    self.invoke_result = Some(results.join(", "));
                    self.invoke_error = None;
                }
                Err(e) => {
                    self.invoke_error = Some(format!("{:#}", e));
                    self.invoke_result = None;
                }
            }
        } else {
            self.invoke_error = Some("Runtime not available".into());
        }
    }

    pub fn close_invoke_dialog(&mut self) {
        self.input_mode = InputMode::Normal;
        self.invoke_export_idx = None;
        self.invoke_args.clear();
        self.invoke_result = None;
        self.invoke_error = None;
    }
}
