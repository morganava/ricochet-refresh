#include "connection_status_panel.hpp"

#include "enums.hpp"
#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"

ConnectionStatusPanel::ConnectionStatusPanel(wxWindow* parent) : wxPanel(parent) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto title = new wxStaticText(this, wxID_ANY, Strings::ConnectionStatusPanel::title());
    title->SetFont(Fonts::title_font());

    // status

    auto status_sizer = new wxBoxSizer(wxHORIZONTAL);

    this->backend_text = new wxStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionStatusPanel::backend_label(
            Strings::ConnectionStatusPanel::none_client_string()
        )
    );
    this->connection_status_text = new wxStaticText(
        this,
        wxID_ANY,
        Strings::ConnectionStatusPanel::status_label(ConnectionStatus::Offline)
    );

    status_sizer->Add(this->backend_text, 0);
    status_sizer->AddStretchSpacer(1);
    status_sizer->Add(this->connection_status_text, 0);

    // logs

    this->logs_textbox = new wxTextCtrl(
        this,
        wxID_ANY,
        "",
        wxDefaultPosition,
        wxDefaultSize,
        wxTE_MULTILINE | wxTE_READONLY | wxTE_WORDWRAP
    );

    // buttons

    auto button_sizer = new wxBoxSizer(wxHORIZONTAL);

    this->copy_logs_button =
        new wxButton(this, wxID_ANY, Strings::ConnectionStatusPanel::copy_tor_logs_button());
    this->copy_logs_button_timer.Bind(wxEVT_TIMER, [this](wxTimerEvent&) {
        this->copy_logs_button->SetLabel(Strings::ConnectionStatusPanel::copy_tor_logs_button());
        this->Layout();
    });

    this->copy_logs_button->Bind(wxEVT_BUTTON, [=, this](wxCommandEvent&) {
        this->copy_tor_logs();

        this->copy_logs_button->SetLabel(Strings::ConnectionStatusPanel::copied());
        this->Layout();

        this->copy_logs_button_timer.Stop();
        this->copy_logs_button_timer.StartOnce(1500);
    });

    auto close_button = new wxButton(this, wxID_OK, Strings::ConnectionStatusPanel::close_button());
    close_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->close(); });

    button_sizer->Add(this->copy_logs_button, 0);
    button_sizer->AddStretchSpacer(1);
    button_sizer->Add(close_button, 0);

    v_sizer->Add(title, 0, wxEXPAND | wxALL, Metrics::PADDING_MEDIUM);
    v_sizer->Add(status_sizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(logs_textbox, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->Add(button_sizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);
    this->SetSizerAndFit(v_sizer);
}

void ConnectionStatusPanel::reset_widgets() {
    this->backend_text->SetLabel(Strings::ConnectionStatusPanel::backend_label(
        Strings::ConnectionStatusPanel::none_client_string()
    ));
    this->connection_status_text->SetLabel(
        Strings::ConnectionStatusPanel::status_label(ConnectionStatus::Offline)
    );
    this->logs_textbox->SetValue("");

    this->Layout();
}

void ConnectionStatusPanel::set_backend(const wxString& backend) {
    this->backend_text->SetLabel(Strings::ConnectionStatusPanel::backend_label(backend));
    this->Layout();
}

void ConnectionStatusPanel::add_log(const wxString& log_line) {
    this->logs_textbox->AppendText(log_line);
    this->logs_textbox->AppendText(Strings::Common::new_line());
}

void ConnectionStatusPanel::set_connection_status(ConnectionStatus connection_status) {
    this->connection_status_text->SetLabel(
        Strings::ConnectionStatusPanel::status_label(connection_status)
    );
    this->Layout();
}

bool ConnectionStatusPanel::copy_tor_logs() {
    auto clipboard = wxTheClipboard;
    if (clipboard->Open()) {
        clipboard->SetData(new wxTextDataObject(this->logs_textbox->GetValue()));
        clipboard->Flush();
        clipboard->Close();

        LOG_INFO("Copied Tor Logs");
        return true;
    } else {
        LOG_ERROR("Could not open clipboard");
        return false;
    }
}

void ConnectionStatusPanel::close() {
    LOG_INFO("Close");

    wxGetApp().get_main_frame().hide_overlay_panel();
}
