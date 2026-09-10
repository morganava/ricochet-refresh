#include "file_transfers_panel.hpp"

#include "enums.hpp"
#include "ffi.hpp"
#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/file_transfers_panel/file_transfer_details_panel.hpp"
#include "ui/panels/file_transfers_panel/file_transfer_row.hpp"
#include "ui/panels/file_transfers_panel/file_transfers_panel_list_model.hpp"

//
// FileTransferPanel
//

FileTransfersPanel::FileTransfersPanel(wxWindow* parent) : wxPanel(parent), update_timer(this) {
    this->v_sizer = new wxBoxSizer(wxVERTICAL);

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
    // bind row select event
    file_transfers_data_view_ctrl
        ->Bind(wxEVT_DATAVIEW_SELECTION_CHANGED, &FileTransfersPanel::on_row_select, this);

    // buttons

    auto button_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto close_button = new wxButton(this, wxID_OK, Strings::FileTransfersPanel::close_button());
    close_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->close(); });

    button_sizer->AddStretchSpacer(1);
    button_sizer->Add(close_button, 0);

    this->v_sizer->Add(title, 0, wxEXPAND | wxALL, Metrics::PADDING_MEDIUM);
    this->v_sizer->Add(
        this->file_transfers_data_view_ctrl,
        2,
        wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM,
        Metrics::PADDING_MEDIUM
    );
    this->v_sizer
        ->Add(button_sizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);
    this->SetSizerAndFit(this->v_sizer);

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
    const auto key = std::make_tuple(session_handle, user_handle, id);
    // first update the list model
    this->list_model->add_file_transfer_row(
        key,
        FileTransferRow(filename, size, direction, sender, receiver, wxDateTime::Now())
    );
    // then create our hidden details panel
    auto* details_panel = new FileTransferDetailsPanel(this);
    this->details_panels.insert(std::make_pair(key, details_panel));
    this->v_sizer->Insert(
        FILE_TRANSFER_DETAILS_INDEX,
        details_panel,
        0,
        wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM,
        Metrics::PADDING_MEDIUM
    );
    this->v_sizer->Hide(details_panel);
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
    // update our sort column
    const auto column_index = event.GetColumn();

    auto* column = this->file_transfers_data_view_ctrl->GetColumn(column_index);
    bool ascending = column->IsSortOrderAscending();
    column->SetSortOrder(ascending);

    const auto sort_column = static_cast<FileTransfersPanelListColumn>(column_index);
    const auto sort_direction = ascending ? SortDirection::Ascending : SortDirection::Descending;
    this->list_model->set_sort_params(sort_column, sort_direction);

    this->sort_rows();
}

void FileTransfersPanel::on_row_select(const wxDataViewEvent& event) {
    const auto data_view_item = this->file_transfers_data_view_ctrl->GetSelection();
    const auto row_index = this->list_model->GetRow(data_view_item);

    // undocumented but this appears to be the behaviour :(
    constexpr unsigned int INVALID_ROW = std::numeric_limits<unsigned int>::max();

    // hide the current details panel
    if (this->selected_file_transfer.has_value()) {
        auto it = this->details_panels.find(this->selected_file_transfer.value());
        TEGO_PANIC_IF(it == this->details_panels.end());
        auto* details_panel = it->second;
        this->v_sizer->Hide(details_panel);
        this->selected_file_transfer = std::nullopt;
    }

    // determine which details panel to show
    if (row_index != INVALID_ROW) {
        // a valid selected row
        const auto file_transfer_key = this->list_model->get_file_transfer_key(row_index);
        this->selected_file_transfer = file_transfer_key;

        // find our panel, show it, and save off
        auto it = this->details_panels.find(file_transfer_key);
        TEGO_PANIC_IF(it == this->details_panels.end());
        auto* details_panel = it->second;
        this->v_sizer->Show(details_panel);
    } else {
        // no row selected
        this->selected_file_transfer = std::nullopt;
    }
    this->v_sizer->Layout();

    LOG_INFO(fmt::format("on_row_select: {}", row_index));
}

void FileTransfersPanel::on_timer_event() {
    // convert raw data to rendered data
    this->list_model->update(wxDateTime::Now());

    this->sort_rows();
}

void FileTransfersPanel::sort_rows() {
    // keep track of our selected row
    const auto selection = this->file_transfers_data_view_ctrl->GetSelection();
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
