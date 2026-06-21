#include "disconnected_panel.hpp"

#include "enums.hpp"
#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/bootstrap_panel.hpp"
#include "ui/panels/bootstrap_panel/connecting_panel.hpp"
#include "ui/panels/connection_status_panel.hpp"
#include "ui/widgets/wrapped_static_text.hpp"

DisconnectedPanel::DisconnectedPanel(wxWindow* parent) : wxPanel(parent) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto title = new wxStaticText(this, wxID_ANY, Strings::DisconnectedPanel::title());
    title->SetFont(Fonts::title_font());

    auto explainer_text =
        new WrappedStaticText(this, wxID_ANY, Strings::DisconnectedPanel::explainer_text());

    auto button_panel = new wxPanel(this, wxID_ANY);
    auto h_button_sizer = new wxBoxSizer(wxHORIZONTAL);

    auto configure_button =
        new wxButton(button_panel, wxID_ANY, Strings::DisconnectedPanel::configure_button());
    configure_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->configure(); });
    auto connect_button =
        new wxButton(button_panel, wxID_ANY, Strings::DisconnectedPanel::connect_button());
    connect_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->connect(); });

    h_button_sizer->AddStretchSpacer(1);
    h_button_sizer->Add(configure_button, 0, wxRIGHT, 8);
    h_button_sizer->Add(connect_button, 0);

    button_panel->SetSizer(h_button_sizer);

    v_sizer->Add(title, 0, wxALIGN_CENTER | wxBOTTOM, Metrics::VERTICAL_PADDING_LARGE);
    v_sizer->Add(
        explainer_text,
        0,
        wxEXPAND | wxALIGN_LEFT | wxBOTTOM,
        Metrics::VERTICAL_PADDING_MEDIUM
    );
    v_sizer->AddStretchSpacer(1);
    v_sizer->Add(button_panel, 0, wxALIGN_RIGHT);

    this->SetSizerAndFit(v_sizer);
}

void DisconnectedPanel::configure() {
    // todo: open the settings panel to the tor
    LOG_INFO("Configure");
    wxGetApp().get_main_frame().show_settings_panel(Settings::Connection);
}

void DisconnectedPanel::connect() {
    LOG_INFO("Connect");

    auto& main_frame = wxGetApp().get_main_frame();

    main_frame.get_bootstrap_panel_mut().get_connecting_panel_mut().update_progress_bar(0);

    const auto& settings = wxGetApp().get_settings();

    std::unique_ptr<tego_tor_config> tor_config;
    tego_settings_get_tor_config(&settings, tego::out(tor_config), tego::panic_on_error());

    auto& context = wxGetApp().get_context_mut();
    tego_context_begin_bootstrap(&context, tor_config.get(), tego::panic_on_error());

    main_frame.get_connection_status_panel_mut().reset_widgets();
    main_frame.get_bootstrap_panel_mut().show_connecting();
}
