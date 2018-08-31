ROOT_DIR:=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

build:
	cd .docker/dockerfiles && docker build -t lalabot_build .
	mkdir -p "$(ROOT_DIR)"/.docker/cargo/{registry,git,target}
	docker run --rm -it \
		-v "$(ROOT_DIR)":/source:cached \
		-v "$(ROOT_DIR)"/.docker/cargo/registry:/root/.cargo/registry:cached \
		-v "$(ROOT_DIR)"/.docker/cargo/git:/root/.cargo/git:cached \
		-v "$(ROOT_DIR)"/.docker/cargo/target:/root/target:cached \
		lalabot_build \
		/bin/bash -l -c 'cargo build --release --target-dir /root/target -Zcompile-progress' \
  && strip /root/target/release/lalafell_bot

# clean:
# 	cargo clean
#	  rm -rf "$(pwd)"/.docker/cargo/{registry,git}
