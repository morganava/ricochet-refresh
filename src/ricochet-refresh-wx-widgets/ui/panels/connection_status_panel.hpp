#pragma once

#include "enums.hpp"

class ConnectionStatusPanel: public wxPanel {
public:
    explicit ConnectionStatusPanel(wxWindow* parent);

    // reset widgets to default (e.g. after cancelling bootstrap)
    void reset_widgets();

    // populate/update various widgets
    void set_backend(const wxString& backend);
    void add_log(const wxString& log_line);
    void set_connection_status(ConnectionStatus connection_status);

private:
    bool copy_tor_logs();
    void close();

    wxStaticText* backend_text = nullptr;
    wxStaticText* connection_status_text = nullptr;
    wxTextCtrl* logs_textbox = nullptr;
    wxButton* copy_logs_button = nullptr;
    wxTimer copy_logs_button_timer;
};
