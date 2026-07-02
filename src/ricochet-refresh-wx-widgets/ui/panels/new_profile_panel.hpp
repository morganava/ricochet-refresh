#pragma once

enum class NewProfile;
enum class ProfileCreationStep;

class NewProfilePanel: public wxPanel {
public:
    NewProfilePanel(wxWindow* parent, NewProfile new_profile);

private:
    // event handlers
    void on_cancel();
    void on_back();
    void on_next();
    void on_finish();

    // reset widgets back to default states
    void reset();

    wxPanel* create_generate_paths_panel();
    wxPanel* create_import_legacy_paths_panel();
    wxPanel* create_credentials_panel();
    wxPanel* create_overview_panel();

    ProfileCreationStep current_step;

    // panels
    wxPanel* paths_panel = nullptr;
    wxPanel* credentials_panel = nullptr;
    wxPanel* overview_panel = nullptr;

    // widgets
    wxButton* cancel_button = nullptr;
    wxButton* back_button = nullptr;
    wxButton* next_button = nullptr;
};
