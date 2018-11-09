docker_image = docker_developer_environment

help:
	$(info -Targets -----------------------------------------------------------------------------)
	$(info -Development Targets -----------------------------------------------------------------)
	$(info lint                         | run lints with clippy)
	$(info benchmark                    | just for fun, really)
	$(info profile                      | only on linux - run callgrind and annotate it)
	$(info journey-tests                | run all stateless journey test)
	$(info continuous-journey-tests     | run all stateless journey test whenever something changes)
	$(info cdc                          | generate the pact to expensify - is used in journey-tests)
	$(info unit                         | run Rust unit tests)
	$(info continuous-unit              | continuously run Rust unit tests)
	$(info -- Use docker for all dependencies - run make interactively from there ----------------)
	$(info interactive-developer-environment-in-docker | gives you everything you need to run all targets)

always:

interactive-developer-environment-in-docker:
	docker build -t $(docker_image) - < etc/developer.Dockerfile
	docker run -v $$PWD:/volume -w /volume -it $(docker_image)

target/debug/expend: always
	cargo build

target/release/expend: always
	cargo build --release

lint:
	cargo clippy --tests

profile: target/release/expend
	valgrind --callgrind-out-file=callgrind.profile --tool=callgrind  $< >/dev/null
	callgrind_annotate --auto=yes callgrind.profile

benchmark: target/release/expend
	hyperfine '$<'

journey-tests: target/debug/expend
	./tests/stateless-journey.sh $<

continuous-journey-tests:
	watchexec $(MAKE) journey-tests

cdc:
	cargo test --test=client_pact

unit:
	cargo test --test=unit

continuous-unit:
	watchexec $(MAKE) unit

