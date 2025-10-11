install-stone:
	wget -O /tmp/stone-prover-linux-x86_64.deb https://github.com/dipdup-io/stone-packaging/releases/latest/download/stone-prover-linux-x86_64.deb
	sudo dpkg -i /tmp/stone-prover-linux-x86_64.deb

install-swiftness:
	cargo install --git https://github.com/m-kus/integrity-calldata-generator --rev e6206805dfe481cbd8f1fbf2629957ae505a8828 swiftness

build:
	scarb --profile release build --package stone_wrapper

execute:
	scarb execute \
		--package stone_wrapper \
		--arguments-file tests/data/stwo_adapter_proof.json \
		--print-resource-usage

bootload:
	mkdir -p target
	python3 ../../scripts/data/generate_program_input.py \
		--executable target/release/stone_wrapper.executable.json \
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
		--layout recursive_with_poseidon \
		--proof_mode

prove:
	cpu_air_prover -v=1 \
		--logtostderr \
		--parameter_file stone_params.json \
		--prover_config_file stone_config.json \
		--private_input_file target/priv.json \
		--public_input_file target/pub.json \
		--out_file target/proof.json \
		--generate_annotations true

calldata:
	rm -rf ./tests/data/calldata
	mkdir ./tests/data/calldata
	swiftness --proof target/proof.json --layout recursive_with_poseidon --hasher keccak_160_lsb --stone-version stone6 --out tests/data/calldata

register-fact:
	JOB_ID=$$((RANDOM % 10000 + 1)); \
	echo "Job $$JOB_ID"; \
	./scripts/register_fact.sh $$JOB_ID recursive_with_poseidon keccak_160_lsb stone6 cairo1
