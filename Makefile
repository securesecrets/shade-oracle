# INTERNAL CONTRACT DIRECTORIES
contracts_dir=contracts
compiled_dir=compiled
checksum_dir=${compiled_dir}/checksum

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

# ORACLES = oracle_router proxy_band_oracle secretswap_lp_oracle siennaswap_lp_oracle siennaswap_lp_spot_oracle shade_staking_derivative_oracle earn_v1_oracle mock_band
ORACLES = proxy_band_oracle siennaswap_lp_spot_oracle shade_staking_derivative_oracle oracle_router siennaswap_lp_oracle
CONTRACTS = ${ORACLES}

COMPILED = ${CONTRACTS:=.wasm.gz}

release: build_release compress

debug: build_debug compress

build_release:
	(cd ${contracts_dir}; RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked)

build_debug:
	(cd ${contracts_dir}; RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown)

deploy-testnet:
	cd packages/shade_oracles_integration && export RUST_BACKTRACE=full && cargo run --bin "deploy"

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
	