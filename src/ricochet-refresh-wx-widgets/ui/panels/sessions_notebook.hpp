#pragma once

class SessionsNotebook: public wxPanel {
public:
    explicit SessionsNotebook(wxWindow* parent);
    void open_session(const wxString& profile_path);
    void close_focused_session();
    void close_session(class SessionPanel*);

private:
    wxNotebook* session_notebook = nullptr;

    std::optional<class SessionPanel*> get_session_data_by_profile_path(const wxString& path);
    std::optional<class SessionPanel*> get_session_data_by_session_handle(tego_session_handle handle
    );
};
