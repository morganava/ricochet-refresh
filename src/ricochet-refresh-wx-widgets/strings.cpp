#include "strings.hpp"

#include "enums.hpp"
#include "ffi.hpp"

wxString Strings::from_utf8(const char8_t str[]) {
    return wxString::FromUTF8(reinterpret_cast<const char*>(str));
}

wxString Strings::Common::ricochet_v4_id_uri(const tego_v3_onion_service_id* service_id) {
    std::unique_ptr<tego_string> service_id_string;
    tego_v3_onion_service_id_to_string(
        service_id,
        tego::out(service_id_string),
        tego::panic_on_error()
    );

    return wxString::Format("ricochet-v4://%s", into_wxString(service_id_string));
}

wxString Strings::FileTransfersPanel::status_formatted(
    FileTransferDirection direction,
    double percent_complete
) {
    switch (direction) {
        case FileTransferDirection::Upload:
            if (percent_complete < 100.0) {
                auto fmt_string = Locale::translate(u8"Uploading {:.1f}%").utf8_string();
                return fmt::format(fmt::runtime(fmt_string), percent_complete);
            } else {
                return Locale::translate(u8"Uploaded 100.0%");
            }
        case FileTransferDirection::Download:
            if (percent_complete < 100.0) {
                auto fmt_string = Locale::translate(u8"Downloading {:.1f}%").utf8_string();
                return fmt::format(fmt::runtime(fmt_string), percent_complete);
            } else {
                return Locale::translate(u8"Downloaded 100.0%");
            }
    }
    TEGO_PANIC_MSG("unknown FileTransferDirection: {}", static_cast<int>(direction));
}

// file size constants for localization calculations
constexpr double KIBIBYTE = 1024.0;
constexpr double MEBIBYTE = KIBIBYTE * 1024.0;
constexpr double GIBIBYTE = MEBIBYTE * 1024.0;
constexpr double TEBIBYTE = GIBIBYTE * 1024.0;

wxString Strings::Common::file_size_formatted(uint64_t bytes) {
    const auto bytes_double = static_cast<double>(bytes);
    if (bytes_double < KIBIBYTE) {
        // a number of bytes (e.g. "128 B")
        const auto fmt_string = Locale::translate(u8"{} B").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes);
    } else if (bytes_double < MEBIBYTE) {
        // a number of kibibytes (e.g. "1.2 KiB"; a kibibyte is 1024 bytes)
        const auto fmt_string = Locale::translate(u8"{:.1f} KiB").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes_double / KIBIBYTE);
    } else if (bytes_double < GIBIBYTE) {
        // a number of mebibytes (e.g. "1.2 MiB"; a mebibyte is 1024 kibibytes)
        const auto fmt_string = Locale::translate(u8"{:.1f} MiB").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes_double / MEBIBYTE);
    } else if (bytes_double < TEBIBYTE) {
        // a number of gibibytes (e.g. "1.2 GiB"; a gibibyte is 1024 mebibytes)
        const auto fmt_string = Locale::translate(u8"{:.1f} GiB").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes_double / MEBIBYTE);
    } else {
        // a number of tebiibytes (e.g. "1.2 TiB"; a tebibyte is 1024 gibibytes)
        const auto fmt_string = Locale::translate(u8"{:.1f} TiB").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes_double / GIBIBYTE);
    }
}

