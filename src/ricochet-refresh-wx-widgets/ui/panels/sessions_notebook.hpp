#pragma once

class SessionsNotebook: public wxPanel {
public:
    explicit SessionsNotebook(wxWindow* parent);
    void open_session(const wxString& profile_path);

private:
    wxNotebook* session_notebook = nullptr;
};
