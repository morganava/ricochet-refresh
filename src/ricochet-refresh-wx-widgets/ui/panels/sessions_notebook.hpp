#pragma once

class SessionsNotebook: public wxPanel {
public:
    explicit SessionsNotebook(wxWindow* parent);
    void open_session(const wxString& profile_path);
    void close_focused_session();
    void close_session(class SessionPanel*);

    class SessionPanel* get_session_panel_by_session_handle(tego_session_handle handle);

private:
    wxNotebook* session_notebook = nullptr;

    class SessionPanel* get_session_panel_by_profile_path(const wxString& path);
};
