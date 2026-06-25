if (NOT TOR_EXPERT_BUNDLE_ARCHIVE)
    message(FATAL_ERROR "TOR_EXPERT_BUNDLE_ARCHIVE must be defined (e.g. -DTOR_EXPERT_BUNDLE_ARCHIVE=/path/to/tor-expert-bundle.tar.gz)")
endif()

if (NOT TOR_EXPERT_BUNDLE_PLUGGABLE_TRANSPORTS)
    message(FATAL_ERROR "TOR_EXPERT_BUNDLE_PLUGGABLE_TRANSPORTS must be defined (e.g. -DTOR_EXPERT_BUNDLE_PLUGGABLE_TRANSPORTS='conjure-client;lyrebird')")
endif()

# extract the tarball
set (TOR_EXPERT_BUNDLE_EXTRACT_DIR "${CMAKE_CURRENT_BINARY_DIR}/_extract-tor-expert-bundle")
file (MAKE_DIRECTORY "${TOR_EXPERT_BUNDLE_EXTRACT_DIR}")
execute_process(
    COMMAND ${CMAKE_COMMAND} -E tar xzf "${TOR_EXPERT_BUNDLE_ARCHIVE}"
    WORKING_DIRECTORY "${TOR_EXPERT_BUNDLE_EXTRACT_DIR}"
    COMMAND_ERROR_IS_FATAL ANY
)

# copy pluggable-transports to output directory
file (MAKE_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/pluggable_transports")
foreach(pluggable_transport ${TOR_EXPERT_BUNDLE_PLUGGABLE_TRANSPORTS})
    execute_process(
        COMMAND ${CMAKE_COMMAND} -E copy "${TOR_EXPERT_BUNDLE_EXTRACT_DIR}/tor/pluggable_transports/${pluggable_transport}${CMAKE_EXECUTABLE_SUFFIX}" "${CMAKE_CURRENT_BINARY_DIR}/pluggable_transports/"
        COMMAND_ERROR_IS_FATAL ANY
    )
endforeach()

# copy pt_config.json to output directory and remove the temp extract directory
set(PT_CONFIG_JSON "${CMAKE_CURRENT_BINARY_DIR}/pt_config.json")
execute_process(
    COMMAND ${CMAKE_COMMAND} -E copy "${TOR_EXPERT_BUNDLE_EXTRACT_DIR}/tor/pluggable_transports/pt_config.json" "${PT_CONFIG_JSON}"
    COMMAND ${CMAKE_COMMAND} -E rm -rf "${TOR_EXPERT_BUNDLE_EXTRACT_DIR}")

# construct list of pt binaries
set (PLUGGABLE_TRANSPORT_BINARIES)
foreach(pluggable_transport ${TOR_EXPERT_BUNDLE_PLUGGABLE_TRANSPORTS})
    list(APPEND PLUGGABLE_TRANSPORT_BINARIES "${CMAKE_CURRENT_BINARY_DIR}/pluggable_transports/${pluggable_transport}${CMAKE_EXECUTABLE_SUFFIX}")
endforeach()

# setup custom target
add_library(tor_expert_bundle INTERFACE)
add_custom_target(tor_expert_bundle_target ALL
    DEPENDS ${PT_CONFIG_JSON} ${PLUGGABLE_TRANSPORT_BINARIES})
set_target_properties(tor_expert_bundle PROPERTIES
    PT_CONFIG_JSON "${PT_CONFIG_JSON}")

# install target
install(DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/pluggable_transports" TYPE BIN USE_SOURCE_PERMISSIONS)
