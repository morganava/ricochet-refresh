#include "new_profile_panel.hpp"

#include "enums.hpp"
#include "ffi.hpp"
#include "paths.hpp"
#include "strings.hpp"
#include "ui/error_popup.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"
#include "ui/palette.hpp"

NewProfilePanel::NewProfilePanel(wxWindow* parent, NewProfile new_profile) :
    wxPanel(parent),
    new_profile_type(new_profile),
    current_step(ProfileCreationStep::SetPaths) {
    auto root_sizer = new wxBoxSizer(wxVERTICAL);
    auto h_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto title = new wxStaticText(this, wxID_ANY, Strings::NewProfilePanel::title(new_profile));
    title->SetFont(Fonts::title_font());

    this->paths_panel = [&, this]() -> wxPanel* {
        switch (this->new_profile_type) {
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
    this->paths.profile_destination_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_READONLY
    );
    auto profile_destination_browse_button =
        new wxButton(panel, wxID_ANY, Strings::NewProfilePanel::browse_button());
    profile_destination_browse_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) {
        this->on_get_new_profile_destination();
    });

    profile_destination_sizer->Add(
        this->paths.profile_destination_textbox,
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
    this->paths.legacy_profile_destination_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_READONLY
    );
    auto legacy_profile_destination_browse_button =
        new wxButton(panel, wxID_ANY, Strings::NewProfilePanel::browse_button());
    legacy_profile_destination_browse_button->Bind(wxEVT_BUTTON, [this](const wxCommandEvent&) {
        this->on_get_legacy_profile_destination();
    });

    legacy_profile_destination_sizer->Add(
        this->paths.legacy_profile_destination_textbox,
        1,
        wxEXPAND | wxRIGHT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
    legacy_profile_destination_sizer->Add(legacy_profile_destination_browse_button, 0, wxEXPAND);

    auto profile_destination_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::profile_destination());
    auto profile_destination_sizer = new wxBoxSizer(wxHORIZONTAL);
    this->paths.profile_destination_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_READONLY
    );
    auto profile_destination_browse_button =
        new wxButton(panel, wxID_ANY, Strings::NewProfilePanel::browse_button());
    profile_destination_browse_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) {
        this->on_get_new_profile_destination();
    });

    profile_destination_sizer->Add(
        this->paths.profile_destination_textbox,
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
    this->credentials.display_name_textbox = new wxTextCtrl(panel, wxID_ANY);
    this->credentials.display_name_textbox->Bind(wxEVT_TEXT, [this](const wxCommandEvent&) {
        this->on_display_name_changed();
    });
    display_name_sizer->Add(this->credentials.display_name_textbox, 1, wxEXPAND);

    auto password_heading = new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::password());
    auto password_sizer = new wxBoxSizer(wxHORIZONTAL);
    this->credentials.password_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_PASSWORD
    );
    this->credentials.password_textbox->Bind(wxEVT_TEXT, [this](const wxCommandEvent&) {
        this->on_password_changed();
    });
    password_sizer->Add(this->credentials.password_textbox, 1, wxEXPAND);

    auto confirm_password_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::confirm_password());
    auto confirm_password_sizer = new wxBoxSizer(wxHORIZONTAL);
    this->credentials.confirm_password_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_PASSWORD
    );
    this->credentials.confirm_password_textbox->Bind(wxEVT_TEXT, [this](const wxCommandEvent&) {
        this->on_confirmed_password_changed();
    });
    confirm_password_sizer->Add(this->credentials.confirm_password_textbox, 1, wxEXPAND);

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
    this->overview.profile_destination_textbox = new wxTextCtrl(
        panel,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_READONLY
    );
    profile_destination_sizer->Add(this->overview.profile_destination_textbox, 1);

    auto ricochet_id_heading =
        new wxStaticText(panel, wxID_ANY, Strings::NewProfilePanel::ricochet_id());
    auto ricochet_id_sizer = new wxBoxSizer(wxHORIZONTAL);
    this->overview.ricochet_id_textbox = new wxTextCtrl(
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
    ricochet_id_copy_button->Bind(wxEVT_BUTTON, [this](const wxCommandEvent&) {
        this->on_copy_ricochet_id();
    });

    ricochet_id_sizer->Add(
        this->overview.ricochet_id_textbox,
        1,
        wxEXPAND | wxRIGHT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
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
            this->current_step = ProfileCreationStep::CreateCredentials;
            this->credentials_panel->Show();
            this->overview_panel->Hide();
            this->back_button->Enable(true);
            this->update_next_button();
            break;
        case ProfileCreationStep::CreateCredentials:
            this->current_step = ProfileCreationStep::SetPaths;
            this->paths_panel->Show();
            this->credentials_panel->Hide();
            this->back_button->Enable(false);
            this->update_next_button();
            break;
        default:
            tego::panic("unexpected step");
    }
    this->Layout();
}

