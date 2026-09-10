#pragma once

enum class FileTransferDirection;
enum class Ordering;
enum class FileTransfersPanelListColumn : unsigned int;
enum class SortDirection;
enum class FileTransferStatus;
class FileTransferRow;

// TODO:
// ensure selection index is updated when we insert a row (and remove a row)
// de-dupe code in on_sort and on_timer_event for maintaining selection index
// filename should show name chosen by local user
// clicking on entry shows extended information
// - extended information should show the original name for downloads
// stop updating rows when complete
// transfers should be added when they start not when received
// right click menu to:
// - Open Folder
// - Cancel Transfer
// - Remove transfer
// - Remove and Delete files (for downlaods)
// Update 'Status' entry to also include "Cancelled" or "Error"

class FileTransfersPanelListModel: public wxDataViewVirtualListModel {
public:
    FileTransfersPanelListModel();

    void add_file_transfer_row(
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id file_transfer_id,
        FileTransferRow&& row
    );

    // called on a regular interval to update each of our row's speed-derived stats
    void update(const wxDateTime& now);

    // called when receiving file transfer-related events to update a particular row's raw values
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
    // update the sort column, direction, and selected row index, returning the new selected row index
    std::optional<size_t> sort(
        FileTransfersPanelListColumn column,
        SortDirection direction,
        std::optional<size_t> selected_index
    );
    // update the selected index and sort, returning the new selected row index
    std::optional<size_t> sort(std::optional<size_t> selected_index);
    // sort the rows and return the new index of the selected row index
    std::optional<size_t> sort();

    typedef std::tuple<tego_session_handle, tego_user_handle, tego_file_transfer_id>
        FileTransferKey;

    bool less(const FileTransferKey& a, const FileTransferKey& b);

    std::map<FileTransferKey, FileTransferRow> file_transfer_rows;
    std::vector<FileTransferKey> index_vector;

    FileTransfersPanelListColumn sort_column;
    SortDirection sort_direction;
    std::optional<size_t> selected_index = std::nullopt;

    friend class FileTransfersPanel;
};

class FileTransfersPanel: public wxPanel {
public:
    explicit FileTransfersPanel(wxWindow* parent);

    void add_file_transfer(
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id id,
        wxString filename,
        uint64_t size,
        FileTransferDirection direction,
        wxString sender,
        wxString receiver
    );

    void update_file_transfer_progress(
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id id,
        uint64_t bytes_transferred
    );

    void update_file_transfer_status(
        tego_session_handle session_handle,
        tego_user_handle user_handle,
        tego_file_transfer_id id,
        FileTransferStatus status
    );

private:
    // fired when user clicks one of the headers, forces resort
    void on_sort(const wxDataViewEvent& event);
    // fired every 500ms to update rows rendered values and force resort
    void on_timer_event();
    void close();

    wxDataViewCtrl* file_transfers_data_view_ctrl = nullptr;
    wxObjectDataPtr<FileTransfersPanelListModel> list_model;
    wxTimer update_timer;
};
