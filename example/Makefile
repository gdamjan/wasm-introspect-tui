
default: component.wasm

%.wasm: %.wat
	wat2wasm -o $@ $<


component.wasm: adder.wasm wit/adder.wit
	wasm-tools component embed wit/adder.wit adder.wasm -o component.wasm
