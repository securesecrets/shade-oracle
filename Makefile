# INTERNAL CONTRACT DIRECTORIES
contracts_dir=contracts
compiled_dir=compiled
checksum_dir=${compiled_dir}/checksum
sub_dirs = oracles mocks strategies

# EXTERNAL CONTRACT DIRECTORIES
external_dir=external_projects
external_compiled_dir=external_projects/external_compiled
external_checksum_dir=${external_compiled_dir}/checksum
sienna_dir=external_projects/SiennaNetwork/contracts

# Compresses the wasm file, args: compressed_file_name, built_file_name
define compress_wasm =
{ \
(cd $(contracts_dir)/$(1); cargo unit-test);\
TARGET_FILE=`echo $(2) | cut -f 2 -d /`;\
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$$TARGET_FILE.wasm -o ./$$TARGET_FILE.wasm;\
echo $(md5sum $$TARGET_FILE.wasm | cut -f 1 -d " ") >> ${checksum_dir}/$(1).txt;\
cat ./$$TARGET_FILE.wasm | gzip -n -9 > ${compiled_dir}/$(1).wasm.gz;\
rm ./$$TARGET_FILE.wasm;\
}
endef

# Build external project
#define build_external =
#(cp $(external_dir)/makefiles/$(1) $(external_dir)/$(1)/Makefile)
#(cd $(external_dir)/$(1); make release)
#endef
define build_external =
(cd $(sienna_dir)/$(1);)
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$(2).wasm -o ./$(1).wasm
echo $(md5sum $(1).wasm | cut -f 1 -d " ") >> ${external_checksum_dir}/$(1).txt
cat ./$(1).wasm | gzip -n -9 > ${external_compiled_dir}/$(1).wasm.gz
rm ./$(1).wasm
endef

docker_name=shade-lend_sn-node_1

ORACLES = oracles/oracle_router oracles/proxy_band_oracle oracles/lp_oracle oracles/earn_v1_oracle
CORE = vault overseer liquidation fee_router
MOCK = mocks/mock_band mocks/mock_lp mocks/mock_farm mocks/mock_strategy
TOKENS = snip20 earn_token_v1
CONTRACTS_EXTERNAL = amm_snip20 exchange factory ido launchpad lp_token snip20_sienna

CONTRACTS = overseer vault oracles/oracle_router liquidation #${TOKENS} ${CORE} ${MOCK} ${ORACLES}
#CONTRACTS = mocks/mock_strategy mocks/mock_farm
#CONTRACTS_EXTERNAL = amm-snip20 exchange factory ido launchpad lp-token mgmt rpt snip20-sienna wrapped wrapped-fixes

COMPILED = ${CONTRACTS:=.wasm.gz}
DOCKER_EXEC = docker exec ${docker_name} /bin/bash -c

release: build_release compress

debug: build_debug compress

# For whatever reason, when you build a sandbox that uses SiennaNetwork packages, it throws an error but if you just do
# cargo run --bin $(INSERT-SANDBOX-HERE) manually in the sandbox folder then the sandbox will run properly
sienna: build_sienna compress_external

build_release:
	(cd ${contracts_dir}; RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked)

build_debug:
	(cd ${contracts_dir}; RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --features="debug-print")

build_sienna:
	(cd ${sienna_dir}; RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown)

compress: setup $(CONTRACTS);

compress_external: setup_external $(CONTRACTS_EXTERNAL);

$(CONTRACTS_EXTERNAL):
	$(call build_external,$@,$@)

setup_external: $(external_compiled_dir) $(external_checksum_dir)

$(external_compiled_dir) $(external_checksum_dir):
	mkdir $@

$(CONTRACTS):
	$(call compress_wasm,$@,$@)

setup: $(compiled_dir) $(checksum_dir) $(sub_dirs)

$(sub_dirs):
	mkdir -p $(compiled_dir)/$@
	mkdir -p $(checksum_dir)/$@

$(compiled_dir) $(checksum_dir):
	mkdir -p $@

check:
	cargo check

clippy:
	cargo clippy

clean:
	rm -r $(compiled_dir)

format:
	cargo fmt

slt: start-local-testnet
it: integration-test

start-local-testnet:
	docker-compose up --force-recreate --build

# To run this, substitute sbeembox with desired target -> make sandbox ARGS="sbeembox"
sandbox:
	${DOCKER_EXEC} "cd code/packages/mulberry_integration_tests && export RUST_BACKTRACE=full && cargo run --bin $(ARGS)"

GLOB = *
# To run this, substitute the * with desired regex to filter tests
# Run all test files with oracles in their name -> make integration-test GLOB="oracles"
integration-test:
	${DOCKER_EXEC} "cd code/packages/mulberry_integration_tests && cargo test -- --nocapture --test ${GLOB} --test-threads=1"

mvp-1-box=mvp1box
mvp-1-box:
	${DOCKER_EXEC} "cd code/packages/mulberry_integration_tests && cargo run --bin ${mvp-1-box}"

mvp-1: debug start-local-testnet mvp-1-box