void NewProfilePanel::on_next() {
    switch (this->current_step) {
        case ProfileCreationStep::SetPaths:
            this->current_step = ProfileCreationStep::CreateCredentials;
            this->paths_panel->Hide();
            this->credentials_panel->Show();
            this->back_button->Enable(true);
            this->update_next_button();
            break;
        case ProfileCreationStep::CreateCredentials:
            this->current_step = ProfileCreationStep::Review;
            this->credentials_panel->Hide();
            this->overview_panel->Show();
            this->next_button->SetLabel(Strings::NewProfilePanel::finish_button());
            this->back_button->Enable(true);
            this->update_next_button();
            break;
        default:
            tego::panic("unexpected step");
    }

    this->Layout();
}

void NewProfilePanel::on_finish() {
    std::unique_ptr<tego_profile> profile;
    const auto profile_path = into_tego_string(this->paths.profile_destination_textbox->GetValue());
    const auto nickname = into_tego_string(this->credentials.display_name_textbox->GetValue());
    const auto password = into_tego_string(this->credentials.password_textbox->GetValue());
    std::unique_ptr<tego_error> error;

    switch (this->new_profile_type) {
        case NewProfile::Generate: {
            const auto& identity_private_key = this->paths.identity_private_key;
            tego_profile_store_generate_new(
                tego::out(profile),
                profile_path.get(),
                identity_private_key.get(),
                nickname.get(),
                password.get(),
                tego::out(error)
            );
        } break;
        case NewProfile::ImportLegacy: {
            const auto legacy_profile_path =
                into_tego_string(this->paths.legacy_profile_destination_textbox->GetValue());
            tego_profile_import_legacy(
                tego::out(profile),
                profile_path.get(),
                legacy_profile_path.get(),
                nickname.get(),
                password.get(),
                tego::out(error)
            );

        } break;
        default:
            tego::panic("unexpected new profile type");
    }

    if (error) {
        ErrorPopup::show(
            this,
            Strings::NewProfilePanel::error_creating_profile(),
            tego_error_get_message(error.get())
        );
    } else {
        this->reset();
        wxGetApp().get_main_frame().hide_overlay_panel();
    }
}

void NewProfilePanel::on_copy_ricochet_id() {
    auto clipboard = wxTheClipboard;

    if (clipboard->Open()) {
        clipboard->SetData(new wxTextDataObject(this->overview.ricochet_id_textbox->GetValue()));
        clipboard->Flush();
        clipboard->Close();
    } else {
        LOG_ERROR("Could not open clipboard");
    }
}

void NewProfilePanel::on_get_new_profile_destination() {
    // Create and show the dialog
    wxFileDialog save_profile_dialog(
        this,
        Strings::NewProfilePanel::save_profile_file_dialog_title(),
        Paths::home().GetAbsolutePath(),
        "alice.rr-profile", // default filename
        Strings::NewProfilePanel::profile_file_dialog_wildcard(),
        wxFD_SAVE | wxFD_OVERWRITE_PROMPT
    );

    if (save_profile_dialog.ShowModal() == wxID_OK) {
        auto new_profile_path = save_profile_dialog.GetPath();
        if (!new_profile_path.EndsWith(Strings::Common::profile_file_extension())) {
            new_profile_path.append(Strings::Common::profile_file_extension());
        }
        this->paths.profile_destination_textbox->SetValue(new_profile_path);
        this->overview.profile_destination_textbox->SetValue(new_profile_path);
        if (this->new_profile_type == NewProfile::Generate) {
            // generate a new key for new profile
            tego_ed25519_private_key_generate(
                tego::out(this->paths.identity_private_key),
                tego::panic_on_error()
            );
            this->on_identity_private_key_changed();
        }
        this->update_next_button();
    }
}

void NewProfilePanel::on_get_legacy_profile_destination() {
    wxFileDialog open_legacy_profile_dialog(
        this,
        Strings::NewProfilePanel::open_legacy_profile_file_dialog_title(),
        Paths::config_directory().GetAbsolutePath(),
        "ricochet.json",
        Strings::NewProfilePanel::legacy_profile_dialog_wildcard(),
        wxFD_OPEN | wxFD_FILE_MUST_EXIST | wxFD_SHOW_HIDDEN
    );

    if (open_legacy_profile_dialog.ShowModal() == wxID_OK) {
        auto legacy_profile_path = open_legacy_profile_dialog.GetPath();
        std::unique_ptr<tego_ed25519_private_key> private_key;
        std::unique_ptr<tego_error> error;
        tego_ed25519_private_key_from_legacy_profile(
            tego::out(private_key),
            into_tego_string(legacy_profile_path).get(),
            tego::out(error)
        );
        if (error) {
            ErrorPopup::show(
                this,
                Strings::NewProfilePanel::error_loading_legacy_profile(),
                tego_error_get_message(error.get())
            );
        } else {
            this->paths.legacy_profile_destination_textbox->SetValue(legacy_profile_path);

            this->paths.identity_private_key.reset(private_key.release());
            this->on_identity_private_key_changed();
        }
    }
}

