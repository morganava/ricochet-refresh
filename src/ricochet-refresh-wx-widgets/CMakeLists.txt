cmake_minimum_required(VERSION 3.25)

project(ricochet-refresh)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

#
# Dependencies
#

find_package(wxWidgets 3.2 REQUIRED COMPONENTS base core)
if (NOT wxWidgets_FOUND)
    message(FATAL_ERROR "wxWidgets version >=3.2 required")
else()
    include(${wxWidgets_USE_FILE})
endif()

#
# Ricochet-Refresh
#

if (WINDOWS)
    set(RESOURCE_FILE platform/windows/ricochet-refresh.rc)
    add_executable(ricochet-refresh-wx-widgets WIN32 ${RESOURCE_FILE})
else()
    add_executable(ricochet-refresh-wx-widgets)
endif()
set_target_properties(ricochet-refresh-wx-widgets PROPERTIES OUTPUT_NAME "ricochet-refresh")

set(RICOCHET_REFRESH_SOURCE
    build_config.hpp
    enums.hpp
    ffi.cpp
    ffi.hpp
    locale.cpp
    locale.hpp
    main.cpp
    main.hpp
    paths.cpp
    paths.hpp
    precomp.hpp
    strings.cpp
    strings.hpp
    tego_callbacks.cpp
    tego_callbacks.hpp
    ui/bitmaps.hpp
    ui/bitmaps.cpp
    ui/events.cpp
    ui/events.hpp
    ui/error_popup.cpp
    ui/error_popup.hpp
    ui/fonts.cpp
    ui/fonts.hpp
    ui/main_frame.cpp
    ui/main_frame.hpp
    ui/metrics.cpp
    ui/metrics.hpp
    ui/palette.cpp
    ui/palette.hpp
    ui/panels/bootstrap_panel.cpp
    ui/panels/bootstrap_panel.hpp
    ui/panels/bootstrap_panel/connected_panel.cpp
    ui/panels/bootstrap_panel/connected_panel.hpp
    ui/panels/bootstrap_panel/connecting_panel.cpp
    ui/panels/bootstrap_panel/connecting_panel.hpp
    ui/panels/bootstrap_panel/disconnected_panel.cpp
    ui/panels/bootstrap_panel/disconnected_panel.hpp
    ui/panels/connection_status_panel.cpp
    ui/panels/connection_status_panel.hpp
    ui/panels/sessions_notebook.cpp
    ui/panels/sessions_notebook.hpp
    ui/panels/sessions_notebook/conversations_panel.cpp
    ui/panels/sessions_notebook/conversations_panel.hpp
    ui/panels/sessions_notebook/conversations_panel/chat_panel.cpp
    ui/panels/sessions_notebook/conversations_panel/chat_panel.hpp
    ui/panels/sessions_notebook/conversations_panel/contact_group_heading_panel.cpp
    ui/panels/sessions_notebook/conversations_panel/contact_group_heading_panel.hpp
    ui/panels/sessions_notebook/conversations_panel/contact_list_panel.cpp
    ui/panels/sessions_notebook/conversations_panel/contact_list_panel.hpp
    ui/panels/sessions_notebook/conversations_panel/contact_panel.cpp
    ui/panels/sessions_notebook/conversations_panel/contact_panel.hpp
    ui/panels/sessions_notebook/conversations_panel/message_entry_panel.cpp
    ui/panels/sessions_notebook/conversations_panel/message_entry_panel.hpp
    ui/panels/sessions_notebook/conversations_panel/user_status_panel.cpp
    ui/panels/sessions_notebook/conversations_panel/user_status_panel.hpp
    ui/panels/sessions_notebook/session_panel.cpp
    ui/panels/sessions_notebook/session_panel.hpp
    ui/panels/sessions_notebook/unlock_profile_panel.cpp
    ui/panels/sessions_notebook/unlock_profile_panel.hpp
    ui/panels/new_profile_panel.cpp
    ui/panels/new_profile_panel.hpp
    ui/panels/settings_panel.cpp
    ui/panels/settings_panel.hpp
    ui/panels/settings_panel/connection_settings_panel.cpp
    ui/panels/settings_panel/connection_settings_panel.hpp
    ui/panels/settings_panel/general_settings_panel.cpp
    ui/panels/settings_panel/general_settings_panel.hpp
    ui/panels/settings_panel/interface_settings_panel.cpp
    ui/panels/settings_panel/interface_settings_panel.hpp
    ui/widgets/wrapped_static_text.cpp
    ui/widgets/wrapped_static_text.hpp
)

