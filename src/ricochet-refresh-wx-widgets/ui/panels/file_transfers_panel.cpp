#include "file_transfers_panel.hpp"

#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"

FileTransfersPanel::FileTransfersPanel(wxWindow* parent) : wxPanel(parent) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto title = new wxStaticText(this, wxID_ANY, Strings::FileTransferPanel::title());
    title->SetFont(Fonts::title_font());

    // buttons

    auto button_sizer = new wxBoxSizer(wxHORIZONTAL);
    auto close_button = new wxButton(this, wxID_OK, Strings::FileTransferPanel::close_button());
    close_button->Bind(wxEVT_BUTTON, [this](wxCommandEvent&) { this->close(); });

    button_sizer->AddStretchSpacer(1);
    button_sizer->Add(close_button, 0);

    v_sizer->Add(title, 0, wxEXPAND | wxALL, Metrics::PADDING_MEDIUM);
    v_sizer->AddStretchSpacer(1);
    v_sizer->Add(button_sizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);
    this->SetSizerAndFit(v_sizer);
}

void FileTransfersPanel::close() {
    LOG_INFO("FileTransfersPanel Close");

    wxGetApp().get_main_frame().hide_overlay_panel();
}
