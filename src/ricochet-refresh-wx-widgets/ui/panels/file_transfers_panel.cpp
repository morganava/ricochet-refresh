#include "file_transfers_panel.hpp"

#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"

//
// FileTransferPanelListModel
//

enum class FileTransfersPanelListColumn : unsigned int {
    Filename = 0,
    Size,
    Status,
    Receiver,
    Sender,
    Speed,
    ETA,
};

FileTransfersPanelListModel::FileTransfersPanelListModel() : wxDataViewVirtualListModel() {}

void FileTransfersPanelListModel::AddFileTransferRow(
    tego_session_handle session_handle,
    tego_user_handle user_handle,
    tego_file_transfer_id file_transfer_id,
    FileTransferRow&& row
) {
    auto key = std::make_tuple(session_handle, user_handle, file_transfer_id);
    file_transfer_rows[key] = std::move(row);
    index_vector.push_back(key);
    RowAppended();
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
            variant = data.filename;
            break;
        case FileTransfersPanelListColumn::Size:
            variant = wxString::Format("%llu bytes", data.size);
            break;
        case FileTransfersPanelListColumn::Status: {
            double progress = (data.size > 0) ? (double)data.bytes_transferred / data.size : 0.0;
            variant = wxString::Format("%.1f%%", progress * 100.0);
            break;
        }
        case FileTransfersPanelListColumn::Receiver:
            variant = data.receiver;
            break;
        case FileTransfersPanelListColumn::Sender:
            variant = data.sender;
            break;
        case FileTransfersPanelListColumn::Speed:
            variant = wxString::Format("%llu B/s", data.speed);
            break;
        case FileTransfersPanelListColumn::ETA:
            variant = data.eta.ToString();
            break;
    }
}

int FileTransfersPanelListModel::Compare(
    const wxDataViewItem& item1,
    const wxDataViewItem& item2,
    unsigned int column,
    bool ascending
) const {
    // IDs are row indices
    unsigned int row1 = static_cast<unsigned int>(item1.GetID());
    unsigned int row2 = static_cast<unsigned int>(item2.GetID());

    // Map row index -> map key -> data
    const auto& key1 = index_vector[row1];
    const auto& key2 = index_vector[row2];

    const auto& s1 = file_transfer_rows.at(key1);
    const auto& s2 = file_transfer_rows.at(key2);

    int result = 0;
    switch (static_cast<FileTransfersPanelListColumn>(column)) {
        case FileTransfersPanelListColumn::Filename:
            result = (s1.filename < s2.filename) ? -1 : (s1.filename > s2.filename ? 1 : 0);
            break;
        case FileTransfersPanelListColumn::Size:
            result = (s1.size < s2.size) ? -1 : (s1.size > s2.size ? 1 : 0);
            break;
        case FileTransfersPanelListColumn::Status:
            result = (s1.bytes_transferred < s2.bytes_transferred)
                ? -1
                : (s1.bytes_transferred > s2.bytes_transferred ? 1 : 0);
            break;
        case FileTransfersPanelListColumn::Receiver:
            result = (s1.receiver < s2.receiver) ? -1 : (s1.receiver > s2.receiver ? 1 : 0);
            break;
        case FileTransfersPanelListColumn::Sender:
            result = (s1.sender < s2.sender) ? -1 : (s1.sender > s2.sender ? 1 : 0);
            break;
        case FileTransfersPanelListColumn::Speed:
            result = (s1.speed < s2.speed) ? -1 : (s1.speed > s2.speed ? 1 : 0);
            break;
        case FileTransfersPanelListColumn::ETA:
            result = (s1.eta < s2.eta) ? -1 : (s1.eta > s2.eta ? 1 : 0);
            break;
    }

    return ascending ? result : -result;
}

void FileTransfersPanelListModel::Sort(unsigned int column, bool ascending) {
    std::sort(
        index_vector.begin(),
        index_vector.end(),
        [this, column, ascending](const auto& a, const auto& b) {
            // Wrap keys in items to reuse the Compare logic
            return this->Compare(
                       wxDataViewItem((void*)&a),
                       wxDataViewItem((void*)&b),
                       column,
                       ascending
                   )
                < 0;
        }
    );
    Reset(index_vector.size());
}

bool FileTransfersPanelListModel::GetAttrByRow(
    unsigned int row,
    unsigned int col,
    wxDataViewItemAttr& attr
) const {
    return false;
}

bool FileTransfersPanelListModel::SetValueByRow(
    const wxVariant& variant,
    unsigned int row,
    unsigned int col
) {
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
    this->list_model = new FileTransfersPanelListModel();
    file_transfers_data_view_ctrl->AssociateModel(this->list_model.get());

    this->file_transfers_data_view_ctrl = file_transfers_data_view_ctrl;

    // Bind sorting event
    file_transfers_data_view_ctrl->Bind(
        wxEVT_DATAVIEW_COLUMN_HEADER_CLICK,
        [this](wxDataViewEvent& event) {
            unsigned int col = event.GetColumn();
            auto* column = file_transfers_data_view_ctrl->GetColumn(col);

            // Toggle the visual arrow and tell the model to sort
            bool ascending = !column->IsSortOrderAscending();
            column->SetSortOrder(ascending);

            this->list_model->Sort(col, ascending);
        }
    );

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

void FileTransfersPanel::close() {
    LOG_INFO("FileTransfersPanel Close");

    wxGetApp().get_main_frame().hide_overlay_panel();
}
