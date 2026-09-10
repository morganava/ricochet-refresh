#include "file_transfers_panel_list_model.hpp"

#include "enums.hpp"
#include "ffi.hpp"
#include "strings.hpp"
#include "ui/panels/file_transfers_panel/file_transfer_row.hpp"

//
// FileTransferPanelListModel
//

FileTransfersPanelListModel::FileTransfersPanelListModel() :
    wxDataViewVirtualListModel(),
    sort_column(FileTransfersPanelListColumn::DateAdded),
    sort_direction(SortDirection::Ascending) {}

void FileTransfersPanelListModel::add_file_transfer_row(
    FileTransferKey key,
    FileTransferRow&& row
) {
    file_transfer_rows.emplace(std::make_pair(key, row));

    auto it = index_vector.insert(
        std::upper_bound(
            index_vector.begin(),
            index_vector.end(),
            key,
            [this](const FileTransferKey& a, const FileTransferKey& b) { return this->less(a, b); }
        ),
        key
    );
    auto index = std::distance(index_vector.begin(), it);
    this->RowInserted(static_cast<unsigned int>(index));
}

void FileTransfersPanelListModel::update(const wxDateTime& now) {
    auto tego_now = into_tego_time(now);
    for (auto& it : file_transfer_rows) {
        it.second.update(tego_now);
    }
}

void FileTransfersPanelListModel::update_file_transfer_row(
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id file_transfer_id,
    uint64_t bytes_transferred
) {
    auto key = std::make_tuple(session_handle, user_handle, file_transfer_id);
    if (auto it = this->file_transfer_rows.find(key); it != this->file_transfer_rows.end()) {
        it->second.update_bytes_transferred(bytes_transferred, into_tego_time(wxDateTime::Now()));
    }
}

FileTransferKey FileTransfersPanelListModel::get_file_transfer_key(size_t index) {
    return this->index_vector[index];
}

unsigned int FileTransfersPanelListModel::GetCount() const {
    return static_cast<unsigned int>(this->index_vector.size());
}

void FileTransfersPanelListModel::GetValueByRow(
    wxVariant& variant,
    unsigned int row,
    unsigned int col
) const {
    if (row >= index_vector.size())
        return;

    const auto& key = index_vector[row];
    const auto& data = file_transfer_rows.at(key);

    switch (static_cast<FileTransfersPanelListColumn>(col)) {
        case FileTransfersPanelListColumn::Filename:
            variant = data.get_filename();
            break;
        case FileTransfersPanelListColumn::Size: {
            variant = Strings::Common::file_size_formatted(data.get_size());
            break;
        }
        case FileTransfersPanelListColumn::Status: {
            // TODO: need to handle not started yet
            const auto direction = data.get_direction();
            const auto percent_progress = data.get_percent_progress();

            variant = Strings::FileTransfersPanel::status_formatted(direction, percent_progress);
            break;
        }
        case FileTransfersPanelListColumn::Receiver:
            variant = data.get_receiver();
            break;
        case FileTransfersPanelListColumn::Sender:
            variant = data.get_sender();
            break;
        case FileTransfersPanelListColumn::Speed: {
            variant = Strings::Common::transfer_speed_formatted(data.get_speed());
            break;
        }
        case FileTransfersPanelListColumn::ETA:
            if (auto eta = data.get_eta(); eta.has_value()) {
                variant = Strings::Common::time_span_formatted(eta.value());
            } else {
                variant = wxString();
            }
            break;
        case FileTransfersPanelListColumn::DateAdded:
            variant = data.get_date_added().FormatISOCombined(' ');
    }
}

void FileTransfersPanelListModel::set_sort_params(
    FileTransfersPanelListColumn column,
    SortDirection direction
) {
    this->sort_column = column;
    this->sort_direction = direction;
}

std::optional<size_t> FileTransfersPanelListModel::sort(std::optional<size_t> selected_index) {
    if (selected_index.has_value()) {
        TEGO_PANIC_IF(selected_index.value() >= index_vector.size());
    }
    this->selected_index = selected_index;

    return this->sort();
}

std::optional<size_t> FileTransfersPanelListModel::sort() {
    const auto sort_impl = [this]() {
        std::sort(
            this->index_vector.begin(),
            this->index_vector.end(),
            [this](const FileTransferKey& a, const FileTransferKey& b) { return this->less(a, b); }
        );
        // trigger the owning data view control to re-grab values from the model
        this->Reset(this->index_vector.size());
    };

    if (this->selected_index.has_value()) {
        // get the key of our selected item
        auto selected_key = this->index_vector[this->selected_index.value()];
        // resort
        sort_impl();

        // find new index of key in index_vector
        auto it_selected =
            std::find(this->index_vector.begin(), this->index_vector.end(), selected_key);
        TEGO_PANIC_IF(it_selected == this->index_vector.end());
        this->selected_index = std::distance(this->index_vector.begin(), it_selected);
        return this->selected_index;
    } else {
        sort_impl();
        return std::nullopt;
    }
}

bool FileTransfersPanelListModel::less(const FileTransferKey& a, const FileTransferKey& b) {
    const auto& row_a = this->file_transfer_rows.at(a);
    const auto& row_b = this->file_transfer_rows.at(b);
    const auto ordering = FileTransferRow::compare(row_a, row_b, this->sort_column);
    bool less;
    switch (ordering) {
        case Ordering::Less:
            less = true;
            break;
        case Ordering::Greater:
            less = false;
            break;
        // in case of tie sort by our key
        case Ordering::Equal:
            less = a < b;
            break;
    }
    return (this->sort_direction == SortDirection::Ascending) ? less : !less;
}

bool FileTransfersPanelListModel::SetValueByRow(
    const wxVariant& variant,
    unsigned int row,
    unsigned int col
) {
    LOG_INFO("SetAttrByRow!");
    return false;
}
