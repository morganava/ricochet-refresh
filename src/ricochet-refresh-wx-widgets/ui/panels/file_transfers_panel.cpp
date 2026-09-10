#include "file_transfers_panel.hpp"

#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"

FileTransfersPanel::FileTransfersPanel(wxWindow* parent) : wxPanel(parent) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto title = new wxStaticText(this, wxID_ANY, Strings::FileTransfersPanel::title());
    title->SetFont(Fonts::title_font());

    auto file_transfers_data_view_ctrl = new wxDataViewCtrl(this, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxDV_SINGLE | wxDV_ROW_LINES);
    {
        wxDataViewTextRenderer *tr =
            new wxDataViewTextRenderer( "string", wxDATAVIEW_CELL_INERT );
        wxDataViewColumn *column0 =
            new wxDataViewColumn( "title", tr, 0, FromDIP(200), wxALIGN_LEFT, wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE );
        file_transfers_data_view_ctrl->AppendColumn(column0);
    }
    {
        wxDataViewTextRenderer *tr =
            new wxDataViewTextRenderer( "string", wxDATAVIEW_CELL_INERT );
        wxDataViewColumn *column0 =
            new wxDataViewColumn( "title", tr, 0, FromDIP(200), wxALIGN_LEFT, wxDATAVIEW_COL_SORTABLE | wxDATAVIEW_COL_RESIZABLE );
        file_transfers_data_view_ctrl->AppendColumn(column0);
    }
    // file_transfers_data_view_ctrl->AppendTextColumn(Strings::FileTransfersPanel::size_column_header());
    // file_transfers_data_view_ctrl->AppendTextColumn(Strings::FileTransfersPanel::status_column_header());
    // file_transfers_data_view_ctrl->AppendTextColumn(Strings::FileTransfersPanel::sender_column_header());
    // file_transfers_data_view_ctrl->AppendTextColumn(Strings::FileTransfersPanel::receiver_column_header());
    // file_transfers_data_view_ctrl->AppendTextColumn(Strings::FileTransferPanel::speed_column_header());
    // file_transfers_data_view_ctrl->AppendTextColumn(Strings::FileTransfersPanel::eta_column_header());

    this->file_transfers_data_view_ctrl = file_transfers_data_view_ctrl;

    // buttons

    auto button_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto close_button = new wxButton(this, wxID_OK, Strings::FileTransfersPanel::close_button());
    close_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->close(); });

    button_sizer->AddStretchSpacer(1);
    button_sizer->Add(close_button, 0);

    v_sizer->Add(title, 0, wxEXPAND | wxALL, Metrics::PADDING_MEDIUM);
    v_sizer->Add(this->file_transfers_data_view_ctrl, 2, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);
    v_sizer->AddStretchSpacer(1);
    v_sizer->Add(button_sizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);
    this->SetSizerAndFit(v_sizer);
}

void FileTransfersPanel::close() {
    LOG_INFO("FileTransfersPanel Close");

    wxGetApp().get_main_frame().hide_overlay_panel();
}
