docker_image = docker_developer_environment

help:
	$(info -Targets -----------------------------------------------------------------------------)
	$(info -Development Targets -----------------------------------------------------------------)
	$(info lint                         | run lints with clippy)
	$(info benchmark                    | just for fun, really)
	$(info profile                      | only on linux - run callgrind and annotate it)
	$(info journey-tests                | run all stateless journey test)
	$(info continuous-journey-tests     | run all stateless journey test whenever something changes)
	$(info -- Use docker for all dependencies - run make interactively from there ----------------)
	$(info interactive-developer-environment-in-docker | gives you everything you need to run all targets)

always:

interactive-developer-environment-in-docker:
	docker build -t $(docker_image) - < etc/developer.Dockerfile
	docker run -v $$PWD:/volume -w /volume -it $(docker_image)

target/debug/foobar: always
	cargo build

target/release/foobar: always
	cargo build --release

lint:
	cargo clippy

profile: target/release/foobar
	valgrind --callgrind-out-file=callgrind.profile --tool=callgrind  $< >/dev/null
	callgrind_annotate --auto=yes callgrind.profile

benchmark: target/release/foobar
	hyperfine '$<'

journey-tests: target/debug/foobar
	./tests/stateless-journey.sh $<

continuous-journey-tests:
	watchexec $(MAKE) journey-tests

