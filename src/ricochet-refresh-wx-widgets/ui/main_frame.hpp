#pragma once

class MainFrame: public wxFrame {
public:
    MainFrame();

private:
    void setup_menubar();
    void setup_main_panels(wxBoxSizer* sizer);
    void setup_overlay_panels(wxBoxSizer* sizer);

    // Event Handlers
    void on_exit(wxCommandEvent&) {
        this->Close(true);
    }

    // main panels
    class BootstrapPanel* bootstrap_panel = nullptr;
    // todo: implement this
    class SessionsPanel* profile_notebook_panel = nullptr;

    // overlay panels
    class SettingsPanel* settings_panel = nullptr;
    class ConnectionStatusPanel* connection_status_panel = nullptr;
};
