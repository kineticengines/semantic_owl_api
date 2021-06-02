setup:
	rustup component add llvm-tools-preview && \
	cargo install cargo-binutils
test:setup	 
	cargo clippy && \
	RUSTFLAGS="-Z instrument-coverage" LLVM_PROFILE_FILE="kng-sem-owl-%m.profraw" cargo test --tests && \
	cargo profdata -- merge -sparse semantic_owl_api/kng-sem-owl-*.profraw -o kng-sem-owl.profdata && \
	cargo profdata -- merge -sparse semantic_owl_cli/kng-sem-owl-*.profraw -o kng-sem-owl.profdata && \
	cargo cov -- report --use-color --instr-profile=kng-sem-owl.profdata && \
	cargo cov -- show --use-color --instr-profile=kng-sem-owl.profdata