target_sources(ricochet-refresh-wx-widgets PRIVATE ${RICOCHET_REFRESH_SOURCE})
# clang-format sources before building
if (ENABLE_FORMATTING)
    add_clang_format_target(ricochet-refresh-wx-widgets)
endif()
target_precompile_headers(ricochet-refresh-wx-widgets PRIVATE precomp.hpp)

target_include_directories(ricochet-refresh-wx-widgets PRIVATE ${wxWidgets_INCLUDE_DIRS})
target_include_directories(ricochet-refresh-wx-widgets PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(ricochet-refresh-wx-widgets PRIVATE ${wxWidgets_LIBRARIES})
target_link_libraries(ricochet-refresh-wx-widgets PRIVATE tego_rs)
target_compile_definitions(ricochet-refresh-wx-widgets PRIVATE RICOCHET_REFRESH_VERSION="${RICOCHET_REFRESH_VERSION}")
# enable our various tor-providers
if (ENABLE_BUNDLED_TOR)
    target_compile_definitions(ricochet-refresh-wx-widgets PRIVATE ENABLE_RICOCHET_REFRESH_BUNDLED_TOR=1)
endif()
if (ENABLE_EXTERNAL_TOR)
    target_compile_definitions(ricochet-refresh-wx-widgets PRIVATE ENABLE_RICOCHET_REFRESH_EXTERNAL_TOR=1)
endif()
if (ENABLE_ARTI_CLIENT)
    target_compile_definitions(ricochet-refresh-wx-widgets PRIVATE ENABLE_RICOCHET_REFRESH_ARTI_CLIENT=1)
endif()
if (ENABLE_TOR_EXPERT_BUNDLE)
    target_compile_definitions(ricochet-refresh-wx-widgets PRIVATE ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS=1)
endif()

#
# Localisation
#

find_program(XGETTEXT_EXECUTABLE NAMES xgettext)
find_program(MSGINIT_EXECUTABLE NAMES msginit)
find_program(MSGMERGE_EXECUTABLE NAMES msgmerge)
find_program(MSGFMT_EXECUTABLE NAMES msgfmt)

if (NOT XGETTEXT_EXECUTABLE)
    message(FATAL_ERROR "xgettext not found; required to build translations")
elseif (NOT MSGINIT_EXECUTABLE)
    message(FATAL_ERROR "msginit not found; required to build translations")
elseif (NOT MSGMERGE_EXECUTABLE)
    message(FATAL_ERROR "msgmerge not found; required to build translations")
elseif (NOT MSGFMT_EXECUTABLE)
    message(FATAL_ERROR "msgfmt not found; required to build translations")
endif()

set(RICOCHET_REFRESH_LANGUAGES
    ar # arabic
    de # german
    es # spanish
    nl # dutch
)

set(RICOCHET_REFRESH_MO_FILES)

# see: https://github.com/wxWidgets/wxWidgets/tree/master/samples/internat
set(XGETTEXT_OPTIONS -ktranslate -ktranslate_plural:1,2 --no-wrap --add-comments --from-code=UTF-8)

make_directory(${CMAKE_CURRENT_SOURCE_DIR}/translations)
foreach (language ${RICOCHET_REFRESH_LANGUAGES})
    set(RICOCHET_REFRESH_PO "${CMAKE_CURRENT_SOURCE_DIR}/translations/${language}-ricochet-refresh.po")
    set(RICOCHET_REFRESH_POT "${CMAKE_CURRENT_BINARY_DIR}/translations/${language}/ricochet-refresh.pot")
    set(RICOCHET_REFRESH_MO "${CMAKE_CURRENT_BINARY_DIR}/${language}/ricochet-refresh.mo")

    # Generate .pot
    add_custom_command(
        OUTPUT ${RICOCHET_REFRESH_POT}
        DEPENDS strings.hpp
        COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_CURRENT_BINARY_DIR}/translations/${language}
        COMMAND ${XGETTEXT_EXECUTABLE} ${XGETTEXT_OPTIONS} -o ${RICOCHET_REFRESH_POT} strings.hpp
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    )

    # Create new .po if it doesn't exist
    if (NOT EXISTS ${RICOCHET_REFRESH_PO})
        find_program(SED_EXECUTABLE NAMES sed)
        if (NOT SED_EXECUTABLE)
            message(FATAL_ERROR "sed not found; required to build new ${RICOCHET_REFRESH_PO}")
        endif()

        add_custom_command(
            OUTPUT ${RICOCHET_REFRESH_PO}
            DEPENDS ${RICOCHET_REFRESH_POT}
            COMMAND ${MSGINIT_EXECUTABLE} -l ${language} --no-translator --no-wrap -i ${RICOCHET_REFRESH_POT} -o ${RICOCHET_REFRESH_PO}
            # Update .po metadata
            COMMAND ${SED_EXECUTABLE} -i "s/translations for PACKAGE package/translations for Ricochet-Refresh/g" ${RICOCHET_REFRESH_PO}
            COMMAND ${SED_EXECUTABLE} -i "s/THE PACKAGE\\'S COPYRIGHT HOLDER/Blueprint for Free Speech/g" ${RICOCHET_REFRESH_PO}
            COMMAND ${SED_EXECUTABLE} -i "s/the PACKAGE package/Ricochet-Refresh/g" ${RICOCHET_REFRESH_PO}
            COMMAND ${SED_EXECUTABLE} -i "s/Project-Id-Version: PACKAGE VERSION/Project-Id-Version: Ricochet-Refresh 3.1/g" ${RICOCHET_REFRESH_PO}
            COMMAND ${SED_EXECUTABLE} -i "s/charset=ASCII/charset=UTF-8/g" ${RICOCHET_REFRESH_PO}
        )
    endif()

    # Compile .mo
    file (MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/${language})
    add_custom_command(
        OUTPUT ${RICOCHET_REFRESH_MO}
        DEPENDS ${RICOCHET_REFRESH_PO} ${RICOCHET_REFRESH_POT}
        COMMAND ${MSGMERGE_EXECUTABLE} --backup=off -NU ${RICOCHET_REFRESH_PO} ${RICOCHET_REFRESH_POT}
        COMMAND ${MSGFMT_EXECUTABLE} -vco ${RICOCHET_REFRESH_MO} ${RICOCHET_REFRESH_PO}
        # touch output so it has a later timestamp than the source/modified in-place pot file
        COMMAND ${CMAKE_COMMAND} -E touch ${RICOCHET_REFRESH_MO}
    )
    list(APPEND RICOCHET_REFRESH_MO_FILES ${RICOCHET_REFRESH_MO})
    install(DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/${language}" TYPE BIN)
endforeach()

add_custom_target(ricochet-refresh-wx-widgets_mo_target ALL
    DEPENDS ${RICOCHET_REFRESH_MO_FILES})
if (ENABLE_FORMATTING)
    # format source first before building translations
    add_dependencies(ricochet-refresh-wx-widgets_mo_target ricochet-refresh-wx-widgets_clang-format_target)
endif()
add_dependencies(ricochet-refresh-wx-widgets ricochet-refresh-wx-widgets_mo_target)
if (ENABLE_LINTING)
    add_dependencies(ricochet-refresh-wx-widgets cppcheck_target)
endif()

#
# Install
#

install(TARGETS ricochet-refresh-wx-widgets)
