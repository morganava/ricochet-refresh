#pragma once

enum class OperatingSystem {
    Windows,
    macOS,
    Linux,
};

class BuildConfig {
public:
    constexpr static OperatingSystem operating_system() {
#if defined(__WINDOWS__)
        return OperatingSystem::Windows;
#elif defined(__APPLE__)
        return OperatingSystem::macOS;
#elif defined(__LINUX__)
        return OperatingSystem::Linux;
#else
    // cppcheck-suppress preprocessorErrorDirective
    #error "Target operating system not known at build-time"
#endif
    }
};
