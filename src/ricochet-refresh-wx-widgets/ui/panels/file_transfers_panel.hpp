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

    void add_pending_upload(
        FileTransferKey key,
        wxString filename,
        wxString location,
        uint64_t size,
        wxString hash,
        wxString receiver
    );
    void add_pending_download(
        FileTransferKey key,
        wxString original_filename,
        uint64_t size,
        wxString hash,
        wxString sender
    );
    void begin_pending_download(FileTransferKey key, wxString new_filename, wxString location);
    void update_transfer_progress(FileTransferKey key, uint64_t bytes_transferred);
    void set_transfer_complete(FileTransferKey key);
    void set_transfer_cancelled(FileTransferKey key);
    void set_transfer_failed(FileTransferKey key);

private:
    // fired when user clicks one of the headers, forces resort
    void on_sort(const wxDataViewEvent& event);
    // fired when a user selects/focuses a row
    void on_row_select(const wxDataViewEvent& event);
    // fired every 500ms to update rows rendered values and force resort
    void on_timer_event();
    // sort the underlying model
    void sort_rows();

    void close();

    // the index into the v_sizer we insert details panels
    constexpr static size_t FILE_TRANSFER_DETAILS_INDEX = 2;
    wxBoxSizer* v_sizer = nullptr;
    wxDataViewCtrl* file_transfers_data_view_ctrl = nullptr;
    wxObjectDataPtr<FileTransfersPanelListModel> list_model;
    std::map<FileTransferKey, FileTransferDetailsPanel*> details_panels;
    std::optional<FileTransferKey> selected_file_transfer = std::nullopt;

    // periodically updates the rows
    wxTimer update_timer;
};
