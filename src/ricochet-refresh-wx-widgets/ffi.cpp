#include "ffi.hpp"

std::unique_ptr<tego_string> into_tego_string(const wxString& wx_string) {
    const auto utf8_str = wx_string.utf8_str();

    std::unique_ptr<tego_string> value;
    tego_string_new(tego::out(value), utf8_str.data(), utf8_str.length(), tego::panic_on_error());
    return value;
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