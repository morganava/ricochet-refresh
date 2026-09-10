#pragma once

class FileTransferRow;
enum class FileTransfersPanelListColumn : unsigned int;
enum class SortDirection;
typedef std::tuple<tego_session_handle, tego_user_handle, tego_file_transfer_id> FileTransferKey;

class FileTransfersPanelListModel: public wxDataViewVirtualListModel {
public:
    FileTransfersPanelListModel();

    void add_file_transfer_row(FileTransferKey key, FileTransferRow&& row);

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

    void set_sort_params(FileTransfersPanelListColumn column, SortDirection direction);

    // update the selected index and sort, returning the new selected row index
    std::optional<size_t> sort(std::optional<size_t> selected_index);

private:
    // sort the rows and return the new index of the selected row index
    std::optional<size_t> sort();

    bool less(const FileTransferKey& a, const FileTransferKey& b);

    std::map<FileTransferKey, FileTransferRow> file_transfer_rows;
    std::vector<FileTransferKey> index_vector;

    FileTransfersPanelListColumn sort_column;
    SortDirection sort_direction;
    std::optional<size_t> selected_index = std::nullopt;
};
