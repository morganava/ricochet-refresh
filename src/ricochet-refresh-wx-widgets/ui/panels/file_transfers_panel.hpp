#pragma once

enum class FileTransferDirection;
enum class Ordering;
enum class FileTransfersPanelListColumn : unsigned int;
enum class SortDirection;

struct FileTransferRow {
    wxString filename;
    uint64_t size;
    uint64_t bytes_transferred;
    FileTransferDirection direction;
    wxString sender;
    wxString receiver;
    uint64_t speed; // bytes per second
    wxTimeSpan eta;
    wxDateTime date_added;

    static Ordering compare(
        const FileTransferRow& row1,
        const FileTransferRow& row2,
        FileTransfersPanelListColumn column
    );
};

class FileTransfersPanelListModel: public wxDataViewVirtualListModel {
public:
    FileTransfersPanelListModel();

    void add_file_transfer_row(
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id file_transfer_id,
        FileTransferRow&& row
    );

    void update_file_transfer_row(
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id file_transfer_id,
        uint64_t bytes_transferred
    );

    virtual unsigned int GetCount() const override;
    virtual void
    GetValueByRow(wxVariant& variant, unsigned int row, unsigned int col) const override;
    virtual bool
    SetValueByRow(const wxVariant& variant, unsigned int row, unsigned int col) override;

private:
    void sort(FileTransfersPanelListColumn column, SortDirection direction);
    void sort();

    typedef std::tuple<tego_session_handle, tego_user_handle, tego_file_transfer_id>
        FileTransferKey;

    bool less(const FileTransferKey& a, const FileTransferKey& b);

    std::map<FileTransferKey, FileTransferRow> file_transfer_rows;
    std::vector<FileTransferKey> index_vector;

    FileTransfersPanelListColumn sort_column;
    SortDirection sort_direction;

    friend class FileTransfersPanel;
};

class FileTransfersPanel: public wxPanel {
public:
    explicit FileTransfersPanel(wxWindow* parent);

private:
    void on_sort(const wxDataViewEvent& event);
    void close();

    wxDataViewCtrl* file_transfers_data_view_ctrl = nullptr;
    wxObjectDataPtr<FileTransfersPanelListModel> list_model;
};
