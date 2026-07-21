#pragma once

class ChatPanel;
class MessageEntryPanel;

class ConversationsPanel: public wxSplitterWindow {
public:
    ConversationsPanel(wxWindow* parent, tego_session_handle session_handle);
    ~ConversationsPanel();

    void receive_message(
        const tego_user_handle recipient,
        const wxDateTime& timestamp,
        const wxString& message
    );

private:
    void select_contact(const std::optional<tego_user_handle> contact);
    void remove_contact(tego_user_handle contact);

    tego_session_handle session_handle = TEGO_INVALID_SESSION_HANDLE;

    wxBoxSizer* right_v_sizer = nullptr;

    struct ContactWidgets {
        wxBoxSizer* v_sizer;
        // Chat Widgets
        ChatPanel* chat_panel;
        MessageEntryPanel* message_entry_panel;
    };

    std::unordered_map<tego_user_handle, ContactWidgets> contact_widgets;
};
