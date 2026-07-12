#include "strings.hpp"

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
