build:
	scarb --profile proving build --package stone_wrapper

execute:
	scarb --profile proving execute \
		--package stone_wrapper \
		--arguments-file tests/data/stwo_adapter_proof.json \
		--print-resource-usage

bootload:
	mkdir -p target
	python3 ../../scripts/data/generate_program_input.py \
		--executable ../../target/proving/stone_wrapper.executable.json \
		--args-file tests/data/stwo_adapter_proof.json \
		--program-hash-function poseidon \
		--output target/program-input.json
	cairo_program_runner \
		--program ../../bootloaders/simple_bootloader_compiled.json \
		--program_input target/program-input.json \
		--air_public_input target/pub.json \
		--air_private_input target/priv.json \
		--trace_file $$PWD/target/trace \
		--memory_file $$PWD/target/memory \
		--layout all_cairo_stwo \
		--proof_mode \
		--disable_trace_padding \
		--merge_extra_segments
