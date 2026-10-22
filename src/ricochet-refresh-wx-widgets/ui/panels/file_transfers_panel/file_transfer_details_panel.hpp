#pragma once

enum class FileTransferStatus;

class FileTransferDetailsPanel: public wxPanel {
public:
    // pending upload constructor
    FileTransferDetailsPanel(
        wxWindow* parent,
        const wxString location,
        uint64_t size,
        const wxString& hash
    );
    // pending download constructor
    FileTransferDetailsPanel(wxWindow* parent, uint64_t size, const wxString& hash);

    // update rendering the UI
    void update(const wxDateTime& now);

    void begin_pending_download(wxString location);
    void update_transfer_progress(uint64_t bytes_transferred);
    void set_transfer_complete();
    void set_transfer_cancelled();
    void set_transfer_failed();

    // todo: replace thes

    // updates for the backing data
    void update_status(FileTransferStatus status);
    void update_location(wxString location);

private:
    FileTransferDetailsPanel(wxWindow* parent);
    // widgets
    wxStaticText* time_elapsed_value = nullptr;
    wxStaticText* transferred_value = nullptr;
    wxStaticText* average_speed_value = nullptr;
    wxStaticText* status_value = nullptr;
    wxStaticText* location_value = nullptr;
    wxStaticText* size_value = nullptr;
    wxStaticText* hash_value = nullptr;

    // progress data

    const tego_file_size size;
    FileTransferStatus status;
    std::optional<wxDateTime> transfer_start_time = std::nullopt;
    std::optional<wxDateTime> transfer_end_time = std::nullopt;
    std::optional<tego_file_size> bytes_transferred = std::nullopt;
};
