#include "unlock_profile_panel.hpp"

#include "ffi.hpp"
#include "strings.hpp"
#include "ui/events.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/sessions_notebook.hpp"
#include "ui/panels/sessions_notebook/session_panel.hpp"

constexpr int BOOTSTRAP_PANEL_MIN_WIDTH = 600;
constexpr int BOOTSTRAP_PANEL_MIN_HEIGHT = 400;

UnlockProfilePanel::UnlockProfilePanel(wxWindow* parent, const wxString& profile_path) :
    wxPanel(parent),
    profile_path(profile_path) {
    auto root_h_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto h_centered_panel = new wxPanel(this, wxID_ANY);
    h_centered_panel->SetMinSize(wxSize(BOOTSTRAP_PANEL_MIN_WIDTH, -1));

    root_h_sizer->AddStretchSpacer(1);
    root_h_sizer->Add(h_centered_panel, 2, wxEXPAND);
    root_h_sizer->AddStretchSpacer(1);

    auto v_sizer = new wxBoxSizer(wxVERTICAL);
    h_centered_panel->SetSizer(v_sizer);

    auto v_centered_panel =
        new wxPanel(h_centered_panel, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxBORDER_RAISED);

    v_sizer->AddStretchSpacer(1);
    v_sizer->Add(v_centered_panel, 0, wxEXPAND);
    v_sizer->AddStretchSpacer(1);

    auto center_sizer = new wxBoxSizer(wxVERTICAL);

    auto password_entry_panel = this->create_password_entry_panel(v_centered_panel);

    center_sizer->Add(password_entry_panel, 1, wxEXPAND | wxALL, Metrics::PADDING_XLARGE);

    v_centered_panel->SetSizer(center_sizer);

    this->SetSizerAndFit(root_h_sizer);
}

wxPanel* UnlockProfilePanel::create_password_entry_panel(wxPanel* parent) {
    auto panel = new wxPanel(parent);

    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto title = new wxStaticText(panel, wxID_ANY, Strings::UnlockProfilePanel::title());
    title->SetFont(Fonts::title_font());

    auto profile_path_heading = new wxStaticText(panel, wxID_ANY, this->profile_path);
    auto enter_password =
        new wxStaticText(panel, wxID_ANY, Strings::UnlockProfilePanel::enter_password());
    this->password_entry_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_PASSWORD | wxTE_PROCESS_ENTER
    );
    this->password_entry_textbox->Bind(wxEVT_TEXT_ENTER, [this](const wxCommandEvent&) {
        this->on_try_unlock_profile();
    });
    auto close_button = new wxButton(panel, wxID_CLOSE);
    close_button->Bind(wxEVT_BUTTON, [this](const wxCommandEvent&) { this->on_close(); });
    auto unlock_button =
        new wxButton(panel, wxID_ANY, Strings::UnlockProfilePanel::unlock_button());
    unlock_button->Bind(wxEVT_BUTTON, [this](const wxCommandEvent&) {
        this->on_try_unlock_profile();
    });
    auto button_sizer = new wxBoxSizer(wxHORIZONTAL);
    button_sizer->AddStretchSpacer();
    button_sizer->Add(close_button, 0, wxRIGHT, Metrics::PADDING_SMALL);
    button_sizer->Add(unlock_button, 0);

    // Layout
    v_sizer->Add(title, 0, wxALIGN_CENTER | wxALL, Metrics::PADDING_MEDIUM);
    v_sizer->Add(profile_path_heading, 0, wxALIGN_CENTER | wxBOTTOM, Metrics::PADDING_XLARGE);
    v_sizer->Add(enter_password, 0, wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(this->password_entry_textbox, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(button_sizer, 0, wxEXPAND | wxBOTTOM);

    panel->SetSizerAndFit(v_sizer);
    return panel;
}

void UnlockProfilePanel::on_try_unlock_profile() {
    const auto password = this->password_entry_textbox->GetValue();

    std::unique_ptr<tego_profile> profile;
    if (auto unlocked = tego_profile_try_open_existing(
            tego::out(profile),
            into_tego_string(this->profile_path).get(),
            into_tego_string(password).get(),
            tego::panic_on_error()
        );
        unlocked == TEGO_TRUE) {
        auto evt = ProfileUnlockedEvent(std::move(profile));
        evt.SetEventObject(this);
        this->GetEventHandler()->ProcessEvent(evt);

        LOG_INFO(fmt::format("profile {} unlocked!", this->profile_path));
    } else {
        // wrong password UXs
        LOG_INFO(fmt::format("profile {} wrong password!", this->profile_path));
    }
}

void UnlockProfilePanel::on_close() {
    wxGetApp().get_main_frame().get_sessions_notebook_panel_mut().close_session(
        dynamic_cast<SessionPanel*>(this->GetParent())
    );
}
