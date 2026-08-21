#include "tego_callbacks.hpp"

#include "ffi.hpp"
#include "strings.hpp"
#include "ui/bitmaps.hpp"
#include "ui/main_frame.hpp"
#include "ui/panels/bootstrap_panel.hpp"
#include "ui/panels/bootstrap_panel/connecting_panel.hpp"
#include "ui/panels/connection_status_panel.hpp"
#include "ui/panels/sessions_notebook.hpp"
#include "ui/panels/sessions_notebook/conversations_panel.hpp"
#include "ui/panels/sessions_notebook/session_panel.hpp"

void TegoCallbacks::init(tego_context* context) {
#define SET_TEGO_CALLBACK_IMPL(callback) \
    tego_context_set_##callback##_callback( \
        context, \
        &TegoCallbacks::on_##callback, \
        tego::panic_on_error() \
    );

    SET_TEGO_CALLBACK_IMPL(tor_provider_initialized)
    SET_TEGO_CALLBACK_IMPL(tor_bootstrap_status_changed)
    SET_TEGO_CALLBACK_IMPL(tor_bootstrap_complete)
    SET_TEGO_CALLBACK_IMPL(tor_log_received)
    SET_TEGO_CALLBACK_IMPL(session_began)
    SET_TEGO_CALLBACK_IMPL(chat_request_received)
    SET_TEGO_CALLBACK_IMPL(chat_request_response_received)
    SET_TEGO_CALLBACK_IMPL(user_added)
    SET_TEGO_CALLBACK_IMPL(user_removed)
    SET_TEGO_CALLBACK_IMPL(user_status_changed)
    SET_TEGO_CALLBACK_IMPL(message_received)
    SET_TEGO_CALLBACK_IMPL(message_acknowledged)
    SET_TEGO_CALLBACK_IMPL(file_transfer_request_received)
    SET_TEGO_CALLBACK_IMPL(file_transfer_progress)
    SET_TEGO_CALLBACK_IMPL(file_transfer_complete)
    SET_TEGO_CALLBACK_IMPL(file_transfer_request_acknowledged)
    SET_TEGO_CALLBACK_IMPL(file_transfer_request_response_received)
}

// tego callbacks
void TegoCallbacks::on_tor_provider_initialized(
    tego_context* context,
    tego_tor_config_type tor_provider_type,
    const tego_string* version
) {
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
}

void TegoCallbacks::on_tor_bootstrap_status_changed(
    tego_context* context,
    int32_t progress,
    tego_tor_bootstrap_tag tag
) {
    wxGetApp().CallAfter([=]() {
        wxGetApp()
            .get_main_frame()
            .get_bootstrap_panel_mut()
            .get_connecting_panel_mut()
            .update_progress_bar(static_cast<unsigned>(progress));
    });
}

void TegoCallbacks::on_tor_bootstrap_complete(tego_context* context) {
    wxGetApp().CallAfter([=]() {
        auto& main_frame = wxGetApp().get_main_frame();
        main_frame.get_bootstrap_panel_mut().show_connected();
        main_frame.get_connection_status_panel_mut().set_connection_status(ConnectionStatus::Online
        );
    });
}

void TegoCallbacks::on_tor_log_received(tego_context* context, const tego_string* line) {
    const auto log_line = into_wxString(line);
    wxGetApp().CallAfter([=]() {
        wxGetApp().get_main_frame().get_connection_status_panel_mut().add_log(log_line);
    });
}

void TegoCallbacks::on_session_began(
    tego_context* context,
    tego_session_handle session_handle,
    const tego_user_handle* user_handles,
    const tego_user_type* user_types,
    const tego_string* const* user_display_names,
    size_t user_count
) {
    // local copy of handles
    std::vector<tego_user_handle> user_handles_copy(user_handles, user_handles + user_count);

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
        auto& sessions_notebook = wxGetApp().get_main_frame().get_sessions_notebook_panel_mut();

        auto session_panel = sessions_notebook.get_session_panel_by_session_handle(session_handle);
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

            conversation_panel->add_user(user_handle, user_display_name, avatar, contact_group);
        }
    });
}

void TegoCallbacks::on_chat_request_received(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    const tego_string* message
) {
    auto message_wxstring = into_wxString(message);

    wxGetApp().CallAfter([=]() {
        auto& sessions_notebook = wxGetApp().get_main_frame().get_sessions_notebook_panel_mut();

        auto session_panel = sessions_notebook.get_session_panel_by_session_handle(session_handle);
        if (!session_panel) {
            return;
        }

        LOG_INFO(fmt::format(
            "Chat Request Received; SessionHandle: {}, UserHandle: {}, Message: {}",
            session_handle,
            user_handle,
            message_wxstring
        ));

        auto conversation_panel = session_panel->get_conversations_panel_mut();
        conversation_panel->change_user_contact_group(user_handle, ContactGroup::Requesting);
        conversation_panel->receive_message(user_handle, wxDateTime::Now(), message_wxstring);
    });
}

void TegoCallbacks::on_chat_request_response_received(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    bool accepted_request
) {
    LOG_INFO(fmt::format(
        "Chat Request Response Receeived; SessionHandle: {}, UserHandle: {} Accepted: {}",
        session_handle,
        user_handle,
        accepted_request
    ));
    if (!accepted_request) {
        wxGetApp().CallAfter([=]() {
            auto& sessions_notebook = wxGetApp().get_main_frame().get_sessions_notebook_panel_mut();

            auto session_panel =
                sessions_notebook.get_session_panel_by_session_handle(session_handle);
            if (!session_panel) {
                return;
            }

            auto conversation_panel = session_panel->get_conversations_panel_mut();
            conversation_panel->change_user_contact_group(user_handle, ContactGroup::Rejected);
        });
    }
}

