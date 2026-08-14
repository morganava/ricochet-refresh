#pragma once

enum class ContactGroup;
class ContactListPanel;
class ChatPanel;
class MessageEntryPanel;

class ConversationsPanel: public wxSplitterWindow {
public:
    ConversationsPanel(wxWindow* parent, tego_session_handle session_handle);
    ~ConversationsPanel();

    void add_user(
        const tego_user_handle user_handle,
        const wxString& display_name,
        const wxBitmap& avatar,
        const ContactGroup contact_group
    );

    void
    change_user_contact_group(const tego_user_handle user_handle, const ContactGroup contact_group);

    void receive_message(
        const tego_user_handle recipient,
        const wxDateTime& timestamp,
        const wxString& message
    );

    bool handle_debug_command(const wxString& cmd, const tego_session_handle user_handle);

private:
    void select_contact(const std::optional<tego_user_handle> contact);
    void remove_contact(tego_user_handle contact);

    tego_session_handle session_handle = TEGO_INVALID_SESSION_HANDLE;

    ContactListPanel* contact_list_panel = nullptr;

    wxPanel* right_panel = nullptr;
    wxBoxSizer* right_v_sizer = nullptr;

    struct ContactWidgets {
        // Data
        wxString display_name;
        // Chat Widgets
        wxBoxSizer* v_sizer;
        ChatPanel* chat_panel;
        MessageEntryPanel* message_entry_panel;
    };

    std::unordered_map<tego_user_handle, ContactWidgets> contact_widgets;
};
