#pragma once

struct WrappedStaticText;

enum class TorBackend;
enum class BridgeType;
enum class BuiltinBridge;
enum class ProxyType;

class ConnectionSettingsPanel: public wxScrolled<wxPanel> {
public:
    explicit ConnectionSettingsPanel(wxWindow* parent);

    // load settings from disk and populate the ui
    void load_from_settings();
    // save settings from ui to disk
    void save_to_settings();

private:
    // setters
    void set_tor_backend(TorBackend);
    void set_connect_automatically(bool);
#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    void set_use_bridges(bool);
    void set_bridge_type(BridgeType);
    void set_builtin_bridge(BuiltinBridge);
    void set_custom_bridges(wxString);
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    void set_use_proxy(bool);
    void set_proxy_type(ProxyType);
    void set_proxy_address(wxString, uint16_t);
    void set_proxy_credentials(wxString, wxString);
    void set_use_firewall(bool);
    void set_allowed_ports(wxString);

#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    void enable_bridge_controls();
    void enable_builtin_bridge_controls();
    void enable_custom_bridge_controls();
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    void enable_proxy_address_controls();
    void enable_proxy_authentication_controls();
    void enable_firewall_controls();

#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    void disable_bridge_controls();
    void disable_builtin_bridge_controls();
    void disable_custom_bridge_controls();
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    void disable_proxy_address_controls();
    void disable_proxy_authentication_controls();
    void disable_firewall_controls();

    //
    // Widgets
    //
    wxCheckBox* connect_automatically_toggle = nullptr;
#ifdef ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
    wxRadioButton* bundled_legacy_tor_option = nullptr;
#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    // Bridges
    wxCheckBox* use_bridges_toggle = nullptr;
    wxRadioButton* builtin_bridge_option = nullptr;
    wxRadioButton* obfs4_bridge_option = nullptr;
    WrappedStaticText* obfs4_bridge_description = nullptr;
    wxRadioButton* snowflake_bridge_option = nullptr;
    WrappedStaticText* snowflake_bridge_description = nullptr;
    wxRadioButton* meek_bridge_option = nullptr;
    WrappedStaticText* meek_bridge_description = nullptr;
    wxRadioButton* custom_bridge_option = nullptr;
    wxTextCtrl* custom_bridge_textbox = nullptr;
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    // Proxy
    wxCheckBox* use_proxy_toggle = nullptr;
    wxStaticText* proxy_type_label = nullptr;
    wxComboBox* proxy_type_combobox = nullptr;
    wxStaticText* proxy_address_label = nullptr;
    wxTextCtrl* proxy_address_textbox = nullptr;
    wxStaticText* proxy_port_label = nullptr;
    wxTextCtrl* proxy_port_textbox = nullptr;
    wxStaticText* proxy_username_label = nullptr;
    wxTextCtrl* proxy_username_textbox = nullptr;
    wxStaticText* proxy_password_label = nullptr;
    wxTextCtrl* proxy_password_textbox = nullptr;
    // Firewall
    wxCheckBox* use_firewall_toggle = nullptr;
    wxStaticText* allowed_ports_label = nullptr;
    wxTextCtrl* allowed_ports_textbox = nullptr;
#endif // ENABLE_RICOCHET_REFRESH_BUNDLED_TOR
#ifdef ENABLE_RICOCHET_REFRESH_EXTERNAL_TOR
    wxRadioButton* external_legacy_tor_option = nullptr;
#endif // ENABLE_RICOCHET_REFRESH_EXTERNAL_TOR
#ifdef ENABLE_RICOCHET_REFRESH_ARTI_CLIENT
    wxRadioButton* in_process_arti_option = nullptr;
#endif // ENABLE_RICOCHET_REFRESH_ARTI_CLIENT

    TorBackend backend;
    bool connect_automatically;
#ifdef ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    bool use_bridges;
    BridgeType bridge_type;
    BuiltinBridge builtin_bridge;
    wxString custom_bridges;
#endif // ENABLE_RICOCHET_REFRESH_PLUGGABLE_TRANSPORTS
    uint16_t proxy_port;
    wxString proxy_address;
    wxString proxy_username;
    wxString proxy_password;
    bool use_firewall;
    wxString allowed_ports;
};