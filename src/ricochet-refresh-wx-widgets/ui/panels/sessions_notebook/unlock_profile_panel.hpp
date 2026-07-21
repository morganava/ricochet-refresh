#pragma once

class UnlockProfilePanel: public wxPanel {
public:
    UnlockProfilePanel(wxWindow* parent, const wxString& profile_path);

private:
    wxPanel* create_password_entry_panel(wxPanel* parent, const wxString& profile_path);
};
