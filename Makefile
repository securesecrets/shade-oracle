# INTERNAL CONTRACT DIRECTORIES
contracts_dir=contracts
compiled_dir=compiled
checksum_dir=${compiled_dir}/checksum

build-release=RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

# Compresses the wasm file, args: compressed_file_name, built_file_name
define opt_and_compress = 
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$(2).wasm -o ./$(1).wasm
echo $(md5sum $(1).wasm | cut -f 1 -d " ") >> ${checksum_dir}/$(1).txt
cat ./$(1).wasm | gzip -n -9 > ${compiled_dir}/$(1).wasm.gz
rm ./$(1).wasm
endef

#ORACLES = proxy_band_oracle siennaswap_lp_spot_oracle shade_staking_derivative_oracle oracle_router siennaswap_lp_oracle siennaswap_market_oracle shadeswap_market_oracle index_oracle
ORACLES = siennaswap_lp_spot_oracle oracle_router
MOCKS = mock_band mock_sienna_pair mock_shade_pair

CONTRACTS = ${ORACLES} ${MOCKS}

PKGS = shade_oracles shade_oracles_ensemble shade_oracles_integration

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

compress_all: setup
	@$(MAKE) $(addprefix compress-,$(CONTRACTS))

compress-%: setup
	$(call opt_and_compress,$*,$*)

test:
	@$(MAKE) $(addprefix test-,$(CONTRACTS))

test-%: %
	(cd ${contracts_dir}/$*; cargo test)

$(CONTRACTS): setup
	(cd ${contracts_dir}/$@; ${build-release})
	@$(MAKE) $(addprefix compress-,$(@))

$(PKGS):
	(cd packages/$@; ${build-release})

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
	find . -name "Cargo.lock" -delete
	rm -rf target
	rm -r $(compiled_dir)
	rm -r $(checksum_dir)

format:
	cargo fmt
	
