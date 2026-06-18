#include "main.hpp"

#include "locale.hpp"
#include "strings.hpp"
#include "ui/main_frame.hpp"
#include "ui/panels/bootstrap_panel.hpp"
#include "ui/panels/bootstrap_panel/connecting_panel.hpp"

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
                wxGetApp().get_main_frame().get_bootstrap_panel_mut().show_connected();
            });
        },
        tego::panic_on_error()
    );
}