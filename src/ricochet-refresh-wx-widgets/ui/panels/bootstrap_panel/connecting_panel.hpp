#pragma once

class ConnectingPanel: public wxPanel {
public:
    explicit ConnectingPanel(wxWindow* parent);

    void update_progress_bar(unsigned n);

private:
    void view_logs();
    void cancel();

    wxGauge* progress_bar = nullptr;
};
