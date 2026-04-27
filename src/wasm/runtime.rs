use anyhow::{Result, bail, Context};
use wasmtime::*;

use crate::wasm::inspector::{WasmExport, ExternKind, WasmValType};

pub struct WasmRuntime {
    store: Store<()>,
    instance: Instance,
}

fn parse_val(s: &str, vt: &WasmValType) -> Result<Val> {
    let s = s.trim();
    match vt {
        WasmValType::I32 => Ok(Val::I32(s.parse::<i32>().context("expected i32")?)),
        WasmValType::I64 => Ok(Val::I64(s.parse::<i64>().context("expected i64")?)),
        WasmValType::F32 => Ok(Val::F32(s.parse::<f32>().context("expected f32")?.to_bits())),
        WasmValType::F64 => Ok(Val::F64(s.parse::<f64>().context("expected f64")?.to_bits())),
        other => bail!("unsupported parameter type: {}", other),
    }
}

fn wasmtime_valtype_to_wasm(vt: &ValType) -> WasmValType {
    match vt {
        ValType::I32 => WasmValType::I32,
        ValType::I64 => WasmValType::I64,
        ValType::F32 => WasmValType::F32,
        ValType::F64 => WasmValType::F64,
        ValType::V128 => WasmValType::V128,
        ValType::Ref(r) => WasmValType::Other(format!("{}", r)),
    }
}

pub fn format_val(v: &Val) -> String {
    match v {
        Val::I32(n) => format!("{}", n),
        Val::I64(n) => format!("{}", n),
        Val::F32(bits) => format!("{}", f32::from_bits(*bits)),
        Val::F64(bits) => format!("{}", f64::from_bits(*bits)),
        Val::V128(n) => format!("v128(0x{:032x})", u128::from(*n)),
        other => format!("{:?}", other),
    }
}

impl WasmRuntime {
    pub fn new(wasm_bytes: &[u8]) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes)?;
        let mut store = Store::new(&engine, ());
        let mut linker = Linker::new(&engine);

        // Provide stub implementations for imports
        for import in module.imports() {
            match import.ty() {
                ExternType::Func(func_ty) => {
                    let results: Vec<ValType> = func_ty.results().collect();
                    linker.func_new(
                        import.module(),
                        import.name(),
                        func_ty.clone(),
                        move |_caller, _params, results_out| {
                            for (i, result) in results_out.iter_mut().enumerate() {
                                *result = match results.get(i) {
                                    Some(ValType::I32) => Val::I32(0),
                                    Some(ValType::I64) => Val::I64(0),
                                    Some(ValType::F32) => Val::F32(0),
                                    Some(ValType::F64) => Val::F64(0),
                                    _ => Val::I32(0),
                                };
                            }
                            Ok(())
                        },
                    )?;
                }
                ExternType::Memory(mem_ty) => {
                    let memory = Memory::new(&mut store, mem_ty)?;
                    linker.define(&store, import.module(), import.name(), memory)?;
                }
                ExternType::Table(table_ty) => {
                    let init = Ref::Func(None);
                    let table = Table::new(&mut store, table_ty, init)?;
                    linker.define(&store, import.module(), import.name(), table)?;
                }
                ExternType::Global(global_ty) => {
                    let init = match global_ty.content() {
                        ValType::I32 => Val::I32(0),
                        ValType::I64 => Val::I64(0),
                        ValType::F32 => Val::F32(0),
                        ValType::F64 => Val::F64(0),
                        _ => Val::I32(0),
                    };
                    let global = Global::new(&mut store, global_ty, init)?;
                    linker.define(&store, import.module(), import.name(), global)?;
                }
                ExternType::Tag(_) => {}
            }
        }

        let instance = linker.instantiate(&mut store, &module)?;
        Ok(WasmRuntime { store, instance })
    }

    pub fn call_function(&mut self, export: &WasmExport, arg_strings: &[String]) -> Result<Vec<String>> {
        if !matches!(export.kind, ExternKind::Func) {
            bail!("'{}' is not a function", export.name);
        }

        let func = self.instance
            .get_func(&mut self.store, &export.name)
            .context(format!("function '{}' not found", export.name))?;

        let func_ty = func.ty(&self.store);
        let param_types: Vec<ValType> = func_ty.params().collect();
        let result_types: Vec<ValType> = func_ty.results().collect();

        if arg_strings.len() != param_types.len() {
            bail!("expected {} arguments, got {}", param_types.len(), arg_strings.len());
        }

        let sig = export.signature.as_ref();
        let params: Vec<Val> = if let Some(sig) = sig {
            arg_strings.iter().zip(sig.params.iter())
                .map(|(s, vt)| parse_val(s, vt))
                .collect::<Result<Vec<_>>>()?
        } else {
            arg_strings.iter().zip(param_types.iter())
                .map(|(s, vt)| {
                    let wvt = wasmtime_valtype_to_wasm(vt);
                    parse_val(s, &wvt)
                })
                .collect::<Result<Vec<_>>>()?
        };

        let mut results = vec![Val::I32(0); result_types.len()];
        func.call(&mut self.store, &params, &mut results)?;

        Ok(results.iter().map(format_val).collect())
    }

    pub fn read_memory(&mut self, offset: usize, length: usize) -> Result<Vec<u8>> {
        let memory = self.instance
            .get_memory(&mut self.store, "memory")
            .context("no 'memory' export found")?;

        let data = memory.data(&self.store);
        let end = (offset + length).min(data.len());
        if offset >= data.len() {
            return Ok(Vec::new());
        }
        Ok(data[offset..end].to_vec())
    }

    pub fn memory_size(&mut self) -> Result<usize> {
        let memory = self.instance
            .get_memory(&mut self.store, "memory")
            .context("no 'memory' export found")?;
        Ok(memory.data_size(&self.store))
    }
}
