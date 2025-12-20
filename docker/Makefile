
.PHONEY: \
	build_deps \
	game_runtime_deps \
	manager_deps \
	game_runtime_dev \
	game_runtime_prod \
	manager_dev \
	manager_prod \
	publish_dev \
	publish_prod \
	local_dev



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

#
# Dependency images, used to minimize build times spent in `apt install ...`
#

build_deps_image = leighpauls/manytris_build_deps:latest
build_deps: Dockerfile.build_deps
	$(call docker_build,$<,$(build_deps_image))

game_runtime_deps_image = leighpauls/manytris_game_deps:latest
game_runtime_deps: Dockerfile.game_runtime_deps
	$(call docker_build,$<,$(game_runtime_deps_image))

manager_deps_image = leighpauls/manytris_manager_deps:latest
manager_deps: Dockerfile.manager_deps
	$(call docker_build,$<,$(manager_deps_image))

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
runtime_dep_rules = $(runtime_dockerfile) build_deps game_runtime_deps

game_runtime_dev: $(runtime_dep_rules)
	$(call docker_build,$(runtime_dockerfile),$(game_runtime_local_dev),$(call game_runtime_args,dev,debug))

game_runtime_prod: $(runtime_dep_rules)
	$(call docker_build,$(runtime_dockerfile),$(game_runtime_local_prod),$(call game_runtime_args,release,release))

manager_image = leighpauls/manytris-manager:$(strip $(1))
manager_local_dev = $(call manager_image,dev)
manager_local_prod = $(call manager_image,prod)

# Args: build_profile, target_dir
define manager_args
	--build-arg build_deps_image=$(build_deps_image) \
	--build-arg manager_deps_image=$(manager_deps_image) \
	--build-arg BUILD_PROFILE=$(1) \
	--build-arg TARGET_DIR=$(2)
endef

manager_dockerfile = Dockerfile.manager
manager_dep_rules = $(manager_dockerfile) build_deps manager_deps
manager_dev: $(manager_dep_rules)
	$(call docker_build,$(manager_dockerfile),$(manager_local_dev),$(call manager_args,dev,debug))

manager_prod: $(manager_dep_rules)
	$(call docker_build,$(manager_dockerfile),$(manager_local_prod),$(call manager_args,release,release))

version_file_name = $(realpath ../version.txt)

version = $(shell cat $(version_file_name))
game_runtime_remote_dev_image = $(call game_runtime_image,$(version)-dev)
game_runtime_remote_prod_image = $(call game_runtime_image,$(version)-prod)

manager_remote_dev_image = $(call manager_image,$(version)-dev)
manager_remote_prod_image = $(call manager_image,$(version)-prod)

# Args: local_image, remote image
define tag_and_push
	docker tag $(1) $(2)
	docker push --quiet $(2)
endef

local_dev: game_runtime_dev manager_dev

publish_dev: game_runtime_dev manager_dev $(version_file_name)
	$(call tag_and_push,$(game_runtime_local_dev),$(game_runtime_remote_dev_image))
	$(call tag_and_push,$(manager_local_dev),$(manager_remote_dev_image))

publish_prod: game_runtime_prod manager_prod $(version_file_name)
	$(call tag_and_push,$(game_runtime_local_prod),$(game_runtime_remote_prod_image))
	$(call tag_and_push,$(manager_local_prod),$(manager_remote_prod_image))

