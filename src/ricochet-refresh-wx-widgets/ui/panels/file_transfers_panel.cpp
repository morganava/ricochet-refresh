#include "file_transfers_panel.hpp"

#include "enums.hpp"
#include "ffi.hpp"
#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/file_transfers_panel/speed_calculator.hpp"

//
// FileTransferRow
//

class FileTransferRow {
public:
    FileTransferRow() = default;
    FileTransferRow(
        wxString filename,
        uint64_t size,
        FileTransferDirection direction,
        wxString sender,
        wxString receiver,
        wxDateTime now
    );

    //
    // Getters
    //
    const wxString& get_filename() const {
        return this->filename;
    }

    uint64_t get_size() const {
        return this->size;
    }

    uint64_t get_bytes_transferred() const {
        return this->bytes_transferred;
    }

    FileTransferDirection get_direction() const {
        return this->direction;
    }

    double get_percent_progress() const {
        // special-case completion just in case that static cast + division doesn't work out to 100.0 exactly
        // it *should* but sometimes floating point is weird vOv
        if (this->bytes_transferred == this->size) {
            return 100.0;
        }
        return static_cast<double>(this->bytes_transferred) / static_cast<double>(this->size)
            * 100.0;
    }

    const wxString& get_sender() const {
        return this->sender;
    }

    const wxString& get_receiver() const {
        return this->receiver;
    }

    double get_speed() const {
        return this->speed;
    }

    const std::optional<wxTimeSpan>& get_eta() const {
        return this->eta;
    }

    const wxDateTime& get_date_added() const {
        return this->date_added;
    }

    //
    // Compare for sorting
    //
    static Ordering compare(
        const FileTransferRow& row1,
        const FileTransferRow& row2,
        FileTransfersPanelListColumn column
    );

    //
    // Update methods
    //

    void update(tego_time now);
    void update_bytes_transferred(uint64_t bytes_transferred, tego_time now);

private:
    wxString filename;
    uint64_t size;
    uint64_t bytes_transferred;
    FileTransferDirection direction;
    wxString sender;
    wxString receiver;
    double speed; // bytes per second
    std::optional<wxTimeSpan> eta;
    wxDateTime date_added;

    constexpr static size_t SAMPLES = 32;
    constexpr static size_t TIME_DELTA_MS = 5000; // 5 seconds

    AverageSpeedCalculator<SAMPLES, TIME_DELTA_MS> speed_calculator;
};

FileTransferRow::FileTransferRow(
    wxString filename,
    uint64_t size,
    FileTransferDirection direction,
    wxString sender,
    wxString receiver,
    wxDateTime now
) :
    filename(filename),
    size(size),
    bytes_transferred(0),
    direction(direction),
    sender(sender),
    receiver(receiver),
    speed(0.0),
    eta({}),
    date_added(now),
    speed_calculator(into_tego_time(now)) {}

