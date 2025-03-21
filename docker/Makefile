
.PHONEY: clean

targets = .build_deps .game_runtime_deps

platforms = linux/arm64,linux/amd64

project_root_path = $(realpath ..)

# Args: dockerfile, full_image_name, touchfile, extra_args
define docker_build
	docker buildx build \
		--platform $(platforms) \
		--progress=plain \
		--load \
		-f $(1) \
		-t $(2) \
		$(4) \
		..
	$(if $(3),touch $(3))
endef

build_deps_image = leighpauls/manytris_build_deps:latest
.build_deps: Dockerfile.build_deps
	$(call docker_build,$<,$(build_deps_image),$@)

game_runtime_deps_image = leighpauls/manytris_game_deps:latest
.game_runtime_deps: Dockerfile.game_runtime_deps
	$(call docker_build,$<,$(game_runtime_deps_image),$@)

game_runtime_image = leighpauls/manytris:dev

game_runtime_args = \
	--build-arg build_deps_image=$(build_deps_image) \
	--build-arg game_runtime_deps_image=$(game_runtime_deps_image) \
	--build-arg BUILD_PROFILE=dev \
	--build-arg TARGET_DIR=debug

runtime_dockerfile = Dockerfile.game_runtime
game_runtime: $(runtime_dockerfile) .build_deps .game_runtime_deps
	$(call docker_build,$(runtime_dockerfile),$(game_runtime_image),,$(game_runtime_args))

clean:
	rm -f $(targets)

