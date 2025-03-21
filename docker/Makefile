
.PHONEY: clean game_runtime_dev game_runtime_prod

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

game_runtime_image = leighpauls/manytris:$(1)

# Args: build_profile, target_dir
define game_runtime_args
	--build-arg build_deps_image=$(build_deps_image) \
	--build-arg game_runtime_deps_image=$(game_runtime_deps_image) \
	--build-arg BUILD_PROFILE=$(1) \
	--build-arg TARGET_DIR=$(2)
endef

runtime_dockerfile = Dockerfile.game_runtime
runtime_deps = $(runtime_dockerfile) .build_deps .game_runtime_deps

game_runtime_dev: $(runtime_deps)
	$(call docker_build,$(runtime_dockerfile),$(call game_runtime_image,dev),,$(call game_runtime_args,dev,debug))

game_runtime_prod: $(runtime_deps)
	$(call docker_build,$(runtime_dockerfile),$(call game_runtime_image,prod),,$(call game_runtime_args,release,release))


clean:
	rm -f $(targets)

