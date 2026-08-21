#pragma once

class TegoCallbacks {
public:
    static void init(tego_context* context);

private:
    // tego callbacks
    static void on_tor_provider_initialized(
        tego_context* context,
        tego_tor_config_type tor_provider_type,
        const tego_string* version
    );

    static void on_tor_bootstrap_status_changed(
        tego_context* context,
        int32_t progress,
        tego_tor_bootstrap_tag tag
    );

    static void on_tor_bootstrap_complete(tego_context* context);

    static void on_tor_log_received(tego_context* context, const tego_string* line);

    static void on_session_began(
        tego_context* context,
        tego_session_handle session_handle,
        const tego_user_handle* user_handles,
        const tego_user_type* user_types,
        const tego_string* const* user_display_names,
        size_t user_count
    );

    static void on_chat_request_received(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        const tego_string* message
    );

    static void on_chat_request_response_received(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        bool accepted_request
    );

    static void on_user_added(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_user_type user_type
    );

    static void on_user_removed(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle
    );

    static void on_user_status_changed(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_user_status user_status
    );

    static void on_message_received(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_time timestamp,
        tego_message_id message_id,
        const tego_string* message
    );

    static void on_message_acknowledged(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_message_id message_id,
        bool message_accepted
    );

    static void on_file_transfer_request_received(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id file_transfer_id,
        const tego_string* file_name,
        tego_file_size file_size
    );

    static void on_file_transfer_progress(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id file_transfer_id,
        tego_file_transfer_direction direction,
        tego_file_size bytes_complete,
        tego_file_size bytes_total
    );

    static void on_file_transfer_complete(
        tego_context* context,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id file_transfer_id,
        tego_file_transfer_direction file_transfer_direction,
        tego_file_transfer_result result
    );

    static void on_file_transfer_request_acknowledged(
        tego_context*,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id file_transfer_id,
        bool request_acked
    );

    static void on_file_transfer_request_response_received(
        tego_context*,
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id file_transfer_id,
        tego_file_transfer_response response
    );
};
