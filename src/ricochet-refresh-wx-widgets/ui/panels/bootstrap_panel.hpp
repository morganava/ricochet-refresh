#pragma once

class BootstrapPanel: public wxPanel {
public:
    explicit BootstrapPanel(wxWindow* parent);
    void show_disconnected();
    void show_connecting();
    void show_connected();

    class ConnectingPanel& get_connecting_panel_mut() {
        return *this->connecting_panel;
    }

private:
    class DisconnectedPanel* disconnected_panel = nullptr;
    class ConnectingPanel* connecting_panel = nullptr;
    class ConnectedPanel* connected_panel = nullptr;
};
