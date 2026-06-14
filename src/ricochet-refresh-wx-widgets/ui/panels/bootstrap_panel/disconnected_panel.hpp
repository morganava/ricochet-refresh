#pragma once

class DisconnectedPanel: public wxPanel {
public:
    explicit DisconnectedPanel(wxWindow* parent);

private:
    void configure();
    void connect();
};