void TegoCallbacks::on_user_added(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_user_type user_type
) {
    // todo: add user to contact list
    LOG_INFO(fmt::format(
        "User Added; SessionHandle: {}, UserHandle: {}, UserType: {}",
        session_handle,
        user_handle,
        static_cast<int>(user_type)
    ));
}

void TegoCallbacks::on_user_removed(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle
) {
    // todo: remove user from contact list
    LOG_INFO(
        fmt::format("User Removed; SessionHandle: {}, UserHandle: {}", session_handle, user_handle)
    );
}

void TegoCallbacks::on_user_status_changed(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_user_status user_status
) {
    wxGetApp().CallAfter([=]() {
        auto& sessions_notebook = wxGetApp().get_main_frame().get_sessions_notebook_panel_mut();

        auto session_panel = sessions_notebook.get_session_panel_by_session_handle(session_handle);
        if (!session_panel) {
            return;
        }

        assert(user_status == tego_user_status_online || user_status == tego_user_status_offline);

        auto conversation_panel = session_panel->get_conversations_panel_mut();
        conversation_panel->change_user_contact_group(
            user_handle,
            user_status == tego_user_status_online ? ContactGroup::Connected
                                                   : ContactGroup::Disconnected
        );
    });
}

void TegoCallbacks::on_message_received(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_time timestamp,
    tego_message_id message_id,
    const tego_string* message
) {
    auto timestamp_wxdatetime = into_wxDateTime(timestamp);
    auto message_wxstring = into_wxString(message);
    wxGetApp().CallAfter([=]() {
        auto& sessions_notebook = wxGetApp().get_main_frame().get_sessions_notebook_panel_mut();

        auto session_panel = sessions_notebook.get_session_panel_by_session_handle(session_handle);
        if (!session_panel) {
            return;
        }

        auto conversation_panel = session_panel->get_conversations_panel_mut();
        conversation_panel->receive_message(user_handle, timestamp_wxdatetime, message_wxstring);
    });
}

void TegoCallbacks::on_message_acknowledged(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_message_id message_id,
    bool message_accepted
) {
    // todo: signal in UI message has been received
    if (message_accepted) {
        LOG_INFO(fmt::format(
            "Message accepted; SessionHandle: {}, UserHandle: {}, MessageId: {}",
            session_handle,
            user_handle,
            message_id
        ));
    } else {
        LOG_ERROR(fmt::format(
            "Message not accepted; SessionHandle: {}, UserHandle: {}, MessageId: {}",
            session_handle,
            user_handle,
            message_id
        ));
    }
}

void TegoCallbacks::on_file_transfer_request_received(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id file_transfer_id,
    const tego_string* file_name,
    tego_file_size file_size
) {
    // todo: file transfer UI
    LOG_INFO(fmt::format(
        "File transfer request received; SessionHandle: {}, UserHandle: {}, FileTransferId: {}, FileName: {}, FileSize: {}",
        session_handle,
        user_handle,
        file_transfer_id,
        into_wxString(file_name),
        file_size
    ));
}

void TegoCallbacks::on_file_transfer_progress(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id file_transfer_id,
    tego_file_transfer_direction direction,
    tego_file_size bytes_complete,
    tego_file_size bytes_total
) {
    // todo: file transfer UI
    LOG_INFO(fmt::format(
        "File transfer progress: SessionHandle: {}, UserHandle: {}, FileTransferId: {}, Direction: {}, BytesComplete: {}, BytesTotal: {}",
        session_handle,
        user_handle,
        file_transfer_id,
        static_cast<int>(direction),
        bytes_complete,
        bytes_total
    ));
}

void TegoCallbacks::on_file_transfer_complete(
    tego_context* context,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id file_transfer_id,
    tego_file_transfer_direction file_transfer_direction,
    tego_file_transfer_result result
) {
    // todo: file transfer UI
    LOG_INFO(fmt::format(
        "File transfer complete; SessionHandle: {}, UserHandle: {}, FileTransferId: {}, Direction: {}, Result: {}",
        session_handle,
        user_handle,
        file_transfer_id,
        static_cast<int>(file_transfer_direction),
        static_cast<int>(result)
    ));
}

void TegoCallbacks::on_file_transfer_request_acknowledged(
    tego_context*,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id file_transfer_id,
    bool request_acked
) {
    // todo: file transfer UI
    LOG_INFO(fmt::format(
        "File transfer request ack'd: SessionHandle {}, UserHandle: {}, FileTransferId: {}, RequestAcked: {}",
        session_handle,
        user_handle,
        file_transfer_id,
        request_acked
    ));
}

void TegoCallbacks::on_file_transfer_request_response_received(
    tego_context*,
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id file_transfer_id,
    tego_file_transfer_response response
) {
    // todo: file transfer UI
    LOG_INFO(fmt::format(
        "File transfer request response received: SessionHandle {}, UserHandle: {}, FileTransferId: {}, Response: {}",
        session_handle,
        user_handle,
        file_transfer_id,
        static_cast<int>(response)
    ));
}