wxString Strings::Common::transfer_speed_formatted(uint64_t bytes_per_second) {
    const auto bytes_per_second_double = static_cast<double>(bytes_per_second);
    if (bytes_per_second_double < KIBIBYTE) {
        // a number of bytes per second (e.g. "128 B/s")
        const auto fmt_string = Locale::translate(u8"{} B/s").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes_per_second);
    } else if (bytes_per_second_double < MEBIBYTE) {
        // a number of kibibytes per second (e.g. "1.2 KiB/s"; a kibibyte is 1024 bytes)
        const auto fmt_string = Locale::translate(u8"{:.1f} KiB/s").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes_per_second_double / KIBIBYTE);
    } else if (bytes_per_second_double < GIBIBYTE) {
        // a number of mebibytes per second (e.g. "1.2 MiB/s"; a mebibyte is 1024 kibibytes)
        const auto fmt_string = Locale::translate(u8"{:.1f} MiB/s").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes_per_second_double / MEBIBYTE);
    } else if (bytes_per_second_double < TEBIBYTE) {
        // a number of gibibytes per second (e.g. "1.2 GiBs"; a gibibyte is 1024 mebibytes)
        const auto fmt_string = Locale::translate(u8"{:.1f} GiB/s").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes_per_second_double / MEBIBYTE);
    } else {
        // a number of tebiibytes per second (e.g. "1.2 TiB/s"; a tebibyte is 1024 gibibytes)
        const auto fmt_string = Locale::translate(u8"{:.1f} TiB/s").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), bytes_per_second_double / GIBIBYTE);
    }
}

wxString Strings::Common::time_span_formatted(const wxTimeSpan& eta) {
    constexpr uint64_t SECONDS_PER_MINUTE = 60;
    constexpr uint64_t MINUTES_PER_HOUR = 60;
    constexpr uint64_t HOURS_PER_DAY = 24;
    constexpr uint64_t DAYS_PER_WEEK = 7;
    constexpr uint64_t WEEKS_PER_YEAR = 52;

    const auto eta_seconds = static_cast<uint64_t>(eta.GetSeconds().GetValue());

    const auto seconds_part = eta_seconds % SECONDS_PER_MINUTE;
    const auto eta_minutes = eta_seconds / SECONDS_PER_MINUTE;

    const auto minutes_part = eta_minutes % MINUTES_PER_HOUR;
    const auto eta_hours = eta_minutes / MINUTES_PER_HOUR;

    const auto hours_part = eta_hours % HOURS_PER_DAY;
    const auto eta_days = eta_hours / HOURS_PER_DAY;

    const auto days_part = eta_days % DAYS_PER_WEEK;
    const auto eta_weeks = eta_days / DAYS_PER_WEEK;

    const auto weeks_part = eta_weeks % WEEKS_PER_YEAR;
    const auto eta_years = eta_weeks / WEEKS_PER_YEAR;

    const auto years_part = eta_years;

    if (years_part > 0) {
        const auto fmt_string = Locale::translate(u8"{years}y {weeks}w {days}d").utf8_string();
        return fmt::format(
            fmt::runtime(fmt_string),
            fmt::arg("years", years_part),
            fmt::arg("weeks", weeks_part),
            fmt::arg("days", days_part)
        );
    } else if (weeks_part > 0) {
        const auto fmt_string = Locale::translate(u8"{weeks}w {days}d {hours}h").utf8_string();
        return fmt::format(
            fmt::runtime(fmt_string),
            fmt::arg("weeks", weeks_part),
            fmt::arg("days", days_part),
            fmt::arg("hours", hours_part)
        );
    } else if (days_part > 0) {
        const auto fmt_string = Locale::translate(u8"{days}d {hours}h {minutes}m").utf8_string();
        return fmt::format(
            fmt::runtime(fmt_string),
            fmt::arg("days", days_part),
            fmt::arg("hours", hours_part),
            fmt::arg("minutes", minutes_part)
        );
    } else if (hours_part > 0) {
        const auto fmt_string = Locale::translate(u8"{hours}h {minutes}m {seconds}s").utf8_string();
        return fmt::format(
            fmt::runtime(fmt_string),
            fmt::arg("hours", hours_part),
            fmt::arg("minutes", minutes_part),
            fmt::arg("seconds", seconds_part)
        );
    } else if (minutes_part > 0) {
        const auto fmt_string = Locale::translate(u8"{minutes}m {seconds}s").utf8_string();
        return fmt::format(
            fmt::runtime(fmt_string),
            fmt::arg("minutes", minutes_part),
            fmt::arg("seconds", seconds_part)
        );
    } else {
        const auto fmt_string = Locale::translate(u8"{seconds}s").utf8_string();
        return fmt::format(fmt::runtime(fmt_string), fmt::arg("seconds", seconds_part));
    }
}