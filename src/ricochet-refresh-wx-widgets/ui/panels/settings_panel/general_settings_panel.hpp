#pragma once

class GeneralSettingsPanel: public wxScrolled<wxPanel> {
public:
    explicit GeneralSettingsPanel(wxWindow* parent);

    // load settings from disk and populate the ui
    void load_from_settings();
    // save settings from ui to disk
    void save_to_settings();

private:
    // setters
    void set_start_only_single_instance(bool);
    void set_check_for_updates(bool);

    // widgets
    wxCheckBox* start_only_single_instance_toggle = nullptr;
    wxCheckBox* check_for_updates_on_launch_toggle = nullptr;
};