void NewProfilePanel::on_identity_private_key_changed() {
    std::unique_ptr<tego_v3_onion_service_id> service_id;
    tego_v3_onion_service_id_from_ed25519_private_key(
        tego::out(service_id),
        this->paths.identity_private_key.get(),
        tego::panic_on_error()
    );

    const auto ricochet_id = Strings::Common::ricochet_v4_id_uri(service_id.get());
    this->overview.ricochet_id_textbox->ChangeValue(ricochet_id);
}

void NewProfilePanel::on_display_name_changed() {
    this->update_next_button();
}

void NewProfilePanel::on_password_changed() {
    this->update_confirm_password_background();
    this->update_next_button();
}

void NewProfilePanel::on_confirmed_password_changed() {
    this->update_confirm_password_background();
    this->update_next_button();
}

void NewProfilePanel::update_next_button() {
    switch (this->current_step) {
        case ProfileCreationStep::SetPaths: {
            this->next_button->SetLabel(Strings::NewProfilePanel::next_button());
            auto enable_next = !this->paths.profile_destination_textbox->GetValue().IsEmpty();
            if (this->new_profile_type == NewProfile::ImportLegacy) {
                enable_next = enable_next
                    && !this->paths.legacy_profile_destination_textbox->GetValue().IsEmpty();
            }

            this->next_button->Enable(enable_next);

        } break;
        case ProfileCreationStep::CreateCredentials: {
            const auto display_name = this->credentials.display_name_textbox->GetValue();
            const auto password = this->credentials.password_textbox->GetValue();
            const auto confirmed_password = this->credentials.confirm_password_textbox->GetValue();

            if (!display_name.IsEmpty() && !password.IsEmpty() && password == confirmed_password) {
                this->next_button->Enable(true);
            } else {
                this->next_button->Enable(false);
            }
            this->next_button->SetLabel(Strings::NewProfilePanel::next_button());
        } break;
        case ProfileCreationStep::Review: {
            this->next_button->SetLabel(Strings::NewProfilePanel::finish_button());
            this->next_button->Enable(true);
        } break;
    }
}

void NewProfilePanel::update_confirm_password_background() {
    const auto& palette = Palette::instance();

    const auto password = this->credentials.password_textbox->GetValue();
    const auto confirmed_password = this->credentials.confirm_password_textbox->GetValue();

    if (wxString password_tail; password.StartsWith(confirmed_password, &password_tail)) {
        if (password_tail.IsEmpty()) {
            // password matches
            this->credentials.confirm_password_textbox->SetBackgroundColour(wxNullColour);
        } else {
            // password starts with the confirmed password
            this->credentials.confirm_password_textbox->SetBackgroundColour(palette.yellow);
        }
    } else {
        // passwords do no tmatch at all
        this->credentials.confirm_password_textbox->SetBackgroundColour(palette.red);
    }
}

// reset widgets back to default states
void NewProfilePanel::reset() {
    this->current_step = ProfileCreationStep::SetPaths;

    // Paths panel
    if (this->new_profile_type == NewProfile::ImportLegacy) {
        this->paths.legacy_profile_destination_textbox->ChangeValue(wxEmptyString);
    }

    this->paths.profile_destination_textbox->ChangeValue(wxEmptyString);
    this->paths_panel->Show();

    // Credentials Panel
    this->credentials.display_name_textbox->ChangeValue(wxEmptyString);
    this->credentials.password_textbox->ChangeValue(wxEmptyString);
    this->credentials.confirm_password_textbox->ChangeValue(wxEmptyString);
    this->credentials.confirm_password_textbox->SetBackgroundColour(wxNullColour);
    this->credentials_panel->Hide();

    // Overview Panel
    this->overview.profile_destination_textbox->ChangeValue(wxEmptyString);
    this->overview.ricochet_id_textbox->ChangeValue(wxEmptyString);
    this->overview_panel->Hide();

    // Navigation Butons
    this->back_button->Enable(false);
    this->next_button->Enable(false);
    this->next_button->SetLabel(Strings::NewProfilePanel::next_button());

    this->Layout();
}
