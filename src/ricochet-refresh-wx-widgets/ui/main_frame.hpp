#pragma once

enum class Settings;

class MainFrame: public wxFrame {
public:
    MainFrame();

    void init();

    // main panel methods
    void show_bootstrap_panel();
    void show_sessions_notebook_panel();

    // overlay panel methods
    void show_settings_panel(Settings);
    void show_connection_status_panel();
    void show_generate_profile_panel();
    void show_import_legacy_profile_panel();
    void hide_overlay_panel();

    // panel getters
    class BootstrapPanel& get_bootstrap_panel_mut() {
        return *this->main_panels.bootstrap_panel;
    }

    class SessionsNotebook& get_sessions_notebook_panel_mut() {
        return *this->main_panels.sessions_notebook_panel;
    }

    class ConnectionStatusPanel& get_connection_status_panel_mut() {
        return *this->overlay_panels.connection_status_panel;
    }

    //misc
    void open_profile(const wxString& profile_path);

    void enable_settings_menu_item(bool enable);

private:
    void setup_menubar();
    void setup_main_panels(wxBoxSizer* sizer);
    void setup_overlay_panels(wxBoxSizer* sizer);

    void show_main_panel(wxPanel* panel);
    void show_overlay_panel(wxPanel* panel);

    // Event Handlers
    void on_open_profile(wxCommandEvent&);
    void on_new_profile(wxCommandEvent&);
    void on_import_legacy(wxCommandEvent&);
    void on_close_profile(wxCommandEvent&);

    void on_settings(wxCommandEvent&);

    // menus and menu items
    wxMenuItem* settings_menu_item = nullptr;

    // main panels
    // we only move 'forward' through the main panels (e.g. from the bootstrap panel
    // to the profile_notebook_panel, etc).
    struct {
        wxPanel* current = nullptr;
        class BootstrapPanel* bootstrap_panel = nullptr;
        class SessionsNotebook* sessions_notebook_panel = nullptr;
    } main_panels;

    // overlay panels
    // the overlay panels can come and go and are always valid/available for viewing
    struct {
        wxPanel* current = nullptr;
        class SettingsPanel* settings_panel = nullptr;
        class ConnectionStatusPanel* connection_status_panel = nullptr;
        class NewProfilePanel* generate_profile_panel = nullptr;
        class NewProfilePanel* import_legacy_profile_panel = nullptr;
    } overlay_panels;
};
