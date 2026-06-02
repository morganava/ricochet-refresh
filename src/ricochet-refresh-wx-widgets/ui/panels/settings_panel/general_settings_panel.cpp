#include "general_settings_panel.hpp"

#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/metrics.hpp"

GeneralSettingsPanel::GeneralSettingsPanel(wxWindow* parent) :
    wxScrolled<wxPanel>(parent, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxVSCROLL) {
    this->SetScrollRate(0, this->FromDIP(Metrics::VSCROLL_RATE));

    auto h_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    // Startup

    auto startup_heading =
        new wxStaticText(this, wxID_ANY, Strings::GeneralSettingsPanel::startup_heading());
    startup_heading->SetFont(Fonts::heading_font());

    this->start_only_single_instance_toggle = new wxCheckBox(
        this,
        wxID_ANY,
        Strings::GeneralSettingsPanel::start_only_single_instance_toggle()
    );
    this->start_only_single_instance_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_start_only_single_instance(evt.IsChecked());
    });

    this->check_for_updates_on_launch_toggle = new wxCheckBox(
        this,
        wxID_ANY,
        Strings::GeneralSettingsPanel::check_for_updates_on_launch_toggle()
    );
    this->check_for_updates_on_launch_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_check_for_updates(evt.IsChecked());
    });

    // Layout

    v_sizer->Add(startup_heading, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(
        this->start_only_single_instance_toggle,
        0,
        wxBOTTOM,
        Metrics::VERTICAL_PADDING_MEDIUM
    );
    v_sizer->Add(
        this->check_for_updates_on_launch_toggle,
        0,
        wxBOTTOM,
        Metrics::VERTICAL_PADDING_MEDIUM
    );

    h_sizer->Add(v_sizer, 1, wxEXPAND | wxLEFT | wxRIGHT, Metrics::HORIZONTAL_PADDING_MEDIUM);
    this->SetSizerAndFit(h_sizer);

    this->load_from_settings();
}

void GeneralSettingsPanel::load_from_settings() {
    const auto& settings = wxGetApp().get_settings();

    // start only single instance
    tego_bool start_only_single_instance = TEGO_FALSE;
    tego_settings_get_start_only_single_instance(
        &settings,
        &start_only_single_instance,
        tego::panic_on_error()
    );
    this->start_only_single_instance_toggle->SetValue(start_only_single_instance);

    // check for updates automatically
    tego_bool check_for_updates_automatically = TEGO_FALSE;
    tego_settings_get_check_for_updates_automatically(
        &settings,
        &check_for_updates_automatically,
        tego::panic_on_error()
    );
    this->check_for_updates_on_launch_toggle->SetValue(check_for_updates_automatically);
}

void GeneralSettingsPanel::save_to_settings() {
    auto& settings = wxGetApp().get_settings_mut();

    const bool start_only_single_instance = this->start_only_single_instance_toggle->GetValue();
    tego_settings_set_start_only_single_instance(
        &settings,
        start_only_single_instance ? TEGO_TRUE : TEGO_FALSE,
        tego::panic_on_error()
    );

    const bool check_for_updates_automatically =
        this->check_for_updates_on_launch_toggle->GetValue();
    tego_settings_set_check_for_updates_automatically(
        &settings,
        check_for_updates_automatically ? TEGO_TRUE : TEGO_FALSE,
        tego::panic_on_error()
    );
}

void GeneralSettingsPanel::set_start_only_single_instance(bool enabled) {
    LOG_INFO(fmt::format("Set Start only single instance: : {}", enabled));
}

void GeneralSettingsPanel::set_check_for_updates(bool enabled) {
    LOG_INFO(fmt::format("Set Check for updates on launch: : {}", enabled));
}
