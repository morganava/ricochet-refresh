#pragma once

class UnlockProfilePanel: public wxPanel {
public:
    UnlockProfilePanel(wxWindow* parent, const wxString& profile_path);

private:
    wxPanel* create_password_entry_panel(wxPanel* parent);

    // event handlers
    void on_try_unlock_profile();
    void on_close();

    const wxString profile_path;
    wxTextCtrl* password_entry_textbox = nullptr;
};
