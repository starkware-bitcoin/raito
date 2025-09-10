build-adapter:
	scarb --profile proving build \
		--package stone_wrapper \
		--features stone_adapter \
		--no-default-features

build-verifier:
	scarb --profile proving build \
		--package stone_wrapper \
		--features stone_verifier \
		--no-default-features

execute-adapter:
	scarb --profile proving execute \
		--package stone_wrapper \
		--features stone_adapter \
		--arguments-file tests/data/src_proof.json \
		--print-resource-usage

bootload-adapter:
	mkdir -p target
	python ../../scripts/data/generate_program_input.py \
		--executable ../../target/proving/stone_wrapper.executable.json \
		--args-file tests/data/src_proof.json \
		--program-hash-function poseidon \
		--output target/adapter-input.json
	cairo_program_runner \
		--program ../../bootloaders/simple_bootloader_compiled.json \
		--program_input target/adapter-input.json \
		--air_public_input target/pub.json \
		--air_private_input target/priv.json \
		--trace_file $$PWD/target/trace \
		--memory_file $$PWD/target/memory \
		--layout all_cairo_stwo \
		--proof_mode \
		--disable_trace_padding \
		--merge_extra_segments

prove-adapter:
	adapted_stwo \
		--priv_json target/priv.json \
		--pub_json target/pub.json \
		--params_json params/adapter.json \
		--proof_path target/proof.json \
		--proof-format cairo-serde \
		--verify
