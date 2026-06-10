set(tego_rs_sources
    build.rs
    Cargo.toml
    cbindgen.toml
    include/tego/logger.hpp
    include/tego/tego.hpp
    include/tego/utilities.hpp
    src/callbacks.rs
    src/command_queue.rs
    src/context.rs
    src/error.rs
    src/event_loop_task.rs
    src/ffi/context.rs
    src/ffi/ed25519_private_key.rs
    src/ffi/error.rs
    src/ffi/handle.rs
    src/ffi/logger.rs
    src/ffi/mod.rs
    src/ffi/object_map.rs
    src/ffi/pluggable_transport_config.rs
    src/ffi/settings.rs
    src/ffi/string.rs
    src/ffi/tor_config.rs
    src/ffi/tor_daemon_config.rs
    src/ffi/user_id.rs
    src/ffi/v3_onion_service_id.rs
    src/lib.rs
    src/listener_task.rs
    src/macros.rs
    src/promise.rs
    src/settings.rs
)

set(tego_rs_static_filename "${CMAKE_STATIC_LIBRARY_PREFIX}tego_rs${CMAKE_STATIC_LIBRARY_SUFFIX}" CACHE STRING "tego-rs static library filename" FORCE
)
set(tego_rs_static ${CARGO_TARGET_DIR}/${CARGO_TARGET_TRIPLE}/${CARGO_PROFILE}/${tego_rs_static_filename})
set(tego_rs_include_dir
    ${CARGO_TARGET_DIR}/${CARGO_PROFILE}/include
)

set(tego_rs_outputs
    ${tego_rs_static}
    ${tego_rs_include_dir}/tego.h
)

make_directory(${tego_rs_include_dir}/tego)

# hard-link our authord tego headers to tego-rs' include directory
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/include/tego/tego.hpp ${tego_rs_include_dir}/tego/tego.hpp)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/include/tego/utilities.hpp ${tego_rs_include_dir}/tego/utilities.hpp)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/include/tego/logger.hpp ${tego_rs_include_dir}/tego/logger.hpp)

#
# tego-rs crate feature flags
#
set(TEGO_RS_FEATURE_LIST)
if (ENABLE_RICOCHET_REFRESH_LOGGING)
    list(APPEND TEGO_RS_FEATURE_LIST "logging")
endif()
if(ENABLE_BUNDLED_TOR)
    list(APPEND TEGO_RS_FEATURE_LIST "bundled-tor")
endif()
if(ENABLE_EXTERNAL_TOR)
    list(APPEND TEGO_RS_FEATURE_LIST "external-tor")
endif()
if(ENABLE_ARTI_CLIENT)
    list(APPEND TEGO_RS_FEATURE_LIST "arti-client")
endif()

list(JOIN TEGO_RS_FEATURE_LIST "," TEGO_RS_FEATURES)
if (TEGO_RS_FEATURES)
    set(TEGO_RS_FEATURES "--features" "${TEGO_RS_FEATURES}")
endif()


#
# build tego-rs static lib
#

add_custom_command(
    DEPENDS ${tego_rs_sources}
    OUTPUT ${tego_rs_outputs}
    COMMAND env CARGO_TARGET_DIR=${CARGO_TARGET_DIR} RUSTFLAGS=${RUSTFLAGS} cargo build ${CARGO_FLAGS} ${TEGO_RS_FEATURES}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(tego_rs_target
    DEPENDS ${tego_rs_outputs}
)

#
# create static library target for consumers
#

add_library(tego_rs STATIC IMPORTED GLOBAL)
add_dependencies(tego_rs tego_rs_target)
set_target_properties(tego_rs PROPERTIES
    IMPORTED_LOCATION ${tego_rs_static})
target_include_directories(tego_rs INTERFACE ${tego_rs_include_dir})
if (ENABLE_RICOCHET_REFRESH_LOGGING)
    target_compile_definitions(tego_rs INTERFACE RICOCHET_REFRESH_LOGGING)
    target_link_libraries(tego_rs INTERFACE fmt::fmt)
endif()

# Windows
if (WIN32)
    target_link_libraries(tego_rs INTERFACE ws2_32)
    target_link_libraries(tego_rs INTERFACE userenv)
    target_link_libraries(tego_rs INTERFACE bcrypt)
    target_link_libraries(tego_rs INTERFACE ntdll)
endif()

if (ENABLE_LINTING)
    #
    # Run crate linting
    #
    add_custom_target(tego_rs_cargo_clippy_target
        COMMAND CARGO_TARGET_DIR=${CARGO_TARGET_DIR} RUSTFLAGS=${RUSTFLAGS} cargo clippy ${TEGO_RS_FEATURES}
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
    add_dependencies(linting tego_rs_cargo_clippy_target)
endif()
