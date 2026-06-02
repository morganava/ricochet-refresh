#include "interface_settings_panel.hpp"

#include "enums.hpp"
#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/metrics.hpp"

InterfaceSettingsPanel::InterfaceSettingsPanel(wxWindow* parent) :
    wxScrolled<wxPanel>(parent, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxVSCROLL) {
    this->SetScrollRate(0, this->FromDIP(Metrics::VSCROLL_RATE));

    auto h_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    // Language

    auto language_heading =
        new wxStaticText(this, wxID_ANY, Strings::InterfaceSettingsPanel::language_heading());
    language_heading->SetFont(Fonts::heading_font());

    auto language_h_sizer = new wxBoxSizer(wxHORIZONTAL);

    auto select_interface_language_label = new wxStaticText(
        this,
        wxID_ANY,
        Strings::InterfaceSettingsPanel::select_interface_language_label()
    );
    this->language_combobox = new wxComboBox(
        this,
        wxID_ANY,
        Strings::Enums::Language::system(),
        wxDefaultPosition,
        wxDefaultSize,
        Strings::InterfaceSettingsPanel::supported_languages(),
        wxCB_READONLY
    );
    this->language_combobox->Bind(wxEVT_COMBOBOX, [this](wxCommandEvent& evt) {
        this->set_interface_language(static_cast<Language>(evt.GetInt()));
    });

    language_h_sizer->Add(select_interface_language_label, 0, wxALIGN_CENTER_VERTICAL, 0);
    language_h_sizer->AddStretchSpacer(1);
    language_h_sizer->Add(this->language_combobox, 0, wxALIGN_CENTER_VERTICAL, 0);

    // Toolbars

    auto toolbars_heading =
        new wxStaticText(this, wxID_ANY, Strings::InterfaceSettingsPanel::toolbars_heading());
    toolbars_heading->SetFont(Fonts::heading_font());

    this->show_toolbar_toggle =
        new wxCheckBox(this, wxID_ANY, Strings::InterfaceSettingsPanel::show_toolbar_toggle());

    this->show_toolbar_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_show_toolbar(evt.IsChecked());
    });

    auto button_style_h_sizer = new wxBoxSizer(wxHORIZONTAL);

    this->button_style_label =
        new wxStaticText(this, wxID_ANY, Strings::InterfaceSettingsPanel::button_style_label());

    this->button_style_combobox = new wxComboBox(
        this,
        wxID_ANY,
        Strings::InterfaceSettingsPanel::button_style_icons(),
        wxDefaultPosition,
        wxDefaultSize,
        Strings::InterfaceSettingsPanel::button_styles(),
        wxCB_READONLY
    );
    this->button_style_combobox->Bind(wxEVT_COMBOBOX, [this](wxCommandEvent& evt) {
        this->set_button_style(static_cast<ButtonStyle>(evt.GetInt()));
    });

    button_style_h_sizer->Add(this->button_style_label, 0, wxALIGN_CENTER_VERTICAL, 0);
    button_style_h_sizer->AddStretchSpacer(1);
    button_style_h_sizer->Add(this->button_style_combobox, 0, wxALIGN_CENTER_VERTICAL, 0);

    // Alerts

    auto alerts_heading =
        new wxStaticText(this, wxID_ANY, Strings::InterfaceSettingsPanel::alerts_heading());
    alerts_heading->SetFont(Fonts::heading_font());

    this->show_desktop_notifications_toggle = new wxCheckBox(
        this,
        wxID_ANY,
        Strings::InterfaceSettingsPanel::show_desktop_notifications_toggle()
    );
    this->show_desktop_notifications_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_show_desktop_notifications(evt.IsChecked());
    });

    this->blink_taskbar_icon_toggle = new wxCheckBox(
        this,
        wxID_ANY,
        Strings::InterfaceSettingsPanel::blink_taskbar_icon_toggle()
    );
    this->blink_taskbar_icon_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_blink_taskbar_icon(evt.IsChecked());
    });

    this->enable_audio_notifications_toggle = new wxCheckBox(
        this,
        wxID_ANY,
        Strings::InterfaceSettingsPanel::enable_audio_notifications_toggle()
    );
    this->enable_audio_notifications_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_enable_audio_notifications(evt.IsChecked());
    });

    // Window

    auto window_heading =
        new wxStaticText(this, wxID_ANY, Strings::InterfaceSettingsPanel::window_heading());
    window_heading->SetFont(Fonts::heading_font());

    this->minimize_instead_of_exit_toggle = new wxCheckBox(
        this,
        wxID_ANY,
        Strings::InterfaceSettingsPanel::minimize_instead_of_exit_toggle()
    );
    this->minimize_instead_of_exit_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_minimize_instead_of_exit(evt.IsChecked());
    });

    this->show_system_tray_icon_toggle = new wxCheckBox(
        this,
        wxID_ANY,
        Strings::InterfaceSettingsPanel::show_system_tray_icon_toggle()
    );
    this->show_system_tray_icon_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_show_system_tray_icon(evt.IsChecked());
    });

    auto minimize_to_system_tray_v_sizer = new wxBoxSizer(wxVERTICAL);

    this->minimize_to_system_tray_toggle = new wxCheckBox(
        this,
        wxID_ANY,
        Strings::InterfaceSettingsPanel::minimize_to_system_tray_toggle()
    );
    this->minimize_to_system_tray_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_minimize_to_system_tray(evt.IsChecked());
    });

    minimize_to_system_tray_v_sizer
        ->Add(this->minimize_to_system_tray_toggle, 0, wxLEFT, Metrics::HORIZONTAL_PADDING_XLARGE);

    // Layout

    v_sizer->Add(language_heading, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(language_h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    v_sizer->Add(toolbars_heading, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(this->show_toolbar_toggle, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(button_style_h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    v_sizer->Add(alerts_heading, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(
        this->show_desktop_notifications_toggle,
        0,
        wxBOTTOM,
        Metrics::VERTICAL_PADDING_MEDIUM
    );
    v_sizer->Add(this->blink_taskbar_icon_toggle, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(
        this->enable_audio_notifications_toggle,
        0,
        wxBOTTOM,
        Metrics::VERTICAL_PADDING_MEDIUM
    );

    v_sizer->Add(window_heading, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer
        ->Add(this->minimize_instead_of_exit_toggle, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(show_system_tray_icon_toggle, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(minimize_to_system_tray_v_sizer, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    h_sizer->Add(v_sizer, 1, wxEXPAND | wxLEFT | wxRIGHT, Metrics::HORIZONTAL_PADDING_MEDIUM);
    this->SetSizerAndFit(h_sizer);

    // todo: configure UX based on loaded settings

    this->disable_button_style_controls();
    this->disable_system_tray_icon_controls();

    this->load_from_settings();
}

void InterfaceSettingsPanel::load_from_settings() {
    const auto& settings = wxGetApp().get_settings();

    // interface language
    tego_language language = {};
    tego_settings_get_language(&settings, &language, tego::panic_on_error());
    this->language_combobox->SetSelection(static_cast<int>(language));

    // show toolbar
    tego_bool show_toolbar = TEGO_FALSE;
    tego_settings_get_show_toolbar(&settings, &show_toolbar, tego::panic_on_error());
    this->show_toolbar_toggle->SetValue(show_toolbar);

    // todo: button style
    tego_button_style button_style = {};
    tego_settings_get_button_style(&settings, &button_style, tego::panic_on_error());
    this->button_style_combobox->SetSelection(static_cast<int>(button_style));

    if (show_toolbar) {
        this->enable_button_style_controls();
    } else {
        this->disable_button_style_controls();
    }

    // desktop notifications
    tego_bool show_desktop_notifications = TEGO_FALSE;
    tego_settings_get_show_desktop_notifications(
        &settings,
        &show_desktop_notifications,
        tego::panic_on_error()
    );
    this->show_desktop_notifications_toggle->SetValue(show_desktop_notifications);

    // blink taskbar icon
    tego_bool blink_taskbar_icon = TEGO_FALSE;
    tego_settings_get_blink_taskbar_icon(&settings, &blink_taskbar_icon, tego::panic_on_error());
    this->blink_taskbar_icon_toggle->SetValue(blink_taskbar_icon);

    // enable audio notifications
    tego_bool enable_audio_notifications = TEGO_FALSE;
    tego_settings_get_play_audio_notifications(
        &settings,
        &enable_audio_notifications,
        tego::panic_on_error()
    );
    this->enable_audio_notifications_toggle->SetValue(enable_audio_notifications);

    // minimize instead of exit
    tego_bool minimize_instead_of_exit = TEGO_FALSE;
    tego_settings_get_minimize_instead_of_exit(
        &settings,
        &minimize_instead_of_exit,
        tego::panic_on_error()
    );
    this->minimize_instead_of_exit_toggle->SetValue(minimize_instead_of_exit);

    // show system tray icon
    tego_bool show_system_tray_icon = TEGO_FALSE;
    tego_settings_get_show_system_tray_icon(
        &settings,
        &show_system_tray_icon,
        tego::panic_on_error()
    );
    this->show_system_tray_icon_toggle->SetValue(show_system_tray_icon);

    // minimize to system tray
    tego_bool minimize_to_system_tray = TEGO_FALSE;
    tego_settings_get_minimize_to_system_tray(
        &settings,
        &minimize_to_system_tray,
        tego::panic_on_error()
    );
    this->minimize_to_system_tray_toggle->SetValue(minimize_to_system_tray);
}

void InterfaceSettingsPanel::save_to_settings() {
    auto& settings = wxGetApp().get_settings_mut();

    // interface language
    const auto language = static_cast<tego_language>(this->language_combobox->GetSelection());
    tego_settings_set_language(&settings, language, tego::panic_on_error());

    // show toolbar
    const auto show_toolbar = this->show_toolbar_toggle->GetValue();
    tego_settings_set_show_toolbar(
        &settings,
        show_toolbar ? TEGO_TRUE : TEGO_FALSE,
        tego::panic_on_error()
    );

    // button style
    const auto button_style =
        static_cast<tego_button_style>(this->button_style_combobox->GetSelection());
    tego_settings_set_button_style(&settings, button_style, tego::panic_on_error());

    // desktop notifications
    const auto show_desktop_notifications = this->show_desktop_notifications_toggle->GetValue();
    tego_settings_set_show_desktop_notifications(
        &settings,
        show_desktop_notifications ? TEGO_TRUE : TEGO_FALSE,
        tego::panic_on_error()
    );

    // blink taskbar icon
    const auto blink_taskbar_icon = this->blink_taskbar_icon_toggle->GetValue();
    tego_settings_set_blink_taskbar_icon(
        &settings,
        blink_taskbar_icon ? TEGO_TRUE : TEGO_FALSE,
        tego::panic_on_error()
    );

    // enable audio notifications
    const auto enable_audio_notifications = this->enable_audio_notifications_toggle->GetValue();
    tego_settings_set_play_audio_notifications(
        &settings,
        enable_audio_notifications ? TEGO_TRUE : TEGO_FALSE,
        tego::panic_on_error()
    );

    // minimize instead of exit
    const auto minimize_instead_of_exit = this->minimize_instead_of_exit_toggle->GetValue();
    tego_settings_set_minimize_instead_of_exit(
        &settings,
        minimize_instead_of_exit ? TEGO_TRUE : TEGO_FALSE,
        tego::panic_on_error()
    );

    // show system tray icon
    const auto show_system_tray_icon = this->show_system_tray_icon_toggle->GetValue();
    tego_settings_set_show_system_tray_icon(
        &settings,
        show_system_tray_icon ? TEGO_TRUE : TEGO_FALSE,
        tego::panic_on_error()
    );

    // minimize to system tray
    const auto minimize_to_system_tray = this->minimize_to_system_tray_toggle->GetValue();
    tego_settings_set_minimize_to_system_tray(
        &settings,
        minimize_to_system_tray ? TEGO_TRUE : TEGO_FALSE,
        tego::panic_on_error()
    );
}

void InterfaceSettingsPanel::set_interface_language(Language language) {
    const auto language_str = [&]() {
        switch (language) {
            case Language::System:
                return Strings::Enums::Language::system();
            case Language::Arabic:
                return Strings::Enums::Language::ar();
            case Language::German:
                return Strings::Enums::Language::de();
            case Language::English:
                return Strings::Enums::Language::en();
            case Language::Spanish:
                return Strings::Enums::Language::es();
            case Language::Dutch:
                return Strings::Enums::Language::nl();
            default:
                return wxString("unknown");
        }
    }();
    LOG_INFO(fmt::format("Set Language: {}", language_str));
}

void InterfaceSettingsPanel::set_show_toolbar(bool enabled) {
    LOG_INFO(fmt::format("Set Show toolbar: {}", enabled));
    if (enabled) {
        this->enable_button_style_controls();
    } else {
        this->disable_button_style_controls();
    }
}

void InterfaceSettingsPanel::set_button_style(ButtonStyle button_style) {
    const auto button_style_str = [&]() {
        switch (button_style) {
            case ButtonStyle::Icons:
                return "Icons";
            case ButtonStyle::Text:
                return "Text";
            case ButtonStyle::IconsAndText:
                return "Icons and Text";
            case ButtonStyle::IconsBesideText:
                return "Icons beside Text";
            default:
                return "unknown";
        }
    }();
    LOG_INFO(fmt::format("Set Button Style: {}", button_style_str));
}

void InterfaceSettingsPanel::set_show_desktop_notifications(bool enabled) {
    LOG_INFO(fmt::format("Set Show desktop notifications: {}", enabled));
}

void InterfaceSettingsPanel::set_blink_taskbar_icon(bool enabled) {
    LOG_INFO(fmt::format("Set Blink taskbar icon: {}", enabled));
}

void InterfaceSettingsPanel::set_enable_audio_notifications(bool enabled) {
    LOG_INFO(fmt::format("Set Enable audio notifications: {}", enabled));
}

void InterfaceSettingsPanel::set_minimize_instead_of_exit(bool enabled) {
    LOG_INFO(fmt::format("Set Minimze instead of exit: {}", enabled));
}

void InterfaceSettingsPanel::set_show_system_tray_icon(bool enabled) {
    LOG_INFO(fmt::format("Set Show system tray icon: {}", enabled));
    if (enabled) {
        this->enable_system_tray_icon_controls();
    } else {
        this->disable_system_tray_icon_controls();
    }
}

void InterfaceSettingsPanel::set_minimize_to_system_tray(bool enabled) {
    LOG_INFO(fmt::format("Set Minimize to system tray: {}", enabled));
}

void InterfaceSettingsPanel::enable_button_style_controls() {
    this->button_style_label->Enable();
    this->button_style_combobox->Enable();
}

void InterfaceSettingsPanel::disable_button_style_controls() {
    this->button_style_label->Disable();
    this->button_style_combobox->Disable();
}

void InterfaceSettingsPanel::enable_system_tray_icon_controls() {
    this->minimize_to_system_tray_toggle->Enable();
}

void InterfaceSettingsPanel::disable_system_tray_icon_controls() {
    this->minimize_to_system_tray_toggle->Disable();
}
