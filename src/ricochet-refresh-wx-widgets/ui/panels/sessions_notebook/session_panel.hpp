#pragma once

class SessionPanel: public wxPanel {
public:
    SessionPanel(wxWindow* parent, const wxString& profile_path);

    const wxString& get_profile_path() const;
    std::optional<tego_session_handle> get_session_handle() const;

private:
    // event handlers
    void on_profile_unlocked(std::unique_ptr<tego_profile>&& profile);

    // panels

    class UnlockProfilePanel* unlock_profile_panel = nullptr;
    class ConversationsPanel* conversations_panel = nullptr;

    // data

    wxString profile_path;
    std::optional<tego_session_handle> session_handle;
};
