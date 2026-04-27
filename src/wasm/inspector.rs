use anyhow::Result;
use wasmparser::{Parser, Payload, ValType, TypeRef, ComponentExternalKind};
use std::fmt;

#[derive(Debug, Clone)]
pub enum WasmValType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
    Other(String),
}

impl fmt::Display for WasmValType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmValType::I32 => write!(f, "i32"),
            WasmValType::I64 => write!(f, "i64"),
            WasmValType::F32 => write!(f, "f32"),
            WasmValType::F64 => write!(f, "f64"),
            WasmValType::V128 => write!(f, "v128"),
            WasmValType::FuncRef => write!(f, "funcref"),
            WasmValType::ExternRef => write!(f, "externref"),
            WasmValType::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<ValType> for WasmValType {
    fn from(vt: ValType) -> Self {
        match vt {
            ValType::I32 => WasmValType::I32,
            ValType::I64 => WasmValType::I64,
            ValType::F32 => WasmValType::F32,
            ValType::F64 => WasmValType::F64,
            ValType::V128 => WasmValType::V128,
            ValType::FUNCREF => WasmValType::FuncRef,
            ValType::EXTERNREF => WasmValType::ExternRef,
            other => WasmValType::Other(format!("{:?}", other)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExternKind {
    Func,
    Table,
    Memory,
    Global,
    Tag,
    #[allow(dead_code)]
    Unknown(String),
}

impl fmt::Display for ExternKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExternKind::Func => write!(f, "func"),
            ExternKind::Table => write!(f, "table"),
            ExternKind::Memory => write!(f, "memory"),
            ExternKind::Global => write!(f, "global"),
            ExternKind::Tag => write!(f, "tag"),
            ExternKind::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub params: Vec<WasmValType>,
    pub results: Vec<WasmValType>,
}

impl fmt::Display for FuncSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params: Vec<String> = self.params.iter().map(|p| p.to_string()).collect();
        let results: Vec<String> = self.results.iter().map(|r| r.to_string()).collect();
        write!(f, "({}) -> ({})", params.join(", "), results.join(", "))
    }
}

#[derive(Debug, Clone)]
pub struct WasmImport {
    pub module: String,
    pub name: String,
    pub kind: ExternKind,
    pub signature: Option<FuncSignature>,
}

#[derive(Debug, Clone)]
pub struct WasmExport {
    pub name: String,
    pub kind: ExternKind,
    pub index: u32,
    pub signature: Option<FuncSignature>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MemoryInfo {
    pub index: u32,
    pub initial_pages: u64,
    pub max_pages: Option<u64>,
    pub shared: bool,
    pub memory64: bool,
}

#[derive(Debug, Clone)]
pub struct ComponentImport {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct ComponentExport {
    pub name: String,
    pub kind: String,
}

#[derive(Debug)]
pub struct WasmInfo {
    pub is_component: bool,
    pub imports: Vec<WasmImport>,
    pub exports: Vec<WasmExport>,
    pub memories: Vec<MemoryInfo>,
    pub component_imports: Vec<ComponentImport>,
    pub component_exports: Vec<ComponentExport>,
}

fn component_external_kind_str(kind: ComponentExternalKind) -> String {
    match kind {
        ComponentExternalKind::Module => "module".to_string(),
        ComponentExternalKind::Func => "func".to_string(),
        ComponentExternalKind::Value => "value".to_string(),
        ComponentExternalKind::Type => "type".to_string(),
        ComponentExternalKind::Instance => "instance".to_string(),
        ComponentExternalKind::Component => "component".to_string(),
    }
}

pub fn inspect(wasm_bytes: &[u8]) -> Result<WasmInfo> {
    let parser = Parser::new(0);
    let mut is_component = false;
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut memories = Vec::new();
    let mut component_imports = Vec::new();
    let mut component_exports = Vec::new();

    // Collect function type signatures
    let mut func_types: Vec<FuncSignature> = Vec::new();
    // Map from function index to type index
    let mut func_type_indices: Vec<u32> = Vec::new();
    let mut import_func_count: u32 = 0;

    for payload in parser.parse_all(wasm_bytes) {
        let payload = payload?;
        match payload {
            Payload::Version { encoding, .. } => {
                if encoding == wasmparser::Encoding::Component {
                    is_component = true;
                }
            }
            Payload::TypeSection(reader) => {
                for func_type in reader.into_iter_err_on_gc_types() {
                    let func_type = func_type?;
                    let params: Vec<WasmValType> = func_type.params().iter().map(|p| WasmValType::from(*p)).collect();
                    let results: Vec<WasmValType> = func_type.results().iter().map(|r| WasmValType::from(*r)).collect();
                    func_types.push(FuncSignature { params, results });
                }
            }
            Payload::ImportSection(reader) => {
                for item in reader.into_imports_with_offsets() {
                    let (_offset, import) = item?;
                    let (kind, signature) = match import.ty {
                        TypeRef::Func(type_idx) | TypeRef::FuncExact(type_idx) => {
                            import_func_count += 1;
                            let sig = func_types.get(type_idx as usize).cloned();
                            (ExternKind::Func, sig)
                        }
                        TypeRef::Table(_) => (ExternKind::Table, None),
                        TypeRef::Memory(_) => (ExternKind::Memory, None),
                        TypeRef::Global(_) => (ExternKind::Global, None),
                        TypeRef::Tag(_) => (ExternKind::Tag, None),
                    };
                    imports.push(WasmImport {
                        module: import.module.to_string(),
                        name: import.name.to_string(),
                        kind,
                        signature,
                    });
                }
            }
            Payload::FunctionSection(reader) => {
                for func in reader {
                    let type_idx = func?;
                    func_type_indices.push(type_idx);
                }
            }
            Payload::MemorySection(reader) => {
                for (i, mem) in reader.into_iter().enumerate() {
                    let mem = mem?;
                    memories.push(MemoryInfo {
                        index: i as u32,
                        initial_pages: mem.initial,
                        max_pages: mem.maximum,
                        shared: mem.shared,
                        memory64: mem.memory64,
                    });
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    let kind = match export.kind {
                        wasmparser::ExternalKind::Func | wasmparser::ExternalKind::FuncExact => ExternKind::Func,
                        wasmparser::ExternalKind::Table => ExternKind::Table,
                        wasmparser::ExternalKind::Memory => ExternKind::Memory,
                        wasmparser::ExternalKind::Global => ExternKind::Global,
                        wasmparser::ExternalKind::Tag => ExternKind::Tag,
                    };
                    // Resolve function signature for exported functions
                    let signature = if matches!(kind, ExternKind::Func) {
                        let func_idx = export.index;
                        if func_idx < import_func_count {
                            // It's an imported function re-exported
                            imports.get(func_idx as usize).and_then(|i| i.signature.clone())
                        } else {
                            let local_idx = (func_idx - import_func_count) as usize;
                            func_type_indices.get(local_idx)
                                .and_then(|&type_idx| func_types.get(type_idx as usize))
                                .cloned()
                        }
                    } else {
                        None
                    };
                    exports.push(WasmExport {
                        name: export.name.to_string(),
                        kind,
                        index: export.index,
                        signature,
                    });
                }
            }
            Payload::ComponentImportSection(reader) => {
                for import in reader {
                    let import = import?;
                    component_imports.push(ComponentImport {
                        name: import.name.0.to_string(),
                        kind: component_external_kind_str(import.ty.kind()),
                    });
                }
            }
            Payload::ComponentExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    component_exports.push(ComponentExport {
                        name: export.name.0.to_string(),
                        kind: component_external_kind_str(export.kind),
                    });
                }
            }
            _ => {}
        }
    }

    Ok(WasmInfo {
        is_component,
        imports,
        exports,
        memories,
        component_imports,
        component_exports,
    })
}
