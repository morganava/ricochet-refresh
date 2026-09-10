#pragma once

// TODO:
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

enum class FileTransferDirection;
enum class FileTransferStatus;
class FileTransfersPanelListModel;
typedef std::tuple<tego_session_handle, tego_user_handle, tego_file_transfer_id> FileTransferKey;
class FileTransferDetailsPanel;

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
    // sort the underlying model
    void sort_rows();

    void close();

    constexpr static size_t FILE_TRANSFER_DETAILS_INDEX = 2;
    wxBoxSizer* v_sizer = nullptr;
    wxDataViewCtrl* file_transfers_data_view_ctrl = nullptr;
    wxObjectDataPtr<FileTransfersPanelListModel> list_model;
    std::map<FileTransferKey, FileTransferDetailsPanel*> details_panels;
    std::optional<FileTransferKey> selected_file_transfer = std::nullopt;

    // periodically updates the rows
    wxTimer update_timer;
};
