#include "file_transfers_panel.hpp"

#include "enums.hpp"
#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/file_transfers_panel/speed_calculator.hpp"

//
// FileTransferRow
//

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
    file_transfer_rows[key] = std::move(row);

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

    constexpr uint64_t KIBIBYTE = 1024;
    constexpr uint64_t MEBIBYTE = KIBIBYTE * 1024;
    constexpr uint64_t GIBIBYTE = MEBIBYTE * 1024;
    constexpr uint64_t TEBIBYTE = GIBIBYTE * 1024;

    switch (static_cast<FileTransfersPanelListColumn>(col)) {
        case FileTransfersPanelListColumn::Filename:
            variant = data.filename;
            break;
        case FileTransfersPanelListColumn::Size: {
            const auto data_size = data.size;
            if (data_size < KIBIBYTE) {
                variant = fmt::format("{} B", data_size);
            } else if (data_size < MEBIBYTE) {
                variant = fmt::format(
                    "{:.1f} KiB",
                    static_cast<double>(data_size) / static_cast<double>(KIBIBYTE)
                );
            } else if (data_size < GIBIBYTE) {
                variant = fmt::format(
                    "{:.1f} MiB",
                    static_cast<double>(data_size) / static_cast<double>(MEBIBYTE)
                );
            } else if (data_size < TEBIBYTE) {
                variant = fmt::format(
                    "{:.1f} GiB",
                    static_cast<double>(data_size) / static_cast<double>(GIBIBYTE)
                );
            } else {
                variant = fmt::format(
                    "{:.1f} TiB",
                    static_cast<double>(data_size) / static_cast<double>(TEBIBYTE)
                );
            }
            break;
        }
        case FileTransfersPanelListColumn::Status: {
            const auto progress =
                (data.size > 0) ? (double)data.bytes_transferred / data.size : 0.0;
            const auto percent = progress * 100.0;
            variant = fmt::format("{:.1f}%", percent);
            break;
        }
        case FileTransfersPanelListColumn::Receiver:
            variant = data.receiver;
            break;
        case FileTransfersPanelListColumn::Sender:
            variant = data.sender;
            break;
        case FileTransfersPanelListColumn::Speed: {
            const auto data_speed = data.speed;
            if (data_speed < KIBIBYTE) {
                variant = fmt::format("{} B/s", data_speed);
            } else if (data_speed < MEBIBYTE) {
                variant = fmt::format(
                    "{:.1f} KiB/s",
                    static_cast<double>(data_speed) / static_cast<double>(KIBIBYTE)
                );
            } else if (data_speed < GIBIBYTE) {
                variant = fmt::format(
                    "{:.1f} MiB/s",
                    static_cast<double>(data_speed) / static_cast<double>(MEBIBYTE)
                );
            } else if (data_speed < TEBIBYTE) {
                variant = fmt::format(
                    "{:.1f} GiB/s",
                    static_cast<double>(data_speed) / static_cast<double>(GIBIBYTE)
                );
            } else {
                variant = fmt::format(
                    "{:.1f} TiB/s",
                    static_cast<double>(data_speed) / static_cast<double>(TEBIBYTE)
                );
            }
            break;
        }
        case FileTransfersPanelListColumn::ETA:
            variant = data.eta.Format();
            break;
        case FileTransfersPanelListColumn::DateAdded:
            variant = data.date_added.FormatISOCombined(' ');
    }
}

void FileTransfersPanelListModel::sort(
    FileTransfersPanelListColumn column,
    SortDirection direction
) {
    this->sort_column = column;
    this->sort_direction = direction;

    this->sort();
}

void FileTransfersPanelListModel::sort() {
    std::sort(
        index_vector.begin(),
        index_vector.end(),
        [this](const FileTransferKey& a, const FileTransferKey& b) { return this->less(a, b); }
    );

    Reset(index_vector.size());
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

FileTransfersPanel::FileTransfersPanel(wxWindow* parent) : wxPanel(parent) {
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
        static_cast<unsigned int>(FileTransfersPanelListColumn::Receiver),
        wxDATAVIEW_CELL_INERT,
        wxCOL_WIDTH_DEFAULT,
        wxALIGN_LEFT,
        wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE
    );
    file_transfers_data_view_ctrl->AppendTextColumn(
        Strings::FileTransfersPanel::receiver_column_header(),
        static_cast<unsigned int>(FileTransfersPanelListColumn::Sender),
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

    this->list_model->add_file_transfer_row(
        0,
        0,
        0,
        FileTransferRow {
            "Image2.bmp",
            1024,
            512,
            FileTransferDirection::Download,
            "alice",
            "bob",
            50,
            wxTimeSpan(0, 20, 30),
            wxDateTime::Now()
        }
    );

    this->list_model->add_file_transfer_row(
        0,
        0,
        1,
        FileTransferRow {
            "image3.bmp",
            1024 * 1024 * 512,
            1024 * 1024 * 512 * 3 / 4,
            FileTransferDirection::Download,
            "alice",
            "bob",
            100 * 1024,
            wxTimeSpan(0, 0, 30),
            wxDateTime::Now()
        }
    );

    this->list_model->add_file_transfer_row(
        0,
        0,
        2,
        FileTransferRow {
            "image1.bmp",
            128,
            0,
            FileTransferDirection::Upload,
            "alice",
            "bob",
            0,
            wxTimeSpan(1, 20, 30),
            wxDateTime::Now(),
        }
    );

    this->list_model->add_file_transfer_row(
        0,
        0,
        3,
        FileTransferRow {
            "image4.bmp",
            size_t(1024) * size_t(1024) * size_t(1024) * size_t(512) * size_t(4),
            128 * 1024 * 1024,
            FileTransferDirection::Download,
            "alice",
            "bob",
            100 * 1024 * 1024,
            wxTimeSpan(0, 0, 30),
            wxDateTime::Now()
        }
    );

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
}

void FileTransfersPanel::on_sort(const wxDataViewEvent& event) {
    const auto column_index = event.GetColumn();

    auto* column = file_transfers_data_view_ctrl->GetColumn(column_index);
    bool ascending = column->IsSortOrderAscending();
    column->SetSortOrder(ascending);

    const auto sort_column = static_cast<FileTransfersPanelListColumn>(column_index);
    const auto sort_direction = ascending ? SortDirection::Ascending : SortDirection::Descending;

    this->list_model->sort(sort_column, sort_direction);
}

void FileTransfersPanel::close() {
    wxGetApp().get_main_frame().hide_overlay_panel();
}
