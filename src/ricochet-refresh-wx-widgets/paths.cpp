#include "paths.hpp"

#include "build_config.hpp"

wxFileName Paths::home() {
    return wxFileName::DirName(wxFileName::GetHomeDir());
}

wxFileName Paths::config_directory() {
    auto path = Paths::home();
    if constexpr (constexpr auto os = BuildConfig::operating_system();
                  os == OperatingSystem::Windows) {
        path.AppendDir("AppData");
        path.AppendDir("Local");
        path.AppendDir("ricochet-refresh");
    } else if constexpr (os == OperatingSystem::macOS) {
        path.AppendDir("Library");
        path.AppendDir("Preferences");
        path.AppendDir("ricochet-refresh");
    } else if constexpr (os == OperatingSystem::Linux) {
        path.AppendDir(".config");
        path.AppendDir("ricochet-refresh");
    }
    LOG_INFO(fmt::format("config directory: {}", path.GetAbsolutePath()));
    return path;
}

wxFileName Paths::executable_path() {
    return wxFileName(wxStandardPaths::Get().GetExecutablePath());
}
