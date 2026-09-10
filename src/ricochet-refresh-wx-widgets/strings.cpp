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

wxString Strings::FileTransfersPanel::eta_formatted(const wxTimeSpan& eta) {
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