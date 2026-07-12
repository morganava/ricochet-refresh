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
    void on_copy_ricochet_id();

    void on_get_new_profile_destination();
    void on_get_legacy_profile_destination();

    void on_identity_private_key_changed();
    void on_display_name_changed();
    void on_password_changed();
    void on_confirmed_password_changed();

    void update_next_button();
    void update_confirm_password_background();

    // reset widgets back to default states
    void reset();

    wxPanel* create_generate_paths_panel();
    wxPanel* create_import_legacy_paths_panel();
    wxPanel* create_credentials_panel();
    wxPanel* create_overview_panel();

    NewProfile new_profile_type;
    ProfileCreationStep current_step;

    // panels
    wxPanel* paths_panel = nullptr;
    wxPanel* credentials_panel = nullptr;
    wxPanel* overview_panel = nullptr;

    // widgets
    struct {
        wxTextCtrl* legacy_profile_destination_textbox = nullptr;
        wxTextCtrl* profile_destination_textbox = nullptr;
        std::unique_ptr<tego_ed25519_private_key> identity_private_key;
    } paths;

    struct {
        wxTextCtrl* display_name_textbox = nullptr;
        wxTextCtrl* password_textbox = nullptr;
        wxTextCtrl* confirm_password_textbox = nullptr;
    } credentials;

    struct {
        wxTextCtrl* profile_destination_textbox = nullptr;
        wxTextCtrl* ricochet_id_textbox = nullptr;
    } overview;

    wxButton* cancel_button = nullptr;
    wxButton* back_button = nullptr;
    wxButton* next_button = nullptr;
};