template<typename T>
Ordering compare(const T& a, const T& b) {
    if (a < b) {
        return Ordering::Less;
    } else if (a > b) {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

template<>
Ordering compare<wxString>(const wxString& a, const wxString& b) {
    if (auto cmp = a.CmpNoCase(b); cmp < 0) {
        return Ordering::Less;
    } else if (cmp > 0) {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

template<>
Ordering compare<wxDateTime>(const wxDateTime& a, const wxDateTime& b) {
    if (a.IsEarlierThan(b)) {
        return Ordering::Less;
    } else if (a.IsLaterThan(b)) {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

Ordering FileTransferRow::compare(
    const FileTransferRow& row1,
    const FileTransferRow& row2,
    FileTransfersPanelListColumn column
) {
    switch (column) {
        case FileTransfersPanelListColumn::Filename:
            return ::compare(row1.filename, row2.filename);
        case FileTransfersPanelListColumn::Size:
            return ::compare(row1.size, row2.size);
        case FileTransfersPanelListColumn::Status:
            return ::compare(
                static_cast<double>(row1.bytes_transferred) / static_cast<double>(row1.size),
                static_cast<double>(row2.bytes_transferred) / static_cast<double>(row2.size)
            );
        case FileTransfersPanelListColumn::Receiver:
            return ::compare(row1.receiver, row2.receiver);
        case FileTransfersPanelListColumn::Sender:
            return ::compare(row1.sender, row2.sender);
        case FileTransfersPanelListColumn::Speed:
            return ::compare(row1.speed, row2.speed);
        case FileTransfersPanelListColumn::ETA:
            return ::compare(row1.eta, row2.eta);
        case FileTransfersPanelListColumn::DateAdded:
            return ::compare(row1.date_added, row2.date_added);
    }
    tego::panic(fmt::format("Unknown column type: {}", static_cast<unsigned int>(column)));
}

void FileTransferRow::update(tego_time now) {
    this->speed = this->speed_calculator.get_average_speed(now);

    if (this->speed > 0.0 && this->speed < std::numeric_limits<double>::infinity()) {
        const auto remaining = this->size - this->bytes_transferred;
        const auto eta_milliseconds =
            static_cast<uint64_t>(static_cast<double>(remaining) / this->speed * 1000.0);
        this->eta = wxTimeSpan::Milliseconds(eta_milliseconds);
    } else {
        this->eta = std::nullopt;
    }
}

void FileTransferRow::update_bytes_transferred(uint64_t bytes_transferred, tego_time now) {
    TEGO_PANIC_IF(bytes_transferred < this->bytes_transferred);
    this->bytes_transferred = bytes_transferred;
    this->speed_calculator.add_record(bytes_transferred, now);
}

//
// FileTransferPanelListModel
//

FileTransfersPanelListModel::FileTransfersPanelListModel() :
    wxDataViewVirtualListModel(),
    sort_column(FileTransfersPanelListColumn::DateAdded),
    sort_direction(SortDirection::Ascending) {}

void FileTransfersPanelListModel::add_file_transfer_row(
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id file_transfer_id,
    FileTransferRow&& row
) {
    const auto key = std::make_tuple(session_handle, user_handle, file_transfer_id);
    file_transfer_rows[key] = row;

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

    constexpr double KIBIBYTE = 1024.0;
    constexpr double MEBIBYTE = KIBIBYTE * 1024.0;
    constexpr double GIBIBYTE = MEBIBYTE * 1024.0;
    constexpr double TEBIBYTE = GIBIBYTE * 1024.0;

    switch (static_cast<FileTransfersPanelListColumn>(col)) {
        case FileTransfersPanelListColumn::Filename:
            variant = data.get_filename();
            break;
        case FileTransfersPanelListColumn::Size: {
            const auto data_size = static_cast<double>(data.get_size());
            if (data_size < KIBIBYTE) {
                variant = fmt::format("{} B", data.get_size());
            } else if (data_size < MEBIBYTE) {
                variant = fmt::format("{:.1f} KiB", data_size / KIBIBYTE);
            } else if (data_size < GIBIBYTE) {
                variant = fmt::format("{:.1f} MiB", data_size / MEBIBYTE);
            } else if (data_size < TEBIBYTE) {
                variant = fmt::format("{:.1f} GiB", data_size / GIBIBYTE);
            } else {
                variant = fmt::format("{:.1f} TiB", data_size / TEBIBYTE);
            }
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
            const auto data_speed = data.get_speed();
            if (data_speed < KIBIBYTE) {
                variant = fmt::format("{} B/s", data.get_speed());
            } else if (data_speed < MEBIBYTE) {
                variant = fmt::format("{:.1f} KiB/s", data_speed / KIBIBYTE);
            } else if (data_speed < GIBIBYTE) {
                variant = fmt::format("{:.1f} MiB/s", data_speed / MEBIBYTE);
            } else if (data_speed < TEBIBYTE) {
                variant = fmt::format("{:.1f} GiB/s", data_speed / GIBIBYTE);
            } else {
                variant = fmt::format("{:.1f} TiB/s", data_speed / TEBIBYTE);
            }
            break;
        }
        case FileTransfersPanelListColumn::ETA:
            if (auto eta = data.get_eta(); eta.has_value()) {
                variant = Strings::FileTransfersPanel::eta_formatted(eta.value());
            } else {
                variant = wxString();
            }
            break;
        case FileTransfersPanelListColumn::DateAdded:
            variant = data.get_date_added().FormatISOCombined(' ');
    }
}

std::optional<size_t> FileTransfersPanelListModel::sort(
    FileTransfersPanelListColumn column,
    SortDirection direction,
    std::optional<size_t> selected_index
) {
    this->sort_column = column;
    this->sort_direction = direction;

    return this->sort(selected_index);
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
    auto& row_a = this->file_transfer_rows.at(a);
    auto& row_b = this->file_transfer_rows.at(b);
    auto ordering = FileTransferRow::compare(row_a, row_b, this->sort_column);
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

//
// FileTransferPanel
//

FileTransfersPanel::FileTransfersPanel(wxWindow* parent) : wxPanel(parent), update_timer(this) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto title = new wxStaticText(this, wxID_ANY, Strings::FileTransfersPanel::title());
    title->SetFont(Fonts::title_font());

    auto file_transfers_data_view_ctrl = new wxDataViewCtrl(
        this,
        wxID_ANY,
        wxDefaultPosition,
        wxDefaultSize,
        wxDV_SINGLE | wxDV_ROW_LINES
    );
    file_transfers_data_view_ctrl->AppendTextColumn(
        Strings::FileTransfersPanel::filename_column_header(),
        static_cast<unsigned int>(FileTransfersPanelListColumn::Filename),
        wxDATAVIEW_CELL_INERT,
        wxCOL_WIDTH_DEFAULT,
        wxALIGN_LEFT,
        wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE
    );
    file_transfers_data_view_ctrl->AppendTextColumn(
        Strings::FileTransfersPanel::size_column_header(),
        static_cast<unsigned int>(FileTransfersPanelListColumn::Size),
        wxDATAVIEW_CELL_INERT,
        wxCOL_WIDTH_DEFAULT,
        wxALIGN_LEFT,
        wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE
    );
    file_transfers_data_view_ctrl->AppendTextColumn(
        Strings::FileTransfersPanel::status_column_header(),
        static_cast<unsigned int>(FileTransfersPanelListColumn::Status),
        wxDATAVIEW_CELL_INERT,
        wxCOL_WIDTH_DEFAULT,
        wxALIGN_LEFT,
        wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE
    );
    file_transfers_data_view_ctrl->AppendTextColumn(
        Strings::FileTransfersPanel::sender_column_header(),
        static_cast<unsigned int>(FileTransfersPanelListColumn::Sender),
        wxDATAVIEW_CELL_INERT,
        wxCOL_WIDTH_DEFAULT,
        wxALIGN_LEFT,
        wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE
    );
    file_transfers_data_view_ctrl->AppendTextColumn(
        Strings::FileTransfersPanel::receiver_column_header(),
        static_cast<unsigned int>(FileTransfersPanelListColumn::Receiver),
        wxDATAVIEW_CELL_INERT,
        wxCOL_WIDTH_DEFAULT,
        wxALIGN_LEFT,
        wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE
    );
    file_transfers_data_view_ctrl->AppendTextColumn(
        Strings::FileTransfersPanel::speed_column_header(),
        static_cast<unsigned int>(FileTransfersPanelListColumn::Speed),
        wxDATAVIEW_CELL_INERT,
        wxCOL_WIDTH_DEFAULT,
        wxALIGN_LEFT,
        wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE
    );
    file_transfers_data_view_ctrl->AppendTextColumn(
        Strings::FileTransfersPanel::eta_column_header(),
        static_cast<unsigned int>(FileTransfersPanelListColumn::ETA),
        wxDATAVIEW_CELL_INERT,
        wxCOL_WIDTH_DEFAULT,
        wxALIGN_LEFT,
        wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE
    );
    file_transfers_data_view_ctrl->AppendTextColumn(
        Strings::FileTransfersPanel::date_added_column_header(),
        static_cast<unsigned int>(FileTransfersPanelListColumn::DateAdded),
        wxDATAVIEW_CELL_INERT,
        wxCOL_WIDTH_DEFAULT,
        wxALIGN_LEFT,
        wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE
    );
    this->list_model = new FileTransfersPanelListModel();
    file_transfers_data_view_ctrl->AssociateModel(this->list_model.get());
    this->file_transfers_data_view_ctrl = file_transfers_data_view_ctrl;

    // bind sorting event
    file_transfers_data_view_ctrl
        ->Bind(wxEVT_DATAVIEW_COLUMN_HEADER_CLICK, &FileTransfersPanel::on_sort, this);

    // buttons

    auto button_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto close_button = new wxButton(this, wxID_OK, Strings::FileTransfersPanel::close_button());
    close_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->close(); });

    button_sizer->AddStretchSpacer(1);
    button_sizer->Add(close_button, 0);

    v_sizer->Add(title, 0, wxEXPAND | wxALL, Metrics::PADDING_MEDIUM);
    v_sizer->Add(
        this->file_transfers_data_view_ctrl,
        2,
        wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM,
        Metrics::PADDING_MEDIUM
    );
    v_sizer->AddStretchSpacer(1);
    v_sizer->Add(button_sizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);
    this->SetSizerAndFit(v_sizer);

    // Regularly update all of our records
    this->Bind(wxEVT_TIMER, [this](const wxTimerEvent&) { this->on_timer_event(); });
    // we want stable downloads to update at a rate of 1hz, so we sample at 2hz (i.e once every 500ms)
    this->update_timer.Start(500, false);
}

void FileTransfersPanel::add_file_transfer(
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id id,
    wxString filename,
    uint64_t size,
    FileTransferDirection direction,
    wxString sender,
    wxString receiver
) {
    this->list_model->add_file_transfer_row(
        session_handle,
        user_handle,
        id,
        FileTransferRow(filename, size, direction, sender, receiver, wxDateTime::Now())
    );
}

void FileTransfersPanel::update_file_transfer_progress(
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id id,
    uint64_t bytes_transferred
) {
    this->list_model->update_file_transfer_row(session_handle, user_handle, id, bytes_transferred);
}

void FileTransfersPanel::on_sort(const wxDataViewEvent& event) {
    const auto column_index = event.GetColumn();

    auto* column = file_transfers_data_view_ctrl->GetColumn(column_index);
    bool ascending = column->IsSortOrderAscending();
    column->SetSortOrder(ascending);

    const auto sort_column = static_cast<FileTransfersPanelListColumn>(column_index);
    const auto sort_direction = ascending ? SortDirection::Ascending : SortDirection::Descending;

    // keep track of our selected row
    auto selection = this->file_transfers_data_view_ctrl->GetSelection();
    std::optional<size_t> selected_index = selection.IsOk()
        ? std::optional(static_cast<size_t>(this->list_model->GetRow(selection)))
        : std::nullopt;

    selected_index = this->list_model->sort(sort_column, sort_direction, selected_index);
    if (selected_index.has_value()) {
        wxDataViewItemArray item_array;
        item_array.Add(this->list_model->GetItem(selected_index.value()));
        this->file_transfers_data_view_ctrl->SetSelections(item_array);
    }
}

void FileTransfersPanel::on_timer_event() {
    this->list_model->update(wxDateTime::Now());

    // keep track of our selected row
    auto selection = this->file_transfers_data_view_ctrl->GetSelection();
    std::optional<size_t> selected_index = selection.IsOk()
        ? std::optional(static_cast<size_t>(this->list_model->GetRow(selection)))
        : std::nullopt;

    selected_index = this->list_model->sort(selected_index);
    if (selected_index.has_value()) {
        wxDataViewItemArray item_array;
        item_array.Add(this->list_model->GetItem(selected_index.value()));
        this->file_transfers_data_view_ctrl->SetSelections(item_array);
    }
}

void FileTransfersPanel::close() {
    wxGetApp().get_main_frame().hide_overlay_panel();
}
