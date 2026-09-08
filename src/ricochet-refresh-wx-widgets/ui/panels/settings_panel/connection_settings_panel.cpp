#include "connection_settings_panel.hpp"

#include "enums.hpp"
#include "ffi.hpp"
#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/metrics.hpp"
#include "ui/widgets/wrapped_static_text.hpp"

ConnectionSettingsPanel::ConnectionSettingsPanel(wxWindow* parent) :
    wxScrolled<wxPanel>(parent, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxVSCROLL) {
    this->SetScrollRate(0, this->FromDIP(Metrics::VSCROLL_RATE));

    auto h_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    // Quickstart

    auto quickstart_heading =
        new wxStaticText(this, wxID_ANY, Strings::ConnectionSettingsPanel::quickstart_heading());
    quickstart_heading->SetFont(Fonts::heading_font());

    auto quickstart_description = new WrappedStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::quickstart_description()
    );

    this->connect_automatically_toggle = new wxCheckBox(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::connect_automatically_toggle()
    );
    this->connect_automatically_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_connect_automatically(evt.IsChecked());
    });

    // Backend

    auto backend_heading =
        new wxStaticText(this, wxID_ANY, Strings::ConnectionSettingsPanel::backend_heading());
    backend_heading->SetFont(Fonts::heading_font());

    auto backend_description = new WrappedStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::backend_description()
    );

#ifdef ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
    this->bundled_legacy_tor_option = new wxRadioButton(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::bundled_legacy_tor_option(),
        wxDefaultPosition,
        wxDefaultSize,
        wxRB_GROUP
    );
    this->bundled_legacy_tor_option->Bind(wxEVT_RADIOBUTTON, [this](wxCommandEvent&) {
        this->set_tor_backend(TorBackend::BundledLegacyTor);
    });
#endif // ENABLE_RICOCHET_REFRESH_BUNDLED_TOR

#ifdef ENABLE_RICOCHET_REFRESH_EXTERNAL_TOR
    this->external_legacy_tor_option = new wxRadioButton(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::external_legacy_tor_option()
    );
    this->external_legacy_tor_option->Bind(wxEVT_RADIOBUTTON, [this](wxCommandEvent&) {
        this->set_tor_backend(TorBackend::ExternalLegacyTor);
    });
#endif // ENABLE_RICOCHET_REFRESH_EXTERNAL_TOR

#ifdef ENABLE_RICOCHET_REFRESH_ARTI_CLIENT
    this->in_process_arti_option = new wxRadioButton(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::in_process_arti_option()
    );
    this->in_process_arti_option->Bind(wxEVT_RADIOBUTTON, [this](wxCommandEvent&) {
        this->set_tor_backend(TorBackend::InProcessArti);
    });
#endif // ENABLE_RICOCHET_REFRESH_ARTI_CLIENT

