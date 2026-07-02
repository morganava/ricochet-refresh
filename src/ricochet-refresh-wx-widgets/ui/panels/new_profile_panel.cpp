#include "new_profile_panel.hpp"

#include "enums.hpp"
#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"

NewProfilePanel::NewProfilePanel(wxWindow* parent, NewProfile new_profile) :
    wxPanel(parent),
    current_step(ProfileCreationStep::SetPaths) {
    auto root_sizer = new wxBoxSizer(wxVERTICAL);
    auto h_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto title = new wxStaticText(this, wxID_ANY, Strings::NewProfilePanel::title(new_profile));
    title->SetFont(Fonts::title_font());

    this->paths_panel = [&, this]() -> wxPanel* {
        switch (new_profile) {
            case NewProfile::Generate:
                return this->create_generate_paths_panel();
            case NewProfile::ImportLegacy:
                return this->create_import_legacy_paths_panel();
            default:
                return nullptr;
        }
    }();
    this->credentials_panel = this->create_credentials_panel();
    this->overview_panel = this->create_overview_panel();

    auto button_sizer = new wxStdDialogButtonSizer();

    this->cancel_button = new wxButton(this, wxID_CANCEL);
    this->cancel_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->on_cancel(); });

    this->back_button = new wxButton(this, wxID_ANY, Strings::NewProfilePanel::back_button());
    this->back_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->on_back(); });

    this->next_button = new wxButton(this, wxID_ANY, Strings::NewProfilePanel::next_button());
    this->next_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) {
        if (this->current_step == ProfileCreationStep::Review) {
            this->on_finish();
        } else {
            this->on_next();
        }
    });

    button_sizer->SetCancelButton(this->cancel_button);
    button_sizer->SetAffirmativeButton(this->next_button);
    button_sizer->SetNegativeButton(this->back_button);
    button_sizer->Realize();

    // Layout

    v_sizer->Add(title, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(this->paths_panel, 1, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(this->credentials_panel, 1, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(this->overview_panel, 1, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    h_sizer->Add(v_sizer, 1, wxEXPAND | wxLEFT | wxRIGHT, Metrics::PADDING_MEDIUM);

    root_sizer->Add(h_sizer, 1, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    root_sizer->Add(button_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    this->SetSizerAndFit(root_sizer);

    this->reset();
}

wxPanel* NewProfilePanel::create_generate_paths_panel() {
    auto panel = new wxPanel(this);

    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto progress_heading = new wxStaticText(
        panel,
        wxID_ANY,
        Strings::NewProfilePanel::progress_heading(ProfileCreationStep::SetPaths)
    );
    progress_heading->SetFont(Fonts::heading_font());

    auto profile_destination_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::profile_destination());
    auto profile_destination_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto profile_destination_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_READONLY
    );
    auto profile_destination_browse_button =
        new wxButton(panel, wxID_ANY, Strings::NewProfilePanel::browse_button());

    profile_destination_sizer->Add(
        profile_destination_textbox,
        1,
        wxEXPAND | wxRIGHT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
    profile_destination_sizer->Add(profile_destination_browse_button, 0, wxEXPAND);

    // Layout

    v_sizer->Add(progress_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(profile_destination_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(profile_destination_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);

    panel->SetSizerAndFit(v_sizer);

    return panel;
}

wxPanel* NewProfilePanel::create_import_legacy_paths_panel() {
    auto panel = new wxPanel(this);

    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto progress_heading = new wxStaticText(
        panel,
        wxID_ANY,
        Strings::NewProfilePanel::progress_heading(ProfileCreationStep::SetPaths)
    );
    progress_heading->SetFont(Fonts::heading_font());

    auto legacy_profile_destination_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::legacy_profile_destination());
    auto legacy_profile_destination_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto legacy_profile_destination_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_READONLY
    );
    auto legacy_profile_destination_browse_button =
        new wxButton(panel, wxID_ANY, Strings::NewProfilePanel::browse_button());

    legacy_profile_destination_sizer->Add(
        legacy_profile_destination_textbox,
        1,
        wxEXPAND | wxRIGHT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
    legacy_profile_destination_sizer->Add(legacy_profile_destination_browse_button, 0, wxEXPAND);

    auto profile_destination_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::profile_destination());
    auto profile_destination_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto profile_destination_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_READONLY
    );
    auto profile_destination_browse_button =
        new wxButton(panel, wxID_ANY, Strings::NewProfilePanel::browse_button());

    profile_destination_sizer->Add(
        profile_destination_textbox,
        1,
        wxEXPAND | wxRIGHT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
    profile_destination_sizer->Add(profile_destination_browse_button, 0, wxEXPAND);

    // Layout

    v_sizer->Add(progress_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer
        ->Add(legacy_profile_destination_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(legacy_profile_destination_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(profile_destination_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(profile_destination_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);

    panel->SetSizerAndFit(v_sizer);

    return panel;
}

wxPanel* NewProfilePanel::create_credentials_panel() {
    auto panel = new wxPanel(this);

    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto progress_heading = new wxStaticText(
        panel,
        wxID_ANY,
        Strings::NewProfilePanel::progress_heading(ProfileCreationStep::CreateCredentials)
    );
    progress_heading->SetFont(Fonts::heading_font());

    auto display_name_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::display_name());
    auto display_name_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto display_name_textbox = new wxTextCtrl(panel, wxID_ANY);
    display_name_sizer->Add(display_name_textbox, 1, wxEXPAND);

    auto password_heading = new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::password());
    auto password_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto password_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_PASSWORD
    );
    password_sizer->Add(password_textbox, 1, wxEXPAND);

    auto confirm_password_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::confirm_password());
    auto confirm_password_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto confirm_password_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_PASSWORD
    );
    confirm_password_sizer->Add(confirm_password_textbox, 1, wxEXPAND);

    v_sizer->Add(progress_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(display_name_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(display_name_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(password_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(password_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(confirm_password_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(confirm_password_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);

    panel->SetSizerAndFit(v_sizer);

    return panel;
}

wxPanel* NewProfilePanel::create_overview_panel() {
    auto panel = new wxPanel(this);

    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto progress_heading = new wxStaticText(
        panel,
        wxID_ANY,
        Strings::NewProfilePanel::progress_heading(ProfileCreationStep::Review)
    );
    progress_heading->SetFont(Fonts::heading_font());

    auto profile_destination_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::profile_destination());
    auto profile_destination_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto profile_destination_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_READONLY
    );
    profile_destination_sizer->Add(profile_destination_textbox, 1);

    auto ricochet_id_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::ricochet_id());
    auto ricochet_id_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto ricochet_id_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_READONLY
    );
    auto ricochet_id_copy_button_bitmap = wxArtProvider::GetBitmap(wxART_COPY, wxART_BUTTON);
    auto ricochet_id_copy_button =
        new wxBitmapButton(panel, wxID_COPY, ricochet_id_copy_button_bitmap);

    ricochet_id_sizer
        ->Add(ricochet_id_textbox, 1, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
    ricochet_id_sizer->Add(ricochet_id_copy_button, 0, wxEXPAND);

    v_sizer->Add(progress_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(profile_destination_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(profile_destination_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(ricochet_id_heading, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(ricochet_id_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::PADDING_MEDIUM);

    panel->SetSizerAndFit(v_sizer);

    return panel;
}

//
// Event Handlers
//

void NewProfilePanel::on_cancel() {
    this->reset();
    wxGetApp().get_main_frame().hide_overlay_panel();
}

void NewProfilePanel::on_back() {
    switch (this->current_step) {
        case ProfileCreationStep::Review:
            this->credentials_panel->Show();
            this->overview_panel->Hide();
            this->next_button->SetLabel(Strings::NewProfilePanel::next_button());
            this->current_step = ProfileCreationStep::CreateCredentials;
            break;
        case ProfileCreationStep::CreateCredentials:
            this->paths_panel->Show();
            this->credentials_panel->Hide();
            this->current_step = ProfileCreationStep::SetPaths;
            this->back_button->Enable(false);
            break;
        default:
            tego::panic("unexpected step");
    }
    this->Layout();
}

void NewProfilePanel::on_next() {
    switch (this->current_step) {
        case ProfileCreationStep::SetPaths:
            this->paths_panel->Hide();
            this->credentials_panel->Show();
            this->back_button->Enable(true);
            this->current_step = ProfileCreationStep::CreateCredentials;
            break;
        case ProfileCreationStep::CreateCredentials:
            this->credentials_panel->Hide();
            this->overview_panel->Show();
            this->next_button->SetLabel(Strings::NewProfilePanel::finish_button());
            this->current_step = ProfileCreationStep::Review;
            break;
        default:
            tego::panic("unexpected step");
    }

    this->Layout();
}

void NewProfilePanel::on_finish() {
    LOG_INFO("finish!");
}

// reset widgets back to default states
void NewProfilePanel::reset() {
    this->current_step = ProfileCreationStep::SetPaths;

    this->paths_panel->Show();
    this->credentials_panel->Hide();
    this->overview_panel->Hide();

    this->back_button->Enable(false);
    this->next_button->SetLabel(Strings::NewProfilePanel::next_button());

    this->Layout();
}
