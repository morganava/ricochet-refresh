#include "main.hpp"

#include "enums.hpp"
#include "ffi.hpp"
#include "locale.hpp"
#include "strings.hpp"
#include "ui/bitmaps.hpp"
#include "ui/main_frame.hpp"
#include "ui/panels/bootstrap_panel.hpp"
#include "ui/panels/bootstrap_panel/connecting_panel.hpp"
#include "ui/panels/connection_status_panel.hpp"
#include "ui/panels/sessions_notebook.hpp"
#include "ui/panels/sessions_notebook/conversations_panel.hpp"
#include "ui/panels/sessions_notebook/session_panel.hpp"

wxIMPLEMENT_APP(RicochetRefresh);

bool RicochetRefresh::OnInit() try {
    if (!wxApp::OnInit()) {
        return false;
    }

    this->init_settings();

    Locale::init();

    auto main_frame = new MainFrame();
    main_frame->Show(true);
    this->main_frame = main_frame;

    tego_context_initialize(tego::out(this->context), tego::throw_on_error());

    this->init_callbacks();

    return true;

} catch (std::exception& ex) {
    LOG_ERROR(ex.what());
    LOG_FLUSH();
    return false;
}

//
// Override Methods
//

void RicochetRefresh::OnInitCmdLine(wxCmdLineParser& parser) {
    parser.AddUsageText(
        "Ricochet Refresh is a secure, private, anonymous, and metadata-resistant instant messenger\n"
    );
    parser.AddUsageText("Options:");
    parser.AddSwitch("h", "help", "Displays this help message.");
    parser.AddSwitch("v", "version", "Displays version information.");
    parser.AddOption("c", "config", "Path to a custom config file.", wxCMD_LINE_VAL_STRING);
    parser.AddParam(
        "profile(s)",
        wxCMD_LINE_VAL_STRING,
        wxCMD_LINE_PARAM_OPTIONAL | wxCMD_LINE_PARAM_MULTIPLE
    );
}

bool RicochetRefresh::OnCmdLineParsed(wxCmdLineParser& parser) {
    // Handle --help
    if (parser.Found("help")) {
        parser.Usage();
        return false;
    }

    // Handle --version
    if (parser.Found("version")) {
        puts("Ricochet Refresh " RICOCHET_REFRESH_VERSION);
        return false;
    }

    // Get config file if provided
    if (wxString config_file; parser.Found("config", &config_file)) {
        auto config_file_path = std::filesystem::path(config_file.utf8_string());
        LOG_INFO(fmt::format("Load config: {}", config_file_path.string()));

        this->config_to_load = std::move(config_file_path);
    }

    // Get profile paths to load once we bootstrap
    const size_t profile_count = parser.GetParamCount();
    std::vector<std::filesystem::path> profile_paths;
    profile_paths.reserve(profile_count);
    for (size_t i = 0; i < profile_count; i++) {
        const auto profile = parser.GetParam(i).utf8_string();
        const auto profile_path = std::filesystem::path(profile);
        LOG_INFO(fmt::format("Defer load of: \"{}\"", profile_path.string()));

        profile_paths.push_back(std::move(profile_path));
    }
    this->profiles_to_load = std::move(profile_paths);

    return true;
}

//
// Private Methods
//

void RicochetRefresh::init_settings() {
    if (this->config_to_load) {
        const auto& config_path = this->config_to_load->string();
        const auto config_path_data = config_path.data();
        const auto config_path_length = config_path.size();
        tego_settings_load(
            tego::out(this->settings),
            config_path_data,
            config_path_length,
            tego::throw_on_error()
        );
        this->config_to_load = std::nullopt;
    } else {
        tego_settings_load_default(tego::out(this->settings), tego::throw_on_error());
    }
}

