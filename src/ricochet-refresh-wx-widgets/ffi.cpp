#include "ffi.hpp"

std::unique_ptr<tego_string> into_tego_string(const wxString& value) {
    const auto utf8_str = value.utf8_str();

    std::unique_ptr<tego_string> result;
    tego_string_new(tego::out(result), utf8_str.data(), utf8_str.length(), tego::panic_on_error());
    return result;
}

wxString into_wxString(const std::unique_ptr<tego_string>& value) {
    return into_wxString(value.get());
}

wxString into_wxString(const tego_string* value) {
    size_t size = 0;
    tego_string_get_size(value, &size, tego::panic_on_error());

    std::unique_ptr<char[]> data = std::make_unique<char[]>(size);
    tego_string_get_data(value, data.get(), size, tego::panic_on_error());

    return wxString::FromUTF8Unchecked(data.get(), size - 1);
}

tego_time into_tego_time(const wxDateTime& value) {
    return static_cast<tego_time>(std::max<int64_t>(value.GetValue().GetValue(), 0));
}

wxDateTime into_wxDateTime(const tego_time value) {
    auto seconds = static_cast<time_t>(value / 1000);
    auto milliseconds = static_cast<unsigned short>(value % 1000);

    return wxDateTime(seconds).SetMillisecond(milliseconds);
}