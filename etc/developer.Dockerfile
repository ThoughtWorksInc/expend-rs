from guangie88/rustfmt-clippy:nightly

run cargo install hyperfine watchexec pact-stub-server

run apt-get update
run apt-get install -y valgrind

env PATH=$PATH:/root/.cargo/bin
