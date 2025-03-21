
.PHONEY: build_deps game_runtime_deps game_runtime_dev game_runtime_prod publish_dev publish_prod

platforms = linux/arm64,linux/amd64

project_root_path = $(realpath ..)

# Args: dockerfile, full_image_name, [extra_args]
define docker_build
	docker buildx build \
		--platform $(platforms) \
		--progress=plain \
		-f $(1) \
		-t $(2) \
		$(3) \
		..
endef

build_deps_image = leighpauls/manytris_build_deps:latest
build_deps: Dockerfile.build_deps
	$(call docker_build,$<,$(build_deps_image))

game_runtime_deps_image = leighpauls/manytris_game_deps:latest
game_runtime_deps: Dockerfile.game_runtime_deps
	$(call docker_build,$<,$(game_runtime_deps_image))

game_runtime_image = leighpauls/manytris:$(strip $(1))
game_runtime_local_dev = $(call game_runtime_image,dev)
game_runtime_local_prod = $(call game_runtime_image,prod)

# Args: build_profile, target_dir
define game_runtime_args
	--build-arg build_deps_image=$(build_deps_image) \
	--build-arg game_runtime_deps_image=$(game_runtime_deps_image) \
	--build-arg BUILD_PROFILE=$(1) \
	--build-arg TARGET_DIR=$(2)
endef

runtime_dockerfile = Dockerfile.game_runtime
runtime_deps = $(runtime_dockerfile) build_deps game_runtime_deps

game_runtime_dev: $(runtime_deps)
	$(call docker_build,$(runtime_dockerfile),$(game_runtime_local_dev),$(call game_runtime_args,dev,debug))

game_runtime_prod: $(runtime_deps)
	$(call docker_build,$(runtime_dockerfile),$(game_runtime_local_prod),$(call game_runtime_args,release,release))

version_file_name = version.txt

version = $(shell cat $(version_file_name))
remote_dev_image = $(call game_runtime_image,$(version)-dev)
remote_prod_image = $(call game_runtime_image,$(version)-prod)

publish_dev: game_runtime_dev $(version_file_name)
	docker tag $(game_runtime_local_dev) $(remote_dev_image)
	docker push --quiet $(remote_dev_image)

publish_prod: game_runtime_prod $(version_file_name)
	docker tag $(game_runtime_local_prod) $(remote_prod_image)
	docker push --quiet $(remote_prod_image)
