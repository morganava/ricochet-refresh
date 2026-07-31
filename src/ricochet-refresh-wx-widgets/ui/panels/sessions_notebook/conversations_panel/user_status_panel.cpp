#include "user_status_panel.hpp"

#include "enums.hpp"
#include "strings.hpp"
#include "ui/bitmaps.hpp"
#include "ui/metrics.hpp"

UserStatusPanel::UserStatusPanel(wxWindow* parent) : wxPanel(parent) {
    auto h_sizer = new wxBoxSizer(wxHORIZONTAL);

    wxString visibility_options[static_cast<size_t>(Visibility::Count)] = {
        Strings::Enums::Visibility::visible(),
        Strings::Enums::Visibility::restricted(),
        Strings::Enums::Visibility::hidden(),
        Strings::Enums::Visibility::offline(),
    };

    // todo: would neeed a custom widget to render as we envisioned using wxOwnerDrawnComboBox
    auto visibility_choice = new wxChoice(
        this,
        wxID_ANY,
        wxDefaultPosition,
        wxDefaultSize,
        static_cast<int>(Visibility::Count),
        visibility_options
    );
    // todo: load default visibility from profile
    visibility_choice->SetSelection(static_cast<int>(Visibility::Visible));
    visibility_choice->Bind(wxEVT_CHOICE, [this](const wxCommandEvent& event) {
        this->set_visibility(static_cast<Visibility>(event.GetInt()));
    });

    auto profile_button = new wxBitmapButton(this, wxID_ANY, Bitmaps::default_avatar());
    profile_button->Bind(wxEVT_BUTTON, [this](const wxCommandEvent&) {
        this->on_profile_button_clicked();
    });

    // Layout
    h_sizer->Add(visibility_choice, 1, wxEXPAND);
    h_sizer->Add(profile_button, 0, wxEXPAND);

    this->SetSizerAndFit(h_sizer);
}

void UserStatusPanel::on_profile_button_clicked() {
    LOG_INFO("Profile Button Pressed");
}

void UserStatusPanel::set_visibility(Visibility visibility) {
    LOG_INFO(fmt::format("Set Visibility: {}", Strings::Enums::Visibility::to_string(visibility)));
}
