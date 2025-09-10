STWO_REV ?= c273eed286f20681d9e94aa7a7125b8fea8d7a85

install:
	RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
		cargo install --force \
		--git https://github.com/starkware-libs/stwo-cairo \
		--rev $(STWO_REV) \
		adapted_stwo

build:
	scarb --profile proving build --package stwo_wrapper

execute:
	scarb --profile proving execute \
		--package stwo_wrapper \
		--arguments-file tests/data/raito_proof.json \
		--print-resource-usage

bootload:
	mkdir -p target
	python3 ../../scripts/data/generate_program_input.py \
		--executable ../../target/proving/stwo_wrapper.executable.json \
		--args-file tests/data/raito_proof.json \
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

prove:
	adapted_stwo \
		--priv_json target/priv.json \
		--pub_json target/pub.json \
		--params_json prover_params.json \
		--proof_path target/proof.json \
		--proof-format cairo-serde \
		--verify