#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    // Bridges

    auto bridges_heading =
        new wxStaticText(this, wxID_ANY, Strings::ConnectionSettingsPanel::bridges_heading());
    bridges_heading->SetFont(Fonts::heading_font());
    auto bridges_description = new WrappedStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::bridges_description()
    );
    this->use_bridges_toggle =
        new wxCheckBox(this, wxID_ANY, Strings::ConnectionSettingsPanel::use_bridges_toggle());
    this->use_bridges_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_use_bridges(evt.IsChecked());
    });

    this->builtin_bridge_option = new wxRadioButton(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::builtin_bridge_option(),
        wxDefaultPosition,
        wxDefaultSize,
        wxRB_GROUP
    );
    this->builtin_bridge_option->Bind(wxEVT_RADIOBUTTON, [this](wxCommandEvent&) {
        this->set_bridge_type(BridgeType::Builtin);
    });
    this->custom_bridge_option =
        new wxRadioButton(this, wxID_ANY, Strings::ConnectionSettingsPanel::custom_bridge_option());
    this->custom_bridge_option->Bind(wxEVT_RADIOBUTTON, [this](wxCommandEvent&) {
        this->set_bridge_type(BridgeType::Custom);
    });

    this->obfs4_bridge_option = new wxRadioButton(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::obfs4_bridge_option(),
        wxDefaultPosition,
        wxDefaultSize,
        wxRB_GROUP
    );
    this->obfs4_bridge_option->Bind(wxEVT_RADIOBUTTON, [this](wxCommandEvent&) {
        this->set_builtin_bridge(BuiltinBridge::Obfs4);
    });
    this->snowflake_bridge_option = new wxRadioButton(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::snowflake_bridge_option()
    );
    this->snowflake_bridge_option->Bind(wxEVT_RADIOBUTTON, [this](wxCommandEvent&) {
        this->set_builtin_bridge(BuiltinBridge::Snowflake);
    });
    this->meek_bridge_option =
        new wxRadioButton(this, wxID_ANY, Strings::ConnectionSettingsPanel::meek_bridge_option());
    this->meek_bridge_option->Bind(wxEVT_RADIOBUTTON, [this](wxCommandEvent&) {
        this->set_builtin_bridge(BuiltinBridge::Meek);
    });

    this->obfs4_bridge_description = new WrappedStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::obfs4_bridge_description()
    );
    this->snowflake_bridge_description = new WrappedStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::snowflake_bridge_description()
    );
    this->meek_bridge_description = new WrappedStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::meek_bridge_description()
    );

    this->custom_bridge_textbox = new wxTextCtrl(
        this,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_MULTILINE | wxHSCROLL
    );
    auto custom_bridge_textbox_line_height = Metrics::line_height(*this->custom_bridge_textbox);
    this->custom_bridge_textbox->SetMinSize(wxSize(-1, 4 * custom_bridge_textbox_line_height));
    this->custom_bridge_textbox->SetHint(
        Strings::ConnectionSettingsPanel::custom_bridge_textbox_hint()
    );
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS

    // Network settings

    auto network_settings_heading = new wxStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::network_settings_heading()
    );
    network_settings_heading->SetFont(Fonts::heading_font());

    auto network_settings_description = new WrappedStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::network_settings_description()
    );

    this->use_proxy_toggle =
        new wxCheckBox(this, wxID_ANY, Strings::ConnectionSettingsPanel::use_proxy_toggle());
    this->use_proxy_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_use_proxy(evt.IsChecked());
    });

    this->proxy_type_label =
        new wxStaticText(this, wxID_ANY, Strings::ConnectionSettingsPanel::proxy_type_label());

    this->proxy_type_combobox = new wxComboBox(
        this,
        wxID_ANY,
        Strings::ConnectionSettingsPanel::proxy_socks5(),
        wxDefaultPosition,
        wxDefaultSize,
        Strings::ConnectionSettingsPanel::proxy_types(),
        wxCB_READONLY
    );
    this->proxy_type_combobox->Bind(wxEVT_COMBOBOX, [this](wxCommandEvent& evt) {
        this->set_proxy_type(static_cast<ProxyType>(evt.GetInt()));
    });

    this->proxy_address_label =
        new wxStaticText(this, wxID_ANY, Strings::ConnectionSettingsPanel::proxy_address_label());
    this->proxy_address_textbox = new wxTextCtrl(this, wxID_ANY);
    this->proxy_address_textbox->SetHint(
        Strings::ConnectionSettingsPanel::proxy_address_textbox_hint()
    );
    this->proxy_port_label =
        new wxStaticText(this, wxID_ANY, Strings::ConnectionSettingsPanel::proxy_port_label());
    auto proxy_port_validator = wxIntegerValidator<uint16_t>(&this->proxy_port);
    proxy_port_validator.SetRange(1, 65535);
    this->proxy_port_textbox = new wxTextCtrl(
        this,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_RIGHT,
        proxy_port_validator
    );
    this->proxy_port_textbox->SetMinSize(
        wxSize(7 * Metrics::zero_width(*this->proxy_port_textbox), -1)
    );
    this->proxy_username_label =
        new wxStaticText(this, wxID_ANY, Strings::ConnectionSettingsPanel::proxy_username_label());
    this->proxy_username_textbox = new wxTextCtrl(this, wxID_ANY);
    this->proxy_username_textbox->SetHint(
        Strings::ConnectionSettingsPanel::proxy_username_textbox_hint()
    );
    this->proxy_password_label =
        new wxStaticText(this, wxID_ANY, Strings::ConnectionSettingsPanel::proxy_password_label());
    this->proxy_password_textbox = new wxTextCtrl(
        this,
        wxID_ANY,
        wxEmptyString,
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_PASSWORD
    );
    this->proxy_password_textbox->SetHint(
        Strings::ConnectionSettingsPanel::proxy_password_textbox_hint()
    );

    this->use_firewall_toggle =
        new wxCheckBox(this, wxID_ANY, Strings::ConnectionSettingsPanel::firewall_toggle());
    this->use_firewall_toggle->Bind(wxEVT_CHECKBOX, [this](wxCommandEvent& evt) {
        this->set_use_firewall(evt.IsChecked());
    });
    this->allowed_ports_label =
        new wxStaticText(this, wxID_ANY, Strings::ConnectionSettingsPanel::allowed_ports_label());
    this->allowed_ports_textbox = new wxTextCtrl(this, wxID_ANY);
    allowed_ports_textbox->SetHint(Strings::ConnectionSettingsPanel::allowed_ports_textbox_hint());

    // Layout

    v_sizer->Add(quickstart_heading, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(quickstart_description, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(
        this->connect_automatically_toggle,
        0,
        wxEXPAND | wxBOTTOM,
        Metrics::VERTICAL_PADDING_MEDIUM
    );

    v_sizer->Add(backend_heading, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(backend_description, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
#ifdef ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
    v_sizer->Add(this->bundled_legacy_tor_option, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_SMALL);
#endif // ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
#ifdef ENABLE_RICOCHET_REFRESH_EXTERNAL_TOR
    v_sizer->Add(this->external_legacy_tor_option, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_SMALL);
#endif // ENABLE_RICOCHET_REFRESH_EXTERNAL_TOR
#ifdef ENABLE_RICOCHET_REFRESH_ARTI_CLIENT
    v_sizer->Add(this->in_process_arti_option, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_SMALL);
#endif // ENABLE_RICOCHET_REFRESH_ARTI_CLIENT

    v_sizer->AddSpacer(Metrics::VERTICAL_PADDING_MEDIUM - Metrics::VERTICAL_PADDING_SMALL);

#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    v_sizer->Add(bridges_heading, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(bridges_description, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer
        ->Add(this->use_bridges_toggle, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    auto bridges_v_sizer = new wxBoxSizer(wxVERTICAL);
    v_sizer->Add(bridges_v_sizer, 0, wxEXPAND | wxLEFT, Metrics::HORIZONTAL_PADDING_XLARGE);
    bridges_v_sizer->Add(this->builtin_bridge_option, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_SMALL);
    auto builtin_bridges_v_sizer = new wxBoxSizer(wxVERTICAL);
    builtin_bridges_v_sizer
        ->Add(this->obfs4_bridge_option, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_SMALL);
    builtin_bridges_v_sizer->Add(
        this->obfs4_bridge_description,
        0,
        wxEXPAND | wxBOTTOM,
        Metrics::VERTICAL_PADDING_SMALL
    );
    builtin_bridges_v_sizer
        ->Add(this->snowflake_bridge_option, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_SMALL);
    builtin_bridges_v_sizer->Add(
        this->snowflake_bridge_description,
        0,
        wxEXPAND | wxBOTTOM,
        Metrics::VERTICAL_PADDING_SMALL
    );
    builtin_bridges_v_sizer
        ->Add(this->meek_bridge_option, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_SMALL);
    builtin_bridges_v_sizer->Add(
        this->meek_bridge_description,
        0,
        wxEXPAND | wxBOTTOM,
        Metrics::VERTICAL_PADDING_SMALL
    );

    bridges_v_sizer
        ->Add(builtin_bridges_v_sizer, 0, wxEXPAND | wxLEFT, Metrics::HORIZONTAL_PADDING_XLARGE);
    bridges_v_sizer->Add(this->custom_bridge_option, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_SMALL);
    bridges_v_sizer->Add(
        this->custom_bridge_textbox,
        0,
        wxEXPAND | wxLEFT,
        Metrics::HORIZONTAL_PADDING_XLARGE
    );
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS

    v_sizer->AddSpacer(Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(network_settings_heading, 0, wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
    v_sizer->Add(
        network_settings_description,
        0,
        wxEXPAND | wxBOTTOM,
        Metrics::VERTICAL_PADDING_MEDIUM
    );

    v_sizer->Add(this->use_proxy_toggle, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    auto proxy_v_sizer = new wxBoxSizer(wxVERTICAL);

    auto proxy_h_sizer_1 = new wxBoxSizer(wxHORIZONTAL);
    proxy_h_sizer_1->Add(this->proxy_type_label, 0, wxALIGN_CENTER_VERTICAL);
    proxy_h_sizer_1->AddStretchSpacer(1);
    proxy_h_sizer_1->Add(this->proxy_type_combobox, 0, wxALIGN_CENTER_VERTICAL);

    proxy_v_sizer->Add(proxy_h_sizer_1, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    auto proxy_h_sizer_2 = new wxBoxSizer(wxHORIZONTAL);
    proxy_h_sizer_2->Add(
        this->proxy_address_label,
        0,
        wxALIGN_CENTER_VERTICAL | wxRIGHT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
    proxy_h_sizer_2->Add(this->proxy_address_textbox, 1, wxALIGN_CENTER_VERTICAL);
    proxy_h_sizer_2->Add(
        this->proxy_port_label,
        0,
        wxALIGN_CENTER_VERTICAL | wxLEFT | wxRIGHT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
    proxy_h_sizer_2->Add(this->proxy_port_textbox, 0, wxALIGN_CENTER_VERTICAL);

    proxy_v_sizer->Add(proxy_h_sizer_2, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    auto proxy_h_sizer_3 = new wxBoxSizer(wxHORIZONTAL);
    proxy_h_sizer_3->Add(
        this->proxy_username_label,
        0,
        wxALIGN_CENTER_VERTICAL | wxRIGHT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
    proxy_h_sizer_3->Add(this->proxy_username_textbox, 1, wxALIGN_CENTER_VERTICAL);
    proxy_h_sizer_3->Add(
        this->proxy_password_label,
        0,
        wxALIGN_CENTER_VERTICAL | wxLEFT | wxRIGHT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
    proxy_h_sizer_3->Add(this->proxy_password_textbox, 1, wxALIGN_CENTER_VERTICAL);

    proxy_v_sizer->Add(proxy_h_sizer_3, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    v_sizer->Add(proxy_v_sizer, 0, wxEXPAND | wxLEFT, Metrics::HORIZONTAL_PADDING_XLARGE);

    v_sizer
        ->Add(this->use_firewall_toggle, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);

    auto firewall_v_sizer = new wxBoxSizer(wxVERTICAL);

    auto firewall_h_sizer = new wxBoxSizer(wxHORIZONTAL);
    firewall_h_sizer->Add(
        this->allowed_ports_label,
        0,
        wxALIGN_CENTER_VERTICAL | wxLEFT,
        Metrics::HORIZONTAL_PADDING_SMALL
    );
    firewall_h_sizer->Add(this->allowed_ports_textbox, 1, wxALIGN_CENTER_VERTICAL);

    firewall_v_sizer->Add(firewall_h_sizer, 0, wxEXPAND);
    v_sizer->Add(firewall_v_sizer, 0, wxEXPAND | wxLEFT, Metrics::HORIZONTAL_PADDING_XLARGE);

    h_sizer->Add(v_sizer, 1, wxEXPAND | wxLEFT | wxRIGHT, Metrics::HORIZONTAL_PADDING_MEDIUM);
    this->SetSizerAndFit(h_sizer);

    this->load_from_settings();
}

void ConnectionSettingsPanel::load_from_settings() {
    const auto& settings = wxGetApp().get_settings();

    auto connect_automatically = false;
    tego_settings_get_connect_automatically(
        &settings,
        &connect_automatically,
        tego::panic_on_error()
    );
    this->set_connect_automatically(connect_automatically);

    std::unique_ptr<tego_tor_config> tor_config;
    tego_settings_get_tor_config(&settings, tego::out(tor_config), tego::panic_on_error());

    tego_tor_config_type tor_config_type = {};
    tego_tor_config_get_type(tor_config.get(), &tor_config_type, tego::panic_on_error());

    switch (tor_config_type) {
#ifdef ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
        case tego_tor_config_type_bundled_tor: {
#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
            //
            // Bridge Settings
            //
            std::unique_ptr<tego_bridge_config> bridge_config;
            tego_tor_config_get_bridge_config(
                tor_config.get(),
                tego::out(bridge_config),
                tego::panic_on_error()
            );

            if (bridge_config.get()) {
                this->set_use_bridges(true);

                tego_bridge_config_type bridge_config_type = {};
                tego_bridge_config_get_type(
                    bridge_config.get(),
                    &bridge_config_type,
                    tego::panic_on_error()
                );
                this->set_bridge_type(static_cast<BridgeType>(bridge_config_type));

                switch (bridge_config_type) {
                    case tego_bridge_config_type_builtin: {
                        tego_bridge_builtin builtin;
                        tego_bridge_config_get_bridge_builtin(
                            bridge_config.get(),
                            &builtin,
                            tego::panic_on_error()
                        );
                        this->set_builtin_bridge(static_cast<BuiltinBridge>(builtin));

                    } break;
                    case tego_bridge_config_type_custom: {
                        std::unique_ptr<tego_string> bridge_lines;
                        tego_bridge_config_get_bridge_lines(
                            bridge_config.get(),
                            tego::out(bridge_lines),
                            tego::panic_on_error()
                        );
                        this->set_custom_bridges(into_wxString(bridge_lines));
                    } break;
                }
            } else {
                this->set_use_bridges(false);
            }
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
            //
            // Proxy Settings
            //
            std::unique_ptr<tego_proxy_config> proxy_config;
            tego_tor_config_get_proxy_config(
                tor_config.get(),
                tego::out(proxy_config),
                tego::panic_on_error()
            );

            if (proxy_config.get()) {
                this->set_use_proxy(true);

                tego_proxy_type proxy_type = {};
                tego_proxy_config_get_type(proxy_config.get(), &proxy_type, tego::panic_on_error());
                this->set_proxy_type(static_cast<ProxyType>(proxy_type));

                std::unique_ptr<tego_string> proxy_host;
                uint16_t proxy_port;
                tego_proxy_config_get_address(
                    proxy_config.get(),
                    tego::out(proxy_host),
                    &proxy_port,
                    tego::panic_on_error()
                );
                this->set_proxy_address(into_wxString(proxy_host), proxy_port);

                switch (proxy_type) {
                    case tego_proxy_type_socks5:
                    case tego_proxy_type_https: {
                        std::unique_ptr<tego_string> proxy_username, proxy_password;
                        tego_proxy_config_get_credentials(
                            proxy_config.get(),
                            tego::out(proxy_username),
                            tego::out(proxy_password),
                            tego::panic_on_error()
                        );
                        this->set_proxy_credentials(
                            proxy_username ? into_wxString(proxy_username) : wxString(),
                            proxy_password ? into_wxString(proxy_password) : wxString()
                        );
                    } break;
                }
            } else {
                this->set_use_proxy(false);
            }

            //
            // Firewall Settings
            //
            std::unique_ptr<tego_firewall_config> firewall_config;
            tego_tor_config_get_firewall_config(
                tor_config.get(),
                tego::out(firewall_config),
                tego::panic_on_error()
            );

            if (firewall_config.get()) {
                this->set_use_firewall(true);

                std::unique_ptr<tego_string> allowed_ports;
                tego_firewall_config_get_allowed_ports(
                    firewall_config.get(),
                    tego::out(allowed_ports),
                    tego::panic_on_error()
                );

                this->set_allowed_ports(into_wxString(allowed_ports));
            } else {
                this->set_use_firewall(false);
            }

        } break;
#endif // ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
        default:
            LOG_ERROR(
                fmt::format("Unknown tego_tor_config_type: {}", static_cast<int>(tor_config_type))
            );
    }
}

void ConnectionSettingsPanel::save_to_settings() {
    auto& settings = wxGetApp().get_settings_mut();

    auto connect_automatically = this->connect_automatically_toggle->GetValue();
    tego_settings_set_connect_automatically(
        &settings,
        connect_automatically,
        tego::panic_on_error()
    );

    std::unique_ptr<tego_tor_config> tor_config;

#ifdef ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
    if (this->bundled_legacy_tor_option->GetValue()) {
        //
        // Bridge settings
        //
        std::unique_ptr<tego_bridge_config> bridge_config;
#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
        if (this->use_bridges_toggle->GetValue()) {
            if (this->builtin_bridge_option->GetValue()) {
                if (this->obfs4_bridge_option->GetValue()) {
                    tego_bridge_config_new_builtin(
                        tego::out(bridge_config),
                        tego_bridge_builtin_obfs4,
                        tego::panic_on_error()
                    );
                } else if (this->snowflake_bridge_option->GetValue()) {
                    tego_bridge_config_new_builtin(
                        tego::out(bridge_config),
                        tego_bridge_builtin_snowflake,
                        tego::panic_on_error()
                    );
                } else if (this->meek_bridge_option->GetValue()) {
                    tego_bridge_config_new_builtin(
                        tego::out(bridge_config),
                        tego_bridge_builtin_meek,
                        tego::panic_on_error()
                    );
                }
            } else if (this->custom_bridge_option->GetValue()) {
                auto bridge_lines = into_tego_string(this->custom_bridge_textbox->GetValue());
                tego_bridge_config_new_custom(
                    tego::out(bridge_config),
                    bridge_lines.get(),
                    tego::throw_on_error()
                );
            }
        }

#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
        //
        // Proxy settings
        //
        std::unique_ptr<tego_proxy_config> proxy_config;
        if (this->use_proxy_toggle->GetValue()) {
            auto proxy_type =
                static_cast<tego_proxy_type>(this->proxy_type_combobox->GetSelection());
            const auto address = this->proxy_address_textbox->GetValue();
            this->proxy_port_textbox->TransferDataFromWindow();
            const auto port = this->proxy_port;
            const auto username = this->proxy_username_textbox->GetValue();
            const auto password = this->proxy_password_textbox->GetValue();

            switch (proxy_type) {
                case tego_proxy_type_socks4: {
                    tego_proxy_config_new_socks4(
                        tego::out(proxy_config),
                        into_tego_string(address).get(),
                        port,
                        tego::throw_on_error()
                    );
                } break;
                case tego_proxy_type_socks5: {
                    tego_proxy_config_new_socks5(
                        tego::out(proxy_config),
                        into_tego_string(address).get(),
                        port,
                        into_tego_string(username).get(),
                        into_tego_string(password).get(),
                        tego::throw_on_error()
                    );
                } break;
                case tego_proxy_type_https: {
                    tego_proxy_config_new_https(
                        tego::out(proxy_config),
                        into_tego_string(address).get(),
                        port,
                        into_tego_string(username).get(),
                        into_tego_string(password).get(),
                        tego::throw_on_error()
                    );
                } break;
            }
        }

        //
        // Firewall settings
        //
        std::unique_ptr<tego_firewall_config> firewall_config;
        if (this->use_firewall_toggle->GetValue()) {
            const auto allowed_ports = this->allowed_ports_textbox->GetValue();

            tego_firewall_config_new(
                tego::out(firewall_config),
                into_tego_string(allowed_ports).get(),
                tego::throw_on_error()
            );
        }

        // build tor config
        tego_tor_config_new_bundled_tor(
            tego::out(tor_config),
            bridge_config.get(),
            proxy_config.get(),
            firewall_config.get(),
            tego::panic_on_error()
        );
    }
#endif // ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
    tego_settings_set_tor_config(&settings, tor_config.get(), tego::panic_on_error());
}

void ConnectionSettingsPanel::set_tor_backend(TorBackend tor_backend) {
    switch (tor_backend) {
#ifdef ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
        case TorBackend::BundledLegacyTor:
            LOG_INFO("Set Backend: Bundled Legacy Tor");
            this->bundled_legacy_tor_option->SetValue(true);
            break;
#endif // ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
#ifdef ENABLE_RICOCHET_REFRESH_EXTERNAL_TOR
        case TorBackend::ExternalLegacyTor:
            LOG_INFO("Set Backend: External Legacy Tor");
            this->external_legacy_tor_option->SetValue(true);
            break;
#endif // ENABLE_RICOCHET_REFRESH_EXTERNAL_TOR
#ifdef ENABLE_RICOCHET_REFRESH_ARTI_CLIENT
        case TorBackend::InProcessArti:
            LOG_INFO("Set Backend: In-Process Arti");
            this->in_process_arti_option->SetValue(true);
            break;
#endif // ENABLE_RICOCHET_REFRESH_ARTI_CLIENT
    }
}

void ConnectionSettingsPanel::set_connect_automatically(bool enabled) {
    LOG_INFO(fmt::format("Set Connect Automatically: : {}", enabled));
    this->connect_automatically_toggle->SetValue(enabled);
}

#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
void ConnectionSettingsPanel::set_use_bridges(bool enabled) {
    LOG_INFO(fmt::format("Set Use Bridges: : {}", enabled));
    this->use_bridges_toggle->SetValue(enabled);
    if (enabled) {
        this->enable_bridge_controls();
        if (this->builtin_bridge_option->GetValue()) {
            this->enable_builtin_bridge_controls();
            this->disable_custom_bridge_controls();
        } else if (this->custom_bridge_option->GetValue()) {
            this->disable_builtin_bridge_controls();
            this->enable_custom_bridge_controls();
        }
    } else {
        this->disable_bridge_controls();
        this->disable_builtin_bridge_controls();
        this->disable_custom_bridge_controls();
    }
}

void ConnectionSettingsPanel::set_bridge_type(BridgeType bridge_type) {
    switch (bridge_type) {
        case BridgeType::Builtin:
            LOG_INFO("Set BridgeType: Builtin");
            this->builtin_bridge_option->SetValue(true);
            this->enable_builtin_bridge_controls();
            this->disable_custom_bridge_controls();
            break;
        case BridgeType::Custom:
            LOG_INFO("Set BridgeType: Custom");
            this->custom_bridge_option->SetValue(true);
            this->disable_builtin_bridge_controls();
            this->enable_custom_bridge_controls();
            break;
    }
}

void ConnectionSettingsPanel::set_builtin_bridge(BuiltinBridge builtin_bridge) {
    this->builtin_bridge_option->SetValue(true);
    switch (builtin_bridge) {
        case BuiltinBridge::Obfs4:
            LOG_INFO("Set BuiltinBridge: Obfs4");
            this->obfs4_bridge_option->SetValue(true);
            break;
        case BuiltinBridge::Snowflake:
            LOG_INFO("Set BuiltinBridge: Snowflake");
            this->snowflake_bridge_option->SetValue(true);
            break;
        case BuiltinBridge::Meek:
            LOG_INFO("Set BuiltinBridge: Meek");
            this->meek_bridge_option->SetValue(true);
            break;
    }
}

void ConnectionSettingsPanel::set_custom_bridges(wxString bridge_lines) {
    this->custom_bridge_option->SetValue(true);
    this->custom_bridge_textbox->Clear();
    this->custom_bridge_textbox->WriteText(bridge_lines);
    this->custom_bridge_textbox->SetInsertionPoint(0);
}
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS

void ConnectionSettingsPanel::set_use_proxy(bool enabled) {
    LOG_INFO(fmt::format("Set Use Proxy: : {}", enabled));
    this->use_proxy_toggle->SetValue(enabled);
    if (enabled) {
        this->enable_proxy_address_controls();
        const auto& proxy_type = static_cast<ProxyType>(this->proxy_type_combobox->GetSelection());
        if (proxy_type == ProxyType::SOCKS5 || proxy_type == ProxyType::HTTPS) {
            this->enable_proxy_authentication_controls();
        } else {
            this->disable_proxy_authentication_controls();
        }
    } else {
        this->disable_proxy_address_controls();
        this->disable_proxy_authentication_controls();
    }
}

void ConnectionSettingsPanel::set_proxy_type(ProxyType proxy_type) {
    this->proxy_type_combobox->SetSelection(static_cast<int>(proxy_type));
    switch (proxy_type) {
        case ProxyType::SOCKS4:
            LOG_INFO("Set Proxy Type: SOCKS4");
            this->disable_proxy_authentication_controls();
            break;
        case ProxyType::SOCKS5:
            LOG_INFO("Set Proxy Type: SOCKS5");
            this->enable_proxy_authentication_controls();
            break;
        case ProxyType::HTTPS:
            LOG_INFO("Set Proxy Type: HTTPS");
            this->enable_proxy_authentication_controls();
            break;
    }
}

void ConnectionSettingsPanel::set_proxy_address(wxString host, uint16_t port) {
    LOG_INFO(fmt::format("Set Proxy Address: {}:{}", host, port));
    this->proxy_address_textbox->SetValue(host);
    this->proxy_port = port;
    this->proxy_port_textbox->TransferDataToWindow();
}

void ConnectionSettingsPanel::set_proxy_credentials(wxString username, wxString password) {
    LOG_INFO(
        fmt::format("Set Proxy Credentials: {{ username: '{}', password: '********' }}", username)
    );
    this->proxy_username_textbox->SetValue(username);
    this->proxy_password_textbox->SetValue(password);
}

void ConnectionSettingsPanel::set_use_firewall(bool enabled) {
    LOG_INFO(fmt::format("Set Use Firewall: {}", enabled));
    this->use_firewall_toggle->SetValue(enabled);
    if (enabled) {
        this->enable_firewall_controls();
    } else {
        this->disable_firewall_controls();
    }
}

void ConnectionSettingsPanel::set_allowed_ports(wxString allowed_ports) {
    LOG_INFO(fmt::format("Set Allowed Ports: {}", allowed_ports));
    this->allowed_ports_textbox->SetValue(allowed_ports);
}

#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
void ConnectionSettingsPanel::enable_bridge_controls() {
    this->builtin_bridge_option->Enable();
    this->custom_bridge_option->Enable();
}

void ConnectionSettingsPanel::enable_builtin_bridge_controls() {
    this->obfs4_bridge_option->Enable();
    this->obfs4_bridge_description->Enable();
    this->snowflake_bridge_option->Enable();
    this->snowflake_bridge_description->Enable();
    this->meek_bridge_option->Enable();
    this->meek_bridge_description->Enable();
}

void ConnectionSettingsPanel::enable_custom_bridge_controls() {
    this->custom_bridge_textbox->Enable();
}
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS

void ConnectionSettingsPanel::enable_proxy_address_controls() {
    this->proxy_type_label->Enable();
    this->proxy_type_combobox->Enable();
    this->proxy_address_label->Enable();
    this->proxy_address_textbox->Enable();
    this->proxy_port_label->Enable();
    this->proxy_port_textbox->Enable();
}

void ConnectionSettingsPanel::enable_proxy_authentication_controls() {
    this->proxy_username_label->Enable();
    this->proxy_username_textbox->Enable();
    this->proxy_password_label->Enable();
    this->proxy_password_textbox->Enable();
}

void ConnectionSettingsPanel::enable_firewall_controls() {
    this->allowed_ports_label->Enable();
    this->allowed_ports_textbox->Enable();
}

#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
void ConnectionSettingsPanel::disable_bridge_controls() {
    this->builtin_bridge_option->Disable();
    this->custom_bridge_option->Disable();
    this->disable_builtin_bridge_controls();
    this->disable_custom_bridge_controls();
}

void ConnectionSettingsPanel::disable_builtin_bridge_controls() {
    this->obfs4_bridge_option->Disable();
    this->obfs4_bridge_description->Disable();
    this->snowflake_bridge_option->Disable();
    this->snowflake_bridge_description->Disable();
    this->meek_bridge_option->Disable();
    this->meek_bridge_description->Disable();
}

void ConnectionSettingsPanel::disable_custom_bridge_controls() {
    this->custom_bridge_textbox->Disable();
}
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS

void ConnectionSettingsPanel::disable_proxy_address_controls() {
    this->proxy_type_label->Disable();
    this->proxy_type_combobox->Disable();
    this->proxy_address_label->Disable();
    this->proxy_address_textbox->Disable();
    this->proxy_port_label->Disable();
    this->proxy_port_textbox->Disable();
}

void ConnectionSettingsPanel::disable_proxy_authentication_controls() {
    this->proxy_username_label->Disable();
    this->proxy_username_textbox->Disable();
    this->proxy_password_label->Disable();
    this->proxy_password_textbox->Disable();
}

void ConnectionSettingsPanel::disable_firewall_controls() {
    this->allowed_ports_label->Disable();
    this->allowed_ports_textbox->Disable();
}
