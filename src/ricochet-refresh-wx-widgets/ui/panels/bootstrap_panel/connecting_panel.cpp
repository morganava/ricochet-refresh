#include "connecting_panel.hpp"

#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/bootstrap_panel.hpp"
#include "ui/widgets/wrapped_static_text.hpp"

ConnectingPanel::ConnectingPanel(wxWindow* parent) : wxPanel(parent) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto title = new wxStaticText(this, wxID_ANY, Strings::ConnectingPanel::title());
    title->SetFont(Fonts::title_font());

    auto explainer_text =
        new WrappedStaticText(this, wxID_ANY, Strings::ConnectingPanel::explainer_text());
    this->progress_bar = new wxGauge(this, wxID_ANY, 100);
    auto button_panel = new wxPanel(this, wxID_ANY);
    auto h_button_sizer = new wxBoxSizer(wxHORIZONTAL);

    auto view_logs_button =
        new wxButton(button_panel, wxID_ANY, Strings::ConnectingPanel::view_logs_button());
    view_logs_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->view_logs(); });
    auto cancel_button =
        new wxButton(button_panel, wxID_ANY, Strings::ConnectingPanel::cancel_button());
    cancel_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->cancel(); });

    h_button_sizer->AddStretchSpacer(1);
    h_button_sizer->Add(view_logs_button, 0, wxRIGHT, Metrics::HORIZONTAL_PADDING_MEDIUM);
    h_button_sizer->Add(cancel_button, 0);

    button_panel->SetSizer(h_button_sizer);

    v_sizer->Add(title, 0, wxALIGN_CENTER | wxBOTTOM, Metrics::VERTICAL_PADDING_LARGE);
    v_sizer->Add(
        explainer_text,
        0,
        wxEXPAND | wxALIGN_LEFT | wxBOTTOM,
        Metrics::VERTICAL_PADDING_MEDIUM
    );
    v_sizer->Add(this->progress_bar, 0, wxEXPAND);
    v_sizer->AddStretchSpacer(1);
    v_sizer->Add(button_panel, 0, wxALIGN_RIGHT);

    this->SetSizerAndFit(v_sizer);
}

void ConnectingPanel::update_progress_bar(unsigned n) {
    n = std::min(100u, n);
    this->progress_bar->SetValue(n);
}

void ConnectingPanel::view_logs() {
    LOG_INFO("View Logs");

    wxGetApp().get_main_frame().show_connection_status_panel();
}

void ConnectingPanel::cancel() {
    LOG_INFO("Cancel");

    auto& context = wxGetApp().get_context_mut();
    tego_context_cancel_bootstrap(&context, tego::panic_on_error());

    wxGetApp().get_main_frame().get_bootstrap_panel_mut().show_disconnected();
}