void RicochetRefresh::init_callbacks() {
    auto context = this->context.get();
    // tor provider init'd callback
    tego_context_set_tor_provider_initialized_callback(
        context,
        [](tego_context*, tego_tor_config_type tor_provider_type, const tego_string* version) {
            wxString backend;
            switch (tor_provider_type) {
#ifdef ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
                case tego_tor_config_type_bundled_tor: {
                    assert(version != nullptr);
                    backend = Strings::ConnectionStatusPanel::bundled_client_string(
                        "tor",
                        into_wxString(version)
                    );
                } break;
#endif // ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
            }
            wxGetApp().CallAfter([=]() {
                auto& connection_status_panel =
                    wxGetApp().get_main_frame().get_connection_status_panel_mut();
                connection_status_panel.set_backend(backend);
                connection_status_panel.set_connection_status(ConnectionStatus::Connecting);
            });
        },
        tego::panic_on_error()
    );
    // bootstrap status callback
    tego_context_set_tor_bootstrap_status_changed_callback(
        context,
        [](tego_context*, int32_t progress, enum tego_tor_bootstrap_tag) {
            wxGetApp().CallAfter([=]() {
                wxGetApp()
                    .get_main_frame()
                    .get_bootstrap_panel_mut()
                    .get_connecting_panel_mut()
                    .update_progress_bar(static_cast<unsigned>(progress));
            });
        },
        tego::panic_on_error()
    );
    // bootstrap complete callback
    tego_context_set_tor_bootstrap_complete_callback(
        context,
        [](tego_context*) {
            wxGetApp().CallAfter([=]() {
                auto& main_frame = wxGetApp().get_main_frame();
                main_frame.get_bootstrap_panel_mut().show_connected();
                main_frame.get_connection_status_panel_mut().set_connection_status(
                    ConnectionStatus::Online
                );
            });
        },
        tego::panic_on_error()
    );
    // tor log line received
    tego_context_set_tor_log_received_callback(
        context,
        [](tego_context*, const tego_string* line) {
            const auto log_line = into_wxString(line);
            wxGetApp().CallAfter([=]() {
                wxGetApp().get_main_frame().get_connection_status_panel_mut().add_log(log_line);
            });
        },
        tego::panic_on_error()
    );
    // session began received
    tego_context_set_session_began_callback(
        context,
        [](tego_context*,
           tego_session_handle session_handle,
           const tego_user_handle* user_handles,
           const tego_user_type* user_types,
           const tego_string* const* user_display_names,
           size_t user_count) {
            // local copy of handles
            std::vector<tego_user_handle> user_handles_copy(
                user_handles,
                user_handles + user_count
            );

            // local copy of types
            std::vector<tego_user_type> user_types_copy(user_types, user_types + user_count);

            // local copy of names
            std::vector<wxString> user_display_names_copy;
            user_display_names_copy.reserve(user_count);
            std::transform(
                user_display_names,
                user_display_names + user_count,
                std::back_inserter(user_display_names_copy),
                [](const tego_string* value) -> wxString { return into_wxString(value); }
            );

            // add these users to the proper conversation panel
            wxGetApp().CallAfter([=]() {
                auto& sessions_notebook =
                    wxGetApp().get_main_frame().get_sessions_notebook_panel_mut();

                auto session_panel =
                    sessions_notebook.get_session_panel_by_session_handle(session_handle);
                if (!session_panel) {
                    return;
                }

                auto conversation_panel = session_panel->get_conversations_panel_mut();
                if (!conversation_panel) {
                    return;
                }

                for (size_t k = 0; k < user_count; ++k) {
                    const auto user_type = user_types_copy[k];
                    if (user_type == tego_user_type_owner) {
                        continue;
                    }
                    const auto user_handle = user_handles_copy[k];
                    const auto& user_display_name = user_display_names_copy[k];
                    const auto& avatar = Bitmaps::default_avatar();
                    const auto contact_group = [&]() -> ContactGroup {
                        switch (user_type) {
                            case tego_user_type_requesting:
                                return ContactGroup::Requesting;
                            case tego_user_type_rejected:
                                return ContactGroup::Rejected;
                            case tego_user_type_blocked:
                                return ContactGroup::Blocked;
                            case tego_user_type_allowed:
                            case tego_user_type_pending:
                            default:
                                return ContactGroup::Disconnected;
                        }
                    }();

                    conversation_panel
                        ->add_user(user_handle, user_display_name, avatar, contact_group);
                }
            });
        },
        tego::panic_on_error()
    );
}