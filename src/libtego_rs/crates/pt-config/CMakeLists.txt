set(pt_config_depends
    Cargo.toml
    build.rs
    src/lib.rs
)

set(pt_config_outputs
    ${CARGO_TARGET_DIR}/${CARGO_PROFILE}/libpt_config.d
    ${CARGO_TARGET_DIR}/${CARGO_PROFILE}/libpt_config.rlib)

# path to the pt_config.json file
get_target_property(PT_CONFIG_JSON tor_expert_bundle PT_CONFIG_JSON)

#
# build target
#
add_custom_command(
    DEPENDS ${pt_config_depends}
    OUTPUT ${pt_config_outputs}
    COMMAND env CARGO_TARGET_DIR=${CARGO_TARGET_DIR} RUSTFLAGS=${RUSTFLAGS} PT_CONFIG_JSON="${PT_CONFIG_JSON}" cargo build ${CARGO_FLAGS}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

add_custom_target(pt_config_target
    DEPENDS ${pt_config_outputs})
add_dependencies(pt_config_target tor_expert_bundle_target)

if (ENABLE_LINTING)
    #
    # Run crate linting
    #
    add_custom_target(pt_config_cargo_clippy_target
        COMMAND CARGO_TARGET_DIR=${CARGO_TARGET_DIR} RUSTFLAGS=${RUSTFLAGS} PT_CONFIG_JSON="${PT_CONFIG_JSON}" cargo clippy
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
    add_dependencies(linting pt_config_cargo_clippy_target)
endif()
