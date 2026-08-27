#include "main.hpp"

#include "build_config.hpp"
#include "enums.hpp"
#include "ffi.hpp"
#include "locale.hpp"
#include "strings.hpp"
#include "tego_callbacks.hpp"
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

    #if defined(__WINDOWS__)
    // attach to parent proces console so we can get terminal output on windows
    if constexpr (BuildConfig::logging_enabled()) {
        if (::AttachConsole(ATTACH_PARENT_PROCESS)) {
            std::freopen("CONOUT$", "w", stdout);
        }
    }
    #endif

    this->init_settings();

    Locale::init();

    this->main_frame = new MainFrame();

    tego_context_initialize(tego::out(this->context), tego::throw_on_error());

    TegoCallbacks::init(this->context.get());

    this->CallAfter([this]() {
        this->main_frame->init();
        this->main_frame->Show(true);
    });

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
