#pragma once

enum class FileTransferDirection;

struct FileTransferRow {
    wxString filename;
    uint64_t size;
    uint64_t bytes_transferred;
    FileTransferDirection direction;
    wxString sender;
    wxString receiver;
    uint64_t speed; // bytes per second
    wxTimeSpan eta;
};

class FileTransfersPanelListModel: public wxDataViewVirtualListModel {
public:
    FileTransfersPanelListModel();

    void AddFileTransferRow(
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id file_transfer_id,
        FileTransferRow&& row
    );

    // Added Sort method to handle the physical reordering of the vector
    void Sort(unsigned int column, bool ascending);

    // We need this to tell the control how many rows to render
    virtual unsigned int GetCount() const override {
        return index_vector.size();
    }

    virtual void
    GetValueByRow(wxVariant& variant, unsigned int row, unsigned int col) const override;
    virtual bool
    GetAttrByRow(unsigned int row, unsigned int col, wxDataViewItemAttr& attr) const override;
    virtual bool
    SetValueByRow(const wxVariant& variant, unsigned int row, unsigned int col) override;
    virtual int Compare(
        const wxDataViewItem& item1,
        const wxDataViewItem& item2,
        unsigned int column,
        bool ascending
    ) const override;

private:
    std::map<
        std::tuple<tego_session_handle, tego_user_handle, tego_file_transfer_id>,
        FileTransferRow>
        file_transfer_rows;

    std::vector<std::tuple<tego_session_handle, tego_user_handle, tego_file_transfer_id>>
        index_vector;
};

class FileTransfersPanel: public wxPanel {
public:
    explicit FileTransfersPanel(wxWindow* parent);

private:
    void close();

    wxDataViewCtrl* file_transfers_data_view_ctrl = nullptr;
    wxObjectDataPtr<FileTransfersPanelListModel> list_model;
};
