#pragma once

class FileTransfersPanel: public wxPanel {
public:
    explicit FileTransfersPanel(wxWindow* parent);
private:
    void close();

    wxDataViewCtrl* file_transfers_data_view_ctrl = nullptr;
};
